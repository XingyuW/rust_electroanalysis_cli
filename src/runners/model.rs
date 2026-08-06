//! User-facing outer workflow for validated ISM model definitions.

use crate::{
    model::{
        AssessmentStatus, EquilibriumAssessment, ModelInput, ModelState, UnexplainedResidual,
        built_in_registry, compile_model, default_model_definition,
    },
    model_config::ModelConfig,
    results::{
        MODEL_ANALYSIS_ARTIFACT_KIND, MODEL_RESULT_SCHEMA_VERSION, ModelAnalysisPoint,
        ModelAnalysisReport, ModelCompilationArtifact,
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
        artifact
            .to_json()
            .map_err(|error| RunnerError::Message(error.to_string()))?,
    )?;
    fs::write(
        directory.join("model_validity.csv"),
        "is_valid,checked_domain\ntrue,definition_compiled\n",
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
    _metadata: Option<&Path>,
    _calibration: Option<&Path>,
    _transient: Option<&Path>,
    _eis: Option<&Path>,
    _signal: Option<&Path>,
    _mechanism: Option<&Path>,
    _health: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let (config, _) = load_config(workspace, model_path)?;
    let compiled = compile_model(config.model, built_in_registry())
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let parameters = compiled.default_parameters();
    let state = compiled
        .initialize(&parameters)
        .map_err(|error| RunnerError::Message(error.to_string()))?;
    let inputs = if let Some(path) = input_path {
        let text = fs::read_to_string(path)?;
        serde_json::from_str::<Vec<ModelInput>>(&text)
            .or_else(|_| serde_json::from_str::<ModelInput>(&text).map(|value| vec![value]))?
    } else {
        let mut input = default_input(0.0);
        if measurement.is_some() {
            input.values.insert(
                "observed_voltage_v".into(),
                crate::model::InputValue {
                    value: 0.0,
                    unit: "V".into(),
                },
            );
        }
        vec![input]
    };
    let points = inputs
        .iter()
        .map(|input| {
            let observed = input
                .values
                .get("observed_voltage_v")
                .map(|value| value.value);
            evaluate(&compiled, &state, &parameters, input, observed)
        })
        .collect::<Result<Vec<_>, _>>()?;
    export(workspace, output, ModelAnalysisReport { schema_version: MODEL_RESULT_SCHEMA_VERSION, artifact_kind: MODEL_ANALYSIS_ARTIFACT_KIND.into(), model_definition: compiled.definition().clone(), points, identifiability: compiled.identifiability_report(), evidence: vec!["Optional legacy artifacts were retained as external evidence; no mechanism identity was inferred.".into()] })
}

pub fn report(workspace: &Path, results: &Path, output: Option<&Path>) -> Result<(), RunnerError> {
    let report: ModelAnalysisReport = serde_json::from_str(&fs::read_to_string(results)?)?;
    if report.schema_version != MODEL_RESULT_SCHEMA_VERSION
        || report.artifact_kind != MODEL_ANALYSIS_ARTIFACT_KIND
    {
        return Err(RunnerError::Message(
            "unsupported model-analysis artifact schema or kind".into(),
        ));
    }
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
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("config/model.toml"));
    let mut config = if path.exists() {
        ModelConfig::load(&path).map_err(|error| RunnerError::Message(error.to_string()))?
    } else {
        ModelConfig {
            schema_version: 1,
            model: default_model_definition(),
        }
    };
    // The checked-in config is intentionally a documented template; resolving
    // it uses the versioned reduced-order default rather than an empty model.
    if config.model.components.is_empty() {
        config.model = default_model_definition();
    }
    Ok((config, path))
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
    Ok(ModelAnalysisPoint { time_s: input.time_s, observed_voltage_v: observed, predicted_voltage_v: prediction.predicted_voltage_v, state_values: compiled.state_definitions().iter().zip(&state.values).map(|(spec, value)| (spec.spec.id.clone(), *value)).collect(), contributions: prediction.contributions, equilibrium: EquilibriumAssessment { status: AssessmentStatus::Indeterminate, supporting_evidence: Vec::new(), contradictory_evidence: vec!["Equilibrium recognition requires estimator innovation and environmental evidence.".into()], missing_evidence: vec!["dynamic-state derivative and innovation evidence unavailable in deterministic workflow".into()], validity_domain: compiled.definition().validity_domain.clone() }, validity: compiled.validity_report(state, parameters, input), unexplained_residual_v: residual })
}
fn export(
    workspace: &Path,
    output: Option<&Path>,
    report: ModelAnalysisReport,
) -> Result<(), RunnerError> {
    let directory = output_directory(workspace, output);
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("model_analysis.json"),
        report
            .to_json()
            .map_err(|error| RunnerError::Message(error.to_string()))?,
    )?;
    fs::write(
        directory.join("model_definition_resolved.json"),
        serde_json::to_string_pretty(&report.model_definition)?,
    )?;
    let mut states = csv::Writer::from_path(directory.join("model_states.csv"))?;
    states.write_record(["time_s", "state_id", "value"])?;
    let mut contributions = csv::Writer::from_path(directory.join("model_contributions.csv"))?;
    contributions.write_record(["time_s", "component_id", "owner", "voltage_v"])?;
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
                contribution.owner.clone(),
                contribution.voltage_v.to_string(),
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
