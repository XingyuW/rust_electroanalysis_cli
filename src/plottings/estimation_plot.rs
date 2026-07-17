use crate::results::StateEstimationReport;
use plotters::prelude::*;
use std::{fs, path::Path};

pub fn plot_estimation_report(
    report: &StateEstimationReport,
    directory: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(directory)?;
    let points = &report.estimates;
    if points.is_empty() {
        return Ok(());
    }
    let (x0, x1) = (
        points.first().unwrap().timestamp_s,
        points
            .last()
            .unwrap()
            .timestamp_s
            .max(points.first().unwrap().timestamp_s + 1.0),
    );
    let potential = directory.join("estimated_potential.png");
    let root = BitMapBackend::new(&potential, (900, 500)).into_drawing_area();
    root.fill(&WHITE)?;
    let ys = points
        .iter()
        .flat_map(|p| {
            [p.measurement_v, p.predicted_measurement_v]
                .into_iter()
                .flatten()
        })
        .collect::<Vec<_>>();
    let ymin = ys.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let ymax = ys.iter().copied().reduce(f64::max).unwrap_or(1.0);
    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .caption("Measured and predicted potential", ("sans-serif", 24))
        .set_all_label_area_size(40)
        .build_cartesian_2d(
            x0..x1,
            (ymin - 0.05 * (ymax - ymin).max(1e-6))..(ymax + 0.05 * (ymax - ymin).max(1e-6)),
        )?;
    chart.configure_mesh().draw()?;
    chart
        .draw_series(LineSeries::new(
            points
                .iter()
                .filter_map(|p| p.measurement_v.map(|v| (p.timestamp_s, v))),
            &BLUE,
        ))?
        .label("measured");
    chart
        .draw_series(LineSeries::new(
            points
                .iter()
                .filter_map(|p| p.predicted_measurement_v.map(|v| (p.timestamp_s, v))),
            &RED,
        ))?
        .label("predicted");
    chart.configure_series_labels().draw()?;
    root.present()?;
    let activity = directory.join("estimated_activity.png");
    let root = BitMapBackend::new(&activity, (900, 500)).into_drawing_area();
    root.fill(&WHITE)?;
    let vals = points
        .iter()
        .filter_map(|p| p.activity.map(f64::log10))
        .collect::<Vec<_>>();
    let lo = vals.iter().copied().reduce(f64::min).unwrap_or(-6.0);
    let hi = vals.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .caption("Estimated log10 activity", ("sans-serif", 24))
        .set_all_label_area_size(40)
        .build_cartesian_2d(x0..x1, (lo - 0.5)..(hi + 0.5))?;
    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(
        points
            .iter()
            .filter_map(|p| p.activity.map(|v| (p.timestamp_s, v.log10()))),
        &GREEN,
    ))?;
    root.present()?;
    for (state, filename, title, color) in [
        (
            "baseline_offset",
            "estimated_baseline.png",
            "Estimated baseline offset",
            RED,
        ),
        (
            "polarization",
            "estimated_polarization.png",
            "Estimated polarization",
            MAGENTA,
        ),
        (
            "sensitivity_scale",
            "estimated_condition.png",
            "Estimated sensor-condition proxy",
            BLACK,
        ),
    ] {
        let values = points
            .iter()
            .filter_map(|point| {
                point
                    .filtered_state
                    .iter()
                    .find(|value| value.name == state)
                    .and_then(|value| value.value.map(|x| (point.timestamp_s, x)))
            })
            .collect::<Vec<_>>();
        if !values.is_empty() {
            line_plot(&directory.join(filename), x0, x1, &values, title, color)?;
        }
    }
    let innovations = report
        .diagnostics
        .innovations
        .iter()
        .map(|record| (record.timestamp_s, record.innovation_v))
        .collect::<Vec<_>>();
    if !innovations.is_empty() {
        line_plot(
            &directory.join("estimated_innovations.png"),
            x0,
            x1,
            &innovations,
            "Innovation",
            BLUE,
        )?;
        let nis = report
            .diagnostics
            .innovations
            .iter()
            .map(|record| (record.timestamp_s, record.normalized_innovation_squared))
            .collect::<Vec<_>>();
        line_plot(
            &directory.join("estimated_nis.png"),
            x0,
            x1,
            &nis,
            "Normalized innovation squared",
            GREEN,
        )?;
    }
    Ok(())
}

fn line_plot(
    path: &Path,
    x0: f64,
    x1: f64,
    values: &[(f64, f64)],
    title: &str,
    color: RGBColor,
) -> Result<(), Box<dyn std::error::Error>> {
    let ymin = values
        .iter()
        .map(|(_, y)| *y)
        .reduce(f64::min)
        .unwrap_or(0.0);
    let ymax = values
        .iter()
        .map(|(_, y)| *y)
        .reduce(f64::max)
        .unwrap_or(1.0);
    let padding = 0.05 * (ymax - ymin).max(1e-9);
    let root = BitMapBackend::new(path, (900, 500)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .caption(title, ("sans-serif", 24))
        .set_all_label_area_size(40)
        .build_cartesian_2d(x0..x1, (ymin - padding)..(ymax + padding))?;
    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(values.iter().copied(), &color))?;
    root.present()?;
    Ok(())
}
