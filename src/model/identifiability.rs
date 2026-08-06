use serde::{Deserialize, Serialize};

/// Explicit assessment state that preserves a lack of evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatus {
    NotAssessed,
    Supported,
    Contradicted,
    Indeterminate,
}

/// Placeholder report interface for structural/practical identifiability.
/// Phase 02 intentionally does not infer identifiability from a fit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiabilityReport {
    pub structural: AssessmentStatus,
    pub practical: AssessmentStatus,
    pub parameter_ids: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub warnings: Vec<String>,
}

impl IdentifiabilityReport {
    pub(crate) fn not_assessed(parameter_ids: Vec<String>) -> Self {
        Self {
            structural: AssessmentStatus::NotAssessed,
            practical: AssessmentStatus::NotAssessed,
            parameter_ids,
            contradictory_evidence: Vec::new(),
            missing_evidence: vec![
                "No structural or practical identifiability analysis is implemented in Phase 02."
                    .into(),
            ],
            warnings: Vec::new(),
        }
    }
}
