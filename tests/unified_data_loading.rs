use rust_electroanalysis_cli::data_file::{DataFileType, load_data, measurement_to_plot_data};
use rust_electroanalysis_cli::domain::ElectrochemicalExperiment;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn downstream_plot_series_count(experiment: &ElectrochemicalExperiment) -> usize {
    measurement_to_plot_data(experiment.measurement()).len()
}

#[test]
fn loads_chi_eis_chi_ocpt_and_sensor_csv_with_one_interface() {
    let eis = repo_path("data/EIS/20260317/20260312_QD-5_EIS (0.1M).csv");
    let ocpt = repo_path(
        "data/Shan/20260430/20260430_(20260429-NH4-ISM-1_20260429-NH4-ISM-2)_(NH4)2SO4.csv",
    );
    let sensor = repo_path("data/Sensor-AI Data/Sensor Reading/NH4.csv");

    let loaded_eis = load_data(&eis).expect("load EIS");
    let loaded_ocpt = load_data(&ocpt).expect("load OCPT");
    let loaded_sensor = load_data(&sensor).expect("load sensor CSV");

    assert_eq!(loaded_eis.file_type, DataFileType::ChiEis);
    assert_eq!(loaded_ocpt.file_type, DataFileType::ChiOcpt);
    assert_eq!(loaded_sensor.file_type, DataFileType::SensorCsv);

    fn as_experiment(
        loaded: &rust_electroanalysis_cli::data_file::LoadedExperimentData,
    ) -> &ElectrochemicalExperiment {
        &loaded.experiment
    }
    let _ = as_experiment(&loaded_eis);
    let _ = as_experiment(&loaded_ocpt);
    let _ = as_experiment(&loaded_sensor);

    assert!(!loaded_eis.experiment.sample_matrix.is_empty());
    assert!(!loaded_ocpt.experiment.sample_matrix.is_empty());
    assert!(!loaded_sensor.experiment.sample_matrix.is_empty());
    assert!(
        loaded_sensor
            .experiment
            .sensor_metadata
            .sensor_type
            .is_some()
    );

    assert!(downstream_plot_series_count(&loaded_eis.experiment) >= 1);
    assert!(downstream_plot_series_count(&loaded_ocpt.experiment) >= 1);
    assert!(downstream_plot_series_count(&loaded_sensor.experiment) >= 1);
}

#[test]
fn rejects_binary_content_with_csv_extension() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("unified-binary-{suffix}.csv"));
    fs::write(&path, [0x00, 0xFE, 0x10, 0x20]).expect("write binary fixture");
    let err = load_data(&path).expect_err("binary csv should be rejected");
    let text = err.to_string();
    assert!(text.contains("unsupported") || text.contains("binary"));
    fs::remove_file(path).ok();
}

#[test]
fn rejects_legacy_xls_inputs() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("unified-legacy-{suffix}.xls"));
    fs::write(&path, b"legacy xls fixture").expect("write xls fixture");
    let err = load_data(&path).expect_err("xls should be rejected");
    assert!(err.to_string().contains(".xls"));
    fs::remove_file(path).ok();
}
