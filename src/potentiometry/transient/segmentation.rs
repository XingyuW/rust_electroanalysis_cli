//! Event selection, paired timestamp policies, and baseline segmentation.

use super::models::{BaselineMethod, ResponseMode};
use crate::domain::{
    ExperimentEvent, ExperimentEventKind, MeasurementChannel, MultiChannelMeasurement,
};
use crate::potentiometry::PotentiometryError;
use crate::results::transient::{
    BaselineResult, ConcentrationContext, SegmentSummary, TransientWarning, TransientWarningKind,
};
use crate::transient_config::{
    DuplicateTimestampPolicy, IrregularSamplingPolicy, NonMonotonicPolicy, ResolvedTransientConfig,
};

#[derive(Debug, Clone)]
pub struct PreparedSegment {
    pub summary: SegmentSummary,
    pub baseline: BaselineResult,
    pub fit_time_local: Vec<f64>,
    pub fit_values: Vec<f64>,
    pub response_offset: f64,
    pub warnings: Vec<TransientWarning>,
}

pub fn concentration_context(event: &ExperimentEvent) -> Option<ConcentrationContext> {
    event
        .value
        .filter(|value| value.is_finite())
        .map(|value| ConcentrationContext {
            value,
            unit: event.unit.clone(),
            analyte: event.analyte.clone(),
        })
}

pub fn derive_concentration_before(
    events: &[ExperimentEvent],
    source_index: usize,
    current: &ExperimentEvent,
) -> Option<ConcentrationContext> {
    events[..source_index]
        .iter()
        .rev()
        .filter(|event| {
            event.kind == ExperimentEventKind::ConcentrationStep
                && analytes_compatible(current.analyte.as_deref(), event.analyte.as_deref())
        })
        .filter_map(concentration_context)
        .next()
}

fn analytes_compatible(current: Option<&str>, previous: Option<&str>) -> bool {
    match (current, previous) {
        (Some(current), Some(previous)) => current == previous,
        (None, None) => true,
        _ => false,
    }
}

