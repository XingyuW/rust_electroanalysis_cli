use crate::{
    results::{
        BaselineContextConflict, BaselineFeatureDistribution, BaselineRecordSummary, HealthWarning,
        SensorHealthBaseline,
    },
    signal::statistics,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// Scientific context associated with one health-baseline record.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Context {
    pub sensor_id: Option<String>,
    pub sensor_type: Option<String>,
    pub sensor_design: Option<String>,
    pub analyte: Option<String>,
    pub sample_matrix: Option<String>,
    pub temperature_k: Option<f64>,
    #[serde(default)]
    pub temperature_values_k: Vec<f64>,
    pub experiment_id: Option<String>,
    pub metadata_source: Option<String>,
}

/// Backward-compatible baseline builder for callers that have no metadata.
pub fn build(
    id: &str,
    features: &[(String, Vec<crate::results::HealthFeature>)],
    provenance: crate::domain::AnalysisProvenance,
    minimum_required_records: usize,
) -> SensorHealthBaseline {
    let contextual = features
        .iter()
        .map(|(record_id, features)| (record_id.clone(), features.clone(), Context::default()))
        .collect::<Vec<_>>();
    build_with_contexts(id, &contextual, provenance, minimum_required_records)
}

/// Build a baseline while retaining and validating every record's context.
pub fn build_with_contexts(
    id: &str,
    records_with_context: &[(String, Vec<crate::results::HealthFeature>, Context)],
    provenance: crate::domain::AnalysisProvenance,
    minimum_required_records: usize,
) -> SensorHealthBaseline {
    let mut names = BTreeMap::<String, Vec<&crate::results::HealthFeature>>::new();
    let mut records = Vec::with_capacity(records_with_context.len());
    let mut represented_domains = BTreeSet::new();
    let mut metadata_sources = BTreeSet::new();

    for (record_id, fs, context) in records_with_context {
        let domains = fs.iter().map(|x| x.domain).collect::<BTreeSet<_>>();
        represented_domains.extend(domains.iter().copied());
        if let Some(source) = &context.metadata_source {
            metadata_sources.insert(source.clone());
        }
        records.push(BaselineRecordSummary {
            record_id: record_id.clone(),
            experiment_id: context.experiment_id.clone(),
            sensor_id: context.sensor_id.clone(),
            sensor_type: context.sensor_type.clone(),
            analyte: context.analyte.clone(),
            sample_matrix: context.sample_matrix.clone(),
            temperature_k: context.temperature_k,
            sensor_design: context.sensor_design.clone(),
            domains: domains.into_iter().collect(),
            metadata_source: context.metadata_source.clone(),
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
                empirical_values: x,
            }
        })
        .collect();

    let mut warnings = Vec::new();
    if records.len() < minimum_required_records || records.len() < 2 {
        warnings.push(HealthWarning::InsufficientBaselineRecords);
    }

    let (sensor_type, sensor_type_conflict) =
        consistent_context("sensor_type", records_with_context, |c| {
            c.sensor_type.clone()
        });
    let (sensor_design, sensor_design_conflict) =
        consistent_context("sensor_design", records_with_context, |c| {
            c.sensor_design.clone()
        });
    let (analyte, analyte_conflict) =
        consistent_context("analyte", records_with_context, |c| c.analyte.clone());
    let (sample_matrix, matrix_conflict) =
        consistent_context("sample_matrix", records_with_context, |c| {
            c.sample_matrix.clone()
        });
    let mut conflicts = Vec::new();
    for conflict in [
        sensor_type_conflict,
        sensor_design_conflict,
        analyte_conflict,
        matrix_conflict,
    ]
    .into_iter()
    .flatten()
    {
        warnings.push(match conflict.field.as_str() {
            "analyte" => HealthWarning::MixedAnalyteContext,
            "sample_matrix" => HealthWarning::MixedSampleMatrixContext,
            "sensor_design" => HealthWarning::MixedSensorDesignContext,
            _ => HealthWarning::MixedSensorTypeContext,
        });
        conflicts.push(conflict);
    }

    let temperatures = records_with_context
        .iter()
        .flat_map(|(_, _, c)| {
            if c.temperature_values_k.is_empty() {
                c.temperature_k.into_iter().collect::<Vec<_>>()
            } else {
                c.temperature_values_k.clone()
            }
        })
        .filter(|v| v.is_finite())
        .collect::<Vec<_>>();
    let temperature_domain_k = temperatures
        .iter()
        .copied()
        .reduce(f64::min)
        .zip(temperatures.iter().copied().reduce(f64::max));
    if temperature_domain_k.is_some_and(|(min, max)| min != max) {
        warnings.push(HealthWarning::MixedTemperatureContext);
    }

    SensorHealthBaseline {
        schema_version: 2,
        baseline_id: id.into(),
        sensor_type,
        sensor_design,
        analyte,
        sample_matrix,
        temperature_domain_k,
        feature_distributions: distributions,
        records,
        minimum_required_records,
        represented_domains: represented_domains.into_iter().collect(),
        legacy_minimum_required_domains: None,
        context_conflicts: conflicts,
        metadata_sources: metadata_sources.into_iter().collect(),
        provenance,
        warnings,
    }
}

