use crate::{
    estimation::{
        calibration_adapter::CalibrationObservationModel,
        covariance::{CovarianceResolution, is_psd, resolve_process_covariance, symmetrize},
        environment::{AlignedEnvironment, AlignedEnvironmentSummary},
        error::EstimationError,
        innovation::{InnovationRecord, gating_threshold, residual_autocorrelation},
        measurement::{AuxiliaryObservation, MeasurementObservation},
        model::{StateModel, apply_known_standard_constraint, observation_components},
        state::{
            CalibrationDomainStatus, EstimationWarning, EstimationWarningKind,
            MeasurementUpdateStatus, activity_from_log10,
        },
    },
    estimation_config::ResolvedEstimationConfig,
    results::{FilterDiagnostics, StateEstimatePoint, StateValue},
};
use nalgebra::{DMatrix, DVector};

pub struct FilterInput<'a> {
    pub observations: &'a [MeasurementObservation],
    pub environments: &'a [AlignedEnvironment],
    pub model: &'a StateModel,
    pub calibration: &'a dyn CalibrationObservationModel,
    pub config: &'a ResolvedEstimationConfig,
    pub initial_state: DVector<f64>,
    pub initial_covariance: DMatrix<f64>,
    pub measurement_covariance: &'a CovarianceResolution,
}
pub struct FilterRun {
    pub estimates: Vec<StateEstimatePoint>,
    pub diagnostics: FilterDiagnostics,
    pub process_covariance: Option<CovarianceResolution>,
}

