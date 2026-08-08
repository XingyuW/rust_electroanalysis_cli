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

/// Stable, schema-persisted identifier for a model state.
pub type StateId = String;

/// Stable, schema-persisted identifier for a model parameter.
pub type ParameterId = String;

/// Scientific role of a component's explicit voltage contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentRole {
    Equilibrium,
    Transport,
    Transduction,
    Reference,
    /// `external` was emitted by schema-v1 definitions. It deserializes into
    /// the single canonical runtime role; serialization always emits
    /// `external_disturbance`.
    #[serde(alias = "external")]
    ExternalDisturbance,
    ObservationNoise,
    Auxiliary,
    Unexplained,
}

/// Closed numerical semantics for a component observation.
///
/// This is deliberately not a string: only additive potentials participate in
/// the deterministic voltage reconstruction.  Observation noise is represented
/// as variance in V², and state-only/auxiliary components have no numerical
/// observation contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContributionSemantics {
    #[default]
    AdditivePotential,
    ObservationVariance,
    StateOnly,
    Auxiliary,
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
    /// States with a direct derivative in this component's scalar observation.
    /// Process-only states are intentionally excluded.
    #[serde(default)]
    pub observation_state_ids: Vec<StateId>,
    /// Parameters with a direct derivative in this component's scalar
    /// observation. Process-only parameters are intentionally excluded.
    #[serde(default)]
    pub observation_parameter_ids: Vec<ParameterId>,
    /// Numerical observation derivatives are rejected unless explicitly
    /// declared. Built-ins leave this false and provide analytical coverage.
    #[serde(default)]
    pub numerical_jacobian_supported: bool,
    /// Output unit for an observation-producing component. Additive-potential
    /// outputs use volts and observation-variance outputs use V².
    pub output_unit: Option<String>,
    /// Stable owner name for one explicit contribution. This is never an
    /// unexplained-residual owner.
    pub voltage_contribution_owner: Option<String>,
    /// Explicit, closed composition contract for the component output.
    #[serde(default)]
    pub contribution_semantics: ContributionSemantics,
    /// Read-only schema-v1 migration input. It is never used for numerical
    /// dispatch and is not emitted by current serialization.
    #[serde(default, rename = "composition_rule", skip_serializing)]
    pub legacy_composition_rule: Option<String>,
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
        if let Some(rule) = &self.legacy_composition_rule
            && !matches!(rule.as_str(), "additive_voltage" | "additive_potential")
        {
            return Err(ModelError::UnsupportedCompositionSemantics {
                component: self.id.clone(),
                semantics: rule.clone(),
            });
        }
        for state_id in &self.observation_state_ids {
            if !self.state_ids.contains(state_id) {
                return Err(ModelError::InvalidComponentShape {
                    component: self.id.clone(),
                    message: format!("observation state '{state_id}' is not declared in state_ids"),
                });
            }
        }
        for parameter_id in &self.observation_parameter_ids {
            if !self.parameter_ids.contains(parameter_id) {
                return Err(ModelError::InvalidComponentShape {
                    component: self.id.clone(),
                    message: format!(
                        "observation parameter '{parameter_id}' is not declared in parameter_ids"
                    ),
                });
            }
        }
        if self
            .observation_state_ids
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != self.observation_state_ids.len()
            || self
                .observation_parameter_ids
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.observation_parameter_ids.len()
        {
            return Err(ModelError::InvalidComponentShape {
                component: self.id.clone(),
                message: "direct observation derivative declarations contain duplicate IDs".into(),
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
        match self.contribution_semantics {
            ContributionSemantics::AdditivePotential => {
                if self.voltage_contribution_owner.is_none()
                    || self.output_unit.as_deref() != Some("V")
                {
                    return Err(ModelError::InvalidComponentShape {
                        component: self.id.clone(),
                        message: "additive-potential components require a contribution owner and V output".into(),
                    });
                }
                if matches!(
                    self.role,
                    ComponentRole::ObservationNoise | ComponentRole::Auxiliary
                ) {
                    return Err(ModelError::InvalidComponentShape {
                        component: self.id.clone(),
                        message: "observation-noise and auxiliary components cannot be additive potentials".into(),
                    });
                }
            }
            ContributionSemantics::ObservationVariance => {
                if self.voltage_contribution_owner.is_some()
                    || self.output_unit.as_deref() != Some("V^2")
                {
                    return Err(ModelError::InvalidComponentShape {
                        component: self.id.clone(),
                        message: "observation-variance components require V^2 output and no voltage owner".into(),
                    });
                }
                if !matches!(self.role, ComponentRole::ObservationNoise) {
                    return Err(ModelError::InvalidComponentShape {
                        component: self.id.clone(),
                        message: "only observation-noise components may emit observation variance"
                            .into(),
                    });
                }
            }
            ContributionSemantics::StateOnly | ContributionSemantics::Auxiliary => {
                if self.voltage_contribution_owner.is_some() || self.output_unit.is_some() {
                    return Err(ModelError::InvalidComponentShape {
                        component: self.id.clone(),
                        message: "state-only and auxiliary components cannot declare numerical observation output".into(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Dense row-major Jacobian representation used by the core without coupling
/// it to a particular estimation library.
pub type Jacobian = Vec<Vec<f64>>;

/// How an observation derivative was evaluated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum JacobianMethod {
    Analytic,
    Numerical {
        relative_step: f64,
        absolute_step: f64,
    },
    Mixed,
    NotEvaluated,
}

/// Coverage of the parameters declared to affect a component observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum JacobianStatus {
    Complete,
    Partial {
        missing_parameters: Vec<ParameterId>,
    },
    Unavailable {
        reason: String,
    },
    NotApplicable,
}

/// Local parameter derivative values keyed by stable parameter IDs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterJacobian {
    pub values: Vec<f64>,
    pub covered_parameters: Vec<ParameterId>,
    pub status: JacobianStatus,
    pub method: JacobianMethod,
}

impl ParameterJacobian {
    pub fn analytic(values: impl IntoIterator<Item = (ParameterId, f64)>) -> Self {
        let (covered_parameters, values) = values.into_iter().unzip();
        Self {
            values,
            covered_parameters,
            status: JacobianStatus::Complete,
            method: JacobianMethod::Analytic,
        }
    }

    pub fn not_applicable() -> Self {
        Self {
            values: Vec::new(),
            covered_parameters: Vec::new(),
            status: JacobianStatus::NotApplicable,
            method: JacobianMethod::NotEvaluated,
        }
    }
}

/// Coverage of states declared to affect a component observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum StateJacobianStatus {
    Complete,
    Partial { missing_states: Vec<StateId> },
    Unavailable { reason: String },
    NotApplicable,
}

/// Local state derivative values keyed by stable state IDs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateJacobian {
    pub values: Vec<f64>,
    pub covered_states: Vec<StateId>,
    pub status: StateJacobianStatus,
    pub method: JacobianMethod,
}

impl StateJacobian {
    pub fn analytic(values: impl IntoIterator<Item = (StateId, f64)>) -> Self {
        let (covered_states, values) = values.into_iter().unzip();
        Self {
            values,
            covered_states,
            status: StateJacobianStatus::Complete,
            method: JacobianMethod::Analytic,
        }
    }

    pub fn not_applicable() -> Self {
        Self {
            values: Vec::new(),
            covered_states: Vec::new(),
            status: StateJacobianStatus::NotApplicable,
            method: JacobianMethod::NotEvaluated,
        }
    }
}

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

    /// Returns an observation-noise variance in V². It is intentionally
    /// separate from `observation_voltage` so noise cannot enter the voltage
    /// sum by accident.
    fn observation_variance_v2(
        &self,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<Option<f64>, ModelError> {
        Ok(None)
    }

    /// Local derivative of the additive observation with respect to stable
    /// parameter IDs. Omission is explicit and is never represented by zero.
    fn observation_parameter_jacobian(
        &self,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<ParameterJacobian, ModelError> {
        if self.descriptor().observation_parameter_ids.is_empty() {
            Ok(ParameterJacobian::not_applicable())
        } else {
            Ok(ParameterJacobian {
                values: Vec::new(),
                covered_parameters: Vec::new(),
                status: JacobianStatus::Unavailable {
                    reason: "component did not implement its declared parameter derivative".into(),
                },
                method: JacobianMethod::NotEvaluated,
            })
        }
    }

    fn observation_state_jacobian(
        &self,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<StateJacobian, ModelError> {
        if self.descriptor().observation_state_ids.is_empty() {
            Ok(StateJacobian::not_applicable())
        } else {
            Ok(StateJacobian {
                values: Vec::new(),
                covered_states: Vec::new(),
                status: StateJacobianStatus::Unavailable {
                    reason: "component did not implement its declared state derivative".into(),
                },
                method: JacobianMethod::NotEvaluated,
            })
        }
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
