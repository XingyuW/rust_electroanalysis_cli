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
    evaluate_with_baseline_records(rules, features, comparisons, minimum_mechanistic_domains, 0)
}

/// Evaluate rules while enforcing each rule's stated baseline-record minimum.
/// The compatibility wrapper above cannot establish a baseline-record count and
/// therefore conservatively supplies zero; rules that require baseline history
/// must use this explicit API.
pub fn evaluate_with_baseline_records(
    rules: &[HealthRule],
    features: &[HealthFeature],
    comparisons: &[BaselineComparison],
    minimum_mechanistic_domains: usize,
    baseline_records: usize,
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
        let baseline_ok = baseline_records >= rule.minimum_baseline_records;
        if !baseline_ok {
            unavailable.push(format!(
                "baseline records: required {}, available {}",
                rule.minimum_baseline_records, baseline_records
            ));
        }
        let triggered =
            all_ok && any_ok && unavailable.is_empty() && domains.len() >= required && baseline_ok;
        let evidence = ok
            .iter()
            .map(|name| evidence(name, features, trends, true, triggered))
            .collect::<Vec<_>>();
        let contradictory_evidence = no
            .iter()
            .filter_map(|name| features.iter().find(|feature| &feature.name == name))
            .map(|feature| HealthEvidence {
                domain: feature.domain,
                feature: feature.name.clone(),
                statement: format!("{} contradicts the configured rule condition", feature.name),
                strength: HealthConfidence::Low,
                source: feature.source.clone(),
            })
            .collect::<Vec<_>>();
        let eval = RuleEvaluation {
            rule_id: rule.rule_id.clone(),
            conditions_satisfied: ok,
            conditions_not_satisfied: no,
            conditions_unavailable: unavailable.clone(),
            evidence_domains: domains.iter().copied().collect(),
            supporting_evidence: evidence.clone(),
            contradictory_evidence: contradictory_evidence.clone(),
            severity: rule.severity.clone(),
            confidence,
            triggered,
        };
        if triggered {
            findings.push(HealthFinding {
                finding: rule.finding.clone(),
                severity: rule.severity.clone(),
                confidence: eval.confidence,
                supporting_evidence: evidence,
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
    let f = features.iter().find(|f| f.name == c.feature);
    let b = comparisons.iter().find(|b| b.feature == c.feature);
    match c.operator {
        FeatureOperator::WarningPresent => f.map(|x| x.warning.is_some()),
        FeatureOperator::EvidenceLevelPresent => f.map(|x| x.value.is_some()),
        FeatureOperator::GreaterThan => f.and_then(|x| x.value).zip(c.value).map(|(x, v)| x > v),
        FeatureOperator::LessThan => f.and_then(|x| x.value).zip(c.value).map(|(x, v)| x < v),
        FeatureOperator::RelativeIncreaseGreaterThan => b
            .and_then(|x| x.relative_difference)
            .zip(c.value)
            .map(|(x, v)| x > v),
        FeatureOperator::RelativeDecreaseGreaterThan => b
            .and_then(|x| x.relative_difference)
            .zip(c.value)
            .map(|(x, v)| x < -v),
        FeatureOperator::LogRatioGreaterThan => b
            .and_then(|x| x.log_ratio)
            .zip(c.value)
            .map(|(x, v)| x.abs() > v),
        FeatureOperator::RobustZGreaterThan => b
            .and_then(|x| x.robust_z_score)
            .zip(c.value)
            .map(|(x, v)| x.abs() > v),
        FeatureOperator::TrendIncreasing => features
            .iter()
            .find(|feature| feature.name == format!("trend.{}", c.feature))
            .and_then(|feature| feature.value)
            .map(|value| value > 0.0),
        FeatureOperator::TrendDecreasing => features
            .iter()
            .find(|feature| feature.name == format!("trend.{}", c.feature))
            .and_then(|feature| feature.value)
            .map(|value| value < 0.0),
    }
}
