//! Application workflow boundaries.
//!
//! Runners coordinate parsing, configuration, fitting, reporting, and
//! rendering.  Scientific equations and optimization remain in `impedance/`
//! and data/rendering implementations remain in their existing modules.

use crate::domain::{
    BatchFileFailure, ConfigurationError, DataParsingError, FittingError, ReportingError,
    WorkspaceError,
};
use crate::potentiometry::{PotentiometryError, calibration::CalibrationError};
use std::io;
use std::{error::Error, path::PathBuf};
use thiserror::Error as ThisError;

pub mod calibration;
pub mod estimation;
pub mod evidence;
pub mod fit;
pub mod health;
pub mod mechanism;
pub mod model;
pub mod model_validation;
pub mod plot;
pub mod report;
pub mod search;
pub mod signal;
pub mod transient;

/// Preserved outcome of a directory-level physical-input workflow.
///
/// Callers receive this inside [`RunnerError::PartialBatch`] when some
/// artifacts were written but one or more inputs failed canonical ingestion
/// or were explicitly rejected by the workflow.
#[derive(Debug, Default)]
pub struct BatchRunSummary {
    pub successful_inputs: Vec<PathBuf>,
    pub failures: Vec<BatchFileFailure>,
}

/// Errors crossing a workflow boundary into the CLI.
#[derive(Debug, ThisError)]
pub enum RunnerError {
    #[error(transparent)]
    Configuration(#[from] ConfigurationError),
    #[error(transparent)]
    Data(#[from] DataParsingError),
    #[error("raw-data batch input failures: {failures:?}")]
    BatchInput { failures: Vec<BatchFileFailure> },
    #[error("{workflow} found no candidate input files in {input_dir}")]
    NoInputCandidates {
        workflow: &'static str,
        input_dir: PathBuf,
    },
    #[error(
        "batch completed {successful_count} input(s) but {failure_count} input(s) failed; completed artifacts were preserved"
    )]
    PartialBatch {
        successful_count: usize,
        failure_count: usize,
        summary: BatchRunSummary,
    },
    #[error("search output collision at {output}: {first_input} and {second_input}")]
    OutputCollision {
        output: std::path::PathBuf,
        first_input: std::path::PathBuf,
        second_input: std::path::PathBuf,
    },
    #[error(transparent)]
    Fitting(#[from] FittingError),
    #[error(transparent)]
    Reporting(#[from] ReportingError),
    #[error(transparent)]
    PublicReport(#[from] crate::reporting::PublicReportError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error(transparent)]
    Potentiometry(#[from] PotentiometryError),
    #[error(transparent)]
    Calibration(#[from] CalibrationError),
    #[error(transparent)]
    Signal(#[from] crate::signal::error::SignalError),
    #[error(transparent)]
    SignalComparison(#[from] Box<crate::signal::comparison::SignalComparisonError>),
    #[error(transparent)]
    Health(#[from] crate::health::error::HealthError),
    #[error(transparent)]
    Estimation(#[from] crate::estimation::error::EstimationError),
    #[error(transparent)]
    Artifact(#[from] crate::domain::ArtifactError),
    #[error(transparent)]
    PhaseBSourceScope(#[from] mechanism::PhaseBSourceScopeError),
    #[error("workflow JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("workflow TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("workflow I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("workflow CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("plotting workflow failed: {0}")]
    Backend(#[source] Box<dyn Error + 'static>),
    #[error("workflow error: {0}")]
    Message(String),
}

impl RunnerError {
    /// Converts a mixed batch outcome into the automation-visible error that
    /// retains completed artifacts and every typed per-file failure.
    pub fn partial_batch(summary: BatchRunSummary) -> Self {
        Self::PartialBatch {
            successful_count: summary.successful_inputs.len(),
            failure_count: summary.failures.len(),
            summary,
        }
    }
}

impl From<Box<dyn Error + 'static>> for RunnerError {
    fn from(error: Box<dyn Error + 'static>) -> Self {
        Self::Backend(error)
    }
}

impl From<String> for RunnerError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}

impl From<&str> for RunnerError {
    fn from(message: &str) -> Self {
        Self::Message(message.to_string())
    }
}
