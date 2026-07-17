//! Extraction of calibration observations from experiments and transient fits.

use super::activity::evaluate_activity;
use super::environment::align_environmental_series;
use super::error::CalibrationError;
use super::ionic_strength::ionic_strength_from_config;
use crate::calibration_config::{ObservationExtractionConfig, ResolvedCalibrationConfig};
use crate::domain::{
    ElectrochemicalExperiment, ExperimentEvent, ExperimentEventKind, MeasurementChannel,
};
use crate::potentiometry::units::{Quantity, QuantityUnit};
use crate::results::calibration::{
    CalibrationBranch, CalibrationObservation, CalibrationObservationSet,
    CalibrationPotentialSource, CalibrationWarning, CalibrationWarningKind,
    EnvironmentalAlignmentRecord, SteadyStateSummary,
};
use crate::results::transient::{FitStatus, TransientAnalysisReport};
use std::collections::BTreeMap;

type PotentialSource = (
    f64,
    Option<f64>,
    CalibrationPotentialSource,
    Option<String>,
    Option<SteadyStateSummary>,
    Vec<String>,
);

/// Extract one calibration observation per concentration-step event.
pub fn extract_observations(
    experiment: &ElectrochemicalExperiment,
    channel_name: &str,
    transient_results: Option<&TransientAnalysisReport>,
    config: &ResolvedCalibrationConfig,
) -> Result<CalibrationObservationSet, CalibrationError> {
    let channel = experiment
        .measurement_data
        .channel(channel_name)
        .ok_or_else(|| {
            CalibrationError::InvalidObservation(format!(
                "selected channel '{channel_name}' does not exist"
            ))
        })?;
    let events = experiment
        .events
        .iter()
        .enumerate()
        .filter(|(_, event)| event.kind == ExperimentEventKind::ConcentrationStep)
        .collect::<Vec<_>>();
    if events.is_empty() {
        return Err(CalibrationError::NoObservations);
    }

    let mut observations = Vec::new();
    let mut warnings = Vec::new();
    for (eligible_index, (source_index, event)) in events.iter().enumerate() {
        let observation_id = format!("event-{eligible_index}");
        let analyte = event
            .analyte
            .clone()
            .or_else(|| experiment.sensor_metadata.analyte.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| config.analyte.name.clone());
        let Some(concentration) = event_concentration(event, &analyte, config, &mut warnings)
        else {
            warnings.push(CalibrationWarning::for_observation(
                CalibrationWarningKind::MissingConcentration,
                "concentration-step event has no usable concentration value and was excluded",
                &observation_id,
            ));
            continue;
        };
        let branch = branch_for_event(&events, eligible_index, &concentration, config);
        let preferred = match config.observation_extraction.preferred_source {
            CalibrationPotentialSource::TransientEquilibrium => {
                transient_potential(transient_results, eligible_index, config, &mut warnings)
            }
            CalibrationPotentialSource::SteadyStateWindowMean
            | CalibrationPotentialSource::SteadyStateWindowMedian => steady_state_potential(
                experiment,
                channel,
                event,
                config.observation_extraction.preferred_source,
                &config.observation_extraction,
                &mut warnings,
            )
            .ok(),
            CalibrationPotentialSource::ExplicitObservation => None,
        };
        let potential = preferred.or_else(|| {
            config
                .observation_extraction
                .fallback_source
                .and_then(|source| {
                    if source == CalibrationPotentialSource::TransientEquilibrium {
                        transient_potential(
                            transient_results,
                            eligible_index,
                            config,
                            &mut warnings,
                        )
                    } else {
                        steady_state_potential(
                            experiment,
                            channel,
                            event,
                            source,
                            &config.observation_extraction,
                            &mut warnings,
                        )
                        .ok()
                    }
                })
        });
        let Some((potential_v, standard_error_v, source, status, steady_state, source_warnings)) =
            potential
        else {
            warnings.push(CalibrationWarning::for_observation(
                CalibrationWarningKind::TransientEquilibriumUnavailable,
                format!("no valid potential source was available for concentration-step event {eligible_index}"),
                format!("event-{eligible_index}"),
            ));
            continue;
        };

        let mut environmental_alignment = Vec::new();
        let temperature_k = environmental_temperature(
            experiment,
            event.timestamp,
            config.temperature.environmental_series.as_deref(),
            config.temperature.alignment,
            config.temperature.maximum_gap_s,
            &mut warnings,
            &mut environmental_alignment,
        )
        .or_else(|| {
            event_quantity(event, "temperature", "temperature_celsius").and_then(|quantity| {
                match quantity.to_temperature_k() {
                    Ok(value) => Some(value),
                    Err(error) => {
                        warnings.push(CalibrationWarning::for_observation(
                            CalibrationWarningKind::NonphysicalTemperature,
                            error.to_string(),
                            format!("event-{eligible_index}"),
                        ));
                        None
                    }
                }
            })
        })
        .or_else(|| {
            let default = Quantity::new(config.temperature.default_celsius, QuantityUnit::Celsius).ok()?.to_temperature_k().ok();
            default.map(|_| {
                warnings.push(CalibrationWarning::for_observation(
                    CalibrationWarningKind::MissingTemperature,
                    "using configured default temperature; no aligned temperature observation was available",
                    format!("event-{eligible_index}"),
                ));
                default.unwrap_or(298.15)
            });
            default
        });

        let ionic_strength = event_metadata_f64(event, &["ionic_strength_mol_l", "ionic_strength"])
            .or_else(|| {
                environmental_value(
                    experiment,
                    event.timestamp,
                    Some("ionic_strength"),
                    config.temperature.alignment,
                    config.temperature.maximum_gap_s,
                    &mut warnings,
                    &mut environmental_alignment,
                )
            })
            .or_else(|| {
                ionic_strength_from_config(&config.activity.solution_composition)
                    .ok()
                    .flatten()
            });
        if matches!(
            config.activity.model,
            crate::results::calibration::ActivityModelKind::Davies
                | crate::results::calibration::ActivityModelKind::ExtendedDebyeHuckel
        ) && ionic_strength.is_none()
        {
            warnings.push(CalibrationWarning::for_observation(
                CalibrationWarningKind::MissingIonicStrength,
                "the selected nonideal activity model has no ionic strength",
                format!("event-{eligible_index}"),
            ));
        }
        let conductivity = event_quantity(event, "conductivity", "conductivity").or_else(|| {
            environmental_conductivity(
                experiment,
                event.timestamp,
                config
                    .activity
                    .conductivity_empirical
                    .conductivity_series
                    .as_deref(),
                config.temperature.alignment,
                config.temperature.maximum_gap_s,
                &mut warnings,
                &mut environmental_alignment,
            )
        });
        let explicit_activity = event_metadata_f64(event, &["activity", "activity_after"]);
        let explicit_gamma = event_metadata_f64(event, &["activity_coefficient", "gamma"]);
        let activity = match evaluate_activity(
            Some(&concentration),
            config.analyte.molar_mass_g_per_mol,
            explicit_activity,
            explicit_gamma,
            config.analyte.charge,
            ionic_strength,
            conductivity
                .as_ref()
                .and_then(|value| value.to_conductivity_s_per_m().ok()),
            &config.activity,
        ) {
            Ok(activity) => activity,
            Err(error) => {
                warnings.push(CalibrationWarning::for_observation(
                    activity_warning_kind(&error),
                    error.to_string(),
                    &observation_id,
                ));
                continue;
            }
        };
        let mut source_warnings = source_warnings;
        source_warnings.extend(
            activity
                .warnings
                .iter()
                .map(|warning| warning.message.clone()),
        );
        let interferent_activities = interferent_activities(event, config);
        observations.push(CalibrationObservation {
            observation_id: format!("{}-event-{eligible_index}", experiment.experiment_id),
            experiment_id: experiment.experiment_id.clone(),
            event_index: Some(eligible_index),
            timestamp: Some(event.timestamp),
            analyte,
            ion_charge: config.analyte.charge,
            concentration: Some(concentration),
            molar_concentration_mol_l: quantity_to_molar(event, config),
            activity: Some(activity.activity),
            activity_coefficient: activity.activity_coefficient,
            potential_v,
            potential_standard_error_v: standard_error_v,
            temperature_k,
            ionic_strength_mol_l: ionic_strength,
            conductivity,
            interferent_activities,
            branch,
            source,
            source_fit_status: status,
            source_warnings,
            steady_state,
            environmental_alignment,
            metadata: event_metadata(event),
        });
        let _ = source_index;
    }
    if observations.is_empty() {
        return Err(CalibrationError::NoObservations);
    }
    Ok(CalibrationObservationSet {
        schema_version: 1,
        observations,
        provenance: experiment.provenance.clone(),
        warnings,
    })
}

