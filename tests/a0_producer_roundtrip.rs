use rust_electroanalysis_cli::{
    data_file::EISData,
    domain::{
        AnalysisProvenance, ElectrochemicalExperiment, ExperimentEvent, ExperimentEventKind,
        MeasurementChannel, MultiChannelMeasurement, SensorMetadata, read_artifact, write_artifact,
    },
    health::{assessment::assemble, trend::report as health_trend_report},
    health_config::ResolvedHealthConfig,
    impedance::parse_circuit_string,
    potentiometry::{TransientAnalysisOptions, analyze_experiment},
    potentiometry::{calibration::extract_observations, calibration::fit_calibration},
    results::{
        CalibrationBranch, CalibrationObservation, CalibrationObservationSet,
        CalibrationPotentialSource, CircuitFitResult, EisFitArtifact, HealthFeature,
        HealthTrendReport, MechanismAnalysisReport, SensorHealthAssessment, SignalAnalysisReport,
        StoredCalibrationModel, TransientAnalysisReport,
    },
    signal::analyze_measurement,
    signal_config::ResolvedSignalConfig,
    transient_config::ResolvedTransientConfig,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("a0_producer_roundtrip_{nonce}"));
    fs::create_dir_all(&path).expect("temporary directory");
    path
}

fn provenance(path: &Path) -> AnalysisProvenance {
    AnalysisProvenance {
        software_version: "a0-test".into(),
        input_path: path.into(),
        input_sha256: "a0-test".into(),
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
            Some(if *time < 0.0 {
                0.30
            } else {
                0.20 + 0.10 * (-time / 12.0).exp()
            })
        })
        .collect::<Vec<_>>();
    let measurement = MultiChannelMeasurement::new(
        time,
        vec![MeasurementChannel::new("E1", "V", values).with_source_header("E1/V")],
    )
    .expect("measurement");
    ElectrochemicalExperiment::new(
        "a0-experiment",
        SensorMetadata {
            analyte: Some("Na+".into()),
            ..Default::default()
        },
        None,
        measurement,
        Vec::new(),
        vec![ExperimentEvent {
            timestamp: 0.0,
            kind: ExperimentEventKind::ConcentrationStep,
            value: Some(0.001),
            unit: Some("mol/L".into()),
            analyte: Some("Na+".into()),
            annotation: None,
            metadata: None,
        }],
        "aqueous buffer",
        provenance(path),
    )
    .expect("experiment")
}

fn transient_config() -> ResolvedTransientConfig {
    let mut config = ResolvedTransientConfig::default();
    config.models.enabled = vec![
        rust_electroanalysis_cli::potentiometry::transient::models::TransientModelKind::Single,
    ];
    config.segmentation.post_event_s = 300.0;
    config.segmentation.pre_event_s = 30.0;
    config.segmentation.minimum_points = 20;
    config.uncertainty.bootstrap_iterations = 0;
    config.plotting.enabled = false;
    config.validation.maximum_tau_to_window_ratio = 100.0;
    config
}

