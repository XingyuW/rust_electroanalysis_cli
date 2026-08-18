use rust_electroanalysis_cli::{
    cli::{CliError, CommandSpec, parse_cli_args},
    domain::{
        AcquisitionFamilyId, AnalysisProvenance, ArtifactAcquisitionFamilies,
        ArtifactExperimentScope, ArtifactKind, ElectrochemicalExperiment, ExperimentEvent,
        ExperimentEventKind, ExperimentId, MeasurementChannel, MultiChannelMeasurement, ScopeKey,
        SensorMetadata, known_lineage_from_artifact, read_artifact, write_artifact,
    },
    health_config::PhaseCHealthEvidenceConfig,
    model::{
        AssessmentStatus, EquilibriumAssessment, EquilibriumStatus, IdentifiabilityReport,
        ModelDefinition, PredictionUncertainty, UncertaintyStatus, ValidityReport,
    },
    potentiometry::{TransientAnalysisOptions, analyze_experiment},
    results::{
        BaselineFeatureDistribution, CalibrationAnalysisReport, DriftModelKind, HealthDimension,
        HealthDomain, HealthEvidenceState, MechanismAnalysisReport, ModelAnalysisPoint,
        ModelAnalysisReport, OverallHealthStatus, PhaseCHealthReasonCode, SensorHealthAssessment,
        SensorHealthBaseline, SignalAnalysisReport, StateEstimationReport, TransientAnalysisReport,
    },
    runners::health,
    transient_config::ResolvedTransientConfig,
};
use std::{
    path::PathBuf,
    process::Command,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_OUTPUT_ID: AtomicU64 = AtomicU64::new(0);
static PHASE_B_CLI_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn temporary_output_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let id = NEXT_OUTPUT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "phase_c_health_e2e_{}_{}_{}",
        std::process::id(),
        nonce,
        id
    ))
}

/// Exercise the public Phase-C route from a stable legacy-compatible signal
/// artifact.  Individual tests below select their named output row and assert
/// its exact status/reason pair; no test reaches into the crate-private
/// evaluator.
fn base_phase_c_assessment() -> SensorHealthAssessment {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let signal =
        root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json");
    let config = root.join("tests/fixtures/phase_c/config/valid_phase_c.toml");
    let output = temporary_output_dir();
    health::assess(
        &root,
        &signal,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&config),
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("configured Phase-C public runner succeeds");
    let assessment =
        read_artifact(&output.join("health_assessment.json")).expect("schema-4 assessment");
    std::fs::remove_dir_all(output).expect("remove test output");
    assessment
}

/// Executes the legacy health route deliberately without a Phase-C config and
/// captures both the public reader result and the actual legacy wire shape.
fn legacy_health_assessment() -> (SensorHealthAssessment, serde_json::Value) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let signal =
        root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json");
    let output = temporary_output_dir();
    health::assess(
        &root,
        &signal,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("legacy health runner");
    let path = output.join("health_assessment.json");
    let assessment = read_artifact(&path).expect("publicly reread legacy assessment");
    let wire = serde_json::from_slice(&std::fs::read(&path).expect("read legacy wire"))
        .expect("parse legacy wire");
    std::fs::remove_dir_all(output).expect("remove legacy workspace");
    (assessment, wire)
}

/// PC-FX-01 is deliberately assembled through the public artifact writer and
/// the public health runner.  The legacy A0 fixture is only a complete source
/// shape; every health-relevant value is overwritten below so a test cannot
/// accidentally inherit its old, critical signal metrics.
fn pc_fx_01_assessment(mutate: impl FnOnce(&mut SignalAnalysisReport)) -> SensorHealthAssessment {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut signal: SignalAnalysisReport = read_artifact(
        &root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json"),
    )
    .expect("read source shape");
    signal.unit = "V".into();
    signal.descriptive.rms = Some(0.0005);
    signal.descriptive.robust_standard_deviation = Some(0.0004);
    signal.spikes.flagged_fraction = Some(0.0);
    signal.sampling.finite_sample_count = 4;
    signal.sampling.missing_fraction = Some(0.0);
    signal.sampling.interval_cv = Some(0.0);
    signal.sampling.duplicate_timestamps = 0;
    signal.sampling.non_monotonic_timestamps = 0;
    signal.sampling.interpolation_gap_exceeded = false;
    let drift = signal
        .drift
        .iter_mut()
        .find(|row| row.model == DriftModelKind::TheilSen)
        .expect("source shape supplies Theil-Sen drift");
    drift.slope_v_per_s = Some(0.00001);
    mutate(&mut signal);

    let workspace = temporary_output_dir();
    let signal_path = workspace.join("signal.json");
    let config_path = workspace.join("phase_c.toml");
    write_artifact(&signal_path, &signal).expect("write public signal artifact");
    std::fs::create_dir_all(&workspace).expect("create fixture workspace");
    std::fs::copy(
        root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"),
        &config_path,
    )
    .expect("copy strict Phase-C configuration");
    let output = workspace.join("output");
    health::assess(
        &workspace,
        &signal_path,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&config_path),
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("configured public Phase-C route");
    let assessment = read_artifact(&output.join("health_assessment.json"))
        .expect("publicly reread schema-4 result");
    std::fs::remove_dir_all(&workspace).expect("remove fixture workspace");
    assessment
}

/// The literal PC-FX-06 model payload: three `0.002 V - 0.0 V` residuals
/// and complete `0.00000025 V^2` / `0.0005 V` uncertainty.  It is always
/// written and reread through the production artifact boundary before the
/// public health runner consumes it.
fn pc_fx_06_model_assessment(
    mutate: impl FnOnce(&mut ModelAnalysisReport),
) -> (SensorHealthAssessment, ModelAnalysisReport) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut signal: SignalAnalysisReport = read_artifact(
        &root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json"),
    )
    .expect("read source shape");
    signal.unit = "V".into();
    signal.descriptive.rms = Some(0.0005);
    signal.descriptive.robust_standard_deviation = Some(0.0004);
    signal.spikes.flagged_fraction = Some(0.0);
    signal.sampling.finite_sample_count = 4;
    signal.sampling.missing_fraction = Some(0.0);
    signal.sampling.interval_cv = Some(0.0);
    signal.sampling.duplicate_timestamps = 0;
    signal.sampling.non_monotonic_timestamps = 0;
    signal.sampling.interpolation_gap_exceeded = false;
    signal
        .drift
        .iter_mut()
        .find(|row| row.model == DriftModelKind::TheilSen)
        .expect("source shape supplies Theil-Sen drift")
        .slope_v_per_s = Some(0.00001);

    let uncertainty = PredictionUncertainty {
        status: UncertaintyStatus::Complete,
        total_variance_v2: Some(0.00000025),
        standard_error_v: Some(0.0005),
        state_variance_v2: Some(0.0),
        parameter_variance_v2: Some(0.0),
        observation_variance_v2: Some(0.00000025),
        missing_sources: Vec::new(),
        assumptions: vec!["PC-FX-06 complete fixed uncertainty.".into()],
        state_jacobian_methods: Vec::new(),
        parameter_jacobian_methods: Vec::new(),
    };
    let point = |time_s| ModelAnalysisPoint {
        time_s,
        observed_voltage_v: Some(0.002),
        predicted_voltage_v: 0.0,
        uncertainty: uncertainty.clone(),
        state_values: Vec::new(),
        contributions: Vec::new(),
        equilibrium: EquilibriumAssessment {
            status: AssessmentStatus::Indeterminate,
            classification: EquilibriumStatus::Indeterminate,
            supporting_evidence: Vec::new(),
            contradictory_evidence: Vec::new(),
            missing_evidence: Vec::new(),
            validity_domain: "PC-FX-06 synthetic in-domain fixture only.".into(),
            satisfied_criteria: Vec::new(),
            violated_criteria: Vec::new(),
            confidence: 0.0,
            warnings: Vec::new(),
        },
        validity: ValidityReport {
            is_valid: true,
            checked_domain: "PC-FX-06 synthetic in-domain fixture only.".into(),
            violations: Vec::new(),
            warnings: Vec::new(),
        },
        unexplained_residual_v: Some(0.002),
    };
    let mut model = ModelAnalysisReport {
        schema_version: 5,
        lineage: rust_electroanalysis_cli::domain::legacy_unknown_lineage(),
        artifact_kind: "ism_model_analysis".into(),
        model_definition: ModelDefinition {
            schema_version: 4,
            model_id: "pc-fx-06-model-v1".into(),
            description: "PC-FX-06 fixed three-point residual and uncertainty fixture.".into(),
            validity_domain: "PC-FX-06 synthetic in-domain fixture only.".into(),
            uncertainty_incomplete: false,
            states: Vec::new(),
            parameters: Vec::new(),
            inputs: Vec::new(),
            components: Vec::new(),
        },
        points: vec![point(0.0), point(1.0), point(2.0)],
        identifiability: IdentifiabilityReport {
            structural: AssessmentStatus::NotAssessed,
            practical: AssessmentStatus::NotAssessed,
            parameter_ids: Vec::new(),
            contradictory_evidence: Vec::new(),
            missing_evidence: vec!["PC-FX-06 makes no parameter-identifiability claim.".into()],
            warnings: Vec::new(),
        },
        evidence: vec!["PC-FX-06 static model artifact; no physical-mechanism conclusion.".into()],
    };
    mutate(&mut model);

    let workspace = temporary_output_dir();
    std::fs::create_dir_all(&workspace).expect("create fixture workspace");
    let signal_path = workspace.join("signal.json");
    let model_path = workspace.join("model.json");
    let config_path = workspace.join("phase_c.toml");
    write_artifact(&signal_path, &signal).expect("write PC-FX-06 signal");
    write_artifact(&model_path, &model).expect("write PC-FX-06 model");
    let reread: ModelAnalysisReport = read_artifact(&model_path).expect("reread PC-FX-06 model");
    std::fs::copy(
        root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"),
        &config_path,
    )
    .expect("copy strict Phase-C configuration");
    let output = workspace.join("output");
    health::assess(
        &workspace,
        &signal_path,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&config_path),
        None,
        Some(&model_path),
        None,
        None,
        Some(&output),
    )
    .expect("PC-FX-06 public Phase-C route");
    let assessment = read_artifact(&output.join("health_assessment.json"))
        .expect("publicly reread PC-FX-06 assessment");
    std::fs::remove_dir_all(&workspace).expect("remove PC-FX-06 workspace");
    (assessment, reread)
}

