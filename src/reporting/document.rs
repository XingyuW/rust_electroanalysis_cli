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
pub struct PublicCompatibilityV1 {
    pub required_pair: CompatibilityStatusV1,
    pub optional: Vec<CompatibilityRecordV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityRecordV1 {
    pub input_flag: InputFlagV1,
    pub against_flag: InputFlagV1,
    pub status: CompatibilityStatusV1,
    pub mismatch_axis: Option<crate::reporting::CompatibilityAxis>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicMessageV1 {
    pub code: WarningCodeV1,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicHypothesisV1 {
    pub hypothesis_id: String,
    pub display_name: String,
    pub target_components: Vec<String>,
    pub evidence_level: crate::mechanism::promotion::HypothesisEvidenceLevel,
    pub reason_codes: Vec<crate::mechanism::promotion::PhaseBHypothesisReasonCode>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
pub struct PublicHealthFeatureV1 {
    pub name: String,
    pub value: Option<f64>,
    pub unit: String,
    pub domain: HealthDomain,
    pub source: String,
    pub warning: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
pub struct PublicHealthSectionV1 {
    pub availability: AvailabilityV1,
    pub assessment_id: String,
    pub sensor_id: Option<String>,
    pub experiment_id: Option<String>,
    pub overall_status: OverallHealthStatus,
    pub dimensions: Vec<PublicHealthDimensionV1>,
    pub features: Vec<PublicHealthFeatureV1>,
    pub baseline_comparisons: Vec<PublicBaselineComparisonV1>,
    pub warning_codes: Vec<HealthWarning>,
    pub lineage: LineagePresentationV1,
    pub provenance: ProvenancePresentationV1,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptionalSourceDetailV1 {
    pub analysis_id: Option<String>,
    pub record_count: u64,
    pub measurement_unit: Option<String>,
    pub lineage: LineagePresentationV1,
    pub provenance: ProvenancePresentationV1,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicOptionalSourceV1 {
    pub kind: OptionalSourceKindV1,
    pub availability: AvailabilityV1,
    pub compatibility: CompatibilityStatusV1,
    pub input: Option<PublicInputReferenceV1>,
    pub detail: Option<OptionalSourceDetailV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicLineageSectionV1 {
    pub catalog_supplied: bool,
    pub roots: Vec<PublicLineageRootV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicOutputStatusV1 {
    pub output_kind: GeneratedOutputKindV1,
    pub output_id: String,
    pub relative_path: Option<String>,
    pub format: Option<RenderFormatV1>,
    pub status: RenderStatusV1,
    pub reason: Option<AvailabilityReason>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicOutputIndexV1 {
    pub tables: Vec<PublicOutputStatusV1>,
    pub figures: Vec<PublicOutputStatusV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicLimitationV1 {
    pub code: WarningCodeV1,
    pub message: String,
    pub input_flag: Option<InputFlagV1>,
    pub output_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicRenderingMetadataV1 {
    pub json_schema: String,
    pub numeric_format: String,
    pub csv_newline: String,
    pub timestamp: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

pub fn write_public_summary_json(
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
        input_references: input_references(p),
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
        optional_sources: optional_sources(p),
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
            warnings: messages(&row.warnings),
        })
        .collect::<Vec<_>>();
    comparisons.sort_by(|a, b| a.comparison_id.cmp(&b.comparison_id));
    PublicMechanismSectionV1 {
        availability: if p.mechanism_is_legacy() {
            AvailabilityV1::Unavailable
        } else {
            AvailabilityV1::Available
        },
        analysis_id: p.mechanism.analysis_id.clone(),
        hypotheses,
        comparisons,
        warning_messages: messages(&p.mechanism.warnings),
        lineage: lineage::project_lineage(&p.mechanism.lineage),
        provenance: lineage::project_provenance(p.mechanism.provenance.as_ref()),
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
        warning_codes: p.health.warnings.clone(),
        lineage: lineage::project_lineage(&p.health.lineage),
        provenance: lineage::project_provenance(Some(&p.health.provenance)),
    }
}
fn input_references(p: &PublicReportProjection) -> Vec<PublicInputReferenceV1> {
    vec![
        artifact_reference(
            ArtifactInputFlagV1::Mechanism,
            Some(&p.input_paths.mechanism),
            Some(ArtifactKind::MechanismAnalysis),
            Some(p.mechanism.schema_version),
            Some(&p.mechanism.lineage),
        ),
        artifact_reference(
            ArtifactInputFlagV1::Health,
            Some(&p.input_paths.health),
            Some(ArtifactKind::HealthAssessment),
            Some(p.health.schema_version),
            Some(&p.health.lineage),
        ),
        optional_reference(
            ArtifactInputFlagV1::Eis,
            p.input_paths.eis.as_deref(),
            p.eis
                .as_ref()
                .map(|a| (ArtifactKind::EisFit, a.schema_version, &a.lineage)),
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
        ),
        optional_reference(
            ArtifactInputFlagV1::Signal,
            p.input_paths.signal.as_deref(),
            p.signal
                .as_ref()
                .map(|a| (ArtifactKind::SignalAnalysis, a.schema_version, &a.lineage)),
        ),
        optional_reference(
            ArtifactInputFlagV1::Estimation,
            p.input_paths.estimation.as_deref(),
            p.estimation
                .as_ref()
                .map(|a| (ArtifactKind::StateEstimation, a.schema_version, &a.lineage)),
        ),
        optional_reference(
            ArtifactInputFlagV1::Model,
            p.input_paths.model.as_deref(),
            p.model
                .as_ref()
                .map(|a| (ArtifactKind::ModelAnalysis, a.schema_version, &a.lineage)),
        ),
        PublicInputReferenceV1::LineageCatalog(PublicLineageCatalogInputReferenceV1 {
            supplied_path_basename: p.input_paths.lineage_catalog.as_deref().map(basename),
            schema_version: p.lineage_catalog.as_ref().map(|_| 1),
            availability: if p.lineage_catalog.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
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
        availability: if kind.is_some() {
            if legacy {
                AvailabilityV1::AvailableWithWarnings
            } else {
                AvailabilityV1::Available
            }
        } else {
            AvailabilityV1::NotProvided
        },
    })
}
fn optional_reference(
    flag: ArtifactInputFlagV1,
    path: Option<&Path>,
    input: Option<(ArtifactKind, u32, &ArtifactLineageState)>,
) -> PublicInputReferenceV1 {
    match input {
        Some((kind, schema, lineage_state)) => {
            artifact_reference(flag, path, Some(kind), Some(schema), Some(lineage_state))
        }
        None => artifact_reference(flag, None, None, None, None),
    }
}
fn optional_sources(p: &PublicReportProjection) -> Vec<PublicOptionalSourceV1> {
    vec![
        optional_source(
            OptionalSourceKindV1::Eis,
            p.eis.as_ref().map(|a| {
                (
                    Some(a.fit_id.clone()),
                    a.source.frequency_hz.len() as u64,
                    Some("Ohm".into()),
                    &a.lineage,
                    Some(&a.provenance),
                )
            }),
        ),
        optional_source(
            OptionalSourceKindV1::Transient,
            p.transient.as_ref().map(|a| {
                (
                    Some(a.experiment_id.clone()),
                    a.events.len() as u64,
                    Some(a.channel_unit.clone()),
                    &a.lineage,
                    Some(&a.provenance),
                )
            }),
        ),
        optional_source(
            OptionalSourceKindV1::Calibration,
            p.calibration.as_ref().map(|a| {
                (
                    Some(a.calibration_id.clone()),
                    a.validation.as_ref().map_or(0, |v| v.predictions.len()) as u64,
                    Some("V".into()),
                    &a.lineage,
                    Some(&a.provenance),
                )
            }),
        ),
        optional_source(
            OptionalSourceKindV1::Signal,
            p.signal.as_ref().map(|a| {
                (
                    Some(a.analysis_id.clone()),
                    a.analysis_timestamps.len() as u64,
                    Some(a.unit.clone()),
                    &a.lineage,
                    Some(&a.provenance),
                )
            }),
        ),
        optional_source(
            OptionalSourceKindV1::Estimation,
            p.estimation.as_ref().map(|a| {
                (
                    Some(a.analysis_id.clone()),
                    a.estimates.len() as u64,
                    Some("V".into()),
                    &a.lineage,
                    Some(&a.provenance),
                )
            }),
        ),
        optional_source_model(p),
        PublicOptionalSourceV1 {
            kind: OptionalSourceKindV1::LineageCatalog,
            availability: if p.lineage_catalog.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
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
    Option<&'a AnalysisProvenance>,
);

fn optional_source(
    kind: OptionalSourceKindV1,
    input: Option<OptionalSourceInput<'_>>,
) -> PublicOptionalSourceV1 {
    match input {
        Some((analysis_id, record_count, unit, state, provenance)) => PublicOptionalSourceV1 {
            kind,
            availability: AvailabilityV1::Available,
            compatibility: CompatibilityStatusV1::Compatible,
            input: None,
            detail: Some(OptionalSourceDetailV1 {
                analysis_id,
                record_count,
                measurement_unit: unit,
                lineage: lineage::project_lineage(state),
                provenance: lineage::project_provenance(provenance),
            }),
        },
        None => PublicOptionalSourceV1 {
            kind,
            availability: AvailabilityV1::NotProvided,
            compatibility: CompatibilityStatusV1::NotProvided,
            input: None,
            detail: None,
        },
    }
}
fn optional_source_model(p: &PublicReportProjection) -> PublicOptionalSourceV1 {
    match &p.model {
        Some(model) => PublicOptionalSourceV1 {
            kind: OptionalSourceKindV1::Model,
            availability: AvailabilityV1::AvailableWithWarnings,
            compatibility: CompatibilityStatusV1::LegacyUnknown,
            input: None,
            detail: Some(OptionalSourceDetailV1 {
                analysis_id: None,
                record_count: model.points.len() as u64,
                measurement_unit: Some("V".into()),
                lineage: lineage::project_lineage(&model.lineage),
                provenance: lineage::project_provenance(None),
            }),
        },
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
                relative_path: Some(format!("tables/{}.csv", id.as_str())),
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
fn messages<T: std::fmt::Debug>(warnings: &[T]) -> Vec<PublicMessageV1> {
    warnings
        .iter()
        .map(|warning| PublicMessageV1 {
            code: WarningCodeV1::SourceWarning,
            message: format!("{warning:?}"),
        })
        .collect()
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

pub fn write_markdown_report(
    root: &Path,
    p: &PublicReportProjection,
) -> Result<String, crate::reporting::PublicReportError> {
    let path = root.join("scientific_report.md");
    let mut text = String::new();
    section(&mut text, "Analysis identity and renderer boundary");
    text.push_str(REQUIRED_DISCLAIMER);
    text.push_str("\n\nThis certified renderer projects validated serialized artifacts; it does not refit, reclassify, calculate new thresholds, or resolve lineage.\n\n");
    section(&mut text, "Input artifacts and compatibility state");
    for reference in input_references(p) {
        text.push_str(&format!(
            "- `{}`\n",
            serde_json::to_string(&reference).unwrap_or_default()
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
                "## {} — {}\n\n{}\n\nReason codes: {}\n\n",
                row.definition.hypothesis_id,
                row.definition.display_name,
                mechanism_level_text(row.current.evidence_level.clone()),
                tokens(&row.current.reason_codes)
            ));
        }
    }
    section(&mut text, "Sensor-health assessment");
    if let Some(phase) = &p.health.phase_c {
        for row in &phase.dimension_assessments {
            text.push_str(&format!(
                "## {}\n\n{}; {}; {}\n\nReason codes: {}\n\n",
                token(&row.dimension),
                health_status_text(row.status),
                evidence_state_text(row.evidence_state),
                causal_status_text(row.causal_status),
                tokens(&row.reason_codes)
            ));
        }
    } else {
        text.push_str("Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized.\n\n");
    }
    section(&mut text, "Key evidence and contradictions");
    text.push_str("Serialized contradictions and excluded evidence are retained in the health-dimensions and evidence-provenance tables.\n\n");
    section(&mut text, "Uncertainty and data-quality limitations");
    for item in limitations(p, &FigureId::ALL) {
        text.push_str(&format!("- {}\n", item.message));
    }
    text.push('\n');
    section(&mut text, "Current-versus-baseline comparison");
    text.push_str("Comparable values use the unique serialized HealthFeature unit authority. Comparable-with-warnings is rendered and disclosed without conversion.\n\n");
    section(&mut text, "Optional analysis projections");
    for item in optional_sources(p) {
        text.push_str(&format!(
            "- `{}`: `{}`\n",
            serde_json::to_string(&item.kind).unwrap_or_default(),
            serde_json::to_string(&item.availability).unwrap_or_default()
        ));
    }
    text.push('\n');
    section(&mut text, "Figures");
    text.push_str("Every figure is artifact-only and has no threshold line. Refer to the paired SVG/PNG files and manifest availability records.\n\n");
    section(&mut text, "Tables");
    text.push_str("Seven RFC4180 LF CSV tables contain full public detail. Missing values are `NA`; empty collections are `[]`.\n\n");
    section(&mut text, "Lineage and provenance");
    for root in lineage_section(p).roots {
        text.push_str(&format!(
            "- `{}`: `{}`\n",
            serde_json::to_string(&root.input_flag).unwrap_or_default(),
            serde_json::to_string(&root.lineage).unwrap_or_default()
        ));
    }
    text.push('\n');
    section(&mut text, "Reproducibility metadata");
    text.push_str("JSON uses declaration order and LF; numeric text uses Rust Display with negative zero normalized to `0`; successful output contains no clock or host path.\n");
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
