use crate::{
    evidence::{EvidenceBundle, EvidenceId},
    mechanism::{config::*, evidence::EligibleRequirementEvidence},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepeatabilityStatus {
    Satisfied,
    Failed,
    NotAssessed,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepeatabilityAssessment {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub status: RepeatabilityStatus,
    pub evidence_ids: Vec<EvidenceId>,
    pub sample_standard_deviation_ln_tau: Option<f64>,
}
#[derive(Debug, Error)]
pub enum RepeatabilityAssessmentError {
    #[error("invalid repeatability gate")]
    InvalidGate,
}
pub fn evaluate_repeatability_requirement(
    _h: &MechanismHypothesisDefinition,
    g: &RepeatabilityGate,
    eligible: &[&EligibleRequirementEvidence],
    bundle: &EvidenceBundle,
    _c: &RepeatabilityEvidenceConfig,
) -> Result<RepeatabilityAssessment, RepeatabilityAssessmentError> {
    if !g.maximum_sample_standard_deviation_ln_tau.is_finite() {
        return Err(RepeatabilityAssessmentError::InvalidGate);
    }
    let mut ids = eligible
        .iter()
        .flat_map(|r| r.support_evidence_ids.clone())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    let values = ids
        .iter()
        .filter_map(|id| {
            bundle
                .records
                .iter()
                .find(|r| &r.evidence_id == id)?
                .quantity
                .as_ref()?
                .value
                .is_sign_positive()
                .then(|| {
                    bundle
                        .records
                        .iter()
                        .find(|r| &r.evidence_id == id)
                        .unwrap()
                        .quantity
                        .as_ref()
                        .unwrap()
                        .value
                        .ln()
                })
        })
        .collect::<Vec<_>>();
    let mut out = RepeatabilityAssessment {
        requirement_ids: g.requirement_ids.clone(),
        status: RepeatabilityStatus::NotAssessed,
        evidence_ids: ids,
        sample_standard_deviation_ln_tau: None,
    };
    if values.len() < g.minimum_independent_families || values.len() < 2 {
        return Ok(out);
    }
    let mean = values.iter().sum::<f64>() / (values.len() as f64);
    let sd = (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() as f64 - 1.))
        .sqrt();
    out.sample_standard_deviation_ln_tau = Some(sd);
    out.status = if sd <= g.maximum_sample_standard_deviation_ln_tau {
        RepeatabilityStatus::Satisfied
    } else {
        RepeatabilityStatus::Failed
    };
    Ok(out)
}
