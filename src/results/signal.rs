//! Durable, schema-versioned signal-quality artifacts.

use crate::{domain::AnalysisProvenance, signal_config::ResolvedSignalConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalWindowSource {
    EntireMeasurement,
    ExplicitInterval,
    EventRelative,
    StableExperimentRegion,
    ResidualArtifact,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalWindowSummary {
    pub source: SignalWindowSource,
    pub start_s: Option<f64>,
    pub end_s: Option<f64>,
    pub source_observation_count: usize,
    pub source_timestamps: Vec<f64>,
    pub selected_observation_count: usize,
    pub excluded_observations: usize,
    pub missing_observations: usize,
    pub excluded_intervals: Vec<(f64, f64)>,
    pub resampling_method: Option<String>,
    pub detrending_method: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamplingAnalysis {
    pub sample_count: usize,
    pub finite_sample_count: usize,
    pub missing_fraction: Option<f64>,
    pub start_time_s: Option<f64>,
    pub end_time_s: Option<f64>,
    pub duration_s: Option<f64>,
    pub median_interval_s: Option<f64>,
    pub mean_interval_s: Option<f64>,
    pub interval_stddev_s: Option<f64>,
    pub interval_cv: Option<f64>,
    pub minimum_interval_s: Option<f64>,
    pub maximum_interval_s: Option<f64>,
    pub duplicate_timestamps: usize,
    pub non_monotonic_timestamps: usize,
    pub effective_frequency_hz: Option<f64>,
    pub is_regular: bool,
    pub target_interval_s: Option<f64>,
    pub interpolation_count: usize,
    pub interpolation_gap_exceeded: bool,
    pub interpolated_indices: Vec<usize>,
    #[serde(default)]
    pub output_missing_indices: Vec<usize>,
    #[serde(default)]
    pub sorted_rows: usize,
    #[serde(default)]
    pub resolved_duplicate_groups: usize,
    #[serde(default)]
    pub transformations: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quantile {
    pub probability: f64,
    pub value: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DescriptiveStatistics {
    pub count: usize,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub standard_deviation: Option<f64>,
    pub sample_variance: Option<f64>,
    pub rms: Option<f64>,
    pub peak_to_peak: Option<f64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub quantiles: Vec<Quantile>,
    pub median_absolute_deviation: Option<f64>,
    pub robust_standard_deviation: Option<f64>,
    pub interquartile_range: Option<f64>,
    pub skewness: Option<f64>,
    pub excess_kurtosis: Option<f64>,
    pub confidence_interval: Option<(f64, f64)>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PsdPeak {
    pub frequency_hz: f64,
    pub psd: f64,
    pub interpretation: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PsdBandPower {
    pub name: String,
    pub minimum_hz: f64,
    pub maximum_hz: f64,
    pub integrated_power: Option<f64>,
    pub fraction: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PsdAnalysis {
    pub frequency_hz: Vec<f64>,
    pub psd_unit: String,
    pub psd: Vec<f64>,
    pub amplitude_spectral_density: Vec<f64>,
    pub segment_points: usize,
    pub segment_count: usize,
    pub overlap_fraction: f64,
    pub frequency_resolution_hz: Option<f64>,
    pub nyquist_hz: Option<f64>,
    pub dominant_peaks: Vec<PsdPeak>,
    pub total_integrated_power: Option<f64>,
    pub band_powers: Vec<PsdBandPower>,
    pub spectral_centroid_hz: Option<f64>,
    pub spectral_rolloff_hz: Option<f64>,
    pub parseval_time_variance: Option<f64>,
    pub parseval_integrated_power: Option<f64>,
    pub parseval_relative_error: Option<f64>,
    pub warnings: Vec<SignalWarning>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllanPoint {
    pub averaging_time_s: f64,
    pub deviation: Option<f64>,
    pub effective_differences: usize,
    pub approximate_uncertainty: Option<f64>,
    pub log_log_slope: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllanAnalysis {
    pub points: Vec<AllanPoint>,
    pub minimum_deviation: Option<f64>,
    pub minimum_averaging_time_s: Option<f64>,
    pub warnings: Vec<SignalWarning>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DriftModelKind {
    OrdinaryLinear,
    WeightedLinear,
    TheilSen,
    EventSegmentedLinear,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriftAnalysis {
    pub model: DriftModelKind,
    pub slope_v_per_s: Option<f64>,
    pub slope_mv_per_h: Option<f64>,
    pub slope_mv_per_day: Option<f64>,
    pub intercept_v: Option<f64>,
    pub standard_error: Option<f64>,
    pub confidence_interval: Option<(f64, f64)>,
    pub r_squared: Option<f64>,
    pub robust_residual_scale: Option<f64>,
    pub observations: usize,
    pub duration_s: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpikeFlag {
    pub index: usize,
    pub timestamp_s: f64,
    pub value: f64,
    pub local_median: Option<f64>,
    pub local_mad: Option<f64>,
    pub normalized_deviation: Option<f64>,
    pub sustained_step: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpikeAnalysis {
    pub method: String,
    pub flagged: Vec<SpikeFlag>,
    pub flagged_fraction: Option<f64>,
    pub maximum_flagged_fraction: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelCorrelationResult {
    pub channel_a: String,
    pub channel_b: String,
    pub observations: usize,
    pub pearson: Option<f64>,
    pub spearman: Option<f64>,
    pub covariance: Option<f64>,
    pub lags_s: Vec<f64>,
    pub cross_correlation: Vec<f64>,
    pub lag_of_max_absolute_correlation_s: Option<f64>,
    pub common_mode_fraction: Option<f64>,
    pub channel_specific_residual_scale_a: Option<f64>,
    pub channel_specific_residual_scale_b: Option<f64>,
    pub warning: Option<SignalWarning>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResidualSummary {
    pub mean: Option<f64>,
    pub standard_deviation: Option<f64>,
    pub rmse: Option<f64>,
    pub lag1_autocorrelation: Option<f64>,
    pub durbin_watson: Option<f64>,
    pub runs_statistic: Option<f64>,
    pub drift: Option<DriftAnalysis>,
    pub spike_fraction: Option<f64>,
    pub autocorrelation: Vec<f64>,
    pub psd: Option<PsdAnalysis>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EisResidualSummary {
    pub observations: usize,
    pub real_mean: Option<f64>,
    pub imaginary_mean: Option<f64>,
    pub magnitude_mean: Option<f64>,
    pub low_frequency_bias: Option<f64>,
    pub high_frequency_bias: Option<f64>,
    pub systematic_sign_runs: usize,
    pub frequency_dependent_magnitude: Vec<(f64, f64)>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ResidualAnalysisResult {
    TimeDomain {
        source: String,
        summary: ResidualSummary,
    },
    Eis {
        source: String,
        summary: EisResidualSummary,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalComparisonRecord {
    pub record_id: String,
    pub category: String,
    pub channel: String,
    pub count: usize,
    pub mean: Option<f64>,
    pub standard_deviation: Option<f64>,
    pub robust_standard_deviation: Option<f64>,
    pub drift_slope_v_per_s: Option<f64>,
    pub spike_fraction: Option<f64>,
    pub warnings: Vec<SignalWarning>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalWarning {
    IrregularSampling,
    DuplicateTimestamps,
    NonMonotonicTimestamps,
    ExcessiveMissingData,
    RecordTooShort,
    ResamplingPerformed,
    InterpolationGapExceeded,
    PoorPsdResolution,
    InsufficientWelchSegments,
    PoorParsevalAgreement,
    InsufficientAllanClusters,
    DriftDurationInsufficient,
    ExcessiveSpikeFraction,
    CorrelationSampleCountInsufficient,
    ResidualArtifactIncompatible,
    InvalidConfiguration,
    EmptyWindow,
    NonFiniteTimestamp,
    TimestampPolicyApplied,
    DuplicatePolicyApplied,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalAnalysisReport {
    pub schema_version: u32,
    pub analysis_id: String,
    pub experiment_id: Option<String>,
    pub sensor_id: Option<String>,
    pub channel: String,
    pub unit: String,
    pub analysis_timestamps: Vec<f64>,
    pub analysis_values: Vec<Option<f64>>,
    pub window: SignalWindowSummary,
    pub sampling: SamplingAnalysis,
    pub descriptive: DescriptiveStatistics,
    pub psd: Option<PsdAnalysis>,
    pub allan: Option<AllanAnalysis>,
    pub drift: Vec<DriftAnalysis>,
    pub spikes: SpikeAnalysis,
    pub correlations: Vec<ChannelCorrelationResult>,
    pub residual_analysis: Vec<ResidualAnalysisResult>,
    pub configuration: ResolvedSignalConfig,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<SignalWarning>,
}
