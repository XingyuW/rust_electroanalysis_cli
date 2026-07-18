//! Generic plotting pipeline for [`PlotData`]-based workflows.
//!
//! # Purpose
//!
//! This module is the plotting-side complement to [`crate::data_op`].  Where
//! `data_op` defines how domain-specific data is reduced to [`PlotData`], this
//! module defines how a slice of `PlotData` values is rendered to SVG and PNG
//! files via the existing `plot_hq` primitive.
//!
//! # Responsibilities
//!
//! * Domain-neutral default [`PublicationConfig`] values for individual and
//!   combined generic plots.
//! * Directory-scanning helpers that load files ([`ElectrochemData`] →
//!   [`PlotData`] via [`IntoPlotData`]), and drive both per-file and
//!   all-in-one rendering.
//! * Outcome types ([`GenericPlotOutcome`], [`GenericDirectoryPlotOutcome`])
//!   that parallel the `Chi*` / `EIS*` outcome types from the specialised
//!   plotting modules.
//!
//! # Separation of concerns
//!
//! This module handles **rendering only**.  It never resolves configuration —
//! callers (typically `plot_runner.rs`) are required to pass already-resolved
//! [`ResolvedPlotConfig`] values.  See the pipeline description in
//! [`crate::data_op`] for the full config-resolution sequence.
//!
//! # Extensibility
//!
//! The directory-scanning functions currently load files through
//! [`ElectrochemData::parse_file`] then convert via [`IntoPlotData`].  To
//! support a different file format:
//!
//! 1. Write a parser that produces some `T: IntoPlotData`.
//! 2. Add a branch or sister function that loads files through that parser.
//! 3. Convert with `your_data.into_plot_data()` and feed the resulting
//!    `Vec<PlotData>` into [`plot_generic_datasets`].
//!
//! No changes to `plotting.rs`, `plot_hq`, or anything else are needed.

use crate::data_file::{InputKind, PlotData, load_data, measurement_to_plot_data};
use crate::plottings::plotting::{
    PlotColor, PlotLegendPosition, PublicationConfig, ResolvedPlotConfig, plot_hq,
};

use std::error::Error;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Outcome types
// ─────────────────────────────────────────────────────────────────────────────

/// The outcome of plotting a single file through the generic pathway.
#[derive(Debug, Clone)]
pub struct GenericPlotOutcome {
    /// The input file that was consumed.
    pub input_file: PathBuf,
    /// Base path of the output files (without extension).  Append `.svg` or
    /// `.png` to obtain the concrete file paths.
    pub output_base: PathBuf,
    /// The `PlotData` that was rendered (includes label and optional date).
    pub data: PlotData,
}

/// A file that was skipped during a directory scan.
#[derive(Debug, Clone)]
pub struct GenericPlotSkip {
    /// The file that was skipped.
    pub input_file: PathBuf,
    /// Human-readable reason (unsupported extension, parse error, …).
    pub reason: String,
}

/// Aggregate outcome from a directory-level generic plot job.
///
/// Mirrors [`crate::plottings::ChiDirectoryPlotOutcome`] to keep the caller
/// interface uniform across plot types.
#[derive(Debug, Clone)]
pub struct GenericDirectoryPlotOutcome {
    /// One entry for every file that was successfully plotted.
    pub plotted: Vec<GenericPlotOutcome>,
    /// Files that could not be processed (unsupported type, parse failure, …).
    pub skipped: Vec<GenericPlotSkip>,
    /// Base path for the combined (all-in-one) output files.
    pub combined_output_base: PathBuf,
    /// Base path for the individual output files (without per-file suffixes).
    pub individual_output_base: PathBuf,
}

// ─────────────────────────────────────────────────────────────────────────────
// Domain-neutral default PublicationConfig values
// ─────────────────────────────────────────────────────────────────────────────

