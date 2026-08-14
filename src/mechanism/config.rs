//! Phase-B mechanism-evidence configuration.  These types deliberately sit
//! beside (rather than inside) the frozen A1 evidence contract.

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::Path};
use thiserror::Error;

pub type MechanismHypothesisId = String;
pub type EvidenceRequirementId = String;
pub type IdentifiabilityRequirementId = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MechanismEvidenceConfig {
    pub schema_version: u32,
    pub timescale: TimescaleEvidenceConfig,
    pub amplitude: AmplitudeEvidenceConfig,
    pub repeatability: RepeatabilityEvidenceConfig,
    pub temporal: crate::mechanism::temporal::TemporalJoinConfig,
    pub identifiability: IdentifiabilityGateConfig,
    pub promotion: HypothesisPromotionConfig,
    #[serde(default)]
    pub validation: Option<crate::mechanism::validation::ValidationProtocol>,
    pub hypotheses: Vec<MechanismHypothesisDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MechanismHypothesisDefinition {
    pub hypothesis_id: MechanismHypothesisId,
    pub display_name: String,
    pub target_components: Vec<String>,
    pub evidence_requirements: Vec<EvidenceRequirementBinding>,
    pub pair_requirements: Vec<EvidencePairRequirement>,
    pub critical_requirement_ids: Vec<EvidenceRequirementId>,
    pub timescale_gate: Option<TimescaleGate>,
    pub amplitude_gates: Vec<AmplitudeGate>,
    pub repeatability_gates: Vec<RepeatabilityGate>,
    pub identifiability_bindings: Vec<IdentifiabilityBinding>,
    pub validation_applicability: ValidationApplicability,
    pub role_bindings: Vec<MechanismEvidenceRoleBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RequirementGate {
    #[default]
    Required,
    NotApplicable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRequirementStage {
    #[default]
    Support,
    Validation,
    SupportAndValidation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MechanismEvidenceRole {
    Support,
    Validation,
    Calibration,
    Training,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseBQuantitySemantic {
    TimeConstant,
    Potential,
    Dimensionless,
    Other,
}
/// Phase-B config owns its stable snake-case wire vocabulary without changing
/// the frozen A1 `EvidenceSourceClass` serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseBEvidenceSourceClass {
    Observed,
    ModelDerived,
    ProducerAssessment,
    ExternalReference,
}
impl From<PhaseBEvidenceSourceClass> for crate::evidence::EvidenceSourceClass {
    fn from(value: PhaseBEvidenceSourceClass) -> Self {
        match value {
            PhaseBEvidenceSourceClass::Observed => Self::Observed,
            PhaseBEvidenceSourceClass::ModelDerived => Self::ModelDerived,
            PhaseBEvidenceSourceClass::ProducerAssessment => Self::ProducerAssessment,
            PhaseBEvidenceSourceClass::ExternalReference => Self::ExternalReference,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredEvidenceDirection {
    CandidatePresence,
    Supports,
    Contradicts,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceValidityRequirement {
    Valid,
    ValidOrNotAssessed,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum EvidenceTargetSelector {
    ExactComponent { value: String },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRequirementBinding {
    pub requirement_id: EvidenceRequirementId,
    pub target_selector: EvidenceTargetSelector,
    pub source_class_selectors: Vec<PhaseBEvidenceSourceClass>,
    pub source_field_path: String,
    pub quantity_semantic: PhaseBQuantitySemantic,
    pub required_unit: String,
    pub expected_direction: RequiredEvidenceDirection,
    pub validity_requirement: EvidenceValidityRequirement,
    pub gate: RequirementGate,
    pub stage: EvidenceRequirementStage,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TemporalRequirement {
    NotApplicable,
    Required { join_mode: TemporalJoinMode },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalJoinMode {
    PointPoint,
    PointWindow,
    WindowPoint,
    WindowWindow,
    EventEvent,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidencePairRequirement {
    pub requirement_id: EvidenceRequirementId,
    pub left_requirement_id: EvidenceRequirementId,
    pub right_requirement_id: EvidenceRequirementId,
    pub temporal: TemporalRequirement,
    pub gate: RequirementGate,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MechanismEvidenceRoleBinding {
    pub hypothesis_id: MechanismHypothesisId,
    pub requirement_id: EvidenceRequirementId,
    pub evidence_id: crate::evidence::EvidenceId,
    pub role: MechanismEvidenceRole,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimescaleEvidenceConfig {
    pub algorithm: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimescaleGate {
    pub pair_requirement_id: EvidenceRequirementId,
    pub maximum_log_distance: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmplitudeEvidenceConfig {
    pub algorithm: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmplitudeThreshold {
    pub value: f64,
    pub unit: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedEffect {
    Increase,
    Decrease,
    SameSign,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmplitudeGate {
    pub predicted_requirement_id: EvidenceRequirementId,
    pub observed_requirement_id: EvidenceRequirementId,
    pub expected_effect: ExpectedEffect,
    pub maximum_relative_error: f64,
    pub floor: AmplitudeThreshold,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepeatabilityEvidenceConfig {
    pub algorithm: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepeatabilityGate {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub maximum_sample_standard_deviation_ln_tau: f64,
    pub minimum_independent_families: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentifiabilityGateConfig {
    pub algorithm: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum IdentifiabilityInputSelection {
    ExactPair {
        pair_requirement_id: EvidenceRequirementId,
    },
    AllEligible,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentifiabilityInputBinding {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub selection: IdentifiabilityInputSelection,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifiabilityKind {
    ModeSeparation,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentifiabilityBinding {
    pub requirement_id: IdentifiabilityRequirementId,
    pub gate: RequirementGate,
    pub kind: IdentifiabilityKind,
    pub threshold: f64,
    pub input: IdentifiabilityInputBinding,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValidationApplicability {
    #[default]
    NotApplicable,
    Required,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypothesisPromotionConfig {
    pub minimum_independent_support: usize,
}
#[derive(Debug, Error)]
pub enum MechanismEvidenceConfigError {
    #[error("could not read mechanism-evidence config {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid mechanism-evidence TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("invalid mechanism-evidence config: {0}")]
    Invalid(String),
}

/// The only Phase-B configuration entry point.  Deserialization is deliberately
/// followed by semantic validation so an accepted TOML document is always safe
/// for the runner to consume.
pub fn load_mechanism_evidence_config(
    path: &Path,
) -> Result<MechanismEvidenceConfig, MechanismEvidenceConfigError> {
    let text = fs::read_to_string(path).map_err(|source| MechanismEvidenceConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;
    let config = toml::from_str::<MechanismEvidenceConfig>(&text)?;
    validate_mechanism_evidence_config(&config)?;
    Ok(config)
}

pub fn validate_mechanism_evidence_config(
    config: &MechanismEvidenceConfig,
) -> Result<(), MechanismEvidenceConfigError> {
    if config.schema_version != 1 {
        return Err(MechanismEvidenceConfigError::Invalid(
            "schema_version must be 1".into(),
        ));
    }
    if config.timescale.algorithm != "log_ratio_v1"
        || config.amplitude.algorithm != "signed_relative_error_v1"
        || config.repeatability.algorithm != "independent_ln_tau_sample_sd_v1"
        || config.identifiability.algorithm != "bound_inputs_v1"
    {
        return Err(MechanismEvidenceConfigError::Invalid(
            "unsupported Phase-B algorithm".into(),
        ));
    }
    let temporal = &config.temporal;
    if !temporal.point_tolerance_s.is_finite()
        || temporal.point_tolerance_s < 0.0
        || !temporal.minimum_classified_fraction.is_finite()
        || !(0.0..=1.0).contains(&temporal.minimum_classified_fraction)
        || !temporal.minimum_equilibrium_fraction.is_finite()
        || !(0.0..=1.0).contains(&temporal.minimum_equilibrium_fraction)
    {
        return Err(MechanismEvidenceConfigError::Invalid(
            "invalid temporal tolerance or fraction".into(),
        ));
    }
    if let crate::mechanism::temporal::MixedStatePolicy::MinimumSteadyFraction {
        minimum_fraction,
        ..
    } = temporal.mixed_state_policy
        && (!minimum_fraction.is_finite() || !(0.0..=1.0).contains(&minimum_fraction))
    {
        return Err(MechanismEvidenceConfigError::Invalid(
            "mixed_state_policy.minimum_fraction must be in [0, 1]".into(),
        ));
    }
    if config.hypotheses.is_empty() {
        return Err(MechanismEvidenceConfigError::Invalid(
            "hypotheses is empty".into(),
        ));
    }
    let mut hypotheses = BTreeSet::new();
    for hypothesis in &config.hypotheses {
        if hypothesis.hypothesis_id.is_empty() || !hypotheses.insert(&hypothesis.hypothesis_id) {
            return Err(MechanismEvidenceConfigError::Invalid(
                "hypothesis IDs must be nonempty and unique".into(),
            ));
        }
        let requirements = hypothesis
            .evidence_requirements
            .iter()
            .map(|requirement| requirement.requirement_id.as_str())
            .collect::<BTreeSet<_>>();
        if requirements.len() != hypothesis.evidence_requirements.len()
            || requirements.iter().any(|id| id.is_empty())
        {
            return Err(MechanismEvidenceConfigError::Invalid(
                "requirement IDs must be nonempty and unique".into(),
            ));
        }
        for requirement in &hypothesis.evidence_requirements {
            if requirement.source_field_path.is_empty()
                || requirement.required_unit.is_empty()
                || crate::evidence::validate_ucum_unit(&requirement.required_unit).is_err()
            {
                return Err(MechanismEvidenceConfigError::Invalid(
                    "invalid required evidence unit or source field".into(),
                ));
            }
        }
        let pairs = hypothesis
            .pair_requirements
            .iter()
            .map(|pair| pair.requirement_id.as_str())
            .collect::<BTreeSet<_>>();
        if pairs.len() != hypothesis.pair_requirements.len()
            || hypothesis.pair_requirements.iter().any(|pair| {
                !requirements.contains(pair.left_requirement_id.as_str())
                    || !requirements.contains(pair.right_requirement_id.as_str())
                    || pair.left_requirement_id == pair.right_requirement_id
            })
        {
            return Err(MechanismEvidenceConfigError::Invalid(
                "invalid or duplicate pair requirement".into(),
            ));
        }
        if hypothesis
            .critical_requirement_ids
            .iter()
            .any(|id| !requirements.contains(id.as_str()))
        {
            return Err(MechanismEvidenceConfigError::Invalid(
                "critical requirement does not exist".into(),
            ));
        }
        if let Some(gate) = &hypothesis.timescale_gate
            && (!pairs.contains(gate.pair_requirement_id.as_str())
                || !gate.maximum_log_distance.is_finite()
                || gate.maximum_log_distance < 0.0)
        {
            return Err(MechanismEvidenceConfigError::Invalid(
                "invalid timescale gate".into(),
            ));
        }
        for gate in &hypothesis.amplitude_gates {
            if !requirements.contains(gate.predicted_requirement_id.as_str())
                || !requirements.contains(gate.observed_requirement_id.as_str())
                || !gate.maximum_relative_error.is_finite()
                || gate.maximum_relative_error < 0.0
                || !gate.floor.value.is_finite()
                || gate.floor.value <= 0.0
                || crate::evidence::validate_ucum_unit(&gate.floor.unit).is_err()
            {
                return Err(MechanismEvidenceConfigError::Invalid(
                    "invalid amplitude gate".into(),
                ));
            }
        }
        for gate in &hypothesis.repeatability_gates {
            if gate.requirement_ids.len() < 2
                || gate.minimum_independent_families < 2
                || !gate.maximum_sample_standard_deviation_ln_tau.is_finite()
                || gate.maximum_sample_standard_deviation_ln_tau < 0.0
                || gate
                    .requirement_ids
                    .iter()
                    .any(|id| !requirements.contains(id.as_str()))
            {
                return Err(MechanismEvidenceConfigError::Invalid(
                    "invalid repeatability gate".into(),
                ));
            }
        }
        for binding in &hypothesis.identifiability_bindings {
            if !binding.threshold.is_finite()
                || binding.threshold <= 0.0
                || binding.input.requirement_ids.len() != 2
                || binding
                    .input
                    .requirement_ids
                    .iter()
                    .any(|id| !requirements.contains(id.as_str()))
                || matches!(&binding.input.selection, IdentifiabilityInputSelection::ExactPair { pair_requirement_id } if !pairs.contains(pair_requirement_id.as_str()))
            {
                return Err(MechanismEvidenceConfigError::Invalid(
                    "invalid identifiability binding".into(),
                ));
            }
        }
        let mut roles = BTreeSet::new();
        for role in &hypothesis.role_bindings {
            if role.hypothesis_id != hypothesis.hypothesis_id
                || !requirements.contains(role.requirement_id.as_str())
                || role.evidence_id.0.is_empty()
                || !roles.insert((
                    role.requirement_id.as_str(),
                    role.evidence_id.0.as_str(),
                    role.role,
                ))
            {
                return Err(MechanismEvidenceConfigError::Invalid(
                    "invalid or duplicate role binding".into(),
                ));
            }
        }
    }
    if let Some(validation) = &config.validation
        && (validation.protocol_id.is_empty()
            || validation.version.is_empty()
            || validation.minimum_acquisition_families < 2)
    {
        return Err(MechanismEvidenceConfigError::Invalid(
            "invalid validation protocol".into(),
        ));
    }
    Ok(())
}