pub fn prepare_segment(
    measurement: &MultiChannelMeasurement,
    channel: &MeasurementChannel,
    event: &ExperimentEvent,
    _source_index: usize,
    config: &ResolvedTransientConfig,
) -> Result<PreparedSegment, PotentiometryError> {
    let min_time = measurement
        .time
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .min_by(f64::total_cmp)
        .ok_or_else(|| {
            PotentiometryError::InvalidEventWindow(
                "measurement has no finite timestamps".to_string(),
            )
        })?;
    let max_time = measurement
        .time
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .max_by(f64::total_cmp)
        .ok_or_else(|| {
            PotentiometryError::InvalidEventWindow(
                "measurement has no finite timestamps".to_string(),
            )
        })?;
    if event.timestamp < min_time || event.timestamp > max_time {
        return Err(PotentiometryError::InvalidEventWindow(format!(
            "event timestamp {:.6} lies outside measurement range [{min_time:.6}, {max_time:.6}]",
            event.timestamp
        )));
    }

    let window_start = event.timestamp - config.segmentation.pre_event_s;
    let window_end = event.timestamp + config.segmentation.post_event_s;
    let mut pre_values = Vec::new();
    let mut post_rows = Vec::new();

    for (time, value) in measurement.time.iter().zip(channel.values.iter()) {
        if !time.is_finite() || *time < window_start || *time > window_end {
            continue;
        }
        let local_time = *time - event.timestamp;
        if local_time < 0.0 {
            if local_time >= -config.segmentation.baseline_window_s
                && let Some(value) = value.as_ref().filter(|value| value.is_finite())
            {
                pre_values.push((local_time, *value));
            }
        } else {
            post_rows.push((local_time, *value));
        }
    }

    if post_rows.is_empty() {
        return Err(PotentiometryError::InvalidEventWindow(
            "event has no observations at or after the event timestamp".to_string(),
        ));
    }

    let original_times = post_rows.iter().map(|(time, _)| *time).collect::<Vec<_>>();
    let original_diagnostics = diagnostics_for_times(&original_times);
    if original_diagnostics.duplicate_timestamps > 0
        && config.segmentation.duplicate_timestamp_policy == DuplicateTimestampPolicy::Error
    {
        return Err(PotentiometryError::DuplicateTimestamps);
    }
    if original_diagnostics.non_monotonic_timestamps > 0
        && config.segmentation.non_monotonic_policy == NonMonotonicPolicy::Error
    {
        return Err(PotentiometryError::NonMonotonicTimestamps);
    }
    if original_diagnostics.irregular_sampling
        && config.segmentation.irregular_sampling_policy == IrregularSamplingPolicy::Error
    {
        return Err(PotentiometryError::InvalidEventWindow(
            "irregular sampling is not permitted under the configured policy".to_string(),
        ));
    }

    if original_diagnostics.non_monotonic_timestamps > 0
        && config.segmentation.non_monotonic_policy == NonMonotonicPolicy::Sort
    {
        post_rows.sort_by(|left, right| left.0.total_cmp(&right.0));
    }
    if config.segmentation.duplicate_timestamp_policy == DuplicateTimestampPolicy::Average {
        post_rows = average_duplicate_rows(post_rows);
    }

    let raw_observations = post_rows.len();
    let missing_observations = post_rows
        .iter()
        .filter(|(_, value)| value.is_none())
        .count();
    let missing_fraction =
        (raw_observations > 0).then_some(missing_observations as f64 / raw_observations as f64);
    if missing_fraction
        .is_some_and(|fraction| fraction > config.segmentation.maximum_missing_fraction)
    {
        return Err(PotentiometryError::ExcessiveMissingData {
            fraction: missing_fraction.unwrap_or(1.0),
            maximum: config.segmentation.maximum_missing_fraction,
        });
    }

    let fit_rows = post_rows
        .iter()
        .filter_map(|(time, value)| value.map(|value| (*time, value)))
        .collect::<Vec<_>>();
    if fit_rows.len() < config.segmentation.minimum_points {
        return Err(PotentiometryError::InsufficientObservations {
            required: config.segmentation.minimum_points,
            actual: fit_rows.len(),
        });
    }
    let finite_duration_s = fit_rows
        .first()
        .zip(fit_rows.last())
        .map(|((first, _), (last, _))| last - first);
    if finite_duration_s.unwrap_or(0.0) < config.segmentation.minimum_duration_s {
        return Err(PotentiometryError::TooShortObservationWindow {
            required: config.segmentation.minimum_duration_s,
            actual: finite_duration_s.unwrap_or(0.0),
        });
    }

    let (baseline, mut warnings) = estimate_baseline(
        &pre_values,
        config.baseline.method,
        config.baseline.response_mode,
    );
    let response_offset = if config.baseline.response_mode == ResponseMode::BaselineRelative {
        baseline.estimate_v.unwrap_or(0.0)
    } else {
        0.0
    };
    if config.baseline.response_mode == ResponseMode::BaselineRelative
        && baseline.estimate_v.is_none()
    {
        warnings.push(TransientWarning::new(
            TransientWarningKind::BaselineUnavailable,
            "pre-event baseline was unavailable; fitting used absolute potential",
        ));
    }
    if original_diagnostics.irregular_sampling {
        warnings.push(TransientWarning::new(
            TransientWarningKind::IrregularSampling,
            "actual irregular timestamps were used without resampling",
        ));
    }
    if original_diagnostics.duplicate_timestamps > 0 {
        warnings.push(TransientWarning::new(
            TransientWarningKind::DuplicateTimestamps,
            "duplicate timestamps were averaged according to configuration",
        ));
    }
    if original_diagnostics.non_monotonic_timestamps > 0 {
        warnings.push(TransientWarning::new(
            TransientWarningKind::NonMonotonicTimestamps,
            "paired observations were sorted by timestamp according to configuration",
        ));
    }

    let fit_time_local = fit_rows.iter().map(|(time, _)| *time).collect::<Vec<_>>();
    let fit_values = fit_rows
        .iter()
        .map(|(_, value)| *value - response_offset)
        .collect::<Vec<_>>();
    let summary = SegmentSummary {
        segment_start: fit_time_local.first().map(|time| event.timestamp + *time),
        segment_end: fit_time_local.last().map(|time| event.timestamp + *time),
        local_start: fit_time_local.first().copied(),
        local_end: fit_time_local.last().copied(),
        finite_duration_s,
        raw_observations,
        finite_fitted_observations: fit_rows.len(),
        missing_observations,
        missing_fraction,
        irregular_sampling: original_diagnostics.irregular_sampling,
        duplicate_timestamps: original_diagnostics.duplicate_timestamps,
        non_monotonic_timestamps: original_diagnostics.non_monotonic_timestamps,
        raw_time_local: post_rows.iter().map(|(time, _)| *time).collect(),
        raw_potential_v: post_rows.iter().map(|(_, value)| *value).collect(),
        fitted_time_local: fit_time_local.clone(),
    };

    Ok(PreparedSegment {
        summary,
        baseline,
        fit_time_local,
        fit_values,
        response_offset,
        warnings,
    })
}

