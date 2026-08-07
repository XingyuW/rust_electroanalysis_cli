//! Compile-time/public-boundary coverage for the canonical physical-input path.

use electrodata_io::{
    ColumnNamePolicy, CoordinateOrderPolicy, HeaderPolicy, InvalidCellPolicy, InvalidTimePolicy,
    RaggedRowPolicy, ReadProfile, SheetSelector, ValidationLevel,
};
use rust_electroanalysis_cli::data_file::{
    EISData, parse_measurement_file, project_compatibility_read_options, read_dataset,
    read_dataset_with_sheet,
};
use rust_electroanalysis_cli::{
    CircuitFitResult,
    domain::{AnalysisProvenance, DataParsingError},
    results::EisFitArtifact,
};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

fn temporary_csv(name: &str, content: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("canonical-coordinate-{name}-{nonce}.csv"));
    fs::write(&path, content).expect("write fixture");
    path
}

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
fn compatibility_profile_is_fully_locked() {
    let options = project_compatibility_read_options();
    assert_eq!(options.profile, ReadProfile::Compatibility);
    assert_eq!(options.invalid_time_policy, InvalidTimePolicy::SkipRow);
    assert_eq!(options.invalid_cell_policy, InvalidCellPolicy::Null);
    assert_eq!(options.ragged_row_policy, RaggedRowPolicy::PadNulls);
    assert_eq!(options.header_policy, HeaderPolicy::Auto);
    assert_eq!(
        options.coordinate_order_policy,
        CoordinateOrderPolicy::Preserve
    );
    assert_eq!(options.column_name_policy, ColumnNamePolicy::Canonical);
    assert_eq!(options.sheet, SheetSelector::Auto);
    assert_eq!(options.validation, ValidationLevel::Structural);
    assert!(!options.allow_lossy_utf8);
}

#[test]
fn compatibility_recoveries_keep_structured_provider_evidence() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/io_migration");
    for (fixture, expected_code, expected_recovery) in [
        (
            "regular_invalid_numeric.csv",
            "ValueReplacedWithMissing",
            "ValueReplacedWithMissing",
        ),
        (
            "regular_malformed_timestamp.csv",
            "MalformedTimestamp",
            "RowSkipped",
        ),
        ("regular_ragged_rows.csv", "RaggedRow", "None"),
    ] {
        let parsed = parse_measurement_file(root.join(fixture)).expect("compatibility parse");
        let diagnostic = parsed
            .diagnostics
            .ingestion_diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == expected_code)
            .expect("diagnostic must be retained");
        assert_eq!(diagnostic.recovery, expected_recovery, "{fixture}");
        assert!(diagnostic.row.is_some(), "{fixture}");
        if fixture != "regular_ragged_rows.csv" {
            assert!(diagnostic.column_index.is_some(), "{fixture}");
        }
    }
    let ragged = parse_measurement_file(root.join("regular_ragged_rows.csv"))
        .expect("ragged compatibility parse");
    assert!(
        ragged
            .diagnostics
            .ingestion_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "RowPadded" && diagnostic.recovery == "RowPadded")
    );
    let clean = parse_measurement_file(root.join("regular_two_column.csv")).expect("clean parse");
    assert!(
        clean
            .diagnostics
            .ingestion_diagnostics
            .iter()
            .all(|diagnostic| {
                !matches!(
                    diagnostic.code.as_str(),
                    "MalformedTimestamp" | "ValueReplacedWithMissing" | "RaggedRow" | "RowPadded"
                )
            })
    );
}

#[test]
fn public_eis_construction_preserves_source_and_derived_semantics() {
    let data = EISData::from_impedance(vec![1.0], vec![3.0], vec![4.0])
        .with_source_bode(Some(vec![Some(99.0)]), Some(vec![Some(-12.0)]));
    assert_eq!(data.source_measured_magnitude(), Some(&[Some(99.0)][..]));
    assert_eq!(data.source_measured_phase(), Some(&[Some(-12.0)][..]));
    assert_eq!(data.derived_magnitude(), &[5.0]);
    assert_ne!(
        data.source_measured_phase().unwrap()[0],
        Some(data.derived_phase()[0])
    );
}

#[test]
fn canonical_time_series_fixture_uses_typed_domain_access() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase2/sensor.csv");
    let dataset = read_dataset(&input).expect("canonical fixture read");
    let view = dataset.time_series_view().expect("typed time-series view");
    assert_eq!(view.time_seconds.as_ref().map(Vec::len), Some(121));
    assert_eq!(view.measurements.len(), 1);
    assert_eq!(view.measurements[0].original_name.as_deref(), Some("E1/V"));
}

