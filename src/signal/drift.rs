use super::statistics;
use crate::results::{DriftAnalysis, DriftModelKind};
pub fn estimate(time: &[f64], values: &[f64], model: DriftModelKind) -> DriftAnalysis {
    let n = time.len().min(values.len());
    let duration = if n > 1 {
        Some((time[n - 1] - time[0]).abs())
    } else {
        None
    };
    if n < 2 {
        return DriftAnalysis {
            model,
            slope_v_per_s: None,
            slope_mv_per_h: None,
            slope_mv_per_day: None,
            intercept_v: None,
            standard_error: None,
            confidence_interval: None,
            r_squared: None,
            robust_residual_scale: None,
            observations: n,
            duration_s: duration,
        };
    }
    let slope = if matches!(model, DriftModelKind::TheilSen) {
        let mut slopes = Vec::new();
        for i in 0..n {
            for j in i + 1..n {
                if time[j] != time[i] {
                    slopes.push((values[j] - values[i]) / (time[j] - time[i]));
                }
            }
        }
        statistics::median(&mut slopes)
    } else {
        let xm = statistics::mean(&time[..n]).unwrap_or(0.0);
        let ym = statistics::mean(&values[..n]).unwrap_or(0.0);
        let den = time[..n].iter().map(|x| (x - xm).powi(2)).sum::<f64>();
        (den > 0.0).then_some(
            time[..n]
                .iter()
                .zip(&values[..n])
                .map(|(x, y)| (x - xm) * (y - ym))
                .sum::<f64>()
                / den,
        )
    };
    let intercept = slope.map(|b| {
        statistics::mean(&values[..n]).unwrap_or(0.0)
            - b * statistics::mean(&time[..n]).unwrap_or(0.0)
    });
    let residuals = slope
        .zip(intercept)
        .map(|(b, a)| {
            values[..n]
                .iter()
                .zip(&time[..n])
                .map(|(y, x)| y - (a + b * x))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let scale = {
        let mut r = residuals.iter().map(|v| v.abs()).collect::<Vec<_>>();
        statistics::median(&mut r).map(|v| 1.4826 * v)
    };
    let sse = residuals.iter().map(|v| v * v).sum::<f64>();
    let mean = statistics::mean(&values[..n]).unwrap_or(0.0);
    let sst = values[..n].iter().map(|v| (v - mean).powi(2)).sum::<f64>();
    let r2 = (sst > 0.0).then_some(1.0 - sse / sst);
    let se = (n > 2).then(|| {
        (sse / (n - 2) as f64).sqrt()
            / (time[..n]
                .iter()
                .map(|x| (x - statistics::mean(&time[..n]).unwrap_or(0.0)).powi(2))
                .sum::<f64>())
            .sqrt()
    });
    let ci = slope.zip(se).map(|(b, s)| (b - 1.96 * s, b + 1.96 * s));
    DriftAnalysis {
        model,
        slope_v_per_s: slope,
        slope_mv_per_h: slope.map(|v| v * 3_600_000.0),
        slope_mv_per_day: slope.map(|v| v * 86_400_000.0),
        intercept_v: intercept,
        standard_error: se,
        confidence_interval: ci,
        r_squared: r2,
        robust_residual_scale: scale,
        observations: n,
        duration_s: duration,
    }
}
