#![allow(clippy::type_complexity)]

use crate::{
    estimation::{
        covariance::{is_psd, resolve_process_covariance, symmetrize},
        ekf::{FilterInput, FilterRun},
        environment::{AlignedEnvironment, AlignedEnvironmentSummary},
        error::EstimationError,
        innovation::{InnovationRecord, gating_threshold, residual_autocorrelation},
        measurement::AuxiliaryObservation,
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

pub fn sigma_points(
    mean: &DVector<f64>,
    covariance: &DMatrix<f64>,
    config: &ResolvedEstimationConfig,
) -> Result<(Vec<DVector<f64>>, Vec<f64>, Vec<f64>, usize), EstimationError> {
    let n = mean.len();
    if covariance.nrows() != n || covariance.ncols() != n {
        return Err(EstimationError::Covariance(
            "UKF covariance dimension mismatch".into(),
        ));
    }
    let lambda = config.ukf.alpha.powi(2) * (n as f64 + config.ukf.kappa) - n as f64;
    let c = n as f64 + lambda;
    let mut jitter = 0.0;
    let mut factor = None;
    for attempt in 0..config.ukf.maximum_jitter_attempts {
        let mut p = covariance.clone();
        if jitter > 0.0 {
            for i in 0..n {
                p[(i, i)] += jitter;
            }
        }
        if let Some(ch) = p.cholesky() {
            factor = Some(ch);
            break;
        }
        if attempt + 1 < config.ukf.maximum_jitter_attempts {
            jitter = if jitter == 0.0 {
                config.ukf.initial_jitter
            } else {
                jitter * config.ukf.jitter_multiplier
            };
        }
    }
    let Some(ch) = factor else {
        return Err(EstimationError::Numerical(
            "UKF covariance factorization failed after configured jitter attempts".into(),
        ));
    };
    let scale = c.sqrt();
    let l = ch.l();
    let mut points = Vec::with_capacity(2 * n + 1);
    points.push(mean.clone());
    for i in 0..n {
        let column = l.column(i) * scale;
        points.push(mean + &column);
        points.push(mean - &column);
    }
    let mut wm = vec![0.5 / c; 2 * n + 1];
    let mut wc = wm.clone();
    wm[0] = lambda / c;
    wc[0] = lambda / c + (1.0 - config.ukf.alpha.powi(2) + config.ukf.beta);
    Ok((points, wm, wc, (jitter > 0.0) as usize))
}

pub fn run(input: FilterInput<'_>) -> Result<FilterRun, EstimationError> {
    let nobs = input.observations.len();
    if nobs == 0 {
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
    let mut estimates = Vec::with_capacity(nobs);
    let mut records = Vec::new();
    let mut process_resolution = None;
    let mut jitter_count = 0;
    let mut accepted = 0;
    let mut rejected = 0;
    let mut predict_only = 0;
    let mut failures = 0;
    let mut domain_excursions = 0;
    let mut previous = None;
    for (index, obs) in input.observations.iter().enumerate() {
        let env = input.environments.get(index).cloned().unwrap_or_default();
        let mut warnings = input.calibration.warnings(&env);
        let (pred_state, pred_cov) = if index == 0 {
            (state.clone(), cov.clone())
        } else {
            let dt = obs.timestamp_s - previous.unwrap();
            let (pts, wm, wc, j) = sigma_points(&state, &cov, input.config)?;
            jitter_count += j;
            let transformed = pts
                .iter()
                .map(|p| input.model.process_state(p, dt, &env))
                .collect::<Vec<_>>();
            let mut mean = DVector::zeros(input.model.dimension());
            for (w, p) in wm.iter().zip(&transformed) {
                mean += p * *w;
            }
            let (q, res) = resolve_process_covariance(input.config, input.model, dt)?;
            process_resolution.get_or_insert(res);
            let mut p = DMatrix::zeros(input.model.dimension(), input.model.dimension());
            for (w, x) in wc.iter().zip(&transformed) {
                let d = x - &mean;
                p += &d * d.transpose() * *w;
            }
            p += q;
            symmetrize(&mut p);
            if !is_psd(&p, 1e-8) {
                return Err(EstimationError::Covariance(
                    "UKF predicted covariance is not positive semidefinite".into(),
                ));
            }
            (mean, p)
        };
        previous = Some(obs.timestamp_s);
        let mut filtered_state = pred_state.clone();
        let mut filtered_cov = pred_cov.clone();
        let mut predicted_measurement = None;
        let mut innovation = None;
        let mut innovation_variance = None;
        let mut standardized = None;
        let mut nis = None;
        let mut auxiliary_observations: Vec<AuxiliaryObservation> = Vec::new();
        let mut status = MeasurementUpdateStatus::Updated;
        let domain = input.calibration.valid_domain_check(
            pred_state[input.model.index("log10_activity").unwrap_or(0)],
            &env,
        );
        let domain_distance = input.calibration.domain_distance(
            pred_state[input.model.index("log10_activity").unwrap_or(0)],
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
            match sigma_measurement(&pred_state, &pred_cov, &input, &env) {
                Ok((pred, s, hx, px, j)) => {
                    jitter_count += j;
                    predicted_measurement = Some(pred);
                    let nu = measured - pred;
                    innovation = Some(nu);
                    innovation_variance = Some(s);
                    standardized = Some(nu / s.sqrt());
                    nis = Some(nu * nu / s);
                    let accepted_update =
                        !input.config.filter.reject_outliers || nu * nu / s <= gate;
                    let k = &px / s;
                    records.push(InnovationRecord {
                        timestamp_s: obs.timestamp_s,
                        innovation_v: nu,
                        innovation_variance_v2: s,
                        standardized_innovation: nu / s.sqrt(),
                        normalized_innovation_squared: nu * nu / s,
                        accepted: accepted_update,
                        gating_threshold: gate,
                        kalman_gain: k.iter().copied().collect(),
                        predicted_measurement_v: pred,
                        measurement_residual_v: nu,
                        log_likelihood: Some(
                            -0.5 * ((2.0 * std::f64::consts::PI * s).ln() + nu * nu / s),
                        ),
                    });
                    if accepted_update {
                        filtered_state = &pred_state + &k * nu;
                        let mut p = &pred_cov - &k * s * k.transpose();
                        symmetrize(&mut p);
                        if !is_psd(&p, 1e-8) {
                            return Err(EstimationError::Covariance(
                                "UKF updated covariance is not positive semidefinite".into(),
                            ));
                        }
                        filtered_cov = p;
                        accepted += 1;
                    } else {
                        status = MeasurementUpdateStatus::RejectedByGate;
                        rejected += 1;
                        warnings.push(EstimationWarning::at(
                            EstimationWarningKind::InnovationRejected,
                            format!(
                                "innovation NIS {:.6} exceeded gate {:.6}",
                                nu * nu / s,
                                gate
                            ),
                            obs.timestamp_s,
                        ));
                    }
                    let _ = hx;
                }
                Err(error) => {
                    status = MeasurementUpdateStatus::MissingEnvironment;
                    failures += 1;
                    warnings.push(EstimationWarning::at(
                        EstimationWarningKind::MissingRequiredEnvironment,
                        error.to_string(),
                        obs.timestamp_s,
                    ));
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
                filtered_state[i] = filtered_state[i].clamp(lo, hi);
                warnings.push(EstimationWarning::at(
                    EstimationWarningKind::StateBoundApproached,
                    format!(
                        "state '{}' crossed configured bounds; explicit bound projection applied",
                        d.name
                    ),
                    obs.timestamp_s,
                ));
            }
        }
        let activity =
            activity_from_log10(filtered_state[input.model.index("log10_activity").unwrap_or(0)]);
        let aidx = input.model.index("log10_activity").unwrap_or(0);
        let ase = filtered_cov[(aidx, aidx)].max(0.0).sqrt();
        let concentration = input
            .calibration
            .activity_model_is_ideal()
            .then_some(activity)
            .flatten();
        estimates.push(StateEstimatePoint {
            timestamp_s: obs.timestamp_s,
            measurement_v: obs.potential_v,
            predicted_measurement_v: predicted_measurement,
            innovation_v: innovation,
            innovation_variance_v2: innovation_variance,
            standardized_innovation: standardized,
            normalized_innovation_squared: nis,
            update_status: status,
            filtered_state: values(&filtered_state, &filtered_cov, input.model),
            predicted_state: values(&pred_state, &pred_cov, input.model),
            filtered_covariance: matrix(&filtered_cov),
            predicted_covariance: matrix(&pred_cov),
            calibration_domain_status: domain,
            domain_distance,
            environmental_context: AlignedEnvironmentSummary::from(&env),
            activity,
            activity_standard_error: activity.map(|x| std::f64::consts::LN_10 * x * ase),
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
    let iv = records.iter().map(|r| r.innovation_v).collect::<Vec<_>>();
    let nv = records
        .iter()
        .map(|r| r.normalized_innovation_squared)
        .collect::<Vec<_>>();
    let diagnostics = FilterDiagnostics {
        innovation_mean: mean(&iv),
        innovation_standard_deviation: stddev(&iv),
        nis_mean: mean(&nv),
        nis_exceedance_rate: (!nv.is_empty())
            .then_some(nv.iter().filter(|x| **x > gate).count() as f64 / nv.len() as f64),
        accepted_update_count: accepted,
        rejected_update_count: rejected,
        predict_only_count: predict_only,
        log_likelihood: Some(records.iter().filter_map(|r| r.log_likelihood).sum()),
        residual_autocorrelation: residual_autocorrelation(&iv),
        numerical_failures: failures,
        covariance_jitter_count: jitter_count,
        domain_excursion_count: domain_excursions,
        innovations: records,
    };
    Ok(FilterRun {
        estimates,
        diagnostics,
        process_covariance: process_resolution,
    })
}

fn sigma_measurement(
    state: &DVector<f64>,
    cov: &DMatrix<f64>,
    input: &FilterInput<'_>,
    env: &AlignedEnvironment,
) -> Result<(f64, f64, DVector<f64>, DVector<f64>, usize), EstimationError> {
    let (points, wm, wc, j) = sigma_points(state, cov, input.config)?;
    let ys = points
        .iter()
        .map(|p| observation_components(p, env, input.model, input.calibration).map(|x| x.0))
        .collect::<Result<Vec<_>, _>>()?;
    let pred = ys.iter().zip(&wm).map(|(y, w)| y * w).sum::<f64>();
    let mut variance = 0.0;
    let mut cross = DVector::zeros(state.len());
    for ((p, y), w) in points.iter().zip(&ys).zip(&wc) {
        let dy = y - pred;
        variance += w * dy * dy;
        cross += &(p - state) * (*w * dy);
    }
    variance += input.measurement_covariance.final_variance;
    if !variance.is_finite() || variance <= 0.0 {
        return Err(EstimationError::Covariance(
            "UKF innovation variance is invalid".into(),
        ));
    }
    Ok((pred, variance, DVector::zeros(state.len()), cross, j))
}
fn matrix(m: &DMatrix<f64>) -> Vec<Vec<f64>> {
    (0..m.nrows())
        .map(|i| (0..m.ncols()).map(|j| m[(i, j)]).collect())
        .collect()
}
fn values(s: &DVector<f64>, p: &DMatrix<f64>, m: &StateModel) -> Vec<StateValue> {
    m.definitions
        .iter()
        .enumerate()
        .map(|(i, d)| StateValue {
            name: d.name.clone(),
            value: Some(s[i]),
            standard_error: Some(p[(i, i)].max(0.0).sqrt()),
            lower: d.lower_bound,
            upper: d.upper_bound,
            unit: d.unit.clone(),
            latent: true,
        })
        .collect()
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
