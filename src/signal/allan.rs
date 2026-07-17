use super::{error::SignalError, statistics};
use crate::{
    results::{AllanAnalysis, AllanPoint, SignalWarning},
    signal_config::AllanConfig,
};

pub fn overlapping(
    time: &[f64],
    values: &[f64],
    config: &AllanConfig,
) -> Result<AllanAnalysis, SignalError> {
    if values.len() != time.len() || values.len() < 4 {
        return Err(SignalError::invalid(
            "Allan deviation requires at least four samples",
        ));
    }
    let dt = statistics::median(&mut time.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>())
        .ok_or_else(|| SignalError::Sampling("sampling interval unavailable".into()))?;
    if time.windows(2).any(|w| w[1] <= w[0]) {
        return Err(SignalError::Sampling(
            "Allan deviation requires increasing timestamps".into(),
        ));
    }
    let max_m = (values.len() / 2).max(1);
    let mut ms = Vec::new();
    for i in 1..=config.tau_points.max(1) {
        let m = 1 + ((max_m.saturating_sub(1) * i) / config.tau_points.max(1));
        if !ms.contains(&m) {
            ms.push(m);
        }
    }
    let mut points = Vec::new();
    let mut warnings = Vec::new();
    for m in ms {
        let count = values.len().saturating_sub(2 * m) + 1;
        let mut sum = 0.0;
        if count > 0 {
            for k in 0..count {
                let a = values[k..k + m].iter().sum::<f64>() / m as f64;
                let b = values[k + m..k + 2 * m].iter().sum::<f64>() / m as f64;
                sum += (b - a).powi(2);
            }
            let dev = (sum / (2.0 * count as f64)).sqrt();
            points.push(AllanPoint {
                averaging_time_s: m as f64 * dt,
                deviation: Some(dev),
                effective_differences: count,
                approximate_uncertainty: Some(dev / (2.0 * count as f64).sqrt()),
                log_log_slope: None,
            });
            if count < config.minimum_clusters {
                warnings.push(SignalWarning::InsufficientAllanClusters);
            }
        }
    }
    for i in 1..points.len() {
        let a = points[i - 1].deviation;
        let b = points[i].deviation;
        points[i].log_log_slope = a.zip(b).map(|(x, y)| {
            let dtau = points[i].averaging_time_s / points[i - 1].averaging_time_s;
            (y.max(1e-30).ln() - x.max(1e-30).ln()) / dtau.ln()
        });
    }
    let min = points
        .iter()
        .filter_map(|p| p.deviation.map(|v| (v, p.averaging_time_s)))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    Ok(AllanAnalysis {
        points,
        warnings: dedup(warnings),
        minimum_deviation: min.map(|x| x.0),
        minimum_averaging_time_s: min.map(|x| x.1),
    })
}
fn dedup(mut v: Vec<SignalWarning>) -> Vec<SignalWarning> {
    v.dedup();
    v
}
