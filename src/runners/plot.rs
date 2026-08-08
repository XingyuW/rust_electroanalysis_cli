//! Plot command orchestration.

use crate::cli::PlotTarget;
use crate::plot_config::LoadedPlotConfig;
use crate::runners::{BatchRunSummary, RunnerError};
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
            let mut batch = BatchRunSummary::default();
            collect_batch_result(
                crate::plot_runner::run_eis_plots(workspace_dir, plot_config),
                &mut batch,
            )?;
            collect_batch_result(
                crate::plot_runner::run_regular_plots(workspace_dir, plot_config),
                &mut batch,
            )?;
            collect_batch_result(
                crate::plot_runner::run_generic_plots(workspace_dir, plot_config),
                &mut batch,
            )?;
            if !batch.failures.is_empty() {
                return if batch.successful_inputs.is_empty() {
                    Err(RunnerError::BatchInput {
                        failures: batch.failures,
                    })
                } else {
                    Err(RunnerError::partial_batch(batch))
                };
            }
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

fn collect_batch_result(
    result: Result<(), RunnerError>,
    aggregate: &mut BatchRunSummary,
) -> Result<(), RunnerError> {
    match result {
        Ok(()) => Ok(()),
        Err(RunnerError::PartialBatch { summary, .. }) => {
            aggregate
                .successful_inputs
                .extend(summary.successful_inputs);
            aggregate.failures.extend(summary.failures);
            Ok(())
        }
        Err(RunnerError::BatchInput { failures }) => {
            aggregate.failures.extend(failures);
            Ok(())
        }
        Err(error) => Err(error),
    }
}
