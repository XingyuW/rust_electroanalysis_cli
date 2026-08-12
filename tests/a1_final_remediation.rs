use rust_electroanalysis_cli::{
    EvidenceBundleInputs, assemble_evidence_bundle,
    calibration_config::ResolvedCalibrationConfig,
    data_file::EISData,
    domain::{
        AcquisitionFamilyId, AnalysisProvenance, ArtifactAcquisitionFamilies,
        ArtifactDependencyRole, ArtifactExperimentScope, ArtifactKind, ExperimentId, ScopeKey,
        VersionedArtifact, known_lineage_from_artifact, read_artifact, write_artifact,
    },
    estimation::simulation,
    impedance::parse_circuit_string,
    potentiometry::calibration::{fit_calibration, observations::extract_observations},
    results::{
        CalibrationAnalysisReport, CalibrationPotentialSource, CircuitFitResult, EisFitArtifact,
        StateEstimationReport, StoredCalibrationModel, TransientAnalysisReport,
    },
    runners::estimation::{self as estimation_runner, RunOptions},
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_workspace(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("a1_final_{label}_{}_{}", std::process::id(), nonce));
    fs::create_dir_all(&path).unwrap();
    path
}

fn known_lineage<T: serde::Serialize>(
    kind: ArtifactKind,
    schema_version: u32,
    artifact: &T,
) -> rust_electroanalysis_cli::domain::ArtifactLineageState {
    known_lineage_with_family(kind, schema_version, artifact, "a1-final-family")
}

