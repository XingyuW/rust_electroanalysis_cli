//! Typed errors shared by the CLI, scientific core, and workflow boundaries.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors raised while loading, validating, or serializing TOML configuration.
#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("configuration I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("configuration parse error for {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("configuration serialization error: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

impl ConfigurationError {
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn parse(path: impl Into<PathBuf>, source: toml::de::Error) -> Self {
        Self::Parse {
            path: path.into(),
            source,
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }
}

/// Errors raised while reading or structurally interpreting instrument data.
#[derive(Debug, Error)]
pub enum DataParsingError {
    #[error("data I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid data{}: {message}", path_suffix(path))]
    Invalid {
        path: Option<PathBuf>,
        message: String,
    },
    #[error(transparent)]
    Configuration(#[from] ConfigurationError),
    #[error(transparent)]
    Fitting(#[from] FittingError),
}

impl DataParsingError {
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid {
            path: None,
            message: message.into(),
        }
    }

    pub fn invalid_at(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Invalid {
            path: Some(path.into()),
            message: message.into(),
        }
    }
}

impl From<io::Error> for DataParsingError {
    fn from(source: io::Error) -> Self {
        Self::io("<input>", source)
    }
}

/// Errors raised by circuit parsing, numerical fitting, and ECM search.
#[derive(Debug, Error)]
pub enum FittingError {
    #[error("invalid fitting input: {0}")]
    InvalidInput(String),
    #[error("circuit parse error: {0}")]
    CircuitParse(String),
    #[error("optimizer failed: {0}")]
    Optimizer(String),
    #[error("ECM search failed: {0}")]
    Search(String),
    #[error("regression failed: {0}")]
    Regression(String),
    #[error("fitting I/O error: {0}")]
    Io(#[from] io::Error),
}

impl FittingError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn circuit_parse(message: impl Into<String>) -> Self {
        Self::CircuitParse(message.into())
    }

    pub fn optimizer(message: impl Into<String>) -> Self {
        Self::Optimizer(message.into())
    }

    pub fn search(message: impl Into<String>) -> Self {
        Self::Search(message.into())
    }

    pub fn regression(message: impl Into<String>) -> Self {
        Self::Regression(message.into())
    }
}

/// Errors raised while constructing or writing fit/search reports.
#[derive(Debug, Error)]
pub enum ReportingError {
    #[error(transparent)]
    Fitting(#[from] FittingError),
    #[error("report parameter count mismatch for {circuit}: expected {expected}, got {actual}")]
    ParameterCountMismatch {
        circuit: String,
        expected: usize,
        actual: usize,
    },
    #[error("report I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid report data: {0}")]
    Invalid(String),
}

impl ReportingError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }
}

/// Errors raised while creating the workspace or persisting application state.
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("workspace I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    Configuration(#[from] ConfigurationError),
    #[error("workspace error: {0}")]
    Invalid(String),
}

impl WorkspaceError {
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }
}

impl From<io::Error> for WorkspaceError {
    fn from(source: io::Error) -> Self {
        Self::io("<workspace>", source)
    }
}

/// Errors raised while converting parsed data into renderable plot series.
#[derive(Debug, Error)]
pub enum PlottingError {
    #[error(transparent)]
    Fitting(#[from] FittingError),
    #[error("plotting data error: {0}")]
    Data(String),
}

impl PlottingError {
    pub fn data(message: impl Into<String>) -> Self {
        Self::Data(message.into())
    }
}

fn path_suffix(path: &Option<PathBuf>) -> String {
    path.as_ref()
        .map(|path| format!(" for {}", path.display()))
        .unwrap_or_default()
}
