use super::statistics;
use crate::{
    results::{SpikeAnalysis, SpikeFlag},
    signal_config::SpikesConfig,
};
pub fn detect(time: &[f64], values: &[Option<f64>], config: &SpikesConfig) -> SpikeAnalysis {
    let n = values.len();
    let half = config.window_points.max(3) / 2;
    let mut out = Vec::new();
    for i in 0..n {
        let Some(v) = values[i] else { continue };
        let lo = i.saturating_sub(half);
        let hi = (i + half + 1).min(n);
        let local = values[lo..hi]
            .iter()
            .flatten()
            .copied()
            .filter(|x| x.is_finite())
            .collect::<Vec<_>>();
        if local.len() < config.minimum_local_observations {
            continue;
        }
        let med = statistics::median(&mut local.clone());
        let mut dev = local
            .iter()
            .map(|x| (x - med.unwrap_or(0.0)).abs())
            .collect::<Vec<_>>();
        let mad = statistics::median(&mut dev);
        let norm = match (mad, med) {
            (Some(m), Some(center)) if m > 0.0 => Some((v - center).abs() / (1.4826 * m)),
            (Some(_), Some(center)) if (v - center).abs() > 0.0 => Some(1.0e12),
            _ => None,
        };
        if norm.is_some_and(|z| z > config.mad_threshold) {
            let sustained = i > 0
                && i + 1 < n
                && values[i - 1].is_some_and(|x| (x - v).abs() < mad.unwrap_or(0.0))
                && values[i + 1].is_some_and(|x| (x - v).abs() < mad.unwrap_or(0.0));
            out.push(SpikeFlag {
                index: i,
                timestamp_s: time.get(i).copied().unwrap_or(0.0),
                value: v,
                local_median: med,
                local_mad: mad,
                normalized_deviation: norm,
                sustained_step: sustained,
            });
        }
    }
    let fraction = (n > 0).then_some((out.len() as f64) / (n as f64));
    SpikeAnalysis {
        method: config.method.clone(),
        flagged: out,
        flagged_fraction: fraction,
        maximum_flagged_fraction: config.maximum_flagged_fraction,
    }
}
