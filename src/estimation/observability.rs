use crate::{
    estimation::{
        calibration_adapter::CalibrationObservationModel,
        environment::AlignedEnvironment,
        error::EstimationError,
        model::{StateModel, observation_components},
        state::{EstimationWarning, EstimationWarningKind},
    },
    estimation_config::ResolvedEstimationConfig,
};
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservabilityReport {
    pub state_count: usize,
    pub measurement_count: usize,
    pub observation_jacobian: Vec<Vec<f64>>,
    pub observability_matrix: Vec<Vec<f64>>,
    pub numerical_rank: usize,
    pub condition_number: Option<f64>,
    pub weakly_observable_states: Vec<String>,
    pub unobservable_states: Vec<String>,
    pub empirical_identifiability_passed: bool,
    #[serde(default)]
    pub empirical_output_sensitivity: std::collections::BTreeMap<String, f64>,
    #[serde(default)]
    pub state_pair_confounding: Vec<(String, String, f64)>,
    #[serde(default)]
    pub empirical_horizon_steps: usize,
    pub warnings: Vec<EstimationWarning>,
}

/// Diagnose local linearized observability and separately perform a finite
/// perturbation experiment over the actual early environment trajectory.
pub fn diagnose(
    model: &StateModel,
    state: &DVector<f64>,
    environments: &[AlignedEnvironment],
    calibration: &dyn CalibrationObservationModel,
    config: &ResolvedEstimationConfig,
) -> Result<ObservabilityReport, EstimationError> {
    let initial_environment = environments.first().ok_or_else(|| {
        EstimationError::invalid("observability requires an environment trajectory")
    })?;
    let (_, initial_h) = observation_components(state, initial_environment, model, calibration)?;
    let horizon = config
        .observability
        .horizon_steps
        .min(100)
        .min(environments.len().max(1));
    let mut rows: Vec<Vec<f64>> = Vec::new();
    let mut phi = DMatrix::identity(model.dimension(), model.dimension());
    let mut nominal = state.clone();
    let mut previous_time = initial_environment.timestamp_s;
    for step in 0..horizon.max(1) {
        let env = environments.get(step).unwrap_or(initial_environment);
        let (_, h) = observation_components(&nominal, env, model, calibration)?;
        rows.push((h.transpose() * &phi).row(0).iter().copied().collect());
        if step + 1 < horizon {
            let next = environments.get(step + 1).unwrap_or(env);
            let dt = (next.timestamp_s - previous_time).max(f64::EPSILON);
            let f = model.transition_matrix(dt);
            nominal = model.process_state(&nominal, dt, next);
            phi = f * phi;
            previous_time = next.timestamp_s;
        }
    }
    let o = DMatrix::from_vec(
        rows.len(),
        model.dimension(),
        rows.iter().flatten().copied().collect(),
    );
    let singular = o.clone().svd(false, false).singular_values;
    let rank = singular
        .iter()
        .filter(|value| **value > config.observability.rank_tolerance)
        .count();
    let condition_number = singular
        .iter()
        .copied()
        .filter(|value| *value > config.observability.rank_tolerance)
        .fold(None, |acc, value| {
            Some(acc.map_or(value, |max: f64| max.max(value)))
        })
        .zip(
            singular
                .iter()
                .copied()
                .filter(|value| *value > config.observability.rank_tolerance)
                .reduce(f64::min),
        )
        .map(|(max, min)| max / min);

    let mut warnings = Vec::new();
    let mut unobservable = Vec::new();
    let mut weak = Vec::new();
    for i in 0..model.dimension() {
        let sensitivity = (0..o.nrows())
            .map(|row| o[(row, i)].abs())
            .fold(0.0, f64::max);
        if sensitivity <= config.observability.rank_tolerance {
            unobservable.push(model.definitions[i].name.clone());
            warnings.push(EstimationWarning::new(
                EstimationWarningKind::UnobservableModel,
                format!(
                    "state '{}' has negligible local observation sensitivity",
                    model.definitions[i].name
                ),
            ));
        } else if sensitivity < 1.0e-4 {
            weak.push(model.definitions[i].name.clone());
            warnings.push(EstimationWarning::new(
                EstimationWarningKind::WeaklyObservableState,
                format!("state '{}' is weakly observable", model.definitions[i].name),
            ));
        }
    }
    if rank < model.dimension() {
        warnings.push(EstimationWarning::new(
            EstimationWarningKind::UnobservableModel,
            format!(
                "local observability rank {rank} is below state count {}",
                model.dimension()
            ),
        ));
    }
    if condition_number.is_some_and(|value| value > config.observability.maximum_condition_number) {
        warnings.push(EstimationWarning::new(
            EstimationWarningKind::WeaklyObservableState,
            "local observability matrix is ill-conditioned",
        ));
    }

    let perturbation = config.observability.empirical_perturbation;
    let mut output_vectors = Vec::new();
    let mut empirical_output_sensitivity = std::collections::BTreeMap::new();
    for index in 0..model.dimension() {
        let mut plus = state.clone();
        let mut minus = state.clone();
        plus[index] += perturbation;
        minus[index] -= perturbation;
        let plus_outputs = simulate_outputs(&plus, environments, horizon, model, calibration)?;
        let minus_outputs = simulate_outputs(&minus, environments, horizon, model, calibration)?;
        let vector = plus_outputs
            .iter()
            .zip(&minus_outputs)
            .map(|(left, right)| (left - right) / (2.0 * perturbation))
            .collect::<Vec<_>>();
        let sensitivity = (vector.iter().map(|value| value * value).sum::<f64>()
            / vector.len().max(1) as f64)
            .sqrt();
        empirical_output_sensitivity.insert(model.definitions[index].name.clone(), sensitivity);
        output_vectors.push(vector);
    }
    let mut state_pair_confounding = Vec::new();
    for i in 0..model.dimension() {
        for j in (i + 1)..model.dimension() {
            let correlation = correlation(&output_vectors[i], &output_vectors[j]);
            state_pair_confounding.push((
                model.definitions[i].name.clone(),
                model.definitions[j].name.clone(),
                correlation,
            ));
        }
    }
    let empirical_identifiability_passed = empirical_output_sensitivity
        .values()
        .all(|value| *value > config.observability.empirical_sensitivity_tolerance)
        && state_pair_confounding
            .iter()
            .all(|(_, _, value)| value.abs() < 0.999);
    if !empirical_identifiability_passed {
        warnings.push(EstimationWarning::new(
            EstimationWarningKind::WeaklyObservableState,
            "finite-perturbation output sensitivity found weak or confounded states; this is distinct from matrix rank",
        ));
    }
    Ok(ObservabilityReport {
        state_count: model.dimension(),
        measurement_count: rows.len(),
        observation_jacobian: vec![initial_h.iter().copied().collect()],
        observability_matrix: rows,
        numerical_rank: rank,
        condition_number,
        weakly_observable_states: weak,
        unobservable_states: unobservable,
        empirical_identifiability_passed,
        empirical_output_sensitivity,
        state_pair_confounding,
        empirical_horizon_steps: horizon.max(1),
        warnings,
    })
}

