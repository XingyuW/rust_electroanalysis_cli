//! Phase-B mechanism-evidence configuration.  These types deliberately sit
//! beside (rather than inside) the frozen A1 evidence contract.

use crate::evidence::EvidenceSourceClass;
use serde::{Deserialize, Serialize};

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
    pub mixed_state: MixedStateConfig,
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
    #[serde(default)]
    pub evidence_requirements: Vec<EvidenceRequirementBinding>,
    #[serde(default)]
    pub pair_requirements: Vec<EvidencePairRequirement>,
    #[serde(default)]
    pub critical_requirement_ids: Vec<EvidenceRequirementId>,
    #[serde(default)]
    pub timescale_gate: Option<TimescaleGate>,
    #[serde(default)]
    pub amplitude_gates: Vec<AmplitudeGate>,
    #[serde(default)]
    pub repeatability_gates: Vec<RepeatabilityGate>,
    #[serde(default)]
    pub identifiability_bindings: Vec<IdentifiabilityBinding>,
    #[serde(default)]
    pub validation_applicability: ValidationApplicability,
    #[serde(default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    Assessed,
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
    pub source_class_selectors: Vec<EvidenceSourceClass>,
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
    All,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentifiabilityInputBinding {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    #[serde(default = "all_selection")]
    pub selection: IdentifiabilityInputSelection,
}
fn all_selection() -> IdentifiabilityInputSelection {
    IdentifiabilityInputSelection::All
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixedStateConfig {
    pub classification_source: String,
}