fn event_concentration(
    event: &ExperimentEvent,
    _analyte: &str,
    config: &ResolvedCalibrationConfig,
    warnings: &mut Vec<CalibrationWarning>,
) -> Option<Quantity> {
    let Some(value) = event.value else {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::MissingConcentration,
            "concentration-step event has no concentration value",
        ));
        return None;
    };
    let Some(unit_name) = event.unit.as_deref().or_else(|| {
        event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("concentration_unit").map(String::as_str))
    }) else {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::UnknownUnit,
            "concentration-step event has no concentration unit",
        ));
        return None;
    };
    match unit_name.parse::<QuantityUnit>() {
        Ok(unit) => match Quantity::new(value, unit) {
            Ok(quantity) => {
                if quantity_to_molar_quantity(&quantity, config.analyte.molar_mass_g_per_mol)
                    .is_none()
                    && config.analyte.molar_mass_g_per_mol.is_none()
                {
                    warnings.push(CalibrationWarning::new(
                        CalibrationWarningKind::MissingMolarMass,
                        format!("molar mass is required for {unit_name}"),
                    ));
                }
                Some(quantity)
            }
            Err(error) => {
                warnings.push(CalibrationWarning::new(
                    CalibrationWarningKind::NonpositiveConcentration,
                    error.to_string(),
                ));
                None
            }
        },
        Err(error) => {
            warnings.push(CalibrationWarning::new(
                CalibrationWarningKind::UnknownUnit,
                error.to_string(),
            ));
            None
        }
    }
}

