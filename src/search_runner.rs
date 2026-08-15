//! EIS equivalent-circuit search pipeline.
//!
//! This module enumerates candidate input paths, excludes known
//! application-generated artifacts, and delegates physical-format, container,
//! worksheet, CHI/EIS header, and role recognition to `electrodata-io`. The
//! provider owns raw CSV/TXT/DAT/XLSX reading, content-aware unusual-extension
//! and binary detection, canonical DatasetKind/ColumnRole assignment,
//! malformed-row recovery, diagnostics, and provenance. Canonical EIS datasets
//! are then checked for scientific ECM-search suitability and passed to the
//! search/reporting and optional ranked-model plotting workflow.
//! Analysis-artifact reads remain consumer-owned because they are versioned
//! result contracts, not physical measurements.

use crate::{
    data_file::chi_file::EISData,
    impedance::discover_equivalent_circuits_with_config,
    plot_config::{LoadedPlotConfig, PlotConfig, PlotJob, PlotJobKind, RenderConfig},
    plottings::{
        PlotAxisScale, PlotSeries, PlotSeriesKind, PublicationConfig, best_ranked_search_fit,
        eis_combined_publication_config, plot_ranked_search_report, plot_rendered_series_hq,
    },
    search_config::{LoadedEcmSearchConfig, RuntimeEcmSearchConfig},
};
use crate::{
    domain::BatchFileFailure,
    runners::{BatchRunSummary, RunnerError},
};
use std::fs;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Files accepted by canonical ingestion together with typed per-file
/// failures. The underlying `electrodata_io::Error` is never flattened.
#[derive(Debug)]
pub struct SearchInputCollection {
    pub files: Vec<PathBuf>,
    pub failures: Vec<BatchFileFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchLogLevel {
    Info,
    Warning,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the full ECM search pipeline for a single EIS file or every eligible
/// file inside a directory.
///
/// For each canonical EIS input the function:
/// 1. Uses [`EISData::parse_file`], whose file read delegates to
///    `electrodata-io`.
/// 2. Runs `discover_equivalent_circuits_with_config`.
/// 3. Writes a plain-text report and a CSV ranking table.
/// 4. Optionally renders ranked-model plots when `plot_top_n > 0`.
pub fn run_eis_search(
    workspace_dir: &Path,
    search_target: &Path,
    sheet: Option<&str>,
    search_config_path: Option<&Path>,
    search_output: Option<&Path>,
    search_top: Option<usize>,
) -> Result<(), RunnerError> {
    let loaded_search_config = RuntimeEcmSearchConfig::load(workspace_dir, search_config_path)?;
    run_eis_search_with_loaded_config(
        workspace_dir,
        search_target,
        sheet,
        loaded_search_config,
        search_output,
        search_top,
        None,
        |level, message| match level {
            SearchLogLevel::Info => println!("{message}"),
            SearchLogLevel::Warning => eprintln!("{message}"),
        },
    )
}

#[allow(clippy::too_many_arguments)] // stable workflow API; sheet is canonical provider selection.
pub fn run_eis_search_with_loaded_config<F>(
    workspace_dir: &Path,
    search_target: &Path,
    sheet: Option<&str>,
    loaded_search_config: LoadedEcmSearchConfig,
    search_output: Option<&Path>,
    search_top: Option<usize>,
    search_plot_config_override: Option<PublicationConfig>,
    mut log: F,
) -> Result<(), RunnerError>
where
    F: FnMut(SearchLogLevel, &str),
{
    let target = resolve_cli_path(workspace_dir, search_target);
    let input_collection = collect_eis_search_inputs(&target, sheet)?;
    let input_files = input_collection.files;
    let mut failures = input_collection.failures;
    let output_path = search_output.map(|path| resolve_cli_path(workspace_dir, path));
    let search_config = loaded_search_config
        .config
        .resolve_search_config(search_top);
    let plot_top_n = loaded_search_config.config.resolved_plot_top_n();
    let configured_plot_dir = loaded_search_config
        .config
        .resolve_plot_output_dir(&loaded_search_config.base_dir);
    let search_plot_config = if plot_top_n > 0 {
        search_plot_config_override.unwrap_or_else(|| {
            resolve_search_plot_publication_config(workspace_dir, &target, &mut log)
        })
    } else {
        eis_combined_publication_config()
    };
    let mut combined_search_nyquist_series: Vec<Vec<PlotSeries>> = Vec::new();

    if let Some(source_path) = loaded_search_config.source_path.as_ref() {
        emit_info(
            &mut log,
            format!("Search config: {}", source_path.display()),
        );
        emit_info(&mut log, "");
    }
    for warning in &loaded_search_config.warnings {
        emit_warning(&mut log, format!("search config warning: {warning}"));
    }

    // Report skipped files when processing a whole directory.
    if target.is_dir() && !failures.is_empty() {
        emit_info(
            &mut log,
            format!(
                "Skipping {} ignored file(s) in {}:",
                failures.len(),
                target.display()
            ),
        );
        for failure in failures.iter().take(8) {
            let name = failure
                .path()
                .file_name()
                .and_then(|v| v.to_str())
                .map(|v| v.to_string())
                .unwrap_or_else(|| failure.path().to_string_lossy().into_owned());
            emit_info(&mut log, format!("  {} ({failure})", name));
        }
        if failures.len() > 8 {
            emit_info(&mut log, format!("  ... and {} more", failures.len() - 8));
        }
        emit_info(&mut log, "");
    }

    let mut successful_inputs = Vec::new();
    let mut resolved_outputs = BTreeMap::new();
    for input_file in input_files {
        let data = match EISData::parse_file_with_sheet(&input_file, sheet) {
            Ok(data) => data,
            Err(error) => {
                let failure = BatchFileFailure::canonical(input_file, error);
                emit_warning(&mut log, format!("EIS search input failure: {failure}"));
                failures.push(failure);
                continue;
            }
        };
        let report = discover_equivalent_circuits_with_config(
            &data.freq,
            &data.z_re,
            &data.z_im,
            &data.phase,
            &search_config,
        )?;

        emit_info(&mut log, format!("EIS Search: {}", input_file.display()));
        emit_info(&mut log, format!("Label: {}", data.label));
        emit_info(&mut log, report.summary());
        emit_info(&mut log, "");
        emit_info(&mut log, report.ranking_table());
        emit_info(&mut log, "");

        // Resolve and reserve every per-input artifact before writing.  This
        // makes a future naming regression a structured failure, never an
        // overwrite of scientific output.
        let export_path =
            resolve_search_export_path(&input_file, output_path.as_deref(), target.is_dir())?;
        let csv_export_path = resolve_search_csv_export_path(&export_path);
        reserve_search_output(&mut resolved_outputs, &export_path, &input_file)?;
        reserve_search_output(&mut resolved_outputs, &csv_export_path, &input_file)?;
        let plot_output_base = (plot_top_n > 0).then(|| {
            resolve_search_plot_output_base(
                &input_file,
                configured_plot_dir.as_deref(),
                target.is_dir(),
            )
        });
        if let Some(base) = &plot_output_base {
            reserve_search_output(&mut resolved_outputs, base, &input_file)?;
        }

        // Write text and CSV reports.
        report.export_detailed_report(&export_path)?;
        report.export_ranking_csv(&csv_export_path)?;
        emit_info(
            &mut log,
            format!("Search report written to: {}", export_path.display()),
        );
        emit_info(
            &mut log,
            format!("Search CSV written to: {}", csv_export_path.display()),
        );
        successful_inputs.push(input_file.clone());

        // Optionally render plots for the top-N candidates.
        if plot_top_n > 0 {
            if let Some(best_fit) = best_ranked_search_fit(&report) {
                combined_search_nyquist_series.push(pair_dataset_experimental_and_fitted_colors(
                    data.nyquist_series_for_fit(&best_fit),
                ));
            }

            let plot_output_base = plot_output_base.expect("plot output reserved when plotting");
            if let Some(parent) = plot_output_base.parent() {
                fs::create_dir_all(parent)?;
            }
            let plot_outcome = plot_ranked_search_report(
                &data,
                &report,
                &plot_output_base,
                &search_plot_config,
                plot_top_n,
            )?;
            emit_info(
                &mut log,
                format!(
                    "Top-{} search plots written to base path: {}",
                    plot_outcome.plotted_candidates,
                    plot_outcome.output_base.display()
                ),
            );
            for (index, path) in plot_outcome.individual_output_bases.iter().enumerate() {
                emit_info(
                    &mut log,
                    format!(
                        "  Rank {} individual plots written to base path: {}",
                        index + 1,
                        path.display()
                    ),
                );
            }
        }
        emit_info(&mut log, "");
    }

    if successful_inputs.is_empty() {
        return if failures.is_empty() {
            Err(RunnerError::NoInputCandidates {
                workflow: "EIS search",
                input_dir: target,
            })
        } else {
            Err(RunnerError::BatchInput { failures })
        };
    }

    if plot_top_n > 0 && combined_search_nyquist_series.len() > 1 {
        let combined_plot_base = resolve_search_combined_plot_output_base(
            &target,
            configured_plot_dir.as_deref(),
            target.is_dir(),
        );
        if let Some(parent) = combined_plot_base.parent() {
            fs::create_dir_all(parent)?;
        }

        let combined_plot_config = search_plot_config
            .clone()
            .with_default_axis_labels("Z' (Ohm)", "-Z'' (Ohm)");
        plot_rendered_series_hq(
            combined_plot_base.to_string_lossy().as_ref(),
            &combined_search_nyquist_series,
            &combined_plot_config,
            true,
            PlotAxisScale::Linear,
        )?;

        emit_info(
            &mut log,
            format!(
                "Combined search overlay plot written to base path: {}",
                combined_plot_base.display()
            ),
        );
        emit_info(&mut log, "");
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(RunnerError::partial_batch(BatchRunSummary {
            successful_inputs,
            failures,
        }))
    }
}

fn emit_info(log: &mut dyn FnMut(SearchLogLevel, &str), message: impl Into<String>) {
    let message = message.into();
    log(SearchLogLevel::Info, message.as_str());
}

fn emit_warning(log: &mut dyn FnMut(SearchLogLevel, &str), message: impl Into<String>) {
    let message = message.into();
    log(SearchLogLevel::Warning, message.as_str());
}

// ---------------------------------------------------------------------------
// Private helpers – path resolution
// ---------------------------------------------------------------------------

/// Resolve a path supplied on the CLI: absolute paths are used as-is; relative
/// paths are resolved against `workspace_dir` (the process working directory).
fn resolve_cli_path(workspace_dir: &Path, input: &Path) -> PathBuf {
    if input.is_absolute() {
        input.to_path_buf()
    } else {
        workspace_dir.join(input)
    }
}

/// Determine the output path for the plain-text ECM search report.
///
/// When `multi_input_search` is true the `configured_output` is treated as a
/// directory; otherwise it may be a concrete file path.
fn resolve_search_export_path(
    input_file: &Path,
    configured_output: Option<&Path>,
    multi_input_search: bool,
) -> Result<PathBuf, RunnerError> {
    let default_name = format!("{}_ecm_search.txt", search_input_identity(input_file));

    match configured_output {
        Some(output_path) if multi_input_search => Ok(output_path.join(default_name)),
        Some(output_path) => {
            if output_path.extension().is_none() {
                Ok(output_path.join(default_name))
            } else {
                Ok(output_path.to_path_buf())
            }
        }
        None => Ok(input_file.with_file_name(default_name)),
    }
}

/// Derive the base path for per-file search plots from the input file and an
/// optional configured output directory.
fn resolve_search_plot_output_base(
    input_file: &Path,
    configured_output_dir: Option<&Path>,
    multi_input_search: bool,
) -> PathBuf {
    let identity = search_input_identity(input_file);

    match configured_output_dir {
        Some(output_dir) if multi_input_search => {
            output_dir.join(identity).join("ecm_search_top_models")
        }
        Some(output_dir) => output_dir.join("ecm_search_top_models"),
        None => input_file.with_file_name(format!("{identity}_ecm_search_top_models")),
    }
}

/// Derive the base path for the cross-file combined search overlay.
fn resolve_search_combined_plot_output_base(
    search_target: &Path,
    configured_output_dir: Option<&Path>,
    multi_input_search: bool,
) -> PathBuf {
    match configured_output_dir {
        Some(output_dir) => output_dir.join("combined").join("ecm_search_all_datasets"),
        None if multi_input_search => search_target
            .join("combined")
            .join("ecm_search_all_datasets"),
        None => {
            let stem = search_target
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or("eis_search");
            search_target.with_file_name(format!("{stem}_ecm_search_all_datasets"))
        }
    }
}

/// Derive the CSV ranking export path by replacing the `.txt` extension of the
/// plain-text report path with `.csv`.
fn resolve_search_csv_export_path(text_report_path: &Path) -> PathBuf {
    text_report_path.with_extension("csv")
}

/// Human-readable, deterministic identity for one physical source.  The
/// extension is part of provenance: `eis.csv` and `eis.xlsx` must never share
/// an analysis artifact name.
fn search_input_identity(input_file: &Path) -> String {
    let stem = input_file
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("eis_search");
    let extension = input_file
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("input");
    format!(
        "{}__{}",
        sanitize_output_component(stem),
        sanitize_output_component(extension)
    )
}

fn sanitize_output_component(value: &str) -> String {
    let result = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if result.is_empty() {
        "input".to_string()
    } else {
        result
    }
}

fn reserve_search_output(
    reserved: &mut BTreeMap<PathBuf, PathBuf>,
    output: &Path,
    input: &Path,
) -> Result<(), RunnerError> {
    if let Some(first_input) = reserved.get(output) {
        return Err(RunnerError::OutputCollision {
            output: output.to_path_buf(),
            first_input: first_input.clone(),
            second_input: input.to_path_buf(),
        });
    }
    reserved.insert(output.to_path_buf(), input.to_path_buf());
    Ok(())
}

fn pair_dataset_experimental_and_fitted_colors(series: Vec<PlotSeries>) -> Vec<PlotSeries> {
    series
        .into_iter()
        .map(|mut item| {
            if item.kind == PlotSeriesKind::Fitted {
                // Reuse the paired-color rendering path so each dataset's fit
                // inherits the same palette color as its experimental points.
                item.kind = PlotSeriesKind::RegressionFit;
                item.fit_info = None;
            }
            item
        })
        .collect()
}

fn resolve_search_plot_publication_config(
    workspace_dir: &Path,
    search_target: &Path,
    log: &mut dyn FnMut(SearchLogLevel, &str),
) -> PublicationConfig {
    let loaded_plot_config = match PlotConfig::load(workspace_dir, None) {
        Ok(config) => config,
        Err(error) => {
            emit_warning(
                log,
                format!(
                    "failed to load plotting config for search plotting, using defaults: {}",
                    error
                ),
            );
            return eis_combined_publication_config();
        }
    };

    resolve_search_plot_publication_config_from_loaded(
        workspace_dir,
        search_target,
        &loaded_plot_config,
        log,
    )
}

pub fn resolve_search_plot_publication_config_from_loaded(
    workspace_dir: &Path,
    search_target: &Path,
    loaded_plot_config: &LoadedPlotConfig,
    log: &mut dyn FnMut(SearchLogLevel, &str),
) -> PublicationConfig {
    let default_config = eis_combined_publication_config();

    for warning in &loaded_plot_config.warnings {
        emit_warning(log, format!("plot config warning: {}", warning));
    }

    let render_applied = match apply_render_config_to_publication(
        &default_config,
        loaded_plot_config.render_config(),
    ) {
        Ok(config) => config,
        Err(error) => {
            emit_warning(
                log,
                format!(
                    "invalid [render] plot settings for search plotting, using defaults: {}",
                    error
                ),
            );
            return default_config;
        }
    };

    let jobs = match loaded_plot_config.resolve_jobs(PlotJobKind::Eis, workspace_dir) {
        Ok(resolved_jobs) => resolved_jobs,
        Err(error) => {
            emit_warning(
                log,
                format!(
                    "failed to resolve EIS plot job styles for search plotting, using defaults: {}",
                    error
                ),
            );
            return render_applied;
        }
    };

    let Some(selected_job) = select_matching_eis_plot_job(&jobs, search_target) else {
        return render_applied;
    };

    match selected_job.style.apply_to_combined(&render_applied) {
        Ok(config) => config,
        Err(error) => {
            emit_warning(
                log,
                format!(
                    "failed to apply EIS combined style for search plotting, using defaults: {}",
                    error
                ),
            );
            render_applied
        }
    }
}

fn select_matching_eis_plot_job<'a>(
    jobs: &'a [PlotJob],
    search_target: &Path,
) -> Option<&'a PlotJob> {
    if jobs.is_empty() {
        return None;
    }

