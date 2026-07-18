//! Workflow orchestration for equilibrium potentiometric calibration.

use crate::calibration_config::ResolvedCalibrationConfig;
use crate::domain::{AnalysisProvenance, DataParsingError};
use crate::potentiometry::calibration::{
    extract_observations, fit_calibration, prediction, stored_model_from_report,
    validate_stored_model,
};
use crate::results::calibration::{
    CalibrationAnalysisReport, CalibrationObservationSet, CalibrationValidationResult,
    StoredCalibrationModel,
};
use crate::runners::RunnerError;
use csv::Writer;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ExtractOptions<'a> {
    pub input_path: &'a Path,
    pub metadata_path: &'a Path,
    pub channel: &'a str,
    pub sheet: Option<&'a str>,
    pub transient_results_path: Option<&'a Path>,
    pub config_path: Option<&'a Path>,
    pub output_path: Option<&'a Path>,
}

pub fn extract(workspace_dir: &Path, options: ExtractOptions<'_>) -> Result<(), RunnerError> {
    let input = resolve_path(workspace_dir, options.input_path);
    let metadata = resolve_path(workspace_dir, options.metadata_path);
    let loaded = ResolvedCalibrationConfig::load(workspace_dir, options.config_path)?;
    for warning in &loaded.warnings {
        eprintln!("Warning: {warning}");
    }
    let (experiment, diagnostics) =
        crate::data_file::measurement_parser::load_experiment_with_sheet(
            &input,
            &metadata,
            options.sheet,
        )?;
    if diagnostics.has_issues() {
        eprintln!(
            "Warning: input diagnostics report {} malformed rows and {} missing values",
            diagnostics.malformed_rows, diagnostics.missing_values
        );
    }
    let transient = options
        .transient_results_path
        .map(|path| resolve_path(workspace_dir, path))
        .map(|path| read_json::<crate::results::transient::TransientAnalysisReport>(&path))
        .transpose()?;
    let mut observation_set = extract_observations(
        &experiment,
        options.channel,
        transient.as_ref(),
        &loaded.config,
    )?;
    observation_set.provenance =
        AnalysisProvenance::from_paths(&input, loaded.source_path.as_deref())
            .map_err(DataParsingError::from)?;
    let destination = output_file(
        workspace_dir,
        options.output_path,
        &loaded.config.export.observations_filename,
    );
    write_json(&destination, &observation_set)?;
    println!(
        "Calibration observations written to {} ({} observation(s))",
        destination.display(),
        observation_set.observations.len()
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn fit(
    workspace_dir: &Path,
    observations_path: &Path,
    config_path: Option<&Path>,
    output_path: Option<&Path>,
    model: Option<&str>,
    selection: Option<&str>,
    bootstrap: Option<usize>,
    seed: Option<u64>,
) -> Result<(), RunnerError> {
    let observations_path = resolve_path(workspace_dir, observations_path);
    let loaded = ResolvedCalibrationConfig::load(workspace_dir, config_path)?;
    for warning in &loaded.warnings {
        eprintln!("Warning: {warning}");
    }
    let mut config = loaded.config;
    config.apply_cli_overrides(model, selection, bootstrap, seed)?;
    let mut observation_set: CalibrationObservationSet = read_json(&observations_path)?;
    observation_set.provenance =
        AnalysisProvenance::from_paths(&observations_path, loaded.source_path.as_deref())
            .map_err(DataParsingError::from)?;
    let mut report = fit_calibration(&observation_set, &config)?;
    report.provenance = observation_set.provenance.clone();
    let output_dir = output_directory(workspace_dir, output_path);
    fs::create_dir_all(&output_dir).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(&output_dir, error)
    })?;
    let model = stored_model_from_report(&report)?;
    write_json(&output_dir.join(&config.export.model_filename), &model)?;
    write_json(&output_dir.join(&config.export.results_filename), &report)?;
    write_observation_csv(&output_dir.join(&config.export.features_filename), &report)?;
    write_residual_csv(&output_dir.join(&config.export.residuals_filename), &report)?;
    write_validation_csv(
        &output_dir.join(&config.export.validation_filename),
        report.validation.as_ref(),
    )?;
    fs::write(
        output_dir.join(&config.export.report_filename),
        human_report(&report),
    )
    .map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(
            output_dir.join(&config.export.report_filename),
            error,
        )
    })?;
    if config.plotting.enabled {
        crate::plottings::plot_calibration_report(&report, &observation_set, &output_dir)?;
    }
    println!("Calibration fit written to {}", output_dir.display());
    Ok(())
}

