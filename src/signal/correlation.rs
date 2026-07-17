use super::statistics;
use crate::{
    results::{ChannelCorrelationResult, SignalWarning},
    signal_config::CorrelationConfig,
};

pub fn pair(
    a_name: &str,
    a_time: &[f64],
    a: &[Option<f64>],
    b_name: &str,
    b_time: &[f64],
    b: &[Option<f64>],
    config: &CorrelationConfig,
) -> ChannelCorrelationResult {
    let mut pairs = Vec::new();
    for i in 0..a.len().min(a_time.len()) {
        let Some(x) = a[i] else { continue };
        let Some(j) = b_time.iter().position(|t| (*t - a_time[i]).abs() < 1e-9) else {
            continue;
        };
        let Some(y) = b.get(j).copied().flatten() else {
            continue;
        };
        if x.is_finite() && y.is_finite() {
            pairs.push((x, y));
        }
    }
    let n = pairs.len();
    let xs = pairs.iter().map(|p| p.0).collect::<Vec<_>>();
    let ys = pairs.iter().map(|p| p.1).collect::<Vec<_>>();
    let pearson = correlation(&xs, &ys);
    let spearman = correlation(&ranks(&xs), &ranks(&ys));
    let covariance = if n > 1 {
        let mx = statistics::mean(&xs).unwrap_or(0.0);
        let my = statistics::mean(&ys).unwrap_or(0.0);
        Some(
            xs.iter()
                .zip(&ys)
                .map(|(x, y)| (x - mx) * (y - my))
                .sum::<f64>()
                / (n - 1) as f64,
        )
    } else {
        None
    };
    let mut lags = Vec::new();
    let mut cross = Vec::new();
    if n >= config.minimum_observations {
        let max_lag = config.maximum_lag_s.max(0.0);
        let step = config.lag_step_s.unwrap_or_else(|| {
            a_time
                .windows(2)
                .next()
                .map(|w| (w[1] - w[0]).abs())
                .filter(|v| *v > 0.0)
                .unwrap_or(1.0)
        });
        let mut lag = -max_lag;
        while lag <= max_lag + step * 1e-9 {
            let mut x = Vec::new();
            let mut y = Vec::new();
            for i in 0..a.len() {
                let Some(ax) = a[i] else { continue };
                let target = a_time[i] + lag;
                let Some(j) = b_time
                    .iter()
                    .position(|t| (*t - target).abs() <= step / 2.0)
                else {
                    continue;
                };
                if let Some(by) = b.get(j).copied().flatten() {
                    x.push(ax);
                    y.push(by);
                }
            }
            if let Some(c) = correlation(&x, &y) {
                lags.push(lag);
                cross.push(c);
            }
            lag += step;
        }
    }
    let lag_max = cross
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.abs().total_cmp(&b.abs()))
        .and_then(|(i, _)| lags.get(i).copied());
    let residual = xs.iter().zip(&ys).map(|(x, y)| x - y).collect::<Vec<_>>();
    ChannelCorrelationResult {
        channel_a: a_name.into(),
        channel_b: b_name.into(),
        observations: n,
        pearson,
        spearman,
        covariance,
        lags_s: lags,
        cross_correlation: cross,
        lag_of_max_absolute_correlation_s: lag_max,
        common_mode_fraction: pearson.map(f64::abs),
        channel_specific_residual_scale_a: statistics::stddev(&residual),
        channel_specific_residual_scale_b: statistics::stddev(&residual),
        warning: (n < config.minimum_observations)
            .then_some(SignalWarning::CorrelationSampleCountInsufficient),
    }
}
fn correlation(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.len() != b.len() || a.len() < 2 {
        return None;
    }
    let ma = statistics::mean(a)?;
    let mb = statistics::mean(b)?;
    let na = a.iter().map(|x| (x - ma).powi(2)).sum::<f64>();
    let nb = b.iter().map(|x| (x - mb).powi(2)).sum::<f64>();
    (na > 0.0 && nb > 0.0).then_some(
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - ma) * (y - mb))
            .sum::<f64>()
            / (na * nb).sqrt(),
    )
}
fn ranks(v: &[f64]) -> Vec<f64> {
    let mut order = (0..v.len()).collect::<Vec<_>>();
    order.sort_by(|a, b| v[*a].total_cmp(&v[*b]));
    let mut result = vec![0.0; v.len()];
    for (i, j) in order.into_iter().enumerate() {
        result[j] = i as f64 + 1.0;
    }
    result
}
