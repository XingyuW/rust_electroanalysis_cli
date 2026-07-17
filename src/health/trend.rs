use crate::{
    results::{HealthTrend, HealthTrendPoint, HealthTrendReport, HealthWarning},
    signal::{drift, statistics},
};
pub fn calculate(
    feature: &str,
    points: Vec<(String, Option<f64>, Option<f64>)>,
    baseline: Option<f64>,
) -> HealthTrend {
    let vals = points
        .iter()
        .filter_map(|(_, x, y)| x.zip(*y))
        .collect::<Vec<_>>();
    let xs = vals.iter().map(|p| p.0).collect::<Vec<_>>();
    let ys = vals.iter().map(|p| p.1).collect::<Vec<_>>();
    let p = points
        .into_iter()
        .map(|(id, x, ind)| HealthTrendPoint {
            record_id: id,
            independent_value: ind,
            feature: feature.into(),
            value: x,
            absolute_change: x.zip(baseline).map(|(v, b)| v - b),
            relative_change: x
                .zip(baseline)
                .and_then(|(v, b)| (b != 0.0).then_some((v - b) / b)),
            log_change: x
                .zip(baseline)
                .and_then(|(v, b)| (v > 0.0 && b > 0.0).then_some((v / b).ln())),
            change_from_baseline: x.zip(baseline).map(|(v, b)| v - b),
        })
        .collect::<Vec<_>>();
    let (s, rs) = if xs.len() > 1 {
        let t = (0..xs.len()).map(|i| i as f64).collect::<Vec<_>>();
        let ordinary =
            drift::estimate(&t, &ys, crate::results::DriftModelKind::OrdinaryLinear).slope_v_per_s;
        let robust =
            drift::estimate(&t, &ys, crate::results::DriftModelKind::TheilSen).slope_v_per_s;
        (ordinary, robust)
    } else {
        (None, None)
    };
    let rank = if xs.len() > 1 {
        Some(rank_corr(&xs, &ys))
    } else {
        None
    };
    HealthTrend {
        feature: feature.into(),
        points: p,
        ordinary_slope: s,
        theil_sen_slope: rs,
        rank_correlation: rank,
        replicate_standard_deviation: statistics::stddev(&ys),
        warnings: Vec::new(),
    }
}
fn rank_corr(a: &[f64], b: &[f64]) -> f64 {
    let rank = |x: &[f64]| {
        let mut o = (0..x.len()).collect::<Vec<_>>();
        o.sort_by(|i, j| x[*i].total_cmp(&x[*j]));
        let mut r = vec![0.0; x.len()];
        for (i, j) in o.into_iter().enumerate() {
            r[j] = i as f64;
        }
        r
    };
    let ar = rank(a);
    let br = rank(b);
    let ma = statistics::mean(&ar).unwrap_or(0.0);
    let mb = statistics::mean(&br).unwrap_or(0.0);
    let den = (ar.iter().map(|v| (v - ma).powi(2)).sum::<f64>()
        * br.iter().map(|v| (v - mb).powi(2)).sum::<f64>())
    .sqrt();
    if den > 0.0 {
        ar.iter()
            .zip(br)
            .map(|(x, y)| (x - ma) * (y - mb))
            .sum::<f64>()
            / den
    } else {
        0.0
    }
}
pub fn report(
    id: &str,
    trends: Vec<HealthTrend>,
    provenance: crate::domain::AnalysisProvenance,
) -> HealthTrendReport {
    HealthTrendReport {
        schema_version: 1,
        analysis_id: id.into(),
        trends,
        provenance,
        warnings: Vec::<HealthWarning>::new(),
    }
}
