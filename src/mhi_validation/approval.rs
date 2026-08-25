//! Immutable embedded trust authority for physical Phase-E claims.

use super::{MhiValidationError, protocol::MhiValidationProtocolV1};
use crate::results::MhiValidationDatasetV1;
use crate::validation_config::ReferenceAuthorityRuleV1;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

const EMBEDDED_TRUST_STORE: &[u8] =
    include_bytes!("../../config/mhi_physical_approval_trust_store.schema1.json");

#[cfg(test)]
#[path = "approval_kat.rs"]
mod approval_kat;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalApprovalTrustRootV1 {
    pub trust_root_id: String,
    pub project_owner_authority_id: String,
    pub owner_ed25519_public_key_hex: String,
    pub registry_authority_id: String,
    pub registry_ed25519_public_key_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalApprovalTrustStoreV1 {
    pub schema_version: u32,
    pub trust_store_id: String,
    pub provisioning_state: PhysicalApprovalProvisioningStateV1,
    pub trust_roots: Vec<PhysicalApprovalTrustRootV1>,
}

/// The production authority is deliberately shipped unprovisioned in Phase E.
/// A later, separately reviewed provisioning change may add immutable roots;
/// neither a protocol nor any runtime input can alter this state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalApprovalProvisioningStateV1 {
    #[serde(rename = "UNPROVISIONED")]
    Unprovisioned,
    #[serde(rename = "PROVISIONED")]
    Provisioned,
}

#[derive(Debug, Clone)]
/// Verified production authority is opaque.  It can only be obtained by
/// validating the bytes embedded in this crate.
///
/// ```compile_fail
/// use rust_electroanalysis_cli::mhi_validation::approval::{
///     PhysicalApprovalTrustStoreV1, VerifiedEmbeddedTrustStore,
/// };
///
/// let _forged = VerifiedEmbeddedTrustStore {
///     store: PhysicalApprovalTrustStoreV1 {
///         schema_version: 1,
///         trust_store_id: String::new(),
///         provisioning_state: todo!(),
///         trust_roots: Vec::new(),
///     },
///     source_file_sha256: String::new(),
/// };
/// ```
pub struct VerifiedEmbeddedTrustStore {
    store: PhysicalApprovalTrustStoreV1,
    source_file_sha256: String,
}

impl PhysicalApprovalTrustStoreV1 {
    pub fn from_embedded_bytes() -> Result<VerifiedEmbeddedTrustStore, MhiValidationError> {
        let store: Self = serde_json::from_slice(EMBEDDED_TRUST_STORE).map_err(|error| {
            MhiValidationError::Approval(format!("embedded trust store JSON: {error}"))
        })?;
        store.validate()?;
        let mut hash = Sha256::new();
        hash.update(EMBEDDED_TRUST_STORE);
        Ok(VerifiedEmbeddedTrustStore {
            store,
            source_file_sha256: format!("{:x}", hash.finalize()),
        })
    }

    pub fn validate(&self) -> Result<(), MhiValidationError> {
        if self.schema_version != 1 || self.trust_store_id != "mhi_physical_approval_trust_store_v1"
        {
            return Err(approval("invalid embedded trust-store identity"));
        }
        match self.provisioning_state {
            PhysicalApprovalProvisioningStateV1::Unprovisioned if !self.trust_roots.is_empty() => {
                return Err(approval(
                    "UNPROVISIONED trust store must have no trust roots",
                ));
            }
            PhysicalApprovalProvisioningStateV1::Provisioned if self.trust_roots.is_empty() => {
                return Err(approval("PROVISIONED trust store must have trust roots"));
            }
            _ => {}
        }
        let mut root_ids = BTreeSet::new();
        let mut authorities = BTreeSet::new();
        let mut keys = BTreeSet::new();
        let mut previous = None;
        for root in &self.trust_roots {
            stable_id("trust_root_id", &root.trust_root_id)?;
            if previous
                .as_ref()
                .is_some_and(|id: &String| id >= &root.trust_root_id)
                || !root_ids.insert(root.trust_root_id.clone())
            {
                return Err(approval("trust roots must be sorted and unique"));
            }
            previous = Some(root.trust_root_id.clone());
            stable_id(
                "project_owner_authority_id",
                &root.project_owner_authority_id,
            )?;
            stable_id("registry_authority_id", &root.registry_authority_id)?;
            if !authorities.insert(root.project_owner_authority_id.clone())
                || !authorities.insert(root.registry_authority_id.clone())
            {
                return Err(approval("trust authority IDs must be globally unique"));
            }
            for key_hex in [
                &root.owner_ed25519_public_key_hex,
                &root.registry_ed25519_public_key_hex,
            ] {
                let bytes = hex_32(key_hex)?;
                let key = VerifyingKey::from_bytes(&bytes)
                    .map_err(|_| approval("PhysicalApprovalPublicKeyInvalid"))?;
                let canonical = key.to_edwards().compress().to_bytes();
                if canonical != bytes {
                    return Err(approval("PhysicalApprovalNoncanonicalPublicKey"));
                }
                if key.is_weak() {
                    return Err(approval("PhysicalApprovalWeakPublicKey"));
                }
                if !keys.insert(canonical) {
                    return Err(approval("trust public keys must be globally unique"));
                }
            }
        }
        Ok(())
    }

