//! Equilibrium potentiometric calibration and prediction.

pub mod activity;
pub mod environment;
pub mod error;
pub mod fitting;
pub mod ionic_strength;
pub mod nernst;
pub mod nicolsky_eisenman;
pub mod observations;
pub mod prediction;
pub mod uncertainty;
pub mod validation;

pub use error::CalibrationError;
pub use observations::extract_observations;

use crate::calibration_config::ResolvedCalibrationConfig;
use crate::results::calibration::{
    CalibrationAnalysisReport, CalibrationFitStatus, CalibrationModelKind,
    CalibrationObservationSet, CalibrationSelectionCriterion, CalibrationValidationResult,
    CalibrationWarning, CalibrationWarningKind, HysteresisResult, StoredCalibrationModel,
};

/// Fit all configured calibration candidates and select a scientifically valid
/// candidate using the configured information criterion.
pub fn fit_calibration(
    observation_set: &CalibrationObservationSet,
    config: &ResolvedCalibrationConfig,
) -> Result<CalibrationAnalysisReport, CalibrationError> {
    let observations = observation_set
        .observations
        .iter()
        .filter(|observation| match config.selection.branch {
            crate::results::calibration::CalibrationBranch::Mixed
            | crate::results::calibration::CalibrationBranch::Unknown => true,
            branch => observation.branch == branch,
        })
        .cloned()
        .collect::<Vec<_>>();
    if observations.is_empty() {
        return Err(CalibrationError::NoObservations);
    }
    let analyte = if config.analyte.name == "auto" {
        observations
            .first()
            .map(|observation| observation.analyte.clone())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        config.analyte.name.clone()
    };
    let mut candidates = Vec::new();
    for model in config.models.enabled.iter().copied() {
        match fitting::fit_model(&observations, config, model) {
            Ok(mut result) => {
                if result.statistics.observations != observations.len() {
                    result.warnings.push(CalibrationWarning::new(
                        CalibrationWarningKind::ComparisonDifferentObservationSets,
                        "candidate model was fitted to a different finite observation set",
                    ));
                }
                candidates.push(result);
            }
            Err(error) => candidates.push(failed_model(
                model,
                config.activity.model,
                error.to_string(),
            )),
        }
    }
    let selected_index = select_candidate(
        &mut candidates,
        config.selection.criterion,
        &observations,
        config,
    )?;
    if selected_index.is_none() {
        return Err(CalibrationError::AllModelsFailed);
    }
    if let Some(index) = selected_index {
        uncertainty::bootstrap_model(
            &observations,
            config,
            candidates[index].model_kind,
            &mut candidates[index],
        )?;
    }
    let selected_model = selected_index.map(|index| candidates[index].model_kind);
    let hysteresis = config
        .hysteresis
        .analyze
        .then(|| hysteresis(&observations, config));
    let validation = selected_model
        .and_then(|model| validation::cross_validate(&observations, config, model).ok());
    let mut warnings = observation_set.warnings.clone();
    if let Some(hysteresis) = &hysteresis {
        warnings.extend(hysteresis.warnings.clone());
    }
    if observations
        .iter()
        .filter_map(|observation| observation.log10_activity())
        .map(|value| format!("{value:.10}"))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        < 3
    {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::InsufficientConcentrationLevels,
            "fewer than three distinct activity levels are available",
        ));
    }
    if let Some(validation) = &validation {
        warnings.extend(validation.warnings.clone());
    }
    Ok(CalibrationAnalysisReport {
        schema_version: 1,
        calibration_id: format!("{analyte}-calibration"),
        analyte,
        ion_charge: config.analyte.charge,
        source_experiments: observations
            .iter()
            .map(|observation| observation.experiment_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect(),
        observation_summary: summary(&observations),
        configuration: config.clone(),
        candidate_models: candidates,
        selected_model,
        hysteresis,
        validation,
        provenance: observation_set.provenance.clone(),
        warnings,
    })
}

