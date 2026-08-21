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
    #[error("PhysicalApprovalTrustNotProvisioned")]
    PhysicalApprovalTrustNotProvisioned,
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
    #[error("MHI validation output is not a managed Phase-E bundle: {0}")]
    OutputNotManaged(PathBuf),
    #[error("MHI validation publication is locked: {0}")]
    PublicationLocked(PathBuf),
    #[error("MHI validation publication lock is invalid: {0}")]
    PublicationLockFileInvalid(PathBuf),
    #[error("MHI validation publication recovery residue exists near: {0}")]
    PublicationRecoveryResidue(PathBuf),
    #[error(
        "MHI validation filesystem does not provide the required atomic publication primitive: {0}"
    )]
    UnsupportedAtomicPublicationFilesystem(PathBuf),
    #[error("MHI validation publication destination was concurrently created: {0}")]
    PublicationConcurrentDestinationCreated(PathBuf),
    #[error("MHI validation managed output changed before publication: {0}")]
    PublicationConcurrentManagedOutputChanged(PathBuf),
    #[error("MHI validation committed visible output changed before cleanup: {0}")]
    PublicationCommittedVisibleOutputChanged(PathBuf),
    #[error("MHI validation exchanged old generation changed before cleanup: {0}")]
    PublicationCommittedForeignSwapDetected(PathBuf),
    #[error(
        "MHI validation publication is visible but durability is unconfirmed at {output} during {operation}"
    )]
    PublicationDurabilityUnconfirmed {
        output: PathBuf,
        operation: &'static str,
    },
    #[error("MHI validation publication committed but cleanup failed near: {0}")]
    PublicationCommittedCleanupFailed(PathBuf),
}
