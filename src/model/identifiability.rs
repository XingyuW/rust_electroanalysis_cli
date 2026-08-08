use serde::{Deserialize, Serialize};

/// Declarative structural information consumed by a later identifiability
/// adapter. It deliberately makes no empirical claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiabilityMetadata {
    pub states_requiring_independent_observations: Vec<String>,
    pub parameter_requirements: Vec<ParameterIdentifiabilityRequirement>,
    pub component_sensitivity_targets: Vec<String>,
    #[serde(default)]
    pub component_requirements: Vec<IdentifiabilityRequirement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifiabilityRequirementKind {
    ActivityExcitation,
    TransientExcitation,
    ObservationDurationRelativeToTimescale,
    ModeSeparation,
    ReferenceAnchor,
    IndependentCovariateVariation,
    InterferentVariation,
    TemperatureVariation,
    RepeatedStandards,
    AuxiliaryObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementSeverity {
    Required,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiabilityRequirement {
    pub requirement_id: String,
    #[serde(default)]
    pub scope: IdentifiabilityScope,
    pub component_id: String,
    #[serde(default)]
    pub component_ids: Vec<String>,
    pub kind: IdentifiabilityRequirementKind,
    #[serde(default)]
    pub target_states: Vec<String>,
    #[serde(default)]
    pub target_parameters: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub quantitative_criterion: Option<String>,
    pub severity: RequirementSeverity,
}

/// Whether the requirement is emitted by the compiled graph or by an optional
/// capability advertised by a model profile.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IdentifiabilityScope {
    #[default]
    Active,
    Conditional {
        component_kind: String,
        activation_condition: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterIdentifiabilityRequirement {
    pub parameter_id: String,
    pub requirements: Vec<String>,
}

/// Explicit assessment state that preserves a lack of evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatus {
    NotAssessed,
    Supported,
    Contradicted,
    Indeterminate,
}

/// Placeholder report interface for structural/practical identifiability.
/// Phase 02 intentionally does not infer identifiability from a fit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiabilityReport {
    pub structural: AssessmentStatus,
    pub practical: AssessmentStatus,
    pub parameter_ids: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub warnings: Vec<String>,
}

impl IdentifiabilityReport {
    pub(crate) fn not_assessed(parameter_ids: Vec<String>) -> Self {
        Self {
            structural: AssessmentStatus::NotAssessed,
            practical: AssessmentStatus::NotAssessed,
            parameter_ids,
            contradictory_evidence: Vec::new(),
            missing_evidence: vec![
                "No structural or practical identifiability analysis is implemented in Phase 02."
                    .into(),
            ],
            warnings: Vec::new(),
        }
    }
}
