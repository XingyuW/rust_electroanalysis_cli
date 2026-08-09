use rust_electroanalysis_cli::estimation::simulation;
use rust_electroanalysis_cli::{
    data_file::{EISData, measurement_parser::parse_measurement_file_with_sheet},
    domain::{DataParsingError, write_artifact},
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static CLI_LOCK: Mutex<()> = Mutex::new(());

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn fixture(name: &str) -> PathBuf {
    repo_path(&format!("tests/fixtures/xlsx/{name}"))
}

fn temp_workspace(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rust_electroanalysis_xlsx_{prefix}_{}_{}",
        std::process::id(),
        nonce
    ));
    fs::create_dir_all(&root).expect("create temp workspace");
    root
}

fn write_metadata(root: &Path) -> PathBuf {
    let path = root.join("metadata.toml");
    fs::write(
        &path,
        "experiment_id = 'xlsx-test'\nsample_matrix = 'buffer'\n\n[sensor]\nsensor_id = 's1'\n",
    )
    .expect("write metadata");
    path
}

fn write_estimation_config_for_short_segments(root: &Path) -> PathBuf {
    let base = fs::read_to_string(repo_path("config/estimation.toml")).expect("read base config");
    let adjusted = base
        .replace("minimum_segment_points = 10", "minimum_segment_points = 2")
        .replace(
            "kind = \"activity_baseline_polarization\"",
            "kind = \"activity\"",
        );
    let path = root.join("estimation_short_segments.toml");
    fs::write(&path, adjusted).expect("write adjusted estimation config");
    path
}

#[test]
fn parser_auto_selects_single_compatible_sheet() {
    let parsed = parse_measurement_file_with_sheet(fixture("single_timeseries.xlsx"), None)
        .expect("single sheet workbook should parse");
    assert_eq!(parsed.measurement.time.len(), 3);
    assert!(
        parsed
            .diagnostics
            .messages
            .iter()
            .any(|message| message.contains("worksheet selected: 'measurement'"))
    );
}

#[test]
fn parser_requires_sheet_when_multiple_compatible_sheets_exist() {
    let err = parse_measurement_file_with_sheet(fixture("multi_timeseries.xlsx"), None)
        .expect_err("ambiguous workbook should fail without --sheet");
    match err {
        DataParsingError::ElectrodataIo(error) => match error.as_ref() {
            electrodata_io::Error::AmbiguousWorksheet { path, candidates } => {
                assert!(path.ends_with("multi_timeseries.xlsx"));
                assert_eq!(candidates.len(), 2);
                assert!(
                    candidates
                        .iter()
                        .any(|candidate| candidate.name == "SheetA")
                );
                assert!(
                    candidates
                        .iter()
                        .any(|candidate| candidate.name == "SheetB")
                );
            }
            other => panic!("expected canonical AmbiguousWorksheet, got {other:?}"),
        },
        other => panic!("expected canonical error wrapper, got {other:?}"),
    }

    let parsed =
        parse_measurement_file_with_sheet(fixture("multi_timeseries.xlsx"), Some("SheetA"))
            .expect("explicit sheet should parse");
    assert_eq!(parsed.measurement.time.len(), 6);
}

#[test]
fn parser_rejects_eis_only_workbook_for_time_series_ingestion() {
    let err = parse_measurement_file_with_sheet(fixture("eis_only.xlsx"), None)
        .expect_err("EIS-only workbook must be rejected");
    assert!(err.to_string().contains("cannot be viewed as TimeSeries"));
}

#[test]
fn canonical_eis_xlsx_honors_explicit_worksheet_selection() {
    let data = EISData::parse_file_with_sheet(fixture("eis_only.xlsx"), Some("EIS"))
        .expect("explicit EIS worksheet should parse canonically");
    assert!(!data.freq.is_empty());
    let err = EISData::parse_file_with_sheet(fixture("eis_only.xlsx"), Some("missing"))
        .expect_err("missing worksheet must retain canonical selection error");
    match err {
        DataParsingError::ElectrodataIo(error) => match error.as_ref() {
            electrodata_io::Error::MissingWorksheet { worksheet, .. } => {
                assert_eq!(worksheet, "missing")
            }
            other => panic!("expected canonical MissingWorksheet, got {other:?}"),
        },
        other => panic!("expected canonical error wrapper, got {other:?}"),
    }
}

#[test]
fn cli_signal_characterize_supports_sheet_selection_for_xlsx() {
    let _guard = CLI_LOCK.lock().expect("lock");
    let workspace = temp_workspace("signal");
    let metadata = write_metadata(&workspace);
    let output_dir = workspace.join("signal_output");
    let binary = env!("CARGO_BIN_EXE_rust_electroanalysis_cli");

    let ok = Command::new(binary)
        .args([
            "signal",
            "characterize",
            "--input",
            fixture("multi_timeseries.xlsx")
                .to_str()
                .expect("fixture path"),
            "--metadata",
            metadata.to_str().expect("metadata path"),
            "--sheet",
            "SheetA",
            "--channel",
            "E/V",
            "--config",
            repo_path("config/signal.toml")
                .to_str()
                .expect("config path"),
            "--output",
            output_dir.to_str().expect("output path"),
        ])
        .current_dir(&workspace)
        .output()
        .expect("run signal characterize");
    assert!(
        ok.status.success(),
        "signal characterize failed: {}",
        String::from_utf8_lossy(&ok.stderr)
    );

    let fail = Command::new(binary)
        .args([
            "signal",
            "characterize",
            "--input",
            fixture("multi_timeseries.xlsx")
                .to_str()
                .expect("fixture path"),
            "--metadata",
            metadata.to_str().expect("metadata path"),
            "--channel",
            "E/V",
            "--config",
            repo_path("config/signal.toml")
                .to_str()
                .expect("config path"),
            "--output",
            output_dir.to_str().expect("output path"),
        ])
        .current_dir(&workspace)
        .output()
        .expect("run signal characterize without sheet");
    assert!(!fail.status.success());
    assert!(String::from_utf8_lossy(&fail.stderr).contains("ambiguous compatible worksheets"));

    fs::remove_dir_all(workspace).ok();
}