#[test]
fn domain_measurement_preserves_raw_coordinate_values_and_unit() {
    for (name, header, expected_unit, expected_time) in [
        ("seconds", "Time/sec", "s", vec![0.0, 1.5, 3.0]),
        ("hours", "Time/h", "h", vec![0.0, 0.5, 1.0]),
        ("days", "Time/day", "day", vec![0.0, 0.25, 1.0]),
        ("headerless", "", "", vec![0.0, 2.0, 4.0]),
    ] {
        let content = if header.is_empty() {
            "0,0.1\n2,0.2\n4,0.3\n".to_string()
        } else {
            format!(
                "{header},Potential/V\n0,0.1\n{},0.2\n{},0.3\n",
                expected_time[1], expected_time[2]
            )
        };
        let path = temporary_csv(name, &content);
        let parsed = parse_measurement_file(&path).expect("canonical parse");
        assert_eq!(parsed.measurement.time, expected_time, "{name}");
        assert_eq!(parsed.measurement.time_unit, expected_unit, "{name}");
        fs::remove_file(path).ok();
    }
}

#[test]
fn scientific_seconds_conversion_is_explicit_and_recorded() {
    let path = temporary_csv(
        "hours-conversion",
        "Time/h,Potential/V\n0,0.1\n0.5,0.2\n1,0.3\n",
    );
    let raw = parse_measurement_file(&path)
        .expect("canonical parse")
        .measurement;
    let normalized = raw.normalized_to_seconds().expect("explicit conversion");
    assert_eq!(raw.time, vec![0.0, 0.5, 1.0]);
    assert_eq!(normalized.time, vec![0.0, 1_800.0, 3_600.0]);
    assert_eq!(
        normalized.time_conversion.expect("conversion").source_unit,
        "h"
    );
    fs::remove_file(path).ok();
}

#[test]
fn eis_artifact_keeps_source_bode_values_distinct_from_derived_values() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/io_migration");
    let three_path = fixture_dir.join("chi_eis_three_column.csv");
    let four_path = fixture_dir.join("chi_eis_four_column.csv");
    let five_path = fixture_dir.join("chi_eis_five_column.csv");
    let three = EISData::parse_file(&three_path).expect("three-column EIS");
    let four = EISData::parse_file(&four_path).expect("four-column EIS");
    let five = EISData::parse_file(&five_path).expect("five-column EIS");
    assert!(three.measured_magnitude.is_none());
    assert!(three.measured_phase.is_none());
    assert!(five.measured_magnitude.is_some());
    assert!(five.measured_phase.is_some());
    assert!(four.measured_magnitude.is_none());
    assert_eq!(four.source_measured_phase().unwrap()[0], Some(-5.710_593));
    assert_ne!(
        four.source_measured_phase().unwrap()[0],
        Some(four.derived_phase()[0])
    );

    let fit = CircuitFitResult {
        fitted_parameters: vec![1.0],
        parameter_names: vec!["R_0".into()],
        parameter_units: vec!["Ohm".into()],
        fitted_z_re: three.z_re.clone(),
        fitted_z_im: three.z_im.clone(),
        fitted_magnitude: three.derived_magnitude.clone(),
        fitted_phase: three.derived_phase.clone(),
    };
    let artifact = EisFitArtifact::from_fit(
        &three,
        "R0",
        &fit,
        AnalysisProvenance::from_paths(&three_path, None).expect("provenance"),
    );
    assert!(artifact.source.source_measured_magnitude_ohm.is_none());
    assert!(artifact.source.source_measured_phase_deg.is_none());
    assert_eq!(
        artifact.source.derived_magnitude_ohm,
        three.derived_magnitude
    );
    assert_eq!(artifact.source.derived_phase_deg, three.derived_phase);

    let supplied = five.measured_magnitude.as_ref().expect("source magnitude");
    assert_eq!(supplied[0], Some(10.049_876));
    assert_ne!(supplied[0], Some(five.derived_magnitude[0]));
    assert_eq!(five.source_measured_phase().unwrap()[0], Some(-5.710_593));
    assert_ne!(
        five.source_measured_phase().unwrap()[0],
        Some(five.derived_phase()[0])
    );

    let mut legacy = serde_json::to_value(&artifact).expect("serialize v2 artifact");
    let root = legacy.as_object_mut().expect("artifact object");
    root.insert("schema_version".into(), serde_json::json!(1));
    let mut legacy_source = root
        .remove("source")
        .expect("v2 source field")
        .as_object()
        .expect("source object")
        .clone();
    legacy_source.remove("source_measured_magnitude_ohm");
    legacy_source.remove("source_measured_phase_deg");
    let magnitude = legacy_source
        .remove("derived_magnitude_ohm")
        .expect("derived magnitude");
    let phase = legacy_source
        .remove("derived_phase_deg")
        .expect("derived phase");
    legacy_source.insert("magnitude_ohm".into(), magnitude);
    legacy_source.insert("phase_deg".into(), phase);
    root.insert("measured".into(), serde_json::Value::Object(legacy_source));
    let decoded: EisFitArtifact = serde_json::from_value(legacy).expect("schema-v1 compatibility");
    assert!(decoded.source.source_measured_magnitude_ohm.is_none());
    assert_eq!(
        decoded.source.derived_magnitude_ohm,
        three.derived_magnitude
    );
}
