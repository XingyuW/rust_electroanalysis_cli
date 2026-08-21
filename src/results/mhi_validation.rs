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
    mhi_validation::statistics::MetricValueV1,
    validation_config::{
        AcceptanceRuleV1, BlindingStateV1, CohortRoleV1, DomainKeyV1, DomainSelectorV1,
        EndpointKindV1, EvidenceOriginV1, ExclusionReasonV1, HealthTargetV1,
        LeakageNotEvaluatedReasonV1, RecordDecisionV1, ReferenceDependencyCompletenessV1,
        ReleaseClaimOutcomeV1, RequestedValidationLevelV1, RuleEvaluationResultV1,
        SeparationStatusV1, SeparationUnknownReasonV1, ValidationOutcomeV1,
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

    /// Binds a closed dataset to one exact normalized protocol before the
    /// reader opens any scientific source.  This is where the approved
    /// endpoint-scoped scientific key is enforceable: the same source may be
    /// relevant to different endpoint kinds, but cannot inflate a single
    /// endpoint denominator under another record ID or path.
    pub fn validate_against_protocol(
        &self,
        protocol: &crate::mhi_validation::MhiValidationProtocolV1,
        protocol_sha256: &str,
    ) -> Result<(), crate::mhi_validation::MhiValidationError> {
        self.validate_structure().map_err(|error| {
            crate::mhi_validation::MhiValidationError::Dataset(error.to_string())
        })?;
        if self.protocol_sha256 != protocol_sha256 {
            return Err(crate::mhi_validation::MhiValidationError::Dataset(
                "dataset protocol_sha256 does not bind the exact protocol bytes".into(),
            ));
        }
        let ensure_unique = |endpoint_id: &str,
                             mechanism: bool,
                             role: CohortRoleV1,
                             domain: &DomainSelectorV1|
         -> Result<(), crate::mhi_validation::MhiValidationError> {
            let mut keys = BTreeSet::new();
            for record in &self.records {
                if record.cohort_role != role || !domain.contains(&record.domain) {
                    continue;
                }
                let source = if mechanism {
                    record.mechanism_source.as_ref()
                } else {
                    record.health_source.as_ref()
                };
                let Some(source) = source else {
                    continue;
                };
                let key = match &source.expected_lineage {
                    ExpectedLineageV1::Known {
                        artifact_id,
                        semantic_sha256,
                    } => format!(
                        "{}:{}:{}:{}",
                        source.expected_artifact_kind.as_str(),
                        artifact_id.0,
                        semantic_sha256,
                        endpoint_id
                    ),
                    ExpectedLineageV1::LegacyUnknown { schema_version, .. } => format!(
                        "{}:{}:{}:{}",
                        source.expected_artifact_kind.as_str(),
                        schema_version,
                        source.source_file_sha256,
                        endpoint_id
                    ),
                };
                if !keys.insert(key) {
                    return Err(crate::mhi_validation::MhiValidationError::Dataset(
                        "duplicate assessed scientific source key for endpoint".into(),
                    ));
                }
            }
            Ok(())
        };
        for endpoint in &protocol.mechanism_endpoints {
            ensure_unique(
                &endpoint.endpoint_id,
                true,
                endpoint.cohort_role,
                &endpoint.domain,
            )?;
        }
        for endpoint in &protocol.health_endpoints {
            ensure_unique(
                &endpoint.endpoint_id,
                false,
                endpoint.cohort_role,
                &endpoint.domain,
            )?;
        }
        Ok(())
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
    pub requested_level: RequestedValidationLevelV1,
    pub statement: String,
    pub domain: DomainSelectorV1,
    pub supporting_endpoint_ids: Vec<String>,
    pub outcome: ReleaseClaimOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolAuthorityV1 {
    pub protocol_id: String,
    pub schema_version: u32,
    pub source_file_sha256: String,
    pub registration: crate::mhi_validation::protocol::ProtocolRegistrationV1,
    pub physical_approval_authority: crate::validation_config::PhysicalApprovalAuthorityV1,
    pub normalized_protocol: crate::mhi_validation::MhiValidationProtocolV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetSourceReferenceV1 {
    Known {
        dataset_id: String,
        schema: u32,
        artifact_id: ArtifactId,
        semantic_sha256: String,
        source_file_sha256: String,
    },
    LegacyUnknown {
        dataset_id: String,
        schema: u32,
        legacy_fingerprint: String,
        source_file_sha256: String,
        reason: LegacyLineageReasonV1,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetAuthorityV1 {
    pub dataset_id: String,
    pub schema_version: u32,
    pub protocol_sha256: String,
    pub cohort_semantic_sha256: String,
    pub source: DatasetSourceReferenceV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableDocumentReferenceV1 {
    pub immutable_reference_uri: String,
    pub document_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalAuthorityV1 {
    pub approval_source_file_sha256: String,
    pub approval_record_id: String,
    pub trust_store_id: String,
    pub approval_purpose: String,
    pub trust_store_sha256: String,
    pub trust_root_id: String,
    pub project_owner_authority_id: String,
    pub registry_authority_id: String,
    pub owner_authority_document: ImmutableDocumentReferenceV1,
    pub registry_record: ImmutableDocumentReferenceV1,
    pub owner_signature_verified: bool,
    pub registry_signature_verified: bool,
    pub binding_status: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilitySourceRoleV1 {
    Protocol,
    Dataset,
    LineageCatalog,
    OwnerApproval,
    MechanismSource,
    HealthSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityResultV1 {
    Compatible,
    ReadableLegacyExcluded,
    CurrentLegacyUnknownExcluded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityRowV1 {
    pub record_id: Option<String>,
    pub source_role: CompatibilitySourceRoleV1,
    pub relative_path: String,
    pub expected_kind: Option<ArtifactKind>,
    pub actual_kind: Option<ArtifactKind>,
    pub expected_schema: u32,
    pub actual_schema: u32,
    pub expected_file_sha256: String,
    pub actual_file_sha256: String,
    pub expected_artifact_id: Option<ArtifactId>,
    pub actual_artifact_id: Option<ArtifactId>,
    pub expected_semantic_sha256: Option<String>,
    pub actual_semantic_sha256: Option<String>,
    pub result: CompatibilityResultV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordAccountingRowV1 {
    pub endpoint_id: String,
    pub stratum_id: String,
    pub record_id: String,
    pub decision: RecordDecisionV1,
    pub primary_reason: Option<ExclusionReasonV1>,
    pub secondary_reasons: Vec<ExclusionReasonV1>,
    pub assessed_source_key: Option<ScientificSourceKeyV1>,
    pub reference_endpoint_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CohortRowV1 {
    pub endpoint_id: String,
    pub stratum_id: String,
    pub endpoint_kind: EndpointKindV1,
    pub cohort_role: CohortRoleV1,
    pub declared_record_ids: Vec<String>,
    pub eligible_record_ids: Vec<String>,
    pub excluded_record_ids: Vec<String>,
    pub not_applicable_record_ids: Vec<String>,
    pub development_record_ids: Vec<String>,
    pub validation_record_ids: Vec<String>,
    pub holdout_record_ids: Vec<String>,
    pub declared_count: u64,
    pub eligible_count: u64,
    pub excluded_count: u64,
    pub not_applicable_count: u64,
    pub exclusion_rate: MetricValueV1,
    pub evaluable_count: Option<u64>,
    pub indeterminate_count: Option<u64>,
    pub data_quality_insufficient_count: Option<u64>,
    pub coverage: Option<MetricValueV1>,
    pub indeterminate_rate: Option<MetricValueV1>,
    pub data_quality_insufficient_rate: Option<MetricValueV1>,
    pub outcome: ValidationOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeakageRowV1 {
    pub endpoint_id: String,
    pub stratum_id: String,
    pub record_id: String,
    pub separation_status: Option<SeparationStatusV1>,
    pub not_evaluated_reason: Option<LeakageNotEvaluatedReasonV1>,
    pub compared_development_record_ids: Vec<String>,
    pub shared_artifact_ids: Vec<ArtifactId>,
    pub shared_source_sha256s: Vec<String>,
    pub shared_experiment_ids: Vec<String>,
    pub shared_family_ids: Vec<String>,
    pub unknown_reasons: Vec<SeparationUnknownReasonV1>,
    pub decision: RecordDecisionV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuleActualV1 {
    Count { value: u64 },
    BinomialRate { value: MetricValueV1 },
    BalancedAccuracy { value: BalancedAccuracyV1 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleEvaluationV1 {
    pub rule: AcceptanceRuleV1,
    pub actual: RuleActualV1,
    pub result: RuleEvaluationResultV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum BalancedAccuracyV1 {
    Available {
        sensitivity_metric_id: String,
        specificity_metric_id: String,
        point_estimate: f64,
    },
    Unavailable {
        sensitivity_metric_id: String,
        specificity_metric_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum OutcomeReasonV1 {
    HoldoutKnownOverlap { record_id: String },
    HoldoutUnknownSeparation { record_id: String },
    DeclaredCriticalFalsification { record_id: String },
    EmptyView,
    EligibleRecordMinimumNotMet { actual: u64, minimum: u64 },
    IndependentFamilyMinimumNotMet { actual: u64, minimum: u64 },
    RequiredStratumIndeterminate { stratum_id: String },
    ReferenceUncertaintyUnavailable { record_id: String },
    RequiredRuleUnavailable { rule_id: String },
    RequiredRuleFalse { rule_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MechanismResultV1 {
    pub endpoint_id: String,
    pub stratum_id: String,
    pub eligible_record_ids: Vec<String>,
    pub eligible_family_ids: Vec<String>,
    pub support_record_ids: Vec<String>,
    pub critical_contradiction_record_ids: Vec<String>,
    pub declared_critical_falsification_record_ids: Vec<String>,
    pub not_assessed_or_other_record_ids: Vec<String>,
    pub eligible_count: u64,
    pub independent_family_count: u64,
    pub support_count: u64,
    pub critical_contradiction_count: u64,
    pub declared_critical_falsification_count: u64,
    pub not_assessed_or_other_count: u64,
    pub support_fraction: MetricValueV1,
    pub contradiction_fraction: MetricValueV1,
    pub not_assessed_fraction: MetricValueV1,
    pub rule_evaluations: Vec<RuleEvaluationV1>,
    pub outcome_reasons: Vec<OutcomeReasonV1>,
    pub limitations: Vec<String>,
    pub outcome: ValidationOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthResultV1 {
    pub endpoint_id: String,
    pub stratum_id: String,
    pub eligible_record_ids: Vec<String>,
    pub eligible_family_ids: Vec<String>,
    pub tp_record_ids: Vec<String>,
    pub tn_record_ids: Vec<String>,
    pub fp_record_ids: Vec<String>,
    pub fn_record_ids: Vec<String>,
    pub indeterminate_record_ids: Vec<String>,
    pub data_quality_insufficient_record_ids: Vec<String>,
    pub eligible_count: u64,
    pub independent_family_count: u64,
    pub tp: u64,
    pub tn: u64,
    pub fp: u64,
    pub r#fn: u64,
    pub indeterminate: u64,
    pub data_quality_insufficient: u64,
    pub evaluable: u64,
    pub coverage: MetricValueV1,
    pub indeterminate_rate: MetricValueV1,
    pub data_quality_insufficient_rate: MetricValueV1,
    pub sensitivity: MetricValueV1,
    pub specificity: MetricValueV1,
    pub false_positive_rate: MetricValueV1,
    pub false_negative_rate: MetricValueV1,
    pub balanced_accuracy: BalancedAccuracyV1,
    pub rule_evaluations: Vec<RuleEvaluationV1>,
    pub outcome_reasons: Vec<OutcomeReasonV1>,
    pub limitations: Vec<String>,
    pub outcome: ValidationOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExclusionRowV1 {
    pub endpoint_id: String,
    pub stratum_id: String,
    pub record_id: String,
    pub primary_reason: ExclusionReasonV1,
    pub secondary_reasons: Vec<ExclusionReasonV1>,
    pub assessed_source_key: Option<ScientificSourceKeyV1>,
    pub reference_endpoint_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceReferenceV1 {
    KnownArtifact {
        kind: ArtifactKind,
        schema: u32,
        artifact_id: ArtifactId,
        semantic_sha256: String,
        source_file_sha256: String,
    },
    LegacyArtifact {
        kind: ArtifactKind,
        schema: u32,
        legacy_fingerprint: String,
        source_file_sha256: String,
        reason: LegacyLineageReasonV1,
    },
    LineageCatalog {
        schema: u32,
        source_file_sha256: String,
    },
    ReferenceAuthority {
        reference_source_id: String,
        source_file_sha256: String,
        origin: EvidenceOriginV1,
    },
    ApprovalTrustStore {
        trust_store_id: String,
        source_file_sha256: String,
    },
    OwnerApproval {
        approval_record_id: String,
        source_file_sha256: String,
        registry_record_sha256: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationProvenanceV1 {
    pub software_version: String,
    pub git_commit: Option<String>,
    pub protocol_sha256: String,
    pub dataset_source: DatasetSourceReferenceV1,
    pub consumed_sources: Vec<SourceReferenceV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationWarningCodeV1 {
    LegacySourceExcluded,
    ReferenceUncertaintyUnavailable,
    DeclaredSourceMissing,
    PhysicalScopeLimitation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationWarningV1 {
    pub code: ValidationWarningCodeV1,
    pub related_id: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MhiValidationReportV1 {
    pub schema_version: u32,
    pub artifact_kind: String,
    pub report_id: String,
    pub protocol: ProtocolAuthorityV1,
    pub dataset: DatasetAuthorityV1,
    pub approval: Option<ApprovalAuthorityV1>,
    pub compatibility: Vec<CompatibilityRowV1>,
    pub record_accounting: Vec<RecordAccountingRowV1>,
    pub cohorts: Vec<CohortRowV1>,
    pub leakage_assessment: Vec<LeakageRowV1>,
    pub mechanism_results: Vec<MechanismResultV1>,
    pub health_results: Vec<HealthResultV1>,
    pub exclusions: Vec<ExclusionRowV1>,
    pub release_claims: Vec<ReleaseClaimResultV1>,
    pub overall_status: ValidationOutcomeV1,
    pub lineage: crate::domain::ArtifactLineageState,
    pub provenance: ValidationProvenanceV1,
    pub warnings: Vec<ValidationWarningV1>,
}

impl MhiValidationReportV1 {
    /// Computes the frozen report identity from authorities, not from derived
    /// result fields.  Altering an assessment must therefore be caught by the
    /// structural/replay comparison rather than being able to mint a new ID.
    pub fn computed_report_id(&self) -> Result<String, ArtifactError> {
        let preimage = serde_json::json!({
            "identity_domain": "mhi_validation_report_id_v1",
            "protocol_sha256": self.protocol.source_file_sha256,
            "dataset_source": self.dataset.source,
            "consumed_sources": self.provenance.consumed_sources,
        });
        let bytes = serde_jcs::to_vec(&preimage).map_err(|error| ArtifactError::Validation {
            message: format!("MHI validation report canonicalization failed: {error}"),
        })?;
        let mut hash = Sha256::new();
        hash.update(bytes);
        Ok(format!("sha256:{:x}", hash.finalize()))
    }

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
        self.protocol
            .normalized_protocol
            .validate()
            .map_err(|error| ArtifactError::Validation {
                message: format!("report protocol authority is invalid: {error}"),
            })?;
        if self.protocol.schema_version != 1
            || self.protocol.protocol_id != self.protocol.normalized_protocol.protocol_id
            || self.protocol.registration != self.protocol.normalized_protocol.registration
            || self.protocol.physical_approval_authority
                != self
                    .protocol
                    .normalized_protocol
                    .physical_approval_authority
        {
            return invalid("report protocol authority does not match its normalized protocol");
        }
        sha(
            "report protocol source_file_sha256",
            &self.protocol.source_file_sha256,
        )?;
        if self.dataset.schema_version != 1
            || self.dataset.dataset_id.is_empty()
            || self.dataset.protocol_sha256 != self.protocol.source_file_sha256
        {
            return invalid("report dataset authority does not bind the protocol authority");
        }
        sha(
            "report dataset protocol_sha256",
            &self.dataset.protocol_sha256,
        )?;
        sha(
            "report dataset cohort_semantic_sha256",
            &self.dataset.cohort_semantic_sha256,
        )?;
        validate_dataset_source(&self.dataset.source, &self.dataset.dataset_id)?;
        if self.provenance.protocol_sha256 != self.protocol.source_file_sha256
            || self.provenance.dataset_source != self.dataset.source
        {
            return invalid("report provenance does not bind protocol and dataset authorities");
        }
        let physical_requested =
            self.protocol
                .normalized_protocol
                .release_scope
                .iter()
                .any(|claim| {
                    claim.requested_level
                        == crate::validation_config::RequestedValidationLevelV1::Physical
                });
        match (&self.approval, physical_requested) {
            (None, false) => {}
            (Some(approval), true)
                if approval.approval_purpose == "pre_scoring_physical_validation_cohort_lock"
                    && approval.binding_status == "verified"
                    && approval.owner_signature_verified
                    && approval.registry_signature_verified =>
            {
                sha(
                    "approval source_file_sha256",
                    &approval.approval_source_file_sha256,
                )?;
                sha("approval trust_store_sha256", &approval.trust_store_sha256)?;
                sha(
                    "approval owner authority document SHA-256",
                    &approval.owner_authority_document.document_sha256,
                )?;
                sha(
                    "approval registry document SHA-256",
                    &approval.registry_record.document_sha256,
                )?;
            }
            _ => return invalid("report physical-approval authority has an invalid shape"),
        }
        canonical_source_references(&self.provenance.consumed_sources)?;
        canonical_rows(
            &self.compatibility,
            |row| {
                (
                    row.source_role,
                    row.record_id.clone().unwrap_or_default(),
                    row.relative_path.clone(),
                )
            },
            "compatibility rows",
        )?;
        canonical_rows(
            &self.record_accounting,
            |row| {
                (
                    row.endpoint_id.clone(),
                    row.stratum_id.clone(),
                    row.record_id.clone(),
                )
            },
            "record accounting rows",
        )?;
        canonical_rows(
            &self.cohorts,
            |row| (row.endpoint_id.clone(), view_sort_key(&row.stratum_id)),
            "cohort rows",
        )?;
        canonical_rows(
            &self.leakage_assessment,
            |row| {
                (
                    row.endpoint_id.clone(),
                    row.stratum_id.clone(),
                    row.record_id.clone(),
                )
            },
            "leakage rows",
        )?;
        canonical_rows(
            &self.mechanism_results,
            |row| (row.endpoint_id.clone(), view_sort_key(&row.stratum_id)),
            "mechanism results",
        )?;
        canonical_rows(
            &self.health_results,
            |row| (row.endpoint_id.clone(), view_sort_key(&row.stratum_id)),
            "health results",
        )?;
        canonical_rows(
            &self.release_claims,
            |claim| claim.claim_id.clone(),
            "release claims",
        )?;
        canonical_rows(
            &self.warnings,
            |warning| {
                (
                    warning.code.clone(),
                    warning.related_id.clone(),
                    warning.detail.clone(),
                )
            },
            "warnings",
        )?;
        for cohort in &self.cohorts {
            validate_cohort_row(cohort)?;
        }
        validate_accounting_projection(self)?;
        validate_exclusion_projection(self)?;
        for result in &self.mechanism_results {
            validate_mechanism_result(result)?;
        }
        for result in &self.health_results {
            validate_health_result(result)?;
        }
        for claim in &self.release_claims {
            valid_id("claim_id", &claim.claim_id)?;
            if claim.statement.is_empty() || !is_sorted_unique(&claim.supporting_endpoint_ids) {
                return invalid(
                    "release claims must have nonempty statements and canonical endpoints",
                );
            }
        }
        if self.computed_report_id()? != self.report_id {
            return invalid("report_id does not match the frozen authority preimage");
        }
        validate_serialized_finite(self)
    }
}

fn validate_dataset_source(
    source: &DatasetSourceReferenceV1,
    dataset_id: &str,
) -> Result<(), ArtifactError> {
    match source {
        DatasetSourceReferenceV1::Known {
            dataset_id: actual,
            schema,
            artifact_id,
            semantic_sha256,
            source_file_sha256,
        } => {
            if actual != dataset_id
                || *schema != 1
                || artifact_id.0 != format!("sha256:{semantic_sha256}")
            {
                return invalid("known report dataset source does not bind its identity");
            }
            sha("dataset semantic_sha256", semantic_sha256)?;
            sha("dataset source_file_sha256", source_file_sha256)
        }
        DatasetSourceReferenceV1::LegacyUnknown {
            dataset_id: actual,
            schema,
            legacy_fingerprint,
            source_file_sha256,
            ..
        } => {
            if actual != dataset_id || *schema != 1 {
                return invalid("legacy report dataset source has an invalid identity");
            }
            sha("dataset legacy_fingerprint", legacy_fingerprint)?;
            sha("dataset source_file_sha256", source_file_sha256)
        }
    }
}

fn canonical_source_references(values: &[SourceReferenceV1]) -> Result<(), ArtifactError> {
    let keys = values.iter().map(source_reference_key).collect::<Vec<_>>();
    if !strictly_sorted(&keys) {
        return invalid("consumed source references must be canonical and unique");
    }
    Ok(())
}

fn source_reference_key(value: &SourceReferenceV1) -> (u8, String, String, String) {
    match value {
        SourceReferenceV1::KnownArtifact {
            kind,
            artifact_id,
            source_file_sha256,
            ..
        } => (
            0,
            kind.as_str().into(),
            artifact_id.0.clone(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::LegacyArtifact {
            kind,
            source_file_sha256,
            legacy_fingerprint,
            ..
        } => (
            1,
            kind.as_str().into(),
            legacy_fingerprint.clone(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::LineageCatalog {
            source_file_sha256, ..
        } => (
            2,
            "lineage_catalog".into(),
            String::new(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::ReferenceAuthority {
            reference_source_id,
            source_file_sha256,
            ..
        } => (
            3,
            "reference_authority".into(),
            reference_source_id.clone(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::ApprovalTrustStore {
            trust_store_id,
            source_file_sha256,
        } => (
            4,
            "approval_trust_store".into(),
            trust_store_id.clone(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::OwnerApproval {
            approval_record_id,
            source_file_sha256,
            ..
        } => (
            5,
            "owner_approval".into(),
            approval_record_id.clone(),
            source_file_sha256.clone(),
        ),
    }
}

fn view_sort_key(value: &str) -> (u8, String) {
    (u8::from(value != "overall"), value.into())
}

fn canonical_rows<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> K,
    name: &str,
) -> Result<(), ArtifactError> {
    let keys = values.iter().map(key).collect::<Vec<_>>();
    if !strictly_sorted(&keys) {
        return invalid(format!("{name} must be canonical and unique"));
    }
    Ok(())
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn is_sorted_unique(values: &[String]) -> bool {
    strictly_sorted(values)
}

fn validate_cohort_row(row: &CohortRowV1) -> Result<(), ArtifactError> {
    for set in [
        &row.declared_record_ids,
        &row.eligible_record_ids,
        &row.excluded_record_ids,
        &row.not_applicable_record_ids,
        &row.development_record_ids,
        &row.validation_record_ids,
        &row.holdout_record_ids,
    ] {
        if !is_sorted_unique(set) {
            return invalid("cohort record ID sets must be canonical and unique");
        }
    }
    if !disjoint_union(
        &row.declared_record_ids,
        &row.eligible_record_ids,
        &row.excluded_record_ids,
    ) || row.declared_count != row.declared_record_ids.len() as u64
        || row.eligible_count != row.eligible_record_ids.len() as u64
        || row.excluded_count != row.excluded_record_ids.len() as u64
        || row.not_applicable_count != row.not_applicable_record_ids.len() as u64
    {
        return invalid("cohort sets and counts do not reconstruct");
    }
    metric_matches(
        &row.exclusion_rate,
        row.excluded_count,
        row.declared_count,
        "denominator_zero",
    )
}

fn disjoint_union(all: &[String], left: &[String], right: &[String]) -> bool {
    let mut union = left.iter().chain(right).cloned().collect::<Vec<_>>();
    union.sort();
    union.dedup();
    union == all && left.iter().all(|id| !right.contains(id))
}

fn metric_matches(
    metric: &MetricValueV1,
    numerator: u64,
    denominator: u64,
    unavailable: &str,
) -> Result<(), ArtifactError> {
    let expected = crate::mhi_validation::statistics::wilson_95_checked(numerator, denominator)
        .map_err(|message| ArtifactError::Validation {
            message: message.into(),
        })?;
    match (&expected, metric) {
        (
            MetricValueV1::Unavailable {
                numerator: expected_numerator,
                denominator: expected_denominator,
                ..
            },
            MetricValueV1::Unavailable {
                numerator: actual_numerator,
                denominator: actual_denominator,
                reason: actual,
            },
        ) if expected_numerator == actual_numerator
            && expected_denominator == actual_denominator
            && actual == unavailable =>
        {
            Ok(())
        }
        _ if &expected == metric => Ok(()),
        _ => invalid("metric does not reconstruct from numerator and denominator"),
    }
}

fn validate_accounting_projection(report: &MhiValidationReportV1) -> Result<(), ArtifactError> {
    for cohort in &report.cohorts {
        let rows = report
            .record_accounting
            .iter()
            .filter(|row| {
                row.endpoint_id == cohort.endpoint_id && row.stratum_id == cohort.stratum_id
            })
            .collect::<Vec<_>>();
        let declared = rows
            .iter()
            .filter(|row| row.decision != RecordDecisionV1::NotApplicable)
            .map(|row| row.record_id.clone())
            .collect::<Vec<_>>();
        let eligible = rows
            .iter()
            .filter(|row| row.decision == RecordDecisionV1::Eligible)
            .map(|row| row.record_id.clone())
            .collect::<Vec<_>>();
        let excluded = rows
            .iter()
            .filter(|row| row.decision == RecordDecisionV1::Excluded)
            .map(|row| row.record_id.clone())
            .collect::<Vec<_>>();
        let not_applicable = rows
            .iter()
            .filter(|row| row.decision == RecordDecisionV1::NotApplicable)
            .map(|row| row.record_id.clone())
            .collect::<Vec<_>>();
        if declared != cohort.declared_record_ids
            || eligible != cohort.eligible_record_ids
            || excluded != cohort.excluded_record_ids
            || not_applicable != cohort.not_applicable_record_ids
        {
            return invalid("record accounting does not project to cohort membership");
        }
        for row in rows {
            match row.decision {
                RecordDecisionV1::Eligible | RecordDecisionV1::NotApplicable
                    if row.primary_reason.is_some() || !row.secondary_reasons.is_empty() =>
                {
                    return invalid("non-excluded accounting row has reasons");
                }
                RecordDecisionV1::Excluded if row.primary_reason.is_none() => {
                    return invalid("excluded accounting row lacks a primary reason");
                }
                _ => {}
            }
            if !row
                .secondary_reasons
                .windows(2)
                .all(|pair| pair[0].ordinal() < pair[1].ordinal())
            {
                return invalid("secondary exclusion reasons are not canonical");
            }
        }
    }
    for leak in &report.leakage_assessment {
        let Some(accounting) = report.record_accounting.iter().find(|row| {
            row.endpoint_id == leak.endpoint_id
                && row.stratum_id == leak.stratum_id
                && row.record_id == leak.record_id
        }) else {
            return invalid("leakage row has no accounting row");
        };
        if leak.decision != accounting.decision {
            return invalid("leakage and accounting decisions disagree");
        }
        let evaluated = leak.separation_status.is_some();
        if evaluated == leak.not_evaluated_reason.is_some() {
            return invalid("leakage row must be evaluated xor have a not-evaluated reason");
        }
        if matches!(
            leak.separation_status,
            Some(SeparationStatusV1::KnownSeparated)
        ) && (!leak.shared_artifact_ids.is_empty()
            || !leak.shared_source_sha256s.is_empty()
            || !leak.shared_experiment_ids.is_empty()
            || !leak.shared_family_ids.is_empty()
            || !leak.unknown_reasons.is_empty())
        {
            return invalid("known-separated leakage row retains overlap or uncertainty");
        }
    }
    Ok(())
}

fn validate_exclusion_projection(report: &MhiValidationReportV1) -> Result<(), ArtifactError> {
    let expected = report
        .record_accounting
        .iter()
        .filter_map(|row| {
            Some(ExclusionRowV1 {
                endpoint_id: row.endpoint_id.clone(),
                stratum_id: row.stratum_id.clone(),
                record_id: row.record_id.clone(),
                primary_reason: row.primary_reason?,
                secondary_reasons: row.secondary_reasons.clone(),
                assessed_source_key: row.assessed_source_key.clone(),
                reference_endpoint_id: row.reference_endpoint_id.clone(),
            })
        })
        .collect::<Vec<_>>();
    if report.exclusions != expected {
        return invalid("exclusions do not exactly project accounting rows");
    }
    Ok(())
}

fn validate_mechanism_result(row: &MechanismResultV1) -> Result<(), ArtifactError> {
    for set in [
        &row.eligible_record_ids,
        &row.eligible_family_ids,
        &row.support_record_ids,
        &row.critical_contradiction_record_ids,
        &row.declared_critical_falsification_record_ids,
        &row.not_assessed_or_other_record_ids,
    ] {
        if !is_sorted_unique(set) {
            return invalid("mechanism result sets must be canonical and unique");
        }
    }
    if !disjoint_union(
        &row.eligible_record_ids,
        &row.support_record_ids,
        &merge_ids(
            &row.critical_contradiction_record_ids,
            &row.not_assessed_or_other_record_ids,
        ),
    ) || !disjoint(
        &row.support_record_ids,
        &row.critical_contradiction_record_ids,
    ) || !disjoint(
        &row.support_record_ids,
        &row.not_assessed_or_other_record_ids,
    ) || !disjoint(
        &row.critical_contradiction_record_ids,
        &row.not_assessed_or_other_record_ids,
    ) || row.eligible_count != row.eligible_record_ids.len() as u64
        || row.independent_family_count != row.eligible_family_ids.len() as u64
        || row.support_count != row.support_record_ids.len() as u64
        || row.critical_contradiction_count != row.critical_contradiction_record_ids.len() as u64
        || row.declared_critical_falsification_count
            != row.declared_critical_falsification_record_ids.len() as u64
        || row.not_assessed_or_other_count != row.not_assessed_or_other_record_ids.len() as u64
    {
        return invalid("mechanism result counts or category union do not reconstruct");
    }
    metric_matches(
        &row.support_fraction,
        row.support_count,
        row.eligible_count,
        "denominator_zero",
    )?;
    metric_matches(
        &row.contradiction_fraction,
        row.critical_contradiction_count,
        row.eligible_count,
        "denominator_zero",
    )?;
    metric_matches(
        &row.not_assessed_fraction,
        row.not_assessed_or_other_count,
        row.eligible_count,
        "denominator_zero",
    )?;
    validate_rule_evaluations(&row.rule_evaluations)?;
    validate_outcome_reasons(&row.outcome_reasons)
}

fn validate_health_result(row: &HealthResultV1) -> Result<(), ArtifactError> {
    let categories = [
        &row.tp_record_ids,
        &row.tn_record_ids,
        &row.fp_record_ids,
        &row.fn_record_ids,
        &row.indeterminate_record_ids,
        &row.data_quality_insufficient_record_ids,
    ];
    if categories.iter().any(|set| !is_sorted_unique(set))
        || !is_sorted_unique(&row.eligible_record_ids)
        || !is_sorted_unique(&row.eligible_family_ids)
    {
        return invalid("health result sets must be canonical and unique");
    }
    let mut union = categories
        .iter()
        .flat_map(|set| set.iter().cloned())
        .collect::<Vec<_>>();
    union.sort();
    if union.windows(2).any(|pair| pair[0] == pair[1])
        || union != row.eligible_record_ids
        || row.eligible_count != row.eligible_record_ids.len() as u64
        || row.independent_family_count != row.eligible_family_ids.len() as u64
        || row.tp != row.tp_record_ids.len() as u64
        || row.tn != row.tn_record_ids.len() as u64
        || row.fp != row.fp_record_ids.len() as u64
        || row.r#fn != row.fn_record_ids.len() as u64
        || row.indeterminate != row.indeterminate_record_ids.len() as u64
        || row.data_quality_insufficient != row.data_quality_insufficient_record_ids.len() as u64
        || row.evaluable != row.tp + row.tn + row.fp + row.r#fn
    {
        return invalid("health result counts or six-way category union do not reconstruct");
    }
    metric_matches(
        &row.coverage,
        row.evaluable,
        row.eligible_count,
        "denominator_zero",
    )?;
    metric_matches(
        &row.indeterminate_rate,
        row.indeterminate,
        row.eligible_count,
        "denominator_zero",
    )?;
    metric_matches(
        &row.data_quality_insufficient_rate,
        row.data_quality_insufficient,
        row.eligible_count,
        "denominator_zero",
    )?;
    metric_matches(
        &row.sensitivity,
        row.tp,
        row.tp + row.r#fn,
        "positive_class_denominator_zero",
    )?;
    metric_matches(
        &row.specificity,
        row.tn,
        row.tn + row.fp,
        "negative_class_denominator_zero",
    )?;
    metric_matches(
        &row.false_positive_rate,
        row.fp,
        row.fp + row.tn,
        "negative_class_denominator_zero",
    )?;
    metric_matches(
        &row.false_negative_rate,
        row.r#fn,
        row.r#fn + row.tp,
        "positive_class_denominator_zero",
    )?;
    match (&row.balanced_accuracy, &row.sensitivity, &row.specificity) {
        (
            BalancedAccuracyV1::Available { point_estimate, .. },
            MetricValueV1::Available {
                point_estimate: sensitivity,
                ..
            },
            MetricValueV1::Available {
                point_estimate: specificity,
                ..
            },
        ) if *point_estimate == (*sensitivity + *specificity) / 2.0 => {}
        (BalancedAccuracyV1::Unavailable { reason, .. }, _, _)
            if reason == "sensitivity_or_specificity_unavailable"
                && (!matches!(row.sensitivity, MetricValueV1::Available { .. })
                    || !matches!(row.specificity, MetricValueV1::Available { .. })) => {}
        _ => {
            return invalid(
                "balanced accuracy does not reconstruct from sensitivity and specificity",
            );
        }
    }
    validate_rule_evaluations(&row.rule_evaluations)?;
    validate_outcome_reasons(&row.outcome_reasons)
}

fn validate_rule_evaluations(values: &[RuleEvaluationV1]) -> Result<(), ArtifactError> {
    for value in values {
        match (&value.rule, &value.actual) {
            (AcceptanceRuleV1::Count { .. }, RuleActualV1::Count { .. })
            | (
                AcceptanceRuleV1::Rate {
                    metric: crate::validation_config::RateMetricV1::BalancedAccuracy,
                    ..
                },
                RuleActualV1::BalancedAccuracy { .. },
            ) => {}
            (
                AcceptanceRuleV1::Rate {
                    metric: crate::validation_config::RateMetricV1::BalancedAccuracy,
                    ..
                },
                _,
            ) => return invalid("balanced-accuracy rule has an invalid actual shape"),
            (AcceptanceRuleV1::Rate { .. }, RuleActualV1::BinomialRate { .. }) => {}
            _ => return invalid("rule has an invalid actual shape"),
        }
    }
    Ok(())
}

fn validate_outcome_reasons(values: &[OutcomeReasonV1]) -> Result<(), ArtifactError> {
    let keys = values.iter().map(outcome_reason_key).collect::<Vec<_>>();
    if !strictly_sorted(&keys) {
        return invalid("outcome reasons must be canonically ordered and unique");
    }
    Ok(())
}

fn outcome_reason_key(value: &OutcomeReasonV1) -> (u8, String) {
    let ordinal = match value {
        OutcomeReasonV1::HoldoutKnownOverlap { .. } => 2,
        OutcomeReasonV1::HoldoutUnknownSeparation { .. } => 3,
        OutcomeReasonV1::DeclaredCriticalFalsification { .. } => 4,
        OutcomeReasonV1::EmptyView
        | OutcomeReasonV1::EligibleRecordMinimumNotMet { .. }
        | OutcomeReasonV1::IndependentFamilyMinimumNotMet { .. }
        | OutcomeReasonV1::RequiredStratumIndeterminate { .. } => 5,
        OutcomeReasonV1::ReferenceUncertaintyUnavailable { .. } => 6,
        OutcomeReasonV1::RequiredRuleUnavailable { .. } => 7,
        OutcomeReasonV1::RequiredRuleFalse { .. } => 8,
    };
    (ordinal, format!("{value:?}"))
}

fn merge_ids(left: &[String], right: &[String]) -> Vec<String> {
    let mut values = left.iter().chain(right).cloned().collect::<Vec<_>>();
    values.sort();
    values
}
fn disjoint(left: &[String], right: &[String]) -> bool {
    left.iter().all(|value| !right.contains(value))
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
