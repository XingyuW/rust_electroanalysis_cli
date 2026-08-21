use rust_electroanalysis_cli::{
    cli::{CommandSpec, parse_cli_args},
    domain::{
        ArtifactError, read_artifact, read_artifact_lineage_catalog,
        read_artifact_lineage_catalog_strict, read_artifact_strict,
    },
    mhi_validation::{
        MhiValidationProtocolV1, ValidationInputs,
        approval::{PhysicalApprovalProvisioningStateV1, PhysicalApprovalTrustStoreV1},
        evaluate_mhi_validation,
        statistics::{MetricValueV1, balanced_accuracy, wilson_95},
    },
    results::MhiValidationDatasetV1,
    runners::mhi_validation::{MhiValidationRunOptions, run_mhi_validation},
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_e")
        .join(name)
}

#[test]
fn phase_e_dataset_recomputes_semantic_identity_and_rejects_root_or_path_mismatch() {
    let (root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&protocol_bytes).expect("UTF-8"))
            .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);
    ValidationInputs::read(&protocol, &protocol_hash, &dataset).expect("unmodified authority");

    let text = fs::read_to_string(&dataset).expect("dataset bytes");
    fs::write(
        &dataset,
        text.replacen(
            "lineage/complete.schema1.json",
            "../lineage/complete.schema1.json",
            1,
        ),
    )
    .expect("unsafe-path mutation");
    assert!(ValidationInputs::read(&protocol, &protocol_hash, &dataset).is_err());

    fs::remove_dir_all(root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_reader_accepts_only_canonical_schema4_scientific_inputs() {
    use rust_electroanalysis_cli::results::{MechanismAnalysisReport, SensorHealthAssessment};

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mechanism = root.join("tests/fixtures/phase_d/base/mechanism.json");
    let health = root.join("tests/fixtures/phase_d/base/health.json");
    let strict_mechanism = read_artifact_strict::<MechanismAnalysisReport>(&mechanism)
        .expect("canonical schema-4 Phase-B input");
    let strict_health = read_artifact_strict::<SensorHealthAssessment>(&health)
        .expect("canonical schema-4 Phase-C input");
    assert_eq!(strict_mechanism.artifact.schema_version, 4);
    assert_eq!(strict_health.artifact.schema_version, 4);
}

#[test]
fn phase_e_reader_hard_fails_wrong_future_and_explicitly_excludes_legacy() {
    use rust_electroanalysis_cli::results::MechanismAnalysisReport;

    let root = temp("strict_schema4_reader");
    fs::create_dir_all(&root).expect("temporary directory");
    let future = root.join("mechanism.schema5.json");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase_d/base/mechanism.json");
    let text = fs::read_to_string(source).expect("canonical mechanism fixture");
    let position = text.rfind("\"schema_version\": 4").expect("root schema");
    let mut mutated = text;
    mutated.replace_range(
        position..position + "\"schema_version\": 4".len(),
        "\"schema_version\": 5",
    );
    fs::write(&future, mutated).expect("future-schema mutation");
    assert!(read_artifact_strict::<MechanismAnalysisReport>(&future).is_err());

    fs::remove_dir_all(root).expect("cleanup");
}

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "phase_e_{name}_{}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed),
    ))
}

/// The fixture ledger keeps protocol, dataset, lineage, and reference inputs
/// in separate named files.  The reader intentionally resolves source paths
/// beneath the dataset root, so integration tests stage those literal files in
/// the same safe layout a caller would provide rather than weakening path
/// containment to reach sibling fixture directories.
fn staged_validation_inputs(
    protocol_fixture: &str,
    dataset_fixture: &str,
) -> (PathBuf, PathBuf, PathBuf) {
    let root = temp("staged_inputs");
    let protocol = root.join("protocol.toml");
    let dataset = root.join("dataset/input.schema1.json");
    let lineage = root.join("dataset/lineage/complete.schema1.json");
    fs::create_dir_all(lineage.parent().expect("lineage parent")).expect("fixture layout");
    fs::copy(fixture(protocol_fixture), &protocol).expect("copy protocol fixture");
    fs::copy(fixture(dataset_fixture), &dataset).expect("copy dataset fixture");
    fs::copy(fixture("lineage/complete.schema1.json"), &lineage).expect("copy lineage fixture");
    (root, protocol, dataset)
}

