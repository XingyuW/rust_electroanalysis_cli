//! TOML experiment metadata loading, kept separate from plot configuration.

use super::experiment::{
    ElectrochemicalExperiment, EnvironmentalSeries, ExperimentEvent, ReferenceMetadata,
    SensorMetadata,
};
use super::measurement::MultiChannelMeasurement;
use super::provenance::AnalysisProvenance;
use super::{ConfigurationError, DataParsingError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// TOML document used to describe an experiment independently of plotting
/// configuration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExperimentMetadataDocument {
    pub experiment_id: String,
    #[serde(default)]
    pub sensor: SensorMetadata,
    #[serde(default)]
    pub reference: Option<ReferenceMetadata>,
    #[serde(default)]
    pub sample_matrix: String,
    #[serde(default, alias = "environmental_series")]
    pub environmental_data: Vec<EnvironmentalSeries>,
    #[serde(default)]
    pub events: Vec<ExperimentEvent>,
}

pub fn load_experiment_metadata(
    path: impl AsRef<Path>,
) -> Result<ExperimentMetadataDocument, ConfigurationError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|error| ConfigurationError::io(path, error))?;
    toml::from_str(&text).map_err(|error| ConfigurationError::parse(path, error))
}

pub fn build_experiment(
    document: ExperimentMetadataDocument,
    measurement: MultiChannelMeasurement,
    input_path: impl AsRef<Path>,
    metadata_path: impl AsRef<Path>,
) -> Result<ElectrochemicalExperiment, DataParsingError> {
    let metadata_path = metadata_path.as_ref();
    let provenance = AnalysisProvenance::from_paths(input_path, Some(metadata_path))?;
    ElectrochemicalExperiment::new(
        document.experiment_id,
        document.sensor,
        document.reference,
        measurement,
        document.environmental_data,
        document.events,
        document.sample_matrix,
        provenance,
    )
}
