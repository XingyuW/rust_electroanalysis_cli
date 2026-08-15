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
        uncertainty_incomplete: false,
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

/// Version-1 scientifically neutral reduced-order composition.  This is kept
/// separate from `default_model_definition` so schema-v1 workflow fixtures
/// remain reproducible while callers can explicitly opt into the new model.
pub fn reduced_ism_v1_definition() -> ModelDefinition {
    ModelDefinition {
        schema_version: super::MODEL_DEFINITION_SCHEMA_VERSION,
        model_id: "reduced_ism_v1".into(),
        description: "Activity-first reduced-order ISM model with independent phenomenological dynamic modes.".into(),
        validity_domain: "Canonical activity inputs within declared calibration and temperature domains.".into(),
        uncertainty_incomplete: false,
        states: vec![
            state("dynamic_fast_potential_v"),
            state("dynamic_slow_potential_v"),
            state("reference_offset_v"),
        ],
        parameters: vec![
            parameter("standard_potential_v", "V", -2.0, 2.0, 0.0),
            parameter("ion_charge", "dimensionless", -4.0, 4.0, 1.0),
            parameter("dynamic_fast_tau_s", "s", 1e-6, 1e6, 2.0),
            parameter("dynamic_fast_gain_v_per_decade", "V", -10.0, 10.0, 0.02),
            parameter("dynamic_slow_tau_s", "s", 1e-6, 1e7, 35.0),
            parameter("dynamic_slow_gain_v_per_decade", "V", -10.0, 10.0, 0.01),
            parameter("observation_variance_v2", "V^2", 1e-18, 1.0, 1e-6),
        ],
        inputs: vec![
            input("target_activity", "activity", true),
            input("temperature", "K", true),
            input("delta_log10_activity", "activity", false),
            input("transduction_drive", "activity", false),
            input("conductivity", "S/m", false),
            input("flow", "m/s", false),
        ],
        components: vec![
            v1_descriptor(
                "activity_input",
                "activity.input",
                ComponentRole::Auxiliary,
                super::InterpretationStatus::ExperimentallySupported,
                vec!["target_activity"], vec![], vec![], None,
                super::ContributionSemantics::Auxiliary,
                "canonical activity input adapter", BTreeMap::new(),
                vec!["activity-domain coverage", "input provenance"],
            ),
            v1_descriptor(
                "equilibrium_nernst",
                "equilibrium.nernst",
                ComponentRole::Equilibrium,
                super::InterpretationStatus::ExperimentallySupported,
                vec!["target_activity", "temperature"], vec![],
                vec!["standard_potential_v", "ion_charge"], Some("equilibrium"),
                super::ContributionSemantics::AdditivePotential,
                "adapter to calibrated Nernst equation", BTreeMap::from([("activity_input_id".into(), "target_activity".into())]),
                vec!["calibration validation", "activity-domain coverage", "temperature-domain coverage"],
            ),
            v1_descriptor(
                "dynamic_fast", "dynamics.first_order", ComponentRole::Transport,
                super::InterpretationStatus::Phenomenological,
                vec!["delta_log10_activity"], vec!["dynamic_fast_potential_v"],
                vec!["dynamic_fast_tau_s", "dynamic_fast_gain_v_per_decade"], Some("dynamic_fast"),
                super::ContributionSemantics::AdditivePotential,
                "dx/dt=-x/tau; x+=gain*delta_log10_activity", BTreeMap::from([
                    ("peer_tau_s".into(), "35".into()),
                    ("separation_threshold".into(), "2".into()),
                ]),
                vec!["controlled concentration-step response", "repeatability", "observation-window coverage", "identifiability"],
            ),
            v1_descriptor(
                "dynamic_slow", "dynamics.first_order", ComponentRole::Transport,
                super::InterpretationStatus::Phenomenological,
                vec!["delta_log10_activity"], vec!["dynamic_slow_potential_v"],
                vec!["dynamic_slow_tau_s", "dynamic_slow_gain_v_per_decade"], Some("dynamic_slow"),
                super::ContributionSemantics::AdditivePotential,
                "dx/dt=-x/tau; x+=gain*delta_log10_activity", BTreeMap::new(),
                vec!["controlled concentration-step response", "repeatability", "observation-window coverage", "identifiability"],
            ),
            v1_descriptor(
                "reference_offset", "reference.offset", ComponentRole::Reference,
                super::InterpretationStatus::Phenomenological,
                vec![], vec!["reference_offset_v"], vec![], Some("reference"),
                super::ContributionSemantics::AdditivePotential,
                "db_ref/dt=0; E_reference=b_ref", BTreeMap::new(),
                vec!["constant-standard drift", "reference-control measurement", "common-mode channel evidence"],
            ),
            v1_descriptor(
                "observation_noise", "disturbance.observation_variance", ComponentRole::ObservationNoise,
                super::InterpretationStatus::Phenomenological,
                vec![], vec![], vec!["observation_variance_v2"], None,
                super::ContributionSemantics::ObservationVariance,
                "configured observation variance", BTreeMap::new(),
                vec!["repeat measurement noise characterization"],
            ),
        ],
    }
}

