use super::{error::SignalError, statistics};
use crate::{
    results::SamplingAnalysis,
    signal_config::{
        DuplicateTimestampPolicy, NonMonotonicTimestampPolicy, SamplingConfig, SamplingPolicy,
    },
};

#[allow(clippy::type_complexity)]
pub fn analyze_sampling(
    time: &[f64],
    values: &[Option<f64>],
    config: &SamplingConfig,
) -> Result<(SamplingAnalysis, Vec<f64>, Vec<Option<f64>>), SignalError> {
    if time.len() != values.len() {
        return Err(SignalError::invalid(
            "time and values have different lengths",
        ));
    }
    if time.is_empty() {
        return Err(SignalError::invalid("empty time axis"));
    }
    if time.iter().any(|value| !value.is_finite()) {
        return Err(SignalError::Sampling(
            "timestamps must be finite before sampling analysis".into(),
        ));
    }

    let nonmonotonic_count = time.windows(2).filter(|pair| pair[1] < pair[0]).count();
    if nonmonotonic_count > 0
        && matches!(
            config.non_monotonic_timestamp_policy,
            NonMonotonicTimestampPolicy::Error
        )
    {
        return Err(SignalError::Sampling(
            "non-monotonic timestamps require SortPaired policy".into(),
        ));
    }
    let mut rows = time
        .iter()
        .copied()
        .zip(values.iter().copied())
        .enumerate()
        .map(|(original_index, (timestamp, value))| (timestamp, value, original_index))
        .collect::<Vec<_>>();
    let mut transformations = Vec::new();
    let mut sorted_rows = 0;
    if nonmonotonic_count > 0 {
        // `sort_by` is stable, so First and Last remain deterministic for a
        // duplicate group while the timestamp/value pairing is preserved.
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        sorted_rows = rows
            .iter()
            .enumerate()
            .filter(|(new_index, row)| *new_index != row.2)
            .count();
        transformations.push(format!(
            "sorted {sorted_rows} paired rows using non_monotonic_timestamp_policy"
        ));
    }

    let mut resolved_duplicate_groups = 0;
    let mut duplicate_count = 0;
    let mut unique_rows = Vec::with_capacity(rows.len());
    let mut index = 0;
    while index < rows.len() {
        let mut end = index + 1;
        while end < rows.len() && rows[end].0 == rows[index].0 {
            end += 1;
        }
        let group = &rows[index..end];
        if group.len() > 1 {
            duplicate_count += group.len() - 1;
            if matches!(
                config.duplicate_timestamp_policy,
                DuplicateTimestampPolicy::Error
            ) {
                return Err(SignalError::Sampling(format!(
                    "duplicate timestamp group at {} contains {} rows",
                    group[0].0,
                    group.len()
                )));
            }
            resolved_duplicate_groups += 1;
            let value = match config.duplicate_timestamp_policy {
                DuplicateTimestampPolicy::Average => {
                    let finite = group
                        .iter()
                        .filter_map(|(_, value, _)| *value)
                        .collect::<Vec<_>>();
                    if finite.is_empty() {
                        None
                    } else {
                        Some(finite.iter().sum::<f64>() / finite.len() as f64)
                    }
                }
                DuplicateTimestampPolicy::First => group.first().and_then(|(_, value, _)| *value),
                DuplicateTimestampPolicy::Last => group.last().and_then(|(_, value, _)| *value),
                DuplicateTimestampPolicy::Error => {
                    return Err(SignalError::Sampling(
                        "duplicate timestamp policy rejected a duplicate group".into(),
                    ));
                }
            };
            unique_rows.push((group[0].0, value));
            transformations.push(format!(
                "resolved duplicate timestamp group at {} with {:?}",
                group[0].0, config.duplicate_timestamp_policy
            ));
        } else {
            unique_rows.push((group[0].0, group[0].1));
        }
        index = end;
    }
    if resolved_duplicate_groups > 0 {
        transformations.push(format!(
            "resolved {resolved_duplicate_groups} duplicate timestamp groups"
        ));
    }
    if unique_rows.windows(2).any(|pair| pair[1].0 <= pair[0].0) {
        return Err(SignalError::Sampling(
            "timestamps are not strictly increasing after policy application".into(),
        ));
    }

    let input_intervals = unique_rows
        .windows(2)
        .map(|pair| pair[1].0 - pair[0].0)
        .collect::<Vec<_>>();
    let mean = statistics::mean(&input_intervals);
    let sd = statistics::stddev(&input_intervals);
    let median = if input_intervals.is_empty() {
        None
    } else {
        let mut intervals = input_intervals.clone();
        statistics::median(&mut intervals)
    };
    let input_regular = median.is_some_and(|m| {
        m > 0.0
            && input_intervals
                .iter()
                .all(|d| ((d - m) / m).abs() <= config.regularity_relative_tolerance)
    });
    let finite_count = unique_rows
        .iter()
        .filter_map(|(_, value)| *value)
        .filter(|value| value.is_finite())
        .count();
    let missing = unique_rows.len().saturating_sub(finite_count);
    let target = config.resample_interval_s.or(median);
    let mut analysis = SamplingAnalysis {
        sample_count: unique_rows.len(),
        finite_sample_count: finite_count,
        missing_fraction: (!unique_rows.is_empty())
            .then_some(missing as f64 / unique_rows.len() as f64),
        start_time_s: unique_rows.first().map(|row| row.0),
        end_time_s: unique_rows.last().map(|row| row.0),
        duration_s: unique_rows
            .first()
            .zip(unique_rows.last())
            .map(|(a, b)| b.0 - a.0),
        median_interval_s: median,
        mean_interval_s: mean,
        interval_stddev_s: sd,
        interval_cv: sd.zip(mean).and_then(|(s, m)| (m > 0.0).then_some(s / m)),
        minimum_interval_s: input_intervals.iter().copied().reduce(f64::min),
        maximum_interval_s: input_intervals.iter().copied().reduce(f64::max),
        duplicate_timestamps: duplicate_count,
        non_monotonic_timestamps: nonmonotonic_count,
        effective_frequency_hz: median.and_then(|value| (value > 0.0).then_some(1.0 / value)),
        is_regular: input_regular,
        target_interval_s: None,
        interpolation_count: 0,
        interpolation_gap_exceeded: false,
        interpolated_indices: Vec::new(),
        output_missing_indices: Vec::new(),
        sorted_rows,
        resolved_duplicate_groups,
        transformations,
    };

    if matches!(config.policy, SamplingPolicy::RequireRegular) && !analysis.is_regular {
        return Err(SignalError::Sampling("regular sampling is required".into()));
    }
    if matches!(config.policy, SamplingPolicy::ResampleLinear) && !analysis.is_regular {
        let step = target
            .ok_or_else(|| SignalError::Sampling("resampling interval is unavailable".into()))?;
        let (new_time, new_values, interpolated, missing_indices, gap) =
            resample_linear(&unique_rows, step, config.maximum_interpolation_gap_s)?;
        analysis.target_interval_s = Some(step);
        analysis.interpolation_count = interpolated.len();
        analysis.interpolated_indices = interpolated;
        analysis.output_missing_indices = missing_indices;
        analysis.interpolation_gap_exceeded = gap;
        analysis.is_regular = is_regular_grid(&new_time, config.regularity_relative_tolerance);
        analysis.transformations.push(format!(
            "resampled output on regular grid with interval {step}"
        ));
        return Ok((analysis, new_time, new_values));
    }
    Ok((
        analysis,
        unique_rows.iter().map(|row| row.0).collect(),
        unique_rows.iter().map(|row| row.1).collect(),
    ))
}

