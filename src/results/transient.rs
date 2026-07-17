//! Stable serializable result structures for potentiometric transient fits.

use crate::domain::{AnalysisProvenance, ExperimentEvent, ParseDiagnostics};
use crate::potentiometry::transient::models::{BaselineMethod, ResponseMode, TransientModelKind};
use crate::transient_config::ResolvedTransientConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitStatus {
    Converged,
    Failed,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransientWarningKind {
    IrregularSampling,
    DuplicateTimestamps,
    NonMonotonicTimestamps,
    BaselineUnavailable,
    LongTimeConstant,
    PoorTauSeparation,
    NegligibleAmplitude,
    ParameterAtBound,
    SingularCovariance,
    HighResidualAutocorrelation,
    NotIdentifiable,
    BootstrapUnavailable,
    AllModelsFailed,
    AiccUnavailable,
    EventOutsideRange,
    OptimizerFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransientWarning {
    pub kind: TransientWarningKind,
    pub message: String,
}

impl TransientWarning {
    pub fn new(kind: TransientWarningKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConcentrationContext {
    pub value: f64,
    pub unit: Option<String>,
    pub analyte: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SegmentSummary {
    pub segment_start: Option<f64>,
    pub segment_end: Option<f64>,
    pub local_start: Option<f64>,
    pub local_end: Option<f64>,
    pub finite_duration_s: Option<f64>,
    pub raw_observations: usize,
    pub finite_fitted_observations: usize,
    pub missing_observations: usize,
    pub missing_fraction: Option<f64>,
    pub irregular_sampling: bool,
    pub duplicate_timestamps: usize,
    pub non_monotonic_timestamps: usize,
    pub raw_time_local: Vec<f64>,
    pub raw_potential_v: Vec<Option<f64>>,
    pub fitted_time_local: Vec<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BaselineResult {
    pub method: BaselineMethod,
    pub response_mode: ResponseMode,
    pub estimate_v: Option<f64>,
    pub slope_v_per_s: Option<f64>,
    pub finite_points: usize,
    pub time_local: Vec<f64>,
    pub potential_v: Vec<f64>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedTransientParameter {
    pub name: String,
    pub unit: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterConfidenceInterval {
    pub name: String,
    pub unit: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub confidence_level: f64,
    pub successful_iterations: usize,
    pub failed_iterations: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransientFeatures {
    pub event_timestamp: Option<f64>,
    pub segment_start: Option<f64>,
    pub segment_end: Option<f64>,
    pub raw_observations: usize,
    pub finite_fitted_observations: usize,
    pub missing_fraction: Option<f64>,
    pub baseline_estimate_v: Option<f64>,
    pub initial_measured_potential_v: Option<f64>,
    pub fitted_equilibrium_potential_v: Option<f64>,
    pub total_response_amplitude_v: Option<f64>,
    pub fast_amplitude_v: Option<f64>,
    pub slow_amplitude_v: Option<f64>,
    pub tau_fast_s: Option<f64>,
    pub tau_slow_s: Option<f64>,
    pub stretched_beta: Option<f64>,
    pub drift_rate_v_per_s: Option<f64>,
    pub initial_response_rate_v_per_s: Option<f64>,
    pub time_to_63_2_percent_s: Option<f64>,
    pub time_to_90_percent_s: Option<f64>,
    pub time_to_95_percent_s: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransientFitStatistics {
    pub rmse_v: Option<f64>,
    pub mae_v: Option<f64>,
    pub r_squared: Option<f64>,
    pub adjusted_r_squared: Option<f64>,
    pub rss: Option<f64>,
    pub aic: Option<f64>,
    pub aicc: Option<f64>,
    pub bic: Option<f64>,
    pub durbin_watson: Option<f64>,
    pub lag1_residual_autocorrelation: Option<f64>,
    pub maximum_absolute_residual_v: Option<f64>,
    pub criterion_delta: Option<f64>,
    pub model_weight: Option<f64>,
    pub convergence_status: String,
    pub optimizer_termination_reason: Option<String>,
    pub covariance_condition_number: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransientFitResult {
    pub model: TransientModelKind,
    pub status: FitStatus,
    pub parameters: Vec<FittedTransientParameter>,
    pub derived_features: TransientFeatures,
    pub statistics: TransientFitStatistics,
    pub confidence_intervals: Vec<ParameterConfidenceInterval>,
    pub predicted_v: Vec<f64>,
    pub residuals_v: Vec<f64>,
    pub warnings: Vec<TransientWarning>,
    #[serde(skip)]
    pub(crate) fit_parameters: Vec<f64>,
    #[serde(skip)]
    pub(crate) response_offset: f64,
}

impl TransientFitResult {
    pub fn failed(model: TransientModelKind, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            model,
            status: FitStatus::Failed,
            parameters: Vec::new(),
            derived_features: TransientFeatures::default(),
            statistics: TransientFitStatistics {
                convergence_status: "failed".to_string(),
                optimizer_termination_reason: Some(reason.clone()),
                ..TransientFitStatistics::default()
            },
            confidence_intervals: Vec::new(),
            predicted_v: Vec::new(),
            residuals_v: Vec::new(),
            warnings: vec![TransientWarning::new(
                TransientWarningKind::OptimizerFailure,
                reason,
            )],
            fit_parameters: Vec::new(),
            response_offset: 0.0,
        }
    }

    pub fn is_successful(&self) -> bool {
        self.status == FitStatus::Converged
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransientFitFailure {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransientEventResult {
    pub event_index: usize,
    pub event: ExperimentEvent,
    pub concentration_before: Option<ConcentrationContext>,
    pub concentration_after: Option<ConcentrationContext>,
    pub segment: SegmentSummary,
    pub baseline: BaselineResult,
    pub candidate_fits: Vec<TransientFitResult>,
    pub selected_model: Option<TransientModelKind>,
    pub warnings: Vec<TransientWarning>,
    pub failure: Option<TransientFitFailure>,
}

impl TransientEventResult {
    pub fn failed(
        event_index: usize,
        event: ExperimentEvent,
        concentration_before: Option<ConcentrationContext>,
        concentration_after: Option<ConcentrationContext>,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        Self {
            event_index,
            event,
            concentration_before,
            concentration_after,
            segment: SegmentSummary::default(),
            baseline: BaselineResult::default(),
            candidate_fits: Vec::new(),
            selected_model: None,
            warnings: vec![TransientWarning::new(
                TransientWarningKind::EventOutsideRange,
                message.clone(),
            )],
            failure: Some(TransientFitFailure { message }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransientAnalysisReport {
    pub schema_version: u32,
    pub experiment_id: String,
    pub channel: String,
    pub channel_unit: String,
    pub parse_diagnostics: ParseDiagnostics,
    pub configuration: ResolvedTransientConfig,
    pub provenance: AnalysisProvenance,
    pub events: Vec<TransientEventResult>,
}
