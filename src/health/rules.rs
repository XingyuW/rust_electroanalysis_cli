use crate::{
    health_config::{FeatureCondition, FeatureOperator, HealthRule},
    results::{
        BaselineComparison, HealthConfidence, HealthEvidence, HealthFeature, HealthFinding,
        RuleEvaluation,
    },
};
use std::collections::BTreeSet;
pub fn evaluate(
    rules: &[HealthRule],
    features: &[HealthFeature],
    comparisons: &[BaselineComparison],
    minimum_mechanistic_domains: usize,
) -> (Vec<RuleEvaluation>, Vec<HealthFinding>) {
    let mut evaluations = Vec::new();
    let mut findings = Vec::new();
    for rule in rules {
        let mut ok = Vec::new();
        let mut no = Vec::new();
        let mut unavailable = Vec::new();
        let mut domains = BTreeSet::new();
        for c in rule.all_of.iter().chain(rule.any_of.iter()) {
            let result = condition(c, features, comparisons);
            match result {
                Some(true) => {
                    ok.push(c.feature.clone());
                    if let Some(f) = features.iter().find(|f| f.name == c.feature) {
                        domains.insert(f.domain);
                    }
                }
                Some(false) => no.push(c.feature.clone()),
                None => unavailable.push(c.feature.clone()),
            }
        }
        let all_ok = rule
            .all_of
            .iter()
            .all(|c| condition(c, features, comparisons) == Some(true));
        let any_ok = rule.any_of.is_empty()
            || rule
                .any_of
                .iter()
                .any(|c| condition(c, features, comparisons) == Some(true));
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
        let evidence = ok
            .iter()
            .filter_map(|name| features.iter().find(|f| &f.name == name))
            .map(|f| HealthEvidence {
                domain: f.domain,
                feature: f.name.clone(),
                statement: format!("{} satisfied configured rule condition", f.name),
                strength: if triggered {
                    HealthConfidence::Moderate
                } else {
                    HealthConfidence::Low
                },
                source: f.source.clone(),
            })
            .collect::<Vec<_>>();
        let eval = RuleEvaluation {
            rule_id: rule.rule_id.clone(),
            conditions_satisfied: ok.clone(),
            conditions_not_satisfied: no,
            conditions_unavailable: unavailable.clone(),
            evidence_domains: domains.iter().copied().collect(),
            supporting_evidence: evidence.clone(),
            contradictory_evidence: Vec::new(),
            severity: rule.severity.clone(),
            confidence: if triggered {
                if domains.len() >= 3 {
                    HealthConfidence::High
                } else {
                    HealthConfidence::Moderate
                }
            } else {
                HealthConfidence::Insufficient
            },
            triggered,
        };
        if triggered {
            findings.push(HealthFinding {
                finding: rule.finding.clone(),
                severity: rule.severity.clone(),
                confidence: eval.confidence,
                supporting_evidence: evidence,
                contradictory_evidence: Vec::new(),
                unavailable_evidence: unavailable,
                alternative_explanations: rule.alternative_explanations.clone(),
                triggered_rules: vec![rule.rule_id.clone()],
            });
        }
        evaluations.push(eval);
    }
    (evaluations, findings)
}
fn condition(
    c: &FeatureCondition,
    features: &[HealthFeature],
    comparisons: &[BaselineComparison],
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
        FeatureOperator::TrendIncreasing | FeatureOperator::TrendDecreasing => None,
    }
}
