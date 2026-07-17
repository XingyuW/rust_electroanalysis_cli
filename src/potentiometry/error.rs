//! Typed failures raised by potentiometric transient analysis.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PotentiometryError {
    #[error("selected measurement channel '{channel}' does not exist")]
    MissingChannel { channel: String },
    #[error("experiment metadata is required for transient analysis: {0}")]
    MissingMetadata(String),
    #[error("no eligible '{event_kind}' events were found")]
    NoEligibleEvents { event_kind: String },
    #[error("invalid event window: {0}")]
    InvalidEventWindow(String),
    #[error("segment has too few finite observations: required {required}, got {actual}")]
    InsufficientObservations { required: usize, actual: usize },
    #[error("segment duration is too short: required {required:.6} s, got {actual:.6} s")]
    TooShortObservationWindow { required: f64, actual: f64 },
    #[error("segment missing fraction {fraction:.3} exceeds configured maximum {maximum:.3}")]
    ExcessiveMissingData { fraction: f64, maximum: f64 },
    #[error("duplicate timestamps are not permitted under the selected policy")]
    DuplicateTimestamps,
    #[error("non-monotonic timestamps are not permitted under the selected policy")]
    NonMonotonicTimestamps,
    #[error("invalid transient configuration: {0}")]
    InvalidConfiguration(String),
    #[error("optimizer failed for model {model}: {reason}")]
    OptimizerFailure { model: String, reason: String },
    #[error("all candidate transient models failed for event {event_index}")]
    AllCandidateModelsFailed { event_index: usize },
    #[error("bootstrap failed: {0}")]
    BootstrapFailure(String),
    #[error("transient export failed for {path}: {source}")]
    Export {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("transient serialization failed for {path}: {source}")]
    Serialization {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("transient plotting failed for {path}: {message}")]
    Plotting { path: PathBuf, message: String },
    #[error("transient analysis error: {0}")]
    Invalid(String),
}

impl PotentiometryError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }

    pub fn export(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Export {
            path: path.into(),
            source,
        }
    }
}
