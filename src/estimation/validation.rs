use crate::{
    estimation::state::MeasurementUpdateStatus,
    estimation::state::{EstimationWarning, EstimationWarningKind},
    estimation_config::{StateValidationConfig, TruthAlignmentPolicy},
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
    let alignment_tolerance_s = report.configuration.validation.maximum_alignment_gap_s;
    let matching = align_samples(
        report,
        truth,
        report.configuration.validation.alignment_policy,
        alignment_tolerance_s,
        report.configuration.validation.allow_truth_reuse,
    );
    let samples = matching.samples;
    let mut warnings = Vec::new();
    if truth.windows(2).any(|window| {
        !window[0].timestamp_s.is_finite()
            || !window[1].timestamp_s.is_finite()
            || window[1].timestamp_s <= window[0].timestamp_s
    }) {
        warnings.push(EstimationWarning::new(
            EstimationWarningKind::ModelDiscrepancy,
            "truth timestamps are not strictly increasing; truth alignment was rejected",
        ));
    }
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
    for sample in &samples {
        let truth_point = &sample.truth;
        let estimate_point = sample.estimate;
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
                    .and_then(|state| {
                        if definition.name == "log10_activity" {
                            state.latent_value.or(state.value)
                        } else {
                            state.value
                        }
                    })
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
        for sample in &samples {
            let truth_point = &sample.truth;
            let point = sample.estimate;
            let actual = truth_value(truth_point, &def.name);
            let estimate = point
                .filtered_state
                .iter()
                .find(|state| state.name == def.name)
                .and_then(|state| {
                    if def.name == "log10_activity" {
                        state.latent_value.or(state.value)
                    } else {
                        state.value
                    }
                });
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
                validation_state_config(report, &def.name),
            ),
            step_response_delay_s: step_response_delay(
                &samples_for_state,
                validation_state_config(report, &def.name),
            ),
            maximum_transient_error: errors.iter().map(|x| x.abs()).reduce(f64::max),
            outlier_rejection_rate: (outlier_count > 0)
                .then_some(rejected_outliers as f64 / outlier_count as f64),
            calibration_domain_violations: report.diagnostics.domain_excursion_count,
            sample_count: count,
            nees_consistency_interval: consistency_interval(nees.len(), 1.0, confidence, z_score),
        });
    }
    StateValidationResult {
        truth_source: source,
        metrics,
        vector_nees_mean: (!vector_nees.is_empty())
            .then_some(vector_nees.iter().sum::<f64>() / vector_nees.len() as f64),
        vector_nees_count: vector_nees.len(),
        vector_nees_consistency_interval: consistency_interval(
            vector_nees.len(),
            report.state_definitions.len() as f64,
            confidence,
            z_score,
        ),
        matched_sample_count: samples.len(),
        alignment_tolerance_s: Some(alignment_tolerance_s),
        unmatched_estimate_timestamps_s: matching.unmatched_estimate_timestamps_s,
        unmatched_truth_timestamps_s: matching.unmatched_truth_timestamps_s,
        alignment_policy: Some(format!(
            "{:?}",
            report.configuration.validation.alignment_policy
        )),
        alignment_methods: matching.methods,
        warnings,
    }
}

struct AlignedSample<'a> {
    truth: TruthPoint,
    estimate: &'a crate::results::StateEstimatePoint,
}

struct SampleAlignment<'a> {
    samples: Vec<AlignedSample<'a>>,
    unmatched_estimate_timestamps_s: Vec<f64>,
    unmatched_truth_timestamps_s: Vec<f64>,
    methods: Vec<String>,
}

