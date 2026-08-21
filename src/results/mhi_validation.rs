//! Durable Phase-E validation dataset and report artifacts.
//!
//! Both kinds are schema-1-only and additive; no legacy Phase-B/C/D payload is
//! migrated into this validation vocabulary.

use crate::{
    domain::{
        ArtifactError, ArtifactKind, CurrentArtifactKindPolicy, VersionedArtifact,
        validate_serialized_finite,
    },
    validation_config::{
        CohortRoleV1, DomainKeyV1, EvidenceOriginV1, ReleaseClaimOutcomeV1, ValidationOutcomeV1,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelativeSourceV1 {
    pub relative_path: String,
    pub schema_version: u32,
    pub source_file_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSourceExpectationV1 {
    pub relative_path: String,
    pub expected_artifact_kind: String,
    pub expected_schema_version: u32,
    pub source_file_sha256: String,
    pub expected_lineage: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRecordV1 {
    pub record_id: String,
    pub cohort_role: CohortRoleV1,
    pub mechanism_source: Option<ArtifactSourceExpectationV1>,
    pub health_source: Option<ArtifactSourceExpectationV1>,
    pub declared_scope: serde_json::Value,
    pub domain: DomainKeyV1,
    pub evidence_origin: EvidenceOriginV1,
    pub reference_endpoints: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerApprovalSourceV1 {
    pub relative_path: String,
    pub schema_version: u32,
    pub source_file_sha256: String,
    pub expected_approval_record_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MhiValidationDatasetV1 {
    pub schema_version: u32,
    #[serde(default)]
    pub artifact_kind: String,
    pub dataset_id: String,
    pub protocol_sha256: String,
    pub cohort_semantic_sha256: String,
    pub lineage_catalog_source: RelativeSourceV1,
    pub reference_sources: Vec<serde_json::Value>,
    pub records: Vec<ValidationRecordV1>,
    pub owner_approval_source: Option<OwnerApprovalSourceV1>,
    pub lineage: crate::domain::ArtifactLineageState,
    pub provenance: serde_json::Value,
    pub warnings: Vec<serde_json::Value>,
}

impl MhiValidationDatasetV1 {
    pub fn validate_structure(&self) -> Result<(), ArtifactError> {
        if self.schema_version != 1 {
            return invalid("MHI validation dataset is schema-1 only");
        }
        valid_id("dataset_id", &self.dataset_id)?;
        sha("protocol_sha256", &self.protocol_sha256)?;
        sha("cohort_semantic_sha256", &self.cohort_semantic_sha256)?;
        valid_relative_path(&self.lineage_catalog_source.relative_path)?;
        if self.lineage_catalog_source.schema_version != 1 {
            return invalid("lineage catalog schema must be 1");
        }
        sha(
            "lineage catalog source_file_sha256",
            &self.lineage_catalog_source.source_file_sha256,
        )?;
        if self.records.is_empty() {
            return invalid("validation dataset records must be nonempty");
        }
        let mut ids = BTreeSet::new();
        let mut sources = BTreeSet::new();
        let mut previous = None;
        for record in &self.records {
            valid_id("record_id", &record.record_id)?;
            if previous
                .as_ref()
                .is_some_and(|id: &String| id >= &record.record_id)
            {
                return invalid("dataset records must be canonical and unique");
            }
            previous = Some(record.record_id.clone());
            if !ids.insert(record.record_id.clone()) {
                return invalid("dataset record IDs must be unique");
            }
            if !record.domain.temperature_kelvin.is_finite()
                || record.domain.temperature_kelvin <= 0.0
                || record.domain.temperature_kelvin.to_bits() == (-0.0f64).to_bits()
            {
                return invalid("dataset temperatures must be finite and positive");
            }
            for source in [
                record.mechanism_source.as_ref(),
                record.health_source.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                valid_relative_path(&source.relative_path)?;
                sha("source_file_sha256", &source.source_file_sha256)?;
                if !sources.insert((
                    source.expected_artifact_kind.clone(),
                    source.source_file_sha256.clone(),
                )) {
                    return invalid("duplicate assessed scientific source key");
                }
            }
        }
        if let Some(source) = &self.owner_approval_source {
            valid_relative_path(&source.relative_path)?;
            sha(
                "owner approval source_file_sha256",
                &source.source_file_sha256,
            )?;
            valid_id(
                "expected_approval_record_id",
                &source.expected_approval_record_id,
            )?;
            if source.schema_version != 1 {
                return invalid("owner approval schema must be 1");
            }
        }
        validate_serialized_finite(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseClaimResultV1 {
    pub claim_id: String,
    pub requested_level: crate::validation_config::RequestedValidationLevelV1,
    pub outcome: ReleaseClaimOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MhiValidationReportV1 {
    pub schema_version: u32,
    #[serde(default)]
    pub artifact_kind: String,
    pub report_id: String,
    pub protocol_sha256: String,
    pub dataset_id: String,
    pub dataset_source_file_sha256: String,
    pub approval_trust_store_sha256: Option<String>,
    pub release_claims: Vec<ReleaseClaimResultV1>,
    pub overall_status: ValidationOutcomeV1,
    pub payload: serde_json::Value,
    pub lineage: crate::domain::ArtifactLineageState,
    pub provenance: serde_json::Value,
    pub warnings: Vec<serde_json::Value>,
}

impl MhiValidationReportV1 {
    pub fn validate_structure(&self) -> Result<(), ArtifactError> {
        if self.schema_version != 1 {
            return invalid("MHI validation report is schema-1 only");
        }
        if !self.report_id.starts_with("sha256:") || self.report_id.len() != 71 {
            return invalid("report_id must be a SHA-256 artifact ID");
        }
        sha("protocol_sha256", &self.protocol_sha256)?;
        sha(
            "dataset_source_file_sha256",
            &self.dataset_source_file_sha256,
        )?;
        if let Some(hash) = &self.approval_trust_store_sha256 {
            sha("approval_trust_store_sha256", hash)?;
        }
        let mut claims = BTreeSet::new();
        for claim in &self.release_claims {
            valid_id("claim_id", &claim.claim_id)?;
            if !claims.insert(claim.claim_id.clone()) {
                return invalid("release claim IDs must be unique");
            }
        }
        validate_serialized_finite(self)
    }
}

impl VersionedArtifact for MhiValidationDatasetV1 {
    const ARTIFACT_KIND: ArtifactKind = ArtifactKind::MhiValidationDataset;
    const CURRENT_SCHEMA_VERSION: u32 = 1;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32] = &[];
    const CURRENT_ARTIFACT_KIND_POLICY: CurrentArtifactKindPolicy =
        CurrentArtifactKindPolicy::Required;
    fn schema_version(&self) -> u32 {
        self.schema_version
    }
    fn validate_before_json(&self) -> Result<(), ArtifactError> {
        self.validate_structure()
    }
    fn validate_after_read(&self) -> Result<(), ArtifactError> {
        self.validate_structure()
    }
}

impl VersionedArtifact for MhiValidationReportV1 {
    const ARTIFACT_KIND: ArtifactKind = ArtifactKind::MhiValidationReport;
    const CURRENT_SCHEMA_VERSION: u32 = 1;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32] = &[];
    const CURRENT_ARTIFACT_KIND_POLICY: CurrentArtifactKindPolicy =
        CurrentArtifactKindPolicy::Required;
    fn schema_version(&self) -> u32 {
        self.schema_version
    }
    fn validate_before_json(&self) -> Result<(), ArtifactError> {
        self.validate_structure()
    }
    fn validate_after_read(&self) -> Result<(), ArtifactError> {
        self.validate_structure()
    }
}

fn invalid(message: impl Into<String>) -> Result<(), ArtifactError> {
    Err(ArtifactError::Validation {
        message: message.into(),
    })
}
fn valid_id(name: &str, value: &str) -> Result<(), ArtifactError> {
    if value.is_empty()
        || !value.bytes().enumerate().all(|(index, byte)| {
            (byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
                && (index != 0 || byte.is_ascii_alphanumeric())
        })
    {
        invalid(format!("{name} must be a stable ID"))
    } else {
        Ok(())
    }
}
fn sha(name: &str, value: &str) -> Result<(), ArtifactError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        invalid(format!("{name} must be lowercase SHA-256"))
    } else {
        Ok(())
    }
}
fn valid_relative_path(path: &str) -> Result<(), ArtifactError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\0')
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
        || path.contains('\\')
    {
        invalid("relative path is unsafe")
    } else {
        Ok(())
    }
}
