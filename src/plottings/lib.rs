//! Plotting subsystem facade.
//!
//! Re-exports domain-specific plot runners (`chi_plot`, `eis_plot`,
//! `generic_plot`) and the shared low-level renderer (`plotting`).

pub mod calibration_plot;
pub mod chi_plot;
pub mod eis_plot;
pub mod estimation_plot;
pub mod generic_plot;
pub mod health_plot;
pub mod mechanism_plot;
pub mod plotting;
pub mod signal_plot;
pub mod transient_plot;

pub use calibration_plot::plot_calibration_report;
pub use chi_plot::{
    ChiDirectoryPlotOutcome, ChiPlotOutcome, ChiPlotSkip, pb_sensor_combined_publication_config,
    pb_sensor_individual_publication_config, pb_sensor_publication_config, plot_chi_directory,
    plot_chi_directory_with_configs, plot_chi_directory_with_configs_and_transforms, plot_chi_file,
    plot_chi_file_with_transform,
};
pub use eis_plot::{
    EISDirectoryPlotOutcome, EISPlotOutcome, RankedSearchPlotOutcome, best_ranked_search_fit,
    eis_combined_publication_config, eis_individual_publication_config, eis_publication_config,
    plot_eis_directory, plot_eis_directory_with_configs, plot_eis_file, plot_ranked_search_report,
};
pub use generic_plot::{
    GenericDirectoryPlotOutcome, GenericPlotOutcome, GenericPlotSkip, LoadedGenericDataset,
    generic_combined_publication_config, generic_individual_publication_config,
    load_generic_datasets_from_dir, plot_generic_datasets, plot_generic_directory,
    plot_generic_directory_with_configs,
};
pub use mechanism_plot::plot_mechanism_report;
pub use plotting::{
    AxisScale, AxisScaleKind, FillBetweenMode, PieValueLabelMode, PlotAxisScale, PlotColor,
    PlotDataSeries, PlotLegendPosition, PlotLineStyle, PlotMarkerShape, PlotSeries, PlotSeriesKind,
    PlotType, PublicationConfig, RegressionAnnotationLayout, ResolvedPlotConfig,
    ScientificNotationStyle, plot_hq, plot_rendered_series_hq, plot_rendered_series_panels_hq,
};
pub use transient_plot::plot_transient_event;
