//! Durable artifacts for compiled ISM model definitions.

use crate::model::{
    CompiledIsmModel, IdentifiabilityReport, ModelDefinition, ModelError, ValidityReport,
};
use serde::{Deserialize, Serialize};

pub const MODEL_RESULT_SCHEMA_VERSION: u32 = 1;
pub const MODEL_COMPILATION_ARTIFACT_KIND: &str = "ism_model_compilation";

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
