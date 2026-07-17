//! Deterministic nonlinear least-squares fitting and residual bootstrap.

use super::diagnostics::{compute_statistics, derived_features};
use super::models::{TransientModelKind, evaluate};
use crate::potentiometry::PotentiometryError;
use crate::potentiometry::transient::segmentation::PreparedSegment;
use crate::results::transient::{
    FitStatus, FittedTransientParameter, ParameterConfidenceInterval, TransientFitResult,
    TransientWarning, TransientWarningKind,
};
use crate::transient_config::ResolvedTransientConfig;
use levenberg_marquardt::{LeastSquaresProblem, LevenbergMarquardt};
use nalgebra::{DMatrix, DVector, Dyn, Owned};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

pub fn fit_transient_model(
    segment: &PreparedSegment,
    model: TransientModelKind,
    config: &ResolvedTransientConfig,
    rng: &mut StdRng,
) -> Result<TransientFitResult, PotentiometryError> {
    fit_model_on_values(segment, model, config, &segment.fit_values, rng)
}

fn fit_model_on_values(
    segment: &PreparedSegment,
    model: TransientModelKind,
    config: &ResolvedTransientConfig,
    values: &[f64],
    rng: &mut StdRng,
) -> Result<TransientFitResult, PotentiometryError> {
    if values.len() != segment.fit_time_local.len() || values.is_empty() {
        return Err(PotentiometryError::invalid(
            "transient fit values are not aligned with segment timestamps",
        ));
    }
    let duration = segment.summary.finite_duration_s.unwrap_or(1.0).max(1e-6);
    let starts = initial_guesses(model, &segment.fit_time_local, values, duration, config);
    let mut starts = starts;
    starts.shuffle(rng);
    starts.truncate(config.optimizer.multiple_starts.max(1));

    let mut best: Option<(Vec<f64>, String, f64)> = None;
    for physical_start in starts {
        let internal_start = physical_to_internal(model, &physical_start, config, duration)?;
        let problem = TransientLeastSquaresProblem {
            model,
            times: segment.fit_time_local.clone(),
            values: values.to_vec(),
            beta_min: config.models.beta_min,
            beta_max: config.models.beta_max,
            params: DVector::from_vec(internal_start),
        };
        let solver = LevenbergMarquardt::new()
            .with_ftol(config.optimizer.ftol)
            .with_xtol(config.optimizer.xtol)
            .with_gtol(config.optimizer.gtol)
            .with_patience(
                config
                    .optimizer
                    .patience
                    .min(config.optimizer.maximum_iterations.max(1)),
            )
            .with_stepbound(config.optimizer.step_bound);
        let (result, report) = solver.minimize(problem);
        if !report.termination.was_successful() || !report.objective_function.is_finite() {
            continue;
        }
        let physical = internal_to_physical(model, &result.params, config)?;
        if physical.iter().any(|value| !value.is_finite()) {
            continue;
        }
        let objective = report.objective_function;
        let reason = format!("{:?}", report.termination);
        let is_better = best
            .as_ref()
            .is_none_or(|(_, _, best_objective)| objective < *best_objective);
        if is_better {
            best = Some((physical, reason, objective));
        }
    }

    let (physical_parameters, termination_reason, _) =
        best.ok_or_else(|| PotentiometryError::OptimizerFailure {
            model: model.to_string(),
            reason: "no initial guess reached a successful finite LM termination".to_string(),
        })?;
    let predicted_relative = segment
        .fit_time_local
        .iter()
        .map(|time| evaluate(model, &physical_parameters, *time).map(|components| components.total))
        .collect::<Result<Vec<_>, _>>()?;
    let predicted_absolute = predicted_relative
        .iter()
        .map(|value| value + segment.response_offset)
        .collect::<Vec<_>>();
    let observed_absolute = values
        .iter()
        .map(|value| *value + segment.response_offset)
        .collect::<Vec<_>>();
    let (mut statistics, mut warnings) = compute_statistics(
        &observed_absolute,
        &predicted_absolute,
        model.parameter_count(),
        config.selection.criterion,
    )?;
    statistics.optimizer_termination_reason = Some(termination_reason);
    let (features, feature_warnings) = derived_features(
        model,
        &physical_parameters,
        &segment.summary,
        &segment.baseline,
        segment.response_offset,
        config,
    )?;
    warnings.extend(feature_warnings);
    let covariance_condition_number = covariance_condition_number(
        model,
        &physical_parameters,
        &segment.fit_time_local,
        &segment.fit_values,
        config,
        duration,
    );
    statistics.covariance_condition_number = covariance_condition_number;
    if covariance_condition_number.is_none() {
        warnings.push(TransientWarning::new(
            TransientWarningKind::SingularCovariance,
            "the local covariance approximation was singular or poorly conditioned",
        ));
    }
    if statistics
        .lag1_residual_autocorrelation
        .is_some_and(|value| value.abs() > config.validation.high_autocorrelation_threshold)
    {
        warnings.push(TransientWarning::new(
            TransientWarningKind::HighResidualAutocorrelation,
            "lag-1 residual autocorrelation exceeds the configured warning threshold",
        ));
    }
    if statistics.aicc.is_none() {
        warnings.push(TransientWarning::new(
            TransientWarningKind::AiccUnavailable,
            "AICc is unavailable for this observation-to-parameter ratio",
        ));
    }
    add_bound_warnings(model, &physical_parameters, config, duration, &mut warnings);

    let parameters = model
        .parameter_names()
        .iter()
        .zip(model.parameter_units().iter())
        .zip(physical_parameters.iter())
        .map(|((name, unit), value)| FittedTransientParameter {
            name: (*name).to_string(),
            unit: (*unit).to_string(),
            value: if *name == "E_infinity" {
                *value + segment.response_offset
            } else {
                *value
            },
        })
        .collect();

    Ok(TransientFitResult {
        model,
        status: if warnings
            .iter()
            .any(|warning| warning.kind == TransientWarningKind::NotIdentifiable)
        {
            FitStatus::Invalid
        } else {
            FitStatus::Converged
        },
        parameters,
        derived_features: features,
        statistics,
        confidence_intervals: Vec::new(),
        predicted_v: predicted_absolute.clone(),
        residuals_v: observed_absolute
            .iter()
            .zip(predicted_absolute.iter())
            .map(|(observed, predicted)| observed - predicted)
            .collect(),
        warnings,
        fit_parameters: physical_parameters,
        response_offset: segment.response_offset,
    })
}

