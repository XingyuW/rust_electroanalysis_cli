use super::input::validate_unit;
use super::{
    error::ModelError,
    state::{DeclaredUncertaintyClass, UncertaintySpec},
};
use serde::{Deserialize, Serialize};

/// How a parameter value enters an evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParameterValueSource {
    #[default]
    Fixed,
    Fitted,
    ExternallySupplied,
    ExternallySuppliedFixed,
}

/// Versioned metadata and constraints for a model parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub id: String,
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_description")]
    pub description: String,
    pub unit: String,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub default_value: f64,
    #[serde(default = "default_unknown_uncertainty")]
    pub uncertainty: UncertaintySpec,
    pub source: String,
    #[serde(default = "default_equation_version")]
    pub equation_version: u32,
    #[serde(default)]
    pub identifiability_requirements: Vec<String>,
    #[serde(default)]
    pub value_source: ParameterValueSource,
    pub validity_domain: String,
}

const fn default_equation_version() -> u32 {
    1
}

fn default_name() -> String {
    "unspecified parameter name (schema-v1 migration)".into()
}

fn default_description() -> String {
    "No parameter description was present in the schema-v1 definition.".into()
}

fn default_unknown_uncertainty() -> UncertaintySpec {
    UncertaintySpec::Unknown {
        reason: "parameter uncertainty was not declared".into(),
    }
}

impl ParameterSpec {
    /// Schema-declared uncertainty semantics.  `value_source` is validated
    /// with this declaration by `ModelDefinition`; covariance cannot override
    /// the resulting class.
    pub const fn declared_uncertainty_class(&self) -> DeclaredUncertaintyClass {
        match self.value_source {
            // Schema validation requires a fitted value to carry a positive,
            // finite typed uncertainty. Keeping the source in this decision
            // prevents future covariance handling from reclassifying it.
            ParameterValueSource::Fitted => DeclaredUncertaintyClass::StochasticKnown,
            ParameterValueSource::Fixed
            | ParameterValueSource::ExternallySupplied
            | ParameterValueSource::ExternallySuppliedFixed => self.uncertainty.declared_class(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier { kind: "parameter" });
        }
        if self.name.trim().is_empty() || self.description.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier {
                kind: "parameter name or description",
            });
        }
        if self.equation_version == 0 {
            return Err(ModelError::EmptyIdentifier {
                kind: "parameter equation version",
            });
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
        self.uncertainty.variance_in(&self.unit)?;
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
