//! Single-file EIS fit command orchestration.

use crate::data_file::EISData;
use crate::domain::AnalysisProvenance;
use crate::impedance::fit_circuit_detailed;
use crate::impedance::reporting::format_circuit_fit_report;
use crate::results::EisFitArtifact;
use crate::runners::RunnerError;
use std::fs;
use std::path::{Path, PathBuf};

/// Fit one EIS file and print or write a named-parameter report.
pub fn run(
    workspace_dir: &Path,
    input: &Path,
    circuit_model: Option<&str>,
    output: Option<&Path>,
    artifact: Option<&Path>,
    report: Option<&Path>,
) -> Result<(), RunnerError> {
    let input = resolve_path(workspace_dir, input);
    let data = EISData::parse_file(&input)?;
    let model = circuit_model.unwrap_or(&data.circuit_model);
    let detailed = fit_circuit_detailed(model, &data.freq, &data.z_re, &data.z_im, &data.phase)?;
    let result = detailed.legacy_result.clone();
    let legacy_report = format_circuit_fit_report(model, &result);

    let artifact_value = if artifact.is_some() || report.is_some() {
        let provenance = AnalysisProvenance::from_paths(&input, None)
            .map_err(crate::domain::DataParsingError::from)?;
        Some(EisFitArtifact::from_detailed_fit(
            &data,
            model,
            &result,
            detailed.covariance,
            detailed.condition_number,
            detailed.jacobian_rank,
            provenance,
        ))
    } else {
        None
    };

    if let Some(path) = artifact {
        let path = resolve_path(workspace_dir, path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let artifact_value = artifact_value
            .as_ref()
            .expect("artifact constructed when requested");
        fs::write(
            &path,
            serde_json::to_string_pretty(artifact_value)
                .map_err(|e| RunnerError::Message(e.to_string()))?,
        )?;
        println!("EIS fit artifact written to: {}", path.display());
    }
    if let Some(path) = report {
        let path = resolve_path(workspace_dir, path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let artifact_value = artifact_value
            .as_ref()
            .expect("artifact constructed when requested");
        fs::write(&path, human_artifact_report(artifact_value))?;
        println!("EIS artifact report written to: {}", path.display());
    }

    if let Some(output) = output {
        let output = resolve_path(workspace_dir, output);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, legacy_report)?;
        println!("Fit report written to: {}", output.display());
    } else {
        println!("{legacy_report}");
    }

    Ok(())
}

pub fn export(
    workspace_dir: &Path,
    input: &Path,
    circuit_model: Option<&str>,
    artifact: &Path,
    report: Option<&Path>,
) -> Result<(), RunnerError> {
    run(
        workspace_dir,
        input,
        circuit_model,
        None,
        Some(artifact),
        report,
    )
}

fn human_artifact_report(artifact: &EisFitArtifact) -> String {
    let mut text = format!(
        "EIS fit artifact\n================\nFit ID: {}\nCircuit: {}\nValid frequency points: {}\nRMSE: {:?}\nWeighted RMSE: {:?}\nAIC: {:?}\nAICc: {:?}\nBIC: {:?}\n\nParameters (fitted quantities):\n",
        artifact.fit_id,
        artifact.circuit_expression,
        artifact.statistics.valid_frequency_points,
        artifact.statistics.rmse,
        artifact.statistics.weighted_rmse,
        artifact.statistics.aic,
        artifact.statistics.aicc,
        artifact.statistics.bic
    );
    for parameter in &artifact.parameters {
        text.push_str(&format!(
            "  {} = {:.6e} {}\n",
            parameter.name, parameter.value, parameter.unit
        ));
    }
    text.push_str("\nDerived quantities require topology-aware mechanism analysis. Numerical agreement is temporal compatibility, not proof of a mechanism.\n");
    if !artifact.warnings.is_empty() {
        text.push_str("\nWarnings:\n");
        for warning in &artifact.warnings {
            text.push_str(&format!("  [{:?}] {}\n", warning.kind, warning.message));
        }
    }
    text
}

fn resolve_path(workspace_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_dir.join(path)
    }
}
