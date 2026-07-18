//! Excel workbook parser integrated into the shared data-file architecture.

use crate::domain::{DataParsingError, MeasurementParseResult};
use calamine::{Data, Reader, open_workbook_auto};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExcelTable {
    pub source_path: String,
    pub sheet_name: String,
    pub header_row_index: usize,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub rows_skipped_before_header: usize,
    pub unit_row_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ExcelMeasurementParseResult {
    pub parsed: MeasurementParseResult,
    pub sheet_name: String,
    pub header_row_index: usize,
    pub rows_skipped_before_header: usize,
    pub unit_row_index: Option<usize>,
}

#[derive(Debug, Clone)]
struct SheetCandidate {
    name: String,
    table: Option<ExcelTable>,
    non_empty: bool,
    eis_like_only: bool,
}

pub fn parse_excel_measurement(
    path: impl AsRef<Path>,
    sheet_name: Option<&str>,
) -> Result<ExcelMeasurementParseResult, DataParsingError> {
    let path = path.as_ref();
    let table = read_worksheet(path, sheet_name)?;
    let parsed = crate::data_file::measurement_parser::parse_measurement_table(
        path,
        &table.headers,
        &table.rows,
        &crate::data_file::measurement_parser::TableParseMetadata {
            source_sheet: Some(table.sheet_name.clone()),
            header_row_index: Some(table.header_row_index + 1),
            skipped_leading_rows: table.rows_skipped_before_header,
            unit_row_index: table.unit_row_index.map(|index| index + 1),
            parser_kind: Some("excel_time_series".to_string()),
        },
    )?;
    Ok(ExcelMeasurementParseResult {
        parsed,
        sheet_name: table.sheet_name,
        header_row_index: table.header_row_index,
        rows_skipped_before_header: table.rows_skipped_before_header,
        unit_row_index: table.unit_row_index,
    })
}

pub fn read_worksheet(
    path: impl AsRef<Path>,
    sheet_name: Option<&str>,
) -> Result<ExcelTable, DataParsingError> {
    let path = path.as_ref();
    let mut workbook = open_workbook_auto(path).map_err(|error| {
        DataParsingError::invalid_at(path, format!("failed to open Excel workbook: {error}"))
    })?;
    let sheet_names = workbook.sheet_names();
    if sheet_names.is_empty() {
        return Err(DataParsingError::invalid_at(
            path,
            "workbook contains no worksheets; expected at least one worksheet with a time/timestamp column and one measurement channel",
        ));
    }

    if let Some(requested) = sheet_name {
        let chosen = resolve_requested_sheet(&sheet_names, requested, path)?;
        let range = workbook.worksheet_range(&chosen).map_err(|error| {
            DataParsingError::invalid_at(
                path,
                format!("failed to read worksheet '{chosen}': {error}"),
            )
        })?;
        return convert_range_to_table(path, &chosen, range);
    }

    let mut candidates = Vec::new();
    for name in &sheet_names {
        let range = workbook.worksheet_range(name).map_err(|error| {
            DataParsingError::invalid_at(
                path,
                format!("failed to read worksheet '{name}': {error}"),
            )
        })?;
        candidates.push(classify_sheet(path, name, range)?);
    }

    let non_empty_count = candidates
        .iter()
        .filter(|candidate| candidate.non_empty)
        .count();
    if non_empty_count == 0 {
        return Err(DataParsingError::invalid_at(
            path,
            "all worksheets are empty; expected one worksheet containing a time-series table",
        ));
    }

    let compatible = candidates
        .iter()
        .filter_map(|candidate| candidate.table.as_ref())
        .collect::<Vec<_>>();

    if compatible.len() == 1 {
        return Ok(compatible[0].clone());
    }
    if compatible.len() > 1 {
        let names = compatible
            .iter()
            .map(|candidate| candidate.sheet_name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(DataParsingError::invalid_at(
            path,
            format!(
                "multiple compatible time-series worksheets were found ({names}); specify --sheet <NAME> to select one",
            ),
        ));
    }

    if candidates
        .iter()
        .any(|candidate| candidate.non_empty && candidate.eis_like_only)
    {
        return Err(DataParsingError::invalid_at(
            path,
            "no compatible time-series worksheet was found; workbook appears to contain only EIS-style sheets. XLSX EIS ingestion is not supported in this workflow. Export EIS data as CHI/text format.",
        ));
    }

    let non_empty_names = candidates
        .iter()
        .filter(|candidate| candidate.non_empty)
        .map(|candidate| candidate.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(DataParsingError::invalid_at(
        path,
        format!(
            "no compatible time-series worksheet was found in non-empty sheets ({non_empty_names}); expected one time/timestamp column and at least one measurement channel",
        ),
    ))
}

fn resolve_requested_sheet(
    sheet_names: &[String],
    requested: &str,
    path: &Path,
) -> Result<String, DataParsingError> {
    if let Some(exact) = sheet_names.iter().find(|name| name.as_str() == requested) {
        return Ok(exact.clone());
    }

    let case_insensitive = sheet_names
        .iter()
        .filter(|name| name.eq_ignore_ascii_case(requested))
        .collect::<Vec<_>>();
    if case_insensitive.len() == 1 {
        return Ok(case_insensitive[0].clone());
    }

    Err(DataParsingError::invalid_at(
        path,
        format!(
            "worksheet '{requested}' was not found; available worksheets: {}",
            sheet_names.join(", ")
        ),
    ))
}

fn classify_sheet(
    path: &Path,
    sheet_name: &str,
    range: calamine::Range<Data>,
) -> Result<SheetCandidate, DataParsingError> {
    let rows = range.rows().map(|row| row.to_vec()).collect::<Vec<_>>();
    if rows.is_empty() {
        return Ok(SheetCandidate {
            name: sheet_name.to_string(),
            table: None,
            non_empty: false,
            eis_like_only: false,
        });
    }
    let non_empty = rows.iter().any(|row| {
        row.iter()
            .any(|cell| !cell_to_string(cell).trim().is_empty())
    });
    if !non_empty {
        return Ok(SheetCandidate {
            name: sheet_name.to_string(),
            table: None,
            non_empty: false,
            eis_like_only: false,
        });
    }

    let table = convert_rows_to_table(path, sheet_name, &rows)?;
    if let Some(table) = table {
        return Ok(SheetCandidate {
            name: sheet_name.to_string(),
            table: Some(table),
            non_empty: true,
            eis_like_only: false,
        });
    }

    let eis_like_only = rows.iter().any(|row| {
        let headers = row.iter().map(cell_to_string).collect::<Vec<_>>();
        looks_like_eis_header(&headers)
    });
    Ok(SheetCandidate {
        name: sheet_name.to_string(),
        table: None,
        non_empty: true,
        eis_like_only,
    })
}

fn convert_range_to_table(
    path: &Path,
    sheet_name: &str,
    range: calamine::Range<Data>,
) -> Result<ExcelTable, DataParsingError> {
    let rows = range.rows().map(|row| row.to_vec()).collect::<Vec<_>>();
    let Some(table) = convert_rows_to_table(path, sheet_name, &rows)? else {
        if rows
            .iter()
            .any(|row| looks_like_eis_header(&row.iter().map(cell_to_string).collect::<Vec<_>>()))
        {
            return Err(DataParsingError::invalid_at(
                path,
                format!(
                    "worksheet '{sheet_name}' is EIS-like but XLSX EIS ingestion is not supported here; provide CHI/text EIS input instead"
                ),
            ));
        }
        return Err(DataParsingError::invalid_at(
            path,
            format!(
                "worksheet '{sheet_name}' does not contain a compatible time-series header; expected one time/timestamp column and at least one measurement channel",
            ),
        ));
    };
    Ok(table)
}

fn convert_rows_to_table(
    path: &Path,
    sheet_name: &str,
    rows: &[Vec<Data>],
) -> Result<Option<ExcelTable>, DataParsingError> {
    let mut best_header_index = None;
    let mut best_headers = Vec::new();
    let mut best_time_index = None;

    for (row_index, row) in rows.iter().enumerate() {
        let headers = row.iter().map(cell_to_string).collect::<Vec<_>>();
        let trimmed = headers
            .iter()
            .map(|header| header.trim().to_string())
            .collect::<Vec<_>>();
        if trimmed.iter().filter(|header| !header.is_empty()).count() < 2 {
            continue;
        }
        let Some(time_index) = trimmed
            .iter()
            .position(|header| is_time_header_label(header))
        else {
            continue;
        };
        let channel_count = trimmed
            .iter()
            .enumerate()
            .filter(|(index, header)| *index != time_index && !header.is_empty())
            .count();
        if channel_count == 0 {
            continue;
        }
        let mut normalized = HashSet::new();
        let mut has_duplicate = false;
        for header in &trimmed {
            if header.is_empty() {
                continue;
            }
            let key = normalize_header(header);
            if !normalized.insert(key) {
                has_duplicate = true;
                break;
            }
        }
        if has_duplicate {
            return Err(DataParsingError::invalid_at(
                path,
                format!(
                    "worksheet '{sheet_name}' has duplicate header names that cannot be disambiguated safely; rename duplicate columns",
                ),
            ));
        }
        best_header_index = Some(row_index);
        best_headers = trimmed;
        best_time_index = Some(time_index);
        break;
    }

    let Some(header_row_index) = best_header_index else {
        return Ok(None);
    };
    let time_index = best_time_index.expect("time index exists with header");

    let mut unit_row_index = None;
    let mut data_start_index = header_row_index + 1;
    if let Some(row) = rows.get(data_start_index) {
        let values = row.iter().map(cell_to_string).collect::<Vec<_>>();
        if looks_like_unit_row(&values, time_index) {
            unit_row_index = Some(data_start_index);
            data_start_index += 1;
        }
    }

    let mut output_rows = Vec::new();
    for (row_index, row) in rows.iter().enumerate().skip(data_start_index) {
        let mut values = row.iter().map(cell_to_string).collect::<Vec<_>>();
        if values.len() < best_headers.len() {
            values.resize(best_headers.len(), String::new());
        } else {
            values.truncate(best_headers.len());
        }
        if values.iter().all(|value| value.trim().is_empty()) {
            continue;
        }

        if values
            .get(time_index)
            .is_some_and(|value| value.starts_with("#CALC!("))
        {
            return Err(DataParsingError::invalid_at(
                path,
                format!(
                    "worksheet '{sheet_name}' row {} has a formula error in the time column; store a cached numeric timestamp value before export",
                    row_index + 1
                ),
            ));
        }
        output_rows.push(values);
    }

    if output_rows.is_empty() {
        return Err(DataParsingError::invalid_at(
            path,
            format!("worksheet '{sheet_name}' has a compatible header but no valid data rows",),
        ));
    }

    Ok(Some(ExcelTable {
        source_path: path.display().to_string(),
        sheet_name: sheet_name.to_string(),
        header_row_index,
        headers: best_headers,
        rows: output_rows,
        rows_skipped_before_header: header_row_index,
        unit_row_index,
    }))
}

fn is_time_header_label(value: &str) -> bool {
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

fn looks_like_eis_header(headers: &[String]) -> bool {
    let normalized = headers
        .iter()
        .map(|header| normalize_header(header))
        .collect::<Vec<_>>();
    let has_freq = normalized.iter().any(|header| header == "freq/hz");
    let has_impedance = normalized
        .iter()
        .any(|header| header == "z'/ohm" || header == "z\"/ohm");
    has_freq && has_impedance
}

fn looks_like_unit_row(values: &[String], time_index: usize) -> bool {
    if values.is_empty() {
        return false;
    }
    let mut non_time_non_empty = 0usize;
    let mut likely_units = 0usize;
    for (index, value) in values.iter().enumerate() {
        if index == time_index {
            continue;
        }
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        non_time_non_empty += 1;
        if trimmed.len() <= 12
            && trimmed
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '%' | 'Ω' | '^'))
        {
            likely_units += 1;
        }
    }
    non_time_non_empty > 0 && likely_units == non_time_non_empty
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Float(value) => {
            if value.is_nan() {
                String::new()
            } else {
                value.to_string()
            }
        }
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) => value.clone(),
        Data::DurationIso(value) => value.clone(),
        Data::Error(error) => format!("#CALC!({error})"),
    }
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
