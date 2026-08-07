#![allow(dead_code)]

//! Legacy raw-reader compatibility implementation retained only for parity
//! verification during the electrodata-io migration.
//!
//! Production workflows must use `electrodata_domain_adapter`, whose typed
//! conversion path never accesses `Dataset::data` or registers local format
//! detection.  This module deliberately remains until parity review authorizes
//! removal of the historical project parser.

use crate::domain::{
    DataParsingError, MeasurementChannel, MeasurementParseResult, MultiChannelMeasurement,
    ParseDiagnostics,
};
use electrodata_io::{
    ColumnDescriptor, ColumnRole, DataReader, Dataset, DetectedLayout, Detection, FormatHandler,
    FormatId, Probe, ReadOptions, SheetSelector, Unit,
};
use std::path::Path;

const PROJECT_TABULAR_FORMAT: &str = "rust-electroanalysis-tabular";

/// Handles the wider time-series tables and compact four-column EIS tables
/// accepted by the project before it adopted `electrodata-io`. Built-in crate
/// handlers retain priority for all formats they recognize.
struct ProjectTabularHandler;

impl FormatHandler for ProjectTabularHandler {
    fn format_id(&self) -> FormatId {
        FormatId::Custom(PROJECT_TABULAR_FORMAT.to_string())
    }

    fn detect(&self, probe: &Probe, _options: &ReadOptions) -> Option<Detection> {
        for (header_row, row) in probe.rows.iter().enumerate() {
            if is_compact_eis_header(row) {
                let data_start_row = first_numeric_row(probe, header_row + 1, row.len())?;
                return Some(Detection {
                    format: self.format_id(),
                    confidence: 0.90,
                    evidence: vec!["project-compatible four-column EIS table".to_string()],
                    layout: DetectedLayout {
                        header_row: Some(header_row),
                        data_start_row,
                        metadata_range: (header_row > 0).then_some(0..header_row),
                        expected_column_count: row.len(),
                        has_header: true,
                    },
                });
            }

            let Some(time_index) = row.iter().position(|value| is_time_header(value)) else {
                continue;
            };
            if row.len() < 2 {
                continue;
            }
            let data_start_row = first_numeric_row(probe, header_row + 1, row.len())?;
            let confidence = if row.len() > 2 { 0.85 } else { 0.65 };
            return Some(Detection {
                format: self.format_id(),
                confidence,
                evidence: vec![format!(
                    "project-compatible time-series table with time column {}",
                    time_index + 1
                )],
                layout: DetectedLayout {
                    header_row: Some(header_row),
                    data_start_row,
                    metadata_range: (header_row > 0).then_some(0..header_row),
                    expected_column_count: row.len(),
                    has_header: true,
                },
            });
        }
        None
    }

    fn columns(
        &self,
        probe: &Probe,
        detection: &Detection,
    ) -> electrodata_io::Result<Vec<ColumnDescriptor>> {
        let header = probe
            .row(
                detection
                    .layout
                    .header_row
                    .ok_or(electrodata_io::Error::HeaderNotFound)?,
            )
            .ok_or(electrodata_io::Error::HeaderNotFound)?;

        if is_compact_eis_header(header) {
            let roles = vec![
                ("frequency_hz", ColumnRole::Frequency, Unit::Hertz),
                ("z_real_ohm", ColumnRole::ImpedanceReal, Unit::Ohm),
                ("z_imag_ohm", ColumnRole::ImpedanceImaginary, Unit::Ohm),
            ];
            let roles = if header.len() == 4 {
                roles
                    .into_iter()
                    .chain(std::iter::once((
                        "phase_deg",
                        ColumnRole::Phase,
                        Unit::Degree,
                    )))
                    .collect::<Vec<_>>()
            } else {
                roles
            };
            return Ok(header
                .iter()
                .zip(roles)
                .map(
                    |(source_name, (canonical_name, role, unit))| ColumnDescriptor {
                        source_name: source_name.trim().to_string(),
                        canonical_name: canonical_name.to_string(),
                        role,
                        unit: Some(unit),
                    },
                )
                .collect());
        }

        Ok(header
            .iter()
            .enumerate()
            .map(|(index, source_name)| ColumnDescriptor {
                source_name: source_name.trim().to_string(),
                canonical_name: if is_time_header(source_name) {
                    "time".to_string()
                } else {
                    format!("channel_{index}")
                },
                role: if is_time_header(source_name) {
                    ColumnRole::Time
                } else {
                    ColumnRole::Custom(format!("channel:{index}"))
                },
                unit: unit_from_header(source_name),
            })
            .collect())
    }
}

