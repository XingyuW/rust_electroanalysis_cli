//! Shared scientific measurement structures for time-series sensor data.

use super::DataParsingError;
use super::diagnostics::ParseDiagnostics;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type ChannelMetadata = BTreeMap<String, String>;

/// One named signal sharing a measurement time axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementChannel {
    pub name: String,
    pub unit: String,
    pub values: Vec<Option<f64>>,
    /// Optional per-sample variance in the channel's declared unit squared.
    #[serde(default)]
    pub variance: Option<Vec<Option<f64>>>,
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[serde(default)]
    pub analyte_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<ChannelMetadata>,
}

impl MeasurementChannel {
    pub fn new(name: impl Into<String>, unit: impl Into<String>, values: Vec<Option<f64>>) -> Self {
        Self {
            name: name.into(),
            unit: unit.into(),
            values,
            variance: None,
            sensor_id: None,
            analyte_id: None,
            metadata: None,
        }
    }

    pub fn from_values(name: impl Into<String>, unit: impl Into<String>, values: Vec<f64>) -> Self {
        Self::new(name, unit, values.into_iter().map(Some).collect())
    }

    pub fn with_sensor_id(mut self, sensor_id: impl Into<String>) -> Self {
        self.sensor_id = Some(sensor_id.into());
        self
    }

    pub fn with_variance(mut self, variance: Vec<Option<f64>>) -> Self {
        self.variance = Some(variance);
        self
    }

    pub fn with_analyte_id(mut self, analyte_id: impl Into<String>) -> Self {
        self.analyte_id = Some(analyte_id.into());
        self
    }

    pub fn missing_value_count(&self) -> usize {
        self.values
            .iter()
            .filter(|value| value.is_none_or(|value| !value.is_finite()))
            .count()
    }

    pub fn validate(&self, expected_len: usize) -> Result<(), DataParsingError> {
        if self.values.len() != expected_len {
            return Err(DataParsingError::invalid(format!(
                "channel '{}' has {} values but the shared time axis has {} entries",
                self.name,
                self.values.len(),
                expected_len
            )));
        }
        if self
            .variance
            .as_ref()
            .is_some_and(|variance| variance.len() != expected_len)
        {
            return Err(DataParsingError::invalid(format!(
                "channel '{}' variance has a different length than the shared time axis",
                self.name
            )));
        }

        if self.values.iter().flatten().any(|value| !value.is_finite()) {
            return Err(DataParsingError::invalid(format!(
                "channel '{}' contains a non-finite value",
                self.name
            )));
        }
        if self
            .variance
            .iter()
            .flatten()
            .flatten()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(DataParsingError::invalid(format!(
                "channel '{}' contains a non-finite or negative variance",
                self.name
            )));
        }

        Ok(())
    }
}

/// Multiple named channels measured against one shared time axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiChannelMeasurement {
    pub time: Vec<f64>,
    pub channels: Vec<MeasurementChannel>,
}

impl MultiChannelMeasurement {
    pub fn new(
        time: Vec<f64>,
        channels: Vec<MeasurementChannel>,
    ) -> Result<Self, DataParsingError> {
        let measurement = Self { time, channels };
        measurement.validate()?;
        Ok(measurement)
    }

    pub fn validate(&self) -> Result<(), DataParsingError> {
        if self.time.is_empty() {
            return Err(DataParsingError::invalid("measurement time axis is empty"));
        }
        if self.channels.is_empty() {
            return Err(DataParsingError::invalid("measurement has no channels"));
        }
        if self.time.iter().any(|time| !time.is_finite()) {
            return Err(DataParsingError::invalid(
                "measurement time axis contains a non-finite value",
            ));
        }
        for channel in &self.channels {
            channel.validate(self.time.len())?;
        }
        Ok(())
    }

    pub fn time_axis(&self) -> &[f64] {
        &self.time
    }

    pub fn channel(&self, name: &str) -> Option<&MeasurementChannel> {
        self.channels.iter().find(|channel| {
            channel.name == name
                || (!channel.unit.is_empty()
                    && format!("{}/{}", channel.name, channel.unit) == name)
                || (!channel.unit.is_empty()
                    && format!("{} [{}]", channel.name, channel.unit) == name)
        })
    }

    pub fn missing_value_count(&self) -> usize {
        self.channels
            .iter()
            .map(MeasurementChannel::missing_value_count)
            .sum()
    }

    pub fn missing_values_by_channel(&self) -> BTreeMap<String, usize> {
        self.channels
            .iter()
            .map(|channel| (channel.name.clone(), channel.missing_value_count()))
            .collect()
    }

    pub fn diagnostics(&self) -> ParseDiagnostics {
        ParseDiagnostics::from_measurement(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{MeasurementChannel, MultiChannelMeasurement};

    #[test]
    fn validates_shared_axis_alignment_and_reports_missing_values() {
        let measurement = MultiChannelMeasurement::new(
            vec![0.0, 1.0, 2.0],
            vec![
                MeasurementChannel::new("potential", "V", vec![Some(0.1), None, Some(0.3)]),
                MeasurementChannel::from_values("temperature", "C", vec![25.0, 25.1, 25.2]),
            ],
        )
        .expect("aligned channels should validate");

        assert_eq!(measurement.missing_value_count(), 1);
        assert_eq!(measurement.missing_values_by_channel()["potential"], 1);
        assert!(!measurement.diagnostics().irregular_sampling);
    }

    #[test]
    fn rejects_misaligned_channels() {
        let error = MultiChannelMeasurement::new(
            vec![0.0, 1.0],
            vec![MeasurementChannel::from_values("potential", "V", vec![0.1])],
        )
        .expect_err("misaligned channel should fail");

        assert!(error.to_string().contains("shared time axis"));
    }

    #[test]
    fn detects_irregular_sampling_and_duplicate_timestamps() {
        let measurement = MultiChannelMeasurement::new(
            vec![0.0, 1.0, 3.0, 1.0],
            vec![MeasurementChannel::from_values(
                "potential",
                "V",
                vec![0.1, 0.2, 0.3, 0.4],
            )],
        )
        .expect("finite time axis should validate");
        let diagnostics = measurement.diagnostics();

        assert!(diagnostics.irregular_sampling);
        assert_eq!(diagnostics.duplicate_timestamps, 1);
        assert_eq!(diagnostics.non_monotonic_timestamps, 1);
    }
}
