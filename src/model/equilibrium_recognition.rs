//! Neutral, evidence-driven equilibrium recognition for model-core callers.
//!
//! It deliberately consumes no signal-processing, estimation, health, or
//! mechanism modules. A small measured-voltage slope alone is never evidence
//! of equilibrium here.

use super::{
    identifiability::AssessmentStatus, output::UncertaintyStatus, validity::ValidityReport,
};
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

/// Model-core evidence. Optional high-level diagnostic fields are deliberately
/// plain numbers supplied by the caller; this module never imports their
/// producers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquilibriumEvidence {
    pub dynamic_state_derivative_norm: Option<f64>,
    pub dynamic_potential_magnitude_v: Option<f64>,
    pub equilibrium_gap_v: Option<f64>,
    #[serde(default)]
    pub elapsed_tau_ratios: Vec<f64>,
    pub environmental_stability: Option<f64>,
    pub innovation_metric: Option<f64>,
    pub residual_autocorrelation: Option<f64>,
    pub observable: Option<bool>,
    pub validity: ValidityReport,
    pub uncertainty_status: UncertaintyStatus,
    pub external_disturbance_potential_v: Option<f64>,
}

/// Transparent thresholds for the V1 recognizer. They must be set in the
/// experimental context rather than inferred from a voltage trace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquilibriumRecognitionConfig {
    pub derivative_threshold_v_per_s: f64,
    pub dynamic_potential_threshold_v: f64,
    pub equilibrium_gap_threshold_v: f64,
    pub external_disturbance_threshold_v: f64,
    /// Larger values mean a more stable environment. Values below this bound
    /// are clearly disturbed; values up to `marginal_environmental_stability`
    /// are quasi-equilibrium only.
    pub environmental_stability_threshold: f64,
    pub marginal_environmental_stability: f64,
    pub minimum_elapsed_tau_ratio: f64,
    pub innovation_threshold: f64,
    pub residual_autocorrelation_threshold: f64,
}

impl Default for EquilibriumRecognitionConfig {
    fn default() -> Self {
        Self {
            derivative_threshold_v_per_s: 1e-5,
            dynamic_potential_threshold_v: 1e-4,
            equilibrium_gap_threshold_v: 1e-4,
            external_disturbance_threshold_v: 1e-4,
            environmental_stability_threshold: 0.8,
            marginal_environmental_stability: 0.95,
            minimum_elapsed_tau_ratio: 3.0,
            innovation_threshold: 2.0,
            residual_autocorrelation_threshold: 0.2,
        }
    }
}

/// Detailed, auditable decision result. `status` remains the generic model
/// assessment status retained for existing consumers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquilibriumAssessment {
    pub status: AssessmentStatus,
    #[serde(default)]
    pub classification: EquilibriumStatus,
    pub supporting_evidence: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub validity_domain: String,
    #[serde(default)]
    pub satisfied_criteria: Vec<String>,
    #[serde(default)]
    pub violated_criteria: Vec<String>,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Classify supplied neutral evidence. Nonfinite values count as missing;
