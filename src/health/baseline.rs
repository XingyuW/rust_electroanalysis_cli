use crate::{
    results::{
        BaselineFeatureDistribution, BaselineRecordSummary, HealthWarning, SensorHealthBaseline,
    },
    signal::statistics,
};
use serde::Deserialize;
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Context {
    pub sensor_id: Option<String>,
    pub sensor_design: Option<String>,
    pub analyte: Option<String>,
    pub sample_matrix: Option<String>,
    pub temperature_k: Option<f64>,
}
pub fn build(
    id: &str,
    features: &[(String, Vec<crate::results::HealthFeature>)],
    provenance: crate::domain::AnalysisProvenance,
    minimum: usize,
) -> SensorHealthBaseline {
    let mut names =
        std::collections::BTreeMap::<String, Vec<&crate::results::HealthFeature>>::new();
    let mut records = Vec::new();
    for (name, fs) in features {
        let domains = fs
            .iter()
            .map(|x| x.domain)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        records.push(BaselineRecordSummary {
            record_id: name.clone(),
            sensor_id: None,
            analyte: None,
            sample_matrix: None,
            temperature_k: None,
            sensor_design: None,
            domains,
        });
        for f in fs {
            if f.value.is_some_and(|v| v.is_finite()) {
                names.entry(f.name.clone()).or_default().push(f);
            }
        }
    }
    let distributions = names
        .into_iter()
        .map(|(name, fs)| {
            let mut x = fs.iter().filter_map(|f| f.value).collect::<Vec<_>>();
            x.sort_by(f64::total_cmp);
            let m = statistics::mean(&x);
            let sd = statistics::stddev(&x);
            let median = statistics::quantile(&x, 0.5);
            let mad = median.map(|v| {
                let mut d = x.iter().map(|a| (a - v).abs()).collect::<Vec<_>>();
                statistics::median(&mut d).unwrap_or(0.0)
            });
            let feature = fs[0];
            BaselineFeatureDistribution {
                feature: name,
                unit: feature.unit.clone(),
                domain: feature.domain,
                sample_count: x.len(),
                mean: m,
                standard_deviation: sd,
                median,
                mad,
                quantiles: vec![
                    (0.25, statistics::quantile(&x, 0.25)),
                    (0.5, median),
                    (0.75, statistics::quantile(&x, 0.75)),
                ],
                minimum: x.first().copied(),
                maximum: x.last().copied(),
                reference_direction: None,
                comparison_context: None,
            }
        })
        .collect();
    let mut warnings = Vec::new();
    if records.len() < minimum {
        warnings.push(HealthWarning::InsufficientBaselineRecords);
    }
    if records.len() < 2 {
        warnings.push(HealthWarning::InsufficientBaselineRecords);
    }
    SensorHealthBaseline {
        schema_version: 1,
        baseline_id: id.into(),
        sensor_type: None,
        analyte: None,
        sample_matrix: None,
        temperature_domain_k: None,
        feature_distributions: distributions,
        records,
        minimum_required_domains: minimum,
        provenance,
        warnings,
    }
}
