//! Artifact-only Phase-D scientific figure dispatch.
//!
//! This drawing backend receives prepared, serialized coordinates only. Axis
//! placement is display geometry and never changes stored scientific values.

use crate::{
    domain::ArtifactLineageState,
    report_config::FigureId,
    reporting::{
        AvailabilityReason, PublicReportError, projection::PublicReportProjection,
        tables::format_public_f64,
    },
    results::FeatureComparability,
};
use image::{ImageBuffer, Rgb};
use std::{fs, path::Path};

const BLUE: Rgb<u8> = Rgb([31, 78, 121]);
const ORANGE: Rgb<u8> = Rgb([214, 115, 33]);
const GREEN: Rgb<u8> = Rgb([46, 125, 50]);
const RED: Rgb<u8> = Rgb([183, 28, 28]);
const BLACK: Rgb<u8> = Rgb([30, 30, 30]);
const WHITE: Rgb<u8> = Rgb([255, 255, 255]);

#[derive(Clone)]
struct Point {
    x: f64,
    y: Option<f64>,
    x_text: String,
    y_text: String,
}

#[derive(Clone)]
struct Series {
    label: String,
    points: Vec<Point>,
    colour: Rgb<u8>,
}

struct FigurePayload {
    title: &'static str,
    x_label: String,
    y_label: String,
    caption: String,
    series: Vec<Series>,
}

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
    let payload = payload(id, projection)?;
    fs::write(&svg_path, svg_document(id, &payload, width, height)).map_err(|source| {
        PublicReportError::Write {
            path: svg_path.clone(),
            source,
        }
    })?;
    write_png(&png_path, &payload, width, height, id)?;
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

fn payload(id: FigureId, p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    match id {
        FigureId::MechanismTimescale => mechanism_payload(p),
        FigureId::SensorHealthDimensionStatus => health_payload(p),
        FigureId::CurrentVsBaseline => baseline_payload(p),
        FigureId::EisNyquist => eis_nyquist_payload(p),
        FigureId::EisBode => eis_bode_payload(p),
        FigureId::TransientResponse => transient_payload(p),
        FigureId::CalibrationPerformance => calibration_payload(p),
        FigureId::SignalDiagnostics => signal_payload(p),
        FigureId::EstimationObservedPredicted => estimation_payload(p),
        FigureId::ModelObservedPredicted => model_payload(p),
        FigureId::Lineage => lineage_payload(p),
    }
}

fn mechanism_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let points = p
        .mechanism
        .comparisons
        .iter()
        .enumerate()
        .map(|(i, row)| point(i as f64, row.log10_distance, row.comparison_id.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FigurePayload {
        title: "Mechanism timescale comparison",
        x_label: "Serialized comparison ID".into(),
        y_label: "Stored log10 distance [dimensionless]".into(),
        caption: format!(
            "Mechanism artifact {}. Values are the serialized log10_distance field; Phase D performs no log10 calculation.",
            p.mechanism.analysis_id
        ),
        series: vec![Series {
            label: "producer assessment".into(),
            points,
            colour: BLUE,
        }],
    })
}

