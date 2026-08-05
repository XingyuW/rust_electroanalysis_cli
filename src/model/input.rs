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
    Temperature,
    Conductivity,
    Time,
    PotentialRate,
}

pub(crate) fn validate_unit(unit: &str, subject: String) -> Result<ModelUnitDimension, ModelError> {
    unit_dimension(unit).ok_or_else(|| ModelError::InvalidUnit {
        subject,
        unit: unit.to_string(),
    })
}

pub(crate) fn units_compatible(expected: &str, found: &str) -> bool {
    matches!((unit_dimension(expected), unit_dimension(found)), (Some(left), Some(right)) if left == right)
}

fn unit_dimension(unit: &str) -> Option<ModelUnitDimension> {
    match unit.trim().to_ascii_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Some(ModelUnitDimension::Time),
        "v/s" | "volt/s" | "volts/s" => Some(ModelUnitDimension::PotentialRate),
        _ => QuantityUnit::from_str(unit)
            .ok()
            .map(|unit| match unit.dimension() {
                QuantityDimension::Concentration => ModelUnitDimension::Concentration,
                QuantityDimension::Activity => ModelUnitDimension::Activity,
                QuantityDimension::Potential => ModelUnitDimension::Potential,
                QuantityDimension::Temperature => ModelUnitDimension::Temperature,
                QuantityDimension::Conductivity => ModelUnitDimension::Conductivity,
            }),
    }
}
