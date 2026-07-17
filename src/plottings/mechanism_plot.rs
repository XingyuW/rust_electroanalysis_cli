//! Plotting adapters for mechanism results; no scientific calculations live here.

use crate::plottings::plotting::{
    PlotAxisScale, PlotSeries, PublicationConfig, plot_rendered_series_hq,
};
use crate::results::MechanismAnalysisReport;
use std::path::{Path, PathBuf};

pub fn plot_mechanism_report(
    report: &MechanismAnalysisReport,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;
    let mut outputs = Vec::new();
    let map = output_dir.join("timescale_map");
    let eis = report
        .eis_timescales
        .iter()
        .filter(|t| t.value_s > 0.0)
        .enumerate()
        .map(|(i, t)| (i as f64, t.value_s.log10()))
        .collect::<Vec<_>>();
    let transient = report
        .transient_timescales
        .iter()
        .filter(|t| t.value_s > 0.0)
        .enumerate()
        .map(|(i, t)| (i as f64, t.value_s.log10()))
        .collect::<Vec<_>>();
    if !eis.is_empty() || !transient.is_empty() {
        plot_rendered_series_hq(
            map.to_string_lossy().as_ref(),
            &[vec![
                PlotSeries::fitted("EIS-derived timescales".to_string(), eis),
                PlotSeries::experimental("transient-fitted timescales".to_string(), transient),
            ]],
            &PublicationConfig::default()
                .with_default_axis_labels("timescale index", "log10(timescale / s)"),
            true,
            PlotAxisScale::Linear,
        )?;
        outputs.push(map.with_extension("svg"));
        outputs.push(map.with_extension("png"));
    }
    let ratio = output_dir.join("timescale_ratio");
    let points = report
        .comparisons
        .iter()
        .filter_map(|c| c.ratio.map(|r| (0.0, r.log10())))
        .collect::<Vec<_>>();
    if !points.is_empty() {
        plot_rendered_series_hq(
            ratio.to_string_lossy().as_ref(),
            &[vec![PlotSeries::experimental(
                "log10 EIS/transient ratio".to_string(),
                points,
            )]],
            &PublicationConfig::default().with_default_axis_labels("comparison", "log10 ratio"),
            true,
            PlotAxisScale::Linear,
        )?;
        outputs.push(ratio.with_extension("svg"));
        outputs.push(ratio.with_extension("png"));
    }
    Ok(outputs)
}
