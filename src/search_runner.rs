//! EIS equivalent-circuit search pipeline.
//!
//! This module is responsible for:
//! * discovering eligible EIS data files in a file-system target,
//! * validating each file against the expected CHI EIS header,
//! * orchestrating the ECM search and writing text / CSV reports, and
//! * rendering optional ranked-model plots via the plotting layer.

use crate::runners::RunnerError;
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
use std::fs;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Files that will be passed to the search together with those that were
/// skipped (each paired with a human-readable reason string).
#[derive(Debug, Clone)]
pub struct SearchInputCollection {
    pub files: Vec<PathBuf>,
    pub skipped: Vec<(PathBuf, String)>,
}

/// Per-file classification result used by [`collect_eis_search_inputs`].
#[derive(Debug, Clone, PartialEq, Eq)]
enum SearchInputDecision {
    Include,
    Skip(&'static str),
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
/// For each file the function:
/// 1. Parses EIS data via [`EISData::parse_file`].
/// 2. Runs `discover_equivalent_circuits_with_config`.
/// 3. Writes a plain-text report and a CSV ranking table.
/// 4. Optionally renders ranked-model plots when `plot_top_n > 0`.
pub fn run_eis_search(
    workspace_dir: &Path,
    search_target: &Path,
    search_config_path: Option<&Path>,
    search_output: Option<&Path>,
    search_top: Option<usize>,
) -> Result<(), RunnerError> {
    let loaded_search_config = RuntimeEcmSearchConfig::load(workspace_dir, search_config_path)?;
    run_eis_search_with_loaded_config(
        workspace_dir,
        search_target,
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

pub fn run_eis_search_with_loaded_config<F>(
    workspace_dir: &Path,
    search_target: &Path,
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
    let input_collection = collect_eis_search_inputs(&target)?;
    let input_files = input_collection.files;
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
    if target.is_dir() && !input_collection.skipped.is_empty() {
        emit_info(
            &mut log,
            format!(
                "Skipping {} ignored file(s) in {}:",
                input_collection.skipped.len(),
                target.display()
            ),
        );
        for (path, reason) in input_collection.skipped.iter().take(8) {
            let name = path
                .file_name()
                .and_then(|v| v.to_str())
                .map(|v| v.to_string())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            emit_info(&mut log, format!("  {} ({})", name, reason));
        }
        if input_collection.skipped.len() > 8 {
            emit_info(
                &mut log,
                format!("  ... and {} more", input_collection.skipped.len() - 8),
            );
        }
        emit_info(&mut log, "");
    }

    for input_file in input_files {
        let data = EISData::parse_file(&input_file)?;
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

        // Write text and CSV reports.
        let export_path =
            resolve_search_export_path(&input_file, output_path.as_deref(), target.is_dir())?;
        report.export_detailed_report(&export_path)?;
        let csv_export_path = resolve_search_csv_export_path(&export_path);
        report.export_ranking_csv(&csv_export_path)?;
        emit_info(
            &mut log,
            format!("Search report written to: {}", export_path.display()),
        );
        emit_info(
            &mut log,
            format!("Search CSV written to: {}", csv_export_path.display()),
        );

        // Optionally render plots for the top-N candidates.
        if plot_top_n > 0 {
            if let Some(best_fit) = best_ranked_search_fit(&report) {
                combined_search_nyquist_series.push(pair_dataset_experimental_and_fitted_colors(
                    data.nyquist_series_for_fit(&best_fit),
                ));
            }

            let plot_output_base = resolve_search_plot_output_base(
                &input_file,
                configured_plot_dir.as_deref(),
                target.is_dir(),
            );
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

    Ok(())
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
    let default_name = format!(
        "{}_ecm_search.txt",
        input_file
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or("eis_search")
    );

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
    let stem = input_file
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or("eis_search");

    match configured_output_dir {
        Some(output_dir) if multi_input_search => {
            output_dir.join(stem).join("ecm_search_top_models")
        }
        Some(output_dir) => output_dir.join("ecm_search_top_models"),
        None => input_file.with_file_name(format!("{stem}_ecm_search_top_models")),
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

/// Walk `target` (a single file **or** a directory) and return the files that
/// are eligible for EIS search together with those that were skipped.
///
/// Returns an error when the target does not exist or when no eligible files
/// are found inside a directory.
fn collect_eis_search_inputs(target: &Path) -> Result<SearchInputCollection, RunnerError> {
    if !target.exists() {
        return Err(format!("Search target does not exist: {}", target.display()).into());
    }

    // Single-file shortcut.
    if target.is_file() {
        return Ok(SearchInputCollection {
            files: vec![target.to_path_buf()],
            skipped: Vec::new(),
        });
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
    let mut skipped = Vec::new();

    for path in entries
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
    {
        match classify_eis_search_input(&path) {
            Ok(SearchInputDecision::Include) => files.push(path),
            Ok(SearchInputDecision::Skip(reason)) => skipped.push((path, reason.to_string())),
            Err(error) => skipped.push((path, error.to_string())),
        }
    }

    if files.is_empty() {
        return Err(format!("No supported EIS files found in {}", target.display()).into());
    }

    Ok(SearchInputCollection { files, skipped })
}

/// Classify a single file as [`SearchInputDecision::Include`] or
/// [`SearchInputDecision::Skip`] based on its extension, stem, and whether its
/// header contains the `Freq/Hz` marker expected in CHI EIS exports.
fn classify_eis_search_input(path: &Path) -> Result<SearchInputDecision, io::Error> {
    let kind = crate::data_file::InputKind::classify_path(path);

    // Reject known binary extensions before attempting to read the file.
    if kind.is_unsupported_binary() {
        return Ok(SearchInputDecision::Skip("unsupported binary extension"));
    }

    let stem = path
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    // Skip files that were themselves generated by a previous search run.
    if stem.ends_with("_ecm_search") {
        return Ok(SearchInputDecision::Skip("generated search report"));
    }
    if stem.contains("fit_report") {
        return Ok(SearchInputDecision::Skip("generated fit report"));
    }

    // For text files, verify EIS header presence.
    if kind.is_supported_text() || kind == crate::data_file::InputKind::Unknown {
        if file_has_eis_header(path)? {
            return Ok(SearchInputDecision::Include);
        }
        return Ok(SearchInputDecision::Skip("missing Freq/Hz header"));
    }

    // Excel files are not EIS sources in this workflow.
    Ok(SearchInputDecision::Skip("unsupported extension"))
}

/// Return `true` if the file's first 64 lines contain a `Freq/Hz` marker,
/// which is the standard column header in CHI EIS data exports.
fn file_has_eis_header(path: &Path) -> Result<bool, io::Error> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines().take(64) {
        let line = line?;
        if line.to_ascii_lowercase().contains("freq/hz") {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::{
        pair_dataset_experimental_and_fitted_colors, resolve_search_combined_plot_output_base,
    };
    use crate::plottings::{PlotSeries, PlotSeriesKind};
    use std::path::Path;

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
}
