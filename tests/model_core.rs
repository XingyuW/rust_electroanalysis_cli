use clap::Parser;
use rust_electroanalysis_cli::{
    cli::Cli,
    model::{
        ComponentDescriptor, ComponentFactory, ComponentRegistry, ComponentRole,
        ContributionSemantics, InputRequirement, InputSpec, InputValue, InterpretationStatus,
        IsmComponent, ModelDefinition, ModelError, ModelInput, ModelState, ParameterSpec,
        ParameterValueSource, ParameterValues, StateInitializationSource, StateSpec,
        StateTransformation, UncertaintySpec, compile_model,
    },
    model_config::{MODEL_CONFIG_SCHEMA_VERSION, ModelConfig},
    results::model::{MODEL_COMPILATION_ARTIFACT_KIND, ModelCompilationArtifact},
};
use std::collections::BTreeMap;

#[derive(Debug)]
struct MockVoltageComponent {
    descriptor: ComponentDescriptor,
    voltage_v: f64,
}

impl IsmComponent for MockVoltageComponent {
    fn descriptor(&self) -> &ComponentDescriptor {
        &self.descriptor
    }

    fn observation_voltage(
        &self,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<Option<f64>, ModelError> {
        Ok(Some(self.voltage_v))
    }

    fn observation_jacobian(
        &self,
        _state_dimension: usize,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<Vec<f64>, ModelError> {
        Ok(vec![2.0])
    }

    fn observation_parameter_jacobian(
        &self,
        _parameter_dimension: usize,
        _state: &ModelState,
        _parameters: &ParameterValues,
        _input: &ModelInput,
    ) -> Result<Vec<f64>, ModelError> {
        Ok(vec![3.0])
    }
}

fn mock_factory(descriptor: &ComponentDescriptor) -> Result<Box<dyn IsmComponent>, ModelError> {
    let value = descriptor
        .metadata
        .get("voltage_v")
        .ok_or_else(|| ModelError::MissingReference {
            component: descriptor.id.clone(),
            kind: "mock metadata",
            id: "voltage_v".into(),
        })?
        .parse::<f64>()
        .map_err(|_| ModelError::NonFinite {
            subject: format!("mock component '{}' voltage", descriptor.id),
        })?;
    if !value.is_finite() {
        return Err(ModelError::NonFinite {
            subject: format!("mock component '{}' voltage", descriptor.id),
        });
    }
    Ok(Box::new(MockVoltageComponent {
        descriptor: descriptor.clone(),
        voltage_v: value,
    }))
}

fn registry() -> ComponentRegistry {
    ComponentRegistry::from_static_factories([(
        "test.constant_voltage",
        mock_factory as ComponentFactory,
    )])
}

fn component(id: &str, role: ComponentRole, owner: &str, voltage_v: f64) -> ComponentDescriptor {
    ComponentDescriptor {
        id: id.into(),
        kind: "test.constant_voltage".into(),
        role,
        interpretation_status: InterpretationStatus::Phenomenological,
        depends_on: Vec::new(),
        required_inputs: vec![InputRequirement {
            id: "temperature".into(),
            unit: "K".into(),
        }],
        state_ids: vec!["memory".into()],
        parameter_ids: vec!["offset".into()],
        output_unit: Some("V".into()),
        voltage_contribution_owner: Some(owner.into()),
        contribution_semantics: ContributionSemantics::AdditivePotential,
        legacy_composition_rule: None,
        source: "test fixture".into(),
        validity_domain: "synthetic bounded fixture".into(),
        equation: "test constant voltage".into(),
        equation_version: 1,
        assumptions: vec!["test-only component".into()],
        evidence_requirements: vec![rust_electroanalysis_cli::EvidenceRequirement {
            hypothesis_id: "test".into(),
            proposed_mechanism_label: "unassigned".into(),
            independent_evidence_types: vec!["test".into()],
            minimum_independent_observations: 1,
            validity_domain: "test".into(),
            alternatives_to_consider: vec!["test".into()],
            required_uncertainty_statement: "test".into(),
        }],
        metadata: BTreeMap::from([("voltage_v".into(), voltage_v.to_string())]),
    }
}

fn definition() -> ModelDefinition {
    ModelDefinition {
        schema_version: 2,
        model_id: "test-model".into(),
        description: "mock-component compilation fixture".into(),
        validity_domain: "synthetic bounded fixture".into(),
        uncertainty_incomplete: false,
        states: vec![StateSpec {
            id: "memory".into(),
            name: "memory".into(),
            description: "synthetic test memory state".into(),
            unit: "V".into(),
            transformation: StateTransformation::Identity,
            initialization_source: StateInitializationSource::DeclaredDefault,
            lower_bound: -1.0,
            upper_bound: 1.0,
            initial_value: 0.0,
            source: "test fixture".into(),
            process_equation_version: 1,
            observability_requirements: vec!["synthetic observable state".into()],
            validity_domain: "synthetic bounded fixture".into(),
            initial_uncertainty: UncertaintySpec::Variance {
                value: 0.01,
                unit: "V^2".into(),
            },
        }],
        parameters: vec![ParameterSpec {
            id: "offset".into(),
            name: "offset".into(),
            description: "synthetic test offset".into(),
            unit: "V".into(),
            lower_bound: -1.0,
            upper_bound: 1.0,
            default_value: 0.0,
            uncertainty: UncertaintySpec::StandardDeviation {
                value: 0.01,
                unit: "V".into(),
            },
            source: "test fixture".into(),
            equation_version: 1,
            identifiability_requirements: vec!["synthetic identifiable parameter".into()],
            value_source: ParameterValueSource::Fixed,
            validity_domain: "synthetic bounded fixture".into(),
        }],
        inputs: vec![InputSpec {
            id: "temperature".into(),
            unit: "K".into(),
            required: false,
            source: "test fixture".into(),
            validity_domain: "synthetic bounded fixture".into(),
        }],
        components: vec![
            component(
                "equilibrium",
                ComponentRole::Equilibrium,
                "equilibrium",
                0.2,
            ),
            component(
                "external",
                ComponentRole::ExternalDisturbance,
                "external",
                0.05,
            ),
        ],
    }
}

fn input() -> ModelInput {
    ModelInput {
        time_s: 0.0,
        values: BTreeMap::from([(
            "temperature".into(),
            InputValue {
                value: 298.15,
                unit: "K".into(),
            },
        )]),
    }
}

#[test]
fn rejects_duplicate_component_id() {
    let mut model = definition();
    model.components.push(component(
        "equilibrium",
        ComponentRole::ExternalDisturbance,
        "different-owner",
        0.0,
    ));
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::DuplicateIdentifier {
            kind: "component",
            ..
        })
    ));
}