pub fn stored_model_from_report(
    report: &CalibrationAnalysisReport,
) -> Result<StoredCalibrationModel, CalibrationError> {
    let model_kind = report
        .selected_model
        .ok_or(CalibrationError::AllModelsFailed)?;
    let result = report
        .candidate_models
        .iter()
        .find(|candidate| {
            candidate.model_kind == model_kind
                && candidate.status == CalibrationFitStatus::Converged
        })
        .ok_or(CalibrationError::AllModelsFailed)?;
    Ok(StoredCalibrationModel {
        schema_version: 1,
        analyte: report.analyte.clone(),
        ion_charge: report.ion_charge,
        model_kind,
        activity_model: result.activity_model,
        temperature_mode: report.configuration.temperature.mode,
        slope_mode: report.configuration.nernst.slope_mode,
        response_sign: report.configuration.nernst.response_sign,
        parameters: result.parameters.clone(),
        selectivity_coefficients: result.selectivity_coefficients.clone(),
        valid_domain: result.valid_domain.clone(),
        training_statistics: result.statistics.clone(),
        configuration: report.configuration.clone(),
        provenance: report.provenance.clone(),
    })
}

pub fn validate_stored_model(
    model: &StoredCalibrationModel,
    observations: &[crate::results::calibration::CalibrationObservation],
) -> Result<CalibrationValidationResult, CalibrationError> {
    validation::cross_validate(observations, &model.configuration, model.model_kind)
}

fn select_candidate(
    candidates: &mut [crate::results::calibration::CalibrationModelResult],
    criterion: CalibrationSelectionCriterion,
    observations: &[crate::results::calibration::CalibrationObservation],
    config: &ResolvedCalibrationConfig,
) -> Result<Option<usize>, CalibrationError> {
    let valid = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            candidate.status == CalibrationFitStatus::Converged
                && candidate.statistics.observations > 0
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if valid.is_empty() {
        return Ok(None);
    }
    let scores = valid
        .iter()
        .map(|index| match criterion {
            _ if candidates[*index].status != CalibrationFitStatus::Converged => None,
            CalibrationSelectionCriterion::Aic => candidates[*index].statistics.aic,
            CalibrationSelectionCriterion::Aicc => candidates[*index]
                .statistics
                .aicc
                .or(candidates[*index].statistics.aic),
            CalibrationSelectionCriterion::Bic => candidates[*index].statistics.bic,
            CalibrationSelectionCriterion::CrossValidation => {
                validation::cross_validate(observations, config, candidates[*index].model_kind)
                    .ok()
                    .and_then(|result| result.rmse_potential_v)
            }
        })
        .collect::<Vec<_>>();
    let Some(best_score) = scores.iter().filter_map(|value| *value).reduce(f64::min) else {
        return Ok(None);
    };
    let mut selected = None;
    let mut weight_denominator = 0.0;
    for (index, score) in valid.iter().zip(scores.iter()) {
        if let Some(score) = score {
            let delta = *score - best_score;
            candidates[*index].statistics.criterion_delta = Some(delta);
            let weight = (-0.5 * delta).exp();
            weight_denominator += weight;
            if (*score - best_score).abs() < 1e-12 {
                selected = Some(*index);
            }
        }
    }
    for candidate in candidates.iter_mut() {
        if let Some(delta) = candidate.statistics.criterion_delta {
            candidate.statistics.model_weight =
                Some((-0.5 * delta).exp() / weight_denominator.max(f64::EPSILON));
        }
    }
    Ok(selected)
}

fn failed_model(
    model: CalibrationModelKind,
    activity_model: crate::results::calibration::ActivityModelKind,
    reason: String,
) -> crate::results::calibration::CalibrationModelResult {
    crate::results::calibration::CalibrationModelResult {
        model_kind: model,
        status: CalibrationFitStatus::Failed,
        activity_model,
        parameters: Vec::new(),
        selectivity_coefficients: Vec::new(),
        equation: String::new(),
        theoretical_slope_v_per_decade: None,
        fitted_slope_v_per_decade: None,
        slope_efficiency: None,
        statistics: crate::results::calibration::CalibrationFitStatistics {
            convergence_reason: Some(reason.clone()),
            ..Default::default()
        },
        predicted_potential_v: Vec::new(),
        residuals_v: Vec::new(),
        standardized_residuals: Vec::new(),
        confidence_intervals: Vec::new(),
        valid_domain: Default::default(),
        warnings: vec![CalibrationWarning::new(
            CalibrationWarningKind::NicolskyNonIdentifiable,
            reason,
        )],
    }
}

