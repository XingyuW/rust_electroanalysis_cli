//! Versioned configuration wrapper for model definitions.
//!
//! Configuration loading is outside `model`; this module is deliberately not
//! imported by the dependency-clean core.

use crate::model::{MODEL_DEFINITION_SCHEMA_VERSION, ModelDefinition, ModelError};
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
        if self.model.schema_version != MODEL_DEFINITION_SCHEMA_VERSION {
            return Err(ModelError::UnsupportedSchemaVersion {
                found: self.model.schema_version,
                expected: MODEL_DEFINITION_SCHEMA_VERSION,
            });
        }
        self.model.validate_schema()
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ModelError> {
        let text = fs::read_to_string(path).map_err(|error| ModelError::Io(error.to_string()))?;
        let config =
            toml::from_str::<Self>(&text).map_err(|error| ModelError::Toml(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }
}