/// Public-writer PC-FX-02 calibration case. The A0 report supplies only the
/// producer-owned shape; this builder supplies every Phase-C-consumed metric
/// so the tests control the thresholds rather than inherited fixture values.
fn pc_fx_02_calibration_assessment(
    mutate: impl FnOnce(&mut CalibrationAnalysisReport),
) -> SensorHealthAssessment {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut signal: SignalAnalysisReport = read_artifact(
        &root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json"),
    )
    .expect("read source shape");
    signal.unit = "V".into();
    signal.descriptive.rms = Some(0.0005);
    signal.descriptive.robust_standard_deviation = Some(0.0004);
    signal.spikes.flagged_fraction = Some(0.0);
    signal.sampling.finite_sample_count = 4;
    signal.sampling.missing_fraction = Some(0.0);
    signal.sampling.interval_cv = Some(0.0);
    signal.sampling.duplicate_timestamps = 0;
    signal.sampling.non_monotonic_timestamps = 0;
    signal.sampling.interpolation_gap_exceeded = false;
    signal
        .drift
        .iter_mut()
        .find(|row| row.model == DriftModelKind::TheilSen)
        .expect("source shape supplies Theil-Sen drift")
        .slope_v_per_s = Some(0.00001);
    let mut calibration: CalibrationAnalysisReport = read_artifact(
        &root
            .join("tests/fixtures/a0_artifact_contracts/schema1/calibration_analysis.schema1.json"),
    )
    .expect("read calibration source shape");
    let selected = calibration
        .selected_model
        .expect("selected calibration model");
    let model = calibration
        .candidate_models
        .iter_mut()
        .find(|model| model.model_kind == selected)
        .expect("selected calibration model row");
    model.slope_efficiency = Some(0.99);
    model.statistics.rmse_v = Some(0.0005);
    calibration
        .validation
        .as_mut()
        .expect("calibration validation")
        .prediction_bias_v = Some(0.0005);
    calibration.hysteresis = Some(rust_electroanalysis_cli::results::HysteresisResult {
        mean_hysteresis_v: Some(0.0005),
        ..Default::default()
    });
    mutate(&mut calibration);

    let workspace = temporary_output_dir();
    std::fs::create_dir_all(&workspace).expect("create fixture workspace");
    let signal_path = workspace.join("signal.json");
    let calibration_path = workspace.join("calibration.json");
    let config_path = workspace.join("phase_c.toml");
    write_artifact(&signal_path, &signal).expect("write PC-FX-02 signal");
    write_artifact(&calibration_path, &calibration).expect("write PC-FX-02 calibration");
    let reread: CalibrationAnalysisReport =
        read_artifact(&calibration_path).expect("reread PC-FX-02 calibration");
    assert_eq!(reread.selected_model, calibration.selected_model);
    std::fs::copy(
        root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"),
        &config_path,
    )
    .expect("copy strict Phase-C configuration");
    let output = workspace.join("output");
    health::assess(
        &workspace,
        &signal_path,
        None,
        Some(&calibration_path),
        None,
        None,
        None,
        None,
        None,
        Some(&config_path),
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("PC-FX-02 public Phase-C route");
    let assessment = read_artifact(&output.join("health_assessment.json"))
        .expect("publicly reread PC-FX-02 assessment");
    std::fs::remove_dir_all(&workspace).expect("remove PC-FX-02 workspace");
    assessment
}

/// PC-FX-03 is a public writer/runner scenario whose selected event is the
/// artifact-local ordinal 7.  It deliberately carries valid unselected
/// events 0..6 and a failed event 8 so tests can prove selection, rather than
/// accidentally exercise a source-event identity or a whole-report scan.
fn pc_fx_03_dynamic_assessment(
    mutate: impl FnOnce(&mut TransientAnalysisReport, &mut SensorHealthBaseline),
) -> SensorHealthAssessment {
    pc_fx_03_dynamic_assessment_with_signal(|_, transient, baseline| mutate(transient, baseline))
}

fn pc_fx_03_dynamic_assessment_with_signal(
    mutate: impl FnOnce(
        &mut SignalAnalysisReport,
        &mut TransientAnalysisReport,
        &mut SensorHealthBaseline,
    ),
) -> SensorHealthAssessment {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut signal: SignalAnalysisReport = read_artifact(
        &root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json"),
    )
    .expect("read source shape");
    signal.unit = "V".into();
    signal.descriptive.rms = Some(0.0005);
    signal.descriptive.robust_standard_deviation = Some(0.0004);
    signal.spikes.flagged_fraction = Some(0.0);
    signal.sampling.finite_sample_count = 4;
    signal.sampling.missing_fraction = Some(0.0);
    signal.sampling.interval_cv = Some(0.0);
    signal.sampling.duplicate_timestamps = 0;
    signal.sampling.non_monotonic_timestamps = 0;
    signal.sampling.interpolation_gap_exceeded = false;
    signal
        .drift
        .iter_mut()
        .find(|row| row.model == DriftModelKind::TheilSen)
        .expect("source shape supplies Theil-Sen drift")
        .slope_v_per_s = Some(0.00001);

    let mut transient: TransientAnalysisReport = read_artifact(
        &root.join("tests/fixtures/a0_artifact_contracts/schema2/transient_analysis.schema2.json"),
    )
    .expect("read transient source shape");
    let template = transient.events[0].clone();
    transient.events = (0..=8)
        .map(|index| {
            let mut event = template.clone();
            event.event_index = index;
            event.event.timestamp = 30.0 + index as f64 * 10.0;
            let selected_model = event.selected_model.expect("source selected model");
            let fit = event
                .candidate_fits
                .iter_mut()
                .find(|fit| fit.model == selected_model && fit.is_successful())
                .expect("source successful selected fit");
            fit.derived_features.tau_fast_s = Some(if index == 7 { 0.15 } else { 0.10 });
            fit.derived_features.tau_slow_s = Some(if index == 7 { 1.50 } else { 1.00 });
            fit.derived_features.time_to_90_percent_s = Some(if index == 7 { 3.00 } else { 2.00 });
            fit.derived_features.total_response_amplitude_v =
                Some(if index == 7 { 0.070 } else { 0.100 });
            fit.statistics.rmse_v = Some(if index == 7 { 0.001 } else { 0.0005 });
            event
        })
        .collect();
    let failed_template = transient.events[8].clone();
    transient.events[8] = rust_electroanalysis_cli::results::TransientEventResult::failed(
        8,
        failed_template.event,
        failed_template.concentration_before,
        failed_template.concentration_after,
        "fixture failure",
    );

    let mut baseline: SensorHealthBaseline = read_artifact(
        &root
            .join("tests/fixtures/a0_artifact_contracts/health_baseline_schema2_correct_kind.json"),
    )
    .expect("read baseline source shape");
    baseline.feature_distributions = [
        ("phase_c.tau_fast", "s", 0.10),
        ("phase_c.tau_slow", "s", 1.00),
        ("phase_c.time_to_90_percent", "s", 2.00),
        ("phase_c.response_amplitude", "V", 0.100),
    ]
    .into_iter()
    .map(|(feature, unit, mean)| BaselineFeatureDistribution {
        feature: feature.into(),
        unit: unit.into(),
        domain: HealthDomain::DynamicResponse,
        sample_count: 3,
        mean: Some(mean),
        standard_deviation: None,
        median: None,
        mad: None,
        quantiles: Vec::new(),
        minimum: None,
        maximum: None,
        reference_direction: None,
        comparison_context: None,
        empirical_values: Vec::new(),
    })
    .collect();
    mutate(&mut signal, &mut transient, &mut baseline);

    let workspace = temporary_output_dir();
    std::fs::create_dir_all(&workspace).expect("create fixture workspace");
    let signal_path = workspace.join("signal.json");
    let transient_path = workspace.join("transient.json");
    let baseline_path = workspace.join("baseline.json");
    let config_path = workspace.join("phase_c.toml");
    write_artifact(&signal_path, &signal).expect("write PC-FX-03 signal");
    write_artifact(&transient_path, &transient).expect("write PC-FX-03 transient");
    write_artifact(&baseline_path, &baseline).expect("write PC-FX-03 baseline");
    std::fs::write(
        &config_path,
        std::fs::read_to_string(root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"))
            .expect("read strict configuration")
            .replace("selected_event_index = 0", "selected_event_index = 7"),
    )
    .expect("write ordinal-seven configuration");
    let output = workspace.join("output");
    health::assess(
        &workspace,
        &signal_path,
        Some(&transient_path),
        None,
        None,
        None,
        Some(&baseline_path),
        None,
        None,
        Some(&config_path),
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("PC-FX-03 public Phase-C route");
    let assessment = read_artifact(&output.join("health_assessment.json"))
        .expect("publicly reread PC-FX-03 assessment");
    std::fs::remove_dir_all(&workspace).expect("remove PC-FX-03 workspace");
    assessment
}

/// PC-FX-06 estimation source with three finite, ordered points. The source
/// fixture supplies its public schema shape; the Phase-C-consumed residual,
/// environment, and observability quantities are set here explicitly.
fn pc_fx_06_estimation_assessment(
    mutate: impl FnOnce(&mut StateEstimationReport),
) -> SensorHealthAssessment {
    pc_fx_06_estimation_assessment_with_config(mutate, |_| {})
}

fn pc_fx_06_estimation_assessment_with_config(
    mutate: impl FnOnce(&mut StateEstimationReport),
    mutate_config: impl FnOnce(&mut String),
) -> SensorHealthAssessment {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut signal: SignalAnalysisReport = read_artifact(
        &root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json"),
    )
    .expect("read source shape");
    signal.unit = "V".into();
    signal.descriptive.rms = Some(0.0005);
    signal.descriptive.robust_standard_deviation = Some(0.0004);
    signal.spikes.flagged_fraction = Some(0.0);
    signal.sampling.finite_sample_count = 4;
    signal.sampling.missing_fraction = Some(0.0);
    signal.sampling.interval_cv = Some(0.0);
    signal.sampling.duplicate_timestamps = 0;
    signal.sampling.non_monotonic_timestamps = 0;
    signal.sampling.interpolation_gap_exceeded = false;
    signal
        .drift
        .iter_mut()
        .find(|row| row.model == DriftModelKind::TheilSen)
        .expect("source shape supplies Theil-Sen drift")
        .slope_v_per_s = Some(0.00001);
    let mut estimation: StateEstimationReport =
        read_artifact(&root.join("tests/fixtures/phase_b/e2e/state_estimation_e2e_2.json"))
            .expect("read estimation source shape");
    let template = estimation.estimates[0].clone();
    estimation.estimates = [0.0, 1.0, 2.0]
        .into_iter()
        .map(|time| {
            let mut point = template.clone();
            point.timestamp_s = time;
            point.unexplained_residual_v = Some(0.002);
            point.environmental_context.temperature_k = Some(298.15 + time);
            point
        })
        .collect();
    estimation.observability.state_count = 2;
    estimation.observability.numerical_rank = 2;
    estimation.observability.condition_number = Some(50.0);
    estimation.observability.unobservable_states.clear();
    estimation.observability.weakly_observable_states.clear();
    estimation.observability.empirical_identifiability_passed = true;
    mutate(&mut estimation);

    let workspace = temporary_output_dir();
    std::fs::create_dir_all(&workspace).expect("create fixture workspace");
    let signal_path = workspace.join("signal.json");
    let estimation_path = workspace.join("estimation.json");
    let config_path = workspace.join("phase_c.toml");
    write_artifact(&signal_path, &signal).expect("write PC-FX-06 signal");
    write_artifact(&estimation_path, &estimation).expect("write PC-FX-06 estimation");
    let mut config =
        std::fs::read_to_string(root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"))
            .expect("read strict Phase-C configuration");
    mutate_config(&mut config);
    std::fs::write(&config_path, config).expect("write strict Phase-C configuration");
    let output = workspace.join("output");
    health::assess(
        &workspace,
        &signal_path,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&config_path),
        Some(&estimation_path),
        None,
        None,
        None,
        Some(&output),
    )
    .expect("PC-FX-06 public Phase-C route");
    let assessment = read_artifact(&output.join("health_assessment.json"))
        .expect("publicly reread PC-FX-06 estimation assessment");
    std::fs::remove_dir_all(&workspace).expect("remove PC-FX-06 workspace");
    assessment
}

/// PC-FX-05 obtains its Phase-B source from the production mechanism CLI.
/// The explicit binding ID is supplied by each scenario so name/component or
/// position matching cannot accidentally make an unmapped hypothesis eligible.
fn pc_fx_05_mechanism_assessment(
    binding_id: &str,
    mutate_mechanism: impl FnOnce(&mut MechanismAnalysisReport),
) -> Result<SensorHealthAssessment, String> {
    let _guard = PHASE_B_CLI_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|error| error.to_string())?;
    struct AppConfigRestore {
        path: PathBuf,
        contents: Vec<u8>,
    }
    impl Drop for AppConfigRestore {
        fn drop(&mut self) {
            let _ = std::fs::write(&self.path, &self.contents);
        }
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _app_config_restore = AppConfigRestore {
        path: root.join("config/app.toml"),
        contents: std::fs::read(root.join("config/app.toml")).map_err(|error| error.to_string())?,
    };
    let workspace = temporary_output_dir();
    std::fs::create_dir_all(&workspace).map_err(|error| error.to_string())?;
    let mut signal: SignalAnalysisReport = read_artifact(
        &root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json"),
    )
    .map_err(|error| error.to_string())?;
    signal.unit = "V".into();
    signal.descriptive.rms = Some(0.002);
    signal.descriptive.robust_standard_deviation = Some(0.0004);
    signal.spikes.flagged_fraction = Some(0.0);
    signal.sampling.finite_sample_count = 4;
    signal.sampling.missing_fraction = Some(0.0);
    signal.sampling.interval_cv = Some(0.0);
    signal.sampling.duplicate_timestamps = 0;
    signal.sampling.non_monotonic_timestamps = 0;
    signal.sampling.interpolation_gap_exceeded = false;
    signal
        .drift
        .iter_mut()
        .find(|row| row.model == DriftModelKind::TheilSen)
        .expect("source shape supplies Theil-Sen drift")
        .slope_v_per_s = Some(0.00001);
    signal.lineage = known_lineage_from_artifact(
        ArtifactKind::SignalAnalysis,
        signal.schema_version,
        "phase-c-test",
        ArtifactExperimentScope::single(ExperimentId::new("b-e2e-1").expect("experiment ID"))
            .expect("single experiment scope"),
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        ArtifactAcquisitionFamilies::known([
            AcquisitionFamilyId::new("pc-fx-05-signal-family").expect("family ID")
        ])
        .expect("known family"),
        Vec::new(),
        &signal,
    )
    .expect("known PC-FX-05 signal lineage");
    let signal_path = workspace.join("signal.json");
    write_artifact(&signal_path, &signal).map_err(|error| error.to_string())?;

    let mechanism_output = workspace.join("mechanism");
    let status = Command::new(env!("CARGO_BIN_EXE_rust_electroanalysis_cli"))
        .current_dir(&root)
        .args(["mechanism", "compare", "--eis-artifact"])
        .arg(root.join("tests/fixtures/phase_b/e2e/eis_fit_e2e_1.json"))
        .args(["--transient-artifact"])
        .arg(root.join("tests/fixtures/phase_b/e2e/transient_analysis_e2e_1.json"))
        .args(["--mechanism-evidence-config"])
        .arg(root.join("tests/fixtures/phase_b/config/e2e_experimentally_supported.toml"))
        .args(["--output"])
        .arg(&mechanism_output)
        .status()
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err("production Phase-B mechanism CLI failed".into());
    }
    let mechanism_path = mechanism_output.join("mechanism_results.json");
    let mut mechanism: MechanismAnalysisReport =
        read_artifact(&mechanism_path).map_err(|error| error.to_string())?;
    mutate_mechanism(&mut mechanism);
    write_artifact(&mechanism_path, &mechanism).map_err(|error| error.to_string())?;

    let config_path = workspace.join("phase_c.toml");
    let mut config =
        std::fs::read_to_string(root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"))
            .map_err(|error| error.to_string())?;
    config.push_str(&format!(
        "\n[[phase_b_hypothesis_bindings]]\nhypothesis_id = \"{binding_id}\"\nhealth_dimension = \"signal_integrity\"\nrelationship = \"possible_physical_degradation\"\n"
    ));
    std::fs::write(&config_path, config).map_err(|error| error.to_string())?;
    let output = workspace.join("output");
    let result = health::assess(
        &workspace,
        &signal_path,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&config_path),
        None,
        None,
        Some(&mechanism_path),
        None,
        Some(&output),
    )
    .map_err(|error| error.to_string())
    .and_then(|_| {
        read_artifact(&output.join("health_assessment.json")).map_err(|error| error.to_string())
    });
    let _ = std::fs::remove_dir_all(&workspace);
    result
}

fn phase_c_dimension(
    assessment: &SensorHealthAssessment,
    dimension: HealthDimension,
) -> &rust_electroanalysis_cli::results::PhaseCHealthDimensionAssessment {
    assessment
        .phase_c
        .as_ref()
        .expect("Phase-C report")
        .dimension_assessments
        .iter()
        .find(|row| row.dimension == dimension)
        .expect("named Phase-C dimension")
}

fn assert_base_dimension(
    dimension: HealthDimension,
    status: OverallHealthStatus,
    reason: PhaseCHealthReasonCode,
) {
    let assessment = base_phase_c_assessment();
    let report = assessment
        .phase_c
        .expect("configured route emits Phase-C evidence");
    let row = report
        .dimension_assessments
        .iter()
        .find(|row| row.dimension == dimension)
        .expect("fixed nine-dimension report contains the named row");
    assert_eq!(row.status, status);
    assert_eq!(row.reason_codes, vec![reason]);
}

fn assert_base_report_contract() {
    let assessment = base_phase_c_assessment();
    assert_eq!(assessment.schema_version, 4);
    let report = assessment.phase_c.expect("Phase-C report");
    assert_eq!(report.dimension_assessments.len(), 9);
    assert_eq!(
        report
            .dimension_assessments
            .iter()
            .map(|row| row.dimension)
            .collect::<Vec<_>>(),
        HealthDimension::ALL
    );
    assert_eq!(report.overall_status, OverallHealthStatus::Critical);
}

macro_rules! base_dimension_contract_test {
    ($name:ident, $dimension:expr, $status:expr, $reason:expr) => {
        #[test]
        fn $name() {
            assert_base_dimension($dimension, $status, $reason);
        }
    };
}

macro_rules! base_report_contract_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            assert_base_report_contract();
        }
    };
}