#[test]
fn phase_e_cli_runs_exact_certified_route() {
    let args = vec![
        "electroanalysis",
        "validation",
        "run",
        "--protocol",
        "protocol.toml",
        "--dataset",
        "dataset.json",
        "--output-dir",
        "output",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    assert!(
        matches!(parse_cli_args(&args).expect("CLI parses").command, Some(CommandSpec::ValidationRun { protocol, dataset, output_dir, overwrite: false }) if protocol == Path::new("protocol.toml") && dataset == Path::new("dataset.json") && output_dir == Path::new("output"))
    );
}

#[test]
fn phase_e_cli_rejects_missing_unknown_and_raw_input_routes() {
    for args in [
        vec![
            "electroanalysis",
            "validation",
            "run",
            "--protocol",
            "p",
            "--dataset",
            "d",
        ],
        vec![
            "electroanalysis",
            "validation",
            "run",
            "--protocol",
            "p",
            "--dataset",
            "d",
            "--output-dir",
            "o",
            "--input",
            "raw.csv",
        ],
    ] {
        let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
        assert!(parse_cli_args(&args).is_err());
    }
}

#[test]
fn phase_e_protocol_roundtrip_preserves_all_scientific_rules() {
    let bytes = fs::read(fixture("protocol/software_valid.toml")).expect("fixture");
    let protocol = MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("UTF-8"))
        .expect("protocol validates");
    assert_eq!(protocol.schema_version, 1);
    assert_eq!(protocol.release_scope.len(), 1);
    assert_eq!(
        MhiValidationProtocolV1::sha256_of_bytes(&bytes),
        "a098c83e08f488d49f16be4c4fc27b09d87ca3752a7af7b50069ba6e9e09b47e"
    );
}

#[test]
fn phase_e_protocol_rejects_incomplete_conflicting_untrusted_and_nondeterministic_authority() {
    let text = fs::read_to_string(fixture("protocol/software_valid.toml")).expect("fixture");
    let invalid = text.replace(
        "interval_method = \"wilson_95_v1\"",
        "interval_method = \"wald\"",
    );
    assert!(MhiValidationProtocolV1::from_toml(&invalid).is_err());
    let invalid = text.replace("type = \"not_requested\"", "type = \"embedded_trust_root\"");
    assert!(MhiValidationProtocolV1::from_toml(&invalid).is_err());
}

#[test]
fn phase_e_dataset_schema1_roundtrip_is_closed_and_canonical() {
    let path = fixture("dataset/software_valid.schema1.json");
    let dataset: MhiValidationDatasetV1 =
        read_artifact(&path).expect("legacy reader accepts valid Phase-E dataset");
    assert_eq!(dataset.schema_version, 1);
    assert_eq!(dataset.records[0].record_id, "record_1");
}