#[test]
fn rejects_missing_dependency() {
    let mut model = definition();
    model.components[0].depends_on = vec!["absent".into()];
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::MissingDependency { .. })
    ));
}

#[test]
fn rejects_missing_declared_input() {
    let mut model = definition();
    model.components[0].required_inputs[0].id = "activity".into();
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::MissingInput { .. })
    ));
}

#[test]
fn rejects_circular_dependency() {
    let mut model = definition();
    model.components[0].depends_on = vec!["external".into()];
    model.components[1].depends_on = vec!["equilibrium".into()];
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::CircularDependency { .. })
    ));
}

#[test]
fn rejects_unit_mismatch() {
    let mut model = definition();
    model.inputs[0].unit = "V".into();
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::UnitMismatch { .. })
    ));
}

#[test]
fn rejects_duplicate_voltage_contribution_owner() {
    let mut model = definition();
    model.components[1].voltage_contribution_owner = Some("equilibrium".into());
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::DuplicateContributionOwner { .. })
    ));
}

#[test]
fn rejects_parameter_bound_violation() {
    let compiled = compile_model(definition(), &registry()).expect("compile fixture");
    assert!(matches!(
        compiled.initialize(&ParameterValues::new(vec![1.1])),
        Err(ModelError::BoundViolation {
            kind: "parameter",
            ..
        })
    ));
}

