//! Reference implementation copied from commit cc6f283 for migration parity only.
//! Never used by production.
//!
//! This deliberately has no `electrodata_io` dependency and does not install
//! any provider handlers. It preserves only the archived parsing behavior
//! required by `io_migration_parity`.

use calamine::{DataType, Reader, open_workbook_auto};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub name: String,
    pub unit: String,
    pub values: Vec<Option<f64>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeSeries {
    pub raw_time: Vec<f64>,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Eis {
    pub frequency: Vec<f64>,
    pub real: Vec<f64>,
    pub imaginary: Vec<f64>,
    pub phase: Vec<f64>,
    pub measured_magnitude: Option<Vec<f64>>,
    pub measured_phase: Option<Vec<f64>>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Dataset {
    TimeSeries(TimeSeries),
    Eis(Eis),
}

pub fn read(path: &Path) -> Result<Dataset, String> {
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xlsx"))
    {
        return read_xlsx(path).map(Dataset::TimeSeries);
    }
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let lines = text.lines().map(str::trim).collect::<Vec<_>>();
    if lines.iter().any(|line| line.starts_with("Freq/Hz")) {
        read_eis(&lines).map(Dataset::Eis)
    } else {
        read_time_series(&lines).map(Dataset::TimeSeries)
    }
}

fn read_time_series(lines: &[&str]) -> Result<TimeSeries, String> {
    let (header_index, headers) = lines
        .iter()
        .enumerate()
        .find_map(|(index, line)| {
            let fields = split_csv(line);
            (fields.len() >= 2 && fields.iter().any(|field| is_time_header(field)))
                .then_some((index, fields))
        })
        .ok_or_else(|| "missing time-series header".to_string())?;
    let time_index = headers
        .iter()
        .position(|header| is_time_header(header))
        .ok_or_else(|| "missing time-series header".to_string())?;
    let channels = headers
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != time_index)
        .map(|(_, header)| {
            let (name, unit) = parse_channel_header(header);
            Channel {
                name,
                unit,
                values: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    if channels.is_empty() {
        return Err("time-series header does not contain any measurement channels".to_string());
    }

    let mut result = TimeSeries {
        raw_time: Vec::new(),
        channels,
    };
    for line in lines
        .iter()
        .skip(header_index + 1)
        .filter(|line| !line.is_empty())
    {
        let fields = split_csv(line);
        let Some(timestamp) = fields.get(time_index).and_then(|value| parse_number(value)) else {
            continue;
        };
        result.raw_time.push(timestamp);
        for (channel_index, source_index) in headers
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != time_index)
            .map(|(index, _)| index)
            .enumerate()
        {
            result.channels[channel_index].values.push(
                fields
                    .get(source_index)
                    .and_then(|value| parse_number(value)),
            );
        }
    }
    (!result.raw_time.is_empty())
        .then_some(result)
        .ok_or_else(|| "no valid time-series rows were found".to_string())
}

fn read_eis(lines: &[&str]) -> Result<Eis, String> {
    let header_index = lines
        .iter()
        .position(|line| line.starts_with("Freq/Hz"))
        .ok_or_else(|| "missing Freq/Hz header".to_string())?;
    let headers = split_csv(lines[header_index]);
    let column = |name: &str| {
        headers
            .iter()
            .position(|field| normalize_header(field) == name)
    };
    let frequency_index = column("freq/hz").unwrap_or(0);
    let real_index = column("z'/ohm").unwrap_or(1);
    let imaginary_index = column("z\"/ohm").unwrap_or(2);
    let phase_index = column("phase/deg").unwrap_or(4);
    let magnitude_index = column("z/ohm");
    let mut result = Eis {
        frequency: Vec::new(),
        real: Vec::new(),
        imaginary: Vec::new(),
        phase: Vec::new(),
        measured_magnitude: magnitude_index.map(|_| Vec::new()),
        measured_phase: Some(Vec::new()),
        metadata: metadata(lines, header_index),
    };
    for line in lines
        .iter()
        .skip(header_index + 1)
        .filter(|line| !line.is_empty())
    {
        let fields = split_csv(line);
        let required = [frequency_index, real_index, imaginary_index, phase_index];
        if required.iter().any(|index| fields.len() <= *index) {
            continue;
        }
        let Some(frequency) = parse_number(fields[frequency_index]) else {
            continue;
        };
        let Some(real) = parse_number(fields[real_index]) else {
            continue;
        };
        let Some(imaginary) = parse_number(fields[imaginary_index]) else {
            continue;
        };
        let Some(phase) = parse_number(fields[phase_index]) else {
            continue;
        };
        result.frequency.push(frequency);
        result.real.push(real);
        result.imaginary.push(imaginary);
        result.phase.push(phase);
        if let Some(index) = magnitude_index
            && let Some(magnitude) = fields.get(index).and_then(|value| parse_number(value))
            && let Some(values) = &mut result.measured_magnitude
        {
            values.push(magnitude);
        }
        if let Some(values) = &mut result.measured_phase {
            values.push(phase);
        }
    }
    (!result.frequency.is_empty())
        .then_some(result)
        .ok_or_else(|| "no numeric EIS rows found".to_string())
}

fn read_xlsx(path: &Path) -> Result<TimeSeries, String> {
    let mut workbook = open_workbook_auto(path).map_err(|error| error.to_string())?;
    let sheet = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "workbook contains no worksheets".to_string())?;
    let range = workbook
        .worksheet_range(&sheet)
        .map_err(|error| error.to_string())?;
    let lines = range
        .rows()
        .map(|row| {
            row.iter()
                .map(|cell| cell.as_string().unwrap_or_default())
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>();
    let borrowed = lines.iter().map(String::as_str).collect::<Vec<_>>();
    read_time_series(&borrowed)
}

fn metadata(lines: &[&str], stop: usize) -> BTreeMap<String, String> {
    lines[..stop]
        .iter()
        .filter_map(|line| line.split_once(':').or_else(|| line.split_once('=')))
        .map(|(key, value)| (normalize_header(key), value.trim().to_string()))
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
        .collect()
}

fn split_csv(line: &str) -> Vec<&str> {
    line.split(',').map(str::trim).collect()
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

fn is_time_header(value: &str) -> bool {
    let value = normalize_header(value);
    value == "time"
        || value.starts_with("time/")
        || value.starts_with("time(")
        || value == "timestamp"
        || value.starts_with("timestamp/")
        || value.starts_with("timestamp(")
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

fn parse_number(value: &str) -> Option<f64> {
    let value = value.trim();
    (!value.is_empty()
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "na" | "n/a" | "nan" | "null" | "missing"
        ))
    .then(|| value.parse::<f64>().ok().filter(|value| value.is_finite()))
    .flatten()
}
