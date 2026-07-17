//! Stable serializable calibration observations, models, validation, and predictions.

use crate::calibration_config::ResolvedCalibrationConfig;
use crate::domain::AnalysisProvenance;
use crate::potentiometry::units::{Quantity, QuantityUnit};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationBranch {
    Ascending,
    Descending,
    Mixed,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationPotentialSource {
    TransientEquilibrium,
    #[serde(alias = "steady_state_mean")]
    SteadyStateWindowMean,
    #[serde(alias = "steady_state_median")]
    SteadyStateWindowMedian,
    ExplicitObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActivityModelKind {
    #[default]
    Ideal,
    Davies,
    ExtendedDebyeHuckel,
    ConductivityEmpirical,
    UserProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NernstSlopeMode {
    #[default]
    Free,
    FixedTheoretical,
    PriorConstrained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureMode {
    Constant,
    #[default]
    ObservationSpecific,
    ReferenceNormalized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseSign {
    #[default]
    Auto,
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationModelKind {
    #[default]
    Nernst,
    NicolskyEisenman,
    ConductivityEmpirical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationFitStatus {
    Converged,
    #[default]
    Failed,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationSelectionCriterion {
    Aic,
    #[default]
    Aicc,
    Bic,
    CrossValidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CrossValidationMode {
    #[default]
    None,
    LeaveOneOut,
    LeaveOneConcentrationLevelOut,
    KFold,
    LeaveOneExperimentOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentalAlignment {
    Nearest,
    #[default]
    LinearInterpolation,
    WindowMean,
    WindowMedian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WeightingMode {
    #[default]
    Uniform,
    PotentialStandardError,
    UserProvided,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalAlignmentRecord {
    pub source_series: String,
    pub alignment: EnvironmentalAlignment,
    pub source_timestamps: Vec<f64>,
    pub interpolated: bool,
    pub time_gap_s: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteadyStateSummary {
    pub window_start_s: f64,
    pub window_end_s: f64,
    pub finite_points: usize,
    pub mean_v: Option<f64>,
    pub median_v: Option<f64>,
    pub standard_deviation_v: Option<f64>,
    pub standard_error_v: Option<f64>,
    pub linear_slope_v_per_s: Option<f64>,
    pub missing_fraction: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationObservation {
    pub observation_id: String,
    pub experiment_id: String,
    pub event_index: Option<usize>,
    pub timestamp: Option<f64>,
    pub analyte: String,
    pub ion_charge: i32,
    pub concentration: Option<Quantity>,
    pub molar_concentration_mol_l: Option<f64>,
    pub activity: Option<f64>,
    pub activity_coefficient: Option<f64>,
    pub potential_v: f64,
    pub potential_standard_error_v: Option<f64>,
    pub temperature_k: Option<f64>,
    pub ionic_strength_mol_l: Option<f64>,
    pub conductivity: Option<Quantity>,
    pub interferent_activities: BTreeMap<String, f64>,
    pub branch: CalibrationBranch,
    pub source: CalibrationPotentialSource,
    pub source_fit_status: Option<String>,
    pub source_warnings: Vec<String>,
    pub steady_state: Option<SteadyStateSummary>,
    #[serde(default)]
    pub environmental_alignment: Vec<EnvironmentalAlignmentRecord>,
    pub metadata: BTreeMap<String, String>,
}

impl CalibrationObservation {
    pub fn log10_activity(&self) -> Option<f64> {
        self.activity
            .filter(|value| value.is_finite() && *value > 0.0)
            .map(f64::log10)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationObservationSet {
    pub schema_version: u32,
    pub observations: Vec<CalibrationObservation>,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<CalibrationWarning>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationObservationSummary {
    pub total_observations: usize,
    pub ascending_observations: usize,
    pub descending_observations: usize,
    pub unknown_branch_observations: usize,
    pub concentration_levels: usize,
    pub experiments: usize,
    pub finite_activities: usize,
    pub potential_range_v: Option<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationWarning {
    pub kind: CalibrationWarningKind,
    pub message: String,
    pub observation_id: Option<String>,
}

impl CalibrationWarning {
    pub fn new(kind: CalibrationWarningKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            observation_id: None,
        }
    }

    pub fn for_observation(
        kind: CalibrationWarningKind,
        message: impl Into<String>,
        observation_id: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            observation_id: Some(observation_id.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationWarningKind {
    MissingConcentration,
    MissingMolarMass,
    UnknownUnit,
    NonpositiveConcentration,
    NonpositiveActivity,
    MissingTemperature,
    NonphysicalTemperature,
    MissingIonicStrength,
    ActivityValidityExceeded,
    MissingIonSizeParameter,
    MissingConductivity,
    ConductivityExtrapolation,
    TransientEquilibriumUnavailable,
    TransientFitWarning,
    SteadyStateUnstable,
    InsufficientConcentrationLevels,
    LimitedActivityRange,
    NonNernstianSlope,
    SlopeSignInconsistent,
    HighHysteresis,
    InfluentialObservation,
    HighLeverage,
    PoorConditionNumber,
    SingularCovariance,
    PoorCrossValidation,
    PredictionExtrapolation,
    NicolskyNonIdentifiable,
    SelectivityCoefficientAtBound,
    BootstrapUnavailable,
    EmpiricalConductivityCorrection,
    UnsupportedPriorSlope,
    ComparisonDifferentObservationSets,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationParameter {
    pub name: String,
    pub unit: String,
    pub value: f64,
    pub standard_error: Option<f64>,
    pub lower_bound: Option<f64>,
    pub upper_bound: Option<f64>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationConfidenceInterval {
    pub parameter: String,
    pub unit: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub confidence_level: f64,
    pub successful_iterations: usize,
    pub failed_iterations: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationFitStatistics {
    pub observations: usize,
    pub fitted_parameters: usize,
    pub rss: Option<f64>,
    pub weighted_rss: Option<f64>,
    pub rmse_v: Option<f64>,
    pub mae_v: Option<f64>,
    pub r_squared: Option<f64>,
    pub adjusted_r_squared: Option<f64>,
    pub aic: Option<f64>,
    pub aicc: Option<f64>,
    pub bic: Option<f64>,
    pub criterion_delta: Option<f64>,
    pub model_weight: Option<f64>,
    pub parameter_covariance: Option<Vec<Vec<f64>>>,
    pub condition_number: Option<f64>,
    pub durbin_watson: Option<f64>,
    pub leverage: Vec<f64>,
    pub cooks_distance: Vec<f64>,
    pub convergence_reason: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationDomain {
    pub log10_activity_min: Option<f64>,
    pub log10_activity_max: Option<f64>,
    pub molar_concentration_min: Option<f64>,
    pub molar_concentration_max: Option<f64>,
    pub temperature_min_k: Option<f64>,
    pub temperature_max_k: Option<f64>,
    pub conductivity_min_s_per_m: Option<f64>,
    pub conductivity_max_s_per_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectivityCoefficient {
    pub primary_analyte: String,
    pub interferent: String,
    pub value: f64,
    pub source: String,
    pub standard_error: Option<f64>,
    pub confidence_interval: Option<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationModelResult {
    pub model_kind: CalibrationModelKind,
    pub status: CalibrationFitStatus,
    pub activity_model: ActivityModelKind,
    pub parameters: Vec<CalibrationParameter>,
    pub selectivity_coefficients: Vec<SelectivityCoefficient>,
    pub equation: String,
    pub theoretical_slope_v_per_decade: Option<f64>,
    pub fitted_slope_v_per_decade: Option<f64>,
    pub slope_efficiency: Option<f64>,
    pub statistics: CalibrationFitStatistics,
    pub predicted_potential_v: Vec<f64>,
    pub residuals_v: Vec<f64>,
    pub standardized_residuals: Vec<Option<f64>>,
    pub confidence_intervals: Vec<CalibrationConfidenceInterval>,
    pub valid_domain: CalibrationDomain,
    pub warnings: Vec<CalibrationWarning>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HysteresisResult {
    pub matching_tolerance_log10_activity: f64,
    pub paired_observations: usize,
    pub mean_hysteresis_v: Option<f64>,
    pub median_hysteresis_v: Option<f64>,
    pub maximum_absolute_hysteresis_v: Option<f64>,
    pub activity_specific_hysteresis: Vec<(f64, f64)>,
    pub warnings: Vec<CalibrationWarning>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ValidationFoldResult {
    pub fold_id: String,
    pub held_out_observations: usize,
    pub failed_predictions: usize,
    pub extrapolation_count: usize,
    pub rmse_potential_v: Option<f64>,
    pub mae_potential_v: Option<f64>,
    pub rmse_log10_activity: Option<f64>,
    pub mae_log10_activity: Option<f64>,
    pub prediction_bias_v: Option<f64>,
    pub coverage: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationPredictionPoint {
    pub observation_id: String,
    pub observed_potential_v: f64,
    pub predicted_potential_v: Option<f64>,
    pub observed_log10_activity: Option<f64>,
    pub predicted_log10_activity: Option<f64>,
    pub extrapolated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationValidationResult {
    pub mode: CrossValidationMode,
    pub folds: Vec<ValidationFoldResult>,
    pub rmse_potential_v: Option<f64>,
    pub mae_potential_v: Option<f64>,
    pub rmse_log10_activity: Option<f64>,
    pub mae_log10_activity: Option<f64>,
    pub concentration_relative_error: Option<f64>,
    pub prediction_bias_v: Option<f64>,
    pub interval_coverage: Option<f64>,
    pub failed_predictions: usize,
    pub extrapolation_count: usize,
    #[serde(default)]
    pub predictions: Vec<ValidationPredictionPoint>,
    pub warnings: Vec<CalibrationWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationAnalysisReport {
    pub schema_version: u32,
    pub calibration_id: String,
    pub analyte: String,
    pub ion_charge: i32,
    pub source_experiments: Vec<String>,
    pub observation_summary: CalibrationObservationSummary,
    pub configuration: ResolvedCalibrationConfig,
    pub candidate_models: Vec<CalibrationModelResult>,
    pub selected_model: Option<CalibrationModelKind>,
    pub hysteresis: Option<HysteresisResult>,
    pub validation: Option<CalibrationValidationResult>,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<CalibrationWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredCalibrationModel {
    pub schema_version: u32,
    pub analyte: String,
    pub ion_charge: i32,
    pub model_kind: CalibrationModelKind,
    pub activity_model: ActivityModelKind,
    pub temperature_mode: TemperatureMode,
    pub slope_mode: NernstSlopeMode,
    pub response_sign: ResponseSign,
    pub parameters: Vec<CalibrationParameter>,
    pub selectivity_coefficients: Vec<SelectivityCoefficient>,
    pub valid_domain: CalibrationDomain,
    pub training_statistics: CalibrationFitStatistics,
    pub configuration: ResolvedCalibrationConfig,
    pub provenance: AnalysisProvenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationPrediction {
    pub potential_v: Option<f64>,
    pub potential_standard_error_v: Option<f64>,
    pub predicted_activity: Option<f64>,
    pub predicted_activity_lower: Option<f64>,
    pub predicted_activity_upper: Option<f64>,
    pub predicted_molar_concentration_mol_l: Option<f64>,
    pub predicted_concentration_unit: Option<QuantityUnit>,
    pub temperature_k: Option<f64>,
    pub extrapolated: bool,
    pub distance_from_domain_log10_activity: Option<f64>,
    pub warnings: Vec<CalibrationWarning>,
}
