//! Shared application-domain contracts.
//!
//! Phase 1 adds stable scientific measurements, experiment metadata,
//! diagnostics, and provenance while keeping the existing `data_file`,
//! `impedance`, and `plottings` implementation boundaries intact.

pub mod artifact;
pub mod diagnostics;
pub mod errors;
pub mod experiment;
pub mod measurement;
pub mod metadata;
pub mod provenance;

pub use artifact::{
    ArtifactError, ArtifactKind, VersionedArtifact, read_artifact, validate_serialized_finite,
    write_artifact,
};
pub use diagnostics::{IngestionDiagnostic, MeasurementParseResult, ParseDiagnostics};
pub use errors::{
    BatchFileFailure, ConfigurationError, DataParsingError, FittingError, PlottingError,
    ProvenanceError, ReportingError, WorkspaceError,
};
pub use experiment::{
    ElectrochemicalExperiment, EnvironmentalSeries, ExperimentEvent, ExperimentEventKind,
    ReferenceMetadata, SensorMetadata,
};
pub use measurement::{
    CHANNEL_ALIASES_METADATA_KEY, ChannelMetadata, CoordinateConversion, MeasurementChannel,
    MultiChannelMeasurement, SOURCE_HEADER_METADATA_KEY,
};
pub use metadata::{ExperimentMetadataDocument, build_experiment, load_experiment_metadata};
pub use provenance::AnalysisProvenance;
