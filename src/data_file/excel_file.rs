//! Excel compatibility wrappers backed entirely by `electrodata-io`.

use crate::domain::{DataParsingError, MeasurementParseResult};
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

pub fn parse_excel_measurement(
    path: impl AsRef<Path>,
    sheet_name: Option<&str>,
) -> Result<ExcelMeasurementParseResult, DataParsingError> {
    let path = path.as_ref();
    let dataset =
        crate::data_file::electrodata_domain_adapter::read_dataset_with_sheet(path, sheet_name)?;
    let parsed = crate::data_file::electrodata_domain_adapter::measurement_parse_result(&dataset)?;
    let selected_sheet = dataset
        .metadata
        .provenance
        .worksheet
        .clone()
        .unwrap_or_default();
    let header_row_index = dataset.metadata.raw_rows.len();

    Ok(ExcelMeasurementParseResult {
        parsed,
        sheet_name: selected_sheet,
        header_row_index,
        rows_skipped_before_header: header_row_index,
        unit_row_index: None,
    })
}

/// Retained for callers that need the former table-shaped compatibility API.
/// The workbook and worksheet selection are performed by `electrodata-io`;
/// this function only adapts canonical typed values back into strings.
pub fn read_worksheet(
    path: impl AsRef<Path>,
    sheet_name: Option<&str>,
) -> Result<ExcelTable, DataParsingError> {
    let path = path.as_ref();
    let dataset =
        crate::data_file::electrodata_domain_adapter::read_dataset_with_sheet(path, sheet_name)?;
    let headers = dataset
        .canonical_columns()
        .iter()
        .map(|column| {
            column
                .original_name
                .clone()
                .unwrap_or_else(|| column.name.clone())
        })
        .collect::<Vec<_>>();
    let columns = dataset
        .canonical_columns()
        .iter()
        .map(|column| column.values.clone())
        .collect::<Vec<_>>();
    let rows = (0..columns.first().map_or(0, Vec::len))
        .map(|row_index| {
            columns
                .iter()
                .map(|column| {
                    column
                        .get(row_index)
                        .copied()
                        .flatten()
                        .map(|value| value.to_string())
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();
    let header_row_index = dataset.metadata.raw_rows.len();

    Ok(ExcelTable {
        source_path: path.to_string_lossy().to_string(),
        sheet_name: dataset
            .metadata
            .provenance
            .worksheet
            .clone()
            .unwrap_or_default(),
        header_row_index,
        headers,
        rows,
        rows_skipped_before_header: header_row_index,
        unit_row_index: None,
    })
}
