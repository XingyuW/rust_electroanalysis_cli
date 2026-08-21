//! Immutable embedded trust authority for physical Phase-E claims.

use super::{MhiValidationError, protocol::MhiValidationProtocolV1};
use crate::results::MhiValidationDatasetV1;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

const EMBEDDED_TRUST_STORE: &[u8] =
    include_bytes!("../../config/mhi_physical_approval_trust_store.schema1.json");

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
    pub trust_roots: Vec<PhysicalApprovalTrustRootV1>,
}

#[derive(Debug, Clone)]
pub struct VerifiedEmbeddedTrustStore {
    pub store: PhysicalApprovalTrustStoreV1,
    pub source_file_sha256: String,
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
        if self.schema_version != 1
            || self.trust_store_id != "mhi_physical_approval_trust_store_v1"
            || self.trust_roots.is_empty()
        {
            return Err(approval("invalid embedded trust-store identity"));
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

    pub fn root(&self, id: &str) -> Result<&PhysicalApprovalTrustRootV1, MhiValidationError> {
        self.trust_roots
            .iter()
            .find(|root| root.trust_root_id == id)
            .ok_or_else(|| approval("declared trust root does not exist"))
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
    pub fn read_and_validate(
        path: &Path,
        expected_file_sha256: &str,
        embedded: &VerifiedEmbeddedTrustStore,
        protocol: &MhiValidationProtocolV1,
        dataset: &MhiValidationDatasetV1,
    ) -> Result<Self, MhiValidationError> {
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
        Ok(approval)
    }

    pub fn validate(
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
        let root = embedded.store.root(&self.trust_root_id)?;
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

fn verify_strict(
    key_hex: &str,
    signature_hex: &str,
    message: &[u8],
    error: &'static str,
) -> Result<(), MhiValidationError> {
    let bytes = hex_32(key_hex)?;
    let key = VerifyingKey::from_bytes(&bytes)
        .map_err(|_| approval("PhysicalApprovalPublicKeyInvalid"))?;
    if key.to_edwards().compress().to_bytes() != bytes || key.is_weak() {
        return Err(approval("PhysicalApprovalPublicKeyInvalid"));
    }
    let signature =
        Signature::try_from(hex_64(signature_hex)?.as_slice()).map_err(|_| approval(error))?;
    key.verify_strict(message, &signature)
        .map_err(|_| approval(error))
}
fn hex_32(value: &str) -> Result<[u8; 32], MhiValidationError> {
    let bytes = decode_hex(value)?;
    bytes
        .try_into()
        .map_err(|_| approval("PhysicalApprovalPublicKeyInvalid"))
}
fn hex_64(value: &str) -> Result<[u8; 64], MhiValidationError> {
    let bytes = decode_hex(value)?;
    bytes
        .try_into()
        .map_err(|_| approval("invalid Ed25519 signature encoding"))
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
