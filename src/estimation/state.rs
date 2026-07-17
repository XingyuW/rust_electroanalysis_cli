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
            StateTransformKind::IdentityLog10 | StateTransformKind::Log10Positive => {
                Self::Log10Positive
            }
            StateTransformKind::LogPositive => Self::LogPositive,
            StateTransformKind::LogisticBounded => Self::LogisticBounded,
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
) -> Vec<StateDefinition> {
    let mut states = vec![StateDefinition {
        name: "log10_activity".into(),
        unit: "log10(activity)".into(),
        transform: StateTransform::Log10Positive,
        lower_bound: None,
        upper_bound: None,
        interpretation: "latent base-10 logarithm of target-ion activity; activity is 10^value"
            .into(),
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
            transform: StateTransform::Identity,
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