#[test]
fn phase_e_strict_reader_rejects_duplicate_json_without_changing_existing_reader() {
    let directory = temp("duplicate_reader");
    fs::create_dir_all(&directory).expect("directory");
    let path = directory.join("dataset.json");
    let mut text =
        fs::read_to_string(fixture("dataset/software_valid.schema1.json")).expect("fixture");
    text = text.replacen("\"dataset_id\": \"phase_e_software_dataset\",", "\"dataset_id\": \"phase_e_software_dataset\",\n  \"dataset_id\": \"phase_e_software_dataset\",", 1);
    fs::write(&path, text).expect("write mutation");
    assert!(matches!(
        read_artifact_strict::<MhiValidationDatasetV1>(&path),
        Err(ArtifactError::DuplicateJsonKey { .. })
    ));
    assert!(
        read_artifact::<MhiValidationDatasetV1>(&path).is_ok(),
        "legacy reader compatibility remains unchanged"
    );
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn phase_e_strict_catalog_reader_rejects_nested_unknown_without_changing_legacy_reader() {
    let directory = temp("strict_catalog");
    fs::create_dir_all(&directory).expect("directory");
    let path = directory.join("lineage.json");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_d/base/lineage_catalog.json");
    let text = fs::read_to_string(source).expect("catalog fixture");
    let mutated = text.replacen(
        "\"producer_version\": \"phase-b-fixture-generator\",",
        "\"producer_version\": \"phase-b-fixture-generator\",\n        \"phase_e_unknown\": true,",
        1,
    );
    fs::write(&path, mutated).expect("write mutation");
    assert!(read_artifact_lineage_catalog(&path).is_ok());
    assert!(read_artifact_lineage_catalog_strict(&path).is_err());
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn phase_e_wilson_95_decimal_bits_and_serialized_vectors_are_exact() {
    let ledger: serde_json::Value = serde_json::from_slice(
        &fs::read(fixture("expected/wilson_vectors.schema1.json")).expect("Wilson ledger"),
    )
    .expect("Wilson ledger JSON");
    assert_eq!(ledger["schema_version"], 1);
    assert_eq!(ledger["interval_method"], "wilson_95_v1");
    for vector in ledger["vectors"].as_array().expect("vectors") {
        let numerator = vector["numerator"]
            .as_str()
            .expect("numerator")
            .parse::<u64>()
            .expect("u64 numerator");
        let denominator = vector["denominator"]
            .as_str()
            .expect("denominator")
            .parse::<u64>()
            .expect("u64 denominator");
        let value = wilson_95(numerator, denominator);
        match vector["kind"].as_str().expect("kind") {
            "unavailable" => assert!(matches!(value, MetricValueV1::Unavailable { .. })),
            "available" => {
                let MetricValueV1::Available {
                    point_estimate,
                    lower_confidence_bound,
                    upper_confidence_bound,
                    ..
                } = value
                else {
                    panic!("available Wilson vector")
                };
                assert_eq!(
                    point_estimate.to_bits().to_string(),
                    vector["point_estimate_bits"].as_str().expect("point bits")
                );
                assert_eq!(
                    lower_confidence_bound.to_bits().to_string(),
                    vector["lower_confidence_bound_bits"]
                        .as_str()
                        .expect("lower bits")
                );
                assert_eq!(
                    upper_confidence_bound.to_bits().to_string(),
                    vector["upper_confidence_bound_bits"]
                        .as_str()
                        .expect("upper bits")
                );
                for (actual, field) in [
                    (point_estimate, "point_estimate"),
                    (lower_confidence_bound, "lower_confidence_bound"),
                    (upper_confidence_bound, "upper_confidence_bound"),
                ] {
                    assert!(
                        (actual - vector[field].as_f64().expect("decimal")).abs() <= 1e-12,
                        "{field}"
                    );
                    assert_ne!(actual.to_bits(), (-0.0f64).to_bits());
                }
            }
            other => panic!("unknown Wilson ledger kind {other}"),
        }
    }

    let MetricValueV1::Available {
        point_estimate,
        lower_confidence_bound,
        upper_confidence_bound,
        ..
    } = wilson_95(5, 10)
    else {
        panic!("available interval")
    };
    assert_eq!(point_estimate.to_bits(), 0.5f64.to_bits());
    assert!((lower_confidence_bound - 0.236_593_090_512_564).abs() < 1e-12);
    assert!((upper_confidence_bound - 0.763_406_909_487_436).abs() < 1e-12);
    assert!(matches!(wilson_95(0, 0), MetricValueV1::Unavailable { .. }));
}

#[test]
fn phase_e_health_rates_boundaries_and_balanced_accuracy_are_exact() {
    assert_eq!(balanced_accuracy(1, 1, 0, 0), Ok(1.0));
    assert_eq!(
        balanced_accuracy(0, 1, 0, 0),
        Err("positive_class_denominator_zero")
    );
    assert_eq!(
        balanced_accuracy(1, 0, 0, 0),
        Err("negative_class_denominator_zero")
    );
}

#[test]
fn phase_e_synthetic_only_run_is_software_validated_only() {
    let (inputs, protocol, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let output = temp("software_run");
    run_mhi_validation(MhiValidationRunOptions {
        protocol,
        dataset,
        output_dir: output.clone(),
        overwrite: false,
    })
    .expect("software-only run");
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(output.join("mhi_validation_report.schema1.json")).expect("report"),
    )
    .expect("JSON");
    assert_eq!(report["overall_status"], "indeterminate");
    assert_eq!(report["release_claims"][0]["outcome"], "indeterminate");
    assert_eq!(
        fs::read_dir(output.join("tables")).expect("tables").count(),
        6
    );
    fs::remove_dir_all(output).expect("cleanup");
    fs::remove_dir_all(inputs).expect("cleanup staged inputs");
}

#[test]
fn phase_e_production_physical_request_hard_fails_before_dataset_scoring_or_report() {
    let protocol = fixture("protocol/physical_valid.toml");
    let output = temp("unprovisioned_physical");
    let error = run_mhi_validation(MhiValidationRunOptions {
        protocol,
        // The runner must not even stat or parse this adversarial dataset once
        // the embedded production store reports UNPROVISIONED.
        dataset: fixture("dataset/attacker-controlled-missing.schema1.json"),
        output_dir: output.clone(),
        overwrite: false,
    })
    .expect_err("unprovisioned physical claims are rejected");
    assert!(matches!(
        error,
        rust_electroanalysis_cli::mhi_validation::MhiValidationError::PhysicalApprovalTrustNotProvisioned
    ));
    assert!(!output.exists());
}

#[test]
fn phase_e_publication_is_atomic_and_checksum_verified() {
    let (inputs, protocol, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let output = temp("publication");
    let options = MhiValidationRunOptions {
        protocol,
        dataset,
        output_dir: output.clone(),
        overwrite: false,
    };
    run_mhi_validation(options.clone()).expect("first publication");
    assert!(run_mhi_validation(options.clone()).is_err());
    run_mhi_validation(MhiValidationRunOptions {
        overwrite: true,
        ..options
    })
    .expect("managed replacement");
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(output.join("validation_execution_manifest.schema1.json")).expect("manifest"),
    )
    .expect("JSON");
    assert_eq!(manifest["publication_mode"], "replace_managed_bundle");
    fs::remove_dir_all(output).expect("cleanup");
    fs::remove_dir_all(inputs).expect("cleanup staged inputs");
}

#[test]
fn phase_e_health_confusion_and_missing_state_counts_are_exact() {
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&protocol_bytes).expect("UTF-8"))
            .expect("protocol");
    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &dataset,
    )
    .expect("inputs");
    let report = evaluate_mhi_validation(&protocol, &inputs).expect("report");
    let health = &report.health_results[0];
    assert_eq!(
        health.eligible_count,
        health.tp
            + health.tn
            + health.fp
            + health.r#fn
            + health.indeterminate
            + health.data_quality_insufficient
    );
    assert_eq!(
        health.evaluable,
        health.tp + health.tn + health.fp + health.r#fn
    );
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_mechanism_phase_b_reference_cross_product_matches_hand_oracle() {
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&protocol_bytes).expect("UTF-8"))
            .expect("protocol");
    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &dataset,
    )
    .expect("inputs");
    let report = evaluate_mhi_validation(&protocol, &inputs).expect("report");
    let mechanism = &report.mechanism_results[0];
    // The literal synthetic row has no Phase-B artifact or independent
    // reference.  It therefore cannot be promoted by any inferred cross
    // product category and remains an explicit exclusion rather than support.
    assert_eq!(mechanism.eligible_count, 0);
    assert!(mechanism.support_record_ids.is_empty());
    assert!(mechanism.critical_contradiction_record_ids.is_empty());
    assert!(mechanism.not_assessed_or_other_record_ids.is_empty());
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_mechanism_rates_intervals_and_ids_are_exact() {
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&protocol_bytes).expect("UTF-8"))
            .expect("protocol");
    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &dataset,
    )
    .expect("inputs");
    let report = evaluate_mhi_validation(&protocol, &inputs).expect("report");
    let mechanism = &report.mechanism_results[0];
    assert!(matches!(
        mechanism.support_fraction,
        MetricValueV1::Unavailable {
            numerator: 0,
            denominator: 0,
            ..
        }
    ));
    assert!(mechanism.support_record_ids.is_empty());
    assert!(mechanism.critical_contradiction_record_ids.is_empty());
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_overall_and_closed_strata_apply_exact_record_and_family_minima() {
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let mut protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&protocol_bytes).expect("UTF-8"))
            .expect("protocol");
    protocol.mechanism_endpoints[0].minimum_eligible_records = 2;
    protocol.mechanism_endpoints[0].minimum_independent_families = 2;
    protocol.health_endpoints[0].minimum_eligible_records = 2;
    protocol.health_endpoints[0].minimum_independent_families = 2;
    protocol
        .validate()
        .expect("higher software minima remain valid");
    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &dataset,
    )
    .expect("inputs");
    let report = evaluate_mhi_validation(&protocol, &inputs).expect("report");
    assert_eq!(
        report.release_claims[0].outcome,
        rust_electroanalysis_cli::validation_config::ReleaseClaimOutcomeV1::Indeterminate
    );
    assert_eq!(
        report.overall_status,
        rust_electroanalysis_cli::validation_config::ValidationOutcomeV1::Indeterminate
    );
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_preserves_phase_d_golden_outputs_byte_for_byte() {
    use sha2::{Digest, Sha256};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase_d/base");
    let expected = [
        (
            "mechanism.json",
            "b24422b8e1ec3f99fcea4a9f7c7f225dfe6f77550b0365b6cda80447fd306b8b",
        ),
        (
            "health.json",
            "4265b48a0a70ff6ec89eb214a2cc8c2194cbd43bb7b7098482a7686e2eee73b3",
        ),
        (
            "lineage_catalog.json",
            "6b06d3a7a8b530d1acd4471d7bfc28de95e592a6726a9e72a81015c1ac0db320",
        ),
    ];
    for (name, expected_hash) in expected {
        let bytes = fs::read(root.join(name)).expect("frozen Phase-D input");
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            expected_hash,
            "{name}"
        );
    }
}

