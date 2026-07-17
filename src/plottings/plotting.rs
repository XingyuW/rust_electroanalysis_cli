#![allow(
    clippy::collapsible_if,
    clippy::too_many_arguments,
    clippy::manual_clamp,
    clippy::unnecessary_map_or
)]

use crate::domain::PlottingError;
use crate::regression_mod::{LinearFit, RegressionKind, compute_regression_with_fit};
use image::RgbaImage;
use plotters::coord::Shift;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::create_dir_all;
use std::io::Error as IoError;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlotSeriesKind {
    Experimental,
    Fitted,
    /// Regression-derived fitted curve. Rendered with the same color as the
    /// paired `Experimental` series and carries [`LinearFit`] statistics for
    /// optional on-plot annotation via `reg_info_print`.
    RegressionFit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlotAxisScale {
    Linear,
    Log10,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotLegendPosition {
    UpperLeft,
    MiddleLeft,
    LowerLeft,
    UpperMiddle,
    MiddleMiddle,
    LowerMiddle,
    UpperRight,
    MiddleRight,
    LowerRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotMarkerShape {
    Circle,
    Square,
    Triangle,
    Cross,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotLineStyle {
    Solid,
    Dashed,
}

/// High-level plot geometry selected for the generic plotting engine.
///
/// `Line` remains the default to preserve all current behavior when older
/// configs do not specify any plot type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlotType {
    #[default]
    Line,
    Scatter,
    VerticalBar,
    HorizontalBar,
    GroupedBar,
    StackedBar,
    FillBetween,
    StackPlot,
    Pie,
}

/// How fill-between plots derive the lower bound of the shaded region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FillBetweenMode {
    #[default]
    BetweenCurves,
    ToZero,
    ToBaseline,
}

/// Controls which text is printed directly on pie-chart slices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PieValueLabelMode {
    None,
    Percentage,
    Value,
    #[default]
    ValueAndPercentage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RegressionAnnotationLayout {
    #[default]
    MultiLine,
    SingleLine,
}

/// Scale kind used for TOML/serde deserialization of per-axis scale configuration.
///
/// Pair this with an optional `*_log_base` field in `RawPlotStyle` to build
/// the full `AxisScale` value stored in `PublicationConfig`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisScaleKind {
    /// Standard linear axis (default).
    Linear,
    /// Logarithmic axis.  The base is specified separately via `x_axis_log_base`
    /// / `y_axis_log_base` in the style config (default `e`).
    Log,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificNotationStyle {
    Full,
    #[default]
    Normalized,
}

/// Runtime axis scale carried by [`PublicationConfig`].
///
/// Built from [`AxisScaleKind`] plus an optional base during config resolution.
/// Drives coordinate pre-transformation and tick-label formatting inside
/// `draw_plot_area`.
#[derive(Clone, Copy, Debug, Default)]
pub enum AxisScale {
    /// Standard linear axis (default).
    #[default]
    Linear,
    /// Logarithmic axis with a configurable base.
    ///
    /// * `base` must be finite and greater than `1.0`.
    /// * Common values: `e` (natural log, default), `10.0` (decades), `2.0`
    ///   (octaves).
    Log {
        /// The logarithm base; must be finite and `> 1.0`.
        base: f64,
    },
}

impl PartialEq for AxisScale {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AxisScale::Linear, AxisScale::Linear) => true,
            (AxisScale::Log { base: b1 }, AxisScale::Log { base: b2 }) => (b1 - b2).abs() < 1e-10,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlotSeries {
    pub label: String,
    pub points: Vec<(f64, f64)>,
    pub kind: PlotSeriesKind,
    /// Regression fit statistics. Populated only for
    /// [`PlotSeriesKind::RegressionFit`] series; `None` for all others.
    pub fit_info: Option<LinearFit>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f64,
}

impl PlotColor {
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 1.0,
        }
    }

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: f64) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    fn to_rgba(self) -> RGBAColor {
        RGBAColor(self.red, self.green, self.blue, self.alpha.clamp(0.0, 1.0))
    }
}

impl PlotSeries {
    pub fn experimental(label: String, points: Vec<(f64, f64)>) -> Self {
        Self {
            label,
            points,
            kind: PlotSeriesKind::Experimental,
            fit_info: None,
        }
    }

    pub fn fitted(label: String, points: Vec<(f64, f64)>) -> Self {
        Self {
            label,
            points,
            kind: PlotSeriesKind::Fitted,
            fit_info: None,
        }
    }

    /// Construct a regression-derived fitted series carrying the [`LinearFit`]
    /// statistics needed for optional annotation.
    pub fn regression_fit(label: String, points: Vec<(f64, f64)>, fit: LinearFit) -> Self {
        Self {
            label,
            points,
            kind: PlotSeriesKind::RegressionFit,
            fit_info: Some(fit),
        }
    }
}

pub trait PlotDataSeries {
    fn label(&self) -> &str;
    fn x_values(&self) -> &[f64];
    fn y_values(&self) -> &[f64];

    fn points(&self) -> Vec<(f64, f64)> {
        self.x_values()
            .iter()
            .zip(self.y_values().iter())
            .map(|(x, y)| (*x, *y))
            .collect()
    }

    fn plot_series(&self) -> Result<Vec<PlotSeries>, PlottingError> {
        Ok(vec![PlotSeries::experimental(
            self.label().to_string(),
            self.points(),
        )])
    }
}

/// Fully-materialized, immutable plot configuration passed directly to all
/// rendering functions.
///
/// `PublicationConfig` is the **resolved** output of the configuration
/// pipeline — every field holds a concrete value.  Callers build it by
/// walking the following precedence layers (highest priority wins):
///
/// | Priority | Source | Mechanism |
/// |----------|--------|-----------|
/// | 1 (highest) | CLI arguments | `RawPlotStyle` overrides via `PlotJobStyle` |
/// | 2 | `plot_config.toml` | `RawPlotStyle::apply_to(base)` |
/// | 3 | Domain defaults (`eis_plot.rs` / `chi_plot.rs`) | `*_publication_config()` functions |
/// | 4 (lowest) | Global sentinel defaults | `PublicationConfig::default()` |
///
/// After full precedence resolution, per-plot-type axis labels are filled in
/// via [`PublicationConfig::with_default_axis_labels`], which only sets a
/// label that is still at the global sentinel value (`"X Values"` /
/// `"Y Values"`), so any explicitly user-supplied label is always preserved.
///
/// Once handed to a rendering function a `PublicationConfig` must be treated
/// as immutable — rendering functions never mutate it.
#[derive(Clone, Debug)]
pub struct PublicationConfig {
    pub dpi: f32,
    pub width_inches: f32,
    pub height_inches: f32,
    pub font_size_pt: f32,
    pub line_width: u32,
    pub plot_ratio_x: f32,
    pub plot_ratio_y: f32,
    pub x_lim: Option<(f64, f64)>,
    pub y_lim: Option<(f64, f64)>,
    pub legend_font_ratio: f32,
    pub x_label: String,
    pub y_label: String,
    pub png_font: String,
    pub svg_font: String,
    pub x_tick_scale: f32,
    pub y_tick_scale: f32,
    pub experimental_marker_radius: i32,
    pub experimental_marker_shape: PlotMarkerShape,
    pub experimental_line_width: Option<u32>,
    pub experimental_line_style: PlotLineStyle,
    pub experimental_color: Option<PlotColor>,
    pub series_palette: Vec<PlotColor>,
    pub fitted_line_width: Option<u32>,
    pub fitted_line_style: PlotLineStyle,
    pub fitted_color: Option<PlotColor>,
    pub legend_position: PlotLegendPosition,
    /// Geometry used for the experimental series.
    pub plot_type: PlotType,
    /// Bar width expressed as a fraction of the available slot width.
    pub bar_width_ratio: f64,
    /// Optional categorical labels used by bar and pie plots.
    pub category_labels: Vec<String>,
    /// Shading strategy for fill-between plots.
    pub fill_between_mode: FillBetweenMode,
    /// Baseline used when `fill_between_mode = to_baseline`.
    pub fill_baseline: f64,
    /// Alpha used for filled regions, bars, stacks, and pie slices.
    pub fill_alpha: f64,
    /// Slice-label mode for pie charts.
    pub pie_value_label_mode: PieValueLabelMode,
    /// Minimum percentage required before a slice receives an on-slice label.
    pub pie_min_label_percentage: f64,
    // ── Axis scaling ─────────────────────────────────────────────────────────
    /// Scale for the x-axis.  Defaults to [`AxisScale::Linear`].
    ///
    /// Can be overridden at call-time via the legacy `x_scale: PlotAxisScale`
    /// parameter on [`plot_rendered_series_hq`] /
    /// [`plot_rendered_series_panels_hq`].
    pub x_scale: AxisScale,
    /// Internal precedence marker for domain-default axis scaling.
    #[doc(hidden)]
    pub x_scale_is_explicit: bool,
    /// Scale for the y-axis.  Defaults to [`AxisScale::Linear`].
    pub y_scale: AxisScale,
    /// Internal precedence marker for domain-default axis scaling.
    #[doc(hidden)]
    pub y_scale_is_explicit: bool,
    /// Internal flag to render logarithmic x-axis tick labels as exponents
    /// rather than back-transformed values.
    #[doc(hidden)]
    pub x_log_ticks_as_exponents: bool,
    /// Internal flag to render logarithmic y-axis tick labels as exponents
    /// rather than back-transformed values.
    #[doc(hidden)]
    pub y_log_ticks_as_exponents: bool,
    // ── Scientific notation for tick labels ──────────────────────────────────
    /// When `true`, x-axis tick labels switch to scientific notation whenever
    /// the axis maximum absolute value meets or exceeds
    /// [`PublicationConfig::sci_notation_threshold_x`].
    pub sci_notation_x: bool,
    /// Internal precedence marker for domain-default scientific notation.
    #[doc(hidden)]
    pub sci_notation_x_is_explicit: bool,
    /// Absolute-value threshold above which the x-axis switches to scientific
    /// notation.  Only meaningful when `sci_notation_x` is `true`.
    pub sci_notation_threshold_x: f64,
    /// Rendering style used once scientific notation is activated on the
    /// x-axis.
    pub sci_notation_style_x: ScientificNotationStyle,
    /// When `true`, y-axis tick labels switch to scientific notation whenever
    /// the axis maximum absolute value meets or exceeds
    /// [`PublicationConfig::sci_notation_threshold_y`].
    pub sci_notation_y: bool,
    /// Internal precedence marker for domain-default scientific notation.
    #[doc(hidden)]
    pub sci_notation_y_is_explicit: bool,
    /// Absolute-value threshold above which the y-axis switches to scientific
    /// notation.  Only meaningful when `sci_notation_y` is `true`.
    pub sci_notation_threshold_y: f64,
    /// Rendering style used once scientific notation is activated on the
    /// y-axis.
    pub sci_notation_style_y: ScientificNotationStyle,
    // ── Tick label decimal precision ─────────────────────────────────────────
    /// Number of decimal places used when formatting x-axis tick labels.
    /// Defaults to `2`.  Scientific-notation labels are not affected by this.
    pub x_tick_decimals: usize,
    /// Number of decimal places used when formatting y-axis tick labels.
    /// Defaults to `2`.  Scientific-notation labels are not affected by this.
    pub y_tick_decimals: usize,
    // ── PNG supersampling anti-aliasing ──────────────────────────────────────
    /// Internal upscaling factor used when rendering PNG output.
    ///
    /// # Root cause of aliasing
    ///
    /// Plotters' `BitMapBackend` uses a Bresenham-style pixel-grid rasterizer
    /// with no subpixel blending.  Diagonal lines and curved font glyphs
    /// therefore produce visible "staircase" artifacts at any resolution.
    /// SVG output is unaffected because it is a vector format.
    ///
    /// # Fix — Supersampling Anti-Aliasing (SSAA)
    ///
    /// The PNG pipeline renders internally at `png_scale_factor × target_size`,
    /// scaling all pixel-space parameters proportionally (DPI, line widths,
    /// marker radii).  After drawing, the oversize image is downsampled to the
    /// target dimensions using a bilinear (`Triangle`) filter.  The weighted
    /// averaging across the N² source pixels that map to each output pixel
    /// smooths away staircase artifacts without altering any plotting logic or
    /// SVG output.
    ///
    /// A bilinear filter is used rather than Lanczos3 because Lanczos3 carries
    /// negative lobes (sinc-kernel ringing).  When downsampling a coloured line
    /// against the near-black sentinel background `(1, 2, 3)`, those negative
    /// lobes push interpolated edge-pixel values below zero; the image library
    /// clamps them to `(0, 0, 0)` = black.  Because only exact sentinel pixels
    /// are made transparent by the chroma-key pass, these clamped-black pixels
    /// remain fully opaque just outside every stroke, producing the visible dark
    /// halo/outline on lines and serrated edges on text glyphs.  The bilinear
    /// filter has no negative lobes — every interpolated value is a true
    /// weighted average — so edge pixels become a natural soft blend with no
    /// clamping artefacts.
    ///
    /// * `1` — disabled (raw 1× rendering, same as the old behaviour).
    /// * `2` — default; 4× pixel area, good balance of quality and memory.
    /// * `3` — higher quality at the cost of 9× pixel buffer.
    pub png_scale_factor: u32,
    // ── Regression overlay ───────────────────────────────────────────────────
    /// Optional regression model to overlay on experimental data.
    ///
    /// When `Some(kind)`, the plotting pipeline:
    /// 1. Renders experimental data as **scatter only** (no connecting line),
    ///    regardless of any `experimental_line_width` setting.
    /// 2. Computes a regression fit and overlays the result as a continuous
    ///    **fitted** line using the fitted-series style (color, line width).
    ///
    /// `None` (the default) preserves the existing rendering behavior exactly.
    pub regression: Option<RegressionKind>,
    // ── Regression info annotation ───────────────────────────────────────────
    /// Controls which regression statistics are printed on the figure.
    ///
    /// When `Some([show_eq, show_r2])`:
    /// * `show_eq` — display the fitted equation `y = m·x + b` next to each
    ///   regression line.
    /// * `show_r2` — display the coefficient of determination R².
    ///
    /// Both booleans may be `true` simultaneously.  The text is rendered in
    /// the same color as the corresponding regression line, stacked in the
    /// upper-left corner of the plot area and stepping down for each series.
    ///
    /// `None` (the default) prints nothing — backward compatible.
    pub reg_info_print: Option<[bool; 2]>,
    /// Controls optional extended regression metrics printed on the figure.
    ///
    /// Order: `[show_n, show_rmse, show_mae, show_r]`.
    pub reg_metrics_print: Option<[bool; 4]>,
    /// Layout mode for regression annotation text blocks.
    pub regression_annotation_layout: RegressionAnnotationLayout,
    /// Symbolic x-term used in rendered regression equations.
    ///
    /// Defaults to `"x"`. When data transformation is enabled this can be
    /// set to an explicit transformed expression such as `log10(x)`.
    pub regression_x_term: String,
    /// Symbolic y-term used in rendered regression equations.
    ///
    /// Defaults to `"y"`. When data transformation is enabled this can be
    /// set to an explicit transformed expression such as `log10(y)`.
    pub regression_y_term: String,
    // ── Point (dot) visibility ───────────────────────────────────────────────
    /// When `true`, point markers are drawn on top of line plots for
    /// experimental series.
    ///
    /// Defaults to `false` — lines are rendered without dot overlays unless
    /// this flag is explicitly enabled via TOML (`show_points = true`) or the
    /// Runtime toggle.
    pub show_points: bool,
}

