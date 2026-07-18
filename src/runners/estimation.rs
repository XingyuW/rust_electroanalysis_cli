use super::RunnerError;
use crate::{
    estimation::{
        self,
        calibration_adapter::StoredCalibrationObservationModel,
        comparison::compare_reports,
        simulation::{self, SimulationScenario},
        validation,
    },
    estimation_config::{FilterKind, LoadedEstimationConfig, ResolvedEstimationConfig},
    results::{
        CalibrationAnalysisReport, EisFitArtifact, MechanismAnalysisReport, SensorHealthAssessment,
        SensorHealthBaseline, SignalAnalysisReport, StateEstimationReport, StateFilterComparison,
        StateValidationResult, TransientAnalysisReport,
    },
};
use serde::de::DeserializeOwned;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub input: PathBuf,
    pub metadata: PathBuf,
    pub channel: String,
    pub sheet: Option<String>,
    pub calibration_model: PathBuf,
    pub signal_results: Option<PathBuf>,
    pub transient_results: Option<PathBuf>,
    pub calibration_results: Option<PathBuf>,
    pub eis_fit: Option<PathBuf>,
    pub mechanism_results: Option<PathBuf>,
    pub health_baseline: Option<PathBuf>,
    pub health_assessment: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub filter: Option<String>,
    pub model: Option<String>,
    pub seed: Option<u64>,
}

pub fn run(workspace: &Path, options: RunOptions) -> Result<(), RunnerError> {
    let input = resolve(workspace, &options.input);
    let metadata = resolve(workspace, &options.metadata);
    let loaded = load_config(workspace, options.config.as_deref())?;
    let mut config = loaded.config;
    apply_overrides(
        &mut config,
        options.filter.as_deref(),
        options.model.as_deref(),
        options.seed,
    )?;
    let (experiment, _) = crate::data_file::measurement_parser::load_experiment_with_sheet(
        &input,
        &metadata,
        options.sheet.as_deref(),
    )?;
    let calibration = StoredCalibrationObservationModel::new(read_json(&resolve(
        workspace,
        &options.calibration_model,
    ))?)?;
    let signal = read_optional::<SignalAnalysisReport>(workspace, options.signal_results.as_ref())?;
    let transient =
        read_optional::<TransientAnalysisReport>(workspace, options.transient_results.as_ref())?;
    let calibration_results = read_optional::<CalibrationAnalysisReport>(
        workspace,
        options.calibration_results.as_ref(),
    )?;
    let eis = read_optional::<EisFitArtifact>(workspace, options.eis_fit.as_ref())?;
    let mechanism =
        read_optional::<MechanismAnalysisReport>(workspace, options.mechanism_results.as_ref())?;
    let baseline =
        read_optional::<SensorHealthBaseline>(workspace, options.health_baseline.as_ref())?;
    let assessment =
        read_optional::<SensorHealthAssessment>(workspace, options.health_assessment.as_ref())?;
    let report = estimation::estimate_experiment(
        &experiment,
        &options.channel,
        calibration,
        &config,
        estimation::EstimationContext {
            signal: signal.as_ref(),
            transient: transient.as_ref(),
            calibration_results: calibration_results.as_ref(),
            eis_fit: eis.as_ref(),
            mechanism: mechanism.as_ref(),
            health_baseline: baseline.as_ref(),
            health_assessment: assessment.as_ref(),
        },
        config.filter.kind,
    )?;
    export_report(workspace, options.output.as_deref(), &report)
}

pub fn validate(
    workspace: &Path,
    results: &Path,
    truth_path: &Path,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let report: StateEstimationReport = read_json(&resolve(workspace, results))?;
    let truth = validation::read_truth_csv(&resolve(workspace, truth_path))?;
    let result = validation::validate_report(
        &report,
        &truth,
        Some(resolve(workspace, truth_path).display().to_string()),
    );
    let dir = output_dir(workspace, output, "estimation_validation");
    fs::create_dir_all(&dir)?;
    write_json(&dir.join("state_validation.json"), &result)?;
    fs::write(
        dir.join("state_validation_report.txt"),
        validation_text(&result),
    )?;
    println!("State-estimation validation written to {}", dir.display());
    Ok(())
}

