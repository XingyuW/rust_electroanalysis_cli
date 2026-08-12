//! Permanent scientific guardrail regression coverage for Reduced-Order ISM V1.

use rust_electroanalysis_cli::model::{
    ApplicabilityConstraint, ApplicabilityConstraintProvenance, ComponentApplicabilityDomain,
    DomainEnforcement, DomainSource, DomainStatus, DomainSubject, EquilibriumEvidence,
    EquilibriumRecognitionConfig, EquilibriumStatus, EvidenceValue, InputValue,
    InterpretationStatus, ModelError, ModelInput, NumericInterval, UncertaintyStatus,
    ValidityReport, built_in_registry, compile_model, exact_nonzero_charge, recognize_equilibrium,
    reduced_ism_v1_definition,
};
use rust_electroanalysis_cli::{
    domain::write_artifact,
    results::{ModelAnalysisPoint, ModelAnalysisReport, ModelCompilationArtifact},
};
use std::collections::BTreeMap;

fn input(activity: f64, temperature_k: f64) -> ModelInput {
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
                    value: temperature_k,
                    unit: "K".into(),
                },
            ),
        ]),
    }
}

fn constraint(
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

#[test]
fn exact_discrete_charges_are_never_coerced() {
    for value in [-3.0, -2.0, -1.0, 1.0, 2.0, 3.0] {
        assert_eq!(
            exact_nonzero_charge("ion_charge", value).unwrap(),
            value as i32
        );
    }
    for value in [
        0.0,
        1.5,
        -1.5,
        f64::NAN,
        f64::INFINITY,
        i32::MAX as f64 + 1.0,
    ] {
        assert!(matches!(
            exact_nonzero_charge("ion_charge", value),
            Err(ModelError::InvalidDiscreteParameter { .. })
        ));
    }

    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    for value in [1.5, -1.5, 0.0, f64::NAN, f64::INFINITY] {
        let mut parameters = model.default_parameters();
        parameters.values[1] = value;
        assert!(matches!(
            model.validate_parameters(&parameters),
            Err(ModelError::InvalidDiscreteParameter { .. })
        ));
    }
}

#[test]
fn declared_domains_are_reported_and_can_reject() {
    let mut definition = reduced_ism_v1_definition();
    let domain = ComponentApplicabilityDomain {
        target_activity: Some(NumericInterval {
            lower: 1e-6,
            upper: 1.0,
        }),
        temperature_k: Some(NumericInterval {
            lower: 290.0,
            upper: 310.0,
        }),
        interferent_activities: BTreeMap::new(),
        environmental_inputs: BTreeMap::new(),
        source: DomainSource::CalibrationArtifact,
        enforcement: DomainEnforcement::Warn,
    };
    definition.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&domain).unwrap(),
    );
    let model = compile_model(definition.clone(), built_in_registry()).unwrap();
    let state = model.initialize(&model.default_parameters()).unwrap();
    let reports =
        model.component_validity_reports(&state, &model.default_parameters(), &input(1e-6, 298.15));
    assert_eq!(
        reports
            .iter()
            .find(|report| report.component_id == "equilibrium_nernst")
            .unwrap()
            .domain_status,
        DomainStatus::NearBoundary
    );
    let prediction = model
        .component_contributions(&state, &model.default_parameters(), &input(1e300, 1.0))
        .unwrap();
    assert!(prediction.iter().any(|item| {
        item.warnings
            .iter()
            .any(|warning| format!("{warning:?}").contains("outside declared applicability domain"))
    }));

    definition.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&ComponentApplicabilityDomain {
            enforcement: DomainEnforcement::Reject,
            ..domain
        })
        .unwrap(),
    );
    let rejecting = compile_model(definition, built_in_registry()).unwrap();
    let state = rejecting
        .initialize(&rejecting.default_parameters())
        .unwrap();
    assert!(
        rejecting
            .component_contributions(
                &state,
                &rejecting.default_parameters(),
                &input(1e300, 298.15)
            )
            .is_err()
    );
}

