use super::identifiability::AssessmentStatus;
use serde::{Deserialize, Serialize};

/// Evidence-preserving equilibrium assessment. No component or time constant
/// is automatically interpreted as physical equilibrium behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquilibriumAssessment {
    pub status: AssessmentStatus,
    pub supporting_evidence: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub validity_domain: String,
}
