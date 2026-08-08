use rust_electroanalysis_cli::{
    model::{
        InputRequirement, InputSpec, ModelInput, ParameterSpec, ParameterValueSource,
        UncertaintySpec, built_in_registry, compile_model, default_model_definition,
    },
    potentiometry::calibration::nicolsky_eisenman::{InterferentModelInput, evaluate_potential},
};
use std::collections::BTreeMap;

fn input(time_s: f64) -> ModelInput {
    ModelInput {
        time_s,
        values: BTreeMap::from([
            (
                "primary_concentration".into(),
                rust_electroanalysis_cli::model::InputValue {
                    value: 0.01,
                    unit: "mol/L".into(),
                },
            ),
            (
                "temperature".into(),
                rust_electroanalysis_cli::model::InputValue {
                    value: 298.15,
                    unit: "K".into(),
                },
            ),
            (
                "driving_step_v".into(),
                rust_electroanalysis_cli::model::InputValue {
                    value: 0.1,
                    unit: "V".into(),
                },
            ),
        ]),
    }
}

fn compiled() -> rust_electroanalysis_cli::CompiledIsmModel {
    compile_model(default_model_definition(), built_in_registry())
        .expect("compile default reduced-order model")
}

#[test]
fn pure_nernst_equilibrium_uses_existing_adapter() {
    let model = compiled();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let prediction = model
        .observation_prediction(&state, &parameters, &input(0.0), None)
        .unwrap();
    assert!(prediction.predicted_voltage_v > -0.2 && prediction.predicted_voltage_v < 0.0);
}