#[test]
fn candidate_transduction_cannot_claim_stronger_interpretation() {
    let mut definition = reduced_ism_v1_definition();
    definition.components[2].kind = "transduction.first_order_candidate".into();
    definition.components[2].interpretation_status = InterpretationStatus::Hypothesized;
    assert!(compile_model(definition.clone(), built_in_registry()).is_ok());
    definition.components[2].interpretation_status = InterpretationStatus::ValidatedForDomain;
    assert!(matches!(
        compile_model(definition, built_in_registry()),
        Err(ModelError::InvalidInterpretationStatus { .. })
    ));
}

fn evidence(disturbance: EvidenceValue<f64>) -> EquilibriumEvidence {
    EquilibriumEvidence {
        dynamic_state_derivative_norm: Some(1e-7),
        dynamic_potential_magnitude_v: Some(1e-6),
        equilibrium_gap_v: Some(1e-6),
        elapsed_tau_ratios: vec![5.0],
        environmental_stability: Some(1.0),
        innovation_metric: Some(0.0),
        residual_autocorrelation: Some(0.0),
        observable: Some(true),
        validity: ValidityReport {
            is_valid: true,
            checked_domain: "test".into(),
            violations: vec![],
            warnings: vec![],
        },
        uncertainty_status: UncertaintyStatus::Complete,
        external_disturbance_potential_v: disturbance,
    }
}

#[test]
fn missing_disturbance_evidence_is_not_zero() {
    let config = EquilibriumRecognitionConfig::default();
    assert_eq!(
        recognize_equilibrium(
            &evidence(EvidenceValue::Missing {
                reason: "sensor unavailable".into()
            }),
            &config
        )
        .classification,
        EquilibriumStatus::Indeterminate
    );
    assert_eq!(
        recognize_equilibrium(&evidence(EvidenceValue::NotApplicable), &config).classification,
        EquilibriumStatus::Equilibrium
    );
    assert_eq!(
        recognize_equilibrium(&evidence(EvidenceValue::Present(1.0)), &config).classification,
        EquilibriumStatus::Disturbed
    );
}

#[test]
fn component_identifiability_requirements_are_structured_and_deterministic() {
    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    let metadata = model.identifiability_metadata();
    assert_eq!(metadata, model.identifiability_metadata());
    let text = format!("{:?}", metadata.component_requirements);
    assert!(text.contains("TransientExcitation"));
    assert!(text.contains("ObservationDurationRelativeToTimescale"));
    assert!(text.contains("ReferenceAnchor"));
    assert!(metadata.component_requirements.iter().any(|requirement| {
        requirement.kind
            == rust_electroanalysis_cli::model::IdentifiabilityRequirementKind::ModeSeparation
            && requirement.component_ids == ["dynamic_fast", "dynamic_slow"]
            && !requirement.requirement_id.is_empty()
    }));
    assert!(metadata.component_requirements.iter().any(|requirement| matches!(
        requirement.scope,
        rust_electroanalysis_cli::model::IdentifiabilityScope::Conditional { .. }
    ) && matches!(requirement.kind,
        rust_electroanalysis_cli::model::IdentifiabilityRequirementKind::InterferentVariation
            | rust_electroanalysis_cli::model::IdentifiabilityRequirementKind::IndependentCovariateVariation
            | rust_electroanalysis_cli::model::IdentifiabilityRequirementKind::AuxiliaryObservation
    )));
}

