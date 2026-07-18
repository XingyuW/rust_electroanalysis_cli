//! Generalized data interface layer.
//!
//! # Purpose
//!
//! This module defines [`PlotData`] — the **canonical plottable representation**
//! that bridges raw input data and the rendering engine in `plotting.rs`.
//!
//! # Point-selection extension
//!
//! In addition to holding raw data, `PlotData` supports **point selection**:
//! extracting a subset of data points from a dataset to plot only those points
//! while leaving the source data unchanged.
//!
//! Two modes are supported (see [`PointSelection`]):
//!
//! * **Position-based** (`positions = [1, 5, 10]`) — select data points by
//!   their 1-based positions in the dataset.  Position 1 refers to the first
//!   data point.
//! * **X-value-based** (`x_values = [2.5, 3.5]`) — select data points by
//!   target x-values; the nearest available x-value is chosen for each target.
//!
//! Selection is implemented entirely in this module (the data layer) and is
//! intentionally absent from `plotting.rs`.  The selected subset is itself a
//! valid [`PlotData`] and flows through the rendering pipeline unchanged —
//! there are no special plotting branches for selected data.
//!
//! ## Configuration
//!
//! Point selection is declared in `plot_config.toml` inside a `[[generic_plot]]`
//! job block, within the `individual_style` or `combined_style` sub-table:
//!
//! ```toml
//! [[generic_plot]]
//! input_dir = "data/"
//!
//! [generic_plot.individual_style]
//! plot_positions = [1, 5, 10, 20]   # 1-based; individual plots only
//!
//! [generic_plot.combined_style]
//! plot_values = [2.5, 3.5, 4.5]    # nearest-x match; combined plot only
//! ```
//!
//! The resolved selection is carried by [`crate::plot_config::PlotJob`] and
//! applied by the runner before calling the plotting functions, keeping
//! selection logic out of the rendering layer.
//!
//! ## Multi-line datasets
//!
//! When [`PlotData`] carries additional y-series (via [`PlotData::y_series`]),
//! selection keeps all series aligned: the same x-indices are extracted from
//! every y-series so that the resulting subset is a valid multi-series
//! `PlotData`.  See [`PlotData::select_by_positions`] for details.
//!
//! ## Why `PlotData` is the abstraction boundary
//!
//! Without a shared intermediate type, every new data source would need to
//! reach directly into `plotting.rs` through its own domain type.  That
//! coupling makes the plotting engine aware of every distinct upstream type
//! and requires both sides to change whenever one does.
//!
//! `PlotData` breaks this coupling:
//!
//! * **On the data side**, each source (file parser, runtime computation,
//!   external crate) is responsible only for producing a `PlotData`.  It
//!   never needs to know anything about how plotting works internally.
//! * **On the plotting side**, `plotting.rs` only knows about `PlotDataSeries`
//!   (a trait that `PlotData` implements).  It never needs to know where the
//!   data came from.
//!
//! ## How raw data becomes `PlotData`
//!
//! Three paths exist, all converging on the same type:
//!
//! 1. **From file-parsed structs** via the [`From`] / [`IntoPlotData`] impls:
//!    ```rust,ignore
//!    let data: PlotData = ElectrochemData::parse_file("foo.csv")?.into();
//!    ```
//! 2. **From runtime-assembled vectors** via the constructor or builder:
//!    ```rust,ignore
//!    let data = PlotData::new(x_vec, y_vec).with_label("My Series");
//!    let data = PlotData::builder()
//!        .x_values(x_vec)
//!        .y_values(y_vec)
//!        .label("My Series")
//!        .build();
//!    ```
//! 3. **For future types**, implement [`IntoPlotData`] once and no plotting
//!    code needs to change:
//!    ```rust,ignore
//!    impl IntoPlotData for MyNewType {
//!        fn into_plot_data(self) -> PlotData {
//!            PlotData::new(self.xs, self.ys).with_label(self.name)
//!        }
//!    }
//!    ```
//!
//! ## How configuration flows into plotting
//!
//! `PlotData` carries only data — never configuration.  Configuration follows
//! a separate pipeline (see [`crate::plot_config`]):
//!
//! ```text
//! CLI args ──────────────────────────────────────────────────► (highest priority)
//!                   ↓
//! plot_config.toml (RawPlotStyle) ──────────────────────────►
//!                   ↓
//! [render] global defaults ─────────────────────────────────►
//!                   ↓
//! domain defaults (eis_plot / chi_plot / generic_plot) ─────►
//!                   ↓
//! PublicationConfig sentinel defaults ──────────────────────► (lowest priority)
//!                   ↓
//!             ResolvedPlotConfig  ──►  plotting functions
//! ```
//!
//! `PlotJobStyle::apply_to_individual` / `apply_to_combined` perform the
//! actual merge.  The result is a [`crate::plottings::ResolvedPlotConfig`]
//! (a type alias for [`crate::plottings::PublicationConfig`]) — a fully
//! materialized configuration with no `Option` fields — that is passed
//! directly into `plot_hq()`.

