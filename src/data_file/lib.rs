//! Data-ingestion and normalization layer.
//!
//! This module family is the canonical `Dataset`-to-domain adaptation and
//! compatibility façade consumed by plotting and search pipelines. Physical
//! CSV/TXT/DAT/XLSX parsing, CHI recognition, binary detection, and malformed
//! row recovery belong exclusively to `electrodata-io`.
//! - `chi_file`: scientific EIS/OCPT domain types and canonical provider adapters.
//! - `data_op`: generic `PlotData` container and point-selection utilities.
//! - `value_transform`: display/run-time axis transform resolution.
//!
//! `calamine` is a test-only archived-parity dependency, never a consumer
//! production workbook parser.

pub mod chi_file;
pub mod data_op;
pub(crate) mod electrodata_adapter;
pub mod electrodata_domain_adapter;
pub mod excel_file;
pub mod input_kind;
pub mod measurement_adapter;
pub mod measurement_parser;
pub mod value_transform;

pub use crate::domain::{DataParsingError, MeasurementParseResult};
pub use chi_file::{EISData, EISFitResult, ElectrochemData};
pub use data_op::{
    DataFileType, IntoPlotData, LoadedExperimentData, PlotData, PlotDataBuilder, PlotDataError,
    PointSelection, YSeries, load_data,
};
pub use electrodata_domain_adapter::{
    project_compatibility_read_options, read_dataset, read_dataset_with_sheet,
};
pub use input_kind::InputKind;
pub use measurement_adapter::{channel_to_plot_data, measurement_to_plot_data, to_plot_data};
#[allow(deprecated)]
pub use measurement_parser::{load_experiment, parse_measurement_file, parse_measurement_text};
pub use value_transform::{
    AxisTransforms, TransformKind, TransformWarning, ValueTransform, resolve_axis_transforms,
    resolve_transform,
};
