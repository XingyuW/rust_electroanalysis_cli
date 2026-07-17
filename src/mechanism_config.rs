//! TOML configuration for the Phase 4 integration layer.

use crate::domain::ConfigurationError;
use crate::results::{MechanismHypothesis, ResolvedMechanismConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MECHANISM_CONFIG_PATH: &str = "config/mechanism.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawMechanismConfig {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default)]
    eis: RawEis,
    #[serde(default)]
    transient: RawTransient,
    #[serde(default)]
    timescale: RawTimescale,
    #[serde(default)]
    matching: RawMatching,
    #[serde(default)]
    comparison: RawComparison,
    #[serde(default)]
    evidence: RawEvidence,
    #[serde(default)]
    trend: RawTrend,
    #[serde(default)]
    #[allow(dead_code)]
    plotting: RawPlotting,
    #[serde(default)]
    #[allow(dead_code)]
    export: RawExport,
    #[serde(default)]
    hypotheses: Vec<MechanismHypothesis>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawEis {
    #[serde(default = "default_true")]
    allow_warning_fits: bool,
    #[serde(default)]
    #[allow(dead_code)]
    require_uncertainty: bool,
    #[serde(default = "default_confidence")]
    confidence_level: f64,
    #[serde(default = "default_seed")]
    seed: u64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawTransient {
    #[serde(default = "default_true")]
    allow_warning_fits: bool,
    #[serde(default = "default_true")]
    selected_model_only: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawTimescale {
    #[serde(default = "default_mc")]
    monte_carlo_samples: usize,
    #[serde(default = "default_seed")]
    seed: u64,
    #[serde(default = "default_margin")]
    frequency_boundary_margin: f64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawMatching {
    #[serde(default = "default_true")]
    require_experiment_id: bool,
    #[serde(default)]
    require_sensor_id: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawComparison {
    #[serde(default = "default_ratio_weak")]
    ratio_weak: f64,
    #[serde(default = "default_ratio_moderate")]
    ratio_moderate: f64,
    #[serde(default = "default_ratio_strong")]
    ratio_strong: f64,
    #[serde(default = "default_log_weak")]
    log_distance_weak: f64,
    #[serde(default = "default_log_moderate")]
    log_distance_moderate: f64,
    #[serde(default = "default_log_strong")]
    log_distance_strong: f64,
    #[serde(default)]
    minimum_fit_quality: f64,
    #[serde(default = "default_compat_lower")]
    compatibility_ratio_lower: f64,
    #[serde(default = "default_compat_upper")]
    compatibility_ratio_upper: f64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawEvidence {
    #[serde(default = "default_replicates")]
    minimum_replicates_for_strong: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawTrend {
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_min_records")]
    minimum_records: usize,
    #[serde(default = "default_independent_variable")]
    independent_variable: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawPlotting {}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RawExport {}

fn default_schema_version() -> u32 {
    1
}
fn default_true() -> bool {
    true
}
fn default_confidence() -> f64 {
    0.95
}
fn default_seed() -> u64 {
    42
}
fn default_mc() -> usize {
    10_000
}
fn default_margin() -> f64 {
    0.1
}
fn default_ratio_weak() -> f64 {
    10.0
}
fn default_ratio_moderate() -> f64 {
    3.0
}
fn default_ratio_strong() -> f64 {
    1.5
}
fn default_log_weak() -> f64 {
    1.0
}
fn default_log_moderate() -> f64 {
    0.5
}
fn default_log_strong() -> f64 {
    0.1761
}
fn default_compat_lower() -> f64 {
    0.5
}
fn default_compat_upper() -> f64 {
    2.0
}
fn default_replicates() -> usize {
    3
}
fn default_min_records() -> usize {
    3
}
fn default_independent_variable() -> String {
    "sensor_age_days".to_string()
}

#[derive(Debug, Clone)]
pub struct LoadedMechanismConfig {
    pub config: ResolvedMechanismConfig,
    pub source_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

impl LoadedMechanismConfig {
    #[allow(clippy::field_reassign_with_default)]
    pub fn load(workspace: &Path, path: Option<&Path>) -> Result<Self, ConfigurationError> {
        let path = path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace.join(DEFAULT_MECHANISM_CONFIG_PATH));
        let mut warnings = Vec::new();
        let raw = if path.exists() {
            toml::from_str::<RawMechanismConfig>(
                &fs::read_to_string(&path).map_err(|e| ConfigurationError::io(&path, e))?,
            )
            .map_err(|e| ConfigurationError::parse(&path, e))?
        } else {
            warnings.push(format!(
                "mechanism config {} was not found; defaults used",
                path.display()
            ));
            RawMechanismConfig {
                schema_version: 1,
                ..Default::default()
            }
        };
        if raw.schema_version != 1 {
            return Err(ConfigurationError::invalid(format!(
                "unsupported mechanism config schema version {}",
                raw.schema_version
            )));
        }
        let mut config = ResolvedMechanismConfig {
            schema_version: raw.schema_version,
            ..ResolvedMechanismConfig::default()
        };
        config.confidence_level = raw.eis.confidence_level;
        config.allow_warning_fits = raw.eis.allow_warning_fits;
        config.ratio_weak = raw.comparison.ratio_weak;
        config.ratio_moderate = raw.comparison.ratio_moderate;
        config.ratio_strong = raw.comparison.ratio_strong;
        config.log_distance_weak = raw.comparison.log_distance_weak;
        config.log_distance_moderate = raw.comparison.log_distance_moderate;
        config.log_distance_strong = raw.comparison.log_distance_strong;
        config.minimum_fit_quality = raw.comparison.minimum_fit_quality;
        config.compatibility_ratio_lower = raw.comparison.compatibility_ratio_lower;
        config.compatibility_ratio_upper = raw.comparison.compatibility_ratio_upper;
        config.require_experiment_id = raw.matching.require_experiment_id;
        config.require_sensor_id = raw.matching.require_sensor_id;
        config.minimum_replicates_for_strong = raw.evidence.minimum_replicates_for_strong;
        config.trend_minimum_records = raw.trend.minimum_records;
        config.trend_independent_variable = raw.trend.independent_variable;
        config.frequency_boundary_margin = raw.timescale.frequency_boundary_margin;
        config.monte_carlo_samples = raw.timescale.monte_carlo_samples;
        config.seed = raw.timescale.seed;
        config.selected_model_only = raw.transient.selected_model_only;
        config.hypotheses = raw.hypotheses;
        validate(&config)?;
        Ok(Self {
            config,
            source_path: path.exists().then_some(path),
            warnings,
        })
    }
}

pub fn validate(config: &ResolvedMechanismConfig) -> Result<(), ConfigurationError> {
    if !(0.0 < config.confidence_level && config.confidence_level < 1.0) {
        return Err(ConfigurationError::invalid(
            "confidence_level must be between 0 and 1",
        ));
    }
    if !(config.ratio_strong <= config.ratio_moderate
        && config.ratio_moderate <= config.ratio_weak
        && config.ratio_strong >= 1.0)
    {
        return Err(ConfigurationError::invalid(
            "ratio thresholds must satisfy 1 <= strong <= moderate <= weak",
        ));
    }
    if !(config.log_distance_strong <= config.log_distance_moderate
        && config.log_distance_moderate <= config.log_distance_weak
        && config.log_distance_strong >= 0.0)
    {
        return Err(ConfigurationError::invalid(
            "log-distance thresholds are not ordered",
        ));
    }
    if !(0.0 < config.compatibility_ratio_lower
        && config.compatibility_ratio_lower < config.compatibility_ratio_upper)
    {
        return Err(ConfigurationError::invalid(
            "compatibility ratio bounds are invalid",
        ));
    }
    if !(0.0..=1.0).contains(&config.frequency_boundary_margin) || config.monte_carlo_samples == 0 {
        return Err(ConfigurationError::invalid(
            "frequency boundary margin or Monte Carlo sample count is invalid",
        ));
    }
    Ok(())
}
