use crate::estimation_config::{StateModelKind, StateTransformKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateDefinition {
    pub name: String,
    pub unit: String,
    pub transform: StateTransform,
    pub lower_bound: Option<f64>,
    pub upper_bound: Option<f64>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateTransform {
    Identity,
    Log10Positive,
    LogPositive,
    LogisticBounded,
}

impl StateTransform {
    pub fn from_config(kind: StateTransformKind) -> Self {
        match kind {
            StateTransformKind::IdentityLog10 => Self::Identity,
            StateTransformKind::Log10Positive => Self::Log10Positive,
            StateTransformKind::LogPositive => Self::LogPositive,
            StateTransformKind::LogisticBounded => Self::LogisticBounded,
        }
    }
    pub fn from_physical(self, value: f64, lower: Option<f64>, upper: Option<f64>) -> Option<f64> {
        let result = match self {
            Self::Identity => value,
            Self::Log10Positive => value.log10(),
            Self::LogPositive => value.ln(),
            Self::LogisticBounded => {
                let (lo, hi) = (lower?, upper?);
                if !(lo..hi).contains(&value) {
                    return None;
                }
                ((value - lo) / (hi - value)).ln()
            }
        };
        result.is_finite().then_some(result)
    }
    /// Derivative of physical value with respect to the latent coordinate.
    pub fn derivative(self, latent: f64, lower: Option<f64>, upper: Option<f64>) -> Option<f64> {
        let result = match self {
            Self::Identity => 1.0,
            Self::Log10Positive => std::f64::consts::LN_10 * 10_f64.powf(latent),
            Self::LogPositive => latent.exp(),
            Self::LogisticBounded => {
                let (lo, hi) = (lower?, upper?);
                let sigmoid = 1.0 / (1.0 + (-latent).exp());
                (hi - lo) * sigmoid * (1.0 - sigmoid)
            }
        };
        result.is_finite().then_some(result)
    }
    pub fn validate_bounds(self, lower: Option<f64>, upper: Option<f64>) -> Result<(), String> {
        if matches!(self, Self::LogisticBounded) {
            match (lower, upper) {
                (Some(lo), Some(hi)) if lo.is_finite() && hi.is_finite() && hi > lo => Ok(()),
                _ => Err("logistic transform requires finite ordered bounds".into()),
            }
        } else {
            Ok(())
        }
    }
    pub fn to_physical(self, value: f64, lower: Option<f64>, upper: Option<f64>) -> Option<f64> {
        let result = match self {
            Self::Identity => value,
            Self::Log10Positive => 10_f64.powf(value),
            Self::LogPositive => value.exp(),
            Self::LogisticBounded => {
                let (lo, hi) = (lower?, upper?);
                lo + (hi - lo) / (1.0 + (-value).exp())
            }
        };
        result.is_finite().then_some(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationDomainStatus {
    Inside,
    NearBoundary,
    Outside,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementUpdateStatus {
    Updated,
    RejectedByGate,
    PredictOnly,
    MissingEnvironment,
    NumericalFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimationWarningKind {
    MissingCalibrationModel,
    UnsupportedCalibrationModel,
    InvalidCalibrationSlope,
    MissingTemperature,
    MissingIonicStrength,
    MissingConductivity,
    MissingInterferentActivity,
    EnvironmentalAlignmentGap,
    CalibrationExtrapolation,
    DuplicateTimestamp,
    NonMonotonicTimestamp,
    MissingMeasurement,
    PredictOnly,
    InnovationRejected,
    CovarianceNotPositiveSemidefinite,
    CovarianceJitterApplied,
    CovarianceFactorizationFailed,
    InvalidProcessCovariance,
    InvalidMeasurementCovariance,
    TransientPriorUnavailable,
    TransientPriorWarning,
    SignalVarianceUnavailable,
    AuxiliaryObservationRejected,
    HealthContextNoncomparable,
    ConditionStateNotIdentifiable,
    UnobservableModel,
    WeaklyObservableState,
    StateBoundApproached,
    StateTransformFailure,
    FilterDivergence,
    SerializationNonfiniteValue,
    TauOutsideObservationWindow,
    ModelDiscrepancy,
    MissingRequiredEnvironment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimationWarning {
    pub kind: EstimationWarningKind,
    pub message: String,
    pub timestamp_s: Option<f64>,
}
impl EstimationWarning {
    pub fn new(kind: EstimationWarningKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            timestamp_s: None,
        }
    }
    pub fn at(kind: EstimationWarningKind, message: impl Into<String>, timestamp_s: f64) -> Self {
        Self {
            kind,
            message: message.into(),
            timestamp_s: Some(timestamp_s),
        }
    }
}

pub fn state_definitions(
    model: StateModelKind,
    condition: bool,
    condition_lower: f64,
    condition_upper: f64,
    activity_transform: StateTransformKind,
) -> Vec<StateDefinition> {
    let configured_transform = StateTransform::from_config(activity_transform);
    // LogisticBounded is meaningful for the bounded sensitivity proxy.  The
    // activity state remains an interpretable log10 coordinate.
    let activity_transform = if matches!(configured_transform, StateTransform::LogisticBounded) {
        StateTransform::Identity
    } else {
        configured_transform
    };
    let mut states = vec![StateDefinition {
        name: "log10_activity".into(),
        unit: if matches!(activity_transform, StateTransform::Identity) {
            "log10(activity)".into()
        } else {
            "activity".into()
        },
        transform: activity_transform,
        lower_bound: None,
        upper_bound: None,
        interpretation: if matches!(activity_transform, StateTransform::Identity) {
            "latent base-10 logarithm of target-ion activity; activity is 10^value".into()
        } else {
            "physical positive activity with a separately exported latent logarithmic coordinate"
                .into()
        },
    }];
    if matches!(
        model,
        StateModelKind::ActivityBaseline
            | StateModelKind::ActivityBaselinePolarization
            | StateModelKind::Custom
    ) {
        states.push(StateDefinition {
            name: "baseline_offset".into(),
            unit: "V".into(),
            transform: StateTransform::Identity,
            lower_bound: None,
            upper_bound: None,
            interpretation: "latent slowly varying reference or baseline voltage offset".into(),
        });
    }
    if matches!(
        model,
        StateModelKind::ActivityBaselinePolarization | StateModelKind::Custom
    ) {
        states.push(StateDefinition {
            name: "polarization".into(),
            unit: "V".into(),
            transform: StateTransform::Identity,
            lower_bound: None,
            upper_bound: None,
            interpretation: "latent nonequilibrium dynamic polarization voltage".into(),
        });
    }
    if condition {
        states.push(StateDefinition {
            name: "sensitivity_scale".into(),
            unit: "dimensionless".into(),
            transform: if matches!(configured_transform, StateTransform::LogisticBounded) {
                StateTransform::LogisticBounded
            } else {
                StateTransform::Identity
            },
            lower_bound: Some(condition_lower),
            upper_bound: Some(condition_upper),
            interpretation: "latent neutral sensor-condition or sensitivity proxy; not a diagnosis"
                .into(),
        });
    }
    states
}

pub fn activity_from_log10(log10_activity: f64) -> Option<f64> {
    10_f64
        .powf(log10_activity)
        .is_finite()
        .then_some(10_f64.powf(log10_activity))
}