#[test]
fn applicability_constraints_are_explicitly_bound_and_never_skipped() {
    let mut definition = reduced_ism_v1_definition();
    definition.components[1].applicability_constraints = vec![
        ApplicabilityConstraint {
            id: "activity".into(),
            subject: DomainSubject::Input("target_activity".into()),
            interval: NumericInterval {
                lower: 1e-6,
                upper: 1.0,
            },
            source: DomainSource::CalibrationArtifact,
            enforcement: DomainEnforcement::Warn,
            provenance: vec![],
        },
        ApplicabilityConstraint {
            id: "temperature".into(),
            subject: DomainSubject::Input("temperature".into()),
            interval: NumericInterval {
                lower: 290.0,
                upper: 310.0,
            },
            source: DomainSource::CalibrationArtifact,
            enforcement: DomainEnforcement::Warn,
            provenance: vec![],
        },
    ];
    let model = compile_model(definition, built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let report = model
        .component_validity_reports(&state, &parameters, &input(0.1, 298.15))
        .into_iter()
        .find(|item| item.component_id == "equilibrium_nernst")
        .unwrap();
    assert_eq!(report.domain_status, DomainStatus::InsideDomain);
    assert_eq!(
        report.constraint_statuses.len(),
        2,
        "every constraint is evaluated exactly once"
    );

    let mut unavailable = ModelInput::empty(0.0);
    unavailable.values.insert(
        "target_activity".into(),
        InputValue {
            value: 0.1,
            unit: "activity".into(),
        },
    );
    // Temperature remains model-required, so use an optional constrained input
    // to demonstrate domain availability without weakening runtime input rules.
    let mut optional = reduced_ism_v1_definition();
    optional.inputs[4].required = false;
    optional.components[4].required_inputs.push(
        rust_electroanalysis_cli::model::InputRequirement {
            id: "conductivity".into(),
            unit: "S/m".into(),
        },
    );
    optional.components[4].applicability_constraints = vec![ApplicabilityConstraint {
        id: "conductivity".into(),
        subject: DomainSubject::Input("conductivity".into()),
        interval: NumericInterval {
            lower: 0.1,
            upper: 1.0,
        },
        source: DomainSource::UserConfiguration,
        enforcement: DomainEnforcement::Warn,
        provenance: vec![],
    }];
    let optional_model = compile_model(optional, built_in_registry()).unwrap();
    let optional_state = optional_model
        .initialize(&optional_model.default_parameters())
        .unwrap();
    let optional_report = optional_model
        .component_validity_reports(
            &optional_state,
            &optional_model.default_parameters(),
            &unavailable,
        )
        .into_iter()
        .find(|item| item.component_id == "reference_offset")
        .unwrap();
    assert_eq!(
        optional_report.domain_status,
        DomainStatus::DomainUnavailable
    );
}

#[test]
fn legacy_and_typed_applicability_constraints_are_merged_losslessly() {
    let mut definition = reduced_ism_v1_definition();
    definition.components[1].applicability_constraints = vec![constraint(
        "temperature",
        "temperature",
        290.0,
        310.0,
        DomainEnforcement::Warn,
    )];
    definition.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&ComponentApplicabilityDomain {
            target_activity: Some(NumericInterval {
                lower: 1e-6,
                upper: 1.0,
            }),
            temperature_k: None,
            interferent_activities: BTreeMap::new(),
            environmental_inputs: BTreeMap::new(),
            source: DomainSource::CalibrationArtifact,
            enforcement: DomainEnforcement::Warn,
        })
        .unwrap(),
    );

    let model = compile_model(definition, built_in_registry()).unwrap();
    let constraints = &model.definition().components[1].applicability_constraints;
    assert_eq!(constraints.len(), 2, "legacy constraint must not disappear");
    assert_eq!(
        constraints[0].subject,
        DomainSubject::Input("target_activity".into())
    );
    assert_eq!(
        constraints[1].subject,
        DomainSubject::Input("temperature".into())
    );
    assert_eq!(
        constraints[0].provenance,
        vec![ApplicabilityConstraintProvenance::LegacyMetadata]
    );
    assert_eq!(
        constraints[1].provenance,
        vec![ApplicabilityConstraintProvenance::TypedDeclaration]
    );
}

#[test]
fn exact_legacy_typed_duplicates_deduplicate_and_conflicts_are_typed() {
    let legacy = ComponentApplicabilityDomain {
        target_activity: Some(NumericInterval {
            lower: 1e-6,
            upper: 1.0,
        }),
        temperature_k: None,
        interferent_activities: BTreeMap::new(),
        environmental_inputs: BTreeMap::new(),
        source: DomainSource::CalibrationArtifact,
        enforcement: DomainEnforcement::Warn,
    };
    let mut duplicate = reduced_ism_v1_definition();
    duplicate.components[1].applicability_constraints = vec![constraint(
        "target_activity",
        "target_activity",
        1e-6,
        1.0,
        DomainEnforcement::Warn,
    )];
    duplicate.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&legacy).unwrap(),
    );
    let model = compile_model(duplicate, built_in_registry()).unwrap();
    let constraints = &model.definition().components[1].applicability_constraints;
    assert_eq!(constraints.len(), 1);
    assert_eq!(
        constraints[0].provenance,
        vec![
            ApplicabilityConstraintProvenance::TypedDeclaration,
            ApplicabilityConstraintProvenance::LegacyMetadata,
        ]
    );

    let mut conflicting = reduced_ism_v1_definition();
    conflicting.components[1].applicability_constraints = vec![constraint(
        "typed_activity",
        "target_activity",
        1e-4,
        1.0,
        DomainEnforcement::Warn,
    )];
    conflicting.components[1].metadata.insert(
        "applicability_domain".into(),
        serde_json::to_string(&legacy).unwrap(),
    );
    assert!(matches!(
        compile_model(conflicting, built_in_registry()),
        Err(ModelError::ConflictingApplicabilityConstraints { .. })
    ));
}