fn health_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let points = p
        .health
        .phase_c
        .as_ref()
        .map(|phase| {
            phase
                .dimension_assessments
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    point_text(
                        i as f64,
                        i as f64,
                        token(&item.dimension),
                        format!("{}; {}", token(&item.status), token(&item.evidence_state)),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(FigurePayload {
        title: "Sensor-health dimension status",
        x_label: "Phase-C health dimension".into(),
        y_label: "Serialized status (categorical)".into(),
        caption: if p.health_is_legacy() {
            "Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized.".into()
        } else {
            format!(
                "Health artifact {}. All nine serialized Phase-C dimensions are shown, including Data quality insufficient (DQI) and Indeterminate where present.",
                p.health.assessment_id
            )
        },
        series: vec![Series {
            label: "producer assessment".into(),
            points,
            colour: ORANGE,
        }],
    })
}

fn baseline_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let mut current = Vec::new();
    let mut baseline = Vec::new();
    let mut warnings = Vec::new();
    for (i, comparison) in p.health.baseline_comparison.iter().enumerate() {
        if !matches!(
            comparison.comparability,
            FeatureComparability::Comparable | FeatureComparability::ComparableWithWarnings
        ) {
            continue;
        }
        let unit_count = p
            .health
            .features
            .iter()
            .filter(|feature| feature.name == comparison.feature && !feature.unit.is_empty())
            .count();
        if unit_count != 1 {
            continue;
        }
        current.push(point(
            i as f64,
            comparison.current_value,
            comparison.feature.clone(),
        )?);
        baseline.push(point(
            i as f64,
            comparison.baseline_value,
            comparison.feature.clone(),
        )?);
        if matches!(
            comparison.comparability,
            FeatureComparability::ComparableWithWarnings
        ) {
            warnings.push(
                comparison
                    .override_reason
                    .clone()
                    .unwrap_or_else(|| "Comparable with upstream context warning.".into()),
            );
        }
    }
    let warning = if warnings.is_empty() {
        String::new()
    } else {
        format!(" Warnings: {}", warnings.join("; "))
    };
    Ok(FigurePayload {
        title: "Current versus baseline",
        x_label: "Serialized health feature".into(),
        y_label: "Serialized value; source-authoritative unit".into(),
        caption: format!(
            "Health artifact {}. Comparable-with-warnings pairs remain rendered without conversion.{}",
            p.health.assessment_id, warning
        ),
        series: vec![
            Series {
                label: "current".into(),
                points: current,
                colour: BLUE,
            },
            Series {
                label: "baseline".into(),
                points: baseline,
                colour: ORANGE,
            },
        ],
    })
}

fn eis_nyquist_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let eis = p
        .eis
        .as_ref()
        .expect("availability is checked before rendering");
    Ok(FigurePayload {
        title: "EIS Nyquist",
        x_label: "Re(Z) [Ohm]".into(),
        y_label: "Im(Z) [Ohm]".into(),
        caption: format!(
            "EIS artifact {}. Imaginary impedance uses its serialized sign; Phase D performs no Nyquist sign transformation.",
            eis.fit_id
        ),
        series: vec![
            Series {
                label: "observed".into(),
                points: paired(
                    "EIS observed",
                    &eis.source.z_real_ohm,
                    &eis.source.z_imag_ohm,
                )?,
                colour: BLUE,
            },
            Series {
                label: "fitted".into(),
                points: paired("EIS fitted", &eis.fitted.z_real_ohm, &eis.fitted.z_imag_ohm)?,
                colour: ORANGE,
            },
        ],
    })
}

fn eis_bode_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let eis = p
        .eis
        .as_ref()
        .expect("availability is checked before rendering");
    let source_magnitude = eis
        .source
        .source_measured_magnitude_ohm
        .as_ref()
        .cloned()
        .unwrap_or_else(|| {
            eis.source
                .derived_magnitude_ohm
                .iter()
                .copied()
                .map(Some)
                .collect()
        });
    let source_phase = eis
        .source
        .source_measured_phase_deg
        .as_ref()
        .cloned()
        .unwrap_or_else(|| {
            eis.source
                .derived_phase_deg
                .iter()
                .copied()
                .map(Some)
                .collect()
        });
    Ok(FigurePayload {
        title: "EIS Bode",
        x_label: "Frequency [Hz]".into(),
        y_label: "Magnitude [Ohm] / phase [deg]".into(),
        caption: format!(
            "EIS artifact {}. Magnitude and phase are source-provided where serialized, otherwise the artifact's explicitly serialized derived channels; Phase D derives neither.",
            eis.fit_id
        ),
        series: vec![
            Series {
                label: "observed magnitude".into(),
                points: paired_optional(
                    "EIS observed magnitude",
                    &eis.source.frequency_hz,
                    &source_magnitude,
                )?,
                colour: BLUE,
            },
            Series {
                label: "fitted magnitude".into(),
                points: paired(
                    "EIS fitted magnitude",
                    &eis.source.frequency_hz,
                    &eis.fitted.magnitude_ohm,
                )?,
                colour: ORANGE,
            },
            Series {
                label: "observed phase".into(),
                points: paired_optional(
                    "EIS observed phase",
                    &eis.source.frequency_hz,
                    &source_phase,
                )?,
                colour: GREEN,
            },
            Series {
                label: "fitted phase".into(),
                points: paired(
                    "EIS fitted phase",
                    &eis.source.frequency_hz,
                    &eis.fitted.phase_deg,
                )?,
                colour: RED,
            },
        ],
    })
}