/// Type alias that makes the role of [`PublicationConfig`] in the
/// configuration pipeline explicit.
///
/// `ResolvedPlotConfig` **is** `PublicationConfig` — they are the same type
/// under the hood.  The alias exists so that call-sites near the end of the
/// config resolution pipeline can use a name that reflects the semantics
/// ("resolved") rather than the rendering concern ("publication"):
///
/// ```text
/// CLI args → load TOML → merge with global defaults
///          → PlotJobStyle::apply_to_individual/combined
///          → ResolvedPlotConfig   ──►   plot_hq / plot_rendered_series_hq
/// ```
///
/// Using `ResolvedPlotConfig` at function boundaries signals to the reader
/// that configuration resolution is **complete** and the value is ready to
/// be consumed by a rendering function.  Using `PublicationConfig` is
/// equally valid and refers to the same concrete type.
pub type ResolvedPlotConfig = PublicationConfig;

impl Default for PublicationConfig {
    fn default() -> Self {
        Self {
            dpi: 300.0,
            width_inches: 8.0,
            height_inches: 6.0,
            font_size_pt: 25.0,
            line_width: 3,
            plot_ratio_x: 10.0,
            plot_ratio_y: 6.0,
            x_lim: None,
            y_lim: None,
            legend_font_ratio: 0.75,
            x_label: "X Values".to_string(),
            y_label: "Y Values".to_string(),
            png_font: "Arial".to_string(),
            svg_font: "Arial".to_string(),
            x_tick_scale: 1.0,
            y_tick_scale: 1.0,
            experimental_marker_radius: 8,
            experimental_marker_shape: PlotMarkerShape::Circle,
            experimental_line_width: None,
            experimental_line_style: PlotLineStyle::Solid,
            experimental_color: None,
            series_palette: vec![
                PlotColor::rgb(24, 78, 119),
                PlotColor::rgb(193, 18, 31),
                PlotColor::rgb(58, 134, 90),
                PlotColor::rgb(138, 43, 226),
                PlotColor::rgb(219, 122, 32),
                PlotColor::rgb(0, 119, 182),
                PlotColor::rgb(90, 24, 154),
                PlotColor::rgb(106, 76, 147),
            ],
            fitted_line_width: None,
            fitted_line_style: PlotLineStyle::Solid,
            fitted_color: None,
            legend_position: PlotLegendPosition::UpperRight,
            plot_type: PlotType::Line,
            bar_width_ratio: 0.72,
            category_labels: Vec::new(),
            fill_between_mode: FillBetweenMode::BetweenCurves,
            fill_baseline: 0.0,
            fill_alpha: 0.35,
            pie_value_label_mode: PieValueLabelMode::ValueAndPercentage,
            pie_min_label_percentage: 3.0,
            x_scale: AxisScale::Linear,
            x_scale_is_explicit: false,
            y_scale: AxisScale::Linear,
            y_scale_is_explicit: false,
            x_log_ticks_as_exponents: false,
            y_log_ticks_as_exponents: false,
            sci_notation_x: true,
            sci_notation_x_is_explicit: false,
            sci_notation_threshold_x: 10_000.0,
            sci_notation_style_x: ScientificNotationStyle::Normalized,
            sci_notation_y: true,
            sci_notation_y_is_explicit: false,
            sci_notation_threshold_y: 10_000.0,
            sci_notation_style_y: ScientificNotationStyle::Normalized,
            x_tick_decimals: 2,
            y_tick_decimals: 2,
            png_scale_factor: 2,
            regression: None,
            reg_info_print: None,
            reg_metrics_print: None,
            regression_annotation_layout: RegressionAnnotationLayout::MultiLine,
            regression_x_term: "x".to_string(),
            regression_y_term: "y".to_string(),
            show_points: false,
        }
    }
}

impl PublicationConfig {
    /// Validate that all rendering and axis parameters are numerically valid
    /// before chart construction.
    ///
    /// Returns descriptive string errors for user-facing config diagnostics.
    pub fn validate(&self) -> Result<(), crate::domain::ConfigurationError> {
        if !self.dpi.is_finite() || self.dpi <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid dpi: expected a positive finite value",
            ));
        }
        if !self.width_inches.is_finite() || self.width_inches <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid width_inches: expected a positive finite value",
            ));
        }
        if !self.height_inches.is_finite() || self.height_inches <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid height_inches: expected a positive finite value",
            ));
        }
        if !self.font_size_pt.is_finite() || self.font_size_pt <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid font_size_pt: expected a positive finite value",
            ));
        }
        if self.line_width == 0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid line_width: expected a value greater than zero",
            ));
        }
        if !self.plot_ratio_x.is_finite() || self.plot_ratio_x <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid plot_ratio_x: expected a positive finite value",
            ));
        }
        if !self.plot_ratio_y.is_finite() || self.plot_ratio_y <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid plot_ratio_y: expected a positive finite value",
            ));
        }
        if !self.legend_font_ratio.is_finite() || self.legend_font_ratio <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid legend_font_ratio: expected a positive finite value",
            ));
        }
        if !self.x_tick_scale.is_finite() || self.x_tick_scale <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid x_tick_scale: expected a positive finite value",
            ));
        }
        if !self.y_tick_scale.is_finite() || self.y_tick_scale <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid y_tick_scale: expected a positive finite value",
            ));
        }
        if self.experimental_marker_radius <= 0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid experimental_marker_radius: expected a value greater than zero",
            ));
        }
        if let Some(width) = self.experimental_line_width {
            if width == 0 {
                return Err(crate::domain::ConfigurationError::invalid(
                    "invalid experimental_line_width: expected a value greater than zero",
                ));
            }
        }
        if let Some(width) = self.fitted_line_width {
            if width == 0 {
                return Err(crate::domain::ConfigurationError::invalid(
                    "invalid fitted_line_width: expected a value greater than zero",
                ));
            }
        }
        if !self.bar_width_ratio.is_finite() || self.bar_width_ratio <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid bar_width_ratio: expected a positive finite value",
            ));
        }
        if !self.fill_baseline.is_finite() {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid fill_baseline: expected a finite value",
            ));
        }
        if !self.fill_alpha.is_finite() || !(0.0..=1.0).contains(&self.fill_alpha) {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid fill_alpha: expected a finite value in [0, 1]",
            ));
        }
        if !self.pie_min_label_percentage.is_finite() || self.pie_min_label_percentage < 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid pie_min_label_percentage: expected a finite value >= 0",
            ));
        }
        if let Some((xmin, xmax)) = self.x_lim {
            if !xmin.is_finite() || !xmax.is_finite() || xmin >= xmax {
                return Err(crate::domain::ConfigurationError::invalid(
                    "invalid x_lim: expected finite values with min < max",
                ));
            }
        }
        if let Some((ymin, ymax)) = self.y_lim {
            if !ymin.is_finite() || !ymax.is_finite() || ymin >= ymax {
                return Err(crate::domain::ConfigurationError::invalid(
                    "invalid y_lim: expected finite values with min < max",
                ));
            }
        }

        // Axis scale validation
        if let AxisScale::Log { base } = self.x_scale {
            if !base.is_finite() || base <= 1.0 {
                return Err(crate::domain::ConfigurationError::invalid(
                    "invalid x_scale log base: must be a finite value greater than 1.0",
                ));
            }
        }
        if let AxisScale::Log { base } = self.y_scale {
            if !base.is_finite() || base <= 1.0 {
                return Err(crate::domain::ConfigurationError::invalid(
                    "invalid y_scale log base: must be a finite value greater than 1.0",
                ));
            }
        }

        // Scientific-notation threshold validation
        if !self.sci_notation_threshold_x.is_finite() || self.sci_notation_threshold_x <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid sci_notation_threshold_x: must be a positive finite value",
            ));
        }
        if !self.sci_notation_threshold_y.is_finite() || self.sci_notation_threshold_y <= 0.0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid sci_notation_threshold_y: must be a positive finite value",
            ));
        }

        // PNG supersampling scale factor
        if self.png_scale_factor == 0 {
            return Err(crate::domain::ConfigurationError::invalid(
                "invalid png_scale_factor: expected a value >= 1",
            ));
        }

        Ok(())
    }

    /// Apply domain-specific axis labels as a lowest-priority fallback.
    ///
    /// A label is set **only** when the current value is still the sentinel
    /// default (`"X Values"` / `"Y Values"` from
    /// [`PublicationConfig::default()`]).  Any label explicitly supplied
    /// through TOML or CLI overrides is preserved unchanged.
    ///
    /// # Usage
    ///
    /// Call this **after** all TOML and CLI overrides have been applied and
    /// immediately before passing the config to a rendering function:
    ///
    /// ```ignore
    /// let cfg = individual_config
    ///     .clone()
    ///     .with_default_axis_labels("Z' (Ohm)", "-Z'' (Ohm)");
    /// ```
    ///
    /// # Precedence (highest → lowest)
    ///
    /// 1. User-supplied label via TOML / CLI
    /// 2. Domain label provided to this method
    /// 3. Global sentinel `"X Values"` / `"Y Values"`
    #[must_use]
    pub fn with_default_axis_labels(mut self, x_label: &str, y_label: &str) -> Self {
        let sentinel = Self::default();
        if self.x_label == sentinel.x_label {
            self.x_label = x_label.to_string();
        }
        if self.y_label == sentinel.y_label {
            self.y_label = y_label.to_string();
        }
        self
    }

    /// Set symbolic regression equation terms used for on-plot annotations.
    #[must_use]
    pub fn with_regression_terms(
        mut self,
        x_term: impl Into<String>,
        y_term: impl Into<String>,
    ) -> Self {
        self.regression_x_term = x_term.into();
        self.regression_y_term = y_term.into();
        self
    }

    /// Apply plot-type axis-scale defaults without overriding explicit
    /// higher-priority choices from TOML or CLI-derived config layers.
    #[must_use]
    pub fn with_default_axis_scales(
        mut self,
        x_scale: Option<AxisScale>,
        y_scale: Option<AxisScale>,
    ) -> Self {
        if let Some(scale) = x_scale {
            if !self.x_scale_is_explicit {
                self.x_scale = scale;
            }
        }
        if let Some(scale) = y_scale {
            if !self.y_scale_is_explicit {
                self.y_scale = scale;
            }
        }
        self
    }

    /// Apply plot-type scientific-notation defaults without overriding
    /// explicit higher-priority choices from TOML or CLI-derived config layers.
    #[must_use]
    pub fn with_default_scientific_notation(
        mut self,
        sci_notation_x: Option<bool>,
        sci_notation_y: Option<bool>,
    ) -> Self {
        if let Some(enabled) = sci_notation_x {
            if !self.sci_notation_x_is_explicit {
                self.sci_notation_x = enabled;
            }
        }
        if let Some(enabled) = sci_notation_y {
            if !self.sci_notation_y_is_explicit {
                self.sci_notation_y = enabled;
            }
        }
        self
    }

    /// Apply plot-type defaults for whether logarithmic tick labels should be
    /// displayed as exponents instead of back-transformed values.
    #[must_use]
    pub fn with_default_log_tick_exponents(
        mut self,
        x_log_ticks_as_exponents: Option<bool>,
        y_log_ticks_as_exponents: Option<bool>,
    ) -> Self {
        if let Some(enabled) = x_log_ticks_as_exponents {
            self.x_log_ticks_as_exponents = enabled;
        }
        if let Some(enabled) = y_log_ticks_as_exponents {
            self.y_log_ticks_as_exponents = enabled;
        }
        self
    }

    /// Return a clone of this config with all **pixel-space** drawing
    /// parameters multiplied by `factor`.
    ///
    /// Called exclusively by the PNG supersampling pipeline: the scaled config
    /// drives rendering at `factor × target_resolution`; the result is then
    /// downsampled to the final output dimensions by [`finish_png_supersampled`],
    /// giving smooth anti-aliased edges on both lines and text.
    ///
    /// Fields expressed in physical/logical units (`width_inches`,
    /// `height_inches`, `font_size_pt`) are **intentionally left unchanged**
    /// because `font_size_px = (font_size_pt / 72) × dpi` and
    /// `pixel_width = width_inches × dpi` already scale correctly when `dpi`
    /// is multiplied.
    ///
    /// Returns `self.clone()` unchanged when `factor <= 1`.
    fn scale_for_supersampling(&self, factor: u32) -> Self {
        if factor <= 1 {
            return self.clone();
        }
        let f = factor as f32;
        let fi = factor as i32;
        let mut scaled = self.clone();
        // DPI drives font_size_px, margin sizes, and tick sizes inside draw_plot_area.
        scaled.dpi *= f;
        // Pixel-space stroke widths for axes, spines, and series lines.
        scaled.line_width = (self.line_width as f32 * f).round() as u32;
        // Pixel-space marker radius for experimental data points.
        scaled.experimental_marker_radius = self.experimental_marker_radius * fi;
        // Optional per-series line widths.
        scaled.experimental_line_width = self
            .experimental_line_width
            .map(|w| (w as f32 * f).round() as u32);
        scaled.fitted_line_width = self
            .fitted_line_width
            .map(|w| (w as f32 * f).round() as u32);
        scaled
    }
}

pub fn plot_hq<D: PlotDataSeries>(
    figname: &str,
    datasets: &[D],
    config: &PublicationConfig,
    plot_all_in_one: bool,
) -> Result<(), Box<dyn Error>> {
    // Convert arbitrary PlotDataSeries inputs into pre-rendered series groups.
    let rendered_series = prepare_rendered_series(datasets)?;

    // When regression is configured, augment every series group with a fitted
    // regression line and suppress the connecting line for experimental data
    // (reducing it to scatter markers only).
    if let Some(reg_kind) = config.regression {
        if !plot_type_supports_regression(config.plot_type) {
            eprintln!(
                "Warning: regression overlays are only supported for line and scatter plots; ignoring regression for {:?}",
                config.plot_type
            );
        } else {
            let augmented = augment_with_regression(&rendered_series, reg_kind)?;
            let scatter_config = PublicationConfig {
                experimental_line_width: None,
                show_points: true,
                ..config.clone()
            };
            return plot_rendered_series_hq(
                figname,
                &augmented,
                &scatter_config,
                plot_all_in_one,
                PlotAxisScale::Linear,
            );
        }
    }

    plot_rendered_series_hq(
        figname,
        &rendered_series,
        config,
        plot_all_in_one,
        PlotAxisScale::Linear,
    )
}