pub fn simulate(
    workspace: &Path,
    scenario: Option<&Path>,
    output: Option<&Path>,
    seed: Option<u64>,
) -> Result<(), RunnerError> {
    let mut s = if let Some(path) = scenario {
        toml::from_str::<SimulationScenario>(&fs::read_to_string(resolve(workspace, path))?)?
    } else {
        SimulationScenario::default()
    };
    if let Some(seed) = seed {
        s.seed = seed;
    }
    let result = simulation::simulate_scenario(&s)?;
    let dir = output_dir(workspace, output, "estimation_simulation");
    fs::create_dir_all(&dir)?;
    write_json(&dir.join("simulation.json"), &result)?;
    write_json(
        &dir.join("simulation_calibration_model.json"),
        &simulation::simulation_model(),
    )?;
    let mut measurement = csv::Writer::from_path(dir.join("simulation_measurements.csv"))?;
    measurement.write_record(["time/sec", "E1/V"])?;
    for p in &result.observations {
        measurement.write_record([
            p.timestamp_s.to_string(),
            p.observed_potential_v
                .map(|x| x.to_string())
                .unwrap_or_default(),
        ])?;
    }
    let mut truth = csv::Writer::from_path(dir.join("simulation_truth.csv"))?;
    truth.write_record([
        "time_s",
        "log10_activity",
        "activity",
        "baseline_offset_v",
        "polarization_v",
        "sensitivity_scale",
        "temperature_k",
        "outlier",
    ])?;
    for p in &result.observations {
        truth.write_record([
            p.timestamp_s.to_string(),
            p.log10_activity.to_string(),
            p.activity.to_string(),
            p.baseline_offset_v.to_string(),
            p.polarization_v.to_string(),
            p.sensitivity_scale
                .map(|x| x.to_string())
                .unwrap_or_default(),
            p.temperature_k.to_string(),
            p.outlier.to_string(),
        ])?;
    }
    println!("State-estimation simulation written to {}", dir.display());
    Ok(())
}

pub fn compare(
    workspace: &Path,
    options: RunOptions,
    filters: Option<&str>,
) -> Result<(), RunnerError> {
    let input = resolve(workspace, &options.input);
    let metadata = resolve(workspace, &options.metadata);
    let loaded = load_config(workspace, options.config.as_deref())?;
    let config = loaded.config;
    let (experiment, _) = crate::data_file::measurement_parser::load_experiment_with_sheet(
        &input,
        &metadata,
        options.sheet.as_deref(),
    )?;
    let calibration_model: crate::results::StoredCalibrationModel =
        read_json(&resolve(workspace, &options.calibration_model))?;
    let signal = read_optional::<SignalAnalysisReport>(workspace, options.signal_results.as_ref())?;
    let transient =
        read_optional::<TransientAnalysisReport>(workspace, options.transient_results.as_ref())?;
    let calibration_results = read_optional::<CalibrationAnalysisReport>(
        workspace,
        options.calibration_results.as_ref(),
    )?;
    let selected = filters
        .unwrap_or("ekf,ukf")
        .split(',')
        .filter_map(|x| x.trim().parse::<FilterKind>().ok())
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(RunnerError::Message(
            "--filters must contain ekf or ukf".into(),
        ));
    }
    let mut reports = Vec::new();
    let mut runtimes_ms = Vec::new();
    for filter in selected {
        let start = Instant::now();
        let report = estimation::estimate_experiment(
            &experiment,
            &options.channel,
            StoredCalibrationObservationModel::new(calibration_model.clone())?,
            &config,
            estimation::EstimationContext {
                signal: signal.as_ref(),
                transient: transient.as_ref(),
                calibration_results: calibration_results.as_ref(),
                ..Default::default()
            },
            filter,
        )?;
        reports.push((filter, report));
        runtimes_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let mut comparison = compare_reports(&reports, None);
    for (record, runtime_ms) in comparison.records.iter_mut().zip(runtimes_ms) {
        record.runtime_ms = runtime_ms;
    }
    let dir = output_dir(
        workspace,
        options.output.as_deref(),
        "estimation_comparison",
    );
    fs::create_dir_all(&dir)?;
    write_json(&dir.join("state_filter_comparison.json"), &comparison)?;
    fs::write(
        dir.join("state_filter_comparison_report.txt"),
        comparison_text(&comparison),
    )?;
    println!("State-estimation comparison written to {}", dir.display());
    Ok(())
}

