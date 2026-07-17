//! Orchestration for `electroanalysis transient fit`.

use crate::data_file::load_experiment;
use crate::domain::ExperimentEventKind;
use crate::plottings::plot_transient_event;
use crate::potentiometry::{TransientAnalysisOptions, analyze_experiment};
use crate::results::transient::TransientAnalysisReport;
use crate::runners::RunnerError;
use crate::transient_config::ResolvedTransientConfig;
use crate::workspace::WorkspaceSetup;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub fn run(
    workspace_dir: &Path,
    _workspace_setup: &WorkspaceSetup,
    input_path: &Path,
    metadata_path: &Path,
    channel: &str,
    config_path: Option<&Path>,
    output_path: Option<&Path>,
    event_kind_name: &str,
    event_index: Option<usize>,
    model: Option<&str>,
    selection: Option<&str>,
    bootstrap: Option<usize>,
    seed: Option<u64>,
) -> Result<(), RunnerError> {
    let input = resolve_path(workspace_dir, input_path);
    let metadata = resolve_path(workspace_dir, metadata_path);
    let loaded_config = ResolvedTransientConfig::load(workspace_dir, config_path)?;
    for warning in &loaded_config.warnings {
        eprintln!("Warning: {warning}");
    }
    let mut config = loaded_config.config;
    config.apply_cli_overrides(model, selection, bootstrap, seed)?;
    let event_kind = event_kind_from_cli(event_kind_name)?;
    if !metadata.is_file() {
        return Err(crate::potentiometry::PotentiometryError::MissingMetadata(
            metadata.display().to_string(),
        )
        .into());
    }
    let (experiment, parse_diagnostics) = load_experiment(&input, &metadata)?;
    if parse_diagnostics.has_issues() {
        eprintln!(
            "Warning: input diagnostics report {} malformed rows, {} missing values, and irregular_sampling={}",
            parse_diagnostics.malformed_rows,
            parse_diagnostics.missing_values,
            parse_diagnostics.irregular_sampling
        );
    }

    let options = TransientAnalysisOptions {
        event_kind,
        event_index,
        config: config.clone(),
    };
    let mut report = analyze_experiment(&experiment, channel, &options)?;
    report.parse_diagnostics = parse_diagnostics.clone();
    // `load_experiment` records the metadata TOML as the Phase 1
    // configuration identity.  For this workflow the resolved transient TOML
    // is the runtime configuration, so replace the provenance identity with
    // the exact input/configuration pair used for this analysis.
    report.provenance =
        crate::domain::AnalysisProvenance::from_paths(&input, config.source_path.as_deref())
            .map_err(crate::domain::DataParsingError::from)?;
    let output_dir = output_path
        .map(|path| resolve_path(workspace_dir, path))
        .unwrap_or_else(|| workspace_dir.join("output"));
    export_report(&report, &output_dir, &config)?;
    if config.plotting.enabled {
        for event in &report.events {
            if event.selected_model.is_some() {
                let _ = plot_transient_event(
                    event,
                    &output_dir,
                    config.plotting.include_components,
                    config.plotting.include_residuals,
                    config.plotting.include_model_comparison,
                )?;
            }
        }
    }
    println!(
        "Transient analysis written to {} ({} event result(s))",
        output_dir.display(),
        report.events.len()
    );
    Ok(())
}

fn event_kind_from_cli(kind: &str) -> Result<ExperimentEventKind, RunnerError> {
    Ok(match kind {
        "concentration-step" => ExperimentEventKind::ConcentrationStep,
        "flow-change" => ExperimentEventKind::FlowChange,
        "temperature-change" => ExperimentEventKind::TemperatureChange,
        "ionic-strength-change" => ExperimentEventKind::IonicStrengthChange,
        "interferent-addition" => ExperimentEventKind::InterferentAddition,
        "flush-start" => ExperimentEventKind::FlushStart,
        "reading-start" => ExperimentEventKind::ReadingStart,
        "flush-end" => ExperimentEventKind::FlushEnd,
        "manual-annotation" => ExperimentEventKind::ManualAnnotation,
        other => {
            return Err(crate::potentiometry::PotentiometryError::invalid(format!(
                "unsupported event kind '{other}'"
            ))
            .into());
        }
    })
}