/// Augment each series group in `rendered_series` with a regression-fitted
/// curve, inserted as a [`PlotSeriesKind::RegressionFit`] series.
///
/// For every `Experimental` series in each group the function computes a
/// regression using `kind`, then appends a `RegressionFit` series whose
/// points span the same x-range at high resolution (200 samples) and whose
/// [`PlotSeries::fit_info`] carries the [`LinearFit`] statistics for optional
/// on-plot annotation.  Series with fewer than 2 points are skipped silently.
///
/// If regression fails for a series (e.g. all x values are identical) a
/// warning is printed to stderr and that series is left without an overlay
/// — the rest of the plot proceeds normally.
fn augment_with_regression(
    rendered_series: &[Vec<PlotSeries>],
    kind: RegressionKind,
) -> Result<Vec<Vec<PlotSeries>>, Box<dyn Error>> {
    let mut result = Vec::with_capacity(rendered_series.len());
    for group in rendered_series {
        let mut new_group = group.clone();
        for series in group
            .iter()
            .filter(|s| s.kind == PlotSeriesKind::Experimental)
        {
            if series.points.len() < 2 {
                continue;
            }
            let xs: Vec<f64> = series.points.iter().map(|(x, _)| *x).collect();
            let ys: Vec<f64> = series.points.iter().map(|(_, y)| *y).collect();
            match compute_regression_with_fit(&xs, &ys, kind) {
                Ok((fitted_points, fit)) => {
                    new_group.push(PlotSeries::regression_fit(
                        format!("{} (fit)", series.label),
                        fitted_points,
                        fit,
                    ));
                }
                Err(e) => {
                    eprintln!("Warning: regression skipped for '{}': {e}", series.label);
                }
            }
        }
        result.push(new_group);
    }
    Ok(result)
}

/// Plot one or more pre-rendered series groups to SVG and PNG files.
///
/// The `x_scale` parameter is a **legacy override** kept for backward
/// compatibility.  Passing [`PlotAxisScale::Log10`] forces a base-10 log
/// x-axis regardless of `config.x_scale`.  Passing [`PlotAxisScale::Linear`]
/// (the default) defers to `config.x_scale` (and `config.y_scale`) for both
/// axes, enabling independent per-axis control including arbitrary log bases.
pub fn plot_rendered_series_hq(
    figname: &str,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    plot_all_in_one: bool,
    x_scale: PlotAxisScale,
) -> Result<(), Box<dyn Error>> {
    // Legacy x-scale override is applied here for backward compatibility.
    if rendered_series.is_empty() {
        return Err("No datasets available for plotting".into());
    }

    // Apply the legacy x_scale override: PlotAxisScale::Log10 forces
    // config.x_scale = AxisScale::Log { base: 10.0 }.
    let config = apply_legacy_x_scale(config, x_scale);
    let config = &config;
    config
        .validate()
        .map_err(|err| -> Box<dyn Error> { err.into() })?;

    let pixel_width = (config.width_inches * config.dpi) as u32;
    let pixel_height = (config.height_inches * config.dpi) as u32;
    let png_bg = RGBColor(1, 2, 3);

    if plot_all_in_one {
        let (svg_path, png_path) = output_paths(figname)?;

        let svg_root = SVGBackend::new(&svg_path, (pixel_width, pixel_height)).into_drawing_area();
        draw_plot(
            svg_root,
            rendered_series,
            config,
            pixel_width,
            pixel_height,
            RGBAColor(255, 255, 255, 0.0),
            false,
        )?;

        // PNG: render into an oversize in-memory buffer, then downsample with
        // bilinear (Triangle) filter to achieve supersampling anti-aliasing
        // (see png_scale_factor).  Lanczos3 is intentionally avoided: its
        // negative lobes clamp to black at line/text edges against the
        // near-black sentinel background, creating visible dark halos.
        let scale = config.png_scale_factor.max(1);
        let scaled_config = config.scale_for_supersampling(scale);
        let render_w = pixel_width * scale;
        let render_h = pixel_height * scale;
        let mut buf = vec![0u8; (render_w * render_h * 3) as usize];
        {
            let png_root =
                BitMapBackend::with_buffer(&mut buf, (render_w, render_h)).into_drawing_area();
            draw_plot(
                png_root,
                rendered_series,
                &scaled_config,
                render_w,
                render_h,
                RGBAColor(png_bg.0, png_bg.1, png_bg.2, 1.0),
                true,
            )?;
        }
        finish_png_supersampled(
            buf,
            &png_path,
            pixel_width,
            pixel_height,
            render_w,
            render_h,
            scale,
            png_bg,
        )?;
    } else {
        let input_path = Path::new(figname);
        let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("electrochemical_publication_plot");

        // Pre-compute scale / render dimensions once; they are invariant across series groups.
        let scale = config.png_scale_factor.max(1);
        let scaled_config = config.scale_for_supersampling(scale);
        let render_w = pixel_width * scale;
        let render_h = pixel_height * scale;

        for (idx, series_group) in rendered_series.iter().enumerate() {
            let suffix = if series_group.is_empty() {
                format!("series_{:02}", idx + 1)
            } else {
                let label = series_group
                    .iter()
                    .find_map(|series| match series.kind {
                        PlotSeriesKind::Experimental => Some(series.label.as_str()),
                        PlotSeriesKind::Fitted | PlotSeriesKind::RegressionFit => None,
                    })
                    .or_else(|| series_group.first().map(|series| series.label.as_str()))
                    .unwrap_or("");
                sanitize_filename_component(label)
            };
            let single_name = parent.join(format!("{stem}_{suffix}"));
            let single_name = single_name.to_string_lossy().into_owned();
            let (svg_path, png_path) = output_paths(&single_name)?;

            let svg_root =
                SVGBackend::new(&svg_path, (pixel_width, pixel_height)).into_drawing_area();
            draw_plot(
                svg_root,
                std::slice::from_ref(series_group),
                config,
                pixel_width,
                pixel_height,
                RGBAColor(255, 255, 255, 0.0),
                false,
            )?;

            let mut buf = vec![0u8; (render_w * render_h * 3) as usize];
            {
                let png_root =
                    BitMapBackend::with_buffer(&mut buf, (render_w, render_h)).into_drawing_area();
                draw_plot(
                    png_root,
                    std::slice::from_ref(series_group),
                    &scaled_config,
                    render_w,
                    render_h,
                    RGBAColor(png_bg.0, png_bg.1, png_bg.2, 1.0),
                    true,
                )?;
            }
            finish_png_supersampled(
                buf,
                &png_path,
                pixel_width,
                pixel_height,
                render_w,
                render_h,
                scale,
                png_bg,
            )?;
        }
    }

    Ok(())
}

/// Plot pre-rendered series in a side-by-side panel layout.
///
/// The `x_scale` parameter is a **legacy override** kept for backward
/// compatibility.  See [`plot_rendered_series_hq`] for details.
pub fn plot_rendered_series_panels_hq(
    figname: &str,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    x_scale: PlotAxisScale,
) -> Result<(), Box<dyn Error>> {
    // Panel mode draws one series-group per panel in a single wide canvas.
    if rendered_series.is_empty() {
        return Err("No datasets available for plotting".into());
    }

    // Apply the legacy x_scale override.
    let config = apply_legacy_x_scale(config, x_scale);
    let config = &config;
    config
        .validate()
        .map_err(|err| -> Box<dyn Error> { err.into() })?;

    let panel_count = rendered_series.len() as u32;
    let pixel_width = (config.width_inches * config.dpi * panel_count as f32) as u32;
    let pixel_height = (config.height_inches * config.dpi) as u32;
    let png_bg = RGBColor(1, 2, 3);
    let (svg_path, png_path) = output_paths(figname)?;

    let svg_root = SVGBackend::new(&svg_path, (pixel_width, pixel_height)).into_drawing_area();
    draw_plot_panels(
        svg_root,
        rendered_series,
        config,
        pixel_width,
        pixel_height,
        RGBAColor(255, 255, 255, 0.0),
        false,
    )?;

    // PNG: supersample for anti-aliasing (see png_scale_factor docs).
    let scale = config.png_scale_factor.max(1);
    let scaled_config = config.scale_for_supersampling(scale);
    let render_w = pixel_width * scale;
    let render_h = pixel_height * scale;
    let mut buf = vec![0u8; (render_w * render_h * 3) as usize];
    {
        let png_root =
            BitMapBackend::with_buffer(&mut buf, (render_w, render_h)).into_drawing_area();
        draw_plot_panels(
            png_root,
            rendered_series,
            &scaled_config,
            render_w,
            render_h,
            RGBAColor(png_bg.0, png_bg.1, png_bg.2, 1.0),
            true,
        )?;
    }
    finish_png_supersampled(
        buf,
        &png_path,
        pixel_width,
        pixel_height,
        render_w,
        render_h,
        scale,
        png_bg,
    )?;

    Ok(())
}

fn prepare_rendered_series<D: PlotDataSeries>(
    datasets: &[D],
) -> Result<Vec<Vec<PlotSeries>>, Box<dyn Error>> {
    // Delegates domain-specific series expansion to each dataset implementation.
    if datasets.is_empty() {
        return Err("No datasets available for plotting".into());
    }

    datasets
        .iter()
        .map(|dataset| dataset.plot_series())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Error preparing plot series: {err}").into())
}

fn sanitize_filename_component(label: &str) -> String {
    // Keep file names stable and cross-platform safe by normalizing to
    // alnum/underscore tokens.
    let mut out = String::with_capacity(label.len());
    for ch in label.chars() {
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

fn output_paths(figname: &str) -> Result<(String, String), IoError> {
    // Always emit both SVG and PNG siblings for one logical figure name.
    let input_path = Path::new(figname);
    let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
    if !parent.as_os_str().is_empty() {
        create_dir_all(parent)?;
    }

    let extension = input_path.extension().and_then(|value| value.to_str());
    let stem = match extension {
        Some("svg") | Some("png") => input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("electrochemical_publication_plot"),
        _ => input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("electrochemical_publication_plot"),
    };

    let svg_path = parent.join(format!("{stem}.svg"));
    let png_path = parent.join(format!("{stem}.png"));

    Ok((
        svg_path.to_string_lossy().into_owned(),
        png_path.to_string_lossy().into_owned(),
    ))
}

/// Downsample a raw RGB render buffer to the target PNG dimensions and write
/// the final file with chroma-key transparency applied.
///
/// This is the second stage of the SSAA pipeline:
/// 1. `buf` contains RGB pixels rendered at `render_w × render_h`
///    (the oversize internal resolution).
/// 2. When `scale > 1`, a bilinear (`Triangle`) filter averages the N²
///    source pixels per output pixel, blending away staircase artifacts from
///    `BitMapBackend`'s non-anti-aliased rasterizer.
/// 3. The chroma-key pass marks the sentinel background colour transparent,
///    preserving the existing transparent-PNG contract.
///
/// # Why bilinear and not Lanczos3?
///
/// Lanczos3 has negative lobes in its sinc kernel.  When downsampling a
/// coloured data line rendered against the near-black sentinel `(1, 2, 3)`,
/// those negative lobes drive interpolated edge values below zero.  The
/// `image` crate clamps them to `(0, 0, 0)` = black.  Only exact sentinel
/// pixels are removed by the chroma-key pass, so these clamped-black pixels
/// stay fully opaque just outside every stroke — appearing as a dark
/// halo/outline on lines and as serrated edges on text glyphs.
///
/// The bilinear filter has *no* negative lobes.  Every edge pixel is a
/// genuine weighted average of the source pixels; values are naturally
/// bounded between the line colour and the sentinel, producing smooth
/// anti-aliased edges with no clamping artefacts.
fn finish_png_supersampled(
    buf: Vec<u8>,
    png_path: &str,
    pixel_width: u32,
    pixel_height: u32,
    render_w: u32,
    render_h: u32,
    scale: u32,
    key: RGBColor,
) -> Result<(), Box<dyn Error>> {
    let rgb_img = image::RgbImage::from_raw(render_w, render_h, buf)
        .ok_or("PNG supersampling: render buffer size does not match declared dimensions")?;

    // Downsample with bilinear (Triangle) when rendering at >1×; otherwise
    // just convert.  Triangle has no negative lobes, so no clamping to black.
    let mut rgba_img: RgbaImage = if scale > 1 {
        image::DynamicImage::ImageRgb8(rgb_img)
            .resize_exact(
                pixel_width,
                pixel_height,
                image::imageops::FilterType::Triangle,
            )
            .to_rgba8()
    } else {
        image::DynamicImage::ImageRgb8(rgb_img).to_rgba8()
    };

    // Chroma-key transparency: replace the sentinel background colour with
    // fully transparent pixels (matches the original behaviour).
    for pixel in rgba_img.pixels_mut() {
        if pixel[0] == key.0 && pixel[1] == key.1 && pixel[2] == key.2 {
            pixel[3] = 0;
        }
    }

    rgba_img.save(png_path)?;
    Ok(())
}

fn to_series_label_position(position: PlotLegendPosition) -> SeriesLabelPosition {
    // Pure enum translation between app-level and Plotters legend positions.
    match position {
        PlotLegendPosition::UpperLeft => SeriesLabelPosition::UpperLeft,
        PlotLegendPosition::MiddleLeft => SeriesLabelPosition::MiddleLeft,
        PlotLegendPosition::LowerLeft => SeriesLabelPosition::LowerLeft,
        PlotLegendPosition::UpperMiddle => SeriesLabelPosition::UpperMiddle,
        PlotLegendPosition::MiddleMiddle => SeriesLabelPosition::MiddleMiddle,
        PlotLegendPosition::LowerMiddle => SeriesLabelPosition::LowerMiddle,
        PlotLegendPosition::UpperRight => SeriesLabelPosition::UpperRight,
        PlotLegendPosition::MiddleRight => SeriesLabelPosition::MiddleRight,
        PlotLegendPosition::LowerRight => SeriesLabelPosition::LowerRight,
    }
}

/// Resolve the colour for the series at `index` from the fully-resolved config.
///
/// Resolution order (first match wins):
/// 1. `series_palette[index]` — per-series colour from the config pipeline.
///    When `index >= len` the palette cycles.
/// 2. `experimental_color` — blanket single-colour fallback (individual plots).
/// 3. Unreachable in practice: `PublicationConfig::default()` always populates
///    `series_palette`, so all indices are covered by step 1.
fn palette_color_for_series(config: &PublicationConfig, index: usize) -> PlotColor {
    if !config.series_palette.is_empty() {
        return config.series_palette[index % config.series_palette.len()];
    }
    // Palette is empty (set explicitly by a domain default that opts out);
    // fall back to the blanket experimental_color.
    config
        .experimental_color
        .unwrap_or(PlotColor::rgb(24, 78, 119))
}

// ── Axis-scale helpers ────────────────────────────────────────────────────────

/// Apply a log-base transform to a single value.  Returns the value unchanged
/// for [`AxisScale::Linear`].  The caller is responsible for ensuring `v > 0`
/// before calling with a log scale.
#[inline]
fn transform_value(v: f64, scale: AxisScale) -> f64 {
    match scale {
        AxisScale::Linear => v,
        AxisScale::Log { base } => v.log(base),
    }
}

/// Pre-transform all data series coordinates for the given axis scales.
///
/// For a log axis every coordinate is replaced by `log_base(v)`.  Returns an
/// error if any data value is non-positive on a log-scaled axis.
fn transform_rendered_series(
    rendered_series: &[Vec<PlotSeries>],
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Result<Vec<Vec<PlotSeries>>, PlottingError> {
    rendered_series
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|series| {
                    let new_points = series
                        .points
                        .iter()
                        .map(|(x, y)| {
                            let tx = match x_scale {
                                AxisScale::Linear => *x,
                                AxisScale::Log { base } => {
                                    if *x <= 0.0 {
                                        return Err(PlottingError::data(format!(
                                            "log x-axis requires positive data values (got {x})"
                                        )));
                                    }
                                    x.log(base)
                                }
                            };
                            let ty = match y_scale {
                                AxisScale::Linear => *y,
                                AxisScale::Log { base } => {
                                    if *y <= 0.0 {
                                        return Err(PlottingError::data(format!(
                                            "log y-axis requires positive data values (got {y})"
                                        )));
                                    }
                                    y.log(base)
                                }
                            };
                            Ok((tx, ty))
                        })
                        .collect::<Result<Vec<_>, PlottingError>>()?;
                    Ok(PlotSeries {
                        label: series.label.clone(),
                        points: new_points,
                        kind: series.kind,
                        fit_info: series.fit_info.clone(),
                    })
                })
                .collect::<Result<Vec<_>, PlottingError>>()
        })
        .collect::<Result<Vec<_>, PlottingError>>()
}

