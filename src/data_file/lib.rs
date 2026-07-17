//! Data-ingestion and normalization layer.
//!
//! This module family converts domain input files into stable in-memory data
//! structures consumed by plotting and search pipelines.
//! - `chi_file`: parsers for CHI-style electrochemical exports and EIS files.
//! - `data_op`: generic `PlotData` container and point-selection utilities.
//! - `value_transform`: display/run-time axis transform resolution.

pub mod chi_file;
pub mod data_op;
pub mod value_transform;

pub use crate::domain::DataParsingError;
pub use chi_file::{EISData, EISFitResult, ElectrochemData};
pub use data_op::{
    IntoPlotData, PlotData, PlotDataBuilder, PlotDataError, PointSelection, YSeries,
};
pub use value_transform::{
    AxisTransforms, TransformKind, TransformWarning, ValueTransform, resolve_axis_transforms,
    resolve_transform,
};
