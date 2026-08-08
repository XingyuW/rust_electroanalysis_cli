use crate::{
    estimation::{
        calibration_adapter::CalibrationObservationModel,
        environment::AlignedEnvironment,
        error::EstimationError,
        measurement::{AuxiliaryObservation, AuxiliaryObservationKind},
        state::{StateDefinition, state_definitions},
    },
    estimation_config::{
        CompiledEstimationProfile, EstimationModelBackend, ProcessNoiseConfig,
        ResolvedEstimationConfig, StateModelKind,
    },
    results::FeatureComparability,
};
use nalgebra::{DMatrix, DVector};

#[derive(Clone)]
pub struct StateModel {
    pub kind: StateModelKind,
    pub definitions: Vec<StateDefinition>,
    pub tau_p_s: f64,
    pub tau_uncertainty_s: Option<f64>,
    pub gain: f64,
    pub(crate) compiled: Option<std::sync::Arc<crate::model::CompiledIsmModel>>,
    pub(crate) compiled_parameters: Option<crate::model::ParameterValues>,
    pub(crate) resolved_input_bindings:
        Option<std::sync::Arc<super::ism_adapter::ResolvedModelInputBindings>>,
    pub(crate) definition_source: Option<super::ism_adapter::ResolvedModelDefinitionSource>,
}