fn transient_potential(
    report: Option<&TransientAnalysisReport>,
    eligible_index: usize,
    config: &ResolvedCalibrationConfig,
    warnings: &mut Vec<CalibrationWarning>,
) -> Option<PotentialSource> {
    let event = report?
        .events
        .iter()
        .find(|event| event.event_index == eligible_index)?;
    let selected = event.selected_model?;
    let fit = event
        .candidate_fits
        .iter()
        .find(|fit| fit.model == selected)?;
    if fit.status != FitStatus::Converged {
        warnings.push(CalibrationWarning::for_observation(
            CalibrationWarningKind::TransientEquilibriumUnavailable,
            "selected transient fit was not converged",
            format!("event-{eligible_index}"),
        ));
        return None;
    }
    if !config.observation_extraction.allow_warning_fits && !fit.warnings.is_empty() {
        warnings.push(CalibrationWarning::for_observation(
            CalibrationWarningKind::TransientFitWarning,
            "selected transient fit carries warnings and was excluded by configuration",
            format!("event-{eligible_index}"),
        ));
        return None;
    }
    let raw_potential = fit.derived_features.fitted_equilibrium_potential_v?;
    // Phase 2 stores this explicitly in volts, independent of the source
    // channel's display unit (which may be mV).
    let potential = raw_potential.is_finite().then_some(raw_potential)?;
    let standard_error = fit
        .confidence_intervals
        .iter()
        .find(|interval| interval.name == "E_infinity")
        .and_then(|interval| interval.lower.zip(interval.upper))
        .and_then(|(lower, upper)| {
            (lower.is_finite() && upper.is_finite()).then_some((upper - lower).abs() / 3.92)
        })
        .filter(|value| value.is_finite() && *value > 0.0);
    let mut source_warnings = fit
        .warnings
        .iter()
        .map(|warning| warning.message.clone())
        .collect::<Vec<_>>();
    if !source_warnings.is_empty() {
        warnings.push(CalibrationWarning::for_observation(
            CalibrationWarningKind::TransientFitWarning,
            "transient equilibrium was accepted from a warning-bearing converged fit",
            format!("event-{eligible_index}"),
        ));
    }
    Some((
        potential,
        standard_error,
        CalibrationPotentialSource::TransientEquilibrium,
        Some(format!("{:?}", fit.status)),
        None,
        std::mem::take(&mut source_warnings),
    ))
}

