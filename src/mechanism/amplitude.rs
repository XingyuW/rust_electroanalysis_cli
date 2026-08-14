use crate::{
    evidence::{EvidenceBundle, EvidenceId},
    mechanism::{config::*, evidence::EligibleRequirementEvidence},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmplitudeStatus {
    Satisfied,
    Contradicted,
    Inconclusive,
    NotAssessed,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmplitudeReasonCode {
    DirectionMismatch,
    RelativeErrorExceeded,
    MissingEvidence,
    IncompatibleUnit,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmplitudeAssessment {
    pub predicted_requirement_id: EvidenceRequirementId,
    pub observed_requirement_id: EvidenceRequirementId,
    pub status: AmplitudeStatus,
    pub predicted_evidence_id: Option<EvidenceId>,
    pub observed_evidence_id: Option<EvidenceId>,
    pub threshold: AmplitudeThreshold,
    pub relative_error: Option<f64>,
    pub reasons: Vec<AmplitudeReasonCode>,
}
#[derive(Debug, Error)]
pub enum AmplitudeAssessmentError {
    #[error("invalid amplitude gate")]
    InvalidGate,
}
pub fn evaluate_amplitude_requirement(
    _h: &MechanismHypothesisDefinition,
    g: &AmplitudeGate,
    eligible: (&EligibleRequirementEvidence, &EligibleRequirementEvidence),
    bundle: &EvidenceBundle,
    _c: &AmplitudeEvidenceConfig,
) -> Result<AmplitudeAssessment, AmplitudeAssessmentError> {
    let (p, o) = (
        eligible.0.support_evidence_ids.first(),
        eligible.1.support_evidence_ids.first(),
    );
    let mut out = AmplitudeAssessment {
        predicted_requirement_id: g.predicted_requirement_id.clone(),
        observed_requirement_id: g.observed_requirement_id.clone(),
        status: AmplitudeStatus::NotAssessed,
        predicted_evidence_id: p.cloned(),
        observed_evidence_id: o.cloned(),
        threshold: g.floor.clone(),
        relative_error: None,
        reasons: vec![],
    };
    if !g.floor.value.is_finite()
        || g.floor.value <= 0.0
        || !g.maximum_relative_error.is_finite()
        || g.maximum_relative_error < 0.0
    {
        return Err(AmplitudeAssessmentError::InvalidGate);
    };
    let vals = p.zip(o).and_then(|(p, o)| {
        Some((
            bundle
                .records
                .iter()
                .find(|r| &r.evidence_id == p)?
                .quantity
                .as_ref()?,
            bundle
                .records
                .iter()
                .find(|r| &r.evidence_id == o)?
                .quantity
                .as_ref()?,
        ))
    });
    let Some((p, o)) = vals else {
        out.reasons.push(AmplitudeReasonCode::MissingEvidence);
        return Ok(out);
    };
    let Some(p) = convert_to_unit(p.value, &p.unit, &g.floor.unit) else {
        out.reasons.push(AmplitudeReasonCode::IncompatibleUnit);
        return Ok(out);
    };
    let Some(o) = convert_to_unit(o.value, &o.unit, &g.floor.unit) else {
        out.reasons.push(AmplitudeReasonCode::IncompatibleUnit);
        return Ok(out);
    };
    let d = o - p;
    let denominator = p.abs().max(o.abs()).max(g.floor.value);
    let r = d / denominator;
    out.relative_error = Some(r.abs());
    let direction = match g.expected_effect {
        ExpectedEffect::Increase => d > 0.,
        ExpectedEffect::Decrease => d < 0.,
        ExpectedEffect::SameSign => p * o > 0.,
    };
    if !direction {
        out.status = AmplitudeStatus::Contradicted;
        out.reasons.push(AmplitudeReasonCode::DirectionMismatch)
    } else if r.abs() <= g.maximum_relative_error {
        out.status = AmplitudeStatus::Satisfied
    } else {
        out.status = AmplitudeStatus::Inconclusive;
        out.reasons.push(AmplitudeReasonCode::RelativeErrorExceeded)
    };
    Ok(out)
}

/// Phase B's approved V1 conversion vocabulary.  We intentionally keep this
/// small rather than treating matching display strings as compatible units.
fn convert_to_unit(value: f64, from: &str, to: &str) -> Option<f64> {
    if !value.is_finite()
        || crate::evidence::validate_ucum_unit(from).is_err()
        || crate::evidence::validate_ucum_unit(to).is_err()
    {
        return None;
    }
    let factor = match (from, to) {
        (a, b) if a == b => 1.0,
        ("V", "mV") => 1_000.0,
        ("V", "µV") => 1_000_000.0,
        ("mV", "V") => 1e-3,
        ("mV", "µV") => 1_000.0,
        ("µV", "V") => 1e-6,
        ("µV", "mV") => 1e-3,
        ("s", "ms") => 1_000.0,
        ("ms", "s") => 1e-3,
        ("s", "min") => 1.0 / 60.0,
        ("min", "s") => 60.0,
        ("ms", "min") => 1.0 / 60_000.0,
        ("min", "ms") => 60_000.0,
        // Dimensionless/time values have no prefixed V1 representation.
        ("1", "dimensionless") | ("dimensionless", "1") => 1.0,
        _ => return None,
    };
    Some(value * factor)
}
