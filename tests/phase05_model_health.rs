use rust_electroanalysis_cli::{
    data_file::EISData,
    domain::AnalysisProvenance,
    health::{
        features::{ModelHealthSnapshot, from_model},
        rules,
    },
    health_config::{
        FeatureCondition, FeatureOperator, HealthFindingKind, HealthRule, HealthSeverity,
    },
    mechanism::{assess_component_hypothesis, eis_prior},
    results::{
        CircuitFitResult, ComponentHypothesisDefinition, ComponentPriorMapping,
        ComponentPriorSource, EvidenceLevel, HealthDomain, HealthFeature,
    },
};
use std::{collections::BTreeMap, path::PathBuf};

fn provenance() -> AnalysisProvenance {
    AnalysisProvenance {
        software_version: "test".into(),
        input_path: PathBuf::from("test"),
        input_sha256: "0".repeat(64),
        configuration_path: None,
        configuration_sha256: None,
        generation_timestamp: 0,
        git_commit: None,
    }
}

fn eis() -> rust_electroanalysis_cli::results::EisFitArtifact {
    let input = EISData {
        date: String::new(),
        test_type: "EIS".into(),
        instrument_model: String::new(),
        freq: vec![1.0],
        phase: vec![0.0],
        z_re: vec![1.0],
        z_im: vec![0.0],
        label: "test".into(),
        metadata: BTreeMap::new(),
        circuit_model: "R0".into(),
    };
    let fit = CircuitFitResult {
        fitted_parameters: vec![42.0],
        parameter_names: vec!["R0".into()],
        parameter_units: vec!["ohm".into()],
        fitted_z_re: vec![1.0],
        fitted_z_im: vec![0.0],
        fitted_magnitude: vec![1.0],
        fitted_phase: vec![0.0],
    };
    rust_electroanalysis_cli::results::EisFitArtifact::from_fit(&input, "R0", &fit, provenance())
}

#[test]
fn eis_mapping_targets_explicit_component_id_only() {
    let mapping = ComponentPriorMapping {
        mapping_id: "eis-contact".into(),
        source: ComponentPriorSource::EisCircuitPath,
        component_id: "contact_rc".into(),
        component_parameter_id: "resistance_ohm".into(),
        source_path: "R0".into(),
    };
    let prior = eis_prior(&eis(), &mapping).expect("explicit EIS element maps");
    assert_eq!(prior.component_id, "contact_rc");
    assert_eq!(prior.value, 42.0);
    let missing = ComponentPriorMapping {
        source_path: "R_missing".into(),
        ..mapping
    };
    assert!(eis_prior(&eis(), &missing).is_none());
}

#[test]
fn matching_timescales_without_replicates_is_weak() {
    let definition = ComponentHypothesisDefinition {
        hypothesis_id: "fast-mode".into(),
        component_ids: vec!["fast_mode".into()],
        description: "neutral mode".into(),
        applicability_domain: "validated aqueous steps".into(),
        minimum_independent_replicates: 3,
    };
    let assessment = assess_component_hypothesis(
        &definition,
        vec!["EIS and transient timescales match".into()],
        vec![],
        vec![],
        1,
    );
    assert_eq!(assessment.evidence_level, EvidenceLevel::Weak);
    assert!(!assessment.missing_evidence.is_empty());
}

#[test]
fn contradictory_evidence_reduces_component_hypothesis_confidence() {
    let definition = ComponentHypothesisDefinition {
        hypothesis_id: "mode".into(),
        component_ids: vec!["mode".into()],
        description: "neutral".into(),
        applicability_domain: "test".into(),
        minimum_independent_replicates: 1,
    };
    let assessment = assess_component_hypothesis(
        &definition,
        vec!["supports".into()],
        vec!["calibration contradicts".into()],
        vec![],
        3,
    );
    assert_eq!(assessment.evidence_level, EvidenceLevel::Contradictory);
    assert_eq!(assessment.contradictory_evidence.len(), 1);
}

#[test]
fn residual_deterioration_is_a_health_warning_feature() {
    let snapshot = ModelHealthSnapshot {
        unexplained_residual_rms_v: Some(0.025),
        component_validity_failures: BTreeMap::from([("slow_mode".into(), 1)]),
        ..Default::default()
    };
    let features = from_model(&snapshot);
    assert!(
        features
            .iter()
            .any(|feature| feature.name == "model.unexplained_residual_rms"
                && feature.value == Some(0.025))
    );
    assert!(features.iter().any(|feature| feature.warning.is_some()));
}

#[test]
fn mechanistic_health_rule_needs_multiple_domains_and_keeps_contradiction() {
    let rule = HealthRule {
        rule_id: "fouling".into(),
        finding: HealthFindingKind::ProbableFouling,
        severity: HealthSeverity::Major,
        all_of: vec![FeatureCondition {
            feature: "slow".into(),
            operator: FeatureOperator::GreaterThan,
            value: Some(1.0),
        }],
        any_of: vec![],
        minimum_evidence_domains: 2,
        minimum_baseline_records: 3,
        alternative_explanations: vec![],
    };
    let features = vec![HealthFeature {
        name: "slow".into(),
        value: Some(0.5),
        unit: "s".into(),
        domain: HealthDomain::DynamicResponse,
        source: "test".into(),
        warning: None,
    }];
    let (evaluations, findings) = rules::evaluate(&[rule], &features, &[], 2);
    assert!(!evaluations[0].triggered);
    assert_eq!(evaluations[0].contradictory_evidence.len(), 1);
    assert!(findings.is_empty());
}
