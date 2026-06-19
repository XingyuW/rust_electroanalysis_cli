//! CLI binary entrypoint.
//!
//! Dispatches to either plotting workflows or ECM-search workflows based on
//! parsed command-line flags.

use rust_plots::cli::{PlotTarget, parse_cli_args, print_usage};
use rust_plots::plot_config::PlotConfig;
use rust_plots::workspace::{self, LastRunMode};
use rust_plots::{plot_runner, search_runner};

/// Parse CLI args, resolve configuration, and dispatch workflow execution.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        print_usage(&args[0]);
        return Ok(());
    }

    let cli_args = parse_cli_args(&args)?;
    let workspace_dir = std::env::current_dir()?;
    let mut workspace_setup = workspace::prepare_workspace(&workspace_dir)?;
    for warning in &workspace_setup.warnings {
        eprintln!("Warning: {warning}");
    }

    // When a search target is provided, run the ECM search pipeline instead
    // of the plotting pipeline.
    if let Some(search_target) = cli_args.search_target.as_deref() {
        workspace_setup.record_last_run(
            LastRunMode::Search,
            cli_args.plot_config_path.as_deref(),
            cli_args.search_config_path.as_deref(),
            cli_args.search_output.as_deref(),
            cli_args.search_top,
        )?;
        return search_runner::run_eis_search(
            &workspace_dir,
            search_target,
            cli_args.search_config_path.as_deref(),
            cli_args.search_output.as_deref(),
            cli_args.search_top,
        );
    }

    let plot_config = PlotConfig::load(&workspace_dir, cli_args.plot_config_path.as_deref())
        .map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;

    let mode = match cli_args.plot_target.unwrap_or(PlotTarget::All) {
        PlotTarget::All => LastRunMode::PlotAll,
        PlotTarget::Eis => LastRunMode::PlotEis,
        PlotTarget::RegularPlot => LastRunMode::PlotRegular,
        PlotTarget::GenericPlot => LastRunMode::PlotGeneric,
    };
    workspace_setup.record_last_run(
        mode,
        cli_args.plot_config_path.as_deref(),
        cli_args.search_config_path.as_deref(),
        cli_args.search_output.as_deref(),
        cli_args.search_top,
    )?;

    for warning in &plot_config.warnings {
        eprintln!("Warning: {warning}");
    }

    match cli_args.plot_target {
        None => {
            // `--help` / `-h` was requested.
            print_usage(&args[0]);
            Ok(())
        }
        Some(PlotTarget::All) => {
            // Run all three plot pipelines.  Generic plots are a silent no-op
            // when no [[generic_plot]] blocks are present in the config.
            plot_runner::run_eis_plots(&workspace_dir, &plot_config)?;
            plot_runner::run_regular_plots(&workspace_dir, &plot_config)?;
            plot_runner::run_generic_plots(&workspace_dir, &plot_config)?;
            Ok(())
        }
        Some(PlotTarget::Eis) => plot_runner::run_eis_plots(&workspace_dir, &plot_config),
        Some(PlotTarget::RegularPlot) => {
            plot_runner::run_regular_plots(&workspace_dir, &plot_config)
        }
        Some(PlotTarget::GenericPlot) => {
            plot_runner::run_generic_plots(&workspace_dir, &plot_config)
        }
    }
}
