use super::{
    ComponentDescriptor, ComponentRole, EvidenceRequirement, InputRequirement, InputSpec,
    ModelDefinition, ParameterSpec, StateSpec,
};
use std::collections::BTreeMap;

/// Default reduced-order model: equilibrium, fast/slow phenomenological modes,
/// baseline drift, and zero-mean observation-noise metadata.
pub fn default_model_definition() -> ModelDefinition {
    ModelDefinition {
        schema_version: super::MODEL_DEFINITION_SCHEMA_VERSION,
        model_id: "ism-reduced-order-v1".into(),
        description: "Reduced-order ISM baseline with explicit phenomenological modes.".into(),
        validity_domain:
            "Aqueous potentiometric steps within declared calibration/activity domains.".into(),
        states: vec![state("fast_mode_v"), state("slow_mode_v")],
        parameters: vec![
            parameter("standard_potential_v", "V", -2.0, 2.0, 0.0),
            parameter("ion_charge", "dimensionless", -4.0, 4.0, 1.0),
            parameter("fast_tau_s", "s", 1e-4, 1e5, 1.0),
            parameter("fast_gain", "dimensionless", -10.0, 10.0, 1.0),
            parameter("slow_tau_s", "s", 1e-4, 1e6, 30.0),
            parameter("slow_gain", "dimensionless", -10.0, 10.0, 0.2),
            parameter("baseline_drift_v_per_s", "V/s", -1.0, 1.0, 0.0),
            parameter("observation_noise_std_v", "V", 0.0, 1.0, 1e-3),
        ],
        inputs: vec![
            input("primary_concentration", "mol/L", true),
            input("temperature", "K", true),
            input("driving_step_v", "V", true),
        ],
        components: vec![
            descriptor(
                "equilibrium",
                "equilibrium.nernst",
                ComponentRole::Equilibrium,
                vec!["primary_concentration", "temperature"],
                vec![],
                vec!["standard_potential_v", "ion_charge"],
                Some("equilibrium"),
                "EQ-CAL-001 adapter",
                BTreeMap::from([("activity_model".into(), "ideal".into())]),
            ),
            descriptor(
                "fast_mode",
                "transport.first_order_relaxation",
                ComponentRole::Transport,
                vec!["driving_step_v"],
                vec!["fast_mode_v"],
                vec!["fast_tau_s", "fast_gain"],
                Some("fast_mode"),
                "EQ-TR-001 adapter",
                BTreeMap::new(),
            ),
            descriptor(
                "slow_mode",
                "transport.first_order_relaxation",
                ComponentRole::Transport,
                vec!["driving_step_v"],
                vec!["slow_mode_v"],
                vec!["slow_tau_s", "slow_gain"],
                Some("slow_mode"),
                "EQ-TR-001 adapter",
                BTreeMap::new(),
            ),
            descriptor(
                "baseline_drift",
                "disturbance.linear_drift",
                ComponentRole::ExternalDisturbance,
                vec![],
                vec![],
                vec!["baseline_drift_v_per_s"],
                Some("baseline_drift"),
                "linear covariate offset",
                BTreeMap::new(),
            ),
            descriptor(
                "observation_noise",
                "disturbance.stochastic_observation_noise",
                ComponentRole::ObservationNoise,
                vec![],
                vec![],
                vec!["observation_noise_std_v"],
                None,
                "zero-mean observation-noise declaration",
                BTreeMap::new(),
            ),
        ],
    }
}

fn state(id: &str) -> StateSpec {
    StateSpec {
        id: id.into(),
        name: id.replace('_', " "),
        description:
            "Phenomenological reduced-order voltage state; no physical mechanism is implied.".into(),
        unit: "V".into(),
        transformation: super::StateTransformation::Identity,
        initialization_source: super::StateInitializationSource::DeclaredDefault,
        lower_bound: -10.0,
        upper_bound: 10.0,
        initial_value: 0.0,
        source: "default reduced-order model".into(),
        process_equation_version: 1,
        observability_requirements: vec![
            "State observability must be assessed before interpretation.".into(),
        ],
        validity_domain: "reduced-order voltage mode".into(),
        uncertainty_representation: super::UncertaintyRepresentation::NotSpecified,
    }
}
fn parameter(
    id: &str,
    unit: &str,
    lower_bound: f64,
    upper_bound: f64,
    default_value: f64,
) -> ParameterSpec {
    ParameterSpec {
        id: id.into(),
        name: id.replace('_', " "),
        description: "Reduced-order model parameter; domain-specific calibration is required."
            .into(),
        unit: unit.into(),
        lower_bound,
        upper_bound,
        default_value,
        uncertainty: 0.0,
        source: "default reduced-order model".into(),
        equation_version: 1,
        identifiability_requirements: vec![
            "Structural and practical identifiability must be assessed before interpretation."
                .into(),
        ],
        value_source: super::ParameterValueSource::Fitted,
        validity_domain: "must be calibrated for the experimental domain".into(),
    }
}
fn input(id: &str, unit: &str, required: bool) -> InputSpec {
    InputSpec {
        id: id.into(),
        unit: unit.into(),
        required,
        source: "experiment input".into(),
        validity_domain: "declared input unit and finite value".into(),
    }
}
#[allow(clippy::too_many_arguments)]
fn descriptor(
    id: &str,
    kind: &str,
    role: ComponentRole,
    input_ids: Vec<&str>,
    state_ids: Vec<&str>,
    parameter_ids: Vec<&str>,
    owner: Option<&str>,
    equation: &str,
    metadata: BTreeMap<String, String>,
) -> ComponentDescriptor {
    ComponentDescriptor {
        id: id.into(),
        kind: kind.into(),
        role,
        interpretation_status: super::InterpretationStatus::Phenomenological,
        depends_on: Vec::new(),
        required_inputs: input_ids
            .into_iter()
            .map(|id| InputRequirement {
                id: id.into(),
                unit: match id {
                    "temperature" => "K",
                    "primary_concentration" => "mol/L",
                    _ => "V",
                }
                .into(),
            })
            .collect(),
        state_ids: state_ids.into_iter().map(str::to_string).collect(),
        parameter_ids: parameter_ids.into_iter().map(str::to_string).collect(),
        output_unit: owner.map(|_| "V".into()),
        voltage_contribution_owner: owner.map(str::to_string),
        composition_rule: owner.map(|_| "additive_voltage".into()),
        source: "Phase 03 built-in component".into(),
        validity_domain: "reduced-order component; no mechanism confirmation implied".into(),
        equation: equation.into(),
        equation_version: 1,
        assumptions: vec!["Phenomenological component terms are not mechanism labels.".into()],
        evidence_requirements: vec![EvidenceRequirement {
            hypothesis_id: format!("{id}-mechanism"),
            proposed_mechanism_label: "unassigned".into(),
            independent_evidence_types: vec!["independent experiment".into()],
            minimum_independent_observations: 2,
            validity_domain: "declared model domain".into(),
            alternatives_to_consider: vec!["other reduced-order explanations".into()],
            required_uncertainty_statement:
                "parameter uncertainty is required before interpretation".into(),
        }],
        metadata,
    }
}
