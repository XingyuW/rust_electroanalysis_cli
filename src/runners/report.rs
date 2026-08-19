//! Certified Phase-D public report runner and atomic bundle publication.

use crate::reporting::lineage::LineagePresentationV1;
use crate::{
    report_config::{ReportRenderOptions, SelectionMode},
    reporting::{
        AvailabilityReason, PublicReportError,
        document::{
            ArtifactInputFlagV1, AvailabilityV1, CatalogValidationV1, CompatibilityStatusV1,
            GeneratedOutputKindV1, InputFlagV1, RenderFormatV1, RenderStatusV1, WarningCodeV1,
            write_markdown_report, write_public_summary_json,
        },
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
    let effective_options = resolved_defaults(options, &projection);
    preflight_output(&effective_options.output_dir, effective_options.overwrite)?;
    let staging = create_staging(&effective_options.output_dir)?;
    let result = render_staging(&staging, &effective_options, &projection);
    let (written, unavailable) = match result {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    validate_staging(&staging, &written)?;
    publish(
        &staging,
        &effective_options.output_dir,
        effective_options.overwrite,
    )?;
    Ok(ReportRenderOutcome {
        output_dir: effective_options.output_dir.clone(),
        written_files: written.len(),
        unavailable_files: unavailable.len(),
    })
}

/// Resolves only the contract's best-effort default figures.  Explicit
/// selections are already complete and remain strict.  This is deliberately
/// based solely on supplied artifacts, never on inferred scientific data.
fn resolved_defaults(
    options: &ReportRenderOptions,
    projection: &PublicReportProjection,
) -> ReportRenderOptions {
    let mut resolved = options.clone();
    if resolved.selection.figures_mode != SelectionMode::Default {
        return resolved;
    }
    let mut append = |id| {
        if !resolved.selection.figures.contains(&id) {
            resolved.selection.figures.push(id);
        }
    };
    if projection.eis.is_some() {
        append(crate::report_config::FigureId::EisNyquist);
        append(crate::report_config::FigureId::EisBode);
    }
    if projection.transient.is_some() {
        append(crate::report_config::FigureId::TransientResponse);
    }
    if projection.calibration.is_some() || projection.calibration_observations.is_some() {
        append(crate::report_config::FigureId::CalibrationPerformance);
    }
    if projection.signal.is_some() {
        append(crate::report_config::FigureId::SignalDiagnostics);
    }
    if projection.estimation.is_some() {
        append(crate::report_config::FigureId::EstimationObservedPredicted);
    }
    if projection.model.is_some() {
        append(crate::report_config::FigureId::ModelObservedPredicted);
    }
    resolved
}

type UnavailableOutputs = Vec<(String, AvailabilityReason)>;
type StagingRender = (Vec<String>, UnavailableOutputs);

fn render_staging(
    root: &Path,
    options: &ReportRenderOptions,
    projection: &PublicReportProjection,
) -> Result<StagingRender, PublicReportError> {
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
    let manifest = write_manifest(root, options, projection, &written, &unavailable)?;
    written.push(manifest);
    Ok((written, unavailable))
}

#[derive(Serialize)]
struct RenderManifestV1<'a> {
    schema_version: u32,
    output_kind: &'static str,
    renderer_contract: &'static str,
    route: &'static str,
    final_output_status: &'static str,
    input_references: Vec<ManifestInputReferenceV1>,
    requested: Requested<'a>,
    render_order: Vec<ManifestRenderStepV1>,
    generated_files: Vec<ManifestGeneratedFileV1>,
    unavailable_outputs: Vec<ManifestUnavailableOutputV1>,
    warnings: Vec<ManifestWarningV1>,
    legacy_input_notices: Vec<ManifestLegacyNoticeV1>,
    optional_compatibility: Vec<ManifestCompatibilityOutcomeV1>,
    determinism: Determinism,
}
#[derive(Serialize)]
#[serde(tag = "input_kind", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
enum ManifestInputReferenceV1 {
    Artifact(ManifestArtifactInputReferenceV1),
    LineageCatalog(ManifestLineageCatalogInputReferenceV1),
}
#[derive(Serialize)]
struct ManifestArtifactInputReferenceV1 {
    input_flag: ArtifactInputFlagV1,
    supplied_path_basename: Option<String>,
    artifact_kind: Option<crate::domain::ArtifactKind>,
    schema_version: Option<u32>,
    lineage: Option<LineagePresentationV1>,
    acquisition_families: Option<crate::reporting::lineage::AcquisitionFamilyPresentationV1>,
    availability: AvailabilityV1,
    compatibility: CompatibilityStatusV1,
}
#[derive(Serialize)]
struct ManifestLineageCatalogInputReferenceV1 {
    supplied_path_basename: Option<String>,
    schema_version: Option<u32>,
    availability: AvailabilityV1,
    validation: CatalogValidationV1,
    compatibility: CompatibilityStatusV1,
}
#[derive(Serialize)]
struct ManifestRenderStepV1 {
    ordinal: u32,
    output_kind: GeneratedOutputKindV1,
    output_id: Option<String>,
    relative_path: String,
}
#[derive(Serialize)]
struct ManifestGeneratedFileV1 {
    relative_path: String,
    output_kind: GeneratedOutputKindV1,
    output_id: Option<String>,
    format: RenderFormatV1,
    status: RenderStatusV1,
    source_input_flags: Vec<InputFlagV1>,
}
#[derive(Serialize)]
struct ManifestUnavailableOutputV1 {
    output_kind: GeneratedOutputKindV1,
    output_id: String,
    reason: AvailabilityReason,
}
#[derive(Serialize)]
struct ManifestWarningV1 {
    code: WarningCodeV1,
    message: String,
    input_flag: Option<InputFlagV1>,
    output_id: Option<String>,
}
#[derive(Serialize)]
struct ManifestLegacyNoticeV1 {
    input_flag: InputFlagV1,
    schema_version: u32,
    notice: &'static str,
}
#[derive(Serialize)]
struct ManifestCompatibilityOutcomeV1 {
    input_flag: InputFlagV1,
    against_flag: InputFlagV1,
    status: CompatibilityStatusV1,
    mismatch_axis: Option<crate::reporting::CompatibilityAxis>,
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
    projection: &PublicReportProjection,
    written: &[String],
    unavailable: &[(String, AvailabilityReason)],
) -> Result<String, PublicReportError> {
    let path = root.join("render_manifest.schema1.json");
    let formats = match options.format {
        crate::report_config::ReportFormat::All => vec!["json", "markdown"],
        crate::report_config::ReportFormat::Json => vec!["json"],
        crate::report_config::ReportFormat::Markdown => vec!["markdown"],
    };
    // The manifest is itself a generated certified output.  Include its fixed
    // final path before serializing so the manifest describes the complete
    // staged bundle without any post-write mutation or self-referential hash.
    let mut generated_paths = written.to_vec();
    generated_paths.push("render_manifest.schema1.json".into());
    let generated_files = generated_paths
        .iter()
        .map(generated_file)
        .collect::<Vec<_>>();
    let render_order = generated_files
        .iter()
        .enumerate()
        .map(|(ordinal, file)| ManifestRenderStepV1 {
            ordinal: ordinal as u32,
            output_kind: file.output_kind,
            output_id: file.output_id.clone(),
            relative_path: file.relative_path.clone(),
        })
        .collect();
    let manifest = RenderManifestV1 {
        schema_version: 1,
        output_kind: "phase_d_render_manifest",
        renderer_contract: "mhi_v1_phase_d_public_output_v1",
        route: "electroanalysis report render",
        final_output_status: "published",
        input_references: manifest_input_references(projection),
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
        render_order,
        generated_files,
        unavailable_outputs: unavailable
            .iter()
            .map(|(output_id, reason)| ManifestUnavailableOutputV1 {
                output_kind: GeneratedOutputKindV1::Figure,
                output_id: output_id.clone(),
                reason: *reason,
            })
            .collect(),
        warnings: manifest_warnings(projection, unavailable),
        legacy_input_notices: legacy_notices(projection),
        optional_compatibility: projection
            .optional_compatibility
            .iter()
            .map(|(input, against, outcome)| ManifestCompatibilityOutcomeV1 {
                input_flag: input_flag(input),
                against_flag: input_flag(against),
                status: compatibility_status(outcome.status),
                mismatch_axis: outcome.mismatch_axis,
            })
            .collect(),
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

fn manifest_input_references(projection: &PublicReportProjection) -> Vec<ManifestInputReferenceV1> {
    use crate::reporting::lineage::{project_families, project_lineage};
    let artifact = |flag,
                    path: Option<&Path>,
                    kind,
                    schema,
                    lineage_state: Option<&crate::domain::ArtifactLineageState>,
                    status| {
        let legacy = matches!(
            lineage_state,
            Some(crate::domain::ArtifactLineageState::LegacyUnknown { .. })
        );
        ManifestInputReferenceV1::Artifact(ManifestArtifactInputReferenceV1 {
            input_flag: flag,
            supplied_path_basename: path.map(path_basename),
            artifact_kind: kind,
            schema_version: schema,
            lineage: lineage_state.map(project_lineage),
            acquisition_families: lineage_state.map(|state| match state {
                crate::domain::ArtifactLineageState::Known { identity, .. } => {
                    project_families(&identity.acquisition_families, false)
                }
                crate::domain::ArtifactLineageState::LegacyUnknown { .. } => {
                    project_families(&crate::domain::ArtifactAcquisitionFamilies::Unknown, legacy)
                }
            }),
            availability: status,
            compatibility: if legacy {
                CompatibilityStatusV1::LegacyUnknown
            } else if kind.is_some() {
                CompatibilityStatusV1::Compatible
            } else {
                CompatibilityStatusV1::NotProvided
            },
        })
    };
    vec![
        artifact(
            ArtifactInputFlagV1::Mechanism,
            Some(&projection.input_paths.mechanism),
            Some(crate::domain::ArtifactKind::MechanismAnalysis),
            Some(projection.mechanism.schema_version),
            Some(&projection.mechanism.lineage),
            AvailabilityV1::Available,
        ),
        artifact(
            ArtifactInputFlagV1::Health,
            Some(&projection.input_paths.health),
            Some(crate::domain::ArtifactKind::HealthAssessment),
            Some(projection.health.schema_version),
            Some(&projection.health.lineage),
            AvailabilityV1::Available,
        ),
        artifact(
            ArtifactInputFlagV1::Eis,
            projection.input_paths.eis.as_deref(),
            projection
                .eis
                .as_ref()
                .map(|_| crate::domain::ArtifactKind::EisFit),
            projection.eis.as_ref().map(|v| v.schema_version),
            projection.eis.as_ref().map(|v| &v.lineage),
            if projection.eis.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
        ),
        artifact(
            ArtifactInputFlagV1::Transient,
            projection.input_paths.transient.as_deref(),
            projection
                .transient
                .as_ref()
                .map(|_| crate::domain::ArtifactKind::TransientAnalysis),
            projection.transient.as_ref().map(|v| v.schema_version),
            projection.transient.as_ref().map(|v| &v.lineage),
            if projection.transient.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
        ),
        artifact(
            ArtifactInputFlagV1::Calibration,
            projection.input_paths.calibration.as_deref(),
            projection
                .calibration
                .as_ref()
                .map(|_| crate::domain::ArtifactKind::CalibrationAnalysis),
            projection.calibration.as_ref().map(|v| v.schema_version),
            projection.calibration.as_ref().map(|v| &v.lineage),
            if projection.calibration.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
        ),
        artifact(
            ArtifactInputFlagV1::CalibrationObservations,
            projection.input_paths.calibration_observations.as_deref(),
            projection
                .calibration_observations
                .as_ref()
                .map(|_| crate::domain::ArtifactKind::CalibrationObservations),
            projection
                .calibration_observations
                .as_ref()
                .map(|v| v.schema_version),
            projection
                .calibration_observations
                .as_ref()
                .map(|v| &v.lineage),
            if projection.calibration_observations.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
        ),
        artifact(
            ArtifactInputFlagV1::Signal,
            projection.input_paths.signal.as_deref(),
            projection
                .signal
                .as_ref()
                .map(|_| crate::domain::ArtifactKind::SignalAnalysis),
            projection.signal.as_ref().map(|v| v.schema_version),
            projection.signal.as_ref().map(|v| &v.lineage),
            if projection.signal.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
        ),
        artifact(
            ArtifactInputFlagV1::Estimation,
            projection.input_paths.estimation.as_deref(),
            projection
                .estimation
                .as_ref()
                .map(|_| crate::domain::ArtifactKind::StateEstimation),
            projection.estimation.as_ref().map(|v| v.schema_version),
            projection.estimation.as_ref().map(|v| &v.lineage),
            if projection.estimation.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
        ),
        artifact(
            ArtifactInputFlagV1::Model,
            projection.input_paths.model.as_deref(),
            projection
                .model
                .as_ref()
                .map(|_| crate::domain::ArtifactKind::ModelAnalysis),
            projection.model.as_ref().map(|v| v.schema_version),
            projection.model.as_ref().map(|v| &v.lineage),
            if projection.model.is_some() {
                AvailabilityV1::AvailableWithWarnings
            } else {
                AvailabilityV1::NotProvided
            },
        ),
        ManifestInputReferenceV1::LineageCatalog(ManifestLineageCatalogInputReferenceV1 {
            supplied_path_basename: projection
                .input_paths
                .lineage_catalog
                .as_deref()
                .map(path_basename),
            schema_version: projection.lineage_catalog.as_ref().map(|_| 1),
            availability: if projection.lineage_catalog.is_some() {
                AvailabilityV1::Available
            } else {
                AvailabilityV1::NotProvided
            },
            validation: if projection.lineage_catalog.is_some() {
                CatalogValidationV1::Validated
            } else {
                CatalogValidationV1::NotApplicable
            },
            compatibility: CompatibilityStatusV1::NotApplicable,
        }),
    ]
}
fn generated_file(relative_path: &String) -> ManifestGeneratedFileV1 {
    let (output_kind, output_id, format) = if relative_path == "public_summary.schema1.json" {
        (GeneratedOutputKindV1::Summary, None, RenderFormatV1::Json)
    } else if relative_path == "scientific_report.md" {
        (
            GeneratedOutputKindV1::Markdown,
            None,
            RenderFormatV1::Markdown,
        )
    } else if relative_path.starts_with("tables/") {
        (
            GeneratedOutputKindV1::Table,
            relative_path
                .strip_prefix("tables/")
                .and_then(|name| name.strip_suffix(".csv"))
                .map(str::to_owned),
            RenderFormatV1::Csv,
        )
    } else {
        let format = if relative_path.ends_with(".svg") {
            RenderFormatV1::Svg
        } else {
            RenderFormatV1::Png
        };
        (
            GeneratedOutputKindV1::Figure,
            relative_path
                .strip_prefix("figures/")
                .and_then(|name| {
                    name.strip_suffix(".svg")
                        .or_else(|| name.strip_suffix(".png"))
                })
                .map(str::to_owned),
            format,
        )
    };
    ManifestGeneratedFileV1 {
        relative_path: relative_path.clone(),
        output_kind,
        output_id,
        format,
        status: RenderStatusV1::Written,
        source_input_flags: vec![InputFlagV1::Mechanism, InputFlagV1::Health],
    }
}
fn manifest_warnings(
    projection: &PublicReportProjection,
    unavailable: &[(String, AvailabilityReason)],
) -> Vec<ManifestWarningV1> {
    let mut warnings = Vec::new();
    if projection.lineage_catalog.is_none() {
        warnings.push(ManifestWarningV1 {
            code: WarningCodeV1::CatalogNotSupplied,
            message: "Lineage catalog not supplied; only serialized direct lineage is shown."
                .into(),
            input_flag: Some(InputFlagV1::LineageCatalog),
            output_id: None,
        });
    }
    for comparison in &projection.health.baseline_comparison {
        if matches!(
            comparison.comparability,
            crate::results::FeatureComparability::ComparableWithWarnings
        ) {
            warnings.push(ManifestWarningV1 {
                code: WarningCodeV1::BaselineComparableWithWarnings,
                message: comparison
                    .override_reason
                    .clone()
                    .unwrap_or_else(|| "Comparable with upstream context warning.".into()),
                input_flag: Some(InputFlagV1::Health),
                output_id: Some(
                    crate::report_config::FigureId::CurrentVsBaseline
                        .as_str()
                        .into(),
                ),
            });
        }
    }
    for (id, _) in unavailable {
        warnings.push(ManifestWarningV1 {
            code: WarningCodeV1::OutputUnavailable,
            message: format!("{id} unavailable"),
            input_flag: None,
            output_id: Some(id.clone()),
        });
    }
    warnings
}
fn legacy_notices(projection: &PublicReportProjection) -> Vec<ManifestLegacyNoticeV1> {
    let mut notices = Vec::new();
    if projection.mechanism_is_legacy() {
        notices.push(ManifestLegacyNoticeV1 {
            input_flag: InputFlagV1::Mechanism,
            schema_version: projection.mechanism.schema_version,
            notice: "legacy_mechanism_assessment_not_serialized",
        });
    }
    if projection.health_is_legacy() {
        notices.push(ManifestLegacyNoticeV1 {
            input_flag: InputFlagV1::Health,
            schema_version: projection.health.schema_version,
            notice: "legacy_phase_c_not_serialized",
        });
    }
    notices
}
fn input_flag(value: &str) -> InputFlagV1 {
    match value.trim_start_matches("--") {
        "mechanism" => InputFlagV1::Mechanism,
        "health" => InputFlagV1::Health,
        "eis" => InputFlagV1::Eis,
        "transient" => InputFlagV1::Transient,
        "calibration" => InputFlagV1::Calibration,
        "calibration-observations" => InputFlagV1::CalibrationObservations,
        "signal" => InputFlagV1::Signal,
        "estimation" => InputFlagV1::Estimation,
        "model" => InputFlagV1::Model,
        _ => unreachable!("fixed report input flag"),
    }
}
fn compatibility_status(
    value: crate::reporting::reader::CompatibilityStatus,
) -> CompatibilityStatusV1 {
    match value {
        crate::reporting::reader::CompatibilityStatus::Compatible => {
            CompatibilityStatusV1::Compatible
        }
        crate::reporting::reader::CompatibilityStatus::LegacyUnknown => {
            CompatibilityStatusV1::LegacyUnknown
        }
        crate::reporting::reader::CompatibilityStatus::NotProvided => {
            CompatibilityStatusV1::NotProvided
        }
        crate::reporting::reader::CompatibilityStatus::NotApplicable => {
            CompatibilityStatusV1::NotApplicable
        }
    }
}
fn path_basename(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned()
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

fn validate_staging(root: &Path, written: &[String]) -> Result<(), PublicReportError> {
    let mut actual = Vec::new();
    collect_staged_files(root, root, &mut actual)?;
    actual.sort();
    let mut expected = written.to_vec();
    expected.sort();
    if actual != expected {
        return Err(PublicReportError::StagingValidation {
            path: root.to_path_buf(),
            detail: format!(
                "staging files differ from the certified output set: expected {expected:?}, found {actual:?}"
            ),
        });
    }
    for relative in actual {
        let path = root.join(&relative);
        let bytes = fs::read(&path).map_err(|source| PublicReportError::Staging {
            path: path.clone(),
            source,
        })?;
        if relative.ends_with(".json")
            || relative.ends_with(".csv")
            || relative.ends_with(".md")
            || relative.ends_with(".svg")
        {
            std::str::from_utf8(&bytes).map_err(|_| PublicReportError::StagingValidation {
                path: path.clone(),
                detail: "text output is not UTF-8".into(),
            })?;
        }
        if relative.ends_with(".png") && image::image_dimensions(&path).is_err() {
            return Err(PublicReportError::StagingValidation {
                path,
                detail: "PNG output could not be decoded".into(),
            });
        }
    }
    Ok(())
}

fn collect_staged_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<String>,
) -> Result<(), PublicReportError> {
    for entry in fs::read_dir(directory).map_err(|source| PublicReportError::Staging {
        path: directory.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| PublicReportError::Staging {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|source| PublicReportError::Staging {
                path: path.clone(),
                source,
            })?;
        if kind.is_symlink() {
            return Err(PublicReportError::StagingValidation {
                path,
                detail: "staging must not contain symlinks".into(),
            });
        }
        if kind.is_dir() {
            collect_staged_files(root, &path, files)?;
        } else if kind.is_file() {
            files.push(
                path.strip_prefix(root)
                    .map_err(|_| PublicReportError::StagingValidation {
                        path: path.clone(),
                        detail: "staging file escapes root".into(),
                    })?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        } else {
            return Err(PublicReportError::StagingValidation {
                path,
                detail: "staging contains an unsupported entry".into(),
            });
        }
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
