use super::identifiability::AssessmentStatus;
use serde::{Deserialize, Serialize};

/// Operational state of equilibrium evidence. These classifications are not
/// physical mechanism assignments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EquilibriumStatus {
    Equilibrium,
    QuasiEquilibrium,
    Transitional,
    Disturbed,
    #[default]
    Indeterminate,
}

/// Evidence fields an equilibrium recognizer must request before deciding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EquilibriumEvidenceRequirements {
    pub dynamic_state_derivatives: bool,
    pub dynamic_voltage_magnitude: bool,
    pub measured_equilibrium_gap: bool,
    pub elapsed_time_relative_to_time_constants: bool,
    pub innovation_statistics: bool,
    pub residual_autocorrelation: bool,
    pub environmental_stability: bool,
    pub calibration_domain_validity: bool,
    pub uncertainty: bool,
    pub observability: bool,
}

/// Evidence-preserving equilibrium assessment. No component or time constant
/// is automatically interpreted as physical equilibrium behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquilibriumAssessment {
    pub status: AssessmentStatus,
    #[serde(default)]
    pub classification: EquilibriumStatus,
    pub supporting_evidence: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub validity_domain: String,
}
