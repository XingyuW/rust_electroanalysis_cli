//! Shared application-domain contracts.
//!
//! Phase 1 adds stable scientific measurements, experiment metadata,
//! diagnostics, and provenance while keeping the existing `data_file`,
//! `impedance`, and `plottings` implementation boundaries intact.

pub mod diagnostics;
pub mod errors;
pub mod experiment;
pub mod measurement;
pub mod metadata;
pub mod provenance;

pub use diagnostics::{MeasurementParseResult, ParseDiagnostics};
pub use errors::{
    ConfigurationError, DataParsingError, FittingError, PlottingError, ProvenanceError,
    ReportingError, WorkspaceError,
};
pub use experiment::{
    ElectrochemicalExperiment, EnvironmentalSeries, ExperimentEvent, ExperimentEventKind,
    ReferenceMetadata, SensorMetadata,
};
pub use measurement::{ChannelMetadata, MeasurementChannel, MultiChannelMeasurement};
pub use metadata::{ExperimentMetadataDocument, build_experiment, load_experiment_metadata};
pub use provenance::AnalysisProvenance;
