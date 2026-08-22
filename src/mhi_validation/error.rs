use crate::domain::ArtifactError;
use std::{fmt, io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationPathState {
    Absent,
    ValidManagedBundle,
    Unmanaged,
    Symlink,
}

impl fmt::Display for PublicationPathState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Absent => "absent",
            Self::ValidManagedBundle => "valid_managed_bundle",
            Self::Unmanaged => "unmanaged",
            Self::Symlink => "symlink",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationIdentityResult {
    Match,
    Mismatch,
    Unavailable,
}

impl fmt::Display for PublicationIdentityResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Match => "match",
            Self::Mismatch => "mismatch",
            Self::Unavailable => "unavailable",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationFingerprintResult {
    Match,
    Mismatch,
    NotEvaluated,
}

impl fmt::Display for PublicationFingerprintResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Match => "match",
            Self::Mismatch => "mismatch",
            Self::NotEvaluated => "not_evaluated",
        })
    }
}

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
    #[error("SupportingEndpointClaimDomainMismatch")]
    SupportingEndpointClaimDomainMismatch,
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
    #[error(
        "MHI validation publication recovery residue near {output}: output_state={output_state}, stage_state={stage_state}, backup_state={backup_state}, remaining_paths={remaining_paths:?}"
    )]
    PublicationRecoveryResidue {
        output: PathBuf,
        output_state: PublicationPathState,
        stage_state: PublicationPathState,
        backup_state: PublicationPathState,
        remaining_paths: Vec<PathBuf>,
    },
    #[error(
        "MHI validation filesystem does not provide the required atomic publication primitive: {0}"
    )]
    UnsupportedAtomicPublicationFilesystem(PathBuf),
    #[error("MHI validation publication destination was concurrently created: {output}")]
    PublicationConcurrentDestinationCreated { output: PathBuf },
    #[error(
        "MHI validation managed output changed before publication: {output}, output_state={output_state}, identity_result={identity_result}, fingerprint_result={fingerprint_result}, remaining_paths={remaining_paths:?}"
    )]
    PublicationConcurrentManagedOutputChanged {
        output: PathBuf,
        output_state: PublicationPathState,
        identity_result: PublicationIdentityResult,
        fingerprint_result: PublicationFingerprintResult,
        remaining_paths: Vec<PathBuf>,
    },
    #[error(
        "MHI validation committed visible output changed before cleanup: {output}, output_state={output_state}, identity_result={identity_result}, fingerprint_result={fingerprint_result}, remaining_paths={remaining_paths:?}"
    )]
    PublicationCommittedVisibleOutputChanged {
        output: PathBuf,
        output_state: PublicationPathState,
        identity_result: PublicationIdentityResult,
        fingerprint_result: PublicationFingerprintResult,
        remaining_paths: Vec<PathBuf>,
    },
    #[error(
        "MHI validation exchanged old generation changed before cleanup: {output}, stage_state={stage_state}, identity_result={identity_result}, fingerprint_result={fingerprint_result}, remaining_paths={remaining_paths:?}"
    )]
    PublicationCommittedForeignSwapDetected {
        output: PathBuf,
        stage_state: PublicationPathState,
        identity_result: PublicationIdentityResult,
        fingerprint_result: PublicationFingerprintResult,
        remaining_paths: Vec<PathBuf>,
    },
    #[error(
        "MHI validation publication is visible but durability is unconfirmed at {output} during {operation}"
    )]
    PublicationDurabilityUnconfirmed {
        output: PathBuf,
        operation: &'static str,
        fsync_error: String,
        remaining_paths: Vec<PathBuf>,
    },
    #[error(
        "MHI validation publication committed but cleanup failed near {output}: stage_state={stage_state}, backup_state={backup_state}, remaining_paths={remaining_paths:?}, cleanup_error={cleanup_error}"
    )]
    PublicationCommittedCleanupFailed {
        output: PathBuf,
        stage_state: PublicationPathState,
        backup_state: PublicationPathState,
        remaining_paths: Vec<PathBuf>,
        cleanup_error: String,
    },
    #[error(
        "MHI validation staging failed and cleanup failed: primary_error={primary_error}, remaining_paths={remaining_paths:?}"
    )]
    PublicationStagingCleanupFailed {
        primary_error: String,
        remaining_paths: Vec<PathBuf>,
    },
}
