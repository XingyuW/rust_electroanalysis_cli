//! Compile-time/public-boundary coverage for the canonical physical-input path.

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
    let five_path = fixture_dir.join("chi_eis_five_column.csv");
    let three = EISData::parse_file(&three_path).expect("three-column EIS");
    let five = EISData::parse_file(&five_path).expect("five-column EIS");
    assert!(three.measured_magnitude.is_none());
    assert!(three.measured_phase.is_none());
    assert!(five.measured_magnitude.is_some());
    assert!(five.measured_phase.is_some());

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
