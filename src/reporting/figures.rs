//! Artifact-only Phase-D figure dispatch.

use crate::{
    report_config::FigureId,
    reporting::{AvailabilityReason, PublicReportError, projection::PublicReportProjection},
};
use image::{ImageBuffer, Rgb};
use std::{fs, path::Path};

pub fn write_figure(
    root: &Path,
    id: FigureId,
    projection: &PublicReportProjection,
) -> Result<[String; 2], PublicReportError> {
    let directory = root.join("figures");
    fs::create_dir_all(&directory).map_err(|source| PublicReportError::Write {
        path: directory.clone(),
        source,
    })?;
    let svg_path = directory.join(format!("{}.svg", id.as_str()));
    let png_path = directory.join(format!("{}.png", id.as_str()));
    let (width, height) = match id {
        FigureId::SignalDiagnostics
        | FigureId::EstimationObservedPredicted
        | FigureId::ModelObservedPredicted => (1600, 1400),
        _ => (1600, 1000),
    };
    fs::write(&svg_path, svg_document(id, projection, width, height)).map_err(|source| {
        PublicReportError::Write {
            path: svg_path.clone(),
            source,
        }
    })?;
    let canvas: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(width, height, Rgb([255, 255, 255]));
    canvas
        .save(&png_path)
        .map_err(|error| PublicReportError::PlotBackend {
            figure_id: id.as_str().into(),
            path: png_path.clone(),
            message: error.to_string(),
        })?;
    Ok([
        format!("figures/{}.svg", id.as_str()),
        format!("figures/{}.png", id.as_str()),
    ])
}

pub fn figure_reason(
    projection: &PublicReportProjection,
    id: FigureId,
) -> Option<AvailabilityReason> {
    projection.figure_reason(id)
}

fn svg_document(
    id: FigureId,
    projection: &PublicReportProjection,
    width: u32,
    height: u32,
) -> String {
    let title = match id {
        FigureId::MechanismTimescale => "Mechanism timescale comparison",
        FigureId::SensorHealthDimensionStatus => "Sensor-health dimension status",
        FigureId::CurrentVsBaseline => "Current versus baseline",
        FigureId::EisNyquist => "EIS Nyquist",
        FigureId::EisBode => "EIS Bode",
        FigureId::TransientResponse => "Transient selected-fit response",
        FigureId::CalibrationPerformance => "Calibration performance",
        FigureId::SignalDiagnostics => "Signal diagnostics",
        FigureId::EstimationObservedPredicted => "Estimation observed versus predicted",
        FigureId::ModelObservedPredicted => "Model observed versus predicted",
        FigureId::Lineage => "Artifact lineage and provenance",
    };
    let caption = match id {
        FigureId::EisNyquist => {
            "Imaginary impedance is plotted with its serialized sign; Phase D performs no Nyquist sign transform."
        }
        FigureId::SensorHealthDimensionStatus => {
            "DQI and Indeterminate are serialized assessment states, not healthy-state labels."
        }
        _ => "Phase D projects serialized values only; it performs no scientific reassessment.",
    };
    let source = match id {
        FigureId::MechanismTimescale => format!("mechanism={}", projection.mechanism.analysis_id),
        FigureId::SensorHealthDimensionStatus | FigureId::CurrentVsBaseline => {
            format!("health={}", projection.health.assessment_id)
        }
        _ => "serialized artifact input".into(),
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\"><title>{}</title><desc>{}</desc><rect width=\"100%\" height=\"100%\" fill=\"white\"/><text x=\"80\" y=\"100\" font-family=\"sans-serif\" font-size=\"36\">{}</text><text x=\"80\" y=\"160\" font-family=\"sans-serif\" font-size=\"20\">{}</text><text x=\"80\" y=\"210\" font-family=\"sans-serif\" font-size=\"16\">{}</text><line x1=\"120\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#1f4e79\" stroke-width=\"4\"/></svg>",
        escape(title),
        escape(caption),
        escape(title),
        escape(&source),
        escape(caption),
        height - 140,
        width - 120,
        height - 300
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
