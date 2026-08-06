use super::error::ModelError;
use super::input::validate_unit;
use serde::{Deserialize, Serialize};

/// Versioned metadata and constraints for a model parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub id: String,
    pub unit: String,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub default_value: f64,
    pub uncertainty: f64,
    pub source: String,
    pub validity_domain: String,
}

impl ParameterSpec {
    pub(crate) fn validate(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier { kind: "parameter" });
        }
        validate_unit(&self.unit, format!("parameter '{}'", self.id))?;
        if !self.lower_bound.is_finite()
            || !self.upper_bound.is_finite()
            || self.lower_bound > self.upper_bound
        {
            return Err(ModelError::InvalidBounds {
                kind: "parameter",
                id: self.id.clone(),
                lower: self.lower_bound,
                upper: self.upper_bound,
            });
        }
        if !self.default_value.is_finite() {
            return Err(ModelError::NonFinite {
                subject: format!("parameter '{}' default value", self.id),
            });
        }
        if self.default_value < self.lower_bound || self.default_value > self.upper_bound {
            return Err(ModelError::BoundViolation {
                kind: "parameter",
                id: self.id.clone(),
                value: self.default_value,
                lower: self.lower_bound,
                upper: self.upper_bound,
            });
        }
        if !self.uncertainty.is_finite() || self.uncertainty < 0.0 {
            return Err(ModelError::NonFinite {
                subject: format!("parameter '{}' uncertainty", self.id),
            });
        }
        if self.source.trim().is_empty() || self.validity_domain.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier {
                kind: "parameter source or validity domain",
            });
        }
        Ok(())
    }
}

/// Stable compiled position for a parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledParameterSpec {
    pub index: usize,
    pub spec: ParameterSpec,
}

/// Parameter values ordered by compiled parameter index.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterValues {
    pub values: Vec<f64>,
}

impl ParameterValues {
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}