fn calibration_input(path: &Path) -> CalibrationObservationSet {
    let slope = rust_electroanalysis_cli::potentiometry::calibration::nernst::theoretical_slope_v_per_decade(298.15, 1)
        .expect("theoretical slope");
    let observations = [1e-5, 1e-4, 1e-3, 1e-2, 1e-1]
        .into_iter()
        .enumerate()
        .map(|(index, activity)| CalibrationObservation {
            observation_id: format!("obs-{index}"),
            experiment_id: "a0-calibration".into(),
            event_index: Some(index),
            timestamp: Some(index as f64),
            analyte: "Na+".into(),
            ion_charge: 1,
            concentration: Some(
                rust_electroanalysis_cli::potentiometry::units::Quantity::new(
                    activity,
                    rust_electroanalysis_cli::potentiometry::units::QuantityUnit::MolPerL,
                )
                .expect("quantity"),
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
        schema_version: 2,
        observations,
        provenance: provenance(path),
        warnings: Vec::new(),
    }
}

fn eis(path: &Path) -> EisFitArtifact {
    let circuit = "R0";
    let node = parse_circuit_string(circuit).expect("circuit");
    let input = EISData {
        date: String::new(),
        test_type: "EIS".into(),
        instrument_model: String::new(),
        freq: vec![1.0, 10.0],
        phase: vec![0.0, 0.0],
        z_re: vec![1.0, 1.0],
        z_im: vec![0.0, 0.0],
        measured_magnitude: None,
        measured_phase: None,
        derived_magnitude: vec![1.0, 1.0],
        derived_phase: vec![0.0, 0.0],
        label: "a0".into(),
        metadata: BTreeMap::new(),
        circuit_model: circuit.into(),
    };
    let fit = CircuitFitResult {
        fitted_parameters: vec![1.0],
        parameter_names: node.get_param_names(),
        parameter_units: node.get_param_units(),
        fitted_z_re: vec![1.0, 1.0],
        fitted_z_im: vec![0.0, 0.0],
        fitted_magnitude: vec![1.0, 1.0],
        fitted_phase: vec![0.0, 0.0],
    };
    EisFitArtifact::from_fit(&input, circuit, &fit, provenance(path))
}

fn roundtrip<T: rust_electroanalysis_cli::VersionedArtifact>(path: &Path, value: &T) -> T {
    write_artifact(path, value).expect("write artifact");
    read_artifact(path).expect("reread artifact")
}

#[test]
fn mhi_t02d_legacy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/a0_artifact_contracts/schema1");

    let transient: TransientAnalysisReport =
        read_artifact(&root.join("transient_analysis.schema1.json")).expect("transient");
    assert_eq!(transient.schema_version, 1);
    assert_eq!(transient.experiment_id, "a0-experiment");
    assert_eq!(transient.channel, "E1");
    assert_eq!(transient.events.len(), 1);
    assert_eq!(transient.events[0].candidate_fits.len(), 1);

    let observations: CalibrationObservationSet =
        read_artifact(&root.join("calibration_observations.schema1.json"))
            .expect("calibration observations");
    assert_eq!(observations.schema_version, 1);
    assert_eq!(observations.observations.len(), 1);
    assert_eq!(observations.observations[0].analyte, "Na+");
    assert_eq!(observations.observations[0].potential_v, 0.2);

    let model: StoredCalibrationModel =
        read_artifact(&root.join("calibration_model.schema1.json")).expect("calibration model");
    assert_eq!(model.schema_version, 1);
    assert_eq!(model.analyte, "Na+");
    assert!(
        model
            .parameters
            .iter()
            .any(|parameter| parameter.name == "slope")
    );

    let calibration: rust_electroanalysis_cli::results::CalibrationAnalysisReport =
        read_artifact(&root.join("calibration_analysis.schema1.json"))
            .expect("calibration analysis");
    assert_eq!(calibration.schema_version, 1);
    assert_eq!(calibration.calibration_id, "Na+-calibration");
    assert_eq!(calibration.analyte, "Na+");

    let signal: SignalAnalysisReport =
        read_artifact(&root.join("signal_analysis.schema1.json")).expect("signal analysis");
    assert_eq!(signal.schema_version, 1);
    assert_eq!(signal.channel, "E1");
    assert_eq!(signal.unit, "V");
    assert!(!signal.analysis_timestamps.is_empty());

    let mechanism: MechanismAnalysisReport =
        read_artifact(&root.join("mechanism_analysis.schema1.json")).expect("mechanism");
    assert_eq!(mechanism.schema_version, 1);
    assert!(mechanism.analysis_id.starts_with("mechanism:"));
    assert_eq!(mechanism.transient_timescales.len(), 1);
    assert_eq!(mechanism.transient_timescales[0].value_s, 12.0);

    let assessment: SensorHealthAssessment =
        read_artifact(&root.join("health_assessment.schema1.json")).expect("health assessment");
    assert_eq!(assessment.schema_version, 1);
    assert_eq!(assessment.experiment_id.as_deref(), Some("a0-experiment"));
    assert_eq!(assessment.features.len(), 1);
    assert_eq!(assessment.features[0].value, Some(0.2));

    let trend: HealthTrendReport =
        read_artifact(&root.join("health_trend.schema1.json")).expect("health trend");
    assert_eq!(trend.schema_version, 1);
    assert_eq!(trend.analysis_id, "a0-trend");
}

#[test]
fn mhi_t02a_current_correct_kind() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/a0_artifact_contracts/schema2");

    let transient: TransientAnalysisReport =
        read_artifact(&root.join("transient_analysis.schema2.json")).expect("transient");
    assert_eq!(transient.schema_version, 2);
    assert_eq!(transient.channel, "E1");
    assert_eq!(transient.events.len(), 1);

    let observations: CalibrationObservationSet =
        read_artifact(&root.join("calibration_observations.schema2.json"))
            .expect("calibration observations");
    assert_eq!(observations.schema_version, 2);
    assert_eq!(observations.observations[0].analyte, "Na+");
    assert_eq!(observations.observations[0].potential_v, 0.2);

    let model: StoredCalibrationModel =
        read_artifact(&root.join("calibration_model.schema2.json")).expect("calibration model");
    assert_eq!(model.schema_version, 2);
    assert_eq!(model.analyte, "Na+");
    assert!(
        model
            .parameters
            .iter()
            .any(|parameter| parameter.name == "slope")
    );

    let calibration: rust_electroanalysis_cli::results::CalibrationAnalysisReport =
        read_artifact(&root.join("calibration_analysis.schema2.json"))
            .expect("calibration analysis");
    assert_eq!(calibration.schema_version, 2);
    assert_eq!(calibration.calibration_id, "Na+-calibration");
    assert_eq!(calibration.analyte, "Na+");

    let signal: SignalAnalysisReport =
        read_artifact(&root.join("signal_analysis.schema2.json")).expect("signal analysis");
    assert_eq!(signal.schema_version, 2);
    assert_eq!(signal.channel, "E1");
    assert_eq!(signal.unit, "V");
    assert!(!signal.analysis_timestamps.is_empty());

    let mechanism: MechanismAnalysisReport =
        read_artifact(&root.join("mechanism_analysis.schema2.json")).expect("mechanism");
    assert_eq!(mechanism.schema_version, 2);
    assert!(mechanism.analysis_id.starts_with("mechanism:"));
    assert_eq!(mechanism.transient_timescales.len(), 1);

    let assessment: SensorHealthAssessment =
        read_artifact(&root.join("health_assessment.schema2.json")).expect("health assessment");
    assert_eq!(assessment.schema_version, 2);
    assert_eq!(assessment.experiment_id.as_deref(), Some("a0-experiment"));
    assert_eq!(assessment.features[0].value, Some(0.2));

    let trend: HealthTrendReport =
        read_artifact(&root.join("health_trend.schema2.json")).expect("health trend");
    assert_eq!(trend.schema_version, 2);
    assert_eq!(trend.analysis_id, "a0-trend");
}

