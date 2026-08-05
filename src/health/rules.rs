use crate::{
    health_config::{FeatureCondition, FeatureOperator, HealthRule},
    results::{
        BaselineComparison, HealthConfidence, HealthDomain, HealthEvidence, HealthFeature,
        HealthFinding, HealthTrend, RuleEvaluation,
    },
};
use std::collections::BTreeSet;

/// Evaluate rules without longitudinal evidence. Trend operators are reported
/// unavailable rather than being approximated from a single observation.
pub fn evaluate(
    rules: &[HealthRule],
    features: &[HealthFeature],
    comparisons: &[BaselineComparison],
    minimum_mechanistic_domains: usize,
) -> (Vec<RuleEvaluation>, Vec<HealthFinding>) {
    evaluate_with_trends(
        rules,
        features,
        comparisons,
        &[],
        minimum_mechanistic_domains,
    )
}

/// Evaluate rules using explicit trend artifacts when they are available.
pub fn evaluate_with_trends(
    rules: &[HealthRule],
    features: &[HealthFeature],
    comparisons: &[BaselineComparison],
    trends: &[HealthTrend],
    minimum_mechanistic_domains: usize,
) -> (Vec<RuleEvaluation>, Vec<HealthFinding>) {
    let mut evaluations = Vec::new();
    let mut findings = Vec::new();
    for rule in rules {
        let mut ok = Vec::new();
        let mut no = Vec::new();
        let mut unavailable = Vec::new();
        let mut domains = BTreeSet::new();
        for condition in rule.all_of.iter().chain(rule.any_of.iter()) {
            let result = condition_result(
                condition,
                features,
                comparisons,
                trends,
                rule.minimum_baseline_records,
            );
            match result {
                Some(true) => {
                    ok.push(condition.feature.clone());
                    domains.insert(domain_for(condition.feature.as_str(), features));
                }
                Some(false) => no.push(condition.feature.clone()),
                None => unavailable.push(condition.feature.clone()),
            }
        }
        let all_ok = rule.all_of.iter().all(|condition| {
            condition_result(
                condition,
                features,
                comparisons,
                trends,
                rule.minimum_baseline_records,
            ) == Some(true)
        });
        let any_ok = rule.any_of.is_empty()
            || rule.any_of.iter().any(|condition| {
                condition_result(
                    condition,
                    features,
                    comparisons,
                    trends,
                    rule.minimum_baseline_records,
                ) == Some(true)
            });
        let required = rule.minimum_evidence_domains.max(
            if matches!(
                rule.finding,
                crate::health_config::HealthFindingKind::ProbableFouling
                    | crate::health_config::HealthFindingKind::ProbableReferenceInstability
                    | crate::health_config::HealthFindingKind::ProbableContactIssue
            ) {
                minimum_mechanistic_domains
            } else {
                0
            },
        );
        let triggered = all_ok && any_ok && unavailable.is_empty() && domains.len() >= required;
        let supporting_evidence = ok
            .iter()
            .map(|name| evidence(name, features, trends, true, triggered))
            .collect::<Vec<_>>();
        let contradictory_evidence = no
            .iter()
            .map(|name| evidence(name, features, trends, false, false))
            .collect::<Vec<_>>();
        let confidence = if triggered {
            let base = if domains.len() >= 3 {
                HealthConfidence::High
            } else {
                HealthConfidence::Moderate
            };
            lower_for_contradictions(base, contradictory_evidence.len())
        } else {
            HealthConfidence::Insufficient
        };
        let evaluation = RuleEvaluation {
            rule_id: rule.rule_id.clone(),
            conditions_satisfied: ok,
            conditions_not_satisfied: no,
            conditions_unavailable: unavailable.clone(),
            evidence_domains: domains.iter().copied().collect(),
            supporting_evidence: supporting_evidence.clone(),
            contradictory_evidence: contradictory_evidence.clone(),
            severity: rule.severity.clone(),
            confidence,
            triggered,
        };
        if triggered {
            findings.push(HealthFinding {
                finding: rule.finding.clone(),
                severity: rule.severity.clone(),
                confidence,
                supporting_evidence,
                contradictory_evidence,
                unavailable_evidence: unavailable,
                alternative_explanations: rule.alternative_explanations.clone(),
                triggered_rules: vec![rule.rule_id.clone()],
            });
        }
        evaluations.push(evaluation);
    }
    (evaluations, findings)
}

