use crate::{
    domain::ArtifactAcquisitionFamilies,
    evidence::{
        EvidenceArtifactSource, EvidenceBundle, EvidenceExperimentScope, EvidenceId,
        EvidenceIndependence, EvidencePairKey,
    },
    mechanism::{config::*, evidence::EligibleHypothesisEvidence},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationProtocol {
    pub protocol_id: String,
    pub version: String,
    pub minimum_acquisition_families: usize,
    pub required_conditions: Vec<ValidationCondition>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationCondition {
    pub condition_id: String,
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub experiment_scope: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationProtocolStatus {
    Satisfied,
    NotSatisfied,
    NotAssessed,
    NotApplicable,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationReasonCode {
    MissingEvidence,
    UnknownAcquisitionFamily,
    Passed,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationAssessment {
    pub protocol_id: String,
    pub status: ValidationProtocolStatus,
    pub evidence_ids: Vec<EvidenceId>,
    pub acquisition_family_ids: Vec<String>,
    pub passed_condition_ids: Vec<String>,
    pub reasons: Vec<ValidationReasonCode>,
}
#[derive(Debug, Error)]
pub enum ValidationAssessmentError {
    #[error("required validation protocol is absent")]
    MissingProtocol,
}
pub fn evaluate_validation_protocol(
    h: &MechanismHypothesisDefinition,
    e: &EligibleHypothesisEvidence,
    roles: &[MechanismEvidenceRoleBinding],
    bundle: &EvidenceBundle,
    p: Option<&ValidationProtocol>,
) -> Result<ValidationAssessment, ValidationAssessmentError> {
    let Some(p) = p else {
        return if h.validation_applicability == ValidationApplicability::Required {
            Err(ValidationAssessmentError::MissingProtocol)
        } else {
            Ok(ValidationAssessment {
                protocol_id: "not-applicable".into(),
                status: ValidationProtocolStatus::NotApplicable,
                evidence_ids: vec![],
                acquisition_family_ids: vec![],
                passed_condition_ids: vec![],
                reasons: vec![],
            })
        };
    };
    // Eligibility has already applied the concrete record constraints.  Only
    // explicit Validation bindings may enter this set: source kind, field
    // name, or artifact type never implies a validation role.
    let mut candidate_ids = vec![];
    for role in roles
        .iter()
        .filter(|x| x.role == MechanismEvidenceRole::Validation)
    {
        if e.requirements
            .iter()
            .find(|r| r.requirement_id == role.requirement_id)
            .is_some_and(|r| r.support_evidence_ids.contains(&role.evidence_id))
        {
            candidate_ids.push(role.evidence_id.clone())
        }
    }
    candidate_ids.sort();
    candidate_ids.dedup();

    let record_for = |id: &EvidenceId| {
        bundle
            .records
            .iter()
            .find(|record| &record.evidence_id == id)
    };
    let families_for = |id: &EvidenceId| {
        record_for(id)
            .and_then(|record| match &record.source.artifact {
                EvidenceArtifactSource::Known { artifact_id, .. } => bundle
                    .lineage_catalog
                    .artifacts
                    .get(artifact_id)
                    .and_then(|node| match &node.identity.acquisition_families {
                        ArtifactAcquisitionFamilies::Known(values) => Some(
                            values
                                .iter()
                                .map(|family| family.0.clone())
                                .collect::<Vec<_>>(),
                        ),
                        ArtifactAcquisitionFamilies::Unknown => None,
                    }),
                EvidenceArtifactSource::LegacyUnknown { .. } => None,
            })
            .unwrap_or_default()
    };

    // A validation family must be distinct from every explicit non-validation
    // role candidate.  This prevents calibration/training leakage even when a
    // later gate would otherwise make the validation row look eligible.
    let mut excluded_families = Vec::new();
    for role in roles
        .iter()
        .filter(|role| role.role != MechanismEvidenceRole::Validation)
    {
        excluded_families.extend(families_for(&role.evidence_id));
    }
    excluded_families.sort();
    excluded_families.dedup();

    // Select a deterministic independent subset.  A known acquisition family
    // is necessary but not sufficient: its selected record must also be A1
    // independent of every previously selected record.
    let mut ids: Vec<EvidenceId> = Vec::new();
    let mut families: Vec<String> = Vec::new();
    for id in &candidate_ids {
        let record_families = families_for(id);
        if record_families.is_empty()
            || record_families
                .iter()
                .any(|family| excluded_families.contains(family))
            || record_families
                .iter()
                .any(|family| families.contains(family))
        {
            continue;
        }
        let independent = ids.iter().all(|selected| {
            EvidencePairKey::canonical(selected.clone(), id.clone())
                .ok()
                .and_then(|pair| {
                    bundle
                        .independence_assessments
                        .iter()
                        .find(|assessment| assessment.pair == pair)
                })
                .is_some_and(|assessment| {
                    assessment.classification == EvidenceIndependence::Independent
                })
        });
        if independent {
            ids.push(id.clone());
            families.extend(record_families);
        }
    }
    ids.sort();
    families.sort();
    families.dedup();
    let mut passed = p
        .required_conditions
        .iter()
        .filter(|c| {
            c.requirement_ids.iter().all(|requirement_id| {
                roles
                    .iter()
                    .filter(|role| {
                        role.role == MechanismEvidenceRole::Validation
                            && role.requirement_id == *requirement_id
                    })
                    .any(|role| {
                        ids.contains(&role.evidence_id)
                            && record_for(&role.evidence_id).is_some_and(|record| {
                                matches!(
                                    &record.experiment_scope,
                                    EvidenceExperimentScope::Single { experiment_id, .. }
                                        if experiment_id.0 == c.experiment_scope
                                )
                            })
                    })
            })
        })
        .map(|c| c.condition_id.clone())
        .collect::<Vec<_>>();
    passed.sort();
    let good = passed.len() == p.required_conditions.len()
        && families.len() >= p.minimum_acquisition_families;
    let reasons = if good {
        vec![ValidationReasonCode::Passed]
    } else if candidate_ids.len() > ids.len() {
        vec![ValidationReasonCode::UnknownAcquisitionFamily]
    } else {
        vec![ValidationReasonCode::MissingEvidence]
    };
    Ok(ValidationAssessment {
        protocol_id: p.protocol_id.clone(),
        status: if good {
            ValidationProtocolStatus::Satisfied
        } else {
            ValidationProtocolStatus::NotSatisfied
        },
        evidence_ids: ids,
        acquisition_family_ids: families,
        passed_condition_ids: passed,
        reasons,
    })
}
