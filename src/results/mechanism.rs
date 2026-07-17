//! Stable serializable outputs for transient/EIS comparison.

use crate::domain::AnalysisProvenance;
use crate::transient_config::ResolvedTransientConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimescaleSource {
    EisCircuit,
    TransientFit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimescaleValidity {
    Valid,
    ValidWithWarnings,
    Invalid,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimescaleDerivation {
    pub equation: String,
    pub circuit_path: Option<String>,
    pub convention: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MechanismWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacteristicTimescale {
    pub timescale_id: String,
    pub source: TimescaleSource,
    pub label: String,
    pub value_s: f64,
    pub standard_error_s: Option<f64>,
    pub confidence_interval_s: Option<(f64, f64)>,
    pub derivation: TimescaleDerivation,
    pub source_parameters: Vec<String>,
    pub semantic_role: Option<String>,
    pub validity: TimescaleValidity,
    pub warnings: Vec<MechanismWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    None,
    Weak,
    Moderate,
    Strong,
    Contradictory,
    Insufficient,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimescaleComparison {
    pub comparison_id: String,
    pub record_id: String,
    pub eis_timescale_id: String,
    pub transient_timescale_id: String,
    pub ratio: Option<f64>,
    pub log10_distance: Option<f64>,
    pub symmetric_relative_difference: Option<f64>,
    pub confidence_interval_overlap: Option<bool>,
    pub compatibility_probability: Option<f64>,
    pub evidence_level: EvidenceLevel,
    pub supporting_evidence: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub assumptions: Vec<String>,
    pub alternative_explanations: Vec<String>,
    pub warnings: Vec<MechanismWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanismRecordSummary {
    pub record_id: String,
    pub experiment_id: Option<String>,
    pub sensor_id: Option<String>,
    pub condition: Option<String>,
    pub sensor_age_days: Option<f64>,
    pub metadata: BTreeMap<String, String>,
    pub calibration_context_available: bool,
    pub warnings: Vec<MechanismWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HypothesisAssessment {
    pub hypothesis_id: String,
    pub transient_timescale: String,
    pub eis_role: String,
    pub description: String,
    pub assessment: String,
    pub supporting_observations: Vec<String>,
    pub contradictory_observations: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub assumptions: Vec<String>,
    pub alternative_explanations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanismTrendResult {
    pub variable: String,
    pub independent_variable: String,
    pub records: usize,
    pub absolute_change: Option<f64>,
    pub relative_change: Option<f64>,
    pub log_change: Option<f64>,
    pub slope: Option<f64>,
    pub robust_slope: Option<f64>,
    pub rank_correlation: Option<f64>,
    pub replicate_variability: Option<f64>,
    pub warnings: Vec<MechanismWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanismRecordInput {
    pub record_id: String,
    pub experiment_id: Option<String>,
    pub sensor_id: Option<String>,
    pub eis_fit: String,
    pub transient_results: String,
    pub calibration_results: Option<String>,
    pub metadata: Option<String>,
    pub condition: Option<String>,
    pub sensor_age_days: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanismHypothesis {
    pub hypothesis_id: String,
    pub transient_timescale: String,
    pub eis_role: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanismManifest {
    pub schema_version: u32,
    pub records: Vec<MechanismRecordInput>,
    #[serde(default)]
    pub hypotheses: Vec<MechanismHypothesis>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedMechanismConfig {
    pub schema_version: u32,
    pub confidence_level: f64,
    pub allow_warning_fits: bool,
    pub ratio_weak: f64,
    pub ratio_moderate: f64,
    pub ratio_strong: f64,
    pub log_distance_weak: f64,
    pub log_distance_moderate: f64,
    pub log_distance_strong: f64,
    pub minimum_fit_quality: f64,
    pub compatibility_ratio_lower: f64,
    pub compatibility_ratio_upper: f64,
    pub require_experiment_id: bool,
    pub require_sensor_id: bool,
    pub minimum_replicates_for_strong: usize,
    pub trend_minimum_records: usize,
    pub trend_independent_variable: String,
    pub frequency_boundary_margin: f64,
    pub monte_carlo_samples: usize,
    pub seed: u64,
    pub selected_model_only: bool,
    pub hypotheses: Vec<MechanismHypothesis>,
}

impl Default for ResolvedMechanismConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            confidence_level: 0.95,
            allow_warning_fits: true,
            ratio_weak: 10.0,
            ratio_moderate: 3.0,
            ratio_strong: 1.5,
            log_distance_weak: 1.0,
            log_distance_moderate: 0.5,
            log_distance_strong: 0.1761,
            minimum_fit_quality: 0.0,
            compatibility_ratio_lower: 0.5,
            compatibility_ratio_upper: 2.0,
            require_experiment_id: true,
            require_sensor_id: false,
            minimum_replicates_for_strong: 3,
            trend_minimum_records: 3,
            trend_independent_variable: "sensor_age_days".to_string(),
            frequency_boundary_margin: 0.1,
            monte_carlo_samples: 10_000,
            seed: 42,
            selected_model_only: true,
            hypotheses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanismAnalysisReport {
    pub schema_version: u32,
    pub analysis_id: String,
    pub records: Vec<MechanismRecordSummary>,
    pub eis_timescales: Vec<CharacteristicTimescale>,
    pub transient_timescales: Vec<CharacteristicTimescale>,
    pub comparisons: Vec<TimescaleComparison>,
    pub hypotheses: Vec<HypothesisAssessment>,
    pub trends: Vec<MechanismTrendResult>,
    pub configuration: ResolvedMechanismConfig,
    pub provenance: Option<AnalysisProvenance>,
    pub warnings: Vec<MechanismWarning>,
    #[serde(default)]
    pub transient_configuration: Option<ResolvedTransientConfig>,
}
