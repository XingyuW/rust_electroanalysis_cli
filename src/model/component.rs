use super::{
    error::ModelError, evidence::EvidenceRequirement, input::ModelInput,
    parameter::ParameterValues, state::ModelState,
};
use serde::{Deserialize, Serialize};

/// Stable, schema-persisted identifier for a model component.
///
/// The alias deliberately keeps the on-disk representation compatible with
/// existing version-1 model definitions while making identifier intent clear at
/// public API boundaries.
pub type ComponentId = String;

/// Scientific role of a component's explicit voltage contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentRole {
    Equilibrium,
    Transport,
    Transduction,
    Reference,
    ExternalDisturbance,
    ObservationNoise,
    Auxiliary,
    Unexplained,
    /// Backward-compatible spelling for `ExternalDisturbance`. New
    /// definitions must use `ExternalDisturbance`.
    #[serde(alias = "external")]
    External,
}

/// Strength of the physical interpretation attached to a component.
///
/// This status describes the component interpretation, not fit quality. In
/// particular, an exponential fit begins as `Phenomenological`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationStatus {
    #[default]
    Phenomenological,
    Hypothesized,
    ExperimentallySupported,
    ValidatedForDomain,
}

/// Declarative component metadata used for graph and unit validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDescriptor {
    pub id: ComponentId,
    pub kind: String,
    pub role: ComponentRole,
    #[serde(default)]
    pub interpretation_status: InterpretationStatus,
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
    /// Declares how this contribution combines with other contributions. A
    /// voltage-producing component must state this explicitly so terms cannot
    /// silently overlap semantically.
    #[serde(default = "default_composition_rule")]
    pub composition_rule: Option<String>,
    pub source: String,
    pub validity_domain: String,
    #[serde(default)]
    pub equation: String,
    #[serde(default)]
    pub equation_version: u32,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub evidence_requirements: Vec<EvidenceRequirement>,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

fn default_composition_rule() -> Option<String> {
    // Schema-v1 definitions did not encode composition. Their only supported
    // deterministic observation semantics were additive voltage terms.
    Some("additive_voltage".into())
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
        if self.equation.trim().is_empty() || self.equation_version == 0 {
            return Err(ModelError::EmptyIdentifier {
                kind: "component equation or equation version",
            });
        }
        if self.assumptions.is_empty() || self.evidence_requirements.is_empty() {
            return Err(ModelError::EmptyIdentifier {
                kind: "component assumptions or evidence requirements",
            });
        }
        if self.voltage_contribution_owner.is_some() && self.output_unit.is_none() {
            return Err(ModelError::InvalidUnit {
                subject: format!("voltage component '{}'", self.id),
                unit: "missing output unit".into(),
            });
        }
        if self.voltage_contribution_owner.is_some()
            && self.composition_rule.as_deref().is_none_or(str::is_empty)
        {
            return Err(ModelError::InvalidComponentShape {
                component: self.id.clone(),
                message: "voltage contribution requires an explicit composition rule".into(),
            });
        }
        if matches!(self.role, ComponentRole::Unexplained)
            && (self.voltage_contribution_owner.is_some() || self.output_unit.is_some())
        {
            return Err(ModelError::InvalidComponentShape {
                component: self.id.clone(),
                message: "unexplained residuals are not model components".into(),
            });
        }
        if matches!(self.role, ComponentRole::ObservationNoise)
            && self.voltage_contribution_owner.is_some()
        {
            return Err(ModelError::InvalidComponentShape {
                component: self.id.clone(),
                message: "observation noise must remain separate from deterministic voltage contributions".into(),
            });
        }
        Ok(())
    }
}

/// Dense row-major Jacobian representation used by the core without coupling
/// it to a particular estimation library.
pub type Jacobian = Vec<Vec<f64>>;

#[derive(Debug, Clone, Default)]
pub struct ComponentBindings {
    pub state_indices: std::collections::BTreeMap<String, usize>,
    pub parameter_indices: std::collections::BTreeMap<String, usize>,
}

/// Runtime implementation created by a static component factory.
pub trait IsmComponent: Send + Sync {
    fn descriptor(&self) -> &ComponentDescriptor;

    fn bind(&mut self, _bindings: &ComponentBindings) -> Result<(), ModelError> {
        Ok(())
    }

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

    fn validity_warnings(
        &self,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<Vec<String>, ModelError> {
        Ok(Vec::new())
    }
}

pub(crate) fn identity(size: usize) -> Jacobian {
    let mut result = vec![vec![0.0; size]; size];
    for (index, row) in result.iter_mut().enumerate() {
        row[index] = 1.0;
    }
    result
}
