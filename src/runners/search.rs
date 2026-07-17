//! EIS search command orchestration.

use crate::runners::RunnerError;
use std::path::Path;

/// Run the existing EIS search pipeline through a typed workflow boundary.
pub fn run(
    workspace_dir: &Path,
    search_target: &Path,
    search_config_path: Option<&Path>,
    search_output: Option<&Path>,
    search_top: Option<usize>,
) -> Result<(), RunnerError> {
    crate::search_runner::run_eis_search(
        workspace_dir,
        search_target,
        search_config_path,
        search_output,
        search_top,
    )?;
    Ok(())
}
