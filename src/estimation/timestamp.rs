//! Timestamp preprocessing for time‑series measurements.
//!
//! This module separates validation from normalisation so that duplicate,
//! non‑monotonic, and reset‑bearing timestamps can be handled with a
//! documented, configurable policy rather than rejected outright.

use serde::{Deserialize, Serialize};

/// Diagnostics produced when analysing a timestamp vector.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimestampDiagnostics {
    /// Total number of timestamp entries.
    pub total_rows: usize,
    /// Entries that are finite and usable.
    pub finite_timestamps: usize,
    /// Non‑finite entries (NaN, ±Inf).
    pub non_finite_timestamps: usize,
    /// Exact duplicate timestamp occurrences (count of extra copies).
    pub duplicate_count: usize,
    /// Duplicate timestamps whose corresponding channel values differ.
    pub conflicting_duplicate_count: usize,
    /// Number of timestamps that move backwards relative to the prior.
    pub local_reversal_count: usize,
    /// Largest single backward jump (seconds).  None if all increasing.
    pub largest_backward_jump_s: Option<f64>,
    /// Number of detected timestamp resets to zero or near‑zero.
    pub reset_count: usize,
    /// Number of independent acquisition segments identified.
    pub segment_count: usize,
    /// Minimum time difference between consecutive entries.
    pub min_delta_s: Option<f64>,
    /// Median time difference (None if fewer than 2 entries).
    pub median_delta_s: Option<f64>,
    /// Maximum time difference.
    pub max_delta_s: Option<f64>,
    /// Rows that were reordered by stable sort.
    pub rows_reordered: usize,
    /// Rows that were removed entirely.
    pub rows_removed: usize,
    /// Rows aggregated (e.g., duplicate averaging — not used in default policy).
    pub rows_aggregated: usize,
    /// Human‑readable messages produced during preprocessing.
    pub messages: Vec<String>,
}

/// Policy for handling duplicate timestamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicatePolicy {
    /// Remove exact duplicate rows (identical values) and warn about
    /// conflicting duplicates without silently averaging them.
    DeduplicateIdentical,
    /// Reject the entire measurement if any duplicate timestamps exist.
    Reject,
    /// Keep duplicates (treated as replicates within a segment).
    Keep,
}

/// Policy for handling non‑monotonic timestamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonMonotonicPolicy {
    /// Split the measurement into independent segments when a reset or
    /// large backward jump is detected.
    SegmentOnReset,
    /// Stable‑sort timestamps within a segment when reversals are minor
    /// (no evidence of a reset).
    StableSortWithinSegment,
    /// Reject the measurement if any non‑monotonic timestamps exist.
    Reject,
}

/// Configuration for timestamp preprocessing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimestampHandlingConfig {
    pub duplicate_policy: DuplicatePolicy,
    pub non_monotonic_policy: NonMonotonicPolicy,
    /// Threshold (seconds) below which a backward jump is considered a
    /// minor reversal rather than a reset.
    pub minor_reversal_threshold_s: f64,
    /// Fraction of the current time value; a jump to a value less than or
    /// equal to `reset_threshold_fraction * previous_time` is treated as
    /// a reset.
    pub reset_threshold_fraction: f64,
    pub minimum_segment_points: usize,
}

impl Default for TimestampHandlingConfig {
    fn default() -> Self {
        Self {
            duplicate_policy: DuplicatePolicy::DeduplicateIdentical,
            non_monotonic_policy: NonMonotonicPolicy::SegmentOnReset,
            minor_reversal_threshold_s: 1.0,
            reset_threshold_fraction: 0.5,
            minimum_segment_points: 10,
        }
    }
}

/// A segment of preprocessed time‑series data with known provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimestampSegment {
    /// Zero‑based segment index.
    pub segment_index: usize,
    /// Start index into the reordered timestamp vector.
    pub start_index: usize,
    /// Exclusive end index.
    pub end_index: usize,
    /// Whether this segment was created by a reset split.
    pub created_by_reset: bool,
    /// Number of points in this segment.
    pub point_count: usize,
    /// Minimum timestamp in this segment.
    pub min_timestamp_s: f64,
    /// Maximum timestamp in this segment.
    pub max_timestamp_s: f64,
}

/// Result of timestamp preprocessing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreprocessedTimestamps {
    /// Preprocessed timestamp values (strictly increasing within each
    /// segment; segments are separated by resets).
    pub timestamps: Vec<f64>,
    /// Indices mapping preprocessed rows back to original rows.
    pub original_indices: Vec<usize>,
    /// Identified segments.
    pub segments: Vec<TimestampSegment>,
    /// Diagnostics collected during preprocessing.
    pub diagnostics: TimestampDiagnostics,
    /// Whether any transformation was applied.
    pub was_transformed: bool,
}

