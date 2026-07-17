//! Plot job orchestration for the `rust_plots` binary.
//!
//! This module resolves the jobs declared in `plot_config.toml` and drives
//! the two rendering pipelines exposed by the library crate:
//! * **EIS plots** – Nyquist and Bode diagrams produced by `eis_plot`.
//! * **Regular plots** – CHI timeseries / Pb-sensor diagrams produced by
//!   `chi_plot`.

use crate::runners::RunnerError;
use crate::{
    data_file::{
        ElectrochemData, PlotData, load_data, measurement_to_plot_data,
        value_transform::{AxisTransforms, regression_axis_term},
    },
    plot_config::{LoadedPlotConfig, PlotJob, PlotJobKind, RenderConfig},
    plottings::{
        PublicationConfig, eis_combined_publication_config, eis_individual_publication_config,
        generic_combined_publication_config, generic_individual_publication_config,
        load_generic_datasets_from_dir, pb_sensor_combined_publication_config,
        pb_sensor_individual_publication_config, plot_chi_directory_with_configs_and_transforms,
        plot_eis_directory_with_configs, plot_eis_file, plot_generic_datasets, plot_hq,
    },
};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotRunLogLevel {
    Info,
    Warning,
}

// ---------------------------------------------------------------------------
// EIS plotting
// ---------------------------------------------------------------------------

/// Execute all EIS plot jobs declared in `plot_config`.
///
/// For each job the function creates the output directory, applies any
/// per-job style overrides on top of the default EIS publication config, and
/// calls [`plot_eis_directory_with_configs`] to render individual and combined
/// Nyquist / Bode figures.
pub fn run_eis_plots(
    workspace_dir: &Path,
    plot_config: &LoadedPlotConfig,
) -> Result<(), RunnerError> {
    run_eis_plots_with_logger(workspace_dir, plot_config, |level, message| match level {
        PlotRunLogLevel::Info => println!("{message}"),
        PlotRunLogLevel::Warning => eprintln!("{message}"),
    })
}