/// no default value is invented for absent uncertainty or diagnostics.
pub fn recognize_equilibrium(
    evidence: &EquilibriumEvidence,
    config: &EquilibriumRecognitionConfig,
) -> EquilibriumAssessment {
    let mut satisfied = Vec::new();
    let mut violated = Vec::new();
    let mut missing = Vec::new();
    let mut warnings = Vec::new();
    let finite = |value: Option<f64>, name: &str, missing: &mut Vec<String>| -> Option<f64> {
        match value.filter(|value| value.is_finite()) {
            Some(value) => Some(value),
            None => {
                missing.push(name.into());
                None
            }
        }
    };
    let derivative = finite(
        evidence.dynamic_state_derivative_norm,
        "dynamic state derivative",
        &mut missing,
    );
    let dynamic = finite(
        evidence.dynamic_potential_magnitude_v,
        "dynamic potential magnitude",
        &mut missing,
    );
    let gap = finite(evidence.equilibrium_gap_v, "equilibrium gap", &mut missing);
    let environment = finite(
        evidence.environmental_stability,
        "environmental stability",
        &mut missing,
    );
    let innovation = finite(
        evidence.innovation_metric,
        "innovation metric",
        &mut missing,
    );
    let residual = finite(
        evidence.residual_autocorrelation,
        "residual autocorrelation",
        &mut missing,
    );
    if evidence.elapsed_tau_ratios.is_empty()
        || evidence
            .elapsed_tau_ratios
            .iter()
            .any(|value| !value.is_finite())
    {
        missing.push("elapsed time relative to all dynamic time constants".into());
    }
    if evidence.observable != Some(true) {
        missing.push("model observability".into());
    }
    if !evidence.validity.is_valid {
        missing.push("acceptable component validity".into());
    }
    if evidence.uncertainty_status != UncertaintyStatus::Complete {
        missing.push("complete required uncertainty".into());
    }
    if !missing.is_empty() {
        warnings
            .push("equilibrium is indeterminate because required evidence is unavailable".into());
        return assessment(
            EquilibriumStatus::Indeterminate,
            satisfied,
            violated,
            missing,
            warnings,
            evidence,
        );
    }
    let external = evidence
        .external_disturbance_potential_v
        .unwrap_or(0.0)
        .abs();
    if external > config.external_disturbance_threshold_v {
        violated.push("external disturbance exceeds threshold".into());
        return assessment(
            EquilibriumStatus::Disturbed,
            satisfied,
            violated,
            missing,
            warnings,
            evidence,
        );
    }
    let environment = environment.expect("checked above");
    if environment < config.environmental_stability_threshold {
        violated.push("environmental stability clearly fails".into());
        return assessment(
            EquilibriumStatus::Disturbed,
            satisfied,
            violated,
            missing,
            warnings,
            evidence,
        );
    }
    let derivative = derivative.expect("checked above").abs();
    let dynamic = dynamic.expect("checked above").abs();
    if derivative > config.derivative_threshold_v_per_s
        || dynamic > config.dynamic_potential_threshold_v
    {
        violated.push("dynamic mode remains above transition threshold".into());
        return assessment(
            EquilibriumStatus::Transitional,
            satisfied,
            violated,
            missing,
            warnings,
            evidence,
        );
    }
    let gap = gap.expect("checked above").abs();
    let elapsed_ok = evidence
        .elapsed_tau_ratios
        .iter()
        .all(|ratio| *ratio >= config.minimum_elapsed_tau_ratio);
    let marginal = gap > config.equilibrium_gap_threshold_v
        || !elapsed_ok
        || environment < config.marginal_environmental_stability
        || innovation.expect("checked above").abs() > config.innovation_threshold
        || residual.expect("checked above").abs() > config.residual_autocorrelation_threshold;
    if marginal {
        warnings
            .push("dynamics are small but one or more equilibrium criteria are marginal".into());
        return assessment(
            EquilibriumStatus::QuasiEquilibrium,
            satisfied,
            violated,
            missing,
            warnings,
            evidence,
        );
    }
    satisfied.extend(
        [
            "dynamic derivative below threshold",
            "dynamic potential below threshold",
            "equilibrium gap below threshold",
            "validity acceptable",
            "model observable",
            "required uncertainty complete",
            "environment stable",
            "elapsed time covers dynamic modes",
        ]
        .into_iter()
        .map(str::to_string),
    );
    assessment(
        EquilibriumStatus::Equilibrium,
        satisfied,
        violated,
        missing,
        warnings,
        evidence,
    )
}

fn assessment(
    status: EquilibriumStatus,
    satisfied: Vec<String>,
    violated: Vec<String>,
    missing: Vec<String>,
    warnings: Vec<String>,
    evidence: &EquilibriumEvidence,
) -> EquilibriumAssessment {
    let total = satisfied.len() + violated.len() + missing.len();
    EquilibriumAssessment {
        status: if status == EquilibriumStatus::Indeterminate {
            AssessmentStatus::Indeterminate
        } else if matches!(
            status,
            EquilibriumStatus::Disturbed | EquilibriumStatus::Transitional
        ) {
            AssessmentStatus::Contradicted
        } else {
            AssessmentStatus::Supported
        },
        classification: status,
        supporting_evidence: satisfied.clone(),
        contradictory_evidence: violated.clone(),
        missing_evidence: missing.clone(),
        validity_domain: evidence.validity.checked_domain.clone(),
        satisfied_criteria: satisfied,
        violated_criteria: violated,
        confidence: if total == 0 {
            0.0
        } else {
            (1.0 - (warnings.len() + 1) as f64 / (total + 1) as f64).max(0.0)
        },
        warnings,
    }
}