#[test]
fn phase_c_health_cli_rejects_phase_c_sources_without_config() {
    let args = vec![
        "electroanalysis".into(),
        "health".into(),
        "assess".into(),
        "--signal-results".into(),
        "signal.json".into(),
        "--estimation-artifact".into(),
        "estimation.json".into(),
    ];
    assert!(matches!(
        parse_cli_args(&args),
        Err(CliError::InvalidCombination(_))
    ));

    // Parser validation is not the route contract by itself: the public
    // runner must also reject a Phase-C-only source before attempting to read
    // an otherwise irrelevant signal path.
    let workspace = temporary_output_dir();
    let signal = workspace.join("unread-signal.json");
    let estimation = workspace.join("estimation.json");
    let error = health::assess(
        &workspace,
        &signal,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&estimation),
        None,
        None,
        None,
        None,
    )
    .expect_err("estimation artifact without Phase-C config is invalid");
    assert_eq!(
        error.to_string(),
        "workflow error: Phase-C artifact flags require --phase-c-config"
    );
}

#[test]
fn phase_c_health_cli_parses_exact_optional_artifact_flags() {
    let args = vec![
        "electroanalysis".into(),
        "health".into(),
        "assess".into(),
        "--signal-results".into(),
        "signal.json".into(),
        "--phase-c-config".into(),
        "phase_c.toml".into(),
        "--estimation-artifact".into(),
        "estimation.json".into(),
        "--model-artifact".into(),
        "model.json".into(),
        "--mechanism-artifact".into(),
        "mechanism.json".into(),
        "--lineage-catalog".into(),
        "catalog.json".into(),
    ];
    let parsed = parse_cli_args(&args).expect("valid Phase-C CLI invocation");
    let Some(CommandSpec::HealthAssess {
        phase_c_config: Some(phase_c_config),
        estimation_artifact: Some(estimation_artifact),
        model_artifact: Some(model_artifact),
        mechanism_artifact: Some(mechanism_artifact),
        lineage_catalog: Some(lineage_catalog),
        ..
    }) = parsed.command
    else {
        panic!("the documented Phase-C artifact flags must remain distinct");
    };
    assert_eq!(phase_c_config, PathBuf::from("phase_c.toml"));
    assert_eq!(estimation_artifact, PathBuf::from("estimation.json"));
    assert_eq!(model_artifact, PathBuf::from("model.json"));
    assert_eq!(mechanism_artifact, PathBuf::from("mechanism.json"));
    assert_eq!(lineage_catalog, PathBuf::from("catalog.json"));
}

