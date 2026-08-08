use serde::{Deserialize, Serialize};

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
