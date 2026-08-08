//! Dependency-clean direction adapter from legacy estimation into the compiled ISM graph.

use super::{
    calibration_adapter::{CalibrationObservationModel, StoredCalibrationObservationModel},
    environment::AlignedEnvironment,
    error::EstimationError,
    state::{StateDefinition, StateTransform},
};
use crate::{
    estimation_config::ResolvedEstimationConfig,
    model::{
        ComponentBindings, ComponentDescriptor, ComponentFactory, ComponentRegistry, ComponentRole,
        EvidenceRequirement, InputSpec, InputValue, InterpretationStatus, IsmComponent, Jacobian,
        ModelDefinition, ModelError, ModelInput, ModelState, ParameterSpec, ParameterValueSource,
        ParameterValues, StateInitializationSource, StateJacobian, StateSpec, StateTransformation,
        UncertaintySpec, compile_model,
    },
    results::StoredCalibrationModel,
};
use std::collections::BTreeMap;

pub fn compile_legacy_model(
    config: &ResolvedEstimationConfig,
    definitions: &[StateDefinition],
    tau_s: f64,
    tau_uncertainty_s: Option<f64>,
    calibration: &StoredCalibrationModel,
) -> Result<crate::model::CompiledIsmModel, EstimationError> {
    let definition =
        legacy_model_definition(config, definitions, tau_s, tau_uncertainty_s, calibration)?;
    let registry = ComponentRegistry::from_static_factories([
        ("estimation.legacy_equilibrium", factory as ComponentFactory),
        ("estimation.legacy_baseline", factory as ComponentFactory),
        (
            "estimation.legacy_polarization",
            factory as ComponentFactory,
        ),
        ("estimation.legacy_sensitivity", factory as ComponentFactory),
    ]);
    compile_model(definition, &registry)
        .map_err(|error| EstimationError::config(format!("legacy ISM compilation failed: {error}")))
}

