#![allow(
    clippy::collapsible_if,
    clippy::filter_next,
    clippy::unnecessary_unwrap
)]

use crate::{
    domain::{ElectrochemicalExperiment, EnvironmentalSeries, ExperimentEventKind},
    estimation::{
        error::EstimationError,
        state::{EstimationWarning, EstimationWarningKind},
    },
    estimation_config::{AlignmentKind, EnvironmentConfig},
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignedValue {
    pub value: f64,
    pub source_series: String,
    pub source_timestamps: Vec<f64>,
    pub alignment: AlignmentMethod,
    pub time_gap_s: f64,
    pub interpolated: bool,
    pub extrapolated: bool,
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
    pub polarization_input_v: Option<f64>,
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
        if let Some(value) = align_series(series, timestamp_s, method, config)? {
            result.temperature_k = Some(to_kelvin(value.value, &series.unit)?);
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
        if let Some(value) = align_series(series, timestamp_s, method, config)? {
            result.conductivity_s_per_m = Some(to_conductivity(value.value, &series.unit));
            result.values.push(value);
        }
    } else if let Some(value) = config.fallback_conductivity_s_per_m {
        result.conductivity_s_per_m = Some(value);
    }
    if let Some(series) = find(&config.ionic_strength_series) {
        if let Some(value) = align_series(series, timestamp_s, method, config)? {
            result.ionic_strength_mol_l = Some(value.value);
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
            if let Some(value) = event.value.filter(|v| v.is_finite() && *v > 0.0) {
                result.known_activity_log10 = Some(value.log10());
                result.known_standard = event
                    .annotation
                    .as_deref()
                    .is_some_and(|x| x.to_ascii_lowercase().contains("standard"));
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
    }))
}
fn to_kelvin(value: f64, unit: &str) -> Result<f64, EstimationError> {
    let k = if unit.to_ascii_lowercase().contains('k') {
        value
    } else {
        value + 273.15
    };
    if k.is_finite() && k > 0.0 {
        Ok(k)
    } else {
        Err(EstimationError::invalid("temperature is not physical"))
    }
}
fn to_conductivity(value: f64, unit: &str) -> f64 {
    let u = unit.to_ascii_lowercase();
    if u.contains("ms") {
        value * 1e-3
    } else if u.contains("us") {
        value * 1e-6
    } else {
        value
    }
}