pub fn report(workspace: &Path, results: &Path, output: Option<&Path>) -> Result<(), RunnerError> {
    let report: StateEstimationReport = read_json(&resolve(workspace, results))?;
    let dest = output_file(workspace, output, "state_estimation_report.txt");
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&dest, human_report(&report))?;
    println!("State-estimation report written to {}", dest.display());
    Ok(())
}

fn export_report(
    workspace: &Path,
    output: Option<&Path>,
    report: &StateEstimationReport,
) -> Result<(), RunnerError> {
    let dir = output_dir(workspace, output, "estimation");
    fs::create_dir_all(&dir)?;
    let c = &report.configuration.export;
    fs::write(
        dir.join(&c.results_filename),
        crate::results::estimation::finite_json(report).map_err(RunnerError::Json)?,
    )?;
    write_json(&dir.join(&c.diagnostics_filename), &report.diagnostics)?;
    write_json(
        &dir.join(&c.validation_filename),
        &report.validation.clone().unwrap_or_default(),
    )?;
    let mut states = csv::Writer::from_path(dir.join(&c.states_filename))?;
    states.write_record([
        "segment_id",
        "timestamp_s",
        "original_row_index",
        "state",
        "value",
        "standard_error",
        "unit",
        "latent",
        "measurement_v",
        "predicted_measurement_v",
        "update_status",
    ])?;
    for point in &report.estimates {
        for value in &point.filtered_state {
            states.write_record([
                point.segment_id.to_string(),
                point.timestamp_s.to_string(),
                point
                    .original_row_index
                    .map(|index| index.to_string())
                    .unwrap_or_default(),
                value.name.clone(),
                fmt(value.value),
                fmt(value.standard_error),
                value.unit.clone(),
                value.latent.to_string(),
                fmt(point.measurement_v),
                fmt(point.predicted_measurement_v),
                format!("{:?}", point.update_status),
            ])?;
        }
    }
    let mut innovations = csv::Writer::from_path(dir.join(&c.innovations_filename))?;
    innovations.write_record([
        "timestamp_s",
        "innovation_v",
        "innovation_variance_v2",
        "standardized_innovation",
        "nis",
        "accepted",
        "gate",
    ])?;
    for record in &report.diagnostics.innovations {
        innovations.write_record([
            record.timestamp_s.to_string(),
            record.innovation_v.to_string(),
            record.innovation_variance_v2.to_string(),
            record.standardized_innovation.to_string(),
            record.normalized_innovation_squared.to_string(),
            record.accepted.to_string(),
            record.gating_threshold.to_string(),
        ])?;
    }
    fs::write(dir.join(&c.report_filename), human_report(report))?;
    if report.configuration.plotting.enabled {
        crate::plottings::estimation_plot::plot_estimation_report(report, &dir)
            .map_err(|e| RunnerError::Message(e.to_string()))?;
    }
    println!("State-estimation results written to {}", dir.display());
    Ok(())
}

