//! Typed regression coverage for the canonical provider error boundary.

use rust_electroanalysis_cli::{
    data_file::{read_dataset, read_dataset_with_sheet},
    domain::DataParsingError,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn temporary_csv(label: &str, contents: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("typed-canonical-{label}-{nonce}.csv"));
    fs::write(&path, contents).expect("write fixture");
    path
}

fn provider_error(error: DataParsingError) -> Box<electrodata_io::Error> {
    match error {
        DataParsingError::ElectrodataIo(error) => error,
        other => panic!("expected DataParsingError::ElectrodataIo, got {other:?}"),
    }
}

fn contains_variant(
    error: &electrodata_io::Error,
    predicate: &dyn Fn(&electrodata_io::Error) -> bool,
) -> bool {
    predicate(error)
        || matches!(error, electrodata_io::Error::ReadContext { source, .. } if contains_variant(source, predicate))
}

#[test]
fn canonical_errors_preserve_detection_schema_and_accessor_variants() {
    let unknown = temporary_csv("unknown", "not an electrochemical dataset\ntext only\n");
    let error = provider_error(read_dataset(&unknown).expect_err("unknown format"));
    assert!(contains_variant(error.as_ref(), &|error| matches!(
        error,
        electrodata_io::Error::UnknownFormat { path, best_confidence, threshold }
        if path == &unknown && best_confidence < threshold
    )));
    fs::remove_file(&unknown).ok();

    let missing_role = temporary_csv("missing-role", "A.C. Impedance\nFreq/Hz,Z'/ohm\n1000,10\n");
    let error = provider_error(read_dataset(&missing_role).expect_err("missing EIS role"));
    assert!(
        contains_variant(error.as_ref(), &|error| matches!(
            error,
            electrodata_io::Error::MissingRequiredRole { path, role, detected_roles, .. }
            if path == &missing_role && *role == electrodata_io::ColumnRole::ImpedanceImaginary
                && detected_roles.contains(&electrodata_io::ColumnRole::Frequency)
        )),
        "unexpected canonical error: {error:?}"
    );
    fs::remove_file(&missing_role).ok();

    let conflict = temporary_csv(
        "eis-conflict",
        "Freq/Hz,Z'/ohm,Z'/ohm,Z\"/ohm\n1000,10,11,-1\n",
    );
    let error = provider_error(read_dataset(&conflict).expect_err("EIS schema conflict"));
    assert!(contains_variant(error.as_ref(), &|error| matches!(
        error,
        electrodata_io::Error::EisSchemaConflict { details }
        if details.path == conflict && details.conflicting_role.is_some()
    )));
    fs::remove_file(&conflict).ok();

    let time_series = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/io_migration/regular_two_column.csv");
    let dataset = read_dataset(&time_series).expect("time series dataset");
    let accessor = dataset.eis_view().expect_err("wrong dataset view");
    assert!(matches!(
        accessor,
        electrodata_io::Error::WrongDatasetKind { .. }
            | electrodata_io::Error::InvalidDatasetView { .. }
    ));
}

#[test]
fn canonical_worksheet_error_keeps_read_context_and_requested_sheet() {
    let workbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xlsx/eis_only.xlsx");
    let error = provider_error(
        read_dataset_with_sheet(&workbook, Some("missing")).expect_err("missing worksheet"),
    );
    assert!(contains_variant(error.as_ref(), &|error| matches!(
        error,
        electrodata_io::Error::MissingWorksheet { path, worksheet }
        if path == &workbook && worksheet == "missing"
    )));
    if let electrodata_io::Error::ReadContext {
        worksheet, source, ..
    } = error.as_ref()
    {
        assert_eq!(worksheet.as_deref(), Some("missing"));
        assert!(matches!(
            source.as_ref(),
            electrodata_io::Error::MissingWorksheet { .. }
        ));
    }
}
