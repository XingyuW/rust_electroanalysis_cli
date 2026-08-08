use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EstimationError {
    #[error("estimation input is invalid: {0}")]
    InvalidInput(String),
    #[error("estimation configuration is invalid: {0}")]
    InvalidConfiguration(String),
    #[error(
        "unknown compiled model input binding target '{target_input_id}' from '{source_declaration}' for model '{model_id}'"
    )]
    UnknownModelInputBindingTarget {
        target_input_id: String,
        source_declaration: String,
        model_id: String,
    },
    #[error(
        "unsupported compiled model input source '{source_declaration}' for target '{target_input_id}' in model '{model_id}'"
    )]
    UnsupportedModelInputSource {
        target_input_id: String,
        source_declaration: String,
        model_id: String,
    },
    #[error(
        "missing compiled model input source for target '{target_input_id}' from '{source_declaration}' (expected '{expected_unit}') in model '{model_id}'"
    )]
    MissingModelInputSource {
        target_input_id: String,
        source_declaration: String,
        expected_unit: String,
        model_id: String,
    },
    #[error(
        "compiled model input unit mismatch for target '{target_input_id}' from '{source_declaration}': expected '{expected_unit}', actual '{actual_unit}' in model '{model_id}'"
    )]
    ModelInputUnitMismatch {
        target_input_id: String,
        source_declaration: String,
        expected_unit: String,
        actual_unit: String,
        model_id: String,
    },
    #[error(
        "duplicate compiled model input binding for target '{target_input_id}' in model '{model_id}': {declarations:?}"
    )]
    DuplicateModelInputBinding {
        target_input_id: String,
        declarations: Vec<String>,
        model_id: String,
    },
    #[error("calibration observation model failed: {0}")]
    Calibration(String),
    #[error("estimation numerical failure: {0}")]
    Numerical(String),
    #[error("estimation covariance failure: {0}")]
    Covariance(String),
    #[error("compiled ISM model integration failed during {context}: {source}")]
    CompiledModel {
        context: &'static str,
        #[source]
        source: Box<crate::model::ModelError>,
    },
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
    pub fn compiled(context: &'static str, source: crate::model::ModelError) -> Self {
        Self::CompiledModel {
            context,
            source: Box::new(source),
        }
    }
}