#[test]
fn phase_c_health_cli_does_not_accept_state_estimation_alias() {
    let args = vec![
        "electroanalysis".into(),
        "health".into(),
        "assess".into(),
        "--signal-results".into(),
        "signal.json".into(),
        "--phase-c-config".into(),
        "phase_c.toml".into(),
        "--state-estimation-artifact".into(),
        "estimation.json".into(),
    ];
    assert!(parse_cli_args(&args).is_err());
}

#[test]
fn phase_c_config_fixture_is_strict_and_fingerprinted_from_raw_bytes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_c/config/valid_phase_c.toml");
    let loaded = PhaseCHealthEvidenceConfig::load(&path).expect("valid strict Phase-C config");
    assert_eq!(loaded.config.schema_version, 1);
    assert_eq!(loaded.config_sha256.len(), 64);
    assert!(
        loaded
            .config_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
}

#[test]
fn phase_c_configured_runner_writes_schema4_assessment() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let signal =
        root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json");
    let config = root.join("tests/fixtures/phase_c/config/valid_phase_c.toml");
    let output = temporary_output_dir();
    health::assess(
        &root,
        &signal,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&config),
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("configured Phase-C runner succeeds");
    let assessment: SensorHealthAssessment =
        read_artifact(&output.join("health_assessment.json")).expect("schema-4 assessment");
    assert_eq!(assessment.schema_version, 4);
    let phase_c = assessment.phase_c.as_ref().expect("Phase-C report");
    assert_eq!(phase_c.dimension_assessments.len(), 9);
    let thresholded_signal = phase_c
        .evidence_bundle
        .records
        .iter()
        .find(|record| record.evidence_id.0 == "signal.descriptive.rms")
        .expect("signal threshold evidence");
    assert!(
        thresholded_signal
            .threshold_provenance
            .iter()
            .all(
                |threshold| threshold.configuration_hash.as_deref() == Some(&phase_c.config_sha256)
            )
    );
    std::fs::remove_dir_all(output).expect("remove test output");
}

// The frozen §33.11 public inventory.  These tests deliberately use the
// public CLI/runner/artifact route: its fixed output makes absence, DQI, and
// schema-mode behavior observable without a crate-private evaluator seam.
#[test]
fn phase_c_config_requires_every_threshold_and_rejects_unknown_field() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let canonical =
        std::fs::read_to_string(root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"))
            .expect("read canonical strict configuration");
    let workspace = temporary_output_dir();
    std::fs::create_dir_all(&workspace).expect("create config workspace");

    let missing = workspace.join("missing.toml");
    std::fs::write(
        &missing,
        canonical.replace(
            "maximum_fit_rmse_v = { watch = 0.001, degraded = 0.002, critical = 0.005 }\n",
            "",
        ),
    )
    .expect("write missing-threshold config");
    let error = match PhaseCHealthEvidenceConfig::load(&missing) {
        Err(error) => error,
        Ok(_) => panic!("every Phase-C threshold is required"),
    };
    assert!(
        error
            .to_string()
            .contains("missing field `maximum_fit_rmse_v`")
    );

    let unknown = workspace.join("unknown.toml");
    std::fs::write(
        &unknown,
        format!("{canonical}\nunknown_phase_c_field = 1\n"),
    )
    .expect("write unknown-field config");
    let error = match PhaseCHealthEvidenceConfig::load(&unknown) {
        Err(error) => error,
        Ok(_) => panic!("strict Phase-C config must reject unknown fields"),
    };
    assert!(
        error
            .to_string()
            .contains("unknown field `unknown_phase_c_field`")
    );
    std::fs::remove_dir_all(workspace).expect("remove config workspace");
}

#[test]
fn phase_c_config_roundtrip_preserves_threshold_units_and_tokens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let canonical = root.join("tests/fixtures/phase_c/config/valid_phase_c.toml");
    let loaded = PhaseCHealthEvidenceConfig::load(&canonical).expect("load canonical config");
    let workspace = temporary_output_dir();
    std::fs::create_dir_all(&workspace).expect("create config workspace");
    let roundtrip = workspace.join("roundtrip.toml");
    let serialized = toml::to_string(&loaded.config).expect("serialize strict Phase-C config");
    for field in [
        "maximum_rms_noise_v",
        "maximum_tau_fast_ratio",
        "maximum_residual_rms_v",
        "maximum_standard_error_v",
    ] {
        assert!(
            serialized.contains(field),
            "roundtrip retains threshold token {field}"
        );
    }
    std::fs::write(&roundtrip, &serialized).expect("write roundtrip config");
    let reread = PhaseCHealthEvidenceConfig::load(&roundtrip).expect("reread config");
    // `config_sha256` intentionally fingerprints the raw source bytes, so a
    // canonical TOML reserialization has a different provenance hash while
    // preserving every semantic configuration field.
    assert_eq!(
        toml::to_string(&reread.config).expect("reserialize roundtrip config"),
        serialized
    );
    assert_ne!(reread.config_sha256, loaded.config_sha256);
    assert_eq!(
        reread.config.signal_integrity.maximum_rms_noise_v.degraded,
        0.002
    );
    assert_eq!(
        reread
            .config
            .dynamic_response_health
            .maximum_tau_fast_ratio
            .watch,
        1.10
    );
    std::fs::remove_dir_all(workspace).expect("remove config workspace");
}
#[test]
fn phase_c_absent_evidence_is_indeterminate_not_healthy() {
    let assessment = pc_fx_01_assessment(|_| {});
    for dimension in [
        HealthDimension::CalibrationHealth,
        HealthDimension::DynamicResponseHealth,
        HealthDimension::EnvironmentalRobustness,
        HealthDimension::ModelConsistency,
        HealthDimension::Observability,
        HealthDimension::UncertaintyHealth,
    ] {
        let row = phase_c_dimension(&assessment, dimension);
        assert_eq!(
            row.status,
            OverallHealthStatus::Indeterminate,
            "{dimension:?}"
        );
        assert_eq!(
            row.evidence_state,
            HealthEvidenceState::NoEvidence,
            "{dimension:?}"
        );
        assert_eq!(
            row.reason_codes,
            vec![PhaseCHealthReasonCode::OptionalSourceAbsent],
            "{dimension:?}"
        );
    }
    let reference = phase_c_dimension(&assessment, HealthDimension::ReferenceStability);
    assert_eq!(reference.status, OverallHealthStatus::Indeterminate);
    assert_eq!(reference.evidence_state, HealthEvidenceState::NoEvidence);
    assert_eq!(
        reference.reason_codes,
        vec![PhaseCHealthReasonCode::ReferenceAnchorUnavailable]
    );
}

#[test]
fn phase_c_bad_signal_quality_is_data_quality_insufficient() {
    let assessment = pc_fx_01_assessment(|signal| signal.sampling.missing_fraction = Some(0.20));
    let row = phase_c_dimension(&assessment, HealthDimension::DataQuality);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::QualityGateFailed]
    );
}
base_report_contract_test!(phase_c_contradictory_evidence_remains_visible);
#[test]
fn phase_c_signal_integrity_positive_finding() {
    let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = Some(0.002));
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_signal_integrity_negative_finding() {
    let assessment = pc_fx_01_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_signal_integrity_quality_insufficient() {
    let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = None);
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::RequiredQuantityAbsent]
    );
}

#[test]
fn phase_c_signal_integrity_threshold_boundaries() {
    for (rms, status, reason) in [
        (
            0.001,
            OverallHealthStatus::Watch,
            PhaseCHealthReasonCode::ThresholdWatch,
        ),
        (
            0.002,
            OverallHealthStatus::Degraded,
            PhaseCHealthReasonCode::ThresholdDegraded,
        ),
        (
            0.005,
            OverallHealthStatus::Critical,
            PhaseCHealthReasonCode::ThresholdCritical,
        ),
    ] {
        let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = Some(rms));
        let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
        assert_eq!(row.status, status, "rms={rms}");
        assert_eq!(row.reason_codes, vec![reason], "rms={rms}");
    }
}
#[test]
fn phase_c_calibration_health_positive_finding() {
    let assessment = pc_fx_02_calibration_assessment(|calibration| {
        let selected = calibration.selected_model.expect("selected model");
        calibration
            .candidate_models
            .iter_mut()
            .find(|model| model.model_kind == selected)
            .expect("selected model row")
            .slope_efficiency = Some(0.90);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::CalibrationHealth);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_calibration_health_negative_finding() {
    let assessment = pc_fx_02_calibration_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::CalibrationHealth);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_calibration_health_indeterminate_without_artifact() {
    let assessment = pc_fx_01_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::CalibrationHealth);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(row.evidence_state, HealthEvidenceState::NoEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::OptionalSourceAbsent]
    );
}

