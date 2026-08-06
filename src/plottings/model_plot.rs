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
    line(
        directory.join("model_measured_vs_predicted.png"),
        x0,
        x1,
        &report
            .points
            .iter()
            .map(|point| (point.time_s, point.predicted_voltage_v))
            .collect::<Vec<_>>(),
        "Measured versus predicted potential",
        RED,
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
    line(
        directory.join("model_equilibrium_vs_nonequilibrium.png"),
        x0,
        x1,
        &report
            .points
            .iter()
            .map(|point| {
                (
                    point.time_s,
                    point
                        .contributions
                        .iter()
                        .filter(|value| {
                            matches!(value.role, crate::model::ComponentRole::Equilibrium)
                        })
                        .map(|value| value.voltage_v)
                        .sum(),
                )
            })
            .collect::<Vec<_>>(),
        "Equilibrium potential",
        GREEN,
    )?;
    line(
        directory.join("model_component_contributions.png"),
        x0,
        x1,
        &report
            .points
            .iter()
            .map(|point| {
                (
                    point.time_s,
                    point
                        .contributions
                        .iter()
                        .map(|value| value.voltage_v)
                        .sum(),
                )
            })
            .collect::<Vec<_>>(),
        "Component contributions",
        MAGENTA,
    )?;
    line(
        directory.join("model_state_trajectories.png"),
        x0,
        x1,
        &report
            .points
            .iter()
            .map(|point| {
                (
                    point.time_s,
                    point.state_values.iter().map(|(_, value)| value).sum(),
                )
            })
            .collect::<Vec<_>>(),
        "State trajectories",
        BLUE,
    )?;
    line(
        directory.join("model_parameter_uncertainty.png"),
        x0,
        x1,
        &report
            .points
            .iter()
            .map(|point| (point.time_s, 0.0))
            .collect::<Vec<_>>(),
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
