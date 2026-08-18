//! Configuration for baseline comparison and transparent health rules.

use crate::{
    domain::ConfigurationError, health::error::HealthError,
    mechanism::config::MechanismHypothesisId, results::HealthDimension,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureOperator {
    GreaterThan,
    LessThan,
    RelativeIncreaseGreaterThan,
    RelativeDecreaseGreaterThan,
    LogRatioGreaterThan,
    RobustZGreaterThan,
    WarningPresent,
    TrendIncreasing,
    TrendDecreasing,
    EvidenceLevelPresent,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureCondition {
    pub feature: String,
    pub operator: FeatureOperator,
    pub value: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HealthSeverity {
    Informational,
    Minor,
    #[default]
    Moderate,
    Major,
    Critical,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthFindingKind {
    ElevatedNoise,
    ExcessiveDrift,
    FrequentSpikes,
    SlowResponse,
    ReducedResponseAmplitude,
    ReducedSensitivity,
    HighHysteresis,
    PoorCalibrationPrediction,
    EisParameterShift,
    PoorModelIdentifiability,
    ProbableFouling,
    ProbableReferenceInstability,
    ProbableContactIssue,
    EnvironmentalMismatch,
    DataQualityProblem,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthRule {
    pub rule_id: String,
    pub finding: HealthFindingKind,
    pub severity: HealthSeverity,
    #[serde(default)]
    pub all_of: Vec<FeatureCondition>,
    #[serde(default)]
    pub any_of: Vec<FeatureCondition>,
    #[serde(default)]
    pub minimum_evidence_domains: usize,
    #[serde(default)]
    pub minimum_baseline_records: usize,
    #[serde(default)]
    pub alternative_explanations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BaselineConfig {
    /// Minimum number of baseline records, not a domain count.
    #[serde(alias = "minimum_records")]
    pub minimum_required_records: usize,
    pub robust_statistics: bool,
}
impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            minimum_required_records: 3,
            robust_statistics: true,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ComparabilityConfig {
    pub require_same_analyte: bool,
    pub require_same_sample_matrix: bool,
    pub maximum_temperature_difference_k: f64,
    pub require_same_sensor_design: bool,
}
impl Default for ComparabilityConfig {
    fn default() -> Self {
        Self {
            require_same_analyte: true,
            require_same_sample_matrix: true,
            maximum_temperature_difference_k: 2.0,
            require_same_sensor_design: true,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NormalizationConfig {
    pub use_relative_difference: bool,
    pub use_robust_z_score: bool,
    pub minimum_baseline_records_for_z_score: usize,
}
impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            use_relative_difference: true,
            use_robust_z_score: true,
            minimum_baseline_records_for_z_score: 5,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HealthAssessmentConfig {
    pub minimum_domains_for_assessment: usize,
    pub minimum_domains_for_mechanistic_finding: usize,
    pub allow_warning_artifacts: bool,
}
impl Default for HealthAssessmentConfig {
    fn default() -> Self {
        Self {
            minimum_domains_for_assessment: 2,
            minimum_domains_for_mechanistic_finding: 2,
            allow_warning_artifacts: true,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HealthPlotConfig {
    pub enabled: bool,
}
impl Default for HealthPlotConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HealthExportConfig {
    pub baseline_filename: String,
    pub assessment_filename: String,
    pub features_filename: String,
    pub findings_filename: String,
    pub trends_filename: String,
    pub report_filename: String,
}
impl Default for HealthExportConfig {
    fn default() -> Self {
        Self {
            baseline_filename: "health_baseline.json".into(),
            assessment_filename: "health_assessment.json".into(),
            features_filename: "health_features.csv".into(),
            findings_filename: "health_findings.csv".into(),
            trends_filename: "health_trends.csv".into(),
            report_filename: "health_report.txt".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ResolvedHealthConfig {
    pub schema_version: u32,
    pub baseline: BaselineConfig,
    pub comparability: ComparabilityConfig,
    pub normalization: NormalizationConfig,
    pub assessment: HealthAssessmentConfig,
    #[serde(default)]
    pub rules: Vec<HealthRule>,
    pub plotting: HealthPlotConfig,
    pub export: HealthExportConfig,
}
impl Default for ResolvedHealthConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            baseline: Default::default(),
            comparability: Default::default(),
            normalization: Default::default(),
            assessment: Default::default(),
            rules: Vec::new(),
            plotting: Default::default(),
            export: Default::default(),
        }
    }
}
pub struct LoadedHealthConfig {
    pub config: ResolvedHealthConfig,
    pub source_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}
impl LoadedHealthConfig {
    pub fn load(
        workspace: &Path,
        override_path: Option<&Path>,
    ) -> Result<Self, ConfigurationError> {
        let path = override_path
            .map(|p| {
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    workspace.join(p)
                }
            })
            .or_else(|| {
                let p = workspace.join("config/health.toml");
                p.exists().then_some(p)
            });
        let Some(path) = path else {
            return Ok(Self {
                config: Default::default(),
                source_path: None,
                warnings: vec!["health configuration not found; defaults used".into()],
            });
        };
        let text = fs::read_to_string(&path).map_err(|e| ConfigurationError::io(&path, e))?;
        let config: ResolvedHealthConfig =
            toml::from_str(&text).map_err(|e| ConfigurationError::parse(&path, e))?;
        if config.schema_version != 1 {
            return Err(ConfigurationError::invalid(format!(
                "unsupported health configuration schema {}",
                config.schema_version
            )));
        }
        let mut ids = std::collections::BTreeSet::new();
        for rule in &config.rules {
            if !ids.insert(&rule.rule_id) {
                return Err(ConfigurationError::invalid(format!(
                    "duplicate health rule id {}",
                    rule.rule_id
                )));
            }
        }
        Ok(Self {
            config,
            source_path: Some(path),
            warnings: Vec::new(),
        })
    }
}

/// Strict, opt-in configuration for the Phase-C evidence evaluator.  It is
/// intentionally independent of the permissive legacy health configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCHealthEvidenceConfig {
    pub schema_version: u32,
    pub maximum_reference_alignment_difference_s: f64,
    pub data_quality: PhaseCDataQualityConfig,
    pub signal_integrity: PhaseCSignalIntegrityConfig,
    pub calibration_health: PhaseCCalibrationHealthConfig,
    pub dynamic_response_health: PhaseCDynamicResponseHealthConfig,
    pub environmental_robustness: PhaseCEnvironmentalRobustnessConfig,
    pub model_consistency: PhaseCModelConsistencyConfig,
    pub observability: PhaseCObservabilityConfig,
    pub uncertainty_health: PhaseCUncertaintyHealthConfig,
    pub causal_promotion: PhaseCCausalPromotionConfig,
    #[serde(default)]
    pub phase_b_hypothesis_bindings: Vec<PhaseCHypothesisBinding>,
    /// Runtime provenance; skipped so it cannot alter the strict TOML shape.
    #[serde(skip)]
    config_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LevelThreshold {
    pub watch: f64,
    pub degraded: f64,
    pub critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCDataQualityConfig {
    pub minimum_finite_samples: usize,
    pub maximum_missing_fraction: f64,
    pub maximum_interval_cv: f64,
    pub maximum_duplicate_timestamps: usize,
    pub maximum_non_monotonic_timestamps: usize,
    pub allow_interpolation_gap_exceeded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCSignalIntegrityConfig {
    pub maximum_rms_noise_v: LevelThreshold,
    pub maximum_robust_noise_standard_deviation_v: LevelThreshold,
    pub maximum_spike_fraction: LevelThreshold,
    pub maximum_absolute_drift_v_per_s: LevelThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCCalibrationHealthConfig {
    pub maximum_absolute_slope_efficiency_error: LevelThreshold,
    pub maximum_rmse_v: LevelThreshold,
    pub maximum_absolute_prediction_bias_v: LevelThreshold,
    pub maximum_hysteresis_v: LevelThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCDynamicResponseHealthConfig {
    pub selected_event_index: usize,
    pub baseline_tau_fast_feature: String,
    pub baseline_tau_slow_feature: String,
    pub baseline_time_to_90_percent_feature: String,
    pub baseline_response_amplitude_feature: String,
    pub maximum_tau_fast_ratio: LevelThreshold,
    pub maximum_tau_slow_ratio: LevelThreshold,
    pub maximum_time_to_90_percent_ratio: LevelThreshold,
    pub maximum_response_amplitude_relative_loss: LevelThreshold,
    pub maximum_fit_rmse_v: LevelThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentalCovariate {
    TemperatureK,
    ConductivitySPerM,
    IonicStrengthMolL,
    Flow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCEnvironmentalRobustnessConfig {
    pub covariate: EnvironmentalCovariate,
    pub minimum_points: usize,
    pub minimum_covariate_range: f64,
    pub minimum_absolute_spearman_correlation: LevelThreshold,
    pub minimum_residual_rms_v: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCModelConsistencyConfig {
    pub maximum_residual_rms_v: LevelThreshold,
    pub maximum_residual_bias_v: LevelThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCObservabilityConfig {
    pub maximum_condition_number: LevelThreshold,
    pub require_empirical_identifiability: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCUncertaintyHealthConfig {
    pub maximum_partial_uncertainty_fraction: LevelThreshold,
    pub maximum_standard_error_v: LevelThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCCausalPromotionConfig {
    pub minimum_independent_supporting_evidence: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PhaseCHypothesisRelationship {
    PossiblePhysicalDegradation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PhaseCHypothesisBinding {
    pub hypothesis_id: MechanismHypothesisId,
    pub health_dimension: HealthDimension,
    pub relationship: PhaseCHypothesisRelationship,
}

pub struct LoadedPhaseCHealthEvidenceConfig {
    pub config: PhaseCHealthEvidenceConfig,
    pub source_path: PathBuf,
    pub config_sha256: String,
}

impl PhaseCHealthEvidenceConfig {
    pub fn load(path: &Path) -> Result<LoadedPhaseCHealthEvidenceConfig, HealthError> {
        let bytes = fs::read(path).map_err(HealthError::Io)?;
        let text =
            std::str::from_utf8(&bytes).map_err(|error| HealthError::InvalidPhaseCConfig {
                message: format!("configuration is not valid UTF-8: {error}"),
            })?;
        let mut config: Self =
            toml::from_str(text).map_err(|error| HealthError::InvalidPhaseCConfig {
                message: error.to_string(),
            })?;
        config.validate()?;
        let config_sha256 = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        config.config_sha256 = Some(config_sha256.clone());
        Ok(LoadedPhaseCHealthEvidenceConfig {
            config,
            source_path: path.to_path_buf(),
            config_sha256,
        })
    }

    pub(crate) fn configuration_hash(&self) -> Option<&str> {
        self.config_sha256.as_deref()
    }

    pub fn validate(&self) -> Result<(), HealthError> {
        let invalid = |message: &str| HealthError::InvalidPhaseCConfig {
            message: message.into(),
        };
        if self.schema_version != 1
            || !self.maximum_reference_alignment_difference_s.is_finite()
            || self.maximum_reference_alignment_difference_s < 0.0
        {
            return Err(invalid(
                "schema_version must be 1 and maximum_reference_alignment_difference_s must be finite and non-negative",
            ));
        }
        if self.data_quality.minimum_finite_samples < 2
            || !self.data_quality.maximum_missing_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.data_quality.maximum_missing_fraction)
            || !self.data_quality.maximum_interval_cv.is_finite()
            || self.data_quality.maximum_interval_cv < 0.0
        {
            return Err(invalid("invalid data_quality configuration"));
        }
        for threshold in [
            &self.signal_integrity.maximum_rms_noise_v,
            &self
                .signal_integrity
                .maximum_robust_noise_standard_deviation_v,
            &self.signal_integrity.maximum_spike_fraction,
            &self.signal_integrity.maximum_absolute_drift_v_per_s,
            &self
                .calibration_health
                .maximum_absolute_slope_efficiency_error,
            &self.calibration_health.maximum_rmse_v,
            &self.calibration_health.maximum_absolute_prediction_bias_v,
            &self.calibration_health.maximum_hysteresis_v,
            &self.dynamic_response_health.maximum_tau_fast_ratio,
            &self.dynamic_response_health.maximum_tau_slow_ratio,
            &self
                .dynamic_response_health
                .maximum_time_to_90_percent_ratio,
            &self
                .dynamic_response_health
                .maximum_response_amplitude_relative_loss,
            &self.dynamic_response_health.maximum_fit_rmse_v,
            &self
                .environmental_robustness
                .minimum_absolute_spearman_correlation,
            &self.model_consistency.maximum_residual_rms_v,
            &self.model_consistency.maximum_residual_bias_v,
            &self.observability.maximum_condition_number,
            &self.uncertainty_health.maximum_partial_uncertainty_fraction,
            &self.uncertainty_health.maximum_standard_error_v,
        ] {
            if !threshold.watch.is_finite()
                || !threshold.degraded.is_finite()
                || !threshold.critical.is_finite()
                || threshold.watch < 0.0
                || !(threshold.watch < threshold.degraded
                    && threshold.degraded < threshold.critical)
            {
                return Err(invalid(
                    "each Phase-C threshold must be finite and strictly ordered",
                ));
            }
        }
        for threshold in [
            &self.dynamic_response_health.maximum_tau_fast_ratio,
            &self.dynamic_response_health.maximum_tau_slow_ratio,
            &self
                .dynamic_response_health
                .maximum_time_to_90_percent_ratio,
            &self.observability.maximum_condition_number,
            &self.uncertainty_health.maximum_standard_error_v,
        ] {
            if threshold.watch <= 0.0 {
                return Err(invalid(
                    "ratio, condition-number, and standard-error thresholds must be strictly positive",
                ));
            }
        }
        if self
            .dynamic_response_health
            .baseline_tau_fast_feature
            .is_empty()
            || self
                .dynamic_response_health
                .baseline_tau_slow_feature
                .is_empty()
            || self
                .dynamic_response_health
                .baseline_time_to_90_percent_feature
                .is_empty()
            || self
                .dynamic_response_health
                .baseline_response_amplitude_feature
                .is_empty()
            || self.environmental_robustness.minimum_points < 3
            || !self
                .environmental_robustness
                .minimum_covariate_range
                .is_finite()
            || self.environmental_robustness.minimum_covariate_range <= 0.0
            || !self
                .environmental_robustness
                .minimum_residual_rms_v
                .is_finite()
            || self.environmental_robustness.minimum_residual_rms_v < 0.0
            || self
                .causal_promotion
                .minimum_independent_supporting_evidence
                < 2
        {
            return Err(invalid(
                "invalid Phase-C dynamic, environmental, or causal configuration",
            ));
        }
        let mut previous = None;
        for binding in &self.phase_b_hypothesis_bindings {
            if binding.hypothesis_id.is_empty()
                || !matches!(
                    binding.health_dimension,
                    HealthDimension::SignalIntegrity
                        | HealthDimension::CalibrationHealth
                        | HealthDimension::DynamicResponseHealth
                )
            {
                return Err(invalid("invalid Phase-C hypothesis binding"));
            }
            if let Some(last) = previous.replace(&binding.hypothesis_id)
                && last >= &binding.hypothesis_id
            {
                return Err(invalid(
                    "Phase-C hypothesis bindings must be lexical and unique",
                ));
            }
        }
        Ok(())
    }
}