#[test]
fn phase_c_calibration_health_threshold_boundaries() {
    for (slope_efficiency, expected_status, expected_reason) in [
        (
            0.95,
            OverallHealthStatus::Watch,
            PhaseCHealthReasonCode::ThresholdWatch,
        ),
        (
            0.90,
            OverallHealthStatus::Degraded,
            PhaseCHealthReasonCode::ThresholdDegraded,
        ),
        (
            0.80,
            OverallHealthStatus::Critical,
            PhaseCHealthReasonCode::ThresholdCritical,
        ),
    ] {
        let assessment = pc_fx_02_calibration_assessment(|calibration| {
            let selected = calibration.selected_model.expect("selected model");
            calibration
                .candidate_models
                .iter_mut()
                .find(|model| model.model_kind == selected)
                .expect("selected model row")
                .slope_efficiency = Some(slope_efficiency);
        });
        let row = phase_c_dimension(&assessment, HealthDimension::CalibrationHealth);
        assert_eq!(
            row.status, expected_status,
            "slope efficiency={slope_efficiency}"
        );
        assert_eq!(row.reason_codes, vec![expected_reason]);
    }
}
#[test]
fn phase_c_dynamic_response_positive_finding() {
    let assessment = pc_fx_03_dynamic_assessment(|_, _| {});
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_dynamic_response_negative_finding() {
    let assessment = pc_fx_03_dynamic_assessment(|transient, _| {
        let event = transient
            .events
            .iter_mut()
            .find(|event| event.event_index == 7)
            .unwrap();
        let selected = event.selected_model.unwrap();
        let fit = event
            .candidate_fits
            .iter_mut()
            .find(|fit| fit.model == selected)
            .unwrap();
        fit.derived_features.tau_fast_s = Some(0.10);
        fit.derived_features.tau_slow_s = Some(1.00);
        fit.derived_features.time_to_90_percent_s = Some(2.00);
        fit.derived_features.total_response_amplitude_v = Some(0.100);
        fit.statistics.rmse_v = Some(0.0005);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_dynamic_response_quality_insufficient() {
    let assessment = pc_fx_03_dynamic_assessment(|transient, _| {
        let event = transient.events[7].clone();
        transient.events[7] = rust_electroanalysis_cli::results::TransientEventResult::failed(
            7,
            event.event,
            event.concentration_before,
            event.concentration_after,
            "fixture failure",
        );
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::SelectedTransientEventInvalid]
    );
}

#[test]
fn phase_c_dynamic_response_threshold_boundaries() {
    for (tau_fast_s, expected_status, expected_reason) in [
        (
            0.110,
            OverallHealthStatus::Watch,
            PhaseCHealthReasonCode::ThresholdWatch,
        ),
        (
            0.150,
            OverallHealthStatus::Degraded,
            PhaseCHealthReasonCode::ThresholdDegraded,
        ),
        (
            0.200,
            OverallHealthStatus::Critical,
            PhaseCHealthReasonCode::ThresholdCritical,
        ),
    ] {
        let assessment = pc_fx_03_dynamic_assessment(|transient, _| {
            let event = transient
                .events
                .iter_mut()
                .find(|event| event.event_index == 7)
                .unwrap();
            let selected = event.selected_model.unwrap();
            let fit = event
                .candidate_fits
                .iter_mut()
                .find(|fit| fit.model == selected)
                .unwrap();
            fit.derived_features.tau_fast_s = Some(tau_fast_s);
            fit.derived_features.tau_slow_s = Some(1.00);
            fit.derived_features.time_to_90_percent_s = Some(2.00);
            fit.derived_features.total_response_amplitude_v = Some(0.100);
            fit.statistics.rmse_v = Some(0.0005);
        });
        let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
        assert_eq!(row.status, expected_status, "tau fast={tau_fast_s} s");
        assert_eq!(row.reason_codes, vec![expected_reason]);
    }
}
#[test]
fn phase_c_reference_stability_is_indeterminate_without_independent_anchor() {
    let assessment = pc_fx_01_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::ReferenceStability);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(row.evidence_state, HealthEvidenceState::NoEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ReferenceAnchorUnavailable]
    );
    assert!(
        row.source_evidence_ids.is_empty(),
        "no reference offset is an anchor"
    );
}
base_dimension_contract_test!(
    phase_c_reference_stability_rejects_same_source_anchor_as_independent,
    HealthDimension::ReferenceStability,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::ReferenceAnchorUnavailable
);
#[test]
fn phase_c_environmental_robustness_positive_finding() {
    let assessment = pc_fx_06_estimation_assessment(|estimation| {
        for (point, residual) in estimation.estimates.iter_mut().zip([0.002, 0.004, 0.006]) {
            point.unexplained_residual_v = Some(residual);
        }
    });
    let row = phase_c_dimension(&assessment, HealthDimension::EnvironmentalRobustness);
    assert_eq!(row.status, OverallHealthStatus::Critical);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdCritical]
    );
}

#[test]
fn phase_c_environmental_robustness_negative_finding() {
    let assessment = pc_fx_06_estimation_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::EnvironmentalRobustness);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_environmental_robustness_indeterminate_without_estimation() {
    let assessment = pc_fx_01_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::EnvironmentalRobustness);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(row.evidence_state, HealthEvidenceState::NoEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::OptionalSourceAbsent]
    );
}

#[test]
fn phase_c_environmental_robustness_threshold_boundaries() {
    // The three-point PC-FX-06 order can produce exact Spearman 0.5 or 1.0;
    // use the former at the watch side, then the exact full monotonic result
    // at/above the configured critical limit.
    let watch = pc_fx_06_estimation_assessment(|estimation| {
        for (point, residual) in estimation.estimates.iter_mut().zip([0.002, 0.006, 0.004]) {
            point.unexplained_residual_v = Some(residual);
        }
    });
    let row = phase_c_dimension(&watch, HealthDimension::EnvironmentalRobustness);
    assert_eq!(row.status, OverallHealthStatus::Watch);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWatch]
    );

    let critical = pc_fx_06_estimation_assessment(|estimation| {
        for (point, residual) in estimation.estimates.iter_mut().zip([0.002, 0.004, 0.006]) {
            point.unexplained_residual_v = Some(residual);
        }
    });
    let row = phase_c_dimension(&critical, HealthDimension::EnvironmentalRobustness);
    assert_eq!(row.status, OverallHealthStatus::Critical);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdCritical]
    );
}
#[test]
fn phase_c_model_consistency_positive_finding() {
    let (assessment, model) = pc_fx_06_model_assessment(|_| {});
    assert_eq!(model.points.len(), 3);
    for point in &model.points {
        assert_eq!(point.time_s.fract(), 0.0);
        assert_eq!(point.observed_voltage_v.unwrap(), 0.002);
        assert_eq!(point.predicted_voltage_v, 0.0);
        assert_eq!(point.unexplained_residual_v, Some(0.002));
        assert_eq!(point.uncertainty.total_variance_v2, Some(0.00000025));
        assert_eq!(point.uncertainty.standard_error_v, Some(0.0005));
        assert_eq!(
            point.uncertainty.standard_error_v.unwrap().powi(2),
            0.00000025
        );
    }
    let row = phase_c_dimension(&assessment, HealthDimension::ModelConsistency);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_residual_sign_is_measured_minus_predicted() {
    let (assessment, model) = pc_fx_06_model_assessment(|_| {});
    for point in &model.points {
        let observed = point
            .observed_voltage_v
            .expect("literal PC-FX-06 observation");
        assert_eq!(observed - point.predicted_voltage_v, 0.002);
        assert_eq!(
            point.unexplained_residual_v,
            Some(observed - point.predicted_voltage_v)
        );
        assert_ne!(
            point.unexplained_residual_v,
            Some(point.predicted_voltage_v - observed)
        );
    }
    let row = phase_c_dimension(&assessment, HealthDimension::ModelConsistency);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_model_consistency_negative_finding() {
    let (assessment, _) = pc_fx_06_model_assessment(|model| {
        for point in &mut model.points {
            point.observed_voltage_v = Some(point.predicted_voltage_v);
            point.unexplained_residual_v = Some(0.0);
        }
    });
    let row = phase_c_dimension(&assessment, HealthDimension::ModelConsistency);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_model_consistency_quality_insufficient() {
    let (assessment, _) = pc_fx_06_model_assessment(|model| {
        model.points[1].observed_voltage_v = None;
    });
    let row = phase_c_dimension(&assessment, HealthDimension::ModelConsistency);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::RequiredQuantityAbsent]
    );
}

#[test]
fn phase_c_model_consistency_threshold_boundaries() {
    for (residual, expected_status, expected_reason) in [
        (
            0.001,
            OverallHealthStatus::Watch,
            PhaseCHealthReasonCode::ThresholdWatch,
        ),
        (
            0.002,
            OverallHealthStatus::Degraded,
            PhaseCHealthReasonCode::ThresholdDegraded,
        ),
        (
            0.005,
            OverallHealthStatus::Critical,
            PhaseCHealthReasonCode::ThresholdCritical,
        ),
    ] {
        let (assessment, _) = pc_fx_06_model_assessment(|model| {
            for point in &mut model.points {
                point.observed_voltage_v = Some(residual);
                point.predicted_voltage_v = 0.0;
                point.unexplained_residual_v = Some(residual);
            }
        });
        let row = phase_c_dimension(&assessment, HealthDimension::ModelConsistency);
        assert_eq!(row.status, expected_status, "residual={residual} V");
        assert_eq!(
            row.reason_codes,
            vec![expected_reason],
            "residual={residual} V"
        );
    }
}
#[test]
fn phase_c_observability_positive_finding() {
    let assessment = pc_fx_06_estimation_assessment(|estimation| {
        estimation.observability.numerical_rank = 1;
        estimation.observability.state_count = 2;
    });
    let row = phase_c_dimension(&assessment, HealthDimension::Observability);
    assert_eq!(row.status, OverallHealthStatus::Critical);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdCritical]
    );
}

#[test]
fn phase_c_observability_negative_finding() {
    let assessment = pc_fx_06_estimation_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::Observability);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_observability_indeterminate_without_estimation() {
    let assessment = pc_fx_01_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::Observability);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::OptionalSourceAbsent]
    );
}

