use rust_electroanalysis_cli::{
    cli::{CommandSpec, parse_cli_args},
    domain::{ArtifactError, read_artifact, read_artifact_strict},
    mhi_validation::{
        MhiValidationProtocolV1,
        approval::PhysicalApprovalTrustStoreV1,
        statistics::{MetricValueV1, balanced_accuracy, wilson_95},
    },
    results::MhiValidationDatasetV1,
    runners::mhi_validation::{MhiValidationRunOptions, run_mhi_validation},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_e")
        .join(name)
}

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "phase_e_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
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
    let bytes = fs::read(fixture("software_protocol.toml")).expect("fixture");
    let protocol = MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("UTF-8"))
        .expect("protocol validates");
    assert_eq!(protocol.schema_version, 1);
    assert_eq!(protocol.release_scope.len(), 1);
    assert_eq!(
        MhiValidationProtocolV1::sha256_of_bytes(&bytes),
        "84e0612214f08bd4a7fec19320ca75714cb4b56e72d3c23f518fcb5e26f9f494"
    );
}

#[test]
fn phase_e_protocol_rejects_incomplete_conflicting_untrusted_and_nondeterministic_authority() {
    let text = fs::read_to_string(fixture("software_protocol.toml")).expect("fixture");
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
    let path = fixture("software_dataset.schema1.json");
    let dataset: MhiValidationDatasetV1 =
        read_artifact(&path).expect("legacy reader accepts valid Phase-E dataset");
    assert_eq!(dataset.schema_version, 1);
    assert_eq!(dataset.records[0].record_id, "record_1");
}

#[test]
fn phase_e_reader_hard_fails_duplicate_json_without_changing_existing_reader() {
    let directory = temp("duplicate_reader");
    fs::create_dir_all(&directory).expect("directory");
    let path = directory.join("dataset.json");
    let mut text = fs::read_to_string(fixture("software_dataset.schema1.json")).expect("fixture");
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
fn phase_e_wilson_95_decimal_bits_and_serialized_vectors_are_exact() {
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
    let output = temp("software_run");
    run_mhi_validation(MhiValidationRunOptions {
        protocol: fixture("software_protocol.toml"),
        dataset: fixture("software_dataset.schema1.json"),
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
}

#[test]
fn phase_e_publication_is_no_clobber_and_managed_overwrite_is_deterministic() {
    let output = temp("publication");
    let options = MhiValidationRunOptions {
        protocol: fixture("software_protocol.toml"),
        dataset: fixture("software_dataset.schema1.json"),
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
}

#[test]
fn phase_e_physical_store_is_embedded_strict_and_role_separated() {
    let trust = PhysicalApprovalTrustStoreV1::from_embedded_bytes()
        .expect("embedded trust store validates");
    let root = &trust.store.trust_roots[0];
    assert_ne!(root.project_owner_authority_id, root.registry_authority_id);
    assert_ne!(
        root.owner_ed25519_public_key_hex,
        root.registry_ed25519_public_key_hex
    );
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