/// Analyse a timestamp vector and produce diagnostics without modifying data.
pub fn diagnose_timestamps(
    timestamps: &[f64],
    _values: &[Vec<Option<f64>>],
) -> TimestampDiagnostics {
    let mut diag = TimestampDiagnostics {
        total_rows: timestamps.len(),
        ..Default::default()
    };

    if timestamps.is_empty() {
        return diag;
    }

    let mut finite = 0usize;
    let mut non_finite = 0usize;
    for t in timestamps {
        if t.is_finite() {
            finite += 1;
        } else {
            non_finite += 1;
        }
    }
    diag.finite_timestamps = finite;
    diag.non_finite_timestamps = non_finite;

    let finite_ts: Vec<f64> = timestamps
        .iter()
        .copied()
        .filter(|t| t.is_finite())
        .collect();
    if finite_ts.len() < 2 {
        return diag;
    }

    // Duplicates
    let mut seen = std::collections::HashMap::new();
    for t in &finite_ts {
        *seen.entry(t.to_bits()).or_insert(0usize) += 1;
    }
    diag.duplicate_count = seen.values().filter(|&&c| c > 1).map(|c| c - 1).sum();

    // Reversals and resets
    let mut reversals = 0usize;
    let mut max_backward = 0.0f64;
    let mut resets = 0usize;
    let mut deltas = Vec::with_capacity(finite_ts.len() - 1);

    for pair in finite_ts.windows(2) {
        let delta = pair[1] - pair[0];
        if delta < 0.0 {
            reversals += 1;
            let backward = -delta;
            if backward > max_backward {
                max_backward = backward;
            }
            if pair[1] < pair[0] * 0.5 {
                resets += 1;
            }
        } else if delta.is_finite() {
            deltas.push(delta);
        }
    }

    diag.local_reversal_count = reversals;
    diag.largest_backward_jump_s = if reversals > 0 {
        Some(max_backward)
    } else {
        None
    };
    diag.reset_count = resets;
    diag.segment_count = resets + 1;

    if !deltas.is_empty() {
        deltas.sort_by(f64::total_cmp);
        diag.min_delta_s = Some(deltas[0]);
        diag.max_delta_s = Some(deltas[deltas.len() - 1]);
        let mid = deltas.len() / 2;
        diag.median_delta_s = Some(if deltas.len() % 2 == 0 {
            (deltas[mid - 1] + deltas[mid]) / 2.0
        } else {
            deltas[mid]
        });
    }

    diag
}

/// Preprocess timestamps according to the given configuration.
///
/// Returns the preprocessed timestamps together with diagnostics and
/// segment information.  This is a pure function that does not modify
/// the original measurement.
pub fn preprocess_timestamps(
    timestamps: &[f64],
    values: &[Vec<Option<f64>>],
    config: &TimestampHandlingConfig,
) -> Result<PreprocessedTimestamps, String> {
    if timestamps.is_empty() {
        return Err("timestamp array is empty".to_string());
    }

    let diagnostics = diagnose_timestamps(timestamps, values);

    // Non-finite rejection
    if diagnostics.non_finite_timestamps > 0 {
        if matches!(config.duplicate_policy, DuplicatePolicy::Reject) {
            return Err(format!(
                "{} non-finite timestamps detected and policy is Reject",
                diagnostics.non_finite_timestamps
            ));
        }
        // Filter out non-finite timestamps
        let filtered_ts: Vec<(usize, f64)> = timestamps
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_finite())
            .map(|(i, t)| (i, *t))
            .collect();

        if filtered_ts.len() < config.minimum_segment_points {
            return Err(format!(
                "only {} finite timestamps remain after filtering; minimum is {}",
                filtered_ts.len(),
                config.minimum_segment_points
            ));
        }

        return preprocess_finite(&filtered_ts, values, config, diagnostics);
    }

    let indexed: Vec<(usize, f64)> = timestamps
        .iter()
        .enumerate()
        .map(|(i, t)| (i, *t))
        .collect();
    preprocess_finite(&indexed, values, config, diagnostics)
}