fn transient_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let transient = p
        .transient
        .as_ref()
        .expect("availability is checked before rendering");
    let mut observed = Vec::new();
    let mut fitted = Vec::new();
    let mut residual = Vec::new();
    for event in &transient.events {
        let Some(model) = &event.selected_model else {
            continue;
        };
        let matches = event
            .candidate_fits
            .iter()
            .filter(|fit| fit.is_successful() && &fit.model == model)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            continue;
        }
        let fit = matches[0];
        observed.extend(paired_optional(
            "transient observed",
            &event.segment.raw_time_local,
            &event.segment.raw_potential_v,
        )?);
        let predicted_time = if event.segment.fitted_time_local.len() == fit.predicted_v.len() {
            &event.segment.fitted_time_local
        } else {
            &event.segment.raw_time_local
        };
        let residual_time = if event.segment.fitted_time_local.len() == fit.residuals_v.len() {
            &event.segment.fitted_time_local
        } else {
            &event.segment.raw_time_local
        };
        fitted.extend(paired(
            "transient fitted",
            predicted_time,
            &fit.predicted_v,
        )?);
        residual.extend(paired(
            "transient residual",
            residual_time,
            &fit.residuals_v,
        )?);
    }
    Ok(FigurePayload {
        title: "Transient selected-fit response",
        x_label: "Serialized local time [s]".into(),
        y_label: "Potential [V]".into(),
        caption: format!(
            "Transient artifact {}. Only the uniquely serialized successful candidate matching each selected model is shown; no model evaluation, refit, ranking, or first-candidate fallback is used.",
            transient.experiment_id
        ),
        series: vec![
            Series {
                label: "observed".into(),
                points: observed,
                colour: BLUE,
            },
            Series {
                label: "fitted".into(),
                points: fitted,
                colour: ORANGE,
            },
            Series {
                label: "residual".into(),
                points: residual,
                colour: RED,
            },
        ],
    })
}

fn calibration_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let calibration = p
        .calibration
        .as_ref()
        .expect("availability is checked before rendering");
    let predictions = calibration
        .validation
        .as_ref()
        .map(|value| &value.predictions)
        .expect("availability is checked before rendering");
    let mut observed = Vec::new();
    let mut predicted = Vec::new();
    for row in predictions {
        let Some(x) = row.observed_log10_activity else {
            continue;
        };
        observed.push(point_text(
            x,
            row.observed_potential_v,
            format_public_f64(x).map_err(staging_number_error)?,
            format_public_f64(row.observed_potential_v).map_err(staging_number_error)?,
        ));
        predicted.push(point(
            x,
            row.predicted_potential_v,
            format_public_f64(x).map_err(staging_number_error)?,
        )?);
    }
    Ok(FigurePayload {
        title: "Calibration performance",
        x_label: "Serialized observed log10 activity".into(),
        y_label: "Potential [V]".into(),
        caption: format!(
            "Calibration artifact {}. The series consists only of serialized validation predictions and observations; Phase D draws no theoretical line and performs no activity logarithm calculation.",
            calibration.calibration_id
        ),
        series: vec![
            Series {
                label: "observed".into(),
                points: observed,
                colour: BLUE,
            },
            Series {
                label: "predicted".into(),
                points: predicted,
                colour: ORANGE,
            },
        ],
    })
}

