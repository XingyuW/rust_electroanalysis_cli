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
                    status: if fs.iter().any(|f| f.warning.is_some()) {
                        OverallHealthStatus::Watch
                    } else {
                        OverallHealthStatus::WithinBaseline
                    },
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
        schema_version: 1,
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