/// Default [`PublicationConfig`] for **individual** generic plots.
///
/// These values are deliberately neutral — no domain-specific axis labels or
/// colours are prescribed.  The axis labels fall back to the `PublicationConfig`
/// sentinel values (`"X Values"` / `"Y Values"`), which are then
/// overridden by any `x_label`/`y_label` the user supplies in
/// `plot_config.toml` or via CLI.
///
/// This function sits at priority 3 in the config pipeline:
/// ```text
/// domain defaults (this fn)
///     → [render] settings
///     → per-job style from TOML
///     → CLI overrides
///     → ResolvedPlotConfig
/// ```
pub fn generic_individual_publication_config() -> PublicationConfig {
    PublicationConfig {
        width_inches: 8.4,
        height_inches: 5.8,
        font_size_pt: 22.0,
        line_width: 3,
        plot_ratio_x: 10.6,
        plot_ratio_y: 6.2,
        legend_font_ratio: 0.62,
        experimental_marker_radius: 8,
        experimental_color: Some(PlotColor::rgb(22, 94, 131)),
        series_palette: vec![],
        legend_position: PlotLegendPosition::UpperRight,
        ..Default::default()
    }
}

/// Default [`PublicationConfig`] for **combined** (multi-series overlay)
/// generic plots.
///
/// A standard palette is provided for up to nine distinct series.  More
/// series will cycle through the Plotters default palette.
pub fn generic_combined_publication_config() -> PublicationConfig {
    PublicationConfig {
        width_inches: 9.2,
        height_inches: 6.0,
        font_size_pt: 22.0,
        line_width: 3,
        plot_ratio_x: 11.0,
        plot_ratio_y: 6.2,
        legend_font_ratio: 0.56,
        experimental_marker_radius: 7,
        series_palette: vec![
            PlotColor::rgb(22, 94, 131),
            PlotColor::rgb(178, 80, 25),
            PlotColor::rgb(46, 125, 50),
            PlotColor::rgb(111, 66, 193),
            PlotColor::rgb(198, 70, 52),
            PlotColor::rgb(0, 128, 128),
            PlotColor::rgb(188, 108, 37),
            PlotColor::rgb(87, 117, 144),
            PlotColor::rgb(173, 32, 32),
        ],
        legend_position: PlotLegendPosition::UpperRight,
        ..Default::default()
    }
}

/// A dataset loaded from disk but not yet rendered.
///
/// Returned by [`load_generic_datasets_from_dir`] as part of the
/// load → select → render pipeline.  Pairing each [`PlotData`] with its
/// source path allows the runner to report per-file progress and attribution
/// after rendering.
#[derive(Debug, Clone)]
pub struct LoadedGenericDataset {
    /// The [`PlotData`] produced by parsing the source file.
    pub data: PlotData,
    /// The file path that was parsed to produce `data`.
    pub source_file: PathBuf,
}

