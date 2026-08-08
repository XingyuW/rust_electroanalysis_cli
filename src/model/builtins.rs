//! Static, reduced-order ISM component adapters.
//!
//! These components deliberately delegate equilibrium/activity and relaxation
//! evaluation to the established potentiometry modules. Their names describe
//! observed phenomenology or candidate structures, not confirmed mechanisms.

use super::{
    ComponentBindings, ComponentDescriptor, ComponentFactory, IsmComponent, Jacobian,
    JacobianMethod, JacobianStatus, ModelError, ModelInput, ModelState, ParameterJacobian,
    ParameterValues, StateJacobian,
};
use crate::potentiometry::{
    calibration::{
        activity::evaluate_model_activity,
        nernst::{FARADAY_C_PER_MOL, GAS_CONSTANT_J_PER_MOL_K, evaluate_nernst_auto},
        nicolsky_eisenman::{InterferentModelInput, effective_activity, evaluate_potential},
    },
    transient::models::{TransientModelKind, evaluate as evaluate_transient},
    units::{Quantity, QuantityUnit},
};
use std::str::FromStr;

pub(crate) fn static_factories() -> Vec<(&'static str, ComponentFactory)> {
    [
        "equilibrium.nernst",
        "equilibrium.nicolsky_eisenman",
        "activity.ideal",
        "activity.davies",
        "activity.extended_debye_huckel",
        "activity.input",
        "dynamics.first_order",
        "transduction.first_order_candidate",
        "reference.offset",
        "disturbance.linear_covariate",
        "disturbance.observation_variance",
        "transport.first_order_relaxation",
        "transport.two_mode_relaxation",
        "transport.stretched_relaxation",
        "transport.partition_delay",
        "transduction.ideal",
        "transduction.solid_contact_rc_candidate",
        "transduction.interfacial_polarization_candidate",
        "disturbance.baseline_random_walk",
        "disturbance.linear_drift",
        "disturbance.temperature_covariate",
        "disturbance.conductivity_covariate",
        "disturbance.flow_covariate",
        "disturbance.stochastic_observation_noise",
    ]
    .into_iter()
    .map(|kind| (kind, create_builtin as ComponentFactory))
    .collect()
}

fn create_builtin(descriptor: &ComponentDescriptor) -> Result<Box<dyn IsmComponent>, ModelError> {
    validate_builtin_shape(descriptor)?;
    Ok(Box::new(BuiltinComponent {
        descriptor: descriptor.clone(),
        bindings: ComponentBindings::default(),
    }))
}

struct BuiltinComponent {
    descriptor: ComponentDescriptor,
    bindings: ComponentBindings,
}

impl BuiltinComponent {
    fn state_index(&self, id: &str) -> Result<usize, ModelError> {
        self.bindings
            .state_indices
            .get(id)
            .copied()
            .ok_or_else(|| missing(&self.descriptor.id, "state", id))
    }

    fn parameter(&self, values: &ParameterValues, id: &str) -> Result<f64, ModelError> {
        let index = self.parameter_index(id)?;
        values
            .values
            .get(index)
            .copied()
            .ok_or_else(|| missing(&self.descriptor.id, "parameter index", id))
    }

    fn parameter_index(&self, id: &str) -> Result<usize, ModelError> {
        self.bindings
            .parameter_indices
            .get(id)
            .copied()
            .ok_or_else(|| missing(&self.descriptor.id, "parameter", id))
    }

    fn input(&self, input: &ModelInput, id: &str) -> Result<f64, ModelError> {
        input
            .values
            .get(id)
            .map(|value| value.value)
            .ok_or_else(|| ModelError::MissingInput {
                component: self.descriptor.id.clone(),
                input: id.into(),
            })
    }

