#![allow(clippy::collapsible_if)]

//! Plot-job configuration loading, TOML parsing, and precedence resolution.
//!
//! # Configuration precedence (highest → lowest)
//!
//! 1. **CLI arguments** — expressed as a `RawPlotStyle` layer merged last via
//!    [`PlotJobStyle::apply_to_individual`] / [`PlotJobStyle::apply_to_combined`].
//! 2. **`plot_config.toml`** — `RawPlotStyle` fields; `None` fields are skipped
//!    so they do not silently override lower-priority values.
//! 3. **`[render]` section** — global rendering knobs (e.g. `png_scale_factor`,
//!    `png_dpi`) applied on top of the domain defaults.
//! 4. **Domain defaults** — the `*_publication_config()` functions in
//!    `eis_plot.rs`, `chi_plot.rs`, and `generic_plot.rs` supply
//!    plot-type-appropriate dimensions, fonts, colors, and marker settings.
//! 5. **Global sentinel defaults** — `PublicationConfig::default()` provides
//!    the baseline fallback for every field, including the default
//!    `series_palette`.
//! 6. **Per-plot-type axis labels** — applied last via
//!    `PublicationConfig::with_default_axis_labels()`.  This method is
//!    sentinel-guarded: it only writes a label when the field is still
//!    `"X Values"` / `"Y Values"`, so any user-supplied value from layers
//!    1–2 is never overwritten.
//!
//! # Color resolution in the rendering layer
//!
//! For experimental data the resolved `series_palette` always takes priority
//! over the scalar `experimental_color`.  When a config's palette covers an
//! index, its colour is used; when the palette is empty (individual domain
//! defaults), the scalar `experimental_color` serves as the fallback.  This
//! ensures that a user-supplied `series_palette` is never silently overridden.

use crate::DEFAULT_LOG_BASE;
use crate::data_file::PointSelection;
use crate::data_file::value_transform::{AxisTransforms, TransformKind, resolve_axis_transforms};
use crate::plottings::{
    AxisScale, AxisScaleKind, FillBetweenMode, PieValueLabelMode, PlotColor, PlotLegendPosition,
    PlotLineStyle, PlotMarkerShape, PlotType, PublicationConfig, RegressionAnnotationLayout,
    ScientificNotationStyle,
};
use crate::regression_mod::RegressionKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_PLOT_CONFIG_PATH: &str = "config/plotting.toml";
pub const LEGACY_PLOT_CONFIG_PATH: &str = "plot_config.toml";
pub const PLOT_CONFIG_SCHEMA_VERSION: u32 = 1;

/// Unified shared configuration — single source of truth for workflow paths
/// and the global style baseline inherited by all workflow jobs unless
/// explicitly overridden at the job level.
///
/// In `plot_config.toml` this corresponds to the `[shared]` section with
/// optional `[shared.style]`, `[shared.individual_style]`, and
/// `[shared.combined_style]` sub-tables:
///
/// ```toml
/// [shared]
/// workspace_dir = "/path/to/workspace"
/// input_path    = "/path/to/input"
/// output_path   = "/path/to/output"
/// output_prefix = "plot"
///
/// [shared.style]
/// dpi              = 306.0
/// width_inches     = 7.2
/// height_inches    = 5.2
/// font_size_pt     = 21.0
/// experimental_color = "#0000ff"
/// fitted_color     = "#ff6a00"
/// legend_position  = "upper_right"
/// png_scale_factor = 2
///
/// [shared.individual_style]
/// # optional per-individual-plot overrides inherited by all jobs
///
/// [shared.combined_style]
/// # optional per-combined-plot overrides inherited by all jobs
/// ```
///
/// # Configuration precedence (style layers, highest → lowest)
///
/// 1. CLI arguments
/// 2. Per-job `[<workflow>.style]` / `[<workflow>.individual_style]` / `[<workflow>.combined_style]`
/// 3. Named style preset referenced by `style_preset` in the job entry
/// 4. **`[shared.style]` / `[shared.individual_style]` / `[shared.combined_style]`** ← this section
/// 5. `[render]` global knobs
/// 6. Domain defaults (`eis_publication_config()` etc.)
/// 7. `PublicationConfig::default()`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SharedConfig {
    // ── Shared path/session state ───────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_is_directory: Option<bool>,

    // ── Shared style baseline (inherited by all workflow jobs) ─────────────
    /// Job-wide style overrides applied to every workflow as the lowest
    /// user-controllable layer before domain defaults.
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub style: RawPlotStyle,
    /// Individual-plot style overrides applied to every workflow's individual
    /// outputs.
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub individual_style: RawPlotStyle,
    /// Combined-plot style overrides applied to every workflow's overlay
    /// outputs.
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub combined_style: RawPlotStyle,
}

