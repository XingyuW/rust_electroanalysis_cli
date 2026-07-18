use rust_electroanalysis_cli::data_file::measurement_parser::parse_measurement_file_with_sheet;
use rust_electroanalysis_cli::estimation::simulation;
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
    let adjusted = base.replace("minimum_segment_points = 10", "minimum_segment_points = 2");
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
    assert!(
        err.to_string()
            .contains("multiple compatible time-series worksheets")
    );

    let parsed =
        parse_measurement_file_with_sheet(fixture("multi_timeseries.xlsx"), Some("SheetA"))
            .expect("explicit sheet should parse");
    assert_eq!(parsed.measurement.time.len(), 6);
}

#[test]
fn parser_rejects_eis_only_workbook_for_time_series_ingestion() {
    let err = parse_measurement_file_with_sheet(fixture("eis_only.xlsx"), None)
        .expect_err("EIS-only workbook must be rejected");
    assert!(
        err.to_string()
            .contains("XLSX EIS ingestion is not supported")
    );
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
        .current_dir(repo_path(""))
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
        .current_dir(repo_path(""))
        .output()
        .expect("run signal characterize without sheet");
    assert!(!fail.status.success());
    assert!(
        String::from_utf8_lossy(&fail.stderr)
            .contains("multiple compatible time-series worksheets")
    );

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
    fs::write(
        &model_path,
        serde_json::to_string_pretty(&simulation::simulation_model()).expect("serialize model"),
    )
    .expect("write model");

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
        .current_dir(repo_path(""))
        .output()
        .expect("run estimate run");
    assert!(
        run.status.success(),
        "estimate run failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(estimate_dir.join("state_estimation.json").is_file());

    fs::remove_dir_all(workspace).ok();
}
