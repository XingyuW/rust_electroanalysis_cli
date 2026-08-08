//! Permanent scientific guardrail regression coverage for Reduced-Order ISM V1.

use rust_electroanalysis_cli::model::{
    ComponentApplicabilityDomain, DomainEnforcement, DomainSource, DomainStatus,
    EquilibriumEvidence, EquilibriumRecognitionConfig, EquilibriumStatus, EvidenceValue,
    InputValue, InterpretationStatus, ModelError, ModelInput, NumericInterval, UncertaintyStatus,
    ValidityReport, built_in_registry, compile_model, exact_nonzero_charge, recognize_equilibrium,
    reduced_ism_v1_definition,
};
use rust_electroanalysis_cli::results::{ModelAnalysisPoint, ModelAnalysisReport};
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
        schema_version: 3,
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
