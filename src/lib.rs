//! Crate entry point for the CLI-focused electroanalysis backend.
//!
//! Scientific logic (parsing, fitting, plotting, and search) is exposed as
//! reusable modules consumed by the `rust_plots` CLI binary.

extern crate self as rust_plots;

#[path = "data_file/lib.rs"]
pub mod data_file;
pub use data_file::{chi_file, data_op};

#[path = "plottings/lib.rs"]
pub mod plottings;
pub use plottings::plotting;

#[path = "impedance/lib.rs"]
pub mod impedance;
pub mod mechanism;

pub mod calibration_config;
pub mod cli;
pub mod domain;
pub mod fitting;
pub mod health;
pub mod health_config;
pub mod mechanism_config;
pub mod plot_config;
pub mod plot_runner;
pub mod potentiometry;
pub mod regression_mod;
pub mod results;
pub mod runners;
pub mod search_config;
pub mod search_runner;
pub mod signal;
pub mod signal_config;
pub mod transient_config;
pub mod workspace;

pub use domain::{
    AnalysisProvenance, ChannelMetadata, ConfigurationError, DataParsingError,
    ElectrochemicalExperiment, EnvironmentalSeries, ExperimentEvent, ExperimentEventKind,
    ExperimentMetadataDocument, FittingError, MeasurementChannel, MeasurementParseResult,
    MultiChannelMeasurement, ParseDiagnostics, PlottingError, ProvenanceError, ReferenceMetadata,
    ReportingError, SensorMetadata, WorkspaceError, load_experiment_metadata,
};
pub use potentiometry::units::{Quantity, QuantityUnit};
pub use results::CircuitFitResult;
pub use results::calibration::{
    CalibrationAnalysisReport, CalibrationBranch, CalibrationModelKind, CalibrationObservation,
    CalibrationObservationSet, CalibrationPotentialSource, CalibrationPrediction,
    StoredCalibrationModel,
};
pub use results::health::{SensorHealthAssessment, SensorHealthBaseline};
pub use results::signal::SignalAnalysisReport;
pub use results::transient::{
    TransientAnalysisReport, TransientEventResult, TransientFeatures, TransientFitResult,
};

/// Default logarithm base used whenever a log axis or log transform is enabled
/// without an explicit base override.
///
/// Base 10 is the default because it produces directly interpretable tick
/// labels (orders of magnitude) and is the conventional choice in
/// electrochemical and sensor data visualisation.  Users can always supply an
/// explicit `x_transform_base` / `y_transform_base` (or `x_axis_log_base` /
/// `y_axis_log_base`) to override this for a specific job.
pub const DEFAULT_LOG_BASE: f64 = 10.0;
