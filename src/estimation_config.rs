//! Configuration for offline, physics-informed state estimation.

use crate::domain::ConfigurationError;
use crate::estimation::timestamp::TimestampHandlingConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub const ESTIMATION_CONFIG_SCHEMA_VERSION: u32 = 3;
pub const DEFAULT_ESTIMATION_CONFIG_PATH: &str = "config/estimation.toml";

#[derive(Debug, Clone)]
pub struct LoadedEstimationConfig {
    pub config: ResolvedEstimationConfig,
    pub source_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FilterKind {
    Ekf,
    #[default]
    Ukf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StateModelKind {
    Activity,
    ActivityBaseline,
    #[default]
    ActivityBaselinePolarization,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StateTransformKind {
    #[default]
    IdentityLog10,
    Log10Positive,
    LogPositive,
    LogisticBounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CovarianceSourceKind {
    #[default]
    Configured,
    SignalArtifact,
    CalibrationArtifact,
    TransientArtifact,
    EstimatedFromTrainingData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentKind {
    Nearest,
    #[default]
    LinearInterpolation,
    WindowMean,
    WindowMedian,
    HoldPrevious,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TauSourceKind {
    #[default]
    Transient,
    Configured,
    Mechanism,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AggregationKind {
    #[default]
    Median,
    Mean,
    First,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PolarizationInputModel {
    /// Do not create an input from event data.
    #[default]
    None,
    /// Read a one-shot voltage impulse from event metadata.
    ExplicitEventVoltage,
    /// Convert a standard activity step into a configured voltage impulse.
    ActivityStepGain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementNoiseSourceKind {
    #[default]
    Configured,
    PerObservation,
    SignalRobustVariance,
    StableWindowVariance,
    CalibrationResidualVariance,
    CalibrationPredictionUncertainty,
}

/// Policy used to align a validation estimate to an independently generated
/// truth trajectory.  Truth is never silently sorted or reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TruthAlignmentPolicy {
    Exact,
    #[default]
    NearestWithinTolerance,
    LinearInterpolation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResolvedEstimationConfig {
    pub schema_version: u32,
    pub filter: FilterConfig,
    pub state_model: StateModelConfig,
    pub initialization: InitializationConfig,
    pub initial_covariance: InitialCovarianceConfig,
    pub process_noise: ProcessNoiseConfig,
    pub measurement_noise: MeasurementNoiseConfig,
    pub polarization: PolarizationConfig,
    pub environment: EnvironmentConfig,
    pub observability: ObservabilityConfig,
    pub ekf: EkfConfig,
    pub ukf: UkfConfig,
    pub extrapolation: ExtrapolationConfig,
    pub validation: ValidationConfig,
    pub auxiliary: AuxiliaryConfig,
    pub plotting: EstimationPlottingConfig,
    pub export: EstimationExportConfig,
    pub timestamp_handling: TimestampHandlingConfig,
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

impl Default for ResolvedEstimationConfig {
    fn default() -> Self {
        Self {
            schema_version: ESTIMATION_CONFIG_SCHEMA_VERSION,
            filter: FilterConfig::default(),
            state_model: StateModelConfig::default(),
            initialization: InitializationConfig::default(),
            initial_covariance: InitialCovarianceConfig::default(),
            process_noise: ProcessNoiseConfig::default(),
            measurement_noise: MeasurementNoiseConfig::default(),
            polarization: PolarizationConfig::default(),
            environment: EnvironmentConfig::default(),
            observability: ObservabilityConfig::default(),
            ekf: EkfConfig::default(),
            ukf: UkfConfig::default(),
            extrapolation: ExtrapolationConfig::default(),
            validation: ValidationConfig::default(),
            auxiliary: AuxiliaryConfig::default(),
            plotting: EstimationPlottingConfig::default(),
            export: EstimationExportConfig::default(),
            timestamp_handling: TimestampHandlingConfig::default(),
            source_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterConfig {
    pub kind: FilterKind,
    pub confidence_level: f64,
    pub innovation_gate_probability: f64,
    pub reject_outliers: bool,
}
impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            kind: FilterKind::Ukf,
            confidence_level: 0.95,
            innovation_gate_probability: 0.997,
            reject_outliers: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StateModelConfig {
    pub kind: StateModelKind,
    pub activity_transform: StateTransformKind,
    pub include_condition_state: bool,
    pub condition_lower: f64,
    pub condition_upper: f64,
    pub condition_initial: f64,
}
impl Default for StateModelConfig {
    fn default() -> Self {
        Self {
            kind: StateModelKind::ActivityBaselinePolarization,
            activity_transform: StateTransformKind::IdentityLog10,
            include_condition_state: false,
            condition_lower: 0.5,
            condition_upper: 1.5,
            condition_initial: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InitializationConfig {
    pub activity_source: String,
    pub initial_activity: f64,
    pub initial_activity_unit: String,
    pub baseline_v: f64,
    pub polarization_v: f64,
    pub condition_value: f64,
    pub previous_artifact: Option<PathBuf>,
    pub steady_window_s: f64,
}
impl Default for InitializationConfig {
    fn default() -> Self {
        Self {
            activity_source: "calibration_inversion".into(),
            initial_activity: 0.001,
            initial_activity_unit: "mol/L".into(),
            baseline_v: 0.0,
            polarization_v: 0.0,
            condition_value: 1.0,
            previous_artifact: None,
            steady_window_s: 30.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InitialCovarianceConfig {
    pub log10_activity_variance: f64,
    pub baseline_variance_v2: f64,
    pub polarization_variance_v2: f64,
    pub condition_variance: f64,
}
impl Default for InitialCovarianceConfig {
    fn default() -> Self {
        Self {
            log10_activity_variance: 0.25,
            baseline_variance_v2: 1e-4,
            polarization_variance_v2: 1e-4,
            condition_variance: 0.01,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessNoiseConfig {
    pub activity_variance_per_s: f64,
    pub baseline_variance_v2_per_s: f64,
    pub polarization_variance_v2_per_s: f64,
    pub condition_variance_per_s: f64,
    pub source: CovarianceSourceKind,
}
impl Default for ProcessNoiseConfig {
    fn default() -> Self {
        Self {
            activity_variance_per_s: 1e-5,
            baseline_variance_v2_per_s: 1e-10,
            polarization_variance_v2_per_s: 1e-8,
            condition_variance_per_s: 1e-9,
            source: CovarianceSourceKind::Configured,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MeasurementNoiseConfig {
    pub source: MeasurementNoiseSourceKind,
    pub configured_variance_v2: f64,
    pub minimum_variance_v2: f64,
    pub maximum_variance_v2: f64,
    pub per_observation_variance: Option<String>,
    pub inflate_outside_domain: bool,
}
impl Default for MeasurementNoiseConfig {
    fn default() -> Self {
        Self {
            source: MeasurementNoiseSourceKind::SignalRobustVariance,
            configured_variance_v2: 1e-6,
            minimum_variance_v2: 1e-12,
            maximum_variance_v2: 1.0,
            per_observation_variance: None,
            inflate_outside_domain: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PolarizationConfig {
    pub tau_source: TauSourceKind,
    pub transient_parameter: String,
    pub aggregation: AggregationKind,
    pub configured_tau_s: f64,
    pub tau_uncertainty_s: Option<f64>,
    pub gain: f64,
    pub input_event_kind: Option<String>,
    pub input_model: PolarizationInputModel,
    pub gain_v_per_log10_activity: f64,
}
impl Default for PolarizationConfig {
    fn default() -> Self {
        Self {
            tau_source: TauSourceKind::Transient,
            transient_parameter: "tau_slow".into(),
            aggregation: AggregationKind::Median,
            configured_tau_s: 30.0,
            tau_uncertainty_s: None,
            gain: 1.0,
            input_event_kind: Some("concentration_step".into()),
            input_model: PolarizationInputModel::None,
            gain_v_per_log10_activity: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnvironmentConfig {
    pub temperature_series: Option<String>,
    pub conductivity_series: Option<String>,
    pub ionic_strength_series: Option<String>,
    pub flow_series: Option<String>,
    pub interferent_series: BTreeMap<String, String>,
    pub alignment: AlignmentKind,
    pub maximum_gap_s: f64,
    pub window_half_width_s: f64,
    pub allow_configured_fallback: bool,
    pub fallback_temperature_celsius: f64,
    pub fallback_conductivity_s_per_m: Option<f64>,
    pub fallback_ionic_strength_mol_l: Option<f64>,
}
impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            temperature_series: Some("temperature".into()),
            conductivity_series: Some("conductivity".into()),
            ionic_strength_series: Some("ionic_strength".into()),
            flow_series: Some("flow".into()),
            interferent_series: BTreeMap::new(),
            alignment: AlignmentKind::LinearInterpolation,
            maximum_gap_s: 60.0,
            window_half_width_s: 5.0,
            allow_configured_fallback: true,
            fallback_temperature_celsius: 25.0,
            fallback_conductivity_s_per_m: None,
            fallback_ionic_strength_mol_l: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObservabilityConfig {
    pub enabled: bool,
    pub horizon_steps: usize,
    pub rank_tolerance: f64,
    pub maximum_condition_number: f64,
    pub reject_unobservable_model: bool,
    pub empirical_perturbation: f64,
    pub empirical_sensitivity_tolerance: f64,
}
impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            horizon_steps: 20,
            rank_tolerance: 1e-8,
            maximum_condition_number: 1e10,
            reject_unobservable_model: true,
            empirical_perturbation: 1.0e-3,
            empirical_sensitivity_tolerance: 1.0e-8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EkfConfig {
    pub numerical_jacobian_relative_step: f64,
    pub use_joseph_update: bool,
}
impl Default for EkfConfig {
    fn default() -> Self {
        Self {
            numerical_jacobian_relative_step: 1e-6,
            use_joseph_update: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UkfConfig {
    pub alpha: f64,
    pub beta: f64,
    pub kappa: f64,
    pub initial_jitter: f64,
    pub jitter_multiplier: f64,
    pub maximum_jitter_attempts: usize,
}
impl Default for UkfConfig {
    fn default() -> Self {
        Self {
            alpha: 0.001,
            beta: 2.0,
            kappa: 0.0,
            initial_jitter: 1e-12,
            jitter_multiplier: 10.0,
            maximum_jitter_attempts: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtrapolationConfig {
    pub warn_outside_domain: bool,
    pub inflate_measurement_variance: bool,
    pub variance_inflation_factor: f64,
    pub near_boundary_fraction: f64,
    pub near_boundary_variance_inflation_factor: f64,
}
impl Default for ExtrapolationConfig {
    fn default() -> Self {
        Self {
            warn_outside_domain: true,
            inflate_measurement_variance: false,
            variance_inflation_factor: 4.0,
            near_boundary_fraction: 0.05,
            near_boundary_variance_inflation_factor: 1.25,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StateValidationConfig {
    pub absolute_convergence_tolerance: f64,
    pub minimum_consecutive_converged_points: usize,
    pub step_detection_threshold: f64,
    pub step_response_fraction: f64,
}
impl Default for StateValidationConfig {
    fn default() -> Self {
        Self {
            absolute_convergence_tolerance: 0.05,
            minimum_consecutive_converged_points: 1,
            step_detection_threshold: 1.0e-6,
            step_response_fraction: 0.9,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ValidationConfig {
    pub alignment_policy: TruthAlignmentPolicy,
    pub maximum_alignment_gap_s: f64,
    pub allow_truth_reuse: bool,
    pub states: BTreeMap<String, StateValidationConfig>,
}
impl Default for ValidationConfig {
    fn default() -> Self {
        let mut states = BTreeMap::new();
        states.insert("log10_activity".into(), StateValidationConfig::default());
        states.insert(
            "baseline_offset".into(),
            StateValidationConfig {
                absolute_convergence_tolerance: 0.001,
                ..StateValidationConfig::default()
            },
        );
        states.insert(
            "polarization".into(),
            StateValidationConfig {
                absolute_convergence_tolerance: 0.001,
                ..StateValidationConfig::default()
            },
        );
        states.insert(
            "sensitivity_scale".into(),
            StateValidationConfig {
                absolute_convergence_tolerance: 0.02,
                ..StateValidationConfig::default()
            },
        );
        Self {
            alignment_policy: TruthAlignmentPolicy::NearestWithinTolerance,
            maximum_alignment_gap_s: 0.5,
            allow_truth_reuse: false,
            states,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AuxiliaryConfig {
    pub condition_requires_auxiliary: bool,
    pub allow_known_standard_events: bool,
    pub allow_reference_activity: bool,
    pub known_log10_activity_variance: f64,
    /// Legacy field retained only for deserialization and explicit migration.
    #[serde(default, rename = "standard_variance_v2")]
    pub legacy_standard_variance_v2: Option<f64>,
}
impl Default for AuxiliaryConfig {
    fn default() -> Self {
        Self {
            condition_requires_auxiliary: true,
            allow_known_standard_events: true,
            allow_reference_activity: true,
            known_log10_activity_variance: 1e-8,
            legacy_standard_variance_v2: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EstimationPlottingConfig {
    pub enabled: bool,
    pub include_state_uncertainty: bool,
    pub include_innovations: bool,
    pub include_nis: bool,
    pub include_environment: bool,
    pub include_covariance: bool,
}
impl Default for EstimationPlottingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_state_uncertainty: true,
            include_innovations: true,
            include_nis: true,
            include_environment: true,
            include_covariance: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EstimationExportConfig {
    pub results_filename: String,
    pub states_filename: String,
    pub innovations_filename: String,
    pub diagnostics_filename: String,
    pub validation_filename: String,
    pub report_filename: String,
}
impl Default for EstimationExportConfig {
    fn default() -> Self {
        Self {
            results_filename: "state_estimation.json".into(),
            states_filename: "state_estimates.csv".into(),
            innovations_filename: "state_innovations.csv".into(),
            diagnostics_filename: "state_diagnostics.json".into(),
            validation_filename: "state_validation.json".into(),
            report_filename: "state_estimation_report.txt".into(),
        }
    }
}

impl ResolvedEstimationConfig {
    pub fn known_log10_activity_variance(&self) -> f64 {
        self.auxiliary
            .legacy_standard_variance_v2
            .unwrap_or(self.auxiliary.known_log10_activity_variance)
    }
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        if self.schema_version != ESTIMATION_CONFIG_SCHEMA_VERSION {
            return Err(ConfigurationError::invalid(format!(
                "estimation schema version {} is unsupported",
                self.schema_version
            )));
        }
        for (name, value) in [
            ("confidence_level", self.filter.confidence_level),
            (
                "innovation_gate_probability",
                self.filter.innovation_gate_probability,
            ),
        ] {
            if !value.is_finite() || !(0.0..1.0).contains(&value) {
                return Err(ConfigurationError::invalid(format!(
                    "{name} must be between 0 and 1"
                )));
            }
        }
        for (name, value) in [
            (
                "log10_activity_variance",
                self.initial_covariance.log10_activity_variance,
            ),
            (
                "baseline_variance_v2",
                self.initial_covariance.baseline_variance_v2,
            ),
            (
                "polarization_variance_v2",
                self.initial_covariance.polarization_variance_v2,
            ),
            (
                "condition_variance",
                self.initial_covariance.condition_variance,
            ),
            (
                "activity_variance_per_s",
                self.process_noise.activity_variance_per_s,
            ),
            (
                "baseline_variance_v2_per_s",
                self.process_noise.baseline_variance_v2_per_s,
            ),
            (
                "polarization_variance_v2_per_s",
                self.process_noise.polarization_variance_v2_per_s,
            ),
            (
                "condition_variance_per_s",
                self.process_noise.condition_variance_per_s,
            ),
            (
                "minimum_variance_v2",
                self.measurement_noise.minimum_variance_v2,
            ),
            (
                "maximum_variance_v2",
                self.measurement_noise.maximum_variance_v2,
            ),
            (
                "configured_variance_v2",
                self.measurement_noise.configured_variance_v2,
            ),
            (
                "known_log10_activity_variance",
                self.auxiliary.known_log10_activity_variance,
            ),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(ConfigurationError::invalid(format!(
                    "{name} must be finite and nonnegative"
                )));
            }
        }
        if !self
            .timestamp_handling
            .minor_reversal_threshold_s
            .is_finite()
            || self.timestamp_handling.minor_reversal_threshold_s < 0.0
            || !self.timestamp_handling.reset_threshold_s.is_finite()
            || self.timestamp_handling.reset_threshold_s < 0.0
            || !self.timestamp_handling.reset_threshold_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.timestamp_handling.reset_threshold_fraction)
            || self.timestamp_handling.minimum_segment_points == 0
        {
            return Err(ConfigurationError::invalid(
                "timestamp handling configuration is invalid",
            ));
        }
        if self.measurement_noise.maximum_variance_v2 < self.measurement_noise.minimum_variance_v2 {
            return Err(ConfigurationError::invalid(
                "measurement variance bounds are inverted",
            ));
        }
        if self.measurement_noise.configured_variance_v2 <= 0.0
            || self.measurement_noise.minimum_variance_v2 <= 0.0
            || self.known_log10_activity_variance() <= 0.0
        {
            return Err(ConfigurationError::invalid(
                "configured measurement and auxiliary variances must be positive",
            ));
        }
        if !self.environment.maximum_gap_s.is_finite() || self.environment.maximum_gap_s <= 0.0 {
            return Err(ConfigurationError::invalid(
                "environment maximum_gap_s must be positive",
            ));
        }
        if self.observability.horizon_steps == 0
            || !self.observability.rank_tolerance.is_finite()
            || self.observability.rank_tolerance <= 0.0
        {
            return Err(ConfigurationError::invalid(
                "observability horizon and tolerance are invalid",
            ));
        }
        if !self.observability.empirical_perturbation.is_finite()
            || self.observability.empirical_perturbation <= 0.0
            || !self
                .observability
                .empirical_sensitivity_tolerance
                .is_finite()
            || self.observability.empirical_sensitivity_tolerance < 0.0
        {
            return Err(ConfigurationError::invalid(
                "observability empirical perturbation settings are invalid",
            ));
        }
        if !self.extrapolation.near_boundary_fraction.is_finite()
            || self.extrapolation.near_boundary_fraction < 0.0
            || self.extrapolation.near_boundary_fraction > 1.0
            || !self
                .extrapolation
                .near_boundary_variance_inflation_factor
                .is_finite()
            || self.extrapolation.near_boundary_variance_inflation_factor < 1.0
            || !self.extrapolation.variance_inflation_factor.is_finite()
            || self.extrapolation.variance_inflation_factor < 1.0
        {
            return Err(ConfigurationError::invalid(
                "calibration-domain variance inflation settings are invalid",
            ));
        }
        if !self.validation.maximum_alignment_gap_s.is_finite()
            || self.validation.maximum_alignment_gap_s < 0.0
        {
            return Err(ConfigurationError::invalid(
                "validation maximum_alignment_gap_s must be finite and nonnegative",
            ));
        }
        for (name, state) in &self.validation.states {
            if !state.absolute_convergence_tolerance.is_finite()
                || state.absolute_convergence_tolerance < 0.0
                || state.minimum_consecutive_converged_points == 0
                || !state.step_detection_threshold.is_finite()
                || state.step_detection_threshold < 0.0
                || !state.step_response_fraction.is_finite()
                || !(0.0..=1.0).contains(&state.step_response_fraction)
            {
                return Err(ConfigurationError::invalid(format!(
                    "validation state '{name}' has invalid thresholds",
                )));
            }
        }
        if !self.ukf.alpha.is_finite()
            || self.ukf.alpha <= 0.0
            || !self.ukf.beta.is_finite()
            || self.ukf.beta < 0.0
            || !self.ukf.initial_jitter.is_finite()
            || self.ukf.initial_jitter < 0.0
            || self.ukf.maximum_jitter_attempts == 0
        {
            return Err(ConfigurationError::invalid("UKF parameters are invalid"));
        }
        if !self.polarization.configured_tau_s.is_finite()
            || self.polarization.configured_tau_s <= 0.0
        {
            return Err(ConfigurationError::invalid(
                "polarization tau must be positive",
            ));
        }
        if !self.polarization.gain_v_per_log10_activity.is_finite() {
            return Err(ConfigurationError::invalid(
                "polarization activity-step gain must be finite",
            ));
        }
        if !self.state_model.condition_lower.is_finite()
            || !self.state_model.condition_upper.is_finite()
            || self.state_model.condition_upper <= self.state_model.condition_lower
        {
            return Err(ConfigurationError::invalid(
                "condition state bounds are invalid",
            ));
        }
        Ok(())
    }

    pub fn load(
        workspace: &Path,
        override_path: Option<&Path>,
    ) -> Result<LoadedEstimationConfig, ConfigurationError> {
        let path = override_path
            .map(|p| {
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    workspace.join(p)
                }
            })
            .unwrap_or_else(|| workspace.join(DEFAULT_ESTIMATION_CONFIG_PATH));
        let mut warnings = Vec::new();
        if !path.exists() {
            let config = Self::default();
            config.validate()?;
            warnings.push(format!(
                "estimation config {} was not found; defaults were used",
                path.display()
            ));
            return Ok(LoadedEstimationConfig {
                config,
                source_path: None,
                warnings,
            });
        }
        let text = fs::read_to_string(&path).map_err(|e| ConfigurationError::io(&path, e))?;
        let mut config: Self =
            toml::from_str(&text).map_err(|e| ConfigurationError::parse(&path, e))?;
        if config.schema_version < ESTIMATION_CONFIG_SCHEMA_VERSION {
            if let Some(legacy_variance) = config.auxiliary.legacy_standard_variance_v2 {
                config.auxiliary.known_log10_activity_variance = legacy_variance;
                warnings.push(
                    "migrated legacy auxiliary.standard_variance_v2 to known_log10_activity_variance; the stored value is retained as log10(activity)^2, despite the legacy voltage-squared name".into(),
                );
            } else {
                warnings.push(
                    "migrated estimation configuration to schema version 3; verify validation alignment and state-specific threshold defaults".into(),
                );
            }
            config.schema_version = ESTIMATION_CONFIG_SCHEMA_VERSION;
        } else if config.schema_version > ESTIMATION_CONFIG_SCHEMA_VERSION {
            warnings.push(format!(
                "estimation config schema {} is unsupported",
                config.schema_version
            ));
        }
        if config.auxiliary.legacy_standard_variance_v2.is_some()
            && config.schema_version == ESTIMATION_CONFIG_SCHEMA_VERSION
        {
            warnings.push(
                "legacy auxiliary.standard_variance_v2 is present; use known_log10_activity_variance and verify the value is in log10(activity)^2".into(),
            );
        }
        config.source_path = Some(path.clone());
        config.validate()?;
        Ok(LoadedEstimationConfig {
            config,
            source_path: Some(path),
            warnings,
        })
    }
}

impl std::str::FromStr for FilterKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "ekf" => Ok(Self::Ekf),
            "ukf" => Ok(Self::Ukf),
            _ => Err(format!("unknown filter '{s}'")),
        }
    }
}
impl std::str::FromStr for StateModelKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "activity" | "activity_only" => Ok(Self::Activity),
            "activity_baseline" => Ok(Self::ActivityBaseline),
            "activity_baseline_polarization" | "activity_baseline_polarisation" => {
                Ok(Self::ActivityBaselinePolarization)
            }
            "custom" => Ok(Self::Custom),
            _ => Err(format!("unknown state model '{s}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_validate() {
        ResolvedEstimationConfig::default().validate().unwrap();
    }

    #[test]
    fn legacy_auxiliary_variance_migrates_with_warning() {
        let path = std::env::temp_dir().join(format!(
            "rust-electroanalysis-estimation-{}.toml",
            std::process::id()
        ));
        std::fs::write(
            &path,
            "schema_version = 1\n[auxiliary]\nstandard_variance_v2 = 2.5e-7\n",
        )
        .unwrap();
        let loaded = ResolvedEstimationConfig::load(
            path.parent().unwrap(),
            Some(std::path::Path::new(path.file_name().unwrap())),
        )
        .unwrap();
        assert_eq!(
            loaded.config.schema_version,
            ESTIMATION_CONFIG_SCHEMA_VERSION
        );
        assert_eq!(loaded.config.known_log10_activity_variance(), 2.5e-7);
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("standard_variance_v2"))
        );
        std::fs::remove_file(path).unwrap();
    }
}
