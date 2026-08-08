use super::error::ModelError;
use super::input::validate_unit;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

/// Typed uncertainty declaration. Numeric zero is meaningful only for an
/// explicitly deterministic quantity; unknown uncertainty is never coerced to
/// zero.
#[derive(Debug, Clone, PartialEq)]
pub enum UncertaintySpec {
    Deterministic,
    StandardDeviation { value: f64, unit: String },
    Variance { value: f64, unit: String },
    Unknown { reason: String },
}

/// Semantic uncertainty class declared by the model schema.
///
/// Covariance matrices quantify members of this class but never change it.
/// In particular, an all-zero covariance row is not evidence that a quantity
/// declared stochastic is deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclaredUncertaintyClass {
    Deterministic,
    StochasticKnown,
    StochasticUnknown,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum UncertaintySpecWire {
    Deterministic,
    StandardDeviation { value: f64, unit: String },
    Variance { value: f64, unit: String },
    Unknown { reason: String },
}

impl Serialize for UncertaintySpec {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let wire = match self {
            Self::Deterministic => UncertaintySpecWire::Deterministic,
            Self::StandardDeviation { value, unit } => UncertaintySpecWire::StandardDeviation {
                value: *value,
                unit: unit.clone(),
            },
            Self::Variance { value, unit } => UncertaintySpecWire::Variance {
                value: *value,
                unit: unit.clone(),
            },
            Self::Unknown { reason } => UncertaintySpecWire::Unknown {
                reason: reason.clone(),
            },
        };
        wire.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UncertaintySpec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Typed(UncertaintySpecWire),
            LegacyNumber(f64),
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Typed(UncertaintySpecWire::Deterministic) => Self::Deterministic,
            Wire::Typed(UncertaintySpecWire::StandardDeviation { value, unit }) => {
                Self::StandardDeviation { value, unit }
            }
            Wire::Typed(UncertaintySpecWire::Variance { value, unit }) => {
                Self::Variance { value, unit }
            }
            Wire::Typed(UncertaintySpecWire::Unknown { reason }) => Self::Unknown { reason },
            Wire::LegacyNumber(_legacy_value) => Self::Unknown {
                reason: "legacy numeric uncertainty requires explicit typed migration".into(),
            },
        })
    }
}

impl UncertaintySpec {
    pub const fn declared_class(&self) -> DeclaredUncertaintyClass {
        match self {
            Self::Deterministic => DeclaredUncertaintyClass::Deterministic,
            Self::StandardDeviation { .. } | Self::Variance { .. } => {
                DeclaredUncertaintyClass::StochasticKnown
            }
            Self::Unknown { .. } => DeclaredUncertaintyClass::StochasticUnknown,
        }
    }

    pub fn variance_in(&self, expected_unit: &str) -> Result<Option<f64>, ModelError> {
        match self {
            Self::Deterministic => Ok(Some(0.0)),
            Self::StandardDeviation { value, unit } => {
                if unit != expected_unit || !value.is_finite() || *value <= 0.0 {
                    return Err(ModelError::InvalidUncertainty {
                        subject: expected_unit.into(),
                    });
                }
                Ok(Some(value * value))
            }
            Self::Variance { value, unit } => {
                if unit != &format!("{expected_unit}^2") || !value.is_finite() || *value <= 0.0 {
                    return Err(ModelError::InvalidUncertainty {
                        subject: expected_unit.into(),
                    });
                }
                Ok(Some(*value))
            }
            Self::Unknown { .. } => Ok(None),
        }
    }

    pub fn missing_reason(&self) -> Option<String> {
        match self {
            Self::Unknown { reason } => Some(reason.clone()),
            _ => None,
        }
    }

    pub fn is_positive_finite(&self, expected_unit: &str) -> bool {
        matches!(self.variance_in(expected_unit), Ok(Some(value)) if value.is_finite() && value > 0.0)
    }
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
    #[serde(default = "default_unknown_uncertainty")]
    pub initial_uncertainty: UncertaintySpec,
}

fn default_unknown_uncertainty() -> UncertaintySpec {
    UncertaintySpec::Unknown {
        reason: "initial-state uncertainty was not declared".into(),
    }
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
    /// Schema-declared uncertainty semantics for prediction propagation.
    pub const fn declared_uncertainty_class(&self) -> DeclaredUncertaintyClass {
        match self.initialization_source {
            // Schema validation requires an estimated initial state to have a
            // positive finite typed uncertainty.
            StateInitializationSource::Estimated => DeclaredUncertaintyClass::StochasticKnown,
            StateInitializationSource::DeclaredDefault
            | StateInitializationSource::Calibration
            | StateInitializationSource::Measurement
            | StateInitializationSource::External => self.initial_uncertainty.declared_class(),
        }
    }

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
        self.initial_uncertainty.variance_in(&self.unit)?;
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