pub fn run(input: FilterInput<'_>) -> Result<FilterRun, EstimationError> {
    let n = input.observations.len();
    if n == 0 {
        return Err(EstimationError::invalid("no time-series observations"));
    }
    let mut state = input.initial_state.clone();
    let mut cov = input.initial_covariance.clone();
    if !is_psd(&cov, 1e-10) {
        return Err(EstimationError::Covariance(
            "initial covariance is not positive semidefinite".into(),
        ));
    }
    let gate = gating_threshold(input.config.filter.innovation_gate_probability);
    let mut points = Vec::with_capacity(n);
    let mut records = Vec::new();
    let mut process_resolution = None;
    let mut accepted = 0;
    let mut rejected = 0;
    let mut predict_only = 0;
    let mut failures = 0;
    let mut domain_excursions = 0;
    let mut previous_time = None;
    for (index, obs) in input.observations.iter().enumerate() {
        let mut warnings = input
            .environments
            .get(index)
            .map(|e| input.calibration.warnings(e))
            .unwrap_or_default();
        let env = input.environments.get(index).cloned().unwrap_or_default();
        let (predicted_state, predicted_cov) = if index == 0 {
            (state.clone(), cov.clone())
        } else {
            let dt = obs.timestamp_s - previous_time.unwrap();
            let f = input.model.transition_matrix(dt);
            let propagated = input.model.process_state(&state, dt, &env);
            let (q, res) = resolve_process_covariance(input.config, input.model, dt)?;
            process_resolution.get_or_insert(res);
            let mut p = &f * &cov * f.transpose() + q;
            symmetrize(&mut p);
            if !is_psd(&p, 1e-8) {
                warnings.push(EstimationWarning::at(
                    EstimationWarningKind::CovarianceNotPositiveSemidefinite,
                    "predicted covariance is not positive semidefinite",
                    obs.timestamp_s,
                ));
                return Err(EstimationError::Covariance(
                    "predicted covariance is not positive semidefinite".into(),
                ));
            }
            (propagated, p)
        };
        previous_time = Some(obs.timestamp_s);
        let mut filtered_state = predicted_state.clone();
        let mut filtered_cov = predicted_cov.clone();
        let mut predicted_measurement = None;
        let mut innovation = None;
        let mut innovation_variance = None;
        let mut standardized = None;
        let mut nis = None;
        let mut auxiliary_observations: Vec<AuxiliaryObservation> = Vec::new();
        let mut status = MeasurementUpdateStatus::Updated;
        let domain = input.calibration.valid_domain_check(
            predicted_state[input.model.index("log10_activity").unwrap_or(0)],
            &env,
        );
        let domain_distance = input.calibration.domain_distance(
            predicted_state[input.model.index("log10_activity").unwrap_or(0)],
            &env,
        );
        if matches!(domain, CalibrationDomainStatus::Outside) {
            domain_excursions += 1;
            warnings.push(EstimationWarning::at(
                EstimationWarningKind::CalibrationExtrapolation,
                "state is outside the stored calibration domain",
                obs.timestamp_s,
            ));
        }
        if let Some(measured) = obs.potential_v {
            match observation_components(&predicted_state, &env, input.model, input.calibration) {
                Ok((pred, h)) => {
                    predicted_measurement = Some(pred);
                    let r = obs
                        .observation_variance_v2
                        .filter(|value| value.is_finite() && *value > 0.0)
                        .unwrap_or(input.measurement_covariance.final_variance);
                    let s = (h.transpose() * &predicted_cov * &h)[(0, 0)] + r;
                    if !s.is_finite() || s <= 0.0 {
                        return Err(EstimationError::Covariance(
                            "innovation variance is invalid".into(),
                        ));
                    }
                    let nu = measured - pred;
                    let std = nu / s.sqrt();
                    let n_i = nu * nu / s;
                    innovation = Some(nu);
                    innovation_variance = Some(s);
                    standardized = Some(std);
                    nis = Some(n_i);
                    let accepted_update = !input.config.filter.reject_outliers || n_i <= gate;
                    let k = &predicted_cov * &h / s;
                    records.push(InnovationRecord {
                        timestamp_s: obs.timestamp_s,
                        innovation_v: nu,
                        innovation_variance_v2: s,
                        standardized_innovation: std,
                        normalized_innovation_squared: n_i,
                        accepted: accepted_update,
                        gating_threshold: gate,
                        kalman_gain: k.iter().copied().collect(),
                        predicted_measurement_v: pred,
                        measurement_residual_v: nu,
                        log_likelihood: Some(-0.5 * ((2.0 * std::f64::consts::PI * s).ln() + n_i)),
                    });
                    if accepted_update {
                        filtered_state = &predicted_state + &k * nu;
                        let identity =
                            DMatrix::identity(input.model.dimension(), input.model.dimension());
                        let kh = &identity - &k * h.transpose();
                        let mut p = &kh * &predicted_cov * kh.transpose() + &k * r * k.transpose();
                        symmetrize(&mut p);
                        if !is_psd(&p, 1e-8) {
                            warnings.push(EstimationWarning::at(
                                EstimationWarningKind::CovarianceNotPositiveSemidefinite,
                                "Joseph-form updated covariance is not positive semidefinite",
                                obs.timestamp_s,
                            ));
                            return Err(EstimationError::Covariance(
                                "updated covariance is not positive semidefinite".into(),
                            ));
                        }
                        filtered_cov = p;
                        accepted += 1;
                    } else {
                        status = MeasurementUpdateStatus::RejectedByGate;
                        rejected += 1;
                        warnings.push(EstimationWarning::at(
                            EstimationWarningKind::InnovationRejected,
                            format!("innovation NIS {:.6} exceeded gate {:.6}", n_i, gate),
                            obs.timestamp_s,
                        ));
                    }
                }
                Err(error) => {
                    status = MeasurementUpdateStatus::MissingEnvironment;
                    warnings.push(EstimationWarning::at(
                        EstimationWarningKind::MissingRequiredEnvironment,
                        error.to_string(),
                        obs.timestamp_s,
                    ));
                    failures += 1;
                }
            }
        } else {
            status = MeasurementUpdateStatus::PredictOnly;
            predict_only += 1;
            warnings.push(EstimationWarning::at(
                EstimationWarningKind::MissingMeasurement,
                "potential is missing; predict-only step retained",
                obs.timestamp_s,
            ));
        }
        if let Some(auxiliary) = apply_known_standard_constraint(
            &mut filtered_state,
            &mut filtered_cov,
            &env,
            input.config,
        )? {
            auxiliary_observations.push(auxiliary);
        }
        for (i, d) in input.model.definitions.iter().enumerate() {
            if let (Some(lo), Some(hi)) = (d.lower_bound, d.upper_bound)
                && (filtered_state[i] < lo || filtered_state[i] > hi)
            {
                warnings.push(EstimationWarning::at(
                    EstimationWarningKind::StateBoundApproached,
                    format!(
                        "state '{}' crossed configured bounds; explicit bound projection applied",
                        d.name
                    ),
                    obs.timestamp_s,
                ));
                filtered_state[i] = filtered_state[i].clamp(lo, hi);
            }
        }
        let state_values = state_values(&filtered_state, &filtered_cov, input.model);
        let predicted_values = state_values_from(&predicted_state, &predicted_cov, input.model);
        let activity =
            activity_from_log10(filtered_state[input.model.index("log10_activity").unwrap_or(0)]);
        let activity_se = filtered_cov[(
            input.model.index("log10_activity").unwrap_or(0),
            input.model.index("log10_activity").unwrap_or(0),
        )]
            .max(0.0)
            .sqrt();
        let concentration = input
            .calibration
            .activity_model_is_ideal()
            .then_some(activity)
            .flatten();
        points.push(StateEstimatePoint {
            timestamp_s: obs.timestamp_s,
            measurement_v: obs.potential_v,
            predicted_measurement_v: predicted_measurement,
            innovation_v: innovation,
            innovation_variance_v2: innovation_variance,
            standardized_innovation: standardized,
            normalized_innovation_squared: nis,
            update_status: status,
            filtered_state: state_values,
            predicted_state: predicted_values,
            filtered_covariance: matrix_vec(&filtered_cov),
            predicted_covariance: matrix_vec(&predicted_cov),
            calibration_domain_status: domain,
            domain_distance,
            environmental_context: AlignedEnvironmentSummary::from(&env),
            activity,
            activity_standard_error: activity.map(|a| std::f64::consts::LN_10 * a * activity_se),
            molar_concentration_mol_l: concentration,
            concentration_unit: concentration.map(|_| "mol/L".into()),
            concentration_assumptions: concentration.map(|_| {
                "ideal activity model; molar concentration is reported as activity in mol/L".into()
            }),
            auxiliary_observations,
            warnings,
        });
        state = filtered_state;
        cov = filtered_cov;
    }
    let innovations = records.iter().map(|r| r.innovation_v).collect::<Vec<_>>();
    let nis_values = records
        .iter()
        .map(|r| r.normalized_innovation_squared)
        .collect::<Vec<_>>();
    let diagnostics = FilterDiagnostics {
        innovation_mean: mean(&innovations),
        innovation_standard_deviation: stddev(&innovations),
        nis_mean: mean(&nis_values),
        nis_exceedance_rate: (!nis_values.is_empty()).then_some(
            nis_values.iter().filter(|v| **v > gate).count() as f64 / nis_values.len() as f64,
        ),
        accepted_update_count: accepted,
        rejected_update_count: rejected,
        predict_only_count: predict_only,
        log_likelihood: Some(records.iter().filter_map(|r| r.log_likelihood).sum()),
        residual_autocorrelation: residual_autocorrelation(&innovations),
        numerical_failures: failures,
        covariance_jitter_count: 0,
        domain_excursion_count: domain_excursions,
        innovations: records,
    };
    Ok(FilterRun {
        estimates: points,
        diagnostics,
        process_covariance: process_resolution,
    })
}

fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then_some(v.iter().sum::<f64>() / v.len() as f64)
}
fn stddev(v: &[f64]) -> Option<f64> {
    if v.len() < 2 {
        return None;
    };
    let m = mean(v)?;
    Some((v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt())
}
fn matrix_vec(m: &DMatrix<f64>) -> Vec<Vec<f64>> {
    (0..m.nrows())
        .map(|i| (0..m.ncols()).map(|j| m[(i, j)]).collect())
        .collect()
}
fn state_values(state: &DVector<f64>, cov: &DMatrix<f64>, model: &StateModel) -> Vec<StateValue> {
    state_values_from(state, cov, model)
}
fn state_values_from(
    state: &DVector<f64>,
    cov: &DMatrix<f64>,
    model: &StateModel,
) -> Vec<StateValue> {
    model
        .definitions
        .iter()
        .enumerate()
        .map(|(i, d)| StateValue {
            name: d.name.clone(),
            value: Some(state[i]),
            standard_error: Some(cov[(i, i)].max(0.0).sqrt()),
            lower: d.lower_bound,
            upper: d.upper_bound,
            unit: d.unit.clone(),
            latent: true,
        })
        .collect()
}
