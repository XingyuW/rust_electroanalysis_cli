use rust_electroanalysis_cli::data_file::{
    DataFileType, load_data, measurement_to_plot_data, read_dataset,
};
use rust_electroanalysis_cli::domain::{DataParsingError, ElectrochemicalExperiment};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn fixture_path(name: &str) -> std::path::PathBuf {
    repo_path("tests/fixtures/unified_data_loading").join(name)
}

fn downstream_plot_series_count(experiment: &ElectrochemicalExperiment) -> usize {
    measurement_to_plot_data(experiment.measurement()).len()
}

#[test]
fn loads_eis_ocpt_multichannel_ocpt_and_regular_data_with_one_interface() {
    let eis = fixture_path("eis.csv");
    let ocpt = fixture_path("chi_ocpt.csv");
    let multichannel_ocpt = fixture_path("chi_multichannel_ocpt.csv");
    let sensor = fixture_path("regular.csv");

    let canonical_eis = read_dataset(&eis).expect("read canonical EIS fixture");
    assert_eq!(
        canonical_eis.kind(),
        electrodata_io::DatasetKind::ImpedanceSpectrum
    );
    assert_eq!(canonical_eis.canonical_columns()[0].values.len(), 3);
    let eis_columns = canonical_eis.canonical_columns();
    assert_eq!(eis_columns[0].role, electrodata_io::ColumnRole::Frequency);
    assert_eq!(eis_columns[0].unit, Some(electrodata_io::Unit::Hertz));
    assert_eq!(
        eis_columns[1].role,
        electrodata_io::ColumnRole::ImpedanceReal
    );
    assert_eq!(eis_columns[1].unit, Some(electrodata_io::Unit::Ohm));
    assert_eq!(
        eis_columns[2].role,
        electrodata_io::ColumnRole::ImpedanceImaginary
    );
    assert_eq!(eis_columns[2].unit, Some(electrodata_io::Unit::Ohm));
    assert_eq!(
        eis_columns[3].role,
        electrodata_io::ColumnRole::ImpedanceMagnitude
    );
    assert_eq!(eis_columns[3].unit, Some(electrodata_io::Unit::Ohm));
    assert_eq!(
        eis_columns[4].role,
        electrodata_io::ColumnRole::ImpedancePhase
    );
    assert_eq!(eis_columns[4].unit, Some(electrodata_io::Unit::Degree));
    let eis_view = canonical_eis.eis_view().expect("required EIS components");
    assert_eq!(
        eis_view.frequency.values,
        vec![Some(1000.0), Some(100.0), Some(10.0)]
    );
    assert_eq!(
        eis_view.real.values,
        vec![Some(10.0), Some(15.0), Some(20.0)]
    );
    assert_eq!(
        eis_view.imaginary.values,
        vec![Some(-1.0), Some(-5.0), Some(-10.0)]
    );
    assert!(eis_view.measured_magnitude.is_some());
    assert!(eis_view.measured_phase.is_some());

    for (path, expected_channels, expected_coordinate, expected_channel_headers) in [
        (&ocpt, 1, "Time/sec", vec!["Potential/V"]),
        (&multichannel_ocpt, 2, "Time/sec", vec!["E1/V", "E5/V"]),
        (&sensor, 1, "Time/sec", vec!["Potential/V"]),
    ] {
        let dataset = read_dataset(path).expect("read canonical time-series fixture");
        assert_eq!(dataset.kind(), electrodata_io::DatasetKind::TimeSeries);
        let view = dataset.time_series_view().expect("time-series roles");
        assert_eq!(view.time.values, vec![Some(0.0), Some(1.0), Some(2.0)]);
        assert_eq!(
            view.time.original_name.as_deref(),
            Some(expected_coordinate)
        );
        assert_eq!(view.time.unit, Some(electrodata_io::Unit::Second));
        assert_eq!(view.measurements.len(), expected_channels);
        assert_eq!(
            view.measurements
                .iter()
                .map(|channel| channel.original_name.as_deref())
                .collect::<Vec<_>>(),
            expected_channel_headers
                .into_iter()
                .map(Some)
                .collect::<Vec<_>>()
        );
        assert!(
            view.measurements
                .iter()
                .all(|channel| channel.unit == Some(electrodata_io::Unit::Volt))
        );
    }

    let loaded_eis = load_data(&eis).expect("load EIS");
    let loaded_ocpt = load_data(&ocpt).expect("load OCPT");
    let loaded_multichannel_ocpt = load_data(&multichannel_ocpt).expect("load multichannel OCPT");
    let loaded_sensor = load_data(&sensor).expect("load sensor CSV");

    assert_eq!(loaded_eis.file_type, DataFileType::ChiEis);
    assert_eq!(loaded_ocpt.file_type, DataFileType::ChiOcpt);
    assert_eq!(loaded_multichannel_ocpt.file_type, DataFileType::ChiOcpt);
    assert_eq!(loaded_sensor.file_type, DataFileType::SensorCsv);
    assert_eq!(loaded_eis.experiment.measurement().time.len(), 3);
    assert_eq!(loaded_ocpt.experiment.measurement().time.len(), 3);
    assert_eq!(
        loaded_multichannel_ocpt.experiment.measurement().time.len(),
        3
    );
    assert_eq!(loaded_sensor.experiment.measurement().time.len(), 3);
    assert_eq!(loaded_ocpt.experiment.measurement().time_unit, "s");
    assert_eq!(loaded_sensor.experiment.measurement().time_unit, "s");
    assert_eq!(loaded_ocpt.experiment.measurement().channels.len(), 1);
    assert!(
        loaded_multichannel_ocpt
            .experiment
            .measurement()
            .channels
            .len()
            >= 2
    );

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
    assert_eq!(
        loaded_ocpt.experiment.measurement().channels[0].name,
        "Potential"
    );
    assert_eq!(loaded_ocpt.experiment.measurement().channels[0].unit, "V");
    assert_eq!(
        loaded_multichannel_ocpt.experiment.measurement().channels[0].name,
        "E1"
    );
    assert_eq!(
        loaded_multichannel_ocpt.experiment.measurement().channels[1].name,
        "E5"
    );
    assert_eq!(
        loaded_sensor.experiment.measurement().channels[0].name,
        "Potential"
    );
    assert_eq!(loaded_sensor.experiment.measurement().channels[0].unit, "V");

    assert!(downstream_plot_series_count(&loaded_eis.experiment) >= 1);
    assert!(downstream_plot_series_count(&loaded_ocpt.experiment) >= 1);
    assert!(downstream_plot_series_count(&loaded_multichannel_ocpt.experiment) >= 2);
    assert!(downstream_plot_series_count(&loaded_sensor.experiment) >= 1);
}

#[test]
fn unified_loading_fixtures_are_committed_repository_inputs() {
    let repository = fs::canonicalize(repo_path("")).expect("canonical repository root");
    let root = fs::canonicalize(repo_path("tests/fixtures/unified_data_loading"))
        .expect("canonical fixture directory");
    assert!(root.starts_with(repository));
    for name in [
        "eis.csv",
        "chi_ocpt.csv",
        "chi_multichannel_ocpt.csv",
        "regular.csv",
    ] {
        let path = fs::canonicalize(root.join(name)).expect("canonical fixture path");
        assert!(
            path.is_file(),
            "missing committed fixture: {}",
            path.display()
        );
        assert!(path.starts_with(&root));
    }
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
    match err {
        DataParsingError::ElectrodataIo(error) => match error.as_ref() {
            electrodata_io::Error::UnsupportedBinary {
                path: error_path,
                magic,
                ..
            } => {
                assert_eq!(error_path, &path);
                assert!(!magic.is_empty());
            }
            other => panic!("expected canonical UnsupportedBinary, got {other:?}"),
        },
        other => panic!("expected canonical error wrapper, got {other:?}"),
    }
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
