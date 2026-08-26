use rust_electroanalysis_cli::{
    cli::{CliError, CommandSpec, parse_cli_args},
    domain::{
        ArtifactError, read_artifact, read_artifact_lineage_catalog,
        read_artifact_lineage_catalog_strict, read_artifact_strict,
    },
    mhi_validation::{
        MhiValidationError, MhiValidationProtocolV1, ValidationInputs,
        approval::{
            PhysicalApprovalProvisioningStateV1, PhysicalApprovalTrustRootV1,
            PhysicalApprovalTrustStoreV1,
        },
        evaluate_mhi_validation,
        statistics::{MetricValueV1, balanced_accuracy, wilson_95, wilson_95_checked},
    },
    results::{
        ExpectedLineageV1, MechanismReferenceOutcomeV1, MhiValidationDatasetV1,
        ReferenceEndpointV1, ReferenceSourceAuthorityV1, ReferenceUncertaintyV1,
    },
    runners::mhi_validation::{MhiValidationRunOptions, run_mhi_validation},
    validation_config::{
        AcceptanceRuleV1, BlindingStateV1, CategoricalSelectorV1, CohortRoleV1, ComparatorV1,
        DomainKeyV1, PhysicalApprovalAuthorityV1, RateMetricV1, RateTargetV1,
        ReferenceAuthorityRuleV1, ReferenceDependencyCompletenessV1, RequiredStratumV1,
        StratumPredicateV1, TemperatureBandV1, TemperatureSelectorV1,
    },
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
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

fn fixture_regular_files(root: &Path, directory: &Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(directory).expect("fixture directory") {
        let entry = entry.expect("fixture entry");
        let path = entry.path();
        let kind = entry.file_type().expect("fixture file type");
        assert!(
            !kind.is_symlink(),
            "fixture inventory rejects symlink {path:?}"
        );
        if kind.is_dir() {
            fixture_regular_files(root, &path, files);
        } else {
            assert!(
                kind.is_file(),
                "fixture inventory rejects non-file {path:?}"
            );
            files.push(
                path.strip_prefix(root)
                    .expect("fixture path beneath root")
                    .to_str()
                    .expect("UTF-8 fixture path")
                    .replace('\\', "/"),
            );
        }
    }
}

fn expected_fixture_paths() -> BTreeSet<String> {
    let bytes = fs::read(fixture("expected/phase_e_fixture_inventory.schema1.json"))
        .expect("closed fixture inventory");
    let inventory: serde_json::Value = serde_json::from_slice(&bytes).expect("inventory JSON");
    let rows = inventory.as_array().expect("inventory array");
    let mut paths = BTreeSet::new();
    for row in rows {
        let relative_path = row["relative_path"].as_str().expect("relative path");
        assert!(
            !relative_path.contains('*')
                && !relative_path.contains("..")
                && !relative_path.is_empty(),
            "literal fixture path"
        );
        let mappings = row["mappings"].as_array().expect("fixture mappings");
        assert!(!mappings.is_empty(), "fixture must map to an R2 test");
        let mut previous = None;
        for mapping in mappings {
            for name in [
                "requirement_id",
                "acceptance_criterion_id",
                "test_id",
                "expected_result_id",
            ] {
                assert!(mapping[name].as_str().is_some(), "mapping {name}");
            }
            assert!(
                mapping["mutation_case_ids"]
                    .as_array()
                    .is_some_and(|ids| !ids.is_empty()),
                "mutation mapping"
            );
            let key = format!(
                "{}\0{}\0{}\0{}\0{}",
                mapping["requirement_id"].as_str().expect("requirement ID"),
                mapping["acceptance_criterion_id"]
                    .as_str()
                    .expect("acceptance criterion ID"),
                mapping["test_id"].as_str().expect("test ID"),
                mapping["mutation_case_ids"],
                mapping["expected_result_id"]
                    .as_str()
                    .expect("expected result ID")
            );
            assert!(
                previous.as_ref().is_none_or(|old| old < &key),
                "canonical mapping order"
            );
            previous = Some(key);
        }
        assert!(
            paths.insert(relative_path.to_owned()),
            "duplicate fixture row"
        );
    }
    paths
}

fn assert_scientific_bundle(output: &Path) {
    let managed = [
        "mhi_validation_report.schema1.json",
        "validation_summary.md",
        "tables/cohort_coverage.csv",
        "tables/leakage_assessment.csv",
        "tables/mechanism_validation.csv",
        "tables/health_validation.csv",
        "tables/exclusion_ledger.csv",
        "tables/compatibility_matrix.csv",
    ];
    for relative in managed {
        assert!(
            output.join(relative).is_file(),
            "managed scientific file {relative}"
        );
    }
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(output.join("mhi_validation_report.schema1.json")).expect("report"),
    )
    .expect("report JSON");
    assert_eq!(report["overall_status"], "meets_protocol");
    assert_eq!(
        report["release_claims"][0]["outcome"],
        "software_validated_only"
    );
    assert_eq!(
        report["mechanism_results"][0]["support_record_ids"],
        serde_json::json!(["record_1", "record_2"])
    );
    assert_eq!(
        report["health_results"][0]["tp_record_ids"],
        serde_json::json!(["record_2"])
    );
    assert_eq!(
        report["health_results"][0]["tn_record_ids"],
        serde_json::json!(["record_1"])
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(output.join("validation_execution_manifest.schema1.json")).expect("manifest"),
    )
    .expect("manifest JSON");
    let files = manifest["generated_files"]
        .as_array()
        .expect("manifest files");
    assert_eq!(files.len(), 8, "manifest excludes itself");
    assert!(
        files
            .iter()
            .all(|file| file["relative_path"] != "validation_execution_manifest.schema1.json")
    );
}

/// The committed R2 bundle list is the output authority.  This helper derives
/// neither hashes nor expected paths from production output: it first checks
/// the independently committed golden bytes and then compares the certified
/// route byte-for-byte against that sealed list.
fn assert_exact_golden_bundle(output: &Path) {
    let expected = fs::read_to_string(fixture("expected/golden_bundle_file_sha256s.txt"))
        .expect("independent golden bundle digest list");
    let expected = expected
        .lines()
        .map(|line| {
            let mut columns = line.split('\t');
            let relative_path = columns.next().expect("golden relative path");
            let byte_length = columns
                .next()
                .expect("golden byte length")
                .parse::<u64>()
                .expect("golden byte length integer");
            let sha256 = columns.next().expect("golden SHA-256");
            assert!(columns.next().is_none(), "three golden-list columns");
            (relative_path.to_owned(), byte_length, sha256.to_owned())
        })
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 9, "R2 certifies exactly nine managed files");

    let mut actual_paths = Vec::new();
    fixture_regular_files(output, output, &mut actual_paths);
    actual_paths.sort();
    let expected_paths = expected
        .iter()
        .map(|(path, _, _)| path.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_paths, expected_paths,
        "no extra or missing managed file"
    );

    for (relative_path, byte_length, sha256) in expected {
        let golden_path = fixture("expected/golden_bundle").join(&relative_path);
        let golden_bytes = fs::read(&golden_path).expect("committed golden bytes");
        assert_eq!(
            golden_bytes.len() as u64,
            byte_length,
            "golden length {relative_path}"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&golden_bytes)),
            sha256,
            "golden digest {relative_path}"
        );

        let actual = fs::read(output.join(&relative_path)).expect("certified output bytes");
        assert_eq!(
            actual.len() as u64,
            byte_length,
            "output length {relative_path}"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&actual)),
            sha256,
            "output digest {relative_path}"
        );
        assert_eq!(actual, golden_bytes, "output bytes {relative_path}");
    }
}

fn protocol_fixture(name: &str) -> MhiValidationProtocolV1 {
    let bytes = fs::read(fixture(name)).expect("protocol fixture bytes");
    MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("protocol UTF-8"))
        .expect("valid protocol fixture")
}

fn protocol_error(result: Result<(), MhiValidationError>, expected: &str) {
    match result {
        Err(MhiValidationError::Protocol(actual)) => assert_eq!(actual, expected),
        Err(other) => panic!("expected protocol error {expected:?}, received {other:?}"),
        Ok(()) => panic!("expected protocol error {expected:?}, received success"),
    }
}

fn cli_error_kind(args: &[&str]) -> clap::error::ErrorKind {
    let args = args
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    match parse_cli_args(&args) {
        Err(CliError::Parse(error)) => error.kind(),
        Err(other) => panic!("expected Clap parse error, received {other:?}"),
        Ok(_) => panic!("expected Clap parse error, received success"),
    }
}

