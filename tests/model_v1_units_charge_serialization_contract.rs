//! Permanent V1 component-path unit, discrete-charge, and artifact guards.

use rust_electroanalysis_cli::{
    domain::write_artifact,
    model::{
        ComponentRole, ContributionSemantics, EquilibriumEvidence, EquilibriumRecognitionConfig,
        EvidenceValue, InputRequirement, InputValue, ModelError, ModelInput, UncertaintyStatus,
        ValidityReport, built_in_registry, compile_model, recognize_equilibrium,
        reduced_ism_v1_definition,
    },
    results::{
        MODEL_RESULT_SCHEMA_VERSION, ModelAnalysisPoint, ModelAnalysisReport,
        ModelCompilationArtifact,
    },
};
use std::collections::BTreeMap;

fn input() -> ModelInput {
    ModelInput {
        time_s: 0.0,
        values: BTreeMap::from([
            (
                "target_activity".into(),
                InputValue {
                    value: 0.1,
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
fn nernst_component_rejects_every_invalid_discrete_charge_without_coercion() {
    for charge in [-3.0, -2.0, -1.0, 1.0, 2.0, 3.0] {
        let mut definition = reduced_ism_v1_definition();
        definition.parameters[1].default_value = charge;
        let model = compile_model(definition, built_in_registry()).unwrap();
        let parameters = model.default_parameters();
        let state = model.initialize(&parameters).unwrap();
        assert!(
            model
                .component_contributions(&state, &parameters, &input())
                .is_ok(),
            "{charge}"
        );
    }
    for charge in [0.0, 1.5, -1.5] {
        let mut definition = reduced_ism_v1_definition();
        definition.parameters[1].default_value = charge;
        assert!(
            matches!(compile_model(definition, built_in_registry()), Err(ModelError::InvalidDiscreteParameter { parameter_id, .. }) if parameter_id == "ion_charge"),
            "{charge}"
        );
    }
    let mut out_of_range = reduced_ism_v1_definition();
    out_of_range.parameters[1].default_value = i32::MAX as f64 + 1.0;
    assert!(
        matches!(compile_model(out_of_range, built_in_registry()), Err(ModelError::BoundViolation { id, .. }) if id == "ion_charge")
    );
    for charge in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut definition = reduced_ism_v1_definition();
        definition.parameters[1].default_value = charge;
        assert!(
            matches!(compile_model(definition, built_in_registry()), Err(ModelError::NonFinite { subject }) if subject.contains("ion_charge")),
            "{charge}"
        );
    }
    let mut fitted = reduced_ism_v1_definition();
    fitted.parameters[1].value_source =
        rust_electroanalysis_cli::model::ParameterValueSource::Fitted;
    assert!(
        matches!(compile_model(fitted, built_in_registry()), Err(ModelError::InvalidDiscreteParameter { parameter_id, .. }) if parameter_id == "ion_charge")
    );
}

fn nicolsky_definition(charge: f64) -> rust_electroanalysis_cli::model::ModelDefinition {
    let mut definition = reduced_ism_v1_definition();
    definition
        .inputs
        .push(rust_electroanalysis_cli::model::InputSpec {
            id: "interferent_activity".into(),
            unit: "activity".into(),
            required: true,
            source: "test".into(),
            validity_domain: "test".into(),
        });
    let mut selectivity = definition.parameters[0].clone();
    selectivity.id = "selectivity".into();
    selectivity.name = "selectivity".into();
    selectivity.unit = "dimensionless".into();
    selectivity.lower_bound = 0.0;
    selectivity.upper_bound = 10.0;
    selectivity.default_value = 0.01;
    definition.parameters.push(selectivity);
    let nicolsky = &mut definition.components[1];
    nicolsky.kind = "equilibrium.nicolsky_eisenman".into();
    nicolsky.required_inputs.push(InputRequirement {
        id: "interferent_activity".into(),
        unit: "activity".into(),
    });
    nicolsky.parameter_ids.push("selectivity".into());
    nicolsky
        .observation_parameter_ids
        .push("selectivity".into());
    nicolsky.metadata.insert(
        "interferents".into(),
        format!("interferent:{charge}:interferent_activity:selectivity"),
    );
    definition
}

#[test]
fn nicolsky_component_validates_target_and_interferent_charges_on_its_actual_path() {
    let mut valid_input = input();
    valid_input.values.insert(
        "interferent_activity".into(),
        InputValue {
            value: 0.01,
            unit: "activity".into(),
        },
    );
    for charge in [-3.0, -2.0, -1.0, 1.0, 2.0, 3.0] {
        let model = compile_model(nicolsky_definition(charge), built_in_registry()).unwrap();
        let parameters = model.default_parameters();
        let state = model.initialize(&parameters).unwrap();
        assert!(
            model
                .component_contributions(&state, &parameters, &valid_input)
                .is_ok(),
            "{charge}"
        );
    }
    for charge in [0.0, 1.5, -1.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let model = compile_model(nicolsky_definition(charge), built_in_registry()).unwrap();
        let parameters = model.default_parameters();
        let state = model.initialize(&parameters).unwrap();
        assert!(
            matches!(model.component_contributions(&state, &parameters, &valid_input), Err(ModelError::InvalidDiscreteParameter { parameter_id, .. }) if parameter_id == "equilibrium_nernst.interferent.interferent.charge"),
            "{charge}"
        );
    }
    let mut bad_target = nicolsky_definition(-1.0);
    bad_target.parameters[1].default_value = 1.5;
    assert!(
        matches!(compile_model(bad_target, built_in_registry()), Err(ModelError::InvalidDiscreteParameter { parameter_id, .. }) if parameter_id == "ion_charge")
    );
    for charge in [
        i32::MAX as f64 + 1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let model = compile_model(nicolsky_definition(charge), built_in_registry()).unwrap();
        let parameters = model.default_parameters();
        let state = model.initialize(&parameters).unwrap();
        assert!(matches!(
            model.component_contributions(&state, &parameters, &valid_input),
            Err(ModelError::InvalidDiscreteParameter { parameter_id, .. })
                if parameter_id == "equilibrium_nernst.interferent.interferent.charge"
        ));
    }
}

fn covariate(
    unit: &str,
    sensitivity: &str,
    reference: &str,
) -> rust_electroanalysis_cli::model::ModelDefinition {
    let mut definition = reduced_ism_v1_definition();
    definition.inputs[4].id = "covariate".into();
    definition.inputs[4].unit = unit.into();
    definition.parameters[2].unit = sensitivity.into();
    definition.parameters[3].unit = reference.into();
    let component = &mut definition.components[4];
    component.id = "covariate_component".into();
    component.kind = "disturbance.linear_covariate".into();
    component.role = ComponentRole::ExternalDisturbance;
    component.required_inputs = vec![InputRequirement {
        id: "covariate".into(),
        unit: unit.into(),
    }];
    component.state_ids.clear();
    component.observation_state_ids.clear();
    component.parameter_ids = vec![
        "dynamic_fast_tau_s".into(),
        "dynamic_fast_gain_v_per_decade".into(),
    ];
    component.observation_parameter_ids = component.parameter_ids.clone();
    component.output_unit = Some("V".into());
    component.voltage_contribution_owner = Some("covariate".into());
    component.contribution_semantics = ContributionSemantics::AdditivePotential;
    definition
}

#[test]
fn covariate_units_accept_exact_contracts_and_report_precise_mismatches() {
    for (unit, sensitivity) in [
        ("K", "V/K"),
        ("S/m", "V/(S/m)"),
        ("m/s", "V/(m/s)"),
        ("%RH", "V/%RH"),
        ("ppm", "V/ppm"),
    ] {
        assert!(
            compile_model(covariate(unit, sensitivity, unit), built_in_registry()).is_ok(),
            "{unit}"
        );
    }
    for (unit, sensitivity, reference, parameter) in [
        ("K", "V/K", "V", "dynamic_fast_gain_v_per_decade"),
        ("K", "V", "K", "dynamic_fast_tau_s"),
        ("S/m", "V/(S/m)", "V", "dynamic_fast_gain_v_per_decade"),
        ("m/s", "V/(m/s)", "K", "dynamic_fast_gain_v_per_decade"),
        ("%RH", "V/%RH", "ppm", "dynamic_fast_gain_v_per_decade"),
        ("ppm", "V/%RH", "ppm", "dynamic_fast_tau_s"),
    ] {
        assert!(
            matches!(compile_model(covariate(unit, sensitivity, reference), built_in_registry()), Err(ModelError::ParameterUnitMismatch { component, parameter_id, expected, found }) if component == "covariate_component" && parameter_id == parameter && !expected.is_empty() && !found.is_empty())
        );
    }
    let artifact = ModelCompilationArtifact::from_compiled(
        &compile_model(covariate("%RH", "V/%RH", "%RH"), built_in_registry()).unwrap(),
    );
    let json = artifact.to_json().unwrap();
    assert!(json.contains("%RH") && json.contains("V/%RH"));
    let round_trip: ModelCompilationArtifact = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip.to_json().unwrap(), json);
}

fn analysis_report() -> ModelAnalysisReport {
    let model = compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let state = model.initialize(&parameters).unwrap();
    let prediction = model
        .observation_prediction(&state, &parameters, &input(), None)
        .unwrap();
    let evidence = EquilibriumEvidence {
        dynamic_state_derivative_norm: Some(0.0),
        dynamic_potential_magnitude_v: Some(0.0),
        equilibrium_gap_v: Some(0.0),
        elapsed_tau_ratios: vec![4.0],
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
        external_disturbance_potential_v: EvidenceValue::NotApplicable,
    };
    ModelAnalysisReport {
        schema_version: MODEL_RESULT_SCHEMA_VERSION,
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
                .map(|(i, value)| (format!("state_{i}"), *value))
                .collect(),
            contributions: prediction.contributions,
            equilibrium: recognize_equilibrium(&evidence, &EquilibriumRecognitionConfig::default()),
            validity: model.validity_report(&state, &parameters, &input()),
            unexplained_residual_v: None,
        }],
        identifiability: model.identifiability_report(),
        evidence: vec!["finite".into()],
    }
}

#[test]
fn public_serialization_paths_reject_nested_nonfinite_values_without_creating_files() {
    let mut compilation = ModelCompilationArtifact::from_compiled(
        &compile_model(reduced_ism_v1_definition(), built_in_registry()).unwrap(),
    );
    compilation.model_definition.components[1]
        .applicability_constraints
        .push(rust_electroanalysis_cli::model::ApplicabilityConstraint {
            id: "bad".into(),
            subject: rust_electroanalysis_cli::model::DomainSubject::Input(
                "target_activity".into(),
            ),
            interval: rust_electroanalysis_cli::model::NumericInterval {
                lower: f64::NAN,
                upper: 1.0,
            },
            source: rust_electroanalysis_cli::model::DomainSource::CalibrationArtifact,
            enforcement: rust_electroanalysis_cli::model::DomainEnforcement::Warn,
            provenance: vec![],
        });
    assert!(
        matches!(compilation.to_json(), Err(ModelError::NonFiniteResult { path }) if path.contains("interval.lower"))
    );

    let mut report = analysis_report();
    report.points[0].state_values[0].1 = f64::INFINITY;
    report.points[0].contributions[0]
        .auxiliary_outputs
        .insert("nested".into(), f64::NEG_INFINITY);
    let path = std::env::temp_dir().join(format!("ism-v1-nonfinite-{}.json", std::process::id()));
    std::fs::remove_file(&path).ok();
    assert!(
        matches!(report.to_json(), Err(ModelError::NonFiniteResult { path }) if path.contains("state_values[0][1]"))
    );
    assert!(
        matches!(write_artifact(&path, &report), Err(rust_electroanalysis_cli::domain::ArtifactError::NonFiniteValue { field_path, .. }) if field_path.contains("state_values[0][1]"))
    );
    assert!(!path.exists());
    let finite = analysis_report();
    let finite_path =
        std::env::temp_dir().join(format!("ism-v1-finite-{}.json", std::process::id()));
    std::fs::remove_file(&finite_path).ok();
    assert!(finite.to_json().is_ok());
    write_artifact(&finite_path, &finite).unwrap();
    assert!(finite_path.exists());
    std::fs::remove_file(finite_path).unwrap();
}

#[test]
fn analysis_serialization_rejects_each_uncertainty_and_nested_numeric_path() {
    for (field, value) in [
        ("state_variance_v2", f64::NAN),
        ("parameter_variance_v2", f64::INFINITY),
        ("observation_variance_v2", f64::NEG_INFINITY),
        ("total_variance_v2", f64::NAN),
        ("standard_error_v", f64::INFINITY),
        ("equilibrium.confidence", f64::NEG_INFINITY),
        ("contribution.auxiliary", f64::NAN),
        ("unexplained_residual_v", f64::INFINITY),
    ] {
        let mut report = analysis_report();
        match field {
            "state_variance_v2" => report.points[0].uncertainty.state_variance_v2 = Some(value),
            "parameter_variance_v2" => {
                report.points[0].uncertainty.parameter_variance_v2 = Some(value)
            }
            "observation_variance_v2" => {
                report.points[0].uncertainty.observation_variance_v2 = Some(value)
            }
            "total_variance_v2" => report.points[0].uncertainty.total_variance_v2 = Some(value),
            "standard_error_v" => report.points[0].uncertainty.standard_error_v = Some(value),
            "equilibrium.confidence" => report.points[0].equilibrium.confidence = value,
            "contribution.auxiliary" => {
                report.points[0].contributions[0]
                    .auxiliary_outputs
                    .insert("nested".into(), value);
            }
            "unexplained_residual_v" => report.points[0].unexplained_residual_v = Some(value),
            _ => unreachable!(),
        }
        let expected = match field {
            "contribution.auxiliary" => "auxiliary_outputs[\"nested\"]",
            _ => field,
        };
        let json_path = match report.to_json() {
            Err(ModelError::NonFiniteResult { path }) => path,
            other => panic!("{field}: expected non-finite serialization error, got {other:?}"),
        };
        assert!(json_path.contains(expected), "{field}: {json_path}");
        let path = std::env::temp_dir().join(format!("ism-v1-{field}-{}.json", std::process::id()));
        std::fs::remove_file(&path).ok();
        let write_path = match write_artifact(&path, &report) {
            Err(rust_electroanalysis_cli::domain::ArtifactError::NonFiniteValue {
                field_path,
                ..
            }) => field_path,
            other => panic!("{field}: expected writer non-finite error, got {other:?}"),
        };
        assert_eq!(json_path, write_path);
        assert!(!path.exists());
    }
}