use crate::data_file::{EISData, ElectrochemData, parse_measurement_file};
use crate::domain::{
    AnalysisProvenance, DataParsingError, ElectrochemicalExperiment, MeasurementChannel,
    MultiChannelMeasurement, ParseDiagnostics, PlottingError, SensorMetadata,
};
use crate::plottings::{PlotDataSeries, PlotSeries};
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────
// YSeries — a single additional named y-series
// ─────────────────────────────────────────────────────────────────────────────

/// A single additional y-series that shares the x-axis of a [`PlotData`].
///
/// `YSeries` holds the y-values and an optional legend label for one extra
/// line or scatter series beyond the primary `PlotData::y_values` series.
/// It is stored in [`PlotData::y_series`] and returned as additional
/// [`PlotSeries`] by [`PlotDataSeries::plot_series`].
///
/// # Length invariant
///
/// `values` must have the same length as the parent `PlotData::x_values`.
/// If the lengths differ, the shortest is used when producing point pairs
/// (the same convention used by the default [`PlotDataSeries`] implementation).
///
/// # Construction
///
/// ```rust,ignore
/// let extra = YSeries {
///     values: vec![1.0, 2.0, 3.0],
///     label: Some("Control run".to_string()),
/// };
/// data.y_series.push(extra);
/// ```
///
/// Or via the builder:
///
/// ```rust,ignore
/// let data = PlotData::builder()
///     .x_values(xs)
///     .y_values(ys_primary)
///     .add_y_series(ys_control, Some("Control run".to_string()))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct YSeries {
    /// Y-axis values for this series; must be the same length as the parent
    /// `PlotData::x_values`.
    pub values: Vec<f64>,
    /// Optional legend label for this series.  When `None` a generated label
    /// `"Series N"` (where N ≥ 2) is used when producing [`PlotSeries`].
    pub label: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// PlotData — the canonical plottable unit
// ─────────────────────────────────────────────────────────────────────────────

/// The canonical, domain-agnostic plottable unit.
///
/// `PlotData` is the **only** data type that the generic plotting pathway
/// (`generic_plot.rs`, `plot_hq`) needs to know about.  All domain-specific
/// types (electrochemical timeseries, impedance spectra, sensor readings,
/// simulation outputs, …) are converted into `PlotData` before being handed
/// to the rendering layer.
///
/// # Field semantics
///
/// | Field | Role |
/// |-------|------|
/// | `x_values` | Horizontal axis values, one entry per data point |
/// | `y_values` | Vertical axis values (primary series), same length as `x_values` |
/// | `y_series` | Additional named y-series sharing the same x-axis (empty = single-series) |
/// | `label` | Optional series name for `y_values` shown in the legend |
/// | `date` | Optional provenance metadata; not displayed in plots |
///
/// # Single-series vs. multi-series
///
/// When `y_series` is empty (the default), `PlotData` behaves exactly as a
/// simple x/y pair — the primary `y_values` series is rendered as a single
/// line or scatter series.  This is the common case.
///
/// To represent multiple lines sharing a common x-axis, push additional
/// [`YSeries`] values into `y_series`.  All y-series are rendered together
/// via the [`PlotDataSeries::plot_series`] override, which returns one
/// [`PlotSeries`] per y-series (primary first, then each `y_series` element
/// in order).
///
/// # Point selection
///
/// Call [`PlotData::select_by_positions`], [`PlotData::select_by_x_values`],
/// or [`PlotData::select_points`] to extract a subset of data points.  The
/// result is a new `PlotData` with:
/// * the same metadata (`label`, `date`) as the source
/// * only the selected `(x, y)` pairs
/// * all `y_series` aligned with the same selected x-indices
///
/// The selected subset is a valid `PlotData` and passes through the existing
/// plotting pipeline without any special handling.
///
/// # Construction
///
/// ```rust,ignore
/// // Minimal runtime construction
/// let d = PlotData::new(x_vec, y_vec);
///
/// // With optional metadata (builder-style chaining)
/// let d = PlotData::new(x_vec, y_vec)
///     .with_label("Batch A")
///     .with_date("2026-03-26");
///
/// // Via the explicit builder (supports multi-series)
/// let d = PlotData::builder()
///     .x_values(xs)
///     .y_values(ys)
///     .label("Primary")
///     .add_y_series(ys_extra, Some("Control"))
///     .build();
///
/// // Converted from a file-parsed domain struct
/// let electrochem = ElectrochemData::parse_file("sensor.csv")?;
/// let d: PlotData = electrochem.into();
/// ```
#[derive(Debug, Clone)]
pub struct PlotData {
    /// Optional provenance metadata — not rendered in the plot.
    pub date: Option<String>,
    /// Paired horizontal-axis values; must have the same length as `y_values`
    /// and every element of `y_series`.
    pub x_values: Vec<f64>,
    /// Paired vertical-axis values (primary series); must have the same
    /// length as `x_values`.
    pub y_values: Vec<f64>,
    /// Additional y-series sharing the same x-axis.  Each element is a
    /// [`YSeries`] with its own y-values and optional label.  When empty
    /// (the default) `PlotData` behaves as a standard single-series dataset.
    ///
    /// All y-series (primary and additional) must have the same length as
    /// `x_values`.  Selection operations keep all series aligned.
    pub y_series: Vec<YSeries>,
    /// Series label for the primary `y_values` series, displayed in the
    /// legend.  Falls back to `"Data"` when `None`.  Use
    /// [`PlotData::with_label`] or [`PlotDataBuilder::label`] to supply a
    /// custom name.
    pub label: Option<String>,
}