fn is_regular_grid(time: &[f64], tolerance: f64) -> bool {
    let Some(step) = time.windows(2).next().map(|pair| pair[1] - pair[0]) else {
        return true;
    };
    step > 0.0
        && time
            .windows(2)
            .all(|pair| ((pair[1] - pair[0] - step) / step).abs() <= tolerance)
}

#[allow(clippy::type_complexity)]
fn resample_linear(
    rows: &[(f64, Option<f64>)],
    step: f64,
    max_gap: f64,
) -> Result<(Vec<f64>, Vec<Option<f64>>, Vec<usize>, Vec<usize>, bool), SignalError> {
    if !step.is_finite() || step <= 0.0 {
        return Err(SignalError::invalid(
            "resampling interval must be positive and finite",
        ));
    }
    if !max_gap.is_finite() || max_gap < 0.0 {
        return Err(SignalError::invalid(
            "maximum interpolation gap must be finite and non-negative",
        ));
    }
    let start = rows
        .first()
        .map(|row| row.0)
        .ok_or_else(|| SignalError::invalid("empty time axis"))?;
    let end = rows.last().map(|row| row.0).unwrap_or(start);
    let count = ((end - start) / step).floor() as usize + 1;
    let mut output_time = Vec::with_capacity(count);
    let mut output_values = Vec::with_capacity(count);
    let mut interpolated = Vec::new();
    let mut missing_indices = Vec::new();
    let mut gap_exceeded = false;
    let mut right = 0usize;
    let tolerance = step.abs() * 1e-9;

    for output_index in 0..count {
        let query = start + output_index as f64 * step;
        output_time.push(query);
        while right < rows.len() && rows[right].0 < query - tolerance {
            right += 1;
        }
        if right < rows.len() && (rows[right].0 - query).abs() <= tolerance {
            if rows[right].1.is_none() {
                missing_indices.push(output_index);
            }
            output_values.push(rows[right].1);
            continue;
        }
        if right == 0 || right >= rows.len() {
            output_values.push(None);
            missing_indices.push(output_index);
            continue;
        }
        let left = right - 1;
        let gap = rows[right].0 - rows[left].0;
        if gap > max_gap || rows[left].1.is_none() || rows[right].1.is_none() {
            if gap > max_gap {
                gap_exceeded = true;
            }
            output_values.push(None);
            missing_indices.push(output_index);
            continue;
        }
        let fraction = (query - rows[left].0) / gap;
        let value =
            rows[left].1.unwrap() + (rows[right].1.unwrap() - rows[left].1.unwrap()) * fraction;
        output_values.push(Some(value));
        interpolated.push(output_index);
    }
    Ok((
        output_time,
        output_values,
        interpolated,
        missing_indices,
        gap_exceeded,
    ))
}

