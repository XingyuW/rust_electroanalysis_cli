use serde::{Deserialize, Serialize};

/// Distinguishes observed evidence from evidence that is not applicable and
/// evidence that was required but unavailable.  `Missing` is never coerced to
/// a numerical neutral value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status", content = "value")]
pub enum EvidenceValue<T> {
    Present(T),
    NotApplicable,
    Missing { reason: String },
}

/// Outcome of checking declared evidence without assigning a mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAssessmentStatus {
    NotAssessed,
    Insufficient,
    Supporting,
    Contradictory,
}

/// Evidence that must be available before a fitted mode can receive a physical
/// mechanism label. The core never assigns the label automatically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    pub hypothesis_id: String,
    pub proposed_mechanism_label: String,
    pub independent_evidence_types: Vec<String>,
    pub minimum_independent_observations: usize,
    pub validity_domain: String,
    pub alternatives_to_consider: Vec<String>,
    pub required_uncertainty_statement: String,
}

/// Evidence ledger for one proposed interpretation. Consumers retain both
/// supporting and contradictory observations; this contract never converts a
/// fit into a mechanism assignment automatically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceAssessment {
    pub status: EvidenceAssessmentStatus,
    pub hypothesis_id: String,
    pub supporting_evidence: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub assessed_domain: String,
}