#[test]
fn phase_e_dataset_recomputes_semantic_identity_and_rejects_root_or_path_mismatch() {
    use rust_electroanalysis_cli::domain::{AcquisitionFamilyId, ArtifactAcquisitionFamilies};

    let (root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let protocol_bytes = fs::read(&protocol_path).expect("protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);
    ValidationInputs::read(&protocol, &protocol_hash, &dataset_path)
        .expect("complete source authority is readable");
    let valid_dataset_bytes = fs::read(&dataset_path).expect("valid dataset bytes");

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let source_alias = dataset_path
            .parent()
            .expect("dataset parent")
            .join("source-alias");
        symlink(
            dataset_path
                .parent()
                .expect("dataset parent")
                .join("sources"),
            &source_alias,
        )
        .expect("source intermediate symlink");
        let mut symlinked = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
            .expect("test dataset")
            .artifact;
        symlinked.records[0]
            .mechanism_source
            .as_mut()
            .expect("mechanism source")
            .relative_path = "source-alias/mechanism_a.schema4.json".into();
        write_test_dataset(&dataset_path, &mut symlinked);
        assert!(matches!(
            ValidationInputs::read(&protocol, &protocol_hash, &dataset_path),
            Err(MhiValidationError::UnsafePath(_))
        ));
        fs::write(&dataset_path, &valid_dataset_bytes).expect("restore dataset fixture");
        fs::remove_file(source_alias).expect("remove source intermediate symlink");
    }

    for unsafe_path in [
        "/absolute/lineage.json",
        "../lineage/complete.schema1.json",
        "",
        "lineage/../complete.schema1.json",
    ] {
        let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
            .expect("test dataset")
            .artifact;
        dataset.lineage_catalog_source.relative_path = unsafe_path.into();
        // The cohort preimage intentionally excludes operational paths; this
        // verifies that path validation, not a stale hash, rejects each case.
        fs::write(
            &dataset_path,
            serde_json::to_vec_pretty(&dataset).expect("dataset serialization"),
        )
        .expect("unsafe dataset mutation");
        assert!(
            ValidationInputs::read(&protocol, &protocol_hash, &dataset_path).is_err(),
            "unsafe path {unsafe_path:?}"
        );
        fs::write(&dataset_path, &valid_dataset_bytes).expect("restore dataset fixture");
    }

    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    dataset.cohort_semantic_sha256 = "0".repeat(64);
    fs::write(
        &dataset_path,
        serde_json::to_vec_pretty(&dataset).expect("dataset serialization"),
    )
    .expect("false embedded cohort hash");
    assert!(matches!(
        read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path),
        Err(ArtifactError::Validation { ref message })
            if message == "cohort_semantic_sha256 does not match the canonical dataset preimage"
    ));
    fs::remove_dir_all(&root).expect("false-hash root cleanup");

    let (root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let protocol_bytes = fs::read(&protocol_path).expect("protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);

    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    dataset.lineage_catalog_source.source_file_sha256 = "0".repeat(64);
    write_test_dataset(&dataset_path, &mut dataset);
    assert!(matches!(
        ValidationInputs::read(&protocol, &protocol_hash, &dataset_path),
        Err(MhiValidationError::Dataset(ref message))
            if message == "lineage catalog checksum does not match dataset authority"
    ));
    fs::remove_dir_all(&root).expect("catalog-hash root cleanup");

    let (root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let protocol_bytes = fs::read(&protocol_path).expect("protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);
    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    dataset.records[0]
        .mechanism_source
        .as_mut()
        .expect("source")
        .source_file_sha256 = "0".repeat(64);
    write_test_dataset(&dataset_path, &mut dataset);
    assert!(matches!(
        ValidationInputs::read(&protocol, &protocol_hash, &dataset_path),
        Err(MhiValidationError::Dataset(ref message))
            if message == "mechanism source checksum or schema mismatch"
    ));
    fs::remove_dir_all(&root).expect("source-hash root cleanup");

    let (root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let protocol_bytes = fs::read(&protocol_path).expect("protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);
    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    let ExpectedLineageV1::Known {
        artifact_id,
        semantic_sha256,
    } = &mut dataset.records[0]
        .mechanism_source
        .as_mut()
        .expect("source")
        .expected_lineage
    else {
        panic!("source has known lineage")
    };
    *semantic_sha256 = "0".repeat(64);
    *artifact_id =
        rust_electroanalysis_cli::domain::ArtifactId(format!("sha256:{semantic_sha256}"));
    write_test_dataset(&dataset_path, &mut dataset);
    let error = ValidationInputs::read(&protocol, &protocol_hash, &dataset_path)
        .expect_err("false embedded lineage semantic hash");
    assert_eq!(
        error.to_string(),
        "MHI validation dataset error: source embedded lineage does not match the dataset expectation"
    );
    fs::remove_dir_all(&root).expect("lineage-expectation root cleanup");

    let (root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let protocol_bytes = fs::read(&protocol_path).expect("protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);
    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    dataset.records[0].declared_scope.acquisition_families = ArtifactAcquisitionFamilies::Known(
        vec![AcquisitionFamilyId("declared-but-not-embedded".into())],
    );
    write_test_dataset(&dataset_path, &mut dataset);
    assert!(matches!(
        ValidationInputs::read(&protocol, &protocol_hash, &dataset_path),
        Err(MhiValidationError::Dataset(ref message))
            if message == "record declared scope differs from the known source identity"
    ));
    fs::remove_dir_all(root).expect("declared-family root cleanup");

    let (root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let protocol_bytes = fs::read(&protocol_path).expect("protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);
    let renamed = dataset_path
        .parent()
        .expect("dataset parent")
        .join("sources/renamed_duplicate.schema4.json");
    fs::copy(
        dataset_path
            .parent()
            .expect("dataset parent")
            .join("sources/mechanism_a.schema4.json"),
        &renamed,
    )
    .expect("renamed duplicate source");
    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    dataset.records[1].mechanism_source = dataset.records[0].mechanism_source.clone();
    dataset.records[1]
        .mechanism_source
        .as_mut()
        .expect("copied mechanism source")
        .relative_path = "sources/renamed_duplicate.schema4.json".into();
    dataset.records[1].declared_scope = dataset.records[0].declared_scope.clone();
    write_test_dataset(&dataset_path, &mut dataset);
    assert!(matches!(
        ValidationInputs::read(&protocol, &protocol_hash, &dataset_path),
        Err(MhiValidationError::Dataset(ref message))
            if message == "duplicate assessed scientific source key for endpoint"
    ));
    fs::remove_dir_all(root).expect("renamed duplicate root cleanup");
}

#[test]
fn phase_e_holdout_rejects_known_lineage_scope_and_family_overlap() {
    use rust_electroanalysis_cli::{
        mhi_validation::partition::{EndpointPartitionSpec, EndpointSource, partition_endpoint},
        validation_config::{RecordDecisionV1, SeparationStatusV1},
    };

    let (root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    add_reference_overlap_records(&mut dataset);
    write_test_dataset(&dataset_path, &mut dataset);

    let bytes = fs::read(protocol_path).expect("protocol");
    let mut protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("protocol UTF-8"))
            .expect("protocol");
    protocol.mechanism_endpoints[0].cohort_role = CohortRoleV1::Holdout;
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&bytes),
        &dataset_path,
    )
    .expect("lineage-overlap inputs");
    let endpoint = &protocol.mechanism_endpoints[0];
    let partition = partition_endpoint(
        &inputs,
        EndpointPartitionSpec {
            endpoint_id: &endpoint.endpoint_id,
            cohort_role: endpoint.cohort_role,
            domain: &endpoint.domain,
            required_strata: &endpoint.required_strata,
            reference_rule: &endpoint.reference_rule,
            source: EndpointSource::Mechanism,
            physical: false,
        },
    )
    .expect("overlap classification");
    let holdout = partition
        .rows
        .iter()
        .find(|row| row.record_id == "holdout_1")
        .expect("holdout row");
    assert_eq!(holdout.decision, RecordDecisionV1::Eligible);
    assert_eq!(
        holdout.separation_status,
        Some(SeparationStatusV1::KnownOverlap)
    );
    assert!(!holdout.shared_artifact_ids.is_empty());
    assert_eq!(holdout.compared_development_record_ids, ["development_1"]);
    fs::remove_dir_all(root).expect("lineage-overlap cleanup");
}

#[test]
fn phase_e_holdout_unknown_separation_is_indeterminate_without_fabrication() {
    use rust_electroanalysis_cli::{
        mhi_validation::partition::{EndpointPartitionSpec, EndpointSource, partition_endpoint},
        validation_config::{RecordDecisionV1, SeparationStatusV1, SeparationUnknownReasonV1},
    };

    let (root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    dataset.records[0].record_id = "holdout_unknown_1".into();
    dataset.records[0].cohort_role = CohortRoleV1::Holdout;
    dataset.records[0].reference_endpoints = vec![mechanism_reference("holdout_unknown_1")];
    dataset.reference_sources = vec![ReferenceSourceAuthorityV1 {
        reference_source_id: "reference_1".into(),
        source_file_sha256: "1".repeat(64),
        evidence_origin: rust_electroanalysis_cli::validation_config::EvidenceOriginV1::Synthetic,
        dependency_completeness: ReferenceDependencyCompletenessV1::Unknown,
        experiment_scope: rust_electroanalysis_cli::domain::ArtifactExperimentScope::Unknown,
        acquisition_families:
            rust_electroanalysis_cli::domain::ArtifactAcquisitionFamilies::Unknown,
        direct_dependencies: vec![],
    }];
    write_test_dataset(&dataset_path, &mut dataset);

    let bytes = fs::read(protocol_path).expect("protocol");
    let mut protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("protocol UTF-8"))
            .expect("protocol");
    protocol.mechanism_endpoints[0].cohort_role = CohortRoleV1::Holdout;
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&bytes),
        &dataset_path,
    )
    .expect("unknown-separation inputs");
    let endpoint = &protocol.mechanism_endpoints[0];
    let partition = partition_endpoint(
        &inputs,
        EndpointPartitionSpec {
            endpoint_id: &endpoint.endpoint_id,
            cohort_role: endpoint.cohort_role,
            domain: &endpoint.domain,
            required_strata: &endpoint.required_strata,
            reference_rule: &endpoint.reference_rule,
            source: EndpointSource::Mechanism,
            physical: false,
        },
    )
    .expect("unknown classification");
    let row = partition.rows.first().expect("single holdout row");
    assert_eq!(row.decision, RecordDecisionV1::Eligible);
    assert_eq!(
        row.separation_status,
        Some(SeparationStatusV1::UnknownSeparation)
    );
    assert!(row.shared_artifact_ids.is_empty());
    assert!(row.shared_family_ids.is_empty());
    assert!(
        row.unknown_reasons
            .contains(&SeparationUnknownReasonV1::ReferenceDependencyIncomplete)
    );
    assert!(
        row.unknown_reasons
            .contains(&SeparationUnknownReasonV1::ReferenceExperimentScopeUnknown)
    );
    fs::remove_dir_all(root).expect("unknown-separation cleanup");
}

#[test]
fn phase_e_combined_reference_catalog_closure_and_authority_are_total() {
    use rust_electroanalysis_cli::{
        mhi_validation::partition::reference_exclusion_reasons,
        validation_config::{BlindingStateV1, ExclusionReasonV1},
    };

    let protocol = protocol_fixture("protocol/software_valid.toml");
    let rule = &protocol.mechanism_endpoints[0].reference_rule;
    let valid = mechanism_reference("authority");
    assert_eq!(
        reference_exclusion_reasons(rule, &valid, false).expect("valid reference"),
        vec![]
    );

    let mut unavailable = valid.clone();
    let ReferenceEndpointV1::Mechanism { outcome, .. } = &mut unavailable else {
        unreachable!()
    };
    *outcome = MechanismReferenceOutcomeV1::Unavailable;
    assert_eq!(
        reference_exclusion_reasons(rule, &unavailable, false).expect("software unavailable"),
        vec![ExclusionReasonV1::ReferenceOutcomeUnavailable]
    );
    assert!(matches!(
        reference_exclusion_reasons(rule, &unavailable, true),
        Err(MhiValidationError::Dataset(ref message))
            if message == "PhysicalReferenceOutcomeUnavailable"
    ));

    let mut wrong_method = valid.clone();
    let ReferenceEndpointV1::Mechanism { method_id, .. } = &mut wrong_method else {
        unreachable!()
    };
    *method_id = "different_method".into();
    assert_eq!(
        reference_exclusion_reasons(rule, &wrong_method, false).expect("wrong method"),
        vec![ExclusionReasonV1::ReferenceMethodNotAllowed]
    );

    let mut wrong_authority = valid.clone();
    let ReferenceEndpointV1::Mechanism { authority_id, .. } = &mut wrong_authority else {
        unreachable!()
    };
    *authority_id = "different_authority".into();
    assert_eq!(
        reference_exclusion_reasons(rule, &wrong_authority, false).expect("wrong authority"),
        vec![ExclusionReasonV1::ReferenceAuthorityNotAllowed]
    );

    let mut unblinded = valid.clone();
    let ReferenceEndpointV1::Mechanism { blinding_state, .. } = &mut unblinded else {
        unreachable!()
    };
    *blinding_state = BlindingStateV1::NotBlinded;
    assert_eq!(
        reference_exclusion_reasons(rule, &unblinded, false).expect("unblinded reference"),
        vec![ExclusionReasonV1::ReferenceBlindingNotAllowed]
    );

    let mut unknown_blinding = valid.clone();
    let ReferenceEndpointV1::Mechanism { blinding_state, .. } = &mut unknown_blinding else {
        unreachable!()
    };
    *blinding_state = BlindingStateV1::Unknown;
    assert_eq!(
        reference_exclusion_reasons(rule, &unknown_blinding, false)
            .expect("unknown blinding reference"),
        vec![ExclusionReasonV1::ReferenceBlindingNotAllowed]
    );

    let mut wrong_method_version = valid.clone();
    let ReferenceEndpointV1::Mechanism { method_version, .. } = &mut wrong_method_version else {
        unreachable!()
    };
    *method_version = "2".into();
    assert_eq!(
        reference_exclusion_reasons(rule, &wrong_method_version, false)
            .expect("wrong method version"),
        vec![ExclusionReasonV1::ReferenceMethodNotAllowed]
    );

    let mut missing_uncertainty = valid.clone();
    let ReferenceEndpointV1::Mechanism { uncertainty, .. } = &mut missing_uncertainty else {
        unreachable!()
    };
    *uncertainty = ReferenceUncertaintyV1::Unavailable {
        reason: "not quantified".into(),
    };
    assert_eq!(
        reference_exclusion_reasons(rule, &missing_uncertainty, false)
            .expect("missing uncertainty"),
        vec![ExclusionReasonV1::ReferenceUncertaintyUnavailable]
    );

    for (mutation, expected) in [
        (
            "wrong_measure",
            ExclusionReasonV1::ReferenceUncertaintyMeasureMismatch,
        ),
        (
            "wrong_unit",
            ExclusionReasonV1::ReferenceUncertaintyUnitMismatch,
        ),
        (
            "above_maximum",
            ExclusionReasonV1::ReferenceUncertaintyAboveMaximum,
        ),
    ] {
        let mut reference = valid.clone();
        let ReferenceEndpointV1::Mechanism { uncertainty, .. } = &mut reference else {
            unreachable!()
        };
        let ReferenceUncertaintyV1::Quantified {
            measure_id,
            value,
            unit,
        } = uncertainty
        else {
            unreachable!()
        };
        match mutation {
            "wrong_measure" => *measure_id = "other_measure".into(),
            "wrong_unit" => *unit = "V".into(),
            "above_maximum" => *value = 1.1,
            _ => unreachable!(),
        }
        assert_eq!(
            reference_exclusion_reasons(rule, &reference, false).expect("reference mutation"),
            vec![expected]
        );
    }

    let mut allow_unavailable = rule.clone();
    match &mut allow_unavailable {
        rust_electroanalysis_cli::validation_config::ReferenceAuthorityRuleV1::Mechanism {
            uncertainty_rule,
            ..
        } => {
            *uncertainty_rule =
                rust_electroanalysis_cli::validation_config::ReferenceUncertaintyRuleV1::AllowUnavailableWithLimitation;
        }
        _ => unreachable!("mechanism fixture uses mechanism rule"),
    }
    assert_eq!(
        reference_exclusion_reasons(&allow_unavailable, &missing_uncertainty, false)
            .expect("allowed uncertainty limitation"),
        Vec::<ExclusionReasonV1>::new(),
        "allowed unavailable uncertainty is not silently recast as an authority exclusion"
    );
}

#[test]
fn phase_e_exclusions_and_acceptance_use_complete_ordered_precedence() {
    use rust_electroanalysis_cli::validation_config::ExclusionReasonV1;

    let expected: serde_json::Value = serde_json::from_slice(
        &fs::read(fixture("expected/exclusion_precedence.schema1.json"))
            .expect("literal precedence oracle"),
    )
    .expect("precedence oracle JSON");
    let tokens = expected["ordered_primary_reasons"]
        .as_array()
        .expect("reason tokens")
        .iter()
        .map(|value| value.as_str().expect("reason token"))
        .collect::<Vec<_>>();
    assert_eq!(
        tokens,
        [
            "missing_endpoint_artifact_path",
            "missing_reference_endpoint",
            "known_overlap",
            "unknown_separation",
            "reference_authority_ineligible",
        ]
    );
    let complete = [
        ExclusionReasonV1::MissingEndpointArtifactPath,
        ExclusionReasonV1::SourceNotPhaseBOrCScoreable,
        ExclusionReasonV1::MissingReferenceEndpoint,
        ExclusionReasonV1::ReferenceOutcomeUnavailable,
        ExclusionReasonV1::ReferenceMethodNotAllowed,
        ExclusionReasonV1::ReferenceAuthorityNotAllowed,
        ExclusionReasonV1::ReferenceBlindingNotAllowed,
        ExclusionReasonV1::ReferenceUncertaintyUnavailable,
        ExclusionReasonV1::ReferenceUncertaintyMeasureMismatch,
        ExclusionReasonV1::ReferenceUncertaintyUnitMismatch,
        ExclusionReasonV1::ReferenceUncertaintyAboveMaximum,
        ExclusionReasonV1::ValidationKnownOverlap,
        ExclusionReasonV1::ValidationUnknownSeparation,
    ];
    assert_eq!(
        complete
            .iter()
            .map(|reason| reason.ordinal())
            .collect::<Vec<_>>(),
        (1..=13).collect::<Vec<_>>(),
        "the complete exclusion order is an explicit contract"
    );

    use rust_electroanalysis_cli::{
        mhi_validation::partition::{EndpointPartitionSpec, EndpointSource, partition_endpoint},
        validation_config::RecordDecisionV1,
    };
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    let (root, _, dataset_path) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("dataset")
        .artifact;
    dataset.records[0].mechanism_source = None;
    dataset.records[0]
        .reference_endpoints
        .retain(|reference| !matches!(reference, ReferenceEndpointV1::Mechanism { .. }));
    write_test_dataset(&dataset_path, &mut dataset);
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &dataset_path,
    )
    .expect("precedence inputs");
    let endpoint = &protocol.mechanism_endpoints[0];
    let partition = partition_endpoint(
        &inputs,
        EndpointPartitionSpec {
            endpoint_id: &endpoint.endpoint_id,
            cohort_role: endpoint.cohort_role,
            domain: &endpoint.domain,
            required_strata: &endpoint.required_strata,
            reference_rule: &endpoint.reference_rule,
            source: EndpointSource::Mechanism,
            physical: false,
        },
    )
    .expect("precedence partition");
    let row = partition
        .rows
        .iter()
        .find(|row| row.record_id == "record_1")
        .expect("mutated row");
    assert_eq!(row.decision, RecordDecisionV1::Excluded);
    assert_eq!(
        row.primary_reason,
        Some(ExclusionReasonV1::MissingEndpointArtifactPath)
    );
    assert_eq!(
        row.secondary_reasons,
        [ExclusionReasonV1::MissingReferenceEndpoint]
    );
    assert_eq!(
        row.not_evaluated_reason.map(|reason| format!("{reason:?}")),
        Some("MissingEndpointArtifactPath".into())
    );
    fs::remove_dir_all(root).expect("precedence cleanup");
}

#[test]
fn phase_e_reader_accepts_only_canonical_schema4_scientific_inputs() {
    use rust_electroanalysis_cli::results::{MechanismAnalysisReport, SensorHealthAssessment};

    for mechanism in [
        "mechanism/supported.schema4.json",
        "mechanism/contradicted.schema4.json",
        "mechanism/all_levels.schema4.json",
    ] {
        let strict_mechanism = read_artifact_strict::<MechanismAnalysisReport>(&fixture(mechanism))
            .expect("canonical Phase-E schema-4 Phase-B input");
        assert_eq!(strict_mechanism.artifact.schema_version, 4, "{mechanism}");
    }
    for health in [
        "health/within_baseline.schema4.json",
        "health/alert.schema4.json",
        "health/all_status_reference_pairs.schema4.json",
    ] {
        let strict_health = read_artifact_strict::<SensorHealthAssessment>(&fixture(health))
            .expect("canonical Phase-E schema-4 Phase-C input");
        assert_eq!(strict_health.artifact.schema_version, 4, "{health}");
    }
}

#[test]
fn phase_e_reader_hard_fails_wrong_future_and_explicitly_excludes_legacy() {
    use rust_electroanalysis_cli::results::{MechanismAnalysisReport, SensorHealthAssessment};

    let root = temp("strict_schema4_reader");
    fs::create_dir_all(&root).expect("temporary directory");
    let future = root.join("mechanism.schema5.json");
    let source = fixture("mechanism/supported.schema4.json");
    let text = fs::read_to_string(source).expect("canonical mechanism fixture");
    let position = text.rfind("\"schema_version\": 4").expect("root schema");
    let mut mutated = text;
    mutated.replace_range(
        position..position + "\"schema_version\": 4".len(),
        "\"schema_version\": 5",
    );
    fs::write(&future, mutated).expect("future-schema mutation");
    assert!(read_artifact_strict::<MechanismAnalysisReport>(&future).is_err());

    let wrong_kind = root.join("mechanism-wrong-kind.schema4.json");
    let text = fs::read_to_string(fixture("mechanism/supported.schema4.json"))
        .expect("canonical mechanism fixture")
        .replacen(
            "\"artifact_kind\": \"mechanism_analysis\"",
            "\"artifact_kind\": \"health_assessment\"",
            1,
        );
    fs::write(&wrong_kind, text).expect("wrong-kind mutation");
    assert!(matches!(
        read_artifact_strict::<MechanismAnalysisReport>(&wrong_kind),
        Err(ArtifactError::IncompatibleKind { .. })
    ));

    let duplicate = root.join("health-duplicate.schema4.json");
    let text = fs::read_to_string(fixture("health/within_baseline.schema4.json"))
        .expect("canonical health fixture")
        .replacen(
            "\"assessment_id\": \"health:phase-e:within-baseline\",",
            "\"assessment_id\": \"health:phase-e:within-baseline\",\n  \"assessment_id\": \"duplicate\",",
            1,
        );
    fs::write(&duplicate, text).expect("duplicate nested-key mutation");
    assert!(matches!(
        read_artifact_strict::<SensorHealthAssessment>(&duplicate),
        Err(ArtifactError::DuplicateJsonKey { ref key, .. }) if key == "assessment_id"
    ));

    // The generic strict reader remains additive for historic callers.  The
    // Phase-E source boundary, rather than that existing API, rejects legacy
    // artifacts from scoring.
    assert!(
        read_artifact::<MechanismAnalysisReport>(&fixture("mechanism/legacy.schema3.json")).is_ok()
    );
    assert!(
        read_artifact::<SensorHealthAssessment>(&fixture("health/legacy.schema3.json")).is_ok()
    );
    let (source_root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
    let source_path = dataset_path
        .parent()
        .expect("dataset parent")
        .join("sources/mechanism_a.schema4.json");
    fs::copy(fixture("mechanism/legacy.schema3.json"), &source_path)
        .expect("legacy source mutation");
    let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
        .expect("test dataset")
        .artifact;
    {
        use sha2::{Digest, Sha256};
        dataset.records[0]
            .mechanism_source
            .as_mut()
            .expect("source")
            .source_file_sha256 = format!(
            "{:x}",
            Sha256::digest(fs::read(&source_path).expect("legacy source bytes"))
        );
    }
    write_test_dataset(&dataset_path, &mut dataset);
    let protocol_bytes = fs::read(protocol_path).expect("protocol bytes");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    assert!(matches!(
        ValidationInputs::read(
            &protocol,
            &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
            &dataset_path
        ),
        Err(MhiValidationError::Dataset(ref message))
            if message == "mechanism scientific sources must be schema-4 mechanism_analysis"
    ));
    fs::remove_dir_all(source_root).expect("legacy source cleanup");

    for mutation in [
        "accepted_unknown_scope",
        "known_experiment_scope",
        "known_family_scope",
    ] {
        let (source_root, protocol_path, dataset_path) = staged_dataset_with_scoreable_mechanism();
        let source_path = dataset_path
            .parent()
            .expect("dataset parent")
            .join("sources/mechanism_a.schema4.json");
        let mut source_wire: serde_json::Value =
            serde_json::from_slice(&fs::read(&source_path).expect("current source bytes"))
                .expect("current source JSON");
        source_wire["lineage"] = serde_json::json!({
            "LegacyUnknown": {
                "source_schema_version": 4,
                "reason": "MigrationInformationUnavailable"
            }
        });
        fs::write(
            &source_path,
            serde_json::to_vec_pretty(&source_wire).expect("legacy-unknown source JSON"),
        )
        .expect("legacy-unknown source");
        let source_hash = format!(
            "{:x}",
            Sha256::digest(fs::read(&source_path).expect("legacy-unknown source bytes"))
        );
        let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
            .expect("legacy-unknown dataset")
            .artifact;
        let record = &mut dataset.records[0];
        record.health_source = None;
        record
            .mechanism_source
            .as_mut()
            .expect("mechanism source")
            .source_file_sha256 = source_hash.clone();
        record.mechanism_source.as_mut().expect("mechanism source").expected_lineage =
            ExpectedLineageV1::LegacyUnknown {
                schema_version: 4,
                legacy_source_fingerprint: source_hash,
                reason: rust_electroanalysis_cli::results::LegacyLineageReasonV1::MigrationInformationUnavailable,
            };
        record.declared_scope.experiment_scope =
            rust_electroanalysis_cli::domain::ArtifactExperimentScope::Unknown;
        record.declared_scope.sensor_scope =
            rust_electroanalysis_cli::domain::ScopeKey::Unspecified;
        record.declared_scope.channel_scope =
            rust_electroanalysis_cli::domain::ScopeKey::Unspecified;
        record.declared_scope.acquisition_families =
            rust_electroanalysis_cli::domain::ArtifactAcquisitionFamilies::Unknown;
        match mutation {
            "accepted_unknown_scope" => {}
            "known_experiment_scope" => {
                record.declared_scope.experiment_scope =
                    rust_electroanalysis_cli::domain::ArtifactExperimentScope::Single {
                        experiment_id: rust_electroanalysis_cli::domain::ExperimentId(
                            "forged-experiment".into(),
                        ),
                    };
            }
            "known_family_scope" => {
                record.declared_scope.acquisition_families =
                    rust_electroanalysis_cli::domain::ArtifactAcquisitionFamilies::Known(vec![
                        rust_electroanalysis_cli::domain::AcquisitionFamilyId(
                            "forged-family".into(),
                        ),
                    ]);
            }
            _ => unreachable!(),
        }
        write_test_dataset(&dataset_path, &mut dataset);
        let protocol_bytes = fs::read(&protocol_path).expect("protocol bytes");
        let protocol = MhiValidationProtocolV1::from_toml(
            std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
        )
        .expect("protocol");
        let result = ValidationInputs::read(
            &protocol,
            &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
            &dataset_path,
        );
        if mutation == "accepted_unknown_scope" {
            result.expect("LegacyUnknown with exact unknown scope is accepted");
        } else {
            assert!(matches!(
                result,
                Err(MhiValidationError::Dataset(ref message))
                    if message == "LegacyUnknown source requires unknown declared scope"
            ));
        }
        fs::remove_dir_all(source_root).expect("legacy-unknown scope cleanup");
    }

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
    let source_dir = dataset.parent().expect("dataset parent").join("sources");
    fs::create_dir_all(&source_dir).expect("source fixture layout");
    for (fixture_name, staged_name) in [
        (
            "mechanism/supported.schema4.json",
            "mechanism_a.schema4.json",
        ),
        (
            "mechanism/all_levels.schema4.json",
            "mechanism_c.schema4.json",
        ),
        (
            "health/within_baseline.schema4.json",
            "health_a.schema4.json",
        ),
        ("health/alert.schema4.json", "health_c.schema4.json"),
    ] {
        fs::copy(fixture(fixture_name), source_dir.join(staged_name))
            .expect("copy literal scientific source");
    }
    (root, protocol, dataset)
}

fn copy_fixture_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("fixture tree destination");
    for entry in fs::read_dir(source).expect("fixture tree source") {
        let entry = entry.expect("fixture tree entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().expect("fixture tree type").is_dir() {
            copy_fixture_tree(&source_path, &destination_path);
        } else {
            fs::copy(source_path, destination_path).expect("fixture tree file");
        }
    }
}

fn staged_physical_inputs() -> (PathBuf, PathBuf) {
    let root = temp("public_physical_inputs");
    let protocol = root.join("protocol.toml");
    let dataset = root.join("dataset/input.schema1.json");
    fs::create_dir_all(dataset.parent().expect("dataset parent")).expect("dataset layout");
    fs::copy(fixture("protocol/physical_valid.toml"), &protocol).expect("physical protocol");
    fs::copy(fixture("dataset/physical_valid.schema1.json"), &dataset).expect("physical dataset");
    copy_fixture_tree(
        &fixture("mechanism/physical"),
        &dataset.parent().expect("dataset parent").join("physical"),
    );
    copy_fixture_tree(
        &fixture("health/physical"),
        &dataset.parent().expect("dataset parent").join("physical"),
    );
    fs::create_dir_all(dataset.parent().expect("dataset parent").join("lineage"))
        .expect("physical lineage layout");
    fs::copy(
        fixture("lineage/physical_complete.schema1.json"),
        dataset
            .parent()
            .expect("dataset parent")
            .join("lineage/physical_complete.schema1.json"),
    )
    .expect("physical lineage");
    (root, dataset)
}

/// Stages the same source topology the production reader consumes, while
/// retaining the permanent Phase-E source fixtures as the literal authority.
/// This is deliberately a test-owned directory: the R2 fixture ledger is
/// closed over the checked-in files and forbids aliases or generated permanent sources.
fn staged_dataset_with_scoreable_mechanism() -> (PathBuf, PathBuf, PathBuf) {
    staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    )
}

fn write_test_dataset(path: &Path, dataset: &mut MhiValidationDatasetV1) {
    dataset.cohort_semantic_sha256 = dataset
        .computed_cohort_semantic_sha256()
        .expect("test-owned dataset identity");
    fs::write(
        path,
        serde_json::to_vec_pretty(dataset).expect("dataset serialization"),
    )
    .expect("test-owned dataset write");
}

fn mechanism_reference(record_id: &str) -> ReferenceEndpointV1 {
    ReferenceEndpointV1::Mechanism {
        endpoint_id: "mechanism_endpoint".into(),
        reference_endpoint_id: format!("mechanism_reference_{record_id}"),
        reference_source_id: "reference_1".into(),
        hypothesis_id: "b-hypothesis".into(),
        outcome: MechanismReferenceOutcomeV1::Supports,
        method_id: "reference_method".into(),
        method_version: "1".into(),
        authority_id: "reference_authority".into(),
        blinding_state: BlindingStateV1::BlindedToAssessment,
        uncertainty: ReferenceUncertaintyV1::Quantified {
            measure_id: "uncertainty".into(),
            value: 0.1,
            unit: "1".into(),
        },
        limitations: vec![],
    }
}

fn add_reference_overlap_records(dataset: &mut MhiValidationDatasetV1) {
    let mut development = dataset.records[0].clone();
    development.record_id = "development_1".into();
    development.cohort_role = CohortRoleV1::Development;
    development.reference_endpoints = vec![mechanism_reference(&development.record_id)];
    let mut holdout = dataset.records[0].clone();
    holdout.record_id = "holdout_1".into();
    holdout.cohort_role = CohortRoleV1::Holdout;
    holdout.reference_endpoints = vec![mechanism_reference(&holdout.record_id)];
    dataset.records = vec![development, holdout];
    dataset.reference_sources = vec![ReferenceSourceAuthorityV1 {
        reference_source_id: "reference_1".into(),
        source_file_sha256: "1".repeat(64),
        evidence_origin: rust_electroanalysis_cli::validation_config::EvidenceOriginV1::Synthetic,
        dependency_completeness: ReferenceDependencyCompletenessV1::Complete,
        experiment_scope: rust_electroanalysis_cli::domain::ArtifactExperimentScope::Unknown,
        acquisition_families:
            rust_electroanalysis_cli::domain::ArtifactAcquisitionFamilies::Unknown,
        direct_dependencies: vec![],
    }];
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

    let (inputs, protocol, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let output = temp("exact_certified_route");
    run_mhi_validation(MhiValidationRunOptions {
        protocol,
        dataset,
        output_dir: output.clone(),
        overwrite: false,
    })
    .expect("exact certified route");
    assert_scientific_bundle(&output);
    assert_exact_golden_bundle(&output);
    fs::remove_dir_all(output).expect("output cleanup");
    fs::remove_dir_all(inputs).expect("input cleanup");
}

#[test]
fn phase_e_repeated_certified_validation_keeps_all_nine_bytes_identical() {
    let (inputs, protocol, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let first = temp("deterministic_first");
    let second = temp("deterministic_second");
    run_mhi_validation(MhiValidationRunOptions {
        protocol: protocol.clone(),
        dataset: dataset.clone(),
        output_dir: first.clone(),
        overwrite: false,
    })
    .expect("first certified validation");
    run_mhi_validation(MhiValidationRunOptions {
        protocol,
        dataset,
        output_dir: second.clone(),
        overwrite: false,
    })
    .expect("second certified validation");

    assert_exact_golden_bundle(&first);
    assert_exact_golden_bundle(&second);
    for relative in [
        "mhi_validation_report.schema1.json",
        "validation_execution_manifest.schema1.json",
        "validation_summary.md",
        "tables/cohort_coverage.csv",
        "tables/leakage_assessment.csv",
        "tables/mechanism_validation.csv",
        "tables/health_validation.csv",
        "tables/exclusion_ledger.csv",
        "tables/compatibility_matrix.csv",
    ] {
        assert_eq!(
            fs::read(first.join(relative)).expect("first deterministic bytes"),
            fs::read(second.join(relative)).expect("second deterministic bytes"),
            "successful output bytes {relative}"
        );
    }
    fs::remove_dir_all(first).expect("first deterministic output cleanup");
    fs::remove_dir_all(second).expect("second deterministic output cleanup");
    fs::remove_dir_all(inputs).expect("deterministic input cleanup");
}

#[test]
fn phase_e_cli_rejects_missing_unknown_and_raw_input_routes() {
    use clap::error::ErrorKind;

    let required = ["electroanalysis", "validation", "run"];
    assert_eq!(
        cli_error_kind(&[
            required[0],
            required[1],
            required[2],
            "--dataset",
            "d",
            "--output-dir",
            "o",
        ]),
        ErrorKind::MissingRequiredArgument,
        "--protocol is mandatory"
    );
    assert_eq!(
        cli_error_kind(&[
            required[0],
            required[1],
            required[2],
            "--protocol",
            "p",
            "--output-dir",
            "o",
        ]),
        ErrorKind::MissingRequiredArgument,
        "--dataset is mandatory"
    );
    assert_eq!(
        cli_error_kind(&[
            required[0],
            required[1],
            required[2],
            "--protocol",
            "p",
            "--dataset",
            "d",
        ]),
        ErrorKind::MissingRequiredArgument,
        "--output-dir is mandatory"
    );
    assert_eq!(
        cli_error_kind(&[
            required[0],
            required[1],
            required[2],
            "--protocol",
            "p",
            "--dataset",
            "d",
            "--output-dir",
            "o",
            "--unknown",
        ]),
        ErrorKind::UnknownArgument,
        "unrecognized Phase-E option is rejected by Clap"
    );
    assert_eq!(
        cli_error_kind(&[
            required[0],
            required[1],
            required[2],
            "--protocol",
            "p",
            "--dataset",
            "d",
            "--output-dir",
            "o",
            "--input",
            "raw.csv",
        ]),
        ErrorKind::UnknownArgument,
        "raw-input route is not part of the certified command"
    );
    assert_eq!(
        cli_error_kind(&[
            required[0],
            "alias",
            required[1],
            "run",
            "--protocol",
            "p",
            "--dataset",
            "d",
            "--output-dir",
            "o",
        ]),
        ErrorKind::InvalidSubcommand,
        "alias validation run is not a second Phase-E route"
    );

    let (root, protocol, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let output = temp("cli_input_symlink");
    let protocol_link = root.join("protocol-link.toml");
    let dataset_link = root.join("dataset-link.json");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&protocol, &protocol_link).expect("protocol symlink");
        let error = run_mhi_validation(MhiValidationRunOptions {
            protocol: protocol_link,
            dataset: dataset.clone(),
            output_dir: output.clone(),
            overwrite: false,
        })
        .expect_err("protocol symlink is not a Phase-E root file");
        assert!(matches!(error, MhiValidationError::UnsafePath(_)));
        assert!(!output.exists(), "rejected protocol writes no output");

        std::os::unix::fs::symlink(&dataset, &dataset_link).expect("dataset symlink");
        let error = run_mhi_validation(MhiValidationRunOptions {
            protocol,
            dataset: dataset_link,
            output_dir: output.clone(),
            overwrite: false,
        })
        .expect_err("dataset symlink is not a Phase-E root file");
        assert!(matches!(error, MhiValidationError::UnsafePath(_)));
        assert!(!output.exists(), "rejected dataset writes no output");
    }
    #[cfg(not(unix))]
    {
        let _ = (&protocol_link, &dataset_link, &output);
    }
    fs::remove_dir_all(root).expect("staged-input cleanup");
}

#[test]
fn phase_e_protocol_roundtrip_preserves_all_scientific_rules() {
    for (fixture_path, expected_hash) in [
        (
            "protocol/software_valid.toml",
            "c141c0f28246bb16edcca79e05caee065516c7b06770d08a536b31a43857cf17",
        ),
        (
            "protocol/physical_valid.toml",
            "a7c7938faed65ec9a3e346194a19f9d4401b66f9e81a8695aec2931f5cf97501",
        ),
    ] {
        let bytes = fs::read(fixture(fixture_path)).expect("fixture");
        let protocol = MhiValidationProtocolV1::from_toml(
            std::str::from_utf8(&bytes).expect("protocol UTF-8"),
        )
        .expect("protocol validates");
        let serialized = toml::to_string(&protocol).expect("typed protocol serialization");
        let reparsed = MhiValidationProtocolV1::from_toml(&serialized).expect("reparse");
        assert_eq!(
            reparsed, protocol,
            "typed protocol round trip {fixture_path}"
        );
        assert_eq!(protocol.schema_version, 1);
        assert_eq!(protocol.release_scope.len(), 1);
        assert_eq!(
            MhiValidationProtocolV1::sha256_of_bytes(&bytes),
            expected_hash
        );
    }
}

#[test]
fn phase_e_protocol_rejects_incomplete_conflicting_untrusted_and_nondeterministic_authority() {
    let text = fs::read_to_string(fixture("protocol/software_valid.toml")).expect("fixture");

    // Schema failures are intentionally distinct from semantic failures.
    // They must stop at the closed protocol reader before any dataset path is
    // opened, so each malformed input is asserted independently.
    for (case, invalid) in [
        (
            "missing required protocol field",
            text.replacen("title = \"Phase E software-only validation fixture\"\n", "", 1),
        ),
        (
            "unknown protocol field",
            text.replacen(
                "title = \"Phase E software-only validation fixture\"",
                "title = \"Phase E software-only validation fixture\"\nunknown_phase_e_field = true",
                1,
            ),
        ),
        (
            "duplicate protocol key",
            text.replacen(
                "protocol_id = \"phase_e_software_protocol\"",
                "protocol_id = \"phase_e_software_protocol\"\nprotocol_id = \"duplicate\"",
                1,
            ),
        ),
        (
            "mechanism allowed-outcomes field is not part of the frozen schema",
            text.replacen(
                "critical_policy = \"any_contradicted_record_fails\"",
                "critical_policy = \"any_contradicted_record_fails\"\nallowed_outcomes = [\"supports\"]",
                1,
            ),
        ),
    ] {
        assert!(
            matches!(MhiValidationProtocolV1::from_toml(&invalid), Err(MhiValidationError::Toml(_))),
            "{case}"
        );
    }

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    if let AcceptanceRuleV1::Rate { threshold, .. } =
        &mut protocol.mechanism_endpoints[0].acceptance_rules[0]
    {
        *threshold = f64::NAN;
    }
    protocol_error(
        protocol.validate(),
        "rate threshold must be finite in [0,1]",
    );

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    if let AcceptanceRuleV1::Rate { threshold, .. } =
        &mut protocol.mechanism_endpoints[0].acceptance_rules[0]
    {
        *threshold = -0.0;
    }
    protocol_error(
        protocol.validate(),
        "rate threshold must be finite in [0,1]",
    );

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.health_endpoints[0].predicted_negative_statuses = vec!["watch".into()];
    protocol_error(
        protocol.validate(),
        "health status sets must be a disjoint exact partition",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.health_endpoints[0]
        .predicted_positive_statuses
        .push("watch".into());
    protocol_error(
        protocol.validate(),
        "health status sets must be a disjoint exact partition",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.health_endpoints[0]
        .reference_positive_labels
        .push("normal".into());
    protocol_error(
        protocol.validate(),
        "health reference classes must be a disjoint exact partition",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.health_endpoints[0].reference_negative_labels = vec!["alert".into()];
    protocol_error(
        protocol.validate(),
        "health reference classes must be a disjoint exact partition",
    );

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    if let AcceptanceRuleV1::Rate { metric, .. } =
        &mut protocol.mechanism_endpoints[0].acceptance_rules[0]
    {
        *metric = RateMetricV1::Coverage;
    }
    protocol_error(
        protocol.validate(),
        "acceptance metric is not valid for endpoint kind",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    if let AcceptanceRuleV1::Rate { metric, target, .. } =
        &mut protocol.health_endpoints[0].acceptance_rules[0]
    {
        *metric = RateMetricV1::BalancedAccuracy;
        *target = RateTargetV1::LowerConfidenceBound;
    }
    protocol_error(
        protocol.validate(),
        "balanced_accuracy has point_estimate only",
    );

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0].acceptance_rules.clear();
    protocol_error(protocol.validate(), "acceptance_rules must be nonempty");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    if let AcceptanceRuleV1::Rate { comparator, .. } =
        &mut protocol.mechanism_endpoints[0].acceptance_rules[0]
    {
        *comparator = ComparatorV1::LessThanOrEqual;
    }
    protocol_error(
        protocol.validate(),
        "mechanism endpoints require a support_fraction greater_than_or_equal rule",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.health_endpoints[0].acceptance_rules.remove(2);
    protocol_error(
        protocol.validate(),
        "health endpoints require coverage, sensitivity, and specificity greater_than_or_equal rules",
    );

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.health_endpoints[0].endpoint_id = "mechanism_endpoint".into();
    protocol_error(protocol.validate(), "endpoint IDs must be globally unique");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    if let AcceptanceRuleV1::Rate { rule_id, .. } =
        &mut protocol.health_endpoints[0].acceptance_rules[2]
    {
        *rule_id = "sensitivity".into();
    }
    protocol_error(protocol.validate(), "acceptance-rule IDs must be unique");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    if let ReferenceAuthorityRuleV1::Mechanism {
        allowed_authority_ids,
        ..
    } = &mut protocol.mechanism_endpoints[0].reference_rule
    {
        allowed_authority_ids.push("reference_authority".into());
    }
    protocol_error(
        protocol.validate(),
        "allowed_authority_ids must be sorted, nonempty, and duplicate-free",
    );

    let valid_stratum = RequiredStratumV1 {
        stratum_id: "analyte_pb".into(),
        predicates: vec![StratumPredicateV1::AnalyteEquals { id: "Pb".into() }],
        minimum_eligible_records: 1,
        minimum_independent_families: 1,
    };
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0].required_strata =
        vec![valid_stratum.clone(), valid_stratum.clone()];
    protocol_error(
        protocol.validate(),
        "strata must be unique and have positive minima",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0].required_strata = vec![RequiredStratumV1 {
        stratum_id: "empty".into(),
        predicates: vec![],
        minimum_eligible_records: 1,
        minimum_independent_families: 1,
    }];
    protocol_error(
        protocol.validate(),
        "required stratum predicates must be nonempty",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    let mut zero_minimum = valid_stratum.clone();
    zero_minimum.minimum_eligible_records = 0;
    protocol.mechanism_endpoints[0].required_strata = vec![zero_minimum];
    protocol_error(
        protocol.validate(),
        "strata must be unique and have positive minima",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0].required_strata = vec![RequiredStratumV1 {
        stratum_id: "repeated_axis".into(),
        predicates: vec![
            StratumPredicateV1::AnalyteEquals { id: "Pb".into() },
            StratumPredicateV1::AnalyteEquals { id: "Cd".into() },
        ],
        minimum_eligible_records: 1,
        minimum_independent_families: 1,
    }];
    protocol_error(
        protocol.validate(),
        "stratum predicates must be canonically ordered and use each axis once",
    );

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.target_domain.temperature = TemperatureSelectorV1::Bands {
        bands: vec![
            TemperatureBandV1 {
                lower_kelvin_inclusive: 290.0,
                upper_kelvin_exclusive: 300.0,
            },
            TemperatureBandV1 {
                lower_kelvin_inclusive: 299.0,
                upper_kelvin_exclusive: 310.0,
            },
        ],
    };
    protocol_error(
        protocol.validate(),
        "target_domain.temperature bands must be ordered and non-overlapping",
    );

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0].cohort_role = CohortRoleV1::Development;
    protocol_error(protocol.validate(), "development is not scoreable");

    let mut protocol = protocol_fixture("protocol/physical_valid.toml");
    protocol.mechanism_endpoints[0].minimum_eligible_records = 1;
    protocol_error(
        protocol.validate(),
        "physical claims require domain-equal holdout endpoints with minima of two",
    );
    let mut protocol = protocol_fixture("protocol/physical_valid.toml");
    protocol.physical_approval_authority = PhysicalApprovalAuthorityV1::NotRequested;
    protocol_error(
        protocol.validate(),
        "physical approval authority must be requested iff a claim is physical",
    );
    let mut protocol = protocol_fixture("protocol/physical_valid.toml");
    protocol.physical_approval_authority = PhysicalApprovalAuthorityV1::EmbeddedTrustRoot {
        trust_root_id: String::new(),
    };
    protocol_error(
        protocol.validate(),
        "physical_approval_authority.trust_root_id must be a stable ID",
    );

    // Exact equality, rather than either containment direction, is required
    // between every supporting endpoint and the claim it supports.
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0].domain.analyte = CategoricalSelectorV1::Allowed {
        ids: vec!["Pb".into()],
    };
    assert!(matches!(
        protocol.validate(),
        Err(MhiValidationError::SupportingEndpointClaimDomainMismatch)
    ));
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.release_scope[0].domain.analyte = CategoricalSelectorV1::Allowed {
        ids: vec!["Pb".into()],
    };
    assert!(matches!(
        protocol.validate(),
        Err(MhiValidationError::SupportingEndpointClaimDomainMismatch)
    ));
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.release_scope[0].domain.analyte = CategoricalSelectorV1::Allowed {
        ids: vec!["Cd".into(), "Pb".into()],
    };
    protocol.health_endpoints[0].domain.analyte = CategoricalSelectorV1::Allowed {
        ids: vec!["Cd".into(), "Pb".into()],
    };
    protocol.mechanism_endpoints[0].domain.analyte = CategoricalSelectorV1::Allowed {
        ids: vec!["Pb".into()],
    };
    assert!(matches!(
        protocol.validate(),
        Err(MhiValidationError::SupportingEndpointClaimDomainMismatch)
    ));

    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0]
        .acceptance_rules
        .push(AcceptanceRuleV1::Rate {
            rule_id: "support_upper".into(),
            metric: RateMetricV1::SupportFraction,
            target: RateTargetV1::PointEstimate,
            comparator: ComparatorV1::LessThanOrEqual,
            threshold: 0.4,
        });
    protocol_error(
        protocol.validate(),
        "acceptance-rule bounds are contradictory",
    );

    // Each mandatory rule is independently authoritative.  Mutating a
    // different rule must never be masked by a generic parse failure.
    for removed in [0usize, 1, 2] {
        let mut protocol = protocol_fixture("protocol/software_valid.toml");
        protocol.health_endpoints[0]
            .acceptance_rules
            .remove(removed);
        protocol_error(
            protocol.validate(),
            "health endpoints require coverage, sensitivity, and specificity greater_than_or_equal rules",
        );
    }
    for changed in [0usize, 1, 2] {
        let mut protocol = protocol_fixture("protocol/software_valid.toml");
        let AcceptanceRuleV1::Rate { comparator, .. } =
            &mut protocol.health_endpoints[0].acceptance_rules[changed]
        else {
            unreachable!("health fixture uses rate rules")
        };
        *comparator = ComparatorV1::LessThanOrEqual;
        protocol_error(
            protocol.validate(),
            "health endpoints require coverage, sensitivity, and specificity greater_than_or_equal rules",
        );
    }
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0].minimum_eligible_records = 0;
    protocol_error(protocol.validate(), "endpoint minima must be positive");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.health_endpoints[0].minimum_independent_families = 0;
    protocol_error(protocol.validate(), "endpoint minima must be positive");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.mechanism_endpoints[0].required_strata = vec![RequiredStratumV1 {
        stratum_id: "invalid_temperature".into(),
        predicates: vec![StratumPredicateV1::TemperatureBand {
            lower_kelvin_inclusive: 300.0,
            upper_kelvin_exclusive: 300.0,
        }],
        minimum_eligible_records: 1,
        minimum_independent_families: 1,
    }];
    protocol_error(
        protocol.validate(),
        "stratum temperature band must be finite positive lower < upper",
    );
    let mut protocol = protocol_fixture("protocol/physical_valid.toml");
    protocol.mechanism_endpoints[0].minimum_independent_families = 1;
    protocol_error(
        protocol.validate(),
        "physical claims require domain-equal holdout endpoints with minima of two",
    );

    // SCI-P1-001: temperature bands are the exact lower-inclusive /
    // upper-exclusive set union.  E-T04 owns the protocol/domain authority
    // for adjacency, union containment, semantic equality, and gaps.
    fn temperature_selector(bands: &[(f64, f64)]) -> TemperatureSelectorV1 {
        TemperatureSelectorV1::Bands {
            bands: bands
                .iter()
                .map(|(lower, upper)| TemperatureBandV1 {
                    lower_kelvin_inclusive: *lower,
                    upper_kelvin_exclusive: *upper,
                })
                .collect(),
        }
    }

    fn set_temperature_domains(
        protocol: &mut MhiValidationProtocolV1,
        target: &[(f64, f64)],
        endpoint_and_claim: &[(f64, f64)],
    ) {
        protocol.target_domain.temperature = temperature_selector(target);
        let endpoint_temperature = temperature_selector(endpoint_and_claim);
        for endpoint in &mut protocol.mechanism_endpoints {
            endpoint.domain.temperature = endpoint_temperature.clone();
        }
        for endpoint in &mut protocol.health_endpoints {
            endpoint.domain.temperature = endpoint_temperature.clone();
        }
        protocol.release_scope[0].domain.temperature = endpoint_temperature;
    }

    fn domain_key(temperature_kelvin: f64) -> DomainKeyV1 {
        DomainKeyV1 {
            analyte_id: "analyte".into(),
            matrix_id: "matrix".into(),
            sensor_design_id: "design".into(),
            sensor_id: "sensor".into(),
            campaign_id: "campaign".into(),
            temperature_kelvin,
        }
    }

    // Adjacent bands are valid, while actual overlap and unsorted bands keep
    // the existing protocol error authority.
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 300.0), (300.0, 310.0)],
        &[(295.0, 305.0)],
    );
    protocol.validate().expect("adjacent bands are valid");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 301.0), (300.0, 310.0)],
        &[(295.0, 305.0)],
    );
    protocol_error(
        protocol.validate(),
        "target_domain.temperature bands must be ordered and non-overlapping",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(300.0, 310.0), (290.0, 300.0)],
        &[(295.0, 305.0)],
    );
    protocol_error(
        protocol.validate(),
        "target_domain.temperature bands must be ordered and non-overlapping",
    );

    // Counterexample 1 and the general union sweep: one left interval can be
    // covered by two or three adjacent right intervals, and multiple left
    // intervals can be covered by different portions of that union.
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 300.0), (300.0, 310.0)],
        &[(295.0, 305.0)],
    );
    protocol
        .validate()
        .expect("[295,305) is covered by two adjacent bands");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 295.0), (295.0, 300.0), (300.0, 310.0)],
        &[(292.0, 305.0)],
    );
    protocol
        .validate()
        .expect("one left band is covered by three adjacent bands");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 300.0), (300.0, 310.0), (310.0, 320.0)],
        &[(292.0, 295.0), (305.0, 315.0)],
    );
    protocol
        .validate()
        .expect("multiple left bands are covered by the right union");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 295.0), (300.0, 310.0)],
        &[(300.0, 305.0)],
    );
    protocol
        .validate()
        .expect("irrelevant gap before the left union is allowed");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 300.0), (305.0, 310.0)],
        &[(292.0, 295.0)],
    );
    protocol
        .validate()
        .expect("irrelevant gap after the left union is allowed");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 300.0), (301.0, 310.0)],
        &[(299.0, 302.0)],
    );
    protocol_error(
        protocol.validate(),
        "mechanism endpoint domain exceeds target_domain",
    );
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(&mut protocol, &[(290.0, 310.0)], &[(290.0, 310.0)]);
    protocol
        .validate()
        .expect("exact equal bands are contained");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(&mut protocol, &[(290.0, 310.0)], &[(295.0, 305.0)]);
    protocol
        .validate()
        .expect("a narrower single interval is contained");

    // Counterexamples 2 and 5: endpoint/claim equality is semantic mutual
    // subset, so split and merged adjacent representations are interchangeable
    // without mutating either declared vector.
    for (endpoint_bands, claim_bands) in [
        (&[(290.0, 295.0), (295.0, 300.0)][..], &[(290.0, 300.0)][..]),
        (&[(290.0, 300.0)][..], &[(290.0, 295.0), (295.0, 300.0)][..]),
    ] {
        let mut protocol = protocol_fixture("protocol/software_valid.toml");
        let endpoint_temperature = temperature_selector(endpoint_bands);
        let claim_temperature = temperature_selector(claim_bands);
        for endpoint in &mut protocol.mechanism_endpoints {
            endpoint.domain.temperature = endpoint_temperature.clone();
        }
        for endpoint in &mut protocol.health_endpoints {
            endpoint.domain.temperature = endpoint_temperature.clone();
        }
        protocol.release_scope[0].domain.temperature = claim_temperature;
        assert_ne!(
            protocol.mechanism_endpoints[0].domain.temperature,
            protocol.release_scope[0].domain.temperature,
            "the regression uses distinct structural segmentations"
        );
        protocol
            .validate()
            .expect("semantically equal endpoint and claim domains validate");
    }

    // Boundary semantics remain exact: the first interval excludes 300, the
    // adjacent second interval includes it, with no tolerance or epsilon.
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.target_domain.temperature = temperature_selector(&[(290.0, 300.0)]);
    let just_below = f64::from_bits(300.0f64.to_bits() - 1);
    assert!(protocol.target_domain.contains(&domain_key(just_below)));
    assert!(!protocol.target_domain.contains(&domain_key(300.0)));
    protocol.target_domain.temperature = temperature_selector(&[(290.0, 300.0), (300.0, 310.0)]);
    assert!(protocol.target_domain.contains(&domain_key(300.0)));
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 300.0), (300.0, 310.0)],
        &[(300.0, 305.0)],
    );
    protocol
        .validate()
        .expect("[300,305) is covered by the interval beginning at 300");

    // Counterexample 6: a claim that is broader than its supporting endpoint
    // remains rejected even when the bands are adjacent.
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    let endpoint_temperature = temperature_selector(&[(290.0, 300.0)]);
    let claim_temperature = temperature_selector(&[(290.0, 300.0), (300.0, 310.0)]);
    for endpoint in &mut protocol.mechanism_endpoints {
        endpoint.domain.temperature = endpoint_temperature.clone();
    }
    for endpoint in &mut protocol.health_endpoints {
        endpoint.domain.temperature = endpoint_temperature.clone();
    }
    protocol.release_scope[0].domain.temperature = claim_temperature;
    assert!(matches!(
        protocol.validate(),
        Err(MhiValidationError::SupportingEndpointClaimDomainMismatch)
    ));

    // Target-domain subset uses the same union operation.  A spanning claim
    // is accepted, while a genuine target gap rejects the claim before source
    // reading or scoring.
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(290.0, 300.0), (300.0, 310.0)],
        &[(295.0, 305.0)],
    );
    protocol
        .validate()
        .expect("target union contains a spanning claim");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.target_domain.temperature = temperature_selector(&[(290.0, 300.0), (301.0, 310.0)]);
    let endpoint_temperature = temperature_selector(&[(299.0, 300.0)]);
    let claim_temperature = temperature_selector(&[(299.0, 302.0)]);
    for endpoint in &mut protocol.mechanism_endpoints {
        endpoint.domain.temperature = endpoint_temperature.clone();
    }
    for endpoint in &mut protocol.health_endpoints {
        endpoint.domain.temperature = endpoint_temperature.clone();
    }
    protocol.release_scope[0].domain.temperature = claim_temperature;
    protocol_error(
        protocol.validate(),
        "release claim domain exceeds target_domain",
    );

    // Categorical mutual-subset behavior remains unchanged across every other
    // domain axis, including AnyDeclared.
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    for endpoint in &mut protocol.mechanism_endpoints {
        endpoint.domain.analyte = CategoricalSelectorV1::Allowed {
            ids: vec!["A".into()],
        };
    }
    for endpoint in &mut protocol.health_endpoints {
        endpoint.domain.analyte = CategoricalSelectorV1::Allowed {
            ids: vec!["A".into()],
        };
    }
    protocol.release_scope[0].domain.analyte = CategoricalSelectorV1::Allowed {
        ids: vec!["A".into()],
    };
    protocol.validate().expect("Allowed {A} equals Allowed {A}");
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    for endpoint in &mut protocol.mechanism_endpoints {
        endpoint.domain.analyte = CategoricalSelectorV1::Allowed {
            ids: vec!["A", "B"].into_iter().map(String::from).collect(),
        };
    }
    for endpoint in &mut protocol.health_endpoints {
        endpoint.domain.analyte = CategoricalSelectorV1::Allowed {
            ids: vec!["A", "B"].into_iter().map(String::from).collect(),
        };
    }
    protocol.release_scope[0].domain.analyte = CategoricalSelectorV1::Allowed {
        ids: vec!["A".into()],
    };
    assert!(matches!(
        protocol.validate(),
        Err(MhiValidationError::SupportingEndpointClaimDomainMismatch)
    ));
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    protocol.release_scope[0].domain.analyte = CategoricalSelectorV1::Allowed {
        ids: vec!["A".into()],
    };
    assert!(matches!(
        protocol.validate(),
        Err(MhiValidationError::SupportingEndpointClaimDomainMismatch)
    ));

    // The physical protocol path uses the same semantic equality rule without
    // changing the signed KAT fixtures or their serialized bytes.
    let mut protocol = protocol_fixture("protocol/physical_valid.toml");
    set_temperature_domains(
        &mut protocol,
        &[(298.0, 299.0)],
        &[(298.0, 298.5), (298.5, 299.0)],
    );
    protocol.release_scope[0].domain.temperature = temperature_selector(&[(298.0, 299.0)]);
    protocol
        .validate()
        .expect("physical split endpoint and merged claim domains validate");
    let mut protocol = protocol_fixture("protocol/physical_valid.toml");
    set_temperature_domains(&mut protocol, &[(298.0, 299.0)], &[(298.0, 299.0)]);
    protocol.release_scope[0].domain.temperature =
        temperature_selector(&[(298.0, 298.5), (298.5, 299.0)]);
    protocol
        .validate()
        .expect("physical merged endpoint and split claim domains validate");
}

