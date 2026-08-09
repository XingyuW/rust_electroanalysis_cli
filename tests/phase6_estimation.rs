use rust_electroanalysis_cli::{
    calibration_config::ActivityConfig,
    domain::{
        AnalysisProvenance, ElectrochemicalExperiment, EnvironmentalSeries, ExperimentEvent,
        ExperimentEventKind, MeasurementChannel, MultiChannelMeasurement, SensorMetadata,
    },
    estimation::{
        self,
        calibration_adapter::{CalibrationObservationModel, StoredCalibrationObservationModel},
        environment::{
            AlignedEnvironment, AlignedValue, AlignmentMethod, EventFieldValue, align_experiment,
            align_experiment_with_polarization, resolve_standard_activity,
        },
        measurement::observations,
        simulation,
        state::{CalibrationDomainStatus, MeasurementUpdateStatus},
    },
    estimation_config::{
        CompiledEstimationProfile, EnvironmentConfig, EstimationModelBackend, FilterKind,
        MeasurementNoiseSourceKind, PolarizationInputModel, ResolvedEstimationConfig,
        StateModelKind, StateTransformKind, TransductionDriveSource, TruthAlignmentPolicy,
    },
};
use std::{fs, path::PathBuf, str::FromStr};

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

fn custom_binding_config() -> ResolvedEstimationConfig {
    let mut c = config(StateModelKind::Activity);
    c.model.backend = EstimationModelBackend::Compiled;
    c.model.profile = CompiledEstimationProfile::Custom;
    c.model.definition = Some(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/estimation/custom_flow_drive.toml"),
    );
    c.environment.temperature_series = None;
    c.environment.conductivity_series = None;
    c.environment.ionic_strength_series = None;
    c.environment.allow_configured_fallback = false;
    c
}

fn custom_binding_model(
    config: &ResolvedEstimationConfig,
) -> rust_electroanalysis_cli::estimation::model::StateModel {
    let calibration = simulation::simulation_model();
    rust_electroanalysis_cli::estimation::model::StateModel::new_compiled(
        config,
        10.0,
        None,
        &calibration,
    )
    .unwrap()
}

fn nicolsky_calibration_model() -> rust_electroanalysis_cli::results::StoredCalibrationModel {
    let mut model = simulation::simulation_model();
    model.analyte = "Ca2+".into();
    model.ion_charge = 2;
    model.model_kind = rust_electroanalysis_cli::results::CalibrationModelKind::NicolskyEisenman;
    model.temperature_mode =
        rust_electroanalysis_cli::results::TemperatureMode::ObservationSpecific;
    model.parameters[0].value = 0.18;
    model.selectivity_coefficients =
        vec![rust_electroanalysis_cli::results::SelectivityCoefficient {
            primary_analyte: "Ca2+".into(),
            interferent: "Cl-".into(),
            value: 0.35,
            source: "tracked Nicolsky parity fixture".into(),
            standard_error: None,
            confidence_interval: None,
        }];
    model.configuration.nicolsky_eisenman.interferents = vec![
        rust_electroanalysis_cli::calibration_config::InterferentConfig {
            name: "Cl-".into(),
            charge: -1,
            selectivity_coefficient: Some(0.35),
            source: "tracked Nicolsky parity fixture".into(),
        },
    ];
    model
}

