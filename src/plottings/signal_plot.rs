//! Signal plot adapters. Calculations are deliberately owned by `signal`.
use crate::results::SignalAnalysisReport;
use plotters::prelude::*;
use std::path::Path;

pub fn plot_signal_report(
    report: &SignalAnalysisReport,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    plot_xy(
        output.join("signal_raw.png"),
        "Raw signal",
        &report.analysis_timestamps,
        &report.analysis_values,
        &report.unit,
    )?;
    let intervals = report
        .analysis_timestamps
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect::<Vec<_>>();
    let interval_time = report
        .analysis_timestamps
        .iter()
        .skip(1)
        .copied()
        .collect::<Vec<_>>();
    plot_values(
        output.join("signal_sampling_interval.png"),
        "Sampling interval",
        &interval_time,
        &intervals,
        "s",
    )?;
    if let Some(psd) = &report.psd {
        plot_values(
            output.join("signal_psd.png"),
            "Welch PSD",
            &psd.frequency_hz,
            &psd.psd,
            &psd.psd_unit,
        )?;
        plot_values(
            output.join("signal_asd.png"),
            "Amplitude spectral density",
            &psd.frequency_hz,
            &psd.amplitude_spectral_density,
            "unit/sqrt(Hz)",
        )?;
    }
    if let Some(allan) = &report.allan {
        let x = allan
            .points
            .iter()
            .map(|p| p.averaging_time_s)
            .collect::<Vec<_>>();
        let y = allan.points.iter().map(|p| p.deviation).collect::<Vec<_>>();
        plot_xy(
            output.join("signal_allan.png"),
            "Allan deviation",
            &x,
            &y,
            &report.unit,
        )?;
    }
    if !report.spikes.flagged.is_empty() {
        let x = report
            .spikes
            .flagged
            .iter()
            .map(|p| p.timestamp_s)
            .collect::<Vec<_>>();
        let y = report
            .spikes
            .flagged
            .iter()
            .map(|p| Some(p.value))
            .collect::<Vec<_>>();
        plot_xy(
            output.join("signal_spike_flags.png"),
            "Spike flags",
            &x,
            &y,
            &report.unit,
        )?;
    }
    for (i, c) in report.correlations.iter().enumerate() {
        if !c.lags_s.is_empty() {
            plot_values(
                output.join(format!("signal_cross_correlation_{i}.png")),
                "Cross-correlation",
                &c.lags_s,
                &c.cross_correlation,
                "correlation",
            )?;
        }
    }
    Ok(())
}
fn plot_xy(
    path: impl AsRef<Path>,
    title: &str,
    x: &[f64],
    y: &[Option<f64>],
    unit: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let points = x
        .iter()
        .copied()
        .zip(y.iter().copied().flatten())
        .collect::<Vec<_>>();
    if points.len() < 2 {
        return Ok(());
    }
    let (xmin, xmax) = (points.first().unwrap().0, points.last().unwrap().0);
    let ymin = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let ymax = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let pad = ((ymax - ymin).abs() * 0.05).max(1e-12);
    let root = BitMapBackend::new(path.as_ref(), (1000, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(format!("{title} [{unit}]"), ("sans-serif", 24))
        .margin(20)
        .set_all_label_area_size(45)
        .build_cartesian_2d(xmin..xmax, (ymin - pad)..(ymax + pad))?;
    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(points, &BLUE))?;
    root.present()?;
    Ok(())
}
fn plot_values(
    path: impl AsRef<Path>,
    title: &str,
    x: &[f64],
    y: &[f64],
    unit: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    plot_xy(
        path,
        title,
        x,
        &y.iter().copied().map(Some).collect::<Vec<_>>(),
        unit,
    )
}
