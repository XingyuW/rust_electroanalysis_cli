//! Experiment-level metadata and event structures.

use super::DataParsingError;
use super::measurement::{ChannelMetadata, MultiChannelMeasurement};
use super::provenance::AnalysisProvenance;
use serde::{Deserialize, Serialize};

/// Metadata describing the working sensor.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SensorMetadata {
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub sensor_type: Option<String>,
    #[serde(default)]
    pub analyte: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub metadata: Option<ChannelMetadata>,
}

/// Metadata describing an optional reference electrode.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReferenceMetadata {
    #[serde(default)]
    pub reference_id: Option<String>,
    #[serde(default)]
    pub electrode_type: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub potential: Option<f64>,
    #[serde(default)]
    pub potential_unit: Option<String>,
    #[serde(default)]
    pub metadata: Option<ChannelMetadata>,
}

/// An environmental signal such as temperature, flow, or ionic strength.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalSeries {
    pub name: String,
    pub unit: String,
    #[serde(alias = "time_axis")]
    pub time: Vec<f64>,
    pub values: Vec<Option<f64>>,
    #[serde(default)]
    pub metadata: Option<ChannelMetadata>,
}

impl EnvironmentalSeries {
    pub fn validate(&self) -> Result<(), DataParsingError> {
        if self.time.len() != self.values.len() {
            return Err(DataParsingError::invalid(format!(
                "environmental series '{}' has {} timestamps and {} values",
                self.name,
                self.time.len(),
                self.values.len()
            )));
        }
        if self.time.iter().any(|time| !time.is_finite()) {
            return Err(DataParsingError::invalid(format!(
                "environmental series '{}' contains a non-finite timestamp",
                self.name
            )));
        }
        if self.values.iter().flatten().any(|value| !value.is_finite()) {
            return Err(DataParsingError::invalid(format!(
                "environmental series '{}' contains a non-finite value",
                self.name
            )));
        }
        Ok(())
    }
}

/// Event categories needed to describe a sensor experiment without encoding
/// experiment-specific behavior into the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentEventKind {
    ConcentrationStep,
    FlowChange,
    TemperatureChange,
    IonicStrengthChange,
    InterferentAddition,
    FlushStart,
    ReadingStart,
    FlushEnd,
    ManualAnnotation,
}

/// A timestamped experimental action or annotation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExperimentEvent {
    pub timestamp: f64,
    pub kind: ExperimentEventKind,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub analyte: Option<String>,
    #[serde(default)]
    pub annotation: Option<String>,
    #[serde(default)]
    pub metadata: Option<ChannelMetadata>,
}

impl ExperimentEvent {
    pub fn validate(&self) -> Result<(), DataParsingError> {
        if !self.timestamp.is_finite() {
            return Err(DataParsingError::invalid(
                "experiment event timestamp must be finite",
            ));
        }
        if self.value.is_some_and(|value| !value.is_finite()) {
            return Err(DataParsingError::invalid(
                "experiment event value must be finite when present",
            ));
        }
        Ok(())
    }
}

/// Complete experiment context around a scientific measurement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElectrochemicalExperiment {
    pub experiment_id: String,
    pub sensor_metadata: SensorMetadata,
    #[serde(default)]
    pub reference_metadata: Option<ReferenceMetadata>,
    pub measurement_data: MultiChannelMeasurement,
    #[serde(default)]
    pub environmental_data: Vec<EnvironmentalSeries>,
    #[serde(default)]
    pub events: Vec<ExperimentEvent>,
    pub sample_matrix: String,
    pub provenance: AnalysisProvenance,
}

impl ElectrochemicalExperiment {
    // An experiment is intentionally assembled from all of its validated
    // scientific components at this boundary; keep the explicit constructor
    // rather than hiding required provenance or metadata in defaults.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        experiment_id: impl Into<String>,
        sensor_metadata: SensorMetadata,
        reference_metadata: Option<ReferenceMetadata>,
        measurement_data: MultiChannelMeasurement,
        environmental_data: Vec<EnvironmentalSeries>,
        mut events: Vec<ExperimentEvent>,
        sample_matrix: impl Into<String>,
        provenance: AnalysisProvenance,
    ) -> Result<Self, DataParsingError> {
        measurement_data.validate()?;
        for series in &environmental_data {
            series.validate()?;
        }
        for event in &events {
            event.validate()?;
        }
        events.sort_by(|left, right| {
            left.timestamp
                .partial_cmp(&right.timestamp)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(Self {
            experiment_id: experiment_id.into(),
            sensor_metadata,
            reference_metadata,
            measurement_data,
            environmental_data,
            events,
            sample_matrix: sample_matrix.into(),
            provenance,
        })
    }

    pub fn ordered_events(&self) -> &[ExperimentEvent] {
        &self.events
    }

    pub fn measurement(&self) -> &MultiChannelMeasurement {
        &self.measurement_data
    }
}

#[cfg(test)]
mod tests {
    use super::{ElectrochemicalExperiment, ExperimentEvent, ExperimentEventKind, SensorMetadata};
    use crate::domain::{AnalysisProvenance, MeasurementChannel, MultiChannelMeasurement};
    use std::path::PathBuf;

    fn provenance() -> AnalysisProvenance {
        AnalysisProvenance {
            software_version: "test".to_string(),
            input_path: PathBuf::from("input.csv"),
            input_sha256: "input".to_string(),
            configuration_path: None,
            configuration_sha256: None,
            generation_timestamp: 1,
            git_commit: None,
        }
    }

    #[test]
    fn orders_events_by_timestamp() {
        let measurement = MultiChannelMeasurement::new(
            vec![0.0, 1.0],
            vec![MeasurementChannel::from_values(
                "potential",
                "V",
                vec![0.1, 0.2],
            )],
        )
        .expect("measurement");
        let experiment = ElectrochemicalExperiment::new(
            "exp-1",
            SensorMetadata::default(),
            None,
            measurement,
            Vec::new(),
            vec![
                ExperimentEvent {
                    timestamp: 2.0,
                    kind: ExperimentEventKind::FlushEnd,
                    value: None,
                    unit: None,
                    analyte: None,
                    annotation: None,
                    metadata: None,
                },
                ExperimentEvent {
                    timestamp: 1.0,
                    kind: ExperimentEventKind::ReadingStart,
                    value: None,
                    unit: None,
                    analyte: None,
                    annotation: None,
                    metadata: None,
                },
            ],
            "buffer",
            provenance(),
        )
        .expect("experiment");

        assert_eq!(experiment.events[0].timestamp, 1.0);
        assert_eq!(experiment.events[1].timestamp, 2.0);
    }
}
