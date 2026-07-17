//! Reproducible residual bootstrap for calibration parameters.

use super::error::CalibrationError;
use super::fitting::fit_model;
use crate::calibration_config::ResolvedCalibrationConfig;
use crate::results::calibration::{
    CalibrationConfidenceInterval, CalibrationModelKind, CalibrationModelResult,
    CalibrationObservation, CalibrationWarning, CalibrationWarningKind,
};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub fn bootstrap_model(
    observations: &[CalibrationObservation],
    config: &ResolvedCalibrationConfig,
    model: CalibrationModelKind,
    fit: &mut CalibrationModelResult,
) -> Result<(), CalibrationError> {
    let iterations = config.uncertainty.bootstrap_iterations;
    if iterations == 0 {
        fit.warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::BootstrapUnavailable,
            "bootstrap_iterations is zero; confidence intervals are unavailable",
        ));
        return Ok(());
    }
    if fit.predicted_potential_v.len() != observations.len() {
        fit.warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::BootstrapUnavailable,
            "bootstrap inputs do not align with the fitted observation set",
        ));
        return Ok(());
    }
    let residuals = fit.residuals_v.clone();
    if residuals.is_empty() {
        fit.warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::BootstrapUnavailable,
            "bootstrap residuals are unavailable",
        ));
        return Ok(());
    }
    let mut rng = StdRng::seed_from_u64(config.uncertainty.seed);
    let mut samples = vec![Vec::<f64>::new(); fit.parameters.len()];
    let mut successful = 0usize;
    for _ in 0..iterations {
        let mut bootstrap_observations = observations.to_vec();
        for (index, observation) in bootstrap_observations.iter_mut().enumerate() {
            let residual = residuals
                .get(rng.gen_range(0..residuals.len()))
                .copied()
                .unwrap_or(0.0);
            let prediction = fit
                .predicted_potential_v
                .get(index)
                .copied()
                .unwrap_or(observation.potential_v);
            observation.potential_v = prediction + residual;
        }
        if let Ok(result) = fit_model(&bootstrap_observations, config, model)
            && result.status == crate::results::calibration::CalibrationFitStatus::Converged
            && result
                .parameters
                .iter()
                .all(|parameter| parameter.value.is_finite())
        {
            successful += 1;
            for (index, parameter) in result.parameters.iter().enumerate() {
                if let Some(values) = samples.get_mut(index) {
                    values.push(parameter.value);
                }
            }
        }
    }
    let failed = iterations.saturating_sub(successful);
    let success_fraction = successful as f64 / iterations as f64;
    if success_fraction < config.uncertainty.minimum_success_fraction {
        fit.confidence_intervals.clear();
        fit.warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::BootstrapUnavailable,
            format!("only {successful}/{iterations} bootstrap iterations succeeded; confidence intervals are suppressed"),
        ));
        return Ok(());
    }
    let alpha = (1.0 - config.uncertainty.confidence_level) / 2.0;
    fit.confidence_intervals = fit
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| {
            let mut values = samples.get(index)?.clone();
            if values.is_empty() {
                return None;
            }
            values.sort_by(f64::total_cmp);
            Some(CalibrationConfidenceInterval {
                parameter: parameter.name.clone(),
                unit: parameter.unit.clone(),
                lower: Some(percentile(&values, alpha)),
                upper: Some(percentile(&values, 1.0 - alpha)),
                confidence_level: config.uncertainty.confidence_level,
                successful_iterations: successful,
                failed_iterations: failed,
            })
        })
        .collect();
    for coefficient in &mut fit.selectivity_coefficients {
        let name = format!("log_Kpot_{}", coefficient.interferent);
        if let Some(interval) = fit
            .confidence_intervals
            .iter()
            .find(|interval| interval.parameter == name)
        {
            coefficient.confidence_interval = interval
                .lower
                .zip(interval.upper)
                .map(|(lower, upper)| (lower.exp(), upper.exp()));
        }
    }
    Ok(())
}

fn percentile(values: &[f64], fraction: f64) -> f64 {
    if values.len() == 1 {
        return values[0];
    }
    let position = fraction.clamp(0.0, 1.0) * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        values[lower]
    } else {
        values[lower] + (values[upper] - values[lower]) * (position - lower as f64)
    }
}
