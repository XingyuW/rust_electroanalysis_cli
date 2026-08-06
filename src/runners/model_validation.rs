//! Reproducible validation-study artifact export.
use crate::{
    model_validation::evaluate_manifest, results::ValidationManifest, runners::RunnerError,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn run(
    workspace: &Path,
    manifest_path: &Path,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        workspace.join(manifest_path)
    };
    let manifest: ValidationManifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    let results = evaluate_manifest(&manifest, manifest_path.parent().unwrap_or(workspace))
        .map_err(RunnerError::Message)?;
    let directory = output
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("output/model_validation"));
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("validation_results.json"),
        results.to_json()?,
    )?;
    fs::write(
        directory.join("identifiability_report.json"),
        serde_json::to_string_pretty(&results.identifiability_report)?,
    )?;
    let mut metrics = csv::Writer::from_path(directory.join("validation_metrics.csv"))?;
    metrics.write_record([
        "experiment_id",
        "metric",
        "value",
        "unit",
        "evidence_status",
    ])?;
    for metric in &results.metrics {
        metrics.write_record([
            metric.experiment_id.clone(),
            metric.metric.clone(),
            metric
                .value
                .map(|value| value.to_string())
                .unwrap_or_default(),
            metric.unit.clone(),
            metric.evidence_status.clone(),
        ])?;
    }
    metrics.flush()?;
    let mut comparison = csv::Writer::from_path(directory.join("model_comparison.csv"))?;
    comparison.write_record([
        "model_id",
        "observations",
        "rmse_v",
        "prediction_coverage",
        "contribution_reconstruction_error_v",
        "criterion",
    ])?;
    for row in &results.model_comparison {
        comparison.write_record([
            row.model_id.clone(),
            row.observations.to_string(),
            row.rmse_v
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.prediction_coverage
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.contribution_reconstruction_error_v
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.criterion.clone(),
        ])?;
    }
    comparison.flush()?;
    fs::write(
        directory.join("validation_report.txt"),
        format!(
            "Model validation study: {}\nExperiments: {}\nWarnings:\n{}\n\nSynthetic data alone do not establish physical validation.\n",
            results.study_id,
            manifest.experiments.len(),
            results.warnings.join("\n")
        ),
    )?;
    Ok(())
}
