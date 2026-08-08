use rust_electroanalysis_cli::{
    model::{
        EquilibriumEvidence, EquilibriumRecognitionConfig, EquilibriumStatus, InputValue,
        ModelInput, UncertaintyStatus, ValidityReport, built_in_registry, compile_model,
        recognize_equilibrium, reduced_ism_v1_definition,
    },
    potentiometry::calibration::nernst::evaluate_nernst_auto,
};
use std::collections::BTreeMap;

fn input(activity: f64) -> ModelInput {
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
                    value: 298.15,
                    unit: "K".into(),
                },
            ),
        ]),
    }
}

#[test]
fn v1_nernst_is_an_activity_first_adapter_with_parity() {
    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    let state = model.initialize(&model.default_parameters()).unwrap();
    for activity in [1e-6, 1e-3, 0.1, 1.0] {
        let parameters = model.default_parameters();
        let prediction = model
            .observation_prediction(&state, &parameters, &input(activity), None)
            .unwrap();
        let expected = evaluate_nernst_auto(0.0, activity, 298.15, 1).unwrap();
        let equilibrium = prediction
            .contributions
            .iter()
            .find(|item| item.component_id == "equilibrium_nernst")
            .unwrap();
        assert!((equilibrium.potential_v.unwrap() - expected).abs() < 1e-14);
    }
}

#[test]
fn v1_dynamic_modes_have_explicit_event_and_analytical_decay() {
    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let initial = model.initialize(&parameters).unwrap();
    let mut event = input(0.1);
    event.values.insert(
        "delta_log10_activity".into(),
        InputValue {
            value: 2.0,
            unit: "activity".into(),
        },
    );
    let stepped = model
        .process_transition(&initial, &parameters, &event, 0.0)
        .unwrap();
    let fast = model.state_index("dynamic_fast_potential_v").unwrap();
    let slow = model.state_index("dynamic_slow_potential_v").unwrap();
    assert!((stepped.values[fast] - 0.04).abs() < 1e-14);
    assert!((stepped.values[slow] - 0.02).abs() < 1e-14);

    let decayed = model
        .process_transition(&stepped, &parameters, &input(0.1), 2.0)
        .unwrap();
    assert!((decayed.values[fast] - 0.04 / std::f64::consts::E).abs() < 1e-14);
    assert!((decayed.values[slow] - 0.02 * (-2.0_f64 / 35.0).exp()).abs() < 1e-14);
    let derivative = model
        .process_derivative(&decayed, &parameters, &input(0.1))
        .unwrap();
    assert!((derivative[fast] + decayed.values[fast] / 2.0).abs() < 1e-14);
}

#[test]
fn v1_decomposition_keeps_reference_and_variance_out_of_each_others_categories() {
    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let prediction = model
        .observation_prediction(&state, &parameters, &input(0.1), None)
        .unwrap();
    prediction.verify_reconstruction(1e-14).unwrap();
    let totals = prediction.categorized_totals();
    assert_eq!(totals.reference_v, 0.0);
    assert_eq!(totals.observation_variance_v2, 1e-6);
    assert!(
        prediction
            .contributions
            .iter()
            .any(|item| item.component_id == "observation_noise"
                && item.potential_v.is_none()
                && item.variance_v2 == Some(1e-6))
    );
}

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
        external_disturbance_potential_v: Some(0.0),
    }
}

#[test]
fn v1_recognizer_requires_dynamic_evidence_not_a_small_voltage_slope() {
    let config = EquilibriumRecognitionConfig::default();
    assert_eq!(
        recognize_equilibrium(&evidence(), &config).classification,
        EquilibriumStatus::Equilibrium
    );
    let mut slow_state = evidence();
    slow_state.dynamic_potential_magnitude_v = Some(1e-3);
    assert_eq!(
        recognize_equilibrium(&slow_state, &config).classification,
        EquilibriumStatus::Transitional
    );
    let mut missing = evidence();
    missing.uncertainty_status = UncertaintyStatus::Unavailable;
    assert_eq!(
        recognize_equilibrium(&missing, &config).classification,
        EquilibriumStatus::Indeterminate
    );
}