/// Format a single tick-label value for display on a chart axis.
///
/// * For a **linear** axis `v_transformed` is the raw data value.
/// * For a **log** axis `v_transformed` is the exponent (i.e. `log_base(data)`).
///   Only integer-valued exponents receive a label; fractional positions return
///   an empty string so that only full-power ticks are annotated.
/// * For a scientific-notation axis, every non-zero label is formatted in
///   scientific notation once the axis maximum crosses the configured
///   threshold.
/// * `decimals` controls the number of decimal places via [`format_tick`] for
///   all non-scientific labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AxisScientificNotationMode {
    Disabled,
    Full,
    Normalized { exponent: i32 },
}

#[derive(Clone, Copy, Debug)]
struct AxisTickFormatter {
    scale: AxisScale,
    log_ticks_as_exponents: bool,
    notation_mode: AxisScientificNotationMode,
    decimals: usize,
}

impl AxisTickFormatter {
    fn new(
        scale: AxisScale,
        log_ticks_as_exponents: bool,
        sci_enabled: bool,
        sci_threshold: f64,
        sci_style: ScientificNotationStyle,
        decimals: usize,
        axis_min: f64,
        axis_max: f64,
    ) -> Self {
        let notation_mode = if log_ticks_as_exponents && matches!(scale, AxisScale::Log { .. }) {
            AxisScientificNotationMode::Disabled
        } else {
            axis_scientific_notation_mode(sci_enabled, sci_threshold, sci_style, axis_min, axis_max)
        };
        Self {
            scale,
            log_ticks_as_exponents,
            notation_mode,
            decimals,
        }
    }

    fn format(self, v_transformed: f64) -> String {
        let display_value = match self.scale {
            AxisScale::Linear => v_transformed,
            AxisScale::Log { base } => {
                if (v_transformed - v_transformed.round()).abs() > 0.05 {
                    return String::new();
                }
                let exponent = v_transformed.round();
                if self.log_ticks_as_exponents {
                    return format_tick_trimmed(exponent, 0);
                }
                base.powf(exponent)
            }
        };
        match self.notation_mode {
            AxisScientificNotationMode::Disabled => format_tick(display_value, self.decimals),
            AxisScientificNotationMode::Full => format_full_scientific_value(display_value),
            AxisScientificNotationMode::Normalized { exponent } => {
                format_normalized_value(display_value, exponent, self.decimals)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct AxisDisplayFormat {
    label: String,
    tick_formatter: AxisTickFormatter,
}

impl AxisDisplayFormat {
    fn new(
        label: &str,
        scale: AxisScale,
        log_ticks_as_exponents: bool,
        sci_enabled: bool,
        sci_threshold: f64,
        sci_style: ScientificNotationStyle,
        decimals: usize,
        axis_min: f64,
        axis_max: f64,
    ) -> Self {
        let tick_formatter = AxisTickFormatter::new(
            scale,
            log_ticks_as_exponents,
            sci_enabled,
            sci_threshold,
            sci_style,
            decimals,
            axis_min,
            axis_max,
        );
        let label = if log_ticks_as_exponents && matches!(scale, AxisScale::Log { .. }) {
            label.to_string()
        } else {
            format_axis_label(label, tick_formatter.notation_mode)
        };
        Self {
            label,
            tick_formatter,
        }
    }
}

fn should_use_scientific_notation(
    sci_enabled: bool,
    sci_threshold: f64,
    axis_min: f64,
    axis_max: f64,
) -> bool {
    sci_enabled && axis_min.abs().max(axis_max.abs()) >= sci_threshold
}

fn axis_scientific_notation_mode(
    sci_enabled: bool,
    sci_threshold: f64,
    sci_style: ScientificNotationStyle,
    axis_min: f64,
    axis_max: f64,
) -> AxisScientificNotationMode {
    // Decide if/which notation mode should be active for this axis range.
    if !should_use_scientific_notation(sci_enabled, sci_threshold, axis_min, axis_max) {
        return AxisScientificNotationMode::Disabled;
    }

    match sci_style {
        ScientificNotationStyle::Full => AxisScientificNotationMode::Full,
        ScientificNotationStyle::Normalized => {
            let exponent = common_scientific_exponent(axis_min, axis_max);
            if exponent == 0 {
                AxisScientificNotationMode::Disabled
            } else {
                AxisScientificNotationMode::Normalized { exponent }
            }
        }
    }
}

fn common_scientific_exponent(axis_min: f64, axis_max: f64) -> i32 {
    let max_abs = axis_min.abs().max(axis_max.abs());
    if !max_abs.is_finite() || max_abs == 0.0 {
        0
    } else {
        max_abs.log10().floor() as i32
    }
}

fn format_axis_label(label: &str, notation_mode: AxisScientificNotationMode) -> String {
    let AxisScientificNotationMode::Normalized { exponent } = notation_mode else {
        return label.to_string();
    };

    if exponent == 0 {
        return label.to_string();
    }

    let scale_text = format!("×10^{exponent}");
    if let Some((base_label, unit)) = split_axis_label_units(label) {
        format!("{base_label} ({scale_text} {unit})")
    } else {
        format!("{label} ({scale_text})")
    }
}

fn split_axis_label_units(label: &str) -> Option<(&str, &str)> {
    let trimmed = label.trim();
    if !trimmed.ends_with(')') {
        return None;
    }
    let open_idx = trimmed.rfind(" (")?;
    let base = trimmed[..open_idx].trim();
    let unit = trimmed[open_idx + 2..trimmed.len() - 1].trim();
    if base.is_empty() || unit.is_empty() {
        None
    } else {
        Some((base, unit))
    }
}

/// Format a single value to a fixed number of decimal places, rounding (not
/// truncating) to the requested precision.  This is the canonical decimal
/// tick-label formatter consumed by the higher-level axis-formatting helpers.
///
/// * `decimals = 0` → integer display, e.g. `"42"`.
/// * `decimals = 2` → two decimal places, e.g. `"3.14"`.
///
/// Scientific-notation overrides bypass this function entirely.
fn format_tick(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return String::new();
    }
    // Round to the configured number of decimal places before formatting.
    let factor = 10_f64.powi(decimals as i32);
    let rounded = (value * factor).round() / factor;
    format!("{:.prec$}", rounded, prec = decimals)
}

/// Format a finite `f64` using full scientific notation.
fn format_full_scientific_value(v: f64) -> String {
    if !v.is_finite() {
        return String::new();
    }
    if v != 0.0 {
        let abs_v = v.abs();
        let exp = abs_v.log10().floor() as i32;
        let mantissa = v / 10_f64.powi(exp);
        if (mantissa - mantissa.round()).abs() < 0.005 {
            format!("{:.0}×10^{}", mantissa.round(), exp)
        } else {
            format!("{:.2}×10^{}", mantissa, exp)
        }
    } else {
        "0".to_string()
    }
}

fn format_normalized_value(v: f64, exponent: i32, decimals: usize) -> String {
    if !v.is_finite() {
        return String::new();
    }

    let normalized = v / 10_f64.powi(exponent);
    if !normalized.is_finite() {
        return String::new();
    }

    if normalized == 0.0 {
        return format_tick_trimmed(0.0, decimals);
    }

    let abs_v = normalized.abs();
    if (1e-3..1e4).contains(&abs_v) {
        format_tick_trimmed(normalized, decimals)
    } else {
        let exp = abs_v.log10().floor() as i32;
        let mantissa = normalized / 10_f64.powi(exp);
        if (mantissa.abs() - 1.0).abs() < 0.005 {
            if mantissa.is_sign_negative() {
                format!("-10^{exp}")
            } else {
                format!("10^{exp}")
            }
        } else {
            format!("{}×10^{}", format_tick_trimmed(mantissa, 2), exp)
        }
    }
}

fn format_tick_trimmed(value: f64, decimals: usize) -> String {
    let mut rendered = format_tick(value, decimals);
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
    }
    if rendered == "-0" {
        "0".to_string()
    } else {
        rendered
    }
}

// ── Axis-limit auto-padding ───────────────────────────────────────────────────

/// Compute padded axis bounds from a flat slice of raw data values.
///
/// Steps:
/// 1. Compute `min_val` and `max_val` from the data (non-finite values ignored).
/// 2. Compute `range = max_val - min_val`.
/// 3. Return `(min_val - range * padding, max_val + range * padding)`.
///
/// **Range-based additive padding** is sign-agnostic: it is correct for
/// all-positive, all-negative, and mixed-sign datasets without any conditional
/// branches.  For all-positive data it is equivalent to the approximation
/// `min * (1 - padding)` / `max * (1 + padding)`.
///
/// **Degenerate case** (`min ≈ max`): the range is expanded symmetrically by
/// `|value| * padding`, or by the constant `1.0` when the value is near zero.
///
/// Calling this function for combined plots (pass the aggregated slice of all
/// x/y values across every dataset) and for single plots (pass the per-dataset
/// slice) produces correctly scoped bounds in each case.
///
/// # Future extension
/// Expose `padding` as a configurable `axis_padding: f64` field in
/// [`PublicationConfig`] (and the corresponding TOML key) to let users adjust
/// the padding fraction without recompiling.
///
/// # Panics (debug only)
/// Panics in debug builds if `data` is empty.
fn compute_axis_bounds(data: &[f64], padding: f64) -> (f64, f64) {
    debug_assert!(!data.is_empty(), "compute_axis_bounds: data slice is empty");

    // Walk the data once to find finite min and max.
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    for &v in data {
        if v.is_finite() {
            if v < min_val {
                min_val = v;
            }
            if v > max_val {
                max_val = v;
            }
        }
    }

    // Guard against all-non-finite data (caught explicitly by the caller, but
    // we need valid numbers to proceed).
    if !min_val.is_finite() || !max_val.is_finite() {
        return (-1.0, 1.0);
    }

    let range = max_val - min_val;

    if range.abs() < f64::EPSILON {
        // Degenerate case: all values are equal.  Expand symmetrically so the
        // axis still has a non-zero extent.
        let delta = if min_val.abs() > f64::EPSILON {
            min_val.abs() * padding
        } else {
            1.0
        };
        return (min_val - delta, max_val + delta);
    }

    // Additive range-based padding.  Using the data range (not the values
    // themselves) means the delta is always positive and directionally
    // correct regardless of the sign of min_val / max_val.
    let pad = range * padding;
    (min_val - pad, max_val + pad)
}

// ── Apply legacy PlotAxisScale parameter ─────────────────────────────────────

/// Return a cloned config with `x_scale` overridden when the legacy
/// `PlotAxisScale` parameter requests log10.  If the parameter is `Linear`,
/// the config's own `x_scale` is preserved as-is.
fn apply_legacy_x_scale(config: &PublicationConfig, x_scale: PlotAxisScale) -> PublicationConfig {
    match x_scale {
        PlotAxisScale::Log10 => {
            let mut c = config.clone();
            c.x_scale = AxisScale::Log { base: 10.0 };
            c.x_scale_is_explicit = true;
            c
        }
        PlotAxisScale::Linear => config.clone(),
    }
}

fn plot_type_supports_regression(plot_type: PlotType) -> bool {
    matches!(plot_type, PlotType::Line | PlotType::Scatter)
}

#[derive(Clone, Debug)]
struct RemappedPlotSeries {
    series: Vec<Vec<PlotSeries>>,
    x_category_labels: Option<Vec<String>>,
    y_category_labels: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
struct StackLayer {
    label: String,
    lower: Vec<(f64, f64)>,
    upper: Vec<(f64, f64)>,
}

type PieSlices = (Vec<f64>, Vec<String>, Vec<RGBColor>);

fn effective_axis_scales(
    config: &PublicationConfig,
) -> Result<(AxisScale, AxisScale), Box<dyn Error>> {
    match config.plot_type {
        PlotType::Line | PlotType::Scatter | PlotType::FillBetween => {
            Ok((config.x_scale, config.y_scale))
        }
        PlotType::StackPlot => {
            if matches!(config.y_scale, AxisScale::Log { .. }) {
                Err("stack plots require a linear y-axis because stacked areas use additive baselines".into())
            } else {
                Ok((config.x_scale, AxisScale::Linear))
            }
        }
        PlotType::VerticalBar
        | PlotType::HorizontalBar
        | PlotType::GroupedBar
        | PlotType::StackedBar => {
            if matches!(config.x_scale, AxisScale::Log { .. })
                || matches!(config.y_scale, AxisScale::Log { .. })
            {
                Err("bar plots currently require linear axes".into())
            } else {
                Ok((AxisScale::Linear, AxisScale::Linear))
            }
        }
        PlotType::Pie => Ok((AxisScale::Linear, AxisScale::Linear)),
    }
}

fn collect_axis_values(rendered_series: &[Vec<PlotSeries>]) -> (Vec<f64>, Vec<f64>) {
    let mut x_values = Vec::new();
    let mut y_values = Vec::new();
    for series_group in rendered_series {
        for series in series_group {
            for &(x, y) in &series.points {
                x_values.push(x);
                y_values.push(y);
            }
        }
    }
    (x_values, y_values)
}

fn collect_unique_x_values(rendered_series: &[Vec<PlotSeries>]) -> Vec<f64> {
    let mut values: Vec<f64> = rendered_series
        .iter()
        .flat_map(|group| group.iter())
        .filter(|series| series.kind == PlotSeriesKind::Experimental)
        .flat_map(|series| series.points.iter().map(|(x, _)| *x))
        .filter(|value| value.is_finite())
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    values
}

fn map_x_to_slot(categories: &[f64], value: f64) -> Result<f64, Box<dyn Error>> {
    categories
        .iter()
        .position(|candidate| (*candidate - value).abs() < 1e-9)
        .map(|idx| idx as f64)
        .ok_or_else(|| format!("failed to map categorical bar position {value}").into())
}

fn remap_series_for_plot_type(
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
) -> Result<RemappedPlotSeries, Box<dyn Error>> {
    match config.plot_type {
        PlotType::VerticalBar => {
            if config.category_labels.is_empty() {
                return Ok(RemappedPlotSeries {
                    series: rendered_series.to_vec(),
                    x_category_labels: None,
                    y_category_labels: None,
                });
            }
            let expected = config.category_labels.len();
            let mapped = rendered_series
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|series| PlotSeries {
                            label: series.label.clone(),
                            points: series
                                .points
                                .iter()
                                .enumerate()
                                .map(|(idx, (_, y))| (idx as f64, *y))
                                .collect(),
                            kind: series.kind,
                            fit_info: series.fit_info.clone(),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            for group in &mapped {
                for series in group {
                    if series.kind == PlotSeriesKind::Experimental
                        && series.points.len() != expected
                    {
                        return Err(format!(
                            "category_labels has {} entries but series '{}' has {} points",
                            expected,
                            series.label,
                            series.points.len()
                        )
                        .into());
                    }
                }
            }
            Ok(RemappedPlotSeries {
                series: mapped,
                x_category_labels: Some(config.category_labels.clone()),
                y_category_labels: None,
            })
        }
        PlotType::GroupedBar | PlotType::StackedBar => {
            let categories = if config.category_labels.is_empty() {
                collect_unique_x_values(rendered_series)
            } else {
                Vec::new()
            };
            let mapped = rendered_series
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|series| {
                            let points = if config.category_labels.is_empty() {
                                series
                                    .points
                                    .iter()
                                    .map(|(x, y)| Ok((map_x_to_slot(&categories, *x)?, *y)))
                                    .collect::<Result<Vec<_>, Box<dyn Error>>>()?
                            } else {
                                series
                                    .points
                                    .iter()
                                    .enumerate()
                                    .map(|(idx, (_, y))| (idx as f64, *y))
                                    .collect::<Vec<_>>()
                            };
                            Ok(PlotSeries {
                                label: series.label.clone(),
                                points,
                                kind: series.kind,
                                fit_info: series.fit_info.clone(),
                            })
                        })
                        .collect::<Result<Vec<_>, Box<dyn Error>>>()
                })
                .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
            let labels = if config.category_labels.is_empty() {
                categories
                    .iter()
                    .map(|value| format_tick_trimmed(*value, config.x_tick_decimals))
                    .collect()
            } else {
                config.category_labels.clone()
            };
            Ok(RemappedPlotSeries {
                series: mapped,
                x_category_labels: Some(labels),
                y_category_labels: None,
            })
        }
        PlotType::HorizontalBar => {
            let labels = if config.category_labels.is_empty() {
                let categories = collect_unique_x_values(rendered_series);
                categories
                    .iter()
                    .map(|value| format_tick_trimmed(*value, config.x_tick_decimals))
                    .collect::<Vec<_>>()
            } else {
                config.category_labels.clone()
            };
            let mapped = rendered_series
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|series| PlotSeries {
                            label: series.label.clone(),
                            points: series
                                .points
                                .iter()
                                .enumerate()
                                .map(|(idx, (_, y))| (*y, idx as f64))
                                .collect(),
                            kind: series.kind,
                            fit_info: series.fit_info.clone(),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            for group in &mapped {
                for series in group {
                    if series.kind == PlotSeriesKind::Experimental
                        && series.points.len() != labels.len()
                    {
                        return Err(format!(
                            "horizontal bar categories expect {} points but series '{}' has {}",
                            labels.len(),
                            series.label,
                            series.points.len()
                        )
                        .into());
                    }
                }
            }
            Ok(RemappedPlotSeries {
                series: mapped,
                x_category_labels: None,
                y_category_labels: Some(labels),
            })
        }
        _ => Ok(RemappedPlotSeries {
            series: rendered_series.to_vec(),
            x_category_labels: None,
            y_category_labels: None,
        }),
    }
}

fn format_category_tick(value: f64, labels: &[String]) -> String {
    let rounded = value.round();
    if (value - rounded).abs() > 0.25 {
        return String::new();
    }
    let idx = rounded as isize;
    if idx < 0 {
        return String::new();
    }
    labels.get(idx as usize).cloned().unwrap_or_default()
}

fn point_y_for_x(points: &[(f64, f64)], x: f64) -> f64 {
    points
        .iter()
        .find(|(candidate, _)| (*candidate - x).abs() < 1e-9)
        .map(|(_, y)| *y)
        .unwrap_or(0.0)
}

fn build_stacked_layers(rendered_series: &[Vec<PlotSeries>]) -> Vec<StackLayer> {
    let series: Vec<&PlotSeries> = rendered_series
        .iter()
        .flat_map(|group| group.iter())
        .filter(|series| series.kind == PlotSeriesKind::Experimental)
        .collect();
    if series.is_empty() {
        return Vec::new();
    }

    let mut categories: Vec<f64> = series
        .iter()
        .flat_map(|series| series.points.iter().map(|(x, _)| *x))
        .collect();
    categories.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    categories.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    let mut cumulative = vec![0.0; categories.len()];
    let mut layers = Vec::with_capacity(series.len());
    for series in series {
        let mut lower = Vec::with_capacity(categories.len());
        let mut upper = Vec::with_capacity(categories.len());
        for (idx, x) in categories.iter().enumerate() {
            let base = cumulative[idx];
            let value = point_y_for_x(&series.points, *x);
            lower.push((*x, base));
            cumulative[idx] = base + value;
            upper.push((*x, cumulative[idx]));
        }
        layers.push(StackLayer {
            label: series.label.clone(),
            lower,
            upper,
        });
    }
    layers
}

fn adjust_chart_bounds_for_plot_type(
    x_min: &mut f64,
    x_max: &mut f64,
    y_min: &mut f64,
    y_max: &mut f64,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
) -> Result<(), Box<dyn Error>> {
    match config.plot_type {
        PlotType::VerticalBar | PlotType::GroupedBar => {
            *y_min = y_min.min(0.0);
            *y_max = y_max.max(0.0);
        }
        PlotType::HorizontalBar => {
            *x_min = x_min.min(0.0);
            *x_max = x_max.max(0.0);
        }
        PlotType::FillBetween => match config.fill_between_mode {
            FillBetweenMode::BetweenCurves => {}
            FillBetweenMode::ToZero => {
                if matches!(config.y_scale, AxisScale::Log { .. }) {
                    return Err(
                        "fill_between_mode = to_zero is not supported with a logarithmic y-axis"
                            .into(),
                    );
                }
                *y_min = y_min.min(0.0);
                *y_max = y_max.max(0.0);
            }
            FillBetweenMode::ToBaseline => {
                if matches!(config.y_scale, AxisScale::Log { .. }) && config.fill_baseline <= 0.0 {
                    return Err(
                        "fill_baseline must be positive when using a logarithmic y-axis".into(),
                    );
                }
                *y_min = y_min.min(config.fill_baseline);
                *y_max = y_max.max(config.fill_baseline);
            }
        },
        PlotType::StackedBar | PlotType::StackPlot => {
            let layers = build_stacked_layers(rendered_series);
            for layer in layers {
                for (_, value) in layer.lower.iter().chain(layer.upper.iter()) {
                    *y_min = y_min.min(*value);
                    *y_max = y_max.max(*value);
                }
            }
            *y_min = y_min.min(0.0);
            *y_max = y_max.max(0.0);
        }
        PlotType::Line | PlotType::Scatter | PlotType::Pie => {}
    }
    Ok(())
}

fn experimental_series(rendered_series: &[Vec<PlotSeries>]) -> Vec<&PlotSeries> {
    rendered_series
        .iter()
        .flat_map(|group| group.iter())
        .filter(|series| series.kind == PlotSeriesKind::Experimental)
        .collect()
}

fn build_fill_polygon(upper: &[(f64, f64)], lower: &[(f64, f64)]) -> Vec<(f64, f64)> {
    upper
        .iter()
        .copied()
        .chain(lower.iter().rev().copied())
        .collect()
}

fn draw_unlabeled_line_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    points: Vec<(f64, f64)>,
    style: ShapeStyle,
    line_style: PlotLineStyle,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    match line_style {
        PlotLineStyle::Solid => {
            chart.draw_series(LineSeries::new(points, style))?;
        }
        PlotLineStyle::Dashed => {
            let segments = dashed_segments(&points, 12);
            chart.draw_series(
                segments
                    .into_iter()
                    .map(|segment| PathElement::new(segment, style)),
            )?;
        }
    }
    Ok(())
}

fn draw_bar_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let series = experimental_series(rendered_series);
    if series.is_empty() {
        return Err("bar plots require at least one experimental series".into());
    }
    let total_width = config.bar_width_ratio.clamp(0.1, 0.95);
    let bar_width = total_width / series.len().max(1) as f64;

