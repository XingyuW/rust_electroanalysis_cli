//! Calibration-specific adapters into the existing publication renderer.

use crate::plottings::plotting::{
    PlotColor, PlotSeries, PublicationConfig, plot_rendered_series_hq,
};
use crate::potentiometry::calibration::error::CalibrationError;
use crate::results::calibration::{
    CalibrationAnalysisReport, CalibrationBranch, CalibrationObservationSet,
};
use std::path::{Path, PathBuf};

pub fn plot_calibration_report(
    report: &CalibrationAnalysisReport,
    observations: &CalibrationObservationSet,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, CalibrationError> {
    std::fs::create_dir_all(output_dir)
        .map_err(|error| CalibrationError::export(output_dir, error))?;
    let config = publication_config();
    let mut outputs = Vec::new();
    let Some(selected_model) = report.selected_model else {
        return Ok(outputs);
    };
    let Some(fit) = report
        .candidate_models
        .iter()
        .find(|candidate| candidate.model_kind == selected_model)
    else {
        return Ok(outputs);
    };
    let points = observations
        .observations
        .iter()
        .filter_map(|observation| {
            observation
                .log10_activity()
                .zip(Some(observation.potential_v))
        })
        .collect::<Vec<_>>();
    let fitted = points
        .iter()
        .zip(fit.predicted_potential_v.iter())
        .map(|((x, _), y)| (*x, *y))
        .collect::<Vec<_>>();
    render(
        &output_dir.join("calibration_potential_vs_activity"),
        vec![vec![
            PlotSeries::experimental("observed potential".to_string(), points),
            PlotSeries::fitted("selected calibration model".to_string(), fitted),
        ]],
        config
            .clone()
            .with_default_axis_labels("log10 activity", "Potential (V)"),
        &mut outputs,
    )?;
    let concentration_points = observations
        .observations
        .iter()
        .filter_map(|observation| {
            observation
                .molar_concentration_mol_l
                .filter(|value| value.is_finite() && *value > 0.0)
                .map(|value| (value.log10(), observation.potential_v))
        })
        .collect::<Vec<_>>();
    if !concentration_points.is_empty() {
        render(
            &output_dir.join("calibration_potential_vs_concentration"),
            vec![vec![PlotSeries::experimental(
                "observed potential".to_string(),
                concentration_points,
            )]],
            config
                .clone()
                .with_default_axis_labels("log10 molar concentration (mol/L)", "Potential (V)"),
            &mut outputs,
        )?;
    }
    if let (Some(slope), Some(e0), Some(x_min), Some(x_max)) = (
        fit.theoretical_slope_v_per_decade,
        fit.parameters
            .iter()
            .find(|parameter| parameter.name == "E0")
            .map(|parameter| parameter.value),
        fit.valid_domain.log10_activity_min,
        fit.valid_domain.log10_activity_max,
    ) {
        let theoretical = line_points(x_min, x_max, e0, slope);
        render(
            &output_dir.join("calibration_theoretical_slope"),
            vec![vec![PlotSeries::fitted(
                "theoretical Nernst slope".to_string(),
                theoretical,
            )]],
            config
                .clone()
                .with_default_axis_labels("log10 activity", "Potential (V)"),
            &mut outputs,
        )?;
    }
    let residuals = fit
        .residuals_v
        .iter()
        .enumerate()
        .filter_map(|(index, residual)| {
            observations
                .observations
                .get(index)
                .and_then(|observation| observation.log10_activity())
                .map(|x| (x, *residual))
        })
        .collect::<Vec<_>>();
    if !residuals.is_empty() {
        render(
            &output_dir.join("calibration_residuals"),
            vec![vec![PlotSeries::experimental(
                "potential residual".to_string(),
                residuals,
            )]],
            config
                .clone()
                .with_default_axis_labels("log10 activity", "Residual (V)"),
            &mut outputs,
        )?;
    }
    let ascending = observations
        .observations
        .iter()
        .filter(|observation| observation.branch == CalibrationBranch::Ascending)
        .filter_map(|observation| {
            observation
                .log10_activity()
                .map(|x| (x, observation.potential_v))
        })
        .collect::<Vec<_>>();
    let descending = observations
        .observations
        .iter()
        .filter(|observation| observation.branch == CalibrationBranch::Descending)
        .filter_map(|observation| {
            observation
                .log10_activity()
                .map(|x| (x, observation.potential_v))
        })
        .collect::<Vec<_>>();
    if !ascending.is_empty() || !descending.is_empty() {
        render(
            &output_dir.join("calibration_branches"),
            vec![vec![
                PlotSeries::experimental("ascending".to_string(), ascending),
                PlotSeries::experimental("descending".to_string(), descending),
            ]],
            config
                .clone()
                .with_default_axis_labels("log10 activity", "Potential (V)"),
            &mut outputs,
        )?;
    }
    if let Some(hysteresis) = &report.hysteresis
        && !hysteresis.activity_specific_hysteresis.is_empty()
    {
        render(
            &output_dir.join("calibration_hysteresis"),
            vec![vec![PlotSeries::experimental(
                "descending minus ascending".to_string(),
                hysteresis.activity_specific_hysteresis.clone(),
            )]],
            config
                .clone()
                .with_default_axis_labels("log10 activity", "Hysteresis (V)"),
            &mut outputs,
        )?;
    }
    if let Some(validation) = &report.validation {
        let points = validation
            .predictions
            .iter()
            .filter_map(|point| {
                point
                    .predicted_potential_v
                    .map(|predicted| (point.observed_potential_v, predicted))
            })
            .collect::<Vec<_>>();
        if !points.is_empty() {
            render(
                &output_dir.join("calibration_validation"),
                vec![vec![PlotSeries::experimental(
                    "observed versus predicted potential".to_string(),
                    points,
                )]],
                config
                    .with_default_axis_labels("Observed potential (V)", "Predicted potential (V)"),
                &mut outputs,
            )?;
        }
    }
    Ok(outputs)
}

fn line_points(x_min: f64, x_max: f64, intercept: f64, slope: f64) -> Vec<(f64, f64)> {
    let count = 50usize;
    (0..=count)
        .map(|index| {
            let fraction = index as f64 / count as f64;
            let x = x_min + fraction * (x_max - x_min);
            (x, intercept + slope * x)
        })
        .collect()
}

fn render(
    base: &Path,
    series: Vec<Vec<PlotSeries>>,
    config: PublicationConfig,
    outputs: &mut Vec<PathBuf>,
) -> Result<(), CalibrationError> {
    plot_rendered_series_hq(
        base.to_string_lossy().as_ref(),
        &series,
        &config,
        true,
        crate::plottings::plotting::PlotAxisScale::Linear,
    )
    .map_err(|error| CalibrationError::Plotting {
        path: base.to_path_buf(),
        message: error.to_string(),
    })?;
    outputs.push(base.with_extension("svg"));
    outputs.push(base.with_extension("png"));
    Ok(())
}

fn publication_config() -> PublicationConfig {
    PublicationConfig {
        width_inches: 8.4,
        height_inches: 5.8,
        font_size_pt: 22.0,
        line_width: 3,
        experimental_marker_radius: 5,
        experimental_color: Some(PlotColor::rgb(29, 78, 137)),
        fitted_color: Some(PlotColor::rgb(198, 64, 52)),
        ..Default::default()
    }
}
