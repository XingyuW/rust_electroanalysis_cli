use rust_electroanalysis_cli::data_file::{
    ElectrochemData, measurement_to_plot_data, parse_measurement_text,
};
use rust_electroanalysis_cli::domain::{
    ExperimentEventKind, MultiChannelMeasurement, load_experiment_metadata,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_fixture(extension: &str, contents: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "rust_electroanalysis_phase1_{}_{}.{}",
        std::process::id(),
        suffix,
        extension
    ));
    fs::write(&path, contents).expect("write fixture");
    path
}

fn remove_fixture(path: PathBuf) {
    fs::remove_file(path).expect("remove fixture");
}

#[test]
fn parses_single_channel_sensor_data() {
    let parsed = parse_measurement_text(
        "time/sec,potential/V\n0,0.101\n1,0.102\n2,0.103\n",
        "single.csv",
    )
    .expect("single-channel data should parse");

    assert_eq!(parsed.measurement.time, vec![0.0, 1.0, 2.0]);
    assert_eq!(parsed.measurement.channels.len(), 1);
    assert_eq!(parsed.measurement.channels[0].name, "potential");
    assert_eq!(parsed.measurement.channels[0].unit, "V");
    assert_eq!(parsed.diagnostics.total_rows, 3);
    assert_eq!(parsed.diagnostics.successfully_parsed_rows, 3);
    assert!(!parsed.diagnostics.has_issues());
}

#[test]
fn parses_multiple_named_channels_on_one_time_axis() {
    let parsed = parse_measurement_text(
        "time/sec,potential/V,temperature/C\n0,0.101,25.0\n1,0.102,25.1\n",
        "multi.csv",
    )
    .expect("multi-channel data should parse");

    assert_eq!(parsed.measurement.channels.len(), 2);
    assert_eq!(parsed.measurement.channels[0].name, "potential");
    assert_eq!(parsed.measurement.channels[0].unit, "V");
    assert_eq!(parsed.measurement.channels[1].name, "temperature");
    assert_eq!(parsed.measurement.channels[1].unit, "C");
    assert_eq!(parsed.measurement.channels[1].values[1], Some(25.1));
}

#[test]
fn reports_malformed_rows_and_missing_values() {
    let parsed = parse_measurement_text(
        "time/sec,potential/V,temperature/C\n0,0.1,25\nbad,0.2,25\n1,not-a-number,26\n2,0.3\n3,,27\n",
        "malformed.csv",
    )
    .expect("valid rows should still be returned");

    assert_eq!(parsed.diagnostics.total_rows, 5);
    assert_eq!(parsed.diagnostics.successfully_parsed_rows, 4);
    assert_eq!(parsed.diagnostics.skipped_rows, 1);
    assert_eq!(parsed.diagnostics.malformed_rows, 3);
    assert_eq!(parsed.diagnostics.missing_values, 3);
    assert_eq!(parsed.measurement.channels[0].values[1], None);
    assert_eq!(parsed.measurement.channels[1].values[2], None);
    assert!(parsed.diagnostics.has_issues());
    assert!(
        parsed
            .diagnostics
            .messages
            .iter()
            .any(|message| message.contains("timestamp"))
    );
}

#[test]
fn reports_irregular_duplicate_and_non_monotonic_sampling() {
    let parsed = parse_measurement_text(
        "time/sec,potential/V\n0,0.1\n1,0.2\n3,0.3\n1,0.4\n",
        "sampling.csv",
    )
    .expect("sampling diagnostics should not reject finite timestamps");

    assert!(parsed.diagnostics.irregular_sampling);
    assert_eq!(parsed.diagnostics.duplicate_timestamps, 1);
    assert_eq!(parsed.diagnostics.non_monotonic_timestamps, 1);
}

