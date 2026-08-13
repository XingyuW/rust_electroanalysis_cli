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
    NotSatisfied,
    NotAssessed,
    NotApplicable,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentifiabilityAssessmentReasonCode {
    MissingInput,
    ThresholdSatisfied,
    ThresholdNotSatisfied,
    UnsupportedMetricInput,
    NonFiniteInput,
    NotApplicableByDefinition,
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
    if !b.threshold.is_finite() || b.threshold <= 0.0 {
        return Err(IdentifiabilityAssessmentError::InvalidThreshold);
    }
    if b.gate == RequirementGate::NotApplicable {
        return Ok(IdentifiabilityAssessment {
            requirement_id: b.requirement_id.clone(),
            status: IdentifiabilityAssessmentStatus::NotApplicable,
            metric_value: None,
            evidence_ids: vec![],
            reasons: vec![IdentifiabilityAssessmentReasonCode::NotApplicableByDefinition],
        });
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
    if values.len() != 2
        || !matches!(
            b.input.selection,
            IdentifiabilityInputSelection::ExactPair { .. }
        )
    {
        out.reasons
            .push(IdentifiabilityAssessmentReasonCode::UnsupportedMetricInput);
        return Ok(out);
    }
    if values
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        out.reasons
            .push(IdentifiabilityAssessmentReasonCode::NonFiniteInput);
        return Ok(out);
    }
    let metric = values[0].max(values[1]) / values[0].min(values[1]);
    out.metric_value = Some(metric);
    out.status = if metric >= b.threshold {
        out.reasons
            .push(IdentifiabilityAssessmentReasonCode::ThresholdSatisfied);
        IdentifiabilityAssessmentStatus::Satisfied
    } else {
        out.reasons
            .push(IdentifiabilityAssessmentReasonCode::ThresholdNotSatisfied);
        IdentifiabilityAssessmentStatus::NotSatisfied
    };
    Ok(out)
}