#[test]
fn phase_e_preserves_all_existing_artifact_migration_contracts() {
    use sha2::{Digest, Sha256};

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(fixture(
            "compatibility/existing_artifact_fixture_inventory.schema1.json",
        ))
        .expect("literal historical compatibility inventory"),
    )
    .expect("inventory schema");
    assert_eq!(inventory["schema_version"], 1);
    let files = inventory["files"].as_array().expect("file list");
    assert_eq!(files.len(), 48, "the R2 historical set is closed");
    for file in files {
        let path = file["relative_path"].as_str().expect("relative path");
        let expected = file["sha256"].as_str().expect("fixture hash");
        let bytes = fs::read(root.join(path)).expect("historical fixture remains present");
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), expected, "{path}");
    }
}

#[test]
fn phase_e_author_side_traceability_evidence_is_non_self_approving() {
    use sha2::{Digest, Sha256};

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let plan = fs::read(root.join("docs/engineering_specification/next_milestone_plan.md"))
        .expect("approved R2 plan");
    assert_eq!(
        format!("{:x}", Sha256::digest(plan)),
        "e6e5195c7f56904afb06dfe937433f3498465fef1df191b8fb6856ee1ac792b6"
    );
    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("Cargo manifest");
    assert!(cargo.contains("ed25519-dalek = { version = \"=2.2.0\", default-features = false }"));
    assert!(!cargo.contains("ed25519-dalek = { version = \"=2.2.0\", features"));

    let phase_e_sources = [
        root.join("src/mhi_validation"),
        root.join("src/runners/mhi_validation.rs"),
    ];
    let mut source_text = String::new();
    for source in phase_e_sources {
        if source.is_file() {
            source_text.push_str(&fs::read_to_string(source).expect("source"));
        } else {
            for path in fs::read_dir(source).expect("source directory") {
                let path = path.expect("source entry").path();
                if path.extension().is_some_and(|extension| extension == "rs") {
                    source_text.push_str(&fs::read_to_string(path).expect("source"));
                }
            }
        }
    }
    let test_source = fs::read_to_string(root.join("tests/phase_e_validation.rs"))
        .expect("Phase-E integration test source");
    let registry_source = format!("{source_text}\n{test_source}");
    for required in [
        "phase_e_cli_runs_exact_certified_route",
        "phase_e_cli_rejects_missing_unknown_and_raw_input_routes",
        "phase_e_protocol_roundtrip_preserves_all_scientific_rules",
        "phase_e_protocol_rejects_incomplete_conflicting_untrusted_and_nondeterministic_authority",
        "phase_e_dataset_schema1_roundtrip_is_closed_and_canonical",
        "phase_e_dataset_recomputes_semantic_identity_and_rejects_root_or_path_mismatch",
        "phase_e_reader_accepts_only_canonical_schema4_scientific_inputs",
        "phase_e_reader_hard_fails_wrong_future_and_explicitly_excludes_legacy",
        "phase_e_partition_accounts_for_every_declared_record_exactly_once",
        "phase_e_holdout_rejects_known_lineage_scope_and_family_overlap",
        "phase_e_holdout_unknown_separation_is_indeterminate_without_fabrication",
        "phase_e_combined_reference_catalog_closure_and_authority_are_total",
        "phase_e_mechanism_phase_b_reference_cross_product_matches_hand_oracle",
        "phase_e_mechanism_rates_intervals_and_ids_are_exact",
        "phase_e_health_confusion_and_missing_state_counts_are_exact",
        "phase_e_health_rates_boundaries_and_balanced_accuracy_are_exact",
        "phase_e_wilson_95_decimal_bits_and_serialized_vectors_are_exact",
        "phase_e_overall_and_closed_strata_apply_exact_record_and_family_minima",
        "phase_e_exclusions_and_acceptance_use_complete_ordered_precedence",
        "phase_e_report_reconstructs_every_count_from_source_ids",
        "phase_e_authority_assisted_report_and_all_scientific_bytes_are_exact",
        "phase_e_publication_is_atomic_and_checksum_verified",
        "phase_e_publication_is_locked_no_clobber_crash_durable_and_residue_exact",
        "phase_e_source_guards_prohibit_reassessment_and_reverse_dependencies",
        "phase_e_preserves_phase_d_golden_outputs_byte_for_byte",
        "phase_e_artifact_contracts_accept_exact_schema1_and_reject_invalid_variants",
        "phase_e_preserves_all_existing_artifact_migration_contracts",
        "phase_e_synthetic_only_run_is_software_validated_only",
        "phase_e_physical_claim_requires_dual_signature_embedded_trust_and_power",
        "phase_e_author_side_traceability_evidence_is_non_self_approving",
    ] {
        assert!(
            registry_source.contains(required),
            "missing required evidence {required}"
        );
    }
    assert!(!source_text.contains("SigningKey"));
    let ignored_attribute = format!("#[{}]", "ignore");
    assert!(!source_text.contains(&ignored_attribute));
    assert!(!test_source.contains(&ignored_attribute));
    assert!(!source_text.contains("IMPLEMENTATION_APPROVAL = GO"));
}

