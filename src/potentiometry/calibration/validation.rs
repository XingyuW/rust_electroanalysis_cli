//! Ordered cross-validation for calibration models.

use super::error::CalibrationError;
use super::fitting::fit_model;
use super::nicolsky_eisenman::{InterferentModelInput, evaluate_potential};
use crate::calibration_config::ResolvedCalibrationConfig;
use crate::results::calibration::{
    CalibrationModelKind, CalibrationObservation, CalibrationValidationResult, CalibrationWarning,
    CalibrationWarningKind, CrossValidationMode, ValidationFoldResult, ValidationPredictionPoint,
};

pub fn cross_validate(
    observations: &[CalibrationObservation],
    config: &ResolvedCalibrationConfig,
    model: CalibrationModelKind,
) -> Result<CalibrationValidationResult, CalibrationError> {
    if observations.len() < 3 {
        return Ok(CalibrationValidationResult {
            mode: config.validation.mode,
            warnings: vec![CalibrationWarning::new(
                CalibrationWarningKind::PoorCrossValidation,
                "cross-validation requires at least three observations",
            )],
            ..CalibrationValidationResult::default()
        });
    }
    if !matches!(
        config.validation.mode,
        CrossValidationMode::LeaveOneOut | CrossValidationMode::LeaveOneConcentrationLevelOut
    ) {
        return Ok(CalibrationValidationResult {
            mode: config.validation.mode,
            warnings: vec![CalibrationWarning::new(
                CalibrationWarningKind::PoorCrossValidation,
                "the requested validation mode is reserved for a future phase; no random split was performed",
            )],
            ..CalibrationValidationResult::default()
        });
    }
    let groups = concentration_groups(observations);
    let folds = if config.validation.mode == CrossValidationMode::LeaveOneOut {
        observations
            .iter()
            .enumerate()
            .map(|(index, _)| vec![index])
            .collect::<Vec<_>>()
    } else {
        groups
    };
    let mut fold_results = Vec::new();
    let mut all_potential_errors = Vec::new();
    let mut all_log_errors = Vec::new();
    let mut failed_predictions = 0usize;
    let mut extrapolation_count = 0usize;
    let mut validation_predictions = Vec::new();
    for (fold_index, held_out) in folds.iter().enumerate() {
        let train = observations
            .iter()
            .enumerate()
            .filter(|(index, _)| !held_out.contains(index))
            .map(|(_, observation)| observation.clone())
            .collect::<Vec<_>>();
        let held = held_out
            .iter()
            .filter_map(|index| observations.get(*index))
            .collect::<Vec<_>>();
        let fit = match fit_model(&train, config, model) {
            Ok(fit) => fit,
            Err(_) => {
                failed_predictions += held.len();
                fold_results.push(ValidationFoldResult {
                    fold_id: format!("fold-{fold_index}"),
                    held_out_observations: held.len(),
                    failed_predictions: held.len(),
                    ..ValidationFoldResult::default()
                });
                continue;
            }
        };
        let domain_min = train
            .iter()
            .filter_map(CalibrationObservation::log10_activity)
            .reduce(f64::min);
        let domain_max = train
            .iter()
            .filter_map(CalibrationObservation::log10_activity)
            .reduce(f64::max);
        let mut fold_potential_errors = Vec::new();
        let mut fold_log_errors = Vec::new();
        let mut fold_failures = 0usize;
        let mut fold_extrapolations = 0usize;
        for row in held {
            let Some(log_activity) = row.log10_activity() else {
                fold_failures += 1;
                continue;
            };
            if domain_min.is_some_and(|minimum| log_activity < minimum)
                || domain_max.is_some_and(|maximum| log_activity > maximum)
            {
                fold_extrapolations += 1;
            }
            match predict_from_fit(&fit, row, config) {
                Ok((potential, predicted_log)) => {
                    fold_potential_errors.push(potential - row.potential_v);
                    if let Some(predicted_log) = predicted_log {
                        fold_log_errors.push(predicted_log - log_activity);
                    }
                    validation_predictions.push(ValidationPredictionPoint {
                        observation_id: row.observation_id.clone(),
                        observed_potential_v: row.potential_v,
                        predicted_potential_v: Some(potential),
                        observed_log10_activity: Some(log_activity),
                        predicted_log10_activity: predicted_log,
                        extrapolated: domain_min.is_some_and(|minimum| log_activity < minimum)
                            || domain_max.is_some_and(|maximum| log_activity > maximum),
                    });
                }
                Err(_) => {
                    fold_failures += 1;
                    validation_predictions.push(ValidationPredictionPoint {
                        observation_id: row.observation_id.clone(),
                        observed_potential_v: row.potential_v,
                        predicted_potential_v: None,
                        observed_log10_activity: Some(log_activity),
                        predicted_log10_activity: None,
                        extrapolated: domain_min.is_some_and(|minimum| log_activity < minimum)
                            || domain_max.is_some_and(|maximum| log_activity > maximum),
                    });
                }
            }
        }
        failed_predictions += fold_failures;
        extrapolation_count += fold_extrapolations;
        all_potential_errors.extend(fold_potential_errors.iter().copied());
        all_log_errors.extend(fold_log_errors.iter().copied());
        fold_results.push(ValidationFoldResult {
            fold_id: format!("fold-{fold_index}"),
            held_out_observations: held_out.len(),
            failed_predictions: fold_failures,
            extrapolation_count: fold_extrapolations,
            rmse_potential_v: rms(&fold_potential_errors),
            mae_potential_v: mae(&fold_potential_errors),
            rmse_log10_activity: rms(&fold_log_errors),
            mae_log10_activity: mae(&fold_log_errors),
            prediction_bias_v: mean(&fold_potential_errors),
            coverage: None,
        });
    }
    let mut warnings = Vec::new();
    if failed_predictions > 0 {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::PoorCrossValidation,
            format!("{failed_predictions} cross-validation predictions failed"),
        ));
    }
    Ok(CalibrationValidationResult {
        mode: config.validation.mode,
        folds: fold_results,
        rmse_potential_v: rms(&all_potential_errors),
        mae_potential_v: mae(&all_potential_errors),
        rmse_log10_activity: rms(&all_log_errors),
        mae_log10_activity: mae(&all_log_errors),
        concentration_relative_error: None,
        prediction_bias_v: mean(&all_potential_errors),
        interval_coverage: None,
        failed_predictions,
        extrapolation_count,
        predictions: validation_predictions,
        warnings,
    })
}

