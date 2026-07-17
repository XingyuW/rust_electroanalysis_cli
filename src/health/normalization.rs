use super::baseline::Context;
use crate::results::{
    BaselineComparison, BaselineFeatureDistribution, FeatureComparability, HealthFeature,
};
pub fn compare(
    feature: &HealthFeature,
    baseline: Option<&BaselineFeatureDistribution>,
    comparability: FeatureComparability,
) -> BaselineComparison {
    let b = baseline.and_then(|x| x.mean);
    let current = feature.value;
    let abs = current.zip(b).map(|(x, y)| x - y);
    let rel = current
        .zip(b)
        .and_then(|(x, y)| (y != 0.0).then_some((x - y) / y));
    let log = current
        .zip(b)
        .and_then(|(x, y)| (x > 0.0 && y > 0.0).then_some((x / y).ln()));
    let z = current
        .zip(b)
        .zip(baseline.and_then(|x| x.standard_deviation))
        .and_then(|((x, y), s)| (s.is_finite() && s > 0.0).then_some((x - y) / s));
    let rz = current
        .zip(b)
        .zip(baseline.and_then(|x| x.mad))
        .and_then(|((x, y), s)| (s.is_finite() && s > 0.0).then_some((x - y) / (1.4826 * s)));
    let p = current.and_then(|x| {
        baseline.and_then(|d| {
            let min = d.minimum?;
            let max = d.maximum?;
            (max > min).then_some((x - min) / (max - min) * 100.0)
        })
    });
    BaselineComparison {
        feature: feature.name.clone(),
        current_value: current,
        baseline_value: b,
        comparability,
        absolute_difference: abs,
        relative_difference: rel,
        log_ratio: log,
        z_score: z,
        robust_z_score: rz,
        percentile_position: p,
        override_reason: None,
    }
}
pub fn comparable(
    current: &Context,
    base: &Context,
    config: &crate::health_config::ComparabilityConfig,
) -> (FeatureComparability, Option<String>) {
    let mut warnings: Vec<String> = Vec::new();
    if config.require_same_analyte && current.analyte != base.analyte {
        return (
            FeatureComparability::NotComparable,
            Some("analyte differs".into()),
        );
    }
    if config.require_same_sample_matrix && current.sample_matrix != base.sample_matrix {
        return (
            FeatureComparability::NotComparable,
            Some("sample matrix differs".into()),
        );
    }
    if let (Some(a), Some(b)) = (current.temperature_k, base.temperature_k) {
        if (a - b).abs() > config.maximum_temperature_difference_k {
            return (
                FeatureComparability::NotComparable,
                Some("temperature differs beyond configured context".into()),
            );
        }
        if a != b {
            warnings.push("temperature differs within configured tolerance".into());
        }
    }
    if config.require_same_sensor_design && current.sensor_design != base.sensor_design {
        return (
            FeatureComparability::Unknown,
            Some("sensor design is unavailable or differs".into()),
        );
    }
    if warnings.is_empty() {
        (FeatureComparability::Comparable, None)
    } else {
        (
            FeatureComparability::ComparableWithWarnings,
            Some(warnings.join("; ")),
        )
    }
}