    fn activity(
        &self,
        input: &ModelInput,
        parameters: &ParameterValues,
    ) -> Result<(f64, Vec<String>), ModelError> {
        // V1 components consume canonical, already-calculated activity.  The
        // concentration adapter below is retained only for schema-v1 models.
        let activity_input_id = self
            .descriptor
            .metadata
            .get("activity_input_id")
            .map(String::as_str)
            .unwrap_or("target_activity");
        if let Some(activity) = input.values.get(activity_input_id) {
            if !activity.value.is_finite() || activity.value <= 0.0 {
                return Err(evaluation(
                    &self.descriptor.id,
                    "target activity must be finite and positive",
                ));
            }
            return Ok((activity.value, Vec::new()));
        }
        let concentration =
            input
                .values
                .get("primary_concentration")
                .ok_or_else(|| ModelError::MissingInput {
                    component: self.descriptor.id.clone(),
                    input: "primary_concentration".into(),
                })?;
        let unit = QuantityUnit::from_str(&concentration.unit)
            .map_err(|error| evaluation(&self.descriptor.id, error))?;
        let quantity = Quantity::new(concentration.value, unit)
            .map_err(|error| evaluation(&self.descriptor.id, error))?;
        let model = self
            .descriptor
            .metadata
            .get("activity_model")
            .map(String::as_str)
            .unwrap_or("ideal");
        let ionic_strength = input.values.get("ionic_strength").map(|value| value.value);
        let charge = self.parameter(parameters, &self.descriptor.parameter_ids[1])?;
        let charge = if charge.is_finite() {
            charge.round() as i32
        } else {
            1
        };
        let result = evaluate_model_activity(
            &quantity,
            charge,
            ionic_strength,
            model,
            self.descriptor
                .metadata
                .get("ion_size_angstrom")
                .map(|value| value.parse())
                .transpose()
                .map_err(|_| evaluation(&self.descriptor.id, "invalid ion_size_angstrom"))?,
        )
        .map_err(|error| evaluation(&self.descriptor.id, error))?;
        Ok((
            result.activity,
            result
                .warnings
                .into_iter()
                .map(|warning| warning.message)
                .collect(),
        ))
    }

    fn decay(&self, tau_s: f64, dt_s: f64, beta: Option<f64>) -> Result<f64, ModelError> {
        let (kind, parameters) = match beta {
            Some(beta) => (TransientModelKind::Stretched, vec![0.0, 1.0, tau_s, beta]),
            None => (TransientModelKind::Single, vec![0.0, 1.0, tau_s]),
        };
        evaluate_transient(kind, &parameters, dt_s)
            .map(|result| result.fast.unwrap_or(0.0))
            .map_err(|error| evaluation(&self.descriptor.id, error))
    }

    #[allow(clippy::too_many_arguments)]
    fn process_mode(
        &self,
        state: &mut ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        state_id: &str,
        tau_id: &str,
        gain_id: &str,
        beta: Option<f64>,
        dt_s: f64,
    ) -> Result<(), ModelError> {
        let index = self.state_index(state_id)?;
        let decay = self.decay(self.parameter(parameters, tau_id)?, dt_s, beta)?;
        let target = self.parameter(parameters, gain_id)? * self.input(input, "driving_step_v")?;
        state.values[index] = decay * state.values[index] + (1.0 - decay) * target;
        Ok(())
    }

    fn process_v1_first_order(
        &self,
        state: &mut ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        dt_s: f64,
    ) -> Result<(), ModelError> {
        let index = self.state_index(&self.descriptor.state_ids[0])?;
        let tau = self.parameter(parameters, &self.descriptor.parameter_ids[0])?;
        let gain = self.parameter(parameters, &self.descriptor.parameter_ids[1])?;
        state.values[index] *= self.decay(tau, dt_s, None)?;
        // The event is intentionally explicit.  Absence means no event, not
        // an inferred step from the measured potential.
        if let Some(event) = input.values.get("delta_log10_activity") {
            if !event.value.is_finite() {
                return Err(evaluation(&self.descriptor.id, "event step must be finite"));
            }
            state.values[index] += gain * event.value;
        } else if self.descriptor.kind == "transduction.first_order_candidate"
            && let Some(drive) = input.values.get("transduction_drive")
        {
            if !drive.value.is_finite() {
                return Err(evaluation(
                    &self.descriptor.id,
                    "transduction drive must be finite",
                ));
            }
            state.values[index] += gain * drive.value;
        }
        Ok(())
    }

    fn v1_first_order_derivative(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        dimension: usize,
    ) -> Result<Vec<f64>, ModelError> {
        let index = self.state_index(&self.descriptor.state_ids[0])?;
        let tau = self.parameter(parameters, &self.descriptor.parameter_ids[0])?;
        if !tau.is_finite() || tau <= 0.0 {
            return Err(evaluation(
                &self.descriptor.id,
                "time constant must be positive",
            ));
        }
        let mut derivative = vec![0.0; dimension];
        derivative[index] = -state.values[index] / tau;
        Ok(derivative)
    }
}

