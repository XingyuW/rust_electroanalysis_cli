use super::{error::ModelError, input::ModelInput, parameter::ParameterValues, state::ModelState};
use serde::{Deserialize, Serialize};

/// Scientific role of a component's explicit voltage contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentRole {
    Equilibrium,
    Transport,
    Transduction,
    Reference,
    External,
}

/// Declarative component metadata used for graph and unit validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDescriptor {
    pub id: String,
    pub kind: String,
    pub role: ComponentRole,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub required_inputs: Vec<super::input::InputRequirement>,
    #[serde(default)]
    pub state_ids: Vec<String>,
    #[serde(default)]
    pub parameter_ids: Vec<String>,
    /// Output unit for an observation-producing component. Voltage contributions
    /// must use a unit compatible with volts.
    pub output_unit: Option<String>,
    /// Stable owner name for one explicit contribution. This is never an
    /// unexplained-residual owner.
    pub voltage_contribution_owner: Option<String>,
    pub source: String,
    pub validity_domain: String,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

impl ComponentDescriptor {
    pub(crate) fn validate_shape(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier { kind: "component" });
        }
        if self.kind.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier {
                kind: "component kind",
            });
        }
        if self.source.trim().is_empty() || self.validity_domain.trim().is_empty() {
            return Err(ModelError::EmptyIdentifier {
                kind: "component source or validity domain",
            });
        }
        if self.voltage_contribution_owner.is_some() && self.output_unit.is_none() {
            return Err(ModelError::InvalidUnit {
                subject: format!("voltage component '{}'", self.id),
                unit: "missing output unit".into(),
            });
        }
        Ok(())
    }
}

/// Dense row-major Jacobian representation used by the core without coupling
/// it to a particular estimation library.
pub type Jacobian = Vec<Vec<f64>>;

/// Runtime implementation created by a static component factory.
pub trait IsmComponent: Send + Sync {
    fn descriptor(&self) -> &ComponentDescriptor;

    fn initialize(
        &self,
        _state: &mut ModelState,
        _parameters: &ParameterValues,
    ) -> Result<(), ModelError> {
        Ok(())
    }

    fn process_transition(
        &self,
        _state: &mut ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
        _dt_s: f64,
    ) -> Result<(), ModelError> {
        Ok(())
    }

    fn process_jacobian(
        &self,
        state_dimension: usize,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
        _dt_s: f64,
    ) -> Result<Jacobian, ModelError> {
        Ok(identity(state_dimension))
    }

    /// Returns an explicit voltage contribution, or `None` for a state-only
    /// component. The compiler prevents two components from owning the same
    /// contribution name.
    fn observation_voltage(
        &self,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<Option<f64>, ModelError> {
        Ok(None)
    }

    fn observation_jacobian(
        &self,
        state_dimension: usize,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<Vec<f64>, ModelError> {
        Ok(vec![0.0; state_dimension])
    }
}

pub(crate) fn identity(size: usize) -> Jacobian {
    let mut result = vec![vec![0.0; size]; size];
    for (index, row) in result.iter_mut().enumerate() {
        row[index] = 1.0;
    }
    result
}
