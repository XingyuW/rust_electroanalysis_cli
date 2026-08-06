//! Reproducible, evidence-preserving validation artifacts for model studies.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MODEL_VALIDATION_SCHEMA_VERSION: u32 = 1;
pub const MODEL_VALIDATION_ARTIFACT_KIND: &str = "ism_model_validation";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationDatasetCategory {
    StableStandard,
    ConcentrationSteps,
    ReverseSteps,
    IonicStrengthVariation,
    TemperatureVariation,
    Interferents,
    FlowChanges,
    ReferenceSubstitution,
    MembraneThicknessVariation,
    SolidContactVariation,
    ControlledFouling,
    SensorAging,
    PairedEisAndTransient,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationExperiment {
    pub experiment_id: String,
    pub category: ValidationDatasetCategory,
    pub sensor_id: String,
    pub analysis_path: String,
    #[serde(default)]
    pub reference_state_values: BTreeMap<String, f64>,
    #[serde(default)]
    pub reference_parameter_values: BTreeMap<String, f64>,
    #[serde(default)]
    pub expected_prediction_coverage: Option<f64>,
    #[serde(default)]
    pub is_real_experiment: bool,
    #[serde(default)]
    pub calibration_transfer_group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationManifest {
    pub schema_version: u32,
    pub study_id: String,
    pub experiments: Vec<ValidationExperiment>,
    pub model_comparison_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationMetric {
    pub experiment_id: String,
    pub metric: String,
    pub value: Option<f64>,
    pub unit: String,
    pub evidence_status: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelComparisonRow {
    pub model_id: String,
    pub observations: usize,
    pub rmse_v: Option<f64>,
    pub prediction_coverage: Option<f64>,
    pub contribution_reconstruction_error_v: Option<f64>,
    pub criterion: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResults {
    pub schema_version: u32,
    pub artifact_kind: String,
    pub study_id: String,
    pub metrics: Vec<ValidationMetric>,
    pub identifiability_report: serde_json::Value,
    pub model_comparison: Vec<ModelComparisonRow>,
    pub warnings: Vec<String>,
}

impl ValidationResults {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
