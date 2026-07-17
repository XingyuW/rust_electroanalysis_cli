//! CLI binary entrypoint.
//!
//! Parsing and command validation live in cli.rs; this file only prepares
//! the workspace and dispatches to typed workflow runners.

use rust_electroanalysis_cli::cli::{CliError, CommandSpec, parse_cli_args, print_usage};
use rust_electroanalysis_cli::domain::{ConfigurationError, WorkspaceError};
use rust_electroanalysis_cli::plot_config::PlotConfig;
use rust_electroanalysis_cli::runners::{RunnerError, calibration, fit, plot, search, transient};
use rust_electroanalysis_cli::workspace::{self, LastRunMode};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
enum ApplicationError {
    #[error(transparent)]
    Cli(#[from] CliError),
    #[error(transparent)]
    Configuration(#[from] ConfigurationError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error(transparent)]
    Runner(#[from] RunnerError),
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

/// Parse CLI args, resolve configuration, and dispatch the selected workflow.
fn run() -> Result<(), ApplicationError> {
    let raw_args: Vec<String> = std::env::args().collect();
    let parsed = parse_cli_args(&raw_args)?;

    if let Some(help_text) = parsed.help_text {
        print!("{help_text}");
        return Ok(());
    }

    if raw_args.len() == 1 {
        print_usage(
            raw_args
                .first()
                .map(String::as_str)
                .unwrap_or("electroanalysis"),
        );
        return Ok(());
    }

    let Some(command) = parsed.command else {
        print_usage(
            raw_args
                .first()
                .map(String::as_str)
                .unwrap_or("electroanalysis"),
        );
        return Ok(());
    };

    let workspace_dir = std::env::current_dir().map_err(WorkspaceError::from)?;
    let mut workspace_setup = workspace::prepare_workspace(&workspace_dir)?;
    for warning in &workspace_setup.warnings {
        eprintln!("Warning: {warning}");
    }

    match command {
        CommandSpec::Plot {
            target,
            plot_config_path,
        } => {
            let plot_config = PlotConfig::load(&workspace_dir, plot_config_path.as_deref())?;
            let mode = match target {
                rust_electroanalysis_cli::cli::PlotTarget::All => LastRunMode::PlotAll,
                rust_electroanalysis_cli::cli::PlotTarget::Eis => LastRunMode::PlotEis,
                rust_electroanalysis_cli::cli::PlotTarget::RegularPlot => LastRunMode::PlotRegular,
                rust_electroanalysis_cli::cli::PlotTarget::GenericPlot => LastRunMode::PlotGeneric,
            };
            workspace_setup.record_last_run(mode, plot_config_path.as_deref(), None, None, None)?;
            for warning in &plot_config.warnings {
                eprintln!("Warning: {warning}");
            }
            plot::run(&workspace_dir, &plot_config, target)?;
        }
        CommandSpec::EisSearch {
            input,
            search_config_path,
            search_output,
            search_top,
        } => {
            workspace_setup.record_last_run(
                LastRunMode::Search,
                None,
                search_config_path.as_deref(),
                search_output.as_deref(),
                search_top,
            )?;
            search::run(
                &workspace_dir,
                &input,
                search_config_path.as_deref(),
                search_output.as_deref(),
                search_top,
            )?;
        }
        CommandSpec::EisFit {
            input,
            circuit_model,
            output,
        } => {
            workspace_setup.record_last_run(LastRunMode::EisFit, None, None, None, None)?;
            fit::run(
                &workspace_dir,
                &input,
                circuit_model.as_deref(),
                output.as_deref(),
            )?;
        }
        CommandSpec::TransientFit {
            input,
            metadata,
            channel,
            config_path,
            output,
            event_kind,
            event_index,
            model,
            selection,
            bootstrap,
            seed,
        } => {
            workspace_setup.record_last_run(
                LastRunMode::TransientFit,
                None,
                config_path.as_deref(),
                output.as_deref(),
                None,
            )?;
            transient::run(
                &workspace_dir,
                &workspace_setup,
                &input,
                &metadata,
                &channel,
                config_path.as_deref(),
                output.as_deref(),
                &event_kind,
                event_index,
                model.as_deref(),
                selection.as_deref(),
                bootstrap,
                seed,
            )?;
        }
        CommandSpec::CalibrationExtract {
            input,
            metadata,
            channel,
            transient_results,
            config_path,
            output,
        } => {
            workspace_setup.record_calibration_run(
                LastRunMode::CalibrationExtract,
                config_path.as_deref(),
                output.as_deref(),
            )?;
            calibration::extract(
                &workspace_dir,
                &input,
                &metadata,
                &channel,
                transient_results.as_deref(),
                config_path.as_deref(),
                output.as_deref(),
            )?;
        }
        CommandSpec::CalibrationFit {
            observations,
            config_path,
            output,
            model,
            selection,
            bootstrap,
            seed,
        } => {
            workspace_setup.record_calibration_run(
                LastRunMode::CalibrationFit,
                config_path.as_deref(),
                output.as_deref(),
            )?;
            calibration::fit(
                &workspace_dir,
                &observations,
                config_path.as_deref(),
                output.as_deref(),
                model.as_deref(),
                selection.as_deref(),
                bootstrap,
                seed,
            )?;
        }
        CommandSpec::CalibrationValidate {
            model,
            observations,
            output,
        } => {
            workspace_setup.record_calibration_run(
                LastRunMode::CalibrationValidate,
                None,
                output.as_deref(),
            )?;
            calibration::validate(&workspace_dir, &model, &observations, output.as_deref())?;
        }
        CommandSpec::CalibrationPredict {
            model,
            potential,
            temperature,
            input,
            channel,
            output,
        } => {
            workspace_setup.record_calibration_run(
                LastRunMode::CalibrationPredict,
                None,
                output.as_deref(),
            )?;
            calibration::predict(
                &workspace_dir,
                &model,
                potential,
                temperature,
                input.as_deref(),
                channel.as_deref(),
                output.as_deref(),
            )?;
        }
    }

    Ok(())
}
