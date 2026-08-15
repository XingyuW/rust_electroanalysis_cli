use rust_electroanalysis_cli::model::{
    ComponentDescriptor, ComponentFactory, ComponentRegistry, InitializationContext, ModelError,
    StateInitializationSource, built_in_registry, compile_model, default_model_definition,
};
use std::collections::BTreeMap;

fn rejected_factory(
    descriptor: &ComponentDescriptor,
) -> Result<Box<dyn rust_electroanalysis_cli::model::IsmComponent>, ModelError> {
    Err(ModelError::ComponentEvaluation {
        component: descriptor.id.clone(),
        message: "test factory is never constructed".into(),
    })
}

#[test]
fn registry_rejects_duplicate_kind_without_replacing_the_original() {
    let mut registry = ComponentRegistry::new();
    registry
        .register("test.synthetic", rejected_factory as ComponentFactory)
        .unwrap();
    assert!(matches!(
        registry.register("test.synthetic", rejected_factory as ComponentFactory),
        Err(ModelError::DuplicateComponentKind { .. })
    ));
}

#[test]
fn graph_rejects_self_dependency_explicitly() {
    let mut definition = default_model_definition();
    definition.components[0].depends_on = vec![definition.components[0].id.clone()];
    assert!(matches!(
        compile_model(definition, built_in_registry()),
        Err(ModelError::SelfDependency { .. })
    ));
}

#[test]
fn compiled_bindings_summary_and_neutral_initialization_are_stable() {
    let definition = default_model_definition();
    let model = compile_model(definition, built_in_registry()).unwrap();
    let parameters = model.default_parameters();
    let initialized = model
        .initialize_with_context(
            &parameters,
            &InitializationContext {
                state_values: BTreeMap::from([("fast_mode_v".into(), 0.25)]),
                source: Some(StateInitializationSource::Measurement),
                known_experimental_context: BTreeMap::new(),
            },
        )
        .unwrap();
    assert_eq!(
        initialized.state.values[model.state_index("fast_mode_v").unwrap()],
        0.25
    );
    assert_eq!(
        initialized.sources[model.state_index("fast_mode_v").unwrap()],
        StateInitializationSource::Measurement
    );
    assert_eq!(model.state_id(0), Some("fast_mode_v"));
    assert_eq!(model.parameter_id(0), Some("standard_potential_v"));
    assert_eq!(model.state_slice("fast_mode"), Some([0].as_slice()));
    assert_eq!(model.parameter_slice("fast_mode"), Some([2, 3].as_slice()));
    let summary = model.compiled_summary();
    assert_eq!(
        summary.component_order,
        vec![
            "baseline_drift",
            "equilibrium",
            "fast_mode",
            "observation_noise",
            "slow_mode"
        ]
    );
    let round_trip: rust_electroanalysis_cli::model::CompiledModelSummary =
        serde_json::from_str(&serde_json::to_string(&summary).unwrap()).unwrap();
    assert_eq!(summary, round_trip);
}