impl PlotData {
    /// Construct a `PlotData` from raw x and y vectors.
    ///
    /// This is the primary constructor for **runtime-assembled data** — data
    /// that is not read from a file but computed or received at execution time.
    /// `label`, `date`, and `y_series` default to empty / `None` and can be
    /// set with the `with_*` methods or the builder.
    ///
    /// ```rust,ignore
    /// let d = PlotData::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 4.0]);
    /// ```
    pub fn new(x_values: Vec<f64>, y_values: Vec<f64>) -> Self {
        Self {
            date: None,
            x_values,
            y_values,
            y_series: Vec::new(),
            label: None,
        }
    }

    /// Attach a series label and return the modified `PlotData`.
    ///
    /// The label is displayed in the legend.  Calling this method again
    /// replaces the previous label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Attach a date string (provenance metadata) and return the modified
    /// `PlotData`.  The date is not rendered in the plot.
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    /// Return a fresh [`PlotDataBuilder`] for step-by-step construction.
    ///
    /// Prefer the builder when constructing `PlotData` from many optional
    /// fields gathered across multiple steps.
    ///
    /// ```rust,ignore
    /// let d = PlotData::builder()
    ///     .x_values(xs)
    ///     .y_values(ys)
    ///     .label("Run 3")
    ///     .add_y_series(ys_extra, Some("Control"))
    ///     .build();
    /// ```
    pub fn builder() -> PlotDataBuilder {
        PlotDataBuilder::default()
    }

    // ── Selection methods ───────────────────────────────────────────────────

