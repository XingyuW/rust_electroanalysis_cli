//! Permanent V1 equilibrium-recognition and identifiability regression matrix.

use rust_electroanalysis_cli::model::{
    AssessmentStatus, EquilibriumEvidence, EquilibriumRecognitionConfig, EquilibriumStatus,
    EvidenceValue, IdentifiabilityRequirementKind, IdentifiabilityScope, InterpretationStatus,
    UncertaintyStatus, ValidityReport, built_in_registry, compile_model, recognize_equilibrium,
    reduced_ism_v1_definition,
};

fn evidence() -> EquilibriumEvidence {
    EquilibriumEvidence {
        dynamic_state_derivative_norm: Some(1e-7),
        dynamic_potential_magnitude_v: Some(1e-6),
        equilibrium_gap_v: Some(1e-6),
        elapsed_tau_ratios: vec![4.0, 5.0],
        environmental_stability: Some(1.0),
        innovation_metric: Some(0.1),
        residual_autocorrelation: Some(0.01),
        observable: Some(true),
        validity: ValidityReport {
            is_valid: true,
            checked_domain: "test".into(),
            violations: vec![],
            warnings: vec![],
        },
        uncertainty_status: UncertaintyStatus::Complete,
        external_disturbance_potential_v: EvidenceValue::Present(0.0),
    }
}

#[test]
fn equilibrium_recognition_classifies_every_v1_status_with_auditable_criteria() {
    let config = EquilibriumRecognitionConfig::default();
    let equilibrium = recognize_equilibrium(&evidence(), &config);
    assert_eq!(equilibrium.classification, EquilibriumStatus::Equilibrium);
    assert_eq!(equilibrium.status, AssessmentStatus::Supported);
    assert!(
        equilibrium
            .satisfied_criteria
            .iter()
            .any(|item| item == "equilibrium gap below threshold")
    );
    assert!(equilibrium.violated_criteria.is_empty());

    let mut quasi_evidence = evidence();
    quasi_evidence.environmental_stability = Some(0.9);
    let quasi = recognize_equilibrium(&quasi_evidence, &config);
    assert_eq!(quasi.classification, EquilibriumStatus::QuasiEquilibrium);
    assert_eq!(quasi.status, AssessmentStatus::Supported);
    assert!(
        quasi_evidence.environmental_stability.unwrap() >= config.environmental_stability_threshold
            && quasi_evidence.environmental_stability.unwrap()
                < config.marginal_environmental_stability,
        "environmental stability is the exact marginal criterion preventing full equilibrium"
    );
    assert_eq!(
        quasi.warnings,
        vec!["dynamics are small but one or more equilibrium criteria are marginal"]
    );
    assert!(
        quasi.violated_criteria.is_empty(),
        "marginal stability, rather than a failure, is the quasi criterion"
    );

    let mut transitional_evidence = evidence();
    transitional_evidence.dynamic_potential_magnitude_v = Some(1e-3);
    let transitional = recognize_equilibrium(&transitional_evidence, &config);
    assert_eq!(transitional.classification, EquilibriumStatus::Transitional);
    assert_eq!(transitional.status, AssessmentStatus::Contradicted);
    assert!(
        transitional
            .violated_criteria
            .iter()
            .any(|item| item.contains("dynamic mode"))
    );

    let mut disturbed_evidence = evidence();
    disturbed_evidence.external_disturbance_potential_v = EvidenceValue::Present(1e-3);
    let disturbed = recognize_equilibrium(&disturbed_evidence, &config);
    assert_eq!(disturbed.classification, EquilibriumStatus::Disturbed);
    assert_eq!(disturbed.status, AssessmentStatus::Contradicted);
    assert!(
        disturbed
            .violated_criteria
            .iter()
            .any(|item| item.contains("external disturbance"))
    );

    let mut indeterminate_evidence = evidence();
    indeterminate_evidence.uncertainty_status = UncertaintyStatus::Unavailable;
    let indeterminate = recognize_equilibrium(&indeterminate_evidence, &config);
    assert_eq!(
        indeterminate.classification,
        EquilibriumStatus::Indeterminate
    );
    assert_eq!(indeterminate.status, AssessmentStatus::Indeterminate);
    assert!(
        indeterminate
            .missing_evidence
            .iter()
            .any(|item| item.contains("uncertainty"))
    );
    assert!(
        indeterminate
            .warnings
            .iter()
            .any(|item| item.contains("indeterminate"))
    );
    assert!(indeterminate.confidence >= 0.0 && indeterminate.confidence <= 1.0);
}

