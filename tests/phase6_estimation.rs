use rust_electroanalysis_cli::{
    domain::{
        AnalysisProvenance, ElectrochemicalExperiment, ExperimentEvent, ExperimentEventKind,
        MeasurementChannel, MultiChannelMeasurement, SensorMetadata,
    },
    estimation::{
        self,
        calibration_adapter::{CalibrationObservationModel, StoredCalibrationObservationModel},
        environment::{
            AlignedEnvironment, AlignmentMethod, align_experiment,
            align_experiment_with_polarization, resolve_standard_activity,
        },
        measurement::observations,
        simulation,
        state::{CalibrationDomainStatus, MeasurementUpdateStatus},
    },
    estimation_config::{
        EnvironmentConfig, FilterKind, MeasurementNoiseSourceKind, PolarizationInputModel,
        ResolvedEstimationConfig, StateModelKind,
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
fn measurement_adapter_converts_potential_and_variance_to_volts() {
    let measurement = MultiChannelMeasurement::new(
        vec![0.0, 1.0],
        vec![
            MeasurementChannel::new("E", "mV", vec![Some(100.0), Some(200.0)])
                .with_variance(vec![Some(4.0), Some(9.0)]),
        ],
    )
    .unwrap();
    let rows = observations(&measurement, "E").unwrap();
    assert_eq!(rows[0].potential_v, Some(0.1));
    assert_eq!(rows[0].observation_variance_v2, Some(4e-6));
    assert!(
        observations(
            &MultiChannelMeasurement::new(
                vec![0.0, 1.0],
                vec![MeasurementChannel::from_values(
                    "E",
                    "mol/L",
                    vec![1.0, 2.0]
                )]
            )
            .unwrap(),
            "E"
        )
        .is_err()
    );
}

#[test]
fn nicolsky_derivative_uses_supplied_activity_and_preserves_sign() {
    use rust_electroanalysis_cli::potentiometry::calibration::nicolsky_eisenman::{
        InterferentModelInput, derivative_log10_activity, evaluate_potential,
    };
    let interferents = vec![InterferentModelInput {
        name: "K".into(),
        charge: 1,
        activity: 1e-2,
        selectivity_coefficient: 0.1,
    }];
    for &x in &[-6.0, -3.0, 0.0] {
        let h = 1e-6;
        let numerical = (evaluate_potential(0.2, 10_f64.powf(x + h), 1, 298.15, &interferents)
            .unwrap()
            - evaluate_potential(0.2, 10_f64.powf(x - h), 1, 298.15, &interferents).unwrap())
            / (2.0 * h);
        let analytical =
            derivative_log10_activity(10_f64.powf(x), 1, 298.15, &interferents).unwrap();
        assert!((numerical - analytical).abs() < 1e-8);
        let negative = derivative_log10_activity(10_f64.powf(x), -1, 298.15, &[]).unwrap();
        assert!(analytical > 0.0 && negative < 0.0);
    }
}

#[test]
fn polarization_input_is_one_shot_and_conservative_by_default() {
    let mut exp = experiment(vec![Some(0.0); 4], vec![0.0, 1.0, 2.0, 3.0]);
    exp.events.push(ExperimentEvent {
        timestamp: 1.5,
        kind: ExperimentEventKind::ConcentrationStep,
        value: Some(1e-3),
        unit: Some("mol/L".into()),
        analyte: None,
        annotation: Some("standard".into()),
        metadata: Some([("polarization_input_v".into(), "0.02".into())].into()),
    });
    let mut p = ResolvedEstimationConfig::default().polarization;
    p.input_model = PolarizationInputModel::ExplicitEventVoltage;
    p.input_event_kind = Some("concentration_step".into());
    let e0 = align_experiment_with_polarization(&exp, 2.0, &EnvironmentConfig::default(), None, &p)
        .unwrap();
    assert_eq!(e0.polarization_input_v, Some(0.02));
    let e1 =
        align_experiment_with_polarization(&exp, 3.0, &EnvironmentConfig::default(), Some(&e0), &p)
            .unwrap();
    assert_eq!(e1.polarization_input_v, None);
    let conservative = align_experiment_with_polarization(
        &exp,
        2.0,
        &EnvironmentConfig::default(),
        None,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(conservative.polarization_input_v, None);
    let _ = AlignmentMethod::Nearest;
}

#[test]
fn known_standard_pipeline_requires_units_and_nonideal_context() {
    use rust_electroanalysis_cli::{
        calibration_config::ActivityConfig, domain::EnvironmentalSeries, results::ActivityModelKind,
    };
    let mut ideal_experiment = experiment(vec![Some(0.0)], vec![0.0]);
    ideal_experiment.events.push(ExperimentEvent {
        timestamp: 0.0,
        kind: ExperimentEventKind::ConcentrationStep,
        value: Some(1.0),
        unit: Some("mmol/L".into()),
        analyte: None,
        annotation: Some("known standard".into()),
        metadata: None,
    });
    let mut env =
        align_experiment(&ideal_experiment, 0.0, &EnvironmentConfig::default(), None).unwrap();
    resolve_standard_activity(&mut env, &ActivityConfig::default(), None, 1).unwrap();
    assert_eq!(env.known_activity_log10, Some(-3.0));
    assert!(
        env.known_standard_assumption
            .as_ref()
            .unwrap()
            .contains("ideal")
    );

    let nonideal = ActivityConfig {
        model: ActivityModelKind::Davies,
        ..ActivityConfig::default()
    };
    assert!(resolve_standard_activity(&mut env.clone(), &nonideal, None, 1).is_err());
    ideal_experiment
        .environmental_data
        .push(EnvironmentalSeries {
            name: "ionic_strength".into(),
            unit: "mmol/L".into(),
            time: vec![0.0],
            values: vec![Some(100.0)],
            metadata: None,
        });
    let mut with_ionic =
        align_experiment(&ideal_experiment, 0.0, &EnvironmentConfig::default(), None).unwrap();
    resolve_standard_activity(&mut with_ionic, &nonideal, None, 1).unwrap();
    assert!(with_ionic.known_activity_log10.unwrap() < -3.0);

    let mut ambiguous = experiment(vec![Some(0.0)], vec![0.0]);
    ambiguous.events.push(ExperimentEvent {
        timestamp: 0.0,
        kind: ExperimentEventKind::ConcentrationStep,
        value: Some(1.0),
        unit: None,
        analyte: None,
        annotation: Some("known standard".into()),
        metadata: None,
    });
    assert!(align_experiment(&ambiguous, 0.0, &EnvironmentConfig::default(), None).is_err());
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