#[test]
fn phase_e_dataset_schema1_roundtrip_is_closed_and_canonical() {
    let path = fixture("dataset/software_valid.schema1.json");
    let strict = read_artifact_strict::<MhiValidationDatasetV1>(&path)
        .expect("strict reader accepts canonical schema-1 dataset");
    assert_eq!(strict.artifact.schema_version, 1);
    assert_eq!(strict.artifact.records.len(), 2);
    assert_eq!(strict.artifact.records[0].record_id, "record_1");
    assert_eq!(strict.source_bytes, fs::read(&path).expect("fixture bytes"));
    assert_eq!(
        strict.source_file_sha256,
        "59955a38c93193740ffeb7abc9b4b2a2e5df37ea715c041d1978c7819a1ff657"
    );
    let legacy: MhiValidationDatasetV1 =
        read_artifact(&path).expect("legacy reader accepts valid Phase-E dataset");
    assert_eq!(legacy, strict.artifact, "unmodified typed read is exact");

    let directory = temp("dataset_canonical_order");
    fs::create_dir_all(&directory).expect("temporary directory");
    let source = fs::read_to_string(&path).expect("dataset fixture");
    let start = source
        .find("    {\n      \"record_id\"")
        .expect("record start");
    let end = source[start..]
        .find("\n    }\n  ],")
        .map(|offset| start + offset + "\n    }".len())
        .expect("record end");
    let record = &source[start..end];
    let earlier = record.replacen("\"record_1\"", "\"record_0\"", 1);
    let reordered = source.replacen(record, &format!("{record},\n{earlier}"), 1);
    let reordered_path = directory.join("reordered.schema1.json");
    fs::write(&reordered_path, reordered).expect("reordered dataset");
    assert!(matches!(
        read_artifact_strict::<MhiValidationDatasetV1>(&reordered_path),
        Err(ArtifactError::Validation { ref message })
            if message == "dataset records must be canonical and unique"
    ));

    let duplicated = source.replacen(
        "\"record_id\": \"record_2\"",
        "\"record_id\": \"record_1\"",
        1,
    );
    let duplicate_path = directory.join("duplicate.schema1.json");
    fs::write(&duplicate_path, duplicated).expect("duplicate dataset");
    assert!(matches!(
        read_artifact_strict::<MhiValidationDatasetV1>(&duplicate_path),
        Err(ArtifactError::Validation { ref message })
            if message == "dataset records must be canonical and unique"
    ));

    let mut arbitrary_domain = read_artifact_strict::<MhiValidationDatasetV1>(&path)
        .expect("canonical dataset")
        .artifact;
    arbitrary_domain.records[0].domain.analyte_id = "unlisted-but-valid".into();
    let arbitrary_domain_path = directory.join("arbitrary_domain.schema1.json");
    write_test_dataset(&arbitrary_domain_path, &mut arbitrary_domain);
    read_artifact_strict::<MhiValidationDatasetV1>(&arbitrary_domain_path)
        .expect("AnyDeclared-compatible categorical ID remains structurally valid");

    let mut invalid_domain = arbitrary_domain.clone();
    invalid_domain.records[0].domain.analyte_id = "not a stable id".into();
    let invalid_domain_path = directory.join("invalid_domain.schema1.json");
    write_test_dataset(&invalid_domain_path, &mut invalid_domain);
    assert!(matches!(
        read_artifact_strict::<MhiValidationDatasetV1>(&invalid_domain_path),
        Err(ArtifactError::Validation { ref message }) if message.contains("analyte_id")
    ));

    let mut unknown_provenance: serde_json::Value = serde_json::from_str(&source).expect("JSON");
    unknown_provenance["provenance"]["unexpected"] = serde_json::Value::Bool(true);
    let unknown_provenance_path = directory.join("unknown_provenance.schema1.json");
    fs::write(
        &unknown_provenance_path,
        serde_json::to_vec_pretty(&unknown_provenance).expect("provenance mutation"),
    )
    .expect("unknown provenance");
    assert!(read_artifact_strict::<MhiValidationDatasetV1>(&unknown_provenance_path).is_err());

    let mut malformed_warning: serde_json::Value = serde_json::from_str(&source).expect("JSON");
    malformed_warning["warnings"] = serde_json::json!([{"unexpected": true}]);
    let malformed_warning_path = directory.join("malformed_warning.schema1.json");
    fs::write(
        &malformed_warning_path,
        serde_json::to_vec_pretty(&malformed_warning).expect("warning mutation"),
    )
    .expect("malformed warning");
    assert!(read_artifact_strict::<MhiValidationDatasetV1>(&malformed_warning_path).is_err());
    fs::remove_dir_all(directory).expect("dataset-mutation cleanup");
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

    // R2 closes every nested catalog object for Phase E without changing the
    // legacy parser.  The externally tagged enum cases remain legacy parser
    // rejections too; ordinary struct fields retain their permissive baseline.
    for (case, mutated, legacy_accepts) in [
        (
            "catalog node",
            text.replacen(
                "      \"identity\": {",
                "      \"phase_e_unknown\": true,\n      \"identity\": {",
                1,
            ),
            true,
        ),
        (
            "identity",
            text.replacen(
                "        \"artifact_kind\": \"state_estimation\",",
                "        \"artifact_kind\": \"state_estimation\",\n        \"phase_e_unknown\": true,",
                1,
            ),
            true,
        ),
        (
            "dependency",
            text.replacen(
                "          \"role\": \"TransformationInput\"\n        }",
                "          \"role\": \"TransformationInput\",\n          \"phase_e_unknown\": true\n        }",
                1,
            ),
            true,
        ),
        (
            "single experiment scope payload",
            text.replacen(
                "            \"experiment_id\": \"b-e2e-1\"",
                "            \"experiment_id\": \"b-e2e-1\",\n            \"phase_e_unknown\": true",
                1,
            ),
            true,
        ),
        (
            "known acquisition families payload",
            text.replacen(
                "          \"Known\": [\n            \"b-family-estimation\"\n          ]",
                "          \"Known\": [\n            \"b-family-estimation\"\n          ],\n          \"phase_e_unknown\": true",
                1,
            ),
            false,
        ),
        (
            "specific scope-key tag",
            text.replacen(
                "        \"sensor_scope\": \"Unspecified\",",
                "        \"sensor_scope\": {\"Specific\": \"sensor-a\", \"phase_e_unknown\": true},",
                1,
            ),
            false,
        ),
    ] {
        fs::write(&path, mutated).expect("nested mutation");
        assert!(
            read_artifact_lineage_catalog_strict(&path).is_err(),
            "strict Phase-E catalog reader rejects unknown {case}"
        );
        assert_eq!(
            read_artifact_lineage_catalog(&path).is_ok(),
            legacy_accepts,
            "legacy reader baseline for {case}"
        );
    }
    fs::remove_dir_all(directory).expect("cleanup");
}

