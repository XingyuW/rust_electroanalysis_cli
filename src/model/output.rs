use super::{
    component::{ComponentRole, ContributionSemantics, JacobianMethod},
    error::ModelError,
};
use serde::{Deserialize, Serialize};

/// Configured numerical tolerance for reconstruction of the voltage sum.
pub const DEFAULT_POTENTIAL_RECONSTRUCTION_TOLERANCE_V: f64 = 1e-12;

/// Non-fatal contract warning produced by model evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "message")]
pub enum ModelWarning {
    Validity(String),
    Identifiability(String),
    Evidence(String),
    Reconstruction(String),
}

/// One categorized component output.  A component can emit potential *or*
/// variance according to its declared semantics; the two units never share a
/// numeric field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentContribution {
    pub component_id: String,
    pub owner: Option<String>,
    pub role: ComponentRole,
    pub semantics: ContributionSemantics,
    pub potential_v: Option<f64>,
    pub variance_v2: Option<f64>,
    pub source: String,
    pub validity_domain: String,
}

/// Explicit status of propagated prediction uncertainty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyStatus {
    Complete,
    Partial,
    Unavailable,
    NotRequested,
}

/// First-order uncertainty decomposition for a scalar voltage prediction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionUncertainty {
    pub status: UncertaintyStatus,
    pub total_variance_v2: Option<f64>,
    pub standard_error_v: Option<f64>,
    pub state_variance_v2: Option<f64>,
    pub parameter_variance_v2: Option<f64>,
    pub observation_variance_v2: Option<f64>,
    #[serde(default)]
    pub missing_sources: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub state_jacobian_methods: Vec<JacobianMethod>,
    #[serde(default)]
    pub parameter_jacobian_methods: Vec<JacobianMethod>,
}

impl PredictionUncertainty {
    pub fn not_requested() -> Self {
        Self {
            status: UncertaintyStatus::NotRequested,
            total_variance_v2: None,
            standard_error_v: None,
            state_variance_v2: None,
            parameter_variance_v2: None,
            observation_variance_v2: None,
            missing_sources: Vec::new(),
            assumptions: Vec::new(),
            state_jacobian_methods: Vec::new(),
            parameter_jacobian_methods: Vec::new(),
        }
    }
}

/// Optional full covariance inputs for first-order propagation. Matrices use
/// compiled state/parameter order and may contain off-diagonal covariance.
#[derive(Debug, Clone, PartialEq)]
pub struct PredictionUncertaintyInput {
    /// `false` is the only path to `NotRequested`.
    pub requested: bool,
    pub state_covariance: Option<Vec<Vec<f64>>>,
    pub parameter_covariance: Option<Vec<Vec<f64>>>,
    pub observation_variance_v2: Option<f64>,
}

impl Default for PredictionUncertaintyInput {
    fn default() -> Self {
        Self {
            requested: true,
            state_covariance: None,
            parameter_covariance: None,
            observation_variance_v2: None,
        }
    }
}

/// Explicit status of the unexplained residual. Missing observed voltage is
/// represented as missing evidence rather than silently omitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status", content = "value_v")]
pub enum UnexplainedResidual {
    Observed(f64),
    MissingObservedVoltage,
}

/// Decomposed model observation with an auditable unexplained residual field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationPrediction {
    pub predicted_voltage_v: f64,
    pub contributions: Vec<ComponentContribution>,
    pub uncertainty: PredictionUncertainty,
    pub unexplained_residual: UnexplainedResidual,
}

impl ObservationPrediction {
    pub(crate) fn new(
        contributions: Vec<ComponentContribution>,
        observed_voltage_v: Option<f64>,
        uncertainty: PredictionUncertainty,
    ) -> Result<Self, ModelError> {
        let predicted_voltage_v = contributions.iter().try_fold(0.0, |sum, contribution| {
            if contribution.semantics != ContributionSemantics::AdditivePotential {
                return Ok(sum);
            }
            let value = contribution.potential_v.ok_or_else(|| {
                ModelError::IncompatibleContributionOutput {
                    component: contribution.component_id.clone(),
                    semantics: contribution.semantics,
                }
            })?;
            if !value.is_finite() {
                return Err(ModelError::NonFiniteContribution {
                    component: contribution.component_id.clone(),
                });
            }
            Ok(sum + value)
        })?;
        if !predicted_voltage_v.is_finite() {
            return Err(ModelError::NonFinite {
                subject: "predicted voltage".into(),
            });
        }
        let unexplained_residual = match observed_voltage_v {
            Some(observed) if observed.is_finite() => {
                UnexplainedResidual::Observed(observed - predicted_voltage_v)
            }
            Some(_) => {
                return Err(ModelError::NonFinite {
                    subject: "observed voltage".into(),
                });
            }
            None => UnexplainedResidual::MissingObservedVoltage,
        };
        Ok(Self {
            predicted_voltage_v,
            contributions,
            uncertainty,
            unexplained_residual,
        })
    }

    /// Checks the stated reconstruction invariant using only explicitly
    /// additive-potential terms. Observation variance and non-numerical
    /// components are intentionally excluded.
    pub fn verify_reconstruction(&self, tolerance_v: f64) -> Result<(), ModelError> {
        if !tolerance_v.is_finite() || tolerance_v < 0.0 {
            return Err(ModelError::InvalidTolerance { tolerance_v });
        }
        let reconstructed = self
            .contributions
            .iter()
            .filter(|item| item.semantics == ContributionSemantics::AdditivePotential)
            .map(|item| item.potential_v.unwrap_or(f64::NAN))
            .sum::<f64>();
        if !reconstructed.is_finite()
            || (reconstructed - self.predicted_voltage_v).abs() > tolerance_v
        {
            return Err(ModelError::ContributionReconstruction {
                predicted_v: self.predicted_voltage_v,
                reconstructed_v: reconstructed,
                tolerance_v,
            });
        }
        Ok(())
    }
}

/// Public scientific name for a decomposed model observation.
pub type ModelPrediction = ObservationPrediction;