pub fn run_eis_plots_with_logger<F>(
    workspace_dir: &Path,
    plot_config: &LoadedPlotConfig,
    mut log: F,
) -> Result<(), RunnerError>
where
    F: FnMut(PlotRunLogLevel, &str),
{
    // Resolve only EIS jobs, then process each with shared render defaults.
    let jobs = resolve_jobs(plot_config, PlotJobKind::Eis, workspace_dir)?;
    let render = plot_config.render_config();

    for (idx, job) in jobs.iter().enumerate() {
        fs::create_dir_all(&job.output_dir)?;

        // Apply [render] global defaults first, then per-job style overrides on top.
        let individual_config = job
            .style
            .apply_to_individual(&apply_render_config(
                &eis_individual_publication_config(),
                render,
            )?)
            .map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;
        let combined_config = job
            .style
            .apply_to_combined(&apply_render_config(
                &eis_combined_publication_config(),
                render,
            )?)
            .map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;

        emit_plot_info(
            &mut log,
            format!(
                "Running EIS job {}: input={} output={} prefix='{}'",
                idx + 1,
                job.input_dir.display(),
                job.output_dir.display(),
                job.output_prefix
            ),
        );

        if !job.input_is_directory {
            // Single-file mode: plot the file directly without directory scan.
            let file_path = &job.input_dir;
            let stem = file_path.file_stem().unwrap_or_default().to_string_lossy();
            let prefix = job.output_prefix.trim();
            let output_name = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{prefix}_{stem}")
            };
            let output_base = job.output_dir.join(&output_name);
            fs::create_dir_all(&job.output_dir)?;

            let outcome = plot_eis_file(file_path, &output_base, &combined_config, None)?;
            emit_plot_info(
                &mut log,
                format!("Processed file: {}", outcome.input_file.display()),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Metadata - Date: {}, Test Type: {}, Instrument Model: {}, Circuit Model: {}, Data Points: {}",
                    outcome.data.date,
                    outcome.data.test_type,
                    outcome.data.instrument_model,
                    outcome.data.circuit_model,
                    outcome.data.point_count()
                ),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Plot written to base path: {}",
                    outcome.output_base.display()
                ),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Fit report written to: {}",
                    outcome.fit_report_path.display()
                ),
            );
            continue;
        }

        let plotted = plot_eis_directory_with_configs(
            &job.input_dir,
            &job.output_dir,
            &job.output_prefix,
            &individual_config,
            &combined_config,
            None,
        )?;

        for outcome in plotted.plotted {
            emit_plot_info(
                &mut log,
                format!("Processed file: {}", outcome.input_file.display()),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Metadata - Date: {}, Test Type: {}, Instrument Model: {}, Circuit Model: {}, Data Points: {}",
                    outcome.data.date,
                    outcome.data.test_type,
                    outcome.data.instrument_model,
                    outcome.data.circuit_model,
                    outcome.data.point_count()
                ),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Plot written to base path: {}",
                    outcome.output_base.display()
                ),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Fit report written to: {}",
                    outcome.fit_report_path.display()
                ),
            );
        }

        emit_plot_info(
            &mut log,
            format!(
                "Combined EIS plots written to base path: {}",
                plotted.combined_output_base.display()
            ),
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Regular (CHI / Pb-sensor) plotting
// ---------------------------------------------------------------------------

/// Execute all regular-plot jobs (CHI timeseries / Pb-sensor) declared in
/// `plot_config`.
///
/// Works analogously to [`run_eis_plots`] but targets the CHI rendering
/// pipeline and surfaces per-file skip reasons through stderr.
pub fn run_regular_plots(
    workspace_dir: &Path,
    plot_config: &LoadedPlotConfig,
) -> Result<(), RunnerError> {
    run_regular_plots_with_logger(workspace_dir, plot_config, |level, message| match level {
        PlotRunLogLevel::Info => println!("{message}"),
        PlotRunLogLevel::Warning => eprintln!("{message}"),
    })
}

pub fn run_regular_plots_with_logger<F>(
    workspace_dir: &Path,
    plot_config: &LoadedPlotConfig,
    mut log: F,
) -> Result<(), RunnerError>
where
    F: FnMut(PlotRunLogLevel, &str),
{
    // Resolve only regular-plot jobs, then apply runtime style/transform
    // overlays before dispatching each job.
    let jobs = resolve_jobs(plot_config, PlotJobKind::RegularPlot, workspace_dir)?;
    let render = plot_config.render_config();

    for (idx, job) in jobs.iter().enumerate() {
        fs::create_dir_all(&job.output_dir)?;

        // Apply [render] global defaults first, then per-job style overrides on top.
        let individual_config = job
            .style
            .apply_to_individual(&apply_render_config(
                &pb_sensor_individual_publication_config(),
                render,
            )?)
            .map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;
        let combined_config = job
            .style
            .apply_to_combined(&apply_render_config(
                &pb_sensor_combined_publication_config(),
                render,
            )?)
            .map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;
        let individual_config =
            apply_regression_equation_terms(individual_config, &job.individual_transforms);
        let combined_config =
            apply_regression_equation_terms(combined_config, &job.combined_transforms);

        emit_plot_info(
            &mut log,
            format!(
                "Running regular plot job {}: input={} output={} prefix='{}'",
                idx + 1,
                job.input_dir.display(),
                job.output_dir.display(),
                job.output_prefix
            ),
        );

        if !job.input_is_directory {
            // Single-file mode: plot the file directly without directory scan.
            let file_path = &job.input_dir;
            let series_count = ElectrochemData::series_count(file_path)?;
            let stem = file_path.file_stem().unwrap_or_default().to_string_lossy();
            let prefix = job.output_prefix.trim();
            let output_name = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{prefix}_{stem}")
            };
            let individual_output_base = job.output_dir.join("individual").join(&output_name);
            fs::create_dir_all(individual_output_base.parent().unwrap_or(&job.output_dir))?;

            if series_count > 1 {
                let datasets = ElectrochemData::parse_file_series(file_path)?;
                let combined_suffix = if prefix.is_empty() {
                    format!("{stem}_overlay")
                } else {
                    format!("{prefix}_{stem}_overlay")
                };
                let combined_output_base = job.output_dir.join("combined").join(combined_suffix);
                fs::create_dir_all(combined_output_base.parent().unwrap_or(&job.output_dir))?;

                let individual_plot_config = individual_config
                    .clone()
                    .with_default_axis_labels("Time (s)", "Potential (V)");
                let combined_plot_config = combined_config
                    .clone()
                    .with_default_axis_labels("Time (s)", "Potential (V)");

                let mut individual_datasets = datasets.clone();
                let mut combined_datasets = datasets.clone();
                apply_axis_transforms_to_electrochem_with_logger(
                    &mut individual_datasets,
                    &job.individual_transforms,
                    &mut log,
                );
                apply_axis_transforms_to_electrochem_with_logger(
                    &mut combined_datasets,
                    &job.combined_transforms,
                    &mut log,
                );

                plot_hq(
                    individual_output_base.to_string_lossy().as_ref(),
                    &individual_datasets,
                    &individual_plot_config,
                    false,
                )?;
                plot_hq(
                    combined_output_base.to_string_lossy().as_ref(),
                    &combined_datasets,
                    &combined_plot_config,
                    true,
                )?;

                emit_plot_info(
                    &mut log,
                    format!(
                        "Expanded multi-column regular-plot file into {} datasets: {}",
                        datasets.len(),
                        file_path.display()
                    ),
                );
                for data in &datasets {
                    emit_plot_info(
                        &mut log,
                        format!(
                            "Metadata - Date: {}, Test Type: {}, Instrument Model: {}, Label: {}, Data Points: {}",
                            data.date,
                            data.test_type,
                            data.instrument_model,
                            data.label,
                            data.x_values.len()
                        ),
                    );
                }
                emit_plot_info(
                    &mut log,
                    format!(
                        "Individual regular-plot outputs written under: {}/individual/",
                        job.output_dir.display()
                    ),
                );
                emit_plot_info(
                    &mut log,
                    format!(
                        "Combined regular-plot output written to base path: {}",
                        combined_output_base.display()
                    ),
                );
                continue;
            }

            let raw_data = ElectrochemData::parse_file(file_path)?;
            let mut transformed = vec![raw_data.clone()];
            apply_axis_transforms_to_electrochem_with_logger(
                &mut transformed,
                &job.individual_transforms,
                &mut log,
            );
            let plot_config = individual_config
                .clone()
                .with_default_axis_labels("Time (s)", "Potential (V)");
            plot_hq(
                individual_output_base.to_string_lossy().as_ref(),
                &transformed,
                &plot_config,
                true,
            )?;
            emit_plot_info(
                &mut log,
                format!("Processed regular-plot file: {}", file_path.display()),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Metadata - Date: {}, Test Type: {}, Instrument Model: {}, Data Points: {}",
                    raw_data.date,
                    raw_data.test_type,
                    raw_data.instrument_model,
                    raw_data.x_values.len()
                ),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Plot written to base path: {}",
                    individual_output_base.display()
                ),
            );
            continue;
        }

        let pb_sensor_plotted = plot_chi_directory_with_configs_and_transforms(
            &job.input_dir,
            &job.output_dir,
            &job.output_prefix,
            &individual_config,
            &combined_config,
            &job.individual_transforms,
            &job.combined_transforms,
        )?;

        for outcome in pb_sensor_plotted.plotted {
            emit_plot_info(
                &mut log,
                format!(
                    "Processed regular-plot file: {}",
                    outcome.input_file.display()
                ),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Metadata - Date: {}, Test Type: {}, Instrument Model: {}, Data Points: {}",
                    outcome.data.date,
                    outcome.data.test_type,
                    outcome.data.instrument_model,
                    outcome.data.x_values.len()
                ),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Plot written to base path: {}",
                    outcome.output_base.display()
                ),
            );
        }

        for skipped in pb_sensor_plotted.skipped {
            emit_plot_warning(
                &mut log,
                format!(
                    "Skipped regular-plot file: {} ({})",
                    skipped.input_file.display(),
                    skipped.reason
                ),
            );
        }

        emit_plot_info(
            &mut log,
            format!(
                "Combined regular-plot output written to base path: {}",
                pb_sensor_plotted.combined_output_base.display()
            ),
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Generic (PlotData-based) plotting
// ---------------------------------------------------------------------------

/// Execute all generic plot jobs declared in `plot_config`.
///
/// Generic jobs use the domain-agnostic [`PlotData`] pathway.  The pipeline
/// follows the **load → select → render** sequence so that point selection
/// (configured via `plot_positions` / `plot_values` in the job's style blocks)
/// is applied as a data transformation before anything is handed to the
/// rendering engine:
///
/// ```text
/// [[generic_plot]] input_dir
///     → load files  (ElectrochemData → PlotData via IntoPlotData)
///     → apply individual_selection  (PlotData::select_points — optional)
///     → apply combined_selection    (PlotData::select_points — optional)
///     → plot_generic_datasets       (plot_hq → SVG + PNG)
/// ```
///
/// Individual and combined plots may use different point selections (or no
/// selection at all) independently, controlled by `individual_style` and
/// `combined_style` in the TOML job block respectively.
///
/// The configuration pipeline mirrors the EIS and regular-plot jobs:
///
/// ```text
/// generic domain defaults
///     → [render] global settings
///     → per-job style from [[generic_plot]] block
///     → CLI overrides (applied externally via --plot-config)
///     → ResolvedPlotConfig passed to plot_generic_datasets
/// ```
///
/// If no `[[generic_plot]]` blocks are present in the config this function
/// is a silent no-op, preserving full backward compatibility.
pub fn run_generic_plots(
    workspace_dir: &Path,
    plot_config: &LoadedPlotConfig,
) -> Result<(), RunnerError> {
    run_generic_plots_with_logger(workspace_dir, plot_config, |level, message| match level {
        PlotRunLogLevel::Info => println!("{message}"),
        PlotRunLogLevel::Warning => eprintln!("{message}"),
    })
}

pub fn run_generic_plots_with_logger<F>(
    workspace_dir: &Path,
    plot_config: &LoadedPlotConfig,
    mut log: F,
) -> Result<(), RunnerError>
where
    F: FnMut(PlotRunLogLevel, &str),
{
    // Generic mode is the most flexible path and supports point selection,
    // reassignment, transforms, and optional aggregation mode.
    let jobs = resolve_jobs(plot_config, PlotJobKind::GenericPlot, workspace_dir)?;

    if jobs.is_empty() {
        // No [[generic_plot]] entries configured — nothing to do.
        return Ok(());
    }

    let render = plot_config.render_config();

    for (idx, job) in jobs.iter().enumerate() {
        fs::create_dir_all(&job.output_dir)?;

        // Resolve PublicationConfig for individual and combined plots.
        let individual_config = job
            .style
            .apply_to_individual(&apply_render_config(
                &generic_individual_publication_config(),
                render,
            )?)
            .map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;
        let combined_config = job
            .style
            .apply_to_combined(&apply_render_config(
                &generic_combined_publication_config(),
                render,
            )?)
            .map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;
        let individual_config =
            apply_regression_equation_terms(individual_config, &job.individual_transforms);
        let combined_config =
            apply_regression_equation_terms(combined_config, &job.combined_transforms);

        emit_plot_info(
            &mut log,
            format!(
                "Running generic plot job {}: input={} output={} prefix='{}'",
                idx + 1,
                job.input_dir.display(),
                job.output_dir.display(),
                job.output_prefix
            ),
        );

        if !job.input_is_directory {
            // Single-file mode: load just this one file.
            let file_path = &job.input_dir;
            let loaded = load_data(file_path)?;
            let plot_data: Vec<PlotData> =
                measurement_to_plot_data(loaded.experiment.measurement());
            let stem = file_path.file_stem().unwrap_or_default().to_string_lossy();
            let prefix = job.output_prefix.trim();
            let output_name = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{prefix}_{stem}")
            };
            let individual_output_base = job.output_dir.join("individual").join(&output_name);
            let combined_suffix = if prefix.is_empty() {
                format!("{stem}_overlay")
            } else {
                format!("{prefix}_{stem}_overlay")
            };
            let combined_output_base = job.output_dir.join("combined").join(combined_suffix);

            fs::create_dir_all(individual_output_base.parent().unwrap_or(&job.output_dir))?;
            fs::create_dir_all(combined_output_base.parent().unwrap_or(&job.output_dir))?;

            let mut individual_datasets = apply_optional_selection(
                &plot_data.iter().collect::<Vec<_>>(),
                job.individual_selection.as_ref(),
            )?;
            let mut combined_datasets = apply_optional_selection(
                &plot_data.iter().collect::<Vec<_>>(),
                job.combined_selection.as_ref(),
            )?;

            // Apply point reassignment (after selection, before transforms).
            apply_point_reassignment(
                &mut individual_datasets,
                job.individual_assign_x.as_deref(),
                job.individual_assign_y.as_deref(),
            )?;
            apply_point_reassignment(
                &mut combined_datasets,
                job.combined_assign_x.as_deref(),
                job.combined_assign_y.as_deref(),
            )?;

            // Apply value transforms (after reassignment, before rendering).
            apply_axis_transforms(
                &mut individual_datasets,
                &job.individual_transforms,
                &mut log,
            );
            apply_axis_transforms(&mut combined_datasets, &job.combined_transforms, &mut log);

            plot_generic_datasets(
                &individual_datasets,
                &combined_datasets,
                &individual_output_base,
                &combined_output_base,
                &individual_config,
                &combined_config,
            )?;

            emit_plot_info(
                &mut log,
                format!(
                    "Processed generic-plot file: {} ({} dataset{})",
                    file_path.display(),
                    plot_data.len(),
                    if plot_data.len() == 1 { "" } else { "s" }
                ),
            );
            for data in &plot_data {
                emit_plot_info(
                    &mut log,
                    format!(
                        "Metadata — label: {}, date: {}",
                        data.label.as_deref().unwrap_or("(none)"),
                        data.date.as_deref().unwrap_or("(none)"),
                    ),
                );
            }
            emit_plot_info(
                &mut log,
                format!(
                    "Plot written to base path: {}",
                    individual_output_base.display()
                ),
            );
            continue;
        }

        // ── Step 1: load ──────────────────────────────────────────────────
        // Parse all supported files in the input directory into PlotData.
        // No rendering happens here.
        let (mut loaded, skipped) = load_generic_datasets_from_dir(&job.input_dir)?;

        for s in &skipped {
            emit_plot_warning(
                &mut log,
                format!(
                    "Skipped generic-plot file: {} ({})",
                    s.input_file.display(),
                    s.reason
                ),
            );
        }

        if loaded.is_empty() {
            emit_plot_warning(
                &mut log,
                format!(
                    "No valid datasets found in {} — skipping job {}",
                    job.input_dir.display(),
                    idx + 1
                ),
            );
            continue;
        }

        // ── Step 2: build output paths ────────────────────────────────────
        // Paths are needed by both aggregation mode and standard mode;
        // compute them once before the mode branch.
        let prefix = job.output_prefix.trim();
        let individual_output_base = job
            .output_dir
            .join("individual")
            .join(if prefix.is_empty() {
                "generic_plot".to_string()
            } else {
                prefix.to_string()
            });
        let combined_suffix = if prefix.is_empty() {
            "overlay".to_string()
        } else {
            format!("{prefix}_overlay")
        };
        let combined_output_base = job.output_dir.join("combined").join(combined_suffix);

        fs::create_dir_all(individual_output_base.parent().unwrap_or(&job.output_dir))?;
        fs::create_dir_all(combined_output_base.parent().unwrap_or(&job.output_dir))?;

        // ── Step 3a: aggregation mode ─────────────────────────────────────
        // When aggregate_points_across_files is enabled, reorganize the
        // selected data so each output series represents ONE selected x
        // position across ALL files (transposed from the default layout).
        //
        // Default (N files, M selected x): N series × M points.
        // Aggregation (N files, M selected x): M series × N points.
        //   - x-axis: file index (0-based integer)
        //   - y-axis: y-value at that x-position from each file
        //   - label:  the representative x-value for that position
        if job.aggregate_points_across_files {
            // ── Sort for deterministic ordering ───────────────────────────
            // Default: lexicographic by file name.
            // Optional: by file modification time (oldest-first).
            if job.aggregate_sort_by_mtime {
                loaded.sort_by_key(|d| {
                    d.source_file
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                });
            } else {
                loaded.sort_by(|a, b| a.source_file.file_name().cmp(&b.source_file.file_name()));
            }

            // Apply combined_selection to every file to obtain M selected
            // points per file.  When no selection is configured, all points
            // from each file are used (each file may have a different length).
            let per_file: Vec<PlotData> = apply_optional_selection(
                &loaded.iter().map(|d| &d.data).collect::<Vec<_>>(),
                job.combined_selection.as_ref(),
            )?;

            let n_files = per_file.len();
            let n_pts = per_file.first().map(|d| d.x_values.len()).unwrap_or(0);

            if n_pts == 0 {
                emit_plot_warning(
                    &mut log,
                    format!(
                        "aggregate_points_across_files: no selected points found for job {} \
                         — falling back to standard mode",
                        idx + 1
                    ),
                );
                // Fall through to standard mode below — do NOT `continue`.
            } else {
                // Build the M aggregated PlotData values.
                let file_indices: Vec<f64> = (0..n_files).map(|i| i as f64).collect();
                let aggregated: Vec<PlotData> = (0..n_pts)
                    .map(|j| {
                        // Representative x-value from the first file's selected data.
                        let x_repr = per_file[0].x_values[j];
                        let y_vals: Vec<f64> = per_file
                            .iter()
                            .filter_map(|d| d.y_values.get(j))
                            .copied()
                            .collect();
                        let n = y_vals.len();
                        PlotData::new(file_indices[..n].to_vec(), y_vals)
                            .with_label(format_x_value_label(x_repr))
                    })
                    .collect();

                emit_plot_info(
                    &mut log,
                    format!(
                        "Aggregation mode: {} files × {} selected x-positions → \
                         {} series of {} points each",
                        n_files,
                        n_pts,
                        aggregated.len(),
                        n_files
                    ),
                );

                // Apply value transforms to aggregated datasets.
                let mut agg_individual = aggregated.clone();
                let mut agg_combined = aggregated;
                apply_axis_transforms(&mut agg_individual, &job.individual_transforms, &mut log);
                apply_axis_transforms(&mut agg_combined, &job.combined_transforms, &mut log);

                plot_generic_datasets(
                    &agg_individual,
                    &agg_combined,
                    &individual_output_base,
                    &combined_output_base,
                    &individual_config,
                    &combined_config,
                )?;

                // ── Numbered file-order summary ───────────────────────────
                let order_label = if job.aggregate_sort_by_mtime {
                    "modification time (oldest-first)"
                } else {
                    "file name (lexicographic)"
                };
                emit_plot_info(
                    &mut log,
                    format!("Aggregation file order (sorted by {order_label}):"),
                );
                for (i, loaded_ds) in loaded.iter().enumerate() {
                    emit_plot_info(
                        &mut log,
                        format!(
                            "  {}. {}",
                            i + 1,
                            loaded_ds
                                .source_file
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("<unknown>")
                        ),
                    );
                }
                emit_plot_info(
                    &mut log,
                    format!(
                        "Aggregated individual plots written under: {}/individual/",
                        job.output_dir.display()
                    ),
                );
                emit_plot_info(
                    &mut log,
                    format!(
                        "Aggregated combined plot written to base path: {}",
                        combined_output_base.display()
                    ),
                );
                continue;
            }
        }

        // ── Step 3b: standard mode — select ───────────────────────────────
        // Apply point selection (if configured) independently for individual
        // and combined plots.  When no selection is configured the full
        // dataset is used unchanged.
        let mut individual_datasets: Vec<PlotData> = apply_optional_selection(
            &loaded.iter().map(|d| &d.data).collect::<Vec<_>>(),
            job.individual_selection.as_ref(),
        )?;
        let mut combined_datasets: Vec<PlotData> = apply_optional_selection(
            &loaded.iter().map(|d| &d.data).collect::<Vec<_>>(),
            job.combined_selection.as_ref(),
        )?;

        // ── Step 3b: standard mode — reassign & transform ─────────────────
        apply_point_reassignment(
            &mut individual_datasets,
            job.individual_assign_x.as_deref(),
            job.individual_assign_y.as_deref(),
        )?;
        apply_point_reassignment(
            &mut combined_datasets,
            job.combined_assign_x.as_deref(),
            job.combined_assign_y.as_deref(),
        )?;
        apply_axis_transforms(
            &mut individual_datasets,
            &job.individual_transforms,
            &mut log,
        );
        apply_axis_transforms(&mut combined_datasets, &job.combined_transforms, &mut log);

        // ── Step 3b: standard mode — render ───────────────────────────────
        plot_generic_datasets(
            &individual_datasets,
            &combined_datasets,
            &individual_output_base,
            &combined_output_base,
            &individual_config,
            &combined_config,
        )?;

        // ── Report ────────────────────────────────────────────────────────
        for loaded_ds in &loaded {
            emit_plot_info(
                &mut log,
                format!(
                    "Processed generic-plot file: {}",
                    loaded_ds.source_file.display()
                ),
            );
            emit_plot_info(
                &mut log,
                format!(
                    "Metadata — label: {}, date: {}",
                    loaded_ds.data.label.as_deref().unwrap_or("(none)"),
                    loaded_ds.data.date.as_deref().unwrap_or("(none)"),
                ),
            );
        }

        if job.individual_selection.is_some() {
            emit_plot_info(
                &mut log,
                format!(
                    "Individual plots: point selection applied ({} → {} points per series)",
                    loaded.first().map(|d| d.data.x_values.len()).unwrap_or(0),
                    individual_datasets
                        .first()
                        .map(|d| d.x_values.len())
                        .unwrap_or(0),
                ),
            );
        }
        if job.combined_selection.is_some() {
            emit_plot_info(
                &mut log,
                format!(
                    "Combined plot:    point selection applied ({} → {} points per series)",
                    loaded.first().map(|d| d.data.x_values.len()).unwrap_or(0),
                    combined_datasets
                        .first()
                        .map(|d| d.x_values.len())
                        .unwrap_or(0),
                ),
            );
        }

        emit_plot_info(
            &mut log,
            format!(
                "Individual plots written under: {}/individual/",
                job.output_dir.display()
            ),
        );
        emit_plot_info(
            &mut log,
            format!(
                "Combined plot written to base path: {}",
                combined_output_base.display()
            ),
        );
    }

    Ok(())
}