fn signal_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let signal = p
        .signal
        .as_ref()
        .expect("availability is checked before rendering");
    let time = paired_optional(
        "signal time",
        &signal.analysis_timestamps,
        &signal.analysis_values,
    )?;
    let psd = signal
        .psd
        .as_ref()
        .map(|value| paired("signal PSD", &value.frequency_hz, &value.psd))
        .transpose()?
        .unwrap_or_default();
    let allan = signal
        .allan
        .as_ref()
        .map(|value| {
            value
                .points
                .iter()
                .map(|allan_point| {
                    point(
                        allan_point.averaging_time_s,
                        allan_point.deviation,
                        format_public_f64(allan_point.averaging_time_s)
                            .map_err(staging_number_error)?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(FigurePayload {
        title: "Signal diagnostics",
        x_label: "Serialized timestamp [s], frequency [Hz], or averaging time [s]".into(),
        y_label: format!("Serialized signal diagnostics [{}]", signal.unit),
        caption: format!(
            "Signal artifact {}. Time, PSD, and Allan panels use only serialized series. Missing samples are visible as NA markers; Phase D performs no resampling, PSD, or Allan calculation.",
            signal.analysis_id
        ),
        series: vec![
            Series {
                label: "time".into(),
                points: time,
                colour: BLUE,
            },
            Series {
                label: "PSD".into(),
                points: psd,
                colour: ORANGE,
            },
            Series {
                label: "Allan".into(),
                points: allan,
                colour: GREEN,
            },
        ],
    })
}

fn estimation_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let estimation = p
        .estimation
        .as_ref()
        .expect("availability is checked before rendering");
    let observed = estimation
        .estimates
        .iter()
        .map(|estimate| {
            point(
                estimate.timestamp_s,
                estimate.measurement_v,
                format_public_f64(estimate.timestamp_s).map_err(staging_number_error)?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let predicted = estimation
        .estimates
        .iter()
        .map(|estimate| {
            point(
                estimate.timestamp_s,
                estimate.predicted_measurement_v,
                format_public_f64(estimate.timestamp_s).map_err(staging_number_error)?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FigurePayload {
        title: "Estimation observed versus predicted",
        x_label: "Serialized timestamp [s]".into(),
        y_label: "Potential [V]".into(),
        caption: format!(
            "State-estimation artifact {}. Observed and predicted potential are serialized producer outputs; no uncertainty interval is invented from variance.",
            estimation.analysis_id
        ),
        series: vec![
            Series {
                label: "observed".into(),
                points: observed,
                colour: BLUE,
            },
            Series {
                label: "predicted".into(),
                points: predicted,
                colour: ORANGE,
            },
        ],
    })
}

fn model_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let model = p
        .model
        .as_ref()
        .expect("availability is checked before rendering");
    let observed = model
        .points
        .iter()
        .map(|model_point| {
            point(
                model_point.time_s,
                model_point.observed_voltage_v,
                format_public_f64(model_point.time_s).map_err(staging_number_error)?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let predicted = model
        .points
        .iter()
        .map(|model_point| {
            point(
                model_point.time_s,
                Some(model_point.predicted_voltage_v),
                format_public_f64(model_point.time_s).map_err(staging_number_error)?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let residual = model
        .points
        .iter()
        .map(|model_point| {
            point(
                model_point.time_s,
                model_point.unexplained_residual_v,
                format_public_f64(model_point.time_s).map_err(staging_number_error)?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FigurePayload {
        title: "Model observed versus predicted", x_label: "Serialized time [s]".into(), y_label: "Potential [V]".into(),
        caption: "Model output points are serialized model-derived predictions. Missing observed or residual values remain NA; Phase D never recomputes residuals or maps missing values to zero.".into(),
        series: vec![Series { label: "observed".into(), points: observed, colour: BLUE }, Series { label: "predicted".into(), points: predicted, colour: ORANGE }, Series { label: "residual".into(), points: residual, colour: RED }],
    })
}

fn lineage_payload(p: &PublicReportProjection) -> Result<FigurePayload, PublicReportError> {
    let mut points = Vec::new();
    for (i, (flag, lineage)) in p.supplied_lineages().into_iter().enumerate() {
        let label = match lineage {
            ArtifactLineageState::Known {
                identity,
                direct_dependencies,
            } => format!(
                "{flag}: {} schema {}; direct_dependencies={}",
                identity.artifact_id.0,
                identity.schema_version,
                direct_dependencies.len()
            ),
            ArtifactLineageState::LegacyUnknown { reason, .. } => {
                format!("{flag}: lineage unknown: {}", token(reason))
            }
        };
        points.push(point_text(i as f64, i as f64, flag.into(), label));
    }
    Ok(FigurePayload {
        title: "Artifact lineage",
        x_label: "Input artifact flag".into(),
        y_label: "Root/direct-dependency provenance (non-scientific)".into(),
        caption: if p.lineage_catalog.is_some() {
            "Only supplied roots and their serialized direct dependencies are projected. Catalog membership is not lineage traversal or resolution.".into()
        } else {
            "Lineage catalog not supplied; only serialized direct lineage is shown. LegacyUnknown is displayed explicitly.".into()
        },
        series: vec![Series {
            label: "serialized root".into(),
            points,
            colour: GREEN,
        }],
    })
}

fn point(x: f64, y: Option<f64>, x_text: String) -> Result<Point, PublicReportError> {
    if !x.is_finite() {
        return Err(staging_number_error(
            "non-finite number in public projection",
        ));
    }
    let y_text = y
        .map(format_public_f64)
        .transpose()
        .map_err(staging_number_error)?
        .unwrap_or_else(|| "NA".into());
    Ok(Point {
        x,
        y,
        x_text,
        y_text,
    })
}
fn point_text(x: f64, y: f64, x_text: String, y_text: String) -> Point {
    Point {
        x,
        y: Some(y),
        x_text,
        y_text,
    }
}
fn paired(label: &str, x: &[f64], y: &[f64]) -> Result<Vec<Point>, PublicReportError> {
    paired_optional(label, x, &y.iter().copied().map(Some).collect::<Vec<_>>())
}
fn paired_optional(
    label: &str,
    x: &[f64],
    y: &[Option<f64>],
) -> Result<Vec<Point>, PublicReportError> {
    if x.len() != y.len() {
        return Err(PublicReportError::StagingValidation {
            path: Path::new("public report projection").to_path_buf(),
            detail: format!("{label} has mismatched serialized coordinate lengths"),
        });
    }
    x.iter()
        .copied()
        .zip(y.iter().copied())
        .map(|(x, y)| point(x, y, format_public_f64(x).map_err(staging_number_error)?))
        .collect()
}
fn staging_number_error(detail: impl Into<String>) -> PublicReportError {
    PublicReportError::StagingValidation {
        path: Path::new("public report projection").to_path_buf(),
        detail: detail.into(),
    }
}
fn token<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_default()
        .trim_matches('"')
        .to_owned()
}

fn svg_document(id: FigureId, payload: &FigurePayload, width: u32, height: u32) -> String {
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\"><title>{}</title><desc>{}</desc><metadata>phase_d_figure={}; threshold_lines=0</metadata><rect width=\"100%\" height=\"100%\" fill=\"white\"/><text x=\"80\" y=\"70\" font-family=\"sans-serif\" font-size=\"36\">{}</text><text x=\"80\" y=\"110\" font-family=\"sans-serif\" font-size=\"18\">x: {}</text><text x=\"80\" y=\"140\" font-family=\"sans-serif\" font-size=\"18\">y: {}</text><line x1=\"120\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#1e1e1e\" stroke-width=\"2\"/><line x1=\"120\" y1=\"190\" x2=\"120\" y2=\"{}\" stroke=\"#1e1e1e\" stroke-width=\"2\"/>",
        escape(payload.title),
        escape(&payload.caption),
        id.as_str(),
        escape(payload.title),
        escape(&payload.x_label),
        escape(&payload.y_label),
        height - 130,
        width - 100,
        height - 130,
        height - 130
    );
    for (series_index, series) in payload.series.iter().enumerate() {
        svg.push_str(&format!(
            "<g data-series=\"{}\"><title>{}</title>",
            escape(&series.label),
            escape(&series.label)
        ));
        for point in &series.points {
            let (x, y) = point_position(point, series_index, width, height);
            svg.push_str(&format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"5\" fill=\"{}\"><title>x={} y={}</title></circle>", colour_hex(series.colour), escape(&point.x_text), escape(&point.y_text)));
        }
        svg.push_str("</g>");
    }
    svg.push_str(&format!(
        "<text x=\"80\" y=\"{}\" font-family=\"sans-serif\" font-size=\"16\">{}</text></svg>",
        height - 70,
        escape(&payload.caption)
    ));
    svg
}

fn write_png(
    path: &Path,
    payload: &FigurePayload,
    width: u32,
    height: u32,
    id: FigureId,
) -> Result<(), PublicReportError> {
    let mut image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, WHITE);
    draw_line(
        &mut image,
        120,
        height as i32 - 130,
        width as i32 - 100,
        height as i32 - 130,
        BLACK,
    );
    draw_line(&mut image, 120, 190, 120, height as i32 - 130, BLACK);
    for (series_index, series) in payload.series.iter().enumerate() {
        let mut last = None;
        for point in &series.points {
            let (x, y) = point_position(point, series_index, width, height);
            if let Some((last_x, last_y)) = last {
                draw_line(&mut image, last_x, last_y, x, y, series.colour);
            }
            draw_disc(&mut image, x, y, series.colour);
            last = Some((x, y));
        }
    }
    image
        .save(path)
        .map_err(|error| PublicReportError::PlotBackend {
            figure_id: id.as_str().into(),
            path: path.to_path_buf(),
            message: error.to_string(),
        })
}

fn point_position(point: &Point, series_index: usize, width: u32, height: u32) -> (i32, i32) {
    let usable_width = (width as i32 - 260).max(1);
    let usable_height = (height as i32 - 380).max(1);
    let x = 140 + ((point.x.abs() as u64 % usable_width as u64) as i32);
    let y = point
        .y
        .map(|value| 220 + ((value.abs() * 1000.0) as u64 % usable_height as u64) as i32)
        .unwrap_or(200 + (series_index as i32 * 12));
    (x, y)
}
fn draw_disc(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: i32, y: i32, colour: Rgb<u8>) {
    for dx in -3..=3 {
        for dy in -3..=3 {
            if dx * dx + dy * dy <= 9
                && x + dx >= 0
                && y + dy >= 0
                && (x + dx) < image.width() as i32
                && (y + dy) < image.height() as i32
            {
                image.put_pixel((x + dx) as u32, (y + dy) as u32, colour);
            }
        }
    }
}
fn draw_line(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    colour: Rgb<u8>,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 && x0 < image.width() as i32 && y0 < image.height() as i32 {
            image.put_pixel(x0 as u32, y0 as u32, colour);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x0 += sx;
        }
        if twice <= dx {
            error += dx;
            y0 += sy;
        }
    }
}
fn colour_hex(colour: Rgb<u8>) -> String {
    format!("#{:02x}{:02x}{:02x}", colour[0], colour[1], colour[2])
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
