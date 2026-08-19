//! Certified Phase-D public report runner and atomic bundle publication.

use crate::{
    report_config::{ReportRenderOptions, SelectionMode},
    reporting::{
        AvailabilityReason, PublicReportError,
        document::{write_markdown_report, write_public_summary_json},
        figures::{figure_reason, write_figure},
        projection::PublicReportProjection,
        reader::ReportInputs,
        tables::write_selected_tables,
    },
};
use serde::Serialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Clone)]
pub struct ReportRenderOutcome {
    pub output_dir: PathBuf,
    pub written_files: usize,
    pub unavailable_files: usize,
}

pub fn render(options: &ReportRenderOptions) -> Result<ReportRenderOutcome, PublicReportError> {
    options.validate_pairing()?;
    let inputs = ReportInputs::read(options)?;
    let projection = PublicReportProjection::from_inputs(&inputs);
    preflight_output(&options.output_dir, options.overwrite)?;
    let staging = create_staging(&options.output_dir)?;
    let result = render_staging(&staging, options, &projection);
    let (written, unavailable) = match result {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    publish(&staging, &options.output_dir, options.overwrite)?;
    Ok(ReportRenderOutcome {
        output_dir: options.output_dir.clone(),
        written_files: written,
        unavailable_files: unavailable,
    })
}

fn render_staging(
    root: &Path,
    options: &ReportRenderOptions,
    projection: &PublicReportProjection,
) -> Result<(usize, usize), PublicReportError> {
    let mut written = Vec::new();
    let mut unavailable = Vec::new();
    if options.format.writes_json() {
        written.push(write_public_summary_json(
            root,
            projection,
            &options.selection.tables,
            &options.selection.figures,
        )?);
    }
    if options.format.writes_markdown() {
        written.push(write_markdown_report(root, projection)?);
    }
    written.extend(write_selected_tables(
        root,
        projection,
        &options.selection.tables,
    )?);
    for id in &options.selection.figures {
        if let Some(reason) = figure_reason(projection, *id) {
            if options.selection.figures_mode == SelectionMode::Explicit {
                return Err(PublicReportError::RequestedOutputUnavailable {
                    output_id: id.as_str().into(),
                    reason,
                });
            }
            unavailable.push((id.as_str().to_string(), reason));
        } else {
            written.extend(write_figure(root, *id, projection)?);
        }
    }
    let manifest = write_manifest(root, options, &written, &unavailable)?;
    written.push(manifest);
    Ok((written.len(), unavailable.len()))
}

#[derive(Serialize)]
struct Manifest<'a> {
    schema_version: u32,
    output_kind: &'static str,
    renderer_contract: &'static str,
    route: &'static str,
    final_output_status: &'static str,
    requested: Requested<'a>,
    generated_files: &'a [String],
    unavailable_outputs: &'a [(String, AvailabilityReason)],
    determinism: Determinism,
}
#[derive(Serialize)]
struct Requested<'a> {
    formats: Vec<&'static str>,
    figures: Vec<&'a str>,
    tables: Vec<&'a str>,
    figures_mode: &'static str,
    tables_mode: &'static str,
    overwrite: bool,
}
#[derive(Serialize)]
struct Determinism {
    json_object_order: &'static str,
    array_order: &'static str,
    numeric_format: &'static str,
    csv: &'static str,
    path_separator: &'static str,
    clock: Option<String>,
}

fn write_manifest(
    root: &Path,
    options: &ReportRenderOptions,
    written: &[String],
    unavailable: &[(String, AvailabilityReason)],
) -> Result<String, PublicReportError> {
    let path = root.join("render_manifest.schema1.json");
    let formats = match options.format {
        crate::report_config::ReportFormat::All => vec!["json", "markdown"],
        crate::report_config::ReportFormat::Json => vec!["json"],
        crate::report_config::ReportFormat::Markdown => vec!["markdown"],
    };
    let manifest = Manifest {
        schema_version: 1,
        output_kind: "phase_d_render_manifest",
        renderer_contract: "mhi_v1_phase_d_public_output_v1",
        route: "electroanalysis report render",
        final_output_status: "published",
        requested: Requested {
            formats,
            figures: options
                .selection
                .figures
                .iter()
                .map(|id| id.as_str())
                .collect(),
            tables: options
                .selection
                .tables
                .iter()
                .map(|id| id.as_str())
                .collect(),
            figures_mode: if options.selection.figures_mode == SelectionMode::Default {
                "default"
            } else {
                "explicit"
            },
            tables_mode: if options.selection.tables_mode == SelectionMode::Default {
                "default"
            } else {
                "explicit"
            },
            overwrite: options.overwrite,
        },
        generated_files: written,
        unavailable_outputs: unavailable,
        determinism: Determinism {
            json_object_order: "declaration_order",
            array_order: "contract_order",
            numeric_format: "rust_display_normalized_negative_zero_v1",
            csv: "rfc4180_lf_utf8_v1",
            path_separator: "/",
            clock: None,
        },
    };
    let mut bytes = serde_json::to_vec_pretty(&manifest).map_err(|source| {
        PublicReportError::Serialization {
            path: path.clone(),
            source,
        }
    })?;
    bytes.push(b'\n');
    fs::write(&path, bytes).map_err(|source| PublicReportError::Write {
        path: path.clone(),
        source,
    })?;
    Ok("render_manifest.schema1.json".into())
}

