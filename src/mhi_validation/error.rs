use crate::domain::ArtifactError;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MhiValidationError {
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    #[error("MHI validation protocol error: {0}")]
    Protocol(String),
    #[error("MHI validation dataset error: {0}")]
    Dataset(String),
    #[error("MHI validation physical approval error: {0}")]
    Approval(String),
    #[error("MHI validation input path is unsafe: {0}")]
    UnsafePath(PathBuf),
    #[error("MHI validation I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("MHI validation JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("MHI validation TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("MHI validation output already exists: {0}")]
    OutputAlreadyExists(PathBuf),
}
