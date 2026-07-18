//! Generic time-series parsing into the Phase 1 scientific model.

use crate::domain::{
    DataParsingError, MeasurementChannel, MeasurementParseResult, MultiChannelMeasurement,
    ParseDiagnostics,
};
use std::fs;
use std::path::Path;

/// Parse a CSV/TXT/DAT time-series file with one time column and one or more
/// named numeric channels.
///
/// Binary files are rejected by the centralized `load_data` entrypoint before
/// reaching this function, but a content-level guard is added here as a
/// defence-in-depth measure.
pub fn parse_measurement_file(
    path: impl AsRef<Path>,
) -> Result<MeasurementParseResult, DataParsingError> {
    let path = path.as_ref();

    // Defence-in-depth binary guard.
    let kind = crate::data_file::InputKind::classify_by_extension(path);
    if kind.is_unsupported_binary() {
        return Err(DataParsingError::invalid_at(
            path,
            format!(
                "Unsupported input file '{}': binary input is not supported. \
                 Export the dataset as CSV, XLSX, or another documented text-based format.",
                path.display()
            ),
        ));
    }

    let text = fs::read_to_string(path).map_err(|error| DataParsingError::io(path, error))?;
    parse_measurement_text(&text, path)
}

/// Parse a CHI-style or generic time-series text buffer.  Metadata/preamble
/// lines are ignored until a time-oriented header is found.
pub fn parse_measurement_text(
    text: &str,
    source: impl AsRef<Path>,
) -> Result<MeasurementParseResult, DataParsingError> {
    let source = source.as_ref();
    let lines = text.lines().map(str::trim).collect::<Vec<_>>();
    let (header_index, time_index, headers) = find_time_header(&lines)
        .ok_or_else(|| DataParsingError::invalid_at(source, "missing time-series header"))?;
    let channel_indices = headers
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != time_index)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let channel_headers = headers
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != time_index)
        .map(|(_, header)| parse_channel_header(header))
        .collect::<Vec<_>>();

    if channel_headers.is_empty() {
        return Err(DataParsingError::invalid_at(
            source,
            "time-series header does not contain any measurement channels",
        ));
    }

    let mut time = Vec::new();
    let mut channel_values = vec![Vec::new(); channel_headers.len()];
    let mut diagnostics = ParseDiagnostics::default();

    for (line_number, line) in lines.iter().enumerate().skip(header_index + 1) {
        if line.is_empty() {
            continue;
        }
        diagnostics.total_rows += 1;
        let fields = split_csv(line);
        let timestamp = fields
            .get(time_index)
            .and_then(|value| parse_optional_number(value));

        let Some(timestamp) = timestamp else {
            diagnostics.skipped_rows += 1;
            diagnostics.malformed_rows += 1;
            diagnostics.messages.push(format!(
                "row {} skipped: timestamp is missing or malformed",
                line_number + 1
            ));
            continue;
        };

        time.push(timestamp);
        diagnostics.successfully_parsed_rows += 1;
        let mut row_malformed = false;

        for (channel_index, &header_index) in channel_indices.iter().enumerate() {
            let value = fields
                .get(header_index)
                .and_then(|value| parse_optional_number(value));
            if value.is_none() {
                diagnostics.missing_values += 1;
                if fields
                    .get(header_index)
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    row_malformed = true;
                    diagnostics.messages.push(format!(
                        "row {} channel '{}' is malformed",
                        line_number + 1,
                        channel_headers[channel_index].0
                    ));
                }
            }
            channel_values[channel_index].push(value);
        }

        if fields.len() < headers.len() {
            row_malformed = true;
            diagnostics.messages.push(format!(
                "row {} is incomplete: expected {} columns, got {}",
                line_number + 1,
                headers.len(),
                fields.len()
            ));
        } else if fields.len() > headers.len() {
            row_malformed = true;
            diagnostics.messages.push(format!(
                "row {} contains {} extra columns",
                line_number + 1,
                fields.len() - headers.len()
            ));
        }

        if row_malformed {
            diagnostics.malformed_rows += 1;
        }
    }

    if time.is_empty() {
        return Err(DataParsingError::invalid_at(
            source,
            "no valid time-series rows were found",
        ));
    }

    let channels = channel_headers
        .into_iter()
        .zip(channel_values)
        .map(|((name, unit), values)| MeasurementChannel::new(name, unit, values))
        .collect::<Vec<_>>();
    let measurement = MultiChannelMeasurement::new(time, channels)?;
    diagnostics.update_time_axis(&measurement.time);

    Ok(MeasurementParseResult {
        measurement,
        diagnostics,
    })
}

fn find_time_header<'a>(lines: &[&'a str]) -> Option<(usize, usize, Vec<&'a str>)> {
    lines.iter().enumerate().find_map(|(line_index, line)| {
        let fields = split_csv(line);
        if fields.len() < 2 {
            return None;
        }
        let time_index = fields.iter().position(|field| is_time_header(field))?;
        let headers = fields;
        Some((line_index, time_index, headers))
    })
}

fn is_time_header(value: &str) -> bool {
    let normalized = normalize_header(value);
    normalized == "time"
        || normalized.starts_with("time/")
        || normalized.starts_with("time(")
        || normalized.starts_with("time[")
        || normalized == "timestamp"
        || normalized.starts_with("timestamp/")
        || normalized.starts_with("timestamp(")
        || normalized.starts_with("timestamp[")
}

fn parse_channel_header(header: &str) -> (String, String) {
    let header = header.trim();
    if let Some((name, unit)) = header.rsplit_once('/') {
        return (name.trim().to_string(), unit.trim().to_string());
    }
    if let Some(open) = header.rfind('(')
        && header.ends_with(')')
    {
        return (
            header[..open].trim().to_string(),
            header[open + 1..header.len() - 1].trim().to_string(),
        );
    }
    (header.to_string(), String::new())
}

fn normalize_header(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect()
}

fn split_csv(line: &str) -> Vec<&str> {
    line.split(',').map(str::trim).collect()
}

fn parse_optional_number(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty()
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "na" | "n/a" | "nan" | "null" | "missing"
        )
    {
        return None;
    }
    value.parse::<f64>().ok().filter(|value| value.is_finite())
}

/// Load metadata and a measurement file into one experiment object.
pub fn load_experiment(
    measurement_path: impl AsRef<Path>,
    metadata_path: impl AsRef<Path>,
) -> Result<(crate::domain::ElectrochemicalExperiment, ParseDiagnostics), DataParsingError> {
    let measurement_path = measurement_path.as_ref();
    let metadata_path = metadata_path.as_ref();
    let parsed = parse_measurement_file(measurement_path)?;
    let metadata = crate::domain::load_experiment_metadata(metadata_path)?;
    let diagnostics = parsed.diagnostics.clone();
    let experiment = crate::domain::metadata::build_experiment(
        metadata,
        parsed.measurement,
        measurement_path,
        metadata_path,
    )?;
    Ok((experiment, diagnostics))
}

#[cfg(test)]
mod tests {
    use super::parse_measurement_text;

    #[test]
    fn parses_single_and_multi_channel_rows_with_missing_values() {
        let parsed =
            parse_measurement_text("time/sec,Potential/V\n0,0.1\n1,NA\n2,0.3\n", "fixture.csv")
                .expect("parse measurement");

        assert_eq!(parsed.measurement.channels.len(), 1);
        assert_eq!(parsed.measurement.channels[0].values[1], None);
        assert_eq!(parsed.diagnostics.missing_values, 1);
        assert_eq!(parsed.diagnostics.successfully_parsed_rows, 3);
    }
}
