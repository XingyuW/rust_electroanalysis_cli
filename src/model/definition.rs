use super::{
    component::ComponentDescriptor, error::ModelError, input::InputSpec, parameter::ParameterSpec,
    state::StateSpec,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MODEL_DEFINITION_SCHEMA_VERSION: u32 = 4;

/// Portable, versioned description of an ISM model graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelDefinition {
    pub schema_version: u32,
    pub model_id: String,
    pub description: String,
    pub validity_domain: String,
    /// Legacy/migration metadata recording that an older artifact was missing
    /// uncertainty. It never bypasses current validation and is never
    /// interpreted as zero uncertainty.
    #[serde(default)]
    pub uncertainty_incomplete: bool,
    pub states: Vec<StateSpec>,
    pub parameters: Vec<ParameterSpec>,
    pub inputs: Vec<InputSpec>,
    pub components: Vec<ComponentDescriptor>,
}

impl ModelDefinition {
    pub fn validate_schema(&self) -> Result<(), ModelError> {
        if !(1..=MODEL_DEFINITION_SCHEMA_VERSION).contains(&self.schema_version) {
            return Err(ModelError::UnsupportedSchemaVersion {
                found: self.schema_version,
                expected: MODEL_DEFINITION_SCHEMA_VERSION,
            });
        }
        if self.model_id.trim().is_empty()
            || self.description.trim().is_empty()
            || self.validity_domain.trim().is_empty()
        {
            return Err(ModelError::EmptyIdentifier { kind: "model" });
        }
        validate_unique(self.states.iter().map(|item| item.id.as_str()), "state")?;
        validate_unique(
            self.parameters.iter().map(|item| item.id.as_str()),
            "parameter",
        )?;
        validate_unique(self.inputs.iter().map(|item| item.id.as_str()), "input")?;
        validate_unique(
            self.components.iter().map(|item| item.id.as_str()),
            "component",
        )?;
        for state in &self.states {
            state.validate()?;
            if matches!(
                state.initialization_source,
                super::state::StateInitializationSource::Estimated
            ) && !state.initial_uncertainty.is_positive_finite(&state.unit)
            {
                return Err(ModelError::InvalidUncertainty {
                    subject: format!("estimated state '{}'", state.id),
                });
            }
        }
        for parameter in &self.parameters {
            parameter.validate()?;
            if matches!(
                parameter.value_source,
                super::parameter::ParameterValueSource::Fitted
            ) && !parameter.uncertainty.is_positive_finite(&parameter.unit)
            {
                return Err(ModelError::InvalidUncertainty {
                    subject: format!("fitted parameter '{}'", parameter.id),
                });
            }
        }
        for input in &self.inputs {
            input.validate()?;
        }
        for component in &self.components {
            component.validate_shape()?;
        }
        Ok(())
    }
}

fn validate_unique<'a>(
    ids: impl Iterator<Item = &'a str>,
    kind: &'static str,
) -> Result<(), ModelError> {
    let mut known = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier { kind });
        }
        if !known.insert(id) {
            return Err(ModelError::DuplicateIdentifier {
                kind,
                id: id.to_string(),
            });
        }
    }
    Ok(())
}
