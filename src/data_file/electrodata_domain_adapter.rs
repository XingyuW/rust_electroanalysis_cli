//! Canonical physical-input to project-domain conversion.
//!
//! `electrodata-io` owns container detection, parsing, schema recognition,
//! recovery policy, worksheet selection, and input diagnostics.  This module
//! deliberately performs only domain conversion and does not inspect raw
//! headers or the compatibility Polars representation.

use crate::data_file::chi_file::EISData;
use crate::domain::{
    DataParsingError, MeasurementChannel, MeasurementParseResult, MultiChannelMeasurement,
    ParseDiagnostics,
};
use electrodata_io::{
    Column, ColumnRole, Dataset, InvalidCellPolicy, InvalidTimePolicy, RaggedRowPolicy,
    ReadOptions, SheetSelector,
};
use std::collections::BTreeMap;
use std::path::Path;

/// The explicit compatibility policy used by normal project workflows.
///
/// It preserves historical loadability without changing electrodata-io's
/// strict defaults: malformed coordinates skip their row, malformed
/// measurement cells become null, ragged rows are padded/null-trimmed, and
/// source coordinate ordering is retained for downstream scientific handling.
pub fn project_compatibility_read_options() -> ReadOptions {
    ReadOptions::compatibility()
        .with_invalid_time_policy(InvalidTimePolicy::SkipRow)
        .with_invalid_cell_policy(InvalidCellPolicy::Null)
        .with_ragged_row_policy(RaggedRowPolicy::PadNulls)
}

/// Reads with the project's explicit compatibility profile.
pub fn read_dataset(path: impl AsRef<Path>) -> Result<Dataset, DataParsingError> {
    electrodata_io::read_with_options(path, &project_compatibility_read_options())
        .map_err(Into::into)
}

/// Reads with the project's compatibility profile and a caller-selected worksheet.
pub fn read_dataset_with_sheet(
    path: impl AsRef<Path>,
    sheet_name: Option<&str>,
) -> Result<Dataset, DataParsingError> {
    let options = match sheet_name {
        Some(name) => {
            project_compatibility_read_options().with_sheet(SheetSelector::Name(name.to_string()))
        }
        None => project_compatibility_read_options(),
    };
    electrodata_io::read_with_options(path, &options).map_err(Into::into)
}

/// Converts a canonical dataset to the aligned project time-series domain.
impl TryFrom<&Dataset> for MultiChannelMeasurement {
    type Error = DataParsingError;