pub(crate) fn read_dataset(
    path: &Path,
    sheet_name: Option<&str>,
) -> Result<Dataset, DataParsingError> {
    let reader = DataReader::builder()
        .with_builtin_formats()
        .register(ProjectTabularHandler)
        .build();
    let result = if let Some(sheet_name) = sheet_name {
        let options = ReadOptions::new().with_sheet(SheetSelector::Name(sheet_name.to_string()));
        reader.read_with_options(path, &options)
    } else {
        reader.read(path)
    };
    result.map_err(|error| map_read_error(path, error))
}

pub(crate) fn measurement_from_dataset(
    dataset: &Dataset,
    source: &Path,
) -> Result<MeasurementParseResult, DataParsingError> {
    if dataset
        .columns
        .iter()
        .any(|column| column.role == ColumnRole::Frequency)
    {
        let message = if matches!(
            dataset.metadata.provenance.container,
            electrodata_io::ContainerFormat::Xlsx
                | electrodata_io::ContainerFormat::Xls
                | electrodata_io::ContainerFormat::Xlsb
                | electrodata_io::ContainerFormat::Ods
        ) {
            "worksheet is EIS-like and cannot be used by this time-series workflow"
        } else {
            "EIS data cannot be loaded through the time-series measurement workflow"
        };
        return Err(DataParsingError::invalid_at(source, message));
    }

    let time_descriptor = dataset
        .columns
        .iter()
        .find(|column| matches!(column.role, ColumnRole::Time | ColumnRole::X))
        .ok_or_else(|| DataParsingError::invalid_at(source, "missing time-series header"))?;
    let time_values = descriptor_values(dataset, time_descriptor, source)?;
    let mut retained_rows = Vec::with_capacity(time_values.len());
    let mut time = Vec::with_capacity(time_values.len());
    for (index, value) in time_values.into_iter().enumerate() {
        if let Some(value) = value {
            retained_rows.push(index);
            time.push(value);
        }
    }
    if time.is_empty() {
        return Err(DataParsingError::invalid_at(
            source,
            "no valid time-series rows were found",
        ));
    }

    let channel_descriptors = dataset
        .columns
        .iter()
        .filter(|column| !matches!(column.role, ColumnRole::Time | ColumnRole::X))
        .collect::<Vec<_>>();
    if channel_descriptors.is_empty() {
        return Err(DataParsingError::invalid_at(
            source,
            "time-series header does not contain any measurement channels",
        ));
    }

    let mut channels = Vec::with_capacity(channel_descriptors.len());
    for descriptor in channel_descriptors {
        let values = descriptor_values(dataset, descriptor, source)?;
        let values = retained_rows
            .iter()
            .map(|index| values.get(*index).copied().flatten())
            .collect();
        let (name, header_unit) = parse_channel_header(&descriptor.source_name);
        let unit = header_unit.or_else(|| descriptor.unit.as_ref().map(unit_label));
        channels.push(MeasurementChannel::new(
            if name.is_empty() {
                descriptor.canonical_name.clone()
            } else {
                name
            },
            unit.unwrap_or_default(),
            values,
        ));
    }

    let measurement = MultiChannelMeasurement::new(time, channels)?;
    let mut diagnostics = ParseDiagnostics::from_measurement(&measurement);
    diagnostics.total_rows = dataset.data.height();
    diagnostics.skipped_rows = dataset.data.height().saturating_sub(retained_rows.len());
    diagnostics.malformed_rows = diagnostics.skipped_rows;
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

pub(crate) fn descriptor_values(
    dataset: &Dataset,
    descriptor: &ColumnDescriptor,
    source: &Path,
) -> Result<Vec<Option<f64>>, DataParsingError> {
    let column = dataset
        .data
        .column(&descriptor.canonical_name)
        .or_else(|_| dataset.data.column(descriptor.source_name.trim()))
        .map_err(|error| DataParsingError::invalid_at(source, error.to_string()))?;
    let values = column
        .f64()
        .map_err(|error| DataParsingError::invalid_at(source, error.to_string()))?;
    Ok((0..values.len()).map(|index| values.get(index)).collect())
}

#[allow(dead_code)]
pub(crate) fn role_values(
    dataset: &Dataset,
    role: &ColumnRole,
    source: &Path,
) -> Result<Vec<f64>, DataParsingError> {
    let descriptor = dataset
        .columns
        .iter()
        .find(|descriptor| &descriptor.role == role)
        .ok_or_else(|| {
            DataParsingError::invalid_at(source, format!("missing required column role {role:?}"))
        })?;
    descriptor_values(dataset, descriptor, source)?
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            DataParsingError::invalid_at(
                source,
                format!(
                    "required column '{}' contains missing values",
                    descriptor.source_name
                ),
            )
        })
}

