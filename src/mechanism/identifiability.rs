use crate::{
    evidence::{EvidenceBundle, EvidenceId},
    mechanism::{config::*, evidence::EligibleHypothesisEvidence},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifiabilityAssessmentStatus {
    Satisfied,
    Failed,
    NotAssessed,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifiabilityAssessmentReasonCode {
    MissingInput,
    ThresholdNotMet,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentifiabilityAssessment {
    pub requirement_id: IdentifiabilityRequirementId,
    pub status: IdentifiabilityAssessmentStatus,
    pub metric_value: Option<f64>,
    pub evidence_ids: Vec<EvidenceId>,
    pub reasons: Vec<IdentifiabilityAssessmentReasonCode>,
}
#[derive(Debug, Error)]
pub enum IdentifiabilityAssessmentError {
    #[error("invalid threshold")]
    InvalidThreshold,
}
pub fn evaluate_identifiability_binding(
    _h: &MechanismHypothesisDefinition,
    b: &IdentifiabilityBinding,
    e: &EligibleHypothesisEvidence,
    bundle: &EvidenceBundle,
    _ind: &[crate::evidence::EvidenceIndependenceAssessment],
    _c: &IdentifiabilityGateConfig,
) -> Result<IdentifiabilityAssessment, IdentifiabilityAssessmentError> {
    if !b.threshold.is_finite() {
        return Err(IdentifiabilityAssessmentError::InvalidThreshold);
    }
    let ids = b
        .input
        .requirement_ids
        .iter()
        .filter_map(|id| e.requirements.iter().find(|r| &r.requirement_id == id))
        .flat_map(|r| r.support_evidence_ids.clone())
        .collect::<Vec<_>>();
    let values = ids
        .iter()
        .filter_map(|id| {
            bundle
                .records
                .iter()
                .find(|r| &r.evidence_id == id)?
                .quantity
                .as_ref()
                .map(|q| q.value)
        })
        .collect::<Vec<_>>();
    let mut out = IdentifiabilityAssessment {
        requirement_id: b.requirement_id.clone(),
        status: IdentifiabilityAssessmentStatus::NotAssessed,
        metric_value: None,
        evidence_ids: ids,
        reasons: vec![],
    };
    if values.len() < 2 {
        out.reasons
            .push(IdentifiabilityAssessmentReasonCode::MissingInput);
        return Ok(out);
    }
    let metric = (values[0].ln() - values[1].ln()).abs();
    out.metric_value = Some(metric);
    out.status = if metric >= b.threshold {
        IdentifiabilityAssessmentStatus::Satisfied
    } else {
        out.reasons
            .push(IdentifiabilityAssessmentReasonCode::ThresholdNotMet);
        IdentifiabilityAssessmentStatus::Failed
    };
    Ok(out)
}