    fn try_from(dataset: &Dataset) -> Result<Self, Self::Error> {
        let view = dataset.time_series_view()?;
        // The domain boundary preserves the decoded source coordinate exactly.
        // Normalization to seconds belongs to the scientific algorithm that
        // explicitly requires it, together with its recorded conversion.
        let time = view
            .coordinate_values()
            .iter()
            .enumerate()
            .map(|(row, value)| {
                (*value).ok_or_else(|| {
                    DataParsingError::invalid(format!(
                        "canonical time-series view retained a missing time value at row {}",
                        dataset.source_row(row).unwrap_or(row + 1)
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let coordinate_unit = unit_label(view.coordinate_unit());
        let coordinate_name = view.time.original_name.clone();
        let channels = view
            .measurements
            .into_iter()
            .map(channel_from_canonical)
            .collect::<Vec<_>>();
        MultiChannelMeasurement::new_with_coordinate(
            time,
            coordinate_unit,
            coordinate_name,
            channels,
        )
    }
}

/// Converts canonical diagnostics into the project's analysis-facing summary.
pub fn measurement_parse_result(
    dataset: &Dataset,
) -> Result<MeasurementParseResult, DataParsingError> {
    let measurement = MultiChannelMeasurement::try_from(dataset)?;
    let summary = dataset.diagnostic_summary();
    let mut diagnostics = ParseDiagnostics::from_measurement(&measurement);
    diagnostics.total_rows = summary.total_source_rows;
    diagnostics.successfully_parsed_rows = summary.retained_rows;
    diagnostics.skipped_rows = summary.skipped_rows;
    diagnostics.malformed_rows = summary.malformed_rows;
    diagnostics.missing_values = summary.missing_cells;
    diagnostics.messages.extend(
        dataset
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone()),
    );
    if let Some(sheet) = &dataset.metadata.provenance.worksheet {
        diagnostics
            .messages
            .push(format!("worksheet selected: '{sheet}'"));
    }
    diagnostics
        .messages
        .push(format!("parser kind: electrodata-io ({})", dataset.format));
    Ok(MeasurementParseResult {
        measurement,
        diagnostics,
    })
}

/// Converts canonical EIS roles while retaining source-measured optional
/// magnitude/phase separately from domain values derived for legacy fitting.
impl TryFrom<&Dataset> for EISData {
    type Error = DataParsingError;

    fn try_from(dataset: &Dataset) -> Result<Self, Self::Error> {
        let view = dataset.eis_view()?;
        let frequency = required_values(view.frequency, "frequency")?;
        let z_re = required_values(view.real, "real impedance")?;
        let z_im = required_values(view.imaginary, "imaginary impedance")?;
        let measured_magnitude = view.measured_magnitude.map(|column| column.values.clone());
        let measured_phase = view.measured_phase.map(|column| column.values.clone());
        let derived_magnitude: Vec<f64> = z_re
            .iter()
            .zip(&z_im)
            .map(|(real, imaginary)| real.hypot(*imaginary))
            .collect();
        let derived_phase: Vec<f64> = z_re
            .iter()
            .zip(&z_im)
            .map(|(real, imaginary)| imaginary.atan2(*real).to_degrees())
            .collect();
        // The pre-existing fitting domain requires a complete phase vector.
        // Preserve source phase independently and derive only missing/absent
        // values for that legacy analysis convenience field.
        let phase = derived_phase
            .iter()
            .enumerate()
            .map(|(index, derived)| {
                measured_phase
                    .as_ref()
                    .and_then(|values| values.get(index).copied().flatten())
                    .unwrap_or(*derived)
            })
            .collect();
        let metadata = metadata_entries(dataset);
        Ok(EISData {
            date: dataset
                .metadata
                .acquisition
                .recorded_at
                .map(|value| value.to_string())
                .unwrap_or_default(),
            test_type: dataset
                .metadata
                .acquisition
                .technique
                .clone()
                .unwrap_or_default(),
            instrument_model: dataset
                .metadata
                .acquisition
                .instrument_model
                .clone()
                .unwrap_or_default(),
            freq: frequency,
            phase,
            z_re,
            z_im,
            measured_magnitude,
            measured_phase,
            derived_magnitude,
            derived_phase,
            label: dataset.metadata.provenance.source_name.clone(),
            metadata,
            circuit_model: String::new(),
        })
    }
}

fn channel_from_canonical(column: &Column) -> MeasurementChannel {
    MeasurementChannel::new(
        column
            .original_name
            .clone()
            .unwrap_or_else(|| column.name.clone()),
        unit_label(column.unit.as_ref()),
        column.values.clone(),
    )
}

fn required_values(column: &Column, field: &str) -> Result<Vec<f64>, DataParsingError> {
    column
        .values
        .iter()
        .enumerate()
        .map(|(row, value)| {
            value.ok_or_else(|| {
                DataParsingError::invalid(format!("{field} is missing at retained row {}", row + 1))
            })
        })
        .collect()
}

fn metadata_entries(dataset: &Dataset) -> BTreeMap<String, String> {
    let mut entries = dataset
        .metadata
        .parameters
        .iter()
        .map(|parameter| {
            (
                normalize_metadata_key(&parameter.raw_name),
                parameter.raw_value.clone(),
            )
        })
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
        .collect::<BTreeMap<_, _>>();
    entries.extend(
        dataset
            .metadata
            .raw_rows
            .iter()
            .filter_map(|row| {
                row.reconstructed_text
                    .split_once(':')
                    .or_else(|| row.reconstructed_text.split_once('='))
            })
            .map(|(key, value)| (normalize_metadata_key(key), value.trim().to_string()))
            .filter(|(key, value)| !key.is_empty() && !value.is_empty()),
    );
    entries
}

fn normalize_metadata_key(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect()
}

fn unit_label(unit: Option<&electrodata_io::Unit>) -> String {
    match unit.unwrap_or(&electrodata_io::Unit::Unknown) {
        electrodata_io::Unit::Unknown => String::new(),
        electrodata_io::Unit::Second => "s".to_string(),
        electrodata_io::Unit::Hour => "h".to_string(),
        electrodata_io::Unit::Day => "day".to_string(),
        electrodata_io::Unit::Hertz => "Hz".to_string(),
        electrodata_io::Unit::Ohm => "ohm".to_string(),
        electrodata_io::Unit::Degree => "deg".to_string(),
        electrodata_io::Unit::Volt => "V".to_string(),
        electrodata_io::Unit::Millivolt => "mV".to_string(),
        electrodata_io::Unit::Ampere => "A".to_string(),
        electrodata_io::Unit::Milliampere => "mA".to_string(),
        electrodata_io::Unit::Microampere => "uA".to_string(),
        electrodata_io::Unit::Other(value) => value.clone(),
    }
}

/// Returns whether a dataset is canonically recognized as EIS without raw
/// header probing.  Kept public for dispatch-only callers.
pub fn is_eis(dataset: &Dataset) -> bool {
    matches!(
        dataset.kind(),
        electrodata_io::DatasetKind::ImpedanceSpectrum
    ) || dataset.canonical_column(&ColumnRole::Frequency).is_ok()
}
