use crate::{
    estimation::state::MeasurementUpdateStatus,
    results::{StateEstimationReport, StateMetric, StateValidationResult},
};
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TruthPoint {
    pub timestamp_s: f64,
    pub log10_activity: Option<f64>,
    pub activity: Option<f64>,
    pub baseline_offset_v: Option<f64>,
    pub polarization_v: Option<f64>,
    pub sensitivity_scale: Option<f64>,
    #[serde(default)]
    pub outlier: bool,
}
pub fn validate_report(
    report: &StateEstimationReport,
    truth: &[TruthPoint],
    source: Option<String>,
) -> StateValidationResult {
    let mut metrics = Vec::new();
    let samples = report
        .estimates
        .iter()
        .filter_map(|point| {
            let truth = truth.iter().min_by(|a, b| {
                (a.timestamp_s - point.timestamp_s)
                    .abs()
                    .total_cmp(&(b.timestamp_s - point.timestamp_s).abs())
            })?;
            Some((truth, point))
        })
        .collect::<Vec<_>>();
    let mut vector_nees = Vec::new();
    for (truth_point, estimate_point) in &samples {
        let actual = report
            .state_definitions
            .iter()
            .map(|definition| truth_value(truth_point, &definition.name))
            .collect::<Vec<_>>();
        let estimate = report
            .state_definitions
            .iter()
            .map(|definition| {
                estimate_point
                    .filtered_state
                    .iter()
                    .find(|state| state.name == definition.name)
                    .and_then(|state| state.value)
            })
            .collect::<Vec<_>>();
        if actual.iter().all(Option::is_some) && estimate.iter().all(Option::is_some) {
            let covariance = matrix(&estimate_point.filtered_covariance);
            if covariance.nrows() == actual.len()
                && covariance.ncols() == actual.len()
                && covariance.iter().all(|value| value.is_finite())
                && let Some(inverse) = covariance.try_inverse()
            {
                let error = DVector::from_iterator(
                    actual.len(),
                    actual
                        .iter()
                        .zip(&estimate)
                        .map(|(a, e)| e.unwrap() - a.unwrap()),
                );
                let nees = (error.transpose() * inverse * error)[(0, 0)];
                if nees.is_finite() {
                    vector_nees.push(nees);
                }
            }
        }
    }
    for def in &report.state_definitions {
        let mut errors = Vec::new();
        let mut nees = Vec::new();
        let mut cover = 0;
        let mut count = 0;
        let mut outlier_count = 0;
        let mut rejected_outliers = 0;
        let mut samples_for_state = Vec::new();
        for (truth_point, point) in &samples {
            let actual = truth_value(truth_point, &def.name);
            let estimate = point
                .filtered_state
                .iter()
                .find(|state| state.name == def.name)
                .and_then(|state| state.value);
            if let (Some(actual), Some(estimate)) = (actual, estimate) {
                let err = estimate - actual;
                errors.push(err);
                samples_for_state.push((point.timestamp_s, actual, estimate));
                if truth_point.outlier {
                    outlier_count += 1;
                    if point.update_status == MeasurementUpdateStatus::RejectedByGate {
                        rejected_outliers += 1;
                    }
                }
                if let Some(s) = point
                    .filtered_state
                    .iter()
                    .find(|x| x.name == def.name)
                    .and_then(|x| x.standard_error)
                {
                    count += 1;
                    if err.abs() <= 1.96 * s {
                        cover += 1;
                    }
                    if s > 0.0 {
                        nees.push(err * err / (s * s));
                    }
                }
            }
        }
        let rmse = (!errors.is_empty())
            .then_some((errors.iter().map(|x| x * x).sum::<f64>() / errors.len() as f64).sqrt());
        let mae = (!errors.is_empty())
            .then_some(errors.iter().map(|x| x.abs()).sum::<f64>() / errors.len() as f64);
        let bias = (!errors.is_empty()).then_some(errors.iter().sum::<f64>() / errors.len() as f64);
        metrics.push(StateMetric {
            state: def.name.clone(),
            unit: def.unit.clone(),
            rmse,
            mae,
            bias,
            interval_coverage: (count > 0).then_some(cover as f64 / count as f64),
            nees_mean: (!nees.is_empty()).then_some(nees.iter().sum::<f64>() / nees.len() as f64),
            convergence_time_s: convergence_time(&samples_for_state),
            step_response_delay_s: step_response_delay(&samples_for_state),
            maximum_transient_error: errors.iter().map(|x| x.abs()).reduce(f64::max),
            outlier_rejection_rate: (outlier_count > 0)
                .then_some(rejected_outliers as f64 / outlier_count as f64),
            calibration_domain_violations: report.diagnostics.domain_excursion_count,
        });
    }
    StateValidationResult {
        truth_source: source,
        metrics,
        vector_nees_mean: (!vector_nees.is_empty())
            .then_some(vector_nees.iter().sum::<f64>() / vector_nees.len() as f64),
        vector_nees_count: vector_nees.len(),
        warnings: Vec::new(),
    }
}

