//! Fit statistics, residual diagnostics, and derived transient features.

use super::models::{ModelComponents, TransientModelKind, evaluate, initial_response_rate};
use crate::potentiometry::PotentiometryError;
use crate::results::transient::{
    BaselineResult, SegmentSummary, TransientFeatures, TransientFitStatistics, TransientWarning,
    TransientWarningKind,
};
use crate::transient_config::{ResolvedTransientConfig, SelectionCriterion};

pub fn compute_statistics(
    observed: &[f64],
    predicted: &[f64],
    parameter_count: usize,
    criterion: SelectionCriterion,
) -> Result<(TransientFitStatistics, Vec<TransientWarning>), PotentiometryError> {
    if observed.len() != predicted.len() || observed.is_empty() {
        return Err(PotentiometryError::invalid(
            "statistics require equally sized non-empty observations and predictions",
        ));
    }
    if observed
        .iter()
        .chain(predicted.iter())
        .any(|value| !value.is_finite())
    {
        return Err(PotentiometryError::invalid(
            "statistics cannot be computed from non-finite values",
        ));
    }

    let residuals = observed
        .iter()
        .zip(predicted.iter())
        .map(|(observed, predicted)| observed - predicted)
        .collect::<Vec<_>>();
    let n = residuals.len();
    let rss = residuals.iter().map(|value| value * value).sum::<f64>();
    let rss_for_log = rss.max(f64::MIN_POSITIVE);
    let log_likelihood_term = n as f64 * (rss_for_log / n as f64).ln();
    let aic = log_likelihood_term + 2.0 * parameter_count as f64;
    let aicc = (n > parameter_count + 1).then(|| {
        aic + (2.0 * parameter_count as f64 * (parameter_count as f64 + 1.0))
            / (n as f64 - parameter_count as f64 - 1.0)
    });
    let bic = log_likelihood_term + parameter_count as f64 * (n as f64).ln();
    let mean = observed.iter().sum::<f64>() / n as f64;
    let total_sum_squares = observed
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>();
    let r_squared = (total_sum_squares > f64::EPSILON).then_some(1.0 - rss / total_sum_squares);
    let adjusted_r_squared = (r_squared.is_some() && n > parameter_count + 1).then(|| {
        1.0 - (1.0 - r_squared.unwrap_or(0.0)) * (n as f64 - 1.0)
            / (n as f64 - parameter_count as f64 - 1.0)
    });
    let rmse = (rss / n as f64).sqrt();
    let mae = residuals.iter().map(|value| value.abs()).sum::<f64>() / n as f64;
    let maximum_absolute_residual = residuals
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    let durbin_watson_denominator = rss.max(f64::MIN_POSITIVE);
    let durbin_watson = if n > 1 {
        Some(
            residuals
                .windows(2)
                .map(|window| (window[1] - window[0]).powi(2))
                .sum::<f64>()
                / durbin_watson_denominator,
        )
    } else {
        None
    };
    let lag1_residual_autocorrelation = if n > 1 && rss > f64::EPSILON {
        Some(
            residuals
                .windows(2)
                .map(|window| window[0] * window[1])
                .sum::<f64>()
                / rss,
        )
    } else {
        None
    };
    let selected_criterion_value = match criterion {
        SelectionCriterion::Aic => aic,
        SelectionCriterion::Bic => bic,
    };
    let mut warnings = Vec::new();
    if aicc.is_none() {
        warnings.push(TransientWarning::new(
            TransientWarningKind::AiccUnavailable,
            "AICc is unavailable because the observation count is not greater than k + 1",
        ));
    }

    Ok((
        TransientFitStatistics {
            rmse_v: Some(rmse),
            mae_v: Some(mae),
            r_squared,
            adjusted_r_squared,
            rss: Some(rss),
            aic: Some(aic),
            aicc,
            bic: Some(bic),
            durbin_watson,
            lag1_residual_autocorrelation,
            maximum_absolute_residual_v: Some(maximum_absolute_residual),
            criterion_delta: None,
            model_weight: None,
            convergence_status: "converged".to_string(),
            optimizer_termination_reason: None,
            covariance_condition_number: None,
        },
        warnings,
    ))
    .map(|(mut statistics, warnings)| {
        statistics.criterion_delta = Some(selected_criterion_value);
        (statistics, warnings)
    })
}

