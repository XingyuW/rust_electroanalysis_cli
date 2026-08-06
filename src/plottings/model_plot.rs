//! Minimal finite-only plots for user-facing ISM model analyses.
use crate::results::ModelAnalysisReport;
use plotters::prelude::*;
use std::path::Path;

pub fn plot_model_analysis(
    report: &ModelAnalysisReport,
    directory: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if report.points.is_empty() {
        return Ok(());
    }
    let x0 = report
        .points
        .first()
        .map(|point| point.time_s)
        .unwrap_or(0.0);
    let x1 = report
        .points
        .last()
        .map(|point| point.time_s)
        .unwrap_or(1.0)
        .max(x0 + 1.0);
    let predicted = report
        .points
        .iter()
        .map(|point| (point.time_s, point.predicted_voltage_v))
        .collect::<Vec<_>>();
    let measured = report
        .points
        .iter()
        .filter_map(|point| point.observed_voltage_v.map(|value| (point.time_s, value)))
        .collect::<Vec<_>>();
    multi_line(
        directory.join("model_measured_vs_predicted.png"),
        x0,
        x1,
        &[("predicted", predicted, RED), ("measured", measured, BLUE)],
        "Measured versus predicted potential",
    )?;
    line(
        directory.join("model_unexplained_residual.png"),
        x0,
        x1,
        &report
            .points
            .iter()
            .filter_map(|point| {
                point
                    .unexplained_residual_v
                    .map(|value| (point.time_s, value))
            })
            .collect::<Vec<_>>(),
        "Unexplained residual",
        BLUE,
    )?;
    line(
        directory.join("model_equilibrium_status.png"),
        x0,
        x1,
        &report
            .points
            .iter()
            .map(|point| {
                (
                    point.time_s,
                    if point.equilibrium.supporting_evidence.is_empty() {
                        0.0
                    } else {
                        1.0
                    },
                )
            })
            .collect::<Vec<_>>(),
        "Equilibrium status evidence",
        GREEN,
    )?;
    let equilibrium_values = report
        .points
        .iter()
        .map(|point| {
            (
                point.time_s,
                point
                    .contributions
                    .iter()
                    .filter(|value| matches!(value.role, crate::model::ComponentRole::Equilibrium))
                    .map(|value| value.voltage_v)
                    .sum(),
            )
        })
        .collect::<Vec<_>>();
    let nonequilibrium_values = report
        .points
        .iter()
        .map(|point| {
            (
                point.time_s,
                point
                    .contributions
                    .iter()
                    .filter(|value| !matches!(value.role, crate::model::ComponentRole::Equilibrium))
                    .map(|value| value.voltage_v)
                    .sum(),
            )
        })
        .collect::<Vec<_>>();
    multi_line(
        directory.join("model_equilibrium_vs_nonequilibrium.png"),
        x0,
        x1,
        &[
            ("equilibrium", equilibrium_values, GREEN),
            ("nonequilibrium", nonequilibrium_values, MAGENTA),
        ],
        "Equilibrium versus nonequilibrium potential",
    )?;
    let mut component_series = std::collections::BTreeMap::<String, Vec<(f64, f64)>>::new();
    for point in &report.points {
        for contribution in &point.contributions {
            component_series
                .entry(contribution.component_id.clone())
                .or_default()
                .push((point.time_s, contribution.voltage_v));
        }
    }
    let component_series = component_series
        .into_iter()
        .enumerate()
        .map(|(index, (name, values))| {
            let colors = [RED, BLUE, GREEN, MAGENTA, CYAN, BLACK];
            (name, values, colors[index % colors.len()])
        })
        .collect::<Vec<_>>();
    multi_line_owned(
        directory.join("model_component_contributions.png"),
        x0,
        x1,
        &component_series,
        "Component contributions",
    )?;
    let mut state_series = std::collections::BTreeMap::<String, Vec<(f64, f64)>>::new();
    for point in &report.points {
        for (state_id, value) in &point.state_values {
            state_series
                .entry(state_id.clone())
                .or_default()
                .push((point.time_s, *value));
        }
    }
    let state_series = state_series
        .into_iter()
        .enumerate()
        .map(|(index, (name, values))| {
            let colors = [BLUE, RED, GREEN, MAGENTA, CYAN, BLACK];
            (name, values, colors[index % colors.len()])
        })
        .collect::<Vec<_>>();
    multi_line_owned(
        directory.join("model_state_trajectories.png"),
        x0,
        x1,
        &state_series,
        "State trajectories",
    )?;
    let parameter_uncertainty = report
        .model_definition
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (index as f64, parameter.uncertainty))
        .collect::<Vec<_>>();
    line(
        directory.join("model_parameter_uncertainty.png"),
        0.0,
        report.model_definition.parameters.len().max(1) as f64,
        &parameter_uncertainty,
        "Parameter uncertainty declaration",
        BLACK,
    )?;
    line(
        directory.join("model_validity_markers.png"),
        x0,
        x1,
        &report
            .points
            .iter()
            .map(|point| {
                (
                    point.time_s,
                    if point.validity.is_valid { 1.0 } else { 0.0 },
                )
            })
            .collect::<Vec<_>>(),
        "Validity and extrapolation markers",
        RED,
    )?;
    Ok(())
}

