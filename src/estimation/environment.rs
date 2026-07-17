#![allow(
    clippy::collapsible_if,
    clippy::filter_next,
    clippy::unnecessary_unwrap
)]

use crate::{
    calibration_config::ActivityConfig,
    domain::{ElectrochemicalExperiment, EnvironmentalSeries, ExperimentEventKind},
    estimation::{
        error::EstimationError,
        state::{EstimationWarning, EstimationWarningKind},
    },
    estimation_config::{
        AlignmentKind, EnvironmentConfig, PolarizationConfig, PolarizationInputModel,
    },
    potentiometry::{
        calibration::activity::evaluate_activity,
        units::{Quantity, QuantityUnit},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentMethod {
    Nearest,
    LinearInterpolation,
    WindowMean,
    WindowMedian,
    HoldPrevious,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandardValueKind {
    Activity,
    MolarConcentration,
    MassConcentration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignedValue {
    pub value: f64,
    pub source_series: String,
    pub source_timestamps: Vec<f64>,
    pub alignment: AlignmentMethod,
    pub time_gap_s: f64,
    pub interpolated: bool,
    pub extrapolated: bool,
    #[serde(default)]
    pub source_unit: Option<String>,
    #[serde(default)]
    pub conversion: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AlignedEnvironment {
    pub timestamp_s: f64,
    pub temperature_k: Option<f64>,
    pub conductivity_s_per_m: Option<f64>,
    pub ionic_strength_mol_l: Option<f64>,
    pub flow: Option<f64>,
    pub interferent_activities: BTreeMap<String, f64>,
    pub known_activity_log10: Option<f64>,
    pub known_standard: bool,
    #[serde(default)]
    pub known_standard_value_kind: Option<StandardValueKind>,
    #[serde(default)]
    pub known_standard_unit: Option<String>,
    #[serde(default)]
    pub known_standard_raw_value: Option<f64>,
    #[serde(default)]
    pub known_molar_concentration_mol_l: Option<f64>,
    #[serde(default)]
    pub known_standard_assumption: Option<String>,
    #[serde(default)]
    pub known_standard_provenance: Option<String>,
    pub polarization_input_v: Option<f64>,
    #[serde(default)]
    pub polarization_input_source: Option<String>,
    #[serde(default)]
    pub polarization_event_timestamp_s: Option<f64>,
    pub values: Vec<AlignedValue>,
    pub warnings: Vec<EstimationWarning>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AlignedEnvironmentSummary {
    pub temperature_k: Option<f64>,
    pub conductivity_s_per_m: Option<f64>,
    pub ionic_strength_mol_l: Option<f64>,
    pub flow: Option<f64>,
    pub known_activity_log10: Option<f64>,
    pub known_standard: bool,
    #[serde(default)]
    pub known_standard_value_kind: Option<StandardValueKind>,
    #[serde(default)]
    pub known_standard_unit: Option<String>,
    #[serde(default)]
    pub known_standard_assumption: Option<String>,
    #[serde(default)]
    pub known_standard_provenance: Option<String>,
    #[serde(default)]
    pub polarization_input_source: Option<String>,
    #[serde(default)]
    pub polarization_event_timestamp_s: Option<f64>,
    pub interferent_activities: BTreeMap<String, f64>,
    pub source_records: Vec<AlignedValueSummary>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignedValueSummary {
    pub source_series: String,
    pub source_timestamps: Vec<f64>,
    pub alignment: AlignmentMethod,
    pub time_gap_s: f64,
    pub interpolated: bool,
    pub extrapolated: bool,
    #[serde(default)]
    pub source_unit: Option<String>,
    #[serde(default)]
    pub conversion: Option<String>,
}
impl From<&AlignedEnvironment> for AlignedEnvironmentSummary {
    fn from(e: &AlignedEnvironment) -> Self {
        Self {
            temperature_k: e.temperature_k,
            conductivity_s_per_m: e.conductivity_s_per_m,
            ionic_strength_mol_l: e.ionic_strength_mol_l,
            flow: e.flow,
            known_activity_log10: e.known_activity_log10,
            known_standard: e.known_standard,
            known_standard_value_kind: e.known_standard_value_kind,
            known_standard_unit: e.known_standard_unit.clone(),
            known_standard_assumption: e.known_standard_assumption.clone(),
            known_standard_provenance: e.known_standard_provenance.clone(),
            polarization_input_source: e.polarization_input_source.clone(),
            polarization_event_timestamp_s: e.polarization_event_timestamp_s,
            interferent_activities: e.interferent_activities.clone(),
            source_records: e
                .values
                .iter()
                .map(|v| AlignedValueSummary {
                    source_series: v.source_series.clone(),
                    source_timestamps: v.source_timestamps.clone(),
                    alignment: v.alignment,
                    time_gap_s: v.time_gap_s,
                    interpolated: v.interpolated,
                    extrapolated: v.extrapolated,
                    source_unit: v.source_unit.clone(),
                    conversion: v.conversion.clone(),
                })
                .collect(),
        }
    }
}

pub fn align_experiment(
    experiment: &ElectrochemicalExperiment,
    timestamp_s: f64,
    config: &EnvironmentConfig,
    previous: Option<&AlignedEnvironment>,
) -> Result<AlignedEnvironment, EstimationError> {
    align_experiment_with_polarization(
        experiment,
        timestamp_s,
        config,
        previous,
        &PolarizationConfig::default(),
    )
}

pub fn align_experiment_with_polarization(
    experiment: &ElectrochemicalExperiment,
    timestamp_s: f64,
    config: &EnvironmentConfig,
    previous: Option<&AlignedEnvironment>,
    polarization_config: &PolarizationConfig,
) -> Result<AlignedEnvironment, EstimationError> {
    if !timestamp_s.is_finite() {
        return Err(EstimationError::invalid(
            "environment timestamp is nonfinite",
        ));
    }
    let mut result = AlignedEnvironment {
        timestamp_s,
        ..Default::default()
    };
    let method = match config.alignment {
        AlignmentKind::Nearest => AlignmentMethod::Nearest,
        AlignmentKind::LinearInterpolation => AlignmentMethod::LinearInterpolation,
        AlignmentKind::WindowMean => AlignmentMethod::WindowMean,
        AlignmentKind::WindowMedian => AlignmentMethod::WindowMedian,
        AlignmentKind::HoldPrevious => AlignmentMethod::HoldPrevious,
    };
    let find = |name: &Option<String>| {
        name.as_deref()
            .and_then(|n| experiment.environmental_data.iter().find(|s| s.name == n))
    };
    if let Some(series) = find(&config.temperature_series) {
        if let Some(mut value) = align_series(series, timestamp_s, method, config)? {
            result.temperature_k = Some(to_kelvin(value.value, &series.unit)?);
            value.source_unit = Some(series.unit.clone());
            value.conversion = Some("converted to K".into());
            result.values.push(value);
        } else {
            result.warnings.push(EstimationWarning::at(
                EstimationWarningKind::EnvironmentalAlignmentGap,
                format!(
                    "temperature series '{}' has no value within the configured gap",
                    series.name
                ),
                timestamp_s,
            ));
        }
    } else if config.allow_configured_fallback {
        result.temperature_k = Some(to_kelvin(config.fallback_temperature_celsius, "C")?);
        result.warnings.push(EstimationWarning::at(
            EstimationWarningKind::MissingTemperature,
            "temperature series is missing; configured fallback used",
            timestamp_s,
        ));
    } else {
        result.warnings.push(EstimationWarning::at(
            EstimationWarningKind::MissingTemperature,
            "temperature series is missing",
            timestamp_s,
        ));
    }
    if let Some(series) = find(&config.conductivity_series) {
        if let Some(mut value) = align_series(series, timestamp_s, method, config)? {
            result.conductivity_s_per_m = Some(to_conductivity(value.value, &series.unit)?);
            value.source_unit = Some(series.unit.clone());
            value.conversion = Some("converted to S/m".into());
            result.values.push(value);
        }
    } else if let Some(value) = config.fallback_conductivity_s_per_m {
        result.conductivity_s_per_m = Some(value);
    }
    if let Some(series) = find(&config.ionic_strength_series) {
        if let Some(mut value) = align_series(series, timestamp_s, method, config)? {
            let quantity = Quantity::parse(value.value, &series.unit).map_err(|error| {
                EstimationError::invalid(format!(
                    "ionic-strength series '{}' has unsupported unit '{}': {error}",
                    series.name, series.unit
                ))
            })?;
            result.ionic_strength_mol_l = Some(
                quantity
                    .to_molar_concentration(None)
                    .map_err(|error| EstimationError::invalid(error.to_string()))?,
            );
            value.source_unit = Some(series.unit.clone());
            value.conversion = Some("converted to mol/L".into());
            result.values.push(value);
        }
    } else {
        result.ionic_strength_mol_l = config.fallback_ionic_strength_mol_l;
    }
    if let Some(series) = find(&config.flow_series) {
        if let Some(value) = align_series(series, timestamp_s, method, config)? {
            result.flow = Some(value.value);
            result.values.push(value);
        }
    }
    for (name, series_name) in &config.interferent_series {
        if let Some(series) = experiment
            .environmental_data
            .iter()
            .find(|s| s.name == *series_name)
        {
            if let Some(value) = align_series(series, timestamp_s, method, config)? {
                result
                    .interferent_activities
                    .insert(name.clone(), value.value);
                result.values.push(value);
            }
        } else {
            result.warnings.push(EstimationWarning::at(
                EstimationWarningKind::MissingInterferentActivity,
                format!("interferent activity series '{}' is missing", series_name),
                timestamp_s,
            ));
        }
    }
    let latest_event = experiment
        .ordered_events()
        .iter()
        .enumerate()
        .filter(|(_, event)| event.timestamp <= timestamp_s)
        .next_back();
    if let Some((_, event)) = latest_event {
        if event.kind == ExperimentEventKind::ConcentrationStep {
            if event.value.is_some_and(|v| v.is_finite() && v > 0.0) {
                let metadata = event.metadata.as_ref();
                let is_standard = event
                    .annotation
                    .as_deref()
                    .is_some_and(|x| x.to_ascii_lowercase().contains("standard"))
                    || metadata
                        .and_then(|m| m.get("known_standard"))
                        .is_some_and(|x| x.eq_ignore_ascii_case("true"));
                result.known_standard = is_standard;
                if is_standard {
                    parse_standard_event(&mut result, event)?;
                }
            }
        }
    }
    if matches!(config.alignment, AlignmentKind::HoldPrevious) && previous.is_some() {
        let p = previous.unwrap();
        if result.temperature_k.is_none() {
            result.temperature_k = p.temperature_k;
        }
        if result.conductivity_s_per_m.is_none() {
            result.conductivity_s_per_m = p.conductivity_s_per_m;
        }
        if result.ionic_strength_mol_l.is_none() {
            result.ionic_strength_mol_l = p.ionic_strength_mol_l;
        }
    }
    apply_polarization_input(
        &mut result,
        experiment,
        timestamp_s,
        previous,
        polarization_config,
    )?;
    Ok(result)
}

fn align_series(
    series: &EnvironmentalSeries,
    timestamp: f64,
    method: AlignmentMethod,
    config: &EnvironmentConfig,
) -> Result<Option<AlignedValue>, EstimationError> {
    let mut pairs = series
        .time
        .iter()
        .copied()
        .zip(series.values.iter().copied())
        .filter_map(|(t, v)| (t.is_finite() && v.is_some_and(f64::is_finite)).then_some((t, v?)))
        .collect::<Vec<_>>();
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    if pairs.is_empty() {
        return Ok(None);
    }
    let nearest = pairs
        .iter()
        .min_by(|a, b| (a.0 - timestamp).abs().total_cmp(&(b.0 - timestamp).abs()))
        .copied()
        .unwrap();
    let gap = (nearest.0 - timestamp).abs();
    if gap > config.maximum_gap_s {
        return Ok(None);
    }
    let (value, times, interpolated) = match method {
        AlignmentMethod::Nearest | AlignmentMethod::HoldPrevious | AlignmentMethod::Fallback => {
            let candidate = if method == AlignmentMethod::HoldPrevious {
                pairs
                    .iter()
                    .rev()
                    .find(|(t, _)| *t <= timestamp)
                    .copied()
                    .unwrap_or(nearest)
            } else {
                nearest
            };
            (candidate.1, vec![candidate.0], false)
        }
        AlignmentMethod::LinearInterpolation => {
            if let Some(window) = pairs
                .windows(2)
                .find(|w| w[0].0 <= timestamp && timestamp <= w[1].0)
            {
                if window[1].0 - window[0].0 > config.maximum_gap_s {
                    return Ok(None);
                }
                let f = (timestamp - window[0].0) / (window[1].0 - window[0].0).max(f64::EPSILON);
                (
                    window[0].1 + f * (window[1].1 - window[0].1),
                    vec![window[0].0, window[1].0],
                    true,
                )
            } else {
                (nearest.1, vec![nearest.0], false)
            }
        }
        AlignmentMethod::WindowMean | AlignmentMethod::WindowMedian => {
            let mut values = pairs
                .iter()
                .filter(|(t, _)| (*t - timestamp).abs() <= config.window_half_width_s)
                .map(|(_, v)| *v)
                .collect::<Vec<_>>();
            if values.is_empty() {
                return Ok(None);
            }
            values.sort_by(f64::total_cmp);
            let v = if method == AlignmentMethod::WindowMean {
                values.iter().sum::<f64>() / values.len() as f64
            } else if values.len() % 2 == 0 {
                (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
            } else {
                values[values.len() / 2]
            };
            (v, vec![timestamp], false)
        }
    };
    Ok(Some(AlignedValue {
        value,
        source_series: series.name.clone(),
        source_timestamps: times,
        alignment: method,
        time_gap_s: gap,
        interpolated,
        extrapolated: timestamp < pairs[0].0 || timestamp > pairs.last().unwrap().0,
        source_unit: None,
        conversion: None,
    }))
}
fn to_kelvin(value: f64, unit: &str) -> Result<f64, EstimationError> {
    Quantity::parse(value, unit)
        .and_then(|quantity| quantity.to_temperature_k())
        .map_err(|error| EstimationError::invalid(error.to_string()))
}
fn to_conductivity(value: f64, unit: &str) -> Result<f64, EstimationError> {
    Quantity::parse(value, unit)
        .and_then(|quantity| quantity.to_conductivity_s_per_m())
        .map_err(|error| EstimationError::invalid(error.to_string()))
}

fn parse_standard_event(
    result: &mut AlignedEnvironment,
    event: &crate::domain::ExperimentEvent,
) -> Result<(), EstimationError> {
    let metadata = event.metadata.as_ref();
    let unit_name = event
        .unit
        .as_deref()
        .or_else(|| metadata.and_then(|m| m.get("standard_unit").map(String::as_str)));
    let kind_name = metadata.and_then(|m| m.get("standard_value_kind").map(String::as_str));
    let value = event
        .value
        .ok_or_else(|| EstimationError::invalid("known standard has no value"))?;
    let kind_from_metadata = kind_name
        .map(|kind| match kind.to_ascii_lowercase().as_str() {
            "activity" => Ok(StandardValueKind::Activity),
            "molar_concentration" | "molar concentration" => {
                Ok(StandardValueKind::MolarConcentration)
            }
            "mass_concentration" | "mass concentration" => Ok(StandardValueKind::MassConcentration),
            _ => Err(EstimationError::invalid(format!(
                "unsupported standard value kind '{kind}'"
            ))),
        })
        .transpose()?;
    let quantity = unit_name
        .map(|unit| Quantity::parse(value, unit))
        .transpose()
        .map_err(|error| {
            EstimationError::invalid(format!("known standard unit is invalid: {error}"))
        })?;
    let kind = kind_from_metadata
        .or_else(|| {
            quantity.as_ref().map(|q| match q.unit {
                QuantityUnit::Activity => StandardValueKind::Activity,
                QuantityUnit::MolPerL | QuantityUnit::MmolPerL | QuantityUnit::MicromolPerL => {
                    StandardValueKind::MolarConcentration
                }
                QuantityUnit::MgPerL | QuantityUnit::GPerL => StandardValueKind::MassConcentration,
                _ => StandardValueKind::MassConcentration,
            })
        })
        .ok_or_else(|| {
            EstimationError::invalid("known standard requires an explicit unit or metadata kind")
        })?;
    result.known_standard_value_kind = Some(kind);
    result.known_standard_unit = unit_name.map(str::to_string);
    result.known_standard_raw_value = Some(value);
    result.known_standard_provenance =
        Some("concentration-step event with explicit standard metadata".into());
    match kind {
        StandardValueKind::Activity => {
            let activity = quantity
                .ok_or_else(|| {
                    EstimationError::invalid("activity standard requires an activity unit")
                })?
                .to_activity()
                .map_err(|error| EstimationError::invalid(error.to_string()))?;
            result.known_activity_log10 = Some(activity.log10());
            result.known_standard_assumption = Some("event explicitly declared activity".into());
        }
        StandardValueKind::MolarConcentration => {
            let molar = quantity
                .ok_or_else(|| {
                    EstimationError::invalid("molar standard requires a concentration unit")
                })?
                .to_molar_concentration(None)
                .map_err(|error| EstimationError::invalid(error.to_string()))?;
            result.known_molar_concentration_mol_l = Some(molar);
            result.known_activity_log10 = Some(molar.log10());
            result.known_standard_assumption = Some("ideal activity (activity equals molar concentration) pending calibration-model correction".into());
        }
        StandardValueKind::MassConcentration => {
            result.known_standard_assumption = Some(
                "mass concentration conversion deferred until analyte molar mass is available"
                    .into(),
            );
        }
    }
    Ok(())
}

fn apply_polarization_input(
    result: &mut AlignedEnvironment,
    experiment: &ElectrochemicalExperiment,
    timestamp_s: f64,
    previous: Option<&AlignedEnvironment>,
    config: &PolarizationConfig,
) -> Result<(), EstimationError> {
    if matches!(config.input_model, PolarizationInputModel::None) {
        return Ok(());
    }
    let kind = config
        .input_event_kind
        .as_deref()
        .unwrap_or("concentration_step");
    let event = experiment
        .ordered_events()
        .iter()
        .filter(|event| event.timestamp <= timestamp_s && event_kind_matches(event, kind))
        .next_back();
    let Some(event) = event else { return Ok(()) };
    if previous.is_some_and(|prior| event.timestamp <= prior.timestamp_s) {
        return Ok(());
    }
    let input = match config.input_model {
        PolarizationInputModel::None => None,
        PolarizationInputModel::ExplicitEventVoltage => {
            let metadata = event.metadata.as_ref();
            let value = metadata
                .and_then(|m| m.get("polarization_input_v"))
                .ok_or_else(|| {
                    EstimationError::invalid(
                        "explicit polarization event lacks metadata polarization_input_v",
                    )
                })?
                .parse::<f64>()
                .map_err(|_| {
                    EstimationError::invalid("explicit polarization input is not numeric")
                })?;
            let unit = metadata
                .and_then(|m| m.get("polarization_input_unit"))
                .map(String::as_str)
                .unwrap_or("V");
            Some(
                Quantity::parse(value, unit)
                    .and_then(|q| q.to_potential_v())
                    .map_err(|e| EstimationError::invalid(e.to_string()))?,
            )
        }
        PolarizationInputModel::ActivityStepGain => {
            let current = event_log10_activity(event)?;
            let previous_log = event
                .metadata
                .as_ref()
                .and_then(|m| m.get("previous_log10_activity"))
                .and_then(|value| value.parse::<f64>().ok())
                .or_else(|| {
                    experiment
                        .ordered_events()
                        .iter()
                        .rev()
                        .filter(|candidate| {
                            candidate.timestamp < event.timestamp && candidate.kind == event.kind
                        })
                        .find_map(|candidate| event_log10_activity(candidate).ok())
                });
            previous_log.map(|previous| config.gain_v_per_log10_activity * (current - previous))
        }
    };
    if let Some(input) = input.filter(|value| value.is_finite()) {
        result.polarization_input_v = Some(input);
        result.polarization_event_timestamp_s = Some(event.timestamp);
        result.polarization_input_source =
            Some(format!("{:?} from {} event", config.input_model, kind));
    }
    Ok(())
}

fn event_kind_matches(event: &crate::domain::ExperimentEvent, configured: &str) -> bool {
    let expected = configured.trim().to_ascii_lowercase();
    format!("{:?}", event.kind).to_ascii_lowercase() == expected
        || matches!(
            (event.kind, expected.as_str()),
            (ExperimentEventKind::ConcentrationStep, "concentration_step")
                | (ExperimentEventKind::ConcentrationStep, "concentration-step")
        )
}

fn event_log10_activity(event: &crate::domain::ExperimentEvent) -> Result<f64, EstimationError> {
    let value = event
        .value
        .ok_or_else(|| EstimationError::invalid("activity-step event lacks a value"))?;
    let unit = event
        .unit
        .as_deref()
        .ok_or_else(|| EstimationError::invalid("activity-step event lacks an explicit unit"))?;
    let quantity =
        Quantity::parse(value, unit).map_err(|e| EstimationError::invalid(e.to_string()))?;
    let activity = match quantity.unit {
        QuantityUnit::Activity => quantity
            .to_activity()
            .map_err(|e| EstimationError::invalid(e.to_string()))?,
        _ => quantity
            .to_molar_concentration(None)
            .map_err(|e| EstimationError::invalid(e.to_string()))?,
    };
    Ok(activity.log10())
}

/// Resolve the parsed standard against the same Phase 3 activity model used by
/// calibration. Nonideal models require ionic strength explicitly.
pub fn resolve_standard_activity(
    environment: &mut AlignedEnvironment,
    activity_config: &ActivityConfig,
    molar_mass_g_per_mol: Option<f64>,
    ion_charge: i32,
) -> Result<(), EstimationError> {
    if !environment.known_standard {
        return Ok(());
    }
    if matches!(
        environment.known_standard_value_kind,
        Some(StandardValueKind::Activity)
    ) {
        return Ok(());
    }
    let quantity = if matches!(
        environment.known_standard_value_kind,
        Some(StandardValueKind::MassConcentration)
    ) {
        let unit = environment
            .known_standard_unit
            .as_deref()
            .ok_or_else(|| EstimationError::invalid("mass standard has no unit"))?;
        Quantity::parse(
            environment.known_standard_raw_value.unwrap_or(f64::NAN),
            unit,
        )
        .map_err(|error| EstimationError::invalid(error.to_string()))?
    } else {
        Quantity::new(
            environment.known_molar_concentration_mol_l.ok_or_else(|| {
                EstimationError::invalid("standard concentration was not converted")
            })?,
            QuantityUnit::MolPerL,
        )
        .map_err(|error| EstimationError::invalid(error.to_string()))?
    };
    let evaluation = evaluate_activity(
        Some(&quantity),
        molar_mass_g_per_mol,
        None,
        None,
        ion_charge,
        environment.ionic_strength_mol_l,
        environment.conductivity_s_per_m,
        activity_config,
    )
    .map_err(|error| EstimationError::Calibration(error.to_string()))?;
    environment.known_activity_log10 = Some(evaluation.log10_activity);
    environment.known_standard_assumption = Some(match activity_config.model {
        crate::results::calibration::ActivityModelKind::Ideal => {
            "ideal activity equals molar concentration".into()
        }
        _ => "activity coefficient calculated by the configured Phase 3 activity model".into(),
    });
    environment.known_standard_provenance = Some(format!(
        "converted {} to mol/L and evaluated {:?} activity model",
        environment
            .known_standard_unit
            .as_deref()
            .unwrap_or("mol/L"),
        activity_config.model
    ));
    Ok(())
}
