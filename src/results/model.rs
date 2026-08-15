//! Durable artifacts for compiled ISM model definitions.

use crate::domain::{ArtifactError, validate_serialized_finite};
use crate::model::{
    CompiledIsmModel, ComponentContribution, EquilibriumAssessment, IdentifiabilityReport,
    ModelDefinition, ModelError, PredictionUncertainty, ValidityReport,
};
use serde::{Deserialize, Serialize};

pub const MODEL_RESULT_SCHEMA_VERSION: u32 = 5;
pub const MODEL_COMPILATION_ARTIFACT_KIND: &str = "ism_model_compilation";
pub const MODEL_ANALYSIS_ARTIFACT_KIND: &str = "ism_model_analysis";

/// Serializable record of a validated model definition and its explicit
/// limitations. It contains no fitted values or inferred mechanisms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCompilationArtifact {
    pub schema_version: u32,
    #[serde(default = "crate::domain::legacy_unknown_lineage")]
    pub lineage: crate::domain::ArtifactLineageState,
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
    #[serde(default = "crate::domain::legacy_unknown_lineage")]
    pub lineage: crate::domain::ArtifactLineageState,
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

    /// Uses the same recursive finite-value serializer guard as
    /// `write_artifact`, before serde JSON can map NaN or infinity to `null`.
    pub fn validate_finite(&self) -> Result<(), ModelError> {
        validate_model_artifact_finite(self)
    }
}

fn validate_model_artifact_finite<T: Serialize>(artifact: &T) -> Result<(), ModelError> {
    validate_serialized_finite(artifact).map_err(|error| match error {
        ArtifactError::NonFiniteValue { field_path, .. } => {
            ModelError::NonFiniteResult { path: field_path }
        }
        error => ModelError::Json(error.to_string()),
    })
}

impl ModelCompilationArtifact {
    pub fn from_compiled(model: &CompiledIsmModel) -> Self {
        let mut artifact = Self {
            schema_version: MODEL_RESULT_SCHEMA_VERSION,
            lineage: crate::domain::current_unknown_lineage(MODEL_RESULT_SCHEMA_VERSION),
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
        };
        artifact.lineage = crate::domain::known_lineage_from_artifact(
            crate::domain::ArtifactKind::ModelCompilation,
            artifact.schema_version,
            format!("rust_electroanalysis_cli@{}", env!("CARGO_PKG_VERSION")),
            crate::domain::ArtifactExperimentScope::Unknown,
            crate::domain::ScopeKey::Unspecified,
            crate::domain::ScopeKey::Unspecified,
            crate::domain::ArtifactAcquisitionFamilies::Unknown,
            Vec::new(),
            &artifact,
        )
        .unwrap_or_else(|_| crate::domain::current_unknown_lineage(MODEL_RESULT_SCHEMA_VERSION));
        artifact
    }

    /// The supported serialization path validates all numeric definition
    /// fields before serde can turn a non-finite float into JSON `null`.
    pub fn to_json(&self) -> Result<String, ModelError> {
        self.model_definition.validate_schema()?;
        validate_model_artifact_finite(self)?;
        serde_json::to_string_pretty(self).map_err(|error| ModelError::Json(error.to_string()))
    }

    pub fn validate_finite(&self) -> Result<(), ModelError> {
        validate_model_artifact_finite(self)
    }
}