pub fn derived_features(
    model: TransientModelKind,
    parameters: &[f64],
    segment: &SegmentSummary,
    baseline: &BaselineResult,
    response_offset: f64,
    config: &ResolvedTransientConfig,
) -> Result<(TransientFeatures, Vec<TransientWarning>), PotentiometryError> {
    let initial_measured = segment
        .fitted_time_local
        .first()
        .and_then(|_| segment.raw_potential_v.iter().flatten().next().copied());
    let equilibrium = parameters
        .first()
        .copied()
        .map(|value| value + response_offset);
    let initial_model = evaluate(model, parameters, 0.0)?.total;
    let initial_response_rate = initial_response_rate(model, parameters);
    let total_amplitude =
        equilibrium.map(|equilibrium| equilibrium - (initial_model + response_offset));
    let mut features = TransientFeatures {
        event_timestamp: segment
            .segment_start
            .map(|start| start - segment.local_start.unwrap_or(0.0)),
        segment_start: segment.segment_start,
        segment_end: segment.segment_end,
        raw_observations: segment.raw_observations,
        finite_fitted_observations: segment.finite_fitted_observations,
        missing_fraction: segment.missing_fraction,
        baseline_estimate_v: baseline.estimate_v,
        initial_measured_potential_v: initial_measured,
        fitted_equilibrium_potential_v: equilibrium,
        total_response_amplitude_v: total_amplitude,
        fast_amplitude_v: None,
        slow_amplitude_v: None,
        tau_fast_s: None,
        tau_slow_s: None,
        stretched_beta: None,
        drift_rate_v_per_s: None,
        initial_response_rate_v_per_s: initial_response_rate,
        time_to_63_2_percent_s: None,
        time_to_90_percent_s: None,
        time_to_95_percent_s: None,
    };

    match model {
        TransientModelKind::Single => {
            features.fast_amplitude_v = parameters.get(1).copied();
            features.tau_fast_s = parameters.get(2).copied();
        }
        TransientModelKind::Double | TransientModelKind::DoubleDrift => {
            features.fast_amplitude_v = parameters.get(1).copied();
            features.slow_amplitude_v = parameters.get(2).copied();
            features.tau_fast_s = parameters.get(3).copied();
            features.tau_slow_s = parameters.get(4).copied();
            if model == TransientModelKind::DoubleDrift {
                features.drift_rate_v_per_s = parameters.get(5).copied();
            }
        }
        TransientModelKind::Stretched => {
            features.fast_amplitude_v = parameters.get(1).copied();
            features.tau_fast_s = parameters.get(2).copied();
            features.stretched_beta = parameters.get(3).copied();
        }
    }

    let mut warnings = Vec::new();
    let window = segment.finite_duration_s.unwrap_or(0.0);
    let largest_tau = features.tau_slow_s.or(features.tau_fast_s).unwrap_or(0.0);
    if largest_tau > 0.0
        && window > 0.0
        && largest_tau / window > config.validation.maximum_tau_to_window_ratio
    {
        warnings.push(TransientWarning::new(
            TransientWarningKind::LongTimeConstant,
            "a fitted timescale is close to or longer than the fitted observation window",
        ));
    }
    if let (Some(fast), Some(slow)) = (features.tau_fast_s, features.tau_slow_s) {
        if slow / fast < config.validation.minimum_tau_ratio {
            warnings.push(TransientWarning::new(
                TransientWarningKind::PoorTauSeparation,
                format!(
                    "fast and slow fitted timescales have ratio {:.3}, below configured {:.3}",
                    slow / fast,
                    config.validation.minimum_tau_ratio
                ),
            ));
        }
        let total = features.fast_amplitude_v.unwrap_or(0.0).abs()
            + features.slow_amplitude_v.unwrap_or(0.0).abs();
        if total > 0.0
            && (features.fast_amplitude_v.unwrap_or(0.0).abs() / total
                < config.validation.negligible_amplitude_fraction
                || features.slow_amplitude_v.unwrap_or(0.0).abs() / total
                    < config.validation.negligible_amplitude_fraction)
        {
            warnings.push(TransientWarning::new(
                TransientWarningKind::NegligibleAmplitude,
                "one fitted exponential amplitude is negligible relative to the combined amplitude",
            ));
        }
    }

    features.time_to_63_2_percent_s = time_to_fraction(model, parameters, 0.632, window);
    features.time_to_90_percent_s = time_to_fraction(model, parameters, 0.90, window);
    features.time_to_95_percent_s = time_to_fraction(model, parameters, 0.95, window);
    Ok((features, warnings))
}

fn time_to_fraction(
    model: TransientModelKind,
    parameters: &[f64],
    fraction: f64,
    window: f64,
) -> Option<f64> {
    let initial = evaluate(model, parameters, 0.0).ok()?.total;
    let equilibrium = parameters.first().copied()?;
    let initial_distance = (initial - equilibrium).abs();
    if initial_distance <= f64::EPSILON {
        return Some(0.0);
    }
    let upper = match model {
        TransientModelKind::Single | TransientModelKind::Stretched => {
            parameters
                .get(2)
                .copied()
                .unwrap_or(window)
                .max(window)
                .max(1.0)
                * 20.0
        }
        TransientModelKind::Double | TransientModelKind::DoubleDrift => {
            parameters
                .get(4)
                .copied()
                .unwrap_or(window)
                .max(window)
                .max(1.0)
                * 20.0
        }
    };
    let target = initial_distance * (1.0 - fraction);
    let steps = 2000usize;
    let mut previous_time = 0.0;
    let mut previous_distance = initial_distance;
    for step in 1..=steps {
        let time = upper * step as f64 / steps as f64;
        let distance = (evaluate(model, parameters, time).ok()?.total - equilibrium).abs();
        if distance <= target && previous_distance > target {
            let ratio = (previous_distance - target)
                / (previous_distance - distance).max(f64::MIN_POSITIVE);
            return Some(previous_time + (time - previous_time) * ratio);
        }
        if distance <= target {
            return Some(time);
        }
        previous_time = time;
        previous_distance = distance;
    }
    None
}

pub fn finite_model_components(
    model: TransientModelKind,
    parameters: &[f64],
    times: &[f64],
) -> Result<Vec<ModelComponents>, PotentiometryError> {
    times
        .iter()
        .map(|time| evaluate(model, parameters, *time))
        .collect()
}
