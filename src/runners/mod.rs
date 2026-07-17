//! Application workflow boundaries.
//!
//! Runners coordinate parsing, configuration, fitting, reporting, and
//! rendering.  Scientific equations and optimization remain in `impedance/`
//! and data/rendering implementations remain in their existing modules.

use crate::domain::{
    ConfigurationError, DataParsingError, FittingError, ReportingError, WorkspaceError,
};
use crate::potentiometry::PotentiometryError;
use std::error::Error;
use std::io;
use thiserror::Error as ThisError;

pub mod fit;
pub mod plot;
pub mod search;
pub mod transient;

/// Errors crossing a workflow boundary into the CLI.
#[derive(Debug, ThisError)]
pub enum RunnerError {
    #[error(transparent)]
    Configuration(#[from] ConfigurationError),
    #[error(transparent)]
    Data(#[from] DataParsingError),
    #[error(transparent)]
    Fitting(#[from] FittingError),
    #[error(transparent)]
    Reporting(#[from] ReportingError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error(transparent)]
    Potentiometry(#[from] PotentiometryError),
    #[error("workflow I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("plotting workflow failed: {0}")]
    Backend(#[source] Box<dyn Error + 'static>),
    #[error("workflow error: {0}")]
    Message(String),
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
