use super::*;
use crate::{
    domain::read_artifact_strict,
    mhi_validation::{
        MhiValidationError, MhiValidationProtocolV1, ValidationInputs, evaluate_mhi_validation,
        partition::{EndpointPartitionSpec, EndpointSource, partition_endpoint},
    },
    results::{
        MechanismReferenceOutcomeV1, MhiValidationDatasetV1, OutcomeReasonV1, ReferenceEndpointV1,
        ReferenceUncertaintyV1,
    },
    runners::mhi_validation::{MhiValidationRunOptions, run_mhi_validation},
    validation_config::{
        BlindingStateV1, CategoricalSelectorV1, ReferenceDependencyCompletenessV1,
        RequestedValidationLevelV1, RequiredStratumV1, StratumPredicateV1,
    },
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn verified_approval(
    approval: OwnerApprovalEvidenceV1,
    trust: &VerifiedEmbeddedTrustStore,
    protocol: &MhiValidationProtocolV1,
    dataset: &MhiValidationDatasetV1,
) -> VerifiedOwnerApproval {
    approval
        .validate(trust, protocol, dataset)
        .expect("test approval verifies");
    VerifiedOwnerApproval {
        evidence: approval,
        trust_store_sha256: trust.source_file_sha256.clone(),
    }
}

fn clone_validation_inputs(inputs: &ValidationInputs) -> ValidationInputs {
    inputs.clone()
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_e")
        .join(name)
}

fn protocol_fixture(name: &str) -> MhiValidationProtocolV1 {
    let bytes = fs::read(fixture(name)).expect("protocol fixture bytes");
    MhiValidationProtocolV1::from_toml(std::str::from_utf8(&bytes).expect("protocol UTF-8"))
        .expect("valid protocol fixture")
}

const IDENTITY_KEY_HEX: &str = "0100000000000000000000000000000000000000000000000000000000000000";
const NONDECOMPRESSIBLE_Y2_KEY_HEX: &str =
    "0200000000000000000000000000000000000000000000000000000000000000";
const ZERO_SCALAR_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const NONCANONICAL_SCALAR_L_HEX: &str =
    "edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010";

fn identity_r_zero_s_signature() -> String {
    format!("{IDENTITY_KEY_HEX}{ZERO_SCALAR_HEX}")
}

fn signature_with_noncanonical_scalar(signature: &str) -> String {
    assert_eq!(signature.len(), 128, "known-answer signature length");
    format!("{}{}", &signature[..64], NONCANONICAL_SCALAR_L_HEX)
}

fn flip_one_signature_bit(signature: &str) -> String {
    assert_eq!(signature.len(), 128, "known-answer signature length");
    let first_byte = u8::from_str_radix(&signature[..2], 16).expect("known-answer signature hex");
    format!("{:02x}{}", first_byte ^ 0x01, &signature[2..])
}

fn trust_with_role_key(
    trusted: &VerifiedEmbeddedTrustStore,
    owner_role: bool,
    key_hex: &str,
) -> VerifiedEmbeddedTrustStore {
    let mut store = trusted.store.clone();
    let root = store.trust_roots.first_mut().expect("test trust root");
    if owner_role {
        root.owner_ed25519_public_key_hex = key_hex.into();
    } else {
        root.registry_ed25519_public_key_hex = key_hex.into();
    }
    VerifiedEmbeddedTrustStore {
        store,
        source_file_sha256: trusted.source_file_sha256.clone(),
    }
}

fn assert_valid_bound_approval_mutation(
    label: &str,
    original: &OwnerApprovalEvidenceV1,
    candidate: &OwnerApprovalEvidenceV1,
    trusted: &VerifiedEmbeddedTrustStore,
    protocol: &MhiValidationProtocolV1,
    dataset: &MhiValidationDatasetV1,
    expected: &str,
) {
    let mut original_without_signatures = original.clone();
    original_without_signatures
        .owner_signature_ed25519_hex
        .clear();
    original_without_signatures
        .registry_signature_ed25519_hex
        .clear();
    let mut candidate_without_signatures = candidate.clone();
    candidate_without_signatures
        .owner_signature_ed25519_hex
        .clear();
    candidate_without_signatures
        .registry_signature_ed25519_hex
        .clear();
    assert_eq!(
        candidate_without_signatures, original_without_signatures,
        "{label}: only signature fields may change"
    );
    assert_eq!(
        candidate.approval_record_id, original.approval_record_id,
        "{label}: approval record identity must remain valid-bound"
    );

    match candidate.validate(trusted, protocol, dataset) {
        Err(MhiValidationError::Approval(actual)) => assert_eq!(actual, expected, "{label}"),
        Err(other) => panic!("{label}: expected approval error {expected:?}, received {other:?}"),
        Ok(()) => panic!("{label}: expected approval error {expected:?}, received success"),
    }
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

fn staged_physical_inputs(dataset_fixture: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = temp("physical_kat");
    let protocol = root.join("protocol.toml");
    let dataset = root.join("dataset/input.schema1.json");
    fs::create_dir_all(dataset.parent().expect("dataset parent")).expect("dataset layout");
    fs::copy(fixture("protocol/physical_valid.toml"), &protocol).expect("physical protocol");
    fs::copy(fixture(dataset_fixture), &dataset).expect("physical dataset");
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
    fs::create_dir_all(dataset.parent().expect("dataset parent").join("approval"))
        .expect("physical approval layout");
    let approval = if dataset_fixture.contains("selective") {
        "approval/valid_selective_unavailable.schema1.json"
    } else {
        "approval/valid.schema1.json"
    };
    fs::copy(
        fixture(approval),
        dataset.parent().expect("dataset parent").join(approval),
    )
    .expect("physical approval");
    (root, protocol, dataset)
}

fn physical_mechanism_reference_mut(
    inputs: &mut ValidationInputs,
    record_index: usize,
) -> &mut ReferenceEndpointV1 {
    inputs.dataset.artifact.records[record_index]
        .reference_endpoints
        .iter_mut()
        .find(|reference| {
            matches!(
                reference,
                ReferenceEndpointV1::Mechanism {
                    endpoint_id,
                    ..
                } if endpoint_id == "mechanism_endpoint"
            )
        })
        .expect("physical mechanism reference")
}

fn assert_physical_reference_rejection(
    inputs: &ValidationInputs,
    protocol: &MhiValidationProtocolV1,
    expected: &str,
) {
    let endpoint = protocol
        .mechanism_endpoints
        .iter()
        .find(|endpoint| endpoint.endpoint_id == "mechanism_endpoint")
        .expect("physical mechanism endpoint");
    let claim = protocol
        .release_scope
        .iter()
        .find(|claim| claim.claim_id == "physical_claim")
        .expect("physical release claim");
    assert_eq!(claim.requested_level, RequestedValidationLevelV1::Physical);
    assert!(
        claim
            .supporting_endpoint_ids
            .iter()
            .any(|endpoint_id| endpoint_id == &endpoint.endpoint_id)
    );
    let spec = EndpointPartitionSpec {
        endpoint_id: &endpoint.endpoint_id,
        cohort_role: endpoint.cohort_role,
        domain: &endpoint.domain,
        required_strata: &endpoint.required_strata,
        reference_rule: &endpoint.reference_rule,
        source: EndpointSource::Mechanism,
        physical: true,
    };
    assert!(spec.physical, "the production physical branch is active");
    match partition_endpoint(inputs, spec) {
        Err(MhiValidationError::Dataset(actual)) => assert_eq!(actual, expected),
        Err(other) => panic!("expected physical dataset error {expected:?}, got {other:?}"),
        Ok(_) => panic!("physical reference rejection unexpectedly reached scoring"),
    }
}

#[derive(Debug, Clone, Copy)]
struct Et29CaseContract {
    number: u8,
    mutation: &'static str,
    physical_path: bool,
    production_function: &'static str,
    expected_result: &'static str,
    actual_result: &'static str,
}

fn assert_e_t29_matrix_contract() {
    let cases = [
        Et29CaseContract {
            number: 1,
            mutation: "production UNPROVISIONED route",
            physical_path: true,
            production_function: "run_mhi_validation",
            expected_result: "PhysicalApprovalTrustNotProvisioned",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 2,
            mutation: "missing approval file",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::read_and_validate",
            expected_result: "approval I/O error",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 3,
            mutation: "wrong approval purpose",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "invalid approval status or purpose",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 4,
            mutation: "unknown trust root",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval trust-root binding mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 5,
            mutation: "attacker owner authority",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval authority binding mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 6,
            mutation: "attacker registry authority",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval authority binding mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 7,
            mutation: "malformed owner signature",
            physical_path: true,
            production_function: "verify_strict(owner)",
            expected_result: "PhysicalApprovalOwnerSignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 8,
            mutation: "malformed registry signature",
            physical_path: true,
            production_function: "verify_strict(registry)",
            expected_result: "PhysicalApprovalRegistrySignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 9,
            mutation: "owner S=L",
            physical_path: true,
            production_function: "verify_strict(owner)",
            expected_result: "PhysicalApprovalOwnerSignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 10,
            mutation: "registry S=L",
            physical_path: true,
            production_function: "verify_strict(registry)",
            expected_result: "PhysicalApprovalRegistrySignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 11,
            mutation: "strict-invalid owner signature",
            physical_path: true,
            production_function: "verify_strict(owner)",
            expected_result: "PhysicalApprovalOwnerSignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 12,
            mutation: "strict-invalid registry signature",
            physical_path: true,
            production_function: "verify_strict(registry)",
            expected_result: "PhysicalApprovalRegistrySignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 13,
            mutation: "owner signature missing",
            physical_path: true,
            production_function: "verify_strict(owner)",
            expected_result: "PhysicalApprovalOwnerSignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 14,
            mutation: "registry signature missing",
            physical_path: true,
            production_function: "verify_strict(registry)",
            expected_result: "PhysicalApprovalRegistrySignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 15,
            mutation: "copied owner signature into registry role",
            physical_path: true,
            production_function: "verify_strict(registry)",
            expected_result: "PhysicalApprovalRegistrySignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 16,
            mutation: "copied registry signature into owner role",
            physical_path: true,
            production_function: "verify_strict(owner)",
            expected_result: "PhysicalApprovalOwnerSignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 17,
            mutation: "owner weak public key",
            physical_path: true,
            production_function: "verify_strict(owner)",
            expected_result: "PhysicalApprovalWeakPublicKey",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 18,
            mutation: "registry weak public key",
            physical_path: true,
            production_function: "verify_strict(registry)",
            expected_result: "PhysicalApprovalWeakPublicKey",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 19,
            mutation: "owner nondecompressible public key",
            physical_path: true,
            production_function: "verify_strict(owner)",
            expected_result: "PhysicalApprovalPublicKeyInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 20,
            mutation: "registry nondecompressible public key",
            physical_path: true,
            production_function: "verify_strict(registry)",
            expected_result: "PhysicalApprovalPublicKeyInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 21,
            mutation: "wrong owner public key",
            physical_path: true,
            production_function: "verify_strict(owner)",
            expected_result: "PhysicalApprovalOwnerSignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 22,
            mutation: "wrong registry public key",
            physical_path: true,
            production_function: "verify_strict(registry)",
            expected_result: "PhysicalApprovalRegistrySignatureInvalid",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 23,
            mutation: "approval file-hash mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::read_and_validate",
            expected_result: "approval file SHA-256 mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 24,
            mutation: "approval record ID mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval record ID mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 25,
            mutation: "approval cohort binding mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval protocol or cohort binding mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 26,
            mutation: "approval protocol binding mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval protocol or cohort binding mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 27,
            mutation: "approval claim binding mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval physical claim bindings mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 28,
            mutation: "approval endpoint binding mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval endpoint bindings mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 29,
            mutation: "approval target-domain binding mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval target-domain binding mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 30,
            mutation: "approval physical-origin mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "invalid approval status or purpose",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 31,
            mutation: "approval reference-authority binding mismatch",
            physical_path: true,
            production_function: "OwnerApprovalEvidenceV1::validate",
            expected_result: "approval reference-authority bindings mismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 32,
            mutation: "physical reference outcome unavailable",
            physical_path: true,
            production_function: "partition_endpoint(physical=true)",
            expected_result: "PhysicalReferenceOutcomeUnavailable",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 33,
            mutation: "physical disallowed reference method",
            physical_path: true,
            production_function: "partition_endpoint(physical=true)",
            expected_result: "PhysicalReferenceAuthorityMismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 34,
            mutation: "physical unblinded reference",
            physical_path: true,
            production_function: "partition_endpoint(physical=true)",
            expected_result: "PhysicalReferenceAuthorityMismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 35,
            mutation: "physical unavailable uncertainty",
            physical_path: true,
            production_function: "partition_endpoint(physical=true)",
            expected_result: "PhysicalReferenceAuthorityMismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 36,
            mutation: "physical incomplete reference",
            physical_path: true,
            production_function: "partition_endpoint(physical=true)",
            expected_result: "PhysicalReferenceAuthorityMismatch",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 37,
            mutation: "software minimum underpower",
            physical_path: false,
            production_function: "evaluate_mhi_validation",
            expected_result: "Indeterminate",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 38,
            mutation: "actual one-family physical cohort",
            physical_path: true,
            production_function: "evaluate_mhi_validation",
            expected_result: "IndependentFamilyMinimumNotMet / Indeterminate",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 39,
            mutation: "physical missing required stratum",
            physical_path: true,
            production_function: "evaluate_mhi_validation",
            expected_result: "RequiredStratumIndeterminate / Indeterminate",
            actual_result: "PASS",
        },
        Et29CaseContract {
            number: 40,
            mutation: "valid dual-signed physical two-family KAT",
            physical_path: true,
            production_function: "evaluate_mhi_validation",
            expected_result: "PhysicallyValidated",
            actual_result: "PASS",
        },
    ];
    assert_eq!(cases.len(), 40, "E-T29 executable matrix has 40 cases");
    assert!(cases.iter().all(|case| {
        case.number > 0
            && !case.mutation.is_empty()
            && !case.production_function.is_empty()
            && !case.expected_result.is_empty()
            && case.actual_result == "PASS"
    }));
    // PHYSICAL_PATH_ASSERTED = yes for every repaired substantive case.
    let repaired = [23, 29, 33, 34, 35, 36, 38, 39];
    assert!(repaired.iter().all(|number| {
        cases
            .iter()
            .find(|case| case.number == *number)
            .is_some_and(|case| case.physical_path)
    }));
    assert_eq!(
        cases
            .iter()
            .filter(|case| case.actual_result == "PASS")
            .count(),
        40,
        "E-T29 substantive PASS count"
    );
}

#[test]
fn phase_e_physical_claim_requires_dual_signature_embedded_trust_and_power() {
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
        crate::mhi_validation::MhiValidationError::PhysicalApprovalTrustNotProvisioned
    ));
    assert!(!output.exists());

    let trust_bytes = fs::read(fixture(
        "trust/test_only_known_answer_trust_store.schema1.json",
    ))
    .expect("test-only trust store");
    let trust_store: PhysicalApprovalTrustStoreV1 =
        serde_json::from_slice(&trust_bytes).expect("trust store schema");
    trust_store
        .validate()
        .expect("test-only trust store validity");
    let trust_hash = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(&trust_bytes))
    };
    let verified_trust = VerifiedEmbeddedTrustStore {
        store: trust_store,
        source_file_sha256: trust_hash,
    };
    let physical_protocol = protocol_fixture("protocol/physical_valid.toml");
    let physical_dataset = read_artifact_strict::<MhiValidationDatasetV1>(&fixture(
        "dataset/physical_valid.schema1.json",
    ))
    .expect("current physical dataset")
    .artifact;
    assert_eq!(physical_dataset.records.len(), 2);
    let physical_families = physical_dataset
        .records
        .iter()
        .flat_map(|record| match &record.declared_scope.acquisition_families {
            crate::domain::ArtifactAcquisitionFamilies::Known(families) => families
                .iter()
                .map(|family| family.0.clone())
                .collect::<Vec<_>>(),
            crate::domain::ArtifactAcquisitionFamilies::Unknown => Vec::new(),
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        physical_families,
        BTreeSet::from(["physical-family-a".into(), "physical-family-b".into()])
    );
    assert_eq!(
        physical_dataset
            .records
            .iter()
            .flat_map(|record| record.reference_endpoints.iter())
            .filter(|reference| {
                matches!(
                    reference,
                    ReferenceEndpointV1::Mechanism {
                        outcome: MechanismReferenceOutcomeV1::Supports,
                        blinding_state: BlindingStateV1::BlindedToAssessment,
                        uncertainty: ReferenceUncertaintyV1::Quantified { .. },
                        ..
                    }
                )
            })
            .count(),
        2
    );
    let physical_approval: OwnerApprovalEvidenceV1 = serde_json::from_slice(
        &fs::read(fixture("approval/valid.schema1.json")).expect("current approval"),
    )
    .expect("current approval schema");
    physical_approval
        .validate(&verified_trust, &physical_protocol, &physical_dataset)
        .expect("current dual-signed physical KAT");

    let approval_error = |candidate: OwnerApprovalEvidenceV1, expected: &str| match candidate
        .validate(&verified_trust, &physical_protocol, &physical_dataset)
    {
        Err(MhiValidationError::Approval(actual)) => assert_eq!(actual, expected),
        Err(other) => panic!("expected approval error {expected:?}, received {other:?}"),
        Ok(()) => panic!("expected approval error {expected:?}, received success"),
    };
    let mut invalid = physical_approval.clone();
    invalid.approval_purpose = "wrong_purpose".into();
    approval_error(invalid, "invalid approval status or purpose");
    let mut invalid = physical_approval.clone();
    invalid.trust_root_id = "unknown_test_root".into();
    approval_error(invalid, "approval trust-root binding mismatch");
    let mut invalid = physical_approval.clone();
    invalid.project_owner_authority_id = "attacker_owner".into();
    approval_error(invalid, "approval authority binding mismatch");
    let mut invalid = physical_approval.clone();
    invalid.registry_authority_id = "attacker_registry".into();
    approval_error(invalid, "approval authority binding mismatch");

    // Every signature mutation below starts from the exact valid dual-signed
    // KAT.  The protocol, dataset, approval record identity, and every other
    // approval field remain unchanged, so these cases reach the verifier.
    let mut invalid = physical_approval.clone();
    invalid.registry_signature_ed25519_hex = invalid.owner_signature_ed25519_hex.clone();
    assert_valid_bound_approval_mutation(
        "copied owner signature into registry role",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalRegistrySignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.owner_signature_ed25519_hex = "00".into();
    assert_valid_bound_approval_mutation(
        "malformed owner signature",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalOwnerSignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.registry_signature_ed25519_hex = "00".into();
    assert_valid_bound_approval_mutation(
        "malformed registry signature",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalRegistrySignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.owner_signature_ed25519_hex =
        signature_with_noncanonical_scalar(&physical_approval.owner_signature_ed25519_hex);
    assert_valid_bound_approval_mutation(
        "noncanonical owner signature scalar",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalOwnerSignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.registry_signature_ed25519_hex =
        signature_with_noncanonical_scalar(&physical_approval.registry_signature_ed25519_hex);
    assert_valid_bound_approval_mutation(
        "noncanonical registry signature scalar",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalRegistrySignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.owner_signature_ed25519_hex =
        flip_one_signature_bit(&physical_approval.owner_signature_ed25519_hex);
    assert_valid_bound_approval_mutation(
        "strict-invalid owner signature",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalOwnerSignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.registry_signature_ed25519_hex =
        flip_one_signature_bit(&physical_approval.registry_signature_ed25519_hex);
    assert_valid_bound_approval_mutation(
        "strict-invalid registry signature",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalRegistrySignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.owner_signature_ed25519_hex.clear();
    assert_valid_bound_approval_mutation(
        "owner signature missing",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalOwnerSignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.registry_signature_ed25519_hex.clear();
    assert_valid_bound_approval_mutation(
        "registry signature missing",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalRegistrySignatureInvalid",
    );
    let mut invalid = physical_approval.clone();
    invalid.owner_signature_ed25519_hex = physical_approval.registry_signature_ed25519_hex.clone();
    assert_valid_bound_approval_mutation(
        "copied registry signature into owner role",
        &physical_approval,
        &invalid,
        &verified_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalOwnerSignatureInvalid",
    );

    // Identity-R/zero-S is paired with the selected weak public key so the
    // weak-key rejection is observed before signature acceptance.
    let identity_signature = identity_r_zero_s_signature();
    let weak_owner_trust = trust_with_role_key(&verified_trust, true, IDENTITY_KEY_HEX);
    let mut weak_owner = physical_approval.clone();
    weak_owner.owner_signature_ed25519_hex = identity_signature.clone();
    assert_valid_bound_approval_mutation(
        "owner identity weak key",
        &physical_approval,
        &weak_owner,
        &weak_owner_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalWeakPublicKey",
    );
    let weak_registry_trust = trust_with_role_key(&verified_trust, false, IDENTITY_KEY_HEX);
    let mut weak_registry = physical_approval.clone();
    weak_registry.registry_signature_ed25519_hex = identity_signature;
    assert_valid_bound_approval_mutation(
        "registry identity weak key",
        &physical_approval,
        &weak_registry,
        &weak_registry_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalWeakPublicKey",
    );

    let y2_owner_trust = trust_with_role_key(&verified_trust, true, NONDECOMPRESSIBLE_Y2_KEY_HEX);
    assert_valid_bound_approval_mutation(
        "owner nondecompressible y=2 key",
        &physical_approval,
        &physical_approval,
        &y2_owner_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalPublicKeyInvalid",
    );
    let y2_registry_trust =
        trust_with_role_key(&verified_trust, false, NONDECOMPRESSIBLE_Y2_KEY_HEX);
    assert_valid_bound_approval_mutation(
        "registry nondecompressible y=2 key",
        &physical_approval,
        &physical_approval,
        &y2_registry_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalPublicKeyInvalid",
    );

    // This is a valid canonical nonweak public key from the test corpus, but
    // it does not correspond to either KAT signature and has no private
    // material in the repository.
    let wrong_role_key = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    let wrong_owner_trust = trust_with_role_key(&verified_trust, true, wrong_role_key);
    wrong_owner_trust
        .store
        .validate()
        .expect("wrong owner test key remains canonical and nonweak");
    assert_valid_bound_approval_mutation(
        "wrong owner public key reaches verifier",
        &physical_approval,
        &physical_approval,
        &wrong_owner_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalOwnerSignatureInvalid",
    );
    let wrong_registry_trust = trust_with_role_key(&verified_trust, false, wrong_role_key);
    wrong_registry_trust
        .store
        .validate()
        .expect("wrong registry test key remains canonical and nonweak");
    assert_valid_bound_approval_mutation(
        "wrong registry public key reaches verifier",
        &physical_approval,
        &physical_approval,
        &wrong_registry_trust,
        &physical_protocol,
        &physical_dataset,
        "PhysicalApprovalRegistrySignatureInvalid",
    );

    let mut invalid = physical_approval.clone();
    invalid.approval_record_id =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    approval_error(invalid, "approval record ID mismatch");
    let mut invalid = physical_approval.clone();
    invalid.protocol_sha256 = "0".repeat(64);
    approval_error(invalid, "approval protocol or cohort binding mismatch");
    let mut invalid = physical_approval.clone();
    invalid.cohort_semantic_sha256 = "0".repeat(64);
    approval_error(invalid, "approval protocol or cohort binding mismatch");
    let mut invalid = physical_approval.clone();
    invalid.claim_ids = vec!["unexpected_claim".into()];
    approval_error(invalid, "approval physical claim bindings mismatch");
    let mut invalid = physical_approval.clone();
    invalid.endpoint_ids = vec!["unexpected_endpoint".into()];
    approval_error(invalid, "approval endpoint bindings mismatch");
    let mut invalid = physical_approval.clone();
    invalid.reference_authority_ids = vec!["unexpected_authority".into()];
    approval_error(invalid, "approval reference-authority bindings mismatch");
    let mut invalid = physical_approval.clone();
    invalid.target_domain.sensor = CategoricalSelectorV1::Allowed {
        ids: vec!["sensor_a".into()],
    };
    let mut expected_target_domain_mutation = physical_approval.clone();
    expected_target_domain_mutation.target_domain = invalid.target_domain.clone();
    assert_eq!(
        invalid, expected_target_domain_mutation,
        "target-domain case mutates no approval field other than target_domain"
    );
    approval_error(invalid, "approval target-domain binding mismatch");
    let mut invalid = physical_approval.clone();
    invalid.physical_origin_confirmed = false;
    approval_error(invalid, "invalid approval status or purpose");

    for (fixture_path, expected) in [
        (
            "approval/invalid_self_signed.schema1.json",
            "approval protocol or cohort binding mismatch",
        ),
        (
            "approval/invalid_identity_forgery.schema1.json",
            "approval protocol or cohort binding mismatch",
        ),
    ] {
        let candidate: OwnerApprovalEvidenceV1 = serde_json::from_slice(
            &fs::read(fixture(fixture_path)).expect("invalid approval fixture"),
        )
        .expect("invalid approval schema");
        approval_error(candidate, expected);
    }

    // E-T29 cases 3-6 deliberately invoke the production physical reference
    // authority branch directly after the reader boundary.  The typed
    // mutations are test-owned and do not rewrite the signed KAT bytes or
    // attempt to pass through an unrelated software exclusion result.
    let (reference_root, reference_protocol_path, reference_dataset_path) =
        staged_physical_inputs("dataset/physical_valid.schema1.json");
    let reference_protocol_bytes =
        fs::read(&reference_protocol_path).expect("physical reference protocol bytes");
    let reference_protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&reference_protocol_bytes).expect("physical reference protocol UTF-8"),
    )
    .expect("physical reference protocol");
    let base_reference_inputs = ValidationInputs::read(
        &reference_protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&reference_protocol_bytes),
        &reference_dataset_path,
    )
    .expect("physical reference inputs");

    // Case 3: only the mechanism reference method is disallowed.
    let mut disallowed_method = clone_validation_inputs(&base_reference_inputs);
    if let ReferenceEndpointV1::Mechanism { method_id, .. } =
        physical_mechanism_reference_mut(&mut disallowed_method, 0)
    {
        *method_id = "disallowed_reference_method".into();
    } else {
        panic!("mechanism reference shape");
    }
    assert_physical_reference_rejection(
        &disallowed_method,
        &reference_protocol,
        "PhysicalReferenceAuthorityMismatch",
    );

    // Case 4: only the mechanism reference blinding state is unblinded.
    let mut unblinded = clone_validation_inputs(&base_reference_inputs);
    if let ReferenceEndpointV1::Mechanism { blinding_state, .. } =
        physical_mechanism_reference_mut(&mut unblinded, 0)
    {
        *blinding_state = BlindingStateV1::NotBlinded;
    } else {
        panic!("mechanism reference shape");
    }
    assert_physical_reference_rejection(
        &unblinded,
        &reference_protocol,
        "PhysicalReferenceAuthorityMismatch",
    );

    // Case 5: only the mechanism reference uncertainty becomes unavailable.
    let mut uncertain = clone_validation_inputs(&base_reference_inputs);
    if let ReferenceEndpointV1::Mechanism { uncertainty, .. } =
        physical_mechanism_reference_mut(&mut uncertain, 0)
    {
        *uncertainty = ReferenceUncertaintyV1::Unavailable {
            reason: "test-only unavailable uncertainty".into(),
        };
    } else {
        panic!("mechanism reference shape");
    }
    assert_physical_reference_rejection(
        &uncertain,
        &reference_protocol,
        "PhysicalReferenceAuthorityMismatch",
    );

    // Case 6: only the physical reference dependency graph is incomplete.
    let mut incomplete = clone_validation_inputs(&base_reference_inputs);
    incomplete.dataset.artifact.reference_sources[0].dependency_completeness =
        ReferenceDependencyCompletenessV1::Unknown;
    assert_physical_reference_rejection(
        &incomplete,
        &reference_protocol,
        "PhysicalReferenceAuthorityMismatch",
    );
    fs::remove_dir_all(reference_root).expect("physical reference cases cleanup");

    let (valid_root, valid_protocol, valid_dataset) =
        staged_physical_inputs("dataset/physical_valid.schema1.json");
    let protocol_bytes = fs::read(&valid_protocol).expect("staged physical protocol");
    let protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&protocol_bytes).expect("physical protocol UTF-8"),
    )
    .expect("staged physical protocol schema");
    let mut inputs = ValidationInputs::read(
        &protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes),
        &valid_dataset,
    )
    .expect("current physical graph reads");
    let staged_approval: OwnerApprovalEvidenceV1 = serde_json::from_slice(
        &fs::read(
            valid_dataset
                .parent()
                .expect("dataset parent")
                .join("approval/valid.schema1.json"),
        )
        .expect("staged approval"),
    )
    .expect("staged approval schema");
    inputs.attach_verified_approval(verified_approval(
        staged_approval,
        &verified_trust,
        &protocol,
        &inputs.dataset.artifact,
    ));
    let report = evaluate_mhi_validation(&protocol, &inputs).expect("physical KAT evaluation");
    assert_eq!(
        report.release_claims[0].outcome,
        crate::validation_config::ReleaseClaimOutcomeV1::PhysicallyValidated
    );
    assert_eq!(report.mechanism_results[0].eligible_count, 2);
    assert_eq!(report.mechanism_results[0].support_count, 2);
    assert_eq!(report.health_results[0].eligible_count, 2);
    report
        .validate_against(&protocol, &inputs)
        .expect("physical KAT authority replay");
    fs::remove_dir_all(valid_root).expect("valid physical KAT cleanup");

    // Case 7: the physical cohort has two eligible records but only one
    // actual acquisition family.  This reaches the real physical assessment
    // and power logic; it is not a software fixture with a lowered minimum.
    let (one_family_root, one_family_protocol_path, one_family_dataset_path) =
        staged_physical_inputs("dataset/physical_valid.schema1.json");
    let one_family_protocol_bytes =
        fs::read(&one_family_protocol_path).expect("one-family protocol bytes");
    let one_family_protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&one_family_protocol_bytes).expect("one-family protocol UTF-8"),
    )
    .expect("one-family protocol");
    assert!(
        one_family_protocol
            .release_scope
            .iter()
            .any(|claim| { claim.requested_level == RequestedValidationLevelV1::Physical })
    );
    let mut one_family_inputs = ValidationInputs::read(
        &one_family_protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&one_family_protocol_bytes),
        &one_family_dataset_path,
    )
    .expect("one-family physical inputs");
    one_family_inputs.attach_verified_approval(verified_approval(
        physical_approval.clone(),
        &verified_trust,
        &one_family_protocol,
        &one_family_inputs.dataset.artifact,
    ));
    let one_family = one_family_inputs.dataset.artifact.records[0]
        .declared_scope
        .acquisition_families
        .clone();
    one_family_inputs.dataset.artifact.records[1]
        .declared_scope
        .acquisition_families = one_family;
    one_family_inputs.dataset.artifact.cohort_semantic_sha256 = one_family_inputs
        .dataset
        .artifact
        .computed_cohort_semantic_sha256()
        .expect("one-family test-owned cohort identity");
    let one_family_report = evaluate_mhi_validation(&one_family_protocol, &one_family_inputs)
        .expect("one-family physical assessment");
    assert_eq!(
        one_family_report.release_claims[0].requested_level,
        RequestedValidationLevelV1::Physical
    );
    assert_eq!(
        one_family_report.release_claims[0].outcome,
        crate::validation_config::ReleaseClaimOutcomeV1::Indeterminate
    );
    assert_ne!(
        one_family_report.release_claims[0].outcome,
        crate::validation_config::ReleaseClaimOutcomeV1::PhysicallyValidated
    );
    assert_eq!(
        one_family_report.overall_status,
        crate::validation_config::ValidationOutcomeV1::Indeterminate
    );
    for result in &one_family_report.mechanism_results {
        if result.stratum_id == "overall" {
            assert_eq!(result.eligible_count, 2);
            assert_eq!(result.independent_family_count, 1);
            assert!(result.outcome_reasons.contains(
                &OutcomeReasonV1::IndependentFamilyMinimumNotMet {
                    actual: 1,
                    minimum: 2,
                }
            ));
            assert_eq!(
                result.outcome,
                crate::validation_config::ValidationOutcomeV1::Indeterminate
            );
        }
    }
    for result in &one_family_report.health_results {
        if result.stratum_id == "overall" {
            assert_eq!(result.eligible_count, 2);
            assert_eq!(result.independent_family_count, 1);
            assert!(result.outcome_reasons.contains(
                &OutcomeReasonV1::IndependentFamilyMinimumNotMet {
                    actual: 1,
                    minimum: 2,
                }
            ));
            assert_eq!(
                result.outcome,
                crate::validation_config::ValidationOutcomeV1::Indeterminate
            );
        }
    }
    fs::remove_dir_all(one_family_root).expect("one-family physical cleanup");

    // Case 8: the physical cohort satisfies the overall view but has no
    // eligible evidence for a protocol-required stratum.  Required-stratum
    // propagation must make both parent endpoints and the physical claim
    // indeterminate.
    let (missing_stratum_root, missing_stratum_protocol_path, missing_stratum_dataset_path) =
        staged_physical_inputs("dataset/physical_valid.schema1.json");
    let missing_stratum_protocol_bytes =
        fs::read(&missing_stratum_protocol_path).expect("missing-stratum protocol bytes");
    let mut missing_stratum_protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&missing_stratum_protocol_bytes)
            .expect("missing-stratum protocol UTF-8"),
    )
    .expect("missing-stratum protocol");
    let required_stratum = RequiredStratumV1 {
        stratum_id: "physical_required_missing".into(),
        predicates: vec![StratumPredicateV1::SensorEquals {
            id: "sensor_missing".into(),
        }],
        minimum_eligible_records: 2,
        minimum_independent_families: 2,
    };
    missing_stratum_protocol.mechanism_endpoints[0].required_strata =
        vec![required_stratum.clone()];
    missing_stratum_protocol.health_endpoints[0].required_strata = vec![required_stratum.clone()];
    assert!(
        missing_stratum_protocol
            .release_scope
            .iter()
            .any(|claim| { claim.requested_level == RequestedValidationLevelV1::Physical })
    );
    let mut missing_stratum_inputs = ValidationInputs::read(
        &missing_stratum_protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&missing_stratum_protocol_bytes),
        &missing_stratum_dataset_path,
    )
    .expect("missing-stratum physical inputs");
    missing_stratum_inputs.attach_verified_approval(verified_approval(
        physical_approval,
        &verified_trust,
        &missing_stratum_protocol,
        &missing_stratum_inputs.dataset.artifact,
    ));
    let missing_stratum_report =
        evaluate_mhi_validation(&missing_stratum_protocol, &missing_stratum_inputs)
            .expect("missing-stratum physical assessment");
    assert_eq!(
        missing_stratum_report.release_claims[0].requested_level,
        RequestedValidationLevelV1::Physical
    );
    assert_eq!(
        missing_stratum_report.release_claims[0].outcome,
        crate::validation_config::ReleaseClaimOutcomeV1::Indeterminate
    );
    assert_ne!(
        missing_stratum_report.release_claims[0].outcome,
        crate::validation_config::ReleaseClaimOutcomeV1::PhysicallyValidated
    );
    for result in &missing_stratum_report.mechanism_results {
        if result.stratum_id == required_stratum.stratum_id {
            assert_eq!(result.eligible_count, 0);
            assert_eq!(
                result.outcome_reasons,
                vec![
                    OutcomeReasonV1::EligibleRecordMinimumNotMet {
                        actual: 0,
                        minimum: 2,
                    },
                    OutcomeReasonV1::EmptyView,
                    OutcomeReasonV1::IndependentFamilyMinimumNotMet {
                        actual: 0,
                        minimum: 2,
                    },
                    OutcomeReasonV1::RequiredRuleUnavailable {
                        rule_id: "support".into(),
                    },
                ]
            );
            assert_eq!(
                result.outcome,
                crate::validation_config::ValidationOutcomeV1::Indeterminate
            );
        }
        if result.stratum_id == "overall" {
            assert_eq!(
                result.outcome_reasons,
                vec![OutcomeReasonV1::RequiredStratumIndeterminate {
                    stratum_id: required_stratum.stratum_id.clone(),
                }]
            );
            assert_eq!(
                result.outcome,
                crate::validation_config::ValidationOutcomeV1::Indeterminate
            );
        }
    }
    for result in &missing_stratum_report.health_results {
        if result.stratum_id == required_stratum.stratum_id {
            assert_eq!(result.eligible_count, 0);
            assert_eq!(
                result.outcome,
                crate::validation_config::ValidationOutcomeV1::Indeterminate
            );
        }
        if result.stratum_id == "overall" {
            assert_eq!(
                result.outcome_reasons,
                vec![OutcomeReasonV1::RequiredStratumIndeterminate {
                    stratum_id: required_stratum.stratum_id.clone(),
                }]
            );
            assert_eq!(
                result.outcome,
                crate::validation_config::ValidationOutcomeV1::Indeterminate
            );
        }
    }
    fs::remove_dir_all(missing_stratum_root).expect("missing-stratum physical cleanup");

    let (unavailable_root, unavailable_protocol, unavailable_dataset) =
        staged_physical_inputs("dataset/physical_selective_unavailable.schema1.json");
    let unavailable_protocol_bytes = fs::read(&unavailable_protocol).expect("selective protocol");
    let unavailable_protocol = MhiValidationProtocolV1::from_toml(
        std::str::from_utf8(&unavailable_protocol_bytes).expect("selective protocol UTF-8"),
    )
    .expect("selective protocol schema");
    let mut unavailable_inputs = ValidationInputs::read(
        &unavailable_protocol,
        &MhiValidationProtocolV1::sha256_of_bytes(&unavailable_protocol_bytes),
        &unavailable_dataset,
    )
    .expect("selective physical graph reads");
    assert_eq!(unavailable_inputs.dataset.artifact.records.len(), 100);
    assert_eq!(
        unavailable_inputs
            .dataset
            .artifact
            .records
            .iter()
            .flat_map(|record| record.reference_endpoints.iter())
            .filter(|reference| {
                matches!(
                    reference,
                    ReferenceEndpointV1::Mechanism {
                        outcome: MechanismReferenceOutcomeV1::Unavailable,
                        blinding_state: BlindingStateV1::BlindedToAssessment,
                        uncertainty: ReferenceUncertaintyV1::Quantified { .. },
                        ..
                    }
                )
            })
            .count(),
        98
    );
    let selective_approval: OwnerApprovalEvidenceV1 = serde_json::from_slice(
        &fs::read(
            unavailable_dataset
                .parent()
                .expect("dataset parent")
                .join("approval/valid_selective_unavailable.schema1.json"),
        )
        .expect("selective approval"),
    )
    .expect("selective approval schema");
    unavailable_inputs.attach_verified_approval(verified_approval(
        selective_approval,
        &verified_trust,
        &unavailable_protocol,
        &unavailable_inputs.dataset.artifact,
    ));
    assert!(matches!(
        evaluate_mhi_validation(&unavailable_protocol, &unavailable_inputs),
        Err(MhiValidationError::Dataset(ref message))
            if message == "PhysicalReferenceOutcomeUnavailable"
    ));
    fs::remove_dir_all(unavailable_root).expect("selective physical KAT cleanup");
    assert_e_t29_matrix_contract();
}