    let target_path = search_target.to_path_buf();
    let target_dir = if search_target.is_dir() {
        search_target.to_path_buf()
    } else {
        search_target
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| search_target.to_path_buf())
    };

    jobs.iter()
        .filter(|job| {
            target_path.starts_with(&job.input_dir) || target_dir.starts_with(&job.input_dir)
        })
        .max_by_key(|job| job.input_dir.components().count())
        .or_else(|| jobs.first())
}

fn apply_render_config_to_publication(
    base: &PublicationConfig,
    render: &RenderConfig,
) -> Result<PublicationConfig, crate::domain::ConfigurationError> {
    let mut config = base.clone();
    if let Some(scale) = render.png_scale_factor {
        if scale == 0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid render.png_scale_factor: expected a value >= 1",
            ));
        }
        config.png_scale_factor = scale;
    }
    if let Some(dpi) = render.png_dpi {
        if !dpi.is_finite() || dpi <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid render.png_dpi: expected a positive finite value",
            ));
        }
        config.dpi = dpi;
    }
    Ok(config)
}

// ---------------------------------------------------------------------------
// Private helpers – file discovery
// ---------------------------------------------------------------------------

/// Walk `target` (a single file **or** a directory), read every non-artifact
/// file through the canonical reader, then accept only impedance datasets.
fn collect_eis_search_inputs(
    target: &Path,
    sheet: Option<&str>,
) -> Result<SearchInputCollection, RunnerError> {
    if !target.exists() {
        return Err(format!("Search target does not exist: {}", target.display()).into());
    }

    // A single file follows the same canonical read/worksheet-selection path
    // as directory discovery. This preserves provider errors and prevents a
    // workbook from being rejected before its explicit --sheet reaches it.
    if target.is_file() {
        return match crate::data_file::read_dataset_with_sheet(target, sheet) {
            Ok(dataset) if dataset.kind() == electrodata_io::DatasetKind::ImpedanceSpectrum => {
                Ok(SearchInputCollection {
                    files: vec![target.to_path_buf()],
                    failures: Vec::new(),
                })
            }
            Ok(dataset) => Ok(SearchInputCollection {
                files: Vec::new(),
                failures: vec![BatchFileFailure::canonical(
                    target,
                    dataset
                        .eis_view()
                        .expect_err("non-EIS dataset must reject EIS view")
                        .into(),
                )],
            }),
            Err(error) => Ok(SearchInputCollection {
                files: Vec::new(),
                failures: vec![BatchFileFailure::canonical(target, error)],
            }),
        };
    }

    if !target.is_dir() {
        return Err(format!(
            "Search target is neither a file nor a directory: {}",
            target.display()
        )
        .into());
    }

    // Read and sort directory entries for deterministic ordering.
    let mut entries = fs::read_dir(target)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    let mut files = Vec::new();
    let mut failures = Vec::new();

    for path in entries
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
    {
        if is_generated_search_artifact(&path) {
            failures.push(BatchFileFailure::rejected(
                path,
                "application-generated search artifact",
            ));
            continue;
        }
        match crate::data_file::read_dataset_with_sheet(&path, sheet) {
            Ok(dataset) if dataset.kind() == electrodata_io::DatasetKind::ImpedanceSpectrum => {
                files.push(path)
            }
            Ok(dataset) => failures.push(BatchFileFailure::canonical(
                path,
                dataset
                    .eis_view()
                    .expect_err("non-EIS dataset must reject EIS view")
                    .into(),
            )),
            Err(error) => failures.push(BatchFileFailure::canonical(path, error)),
        }
    }

    Ok(SearchInputCollection { files, failures })
}

