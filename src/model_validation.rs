//! Outer validation-study calculations; this module does not add mechanisms.
use crate::{
    domain::ArtifactError,
    results::{
        MODEL_VALIDATION_ARTIFACT_KIND, MODEL_VALIDATION_SCHEMA_VERSION, ModelAnalysisReport,
        ModelComparisonRow, ValidationManifest, ValidationMetric, ValidationResults,
    },
};
use std::{collections::BTreeSet, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelValidationError {
    #[error("unsupported validation manifest schema {found}; expected {expected}")]
    UnsupportedManifestSchema { found: u32, expected: u32 },
    #[error("validation manifest contains duplicate experiment ID '{0}'")]
    DuplicateExperiment(String),
    #[error("validation experiment '{0}' has an empty identifier, sensor ID, or analysis path")]
    InvalidExperiment(String),
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    #[error("validation result contains a non-finite value for {0}")]
    NonFinite(String),
    #[error("validation JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn evaluate_manifest(
    manifest: &ValidationManifest,
    base: &Path,
) -> Result<ValidationResults, ModelValidationError> {
    if manifest.schema_version != MODEL_VALIDATION_SCHEMA_VERSION {
        return Err(ModelValidationError::UnsupportedManifestSchema {
            found: manifest.schema_version,
            expected: MODEL_VALIDATION_SCHEMA_VERSION,
        });
    }
    let mut ids = BTreeSet::new();
    for experiment in &manifest.experiments {
        if experiment.experiment_id.trim().is_empty()
            || experiment.sensor_id.trim().is_empty()
            || experiment.analysis_path.trim().is_empty()
        {
            return Err(ModelValidationError::InvalidExperiment(
                experiment.experiment_id.clone(),
            ));
        }
        if !ids.insert(experiment.experiment_id.clone()) {
            return Err(ModelValidationError::DuplicateExperiment(
                experiment.experiment_id.clone(),
            ));
        }
    }

    let mut metrics = Vec::new();
    let mut comparison = Vec::new();
    let mut warnings = Vec::new();
    let mut identifiability = Vec::new();
    for experiment in &manifest.experiments {
        let analysis: ModelAnalysisReport =
            crate::domain::read_artifact(&base.join(&experiment.analysis_path))?;
        let summary = analysis_summary(&analysis);
        let status = if experiment.is_real_experiment {
            "real_experiment"
        } else {
            "synthetic_or_unverified"
        };
        metrics.push(metric(
            experiment,
            "prediction_rmse",
            summary.rmse_v,
            "V",
            status,
            synthetic_note(experiment),
        ));
        metrics.push(metric(
            experiment,
            "contribution_reconstruction_error",
            summary.reconstruction_error_v,
            "V",
            status,
            synthetic_note(experiment),
        ));
        metrics.push(metric(
            experiment,
            "prediction_coverage",
            None,
            "fraction",
            "missing_prediction_intervals",
            vec![format!(
                "Coverage requires per-observation prediction intervals; the optional acceptance target was {:?} and is not itself a measured coverage.",
                experiment.expected_prediction_coverage
            )],
        ));

        for (state_id, truth) in &experiment.reference_state_values {
            let recovered = analysis.points.last().and_then(|point| {
                point
                    .state_values
                    .iter()
                    .find(|(id, _)| id == state_id)
                    .map(|(_, value)| *value)
            });
            metrics.push(metric(
                experiment,
                &format!("endpoint_state_recovery_error.{state_id}"),
                recovered.map(|value| (value - truth).abs()),
                analysis
                    .model_definition
                    .states
                    .iter()
                    .find(|state| state.id == *state_id)
                    .map(|state| state.unit.as_str())
                    .unwrap_or("unknown"),
                if recovered.is_some() { status } else { "missing_state" },
                vec!["This is endpoint recovery only; trajectory recovery requires timestamped reference states.".into()],
            ));
        }
        for parameter_id in experiment.reference_parameter_values.keys() {
            metrics.push(metric(
                experiment,
                &format!("parameter_recovery_error.{parameter_id}"),
                None,
                analysis
                    .model_definition
                    .parameters
                    .iter()
                    .find(|parameter| parameter.id == *parameter_id)
                    .map(|parameter| parameter.unit.as_str())
                    .unwrap_or("unknown"),
                "missing_fitted_parameter_estimate",
                vec!["Model-definition defaults are not fitted estimates and are never scored as recovered parameters.".into()],
            ));
        }
        metrics.push(metric(
            experiment,
            "equilibrium_recognition_accuracy",
            None,
            "fraction",
            "missing_reference_labels",
            vec!["Timestamped reference equilibrium labels are required.".into()],
        ));
        metrics.push(metric(
            experiment,
            "calibration_transfer_validation",
            None,
            "fraction",
            if experiment.calibration_transfer_group.is_some() {
                "missing_held_out_transfer_predictions"
            } else {
                "missing_transfer_group"
            },
            vec!["A transfer-group label is not a transfer-accuracy measurement.".into()],
        ));
        comparison.push(comparison_row(&analysis, &summary));
        identifiability.push(serde_json::json!({
            "experiment_id": experiment.experiment_id,
            "model_id": analysis.model_definition.model_id,
            "report": analysis.identifiability,
            "profile_likelihood": {"status": "missing", "reason": "fitted likelihood evaluations were not supplied"}
        }));
        if !experiment.is_real_experiment {
            warnings.push(format!(
                "{} is not marked as a real experiment",
                experiment.experiment_id
            ));
        }
    }

    for path in &manifest.model_comparison_paths {
        let analysis: ModelAnalysisReport = crate::domain::read_artifact(&base.join(path))?;
        let summary = analysis_summary(&analysis);
        comparison.push(comparison_row(&analysis, &summary));
    }

    let sensors = manifest
        .experiments
        .iter()
        .map(|experiment| experiment.sensor_id.as_str())
        .collect::<BTreeSet<_>>();
    metrics.push(ValidationMetric {
        experiment_id: manifest.study_id.clone(),
        metric: "cross_sensor_generalization".into(),
        value: None,
        unit: "fraction".into(),
        evidence_status: if sensors.len() >= 2 {
            "missing_held_out_sensor_predictions".into()
        } else {
            "insufficient_sensors".into()
        },
        notes: vec![format!(
            "{} sensor(s) declared; sensor count is not a generalization-performance metric.",
            sensors.len()
        )],
    });
    Ok(ValidationResults {
        schema_version: MODEL_VALIDATION_SCHEMA_VERSION,
        artifact_kind: MODEL_VALIDATION_ARTIFACT_KIND.into(),
        study_id: manifest.study_id.clone(),
        metrics,
        identifiability_report: serde_json::json!({
            "experiments": identifiability,
            "limitations": ["Profile likelihood requires fitted likelihood evaluations over declared parameter profiles."]
        }),
        model_comparison: comparison,
        warnings,
    })
}

struct AnalysisSummary {
    rmse_v: Option<f64>,
    reconstruction_error_v: Option<f64>,
}

fn analysis_summary(analysis: &ModelAnalysisReport) -> AnalysisSummary {
    let residuals = analysis
        .points
        .iter()
        .filter_map(|point| point.unexplained_residual_v)
        .collect::<Vec<_>>();
    let rmse_v = (!residuals.is_empty()).then(|| {
        (residuals.iter().map(|value| value * value).sum::<f64>() / residuals.len() as f64).sqrt()
    });
    let reconstruction_error_v = (!analysis.points.is_empty()).then(|| {
        analysis
            .points
            .iter()
            .map(|point| {
                (point.predicted_voltage_v
                    - point
                        .contributions
                        .iter()
                        .map(|value| value.voltage_v)
                        .sum::<f64>())
                .abs()
            })
            .fold(0.0, f64::max)
    });
    AnalysisSummary {
        rmse_v,
        reconstruction_error_v,
    }
}

fn metric(
    experiment: &crate::results::ValidationExperiment,
    name: &str,
    value: Option<f64>,
    unit: &str,
    evidence_status: &str,
    notes: Vec<String>,
) -> ValidationMetric {
    ValidationMetric {
        experiment_id: experiment.experiment_id.clone(),
        metric: name.into(),
        value,
        unit: unit.into(),
        evidence_status: evidence_status.into(),
        notes,
    }
}

fn synthetic_note(experiment: &crate::results::ValidationExperiment) -> Vec<String> {
    (!experiment.is_real_experiment)
        .then_some("Synthetic-only evidence does not establish physical validation.".into())
        .into_iter()
        .collect()
}

fn comparison_row(analysis: &ModelAnalysisReport, summary: &AnalysisSummary) -> ModelComparisonRow {
    ModelComparisonRow {
        model_id: analysis.model_definition.model_id.clone(),
        observations: analysis.points.len(),
        rmse_v: summary.rmse_v,
        prediction_coverage: None,
        contribution_reconstruction_error_v: summary.reconstruction_error_v,
        criterion: "Descriptive RMSE on a declared common dataset; likelihood-based criteria require fitted likelihoods and identical observations.".into(),
        limitations: vec![
            "Prediction intervals and fitted likelihood evaluations were not supplied.".into(),
            "Lower RMSE alone is not evidence for a physical mechanism.".into(),
        ],
    }
}
