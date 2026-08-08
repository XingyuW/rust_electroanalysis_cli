//! Permanent V1 regression matrix for applicability migration and enforcement.

use rust_electroanalysis_cli::model::{
    ApplicabilityConstraint, ApplicabilityConstraintProvenance, ComponentApplicabilityDomain,
    DomainEnforcement, DomainSource, DomainStatus, DomainSubject, InputRequirement, InputValue,
    ModelError, ModelInput, ModelWarning, NumericInterval, built_in_registry, compile_model,
    reduced_ism_v1_definition,
};
use std::collections::BTreeMap;

fn input(activity: f64, temperature: f64) -> ModelInput {
    ModelInput {
        time_s: 0.0,
        values: BTreeMap::from([
            (
                "target_activity".into(),
                InputValue {
                    value: activity,
                    unit: "activity".into(),
                },
            ),
            (
                "temperature".into(),
                InputValue {
                    value: temperature,
                    unit: "K".into(),
                },
            ),
        ]),
    }
}

fn typed(
    id: &str,
    subject: &str,
    lower: f64,
    upper: f64,
    enforcement: DomainEnforcement,
) -> ApplicabilityConstraint {
    ApplicabilityConstraint {
        id: id.into(),
        subject: DomainSubject::Input(subject.into()),
        interval: NumericInterval { lower, upper },
        source: DomainSource::CalibrationArtifact,
        enforcement,
        provenance: vec![],
    }
}

fn legacy_activity(
    lower: f64,
    upper: f64,
    enforcement: DomainEnforcement,
) -> ComponentApplicabilityDomain {
    ComponentApplicabilityDomain {
        target_activity: Some(NumericInterval { lower, upper }),
        temperature_k: None,
        interferent_activities: BTreeMap::new(),
        environmental_inputs: BTreeMap::new(),
        source: DomainSource::CalibrationArtifact,
        enforcement,
    }
}

fn compiled_constraints(
    definition: rust_electroanalysis_cli::model::ModelDefinition,
) -> Vec<ApplicabilityConstraint> {
    compile_model(definition, built_in_registry())
        .unwrap()
        .definition()
        .components[1]
        .applicability_constraints
        .clone()
}

#[test]
fn legacy_only_constraint_is_migrated_losslessly() {
    let mut definition = reduced_ism_v1_definition();
    definition.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&legacy_activity(1e-6, 1.0, DomainEnforcement::Reject)).unwrap(),
    );
    let constraints = compiled_constraints(definition);
    assert_eq!(
        constraints,
        vec![ApplicabilityConstraint {
            id: "target_activity".into(),
            subject: DomainSubject::Input("target_activity".into()),
            interval: NumericInterval {
                lower: 1e-6,
                upper: 1.0
            },
            source: DomainSource::CalibrationArtifact,
            enforcement: DomainEnforcement::Reject,
            provenance: vec![ApplicabilityConstraintProvenance::LegacyMetadata],
        }]
    );
}

#[test]
fn typed_only_constraint_is_resolved_and_preserved() {
    let mut definition = reduced_ism_v1_definition();
    definition.components[1].applicability_constraints = vec![typed(
        "temperature",
        "temperature",
        290.0,
        310.0,
        DomainEnforcement::Warn,
    )];
    let constraints = compiled_constraints(definition);
    assert_eq!(
        constraints[0].subject,
        DomainSubject::Input("temperature".into())
    );
    assert_eq!(
        constraints[0].interval,
        NumericInterval {
            lower: 290.0,
            upper: 310.0
        }
    );
    assert_eq!(constraints[0].enforcement, DomainEnforcement::Warn);
    assert_eq!(
        constraints[0].provenance,
        vec![ApplicabilityConstraintProvenance::TypedDeclaration]
    );
}

