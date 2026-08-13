use crate::{
    domain::ArtifactKind,
    evidence::{EvidenceBundle, EvidenceId},
    mechanism::config::{EvidenceRequirementId, TemporalJoinMode},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClockId(pub String);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClassificationSource {
    StateEstimationEquilibriumAssessment,
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalClassificationMetadata {
    pub classified_fraction: Option<f64>,
    pub equilibrium_fraction: Option<f64>,
    pub steady_state_fraction: Option<f64>,
    pub classification_source: TemporalClassificationSource,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EvidenceTemporalSupport {
    Point {
        timestamp_s: f64,
    },
    Window {
        start_s: f64,
        end_s: f64,
    },
    Event {
        event_id: String,
        start_s: f64,
        end_s: f64,
    },
    Aggregate,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalSupportProvenance {
    pub adapter_id: String,
    pub source_artifact_kind: ArtifactKind,
    pub source_field_paths: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceTemporalMetadata {
    pub evidence_id: EvidenceId,
    pub support: EvidenceTemporalSupport,
    pub clock_id: Option<ClockId>,
    pub classification: TemporalClassificationMetadata,
    pub provenance: TemporalSupportProvenance,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EvidenceTemporalMetadataCatalog {
    pub entries: BTreeMap<EvidenceId, EvidenceTemporalMetadata>,
}
impl Serialize for EvidenceTemporalMetadataCatalog {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.entries.serialize(s)
    }
}
impl<'de> Deserialize<'de> for EvidenceTemporalMetadataCatalog {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self {
            entries: BTreeMap::deserialize(d)?,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalJoinConfig {
    pub point_tolerance_s: f64,
    pub window_overlap_rule: WindowOverlapRule,
    pub event_identity_rule: EventIdentityRule,
    pub minimum_classified_fraction: f64,
    pub minimum_equilibrium_fraction: f64,
    pub clock_mismatch_behavior: ClockMismatchBehavior,
    pub scope_mismatch_behavior: ScopeMismatchBehavior,
    pub mixed_state_policy: MixedStatePolicy,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowOverlapRule {
    PositiveDuration,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventIdentityRule {
    Exact,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockMismatchBehavior {
    Indeterminate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeMismatchBehavior {
    Indeterminate,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MixedStatePolicy {
    RequireAllSteady {
        allow_quasi_equilibrium: bool,
    },
    MinimumSteadyFraction {
        minimum_fraction: f64,
        allow_quasi_equilibrium: bool,
        reject_if_disturbed: bool,
    },
    WorstCase,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalJoinOutcome {
    Eligible,
    Ineligible,
    Indeterminate,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalJoinReasonCode {
    MissingMetadata,
    ModeSupportMismatch,
    UnknownSupport,
    ClockMismatch,
    ClockUnknown,
    ScopeMismatch,
    ScopeAmbiguous,
    PointToleranceExceeded,
    WindowNoPositiveOverlap,
    PointOutsideWindow,
    EventIdentityMismatch,
    ClassificationUnavailable,
    ClassifiedFractionBelowMinimum,
    EquilibriumFractionBelowMinimum,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalJoinRequest {
    pub requirement_id: EvidenceRequirementId,
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
    pub mode: TemporalJoinMode,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalJoinAssessment {
    pub requirement_id: EvidenceRequirementId,
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
    pub mode: TemporalJoinMode,
    pub outcome: TemporalJoinOutcome,
    pub classified_fraction: Option<f64>,
    pub equilibrium_fraction: Option<f64>,
    pub steady_state_fraction: Option<f64>,
    pub reasons: Vec<TemporalJoinReasonCode>,
}
#[derive(Debug, Error)]
pub enum TemporalJoinError {
    #[error("same evidence id")]
    SameEvidenceId,
    #[error("unknown evidence id {0}")]
    UnknownEvidenceId(String),
    #[error("missing temporal metadata {0}")]
    MissingTemporalMetadata(String),
    #[error("invalid temporal configuration")]
    InvalidConfig,
}
pub fn evaluate_temporal_join(
    request: &TemporalJoinRequest,
    bundle: &EvidenceBundle,
    catalog: &EvidenceTemporalMetadataCatalog,
    config: &TemporalJoinConfig,
) -> Result<TemporalJoinAssessment, TemporalJoinError> {
    if request.left_evidence_id == request.right_evidence_id {
        return Err(TemporalJoinError::SameEvidenceId);
    };
    if !config.point_tolerance_s.is_finite() || config.point_tolerance_s < 0.0 {
        return Err(TemporalJoinError::InvalidConfig);
    };
    let left = bundle
        .records
        .iter()
        .find(|r| r.evidence_id == request.left_evidence_id)
        .ok_or_else(|| TemporalJoinError::UnknownEvidenceId(request.left_evidence_id.0.clone()))?;
    let right = bundle
        .records
        .iter()
        .find(|r| r.evidence_id == request.right_evidence_id)
        .ok_or_else(|| TemporalJoinError::UnknownEvidenceId(request.right_evidence_id.0.clone()))?;
    let l = catalog
        .entries
        .get(&left.evidence_id)
        .ok_or_else(|| TemporalJoinError::MissingTemporalMetadata(left.evidence_id.0.clone()))?;
    let r = catalog
        .entries
        .get(&right.evidence_id)
        .ok_or_else(|| TemporalJoinError::MissingTemporalMetadata(right.evidence_id.0.clone()))?;
    let mut a = TemporalJoinAssessment {
        requirement_id: request.requirement_id.clone(),
        left_evidence_id: left.evidence_id.clone(),
        right_evidence_id: right.evidence_id.clone(),
        mode: request.mode,
        outcome: TemporalJoinOutcome::Indeterminate,
        classified_fraction: None,
        equilibrium_fraction: None,
        steady_state_fraction: None,
        reasons: vec![],
    };
    let compatible = matches!(
        (&l.support, &r.support, request.mode),
        (
            EvidenceTemporalSupport::Point { .. },
            EvidenceTemporalSupport::Point { .. },
            TemporalJoinMode::PointPoint
        ) | (
            EvidenceTemporalSupport::Point { .. },
            EvidenceTemporalSupport::Window { .. },
            TemporalJoinMode::PointWindow
        ) | (
            EvidenceTemporalSupport::Window { .. },
            EvidenceTemporalSupport::Point { .. },
            TemporalJoinMode::WindowPoint
        ) | (
            EvidenceTemporalSupport::Window { .. },
            EvidenceTemporalSupport::Window { .. },
            TemporalJoinMode::WindowWindow
        ) | (
            EvidenceTemporalSupport::Event { .. },
            EvidenceTemporalSupport::Event { .. },
            TemporalJoinMode::EventEvent
        )
    );
    if !compatible {
        a.reasons.push(TemporalJoinReasonCode::ModeSupportMismatch);
        return Ok(a);
    };
    if left.experiment_scope != right.experiment_scope {
        a.reasons.push(TemporalJoinReasonCode::ScopeMismatch);
        return Ok(a);
    };
    if l.clock_id != r.clock_id {
        a.reasons
            .push(if l.clock_id.is_some() && r.clock_id.is_some() {
                TemporalJoinReasonCode::ClockMismatch
            } else {
                TemporalJoinReasonCode::ClockUnknown
            });
        return Ok(a);
    }
    let ok = match (&l.support, &r.support) {
        (
            EvidenceTemporalSupport::Point { timestamp_s: x },
            EvidenceTemporalSupport::Point { timestamp_s: y },
        ) => (x - y).abs() <= config.point_tolerance_s,
        (
            EvidenceTemporalSupport::Point { timestamp_s: x },
            EvidenceTemporalSupport::Window { start_s, end_s },
        )
        | (
            EvidenceTemporalSupport::Window { start_s, end_s },
            EvidenceTemporalSupport::Point { timestamp_s: x },
        ) => *x >= *start_s && *x <= *end_s,
        (
            EvidenceTemporalSupport::Window {
                start_s: a,
                end_s: b,
            },
            EvidenceTemporalSupport::Window {
                start_s: c,
                end_s: d,
            },
        ) => a.max(*c) < b.min(*d),
        (
            EvidenceTemporalSupport::Event { event_id: a, .. },
            EvidenceTemporalSupport::Event { event_id: b, .. },
        ) => a == b,
        _ => false,
    };
    a.outcome = if ok {
        TemporalJoinOutcome::Eligible
    } else {
        TemporalJoinOutcome::Ineligible
    };
    if !ok {
        a.reasons.push(match request.mode {
            TemporalJoinMode::PointPoint => TemporalJoinReasonCode::PointToleranceExceeded,
            TemporalJoinMode::PointWindow | TemporalJoinMode::WindowPoint => {
                TemporalJoinReasonCode::PointOutsideWindow
            }
            TemporalJoinMode::WindowWindow => TemporalJoinReasonCode::WindowNoPositiveOverlap,
            TemporalJoinMode::EventEvent => TemporalJoinReasonCode::EventIdentityMismatch,
        })
    };
    Ok(a)
}