#[cfg(test)]
mod tests {
    use super::analyze_sampling;
    use crate::signal_config::{
        DuplicateTimestampPolicy, NonMonotonicTimestampPolicy, SamplingConfig, SamplingPolicy,
    };

    #[test]
    fn sorts_paired_rows_and_averages_duplicates_without_mutating_source() {
        let time = vec![2.0, 0.0, 1.0, 1.0];
        let values = vec![Some(20.0), Some(0.0), Some(8.0), Some(12.0)];
        let original_time = time.clone();
        let original_values = values.clone();
        let config = SamplingConfig {
            policy: SamplingPolicy::ResampleLinear,
            non_monotonic_timestamp_policy: NonMonotonicTimestampPolicy::SortPaired,
            duplicate_timestamp_policy: DuplicateTimestampPolicy::Average,
            resample_interval_s: Some(1.0),
            ..Default::default()
        };
        let (analysis, output_time, output_values) =
            analyze_sampling(&time, &values, &config).unwrap();
        assert_eq!(time, original_time);
        assert_eq!(values, original_values);
        assert_eq!(output_time, vec![0.0, 1.0, 2.0]);
        assert_eq!(output_values, vec![Some(0.0), Some(10.0), Some(20.0)]);
        assert_eq!(analysis.sorted_rows, 4);
        assert_eq!(analysis.resolved_duplicate_groups, 1);
        assert!(analysis.is_regular);
        assert!(!analysis.transformations.is_empty());
    }

    #[test]
    fn duplicate_error_and_nonfinite_timestamp_are_rejected() {
        let error = analyze_sampling(
            &[0.0, 0.0],
            &[Some(1.0), Some(2.0)],
            &SamplingConfig {
                policy: SamplingPolicy::ResampleLinear,
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("duplicate"));
        let error = analyze_sampling(
            &[0.0, f64::NAN],
            &[Some(1.0), Some(2.0)],
            &SamplingConfig::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("finite"));
    }

    #[test]
    fn interpolation_gap_is_missing_and_recorded() {
        let config = SamplingConfig {
            policy: SamplingPolicy::ResampleLinear,
            resample_interval_s: Some(1.0),
            maximum_interpolation_gap_s: 1.5,
            ..Default::default()
        };
        let (analysis, time, values) = analyze_sampling(
            &[0.0, 1.0, 4.0],
            &[Some(0.0), Some(1.0), Some(4.0)],
            &config,
        )
        .unwrap();
        assert_eq!(time, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
        assert_eq!(values[2], None);
        assert_eq!(values[3], None);
        assert!(analysis.interpolation_gap_exceeded);
        assert_eq!(analysis.output_missing_indices, vec![2, 3]);
        assert_eq!(analysis.interpolation_count, 0);
    }

    #[test]
    fn output_grid_is_strictly_increasing() {
        let config = SamplingConfig {
            policy: SamplingPolicy::ResampleLinear,
            resample_interval_s: Some(0.5),
            ..Default::default()
        };
        let (_, time, _) = analyze_sampling(
            &[0.0, 0.7, 1.4],
            &[Some(0.0), Some(0.7), Some(1.4)],
            &config,
        )
        .unwrap();
        assert!(time.windows(2).all(|p| p[1] > p[0]));
    }
}
