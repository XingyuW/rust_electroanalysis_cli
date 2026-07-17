//! Time-domain transient model definitions and constrained evaluation.

use crate::potentiometry::PotentiometryError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransientModelKind {
    Single,
    Double,
    DoubleDrift,
    Stretched,
}

impl TransientModelKind {
    pub const ALL: [Self; 4] = [
        Self::Single,
        Self::Double,
        Self::DoubleDrift,
        Self::Stretched,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Double => "double",
            Self::DoubleDrift => "double-drift",
            Self::Stretched => "stretched",
        }
    }

    pub fn parameter_names(self) -> &'static [&'static str] {
        match self {
            Self::Single => &["E_infinity", "A", "tau"],
            Self::Double => &["E_infinity", "A_fast", "A_slow", "tau_fast", "tau_slow"],
            Self::DoubleDrift => &[
                "E_infinity",
                "A_fast",
                "A_slow",
                "tau_fast",
                "tau_slow",
                "drift",
            ],
            Self::Stretched => &["E_infinity", "A", "tau", "beta"],
        }
    }

    pub fn parameter_units(self) -> &'static [&'static str] {
        match self {
            Self::Single => &["V", "V", "s"],
            Self::Double => &["V", "V", "V", "s", "s"],
            Self::DoubleDrift => &["V", "V", "V", "s", "s", "V/s"],
            Self::Stretched => &["V", "V", "s", "dimensionless"],
        }
    }

    pub fn parameter_count(self) -> usize {
        self.parameter_names().len()
    }
}

impl std::fmt::Display for TransientModelKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for TransientModelKind {
    type Err = PotentiometryError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "single" => Ok(Self::Single),
            "double" => Ok(Self::Double),
            "double-drift" | "double_drift" => Ok(Self::DoubleDrift),
            "stretched" => Ok(Self::Stretched),
            other => Err(PotentiometryError::invalid(format!(
                "unsupported transient model '{other}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BaselineMethod {
    Mean,
    #[default]
    Median,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
    Absolute,
    #[default]
    BaselineRelative,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelComponents {
    pub total: f64,
    pub equilibrium: f64,
    pub fast: Option<f64>,
    pub slow: Option<f64>,
    pub drift: Option<f64>,
}

pub fn evaluate(
    model: TransientModelKind,
    parameters: &[f64],
    time_local: f64,
) -> Result<ModelComponents, PotentiometryError> {
    if !time_local.is_finite() || time_local < 0.0 {
        return Err(PotentiometryError::invalid(
            "transient evaluation requires finite non-negative local time",
        ));
    }
    if parameters.len() != model.parameter_count()
        || parameters.iter().any(|value| !value.is_finite())
    {
        return Err(PotentiometryError::invalid(format!(
            "model {model} received an invalid parameter vector"
        )));
    }

    let components = match model {
        TransientModelKind::Single => {
            let [equilibrium, amplitude, tau] = parameters else {
                unreachable!();
            };
            validate_tau(*tau)?;
            let fast = amplitude * (-time_local / tau).exp();
            ModelComponents {
                total: equilibrium + fast,
                equilibrium: *equilibrium,
                fast: Some(fast),
                slow: None,
                drift: None,
            }
        }
        TransientModelKind::Double => {
            let [
                equilibrium,
                fast_amplitude,
                slow_amplitude,
                tau_fast,
                tau_slow,
            ] = parameters
            else {
                unreachable!();
            };
            validate_ordered_taus(*tau_fast, *tau_slow)?;
            let fast = fast_amplitude * (-time_local / tau_fast).exp();
            let slow = slow_amplitude * (-time_local / tau_slow).exp();
            ModelComponents {
                total: equilibrium + fast + slow,
                equilibrium: *equilibrium,
                fast: Some(fast),
                slow: Some(slow),
                drift: None,
            }
        }
        TransientModelKind::DoubleDrift => {
            let [
                equilibrium,
                fast_amplitude,
                slow_amplitude,
                tau_fast,
                tau_slow,
                drift_rate,
            ] = parameters
            else {
                unreachable!();
            };
            validate_ordered_taus(*tau_fast, *tau_slow)?;
            let fast = fast_amplitude * (-time_local / tau_fast).exp();
            let slow = slow_amplitude * (-time_local / tau_slow).exp();
            let drift = drift_rate * time_local;
            ModelComponents {
                total: equilibrium + fast + slow + drift,
                equilibrium: *equilibrium,
                fast: Some(fast),
                slow: Some(slow),
                drift: Some(drift),
            }
        }
        TransientModelKind::Stretched => {
            let [equilibrium, amplitude, tau, beta] = parameters else {
                unreachable!();
            };
            validate_tau(*tau)?;
            if !beta.is_finite() || *beta <= 0.0 {
                return Err(PotentiometryError::invalid(
                    "stretched-exponential beta must be positive",
                ));
            }
            let fast = amplitude * (-(time_local / tau).powf(*beta)).exp();
            ModelComponents {
                total: equilibrium + fast,
                equilibrium: *equilibrium,
                fast: Some(fast),
                slow: None,
                drift: None,
            }
        }
    };

    if [
        components.total,
        components.equilibrium,
        components.fast.unwrap_or(0.0),
        components.slow.unwrap_or(0.0),
        components.drift.unwrap_or(0.0),
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return Err(PotentiometryError::invalid(
            "transient model produced a non-finite prediction",
        ));
    }
    Ok(components)
}

pub fn initial_response_rate(model: TransientModelKind, parameters: &[f64]) -> Option<f64> {
    match model {
        TransientModelKind::Single => Some(-parameters[1] / parameters[2]),
        TransientModelKind::Double => {
            Some(-parameters[1] / parameters[3] - parameters[2] / parameters[4])
        }
        TransientModelKind::DoubleDrift => {
            Some(-parameters[1] / parameters[3] - parameters[2] / parameters[4] + parameters[5])
        }
        TransientModelKind::Stretched => {
            if parameters[3] < 1.0 {
                None
            } else {
                Some(-parameters[1] / parameters[2])
            }
        }
    }
}

pub fn validate_tau(tau: f64) -> Result<(), PotentiometryError> {
    if tau.is_finite() && tau > 0.0 {
        Ok(())
    } else {
        Err(PotentiometryError::invalid(
            "transient time constant must be finite and positive",
        ))
    }
}

pub fn validate_ordered_taus(tau_fast: f64, tau_slow: f64) -> Result<(), PotentiometryError> {
    if tau_fast.is_finite() && tau_slow.is_finite() && 0.0 < tau_fast && tau_fast < tau_slow {
        Ok(())
    } else {
        Err(PotentiometryError::invalid(
            "double-exponential time constants must satisfy 0 < tau_fast < tau_slow",
        ))
    }
}
