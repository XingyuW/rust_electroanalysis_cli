//! Typed deterministic Phase-D public documents.

use crate::{
    domain::{AnalysisProvenance, ArtifactKind, ArtifactLineageState},
    report_config::{FigureId, TableId},
    reporting::{
        AvailabilityReason,
        claims::{
            REQUIRED_DISCLAIMER, causal_status_text, evidence_state_text, health_status_text,
            mechanism_level_text,
        },
        lineage::{
            self, AcquisitionFamilyPresentationV1, LineagePresentationV1, ProvenancePresentationV1,
            PublicLineageRootV1,
        },
        projection::PublicReportProjection,
        reader::{CompatibilityOutcome, CompatibilityStatus},
        tables::format_public_f64,
    },
    results::{
        CausalStatus, FeatureComparability, HealthDimension, HealthDomain, HealthEvidenceState,
        HealthInterpretationCategory, HealthWarning, OverallHealthStatus,
    },
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputFlagV1 {
    Mechanism,
    Health,
    Eis,
    Transient,
    Calibration,
    CalibrationObservations,
    Signal,
    Estimation,
    Model,
    LineageCatalog,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactInputFlagV1 {
    Mechanism,
    Health,
    Eis,
    Transient,
    Calibration,
    CalibrationObservations,
    Signal,
    Estimation,
    Model,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityV1 {
    Available,
    AvailableWithWarnings,
    NotProvided,
    NotSelected,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogValidationV1 {
    Validated,
    NotApplicable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatusV1 {
    Compatible,
    LegacyUnknown,
    NotProvided,
    NotApplicable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionalSourceKindV1 {
    Eis,
    Transient,
    Calibration,
    Signal,
    Estimation,
    Model,
    LineageCatalog,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderStatusV1 {
    Written,
    Unavailable,
    NotSelected,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedOutputKindV1 {
    Summary,
    Markdown,
    Table,
    Figure,
    Manifest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderFormatV1 {
    Json,
    Markdown,
    Csv,
    Svg,
    Png,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningCodeV1 {
    SourceWarning,
    BaselineComparableWithWarnings,
    LegacyInput,
    LegacyLineage,
    CatalogNotSupplied,
    OutputUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthWarningV1 {
    MissingBaseline,
    InsufficientBaselineRecords,
    BaselineVarianceUnavailable,
    FeatureNoncomparable,
    MissingSignalArtifact,
    MissingTransientArtifact,
    MissingCalibrationArtifact,
    MissingEisArtifact,
    MissingMechanismArtifact,
    ArtifactSchemaMismatch,
    ArtifactConfigurationMismatch,
    EnvironmentalMismatch,
    InsufficientEvidenceDomains,
    ContradictoryEvidence,
    RuleConditionUnavailable,
    SemanticRoleUnavailable,
    AssessmentBasedOnWarningBearingFits,
    InvalidRule,
    NonFiniteArtifact,
    MixedAnalyteContext,
    MixedSampleMatrixContext,
    MixedSensorDesignContext,
    MixedSensorTypeContext,
    MixedTemperatureContext,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "input_kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(clippy::large_enum_variant)]
pub enum PublicInputReferenceV1 {
    Artifact(PublicArtifactInputReferenceV1),
    LineageCatalog(PublicLineageCatalogInputReferenceV1),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicArtifactInputReferenceV1 {
    pub input_flag: ArtifactInputFlagV1,
    pub supplied_path_basename: Option<String>,
    pub artifact_kind: Option<ArtifactKind>,
    pub schema_version: Option<u32>,
    pub lineage: Option<LineagePresentationV1>,
    pub acquisition_families: Option<AcquisitionFamilyPresentationV1>,
    pub availability: AvailabilityV1,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicLineageCatalogInputReferenceV1 {
    pub supplied_path_basename: Option<String>,
    pub schema_version: Option<u32>,
    pub availability: AvailabilityV1,
    pub validation: CatalogValidationV1,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCompatibilityV1 {
    pub required_pair: CompatibilityStatusV1,
    pub optional: Vec<CompatibilityRecordV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityRecordV1 {
    pub input_flag: InputFlagV1,
    pub against_flag: InputFlagV1,
    pub status: CompatibilityStatusV1,
    pub mismatch_axis: Option<crate::reporting::CompatibilityAxis>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicMessageV1 {
    pub code: WarningCodeV1,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicHypothesisV1 {
    pub hypothesis_id: String,
    pub display_name: String,
    pub target_components: Vec<String>,
    pub evidence_level: crate::mechanism::promotion::HypothesisEvidenceLevel,
    pub reason_codes: Vec<crate::mechanism::promotion::PhaseBHypothesisReasonCode>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicTimescaleComparisonV1 {
    pub comparison_id: String,
    pub record_id: String,
    pub eis_timescale_id: String,
    pub transient_timescale_id: String,
    pub ratio: Option<f64>,
    pub log10_distance: Option<f64>,
    pub symmetric_relative_difference: Option<f64>,
    pub confidence_interval_overlap: Option<bool>,
    pub compatibility_probability: Option<f64>,
    pub evidence_level: crate::results::EvidenceLevel,
    pub warnings: Vec<PublicMessageV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicMechanismSectionV1 {
    pub availability: AvailabilityV1,
    pub analysis_id: String,
    pub hypotheses: Vec<PublicHypothesisV1>,
    pub comparisons: Vec<PublicTimescaleComparisonV1>,
    pub warning_messages: Vec<PublicMessageV1>,
    pub lineage: LineagePresentationV1,
    pub provenance: ProvenancePresentationV1,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicHealthDimensionV1 {
    pub dimension: HealthDimension,
    pub status: OverallHealthStatus,
    pub evidence_state: HealthEvidenceState,
    pub interpretation_category: HealthInterpretationCategory,
    pub causal_status: CausalStatus,
    pub reason_codes: Vec<crate::results::PhaseCHealthReasonCode>,
    pub source_evidence_ids: Vec<String>,
    pub source_artifact_ids: Vec<String>,
    pub excluded_evidence_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicHealthFeatureV1 {
    pub name: String,
    pub value: Option<f64>,
    pub unit: String,
    pub domain: HealthDomain,
    pub source: String,
    pub warning: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicBaselineComparisonV1 {
    pub feature: String,
    pub current_value: Option<f64>,
    pub baseline_value: Option<f64>,
    pub comparability: FeatureComparability,
    pub absolute_difference: Option<f64>,
    pub relative_difference: Option<f64>,
    pub log_ratio: Option<f64>,
    pub z_score: Option<f64>,
    pub robust_z_score: Option<f64>,
    pub empirical_percentile: Option<f64>,
    pub range_position_percent: Option<f64>,
    pub override_reason: Option<String>,
    pub baseline_sample_count: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicHealthSectionV1 {
    pub availability: AvailabilityV1,
    pub assessment_id: String,
    pub sensor_id: Option<String>,
    pub experiment_id: Option<String>,
    pub overall_status: OverallHealthStatus,
    pub dimensions: Vec<PublicHealthDimensionV1>,
    pub features: Vec<PublicHealthFeatureV1>,
    pub baseline_comparisons: Vec<PublicBaselineComparisonV1>,
    pub warning_codes: Vec<HealthWarningV1>,
    pub lineage: LineagePresentationV1,
    pub provenance: ProvenancePresentationV1,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionalSourceDetailV1 {
    pub analysis_id: Option<String>,
    pub record_count: u64,
    pub measurement_unit: Option<String>,
    pub lineage: LineagePresentationV1,
    pub provenance: ProvenancePresentationV1,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicOptionalSourceV1 {
    pub kind: OptionalSourceKindV1,
    pub availability: AvailabilityV1,
    pub compatibility: CompatibilityStatusV1,
    pub input: Option<PublicInputReferenceV1>,
    pub detail: Option<OptionalSourceDetailV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicLineageSectionV1 {
    pub catalog_supplied: bool,
    pub roots: Vec<PublicLineageRootV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicOutputStatusV1 {
    pub output_kind: GeneratedOutputKindV1,
    pub output_id: String,
    pub relative_path: Option<String>,
    pub format: Option<RenderFormatV1>,
    pub status: RenderStatusV1,
    pub reason: Option<AvailabilityReason>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicOutputIndexV1 {
    pub tables: Vec<PublicOutputStatusV1>,
    pub figures: Vec<PublicOutputStatusV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicLimitationV1 {
    pub code: WarningCodeV1,
    pub message: String,
    pub input_flag: Option<InputFlagV1>,
    pub output_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicRenderingMetadataV1 {
    pub json_schema: String,
    pub numeric_format: String,
    pub csv_newline: String,
    pub timestamp: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicSummaryV1 {
    pub schema_version: u32,
    pub output_kind: String,
    pub renderer_contract: String,
    pub route: String,
    pub input_references: Vec<PublicInputReferenceV1>,
    pub compatibility: PublicCompatibilityV1,
    pub mechanism: PublicMechanismSectionV1,
    pub sensor_health: PublicHealthSectionV1,
    pub optional_sources: Vec<PublicOptionalSourceV1>,
    pub lineage: PublicLineageSectionV1,
    pub outputs: PublicOutputIndexV1,
    pub limitations: Vec<PublicLimitationV1>,
    pub rendering: PublicRenderingMetadataV1,
}

pub(crate) fn write_public_summary_json(
    root: &Path,
    projection: &PublicReportProjection,
    table_ids: &[TableId],
    figure_ids: &[FigureId],
) -> Result<String, crate::reporting::PublicReportError> {
    let path = root.join("public_summary.schema1.json");
    let summary = public_summary(projection, table_ids, figure_ids);
    let mut bytes = serde_json::to_vec_pretty(&summary).map_err(|source| {
        crate::reporting::PublicReportError::Serialization {
            path: path.clone(),
            source,
        }
    })?;
    bytes.push(b'\n');
    fs::write(&path, bytes).map_err(|source| crate::reporting::PublicReportError::Write {
        path: path.clone(),
        source,
    })?;
    Ok("public_summary.schema1.json".into())
}

fn public_summary(
    p: &PublicReportProjection,
    tables: &[TableId],
    figures: &[FigureId],
) -> PublicSummaryV1 {
    PublicSummaryV1 {
        schema_version: 1,
        output_kind: "phase_d_public_scientific_output".into(),
        renderer_contract: "mhi_v1_phase_d_public_output_v1".into(),
        route: "electroanalysis report render".into(),
        input_references: input_references(p, tables, figures),
        compatibility: PublicCompatibilityV1 {
            required_pair: compatibility(p.required_compatibility),
            optional: p
                .optional_compatibility
                .iter()
                .map(|(input, against, outcome)| CompatibilityRecordV1 {
                    input_flag: flag(input),
                    against_flag: flag(against),
                    status: compatibility(*outcome),
                    mismatch_axis: outcome.mismatch_axis,
                })
                .collect(),
        },
        mechanism: mechanism_section(p),
        sensor_health: health_section(p),
        optional_sources: optional_sources(p, tables, figures),
        lineage: lineage_section(p),
        outputs: output_index(p, tables, figures),
        limitations: limitations(p, figures),
        rendering: PublicRenderingMetadataV1 {
            json_schema: "public_summary.schema1".into(),
            numeric_format: "rust_display_normalized_negative_zero_v1".into(),
            csv_newline: "LF".into(),
            timestamp: None,
        },
    }
}

fn mechanism_section(p: &PublicReportProjection) -> PublicMechanismSectionV1 {
    let mut hypotheses = p
        .mechanism
        .hypothesis_assessments
        .iter()
        .map(|row| PublicHypothesisV1 {
            hypothesis_id: row.definition.hypothesis_id.clone(),
            display_name: row.definition.display_name.clone(),
            target_components: row.definition.target_components.clone(),
            evidence_level: row.current.evidence_level.clone(),
            reason_codes: row.current.reason_codes.clone(),
        })
        .collect::<Vec<_>>();
    hypotheses.sort_by(|a, b| a.hypothesis_id.cmp(&b.hypothesis_id));
    let mut comparisons = p
        .mechanism
        .comparisons
        .iter()
        .map(|row| PublicTimescaleComparisonV1 {
            comparison_id: row.comparison_id.clone(),
            record_id: row.record_id.clone(),
            eis_timescale_id: row.eis_timescale_id.clone(),
            transient_timescale_id: row.transient_timescale_id.clone(),
            ratio: row.ratio,
            log10_distance: row.log10_distance,
            symmetric_relative_difference: row.symmetric_relative_difference,
            confidence_interval_overlap: row.confidence_interval_overlap,
            compatibility_probability: row.compatibility_probability,
            evidence_level: row.evidence_level.clone(),
            warnings: mechanism_messages(&row.warnings),
        })
        .collect::<Vec<_>>();
    comparisons.sort_by(|a, b| a.comparison_id.cmp(&b.comparison_id));
    PublicMechanismSectionV1 {
        availability: if p.mechanism_is_legacy() {
            AvailabilityV1::Unavailable
        } else if !p.mechanism.warnings.is_empty() {
            AvailabilityV1::AvailableWithWarnings
        } else {
            AvailabilityV1::Available
        },
        analysis_id: p.mechanism.analysis_id.clone(),
        hypotheses,
        comparisons,
        warning_messages: mechanism_messages(&p.mechanism.warnings),
        lineage: lineage::project_lineage(&p.mechanism.lineage),
        provenance: lineage::project_provenance(
            p.mechanism
                .provenance
                .as_ref()
                .expect("Phase-D reader requires mechanism provenance"),
        ),
    }
}
fn health_section(p: &PublicReportProjection) -> PublicHealthSectionV1 {
    let dimensions = p
        .health
        .phase_c
        .as_ref()
        .map(|phase| {
            phase
                .dimension_assessments
                .iter()
                .map(|row| PublicHealthDimensionV1 {
                    dimension: row.dimension,
                    status: row.status,
                    evidence_state: row.evidence_state,
                    interpretation_category: row.interpretation_category,
                    causal_status: row.causal_status,
                    reason_codes: row.reason_codes.clone(),
                    source_evidence_ids: row
                        .source_evidence_ids
                        .iter()
                        .map(|id| id.0.clone())
                        .collect(),
                    source_artifact_ids: row
                        .source_artifact_ids
                        .iter()
                        .map(|id| id.0.clone())
                        .collect(),
                    excluded_evidence_ids: row
                        .excluded_evidence_ids
                        .iter()
                        .map(|id| id.0.clone())
                        .collect(),
                })
                .collect()
        })
        .unwrap_or_default();
    let mut features = p
        .health
        .features
        .iter()
        .map(|row| PublicHealthFeatureV1 {
            name: row.name.clone(),
            value: row.value,
            unit: row.unit.clone(),
            domain: row.domain,
            source: row.source.clone(),
            warning: row.warning.clone(),
        })
        .collect::<Vec<_>>();
    features.sort_by(|a, b| a.name.cmp(&b.name).then(a.unit.cmp(&b.unit)));
    let mut baseline_comparisons = p
        .health
        .baseline_comparison
        .iter()
        .map(|row| PublicBaselineComparisonV1 {
            feature: row.feature.clone(),
            current_value: row.current_value,
            baseline_value: row.baseline_value,
            comparability: row.comparability,
            absolute_difference: row.absolute_difference,
            relative_difference: row.relative_difference,
            log_ratio: row.log_ratio,
            z_score: row.z_score,
            robust_z_score: row.robust_z_score,
            empirical_percentile: row.empirical_percentile,
            range_position_percent: row.range_position_percent,
            override_reason: row.override_reason.clone(),
            baseline_sample_count: row.baseline_sample_count as u64,
        })
        .collect::<Vec<_>>();
    baseline_comparisons.sort_by(|a, b| a.feature.cmp(&b.feature));
    PublicHealthSectionV1 {
        availability: if p.health_is_legacy() {
            AvailabilityV1::Unavailable
        } else if !p.health.warnings.is_empty() {
            AvailabilityV1::AvailableWithWarnings
        } else {
            AvailabilityV1::Available
        },
        assessment_id: p.health.assessment_id.clone(),
        sensor_id: p.health.sensor_id.clone(),
        experiment_id: p.health.experiment_id.clone(),
        overall_status: p.health.overall_status,
        dimensions,
        features,
        baseline_comparisons,
        warning_codes: p
            .health
            .warnings
            .iter()
            .map(HealthWarningV1::from)
            .collect(),
        lineage: lineage::project_lineage(&p.health.lineage),
        provenance: lineage::project_provenance(&p.health.provenance),
    }
}
fn input_references(
    p: &PublicReportProjection,
    tables: &[TableId],
    figures: &[FigureId],
) -> Vec<PublicInputReferenceV1> {
    vec![
        artifact_reference(
            ArtifactInputFlagV1::Mechanism,
            Some(&p.input_paths.mechanism),
            Some(ArtifactKind::MechanismAnalysis),
            Some(p.mechanism.schema_version),
            Some(&p.mechanism.lineage),
            source_availability(&p.mechanism.lineage, !p.mechanism.warnings.is_empty()),
        ),
        artifact_reference(
            ArtifactInputFlagV1::Health,
            Some(&p.input_paths.health),
            Some(ArtifactKind::HealthAssessment),
            Some(p.health.schema_version),
            Some(&p.health.lineage),
            source_availability(&p.health.lineage, !p.health.warnings.is_empty()),
        ),
        optional_reference(
            ArtifactInputFlagV1::Eis,
            p.input_paths.eis.as_deref(),
            p.eis
                .as_ref()
                .map(|a| (ArtifactKind::EisFit, a.schema_version, &a.lineage)),
            optional_input_availability(
                p,
                ArtifactInputFlagV1::Eis,
                p.eis
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
        ),
        optional_reference(
            ArtifactInputFlagV1::Transient,
            p.input_paths.transient.as_deref(),
            p.transient.as_ref().map(|a| {
                (
                    ArtifactKind::TransientAnalysis,
                    a.schema_version,
                    &a.lineage,
                )
            }),
            optional_input_availability(
                p,
                ArtifactInputFlagV1::Transient,
                p.transient.map(|value| {
                    (
                        &value.lineage,
                        value.events.iter().any(|event| {
                            !event.warnings.is_empty()
                                || event
                                    .candidate_fits
                                    .iter()
                                    .any(|fit| !fit.warnings.is_empty())
                        }),
                    )
                }),
                tables,
                figures,
            ),
        ),
        optional_reference(
            ArtifactInputFlagV1::Calibration,
            p.input_paths.calibration.as_deref(),
            p.calibration.as_ref().map(|a| {
                (
                    ArtifactKind::CalibrationAnalysis,
                    a.schema_version,
                    &a.lineage,
                )
            }),
            optional_input_availability(
                p,
                ArtifactInputFlagV1::Calibration,
                p.calibration
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
        ),
        optional_reference(
            ArtifactInputFlagV1::CalibrationObservations,
            p.input_paths.calibration_observations.as_deref(),
            p.calibration_observations.as_ref().map(|a| {
                (
                    ArtifactKind::CalibrationObservations,
                    a.schema_version,
                    &a.lineage,
                )
            }),
            optional_input_availability(
                p,
                ArtifactInputFlagV1::CalibrationObservations,
                p.calibration_observations
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
        ),
        optional_reference(
            ArtifactInputFlagV1::Signal,
            p.input_paths.signal.as_deref(),
            p.signal
                .as_ref()
                .map(|a| (ArtifactKind::SignalAnalysis, a.schema_version, &a.lineage)),
            optional_input_availability(
                p,
                ArtifactInputFlagV1::Signal,
                p.signal
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
        ),
        optional_reference(
            ArtifactInputFlagV1::Estimation,
            p.input_paths.estimation.as_deref(),
            p.estimation
                .as_ref()
                .map(|a| (ArtifactKind::StateEstimation, a.schema_version, &a.lineage)),
            optional_input_availability(
                p,
                ArtifactInputFlagV1::Estimation,
                p.estimation
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
        ),
        optional_reference(
            ArtifactInputFlagV1::Model,
            p.input_paths.model.as_deref(),
            p.model
                .as_ref()
                .map(|a| (ArtifactKind::ModelAnalysis, a.schema_version, &a.lineage)),
            optional_input_availability(
                p,
                ArtifactInputFlagV1::Model,
                p.model.map(|value| (&value.lineage, false)),
                tables,
                figures,
            ),
        ),
        PublicInputReferenceV1::LineageCatalog(PublicLineageCatalogInputReferenceV1 {
            supplied_path_basename: p.input_paths.lineage_catalog.as_deref().map(basename),
            schema_version: p.lineage_catalog.as_ref().map(|_| 1),
            availability: catalog_availability(p, tables, figures),
            validation: if p.lineage_catalog.is_some() {
                CatalogValidationV1::Validated
            } else {
                CatalogValidationV1::NotApplicable
            },
        }),
    ]
}
fn artifact_reference(
    flag: ArtifactInputFlagV1,
    path: Option<&Path>,
    kind: Option<ArtifactKind>,
    schema: Option<u32>,
    lineage_state: Option<&ArtifactLineageState>,
    availability: AvailabilityV1,
) -> PublicInputReferenceV1 {
    let legacy = matches!(
        lineage_state,
        Some(ArtifactLineageState::LegacyUnknown { .. })
    );
    PublicInputReferenceV1::Artifact(PublicArtifactInputReferenceV1 {
        input_flag: flag,
        supplied_path_basename: path.map(basename),
        artifact_kind: kind,
        schema_version: schema,
        lineage: lineage_state.map(lineage::project_lineage),
        acquisition_families: lineage_state.map(|state| match state {
            ArtifactLineageState::Known { identity, .. } => {
                lineage::project_families(&identity.acquisition_families, false)
            }
            ArtifactLineageState::LegacyUnknown { .. } => lineage::project_families(
                &crate::domain::ArtifactAcquisitionFamilies::Unknown,
                legacy,
            ),
        }),
        availability,
    })
}
fn optional_reference(
    flag: ArtifactInputFlagV1,
    path: Option<&Path>,
    input: Option<(ArtifactKind, u32, &ArtifactLineageState)>,
    availability: AvailabilityV1,
) -> PublicInputReferenceV1 {
    match input {
        Some((kind, schema, lineage_state)) => artifact_reference(
            flag,
            path,
            Some(kind),
            Some(schema),
            Some(lineage_state),
            availability,
        ),
        None => artifact_reference(flag, None, None, None, None, AvailabilityV1::NotProvided),
    }
}

pub(crate) fn optional_input_availability(
    p: &PublicReportProjection<'_>,
    flag: ArtifactInputFlagV1,
    input: Option<(&ArtifactLineageState, bool)>,
    tables: &[TableId],
    figures: &[FigureId],
) -> AvailabilityV1 {
    let Some((lineage, has_warnings)) = input else {
        return AvailabilityV1::NotProvided;
    };
    let table_selected = match flag {
        ArtifactInputFlagV1::Model => tables.contains(&TableId::ModelConsistency),
        _ => false,
    } || tables.contains(&TableId::ArtifactLineage);
    let relevant_figures = figures.iter().filter(|figure| match flag {
        ArtifactInputFlagV1::Eis => {
            matches!(
                figure,
                FigureId::EisNyquist | FigureId::EisBode | FigureId::Lineage
            )
        }
        ArtifactInputFlagV1::Transient => {
            matches!(figure, FigureId::TransientResponse | FigureId::Lineage)
        }
        ArtifactInputFlagV1::Calibration | ArtifactInputFlagV1::CalibrationObservations => {
            matches!(figure, FigureId::CalibrationPerformance | FigureId::Lineage)
        }
        ArtifactInputFlagV1::Signal => {
            matches!(figure, FigureId::SignalDiagnostics | FigureId::Lineage)
        }
        ArtifactInputFlagV1::Estimation => {
            matches!(
                figure,
                FigureId::EstimationObservedPredicted | FigureId::Lineage
            )
        }
        ArtifactInputFlagV1::Model => {
            matches!(figure, FigureId::ModelObservedPredicted | FigureId::Lineage)
        }
        ArtifactInputFlagV1::Mechanism | ArtifactInputFlagV1::Health => true,
    });
    let mut saw_figure = false;
    let mut renderable_figure = false;
    for figure in relevant_figures {
        saw_figure = true;
        if p.figure_reason(*figure).is_none() {
            renderable_figure = true;
        }
    }
    if table_selected || renderable_figure {
        source_availability(lineage, has_warnings)
    } else if saw_figure {
        AvailabilityV1::Unavailable
    } else {
        AvailabilityV1::NotSelected
    }
}

pub(crate) fn catalog_availability(
    p: &PublicReportProjection<'_>,
    _tables: &[TableId],
    _figures: &[FigureId],
) -> AvailabilityV1 {
    if p.lineage_catalog.is_none() {
        AvailabilityV1::NotProvided
    } else {
        // The catalog is a validated input reference, not an optional
        // scientific series.  Once supplied and accepted by the canonical
        // catalog reader it is available even if no lineage output is selected.
        AvailabilityV1::Available
    }
}

pub(crate) fn source_availability(
    lineage: &ArtifactLineageState,
    has_warnings: bool,
) -> AvailabilityV1 {
    if has_warnings || matches!(lineage, ArtifactLineageState::LegacyUnknown { .. }) {
        AvailabilityV1::AvailableWithWarnings
    } else {
        AvailabilityV1::Available
    }
}

fn optional_sources(
    p: &PublicReportProjection,
    tables: &[TableId],
    figures: &[FigureId],
) -> Vec<PublicOptionalSourceV1> {
    vec![
        optional_source(
            p,
            OptionalSourceKindV1::Eis,
            ArtifactInputFlagV1::Eis,
            "--eis",
            tables,
            figures,
            p.eis.as_ref().map(|a| {
                (
                    Some(a.fit_id.clone()),
                    a.source.frequency_hz.len() as u64,
                    Some("Ohm".into()),
                    &a.lineage,
                    &a.provenance,
                )
            }),
        ),
        optional_source(
            p,
            OptionalSourceKindV1::Transient,
            ArtifactInputFlagV1::Transient,
            "--transient",
            tables,
            figures,
            p.transient.as_ref().map(|a| {
                (
                    Some(a.experiment_id.clone()),
                    a.events.len() as u64,
                    Some(a.channel_unit.clone()),
                    &a.lineage,
                    &a.provenance,
                )
            }),
        ),
        optional_source(
            p,
            OptionalSourceKindV1::Calibration,
            ArtifactInputFlagV1::Calibration,
            "--calibration",
            tables,
            figures,
            p.calibration.as_ref().map(|a| {
                (
                    Some(a.calibration_id.clone()),
                    a.validation.as_ref().map_or(0, |v| v.predictions.len()) as u64,
                    Some("V".into()),
                    &a.lineage,
                    &a.provenance,
                )
            }),
        ),
        optional_source(
            p,
            OptionalSourceKindV1::Signal,
            ArtifactInputFlagV1::Signal,
            "--signal",
            tables,
            figures,
            p.signal.as_ref().map(|a| {
                (
                    Some(a.analysis_id.clone()),
                    a.analysis_timestamps.len() as u64,
                    Some(a.unit.clone()),
                    &a.lineage,
                    &a.provenance,
                )
            }),
        ),
        optional_source(
            p,
            OptionalSourceKindV1::Estimation,
            ArtifactInputFlagV1::Estimation,
            "--estimation",
            tables,
            figures,
            p.estimation.as_ref().map(|a| {
                (
                    Some(a.analysis_id.clone()),
                    a.estimates.len() as u64,
                    Some("V".into()),
                    &a.lineage,
                    &a.provenance,
                )
            }),
        ),
        optional_source_model(p, tables, figures),
        PublicOptionalSourceV1 {
            kind: OptionalSourceKindV1::LineageCatalog,
            availability: catalog_availability(p, tables, figures),
            compatibility: CompatibilityStatusV1::NotApplicable,
            input: None,
            detail: None,
        },
    ]
}
type OptionalSourceInput<'a> = (
    Option<String>,
    u64,
    Option<String>,
    &'a ArtifactLineageState,
    &'a AnalysisProvenance,
);

fn optional_source(
    p: &PublicReportProjection,
    kind: OptionalSourceKindV1,
    input_flag: ArtifactInputFlagV1,
    flag_text: &'static str,
    tables: &[TableId],
    figures: &[FigureId],
    input: Option<OptionalSourceInput<'_>>,
) -> PublicOptionalSourceV1 {
    match input {
        Some((analysis_id, record_count, unit, state, provenance)) => {
            let availability = optional_input_availability(
                p,
                input_flag,
                Some((state, source_has_warnings(p, flag_text))),
                tables,
                figures,
            );
            PublicOptionalSourceV1 {
                kind,
                availability,
                compatibility: optional_source_compatibility(p, flag_text),
                input: Some(artifact_reference(
                    input_flag,
                    input_path(p, flag_text),
                    Some(artifact_kind(input_flag)),
                    Some(artifact_schema(p, flag_text)),
                    Some(state),
                    availability,
                )),
                detail: Some(OptionalSourceDetailV1 {
                    analysis_id,
                    record_count,
                    measurement_unit: unit,
                    lineage: lineage::project_lineage(state),
                    provenance: lineage::project_provenance(provenance),
                }),
            }
        }
        None => PublicOptionalSourceV1 {
            kind,
            availability: AvailabilityV1::NotProvided,
            compatibility: CompatibilityStatusV1::NotProvided,
            input: None,
            detail: None,
        },
    }
}
fn optional_source_compatibility(
    p: &PublicReportProjection,
    flag_text: &str,
) -> CompatibilityStatusV1 {
    let outcomes = p
        .optional_compatibility
        .iter()
        .filter(|(input, _, _)| *input == flag_text)
        .map(|(_, _, outcome)| outcome.status)
        .collect::<Vec<_>>();
    if outcomes.is_empty()
        || outcomes
            .iter()
            .all(|status| *status == CompatibilityStatus::NotProvided)
    {
        CompatibilityStatusV1::NotProvided
    } else if outcomes.contains(&CompatibilityStatus::LegacyUnknown) {
        CompatibilityStatusV1::LegacyUnknown
    } else {
        CompatibilityStatusV1::Compatible
    }
}
fn input_path<'a>(p: &'a PublicReportProjection<'_>, flag: &str) -> Option<&'a Path> {
    match flag {
        "--eis" => p.input_paths.eis.as_deref(),
        "--transient" => p.input_paths.transient.as_deref(),
        "--calibration" => p.input_paths.calibration.as_deref(),
        "--signal" => p.input_paths.signal.as_deref(),
        "--estimation" => p.input_paths.estimation.as_deref(),
        "--model" => p.input_paths.model.as_deref(),
        _ => None,
    }
}
fn artifact_kind(flag: ArtifactInputFlagV1) -> ArtifactKind {
    match flag {
        ArtifactInputFlagV1::Mechanism => ArtifactKind::MechanismAnalysis,
        ArtifactInputFlagV1::Health => ArtifactKind::HealthAssessment,
        ArtifactInputFlagV1::Eis => ArtifactKind::EisFit,
        ArtifactInputFlagV1::Transient => ArtifactKind::TransientAnalysis,
        ArtifactInputFlagV1::Calibration => ArtifactKind::CalibrationAnalysis,
        ArtifactInputFlagV1::CalibrationObservations => ArtifactKind::CalibrationObservations,
        ArtifactInputFlagV1::Signal => ArtifactKind::SignalAnalysis,
        ArtifactInputFlagV1::Estimation => ArtifactKind::StateEstimation,
        ArtifactInputFlagV1::Model => ArtifactKind::ModelAnalysis,
    }
}
fn artifact_schema(p: &PublicReportProjection<'_>, flag: &str) -> u32 {
    match flag {
        "--eis" => p.eis.expect("supplied source").schema_version,
        "--transient" => p.transient.expect("supplied source").schema_version,
        "--calibration" => p.calibration.expect("supplied source").schema_version,
        "--signal" => p.signal.expect("supplied source").schema_version,
        "--estimation" => p.estimation.expect("supplied source").schema_version,
        "--model" => p.model.expect("supplied source").schema_version,
        _ => unreachable!("fixed optional source flag"),
    }
}
fn source_has_warnings(p: &PublicReportProjection<'_>, flag: &str) -> bool {
    match flag {
        "--eis" => p.eis.is_some_and(|value| !value.warnings.is_empty()),
        "--transient" => p.transient.is_some_and(|value| {
            value.events.iter().any(|event| {
                !event.warnings.is_empty()
                    || event
                        .candidate_fits
                        .iter()
                        .any(|fit| !fit.warnings.is_empty())
            })
        }),
        "--calibration" => p
            .calibration
            .is_some_and(|value| !value.warnings.is_empty()),
        "--signal" => p.signal.is_some_and(|value| !value.warnings.is_empty()),
        "--estimation" => p.estimation.is_some_and(|value| !value.warnings.is_empty()),
        "--model" => false,
        _ => false,
    }
}
fn optional_source_model(
    p: &PublicReportProjection,
    tables: &[TableId],
    figures: &[FigureId],
) -> PublicOptionalSourceV1 {
    match &p.model {
        Some(model) => {
            let availability = optional_input_availability(
                p,
                ArtifactInputFlagV1::Model,
                Some((&model.lineage, false)),
                tables,
                figures,
            );
            PublicOptionalSourceV1 {
                kind: OptionalSourceKindV1::Model,
                availability,
                compatibility: optional_source_compatibility(p, "--model"),
                input: Some(artifact_reference(
                    ArtifactInputFlagV1::Model,
                    p.input_paths.model.as_deref(),
                    Some(ArtifactKind::ModelAnalysis),
                    Some(model.schema_version),
                    Some(&model.lineage),
                    availability,
                )),
                // ModelAnalysisReport schema 5 has no AnalysisProvenance field.
                // Omitting the optional detail is the only non-fabricating closed
                // representation; lineage remains available on `input`.
                detail: None,
            }
        }
        None => PublicOptionalSourceV1 {
            kind: OptionalSourceKindV1::Model,
            availability: AvailabilityV1::NotProvided,
            compatibility: CompatibilityStatusV1::NotProvided,
            input: None,
            detail: None,
        },
    }
}
fn lineage_section(p: &PublicReportProjection) -> PublicLineageSectionV1 {
    PublicLineageSectionV1 {
        catalog_supplied: p.lineage_catalog.is_some(),
        roots: p
            .supplied_lineages()
            .into_iter()
            .map(|(name, state)| {
                let present = match state {
                    ArtifactLineageState::Known { identity, .. } => p
                        .lineage_catalog
                        .as_ref()
                        .map(|catalog| catalog.artifacts.contains_key(&identity.artifact_id)),
                    ArtifactLineageState::LegacyUnknown { .. } => None,
                };
                PublicLineageRootV1 {
                    input_flag: flag(name),
                    lineage: lineage::project_lineage(state),
                    direct_dependencies: lineage::project_dependencies(state),
                    root_catalog_entry_present: present,
                }
            })
            .collect(),
    }
}
fn output_index(
    p: &PublicReportProjection,
    tables: &[TableId],
    figures: &[FigureId],
) -> PublicOutputIndexV1 {
    PublicOutputIndexV1 {
        tables: tables
            .iter()
            .map(|id| PublicOutputStatusV1 {
                output_kind: GeneratedOutputKindV1::Table,
                output_id: id.as_str().into(),
                relative_path: Some(format!(
                    "tables/{}.csv",
                    if *id == TableId::HealthDimensions {
                        "health_dimensions"
                    } else {
                        id.as_str()
                    }
                )),
                format: Some(RenderFormatV1::Csv),
                status: RenderStatusV1::Written,
                reason: None,
            })
            .collect(),
        figures: figures
            .iter()
            .map(|id| match p.figure_reason(*id) {
                Some(reason) => PublicOutputStatusV1 {
                    output_kind: GeneratedOutputKindV1::Figure,
                    output_id: id.as_str().into(),
                    relative_path: None,
                    format: None,
                    status: RenderStatusV1::Unavailable,
                    reason: Some(reason),
                },
                None => PublicOutputStatusV1 {
                    output_kind: GeneratedOutputKindV1::Figure,
                    output_id: id.as_str().into(),
                    relative_path: Some(format!("figures/{}.svg", id.as_str())),
                    format: Some(RenderFormatV1::Svg),
                    status: RenderStatusV1::Written,
                    reason: None,
                },
            })
            .collect(),
    }
}
fn limitations(p: &PublicReportProjection, figures: &[FigureId]) -> Vec<PublicLimitationV1> {
    let mut values = Vec::new();
    for warning in &p.mechanism.warnings {
        values.push(PublicLimitationV1 {
            code: WarningCodeV1::SourceWarning,
            message: warning.message.clone(),
            input_flag: Some(InputFlagV1::Mechanism),
            output_id: None,
        });
    }
    for warning in &p.health.warnings {
        values.push(PublicLimitationV1 {
            code: WarningCodeV1::SourceWarning,
            message: token(warning),
            input_flag: Some(InputFlagV1::Health),
            output_id: None,
        });
    }
    if p.mechanism_is_legacy() {
        values.push(PublicLimitationV1 {
            code: WarningCodeV1::LegacyInput,
            message: "Legacy mechanism artifact; Phase B V1 hypothesis assessment unavailable."
                .into(),
            input_flag: Some(InputFlagV1::Mechanism),
            output_id: None,
        });
    }
    if p.health_is_legacy() {
        values.push(PublicLimitationV1 { code: WarningCodeV1::LegacyInput, message: "Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized.".into(), input_flag: Some(InputFlagV1::Health), output_id: None });
    }
    if p.lineage_catalog.is_none() {
        values.push(PublicLimitationV1 {
            code: WarningCodeV1::CatalogNotSupplied,
            message: "Lineage catalog not supplied; only serialized direct lineage is shown."
                .into(),
            input_flag: Some(InputFlagV1::LineageCatalog),
            output_id: None,
        });
    }
    for (flag_text, lineage) in p.supplied_lineages() {
        if matches!(lineage, ArtifactLineageState::LegacyUnknown { .. }) {
            values.push(PublicLimitationV1 {
                code: WarningCodeV1::LegacyLineage,
                message: format!(
                    "{} has LegacyUnknown lineage; no identity, ancestry, or acquisition family is inferred.",
                    flag_text
                ),
                input_flag: Some(flag(flag_text)),
                output_id: None,
            });
        }
    }
    for comparison in &p.health.baseline_comparison {
        if matches!(
            comparison.comparability,
            FeatureComparability::ComparableWithWarnings
        ) {
            values.push(PublicLimitationV1 {
                code: WarningCodeV1::BaselineComparableWithWarnings,
                message: comparison
                    .override_reason
                    .clone()
                    .unwrap_or_else(|| "Comparable with upstream context warning.".into()),
                input_flag: Some(InputFlagV1::Health),
                output_id: Some(FigureId::CurrentVsBaseline.as_str().into()),
            });
        }
    }
    for figure in figures {
        if let Some(reason) = p.figure_reason(*figure) {
            values.push(PublicLimitationV1 {
                code: WarningCodeV1::OutputUnavailable,
                message: format!(
                    "{} is unavailable: {}",
                    figure.as_str(),
                    crate::reporting::claims::unavailable_text(reason)
                ),
                input_flag: None,
                output_id: Some(figure.as_str().into()),
            });
        }
    }
    values
}
fn mechanism_messages(warnings: &[crate::results::MechanismWarning]) -> Vec<PublicMessageV1> {
    warnings
        .iter()
        .map(|warning| PublicMessageV1 {
            code: WarningCodeV1::SourceWarning,
            message: warning.message.clone(),
        })
        .collect()
}

impl From<&HealthWarning> for HealthWarningV1 {
    fn from(value: &HealthWarning) -> Self {
        match value {
            HealthWarning::MissingBaseline => Self::MissingBaseline,
            HealthWarning::InsufficientBaselineRecords => Self::InsufficientBaselineRecords,
            HealthWarning::BaselineVarianceUnavailable => Self::BaselineVarianceUnavailable,
            HealthWarning::FeatureNoncomparable => Self::FeatureNoncomparable,
            HealthWarning::MissingSignalArtifact => Self::MissingSignalArtifact,
            HealthWarning::MissingTransientArtifact => Self::MissingTransientArtifact,
            HealthWarning::MissingCalibrationArtifact => Self::MissingCalibrationArtifact,
            HealthWarning::MissingEisArtifact => Self::MissingEisArtifact,
            HealthWarning::MissingMechanismArtifact => Self::MissingMechanismArtifact,
            HealthWarning::ArtifactSchemaMismatch => Self::ArtifactSchemaMismatch,
            HealthWarning::ArtifactConfigurationMismatch => Self::ArtifactConfigurationMismatch,
            HealthWarning::EnvironmentalMismatch => Self::EnvironmentalMismatch,
            HealthWarning::InsufficientEvidenceDomains => Self::InsufficientEvidenceDomains,
            HealthWarning::ContradictoryEvidence => Self::ContradictoryEvidence,
            HealthWarning::RuleConditionUnavailable => Self::RuleConditionUnavailable,
            HealthWarning::SemanticRoleUnavailable => Self::SemanticRoleUnavailable,
            HealthWarning::AssessmentBasedOnWarningBearingFits => {
                Self::AssessmentBasedOnWarningBearingFits
            }
            HealthWarning::InvalidRule => Self::InvalidRule,
            HealthWarning::NonFiniteArtifact => Self::NonFiniteArtifact,
            HealthWarning::MixedAnalyteContext => Self::MixedAnalyteContext,
            HealthWarning::MixedSampleMatrixContext => Self::MixedSampleMatrixContext,
            HealthWarning::MixedSensorDesignContext => Self::MixedSensorDesignContext,
            HealthWarning::MixedSensorTypeContext => Self::MixedSensorTypeContext,
            HealthWarning::MixedTemperatureContext => Self::MixedTemperatureContext,
        }
    }
}
fn basename(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned()
}
fn compatibility(outcome: CompatibilityOutcome) -> CompatibilityStatusV1 {
    match outcome.status {
        CompatibilityStatus::Compatible => CompatibilityStatusV1::Compatible,
        CompatibilityStatus::LegacyUnknown => CompatibilityStatusV1::LegacyUnknown,
        CompatibilityStatus::NotProvided => CompatibilityStatusV1::NotProvided,
        CompatibilityStatus::NotApplicable => CompatibilityStatusV1::NotApplicable,
    }
}
fn flag(value: &str) -> InputFlagV1 {
    match value.trim_start_matches("--") {
        "mechanism" => InputFlagV1::Mechanism,
        "health" => InputFlagV1::Health,
        "eis" => InputFlagV1::Eis,
        "transient" => InputFlagV1::Transient,
        "calibration" => InputFlagV1::Calibration,
        "calibration-observations" | "calibration_observations" => {
            InputFlagV1::CalibrationObservations
        }
        "signal" => InputFlagV1::Signal,
        "estimation" => InputFlagV1::Estimation,
        "model" => InputFlagV1::Model,
        "lineage-catalog" | "lineage_catalog" => InputFlagV1::LineageCatalog,
        _ => unreachable!("fixed report input flag"),
    }
}

pub(crate) fn write_markdown_report(
    root: &Path,
    p: &PublicReportProjection,
    table_ids: &[TableId],
    figure_ids: &[FigureId],
) -> Result<String, crate::reporting::PublicReportError> {
    let path = root.join("scientific_report.md");
    let mut text = String::new();
    section(&mut text, "Analysis identity and renderer boundary");
    text.push_str(&format!(
        "- Renderer contract: `mhi_v1_phase_d_public_output_v1`\n- Route: `electroanalysis report render`\n- Mechanism analysis: `{}` (schema {})\n- Health assessment: `{}` (schema {})\n\n",
        p.mechanism.analysis_id,
        p.mechanism.schema_version,
        p.health.assessment_id,
        p.health.schema_version
    ));
    text.push_str(REQUIRED_DISCLAIMER);
    text.push_str("\n\nThis certified renderer projects validated serialized artifacts; it does not refit, reclassify, calculate new thresholds, derive missing values, or traverse lineage.\n\n");
    section(&mut text, "Input artifacts and compatibility state");
    text.push_str("| input | kind | schema | availability | lineage | families |\n| --- | --- | --- | --- | --- | --- |\n");
    for reference in input_references(p, table_ids, figure_ids) {
        match reference {
            PublicInputReferenceV1::Artifact(reference) => {
                let (lineage_status, families) =
                    reference
                        .lineage
                        .as_ref()
                        .map_or(("NA".into(), "NA".into()), |lineage| {
                            (
                                token(&lineage.status),
                                reference
                                    .acquisition_families
                                    .as_ref()
                                    .map(|families| {
                                        format!(
                                            "{}:{}",
                                            token(&families.status),
                                            markdown_collection(&families.values)
                                        )
                                    })
                                    .unwrap_or_else(|| "NA".into()),
                            )
                        });
                text.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} |\n",
                    token(&reference.input_flag),
                    reference
                        .artifact_kind
                        .map(|kind| kind.as_str().to_owned())
                        .unwrap_or_else(|| "NA".into()),
                    reference
                        .schema_version
                        .map_or_else(|| "NA".into(), |value| value.to_string()),
                    token(&reference.availability),
                    lineage_status,
                    families
                ));
            }
            PublicInputReferenceV1::LineageCatalog(reference) => {
                text.push_str(&format!(
                    "| lineage_catalog | lineage_catalog | {} | {} | {} | NA |\n",
                    reference
                        .schema_version
                        .map_or_else(|| "NA".into(), |value| value.to_string()),
                    token(&reference.availability),
                    token(&reference.validation)
                ));
            }
        }
    }
    text.push_str(&format!(
        "\nRequired mechanism/health compatibility: `{}`.\n\n",
        token(&compatibility(p.required_compatibility))
    ));
    for (input, against, outcome) in &p.optional_compatibility {
        text.push_str(&format!(
            "- `{input}` against `{against}`: `{}`\n",
            token(&compatibility(*outcome))
        ));
    }
    text.push('\n');
    section(&mut text, "Mechanism assessment");
    if p.mechanism_is_legacy() {
        text.push_str(
            "Legacy mechanism artifact; Phase B V1 hypothesis assessment unavailable.\n\n",
        );
    } else {
        let mut rows = p
            .mechanism
            .hypothesis_assessments
            .iter()
            .collect::<Vec<_>>();
        rows.sort_by(|a, b| a.definition.hypothesis_id.cmp(&b.definition.hypothesis_id));
        for row in rows {
            text.push_str(&format!(
                "## {} — {}\n\n- Assessment: {}\n- Evidence level: `{}`\n- Target components: {}\n- Reason codes: {}\n- Validation: `{}`\n- Contradiction requirements: {}\n\n",
                row.definition.hypothesis_id,
                row.definition.display_name,
                mechanism_level_text(row.current.evidence_level.clone()),
                token(&row.current.evidence_level),
                markdown_collection(&row.definition.target_components),
                tokens(&row.current.reason_codes),
                token(&row.current.validation_status),
                markdown_collection(
                    &row.current
                        .contradiction_summaries
                        .iter()
                        .map(|item| item.requirement_id.clone())
                        .collect::<Vec<_>>()
                )
            ));
        }
    }
    if p.mechanism.comparisons.is_empty() {
        text.push_str("No serialized timescale comparisons are available.\n\n");
    } else {
        text.push_str("## Serialized timescale comparisons\n\n| comparison | EIS ID | transient ID | ratio | stored log10 distance | evidence | supporting | contradictory | limitations |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
        let mut comparisons = p.mechanism.comparisons.iter().collect::<Vec<_>>();
        comparisons.sort_by(|a, b| a.comparison_id.cmp(&b.comparison_id));
        for row in comparisons {
            text.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                markdown_cell(&row.comparison_id),
                markdown_cell(&row.eis_timescale_id),
                markdown_cell(&row.transient_timescale_id),
                markdown_number(row.ratio)?,
                markdown_number(row.log10_distance)?,
                token(&row.evidence_level),
                markdown_collection(&row.supporting_evidence),
                markdown_collection(&row.contradictory_evidence),
                markdown_collection(&row.alternative_explanations)
            ));
        }
        text.push('\n');
    }
    section(&mut text, "Sensor-health assessment");
    text.push_str(&format!(
        "Overall status: **{}** (`{}`).\n\n",
        health_status_text(p.health.overall_status),
        token(&p.health.overall_status)
    ));
    if let Some(phase) = &p.health.phase_c {
        text.push_str("| dimension | status | evidence state | interpretation | causal status | reasons | source evidence | excluded evidence |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
        for row in &phase.dimension_assessments {
            text.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                token(&row.dimension),
                health_status_text(row.status),
                evidence_state_text(row.evidence_state),
                token(&row.interpretation_category),
                causal_status_text(row.causal_status),
                tokens(&row.reason_codes),
                markdown_collection(
                    &row.source_evidence_ids
                        .iter()
                        .map(|id| id.0.clone())
                        .collect::<Vec<_>>()
                ),
                markdown_collection(
                    &row.excluded_evidence_ids
                        .iter()
                        .map(|id| id.0.clone())
                        .collect::<Vec<_>>()
                )
            ));
        }
        text.push('\n');
    } else {
        text.push_str("Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized.\n\n");
    }
    section(&mut text, "Key evidence and contradictions");
    let mut evidence_rows = 0_usize;
    if p.health.phase_c.is_some() {
        for &record in &p.health_evidence_records {
            let direction = token(&record.direction);
            let availability = token(&record.availability);
            let validity = token(&record.validity);
            if matches!(
                record.direction,
                crate::evidence::EvidenceDirection::Contradicts
            ) || !matches!(
                record.availability,
                crate::evidence::EvidenceAvailability::Available
            ) || !matches!(record.validity, crate::evidence::EvidenceValidity::Valid)
            {
                text.push_str(&format!(
                    "- `{}` targets `{}` from `{}`: direction `{}`, availability `{}`, validity `{}`, quantity {} {}.\n",
                    record.evidence_id.0,
                    markdown_cell(&token(&record.target)),
                    markdown_cell(&record.source.field_path),
                    direction,
                    availability,
                    validity,
                    record
                        .quantity
                        .as_ref()
                        .map(|quantity| format_public_f64(quantity.value))
                        .transpose()
                        .map_err(markdown_number_error)?
                        .unwrap_or_else(|| "NA".into()),
                    record
                        .quantity
                        .as_ref()
                        .map(|quantity| quantity.unit.as_str())
                        .unwrap_or("NA")
                ));
                evidence_rows += 1;
            }
        }
    }
    if evidence_rows == 0 {
        text.push_str("No copied Phase-C evidence record is serialized as contradictory, unavailable, or invalid. This absence is not causal proof.\n");
    }
    text.push('\n');
    section(&mut text, "Uncertainty and data-quality limitations");
    let dqi = p
        .health
        .phase_c
        .as_ref()
        .into_iter()
        .flat_map(|phase| &phase.dimension_assessments)
        .filter(|row| {
            matches!(
                row.status,
                OverallHealthStatus::DataQualityInsufficient | OverallHealthStatus::Indeterminate
            )
        })
        .collect::<Vec<_>>();
    if dqi.is_empty() {
        text.push_str("- No dimension is serialized as Data quality insufficient (DQI) or Indeterminate; missing optional evidence remains disclosed below.\n");
    } else {
        for row in dqi {
            text.push_str(&format!(
                "- `{}` is **{}** with evidence state `{}` and reasons {}.\n",
                token(&row.dimension),
                health_status_text(row.status),
                token(&row.evidence_state),
                tokens(&row.reason_codes)
            ));
        }
    }
    for item in limitations(p, &FigureId::ALL) {
        text.push_str(&format!("- {}\n", item.message));
    }
    text.push('\n');
    section(&mut text, "Current-versus-baseline comparison");
    text.push_str("| feature | unit authority | current | baseline | comparability | absolute difference | relative difference | warning |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    if p.health.baseline_comparison.is_empty() {
        text.push_str("| NA | NA | NA | NA | serialized_series_unavailable | NA | NA | [] |\n");
    }
    for comparison in &p.health.baseline_comparison {
        let units = p
            .health
            .features
            .iter()
            .filter(|feature| feature.name == comparison.feature && !feature.unit.is_empty())
            .map(|feature| feature.unit.as_str())
            .collect::<Vec<_>>();
        let unit = if units.len() == 1 { units[0] } else { "NA" };
        let warning = if matches!(
            comparison.comparability,
            FeatureComparability::ComparableWithWarnings
        ) {
            comparison
                .override_reason
                .as_deref()
                .unwrap_or("Comparable with upstream context warning.")
        } else {
            "[]"
        };
        text.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            markdown_cell(&comparison.feature),
            markdown_cell(unit),
            markdown_number(comparison.current_value)?,
            markdown_number(comparison.baseline_value)?,
            token(&comparison.comparability),
            markdown_number(comparison.absolute_difference)?,
            markdown_number(comparison.relative_difference)?,
            markdown_cell(warning)
        ));
    }
    text.push_str("\nNo unit conversion or baseline reclassification is performed.\n\n");
    section(&mut text, "Optional analysis projections");
    for item in optional_sources(p, table_ids, figure_ids) {
        text.push_str(&format!(
            "- `{}`: availability `{}`, compatibility `{}`{}\n",
            token(&item.kind),
            token(&item.availability),
            token(&item.compatibility),
            item.detail
                .as_ref()
                .map_or_else(String::new, |detail| format!(
                    ", analysis `{}`, records {}, unit `{}`",
                    detail.analysis_id.as_deref().unwrap_or("NA"),
                    detail.record_count,
                    detail.measurement_unit.as_deref().unwrap_or("NA")
                ))
        ));
    }
    text.push('\n');
    section(&mut text, "Figures");
    for id in figure_ids {
        if let Some(reason) = p.figure_reason(*id) {
            text.push_str(&format!(
                "- `{}`: unavailable (`{}` — {}).\n",
                id.as_str(),
                token(&reason),
                crate::reporting::claims::unavailable_text(reason)
            ));
        } else {
            text.push_str(&format!(
                "- `{}`: serialized-source SVG and PNG; labels, units, series legend, and missing-value semantics are embedded.\n",
                id.as_str()
            ));
        }
    }
    text.push_str(
        "\nNo figure contains a renderer-created threshold line or scientific recomputation.\n\n",
    );
    section(&mut text, "Tables");
    for id in table_ids {
        text.push_str(&format!(
            "- `{}` → `tables/{}.csv` (RFC 4180, UTF-8, LF).\n",
            id.as_str(),
            match id {
                TableId::HealthDimensions => "health_dimensions",
                other => other.as_str(),
            }
        ));
    }
    text.push_str("\nMissing scalar values are `NA`; empty collections are `[]`. These tables project detail rather than serving as substitutes for the assessments above.\n\n");
    section(&mut text, "Lineage and provenance");
    for root in lineage_section(p).roots {
        match &root.lineage.identity {
            Some(identity) => {
                text.push_str(&format!(
                    "- `{}` root `{}` (`{}`, schema {}, producer `{}`), catalog membership `{}`.\n",
                    token(&root.input_flag),
                    identity.artifact_id,
                    identity.artifact_kind.as_str(),
                    identity.schema_version,
                    markdown_cell(&identity.producer_version),
                    root.root_catalog_entry_present.map_or("NA", |value| if value { "true" } else { "false" })
                ));
                for dependency in &root.direct_dependencies {
                    text.push_str(&format!(
                        "  - direct `{}` edge to `{}` / `{}`.\n",
                        token(&dependency.role),
                        dependency.artifact_kind.as_str(),
                        dependency.artifact_id
                    ));
                }
            }
            None => text.push_str(&format!(
                "- `{}`: LegacyUnknown (source schema `{}`, reason `{}`); no identity or dependency is invented.\n",
                token(&root.input_flag),
                root.lineage
                    .legacy_source_schema_version
                    .map_or_else(|| "NA".into(), |value| value.to_string()),
                root.lineage
                    .legacy_reason
                    .as_ref()
                    .map(token)
                    .unwrap_or_else(|| "NA".into())
            )),
        }
    }
    text.push_str(&format!(
        "\nMechanism provenance: software `{}`, input SHA-256 `{}`, configuration SHA-256 `{}`, git commit `{}`.\n\nHealth provenance: software `{}`, input SHA-256 `{}`, configuration SHA-256 `{}`, git commit `{}`.\n\n",
        p.mechanism.provenance.as_ref().expect("required").software_version,
        p.mechanism.provenance.as_ref().expect("required").input_sha256,
        p.mechanism.provenance.as_ref().expect("required").configuration_sha256.as_deref().unwrap_or("NA"),
        p.mechanism.provenance.as_ref().expect("required").git_commit.as_deref().unwrap_or("NA"),
        p.health.provenance.software_version,
        p.health.provenance.input_sha256,
        p.health.provenance.configuration_sha256.as_deref().unwrap_or("NA"),
        p.health.provenance.git_commit.as_deref().unwrap_or("NA")
    ));
    section(&mut text, "Reproducibility metadata");
    text.push_str("- JSON object order: declaration order.\n- Array order: contract/source order.\n- Numeric format: `rust_display_normalized_negative_zero_v1`.\n- CSV: `rfc4180_lf_utf8_v1`.\n- Relative path separator: `/`.\n- Clock/host metadata: absent (`null` where the closed JSON graph permits it).\n");
    fs::write(&path, text).map_err(|source| crate::reporting::PublicReportError::Write {
        path: path.clone(),
        source,
    })?;
    Ok("scientific_report.md".into())
}
fn section(text: &mut String, heading: &str) {
    text.push_str("# ");
    text.push_str(heading);
    text.push_str("\n\n");
}
fn token<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_default()
        .trim_matches('"')
        .to_owned()
}
fn tokens<T: Serialize>(values: &[T]) -> String {
    if values.is_empty() {
        "[]".into()
    } else {
        values.iter().map(token).collect::<Vec<_>>().join(";")
    }
}
fn markdown_number(value: Option<f64>) -> Result<String, crate::reporting::PublicReportError> {
    value
        .map(format_public_f64)
        .transpose()
        .map_err(markdown_number_error)
        .map(|value| value.unwrap_or_else(|| "NA".into()))
}
fn markdown_number_error(detail: &'static str) -> crate::reporting::PublicReportError {
    crate::reporting::PublicReportError::StagingValidation {
        path: Path::new("scientific_report.md").to_path_buf(),
        detail: detail.into(),
    }
}
fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}
fn markdown_collection(values: &[String]) -> String {
    if values.is_empty() {
        "[]".into()
    } else {
        values
            .iter()
            .map(|value| markdown_cell(value))
            .collect::<Vec<_>>()
            .join(";")
    }
}
