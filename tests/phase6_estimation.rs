use rust_electroanalysis_cli::{
    domain::{
        AnalysisProvenance, ElectrochemicalExperiment, ExperimentEvent, ExperimentEventKind,
        MeasurementChannel, MultiChannelMeasurement, SensorMetadata,
    },
    estimation::{
        self,
        calibration_adapter::{CalibrationObservationModel, StoredCalibrationObservationModel},
        environment::AlignedEnvironment,
        simulation,
        state::{CalibrationDomainStatus, MeasurementUpdateStatus},
    },
    estimation_config::{
        FilterKind, MeasurementNoiseSourceKind, ResolvedEstimationConfig, StateModelKind,
    },
};
use std::{path::PathBuf, str::FromStr};

fn provenance() -> AnalysisProvenance {
    AnalysisProvenance {
        software_version: "test".into(),
        input_path: PathBuf::from("synthetic.csv"),
        input_sha256: "synthetic".into(),
        configuration_path: None,
        configuration_sha256: None,
        generation_timestamp: 1,
        git_commit: None,
    }
}
fn experiment(values: Vec<Option<f64>>, time: Vec<f64>) -> ElectrochemicalExperiment {
    let measurement =
        MultiChannelMeasurement::new(time, vec![MeasurementChannel::new("E1", "V", values)])
            .unwrap();
    ElectrochemicalExperiment::new(
        "phase6",
        SensorMetadata::default(),
        None,
        measurement,
        Vec::new(),
        Vec::new(),
        "buffer",
        provenance(),
    )
    .unwrap()
}
fn config(model: StateModelKind) -> ResolvedEstimationConfig {
    let mut c = ResolvedEstimationConfig::default();
    c.state_model.kind = model;
    c.filter.kind = FilterKind::Ekf;
    c.measurement_noise.source = MeasurementNoiseSourceKind::Configured;
    c.measurement_noise.configured_variance_v2 = 1e-8;
    c.measurement_noise.minimum_variance_v2 = 1e-12;
    c.process_noise.activity_variance_per_s = 1e-8;
    c.process_noise.baseline_variance_v2_per_s = 1e-12;
    c.process_noise.polarization_variance_v2_per_s = 1e-10;
    c.polarization.tau_source =
        rust_electroanalysis_cli::estimation_config::TauSourceKind::Configured;
    c.observability.horizon_steps = 20;
    c.plotting.enabled = false;
    c
}

