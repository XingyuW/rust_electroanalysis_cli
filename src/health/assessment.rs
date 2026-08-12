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
                    status: if findings.iter().any(|finding| {
                        finding
                            .supporting_evidence
                            .iter()
                            .any(|evidence| evidence.domain == *d)
                    }) {
                        finding_status(
                            findings
                                .iter()
                                .filter(|finding| {
                                    finding
                                        .supporting_evidence
                                        .iter()
                                        .any(|evidence| evidence.domain == *d)
                                })
                                .map(|finding| &finding.severity),
                        )
                    } else if comparisons.iter().any(|comparison| {
                        fs.iter().any(|feature| feature.name == comparison.feature)
                            && (comparison.robust_z_score.is_some_and(|z| z.abs() >= 3.0)
                                || comparison
                                    .relative_difference
                                    .is_some_and(|x| x.abs() >= 0.5))
                    }) || fs.iter().any(|feature| feature.warning.is_some())
                    {
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
    let mut assessment = SensorHealthAssessment {
        schema_version: 3,
        lineage: crate::domain::current_unknown_lineage(3),
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
    };
    assessment.lineage = crate::domain::known_lineage_from_artifact(
        crate::domain::ArtifactKind::HealthAssessment,
        assessment.schema_version,
        format!("rust_electroanalysis_cli@{}", env!("CARGO_PKG_VERSION")),
        assessment
            .experiment_id
            .clone()
            .and_then(|id| crate::domain::ExperimentId::new(id).ok())
            .and_then(|id| crate::domain::ArtifactExperimentScope::single(id).ok())
            .unwrap_or(crate::domain::ArtifactExperimentScope::Unknown),
        assessment
            .sensor_id
            .clone()
            .and_then(|id| crate::domain::ScopeKey::specific(id).ok())
            .unwrap_or(crate::domain::ScopeKey::Unspecified),
        crate::domain::ScopeKey::Unspecified,
        crate::domain::ArtifactAcquisitionFamilies::Unknown,
        Vec::new(),
        &assessment,
    )
    .unwrap_or_else(|_| crate::domain::current_unknown_lineage(3));
    assessment
}

fn finding_status<'a>(
    severities: impl Iterator<Item = &'a crate::health_config::HealthSeverity>,
) -> OverallHealthStatus {
    let severities = severities.collect::<Vec<_>>();
    if severities
        .iter()
        .any(|severity| matches!(severity, crate::health_config::HealthSeverity::Critical))
    {
        OverallHealthStatus::Critical
    } else if severities
        .iter()
        .any(|severity| matches!(severity, crate::health_config::HealthSeverity::Major))
    {
        OverallHealthStatus::Degraded
    } else {
        OverallHealthStatus::Watch
    }
}
