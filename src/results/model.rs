//! Durable artifacts for compiled ISM model definitions.

use crate::model::{
    CompiledIsmModel, ComponentContribution, EquilibriumAssessment, IdentifiabilityReport,
    ModelDefinition, ModelError, PredictionUncertainty, ValidityReport,
};
use serde::{Deserialize, Serialize};

pub const MODEL_RESULT_SCHEMA_VERSION: u32 = 4;
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
        self.validate_finite()?;
        serde_json::to_string_pretty(self).map_err(|error| ModelError::Json(error.to_string()))
    }

    /// Validates every numeric leaf before serde's JSON encoder can map NaN
    /// or infinity to `null`. Paths are stable diagnostic locations.
    pub fn validate_finite(&self) -> Result<(), ModelError> {
        for (point_index, point) in self.points.iter().enumerate() {
            let point_path = format!("points[{point_index}]");
            finite(point.time_s, &format!("{point_path}.time_s"))?;
            finite(
                point.predicted_voltage_v,
                &format!("{point_path}.predicted_voltage_v"),
            )?;
            optional_finite(
                point.observed_voltage_v,
                &format!("{point_path}.observed_voltage_v"),
            )?;
            optional_finite(
                point.unexplained_residual_v,
                &format!("{point_path}.unexplained_residual_v"),
            )?;
            for (state_index, (_, value)) in point.state_values.iter().enumerate() {
                finite(
                    *value,
                    &format!("{point_path}.state_values[{state_index}].1"),
                )?;
            }
            for (contribution_index, contribution) in point.contributions.iter().enumerate() {
                let contribution_path = format!("{point_path}.contributions[{contribution_index}]");
                optional_finite(
                    contribution.potential_v,
                    &format!("{contribution_path}.potential_v"),
                )?;
                optional_finite(
                    contribution.variance_v2,
                    &format!("{contribution_path}.variance_v2"),
                )?;
                for (name, value) in &contribution.auxiliary_outputs {
                    finite(
                        *value,
                        &format!("{contribution_path}.auxiliary_outputs[{name:?}]"),
                    )?;
                }
            }
            for (field, value) in [
                ("total_variance_v2", point.uncertainty.total_variance_v2),
                ("standard_error_v", point.uncertainty.standard_error_v),
                ("state_variance_v2", point.uncertainty.state_variance_v2),
                (
                    "parameter_variance_v2",
                    point.uncertainty.parameter_variance_v2,
                ),
                (
                    "observation_variance_v2",
                    point.uncertainty.observation_variance_v2,
                ),
            ] {
                optional_finite(value, &format!("{point_path}.uncertainty.{field}"))?;
            }
            finite(
                point.equilibrium.confidence,
                &format!("{point_path}.equilibrium.confidence"),
            )?;
        }
        Ok(())
    }
}

fn finite(value: f64, path: &str) -> Result<(), ModelError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ModelError::NonFiniteResult { path: path.into() })
    }
}

fn optional_finite(value: Option<f64>, path: &str) -> Result<(), ModelError> {
    value.map_or(Ok(()), |value| finite(value, path))
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
