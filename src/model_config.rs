//! Versioned configuration wrapper for model definitions.
//!
//! Configuration loading is outside `model`; this module is deliberately not
//! imported by the dependency-clean core.

use crate::model::{
    ComponentRole, ContributionSemantics, MODEL_DEFINITION_SCHEMA_VERSION, ModelDefinition,
    ModelError,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const MODEL_CONFIG_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub schema_version: u32,
    pub model: ModelDefinition,
}

impl ModelConfig {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.schema_version != MODEL_CONFIG_SCHEMA_VERSION {
            return Err(ModelError::UnsupportedConfigSchemaVersion {
                found: self.schema_version,
                expected: MODEL_CONFIG_SCHEMA_VERSION,
            });
        }
        if !(1..=MODEL_DEFINITION_SCHEMA_VERSION).contains(&self.model.schema_version) {
            return Err(ModelError::UnsupportedSchemaVersion {
                found: self.model.schema_version,
                expected: MODEL_DEFINITION_SCHEMA_VERSION,
            });
        }
        self.model.validate_schema()
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ModelError> {
        let text = fs::read_to_string(path).map_err(|error| ModelError::Io(error.to_string()))?;
        let mut config =
            toml::from_str::<Self>(&text).map_err(|error| ModelError::Toml(error.to_string()))?;
        config.migrate_legacy_model_definition();
        config.validate()?;
        Ok(config)
    }

    /// Schema-v1 did not carry typed contribution semantics. Its documented
    /// observation-noise role is migrated to V² variance; components with an
    /// old voltage owner become additive, and the remaining components are
    /// state-only. Legacy uncertainty stays explicitly incomplete.
    fn migrate_legacy_model_definition(&mut self) {
        if self.model.schema_version != 1 {
            return;
        }
        self.model.uncertainty_incomplete = true;
        for component in &mut self.model.components {
            component.contribution_semantics = if component.role == ComponentRole::ObservationNoise
            {
                component.output_unit = Some("V^2".into());
                component.voltage_contribution_owner = None;
                ContributionSemantics::ObservationVariance
            } else if component.voltage_contribution_owner.is_some() {
                ContributionSemantics::AdditivePotential
            } else {
                ContributionSemantics::StateOnly
            };
        }
    }
}