/// Explicit reduced-V1 variant used when estimation enables the optional
/// candidate-transduction input. The ordinary reduced definition remains
/// unchanged so inactive compiled scenarios retain their historical state
/// vector and artifact shape.
pub fn reduced_ism_v1_with_transduction_definition() -> ModelDefinition {
    let mut definition = reduced_ism_v1_definition();
    definition
        .states
        .push(state("transduction_candidate_potential_v"));
    definition.parameters.push(parameter(
        "transduction_candidate_tau_s",
        "s",
        1e-6,
        1e7,
        10.0,
    ));
    definition.parameters.push(parameter(
        "transduction_candidate_gain_v_per_decade",
        "V",
        -10.0,
        10.0,
        0.015,
    ));
    let reference_index = definition
        .components
        .iter()
        .position(|component| component.id == "reference_offset")
        .unwrap_or(definition.components.len());
    definition.components.insert(
        reference_index,
        v1_descriptor(
            "transduction_candidate",
            "transduction.first_order_candidate",
            ComponentRole::Transduction,
            super::InterpretationStatus::Hypothesized,
            vec!["transduction_drive"],
            vec!["transduction_candidate_potential_v"],
            vec![
                "transduction_candidate_tau_s",
                "transduction_candidate_gain_v_per_decade",
            ],
            Some("transduction_candidate"),
            super::ContributionSemantics::AdditivePotential,
            "dx/dt=-x/tau; x+=gain*transduction_drive",
            BTreeMap::new(),
            vec![
                "explicit transduction-drive event",
                "independent perturbation",
                "repeatability",
            ],
        ),
    );
    definition
}

