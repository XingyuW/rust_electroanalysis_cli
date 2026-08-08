use super::error::ModelError;
use crate::potentiometry::units::{QuantityDimension, QuantityUnit};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};

/// Declares an externally supplied model input and its required unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSpec {
    pub id: String,
    pub unit: String,
    pub required: bool,
    pub source: String,
    pub validity_domain: String,
}

impl InputSpec {
    pub(crate) fn validate(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty()
            || self.source.trim().is_empty()
            || self.validity_domain.trim().is_empty()
        {
            return Err(ModelError::EmptyIdentifier { kind: "input" });
        }
        validate_unit(&self.unit, format!("input '{}'", self.id)).map(|_| ())
    }
}

/// A component's declared dependency on a named model input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRequirement {
    pub id: String,
    pub unit: String,
}

/// A finite runtime input value with its source unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputValue {
    pub value: f64,
    pub unit: String,
}

/// Runtime inputs supplied for one transition or observation evaluation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelInput {
    pub time_s: f64,
    pub values: BTreeMap<String, InputValue>,
}

impl ModelInput {
    pub fn empty(time_s: f64) -> Self {
        Self {
            time_s,
            values: BTreeMap::new(),
        }
    }
}

/// Unit dimensions accepted by the core. Existing potentiometry units are
/// adapted rather than redefined; time/rate units are structural core units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelUnitDimension {
    Concentration,
    Activity,
    Potential,
    PotentialVariance,
    Temperature,
    Conductivity,
    Time,
    PotentialRate,
    Flow,
    TemperatureSensitivity,
    ConductivitySensitivity,
    FlowSensitivity,
    Custom,
    CustomSensitivity,
}

/// The deliberately small unit grammar needed by V1 covariates.  It keeps
/// custom symbols typed and comparable without introducing symbolic algebra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelUnitExpression {
    Base(UnitAtom),
    Ratio {
        numerator: UnitAtom,
        denominator: UnitAtom,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitAtom {
    Known(String),
    Custom {
        symbol: String,
        normalized_symbol: String,
    },
}

pub(crate) fn validate_unit(unit: &str, subject: String) -> Result<ModelUnitDimension, ModelError> {
    unit_dimension(unit).ok_or_else(|| ModelError::InvalidUnit {
        subject,
        unit: unit.to_string(),
    })
}

pub(crate) fn units_compatible(expected: &str, found: &str) -> bool {
    unit_expression(expected) == unit_expression(found)
}

/// The only compound units required by the V1 linear-covariate contract.
/// This intentionally is not a general-purpose unit algebra.
pub(crate) fn potential_sensitivity_unit(input_unit: &str) -> Option<String> {
    let atom = match unit_expression(input_unit)? {
        ModelUnitExpression::Base(atom) => atom,
        ModelUnitExpression::Ratio { .. } => return None,
    };
    Some(match atom {
        UnitAtom::Known(symbol) if symbol == "K" => "V/K".into(),
        UnitAtom::Known(symbol) if symbol == "S/m" => "V/(S/m)".into(),
        UnitAtom::Known(symbol) if symbol == "m/s" => "V/(m/s)".into(),
        UnitAtom::Known(symbol) => format!("V/{symbol}"),
        UnitAtom::Custom {
            normalized_symbol, ..
        } => format!("V/{normalized_symbol}"),
    })
}

fn unit_expression(unit: &str) -> Option<ModelUnitExpression> {
    let trimmed = unit.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(denominator) = trimmed.strip_prefix("V/") {
        let denominator = denominator
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .unwrap_or(denominator);
        return Some(ModelUnitExpression::Ratio {
            numerator: atom("V")?,
            denominator: atom(denominator)?,
        });
    }
    atom(trimmed).map(ModelUnitExpression::Base)
}

fn atom(unit: &str) -> Option<UnitAtom> {
    let normalized = normalize_unit(unit);
    if normalized.is_empty() {
        return None;
    }
    let known = match normalized.to_ascii_lowercase().as_str() {
        "v" | "volt" | "volts" => Some("V"),
        "k" => Some("K"),
        "s/m" | "s·m^-1" => Some("S/m"),
        "m/s" | "meter/s" | "metre/s" => Some("m/s"),
        "activity" => Some("activity"),
        "dimensionless" => Some("dimensionless"),
        _ => None,
    };
    Some(match known {
        Some(symbol) => UnitAtom::Known(symbol.into()),
        // Custom symbols are deliberately case-sensitive; `%RH` and `ppm`
        // remain scientifically distinct declared units. Micro glyph aliases
        // normalize to the canonical U+00B5 micro sign.
        None => UnitAtom::Custom {
            symbol: unit.trim().into(),
            normalized_symbol: normalized,
        },
    })
}

fn normalize_unit(unit: &str) -> String {
    unit.trim().replace('μ', "µ")
}

fn unit_dimension(unit: &str) -> Option<ModelUnitDimension> {
    match unit.trim().to_ascii_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Some(ModelUnitDimension::Time),
        "v/s" | "volt/s" | "volts/s" => Some(ModelUnitDimension::PotentialRate),
        "v^2" | "volt^2" | "volts^2" => Some(ModelUnitDimension::PotentialVariance),
        "m/s" | "meter/s" | "metre/s" => Some(ModelUnitDimension::Flow),
        "v/k" | "volt/k" => Some(ModelUnitDimension::TemperatureSensitivity),
        "v/(s/m)" | "v/(s·m^-1)" | "v per s/m" => {
            Some(ModelUnitDimension::ConductivitySensitivity)
        }
        "v/(m/s)" | "v per m/s" => Some(ModelUnitDimension::FlowSensitivity),
        _ => QuantityUnit::from_str(unit)
            .ok()
            .map(|unit| match unit.dimension() {
                QuantityDimension::Concentration => ModelUnitDimension::Concentration,
                QuantityDimension::Activity => ModelUnitDimension::Activity,
                QuantityDimension::Potential => ModelUnitDimension::Potential,
                QuantityDimension::Temperature => ModelUnitDimension::Temperature,
                QuantityDimension::Conductivity => ModelUnitDimension::Conductivity,
            })
            .or_else(|| match unit_expression(unit)? {
                ModelUnitExpression::Base(UnitAtom::Custom { .. }) => {
                    Some(ModelUnitDimension::Custom)
                }
                ModelUnitExpression::Ratio {
                    numerator: UnitAtom::Known(numerator),
                    denominator: UnitAtom::Custom { .. },
                } if numerator == "V" => Some(ModelUnitDimension::CustomSensitivity),
                _ => None,
            }),
    }
}