fn predict_from_fit(
    fit: &crate::results::calibration::CalibrationModelResult,
    row: &CalibrationObservation,
    config: &ResolvedCalibrationConfig,
) -> Result<(f64, Option<f64>), CalibrationError> {
    let e0 = fit
        .parameters
        .iter()
        .find(|parameter| parameter.name == "E0")
        .map(|parameter| parameter.value)
        .ok_or_else(|| {
            CalibrationError::NotIdentifiable("cross-validation fit lacks E0".to_string())
        })?;
    match fit.model_kind {
        CalibrationModelKind::Nernst | CalibrationModelKind::ConductivityEmpirical => {
            let slope = fit.fitted_slope_v_per_decade.ok_or_else(|| {
                CalibrationError::NotIdentifiable("cross-validation fit lacks slope".to_string())
            })?;
            let log_activity = row.log10_activity().ok_or_else(|| {
                CalibrationError::InvalidObservation("held-out activity is unavailable".to_string())
            })?;
            Ok((
                e0 + slope * log_activity,
                Some((row.potential_v - e0) / slope),
            ))
        }
        CalibrationModelKind::NicolskyEisenman => {
            let temperature = row.temperature_k.unwrap_or(298.15);
            let interferents = fit
                .selectivity_coefficients
                .iter()
                .map(|coefficient| {
                    Ok(InterferentModelInput {
                        name: coefficient.interferent.clone(),
                        charge: config
                            .nicolsky_eisenman
                            .interferents
                            .iter()
                            .find(|item| item.name == coefficient.interferent)
                            .map(|item| item.charge)
                            .unwrap_or(1),
                        activity: *row
                            .interferent_activities
                            .get(&coefficient.interferent)
                            .ok_or_else(|| {
                                CalibrationError::InvalidObservation(
                                    "held-out interferent activity is unavailable".to_string(),
                                )
                            })?,
                        selectivity_coefficient: coefficient.value,
                    })
                })
                .collect::<Result<Vec<_>, CalibrationError>>()?;
            Ok((
                evaluate_potential(
                    e0,
                    row.activity.unwrap_or(0.0),
                    row.ion_charge,
                    temperature,
                    &interferents,
                )?,
                None,
            ))
        }
    }
}

fn concentration_groups(observations: &[CalibrationObservation]) -> Vec<Vec<usize>> {
    let mut groups: Vec<(f64, Vec<usize>)> = Vec::new();
    for (index, observation) in observations.iter().enumerate() {
        let Some(value) = observation.log10_activity() else {
            continue;
        };
        if let Some((_, indices)) = groups
            .iter_mut()
            .find(|(level, _)| (*level - value).abs() <= 1e-10)
        {
            indices.push(index);
        } else {
            groups.push((value, vec![index]));
        }
    }
    groups.into_iter().map(|(_, indices)| indices).collect()
}

fn rms(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| {
        (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
    })
}
fn mae(values: &[f64]) -> Option<f64> {
    (!values.is_empty())
        .then(|| values.iter().map(|value| value.abs()).sum::<f64>() / values.len() as f64)
}
fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}