#[test]
fn phase_e_lineage_fixture_graphs_exercise_missing_cycle_shared_and_reference_cases() {
    use rust_electroanalysis_cli::domain::{
        ArtifactId, LineageCatalogReadError, LineageResolutionStatus, resolve_known_artifact_id,
    };

    let id = |hex: char| ArtifactId(format!("sha256:{}", hex.to_string().repeat(64)));
    let read = |name: &str| {
        read_artifact_lineage_catalog_strict(&fixture(name))
            .expect("strictly valid lineage graph")
            .catalog
    };

    let missing = read("lineage/missing_ancestor.schema1.json");
    let missing_resolution = resolve_known_artifact_id(&id('b'), &missing);
    assert_eq!(
        missing_resolution.status,
        LineageResolutionStatus::Incomplete
    );
    assert_eq!(missing_resolution.missing_artifact_ids, vec![id('a')]);

    let cycle = read("lineage/cycle.schema1.json");
    let cycle_resolution = resolve_known_artifact_id(&id('b'), &cycle);
    assert_eq!(
        cycle_resolution.status,
        LineageResolutionStatus::CycleDetected
    );
    assert!(cycle_resolution.ancestor_artifact_ids.contains(&id('a')));

    let shared = read("lineage/shared_ancestor.schema1.json");
    let left = resolve_known_artifact_id(&id('b'), &shared);
    let right = resolve_known_artifact_id(&id('c'), &shared);
    assert_eq!(left.status, LineageResolutionStatus::Complete);
    assert_eq!(right.status, LineageResolutionStatus::Complete);
    assert!(left.ancestor_artifact_ids.contains(&id('a')));
    assert!(right.ancestor_artifact_ids.contains(&id('a')));

    let intermediate = read("lineage/reference_intermediate.schema1.json");
    let intermediate_resolution = resolve_known_artifact_id(&id('c'), &intermediate);
    assert_eq!(
        intermediate_resolution.status,
        LineageResolutionStatus::Complete
    );
    assert_eq!(
        intermediate_resolution.ancestor_artifact_ids,
        vec![id('a'), id('b')]
    );

    assert!(matches!(
        read_artifact_lineage_catalog_strict(&fixture("lineage/root_mismatch.schema1.json")),
        Err(LineageCatalogReadError::KeyIdentityMismatch { .. })
    ));
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
        let value = wilson_95_checked(numerator, denominator);
        match vector["kind"].as_str().expect("kind") {
            "unavailable" => assert!(matches!(
                value.expect("valid unavailable vector"),
                MetricValueV1::Unavailable { .. }
            )),
            "hard_error" => assert_eq!(
                value.expect_err("invalid vector must hard-fail"),
                vector["reason"].as_str().expect("hard error reason")
            ),
            "available" => {
                let MetricValueV1::Available {
                    point_estimate,
                    lower_confidence_bound,
                    upper_confidence_bound,
                    ..
                } = value.expect("available vector")
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
        "dataset/synthetic_perfect.schema1.json",
    );
    let fixture_dataset = read_artifact_strict::<MhiValidationDatasetV1>(&fixture(
        "dataset/synthetic_perfect.schema1.json",
    ))
    .expect("strict synthetic fixture");
    assert!(
        fixture_dataset
            .artifact
            .records
            .iter()
            .all(|record| record.evidence_origin
                == rust_electroanalysis_cli::validation_config::EvidenceOriginV1::Synthetic)
    );
    let output = temp("software_run");
    run_mhi_validation(MhiValidationRunOptions {
        protocol: protocol.clone(),
        dataset: dataset.clone(),
        output_dir: output.clone(),
        overwrite: false,
    })
    .expect("software-only run");
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(output.join("mhi_validation_report.schema1.json")).expect("report"),
    )
    .expect("JSON");
    assert_eq!(report["overall_status"], "meets_protocol");
    assert_eq!(
        report["release_claims"][0]["outcome"],
        "software_validated_only"
    );
    assert_eq!(
        fs::read_dir(output.join("tables")).expect("tables").count(),
        6
    );

    // Neither an operational pathname nor a method-flavored filename can
    // manufacture physical evidence.  The declared origin is the only
    // authority, and this renamed synthetic source must retain the software
    // release ceiling.
    let renamed_source = dataset
        .parent()
        .expect("dataset parent")
        .join("sources/physical_method_claimed.schema4.json");
    let original_source = dataset
        .parent()
        .expect("dataset parent")
        .join("sources/mechanism_a.schema4.json");
    fs::rename(&original_source, &renamed_source).expect("rename synthetic source");
    let mut renamed_dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset)
        .expect("staged synthetic dataset")
        .artifact;
    renamed_dataset.records[0]
        .mechanism_source
        .as_mut()
        .expect("synthetic mechanism source")
        .relative_path = "sources/physical_method_claimed.schema4.json".into();
    write_test_dataset(&dataset, &mut renamed_dataset);
    let renamed_output = temp("synthetic_renamed_run");
    run_mhi_validation(MhiValidationRunOptions {
        protocol: protocol.clone(),
        dataset: dataset.clone(),
        output_dir: renamed_output.clone(),
        overwrite: false,
    })
    .expect("renamed synthetic run");
    let renamed_report: serde_json::Value = serde_json::from_slice(
        &fs::read(renamed_output.join("mhi_validation_report.schema1.json"))
            .expect("renamed report"),
    )
    .expect("renamed report JSON");
    assert_eq!(
        renamed_report["release_claims"][0]["outcome"],
        "software_validated_only"
    );
    fs::remove_dir_all(renamed_output).expect("renamed-output cleanup");
    fs::remove_dir_all(output).expect("cleanup");
    fs::remove_dir_all(inputs).expect("cleanup staged inputs");
}

