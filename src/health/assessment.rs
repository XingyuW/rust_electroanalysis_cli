use crate::{
    domain::AnalysisProvenance,
    health_config::ResolvedHealthConfig,
    results::{
        BaselineComparison, HealthDomain, HealthDomainAssessment, HealthFeature, HealthFinding,
        HealthWarning, OverallHealthStatus, RuleEvaluation, SensorHealthAssessment,
    },
};
#[allow(clippy::too_many_arguments)]
pub fn assemble(
    id: &str,
    sensor: Option<String>,
    experiment: Option<String>,
    features: Vec<HealthFeature>,
    comparisons: Vec<BaselineComparison>,
    rules: Vec<RuleEvaluation>,
    mut findings: Vec<HealthFinding>,
    missing: Vec<HealthDomain>,
    config: ResolvedHealthConfig,
    provenance: AnalysisProvenance,
    mut warnings: Vec<HealthWarning>,
) -> SensorHealthAssessment {
    let domains = [
        HealthDomain::DataQuality,
        HealthDomain::SignalNoise,
        HealthDomain::Drift,
        HealthDomain::DynamicResponse,
        HealthDomain::Calibration,
        HealthDomain::Impedance,
        HealthDomain::MechanismEvidence,
    ];
    if rules
        .iter()
        .any(|rule| !rule.contradictory_evidence.is_empty())
        && !warnings.contains(&HealthWarning::ContradictoryEvidence)
    {
        warnings.push(HealthWarning::ContradictoryEvidence);
    }
    let assessments = domains
        .iter()
        .filter_map(|d| {
            let fs = features
                .iter()
                .filter(|f| f.domain == *d)
                .collect::<Vec<_>>();
            if fs.is_empty() && !missing.contains(d) {
                None
            } else {
                Some(HealthDomainAssessment {
                    domain: *d,
                    status: domain_status(*d, &fs, &comparisons, &findings, missing.contains(d)),
                    confidence: if fs.iter().any(|f| f.value.is_some()) {
                        crate::results::HealthConfidence::Moderate
                    } else {
                        crate::results::HealthConfidence::Insufficient
                    },
                    feature_count: fs.len(),
                    available_features: fs.iter().filter(|f| f.value.is_some()).count(),
                    warning_count: fs.iter().filter(|f| f.warning.is_some()).count(),
                })
            }
        })
        .collect();
    if !missing.is_empty() {
        warnings.push(HealthWarning::InsufficientEvidenceDomains);
    }
    let available_domains = features
        .iter()
        .map(|f| f.domain)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let status = if available_domains < config.assessment.minimum_domains_for_assessment {
        OverallHealthStatus::DataQualityInsufficient
    } else if findings
        .iter()
        .any(|f| matches!(f.severity, crate::health_config::HealthSeverity::Critical))
    {
        OverallHealthStatus::Critical
    } else if findings
        .iter()
        .any(|f| matches!(f.severity, crate::health_config::HealthSeverity::Major))
    {
        OverallHealthStatus::Degraded
    } else if !findings.is_empty() {
        OverallHealthStatus::Watch
    } else if missing.len() >= config.assessment.minimum_domains_for_assessment {
        OverallHealthStatus::Indeterminate
    } else {
        OverallHealthStatus::WithinBaseline
    };
    SensorHealthAssessment {
        schema_version: 2,
        assessment_id: id.into(),
        sensor_id: sensor,
        experiment_id: experiment,
        overall_status: status,
        domain_assessments: assessments,
        features,
        findings: std::mem::take(&mut findings),
        rule_evaluations: rules,
        baseline_comparison: comparisons,
        missing_domains: missing,
        configuration: config,
        provenance,
        warnings,
    }
}

fn domain_status(
    domain: HealthDomain,
    features: &[&HealthFeature],
    comparisons: &[BaselineComparison],
    findings: &[HealthFinding],
    missing: bool,
) -> OverallHealthStatus {
    if missing || features.iter().all(|feature| feature.value.is_none()) {
        return OverallHealthStatus::DataQualityInsufficient;
    }
    let severity = findings
        .iter()
        .filter(|finding| {
            finding
                .supporting_evidence
                .iter()
                .any(|evidence| evidence.domain == domain)
        })
        .map(|finding| &finding.severity)
        .max_by_key(|severity| match severity {
            crate::health_config::HealthSeverity::Informational => 0,
            crate::health_config::HealthSeverity::Minor => 1,
            crate::health_config::HealthSeverity::Moderate => 2,
            crate::health_config::HealthSeverity::Major => 3,
            crate::health_config::HealthSeverity::Critical => 4,
        });
    match severity {
        Some(crate::health_config::HealthSeverity::Critical) => OverallHealthStatus::Critical,
        Some(crate::health_config::HealthSeverity::Major) => OverallHealthStatus::Degraded,
        Some(_) => OverallHealthStatus::Watch,
        None if comparisons
            .iter()
            .filter(|comparison| {
                features
                    .iter()
                    .any(|feature| feature.name == comparison.feature)
            })
            .any(|comparison| {
                comparison
                    .robust_z_score
                    .is_some_and(|value| value.abs() >= 3.0)
                    || comparison.z_score.is_some_and(|value| value.abs() >= 3.0)
                    || comparison
                        .relative_difference
                        .is_some_and(|value| value.abs() >= 0.20)
            }) =>
        {
            OverallHealthStatus::Watch
        }
        None if features.iter().any(|feature| feature.warning.is_some()) => {
            OverallHealthStatus::Watch
        }
        None => OverallHealthStatus::WithinBaseline,
    }
}