fn steady_state_potential(
    experiment: &ElectrochemicalExperiment,
    channel: &MeasurementChannel,
    event: &ExperimentEvent,
    source: CalibrationPotentialSource,
    config: &ObservationExtractionConfig,
    warnings: &mut Vec<CalibrationWarning>,
) -> Result<PotentialSource, CalibrationError> {
    let start = event.timestamp + config.steady_state_start_s;
    let end = event.timestamp + config.steady_state_end_s;
    let mut points = experiment
        .measurement_data
        .time
        .iter()
        .copied()
        .zip(channel.values.iter().copied())
        .filter(|(time, _)| *time >= start && *time <= end)
        .collect::<Vec<_>>();
    if points.is_empty()
        || start < *experiment.measurement_data.time.first().unwrap_or(&start)
        || end > *experiment.measurement_data.time.last().unwrap_or(&end)
    {
        return Err(CalibrationError::InvalidSteadyStateWindow(
            "steady-state window lies outside the measurement range".to_string(),
        ));
    }
    let raw_count = points.len();
    let missing = points
        .iter()
        .filter(|(_, value)| value.is_none_or(|value| !value.is_finite()))
        .count();
    points.retain(|(_, value)| value.is_some_and(f64::is_finite));
    let missing_fraction = missing as f64 / raw_count.max(1) as f64;
    if points.len() < config.minimum_points {
        return Err(CalibrationError::InvalidSteadyStateWindow(format!(
            "only {} finite points are available",
            points.len()
        )));
    }
    if missing_fraction > config.maximum_missing_fraction {
        return Err(CalibrationError::InvalidSteadyStateWindow(format!(
            "missing fraction {missing_fraction:.3} exceeds configured maximum"
        )));
    }
    points.sort_by(|left, right| left.0.total_cmp(&right.0));
    if points.windows(2).any(|window| window[0].0 == window[1].0) {
        return Err(CalibrationError::InvalidSteadyStateWindow(
            "duplicate timestamps are not valid for steady-state extraction".to_string(),
        ));
    }
    let finite_points = points
        .iter()
        .filter_map(|(time, value)| value.map(|value| (*time, value)))
        .collect::<Vec<_>>();
    let values = finite_points
        .iter()
        .map(|(_, value)| *value)
        .collect::<Vec<_>>();
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let median = median(&values);
    let sd = standard_deviation(&values, mean);
    let se = (values.len() > 1).then_some(sd / (values.len() as f64).sqrt());
    let slope = linear_slope(&finite_points);
    let mut source_warnings = Vec::new();
    if slope.abs() > config.maximum_absolute_slope_v_per_s {
        source_warnings.push(format!(
            "steady-state slope {slope:.6} V/s exceeds configured stability threshold"
        ));
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::SteadyStateUnstable,
            source_warnings.last().cloned().unwrap_or_default(),
        ));
        return Err(CalibrationError::InvalidSteadyStateWindow(
            "steady-state window is unstable".to_string(),
        ));
    }
    let selected = match source {
        CalibrationPotentialSource::SteadyStateWindowMean => mean,
        CalibrationPotentialSource::SteadyStateWindowMedian => median,
        _ => mean,
    };
    let potential = Quantity::parse(selected, &channel.unit)?.to_potential_v()?;
    let standard_error_v = se.and_then(|value| {
        Quantity::parse(value, &channel.unit)
            .ok()?
            .to_potential_v()
            .ok()
    });
    Ok((
        potential,
        standard_error_v,
        source,
        Some("steady_state".to_string()),
        Some(SteadyStateSummary {
            window_start_s: config.steady_state_start_s,
            window_end_s: config.steady_state_end_s,
            finite_points: values.len(),
            mean_v: Quantity::parse(mean, &channel.unit)
                .ok()
                .and_then(|q| q.to_potential_v().ok()),
            median_v: Quantity::parse(median, &channel.unit)
                .ok()
                .and_then(|q| q.to_potential_v().ok()),
            standard_deviation_v: standard_error_v.map(|_| sd),
            standard_error_v,
            linear_slope_v_per_s: Some(slope),
            missing_fraction: Some(missing_fraction),
        }),
        source_warnings,
    ))
}

fn branch_for_event(
    events: &[(usize, &ExperimentEvent)],
    index: usize,
    current: &Quantity,
    config: &ResolvedCalibrationConfig,
) -> CalibrationBranch {
    let Some((_, previous)) = events.get(..index).and_then(|items| {
        items.iter().rev().find(|(_, event)| {
            event.analyte == events[index].1.analyte || events[index].1.analyte.is_none()
        })
    }) else {
        return CalibrationBranch::Unknown;
    };
    let Some(value) = previous.value else {
        return CalibrationBranch::Unknown;
    };
    let Some(unit) = previous
        .unit
        .as_deref()
        .and_then(|value| value.parse::<QuantityUnit>().ok())
    else {
        return CalibrationBranch::Unknown;
    };
    let Ok(previous_quantity) = Quantity::new(value, unit) else {
        return CalibrationBranch::Unknown;
    };
    let (Ok(previous_molar), Ok(current_molar)) = (
        previous_quantity.to_molar_concentration(config.analyte.molar_mass_g_per_mol),
        current.to_molar_concentration(config.analyte.molar_mass_g_per_mol),
    ) else {
        return CalibrationBranch::Unknown;
    };
    if (current_molar - previous_molar).abs() < f64::EPSILON {
        CalibrationBranch::Unknown
    } else if current_molar > previous_molar {
        CalibrationBranch::Ascending
    } else {
        CalibrationBranch::Descending
    }
}