fn simulate_outputs(
    initial: &DVector<f64>,
    environments: &[AlignedEnvironment],
    horizon: usize,
    model: &StateModel,
    calibration: &dyn CalibrationObservationModel,
) -> Result<Vec<f64>, EstimationError> {
    let mut state = initial.clone();
    let mut output = Vec::with_capacity(horizon.max(1));
    let mut previous_time = environments.first().map(|e| e.timestamp_s).unwrap_or(0.0);
    for step in 0..horizon.max(1) {
        let env = environments
            .get(step)
            .or_else(|| environments.last())
            .ok_or_else(|| EstimationError::invalid("empty environment trajectory"))?;
        output.push(observation_components(&state, env, model, calibration)?.0);
        if step + 1 < horizon {
            let next = environments.get(step + 1).unwrap_or(env);
            let dt = (next.timestamp_s - previous_time).max(f64::EPSILON);
            state = model.process_state(&state, dt, next);
            previous_time = next.timestamp_s;
        }
    }
    Ok(output)
}

fn correlation(left: &[f64], right: &[f64]) -> f64 {
    if left.len() != right.len() || left.len() < 2 {
        return 0.0;
    }
    let mean_left = left.iter().sum::<f64>() / left.len() as f64;
    let mean_right = right.iter().sum::<f64>() / right.len() as f64;
    let numerator = left
        .iter()
        .zip(right)
        .map(|(l, r)| (l - mean_left) * (r - mean_right))
        .sum::<f64>();
    let denominator = (left.iter().map(|l| (l - mean_left).powi(2)).sum::<f64>()
        * right.iter().map(|r| (r - mean_right).powi(2)).sum::<f64>())
    .sqrt();
    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}
