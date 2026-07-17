use thiserror::Error;
#[derive(Debug, Error)]
pub enum HealthError {
    #[error("invalid health input: {0}")]
    InvalidInput(String),
    #[error("health artifact I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("health artifact serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
impl HealthError {
    pub fn invalid(s: impl Into<String>) -> Self {
        Self::InvalidInput(s.into())
    }
}