pub fn bootstrap_fit(
    segment: &PreparedSegment,
    fit: &mut TransientFitResult,
    config: &ResolvedTransientConfig,
) -> Result<(), PotentiometryError> {
    let iterations = config.uncertainty.bootstrap_iterations;
    if iterations == 0 || !fit.is_successful() {
        return Ok(());
    }
    if fit.residuals_v.len() != segment.fit_values.len() {
        return Err(PotentiometryError::BootstrapFailure(
            "selected fit residuals are not aligned with the segment".to_string(),
        ));
    }
    let residuals = fit.residuals_v.clone();
    let fitted_absolute = fit.predicted_v.clone();
    let mut rng = StdRng::seed_from_u64(config.uncertainty.seed);
    let mut successful = Vec::new();
    for _ in 0..iterations {
        let bootstrap_absolute = fitted_absolute
            .iter()
            .map(|prediction| {
                let index = rng.gen_range(0..residuals.len());
                prediction + residuals[index]
            })
            .collect::<Vec<_>>();
        let bootstrap_relative = bootstrap_absolute
            .iter()
            .map(|value| *value - segment.response_offset)
            .collect::<Vec<_>>();
        match fit_model_on_values(segment, fit.model, config, &bootstrap_relative, &mut rng) {
            Ok(result) if result.is_successful() => successful.push(result.fit_parameters),
            _ => {}
        }
    }

    let successful_count = successful.len();
    let required = iterations as f64 * config.uncertainty.minimum_success_fraction;
    let failed_count = iterations - successful_count;
    if successful_count == 0 || (successful_count as f64) < required {
        fit.confidence_intervals.clear();
        fit.warnings.push(TransientWarning::new(
            TransientWarningKind::BootstrapUnavailable,
            format!(
                "bootstrap confidence intervals unavailable: {successful_count}/{iterations} iterations succeeded"
            ),
        ));
        return Ok(());
    }

    let mut intervals = Vec::new();
    let alpha = (1.0 - config.uncertainty.confidence_level) / 2.0;
    for parameter_index in 0..fit.model.parameter_count() {
        let mut values = successful
            .iter()
            .map(|parameters| {
                let value = parameters[parameter_index];
                if parameter_index == 0 {
                    value + segment.response_offset
                } else {
                    value
                }
            })
            .collect::<Vec<_>>();
        values.sort_by(f64::total_cmp);
        intervals.push(ParameterConfidenceInterval {
            name: fit.model.parameter_names()[parameter_index].to_string(),
            unit: fit.model.parameter_units()[parameter_index].to_string(),
            lower: percentile(&values, alpha),
            upper: percentile(&values, 1.0 - alpha),
            confidence_level: config.uncertainty.confidence_level,
            successful_iterations: successful_count,
            failed_iterations: failed_count,
        });
    }
    fit.confidence_intervals = intervals;
    Ok(())
}

