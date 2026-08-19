use crate::domain::{ArtifactError, LineageCatalogReadError};
use serde::Serialize;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityAxis {
    ExperimentScope,
    SensorScope,
    ChannelScope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationPhase {
    BackupRename,
    PublishRename,
    RestoreRename,
    BackupCleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityReason {
    NotProvided,
    NotSelected,
    LegacyPhaseCNotSerialized,
    LegacyMechanismAssessmentNotSerialized,
    LineageLegacyUnknown,
    UnitAuthorityUnavailable,
    NotComparable,
    ComparisonUnknown,
    NoComparableFinitePair,
    SelectedFitNotFound,
    SelectedFitAmbiguous,
    SerializedSeriesInvalid,
    SerializedSeriesUnavailable,
    PairedInputNotProvided,
    CatalogNotSupplied,
}

/// Public runtime failures for the Phase-D public-output route.
#[derive(Debug, Error)]
pub enum PublicReportError {
    #[error("invalid report option combination: {detail}")]
    InvalidCombination { detail: &'static str },
    #[error("invalid {selector} selection: {value}")]
    InvalidSelection {
        selector: &'static str,
        value: String,
    },
    #[error("could not read {flag} artifact at {path}: {source}")]
    Artifact {
        flag: &'static str,
        path: PathBuf,
        #[source]
        source: ArtifactError,
    },
    #[error("could not read lineage catalog at {path}: {source}")]
    LineageCatalog {
        path: PathBuf,
        #[source]
        source: LineageCatalogReadError,
    },
    #[error("required inputs {left_flag} and {right_flag} disagree on {axis:?}: {left} vs {right}")]
    RequiredInputsIncompatible {
        left_flag: &'static str,
        right_flag: &'static str,
        axis: CompatibilityAxis,
        left: String,
        right: String,
    },
    #[error(
        "optional input {flag} disagrees with {required_flag} on {axis:?}: {actual} vs {expected}"
    )]
    OptionalInputIncompatible {
        flag: &'static str,
        required_flag: &'static str,
        axis: CompatibilityAxis,
        actual: String,
        expected: String,
    },
    #[error("invalid report output directory {path}")]
    InvalidOutputDirectory { path: PathBuf },
    #[error("report output already exists at {path}")]
    OutputCollision { path: PathBuf },
    #[error("report output contains unmanaged entry {path}")]
    UnmanagedOutputEntry { path: PathBuf },
    #[error("requested output {output_id} is unavailable: {reason:?}")]
    RequestedOutputUnavailable {
        output_id: String,
        reason: AvailabilityReason,
    },
    #[error("could not create report staging path {path}: {source}")]
    Staging {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("could not write report output {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("could not write report CSV {path}: {source}")]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },
    #[error("could not serialize report output {path}: {source}")]
    Serialization {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("plot backend failed for {figure_id} at {path}: {message}")]
    PlotBackend {
        figure_id: String,
        path: PathBuf,
        message: String,
    },
    #[error("staging validation failed for {path}: {detail}")]
    StagingValidation { path: PathBuf, detail: String },
    #[error("publication failed during {phase:?} for {staging_path}: {source}")]
    Publication {
        phase: PublicationPhase,
        staging_path: PathBuf,
        backup_path: Option<PathBuf>,
        #[source]
        source: io::Error,
    },
    #[error("could not clean up {path}: {source}")]
    Cleanup {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
