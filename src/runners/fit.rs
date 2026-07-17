//! Single-file EIS fit command orchestration.

use crate::data_file::EISData;
use crate::fitting::fit_circuit;
use crate::impedance::reporting::format_circuit_fit_report;
use crate::runners::RunnerError;
use std::fs;
use std::path::{Path, PathBuf};

/// Fit one EIS file and print or write a named-parameter report.
pub fn run(
    workspace_dir: &Path,
    input: &Path,
    circuit_model: Option<&str>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let input = resolve_path(workspace_dir, input);
    let data = EISData::parse_file(&input)?;
    let model = circuit_model.unwrap_or(&data.circuit_model);
    let result = fit_circuit(model, &data.freq, &data.z_re, &data.z_im, &data.phase)?;
    let report = format_circuit_fit_report(model, &result);

    if let Some(output) = output {
        let output = resolve_path(workspace_dir, output);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, report)?;
        println!("Fit report written to: {}", output.display());
    } else {
        println!("{report}");
    }

    Ok(())
}

fn resolve_path(workspace_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_dir.join(path)
    }
}