#[test]
fn equilibrium_recognition_preserves_missing_not_applicable_and_unobservable_evidence() {
    let config = EquilibriumRecognitionConfig::default();
    let mut missing = evidence();
    missing.external_disturbance_potential_v = EvidenceValue::Missing {
        reason: "no disturbance channel".into(),
    };
    let assessment = recognize_equilibrium(&missing, &config);
    assert_eq!(assessment.classification, EquilibriumStatus::Indeterminate);
    assert_eq!(assessment.status, AssessmentStatus::Indeterminate);
    assert!(
        assessment
            .missing_evidence
            .iter()
            .any(|item| item.contains("no disturbance channel"))
    );

    let mut not_applicable = evidence();
    not_applicable.external_disturbance_potential_v = EvidenceValue::NotApplicable;
    assert_eq!(
        recognize_equilibrium(&not_applicable, &config).classification,
        EquilibriumStatus::Equilibrium
    );

    let mut unobservable = evidence();
    unobservable.observable = Some(false);
    let assessment = recognize_equilibrium(&unobservable, &config);
    assert_eq!(assessment.classification, EquilibriumStatus::Indeterminate);
    assert_eq!(assessment.status, AssessmentStatus::Indeterminate);
    assert!(
        assessment
            .missing_evidence
            .iter()
            .any(|item| item == "model observability")
    );

    let mut flat_but_slow = evidence();
    flat_but_slow.dynamic_state_derivative_norm = Some(0.0);
    flat_but_slow.dynamic_potential_magnitude_v = Some(1e-3);
    assert_eq!(
        recognize_equilibrium(&flat_but_slow, &config).classification,
        EquilibriumStatus::Transitional
    );
}

fn active_kinds(
    definition: rust_electroanalysis_cli::model::ModelDefinition,
) -> Vec<(String, IdentifiabilityRequirementKind)> {
    compile_model(definition, built_in_registry())
        .unwrap()
        .identifiability_metadata()
        .component_requirements
        .into_iter()
        .filter(|item| item.scope == IdentifiabilityScope::Active)
        .map(|item| (item.requirement_id, item.kind))
        .collect()
}

#[test]
fn identifiability_metadata_distinguishes_one_and_two_active_dynamic_modes() {
    let mut one_mode = reduced_ism_v1_definition();
    one_mode
        .components
        .retain(|component| component.id != "dynamic_slow");
    let one = active_kinds(one_mode);
    assert!(
        !one.iter()
            .any(|(_, kind)| *kind == IdentifiabilityRequirementKind::ModeSeparation)
    );
    assert!(
        one.iter()
            .any(|(_, kind)| *kind == IdentifiabilityRequirementKind::TransientExcitation)
    );
    assert!(
        one.iter().any(|(_, kind)| *kind
            == IdentifiabilityRequirementKind::ObservationDurationRelativeToTimescale)
    );

    let two = active_kinds(reduced_ism_v1_definition());
    for kind in [
        IdentifiabilityRequirementKind::ModeSeparation,
        IdentifiabilityRequirementKind::TransientExcitation,
        IdentifiabilityRequirementKind::ObservationDurationRelativeToTimescale,
        IdentifiabilityRequirementKind::ReferenceAnchor,
    ] {
        assert!(
            two.iter().any(|(_, found)| *found == kind),
            "missing {kind:?}"
        );
    }

    let metadata = compile_model(reduced_ism_v1_definition(), built_in_registry())
        .unwrap()
        .identifiability_metadata();
    let mode_separation = metadata
        .component_requirements
        .iter()
        .find(|item| {
            item.requirement_id == "dynamic-group:dynamic_fast+dynamic_slow:modeseparation"
        })
        .unwrap();
    assert_eq!(mode_separation.scope, IdentifiabilityScope::Active);
    assert_eq!(
        mode_separation.component_ids,
        vec!["dynamic_fast", "dynamic_slow"]
    );
    assert_eq!(
        mode_separation.target_states,
        vec!["dynamic_fast_potential_v", "dynamic_slow_potential_v"]
    );
    assert_eq!(
        mode_separation.target_parameters,
        vec!["dynamic_fast_tau_s", "dynamic_slow_tau_s"]
    );
    assert_eq!(
        mode_separation.kind,
        IdentifiabilityRequirementKind::ModeSeparation
    );
    assert_eq!(
        mode_separation.quantitative_criterion.as_deref(),
        Some("distinct time constants relative to sampling and duration")
    );
}

