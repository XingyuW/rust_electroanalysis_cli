//! Timestamp diagnostics and preprocessing for state estimation.

use crate::domain::{MeasurementChannel, MultiChannelMeasurement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimestampDiagnostics {
    pub total_rows: usize,
    pub finite_timestamps: usize,
    pub non_finite_timestamps: usize,
    pub duplicate_count: usize,
    pub identical_duplicate_count: usize,
    pub conflicting_duplicate_count: usize,
    pub local_reversal_count: usize,
    pub reset_count: usize,
    pub largest_backward_jump_s: Option<f64>,
    pub min_delta_s: Option<f64>,
    pub median_delta_s: Option<f64>,
    pub max_delta_s: Option<f64>,
    pub segment_count: usize,
    pub rows_reordered: usize,
    pub rows_removed: usize,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicatePolicy {
    DeduplicateIdentical,
    Reject,
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonMonotonicPolicy {
    SegmentOnReset,
    StableSortWithinSegment,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonFiniteTimestampPolicy {
    Reject,
    DropRows,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimestampHandlingConfig {
    pub duplicate_policy: DuplicatePolicy,
    pub non_monotonic_policy: NonMonotonicPolicy,
    pub non_finite_policy: NonFiniteTimestampPolicy,
    pub minor_reversal_threshold_s: f64,
    pub reset_threshold_s: f64,
    pub reset_threshold_fraction: f64,
    pub minimum_segment_points: usize,
}

impl Default for TimestampHandlingConfig {
    fn default() -> Self {
        Self {
            duplicate_policy: DuplicatePolicy::DeduplicateIdentical,
            non_monotonic_policy: NonMonotonicPolicy::SegmentOnReset,
            non_finite_policy: NonFiniteTimestampPolicy::Reject,
            minor_reversal_threshold_s: 1.0,
            reset_threshold_s: 10.0,
            reset_threshold_fraction: 0.5,
            minimum_segment_points: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimestampSegment {
    pub segment_index: usize,
    pub start_index: usize,
    pub end_index: usize,
    pub original_start_index: usize,
    pub original_end_index: usize,
    pub created_by_reset: bool,
    pub point_count: usize,
    pub min_timestamp_s: f64,
    pub max_timestamp_s: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkippedTimestampSegment {
    pub original_start_index: usize,
    pub original_end_index: usize,
    pub point_count: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreprocessedMeasurement {
    pub measurement: MultiChannelMeasurement,
    pub segments: Vec<TimestampSegment>,
    pub skipped_segments: Vec<SkippedTimestampSegment>,
    pub original_indices: Vec<usize>,
    pub diagnostics: TimestampDiagnostics,
    pub applied_policy: TimestampHandlingConfig,
    pub was_transformed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreprocessedTimestamps {
    pub timestamps: Vec<f64>,
    pub original_indices: Vec<usize>,
    pub segments: Vec<TimestampSegment>,
    pub diagnostics: TimestampDiagnostics,
    pub was_transformed: bool,
}

pub fn diagnose_timestamps(
    timestamps: &[f64],
    values: &[Vec<Option<f64>>],
) -> TimestampDiagnostics {
    diagnose_timestamps_with_config(timestamps, values, &TimestampHandlingConfig::default())
}

pub fn diagnose_timestamps_with_config(
    timestamps: &[f64],
    values: &[Vec<Option<f64>>],
    config: &TimestampHandlingConfig,
) -> TimestampDiagnostics {
    let mut diag = TimestampDiagnostics {
        total_rows: timestamps.len(),
        ..Default::default()
    };
    if timestamps.is_empty() {
        return diag;
    }

    let mut finite_indices = Vec::new();
    for (idx, t) in timestamps.iter().enumerate() {
        if t.is_finite() {
            finite_indices.push(idx);
            diag.finite_timestamps += 1;
        } else {
            diag.non_finite_timestamps += 1;
        }
    }
    if finite_indices.len() < 2 {
        return diag;
    }

    use std::collections::BTreeMap;
    let mut groups: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for &idx in &finite_indices {
        groups
            .entry(timestamps[idx].to_bits())
            .or_default()
            .push(idx);
    }

    for group in groups.values() {
        if group.len() <= 1 {
            continue;
        }
        let extra = group.len() - 1;
        diag.duplicate_count += extra;
        if rows_identical(group, values) {
            diag.identical_duplicate_count += extra;
        } else {
            diag.conflicting_duplicate_count += extra;
        }
    }

    let mut positive_deltas = Vec::new();
    for pair in finite_indices.windows(2) {
        let prev = timestamps[pair[0]];
        let curr = timestamps[pair[1]];
        let delta = curr - prev;
        if delta < 0.0 {
            let backward = -delta;
            diag.largest_backward_jump_s = Some(
                diag.largest_backward_jump_s
                    .map(|existing| existing.max(backward))
                    .unwrap_or(backward),
            );
            let is_reset = backward >= config.reset_threshold_s
                || curr <= prev * config.reset_threshold_fraction;
            if is_reset {
                diag.reset_count += 1;
            } else {
                diag.local_reversal_count += 1;
            }
        } else if delta > 0.0 {
            positive_deltas.push(delta);
        }
    }

    if !positive_deltas.is_empty() {
        positive_deltas.sort_by(f64::total_cmp);
        diag.min_delta_s = positive_deltas.first().copied();
        diag.max_delta_s = positive_deltas.last().copied();
        let mid = positive_deltas.len() / 2;
        diag.median_delta_s = Some(if positive_deltas.len() % 2 == 0 {
            (positive_deltas[mid - 1] + positive_deltas[mid]) / 2.0
        } else {
            positive_deltas[mid]
        });
    }

    diag.segment_count = diag.reset_count + 1;
    diag
}

pub fn preprocess_measurement(
    measurement: &MultiChannelMeasurement,
    config: &TimestampHandlingConfig,
) -> Result<PreprocessedMeasurement, String> {
    if measurement.time.is_empty() {
        return Err("timestamp array is empty".to_string());
    }
    validate_config(config)?;

    let value_matrix: Vec<Vec<Option<f64>>> = measurement
        .channels
        .iter()
        .map(|channel| channel.values.clone())
        .collect();
    let mut diagnostics = diagnose_timestamps_with_config(&measurement.time, &value_matrix, config);

    let mut rows: Vec<usize> = (0..measurement.time.len()).collect();
    let mut removed_rows = Vec::new();
    if diagnostics.non_finite_timestamps > 0 {
        match config.non_finite_policy {
            NonFiniteTimestampPolicy::Reject => {
                return Err(format!(
                    "measurement contains {} non-finite timestamps; set non_finite_policy=drop_rows to continue",
                    diagnostics.non_finite_timestamps
                ));
            }
            NonFiniteTimestampPolicy::DropRows => {
                rows.retain(|&idx| {
                    let keep = measurement.time[idx].is_finite();
                    if !keep {
                        removed_rows.push(idx);
                    }
                    keep
                });
                diagnostics.rows_removed += removed_rows.len();
                diagnostics.messages.push(format!(
                    "dropped {} rows with non-finite timestamps",
                    removed_rows.len()
                ));
            }
        }
    }

    let segment_boundaries = detect_segment_boundaries(&measurement.time, &rows, config);
    let mut final_rows = Vec::new();
    let mut segments = Vec::new();
    let mut skipped_segments = Vec::new();
    let mut rows_reordered = 0usize;
    let mut rows_removed = removed_rows.len();

    for (seg_idx, (start, end, created_by_reset)) in segment_boundaries.into_iter().enumerate() {
        let mut segment_rows = rows[start..end].to_vec();
        if segment_rows.is_empty() {
            continue;
        }

        let mut had_reversal = false;
        for pair in segment_rows.windows(2) {
            let prev = measurement.time[pair[0]];
            let curr = measurement.time[pair[1]];
            if curr < prev {
                had_reversal = true;
                let backward = prev - curr;
                let is_reset = backward >= config.reset_threshold_s
                    || curr <= prev * config.reset_threshold_fraction;
                if is_reset {
                    return Err(
                        "internal reset segmentation error: reset remained inside segment".into(),
                    );
                }
                if backward > config.minor_reversal_threshold_s {
                    return Err(format!(
                        "ambiguous backward jump {:.6}s within segment (larger than minor_reversal_threshold_s={:.6})",
                        backward, config.minor_reversal_threshold_s
                    ));
                }
            }
        }

        if had_reversal {
            match config.non_monotonic_policy {
                NonMonotonicPolicy::Reject => {
                    return Err(
                        "non-monotonic timestamps present and non_monotonic_policy=reject".into(),
                    );
                }
                NonMonotonicPolicy::SegmentOnReset
                | NonMonotonicPolicy::StableSortWithinSegment => {
                    let before = segment_rows.clone();
                    segment_rows
                        .sort_by(|a, b| measurement.time[*a].total_cmp(&measurement.time[*b]));
                    rows_reordered += before
                        .iter()
                        .zip(segment_rows.iter())
                        .filter(|(left, right)| left != right)
                        .count();
                }
            }
        }

        let deduplicated = deduplicate_segment_rows(&segment_rows, measurement, config)?;
        rows_removed += segment_rows.len().saturating_sub(deduplicated.len());
        if deduplicated.len() < config.minimum_segment_points {
            skipped_segments.push(SkippedTimestampSegment {
                original_start_index: *segment_rows.first().unwrap_or(&0),
                original_end_index: *segment_rows.last().unwrap_or(&0),
                point_count: deduplicated.len(),
                reason: format!(
                    "segment shorter than minimum_segment_points ({})",
                    config.minimum_segment_points
                ),
            });
            continue;
        }
        let start_index = final_rows.len();
        final_rows.extend_from_slice(&deduplicated);
        let end_index = final_rows.len();
        let min_timestamp_s = deduplicated
            .first()
            .map(|row| measurement.time[*row])
            .unwrap_or(0.0);
        let max_timestamp_s = deduplicated
            .last()
            .map(|row| measurement.time[*row])
            .unwrap_or(0.0);
        segments.push(TimestampSegment {
            segment_index: segments.len(),
            start_index,
            end_index,
            original_start_index: *deduplicated.first().unwrap_or(&0),
            original_end_index: *deduplicated.last().unwrap_or(&0),
            created_by_reset: created_by_reset || seg_idx > 0,
            point_count: deduplicated.len(),
            min_timestamp_s,
            max_timestamp_s,
        });
    }

    if final_rows.is_empty() {
        return Err("no valid segments remain after timestamp preprocessing".to_string());
    }

    diagnostics.rows_reordered += rows_reordered;
    diagnostics.rows_removed = diagnostics.rows_removed.max(rows_removed);
    diagnostics.segment_count = segments.len();
    if !skipped_segments.is_empty() {
        diagnostics.messages.push(format!(
            "{} segment(s) skipped by minimum length rule",
            skipped_segments.len()
        ));
    }

    let transformed = final_rows
        .iter()
        .enumerate()
        .any(|(idx, original)| idx != *original)
        || !removed_rows.is_empty()
        || rows_reordered > 0
        || segments.len() > 1
        || !skipped_segments.is_empty();

    let measurement = select_rows(measurement, &final_rows)?;
    Ok(PreprocessedMeasurement {
        measurement,
        segments,
        skipped_segments,
        original_indices: final_rows,
        diagnostics,
        applied_policy: config.clone(),
        was_transformed: transformed,
    })
}

pub fn preprocess_timestamps(
    timestamps: &[f64],
    values: &[Vec<Option<f64>>],
    config: &TimestampHandlingConfig,
) -> Result<PreprocessedTimestamps, String> {
    if values.iter().any(|series| series.len() != timestamps.len()) {
        return Err("values length does not match timestamps length".into());
    }
    let channels = values
        .iter()
        .enumerate()
        .map(|(idx, series)| MeasurementChannel {
            name: format!("channel_{idx}"),
            unit: "a.u.".to_string(),
            values: series.clone(),
            variance: None,
            sensor_id: None,
            analyte_id: None,
            metadata: None,
        })
        .collect::<Vec<_>>();
    let measurement = MultiChannelMeasurement::new(timestamps.to_vec(), channels)
        .map_err(|error| error.to_string())?;
    let processed = preprocess_measurement(&measurement, config)?;
    Ok(PreprocessedTimestamps {
        timestamps: processed.measurement.time,
        original_indices: processed.original_indices,
        segments: processed.segments,
        diagnostics: processed.diagnostics,
        was_transformed: processed.was_transformed,
    })
}

fn validate_config(config: &TimestampHandlingConfig) -> Result<(), String> {
    for (name, value) in [
        (
            "minor_reversal_threshold_s",
            config.minor_reversal_threshold_s,
        ),
        ("reset_threshold_s", config.reset_threshold_s),
        ("reset_threshold_fraction", config.reset_threshold_fraction),
    ] {
        if !value.is_finite() {
            return Err(format!("{name} must be finite"));
        }
    }
    if config.minor_reversal_threshold_s < 0.0 || config.reset_threshold_s < 0.0 {
        return Err("timestamp thresholds must be nonnegative".to_string());
    }
    if !(0.0..=1.0).contains(&config.reset_threshold_fraction) {
        return Err("reset_threshold_fraction must be between 0 and 1".to_string());
    }
    if config.minimum_segment_points == 0 {
        return Err("minimum_segment_points must be greater than zero".to_string());
    }
    Ok(())
}

fn detect_segment_boundaries(
    time: &[f64],
    rows: &[usize],
    config: &TimestampHandlingConfig,
) -> Vec<(usize, usize, bool)> {
    if rows.is_empty() {
        return Vec::new();
    }
    let mut bounds = Vec::new();
    let mut start = 0usize;
    for i in 1..rows.len() {
        let prev = time[rows[i - 1]];
        let curr = time[rows[i]];
        if curr >= prev {
            continue;
        }
        let backward = prev - curr;
        let is_reset =
            backward >= config.reset_threshold_s || curr <= prev * config.reset_threshold_fraction;
        if is_reset {
            bounds.push((start, i, !bounds.is_empty()));
            start = i;
        }
    }
    bounds.push((start, rows.len(), !bounds.is_empty()));
    bounds
}

fn deduplicate_segment_rows(
    segment_rows: &[usize],
    measurement: &MultiChannelMeasurement,
    config: &TimestampHandlingConfig,
) -> Result<Vec<usize>, String> {
    if segment_rows.is_empty() {
        return Ok(Vec::new());
    }

    let mut output = Vec::with_capacity(segment_rows.len());
    let mut i = 0usize;
    while i < segment_rows.len() {
        let row = segment_rows[i];
        let t = measurement.time[row];
        let mut j = i + 1;
        while j < segment_rows.len() && measurement.time[segment_rows[j]].to_bits() == t.to_bits() {
            j += 1;
        }
        let group = &segment_rows[i..j];
        if group.len() == 1 {
            output.push(row);
            i = j;
            continue;
        }

        match config.duplicate_policy {
            DuplicatePolicy::Reject => {
                return Err(format!(
                    "duplicate timestamp {} encountered and duplicate_policy=reject",
                    t
                ));
            }
            DuplicatePolicy::Keep => {
                return Err(
                    "duplicate_policy=keep is not allowed for state estimation because filters require strictly positive Δt"
                        .to_string(),
                );
            }
            DuplicatePolicy::DeduplicateIdentical => {
                if !rows_identical(
                    group,
                    &measurement
                        .channels
                        .iter()
                        .map(|channel| channel.values.clone())
                        .collect::<Vec<_>>(),
                ) || !variance_rows_identical(group, measurement)
                {
                    return Err(format!(
                        "conflicting duplicate timestamp {} encountered; cannot deduplicate safely",
                        t
                    ));
                }
                output.push(group[0]);
            }
        }
        i = j;
    }

    for pair in output.windows(2) {
        if measurement.time[pair[1]] <= measurement.time[pair[0]] {
            return Err("segment is not strictly increasing after preprocessing".to_string());
        }
    }
    Ok(output)
}

fn row_value_signature(values: &[Vec<Option<f64>>], row: usize) -> Vec<Option<u64>> {
    values
        .iter()
        .map(|series| series.get(row).and_then(|value| value.map(f64::to_bits)))
        .collect()
}

fn rows_identical(rows: &[usize], values: &[Vec<Option<f64>>]) -> bool {
    if rows.len() <= 1 {
        return true;
    }
    let base = row_value_signature(values, rows[0]);
    rows.iter()
        .skip(1)
        .all(|row| row_value_signature(values, *row) == base)
}

fn variance_rows_identical(rows: &[usize], measurement: &MultiChannelMeasurement) -> bool {
    if rows.len() <= 1 {
        return true;
    }
    for channel in &measurement.channels {
        let Some(variance) = channel.variance.as_ref() else {
            continue;
        };
        let first = variance.get(rows[0]).copied().flatten().map(f64::to_bits);
        for row in rows.iter().skip(1) {
            let current = variance.get(*row).copied().flatten().map(f64::to_bits);
            if current != first {
                return false;
            }
        }
    }
    true
}

fn select_rows(
    measurement: &MultiChannelMeasurement,
    rows: &[usize],
) -> Result<MultiChannelMeasurement, String> {
    let time = rows
        .iter()
        .map(|idx| measurement.time[*idx])
        .collect::<Vec<_>>();
    let channels = measurement
        .channels
        .iter()
        .map(|channel| MeasurementChannel {
            name: channel.name.clone(),
            unit: channel.unit.clone(),
            values: rows.iter().map(|idx| channel.values[*idx]).collect(),
            variance: channel
                .variance
                .as_ref()
                .map(|variance| rows.iter().map(|idx| variance[*idx]).collect()),
            sensor_id: channel.sensor_id.clone(),
            analyte_id: channel.analyte_id.clone(),
            metadata: channel.metadata.clone(),
        })
        .collect::<Vec<_>>();
    MultiChannelMeasurement::new(time, channels).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measurement(time: Vec<f64>, values: Vec<Option<f64>>) -> MultiChannelMeasurement {
        MultiChannelMeasurement::new(time, vec![MeasurementChannel::new("E1", "V", values)])
            .expect("measurement")
    }

    #[test]
    fn diagnoses_identical_vs_conflicting_duplicates() {
        let diag = diagnose_timestamps(&[0.0, 1.0, 1.0], &[vec![Some(0.1), Some(0.2), Some(0.2)]]);
        assert_eq!(diag.duplicate_count, 1);
        assert_eq!(diag.identical_duplicate_count, 1);
        assert_eq!(diag.conflicting_duplicate_count, 0);

        let diag_conflict =
            diagnose_timestamps(&[0.0, 1.0, 1.0], &[vec![Some(0.1), Some(0.2), Some(0.3)]]);
        assert_eq!(diag_conflict.conflicting_duplicate_count, 1);
    }

    #[test]
    fn preprocesses_reset_into_segments() {
        let m = measurement(
            vec![0.0, 1.0, 2.0, 0.1, 1.1, 2.1],
            vec![
                Some(0.1),
                Some(0.2),
                Some(0.3),
                Some(0.4),
                Some(0.5),
                Some(0.6),
            ],
        );
        let config = TimestampHandlingConfig {
            minimum_segment_points: 2,
            ..Default::default()
        };
        let processed = preprocess_measurement(&m, &config).expect("processed");
        assert_eq!(processed.segments.len(), 2);
        assert_eq!(processed.measurement.time.len(), 6);
    }

    #[test]
    fn rejects_conflicting_duplicates_when_deduplicating() {
        let m = measurement(
            vec![0.0, 1.0, 1.0, 2.0],
            vec![Some(0.1), Some(0.2), Some(0.3), Some(0.4)],
        );
        let config = TimestampHandlingConfig {
            minimum_segment_points: 2,
            ..Default::default()
        };
        let err = preprocess_measurement(&m, &config).expect_err("must reject conflicts");
        assert!(err.contains("conflicting duplicate"));
    }
}