#[test]
fn phase_e_physical_claim_requires_dual_signature_embedded_trust_and_power() {
    let protocol = fixture("protocol/physical_valid.toml");
    let output = temp("unprovisioned_physical");
    let error = run_mhi_validation(MhiValidationRunOptions {
        protocol,
        // Production authority is embedded and intentionally UNPROVISIONED;
        // the runner must fail before opening any caller-controlled dataset.
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
    assert_scientific_bundle(&output);
    assert!(run_mhi_validation(options.clone()).is_err());
    run_mhi_validation(MhiValidationRunOptions {
        overwrite: true,
        ..options
    })
    .expect("managed replacement");
    assert_scientific_bundle(&output);
    let parent = output.parent().expect("output parent");
    let output_name = output.file_name().expect("output name").to_string_lossy();
    assert!(
        !parent
            .join(format!(".{output_name}.phase-e-stage"))
            .exists()
    );
    assert!(
        !parent
            .join(format!(".{output_name}.phase-e-backup"))
            .exists()
    );
    fs::remove_dir_all(output).expect("cleanup");
    fs::remove_dir_all(inputs).expect("cleanup staged inputs");
}

#[test]
fn phase_e_publication_is_locked_no_clobber_crash_durable_and_residue_exact() {
    let state_table: serde_json::Value = serde_json::from_slice(
        &fs::read(fixture("expected/publication_state_table.schema1.json"))
            .expect("publication-state oracle"),
    )
    .expect("publication-state JSON");
    let states = state_table["states"].as_array().expect("state rows");
    assert_eq!(states.len(), 6, "literal publication state cases");
    for required in [
        "create_new_success",
        "replace_success",
        "precheck_old_changed",
        "exchange_old_changed",
        "proof_new_changed",
        "cleanup_failed",
    ] {
        assert!(
            states
                .iter()
                .any(|state| state["case_id"].as_str() == Some(required)),
            "publication state {required}"
        );
    }
}

#[test]
fn phase_e_health_confusion_and_missing_state_counts_are_exact() {
    use rust_electroanalysis_cli::{
        domain::{ArtifactLineageNode, ArtifactLineageState, known_lineage_from_artifact},
        results::{
            CausalStatus, HealthDimension, HealthEvidenceState, HealthInterpretationCategory,
            OverallHealthStatus, PhaseCHealthReasonCode, SensorHealthAssessment,
        },
    };
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&protocol_bytes).expect("UTF-8"))
            .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);

    // Exercise all four evaluable serialized Phase-C statuses with both
    // reference classes, plus the two non-evaluable statuses.  The test
    // changes serialized source evidence only and recomputes the owning
    // identity/catalog binding before the production reader consumes it.
    for (status, label, expected) in [
        ("within_baseline", "normal", "tn"),
        ("within_baseline", "alert", "fn"),
        ("watch", "normal", "tn"),
        ("watch", "alert", "fn"),
        ("degraded", "normal", "fp"),
        ("degraded", "alert", "tp"),
        ("critical", "normal", "fp"),
        ("critical", "alert", "tp"),
        ("indeterminate", "normal", "indeterminate"),
        ("indeterminate", "alert", "indeterminate"),
        (
            "data_quality_insufficient",
            "normal",
            "data_quality_insufficient",
        ),
        (
            "data_quality_insufficient",
            "alert",
            "data_quality_insufficient",
        ),
    ] {
        let (fixture_root, _, dataset_path) = staged_validation_inputs(
            "protocol/software_valid.toml",
            "dataset/software_valid.schema1.json",
        );
        let source_path = dataset_path
            .parent()
            .expect("dataset parent")
            .join("sources/health_c.schema4.json");
        let staged_source: SensorHealthAssessment =
            serde_json::from_slice(&fs::read(&source_path).expect("staged health source"))
                .expect("staged health JSON");
        let ArtifactLineageState::Known {
            identity: staged_identity,
            ..
        } = staged_source.lineage
        else {
            panic!("staged health source has known lineage")
        };
        let mut source: SensorHealthAssessment = serde_json::from_slice(
            &fs::read(fixture("health/all_status_reference_pairs.schema4.json"))
                .expect("all-status source"),
        )
        .expect("all-status source JSON");
        let ArtifactLineageState::Known {
            identity: _,
            direct_dependencies,
        } = source.lineage.clone()
        else {
            panic!("all-status source has known lineage")
        };
        let phase_c = source.phase_c.as_mut().expect("Phase-C evidence");
        let dimension = phase_c
            .dimension_assessments
            .iter_mut()
            .find(|row| row.dimension == HealthDimension::SignalIntegrity)
            .expect("signal-integrity row");
        dimension.status = serde_json::from_value(serde_json::Value::String(status.into()))
            .expect("health status");
        let (evidence_state, causal_status, reason) = match status {
            "within_baseline" => (
                HealthEvidenceState::AdequateEvidence,
                CausalStatus::Observed,
                PhaseCHealthReasonCode::ThresholdWithinLimit,
            ),
            "watch" => (
                HealthEvidenceState::AdequateEvidence,
                CausalStatus::Observed,
                PhaseCHealthReasonCode::ThresholdWatch,
            ),
            "degraded" => (
                HealthEvidenceState::AdequateEvidence,
                CausalStatus::Observed,
                PhaseCHealthReasonCode::ThresholdDegraded,
            ),
            "critical" => (
                HealthEvidenceState::AdequateEvidence,
                CausalStatus::Observed,
                PhaseCHealthReasonCode::ThresholdCritical,
            ),
            "indeterminate" => (
                HealthEvidenceState::NoEvidence,
                CausalStatus::Indeterminate,
                PhaseCHealthReasonCode::OptionalSourceAbsent,
            ),
            "data_quality_insufficient" => (
                HealthEvidenceState::PoorDataQuality,
                CausalStatus::Indeterminate,
                PhaseCHealthReasonCode::QualityGateFailed,
            ),
            _ => unreachable!(),
        };
        dimension.evidence_state = evidence_state;
        dimension.causal_status = causal_status;
        dimension.interpretation_category = HealthInterpretationCategory::ObservedBehavior;
        dimension.reason_codes = vec![reason];
        let aggregate = if matches!(
            dimension.status,
            OverallHealthStatus::Watch
                | OverallHealthStatus::Degraded
                | OverallHealthStatus::Critical
        ) {
            dimension.status
        } else {
            // The permanent source carries a data-quality row, so this is
            // the canonical aggregate for within-baseline, indeterminate,
            // and DQI signal-integrity cases.
            OverallHealthStatus::DataQualityInsufficient
        };
        source.overall_status = aggregate;
        phase_c.overall_status = aggregate;
        if matches!(
            aggregate,
            OverallHealthStatus::Watch
                | OverallHealthStatus::Degraded
                | OverallHealthStatus::Critical
        ) {
            phase_c.overall_interpretation_categories =
                vec![HealthInterpretationCategory::ObservedBehavior];
            phase_c.overall_causal_status = CausalStatus::Observed;
        } else {
            phase_c.overall_interpretation_categories.clear();
            phase_c.overall_causal_status = CausalStatus::Indeterminate;
        }
        source.lineage = known_lineage_from_artifact(
            staged_identity.artifact_kind,
            staged_identity.schema_version,
            staged_identity.producer_version.clone(),
            staged_identity.experiment_scope.clone(),
            staged_identity.sensor_scope.clone(),
            staged_identity.channel_scope.clone(),
            staged_identity.acquisition_families.clone(),
            direct_dependencies.clone(),
            &source,
        )
        .expect("recomputed health source lineage");
        let ArtifactLineageState::Known {
            identity: new_identity,
            direct_dependencies: new_dependencies,
        } = source.lineage.clone()
        else {
            panic!("recomputed known health lineage")
        };
        let mut source_wire = serde_json::to_value(&source).expect("health source wire");
        source_wire["artifact_kind"] = serde_json::Value::String("health_assessment".into());
        fs::write(
            &source_path,
            serde_json::to_vec_pretty(&source_wire).expect("mutated health source JSON"),
        )
        .expect("mutated health source write");

        let lineage_path = dataset_path
            .parent()
            .expect("dataset parent")
            .join("lineage/complete.schema1.json");
        let mut catalog = read_artifact_lineage_catalog_strict(&lineage_path)
            .expect("complete lineage catalog")
            .catalog;
        catalog.artifacts.remove(&staged_identity.artifact_id);
        catalog.artifacts.insert(
            new_identity.artifact_id.clone(),
            ArtifactLineageNode {
                identity: new_identity.clone(),
                direct_dependencies: new_dependencies,
            },
        );
        fs::write(
            &lineage_path,
            serde_json::to_vec_pretty(&catalog).expect("mutated catalog JSON"),
        )
        .expect("mutated catalog write");

        let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
            .expect("test dataset")
            .artifact;
        {
            use sha2::{Digest, Sha256};
            dataset.lineage_catalog_source.source_file_sha256 = format!(
                "{:x}",
                Sha256::digest(fs::read(&lineage_path).expect("mutated catalog bytes"))
            );
        }
        let record = dataset
            .records
            .iter_mut()
            .find(|record| record.record_id == "record_2")
            .expect("matrix record");
        record
            .health_source
            .as_mut()
            .expect("matrix health source")
            .expected_lineage = ExpectedLineageV1::Known {
            artifact_id: new_identity.artifact_id.clone(),
            semantic_sha256: new_identity.semantic_sha256.clone(),
        };
        let ReferenceEndpointV1::Health {
            label: actual_label, ..
        } = record
            .reference_endpoints
            .iter_mut()
            .find(|reference| {
                matches!(reference, ReferenceEndpointV1::Health { endpoint_id, .. } if endpoint_id == "health_endpoint")
            })
            .expect("health reference")
        else {
            panic!("health reference variant")
        };
        *actual_label = label.into();
        {
            use sha2::{Digest, Sha256};
            record
                .health_source
                .as_mut()
                .expect("matrix health source")
                .source_file_sha256 = format!(
                "{:x}",
                Sha256::digest(fs::read(&source_path).expect("mutated source bytes"))
            );
        }
        write_test_dataset(&dataset_path, &mut dataset);
        let inputs = ValidationInputs::read(&protocol, &protocol_hash, &dataset_path)
            .expect("health matrix inputs");
        let report = evaluate_mhi_validation(&protocol, &inputs).expect("health matrix report");
        let health = &report.health_results[0];
        let contains = |ids: &[String], id: &str| ids.iter().any(|value| value == id);
        assert_eq!(
            contains(&health.tp_record_ids, "record_2"),
            expected == "tp",
            "{status}/{label}"
        );
        assert_eq!(
            contains(&health.tn_record_ids, "record_2"),
            expected == "tn",
            "{status}/{label}"
        );
        assert_eq!(
            contains(&health.fp_record_ids, "record_2"),
            expected == "fp",
            "{status}/{label}"
        );
        assert_eq!(
            contains(&health.fn_record_ids, "record_2"),
            expected == "fn",
            "{status}/{label}"
        );
        assert_eq!(
            contains(&health.indeterminate_record_ids, "record_2"),
            expected == "indeterminate",
            "{status}/{label}"
        );
        assert_eq!(
            contains(&health.data_quality_insufficient_record_ids, "record_2"),
            expected == "data_quality_insufficient",
            "{status}/{label}"
        );
        assert_eq!(
            health.eligible_count,
            health.tp
                + health.tn
                + health.fp
                + health.r#fn
                + health.indeterminate
                + health.data_quality_insufficient,
            "six-way partition for {status}/{label}"
        );
        assert_eq!(
            health.evaluable,
            health.tp + health.tn + health.fp + health.r#fn,
            "evaluable denominator for {status}/{label}"
        );
        fs::remove_dir_all(fixture_root).expect("health matrix cleanup");
    }
}

