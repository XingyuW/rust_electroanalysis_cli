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
        });
    }
    StateFilterComparison {
        schema_version: 1,
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