impl StateModel {
    pub fn new(
        config: &ResolvedEstimationConfig,
        tau_p_s: f64,
        tau_uncertainty_s: Option<f64>,
    ) -> Result<Self, EstimationError> {
        if !tau_p_s.is_finite() || tau_p_s <= 0.0 {
            return Err(EstimationError::config(
                "polarization time constant must be positive",
            ));
        }
        let definitions = state_definitions(
            config.state_model.kind,
            config.state_model.include_condition_state,
            config.state_model.condition_lower,
            config.state_model.condition_upper,
            config.state_model.activity_transform,
        );
        for definition in &definitions {
            definition
                .transform
                .validate_bounds(definition.lower_bound, definition.upper_bound)
                .map_err(EstimationError::config)?;
        }
        Ok(Self {
            kind: config.state_model.kind,
            definitions,
            tau_p_s,
            tau_uncertainty_s,
            gain: config.polarization.gain,
            compiled: None,
            compiled_parameters: None,
            resolved_input_bindings: None,
            definition_source: None,
        })
    }
    pub fn new_compiled(
        config: &ResolvedEstimationConfig,
        tau_p_s: f64,
        tau_uncertainty_s: Option<f64>,
        calibration: &crate::results::StoredCalibrationModel,
    ) -> Result<Self, EstimationError> {
        if !matches!(config.model.backend, EstimationModelBackend::Compiled) {
            return Self::new(config, tau_p_s, tau_uncertainty_s);
        }
        if matches!(config.model.profile, CompiledEstimationProfile::Custom) {
            return super::ism_adapter::compile_custom_model(config, tau_p_s, tau_uncertainty_s);
        }
        if matches!(
            config.model.profile,
            CompiledEstimationProfile::ReducedIsmV1
        ) {
            return super::ism_adapter::compile_reduced_v1_model(
                config,
                tau_p_s,
                tau_uncertainty_s,
                calibration,
            );
        }
        let mut model = Self::new(config, tau_p_s, tau_uncertainty_s)?;
        let compiled = super::ism_adapter::compile_legacy_model(
            config,
            &model.definitions,
            tau_p_s,
            tau_uncertainty_s,
            calibration,
        )?;
        let resolved_bindings =
            super::ism_adapter::resolve_model_input_bindings(&compiled, config)?;
        model.compiled_parameters = Some(compiled.default_parameters());
        model.compiled = Some(std::sync::Arc::new(compiled));
        model.resolved_input_bindings = Some(std::sync::Arc::new(resolved_bindings));
        model.definition_source = Some(super::ism_adapter::ResolvedModelDefinitionSource::Profile(
            config.model.profile,
        ));
        Ok(model)
    }
    pub fn dimension(&self) -> usize {
        self.definitions.len()
    }
    pub fn has_baseline(&self) -> bool {
        self.definitions
            .iter()
            .any(|x| matches!(x.name.as_str(), "baseline_offset" | "reference_offset_v"))
    }
    pub fn has_polarization(&self) -> bool {
        self.definitions.iter().any(|x| {
            matches!(
                x.name.as_str(),
                "polarization" | "dynamic_fast_potential_v" | "dynamic_slow_potential_v"
            )
        })
    }
    pub fn has_condition(&self) -> bool {
        self.definitions
            .iter()
            .any(|x| x.name == "sensitivity_scale")
    }
    pub fn index(&self, name: &str) -> Option<usize> {
        self.definitions.iter().position(|x| x.name == name)
    }
    pub fn log10_activity(&self, state: &DVector<f64>) -> Result<f64, EstimationError> {
        let index = self.index("log10_activity").unwrap_or(0);
        let definition = &self.definitions[index];
        let value = match definition.transform {
            crate::estimation::state::StateTransform::Identity
            | crate::estimation::state::StateTransform::Log10Positive => state[index],
            crate::estimation::state::StateTransform::LogPositive => {
                state[index] / std::f64::consts::LN_10
            }
            crate::estimation::state::StateTransform::LogisticBounded => {
                return Err(EstimationError::config(
                    "logistic activity transform requires a bounded physical activity state",
                ));
            }
        };
        value.is_finite().then_some(value).ok_or_else(|| {
            EstimationError::Numerical(
                "activity transform returned a nonfinite log10 activity".into(),
            )
        })
    }
    pub fn latent_from_log10_activity(&self, log10: f64) -> Result<f64, EstimationError> {
        let index = self.index("log10_activity").unwrap_or(0);
        let definition = &self.definitions[index];
        let latent = match definition.transform {
            crate::estimation::state::StateTransform::Identity
            | crate::estimation::state::StateTransform::Log10Positive => log10,
            crate::estimation::state::StateTransform::LogPositive => {
                log10 * std::f64::consts::LN_10
            }
            crate::estimation::state::StateTransform::LogisticBounded => {
                return Err(EstimationError::config(
                    "logistic activity transform requires a bounded physical activity state",
                ));
            }
        };
        latent.is_finite().then_some(latent).ok_or_else(|| {
            EstimationError::Numerical(
                "activity inverse transform returned a nonfinite state".into(),
            )
        })
    }
    pub fn physical_state_value(&self, state: &DVector<f64>, index: usize) -> Option<f64> {
        let definition = &self.definitions[index];
        definition.transform.to_physical(
            state[index],
            definition.lower_bound,
            definition.upper_bound,
        )
    }
    pub fn transition_matrix(&self, dt_s: f64) -> DMatrix<f64> {
        let mut f = DMatrix::identity(self.dimension(), self.dimension());
        if let Some(i) = self.index("polarization") {
            f[(i, i)] = (-dt_s / self.tau_p_s).exp();
        }
        f
    }
    pub fn transition_matrix_for(
        &self,
        state: &DVector<f64>,
        dt_s: f64,
        environment: &AlignedEnvironment,
    ) -> Result<DMatrix<f64>, EstimationError> {
        let Some(compiled) = &self.compiled else {
            return Ok(self.transition_matrix(dt_s));
        };
        let parameters = self.compiled_parameters.as_ref().ok_or_else(|| {
            EstimationError::Numerical("compiled model parameters are unavailable".into())
        })?;
        let jacobian = compiled
            .process_jacobian(
                &crate::model::ModelState::new(state.iter().copied().collect()),
                parameters,
                &self.compiled_input(state, environment)?,
                dt_s,
            )
            .map_err(|error| EstimationError::Numerical(error.to_string()))?;
        Ok(DMatrix::from_fn(
            jacobian.len(),
            jacobian.len(),
            |row, column| jacobian[row][column],
        ))
    }
    pub fn process_state(
        &self,
        state: &DVector<f64>,
        dt_s: f64,
        environment: &AlignedEnvironment,
    ) -> DVector<f64> {
        let mut next = state.clone();
        if let Some(i) = self.index("polarization") {
            let input = environment.polarization_input_v.unwrap_or(0.0);
            next[i] = (-dt_s / self.tau_p_s).exp() * state[i] + self.gain * input;
        }
        next
    }
    pub fn try_process_state(
        &self,
        state: &DVector<f64>,
        dt_s: f64,
        environment: &AlignedEnvironment,
    ) -> Result<DVector<f64>, EstimationError> {
        let Some(compiled) = &self.compiled else {
            return Ok(self.process_state(state, dt_s, environment));
        };
        let parameters = self.compiled_parameters.as_ref().ok_or_else(|| {
            EstimationError::Numerical("compiled model parameters are unavailable".into())
        })?;
        let next = compiled
            .process_transition(
                &crate::model::ModelState::new(state.iter().copied().collect()),
                parameters,
                &self.compiled_input(state, environment)?,
                dt_s,
            )
            .map_err(|error| EstimationError::Numerical(error.to_string()))?;
        Ok(DVector::from_vec(next.values))
    }