fn percentile(values: &[f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let position = percentile.clamp(0.0, 1.0) * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        Some(values[lower])
    } else {
        Some(values[lower] + (values[upper] - values[lower]) * (position - lower as f64))
    }
}

#[derive(Debug, Clone)]
struct TransientLeastSquaresProblem {
    model: TransientModelKind,
    times: Vec<f64>,
    values: Vec<f64>,
    beta_min: f64,
    beta_max: f64,
    params: DVector<f64>,
}

impl TransientLeastSquaresProblem {
    fn physical_parameters(&self, params: &DVector<f64>) -> Option<Vec<f64>> {
        internal_to_physical_with_bounds(self.model, params, self.beta_min, self.beta_max).ok()
    }

    fn residual_vector(&self, params: &DVector<f64>) -> Option<DVector<f64>> {
        let physical = self.physical_parameters(params)?;
        let residuals = self
            .times
            .iter()
            .zip(self.values.iter())
            .map(|(time, value)| {
                evaluate(self.model, &physical, *time)
                    .ok()
                    .map(|components| components.total - *value)
            })
            .collect::<Option<Vec<_>>>()?;
        if residuals.iter().any(|value| !value.is_finite()) {
            return None;
        }
        Some(DVector::from_vec(residuals))
    }
}

impl LeastSquaresProblem<f64, Dyn, Dyn> for TransientLeastSquaresProblem {
    type ParameterStorage = Owned<f64, Dyn>;
    type ResidualStorage = Owned<f64, Dyn>;
    type JacobianStorage = Owned<f64, Dyn, Dyn>;

    fn set_params(&mut self, params: &DVector<f64>) {
        self.params = params.clone();
    }

    fn params(&self) -> DVector<f64> {
        self.params.clone()
    }

    fn residuals(&self) -> Option<DVector<f64>> {
        self.residual_vector(&self.params)
    }

    fn jacobian(&self) -> Option<DMatrix<f64>> {
        let base = self.params.clone();
        let residual_len = self.values.len();
        let mut jacobian = DMatrix::zeros(residual_len, base.len());
        for parameter_index in 0..base.len() {
            let step = 1e-5 * base[parameter_index].abs().max(1.0);
            let mut plus = base.clone();
            let mut minus = base.clone();
            plus[parameter_index] += step;
            minus[parameter_index] -= step;
            let residual_plus = self.residual_vector(&plus)?;
            let residual_minus = self.residual_vector(&minus)?;
            for row in 0..residual_len {
                jacobian[(row, parameter_index)] =
                    (residual_plus[row] - residual_minus[row]) / (2.0 * step);
            }
        }
        Some(jacobian)
    }
}