    for (idx, series) in series.iter().enumerate() {
        let color = palette_color_for_series(config, idx);
        let fill = PlotColor::rgba(color.red, color.green, color.blue, config.fill_alpha).to_rgba();
        let style = ShapeStyle::from(&fill).filled();
        chart
            .draw_series(series.points.iter().map(|(x, y)| {
                let left = *x - total_width / 2.0 + idx as f64 * bar_width;
                Rectangle::new([(left, 0.0), (left + bar_width, *y)], style)
            }))?
            .label(series.label.clone())
            .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], style));
    }
    Ok(())
}

fn draw_horizontal_bar_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let series = experimental_series(rendered_series);
    if series.is_empty() {
        return Err("horizontal bar plots require at least one experimental series".into());
    }
    let total_width = config.bar_width_ratio.clamp(0.1, 0.95);
    let bar_width = total_width / series.len().max(1) as f64;

    for (idx, series) in series.iter().enumerate() {
        let color = palette_color_for_series(config, idx);
        let fill = PlotColor::rgba(color.red, color.green, color.blue, config.fill_alpha).to_rgba();
        let style = ShapeStyle::from(&fill).filled();
        chart
            .draw_series(series.points.iter().map(|(x, y)| {
                let bottom = *y - total_width / 2.0 + idx as f64 * bar_width;
                Rectangle::new([(0.0, bottom), (*x, bottom + bar_width)], style)
            }))?
            .label(series.label.clone())
            .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], style));
    }
    Ok(())
}

fn draw_stacked_bar_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let layers = build_stacked_layers(rendered_series);
    if layers.is_empty() {
        return Err("stacked bar plots require at least one experimental series".into());
    }
    let width = config.bar_width_ratio.clamp(0.1, 0.95);
    for (idx, layer) in layers.iter().enumerate() {
        let color = palette_color_for_series(config, idx);
        let fill = PlotColor::rgba(color.red, color.green, color.blue, config.fill_alpha).to_rgba();
        let style = ShapeStyle::from(&fill).filled();
        chart
            .draw_series(
                layer
                    .upper
                    .iter()
                    .zip(layer.lower.iter())
                    .map(|(upper, lower)| {
                        Rectangle::new(
                            [
                                (upper.0 - width / 2.0, lower.1),
                                (upper.0 + width / 2.0, upper.1),
                            ],
                            style,
                        )
                    }),
            )?
            .label(layer.label.clone())
            .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], style));
    }
    Ok(())
}

fn draw_fill_between_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    line_width: u32,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    for (idx, group) in rendered_series.iter().enumerate() {
        let experimental: Vec<&PlotSeries> = group
            .iter()
            .filter(|series| series.kind == PlotSeriesKind::Experimental)
            .collect();
        if experimental.is_empty() {
            continue;
        }

        let upper = &experimental[0].points;
        let lower: Vec<(f64, f64)> = match config.fill_between_mode {
            FillBetweenMode::BetweenCurves => {
                if experimental.len() < 2 {
                    return Err("fill_between_mode = between_curves requires two experimental series in each dataset".into());
                }
                let mut xs: Vec<f64> = upper.iter().map(|(x, _)| *x).collect();
                xs.extend(experimental[1].points.iter().map(|(x, _)| *x));
                xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                xs.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
                xs.into_iter()
                    .map(|x| (x, point_y_for_x(&experimental[1].points, x)))
                    .collect()
            }
            FillBetweenMode::ToZero => upper.iter().map(|(x, _)| (*x, 0.0)).collect(),
            FillBetweenMode::ToBaseline => upper
                .iter()
                .map(|(x, _)| (*x, config.fill_baseline))
                .collect(),
        };

        let mut xs: Vec<f64> = upper.iter().map(|(x, _)| *x).collect();
        xs.extend(lower.iter().map(|(x, _)| *x));
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        xs.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        let aligned_upper: Vec<(f64, f64)> =
            xs.iter().map(|x| (*x, point_y_for_x(upper, *x))).collect();
        let aligned_lower: Vec<(f64, f64)> =
            xs.iter().map(|x| (*x, point_y_for_x(&lower, *x))).collect();

        let color = palette_color_for_series(config, idx);
        let fill = PlotColor::rgba(color.red, color.green, color.blue, config.fill_alpha).to_rgba();
        let fill_style = ShapeStyle::from(&fill).filled();
        let line_style = ShapeStyle::from(&color.to_rgba()).stroke_width(line_width.max(1));

        chart
            .draw_series(std::iter::once(Polygon::new(
                build_fill_polygon(&aligned_upper, &aligned_lower),
                fill_style,
            )))?
            .label(experimental[0].label.clone())
            .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], fill_style));

        draw_unlabeled_line_series_linear(
            chart,
            aligned_upper,
            line_style,
            config.experimental_line_style,
        )?;
        if matches!(config.fill_between_mode, FillBetweenMode::BetweenCurves) {
            draw_unlabeled_line_series_linear(
                chart,
                aligned_lower,
                line_style,
                config.experimental_line_style,
            )?;
        }
    }
    Ok(())
}