#[test]
fn preserves_state_index_stability() {
    let first = compile_model(definition(), &registry()).expect("compile first fixture");
    let second = compile_model(definition(), &registry()).expect("compile second fixture");
    assert_eq!(first.state_index("memory"), Some(0));
    assert_eq!(first.state_index("memory"), second.state_index("memory"));
}

#[test]
fn compilation_is_deterministic() {
    let first = compile_model(definition(), &registry()).expect("compile first fixture");
    let second = compile_model(definition(), &registry()).expect("compile second fixture");
    let parameters = first.default_parameters();
    let state = first.initialize(&parameters).expect("initialize fixture");
    let first_ids: Vec<_> = first
        .component_contributions(&state, &parameters, &input())
        .expect("evaluate first fixture")
        .into_iter()
        .map(|item| item.component_id)
        .collect();
    let second_parameters = second.default_parameters();
    let second_state = second
        .initialize(&second_parameters)
        .expect("initialize second fixture");
    let second_ids: Vec<_> = second
        .component_contributions(&second_state, &second_parameters, &input())
        .expect("evaluate second fixture")
        .into_iter()
        .map(|item| item.component_id)
        .collect();
    assert_eq!(first_ids, second_ids);
}

#[test]
fn contribution_sum_equals_predicted_output() {
    let compiled = compile_model(definition(), &registry()).expect("compile fixture");
    let parameters = compiled.default_parameters();
    let state = compiled
        .initialize(&parameters)
        .expect("initialize fixture");
    let prediction = compiled
        .observation_prediction(&state, &parameters, &input(), Some(0.30))
        .expect("predict fixture");
    let contribution_sum: f64 = prediction
        .contributions
        .iter()
        .filter_map(|item| item.potential_v)
        .sum();
    assert!((contribution_sum - prediction.predicted_voltage_v).abs() < 1e-12);
    assert_eq!(prediction.predicted_voltage_v, 0.25);
}

#[test]
fn rejects_invalid_model() {
    let mut model = definition();
    model.schema_version = 999;
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::UnsupportedSchemaVersion { .. })
    ));
}

#[test]
fn old_cli_workflow_surface_is_unchanged() {
    assert!(Cli::try_parse_from(["electroanalysis", "plot"]).is_ok());
    assert!(Cli::try_parse_from(["electroanalysis", "eis", "fit", "sample.csv"]).is_ok());
}

#[test]
fn model_config_and_artifact_have_stable_semantics() {
    let model = definition();
    let config = ModelConfig {
        schema_version: MODEL_CONFIG_SCHEMA_VERSION,
        model: model.clone(),
    };
    let text = toml::to_string(&config).expect("serialize model config");
    let restored: ModelConfig = toml::from_str(&text).expect("deserialize model config");
    restored.validate().expect("validate model config");

    let compiled = compile_model(model, &registry()).expect("compile model artifact fixture");
    let artifact = ModelCompilationArtifact::from_compiled(&compiled);
    assert_eq!(artifact.artifact_kind, MODEL_COMPILATION_ARTIFACT_KIND);
    let json = artifact.to_json().expect("serialize finite model artifact");
    assert!(json.contains("\"schema_version\": 2"));
    assert!(!json.contains("NaN"));
    assert!(!json.contains("Infinity"));
}

#[test]
fn model_artifact_rejects_nonfinite_definition_values() {
    let compiled = compile_model(definition(), &registry()).expect("compile artifact fixture");
    let mut artifact = ModelCompilationArtifact::from_compiled(&compiled);
    artifact.model_definition.parameters[0].default_value = f64::NAN;
    assert!(matches!(
        artifact.to_json(),
        Err(ModelError::NonFinite { .. })
    ));
}

#[test]
fn legacy_external_role_is_canonicalized_on_deserialization() {
    let role: ComponentRole = serde_json::from_str("\"external\"").unwrap();
    assert_eq!(role, ComponentRole::ExternalDisturbance);
    assert_eq!(
        serde_json::to_string(&role).unwrap(),
        "\"external_disturbance\""
    );
}