fn known_lineage_with_family<T: serde::Serialize>(
    kind: ArtifactKind,
    schema_version: u32,
    artifact: &T,
    family: &str,
) -> rust_electroanalysis_cli::domain::ArtifactLineageState {
    known_lineage_from_artifact(
        kind,
        schema_version,
        "a1-final-test",
        ArtifactExperimentScope::single(ExperimentId::new("xlsx-test").unwrap()).unwrap(),
        ScopeKey::specific("s1").unwrap(),
        ScopeKey::specific("E/V").unwrap(),
        ArtifactAcquisitionFamilies::known([AcquisitionFamilyId::new(family).unwrap()]).unwrap(),
        Vec::new(),
        artifact,
    )
    .unwrap()
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn tracked_transient(path: &Path) -> TransientAnalysisReport {
    let mut report: TransientAnalysisReport = read_artifact(&fixture_path(
        "tests/fixtures/a0_artifact_contracts/schema2/transient_analysis.schema2.json",
    ))
    .unwrap();
    report.schema_version = TransientAnalysisReport::CURRENT_SCHEMA_VERSION;
    report.lineage = known_lineage(
        ArtifactKind::TransientAnalysis,
        report.schema_version,
        &report,
    );
    write_artifact(path, &report).unwrap();
    read_artifact(path).unwrap()
}

fn tracked_model(path: &Path) -> StoredCalibrationModel {
    let mut model = simulation::simulation_model();
    model.schema_version = StoredCalibrationModel::CURRENT_SCHEMA_VERSION;
    model.lineage = known_lineage_with_family(
        ArtifactKind::CalibrationModel,
        model.schema_version,
        &model,
        "a1-calibration-family",
    );
    write_artifact(path, &model).unwrap();
    read_artifact(path).unwrap()
}

/// Create a current calibration report through the production observation and
/// fitting paths, then cross the public artifact writer/reader boundary.
fn current_known_calibration(path: &Path) -> CalibrationAnalysisReport {
    let mut config = ResolvedCalibrationConfig::default();
    config.analyte.name = "Na+".into();
    config.observation_extraction.preferred_source =
        CalibrationPotentialSource::SteadyStateWindowMean;
    config.observation_extraction.steady_state_start_s = 0.0;
    config.observation_extraction.steady_state_end_s = 4.0;
    config.observation_extraction.minimum_points = 2;
    config.uncertainty.bootstrap_iterations = 0;

    let measurement = rust_electroanalysis_cli::domain::MultiChannelMeasurement::new(
        (0..=24).map(f64::from).collect(),
        vec![rust_electroanalysis_cli::domain::MeasurementChannel::new(
            "E1",
            "V",
            (0..=24)
                .map(|time| {
                    Some(if time <= 4 {
                        0.10
                    } else if time <= 14 {
                        0.16
                    } else {
                        0.22
                    })
                })
                .collect(),
        )],
    )
    .unwrap();
    let experiment = rust_electroanalysis_cli::domain::ElectrochemicalExperiment::new(
        "current-known-calibration",
        Default::default(),
        None,
        measurement,
        Vec::new(),
        [(0.0, 0.001), (10.0, 0.01), (20.0, 0.1)]
            .into_iter()
            .map(
                |(timestamp, value)| rust_electroanalysis_cli::domain::ExperimentEvent {
                    timestamp,
                    kind: rust_electroanalysis_cli::domain::ExperimentEventKind::ConcentrationStep,
                    value: Some(value),
                    unit: Some("mol/L".into()),
                    analyte: Some("Na+".into()),
                    annotation: None,
                    metadata: None,
                },
            )
            .collect(),
        "buffer",
        AnalysisProvenance {
            software_version: "test".into(),
            input_path: "current-known-calibration.csv".into(),
            input_sha256: "test".into(),
            configuration_path: None,
            configuration_sha256: None,
            generation_timestamp: 1,
            git_commit: None,
        },
    )
    .unwrap();
    let observations = extract_observations(&experiment, "E1", None, &config).unwrap();
    let report = fit_calibration(&observations, &config).unwrap();
    assert!(matches!(
        report.lineage,
        rust_electroanalysis_cli::domain::ArtifactLineageState::Known { .. }
    ));
    write_artifact(path, &report).unwrap();
    read_artifact(path).unwrap()
}

fn estimation_config(
    root: &Path,
    tau_source: &str,
    transient_parameter: &str,
    noise_source: &str,
) -> PathBuf {
    let base = fs::read_to_string(fixture_path("config/estimation.toml")).unwrap();
    let config = base
        .replace("minimum_segment_points = 10", "minimum_segment_points = 2")
        .replace(
            "enabled = true\ninclude_state_uncertainty",
            "enabled = false\ninclude_state_uncertainty",
        )
        .replace(
            "reject_unobservable_model = true",
            "reject_unobservable_model = false",
        )
        .replace(
            "tau_source = \"transient\"",
            &format!("tau_source = \"{tau_source}\""),
        )
        .replace(
            "transient_parameter = \"tau_slow\"",
            &format!("transient_parameter = \"{transient_parameter}\""),
        )
        .replace(
            "source = \"signal_robust_variance\"",
            &format!("source = \"{noise_source}\""),
        );
    let path = root.join(format!(
        "estimation_{tau_source}_{transient_parameter}_{noise_source}.toml"
    ));
    fs::write(&path, config).unwrap();
    path
}

fn write_metadata(root: &Path) -> PathBuf {
    let path = root.join("metadata.toml");
    fs::write(
        &path,
        "experiment_id = 'xlsx-test'\nsample_matrix = 'buffer'\n\n[sensor]\nsensor_id = 's1'\n",
    )
    .unwrap();
    path
}

fn run_estimate(
    label: &str,
    tau_source: &str,
    transient_parameter: &str,
    noise_source: &str,
    transient: Option<&Path>,
    model_known: bool,
    calibration_results: Option<&Path>,
    mechanism: Option<&Path>,
    health_baseline: Option<&Path>,
    health_assessment: Option<&Path>,
) -> (PathBuf, StateEstimationReport) {
    run_estimate_with_polarization(
        label,
        tau_source,
        transient_parameter,
        noise_source,
        30.0,
        0.0,
        transient,
        model_known,
        calibration_results,
        mechanism,
        health_baseline,
        health_assessment,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_estimate_with_polarization(
    label: &str,
    tau_source: &str,
    transient_parameter: &str,
    noise_source: &str,
    configured_tau_s: f64,
    initial_polarization_v: f64,
    transient: Option<&Path>,
    model_known: bool,
    calibration_results: Option<&Path>,
    mechanism: Option<&Path>,
    health_baseline: Option<&Path>,
    health_assessment: Option<&Path>,
) -> (PathBuf, StateEstimationReport) {
    let root = temp_workspace(label);
    let metadata = write_metadata(&root);
    let config = estimation_config(&root, tau_source, transient_parameter, noise_source);
    let config_text = fs::read_to_string(&config)
        .unwrap()
        .replace(
            "configured_tau_s = 30.0",
            &format!("configured_tau_s = {configured_tau_s}"),
        )
        .replace(
            "polarization_v = 0.0",
            &format!("polarization_v = {initial_polarization_v}"),
        );
    fs::write(&config, config_text).unwrap();
    let model_path = root.join("calibration_model.json");
    if model_known {
        tracked_model(&model_path);
    } else {
        write_artifact(&model_path, &simulation::simulation_model()).unwrap();
    }
    let output = root.join("estimation");
    estimation_runner::run(
        &root,
        RunOptions {
            input: fixture_path("tests/fixtures/xlsx/single_timeseries.xlsx"),
            metadata,
            channel: "E/V".into(),
            sheet: Some("measurement".into()),
            calibration_model: model_path,
            signal_results: None,
            transient_results: transient.map(Path::to_path_buf),
            calibration_results: calibration_results.map(Path::to_path_buf),
            eis_fit: None,
            mechanism_results: mechanism.map(Path::to_path_buf),
            health_baseline: health_baseline.map(Path::to_path_buf),
            health_assessment: health_assessment.map(Path::to_path_buf),
            config: Some(config),
            output: Some(output.clone()),
            filter: Some("ukf".into()),
            model: None,
            seed: Some(42),
        },
    )
    .unwrap();
    let report: StateEstimationReport =
        read_artifact(&output.join("state_estimation.json")).unwrap();
    (root, report)
}

fn dependency_entries(
    report: &StateEstimationReport,
) -> Vec<&rust_electroanalysis_cli::domain::ArtifactDependency> {
    match &report.lineage {
        rust_electroanalysis_cli::domain::ArtifactLineageState::Known {
            direct_dependencies,
            ..
        } => direct_dependencies.iter().collect(),
        _ => panic!("runner must write a known estimation lineage"),
    }
}

fn produced_eis_artifact() -> EisFitArtifact {
    let input = EISData::parse_file(&fixture_path(
        "tests/fixtures/eis/randles_cpe_weighted_fit.csv",
    ))
    .unwrap();
    let circuit = "R0-Wo1-Ws1";
    let node = parse_circuit_string(circuit).unwrap();
    let names = node.get_param_names();
    let units = node.get_param_units();
    let parameter_count = names.len();
    let mut covariance = vec![vec![0.0; parameter_count]; parameter_count];
    for (index, row) in covariance.iter_mut().enumerate() {
        row[index] = 0.1;
    }
    covariance[2][4] = 0.02;
    covariance[4][2] = 0.02;
    let fit = CircuitFitResult {
        fitted_parameters: vec![10.0, 1.0, 1.0, 2.0, 1.5],
        parameter_names: names,
        parameter_units: units,
        fitted_z_re: input.z_re.clone(),
        fitted_z_im: input.z_im.clone(),
        fitted_magnitude: input.derived_magnitude.clone(),
        fitted_phase: input.derived_phase.clone(),
    };
    EisFitArtifact::from_detailed_fit(
        &input,
        circuit,
        &fit,
        Some(covariance),
        Some(1.0),
        Some(parameter_count),
        AnalysisProvenance {
            software_version: "test".into(),
            input_path: "eis.csv".into(),
            input_sha256: "test".into(),
            configuration_path: None,
            configuration_sha256: None,
            generation_timestamp: 1,
            git_commit: None,
        },
    )
}

#[test]
fn a1_fr001_t01_transient_used_by_estimation_is_persisted_as_dependency() {
    let root = temp_workspace("t01-input");
    let transient_path = root.join("transient.json");
    let transient = tracked_transient(&transient_path);
    let (workspace, report) = run_estimate(
        "t01",
        "transient",
        "tau",
        "configured",
        Some(&transient_path),
        false,
        None,
        None,
        None,
        None,
    );
    let dependencies = dependency_entries(&report);
    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].artifact_id,
        match transient.lineage {
            rust_electroanalysis_cli::domain::ArtifactLineageState::Known { identity, .. } =>
                identity.artifact_id,
            _ => panic!(),
        }
    );
    assert_eq!(
        dependencies[0].artifact_kind,
        ArtifactKind::TransientAnalysis
    );
    assert_eq!(dependencies[0].role, ArtifactDependencyRole::Initialization);
    assert!(workspace.join("estimation/state_estimation.json").is_file());
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn a1_fr001_t02_transient_fallback_does_not_persist_dependency() {
    const FALLBACK_TAU_S: f64 = 43.25;
    let root = temp_workspace("t02-input");
    let transient_path = root.join("transient.json");
    tracked_transient(&transient_path);
    let (workspace, report) = run_estimate_with_polarization(
        "t02",
        "transient",
        "tau_slow",
        "configured",
        FALLBACK_TAU_S,
        0.071,
        Some(&transient_path),
        false,
        None,
        None,
        None,
        None,
    );
    assert!(
        dependency_entries(&report)
            .iter()
            .all(|d| d.artifact_kind != ArtifactKind::TransientAnalysis)
    );
    let first = &report.estimates[0];
    let second = &report.estimates[1];
    let first_polarization = first
        .filtered_state
        .iter()
        .find(|state| state.name == "polarization")
        .and_then(|state| state.value)
        .unwrap();
    let second_predicted_polarization = second
        .predicted_state
        .iter()
        .find(|state| state.name == "polarization")
        .and_then(|state| state.value)
        .unwrap();
    assert!(first_polarization.abs() > 1.0e-12);
    let expected =
        first_polarization * (-(second.timestamp_s - first.timestamp_s) / FALLBACK_TAU_S).exp();
    assert!((second_predicted_polarization - expected).abs() < 1.0e-12);
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| format!("{warning:?}").contains("TransientPriorUnavailable"))
    );
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn a1_fr001_t03_transient_excluded_by_configured_tau_source_is_not_persisted() {
    let root = temp_workspace("t03-input");
    let transient_path = root.join("transient.json");
    tracked_transient(&transient_path);
    let (workspace, report) = run_estimate(
        "t03",
        "configured",
        "tau",
        "configured",
        Some(&transient_path),
        false,
        None,
        None,
        None,
        None,
    );
    assert!(
        dependency_entries(&report)
            .iter()
            .all(|d| d.artifact_kind != ArtifactKind::TransientAnalysis)
    );
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn a1_fr001_t04_supplied_mechanism_is_not_an_estimator_dependency() {
    let mechanism = fixture_path(
        "tests/fixtures/a0_artifact_contracts/schema1/mechanism_analysis.schema1.json",
    );
    let (workspace, report) = run_estimate(
        "t04",
        "configured",
        "tau",
        "configured",
        None,
        false,
        None,
        Some(&mechanism),
        None,
        None,
    );
    assert!(
        dependency_entries(&report)
            .iter()
            .all(|d| d.artifact_kind != ArtifactKind::MechanismAnalysis)
    );
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn a1_fr001_t05_supplied_health_baseline_is_not_an_estimator_dependency() {
    let baseline = fixture_path(
        "tests/fixtures/a0_artifact_contracts/health_baseline_schema2_correct_kind.json",
    );
    let (workspace, report) = run_estimate(
        "t05",
        "configured",
        "tau",
        "configured",
        None,
        false,
        None,
        None,
        Some(&baseline),
        None,
    );
    assert!(
        dependency_entries(&report)
            .iter()
            .all(|d| d.artifact_kind != ArtifactKind::HealthBaseline)
    );
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn a1_fr001_t06_supplied_health_assessment_is_not_an_estimator_dependency() {
    let assessment =
        fixture_path("tests/fixtures/a0_artifact_contracts/schema1/health_assessment.schema1.json");
    let (workspace, report) = run_estimate(
        "t06",
        "configured",
        "tau",
        "configured",
        None,
        false,
        None,
        None,
        None,
        Some(&assessment),
    );
    assert!(
        dependency_entries(&report)
            .iter()
            .all(|d| d.artifact_kind != ArtifactKind::HealthAssessment)
    );
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn a1_fr001_t07_calibration_variance_used_by_estimation_is_persisted() {
    let root = temp_workspace("t07-input");
    let path = root.join("calibration_results.json");
    let artifact = current_known_calibration(&path);
    let calibration_id = match artifact.lineage {
        rust_electroanalysis_cli::domain::ArtifactLineageState::Known { ref identity, .. } => {
            identity.artifact_id.clone()
        }
        _ => panic!("current calibration producer must create Known lineage"),
    };
    let (workspace, report) = run_estimate(
        "t07",
        "configured",
        "tau",
        "calibration_residual_variance",
        None,
        false,
        Some(&path),
        None,
        None,
        None,
    );
    let dependencies = dependency_entries(&report);
    assert_eq!(
        dependencies
            .iter()
            .filter(|d| d.artifact_kind == ArtifactKind::CalibrationAnalysis)
            .count(),
        1
    );
    assert!(dependencies.iter().any(|d| d.artifact_id == calibration_id
        && d.artifact_kind == ArtifactKind::CalibrationAnalysis
        && d.role == ArtifactDependencyRole::Calibration));
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn a1_fr001_t08_optional_artifacts_present_but_not_selected_are_not_dependencies() {
    let root = temp_workspace("t08-input");
    let transient_path = root.join("transient.json");
    tracked_transient(&transient_path);
    let calibration_path = root.join("calibration_results.json");
    let calibration = current_known_calibration(&calibration_path);
    let calibration_id = match calibration.lineage {
        rust_electroanalysis_cli::domain::ArtifactLineageState::Known { ref identity, .. } => {
            identity.artifact_id.clone()
        }
        _ => panic!("current calibration producer must create Known lineage"),
    };
    let (workspace, report) = run_estimate(
        "t08",
        "configured",
        "tau",
        "configured",
        Some(&transient_path),
        false,
        Some(&calibration_path),
        None,
        None,
        None,
    );
    assert!(
        dependency_entries(&report)
            .iter()
            .all(|dependency| dependency.artifact_id != calibration_id)
    );
    assert!(
        dependency_entries(&report)
            .iter()
            .all(|dependency| dependency.artifact_kind != ArtifactKind::TransientAnalysis)
    );
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn transient_used_by_estimation_produces_dependent_evidence_in_bundle() {
    let root = temp_workspace("e2e-used-input");
    let transient_path = root.join("transient.json");
    let transient_written = tracked_transient(&transient_path);
    let model_path = root.join("calibration_model.json");
    tracked_model(&model_path);
    let (workspace, estimation) = run_estimate(
        "e2e-used",
        "transient",
        "tau",
        "configured",
        Some(&transient_path),
        true,
        None,
        None,
        None,
        None,
    );
    let transient: TransientAnalysisReport = read_artifact(&transient_path).unwrap();
    let calibration_model: StoredCalibrationModel =
        read_artifact(&workspace.join("calibration_model.json")).unwrap();
    let reread_estimation: StateEstimationReport =
        read_artifact(&workspace.join("estimation/state_estimation.json")).unwrap();
    let bundle = assemble_evidence_bundle(EvidenceBundleInputs {
        calibration_model: Some(calibration_model),
        transient: Some(transient),
        estimation: Some(reread_estimation),
        ..Default::default()
    })
    .unwrap();
    let transient_id = match transient_written.lineage {
        rust_electroanalysis_cli::domain::ArtifactLineageState::Known { identity, .. } => {
            identity.artifact_id
        }
        _ => panic!(),
    };
    let transient_record = bundle
        .records
        .iter()
        .find(|record| {
            matches!(
                &record.source.artifact,
                rust_electroanalysis_cli::evidence::EvidenceArtifactSource::Known {
                    artifact_id,
                    ..
                } if artifact_id == &transient_id
            )
        })
        .unwrap();
    let estimation_record = bundle
        .records
        .iter()
        .find(|record| {
            matches!(
                record.source.artifact,
                rust_electroanalysis_cli::evidence::EvidenceArtifactSource::Known {
                    artifact_kind: ArtifactKind::StateEstimation,
                    ..
                }
            )
        })
        .unwrap();
    let pair = rust_electroanalysis_cli::evidence::EvidencePairKey::canonical(
        transient_record.evidence_id.clone(),
        estimation_record.evidence_id.clone(),
    )
    .unwrap();
    let assessment = bundle.lookup_independence(&pair).unwrap();
    assert!(matches!(
        assessment.classification,
        rust_electroanalysis_cli::evidence::EvidenceIndependence::SameSource
            | rust_electroanalysis_cli::evidence::EvidenceIndependence::PartiallyDependent
    ));
    assert_ne!(
        assessment.classification,
        rust_electroanalysis_cli::evidence::EvidenceIndependence::Independent
    );
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(workspace).ok();
    let _ = estimation;
}

#[test]
fn transient_not_used_by_estimation_does_not_create_false_lineage_dependency() {
    let root = temp_workspace("e2e-unused-input");
    let transient_path = root.join("transient.json");
    tracked_transient(&transient_path);
    let (workspace, estimation) = run_estimate(
        "e2e-unused",
        "configured",
        "tau",
        "configured",
        Some(&transient_path),
        true,
        None,
        None,
        None,
        None,
    );
    assert!(
        dependency_entries(&estimation)
            .iter()
            .all(|d| d.artifact_kind != ArtifactKind::TransientAnalysis)
    );
    let transient: TransientAnalysisReport = read_artifact(&transient_path).unwrap();
    let calibration_model: StoredCalibrationModel =
        read_artifact(&workspace.join("calibration_model.json")).unwrap();
    let reread_estimation: StateEstimationReport =
        read_artifact(&workspace.join("estimation/state_estimation.json")).unwrap();
    let bundle = assemble_evidence_bundle(EvidenceBundleInputs {
        calibration_model: Some(calibration_model),
        transient: Some(transient),
        estimation: Some(reread_estimation),
        ..Default::default()
    })
    .unwrap();
    let transient_record = bundle
        .records
        .iter()
        .find(|record| {
            matches!(
                record.source.artifact,
                rust_electroanalysis_cli::evidence::EvidenceArtifactSource::Known {
                    artifact_kind: ArtifactKind::TransientAnalysis,
                    ..
                }
            )
        })
        .unwrap();
    let estimation_record = bundle
        .records
        .iter()
        .find(|record| {
            matches!(
                record.source.artifact,
                rust_electroanalysis_cli::evidence::EvidenceArtifactSource::Known {
                    artifact_kind: ArtifactKind::StateEstimation,
                    ..
                }
            )
        })
        .unwrap();
    let pair = rust_electroanalysis_cli::evidence::EvidencePairKey::canonical(
        transient_record.evidence_id.clone(),
        estimation_record.evidence_id.clone(),
    )
    .unwrap();
    let assessment = bundle.lookup_independence(&pair).unwrap();
    assert_ne!(
        assessment.classification,
        rust_electroanalysis_cli::evidence::EvidenceIndependence::SameSource
    );
    assert!(assessment.shared_ancestor_artifact_ids.is_empty());
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(workspace).ok();
}

#[test]
fn calibration_observations_transient_supplied_but_unused_excludes_transient_lineage() {
    let mut config = ResolvedCalibrationConfig::default();
    config.observation_extraction.preferred_source =
        CalibrationPotentialSource::SteadyStateWindowMean;
    config.observation_extraction.steady_state_start_s = 0.0;
    config.observation_extraction.steady_state_end_s = 4.0;
    config.observation_extraction.minimum_points = 2;
    let measurement = rust_electroanalysis_cli::domain::MultiChannelMeasurement::new(
        vec![0.0, 1.0, 2.0, 3.0, 4.0],
        vec![rust_electroanalysis_cli::domain::MeasurementChannel::new(
            "E1",
            "V",
            vec![Some(0.2); 5],
        )],
    )
    .unwrap();
    let experiment = rust_electroanalysis_cli::domain::ElectrochemicalExperiment::new(
        "calibration-unused",
        Default::default(),
        None,
        measurement,
        Vec::new(),
        vec![rust_electroanalysis_cli::domain::ExperimentEvent {
            timestamp: 0.0,
            kind: rust_electroanalysis_cli::domain::ExperimentEventKind::ConcentrationStep,
            value: Some(0.001),
            unit: Some("mol/L".into()),
            analyte: Some("Na+".into()),
            annotation: None,
            metadata: None,
        }],
        "buffer",
        AnalysisProvenance {
            software_version: "test".into(),
            input_path: "input.csv".into(),
            input_sha256: "test".into(),
            configuration_path: None,
            configuration_sha256: None,
            generation_timestamp: 1,
            git_commit: None,
        },
    )
    .unwrap();
    let root = temp_workspace("calibration-observations");
    let transient_path = root.join("transient.json");
    let transient = tracked_transient(&transient_path);
    let observations = extract_observations(&experiment, "E1", Some(&transient), &config).unwrap();
    assert!(match observations.lineage {
        rust_electroanalysis_cli::domain::ArtifactLineageState::Known {
            ref direct_dependencies,
            ..
        } => direct_dependencies.is_empty(),
        _ => false,
    });
    fs::remove_dir_all(root).ok();
}

#[test]
fn calibration_observations_transient_used_persists_transient_lineage() {
    let mut config = ResolvedCalibrationConfig::default();
    config.observation_extraction.preferred_source =
        CalibrationPotentialSource::TransientEquilibrium;
    let measurement = rust_electroanalysis_cli::domain::MultiChannelMeasurement::new(
        vec![0.0, 1.0, 2.0, 3.0, 4.0],
        vec![rust_electroanalysis_cli::domain::MeasurementChannel::new(
            "E1",
            "V",
            vec![Some(0.2); 5],
        )],
    )
    .unwrap();
    let experiment = rust_electroanalysis_cli::domain::ElectrochemicalExperiment::new(
        "calibration-used",
        Default::default(),
        None,
        measurement,
        Vec::new(),
        vec![rust_electroanalysis_cli::domain::ExperimentEvent {
            timestamp: 0.0,
            kind: rust_electroanalysis_cli::domain::ExperimentEventKind::ConcentrationStep,
            value: Some(0.001),
            unit: Some("mol/L".into()),
            analyte: Some("Na+".into()),
            annotation: None,
            metadata: None,
        }],
        "buffer",
        AnalysisProvenance {
            software_version: "test".into(),
            input_path: "input.csv".into(),
            input_sha256: "test".into(),
            configuration_path: None,
            configuration_sha256: None,
            generation_timestamp: 1,
            git_commit: None,
        },
    )
    .unwrap();
    let root = temp_workspace("calibration-observations-used");
    let transient_path = root.join("transient.json");
    let transient = tracked_transient(&transient_path);
    let observations = extract_observations(&experiment, "E1", Some(&transient), &config).unwrap();
    let dependencies = match observations.lineage {
        rust_electroanalysis_cli::domain::ArtifactLineageState::Known {
            direct_dependencies,
            ..
        } => direct_dependencies,
        _ => panic!("calibration observations must have known lineage"),
    };
    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].artifact_kind,
        ArtifactKind::TransientAnalysis
    );
    assert_eq!(
        dependencies[0].role,
        ArtifactDependencyRole::TransformationInput
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn eis_producer_serialization_and_bundle_preserve_labeled_covariance_axes() {
    let artifact = produced_eis_artifact();
    assert!(artifact.statistics.labeled_parameter_covariance.is_some());
    let root = temp_workspace("eis-bundle");
    let path = root.join("eis.json");
    write_artifact(&path, &artifact).unwrap();
    let reread: EisFitArtifact = read_artifact(&path).unwrap();
    let expected_axes = reread
        .statistics
        .labeled_parameter_covariance
        .as_ref()
        .unwrap()
        .axes
        .iter()
        .map(|axis| axis.axis_id.clone())
        .collect::<Vec<_>>();
    let bundle = assemble_evidence_bundle(EvidenceBundleInputs {
        eis_fit: Some(reread.clone()),
        ..Default::default()
    })
    .unwrap();
    assert!(bundle.timescale_pair_uncertainties.iter().all(|pair| {
        pair.source
            .covariance_source_field_path
            .contains("labeled_parameter_covariance")
    }));
    assert!(
        expected_axes
            .iter()
            .all(|axis| axis.0.starts_with("eis.parameter:"))
    );
    assert!(!bundle.timescale_pair_uncertainties.is_empty());
    fs::remove_dir_all(root).ok();
}

#[test]
fn legacy_eis_unlabeled_covariance_cannot_create_timescale_pair_uncertainty() {
    let mut artifact = produced_eis_artifact();
    artifact.statistics.labeled_parameter_covariance = None;
    artifact.schema_version = 2;
    artifact.lineage = rust_electroanalysis_cli::domain::legacy_unknown_lineage();
    let root = temp_workspace("eis-legacy");
    let path = root.join("legacy-eis.json");
    fs::write(&path, serde_json::to_string(&artifact).unwrap()).unwrap();
    let reread: EisFitArtifact = read_artifact(&path).unwrap();
    let bundle = assemble_evidence_bundle(EvidenceBundleInputs {
        eis_fit: Some(reread),
        ..Default::default()
    })
    .unwrap();
    assert!(bundle.timescale_pair_uncertainties.is_empty());
    fs::remove_dir_all(root).ok();
}