fn draw_stack_plot_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    line_width: u32,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let layers = build_stacked_layers(rendered_series);
    if layers.is_empty() {
        return Err("stack plots require at least one experimental series".into());
    }
    for (idx, layer) in layers.iter().enumerate() {
        let color = palette_color_for_series(config, idx);
        let fill = PlotColor::rgba(color.red, color.green, color.blue, config.fill_alpha).to_rgba();
        let fill_style = ShapeStyle::from(&fill).filled();
        let line_style = ShapeStyle::from(&color.to_rgba()).stroke_width(line_width.max(1));
        chart
            .draw_series(std::iter::once(Polygon::new(
                build_fill_polygon(&layer.upper, &layer.lower),
                fill_style,
            )))?
            .label(layer.label.clone())
            .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], fill_style));
        draw_unlabeled_line_series_linear(
            chart,
            layer.upper.clone(),
            line_style,
            config.experimental_line_style,
        )?;
    }
    Ok(())
}

fn draw_plot_series_cartesian<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    rendered_series: &[Vec<PlotSeries>],
    experimental_line_width: u32,
    data_line_width: u32,
    marker_radius: i32,
    config: &PublicationConfig,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    match config.plot_type {
        PlotType::Line => draw_plot_series_linear(
            chart,
            rendered_series,
            experimental_line_width,
            data_line_width,
            marker_radius,
            config,
            false,
        ),
        PlotType::Scatter => draw_plot_series_linear(
            chart,
            rendered_series,
            experimental_line_width,
            data_line_width,
            marker_radius,
            config,
            true,
        ),
        PlotType::VerticalBar | PlotType::GroupedBar => {
            draw_bar_series_linear(chart, rendered_series, config)
        }
        PlotType::HorizontalBar => {
            draw_horizontal_bar_series_linear(chart, rendered_series, config)
        }
        PlotType::StackedBar => draw_stacked_bar_series_linear(chart, rendered_series, config),
        PlotType::FillBetween => {
            draw_fill_between_series_linear(chart, rendered_series, config, experimental_line_width)
        }
        PlotType::StackPlot => {
            draw_stack_plot_series_linear(chart, rendered_series, config, experimental_line_width)
        }
        PlotType::Pie => Ok(()),
    }
}

fn pie_slice_label(base: &str, value: f64, total: f64, config: &PublicationConfig) -> String {
    let pct = if total > 0.0 {
        value / total * 100.0
    } else {
        0.0
    };
    let suffix = if pct < config.pie_min_label_percentage {
        String::new()
    } else {
        match config.pie_value_label_mode {
            PieValueLabelMode::None => String::new(),
            PieValueLabelMode::Percentage => format!(" ({pct:.1}%)"),
            PieValueLabelMode::Value => {
                format!(" ({})", format_tick_trimmed(value, config.y_tick_decimals))
            }
            PieValueLabelMode::ValueAndPercentage => format!(
                " ({}; {pct:.1}%)",
                format_tick_trimmed(value, config.y_tick_decimals)
            ),
        }
    };
    if base.is_empty() {
        suffix.trim().trim_matches(['(', ')']).to_string()
    } else {
        format!("{base}{suffix}")
    }
}

fn build_pie_slices(
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
) -> Result<PieSlices, Box<dyn Error>> {
    let series = experimental_series(rendered_series);
    if series.is_empty() {
        return Err("pie charts require at least one experimental series".into());
    }

    let mut values = Vec::new();
    let mut base_labels = Vec::new();
    if series.len() == 1 && series[0].points.len() > 1 {
        let labels = if config.category_labels.is_empty() {
            series[0]
                .points
                .iter()
                .map(|(x, _)| format_tick_trimmed(*x, config.x_tick_decimals))
                .collect::<Vec<_>>()
        } else {
            if config.category_labels.len() != series[0].points.len() {
                return Err(format!(
                    "category_labels has {} entries but the pie source series has {} points",
                    config.category_labels.len(),
                    series[0].points.len()
                )
                .into());
            }
            config.category_labels.clone()
        };
        for ((_, value), label) in series[0].points.iter().zip(labels) {
            if *value > 0.0 {
                values.push(*value);
                base_labels.push(label);
            }
        }
    } else {
        for series in series {
            if series.points.len() == 1 {
                let value = series.points[0].1;
                if value > 0.0 {
                    values.push(value);
                    base_labels.push(series.label.clone());
                }
                continue;
            }
            for (idx, (x, value)) in series.points.iter().enumerate() {
                if *value <= 0.0 {
                    continue;
                }
                let suffix = config
                    .category_labels
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| format_tick_trimmed(*x, config.x_tick_decimals));
                let base = if series.label.is_empty() {
                    suffix
                } else {
                    format!("{} - {}", series.label, suffix)
                };
                values.push(*value);
                base_labels.push(base);
            }
        }
    }

    if values.is_empty() {
        return Err("pie charts require at least one positive slice value".into());
    }
    let total: f64 = values.iter().sum();
    let labels = base_labels
        .iter()
        .zip(values.iter())
        .map(|(label, value)| pie_slice_label(label, *value, total, config))
        .collect::<Vec<_>>();
    let colors = (0..values.len())
        .map(|idx| {
            let color = palette_color_for_series(config, idx);
            RGBColor(color.red, color.green, color.blue)
        })
        .collect::<Vec<_>>();
    Ok((values, labels, colors))
}

fn draw_manual_legend<DB>(
    root: &DrawingArea<DB, Shift>,
    entries: &[(String, RGBColor)],
    config: &PublicationConfig,
    font_family: &str,
    font_size_px: u32,
    is_png_output: bool,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    if entries.is_empty() {
        return Ok(());
    }
    let (width, height) = root.dim_in_pixel();
    let line_h = font_size_px.max(12) as i32 + 10;
    let total_h = line_h * entries.len() as i32;
    let legend_w = (entries
        .iter()
        .map(|(label, _)| label.len() as i32)
        .max()
        .unwrap_or(12)
        * (font_size_px.max(12) as i32 / 2 + 2))
        + 40;
    let x = match config.legend_position {
        PlotLegendPosition::UpperLeft
        | PlotLegendPosition::MiddleLeft
        | PlotLegendPosition::LowerLeft => 20,
        PlotLegendPosition::UpperMiddle
        | PlotLegendPosition::MiddleMiddle
        | PlotLegendPosition::LowerMiddle => (width as i32 - legend_w) / 2,
        PlotLegendPosition::UpperRight
        | PlotLegendPosition::MiddleRight
        | PlotLegendPosition::LowerRight => width as i32 - legend_w - 20,
    };
    let y = match config.legend_position {
        PlotLegendPosition::UpperLeft
        | PlotLegendPosition::UpperMiddle
        | PlotLegendPosition::UpperRight => 20,
        PlotLegendPosition::MiddleLeft
        | PlotLegendPosition::MiddleMiddle
        | PlotLegendPosition::MiddleRight => (height as i32 - total_h) / 2,
        PlotLegendPosition::LowerLeft
        | PlotLegendPosition::LowerMiddle
        | PlotLegendPosition::LowerRight => height as i32 - total_h - 20,
    };

    let text_style = if is_png_output {
        FontDesc::from((font_family, font_size_px.max(12), FontStyle::Bold)).color(&BLACK)
    } else {
        (config.svg_font.as_str(), font_size_px.max(12))
            .into_font()
            .style(FontStyle::Bold)
            .color(&BLACK)
    };

    for (idx, (label, color)) in entries.iter().enumerate() {
        let top = y + idx as i32 * line_h;
        root.draw(&Rectangle::new(
            [(x, top + 2), (x + 16, top + 14)],
            ShapeStyle::from(color).filled(),
        ))?;
        root.draw(&Text::new(
            label.clone(),
            (x + 24, top + 14),
            text_style.clone(),
        ))?;
    }
    Ok(())
}

fn draw_pie_plot_area<DB>(
    root: DrawingArea<DB, Shift>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    pixel_width: u32,
    pixel_height: u32,
    is_png_output: bool,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let font_size_px = ((config.font_size_pt / 72.0) * config.dpi) as u32;
    let png_font_family = config.png_font.trim();
    let png_font_family = png_font_family
        .strip_suffix(" Bold")
        .unwrap_or(png_font_family)
        .trim();

    let (values, labels, colors) = build_pie_slices(rendered_series, config)?;
    let legend_on_left = matches!(
        config.legend_position,
        PlotLegendPosition::UpperLeft
            | PlotLegendPosition::MiddleLeft
            | PlotLegendPosition::LowerLeft
    );
    let legend_on_right = matches!(
        config.legend_position,
        PlotLegendPosition::UpperRight
            | PlotLegendPosition::MiddleRight
            | PlotLegendPosition::LowerRight
    );
    let legend_space = if legend_on_left || legend_on_right {
        pixel_width as i32 / 3
    } else {
        0
    };
    let center_x = if legend_on_left {
        ((pixel_width as i32 - legend_space) / 2) + legend_space
    } else if legend_on_right {
        (pixel_width as i32 - legend_space) / 2
    } else {
        pixel_width as i32 / 2
    };
    let center = (center_x, pixel_height as i32 / 2);
    let radius = (pixel_height.min(pixel_width) as f64 * 0.28).max(40.0);

    let label_style = if is_png_output {
        FontDesc::from((png_font_family, font_size_px.max(14), FontStyle::Bold))
    } else {
        (config.svg_font.as_str(), font_size_px.max(14))
            .into_font()
            .style(FontStyle::Bold)
    };

    let mut pie = Pie::new(&center, &radius, &values, &colors, &labels);
    pie.start_angle(-90.0);
    pie.label_style(label_style);
    pie.label_offset(radius * 0.08);
    root.draw(&pie)?;

    let legend_entries = labels
        .iter()
        .cloned()
        .zip(colors.iter().copied())
        .collect::<Vec<_>>();
    draw_manual_legend(
        &root,
        &legend_entries,
        config,
        png_font_family,
        ((font_size_px as f32) * config.legend_font_ratio.max(0.1)) as u32,
        is_png_output,
    )?;

    root.present()?;
    Ok(())
}

fn draw_plot<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    pixel_width: u32,
    pixel_height: u32,
    canvas_bg: RGBAColor,
    is_png_output: bool,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    // Single-canvas draw entry used for both combined and per-series output.
    root.fill(&canvas_bg)?;

    draw_plot_area(
        root,
        rendered_series,
        config,
        pixel_width,
        pixel_height,
        is_png_output,
    )
}

fn draw_plot_panels<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    pixel_width: u32,
    pixel_height: u32,
    canvas_bg: RGBAColor,
    is_png_output: bool,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    // Multi-panel draw entry: split canvas and render one group per panel.
    root.fill(&canvas_bg)?;
    let panel_count = rendered_series.len().max(1);
    let panel_width = pixel_width / panel_count as u32;
    let panels = root.split_evenly((1, panel_count));

    for (panel, series_group) in panels.into_iter().zip(rendered_series.iter()) {
        draw_plot_area(
            panel,
            std::slice::from_ref(series_group),
            config,
            panel_width,
            pixel_height,
            is_png_output,
        )?;
    }

    root.present()?;
    Ok(())
}