    pub fn compiled_model(&self) -> Option<&crate::model::CompiledIsmModel> {
        self.compiled.as_deref()
    }
    pub fn compiled_parameter_values(&self) -> Option<&crate::model::ParameterValues> {
        self.compiled_parameters.as_ref()
    }
    pub fn definition_source(&self) -> Option<&super::ism_adapter::ResolvedModelDefinitionSource> {
        self.definition_source.as_ref()
    }
    pub fn resolved_input_bindings(
        &self,
    ) -> Option<&super::ism_adapter::ResolvedModelInputBindings> {
        self.resolved_input_bindings.as_deref()
    }
    pub fn compiled_observation_prediction(
        &self,
        state: &DVector<f64>,
        environment: &AlignedEnvironment,
        observed_voltage_v: Option<f64>,
    ) -> Result<Option<crate::model::ObservationPrediction>, EstimationError> {
        let Some(compiled) = &self.compiled else {
            return Ok(None);
        };
        let parameters = self.compiled_parameters.as_ref().ok_or_else(|| {
            EstimationError::Numerical("compiled model parameters are unavailable".into())
        })?;
        compiled
            .observation_prediction(
                &crate::model::ModelState::new(state.iter().copied().collect()),
                parameters,
                &self.compiled_input(state, environment)?,
                observed_voltage_v,
            )
            .map(Some)
            .map_err(|error| EstimationError::Numerical(error.to_string()))
    }
    pub fn model_observation_variance_v2(
        &self,
        state: &DVector<f64>,
        environment: &AlignedEnvironment,
    ) -> Result<Option<f64>, EstimationError> {
        self.compiled_observation_prediction(state, environment, None)
            .map(|prediction| {
                prediction.map(|value| value.categorized_totals().observation_variance_v2)
            })
    }

    pub fn compiled_input(
        &self,
        state: &DVector<f64>,
        environment: &AlignedEnvironment,
    ) -> Result<crate::model::ModelInput, EstimationError> {
        if let Some(bindings) = &self.resolved_input_bindings {
            return super::ism_adapter::execute_model_input_bindings(
                bindings,
                Some(self.log10_activity(state)?),
                environment,
            );
        }
        let mut input = super::ism_adapter::model_input(environment);
        if let Ok(log10_activity) = self.log10_activity(state) {
            input.values.insert(
                "target_activity".into(),
                crate::model::InputValue {
                    value: 10_f64.powf(log10_activity),
                    unit: "activity".into(),
                },
            );
        }
        if let Some(delta) = environment
            .delta_log10_activity
            .filter(|value| value.is_finite())
        {
            input.values.insert(
                "delta_log10_activity".into(),
                crate::model::InputValue {
                    value: delta,
                    unit: "activity".into(),
                },
            );
        }
        if let Some(drive) = environment
            .transduction_drive
            .filter(|value| value.is_finite())
        {
            input.values.insert(
                "transduction_drive".into(),
                crate::model::InputValue {
                    value: drive,
                    unit: "activity".into(),
                },
            );
        }
        Ok(input)
    }
    pub fn process_covariance(&self, dt_s: f64, noise: &ProcessNoiseConfig) -> DMatrix<f64> {
        let mut q = DMatrix::zeros(self.dimension(), self.dimension());
        for (i, state) in self.definitions.iter().enumerate() {
            q[(i, i)] = match state.name.as_str() {
                "log10_activity" => noise.activity_variance_per_s * dt_s,
                "baseline_offset" | "reference_offset_v" => noise.baseline_variance_v2_per_s * dt_s,
                "polarization" => {
                    let a = (-2.0 * dt_s / self.tau_p_s).exp();
                    noise.polarization_variance_v2_per_s * self.tau_p_s * (1.0 - a) / 2.0
                }
                state_id
                    if self.compiled.is_some()
                        && self.compiled_time_constant_s(state_id).is_some() =>
                {
                    let tau_s = self
                        .compiled_time_constant_s(state_id)
                        .unwrap_or(self.tau_p_s);
                    let a = (-2.0 * dt_s / tau_s).exp();
                    noise.polarization_variance_v2_per_s * tau_s * (1.0 - a) / 2.0
                }
                "sensitivity_scale" => noise.condition_variance_per_s * dt_s,
                _ => 0.0,
            };
        }
        q
    }

