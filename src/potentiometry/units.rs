//! Centralized physical-unit parsing and conversion for calibration workflows.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum UnitError {
    #[error("unknown unit '{0}'")]
    Unknown(String),
    #[error("unit '{unit}' is not compatible with {expected}")]
    Incompatible { unit: String, expected: String },
    #[error("molar mass is required to convert '{unit}' to mol/L")]
    MissingMolarMass { unit: String },
    #[error("value {value} is not finite for unit '{unit}'")]
    NonFinite { value: f64, unit: String },
    #[error("temperature {value} {unit} is not physically valid")]
    NonPhysicalTemperature { value: f64, unit: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantityUnit {
    MolPerL,
    MmolPerL,
    MicromolPerL,
    MolPerKg,
    MgPerL,
    GPerL,
    Volt,
    Millivolt,
    Kelvin,
    Celsius,
    SiemensPerM,
    MillisiemensPerCm,
    MicrosiemensPerCm,
}

impl QuantityUnit {
    pub fn dimension(self) -> QuantityDimension {
        match self {
            Self::MolPerL
            | Self::MmolPerL
            | Self::MicromolPerL
            | Self::MolPerKg
            | Self::MgPerL
            | Self::GPerL => QuantityDimension::Concentration,
            Self::Volt | Self::Millivolt => QuantityDimension::Potential,
            Self::Kelvin | Self::Celsius => QuantityDimension::Temperature,
            Self::SiemensPerM | Self::MillisiemensPerCm | Self::MicrosiemensPerCm => {
                QuantityDimension::Conductivity
            }
        }
    }

    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::MolPerL => "mol/L",
            Self::MmolPerL => "mmol/L",
            Self::MicromolPerL => "µmol/L",
            Self::MolPerKg => "mol/kg",
            Self::MgPerL => "mg/L",
            Self::GPerL => "g/L",
            Self::Volt => "V",
            Self::Millivolt => "mV",
            Self::Kelvin => "K",
            Self::Celsius => "°C",
            Self::SiemensPerM => "S/m",
            Self::MillisiemensPerCm => "mS/cm",
            Self::MicrosiemensPerCm => "µS/cm",
        }
    }
}