fn draw_plot_area<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    rendered_series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    pixel_width: u32,
    pixel_height: u32,
    is_png_output: bool,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    if matches!(config.plot_type, PlotType::Pie) {
        return draw_pie_plot_area(
            root,
            rendered_series,
            config,
            pixel_width,
            pixel_height,
            is_png_output,
        );
    }

    // Core renderer: resolve fonts/margins/axis transforms and draw chart.
    let dpi_scale = (config.dpi / 300.0).max(0.1);
    let font_size_px = ((config.font_size_pt / 72.0) * config.dpi) as u32;
    let png_font_family = config.png_font.trim();
    let png_font_family = png_font_family
        .strip_suffix(" Bold")
        .unwrap_or(png_font_family)
        .trim();

    // Keep SVG text behavior unchanged and apply a PNG-only bold-family fallback for raster output.
    let axis_font = if is_png_output {
        FontDesc::from((png_font_family, font_size_px, FontStyle::Bold))
    } else {
        (config.svg_font.as_str(), font_size_px)
            .into_font()
            .style(FontStyle::Bold)
    };
    let legend_ratio = config.legend_font_ratio.max(0.1);
    let legend_font_px = ((font_size_px as f32) * legend_ratio) as u32;
    let legend_font = if is_png_output {
        FontDesc::from((png_font_family, legend_font_px.max(12), FontStyle::Bold))
    } else {
        (config.svg_font.as_str(), legend_font_px.max(12))
            .into_font()
            .style(FontStyle::Bold)
    };

    let x_label_area = (font_size_px * 2).max((80.0 * dpi_scale) as u32);
    let y_label_area = (font_size_px * 3).max((120.0 * dpi_scale) as u32);

    // Use modest base margins so the chart does not get pushed to a corner.
    let min_side = pixel_width.min(pixel_height) as f32;
    let mut margin_left = (min_side * 0.03) as u32;
    let mut margin_right = ((min_side * 0.03) as u32).max(font_size_px * 2);
    let mut margin_bottom = (min_side * 0.03) as u32;
    let mut margin_top = (min_side * 0.03) as u32;

    // Adjust outer margins so the inner plot box follows the requested x:y ratio.
    let target_ratio = (config.plot_ratio_x.max(1.0) / config.plot_ratio_y.max(1.0)) as f64;
    let mut plot_w =
        pixel_width as i64 - y_label_area as i64 - margin_left as i64 - margin_right as i64;
    let mut plot_h =
        pixel_height as i64 - x_label_area as i64 - margin_top as i64 - margin_bottom as i64;

    if plot_w <= 0 || plot_h <= 0 {
        return Err("Canvas is too small for current font/margin settings".into());
    }

    let current_ratio = plot_w as f64 / plot_h as f64;
    if current_ratio > target_ratio {
        let desired_w = (plot_h as f64 * target_ratio).round() as i64;
        let extra_w = (plot_w - desired_w).max(0) as u32;
        margin_left += extra_w / 2;
        margin_right += extra_w - (extra_w / 2);
        plot_w = desired_w;
    } else if current_ratio < target_ratio {
        let desired_h = (plot_w as f64 / target_ratio).round() as i64;
        let extra_h = (plot_h - desired_h).max(0) as u32;
        margin_top += extra_h / 2;
        margin_bottom += extra_h - (extra_h / 2);
        plot_h = desired_h;
    }

    if plot_w <= 0 || plot_h <= 0 {
        return Err("Invalid plotting area after ratio adjustment".into());
    }

    let (raw_x_values, raw_y_values) = collect_axis_values(rendered_series);

    if raw_x_values.is_empty() {
        return Err("Parsed datasets contain no numeric points to plot".into());
    }
    let (x_scale, y_scale) = effective_axis_scales(config)?;
    let transformed_series = transform_rendered_series(rendered_series, x_scale, y_scale)?;
    let remapped = remap_series_for_plot_type(&transformed_series, config)?;
    let transformed_series = remapped.series;
    let (chart_x_values, chart_y_values) = collect_axis_values(&transformed_series);

    if chart_x_values.is_empty() {
        return Err("Parsed datasets contain no finite values to plot".into());
    }

    let (mut x_min_chart, mut x_max_chart) = if let Some(labels) = &remapped.x_category_labels {
        (-0.5, labels.len() as f64 - 0.5)
    } else {
        compute_axis_bounds(&chart_x_values, 0.05)
    };
    let (mut y_min_chart, mut y_max_chart) = if let Some(labels) = &remapped.y_category_labels {
        (-0.5, labels.len() as f64 - 0.5)
    } else {
        compute_axis_bounds(&chart_y_values, 0.05)
    };

    adjust_chart_bounds_for_plot_type(
        &mut x_min_chart,
        &mut x_max_chart,
        &mut y_min_chart,
        &mut y_max_chart,
        &transformed_series,
        config,
    )?;

    if remapped.x_category_labels.is_none() {
        if let Some((xmin, xmax)) = config.x_lim {
            if !xmin.is_finite() || !xmax.is_finite() || xmin >= xmax {
                return Err("Invalid x_lim: expected finite values with min < max".into());
            }
            x_min_chart = transform_value(xmin, x_scale);
            x_max_chart = transform_value(xmax, x_scale);
        }
    }
    if remapped.y_category_labels.is_none() {
        if let Some((ymin, ymax)) = config.y_lim {
            if !ymin.is_finite() || !ymax.is_finite() || ymin >= ymax {
                return Err("Invalid y_lim: expected finite values with min < max".into());
            }
            y_min_chart = transform_value(ymin, y_scale);
            y_max_chart = transform_value(ymax, y_scale);
        }
    }

    // For log axes request one label per integer power; for linear axes use a
    // sensible default.
    let x_label_count = match &remapped.x_category_labels {
        Some(labels) => labels.len().max(2).min(30),
        None => match x_scale {
            AxisScale::Linear => 5usize,
            AxisScale::Log { .. } => ((x_max_chart - x_min_chart).abs() as usize + 1)
                .max(2)
                .min(15),
        },
    };
    let y_label_count = match &remapped.y_category_labels {
        Some(labels) => labels.len().max(2).min(30),
        None => match y_scale {
            AxisScale::Linear => 5usize,
            AxisScale::Log { .. } => ((y_max_chart - y_min_chart).abs() as usize + 1)
                .max(2)
                .min(15),
        },
    };

    let (mut raw_x_min, mut raw_x_max) = compute_axis_bounds(&raw_x_values, 0.05);
    let (mut raw_y_min, mut raw_y_max) = compute_axis_bounds(&raw_y_values, 0.05);
    if let Some((xmin, xmax)) = config.x_lim {
        raw_x_min = xmin;
        raw_x_max = xmax;
    }
    if let Some((ymin, ymax)) = config.y_lim {
        raw_y_min = ymin;
        raw_y_max = ymax;
    }

    let x_label_text = if matches!(config.plot_type, PlotType::HorizontalBar) {
        config.y_label.as_str()
    } else {
        config.x_label.as_str()
    };
    let y_label_text = if matches!(config.plot_type, PlotType::HorizontalBar) {
        config.x_label.as_str()
    } else {
        config.y_label.as_str()
    };

    let x_axis_display = AxisDisplayFormat::new(
        x_label_text,
        x_scale,
        config.x_log_ticks_as_exponents,
        config.sci_notation_x,
        config.sci_notation_threshold_x,
        config.sci_notation_style_x,
        config.x_tick_decimals,
        raw_x_min,
        raw_x_max,
    );
    let y_axis_display = AxisDisplayFormat::new(
        y_label_text,
        y_scale,
        config.y_log_ticks_as_exponents,
        config.sci_notation_y,
        config.sci_notation_threshold_y,
        config.sci_notation_style_y,
        config.y_tick_decimals,
        raw_y_min,
        raw_y_max,
    );
    let x_tick_formatter = x_axis_display.tick_formatter;
    let y_tick_formatter = y_axis_display.tick_formatter;
    let x_axis_label = x_axis_display.label;
    let y_axis_label = y_axis_display.label;

    let mut chart_builder = ChartBuilder::on(&root);
    chart_builder
        .margin_left(margin_left)
        .margin_right(margin_right)
        .margin_bottom(margin_bottom)
        .margin_top(margin_top)
        .x_label_area_size(x_label_area)
        .y_label_area_size(y_label_area);

    let axis_style = ShapeStyle::from(&BLACK).stroke_width(config.line_width + 1);
    let spine_style = ShapeStyle::from(&BLACK).stroke_width(config.line_width + 2);
    let data_line_width = config
        .fitted_line_width
        .unwrap_or(config.line_width + 2)
        .max(1);
    let experimental_line_width = config
        .experimental_line_width
        .unwrap_or(config.line_width)
        .max(1);
    let marker_radius = config.experimental_marker_radius.max(1);
    let tick_mark_size = (font_size_px / 7).max((12.0 * dpi_scale) as u32);

    // Build a single Cartesian2d<RangedCoordf64, RangedCoordf64> chart for all
    // scale combinations.  Log-scaled axes have already been mapped into the
    // linear coordinate space by transform_rendered_series / transform_value
    // above, so no special Plotters coordinate type is required.
    let mut chart =
        chart_builder.build_cartesian_2d(x_min_chart..x_max_chart, y_min_chart..y_max_chart)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .set_all_tick_mark_size(tick_mark_size)
        .axis_style(axis_style)
        .x_labels(x_label_count)
        .y_labels(y_label_count)
        .x_label_formatter(&{
            let labels = remapped.x_category_labels.clone();
            move |v| {
                labels
                    .as_ref()
                    .map(|labels| format_category_tick(*v, labels))
                    .unwrap_or_else(|| x_tick_formatter.format(*v))
            }
        })
        .y_label_formatter(&{
            let labels = remapped.y_category_labels.clone();
            move |v| {
                labels
                    .as_ref()
                    .map(|labels| format_category_tick(*v, labels))
                    .unwrap_or_else(|| y_tick_formatter.format(*v))
            }
        })
        .label_style(axis_font.clone())
        .x_desc(x_axis_label.as_str())
        .y_desc(y_axis_label.as_str())
        .axis_desc_style(axis_font)
        .draw()?;

    // Draw the plot box (spine) using chart-space coordinates.
    chart.draw_series(std::iter::once(PathElement::new(
        vec![
            (x_min_chart, y_min_chart),
            (x_max_chart, y_min_chart),
            (x_max_chart, y_max_chart),
            (x_min_chart, y_max_chart),
            (x_min_chart, y_min_chart),
        ],
        spine_style,
    )))?;

    draw_plot_series_cartesian(
        &mut chart,
        &transformed_series,
        experimental_line_width,
        data_line_width,
        marker_radius,
        config,
    )
    .map_err(|e| format!("Error plotting data series: {e}"))?;

    // Optional regression annotation — drawn after series so text sits on top.
    let show_primary = config.reg_info_print.map_or(false, |f| f[0] || f[1]);
    let show_metrics = config
        .reg_metrics_print
        .map_or(false, |f| f[0] || f[1] || f[2] || f[3]);
    if show_primary || show_metrics {
        draw_regression_annotations(
            &mut chart,
            &transformed_series,
            config,
            x_min_chart,
            x_max_chart,
            y_min_chart,
            y_max_chart,
            png_font_family,
            legend_font_px,
            is_png_output,
        )
        .map_err(|e| format!("Error drawing regression annotations: {e}"))?;
    }

    chart
        .configure_series_labels()
        .position(to_series_label_position(config.legend_position))
        .margin(5)
        .label_font(legend_font)
        .background_style(TRANSPARENT)
        .border_style(TRANSPARENT)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Render optional regression statistics as text annotations on the plot.
///
/// Called from [`draw_plot_area`] after all series are drawn when
/// `config.reg_info_print` is set.  For each [`PlotSeriesKind::RegressionFit`]
/// series found in `series`, up to two lines are drawn in the upper-left of
/// the plot area:
///
/// * **Equation** (`flags[0] = true`): `y = m·x + b`
/// * **R²** (`flags[1] = true`): `R² = 0.9876`
///
/// Text is rendered in the same color as the paired experimental series.
/// Layout is controlled by `config.regression_annotation_layout`:
/// * `MultiLine` — one metric per line
/// * `SingleLine` — all enabled metrics on one line separated by `|`
#[allow(clippy::too_many_arguments)]
fn draw_regression_annotations<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    series: &[Vec<PlotSeries>],
    config: &PublicationConfig,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    font_family: &str,
    font_size_px: u32,
    is_png_output: bool,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let [show_eq, show_r2] = config.reg_info_print.unwrap_or([false, false]);
    let [show_n, show_rmse, show_mae, show_r] = config.reg_metrics_print.unwrap_or([false; 4]);
    if !show_eq && !show_r2 && !show_n && !show_rmse && !show_mae && !show_r {
        return Ok(());
    }

    let x_range = x_max - x_min;
    let y_range = y_max - y_min;
    // Inset the annotation block 4 % from the left and 6 % from the top.
    let x_start = x_min + 0.04 * x_range;
    // Each text line steps down by ~7 % of the y-range; series blocks are
    // separated by an additional 2 % gap.
    let line_step = 0.07 * y_range;
    let block_gap = 0.02 * y_range;
    let mut y_cursor = y_max - 0.07 * y_range;

    let ann_font_size = font_size_px.max(12);
    let compact = matches!(
        config.regression_annotation_layout,
        RegressionAnnotationLayout::SingleLine
    );

    for (idx, group) in series.iter().enumerate() {
        for reg_series in group
            .iter()
            .filter(|s| s.kind == PlotSeriesKind::RegressionFit)
        {
            let fit = match &reg_series.fit_info {
                Some(f) => f,
                None => continue,
            };

            let ann_color = palette_color_for_series(config, idx).to_rgba();

            let text_style = if is_png_output {
                FontDesc::from((font_family, ann_font_size, FontStyle::Bold)).color(&ann_color)
            } else {
                (font_family, ann_font_size)
                    .into_font()
                    .style(FontStyle::Bold)
                    .color(&ann_color)
            };

            let mut metric_lines: Vec<String> = Vec::new();
            if show_eq {
                let sign = if fit.intercept >= 0.0 { "+" } else { "-" };
                metric_lines.push(format!(
                    "{} = {:.3}\u{00b7}{} {} {:.3}",
                    config.regression_y_term,
                    fit.slope,
                    config.regression_x_term,
                    sign,
                    fit.intercept.abs()
                ));
            }
            if show_r2 {
                metric_lines.push(format!("R\u{00b2} = {:.4}", fit.r_squared));
            }
            if show_n {
                metric_lines.push(format!("n = {}", fit.sample_count));
            }
            if show_rmse {
                metric_lines.push(format!("RMSE = {:.4}", fit.rmse));
            }
            if show_mae {
                metric_lines.push(format!("MAE = {:.4}", fit.mae));
            }
            if show_r {
                metric_lines.push(format!("r = {:.4}", fit.correlation_coefficient));
            }

            if compact {
                if !metric_lines.is_empty() {
                    chart.draw_series(std::iter::once(Text::new(
                        metric_lines.join(" | "),
                        (x_start, y_cursor),
                        text_style.clone(),
                    )))?;
                    y_cursor -= line_step;
                }
            } else {
                for text in metric_lines {
                    chart.draw_series(std::iter::once(Text::new(
                        text,
                        (x_start, y_cursor),
                        text_style.clone(),
                    )))?;
                    y_cursor -= line_step;
                }
            }

            // Extra gap between annotation blocks for different series.
            y_cursor -= block_gap;
        }
    }

    Ok(())
}

fn draw_plot_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    rendered_series: &[Vec<PlotSeries>],
    experimental_line_width: u32,
    data_line_width: u32,
    marker_radius: i32,
    config: &PublicationConfig,
    force_scatter: bool,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    // Series layering order: fitted lines, regression overlays, experimental
    // lines, then optional markers.
    for (idx, series_group) in rendered_series.iter().enumerate() {
        let experimental_rgba = palette_color_for_series(config, idx).to_rgba();
        let experimental_style = ShapeStyle::from(&experimental_rgba).filled();
        let experimental_line_style =
            ShapeStyle::from(&experimental_rgba).stroke_width(experimental_line_width.max(1));

        // ECM / external fitted series — use the fitted-series color palette.
        for (fit_idx, series) in series_group
            .iter()
            .filter(|series| series.kind == PlotSeriesKind::Fitted)
            .enumerate()
        {
            let points = series.points.clone();
            let label = series.label.clone();
            let fitted_palette = palette_color_for_series(config, idx + fit_idx + 1);
            let fitted_rgba = config
                .fitted_color
                .unwrap_or(PlotColor::rgba(
                    fitted_palette.red,
                    fitted_palette.green,
                    fitted_palette.blue,
                    0.70,
                ))
                .to_rgba();
            let fitted_style = ShapeStyle::from(&fitted_rgba).stroke_width(data_line_width.max(2));
            draw_line_series_linear(chart, points, label, fitted_style, config.fitted_line_style)?;
        }

        // Regression-fitted series — use the *same* color as the paired
        // experimental series so the line is visually consistent with the dots.
        for series in series_group
            .iter()
            .filter(|s| s.kind == PlotSeriesKind::RegressionFit)
        {
            let reg_rgba = palette_color_for_series(config, idx).to_rgba();
            let reg_style = ShapeStyle::from(&reg_rgba).stroke_width(data_line_width.max(2));
            draw_line_series_linear(
                chart,
                series.points.clone(),
                series.label.clone(),
                reg_style,
                config.fitted_line_style,
            )?;
        }

        for series in series_group
            .iter()
            .filter(|series| series.kind == PlotSeriesKind::Experimental)
        {
            let points = series.points.clone();
            let label = series.label.clone();

            // Draw the line unless we are in regression-scatter mode.
            // The regression path signals scatter-only by setting
            // `experimental_line_width: None` while `regression` remains
            // `Some(_)`.  In all other cases (including when
            // `experimental_line_width` is None without regression) the line
            // is always drawn using the resolved width.
            let draw_line = !force_scatter
                && (config.regression.is_none() || config.experimental_line_width.is_some());
            if draw_line {
                draw_line_series_linear(
                    chart,
                    points.clone(),
                    label.clone(),
                    experimental_line_style,
                    config.experimental_line_style,
                )?;
            }

            if force_scatter || config.show_points {
                draw_marker_series_linear(
                    chart,
                    points,
                    label,
                    experimental_style,
                    marker_radius,
                    config.experimental_marker_shape,
                )?;
            }
        }
    }

    Ok(())
}