#[test]
fn custom_flow_drive_binding_executes_in_normal_estimation_runtime() {
    let definition_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/estimation/custom_flow_drive.toml");
    let mut c = config(StateModelKind::Activity);
    c.model.backend = EstimationModelBackend::Compiled;
    c.model.profile = CompiledEstimationProfile::Custom;
    c.model.definition = Some(definition_path.clone());
    c.model
        .input_bindings
        .custom
        .insert("flow_drive".into(), "environment:flow".into());
    c.environment.flow_series = Some("flow".into());
    c.environment.temperature_series = None;
    c.environment.conductivity_series = None;
    c.environment.ionic_strength_series = None;
    c.environment.allow_configured_fallback = false;

    let measurement = MultiChannelMeasurement::new(
        vec![0.0, 1.0, 2.0],
        vec![MeasurementChannel::new(
            "E1",
            "V",
            vec![Some(0.0), Some(0.25), Some(0.5)],
        )],
    )
    .unwrap();
    let exp = ElectrochemicalExperiment::new(
        "custom-flow-drive",
        SensorMetadata::default(),
        None,
        measurement,
        vec![EnvironmentalSeries {
            name: "flow".into(),
            unit: "m/s".into(),
            time: vec![0.0, 1.0, 2.0],
            values: vec![Some(0.0), Some(1.0), Some(2.0)],
            metadata: None,
        }],
        Vec::new(),
        "buffer",
        provenance(),
    )
    .unwrap();
    let calibration =
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap();
    let model = rust_electroanalysis_cli::estimation::model::StateModel::new_compiled(
        &c,
        10.0,
        None,
        &calibration.model,
    )
    .unwrap();
    let state = nalgebra::DVector::from_element(model.dimension(), -3.0);
    let runtime_input = model
        .compiled_input(
            &state,
            &AlignedEnvironment {
                timestamp_s: 1.0,
                flow: Some(1.0),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(runtime_input.values.contains_key("flow_drive"));
    assert!(!runtime_input.values.contains_key("flow"));
    assert_eq!(runtime_input.values["flow_drive"].unit, "m/s");

    let report = estimation::estimate_experiment(
        &exp,
        "E1",
        calibration,
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let binding = report
        .resolved_input_bindings
        .as_ref()
        .and_then(|plan| plan.binding("flow_drive"))
        .expect("custom target binding should be retained in the report");
    assert_eq!(binding.target_input_id, "flow_drive");
    assert_eq!(binding.provenance.source_declaration, "environment:flow");
    assert!(matches!(
        binding.source,
        rust_electroanalysis_cli::estimation::ism_adapter::ModelInputSource::Environment(
            rust_electroanalysis_cli::estimation::ism_adapter::EnvironmentSource::Flow
        )
    ));
    match &report.resolved_model_definition_source {
        Some(
            rust_electroanalysis_cli::estimation::ism_adapter::ResolvedModelDefinitionSource::File {
                path,
                sha256,
            },
        ) => {
            assert_eq!(path, &definition_path);
            assert_eq!(sha256.len(), 64);
        }
        other => panic!("expected hashed custom definition source, got {other:?}"),
    }
    assert_eq!(report.model_id.as_deref(), Some("custom_flow_covariate"));
    let definition = report.model_definition.as_ref().unwrap();
    assert_eq!(
        definition.components[0].kind,
        "disturbance.linear_covariate"
    );
    assert!((report.estimates[1].predicted_measurement_v.unwrap() - 0.25).abs() < 1e-10);
    assert!((report.estimates[2].predicted_measurement_v.unwrap() - 0.5).abs() < 1e-10);
}

#[test]
fn compiled_bindings_cover_standard_custom_target_and_constant_sources() {
    let mut c = custom_binding_config();
    c.model
        .input_bindings
        .custom
        .insert("flow_drive".into(), "constant:100 cm/s".into());
    c.model
        .input_bindings
        .custom
        .insert("temperature_drive".into(), "constant:25 C".into());
    c.model
        .input_bindings
        .custom
        .insert("temperature".into(), "constant:20 C".into());
    let model = custom_binding_model(&c);
    let input = model
        .compiled_input(
            &nalgebra::DVector::from_element(model.dimension(), -3.0),
            &AlignedEnvironment {
                timestamp_s: 0.0,
                ..Default::default()
            },
        )
        .unwrap();
    assert!((input.values["flow_drive"].value - 1.0).abs() < 1e-12);
    assert_eq!(input.values["flow_drive"].unit, "m/s");
    assert!((input.values["temperature_drive"].value - 298.15).abs() < 1e-12);
    assert_eq!(input.values["temperature_drive"].unit, "K");
    assert!((input.values["temperature"].value - 293.15).abs() < 1e-12);
    assert!(matches!(
        model
            .resolved_input_bindings()
            .unwrap()
            .binding("temperature")
            .unwrap()
            .source,
        rust_electroanalysis_cli::estimation::ism_adapter::ModelInputSource::Constant { .. }
    ));
    assert!(matches!(
        model
            .resolved_input_bindings()
            .unwrap()
            .binding("temperature_drive")
            .unwrap()
            .source,
        rust_electroanalysis_cli::estimation::ism_adapter::ModelInputSource::Constant { .. }
    ));
}

#[test]
fn compiled_bindings_support_named_environment_and_event_field_sources() {
    let mut c = custom_binding_config();
    c.model
        .input_bindings
        .custom
        .insert("flow_drive".into(), "environment:flow".into());
    c.model
        .input_bindings
        .custom
        .insert("user_covariate".into(), "environment:humidity".into());
    c.model
        .input_bindings
        .custom
        .insert("event_drive".into(), "event:drive".into());
    let model = custom_binding_model(&c);
    let input = model
        .compiled_input(
            &nalgebra::DVector::from_element(model.dimension(), -3.0),
            &AlignedEnvironment {
                timestamp_s: 1.0,
                flow: Some(2.0),
                values: vec![AlignedValue {
                    value: 40.0,
                    source_series: "humidity".into(),
                    source_timestamps: vec![1.0],
                    alignment: AlignmentMethod::Nearest,
                    time_gap_s: 0.0,
                    interpolated: false,
                    extrapolated: false,
                    source_unit: Some("%RH".into()),
                    conversion: None,
                }],
                event_fields: std::collections::BTreeMap::from([(
                    "drive".into(),
                    EventFieldValue {
                        value: 0.5,
                        unit: "activity".into(),
                        event_timestamps_s: vec![1.0],
                    },
                )]),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(input.values["flow_drive"].value, 2.0);
    assert_eq!(input.values["user_covariate"].value, 40.0);
    assert_eq!(input.values["event_drive"].value, 0.5);
}

#[test]
fn compiled_bindings_reject_typed_target_source_and_unit_failures() {
    let mut unknown = custom_binding_config();
    unknown
        .model
        .input_bindings
        .custom
        .insert("not_a_model_input".into(), "environment:flow".into());
    let error = rust_electroanalysis_cli::estimation::model::StateModel::new_compiled(
        &unknown,
        10.0,
        None,
        &simulation::simulation_model(),
    )
    .err()
    .unwrap();
    assert!(matches!(
        error,
        rust_electroanalysis_cli::estimation::error::EstimationError::UnknownModelInputBindingTarget {
            target_input_id,
            ..
        } if target_input_id == "not_a_model_input"
    ));

    let mut unsupported = custom_binding_config();
    unsupported
        .model
        .input_bindings
        .custom
        .insert("flow_drive".into(), "voltage:flow".into());
    let error = rust_electroanalysis_cli::estimation::model::StateModel::new_compiled(
        &unsupported,
        10.0,
        None,
        &simulation::simulation_model(),
    )
    .err()
    .unwrap();
    assert!(matches!(
        error,
        rust_electroanalysis_cli::estimation::error::EstimationError::UnsupportedModelInputSource {
            target_input_id,
            ..
        } if target_input_id == "flow_drive"
    ));

    let mut mismatch = custom_binding_config();
    mismatch
        .model
        .input_bindings
        .custom
        .insert("flow_drive".into(), "environment:temperature".into());
    let error = rust_electroanalysis_cli::estimation::model::StateModel::new_compiled(
        &mismatch,
        10.0,
        None,
        &simulation::simulation_model(),
    )
    .err()
    .unwrap();
    assert!(matches!(
        error,
        rust_electroanalysis_cli::estimation::error::EstimationError::ModelInputUnitMismatch {
            target_input_id,
            expected_unit,
            actual_unit,
            ..
        } if target_input_id == "flow_drive" && expected_unit == "m/s" && actual_unit == "K"
    ));
}

#[test]
fn duplicate_custom_binding_keys_are_rejected_before_runtime_resolution() {
    let error = toml::from_str::<ResolvedEstimationConfig>(
        r#"
schema_version = 3

[model.input_bindings.custom]
flow_drive = "environment:flow"
flow_drive = "environment:temperature"
"#,
    )
    .unwrap_err();
    assert!(error.to_string().contains("duplicate"));
    assert!(error.to_string().contains("flow_drive"));
}

#[test]
fn compiled_bindings_report_missing_and_optional_sources_without_position_assumptions() {
    let mut missing = custom_binding_config();
    missing
        .model
        .input_bindings
        .custom
        .insert("flow_drive".into(), "environment:flow".into());
    let model = custom_binding_model(&missing);
    let error = model
        .compiled_input(
            &nalgebra::DVector::from_element(model.dimension(), -3.0),
            &AlignedEnvironment {
                timestamp_s: 0.0,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(
        error,
        rust_electroanalysis_cli::estimation::error::EstimationError::MissingModelInputSource {
            target_input_id,
            ..
        } if target_input_id == "flow_drive"
    ));

    let mut optional = custom_binding_config();
    optional
        .model
        .input_bindings
        .custom
        .insert("flow_drive".into(), "constant:0 m/s".into());
    let model = custom_binding_model(&optional);
    let input = model
        .compiled_input(
            &nalgebra::DVector::from_element(model.dimension(), -3.0),
            &AlignedEnvironment {
                timestamp_s: 0.0,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!input.values.contains_key("temperature_drive"));
    assert!(!input.values.contains_key("user_covariate"));
    assert!(!input.values.contains_key("event_drive"));
    assert_eq!(input.values["flow_drive"].value, 0.0);
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
    let (rows, _diag) = observations(&measurement, "E").unwrap();
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
    assert!(report.estimates[1].predicted_measurement_v.is_some());
    assert!(report.estimates[1].unexplained_residual_v.is_none());
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

#[test]
fn per_observation_variance_is_applied_and_recorded() {
    let measurement = MultiChannelMeasurement::new(
        vec![0.0, 1.0, 2.0],
        vec![
            MeasurementChannel::new("E1", "mV", vec![Some(22.52), Some(22.52), Some(22.52)])
                .with_variance(vec![Some(1.0), Some(4.0), Some(9.0)])
                .with_source_header("E1/mV"),
        ],
    )
    .unwrap();
    let exp = ElectrochemicalExperiment::new(
        "phase6-variance",
        SensorMetadata::default(),
        None,
        measurement,
        Vec::new(),
        Vec::new(),
        "buffer",
        provenance(),
    )
    .unwrap();
    let mut c = config(StateModelKind::Activity);
    c.measurement_noise.source = MeasurementNoiseSourceKind::PerObservation;
    let report = estimation::estimate_experiment(
        &exp,
        "E1",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let source_header_report = estimation::estimate_experiment(
        &exp,
        "E1/mV",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    assert_eq!(report, source_header_report);
    assert_eq!(report.channel, "E1");
    let records = &report.diagnostics.innovations;
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].measurement_variance_source, "per_observation");
    assert!((records[0].measurement_variance_v2 - 1.0e-6).abs() < 1.0e-15);
    assert!((records[2].measurement_variance_v2 - 9.0e-6).abs() < 1.0e-15);
    assert_eq!(
        records[2].uninflated_measurement_variance_v2,
        records[2].measurement_variance_v2
    );

    let ukf_by_logical = estimation::estimate_experiment(
        &exp,
        "E1",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ukf,
    )
    .unwrap();
    let ukf_by_source_header = estimation::estimate_experiment(
        &exp,
        "E1/mV",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ukf,
    )
    .unwrap();
    assert_eq!(ukf_by_logical, ukf_by_source_header);
    let comparison_by_logical = estimation::comparison::compare_reports(
        &[
            (FilterKind::Ekf, report.clone()),
            (FilterKind::Ukf, ukf_by_logical),
        ],
        None,
    );
    let comparison_by_source_header = estimation::comparison::compare_reports(
        &[
            (FilterKind::Ekf, source_header_report),
            (FilterKind::Ukf, ukf_by_source_header),
        ],
        None,
    );
    assert_eq!(comparison_by_logical, comparison_by_source_header);
}

#[test]
fn calibration_domain_inflation_is_deterministic_at_boundary_and_outside() {
    let model = simulation::simulation_model();
    let mut c = config(StateModelKind::Activity);
    c.initialization.activity_source = "configured".into();
    c.extrapolation.inflate_measurement_variance = true;
    c.extrapolation.near_boundary_variance_inflation_factor = 1.25;
    c.extrapolation.variance_inflation_factor = 4.0;
    let boundary = 1.0e-8;
    c.initialization.initial_activity = boundary;
    let boundary_report = estimation::estimate_experiment(
        &experiment(vec![Some(0.2 + 0.05916 * -8.0); 3], vec![0.0, 1.0, 2.0]),
        "E1/V",
        StoredCalibrationObservationModel::new(model.clone()).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    assert_eq!(
        boundary_report.diagnostics.innovations[0]
            .variance_inflation_reason
            .as_deref(),
        Some("near calibration-domain boundary")
    );
    assert!(
        (boundary_report.diagnostics.innovations[0].variance_inflation_factor - 1.25).abs() < 1e-12
    );

    c.initialization.initial_activity = 1.0e-9;
    let outside_report = estimation::estimate_experiment(
        &experiment(vec![Some(0.2 + 0.05916 * -9.0); 3], vec![0.0, 1.0, 2.0]),
        "E1/V",
        StoredCalibrationObservationModel::new(model).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    assert_eq!(
        outside_report.diagnostics.innovations[0]
            .variance_inflation_reason
            .as_deref(),
        Some("outside calibration domain")
    );
    assert!(
        (outside_report.diagnostics.innovations[0].variance_inflation_factor - 4.0).abs() < 1e-12
    );
}

#[test]
fn logistic_sensitivity_transform_exports_latent_and_physical_values() {
    let mut c = config(StateModelKind::Activity);
    c.state_model.include_condition_state = true;
    c.state_model.activity_transform = StateTransformKind::LogisticBounded;
    c.state_model.condition_lower = 0.5;
    c.state_model.condition_upper = 1.5;
    let mut exp = experiment(vec![Some(0.02252); 6], (0..6).map(|x| x as f64).collect());
    exp.events.push(ExperimentEvent {
        timestamp: 0.0,
        kind: ExperimentEventKind::ConcentrationStep,
        value: Some(1e-3),
        unit: Some("mol/L".into()),
        analyte: None,
        annotation: Some("known standard".into()),
        metadata: None,
    });
    let report = estimation::estimate_experiment(
        &exp,
        "E1/V",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let sensitivity = report
        .estimates
        .last()
        .unwrap()
        .filtered_state
        .iter()
        .find(|x| x.name == "sensitivity_scale")
        .unwrap();
    assert!(sensitivity.value.unwrap() > 0.5 && sensitivity.value.unwrap() < 1.5);
    assert!(sensitivity.latent_value.unwrap().is_finite());
    assert!(sensitivity.latent);
}

#[test]
fn validation_uses_configured_linear_alignment_and_state_thresholds() {
    let model = simulation::simulation_model();
    let c = config(StateModelKind::Activity);
    let report = estimation::estimate_experiment(
        &experiment(vec![Some(0.02252); 3], vec![0.0, 1.0, 2.0]),
        "E1/V",
        StoredCalibrationObservationModel::new(model).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let mut validation_config = report.configuration.clone();
    validation_config.validation.alignment_policy = TruthAlignmentPolicy::LinearInterpolation;
    validation_config.validation.maximum_alignment_gap_s = 0.75;
    validation_config
        .validation
        .states
        .get_mut("log10_activity")
        .unwrap()
        .minimum_consecutive_converged_points = 2;
    let report = rust_electroanalysis_cli::results::StateEstimationReport {
        configuration: validation_config,
        ..report
    };
    let truth = vec![
        rust_electroanalysis_cli::estimation::validation::TruthPoint {
            timestamp_s: 0.0,
            log10_activity: Some(-3.0),
            activity: Some(1e-3),
            baseline_offset_v: None,
            polarization_v: None,
            sensitivity_scale: None,
            state_values: std::collections::BTreeMap::new(),
            component_potentials_v: std::collections::BTreeMap::new(),
            outlier: false,
        },
        rust_electroanalysis_cli::estimation::validation::TruthPoint {
            timestamp_s: 0.5,
            log10_activity: Some(-3.0),
            activity: Some(1e-3),
            baseline_offset_v: None,
            polarization_v: None,
            sensitivity_scale: None,
            state_values: std::collections::BTreeMap::new(),
            component_potentials_v: std::collections::BTreeMap::new(),
            outlier: false,
        },
        rust_electroanalysis_cli::estimation::validation::TruthPoint {
            timestamp_s: 1.5,
            log10_activity: Some(-3.0),
            activity: Some(1e-3),
            baseline_offset_v: None,
            polarization_v: None,
            sensitivity_scale: None,
            state_values: std::collections::BTreeMap::new(),
            component_potentials_v: std::collections::BTreeMap::new(),
            outlier: false,
        },
        rust_electroanalysis_cli::estimation::validation::TruthPoint {
            timestamp_s: 2.5,
            log10_activity: Some(-3.0),
            activity: Some(1e-3),
            baseline_offset_v: None,
            polarization_v: None,
            sensitivity_scale: None,
            state_values: std::collections::BTreeMap::new(),
            component_potentials_v: std::collections::BTreeMap::new(),
            outlier: false,
        },
    ];
    let result =
        rust_electroanalysis_cli::estimation::validation::validate_report(&report, &truth, None);
    assert_eq!(
        result.alignment_policy.as_deref(),
        Some("LinearInterpolation")
    );
    assert_eq!(result.matched_sample_count, 2);
    assert_eq!(result.alignment_methods.len(), 2);
    assert!(
        result
            .alignment_methods
            .iter()
            .all(|method| method == "linear_interpolation")
    );
}

#[test]
fn deterministic_monte_carlo_fixture_is_reproducible() {
    let scenario = simulation::SimulationScenario {
        sample_count: 24,
        seed: 1234,
        irregular_jitter_s: 0.2,
        measurement_noise_sd_v: 1.0e-4,
        ..Default::default()
    };
    let first = simulation::simulate_scenario(&scenario).unwrap();
    let second = simulation::simulate_scenario(&scenario).unwrap();
    assert_eq!(first, second);
    assert!(
        first
            .observations
            .iter()
            .all(|point| point.timestamp_s.is_finite())
    );
}

#[test]
fn seeded_monte_carlo_nis_diagnostic_is_finite_with_broad_tolerance() {
    let mut nis_means = Vec::new();
    for seed in 1..=8 {
        let scenario = simulation::SimulationScenario {
            sample_count: 32,
            seed,
            activity_step_time_s: None,
            measurement_noise_sd_v: 1.0e-4,
            ..Default::default()
        };
        let simulated = simulation::simulate_scenario(&scenario).unwrap();
        let values = simulated
            .observations
            .iter()
            .map(|point| point.observed_potential_v)
            .collect::<Vec<_>>();
        let times = simulated
            .observations
            .iter()
            .map(|point| point.timestamp_s)
            .collect::<Vec<_>>();
        let report = estimation::estimate_experiment(
            &experiment(values, times),
            "E1/V",
            StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
            &config(StateModelKind::Activity),
            estimation::EstimationContext::default(),
            FilterKind::Ekf,
        )
        .unwrap();
        let nis = report.diagnostics.nis_mean.unwrap();
        assert!(nis.is_finite());
        nis_means.push(nis);
    }
    let ensemble_mean = nis_means.iter().sum::<f64>() / nis_means.len() as f64;
    assert!(ensemble_mean.is_finite() && ensemble_mean < 100.0);
}

#[test]
fn ekf_ukf_comparison_reports_equivalent_input_metrics() {
    let model = simulation::simulation_model();
    let exp = experiment(
        vec![Some(0.02252), Some(0.0226), Some(0.02245), Some(0.02255)],
        vec![0.0, 0.7, 1.9, 3.0],
    );
    let c = config(StateModelKind::Activity);
    let ekf = estimation::estimate_experiment(
        &exp,
        "E1/V",
        StoredCalibrationObservationModel::new(model.clone()).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let ukf = estimation::estimate_experiment(
        &exp,
        "E1/V",
        StoredCalibrationObservationModel::new(model).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ukf,
    )
    .unwrap();
    for report in [&ekf, &ukf] {
        for point in &report.estimates {
            let sum = point
                .component_contributions
                .iter()
                .filter_map(|component| component.potential_v)
                .sum::<f64>();
            assert!((sum - point.predicted_measurement_v.unwrap()).abs() < 1e-9);
            assert!(
                point
                    .component_contributions
                    .iter()
                    .any(|component| component.component_id == "legacy.equilibrium")
            );
            assert!(!matches!(
                point
                    .equilibrium_assessment
                    .as_ref()
                    .map(|item| item.status),
                Some(rust_electroanalysis_cli::model::AssessmentStatus::Supported)
            ));
        }
    }
    let comparison = rust_electroanalysis_cli::estimation::comparison::compare_reports(
        &[(FilterKind::Ekf, ekf), (FilterKind::Ukf, ukf)],
        None,
    );
    assert_eq!(comparison.records.len(), 2);
    assert!(
        comparison
            .records
            .iter()
            .all(|record| record.log_likelihood.is_some())
    );
    assert!(
        comparison
            .records
            .iter()
            .all(|record| record.nis_consistency.is_some())
    );
}

#[test]
fn compiled_legacy_adapter_reproduces_legacy_process_and_observation_equations() {
    let stored = simulation::simulation_model();
    let calibration = StoredCalibrationObservationModel::new(stored.clone()).unwrap();
    let mut config = config(StateModelKind::ActivityBaselinePolarization);
    let legacy =
        rust_electroanalysis_cli::estimation::model::StateModel::new(&config, 7.0, None).unwrap();
    config.model.backend =
        rust_electroanalysis_cli::estimation_config::EstimationModelBackend::Compiled;
    config.model.profile =
        rust_electroanalysis_cli::estimation_config::CompiledEstimationProfile::LegacyEquivalentV1;
    let compiled = rust_electroanalysis_cli::estimation::model::StateModel::new_compiled(
        &config, 7.0, None, &stored,
    )
    .unwrap();
    let compiled_definition = compiled.compiled_model().unwrap().definition();
    assert!(compiled_definition.states.iter().all(|state| {
        matches!(
            state.initialization_source,
            rust_electroanalysis_cli::model::StateInitializationSource::Estimated
        ) && state
            .initial_uncertainty
            .variance_in(&state.unit)
            .is_ok_and(|variance| variance.is_some_and(|value| value > 0.0))
    }));
    assert!(!compiled_definition.states.iter().any(|state| {
        matches!(
            state.initial_uncertainty,
            rust_electroanalysis_cli::model::UncertaintySpec::Unknown { .. }
        )
    }));
    let state = nalgebra::DVector::from_vec(vec![-3.0, 0.01, 0.02]);
    let environment = AlignedEnvironment {
        timestamp_s: 1.0,
        temperature_k: Some(298.15),
        polarization_input_v: Some(0.03),
        ..Default::default()
    };
    let legacy_observation = rust_electroanalysis_cli::estimation::model::observation_components(
        &state,
        &environment,
        &legacy,
        &calibration,
    )
    .unwrap();
    let compiled_observation = rust_electroanalysis_cli::estimation::model::observation_components(
        &state,
        &environment,
        &compiled,
        &calibration,
    )
    .unwrap();
    assert!((legacy_observation.0 - compiled_observation.0).abs() < 1e-12);
    for (legacy, compiled) in legacy_observation
        .1
        .iter()
        .zip(compiled_observation.1.iter())
    {
        assert!((legacy - compiled).abs() < 1e-12);
    }
    let legacy_next = legacy.process_state(&state, 0.5, &environment);
    let compiled_next = compiled
        .try_process_state(&state, 0.5, &environment)
        .unwrap();
    assert!((&legacy_next - &compiled_next).norm() < 1e-12);
}

#[test]
fn compiled_legacy_profile_parity_covers_supported_legacy_state_models() {
    let stored = simulation::simulation_model();
    let calibration = StoredCalibrationObservationModel::new(stored.clone()).unwrap();
    for kind in [
        StateModelKind::Activity,
        StateModelKind::ActivityBaseline,
        StateModelKind::ActivityBaselinePolarization,
    ] {
        let mut config = config(kind);
        let legacy =
            rust_electroanalysis_cli::estimation::model::StateModel::new(&config, 7.0, None)
                .unwrap();
        config.model.backend =
            rust_electroanalysis_cli::estimation_config::EstimationModelBackend::Compiled;
        config.model.profile =
            rust_electroanalysis_cli::estimation_config::CompiledEstimationProfile::LegacyEquivalentV1;
        let compiled = rust_electroanalysis_cli::estimation::model::StateModel::new_compiled(
            &config, 7.0, None, &stored,
        )
        .unwrap();
        let mut state = nalgebra::DVector::zeros(legacy.dimension());
        state[0] = -3.0;
        for index in 1..state.len() {
            state[index] = 0.01 * index as f64;
        }
        let environment = AlignedEnvironment {
            timestamp_s: 1.0,
            temperature_k: Some(298.15),
            polarization_input_v: Some(0.03),
            ..Default::default()
        };
        let legacy_observation =
            rust_electroanalysis_cli::estimation::model::observation_components(
                &state,
                &environment,
                &legacy,
                &calibration,
            )
            .unwrap();
        let compiled_observation =
            rust_electroanalysis_cli::estimation::model::observation_components(
                &state,
                &environment,
                &compiled,
                &calibration,
            )
            .unwrap();
        assert!(
            (legacy_observation.0 - compiled_observation.0).abs() < 1e-12,
            "{kind:?}"
        );
        assert!(
            (&legacy_observation.1 - &compiled_observation.1).norm() < 1e-12,
            "{kind:?}"
        );
        let legacy_next = legacy.process_state(&state, 0.5, &environment);
        let compiled_next = compiled
            .try_process_state(&state, 0.5, &environment)
            .unwrap();
        assert!((&legacy_next - compiled_next).norm() < 1e-12, "{kind:?}");
    }
}

#[test]
fn compiled_legacy_equivalent_permanent_parity_matrix_covers_states_filters_and_scenarios() {
    let cases = [
        (
            "regular timestamps",
            vec![0.0, 1.0, 2.0, 3.0],
            vec![Some(0.02); 4],
        ),
        (
            "irregular timestamps",
            vec![0.0, 0.7, 2.1, 4.0],
            vec![Some(0.02); 4],
        ),
        (
            "missing observations",
            vec![0.0, 1.0, 2.0, 3.0],
            vec![Some(0.02), None, Some(0.02), Some(0.02)],
        ),
        (
            "predict-only interval",
            vec![0.0, 1.0, 2.0],
            vec![Some(0.02), None, None],
        ),
        (
            "innovation gating",
            vec![0.0, 1.0, 2.0, 3.0],
            vec![Some(0.02), Some(0.02), Some(0.3), Some(0.02)],
        ),
        (
            "temperature variation",
            vec![0.0, 1.0, 2.0, 3.0],
            vec![Some(0.02); 4],
        ),
        ("known standard", vec![0.0, 1.0, 2.0], vec![Some(0.02); 3]),
        (
            "calibration extrapolation",
            vec![0.0, 1.0, 2.0],
            vec![Some(0.2 - 0.05916 * 9.0); 3],
        ),
        (
            "baseline drift",
            vec![0.0, 1.0, 2.0, 3.0],
            vec![Some(0.02), Some(0.021), Some(0.022), Some(0.023)],
        ),
        (
            "polarization dynamics",
            vec![0.0, 0.5, 1.7, 3.0],
            vec![Some(0.02); 4],
        ),
    ];
    for (scenario, times, values) in cases {
        for kind in [
            StateModelKind::Activity,
            StateModelKind::ActivityBaseline,
            StateModelKind::ActivityBaselinePolarization,
        ] {
            for filter in [FilterKind::Ekf, FilterKind::Ukf] {
                let mut base = config(kind);
                base.observability.reject_unobservable_model = false;
                if scenario == "innovation gating" {
                    base.filter.innovation_gate_probability = 0.5;
                }
                if scenario == "polarization dynamics" {
                    base.polarization.input_model = PolarizationInputModel::ActivityStepGain;
                    base.polarization.gain_v_per_log10_activity = 0.01;
                }
                let mut exp = experiment(values.clone(), times.clone());
                if scenario == "temperature variation" {
                    exp.environmental_data.push(EnvironmentalSeries {
                        name: "temperature".into(),
                        unit: "K".into(),
                        time: times.clone(),
                        values: times.iter().map(|t| Some(298.15 + t)).collect(),
                        metadata: None,
                    });
                }
                if scenario == "known standard" {
                    exp.events.push(ExperimentEvent {
                        timestamp: 0.0,
                        kind: ExperimentEventKind::ConcentrationStep,
                        value: Some(1e-3),
                        unit: Some("mol/L".into()),
                        analyte: None,
                        annotation: Some("known standard".into()),
                        metadata: None,
                    });
                }
                if scenario == "polarization dynamics" {
                    exp.events.push(ExperimentEvent {
                        timestamp: 1.0,
                        kind: ExperimentEventKind::ConcentrationStep,
                        value: Some(1e-2),
                        unit: Some("mol/L".into()),
                        analyte: None,
                        annotation: None,
                        metadata: None,
                    });
                }
                let stored = simulation::simulation_model();
                let legacy = estimation::estimate_experiment(
                    &exp,
                    "E1",
                    StoredCalibrationObservationModel::new(stored.clone()).unwrap(),
                    &base,
                    estimation::EstimationContext::default(),
                    filter,
                )
                .unwrap();
                let mut compiled_config = base.clone();
                compiled_config.model.backend = EstimationModelBackend::Compiled;
                compiled_config.model.profile = CompiledEstimationProfile::LegacyEquivalentV1;
                let compiled = estimation::estimate_experiment(
                    &exp,
                    "E1",
                    StoredCalibrationObservationModel::new(stored).unwrap(),
                    &compiled_config,
                    estimation::EstimationContext::default(),
                    filter,
                )
                .unwrap();
                assert_eq!(
                    legacy.estimates.len(),
                    compiled.estimates.len(),
                    "{scenario} {kind:?} {filter:?}"
                );
                assert_eq!(
                    legacy.diagnostics.innovations.len(),
                    compiled.diagnostics.innovations.len(),
                    "{scenario} {kind:?} {filter:?}"
                );
                for (left, right) in legacy.estimates.iter().zip(&compiled.estimates) {
                    assert_eq!(
                        left.timestamp_s, right.timestamp_s,
                        "{scenario} {kind:?} {filter:?}"
                    );
                    assert_eq!(
                        left.update_status, right.update_status,
                        "{scenario} {kind:?} {filter:?}"
                    );
                    assert_eq!(
                        left.calibration_domain_status, right.calibration_domain_status,
                        "{scenario} {kind:?} {filter:?}"
                    );
                    for (a, b) in left
                        .predicted_state
                        .iter()
                        .zip(&right.predicted_state)
                        .chain(left.filtered_state.iter().zip(&right.filtered_state))
                    {
                        assert_eq!(a.name, b.name);
                        assert!(
                            (a.value.unwrap_or_default() - b.value.unwrap_or_default()).abs()
                                < 1e-10,
                            "{scenario} {kind:?} {filter:?}"
                        );
                    }
                    for (a, b) in left
                        .predicted_covariance
                        .iter()
                        .flatten()
                        .zip(right.predicted_covariance.iter().flatten())
                        .chain(
                            left.filtered_covariance
                                .iter()
                                .flatten()
                                .zip(right.filtered_covariance.iter().flatten()),
                        )
                    {
                        assert!((a - b).abs() < 1e-10, "{scenario} {kind:?} {filter:?}");
                    }
                    for (a, b) in [
                        (left.predicted_measurement_v, right.predicted_measurement_v),
                        (left.innovation_v, right.innovation_v),
                        (left.innovation_variance_v2, right.innovation_variance_v2),
                        (left.standardized_innovation, right.standardized_innovation),
                        (
                            left.normalized_innovation_squared,
                            right.normalized_innovation_squared,
                        ),
                    ] {
                        if let (Some(a), Some(b)) = (a, b) {
                            assert!((a - b).abs() < 1e-10, "{scenario} {kind:?} {filter:?}");
                        } else {
                            assert_eq!(a.is_some(), b.is_some());
                        }
                    }
                }
                for (left, right) in legacy
                    .diagnostics
                    .innovations
                    .iter()
                    .zip(&compiled.diagnostics.innovations)
                {
                    assert_eq!(left.timestamp_s, right.timestamp_s);
                    assert_eq!(left.accepted, right.accepted);
                    assert_eq!(left.gating_threshold, right.gating_threshold);
                    assert_eq!(left.kalman_gain.len(), right.kalman_gain.len());
                    for (a, b) in left.kalman_gain.iter().zip(&right.kalman_gain) {
                        assert!((a - b).abs() < 1e-10, "{scenario} {kind:?} {filter:?}");
                    }
                }
                assert_eq!(
                    legacy.diagnostics.rejected_update_count,
                    compiled.diagnostics.rejected_update_count,
                    "{scenario} {kind:?} {filter:?}"
                );
                assert_eq!(
                    legacy.diagnostics.predict_only_count, compiled.diagnostics.predict_only_count,
                    "{scenario} {kind:?} {filter:?}"
                );
                // Warning-count/text differences are an allowlisted metadata
                // difference for the adapter boundary; numerical state,
                // gating, and update-status differences are not allowlisted.
            }
        }
    }
}

#[test]
fn compiled_legacy_equivalent_nicolsky_interferent_parity_runs_ekf_and_ukf() {
    let stored = nicolsky_calibration_model();
    let interferent_activities = [0.02, 0.025, 0.03, 0.035, 0.04];
    let temperatures_k = [298.15, 298.35, 298.55, 298.75, 298.95];
    let target_log10_activities = [-3.0, -2.8, -2.6, -2.4, -2.2];
    let times = vec![0.0, 0.7, 1.9, 3.2, 4.4];
    let values = target_log10_activities
        .iter()
        .zip(interferent_activities)
        .zip(temperatures_k)
        .map(|((&log10_activity, interferent_activity), temperature_k)| {
            rust_electroanalysis_cli::potentiometry::calibration::nicolsky_eisenman::evaluate_potential(
                stored.parameters[0].value,
                10_f64.powf(log10_activity),
                stored.ion_charge,
                temperature_k,
                &[rust_electroanalysis_cli::potentiometry::calibration::nicolsky_eisenman::InterferentModelInput {
                    name: "Cl-".into(),
                    charge: -1,
                    activity: interferent_activity,
                    selectivity_coefficient: 0.35,
                }],
            )
            .unwrap()
        })
        .map(Some)
        .collect::<Vec<_>>();
    let experiment = ElectrochemicalExperiment::new(
        "nicolsky-interferent-parity",
        SensorMetadata::default(),
        None,
        MultiChannelMeasurement::new(
            times.clone(),
            vec![MeasurementChannel::new("E1", "V", values)],
        )
        .unwrap(),
        vec![
            EnvironmentalSeries {
                name: "temperature".into(),
                unit: "K".into(),
                time: times.clone(),
                values: temperatures_k.into_iter().map(Some).collect(),
                metadata: None,
            },
            EnvironmentalSeries {
                name: "chloride_activity".into(),
                unit: "activity".into(),
                time: times.clone(),
                values: interferent_activities.into_iter().map(Some).collect(),
                metadata: None,
            },
        ],
        Vec::new(),
        "buffer",
        provenance(),
    )
    .unwrap();

    for filter in [FilterKind::Ekf, FilterKind::Ukf] {
        let mut legacy_config = config(StateModelKind::Activity);
        legacy_config.environment.temperature_series = Some("temperature".into());
        legacy_config
            .environment
            .interferent_series
            .insert("Cl-".into(), "chloride_activity".into());
        legacy_config.observability.reject_unobservable_model = false;
        let legacy = estimation::estimate_experiment(
            &experiment,
            "E1",
            StoredCalibrationObservationModel::new(stored.clone()).unwrap(),
            &legacy_config,
            estimation::EstimationContext::default(),
            filter,
        )
        .unwrap();

        let mut compiled_config = legacy_config.clone();
        compiled_config.model.backend = EstimationModelBackend::Compiled;
        compiled_config.model.profile = CompiledEstimationProfile::LegacyEquivalentV1;
        let compiled = estimation::estimate_experiment(
            &experiment,
            "E1",
            StoredCalibrationObservationModel::new(stored.clone()).unwrap(),
            &compiled_config,
            estimation::EstimationContext::default(),
            filter,
        )
        .unwrap();

        assert_eq!(legacy.estimates.len(), compiled.estimates.len());
        assert_eq!(
            legacy.diagnostics.innovations.len(),
            compiled.diagnostics.innovations.len()
        );
        for (left, right) in legacy.estimates.iter().zip(&compiled.estimates) {
            assert_eq!(left.timestamp_s, right.timestamp_s);
            assert_eq!(left.update_status, right.update_status);
            assert_eq!(
                left.calibration_domain_status,
                right.calibration_domain_status
            );
            for (a, b) in left
                .predicted_state
                .iter()
                .zip(&right.predicted_state)
                .chain(left.filtered_state.iter().zip(&right.filtered_state))
            {
                assert_eq!(a.name, b.name);
                assert!((a.value.unwrap() - b.value.unwrap()).abs() < 1e-10);
            }
            for (a, b) in left
                .predicted_covariance
                .iter()
                .flatten()
                .zip(right.predicted_covariance.iter().flatten())
                .chain(
                    left.filtered_covariance
                        .iter()
                        .flatten()
                        .zip(right.filtered_covariance.iter().flatten()),
                )
            {
                assert!((a - b).abs() < 1e-10);
            }
            for (a, b) in [
                (left.predicted_measurement_v, right.predicted_measurement_v),
                (left.innovation_v, right.innovation_v),
                (left.innovation_variance_v2, right.innovation_variance_v2),
                (
                    left.normalized_innovation_squared,
                    right.normalized_innovation_squared,
                ),
            ] {
                match (a, b) {
                    (Some(a), Some(b)) => assert!((a - b).abs() < 1e-10),
                    (None, None) => {}
                    _ => panic!("Nicolsky parity changed optional diagnostic presence"),
                }
            }
        }
        for (left, right) in legacy
            .diagnostics
            .innovations
            .iter()
            .zip(&compiled.diagnostics.innovations)
        {
            assert_eq!(left.timestamp_s, right.timestamp_s);
            assert_eq!(left.accepted, right.accepted);
            assert_eq!(left.gating_threshold, right.gating_threshold);
            assert!((left.innovation_v - right.innovation_v).abs() < 1e-10);
            assert!((left.innovation_variance_v2 - right.innovation_variance_v2).abs() < 1e-10);
            assert!(
                (left.normalized_innovation_squared - right.normalized_innovation_squared).abs()
                    < 1e-10
            );
        }
        assert_eq!(
            legacy.diagnostics.rejected_update_count,
            compiled.diagnostics.rejected_update_count
        );
        assert_eq!(
            legacy.diagnostics.predict_only_count,
            compiled.diagnostics.predict_only_count
        );
        assert!(legacy.estimates.iter().all(|estimate| {
            estimate
                .environmental_context
                .interferent_activities
                .contains_key("Cl-")
        }));
    }
}

#[test]
fn compiled_legacy_parity_covers_condition_sensitivity_state_for_ekf_and_ukf() {
    for filter in [FilterKind::Ekf, FilterKind::Ukf] {
        let mut legacy_config = config(StateModelKind::ActivityBaselinePolarization);
        legacy_config.state_model.include_condition_state = true;
        legacy_config.auxiliary.condition_requires_auxiliary = false;
        legacy_config.observability.reject_unobservable_model = false;
        legacy_config.filter.kind = filter;
        let mut compiled_config = legacy_config.clone();
        compiled_config.model.backend = EstimationModelBackend::Compiled;
        compiled_config.model.profile = CompiledEstimationProfile::LegacyEquivalentV1;
        let legacy = estimation::estimate_experiment(
            &experiment(
                vec![Some(0.02), Some(0.0201), Some(0.0202), Some(0.0203)],
                vec![0.0, 0.7, 2.1, 4.0],
            ),
            "E1",
            StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
            &legacy_config,
            estimation::EstimationContext::default(),
            filter,
        )
        .unwrap();
        let compiled = estimation::estimate_experiment(
            &experiment(
                vec![Some(0.02), Some(0.0201), Some(0.0202), Some(0.0203)],
                vec![0.0, 0.7, 2.1, 4.0],
            ),
            "E1",
            StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
            &compiled_config,
            estimation::EstimationContext::default(),
            filter,
        )
        .unwrap();
        assert!(
            legacy
                .state_definitions
                .iter()
                .any(|state| state.name == "sensitivity_scale")
        );
        for (left, right) in legacy.estimates.iter().zip(&compiled.estimates) {
            for state in [
                "log10_activity",
                "baseline_offset",
                "polarization",
                "sensitivity_scale",
            ] {
                let left_value = left
                    .filtered_state
                    .iter()
                    .find(|value| value.name == state)
                    .and_then(|value| value.value);
                let right_value = right
                    .filtered_state
                    .iter()
                    .find(|value| value.name == state)
                    .and_then(|value| value.value);
                assert_eq!(
                    left_value.is_some(),
                    right_value.is_some(),
                    "{state} {filter:?}"
                );
                if let (Some(left_value), Some(right_value)) = (left_value, right_value) {
                    // The condition/sensitivity UKF path uses the compiled
                    // parameterized observation Jacobian; its documented
                    // compatibility tolerance is 1e-8 (the ordinary parity
                    // matrix remains at 1e-10).
                    assert!(
                        (left_value - right_value).abs() < 1e-8,
                        "{state} {filter:?}: {left_value} vs {right_value}"
                    );
                }
            }
        }
    }
}

#[test]
fn compiled_activity_events_are_applied_once_at_irregular_transitions_with_provenance() {
    let events = vec![
        ExperimentEvent {
            timestamp: 0.0,
            kind: ExperimentEventKind::ConcentrationStep,
            value: Some(1e-3),
            unit: Some("mol/L".into()),
            analyte: None,
            annotation: None,
            metadata: None,
        },
        ExperimentEvent {
            timestamp: 0.4,
            kind: ExperimentEventKind::ConcentrationStep,
            value: Some(1e-2),
            unit: Some("mol/L".into()),
            analyte: None,
            annotation: None,
            metadata: None,
        },
        ExperimentEvent {
            timestamp: 0.8,
            kind: ExperimentEventKind::ConcentrationStep,
            value: Some(1e-1),
            unit: Some("mol/L".into()),
            analyte: None,
            annotation: None,
            metadata: None,
        },
        ExperimentEvent {
            timestamp: 1.4,
            kind: ExperimentEventKind::ConcentrationStep,
            value: Some(1.0),
            unit: Some("mol/L".into()),
            analyte: None,
            annotation: None,
            metadata: None,
        },
    ];
    let exp = ElectrochemicalExperiment::new(
        "activity-events",
        SensorMetadata::default(),
        None,
        MultiChannelMeasurement::new(
            vec![0.0, 1.0, 2.0],
            vec![MeasurementChannel::new("E1", "V", vec![Some(0.0); 3])],
        )
        .unwrap(),
        Vec::new(),
        events,
        "buffer",
        provenance(),
    )
    .unwrap();
    let mut environments = vec![
        AlignedEnvironment {
            timestamp_s: 0.0,
            temperature_k: Some(298.15),
            ..Default::default()
        },
        AlignedEnvironment {
            timestamp_s: 1.0,
            temperature_k: Some(298.15),
            ..Default::default()
        },
        AlignedEnvironment {
            timestamp_s: 2.0,
            temperature_k: Some(298.15),
            ..Default::default()
        },
    ];
    rust_electroanalysis_cli::estimation::environment::bind_compiled_transition_inputs(
        &exp,
        &mut environments,
        &ActivityConfig::default(),
        None,
        1,
        &TransductionDriveSource::None,
    );
    assert!(environments[0].delta_log10_activity.is_none());
    assert_eq!(
        environments[0].activity_step_event_timestamps_s,
        Vec::<f64>::new()
    );
    assert_eq!(environments[1].delta_log10_activity, Some(2.0));
    assert_eq!(
        environments[1].activity_step_event_timestamps_s,
        vec![0.4, 0.8]
    );
    assert_eq!(environments[2].delta_log10_activity, Some(1.0));
    assert_eq!(environments[2].activity_step_event_timestamps_s, vec![1.4]);
    assert_eq!(
        environments[1].activity_step_event_timestamps_s.len()
            + environments[2].activity_step_event_timestamps_s.len(),
        3
    );

    let mut no_event_environment = vec![
        AlignedEnvironment {
            timestamp_s: 0.0,
            temperature_k: Some(298.15),
            ..Default::default()
        },
        AlignedEnvironment {
            timestamp_s: 0.7,
            temperature_k: Some(298.15),
            ..Default::default()
        },
    ];
    let no_event_experiment = ElectrochemicalExperiment::new(
        "no-events",
        SensorMetadata::default(),
        None,
        MultiChannelMeasurement::new(
            vec![0.0, 0.7],
            vec![MeasurementChannel::new("E1", "V", vec![Some(0.0); 2])],
        )
        .unwrap(),
        Vec::new(),
        Vec::new(),
        "buffer",
        provenance(),
    )
    .unwrap();
    rust_electroanalysis_cli::estimation::environment::bind_compiled_transition_inputs(
        &no_event_experiment,
        &mut no_event_environment,
        &ActivityConfig::default(),
        None,
        1,
        &TransductionDriveSource::None,
    );
    assert!(no_event_environment[1].delta_log10_activity.is_none());
}

#[test]
fn compiled_transduction_drive_modes_cover_none_activity_step_event_field_and_failures() {
    let make_event = |timestamp: f64, value: f64, metadata| ExperimentEvent {
        timestamp,
        kind: ExperimentEventKind::ManualAnnotation,
        value: Some(value),
        unit: Some("activity".into()),
        analyte: None,
        annotation: None,
        metadata,
    };
    let experiment = ElectrochemicalExperiment::new(
        "transduction-modes",
        SensorMetadata::default(),
        None,
        MultiChannelMeasurement::new(
            vec![0.0, 1.0, 2.0],
            vec![MeasurementChannel::new("E1", "V", vec![Some(0.0); 3])],
        )
        .unwrap(),
        Vec::new(),
        vec![
            ExperimentEvent {
                timestamp: 0.0,
                kind: ExperimentEventKind::ConcentrationStep,
                value: Some(1e-3),
                unit: Some("mol/L".into()),
                analyte: None,
                annotation: None,
                metadata: None,
            },
            make_event(
                0.3,
                0.2,
                Some(std::collections::BTreeMap::from([
                    ("drive".into(), "0.2".into()),
                    ("drive_unit".into(), "activity".into()),
                ])),
            ),
            make_event(
                1.6,
                0.3,
                Some(std::collections::BTreeMap::from([
                    ("drive".into(), "0.3".into()),
                    ("drive_unit".into(), "activity".into()),
                ])),
            ),
        ],
        "buffer",
        provenance(),
    )
    .unwrap();
    let aligned = || {
        vec![
            AlignedEnvironment {
                timestamp_s: 0.0,
                temperature_k: Some(298.15),
                ..Default::default()
            },
            AlignedEnvironment {
                timestamp_s: 1.0,
                temperature_k: Some(298.15),
                ..Default::default()
            },
            AlignedEnvironment {
                timestamp_s: 2.0,
                temperature_k: Some(298.15),
                ..Default::default()
            },
        ]
    };
    let mut none = aligned();
    rust_electroanalysis_cli::estimation::environment::bind_compiled_transition_inputs(
        &experiment,
        &mut none,
        &ActivityConfig::default(),
        None,
        1,
        &TransductionDriveSource::None,
    );
    assert!(
        none.iter()
            .all(|environment| environment.transduction_drive.is_none())
    );

    let mut activity_step = aligned();
    rust_electroanalysis_cli::estimation::environment::bind_compiled_transition_inputs(
        &experiment,
        &mut activity_step,
        &ActivityConfig::default(),
        None,
        1,
        &TransductionDriveSource::ActivityStep,
    );
    assert_eq!(activity_step[1].transduction_drive, None);
    assert_eq!(
        activity_step[1].transduction_event_timestamps_s,
        Vec::<f64>::new()
    );

    let mut explicit = aligned();
    rust_electroanalysis_cli::estimation::environment::bind_compiled_transition_inputs(
        &experiment,
        &mut explicit,
        &ActivityConfig::default(),
        None,
        1,
        &TransductionDriveSource::ExplicitEventField {
            field: "drive".into(),
            unit: "activity".into(),
        },
    );
    assert_eq!(explicit[1].transduction_drive, Some(0.2));
    assert_eq!(explicit[2].transduction_drive, Some(0.3));
    assert_eq!(explicit[1].transduction_event_timestamps_s, vec![0.3]);

    let missing_field_source = TransductionDriveSource::ExplicitEventField {
        field: "missing".into(),
        unit: "activity".into(),
    };
    let mut missing = aligned();
    rust_electroanalysis_cli::estimation::environment::bind_compiled_transition_inputs(
        &experiment,
        &mut missing,
        &ActivityConfig::default(),
        None,
        1,
        &missing_field_source,
    );
    assert!(
        missing
            .iter()
            .all(|environment| environment.transduction_drive.is_none())
    );

    let mut invalid_unit = aligned();
    rust_electroanalysis_cli::estimation::environment::bind_compiled_transition_inputs(
        &experiment,
        &mut invalid_unit,
        &ActivityConfig::default(),
        None,
        1,
        &TransductionDriveSource::ExplicitEventField {
            field: "drive".into(),
            unit: "V".into(),
        },
    );
    assert!(
        invalid_unit
            .iter()
            .all(|environment| environment.transduction_drive.is_none())
    );
    assert!(invalid_unit.iter().any(|environment| {
        environment
            .warnings
            .iter()
            .any(|warning| warning.message.contains("incompatible"))
    }));
}

#[test]
fn compiled_reduced_simulation_emits_stable_state_truth_and_one_shot_step_input() {
    let scenario = simulation::SimulationScenario {
        sample_count: 100,
        activity_step_time_s: Some(10.0),
        model: rust_electroanalysis_cli::estimation_config::EstimationModelConfig {
            backend: rust_electroanalysis_cli::estimation_config::EstimationModelBackend::Compiled,
            profile:
                rust_electroanalysis_cli::estimation_config::CompiledEstimationProfile::ReducedIsmV1,
            ..Default::default()
        },
        ..Default::default()
    };
    let output = simulation::simulate_scenario(&scenario).unwrap();
    let stepped = output
        .observations
        .iter()
        .find(|point| point.timestamp_s >= 10.0)
        .and_then(|point| point.compiled.as_ref())
        .unwrap();
    assert!(
        stepped
            .state_values
            .contains_key("dynamic_fast_potential_v")
    );
    assert!(
        stepped
            .state_values
            .contains_key("dynamic_slow_potential_v")
    );
    assert!(stepped.state_values.contains_key("reference_offset_v"));
    assert_eq!(stepped.event_inputs.get("delta_log10_activity"), Some(&1.0));
    assert!(stepped.predicted_potential_v.is_finite());
    assert!(
        output
            .observations
            .iter()
            .filter_map(|point| point.compiled.as_ref())
            .filter(|truth| truth.event_inputs.contains_key("delta_log10_activity"))
            .count()
            == 1
    );
    let validation_truth =
        rust_electroanalysis_cli::estimation::validation::truth_from_simulation(&output);
    assert!(
        validation_truth
            .iter()
            .any(|point| point.state_values.contains_key("dynamic_fast_potential_v"))
    );
    assert!(
        validation_truth
            .iter()
            .any(|point| point.component_potentials_v.contains_key("dynamic_fast"))
    );
}

#[test]
fn compiled_reduced_active_transduction_truth_and_validation_use_stable_state_ids() {
    let scenario = simulation::SimulationScenario {
        sample_count: 18,
        interval_s: 1.0,
        activity_step_time_s: Some(4.0),
        measurement_noise_sd_v: 0.0,
        model: rust_electroanalysis_cli::estimation_config::EstimationModelConfig {
            backend: EstimationModelBackend::Compiled,
            profile: CompiledEstimationProfile::ReducedIsmV1,
            transduction_drive: TransductionDriveSource::ActivityStep,
            ..Default::default()
        },
        ..Default::default()
    };
    let output = simulation::simulate_scenario(&scenario).unwrap();
    let active = output
        .observations
        .iter()
        .find(|point| {
            point
                .compiled
                .as_ref()
                .is_some_and(|truth| truth.event_inputs.contains_key("transduction_drive"))
        })
        .and_then(|point| point.compiled.as_ref())
        .unwrap();
    for id in [
        "dynamic_fast_potential_v",
        "dynamic_slow_potential_v",
        "reference_offset_v",
        "transduction_candidate_potential_v",
    ] {
        assert!(
            active.state_values.contains_key(id),
            "missing stable truth id {id}"
        );
    }
    assert!(
        active
            .component_contributions
            .iter()
            .any(|contribution| contribution.component_id == "transduction_candidate")
    );
    let serialized = serde_json::to_string(active).unwrap();
    assert!(
        serialized.find("dynamic_fast_potential_v").unwrap()
            < serialized
                .find("transduction_candidate_potential_v")
                .unwrap()
    );

    let truth = rust_electroanalysis_cli::estimation::validation::truth_from_simulation(&output);
    let values = output
        .observations
        .iter()
        .map(|point| point.observed_potential_v)
        .collect::<Vec<_>>();
    let times = output
        .observations
        .iter()
        .map(|point| point.timestamp_s)
        .collect::<Vec<_>>();
    let mut config = config(StateModelKind::Activity);
    config.model.backend = EstimationModelBackend::Compiled;
    config.model.profile = CompiledEstimationProfile::ReducedIsmV1;
    config.model.transduction_drive = TransductionDriveSource::ActivityStep;
    let report = estimation::estimate_experiment(
        &experiment(values, times),
        "E1",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &config,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let validation = rust_electroanalysis_cli::estimation::validation::validate_report(
        &report,
        &truth,
        Some("compiled simulation truth".into()),
    );
    for id in [
        "dynamic_fast_potential_v",
        "dynamic_slow_potential_v",
        "reference_offset_v",
        "transduction_candidate_potential_v",
    ] {
        let metric = validation
            .metrics
            .iter()
            .find(|metric| metric.state == id)
            .unwrap();
        assert!(metric.rmse.is_some());
        assert!(metric.mae.is_some());
        assert!(metric.bias.is_some());
        assert!(metric.interval_coverage.is_some());
    }
}

#[test]
fn compiled_validation_keeps_absent_truth_metrics_unavailable() {
    let scenario = simulation::SimulationScenario {
        sample_count: 12,
        interval_s: 1.0,
        activity_step_time_s: Some(4.0),
        measurement_noise_sd_v: 0.0,
        model: rust_electroanalysis_cli::estimation_config::EstimationModelConfig {
            backend: EstimationModelBackend::Compiled,
            profile: CompiledEstimationProfile::ReducedIsmV1,
            ..Default::default()
        },
        ..Default::default()
    };
    let output = simulation::simulate_scenario(&scenario).unwrap();
    let values = output
        .observations
        .iter()
        .map(|point| point.observed_potential_v)
        .collect::<Vec<_>>();
    let times = output
        .observations
        .iter()
        .map(|point| point.timestamp_s)
        .collect::<Vec<_>>();
    let mut config = config(StateModelKind::Activity);
    config.model.backend = EstimationModelBackend::Compiled;
    config.model.profile = CompiledEstimationProfile::ReducedIsmV1;
    config.observability.reject_unobservable_model = false;
    let report = estimation::estimate_experiment(
        &experiment(values, times),
        "E1",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &config,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();

    let mut truth =
        rust_electroanalysis_cli::estimation::validation::truth_from_simulation(&output);
    assert!(
        truth
            .iter()
            .all(|point| point.state_values.contains_key("reference_offset_v"))
    );
    for point in &mut truth {
        point.state_values.remove("reference_offset_v");
    }
    let validation = rust_electroanalysis_cli::estimation::validation::validate_report(
        &report,
        &truth,
        Some("compiled simulation truth with omitted reference state".into()),
    );

    let available = validation
        .metrics
        .iter()
        .find(|metric| metric.state == "dynamic_fast_potential_v")
        .unwrap();
    assert!(available.rmse.is_some());
    assert!(available.mae.is_some());
    assert!(available.bias.is_some());
    let omitted = validation
        .metrics
        .iter()
        .find(|metric| metric.state == "reference_offset_v")
        .unwrap();
    assert_eq!(omitted.sample_count, 0);
    assert!(omitted.rmse.is_none());
    assert!(omitted.mae.is_none());
    assert!(omitted.bias.is_none());
    assert!(omitted.interval_coverage.is_none());
    assert!(validation.matched_sample_count > 0);
}

#[test]
fn state_transform_round_trips_and_reports_derivatives() {
    use rust_electroanalysis_cli::estimation::state::StateTransform;
    let log10 = StateTransform::Log10Positive;
    assert!((log10.to_physical(-3.0, None, None).unwrap() - 1.0e-3).abs() < 1.0e-15);
    assert!((log10.from_physical(1.0e-3, None, None).unwrap() + 3.0).abs() < 1.0e-12);
    assert!(log10.derivative(-3.0, None, None).unwrap() > 0.0);
    let logistic = StateTransform::LogisticBounded;
    let physical = logistic.to_physical(0.0, Some(0.5), Some(1.5)).unwrap();
    assert!((physical - 1.0).abs() < 1.0e-12);
    assert!(
        (logistic
            .from_physical(physical, Some(0.5), Some(1.5))
            .unwrap())
        .abs()
            < 1e-12
    );
    assert!(logistic.validate_bounds(Some(0.5), Some(1.5)).is_ok());
    assert!(logistic.validate_bounds(None, Some(1.5)).is_err());
}

#[test]
fn old_estimation_artifact_defaults_new_diagnostics_fields() {
    let report = estimation::estimate_experiment(
        &experiment(vec![Some(0.02252), Some(0.02252)], vec![0.0, 1.0]),
        "E1/V",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &config(StateModelKind::Activity),
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let mut json = serde_json::to_value(&report).unwrap();
    if let Some(estimates) = json
        .get_mut("estimates")
        .and_then(|value| value.as_array_mut())
    {
        for estimate in estimates {
            if let Some(object) = estimate.as_object_mut() {
                object.remove("posterior_constrained");
                object.remove("applied_measurement_variance_v2");
                object.remove("uninflated_measurement_variance_v2");
                object.remove("measurement_variance_source");
                object.remove("variance_inflation_factor");
                object.remove("variance_inflation_reason");
            }
        }
    }
    let decoded: rust_electroanalysis_cli::results::StateEstimationReport =
        serde_json::from_value(json).unwrap();
    assert!(!decoded.estimates[0].posterior_constrained);
    assert!(decoded.estimates[0].measurement_variance_source.is_none());
}

#[test]
fn old_estimation_configuration_fixture_matrix_preserves_legacy_defaults() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/estimation/migration");
    let cases = [
        (
            "no_model_section.toml",
            EstimationModelBackend::Legacy,
            None,
            false,
        ),
        (
            "early_compiled_model.toml",
            EstimationModelBackend::Compiled,
            None,
            false,
        ),
        (
            "legacy_profile_alias.toml",
            EstimationModelBackend::Legacy,
            None,
            false,
        ),
        (
            "old_variance_and_transient_prior.toml",
            EstimationModelBackend::Legacy,
            Some(12.0),
            true,
        ),
    ];
    for (file, backend, tau, expects_variance_warning) in cases {
        let loaded =
            ResolvedEstimationConfig::load(&root, Some(PathBuf::from(file).as_path())).unwrap();
        assert_eq!(loaded.config.schema_version, 3, "fixture {file}");
        assert_eq!(loaded.config.model.backend, backend, "fixture {file}");
        assert_eq!(
            loaded.config.model.profile,
            CompiledEstimationProfile::LegacyEquivalentV1,
            "fixture {file}"
        );
        assert!(loaded.config.model.definition.is_none());
        assert_eq!(loaded.config.model.input_bindings.custom.len(), 0);
        if let Some(expected_tau) = tau {
            assert_eq!(loaded.config.polarization.configured_tau_s, expected_tau);
            assert_eq!(loaded.config.measurement_noise.configured_variance_v2, 2e-6);
        }
        assert_eq!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("standard_variance_v2")),
            expects_variance_warning,
            "fixture {file}"
        );
    }
}

#[test]
fn estimation_report_matrix_renders_honest_backend_profile_and_custom_definition() {
    let calibration =
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap();
    let run = |config: ResolvedEstimationConfig| {
        estimation::estimate_experiment(
            &experiment(
                vec![Some(0.02252), Some(0.02250), Some(0.02248)],
                vec![0.0, 1.0, 2.0],
            ),
            "E1",
            calibration.clone(),
            &config,
            estimation::EstimationContext::default(),
            FilterKind::Ekf,
        )
        .unwrap()
    };

    let legacy = run(config(StateModelKind::Activity));
    assert_eq!(legacy.model_backend, Some(EstimationModelBackend::Legacy));
    assert!(legacy.model_profile.is_none());
    assert!(
        rust_electroanalysis_cli::runners::estimation::human_report(&legacy)
            .contains("measurement model: stored calibration adapter")
    );

    let mut compiled_legacy = config(StateModelKind::Activity);
    compiled_legacy.model.backend = EstimationModelBackend::Compiled;
    compiled_legacy.model.profile = CompiledEstimationProfile::LegacyEquivalentV1;
    compiled_legacy.observability.reject_unobservable_model = false;
    let compiled_legacy_report = run(compiled_legacy);
    assert_eq!(
        compiled_legacy_report.model_profile,
        Some(CompiledEstimationProfile::LegacyEquivalentV1)
    );
    assert!(
        rust_electroanalysis_cli::runners::estimation::human_report(&compiled_legacy_report)
            .contains("compiled legacy-equivalent profile")
    );

    let mut reduced = config(StateModelKind::Activity);
    reduced.model.backend = EstimationModelBackend::Compiled;
    reduced.model.profile = CompiledEstimationProfile::ReducedIsmV1;
    reduced.observability.reject_unobservable_model = false;
    let reduced_report = run(reduced);
    let reduced_text = rust_electroanalysis_cli::runners::estimation::human_report(&reduced_report);
    assert!(reduced_text.contains("compiled reduced ISM V1"));
    assert!(reduced_text.contains("component labels are not asserted mechanisms"));

    let mut custom = custom_binding_config();
    custom
        .model
        .input_bindings
        .custom
        .insert("flow_drive".into(), "environment:flow".into());
    custom.environment.flow_series = Some("flow".into());
    let custom_experiment = ElectrochemicalExperiment::new(
        "custom-report",
        SensorMetadata::default(),
        None,
        MultiChannelMeasurement::new(
            vec![0.0, 1.0, 2.0],
            vec![MeasurementChannel::new(
                "E1",
                "V",
                vec![Some(0.0), Some(0.25), Some(0.5)],
            )],
        )
        .unwrap(),
        vec![EnvironmentalSeries {
            name: "flow".into(),
            unit: "m/s".into(),
            time: vec![0.0, 1.0, 2.0],
            values: vec![Some(0.0), Some(1.0), Some(2.0)],
            metadata: None,
        }],
        Vec::new(),
        "buffer",
        provenance(),
    )
    .unwrap();
    let custom_report = estimation::estimate_experiment(
        &custom_experiment,
        "E1",
        calibration,
        &custom,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    assert_eq!(
        custom_report.model_profile,
        Some(CompiledEstimationProfile::Custom)
    );
    assert!(custom_report.model_definition.is_some());
    let definition = custom_report.model_definition.as_ref().unwrap();
    assert_eq!(definition.components[0].id, "custom_flow_disturbance");
    assert_eq!(
        definition.components[0].kind,
        "disturbance.linear_covariate"
    );
    assert_eq!(
        definition.components[0].interpretation_status,
        rust_electroanalysis_cli::model::InterpretationStatus::Phenomenological
    );
    let custom_text = rust_electroanalysis_cli::runners::estimation::human_report(&custom_report);
    assert!(custom_text.contains("custom compiled definition"));
    assert!(custom_text.contains("flow_drive"));
    assert!(custom_text.contains("environment:flow"));
}

#[test]
fn old_estimation_artifact_migration_matrix_keeps_identity_honest_and_deterministic() {
    let report = estimation::estimate_experiment(
        &experiment(vec![Some(0.02252), Some(0.02252)], vec![0.0, 1.0]),
        "E1",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &config(StateModelKind::Activity),
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();
    let mut old_report = serde_json::to_value(&report).unwrap();
    old_report["schema_version"] = serde_json::json!(1);
    for field in [
        "model_backend",
        "model_profile",
        "model_id",
        "model_schema_version",
        "compiled_model_summary",
        "state_bindings",
        "model_definition",
        "resolved_model_definition_source",
        "resolved_input_bindings",
        "timestamp_diagnostics",
        "timestamp_policy",
        "timestamp_segments",
        "skipped_timestamp_segments",
        "ingestion_diagnostics",
    ] {
        old_report.as_object_mut().unwrap().remove(field);
    }
    let report_path =
        std::env::temp_dir().join(format!("estimation-old-report-{}.json", std::process::id()));
    fs::write(&report_path, serde_json::to_vec(&old_report).unwrap()).unwrap();
    let migrated: rust_electroanalysis_cli::results::StateEstimationReport =
        rust_electroanalysis_cli::domain::read_artifact(&report_path).unwrap();
    assert_eq!(migrated.model_backend, None);
    assert_eq!(migrated.model_profile, None);
    assert_eq!(migrated.resolved_input_bindings, None);
    fs::remove_file(&report_path).unwrap();

    let mut old_truth = serde_json::to_value(
        simulation::simulate_scenario(&simulation::SimulationScenario {
            sample_count: 3,
            measurement_noise_sd_v: 0.0,
            ..Default::default()
        })
        .unwrap(),
    )
    .unwrap();
    old_truth["schema_version"] = serde_json::json!(2);
    old_truth["scenario"]
        .as_object_mut()
        .unwrap()
        .remove("model");
    for observation in old_truth["observations"].as_array_mut().unwrap() {
        observation.as_object_mut().unwrap().remove("compiled");
    }
    let decoded_truth: simulation::SimulationOutput = serde_json::from_value(old_truth).unwrap();
    assert!(
        decoded_truth
            .observations
            .iter()
            .all(|point| point.compiled.is_none())
    );
    assert_eq!(
        decoded_truth.scenario.model.backend,
        EstimationModelBackend::Legacy
    );

    let validation = rust_electroanalysis_cli::results::validation::ValidationResults {
        schema_version: 1,
        artifact_kind: "ism_model_validation".into(),
        study_id: "old-study".into(),
        metrics: Vec::new(),
        identifiability_report: serde_json::Value::Null,
        model_comparison: Vec::new(),
        warnings: Vec::new(),
    };
    let validation_json = serde_json::to_value(&validation).unwrap();
    let decoded_validation: rust_electroanalysis_cli::results::validation::ValidationResults =
        serde_json::from_value(validation_json.clone()).unwrap();
    assert_eq!(decoded_validation.schema_version, 1);

    let comparison = rust_electroanalysis_cli::estimation::comparison::compare_reports(
        &[(FilterKind::Ekf, report.clone())],
        None,
    );
    let mut old_comparison = serde_json::to_value(&comparison).unwrap();
    old_comparison["schema_version"] = serde_json::json!(2);
    old_comparison
        .as_object_mut()
        .unwrap()
        .remove("ingestion_diagnostics");
    for record in old_comparison["records"].as_array_mut().unwrap() {
        let object = record.as_object_mut().unwrap();
        object.remove("model_backend");
        object.remove("model_profile");
    }
    let decoded_comparison: rust_electroanalysis_cli::results::StateFilterComparison =
        serde_json::from_value(old_comparison).unwrap();
    assert_eq!(decoded_comparison.records[0].model_backend, None);
    assert_eq!(decoded_comparison.records[0].model_profile, None);
    assert_eq!(
        serde_json::to_string(&decoded_comparison).unwrap(),
        serde_json::to_string(&decoded_comparison).unwrap()
    );
}

#[test]
fn estimation_report_records_timestamp_segments_and_row_mapping() {
    let mut c = config(StateModelKind::Activity);
    c.timestamp_handling.minimum_segment_points = 2;
    c.timestamp_handling.reset_threshold_s = 1.0;
    c.timestamp_handling.reset_threshold_fraction = 0.5;
    let report = estimation::estimate_experiment(
        &experiment(
            vec![
                Some(0.02252),
                Some(0.02250),
                Some(0.02248),
                Some(0.02252),
                Some(0.02250),
                Some(0.02248),
            ],
            vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
        ),
        "E1/V",
        StoredCalibrationObservationModel::new(simulation::simulation_model()).unwrap(),
        &c,
        estimation::EstimationContext::default(),
        FilterKind::Ekf,
    )
    .unwrap();

    assert!(report.was_preprocessed);
    assert_eq!(report.timestamp_segments.len(), 2);
    assert!(
        report
            .estimates
            .iter()
            .all(|point| point.original_row_index.is_some())
    );
    let segment_ids = report
        .estimates
        .iter()
        .map(|point| point.segment_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(segment_ids.len(), 2);
}