#[allow(dead_code)]
pub(crate) fn metadata_preamble(dataset: &Dataset) -> (String, String, String) {
    let date = dataset
        .metadata
        .raw_rows
        .first()
        .map(|row| row.reconstructed_text.clone())
        .unwrap_or_default();
    let test_type = dataset
        .metadata
        .raw_rows
        .get(1)
        .map(|row| row.reconstructed_text.clone())
        .or_else(|| dataset.metadata.acquisition.technique.clone())
        .unwrap_or_default();
    let instrument_model = dataset
        .metadata
        .acquisition
        .instrument_model
        .clone()
        .unwrap_or_default();
    (date, test_type, instrument_model)
}

fn map_read_error(path: &Path, error: electrodata_io::Error) -> DataParsingError {
    match error {
        electrodata_io::Error::Io { source, .. } => DataParsingError::io(path, source),
        electrodata_io::Error::InvalidUtf8 { .. } => {
            DataParsingError::invalid_at(path, "unsupported binary content or invalid UTF-8 input")
        }
        electrodata_io::Error::AmbiguousFormat { .. }
        | electrodata_io::Error::AmbiguousDetection { .. } => DataParsingError::invalid_at(
            path,
            format!(
                "multiple compatible time-series worksheets or formats were found; specify --sheet <NAME> when reading a workbook ({error})"
            ),
        ),
        other => DataParsingError::invalid_at(path, other.to_string()),
    }
}

fn first_numeric_row(probe: &Probe, start: usize, width: usize) -> Option<usize> {
    probe
        .rows
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, row)| {
            row.len() == width
                && row.iter().any(|cell| !cell.trim().is_empty())
                && row
                    .iter()
                    .all(|cell| cell.trim().is_empty() || cell.trim().parse::<f64>().is_ok())
        })
        .map(|(index, _)| index)
}

fn is_compact_eis_header(row: &[String]) -> bool {
    if !matches!(row.len(), 3 | 4) {
        return false;
    }
    let keys = row
        .iter()
        .map(|value| header_key(value))
        .collect::<Vec<_>>();
    matches!(keys[0].as_str(), "freq/hz" | "frequency/hz")
        && matches!(keys[1].as_str(), "z'/ohm" | "zreal/ohm")
        && matches!(keys[2].as_str(), "z\"/ohm" | "zimag/ohm")
        && (row.len() == 3 || matches!(keys[3].as_str(), "phase/deg" | "phase/degree"))
}

fn is_time_header(value: &str) -> bool {
    let normalized = header_key(value);
    normalized == "time"
        || normalized.starts_with("time/")
        || normalized.starts_with("time(")
        || normalized.starts_with("time[")
        || normalized == "timestamp"
        || normalized.starts_with("timestamp/")
        || normalized.starts_with("timestamp(")
        || normalized.starts_with("timestamp[")
}

fn header_key(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect()
}

fn unit_from_header(header: &str) -> Option<Unit> {
    let (_, unit) = parse_channel_header(header);
    unit.map(|unit| match unit.to_ascii_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Unit::Second,
        "v" | "volt" | "volts" => Unit::Volt,
        "hz" => Unit::Hertz,
        "ohm" | "ohms" => Unit::Ohm,
        "deg" | "degree" | "degrees" => Unit::Degree,
        _ => Unit::Other(unit),
    })
}

fn parse_channel_header(header: &str) -> (String, Option<String>) {
    let header = header.trim();
    if let Some((name, unit)) = header.rsplit_once('/') {
        return (name.trim().to_string(), Some(unit.trim().to_string()));
    }
    if let Some(open) = header.rfind('(')
        && header.ends_with(')')
    {
        return (
            header[..open].trim().to_string(),
            Some(header[open + 1..header.len() - 1].trim().to_string()),
        );
    }
    (header.to_string(), None)
}

fn unit_label(unit: &Unit) -> String {
    match unit {
        Unit::Unknown => String::new(),
        Unit::Second => "s".to_string(),
        Unit::Hour => "h".to_string(),
        Unit::Day => "day".to_string(),
        Unit::Volt => "V".to_string(),
        Unit::Millivolt => "mV".to_string(),
        Unit::Ampere => "A".to_string(),
        Unit::Milliampere => "mA".to_string(),
        Unit::Microampere => "uA".to_string(),
        Unit::Hertz => "Hz".to_string(),
        Unit::Ohm => "ohm".to_string(),
        Unit::Degree => "deg".to_string(),
        Unit::Other(value) => value.clone(),
    }
}