/// Application artifacts can be excluded without interpreting physical raw
/// formats. All other regular files proceed to canonical ingestion.
fn is_generated_search_artifact(path: &Path) -> bool {
    let stem = path
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if stem.ends_with("_ecm_search") {
        return true;
    }
    if stem.contains("fit_report") {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{
        collect_eis_search_inputs, pair_dataset_experimental_and_fitted_colors,
        resolve_search_combined_plot_output_base, resolve_search_export_path,
        resolve_search_plot_output_base,
    };
    use crate::{
        domain::{BatchFileFailure, DataParsingError},
        plottings::{PlotSeries, PlotSeriesKind},
        runners::RunnerError,
        search_config::RuntimeEcmSearchConfig,
    };
    use std::{
        fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn paired_color_helper_keeps_experimental_and_converts_fitted_series_kind() {
        let original = vec![
            PlotSeries::experimental("dataset A".to_string(), vec![(1.0, 2.0)]),
            PlotSeries::fitted("dataset A fit".to_string(), vec![(1.0, 2.1)]),
        ];

        let transformed = pair_dataset_experimental_and_fitted_colors(original);

        assert_eq!(transformed[0].kind, PlotSeriesKind::Experimental);
        assert_eq!(transformed[1].kind, PlotSeriesKind::RegressionFit);
    }

    #[test]
    fn combined_search_overlay_path_prefers_configured_output_directory() {
        let base = resolve_search_combined_plot_output_base(
            Path::new("/tmp/eis_inputs"),
            Some(Path::new("/tmp/search_plots")),
            true,
        );
        assert_eq!(
            base,
            Path::new("/tmp/search_plots")
                .join("combined")
                .join("ecm_search_all_datasets")
        );
    }

    #[test]
    fn same_stem_inputs_have_distinct_report_and_plot_paths() {
        let output = Path::new("/tmp/search-output");
        let csv = Path::new("/tmp/eis.csv");
        let xlsx = Path::new("/tmp/eis.xlsx");
        assert_ne!(
            resolve_search_export_path(csv, Some(output), true).unwrap(),
            resolve_search_export_path(xlsx, Some(output), true).unwrap()
        );
        assert_ne!(
            resolve_search_plot_output_base(csv, Some(output), true),
            resolve_search_plot_output_base(xlsx, Some(output), true)
        );
    }

    #[test]
    fn report_and_csv_path_collisions_are_structured_errors() {
        let report = resolve_search_export_path(
            Path::new("/tmp/eis.csv"),
            Some(Path::new("/tmp/output.csv")),
            false,
        )
        .unwrap();
        assert_eq!(report, Path::new("/tmp/output.csv"));
        assert_eq!(super::resolve_search_csv_export_path(&report), report);
        let input = Path::new("/tmp/eis.csv");
        let mut reserved = std::collections::BTreeMap::new();
        super::reserve_search_output(&mut reserved, &report, input).unwrap();
        assert!(matches!(
            super::reserve_search_output(&mut reserved, &report, input),
            Err(RunnerError::OutputCollision { .. })
        ));
    }

    #[test]
    fn discovery_passes_explicit_worksheet_to_canonical_reader() {
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xlsx/multi_timeseries.xlsx");
        let without_sheet = collect_eis_search_inputs(&fixture, None).expect("discovery");
        assert_eq!(without_sheet.failures.len(), 1);
        let with_sheet = collect_eis_search_inputs(&fixture, Some("SheetA")).expect("discovery");
        assert_eq!(with_sheet.failures.len(), 1);
        assert!(
            matches!(
                with_sheet.failures.as_slice(),
                [BatchFileFailure::Canonical {
                    source: DataParsingError::ElectrodataIo(source),
                    ..
                }] if provider_contains(source.as_ref(), is_wrong_view)
            ),
            "unexpected typed wrong-view failure: {:?}",
            with_sheet.failures
        );

        // The no-sheet path retains the provider's ambiguity rather than
        // probing workbook content locally.
        assert!(matches!(
            without_sheet.failures.as_slice(),
            [BatchFileFailure::Canonical {
                source: DataParsingError::ElectrodataIo(source),
                ..
            }] if provider_contains(source.as_ref(), is_ambiguous_worksheet)
        ));
    }

    #[test]
    fn discovery_retains_typed_binary_and_unknown_format_provider_errors() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("batch-provider-errors-{nonce}"));
        fs::create_dir_all(&directory).expect("create directory");
        fs::write(directory.join("binary.csv"), [0_u8, 159, 146, 150])
            .expect("write binary fixture");
        fs::write(directory.join("unknown.txt"), "not a scientific table")
            .expect("write text fixture");

        let collection = collect_eis_search_inputs(&directory, None).expect("discovery");
        assert!(collection.files.is_empty());
        assert!(collection.failures.iter().any(|failure| matches!(
            failure,
            BatchFileFailure::Canonical {
                source: DataParsingError::ElectrodataIo(source),
                ..
            } if provider_contains(source.as_ref(), is_unsupported_binary)
        )));
        assert!(collection.failures.iter().any(|failure| matches!(
            failure,
            BatchFileFailure::Canonical {
                source: DataParsingError::ElectrodataIo(source),
                ..
            } if provider_contains(source.as_ref(), is_unknown_format)
        )));
        fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn generated_search_artifacts_are_rejected_before_canonical_ingestion() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("generated-search-artifact-{nonce}"));
        fs::create_dir_all(&directory).expect("create directory");
        let artifact = directory.join("sample_ecm_search.csv");
        fs::write(&artifact, "this is deliberately not a physical input")
            .expect("write generated artifact fixture");

        let collection = collect_eis_search_inputs(&directory, None).expect("discovery");
        assert!(collection.files.is_empty());
        assert!(matches!(
            collection.failures.as_slice(),
            [BatchFileFailure::Rejected { path, reason }]
                if path == &artifact && reason == "application-generated search artifact"
        ));
        fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn mixed_search_batch_returns_typed_partial_error_after_writing_outputs() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mixed-search-batch-{nonce}"));
        let input = root.join("input");
        let output = root.join("output");
        fs::create_dir_all(&input).expect("create input directory");
        fs::write(
            input.join("valid.csv"),
            "Freq/Hz,Z'/ohm,Z\"/ohm,Phase/deg\n1000,10,-1,-5.7\n100,15,-5,-18.4\n10,20,-10,-26.6\n",
        )
        .expect("write EIS fixture");
        fs::write(input.join("binary.csv"), [0_u8, 159, 146, 150]).expect("write binary");
        let config = RuntimeEcmSearchConfig {
            max_ranked_results: Some(1),
            evolution: crate::search_config::RawEvolutionConfig {
                population_size: Some(8),
                generation_limit: Some(1),
                num_individuals_per_parents: Some(2),
                selection_ratio: Some(0.7),
                mutation_rate: Some(0.2),
                reinsertion_ratio: Some(0.75),
                ranking_criterion: None,
            },
            ..Default::default()
        };
        let loaded = crate::search_config::LoadedEcmSearchConfig {
            config,
            base_dir: root.clone(),
            source_path: None,
            warnings: Vec::new(),
        };
        let error = super::run_eis_search_with_loaded_config(
            &root,
            &input,
            None,
            loaded,
            Some(&output),
            Some(1),
            None,
            |_, _| {},
        )
        .expect_err("mixed batch must not report full success");
        match error {
            RunnerError::PartialBatch {
                successful_count,
                failure_count,
                summary,
            } => {
                assert_eq!(successful_count, 1);
                assert_eq!(failure_count, 1);
                assert_eq!(summary.successful_inputs, vec![input.join("valid.csv")]);
                assert!(summary.failures.iter().any(|failure| matches!(
                    failure,
                    BatchFileFailure::Canonical {
                        source: DataParsingError::ElectrodataIo(source),
                        ..
                    } if provider_contains(source.as_ref(), is_unsupported_binary)
                )));
            }
            other => panic!("expected typed partial batch error, got {other:?}"),
        }
        assert!(output.join("valid__csv_ecm_search.txt").is_file());
        assert!(output.join("valid__csv_ecm_search.csv").is_file());
        fs::remove_dir_all(root).ok();
    }

    fn provider_contains(
        error: &electrodata_io::Error,
        predicate: fn(&electrodata_io::Error) -> bool,
    ) -> bool {
        predicate(error)
            || matches!(error, electrodata_io::Error::ReadContext { source, .. } if provider_contains(source, predicate))
    }

    fn is_wrong_view(error: &electrodata_io::Error) -> bool {
        matches!(
            error,
            electrodata_io::Error::InvalidDatasetView { .. }
                | electrodata_io::Error::WrongDatasetKind { .. }
        )
    }

    fn is_ambiguous_worksheet(error: &electrodata_io::Error) -> bool {
        matches!(error, electrodata_io::Error::AmbiguousWorksheet { .. })
    }

    fn is_unsupported_binary(error: &electrodata_io::Error) -> bool {
        matches!(error, electrodata_io::Error::UnsupportedBinary { .. })
    }

    fn is_unknown_format(error: &electrodata_io::Error) -> bool {
        matches!(
            error,
            electrodata_io::Error::UnknownFormat { .. }
                | electrodata_io::Error::UnknownFormatDetailed { .. }
        )
    }
}