#[test]
fn phase_e_mechanism_phase_b_reference_cross_product_matches_hand_oracle() {
    use rust_electroanalysis_cli::{
        domain::{ArtifactLineageNode, ArtifactLineageState, known_lineage_from_artifact},
        results::MechanismAnalysisReport,
    };
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&protocol_bytes).expect("UTF-8"))
            .expect("protocol");
    let protocol_hash = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);

    // The permanent source remains an ordinary schema-4 artifact.  The full
    // Phase-B/reference matrix is exercised by changing only its serialized
    // Phase-B level and the independently declared reference outcome; this
    // never invokes a Phase-B assessor.
    for (level, outcome, expected) in [
        ("absent", "supports", "other"),
        ("not_assessed", "supports", "other"),
        ("not_assessed", "contradicts", "critical"),
        ("not_assessed", "not_assessed", "other"),
        ("not_assessed", "unavailable", "excluded"),
        ("hypothesized", "supports", "other"),
        ("hypothesized", "contradicts", "critical"),
        ("hypothesized", "not_assessed", "other"),
        ("hypothesized", "unavailable", "excluded"),
        ("experimentally_supported", "supports", "other"),
        ("experimentally_supported", "contradicts", "critical"),
        ("experimentally_supported", "not_assessed", "other"),
        ("experimentally_supported", "unavailable", "excluded"),
        ("validated_for_domain", "supports", "support"),
        ("validated_for_domain", "contradicts", "critical"),
        ("validated_for_domain", "not_assessed", "other"),
        ("validated_for_domain", "unavailable", "excluded"),
        ("contradicted", "supports", "critical"),
        ("contradicted", "contradicts", "critical"),
        ("contradicted", "not_assessed", "critical"),
        ("contradicted", "unavailable", "excluded"),
    ] {
        let (fixture_root, _, dataset_path) = staged_validation_inputs(
            "protocol/software_valid.toml",
            "dataset/software_valid.schema1.json",
        );
        let source_path = dataset_path
            .parent()
            .expect("dataset parent")
            .join("sources/mechanism_c.schema4.json");
        let mut source: MechanismAnalysisReport =
            serde_json::from_slice(&fs::read(&source_path).expect("all-levels source"))
                .expect("all-levels source JSON");
        if level == "absent" {
            source.hypothesis_assessments.clear();
        } else {
            source.hypothesis_assessments[0].current.evidence_level =
                serde_json::from_value(serde_json::Value::String(level.into()))
                    .expect("Phase-B evidence level");
        }
        let ArtifactLineageState::Known {
            identity: old_identity,
            direct_dependencies,
        } = source.lineage.clone()
        else {
            panic!("all-levels source has known lineage")
        };
        source.lineage = known_lineage_from_artifact(
            old_identity.artifact_kind,
            old_identity.schema_version,
            old_identity.producer_version.clone(),
            old_identity.experiment_scope.clone(),
            old_identity.sensor_scope.clone(),
            old_identity.channel_scope.clone(),
            old_identity.acquisition_families.clone(),
            direct_dependencies.clone(),
            &source,
        )
        .expect("recomputed Phase-B source lineage");
        let ArtifactLineageState::Known {
            identity: new_identity,
            direct_dependencies: new_dependencies,
        } = source.lineage.clone()
        else {
            panic!("recomputed known lineage")
        };
        let mut source_wire = serde_json::to_value(&source).expect("mutated source wire");
        source_wire["artifact_kind"] = serde_json::Value::String("mechanism_analysis".into());
        fs::write(
            &source_path,
            serde_json::to_vec_pretty(&source_wire).expect("mutated source JSON"),
        )
        .expect("mutated source write");

        let lineage_path = dataset_path
            .parent()
            .expect("dataset parent")
            .join("lineage/complete.schema1.json");
        let mut catalog = read_artifact_lineage_catalog_strict(&lineage_path)
            .expect("complete lineage catalog")
            .catalog;
        catalog.artifacts.remove(&old_identity.artifact_id);
        catalog.artifacts.insert(
            new_identity.artifact_id.clone(),
            ArtifactLineageNode {
                identity: new_identity.clone(),
                direct_dependencies: new_dependencies,
            },
        );
        fs::write(
            &lineage_path,
            serde_json::to_vec_pretty(&catalog).expect("mutated catalog JSON"),
        )
        .expect("mutated catalog write");

        let mut dataset = read_artifact_strict::<MhiValidationDatasetV1>(&dataset_path)
            .expect("test dataset")
            .artifact;
        {
            use sha2::{Digest, Sha256};
            dataset.lineage_catalog_source.source_file_sha256 = format!(
                "{:x}",
                Sha256::digest(fs::read(&lineage_path).expect("mutated catalog bytes"))
            );
        }
        let record = dataset
            .records
            .iter_mut()
            .find(|record| record.record_id == "record_2")
            .expect("matrix record");
        record
            .mechanism_source
            .as_mut()
            .expect("matrix mechanism source")
            .expected_lineage = ExpectedLineageV1::Known {
            artifact_id: new_identity.artifact_id.clone(),
            semantic_sha256: new_identity.semantic_sha256.clone(),
        };
        let ReferenceEndpointV1::Mechanism {
            outcome: actual, ..
        } = record
            .reference_endpoints
            .iter_mut()
            .find(|reference| {
                matches!(reference, ReferenceEndpointV1::Mechanism { endpoint_id, .. } if endpoint_id == "mechanism_endpoint")
            })
            .expect("mechanism reference")
        else {
            panic!("mechanism reference variant")
        };
        *actual = match outcome {
            "supports" => MechanismReferenceOutcomeV1::Supports,
            "contradicts" => MechanismReferenceOutcomeV1::Contradicts,
            "not_assessed" => MechanismReferenceOutcomeV1::NotAssessed,
            "unavailable" => MechanismReferenceOutcomeV1::Unavailable,
            _ => unreachable!(),
        };
        {
            use sha2::{Digest, Sha256};
            record
                .mechanism_source
                .as_mut()
                .expect("matrix mechanism source")
                .source_file_sha256 = format!(
                "{:x}",
                Sha256::digest(fs::read(&source_path).expect("mutated source bytes"))
            );
        }
        write_test_dataset(&dataset_path, &mut dataset);
        let inputs = ValidationInputs::read(&protocol, &protocol_hash, &dataset_path)
            .expect("matrix inputs");
        let report = evaluate_mhi_validation(&protocol, &inputs).expect("matrix report");
        let mechanism = &report.mechanism_results[0];
        let contains = |ids: &[String], id: &str| ids.iter().any(|value| value == id);
        assert_eq!(
            contains(&mechanism.support_record_ids, "record_2"),
            expected == "support",
            "{level}/{outcome}"
        );
        assert_eq!(
            contains(&mechanism.critical_contradiction_record_ids, "record_2"),
            expected == "critical",
            "{level}/{outcome}"
        );
        assert_eq!(
            contains(&mechanism.not_assessed_or_other_record_ids, "record_2"),
            expected == "other",
            "{level}/{outcome}"
        );
        assert_eq!(
            mechanism
                .eligible_record_ids
                .iter()
                .any(|id| id == "record_2"),
            expected != "excluded",
            "{level}/{outcome}"
        );
        if level == "contradicted" {
            assert!(
                mechanism
                    .declared_critical_falsification_record_ids
                    .iter()
                    .any(|id| id == "record_2"),
                "declared Phase-B contradiction stays visible for {outcome}"
            );
        }
        assert_eq!(
            mechanism.eligible_count,
            mechanism.support_count
                + mechanism.critical_contradiction_count
                + mechanism.not_assessed_or_other_count,
            "n = s + c + u for {level}/{outcome}"
        );
        fs::remove_dir_all(fixture_root).expect("matrix cleanup");
    }

    let (fixture_root, _, legacy_dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let legacy_source_path = legacy_dataset
        .parent()
        .expect("dataset parent")
        .join("sources/mechanism_c.schema4.json");
    let mut legacy_source_wire: serde_json::Value =
        serde_json::from_slice(&fs::read(&legacy_source_path).expect("legacy source bytes"))
            .expect("legacy source JSON");
    assert_eq!(legacy_source_wire["schema_version"], serde_json::json!(4));
    assert_eq!(
        legacy_source_wire["artifact_kind"],
        serde_json::json!("mechanism_analysis")
    );
    legacy_source_wire["lineage"] = serde_json::json!({
        "LegacyUnknown": {
            "source_schema_version": 4,
            "reason": "MigrationInformationUnavailable"
        }
    });
    legacy_source_wire["hypothesis_assessments"][0]["current"]["evidence_level"] =
        serde_json::Value::String("contradicted".into());
    fs::write(
        &legacy_source_path,
        serde_json::to_vec_pretty(&legacy_source_wire).expect("legacy source JSON write"),
    )
    .expect("legacy source write");
    let legacy_source_hash = format!(
        "{:x}",
        Sha256::digest(fs::read(&legacy_source_path).expect("legacy source bytes"))
    );
    let mut legacy = read_artifact_strict::<MhiValidationDatasetV1>(&legacy_dataset)
        .expect("legacy dataset")
        .artifact;
    legacy
        .records
        .retain(|record| record.record_id == "record_2");
    let record = legacy.records.first_mut().expect("legacy mechanism record");
    record.health_source = None;
    record.declared_scope.experiment_scope =
        rust_electroanalysis_cli::domain::ArtifactExperimentScope::Unknown;
    record.declared_scope.sensor_scope = rust_electroanalysis_cli::domain::ScopeKey::Unspecified;
    record.declared_scope.channel_scope = rust_electroanalysis_cli::domain::ScopeKey::Unspecified;
    record.declared_scope.acquisition_families =
        rust_electroanalysis_cli::domain::ArtifactAcquisitionFamilies::Unknown;
    let mechanism_source = record
        .mechanism_source
        .as_mut()
        .expect("legacy mechanism source");
    mechanism_source.source_file_sha256 = legacy_source_hash.clone();
    mechanism_source.expected_lineage = ExpectedLineageV1::LegacyUnknown {
        schema_version: 4,
        legacy_source_fingerprint: legacy_source_hash,
        reason: rust_electroanalysis_cli::results::LegacyLineageReasonV1::MigrationInformationUnavailable,
    };
    write_test_dataset(&legacy_dataset, &mut legacy);
    let legacy_inputs = ValidationInputs::read(&protocol, &protocol_hash, &legacy_dataset)
        .expect("LegacyUnknown mechanism inputs");
    let endpoint = &protocol.mechanism_endpoints[0];
    let partition = rust_electroanalysis_cli::mhi_validation::partition::partition_endpoint(
        &legacy_inputs,
        rust_electroanalysis_cli::mhi_validation::partition::EndpointPartitionSpec {
            endpoint_id: &endpoint.endpoint_id,
            cohort_role: endpoint.cohort_role,
            domain: &endpoint.domain,
            required_strata: &endpoint.required_strata,
            reference_rule: &endpoint.reference_rule,
            source: rust_electroanalysis_cli::mhi_validation::partition::EndpointSource::Mechanism,
            physical: false,
        },
    )
    .expect("LegacyUnknown partition");
    let partition_row = partition
        .rows
        .iter()
        .find(|row| row.stratum_id == "overall" && row.record_id == "record_2")
        .expect("LegacyUnknown partition row");
    assert_eq!(
        partition_row.decision,
        rust_electroanalysis_cli::validation_config::RecordDecisionV1::Excluded
    );
    let mut exclusion_reasons = partition_row.secondary_reasons.clone();
    if let Some(primary_reason) = partition_row.primary_reason {
        exclusion_reasons.push(primary_reason);
    }
    assert!(exclusion_reasons.contains(
        &rust_electroanalysis_cli::validation_config::ExclusionReasonV1::SourceNotPhaseBOrCScoreable
    ));
    let legacy_report =
        evaluate_mhi_validation(&protocol, &legacy_inputs).expect("LegacyUnknown mechanism report");
    let mechanism = legacy_report
        .mechanism_results
        .iter()
        .find(|result| result.stratum_id == "overall")
        .expect("LegacyUnknown overall mechanism result");
    assert_eq!(mechanism.eligible_count, 0);
    assert!(
        mechanism
            .declared_critical_falsification_record_ids
            .is_empty()
    );
    assert_eq!(mechanism.declared_critical_falsification_count, 0);
    assert!(mechanism.critical_contradiction_record_ids.is_empty());
    assert!(mechanism.support_record_ids.is_empty());
    assert!(mechanism.not_assessed_or_other_record_ids.is_empty());
    assert_eq!(
        mechanism.outcome,
        rust_electroanalysis_cli::validation_config::ValidationOutcomeV1::Indeterminate
    );
    assert_eq!(
        legacy_report.release_claims[0].outcome,
        rust_electroanalysis_cli::validation_config::ReleaseClaimOutcomeV1::Indeterminate
    );
    assert_eq!(
        legacy_report.overall_status,
        rust_electroanalysis_cli::validation_config::ValidationOutcomeV1::Indeterminate
    );
    assert!(!matches!(
        mechanism.outcome,
        rust_electroanalysis_cli::validation_config::ValidationOutcomeV1::DoesNotMeetProtocol
    ));
    fs::remove_dir_all(fixture_root).expect("LegacyUnknown cleanup");

    let (fixture_root, _, duplicate_dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let mut duplicate = read_artifact_strict::<MhiValidationDatasetV1>(&duplicate_dataset)
        .expect("duplicate-reference dataset")
        .artifact;
    let reference = duplicate.records[0].reference_endpoints[0].clone();
    duplicate.records[0].reference_endpoints.push(reference);
    write_test_dataset(&duplicate_dataset, &mut duplicate);
    let duplicate_result = ValidationInputs::read(&protocol, &protocol_hash, &duplicate_dataset);
    assert!(matches!(
        duplicate_result,
        Err(MhiValidationError::Artifact(ArtifactError::Validation { ref message }))
            if message == "record reference endpoints must be canonical and unique"
    ));
    fs::remove_dir_all(fixture_root).expect("duplicate-reference cleanup");

    let (fixture_root, _, mismatched_dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let mut mismatched = read_artifact_strict::<MhiValidationDatasetV1>(&mismatched_dataset)
        .expect("mismatched-reference dataset")
        .artifact;
    if let Some(ReferenceEndpointV1::Mechanism { hypothesis_id, .. }) = mismatched.records[0]
        .reference_endpoints
        .iter_mut()
        .find(|reference| matches!(reference, ReferenceEndpointV1::Mechanism { .. }))
    {
        *hypothesis_id = "wrong-hypothesis".into();
    }
    write_test_dataset(&mismatched_dataset, &mut mismatched);
    assert!(matches!(
        ValidationInputs::read(&protocol, &protocol_hash, &mismatched_dataset),
        Err(MhiValidationError::Dataset(ref message))
            if message == "ReferenceEndpointBindingMismatch"
    ));
    fs::remove_dir_all(fixture_root).expect("mismatched-reference cleanup");

    let (fixture_root, _, phase_b_dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let phase_b_source = phase_b_dataset
        .parent()
        .expect("dataset parent")
        .join("sources/mechanism_a.schema4.json");
    let mut phase_b_wire: serde_json::Value =
        serde_json::from_slice(&fs::read(&phase_b_source).expect("Phase-B source bytes"))
            .expect("Phase-B source JSON");
    let first_assessment = phase_b_wire["hypothesis_assessments"][0].clone();
    phase_b_wire["hypothesis_assessments"] =
        serde_json::json!([first_assessment.clone(), first_assessment]);
    fs::write(
        &phase_b_source,
        serde_json::to_vec_pretty(&phase_b_wire).expect("duplicate Phase-B JSON"),
    )
    .expect("duplicate Phase-B source");
    let mut phase_b = read_artifact_strict::<MhiValidationDatasetV1>(&phase_b_dataset)
        .expect("Phase-B mutation dataset")
        .artifact;
    phase_b.records[0]
        .mechanism_source
        .as_mut()
        .expect("Phase-B mechanism source")
        .source_file_sha256 = format!(
        "{:x}",
        Sha256::digest(fs::read(&phase_b_source).expect("mutated Phase-B bytes"))
    );
    write_test_dataset(&phase_b_dataset, &mut phase_b);
    assert!(matches!(
        ValidationInputs::read(&protocol, &protocol_hash, &phase_b_dataset),
        Err(MhiValidationError::Dataset(ref message))
            if message == "Phase-B hypothesis ID mismatch or duplicate"
    ));
    fs::remove_dir_all(fixture_root).expect("Phase-B integrity cleanup");
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
        MetricValueV1::Available {
            numerator: 2,
            denominator: 2,
            ..
        }
    ));
    assert_eq!(mechanism.support_record_ids, ["record_1", "record_2"]);
    assert!(mechanism.critical_contradiction_record_ids.is_empty());
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");

    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let mut zero_eligible = read_artifact_strict::<MhiValidationDatasetV1>(&dataset)
        .expect("zero-eligible dataset")
        .artifact;
    for record in &mut zero_eligible.records {
        for reference in &mut record.reference_endpoints {
            if let ReferenceEndpointV1::Mechanism { outcome, .. } = reference {
                *outcome = MechanismReferenceOutcomeV1::Unavailable;
            }
        }
    }
    write_test_dataset(&dataset, &mut zero_eligible);
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &dataset,
    )
    .expect("zero-eligible inputs");
    let report = evaluate_mhi_validation(&protocol, &inputs).expect("zero-eligible report");
    let mechanism = &report.mechanism_results[0];
    assert_eq!(mechanism.eligible_count, 0);
    assert!(matches!(
        mechanism.support_fraction,
        MetricValueV1::Unavailable { ref reason, .. } if reason == "denominator_zero"
    ));
    assert!(
        mechanism
            .declared_critical_falsification_record_ids
            .is_empty()
    );
    fs::remove_dir_all(fixture_root).expect("zero-eligible cleanup");
}

