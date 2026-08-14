use crate::{
    domain::ArtifactAcquisitionFamilies,
    evidence::{EvidenceBundle, EvidenceId, EvidenceIndependence, EvidencePairKey},
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
    let mut candidate_ids = eligible
        .iter()
        .flat_map(|r| r.support_evidence_ids.clone())
        .collect::<Vec<_>>();
    candidate_ids.sort();
    candidate_ids.dedup();
    let first_scope = candidate_ids
        .iter()
        .find_map(|id| {
            bundle
                .records
                .iter()
                .find(|record| &record.evidence_id == id)
        })
        .map(|record| record.experiment_scope.clone());
    let candidates = candidate_ids
        .iter()
        .filter_map(|id| {
            let record = bundle
                .records
                .iter()
                .find(|record| &record.evidence_id == id)?;
            let quantity = record.quantity.as_ref()?;
            let families = match &record.source.artifact {
                crate::evidence::EvidenceArtifactSource::Known { artifact_id, .. } => match &bundle
                    .lineage_catalog
                    .artifacts
                    .get(artifact_id)?
                    .identity
                    .acquisition_families
                {
                    ArtifactAcquisitionFamilies::Known(families) if !families.is_empty() => {
                        families
                            .iter()
                            .map(|family| family.0.clone())
                            .collect::<Vec<_>>()
                    }
                    _ => return None,
                },
                crate::evidence::EvidenceArtifactSource::LegacyUnknown { .. } => return None,
            };
            (quantity.unit == "s"
                && quantity.value.is_finite()
                && quantity.value > 0.0
                && first_scope
                    .as_ref()
                    .is_some_and(|scope| &record.experiment_scope == scope))
            .then(|| (id.clone(), quantity.value.ln(), families))
        })
        .collect::<Vec<_>>();
    let mut selected = Vec::new();
    for size in (2..=candidates.len()).rev() {
        let mut trial = Vec::new();
        if choose_independent_subset(&candidates, size, 0, &mut trial, bundle) {
            selected = trial;
            break;
        }
    }
    let ids = selected
        .iter()
        .map(|(id, _, _)| id.clone())
        .collect::<Vec<_>>();
    let values = selected
        .iter()
        .map(|(_, value, _)| *value)
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

fn choose_independent_subset(
    candidates: &[(EvidenceId, f64, Vec<String>)],
    wanted: usize,
    start: usize,
    selected: &mut Vec<(EvidenceId, f64, Vec<String>)>,
    bundle: &EvidenceBundle,
) -> bool {
    if selected.len() == wanted {
        return true;
    }
    for index in start..candidates.len() {
        let candidate = &candidates[index];
        let family_disjoint = selected
            .iter()
            .all(|(_, _, families)| families.iter().all(|family| !candidate.2.contains(family)));
        let independent = selected.iter().all(|(id, _, _)| {
            EvidencePairKey::canonical(id.clone(), candidate.0.clone())
                .ok()
                .and_then(|pair| bundle.lookup_independence(&pair))
                .is_some_and(|assessment| {
                    assessment.classification == EvidenceIndependence::Independent
                })
        });
        if family_disjoint && independent {
            selected.push(candidate.clone());
            if choose_independent_subset(candidates, wanted, index + 1, selected, bundle) {
                return true;
            }
            selected.pop();
        }
    }
    false
}
