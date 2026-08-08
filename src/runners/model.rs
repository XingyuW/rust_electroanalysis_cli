//! User-facing outer workflow for validated ISM model definitions.

use crate::{
    model::{
        AssessmentStatus, EquilibriumAssessment, EquilibriumStatus, ModelInput, ModelState,
        UnexplainedResidual, built_in_registry, compile_model, default_model_definition,
    },
    model_config::ModelConfig,
    results::{
        EisFitArtifact, MODEL_ANALYSIS_ARTIFACT_KIND, MODEL_RESULT_SCHEMA_VERSION,
        MechanismAnalysisReport, ModelAnalysisPoint, ModelAnalysisReport, ModelCompilationArtifact,
        SensorHealthAssessment, SignalAnalysisReport, StoredCalibrationModel,
        TransientAnalysisReport,
    },
    runners::RunnerError,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub fn validate(
    workspace: &Path,
    model_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let (config, path) = load_config(workspace, model_path)?;
    let compiled = compile_model(config.model, built_in_registry())
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let artifact = ModelCompilationArtifact::from_compiled(&compiled);
    let directory = output_directory(workspace, output);
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("model_definition_resolved.json"),
        serde_json::to_string_pretty(&artifact.model_definition)?,
    )?;
    artifact
        .to_json()
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    crate::domain::write_artifact(&directory.join("model_compilation.json"), &artifact)
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    fs::write(
        directory.join("model_validity.csv"),
        "is_valid,checked_domain,warnings\ntrue,definition_compiled,structural and practical identifiability not assessed\n",
    )?;
    fs::write(
        directory.join("model_evidence.json"),
        serde_json::to_string_pretty(&artifact.identifiability)?,
    )?;
    eprintln!("validated model configuration {}", path.display());
    Ok(())
}

