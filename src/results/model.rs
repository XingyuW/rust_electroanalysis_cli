//! Durable artifacts for compiled ISM model definitions.

use crate::model::{
    CompiledIsmModel, ComponentContribution, EquilibriumAssessment, IdentifiabilityReport,
    ModelDefinition, ModelError, PredictionUncertainty, ValidityReport,
};
use serde::{Deserialize, Serialize};

pub const MODEL_RESULT_SCHEMA_VERSION: u32 = 3;
pub const MODEL_COMPILATION_ARTIFACT_KIND: &str = "ism_model_compilation";
pub const MODEL_ANALYSIS_ARTIFACT_KIND: &str = "ism_model_analysis";

/// Serializable record of a validated model definition and its explicit
/// limitations. It contains no fitted values or inferred mechanisms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCompilationArtifact {
    pub schema_version: u32,
    pub artifact_kind: String,
    pub model_definition: ModelDefinition,
    pub definition_validity: ValidityReport,
    pub identifiability: IdentifiabilityReport,
}

/// One finite, auditable model evaluation used by user-facing workflows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelAnalysisPoint {
    pub time_s: f64,
    pub observed_voltage_v: Option<f64>,
    pub predicted_voltage_v: f64,
    #[serde(default = "crate::model::PredictionUncertainty::not_requested")]
    pub uncertainty: PredictionUncertainty,
    pub state_values: Vec<(String, f64)>,
    pub contributions: Vec<ComponentContribution>,
    pub equilibrium: EquilibriumAssessment,
    pub validity: ValidityReport,
    pub unexplained_residual_v: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelAnalysisReport {
    pub schema_version: u32,
    pub artifact_kind: String,
    pub model_definition: ModelDefinition,
    pub points: Vec<ModelAnalysisPoint>,
    pub identifiability: IdentifiabilityReport,
    pub evidence: Vec<String>,
}

impl ModelAnalysisReport {
    pub fn to_json(&self) -> Result<String, ModelError> {
        self.model_definition.validate_schema()?;
        if self.points.iter().any(|point| {
            !point.time_s.is_finite()
                || !point.predicted_voltage_v.is_finite()
                || point
                    .observed_voltage_v
                    .is_some_and(|value| !value.is_finite())
                || point
                    .unexplained_residual_v
                    .is_some_and(|value| !value.is_finite())
                || point
                    .state_values
                    .iter()
                    .any(|(_, value)| !value.is_finite())
                || point.contributions.iter().any(|value| {
                    value
                        .potential_v
                        .is_some_and(|potential| !potential.is_finite())
                        || value
                            .variance_v2
                            .is_some_and(|variance| !variance.is_finite())
                })
                || [
                    point.uncertainty.total_variance_v2,
                    point.uncertainty.standard_error_v,
                    point.uncertainty.state_variance_v2,
                    point.uncertainty.parameter_variance_v2,
                    point.uncertainty.observation_variance_v2,
                ]
                .into_iter()
                .flatten()
                .any(|value| !value.is_finite())
        }) {
            return Err(ModelError::NonFinite {
                subject: "model analysis report".into(),
            });
        }
        serde_json::to_string_pretty(self).map_err(|error| ModelError::Json(error.to_string()))
    }
}

impl ModelCompilationArtifact {
    pub fn from_compiled(model: &CompiledIsmModel) -> Self {
        Self {
            schema_version: MODEL_RESULT_SCHEMA_VERSION,
            artifact_kind: MODEL_COMPILATION_ARTIFACT_KIND.into(),
            model_definition: model.definition().clone(),
            definition_validity: ValidityReport {
                is_valid: true,
                checked_domain: model.definition().validity_domain.clone(),
                violations: Vec::new(),
                warnings: vec![
                    "Runtime validity requires explicit state, parameter, and input evaluation."
                        .into(),
                ],
            },
            identifiability: model.identifiability_report(),
        }
    }

    /// The supported serialization path validates all numeric definition
    /// fields before serde can turn a non-finite float into JSON `null`.
    pub fn to_json(&self) -> Result<String, ModelError> {
        self.model_definition.validate_schema()?;
        serde_json::to_string_pretty(self).map_err(|error| ModelError::Json(error.to_string()))
    }
}
