//! Configuration loading and validation for transient analysis.

use crate::domain::ConfigurationError;
use crate::potentiometry::transient::models::{BaselineMethod, ResponseMode, TransientModelKind};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const TRANSIENT_CONFIG_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_TRANSIENT_CONFIG_PATH: &str = "config/transient.toml";

#[derive(Debug, Clone)]
pub struct LoadedTransientConfig {
    pub config: ResolvedTransientConfig,
    pub base_dir: PathBuf,
    pub source_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResolvedTransientConfig {
    pub schema_version: u32,
    pub segmentation: SegmentationConfig,
    pub baseline: BaselineConfig,
    pub models: ModelConfig,
    pub optimizer: OptimizerConfig,
    pub selection: SelectionConfig,
    pub validation: ValidationConfig,
    pub uncertainty: UncertaintyConfig,
    pub plotting: TransientPlottingConfig,
    pub export: TransientExportConfig,
    pub source_path: Option<PathBuf>,
}

impl Default for ResolvedTransientConfig {
    fn default() -> Self {
        Self {
            schema_version: TRANSIENT_CONFIG_SCHEMA_VERSION,
            segmentation: SegmentationConfig::default(),
            baseline: BaselineConfig::default(),
            models: ModelConfig::default(),
            optimizer: OptimizerConfig::default(),
            selection: SelectionConfig::default(),
            validation: ValidationConfig::default(),
            uncertainty: UncertaintyConfig::default(),
            plotting: TransientPlottingConfig::default(),
            export: TransientExportConfig::default(),
            source_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SegmentationConfig {
    pub pre_event_s: f64,
    pub post_event_s: f64,
    pub baseline_window_s: f64,
    pub minimum_points: usize,
    pub minimum_duration_s: f64,
    pub maximum_missing_fraction: f64,
    pub duplicate_timestamp_policy: DuplicateTimestampPolicy,
    pub non_monotonic_policy: NonMonotonicPolicy,
    pub irregular_sampling_policy: IrregularSamplingPolicy,
}

impl Default for SegmentationConfig {
    fn default() -> Self {
        Self {
            pre_event_s: 30.0,
            post_event_s: 300.0,
            baseline_window_s: 20.0,
            minimum_points: 20,
            minimum_duration_s: 10.0,
            maximum_missing_fraction: 0.20,
            duplicate_timestamp_policy: DuplicateTimestampPolicy::Error,
            non_monotonic_policy: NonMonotonicPolicy::Sort,
            irregular_sampling_policy: IrregularSamplingPolicy::Allow,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateTimestampPolicy {
    #[default]
    Error,
    Average,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NonMonotonicPolicy {
    #[default]
    Sort,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IrregularSamplingPolicy {
    #[default]
    Allow,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BaselineConfig {
    pub method: BaselineMethod,
    pub response_mode: ResponseMode,
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            method: BaselineMethod::Median,
            response_mode: ResponseMode::BaselineRelative,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    pub enabled: Vec<TransientModelKind>,
    pub beta_min: f64,
    pub beta_max: f64,
}

fn default_models() -> Vec<TransientModelKind> {
    TransientModelKind::ALL.to_vec()
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            enabled: default_models(),
            beta_min: 0.05,
            beta_max: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OptimizerConfig {
    pub maximum_iterations: usize,
    pub ftol: f64,
    pub xtol: f64,
    pub gtol: f64,
    pub patience: usize,
    pub step_bound: f64,
    pub multiple_starts: usize,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            maximum_iterations: 400,
            ftol: 1e-10,
            xtol: 1e-10,
            gtol: 1e-10,
            patience: 400,
            step_bound: 50.0,
            multiple_starts: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SelectionConfig {
    pub criterion: SelectionCriterion,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            criterion: SelectionCriterion::Aic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SelectionCriterion {
    #[default]
    Aic,
    Bic,
}

impl std::fmt::Display for SelectionCriterion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Aic => "aic",
            Self::Bic => "bic",
        })
    }
}

impl std::str::FromStr for SelectionCriterion {
    type Err = ConfigurationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "aic" => Ok(Self::Aic),
            "bic" => Ok(Self::Bic),
            other => Err(ConfigurationError::invalid(format!(
                "selection criterion must be 'aic' or 'bic', got '{other}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ValidationConfig {
    pub minimum_tau_ratio: f64,
    pub maximum_tau_to_window_ratio: f64,
    pub negligible_amplitude_fraction: f64,
    pub high_autocorrelation_threshold: f64,
    pub bound_proximity_fraction: f64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            minimum_tau_ratio: 3.0,
            maximum_tau_to_window_ratio: 1.0,
            negligible_amplitude_fraction: 0.05,
            high_autocorrelation_threshold: 0.8,
            bound_proximity_fraction: 0.01,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UncertaintyConfig {
    pub bootstrap_iterations: usize,
    pub confidence_level: f64,
    pub seed: u64,
    pub minimum_success_fraction: f64,
}

impl Default for UncertaintyConfig {
    fn default() -> Self {
        Self {
            bootstrap_iterations: 500,
            confidence_level: 0.95,
            seed: 42,
            minimum_success_fraction: 0.80,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TransientPlottingConfig {
    pub enabled: bool,
    pub include_components: bool,
    pub include_residuals: bool,
    pub include_model_comparison: bool,
}

impl Default for TransientPlottingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_components: true,
            include_residuals: true,
            include_model_comparison: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TransientExportConfig {
    pub json_filename: String,
    pub features_filename: String,
    pub model_comparison_filename: String,
    pub report_filename: String,
}

impl Default for TransientExportConfig {
    fn default() -> Self {
        Self {
            json_filename: "transient_results.json".to_string(),
            features_filename: "transient_features.csv".to_string(),
            model_comparison_filename: "transient_model_comparison.csv".to_string(),
            report_filename: "transient_report.txt".to_string(),
        }
    }
}

impl ResolvedTransientConfig {
    pub fn load(
        workspace_dir: &Path,
        override_path: Option<&Path>,
    ) -> Result<LoadedTransientConfig, ConfigurationError> {
        let resolved_path = override_path
            .map(|path| resolve_cli_path(path, workspace_dir))
            .unwrap_or_else(|| workspace_dir.join(DEFAULT_TRANSIENT_CONFIG_PATH));
        let source_path = resolved_path.exists().then(|| resolved_path.clone());

        if override_path.is_some() && !resolved_path.exists() {
            return Err(ConfigurationError::invalid(format!(
                "transient config override does not exist: {}",
                resolved_path.display()
            )));
        }
        if !resolved_path.exists() {
            return Ok(LoadedTransientConfig {
                config: Self::default(),
                base_dir: workspace_dir.to_path_buf(),
                source_path: None,
                warnings: Vec::new(),
            });
        }

        let text = fs::read_to_string(&resolved_path)
            .map_err(|error| ConfigurationError::io(&resolved_path, error))?;
        if text.trim().is_empty() {
            return Ok(LoadedTransientConfig {
                config: Self::default(),
                base_dir: resolved_path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| workspace_dir.to_path_buf()),
                source_path,
                warnings: vec![format!(
                    "transient config {} was empty; defaults were used",
                    resolved_path.display()
                )],
            });
        }

        let mut config: Self = toml::from_str(&text).map_err(|error| {
            ConfigurationError::invalid(format!(
                "failed to parse transient config {}: {error}",
                resolved_path.display()
            ))
        })?;
        config.source_path = source_path.clone();
        config.validate()?;

        let mut warnings = Vec::new();
        if config.schema_version != TRANSIENT_CONFIG_SCHEMA_VERSION {
            warnings.push(format!(
                "transient config schema_version {} does not match supported version {}",
                config.schema_version, TRANSIENT_CONFIG_SCHEMA_VERSION
            ));
        }
        Ok(LoadedTransientConfig {
            config,
            base_dir: resolved_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| workspace_dir.to_path_buf()),
            source_path,
            warnings,
        })
    }

    pub fn apply_cli_overrides(
        &mut self,
        model: Option<&str>,
        selection: Option<&str>,
        bootstrap: Option<usize>,
        seed: Option<u64>,
    ) -> Result<(), ConfigurationError> {
        if let Some(model) = model {
            self.models.enabled =
                if model.eq_ignore_ascii_case("all") {
                    TransientModelKind::ALL.to_vec()
                } else {
                    vec![model.parse().map_err(
                        |error: crate::potentiometry::PotentiometryError| {
                            ConfigurationError::invalid(error.to_string())
                        },
                    )?]
                };
        }
        if let Some(selection) = selection {
            self.selection.criterion = selection.parse()?;
        }
        if let Some(bootstrap) = bootstrap {
            self.uncertainty.bootstrap_iterations = bootstrap;
        }
        if let Some(seed) = seed {
            self.uncertainty.seed = seed;
        }
        self.validate()
    }

    pub fn validate(&self) -> Result<(), ConfigurationError> {
        let segmentation = &self.segmentation;
        if !segmentation.pre_event_s.is_finite() || segmentation.pre_event_s < 0.0 {
            return Err(ConfigurationError::invalid(
                "segmentation.pre_event_s must be finite and non-negative",
            ));
        }
        if !segmentation.post_event_s.is_finite() || segmentation.post_event_s <= 0.0 {
            return Err(ConfigurationError::invalid(
                "segmentation.post_event_s must be finite and positive",
            ));
        }
        if !segmentation.baseline_window_s.is_finite() || segmentation.baseline_window_s < 0.0 {
            return Err(ConfigurationError::invalid(
                "segmentation.baseline_window_s must be finite and non-negative",
            ));
        }
        if segmentation.minimum_points == 0 {
            return Err(ConfigurationError::invalid(
                "segmentation.minimum_points must be greater than zero",
            ));
        }
        if !segmentation.minimum_duration_s.is_finite() || segmentation.minimum_duration_s < 0.0 {
            return Err(ConfigurationError::invalid(
                "segmentation.minimum_duration_s must be finite and non-negative",
            ));
        }
        if !segmentation.maximum_missing_fraction.is_finite()
            || !(0.0..=1.0).contains(&segmentation.maximum_missing_fraction)
        {
            return Err(ConfigurationError::invalid(
                "segmentation.maximum_missing_fraction must be between 0 and 1",
            ));
        }
        if self.models.enabled.is_empty() {
            return Err(ConfigurationError::invalid(
                "models.enabled must contain at least one model",
            ));
        }
        if !self.models.beta_min.is_finite()
            || !self.models.beta_max.is_finite()
            || self.models.beta_min <= 0.0
            || self.models.beta_min >= self.models.beta_max
        {
            return Err(ConfigurationError::invalid(
                "models beta bounds must satisfy 0 < beta_min < beta_max",
            ));
        }
        if self.optimizer.maximum_iterations == 0
            || self.optimizer.patience == 0
            || self.optimizer.multiple_starts == 0
        {
            return Err(ConfigurationError::invalid(
                "optimizer iteration, patience, and multiple_starts values must be positive",
            ));
        }
        for (name, value) in [
            ("optimizer.ftol", self.optimizer.ftol),
            ("optimizer.xtol", self.optimizer.xtol),
            ("optimizer.gtol", self.optimizer.gtol),
        ] {
            if !value.is_finite() || value <= 0.0 {
                return Err(ConfigurationError::invalid(format!(
                    "{name} must be finite and positive"
                )));
            }
        }
        if !self.validation.minimum_tau_ratio.is_finite()
            || self.validation.minimum_tau_ratio <= 1.0
            || !self.validation.maximum_tau_to_window_ratio.is_finite()
            || self.validation.maximum_tau_to_window_ratio <= 0.0
            || !self.validation.negligible_amplitude_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.validation.negligible_amplitude_fraction)
        {
            return Err(ConfigurationError::invalid(
                "validation thresholds are outside their valid ranges",
            ));
        }
        if !self.uncertainty.confidence_level.is_finite()
            || !(0.0..1.0).contains(&self.uncertainty.confidence_level)
            || !self.uncertainty.minimum_success_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.uncertainty.minimum_success_fraction)
        {
            return Err(ConfigurationError::invalid(
                "uncertainty confidence and success fractions are outside their valid ranges",
            ));
        }
        if self.export.json_filename.trim().is_empty()
            || self.export.features_filename.trim().is_empty()
            || self.export.model_comparison_filename.trim().is_empty()
            || self.export.report_filename.trim().is_empty()
        {
            return Err(ConfigurationError::invalid(
                "transient export filenames must not be empty",
            ));
        }
        Ok(())
    }
}

fn resolve_cli_path(path: &Path, workspace_dir: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolvedTransientConfig, SelectionCriterion};

    #[test]
    fn defaults_and_cli_overrides_are_validated() {
        let mut config = ResolvedTransientConfig::default();
        assert_eq!(config.models.enabled.len(), 4);
        config
            .apply_cli_overrides(Some("single"), Some("bic"), Some(3), Some(7))
            .expect("CLI overrides");
        assert_eq!(config.models.enabled.len(), 1);
        assert_eq!(config.selection.criterion, SelectionCriterion::Bic);
        assert_eq!(config.uncertainty.bootstrap_iterations, 3);
        assert_eq!(config.uncertainty.seed, 7);
    }

    #[test]
    fn unsupported_schema_is_warning_not_parse_failure() {
        let config: ResolvedTransientConfig =
            toml::from_str("schema_version = 99").expect("schema version should deserialize");
        assert_eq!(config.schema_version, 99);
    }
}
