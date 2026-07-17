use crate::{
    estimation::state::MeasurementUpdateStatus,
    estimation::state::{EstimationWarning, EstimationWarningKind},
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
    let confidence = report
        .configuration
        .filter
        .confidence_level
        .clamp(1e-6, 1.0 - 1e-6);
    let z_score = standard_normal_quantile(0.5 + confidence / 2.0);
    let alignment_tolerance_s =
        resolve_alignment_tolerance_s(&report.estimates, truth).max(f64::EPSILON);
    let matching = align_samples(report, truth, alignment_tolerance_s);
    let samples = matching.samples;
    let mut warnings = Vec::new();
    if !matching.unmatched_estimate_timestamps_s.is_empty() {
        warnings.push(EstimationWarning::new(
            EstimationWarningKind::ModelDiscrepancy,
            format!(
                "{} estimate rows could not be aligned to truth within {:.6} s",
                matching.unmatched_estimate_timestamps_s.len(),
                alignment_tolerance_s
            ),
        ));
    }
    if !matching.unmatched_truth_timestamps_s.is_empty() {
        warnings.push(EstimationWarning::new(
            EstimationWarningKind::MissingMeasurement,
            format!(
                "{} truth rows were unmatched within {:.6} s",
                matching.unmatched_truth_timestamps_s.len(),
                alignment_tolerance_s
            ),
        ));
    }
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
                    if err.abs() <= z_score * s {
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
            convergence_time_s: convergence_time(
                &samples_for_state,
                convergence_threshold(&def.name),
            ),
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
        matched_sample_count: samples.len(),
        alignment_tolerance_s: Some(alignment_tolerance_s),
        unmatched_estimate_timestamps_s: matching.unmatched_estimate_timestamps_s,
        unmatched_truth_timestamps_s: matching.unmatched_truth_timestamps_s,
        warnings,
    }
}

struct SampleAlignment<'a> {
    samples: Vec<(&'a TruthPoint, &'a crate::results::StateEstimatePoint)>,
    unmatched_estimate_timestamps_s: Vec<f64>,
    unmatched_truth_timestamps_s: Vec<f64>,
}

fn align_samples<'a>(
    report: &'a StateEstimationReport,
    truth: &'a [TruthPoint],
    tolerance_s: f64,
) -> SampleAlignment<'a> {
    let mut truth_indices = (0..truth.len()).collect::<Vec<_>>();
    truth_indices.sort_by(|left, right| {
        truth[*left]
            .timestamp_s
            .total_cmp(&truth[*right].timestamp_s)
    });
    let mut estimate_indices = (0..report.estimates.len()).collect::<Vec<_>>();
    estimate_indices.sort_by(|left, right| {
        report.estimates[*left]
            .timestamp_s
            .total_cmp(&report.estimates[*right].timestamp_s)
    });
    let mut used_truth = vec![false; truth.len()];
    let mut samples = Vec::new();
    let mut unmatched_estimate_timestamps_s = Vec::new();
    for estimate_index in estimate_indices {
        let estimate = &report.estimates[estimate_index];
        let mut best_match: Option<(usize, f64)> = None;
        for truth_index in &truth_indices {
            if used_truth[*truth_index] {
                continue;
            }
            let delta = (truth[*truth_index].timestamp_s - estimate.timestamp_s).abs();
            if delta <= tolerance_s {
                if let Some((_, current)) = best_match {
                    if delta < current {
                        best_match = Some((*truth_index, delta));
                    }
                } else {
                    best_match = Some((*truth_index, delta));
                }
            }
        }
        if let Some((truth_index, _)) = best_match {
            used_truth[truth_index] = true;
            samples.push((&truth[truth_index], estimate));
        } else {
            unmatched_estimate_timestamps_s.push(estimate.timestamp_s);
        }
    }
    let unmatched_truth_timestamps_s = truth_indices
        .into_iter()
        .filter(|index| !used_truth[*index])
        .map(|index| truth[index].timestamp_s)
        .collect::<Vec<_>>();
    SampleAlignment {
        samples,
        unmatched_estimate_timestamps_s,
        unmatched_truth_timestamps_s,
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

fn convergence_time(samples: &[(f64, f64, f64)], threshold: f64) -> Option<f64> {
    let first = samples.first()?.0;
    let last_bad = samples
        .iter()
        .filter(|(_, actual, estimate)| (estimate - actual).abs() > threshold)
        .map(|(time, _, _)| *time)
        .reduce(f64::max);
    Some((last_bad.unwrap_or(first) - first).max(0.0))
}

fn convergence_threshold(state: &str) -> f64 {
    match state {
        "log10_activity" => 5e-2,
        "baseline_offset" | "polarization" => 1e-3,
        "sensitivity_scale" => 1e-2,
        _ => 1e-3,
    }
}

fn resolve_alignment_tolerance_s(
    estimates: &[crate::results::StateEstimatePoint],
    truth: &[TruthPoint],
) -> f64 {
    let estimate_step = median_time_step(estimates.iter().map(|point| point.timestamp_s));
    let truth_step = median_time_step(truth.iter().map(|point| point.timestamp_s));
    match (estimate_step, truth_step) {
        (Some(left), Some(right)) => 0.5 * left.min(right),
        (Some(step), None) | (None, Some(step)) => 0.5 * step,
        (None, None) => 1e-9,
    }
}

fn median_time_step<I>(timestamps: I) -> Option<f64>
where
    I: Iterator<Item = f64>,
{
    let mut sorted = timestamps
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    sorted.sort_by(f64::total_cmp);
    let mut deltas = sorted
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .filter(|delta| delta.is_finite() && *delta > 0.0)
        .collect::<Vec<_>>();
    if deltas.is_empty() {
        return None;
    }
    deltas.sort_by(f64::total_cmp);
    Some(deltas[deltas.len() / 2])
}

fn standard_normal_quantile(p: f64) -> f64 {
    let p = p.clamp(1e-15, 1.0 - 1e-15);
    let a: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    let b: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    let c: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    let d: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;
    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
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