pub fn validate(
    workspace_dir: &Path,
    model_path: &Path,
    observations_path: &Path,
    output_path: Option<&Path>,
) -> Result<(), RunnerError> {
    let model_path = resolve_path(workspace_dir, model_path);
    let observations_path = resolve_path(workspace_dir, observations_path);
    let model: StoredCalibrationModel = read_json(&model_path)?;
    let observations: CalibrationObservationSet = read_json(&observations_path)?;
    let validation = validate_stored_model(&model, &observations.observations)?;
    let output_dir = output_directory(workspace_dir, output_path);
    fs::create_dir_all(&output_dir).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(&output_dir, error)
    })?;
    write_json(
        &output_dir.join("calibration_validation_results.json"),
        &validation,
    )?;
    write_validation_csv(
        &output_dir.join("calibration_validation.csv"),
        Some(&validation),
    )?;
    fs::write(
        output_dir.join("calibration_validation_report.txt"),
        validation_report(&validation),
    )
    .map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(
            output_dir.join("calibration_validation_report.txt"),
            error,
        )
    })?;
    println!("Calibration validation written to {}", output_dir.display());
    Ok(())
}

pub fn predict(
    workspace_dir: &Path,
    model_path: &Path,
    potential: Option<f64>,
    temperature_celsius: Option<f64>,
    input_path: Option<&Path>,
    channel: Option<&str>,
    output_path: Option<&Path>,
) -> Result<(), RunnerError> {
    let model_path = resolve_path(workspace_dir, model_path);
    let model: StoredCalibrationModel = read_json(&model_path)?;
    let temperature_k = temperature_celsius.map(|value| value + 273.15);
    let predictions = if let Some(potential) = potential {
        vec![prediction::predict_activity_from_potential(
            &model,
            potential,
            temperature_k,
            &Default::default(),
        )?]
    } else if let Some(input_path) = input_path {
        let input_path = resolve_path(workspace_dir, input_path);
        let channel_name = channel.ok_or_else(|| {
            crate::potentiometry::calibration::error::CalibrationError::InvalidPrediction(
                "--channel is required with --input".to_string(),
            )
        })?;
        let parsed = crate::data_file::measurement_parser::parse_measurement_file_with_sheet(
            &input_path,
            None,
        )?;
        let measurement_channel = parsed.measurement.channel(channel_name).ok_or_else(|| {
            crate::potentiometry::calibration::error::CalibrationError::InvalidPrediction(format!(
                "selected channel '{channel_name}' does not exist"
            ))
        })?;
        measurement_channel
            .values
            .iter()
            .filter_map(|value| {
                value.and_then(|value| {
                    crate::potentiometry::units::Quantity::parse(value, &measurement_channel.unit)
                        .ok()?
                        .to_potential_v()
                        .ok()
                })
            })
            .map(|potential| {
                prediction::predict_activity_from_potential(
                    &model,
                    potential,
                    temperature_k,
                    &Default::default(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        return Err(
            crate::potentiometry::calibration::error::CalibrationError::InvalidPrediction(
                "a potential or input data file is required".to_string(),
            )
            .into(),
        );
    };
    let destination = output_path
        .map(|path| resolve_path(workspace_dir, path))
        .unwrap_or_else(|| workspace_dir.join("prediction.json"));
    if destination
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("csv")
    {
        let mut writer = Writer::from_path(&destination).map_err(|error| {
            crate::potentiometry::calibration::error::CalibrationError::export(
                &destination,
                error.into(),
            )
        })?;
        writer
            .write_record([
                "potential_v",
                "activity",
                "molar_concentration_mol_l",
                "temperature_k",
                "extrapolated",
                "distance_log10_activity",
            ])
            .map_err(|error| {
                crate::potentiometry::calibration::error::CalibrationError::export(
                    &destination,
                    error.into(),
                )
            })?;
        for prediction in &predictions {
            writer
                .write_record([
                    optional(prediction.potential_v),
                    optional(prediction.predicted_activity),
                    optional(prediction.predicted_molar_concentration_mol_l),
                    optional(prediction.temperature_k),
                    prediction.extrapolated.to_string(),
                    optional(prediction.distance_from_domain_log10_activity),
                ])
                .map_err(|error| {
                    crate::potentiometry::calibration::error::CalibrationError::export(
                        &destination,
                        error.into(),
                    )
                })?;
        }
        writer.flush().map_err(|error| {
            crate::potentiometry::calibration::error::CalibrationError::export(&destination, error)
        })?;
    } else {
        if potential.is_some() {
            write_json(&destination, &predictions[0])?;
        } else {
            write_json(&destination, &predictions)?;
        }
    }
    println!(
        "Calibration prediction written to {}",
        destination.display()
    );
    Ok(())
}

fn write_observation_csv(
    path: &Path,
    report: &CalibrationAnalysisReport,
) -> Result<(), RunnerError> {
    let mut writer = Writer::from_path(path).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(path, error.into())
    })?;
    writer
        .write_record([
            "observation_count",
            "selected_model",
            "analyte",
            "charge",
            "rmse_v",
            "slope_v_per_decade",
            "aic",
            "aicc",
            "bic",
        ])
        .map_err(|error| {
            crate::potentiometry::calibration::error::CalibrationError::export(path, error.into())
        })?;
    let selected = report.selected_model.and_then(|model| {
        report
            .candidate_models
            .iter()
            .find(|candidate| candidate.model_kind == model)
    });
    writer
        .write_record([
            report.observation_summary.total_observations.to_string(),
            report
                .selected_model
                .map(|value| format!("{value:?}"))
                .unwrap_or_default(),
            report.analyte.clone(),
            report.ion_charge.to_string(),
            selected
                .map(|value| optional(value.statistics.rmse_v))
                .unwrap_or_default(),
            selected
                .and_then(|value| value.fitted_slope_v_per_decade)
                .map(|value| format!("{value:.12e}"))
                .unwrap_or_default(),
            selected
                .map(|value| optional(value.statistics.aic))
                .unwrap_or_default(),
            selected
                .map(|value| optional(value.statistics.aicc))
                .unwrap_or_default(),
            selected
                .map(|value| optional(value.statistics.bic))
                .unwrap_or_default(),
        ])
        .map_err(|error| {
            crate::potentiometry::calibration::error::CalibrationError::export(path, error.into())
        })?;
    writer.flush().map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(path, error)
    })?;
    Ok(())
}

