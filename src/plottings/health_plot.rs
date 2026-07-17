//! Health plot adapters. Calculations are deliberately owned by `health`.
use crate::results::{HealthTrendReport, SensorHealthAssessment};
use plotters::prelude::*;
use std::path::Path;
pub fn plot_health_assessment(
    report: &SensorHealthAssessment,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let values = report
        .baseline_comparison
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.robust_z_score.or(c.z_score).map(|v| (i as f64, v)))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Ok(());
    }
    let path = output.join("health_feature_deviations.png");
    let root = BitMapBackend::new(&path, (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let ymin = values.iter().map(|p| p.1).fold(0.0, f64::min);
    let ymax = values.iter().map(|p| p.1).fold(0.0, f64::max);
    let pad = (ymax - ymin).abs().max(1.0) * 0.1;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Sensor health: {:?}", report.overall_status),
            ("sans-serif", 24),
        )
        .margin(20)
        .set_all_label_area_size(45)
        .build_cartesian_2d(0.0..values.len() as f64, (ymin - pad)..(ymax + pad))?;
    chart
        .configure_mesh()
        .x_desc("feature index")
        .y_desc("normalized deviation")
        .draw()?;
    chart.draw_series(
        values
            .iter()
            .map(|(x, y)| Circle::new((*x, *y), 4, BLUE.filled())),
    )?;
    root.present()?;
    Ok(())
}
pub fn plot_health_trend(
    report: &HealthTrendReport,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(t) = report.trends.first() else {
        return Ok(());
    };
    let points = t
        .points
        .iter()
        .filter_map(|p| p.independent_value.zip(p.value))
        .collect::<Vec<_>>();
    if points.len() < 2 {
        return Ok(());
    }
    let path = output.join("health_trend.png");
    let root = BitMapBackend::new(&path, (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let xmin = points.first().unwrap().0;
    let xmax = points.last().unwrap().0;
    let ymin = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let ymax = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let pad = (ymax - ymin).abs().max(1e-12) * 0.1;
    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Health trend: {}", t.feature), ("sans-serif", 24))
        .margin(20)
        .set_all_label_area_size(45)
        .build_cartesian_2d(xmin..xmax, (ymin - pad)..(ymax + pad))?;
    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(points, &RED))?;
    root.present()?;
    Ok(())
}
