//! Durable baseline, assessment, evidence, and trend artifacts.

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
    pub analysis_id: String,
    pub trends: Vec<HealthTrend>,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<HealthWarning>,
}
