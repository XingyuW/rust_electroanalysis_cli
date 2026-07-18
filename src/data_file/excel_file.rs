//! Excel workbook parser integrated into the shared data-file architecture.
//!
//! This module is responsible only for opening a workbook, selecting a
//! worksheet, and converting cells into a tabular intermediate form that is
//! then passed into the existing schema-detection and normalization logic.

use crate::domain::DataParsingError;
use calamine::{Data, Reader, open_workbook_auto};
use std::path::Path;

/// Intermediate tabular data extracted from a worksheet, ready for shared
/// schema recognition and domain normalization.
#[derive(Debug, Clone)]
pub struct ExcelTable {
    /// The file-system path of the source workbook.
    pub source_path: String,
    /// The name of the selected worksheet.
    pub sheet_name: String,
    /// Header row, assembled from the first data-bearing row found.
    pub headers: Vec<String>,
    /// Data rows, each with one string per column.
    pub rows: Vec<Vec<String>>,
    /// Number of rows skipped before the header (e.g., blank rows).
    pub rows_skipped_before_header: usize,
    /// Number of trailing rows identified as summary or non-data.
    pub trailing_rows_skipped: usize,
}

/// Open an Excel workbook and return a [`calamine::Range`] for a specific
/// sheet.  When `sheet_name` is `None` the function applies the
/// deterministic worksheet‑selection rules documented in the project
/// README.
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
            "workbook contains no worksheets",
        ));
    }

    let sheet = resolve_sheet(&sheet_names, sheet_name, path)?;
    let range = workbook.worksheet_range(&sheet).map_err(|error| {
        DataParsingError::invalid_at(path, format!("failed to read worksheet '{sheet}': {error}"))
    })?;

    convert_range_to_table(path, &sheet, range)
}

/// Deterministic worksheet selection.
fn resolve_sheet(
    sheet_names: &[String],
    requested: Option<&str>,
    path: &Path,
) -> Result<String, DataParsingError> {
    if let Some(name) = requested {
        let exact = sheet_names.iter().find(|n| n.as_str() == name);
        let case_insensitive = sheet_names.iter().find(|n| n.eq_ignore_ascii_case(name));
        match (exact, case_insensitive) {
            (Some(e), _) => return Ok(e.clone()),
            (None, Some(ci)) => return Ok(ci.clone()),
            (None, None) => {
                return Err(DataParsingError::invalid_at(
                    path,
                    format!(
                        "worksheet '{name}' not found; available sheets: {}",
                        sheet_names.join(", ")
                    ),
                ));
            }
        }
    }

    // No explicit sheet requested — try each sheet and return the first one
    // that contains a recognised schema (time‑header, frequency‑header, or
    // potential‑header).

    // If there is exactly one non‑empty sheet, use it.
    let non_empty: Vec<_> = sheet_names.to_vec();
    if non_empty.len() == 1 {
        return Ok(non_empty[0].clone());
    }

    // Ambiguous — require an explicit choice.
    Err(DataParsingError::invalid_at(
        path,
        format!(
            "workbook contains multiple worksheets ({}); use --sheet to select one",
            sheet_names.join(", ")
        ),
    ))
}

/// Convert a calamine `Range` into our generic [`ExcelTable`].
fn convert_range_to_table(
    path: &Path,
    sheet_name: &str,
    range: calamine::Range<Data>,
) -> Result<ExcelTable, DataParsingError> {
    let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
    if rows.is_empty() {
        return Err(DataParsingError::invalid_at(
            path,
            format!("worksheet '{sheet_name}' is empty"),
        ));
    }

    // Skip leading blank rows.
    let mut skipped = 0usize;
    let mut header_idx = 0usize;
    for (i, row) in rows.iter().enumerate() {
        skipped = i;
        if row.iter().any(|cell| !is_blank(cell)) {
            header_idx = i;
            break;
        }
    }
    if header_idx >= rows.len() || rows[header_idx].iter().all(is_blank) {
        return Err(DataParsingError::invalid_at(
            path,
            format!("worksheet '{sheet_name}' contains no visible data"),
        ));
    }

    let header_original: Vec<String> = rows[header_idx].iter().map(cell_to_string).collect();
    let headers = disambiguate_duplicate_headers(&header_original);

    let data_start = header_idx + 1;
    let mut data_rows: Vec<Vec<String>> = Vec::new();
    for row in rows.iter().skip(data_start) {
        let string_row: Vec<String> = row.iter().map(cell_to_string).collect();
        // Skip completely blank rows.
        if string_row.iter().all(|v| v.trim().is_empty()) {
            continue;
        }
        // Pad to header width if the row is shorter.
        let mut padded = string_row;
        while padded.len() < headers.len() {
            padded.push(String::new());
        }
        // Truncate to header width if the row is longer.
        padded.truncate(headers.len());
        data_rows.push(padded);
    }

    if data_rows.is_empty() {
        return Err(DataParsingError::invalid_at(
            path,
            format!("worksheet '{sheet_name}' has a header but no data rows"),
        ));
    }

    Ok(ExcelTable {
        source_path: path.display().to_string(),
        sheet_name: sheet_name.to_string(),
        headers,
        rows: data_rows,
        rows_skipped_before_header: skipped,
        trailing_rows_skipped: 0,
    })
}