    /// Characteristic decay time resolved by stable compiled state and
    /// parameter IDs. This intentionally never falls back to `tau_p_s` for a
    /// compiled dynamic state: missing metadata means no first-order process
    /// covariance is inferred for that state.
    fn compiled_time_constant_s(&self, state_id: &str) -> Option<f64> {
        let compiled = self.compiled.as_ref()?;
        let parameters = self.compiled_parameters.as_ref()?;
        let component = compiled
            .definition()
            .components
            .iter()
            .find(|component| component.state_ids.iter().any(|id| id == state_id))?;
        let parameter_id = component
            .parameter_ids
            .iter()
            .find(|id| id.ends_with("_tau_s"))?;
        let index = compiled.parameter_index(parameter_id)?;
        parameters
            .values
            .get(index)
            .copied()
            .filter(|tau| tau.is_finite() && *tau > 0.0)
    }

    /// All dynamic modes that must settle before the compiled backend can
    /// claim equilibrium. The identifiers are stable state IDs, never UI
    /// labels. Legacy retains its historical single polarization mode.
    pub fn active_dynamic_time_constants_s(&self) -> Vec<(String, f64)> {
        if self.compiled.is_none() {
            return self
                .index("polarization")
                .map(|_| vec![("polarization".into(), self.tau_p_s)])
                .unwrap_or_default();
        }
        self.definitions
            .iter()
            .filter_map(|definition| {
                self.compiled_time_constant_s(&definition.name)
                    .map(|tau_s| (definition.name.clone(), tau_s))
            })
            .collect()
    }
}

pub fn observation_components(
    state: &DVector<f64>,
    env: &AlignedEnvironment,
    model: &StateModel,
    calibration: &dyn CalibrationObservationModel,
) -> Result<(f64, DVector<f64>), EstimationError> {
    if let Some(compiled) = model.compiled_model() {
        let parameters = model.compiled_parameters.as_ref().ok_or_else(|| {
            EstimationError::Numerical("compiled model parameters are unavailable".into())
        })?;
        let input = model.compiled_input(state, env)?;
        let state = crate::model::ModelState::new(state.iter().copied().collect());
        let prediction = compiled
            .observation_prediction(&state, parameters, &input, None)
            .map_err(|error| EstimationError::Numerical(error.to_string()))?;
        let jacobian = compiled
            .observation_jacobian(&state, parameters, &input)
            .map_err(|error| EstimationError::Numerical(error.to_string()))?;
        return Ok((prediction.predicted_voltage_v, DVector::from_vec(jacobian)));
    }
    let activity = model.log10_activity(state)?;
    let h_activity = calibration.predict_potential(activity, env)?;
    let h_zero = calibration.predict_potential(0.0, env)?;
    let mut value = h_activity;
    let mut jacobian = DVector::zeros(model.dimension());
    let mut activity_jacobian = calibration.jacobian_log10_activity(activity, env)?;
    if let Some(i) = model.index("sensitivity_scale") {
        let scale = model.physical_state_value(state, i).ok_or_else(|| {
            EstimationError::Numerical(
                "sensitivity state transform returned a nonfinite value".into(),
            )
        })?;
        let signal = h_activity - h_zero;
        value = h_zero + scale * signal;
        activity_jacobian *= scale;
        jacobian[i] = signal;
    }
    jacobian[model.index("log10_activity").unwrap_or(0)] = activity_jacobian;
    if let Some(i) = model.index("baseline_offset") {
        value += state[i];
        jacobian[i] = 1.0;
    }
    if let Some(i) = model.index("polarization") {
        value += state[i];
        jacobian[i] = 1.0;
    }
    if !value.is_finite() || jacobian.iter().any(|x| !x.is_finite()) {
        return Err(EstimationError::Numerical(
            "measurement model returned a nonfinite value".into(),
        ));
    }
    Ok((value, jacobian))
}

