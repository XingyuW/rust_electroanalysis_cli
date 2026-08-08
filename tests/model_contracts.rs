//! Architectural tests for the scientific-contract boundary. These tests do
//! not validate a physical transport equation; they protect the invariants
//! required before one can be introduced.

use rust_electroanalysis_cli::model::{
    ComponentRole, ModelError, built_in_registry, compile_model, default_model_definition,
};

#[test]
fn stable_component_ids_are_present_and_unique() {
    let definition = default_model_definition();
    assert!(
        definition
            .components
            .iter()
            .all(|component| !component.id.trim().is_empty())
    );
    assert!(compile_model(definition, built_in_registry()).is_ok());
}

#[test]
fn duplicate_state_and_parameter_ids_are_rejected() {
    let mut duplicate_state = default_model_definition();
    duplicate_state
        .states
        .push(duplicate_state.states[0].clone());
    assert!(matches!(
        compile_model(duplicate_state, built_in_registry()),
        Err(ModelError::DuplicateIdentifier { kind: "state", .. })
    ));

    let mut duplicate_parameter = default_model_definition();
    duplicate_parameter
        .parameters
        .push(duplicate_parameter.parameters[0].clone());
    assert!(matches!(
        compile_model(duplicate_parameter, built_in_registry()),
        Err(ModelError::DuplicateIdentifier {
            kind: "parameter",
            ..
        })
    ));
}

#[test]
fn states_parameters_components_and_validity_are_declared() {
    let definition = default_model_definition();
    assert!(definition.states.iter().all(|state| {
        !state.id.trim().is_empty()
            && !state.name.trim().is_empty()
            && !state.description.trim().is_empty()
            && !state.unit.trim().is_empty()
            && !state.validity_domain.trim().is_empty()
            && state.process_equation_version > 0
    }));
    assert!(definition.parameters.iter().all(|parameter| {
        !parameter.id.trim().is_empty()
            && !parameter.name.trim().is_empty()
            && !parameter.description.trim().is_empty()
            && !parameter.unit.trim().is_empty()
            && !parameter.validity_domain.trim().is_empty()
            && parameter.equation_version > 0
    }));
    assert!(definition.components.iter().all(|component| {
        !component.source.trim().is_empty()
            && !component.validity_domain.trim().is_empty()
            && !component.evidence_requirements.is_empty()
    }));
}

#[test]
fn unexplained_and_observation_noise_never_become_voltage_terms() {
    let mut noise = default_model_definition();
    let observation_noise = noise
        .components
        .iter_mut()
        .find(|component| component.role == ComponentRole::ObservationNoise)
        .expect("default model contains an observation-noise declaration");
    observation_noise.output_unit = Some("V".into());
    observation_noise.voltage_contribution_owner = Some("noise".into());
    observation_noise.composition_rule = Some("additive_voltage".into());
    assert!(matches!(
        compile_model(noise, built_in_registry()),
        Err(ModelError::InvalidComponentShape { .. })
    ));

    let mut unexplained = default_model_definition();
    let component = unexplained
        .components
        .last_mut()
        .expect("default components");
    component.role = ComponentRole::Unexplained;
    component.output_unit = Some("V".into());
    component.voltage_contribution_owner = Some("unexplained".into());
    component.composition_rule = Some("additive_voltage".into());
    assert!(matches!(
        compile_model(unexplained, built_in_registry()),
        Err(ModelError::InvalidComponentShape { .. })
    ));
}

#[test]
fn model_core_has_no_forbidden_high_level_dependencies() {
    let forbidden = [
        "crate::cli",
        "crate::runners",
        "crate::plottings",
        "crate::health",
        "crate::mechanism",
        "crate::estimation",
        "crate::results",
    ];
    let sources = [
        include_str!("../src/model/mod.rs"),
        include_str!("../src/model/error.rs"),
        include_str!("../src/model/definition.rs"),
        include_str!("../src/model/component.rs"),
        include_str!("../src/model/registry.rs"),
        include_str!("../src/model/graph.rs"),
        include_str!("../src/model/compiler.rs"),
        include_str!("../src/model/parameter.rs"),
        include_str!("../src/model/state.rs"),
        include_str!("../src/model/input.rs"),
        include_str!("../src/model/output.rs"),
        include_str!("../src/model/validity.rs"),
        include_str!("../src/model/identifiability.rs"),
        include_str!("../src/model/evidence.rs"),
        include_str!("../src/model/equilibrium_recognition.rs"),
        include_str!("../src/model/defaults.rs"),
        include_str!("../src/model/builtins.rs"),
    ];

    for source in sources {
        for dependency in forbidden {
            assert!(
                !source.contains(dependency),
                "model core must not depend on {dependency}"
            );
        }
    }
}