#[test]
fn phase_e_overall_and_closed_strata_apply_exact_record_and_family_minima() {
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let mut protocol =
        MhiValidationProtocolV1::from_toml(std::str::from_utf8(&protocol_bytes).expect("UTF-8"))
            .expect("protocol");
    protocol.mechanism_endpoints[0].minimum_eligible_records = 3;
    protocol.mechanism_endpoints[0].minimum_independent_families = 2;
    protocol.health_endpoints[0].minimum_eligible_records = 3;
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

    // Membership is exercised from each record's actual domain, rather than
    // merely accepting a declared stratum axis in protocol metadata.
    let predicates = [
        StratumPredicateV1::AnalyteEquals {
            id: "analyte".into(),
        },
        StratumPredicateV1::MatrixEquals {
            id: "matrix".into(),
        },
        StratumPredicateV1::SensorDesignEquals {
            id: "design".into(),
        },
        StratumPredicateV1::SensorEquals {
            id: "sensor".into(),
        },
        StratumPredicateV1::CampaignEquals {
            id: "campaign".into(),
        },
        StratumPredicateV1::TemperatureBand {
            lower_kelvin_inclusive: 298.0,
            upper_kelvin_exclusive: 299.0,
        },
    ];
    for (index, predicate) in predicates.into_iter().enumerate() {
        let mut protocol = protocol_fixture("protocol/software_valid.toml");
        let stratum = RequiredStratumV1 {
            stratum_id: format!("axis_{index}"),
            predicates: vec![predicate],
            minimum_eligible_records: 2,
            minimum_independent_families: 2,
        };
        protocol.mechanism_endpoints[0].required_strata = vec![stratum.clone()];
        protocol.validate().expect("closed stratum protocol");
        let (fixture_root, _, dataset) = staged_validation_inputs(
            "protocol/software_valid.toml",
            "dataset/software_valid.schema1.json",
        );
        let inputs = ValidationInputs::read(
            &protocol,
            &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
            &dataset,
        )
        .expect("stratum inputs");
        let report = evaluate_mhi_validation(&protocol, &inputs).expect("stratum report");
        let mechanism = report
            .mechanism_results
            .iter()
            .find(|result| result.stratum_id == stratum.stratum_id)
            .expect("mechanism stratum result");
        assert_eq!(mechanism.eligible_record_ids, ["record_1", "record_2"]);
        assert_eq!(mechanism.independent_family_count, 2);
        fs::remove_dir_all(fixture_root).expect("stratum cleanup");
    }

    // A passing overall view does not rescue a required empty stratum.
    let mut protocol = protocol_fixture("protocol/software_valid.toml");
    let empty = RequiredStratumV1 {
        stratum_id: "empty_analyte".into(),
        predicates: vec![StratumPredicateV1::AnalyteEquals {
            id: "unobserved".into(),
        }],
        minimum_eligible_records: 1,
        minimum_independent_families: 1,
    };
    protocol.mechanism_endpoints[0].required_strata = vec![empty.clone()];
    let (fixture_root, _, dataset) = staged_validation_inputs(
        "protocol/software_valid.toml",
        "dataset/software_valid.schema1.json",
    );
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &dataset,
    )
    .expect("empty stratum inputs");
    let report = evaluate_mhi_validation(&protocol, &inputs).expect("empty stratum report");
    assert!(report.mechanism_results.iter().any(|result| {
        result.stratum_id == "overall"
            && result.rule_evaluations.iter().all(|rule| {
                rule.result
                    == rust_electroanalysis_cli::validation_config::RuleEvaluationResultV1::True
            })
            && result.outcome
                == rust_electroanalysis_cli::validation_config::ValidationOutcomeV1::Indeterminate
    }));
    assert!(report.mechanism_results.iter().any(|result| {
        result.stratum_id == empty.stratum_id
            && result.eligible_count == 0
            && result.outcome
                == rust_electroanalysis_cli::validation_config::ValidationOutcomeV1::Indeterminate
    }));
    assert_eq!(
        report.release_claims[0].outcome,
        rust_electroanalysis_cli::validation_config::ReleaseClaimOutcomeV1::Indeterminate
    );
    fs::remove_dir_all(fixture_root).expect("empty stratum cleanup");
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
        .expect("approved R6 plan");
    assert_eq!(
        format!("{:x}", Sha256::digest(plan)),
        "0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33"
    );
    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("Cargo manifest");
    assert!(cargo.contains("ed25519-dalek = { version = \"=2.2.0\", default-features = false }"));
    assert!(!cargo.contains("ed25519-dalek = { version = \"=2.2.0\", features"));
    for package in [
        "name = \"curve25519-dalek\"\nversion = \"4.1.3\"",
        "name = \"curve25519-dalek-derive\"\nversion = \"0.1.1\"",
        "name = \"ed25519\"\nversion = \"2.2.3\"",
        "name = \"ed25519-dalek\"\nversion = \"2.2.0\"",
        "name = \"fiat-crypto\"\nversion = \"0.2.9\"",
        "name = \"signature\"\nversion = \"2.2.0\"",
    ] {
        assert!(
            fs::read_to_string(root.join("Cargo.lock"))
                .expect("Cargo lock")
                .contains(package),
            "required locked dependency {package}"
        );
    }

    let fixture_root = fixture("");
    let mut actual_fixture_paths = Vec::new();
    fixture_regular_files(&fixture_root, &fixture_root, &mut actual_fixture_paths);
    actual_fixture_paths.sort();
    let actual_fixture_paths = actual_fixture_paths.into_iter().collect::<BTreeSet<_>>();
    let expected_paths = expected_fixture_paths();
    assert_eq!(expected_paths.len(), 268, "R2 fixture inventory is 268/268");
    assert_eq!(
        actual_fixture_paths, expected_paths,
        "literal inventory must exactly cover every regular Phase-E fixture"
    );
    let inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(fixture("expected/phase_e_fixture_inventory.schema1.json"))
            .expect("closed fixture inventory"),
    )
    .expect("inventory JSON");
    let mappings = inventory
        .as_array()
        .expect("inventory array")
        .iter()
        .flat_map(|row| row["mappings"].as_array().expect("mappings").iter());
    let mut requirements = BTreeSet::new();
    let mut acceptance_criteria = BTreeSet::new();
    let mut tests = BTreeSet::new();
    for mapping in mappings {
        requirements.insert(
            mapping["requirement_id"]
                .as_str()
                .expect("requirement")
                .to_owned(),
        );
        acceptance_criteria.insert(
            mapping["acceptance_criterion_id"]
                .as_str()
                .expect("acceptance criterion")
                .to_owned(),
        );
        tests.insert(mapping["test_id"].as_str().expect("test ID").to_owned());
    }
    assert_eq!(
        requirements,
        (1..=18)
            .map(|id| format!("E-R{id:02}"))
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        acceptance_criteria,
        (1..=18)
            .map(|id| format!("E-AC{id:02}"))
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        tests,
        (1..=30)
            .map(|id| format!("E-T{id:02}"))
            .collect::<BTreeSet<_>>()
    );

    let author_evidence =
        fs::read_to_string(fixture("expected/author_validation_evidence_ledger.md"))
            .expect("author-side evidence");
    for required in [
        "ism-mechanism-health-v1-e-plan-approved",
        "ism-mechanism-health-v1-e-plan-approved-r2",
        "ism-mechanism-health-v1-e-plan-approved-r3",
        "ism-mechanism-health-v1-e-plan-approved-r4",
        "ism-mechanism-health-v1-e-plan-approved-r5",
        "ism-mechanism-health-v1-e-plan-approved-r6",
        "macOS the sole supported",
        "non-UTF-8 early-return `DIR*` resource leak",
        "permanent E-T25 coverage remains weaker",
        "e6e5195c7f56904afb06dfe937433f3498465fef1df191b8fb6856ee1ac792b6",
        "131dc77dc656952469c77a816a36c847d4f38a018f922577441884396009ed4a",
        "0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33",
        "18 requirements",
        "E-R18",
        "E-AC18",
        "E-T30",
        "git diff --check",
        "cargo test --locked --all",
        "cargo doc --locked --workspace --no-deps",
        "PENDING_POST_FREEZE",
        "268/268",
        "29/29",
        "E-T22 is substantive executable coverage",
        "E-T23 is substantive executable coverage",
        "P1-SEC-001 remediation",
        "P1-SEC-002 remediation",
        "approval file-hash mismatch",
        "approval target-domain binding mismatch",
        "physical disallowed-reference-method rejection",
        "physical unblinded-reference rejection",
        "physical uncertainty rejection",
        "physical incomplete-reference rejection",
        "actual one-family physical case",
        "missing-stratum physical case",
        "PHYSICAL_PATH_ASSERTED =",
        "E-T29 = substantive PASS",
        "SCI-P1-001 remediation",
        "valid adjacent bands",
        "union containment",
        "E-R02/E-AC02 restored to passing",
        "External scientific review: PENDING_POST_FREEZE",
    ] {
        assert!(
            author_evidence.contains(required),
            "author evidence {required}"
        );
    }
    assert!(!author_evidence.contains("candidate commit SHA"));
    assert!(!author_evidence.contains("REVIEW_SHA"));
    assert!(!author_evidence.contains("IMPLEMENTATION_APPROVAL = GO"));

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
        "phase_e_e_t22_complete_staging_and_byte_validation_matrix",
        "phase_e_e_t23_lock_holder_process",
        "phase_e_e_t23_true_lock_contention_concurrent_create_and_recovery_matrix",
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
    for required in [
        "PublicationStagingCleanupFailed",
        "PublicationConcurrentManagedOutputChanged",
        "PublicationCommittedForeignSwapDetected",
        "PublicationCommittedVisibleOutputChanged",
        "SyncDirectoryAt",
        "NoReplaceUnsupported",
        "ExchangeUnsupported",
        "DeleteAt",
        "std::process::Command",
    ] {
        assert!(
            registry_source.contains(required),
            "missing publication evidence {required}"
        );
    }
    assert!(!source_text.contains("SigningKey"));
    assert!(source_text.contains("pub(crate) mod output;"));
    assert!(source_text.contains("pub(crate) fn authorize_publication"));
    assert!(source_text.contains("pub(crate) fn publish_authorized_bundle"));
    assert!(!source_text.contains("pub fn publish_bundle"));
    assert!(!source_text.contains("test_authority_validation"));
    let output_source = fs::read_to_string(root.join("src/mhi_validation/output.rs"))
        .expect("publication production source");
    let production_output = output_source
        .split("#[cfg(test)]\nmod tests")
        .next()
        .expect("publication test boundary");
    let production_runner =
        fs::read_to_string(root.join("src/runners/mhi_validation.rs")).expect("runner source");
    assert_eq!(
        production_output
            .matches("publish_authorized_bundle(")
            .count()
            + production_runner
                .matches("publish_authorized_bundle(")
                .count(),
        2,
        "only the internal runner and the capability definition may name the raw publisher"
    );
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
        trust.provisioning_state(),
        PhysicalApprovalProvisioningStateV1::Unprovisioned
    );
    assert!(!trust.is_provisioned());
}

