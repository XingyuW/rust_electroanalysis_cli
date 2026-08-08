use super::error::ModelError;
use super::input::validate_unit;
use serde::{Deserialize, Serialize};

/// Transformation between stored and physical state coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StateTransformation {
    #[default]
    Identity,
    Log,
    Logit,
    Custom(String),
}

/// How an initial state value was obtained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StateInitializationSource {
    #[default]
    DeclaredDefault,
    Calibration,
    Measurement,
    External,
    Estimated,
}

/// How uncertainty for a state is represented without requiring an estimator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyRepresentation {
    #[default]
    NotSpecified,
    StandardDeviation,
    Variance,
    Covariance,
    PriorDistribution,
}

/// Versioned metadata and constraints for one latent model state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSpec {
    pub id: String,
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_description")]
    pub description: String,
    pub unit: String,
    #[serde(default)]
    pub transformation: StateTransformation,
    #[serde(default)]
    pub initialization_source: StateInitializationSource,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub initial_value: f64,
    pub source: String,
    #[serde(default = "default_equation_version")]
    pub process_equation_version: u32,
    #[serde(default)]
    pub observability_requirements: Vec<String>,
    pub validity_domain: String,
    #[serde(default)]
    pub uncertainty_representation: UncertaintyRepresentation,
}

const fn default_equation_version() -> u32 {
    1
}

fn default_name() -> String {
    "unspecified state name (schema-v1 migration)".into()
}

fn default_description() -> String {
    "No state description was present in the schema-v1 definition.".into()
}

impl StateSpec {
    pub(crate) fn validate(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier { kind: "state" });
        }
        if self.name.trim().is_empty() || self.description.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier {
                kind: "state name or description",
            });
        }
        if self.process_equation_version == 0 {
            return Err(ModelError::EmptyIdentifier {
                kind: "state process equation version",
            });
        }
        validate_unit(&self.unit, format!("state '{}'", self.id))?;
        if !self.lower_bound.is_finite()
            || !self.upper_bound.is_finite()
            || self.lower_bound > self.upper_bound
        {
            return Err(ModelError::InvalidBounds {
                kind: "state",
                id: self.id.clone(),
                lower: self.lower_bound,
                upper: self.upper_bound,
            });
        }
        if !self.initial_value.is_finite() {
            return Err(ModelError::NonFinite {
                subject: format!("state '{}' initial value", self.id),
            });
        }
        if self.initial_value < self.lower_bound || self.initial_value > self.upper_bound {
            return Err(ModelError::BoundViolation {
                kind: "state",
                id: self.id.clone(),
                value: self.initial_value,
                lower: self.lower_bound,
                upper: self.upper_bound,
            });
        }
        if self.source.trim().is_empty() || self.validity_domain.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier {
                kind: "state source or validity domain",
            });
        }
        Ok(())
    }
}

/// Stable compiled position for a state.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledStateSpec {
    pub index: usize,
    pub spec: StateSpec,
}

/// State values ordered by compiled state index.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelState {
    pub values: Vec<f64>,
}

impl ModelState {
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}