impl FromStr for QuantityUnit {
    type Err = UnitError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.trim().to_ascii_lowercase().replace(['μ', 'µ'], "u");
        let unit = match normalized.as_str() {
            "mol/l" | "mol/litre" | "mol/liter" | "m" | "molar" => Self::MolPerL,
            "mmol/l" | "mmol/litre" | "mmol/liter" | "mm" => Self::MmolPerL,
            "umol/l" | "umol/litre" | "umol/liter" | "um" => Self::MicromolPerL,
            "mol/kg" | "mol/kgsolvent" | "molal" => Self::MolPerKg,
            "mg/l" | "mg/litre" | "mg/liter" => Self::MgPerL,
            "g/l" | "g/litre" | "g/liter" => Self::GPerL,
            "v" | "volt" | "volts" => Self::Volt,
            "mv" | "millivolt" | "millivolts" => Self::Millivolt,
            "k" | "kelvin" => Self::Kelvin,
            "c" | "°c" | "degc" | "celsius" => Self::Celsius,
            "s/m" | "siemens/m" => Self::SiemensPerM,
            "ms/cm" | "millisiemens/cm" => Self::MillisiemensPerCm,
            "us/cm" | "microsiemens/cm" => Self::MicrosiemensPerCm,
            _ => return Err(UnitError::Unknown(value.trim().to_string())),
        };
        Ok(unit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityDimension {
    Concentration,
    Potential,
    Temperature,
    Conductivity,
}

impl QuantityDimension {
    fn name(self) -> &'static str {
        match self {
            Self::Concentration => "concentration",
            Self::Potential => "potential",
            Self::Temperature => "temperature",
            Self::Conductivity => "conductivity",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: QuantityUnit,
}

impl Quantity {
    pub fn new(value: f64, unit: QuantityUnit) -> Result<Self, UnitError> {
        if !value.is_finite() {
            return Err(UnitError::NonFinite {
                value,
                unit: unit.canonical_name().to_string(),
            });
        }
        Ok(Self { value, unit })
    }

    pub fn parse(value: f64, unit: &str) -> Result<Self, UnitError> {
        Self::new(value, unit.parse()?)
    }

    pub fn require_dimension(&self, expected: QuantityDimension) -> Result<(), UnitError> {
        if self.unit.dimension() == expected {
            Ok(())
        } else {
            Err(UnitError::Incompatible {
                unit: self.unit.canonical_name().to_string(),
                expected: expected.name().to_string(),
            })
        }
    }

    pub fn to_molar_concentration(
        &self,
        molar_mass_g_per_mol: Option<f64>,
    ) -> Result<f64, UnitError> {
        self.require_dimension(QuantityDimension::Concentration)?;
        match self.unit {
            QuantityUnit::MolPerL => Ok(self.value),
            QuantityUnit::MmolPerL => Ok(self.value * 1e-3),
            QuantityUnit::MicromolPerL => Ok(self.value * 1e-6),
            QuantityUnit::GPerL => {
                molar_mass(molar_mass_g_per_mol, self.unit).map(|mass| self.value / mass)
            }
            QuantityUnit::MgPerL => {
                molar_mass(molar_mass_g_per_mol, self.unit).map(|mass| self.value / 1000.0 / mass)
            }
            QuantityUnit::MolPerKg => Err(UnitError::Incompatible {
                unit: self.unit.canonical_name().to_string(),
                expected: "molar concentration (mol/L); solvent density is required for mol/kg"
                    .to_string(),
            }),
            _ => unreachable!("dimension was checked above"),
        }
    }

    pub fn to_potential_v(&self) -> Result<f64, UnitError> {
        self.require_dimension(QuantityDimension::Potential)?;
        Ok(match self.unit {
            QuantityUnit::Volt => self.value,
            QuantityUnit::Millivolt => self.value * 1e-3,
            _ => unreachable!(),
        })
    }

    pub fn to_temperature_k(&self) -> Result<f64, UnitError> {
        self.require_dimension(QuantityDimension::Temperature)?;
        let kelvin = match self.unit {
            QuantityUnit::Kelvin => self.value,
            QuantityUnit::Celsius => self.value + 273.15,
            _ => unreachable!(),
        };
        if !kelvin.is_finite() || kelvin <= 0.0 {
            return Err(UnitError::NonPhysicalTemperature {
                value: self.value,
                unit: self.unit.canonical_name().to_string(),
            });
        }
        Ok(kelvin)
    }

    pub fn to_conductivity_s_per_m(&self) -> Result<f64, UnitError> {
        self.require_dimension(QuantityDimension::Conductivity)?;
        Ok(match self.unit {
            QuantityUnit::SiemensPerM => self.value,
            QuantityUnit::MillisiemensPerCm => self.value * 0.1,
            QuantityUnit::MicrosiemensPerCm => self.value * 1e-4,
            _ => unreachable!(),
        })
    }
}

fn molar_mass(value: Option<f64>, unit: QuantityUnit) -> Result<f64, UnitError> {
    let Some(value) = value else {
        return Err(UnitError::MissingMolarMass {
            unit: unit.canonical_name().to_string(),
        });
    };
    if !value.is_finite() || value <= 0.0 {
        return Err(UnitError::MissingMolarMass {
            unit: unit.canonical_name().to_string(),
        });
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{Quantity, QuantityUnit};
    use std::str::FromStr;

    #[test]
    fn converts_supported_units_without_dimension_confusion() {
        assert_eq!(
            Quantity::parse(1.0, "mmol/L")
                .unwrap()
                .to_molar_concentration(None)
                .unwrap(),
            1e-3
        );
        assert_eq!(
            Quantity::parse(18.0, "mg/L")
                .unwrap()
                .to_molar_concentration(Some(18.0))
                .unwrap(),
            1e-3
        );
        assert!(
            (Quantity::parse(1.0, "mS/cm")
                .unwrap()
                .to_conductivity_s_per_m()
                .unwrap()
                - 0.1)
                .abs()
                < 1e-12
        );
        assert!(
            (Quantity::parse(25.0, "°C")
                .unwrap()
                .to_temperature_k()
                .unwrap()
                - 298.15)
                .abs()
                < 1e-12
        );
        assert_eq!(
            QuantityUnit::from_str("uM").unwrap(),
            QuantityUnit::MicromolPerL
        );
    }

    #[test]
    fn requires_molar_mass_for_mass_concentration() {
        assert!(
            Quantity::parse(1.0, "mg/L")
                .unwrap()
                .to_molar_concentration(None)
                .is_err()
        );
    }
}
