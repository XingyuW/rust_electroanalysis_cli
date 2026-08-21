//! Durable Phase-E validation dataset and report artifacts.
//!
//! Both kinds are schema-1-only and additive; no legacy Phase-B/C/D payload is
//! migrated into this validation vocabulary.

use crate::{
    domain::{
        ArtifactAcquisitionFamilies, ArtifactError, ArtifactExperimentScope, ArtifactId,
        ArtifactKind, CurrentArtifactKindPolicy, ScopeKey, VersionedArtifact,
        validate_serialized_finite,
    },
    validation_config::{
        BlindingStateV1, CohortRoleV1, DomainKeyV1, EvidenceOriginV1, HealthTargetV1,
        ReferenceDependencyCompletenessV1, ReleaseClaimOutcomeV1, ValidationOutcomeV1,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    pub expected_artifact_kind: ArtifactKind,
    pub expected_schema_version: u32,
    pub source_file_sha256: String,
    pub expected_lineage: ExpectedLineageV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyLineageReasonV1 {
    FieldAbsentInLegacyArtifact,
    ExternalArtifactWithoutLineage,
    MigrationInformationUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectedLineageV1 {
    Known {
        artifact_id: ArtifactId,
        semantic_sha256: String,
    },
    LegacyUnknown {
        schema_version: u32,
        legacy_source_fingerprint: String,
        reason: LegacyLineageReasonV1,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredScopeV1 {
    pub experiment_scope: ArtifactExperimentScope,
    pub sensor_scope: ScopeKey,
    pub channel_scope: ScopeKey,
    pub acquisition_families: ArtifactAcquisitionFamilies,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScientificSourceKeyV1 {
    Known {
        artifact_kind: ArtifactKind,
        artifact_id: ArtifactId,
        semantic_sha256: String,
    },
    LegacyUnknown {
        artifact_kind: ArtifactKind,
        schema_version: u32,
        source_file_sha256: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferenceDependencyV1 {
    ReferenceSource { reference_source_id: String },
    ScientificArtifact { source: ScientificSourceKeyV1 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSourceAuthorityV1 {
    pub reference_source_id: String,
    pub source_file_sha256: String,
    pub evidence_origin: EvidenceOriginV1,
    pub dependency_completeness: ReferenceDependencyCompletenessV1,
    pub experiment_scope: ArtifactExperimentScope,
    pub acquisition_families: ArtifactAcquisitionFamilies,
    pub direct_dependencies: Vec<ReferenceDependencyV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferenceUncertaintyV1 {
    Quantified {
        measure_id: String,
        value: f64,
        unit: String,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferenceEndpointV1 {
    Mechanism {
        endpoint_id: String,
        reference_endpoint_id: String,
        reference_source_id: String,
        hypothesis_id: String,
        outcome: MechanismReferenceOutcomeV1,
        method_id: String,
        method_version: String,
        authority_id: String,
        blinding_state: BlindingStateV1,
        uncertainty: ReferenceUncertaintyV1,
        limitations: Vec<String>,
    },
    Health {
        endpoint_id: String,
        reference_endpoint_id: String,
        reference_source_id: String,
        target: HealthTargetV1,
        label: String,
        method_id: String,
        method_version: String,
        authority_id: String,
        blinding_state: BlindingStateV1,
        uncertainty: ReferenceUncertaintyV1,
        limitations: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MechanismReferenceOutcomeV1 {
    Supports,
    Contradicts,
    NotAssessed,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRecordV1 {
    pub record_id: String,
    pub cohort_role: CohortRoleV1,
    pub mechanism_source: Option<ArtifactSourceExpectationV1>,
    pub health_source: Option<ArtifactSourceExpectationV1>,
    pub declared_scope: DeclaredScopeV1,
    pub domain: DomainKeyV1,
    pub evidence_origin: EvidenceOriginV1,
    pub reference_endpoints: Vec<ReferenceEndpointV1>,
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
    pub artifact_kind: String,
    pub dataset_id: String,
    pub protocol_sha256: String,
    pub cohort_semantic_sha256: String,
    pub lineage_catalog_source: RelativeSourceV1,
    pub reference_sources: Vec<ReferenceSourceAuthorityV1>,
    pub records: Vec<ValidationRecordV1>,
    pub owner_approval_source: Option<OwnerApprovalSourceV1>,
    pub lineage: crate::domain::ArtifactLineageState,
    pub provenance: serde_json::Value,
    pub warnings: Vec<serde_json::Value>,
}

impl MhiValidationDatasetV1 {
    /// Reconstructs the cohort authority before the optional approval
    /// attachment is considered.  The caller retains the exact source-file
    /// hash separately; this semantic identity deliberately contains no path
    /// or operational provenance.
    pub fn computed_cohort_semantic_sha256(&self) -> Result<String, ArtifactError> {
        let preimage = serde_json::json!({
            "identity_domain": "mhi_validation_cohort_v1",
            "schema_version": 1,
            "dataset_id": self.dataset_id,
            "protocol_sha256": self.protocol_sha256,
            "records": self.records,
            "reference_sources": self.reference_sources,
            "lineage_catalog_source_sha256": self.lineage_catalog_source.source_file_sha256,
        });
        let bytes = serde_jcs::to_vec(&preimage).map_err(|error| ArtifactError::Validation {
            message: format!("MHI validation cohort canonicalization failed: {error}"),
        })?;
        let mut hash = Sha256::new();
        hash.update(bytes);
        Ok(format!("{:x}", hash.finalize()))
    }

    pub fn validate_structure(&self) -> Result<(), ArtifactError> {
        if self.schema_version != 1 {
            return invalid("MHI validation dataset is schema-1 only");
        }
        if self.artifact_kind != "mhi_validation_dataset" {
            return invalid("MHI validation dataset kind must be mhi_validation_dataset");
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
        let mut reference_source_ids = BTreeSet::new();
        let mut previous_reference_source = None;
        for source in &self.reference_sources {
            valid_id("reference_source_id", &source.reference_source_id)?;
            sha("reference source_file_sha256", &source.source_file_sha256)?;
            source
                .experiment_scope
                .validate()
                .map_err(|error| ArtifactError::Validation {
                    message: error.to_string(),
                })?;
            source
                .acquisition_families
                .validate()
                .map_err(|error| ArtifactError::Validation {
                    message: error.to_string(),
                })?;
            if previous_reference_source
                .as_ref()
                .is_some_and(|previous: &String| previous >= &source.reference_source_id)
                || !reference_source_ids.insert(source.reference_source_id.clone())
            {
                return invalid("reference source IDs must be canonical and unique");
            }
            previous_reference_source = Some(source.reference_source_id.clone());
            validate_reference_dependencies(&source.direct_dependencies)?;
        }
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
                validate_expected_lineage(&source.expected_lineage)?;
                if !sources.insert((
                    source.expected_artifact_kind.as_str().to_string(),
                    source.source_file_sha256.clone(),
                )) {
                    return invalid("duplicate assessed scientific source key");
                }
            }
            validate_record_references(record, &reference_source_ids)?;
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
        if self.computed_cohort_semantic_sha256()? != self.cohort_semantic_sha256 {
            return invalid("cohort_semantic_sha256 does not match the canonical dataset preimage");
        }
        validate_serialized_finite(self)
    }
}

fn validate_expected_lineage(value: &ExpectedLineageV1) -> Result<(), ArtifactError> {
    match value {
        ExpectedLineageV1::Known {
            artifact_id,
            semantic_sha256,
        } => {
            sha("expected lineage semantic_sha256", semantic_sha256)?;
            if artifact_id.0 != format!("sha256:{semantic_sha256}") {
                return invalid("known expected lineage artifact ID must bind semantic_sha256");
            }
        }
        ExpectedLineageV1::LegacyUnknown {
            schema_version,
            legacy_source_fingerprint,
            ..
        } => {
            if *schema_version == 0 {
                return invalid("legacy expected lineage schema_version must be positive");
            }
            sha("legacy_source_fingerprint", legacy_source_fingerprint)?;
        }
    }
    Ok(())
}

fn validate_reference_dependencies(values: &[ReferenceDependencyV1]) -> Result<(), ArtifactError> {
    let mut previous = None;
    for value in values {
        let key = match value {
            ReferenceDependencyV1::ReferenceSource {
                reference_source_id,
            } => {
                valid_id("reference dependency source ID", reference_source_id)?;
                format!("0:{reference_source_id}")
            }
            ReferenceDependencyV1::ScientificArtifact { source } => match source {
                ScientificSourceKeyV1::Known {
                    artifact_kind,
                    artifact_id,
                    semantic_sha256,
                } => {
                    sha("reference scientific semantic_sha256", semantic_sha256)?;
                    if artifact_id.0 != format!("sha256:{semantic_sha256}") {
                        return invalid(
                            "reference scientific source artifact ID does not bind hash",
                        );
                    }
                    format!(
                        "1:{}:{}:{semantic_sha256}",
                        artifact_kind.as_str(),
                        artifact_id.0
                    )
                }
                ScientificSourceKeyV1::LegacyUnknown {
                    artifact_kind,
                    schema_version,
                    source_file_sha256,
                } => {
                    if *schema_version == 0 {
                        return invalid("legacy scientific source schema must be positive");
                    }
                    sha("legacy scientific source hash", source_file_sha256)?;
                    format!(
                        "1:{}:{schema_version}:{source_file_sha256}",
                        artifact_kind.as_str()
                    )
                }
            },
        };
        if previous.as_ref().is_some_and(|last: &String| last >= &key) {
            return invalid("reference dependencies must be canonical and unique");
        }
        previous = Some(key);
    }
    Ok(())
}

fn validate_record_references(
    record: &ValidationRecordV1,
    reference_source_ids: &BTreeSet<String>,
) -> Result<(), ArtifactError> {
    let mut previous = None;
    let mut ids = BTreeSet::new();
    for reference in &record.reference_endpoints {
        let (endpoint_id, reference_endpoint_id, reference_source_id, key) = match reference {
            ReferenceEndpointV1::Mechanism {
                endpoint_id,
                reference_endpoint_id,
                reference_source_id,
                hypothesis_id,
                outcome: _,
                method_id,
                method_version,
                authority_id,
                blinding_state: _,
                uncertainty,
                limitations,
            } => {
                valid_id("mechanism reference hypothesis_id", hypothesis_id)?;
                valid_reference_metadata(
                    method_id,
                    method_version,
                    authority_id,
                    uncertainty,
                    limitations,
                )?;
                (
                    endpoint_id,
                    reference_endpoint_id,
                    reference_source_id,
                    format!("{endpoint_id}:{reference_endpoint_id}"),
                )
            }
            ReferenceEndpointV1::Health {
                endpoint_id,
                reference_endpoint_id,
                reference_source_id,
                target: _,
                label,
                method_id,
                method_version,
                authority_id,
                blinding_state: _,
                uncertainty,
                limitations,
            } => {
                valid_id("health reference label", label)?;
                valid_reference_metadata(
                    method_id,
                    method_version,
                    authority_id,
                    uncertainty,
                    limitations,
                )?;
                (
                    endpoint_id,
                    reference_endpoint_id,
                    reference_source_id,
                    format!("{endpoint_id}:{reference_endpoint_id}"),
                )
            }
        };
        valid_id("reference endpoint endpoint_id", endpoint_id)?;
        valid_id("reference_endpoint_id", reference_endpoint_id)?;
        valid_id("reference_source_id", reference_source_id)?;
        if !reference_source_ids.contains(reference_source_id) {
            return invalid("reference endpoint names an unknown reference source");
        }
        if previous.as_ref().is_some_and(|last: &String| last >= &key)
            || !ids.insert(reference_endpoint_id.clone())
        {
            return invalid("record reference endpoints must be canonical and unique");
        }
        previous = Some(key);
    }
    Ok(())
}

fn valid_reference_metadata(
    method_id: &str,
    method_version: &str,
    authority_id: &str,
    uncertainty: &ReferenceUncertaintyV1,
    limitations: &[String],
) -> Result<(), ArtifactError> {
    valid_id("reference method_id", method_id)?;
    valid_id("reference authority_id", authority_id)?;
    if method_version.is_empty() || limitations.iter().any(|value| value.is_empty()) {
        return invalid("reference method version and limitations must be nonempty when present");
    }
    match uncertainty {
        ReferenceUncertaintyV1::Quantified {
            measure_id,
            value,
            unit,
        } => {
            valid_id("reference uncertainty measure_id", measure_id)?;
            if !value.is_finite()
                || *value < 0.0
                || value.to_bits() == (-0.0f64).to_bits()
                || unit.is_empty()
            {
                return invalid("quantified reference uncertainty must be finite nonnegative");
            }
        }
        ReferenceUncertaintyV1::Unavailable { reason } if reason.is_empty() => {
            return invalid("unavailable reference uncertainty requires a reason");
        }
        ReferenceUncertaintyV1::Unavailable { .. } => {}
    }
    Ok(())
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
        if self.artifact_kind != "mhi_validation_report" {
            return invalid("MHI validation report kind must be mhi_validation_report");
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