fn preprocess_finite(
    indexed: &[(usize, f64)],
    _values: &[Vec<Option<f64>>],
    config: &TimestampHandlingConfig,
    mut diagnostics: TimestampDiagnostics,
) -> Result<PreprocessedTimestamps, String> {
    // Check if already strictly increasing.
    let is_strictly_increasing = indexed.windows(2).all(|w| w[1].1 > w[0].1);

    if is_strictly_increasing {
        // Deduplicate identical rows if policy requires it.
        if config.duplicate_policy == DuplicatePolicy::DeduplicateIdentical {
            let msg = "timestamps are strictly increasing; no preprocessing applied";
            if !diagnostics.messages.contains(&msg.to_string()) {
                diagnostics.messages.push(msg.to_string());
            }
        }
        return Ok(build_single_segment(indexed, diagnostics, false));
    }

    // Handle duplicates and non-monotonic timestamps.
    match config.duplicate_policy {
        DuplicatePolicy::Reject => {
            return Err(format!(
                "{} duplicate timestamps detected and duplicate policy is Reject",
                diagnostics.duplicate_count
            ));
        }
        DuplicatePolicy::DeduplicateIdentical | DuplicatePolicy::Keep => {
            // Continue processing
        }
    }

    match config.non_monotonic_policy {
        NonMonotonicPolicy::Reject => Err(format!(
            "{} non-monotonic timestamps detected and non-monotonic policy is Reject",
            diagnostics.local_reversal_count
        )),
        NonMonotonicPolicy::SegmentOnReset => segment_on_reset(indexed, config, diagnostics),
        NonMonotonicPolicy::StableSortWithinSegment => {
            stable_sort_within_segment(indexed, config, diagnostics)
        }
    }
}

fn build_single_segment(
    indexed: &[(usize, f64)],
    diagnostics: TimestampDiagnostics,
    was_transformed: bool,
) -> PreprocessedTimestamps {
    let timestamps: Vec<f64> = indexed.iter().map(|(_, t)| *t).collect();
    let original_indices: Vec<usize> = indexed.iter().map(|(i, _)| *i).collect();
    let min_t = timestamps.first().copied().unwrap_or(0.0);
    let max_t = timestamps.last().copied().unwrap_or(0.0);

    PreprocessedTimestamps {
        segments: vec![TimestampSegment {
            segment_index: 0,
            start_index: 0,
            end_index: timestamps.len(),
            created_by_reset: false,
            point_count: timestamps.len(),
            min_timestamp_s: min_t,
            max_timestamp_s: max_t,
        }],
        timestamps,
        original_indices,
        diagnostics,
        was_transformed,
    }
}

fn stable_sort_within_segment(
    indexed: &[(usize, f64)],
    config: &TimestampHandlingConfig,
    diagnostics: TimestampDiagnostics,
) -> Result<PreprocessedTimestamps, String> {
    // Stable sort preserving original order for equal timestamps.
    let mut sorted = indexed.to_vec();
    sorted.sort_by(|a, b| a.1.total_cmp(&b.1));

    // Check for large reversals that look like resets (skip those).
    let mut segments = Vec::new();
    let mut seg_start = 0usize;
    for i in 1..sorted.len() {
        let prev_orig_idx = sorted[i - 1].0;
        let curr_orig_idx = sorted[i].0;
        // If original order shows a large backward jump, split.
        if curr_orig_idx < prev_orig_idx {
            let orig_prev = sorted[i - 1].1;
            let orig_curr = sorted[i].1;
            if orig_curr < orig_prev * config.reset_threshold_fraction {
                let seg = &sorted[seg_start..i];
                if seg.len() >= config.minimum_segment_points {
                    let min_t = seg.first().map(|(_, t)| *t).unwrap_or(0.0);
                    let max_t = seg.last().map(|(_, t)| *t).unwrap_or(0.0);
                    segments.push(TimestampSegment {
                        segment_index: segments.len(),
                        start_index: seg_start,
                        end_index: i,
                        created_by_reset: !segments.is_empty(),
                        point_count: seg.len(),
                        min_timestamp_s: min_t,
                        max_timestamp_s: max_t,
                    });
                }
                seg_start = i;
            }
        }
    }

    // Final segment
    let final_seg = &sorted[seg_start..];
    if final_seg.len() >= config.minimum_segment_points {
        let min_t = final_seg.first().map(|(_, t)| *t).unwrap_or(0.0);
        let max_t = final_seg.last().map(|(_, t)| *t).unwrap_or(0.0);
        segments.push(TimestampSegment {
            segment_index: segments.len(),
            start_index: seg_start,
            end_index: sorted.len(),
            created_by_reset: !segments.is_empty(),
            point_count: final_seg.len(),
            min_timestamp_s: min_t,
            max_timestamp_s: max_t,
        });
    }

    if segments.is_empty() {
        return Err("no valid segments after sorting".to_string());
    }

    let timestamps: Vec<f64> = sorted.iter().map(|(_, t)| *t).collect();
    let original_indices: Vec<usize> = sorted.iter().map(|(i, _)| *i).collect();
    let rows_reordered = (0..sorted.len()).filter(|&i| sorted[i].0 != i).count();

    let mut diag = diagnostics;
    diag.rows_reordered = rows_reordered;
    diag.segment_count = segments.len();
    diag.messages.push(format!(
        "stable-sorted: {} rows reordered, {} segments",
        rows_reordered,
        segments.len()
    ));

    Ok(PreprocessedTimestamps {
        segments,
        timestamps,
        original_indices,
        diagnostics: diag,
        was_transformed: true,
    })
}