fn quantity_to_molar(event: &ExperimentEvent, config: &ResolvedCalibrationConfig) -> Option<f64> {
    let quantity = event_concentration_without_warning(event)?;
    quantity
        .to_molar_concentration(config.analyte.molar_mass_g_per_mol)
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
}

fn quantity_to_molar_quantity(quantity: &Quantity, molar_mass: Option<f64>) -> Option<f64> {
    quantity
        .to_molar_concentration(molar_mass)
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
}

fn event_concentration_without_warning(event: &ExperimentEvent) -> Option<Quantity> {
    let unit = event
        .unit
        .as_deref()
        .or_else(|| {
            event
                .metadata
                .as_ref()?
                .get("concentration_unit")
                .map(String::as_str)
        })?
        .parse()
        .ok()?;
    Quantity::new(event.value?, unit).ok()
}

fn activity_warning_kind(error: &CalibrationError) -> CalibrationWarningKind {
    match error {
        CalibrationError::Unit(crate::potentiometry::units::UnitError::MissingMolarMass {
            ..
        }) => CalibrationWarningKind::MissingMolarMass,
        CalibrationError::Unit(crate::potentiometry::units::UnitError::Unknown(_)) => {
            CalibrationWarningKind::UnknownUnit
        }
        CalibrationError::ActivityModel(message) if message.contains("concentration") => {
            CalibrationWarningKind::NonpositiveConcentration
        }
        CalibrationError::ActivityModel(message) if message.contains("ionic strength") => {
            CalibrationWarningKind::MissingIonicStrength
        }
        CalibrationError::ActivityModel(message) if message.contains("conductivity") => {
            CalibrationWarningKind::MissingConductivity
        }
        _ => CalibrationWarningKind::NonpositiveActivity,
    }
}

fn event_quantity(event: &ExperimentEvent, key: &str, alternate: &str) -> Option<Quantity> {
    let metadata = event.metadata.as_ref()?;
    let value = metadata
        .get(key)
        .or_else(|| metadata.get(alternate))?
        .parse::<f64>()
        .ok()?;
    let unit = metadata
        .get(&format!("{key}_unit"))
        .or_else(|| metadata.get(&format!("{alternate}_unit")))?;
    Quantity::parse(value, unit).ok()
}

fn event_metadata_f64(event: &ExperimentEvent, keys: &[&str]) -> Option<f64> {
    let metadata = event.metadata.as_ref()?;
    keys.iter().find_map(|key| {
        metadata
            .get(*key)?
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
    })
}

fn event_metadata(event: &ExperimentEvent) -> BTreeMap<String, String> {
    event.metadata.clone().unwrap_or_default()
}

fn interferent_activities(
    event: &ExperimentEvent,
    config: &ResolvedCalibrationConfig,
) -> BTreeMap<String, f64> {
    let metadata = event.metadata.as_ref();
    config
        .nicolsky_eisenman
        .interferents
        .iter()
        .filter_map(|item| {
            let key = format!("activity_{}", item.name);
            let alternate = format!("interferent_activity_{}", item.name);
            metadata
                .and_then(|map| map.get(&key).or_else(|| map.get(&alternate)))
                .and_then(|value| value.parse().ok())
                .filter(|value: &f64| value.is_finite() && *value > 0.0)
                .map(|value| (item.name.clone(), value))
        })
        .collect()
}