#[test]
fn optional_requirements_promote_to_active_and_serialized_order_is_stable() {
    let baseline = compile_model(reduced_ism_v1_definition(), built_in_registry())
        .unwrap()
        .identifiability_metadata();
    assert!(baseline.component_requirements.iter().any(|item| matches!(
        item.scope,
        IdentifiabilityScope::Conditional { .. }
    ) && item.kind
        == IdentifiabilityRequirementKind::InterferentVariation));

    let mut enabled = reduced_ism_v1_definition();
    enabled.components[1].kind = "equilibrium.nicolsky_eisenman".into();
    enabled.components[2].kind = "transduction.first_order_candidate".into();
    enabled.components[2].interpretation_status = InterpretationStatus::Hypothesized;
    enabled.inputs[4].id = "relative_humidity".into();
    enabled.inputs[4].unit = "%RH".into();
    enabled.parameters[2].unit = "V/%RH".into();
    enabled.parameters[3].unit = "%RH".into();
    let covariate = &mut enabled.components[4];
    covariate.id = "humidity_covariate".into();
    covariate.kind = "disturbance.linear_covariate".into();
    covariate.role = rust_electroanalysis_cli::model::ComponentRole::ExternalDisturbance;
    covariate.required_inputs = vec![rust_electroanalysis_cli::model::InputRequirement {
        id: "relative_humidity".into(),
        unit: "%RH".into(),
    }];
    covariate.state_ids.clear();
    covariate.observation_state_ids.clear();
    covariate.parameter_ids = vec![
        "dynamic_fast_tau_s".into(),
        "dynamic_fast_gain_v_per_decade".into(),
    ];
    covariate.observation_parameter_ids = covariate.parameter_ids.clone();
    let metadata = compile_model(enabled.clone(), built_in_registry())
        .unwrap()
        .identifiability_metadata();
    assert!(
        metadata
            .component_requirements
            .iter()
            .any(|item| item.scope == IdentifiabilityScope::Active
                && item.kind == IdentifiabilityRequirementKind::InterferentVariation)
    );
    assert!(
        metadata
            .component_requirements
            .iter()
            .any(|item| item.scope == IdentifiabilityScope::Active
                && item.kind == IdentifiabilityRequirementKind::AuxiliaryObservation)
    );
    assert!(
        metadata
            .component_requirements
            .iter()
            .any(|item| item.scope == IdentifiabilityScope::Active
                && item.kind == IdentifiabilityRequirementKind::IndependentCovariateVariation)
    );
    for requirement in &metadata.component_requirements {
        assert!(!requirement.requirement_id.is_empty());
        assert!(!requirement.component_id.is_empty());
        assert!(
            !requirement.component_ids.is_empty()
                || matches!(requirement.scope, IdentifiabilityScope::Conditional { .. })
        );
    }
    assert!(metadata.component_requirements.iter().any(|item| {
        item.requirement_id == "equilibrium_nernst.activityexcitation"
            && item.scope == IdentifiabilityScope::Active
            && item.component_id == "equilibrium_nernst"
            && item.kind == IdentifiabilityRequirementKind::ActivityExcitation
            && item.target_parameters == vec!["standard_potential_v", "ion_charge"]
    }));
    let first = serde_json::to_string(&metadata.component_requirements).unwrap();
    enabled.components.reverse();
    let second = serde_json::to_string(
        &compile_model(enabled, built_in_registry())
            .unwrap()
            .identifiability_metadata()
            .component_requirements,
    )
    .unwrap();
    assert_eq!(
        first, second,
        "requirement ordering and IDs must not inherit source component order"
    );
}