pub fn legacy_model_definition(
    config: &ResolvedEstimationConfig,
    definitions: &[StateDefinition],
    tau_s: f64,
    tau_uncertainty_s: Option<f64>,
    calibration: &StoredCalibrationModel,
) -> Result<ModelDefinition, EstimationError> {
    let calibration_json = serde_json::to_string(calibration)
        .map_err(|error| EstimationError::config(format!("calibration serialization: {error}")))?;
    let activity_definition = definitions
        .first()
        .ok_or_else(|| EstimationError::config("legacy estimator has no activity state"))?;
    let states = definitions
        .iter()
        .map(|state| {
            let (initialization_source, initial_uncertainty) =
                state_initial_uncertainty(config, &state.name);
            StateSpec {
                id: state.name.clone(),
                name: state.name.replace('_', " "),
                description: state.interpretation.clone(),
                unit: if state.name == "log10_activity" {
                    "dimensionless".into()
                } else {
                    state.unit.clone()
                },
                transformation: StateTransformation::Custom(format!("{:?}", state.transform)),
                initialization_source,
                lower_bound: if matches!(state.transform, StateTransform::LogisticBounded) {
                    -30.0
                } else {
                    state.lower_bound.unwrap_or_else(|| {
                        if state.name == "log10_activity" {
                            -30.0
                        } else {
                            -10.0
                        }
                    })
                },
                upper_bound: if matches!(state.transform, StateTransform::LogisticBounded) {
                    30.0
                } else {
                    state.upper_bound.unwrap_or_else(|| {
                        if state.name == "log10_activity" {
                            30.0
                        } else {
                            10.0
                        }
                    })
                },
                initial_value: match state.name.as_str() {
                    "baseline_offset" => config.initialization.baseline_v,
                    "polarization" => config.initialization.polarization_v,
                    "sensitivity_scale"
                        if matches!(state.transform, StateTransform::LogisticBounded) =>
                    {
                        0.0
                    }
                    "sensitivity_scale" => config.initialization.condition_value,
                    _ => 0.0,
                },
                source: "legacy estimation state adapter".into(),
                process_equation_version: 1,
                observability_requirements: vec![
                    "Estimator observability must be retained in the compatibility report.".into(),
                ],
                validity_domain: state.interpretation.clone(),
                initial_uncertainty,
            }
        })
        .collect::<Vec<_>>();
    let mut parameters = Vec::new();
    let mut components = Vec::new();
    components.push(descriptor(
        "legacy.equilibrium",
        "estimation.legacy_equilibrium",
        ComponentRole::Equilibrium,
        vec!["log10_activity"],
        Vec::new(),
        "equilibrium",
        BTreeMap::from([
            ("calibration_json".into(), calibration_json.clone()),
            (
                "activity_transform".into(),
                format!("{:?}", activity_definition.transform),
            ),
        ]),
    ));
    if definitions
        .iter()
        .any(|state| state.name == "baseline_offset")
    {
        components.push(descriptor(
            "legacy.reference.baseline",
            "estimation.legacy_baseline",
            ComponentRole::Reference,
            vec!["baseline_offset"],
            Vec::new(),
            "reference",
            BTreeMap::new(),
        ));
    }
    if definitions.iter().any(|state| state.name == "polarization") {
        parameters.extend([
            parameter(
                "legacy_polarization_tau_s",
                "s",
                1e-12,
                1e12,
                tau_s,
                tau_uncertainty_s.map(|value| UncertaintySpec::StandardDeviation {
                    value,
                    unit: "s".into(),
                }),
            ),
            parameter(
                "legacy_polarization_gain",
                "dimensionless",
                -1e6,
                1e6,
                config.polarization.gain,
                None,
            ),
        ]);
        components.push(descriptor(
            "legacy.transport.polarization",
            "estimation.legacy_polarization",
            ComponentRole::Transport,
            vec!["polarization"],
            vec!["legacy_polarization_tau_s", "legacy_polarization_gain"],
            "transport",
            BTreeMap::new(),
        ));
    }
    if let Some(state) = definitions
        .iter()
        .find(|state| state.name == "sensitivity_scale")
    {
        components.push(descriptor(
            "legacy.transduction.sensitivity",
            "estimation.legacy_sensitivity",
            ComponentRole::Transduction,
            vec!["log10_activity", "sensitivity_scale"],
            Vec::new(),
            "transduction",
            BTreeMap::from([
                ("calibration_json".into(), calibration_json),
                (
                    "activity_transform".into(),
                    format!("{:?}", activity_definition.transform),
                ),
                (
                    "sensitivity_transform".into(),
                    format!("{:?}", state.transform),
                ),
                (
                    "sensitivity_lower".into(),
                    state.lower_bound.unwrap_or(0.5).to_string(),
                ),
                (
                    "sensitivity_upper".into(),
                    state.upper_bound.unwrap_or(1.5).to_string(),
                ),
            ]),
        ));
    }
    let mut inputs = vec![
        input("temperature", "K"),
        input("conductivity", "S/m"),
        input("ionic_strength", "mol/L"),
        input("flow", "m/s"),
        input("polarization_input_v", "V"),
    ];
    for coefficient in &calibration.selectivity_coefficients {
        inputs.push(input(
            &format!("interferent.{}", coefficient.interferent),
            "activity",
        ));
    }
    Ok(ModelDefinition {
        schema_version: crate::model::MODEL_DEFINITION_SCHEMA_VERSION,
        model_id: format!("legacy-estimation-{:?}", config.state_model.kind),
        description: "Compiled compatibility representation of the legacy estimation equations"
            .into(),
        validity_domain: "Stored calibration domain and configured legacy estimator bounds".into(),
        uncertainty_incomplete: true,
        states,
        parameters,
        inputs,
        components,
    })
}

pub fn model_input(environment: &AlignedEnvironment) -> ModelInput {
    let mut values = BTreeMap::new();
    let mut insert = |id: &str, value: Option<f64>, unit: &str| {
        if let Some(value) = value {
            values.insert(
                id.into(),
                InputValue {
                    value,
                    unit: unit.into(),
                },
            );
        }
    };
    insert("temperature", environment.temperature_k, "K");
    insert("conductivity", environment.conductivity_s_per_m, "S/m");
    insert("ionic_strength", environment.ionic_strength_mol_l, "mol/L");
    insert("flow", environment.flow, "m/s");
    insert(
        "polarization_input_v",
        environment.polarization_input_v,
        "V",
    );
    for (ion, activity) in &environment.interferent_activities {
        values.insert(
            format!("interferent.{ion}"),
            InputValue {
                value: *activity,
                unit: "activity".into(),
            },
        );
    }
    ModelInput {
        time_s: environment.timestamp_s,
        values,
    }
}

