use crate::{
    estimation::{
        calibration_adapter::CalibrationObservationModel,
        environment::AlignedEnvironment,
        error::EstimationError,
        measurement::{AuxiliaryObservation, AuxiliaryObservationKind},
        state::{StateDefinition, state_definitions},
    },
    estimation_config::{ProcessNoiseConfig, ResolvedEstimationConfig, StateModelKind},
    results::FeatureComparability,
};
use nalgebra::{DMatrix, DVector};

#[derive(Debug, Clone)]
pub struct StateModel {
    pub kind: StateModelKind,
    pub definitions: Vec<StateDefinition>,
    pub tau_p_s: f64,
    pub tau_uncertainty_s: Option<f64>,
    pub gain: f64,
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
        })
    }
    pub fn dimension(&self) -> usize {
        self.definitions.len()
    }
    pub fn has_baseline(&self) -> bool {
        self.definitions.iter().any(|x| x.name == "baseline_offset")
    }
    pub fn has_polarization(&self) -> bool {
        self.definitions.iter().any(|x| x.name == "polarization")
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
    pub fn process_covariance(&self, dt_s: f64, noise: &ProcessNoiseConfig) -> DMatrix<f64> {
        let mut q = DMatrix::zeros(self.dimension(), self.dimension());
        for (i, state) in self.definitions.iter().enumerate() {
            q[(i, i)] = match state.name.as_str() {
                "log10_activity" => noise.activity_variance_per_s * dt_s,
                "baseline_offset" => noise.baseline_variance_v2_per_s * dt_s,
                "polarization" => {
                    let a = (-2.0 * dt_s / self.tau_p_s).exp();
                    noise.polarization_variance_v2_per_s * self.tau_p_s * (1.0 - a) / 2.0
                }
                "sensitivity_scale" => noise.condition_variance_per_s * dt_s,
                _ => 0.0,
            };
        }
        q
    }
}

pub fn observation_components(
    state: &DVector<f64>,
    env: &AlignedEnvironment,
    model: &StateModel,
    calibration: &dyn CalibrationObservationModel,
) -> Result<(f64, DVector<f64>), EstimationError> {
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
    let value = environment.known_activity_log10.unwrap();
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
