#![allow(clippy::map_flatten)]

use crate::{
    estimation::error::EstimationError,
    estimation_config::FilterKind,
    results::{FilterComparisonRecord, StateEstimationReport, StateFilterComparison},
};
use std::time::Instant;

pub fn compare_reports(
    reports: &[(FilterKind, StateEstimationReport)],
    truth: Option<&[crate::estimation::validation::TruthPoint]>,
) -> StateFilterComparison {
    let mut records = Vec::new();
    for (filter, report) in reports {
        let rmse = truth
            .map(|truth| {
                let pairs = truth
                    .iter()
                    .filter_map(|t| t.log10_activity)
                    .zip(
                        report
                            .estimates
                            .iter()
                            .filter_map(|p| p.activity.map(f64::log10)),
                    )
                    .collect::<Vec<_>>();
                if pairs.is_empty() {
                    None
                } else {
                    Some(
                        (pairs.iter().map(|(a, b)| (b - a).powi(2)).sum::<f64>()
                            / pairs.len() as f64)
                            .sqrt(),
                    )
                }
            })
            .flatten();
        let se = report
            .estimates
            .iter()
            .filter_map(|p| p.activity_standard_error)
            .collect::<Vec<_>>();
        records.push(FilterComparisonRecord {
            filter: *filter,
            runtime_ms: 0.0,
            activity_rmse: rmse,
            innovation_mean: report.diagnostics.innovation_mean,
            nis_mean: report.diagnostics.nis_mean,
            rejected_updates: report.diagnostics.rejected_update_count,
            numerical_failures: report.diagnostics.numerical_failures,
            domain_excursions: report.diagnostics.domain_excursion_count,
            mean_activity_standard_error: (!se.is_empty())
                .then_some(se.iter().sum::<f64>() / se.len() as f64),
            warnings: report.warnings.clone(),
            log_likelihood: report.diagnostics.log_likelihood,
            nis_consistency: report.diagnostics.nis_consistency_interval.map(|(lo, hi)| {
                report
                    .diagnostics
                    .nis_mean
                    .is_some_and(|value| value >= lo && value <= hi)
            }),
            nees_mean: report.diagnostics.nees_mean,
            coverage: truth.and_then(|_| {
                report.validation.as_ref().and_then(|validation| {
                    let values = validation
                        .metrics
                        .iter()
                        .filter_map(|metric| metric.interval_coverage)
                        .collect::<Vec<_>>();
                    (!values.is_empty()).then_some(values.iter().sum::<f64>() / values.len() as f64)
                })
            }),
            activity_bias: truth.and_then(|_| {
                report.validation.as_ref().and_then(|validation| {
                    validation
                        .metrics
                        .iter()
                        .find(|metric| metric.state == "log10_activity")
                        .and_then(|metric| metric.bias)
                })
            }),
            rejected_update_rate: {
                let total = report.diagnostics.accepted_update_count
                    + report.diagnostics.rejected_update_count;
                (total > 0)
                    .then_some(report.diagnostics.rejected_update_count as f64 / total as f64)
            },
        });
    }
    StateFilterComparison {
        schema_version: 2,
        records,
        warnings: Vec::new(),
    }
}

pub fn compare_runs(
    mut run: impl FnMut(FilterKind) -> Result<StateEstimationReport, EstimationError>,
    truth: Option<&[crate::estimation::validation::TruthPoint]>,
) -> Result<StateFilterComparison, EstimationError> {
    let mut reports = Vec::new();
    for filter in [FilterKind::Ekf, FilterKind::Ukf] {
        let start = Instant::now();
        let report = run(filter)?;
        let runtime = start.elapsed().as_secs_f64() * 1000.0;
        let mut c = compare_reports(&[(filter, report.clone())], truth);
        if let Some(r) = c.records.first_mut() {
            r.runtime_ms = runtime;
        }
        reports.push((filter, report));
    }
    Ok(compare_reports(&reports, truth))
}