fn export_report(
    report: &TransientAnalysisReport,
    output_dir: &Path,
    config: &ResolvedTransientConfig,
) -> Result<(), RunnerError> {
    fs::create_dir_all(output_dir)
        .map_err(|error| crate::potentiometry::PotentiometryError::export(output_dir, error))?;
    let json_path = output_dir.join(&config.export.json_filename);
    let json_file = File::create(&json_path)
        .map_err(|error| crate::potentiometry::PotentiometryError::export(&json_path, error))?;
    serde_json::to_writer_pretty(json_file, report).map_err(|source| {
        crate::potentiometry::PotentiometryError::Serialization {
            path: json_path.clone(),
            source,
        }
    })?;

    write_features_csv(report, &output_dir.join(&config.export.features_filename))?;
    write_model_comparison_csv(
        report,
        &output_dir.join(&config.export.model_comparison_filename),
    )?;
    let report_path = output_dir.join(&config.export.report_filename);
    fs::write(&report_path, human_report(report, config))
        .map_err(|error| crate::potentiometry::PotentiometryError::export(&report_path, error))?;
    Ok(())
}

fn write_features_csv(report: &TransientAnalysisReport, path: &Path) -> Result<(), RunnerError> {
    let mut writer = csv::Writer::from_path(path)
        .map_err(|error| crate::potentiometry::PotentiometryError::export(path, error.into()))?;
    writer
        .write_record([
            "event_index",
            "event_timestamp",
            "model",
            "status",
            "baseline_v",
            "equilibrium_v",
            "amplitude_v",
            "tau_fast_s",
            "tau_slow_s",
            "beta",
            "drift_v_per_s",
            "initial_rate_v_per_s",
            "time_to_63_2_s",
            "time_to_90_s",
            "time_to_95_s",
            "rmse_v",
            "mae_v",
            "r_squared",
            "adjusted_r_squared",
            "rss",
            "aic",
            "aicc",
            "bic",
            "durbin_watson",
            "lag1_autocorrelation",
            "max_abs_residual_v",
        ])
        .map_err(|error| crate::potentiometry::PotentiometryError::export(path, error.into()))?;
    for event in &report.events {
        let selected = event
            .selected_model
            .and_then(|model| event.candidate_fits.iter().find(|fit| fit.model == model));
        let features = selected.map(|fit| &fit.derived_features);
        let statistics = selected.map(|fit| &fit.statistics);
        write_record(
            &mut writer,
            path,
            [
                event.event_index.to_string(),
                event.event.timestamp.to_string(),
                event
                    .selected_model
                    .map(|model| model.to_string())
                    .unwrap_or_default(),
                selected
                    .map(|fit| format!("{:?}", fit.status))
                    .unwrap_or_else(|| "failed".to_string()),
                optional(features.and_then(|value| value.baseline_estimate_v)),
                optional(features.and_then(|value| value.fitted_equilibrium_potential_v)),
                optional(features.and_then(|value| value.total_response_amplitude_v)),
                optional(features.and_then(|value| value.tau_fast_s)),
                optional(features.and_then(|value| value.tau_slow_s)),
                optional(features.and_then(|value| value.stretched_beta)),
                optional(features.and_then(|value| value.drift_rate_v_per_s)),
                optional(features.and_then(|value| value.initial_response_rate_v_per_s)),
                optional(features.and_then(|value| value.time_to_63_2_percent_s)),
                optional(features.and_then(|value| value.time_to_90_percent_s)),
                optional(features.and_then(|value| value.time_to_95_percent_s)),
                optional(statistics.and_then(|value| value.rmse_v)),
                optional(statistics.and_then(|value| value.mae_v)),
                optional(statistics.and_then(|value| value.r_squared)),
                optional(statistics.and_then(|value| value.adjusted_r_squared)),
                optional(statistics.and_then(|value| value.rss)),
                optional(statistics.and_then(|value| value.aic)),
                optional(statistics.and_then(|value| value.aicc)),
                optional(statistics.and_then(|value| value.bic)),
                optional(statistics.and_then(|value| value.durbin_watson)),
                optional(statistics.and_then(|value| value.lag1_residual_autocorrelation)),
                optional(statistics.and_then(|value| value.maximum_absolute_residual_v)),
            ],
        )?;
    }
    writer
        .flush()
        .map_err(|error| crate::potentiometry::PotentiometryError::export(path, error))?;
    Ok(())
}

fn write_model_comparison_csv(
    report: &TransientAnalysisReport,
    path: &Path,
) -> Result<(), RunnerError> {
    let mut writer = csv::Writer::from_path(path)
        .map_err(|error| crate::potentiometry::PotentiometryError::export(path, error.into()))?;
    writer
        .write_record([
            "event_index",
            "model",
            "status",
            "aic",
            "aicc",
            "bic",
            "criterion_delta",
            "model_weight",
            "rss",
            "rmse_v",
            "warnings",
            "termination_reason",
        ])
        .map_err(|error| crate::potentiometry::PotentiometryError::export(path, error.into()))?;
    for event in &report.events {
        for fit in &event.candidate_fits {
            write_record(
                &mut writer,
                path,
                [
                    event.event_index.to_string(),
                    fit.model.to_string(),
                    format!("{:?}", fit.status),
                    optional(fit.statistics.aic),
                    optional(fit.statistics.aicc),
                    optional(fit.statistics.bic),
                    optional(fit.statistics.criterion_delta),
                    optional(fit.statistics.model_weight),
                    optional(fit.statistics.rss),
                    optional(fit.statistics.rmse_v),
                    fit.warnings.len().to_string(),
                    fit.statistics
                        .optimizer_termination_reason
                        .clone()
                        .unwrap_or_default(),
                ],
            )?;
        }
    }
    writer
        .flush()
        .map_err(|error| crate::potentiometry::PotentiometryError::export(path, error))?;
    Ok(())
}

