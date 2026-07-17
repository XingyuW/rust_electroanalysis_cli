//! Plot command orchestration.

use crate::cli::PlotTarget;
use crate::plot_config::LoadedPlotConfig;
use crate::runners::RunnerError;
use std::path::Path;

/// Dispatch the selected plot workflow while keeping scientific/rendering
/// logic in the existing runner and plotting modules.
pub fn run(
    workspace_dir: &Path,
    plot_config: &LoadedPlotConfig,
    target: PlotTarget,
) -> Result<(), RunnerError> {
    match target {
        PlotTarget::All => {
            crate::plot_runner::run_eis_plots(workspace_dir, plot_config)?;
            crate::plot_runner::run_regular_plots(workspace_dir, plot_config)?;
            crate::plot_runner::run_generic_plots(workspace_dir, plot_config)?;
        }
        PlotTarget::Eis => crate::plot_runner::run_eis_plots(workspace_dir, plot_config)?,
        PlotTarget::RegularPlot => {
            crate::plot_runner::run_regular_plots(workspace_dir, plot_config)?
        }
        PlotTarget::GenericPlot => {
            crate::plot_runner::run_generic_plots(workspace_dir, plot_config)?
        }
    }
    Ok(())
}