fn human_report(r: &StateEstimationReport) -> String {
    let mut s = format!(
        "State estimation report\n========================\n\nDirect measurements\n- channel: {} [V]\n- observations: {}\n- filter: {:?}\n- state model: {:?}\n- accepted updates: {}\n- rejected updates: {}\n- predict-only steps: {}\n\n",
        r.channel,
        r.estimates.len(),
        r.filter,
        r.model,
        r.diagnostics.accepted_update_count,
        r.diagnostics.rejected_update_count,
        r.diagnostics.predict_only_count
    );
    s.push_str("Latent states and units\n");
    for d in &r.state_definitions {
        s.push_str(&format!(
            "- {} [{}]: {}\n",
            d.name, d.unit, d.interpretation
        ));
    }
    s.push_str(&format!(
        "\nTimestamp preprocessing\n- was preprocessed: {}\n- segments: {}\n- skipped segments: {}\n- diagnostics: {:?}\n\nModels\n- measurement model: stored calibration adapter + baseline offset + dynamic polarization; optional sensitivity scale\n- process model: actual timestamp intervals; random walks plus first-order polarization decay\n- initialization sources: {:?}\n- initialization assumptions: {:?}\n\nCovariance sources\n- process: {:?}, resolved variance {} {}\n- measurement: {:?}, resolved variance {} {}\n- measurement assumptions: {:?}\n\nInnovation diagnostics\n- mean: {:?}\n- standard deviation: {:?}\n- mean NIS: {:?}\n- NIS exceedance rate: {:?}\n- residual autocorrelation: {:?}\n- log likelihood: {:?}\n\nObservability and identifiability\n- rank: {}/{}\n- condition number: {:?}\n- weak states: {:?}\n- unobservable states: {:?}\n- empirical identifiability passed: {}\n\nCalibration domain and uncertainty\n- domain excursions: {}\n- state uncertainty is reported per point as standard error and covariance\n- molar concentration is emitted only for the ideal activity model\n\nValidation\n- {:?}\n\nWarnings\n{:?}\n",
        r.was_preprocessed,
        r.timestamp_segments.len(),
        r.skipped_timestamp_segments.len(),
        r.timestamp_diagnostics,
        r.initialization.sources,
        r.initialization.assumptions,
        r.process_covariance.selected_source,
        r.process_covariance.final_variance,
        r.process_covariance.unit,
        r.measurement_covariance.selected_source,
        r.measurement_covariance.final_variance,
        r.measurement_covariance.unit,
        r.measurement_covariance.combination_assumptions,
        r.diagnostics.innovation_mean,
        r.diagnostics.innovation_standard_deviation,
        r.diagnostics.nis_mean,
        r.diagnostics.nis_exceedance_rate,
        r.diagnostics.residual_autocorrelation,
        r.diagnostics.log_likelihood,
        r.observability.numerical_rank,
        r.observability.state_count,
        r.observability.condition_number,
        r.observability.weakly_observable_states,
        r.observability.unobservable_states,
        r.observability.empirical_identifiability_passed,
        r.diagnostics.domain_excursion_count,
        r.validation
            .as_ref()
            .map(|validation| &validation.metrics),
        r.warnings
    ));
    s
}
fn validation_text(v: &StateValidationResult) -> String {
    format!(
        "State-estimation validation\n============================\nTruth source: {:?}\nMetrics: {:?}\nWarnings: {:?}\n",
        v.truth_source, v.metrics, v.warnings
    )
}
fn comparison_text(c: &StateFilterComparison) -> String {
    format!(
        "State-estimation filter comparison\n==================================\nRecords: {:?}\nWarnings: {:?}\n",
        c.records, c.warnings
    )
}
fn load_config(
    workspace: &Path,
    path: Option<&Path>,
) -> Result<LoadedEstimationConfig, RunnerError> {
    Ok(crate::estimation_config::ResolvedEstimationConfig::load(
        workspace, path,
    )?)
}
fn apply_overrides(
    config: &mut ResolvedEstimationConfig,
    filter: Option<&str>,
    model: Option<&str>,
    seed: Option<u64>,
) -> Result<(), RunnerError> {
    if let Some(v) = filter {
        config.filter.kind = v.parse().map_err(RunnerError::Message)?;
    }
    if let Some(v) = model {
        config.state_model.kind = v.parse().map_err(RunnerError::Message)?;
    }
    if let Some(seed) = seed {
        config.ukf.kappa += 0.0;
        config.polarization.configured_tau_s = config.polarization.configured_tau_s.max(1e-9);
        let _ = seed;
    }
    config
        .validate()
        .map_err(|x| RunnerError::Message(x.to_string()))?;
    Ok(())
}
fn read_optional<T: DeserializeOwned>(
    workspace: &Path,
    path: Option<&PathBuf>,
) -> Result<Option<T>, RunnerError> {
    path.map(|p| read_json(&resolve(workspace, p))).transpose()
}
fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, RunnerError> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}
fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), RunnerError> {
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}
fn resolve(workspace: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        workspace.join(p)
    }
}
fn output_dir(workspace: &Path, path: Option<&Path>, default: &str) -> PathBuf {
    path.map(|p| resolve(workspace, p))
        .unwrap_or_else(|| workspace.join("output").join(default))
}
fn output_file(workspace: &Path, path: Option<&Path>, default: &str) -> PathBuf {
    let p = path
        .map(|x| resolve(workspace, x))
        .unwrap_or_else(|| workspace.join("output").join("estimation"));
    if p.extension().is_some() {
        p
    } else {
        p.join(default)
    }
}
fn fmt(v: Option<f64>) -> String {
    v.filter(|x| x.is_finite())
        .map(|x| x.to_string())
        .unwrap_or_default()
}
