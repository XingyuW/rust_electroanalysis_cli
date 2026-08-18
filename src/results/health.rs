//! Durable baseline, assessment, evidence, and trend artifacts.

use crate::domain::ArtifactId;
use crate::evidence::{EvidenceBundle, EvidenceId, EvidenceTarget};
use crate::{
    domain::AnalysisProvenance,
    health_config::{HealthFindingKind, HealthSeverity, ResolvedHealthConfig},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureComparability {
    Comparable,
    ComparableWithWarnings,
    NotComparable,
    Unknown,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthConfidence {
    Insufficient,
    Low,
    Moderate,
    High,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverallHealthStatus {
    WithinBaseline,
    Watch,
    Degraded,
    Critical,
    DataQualityInsufficient,
    Indeterminate,
}

/// The fixed Phase-C sensor-health dimensions.  This is deliberately distinct
/// from the older `HealthDomain` grouping, which remains part of the legacy
/// health projection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HealthDimension {
    SignalIntegrity,
    CalibrationHealth,
    DynamicResponseHealth,
    ReferenceStability,
    EnvironmentalRobustness,
    ModelConsistency,
    Observability,
    UncertaintyHealth,
    DataQuality,
}

impl HealthDimension {
    pub const ALL: [Self; 9] = [
        Self::SignalIntegrity,
        Self::CalibrationHealth,
        Self::DynamicResponseHealth,
        Self::ReferenceStability,
        Self::EnvironmentalRobustness,
        Self::ModelConsistency,
        Self::Observability,
        Self::UncertaintyHealth,
        Self::DataQuality,
    ];
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HealthInterpretationCategory {
    ObservedBehavior,
    ModelInconsistency,
    EnvironmentalEffect,
    CalibrationIssue,
    PossiblePhysicalDegradation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CausalStatus {
    Observed,
    Associated,
    Hypothesized,
    ExperimentallySupported,
    ValidatedForDomain,
    Indeterminate,
}

impl CausalStatus {
    pub const fn strength(self) -> u8 {
        match self {
            Self::Indeterminate => 0,
            Self::Observed => 1,
            Self::Associated => 2,
            Self::Hypothesized => 3,
            Self::ExperimentallySupported => 4,
            Self::ValidatedForDomain => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HealthEvidenceState {
    AdequateEvidence,
    NoEvidence,
    InsufficientEvidence,
    PoorDataQuality,
    ContradictoryEvidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PhaseCHealthReasonCode {
    OptionalSourceAbsent,
    RequiredQuantityAbsent,
    InvalidQuantity,
    UnitMismatch,
    SourceIncompatible,
    ScopeIncompatible,
    TemporalIncompatible,
    IncompleteLineage,
    IndependenceUnknown,
    InsufficientIndependentFamilies,
    BaselineAbsent,
    BaselineInsufficient,
    BaselineIncomparable,
    QualityGateFailed,
    ThresholdWithinLimit,
    ThresholdWatch,
    ThresholdDegraded,
    ThresholdCritical,
    ContradictoryEvidence,
    ModelOutsideDomain,
    ModelValidityUnavailable,
    ObservabilityFailed,
    UncertaintyIncomplete,
    MechanismNoncausal,
    MechanismContradicted,
    ReferenceAnchorUnavailable,
    PhaseBHypothesisUnmapped,
    SelectedTransientEventAbsent,
    SelectedTransientEventAmbiguous,
    SelectedTransientEventInvalid,
    BaselineFeatureAbsent,
    BaselineStatisticAbsent,
    BaselineDenominatorZero,
    BaselineDenominatorNearZero,
    OptionalInvalidSourceExcluded,
}

impl PhaseCHealthReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OptionalSourceAbsent => "optional_source_absent",
            Self::RequiredQuantityAbsent => "required_quantity_absent",
            Self::InvalidQuantity => "invalid_quantity",
            Self::UnitMismatch => "unit_mismatch",
            Self::SourceIncompatible => "source_incompatible",
            Self::ScopeIncompatible => "scope_incompatible",
            Self::TemporalIncompatible => "temporal_incompatible",
            Self::IncompleteLineage => "incomplete_lineage",
            Self::IndependenceUnknown => "independence_unknown",
            Self::InsufficientIndependentFamilies => "insufficient_independent_families",
            Self::BaselineAbsent => "baseline_absent",
            Self::BaselineInsufficient => "baseline_insufficient",
            Self::BaselineIncomparable => "baseline_incomparable",
            Self::QualityGateFailed => "quality_gate_failed",
            Self::ThresholdWithinLimit => "threshold_within_limit",
            Self::ThresholdWatch => "threshold_watch",
            Self::ThresholdDegraded => "threshold_degraded",
            Self::ThresholdCritical => "threshold_critical",
            Self::ContradictoryEvidence => "contradictory_evidence",
            Self::ModelOutsideDomain => "model_outside_domain",
            Self::ModelValidityUnavailable => "model_validity_unavailable",
            Self::ObservabilityFailed => "observability_failed",
            Self::UncertaintyIncomplete => "uncertainty_incomplete",
            Self::MechanismNoncausal => "mechanism_noncausal",
            Self::MechanismContradicted => "mechanism_contradicted",
            Self::ReferenceAnchorUnavailable => "reference_anchor_unavailable",
            Self::PhaseBHypothesisUnmapped => "phase_b_hypothesis_unmapped",
            Self::SelectedTransientEventAbsent => "selected_transient_event_absent",
            Self::SelectedTransientEventAmbiguous => "selected_transient_event_ambiguous",
            Self::SelectedTransientEventInvalid => "selected_transient_event_invalid",
            Self::BaselineFeatureAbsent => "baseline_feature_absent",
            Self::BaselineStatisticAbsent => "baseline_statistic_absent",
            Self::BaselineDenominatorZero => "baseline_denominator_zero",
            Self::BaselineDenominatorNearZero => "baseline_denominator_near_zero",
            Self::OptionalInvalidSourceExcluded => "optional_invalid_source_excluded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCHealthDimensionAssessment {
    pub dimension: HealthDimension,
    pub status: OverallHealthStatus,
    pub evidence_state: HealthEvidenceState,
    pub interpretation_category: HealthInterpretationCategory,
    pub causal_status: CausalStatus,
    pub reason_codes: Vec<PhaseCHealthReasonCode>,
    pub source_evidence_ids: Vec<EvidenceId>,
    pub source_artifact_ids: Vec<ArtifactId>,
    #[serde(default)]
    pub excluded_evidence_ids: Vec<EvidenceId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCSensorHealthEvidenceReport {
    pub config_schema_version: u32,
    pub config_sha256: String,
    pub dimension_assessments: Vec<PhaseCHealthDimensionAssessment>,
    pub overall_status: OverallHealthStatus,
    pub overall_interpretation_categories: Vec<HealthInterpretationCategory>,
    pub overall_causal_status: CausalStatus,
    pub evidence_bundle: EvidenceBundle,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HealthDomain {
    DataQuality,
    SignalNoise,
    Drift,
    DynamicResponse,
    Calibration,
    Impedance,
    MechanismEvidence,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaselineFeatureDistribution {
    pub feature: String,
    pub unit: String,
    pub domain: HealthDomain,
    pub sample_count: usize,
    pub mean: Option<f64>,
    pub standard_deviation: Option<f64>,
    pub median: Option<f64>,
    pub mad: Option<f64>,
    pub quantiles: Vec<(f64, Option<f64>)>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub reference_direction: Option<String>,
    pub comparison_context: Option<String>,
    /// Finite empirical observations retained for a true empirical percentile.
    #[serde(default)]
    pub empirical_values: Vec<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaselineRecordSummary {
    pub record_id: String,
    #[serde(default)]
    pub experiment_id: Option<String>,
    pub sensor_id: Option<String>,
    #[serde(default)]
    pub sensor_type: Option<String>,
    pub analyte: Option<String>,
    pub sample_matrix: Option<String>,
    pub temperature_k: Option<f64>,
    pub sensor_design: Option<String>,
    pub domains: Vec<HealthDomain>,
    #[serde(default)]
    pub metadata_source: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaselineContextConflict {
    pub field: String,
    pub values: Vec<String>,
    pub record_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensorHealthBaseline {
    pub schema_version: u32,
    #[serde(default = "crate::domain::legacy_unknown_lineage")]
    pub lineage: crate::domain::ArtifactLineageState,
    pub baseline_id: String,
    pub sensor_type: Option<String>,
    pub sensor_design: Option<String>,
    pub analyte: Option<String>,
    pub sample_matrix: Option<String>,
    pub temperature_domain_k: Option<(f64, f64)>,
    pub feature_distributions: Vec<BaselineFeatureDistribution>,
    pub records: Vec<BaselineRecordSummary>,
    /// Minimum number of baseline records required by the configuration.
    #[serde(default)]
    pub minimum_required_records: usize,
    /// Domains represented by at least one baseline record.
    #[serde(default)]
    pub represented_domains: Vec<HealthDomain>,
    /// Old schema-1 field retained only for deserialization; it was incorrectly
    /// populated with a record count and is never used for current semantics.
    #[serde(default, alias = "minimum_required_domains")]
    pub legacy_minimum_required_domains: Option<usize>,
    #[serde(default)]
    pub context_conflicts: Vec<BaselineContextConflict>,
    #[serde(default)]
    pub metadata_sources: Vec<String>,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<HealthWarning>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthFeature {
    pub name: String,
    pub value: Option<f64>,
    pub unit: String,
    pub domain: HealthDomain,
    pub source: String,
    pub warning: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaselineComparison {
    pub feature: String,
    pub current_value: Option<f64>,
    pub baseline_value: Option<f64>,
    pub comparability: FeatureComparability,
    pub absolute_difference: Option<f64>,
    pub relative_difference: Option<f64>,
    pub log_ratio: Option<f64>,
    pub z_score: Option<f64>,
    pub robust_z_score: Option<f64>,
    /// Empirical percentile: 100 * fraction of valid baseline values <= current.
    #[serde(default)]
    pub empirical_percentile: Option<f64>,
    /// Legacy min-max range position, explicitly not a statistical percentile.
    #[serde(default, alias = "percentile_position")]
    pub range_position_percent: Option<f64>,
    pub override_reason: Option<String>,
    #[serde(default)]
    pub baseline_sample_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthEvidence {
    pub domain: HealthDomain,
    pub feature: String,
    pub statement: String,
    pub strength: HealthConfidence,
    pub source: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleEvaluation {
    pub rule_id: String,
    pub conditions_satisfied: Vec<String>,
    pub conditions_not_satisfied: Vec<String>,
    pub conditions_unavailable: Vec<String>,
    pub evidence_domains: Vec<HealthDomain>,
    pub supporting_evidence: Vec<HealthEvidence>,
    pub contradictory_evidence: Vec<HealthEvidence>,
    pub severity: HealthSeverity,
    pub confidence: HealthConfidence,
    pub triggered: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthFinding {
    pub finding: HealthFindingKind,
    pub severity: HealthSeverity,
    pub confidence: HealthConfidence,
    pub supporting_evidence: Vec<HealthEvidence>,
    pub contradictory_evidence: Vec<HealthEvidence>,
    pub unavailable_evidence: Vec<String>,
    pub alternative_explanations: Vec<String>,
    pub triggered_rules: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthDomainAssessment {
    pub domain: HealthDomain,
    pub status: OverallHealthStatus,
    pub confidence: HealthConfidence,
    pub feature_count: usize,
    pub available_features: usize,
    pub warning_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthWarning {
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensorHealthAssessment {
    pub schema_version: u32,
    #[serde(default = "crate::domain::legacy_unknown_lineage")]
    pub lineage: crate::domain::ArtifactLineageState,
    pub assessment_id: String,
    pub sensor_id: Option<String>,
    pub experiment_id: Option<String>,
    pub overall_status: OverallHealthStatus,
    pub domain_assessments: Vec<HealthDomainAssessment>,
    pub features: Vec<HealthFeature>,
    pub findings: Vec<HealthFinding>,
    pub rule_evaluations: Vec<RuleEvaluation>,
    pub baseline_comparison: Vec<BaselineComparison>,
    pub missing_domains: Vec<HealthDomain>,
    pub configuration: ResolvedHealthConfig,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<HealthWarning>,
    /// Present only for the current schema-4 Phase-C route.  The legacy
    /// schema-3 writer removes this optional field from its wire payload.
    #[serde(default)]
    pub phase_c: Option<PhaseCSensorHealthEvidenceReport>,
}

impl SensorHealthAssessment {
    pub(crate) fn validate_phase_c(&self) -> Result<(), String> {
        if self.schema_version < 4 {
            if self.phase_c.is_some() {
                return Err("schema-3 health assessment must not contain phase_c".into());
            }
            return Ok(());
        }
        let report = self
            .phase_c
            .as_ref()
            .ok_or_else(|| "schema-4 health assessment requires a non-null phase_c".to_owned())?;
        if report.config_schema_version != 1
            || report.config_sha256.len() != 64
            || !report
                .config_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(
                "schema-4 health assessment has an invalid phase_c configuration identity".into(),
            );
        }
        if report.dimension_assessments.len() != HealthDimension::ALL.len()
            || report
                .dimension_assessments
                .iter()
                .zip(HealthDimension::ALL)
                .any(|(row, expected)| row.dimension != expected)
        {
            return Err(
                "schema-4 health assessment requires exactly one record for each health dimension"
                    .into(),
            );
        }
        if report.overall_status != self.overall_status {
            return Err(
                "schema-4 health assessment overall_status must equal phase_c.overall_status"
                    .into(),
            );
        }
        report.evidence_bundle.validate().map_err(|error| {
            format!("schema-4 health assessment has invalid evidence_bundle: {error}")
        })?;
        let bundle_ids = report
            .evidence_bundle
            .records
            .iter()
            .map(|record| &record.evidence_id)
            .collect::<std::collections::BTreeSet<_>>();
        for row in &report.dimension_assessments {
            if !phase_c_status_matches(row)
                || row.reason_codes.is_empty()
                || (row.interpretation_category
                    == HealthInterpretationCategory::PossiblePhysicalDegradation
                    && (!matches!(
                        row.dimension,
                        HealthDimension::SignalIntegrity
                            | HealthDimension::CalibrationHealth
                            | HealthDimension::DynamicResponseHealth
                    ) || !matches!(
                        row.status,
                        OverallHealthStatus::Degraded | OverallHealthStatus::Critical
                    ) || row.evidence_state != HealthEvidenceState::AdequateEvidence))
            {
                return Err(
                    "schema-4 health assessment has an invalid phase_c dimension record".into(),
                );
            }
            if !reason_codes_are_canonical(&row.reason_codes)
                || !strictly_sorted(&row.source_evidence_ids)
                || !strictly_sorted(&row.source_artifact_ids)
                || !strictly_sorted(&row.excluded_evidence_ids)
            {
                return Err("schema-4 health assessment phase_c collections must be sorted and duplicate-free".into());
            }
            if row
                .source_evidence_ids
                .iter()
                .chain(row.excluded_evidence_ids.iter())
                .any(|id| !bundle_ids.contains(id))
            {
                return Err(
                    "schema-4 health assessment phase_c references unknown evidence".into(),
                );
            }
            if row.source_evidence_ids.iter().any(|id| {
                report
                    .evidence_bundle
                    .records
                    .iter()
                    .find(|record| &record.evidence_id == id)
                    .is_some_and(|record| {
                        !matches!(record.target, EvidenceTarget::HealthDimension(dimension) if dimension == row.dimension)
                    })
            }) {
                return Err("schema-4 health assessment phase_c evidence targets the wrong dimension".into());
            }
        }
        let expected_overall = phase_c_overall_status(&report.dimension_assessments);
        let positive = report
            .dimension_assessments
            .iter()
            .filter(|row| {
                matches!(
                    row.status,
                    OverallHealthStatus::Watch
                        | OverallHealthStatus::Degraded
                        | OverallHealthStatus::Critical
                )
            })
            .collect::<Vec<_>>();
        let mut expected_categories = Vec::new();
        for category in positive.iter().map(|row| row.interpretation_category) {
            if !expected_categories.contains(&category) {
                expected_categories.push(category);
            }
        }
        let expected_causal = positive
            .iter()
            .map(|row| row.causal_status)
            .min_by_key(|status| status.strength())
            .unwrap_or(CausalStatus::Indeterminate);
        if report.overall_status != expected_overall
            || report.overall_interpretation_categories != expected_categories
            || report.overall_causal_status != expected_causal
        {
            return Err(
                "schema-4 health assessment has inconsistent phase_c aggregate fields".into(),
            );
        }
        Ok(())
    }
}

fn phase_c_overall_status(rows: &[PhaseCHealthDimensionAssessment]) -> OverallHealthStatus {
    for status in [
        OverallHealthStatus::Critical,
        OverallHealthStatus::Degraded,
        OverallHealthStatus::Watch,
        OverallHealthStatus::DataQualityInsufficient,
        OverallHealthStatus::Indeterminate,
    ] {
        if rows.iter().any(|row| row.status == status) {
            return status;
        }
    }
    OverallHealthStatus::WithinBaseline
}

fn phase_c_status_matches(row: &PhaseCHealthDimensionAssessment) -> bool {
    match row.evidence_state {
        HealthEvidenceState::AdequateEvidence => matches!(
            row.status,
            OverallHealthStatus::WithinBaseline
                | OverallHealthStatus::Watch
                | OverallHealthStatus::Degraded
                | OverallHealthStatus::Critical
        ),
        HealthEvidenceState::NoEvidence | HealthEvidenceState::InsufficientEvidence => {
            row.status == OverallHealthStatus::Indeterminate
        }
        HealthEvidenceState::PoorDataQuality => {
            row.status == OverallHealthStatus::DataQualityInsufficient
        }
        HealthEvidenceState::ContradictoryEvidence => matches!(
            row.status,
            OverallHealthStatus::Watch
                | OverallHealthStatus::Degraded
                | OverallHealthStatus::Critical
        ),
    }
}

fn reason_codes_are_canonical(items: &[PhaseCHealthReasonCode]) -> bool {
    let Some((primary, secondary)) = items.split_first() else {
        return true;
    };
    !secondary.contains(primary)
        && secondary
            .windows(2)
            .all(|pair| pair[0].as_str() < pair[1].as_str())
}

fn strictly_sorted<T: Ord>(items: &[T]) -> bool {
    items.windows(2).all(|pair| pair[0] < pair[1])
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthTrendPoint {
    pub record_id: String,
    pub independent_value: Option<f64>,
    pub feature: String,
    pub value: Option<f64>,
    pub absolute_change: Option<f64>,
    pub relative_change: Option<f64>,
    pub log_change: Option<f64>,
    pub change_from_baseline: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthTrend {
    pub feature: String,
    pub points: Vec<HealthTrendPoint>,
    pub ordinary_slope: Option<f64>,
    pub theil_sen_slope: Option<f64>,
    pub rank_correlation: Option<f64>,
    pub replicate_standard_deviation: Option<f64>,
    pub warnings: Vec<HealthWarning>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthTrendReport {
    pub schema_version: u32,
    #[serde(default = "crate::domain::legacy_unknown_lineage")]
    pub lineage: crate::domain::ArtifactLineageState,
    pub analysis_id: String,
    pub trends: Vec<HealthTrend>,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<HealthWarning>,
}