fn diagnostics_for_times(time: &[f64]) -> crate::domain::ParseDiagnostics {
    let mut diagnostics = crate::domain::ParseDiagnostics {
        total_rows: time.len(),
        successfully_parsed_rows: time.len(),
        ..crate::domain::ParseDiagnostics::default()
    };
    diagnostics.update_time_axis(time);
    diagnostics
}

fn average_duplicate_rows(mut rows: Vec<(f64, Option<f64>)>) -> Vec<(f64, Option<f64>)> {
    rows.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut averaged = Vec::new();
    let mut index = 0;
    while index < rows.len() {
        let timestamp = rows[index].0;
        let mut values = Vec::new();
        while index < rows.len() && rows[index].0 == timestamp {
            if let Some(value) = rows[index].1 {
                values.push(value);
            }
            index += 1;
        }
        averaged.push((
            timestamp,
            (!values.is_empty()).then_some(values.iter().sum::<f64>() / values.len() as f64),
        ));
    }
    averaged
}

fn estimate_baseline(
    values: &[(f64, f64)],
    method: BaselineMethod,
    response_mode: ResponseMode,
) -> (BaselineResult, Vec<TransientWarning>) {
    let mut result = BaselineResult {
        method,
        response_mode,
        finite_points: values.len(),
        time_local: values.iter().map(|(time, _)| *time).collect(),
        potential_v: values.iter().map(|(_, value)| *value).collect(),
        ..BaselineResult::default()
    };
    let mut warnings = Vec::new();
    if values.is_empty() {
        result.warning = Some("no finite pre-event baseline observations".to_string());
        warnings.push(TransientWarning::new(
            TransientWarningKind::BaselineUnavailable,
            "no finite pre-event baseline observations were available",
        ));
        return (result, warnings);
    }

    match method {
        BaselineMethod::Mean => {
            result.estimate_v =
                Some(values.iter().map(|(_, value)| *value).sum::<f64>() / values.len() as f64);
        }
        BaselineMethod::Median => {
            let mut sorted = values.iter().map(|(_, value)| *value).collect::<Vec<_>>();
            sorted.sort_by(f64::total_cmp);
            result.estimate_v = Some(if sorted.len() % 2 == 0 {
                (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
            } else {
                sorted[sorted.len() / 2]
            });
        }
        BaselineMethod::Linear => {
            let mean_t = values.iter().map(|(time, _)| *time).sum::<f64>() / values.len() as f64;
            let mean_v = values.iter().map(|(_, value)| *value).sum::<f64>() / values.len() as f64;
            let denominator = values
                .iter()
                .map(|(time, _)| (time - mean_t).powi(2))
                .sum::<f64>();
            let slope = if denominator > 0.0 {
                values
                    .iter()
                    .map(|(time, value)| (time - mean_t) * (value - mean_v))
                    .sum::<f64>()
                    / denominator
            } else {
                0.0
            };
            result.slope_v_per_s = Some(slope);
            result.estimate_v = Some(mean_v - slope * mean_t);
        }
    }
    (result, warnings)
}
