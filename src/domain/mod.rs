//! Shared application-domain contracts.
//!
//! Phase 1 adds stable scientific measurements, experiment metadata,
//! diagnostics, and provenance while keeping the existing `data_file`,
//! `impedance`, and `plottings` implementation boundaries intact.

pub mod artifact;
pub mod diagnostics;
pub mod errors;
pub mod experiment;
pub mod lineage;
pub mod measurement;
pub mod metadata;
pub mod provenance;

pub(crate) use artifact::write_legacy_sensor_health_assessment_v3;
pub use artifact::{
    ArtifactError, ArtifactKind, CurrentArtifactKindPolicy, VersionedArtifact, read_artifact,
    validate_serialized_finite, write_artifact,
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
pub use lineage::{
    AcquisitionFamilyId, AggregateExperimentScopeId, ArtifactAcquisitionFamilies,
    ArtifactDependency, ArtifactDependencyRole, ArtifactExperimentScope, ArtifactId,
    ArtifactIdentity, ArtifactLineageCatalog, ArtifactLineageNode, ArtifactLineageState,
    EvidenceIndependence, ExperimentId, LineageResolutionReason, LineageResolutionStatus,
    ResolvedAcquisitionFamilies, ResolvedArtifactLineage, ScopeKey, UnknownLineageReason,
    artifact_identity_from_payload, artifact_scope_from_experiment_ids, current_unknown_lineage,
    dependency_from_lineage, known_lineage_from_artifact, legacy_unknown_lineage,
    lineage_scope_and_families, resolve_known_artifact_id, resolve_lineage, semantic_sha256,
};
pub use measurement::{
    CHANNEL_ALIASES_METADATA_KEY, ChannelMetadata, CoordinateConversion, MeasurementChannel,
    MultiChannelMeasurement, SOURCE_HEADER_METADATA_KEY,
};
pub use metadata::{ExperimentMetadataDocument, build_experiment, load_experiment_metadata};
pub use provenance::AnalysisProvenance;