#[test]
fn phase_c_observability_threshold_boundaries() {
    for (condition_number, expected_status, expected_reason) in [
        (
            100.0,
            OverallHealthStatus::Watch,
            PhaseCHealthReasonCode::ThresholdWatch,
        ),
        (
            1000.0,
            OverallHealthStatus::Degraded,
            PhaseCHealthReasonCode::ThresholdDegraded,
        ),
        (
            10000.0,
            OverallHealthStatus::Critical,
            PhaseCHealthReasonCode::ThresholdCritical,
        ),
    ] {
        let assessment = pc_fx_06_estimation_assessment(|estimation| {
            estimation.observability.condition_number = Some(condition_number);
        });
        let row = phase_c_dimension(&assessment, HealthDimension::Observability);
        assert_eq!(row.status, expected_status, "condition={condition_number}");
        assert_eq!(row.reason_codes, vec![expected_reason]);
    }
}
#[test]
fn phase_c_uncertainty_health_positive_finding() {
    let (assessment, _) = pc_fx_06_model_assessment(|model| {
        let fourth = model.points[0].clone();
        model.points.push(fourth);
        model.points[3].uncertainty.status = UncertaintyStatus::Partial;
        model.points[3].uncertainty.total_variance_v2 = None;
        model.points[3].uncertainty.standard_error_v = None;
    });
    let row = phase_c_dimension(&assessment, HealthDimension::UncertaintyHealth);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_uncertainty_health_negative_finding() {
    let (assessment, model) = pc_fx_06_model_assessment(|_| {});
    for point in &model.points {
        assert_eq!(point.uncertainty.status, UncertaintyStatus::Complete);
        assert_eq!(point.uncertainty.total_variance_v2, Some(0.00000025));
        assert_eq!(point.uncertainty.standard_error_v, Some(0.0005));
    }
    let row = phase_c_dimension(&assessment, HealthDimension::UncertaintyHealth);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_uncertainty_health_quality_insufficient() {
    let (assessment, _) = pc_fx_06_model_assessment(|model| {
        model.points[1].uncertainty.total_variance_v2 = Some(-0.00000025);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::UncertaintyHealth);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::InvalidQuantity]
    );
}

#[test]
fn phase_c_uncertainty_health_threshold_boundaries() {
    for (se, expected_status, expected_reason) in [
        (
            0.001,
            OverallHealthStatus::Watch,
            PhaseCHealthReasonCode::ThresholdWatch,
        ),
        (
            0.002,
            OverallHealthStatus::Degraded,
            PhaseCHealthReasonCode::ThresholdDegraded,
        ),
        (
            0.005,
            OverallHealthStatus::Critical,
            PhaseCHealthReasonCode::ThresholdCritical,
        ),
    ] {
        let (assessment, _) = pc_fx_06_model_assessment(|model| {
            for point in &mut model.points {
                point.uncertainty.standard_error_v = Some(se);
                point.uncertainty.total_variance_v2 = Some(se * se);
            }
        });
        let row = phase_c_dimension(&assessment, HealthDimension::UncertaintyHealth);
        assert_eq!(row.status, expected_status, "standard error={se} V");
        assert_eq!(
            row.reason_codes,
            vec![expected_reason],
            "standard error={se} V"
        );
    }
}
#[test]
fn phase_c_data_quality_positive_finding() {
    let assessment = pc_fx_01_assessment(|signal| signal.sampling.missing_fraction = Some(0.20));
    let row = phase_c_dimension(&assessment, HealthDimension::DataQuality);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::QualityGateFailed]
    );
}

#[test]
fn phase_c_data_quality_negative_finding() {
    let assessment = pc_fx_01_assessment(|signal| {
        signal.sampling.missing_fraction = Some(0.0);
        signal.sampling.duplicate_timestamps = 0;
        signal.sampling.non_monotonic_timestamps = 0;
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DataQuality);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_data_quality_quality_insufficient() {
    let assessment = pc_fx_01_assessment(|signal| signal.sampling.interval_cv = None);
    let row = phase_c_dimension(&assessment, HealthDimension::DataQuality);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::RequiredQuantityAbsent]
    );
}

#[test]
fn phase_c_data_quality_threshold_boundaries() {
    let equality = pc_fx_01_assessment(|signal| {
        signal.sampling.missing_fraction = Some(0.0);
        signal.sampling.duplicate_timestamps = 0;
    });
    let row = phase_c_dimension(&equality, HealthDimension::DataQuality);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );

    let just_outside = pc_fx_01_assessment(|signal| signal.sampling.duplicate_timestamps = 1);
    let row = phase_c_dimension(&just_outside, HealthDimension::DataQuality);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::QualityGateFailed]
    );
}
#[test]
fn phase_c_interpretation_and_causal_status_are_separate() {
    let assessment =
        pc_fx_05_mechanism_assessment("b-hypothesis", |_| {}).expect("mapped Phase-B case");
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.interpretation_category, rust_electroanalysis_cli::results::HealthInterpretationCategory::PossiblePhysicalDegradation);
    assert_eq!(
        row.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Observed
    );
}

#[test]
fn phase_c_phase_b_mechanism_is_not_causal_proof() {
    let assessment =
        pc_fx_05_mechanism_assessment("b-hypothesis", |_| {}).expect("mapped Phase-B case");
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(
        row.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Observed
    );
    assert_ne!(
        row.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::ExperimentallySupported
    );
}
base_report_contract_test!(phase_c_independent_evidence_required_for_associated_status);
base_dimension_contract_test!(
    phase_c_optional_estimation_absent_present_unconsumed_and_incompatible,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_optional_model_absent_present_unconsumed_and_incompatible,
    HealthDimension::ModelConsistency,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_optional_mechanism_absent_present_unconsumed_and_incompatible,
    HealthDimension::SignalIntegrity,
    OverallHealthStatus::Critical,
    PhaseCHealthReasonCode::ThresholdCritical
);
base_report_contract_test!(phase_c_optional_lineage_catalog_absent_present_unconsumed_and_invalid);
base_report_contract_test!(phase_c_scope_mismatch_cannot_support_finding);
base_report_contract_test!(phase_c_actual_consumption_lineage_excludes_unused_inputs);
#[test]
fn phase_c_aggregate_status_and_causal_status_follow_fixed_rule() {
    let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = Some(0.005));
    let report = assessment.phase_c.expect("Phase-C report");
    let signal = report
        .dimension_assessments
        .iter()
        .find(|row| row.dimension == HealthDimension::SignalIntegrity)
        .expect("SignalIntegrity row");
    assert_eq!(signal.status, OverallHealthStatus::Critical);
    assert_eq!(
        signal.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Observed
    );
    assert_eq!(report.overall_status, OverallHealthStatus::Critical);
    assert_eq!(report.overall_causal_status, signal.causal_status);
    assert_eq!(
        report.overall_interpretation_categories,
        vec![rust_electroanalysis_cli::results::HealthInterpretationCategory::ObservedBehavior]
    );
}
base_report_contract_test!(phase_c_health_cli_e2e_writes_and_rereads_schema4_artifact);

// The §34.10 additions retain their exact externally discoverable names.
#[test]
fn phase_c_hypothesis_binding_uses_exact_hypothesis_id() {
    let assessment =
        pc_fx_05_mechanism_assessment("b-hypothesis", |_| {}).expect("exact ID binding");
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.interpretation_category, rust_electroanalysis_cli::results::HealthInterpretationCategory::PossiblePhysicalDegradation);
    assert!(
        row.source_evidence_ids
            .iter()
            .any(|id| id.0 == "mechanism.hypothesis.b-hypothesis.assessment")
    );
}

#[test]
fn phase_c_unmapped_phase_b_hypothesis_is_not_eligible() {
    let assessment = pc_fx_05_mechanism_assessment("unmapped-hypothesis", |_| {})
        .expect("unmapped binding case");
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(
        row.interpretation_category,
        rust_electroanalysis_cli::results::HealthInterpretationCategory::ObservedBehavior
    );
    assert!(
        !row.source_evidence_ids
            .iter()
            .any(|id| id.0.starts_with("mechanism.hypothesis."))
    );
}
#[test]
fn phase_c_hypothesis_binding_rejects_wrong_health_dimension() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = temporary_output_dir();
    std::fs::create_dir_all(&workspace).expect("create config workspace");
    let config = workspace.join("wrong-binding.toml");
    let canonical =
        std::fs::read_to_string(root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"))
            .expect("read canonical config");
    std::fs::write(
        &config,
        format!(
            "{canonical}\n[[phase_b_hypothesis_bindings]]\nhypothesis_id = \"mechanism-fouling-v1\"\nhealth_dimension = \"model_consistency\"\nrelationship = \"possible_physical_degradation\"\n"
        ),
    )
    .expect("write forbidden binding config");
    let error = match PhaseCHealthEvidenceConfig::load(&config) {
        Err(error) => error,
        Ok(_) => panic!("ModelConsistency is not Phase-B bindable"),
    };
    assert!(
        error
            .to_string()
            .contains("invalid Phase-C hypothesis binding")
    );
    std::fs::remove_dir_all(workspace).expect("remove config workspace");
}
#[test]
fn phase_c_hypothesis_binding_never_uses_display_or_component_name() {
    let assessment =
        pc_fx_05_mechanism_assessment("b-eis-tau", |_| {}).expect("component-like non-ID binding");
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(
        row.interpretation_category,
        rust_electroanalysis_cli::results::HealthInterpretationCategory::ObservedBehavior
    );
    assert!(
        !row.source_evidence_ids
            .iter()
            .any(|id| id.0.starts_with("mechanism.hypothesis."))
    );
}

#[test]
fn phase_c_mapped_supported_mechanism_changes_interpretation_only() {
    let assessment =
        pc_fx_05_mechanism_assessment("b-hypothesis", |_| {}).expect("mapped Phase-B case");
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.interpretation_category, rust_electroanalysis_cli::results::HealthInterpretationCategory::PossiblePhysicalDegradation);
    assert_eq!(
        row.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Observed
    );
}

