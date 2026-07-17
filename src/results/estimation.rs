//! Durable state-estimation artifacts.  These types intentionally distinguish
//! measured voltage, predicted voltage, innovations, and latent state values.

use crate::{
    domain::AnalysisProvenance,
    estimation::AuxiliaryObservation,
    estimation::state::{
        CalibrationDomainStatus, EstimationWarning, MeasurementUpdateStatus, StateDefinition,
    },
    estimation::{
        covariance::CovarianceResolution, environment::AlignedEnvironmentSummary,
        initialization::InitializationReport, innovation::InnovationRecord,
        observability::ObservabilityReport,
    },
    estimation_config::{FilterKind, ResolvedEstimationConfig, StateModelKind},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateValue {
    pub name: String,
    pub value: Option<f64>,
    pub standard_error: Option<f64>,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub unit: String,
    pub latent: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateEstimatePoint {
    pub timestamp_s: f64,
    pub measurement_v: Option<f64>,
    pub predicted_measurement_v: Option<f64>,
    pub innovation_v: Option<f64>,
    pub innovation_variance_v2: Option<f64>,
    pub standardized_innovation: Option<f64>,
    pub normalized_innovation_squared: Option<f64>,
    pub update_status: MeasurementUpdateStatus,
    pub filtered_state: Vec<StateValue>,
    pub predicted_state: Vec<StateValue>,
    pub filtered_covariance: Vec<Vec<f64>>,
    pub predicted_covariance: Vec<Vec<f64>>,
    pub calibration_domain_status: CalibrationDomainStatus,
    pub domain_distance: Option<f64>,
    pub environmental_context: AlignedEnvironmentSummary,
    pub activity: Option<f64>,
    pub activity_standard_error: Option<f64>,
    pub molar_concentration_mol_l: Option<f64>,
    pub concentration_unit: Option<String>,
    pub concentration_assumptions: Option<String>,
    pub auxiliary_observations: Vec<AuxiliaryObservation>,
    pub warnings: Vec<EstimationWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FilterDiagnostics {
    pub innovation_mean: Option<f64>,
    pub innovation_standard_deviation: Option<f64>,
    pub nis_mean: Option<f64>,
    pub nis_exceedance_rate: Option<f64>,
    pub accepted_update_count: usize,
    pub rejected_update_count: usize,
    pub predict_only_count: usize,
    pub log_likelihood: Option<f64>,
    pub residual_autocorrelation: Option<f64>,
    pub numerical_failures: usize,
    pub covariance_jitter_count: usize,
    pub domain_excursion_count: usize,
    pub innovations: Vec<InnovationRecord>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateMetric {
    pub state: String,
    pub unit: String,
    pub rmse: Option<f64>,
    pub mae: Option<f64>,
    pub bias: Option<f64>,
    pub interval_coverage: Option<f64>,
    pub nees_mean: Option<f64>,
    pub convergence_time_s: Option<f64>,
    pub step_response_delay_s: Option<f64>,
    pub maximum_transient_error: Option<f64>,
    pub outlier_rejection_rate: Option<f64>,
    pub calibration_domain_violations: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StateValidationResult {
    pub truth_source: Option<String>,
    pub metrics: Vec<StateMetric>,
    pub vector_nees_mean: Option<f64>,
    pub vector_nees_count: usize,
    #[serde(default)]
    pub matched_sample_count: usize,
    #[serde(default)]
    pub alignment_tolerance_s: Option<f64>,
    #[serde(default)]
    pub unmatched_estimate_timestamps_s: Vec<f64>,
    #[serde(default)]
    pub unmatched_truth_timestamps_s: Vec<f64>,
    pub warnings: Vec<EstimationWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterComparisonRecord {
    pub filter: FilterKind,
    pub runtime_ms: f64,
    pub activity_rmse: Option<f64>,
    pub innovation_mean: Option<f64>,
    pub nis_mean: Option<f64>,
    pub rejected_updates: usize,
    pub numerical_failures: usize,
    pub domain_excursions: usize,
    pub mean_activity_standard_error: Option<f64>,
    pub warnings: Vec<EstimationWarning>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateFilterComparison {
    pub schema_version: u32,
    pub records: Vec<FilterComparisonRecord>,
    pub warnings: Vec<EstimationWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateEstimationReport {
    pub schema_version: u32,
    pub analysis_id: String,
    pub experiment_id: String,
    pub sensor_id: Option<String>,
    pub channel: String,
    #[serde(default)]
    pub measurement_source_unit: String,
    #[serde(default)]
    pub measurement_conversion: String,
    pub filter: FilterKind,
    pub model: StateModelKind,
    pub state_definitions: Vec<StateDefinition>,
    pub initialization: InitializationReport,
    pub process_covariance: CovarianceResolution,
    pub measurement_covariance: CovarianceResolution,
    pub observability: ObservabilityReport,
    pub estimates: Vec<StateEstimatePoint>,
    pub diagnostics: FilterDiagnostics,
    pub validation: Option<StateValidationResult>,
    pub configuration: ResolvedEstimationConfig,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<EstimationWarning>,
}

pub fn finite_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let text = serde_json::to_string_pretty(value)?;
    if text.contains("NaN") || text.contains("Infinity") || text.contains("-Infinity") {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "nonfinite value in serialized estimation artifact",
        )));
    }
    Ok(text)
}