#[test]
fn unsupported_legacy_composition_rule_is_rejected() {
    let mut model = definition();
    model.components[0].legacy_composition_rule = Some("multiply_voltage".into());
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::UnsupportedCompositionSemantics { .. })
    ));
}

#[test]
fn state_only_or_auxiliary_output_cannot_enter_voltage_sum() {
    for semantics in [
        ContributionSemantics::StateOnly,
        ContributionSemantics::Auxiliary,
    ] {
        let mut model = definition();
        model.components.truncate(1);
        model.components[0].contribution_semantics = semantics;
        model.components[0].voltage_contribution_owner = None;
        model.components[0].output_unit = None;
        let compiled = compile_model(model, &registry()).unwrap();
        let parameters = compiled.default_parameters();
        let state = compiled.initialize(&parameters).unwrap();
        assert!(matches!(
            compiled.component_contributions(&state, &parameters, &input()),
            Err(ModelError::IncompatibleContributionOutput { .. })
        ));
    }
}

#[test]
fn reconstruction_failure_is_typed() {
    let compiled = compile_model(definition(), &registry()).unwrap();
    let parameters = compiled.default_parameters();
    let state = compiled.initialize(&parameters).unwrap();
    let mut prediction = compiled
        .observation_prediction(&state, &parameters, &input(), None)
        .unwrap();
    prediction.predicted_voltage_v += 1.0;
    assert!(matches!(
        prediction.verify_reconstruction(1e-12),
        Err(ModelError::ContributionReconstruction { .. })
    ));
}

#[test]
fn fitted_missing_or_zero_uncertainty_is_rejected_unless_incomplete() {
    let mut missing = definition();
    missing.parameters[0].value_source = ParameterValueSource::Fitted;
    missing.parameters[0].uncertainty = UncertaintySpec::Unknown {
        reason: "missing".into(),
    };
    assert!(matches!(
        compile_model(missing, &registry()),
        Err(ModelError::InvalidUncertainty { .. })
    ));
    let mut zero = definition();
    zero.parameters[0].value_source = ParameterValueSource::Fitted;
    zero.parameters[0].uncertainty = UncertaintySpec::StandardDeviation {
        value: 0.0,
        unit: "V".into(),
    };
    assert!(matches!(
        compile_model(zero, &registry()),
        Err(ModelError::InvalidUncertainty { .. })
    ));
}

#[test]
fn estimated_state_requires_explicit_uncertainty() {
    let mut model = definition();
    model.states[0].initialization_source = StateInitializationSource::Estimated;
    model.states[0].initial_uncertainty = UncertaintySpec::Unknown {
        reason: "missing".into(),
    };
    assert!(matches!(
        compile_model(model, &registry()),
        Err(ModelError::InvalidUncertainty { .. })
    ));
}

#[test]
fn explicit_covariance_propagates_first_order_state_and_parameter_variance() {
    let mut model = definition();
    model.components.truncate(1);
    let compiled = compile_model(model, &registry()).unwrap();
    let parameters = compiled.default_parameters();
    let state = compiled.initialize(&parameters).unwrap();
    let prediction = compiled
        .observation_prediction_with_uncertainty(
            &state,
            &parameters,
            &input(),
            None,
            rust_electroanalysis_cli::model::PredictionUncertaintyInput {
                state_covariance: Some(vec![vec![0.5]]),
                parameter_covariance: Some(vec![vec![0.25]]),
            },
        )
        .unwrap();
    assert_eq!(
        prediction.uncertainty.status,
        rust_electroanalysis_cli::model::UncertaintyStatus::Complete
    );
    assert!((prediction.uncertainty.state_variance_v2.unwrap() - 2.0).abs() < 1e-12);
    assert!((prediction.uncertainty.parameter_variance_v2.unwrap() - 2.25).abs() < 1e-12);
    assert!((prediction.uncertainty.total_variance_v2.unwrap() - 4.25).abs() < 1e-12);
}