fn state_initial_uncertainty(
    config: &ResolvedEstimationConfig,
    state_id: &str,
) -> (StateInitializationSource, UncertaintySpec) {
    let (variance, unit, process_variance) = match state_id {
        "log10_activity" => (
            config.initial_covariance.log10_activity_variance,
            "dimensionless^2",
            config.process_noise.activity_variance_per_s,
        ),
        "baseline_offset" => (
            config.initial_covariance.baseline_variance_v2,
            "V^2",
            config.process_noise.baseline_variance_v2_per_s,
        ),
        "polarization" => (
            config.initial_covariance.polarization_variance_v2,
            "V^2",
            config.process_noise.polarization_variance_v2_per_s,
        ),
        "sensitivity_scale" => (
            config.initial_covariance.condition_variance,
            "dimensionless^2",
            config.process_noise.condition_variance_per_s,
        ),
        _ => (f64::NAN, "dimensionless^2", f64::NAN),
    };
    if variance == 0.0 && process_variance == 0.0 {
        (
            StateInitializationSource::DeclaredDefault,
            UncertaintySpec::Deterministic,
        )
    } else {
        (
            StateInitializationSource::Estimated,
            UncertaintySpec::Variance {
                value: variance,
                unit: unit.into(),
            },
        )
    }
}

fn factory(descriptor: &ComponentDescriptor) -> Result<Box<dyn IsmComponent>, ModelError> {
    let calibration = descriptor
        .metadata
        .get("calibration_json")
        .map(|text| {
            serde_json::from_str::<StoredCalibrationModel>(text)
                .map_err(|error| evaluation(descriptor, error))
                .and_then(|model| {
                    StoredCalibrationObservationModel::new(model)
                        .map_err(|error| evaluation(descriptor, error))
                })
        })
        .transpose()?;
    Ok(Box::new(LegacyComponent {
        descriptor: descriptor.clone(),
        bindings: ComponentBindings::default(),
        calibration,
    }))
}

struct LegacyComponent {
    descriptor: ComponentDescriptor,
    bindings: ComponentBindings,
    calibration: Option<StoredCalibrationObservationModel>,
}

impl LegacyComponent {
    fn state(&self, state: &ModelState, id: &str) -> Result<f64, ModelError> {
        self.bindings
            .state_indices
            .get(id)
            .and_then(|index| state.values.get(*index))
            .copied()
            .ok_or_else(|| ModelError::MissingReference {
                component: self.descriptor.id.clone(),
                kind: "state",
                id: id.into(),
            })
    }
    fn parameter(&self, parameters: &ParameterValues, id: &str) -> Result<f64, ModelError> {
        self.bindings
            .parameter_indices
            .get(id)
            .and_then(|index| parameters.values.get(*index))
            .copied()
            .ok_or_else(|| ModelError::MissingReference {
                component: self.descriptor.id.clone(),
                kind: "parameter",
                id: id.into(),
            })
    }
    fn environment(&self, input: &ModelInput) -> AlignedEnvironment {
        let get = |id: &str| input.values.get(id).map(|value| value.value);
        AlignedEnvironment {
            timestamp_s: input.time_s,
            temperature_k: get("temperature"),
            conductivity_s_per_m: get("conductivity"),
            ionic_strength_mol_l: get("ionic_strength"),
            flow: get("flow"),
            polarization_input_v: get("polarization_input_v"),
            interferent_activities: input
                .values
                .iter()
                .filter_map(|(id, value)| {
                    id.strip_prefix("interferent.")
                        .map(|ion| (ion.into(), value.value))
                })
                .collect(),
            ..Default::default()
        }
    }
    fn log_activity(&self, state: &ModelState) -> Result<f64, ModelError> {
        let latent = self.state(state, "log10_activity")?;
        Ok(
            if self
                .descriptor
                .metadata
                .get("activity_transform")
                .is_some_and(|value| value == "LogPositive")
            {
                latent / std::f64::consts::LN_10
            } else {
                latent
            },
        )
    }
    fn sensitivity(&self, state: &ModelState) -> Result<f64, ModelError> {
        let latent = self.state(state, "sensitivity_scale")?;
        if self
            .descriptor
            .metadata
            .get("sensitivity_transform")
            .is_some_and(|value| value == "LogisticBounded")
        {
            let lower = parse_metadata(&self.descriptor, "sensitivity_lower")?;
            let upper = parse_metadata(&self.descriptor, "sensitivity_upper")?;
            Ok(lower + (upper - lower) / (1.0 + (-latent).exp()))
        } else {
            Ok(latent)
        }
    }
}