fn align_samples<'a>(
    report: &'a StateEstimationReport,
    truth: &'a [TruthPoint],
    policy: TruthAlignmentPolicy,
    tolerance_s: f64,
    allow_truth_reuse: bool,
) -> SampleAlignment<'a> {
    let truth_indices = (0..truth.len()).collect::<Vec<_>>();
    let mut estimate_indices = (0..report.estimates.len()).collect::<Vec<_>>();
    estimate_indices.sort_by(|left, right| {
        report.estimates[*left]
            .timestamp_s
            .total_cmp(&report.estimates[*right].timestamp_s)
    });
    let mut used_truth = vec![false; truth.len()];
    let mut samples = Vec::new();
    let mut methods = Vec::new();
    if truth.windows(2).any(|window| {
        !window[0].timestamp_s.is_finite()
            || !window[1].timestamp_s.is_finite()
            || window[1].timestamp_s <= window[0].timestamp_s
    }) {
        return SampleAlignment {
            samples,
            unmatched_estimate_timestamps_s: report
                .estimates
                .iter()
                .map(|estimate| estimate.timestamp_s)
                .collect(),
            unmatched_truth_timestamps_s: truth.iter().map(|point| point.timestamp_s).collect(),
            methods,
        };
    }
    let mut unmatched_estimate_timestamps_s = Vec::new();
    for estimate_index in estimate_indices {
        let estimate = &report.estimates[estimate_index];
        let aligned = match policy {
            TruthAlignmentPolicy::Exact => truth_indices
                .iter()
                .copied()
                .filter(|index| allow_truth_reuse || !used_truth[*index])
                .find(|index| {
                    (truth[*index].timestamp_s - estimate.timestamp_s).abs() <= f64::EPSILON
                })
                .map(|index| {
                    (
                        TruthPoint {
                            ..truth[index].clone()
                        },
                        vec![index],
                        "exact".to_string(),
                    )
                }),
            TruthAlignmentPolicy::NearestWithinTolerance => truth_indices
                .iter()
                .copied()
                .filter(|index| allow_truth_reuse || !used_truth[*index])
                .filter_map(|index| {
                    let gap = (truth[index].timestamp_s - estimate.timestamp_s).abs();
                    (gap <= tolerance_s).then_some((index, gap))
                })
                .min_by(|left, right| left.1.total_cmp(&right.1))
                .map(|(index, _)| {
                    (
                        TruthPoint {
                            ..truth[index].clone()
                        },
                        vec![index],
                        "nearest_within_tolerance".to_string(),
                    )
                }),
            TruthAlignmentPolicy::LinearInterpolation => truth
                .windows(2)
                .enumerate()
                .find(|(index, window)| {
                    let unused =
                        allow_truth_reuse || (!used_truth[*index] && !used_truth[*index + 1]);
                    window[0].timestamp_s <= estimate.timestamp_s
                        && estimate.timestamp_s <= window[1].timestamp_s
                        && (estimate.timestamp_s - window[0].timestamp_s) <= tolerance_s
                        && (window[1].timestamp_s - estimate.timestamp_s) <= tolerance_s
                        && unused
                })
                .map(|(index, window)| {
                    let fraction = (estimate.timestamp_s - window[0].timestamp_s)
                        / (window[1].timestamp_s - window[0].timestamp_s).max(f64::EPSILON);
                    (
                        interpolate_truth(&window[0], &window[1], fraction, estimate.timestamp_s),
                        vec![index, index + 1],
                        "linear_interpolation".to_string(),
                    )
                }),
        };
        if let Some((point, indices, method)) = aligned {
            if !allow_truth_reuse {
                for index in indices {
                    used_truth[index] = true;
                }
            }
            methods.push(method.clone());
            samples.push(AlignedSample {
                truth: point,
                estimate,
            });
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
        methods,
    }
}

fn interpolate_truth(
    left: &TruthPoint,
    right: &TruthPoint,
    fraction: f64,
    timestamp_s: f64,
) -> TruthPoint {
    let lerp = |a: Option<f64>, b: Option<f64>| a.zip(b).map(|(a, b)| a + fraction * (b - a));
    TruthPoint {
        timestamp_s,
        log10_activity: lerp(left.log10_activity, right.log10_activity),
        activity: lerp(left.activity, right.activity),
        baseline_offset_v: lerp(left.baseline_offset_v, right.baseline_offset_v),
        polarization_v: lerp(left.polarization_v, right.polarization_v),
        sensitivity_scale: lerp(left.sensitivity_scale, right.sensitivity_scale),
        outlier: left.outlier || right.outlier,
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

fn convergence_time(samples: &[(f64, f64, f64)], config: &StateValidationConfig) -> Option<f64> {
    let first = samples.first()?.0;
    let required = config.minimum_consecutive_converged_points;
    let mut consecutive = 0;
    for (time, actual, estimate) in samples {
        if (estimate - actual).abs() <= config.absolute_convergence_tolerance {
            consecutive += 1;
            if consecutive >= required {
                return Some((*time - first).max(0.0));
            }
        } else {
            consecutive = 0;
        }
    }
    Some((samples.last()?.0 - first).max(0.0))
}

fn validation_state_config<'a>(
    report: &'a StateEstimationReport,
    state: &str,
) -> &'a StateValidationConfig {
    static DEFAULT: std::sync::OnceLock<StateValidationConfig> = std::sync::OnceLock::new();
    report
        .configuration
        .validation
        .states
        .get(state)
        .unwrap_or_else(|| DEFAULT.get_or_init(StateValidationConfig::default))
}

fn consistency_interval(
    count: usize,
    expected: f64,
    _confidence: f64,
    z_score: f64,
) -> Option<(f64, f64)> {
    (count >= 2).then_some((
        0.0_f64.max(expected - z_score * (2.0 * expected / count as f64).sqrt()),
        expected + z_score * (2.0 * expected / count as f64).sqrt(),
    ))
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

fn step_response_delay(samples: &[(f64, f64, f64)], config: &StateValidationConfig) -> Option<f64> {
    let (step_index, step_size) = samples
        .windows(2)
        .enumerate()
        .map(|(index, window)| (index, window[1].1 - window[0].1))
        .max_by(|(_, left), (_, right)| left.abs().total_cmp(&right.abs()))?;
    if step_size.abs() <= config.step_detection_threshold {
        return None;
    }
    let before = samples[step_index].2;
    let target = before + config.step_response_fraction * step_size;
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
    if out
        .windows(2)
        .any(|window| window[1].timestamp_s <= window[0].timestamp_s)
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "truth timestamps must be strictly increasing",
        ));
    }
    Ok(out)
}
