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
    pub warnings: Vec<EstimationWarning>,
}

pub fn diagnose(
    model: &StateModel,
    state: &DVector<f64>,
    env: &AlignedEnvironment,
    calibration: &dyn CalibrationObservationModel,
    config: &ResolvedEstimationConfig,
) -> Result<ObservabilityReport, EstimationError> {
    let (_, h) = observation_components(state, env, model, calibration)?;
    let f = model.transition_matrix(1.0);
    let horizon = config.observability.horizon_steps.min(100);
    let mut rows = Vec::new();
    let mut power = DMatrix::identity(model.dimension(), model.dimension());
    for _ in 0..horizon.max(1) {
        let row = &h.transpose() * &power;
        rows.push(row.row(0).iter().copied().collect::<Vec<_>>());
        power = &power * &f;
    }
    let o = DMatrix::from_vec(
        rows.len(),
        model.dimension(),
        rows.iter().flatten().copied().collect(),
    );
    let svd = o.clone().svd(false, false);
    let singular = svd.singular_values;
    let rank = singular
        .iter()
        .filter(|x| **x > config.observability.rank_tolerance)
        .count();
    let cond = singular
        .iter()
        .copied()
        .filter(|x| *x > config.observability.rank_tolerance)
        .fold(None, |acc, x| Some(acc.map_or(x, |m: f64| m.max(x))))
        .zip(
            singular
                .iter()
                .copied()
                .filter(|x| *x > config.observability.rank_tolerance)
                .reduce(f64::min),
        )
        .map(|(max, min)| max / min);
    let mut warnings = Vec::new();
    let mut unobservable = Vec::new();
    let mut weak = Vec::new();
    for i in 0..model.dimension() {
        let sensitivity = (0..o.nrows()).map(|r| o[(r, i)].abs()).fold(0.0, f64::max);
        if sensitivity <= config.observability.rank_tolerance {
            unobservable.push(model.definitions[i].name.clone());
            warnings.push(EstimationWarning::new(
                EstimationWarningKind::UnobservableModel,
                format!(
                    "state '{}' has negligible local observation sensitivity",
                    model.definitions[i].name
                ),
            ));
        } else if sensitivity < 1e-4 {
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
                "observability rank {rank} is below state count {}",
                model.dimension()
            ),
        ));
    }
    if cond.is_some_and(|x| x > config.observability.maximum_condition_number) {
        warnings.push(EstimationWarning::new(
            EstimationWarningKind::WeaklyObservableState,
            "observability matrix is ill-conditioned",
        ));
    }
    let empirical = rank == model.dimension() && !model.has_condition();
    let report = ObservabilityReport {
        state_count: model.dimension(),
        measurement_count: rows.len(),
        observation_jacobian: vec![h.iter().copied().collect()],
        observability_matrix: rows,
        numerical_rank: rank,
        condition_number: cond,
        weakly_observable_states: weak,
        unobservable_states: unobservable,
        empirical_identifiability_passed: empirical,
        warnings,
    };
    Ok(report)
}
