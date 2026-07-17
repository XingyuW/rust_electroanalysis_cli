//! Transient-specific plotting adapters.
//!
//! This module converts transient results to existing `PlotSeries` values and
//! delegates rendering to the unchanged publication renderer.

use crate::plottings::plotting::{
    PlotColor, PlotSeries, PublicationConfig, plot_rendered_series_hq,
};
use crate::potentiometry::PotentiometryError;
use crate::potentiometry::transient::models::evaluate;
use crate::results::transient::{FitStatus, TransientEventResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn plot_transient_event(
    event: &TransientEventResult,
    output_dir: &Path,
    include_components: bool,
    include_residuals: bool,
    include_model_comparison: bool,
) -> Result<Vec<PathBuf>, PotentiometryError> {
    let Some(selected_model) = event.selected_model else {
        return Ok(Vec::new());
    };
    let fit = event
        .candidate_fits
        .iter()
        .find(|fit| fit.model == selected_model && fit.status == FitStatus::Converged)
        .ok_or_else(|| PotentiometryError::invalid("selected transient fit is unavailable"))?;
    fs::create_dir_all(output_dir)
        .map_err(|error| PotentiometryError::export(output_dir, error))?;
    let stem = format!("transient_event_{}", event.event_index);
    let response_base = output_dir.join(format!("{stem}_response"));
    let residual_base = output_dir.join(format!("{stem}_residuals"));
    let comparison_base = output_dir.join(format!("{stem}_model_comparison"));
    let config = transient_publication_config();
    let mut outputs = Vec::new();

    let raw = PlotSeries::experimental(
        "observed response".to_string(),
        event
            .segment
            .raw_time_local
            .iter()
            .zip(event.segment.raw_potential_v.iter())
            .filter_map(|(time, value)| value.map(|value| (*time, value)))
            .collect(),
    );
    let fitted = PlotSeries::fitted(
        format!("{} fitted response", selected_model),
        event
            .segment
            .fitted_time_local
            .iter()
            .copied()
            .zip(fit.predicted_v.iter().copied())
            .collect(),
    );
    let mut response_group = vec![raw, fitted];
    if let Some(baseline) = event.baseline.estimate_v {
        let end = event.segment.local_end.unwrap_or(0.0).max(0.0);
        response_group.push(PlotSeries::fitted(
            "pre-event baseline".to_string(),
            vec![(0.0, baseline), (end, baseline)],
        ));
    }
    response_group.push(event_marker(event));
    if include_components {
        let component_points = event_components(event, fit);
        response_group.extend(component_points);
    }
    render(
        &response_base,
        vec![response_group],
        config
            .clone()
            .with_default_axis_labels("Local time (s)", "Potential (V)"),
        &mut outputs,
    )?;

    if include_residuals {
        let residual_series = PlotSeries::experimental(
            "fit residual".to_string(),
            event
                .segment
                .fitted_time_local
                .iter()
                .copied()
                .zip(fit.residuals_v.iter().copied())
                .collect(),
        );
        render(
            &residual_base,
            vec![vec![residual_series]],
            config.with_default_axis_labels("Local time (s)", "Residual (V)"),
            &mut outputs,
        )?;
    }

    let comparison_points = event
        .candidate_fits
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            let value = candidate.statistics.aic?;
            value.is_finite().then_some((index as f64, value))
        })
        .collect::<Vec<_>>();
    if include_model_comparison && !comparison_points.is_empty() {
        render(
            &comparison_base,
            vec![vec![PlotSeries::experimental(
                "candidate AIC".to_string(),
                comparison_points,
            )]],
            transient_publication_config().with_default_axis_labels("Candidate model index", "AIC"),
            &mut outputs,
        )?;
    }

    Ok(outputs)
}

fn event_components(
    event: &TransientEventResult,
    fit: &crate::results::transient::TransientFitResult,
) -> Vec<PlotSeries> {
    let mut fast = Vec::new();
    let mut slow = Vec::new();
    let mut drift = Vec::new();
    for time in &event.segment.fitted_time_local {
        if let Ok(components) = evaluate(fit.model, &fit.fit_parameters, *time) {
            if let Some(value) = components.fast {
                fast.push((*time, value));
            }
            if let Some(value) = components.slow {
                slow.push((*time, value));
            }
            if let Some(value) = components.drift {
                drift.push((*time, value));
            }
        }
    }
    let mut series = Vec::new();
    if !fast.is_empty() {
        series.push(PlotSeries::fitted("fast component".to_string(), fast));
    }
    if !slow.is_empty() {
        series.push(PlotSeries::fitted("slow component".to_string(), slow));
    }
    if !drift.is_empty() {
        series.push(PlotSeries::fitted("drift component".to_string(), drift));
    }
    series
}

fn event_marker(event: &TransientEventResult) -> PlotSeries {
    let values = event
        .segment
        .raw_potential_v
        .iter()
        .flatten()
        .copied()
        .chain(event.baseline.estimate_v)
        .collect::<Vec<_>>();
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (min, max) = if min.is_finite() && max.is_finite() && min < max {
        (min, max)
    } else {
        (0.0, 1.0)
    };
    PlotSeries::fitted("event".to_string(), vec![(0.0, min), (0.0, max)])
}

fn transient_publication_config() -> PublicationConfig {
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

fn render(
    base: &Path,
    series: Vec<Vec<PlotSeries>>,
    config: PublicationConfig,
    outputs: &mut Vec<PathBuf>,
) -> Result<(), PotentiometryError> {
    plot_rendered_series_hq(
        base.to_string_lossy().as_ref(),
        &series,
        &config,
        true,
        crate::plottings::plotting::PlotAxisScale::Linear,
    )
    .map_err(|error| PotentiometryError::Plotting {
        path: base.to_path_buf(),
        message: error.to_string(),
    })?;
    outputs.push(base.with_extension("svg"));
    outputs.push(base.with_extension("png"));
    Ok(())
}
