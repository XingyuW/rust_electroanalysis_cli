//! Diagnostics produced while converting source rows into scientific data.

use super::measurement::MultiChannelMeasurement;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Row-level and time-axis diagnostics from a measurement parser.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseDiagnostics {
    /// Number of non-empty data rows considered after the source header.
    pub total_rows: usize,
    /// Rows with a valid timestamp; rows with missing channel cells are still
    /// counted here because their missing values are retained explicitly.
    pub successfully_parsed_rows: usize,
    /// Rows that could not be represented and were not included in the result.
    pub skipped_rows: usize,
    /// Rows containing malformed timestamps, malformed values, or incomplete
    /// source structure.
    pub malformed_rows: usize,
    /// Number of individual channel cells represented as missing values.
    pub missing_values: usize,
    /// True when adjacent sampling intervals are not consistent.
    pub irregular_sampling: bool,
    /// Number of repeated timestamp occurrences.
    pub duplicate_timestamps: usize,
    /// Number of timestamps that move backwards relative to the prior row.
    pub non_monotonic_timestamps: usize,
    /// Additional row or source diagnostics suitable for logs and reports.
    pub messages: Vec<String>,
}

impl ParseDiagnostics {
    pub fn from_measurement(measurement: &MultiChannelMeasurement) -> Self {
        let mut diagnostics = Self {
            total_rows: measurement.time.len(),
            successfully_parsed_rows: measurement.time.len(),
            missing_values: measurement.missing_value_count(),
            ..Self::default()
        };
        diagnostics.update_time_axis(&measurement.time);
        diagnostics
    }

    pub fn update_time_axis(&mut self, time: &[f64]) {
        self.duplicate_timestamps = 0;
        self.non_monotonic_timestamps = 0;
        self.irregular_sampling = false;

        let mut seen_timestamps = HashSet::new();
        for timestamp in time {
            if !seen_timestamps.insert(timestamp.to_bits()) {
                self.duplicate_timestamps += 1;
                self.irregular_sampling = true;
            }
        }

        let mut intervals = Vec::new();
        for pair in time.windows(2) {
            let delta = pair[1] - pair[0];
            if delta == 0.0 {
                self.irregular_sampling = true;
            } else if delta < 0.0 {
                self.non_monotonic_timestamps += 1;
                self.irregular_sampling = true;
            } else if delta.is_finite() {
                intervals.push(delta);
            }
        }

        if let Some(reference) = intervals.first().copied() {
            let tolerance = reference.abs().max(1.0) * 1e-9;
            self.irregular_sampling |= intervals
                .iter()
                .any(|interval| (interval - reference).abs() > tolerance);
        }
    }

    pub fn has_issues(&self) -> bool {
        self.skipped_rows > 0
            || self.malformed_rows > 0
            || self.missing_values > 0
            || self.irregular_sampling
            || self.duplicate_timestamps > 0
            || self.non_monotonic_timestamps > 0
    }
}

/// A parsed measurement together with diagnostics that must remain visible to
/// callers instead of being silently discarded by compatibility parsers.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementParseResult {
    pub measurement: MultiChannelMeasurement,
    pub diagnostics: ParseDiagnostics,
}