/// Scan `input_dir` and parse all supported files into [`PlotData`] without
/// rendering anything.
///
/// This is the **loading half** of the generic plot pipeline.  It mirrors the
/// directory-scanning logic inside [`plot_generic_directory_with_configs`] but
/// stops before rendering, allowing the caller to apply point selection or
/// any other data transformation before invoking the plotting functions.
///
/// # Typical usage (load → select → render):
///
/// ```rust,ignore
/// let (loaded, skipped) = load_generic_datasets_from_dir(&job.input_dir)?;
/// let datasets: Vec<PlotData> = if let Some(sel) = &job.individual_selection {
///     loaded.iter().map(|d| d.data.select_points(sel)).collect::<Result<_,_>>()?
/// } else {
///     loaded.iter().map(|d| d.data.clone()).collect()
/// };
/// plot_generic_datasets(&datasets, &indiv_base, &combined_base, indiv_cfg, combined_cfg)?;
/// ```
///
/// # Returns
///
/// `(loaded, skipped)` where:
/// * `loaded` contains one [`LoadedGenericDataset`] per successfully parsed file,
///   sorted by file path.
/// * `skipped` contains one [`GenericPlotSkip`] per file that was skipped due
///   to an unsupported extension or a parse failure.
pub fn load_generic_datasets_from_dir<P: AsRef<Path>>(
    input_dir: P,
) -> Result<(Vec<LoadedGenericDataset>, Vec<GenericPlotSkip>), Box<dyn Error>> {
    let input_dir = input_dir.as_ref();
    let mut entries = read_dir(input_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    let mut loaded: Vec<LoadedGenericDataset> = Vec::new();
    let mut skipped: Vec<GenericPlotSkip> = Vec::new();

    for entry in entries {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let kind = InputKind::classify_path(&path);
        if kind.is_unsupported_binary() {
            skipped.push(GenericPlotSkip {
                input_file: path,
                reason: kind.skip_reason().to_string(),
            });
            continue;
        }
        if !kind.is_supported_text() && !kind.is_supported_spreadsheet() {
            skipped.push(GenericPlotSkip {
                input_file: path,
                reason: "unsupported extension".to_string(),
            });
            continue;
        }
        match load_data(&path) {
            Ok(parsed) => {
                for data in measurement_to_plot_data(parsed.experiment.measurement()) {
                    loaded.push(LoadedGenericDataset {
                        source_file: path.clone(),
                        data,
                    });
                }
            }
            Err(err) => skipped.push(GenericPlotSkip {
                input_file: path,
                reason: err.to_string(),
            }),
        }
    }

    Ok((loaded, skipped))
}

// ─────────────────────────────────────────────────────────────────────────────
// Core rendering helper
// ─────────────────────────────────────────────────────────────────────────────

/// Render a prepared slice of [`PlotData`] to SVG and PNG output files.
///
/// This is the **innermost plotting primitive** of the generic pathway.
/// It accepts already-prepared data and already-resolved configuration — it
/// performs no file loading, no config resolution, and no directory scanning.
///
/// Both individual (one-series-per-file) and combined (all-series overlay)
/// modes are driven through this function by the higher-level
/// [`plot_generic_directory_with_configs`] helper.
///
/// # Parameters
///
/// * `datasets` — Slice of [`PlotData`] values to render.  Each element
///   provides a distinct named series.
/// * `output_base_individual` — Filesystem base path (no extension) for
///   individual per-series output files.  A per-series suffix is appended
///   internally.
/// * `output_base_combined` — Filesystem base path for the combined overlay
///   output files.
/// * `individual_config` — Fully resolved [`ResolvedPlotConfig`] for
///   individual plots.  Must not contain sentinel-state fields.
/// * `combined_config` — Fully resolved [`ResolvedPlotConfig`] for the
///   combined overlay plot.
///
/// # Config pipeline contract
///
/// Both config arguments **must** be the result of the full resolution
/// pipeline (CLI args → TOML → `[render]` defaults → domain defaults →
/// `PublicationConfig` sentinel defaults).  See `plot_runner.rs` and
/// `PlotJobStyle::apply_to_individual/combined` for the standard mechanism.
pub fn plot_generic_datasets(
    individual_datasets: &[PlotData],
    combined_datasets: &[PlotData],
    output_base_individual: &Path,
    output_base_combined: &Path,
    individual_config: &ResolvedPlotConfig,
    combined_config: &ResolvedPlotConfig,
) -> Result<(), Box<dyn Error>> {
    // The generic pathway uses the sentinel axis labels as-is ("X Values" /
    // "Y Values") unless the user has overridden them in TOML or via CLI.
    // We call with_default_axis_labels here so any TOML/CLI override that is
    // still at sentinel state receives a minimal descriptive fallback.
    let individual_cfg = individual_config
        .clone()
        .with_default_axis_labels("X Values", "Y Values");
    let combined_cfg = combined_config
        .clone()
        .with_default_axis_labels("X Values", "Y Values");

    // Individual: render each dataset by itself.
    for data in individual_datasets {
        let series_name = sanitize_output_component(data.label.as_deref().unwrap_or("data"));
        let base = append_output_suffix(output_base_individual, &series_name);
        plot_hq(
            base.to_string_lossy().as_ref(),
            std::slice::from_ref(data),
            &individual_cfg,
            false,
        )?;
    }

    // Combined: render all datasets as a single overlay figure.
    // Uses combined_datasets which may differ from individual_datasets when
    // independent point selection is configured for each scope.
    plot_hq(
        output_base_combined.to_string_lossy().as_ref(),
        combined_datasets,
        &combined_cfg,
        true,
    )?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Directory-level orchestrators
// ─────────────────────────────────────────────────────────────────────────────

/// Plot all supported data files in `input_dir` using a **single**
/// `PublicationConfig` for both individual and combined outputs.
///
/// Delegates to [`plot_generic_directory_with_configs`] with the same config
/// for both scopes.  Prefer [`plot_generic_directory_with_configs`] when the
/// individual and combined plots need different styling.
pub fn plot_generic_directory<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    output_prefix: &str,
    config: &PublicationConfig,
) -> Result<GenericDirectoryPlotOutcome, Box<dyn Error>> {
    plot_generic_directory_with_configs(input_dir, output_dir, output_prefix, config, config)
}

/// Plot all supported data files in `input_dir` using **separate**
/// [`ResolvedPlotConfig`] values for individual and combined outputs.
///
/// # Workflow inside this function
///
/// 1. Scan `input_dir` for supported file types (`.csv`, `.txt`, `.dat`).
/// 2. Parse each file via [`ElectrochemData::parse_file`].
/// 3. Convert to [`PlotData`] via [`IntoPlotData`] (the
///    `From<ElectrochemData>` impl).
/// 4. Write individual plots (one file per dataset) and one combined overlay.
///
/// ## Extending to new file formats
///
/// Replace step 2–3 with your own parser + `IntoPlotData` implementation.
/// Steps 1, 4, and the outcome types are format-agnostic.
///
/// # Config pipeline contract
///
/// `individual_config` and `combined_config` must already be fully resolved.
/// This function never modifies or re-resolves them.
pub fn plot_generic_directory_with_configs<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    output_prefix: &str,
    individual_config: &ResolvedPlotConfig,
    combined_config: &ResolvedPlotConfig,
) -> Result<GenericDirectoryPlotOutcome, Box<dyn Error>> {
    let input_dir = input_dir.as_ref();
    let output_dir = output_dir.as_ref();

    // Build output base paths following the same naming convention as
    // chi_plot.rs: individual files are nested under an "individual/"
    // subdirectory; the combined overlay lives under "combined/".
    let individual_output_base = output_dir
        .join("individual")
        .join(base_output_name(output_prefix, "generic_plot"));
    let combined_output_base = output_dir
        .join("combined")
        .join(base_output_name_with_suffix(
            output_prefix,
            "overlay",
            "overlay",
        ));

    let mut entries = read_dir(input_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    let mut plotted: Vec<GenericPlotOutcome> = Vec::new();
    let mut skipped: Vec<GenericPlotSkip> = Vec::new();
    let mut datasets: Vec<PlotData> = Vec::new();

    for entry in entries {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let kind = InputKind::classify_path(&path);
        if kind.is_unsupported_binary() {
            skipped.push(GenericPlotSkip {
                input_file: path,
                reason: kind.skip_reason().to_string(),
            });
            continue;
        }
        if !kind.is_supported_text() && !kind.is_supported_spreadsheet() {
            skipped.push(GenericPlotSkip {
                input_file: path,
                reason: "unsupported extension".to_string(),
            });
            continue;
        }

        match load_data(&path) {
            Ok(parsed) => {
                for plot_data in measurement_to_plot_data(parsed.experiment.measurement()) {
                    let series_name =
                        sanitize_output_component(plot_data.label.as_deref().unwrap_or("data"));
                    let output_base = append_output_suffix(&individual_output_base, &series_name);

                    datasets.push(plot_data.clone());
                    plotted.push(GenericPlotOutcome {
                        input_file: path.clone(),
                        output_base,
                        data: plot_data,
                    });
                }
            }
            Err(err) => skipped.push(GenericPlotSkip {
                input_file: path,
                reason: err.to_string(),
            }),
        }
    }

    if datasets.is_empty() {
        return Err(format!("No valid datasets found in {}", input_dir.display()).into());
    }

    // Create subdirectories only after we know there is something to write.
    std::fs::create_dir_all(individual_output_base.parent().unwrap_or(output_dir))?;
    std::fs::create_dir_all(combined_output_base.parent().unwrap_or(output_dir))?;

    plot_generic_datasets(
        &datasets,
        &datasets,
        &individual_output_base,
        &combined_output_base,
        individual_config,
        combined_config,
    )?;

    Ok(GenericDirectoryPlotOutcome {
        plotted,
        skipped,
        combined_output_base,
        individual_output_base,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers (mirror chi_plot.rs naming)
// ─────────────────────────────────────────────────────────────────────────────

fn sanitize_output_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if ch.is_ascii_whitespace() || matches!(ch, '-' | '_' | '.') {
            out.push('_');
        }
    }
    let out = out.trim_matches('_');
    if out.is_empty() {
        "series".to_string()
    } else {
        out.to_string()
    }
}

fn append_output_suffix(base: &Path, suffix: &str) -> PathBuf {
    let parent = base.parent().unwrap_or_else(|| Path::new(""));
    let filename = base
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("generic_plot");
    parent.join(format!("{filename}_{suffix}"))
}

fn base_output_name(prefix: &str, fallback: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn base_output_name_with_suffix(prefix: &str, suffix: &str, fallback: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        format!("{trimmed}_{suffix}")
    }
}
