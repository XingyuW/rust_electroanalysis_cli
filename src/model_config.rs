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

    /// Legacy definitions are enriched only with structural information that
    /// is unambiguous. Numeric uncertainty has already deserialized to
    /// `Unknown`, so validation still requires user-supplied enrichment.
    fn migrate_legacy_model_definition(&mut self) {
        if self.model.schema_version >= MODEL_DEFINITION_SCHEMA_VERSION {
            return;
        }
        let legacy_version = self.model.schema_version;
        self.model.uncertainty_incomplete = true;
        for component in &mut self.model.components {
            if legacy_version == 1 {
                component.contribution_semantics =
                    if component.role == ComponentRole::ObservationNoise {
                        component.output_unit = Some("V^2".into());
                        component.voltage_contribution_owner = None;
                        ContributionSemantics::ObservationVariance
                    } else if component.voltage_contribution_owner.is_some() {
                        ContributionSemantics::AdditivePotential
                    } else {
                        ContributionSemantics::StateOnly
                    };
            }
            if component.contribution_semantics == ContributionSemantics::AdditivePotential {
                match component.kind.as_str() {
                    "transport.first_order_relaxation"
                    | "transport.two_mode_relaxation"
                    | "transport.stretched_relaxation"
                    | "transport.partition_delay"
                    | "transduction.solid_contact_rc_candidate"
                    | "transduction.interfacial_polarization_candidate"
                    | "disturbance.baseline_random_walk" => {
                        component.observation_state_ids = component.state_ids.clone();
                    }
                    "equilibrium.nernst"
                    | "equilibrium.nicolsky_eisenman"
                    | "transduction.ideal"
                    | "disturbance.linear_drift"
                    | "disturbance.temperature_covariate"
                    | "disturbance.conductivity_covariate"
                    | "disturbance.flow_covariate" => {
                        component.observation_parameter_ids = component.parameter_ids.clone();
                    }
                    _ => {
                        component.observation_state_ids = component.state_ids.clone();
                        component.observation_parameter_ids = component.parameter_ids.clone();
                    }
                }
            }
        }
        self.model.schema_version = MODEL_DEFINITION_SCHEMA_VERSION;
    }
}