#[test]
fn mixed_legacy_and_typed_constraints_survive_with_stable_ordering() {
    let mut first = reduced_ism_v1_definition();
    first.components[1].applicability_constraints = vec![typed(
        "temperature",
        "temperature",
        290.0,
        310.0,
        DomainEnforcement::Reject,
    )];
    first.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&legacy_activity(1e-6, 1.0, DomainEnforcement::Warn)).unwrap(),
    );
    let mut reordered = first.clone();
    reordered.components[1].applicability_constraints.reverse();
    let left = compiled_constraints(first);
    let right = compiled_constraints(reordered);
    assert_eq!(left, right);
    assert_eq!(left.len(), 2);
    assert_eq!(
        left[0].subject,
        DomainSubject::Input("target_activity".into())
    );
    assert_eq!(
        left[0].interval,
        NumericInterval {
            lower: 1e-6,
            upper: 1.0
        }
    );
    assert_eq!(left[0].enforcement, DomainEnforcement::Warn);
    assert_eq!(
        left[0].provenance,
        vec![ApplicabilityConstraintProvenance::LegacyMetadata]
    );
    assert_eq!(left[1].subject, DomainSubject::Input("temperature".into()));
    assert_eq!(
        left[1].interval,
        NumericInterval {
            lower: 290.0,
            upper: 310.0
        }
    );
    assert_eq!(left[1].enforcement, DomainEnforcement::Reject);
    assert_eq!(
        left[1].provenance,
        vec![ApplicabilityConstraintProvenance::TypedDeclaration]
    );
}

#[test]
fn exact_duplicate_retains_both_provenance_sources_and_conflicts_name_fields() {
    let mut duplicate = reduced_ism_v1_definition();
    duplicate.components[1].applicability_constraints = vec![typed(
        "typed_activity",
        "target_activity",
        1e-6,
        1.0,
        DomainEnforcement::Warn,
    )];
    duplicate.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&legacy_activity(1e-6, 1.0, DomainEnforcement::Warn)).unwrap(),
    );
    let compiled = compile_model(duplicate, built_in_registry()).unwrap();
    assert_eq!(
        compiled.definition().components[1]
            .applicability_constraints
            .len(),
        1
    );
    assert_eq!(
        compiled.definition().components[1].applicability_constraints[0].provenance,
        vec![
            ApplicabilityConstraintProvenance::TypedDeclaration,
            ApplicabilityConstraintProvenance::LegacyMetadata
        ]
    );
}

#[test]
fn conflicting_intervals_and_policies_return_typed_conflict_context() {
    let mut interval_conflict = reduced_ism_v1_definition();
    interval_conflict.components[1].applicability_constraints = vec![typed(
        "typed_activity",
        "target_activity",
        1e-4,
        1.0,
        DomainEnforcement::Warn,
    )];
    interval_conflict.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&legacy_activity(1e-6, 1.0, DomainEnforcement::Warn)).unwrap(),
    );
    assert!(
        matches!(compile_model(interval_conflict, built_in_registry()), Err(ModelError::ConflictingApplicabilityConstraints { details })
        if details.component_id == "equilibrium_nernst" && details.subject == DomainSubject::Input("target_activity".into())
        && details.first_constraint_id == "target_activity" && details.second_constraint_id == "typed_activity"
        && details.reason == "same subject has no declared intersection/composition rule")
    );

    let mut policy_conflict = reduced_ism_v1_definition();
    policy_conflict.components[1].applicability_constraints = vec![typed(
        "typed_activity",
        "target_activity",
        1e-6,
        1.0,
        DomainEnforcement::Reject,
    )];
    policy_conflict.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&legacy_activity(1e-6, 1.0, DomainEnforcement::Warn)).unwrap(),
    );
    assert!(
        matches!(compile_model(policy_conflict, built_in_registry()), Err(ModelError::ConflictingApplicabilityConstraints { details })
        if details.component_id == "equilibrium_nernst" && details.subject == DomainSubject::Input("target_activity".into())
        && details.first_constraint_id == "target_activity" && details.second_constraint_id == "typed_activity"
        && details.reason == "same subject has no declared intersection/composition rule")
    );
}