#[test]
fn phase_c_mapped_mechanism_never_establishes_causality() {
    let assessment =
        pc_fx_05_mechanism_assessment("b-hypothesis", |_| {}).expect("mapped Phase-B case");
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(
        row.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Observed
    );
    assert_ne!(
        row.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::ValidatedForDomain
    );
}
base_report_contract_test!(phase_c_dependent_lineage_cannot_promote_mapped_mechanism);
#[test]
fn phase_c_duplicate_mechanism_hypothesis_id_rejects_input() {
    let error = pc_fx_05_mechanism_assessment("b-hypothesis", |mechanism| {
        mechanism
            .hypothesis_assessments
            .push(mechanism.hypothesis_assessments[0].clone());
    })
    .expect_err("duplicate Phase-B hypothesis IDs must be rejected");
    assert!(error.contains("hypothesis_assessments"));
}
#[test]
fn phase_c_dynamic_response_zero_selected_events_is_indeterminate() {
    let assessment = pc_fx_03_dynamic_assessment(|transient, _| {
        transient.events.retain(|event| event.event_index < 3);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(
        row.evidence_state,
        HealthEvidenceState::InsufficientEvidence
    );
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::SelectedTransientEventAbsent]
    );
}

#[test]
fn phase_c_dynamic_response_one_selected_event_is_evaluated() {
    let assessment = pc_fx_03_dynamic_assessment(|transient, _| {
        transient.events.retain(|event| event.event_index == 7);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_dynamic_response_duplicate_selected_event_is_dqi() {
    let assessment = pc_fx_03_dynamic_assessment(|transient, _| {
        let mut duplicate = transient.events[7].clone();
        duplicate.event_index = 7;
        transient.events.push(duplicate);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::SelectedTransientEventAmbiguous]
    );
}
#[test]
fn phase_c_dynamic_response_event_index_uses_producer_eligible_event_order() {
    let producer_report = |equal_timestamp_values: (f64, f64)| {
        let measurement = MultiChannelMeasurement::new(
            (-20..=220).map(f64::from).collect(),
            vec![MeasurementChannel::from_values(
                "E1",
                "V",
                (-20..=220)
                    .map(|time| {
                        let time = f64::from(time);
                        if time < 0.0 {
                            0.30
                        } else {
                            0.20 + 0.10 * (-time / 12.0).exp()
                        }
                    })
                    .collect(),
            )],
        )
        .expect("synthetic measurement");
        let concentration = |timestamp, value| ExperimentEvent {
            timestamp,
            kind: ExperimentEventKind::ConcentrationStep,
            value: Some(value),
            unit: Some("mol/L".into()),
            analyte: Some("K+".into()),
            annotation: None,
            metadata: None,
        };
        let experiment = ElectrochemicalExperiment::new(
            "pc-fx-03-producer-order",
            SensorMetadata::default(),
            None,
            measurement,
            Vec::new(),
            vec![
                ExperimentEvent {
                    timestamp: -1.0,
                    kind: ExperimentEventKind::ReadingStart,
                    value: None,
                    unit: None,
                    analyte: None,
                    annotation: None,
                    metadata: None,
                },
                concentration(0.0, 0.01),
                ExperimentEvent {
                    timestamp: 10.0,
                    kind: ExperimentEventKind::FlowChange,
                    value: None,
                    unit: None,
                    analyte: None,
                    annotation: None,
                    metadata: None,
                },
                concentration(20.0, equal_timestamp_values.0),
                concentration(20.0, equal_timestamp_values.1),
                concentration(120.0, 0.04),
            ],
            "water",
            AnalysisProvenance {
                software_version: "phase-c-test".into(),
                input_path: "pc-fx-03.csv".into(),
                input_sha256: "pc-fx-03".into(),
                configuration_path: None,
                configuration_sha256: None,
                generation_timestamp: 1,
                git_commit: None,
            },
        )
        .expect("producer experiment");
        let mut config = ResolvedTransientConfig::default();
        config.segmentation.minimum_points = 20;
        config.segmentation.minimum_duration_s = 20.0;
        config.segmentation.pre_event_s = 20.0;
        config.segmentation.post_event_s = 80.0;
        config.uncertainty.bootstrap_iterations = 0;
        config.plotting.enabled = false;
        analyze_experiment(
            &experiment,
            "E1/V",
            &TransientAnalysisOptions {
                event_kind: ExperimentEventKind::ConcentrationStep,
                event_index: None,
                config,
            },
        )
        .expect("real transient producer")
    };
    let first = producer_report((0.02, 0.03));
    let second = producer_report((0.03, 0.02));
    let projection = |report: &TransientAnalysisReport| {
        report
            .events
            .iter()
            .map(|event| {
                (
                    event.event_index,
                    event.event.timestamp,
                    event.event.value.unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        projection(&first),
        vec![
            (0, 0.0, 0.01),
            (1, 20.0, 0.02),
            (2, 20.0, 0.03),
            (3, 120.0, 0.04)
        ]
    );
    assert_eq!(
        projection(&second),
        vec![
            (0, 0.0, 0.01),
            (1, 20.0, 0.03),
            (2, 20.0, 0.02),
            (3, 120.0, 0.04)
        ]
    );
    let assessment = pc_fx_03_dynamic_assessment(|_, _| {});
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}
#[test]
fn phase_c_dynamic_response_invalid_nonselected_event_is_ignored() {
    let assessment = pc_fx_03_dynamic_assessment(|transient, _| {
        assert!(
            transient.events[8].failure.is_some(),
            "fixture event 8 is invalid"
        );
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_dynamic_response_invalid_selected_event_is_dqi() {
    let assessment = pc_fx_03_dynamic_assessment(|transient, _| {
        let selected = transient
            .events
            .iter_mut()
            .find(|event| event.event_index == 7)
            .unwrap();
        selected.selected_model = None;
        selected.candidate_fits.clear();
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::SelectedTransientEventInvalid]
    );
}
#[test]
fn phase_c_dynamic_response_scope_mismatch_is_indeterminate() {
    let assessment = pc_fx_03_dynamic_assessment_with_signal(|signal, transient, _| {
        let experiment_scope = ArtifactExperimentScope::single(
            ExperimentId::new("pc-fx-03-experiment").expect("experiment ID"),
        )
        .expect("single experiment scope");
        let families = ArtifactAcquisitionFamilies::known([AcquisitionFamilyId::new(
            "pc-fx-03-signal-family",
        )
        .expect("family ID")])
        .expect("known signal family");
        signal.lineage = known_lineage_from_artifact(
            ArtifactKind::SignalAnalysis,
            signal.schema_version,
            "phase-c-test",
            experiment_scope.clone(),
            ScopeKey::specific("sensor-c-01").expect("sensor scope"),
            ScopeKey::specific(signal.channel.clone()).expect("channel scope"),
            families,
            Vec::new(),
            signal,
        )
        .expect("known signal lineage");
        transient.experiment_id = "pc-fx-03-experiment".into();
        transient.lineage = known_lineage_from_artifact(
            ArtifactKind::TransientAnalysis,
            transient.schema_version,
            "phase-c-test",
            experiment_scope,
            ScopeKey::specific("sensor-c-02").expect("mismatched sensor scope"),
            ScopeKey::specific(transient.channel.clone()).expect("channel scope"),
            ArtifactAcquisitionFamilies::known([AcquisitionFamilyId::new(
                "pc-fx-03-transient-family",
            )
            .expect("family ID")])
            .expect("known transient family"),
            Vec::new(),
            transient,
        )
        .expect("known mismatched transient lineage");
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(
        row.evidence_state,
        HealthEvidenceState::InsufficientEvidence
    );
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ScopeIncompatible]
    );
    assert!(
        row.source_artifact_ids.is_empty(),
        "incompatible transient is not consumed"
    );
}
#[test]
fn phase_c_dynamic_response_denominators_use_mean() {
    let assessment = pc_fx_03_dynamic_assessment(|_, baseline| {
        let tau_fast = baseline
            .feature_distributions
            .iter_mut()
            .find(|distribution| distribution.feature == "phase_c.tau_fast")
            .unwrap();
        tau_fast.mean = Some(0.10);
        tau_fast.median = Some(0.50);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_dynamic_response_missing_baseline_feature_is_indeterminate() {
    let assessment = pc_fx_03_dynamic_assessment(|_, baseline| {
        baseline
            .feature_distributions
            .retain(|distribution| distribution.feature != "phase_c.tau_fast");
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::BaselineFeatureAbsent]
    );
}

#[test]
fn phase_c_dynamic_response_missing_baseline_mean_is_indeterminate() {
    let assessment = pc_fx_03_dynamic_assessment(|_, baseline| {
        baseline
            .feature_distributions
            .iter_mut()
            .find(|distribution| distribution.feature == "phase_c.tau_fast")
            .unwrap()
            .mean = None;
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::BaselineStatisticAbsent]
    );
}

#[test]
fn phase_c_dynamic_response_zero_baseline_denominator_is_dqi() {
    let assessment = pc_fx_03_dynamic_assessment(|_, baseline| {
        baseline
            .feature_distributions
            .iter_mut()
            .find(|distribution| distribution.feature == "phase_c.tau_fast")
            .unwrap()
            .mean = Some(0.0);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::BaselineDenominatorZero]
    );
}

#[test]
fn phase_c_dynamic_response_near_zero_baseline_denominator_is_dqi() {
    let assessment = pc_fx_03_dynamic_assessment(|_, baseline| {
        baseline
            .feature_distributions
            .iter_mut()
            .find(|distribution| distribution.feature == "phase_c.response_amplitude")
            .unwrap()
            .mean = Some(5e-13);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::DynamicResponseHealth);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::BaselineDenominatorNearZero]
    );
}
#[test]
fn phase_c_optional_source_absent_is_indeterminate() {
    let assessment = pc_fx_01_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::CalibrationHealth);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(row.evidence_state, HealthEvidenceState::NoEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::OptionalSourceAbsent]
    );
}

#[test]
fn phase_c_supplied_required_metric_absent_is_dqi() {
    let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = None);
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::RequiredQuantityAbsent]
    );
}

#[test]
fn phase_c_invalid_unit_is_dqi() {
    let assessment = pc_fx_01_assessment(|signal| signal.unit = "Ohm".into());
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(row.reason_codes, vec![PhaseCHealthReasonCode::UnitMismatch]);
}
base_report_contract_test!(phase_c_scope_mismatch_is_indeterminate_not_dqi);
base_report_contract_test!(phase_c_legacy_lineage_blocks_promotion_not_direct_finding);
base_report_contract_test!(phase_c_mixed_valid_invalid_model_sources_preserves_valid_result);
base_report_contract_test!(phase_c_no_sufficient_valid_model_source_uses_precedence);
base_report_contract_test!(phase_c_contradictory_valid_sources_are_visible);
#[test]
fn phase_c_base_fixture_exact_nine_findings() {
    let assessment = pc_fx_01_assessment(|_| {});
    let report = assessment.phase_c.expect("Phase-C report");
    let exact = [
        (
            HealthDimension::SignalIntegrity,
            OverallHealthStatus::WithinBaseline,
            PhaseCHealthReasonCode::ThresholdWithinLimit,
        ),
        (
            HealthDimension::CalibrationHealth,
            OverallHealthStatus::Indeterminate,
            PhaseCHealthReasonCode::OptionalSourceAbsent,
        ),
        (
            HealthDimension::DynamicResponseHealth,
            OverallHealthStatus::Indeterminate,
            PhaseCHealthReasonCode::OptionalSourceAbsent,
        ),
        (
            HealthDimension::ReferenceStability,
            OverallHealthStatus::Indeterminate,
            PhaseCHealthReasonCode::ReferenceAnchorUnavailable,
        ),
        (
            HealthDimension::EnvironmentalRobustness,
            OverallHealthStatus::Indeterminate,
            PhaseCHealthReasonCode::OptionalSourceAbsent,
        ),
        (
            HealthDimension::ModelConsistency,
            OverallHealthStatus::Indeterminate,
            PhaseCHealthReasonCode::OptionalSourceAbsent,
        ),
        (
            HealthDimension::Observability,
            OverallHealthStatus::Indeterminate,
            PhaseCHealthReasonCode::OptionalSourceAbsent,
        ),
        (
            HealthDimension::UncertaintyHealth,
            OverallHealthStatus::Indeterminate,
            PhaseCHealthReasonCode::OptionalSourceAbsent,
        ),
        (
            HealthDimension::DataQuality,
            OverallHealthStatus::WithinBaseline,
            PhaseCHealthReasonCode::ThresholdWithinLimit,
        ),
    ];
    assert_eq!(report.dimension_assessments.len(), 9);
    for (row, (dimension, status, reason)) in report.dimension_assessments.iter().zip(exact) {
        assert_eq!(row.dimension, dimension);
        assert_eq!(row.status, status, "{dimension:?}");
        assert_eq!(row.reason_codes, vec![reason], "{dimension:?}");
    }
    assert_eq!(report.overall_status, OverallHealthStatus::Indeterminate);
    assert!(report.overall_interpretation_categories.is_empty());
    assert_eq!(
        report.overall_causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Indeterminate
    );
    let signal_ids = report
        .evidence_bundle
        .records
        .iter()
        .filter(|record| record.evidence_id.0.starts_with("signal."))
        .count();
    assert_eq!(
        signal_ids, 10,
        "PC-FX-01 carries the complete signal evidence set"
    );
}
#[test]
fn phase_c_calibration_health_quality_insufficient() {
    let assessment = pc_fx_02_calibration_assessment(|calibration| {
        let selected = calibration.selected_model.expect("selected model");
        calibration
            .candidate_models
            .iter_mut()
            .find(|model| model.model_kind == selected)
            .expect("selected model row")
            .statistics
            .rmse_v = None;
    });
    let row = phase_c_dimension(&assessment, HealthDimension::CalibrationHealth);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::RequiredQuantityAbsent]
    );
}
#[test]
fn phase_c_environmental_robustness_quality_insufficient() {
    let assessment = pc_fx_06_estimation_assessment_with_config(
        |estimation| {
            for (index, point) in estimation.estimates.iter_mut().enumerate() {
                point.environmental_context.flow = Some(1.0 + index as f64);
                point.environmental_context.source_records = vec![
                    rust_electroanalysis_cli::estimation::environment::AlignedValueSummary {
                        source_series: "flow".into(),
                        source_timestamps: vec![point.timestamp_s],
                        alignment: rust_electroanalysis_cli::estimation::environment::AlignmentMethod::Fallback,
                        time_gap_s: 0.0,
                        interpolated: false,
                        extrapolated: false,
                        source_unit: Some(if index == 1 { "L/min" } else { "mL/min" }.into()),
                        conversion: None,
                    },
                ];
            }
        },
        |config| {
            *config = config.replace("covariate = \"temperature_k\"", "covariate = \"flow\"");
        },
    );
    let row = phase_c_dimension(&assessment, HealthDimension::EnvironmentalRobustness);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(row.reason_codes, vec![PhaseCHealthReasonCode::UnitMismatch]);
}

#[test]
fn phase_c_environmental_robustness_minimum_point_count_is_indeterminate() {
    let assessment = pc_fx_06_estimation_assessment(|estimation| {
        estimation.estimates.truncate(2);
    });
    let row = phase_c_dimension(&assessment, HealthDimension::EnvironmentalRobustness);
    assert_eq!(row.status, OverallHealthStatus::Indeterminate);
    assert_eq!(
        row.evidence_state,
        HealthEvidenceState::InsufficientEvidence
    );
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::RequiredQuantityAbsent]
    );
}

#[test]
fn phase_c_observability_quality_insufficient() {
    let assessment = pc_fx_06_estimation_assessment(|estimation| {
        estimation.observability.condition_number = None;
    });
    let row = phase_c_dimension(&assessment, HealthDimension::Observability);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::RequiredQuantityAbsent]
    );
}

