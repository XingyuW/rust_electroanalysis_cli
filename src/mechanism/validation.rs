use crate::{
    evidence::{EvidenceBundle, EvidenceId},
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
    #[serde(default)]
    pub required_conditions: Vec<ValidationCondition>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationCondition {
    pub condition_id: String,
    pub requirement_id: EvidenceRequirementId,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationProtocolStatus {
    Satisfied,
    Failed,
    NotAssessed,
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
                protocol_id: String::new(),
                status: ValidationProtocolStatus::NotAssessed,
                evidence_ids: vec![],
                acquisition_family_ids: vec![],
                passed_condition_ids: vec![],
                reasons: vec![],
            })
        };
    };
    let mut ids = vec![];
    for role in roles
        .iter()
        .filter(|x| x.role == MechanismEvidenceRole::Validation)
    {
        if e.requirements
            .iter()
            .find(|r| r.requirement_id == role.requirement_id)
            .is_some_and(|r| r.support_evidence_ids.contains(&role.evidence_id))
        {
            ids.push(role.evidence_id.clone())
        }
    }
    ids.sort();
    ids.dedup();
    let mut families = vec![];
    for id in &ids {
        if let Some(record) = bundle.records.iter().find(|r| &r.evidence_id == id) {
            for x in &record.lineage_artifact_ids {
                families.push(x.0.clone())
            }
        }
    }
    families.sort();
    families.dedup();
    let mut passed = p
        .required_conditions
        .iter()
        .filter(|c| {
            e.requirements
                .iter()
                .find(|r| r.requirement_id == c.requirement_id)
                .is_some_and(|r| !r.support_evidence_ids.is_empty())
        })
        .map(|c| c.condition_id.clone())
        .collect::<Vec<_>>();
    passed.sort();
    let good = passed.len() == p.required_conditions.len()
        && families.len() >= p.minimum_acquisition_families;
    Ok(ValidationAssessment {
        protocol_id: p.protocol_id.clone(),
        status: if good {
            ValidationProtocolStatus::Satisfied
        } else {
            ValidationProtocolStatus::Failed
        },
        evidence_ids: ids,
        acquisition_family_ids: families,
        passed_condition_ids: passed,
        reasons: if good {
            vec![ValidationReasonCode::Passed]
        } else {
            vec![ValidationReasonCode::MissingEvidence]
        },
    })
}