#[test]
fn cli_estimate_run_accepts_xlsx_time_series() {
    let _guard = CLI_LOCK.lock().expect("lock");
    let workspace = temp_workspace("estimate");
    let metadata = write_metadata(&workspace);
    let model_path = workspace.join("simulation_calibration_model.json");
    let estimate_dir = workspace.join("estimate");
    let estimation_config = write_estimation_config_for_short_segments(&workspace);
    let binary = env!("CARGO_BIN_EXE_rust_electroanalysis_cli");
    write_artifact(&model_path, &simulation::simulation_model()).expect("write model");

    let run = Command::new(binary)
        .args([
            "estimate",
            "run",
            "--input",
            fixture("single_timeseries.xlsx")
                .to_str()
                .expect("fixture path"),
            "--metadata",
            metadata.to_str().expect("metadata path"),
            "--sheet",
            "measurement",
            "--channel",
            "E/V",
            "--calibration-model",
            model_path.to_str().expect("cal model path"),
            "--config",
            estimation_config.to_str().expect("config path"),
            "--output",
            estimate_dir.to_str().expect("estimate output path"),
            "--filter",
            "ukf",
            "--model",
            "activity",
            "--seed",
            "42",
        ])
        .current_dir(&workspace)
        .output()
        .expect("run estimate run");
    assert!(
        run.status.success(),
        "estimate run failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(estimate_dir.join("state_estimation.json").is_file());

    let compare_dir = workspace.join("compare");
    let compare = Command::new(binary)
        .args([
            "estimate",
            "compare",
            "--input",
            fixture("single_timeseries.xlsx")
                .to_str()
                .expect("fixture path"),
            "--metadata",
            metadata.to_str().expect("metadata path"),
            "--sheet",
            "measurement",
            "--channel",
            "E/V",
            "--calibration-model",
            model_path.to_str().expect("cal model path"),
            "--config",
            estimation_config.to_str().expect("config path"),
            "--output",
            compare_dir.to_str().expect("compare output path"),
        ])
        .current_dir(&workspace)
        .output()
        .expect("run estimate compare");
    assert!(
        compare.status.success(),
        "estimate compare failed: {}",
        String::from_utf8_lossy(&compare.stderr)
    );
    let comparison: serde_json::Value = serde_json::from_slice(
        &fs::read(compare_dir.join("state_filter_comparison.json")).expect("comparison artifact"),
    )
    .expect("comparison JSON");
    assert!(comparison.get("ingestion_diagnostics").is_some());

    fs::remove_dir_all(workspace).ok();
}

#[test]
fn estimate_run_and_compare_share_canonical_ingestion_policy() {
    let _guard = CLI_LOCK.lock().expect("lock");
    let workspace = temp_workspace("ingestion_parity");
    let metadata = write_metadata(&workspace);
    let model_path = workspace.join("simulation_calibration_model.json");
    let config = write_estimation_config_for_short_segments(&workspace);
    let binary = env!("CARGO_BIN_EXE_rust_electroanalysis_cli");
    write_artifact(&model_path, &simulation::simulation_model()).expect("write model");

    let wholly_missing = workspace.join("wholly_missing.csv");
    fs::write(&wholly_missing, "Time/sec,Potential/V\n0,\n1,\n2,\n")
        .expect("write wholly missing fixture");
    let fixture_dir = repo_path("tests/fixtures/io_migration");
    let cases = [
        (fixture_dir.join("regular_two_column.csv"), true),
        (fixture_dir.join("regular_malformed_timestamp.csv"), false),
        (fixture_dir.join("regular_invalid_numeric.csv"), false),
        (fixture_dir.join("regular_ragged_rows.csv"), true),
        (fixture_dir.join("regular_missing_cells.csv"), false),
        (wholly_missing, false),
    ];

    for (index, (input, expected_success)) in cases.iter().enumerate() {
        let run_output = workspace.join(format!("run_{index}"));
        let compare_output = workspace.join(format!("compare_{index}"));
        let common = [
            "--input",
            input.to_str().expect("input path"),
            "--metadata",
            metadata.to_str().expect("metadata path"),
            "--channel",
            "Potential/V",
            "--calibration-model",
            model_path.to_str().expect("model path"),
            "--config",
            config.to_str().expect("config path"),
        ];
        let run = Command::new(binary)
            .args(["estimate", "run"])
            .args(common)
            .args(["--output", run_output.to_str().expect("output path")])
            .current_dir(&workspace)
            .output()
            .expect("estimate run");
        let compare = Command::new(binary)
            .args(["estimate", "compare"])
            .args(common)
            .args(["--output", compare_output.to_str().expect("output path")])
            .current_dir(&workspace)
            .output()
            .expect("estimate compare");
        assert_eq!(run.status.success(), compare.status.success(), "{input:?}");
        assert_eq!(run.status.success(), *expected_success, "{input:?}");
        if *expected_success {
            let run_json =
                fs::read_to_string(run_output.join("state_estimation.json")).expect("run artifact");
            let compare_json =
                fs::read_to_string(compare_output.join("state_filter_comparison.json"))
                    .expect("comparison artifact");
            assert!(run_json.contains("ingestion_diagnostics"));
            assert!(compare_json.contains("ingestion_diagnostics"));
        }
    }

    fs::remove_dir_all(workspace).ok();
}