#[test]
fn mhi_t02f_producer_roundtrip() {
    let root = temp_dir();
    fs::create_dir_all(root.join("config")).unwrap();
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::copy(
        repository.join("config/mechanism.toml"),
        root.join("config/mechanism.toml"),
    )
    .unwrap();
    fs::copy(
        repository.join("config/health.toml"),
        root.join("config/health.toml"),
    )
    .unwrap();
    let input = root.join("input.csv");
    let experiment = experiment(&input);
    let transient = analyze_experiment(
        &experiment,
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config: transient_config(),
        },
    )
    .expect("transient producer");
    assert_eq!(transient.schema_version, 2);
    let transient: TransientAnalysisReport = roundtrip(&root.join("transient.json"), &transient);

    let mut extraction_config =
        rust_electroanalysis_cli::calibration_config::ResolvedCalibrationConfig::default();
    extraction_config.observation_extraction.fallback_source =
        Some(CalibrationPotentialSource::SteadyStateWindowMedian);
    extraction_config
        .observation_extraction
        .steady_state_start_s = 1.0;
    extraction_config.observation_extraction.steady_state_end_s = 120.0;
    let observations =
        extract_observations(&experiment, "E1/V", Some(&transient), &extraction_config)
            .expect("calibration observations producer");
    roundtrip(&root.join("observations.json"), &observations);

    let calibration = fit_calibration(&calibration_input(&input), &extraction_config)
        .expect("calibration analysis producer");
    let calibration = roundtrip(&root.join("calibration.json"), &calibration);
    let model: StoredCalibrationModel =
        rust_electroanalysis_cli::potentiometry::calibration::stored_model_from_report(
            &calibration,
        )
        .expect("calibration model producer");
    roundtrip(&root.join("model.json"), &model);

    let mut signal_config = ResolvedSignalConfig::default();
    signal_config.sampling.policy =
        rust_electroanalysis_cli::signal_config::SamplingPolicy::AllowIrregularTimeDomainOnly;
    let signal = analyze_measurement(
        experiment.measurement(),
        "E1/V",
        Some(&experiment.events),
        &signal_config,
        Some(experiment.provenance.clone()),
    )
    .expect("signal producer");
    roundtrip(&root.join("signal.json"), &signal);

    let eis = eis(&root.join("eis.csv"));
    roundtrip(&root.join("eis.json"), &eis);
    let mechanism_output = root.join("mechanism");
    rust_electroanalysis_cli::runners::mechanism::compare(
        &root,
        Path::new("eis.json"),
        Path::new("transient.json"),
        None,
        None,
        None,
        Some(&mechanism_output),
    )
    .expect("mechanism compare producer");
    let _: MechanismAnalysisReport =
        read_artifact(&mechanism_output.join("mechanism_results.json"))
            .expect("mechanism compare reread");

    fs::write(
        root.join("mechanism.toml"),
        "schema_version = 1\n[[records]]\nrecord_id = 'a0'\nexperiment_id = 'a0-experiment'\neis_fit = 'eis.json'\ntransient_results = 'transient.json'\n",
    )
    .unwrap();
    let mechanism_trend_output = root.join("mechanism-trend");
    rust_electroanalysis_cli::runners::mechanism::trend(
        &root,
        Path::new("mechanism.toml"),
        None,
        Some(&mechanism_trend_output),
    )
    .expect("mechanism trend producer");
    let _: MechanismAnalysisReport =
        read_artifact(&mechanism_trend_output.join("mechanism_results.json"))
            .expect("mechanism trend reread");

    let assessment = assemble(
        "a0-assessment",
        None,
        Some("a0-experiment".into()),
        vec![HealthFeature {
            name: "signal.mean".into(),
            value: Some(0.2),
            unit: "V".into(),
            domain: rust_electroanalysis_cli::results::HealthDomain::SignalNoise,
            source: "signal".into(),
            warning: None,
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        ResolvedHealthConfig::default(),
        provenance(&input),
        Vec::new(),
    );
    let assessment: SensorHealthAssessment = roundtrip(&root.join("assessment.json"), &assessment);
    let trend: HealthTrendReport =
        health_trend_report("a0-trend", Vec::new(), assessment.provenance.clone());
    roundtrip(&root.join("health-trend.json"), &trend);
    fs::remove_dir_all(root).ok();
}