fn dashed_segments(points: &[(f64, f64)], dash_count_hint: usize) -> Vec<Vec<(f64, f64)>> {
    // Convert one polyline into short drawable dash segments.
    if points.len() < 2 {
        return Vec::new();
    }

    let total_length = points
        .windows(2)
        .map(|pair| {
            let dx = pair[1].0 - pair[0].0;
            let dy = pair[1].1 - pair[0].1;
            (dx * dx + dy * dy).sqrt()
        })
        .sum::<f64>();

    if total_length <= f64::EPSILON {
        return vec![points.to_vec()];
    }

    let dash_count = dash_count_hint.max(6) as f64;
    let dash_length = (total_length / (dash_count * 2.0)).max(total_length / 200.0);
    let gap_length = dash_length;
    let cycle = dash_length + gap_length;
    let mut consumed = 0.0;
    let mut segments = Vec::new();

    for pair in points.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let length = (dx * dx + dy * dy).sqrt();
        if length <= f64::EPSILON {
            continue;
        }

        let mut local = 0.0;
        while local < length {
            let cycle_pos = (consumed + local) % cycle;
            if cycle_pos >= dash_length {
                let skip = (cycle - cycle_pos).min(length - local);
                local += skip;
                continue;
            }

            let dash_remaining = dash_length - cycle_pos;
            let draw_len = dash_remaining.min(length - local);
            let start_t = local / length;
            let end_t = (local + draw_len) / length;
            let seg_start = (start.0 + dx * start_t, start.1 + dy * start_t);
            let seg_end = (start.0 + dx * end_t, start.1 + dy * end_t);
            segments.push(vec![seg_start, seg_end]);
            local += draw_len;
        }

        consumed += length;
    }

    segments
}

fn draw_line_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    points: Vec<(f64, f64)>,
    label: String,
    style: ShapeStyle,
    line_style: PlotLineStyle,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    // Draw one logical line series, optionally broken into dash segments.
    match line_style {
        PlotLineStyle::Solid => {
            chart
                .draw_series(LineSeries::new(points, style))?
                .label(label)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 30, y)], style));
        }
        PlotLineStyle::Dashed => {
            let segments = dashed_segments(&points, 12);
            chart
                .draw_series(
                    segments
                        .into_iter()
                        .map(|segment| PathElement::new(segment, style)),
                )?
                .label(label)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 30, y)], style));
        }
    }

    Ok(())
}

fn draw_marker_series_linear<DB>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    points: Vec<(f64, f64)>,
    label: String,
    style: ShapeStyle,
    marker_radius: i32,
    marker_shape: PlotMarkerShape,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    // Draw point markers with shape-specific primitives and legends.
    match marker_shape {
        PlotMarkerShape::Circle => {
            chart
                .draw_series(
                    points
                        .into_iter()
                        .map(|point| Circle::new(point, marker_radius, style)),
                )?
                .label(label)
                .legend(move |(x, y)| Circle::new((x + 15, y), marker_radius, style));
        }
        PlotMarkerShape::Square => {
            chart
                .draw_series(points.into_iter().map(|point| {
                    // Root cause of the blue-fill artifact: Rectangle::new
                    // takes chart-space (data) coordinates, so using
                    // `marker_radius as f64` as an offset produced squares
                    // whose half-side was in data units rather than pixels.
                    // For data ranges much smaller than the radius value (e.g.
                    // 0–1 with radius=8) every square covered the entire plot
                    // area, flooding the output with solid colour.
                    //
                    // Fix: EmptyElement anchors at the chart coordinate and
                    // converts it to a pixel backend position; the Rectangle
                    // added via `+` then uses pixel offsets, matching the
                    // pixel-radius semantics of Circle, TriangleMarker, and
                    // Cross.
                    EmptyElement::at(point)
                        + Rectangle::new(
                            [
                                (-marker_radius, -marker_radius),
                                (marker_radius, marker_radius),
                            ],
                            style,
                        )
                }))?
                .label(label)
                .legend(move |(x, y)| {
                    Rectangle::new(
                        [(x + 8, y - marker_radius), (x + 22, y + marker_radius)],
                        style,
                    )
                });
        }
        PlotMarkerShape::Triangle => {
            chart
                .draw_series(
                    points
                        .into_iter()
                        .map(|point| TriangleMarker::new(point, marker_radius, style)),
                )?
                .label(label)
                .legend(move |(x, y)| TriangleMarker::new((x + 15, y), marker_radius, style));
        }
        PlotMarkerShape::Cross => {
            chart
                .draw_series(
                    points
                        .into_iter()
                        .map(|point| Cross::new(point, marker_radius, style.stroke_width(2))),
                )?
                .label(label)
                .legend(move |(x, y)| {
                    Cross::new((x + 15, y), marker_radius, style.stroke_width(2))
                });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AxisDisplayFormat, AxisScale, AxisTickFormatter, FillBetweenMode, PieValueLabelMode,
        PlotType, PublicationConfig, ScientificNotationStyle, format_axis_label, plot_hq,
        should_use_scientific_notation,
    };
    use crate::data_file::{PlotData, YSeries};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn global_defaults_enable_scientific_notation() {
        let config = PublicationConfig::default();
        assert!(config.sci_notation_x);
        assert!(config.sci_notation_y);
        assert_eq!(config.sci_notation_threshold_x, 10_000.0);
        assert_eq!(config.sci_notation_threshold_y, 10_000.0);
        assert_eq!(
            config.sci_notation_style_x,
            ScientificNotationStyle::Normalized
        );
        assert_eq!(
            config.sci_notation_style_y,
            ScientificNotationStyle::Normalized
        );
    }

    #[test]
    fn scientific_notation_switches_at_axis_threshold() {
        assert!(should_use_scientific_notation(
            true, 10_000.0, 0.0, 12_500.0
        ));
        assert!(should_use_scientific_notation(
            true, 10_000.0, -15_000.0, 200.0
        ));
        assert!(!should_use_scientific_notation(
            true, 10_000.0, -9_999.0, 9_999.0
        ));
        assert!(!should_use_scientific_notation(
            false, 10_000.0, 0.0, 50_000.0
        ));
    }

    #[test]
    fn axis_tick_formatter_formats_full_scientific_ticks() {
        let formatter = AxisTickFormatter::new(
            AxisScale::Linear,
            false,
            true,
            10_000.0,
            ScientificNotationStyle::Full,
            2,
            0.0,
            12_500.0,
        );

        assert_eq!(formatter.format(2500.0), "2.50×10^3");
        assert_eq!(formatter.format(10_000.0), "1×10^4");
        assert_eq!(formatter.format(0.0), "0");
    }

    #[test]
    fn normalized_scientific_notation_moves_power_to_axis_label() {
        let axis_display = AxisDisplayFormat::new(
            "Z' (Ohm)",
            AxisScale::Linear,
            false,
            true,
            10_000.0,
            ScientificNotationStyle::Normalized,
            2,
            0.0,
            250_000.0,
        );

        assert_eq!(axis_display.label, "Z' (×10^5 Ohm)");
        assert_eq!(axis_display.tick_formatter.format(120_000.0), "1.2");
        assert_eq!(axis_display.tick_formatter.format(240_000.0), "2.4");
    }

    #[test]
    fn log_axis_can_render_exponent_ticks_without_scientific_notation_conflicts() {
        let axis_display = AxisDisplayFormat::new(
            "Frequency (Hz)",
            AxisScale::Log { base: 10.0 },
            true,
            true,
            10_000.0,
            ScientificNotationStyle::Normalized,
            2,
            0.01,
            100_000.0,
        );

        assert_eq!(axis_display.label, "Frequency (Hz)");
        assert_eq!(axis_display.tick_formatter.format(-2.0), "-2");
        assert_eq!(axis_display.tick_formatter.format(0.0), "0");
        assert_eq!(axis_display.tick_formatter.format(5.0), "5");
    }

    #[test]
    fn normalized_log_axis_formats_back_transformed_ticks_when_exponent_mode_is_disabled() {
        let axis_display = AxisDisplayFormat::new(
            "Frequency (Hz)",
            AxisScale::Log { base: 10.0 },
            false,
            true,
            10_000.0,
            ScientificNotationStyle::Normalized,
            2,
            0.01,
            100_000.0,
        );

        assert_eq!(axis_display.label, "Frequency (×10^5 Hz)");
        assert_eq!(axis_display.tick_formatter.format(-2.0), "10^-7");
        assert_eq!(axis_display.tick_formatter.format(0.0), "10^-5");
        assert_eq!(axis_display.tick_formatter.format(5.0), "1");
    }

    #[test]
    fn axis_label_scaling_is_inserted_before_units() {
        assert_eq!(
            format_axis_label(
                "|Z| (Ohm)",
                super::AxisScientificNotationMode::Normalized { exponent: 6 }
            ),
            "|Z| (×10^6 Ohm)"
        );
    }

    #[test]
    fn default_axis_scales_respect_explicit_overrides() {
        let cfg = PublicationConfig::default()
            .with_default_axis_scales(Some(AxisScale::Log { base: 10.0 }), None);
        match cfg.x_scale {
            AxisScale::Log { base } => assert!((base - 10.0).abs() < 1e-10),
            AxisScale::Linear => panic!("expected domain default log scale"),
        }

        let explicit = PublicationConfig {
            x_scale: AxisScale::Linear,
            x_scale_is_explicit: true,
            ..PublicationConfig::default()
        }
        .with_default_axis_scales(Some(AxisScale::Log { base: 10.0 }), None);

        assert!(matches!(explicit.x_scale, AxisScale::Linear));
    }

    #[test]
    fn default_scientific_notation_respects_explicit_overrides() {
        let cfg = PublicationConfig::default().with_default_scientific_notation(Some(false), None);
        assert!(!cfg.sci_notation_x);

        let explicit = PublicationConfig {
            sci_notation_x: true,
            sci_notation_x_is_explicit: true,
            ..PublicationConfig::default()
        }
        .with_default_scientific_notation(Some(false), None);

        assert!(explicit.sci_notation_x);
    }

    fn unique_test_output_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rust_plots_{name}_{nanos}"));
        fs::create_dir_all(&dir).expect("temporary output directory should be creatable");
        dir
    }

    fn assert_plot_exports(name: &str, datasets: &[PlotData], config: PublicationConfig) {
        let dir = unique_test_output_dir(name);
        let output_base = dir.join(name);
        plot_hq(
            output_base.to_string_lossy().as_ref(),
            datasets,
            &config,
            true,
        )
        .expect("plot export should succeed");

        assert!(output_base.with_extension("svg").exists());
        assert!(output_base.with_extension("png").exists());

        fs::remove_dir_all(&dir).expect("temporary output directory should be removable");
    }

    fn sample_primary_dataset() -> PlotData {
        PlotData::new(vec![1.0, 2.0, 3.0, 4.0], vec![2.0, 3.5, 2.8, 4.2]).with_label("Signal")
    }

    fn sample_multi_series_dataset() -> PlotData {
        PlotData {
            date: None,
            x_values: vec![1.0, 2.0, 3.0, 4.0],
            y_values: vec![2.0, 3.0, 2.5, 4.0],
            y_series: vec![
                YSeries {
                    values: vec![1.0, 1.4, 1.7, 2.0],
                    label: Some("Control".to_string()),
                },
                YSeries {
                    values: vec![0.5, 1.0, 1.2, 1.6],
                    label: Some("Blank".to_string()),
                },
            ],
            label: Some("Primary".to_string()),
        }
    }

    #[test]
    fn line_and_scatter_plots_export_successfully() {
        let datasets = vec![sample_primary_dataset()];

        assert_plot_exports(
            "line_smoke",
            &datasets,
            PublicationConfig {
                plot_type: PlotType::Line,
                ..PublicationConfig::default()
            },
        );

        assert_plot_exports(
            "scatter_smoke",
            &datasets,
            PublicationConfig {
                plot_type: PlotType::Scatter,
                ..PublicationConfig::default()
            },
        );
    }

    #[test]
    fn bar_plot_variants_export_successfully() {
        let datasets = vec![sample_multi_series_dataset()];

        for (name, plot_type) in [
            ("vertical_bar_smoke", PlotType::VerticalBar),
            ("horizontal_bar_smoke", PlotType::HorizontalBar),
            ("grouped_bar_smoke", PlotType::GroupedBar),
            ("stacked_bar_smoke", PlotType::StackedBar),
        ] {
            assert_plot_exports(
                name,
                &datasets,
                PublicationConfig {
                    plot_type,
                    category_labels: vec![
                        "A".to_string(),
                        "B".to_string(),
                        "C".to_string(),
                        "D".to_string(),
                    ],
                    ..PublicationConfig::default()
                },
            );
        }
    }

    #[test]
    fn fill_stack_and_pie_plots_export_successfully() {
        let stacked = vec![sample_multi_series_dataset()];
        assert_plot_exports(
            "fill_between_smoke",
            &stacked,
            PublicationConfig {
                plot_type: PlotType::FillBetween,
                fill_between_mode: FillBetweenMode::BetweenCurves,
                ..PublicationConfig::default()
            },
        );
        assert_plot_exports(
            "stack_plot_smoke",
            &stacked,
            PublicationConfig {
                plot_type: PlotType::StackPlot,
                ..PublicationConfig::default()
            },
        );

        let pie = vec![sample_primary_dataset()];
        assert_plot_exports(
            "pie_smoke",
            &pie,
            PublicationConfig {
                plot_type: PlotType::Pie,
                category_labels: vec![
                    "Copper".to_string(),
                    "Lead".to_string(),
                    "Nitrate".to_string(),
                    "Other".to_string(),
                ],
                pie_value_label_mode: PieValueLabelMode::ValueAndPercentage,
                ..PublicationConfig::default()
            },
        );
    }
}