fn write_residual_csv(path: &Path, report: &CalibrationAnalysisReport) -> Result<(), RunnerError> {
    let mut writer = Writer::from_path(path).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(path, error.into())
    })?;
    writer
        .write_record(["model", "index", "predicted_potential_v", "residual_v"])
        .map_err(|error| {
            crate::potentiometry::calibration::error::CalibrationError::export(path, error.into())
        })?;
    for candidate in &report.candidate_models {
        for (index, (prediction, residual)) in candidate
            .predicted_potential_v
            .iter()
            .zip(candidate.residuals_v.iter())
            .enumerate()
        {
            writer
                .write_record([
                    format!("{:?}", candidate.model_kind),
                    index.to_string(),
                    format!("{prediction:.12e}"),
                    format!("{residual:.12e}"),
                ])
                .map_err(|error| {
                    crate::potentiometry::calibration::error::CalibrationError::export(
                        path,
                        error.into(),
                    )
                })?;
        }
    }
    writer.flush().map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(path, error)
    })?;
    Ok(())
}

fn write_validation_csv(
    path: &Path,
    validation: Option<&CalibrationValidationResult>,
) -> Result<(), RunnerError> {
    let mut writer = Writer::from_path(path).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(path, error.into())
    })?;
    writer
        .write_record([
            "fold_id",
            "held_out_observations",
            "rmse_potential_v",
            "mae_potential_v",
            "failed_predictions",
            "extrapolation_count",
        ])
        .map_err(|error| {
            crate::potentiometry::calibration::error::CalibrationError::export(path, error.into())
        })?;
    if let Some(validation) = validation {
        for fold in &validation.folds {
            writer
                .write_record([
                    fold.fold_id.clone(),
                    fold.held_out_observations.to_string(),
                    optional(fold.rmse_potential_v),
                    optional(fold.mae_potential_v),
                    fold.failed_predictions.to_string(),
                    fold.extrapolation_count.to_string(),
                ])
                .map_err(|error| {
                    crate::potentiometry::calibration::error::CalibrationError::export(
                        path,
                        error.into(),
                    )
                })?;
        }
    }
    writer.flush().map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(path, error)
    })?;
    Ok(())
}