/// Apply an annotated known-standard event as a scalar Kalman constraint on
/// log10 activity.  The voltage innovation remains separate: this observation
/// is durable auxiliary evidence rather than a second voltage measurement.
pub fn apply_known_standard_constraint(
    state: &mut DVector<f64>,
    covariance: &mut DMatrix<f64>,
    environment: &AlignedEnvironment,
    config: &ResolvedEstimationConfig,
) -> Result<Option<AuxiliaryObservation>, EstimationError> {
    if !config.auxiliary.allow_known_standard_events
        || !environment.known_standard
        || environment.known_activity_log10.is_none()
    {
        return Ok(None);
    }
    let value = environment
        .known_activity_log10
        .ok_or_else(|| EstimationError::config("known-standard event has no known activity"))?;
    let index = 0;
    let variance = config.known_log10_activity_variance();
    if !value.is_finite() || !variance.is_finite() || variance <= 0.0 {
        return Err(EstimationError::Covariance(
            "known-standard auxiliary observation is invalid".into(),
        ));
    }
    let latent_value = match config.state_model.activity_transform {
        crate::estimation_config::StateTransformKind::LogPositive => {
            value * std::f64::consts::LN_10
        }
        _ => value,
    };
    let innovation = latent_value - state[index];
    let innovation_variance = covariance[(index, index)] + variance;
    if !innovation_variance.is_finite() || innovation_variance <= 0.0 {
        return Err(EstimationError::Covariance(
            "known-standard auxiliary innovation variance is invalid".into(),
        ));
    }
    let gain = covariance.column(index).into_owned() / innovation_variance;
    let covariance_row = covariance.row(index).into_owned();
    *state += &gain * innovation;
    *covariance -= &gain * covariance_row;
    super::covariance::symmetrize(covariance);
    if !super::covariance::is_psd(covariance, 1e-8) {
        return Err(EstimationError::Covariance(
            "known-standard auxiliary update produced a non-PSD covariance".into(),
        ));
    }
    Ok(Some(AuxiliaryObservation {
        timestamp_s: environment.timestamp_s,
        observation_type: AuxiliaryObservationKind::KnownActivityStandard,
        value,
        variance: Some(variance),
        unit: "log10(activity)".into(),
        variance_unit: Some("log10(activity)^2".into()),
        source: "annotated concentration-standard event".into(),
        comparability: FeatureComparability::Comparable,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::estimation_config::{
        CompiledEstimationProfile, EstimationModelBackend, ResolvedEstimationConfig,
    };

    #[test]
    fn legacy_configuration_keeps_direct_model_backend() {
        let config = ResolvedEstimationConfig::default();
        let model = StateModel::new_compiled(
            &config,
            30.0,
            None,
            &crate::estimation::simulation::simulation_model(),
        )
        .unwrap();
        assert!(model.compiled_model().is_none());
    }

    #[test]
    fn reduced_profile_has_stable_compiled_state_bindings() {
        let mut config = ResolvedEstimationConfig::default();
        config.model.backend = EstimationModelBackend::Compiled;
        config.model.profile = CompiledEstimationProfile::ReducedIsmV1;
        let model = StateModel::new_compiled(
            &config,
            30.0,
            None,
            &crate::estimation::simulation::simulation_model(),
        )
        .unwrap();
        assert_eq!(
            model
                .definitions
                .iter()
                .map(|state| state.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "log10_activity",
                "dynamic_fast_potential_v",
                "dynamic_slow_potential_v",
                "reference_offset_v"
            ]
        );
        assert!(model.compiled_model().is_some());
    }

    #[test]
    fn reduced_profile_applies_activity_step_once_and_uses_component_taus_for_covariance() {
        let mut config = ResolvedEstimationConfig::default();
        config.model.backend = EstimationModelBackend::Compiled;
        config.model.profile = CompiledEstimationProfile::ReducedIsmV1;
        let model = StateModel::new_compiled(
            &config,
            999.0,
            None,
            &crate::estimation::simulation::simulation_model(),
        )
        .unwrap();
        let state = DVector::from_vec(vec![-3.0, 0.0, 0.0, 0.0]);
        let environment = AlignedEnvironment {
            timestamp_s: 1.0,
            temperature_k: Some(298.15),
            delta_log10_activity: Some(1.0),
            ..Default::default()
        };
        let jumped = model.try_process_state(&state, 0.0, &environment).unwrap();
        assert!((jumped[1] - 0.02).abs() < 1e-12);
        assert!((jumped[2] - 0.01).abs() < 1e-12);
        let no_event = model
            .try_process_state(
                &jumped,
                2.0,
                &AlignedEnvironment {
                    timestamp_s: 3.0,
                    temperature_k: Some(298.15),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!((no_event[1] - jumped[1] * (-1.0_f64).exp()).abs() < 1e-12);
        assert!((no_event[2] - jumped[2] * (-2.0_f64 / 35.0).exp()).abs() < 1e-12);

        let q = model.process_covariance(2.0, &config.process_noise);
        let expected_fast =
            config.process_noise.polarization_variance_v2_per_s * 2.0 * (1.0 - (-2.0_f64).exp())
                / 2.0;
        let expected_slow = config.process_noise.polarization_variance_v2_per_s
            * 35.0
            * (1.0 - (-4.0_f64 / 35.0).exp())
            / 2.0;
        assert!((q[(1, 1)] - expected_fast).abs() < 1e-18);
        assert!((q[(2, 2)] - expected_slow).abs() < 1e-18);
    }
}
