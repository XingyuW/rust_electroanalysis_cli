//! Dependency-clean contracts for future unified ion-selective-membrane models.
//!
//! This core owns model definitions, graph compilation, state/parameter
//! validation, and explicit voltage decomposition. It intentionally contains
//! no Nernst, transient, EIS, or other scientific equation implementation.

mod builtins;
mod compiler;
mod component;
mod defaults;
mod definition;
mod equilibrium_recognition;
mod error;
mod evidence;
mod graph;
mod identifiability;
mod input;
mod output;
mod parameter;
mod registry;
mod state;
mod validity;

pub use compiler::{
    CompiledBindingSummary, CompiledIsmModel, CompiledModelSummary, ComponentBindingSummary,
    compile_model,
};
pub use component::{
    ComponentBindings, ComponentDescriptor, ComponentId, ComponentRole, ContributionSemantics,
    InterpretationStatus, IsmComponent, Jacobian, JacobianMethod, JacobianStatus, ParameterId,
    ParameterJacobian, StateId, StateJacobian, StateJacobianStatus,
};
pub use defaults::default_model_definition;
pub use definition::{MODEL_DEFINITION_SCHEMA_VERSION, ModelDefinition};
pub use equilibrium_recognition::{
    EquilibriumAssessment, EquilibriumEvidenceRequirements, EquilibriumStatus,
};
pub use error::ModelError;
pub use evidence::{EvidenceAssessment, EvidenceAssessmentStatus, EvidenceRequirement};
pub use identifiability::{
    AssessmentStatus, IdentifiabilityMetadata, IdentifiabilityReport,
    ParameterIdentifiabilityRequirement,
};
pub use input::{InputRequirement, InputSpec, InputValue, ModelInput};
pub use output::{
    ComponentContribution, ContributionTotals, DEFAULT_POTENTIAL_RECONSTRUCTION_TOLERANCE_V,
    ModelPrediction, ModelWarning, ObservationPrediction, PredictionUncertainty,
    PredictionUncertaintyInput, UncertaintyStatus, UnexplainedResidual,
};
pub use parameter::{CompiledParameterSpec, ParameterSpec, ParameterValueSource, ParameterValues};
pub use registry::{ComponentFactory, ComponentRegistry, built_in_registry};
pub use state::{
    CompiledStateSpec, DeclaredUncertaintyClass, InitializationContext, InitializedModelState,
    ModelState, StateInitializationSource, StateSpec, StateTransformation, UncertaintySpec,
};
pub use validity::{ComponentValidityReport, ValidityDomain, ValidityReport, ValidityStatus};

/// Public name for the framework's model implementation contract.
pub trait IsmModel {
    fn definition(&self) -> &ModelDefinition;
    fn state_definitions(&self) -> &[CompiledStateSpec];
    fn parameter_definitions(&self) -> &[CompiledParameterSpec];
}

impl IsmModel for CompiledIsmModel {
    fn definition(&self) -> &ModelDefinition {
        self.definition()
    }

    fn state_definitions(&self) -> &[CompiledStateSpec] {
        self.state_definitions()
    }

    fn parameter_definitions(&self) -> &[CompiledParameterSpec] {
        self.parameter_definitions()
    }
}