pub fn simulate(
    workspace: &Path,
    model_path: Option<&Path>,
    output: Option<&Path>,
    steps: usize,
    dt_s: f64,
) -> Result<(), RunnerError> {
    if steps == 0 || !dt_s.is_finite() || dt_s <= 0.0 {
        return Err(RunnerError::Message(
            "--steps must be positive and --dt-s must be finite and positive".into(),
        ));
    }
    let (config, _) = load_config(workspace, model_path)?;
    let compiled = compile_model(config.model, built_in_registry())
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let parameters = compiled.default_parameters();
    let mut state = compiled
        .initialize(&parameters)
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let mut points = Vec::with_capacity(steps);
    for index in 0..steps {
        let input = default_input(index as f64 * dt_s);
        points.push(evaluate(&compiled, &state, &parameters, &input, None)?);
        state = compiled
            .process_transition(&state, &parameters, &input, dt_s)
            .map_err(|error| RunnerError::Message(error.to_string()))?;
    }
    export(
        workspace,
        output,
        ModelAnalysisReport {
            schema_version: MODEL_RESULT_SCHEMA_VERSION,
            artifact_kind: MODEL_ANALYSIS_ARTIFACT_KIND.into(),
            model_definition: compiled.definition().clone(),
            points,
            identifiability: compiled.identifiability_report(),
            evidence: vec![
                "Deterministic synthetic scenario; values are not fitted physical evidence.".into(),
            ],
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn decompose(
    workspace: &Path,
    model_path: Option<&Path>,
    input_path: Option<&Path>,
    measurement: Option<&Path>,
    metadata: Option<&Path>,
    calibration: Option<&Path>,
    transient: Option<&Path>,
    eis: Option<&Path>,
    signal: Option<&Path>,
    mechanism: Option<&Path>,
    health: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let (config, _) = load_config(workspace, model_path)?;
    let compiled = compile_model(config.model, built_in_registry())
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let mut parameters = compiled.default_parameters();
    let mut evidence = Vec::new();
    if let Some(path) = calibration {
        let artifact: StoredCalibrationModel = read_cross_workflow_artifact(workspace, path)?;
        apply_calibration_parameters(&compiled, &mut parameters, &artifact)?;
        evidence.push(format!(
            "Calibration model for '{}' supplied explicit equilibrium parameter values.",
            artifact.analyte
        ));
    }
    validate_optional_artifact::<TransientAnalysisReport>(workspace, transient, &mut evidence)?;
    validate_optional_artifact::<EisFitArtifact>(workspace, eis, &mut evidence)?;
    validate_optional_artifact::<SignalAnalysisReport>(workspace, signal, &mut evidence)?;
    validate_optional_artifact::<MechanismAnalysisReport>(workspace, mechanism, &mut evidence)?;
    validate_optional_artifact::<SensorHealthAssessment>(workspace, health, &mut evidence)?;
    let mut state = compiled
        .initialize(&parameters)
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let mut inputs = if let Some(path) = input_path {
        let text = fs::read_to_string(resolve(workspace, path))?;
        serde_json::from_str::<Vec<ModelInput>>(&text)
            .or_else(|_| serde_json::from_str::<ModelInput>(&text).map(|value| vec![value]))?
    } else {
        if measurement.is_some() || metadata.is_some() {
            return Err(RunnerError::Message(
                "measurement decomposition requires --input with explicit scientific model inputs; concentration/activity is never inferred from voltage alone".into(),
            ));
        }
        vec![default_input(0.0)]
    };
    attach_measurements(workspace, measurement, metadata, &mut inputs)?;
    let mut points = Vec::with_capacity(inputs.len());
    for (index, input) in inputs.iter().enumerate() {
        if index > 0 {
            let previous = &inputs[index - 1];
            let dt_s = input.time_s - previous.time_s;
            if !dt_s.is_finite() || dt_s < 0.0 {
                return Err(RunnerError::Message(
                    "model inputs must have finite, nondecreasing timestamps".into(),
                ));
            }
            state = compiled
                .process_transition(&state, &parameters, previous, dt_s)
                .map_err(|error| RunnerError::Message(error.to_string()))?;
        }
        let observed = input
            .values
            .get("observed_voltage_v")
            .map(|value| value.value);
        points.push(evaluate(&compiled, &state, &parameters, input, observed)?);
    }
    evidence.push("Optional artifacts were schema-validated and retained as evidence; no physical mechanism identity was inferred.".into());
    export(
        workspace,
        output,
        ModelAnalysisReport {
            schema_version: MODEL_RESULT_SCHEMA_VERSION,
            artifact_kind: MODEL_ANALYSIS_ARTIFACT_KIND.into(),
            model_definition: compiled.definition().clone(),
            points,
            identifiability: compiled.identifiability_report(),
            evidence,
        },
    )
}

pub fn report(workspace: &Path, results: &Path, output: Option<&Path>) -> Result<(), RunnerError> {
    let report: ModelAnalysisReport = crate::domain::read_artifact(&resolve(workspace, results))
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let text = format!(
        "ISM Model Analysis Report\nmodel: {}\npoints: {}\nstructural identifiability: {:?}\n\nEvidence\n{}\n",
        report.model_definition.model_id,
        report.points.len(),
        report.identifiability.structural,
        report.evidence.join("\n")
    );
    let path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("output/model_report.txt"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)?;
    Ok(())
}

fn load_config(
    workspace: &Path,
    path: Option<&Path>,
) -> Result<(ModelConfig, PathBuf), RunnerError> {
    let path = path
        .map(|path| resolve(workspace, path))
        .unwrap_or_else(|| workspace.join("config/model.toml"));
    let config = if path.exists() {
        ModelConfig::load(&path).map_err(|error| RunnerError::Message(error.to_string()))?
    } else {
        ModelConfig {
            schema_version: 1,
            model: default_model_definition(),
        }
    };
    Ok((config, path))
}

fn resolve(workspace: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    }
}

fn read_cross_workflow_artifact<T: crate::domain::VersionedArtifact>(
    workspace: &Path,
    path: &Path,
) -> Result<T, RunnerError> {
    let path = resolve(workspace, path);
    crate::domain::read_artifact(&path).map_err(|error| RunnerError::Message(error.to_string()))
}

fn validate_optional_artifact<T: crate::domain::VersionedArtifact>(
    workspace: &Path,
    path: Option<&Path>,
    evidence: &mut Vec<String>,
) -> Result<(), RunnerError> {
    if let Some(path) = path {
        let _: T = read_cross_workflow_artifact(workspace, path)?;
        evidence.push(format!("Validated optional {} artifact.", T::ARTIFACT_KIND));
    }
    Ok(())
}

fn apply_calibration_parameters(
    compiled: &crate::model::CompiledIsmModel,
    parameters: &mut crate::model::ParameterValues,
    calibration: &StoredCalibrationModel,
) -> Result<(), RunnerError> {
    if let Some(index) = compiled.parameter_index("standard_potential_v")
        && let Some(value) = calibration
            .parameters
            .iter()
            .find(|parameter| parameter.name == "E0" && parameter.unit == "V")
            .map(|parameter| parameter.value)
    {
        parameters.values[index] = value;
    }
    if let Some(index) = compiled.parameter_index("ion_charge") {
        parameters.values[index] = f64::from(calibration.ion_charge);
    }
    compiled
        .validate_parameters(parameters)
        .map_err(|error| RunnerError::Message(error.to_string()))
}

fn attach_measurements(
    workspace: &Path,
    measurement: Option<&Path>,
    metadata: Option<&Path>,
    inputs: &mut [ModelInput],
) -> Result<(), RunnerError> {
    let Some(measurement) = measurement else {
        if metadata.is_some() {
            return Err(RunnerError::Message(
                "--metadata requires --measurement".into(),
            ));
        }
        return Ok(());
    };
    let metadata =
        metadata.ok_or_else(|| RunnerError::Message("--measurement requires --metadata".into()))?;
    let (experiment, _) = crate::data_file::measurement_parser::load_experiment_with_sheet(
        resolve(workspace, measurement),
        resolve(workspace, metadata),
        None,
    )?;
    let measured = experiment.measurement();
    if measured.time.len() != inputs.len() {
        return Err(RunnerError::Message(format!(
            "measurement has {} rows but model input has {} rows",
            measured.time.len(),
            inputs.len()
        )));
    }
    let channel = measured
        .channels
        .iter()
        .find(|channel| {
            channel
                .unit
                .parse::<crate::potentiometry::units::QuantityUnit>()
                .is_ok_and(|unit| {
                    unit.dimension() == crate::potentiometry::units::QuantityDimension::Potential
                })
        })
        .ok_or_else(|| RunnerError::Message("measurement has no potential channel".into()))?;
    for (index, input) in inputs.iter_mut().enumerate() {
        if (input.time_s - measured.time[index]).abs() > 1e-9 {
            return Err(RunnerError::Message(format!(
                "measurement and model-input timestamps differ at row {index}"
            )));
        }
        if let Some(value) = channel.values[index] {
            let voltage = crate::potentiometry::units::Quantity::parse(value, &channel.unit)
                .and_then(|quantity| quantity.to_potential_v())
                .map_err(|error| RunnerError::Message(error.to_string()))?;
            input.values.insert(
                "observed_voltage_v".into(),
                crate::model::InputValue {
                    value: voltage,
                    unit: "V".into(),
                },
            );
        }
    }
    Ok(())
}
fn output_directory(workspace: &Path, output: Option<&Path>) -> PathBuf {
    output
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("output/model"))
}
fn default_input(time_s: f64) -> ModelInput {
    let mut values = BTreeMap::new();
    values.insert(
        "primary_concentration".into(),
        crate::model::InputValue {
            value: 1e-3,
            unit: "mol/L".into(),
        },
    );
    values.insert(
        "temperature".into(),
        crate::model::InputValue {
            value: 298.15,
            unit: "K".into(),
        },
    );
    values.insert(
        "driving_step_v".into(),
        crate::model::InputValue {
            value: 0.01,
            unit: "V".into(),
        },
    );
    ModelInput { time_s, values }
}
fn evaluate(
    compiled: &crate::model::CompiledIsmModel,
    state: &ModelState,
    parameters: &crate::model::ParameterValues,
    input: &ModelInput,
    observed: Option<f64>,
) -> Result<ModelAnalysisPoint, RunnerError> {
    let prediction = compiled
        .observation_prediction(state, parameters, input, observed)
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let residual = match prediction.unexplained_residual {
        UnexplainedResidual::Observed(value) => Some(value),
        UnexplainedResidual::MissingObservedVoltage => None,
    };
    Ok(ModelAnalysisPoint { time_s: input.time_s, observed_voltage_v: observed, predicted_voltage_v: prediction.predicted_voltage_v, uncertainty: prediction.uncertainty, state_values: compiled.state_definitions().iter().zip(&state.values).map(|(spec, value)| (spec.spec.id.clone(), *value)).collect(), contributions: prediction.contributions, equilibrium: EquilibriumAssessment { status: AssessmentStatus::Indeterminate, classification: EquilibriumStatus::Indeterminate, supporting_evidence: Vec::new(), contradictory_evidence: vec!["Equilibrium recognition requires estimator innovation and environmental evidence.".into()], missing_evidence: vec!["dynamic-state derivative and innovation evidence unavailable in deterministic workflow".into()], validity_domain: compiled.definition().validity_domain.clone() }, validity: compiled.validity_report(state, parameters, input), unexplained_residual_v: residual })
}
fn export(
    workspace: &Path,
    output: Option<&Path>,
    report: ModelAnalysisReport,
) -> Result<(), RunnerError> {
    let directory = output_directory(workspace, output);
    fs::create_dir_all(&directory)?;
    report
        .to_json()
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    crate::domain::write_artifact(&directory.join("model_analysis.json"), &report)
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    fs::write(
        directory.join("model_definition_resolved.json"),
        serde_json::to_string_pretty(&report.model_definition)?,
    )?;
    let mut states = csv::Writer::from_path(directory.join("model_states.csv"))?;
    states.write_record(["time_s", "state_id", "value"])?;
    let mut contributions = csv::Writer::from_path(directory.join("model_contributions.csv"))?;
    contributions.write_record([
        "time_s",
        "component_id",
        "owner",
        "potential_v",
        "variance_v2",
    ])?;
    let mut equilibrium = csv::Writer::from_path(directory.join("model_equilibrium.csv"))?;
    equilibrium.write_record(["time_s", "status"])?;
    let mut validity = csv::Writer::from_path(directory.join("model_validity.csv"))?;
    validity.write_record(["time_s", "is_valid", "warnings"])?;
    for point in &report.points {
        for (id, value) in &point.state_values {
            states.write_record([point.time_s.to_string(), id.clone(), value.to_string()])?;
        }
        for contribution in &point.contributions {
            contributions.write_record([
                point.time_s.to_string(),
                contribution.component_id.clone(),
                contribution.owner.clone().unwrap_or_default(),
                contribution
                    .potential_v
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                contribution
                    .variance_v2
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ])?;
        }
        equilibrium.write_record([
            point.time_s.to_string(),
            format!("{:?}", point.equilibrium.status),
        ])?;
        validity.write_record([
            point.time_s.to_string(),
            point.validity.is_valid.to_string(),
            point.validity.warnings.join("; "),
        ])?;
    }
    states.flush()?;
    contributions.flush()?;
    equilibrium.flush()?;
    validity.flush()?;
    fs::write(
        directory.join("model_evidence.json"),
        serde_json::to_string_pretty(&report.evidence)?,
    )?;
    crate::plottings::model_plot::plot_model_analysis(&report, &directory)?;
    report_text(&report, &directory.join("model_report.txt"))
}
fn report_text(report: &ModelAnalysisReport, path: &Path) -> Result<(), RunnerError> {
    fs::write(
        path,
        format!(
            "ISM Model Analysis Report\nmodel: {}\npoints: {}\n",
            report.model_definition.model_id,
            report.points.len()
        ),
    )?;
    Ok(())
}