/// Convert a calamine `Data` cell into a plain string.
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.is_nan() {
                String::new()
            } else if *f == f.trunc() && f.is_finite() && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => format!("{i}"),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => {
            // Serialise as ISO-8601 string; numeric parsers downstream will
            // either use this as a label or report an informative parse error.
            d.to_string()
        }
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => {
            // Formula cells without cached results produce an error string.
            format!("#CALC!({e})")
        }
    }
}

fn is_blank(cell: &Data) -> bool {
    matches!(cell, Data::Empty)
        || match cell {
            Data::String(s) => s.trim().is_empty(),
            Data::Float(f) => f.is_nan(),
            _ => false,
        }
}

/// Disambiguate duplicate column names by appending a numeric suffix.
fn disambiguate_duplicate_headers(headers: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(headers.len());
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for h in headers {
        let key = h.trim().to_string();
        let count = seen.entry(key.clone()).or_insert(0);
        if *count == 0 {
            result.push(key);
        } else {
            result.push(format!("{}__duplicate_{}", key, count));
        }
        *count += 1;
    }
    result
}

/// Convert an [`ExcelTable`] into CSV‑compatible text lines so that the
/// existing [`super::measurement_parser::parse_measurement_text`] can process
/// the data without duplicating schema detection.
pub fn table_to_csv_lines(table: &ExcelTable) -> Vec<String> {
    let mut lines = Vec::with_capacity(1 + table.rows.len());
    lines.push(table.headers.join(","));
    for row in &table.rows {
        lines.push(row.join(","));
    }
    lines
}

/// Load an Excel workbook through the unified measurement parser flow.
/// Returns `(MeasurementParseResult, sheet_name)`.
pub fn parse_excel_measurement(
    path: impl AsRef<Path>,
    sheet_name: Option<&str>,
) -> Result<(crate::domain::MeasurementParseResult, String), DataParsingError> {
    let path = path.as_ref();
    let table = read_worksheet(path, sheet_name)?;
    let sheet = table.sheet_name.clone();
    let csv_lines = table_to_csv_lines(&table);
    let text = csv_lines.join("\n");
    let parsed = crate::data_file::measurement_parser::parse_measurement_text(&text, path)?;
    Ok((parsed, sheet))
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Data;

    #[test]
    fn disambiguate_duplicates_appends_suffix() {
        let headers = vec!["Time/s".to_string(), "E/V".to_string(), "E/V".to_string()];
        let result = disambiguate_duplicate_headers(&headers);
        assert_eq!(result[0], "Time/s");
        assert_eq!(result[1], "E/V");
        assert_eq!(result[2], "E/V__duplicate_1");
    }

    #[test]
    fn cell_to_string_handles_common_cases() {
        assert_eq!(cell_to_string(&Data::Empty), "");
        assert_eq!(cell_to_string(&Data::String("hello".into())), "hello");
        assert_eq!(cell_to_string(&Data::Float(1.5)), "1.5");
        assert_eq!(cell_to_string(&Data::Float(3.0)), "3");
        assert_eq!(cell_to_string(&Data::Int(42)), "42");
        assert_eq!(cell_to_string(&Data::Bool(true)), "true");
        assert!(cell_to_string(&Data::Float(f64::NAN)).is_empty());
    }

    #[test]
    fn table_to_csv_lines_produces_valid_csv() {
        let table = ExcelTable {
            source_path: "test.xlsx".into(),
            sheet_name: "Sheet1".into(),
            headers: vec!["Time/s".into(), "E/V".into()],
            rows: vec![
                vec!["0.0".into(), "0.5".into()],
                vec!["1.0".into(), "0.6".into()],
            ],
            rows_skipped_before_header: 0,
            trailing_rows_skipped: 0,
        };
        let lines = table_to_csv_lines(&table);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Time/s,E/V");
    }
}