type BorrowedPlotSeries<'a> = (&'a str, Vec<(f64, f64)>, RGBColor);
type OwnedPlotSeries = (String, Vec<(f64, f64)>, RGBColor);

fn multi_line(
    path: impl AsRef<Path>,
    x0: f64,
    x1: f64,
    series: &[BorrowedPlotSeries<'_>],
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let owned = series
        .iter()
        .map(|(name, values, color)| ((*name).to_string(), values.clone(), *color))
        .collect::<Vec<_>>();
    multi_line_owned(path, x0, x1, &owned, title)
}

fn multi_line_owned(
    path: impl AsRef<Path>,
    x0: f64,
    x1: f64,
    series: &[OwnedPlotSeries],
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let finite = series
        .iter()
        .flat_map(|(_, values, _)| values.iter().map(|(_, value)| *value))
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    let min = finite.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = finite.iter().copied().reduce(f64::max).unwrap_or(1.0);
    let pad = (max - min).abs().max(1e-6) * 0.1;
    let root = BitMapBackend::new(path.as_ref(), (900, 500)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .caption(title, ("sans-serif", 24))
        .set_all_label_area_size(40)
        .build_cartesian_2d(x0..x1, (min - pad)..(max + pad))?;
    chart.configure_mesh().draw()?;
    for (name, values, color) in series {
        if values.is_empty() {
            continue;
        }
        chart
            .draw_series(LineSeries::new(values.clone(), color))?
            .label(name)
            .legend({
                let color = *color;
                move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color)
            });
    }
    chart.configure_series_labels().border_style(BLACK).draw()?;
    root.present()?;
    Ok(())
}
fn line(
    path: impl AsRef<Path>,
    x0: f64,
    x1: f64,
    values: &[(f64, f64)],
    title: &str,
    color: RGBColor,
) -> Result<(), Box<dyn std::error::Error>> {
    let values = if values.is_empty() {
        vec![(x0, 0.0), (x1, 0.0)]
    } else {
        values.to_vec()
    };
    let min = values
        .iter()
        .map(|(_, value)| *value)
        .reduce(f64::min)
        .unwrap_or(0.0);
    let max = values
        .iter()
        .map(|(_, value)| *value)
        .reduce(f64::max)
        .unwrap_or(1.0);
    let pad = (max - min).abs().max(1e-6) * 0.1;
    let root = BitMapBackend::new(path.as_ref(), (900, 500)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .caption(title, ("sans-serif", 24))
        .set_all_label_area_size(40)
        .build_cartesian_2d(x0..x1, (min - pad)..(max + pad))?;
    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(values, &color))?;
    root.present()?;
    Ok(())
}
