//! Canonical physical-input to project-domain conversion.
//!
//! `electrodata-io` owns container detection, parsing, schema recognition,
//! recovery policy, worksheet selection, and input diagnostics.  This module
//! deliberately performs only domain conversion and does not inspect raw
//! headers or the compatibility Polars representation.

use crate::data_file::chi_file::EISData;
use crate::domain::{
    DataParsingError, IngestionDiagnostic, MeasurementChannel, MeasurementParseResult,
    MultiChannelMeasurement, ParseDiagnostics,
};
use electrodata_io::{
    Column, ColumnNamePolicy, ColumnRole, CoordinateOrderPolicy, Dataset, HeaderPolicy,
    InvalidCellPolicy, InvalidTimePolicy, RaggedRowPolicy, ReadOptions, ReadProfile, SheetSelector,
    ValidationLevel,
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
    // Do not inherit scientifically relevant compatibility behavior from the
    // provider profile: every recovery and interpretation policy is locked
    // here as part of the consumer migration contract.
    let mut options = ReadOptions::new()
        .with_profile(ReadProfile::Compatibility)
        .with_invalid_time_policy(InvalidTimePolicy::SkipRow)
        .with_invalid_cell_policy(InvalidCellPolicy::Null)
        .with_ragged_row_policy(RaggedRowPolicy::PadNulls)
        .with_coordinate_order_policy(CoordinateOrderPolicy::Preserve)
        .with_header_policy(HeaderPolicy::Auto)
        .with_column_name_policy(ColumnNamePolicy::Canonical)
        .with_sheet(SheetSelector::Auto)
        .with_validation(ValidationLevel::Structural);
    // Lossy decoding is a scientific-input decision: reject undecodable text
    // rather than silently replacing bytes.
    options.allow_lossy_utf8 = false;
    options
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
    diagnostics.ingestion_diagnostics = dataset
        .diagnostics
        .iter()
        .map(|diagnostic| IngestionDiagnostic {
            code: format!("{:?}", diagnostic.code),
            recovery: format!("{:?}", diagnostic.recovery),
            message: diagnostic.message.clone(),
            row: diagnostic.row,
            column: diagnostic.column.clone(),
            column_index: diagnostic.column_index,
        })
        .collect();
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
    let source_header = column
        .original_name
        .clone()
        .unwrap_or_else(|| column.name.clone());
    let unit = unit_label(column.unit.as_ref());
    let logical_name = logical_channel_name(&source_header, &unit);
    MeasurementChannel::new(logical_name, unit, column.values.clone())
        .with_source_header(source_header)
}

/// Derives the analysis-facing channel identity only when the canonical unit
/// metadata verifies that the final slash-delimited token is a known unit
/// annotation.  This deliberately does not split arbitrary source headers.
fn logical_channel_name(source_header: &str, canonical_unit: &str) -> String {
    let Some((name, suffix)) = source_header.rsplit_once('/') else {
        return source_header.to_string();
    };
    let name = name.trim();
    let suffix = suffix.trim();
    if !name.is_empty() && is_verified_unit_suffix(suffix, canonical_unit) {
        name.to_string()
    } else {
        source_header.to_string()
    }
}

fn is_verified_unit_suffix(suffix: &str, canonical_unit: &str) -> bool {
    matches!(
        canonical_unit,
        "s" | "h" | "day" | "Hz" | "ohm" | "deg" | "V" | "mV" | "A" | "mA" | "uA"
    ) && suffix.eq_ignore_ascii_case(canonical_unit)
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

#[cfg(test)]
mod tests {
    use super::logical_channel_name;

    #[test]
    fn strips_only_a_verified_canonical_unit_suffix() {
        assert_eq!(logical_channel_name("E1/V", "V"), "E1");
        assert_eq!(logical_channel_name("NH4/mV", "mV"), "NH4");
        assert_eq!(logical_channel_name("ORP", "V"), "ORP");
        assert_eq!(logical_channel_name("NO3_sensor/A", "A"), "NO3_sensor");
        assert_eq!(logical_channel_name("NO3_sensor/A", "V"), "NO3_sensor/A");
        assert_eq!(logical_channel_name("path/segment", ""), "path/segment");
    }
}
