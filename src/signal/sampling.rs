use super::{error::SignalError, statistics};
use crate::{
    results::SamplingAnalysis,
    signal_config::{SamplingConfig, SamplingPolicy},
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
    let mut duplicates = 0;
    let mut nonmono = 0;
    let mut intervals = Vec::new();
    for pair in time.windows(2) {
        let d = pair[1] - pair[0];
        if d == 0.0 {
            duplicates += 1;
        }
        if d <= 0.0 {
            nonmono += 1;
        } else {
            intervals.push(d);
        }
    }
    let mean = statistics::mean(&intervals);
    let sd = statistics::stddev(&intervals);
    let median = if intervals.is_empty() {
        None
    } else {
        let mut v = intervals.clone();
        statistics::median(&mut v)
    };
    let regular = median.is_some_and(|m| {
        intervals
            .iter()
            .all(|d| ((d - m) / m).abs() <= config.regularity_relative_tolerance)
    });
    let finite_count = values.iter().flatten().filter(|v| v.is_finite()).count();
    let missing = values.len().saturating_sub(finite_count);
    let duration = time.first().zip(time.last()).map(|(a, b)| (b - a).abs());
    let target = config.resample_interval_s.or(median);
    let mut analysis = SamplingAnalysis {
        sample_count: time.len(),
        finite_sample_count: finite_count,
        missing_fraction: (!values.is_empty()).then_some(missing as f64 / values.len() as f64),
        start_time_s: time.first().copied(),
        end_time_s: time.last().copied(),
        duration_s: duration,
        median_interval_s: median,
        mean_interval_s: mean,
        interval_stddev_s: sd,
        interval_cv: sd.zip(mean).and_then(|(s, m)| (m > 0.0).then_some(s / m)),
        minimum_interval_s: intervals.iter().copied().reduce(f64::min),
        maximum_interval_s: intervals.iter().copied().reduce(f64::max),
        duplicate_timestamps: duplicates,
        non_monotonic_timestamps: nonmono,
        effective_frequency_hz: median.and_then(|v| (v > 0.0).then_some(1.0 / v)),
        is_regular: regular && duplicates == 0 && nonmono == 0,
        target_interval_s: None,
        interpolation_count: 0,
        interpolation_gap_exceeded: false,
        interpolated_indices: Vec::new(),
    };
    if matches!(config.policy, SamplingPolicy::RequireRegular) && !analysis.is_regular {
        return Err(SignalError::Sampling("regular sampling is required".into()));
    }
    if matches!(config.policy, SamplingPolicy::ResampleLinear) && !analysis.is_regular {
        let (new_time, new_values, indices, gap) = resample_linear(
            time,
            values,
            target.ok_or_else(|| {
                SignalError::Sampling("resampling interval is unavailable".into())
            })?,
            config.maximum_interpolation_gap_s,
        )?;
        analysis.target_interval_s = target;
        analysis.interpolation_count = indices.len();
        analysis.interpolated_indices = indices;
        analysis.interpolation_gap_exceeded = gap;
        analysis.is_regular = true;
        return Ok((analysis, new_time, new_values));
    }
    Ok((analysis, time.to_vec(), values.to_vec()))
}

#[allow(clippy::type_complexity)]
fn resample_linear(
    time: &[f64],
    values: &[Option<f64>],
    step: f64,
    max_gap: f64,
) -> Result<(Vec<f64>, Vec<Option<f64>>, Vec<usize>, bool), SignalError> {
    if step.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return Err(SignalError::invalid("resampling interval must be positive"));
    }
    let start = *time
        .first()
        .ok_or_else(|| SignalError::invalid("empty time axis"))?;
    let end = *time
        .last()
        .ok_or_else(|| SignalError::invalid("empty time axis"))?;
    let n = ((end - start) / step).floor() as usize + 1;
    let mut t = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    let mut interp = Vec::new();
    let mut gap_exceeded = false;
    for k in 0..n {
        let q = start + k as f64 * step;
        t.push(q);
        if let Some(i) = time.iter().position(|v| (*v - q).abs() < step * 1e-9) {
            y.push(values[i]);
            continue;
        }
        let right = time.iter().position(|v| *v > q);
        let Some(j) = right else {
            y.push(None);
            continue;
        };
        if j == 0 {
            y.push(None);
            continue;
        }
        let i = j - 1;
        let gap = time[j] - time[i];
        if gap > max_gap || values[i].is_none() || values[j].is_none() {
            gap_exceeded |= gap > max_gap;
            y.push(None);
            continue;
        }
        let f = (q - time[i]) / gap;
        y.push(Some(
            values[i].unwrap() + (values[j].unwrap() - values[i].unwrap()) * f,
        ));
        interp.push(k);
    }
    Ok((t, y, interp, gap_exceeded))
}
