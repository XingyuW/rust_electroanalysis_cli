//! Compile-time/public-boundary coverage for the canonical physical-input path.

use rust_electroanalysis_cli::data_file::{
    project_compatibility_read_options, read_dataset, read_dataset_with_sheet,
};
use rust_electroanalysis_cli::domain::DataParsingError;
use std::path::Path;

#[test]
fn public_raw_input_boundary_is_the_canonical_reader() {
    let _: fn(&Path) -> Result<electrodata_io::Dataset, DataParsingError> =
        |path| read_dataset(path);
    let _: fn(&Path, Option<&str>) -> Result<electrodata_io::Dataset, DataParsingError> =
        |path, sheet| read_dataset_with_sheet(path, sheet);
    let options = project_compatibility_read_options();
    assert_eq!(options.profile, electrodata_io::ReadProfile::Compatibility);
    assert_eq!(
        options.invalid_time_policy,
        electrodata_io::InvalidTimePolicy::SkipRow
    );
    assert_eq!(
        options.invalid_cell_policy,
        electrodata_io::InvalidCellPolicy::Null
    );
    assert_eq!(
        options.ragged_row_policy,
        electrodata_io::RaggedRowPolicy::PadNulls
    );
}

#[test]
fn canonical_time_series_fixture_uses_typed_domain_access() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase2/sensor.csv");
    let dataset = read_dataset(&input).expect("canonical fixture read");
    let view = dataset.time_series_view().expect("typed time-series view");
    assert_eq!(view.time_seconds.len(), 121);
    assert_eq!(view.measurements.len(), 1);
    assert_eq!(view.measurements[0].original_name.as_deref(), Some("E1/V"));
}
