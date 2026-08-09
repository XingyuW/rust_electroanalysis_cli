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
            || rule.any_of.iter().any(|c| {
                condition(c, features, comparisons) == Some(true)
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
            conditions_satisfied: ok.clone(),
            conditions_not_satisfied: no,
            conditions_unavailable: unavailable.clone(),
            evidence_domains: domains.iter().copied().collect(),
            supporting_evidence: evidence.clone(),
            contradictory_evidence: contradictory_evidence.clone(),
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
                contradictory_evidence,
                unavailable_evidence: unavailable,
                alternative_explanations: rule.alternative_explanations.clone(),
                triggered_rules: vec![rule.rule_id.clone()],
            });
        }
        evaluations.push(eval);
    }
    (evaluations, findings)
}

/// Evaluate rules using explicit trend artifacts when they are available.
///
/// This compatibility path retains the trend-aware behavior from the platform
/// hardening work while the baseline-record API above remains the default for
/// non-longitudinal workflows.
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
            match trend_condition(condition, features, comparisons, trends, rule.minimum_baseline_records) {
                Some(true) => {
                    ok.push(condition.feature.clone());
                    domains.insert(domain_for(condition.feature.as_str(), features));
                }
                Some(false) => no.push(condition.feature.clone()),
                None => unavailable.push(condition.feature.clone()),
            }
        }
        let all_ok = rule.all_of.iter().all(|condition| {
            trend_condition(condition, features, comparisons, trends, rule.minimum_baseline_records)
                == Some(true)
        });
        let any_ok = rule.any_of.is_empty()
            || rule.any_of.iter().any(|condition| {
                trend_condition(condition, features, comparisons, trends, rule.minimum_baseline_records)
                    == Some(true)
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
            .map(|name| trend_evidence(name, features, trends, true, triggered))
            .collect::<Vec<_>>();
        let contradictory_evidence = no
            .iter()
            .map(|name| trend_evidence(name, features, trends, false, false))
            .collect::<Vec<_>>();
        let confidence = if triggered {
            lower_for_contradictions(
                if domains.len() >= 3 {
                    HealthConfidence::High
                } else {
                    HealthConfidence::Moderate
                },
                contradictory_evidence.len(),
            )
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

fn trend_condition(
    condition: &FeatureCondition,
    features: &[HealthFeature],
    comparisons: &[BaselineComparison],
    trends: &[HealthTrend],
    minimum_baseline_records: usize,
) -> Option<bool> {
    let feature = features.iter().find(|feature| feature.name == condition.feature);
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

fn trend_evidence(
    name: &str,
    features: &[HealthFeature],
    trends: &[HealthTrend],
    supports: bool,
    triggered: bool,
) -> HealthEvidence {
    let source = features
        .iter()
        .find(|feature| feature.name == name)
        .map(|feature| feature.source.clone())
        .or_else(|| trends.iter().find(|trend| trend.feature == name).map(|_| "health_trend".into()))
        .unwrap_or_else(|| "health_rule".into());
    HealthEvidence {
        domain: domain_for(name, features),
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
