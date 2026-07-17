use thiserror::Error;

#[derive(Debug, Error)]
pub enum MechanismError {
    #[error("mechanism input is invalid: {0}")]
    Invalid(String),
    #[error("mechanism input I/O error for {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("mechanism JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("mechanism TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

impl MechanismError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }
}
