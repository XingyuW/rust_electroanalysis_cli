//! Certified Phase-D public report runner and atomic bundle publication.

use crate::reporting::lineage::LineagePresentationV1;
use crate::{
    report_config::{FigureId, ReportRenderOptions, SelectionMode, TableId},
    reporting::{
        AvailabilityReason, PublicReportError,
        document::{
            ArtifactInputFlagV1, AvailabilityV1, CatalogValidationV1, CompatibilityStatusV1,
            GeneratedOutputKindV1, HealthWarningV1, InputFlagV1, RenderFormatV1, RenderStatusV1,
            WarningCodeV1, catalog_availability, optional_input_availability, source_availability,
            write_markdown_report, write_public_summary_json,
        },
        figures::{figure_reason, write_figure},
        projection::PublicReportProjection,
        reader::ReportInputs,
        tables::write_selected_tables,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
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

/// Narrow binary-facing runner bridge. The actual Phase-D rendering
/// entrypoint remains crate-private and is re-exported only within the crate
/// through `reporting::render_public_report`.
pub fn run(options: &ReportRenderOptions) -> Result<ReportRenderOutcome, PublicReportError> {
    crate::reporting::render_public_report(options)
}

pub(crate) fn render_public_report(
    options: &ReportRenderOptions,
) -> Result<ReportRenderOutcome, PublicReportError> {
    options.validate_pairing()?;
    let inputs = ReportInputs::read(options)?;
    let projection = PublicReportProjection::from_inputs(&inputs);
    let effective_options = resolved_defaults(options, &projection);
    preflight_output(&effective_options.output_dir, effective_options.overwrite)?;
    let figure_availability = preflight_figures(&effective_options, &projection)?;
    let staging = create_staging(&effective_options.output_dir)?;
    let publication = (|| {
        let (written, unavailable) = render_staging(
            &staging,
            &effective_options,
            &projection,
            &figure_availability,
        )?;
        write_test_traversal_audit(&projection)?;
        validate_staging(&staging, &written)?;
        publish(
            &staging,
            &effective_options.output_dir,
            effective_options.overwrite,
        )?;
        Ok::<_, PublicReportError>((written, unavailable))
    })();
    let (written, unavailable) = match publication {
        Ok(result) => result,
        Err(error) => {
            if staging.exists() {
                remove_noncertified(&staging).map_err(|source| PublicReportError::Cleanup {
                    path: staging.clone(),
                    source,
                })?;
            }
            return Err(error);
        }
    };
    Ok(ReportRenderOutcome {
        output_dir: effective_options.output_dir.clone(),
        written_files: written.len(),
        unavailable_files: unavailable.len(),
    })
}

#[cfg(debug_assertions)]
fn write_test_traversal_audit(
    projection: &PublicReportProjection<'_>,
) -> Result<(), PublicReportError> {
    let Some(path) = std::env::var_os("ELECTROANALYSIS_PHASE_D_TEST_TRAVERSAL_AUDIT") else {
        return Ok(());
    };
    let path = PathBuf::from(path);
    let (history_traversals, evidence_traversals, history_count, evidence_count) =
        projection.traversal_audit();
    let document = format!(
        "{{\n  \"mechanism_history_projection_traversals\": {history_traversals},\n  \"health_evidence_projection_traversals\": {evidence_traversals},\n  \"mechanism_history_count\": {history_count},\n  \"health_evidence_count\": {evidence_count}\n}}\n"
    );
    fs::write(&path, document).map_err(|source| PublicReportError::Write { path, source })
}

#[cfg(not(debug_assertions))]
fn write_test_traversal_audit(
    projection: &PublicReportProjection<'_>,
) -> Result<(), PublicReportError> {
    let _ = projection.traversal_audit();
    Ok(())
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
    resolved.selection.figures.sort_by_key(|selected| {
        FigureId::ALL
            .iter()
            .position(|candidate| candidate == selected)
            .expect("closed figure identifier")
    });
    resolved
}

type UnavailableOutputs = Vec<(String, AvailabilityReason)>;
type StagingRender = (Vec<String>, UnavailableOutputs);
type FigureAvailability = Vec<(FigureId, Option<AvailabilityReason>)>;

fn preflight_figures(
    options: &ReportRenderOptions,
    projection: &PublicReportProjection<'_>,
) -> Result<FigureAvailability, PublicReportError> {
    options
        .selection
        .figures
        .iter()
        .map(|id| {
            let reason = figure_reason(projection, *id);
            if let Some(reason) = reason
                && options.selection.figures_mode == SelectionMode::Explicit
            {
                return Err(PublicReportError::RequestedOutputUnavailable {
                    output_id: id.as_str().into(),
                    reason,
                });
            }
            Ok((*id, reason))
        })
        .collect()
}

fn render_staging(
    root: &Path,
    options: &ReportRenderOptions,
    projection: &PublicReportProjection,
    figure_availability: &FigureAvailability,
) -> Result<StagingRender, PublicReportError> {
    #[cfg(debug_assertions)]
    if std::env::var_os("ELECTROANALYSIS_PHASE_D_TEST_NONFINITE_PROJECTION").is_some() {
        return Err(PublicReportError::StagingValidation {
            path: PathBuf::from("public report projection"),
            detail: crate::reporting::format_public_f64(f64::NAN)
                .expect_err("non-finite values are rejected")
                .into(),
        });
    }
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
        written.push(write_markdown_report(
            root,
            projection,
            &options.selection.tables,
            &options.selection.figures,
        )?);
    }
    written.extend(write_selected_tables(
        root,
        projection,
        &options.selection.tables,
    )?);
    for (id, reason) in figure_availability {
        if let Some(reason) = reason {
            unavailable.push((id.as_str().to_string(), *reason));
        } else {
            written.extend(write_figure(root, *id, projection)?);
        }
    }
    let manifest = write_manifest(root, options, projection, &written, &unavailable)?;
    written.push(manifest);
    Ok((written, unavailable))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RenderManifestV1 {
    schema_version: u32,
    output_kind: String,
    renderer_contract: String,
    route: String,
    final_output_status: FinalOutputStatusV1,
    input_references: Vec<ManifestInputReferenceV1>,
    requested: Requested,
    render_order: Vec<ManifestRenderStepV1>,
    generated_files: Vec<ManifestGeneratedFileV1>,
    unavailable_outputs: Vec<ManifestUnavailableOutputV1>,
    warnings: Vec<ManifestWarningV1>,
    legacy_input_notices: Vec<ManifestLegacyNoticeV1>,
    optional_compatibility: Vec<ManifestCompatibilityOutcomeV1>,
    determinism: Determinism,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "input_kind", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
enum ManifestInputReferenceV1 {
    Artifact(ManifestArtifactInputReferenceV1),
    LineageCatalog(ManifestLineageCatalogInputReferenceV1),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestLineageCatalogInputReferenceV1 {
    supplied_path_basename: Option<String>,
    schema_version: Option<u32>,
    availability: AvailabilityV1,
    validation: CatalogValidationV1,
    compatibility: CompatibilityStatusV1,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestRenderStepV1 {
    ordinal: u32,
    output_kind: GeneratedOutputKindV1,
    output_id: Option<String>,
    relative_path: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestGeneratedFileV1 {
    relative_path: String,
    output_kind: GeneratedOutputKindV1,
    output_id: Option<String>,
    format: RenderFormatV1,
    status: RenderStatusV1,
    source_input_flags: Vec<InputFlagV1>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestUnavailableOutputV1 {
    output_kind: GeneratedOutputKindV1,
    output_id: String,
    reason: AvailabilityReason,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestWarningV1 {
    code: WarningCodeV1,
    message: String,
    input_flag: Option<InputFlagV1>,
    output_id: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestLegacyNoticeV1 {
    input_flag: InputFlagV1,
    schema_version: u32,
    notice: LegacyNoticeV1,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestCompatibilityOutcomeV1 {
    input_flag: InputFlagV1,
    against_flag: InputFlagV1,
    status: CompatibilityStatusV1,
    mismatch_axis: Option<crate::reporting::CompatibilityAxis>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requested {
    formats: Vec<RenderFormatV1>,
    figures: Vec<FigureId>,
    tables: Vec<TableId>,
    figures_mode: SelectionMode,
    tables_mode: SelectionMode,
    overwrite: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Determinism {
    json_object_order: JsonObjectOrderV1,
    array_order: ArrayOrderV1,
    numeric_format: String,
    csv: String,
    path_separator: String,
    clock: Option<String>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FinalOutputStatusV1 {
    Published,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum JsonObjectOrderV1 {
    DeclarationOrder,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ArrayOrderV1 {
    ContractOrder,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum LegacyNoticeV1 {
    #[serde(rename = "legacy_phase_c_not_serialized")]
    PhaseCNotSerialized,
    #[serde(rename = "legacy_mechanism_assessment_not_serialized")]
    MechanismAssessmentNotSerialized,
    #[serde(rename = "legacy_lineage_unknown")]
    LineageUnknown,
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
        crate::report_config::ReportFormat::All => {
            vec![RenderFormatV1::Json, RenderFormatV1::Markdown]
        }
        crate::report_config::ReportFormat::Json => vec![RenderFormatV1::Json],
        crate::report_config::ReportFormat::Markdown => vec![RenderFormatV1::Markdown],
    };
    // The manifest is itself a generated certified output.  Include its fixed
    // final path before serializing so the manifest describes the complete
    // staged bundle without any post-write mutation or self-referential hash.
    let mut generated_paths = written.to_vec();
    generated_paths.push("render_manifest.schema1.json".into());
    let generated_files = generated_paths
        .iter()
        .map(|path| generated_file(path, projection))
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
        output_kind: "phase_d_render_manifest".into(),
        renderer_contract: "mhi_v1_phase_d_public_output_v1".into(),
        route: "electroanalysis report render".into(),
        final_output_status: FinalOutputStatusV1::Published,
        input_references: manifest_input_references(
            projection,
            &options.selection.tables,
            &options.selection.figures,
        ),
        requested: Requested {
            formats,
            figures: options.selection.figures.clone(),
            tables: options.selection.tables.clone(),
            figures_mode: options.selection.figures_mode,
            tables_mode: options.selection.tables_mode,
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
            json_object_order: JsonObjectOrderV1::DeclarationOrder,
            array_order: ArrayOrderV1::ContractOrder,
            numeric_format: "rust_display_normalized_negative_zero_v1".into(),
            csv: "rfc4180_lf_utf8_v1".into(),
            path_separator: "/".into(),
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

fn manifest_input_references(
    projection: &PublicReportProjection,
    tables: &[TableId],
    figures: &[FigureId],
) -> Vec<ManifestInputReferenceV1> {
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
            compatibility: manifest_input_compatibility(projection, flag, kind.is_some()),
        })
    };
    vec![
        artifact(
            ArtifactInputFlagV1::Mechanism,
            Some(&projection.input_paths.mechanism),
            Some(crate::domain::ArtifactKind::MechanismAnalysis),
            Some(projection.mechanism.schema_version),
            Some(&projection.mechanism.lineage),
            source_availability(
                &projection.mechanism.lineage,
                !projection.mechanism.warnings.is_empty(),
            ),
        ),
        artifact(
            ArtifactInputFlagV1::Health,
            Some(&projection.input_paths.health),
            Some(crate::domain::ArtifactKind::HealthAssessment),
            Some(projection.health.schema_version),
            Some(&projection.health.lineage),
            source_availability(
                &projection.health.lineage,
                !projection.health.warnings.is_empty(),
            ),
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
            optional_input_availability(
                projection,
                ArtifactInputFlagV1::Eis,
                projection
                    .eis
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
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
            optional_input_availability(
                projection,
                ArtifactInputFlagV1::Transient,
                projection.transient.map(|value| {
                    (
                        &value.lineage,
                        value.events.iter().any(|event| {
                            !event.warnings.is_empty()
                                || event
                                    .candidate_fits
                                    .iter()
                                    .any(|fit| !fit.warnings.is_empty())
                        }),
                    )
                }),
                tables,
                figures,
            ),
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
            optional_input_availability(
                projection,
                ArtifactInputFlagV1::Calibration,
                projection
                    .calibration
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
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
            optional_input_availability(
                projection,
                ArtifactInputFlagV1::CalibrationObservations,
                projection
                    .calibration_observations
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
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
            optional_input_availability(
                projection,
                ArtifactInputFlagV1::Signal,
                projection
                    .signal
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
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
            optional_input_availability(
                projection,
                ArtifactInputFlagV1::Estimation,
                projection
                    .estimation
                    .map(|value| (&value.lineage, !value.warnings.is_empty())),
                tables,
                figures,
            ),
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
            optional_input_availability(
                projection,
                ArtifactInputFlagV1::Model,
                projection.model.map(|value| (&value.lineage, false)),
                tables,
                figures,
            ),
        ),
        ManifestInputReferenceV1::LineageCatalog(ManifestLineageCatalogInputReferenceV1 {
            supplied_path_basename: projection
                .input_paths
                .lineage_catalog
                .as_deref()
                .map(path_basename),
            schema_version: projection.lineage_catalog.as_ref().map(|_| 1),
            availability: catalog_availability(projection, tables, figures),
            validation: if projection.lineage_catalog.is_some() {
                CatalogValidationV1::Validated
            } else {
                CatalogValidationV1::NotApplicable
            },
            compatibility: CompatibilityStatusV1::NotApplicable,
        }),
    ]
}
fn manifest_input_compatibility(
    projection: &PublicReportProjection<'_>,
    flag: ArtifactInputFlagV1,
    supplied: bool,
) -> CompatibilityStatusV1 {
    if !supplied {
        return CompatibilityStatusV1::NotProvided;
    }
    if matches!(
        flag,
        ArtifactInputFlagV1::Mechanism | ArtifactInputFlagV1::Health
    ) {
        return compatibility_status(projection.required_compatibility.status);
    }
    let flag_text = match flag {
        ArtifactInputFlagV1::Eis => "--eis",
        ArtifactInputFlagV1::Transient => "--transient",
        ArtifactInputFlagV1::Calibration => "--calibration",
        ArtifactInputFlagV1::CalibrationObservations => "--calibration-observations",
        ArtifactInputFlagV1::Signal => "--signal",
        ArtifactInputFlagV1::Estimation => "--estimation",
        ArtifactInputFlagV1::Model => "--model",
        ArtifactInputFlagV1::Mechanism | ArtifactInputFlagV1::Health => unreachable!(),
    };
    if projection
        .optional_compatibility
        .iter()
        .filter(|(input, _, _)| *input == flag_text)
        .any(|(_, _, outcome)| {
            outcome.status == crate::reporting::reader::CompatibilityStatus::LegacyUnknown
        })
    {
        CompatibilityStatusV1::LegacyUnknown
    } else {
        CompatibilityStatusV1::Compatible
    }
}
fn generated_file(
    relative_path: &String,
    projection: &PublicReportProjection<'_>,
) -> ManifestGeneratedFileV1 {
    let (output_kind, output_id, format) = if relative_path == "public_summary.schema1.json" {
        (GeneratedOutputKindV1::Summary, None, RenderFormatV1::Json)
    } else if relative_path == "scientific_report.md" {
        (
            GeneratedOutputKindV1::Markdown,
            None,
            RenderFormatV1::Markdown,
        )
    } else if relative_path == "render_manifest.schema1.json" {
        (GeneratedOutputKindV1::Manifest, None, RenderFormatV1::Json)
    } else if relative_path.starts_with("tables/") {
        (
            GeneratedOutputKindV1::Table,
            relative_path
                .strip_prefix("tables/")
                .and_then(|name| name.strip_suffix(".csv"))
                .map(ToOwned::to_owned),
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
        source_input_flags: source_flags(relative_path, projection),
    }
}

fn source_flags(relative_path: &str, projection: &PublicReportProjection<'_>) -> Vec<InputFlagV1> {
    let all_supplied = || {
        let mut flags = vec![InputFlagV1::Mechanism, InputFlagV1::Health];
        for (present, flag) in [
            (projection.eis.is_some(), InputFlagV1::Eis),
            (projection.transient.is_some(), InputFlagV1::Transient),
            (projection.calibration.is_some(), InputFlagV1::Calibration),
            (
                projection.calibration_observations.is_some(),
                InputFlagV1::CalibrationObservations,
            ),
            (projection.signal.is_some(), InputFlagV1::Signal),
            (projection.estimation.is_some(), InputFlagV1::Estimation),
            (projection.model.is_some(), InputFlagV1::Model),
            (
                projection.lineage_catalog.is_some(),
                InputFlagV1::LineageCatalog,
            ),
        ] {
            if present {
                flags.push(flag);
            }
        }
        flags
    };
    let id = relative_path
        .strip_prefix("figures/")
        .and_then(|name| {
            name.strip_suffix(".svg")
                .or_else(|| name.strip_suffix(".png"))
        })
        .or_else(|| {
            relative_path
                .strip_prefix("tables/")
                .and_then(|name| name.strip_suffix(".csv"))
        });
    match id {
        Some("mechanism_timescale" | "mechanism_evidence" | "timescale_comparison") => {
            vec![InputFlagV1::Mechanism]
        }
        Some("sensor_health_dimension_status" | "health_dimensions" | "current_vs_baseline") => {
            vec![InputFlagV1::Health]
        }
        Some("evidence_provenance") => vec![InputFlagV1::Mechanism, InputFlagV1::Health],
        Some("eis_nyquist" | "eis_bode") => vec![InputFlagV1::Eis],
        Some("transient_response") => vec![InputFlagV1::Transient],
        Some("calibration_performance") => vec![
            InputFlagV1::Calibration,
            InputFlagV1::CalibrationObservations,
        ],
        Some("signal_diagnostics") => vec![InputFlagV1::Signal],
        Some("estimation_observed_predicted") => vec![InputFlagV1::Estimation],
        Some("model_observed_predicted" | "model_consistency") => vec![InputFlagV1::Model],
        Some("lineage" | "artifact_lineage") => all_supplied(),
        _ => all_supplied(),
    }
}
fn manifest_warnings(
    projection: &PublicReportProjection,
    unavailable: &[(String, AvailabilityReason)],
) -> Vec<ManifestWarningV1> {
    let mut warnings = Vec::new();
    for warning in &projection.mechanism.warnings {
        warnings.push(ManifestWarningV1 {
            code: WarningCodeV1::SourceWarning,
            message: warning.message.clone(),
            input_flag: Some(InputFlagV1::Mechanism),
            output_id: None,
        });
    }
    for warning in &projection.health.warnings {
        warnings.push(ManifestWarningV1 {
            code: WarningCodeV1::SourceWarning,
            message: enum_token(&HealthWarningV1::from(warning)),
            input_flag: Some(InputFlagV1::Health),
            output_id: None,
        });
    }
    if let Some(eis) = projection.eis {
        for warning in &eis.warnings {
            warnings.push(ManifestWarningV1 {
                code: WarningCodeV1::SourceWarning,
                message: warning.message.clone(),
                input_flag: Some(InputFlagV1::Eis),
                output_id: None,
            });
        }
    }
    if let Some(transient) = projection.transient {
        for event in &transient.events {
            for warning in &event.warnings {
                warnings.push(ManifestWarningV1 {
                    code: WarningCodeV1::SourceWarning,
                    message: warning.message.clone(),
                    input_flag: Some(InputFlagV1::Transient),
                    output_id: None,
                });
            }
            for fit in &event.candidate_fits {
                for warning in &fit.warnings {
                    warnings.push(ManifestWarningV1 {
                        code: WarningCodeV1::SourceWarning,
                        message: warning.message.clone(),
                        input_flag: Some(InputFlagV1::Transient),
                        output_id: None,
                    });
                }
            }
        }
    }
    if let Some(calibration) = projection.calibration {
        for warning in &calibration.warnings {
            warnings.push(ManifestWarningV1 {
                code: WarningCodeV1::SourceWarning,
                message: warning.message.clone(),
                input_flag: Some(InputFlagV1::Calibration),
                output_id: None,
            });
        }
    }
    if let Some(observations) = projection.calibration_observations {
        for warning in &observations.warnings {
            warnings.push(ManifestWarningV1 {
                code: WarningCodeV1::SourceWarning,
                message: warning.message.clone(),
                input_flag: Some(InputFlagV1::CalibrationObservations),
                output_id: None,
            });
        }
    }
    if let Some(signal) = projection.signal {
        for warning in &signal.warnings {
            warnings.push(ManifestWarningV1 {
                code: WarningCodeV1::SourceWarning,
                message: enum_token(warning),
                input_flag: Some(InputFlagV1::Signal),
                output_id: None,
            });
        }
    }
    if let Some(estimation) = projection.estimation {
        for warning in &estimation.warnings {
            warnings.push(ManifestWarningV1 {
                code: WarningCodeV1::SourceWarning,
                message: warning.message.clone(),
                input_flag: Some(InputFlagV1::Estimation),
                output_id: None,
            });
        }
    }
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
    for (id, reason) in unavailable {
        warnings.push(ManifestWarningV1 {
            code: WarningCodeV1::OutputUnavailable,
            message: format!(
                "{id} is unavailable: {}",
                crate::reporting::claims::unavailable_text(*reason)
            ),
            input_flag: None,
            output_id: Some(id.clone()),
        });
    }
    warnings
}
fn enum_token<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .expect("closed enum serialization")
        .trim_matches('"')
        .to_owned()
}
fn legacy_notices(projection: &PublicReportProjection) -> Vec<ManifestLegacyNoticeV1> {
    let mut notices = Vec::new();
    if projection.mechanism_is_legacy() {
        notices.push(ManifestLegacyNoticeV1 {
            input_flag: InputFlagV1::Mechanism,
            schema_version: projection.mechanism.schema_version,
            notice: LegacyNoticeV1::MechanismAssessmentNotSerialized,
        });
    }
    if projection.health_is_legacy() {
        notices.push(ManifestLegacyNoticeV1 {
            input_flag: InputFlagV1::Health,
            schema_version: projection.health.schema_version,
            notice: LegacyNoticeV1::PhaseCNotSerialized,
        });
    }
    for (flag, lineage) in projection.supplied_lineages() {
        if let crate::domain::ArtifactLineageState::LegacyUnknown {
            source_schema_version,
            ..
        } = lineage
        {
            notices.push(ManifestLegacyNoticeV1 {
                input_flag: input_flag(flag),
                schema_version: source_schema_version.unwrap_or_else(|| match flag {
                    "mechanism" => projection.mechanism.schema_version,
                    "health" => projection.health.schema_version,
                    "eis" => projection.eis.expect("supplied").schema_version,
                    "transient" => projection.transient.expect("supplied").schema_version,
                    "calibration" => projection.calibration.expect("supplied").schema_version,
                    "calibration_observations" => {
                        projection
                            .calibration_observations
                            .expect("supplied")
                            .schema_version
                    }
                    "signal" => projection.signal.expect("supplied").schema_version,
                    "estimation" => projection.estimation.expect("supplied").schema_version,
                    "model" => projection.model.expect("supplied").schema_version,
                    _ => unreachable!("fixed input flag"),
                }),
                notice: LegacyNoticeV1::LineageUnknown,
            });
        }
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
    let output_metadata = match fs::symlink_metadata(output) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(_) => {
            return Err(PublicReportError::InvalidOutputDirectory {
                path: output.to_path_buf(),
            });
        }
    };
    if output_metadata.file_type().is_symlink() || !output_metadata.is_dir() {
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
    let mut actual = Vec::new();
    collect_prior_files(root, root, &mut actual)?;
    actual.sort();
    let manifest_path = root.join("render_manifest.schema1.json");
    if !actual
        .iter()
        .any(|relative| relative == "render_manifest.schema1.json")
    {
        return Err(PublicReportError::UnmanagedOutputEntry {
            path: manifest_path,
        });
    }
    let bytes = fs::read(&manifest_path).map_err(|_| PublicReportError::UnmanagedOutputEntry {
        path: manifest_path.clone(),
    })?;
    let manifest: RenderManifestV1 =
        serde_json::from_slice(&bytes).map_err(|_| PublicReportError::UnmanagedOutputEntry {
            path: manifest_path.clone(),
        })?;
    if manifest.schema_version != 1
        || manifest.output_kind != "phase_d_render_manifest"
        || manifest.renderer_contract != "mhi_v1_phase_d_public_output_v1"
        || manifest.route != "electroanalysis report render"
        || !matches!(manifest.final_output_status, FinalOutputStatusV1::Published)
        || manifest.render_order.len() != manifest.generated_files.len()
    {
        return Err(PublicReportError::UnmanagedOutputEntry {
            path: manifest_path,
        });
    }
    let mut declared = Vec::new();
    let mut unique = BTreeSet::new();
    for (ordinal, (step, file)) in manifest
        .render_order
        .iter()
        .zip(&manifest.generated_files)
        .enumerate()
    {
        if step.ordinal != ordinal as u32
            || step.relative_path != file.relative_path
            || step.output_kind != file.output_kind
            || step.output_id != file.output_id
            || file.status != RenderStatusV1::Written
            || !recognized_generated_file(file)
            || !unique.insert(file.relative_path.clone())
        {
            return Err(PublicReportError::UnmanagedOutputEntry {
                path: root.join(&file.relative_path),
            });
        }
        declared.push(file.relative_path.clone());
    }
    declared.sort();
    if declared != actual {
        let mismatch = actual
            .iter()
            .find(|path| !declared.contains(path))
            .or_else(|| declared.iter().find(|path| !actual.contains(path)))
            .cloned()
            .unwrap_or_else(|| "render_manifest.schema1.json".into());
        return Err(PublicReportError::UnmanagedOutputEntry {
            path: root.join(mismatch),
        });
    }
    let has_json = declared
        .iter()
        .any(|path| path == "public_summary.schema1.json");
    let has_markdown = declared.iter().any(|path| path == "scientific_report.md");
    if !has_json && !has_markdown {
        return Err(PublicReportError::UnmanagedOutputEntry {
            path: root.to_path_buf(),
        });
    }
    Ok(())
}

fn collect_prior_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<String>,
) -> Result<(), PublicReportError> {
    for entry in fs::read_dir(directory).map_err(|_| PublicReportError::UnmanagedOutputEntry {
        path: directory.to_path_buf(),
    })? {
        let entry = entry.map_err(|_| PublicReportError::UnmanagedOutputEntry {
            path: directory.to_path_buf(),
        })?;
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|_| PublicReportError::UnmanagedOutputEntry { path: path.clone() })?;
        if kind.is_symlink() {
            return Err(PublicReportError::UnmanagedOutputEntry { path });
        }
        if kind.is_dir() {
            if directory != root
                || !matches!(entry.file_name().to_str(), Some("tables") | Some("figures"))
            {
                return Err(PublicReportError::UnmanagedOutputEntry { path });
            }
            collect_prior_files(root, &path, files)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| PublicReportError::UnmanagedOutputEntry { path: path.clone() })?
                .to_str()
                .ok_or_else(|| PublicReportError::UnmanagedOutputEntry { path: path.clone() })?
                .replace('\\', "/");
            files.push(relative);
        } else {
            return Err(PublicReportError::UnmanagedOutputEntry { path });
        }
    }
    Ok(())
}

fn recognized_generated_file(file: &ManifestGeneratedFileV1) -> bool {
    if file.relative_path.contains("..")
        || file.relative_path.starts_with('/')
        || file.relative_path.contains('\\')
    {
        return false;
    }
    match (
        file.output_kind,
        file.output_id.as_deref(),
        file.format,
        file.relative_path.as_str(),
    ) {
        (
            GeneratedOutputKindV1::Summary,
            None,
            RenderFormatV1::Json,
            "public_summary.schema1.json",
        )
        | (
            GeneratedOutputKindV1::Markdown,
            None,
            RenderFormatV1::Markdown,
            "scientific_report.md",
        )
        | (
            GeneratedOutputKindV1::Manifest,
            None,
            RenderFormatV1::Json,
            "render_manifest.schema1.json",
        ) => true,
        (GeneratedOutputKindV1::Table, Some(id), RenderFormatV1::Csv, path) => TableId::ALL
            .iter()
            .any(|table| id == table.as_str() && path == format!("tables/{}.csv", table.as_str())),
        (GeneratedOutputKindV1::Figure, Some(id), format, path) => {
            FigureId::ALL.iter().any(|figure| {
                if id != figure.as_str() {
                    return false;
                }
                match format {
                    RenderFormatV1::Svg => path == format!("figures/{}.svg", figure.as_str()),
                    RenderFormatV1::Png => path == format!("figures/{}.png", figure.as_str()),
                    _ => false,
                }
            })
        }
        _ => false,
    }
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
    let backup = (0_u32..1000)
        .map(|attempt| {
            parent.join(format!(
                ".{name}.phase-d-backup-{}-{attempt}",
                process::id()
            ))
        })
        .find(|path| {
            fs::symlink_metadata(path).is_err_and(|source| source.kind() == io::ErrorKind::NotFound)
        })
        .ok_or_else(|| PublicReportError::Publication {
            phase: crate::reporting::PublicationPhase::BackupRename,
            staging_path: staging.to_path_buf(),
            backup_path: None,
            source: io::Error::new(io::ErrorKind::AlreadyExists, "exhausted backup paths"),
        })?;
    fs::rename(output, &backup).map_err(|source| PublicReportError::Publication {
        phase: crate::reporting::PublicationPhase::BackupRename,
        staging_path: staging.to_path_buf(),
        backup_path: Some(backup.clone()),
        source,
    })?;
    let publish_rename =
        injected_rename_failure("ELECTROANALYSIS_PHASE_D_TEST_FAIL_PUBLISH_RENAME")
            .map_or_else(|| fs::rename(staging, output), Err);
    if let Err(source) = publish_rename {
        let restore_rename =
            injected_rename_failure("ELECTROANALYSIS_PHASE_D_TEST_FAIL_RESTORE_RENAME")
                .map_or_else(|| fs::rename(&backup, output), Err);
        if let Err(restore_source) = restore_rename {
            return Err(PublicReportError::Publication {
                phase: crate::reporting::PublicationPhase::RestoreRename,
                staging_path: staging.to_path_buf(),
                backup_path: Some(backup),
                source: restore_source,
            });
        }
        return Err(PublicReportError::Publication {
            phase: crate::reporting::PublicationPhase::PublishRename,
            staging_path: staging.to_path_buf(),
            backup_path: None,
            source,
        });
    }
    remove_noncertified(&backup).map_err(|source| PublicReportError::Cleanup {
        path: backup,
        source,
    })
}

fn remove_noncertified(path: &Path) -> io::Result<()> {
    #[cfg(debug_assertions)]
    if let Some(target) = std::env::var_os("ELECTROANALYSIS_PHASE_D_TEST_FAIL_CLEANUP") {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let target = target.to_string_lossy();
        if (target == "staging" && name.contains("phase-d-staging"))
            || (target == "backup" && name.contains("phase-d-backup"))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected Phase-D cleanup failure",
            ));
        }
    }
    fs::remove_dir_all(path)
}

#[cfg(debug_assertions)]
fn injected_rename_failure(variable: &str) -> Option<io::Error> {
    std::env::var_os(variable).map(|_| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("injected Phase-D rename failure from {variable}"),
        )
    })
}

#[cfg(not(debug_assertions))]
fn injected_rename_failure(_variable: &str) -> Option<io::Error> {
    None
}