impl IsmComponent for LegacyComponent {
    fn descriptor(&self) -> &ComponentDescriptor {
        &self.descriptor
    }
    fn bind(&mut self, bindings: &ComponentBindings) -> Result<(), ModelError> {
        self.bindings = bindings.clone();
        Ok(())
    }
    fn process_transition(
        &self,
        state: &mut ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        dt_s: f64,
    ) -> Result<(), ModelError> {
        if self.descriptor.kind != "estimation.legacy_polarization" {
            return Ok(());
        }
        let index = *self
            .bindings
            .state_indices
            .get("polarization")
            .ok_or_else(|| evaluation(&self.descriptor, "missing polarization state"))?;
        let tau = self.parameter(parameters, "legacy_polarization_tau_s")?;
        let gain = self.parameter(parameters, "legacy_polarization_gain")?;
        let drive = input
            .values
            .get("polarization_input_v")
            .map_or(0.0, |value| value.value);
        state.values[index] = (-dt_s / tau).exp() * state.values[index] + gain * drive;
        Ok(())
    }
    fn process_jacobian(
        &self,
        dimension: usize,
        _state: &ModelState,
        parameters: &ParameterValues,
        _input: &ModelInput,
        dt_s: f64,
    ) -> Result<Jacobian, ModelError> {
        let mut jacobian = (0..dimension)
            .map(|row| {
                (0..dimension)
                    .map(|column| if row == column { 1.0 } else { 0.0 })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        if self.descriptor.kind == "estimation.legacy_polarization" {
            let index = self.bindings.state_indices["polarization"];
            jacobian[index][index] =
                (-dt_s / self.parameter(parameters, "legacy_polarization_tau_s")?).exp();
        }
        Ok(jacobian)
    }
    fn observation_voltage(
        &self,
        state: &ModelState,
        _parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<Option<f64>, ModelError> {
        let value = match self.descriptor.kind.as_str() {
            "estimation.legacy_equilibrium" => self
                .calibration
                .as_ref()
                .ok_or_else(|| evaluation(&self.descriptor, "missing calibration"))?
                .predict_potential(self.log_activity(state)?, &self.environment(input))
                .map_err(|error| evaluation(&self.descriptor, error))?,
            "estimation.legacy_baseline" => self.state(state, "baseline_offset")?,
            "estimation.legacy_polarization" => self.state(state, "polarization")?,
            "estimation.legacy_sensitivity" => {
                let calibration = self
                    .calibration
                    .as_ref()
                    .ok_or_else(|| evaluation(&self.descriptor, "missing calibration"))?;
                let environment = self.environment(input);
                let activity = self.log_activity(state)?;
                (self.sensitivity(state)? - 1.0)
                    * (calibration
                        .predict_potential(activity, &environment)
                        .map_err(|error| evaluation(&self.descriptor, error))?
                        - calibration
                            .predict_potential(0.0, &environment)
                            .map_err(|error| evaluation(&self.descriptor, error))?)
            }
            _ => return Err(evaluation(&self.descriptor, "unknown legacy component")),
        };
        Ok(Some(value))
    }
    fn observation_state_jacobian(
        &self,
        state: &ModelState,
        _parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<StateJacobian, ModelError> {
        let dimension = self.bindings.state_indices.len();
        let mut result = vec![0.0; dimension];
        match self.descriptor.kind.as_str() {
            "estimation.legacy_equilibrium" => {
                let index = self.bindings.state_indices["log10_activity"];
                result[index] = self
                    .calibration
                    .as_ref()
                    .ok_or_else(|| evaluation(&self.descriptor, "missing calibration"))?
                    .jacobian_log10_activity(self.log_activity(state)?, &self.environment(input))
                    .map_err(|error| evaluation(&self.descriptor, error))?;
            }
            "estimation.legacy_baseline" => {
                result[self.bindings.state_indices["baseline_offset"]] = 1.0
            }
            "estimation.legacy_polarization" => {
                result[self.bindings.state_indices["polarization"]] = 1.0
            }
            "estimation.legacy_sensitivity" => {
                let calibration = self
                    .calibration
                    .as_ref()
                    .ok_or_else(|| evaluation(&self.descriptor, "missing calibration"))?;
                let environment = self.environment(input);
                let activity = self.log_activity(state)?;
                let signal = calibration
                    .predict_potential(activity, &environment)
                    .map_err(|error| evaluation(&self.descriptor, error))?
                    - calibration
                        .predict_potential(0.0, &environment)
                        .map_err(|error| evaluation(&self.descriptor, error))?;
                result[self.bindings.state_indices["log10_activity"]] =
                    (self.sensitivity(state)? - 1.0)
                        * calibration
                            .jacobian_log10_activity(activity, &environment)
                            .map_err(|error| evaluation(&self.descriptor, error))?;
                result[self.bindings.state_indices["sensitivity_scale"]] = signal;
            }
            _ => {}
        }
        Ok(StateJacobian::analytic(
            self.descriptor
                .observation_state_ids
                .iter()
                .map(|id| {
                    let index = self.bindings.state_indices[id];
                    (id.clone(), result[index])
                })
                .collect::<Vec<_>>(),
        ))
    }
}

fn descriptor(
    id: &str,
    kind: &str,
    role: ComponentRole,
    states: Vec<&str>,
    parameters: Vec<&str>,
    owner: &str,
    metadata: BTreeMap<String, String>,
) -> ComponentDescriptor {
    let state_ids = states.into_iter().map(str::to_string).collect::<Vec<_>>();
    ComponentDescriptor { id: id.into(), kind: kind.into(), role, interpretation_status: InterpretationStatus::Phenomenological, depends_on: Vec::new(), required_inputs: Vec::new(), observation_state_ids: state_ids.clone(), observation_parameter_ids: Vec::new(), numerical_jacobian_supported: false, state_ids, parameter_ids: parameters.into_iter().map(str::to_string).collect(), output_unit: Some("V".into()), voltage_contribution_owner: Some(owner.into()), contribution_semantics: crate::model::ContributionSemantics::AdditivePotential, legacy_composition_rule: None, source: "legacy estimation compatibility adapter".into(), validity_domain: "stored calibration and configured legacy estimator domain".into(), equation: "legacy Phase 6 estimator equation adapter".into(), equation_version: 1, assumptions: vec!["Compatibility adapter preserves legacy numerical semantics without assigning a physical mechanism.".into()], evidence_requirements: vec![EvidenceRequirement { hypothesis_id: format!("{id}.identity"), proposed_mechanism_label: "unassigned".into(), independent_evidence_types: vec!["independent experiment".into()], minimum_independent_observations: 2, validity_domain: "declared calibration domain".into(), alternatives_to_consider: vec!["other reduced-order explanations".into()], required_uncertainty_statement: "state and calibration uncertainty must be retained".into() }], metadata }
}
fn parameter(
    id: &str,
    unit: &str,
    lower: f64,
    upper: f64,
    default_value: f64,
    uncertainty: Option<UncertaintySpec>,
) -> ParameterSpec {
    ParameterSpec {
        id: id.into(),
        name: id.replace('_', " "),
        description: "Legacy estimator compatibility parameter.".into(),
        unit: unit.into(),
        lower_bound: lower,
        upper_bound: upper,
        default_value,
        uncertainty: uncertainty.unwrap_or_else(|| UncertaintySpec::Unknown {
            reason: "legacy estimator compatibility parameter has no configured covariance".into(),
        }),
        source: "legacy estimation configuration".into(),
        equation_version: 1,
        identifiability_requirements: vec![
            "Retain legacy estimator observability and covariance evidence.".into(),
        ],
        value_source: ParameterValueSource::ExternallySupplied,
        characteristic: crate::model::ParameterCharacteristic::Continuous,
        validity_domain: "configured legacy estimator domain".into(),
    }
}
fn input(id: &str, unit: &str) -> InputSpec {
    InputSpec {
        id: id.into(),
        unit: unit.into(),
        required: false,
        source: "aligned estimation environment".into(),
        validity_domain: "finite aligned input when available".into(),
    }
}
fn evaluation(descriptor: &ComponentDescriptor, error: impl std::fmt::Display) -> ModelError {
    ModelError::ComponentEvaluation {
        component: descriptor.id.clone(),
        message: error.to_string(),
    }
}
fn parse_metadata(descriptor: &ComponentDescriptor, id: &str) -> Result<f64, ModelError> {
    descriptor
        .metadata
        .get(id)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| evaluation(descriptor, format!("invalid metadata '{id}'")))
}