fn initial_guesses(
    model: TransientModelKind,
    times: &[f64],
    values: &[f64],
    duration: f64,
    config: &ResolvedTransientConfig,
) -> Vec<Vec<f64>> {
    let initial = values.first().copied().unwrap_or(0.0);
    let late_count = values.len().clamp(1, 10);
    let mut late = values[values.len() - late_count..].to_vec();
    late.sort_by(f64::total_cmp);
    let equilibrium = late[late.len() / 2];
    let amplitude = initial - equilibrium;
    let tau_fast_values = [duration / 100.0, duration / 50.0, duration / 20.0]
        .into_iter()
        .map(|value| value.max(1e-4))
        .collect::<Vec<_>>();
    let tau_slow_values = [duration / 2.0, duration, duration * 2.0]
        .into_iter()
        .map(|value| value.max(1e-3))
        .collect::<Vec<_>>();
    let mut guesses = Vec::new();
    match model {
        TransientModelKind::Single => {
            for tau in tau_slow_values {
                guesses.push(vec![equilibrium, amplitude, tau]);
            }
        }
        TransientModelKind::Double => {
            for tau_fast in &tau_fast_values {
                for tau_slow in &tau_slow_values {
                    if tau_fast < tau_slow {
                        guesses.push(vec![
                            equilibrium,
                            amplitude * 0.7,
                            amplitude * 0.3,
                            *tau_fast,
                            *tau_slow,
                        ]);
                    }
                }
            }
        }
        TransientModelKind::DoubleDrift => {
            for tau_fast in &tau_fast_values {
                for tau_slow in &tau_slow_values {
                    if tau_fast < tau_slow {
                        guesses.push(vec![
                            equilibrium,
                            amplitude * 0.7,
                            amplitude * 0.3,
                            *tau_fast,
                            *tau_slow,
                            0.0,
                        ]);
                    }
                }
            }
        }
        TransientModelKind::Stretched => {
            for tau in tau_slow_values {
                for beta in [0.5, 0.7, 0.9] {
                    guesses.push(vec![equilibrium, amplitude, tau, beta]);
                }
            }
        }
    }
    if guesses.is_empty() {
        guesses.push(vec![equilibrium, amplitude, duration.max(1e-3)]);
    }
    let _ = times;
    let _ = config;
    guesses
}

fn physical_to_internal(
    model: TransientModelKind,
    parameters: &[f64],
    config: &ResolvedTransientConfig,
    duration: f64,
) -> Result<Vec<f64>, PotentiometryError> {
    if parameters.len() != model.parameter_count() {
        return Err(PotentiometryError::invalid(
            "initial guess has wrong parameter count",
        ));
    }
    let tau_max = duration.max(1e-3) * 1000.0;
    let mut internal = parameters.to_vec();
    match model {
        TransientModelKind::Single => internal[2] = parameters[2].clamp(1e-6, tau_max).ln(),
        TransientModelKind::Double | TransientModelKind::DoubleDrift => {
            internal[3] = parameters[3].clamp(1e-6, tau_max).ln();
            internal[4] = (parameters[4] - parameters[3]).max(1e-6).min(tau_max).ln();
        }
        TransientModelKind::Stretched => {
            internal[2] = parameters[2].clamp(1e-6, tau_max).ln();
            internal[3] = bounded_logit(
                parameters[3],
                config.models.beta_min,
                config.models.beta_max,
            );
        }
    }
    Ok(internal)
}

fn internal_to_physical(
    model: TransientModelKind,
    params: &DVector<f64>,
    config: &ResolvedTransientConfig,
) -> Result<Vec<f64>, PotentiometryError> {
    internal_to_physical_with_bounds(
        model,
        params,
        config.models.beta_min,
        config.models.beta_max,
    )
}

fn internal_to_physical_with_bounds(
    model: TransientModelKind,
    params: &DVector<f64>,
    beta_min: f64,
    beta_max: f64,
) -> Result<Vec<f64>, PotentiometryError> {
    if params.len() != model.parameter_count() || params.iter().any(|value| !value.is_finite()) {
        return Err(PotentiometryError::invalid(
            "optimizer parameter vector is invalid",
        ));
    }
    let mut physical = params.iter().copied().collect::<Vec<_>>();
    match model {
        TransientModelKind::Single => physical[2] = params[2].exp(),
        TransientModelKind::Double | TransientModelKind::DoubleDrift => {
            physical[3] = params[3].exp();
            physical[4] = physical[3] + params[4].exp();
        }
        TransientModelKind::Stretched => {
            physical[2] = params[2].exp();
            physical[3] = beta_min + (beta_max - beta_min) * sigmoid(params[3]);
        }
    }
    if physical.iter().any(|value| !value.is_finite()) {
        return Err(PotentiometryError::invalid(
            "inverse parameter transform overflowed",
        ));
    }
    Ok(physical)
}