#[allow(clippy::too_many_arguments)]
fn v1_descriptor(
    id: &str,
    kind: &str,
    role: ComponentRole,
    interpretation_status: super::InterpretationStatus,
    input_ids: Vec<&str>,
    state_ids: Vec<&str>,
    parameter_ids: Vec<&str>,
    owner: Option<&str>,
    semantics: super::ContributionSemantics,
    equation: &str,
    metadata: BTreeMap<String, String>,
    evidence_types: Vec<&str>,
) -> ComponentDescriptor {
    let state_ids = state_ids
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let parameter_ids = parameter_ids
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let observation_state_ids =
        if matches!(semantics, super::ContributionSemantics::AdditivePotential)
            && !state_ids.is_empty()
        {
            state_ids.clone()
        } else {
            Vec::new()
        };
    let observation_parameter_ids = match kind {
        // Charge is a fixed signed categorical choice in the constrained
        // equation, not a differentiable fitted scalar.
        "equilibrium.nernst" => vec!["standard_potential_v".into()],
        "disturbance.linear_covariate" => parameter_ids.clone(),
        _ => Vec::new(),
    };
    ComponentDescriptor {
        id: id.into(),
        kind: kind.into(),
        role,
        interpretation_status,
        depends_on: if id == "equilibrium_nernst" {
            vec!["activity_input".into()]
        } else {
            Vec::new()
        },
        required_inputs: input_ids
            .into_iter()
            .map(|id| InputRequirement {
                id: id.into(),
                unit: match id {
                    "target_activity" | "delta_log10_activity" | "transduction_drive" => "activity",
                    "temperature" => "K",
                    "conductivity" => "S/m",
                    "flow" => "m/s",
                    _ => "activity",
                }
                .into(),
            })
            .collect(),
        state_ids,
        parameter_ids,
        observation_state_ids,
        observation_parameter_ids,
        numerical_jacobian_supported: false,
        output_unit: match semantics {
            super::ContributionSemantics::AdditivePotential => Some("V".into()),
            super::ContributionSemantics::ObservationVariance => Some("V^2".into()),
            _ => None,
        },
        voltage_contribution_owner: owner.map(str::to_string),
        contribution_semantics: semantics,
        legacy_composition_rule: None,
        source: "Reduced-order ISM components V1".into(),
        validity_domain:
            "Declared reduced-order input and calibration domain; no mechanism diagnosis implied."
                .into(),
        equation: equation.into(),
        equation_version: 1,
        assumptions: vec![
            "Dynamic labels describe fitted timescales, not physical mechanisms.".into(),
        ],
        evidence_requirements: vec![EvidenceRequirement {
            hypothesis_id: format!("{id}-interpretation"),
            proposed_mechanism_label: "unassigned".into(),
            independent_evidence_types: evidence_types.into_iter().map(str::to_string).collect(),
            minimum_independent_observations: 2,
            validity_domain: "declared experimental domain".into(),
            alternatives_to_consider: vec!["alternative reduced-order explanations".into()],
            required_uncertainty_statement:
                "runtime covariance is required for complete prediction uncertainty".into(),
        }],
        applicability_constraints: Vec::new(),
        metadata,
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
        initial_uncertainty: super::UncertaintySpec::Deterministic,
    }
}
fn parameter(
    id: &str,
    unit: &str,
    lower_bound: f64,
    upper_bound: f64,
    default_value: f64,
) -> ParameterSpec {
    let (uncertainty, value_source, description, validity_domain) = match id {
        "ion_charge" => (
            super::UncertaintySpec::Deterministic,
            super::ParameterValueSource::Fixed,
            "Fixed analyte charge used by the configured theoretical equilibrium equation.",
            "declared nonzero integer analyte charge",
        ),
        "observation_noise_std_v" | "observation_variance_v2" => (
            super::UncertaintySpec::Deterministic,
            super::ParameterValueSource::ExternallySuppliedFixed,
            "Fixed configured observation-noise variance; it defines R rather than a voltage term.",
            "configured measurement-system noise domain",
        ),
        _ => (
            super::UncertaintySpec::Unknown {
                reason: "default value has no calibration covariance or explicit prior".into(),
            },
            super::ParameterValueSource::ExternallySupplied,
            "Externally supplied reduced-order value; calibration covariance or a prior is required for complete uncertainty.",
            "requires calibration or an explicit prior for the experimental domain",
        ),
    };
    ParameterSpec {
        id: id.into(),
        name: id.replace('_', " "),
        description: description.into(),
        unit: unit.into(),
        lower_bound,
        upper_bound,
        default_value,
        uncertainty,
        source: "default reduced-order model".into(),
        equation_version: 1,
        identifiability_requirements: vec![
            "Structural and practical identifiability must be assessed before interpretation."
                .into(),
        ],
        value_source,
        characteristic: if id == "ion_charge" {
            super::ParameterCharacteristic::DiscreteInteger
        } else {
            super::ParameterCharacteristic::Continuous
        },
        validity_domain: validity_domain.into(),
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
    let state_ids = state_ids
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let parameter_ids = parameter_ids
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let observation_state_ids = match kind {
        "transport.first_order_relaxation"
        | "transport.two_mode_relaxation"
        | "transport.stretched_relaxation"
        | "transport.partition_delay"
        | "transduction.solid_contact_rc_candidate"
        | "transduction.interfacial_polarization_candidate"
        | "disturbance.baseline_random_walk" => state_ids.clone(),
        _ => Vec::new(),
    };
    let observation_parameter_ids = match kind {
        "equilibrium.nernst"
        | "equilibrium.nicolsky_eisenman"
        | "transduction.ideal"
        | "disturbance.linear_drift"
        | "disturbance.temperature_covariate"
        | "disturbance.conductivity_covariate"
        | "disturbance.flow_covariate" => parameter_ids.clone(),
        _ => Vec::new(),
    };
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
        state_ids,
        parameter_ids,
        observation_state_ids,
        observation_parameter_ids,
        numerical_jacobian_supported: false,
        output_unit: if id == "observation_noise" {
            Some("V^2".into())
        } else {
            owner.map(|_| "V".into())
        },
        voltage_contribution_owner: owner.map(str::to_string),
        contribution_semantics: if id == "observation_noise" {
            super::ContributionSemantics::ObservationVariance
        } else if owner.is_some() {
            super::ContributionSemantics::AdditivePotential
        } else {
            super::ContributionSemantics::StateOnly
        },
        legacy_composition_rule: None,
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
        applicability_constraints: Vec::new(),
        metadata,
    }
}