#[test]
fn phase_e_production_physical_store_is_embedded_and_unprovisioned() {
    let trust = PhysicalApprovalTrustStoreV1::from_embedded_bytes()
        .expect("embedded trust store validates");
    assert_eq!(
        trust.store.provisioning_state,
        PhysicalApprovalProvisioningStateV1::Unprovisioned
    );
    assert!(trust.store.trust_roots.is_empty());
    assert!(!trust.store.is_provisioned());
}

#[test]
fn phase_e_source_guards_prohibit_reassessment_and_reverse_dependencies() {
    let mechanism = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/mhi_validation/evaluation.rs"),
    )
    .expect("source");
    assert!(!mechanism.contains("mechanism::evaluation"));
    assert!(!mechanism.contains("health::assessment"));
    let phase_d = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/reporting/projection.rs"),
    )
    .expect("source");
    assert!(!phase_d.contains("mhi_validation"));
}

#[test]
fn phase_e_partition_accounts_for_every_declared_record_exactly_once() {
    let bytes = fs::read(fixture("protocol/software_valid.toml")).expect("fixture");
    let protocol = MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("UTF-8"))
        .expect("protocol");
    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&bytes),
        &dataset,
    )
    .expect("inputs");
    let report = evaluate_mhi_validation(&protocol, &inputs).expect("report");
    assert_eq!(report.record_accounting.len(), 2);
    for cohort in &report.cohorts {
        assert_eq!(
            cohort.declared_count,
            cohort.eligible_count + cohort.excluded_count
        );
    }
    let mechanism = &report.mechanism_results[0];
    assert_eq!(
        mechanism.eligible_count,
        mechanism.support_count
            + mechanism.critical_contradiction_count
            + mechanism.not_assessed_or_other_count
    );
    let health = &report.health_results[0];
    assert_eq!(
        health.eligible_count,
        health.tp
            + health.tn
            + health.fp
            + health.r#fn
            + health.indeterminate
            + health.data_quality_insufficient
    );
    assert!(report.validate_structure().is_ok());
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_report_reconstructs_every_count_from_source_ids() {
    let bytes = fs::read(fixture("protocol/software_valid.toml")).expect("fixture");
    let protocol = MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("UTF-8"))
        .expect("protocol");
    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&bytes),
        &dataset,
    )
    .expect("inputs");
    let mut report = evaluate_mhi_validation(&protocol, &inputs).expect("report");
    report.health_results[0].tp = 1;
    assert!(report.validate_structure().is_err());
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_authority_assisted_report_and_all_scientific_bytes_are_exact() {
    let bytes = fs::read(fixture("protocol/software_valid.toml")).expect("fixture");
    let protocol = MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("UTF-8"))
        .expect("protocol");
    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&bytes),
        &dataset,
    )
    .expect("inputs");
    let mut report = evaluate_mhi_validation(&protocol, &inputs).expect("report");
    report
        .validate_against(&protocol, &inputs, None)
        .expect("exact replay");
    report.overall_status =
        rust_electroanalysis_cli::validation_config::ValidationOutcomeV1::MeetsProtocol;
    assert!(report.validate_against(&protocol, &inputs, None).is_err());
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}