#[test]
fn activity_only_recovers_noise_free_nernst_activity_and_domain_status() {
    let model = simulation::simulation_model();
    let calibration = StoredCalibrationObservationModel::new(model.clone()).unwrap();
    let e0 = 0.2;
    let slope = 0.05916;
    let time = (0..20).map(|i| i as f64).collect::<Vec<_>>();
    let values = time.iter().map(|_| Some(e0 + slope * (-3.0))).collect();
    let report = estimation::estimate_experiment(
        &experiment(values, time),
        "E1/V",
        calibration,
        &config(StateModelKind::Activity),
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let last = report.estimates.last().unwrap();
    assert_eq!(
        last.calibration_domain_status,
        CalibrationDomainStatus::Inside
    );
    assert!((last.activity.unwrap() - 1e-3).abs() < 1e-8);
    assert!(report.diagnostics.accepted_update_count > 0);
}

#[test]
fn missing_measurements_are_predict_only_and_covariance_grows() {
    let model = simulation::simulation_model();
    let mut c = config(StateModelKind::Activity);
    let time = (0..5).map(|i| i as f64).collect::<Vec<_>>();
    let values = vec![Some(0.02252), None, None, Some(0.02252), Some(0.02252)];
    let report = estimation::estimate_experiment(
        &experiment(values, time),
        "E1/V",
        StoredCalibrationObservationModel::new(model).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    assert!(
        report
            .estimates
            .iter()
            .any(|p| p.update_status == MeasurementUpdateStatus::PredictOnly)
    );
    assert!(
        report.estimates[2].filtered_covariance[0][0]
            >= report.estimates[0].filtered_covariance[0][0]
    );
    c.filter.kind = FilterKind::Ukf;
    let _ = c;
}

#[test]
fn activity_baseline_without_auxiliary_is_rejected() {
    let model = simulation::simulation_model();
    let time = (0..10).map(|i| i as f64).collect::<Vec<_>>();
    let values = time.iter().map(|_| Some(0.02252)).collect();
    let error = estimation::estimate_experiment(
        &experiment(values, time),
        "E1/V",
        StoredCalibrationObservationModel::new(model).unwrap(),
        &config(StateModelKind::ActivityBaseline),
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap_err();
    assert!(error.to_string().contains("unobservable"));
}

#[test]
fn annotated_standard_is_recorded_as_auxiliary_state_evidence() {
    let model = simulation::simulation_model();
    let time = (0..10).map(|i| i as f64).collect::<Vec<_>>();
    let values = time.iter().map(|_| Some(0.2 - 0.05916 * 3.0)).collect();
    let mut exp = experiment(values, time);
    exp.events.push(ExperimentEvent {
        timestamp: 0.0,
        kind: ExperimentEventKind::ConcentrationStep,
        value: Some(1e-3),
        unit: Some("mol/L".into()),
        analyte: Some("synthetic".into()),
        annotation: Some("known activity standard".into()),
        metadata: None,
    });
    let report = estimation::estimate_experiment(
        &exp,
        "E1/V",
        StoredCalibrationObservationModel::new(model).unwrap(),
        &config(StateModelKind::ActivityBaseline),
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    assert!(
        report
            .estimates
            .iter()
            .any(|point| !point.auxiliary_observations.is_empty())
    );
    assert!(
        (report.estimates.last().unwrap().filtered_state[0]
            .value
            .unwrap()
            + 3.0)
            .abs()
            < 1e-6
    );
}

#[test]
fn condition_state_requires_independent_information() {
    let model = simulation::simulation_model();
    let mut c = config(StateModelKind::Activity);
    c.state_model.include_condition_state = true;
    let time = (0..10).map(|i| i as f64).collect();
    let values = (0..10).map(|_| Some(0.02252)).collect();
    let error = estimation::estimate_experiment(
        &experiment(values, time),
        "E1/V",
        StoredCalibrationObservationModel::new(model).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap_err();
    assert!(error.to_string().contains("condition state"));
}

#[test]
fn ukf_sigma_points_reproduce_mean_and_covariance() {
    let mut c = ResolvedEstimationConfig::default();
    c.ukf.alpha = 0.3;
    let mean = nalgebra::DVector::from_vec(vec![1.0, -2.0]);
    let covariance = nalgebra::DMatrix::from_row_slice(2, 2, &[2.0, 0.3, 0.3, 1.0]);
    let (points, wm, wc, _) = estimation::ukf::sigma_points(&mean, &covariance, &c).unwrap();
    let recovered = points
        .iter()
        .zip(&wm)
        .fold(nalgebra::DVector::zeros(2), |a, (p, w)| a + p * *w);
    let mut p = nalgebra::DMatrix::zeros(2, 2);
    for (point, w) in points.iter().zip(&wc) {
        let d = point - &recovered;
        p += &d * d.transpose() * *w;
    }
    assert!((recovered - &mean).norm() < 1e-10);
    assert!((p - covariance).norm() < 1e-8);
}

#[test]
fn cli_estimation_boundaries_parse_without_legacy_flags() {
    let args = vec![
        "electroanalysis",
        "estimate",
        "run",
        "--input",
        "x.csv",
        "--metadata",
        "x.toml",
        "--channel",
        "E1/V",
        "--calibration-model",
        "model.json",
    ];
    let parsed = rust_electroanalysis_cli::cli::parse_cli_args(
        &args.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
    )
    .unwrap();
    assert!(matches!(
        parsed.command,
        Some(rust_electroanalysis_cli::cli::CommandSpec::EstimateRun { .. })
    ));
    assert_eq!(FilterKind::from_str("ukf").unwrap(), FilterKind::Ukf);
}

#[test]
fn adapter_requires_nicolsky_interferent_activity() {
    let mut model = simulation::simulation_model();
    model.model_kind = rust_electroanalysis_cli::results::CalibrationModelKind::NicolskyEisenman;
    model.selectivity_coefficients.push(
        rust_electroanalysis_cli::results::SelectivityCoefficient {
            primary_analyte: "synthetic".into(),
            interferent: "K+".into(),
            value: 0.1,
            source: "test".into(),
            standard_error: None,
            confidence_interval: None,
        },
    );
    model.configuration.nicolsky_eisenman.interferents.push(
        rust_electroanalysis_cli::calibration_config::InterferentConfig {
            name: "K+".into(),
            charge: 1,
            selectivity_coefficient: Some(0.1),
            source: "test".into(),
        },
    );
    let adapter = StoredCalibrationObservationModel::new(model).unwrap();
    let error = adapter
        .predict_potential(-3.0, &AlignedEnvironment::default())
        .unwrap_err();
    assert!(error.to_string().contains("interferent"));
}