impl IsmComponent for BuiltinComponent {
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
        match self.descriptor.kind.as_str() {
            "dynamics.first_order" | "transduction.first_order_candidate" => {
                self.process_v1_first_order(state, parameters, input, dt_s)
            }
            "transport.first_order_relaxation" => self.process_mode(
                state,
                parameters,
                input,
                &self.descriptor.state_ids[0],
                &self.descriptor.parameter_ids[0],
                &self.descriptor.parameter_ids[1],
                None,
                dt_s,
            ),
            "transport.two_mode_relaxation" => {
                self.process_mode(
                    state,
                    parameters,
                    input,
                    &self.descriptor.state_ids[0],
                    &self.descriptor.parameter_ids[0],
                    &self.descriptor.parameter_ids[1],
                    None,
                    dt_s,
                )?;
                self.process_mode(
                    state,
                    parameters,
                    input,
                    &self.descriptor.state_ids[1],
                    &self.descriptor.parameter_ids[2],
                    &self.descriptor.parameter_ids[3],
                    None,
                    dt_s,
                )
            }
            "transport.stretched_relaxation" => self.process_mode(
                state,
                parameters,
                input,
                &self.descriptor.state_ids[0],
                &self.descriptor.parameter_ids[0],
                &self.descriptor.parameter_ids[2],
                Some(self.parameter(parameters, &self.descriptor.parameter_ids[1])?),
                dt_s,
            ),
            "transport.partition_delay" => {
                let index = self.state_index(&self.descriptor.state_ids[0])?;
                if input.time_s >= self.parameter(parameters, &self.descriptor.parameter_ids[0])? {
                    state.values[index] = self.input(input, "driving_step_v")?;
                }
                Ok(())
            }
            "transduction.solid_contact_rc_candidate"
            | "transduction.interfacial_polarization_candidate" => self.process_mode(
                state,
                parameters,
                input,
                &self.descriptor.state_ids[0],
                &self.descriptor.parameter_ids[0],
                &self.descriptor.parameter_ids[1],
                None,
                dt_s,
            ),
            "disturbance.baseline_random_walk" => {
                let index = self.state_index(&self.descriptor.state_ids[0])?;
                state.values[index] += self.input(input, "baseline_increment_v")?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn process_derivative(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        _input: &ModelInput,
        state_dimension: usize,
    ) -> Result<Option<Vec<f64>>, ModelError> {
        match self.descriptor.kind.as_str() {
            "dynamics.first_order" | "transduction.first_order_candidate" => Ok(Some(
                self.v1_first_order_derivative(state, parameters, state_dimension)?,
            )),
            "reference.offset" => Ok(Some(vec![0.0; state_dimension])),
            _ => Ok(None),
        }
    }

    fn process_jacobian(
        &self,
        dimension: usize,
        _state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        dt_s: f64,
    ) -> Result<Jacobian, ModelError> {
        let mut result = super::component::identity(dimension);
        match self.descriptor.kind.as_str() {
            "dynamics.first_order" | "transduction.first_order_candidate" => {
                let index = self.state_index(&self.descriptor.state_ids[0])?;
                result[index][index] = self.decay(
                    self.parameter(parameters, &self.descriptor.parameter_ids[0])?,
                    dt_s,
                    None,
                )?;
            }
            "transport.first_order_relaxation"
            | "transduction.solid_contact_rc_candidate"
            | "transduction.interfacial_polarization_candidate" => {
                let index = self.state_index(&self.descriptor.state_ids[0])?;
                result[index][index] = self.decay(
                    self.parameter(parameters, &self.descriptor.parameter_ids[0])?,
                    dt_s,
                    None,
                )?;
            }
            "transport.two_mode_relaxation" => {
                for (state_id, tau_id) in self.descriptor.state_ids.iter().zip([
                    &self.descriptor.parameter_ids[0],
                    &self.descriptor.parameter_ids[2],
                ]) {
                    let index = self.state_index(state_id)?;
                    result[index][index] =
                        self.decay(self.parameter(parameters, tau_id)?, dt_s, None)?;
                }
            }
            "transport.stretched_relaxation" => {
                let index = self.state_index(&self.descriptor.state_ids[0])?;
                result[index][index] = self.decay(
                    self.parameter(parameters, &self.descriptor.parameter_ids[0])?,
                    dt_s,
                    Some(self.parameter(parameters, &self.descriptor.parameter_ids[1])?),
                )?;
            }
            "transport.partition_delay" => {
                let index = self.state_index(&self.descriptor.state_ids[0])?;
                if input.time_s >= self.parameter(parameters, &self.descriptor.parameter_ids[0])? {
                    result[index][index] = 0.0;
                }
            }
            _ => {}
        }
        Ok(result)
    }

    fn observation_voltage(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<Option<f64>, ModelError> {
        let value = match self.descriptor.kind.as_str() {
            "equilibrium.nernst" => {
                let (activity, _) = self.activity(input, parameters)?;
                if let Some(slope_id) = slope_parameter_id(&self.descriptor) {
                    let value = self.parameter(parameters, &self.descriptor.parameter_ids[0])?
                        + self.parameter(parameters, slope_id)? * activity.log10();
                    if !value.is_finite() {
                        return Err(evaluation(
                            &self.descriptor.id,
                            "empirical-slope Nernst prediction was non-finite",
                        ));
                    }
                    value
                } else {
                    evaluate_nernst_auto(
                        self.parameter(parameters, &self.descriptor.parameter_ids[0])?,
                        activity,
                        self.input(input, "temperature")?,
                        self.parameter(parameters, &self.descriptor.parameter_ids[1])?
                            .round() as i32,
                    )
                    .map_err(|error| evaluation(&self.descriptor.id, error))?
                }
            }
            "equilibrium.nicolsky_eisenman" => {
                let (activity, _) = self.activity(input, parameters)?;
                let interferents =
                    parse_interferents(&self.descriptor, &self.bindings, parameters, input)?;
                if let Some(slope_id) = slope_parameter_id(&self.descriptor) {
                    let charge = self
                        .parameter(parameters, &self.descriptor.parameter_ids[1])?
                        .round() as i32;
                    let effective = effective_activity(activity, charge, &interferents)
                        .map_err(|error| evaluation(&self.descriptor.id, error))?;
                    let value = self.parameter(parameters, &self.descriptor.parameter_ids[0])?
                        + self.parameter(parameters, slope_id)? * effective.log10();
                    if !value.is_finite() {
                        return Err(evaluation(
                            &self.descriptor.id,
                            "empirical-slope Nicolsky-Eisenman prediction was non-finite",
                        ));
                    }
                    value
                } else {
                    evaluate_potential(
                        self.parameter(parameters, &self.descriptor.parameter_ids[0])?,
                        activity,
                        self.parameter(parameters, &self.descriptor.parameter_ids[1])?
                            .round() as i32,
                        self.input(input, "temperature")?,
                        &interferents,
                    )
                    .map_err(|error| evaluation(&self.descriptor.id, error))?
                }
            }
            "transport.first_order_relaxation"
            | "transport.stretched_relaxation"
            | "transport.partition_delay"
            | "transduction.solid_contact_rc_candidate"
            | "transduction.interfacial_polarization_candidate"
            | "disturbance.baseline_random_walk" => {
                state.values[self.state_index(&self.descriptor.state_ids[0])?]
            }
            "dynamics.first_order" | "transduction.first_order_candidate" | "reference.offset" => {
                state.values[self.state_index(&self.descriptor.state_ids[0])?]
            }
            "transport.two_mode_relaxation" => {
                state.values[self.state_index(&self.descriptor.state_ids[0])?]
                    + state.values[self.state_index(&self.descriptor.state_ids[1])?]
            }
            "transduction.ideal" => {
                self.parameter(parameters, &self.descriptor.parameter_ids[0])?
                    * self.input(input, "transduction_drive_v")?
                    + self.parameter(parameters, &self.descriptor.parameter_ids[1])?
            }
            "disturbance.linear_drift" => {
                self.parameter(parameters, &self.descriptor.parameter_ids[0])? * input.time_s
            }
            "disturbance.temperature_covariate"
            | "disturbance.conductivity_covariate"
            | "disturbance.flow_covariate" => {
                self.parameter(parameters, &self.descriptor.parameter_ids[0])?
                    * (self.input(input, &self.descriptor.required_inputs[0].id)?
                        - self.parameter(parameters, &self.descriptor.parameter_ids[1])?)
            }
            "disturbance.linear_covariate" => {
                self.parameter(parameters, &self.descriptor.parameter_ids[0])?
                    * (self.input(input, &self.descriptor.required_inputs[0].id)?
                        - self.parameter(parameters, &self.descriptor.parameter_ids[1])?)
            }
            "disturbance.stochastic_observation_noise"
            | "disturbance.observation_variance"
            | "activity.input"
            | "activity.ideal"
            | "activity.davies"
            | "activity.extended_debye_huckel" => return Ok(None),
            _ => {
                return Err(ModelError::UnknownComponentKind {
                    component: self.descriptor.id.clone(),
                    kind: self.descriptor.kind.clone(),
                });
            }
        };
        Ok(Some(value))
    }

    fn observation_variance_v2(
        &self,
        _state: &ModelState,
        parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<Option<f64>, ModelError> {
        match self.descriptor.kind.as_str() {
            "disturbance.stochastic_observation_noise" => {
                let standard_deviation_v =
                    self.parameter(parameters, &self.descriptor.parameter_ids[0])?;
                Ok(Some(standard_deviation_v * standard_deviation_v))
            }
            "disturbance.observation_variance" => {
                let variance = self.parameter(parameters, &self.descriptor.parameter_ids[0])?;
                if !variance.is_finite() || variance <= 0.0 {
                    return Err(evaluation(
                        &self.descriptor.id,
                        "observation variance must be finite and positive",
                    ));
                }
                Ok(Some(variance))
            }
            _ => Ok(None),
        }
    }

    fn auxiliary_outputs(
        &self,
        _state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<std::collections::BTreeMap<String, f64>, ModelError> {
        let mut result = std::collections::BTreeMap::new();
        match self.descriptor.kind.as_str() {
            "activity.input" => {
                let activity = self.input(input, "target_activity")?;
                if !activity.is_finite() || activity <= 0.0 {
                    return Err(evaluation(
                        &self.descriptor.id,
                        "target activity must be finite and positive",
                    ));
                }
                result.insert("activity".into(), activity);
            }
            "equilibrium.nicolsky_eisenman" => {
                let (activity, _) = self.activity(input, parameters)?;
                let interferents =
                    parse_interferents(&self.descriptor, &self.bindings, parameters, input)?;
                let charge = self
                    .parameter(parameters, &self.descriptor.parameter_ids[1])?
                    .round() as i32;
                result.insert(
                    "effective_activity".into(),
                    effective_activity(activity, charge, &interferents)
                        .map_err(|error| evaluation(&self.descriptor.id, error))?,
                );
            }
            _ => {}
        }
        Ok(result)
    }

    fn observation_state_jacobian(
        &self,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<StateJacobian, ModelError> {
        let values = match self.descriptor.kind.as_str() {
            "transport.first_order_relaxation"
            | "transport.stretched_relaxation"
            | "transport.partition_delay"
            | "transduction.solid_contact_rc_candidate"
            | "transduction.interfacial_polarization_candidate"
            | "disturbance.baseline_random_walk" => {
                vec![(self.descriptor.state_ids[0].clone(), 1.0)]
            }
            "dynamics.first_order" | "transduction.first_order_candidate" | "reference.offset" => {
                vec![(self.descriptor.state_ids[0].clone(), 1.0)]
            }
            "transport.two_mode_relaxation" => self
                .descriptor
                .state_ids
                .iter()
                .cloned()
                .map(|id| (id, 1.0))
                .collect(),
            _ => return Ok(StateJacobian::not_applicable()),
        };
        Ok(StateJacobian::analytic(values))
    }

    fn observation_parameter_jacobian(
        &self,
        _state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<ParameterJacobian, ModelError> {
        let ids = &self.descriptor.parameter_ids;
        match self.descriptor.kind.as_str() {
            "equilibrium.nernst" => {
                let mut covered_parameters = vec![ids[0].clone()];
                let mut values = vec![1.0];
                if let Some(slope_id) = slope_parameter_id(&self.descriptor) {
                    let (activity, _) = self.activity(input, parameters)?;
                    covered_parameters.push(slope_id.into());
                    values.push(activity.log10());
                }
                if self.descriptor.metadata.contains_key("activity_input_id") {
                    return Ok(ParameterJacobian {
                        values,
                        covered_parameters,
                        status: JacobianStatus::Complete,
                        method: JacobianMethod::Analytic,
                    });
                }
                Ok(ParameterJacobian {
                    values,
                    covered_parameters,
                    status: JacobianStatus::Partial {
                        missing_parameters: vec![ids[1].clone()],
                    },
                    method: JacobianMethod::Analytic,
                })
            }
            "equilibrium.nicolsky_eisenman" => {
                let (primary_activity, _) = self.activity(input, parameters)?;
                let primary_charge = self.parameter(parameters, &ids[1])?.round() as i32;
                let interferents = parse_interferents_with_ids(
                    &self.descriptor,
                    &self.bindings,
                    parameters,
                    input,
                )?;
                let models = interferents
                    .iter()
                    .map(|(_, model)| model.clone())
                    .collect::<Vec<_>>();
                let effective = effective_activity(primary_activity, primary_charge, &models)
                    .map_err(|error| evaluation(&self.descriptor.id, error))?;
                let scale = if let Some(slope_id) = slope_parameter_id(&self.descriptor) {
                    self.parameter(parameters, slope_id)? / (std::f64::consts::LN_10 * effective)
                } else {
                    GAS_CONSTANT_J_PER_MOL_K * self.input(input, "temperature")?
                        / (f64::from(primary_charge) * FARADAY_C_PER_MOL * effective)
                };
                let mut covered_parameters = vec![ids[0].clone()];
                let mut values = vec![1.0];
                if let Some(slope_id) = slope_parameter_id(&self.descriptor) {
                    covered_parameters.push(slope_id.into());
                    values.push(effective.log10());
                }
                for (id, interferent) in interferents {
                    covered_parameters.push(id);
                    values.push(
                        scale
                            * interferent
                                .activity
                                .powf(f64::from(primary_charge) / f64::from(interferent.charge)),
                    );
                }
                if self.descriptor.metadata.contains_key("activity_input_id") {
                    return Ok(ParameterJacobian {
                        values,
                        covered_parameters,
                        status: JacobianStatus::Complete,
                        method: JacobianMethod::Analytic,
                    });
                }
                Ok(ParameterJacobian {
                    values,
                    covered_parameters,
                    status: JacobianStatus::Partial {
                        missing_parameters: vec![ids[1].clone()],
                    },
                    method: JacobianMethod::Analytic,
                })
            }
            "transduction.ideal" => Ok(ParameterJacobian::analytic([
                (ids[0].clone(), self.input(input, "transduction_drive_v")?),
                (ids[1].clone(), 1.0),
            ])),
            "disturbance.linear_drift" => Ok(ParameterJacobian::analytic([(
                ids[0].clone(),
                input.time_s,
            )])),
            "disturbance.temperature_covariate"
            | "disturbance.conductivity_covariate"
            | "disturbance.flow_covariate" => {
                let covariate = self.input(input, &self.descriptor.required_inputs[0].id)?;
                let sensitivity = self.parameter(parameters, &ids[0])?;
                let reference = self.parameter(parameters, &ids[1])?;
                Ok(ParameterJacobian::analytic([
                    (ids[0].clone(), covariate - reference),
                    (ids[1].clone(), -sensitivity),
                ]))
            }
            "disturbance.linear_covariate" => {
                let covariate = self.input(input, &self.descriptor.required_inputs[0].id)?;
                let sensitivity = self.parameter(parameters, &ids[0])?;
                let reference = self.parameter(parameters, &ids[1])?;
                Ok(ParameterJacobian::analytic([
                    (ids[0].clone(), covariate - reference),
                    (ids[1].clone(), -sensitivity),
                ]))
            }
            _ => Ok(ParameterJacobian::not_applicable()),
        }
    }

    fn validity_warnings(
        &self,
        _state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<Vec<String>, ModelError> {
        let mut warnings = match self.descriptor.kind.as_str() {
            "equilibrium.nernst"
            | "equilibrium.nicolsky_eisenman"
            | "activity.ideal"
            | "activity.davies"
            | "activity.extended_debye_huckel" => self.activity(input, parameters)?.1,
            "transport.two_mode_relaxation" => {
                let fast = self.parameter(parameters, &self.descriptor.parameter_ids[0])?;
                let slow = self.parameter(parameters, &self.descriptor.parameter_ids[2])?;
                if slow / fast < 2.0 {
                    vec!["relaxation modes are poorly separated; individual mode values may not be identifiable".into()]
                } else {
                    Vec::new()
                }
            }
            "dynamics.first_order" => {
                if let Some(peer_tau) = self
                    .descriptor
                    .metadata
                    .get("peer_tau_s")
                    .and_then(|value| value.parse::<f64>().ok())
                {
                    let tau = self.parameter(parameters, &self.descriptor.parameter_ids[0])?;
                    let threshold = self
                        .descriptor
                        .metadata
                        .get("separation_threshold")
                        .and_then(|value| value.parse::<f64>().ok())
                        .unwrap_or(2.0);
                    if tau >= peer_tau || peer_tau / tau < threshold {
                        vec![
                            "dynamic time constants are poorly separated; modes may be confounded"
                                .into(),
                        ]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };
        if let Some(limit) = self
            .descriptor
            .metadata
            .get("maximum_time_s")
            .and_then(|value| value.parse::<f64>().ok())
            && input.time_s > limit
        {
            warnings.push(format!(
                "input time {:.3} s exceeds component validity limit {:.3} s",
                input.time_s, limit
            ));
        }
        Ok(warnings)
    }
}

fn validate_builtin_shape(descriptor: &ComponentDescriptor) -> Result<(), ModelError> {
    let (states, parameters) = match descriptor.kind.as_str() {
        "equilibrium.nernst" | "equilibrium.nicolsky_eisenman" => (0, 2),
        "activity.ideal" | "activity.davies" | "activity.extended_debye_huckel" => (0, 2),
        "activity.input" => (0, 0),
        "dynamics.first_order" | "transduction.first_order_candidate" => (1, 2),
        "reference.offset" => (1, 0),
        "disturbance.linear_covariate" => (0, 2),
        "disturbance.observation_variance" => (0, 1),
        "transport.first_order_relaxation"
        | "transduction.solid_contact_rc_candidate"
        | "transduction.interfacial_polarization_candidate" => (1, 2),
        "transport.two_mode_relaxation" => (2, 4),
        "transport.stretched_relaxation" => (1, 3),
        "transport.partition_delay" => (1, 1),
        "transduction.ideal" => (0, 2),
        "disturbance.baseline_random_walk" => (1, 0),
        "disturbance.linear_drift" => (0, 1),
        "disturbance.temperature_covariate"
        | "disturbance.conductivity_covariate"
        | "disturbance.flow_covariate" => (0, 2),
        "disturbance.stochastic_observation_noise" => (0, 1),
        _ => {
            return Err(ModelError::UnknownComponentKind {
                component: descriptor.id.clone(),
                kind: descriptor.kind.clone(),
            });
        }
    };
    if descriptor.state_ids.len() != states || descriptor.parameter_ids.len() < parameters {
        return Err(ModelError::InvalidComponentShape {
            component: descriptor.id.clone(),
            message: format!(
                "kind '{}' requires {states} state IDs and at least {parameters} parameter IDs; found {} and {}",
                descriptor.kind,
                descriptor.state_ids.len(),
                descriptor.parameter_ids.len()
            ),
        });
    }
    if descriptor.kind == "equilibrium.nernst" {
        let expected = if slope_parameter_id(descriptor).is_some() {
            3
        } else {
            2
        };
        if descriptor.parameter_ids.len() != expected {
            return Err(ModelError::InvalidComponentShape {
                component: descriptor.id.clone(),
                message: "Nernst parameters must be E0 and charge, plus only an explicitly named slope parameter".into(),
            });
        }
    }
    if let Some(slope_id) = slope_parameter_id(descriptor)
        && (!descriptor.parameter_ids.iter().any(|id| id == slope_id)
            || descriptor.parameter_ids[0] == slope_id
            || descriptor.parameter_ids[1] == slope_id)
    {
        return Err(ModelError::InvalidComponentShape {
            component: descriptor.id.clone(),
            message: "slope_parameter_id must name a distinct declared parameter".into(),
        });
    }
    let expected_observation_states = match descriptor.kind.as_str() {
        "transport.first_order_relaxation"
        | "transport.two_mode_relaxation"
        | "transport.stretched_relaxation"
        | "transport.partition_delay"
        | "transduction.solid_contact_rc_candidate"
        | "transduction.interfacial_polarization_candidate"
        | "disturbance.baseline_random_walk" => descriptor.state_ids.clone(),
        "dynamics.first_order" | "transduction.first_order_candidate" | "reference.offset" => {
            descriptor.state_ids.clone()
        }
        _ => Vec::new(),
    };
    let expected_observation_parameters = match descriptor.kind.as_str() {
        "equilibrium.nernst" if descriptor.metadata.contains_key("activity_input_id") => {
            let mut ids = vec![descriptor.parameter_ids[0].clone()];
            if let Some(slope_id) = slope_parameter_id(descriptor) {
                ids.push(slope_id.into());
            }
            ids
        }
        "equilibrium.nicolsky_eisenman"
            if descriptor.metadata.contains_key("activity_input_id") =>
        {
            let mut ids = vec![descriptor.parameter_ids[0].clone()];
            if let Some(slope_id) = slope_parameter_id(descriptor) {
                ids.push(slope_id.into());
            }
            let interferent_ids = parse_interferent_parameter_ids(descriptor)
                .into_iter()
                .filter(|id| !ids.contains(id))
                .collect::<Vec<_>>();
            ids.extend(interferent_ids);
            ids
        }
        "equilibrium.nernst"
        | "equilibrium.nicolsky_eisenman"
        | "transduction.ideal"
        | "disturbance.linear_drift"
        | "disturbance.temperature_covariate"
        | "disturbance.conductivity_covariate"
        | "disturbance.flow_covariate" => descriptor.parameter_ids.clone(),
        "disturbance.linear_covariate" => descriptor.parameter_ids.clone(),
        _ => Vec::new(),
    };
    if descriptor.observation_state_ids != expected_observation_states
        || descriptor.observation_parameter_ids != expected_observation_parameters
    {
        return Err(ModelError::InvalidComponentShape {
            component: descriptor.id.clone(),
            message: format!(
                "kind '{}' has inconsistent direct observation derivative declarations",
                descriptor.kind
            ),
        });
    }
    let required_runtime_input = match descriptor.kind.as_str() {
        "transport.first_order_relaxation"
        | "transport.two_mode_relaxation"
        | "transport.stretched_relaxation"
        | "transport.partition_delay"
        | "transduction.solid_contact_rc_candidate"
        | "transduction.interfacial_polarization_candidate" => Some("driving_step_v"),
        "disturbance.baseline_random_walk" => Some("baseline_increment_v"),
        "transduction.ideal" => Some("transduction_drive_v"),
        // V1 event inputs are optional at ordinary propagation steps. When
        // supplied they are explicit model inputs, never inferred from V.
        "dynamics.first_order" => None,
        "transduction.first_order_candidate" => None,
        "disturbance.temperature_covariate"
        | "disturbance.conductivity_covariate"
        | "disturbance.flow_covariate" => descriptor
            .required_inputs
            .first()
            .map(|input| input.id.as_str()),
        _ => None,
    };
    if let Some(input) = required_runtime_input
        && !descriptor
            .required_inputs
            .iter()
            .any(|item| item.id == input)
    {
        return Err(ModelError::InvalidComponentShape {
            component: descriptor.id.clone(),
            message: format!(
                "kind '{}' must declare required input '{input}'",
                descriptor.kind
            ),
        });
    }
    Ok(())
}
fn slope_parameter_id(descriptor: &ComponentDescriptor) -> Option<&str> {
    descriptor
        .metadata
        .get("slope_parameter_id")
        .map(String::as_str)
}
fn missing(component: &str, kind: &'static str, id: &str) -> ModelError {
    ModelError::MissingReference {
        component: component.into(),
        kind,
        id: id.into(),
    }
}
fn evaluation(component: &str, error: impl std::fmt::Display) -> ModelError {
    ModelError::ComponentEvaluation {
        component: component.into(),
        message: error.to_string(),
    }
}
fn parse_interferents(
    descriptor: &ComponentDescriptor,
    bindings: &ComponentBindings,
    parameters: &ParameterValues,
    input: &ModelInput,
) -> Result<Vec<InterferentModelInput>, ModelError> {
    parse_interferents_with_ids(descriptor, bindings, parameters, input).map(|items| {
        items
            .into_iter()
            .map(|(_, interferent)| interferent)
            .collect()
    })
}

fn parse_interferents_with_ids(
    descriptor: &ComponentDescriptor,
    bindings: &ComponentBindings,
    parameters: &ParameterValues,
    input: &ModelInput,
) -> Result<Vec<(String, InterferentModelInput)>, ModelError> {
    descriptor
        .metadata
        .get("interferents")
        .map(|spec| {
            spec.split(',')
                .filter(|part| !part.trim().is_empty())
                .map(|part| {
                    let fields: Vec<_> = part.split(':').collect();
                    if fields.len() != 4 {
                        return Err(ModelError::ComponentEvaluation {
                            component: descriptor.id.clone(),
                            message: "interferents must be name:charge:input:parameter".into(),
                        });
                    }
                    if !descriptor.parameter_ids.iter().any(|id| id == fields[3]) {
                        return Err(missing(&descriptor.id, "interferent parameter", fields[3]));
                    }
                    let index = bindings
                        .parameter_indices
                        .get(fields[3])
                        .copied()
                        .ok_or_else(|| missing(&descriptor.id, "parameter", fields[3]))?;
                    let charge =
                        fields[1]
                            .parse()
                            .map_err(|_| ModelError::ComponentEvaluation {
                                component: descriptor.id.clone(),
                                message: "interferent charge must be integer".into(),
                            })?;
                    Ok((
                        fields[3].into(),
                        InterferentModelInput {
                            name: fields[0].into(),
                            charge,
                            activity: input
                                .values
                                .get(fields[2])
                                .ok_or_else(|| ModelError::MissingInput {
                                    component: descriptor.id.clone(),
                                    input: fields[2].into(),
                                })?
                                .value,
                            selectivity_coefficient: parameters
                                .values
                                .get(index)
                                .copied()
                                .ok_or_else(|| missing(&descriptor.id, "parameter", fields[3]))?,
                        },
                    ))
                })
                .collect()
        })
        .transpose()
        .map(|value: Option<Vec<_>>| value.unwrap_or_default())
}

fn parse_interferent_parameter_ids(descriptor: &ComponentDescriptor) -> Vec<String> {
    descriptor
        .metadata
        .get("interferents")
        .map(|spec| {
            spec.split(',')
                .filter_map(|item| item.split(':').nth(3))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