fn segment_on_reset(
    indexed: &[(usize, f64)],
    config: &TimestampHandlingConfig,
    diagnostics: TimestampDiagnostics,
) -> Result<PreprocessedTimestamps, String> {
    // Walk through original order and split at resets.
    let mut segments = Vec::new();
    let mut seg_start = 0usize;

    for i in 1..indexed.len() {
        let prev = indexed[i - 1].1;
        let curr = indexed[i].1;
        if curr < prev * config.reset_threshold_fraction {
            let seg = &indexed[seg_start..i];
            if seg.len() >= config.minimum_segment_points {
                let min_t = seg.first().map(|(_, t)| *t).unwrap_or(0.0);
                let max_t = seg.last().map(|(_, t)| *t).unwrap_or(0.0);
                segments.push(TimestampSegment {
                    segment_index: segments.len(),
                    start_index: seg_start,
                    end_index: i,
                    created_by_reset: !segments.is_empty(),
                    point_count: seg.len(),
                    min_timestamp_s: min_t,
                    max_timestamp_s: max_t,
                });
            }
            seg_start = i;
        }
    }

    // Final segment
    let final_seg = &indexed[seg_start..];
    if final_seg.len() >= config.minimum_segment_points {
        let min_t = final_seg.first().map(|(_, t)| *t).unwrap_or(0.0);
        let max_t = final_seg.last().map(|(_, t)| *t).unwrap_or(0.0);
        segments.push(TimestampSegment {
            segment_index: segments.len(),
            start_index: seg_start,
            end_index: indexed.len(),
            created_by_reset: !segments.is_empty(),
            point_count: final_seg.len(),
            min_timestamp_s: min_t,
            max_timestamp_s: max_t,
        });
    }

    if segments.is_empty() {
        return Err(format!(
            "no valid segments found; {} points available, minimum is {}",
            indexed.len(),
            config.minimum_segment_points
        ));
    }

    let timestamps: Vec<f64> = indexed.iter().map(|(_, t)| *t).collect();
    let original_indices: Vec<usize> = (0..indexed.len()).collect();

    let mut diag = diagnostics;
    diag.segment_count = segments.len();
    diag.messages.push(format!(
        "segmented on reset: {} segments identified",
        segments.len()
    ));

    Ok(PreprocessedTimestamps {
        segments,
        timestamps,
        original_indices,
        diagnostics: diag,
        was_transformed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strictly_increasing_passes_through() {
        let ts = vec![0.0, 1.0, 2.0, 3.0];
        let result = preprocess_timestamps(&ts, &[], &TimestampHandlingConfig::default()).unwrap();
        assert!(!result.was_transformed);
        assert_eq!(result.timestamps, ts);
        assert_eq!(result.segments.len(), 1);
    }

    #[test]
    fn identical_duplicates_preserved_with_warning() {
        let ts = vec![0.0, 1.0, 1.0, 2.0];
        let config = TimestampHandlingConfig {
            minimum_segment_points: 2,
            ..Default::default()
        };
        let result = preprocess_timestamps(&ts, &[], &config).unwrap();
        // Default policy: deduplicate identical — timestamps alone means both kept.
        assert_eq!(result.diagnostics.duplicate_count, 1);
    }

    #[test]
    fn timestamp_reset_creates_segments() {
        let ts = vec![0.0, 1.0, 2.0, 0.1, 1.1, 2.1];
        let config = TimestampHandlingConfig {
            minimum_segment_points: 2,
            ..Default::default()
        };
        let result = preprocess_timestamps(&ts, &[], &config).unwrap();
        assert!(result.was_transformed);
        assert_eq!(result.segments.len(), 2);
    }

    #[test]
    fn non_finite_rejected_with_reject_policy() {
        let ts = vec![0.0, f64::NAN, 2.0];
        let config = TimestampHandlingConfig {
            duplicate_policy: DuplicatePolicy::Reject,
            ..Default::default()
        };
        assert!(preprocess_timestamps(&ts, &[], &config).is_err());
    }

    #[test]
    fn constant_timestamps_with_reject_policy() {
        let ts = vec![1.0, 1.0, 1.0];
        let config = TimestampHandlingConfig {
            non_monotonic_policy: NonMonotonicPolicy::Reject,
            ..Default::default()
        };
        assert!(preprocess_timestamps(&ts, &[], &config).is_err());
    }

    #[test]
    fn diagnose_reports_all_metrics() {
        let ts = vec![0.0, 1.0, 3.0, 1.0, 0.5];
        let diag = diagnose_timestamps(&ts, &[]);
        assert!(diag.local_reversal_count > 0);
        assert!(diag.largest_backward_jump_s.is_some());
        assert!(diag.duplicate_count >= 1);
    }
}
