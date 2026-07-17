//! Configuration for baseline comparison and transparent health rules.

use crate::domain::ConfigurationError;
use serde::{Deserialize, Serialize};
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