fn environmental_value(
    experiment: &ElectrochemicalExperiment,
    timestamp: f64,
    name: Option<&str>,
    alignment: crate::results::calibration::EnvironmentalAlignment,
    maximum_gap_s: f64,
    warnings: &mut Vec<CalibrationWarning>,
    alignment_records: &mut Vec<EnvironmentalAlignmentRecord>,
) -> Option<f64> {
    let name = name?;
    let series = experiment
        .environmental_data
        .iter()
        .find(|series| series.name == name)?;
    match align_environmental_series(series, timestamp, alignment, maximum_gap_s, 1.0) {
        Ok(aligned) => {
            alignment_records.push(EnvironmentalAlignmentRecord {
                source_series: aligned.source_series.clone(),
                alignment: aligned.alignment,
                source_timestamps: aligned.source_timestamps.clone(),
                interpolated: aligned.interpolated,
                time_gap_s: aligned.time_gap_s,
            });
            Some(aligned.value)
        }
        Err(error) => {
            warnings.push(CalibrationWarning::new(
                CalibrationWarningKind::MissingIonicStrength,
                error.to_string(),
            ));
            None
        }
    }
}

fn environmental_temperature(
    experiment: &ElectrochemicalExperiment,
    timestamp: f64,
    name: Option<&str>,
    alignment: crate::results::calibration::EnvironmentalAlignment,
    maximum_gap_s: f64,
    warnings: &mut Vec<CalibrationWarning>,
    alignment_records: &mut Vec<EnvironmentalAlignmentRecord>,
) -> Option<f64> {
    let name = name?;
    let series = experiment
        .environmental_data
        .iter()
        .find(|series| series.name == name)?;
    let aligned = align_environmental_series(series, timestamp, alignment, maximum_gap_s, 1.0)
        .inspect_err(|error| {
            warnings.push(CalibrationWarning::new(
                CalibrationWarningKind::MissingTemperature,
                error.to_string(),
            ));
        })
        .ok()?;
    alignment_records.push(EnvironmentalAlignmentRecord {
        source_series: aligned.source_series.clone(),
        alignment: aligned.alignment,
        source_timestamps: aligned.source_timestamps.clone(),
        interpolated: aligned.interpolated,
        time_gap_s: aligned.time_gap_s,
    });
    Quantity::parse(aligned.value, &series.unit)
        .and_then(|quantity| quantity.to_temperature_k())
        .inspect_err(|error| {
            warnings.push(CalibrationWarning::new(
                CalibrationWarningKind::NonphysicalTemperature,
                error.to_string(),
            ));
        })
        .ok()
}

fn environmental_conductivity(
    experiment: &ElectrochemicalExperiment,
    timestamp: f64,
    name: Option<&str>,
    alignment: crate::results::calibration::EnvironmentalAlignment,
    maximum_gap_s: f64,
    warnings: &mut Vec<CalibrationWarning>,
    alignment_records: &mut Vec<EnvironmentalAlignmentRecord>,
) -> Option<Quantity> {
    let name = name?;
    let series = experiment
        .environmental_data
        .iter()
        .find(|series| series.name == name)?;
    let aligned = align_environmental_series(series, timestamp, alignment, maximum_gap_s, 1.0)
        .inspect_err(|error| {
            warnings.push(CalibrationWarning::new(
                CalibrationWarningKind::MissingConductivity,
                error.to_string(),
            ));
        })
        .ok()?;
    alignment_records.push(EnvironmentalAlignmentRecord {
        source_series: aligned.source_series.clone(),
        alignment: aligned.alignment,
        source_timestamps: aligned.source_timestamps.clone(),
        interpolated: aligned.interpolated,
        time_gap_s: aligned.time_gap_s,
    });
    Quantity::parse(aligned.value, &series.unit)
        .and_then(|quantity| quantity.to_conductivity_s_per_m())
        .inspect_err(|error| {
            warnings.push(CalibrationWarning::new(
                CalibrationWarningKind::MissingConductivity,
                error.to_string(),
            ));
        })
        .ok()
        .and_then(|value| Quantity::new(value, QuantityUnit::SiemensPerM).ok())
}

fn linear_slope(points: &[(f64, f64)]) -> f64 {
    let mean_x = points.iter().map(|(x, _)| *x).sum::<f64>() / points.len().max(1) as f64;
    let mean_y = points.iter().map(|(_, y)| *y).sum::<f64>() / points.len().max(1) as f64;
    let denominator = points
        .iter()
        .map(|(x, _)| (x - mean_x).powi(2))
        .sum::<f64>();
    if denominator > 0.0 {
        points
            .iter()
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>()
            / denominator
    } else {
        0.0
    }
}

fn standard_deviation(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    (values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64)
        .sqrt()
}

fn median(values: &[f64]) -> f64 {
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    if values.len().is_multiple_of(2) {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    } else {
        values[values.len() / 2]
    }
}
