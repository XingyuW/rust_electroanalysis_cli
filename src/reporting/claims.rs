//! Total, conservative presentation language for Phase-D output.

use crate::{
    mechanism::promotion::HypothesisEvidenceLevel,
    reporting::AvailabilityReason,
    results::{CausalStatus, HealthEvidenceState, OverallHealthStatus},
};

pub const REQUIRED_DISCLAIMER: &str = "This report projects serialized assessments.  Support, association, consistency, and model-derived prediction do not by themselves establish causal proof.";

pub const fn mechanism_level_text(value: HypothesisEvidenceLevel) -> &'static str {
    match value {
        HypothesisEvidenceLevel::NotAssessed => "not assessed",
        HypothesisEvidenceLevel::Hypothesized => "hypothesized",
        HypothesisEvidenceLevel::ExperimentallySupported => {
            "experimentally supported within the serialized evidence"
        }
        HypothesisEvidenceLevel::ValidatedForDomain => "validated for the serialized domain",
        HypothesisEvidenceLevel::Contradicted => "contradicted by the serialized evidence",
    }
}

pub const fn causal_status_text(value: CausalStatus) -> &'static str {
    match value {
        CausalStatus::Observed => "observed",
        CausalStatus::Associated => "associated with",
        CausalStatus::Hypothesized => "hypothesized",
        CausalStatus::ExperimentallySupported => "experimentally supported",
        CausalStatus::ValidatedForDomain => "validated for domain",
        CausalStatus::Indeterminate => {
            "indeterminate; evidence is unavailable or insufficient as stated"
        }
    }
}

pub const fn health_status_text(value: OverallHealthStatus) -> &'static str {
    match value {
        OverallHealthStatus::WithinBaseline => "within serialized baseline",
        OverallHealthStatus::Watch => "watch",
        OverallHealthStatus::Degraded => "degraded",
        OverallHealthStatus::Critical => "critical",
        OverallHealthStatus::DataQualityInsufficient => "Data quality insufficient (DQI)",
        OverallHealthStatus::Indeterminate => "Indeterminate",
    }
}

pub const fn evidence_state_text(value: HealthEvidenceState) -> &'static str {
    match value {
        HealthEvidenceState::AdequateEvidence => "adequate_evidence",
        HealthEvidenceState::NoEvidence => "no_evidence",
        HealthEvidenceState::InsufficientEvidence => "insufficient_evidence",
        HealthEvidenceState::PoorDataQuality => "poor_data_quality",
        HealthEvidenceState::ContradictoryEvidence => "contradictory_evidence",
    }
}

pub const fn unavailable_text(value: AvailabilityReason) -> &'static str {
    match value {
        AvailabilityReason::NotProvided => "not provided",
        AvailabilityReason::NotSelected => "not selected",
        AvailabilityReason::LegacyPhaseCNotSerialized => {
            "legacy Phase C assessment was not serialized"
        }
        AvailabilityReason::LegacyMechanismAssessmentNotSerialized => {
            "legacy Phase B assessment was not serialized"
        }
        AvailabilityReason::LineageLegacyUnknown => "lineage unknown",
        AvailabilityReason::UnitAuthorityUnavailable => "unit authority unavailable",
        AvailabilityReason::NotComparable => "not comparable",
        AvailabilityReason::ComparisonUnknown => "comparison unknown",
        AvailabilityReason::NoComparableFinitePair => "no comparable finite pair",
        AvailabilityReason::SelectedFitNotFound => "selected fit not found",
        AvailabilityReason::SelectedFitAmbiguous => "selected fit is ambiguous",
        AvailabilityReason::SerializedSeriesInvalid => "serialized series invalid",
        AvailabilityReason::SerializedSeriesUnavailable => "serialized series unavailable",
        AvailabilityReason::PairedInputNotProvided => "paired input not provided",
        AvailabilityReason::CatalogNotSupplied => "catalog not supplied",
    }
}
