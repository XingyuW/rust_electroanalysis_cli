use super::error::ModelError;
use super::input::validate_unit;
use serde::{Deserialize, Serialize};

/// Versioned metadata and constraints for one latent model state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSpec {
    pub id: String,
    pub unit: String,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub initial_value: f64,
    pub source: String,
    pub validity_domain: String,
}

impl StateSpec {
    pub(crate) fn validate(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier { kind: "state" });
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
