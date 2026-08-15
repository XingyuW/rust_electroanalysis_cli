use crate::{
    evidence::{EvidenceBundle, EvidenceId, EvidenceIndependence, EvidencePairKey},
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
    h: &MechanismHypothesisDefinition,
    b: &IdentifiabilityBinding,
    e: &EligibleHypothesisEvidence,
    bundle: &EvidenceBundle,
    ind: &[crate::evidence::EvidenceIndependenceAssessment],
    c: &IdentifiabilityGateConfig,
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
    if c.algorithm != "bound_inputs_v1" {
        return Err(IdentifiabilityAssessmentError::InvalidThreshold);
    }
    let IdentifiabilityInputSelection::ExactPair {
        pair_requirement_id,
    } = &b.input.selection
    else {
        return Ok(unsupported(&b.requirement_id));
    };
    let Some(pair) = h
        .pair_requirements
        .iter()
        .find(|pair| &pair.requirement_id == pair_requirement_id)
    else {
        return Ok(unsupported(&b.requirement_id));
    };
    // The declared pair is the sole legal metric input.  Equal values from a
    // different eligible row cannot substitute for the configured evidence.
    if b.input.requirement_ids.len() != 2
        || b.input.requirement_ids[0] != pair.left_requirement_id
        || b.input.requirement_ids[1] != pair.right_requirement_id
    {
        return Ok(unsupported(&b.requirement_id));
    }
    let ids = match (
        e.requirements
            .iter()
            .find(|row| row.requirement_id == pair.left_requirement_id),
        e.requirements
            .iter()
            .find(|row| row.requirement_id == pair.right_requirement_id),
    ) {
        (Some(left), Some(right))
            if left.support_evidence_ids.len() == 1 && right.support_evidence_ids.len() == 1 =>
        {
            vec![
                left.support_evidence_ids[0].clone(),
                right.support_evidence_ids[0].clone(),
            ]
        }
        _ => Vec::new(),
    };
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
        evidence_ids: ids.clone(),
        reasons: vec![],
    };
    if values.len() != 2 {
        out.reasons
            .push(IdentifiabilityAssessmentReasonCode::UnsupportedMetricInput);
        return Ok(out);
    }
    let pair_key = EvidencePairKey::canonical(ids[0].clone(), ids[1].clone())
        .expect("two distinct exact-pair evidence IDs");
    if !ind.iter().any(|assessment| {
        assessment.pair == pair_key
            && assessment.classification == EvidenceIndependence::Independent
    }) {
        out.reasons
            .push(IdentifiabilityAssessmentReasonCode::MissingInput);
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

fn unsupported(requirement_id: &str) -> IdentifiabilityAssessment {
    IdentifiabilityAssessment {
        requirement_id: requirement_id.into(),
        status: IdentifiabilityAssessmentStatus::NotAssessed,
        metric_value: None,
        evidence_ids: vec![],
        reasons: vec![IdentifiabilityAssessmentReasonCode::UnsupportedMetricInput],
    }
}