#[test]
fn parses_experiment_metadata_and_orders_events() {
    let input_path = write_fixture("csv", "time/sec,potential/V\n0,0.1\n1,0.2\n");
    let metadata_path = write_fixture(
        "toml",
        r#"experiment_id = "exp-001"
sample_matrix = "aqueous buffer"

[sensor]
sensor_id = "sensor-7"
sensor_type = "ion_selective_membrane"
analyte = "K+"
manufacturer = "Example Instruments"

[reference]
reference_id = "ref-1"
electrode_type = "Ag/AgCl"

[[environmental_data]]
name = "temperature"
unit = "C"
time = [0.0, 1.0]
values = [25.0, 25.1]

[[events]]
timestamp = 2.0
kind = "flush_end"

[[events]]
timestamp = 0.5
kind = "reading_start"

[[events]]
timestamp = 1.5
kind = "concentration_step"
value = 0.001
unit = "mol/L"
analyte = "K+"
"#,
    );

    let document = load_experiment_metadata(&metadata_path).expect("metadata should parse");
    assert_eq!(document.sensor.sensor_id.as_deref(), Some("sensor-7"));
    assert_eq!(document.sample_matrix, "aqueous buffer");
    assert_eq!(document.environmental_data.len(), 1);
    assert_eq!(document.events.len(), 3);

    let (experiment, diagnostics) =
        rust_electroanalysis_cli::data_file::load_experiment(&input_path, &metadata_path)
            .expect("metadata and measurement should form an experiment");
    assert_eq!(experiment.experiment_id, "exp-001");
    assert_eq!(experiment.events[0].kind, ExperimentEventKind::ReadingStart);
    assert_eq!(
        experiment.events[1].kind,
        ExperimentEventKind::ConcentrationStep
    );
    assert_eq!(experiment.events[2].kind, ExperimentEventKind::FlushEnd);
    assert_eq!(experiment.provenance.input_sha256.len(), 64);
    assert_eq!(
        experiment
            .provenance
            .configuration_sha256
            .as_ref()
            .map(String::len),
        Some(64)
    );
    assert_eq!(diagnostics.successfully_parsed_rows, 2);

    remove_fixture(input_path);
    remove_fixture(metadata_path);
}

#[test]
fn converts_new_measurements_into_existing_plot_data() {
    let parsed = parse_measurement_text(
        "time/sec,potential/V,temperature/C\n0,0.1,25\n1,NA,25.1\n2,0.3,25.2\n",
        "plot-adapter.csv",
    )
    .expect("measurement should parse");
    let plots = measurement_to_plot_data(&parsed.measurement);

    assert_eq!(plots.len(), 2);
    assert_eq!(plots[0].label.as_deref(), Some("potential [V]"));
    assert_eq!(plots[0].x_values, vec![0.0, 2.0]);
    assert_eq!(plots[0].y_values, vec![0.1, 0.3]);
    assert_eq!(plots[1].label.as_deref(), Some("temperature [C]"));
    assert_eq!(plots[1].x_values, vec![0.0, 1.0, 2.0]);
}

#[test]
fn keeps_existing_chi_parser_and_adds_a_scientific_adapter() {
    let path = write_fixture(
        "csv",
        "2026-01-01\nOCPT\nInstrument Model: Fixture\nTime/sec,Potential/V\n0,0.1\n1,0.2\n",
    );
    let legacy = ElectrochemData::parse_file(&path).expect("legacy CHI parser should work");
    assert_eq!(legacy.x_values, vec![0.0, 1.0]);
    assert_eq!(legacy.y_values, vec![0.1, 0.2]);

    let modern = MultiChannelMeasurement::try_from(legacy.clone())
        .expect("legacy data should convert to the scientific model");
    assert_eq!(modern.time, legacy.x_values);
    assert_eq!(modern.channels[0].values, vec![Some(0.1), Some(0.2)]);

    let (_, diagnostics) = ElectrochemData::parse_file_with_diagnostics(&path)
        .expect("legacy parser diagnostics should work");
    assert_eq!(diagnostics.successfully_parsed_rows, 2);
    remove_fixture(path);
}
