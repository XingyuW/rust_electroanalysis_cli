//! Transparent comparison and evidence classification.

use crate::results::{
    CharacteristicTimescale, EvidenceLevel, MechanismWarning, ResolvedMechanismConfig,
    TimescaleComparison,
};
use crate::{
    evidence::{
        EvidenceArtifactSource, EvidenceAvailability, EvidenceBundle, EvidenceDirection,
        EvidenceExperimentScope, EvidenceId, EvidenceRecord, EvidenceTarget, EvidenceUnitDimension,
        EvidenceValidity, validate_ucum_unit,
    },
    mechanism::{
        config::*,
        evaluation::MechanismAssessmentError,
        preparation::PhaseBEvidencePreparation,
        temporal::{
            TemporalJoinAssessment, TemporalJoinOutcome, TemporalJoinRequest,
            evaluate_temporal_join,
        },
    },
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct BoundEvidencePair {
    pub pair_requirement_id: EvidenceRequirementId,
    /// Structural candidates only.  Scientific selection happens after the
    /// generic eligibility stage has evaluated the candidates.
    pub left_candidate_evidence_ids: Vec<EvidenceId>,
    pub right_candidate_evidence_ids: Vec<EvidenceId>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BoundRequirementEvidence {
    pub requirement_id: EvidenceRequirementId,
    pub candidate_evidence_ids: Vec<EvidenceId>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BoundHypothesisEvidence {
    pub hypothesis_id: MechanismHypothesisId,
    pub requirements: Vec<BoundRequirementEvidence>,
    pub pair_bindings: Vec<BoundEvidencePair>,
    pub role_bindings: Vec<MechanismEvidenceRoleBinding>,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EligibleRequirementEvidence {
    pub requirement_id: EvidenceRequirementId,
    pub support_evidence_ids: Vec<EvidenceId>,
    pub contradictory_evidence_ids: Vec<EvidenceId>,
    pub temporally_ineligible_evidence_ids: Vec<EvidenceId>,
    pub indeterminate_evidence_ids: Vec<EvidenceId>,
    pub temporal_assessments: Vec<TemporalJoinAssessment>,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EligibleHypothesisEvidence {
    pub hypothesis_id: MechanismHypothesisId,
    pub requirements: Vec<EligibleRequirementEvidence>,
}
#[derive(Debug, Error)]
pub enum EvidenceBindingError {
    #[error("unresolved pair binding {0}")]
    UnresolvedPairBinding(String),
    #[error("role/stage mismatch {0}")]
    RoleStageMismatch(String),
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RequirementContradictionSummary {
    pub requirement_id: EvidenceRequirementId,
    pub evidence_ids: Vec<EvidenceId>,
    pub contradiction_count: usize,
    pub strong_critical_count: usize,
}

pub fn bind_hypothesis_evidence(
    h: &MechanismHypothesisDefinition,
    p: &PhaseBEvidencePreparation,
) -> Result<BoundHypothesisEvidence, EvidenceBindingError> {
    let mut requirements = Vec::new();
    for r in &h.evidence_requirements {
        let mut ids=p.bundle.records.iter().filter(|e|matches!((&r.target_selector,&e.target),(EvidenceTargetSelector::ExactComponent{value},EvidenceTarget::ModelComponent(component)) if value==&component.0)).filter(|e|r.source_class_selectors.iter().any(|selector| crate::evidence::EvidenceSourceClass::from(*selector) == e.source_class)&&e.source.field_path==r.source_field_path).map(|e|e.evidence_id.clone()).collect::<Vec<_>>();
        ids.sort();
        requirements.push(BoundRequirementEvidence {
            requirement_id: r.requirement_id.clone(),
            candidate_evidence_ids: ids,
        });
    }
    for role in &h.role_bindings {
        let stage = h
            .evidence_requirements
            .iter()
            .find(|r| r.requirement_id == role.requirement_id)
            .map(|r| r.stage);
        let valid = matches!(
            (role.role, stage),
            (
                MechanismEvidenceRole::Support,
                Some(
                    EvidenceRequirementStage::Support
                        | EvidenceRequirementStage::SupportAndValidation
                )
            ) | (
                MechanismEvidenceRole::Validation,
                Some(
                    EvidenceRequirementStage::Validation
                        | EvidenceRequirementStage::SupportAndValidation
                )
            ) | (
                MechanismEvidenceRole::Calibration | MechanismEvidenceRole::Training,
                _
            )
        );
        if !valid {
            return Err(EvidenceBindingError::RoleStageMismatch(
                role.requirement_id.clone(),
            ));
        }
    }
    let mut pairs = Vec::new();
    for pair in &h.pair_requirements {
        let left = requirements
            .iter()
            .find(|r| r.requirement_id == pair.left_requirement_id)
            .map(|r| r.candidate_evidence_ids.clone())
            .unwrap_or_default();
        let right = requirements
            .iter()
            .find(|r| r.requirement_id == pair.right_requirement_id)
            .map(|r| r.candidate_evidence_ids.clone())
            .unwrap_or_default();
        pairs.push(BoundEvidencePair {
            pair_requirement_id: pair.requirement_id.clone(),
            left_candidate_evidence_ids: left,
            right_candidate_evidence_ids: right,
        });
    }
    pairs.sort_by(|a, b| a.pair_requirement_id.cmp(&b.pair_requirement_id));
    Ok(BoundHypothesisEvidence {
        hypothesis_id: h.hypothesis_id.clone(),
        requirements,
        pair_bindings: pairs,
        role_bindings: h.role_bindings.clone(),
    })
}
fn requirement<'a>(
    h: &'a MechanismHypothesisDefinition,
    id: &str,
) -> Option<&'a EvidenceRequirementBinding> {
    h.evidence_requirements
        .iter()
        .find(|r| r.requirement_id == id)
}
pub fn evaluate_hypothesis_evidence_eligibility(
    h: &MechanismHypothesisDefinition,
    b: &BoundHypothesisEvidence,
    p: &PhaseBEvidencePreparation,
    c: &MechanismEvidenceConfig,
) -> Result<EligibleHypothesisEvidence, MechanismAssessmentError> {
    let mut out = Vec::new();
    for bound in &b.requirements {
        let Some(rule) = requirement(h, &bound.requirement_id) else {
            continue;
        };
        let mut result = EligibleRequirementEvidence {
            requirement_id: bound.requirement_id.clone(),
            support_evidence_ids: vec![],
            contradictory_evidence_ids: vec![],
            temporally_ineligible_evidence_ids: vec![],
            indeterminate_evidence_ids: vec![],
            temporal_assessments: vec![],
        };
        if rule.gate == RequirementGate::NotApplicable {
            out.push(result);
            continue;
        };
        let temporal_pair = h
            .pair_requirements
            .iter()
            .find(|x| {
                x.left_requirement_id == rule.requirement_id
                    || x.right_requirement_id == rule.requirement_id
            })
            .filter(|x| matches!(x.temporal, TemporalRequirement::Required { .. }));
        let temporal_assessments = temporal_pair.map(|pair| {
            let bound_pair = b
                .pair_bindings
                .iter()
                .find(|x| x.pair_requirement_id == pair.requirement_id)
                .ok_or_else(|| MechanismAssessmentError::TemporalAssessmentMissing {
                    requirement_id: pair.requirement_id.clone(),
                })?;
            let mode = match pair.temporal {
                TemporalRequirement::Required { join_mode } => join_mode,
                _ => unreachable!(),
            };
            bound_pair
                .left_candidate_evidence_ids
                .iter()
                .flat_map(|left| {
                    bound_pair
                        .right_candidate_evidence_ids
                        .iter()
                        .map(move |right| (left, right))
                })
                .map(|(left, right)| {
                    evaluate_temporal_join(
                        &TemporalJoinRequest {
                            requirement_id: pair.requirement_id.clone(),
                            left_evidence_id: left.clone(),
                            right_evidence_id: right.clone(),
                            mode,
                        },
                        &p.bundle,
                        &p.temporal_metadata,
                        &c.temporal,
                    )
                    .map_err(|e| MechanismAssessmentError::Invalid(e.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()
        });
        let temporal_assessments = temporal_assessments.transpose()?.unwrap_or_default();
        result.temporal_assessments = temporal_assessments.clone();
        for id in &bound.candidate_evidence_ids {
            let role_authorized = b.role_bindings.iter().any(|binding| {
                binding.requirement_id == rule.requirement_id
                    && binding.evidence_id == *id
                    && matches!(
                        (rule.stage, binding.role),
                        (
                            EvidenceRequirementStage::Support,
                            MechanismEvidenceRole::Support
                        ) | (
                            EvidenceRequirementStage::Validation,
                            MechanismEvidenceRole::Validation
                        ) | (
                            EvidenceRequirementStage::SupportAndValidation,
                            MechanismEvidenceRole::Support
                        ) | (
                            EvidenceRequirementStage::SupportAndValidation,
                            MechanismEvidenceRole::Validation
                        )
                    )
            });
            if !role_authorized {
                continue;
            }
            let Some(record) = p.bundle.records.iter().find(|x| &x.evidence_id == id) else {
                continue;
            };
            let validity_matches = match rule.validity_requirement {
                EvidenceValidityRequirement::Valid => record.validity == EvidenceValidity::Valid,
                EvidenceValidityRequirement::ValidOrNotAssessed => matches!(
                    record.validity,
                    EvidenceValidity::Valid | EvidenceValidity::NotAssessed
                ),
            };
            if record.availability != EvidenceAvailability::Available || !validity_matches {
                continue;
            }
            if !scope_matches_analysis(record, p) {
                continue;
            }
            if !quantity_matches_requirement(record, rule) {
                continue;
            }
            let temporal_for_candidate = temporal_assessments
                .iter()
                .filter(|assessment| {
                    assessment.left_evidence_id == *id || assessment.right_evidence_id == *id
                })
                .collect::<Vec<_>>();
            if !temporal_for_candidate.is_empty()
                && !temporal_for_candidate
                    .iter()
                    .any(|assessment| assessment.outcome == TemporalJoinOutcome::Eligible)
            {
                if temporal_for_candidate
                    .iter()
                    .any(|assessment| assessment.outcome == TemporalJoinOutcome::Indeterminate)
                {
                    result.indeterminate_evidence_ids.push(id.clone());
                } else {
                    result.temporally_ineligible_evidence_ids.push(id.clone());
                }
                continue;
            }
            match (rule.expected_direction, record.direction) {
                (_, EvidenceDirection::Contradicts) => {
                    result.contradictory_evidence_ids.push(id.clone())
                }
                (RequiredEvidenceDirection::Contradicts, _) => {}
                (
                    RequiredEvidenceDirection::CandidatePresence,
                    EvidenceDirection::Supports | EvidenceDirection::Neutral,
                )
                | (RequiredEvidenceDirection::Supports, EvidenceDirection::Supports) => {
                    result.support_evidence_ids.push(id.clone())
                }
                _ => {}
            }
        }
        result.support_evidence_ids.sort();
        result.contradictory_evidence_ids.sort();
        out.push(result)
    }
    out.sort_by(|a, b| a.requirement_id.cmp(&b.requirement_id));
    Ok(EligibleHypothesisEvidence {
        hypothesis_id: h.hypothesis_id.clone(),
        requirements: out,
    })
}

/// Stage 7 record-level scope defense.  The EIS artifact establishes the
/// current Phase-B analysis scope after runner-level source validation.  A
/// candidate must independently prove compatibility: experiment scope lives
/// on the record, while sensor/channel scope is resolved from its typed A1
/// source artifact.  Unknown or legacy provenance never becomes a compatible
/// substitute for a known analysis scope.
fn scope_matches_analysis(
    record: &EvidenceRecord,
    preparation: &PhaseBEvidencePreparation,
) -> bool {
    experiment_scope_matches(
        &preparation.analysis_scope.experiment_scope,
        &record.experiment_scope,
    ) && source_scope_matches_analysis(record, preparation)
}

fn experiment_scope_matches(
    expected: &EvidenceExperimentScope,
    actual: &EvidenceExperimentScope,
) -> bool {
    matches!(
        (expected, actual),
        (
            EvidenceExperimentScope::Single {
                experiment_id: expected,
                ..
            },
            EvidenceExperimentScope::Single {
                experiment_id: actual,
                ..
            }
        ) if expected == actual
    )
}

fn source_scope_matches_analysis(
    record: &EvidenceRecord,
    preparation: &PhaseBEvidencePreparation,
) -> bool {
    let EvidenceArtifactSource::Known { artifact_id, .. } = &record.source.artifact else {
        return false;
    };
    let Some(source) = preparation
        .bundle
        .lineage_catalog
        .artifacts
        .get(artifact_id)
    else {
        return false;
    };
    scope_key_compatible(
        &preparation.analysis_scope.sensor_scope,
        &source.identity.sensor_scope,
    ) && scope_key_compatible(
        &preparation.analysis_scope.channel_scope,
        &source.identity.channel_scope,
    )
}

fn scope_key_compatible(
    expected: &crate::domain::ScopeKey,
    actual: &crate::domain::ScopeKey,
) -> bool {
    match (expected, actual) {
        (
            crate::domain::ScopeKey::Specific(expected),
            crate::domain::ScopeKey::Specific(actual),
        ) => expected == actual,
        (crate::domain::ScopeKey::Specific(_), crate::domain::ScopeKey::All)
        | (crate::domain::ScopeKey::All, crate::domain::ScopeKey::Specific(_))
        | (crate::domain::ScopeKey::All, crate::domain::ScopeKey::All)
        | (crate::domain::ScopeKey::Unspecified, crate::domain::ScopeKey::Unspecified) => true,
        _ => false,
    }
}

fn quantity_matches_requirement(
    record: &crate::evidence::EvidenceRecord,
    requirement: &EvidenceRequirementBinding,
) -> bool {
    let Some(quantity) = &record.quantity else {
        return false;
    };
    let expected_dimension = match requirement.quantity_semantic {
        PhaseBQuantitySemantic::TimeConstant => EvidenceUnitDimension::Time,
        PhaseBQuantitySemantic::Potential => EvidenceUnitDimension::Potential,
        PhaseBQuantitySemantic::Dimensionless => EvidenceUnitDimension::Dimensionless,
        PhaseBQuantitySemantic::Other => return false,
    };
    validate_ucum_unit(&quantity.unit).ok() == Some(expected_dimension)
        && validate_ucum_unit(&requirement.required_unit).ok() == Some(expected_dimension)
        && quantity.value.is_finite()
}
pub fn evaluate_direct_contradictions(
    h: &MechanismHypothesisDefinition,
    e: &EligibleHypothesisEvidence,
    bundle: &EvidenceBundle,
) -> Result<Vec<RequirementContradictionSummary>, MechanismAssessmentError> {
    let mut out = vec![];
    for r in &e.requirements {
        if r.contradictory_evidence_ids.is_empty() {
            continue;
        }
        let mut ids = r.contradictory_evidence_ids.clone();
        ids.sort();
        ids.dedup();
        let strong = ids
            .iter()
            .filter(|id| {
                bundle
                    .records
                    .iter()
                    .find(|x| &x.evidence_id == *id)
                    .is_some_and(|x| {
                        h.critical_requirement_ids.contains(&r.requirement_id)
                            && x.strength == crate::evidence::EvidenceStrength::Strong
                    })
            })
            .count();
        out.push(RequirementContradictionSummary {
            requirement_id: r.requirement_id.clone(),
            contradiction_count: ids.len(),
            evidence_ids: ids,
            strong_critical_count: strong,
        })
    }
    Ok(out)
}

pub fn compare_timescales(
    record_id: &str,
    eis: &CharacteristicTimescale,
    transient: &CharacteristicTimescale,
    config: &ResolvedMechanismConfig,
) -> TimescaleComparison {
    let mut supporting = Vec::new();
    let mut contradictory = Vec::new();
    let mut assumptions = vec!["numerical timescale compatibility is treated as statistical association, not mechanism proof".to_string()];
    let alternatives = vec!["different processes can have similar characteristic timescales".to_string(), "model misspecification or unresolved frequency/observation windows can produce apparent agreement".to_string()];
    let mut warnings = Vec::new();
    let valid = eis.value_s.is_finite()
        && transient.value_s.is_finite()
        && eis.value_s > 0.0
        && transient.value_s > 0.0;
    let (ratio, log_distance, relative) = if valid {
        let ratio = (eis.value_s / transient.value_s).max(transient.value_s / eis.value_s);
        (
            Some(ratio),
            Some((eis.value_s.log10() - transient.value_s.log10()).abs()),
            Some(
                (eis.value_s - transient.value_s).abs() / ((eis.value_s + transient.value_s) / 2.0),
            ),
        )
    } else {
        warnings.push(MechanismWarning {
            kind: "nonpositive_timescale".to_string(),
            message: "comparison requires finite positive timescales".to_string(),
        });
        (None, None, None)
    };
    let overlap = eis
        .confidence_interval_s
        .zip(transient.confidence_interval_s)
        .map(|(a, b)| a.0 <= b.1 && b.0 <= a.1);
    let probability = compatibility_probability(
        eis,
        transient,
        config.compatibility_ratio_lower,
        config.compatibility_ratio_upper,
        config.monte_carlo_samples,
        config.seed,
    );
    let level = match (ratio, log_distance) {
        (Some(r), Some(d)) if r <= config.ratio_strong && d <= config.log_distance_strong => {
            supporting.push(format!(
                "ratio {r:.4} and log10 distance {d:.4} meet the strong numerical thresholds"
            ));
            EvidenceLevel::Strong
        }
        (Some(r), Some(d)) if r <= config.ratio_moderate && d <= config.log_distance_moderate => {
            supporting.push(format!(
                "ratio {r:.4} and log10 distance {d:.4} meet the moderate numerical thresholds"
            ));
            EvidenceLevel::Moderate
        }
        (Some(r), Some(d)) if r <= config.ratio_weak && d <= config.log_distance_weak => {
            supporting.push(format!(
                "ratio {r:.4} and log10 distance {d:.4} indicate weak temporal compatibility"
            ));
            EvidenceLevel::Weak
        }
        (Some(r), Some(d)) => {
            contradictory.push(format!(
                "ratio {r:.4} and log10 distance {d:.4} exceed configured compatibility thresholds"
            ));
            EvidenceLevel::Contradictory
        }
        _ => EvidenceLevel::Insufficient,
    };
    if overlap == Some(true) {
        supporting
            .push("confidence intervals overlap; this is not a formal hypothesis test".to_string());
    } else if overlap == Some(false) {
        contradictory.push("confidence intervals do not overlap".to_string());
    }
    if eis.validity != crate::results::TimescaleValidity::Valid {
        warnings.push(MechanismWarning {
            kind: "eis_timescale_warning".to_string(),
            message: "EIS timescale carries derivation or identifiability warnings".to_string(),
        });
    }
    if transient.validity != crate::results::TimescaleValidity::Valid {
        warnings.push(MechanismWarning {
            kind: "transient_timescale_warning".to_string(),
            message: "transient timescale carries fit or observation-window warnings".to_string(),
        });
    }
    if probability.is_none() {
        assumptions.push("compatibility probability unavailable because uncertainty intervals/covariance were unavailable".to_string());
    }
    TimescaleComparison {
        comparison_id: format!(
            "{record_id}:{}:{}",
            eis.timescale_id, transient.timescale_id
        ),
        record_id: record_id.to_string(),
        eis_timescale_id: eis.timescale_id.clone(),
        transient_timescale_id: transient.timescale_id.clone(),
        ratio,
        log10_distance: log_distance,
        symmetric_relative_difference: relative,
        confidence_interval_overlap: overlap,
        compatibility_probability: probability,
        evidence_level: level,
        supporting_evidence: supporting,
        contradictory_evidence: contradictory,
        assumptions,
        alternative_explanations: alternatives,
        warnings,
    }
}

fn compatibility_probability(
    eis: &CharacteristicTimescale,
    transient: &CharacteristicTimescale,
    lower: f64,
    upper: f64,
    samples: usize,
    seed: u64,
) -> Option<f64> {
    let eis_se = eis.standard_error_s?;
    let transient_se = transient.standard_error_s?;
    if samples == 0 || !eis_se.is_finite() || !transient_se.is_finite() {
        return None;
    }
    let mut state = seed.max(1);
    let mut compatible = 0usize;
    for _ in 0..samples {
        let x = eis.value_s + eis_se * standard_normal(&mut state);
        let y = transient.value_s + transient_se * standard_normal(&mut state);
        if x > 0.0 && y > 0.0 {
            let ratio = x / y;
            if ratio >= lower && ratio <= upper {
                compatible += 1;
            }
        }
    }
    Some(compatible as f64 / samples as f64)
}

fn standard_normal(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    let u1 = ((*state >> 11) as f64) / ((1u64 << 53) as f64);
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    let u2 = ((*state >> 11) as f64) / ((1u64 << 53) as f64);
    (-2.0 * u1.max(f64::MIN_POSITIVE).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}
