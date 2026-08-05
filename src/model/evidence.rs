use serde::{Deserialize, Serialize};

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