fn condition_result(
    condition: &FeatureCondition,
    features: &[HealthFeature],
    comparisons: &[BaselineComparison],
    trends: &[HealthTrend],
    minimum_baseline_records: usize,
) -> Option<bool> {
    let feature = features
        .iter()
        .find(|feature| feature.name == condition.feature);
    let baseline = comparisons
        .iter()
        .find(|comparison| comparison.feature == condition.feature);
    let baseline_sufficient = || {
        minimum_baseline_records == 0
            || baseline.is_some_and(|value| value.baseline_sample_count >= minimum_baseline_records)
    };
    match condition.operator {
        FeatureOperator::WarningPresent => feature.map(|value| value.warning.is_some()),
        FeatureOperator::EvidenceLevelPresent => feature.map(|value| value.value.is_some()),
        FeatureOperator::GreaterThan => feature
            .and_then(|value| value.value)
            .zip(condition.value)
            .map(|(value, threshold)| value > threshold),
        FeatureOperator::LessThan => feature
            .and_then(|value| value.value)
            .zip(condition.value)
            .map(|(value, threshold)| value < threshold),
        FeatureOperator::RelativeIncreaseGreaterThan if baseline_sufficient() => baseline
            .and_then(|value| value.relative_difference)
            .zip(condition.value)
            .map(|(value, threshold)| value > threshold),
        FeatureOperator::RelativeDecreaseGreaterThan if baseline_sufficient() => baseline
            .and_then(|value| value.relative_difference)
            .zip(condition.value)
            .map(|(value, threshold)| value < -threshold),
        FeatureOperator::LogRatioGreaterThan if baseline_sufficient() => baseline
            .and_then(|value| value.log_ratio)
            .zip(condition.value)
            .map(|(value, threshold)| value.abs() > threshold),
        FeatureOperator::RobustZGreaterThan if baseline_sufficient() => baseline
            .and_then(|value| value.robust_z_score)
            .zip(condition.value)
            .map(|(value, threshold)| value.abs() > threshold),
        FeatureOperator::TrendIncreasing => trends
            .iter()
            .find(|trend| trend.feature == condition.feature)
            .and_then(|trend| trend.theil_sen_slope.or(trend.ordinary_slope))
            .map(|slope| slope > condition.value.unwrap_or(0.0)),
        FeatureOperator::TrendDecreasing => trends
            .iter()
            .find(|trend| trend.feature == condition.feature)
            .and_then(|trend| trend.theil_sen_slope.or(trend.ordinary_slope))
            .map(|slope| slope < -condition.value.unwrap_or(0.0)),
        _ => None,
    }
}

fn evidence(
    name: &str,
    features: &[HealthFeature],
    trends: &[HealthTrend],
    supports: bool,
    triggered: bool,
) -> HealthEvidence {
    let domain = domain_for(name, features);
    let source = features
        .iter()
        .find(|feature| feature.name == name)
        .map(|feature| feature.source.clone())
        .or_else(|| {
            trends
                .iter()
                .find(|trend| trend.feature == name)
                .map(|_| "health_trend".into())
        })
        .unwrap_or_else(|| "health_rule".into());
    HealthEvidence {
        domain,
        feature: name.into(),
        statement: if supports {
            format!("{name} satisfied configured rule condition")
        } else {
            format!("{name} contradicted a configured rule condition")
        },
        strength: if supports && triggered {
            HealthConfidence::Moderate
        } else {
            HealthConfidence::Low
        },
        source,
    }
}

fn domain_for(name: &str, features: &[HealthFeature]) -> HealthDomain {
    features
        .iter()
        .find(|feature| feature.name == name)
        .map(|feature| feature.domain)
        .unwrap_or_else(|| {
            if name.starts_with("transient") {
                HealthDomain::DynamicResponse
            } else if name.starts_with("calibration") {
                HealthDomain::Calibration
            } else if name.starts_with("eis") {
                HealthDomain::Impedance
            } else if name.starts_with("mechanism") {
                HealthDomain::MechanismEvidence
            } else if name.contains("drift") {
                HealthDomain::Drift
            } else {
                HealthDomain::DataQuality
            }
        })
}

fn lower_for_contradictions(
    confidence: HealthConfidence,
    contradictory_count: usize,
) -> HealthConfidence {
    if contradictory_count == 0 {
        return confidence;
    }
    match confidence {
        HealthConfidence::High => HealthConfidence::Moderate,
        HealthConfidence::Moderate => HealthConfidence::Low,
        other => other,
    }
}