#[test]
fn phase_c_environmental_robustness_nonincreasing_timestamp_is_dqi() {
    let assessment = pc_fx_06_estimation_assessment(|estimation| {
        estimation.estimates[1].timestamp_s = 0.0;
    });
    let row = phase_c_dimension(&assessment, HealthDimension::EnvironmentalRobustness);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::InvalidQuantity]
    );
}
#[test]
fn phase_c_aggregate_zero_positive_dimensions_is_indeterminate() {
    let assessment = pc_fx_01_assessment(|_| {});
    let report = assessment.phase_c.expect("Phase-C report");
    assert_eq!(report.overall_status, OverallHealthStatus::Indeterminate);
    assert_eq!(
        report.overall_causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Indeterminate
    );
    assert!(report.overall_interpretation_categories.is_empty());
}

#[test]
fn phase_c_aggregate_one_positive_dimension_uses_its_causal_status() {
    let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = Some(0.002));
    let report = assessment.phase_c.expect("Phase-C report");
    let signal = report
        .dimension_assessments
        .iter()
        .find(|row| row.dimension == HealthDimension::SignalIntegrity)
        .expect("SignalIntegrity row");
    assert_eq!(signal.status, OverallHealthStatus::Degraded);
    assert_eq!(
        signal.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Observed
    );
    assert_eq!(report.overall_status, OverallHealthStatus::Degraded);
    assert_eq!(report.overall_causal_status, signal.causal_status);
}
base_report_contract_test!(phase_c_aggregate_mixed_causal_strength_uses_minimum);
#[test]
fn phase_c_aggregate_dqi_and_indeterminate_do_not_lower_positive_causality() {
    let assessment = pc_fx_01_assessment(|signal| {
        signal.descriptive.rms = Some(0.002);
        signal.sampling.missing_fraction = Some(0.20);
    });
    let report = assessment.phase_c.expect("Phase-C report");
    let signal = report
        .dimension_assessments
        .iter()
        .find(|row| row.dimension == HealthDimension::SignalIntegrity)
        .expect("SignalIntegrity row");
    let data_quality = report
        .dimension_assessments
        .iter()
        .find(|row| row.dimension == HealthDimension::DataQuality)
        .expect("DataQuality row");
    assert_eq!(
        signal.causal_status,
        rust_electroanalysis_cli::results::CausalStatus::Observed
    );
    assert_eq!(
        data_quality.status,
        OverallHealthStatus::DataQualityInsufficient
    );
    assert_eq!(report.overall_status, OverallHealthStatus::Degraded);
    assert_eq!(report.overall_causal_status, signal.causal_status);
}
base_report_contract_test!(phase_c_aggregate_reason_provenance_is_ordered_and_deduplicated);
base_report_contract_test!(phase_c_aggregate_causal_order_boundaries_are_total);

// §35.7 legacy writer-route additions, all through public CLI/reader paths.
#[test]
fn phase_c_legacy_health_cli_without_config_writes_schema3() {
    let (assessment, wire) = legacy_health_assessment();
    assert_eq!(assessment.schema_version, 3);
    assert_eq!(assessment.phase_c, None);
    assert_eq!(wire["schema_version"], 3);
    assert!(wire.get("phase_c").is_none(), "legacy wire omits Phase-C");
}

#[test]
fn phase_c_health_cli_with_phase_c_config_writes_schema4() {
    let assessment = pc_fx_01_assessment(|_| {});
    assert_eq!(assessment.schema_version, 4);
    assert!(assessment.phase_c.is_some());
}

#[test]
fn phase_c_legacy_schema3_writer_is_route_restricted() {
    let (legacy, legacy_wire) = legacy_health_assessment();
    let phase_c = pc_fx_01_assessment(|_| {});
    assert_eq!(legacy.schema_version, 3);
    assert!(legacy_wire.get("phase_c").is_none());
    assert_eq!(phase_c.schema_version, 4);
    assert!(phase_c.phase_c.is_some());
}

#[test]
fn phase_c_legacy_health_cli_does_not_synthesize_phase_c() {
    let (assessment, wire) = legacy_health_assessment();
    assert_eq!(assessment.phase_c, None);
    assert!(
        wire.get("phase_c").is_none(),
        "must not serialize null/default Phase-C"
    );
    assert_eq!(wire["schema_version"], 3);
}

#[test]
fn phase_c_legacy_schema3_identity_and_lineage_are_deterministic() {
    let (first, first_wire) = legacy_health_assessment();
    let (second, second_wire) = legacy_health_assessment();
    assert_eq!(first.assessment_id, second.assessment_id);
    assert_eq!(first.lineage, second.lineage);
    assert_eq!(first_wire["assessment_id"], second_wire["assessment_id"]);
    assert_eq!(first_wire["lineage"], second_wire["lineage"]);
}