fn human_report(report: &CalibrationAnalysisReport) -> String {
    let mut text = format!(
        "Equilibrium potentiometric calibration report\n===============================================\n\nAnalyte: {}\nIon charge: {}\n\nMeasured observations and converted activities are distinct quantities. Conductivity corrections are labeled empirical; no automatic mechanism or sensor-failure claim is made.\n\n",
        report.analyte, report.ion_charge
    );
    for candidate in &report.candidate_models {
        text.push_str(&format!("Model {:?}: {:?}\n  equation: {}\n  observations: {}\n  RMSE: {}\n  AIC/AICc/BIC: {}/{}/{}\n", candidate.model_kind, candidate.status, candidate.equation, candidate.statistics.observations, optional(candidate.statistics.rmse_v), optional(candidate.statistics.aic), optional(candidate.statistics.aicc), optional(candidate.statistics.bic)));
        for warning in &candidate.warnings {
            text.push_str(&format!("  warning: {}\n", warning.message));
        }
    }
    text.push_str(&format!(
        "\nSelected model: {}\n",
        report
            .selected_model
            .map(|model| format!("{model:?}"))
            .unwrap_or_else(|| "none".to_string())
    ));
    if let Some(hysteresis) = &report.hysteresis {
        text.push_str(&format!(
            "Hysteresis pairs: {}, mean: {}, maximum absolute: {}\n",
            hysteresis.paired_observations,
            optional(hysteresis.mean_hysteresis_v),
            optional(hysteresis.maximum_absolute_hysteresis_v)
        ));
    }
    for warning in &report.warnings {
        text.push_str(&format!("Warning: {}\n", warning.message));
    }
    text
}

fn validation_report(validation: &CalibrationValidationResult) -> String {
    format!(
        "Calibration validation report\n=============================\nMode: {:?}\nRMSE potential: {}\nMAE potential: {}\nFailed predictions: {}\nExtrapolations: {}\n",
        validation.mode,
        optional(validation.rmse_potential_v),
        optional(validation.mae_potential_v),
        validation.failed_predictions,
        validation.extrapolation_count
    )
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, RunnerError> {
    let text = fs::read_to_string(path).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(path, error)
    })?;
    serde_json::from_str(&text).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::InvalidObservation(format!(
            "failed to parse JSON {}: {error}",
            path.display()
        ))
        .into()
    })
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), RunnerError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            crate::potentiometry::calibration::error::CalibrationError::export(parent, error)
        })?;
    }
    let file = fs::File::create(path).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::export(path, error)
    })?;
    serde_json::to_writer_pretty(file, value).map_err(|error| {
        crate::potentiometry::calibration::error::CalibrationError::Serialization {
            path: path.to_path_buf(),
            source: error,
        }
        .into()
    })
}

fn output_directory(workspace_dir: &Path, path: Option<&Path>) -> PathBuf {
    path.map(|path| resolve_path(workspace_dir, path))
        .unwrap_or_else(|| workspace_dir.join("output").join("calibration"))
}

fn output_file(workspace_dir: &Path, path: Option<&Path>, default_name: &str) -> PathBuf {
    let path = path
        .map(|path| resolve_path(workspace_dir, path))
        .unwrap_or_else(|| workspace_dir.join("output").join("calibration"));
    if path.extension().is_some() {
        path
    } else {
        path.join(default_name)
    }
}

fn resolve_path(workspace_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_dir.join(path)
    }
}

fn optional(value: Option<f64>) -> String {
    value
        .filter(|value| value.is_finite())
        .map(|value| format!("{value:.12e}"))
        .unwrap_or_default()
}
