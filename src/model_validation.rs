//! Outer validation-study calculations; this module does not add mechanisms.
use crate::results::{
    MODEL_VALIDATION_ARTIFACT_KIND, MODEL_VALIDATION_SCHEMA_VERSION, ModelAnalysisReport,
    ModelComparisonRow, ValidationManifest, ValidationMetric, ValidationResults,
};
use std::{fs, path::Path};

pub fn evaluate_manifest(
    manifest: &ValidationManifest,
    base: &Path,
) -> Result<ValidationResults, String> {
    if manifest.schema_version != MODEL_VALIDATION_SCHEMA_VERSION {
        return Err(format!(
            "unsupported validation manifest schema {}",
            manifest.schema_version
        ));
    }
    let mut metrics = Vec::new();
    let mut comparison = Vec::new();
    let mut warnings = Vec::new();
    for experiment in &manifest.experiments {
        let path = base.join(&experiment.analysis_path);
        let analysis: ModelAnalysisReport =
            serde_json::from_str(&fs::read_to_string(&path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        let residuals = analysis
            .points
            .iter()
            .filter_map(|point| point.unexplained_residual_v)
            .collect::<Vec<_>>();
        let rmse = (!residuals.is_empty()).then(|| {
            (residuals.iter().map(|value| value * value).sum::<f64>() / residuals.len() as f64)
                .sqrt()
        });
        let reconstruction = (!analysis.points.is_empty()).then(|| {
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
        let coverage = experiment.expected_prediction_coverage.map(|_| {
            analysis
                .points
                .iter()
                .filter(|point| point.unexplained_residual_v.is_some())
                .count() as f64
                / analysis.points.len().max(1) as f64
        });
        let status = if experiment.is_real_experiment {
            "real_experiment"
        } else {
            "synthetic_or_unverified"
        };
        for (metric, value, unit) in [
            ("prediction_rmse", rmse, "V"),
            ("contribution_reconstruction_error", reconstruction, "V"),
            ("prediction_coverage", coverage, "fraction"),
        ] {
            metrics.push(ValidationMetric {
                experiment_id: experiment.experiment_id.clone(),
                metric: metric.into(),
                value,
                unit: unit.into(),
                evidence_status: status.into(),
                notes: (!experiment.is_real_experiment)
                    .then_some(
                        "Synthetic-only evidence does not establish physical validation.".into(),
                    )
                    .into_iter()
                    .collect(),
            });
        }
        for (state_id, truth) in &experiment.reference_state_values {
            let recovered = analysis.points.last().and_then(|point| {
                point
                    .state_values
                    .iter()
                    .find(|(id, _)| id == state_id)
                    .map(|(_, value)| *value)
            });
            metrics.push(ValidationMetric {
                experiment_id: experiment.experiment_id.clone(),
                metric: format!("state_recovery_error.{state_id}"),
                value: recovered.map(|value| (value - truth).abs()),
                unit: "state unit".into(),
                evidence_status: status.into(),
                notes: Vec::new(),
            });
        }
        for (parameter_id, truth) in &experiment.reference_parameter_values {
            let recovered = analysis
                .model_definition
                .parameters
                .iter()
                .find(|parameter| parameter.id == *parameter_id)
                .map(|parameter| parameter.default_value);
            metrics.push(ValidationMetric {
                experiment_id: experiment.experiment_id.clone(),
                metric: format!("parameter_recovery_error.{parameter_id}"),
                value: recovered.map(|value| (value - truth).abs()),
                unit: "parameter unit".into(),
                evidence_status: status.into(),
                notes: vec!["Compared with the declared reference parameter value.".into()],
            });
        }
        metrics.push(ValidationMetric {
            experiment_id: experiment.experiment_id.clone(),
            metric: "equilibrium_recognition_accuracy".into(),
            value: None,
            unit: "fraction".into(),
            evidence_status: "missing_reference_labels".into(),
            notes: vec![
                "Reference equilibrium labels were not supplied by this manifest schema.".into(),
            ],
        });
        metrics.push(ValidationMetric {
            experiment_id: experiment.experiment_id.clone(),
            metric: "calibration_transfer_validation".into(),
            value: experiment.calibration_transfer_group.as_ref().map(|_| 1.0),
            unit: "availability".into(),
            evidence_status: status.into(),
            notes: vec!["Transfer accuracy requires held-out calibration predictions.".into()],
        });
        comparison.push(ModelComparisonRow { model_id: analysis.model_definition.model_id, observations: analysis.points.len(), rmse_v: rmse, prediction_coverage: coverage, contribution_reconstruction_error_v: reconstruction, criterion: "RMSE and declared prediction coverage; not an evidence-free information criterion".into(), limitations: vec!["Practical profile likelihood is not available without fitted likelihood evaluations.".into()] });
        if !experiment.is_real_experiment {
            warnings.push(format!(
                "{} is not marked as a real experiment",
                experiment.experiment_id
            ));
        }
    }
    let sensors = manifest
        .experiments
        .iter()
        .map(|experiment| experiment.sensor_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    metrics.push(ValidationMetric {
        experiment_id: manifest.study_id.clone(),
        metric: "cross_sensor_generalization".into(),
        value: (sensors.len() >= 2).then_some(sensors.len() as f64),
        unit: "sensor_count".into(),
        evidence_status: if sensors.len() >= 2 {
            "available".into()
        } else {
            "insufficient_sensors".into()
        },
        notes: vec!["Generalization performance requires held-out sensor evaluation.".into()],
    });
    Ok(ValidationResults {
        schema_version: MODEL_VALIDATION_SCHEMA_VERSION,
        artifact_kind: MODEL_VALIDATION_ARTIFACT_KIND.into(),
        study_id: manifest.study_id.clone(),
        metrics,
        identifiability_report: serde_json::json!({"structural": "not_assessed", "practical": "not_assessed", "profile_likelihood": "missing: fitted likelihood evaluations required"}),
        model_comparison: comparison,
        warnings,
    })
}