fn write_record<const N: usize>(
    writer: &mut csv::Writer<File>,
    path: &Path,
    record: [String; N],
) -> Result<(), RunnerError> {
    writer
        .write_record(record)
        .map_err(|error| crate::potentiometry::PotentiometryError::export(path, error.into()))?;
    Ok(())
}

fn human_report(report: &TransientAnalysisReport, config: &ResolvedTransientConfig) -> String {
    let mut text = String::new();
    text.push_str("Potentiometric transient analysis report\n");
    text.push_str("========================================\n\n");
    text.push_str(&format!(
        "Experiment: {}\nChannel: {} [{}]\nSelection: {}\n\n",
        report.experiment_id, report.channel, report.channel_unit, config.selection.criterion
    ));
    text.push_str(&format!(
        "Input diagnostics: {} total rows, {} parsed, {} skipped, {} malformed, {} missing values; irregular_sampling={}, duplicate_timestamps={}, non_monotonic_timestamps={}\n\n",
        report.parse_diagnostics.total_rows,
        report.parse_diagnostics.successfully_parsed_rows,
        report.parse_diagnostics.skipped_rows,
        report.parse_diagnostics.malformed_rows,
        report.parse_diagnostics.missing_values,
        report.parse_diagnostics.irregular_sampling,
        report.parse_diagnostics.duplicate_timestamps,
        report.parse_diagnostics.non_monotonic_timestamps,
    ));
    text.push_str("Fitted quantities are model-derived descriptive features. Time constants are not assigned an electrochemical mechanism by this workflow.\n\n");
    for event in &report.events {
        text.push_str(&format!(
            "Event {} at t = {:.6} s ({:?})\n",
            event.event_index, event.event.timestamp, event.event.kind
        ));
        if let Some(before) = &event.concentration_before {
            text.push_str(&format!(
                "  concentration before: {:.6} {}\n",
                before.value,
                before.unit.as_deref().unwrap_or("")
            ));
        }
        if let Some(after) = &event.concentration_after {
            text.push_str(&format!(
                "  concentration after: {:.6} {}\n",
                after.value,
                after.unit.as_deref().unwrap_or("")
            ));
        }
        if let Some(failure) = &event.failure {
            text.push_str(&format!("  FAILED: {}\n\n", failure.message));
            continue;
        }
        text.push_str(&format!(
            "  observations: {} finite / {} raw, missing fraction: {}\n",
            event.segment.finite_fitted_observations,
            event.segment.raw_observations,
            optional(event.segment.missing_fraction)
        ));
        text.push_str(&format!(
            "  baseline: {} V\n",
            optional(event.baseline.estimate_v)
        ));
        text.push_str(&format!(
            "  selected model: {}\n",
            event
                .selected_model
                .map(|model| model.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));
        for fit in &event.candidate_fits {
            text.push_str(&format!(
                "    {}: {:?}, AIC={}, BIC={}, RMSE={} V\n",
                fit.model,
                fit.status,
                optional(fit.statistics.aic),
                optional(fit.statistics.bic),
                optional(fit.statistics.rmse_v)
            ));
            if !fit.confidence_intervals.is_empty() {
                text.push_str("      bootstrap confidence intervals:\n");
                for interval in &fit.confidence_intervals {
                    text.push_str(&format!(
                        "        {}: [{}, {}] {}\n",
                        interval.name,
                        optional(interval.lower),
                        optional(interval.upper),
                        interval.unit
                    ));
                }
            }
            for warning in &fit.warnings {
                text.push_str(&format!("      warning: {}\n", warning.message));
            }
        }
        for warning in &event.warnings {
            text.push_str(&format!("  warning: {}\n", warning.message));
        }
        text.push('\n');
    }
    text
}

fn optional(value: Option<f64>) -> String {
    value
        .filter(|value| value.is_finite())
        .map(|value| format!("{value:.12e}"))
        .unwrap_or_default()
}

fn resolve_path(workspace_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_dir.join(path)
    }
}