    pub const fn is_provisioned(&self) -> bool {
        matches!(
            self.provisioning_state,
            PhysicalApprovalProvisioningStateV1::Provisioned
        )
    }

    pub fn root(&self, id: &str) -> Result<&PhysicalApprovalTrustRootV1, MhiValidationError> {
        self.trust_roots
            .iter()
            .find(|root| root.trust_root_id == id)
            .ok_or_else(|| approval("declared trust root does not exist"))
    }
}

impl VerifiedEmbeddedTrustStore {
    pub fn provisioning_state(&self) -> PhysicalApprovalProvisioningStateV1 {
        self.store.provisioning_state
    }

    pub fn source_file_sha256(&self) -> &str {
        &self.source_file_sha256
    }

    pub const fn is_provisioned(&self) -> bool {
        self.store.is_provisioned()
    }

    pub(crate) fn root(
        &self,
        id: &str,
    ) -> Result<&PhysicalApprovalTrustRootV1, MhiValidationError> {
        self.store.root(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableDocumentV1 {
    pub immutable_reference_uri: String,
    pub document_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Approval evidence is wire data, not verified authority.  The verifier is
/// crate-private and is never exposed as a downstream test escape hatch.
///
/// ```compile_fail
/// use rust_electroanalysis_cli::mhi_validation::approval::OwnerApprovalEvidenceV1;
///
/// let _ = OwnerApprovalEvidenceV1::validate_for_test_boundary;
/// ```
pub struct OwnerApprovalEvidenceV1 {
    pub schema_version: u32,
    pub approval_record_id: String,
    pub approval_status: String,
    pub approval_purpose: String,
    pub trust_store_id: String,
    pub trust_root_id: String,
    pub project_owner_authority_id: String,
    pub registry_authority_id: String,
    pub owner_authority_document: ImmutableDocumentV1,
    pub registry_record: ImmutableDocumentV1,
    pub protocol_sha256: String,
    pub cohort_semantic_sha256: String,
    pub claim_ids: Vec<String>,
    pub endpoint_ids: Vec<String>,
    pub reference_authority_ids: Vec<String>,
    pub target_domain: crate::validation_config::DomainSelectorV1,
    pub physical_origin_confirmed: bool,
    pub limitations: Vec<String>,
    pub owner_signature_ed25519_hex: String,
    pub registry_signature_ed25519_hex: String,
}

impl OwnerApprovalEvidenceV1 {
    /// Production reader boundary: the approval bytes are verified against
    /// the embedded trust capability before an opaque approval capability is
    /// returned.
    pub(crate) fn read_and_validate(
        path: &Path,
        expected_file_sha256: &str,
        embedded: &VerifiedEmbeddedTrustStore,
        protocol: &MhiValidationProtocolV1,
        dataset: &MhiValidationDatasetV1,
    ) -> Result<VerifiedOwnerApproval, MhiValidationError> {
        let bytes = fs::read(path).map_err(|source| MhiValidationError::Io {
            path: path.into(),
            source,
        })?;
        let mut hash = Sha256::new();
        hash.update(&bytes);
        if format!("{:x}", hash.finalize()) != expected_file_sha256 {
            return Err(approval("approval file SHA-256 mismatch"));
        }
        let approval: Self = serde_json::from_slice(&bytes)
            .map_err(|error| approval(format!("approval JSON: {error}")))?;
        approval.validate(embedded, protocol, dataset)?;
        Ok(VerifiedOwnerApproval {
            evidence: approval,
            trust_store_sha256: embedded.source_file_sha256.clone(),
        })
    }

    fn validate(
        &self,
        embedded: &VerifiedEmbeddedTrustStore,
        protocol: &MhiValidationProtocolV1,
        dataset: &MhiValidationDatasetV1,
    ) -> Result<(), MhiValidationError> {
        if self.schema_version != 1
            || self.approval_status != "approved"
            || self.approval_purpose != "pre_scoring_physical_validation_cohort_lock"
            || !self.physical_origin_confirmed
        {
            return Err(approval("invalid approval status or purpose"));
        }
        if self.trust_store_id != embedded.store.trust_store_id
            || self.protocol_sha256 != dataset.protocol_sha256
            || self.cohort_semantic_sha256 != dataset.cohort_semantic_sha256
        {
            return Err(approval("approval protocol or cohort binding mismatch"));
        }
        let declared_root = match &protocol.physical_approval_authority {
            crate::validation_config::PhysicalApprovalAuthorityV1::EmbeddedTrustRoot {
                trust_root_id,
            } => trust_root_id,
            crate::validation_config::PhysicalApprovalAuthorityV1::NotRequested => {
                return Err(approval("software protocol cannot use physical approval"));
            }
        };
        if declared_root != &self.trust_root_id {
            return Err(approval("approval trust-root binding mismatch"));
        }
        let root = embedded.root(&self.trust_root_id)?;
        if self.project_owner_authority_id != root.project_owner_authority_id
            || self.registry_authority_id != root.registry_authority_id
        {
            return Err(approval("approval authority binding mismatch"));
        }
        let claims = protocol
            .release_scope
            .iter()
            .filter(|claim| {
                claim.requested_level
                    == crate::validation_config::RequestedValidationLevelV1::Physical
            })
            .map(|claim| claim.claim_id.clone())
            .collect::<Vec<_>>();
        if !strictly_sorted(&self.claim_ids) || self.claim_ids != claims {
            return Err(approval("approval physical claim bindings mismatch"));
        }
        let endpoints = protocol
            .release_scope
            .iter()
            .filter(|claim| {
                claim.requested_level
                    == crate::validation_config::RequestedValidationLevelV1::Physical
            })
            .flat_map(|claim| claim.supporting_endpoint_ids.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if !strictly_sorted(&self.endpoint_ids) || self.endpoint_ids != endpoints {
            return Err(approval("approval endpoint bindings mismatch"));
        }
        let reference_authority_ids = protocol
            .release_scope
            .iter()
            .filter(|claim| {
                claim.requested_level
                    == crate::validation_config::RequestedValidationLevelV1::Physical
            })
            .flat_map(|claim| claim.supporting_endpoint_ids.iter())
            .filter_map(|endpoint_id| {
                protocol
                    .mechanism_endpoints
                    .iter()
                    .find(|endpoint| &endpoint.endpoint_id == endpoint_id)
                    .map(|endpoint| &endpoint.reference_rule)
                    .or_else(|| {
                        protocol
                            .health_endpoints
                            .iter()
                            .find(|endpoint| &endpoint.endpoint_id == endpoint_id)
                            .map(|endpoint| &endpoint.reference_rule)
                    })
            })
            .flat_map(allowed_authority_ids)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if !strictly_sorted(&self.reference_authority_ids)
            || self.reference_authority_ids != reference_authority_ids
        {
            return Err(approval("approval reference-authority bindings mismatch"));
        }
        if self.target_domain != protocol.target_domain {
            return Err(approval("approval target-domain binding mismatch"));
        }
        if self.limitations.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(approval("approval limitations must be canonically ordered"));
        }
        let signing_bytes = self.signing_bytes()?;
        let expected_id = format!("sha256:{}", sha256(&signing_bytes));
        if self.approval_record_id != expected_id {
            return Err(approval("approval record ID mismatch"));
        }
        verify_strict(
            &root.owner_ed25519_public_key_hex,
            &self.owner_signature_ed25519_hex,
            &signing_bytes,
            "PhysicalApprovalOwnerSignatureInvalid",
        )?;
        verify_strict(
            &root.registry_ed25519_public_key_hex,
            &self.registry_signature_ed25519_hex,
            &signing_bytes,
            "PhysicalApprovalRegistrySignatureInvalid",
        )?;
        Ok(())
    }

    fn signing_bytes(&self) -> Result<Vec<u8>, MhiValidationError> {
        let payload = serde_json::json!({
            "identity_domain": "mhi_owner_approval_evidence_v1", "schema_version": self.schema_version,
            "approval_status": self.approval_status, "approval_purpose": self.approval_purpose,
            "trust_store_id": self.trust_store_id, "trust_root_id": self.trust_root_id,
            "project_owner_authority_id": self.project_owner_authority_id, "registry_authority_id": self.registry_authority_id,
            "owner_authority_document": self.owner_authority_document, "registry_record": self.registry_record,
            "protocol_sha256": self.protocol_sha256, "cohort_semantic_sha256": self.cohort_semantic_sha256,
            "claim_ids": self.claim_ids, "endpoint_ids": self.endpoint_ids,
            "reference_authority_ids": self.reference_authority_ids, "target_domain": self.target_domain,
            "physical_origin_confirmed": self.physical_origin_confirmed, "limitations": self.limitations,
        });
        let canonical = serde_jcs::to_vec(&payload).map_err(|error| approval(error.to_string()))?;
        let mut bytes = b"mhi_owner_approval_signature_v1\0".to_vec();
        bytes.extend(canonical);
        Ok(bytes)
    }
}

/// An approval becomes an authority capability only after complete approval
/// file and signature validation succeeds against embedded production trust.
#[derive(Debug, Clone)]
pub(crate) struct VerifiedOwnerApproval {
    evidence: OwnerApprovalEvidenceV1,
    trust_store_sha256: String,
}

impl VerifiedOwnerApproval {
    pub(crate) fn approval_record_id(&self) -> &str {
        &self.evidence.approval_record_id
    }

    pub(crate) fn evidence(&self) -> &OwnerApprovalEvidenceV1 {
        &self.evidence
    }

    pub(crate) fn trust_store_sha256(&self) -> &str {
        &self.trust_store_sha256
    }
}

fn allowed_authority_ids(rule: &ReferenceAuthorityRuleV1) -> Vec<String> {
    match rule {
        ReferenceAuthorityRuleV1::Mechanism {
            allowed_authority_ids,
            ..
        }
        | ReferenceAuthorityRuleV1::Health {
            allowed_authority_ids,
            ..
        } => allowed_authority_ids.clone(),
    }
}

fn verify_strict(
    key_hex: &str,
    signature_hex: &str,
    message: &[u8],
    error: &'static str,
) -> Result<(), MhiValidationError> {
    let bytes = hex_32(key_hex)?;
    let key = VerifyingKey::from_bytes(&bytes)
        .map_err(|_| approval("PhysicalApprovalPublicKeyInvalid"))?;
    if key.to_edwards().compress().to_bytes() != bytes {
        return Err(approval("PhysicalApprovalNoncanonicalPublicKey"));
    }
    if key.is_weak() {
        return Err(approval("PhysicalApprovalWeakPublicKey"));
    }
    let signature_bytes = decode_hex(signature_hex).map_err(|_| approval(error))?;
    let signature_bytes: [u8; 64] = signature_bytes.try_into().map_err(|_| approval(error))?;
    let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|_| approval(error))?;
    key.verify_strict(message, &signature)
        .map_err(|_| approval(error))
}
fn hex_32(value: &str) -> Result<[u8; 32], MhiValidationError> {
    let bytes = decode_hex(value)?;
    bytes
        .try_into()
        .map_err(|_| approval("PhysicalApprovalPublicKeyInvalid"))
}
fn decode_hex(value: &str) -> Result<Vec<u8>, MhiValidationError> {
    if !value.len().is_multiple_of(2)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(approval("hex must be lowercase"));
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| approval("invalid hexadecimal"))
        })
        .collect()
}
fn stable_id(name: &str, value: &str) -> Result<(), MhiValidationError> {
    if value.is_empty()
        || !value.bytes().enumerate().all(|(index, byte)| {
            (byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
                && (index != 0 || byte.is_ascii_alphanumeric())
        })
    {
        Err(approval(format!("{name} must be a stable ID")))
    } else {
        Ok(())
    }
}
fn strictly_sorted(values: &[String]) -> bool {
    !values.is_empty() && values.windows(2).all(|pair| pair[0] < pair[1])
}
fn sha256(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}
fn approval(message: impl Into<String>) -> MhiValidationError {
    MhiValidationError::Approval(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mhi_validation::MhiValidationProtocolV1, results::MhiValidationDatasetV1};
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    fn fixture(path: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/phase_e")
            .join(path)
    }

    fn test_trust() -> VerifiedEmbeddedTrustStore {
        let bytes = fs::read(fixture(
            "trust/test_only_known_answer_trust_store.schema1.json",
        ))
        .expect("literal test authority");
        let store: PhysicalApprovalTrustStoreV1 =
            serde_json::from_slice(&bytes).expect("closed test trust schema");
        store
            .validate()
            .expect("test trust is provisioned and strict");
        VerifiedEmbeddedTrustStore {
            store,
            source_file_sha256: sha256(&bytes),
        }
    }

    fn physical_authorities() -> (
        VerifiedEmbeddedTrustStore,
        MhiValidationProtocolV1,
        MhiValidationDatasetV1,
        OwnerApprovalEvidenceV1,
    ) {
        let trust = test_trust();
        let protocol = MhiValidationProtocolV1::from_toml(
            &fs::read_to_string(fixture("protocol/physical_valid.toml"))
                .expect("physical protocol"),
        )
        .expect("physical protocol validates");
        let dataset: MhiValidationDatasetV1 = serde_json::from_slice(
            &fs::read(fixture("dataset/physical_valid.schema1.json")).expect("physical dataset"),
        )
        .expect("physical dataset schema");
        let approval: OwnerApprovalEvidenceV1 = serde_json::from_slice(
            &fs::read(fixture("approval/valid.schema1.json")).expect("literal approval"),
        )
        .expect("approval schema");
        (trust, protocol, dataset, approval)
    }

    fn approval_error(error: MhiValidationError) -> String {
        match error {
            MhiValidationError::Approval(message) => message,
            other => panic!("expected approval error, got {other:?}"),
        }
    }

    #[test]
    fn phase_e_physical_claim_requires_dual_signature_embedded_trust_and_power() {
        let (trust, protocol, dataset, approval) = physical_authorities();
        approval
            .validate(&trust, &protocol, &dataset)
            .expect("literal dual-signed test vector verifies through the pure test boundary");

        let approval_file = fixture("approval/valid.schema1.json");
        let expected_file_sha256 = sha256(&fs::read(&approval_file).expect("approval bytes"));
        OwnerApprovalEvidenceV1::read_and_validate(
            &approval_file,
            &expected_file_sha256,
            &trust,
            &protocol,
            &dataset,
        )
        .expect("file hash and literal approval verify");
        assert!(matches!(
            OwnerApprovalEvidenceV1::read_and_validate(
                &approval_file,
                &"0".repeat(64),
                &trust,
                &protocol,
                &dataset,
            ),
            Err(MhiValidationError::Approval(message))
                if message == "approval file SHA-256 mismatch"
        ));
        assert!(matches!(
            OwnerApprovalEvidenceV1::read_and_validate(
                &fixture("approval/missing.schema1.json"),
                &expected_file_sha256,
                &trust,
                &protocol,
                &dataset,
            ),
            Err(MhiValidationError::Io { .. })
        ));

        let mut wrong_purpose = approval.clone();
        wrong_purpose.approval_purpose = "wrong".into();
        assert_eq!(
            approval_error(
                wrong_purpose
                    .validate(&trust, &protocol, &dataset)
                    .unwrap_err()
            ),
            "invalid approval status or purpose"
        );

        let mut wrong_root = approval.clone();
        wrong_root.trust_root_id = "attacker_root".into();
        assert_eq!(
            approval_error(
                wrong_root
                    .validate(&trust, &protocol, &dataset)
                    .unwrap_err()
            ),
            "approval trust-root binding mismatch"
        );

        let mut wrong_owner = approval.clone();
        wrong_owner.project_owner_authority_id = "attacker_owner".into();
        assert_eq!(
            approval_error(
                wrong_owner
                    .validate(&trust, &protocol, &dataset)
                    .unwrap_err()
            ),
            "approval authority binding mismatch"
        );

        let mut copied_signature = approval.clone();
        copied_signature.registry_signature_ed25519_hex =
            copied_signature.owner_signature_ed25519_hex.clone();
        assert_eq!(
            approval_error(
                copied_signature
                    .validate(&trust, &protocol, &dataset)
                    .unwrap_err()
            ),
            "PhysicalApprovalRegistrySignatureInvalid"
        );

        let mut malformed_signature = approval.clone();
        malformed_signature.owner_signature_ed25519_hex = "00".into();
        assert_eq!(
            approval_error(
                malformed_signature
                    .validate(&trust, &protocol, &dataset)
                    .unwrap_err()
            ),
            "PhysicalApprovalOwnerSignatureInvalid"
        );

        let mut wrong_record = approval.clone();
        wrong_record.approval_record_id =
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
        assert_eq!(
            approval_error(
                wrong_record
                    .validate(&trust, &protocol, &dataset)
                    .unwrap_err()
            ),
            "approval record ID mismatch"
        );

        let mut wrong_cohort = approval.clone();
        wrong_cohort.cohort_semantic_sha256 =
            "0000000000000000000000000000000000000000000000000000000000000000".into();
        assert_eq!(
            approval_error(
                wrong_cohort
                    .validate(&trust, &protocol, &dataset)
                    .unwrap_err()
            ),
            "approval protocol or cohort binding mismatch"
        );

        let identity_key = "0100000000000000000000000000000000000000000000000000000000000000";
        let nondecompressible_key =
            "0200000000000000000000000000000000000000000000000000000000000000";
        for (key, expected) in [
            (identity_key, "PhysicalApprovalWeakPublicKey"),
            (nondecompressible_key, "PhysicalApprovalPublicKeyInvalid"),
        ] {
            for owner_role in [true, false] {
                let mut mutated = trust.store.clone();
                if owner_role {
                    mutated.trust_roots[0].owner_ed25519_public_key_hex = key.into();
                } else {
                    mutated.trust_roots[0].registry_ed25519_public_key_hex = key.into();
                }
                assert_eq!(approval_error(mutated.validate().unwrap_err()), expected);
            }
        }
    }

    #[test]
    fn phase_e_artifact_contracts_accept_exact_schema1_and_reject_invalid_variants() {
        let trusted = test_trust();
        assert_eq!(
            trusted.store.provisioning_state,
            PhysicalApprovalProvisioningStateV1::Provisioned
        );
        assert!(trusted.store.validate().is_ok());

        let mut schema2 = trusted.store.clone();
        schema2.schema_version = 2;
        assert_eq!(
            approval_error(schema2.validate().unwrap_err()),
            "invalid embedded trust-store identity"
        );

        let mut unprovisioned_with_root = trusted.store.clone();
        unprovisioned_with_root.provisioning_state =
            PhysicalApprovalProvisioningStateV1::Unprovisioned;
        assert_eq!(
            approval_error(unprovisioned_with_root.validate().unwrap_err()),
            "UNPROVISIONED trust store must have no trust roots"
        );

        let mut provisioned_without_root = trusted.store.clone();
        provisioned_without_root.trust_roots.clear();
        assert_eq!(
            approval_error(provisioned_without_root.validate().unwrap_err()),
            "PROVISIONED trust store must have trust roots"
        );

        let mut duplicate_authority = trusted.store.clone();
        duplicate_authority.trust_roots[0].registry_authority_id = duplicate_authority.trust_roots
            [0]
        .project_owner_authority_id
        .clone();
        assert_eq!(
            approval_error(duplicate_authority.validate().unwrap_err()),
            "trust authority IDs must be globally unique"
        );

        let mut duplicate_key = trusted.store.clone();
        duplicate_key.trust_roots[0].registry_ed25519_public_key_hex = duplicate_key.trust_roots[0]
            .owner_ed25519_public_key_hex
            .clone();
        assert_eq!(
            approval_error(duplicate_key.validate().unwrap_err()),
            "trust public keys must be globally unique"
        );

        let mut noncanonical = trusted.store.clone();
        noncanonical.trust_roots[0].owner_ed25519_public_key_hex =
            "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f".into();
        assert_eq!(
            approval_error(noncanonical.validate().unwrap_err()),
            "PhysicalApprovalNoncanonicalPublicKey"
        );

        let alias = br#"{
          "schema_version":1,
          "trust_store_id":"mhi_physical_approval_trust_store_v1",
          "provisioning_state":"UNPROVISIONED",
          "roots":[]
        }"#;
        assert!(serde_json::from_slice::<PhysicalApprovalTrustStoreV1>(alias).is_err());
    }
}