#[test]
fn phase_e_public_evaluator_fails_closed_without_verified_approval() {
    let (root, dataset_path) = staged_physical_inputs();
    let protocol_path = fixture("protocol/physical_valid.toml");
    let protocol_bytes = fs::read(&protocol_path).expect("physical protocol bytes");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("physical protocol UTF-8"),
    )
    .expect("physical protocol");
    let inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &dataset_path,
    )
    .expect("public reader accepts the physical graph before approval");
    assert!(matches!(
        evaluate_mhi_validation(&protocol, &inputs),
        Err(MhiValidationError::Approval(ref message))
            if message == "physical evaluation requires a verified owner approval"
    ));
    fs::remove_dir_all(root).expect("public API regression cleanup");
}

#[test]
fn phase_e_artifact_contracts_accept_exact_schema1_and_reject_invalid_variants() {
    fn approval_error(result: Result<(), MhiValidationError>, expected: &str) {
        match result {
            Err(MhiValidationError::Approval(message)) => assert_eq!(message, expected),
            Err(other) => panic!("expected approval error {expected:?}, received {other:?}"),
            Ok(()) => panic!("expected approval error {expected:?}, received success"),
        }
    }

    let known: PhysicalApprovalTrustStoreV1 = serde_json::from_slice(
        &fs::read(fixture(
            "trust/test_only_known_answer_trust_store.schema1.json",
        ))
        .expect("known-answer trust store"),
    )
    .expect("schema-1 test trust store");
    known
        .validate()
        .expect("known public keys are canonical and nonweak");

    let invalid_identity: PhysicalApprovalTrustStoreV1 = serde_json::from_slice(
        &fs::read(fixture(
            "trust/test_only_invalid_identity_weak_key.schema1.json",
        ))
        .expect("weak-key trust store"),
    )
    .expect("schema-1 weak-key fixture");
    approval_error(invalid_identity.validate(), "PhysicalApprovalWeakPublicKey");

    let mut unprovisioned_with_root = known.clone();
    unprovisioned_with_root.provisioning_state = PhysicalApprovalProvisioningStateV1::Unprovisioned;
    approval_error(
        unprovisioned_with_root.validate(),
        "UNPROVISIONED trust store must have no trust roots",
    );
    let mut provisioned_empty = known.clone();
    provisioned_empty.trust_roots.clear();
    approval_error(
        provisioned_empty.validate(),
        "PROVISIONED trust store must have trust roots",
    );

    let mut same_authority = known.clone();
    same_authority.trust_roots[0].registry_authority_id = same_authority.trust_roots[0]
        .project_owner_authority_id
        .clone();
    approval_error(
        same_authority.validate(),
        "trust authority IDs must be globally unique",
    );
    let mut same_key = known.clone();
    same_key.trust_roots[0].registry_ed25519_public_key_hex =
        same_key.trust_roots[0].owner_ed25519_public_key_hex.clone();
    approval_error(
        same_key.validate(),
        "trust public keys must be globally unique",
    );
    let mut nondecompressible = known.clone();
    nondecompressible.trust_roots[0].owner_ed25519_public_key_hex =
        "0200000000000000000000000000000000000000000000000000000000000000".into();
    approval_error(
        nondecompressible.validate(),
        "PhysicalApprovalPublicKeyInvalid",
    );
    let mut noncanonical = known.clone();
    noncanonical.trust_roots[0].owner_ed25519_public_key_hex =
        "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f".into();
    approval_error(
        noncanonical.validate(),
        "PhysicalApprovalNoncanonicalPublicKey",
    );
    let mut wrong_length = known.clone();
    wrong_length.trust_roots[0].owner_ed25519_public_key_hex = "00".repeat(31);
    approval_error(wrong_length.validate(), "PhysicalApprovalPublicKeyInvalid");
    let mut long_key = known.clone();
    long_key.trust_roots[0].owner_ed25519_public_key_hex = "00".repeat(33);
    approval_error(long_key.validate(), "PhysicalApprovalPublicKeyInvalid");
    let mut schema_zero = known.clone();
    schema_zero.schema_version = 0;
    approval_error(
        schema_zero.validate(),
        "invalid embedded trust-store identity",
    );
    let mut schema_two = known.clone();
    schema_two.schema_version = 2;
    approval_error(
        schema_two.validate(),
        "invalid embedded trust-store identity",
    );

    for invalid in [
        br#"{\"schema_version\":1,\"trust_store_id\":\"mhi_physical_approval_trust_store_v1\",\"trust_roots\":[]}"#
            as &[u8],
        br#"{\"schema_version\":1,\"trust_store_id\":\"mhi_physical_approval_trust_store_v1\",\"provisioning_state\":\"UNPROVISIONED\",\"roots\":[]}"#,
        br#"{\"schema_version\":1,\"trust_store_id\":\"mhi_physical_approval_trust_store_v1\",\"provisioning_state\":\"UNPROVISIONED\",\"trust_roots\":[],\"unknown\":true}"#,
    ] {
        assert!(serde_json::from_slice::<PhysicalApprovalTrustStoreV1>(invalid).is_err());
    }

    let mut duplicate_root = known.clone();
    duplicate_root
        .trust_roots
        .push(PhysicalApprovalTrustRootV1 {
            trust_root_id: "test_only_second_root".into(),
            project_owner_authority_id: "test_vector_owner_2".into(),
            owner_ed25519_public_key_hex: known.trust_roots[0].owner_ed25519_public_key_hex.clone(),
            registry_authority_id: "test_vector_registry_2".into(),
            registry_ed25519_public_key_hex: known.trust_roots[0]
                .registry_ed25519_public_key_hex
                .clone(),
        });
    approval_error(
        duplicate_root.validate(),
        "trust public keys must be globally unique",
    );

    let mut duplicate_authority_across_roots = known.clone();
    duplicate_authority_across_roots
        .trust_roots
        .push(PhysicalApprovalTrustRootV1 {
            trust_root_id: "test_only_second_root".into(),
            project_owner_authority_id: known.trust_roots[0].project_owner_authority_id.clone(),
            owner_ed25519_public_key_hex:
                "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c".into(),
            registry_authority_id: "test_vector_registry_2".into(),
            registry_ed25519_public_key_hex:
                "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a".into(),
        });
    approval_error(
        duplicate_authority_across_roots.validate(),
        "trust authority IDs must be globally unique",
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
    let guards = fs::read_to_string(fixture("source_guards/forbidden_dependencies.txt"))
        .expect("literal source-guard fixture");
    for line in guards.lines() {
        let (relative_path, forbidden) = line.split_once('\t').expect("guard mapping");
        let source = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path))
            .expect("guarded source");
        assert!(
            !source.contains(forbidden),
            "forbidden Phase-E dependency {forbidden} in {relative_path}"
        );
    }
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
    assert_eq!(report.record_accounting.len(), 4);
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
    report.health_results[0].tp = 0;
    assert!(report.validate_structure().is_err());
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_report_authority_rejects_independent_count_id_exclusion_family_rule_claim_and_status_mutations()
 {
    use rust_electroanalysis_cli::validation_config::{
        ExclusionReasonV1, RecordDecisionV1, ReleaseClaimOutcomeV1, RuleEvaluationResultV1,
        ValidationOutcomeV1,
    };

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
    let assert_rejected = |candidate: rust_electroanalysis_cli::results::MhiValidationReportV1| {
        assert!(
            candidate.validate_against(&protocol, &inputs).is_err(),
            "authority-assisted replay rejects the independent mutation"
        );
    };

    let mut count = report.clone();
    count.health_results[0].tp += 1;
    assert_rejected(count);

    let mut id_set = report.clone();
    id_set.health_results[0].tp_record_ids.clear();
    assert_rejected(id_set);

    let mut exclusion = report.clone();
    exclusion.record_accounting[0].decision = RecordDecisionV1::Excluded;
    exclusion.record_accounting[0].primary_reason =
        Some(ExclusionReasonV1::MissingReferenceEndpoint);
    assert_rejected(exclusion);

    let mut family = report.clone();
    family.mechanism_results[0].independent_family_count = 1;
    assert_rejected(family);

    let mut rule = report.clone();
    rule.mechanism_results[0].rule_evaluations[0].result = RuleEvaluationResultV1::False;
    assert_rejected(rule);

    let mut claim = report.clone();
    claim.release_claims[0].outcome = ReleaseClaimOutcomeV1::DoesNotMeetProtocol;
    assert_rejected(claim);

    let mut overall = report;
    overall.overall_status = ValidationOutcomeV1::Indeterminate;
    assert_rejected(overall);
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
        .validate_against(&protocol, &inputs)
        .expect("exact replay");
    report.overall_status =
        rust_electroanalysis_cli::validation_config::ValidationOutcomeV1::Indeterminate;
    assert!(report.validate_against(&protocol, &inputs).is_err());
    fs::remove_dir_all(fixture_root).expect("cleanup staged inputs");
}

#[test]
fn phase_e_report_identity_bytes_and_escaping_are_independent_of_operations() {
    let protocol_bytes = fs::read(fixture("protocol/software_valid.toml")).expect("protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("protocol UTF-8"),
    )
    .expect("protocol");
    let (root, _, dataset) = staged_validation_inputs(
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

    let preimage = serde_jcs::to_vec(&serde_json::json!({
        "identity_domain": "mhi_validation_report_id_v1",
        "protocol_sha256": report.protocol.source_file_sha256,
        "dataset_source": report.dataset.source,
        "consumed_sources": report.provenance.consumed_sources,
    }))
    .expect("canonical report-ID preimage");
    let expected_preimage =
        fs::read(fixture("expected/report_identity_preimage.jcs")).expect("R2 JCS preimage");
    assert_eq!(
        preimage,
        expected_preimage
            .strip_suffix(b"\n")
            .unwrap_or(&expected_preimage),
        "report ID has one exact JCS authority preimage"
    );
    assert_eq!(
        report.report_id,
        format!("sha256:{:x}", Sha256::digest(&preimage)),
        "report ID binds protocol, dataset, and sorted consumed sources"
    );
    let source_order = report
        .provenance
        .consumed_sources
        .iter()
        .map(|source| serde_json::to_value(source).expect("serialized source"))
        .collect::<Vec<_>>();
    assert_eq!(
        source_order
            .iter()
            .map(|source| source["type"].as_str().expect("source type"))
            .collect::<Vec<_>>(),
        [
            "known_artifact",
            "known_artifact",
            "known_artifact",
            "known_artifact",
            "lineage_catalog",
            "reference_authority",
        ],
        "consumed sources use the frozen source-kind sort order"
    );
    assert_eq!(
        source_order[..4]
            .iter()
            .map(|source| source["artifact_id"].as_str().expect("artifact ID"))
            .collect::<Vec<_>>(),
        [
            "sha256:2a90f8661e834a85da3b49e7aa18e8cd2c6630730573f10939022382a119b413",
            "sha256:56f277ab229e29d98f35b5160f851af904aab72ded1103d0621889540642c234",
            "sha256:2494b9fcc5799b014e18d32cacbb2c32cf5c2c84d55e4abbb6f2179a6b90fdf0",
            "sha256:b115a28466c07508499d80d42cbf563ee6245b1e103ce4b5ece5122f36220fc9",
        ],
        "known source order is kind then artifact ID"
    );

    let output = temp("identity_operational_values");
    run_mhi_validation(MhiValidationRunOptions {
        protocol: root.join("protocol.toml"),
        dataset,
        output_dir: output.clone(),
        overwrite: false,
    })
    .expect("certified output");
    assert_exact_golden_bundle(&output);
    let all_managed_bytes = [
        "mhi_validation_report.schema1.json",
        "validation_execution_manifest.schema1.json",
        "validation_summary.md",
        "tables/cohort_coverage.csv",
        "tables/leakage_assessment.csv",
        "tables/mechanism_validation.csv",
        "tables/health_validation.csv",
        "tables/exclusion_ledger.csv",
        "tables/compatibility_matrix.csv",
    ]
    .into_iter()
    .flat_map(|relative| fs::read(output.join(relative)).expect("managed bytes"))
    .collect::<Vec<_>>();
    for forbidden in [
        output.to_str().expect("UTF-8 temporary output"),
        "phase-e-stage",
        "phase-e-backup",
        "localhost",
    ] {
        assert!(
            !all_managed_bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden.as_bytes()),
            "operational value {forbidden:?} does not enter scientific bytes"
        );
    }

    let escaping: serde_json::Value = serde_json::from_slice(
        &fs::read(fixture("expected/escaping_vectors.schema1.json")).expect("escaping vectors"),
    )
    .expect("escaping vector JSON");
    for vector in escaping["vectors"].as_array().expect("escaping vectors") {
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .terminator(csv::Terminator::Any(b'\n'))
            .from_writer(Vec::new());
        let value = vector["value"].as_str().unwrap_or_else(|| {
            if vector["label"] == "negative_zero" {
                "0.0"
            } else {
                "NA"
            }
        });
        writer.write_record([value]).expect("CSV escape record");
        let actual = String::from_utf8(writer.into_inner().expect("CSV bytes")).expect("CSV UTF-8");
        assert_eq!(
            actual,
            format!("{}\n", vector["csv"].as_str().expect("expected CSV cell")),
            "exact CSV projection {}",
            vector["label"].as_str().expect("escaping label")
        );
    }
    let mechanism_csv =
        fs::read_to_string(output.join("tables/mechanism_validation.csv")).expect("mechanism CSV");
    let health_csv =
        fs::read_to_string(output.join("tables/health_validation.csv")).expect("health CSV");
    assert!(
        mechanism_csv
            .lines()
            .next()
            .expect("mechanism header")
            .contains("support_count")
    );
    assert!(!mechanism_csv.contains("tp,tn,fp,fn"));
    assert!(
        health_csv
            .lines()
            .next()
            .expect("health header")
            .contains("tp,tn,fp,fn")
    );
    assert!(!health_csv.contains("support_count"));

    fs::remove_dir_all(output).expect("output cleanup");
    fs::remove_dir_all(root).expect("input cleanup");
}
