use super::baseline::Context;
use crate::{
    health_config::{ComparabilityConfig, NormalizationConfig},
    results::{
        BaselineComparison, BaselineFeatureDistribution, FeatureComparability, HealthFeature,
    },
};

/// Compare one feature using only scientifically supported statistics.
pub fn compare(
    feature: &HealthFeature,
    baseline: Option<&BaselineFeatureDistribution>,
    comparability: FeatureComparability,
) -> BaselineComparison {
    compare_with_config(
        feature,
        baseline,
        comparability,
        &NormalizationConfig::default(),
        None,
    )
}

/// Compare a feature with configured normalization and an optional documented
/// comparability override. A non-comparable feature has no numerical baseline
/// comparison unless that override is explicitly supplied by the caller.
pub fn compare_with_config(
    feature: &HealthFeature,
    baseline: Option<&BaselineFeatureDistribution>,
    comparability: FeatureComparability,
    config: &NormalizationConfig,
    documented_override_reason: Option<&str>,
) -> BaselineComparison {
    let allowed = !matches!(comparability, FeatureComparability::NotComparable)
        || documented_override_reason.is_some_and(|reason| !reason.trim().is_empty());
    let b = baseline.and_then(|x| x.mean);
    let current = feature.value;
    let sample_count = baseline.map(|x| x.sample_count).unwrap_or(0);
    let enough_for_z = sample_count >= config.minimum_baseline_records_for_z_score;

    let abs = allowed
        .then(|| current.zip(b))
        .flatten()
        .map(|(x, y)| x - y);
    let rel = if allowed && config.use_relative_difference {
        current
            .zip(b)
            .and_then(|(x, y)| (y != 0.0).then_some((x - y) / y))
    } else {
        None
    };
    let log = allowed
        .then(|| {
            current
                .zip(b)
                .and_then(|(x, y)| (x > 0.0 && y > 0.0).then_some((x / y).ln()))
        })
        .flatten();
    let z = if allowed && enough_for_z {
        current
            .zip(b)
            .zip(baseline.and_then(|x| x.standard_deviation))
            .and_then(|((x, y), s)| (s.is_finite() && s > 0.0).then_some((x - y) / s))
    } else {
        None
    };
    let rz = if allowed && config.use_robust_z_score && enough_for_z {
        current
            .zip(baseline.and_then(|x| x.median))
            .zip(baseline.and_then(|x| x.mad))
            .and_then(|((x, median), mad)| {
                (mad.is_finite() && mad > 0.0).then_some((x - median) / (1.4826 * mad))
            })
    } else {
        None
    };
    let empirical_percentile = if allowed {
        current.and_then(|x| {
            let values = baseline?.empirical_values.as_slice();
            (!values.is_empty()).then_some(
                100.0 * values.iter().filter(|value| **value <= x).count() as f64
                    / values.len() as f64,
            )
        })
    } else {
        None
    };
    let range_position_percent = if allowed {
        current.and_then(|x| {
            baseline.and_then(|d| {
                let min = d.minimum?;
                let max = d.maximum?;
                (max > min).then_some((x - min) / (max - min) * 100.0)
            })
        })
    } else {
        None
    };

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
        empirical_percentile,
        range_position_percent,
        override_reason: documented_override_reason.map(str::to_string),
    }
}

pub fn comparable(
    current: &Context,
    base: &Context,
    config: &ComparabilityConfig,
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
            FeatureComparability::NotComparable,
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

#[cfg(test)]
mod tests {
    use super::{comparable, compare_with_config};
    use crate::{
        health::baseline::Context,
        health_config::{ComparabilityConfig, NormalizationConfig},
        results::{BaselineFeatureDistribution, FeatureComparability, HealthDomain, HealthFeature},
    };

    fn feature(value: f64) -> HealthFeature {
        HealthFeature {
            name: "feature".into(),
            value: Some(value),
            unit: "V".into(),
            domain: HealthDomain::SignalNoise,
            source: "test".into(),
            warning: None,
        }
    }

    fn distribution() -> BaselineFeatureDistribution {
        BaselineFeatureDistribution {
            feature: "feature".into(),
            unit: "V".into(),
            domain: HealthDomain::SignalNoise,
            sample_count: 5,
            mean: Some(12.0),
            standard_deviation: Some(4.0),
            median: Some(2.0),
            mad: Some(1.0),
            quantiles: Vec::new(),
            minimum: Some(1.0),
            maximum: Some(100.0),
            reference_direction: None,
            comparison_context: None,
            empirical_values: vec![1.0, 2.0, 2.0, 3.0, 100.0],
        }
    }

    #[test]
    fn robust_z_uses_median_and_mad() {
        let c = compare_with_config(
            &feature(5.0),
            Some(&distribution()),
            FeatureComparability::Comparable,
            &NormalizationConfig::default(),
            None,
        );
        assert!((c.robust_z_score.unwrap() - (3.0 / 1.4826)).abs() < 1e-12);
        assert_eq!(c.empirical_percentile, Some(80.0));
        assert!((c.range_position_percent.unwrap() - 4.04040404040404).abs() < 1e-12);
    }

    #[test]
    fn minimum_count_and_zero_mad_suppress_z_scores() {
        let config = NormalizationConfig {
            minimum_baseline_records_for_z_score: 6,
            ..Default::default()
        };
        let c = compare_with_config(
            &feature(5.0),
            Some(&distribution()),
            FeatureComparability::Comparable,
            &config,
            None,
        );
        assert!(c.z_score.is_none());
        assert!(c.robust_z_score.is_none());
        let mut d = distribution();
        d.mad = Some(0.0);
        let c = compare_with_config(
            &feature(5.0),
            Some(&d),
            FeatureComparability::Comparable,
            &NormalizationConfig::default(),
            None,
        );
        assert!(c.robust_z_score.is_none());
    }

    #[test]
    fn noncomparable_has_no_numerical_comparisons_without_override() {
        let c = compare_with_config(
            &feature(5.0),
            Some(&distribution()),
            FeatureComparability::NotComparable,
            &NormalizationConfig::default(),
            None,
        );
        assert!(c.absolute_difference.is_none());
        assert!(c.relative_difference.is_none());
        assert!(c.log_ratio.is_none());
        assert!(c.z_score.is_none());
        assert!(c.robust_z_score.is_none());
        assert!(c.empirical_percentile.is_none());
        let overridden = compare_with_config(
            &feature(5.0),
            Some(&distribution()),
            FeatureComparability::NotComparable,
            &NormalizationConfig::default(),
            Some("validated same ionic strength; approved override EXP-1"),
        );
        assert_eq!(overridden.absolute_difference, Some(-7.0));
        assert!(overridden.override_reason.is_some());
    }

    #[test]
    fn comparability_preserves_noncomparable_reason() {
        let (status, reason) = comparable(
            &Context {
                analyte: Some("K+".into()),
                ..Default::default()
            },
            &Context {
                analyte: Some("Na+".into()),
                ..Default::default()
            },
            &ComparabilityConfig::default(),
        );
        assert_eq!(status, FeatureComparability::NotComparable);
        assert_eq!(reason.as_deref(), Some("analyte differs"));
    }
}