fn consistent_context(
    field: &str,
    records: &[(String, Vec<crate::results::HealthFeature>, Context)],
    get: impl Fn(&Context) -> Option<String>,
) -> (Option<String>, Option<BaselineContextConflict>) {
    let values = records
        .iter()
        .filter_map(|(id, _, context)| get(context).map(|value| (id, value)))
        .collect::<Vec<_>>();
    let unique = values
        .iter()
        .map(|(_, value)| value.clone())
        .collect::<BTreeSet<_>>();
    if unique.len() <= 1 {
        return (unique.into_iter().next(), None);
    }
    let record_ids = values.into_iter().map(|(id, _)| id.clone()).collect();
    (
        None,
        Some(BaselineContextConflict {
            field: field.into(),
            values: unique.into_iter().collect(),
            record_ids,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{Context, build_with_contexts};
    use crate::{
        domain::AnalysisProvenance,
        results::{HealthDomain, HealthFeature, HealthWarning, SensorHealthBaseline},
    };
    use std::path::PathBuf;

    fn provenance() -> AnalysisProvenance {
        AnalysisProvenance {
            software_version: "test".into(),
            input_path: PathBuf::from("input.csv"),
            input_sha256: "input".into(),
            configuration_path: None,
            configuration_sha256: None,
            generation_timestamp: 1,
            git_commit: None,
        }
    }

    fn feature(value: f64) -> HealthFeature {
        HealthFeature {
            name: "signal.mean".into(),
            value: Some(value),
            unit: "V".into(),
            domain: HealthDomain::SignalNoise,
            source: "test".into(),
            warning: None,
        }
    }

    fn context(analyte: &str, matrix: &str, design: &str, temperature_k: f64) -> Context {
        Context {
            sensor_id: Some("s1".into()),
            sensor_type: Some("ise".into()),
            sensor_design: Some(design.into()),
            analyte: Some(analyte.into()),
            sample_matrix: Some(matrix.into()),
            temperature_k: Some(temperature_k),
            temperature_values_k: vec![temperature_k],
            experiment_id: Some("experiment".into()),
            metadata_source: Some("metadata.toml".into()),
        }
    }

    #[test]
    fn same_context_and_temperature_domain_are_retained() {
        let rows = vec![
            (
                "r1".into(),
                vec![feature(1.0)],
                context("K+", "buffer", "design-a", 298.15),
            ),
            (
                "r2".into(),
                vec![feature(2.0)],
                context("K+", "buffer", "design-a", 300.15),
            ),
        ];
        let baseline = build_with_contexts("b", &rows, provenance(), 3);
        assert_eq!(baseline.analyte.as_deref(), Some("K+"));
        assert_eq!(baseline.sample_matrix.as_deref(), Some("buffer"));
        assert_eq!(baseline.sensor_design.as_deref(), Some("design-a"));
        assert_eq!(baseline.minimum_required_records, 3);
        assert_eq!(
            baseline.represented_domains,
            vec![HealthDomain::SignalNoise]
        );
        assert_eq!(baseline.temperature_domain_k, Some((298.15, 300.15)));
        assert!(
            baseline
                .warnings
                .contains(&HealthWarning::MixedTemperatureContext)
        );
        assert_eq!(
            baseline.records[0].metadata_source.as_deref(),
            Some("metadata.toml")
        );
    }

    #[test]
    fn conflicting_context_is_explicit_and_identifies_records() {
        let mut rows = vec![
            (
                "r1".into(),
                vec![feature(1.0)],
                context("K+", "buffer", "design-a", 298.15),
            ),
            (
                "r2".into(),
                vec![feature(2.0)],
                context("Na+", "sample", "design-b", 298.15),
            ),
        ];
        rows[1].2.sensor_type = Some("other-ise".into());
        let baseline = build_with_contexts("b", &rows, provenance(), 2);
        assert!(baseline.analyte.is_none());
        assert!(baseline.sample_matrix.is_none());
        assert!(baseline.sensor_design.is_none());
        assert!(
            baseline
                .warnings
                .contains(&HealthWarning::MixedAnalyteContext)
        );
        assert!(
            baseline
                .warnings
                .contains(&HealthWarning::MixedSampleMatrixContext)
        );
        assert!(
            baseline
                .warnings
                .contains(&HealthWarning::MixedSensorDesignContext)
        );
        assert!(
            baseline
                .context_conflicts
                .iter()
                .all(|c| c.record_ids == vec!["r1", "r2"])
        );
    }

    #[test]
    fn old_schema_deserializes_without_reinterpreting_domain_field() {
        let old = r#"{
            "schema_version": 1,
            "baseline_id": "old",
            "sensor_type": null,
            "analyte": null,
            "sample_matrix": null,
            "temperature_domain_k": null,
            "feature_distributions": [],
            "records": [],
            "minimum_required_domains": 7,
            "provenance": {
                "software_version": "old",
                "input_path": "input.csv",
                "input_sha256": "x",
                "configuration_path": null,
                "configuration_sha256": null,
                "generation_timestamp": 1,
                "git_commit": null
            },
            "warnings": []
        }"#;
        let baseline: SensorHealthBaseline = serde_json::from_str(old).unwrap();
        assert_eq!(baseline.legacy_minimum_required_domains, Some(7));
        assert_eq!(baseline.minimum_required_records, 0);
        assert!(baseline.represented_domains.is_empty());
    }
}
