use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EstimationError {
    #[error("estimation input is invalid: {0}")]
    InvalidInput(String),
    #[error("estimation configuration is invalid: {0}")]
    InvalidConfiguration(String),
    #[error("calibration observation model failed: {0}")]
    Calibration(String),
    #[error("estimation numerical failure: {0}")]
    Numerical(String),
    #[error("estimation covariance failure: {0}")]
    Covariance(String),
    #[error("estimation artifact I/O failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("estimation JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("estimation CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("estimation TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

impl EstimationError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }
    pub fn config(message: impl Into<String>) -> Self {
        Self::InvalidConfiguration(message.into())
    }
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
