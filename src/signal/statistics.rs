use crate::results::DescriptiveStatistics;

pub fn finite(values: &[Option<f64>]) -> Vec<f64> {
    values
        .iter()
        .flatten()
        .copied()
        .filter(|v| v.is_finite())
        .collect()
}
pub fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    Some(if n.is_multiple_of(2) {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    } else {
        values[n / 2]
    })
}
pub fn quantile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() || !p.is_finite() {
        return None;
    }
    let p = p.clamp(0.0, 1.0);
    let index = p * (sorted.len() - 1) as f64;
    let low = index.floor() as usize;
    let high = index.ceil() as usize;
    Some(sorted[low] + (sorted[high] - sorted[low]) * (index - low as f64))
}
pub fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}
pub fn stddev(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let m = mean(values)?;
    Some((values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (values.len() - 1) as f64).sqrt())
}
pub fn descriptive(
    values: &[Option<f64>],
    probabilities: &[f64],
    confidence: f64,
) -> DescriptiveStatistics {
    let mut x = finite(values);
    let count = x.len();
    if x.is_empty() {
        return DescriptiveStatistics {
            count,
            quantiles: probabilities
                .iter()
                .map(|p| crate::results::Quantile {
                    probability: *p,
                    value: None,
                })
                .collect(),
            ..empty()
        };
    }
    x.sort_by(f64::total_cmp);
    let mean_v = mean(&x);
    let median_v = quantile(&x, 0.5);
    let sd = stddev(&x);
    let variance = sd.map(|v| v * v);
    let rms = Some((x.iter().map(|v| v * v).sum::<f64>() / count as f64).sqrt());
    let min = x.first().copied();
    let max = x.last().copied();
    let q1 = quantile(&x, 0.25);
    let q3 = quantile(&x, 0.75);
    let mad = median_v.map(|m| {
        let mut deviations = x.iter().map(|v| (v - m).abs()).collect::<Vec<_>>();
        median(&mut deviations).unwrap_or(0.0)
    });
    let robust = mad.map(|v| 1.4826 * v);
    let skew = sd.filter(|v| *v > 0.0).map(|s| {
        x.iter()
            .map(|v| ((v - mean_v.unwrap_or(0.0)) / s).powi(3))
            .sum::<f64>()
            / count as f64
    });
    let kurt = sd.filter(|v| *v > 0.0).map(|s| {
        x.iter()
            .map(|v| ((v - mean_v.unwrap_or(0.0)) / s).powi(4))
            .sum::<f64>()
            / count as f64
            - 3.0
    });
    let ci = sd.map(|s| {
        let z = if confidence >= 0.99 {
            2.576
        } else if confidence >= 0.95 {
            1.96
        } else {
            1.645
        };
        let half = z * s / (count as f64).sqrt();
        (mean_v.unwrap_or(0.0) - half, mean_v.unwrap_or(0.0) + half)
    });
    DescriptiveStatistics {
        count,
        mean: mean_v,
        median: median_v,
        standard_deviation: sd,
        sample_variance: variance,
        rms,
        peak_to_peak: min.zip(max).map(|(a, b)| b - a),
        minimum: min,
        maximum: max,
        quantiles: probabilities
            .iter()
            .map(|p| crate::results::Quantile {
                probability: *p,
                value: quantile(&x, *p),
            })
            .collect(),
        median_absolute_deviation: mad,
        robust_standard_deviation: robust,
        interquartile_range: q1.zip(q3).map(|(a, b)| b - a),
        skewness: skew,
        excess_kurtosis: kurt,
        confidence_interval: ci,
    }
}
fn empty() -> DescriptiveStatistics {
    DescriptiveStatistics {
        count: 0,
        mean: None,
        median: None,
        standard_deviation: None,
        sample_variance: None,
        rms: None,
        peak_to_peak: None,
        minimum: None,
        maximum: None,
        quantiles: Vec::new(),
        median_absolute_deviation: None,
        robust_standard_deviation: None,
        interquartile_range: None,
        skewness: None,
        excess_kurtosis: None,
        confidence_interval: None,
    }
}
