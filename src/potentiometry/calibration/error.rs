//! Typed failures for calibration extraction, fitting, validation, and prediction.

use crate::potentiometry::units::UnitError;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalibrationError {
    #[error("calibration unit error: {0}")]
    Unit(#[from] UnitError),
    #[error(
        "calibration extraction requires explicit concentration information. No compatible concentration-step events or concentration column were found for the input data. Provide one of: (1) experiment metadata containing concentration-step events, (2) a concentration column in the input data, (3) a validated calibration manifest."
    )]
    NoObservations,
    #[error("calibration observation error: {0}")]
    InvalidObservation(String),
    #[error("calibration configuration is invalid: {0}")]
    InvalidConfiguration(String),
    #[error("transient equilibrium is unavailable for event {event_index}")]
    TransientEquilibriumUnavailable { event_index: usize },
    #[error("steady-state window is invalid: {0}")]
    InvalidSteadyStateWindow(String),
    #[error("activity model cannot be evaluated: {0}")]
    ActivityModel(String),
    #[error("calibration fit failed for {model}: {reason}")]
    FitFailure { model: String, reason: String },
    #[error("all calibration candidate models failed")]
    AllModelsFailed,
    #[error("calibration model is not identifiable: {0}")]
    NotIdentifiable(String),
    #[error("prediction is invalid: {0}")]
    InvalidPrediction(String),
    #[error("calibration export failed for {path}: {source}")]
    Export {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("calibration serialization failed for {path}: {source}")]
    Serialization {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("calibration plotting failed for {path}: {message}")]
    Plotting { path: PathBuf, message: String },
    #[error("calibration validation failed: {0}")]
    Validation(String),
}

impl CalibrationError {
    pub fn invalid_observation(message: impl Into<String>) -> Self {
        Self::InvalidObservation(message.into())
    }

    pub fn export(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Export {
            path: path.into(),
            source,
        }
    }
}