fn preflight_output(output: &Path, overwrite: bool) -> Result<(), PublicReportError> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(PublicReportError::InvalidOutputDirectory {
            path: output.to_path_buf(),
        });
    }
    if !output.exists() {
        return Ok(());
    }
    if !output.is_dir() {
        return Err(PublicReportError::InvalidOutputDirectory {
            path: output.to_path_buf(),
        });
    }
    if !overwrite {
        return Err(PublicReportError::OutputCollision {
            path: output.to_path_buf(),
        });
    }
    validate_prior_contract(output)
}

fn validate_prior_contract(root: &Path) -> Result<(), PublicReportError> {
    let mut manifest_found = false;
    for entry in fs::read_dir(root).map_err(|_| PublicReportError::InvalidOutputDirectory {
        path: root.to_path_buf(),
    })? {
        let entry = entry.map_err(|_| PublicReportError::InvalidOutputDirectory {
            path: root.to_path_buf(),
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|_| PublicReportError::UnmanagedOutputEntry { path: path.clone() })?;
        if kind.is_symlink() {
            return Err(PublicReportError::UnmanagedOutputEntry { path });
        }
        match name.as_ref() {
            "public_summary.schema1.json" | "scientific_report.md" if kind.is_file() => {}
            "render_manifest.schema1.json" if kind.is_file() => manifest_found = true,
            "tables" if kind.is_dir() => validate_prior_children(&path, true)?,
            "figures" if kind.is_dir() => validate_prior_children(&path, false)?,
            _ => return Err(PublicReportError::UnmanagedOutputEntry { path }),
        }
    }
    if !manifest_found {
        return Err(PublicReportError::UnmanagedOutputEntry {
            path: root.to_path_buf(),
        });
    }
    Ok(())
}

fn validate_prior_children(directory: &Path, tables: bool) -> Result<(), PublicReportError> {
    for entry in fs::read_dir(directory).map_err(|_| PublicReportError::UnmanagedOutputEntry {
        path: directory.to_path_buf(),
    })? {
        let entry = entry.map_err(|_| PublicReportError::UnmanagedOutputEntry {
            path: directory.to_path_buf(),
        })?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let kind = entry
            .file_type()
            .map_err(|_| PublicReportError::UnmanagedOutputEntry { path: path.clone() })?;
        let accepted = if tables {
            matches!(
                name.as_ref(),
                "mechanism_evidence.csv"
                    | "sensor_health_dimensions.csv"
                    | "evidence_provenance.csv"
                    | "artifact_lineage.csv"
                    | "timescale_comparison.csv"
                    | "model_consistency.csv"
                    | "current_vs_baseline.csv"
            )
        } else {
            crate::report_config::FigureId::ALL.iter().any(|id| {
                name == format!("{}.svg", id.as_str()) || name == format!("{}.png", id.as_str())
            })
        };
        if !accepted || !kind.is_file() || kind.is_symlink() {
            return Err(PublicReportError::UnmanagedOutputEntry { path });
        }
    }
    Ok(())
}

fn create_staging(output: &Path) -> Result<PathBuf, PublicReportError> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| PublicReportError::InvalidOutputDirectory {
            path: output.to_path_buf(),
        })?;
    for attempt in 0_u32..1000 {
        let path = parent.join(format!(
            ".{name}.phase-d-staging-{}-{attempt}",
            process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => return Err(PublicReportError::Staging { path, source }),
        }
    }
    Err(PublicReportError::Staging {
        path: parent.to_path_buf(),
        source: io::Error::new(io::ErrorKind::AlreadyExists, "exhausted staging paths"),
    })
}

fn publish(staging: &Path, output: &Path, overwrite: bool) -> Result<(), PublicReportError> {
    if !overwrite {
        return fs::rename(staging, output).map_err(|source| PublicReportError::Publication {
            phase: crate::reporting::PublicationPhase::PublishRename,
            staging_path: staging.to_path_buf(),
            backup_path: None,
            source,
        });
    }
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    let backup = parent.join(format!(".{name}.phase-d-backup-{}-0", process::id()));
    fs::rename(output, &backup).map_err(|source| PublicReportError::Publication {
        phase: crate::reporting::PublicationPhase::BackupRename,
        staging_path: staging.to_path_buf(),
        backup_path: Some(backup.clone()),
        source,
    })?;
    if let Err(source) = fs::rename(staging, output) {
        let _ = fs::rename(&backup, output);
        return Err(PublicReportError::Publication {
            phase: crate::reporting::PublicationPhase::PublishRename,
            staging_path: staging.to_path_buf(),
            backup_path: Some(backup),
            source,
        });
    }
    fs::remove_dir_all(&backup).map_err(|source| PublicReportError::Cleanup {
        path: backup,
        source,
    })
}
