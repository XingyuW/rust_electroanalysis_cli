use rust_electroanalysis_cli::data_file::load_experiment;
use rust_electroanalysis_cli::domain::{
    AnalysisProvenance, ElectrochemicalExperiment, ExperimentEvent, ExperimentEventKind,
    MeasurementChannel, MultiChannelMeasurement, SensorMetadata,
};
use rust_electroanalysis_cli::potentiometry::calibration::nernst::theoretical_slope_v_per_decade;
use rust_electroanalysis_cli::potentiometry::transient::models::TransientModelKind;
use rust_electroanalysis_cli::potentiometry::{TransientAnalysisOptions, analyze_experiment};
use rust_electroanalysis_cli::results::calibration::{
    CalibrationBranch, CalibrationObservation, CalibrationObservationSet,
    CalibrationPotentialSource,
};
use rust_electroanalysis_cli::transient_config::ResolvedTransientConfig;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("electroanalysis_phase3_{suffix}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn provenance(path: &Path) -> AnalysisProvenance {
    AnalysisProvenance {
        software_version: "phase3-integration".to_string(),
        input_path: path.to_path_buf(),
        input_sha256: "synthetic".to_string(),
        configuration_path: None,
        configuration_sha256: None,
        generation_timestamp: 1,
        git_commit: None,
    }
}

fn experiment(path: &Path) -> ElectrochemicalExperiment {
    let time = (-30..=300).map(f64::from).collect::<Vec<_>>();
    let values = time
        .iter()
        .map(|time| {
            if *time < 0.0 {
                Some(0.30)
            } else {
                Some(0.20 + 0.10 * (-time / 12.0).exp())
            }
        })
        .collect::<Vec<_>>();
    let measurement =
        MultiChannelMeasurement::new(time, vec![MeasurementChannel::new("E1", "V", values)])
            .unwrap();
    ElectrochemicalExperiment::new(
        "phase3-experiment",
        SensorMetadata {
            analyte: Some("Na+".to_string()),
            ..Default::default()
        },
        None,
        measurement,
        Vec::new(),
        vec![ExperimentEvent {
            timestamp: 0.0,
            kind: ExperimentEventKind::ConcentrationStep,
            value: Some(0.001),
            unit: Some("mol/L".to_string()),
            analyte: Some("Na+".to_string()),
            annotation: None,
            metadata: None,
        }],
        "aqueous buffer",
        provenance(path),
    )
    .unwrap()
}

fn calibration_observations() -> CalibrationObservationSet {
    let slope = theoretical_slope_v_per_decade(298.15, 1).unwrap();
    let observations = [1e-5, 1e-4, 1e-3, 1e-2, 1e-1]
        .into_iter()
        .enumerate()
        .map(|(index, activity)| CalibrationObservation {
            observation_id: format!("obs-{index}"),
            experiment_id: "fit-experiment".to_string(),
            event_index: Some(index),
            timestamp: Some(index as f64),
            analyte: "Na+".to_string(),
            ion_charge: 1,
            concentration: Some(
                rust_electroanalysis_cli::potentiometry::units::Quantity::new(
                    activity,
                    rust_electroanalysis_cli::potentiometry::units::QuantityUnit::MolPerL,
                )
                .unwrap(),
            ),
            molar_concentration_mol_l: Some(activity),
            activity: Some(activity),
            activity_coefficient: Some(1.0),
            potential_v: 0.2 + slope * activity.log10(),
            potential_standard_error_v: Some(0.001),
            temperature_k: Some(298.15),
            ionic_strength_mol_l: None,
            conductivity: None,
            interferent_activities: BTreeMap::new(),
            branch: if index == 0 {
                CalibrationBranch::Unknown
            } else {
                CalibrationBranch::Ascending
            },
            source: CalibrationPotentialSource::ExplicitObservation,
            source_fit_status: None,
            source_warnings: Vec::new(),
            steady_state: None,
            environmental_alignment: Vec::new(),
            metadata: BTreeMap::new(),
        })
        .collect();
    CalibrationObservationSet {
        schema_version: 1,
        observations,
        provenance: AnalysisProvenance {
            software_version: "test".to_string(),
            input_path: PathBuf::from("observations.json"),
            input_sha256: "synthetic".to_string(),
            configuration_path: None,
            configuration_sha256: None,
            generation_timestamp: 1,
            git_commit: None,
        },
        warnings: Vec::new(),
    }
}

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

