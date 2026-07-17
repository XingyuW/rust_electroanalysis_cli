use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignalError {
    #[error("invalid signal input: {0}")]
    InvalidInput(String),
    #[error("sampling policy rejected the data: {0}")]
    Sampling(String),
    #[error("signal artifact I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("signal artifact serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
impl SignalError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }
}