fn emit_plot_info(log: &mut dyn FnMut(PlotRunLogLevel, &str), message: impl Into<String>) {
    // Centralized logger adapter keeps level formatting uniform across callers.
    let message = message.into();
    log(PlotRunLogLevel::Info, message.as_str());
}

fn emit_plot_warning(log: &mut dyn FnMut(PlotRunLogLevel, &str), message: impl Into<String>) {
    // Centralized logger adapter keeps level formatting uniform across callers.
    let message = message.into();
    log(PlotRunLogLevel::Warning, message.as_str());
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Apply an optional [`PointSelection`] to a slice of [`PlotData`] references.
///
/// When `selection` is `None` the datasets are cloned unchanged.
/// When `selection` is `Some(sel)` each dataset is passed through
/// [`PlotData::select_points`] and any error is propagated upward.
///
/// This is the sole place in the codebase where selection logic is applied
/// to a batch of datasets.  It lives in the runner (orchestration layer)
/// rather than in `plotting.rs` (rendering layer), keeping selection and
/// rendering cleanly separated.
/// Format a floating-point x-value into a human-readable label used in
/// aggregation-mode series names (e.g. `"x = 42"` or `"x = 3.1416"`).
fn format_x_value_label(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1.0e9 {
        format!("x = {}", x as i64)
    } else {
        format!("x = {x:.4}")
    }
}

fn apply_optional_selection(
    datasets: &[&PlotData],
    selection: Option<&crate::data_file::PointSelection>,
) -> Result<Vec<PlotData>, Box<dyn std::error::Error>> {
    match selection {
        None => Ok(datasets.iter().map(|d| (*d).clone()).collect()),
        Some(sel) => datasets
            .iter()
            .map(|d| {
                d.select_points(sel)
                    .map_err(|e| -> Box<dyn std::error::Error> { e.into() })
            })
            .collect(),
    }
}

/// Extract the relevant [`PlotJob`] list from the loaded config, resolving
/// input/output paths relative to `workspace_dir`.
fn resolve_jobs(
    plot_config: &LoadedPlotConfig,
    kind: PlotJobKind,
    workspace_dir: &Path,
) -> Result<Vec<PlotJob>, Box<dyn std::error::Error>> {
    plot_config
        .resolve_jobs(kind, workspace_dir)
        .map_err(|err| err.into())
}

/// Apply `[render]` global settings onto a domain-default `PublicationConfig`.
///
/// This sits between the hard-coded domain defaults (e.g.
/// `eis_individual_publication_config()`) and the per-job `RawPlotStyle`
/// overrides in the precedence chain:
///
/// ```text
/// domain default → [render] settings → per-job style
/// ```
///
/// Fields absent from `render` (i.e. `None`) leave `base` unchanged, so
/// omitting the `[render]` section is a true no-op.
fn apply_render_config(
    base: &PublicationConfig,
    render: &RenderConfig,
) -> Result<PublicationConfig, Box<dyn std::error::Error>> {
    let mut config = base.clone();
    if let Some(scale) = render.png_scale_factor {
        if scale == 0 {
            return Err("invalid render.png_scale_factor: expected a value >= 1".into());
        }
        config.png_scale_factor = scale;
    }
    if let Some(dpi) = render.png_dpi {
        if !dpi.is_finite() || dpi <= 0.0 {
            return Err("invalid render.png_dpi: expected a positive finite value".into());
        }
        config.dpi = dpi;
    }
    Ok(config)
}

// ---------------------------------------------------------------------------
// Point reassignment
// ---------------------------------------------------------------------------

/// Apply explicit coordinate reassignment to a list of datasets.
///
/// When `assign_x` or `assign_y` is provided, the corresponding coordinate
/// axis of **each** dataset is replaced with the given values.  The count
/// must exactly match the number of data points in each dataset; a mismatch
/// is an error.
///
/// # Ordering
///
/// Reassignment is applied **after** point selection but **before** value
/// transforms, so the reassigned values are subject to the same
/// transformation pipeline as original data.
fn apply_point_reassignment(
    datasets: &mut [PlotData],
    assign_x: Option<&[f64]>,
    assign_y: Option<&[f64]>,
) -> Result<(), Box<dyn std::error::Error>> {
    if assign_x.is_none() && assign_y.is_none() {
        return Ok(());
    }
    for d in datasets.iter_mut() {
        if let Some(xs) = assign_x {
            if xs.len() != d.x_values.len() {
                return Err(format!(
                    "assign_x length ({}) does not match the number of data points ({}) \
                     for dataset '{}'",
                    xs.len(),
                    d.x_values.len(),
                    d.label.as_deref().unwrap_or("(unlabeled)")
                )
                .into());
            }
            d.x_values = xs.to_vec();
        }
        if let Some(ys) = assign_y {
            if ys.len() != d.y_values.len() {
                return Err(format!(
                    "assign_y length ({}) does not match the number of data points ({}) \
                     for dataset '{}'",
                    ys.len(),
                    d.y_values.len(),
                    d.label.as_deref().unwrap_or("(unlabeled)")
                )
                .into());
            }
            d.y_values = ys.to_vec();
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Value transforms
// ---------------------------------------------------------------------------

/// Apply axis transforms to a list of datasets, logging warnings.
///
/// Transforms are applied **in-place** on the mutable dataset slice.
/// Transform warnings (e.g. non-positive values for log) are emitted
/// through the `log` callback at `Warning` level.
fn apply_axis_transforms(
    datasets: &mut [PlotData],
    transforms: &AxisTransforms,
    log: &mut dyn FnMut(PlotRunLogLevel, &str),
) {
    if transforms.is_empty() {
        return;
    }
    for d in datasets.iter_mut() {
        if let Some(ref tx) = transforms.x {
            let warnings = tx.apply_vec(&mut d.x_values);
            for w in &warnings {
                log(
                    PlotRunLogLevel::Warning,
                    &format!(
                        "x-axis transform ({tx}): {w} in dataset '{}'",
                        d.label.as_deref().unwrap_or("(unlabeled)")
                    ),
                );
            }
        }
        if let Some(ref ty) = transforms.y {
            let warnings = ty.apply_vec(&mut d.y_values);
            for w in &warnings {
                log(
                    PlotRunLogLevel::Warning,
                    &format!(
                        "y-axis transform ({ty}): {w} in dataset '{}'",
                        d.label.as_deref().unwrap_or("(unlabeled)")
                    ),
                );
            }
        }
    }
}

fn apply_axis_transforms_to_electrochem_with_logger(
    datasets: &mut [ElectrochemData],
    transforms: &AxisTransforms,
    log: &mut dyn FnMut(PlotRunLogLevel, &str),
) {
    // ElectrochemData variant mirrors PlotData transform behavior for the
    // regular-plot code path before conversion/rendering.
    if transforms.is_empty() {
        return;
    }
    for d in datasets.iter_mut() {
        if let Some(ref tx) = transforms.x {
            let warnings = tx.apply_vec(&mut d.x_values);
            for w in &warnings {
                log(
                    PlotRunLogLevel::Warning,
                    &format!("x-axis transform ({tx}): {w} in dataset '{}'", d.label),
                );
            }
        }
        if let Some(ref ty) = transforms.y {
            let warnings = ty.apply_vec(&mut d.y_values);
            for w in &warnings {
                log(
                    PlotRunLogLevel::Warning,
                    &format!("y-axis transform ({ty}): {w} in dataset '{}'", d.label),
                );
            }
        }
    }
}

fn apply_regression_equation_terms(
    config: PublicationConfig,
    transforms: &AxisTransforms,
) -> PublicationConfig {
    // Keep displayed regression equation terms aligned with active axis
    // transforms so annotations reflect transformed coordinates.
    let x_term = regression_axis_term(transforms.x.as_ref(), "x");
    let y_term = regression_axis_term(transforms.y.as_ref(), "y");
    config.with_regression_terms(x_term, y_term)
}