#[test]
fn nicolsky_eisenman_with_interferents() {
    let mut definition = default_model_definition();
    let equilibrium = &mut definition.components[0];
    equilibrium.kind = "equilibrium.nicolsky_eisenman".into();
    equilibrium.equation = "EQ-CAL-002 adapter".into();
    equilibrium.required_inputs.push(InputRequirement {
        id: "na_activity".into(),
        unit: "activity".into(),
    });
    equilibrium.parameter_ids.insert(2, "selectivity_na".into());
    equilibrium.metadata.insert(
        "interferents".into(),
        "Na+:1:na_activity:selectivity_na".into(),
    );
    definition.parameters.insert(
        2,
        ParameterSpec {
            id: "selectivity_na".into(),
            name: "sodium selectivity".into(),
            description: "Synthetic selectivity coefficient.".into(),
            unit: "dimensionless".into(),
            lower_bound: 1e-9,
            upper_bound: 1.0,
            default_value: 0.1,
            uncertainty: UncertaintySpec::StandardDeviation {
                value: 0.01,
                unit: "dimensionless".into(),
            },
            source: "synthetic test".into(),
            equation_version: 1,
            identifiability_requirements: vec!["synthetic test parameter".into()],
            value_source: ParameterValueSource::Fixed,
            validity_domain: "synthetic".into(),
        },
    );
    definition.inputs.push(InputSpec {
        id: "na_activity".into(),
        unit: "activity".into(),
        required: true,
        source: "synthetic test".into(),
        validity_domain: "synthetic".into(),
    });
    let model = compile_model(definition, built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let mut values = input(0.0);
    values.values.insert(
        "na_activity".into(),
        rust_electroanalysis_cli::model::InputValue {
            value: 0.1,
            unit: "activity".into(),
        },
    );
    let predicted = model
        .component_contributions(&state, &parameters, &values)
        .unwrap()
        .into_iter()
        .find(|item| item.component_id == "equilibrium")
        .unwrap()
        .potential_v
        .unwrap();
    let expected = evaluate_potential(
        0.0,
        0.01,
        1,
        298.15,
        &[InterferentModelInput {
            name: "Na+".into(),
            charge: 1,
            activity: 0.1,
            selectivity_coefficient: 0.1,
        }],
    )
    .unwrap();
    assert!((predicted - expected).abs() < 1e-12);
}

#[test]
fn single_relaxation_after_activity_step() {
    let model = compiled();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let next = model
        .process_transition(&state, &parameters, &input(0.0), 1.0)
        .unwrap();
    assert!(next.values[0] > 0.0 && next.values[0] <= 0.1);
}

#[test]
fn relaxation_transition_is_timestep_invariant_at_equal_elapsed_time() {
    let model = compiled();
    let parameters = model.default_parameters();
    let initial = model.initialize(&parameters).unwrap();
    let one_step = model
        .process_transition(&initial, &parameters, &input(0.0), 1.0)
        .unwrap();
    let half = model
        .process_transition(&initial, &parameters, &input(0.0), 0.5)
        .unwrap();
    let two_steps = model
        .process_transition(&half, &parameters, &input(0.5), 0.5)
        .unwrap();
    assert!((one_step.values[0] - two_steps.values[0]).abs() < 1e-12);
    assert!((one_step.values[1] - two_steps.values[1]).abs() < 1e-12);
}

#[test]
fn nernst_activity_uses_declared_ion_charge() {
    let model = compiled();
    let mut monovalent = model.default_parameters();
    let state = model.initialize(&monovalent).unwrap();
    let mono = model
        .observation_prediction(&state, &monovalent, &input(0.0), None)
        .unwrap()
        .predicted_voltage_v;
    monovalent.values[1] = 2.0;
    let divalent = model
        .observation_prediction(&state, &monovalent, &input(0.0), None)
        .unwrap()
        .predicted_voltage_v;
    assert!((mono / divalent - 2.0).abs() < 1e-10);
}

#[test]
fn malformed_builtin_descriptor_is_rejected_without_panicking() {
    let mut definition = default_model_definition();
    definition.components[1].state_ids.clear();
    assert!(matches!(
        compile_model(definition, built_in_registry()),
        Err(rust_electroanalysis_cli::model::ModelError::InvalidComponentShape { .. })
    ));
}

#[test]
fn two_separated_relaxation_modes_are_distinct() {
    let model = compiled();
    let mut parameters = model.default_parameters();
    parameters.values[2] = 1.0;
    parameters.values[4] = 100.0;
    let state = model.initialize(&parameters).unwrap();
    let next = model
        .process_transition(&state, &parameters, &input(0.0), 1.0)
        .unwrap();
    assert!(next.values[0] > next.values[1]);
}

#[test]
fn nearly_unidentifiable_two_mode_relaxation_warns() {
    let mut definition = default_model_definition();
    definition.components[1].kind = "transport.two_mode_relaxation".into();
    definition.components[1].state_ids = vec!["fast_mode_v".into(), "slow_mode_v".into()];
    definition.components[1].parameter_ids = vec![
        "fast_tau_s".into(),
        "fast_gain".into(),
        "slow_tau_s".into(),
        "slow_gain".into(),
    ];
    definition.components.remove(2);
    definition.components[1].voltage_contribution_owner = Some("transport_modes".into());
    let model = compile_model(definition, built_in_registry()).unwrap();
    let mut parameters = model.default_parameters();
    parameters.values[2] = 1.0;
    parameters.values[4] = 1.5;
    let state = model.initialize(&parameters).unwrap();
    let report = model.validity_report(&state, &parameters, &input(0.0));
    assert!(!report.warnings.is_empty());
}

#[test]
fn baseline_drift_is_explicit() {
    let model = compiled();
    let mut parameters = model.default_parameters();
    parameters.values[6] = 0.002;
    let state = model.initialize(&parameters).unwrap();
    let contribution = model
        .component_contributions(&state, &parameters, &input(10.0))
        .unwrap()
        .into_iter()
        .find(|item| item.component_id == "baseline_drift")
        .unwrap();
    assert!((contribution.potential_v.unwrap() - 0.02).abs() < 1e-12);
}

#[test]
fn temperature_perturbation_changes_nernst_prediction() {
    let model = compiled();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let cold = model
        .observation_prediction(&state, &parameters, &input(0.0), None)
        .unwrap()
        .predicted_voltage_v;
    let mut warm = input(0.0);
    warm.values.get_mut("temperature").unwrap().value = 320.0;
    let hot = model
        .observation_prediction(&state, &parameters, &warm, None)
        .unwrap()
        .predicted_voltage_v;
    assert!(hot < cold);
}

#[test]
fn conductivity_disturbance_is_a_named_external_contribution() {
    let mut definition = default_model_definition();
    let component = &mut definition.components[3];
    component.id = "conductivity".into();
    component.kind = "disturbance.conductivity_covariate".into();
    component.parameter_ids = vec![
        "conductivity_sensitivity".into(),
        "conductivity_reference".into(),
    ];
    component.required_inputs = vec![InputRequirement {
        id: "conductivity".into(),
        unit: "S/m".into(),
    }];
    component.voltage_contribution_owner = Some("conductivity".into());
    component.equation = "linear conductivity covariate".into();
    definition.parameters.push(ParameterSpec {
        id: "conductivity_sensitivity".into(),
        name: "conductivity sensitivity".into(),
        description: "Synthetic conductivity sensitivity.".into(),
        unit: "V/(S/m)".into(),
        lower_bound: -1.0,
        upper_bound: 1.0,
        default_value: 0.01,
        uncertainty: UncertaintySpec::StandardDeviation {
            value: 0.001,
            unit: "V/(S/m)".into(),
        },
        source: "synthetic".into(),
        equation_version: 1,
        identifiability_requirements: vec!["synthetic test parameter".into()],
        value_source: ParameterValueSource::Fixed,
        validity_domain: "synthetic".into(),
    });
    definition.parameters.push(ParameterSpec {
        id: "conductivity_reference".into(),
        name: "conductivity reference".into(),
        description: "Synthetic conductivity reference.".into(),
        unit: "S/m".into(),
        lower_bound: 0.0,
        upper_bound: 10.0,
        default_value: 1.0,
        uncertainty: UncertaintySpec::Deterministic,
        source: "synthetic".into(),
        equation_version: 1,
        identifiability_requirements: vec!["synthetic test parameter".into()],
        value_source: ParameterValueSource::Fixed,
        validity_domain: "synthetic".into(),
    });
    definition.inputs.push(InputSpec {
        id: "conductivity".into(),
        unit: "S/m".into(),
        required: true,
        source: "synthetic".into(),
        validity_domain: "synthetic".into(),
    });
    let model = compile_model(definition, built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let mut values = input(0.0);
    values.values.insert(
        "conductivity".into(),
        rust_electroanalysis_cli::model::InputValue {
            value: 3.0,
            unit: "S/m".into(),
        },
    );
    let value = model
        .component_contributions(&state, &parameters, &values)
        .unwrap()
        .into_iter()
        .find(|item| item.component_id == "conductivity")
        .unwrap()
        .potential_v
        .unwrap();
    assert!((value - 0.02).abs() < 1e-12);
}

#[test]
fn contribution_reconstruction_is_exact() {
    let model = compiled();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let prediction = model
        .observation_prediction(&state, &parameters, &input(0.0), Some(0.0))
        .unwrap();
    assert!(
        (prediction.predicted_voltage_v
            - prediction
                .contributions
                .iter()
                .filter_map(|item| item.potential_v)
                .sum::<f64>())
        .abs()
            < 1e-12
    );
}

#[test]
fn observation_variance_is_categorized_and_never_added_to_voltage() {
    let model = compiled();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let prediction = model
        .observation_prediction(&state, &parameters, &input(0.0), None)
        .unwrap();
    let noise = prediction
        .contributions
        .iter()
        .find(|item| item.component_id == "observation_noise")
        .unwrap();
    assert!(noise.potential_v.is_none());
    assert_eq!(noise.variance_v2, Some(1.0e-6));
    assert_eq!(prediction.uncertainty.observation_variance_v2, Some(1.0e-6));
}

#[test]
fn component_validity_warnings_are_exposed() {
    let mut definition = default_model_definition();
    definition.components[0]
        .metadata
        .insert("maximum_time_s".into(), "1".into());
    let model = compile_model(definition, built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    assert!(
        !model
            .validity_report(&state, &parameters, &input(2.0))
            .warnings
            .is_empty()
    );
}
