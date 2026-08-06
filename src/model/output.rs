use super::{component::ComponentRole, error::ModelError};
use serde::{Deserialize, Serialize};

/// One named, explicit voltage contribution to the predicted potential.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentContribution {
    pub component_id: String,
    pub owner: String,
    pub role: ComponentRole,
    pub voltage_v: f64,
    pub source: String,
    pub validity_domain: String,
}

/// Explicit status of the unexplained residual. Missing observed voltage is
/// represented as missing evidence rather than silently omitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status", content = "value_v")]
pub enum UnexplainedResidual {
    Observed(f64),
    MissingObservedVoltage,
}

/// Decomposed model observation with an auditable unexplained residual field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationPrediction {
    pub predicted_voltage_v: f64,
    pub contributions: Vec<ComponentContribution>,
    pub unexplained_residual: UnexplainedResidual,
}

impl ObservationPrediction {
    pub(crate) fn new(
        contributions: Vec<ComponentContribution>,
        observed_voltage_v: Option<f64>,
    ) -> Result<Self, ModelError> {
        let predicted_voltage_v = contributions.iter().try_fold(0.0, |sum, contribution| {
            if contribution.voltage_v.is_finite() {
                Ok(sum + contribution.voltage_v)
            } else {
                Err(ModelError::NonFiniteContribution {
                    component: contribution.component_id.clone(),
                })
            }
        })?;
        if !predicted_voltage_v.is_finite() {
            return Err(ModelError::NonFinite {
                subject: "predicted voltage".into(),
            });
        }
        let unexplained_residual = match observed_voltage_v {
            Some(observed) if observed.is_finite() => {
                UnexplainedResidual::Observed(observed - predicted_voltage_v)
            }
            Some(_) => {
                return Err(ModelError::NonFinite {
                    subject: "observed voltage".into(),
                });
            }
            None => UnexplainedResidual::MissingObservedVoltage,
        };
        Ok(Self {
            predicted_voltage_v,
            contributions,
            unexplained_residual,
        })
    }
}