fn truth_value(point: &TruthPoint, state: &str) -> Option<f64> {
    match state {
        "log10_activity" => point.log10_activity,
        "baseline_offset" => point.baseline_offset_v,
        "polarization" => point.polarization_v,
        "sensitivity_scale" => point.sensitivity_scale,
        _ => None,
    }
}

fn matrix(values: &[Vec<f64>]) -> DMatrix<f64> {
    if values.is_empty() || values.iter().any(|row| row.len() != values.len()) {
        return DMatrix::zeros(0, 0);
    }
    DMatrix::from_fn(values.len(), values.len(), |i, j| values[i][j])
}

fn convergence_time(samples: &[(f64, f64, f64)]) -> Option<f64> {
    let first = samples.first()?.0;
    let last_bad = samples
        .iter()
        .filter(|(_, actual, estimate)| (estimate - actual).abs() > 1.96e-3)
        .map(|(time, _, _)| *time)
        .reduce(f64::max);
    Some((last_bad.unwrap_or(first) - first).max(0.0))
}

fn step_response_delay(samples: &[(f64, f64, f64)]) -> Option<f64> {
    let (step_index, step_size) = samples
        .windows(2)
        .enumerate()
        .map(|(index, window)| (index, window[1].1 - window[0].1))
        .max_by(|(_, left), (_, right)| left.abs().total_cmp(&right.abs()))?;
    if step_size.abs() <= 1e-9 {
        return None;
    }
    let before = samples[step_index].2;
    let target = before + 0.9 * step_size;
    samples
        .iter()
        .skip(step_index + 1)
        .find(|(_, _, estimate)| {
            if step_size > 0.0 {
                *estimate >= target
            } else {
                *estimate <= target
            }
        })
        .map(|(time, _, _)| (*time - samples[step_index + 1].0).max(0.0))
}

pub fn read_truth_csv(path: &Path) -> Result<Vec<TruthPoint>, std::io::Error> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader.headers()?.clone();
    let idx = |names: &[&str]| {
        names
            .iter()
            .find_map(|n| headers.iter().position(|h| h.eq_ignore_ascii_case(n)))
    };
    let time = idx(&["time_s", "time/sec", "time", "timestamp"]).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "truth CSV lacks time column",
        )
    })?;
    let log = idx(&["log10_activity", "log10 activity"]);
    let act = idx(&["activity"]);
    let base = idx(&["baseline_offset_v", "baseline"]);
    let pol = idx(&["polarization_v", "polarization"]);
    let sens = idx(&["sensitivity_scale", "condition"]);
    let mut out = Vec::new();
    for row in reader.records() {
        let r = row?;
        let parse = |i: Option<usize>| {
            i.and_then(|j| r.get(j))
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| v.is_finite())
        };
        let t = r
            .get(time)
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "truth time is invalid")
            })?;
        out.push(TruthPoint {
            timestamp_s: t,
            log10_activity: log
                .and_then(|i| parse(Some(i)))
                .or_else(|| act.and_then(|i| parse(Some(i)).map(f64::log10))),
            activity: act.and_then(|i| parse(Some(i))),
            baseline_offset_v: base.and_then(|i| parse(Some(i))),
            polarization_v: pol.and_then(|i| parse(Some(i))),
            sensitivity_scale: sens.and_then(|i| parse(Some(i))),
            outlier: idx(&["outlier", "is_outlier"])
                .and_then(|i| r.get(i))
                .is_some_and(|value| {
                    matches!(value.to_ascii_lowercase().as_str(), "true" | "1" | "yes")
                }),
        })
    }
    Ok(out)
}
