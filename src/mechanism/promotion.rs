use crate::{
    evidence::EvidenceId,
    mechanism::{
        amplitude::AmplitudeAssessment,
        config::*,
        evaluation::MechanismAssessmentError,
        evidence::{EligibleHypothesisEvidence, RequirementContradictionSummary},
        identifiability::IdentifiabilityAssessment,
        repeatability::RepeatabilityAssessment,
        validation::{ValidationAssessment, ValidationProtocolStatus},
    },
    model::InterpretationStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisEvidenceLevel {
    NotAssessed,
    Hypothesized,
    ExperimentallySupported,
    ValidatedForDomain,
    Contradicted,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseBHypothesisReasonCode {
    MissingRequiredEvidence,
    CriticalContradiction,
    TimescaleSatisfied,
    AmplitudeSatisfied,
    RepeatabilitySatisfied,
    IdentifiabilitySatisfied,
    ValidationSatisfied,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentInterpretationReasonCode {
    NoPromotion,
    HypothesisEvidence,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInterpretationAssessment {
    pub component_id: String,
    pub prior_status: InterpretationStatus,
    pub assessment_target: Option<InterpretationStatus>,
    pub resulting_status: InterpretationStatus,
    pub supporting_hypothesis_id: MechanismHypothesisId,
    pub evidence_ids: Vec<EvidenceId>,
    pub reasons: Vec<ComponentInterpretationReasonCode>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct HypothesisGateAssessments {
    pub contradiction_summaries: Vec<RequirementContradictionSummary>,
    pub timescale_assessments: Vec<crate::mechanism::timescale::TimescaleAssessment>,
    pub amplitude_assessments: Vec<AmplitudeAssessment>,
    pub repeatability_assessments: Vec<RepeatabilityAssessment>,
    pub identifiability_assessments: Vec<IdentifiabilityAssessment>,
    pub validation_assessment: Option<ValidationAssessment>,
}
#[derive(Debug, Error)]
pub enum ComponentAssessmentError {
    #[error("component assessment: {0}")]
    Invalid(String),
}
pub fn assess_hypothesis(
    h: &MechanismHypothesisDefinition,
    e: &EligibleHypothesisEvidence,
    g: &HypothesisGateAssessments,
    c: &MechanismEvidenceConfig,
) -> Result<crate::results::PhaseBHypothesisAssessment, MechanismAssessmentError> {
    let mut reasons = vec![];
    let support = e
        .requirements
        .iter()
        .filter(|r| !r.support_evidence_ids.is_empty())
        .count();
    let contradicted = g
        .contradiction_summaries
        .iter()
        .any(|x| x.strong_critical_count > 0);
    let mut level = if support == 0 {
        HypothesisEvidenceLevel::NotAssessed
    } else {
        HypothesisEvidenceLevel::Hypothesized
    };
    if contradicted {
        level = HypothesisEvidenceLevel::Contradicted;
        reasons.push(PhaseBHypothesisReasonCode::CriticalContradiction)
    } else if support >= c.promotion.minimum_independent_support {
        level = HypothesisEvidenceLevel::ExperimentallySupported
    };
    if matches!(
        g.validation_assessment.as_ref().map(|x| &x.status),
        Some(ValidationProtocolStatus::Satisfied)
    ) && level == HypothesisEvidenceLevel::ExperimentallySupported
    {
        level = HypothesisEvidenceLevel::ValidatedForDomain;
        reasons.push(PhaseBHypothesisReasonCode::ValidationSatisfied)
    };
    Ok(crate::results::PhaseBHypothesisAssessment {
        hypothesis_id: h.hypothesis_id.clone(),
        evidence_level: level,
        temporal_join_assessments: e
            .requirements
            .iter()
            .flat_map(|x| x.temporal_assessments.clone())
            .collect(),
        timescale_assessments: g.timescale_assessments.clone(),
        amplitude_assessments: g.amplitude_assessments.clone(),
        repeatability_assessments: g.repeatability_assessments.clone(),
        identifiability_assessments: g.identifiability_assessments.clone(),
        contradiction_summaries: g.contradiction_summaries.clone(),
        reason_codes: reasons,
        component_assessments: vec![],
        validation_status: g
            .validation_assessment
            .as_ref()
            .map(|x| x.status.clone())
            .unwrap_or(ValidationProtocolStatus::NotAssessed),
        history: vec![],
    })
}
fn rank(x: InterpretationStatus) -> u8 {
    match x {
        InterpretationStatus::Phenomenological => 0,
        InterpretationStatus::Hypothesized => 1,
        InterpretationStatus::ExperimentallySupported => 2,
        InterpretationStatus::ValidatedForDomain => 3,
    }
}
pub fn assess_components(
    h: &MechanismHypothesisDefinition,
    a: &crate::results::PhaseBHypothesisAssessment,
    prior: &BTreeMap<String, InterpretationStatus>,
) -> Result<Vec<ComponentInterpretationAssessment>, ComponentAssessmentError> {
    let target = match a.evidence_level {
        HypothesisEvidenceLevel::Hypothesized => Some(InterpretationStatus::Hypothesized),
        HypothesisEvidenceLevel::ExperimentallySupported => {
            Some(InterpretationStatus::ExperimentallySupported)
        }
        HypothesisEvidenceLevel::ValidatedForDomain => {
            Some(InterpretationStatus::ValidatedForDomain)
        }
        _ => None,
    };
    let ids: Vec<crate::evidence::EvidenceId> = a
        .temporal_join_assessments
        .iter()
        .flat_map(|x| [x.left_evidence_id.clone(), x.right_evidence_id.clone()])
        .collect();
    Ok(h.target_components
        .iter()
        .map(|id| {
            let p = *prior
                .get(id)
                .unwrap_or(&InterpretationStatus::Phenomenological);
            let result = target.filter(|t| rank(*t) > rank(p)).unwrap_or(p);
            ComponentInterpretationAssessment {
                component_id: id.clone(),
                prior_status: p,
                assessment_target: target,
                resulting_status: result,
                supporting_hypothesis_id: h.hypothesis_id.clone(),
                evidence_ids: ids.clone(),
                reasons: if target.is_some() {
                    vec![ComponentInterpretationReasonCode::HypothesisEvidence]
                } else {
                    vec![ComponentInterpretationReasonCode::NoPromotion]
                },
            }
        })
        .collect())
}