fn run_cli(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rust_electroanalysis_cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn calibration_workflows_extract_fit_validate_and_predict() {
    let root = temp_dir();
    let input = root.join("sensor.csv");
    let metadata = root.join("experiment.toml");
    write(&input, "time/sec,E1/V\n0,0.3\n1,0.29\n");
    write(
        &metadata,
        r#"experiment_id = "phase3-experiment"
sample_matrix = "aqueous buffer"

[sensor]
analyte = "Na+"

[[events]]
timestamp = 0.0
kind = "concentration_step"
value = 0.001
unit = "mol/L"
analyte = "Na+"
"#,
    );
    let (_loaded_experiment, _) = load_experiment(&input, &metadata).unwrap();
    let mut transient_config = ResolvedTransientConfig::default();
    transient_config.models.enabled = vec![TransientModelKind::Single];
    transient_config.segmentation.post_event_s = 300.0;
    transient_config.segmentation.pre_event_s = 30.0;
    transient_config.segmentation.minimum_points = 20;
    transient_config.uncertainty.bootstrap_iterations = 0;
    transient_config.plotting.enabled = false;
    transient_config.validation.maximum_tau_to_window_ratio = 100.0;
    let transient = analyze_experiment(
        &experiment(&input),
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config: transient_config,
        },
    )
    .unwrap();
    let transient_path = root.join("transient_results.json");
    write(&transient_path, &serde_json::to_string(&transient).unwrap());
    let calibration_config = root.join("calibration.toml");
    write(
        &calibration_config,
        r#"schema_version = 1

[analyte]
name = "Na+"
charge = 1

[observation_extraction]
fallback_source = "steady_state_median"
steady_state_start_s = 1.0
steady_state_end_s = 2.0

[plotting]
enabled = false

[uncertainty]
bootstrap_iterations = 0

[validation]
mode = "none"
"#,
    );
    let extracted_path = root.join("extracted.json");
    let output = run_cli(
        &root,
        &[
            "calibration",
            "extract",
            "--input",
            input.to_str().unwrap(),
            "--metadata",
            metadata.to_str().unwrap(),
            "--channel",
            "E1/V",
            "--transient-results",
            transient_path.to_str().unwrap(),
            "--config",
            calibration_config.to_str().unwrap(),
            "--output",
            extracted_path.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let extracted: CalibrationObservationSet =
        serde_json::from_str(&fs::read_to_string(&extracted_path).unwrap()).unwrap();
    assert_eq!(
        extracted.observations[0].source,
        CalibrationPotentialSource::TransientEquilibrium
    );
    assert!(extracted.observations[0].potential_v.is_finite());

    let observations_path = root.join("observations.json");
    write(
        &observations_path,
        &serde_json::to_string(&calibration_observations()).unwrap(),
    );
    let fit_output = root.join("fit-output");
    let output = run_cli(
        &root,
        &[
            "calibration",
            "fit",
            "--observations",
            observations_path.to_str().unwrap(),
            "--config",
            calibration_config.to_str().unwrap(),
            "--bootstrap",
            "5",
            "--seed",
            "7",
            "--output",
            fit_output.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for filename in [
        "calibration_model.json",
        "calibration_results.json",
        "calibration_summary.csv",
        "calibration_residuals.csv",
        "calibration_validation.csv",
        "calibration_report.txt",
    ] {
        assert!(fit_output.join(filename).is_file(), "missing {filename}");
    }
    let fit_report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fit_output.join("calibration_results.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        fit_report["configuration"]["uncertainty"]["bootstrap_iterations"],
        5
    );
    assert_eq!(fit_report["configuration"]["uncertainty"]["seed"], 7);

    let validation_output = root.join("validation-output");
    let output = run_cli(
        &root,
        &[
            "calibration",
            "validate",
            "--model",
            fit_output.join("calibration_model.json").to_str().unwrap(),
            "--observations",
            observations_path.to_str().unwrap(),
            "--output",
            validation_output.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        validation_output
            .join("calibration_validation.csv")
            .is_file()
    );

    let prediction_path = root.join("prediction.json");
    let output = run_cli(
        &root,
        &[
            "calibration",
            "predict",
            "--model",
            fit_output.join("calibration_model.json").to_str().unwrap(),
            "--potential",
            "0.2",
            "--temperature",
            "25",
            "--output",
            prediction_path.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let prediction: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(prediction_path).unwrap()).unwrap();
    assert!(
        prediction["predicted_activity"]
            .as_f64()
            .unwrap()
            .is_finite()
    );
    let unknown = root.join("unknown.csv");
    write(&unknown, "time/sec,E1/V\n0,0.2\n1,0.21\n");
    let predictions_csv = root.join("predictions.csv");
    let output = run_cli(
        &root,
        &[
            "calibration",
            "predict",
            "--model",
            fit_output.join("calibration_model.json").to_str().unwrap(),
            "--input",
            unknown.to_str().unwrap(),
            "--channel",
            "E1/V",
            "--output",
            predictions_csv.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        fs::read_to_string(predictions_csv)
            .unwrap()
            .contains("activity")
    );
    fs::remove_dir_all(root).ok();
}