fn summary(
    observations: &[crate::results::calibration::CalibrationObservation],
) -> crate::results::calibration::CalibrationObservationSummary {
    let potentials = observations
        .iter()
        .map(|observation| observation.potential_v)
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    let levels = observations
        .iter()
        .filter_map(|observation| observation.log10_activity())
        .map(|value| format!("{value:.10}"))
        .collect::<std::collections::BTreeSet<_>>();
    crate::results::calibration::CalibrationObservationSummary {
        total_observations: observations.len(),
        ascending_observations: observations
            .iter()
            .filter(|observation| {
                observation.branch == crate::results::calibration::CalibrationBranch::Ascending
            })
            .count(),
        descending_observations: observations
            .iter()
            .filter(|observation| {
                observation.branch == crate::results::calibration::CalibrationBranch::Descending
            })
            .count(),
        unknown_branch_observations: observations
            .iter()
            .filter(|observation| {
                observation.branch == crate::results::calibration::CalibrationBranch::Unknown
            })
            .count(),
        concentration_levels: levels.len(),
        experiments: observations
            .iter()
            .map(|observation| observation.experiment_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        finite_activities: observations
            .iter()
            .filter(|observation| observation.log10_activity().is_some())
            .count(),
        potential_range_v: min_max(&potentials),
    }
}

fn hysteresis(
    observations: &[crate::results::calibration::CalibrationObservation],
    config: &ResolvedCalibrationConfig,
) -> HysteresisResult {
    let ascending = observations
        .iter()
        .filter(|observation| {
            observation.branch == crate::results::calibration::CalibrationBranch::Ascending
        })
        .collect::<Vec<_>>();
    let descending = observations
        .iter()
        .filter(|observation| {
            observation.branch == crate::results::calibration::CalibrationBranch::Descending
        })
        .collect::<Vec<_>>();
    let mut pairs = Vec::new();
    let mut used_ascending = std::collections::BTreeSet::new();
    for down in descending {
        let Some(down_activity) = down.log10_activity() else {
            continue;
        };
        let best = ascending
            .iter()
            .enumerate()
            .filter(|(index, _)| !used_ascending.contains(index))
            .filter_map(|(index, up)| {
                let up_activity = up.log10_activity()?;
                let distance = (up_activity - down_activity).abs();
                (distance <= config.hysteresis.log_activity_matching_tolerance)
                    .then_some((index, distance, up))
            })
            .min_by(|left, right| left.1.total_cmp(&right.1));
        if let Some((index, _, up)) = best {
            used_ascending.insert(index);
            pairs.push((down_activity, down.potential_v - up.potential_v));
        }
    }
    let values = pairs.iter().map(|(_, value)| *value).collect::<Vec<_>>();
    let mut result = HysteresisResult {
        matching_tolerance_log10_activity: config.hysteresis.log_activity_matching_tolerance,
        paired_observations: pairs.len(),
        mean_hysteresis_v: mean(&values),
        median_hysteresis_v: median(&values),
        maximum_absolute_hysteresis_v: values.iter().map(|value| value.abs()).reduce(f64::max),
        activity_specific_hysteresis: pairs,
        warnings: Vec::new(),
    };
    if result
        .maximum_absolute_hysteresis_v
        .is_some_and(|value| value > config.hysteresis.warning_threshold_v)
    {
        result.warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::HighHysteresis,
            "ascending and descending calibration branches differ beyond the configured hysteresis threshold",
        ));
    }
    result
}

fn min_max(values: &[f64]) -> Option<(f64, f64)> {
    values
        .iter()
        .copied()
        .reduce(f64::min)
        .zip(values.iter().copied().reduce(f64::max))
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    Some(if values.len().is_multiple_of(2) {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    } else {
        values[values.len() / 2]
    })
}