    /// Extract a subset of data points by their **1-based positions**.
    ///
    /// # Index convention
    ///
    /// Positions are **1-based**: position `1` refers to the first data
    /// point, position `2` to the second, and so on.  This is deliberately
    /// user-friendly so that a TOML entry such as
    /// `plot_positions = [1, 5, 10]` selects the 1st, 5th, and 10th points
    /// without any mental offset.  Internally the values are converted to
    /// 0-based indices (`position - 1`) before array access.
    ///
    /// # Duplicate positions
    ///
    /// Repeated positions are allowed and preserved.  If `positions =
    /// [3, 3, 5]`, the third data point appears twice in the result.
    ///
    /// # Order preservation
    ///
    /// The result follows the order of the requested `positions` slice, not
    /// the original dataset order.  Supplying `[10, 1]` returns the tenth
    /// point first.
    ///
    /// # Multi-series alignment
    ///
    /// When `self.y_series` is non-empty, every additional y-series is
    /// filtered using the same indices so that all series remain aligned with
    /// the returned `x_values`.
    ///
    /// # Errors
    ///
    /// * [`PlotDataError::EmptySelection`] — `positions` is empty.
    /// * [`PlotDataError::PositionOutOfBounds`] — any position is `0` or
    ///   greater than `self.x_values.len()`.
    pub fn select_by_positions(&self, positions: &[usize]) -> Result<PlotData, PlotDataError> {
        if positions.is_empty() {
            return Err(PlotDataError::EmptySelection);
        }
        let len = self.x_values.len();
        // Validate and convert to 0-based indices in one pass.
        let indices: Vec<usize> = positions
            .iter()
            .map(|&pos| {
                if pos == 0 || pos > len {
                    Err(PlotDataError::PositionOutOfBounds { position: pos, len })
                } else {
                    Ok(pos - 1) // 1-based → 0-based
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(self.extract_indices(&indices))
    }

    /// Extract a subset of data points by **target x-values**.
    ///
    /// For each value in `x_targets` the method searches `self.x_values` for
    /// the entry whose absolute distance to the target is smallest.  The
    /// index of that entry is used to select the corresponding `(x, y)` pair.
    ///
    /// # Tie-breaking
    ///
    /// When two x-values are equally close to a target the one with the
    /// **lower index** (appearing earlier in the dataset) is chosen.  This
    /// preserves the original dataset order as a stable tie-break.
    ///
    /// # Duplicate targets
    ///
    /// If two entries in `x_targets` resolve to the same x-index the
    /// corresponding data point appears twice in the result.  Duplicates are
    /// allowed and preserved intentionally.
    ///
    /// # Order preservation
    ///
    /// The result follows the order of `x_targets`, not the original dataset
    /// order.  Requesting `[5.0, 1.0]` returns the point nearest to `5.0`
    /// first.
    ///
    /// # Multi-series alignment
    ///
    /// When `self.y_series` is non-empty, every additional y-series is
    /// filtered using the same indices so that all series remain aligned with
    /// the returned `x_values`.
    ///
    /// # Errors
    ///
    /// * [`PlotDataError::EmptySelection`] — `x_targets` is empty.
    /// * [`PlotDataError::EmptyDataset`] — `self.x_values` is empty (there
    ///   are no candidates to match against).
    pub fn select_by_x_values(&self, x_targets: &[f64]) -> Result<PlotData, PlotDataError> {
        if x_targets.is_empty() {
            return Err(PlotDataError::EmptySelection);
        }
        if self.x_values.is_empty() {
            return Err(PlotDataError::EmptyDataset);
        }

        // For each target, pick the index of the nearest x-value.
        // Ties are broken by lower index (first occurrence wins) because
        // `min_by` returns the first minimum when comparisons are equal.
        let indices: Vec<usize> = x_targets
            .iter()
            .map(|&target| {
                self.x_values
                    .iter()
                    .enumerate()
                    .map(|(i, &x)| (i, (x - target).abs()))
                    .min_by(|(_, da), (_, db)| {
                        da.partial_cmp(db).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(i, _)| i)
                    // Safety: x_values is non-empty (checked above), so
                    // min_by always returns Some.
                    .expect("x_values is non-empty")
            })
            .collect();

        Ok(self.extract_indices(&indices))
    }

    /// Select a subset of data points using a [`PointSelection`] value.
    ///
    /// Convenience wrapper that dispatches to
    /// [`select_by_positions`](Self::select_by_positions) or
    /// [`select_by_x_values`](Self::select_by_x_values) depending on the
    /// variant.  Use this when the selection mode is determined at runtime
    /// (e.g. loaded from config).
    ///
    /// ```rust,ignore
    /// let selected = data.select_points(&PointSelection::Positions(vec![1, 5, 10]))?;
    /// let selected = data.select_points(&PointSelection::XValues(vec![2.5, 3.5]))?;
    /// ```
    pub fn select_points(&self, selection: &PointSelection) -> Result<PlotData, PlotDataError> {
        match selection {
            PointSelection::Positions(positions) => self.select_by_positions(positions),
            PointSelection::XValues(targets) => self.select_by_x_values(targets),
        }
    }

    // ── Private helpers ─────────────────────────────────────────────────────

    /// Produce a new `PlotData` by extracting the elements at each index in
    /// `indices` from every series.  All series (primary and additional) are
    /// filtered with the same index set, preserving alignment.
    ///
    /// Metadata (`label`, `date`) is copied from `self` unchanged.
    fn extract_indices(&self, indices: &[usize]) -> PlotData {
        // Apply the same index projection to every series to keep x/y
        // alignment invariant intact after selection.
        let x_values = indices.iter().map(|&i| self.x_values[i]).collect();
        let y_values = indices.iter().map(|&i| self.y_values[i]).collect();
        let y_series = self
            .y_series
            .iter()
            .map(|ys| YSeries {
                values: indices.iter().map(|&i| ys.values[i]).collect(),
                label: ys.label.clone(),
            })
            .collect();
        PlotData {
            date: self.date.clone(),
            x_values,
            y_values,
            y_series,
            label: self.label.clone(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PlotDataSeries implementation — makes PlotData usable by plot_hq()
// ─────────────────────────────────────────────────────────────────────────────

/// `PlotData` implements [`PlotDataSeries`] so it can be passed directly to
/// [`crate::plottings::plot_hq`] (and any other function that is generic over
/// `PlotDataSeries`) without any intermediate adapters.
///
/// This is the mechanism by which the generalized pathway works end-to-end:
/// once data is in `PlotData` form, the plotting engine consumes it through
/// the same trait interface it already uses for `ElectrochemData` and
/// `EISData`.  No changes to the plotting engine are ever required.
///
/// ## Multi-series support
///
/// The `plot_series()` method is overridden to return **all** y-series held
/// by this `PlotData` — the primary `y_values` series (series 1) followed by
/// each element of `y_series` in order.  This means a single `PlotData`
/// value with N y-series produces N lines / scatter series when passed to
/// `plot_hq`.
///
/// When `y_series` is empty the default single-series behaviour is preserved
/// (one `PlotSeries` is returned), ensuring backward compatibility.
impl PlotDataSeries for PlotData {
    fn label(&self) -> &str {
        // Falls back to the generic sentinel "Data" when no label was supplied.
        self.label.as_deref().unwrap_or("Data")
    }

    fn x_values(&self) -> &[f64] {
        &self.x_values
    }

    fn y_values(&self) -> &[f64] {
        &self.y_values
    }

    /// Return all y-series (primary + additional) as a flat `Vec<PlotSeries>`.
    ///
    /// * Series 1 — the primary `y_values` with `label` as its name.
    /// * Series 2+ — each `YSeries` in `y_series`, using its own `label` or
    ///   the generated fallback `"Series N"` where N starts at 2.
    fn plot_series(&self) -> Result<Vec<PlotSeries>, PlottingError> {
        // Primary series.
        let primary_pts: Vec<(f64, f64)> = self
            .x_values
            .iter()
            .zip(self.y_values.iter())
            .map(|(x, y)| (*x, *y))
            .collect();
        let mut all_series = vec![PlotSeries::experimental(
            self.label().to_string(),
            primary_pts,
        )];

        // Additional y-series.
        for (idx, ys) in self.y_series.iter().enumerate() {
            let fallback = format!("Series {}", idx + 2);
            let name = ys.label.as_deref().unwrap_or(&fallback).to_string();
            let pts: Vec<(f64, f64)> = self
                .x_values
                .iter()
                .zip(ys.values.iter())
                .map(|(x, y)| (*x, *y))
                .collect();
            all_series.push(PlotSeries::experimental(name, pts));
        }

        Ok(all_series)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PlotDataBuilder — step-by-step constructor
// ─────────────────────────────────────────────────────────────────────────────

/// An explicit builder for [`PlotData`].
///
/// Created via [`PlotData::builder()`].  All setters consume and return
/// `self` so they can be chained.  Calling [`build`](Self::build) finalises
/// the value; unset optional fields remain `None` / empty.
///
/// ```rust,ignore
/// let d = PlotData::builder()
///     .x_values(vec![0.0, 1.0, 2.0])
///     .y_values(vec![0.0, 0.5, 2.0])
///     .label("Simulation")
///     .date("2026-03-26")
///     .add_y_series(vec![0.1, 0.6, 2.1], Some("Control".to_string()))
///     .build();
/// ```
#[derive(Default)]
pub struct PlotDataBuilder {
    x_values: Vec<f64>,
    y_values: Vec<f64>,
    y_series: Vec<YSeries>,
    label: Option<String>,
    date: Option<String>,
}

impl PlotDataBuilder {
    /// Set the horizontal-axis values.
    pub fn x_values(mut self, values: Vec<f64>) -> Self {
        self.x_values = values;
        self
    }

    /// Set the primary vertical-axis values (first / only series).
    pub fn y_values(mut self, values: Vec<f64>) -> Self {
        self.y_values = values;
        self
    }

    /// Append an additional y-series with an optional label.
    ///
    /// This can be called multiple times to add as many series as needed.
    /// The series are added in call order.  Each `values` vector must be the
    /// same length as `x_values`.
    pub fn add_y_series(mut self, values: Vec<f64>, label: Option<impl Into<String>>) -> Self {
        self.y_series.push(YSeries {
            values,
            label: label.map(Into::into),
        });
        self
    }

    /// Set the series label for the primary y-series (displayed in the legend).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the date provenance string (not rendered in the plot).
    pub fn date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    /// Consume the builder and return a [`PlotData`].
    pub fn build(self) -> PlotData {
        // Builder performs structural assembly only; semantic length checks are
        // deferred to consumers/plotting pathways to preserve existing behavior.
        PlotData {
            date: self.date,
            x_values: self.x_values,
            y_values: self.y_values,
            y_series: self.y_series,
            label: self.label,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PointSelection — configuration-driven selection mode
// ─────────────────────────────────────────────────────────────────────────────

/// Specifies which data points to extract from a [`PlotData`].
///
/// `PointSelection` is the **resolved** configuration type that drives point
/// selection.  It is produced by the config pipeline from TOML fields
/// (`plot_positions` / `plot_values` inside a `[[generic_plot]]` job) and
/// stored on [`crate::plot_config::PlotJob`].  The runner passes it to
/// [`PlotData::select_points`] before handing the resulting subset to the
/// plotting functions.
///
/// # Index convention for `Positions`
///
/// Positions are **1-based** — position `1` is the first data point.  This
/// is the user-visible convention; `select_by_positions` converts internally
/// to 0-based before performing array access.
///
/// # Configuration examples
///
/// ```toml
/// # Inside a [[generic_plot]] block:
///
/// [generic_plot.individual_style]
/// plot_positions = [1, 5, 10, 20]   # 1-based positions
///
/// [generic_plot.combined_style]
/// plot_values = [2.5, 3.5, 4.5]    # nearest-x matching
/// ```
///
/// Only one mode can be active per scope (individual or combined).
/// Setting both `plot_positions` and `plot_values` in the same style block
/// is a configuration error caught at job resolution time.
#[derive(Debug, Clone)]
pub enum PointSelection {
    /// Select points by their **1-based** positions in the dataset.
    ///
    /// `positions = [1, 5, 10]` selects the 1st, 5th, and 10th data points.
    /// Duplicates and non-monotone ordering are both permitted.
    Positions(Vec<usize>),
    /// Select points by target x-values.
    ///
    /// For each target the nearest available x-value is chosen.  Ties are
    /// broken in favour of the earlier-indexed x-value.
    XValues(Vec<f64>),
}

// ─────────────────────────────────────────────────────────────────────────────
// PlotDataError — error type for selection operations
// ─────────────────────────────────────────────────────────────────────────────

/// Errors produced by [`PlotData`] selection methods.
///
/// All variants are self-explanatory and carry enough context to produce a
/// user-friendly diagnostic.  The variants are matched at the call site
/// (typically in `plot_runner.rs`) and converted to `Box<dyn Error>` for
/// propagation through the job pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlotDataError {
    /// The caller provided an empty selection (no positions or x-values).
    EmptySelection,
    /// A 1-based position was `0` or exceeded the number of data points.
    PositionOutOfBounds {
        /// The invalid position value supplied by the caller (1-based).
        position: usize,
        /// The number of data points in the dataset.
        len: usize,
    },
    /// `select_by_x_values` was called on a dataset with no x-values.
    EmptyDataset,
}

impl fmt::Display for PlotDataError {
    /// Render selection failures as user-facing diagnostics suitable for logs
    /// and UI messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlotDataError::EmptySelection => {
                write!(
                    f,
                    "point selection contains no entries (empty positions or values list)"
                )
            }
            PlotDataError::PositionOutOfBounds { position, len } => write!(
                f,
                "position {position} is out of bounds for a dataset with {len} points \
                 (valid 1-based positions are 1 to {len})"
            ),
            PlotDataError::EmptyDataset => write!(
                f,
                "cannot select by x-value: the dataset contains no x-values"
            ),
        }
    }
}

impl std::error::Error for PlotDataError {}

// ─────────────────────────────────────────────────────────────────────────────
// IntoPlotData trait — the extension point for future data types
// ─────────────────────────────────────────────────────────────────────────────

/// Conversion trait: implemented by any type that can be expressed as a
/// `(x_values, y_values, optional metadata)` pair suitable for plotting.
///
/// # Why this trait exists
///
/// Rather than implementing [`PlotDataSeries`] directly for every new type
/// (which would tie each type to plotting details), new data sources implement
/// `IntoPlotData` to produce a `PlotData`.  The plotting pathway then operates
/// uniformly on `PlotData` without ever needing to know the origin type.
///
/// # Adding a new data type
///
/// 1. Define your struct.
/// 2. Implement `IntoPlotData`:
///    ```rust,ignore
///    impl IntoPlotData for MyMeasurement {
///        fn into_plot_data(self) -> PlotData {
///            PlotData::new(self.time_axis, self.signal)
///                .with_label(self.run_id)
///                .with_date(self.recorded_at)
///        }
///    }
///    ```
/// 3. Convert and plot:
///    ```rust,ignore
///    let datasets: Vec<PlotData> = measurements
///        .into_iter()
///        .map(IntoPlotData::into_plot_data)
///        .collect();
///    // Pass to plot_generic_directory_with_configs or use plot_hq directly.
///    ```
///
/// No changes to `plotting.rs` or any existing module are required.
pub trait IntoPlotData {
    fn into_plot_data(self) -> PlotData;
}

// PlotData trivially converts into itself.
impl IntoPlotData for PlotData {
    /// Identity conversion, enabling generic code paths to accept `PlotData`
    /// and domain types uniformly.
    fn into_plot_data(self) -> PlotData {
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversions from existing domain types
// ─────────────────────────────────────────────────────────────────────────────

/// Convert an [`ElectrochemData`] (CHI-instrument timeseries parsed from a
/// `.csv`/`.txt`/`.dat` file) into a generic [`PlotData`].
///
/// The `label` and `date` fields are preserved; the domain-specific
/// `test_type` and `instrument_model` fields are discarded because they have
/// no representation in the generic interface.  This conversion is lossless
/// with respect to plottable content.
///
/// This impl also satisfies [`IntoPlotData`] for `ElectrochemData` below,
/// so callers can use either `.into()` or `.into_plot_data()` depending on
/// which reads more clearly at the call site.
impl From<ElectrochemData> for PlotData {
    /// Domain-to-generic projection used by generic plotting workflows.
    fn from(src: ElectrochemData) -> Self {
        PlotData {
            date: Some(src.date),
            x_values: src.x_values,
            y_values: src.y_values,
            y_series: Vec::new(),
            label: Some(src.label),
        }
    }
}

/// [`IntoPlotData`] for [`ElectrochemData`] delegates to the [`From`] impl so
/// both conversion idioms (`into_plot_data()` and `.into()`) work uniformly.
impl IntoPlotData for ElectrochemData {
    /// Delegate to `From<ElectrochemData> for PlotData`.
    fn into_plot_data(self) -> PlotData {
        PlotData::from(self)
    }
}

/// Supported input categories for automatic loader dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFileType {
    ChiEis,
    ChiOcpt,
    SensorCsv,
    /// Excel workbook that was normalised to the same representation as CSV.
    SensorExcel,
}

/// Unified loader result for all supported input formats.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedExperimentData {
    pub file_type: DataFileType,
    pub experiment: ElectrochemicalExperiment,
    pub diagnostics: ParseDiagnostics,
}

/// Unified, content-detected data loading entrypoint.
///
/// This function enforces the input‑support policy:
/// - Binary files are rejected before any parsing attempt.
/// - Excel workbooks are routed through the spreadsheet parser and then
///   normalised to the same representation as CSV data.
pub fn load_data(path: impl AsRef<Path>) -> Result<LoadedExperimentData, DataParsingError> {
    let path = path.as_ref();

    // --- binary guard --------------------------------------------------
    let kind = crate::data_file::InputKind::classify_by_extension(path);
    if kind.is_unsupported_binary() {
        return Err(DataParsingError::invalid_at(
            path,
            format!(
                "Unsupported input file '{}': binary input is not supported. \
                 Export the dataset as CSV, XLSX, or another documented text-based format.",
                path.display()
            ),
        ));
    }

    // --- Excel dispatch ------------------------------------------------
    if kind == crate::data_file::InputKind::ExcelXlsx
        || kind == crate::data_file::InputKind::ExcelXls
    {
        return load_excel(path, None);
    }

    // --- text dispatch -------------------------------------------------
    let text = std::fs::read_to_string(path).map_err(|error| DataParsingError::io(path, error))?;
    let file_type = detect_file_type(path, &text)?;
    match file_type {
        DataFileType::ChiEis => load_chi_eis(path),
        DataFileType::ChiOcpt => load_time_series(path, DataFileType::ChiOcpt),
        DataFileType::SensorCsv => load_time_series(path, DataFileType::SensorCsv),
        DataFileType::SensorExcel => {
            unreachable!("Excel should be dispatched before text reading")
        }
    }
}

fn load_time_series(
    path: &Path,
    file_type: DataFileType,
) -> Result<LoadedExperimentData, DataParsingError> {
    let parsed = parse_measurement_file(path)?;
    let provenance = AnalysisProvenance::from_paths(path, None)?;
    let experiment_id = file_stem_or_default(path);
    let sensor_metadata = default_sensor_metadata(path, file_type, None);
    let sample_matrix = match file_type {
        DataFileType::ChiOcpt => "chi_export".to_string(),
        DataFileType::SensorCsv => "sensor_csv".to_string(),
        DataFileType::ChiEis => "chi_eis".to_string(),
        DataFileType::SensorExcel => "excel_workbook".to_string(),
    };
    let experiment = ElectrochemicalExperiment::new(
        experiment_id,
        sensor_metadata,
        None,
        parsed.measurement,
        Vec::new(),
        Vec::new(),
        sample_matrix,
        provenance,
    )?;
    Ok(LoadedExperimentData {
        file_type,
        experiment,
        diagnostics: parsed.diagnostics,
    })
}

fn load_excel(
    path: &Path,
    sheet_name: Option<&str>,
) -> Result<LoadedExperimentData, DataParsingError> {
    let (parsed, sheet) = crate::data_file::excel_file::parse_excel_measurement(path, sheet_name)?;
    let provenance = AnalysisProvenance::from_paths(path, None)?;
    let experiment_id = file_stem_or_default(path);
    let sensor_metadata = default_sensor_metadata(path, DataFileType::SensorExcel, None);
    let experiment = ElectrochemicalExperiment::new(
        experiment_id,
        sensor_metadata,
        None,
        parsed.measurement,
        Vec::new(),
        Vec::new(),
        format!("excel_workbook:{sheet}"),
        provenance,
    )?;
    Ok(LoadedExperimentData {
        file_type: DataFileType::SensorExcel,
        experiment,
        diagnostics: parsed.diagnostics,
    })
}

fn load_chi_eis(path: &Path) -> Result<LoadedExperimentData, DataParsingError> {
    let parsed = EISData::parse_file(path)?;
    let measurement = MultiChannelMeasurement::new(
        parsed.freq.clone(),
        vec![
            MeasurementChannel::from_values("Z'", "ohm", parsed.z_re.clone()),
            MeasurementChannel::from_values("Z\"", "ohm", parsed.z_im.clone()),
            MeasurementChannel::from_values("phase", "deg", parsed.phase.clone()),
        ],
    )?;
    let provenance = AnalysisProvenance::from_paths(path, None)?;
    let experiment = ElectrochemicalExperiment::new(
        file_stem_or_default(path),
        default_sensor_metadata(path, DataFileType::ChiEis, Some(&parsed.instrument_model)),
        None,
        measurement,
        Vec::new(),
        Vec::new(),
        "chi_eis".to_string(),
        provenance,
    )?;

    Ok(LoadedExperimentData {
        file_type: DataFileType::ChiEis,
        experiment,
        diagnostics: ParseDiagnostics {
            total_rows: parsed.freq.len(),
            successfully_parsed_rows: parsed.freq.len(),
            ..ParseDiagnostics::default()
        },
    })
}

fn detect_file_type(path: &Path, text: &str) -> Result<DataFileType, DataParsingError> {
    let lines = text.lines().map(str::trim).collect::<Vec<_>>();
    let mut time_header_index = None;
    for (index, line) in lines.iter().enumerate() {
        let fields = split_csv_fields(line);
        if fields.len() < 2 {
            continue;
        }
        let normalized_headers = fields
            .iter()
            .map(|header| normalize_header(header))
            .collect::<Vec<_>>();
        let has_freq = normalized_headers.iter().any(|header| header == "freq/hz");
        let has_impedance = normalized_headers
            .iter()
            .any(|header| header == "z'/ohm" || header == "z\"/ohm");
        if has_freq && has_impedance {
            return Ok(DataFileType::ChiEis);
        }
        if normalized_headers
            .iter()
            .any(|header| is_time_header(header.as_str()))
        {
            time_header_index.get_or_insert(index);
        }
    }

    let Some(header_index) = time_header_index else {
        return Err(DataParsingError::invalid_at(
            path,
            "unsupported file structure: expected a time or frequency header",
        ));
    };

    let preamble = lines
        .iter()
        .take(header_index)
        .map(|line| normalize_header(line))
        .collect::<Vec<_>>();
    let is_chi = preamble.iter().any(|line| {
        line.starts_with("instrumentmodel:")
            || line.starts_with("datasource:")
            || line.starts_with("file:")
            || line.contains("opencircuitpotential-time")
            || line.contains("open-circuitpotential-time")
    });
    Ok(if is_chi {
        DataFileType::ChiOcpt
    } else {
        DataFileType::SensorCsv
    })
}

fn split_csv_fields(line: &str) -> Vec<&str> {
    line.split(',').map(str::trim).collect()
}

fn normalize_header(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect()
}

fn is_time_header(value: &str) -> bool {
    value == "time"
        || value.starts_with("time/")
        || value.starts_with("time(")
        || value.starts_with("time[")
        || value == "timestamp"
        || value.starts_with("timestamp/")
}

fn file_stem_or_default(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "experiment".to_string())
}

fn default_sensor_metadata(
    path: &Path,
    file_type: DataFileType,
    instrument_model: Option<&str>,
) -> SensorMetadata {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "source_file_type".to_string(),
        match file_type {
            DataFileType::ChiEis => "chi_eis".to_string(),
            DataFileType::ChiOcpt => "chi_ocpt".to_string(),
            DataFileType::SensorCsv => "sensor_csv".to_string(),
            DataFileType::SensorExcel => "excel_workbook".to_string(),
        },
    );
    metadata.insert(
        "source_file".to_string(),
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
    );

    SensorMetadata {
        sensor_id: Some(file_stem_or_default(path)),
        name: Some(file_stem_or_default(path)),
        sensor_type: Some(
            match file_type {
                DataFileType::ChiEis => "chi_eis",
                DataFileType::ChiOcpt => "chi_ocpt",
                DataFileType::SensorCsv => "generic_sensor_csv",
                DataFileType::SensorExcel => "generic_sensor_excel",
            }
            .to_string(),
        ),
        analyte: None,
        manufacturer: instrument_model
            .and_then(|model| (!model.trim().is_empty()).then_some("CH Instruments".to_string())),
        model: instrument_model
            .and_then(|model| (!model.trim().is_empty()).then_some(model.to_string())),
        metadata: Some(metadata),
    }
}

#[cfg(test)]
mod loader_tests {
    use super::{DataFileType, load_data};
    use crate::data_file::measurement_adapter::measurement_to_plot_data;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_fixture(prefix: &str, content: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}_{timestamp}.csv"));
        fs::write(&path, content).expect("write fixture");
        path
    }

    #[test]
    fn detects_sensor_csv_with_bom_time_header() {
        let path = write_fixture(
            "sensor_bom",
            "\u{feff}Time(d),Concentration\n0,1.0\n0.1,1.1\n",
        );
        let loaded = load_data(&path).expect("load sensor csv");
        fs::remove_file(&path).ok();

        assert_eq!(loaded.file_type, DataFileType::SensorCsv);
        assert_eq!(loaded.experiment.measurement_data.channels.len(), 1);
        assert_eq!(
            measurement_to_plot_data(&loaded.experiment.measurement_data).len(),
            1
        );
    }

    #[test]
    fn detects_chi_eis_from_frequency_impedance_header() {
        let path = write_fixture(
            "chi_eis",
            "Mar. 17, 2026\nA.C. Impedance\nInstrument Model: CHI760F\nFreq/Hz, Z'/ohm, Z\"/ohm, Phase/deg\n1,10,-1,-5\n0.1,20,-2,-6\n",
        );
        let loaded = load_data(&path).expect("load chis eis");
        fs::remove_file(&path).ok();

        assert_eq!(loaded.file_type, DataFileType::ChiEis);
        assert_eq!(loaded.experiment.measurement_data.channels.len(), 3);
    }
}
