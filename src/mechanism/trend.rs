//! Descriptive longitudinal trend calculations.

use crate::results::{
    CharacteristicTimescale, MechanismRecordSummary, MechanismTrendResult, MechanismWarning,
};

pub fn calculate_trend(
    variable: &str,
    records: &[MechanismRecordSummary],
    values: &[(String, f64)],
    independent_variable: &str,
    minimum_records: usize,
) -> MechanismTrendResult {
    let points = values
        .iter()
        .filter_map(|(id, value)| {
            let x = records
                .iter()
                .find(|record| &record.record_id == id)
                .and_then(|record| record.sensor_age_days);
            x.zip(Some(*value))
                .filter(|(x, y)| x.is_finite() && y.is_finite())
        })
        .collect::<Vec<_>>();
    let mut warnings = Vec::new();
    if points.len() < minimum_records {
        warnings.push(MechanismWarning {
            kind: "insufficient_replicates".to_string(),
            message: format!(
                "{} usable records are below the configured trend minimum of {minimum_records}",
                points.len()
            ),
        });
    }
    let first = points.first().map(|(_, y)| *y);
    let last = points.last().map(|(_, y)| *y);
    let absolute_change = first.zip(last).map(|(a, b)| b - a);
    let relative_change = first
        .filter(|v| v.abs() > f64::MIN_POSITIVE)
        .zip(last)
        .map(|(a, b)| (b - a) / a);
    let log_change = first
        .zip(last)
        .filter(|(a, b)| *a > 0.0 && *b > 0.0)
        .map(|(a, b)| (b / a).ln());
    let slope = linear_slope(&points);
    let robust_slope = theil_sen(&points);
    let rank_correlation = spearman(&points);
    let mean = points.iter().map(|(_, y)| *y).sum::<f64>() / points.len().max(1) as f64;
    let variability = (points.iter().map(|(_, y)| (y - mean).powi(2)).sum::<f64>()
        / points.len().max(1) as f64)
        .sqrt();
    MechanismTrendResult {
        variable: variable.to_string(),
        independent_variable: independent_variable.to_string(),
        records: points.len(),
        absolute_change,
        relative_change,
        log_change,
        slope,
        robust_slope,
        rank_correlation,
        replicate_variability: Some(variability),
        warnings,
    }
}

pub fn timescale_values(
    timescales: &[CharacteristicTimescale],
    record_prefix: &str,
) -> Vec<(String, f64)> {
    timescales
        .iter()
        .filter(|t| {
            t.timescale_id.starts_with(record_prefix) && t.value_s.is_finite() && t.value_s > 0.0
        })
        .map(|t| (t.timescale_id.clone(), t.value_s))
        .collect()
}

fn linear_slope(points: &[(f64, f64)]) -> Option<f64> {
    if points.len() < 2 {
        return None;
    }
    let xm = points.iter().map(|(x, _)| *x).sum::<f64>() / points.len() as f64;
    let ym = points.iter().map(|(_, y)| *y).sum::<f64>() / points.len() as f64;
    let den = points.iter().map(|(x, _)| (x - xm).powi(2)).sum::<f64>();
    (den > 0.0).then_some(points.iter().map(|(x, y)| (x - xm) * (y - ym)).sum::<f64>() / den)
}
fn theil_sen(points: &[(f64, f64)]) -> Option<f64> {
    let mut slopes = Vec::new();
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let dx = points[j].0 - points[i].0;
            if dx != 0.0 {
                slopes.push((points[j].1 - points[i].1) / dx);
            }
        }
    }
    slopes.sort_by(f64::total_cmp);
    slopes.get(slopes.len() / 2).copied()
}
fn spearman(points: &[(f64, f64)]) -> Option<f64> {
    if points.len() < 2 {
        return None;
    }
    let mut xs = points.iter().map(|(x, _)| *x).collect::<Vec<_>>();
    let mut ys = points.iter().map(|(_, y)| *y).collect::<Vec<_>>();
    let rx = rank(&mut xs);
    let ry = rank(&mut ys);
    let xm = rx.iter().sum::<f64>() / rx.len() as f64;
    let ym = ry.iter().sum::<f64>() / ry.len() as f64;
    let num = rx
        .iter()
        .zip(&ry)
        .map(|(x, y)| (x - xm) * (y - ym))
        .sum::<f64>();
    let den = (rx.iter().map(|x| (x - xm).powi(2)).sum::<f64>()
        * ry.iter().map(|y| (y - ym).powi(2)).sum::<f64>())
    .sqrt();
    (den > 0.0).then_some(num / den)
}
fn rank(values: &mut [f64]) -> Vec<f64> {
    let mut order = (0..values.len()).collect::<Vec<_>>();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut out = vec![0.0; values.len()];
    for (rank, &index) in order.iter().enumerate() {
        out[index] = rank as f64 + 1.0;
    }
    out
}