fn evaluate(
    constraints: Vec<ApplicabilityConstraint>,
    model_input: &ModelInput,
) -> Result<Vec<rust_electroanalysis_cli::model::ComponentContribution>, ModelError> {
    let mut definition = reduced_ism_v1_definition();
    definition.components[1].applicability_constraints = constraints;
    let model = compile_model(definition, built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    model.component_contributions(&state, &parameters, model_input)
}

#[test]
fn independent_warn_and_reject_policies_keep_each_constraint_outcome() {
    let warnings = vec![
        typed(
            "activity_warn",
            "target_activity",
            1e-6,
            1.0,
            DomainEnforcement::Warn,
        ),
        typed(
            "temperature_warn",
            "temperature",
            290.0,
            310.0,
            DomainEnforcement::Warn,
        ),
    ];
    let values = evaluate(warnings, &input(2.0, 320.0)).unwrap();
    let nernst = values
        .iter()
        .find(|item| item.component_id == "equilibrium_nernst")
        .unwrap();
    assert_eq!(nernst.warnings.len(), 2);
    assert!(nernst.warnings.iter().all(|warning| matches!(
        warning,
        ModelWarning::Validity(message) if message.contains("outside declared applicability domain")
    )));

    let mixed = vec![
        typed(
            "activity_warn",
            "target_activity",
            1e-6,
            1.0,
            DomainEnforcement::Warn,
        ),
        typed(
            "temperature_reject",
            "temperature",
            290.0,
            310.0,
            DomainEnforcement::Reject,
        ),
    ];
    let values = evaluate(mixed.clone(), &input(2.0, 298.15)).unwrap();
    let nernst = values
        .iter()
        .find(|item| item.component_id == "equilibrium_nernst")
        .unwrap();
    assert_eq!(nernst.warnings.len(), 1);
    assert!(
        matches!(&nernst.warnings[0], ModelWarning::Validity(message) if message.contains("activity_warn"))
    );
    let report = {
        let mut definition = reduced_ism_v1_definition();
        definition.components[1].applicability_constraints = mixed.clone();
        let model = compile_model(definition, built_in_registry()).unwrap();
        let parameters = model.default_parameters();
        let state = model.initialize(&parameters).unwrap();
        model
            .component_validity_reports(&state, &parameters, &input(2.0, 298.15))
            .into_iter()
            .find(|item| item.component_id == "equilibrium_nernst")
            .unwrap()
    };
    assert_eq!(report.domain_status, DomainStatus::OutsideDomain);
    assert_eq!(
        report
            .constraint_statuses
            .iter()
            .map(|item| (item.constraint_id.as_str(), item.status))
            .collect::<Vec<_>>(),
        vec![
            ("activity_warn", DomainStatus::OutsideDomain),
            ("temperature_reject", DomainStatus::InsideDomain)
        ]
    );
    assert!(
        matches!(evaluate(mixed, &input(0.1, 320.0)), Err(ModelError::ApplicabilityConstraintRejected { constraint_id, enforcement: DomainEnforcement::Reject, .. }) if constraint_id == "temperature_reject")
    );
}

#[test]
fn unavailable_warn_and_reject_are_independent_and_ordered() {
    let mut definition = reduced_ism_v1_definition();
    definition.inputs[4].required = false;
    definition.components[4]
        .required_inputs
        .push(InputRequirement {
            id: "conductivity".into(),
            unit: "S/m".into(),
        });
    definition.components[4].applicability_constraints = vec![
        typed("z_warn", "conductivity", 0.1, 1.0, DomainEnforcement::Warn),
        typed("a_warn", "conductivity", 0.2, 2.0, DomainEnforcement::Warn),
    ];
    // Two different contracts for the same subject are deliberately rejected at compilation.
    assert!(matches!(
        compile_model(definition.clone(), built_in_registry()),
        Err(ModelError::ConflictingApplicabilityConstraints { .. })
    ));
    definition.components[4].applicability_constraints = vec![typed(
        "conductivity_warn",
        "conductivity",
        0.1,
        1.0,
        DomainEnforcement::Warn,
    )];
    let model = compile_model(definition.clone(), built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let reports = model.component_validity_reports(&state, &parameters, &input(0.1, 298.15));
    let report = reports
        .iter()
        .find(|report| report.component_id == "reference_offset")
        .unwrap();
    assert_eq!(report.domain_status, DomainStatus::DomainUnavailable);
    assert_eq!(
        report
            .constraint_statuses
            .iter()
            .map(|status| status.constraint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["conductivity_warn"]
    );
    assert_eq!(
        model
            .component_contributions(&state, &parameters, &input(0.1, 298.15))
            .unwrap()
            .len(),
        6
    );

    definition.components[4].applicability_constraints[0].enforcement = DomainEnforcement::Reject;
    let rejecting = compile_model(definition, built_in_registry()).unwrap();
    let state = rejecting.initialize(&parameters).unwrap();
    assert!(
        matches!(rejecting.component_contributions(&state, &parameters, &input(0.1, 298.15)), Err(ModelError::ApplicabilityConstraintRejected { constraint_id, subject: DomainSubject::Input(subject), status: DomainStatus::DomainUnavailable, enforcement: DomainEnforcement::Reject, .. }) if constraint_id == "conductivity_warn" && subject == "conductivity")
    );
}