#[test]
fn applicability_enforcement_is_independent_per_constraint() {
    let mut definition = reduced_ism_v1_definition();
    definition.components[1].applicability_constraints = vec![
        constraint(
            "activity_warn",
            "target_activity",
            1e-6,
            1.0,
            DomainEnforcement::Warn,
        ),
        constraint(
            "temperature_reject",
            "temperature",
            290.0,
            310.0,
            DomainEnforcement::Reject,
        ),
    ];
    let model = compile_model(definition, built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let contributions = model
        .component_contributions(&state, &parameters, &input(2.0, 298.15))
        .expect("passing Reject constraint must not upgrade Warn violation");
    assert!(contributions.iter().any(|contribution| {
        contribution.component_id == "equilibrium_nernst"
            && contribution
                .warnings
                .iter()
                .any(|warning| format!("{warning:?}").contains("activity_warn"))
    }));
    let report = model
        .component_validity_reports(&state, &parameters, &input(2.0, 298.15))
        .into_iter()
        .find(|report| report.component_id == "equilibrium_nernst")
        .unwrap();
    assert_eq!(report.domain_status, DomainStatus::OutsideDomain);
    assert_eq!(report.constraint_statuses.len(), 2);

    let mut rejecting = model.definition().clone();
    rejecting.components[1].applicability_constraints[0].enforcement = DomainEnforcement::Reject;
    let rejecting = compile_model(rejecting, built_in_registry()).unwrap();
    let state = rejecting.initialize(&parameters).unwrap();
    assert!(matches!(
        rejecting.component_contributions(&state, &parameters, &input(2.0, 298.15)),
        Err(ModelError::ApplicabilityConstraintRejected {
            constraint_id,
            enforcement: DomainEnforcement::Reject,
            ..
        }) if constraint_id == "activity_warn"
    ));
}

#[test]
fn model_artifact_json_and_writer_share_nested_finite_validation() {
    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    let mut artifact = ModelCompilationArtifact::from_compiled(&model);
    artifact.model_definition.components[1].applicability_constraints = vec![constraint(
        "bad_interval",
        "target_activity",
        f64::NAN,
        1.0,
        DomainEnforcement::Warn,
    )];
    let to_json_path = match artifact.to_json() {
        Err(ModelError::NonFiniteResult { path }) => path,
        other => panic!("expected nested non-finite error, got {other:?}"),
    };
    assert!(to_json_path.contains("applicability_constraints[0].interval.lower"));
    let path = std::env::temp_dir().join(format!("ism_interval_{}.json", std::process::id()));
    std::fs::remove_file(&path).ok();
    let write_path = match write_artifact(&path, &artifact) {
        Err(rust_electroanalysis_cli::domain::ArtifactError::NonFiniteValue {
            field_path, ..
        }) => field_path,
        other => panic!("expected nested non-finite error, got {other:?}"),
    };
    assert_eq!(to_json_path, write_path);
    assert!(!path.exists());
}

#[test]
fn validated_reference_offset_cannot_bind_target_activity_without_consuming_it() {
    let mut definition = reduced_ism_v1_definition();
    let reference = &mut definition.components[4];
    reference.interpretation_status = InterpretationStatus::ValidatedForDomain;
    reference.applicability_constraints = vec![ApplicabilityConstraint {
        id: "target_activity".into(),
        subject: DomainSubject::Input("target_activity".into()),
        interval: NumericInterval {
            lower: 1e-6,
            upper: 1.0,
        },
        source: DomainSource::CalibrationArtifact,
        enforcement: DomainEnforcement::Reject,
        provenance: vec![],
    }];
    assert!(matches!(
        compile_model(definition, built_in_registry()),
        Err(ModelError::UnresolvedApplicabilityBinding { .. })
    ));
}

#[test]
fn nested_nonfinite_outputs_cannot_serialize_as_json_null() {
    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let mut prediction = model
        .observation_prediction(&state, &parameters, &input(0.1, 298.15), None)
        .unwrap();
    prediction.contributions[0]
        .auxiliary_outputs
        .insert("bad".into(), f64::NAN);
    let report = ModelAnalysisReport {
        schema_version: rust_electroanalysis_cli::results::MODEL_RESULT_SCHEMA_VERSION,
        lineage: rust_electroanalysis_cli::domain::legacy_unknown_lineage(),
        artifact_kind: "ism_model_analysis".into(),
        model_definition: model.definition().clone(),
        points: vec![ModelAnalysisPoint {
            time_s: 0.0,
            observed_voltage_v: None,
            predicted_voltage_v: prediction.predicted_voltage_v,
            uncertainty: prediction.uncertainty,
            state_values: state
                .values
                .iter()
                .enumerate()
                .map(|(index, value)| (format!("state_{index}"), *value))
                .collect(),
            contributions: prediction.contributions,
            equilibrium: recognize_equilibrium(
                &evidence(EvidenceValue::NotApplicable),
                &EquilibriumRecognitionConfig::default(),
            ),
            validity: model.validity_report(&state, &parameters, &input(0.1, 298.15)),
            unexplained_residual_v: None,
        }],
        identifiability: model.identifiability_report(),
        evidence: vec![],
    };
    assert!(
        matches!(report.to_json(), Err(ModelError::NonFiniteResult { path }) if path.contains("auxiliary_outputs[\"bad\"]"))
    );
}

#[test]
fn generic_artifact_writer_rejects_nested_nonfinite_before_json_conversion() {
    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let mut prediction = model
        .observation_prediction(&state, &parameters, &input(0.1, 298.15), None)
        .unwrap();
    prediction.contributions[0]
        .auxiliary_outputs
        .insert("bad".into(), f64::NEG_INFINITY);
    let report = ModelAnalysisReport {
        schema_version: rust_electroanalysis_cli::results::MODEL_RESULT_SCHEMA_VERSION,
        lineage: rust_electroanalysis_cli::domain::legacy_unknown_lineage(),
        artifact_kind: "ism_model_analysis".into(),
        model_definition: model.definition().clone(),
        points: vec![ModelAnalysisPoint {
            time_s: 0.0,
            observed_voltage_v: None,
            predicted_voltage_v: prediction.predicted_voltage_v,
            uncertainty: prediction.uncertainty,
            state_values: vec![],
            contributions: prediction.contributions,
            equilibrium: recognize_equilibrium(
                &evidence(EvidenceValue::NotApplicable),
                &EquilibriumRecognitionConfig::default(),
            ),
            validity: model.validity_report(&state, &parameters, &input(0.1, 298.15)),
            unexplained_residual_v: None,
        }],
        identifiability: model.identifiability_report(),
        evidence: vec![],
    };
    let path = std::env::temp_dir().join(format!("ism_nonfinite_{}.json", std::process::id()));
    std::fs::remove_file(&path).ok();
    assert!(
        matches!(write_artifact(&path, &report), Err(rust_electroanalysis_cli::domain::ArtifactError::NonFiniteValue { field_path, .. }) if field_path.contains("auxiliary_outputs[\"bad\"]"))
    );
    assert!(!path.exists());
}

#[test]
fn custom_covariate_units_require_exact_normalized_symbols() {
    let mut definition = reduced_ism_v1_definition();
    definition.inputs[4].id = "relative_humidity".into();
    definition.inputs[4].unit = "%RH".into();
    definition.inputs[4].required = false;
    definition.parameters[2].unit = "V/%RH".into();
    definition.parameters[3].unit = "%RH".into();
    let covariate = &mut definition.components[4];
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
    assert!(
        compile_model(definition.clone(), built_in_registry()).is_ok(),
        "%RH and V/%RH must be supported"
    );
    definition.parameters[2].unit = "V/ppm".into();
    assert!(
        matches!(compile_model(definition, built_in_registry()), Err(ModelError::ParameterUnitMismatch { component, parameter_id, .. }) if component == "humidity_covariate" && parameter_id == "dynamic_fast_tau_s")
    );
}