fn bounded_logit(value: f64, lower: f64, upper: f64) -> f64 {
    let normalized = ((value - lower) / (upper - lower)).clamp(1e-8, 1.0 - 1e-8);
    (normalized / (1.0 - normalized)).ln()
}

fn sigmoid(value: f64) -> f64 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exp = value.exp();
        exp / (1.0 + exp)
    }
}

fn covariance_condition_number(
    model: TransientModelKind,
    parameters: &[f64],
    times: &[f64],
    values: &[f64],
    config: &ResolvedTransientConfig,
    duration: f64,
) -> Option<f64> {
    let internal = physical_to_internal(model, parameters, config, duration).ok()?;
    let base = DVector::from_vec(internal);
    let residual_for = |params: &DVector<f64>| -> Option<DVector<f64>> {
        let physical = internal_to_physical(model, params, config).ok()?;
        let values = times
            .iter()
            .zip(values.iter())
            .map(|(time, value)| {
                evaluate(model, &physical, *time)
                    .ok()
                    .map(|components| components.total - *value)
            })
            .collect::<Option<Vec<_>>>()?;
        Some(DVector::from_vec(values))
    };
    let mut jacobian = DMatrix::zeros(values.len(), base.len());
    for column in 0..base.len() {
        let step = 1e-5 * base[column].abs().max(1.0);
        let mut plus = base.clone();
        let mut minus = base.clone();
        plus[column] += step;
        minus[column] -= step;
        let plus = residual_for(&plus)?;
        let minus = residual_for(&minus)?;
        for row in 0..values.len() {
            jacobian[(row, column)] = (plus[row] - minus[row]) / (2.0 * step);
        }
    }
    let svd = jacobian.svd(false, false);
    let singular_values = svd.singular_values;
    let largest = singular_values.iter().copied().fold(0.0, f64::max);
    let smallest = singular_values
        .iter()
        .copied()
        .filter(|value| *value > 1e-12)
        .fold(f64::INFINITY, f64::min);
    if largest.is_finite() && smallest.is_finite() {
        Some(largest / smallest)
    } else {
        None
    }
}

fn add_bound_warnings(
    model: TransientModelKind,
    parameters: &[f64],
    config: &ResolvedTransientConfig,
    duration: f64,
    warnings: &mut Vec<TransientWarning>,
) {
    let tau_max = duration.max(1e-3) * 1000.0;
    let tau_values = match model {
        TransientModelKind::Single | TransientModelKind::Stretched => vec![parameters[2]],
        TransientModelKind::Double | TransientModelKind::DoubleDrift => {
            vec![parameters[3], parameters[4]]
        }
    };
    if tau_values.iter().any(|tau| {
        *tau <= 1e-6 * (1.0 + config.validation.bound_proximity_fraction)
            || *tau >= tau_max * (1.0 - config.validation.bound_proximity_fraction)
    }) {
        warnings.push(TransientWarning::new(
            TransientWarningKind::ParameterAtBound,
            "one or more time constants are close to the numerical transform bounds",
        ));
    }
    if model == TransientModelKind::Stretched {
        let beta = parameters[3];
        if beta <= config.models.beta_min * (1.0 + config.validation.bound_proximity_fraction)
            || beta
                >= config.models.beta_max
                    - (config.models.beta_max - config.models.beta_min)
                        * config.validation.bound_proximity_fraction
        {
            warnings.push(TransientWarning::new(
                TransientWarningKind::ParameterAtBound,
                "stretched-exponential beta is close to a configured bound",
            ));
        }
    }
}
