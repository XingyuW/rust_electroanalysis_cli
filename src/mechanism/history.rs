//! Deterministic Phase-B assessment/history identities using RFC-8785 JCS.
use crate::{
    evidence::EvidenceId,
    mechanism::{
        evaluation::MechanismAssessmentError,
        promotion::{
            ComponentInterpretationAssessment, HypothesisEvidenceLevel, HypothesisGateAssessments,
        },
        validation::ValidationProtocolStatus,
    },
    results::PhaseBHypothesisAssessment,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HypothesisHistoryEntry {
    pub history_id: String,
    pub hypothesis_id: String,
    pub prior_level: HypothesisEvidenceLevel,
    pub new_level: HypothesisEvidenceLevel,
    pub assessment_target: Option<crate::model::InterpretationStatus>,
    pub assessment_index: u64,
    pub reason_codes: Vec<crate::mechanism::promotion::PhaseBHypothesisReasonCode>,
    pub source_evidence_ids: Vec<EvidenceId>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HypothesisAssessmentHashView {
    pub hypothesis_id: String,
    pub evidence_level: HypothesisEvidenceLevel,
    pub reason_codes: Vec<crate::mechanism::promotion::PhaseBHypothesisReasonCode>,
    pub temporal_join_assessments: Vec<crate::mechanism::temporal::TemporalJoinAssessment>,
    pub timescale_assessments: Vec<crate::mechanism::timescale::TimescaleAssessment>,
    pub amplitude_assessments: Vec<crate::mechanism::amplitude::AmplitudeAssessment>,
    pub repeatability_assessments: Vec<crate::mechanism::repeatability::RepeatabilityAssessment>,
    pub identifiability_assessments:
        Vec<crate::mechanism::identifiability::IdentifiabilityAssessment>,
    pub contradiction_summaries: Vec<crate::mechanism::evidence::RequirementContradictionSummary>,
    pub validation_assessment: Option<crate::mechanism::validation::ValidationAssessment>,
    pub component_assessments: Vec<ComponentInterpretationAssessment>,
    pub source_evidence_ids: Vec<EvidenceId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssessmentHash(pub String);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HypothesisHistoryIdView {
    pub hypothesis_id: String,
    pub prior_level: HypothesisEvidenceLevel,
    pub new_level: HypothesisEvidenceLevel,
    pub assessment_hash: String,
}
#[derive(Debug, Error)]
pub enum HypothesisAssessmentHashError {
    #[error("non-finite hashed float at {path}")]
    NonFiniteHashedFloat { path: String },
    #[error("source evidence ids mismatch")]
    SourceEvidenceIdsMismatch,
    #[error("validation assessment status mismatch")]
    ValidationAssessmentStatusMismatch,
    #[error("JCS serialization: {detail}")]
    JcsSerialization { detail: String },
}
fn sorted<T: Ord>(mut x: Vec<T>) -> Vec<T> {
    x.sort();
    x.dedup();
    x
}
pub fn build_hypothesis_assessment_hash_view(
    a: &PhaseBHypothesisAssessment,
    g: &HypothesisGateAssessments,
    components: &[ComponentInterpretationAssessment],
    source: &[EvidenceId],
) -> Result<HypothesisAssessmentHashView, HypothesisAssessmentHashError> {
    if a.validation_status
        != g.validation_assessment
            .as_ref()
            .map(|x| x.status.clone())
            .unwrap_or(ValidationProtocolStatus::NotAssessed)
    {
        return Err(HypothesisAssessmentHashError::ValidationAssessmentStatusMismatch);
    };
    let mut used = a
        .temporal_join_assessments
        .iter()
        .flat_map(|x| [x.left_evidence_id.clone(), x.right_evidence_id.clone()])
        .collect::<Vec<_>>();
    used.extend(
        g.contradiction_summaries
            .iter()
            .flat_map(|x| x.evidence_ids.clone()),
    );
    used.extend(
        g.timescale_assessments
            .iter()
            .flat_map(|x| x.evidence_ids.clone()),
    );
    used.extend(g.amplitude_assessments.iter().flat_map(|x| {
        [
            x.predicted_evidence_id.clone(),
            x.observed_evidence_id.clone(),
        ]
        .into_iter()
        .flatten()
    }));
    used.extend(
        g.repeatability_assessments
            .iter()
            .flat_map(|x| x.evidence_ids.clone()),
    );
    used.extend(
        g.identifiability_assessments
            .iter()
            .flat_map(|x| x.evidence_ids.clone()),
    );
    if let Some(v) = &g.validation_assessment {
        used.extend(v.evidence_ids.clone())
    };
    used = sorted(used);
    if used != sorted(source.to_vec()) {
        return Err(HypothesisAssessmentHashError::SourceEvidenceIdsMismatch);
    };
    let mut view = HypothesisAssessmentHashView {
        hypothesis_id: a.hypothesis_id.clone(),
        evidence_level: a.evidence_level.clone(),
        reason_codes: sorted(a.reason_codes.clone()),
        temporal_join_assessments: a.temporal_join_assessments.clone(),
        timescale_assessments: g.timescale_assessments.clone(),
        amplitude_assessments: g.amplitude_assessments.clone(),
        repeatability_assessments: g.repeatability_assessments.clone(),
        identifiability_assessments: g.identifiability_assessments.clone(),
        contradiction_summaries: g.contradiction_summaries.clone(),
        validation_assessment: g.validation_assessment.clone(),
        component_assessments: components.to_vec(),
        source_evidence_ids: used,
    };
    view.temporal_join_assessments.sort_by(|x, y| {
        (&x.requirement_id, &x.left_evidence_id, &x.right_evidence_id).cmp(&(
            &y.requirement_id,
            &y.left_evidence_id,
            &y.right_evidence_id,
        ))
    });
    view.timescale_assessments
        .sort_by(|x, y| x.pair_requirement_id.cmp(&y.pair_requirement_id));
    view.component_assessments
        .sort_by(|x, y| x.component_id.cmp(&y.component_id));
    Ok(view)
}
fn hash<T: Serialize>(v: &T) -> Result<String, HypothesisAssessmentHashError> {
    let bytes =
        serde_jcs::to_vec(v).map_err(|e| HypothesisAssessmentHashError::JcsSerialization {
            detail: e.to_string(),
        })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
pub fn compute_assessment_hash(
    v: &HypothesisAssessmentHashView,
) -> Result<AssessmentHash, HypothesisAssessmentHashError> {
    Ok(AssessmentHash(hash(v)?))
}
pub fn compute_history_id(
    v: &HypothesisHistoryIdView,
) -> Result<String, HypothesisAssessmentHashError> {
    hash(v)
}
pub fn update_hypothesis_history(
    previous: &[HypothesisHistoryEntry],
    a: &PhaseBHypothesisAssessment,
    g: &HypothesisGateAssessments,
    components: &[ComponentInterpretationAssessment],
    source: &[EvidenceId],
) -> Result<Vec<HypothesisHistoryEntry>, MechanismAssessmentError> {
    let view = build_hypothesis_assessment_hash_view(a, g, components, source)
        .map_err(|e| MechanismAssessmentError::Invalid(e.to_string()))?;
    let ah = compute_assessment_hash(&view)
        .map_err(|e| MechanismAssessmentError::Invalid(e.to_string()))?;
    let prior = previous
        .iter()
        .filter(|x| x.hypothesis_id == a.hypothesis_id)
        .max_by_key(|x| x.assessment_index)
        .map(|x| x.new_level.clone())
        .unwrap_or(HypothesisEvidenceLevel::NotAssessed);
    let id = compute_history_id(&HypothesisHistoryIdView {
        hypothesis_id: a.hypothesis_id.clone(),
        prior_level: prior.clone(),
        new_level: a.evidence_level.clone(),
        assessment_hash: ah.0,
    })
    .map_err(|e| MechanismAssessmentError::Invalid(e.to_string()))?;
    if previous.iter().any(|x| x.history_id == id) {
        return Ok(previous.to_vec());
    }
    let mut out = previous.to_vec();
    out.push(HypothesisHistoryEntry {
        history_id: id,
        hypothesis_id: a.hypothesis_id.clone(),
        prior_level: prior,
        new_level: a.evidence_level.clone(),
        assessment_target: components.first().and_then(|x| x.assessment_target),
        assessment_index: out
            .iter()
            .filter(|x| x.hypothesis_id == a.hypothesis_id)
            .map(|x| x.assessment_index)
            .max()
            .unwrap_or(0)
            + 1,
        reason_codes: sorted(a.reason_codes.clone()),
        source_evidence_ids: sorted(source.to_vec()),
    });
    out.sort_by(|x, y| {
        (&x.hypothesis_id, x.assessment_index).cmp(&(&y.hypothesis_id, y.assessment_index))
    });
    Ok(out)
}