impl SharedConfig {
    /// Returns `true` when no field carries a non-default value, meaning the
    /// section can be omitted entirely from the serialized TOML without loss.
    pub fn is_default(&self) -> bool {
        self.workspace_dir.is_none()
            && self.input_path.is_none()
            && self.output_path.is_none()
            && self.output_prefix.is_none()
            && self.config_file_path.is_none()
            && self.input_is_directory.is_none()
            && self.style.is_all_none()
            && self.individual_style.is_all_none()
            && self.combined_style.is_all_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotJobKind {
    Eis,
    RegularPlot,
    /// Domain-agnostic pathway: files are parsed through the CHI-format
    /// loader, converted to `PlotData` via `IntoPlotData`, and rendered
    /// by the generic plotting pipeline in `generic_plot.rs`.
    ///
    /// Unlike [`PlotJobKind::RegularPlot`] the generic pathway imposes no
    /// domain-specific axis-label defaults, making it suitable for any x/y
    /// dataset whose labels are fully determined by the user's TOML config.
    GenericPlot,
}

#[derive(Debug, Clone)]
pub struct PlotJob {
    pub input_dir: PathBuf,
    /// `true`  → `input_dir` is a directory (batch mode).
    /// `false` → `input_dir` is a single file.
    pub input_is_directory: bool,
    pub output_dir: PathBuf,
    pub output_prefix: String,
    pub style: PlotJobStyle,
    /// Resolved point selection for **individual** plots.
    ///
    /// Derived from the job's `individual_style.plot_positions` /
    /// `individual_style.plot_values` fields (with `style` as fallback).  When
    /// `None` no point selection is applied and all data points are plotted.
    pub individual_selection: Option<PointSelection>,
    /// Resolved point selection for the **combined** (overlay) plot.
    ///
    /// Derived from the job's `combined_style.plot_positions` /
    /// `combined_style.plot_values` fields (with `style` as fallback).  When
    /// `None` no point selection is applied and all data points are plotted.
    pub combined_selection: Option<PointSelection>,
    /// When `true`, reorganize selected data across input files before
    /// rendering.  One output line is produced per selected x-position,
    /// each containing y-values from all input files at that position.
    ///
    /// Resolved from `combined_style.aggregate_points_across_files` (falling
    /// back to `style.aggregate_points_across_files`).  Applies only to
    /// directory-mode [`PlotJobKind::GenericPlot`] jobs.
    pub aggregate_points_across_files: bool,
    /// When `true`, input files are sorted by modification time (oldest-first)
    /// before aggregation.  When `false` (the default), files are sorted
    /// lexicographically by file name.
    ///
    /// Resolved from `combined_style.aggregate_sort_by_mtime`.
    pub aggregate_sort_by_mtime: bool,
    /// Resolved axis transforms for **individual** plots.
    pub individual_transforms: AxisTransforms,
    /// Resolved axis transforms for **combined** plots.
    pub combined_transforms: AxisTransforms,
    /// Explicit x-value reassignment for **individual** plots.
    pub individual_assign_x: Option<Vec<f64>>,
    /// Explicit y-value reassignment for **individual** plots.
    pub individual_assign_y: Option<Vec<f64>>,
    /// Explicit x-value reassignment for **combined** plots.
    pub combined_assign_x: Option<Vec<f64>>,
    /// Explicit y-value reassignment for **combined** plots.
    pub combined_assign_y: Option<Vec<f64>>,
}

/// Merged TOML style layers for a single plot job.
///
/// Holds up to three `RawPlotStyle` layers applied in sequence to produce
/// the final [`PublicationConfig`] for each output type:
/// - `style` — job-wide overrides applied first
/// - `individual_style` / `combined_style` — scope-specific overrides applied on top
#[derive(Debug, Clone)]
pub struct PlotJobStyle {
    pub style: RawPlotStyle,
    pub individual_style: RawPlotStyle,
    pub combined_style: RawPlotStyle,
}

impl PlotJobStyle {
    /// Produce the fully-resolved [`PublicationConfig`] for **individual** plots.
    ///
    /// Applies `style` then `individual_style` on top of `base` (the domain
    /// default config from `eis_plot.rs` / `chi_plot.rs`).  Fields absent
    /// from both TOML layers are inherited from `base` unchanged.
    pub fn apply_to_individual(
        &self,
        base: &PublicationConfig,
    ) -> Result<PublicationConfig, String> {
        let common = self.style.apply_to(base)?;
        self.individual_style.apply_to(&common)
    }

    /// Produce the fully-resolved [`PublicationConfig`] for **combined** plots.
    ///
    /// Applies `style` then `combined_style` on top of `base` (the domain
    /// default config from `eis_plot.rs` / `chi_plot.rs`).  Fields absent
    /// from both TOML layers are inherited from `base` unchanged.
    pub fn apply_to_combined(&self, base: &PublicationConfig) -> Result<PublicationConfig, String> {
        let common = self.style.apply_to(base)?;
        self.combined_style.apply_to(&common)
    }

    /// Resolve the [`PointSelection`] for **individual** plots.
    ///
    /// Applies the same priority as style fields:
    /// `individual_style` overrides `style` when both are present.
    /// Returns `None` when neither scope configures a selection.
    ///
    /// # Errors
    ///
    /// Returns an error string if both `plot_positions` and `plot_values` are
    /// set in the same scope (ambiguous selection mode).
    pub fn resolve_individual_selection(&self) -> Result<Option<PointSelection>, String> {
        let base = self.style.resolve_selection()?;
        let specific = self.individual_style.resolve_selection()?;
        Ok(specific.or(base))
    }

    /// Resolve the [`PointSelection`] for the **combined** (overlay) plot.
    ///
    /// Applies the same priority as style fields:
    /// `combined_style` overrides `style` when both are present.
    /// Returns `None` when neither scope configures a selection.
    ///
    /// # Errors
    ///
    /// Returns an error string if both `plot_positions` and `plot_values` are
    /// set in the same scope (ambiguous selection mode).
    pub fn resolve_combined_selection(&self) -> Result<Option<PointSelection>, String> {
        let base = self.style.resolve_selection()?;
        let specific = self.combined_style.resolve_selection()?;
        Ok(specific.or(base))
    }

    /// Resolve axis transforms for **individual** plots.
    ///
    /// Per-axis fields in `individual_style` override those in `style`.
    pub fn resolve_individual_transforms(&self) -> Result<AxisTransforms, String> {
        let merged = self.style.merged_with(&self.individual_style);
        merged.resolve_transforms()
    }

    /// Resolve axis transforms for **combined** plots.
    ///
    /// Per-axis fields in `combined_style` override those in `style`.
    pub fn resolve_combined_transforms(&self) -> Result<AxisTransforms, String> {
        let merged = self.style.merged_with(&self.combined_style);
        merged.resolve_transforms()
    }

    /// Resolve point reassignment for **individual** plots.
    ///
    /// `individual_style` fields override `style`.
    pub fn resolve_individual_assign(&self) -> (Option<Vec<f64>>, Option<Vec<f64>>) {
        let merged = self.style.merged_with(&self.individual_style);
        (merged.assign_x, merged.assign_y)
    }

    /// Resolve point reassignment for **combined** plots.
    ///
    /// `combined_style` fields override `style`.
    pub fn resolve_combined_assign(&self) -> (Option<Vec<f64>>, Option<Vec<f64>>) {
        let merged = self.style.merged_with(&self.combined_style);
        (merged.assign_x, merged.assign_y)
    }
}

#[derive(Debug)]
pub struct LoadedPlotConfig {
    pub config: PlotConfig,
    pub base_dir: PathBuf,
    pub source_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

impl LoadedPlotConfig {
    /// Return the global render configuration parsed from `[render]` in
    /// `plot_config.toml`.  Callers apply these settings as a low-priority
    /// layer on top of the domain defaults before per-job style overrides.
    pub fn render_config(&self) -> &RenderConfig {
        &self.config.render
    }
}

/// Global rendering knobs parsed from the `[render]` section of
/// `plot_config.toml`.
///
/// These settings apply to every plot job unless overridden by a per-job
/// `RawPlotStyle` field (e.g. `png_scale_factor` inside `[[eis]]`).
///
/// # Example TOML
/// ```toml
/// [render]
/// png_scale_factor = 2   # internal upscaling factor (default 2)
/// png_dpi = 300          # optional DPI override for all PNG outputs
/// ```
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RenderConfig {
    /// Internal upscaling factor for the PNG SSAA pipeline.
    /// Matches the default of `PublicationConfig::png_scale_factor` (`2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub png_scale_factor: Option<u32>,
    /// Target DPI for PNG output.  When set, overrides `dpi` in
    /// `PublicationConfig` for all plot jobs before per-job style is applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub png_dpi: Option<f32>,
}

impl RenderConfig {
    /// Returns `true` when both fields are `None`, meaning the entire `[render]`
    /// section can be omitted from the serialized TOML without any loss of
    /// information.  Values in `[shared.style]` take higher precedence anyway.
    pub fn is_default(&self) -> bool {
        self.png_scale_factor.is_none() && self.png_dpi.is_none()
    }
}

/// Per-workflow style configuration — single source of truth for workflow-specific
/// style overrides.
///
/// Contains only style overrides and an optional named preset; path settings
/// (`input_path`, `output_path`, `output_prefix`) are exclusively owned by
/// `[shared]` and are never present here.
///
/// In `plot_config.toml` this corresponds to a **singular** `[eis]`,
/// `[regular_plot]`, or `[generic_plot]` section (NOT an array):
///
/// ```toml
/// [eis]
/// style_preset = "paper"
///
/// [eis.individual_style]
/// experimental_color = "#000dff"
/// ```
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawWorkflowConfig {
    pub style_preset: Option<String>,
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub style: RawPlotStyle,
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub individual_style: RawPlotStyle,
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub combined_style: RawPlotStyle,
}

impl RawWorkflowConfig {
    fn is_none_or_empty(opt: &Option<Self>) -> bool {
        opt.as_ref().is_none_or(|c| {
            c.style_preset.is_none()
                && c.style.is_all_none()
                && c.individual_style.is_all_none()
                && c.combined_style.is_all_none()
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PlotConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    /// EIS workflow style overrides.  All path settings live in `[shared]`.
    #[serde(default, skip_serializing_if = "RawWorkflowConfig::is_none_or_empty")]
    pub eis: Option<RawWorkflowConfig>,
    /// Regular-plot (CHI time-series) workflow style overrides.
    #[serde(default, skip_serializing_if = "RawWorkflowConfig::is_none_or_empty")]
    pub regular_plot: Option<RawWorkflowConfig>,
    /// Generic plotting workflow style overrides.
    ///
    /// Uses the domain-agnostic `PlotData` pathway.  Files are loaded
    /// through the CHI-format parser and rendered by `generic_plot.rs`.
    /// Axis labels have no domain-specific defaults; supply `x_label` /
    /// `y_label` in `[generic_plot.style]` or they fall back to
    /// `"X Values"` / `"Y Values"`.
    #[serde(default, skip_serializing_if = "RawWorkflowConfig::is_none_or_empty")]
    pub generic_plot: Option<RawWorkflowConfig>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub style_presets: BTreeMap<String, RawStylePreset>,
    #[serde(default, skip_serializing_if = "RenderConfig::is_default")]
    pub render: RenderConfig,
    /// Unified shared configuration — single source of truth for
    /// workspace/input/output paths and the global style baseline inherited by
    /// every workflow job.
    #[serde(default, skip_serializing_if = "SharedConfig::is_default")]
    pub shared: SharedConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawStylePreset {
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub style: RawPlotStyle,
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub individual_style: RawPlotStyle,
    #[serde(default, skip_serializing_if = "RawPlotStyle::is_all_none")]
    pub combined_style: RawPlotStyle,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawPlotStyle {
    pub dpi: Option<f32>,
    pub width_inches: Option<f32>,
    pub height_inches: Option<f32>,
    pub font_size_pt: Option<f32>,
    pub line_width: Option<u32>,
    pub plot_ratio_x: Option<f32>,
    pub plot_ratio_y: Option<f32>,
    pub x_lim: Option<[f64; 2]>,
    pub y_lim: Option<[f64; 2]>,
    pub legend_font_ratio: Option<f32>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub png_font: Option<String>,
    pub svg_font: Option<String>,
    pub x_tick_scale: Option<f32>,
    pub y_tick_scale: Option<f32>,
    pub experimental_marker_radius: Option<i32>,
    pub marker_radius: Option<i32>,
    pub experimental_marker_shape: Option<PlotMarkerShape>,
    pub marker: Option<PlotMarkerShape>,
    pub experimental_line_width: Option<u32>,
    pub experimental_color: Option<RawPlotColor>,
    pub series_color: Option<RawPlotColor>,
    pub series_palette: Option<Vec<RawPlotColor>>,
    pub fitted_line_width: Option<u32>,
    pub series_line_width: Option<u32>,
    pub experimental_line_style: Option<PlotLineStyle>,
    pub line_style: Option<PlotLineStyle>,
    pub fitted_color: Option<RawPlotColor>,
    pub fitted_line_style: Option<PlotLineStyle>,
    pub legend_position: Option<PlotLegendPosition>,
    /// Geometry used for the experimental series in the generic renderer.
    pub plot_type: Option<PlotType>,
    /// Fraction of each category slot occupied by a bar.
    pub bar_width_ratio: Option<f64>,
    /// Optional category labels used by categorical bar and pie plots.
    pub category_labels: Option<Vec<String>>,
    /// Shading mode for fill-between plots.
    pub fill_between_mode: Option<FillBetweenMode>,
    /// Baseline used by `fill_between_mode = "to_baseline"`.
    pub fill_baseline: Option<f64>,
    /// Alpha used for fill and slice interiors.
    pub fill_alpha: Option<f64>,
    /// On-slice label mode for pie charts.
    pub pie_value_label_mode: Option<PieValueLabelMode>,
    /// Minimum percentage required before a slice receives an on-slice label.
    pub pie_min_label_percentage: Option<f64>,
    // Axis scaling -------------------------------------------------------
    /// Scale type for the x-axis (`"linear"` or `"log"`).
    pub x_axis_scale: Option<AxisScaleKind>,
    /// Logarithm base for the x-axis.
    ///
    /// Setting this field implies a logarithmic x-axis when `x_axis_scale`
    /// is otherwise omitted.  Defaults to `e` and must be greater than
    /// `1.0`.
    pub x_axis_log_base: Option<f64>,
    /// Scale type for the y-axis (`"linear"` or `"log"`).
    pub y_axis_scale: Option<AxisScaleKind>,
    /// Logarithm base for the y-axis.
    ///
    /// Setting this field implies a logarithmic y-axis when `y_axis_scale`
    /// is otherwise omitted.  Defaults to `e` and must be greater than
    /// `1.0`.
    pub y_axis_log_base: Option<f64>,
    // Scientific notation for tick labels --------------------------------
    /// Enable automatic scientific notation for the x-axis when the axis
    /// maximum absolute value exceeds `sci_notation_threshold_x`.
    pub sci_notation_x: Option<bool>,
    /// Threshold above which the entire x-axis uses scientific notation.
    /// Defaults to `10_000.0`.
    pub sci_notation_threshold_x: Option<f64>,
    /// Rendering style for x-axis scientific notation (`"normalized"` or
    /// `"full"`). Defaults to `"normalized"`.
    pub sci_notation_style_x: Option<ScientificNotationStyle>,
    /// Enable automatic scientific notation for the y-axis.
    pub sci_notation_y: Option<bool>,
    /// Threshold above which the entire y-axis uses scientific notation.
    /// Defaults to `10_000.0`.
    pub sci_notation_threshold_y: Option<f64>,
    /// Rendering style for y-axis scientific notation (`"normalized"` or
    /// `"full"`). Defaults to `"normalized"`.
    pub sci_notation_style_y: Option<ScientificNotationStyle>,
    // Tick label decimal precision ----------------------------------------
    /// Number of decimal places for x-axis tick labels (default `2`).
    /// Scientific-notation labels are not affected by this setting.
    pub x_tick_decimals: Option<usize>,
    /// Number of decimal places for y-axis tick labels (default `2`).
    /// Scientific-notation labels are not affected by this setting.
    pub y_tick_decimals: Option<usize>,
    // PNG supersampling --------------------------------------------------
    /// Internal upscaling factor for PNG anti-aliasing.  Overrides the value
    /// in `[render]` and `PublicationConfig::default()` (`2`) for this job.
    /// Set to `1` to disable supersampling for a specific job.
    pub png_scale_factor: Option<u32>,
    // Point selection ---------------------------------------------------
    /// Select data points by their **1-based** positions before plotting.
    ///
    /// Example: `plot_positions = [1, 5, 10, 20]` passes only the 1st, 5th,
    /// 10th, and 20th data points to the rendering pipeline.  Positions are
    /// 1-based: position 1 is the first data point, position 2 the second.
    ///
    /// Mutually exclusive with `plot_values` in the same style scope.
    pub plot_positions: Option<Vec<usize>>,
    /// Select data points by target x-values before plotting.
    ///
    /// Example: `plot_values = [2.5, 3.5, 4.5]` selects the data point
    /// whose x-value is closest to each target.  Ties are resolved in favour
    /// of the earlier data point.
    ///
    /// Mutually exclusive with `plot_positions` in the same style scope.
    pub plot_values: Option<Vec<f64>>,
    // Aggregation across files ------------------------------------------
    /// Aggregate data points across input files instead of within each file.
    ///
    /// When `true`, reorganizes selected data so each output line represents
    /// one selected x-position evaluated across **all** input files, rather
    /// than one file's full series.
    ///
    /// For example, given 5 files and `plot_values = [10, 20, 30, 40]`:
    /// - **default mode** (off): 5 lines, one per file, each with 4 points.
    /// - **aggregation mode** (on): 4 lines, one per selected x, each with
    ///   5 points — where each point is `(file_index, y_at_x)` for that x
    ///   position across files.
    ///
    /// The x-axis of the aggregated output represents the file index
    /// (0-based integer), and each line is labelled with the corresponding
    /// x-value from the original data.
    ///
    /// Applies only to directory-mode [`PlotJobKind::GenericPlot`] jobs.
    /// When no point selection (`plot_positions` / `plot_values`) is
    /// configured this flag is silently ignored.  Has no effect on
    /// single-file jobs or on EIS / regular-plot workflows.
    ///
    /// Regression overlays configured via `regression` are applied
    /// independently to each aggregated line, reusing the same regression
    /// logic as the standard mode.
    pub aggregate_points_across_files: Option<bool>,
    /// When `true`, files are sorted by modification time (oldest-first)
    /// before aggregation instead of by file name (the default).
    ///
    /// Only meaningful when `aggregate_points_across_files` is also `true`.
    /// Applies only to directory-mode [`PlotJobKind::GenericPlot`] jobs.
    pub aggregate_sort_by_mtime: Option<bool>,
    // Regression overlay ------------------------------------------------
    /// Optional regression model to overlay on the plotted data.
    ///
    /// When set, the data series is rendered as **scatter** (markers only,
    /// no connecting line) and a fitted regression curve is overlaid as a
    /// continuous line.  Omit to preserve the default line-rendering
    /// behaviour.
    ///
    /// # Accepted values
    ///
    /// | TOML value  | Model |
    /// |------------|-------|
    /// | `"linear"` | Ordinary least-squares linear fit y = m·x + b |
    ///
    /// # Example
    ///
    /// ```toml
    /// [[generic_plot]]
    /// input_dir = "data/"
    /// [generic_plot.style]
    /// regression = "linear"
    /// ```
    pub regression: Option<RegressionKind>,
    // Regression info annotation -------------------------------------------
    /// Controls which regression statistics are printed on the figure.
    ///
    /// `[show_equation, show_r_squared]` — each element is a boolean:
    /// * `show_equation` — display the fitted equation `y = m·x + b`.
    /// * `show_r_squared` — display the coefficient of determination R².
    ///
    /// Both may be `true` at the same time.  Annotations are rendered in the
    /// same color as the regression line, stacked in the upper-left corner.
    /// Omitting this field (the default) prints nothing.
    ///
    /// # Example
    ///
    /// ```toml
    /// [generic_plot.style]
    /// regression = "linear"
    /// reg_info_print = [true, true]   # show equation AND R²
    /// ```
    pub reg_info_print: Option<[bool; 2]>,
    /// Controls optional extended regression metrics shown on-plot.
    ///
    /// `[show_n, show_rmse, show_mae, show_r]`:
    /// * `show_n` — sample count used in regression.
    /// * `show_rmse` — root mean squared error.
    /// * `show_mae` — mean absolute error.
    /// * `show_r` — Pearson correlation coefficient.
    pub reg_metrics_print: Option<[bool; 4]>,
    /// Annotation layout for regression statistics.
    ///
    /// Accepted values: `"multi_line"` (default), `"single_line"`.
    pub reg_annotation_layout: Option<RegressionAnnotationLayout>,
    // Point (dot) visibility ----------------------------------------------
    /// When `true`, point markers are rendered on top of line plots for
    /// experimental series.  Defaults to `false` (line-only rendering).
    ///
    /// # Example
    /// ```toml
    /// [shared.style]
    /// show_points = true   # enable dot overlay on line plots
    /// ```
    pub show_points: Option<bool>,
    // Value transforms ---------------------------------------------------
    /// Transform type for the x-axis values before plotting.
    ///
    /// Accepted values: `"log"` (logarithmic), `"-log"` (negative
    /// logarithmic), or `"linear"` (a·x + b).
    /// When omitted, no transform is applied.
    pub x_transform: Option<TransformKind>,
    /// Logarithm base for the x-axis transform.
    ///
    /// Only meaningful when `x_transform = "log"` or `x_transform = "-log"`.
    /// Defaults to `e`.
    /// Must be greater than `1.0`.
    pub x_transform_base: Option<f64>,
    /// Multiplicative factor `a` for x-axis linear transform.
    ///
    /// Only meaningful when `x_transform = "linear"`.  Defaults to `1.0`.
    pub x_transform_a: Option<f64>,
    /// Additive offset `b` for x-axis linear transform.
    ///
    /// Only meaningful when `x_transform = "linear"`.  Defaults to `0.0`.
    pub x_transform_b: Option<f64>,
    /// Transform type for the y-axis values before plotting.
    pub y_transform: Option<TransformKind>,
    /// Logarithm base for the y-axis transform.
    ///
    /// Only meaningful when `y_transform = "log"` or `y_transform = "-log"`.
    /// Defaults to `e`.
    /// Must be greater than `1.0`.
    pub y_transform_base: Option<f64>,
    /// Multiplicative factor `a` for y-axis linear transform.
    pub y_transform_a: Option<f64>,
    /// Additive offset `b` for y-axis linear transform.
    pub y_transform_b: Option<f64>,
    // Point reassignment -------------------------------------------------
    /// Explicit x-values to assign to selected data points.
    ///
    /// When set, the selected dataset y-values are paired with these
    /// x-values instead of the dataset's own x-values.  The number of
    /// entries must exactly match the number of selected points.
    ///
    /// # Example
    /// ```toml
    /// [generic_plot.combined_style]
    /// plot_values = [0.5, 1.0, 1.5]
    /// assign_x = [10, 20, 30]   # must have 3 entries
    /// ```
    pub assign_x: Option<Vec<f64>>,
    /// Explicit y-values to assign to selected data points.
    ///
    /// When set, the selected dataset x-values are paired with these
    /// y-values instead of the dataset's own y-values.  The number of
    /// entries must exactly match the number of selected points.
    pub assign_y: Option<Vec<f64>>,
}

impl RawPlotStyle {
    /// Returns `true` when every field is `None`, meaning this style block
    /// carries no overrides and would have no effect when applied.
    ///
    /// Used by [`SharedConfig::is_default`] to determine whether the
    /// `[shared.style]` / `[shared.individual_style]` / `[shared.combined_style]`
    /// sub-tables can be omitted from the serialized TOML.
    pub fn is_all_none(&self) -> bool {
        self.dpi.is_none()
            && self.width_inches.is_none()
            && self.height_inches.is_none()
            && self.font_size_pt.is_none()
            && self.line_width.is_none()
            && self.plot_ratio_x.is_none()
            && self.plot_ratio_y.is_none()
            && self.x_lim.is_none()
            && self.y_lim.is_none()
            && self.legend_font_ratio.is_none()
            && self.x_label.is_none()
            && self.y_label.is_none()
            && self.png_font.is_none()
            && self.svg_font.is_none()
            && self.x_tick_scale.is_none()
            && self.y_tick_scale.is_none()
            && self.experimental_marker_radius.is_none()
            && self.marker_radius.is_none()
            && self.experimental_marker_shape.is_none()
            && self.marker.is_none()
            && self.experimental_line_width.is_none()
            && self.experimental_color.is_none()
            && self.series_color.is_none()
            && self.series_palette.is_none()
            && self.fitted_line_width.is_none()
            && self.series_line_width.is_none()
            && self.experimental_line_style.is_none()
            && self.line_style.is_none()
            && self.fitted_color.is_none()
            && self.fitted_line_style.is_none()
            && self.legend_position.is_none()
            && self.plot_type.is_none()
            && self.bar_width_ratio.is_none()
            && self.category_labels.is_none()
            && self.fill_between_mode.is_none()
            && self.fill_baseline.is_none()
            && self.fill_alpha.is_none()
            && self.pie_value_label_mode.is_none()
            && self.pie_min_label_percentage.is_none()
            && self.x_axis_scale.is_none()
            && self.x_axis_log_base.is_none()
            && self.y_axis_scale.is_none()
            && self.y_axis_log_base.is_none()
            && self.sci_notation_x.is_none()
            && self.sci_notation_threshold_x.is_none()
            && self.sci_notation_style_x.is_none()
            && self.sci_notation_y.is_none()
            && self.sci_notation_threshold_y.is_none()
            && self.sci_notation_style_y.is_none()
            && self.x_tick_decimals.is_none()
            && self.y_tick_decimals.is_none()
            && self.png_scale_factor.is_none()
            && self.plot_positions.is_none()
            && self.plot_values.is_none()
            && self.aggregate_points_across_files.is_none()
            && self.aggregate_sort_by_mtime.is_none()
            && self.regression.is_none()
            && self.reg_info_print.is_none()
            && self.reg_metrics_print.is_none()
            && self.reg_annotation_layout.is_none()
            && self.show_points.is_none()
            && self.x_transform.is_none()
            && self.x_transform_base.is_none()
            && self.x_transform_a.is_none()
            && self.x_transform_b.is_none()
            && self.y_transform.is_none()
            && self.y_transform_base.is_none()
            && self.y_transform_a.is_none()
            && self.y_transform_b.is_none()
            && self.assign_x.is_none()
            && self.assign_y.is_none()
    }

    pub fn merged_with(&self, overrides: &Self) -> Self {
        Self {
            dpi: overrides.dpi.or(self.dpi),
            width_inches: overrides.width_inches.or(self.width_inches),
            height_inches: overrides.height_inches.or(self.height_inches),
            font_size_pt: overrides.font_size_pt.or(self.font_size_pt),
            line_width: overrides.line_width.or(self.line_width),
            plot_ratio_x: overrides.plot_ratio_x.or(self.plot_ratio_x),
            plot_ratio_y: overrides.plot_ratio_y.or(self.plot_ratio_y),
            x_lim: overrides.x_lim.or(self.x_lim),
            y_lim: overrides.y_lim.or(self.y_lim),
            legend_font_ratio: overrides.legend_font_ratio.or(self.legend_font_ratio),
            x_label: overrides.x_label.clone().or_else(|| self.x_label.clone()),
            y_label: overrides.y_label.clone().or_else(|| self.y_label.clone()),
            png_font: overrides.png_font.clone().or_else(|| self.png_font.clone()),
            svg_font: overrides.svg_font.clone().or_else(|| self.svg_font.clone()),
            x_tick_scale: overrides.x_tick_scale.or(self.x_tick_scale),
            y_tick_scale: overrides.y_tick_scale.or(self.y_tick_scale),
            experimental_marker_radius: overrides
                .experimental_marker_radius
                .or(self.experimental_marker_radius),
            marker_radius: overrides.marker_radius.or(self.marker_radius),
            experimental_marker_shape: overrides
                .experimental_marker_shape
                .or(self.experimental_marker_shape),
            marker: overrides.marker.or(self.marker),
            experimental_line_width: overrides
                .experimental_line_width
                .or(self.experimental_line_width),
            experimental_color: overrides
                .experimental_color
                .clone()
                .or_else(|| self.experimental_color.clone()),
            series_color: overrides
                .series_color
                .clone()
                .or_else(|| self.series_color.clone()),
            series_palette: overrides
                .series_palette
                .clone()
                .or_else(|| self.series_palette.clone()),
            fitted_line_width: overrides.fitted_line_width.or(self.fitted_line_width),
            series_line_width: overrides.series_line_width.or(self.series_line_width),
            experimental_line_style: overrides
                .experimental_line_style
                .or(self.experimental_line_style),
            line_style: overrides.line_style.or(self.line_style),
            fitted_color: overrides
                .fitted_color
                .clone()
                .or_else(|| self.fitted_color.clone()),
            fitted_line_style: overrides.fitted_line_style.or(self.fitted_line_style),
            legend_position: overrides.legend_position.or(self.legend_position),
            plot_type: overrides.plot_type.or(self.plot_type),
            bar_width_ratio: overrides.bar_width_ratio.or(self.bar_width_ratio),
            category_labels: overrides
                .category_labels
                .clone()
                .or_else(|| self.category_labels.clone()),
            fill_between_mode: overrides.fill_between_mode.or(self.fill_between_mode),
            fill_baseline: overrides.fill_baseline.or(self.fill_baseline),
            fill_alpha: overrides.fill_alpha.or(self.fill_alpha),
            pie_value_label_mode: overrides.pie_value_label_mode.or(self.pie_value_label_mode),
            pie_min_label_percentage: overrides
                .pie_min_label_percentage
                .or(self.pie_min_label_percentage),
            x_axis_scale: overrides.x_axis_scale.or(self.x_axis_scale),
            x_axis_log_base: overrides.x_axis_log_base.or(self.x_axis_log_base),
            y_axis_scale: overrides.y_axis_scale.or(self.y_axis_scale),
            y_axis_log_base: overrides.y_axis_log_base.or(self.y_axis_log_base),
            sci_notation_x: overrides.sci_notation_x.or(self.sci_notation_x),
            sci_notation_threshold_x: overrides
                .sci_notation_threshold_x
                .or(self.sci_notation_threshold_x),
            sci_notation_style_x: overrides.sci_notation_style_x.or(self.sci_notation_style_x),
            sci_notation_y: overrides.sci_notation_y.or(self.sci_notation_y),
            sci_notation_threshold_y: overrides
                .sci_notation_threshold_y
                .or(self.sci_notation_threshold_y),
            sci_notation_style_y: overrides.sci_notation_style_y.or(self.sci_notation_style_y),
            x_tick_decimals: overrides.x_tick_decimals.or(self.x_tick_decimals),
            y_tick_decimals: overrides.y_tick_decimals.or(self.y_tick_decimals),
            png_scale_factor: overrides.png_scale_factor.or(self.png_scale_factor),
            plot_positions: overrides
                .plot_positions
                .clone()
                .or_else(|| self.plot_positions.clone()),
            plot_values: overrides
                .plot_values
                .clone()
                .or_else(|| self.plot_values.clone()),
            aggregate_points_across_files: overrides
                .aggregate_points_across_files
                .or(self.aggregate_points_across_files),
            aggregate_sort_by_mtime: overrides
                .aggregate_sort_by_mtime
                .or(self.aggregate_sort_by_mtime),
            regression: overrides.regression.or(self.regression),
            reg_info_print: overrides.reg_info_print.or(self.reg_info_print),
            reg_metrics_print: overrides.reg_metrics_print.or(self.reg_metrics_print),
            reg_annotation_layout: overrides
                .reg_annotation_layout
                .or(self.reg_annotation_layout),
            show_points: overrides.show_points.or(self.show_points),
            x_transform: overrides.x_transform.or(self.x_transform),
            x_transform_base: overrides.x_transform_base.or(self.x_transform_base),
            x_transform_a: overrides.x_transform_a.or(self.x_transform_a),
            x_transform_b: overrides.x_transform_b.or(self.x_transform_b),
            y_transform: overrides.y_transform.or(self.y_transform),
            y_transform_base: overrides.y_transform_base.or(self.y_transform_base),
            y_transform_a: overrides.y_transform_a.or(self.y_transform_a),
            y_transform_b: overrides.y_transform_b.or(self.y_transform_b),
            assign_x: overrides.assign_x.clone().or_else(|| self.assign_x.clone()),
            assign_y: overrides.assign_y.clone().or_else(|| self.assign_y.clone()),
        }
    }

    /// Resolve the point-selection fields of this style into a
    /// [`PointSelection`] variant, or `None` if no selection is configured.
    ///
    /// # Rules
    ///
    /// * If only `plot_positions` is set → `Some(PointSelection::Positions(...))`
    /// * If only `plot_values` is set    → `Some(PointSelection::XValues(...))`
    /// * If neither is set               → `None` (no selection, plot all points)
    ///
    /// # Errors
    ///
    /// Returns an error string if **both** `plot_positions` and `plot_values`
    /// are set in the same scope — setting both is ambiguous.
    pub fn resolve_selection(&self) -> Result<Option<PointSelection>, String> {
        match (&self.plot_positions, &self.plot_values) {
            (Some(_), Some(_)) => Err("style block sets both `plot_positions` and `plot_values`; \
                 only one selection mode may be active per scope"
                .to_string()),
            (Some(positions), None) => Ok(Some(PointSelection::Positions(positions.clone()))),
            (None, Some(values)) => Ok(Some(PointSelection::XValues(values.clone()))),
            (None, None) => Ok(None),
        }
    }

    /// Resolve the axis transforms configured in this style block.
    ///
    /// Returns an [`AxisTransforms`] that can be applied to data just before
    /// plotting.  Absent fields yield no transform for that axis.
    pub fn resolve_transforms(&self) -> Result<AxisTransforms, String> {
        resolve_axis_transforms(
            self.x_transform,
            self.x_transform_base,
            self.x_transform_a,
            self.x_transform_b,
            self.y_transform,
            self.y_transform_base,
            self.y_transform_a,
            self.y_transform_b,
        )
    }

    pub fn apply_to(&self, base: &PublicationConfig) -> Result<PublicationConfig, String> {
        let mut config = base.clone();

        if let Some(value) = self.dpi {
            config.dpi = value;
        }
        if let Some(value) = self.width_inches {
            config.width_inches = value;
        }
        if let Some(value) = self.height_inches {
            config.height_inches = value;
        }
        if let Some(value) = self.font_size_pt {
            config.font_size_pt = value;
        }
        if let Some(value) = self.line_width {
            config.line_width = value;
        }
        if let Some(value) = self.plot_ratio_x {
            config.plot_ratio_x = value;
        }
        if let Some(value) = self.plot_ratio_y {
            config.plot_ratio_y = value;
        }
        if let Some([min, max]) = self.x_lim {
            config.x_lim = Some((min, max));
        }
        if let Some([min, max]) = self.y_lim {
            config.y_lim = Some((min, max));
        }
        if let Some(value) = self.legend_font_ratio {
            config.legend_font_ratio = value;
        }
        if let Some(value) = &self.x_label {
            config.x_label = value.clone();
        }
        if let Some(value) = &self.y_label {
            config.y_label = value.clone();
        }
        if let Some(value) = &self.png_font {
            config.png_font = value.clone();
        }
        if let Some(value) = &self.svg_font {
            config.svg_font = value.clone();
        }
        if let Some(value) = self.x_tick_scale {
            config.x_tick_scale = value;
        }
        if let Some(value) = self.y_tick_scale {
            config.y_tick_scale = value;
        }
        if let Some(value) = self.experimental_marker_radius.or(self.marker_radius) {
            config.experimental_marker_radius = value;
        }
        if let Some(value) = self.experimental_marker_shape.or(self.marker) {
            config.experimental_marker_shape = value;
        }
        if let Some(value) = self.experimental_line_width.or(self.series_line_width) {
            config.experimental_line_width = Some(value);
        }
        if let Some(value) = self
            .experimental_color
            .as_ref()
            .or(self.series_color.as_ref())
        {
            config.experimental_color = Some(value.to_plot_color()?);
        }
        if let Some(values) = &self.series_palette {
            config.series_palette = values
                .iter()
                .map(|value| value.to_plot_color())
                .collect::<Result<Vec<_>, _>>()?;
        }
        if let Some(value) = self.fitted_line_width.or(self.series_line_width) {
            config.fitted_line_width = Some(value);
        }
        if let Some(value) = self.experimental_line_style.or(self.line_style) {
            config.experimental_line_style = value;
        }
        if let Some(value) = &self.fitted_color {
            config.fitted_color = Some(value.to_plot_color()?);
        }
        if let Some(value) = self.fitted_line_style {
            config.fitted_line_style = value;
        }
        if let Some(value) = self.legend_position {
            config.legend_position = value;
        }
        if let Some(value) = self.plot_type {
            config.plot_type = value;
        }
        if let Some(value) = self.bar_width_ratio {
            config.bar_width_ratio = value;
        }
        if let Some(values) = &self.category_labels {
            config.category_labels = values.clone();
        }
        if let Some(value) = self.fill_between_mode {
            config.fill_between_mode = value;
        }
        if let Some(value) = self.fill_baseline {
            config.fill_baseline = value;
        }
        if let Some(value) = self.fill_alpha {
            config.fill_alpha = value;
        }
        if let Some(value) = self.pie_value_label_mode {
            config.pie_value_label_mode = value;
        }
        if let Some(value) = self.pie_min_label_percentage {
            config.pie_min_label_percentage = value;
        }

        // Axis scaling
        if matches!(self.x_axis_scale, Some(AxisScaleKind::Linear))
            && self.x_axis_log_base.is_some()
        {
            return Err(
                "x_axis_log_base cannot be combined with x_axis_scale = \"linear\"".to_string(),
            );
        }
        if let Some(scale_kind) = self.x_axis_scale {
            config.x_scale = match scale_kind {
                AxisScaleKind::Linear => AxisScale::Linear,
                AxisScaleKind::Log => AxisScale::Log {
                    base: self.x_axis_log_base.unwrap_or(DEFAULT_LOG_BASE),
                },
            };
            config.x_scale_is_explicit = true;
        }
        if let Some(base) = self.x_axis_log_base {
            config.x_scale = AxisScale::Log { base };
            config.x_scale_is_explicit = true;
        }
        if matches!(self.y_axis_scale, Some(AxisScaleKind::Linear))
            && self.y_axis_log_base.is_some()
        {
            return Err(
                "y_axis_log_base cannot be combined with y_axis_scale = \"linear\"".to_string(),
            );
        }
        if let Some(scale_kind) = self.y_axis_scale {
            config.y_scale = match scale_kind {
                AxisScaleKind::Linear => AxisScale::Linear,
                AxisScaleKind::Log => AxisScale::Log {
                    base: self.y_axis_log_base.unwrap_or(DEFAULT_LOG_BASE),
                },
            };
            config.y_scale_is_explicit = true;
        }
        if let Some(base) = self.y_axis_log_base {
            config.y_scale = AxisScale::Log { base };
            config.y_scale_is_explicit = true;
        }

        // Scientific notation for tick labels
        if let Some(value) = self.sci_notation_x {
            config.sci_notation_x = value;
            config.sci_notation_x_is_explicit = true;
        }
        if let Some(value) = self.sci_notation_threshold_x {
            config.sci_notation_threshold_x = value;
        }
        if let Some(value) = self.sci_notation_style_x {
            config.sci_notation_style_x = value;
        }
        if let Some(value) = self.sci_notation_y {
            config.sci_notation_y = value;
            config.sci_notation_y_is_explicit = true;
        }
        if let Some(value) = self.sci_notation_threshold_y {
            config.sci_notation_threshold_y = value;
        }
        if let Some(value) = self.sci_notation_style_y {
            config.sci_notation_style_y = value;
        }

        // Tick label decimal precision
        if let Some(value) = self.x_tick_decimals {
            config.x_tick_decimals = value;
        }
        if let Some(value) = self.y_tick_decimals {
            config.y_tick_decimals = value;
        }

        // PNG supersampling scale factor
        if let Some(value) = self.png_scale_factor {
            config.png_scale_factor = value;
        }

        // Regression overlay
        if let Some(kind) = self.regression {
            config.regression = Some(kind);
        }

        // Regression info annotation
        if let Some(flags) = self.reg_info_print {
            config.reg_info_print = Some(flags);
        }
        if let Some(flags) = self.reg_metrics_print {
            config.reg_metrics_print = Some(flags);
        }
        if let Some(layout) = self.reg_annotation_layout {
            config.regression_annotation_layout = layout;
        }

        // Point (dot) visibility
        if let Some(value) = self.show_points {
            config.show_points = value;
        }

        config.validate()?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawPlotColor {
    Hex(String),
    Rgb([u8; 3]),
    RgbaObject(RawRgbaColor),
}

impl RawPlotColor {
    fn to_plot_color(&self) -> Result<PlotColor, String> {
        match self {
            Self::Hex(value) => parse_hex_color(value),
            Self::Rgb([red, green, blue]) => Ok(PlotColor::rgb(*red, *green, *blue)),
            Self::RgbaObject(value) => Ok(PlotColor::rgba(
                value.red,
                value.green,
                value.blue,
                value.alpha.unwrap_or(1.0),
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawRgbaColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: Option<f64>,
}

impl LoadedPlotConfig {
    pub fn resolve_jobs(
        &self,
        kind: PlotJobKind,
        workspace_dir: &Path,
    ) -> Result<Vec<PlotJob>, String> {
        self.config
            .resolve_jobs(kind, &self.base_dir, workspace_dir)
    }
}

/// Migrate a raw TOML value from the legacy multi-entry array format to the
/// new single-table workflow config format.
///
/// # Legacy → new conversions
///
/// * `[[eis]]` array → `[eis]` single table (styles merged; first entry wins)
/// * `[[regular_plot]]` array → `[regular_plot]` single table
/// * `[[generic_plot]]` array → `[generic_plot]` single table
/// * `[[pb_sensor]]` array → `[regular_plot]` single table (if not already set)
///
/// Path fields (`input_dir`, `output_dir`, `output_prefix`,
/// `input_is_directory`) found in the **first** array entry are moved to
/// `[shared]` with the appropriate key rename:
///
/// | array entry key    | `[shared]` key       |
/// |--------------------|----------------------|
/// | `input_dir`        | `input_path`         |
/// | `output_dir`       | `output_path`        |
/// | `output_prefix`    | `output_prefix`      |
/// | `input_is_directory` | `input_is_directory` |
///
/// Existing `[shared]` values are never overwritten — the migration only
/// fills gaps.
fn migrate_legacy_toml_value(mut root: toml::Value) -> toml::Value {
    let toml::Value::Table(ref mut table) = root else {
        return root;
    };

    /// Consume an array of job entries:
    /// - Strip path fields from every entry (return the first non-empty set).
    /// - Merge all style fields into a single table (first entry wins).
    fn consume_array(
        entries: Vec<toml::Value>,
    ) -> (
        toml::map::Map<String, toml::Value>,
        toml::map::Map<String, toml::Value>,
    ) {
        const PATH_KEYS: &[&str] = &[
            "input_dir",
            "output_dir",
            "output_prefix",
            "input_is_directory",
        ];

        let mut merged_style = toml::map::Map::new();
        let mut first_paths = toml::map::Map::new();

        for entry in entries {
            let toml::Value::Table(mut tbl) = entry else {
                continue;
            };
            // Strip path fields; capture the first non-empty set.
            let mut paths = toml::map::Map::new();
            for &pk in PATH_KEYS {
                if let Some(v) = tbl.remove(pk) {
                    paths.insert(pk.to_string(), v);
                }
            }
            if first_paths.is_empty() && !paths.is_empty() {
                first_paths = paths;
            }
            // Merge remaining style fields — first entry's values win.
            for (k, v) in tbl {
                merged_style.entry(k).or_insert(v);
            }
        }
        (merged_style, first_paths)
    }

    /// Move path fields from a per-job entry map into the `[shared]` table,
    /// only filling keys that are not already present.
    fn move_paths_to_shared(
        table: &mut toml::map::Map<String, toml::Value>,
        paths: toml::map::Map<String, toml::Value>,
    ) {
        if paths.is_empty() {
            return;
        }
        let shared = table
            .entry("shared".to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        let toml::Value::Table(shared_tbl) = shared else {
            return;
        };
        const RENAMES: &[(&str, &str)] = &[
            ("input_dir", "input_path"),
            ("output_dir", "output_path"),
            ("output_prefix", "output_prefix"),
            ("input_is_directory", "input_is_directory"),
        ];
        for (src, dst) in RENAMES {
            if let Some(v) = paths.get(*src) {
                shared_tbl
                    .entry(dst.to_string())
                    .or_insert_with(|| v.clone());
            }
        }
    }

    // ── standard workflow keys ─────────────────────────────────────────────
    for key in &["eis", "regular_plot", "generic_plot"] {
        if let Some(toml::Value::Array(entries)) = table.remove(*key) {
            if entries.is_empty() {
                continue;
            }
            let (merged, paths) = consume_array(entries);
            move_paths_to_shared(table, paths);
            if !merged.is_empty() {
                table.insert(key.to_string(), toml::Value::Table(merged));
            }
        }
    }

    // ── legacy [[pb_sensor]] → [regular_plot] ─────────────────────────────
    if let Some(toml::Value::Array(entries)) = table.remove("pb_sensor") {
        if !entries.is_empty() && !table.contains_key("regular_plot") {
            let (merged, paths) = consume_array(entries);
            move_paths_to_shared(table, paths);
            if !merged.is_empty() {
                table.insert("regular_plot".to_string(), toml::Value::Table(merged));
            }
        }
    }

    root
}

impl PlotConfig {
    /// Parse a TOML string into a `PlotConfig`, applying all forward-
    /// compatibility migrations.
    ///
    /// This is the preferred entry-point when loading from an in-memory buffer.
    /// For file-based loading use [`PlotConfig::load`].
    ///
    /// # Migration steps applied
    ///
    /// 1. **Value-level**: `[[eis]]`/`[[regular_plot]]`/`[[generic_plot]]`/`[[pb_sensor]]`
    ///    arrays are converted to single-table `[eis]`/etc. entries and any
    ///    per-job path fields are moved to `[shared]`.
    /// 2. **Struct-level**: no-op (kept for compatibility hooks).
    pub fn from_toml_str(text: &str) -> Result<Self, String> {
        let raw: toml::Value =
            toml::from_str(text).map_err(|err| format!("failed to parse TOML: {err}"))?;
        let migrated = migrate_legacy_toml_value(raw);
        let config: Self = toml::Value::try_into(migrated)
            .map_err(|err: toml::de::Error| format!("failed to load config: {err}"))?;
        Ok(config)
    }

    pub fn load(
        workspace_dir: &Path,
        override_path: Option<&Path>,
    ) -> Result<LoadedPlotConfig, String> {
        let config_path = override_path.map(|path| resolve_cli_config_path(path, workspace_dir));
        let default_path = workspace_dir.join(DEFAULT_PLOT_CONFIG_PATH);
        let legacy_default_path = workspace_dir.join(LEGACY_PLOT_CONFIG_PATH);
        let resolved_default_path = if default_path.exists() {
            default_path
        } else if legacy_default_path.exists() {
            legacy_default_path
        } else {
            workspace_dir.join(DEFAULT_PLOT_CONFIG_PATH)
        };
        let resolved_path = config_path.unwrap_or(resolved_default_path);
        let source_path = if resolved_path.exists() {
            Some(resolved_path.clone())
        } else {
            None
        };

        if override_path.is_some() && !resolved_path.exists() {
            return Err(format!(
                "plot config override does not exist: {}",
                resolved_path.display()
            ));
        }

        if !resolved_path.exists() {
            return Ok(LoadedPlotConfig {
                config: Self::default(),
                base_dir: workspace_dir.to_path_buf(),
                source_path: None,
                warnings: Vec::new(),
            });
        }

        let config_text = fs::read_to_string(&resolved_path)
            .map_err(|err| format!("failed to read {}: {err}", resolved_path.display()))?;

        if config_text.trim().is_empty() {
            return Ok(LoadedPlotConfig {
                config: Self::default(),
                base_dir: workspace_dir.to_path_buf(),
                source_path,
                warnings: Vec::new(),
            });
        }

        let config = match Self::from_toml_str(&config_text) {
            Ok(config) => config,
            Err(error) if override_path.is_none() => {
                return Ok(LoadedPlotConfig {
                    config: Self::default(),
                    base_dir: workspace_dir.to_path_buf(),
                    source_path,
                    warnings: vec![format!(
                        "failed to parse {}: {error}; defaults were used",
                        resolved_path.display()
                    )],
                });
            }
            Err(error) => {
                return Err(format!(
                    "failed to parse {}: {error}",
                    resolved_path.display()
                ));
            }
        };

        let mut warnings = config.compatibility_warnings();
        if let Some(schema_version) = config.schema_version {
            if schema_version != PLOT_CONFIG_SCHEMA_VERSION {
                warnings.push(format!(
                    "plot config schema_version {} does not match supported version {}",
                    schema_version, PLOT_CONFIG_SCHEMA_VERSION
                ));
            }
        }

        let base_dir = resolved_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_dir.to_path_buf());

        Ok(LoadedPlotConfig {
            config,
            base_dir,
            source_path,
            warnings,
        })
    }

    /// Resolve a single [`PlotJob`] for the given workflow `kind`, pulling all
    /// path settings exclusively from `[shared]`.
    ///
    /// # Path resolution
    ///
    /// | `[shared]` key       | Fallback when absent                           |
    /// |----------------------|------------------------------------------------|
    /// | `input_path`         | `workspace_dir` (for Eis / RegularPlot only)   |
    /// | `output_path`        | Same directory as the resolved `input_path`    |
    /// | `output_prefix`      | `""` (empty string)                            |
    /// | `input_is_directory` | `true`                                         |
    ///
    /// For [`PlotJobKind::GenericPlot`], an absent or empty `input_path` in
    /// `[shared]` causes this function to return an **empty** `Vec` so that
    /// callers can skip the pipeline silently.
    ///
    /// # Style resolution (highest → lowest priority)
    ///
    /// 1. Per-workflow `[<kind>.style]` / `[<kind>.individual_style]` / `[<kind>.combined_style]`
    /// 2. Named preset referenced by `[<kind>].style_preset`
    /// 3. `[shared.style]` / `[shared.individual_style]` / `[shared.combined_style]`
    pub fn resolve_jobs(
        &self,
        kind: PlotJobKind,
        config_base_dir: &Path,
        workspace_dir: &Path,
    ) -> Result<Vec<PlotJob>, String> {
        let workflow_cfg: Option<&RawWorkflowConfig> = match kind {
            PlotJobKind::Eis => self.eis.as_ref(),
            PlotJobKind::RegularPlot => self.regular_plot.as_ref(),
            PlotJobKind::GenericPlot => self.generic_plot.as_ref(),
        };

        let kind_label = match kind {
            PlotJobKind::Eis => "eis",
            PlotJobKind::RegularPlot => "regular_plot",
            PlotJobKind::GenericPlot => "generic_plot",
        };

        // ── Resolve input path ─────────────────────────────────────────────
        let input_text = self
            .shared
            .input_path
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let input_is_directory = self.shared.input_is_directory.unwrap_or(true);

        let (input_dir, validate_input) = match input_text {
            Some(text) => (resolve_path(text, config_base_dir, workspace_dir), true),
            None => {
                // GenericPlot has no implicit fallback — return empty when
                // no input is configured.
                if kind == PlotJobKind::GenericPlot {
                    return Ok(Vec::new());
                }
                // Eis / RegularPlot fall back to workspace_dir to mimic the
                // previous behaviour when no config existed.
                (workspace_dir.to_path_buf(), false)
            }
        };

        // Validate only when an explicit path was provided.
        if validate_input {
            if !input_dir.exists() {
                return Err(format!(
                    "[shared].input_path does not exist: {}",
                    input_dir.display()
                ));
            }
            if input_is_directory && !input_dir.is_dir() {
                return Err(format!(
                    "[shared].input_path is not a directory: {}",
                    input_dir.display()
                ));
            }
            if !input_is_directory && !input_dir.is_file() {
                return Err(format!(
                    "[shared].input_path is not a file: {}",
                    input_dir.display()
                ));
            }
        }

        // ── Resolve output path ────────────────────────────────────────────
        let output_dir_fallback = if input_is_directory {
            input_dir.clone()
        } else {
            input_dir
                .parent()
                .unwrap_or(input_dir.as_path())
                .to_path_buf()
        };

        let output_dir = self
            .shared
            .output_path
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|text| resolve_path(text, config_base_dir, workspace_dir))
            .unwrap_or(output_dir_fallback);

        let output_prefix = self
            .shared
            .output_prefix
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_string();

        // ── Resolve style ──────────────────────────────────────────────────
        let raw_workflow = workflow_cfg.cloned().unwrap_or_default();

        let preset_style = match raw_workflow
            .style_preset
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(preset_name) => {
                let preset = self.style_presets.get(preset_name).ok_or_else(|| {
                    format!("[{kind_label}] references unknown style_preset '{preset_name}'")
                })?;
                (
                    preset.style.clone(),
                    preset.individual_style.clone(),
                    preset.combined_style.clone(),
                )
            }
            None => (
                RawPlotStyle::default(),
                RawPlotStyle::default(),
                RawPlotStyle::default(),
            ),
        };

        // Merge order: shared → preset → workflow (right side wins).
        let merged_base = self
            .shared
            .style
            .merged_with(&preset_style.0)
            .merged_with(&raw_workflow.style);
        let merged_individual = self
            .shared
            .individual_style
            .merged_with(&preset_style.1)
            .merged_with(&raw_workflow.individual_style);
        let merged_combined = self
            .shared
            .combined_style
            .merged_with(&preset_style.2)
            .merged_with(&raw_workflow.combined_style);

        let individual_selection = PlotJobStyle {
            style: merged_base.clone(),
            individual_style: merged_individual.clone(),
            combined_style: RawPlotStyle::default(),
        }
        .resolve_individual_selection()
        .map_err(|e| format!("[{kind_label}]: {e}"))?;

        let combined_selection = PlotJobStyle {
            style: merged_base.clone(),
            individual_style: RawPlotStyle::default(),
            combined_style: merged_combined.clone(),
        }
        .resolve_combined_selection()
        .map_err(|e| format!("[{kind_label}]: {e}"))?;

        let individual_transforms = {
            let s = PlotJobStyle {
                style: merged_base.clone(),
                individual_style: merged_individual.clone(),
                combined_style: RawPlotStyle::default(),
            };
            s.resolve_individual_transforms()
                .map_err(|e| format!("[{kind_label}]: {e}"))?
        };

        let combined_transforms = {
            let s = PlotJobStyle {
                style: merged_base.clone(),
                individual_style: RawPlotStyle::default(),
                combined_style: merged_combined.clone(),
            };
            s.resolve_combined_transforms()
                .map_err(|e| format!("[{kind_label}]: {e}"))?
        };

        let (individual_assign_x, individual_assign_y) = {
            let s = PlotJobStyle {
                style: merged_base.clone(),
                individual_style: merged_individual.clone(),
                combined_style: RawPlotStyle::default(),
            };
            s.resolve_individual_assign()
        };

        let (combined_assign_x, combined_assign_y) = {
            let s = PlotJobStyle {
                style: merged_base.clone(),
                individual_style: RawPlotStyle::default(),
                combined_style: merged_combined.clone(),
            };
            s.resolve_combined_assign()
        };

        Ok(vec![PlotJob {
            input_dir,
            input_is_directory,
            output_dir,
            output_prefix,
            style: PlotJobStyle {
                style: merged_base,
                individual_style: merged_individual,
                combined_style: merged_combined.clone(),
            },
            individual_selection,
            combined_selection,
            aggregate_points_across_files: merged_combined
                .aggregate_points_across_files
                .unwrap_or(false),
            aggregate_sort_by_mtime: merged_combined.aggregate_sort_by_mtime.unwrap_or(false),
            individual_transforms,
            combined_transforms,
            individual_assign_x,
            individual_assign_y,
            combined_assign_x,
            combined_assign_y,
        }])
    }

    fn compatibility_warnings(&self) -> Vec<String> {
        Vec::new()
    }
}

fn resolve_cli_config_path(path: &Path, workspace_dir: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_dir.join(path)
    }
}

fn resolve_path(value: &str, config_base_dir: &Path, workspace_dir: &Path) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else if config_base_dir.as_os_str().is_empty() {
        workspace_dir.join(path)
    } else {
        config_base_dir.join(path)
    }
}

fn parse_hex_color(value: &str) -> Result<PlotColor, String> {
    let trimmed = value.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    match hex.len() {
        6 => {
            let red = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| format!("invalid hex color '{value}'"))?;
            let green = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| format!("invalid hex color '{value}'"))?;
            let blue = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| format!("invalid hex color '{value}'"))?;
            Ok(PlotColor::rgb(red, green, blue))
        }
        8 => {
            let red = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| format!("invalid hex color '{value}'"))?;
            let green = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| format!("invalid hex color '{value}'"))?;
            let blue = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| format!("invalid hex color '{value}'"))?;
            let alpha = u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| format!("invalid hex color '{value}'"))?
                as f64
                / 255.0;
            Ok(PlotColor::rgba(red, green, blue, alpha))
        }
        _ => Err(format!(
            "invalid hex color '{value}'; use #RRGGBB or #RRGGBBAA"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{PlotConfig, PlotJobKind, RawPlotStyle};
    use crate::DEFAULT_LOG_BASE;
    use crate::plottings::{
        AxisScale, PlotLegendPosition, PlotLineStyle, PlotMarkerShape, PublicationConfig,
        ScientificNotationStyle,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{suffix}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn shared_style_propagates_to_all_workflows() {
        let workspace = temp_dir("plot_config_shared_style");
        let eis_dir = workspace.join("data/eis");
        fs::create_dir_all(&eis_dir).expect("create eis dir");

        // Use from_toml_str so the legacy [[eis]] / [[regular_plot]] arrays
        // are migrated: input_dir moves to [shared].input_path and the
        // arrays become single-table [eis] / [regular_plot] sections.
        let config = PlotConfig::from_toml_str(
            r##"
[shared.style]
width_inches = 9.0
font_size_pt = 16.0
experimental_color = "#aabbcc"

[[eis]]
input_dir = "data/eis"

[[regular_plot]]
input_dir = "data/eis"
"##,
        )
        .expect("parse config with shared style");

        let eis_jobs = config
            .resolve_jobs(PlotJobKind::Eis, &workspace, &workspace)
            .expect("resolve eis jobs");
        let reg_jobs = config
            .resolve_jobs(PlotJobKind::RegularPlot, &workspace, &workspace)
            .expect("resolve regular jobs");

        let eis_pub = eis_jobs[0]
            .style
            .apply_to_individual(&PublicationConfig::default())
            .expect("apply eis style");
        let reg_pub = reg_jobs[0]
            .style
            .apply_to_individual(&PublicationConfig::default())
            .expect("apply reg style");

        assert_eq!(eis_pub.width_inches, 9.0, "eis: width_inches from shared");
        assert_eq!(eis_pub.font_size_pt, 16.0, "eis: font_size_pt from shared");
        assert_eq!(
            eis_pub.experimental_color.as_ref().expect("color").red,
            0xaa,
            "eis: color from shared"
        );
        assert_eq!(reg_pub.width_inches, 9.0, "reg: width_inches from shared");
        assert_eq!(
            reg_pub.experimental_color.as_ref().expect("color").blue,
            0xcc,
            "reg: color from shared"
        );
    }

    #[test]
    fn per_job_style_overrides_shared_style() {
        let workspace = temp_dir("plot_config_job_overrides_shared");
        let data_dir = workspace.join("data");
        fs::create_dir_all(&data_dir).expect("create data dir");

        // from_toml_str migrates [[eis]] → [eis] and moves input_dir to [shared].
        let config = PlotConfig::from_toml_str(
            r##"
[shared.style]
width_inches = 9.0
font_size_pt = 16.0
experimental_color = "#aabbcc"

[[eis]]
input_dir = "data"

[eis.style]
width_inches = 5.0
"##,
        )
        .expect("parse config");

        let jobs = config
            .resolve_jobs(PlotJobKind::Eis, &workspace, &workspace)
            .expect("resolve jobs");
        let pub_cfg = jobs[0]
            .style
            .apply_to_individual(&PublicationConfig::default())
            .expect("apply style");

        assert_eq!(pub_cfg.width_inches, 5.0, "job override wins");
        assert_eq!(pub_cfg.font_size_pt, 16.0, "shared font propagates");
        assert_eq!(
            pub_cfg.experimental_color.as_ref().expect("color").red,
            0xaa,
            "shared color propagates"
        );
    }

    #[test]
    fn resolves_default_job_when_config_is_empty() {
        let workspace = temp_dir("plot_config_default");

        let config = PlotConfig::default();
        let jobs = config
            .resolve_jobs(PlotJobKind::Eis, &workspace, &workspace)
            .expect("resolve default jobs");

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].input_dir, workspace);
        assert_eq!(jobs[0].output_prefix, "");
    }

    #[test]
    fn resolves_relative_paths_and_defaults_output_dir() {
        let workspace = temp_dir("plot_config_relative");
        let input_dir = workspace.join("data/eis");
        fs::create_dir_all(&input_dir).expect("create input dir");

        // from_toml_str migrates [[eis]] → [eis] and moves input_dir → shared.input_path.
        let config = PlotConfig::from_toml_str(
            r#"
[[eis]]
input_dir = "data/eis"
"#,
        )
        .expect("parse config");
        let jobs = config
            .resolve_jobs(PlotJobKind::Eis, &workspace, &workspace)
            .expect("resolve jobs");

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].input_dir, input_dir);
        assert_eq!(jobs[0].output_dir, input_dir);
        assert_eq!(jobs[0].output_prefix, "");
    }

    #[test]
    fn applies_style_overrides_to_publication_config() {
        let style: RawPlotStyle = toml::from_str(
            r##"
width_inches = 10.5
height_inches = 4.0
experimental_color = "#165E83"
legend_position = "lower_left"
"##,
        )
        .expect("parse style");

        let config = style
            .apply_to(&PublicationConfig::default())
            .expect("apply style");

        assert_eq!(config.width_inches, 10.5);
        assert_eq!(config.height_inches, 4.0);
        assert_eq!(
            config.experimental_color.expect("experimental color").red,
            0x16
        );
        assert_eq!(config.legend_position, PlotLegendPosition::LowerLeft);
    }

    #[test]
    fn regular_plot_alias_fields_apply_to_publication_config() {
        let style: RawPlotStyle = toml::from_str(
            r##"
series_color = "#165E83"
marker = "square"
marker_radius = 11
series_line_width = 6
line_style = "dashed"
"##,
        )
        .expect("parse regular plot style");

        let config = style
            .apply_to(&PublicationConfig::default())
            .expect("apply style");

        assert_eq!(config.experimental_color.expect("series color").blue, 0x83);
        assert_eq!(config.experimental_marker_radius, 11);
        assert_eq!(config.experimental_marker_shape, PlotMarkerShape::Square);
        assert_eq!(config.experimental_line_width, Some(6));
        assert_eq!(config.experimental_line_style, PlotLineStyle::Dashed);
    }

    #[test]
    fn parses_series_palette_hex_colors() {
        let style: RawPlotStyle = toml::from_str(
            r##"
series_palette = ["#165E83", "#B25019", "#2E7D32"]
"##,
        )
        .expect("parse style");

        let config = style
            .apply_to(&PublicationConfig::default())
            .expect("apply palette");

        assert_eq!(config.series_palette.len(), 3);
        assert_eq!(config.series_palette[0].red, 0x16);
        assert_eq!(config.series_palette[1].green, 0x50);
    }

    #[test]
    fn rejects_malformed_hex_palette_colors() {
        let style: RawPlotStyle = toml::from_str(
            r##"
series_palette = ["#165E8Z"]
"##,
        )
        .expect("parse style");

        let error = style
            .apply_to(&PublicationConfig::default())
            .expect_err("malformed hex should fail");

        assert!(error.contains("invalid hex color"));
    }

    #[test]
    fn merges_style_preset_with_job_overrides() {
        let workspace = temp_dir("plot_config_preset_merge");
        let input_dir = workspace.join("data/eis");
        fs::create_dir_all(&input_dir).expect("create input dir");

        // from_toml_str migrates [[eis]] array → [eis] single table.
        let config = PlotConfig::from_toml_str(
            r##"
[style_presets.paper.style]
width_inches = 7.0

[style_presets.paper.individual_style]
experimental_color = "#165E83"

[[eis]]
input_dir = "data/eis"
style_preset = "paper"
style = { height_inches = 4.5 }
individual_style = { fitted_color = "#D2691ECC" }
"##,
        )
        .expect("parse config");

        let jobs = config
            .resolve_jobs(PlotJobKind::Eis, &workspace, &workspace)
            .expect("resolve jobs");

        let individual = jobs[0]
            .style
            .apply_to_individual(&PublicationConfig::default())
            .expect("apply merged style");
        let combined = jobs[0]
            .style
            .apply_to_combined(&PublicationConfig::default())
            .expect("apply merged style");

        assert_eq!(individual.width_inches, 7.0);
        assert_eq!(individual.height_inches, 4.5);
        assert_eq!(
            individual.experimental_color.expect("preset color").red,
            0x16
        );
        assert_eq!(
            individual.fitted_color.expect("job fitted color").green,
            0x69
        );
        assert_eq!(combined.width_inches, 7.0);
    }

    #[test]
    fn rejects_unknown_style_preset() {
        let workspace = temp_dir("plot_config_unknown_preset");
        let input_dir = workspace.join("data/eis");
        fs::create_dir_all(&input_dir).expect("create input dir");

        // from_toml_str migrates [[eis]] → [eis] and moves input_dir → shared.input_path.
        let config = PlotConfig::from_toml_str(
            r#"
[[eis]]
input_dir = "data/eis"
style_preset = "missing"
"#,
        )
        .expect("parse config");
        let error = config
            .resolve_jobs(PlotJobKind::Eis, &workspace, &workspace)
            .expect_err("unknown preset should fail");

        assert!(error.contains("unknown style_preset 'missing'"));
    }

    #[test]
    fn rejects_nonpositive_figure_size() {
        let style: RawPlotStyle = toml::from_str(
            r#"
width_inches = 0.0
"#,
        )
        .expect("parse style");

        let error = style
            .apply_to(&PublicationConfig::default())
            .expect_err("invalid width should fail");

        assert!(error.contains("invalid width_inches"));
    }

    #[test]
    fn rejects_inverted_axis_limits() {
        let style: RawPlotStyle = toml::from_str(
            r#"
x_lim = [10.0, 1.0]
"#,
        )
        .expect("parse style");

        let error = style
            .apply_to(&PublicationConfig::default())
            .expect_err("invalid x_lim should fail");

        assert!(error.contains("invalid x_lim"));
    }

    #[test]
    fn migrates_legacy_pb_sensor_to_regular_plot() {
        // [[pb_sensor]] arrays are silently migrated: the entries become
        // [regular_plot] and any per-job path fields move to [shared].
        let config = PlotConfig::from_toml_str(
            r#"
[[pb_sensor]]
input_dir = "data/Pb Sensor"
style_preset = "fancy"
"#,
        )
        .expect("parse legacy config");

        assert_eq!(
            config.shared.input_path.as_deref(),
            Some("data/Pb Sensor"),
            "input_dir should be migrated to shared.input_path"
        );
        assert!(
            config.regular_plot.is_some(),
            "[[pb_sensor]] entries should migrate to [regular_plot]"
        );
        assert_eq!(
            config
                .regular_plot
                .as_ref()
                .unwrap()
                .style_preset
                .as_deref(),
            Some("fancy"),
            "style_preset should be preserved after migration"
        );
        assert!(
            config.compatibility_warnings().is_empty(),
            "no warnings after auto-migration"
        );
    }

    #[test]
    fn applies_axis_scale_and_scientific_notation_overrides() {
        let style: RawPlotStyle = toml::from_str(
            r#"
x_axis_scale = "log"
x_axis_log_base = 2.0
sci_notation_x = false
sci_notation_threshold_y = 5000.0
sci_notation_style_y = "full"
"#,
        )
        .expect("parse style");

        let config = style
            .apply_to(&PublicationConfig::default())
            .expect("apply style");

        match config.x_scale {
            AxisScale::Log { base } => assert!((base - 2.0).abs() < 1e-10),
            AxisScale::Linear => panic!("expected log x-axis"),
        }
        assert!(config.x_scale_is_explicit);
        assert!(!config.sci_notation_x);
        assert_eq!(config.sci_notation_threshold_y, 5000.0);
        assert_eq!(config.sci_notation_style_y, ScientificNotationStyle::Full);
    }

    #[test]
    fn log_base_implies_log_axis() {
        let style: RawPlotStyle = toml::from_str(
            r#"
x_axis_log_base = 5.0
"#,
        )
        .expect("parse style");

        let config = style
            .apply_to(&PublicationConfig::default())
            .expect("apply style");

        match config.x_scale {
            AxisScale::Log { base } => assert!((base - 5.0).abs() < 1e-10),
            AxisScale::Linear => panic!("expected log x-axis"),
        }
        assert!(config.x_scale_is_explicit);
    }

    #[test]
    fn log_axis_without_explicit_base_uses_default_e() {
        let style: RawPlotStyle = toml::from_str(
            r#"
x_axis_scale = "log"
"#,
        )
        .expect("parse style");

        let config = style
            .apply_to(&PublicationConfig::default())
            .expect("apply style");

        match config.x_scale {
            AxisScale::Log { base } => assert!((base - DEFAULT_LOG_BASE).abs() < 1e-10),
            AxisScale::Linear => panic!("expected log x-axis"),
        }
        assert!(config.x_scale_is_explicit);
    }

    #[test]
    fn negative_log_transform_deserializes_and_uses_default_e() {
        let style: RawPlotStyle = toml::from_str(
            r#"
x_transform = "-log"
"#,
        )
        .expect("parse style");

        let transform = style
            .resolve_transforms()
            .expect("resolve transforms")
            .x
            .expect("x transform");

        match transform {
            crate::data_file::value_transform::ValueTransform::NegLog { base } => {
                assert!((base - DEFAULT_LOG_BASE).abs() < 1e-10);
            }
            other => panic!("expected neg-log transform, got {other:?}"),
        }
    }

    #[test]
    fn rejects_linear_axis_with_log_base_override() {
        let style: RawPlotStyle = toml::from_str(
            r#"
x_axis_scale = "linear"
x_axis_log_base = 2.0
"#,
        )
        .expect("parse style");

        let error = style
            .apply_to(&PublicationConfig::default())
            .expect_err("linear axis plus log base should fail");

        assert!(error.contains("x_axis_log_base"));
    }
}
