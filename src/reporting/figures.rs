//! Artifact-only Phase-D scientific figure dispatch.
//!
//! The renderer accepts only coordinates and categorical states copied from
//! canonical artifacts. Missing values are disclosed as text and never receive
//! a plotting coordinate. Logarithms are used only by display-axis geometry.

use crate::{
    domain::{ArtifactDependencyRole, ArtifactLineageState},
    report_config::FigureId,
    reporting::{
        AvailabilityReason, PublicReportError, projection::PublicReportProjection,
        tables::format_public_f64,
    },
    results::FeatureComparability,
};
use plotters::{coord::Shift, prelude::*};
use std::{collections::BTreeMap, fs, path::Path};

const WIDTH: u32 = 1600;
const STANDARD_HEIGHT: u32 = 1000;
const MULTIPANEL_HEIGHT: u32 = 1400;
const SVG_CONTENT_TOP: f64 = 155.0;

#[derive(Clone, Copy)]
enum AxisScale {
    Linear,
    Log10,
}

#[derive(Clone)]
struct NumericDatum {
    x: f64,
    y: Option<f64>,
    x_text: String,
    y_text: String,
}

#[derive(Clone)]
struct NumericSeries {
    label: String,
    data: Vec<NumericDatum>,
    colour: RGBColor,
    connect: bool,
}

#[derive(Clone)]
struct NumericPanel {
    title: String,
    x_label: String,
    y_label: String,
    x_scale: AxisScale,
    series: Vec<NumericSeries>,
    notes: Vec<String>,
}

#[derive(Clone)]
struct CategoryDatum {
    category: String,
    y: f64,
    y_text: String,
    annotation: String,
}

#[derive(Clone)]
struct CategorySeries {
    label: String,
    data: Vec<CategoryDatum>,
    colour: RGBColor,
}

#[derive(Clone)]
struct CategoryPanel {
    title: String,
    x_label: String,
    y_label: String,
    categories: Vec<String>,
    series: Vec<CategorySeries>,
    notes: Vec<String>,
}

#[derive(Clone)]
struct HealthRow {
    dimension: String,
    status: String,
    evidence_state: String,
    reason_count: usize,
}

#[derive(Clone)]
struct LineageDependency {
    role: String,
    kind: String,
    artifact_id: String,
}

#[derive(Clone)]
struct LineageRoot {
    flag: String,
    label: String,
    catalog_membership: String,
    dependencies: Vec<LineageDependency>,
}

enum FigureBody {
    Numeric(Vec<NumericPanel>),
    Category(Vec<CategoryPanel>),
    HealthGrid(Vec<HealthRow>),
    Lineage(Vec<LineageRoot>),
}

struct FigurePayload {
    title: &'static str,
    caption: String,
    body: FigureBody,
}

pub(crate) fn write_figure(
    root: &Path,
    id: FigureId,
    projection: &PublicReportProjection<'_>,
) -> Result<[String; 2], PublicReportError> {
    let directory = root.join("figures");
    fs::create_dir_all(&directory).map_err(|source| PublicReportError::Write {
        path: directory.clone(),
        source,
    })?;
    let svg_path = directory.join(format!("{}.svg", id.as_str()));
    let png_path = directory.join(format!("{}.png", id.as_str()));
    let payload = payload(id, projection)?;
    let height = match &payload.body {
        FigureBody::Numeric(panels) if panels.len() > 1 => MULTIPANEL_HEIGHT,
        FigureBody::Category(panels) if panels.len() > 1 => MULTIPANEL_HEIGHT,
        _ => STANDARD_HEIGHT,
    };
    fs::write(&svg_path, svg_document(id, &payload, WIDTH, height)).map_err(|source| {
        PublicReportError::Write {
            path: svg_path.clone(),
            source,
        }
    })?;
    write_png(&png_path, &payload, WIDTH, height, id)?;
    Ok([
        format!("figures/{}.svg", id.as_str()),
        format!("figures/{}.png", id.as_str()),
    ])
}

pub(crate) fn figure_reason(
    projection: &PublicReportProjection<'_>,
    id: FigureId,
) -> Option<AvailabilityReason> {
    projection.figure_reason(id)
}

fn payload(
    id: FigureId,
    projection: &PublicReportProjection<'_>,
) -> Result<FigurePayload, PublicReportError> {
    match id {
        FigureId::MechanismTimescale => mechanism_payload(projection),
        FigureId::SensorHealthDimensionStatus => health_payload(projection),
        FigureId::CurrentVsBaseline => baseline_payload(projection),
        FigureId::EisNyquist => eis_nyquist_payload(projection),
        FigureId::EisBode => eis_bode_payload(projection),
        FigureId::TransientResponse => transient_payload(projection),
        FigureId::CalibrationPerformance => calibration_payload(projection),
        FigureId::SignalDiagnostics => signal_payload(projection),
        FigureId::EstimationObservedPredicted => estimation_payload(projection),
        FigureId::ModelObservedPredicted => model_payload(projection),
        FigureId::Lineage => lineage_payload(projection),
    }
}

fn mechanism_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let mut rows = p
        .mechanism
        .comparisons
        .iter()
        .filter(|row| row.log10_distance.is_some_and(f64::is_finite))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.comparison_id.cmp(&right.comparison_id));
    let categories = rows
        .iter()
        .map(|row| row.comparison_id.clone())
        .collect::<Vec<_>>();
    let mut grouped: BTreeMap<String, Vec<CategoryDatum>> = BTreeMap::new();
    for row in rows {
        let value = row.log10_distance.expect("finite rows selected");
        grouped
            .entry(token(&row.evidence_level))
            .or_default()
            .push(CategoryDatum {
                category: row.comparison_id.clone(),
                y: value,
                y_text: format_public_f64(value).map_err(staging_number_error)?,
                annotation: format!(
                    "{}: {} / {}; evidence={}; warnings={}",
                    row.comparison_id,
                    row.eis_timescale_id,
                    row.transient_timescale_id,
                    token(&row.evidence_level),
                    row.warnings.len()
                ),
            });
    }
    let colours = [BLUE, RED, GREEN, MAGENTA, CYAN, BLACK];
    let series = grouped
        .into_iter()
        .enumerate()
        .map(|(index, (label, data))| CategorySeries {
            label,
            data,
            colour: colours[index % colours.len()],
        })
        .collect();
    Ok(FigurePayload {
        title: "Mechanism timescale comparison",
        caption: format!(
            "Mechanism analysis {} from source artifact {}. Values are serialized log10_distance fields in dimensionless units; Phase D performs no log10 calculation and no thresholding. Producer warnings: {}.",
            p.mechanism.analysis_id,
            source_identity_label(&p.mechanism.lineage),
            p.mechanism.warnings.len(),
        ),
        body: FigureBody::Category(vec![CategoryPanel {
            title: "Stored timescale comparison distance".into(),
            x_label: "Serialized comparison ID".into(),
            y_label: "Stored log10 distance [dimensionless]".into(),
            categories,
            series,
            notes: std::iter::once(
                "Direct labels retain serialized EIS/transient IDs and evidence level.".into(),
            )
            .chain(
                p.mechanism
                    .warnings
                    .iter()
                    .map(|warning| format!("producer warning: {}", warning.message)),
            )
            .collect(),
        }]),
    })
}

fn health_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let rows = p
        .health
        .phase_c
        .as_ref()
        .map(|phase| {
            crate::results::HealthDimension::ALL
                .iter()
                .filter_map(|dimension| {
                    phase
                        .dimension_assessments
                        .iter()
                        .find(|item| item.dimension == *dimension)
                })
                .map(|item| HealthRow {
                    dimension: token(&item.dimension),
                    status: token(&item.status),
                    evidence_state: token(&item.evidence_state),
                    reason_count: item.reason_codes.len(),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(FigurePayload {
        title: "Sensor-health dimension status",
        caption: format!(
            "Health assessment {} from source artifact {}. The categorical grid has no numeric unit and shows all nine serialized Phase-C dimensions. Data quality insufficient (DQI) and Indeterminate remain explicit producer states. Producer warnings: {}.",
            p.health.assessment_id,
            source_identity_label(&p.health.lineage),
            p.health.warnings.len(),
        ),
        body: FigureBody::HealthGrid(rows),
    })
}

fn baseline_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let mut by_unit: BTreeMap<String, Vec<&crate::results::BaselineComparison>> = BTreeMap::new();
    let mut warnings = Vec::new();
    for comparison in &p.health.baseline_comparison {
        if !matches!(
            comparison.comparability,
            FeatureComparability::Comparable | FeatureComparability::ComparableWithWarnings
        ) || !comparison.current_value.is_some_and(f64::is_finite)
            || !comparison.baseline_value.is_some_and(f64::is_finite)
        {
            continue;
        }
        let units = p
            .health
            .features
            .iter()
            .filter(|feature| feature.name == comparison.feature && !feature.unit.is_empty())
            .map(|feature| feature.unit.as_str())
            .collect::<Vec<_>>();
        if units.len() != 1 {
            continue;
        }
        by_unit
            .entry(units[0].to_owned())
            .or_default()
            .push(comparison);
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
    let mut panels = Vec::new();
    for (unit, mut rows) in by_unit {
        rows.sort_by(|left, right| left.feature.cmp(&right.feature));
        let categories = rows
            .iter()
            .map(|row| row.feature.clone())
            .collect::<Vec<_>>();
        let current = rows
            .iter()
            .map(|row| {
                category_value(
                    row.feature.clone(),
                    row.current_value.expect("validated"),
                    token(&row.comparability),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let baseline = rows
            .iter()
            .map(|row| {
                category_value(
                    row.feature.clone(),
                    row.baseline_value.expect("validated"),
                    "serialized baseline".into(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        panels.push(CategoryPanel {
            title: format!("Current versus baseline [{unit}]"),
            x_label: "Serialized health feature".into(),
            y_label: format!("Serialized value [{unit}]"),
            categories,
            series: vec![
                CategorySeries {
                    label: "current".into(),
                    data: current,
                    colour: BLUE,
                },
                CategorySeries {
                    label: "baseline".into(),
                    data: baseline,
                    colour: RED,
                },
            ],
            notes: warnings.clone(),
        });
    }
    Ok(FigurePayload {
        title: "Current versus baseline",
        caption: format!(
            "Health assessment {} from source artifact {}. Units come only from the unique serialized HealthFeature match. ComparableWithWarnings pairs are rendered without conversion and disclosed: {}. Producer warnings: {}.",
            p.health.assessment_id,
            source_identity_label(&p.health.lineage),
            if warnings.is_empty() {
                "none".into()
            } else {
                warnings.join("; ")
            },
            p.health.warnings.len(),
        ),
        body: FigureBody::Category(panels),
    })
}

fn category_value(
    category: String,
    value: f64,
    annotation: String,
) -> Result<CategoryDatum, PublicReportError> {
    Ok(CategoryDatum {
        category,
        y: value,
        y_text: format_public_f64(value).map_err(staging_number_error)?,
        annotation,
    })
}

fn eis_nyquist_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let eis = p.eis.expect("availability checked before rendering");
    Ok(FigurePayload {
        title: "EIS Nyquist",
        caption: "Imaginary impedance is plotted with its serialized sign; Phase D performs no Nyquist sign transform.".into(),
        body: FigureBody::Numeric(vec![NumericPanel {
            title: format!(
                "EIS fit {} / source {}",
                eis.fit_id,
                source_identity_label(&eis.lineage)
            ),
            x_label: "Re(Z) [Ohm]".into(),
            y_label: "Im(Z) [Ohm]".into(),
            x_scale: AxisScale::Linear,
            series: vec![
                numeric_series(
                    "observed",
                    &eis.source.z_real_ohm,
                    &eis.source.z_imag_ohm,
                    BLUE,
                    true,
                )?,
                numeric_series(
                    "fitted",
                    &eis.fitted.z_real_ohm,
                    &eis.fitted.z_imag_ohm,
                    RED,
                    true,
                )?,
            ],
            notes: eis_warning_notes(eis),
        }]),
    })
}

fn eis_bode_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let eis = p.eis.expect("availability checked before rendering");
    let magnitude_source = if eis.source.source_measured_magnitude_ohm.is_some() {
        "source_measured_magnitude_ohm"
    } else {
        "derived_magnitude_ohm"
    };
    let phase_source = if eis.source.source_measured_phase_deg.is_some() {
        "source_measured_phase_deg"
    } else {
        "derived_phase_deg"
    };
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
        caption: format!(
            "EIS fit {} from source artifact {}. Frequency is in Hz, magnitude in Ohm, and phase in degrees. Magnitude and phase occupy separate panels. Observed magnitude uses serialized {magnitude_source}; observed phase uses serialized {phase_source}; fitted channels are producer-model outputs. Non-positive frequencies receive no coordinate on the log display axis. Phase D performs no sqrt, atan, or atan2 calculation. Producer warnings: {}.",
            eis.fit_id,
            source_identity_label(&eis.lineage),
            eis.warnings.len(),
        ),
        body: FigureBody::Numeric(vec![
            NumericPanel {
                title: "Magnitude".into(),
                x_label: "Frequency [Hz] (log display axis)".into(),
                y_label: "Magnitude [Ohm]".into(),
                x_scale: AxisScale::Log10,
                series: vec![
                    optional_numeric_series(
                        "observed magnitude",
                        &eis.source.frequency_hz,
                        &source_magnitude,
                        BLUE,
                        true,
                    )?,
                    numeric_series(
                        "fitted magnitude",
                        &eis.source.frequency_hz,
                        &eis.fitted.magnitude_ohm,
                        RED,
                        true,
                    )?,
                ],
                notes: missing_notes(
                    "observed magnitude",
                    &eis.source.frequency_hz,
                    &source_magnitude,
                )?
                .into_iter()
                .chain(log_axis_exclusion_notes(
                    "Bode magnitude",
                    &eis.source.frequency_hz,
                ))
                .chain(eis_warning_notes(eis))
                .collect(),
            },
            NumericPanel {
                title: "Phase".into(),
                x_label: "Frequency [Hz] (log display axis)".into(),
                y_label: "Phase [deg]".into(),
                x_scale: AxisScale::Log10,
                series: vec![
                    optional_numeric_series(
                        "observed phase",
                        &eis.source.frequency_hz,
                        &source_phase,
                        GREEN,
                        true,
                    )?,
                    numeric_series(
                        "fitted phase",
                        &eis.source.frequency_hz,
                        &eis.fitted.phase_deg,
                        MAGENTA,
                        true,
                    )?,
                ],
                notes: missing_notes("observed phase", &eis.source.frequency_hz, &source_phase)?
                    .into_iter()
                    .chain(log_axis_exclusion_notes(
                        "Bode phase",
                        &eis.source.frequency_hz,
                    ))
                    .chain(eis_warning_notes(eis))
                    .collect(),
            },
        ]),
    })
}

fn transient_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let transient = p.transient.expect("availability checked before rendering");
    let mut potential_series = Vec::new();
    let mut residual_series = Vec::new();
    let mut potential_notes = transient_unpaired_notes(transient, true);
    let mut residual_notes = transient_unpaired_notes(transient, false);
    let producer_warnings = transient
        .events
        .iter()
        .flat_map(|event| {
            event.warnings.iter().chain(
                event
                    .candidate_fits
                    .iter()
                    .flat_map(|fit| fit.warnings.iter()),
            )
        })
        .map(|warning| format!("producer warning: {}", warning.message))
        .collect::<Vec<_>>();
    potential_notes.extend(producer_warnings.iter().cloned());
    residual_notes.extend(producer_warnings);
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
        if event.segment.raw_time_local.len() != event.segment.raw_potential_v.len()
            || event.segment.fitted_time_local.is_empty()
            || fit.predicted_v.is_empty()
            || fit.residuals_v.is_empty()
        {
            let note = format!(
                "event {}: unique selected fit has invalid serialized coordinate/value arrays and is not plotted",
                event.event_index
            );
            potential_notes.push(note.clone());
            residual_notes.push(note);
            continue;
        }
        potential_series.push(NumericSeries {
            label: "observed".into(),
            data: optional_data(
                &event.segment.raw_time_local,
                &event.segment.raw_potential_v,
            )?,
            colour: BLUE,
            connect: true,
        });
        potential_series.push(NumericSeries {
            label: "fitted".into(),
            data: serialized_pairs(&event.segment.fitted_time_local, &fit.predicted_v)?,
            colour: RED,
            connect: true,
        });
        residual_series.push(NumericSeries {
            label: "residual".into(),
            data: serialized_pairs(&event.segment.fitted_time_local, &fit.residuals_v)?,
            colour: MAGENTA,
            connect: true,
        });
    }
    Ok(FigurePayload {
        title: "Transient selected-fit response",
        caption: format!(
            "Transient experiment {} from source artifact {}. Time is in seconds and potential/residual are in volts. Observations use segment.raw_time_local; the selected candidate's serialized prediction and residual use segment.fitted_time_local. Events are separate series. Phase D does not fit, evaluate, rank, or choose first. Producer warnings: {}.",
            transient.experiment_id,
            source_identity_label(&transient.lineage),
            transient
                .events
                .iter()
                .map(|event| event.warnings.len()
                    + event
                        .candidate_fits
                        .iter()
                        .map(|fit| fit.warnings.len())
                        .sum::<usize>())
                .sum::<usize>(),
        ),
        body: FigureBody::Numeric(vec![
            NumericPanel {
                title: "Observed and selected fitted potential".into(),
                x_label: "Serialized local time [s]".into(),
                y_label: "Potential [V]".into(),
                x_scale: AxisScale::Linear,
                series: potential_series,
                notes: potential_notes,
            },
            NumericPanel {
                title: "Serialized residual".into(),
                x_label: "Serialized fitted local time [s]".into(),
                y_label: "Residual [V]".into(),
                x_scale: AxisScale::Linear,
                series: residual_series,
                notes: residual_notes,
            },
        ]),
    })
}

fn calibration_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let calibration = p
        .calibration
        .expect("availability checked before rendering");
    let predictions = &calibration
        .validation
        .as_ref()
        .expect("availability checked")
        .predictions;
    let mut observed = Vec::new();
    let mut predicted = Vec::new();
    let mut notes = calibration
        .warnings
        .iter()
        .map(|warning| format!("producer warning: {}", warning.message))
        .collect::<Vec<_>>();
    for row in predictions {
        let Some(x) = row.observed_log10_activity else {
            notes.push(format!(
                "{}: observed_log10_activity=NA",
                row.observation_id
            ));
            continue;
        };
        observed.push(datum(x, Some(row.observed_potential_v))?);
        predicted.push(datum(x, row.predicted_potential_v)?);
        if row.predicted_potential_v.is_none() {
            notes.push(format!("{}: predicted_potential_v=NA", row.observation_id));
        }
    }
    Ok(FigurePayload {
        title: "Calibration performance",
        caption: format!(
            "Calibration analysis {} from source artifact {}. The serialized x coordinate is log10 activity and potential is in volts. Only validation observation/prediction points are plotted; Phase D draws no theoretical calibration curve and computes no residual. Producer warnings: {}.",
            calibration.calibration_id,
            source_identity_label(&calibration.lineage),
            calibration.warnings.len(),
        ),
        body: FigureBody::Numeric(vec![
            NumericPanel {
                title: "Validation potential".into(),
                x_label: "Serialized observed log10 activity".into(),
                y_label: "Potential [V]".into(),
                x_scale: AxisScale::Linear,
                series: vec![
                    NumericSeries {
                        label: "observed".into(),
                        data: observed,
                        colour: BLUE,
                        connect: false,
                    },
                    NumericSeries {
                        label: "predicted".into(),
                        data: predicted,
                        colour: RED,
                        connect: false,
                    },
                ],
                notes,
            },
            NumericPanel {
                title: "Residual availability".into(),
                x_label: "No serialized coordinate".into(),
                y_label: "Residual [V] unavailable".into(),
                x_scale: AxisScale::Linear,
                series: Vec::new(),
                notes: vec![
                    "ValidationPredictionPoint does not serialize a residual; Phase D does not recompute one."
                        .into(),
                ],
            },
        ]),
    })
}

fn signal_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let signal = p.signal.expect("availability checked before rendering");
    let (time_data, mut time_notes) = if signal.analysis_timestamps.len()
        == signal.analysis_values.len()
        && signal
            .analysis_timestamps
            .iter()
            .zip(&signal.analysis_values)
            .any(|(x, y)| x.is_finite() && y.is_some_and(f64::is_finite))
    {
        (
            optional_data(&signal.analysis_timestamps, &signal.analysis_values)?,
            missing_notes(
                "signal",
                &signal.analysis_timestamps,
                &signal.analysis_values,
            )?,
        )
    } else {
        (
            Vec::new(),
            vec!["Time-domain signal unavailable in the serialized artifact.".into()],
        )
    };
    time_notes.extend(
        signal
            .warnings
            .iter()
            .map(|warning| format!("producer warning: {}", token(warning))),
    );
    let (psd_data, psd_notes, psd_unit) = match &signal.psd {
        Some(psd)
            if psd.frequency_hz.len() == psd.psd.len()
                && psd
                    .frequency_hz
                    .iter()
                    .zip(&psd.psd)
                    .any(|(x, y)| x.is_finite() && *x > 0.0 && y.is_finite()) =>
        {
            (
                required_data(&psd.frequency_hz, &psd.psd)?,
                log_axis_exclusion_notes("PSD", &psd.frequency_hz),
                psd.psd_unit.clone(),
            )
        }
        _ => (
            Vec::new(),
            vec!["PSD unavailable in the serialized artifact.".into()],
            "unavailable".into(),
        ),
    };
    let (allan_data, allan_notes) = match &signal.allan {
        Some(allan)
            if allan.points.iter().any(|point| {
                point.averaging_time_s.is_finite()
                    && point.averaging_time_s > 0.0
                    && point.deviation.is_some_and(f64::is_finite)
            }) =>
        {
            let x = allan
                .points
                .iter()
                .map(|point| point.averaging_time_s)
                .collect::<Vec<_>>();
            let y = allan
                .points
                .iter()
                .map(|point| point.deviation)
                .collect::<Vec<_>>();
            let notes = missing_notes("Allan deviation", &x, &y)?
                .into_iter()
                .chain(log_axis_exclusion_notes("Allan deviation", &x))
                .collect();
            (optional_data(&x, &y)?, notes)
        }
        _ => (
            Vec::new(),
            vec!["Allan deviation unavailable in the serialized artifact.".into()],
        ),
    };
    Ok(FigurePayload {
        title: "Signal diagnostics",
        caption: format!(
            "Signal analysis {} from source artifact {}. Time is in seconds, the signal and Allan deviation use {}, and PSD uses its serialized unit. Time, PSD, and Allan domains occupy separate axes and use serialized values only. Missing samples are disclosed without artificial coordinates; Phase D performs no resampling, PSD, or Allan calculation. Producer warnings: {}.",
            signal.analysis_id,
            source_identity_label(&signal.lineage),
            signal.unit,
            signal.warnings.len(),
        ),
        body: FigureBody::Numeric(vec![
            NumericPanel {
                title: "Retained analysis signal".into(),
                x_label: "Time [s]".into(),
                y_label: format!("Signal [{}]", signal.unit),
                x_scale: AxisScale::Linear,
                series: vec![NumericSeries {
                    label: "time signal".into(),
                    data: time_data,
                    colour: BLUE,
                    connect: true,
                }],
                notes: time_notes,
            },
            NumericPanel {
                title: "Power spectral density".into(),
                x_label: "Frequency [Hz] (log display axis)".into(),
                y_label: format!("PSD [{psd_unit}]"),
                x_scale: AxisScale::Log10,
                series: vec![NumericSeries {
                    label: "PSD".into(),
                    data: psd_data,
                    colour: RED,
                    connect: true,
                }],
                notes: psd_notes,
            },
            NumericPanel {
                title: "Allan deviation".into(),
                x_label: "Averaging time [s] (log display axis)".into(),
                y_label: format!("Deviation [{}]", signal.unit),
                x_scale: AxisScale::Log10,
                series: vec![NumericSeries {
                    label: "Allan".into(),
                    data: allan_data,
                    colour: GREEN,
                    connect: true,
                }],
                notes: allan_notes,
            },
        ]),
    })
}

fn estimation_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let estimation = p.estimation.expect("availability checked before rendering");
    let timestamps = estimation
        .estimates
        .iter()
        .map(|point| point.timestamp_s)
        .collect::<Vec<_>>();
    let observed = estimation
        .estimates
        .iter()
        .map(|point| point.measurement_v)
        .collect::<Vec<_>>();
    let predicted = estimation
        .estimates
        .iter()
        .map(|point| point.predicted_measurement_v)
        .collect::<Vec<_>>();
    let mut notes = missing_notes("observed", &timestamps, &observed)?;
    notes.extend(missing_notes("predicted", &timestamps, &predicted)?);
    for point in &estimation.estimates {
        notes.push(format!(
            "t={}: update_status={}; applied_measurement_variance_v2={}",
            format_public_f64(point.timestamp_s).map_err(staging_number_error)?,
            token(&point.update_status),
            number_text(point.applied_measurement_variance_v2)?
        ));
    }
    notes.extend(
        estimation
            .warnings
            .iter()
            .map(|warning| format!("producer warning: {}", token(warning))),
    );
    let mut by_segment = BTreeMap::<usize, (Vec<f64>, Vec<Option<f64>>, Vec<Option<f64>>)>::new();
    for point in &estimation.estimates {
        let (segment_time, segment_observed, segment_predicted) =
            by_segment.entry(point.segment_id).or_default();
        segment_time.push(point.timestamp_s);
        segment_observed.push(point.measurement_v);
        segment_predicted.push(point.predicted_measurement_v);
    }
    let mut series = Vec::new();
    for (segment_id, (segment_time, segment_observed, segment_predicted)) in by_segment {
        series.push(NumericSeries {
            label: format!("observed segment {segment_id}"),
            data: optional_data(&segment_time, &segment_observed)?,
            colour: BLUE,
            connect: true,
        });
        series.push(NumericSeries {
            label: format!("predicted segment {segment_id}"),
            data: optional_data(&segment_time, &segment_predicted)?,
            colour: RED,
            connect: true,
        });
    }
    Ok(FigurePayload {
        title: "Estimation observed versus predicted",
        caption: format!(
            "State-estimation analysis {} from source artifact {}. Time is in seconds and measurement/prediction are serialized potential values in volts; distinct serialized segment_id series are never connected. Variance availability is labelled only; no potential uncertainty interval is invented. Producer warnings: {}.",
            estimation.analysis_id,
            source_identity_label(&estimation.lineage),
            estimation.warnings.len(),
        ),
        body: FigureBody::Numeric(vec![NumericPanel {
            title: "Measured and predicted potential".into(),
            x_label: "Timestamp [s]".into(),
            y_label: "Potential [V]".into(),
            x_scale: AxisScale::Linear,
            series,
            notes,
        }]),
    })
}

fn model_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let model = p.model.expect("availability checked before rendering");
    let time = model
        .points
        .iter()
        .map(|point| point.time_s)
        .collect::<Vec<_>>();
    let observed = model
        .points
        .iter()
        .map(|point| point.observed_voltage_v)
        .collect::<Vec<_>>();
    let predicted = model
        .points
        .iter()
        .map(|point| Some(point.predicted_voltage_v))
        .collect::<Vec<_>>();
    let residual = model
        .points
        .iter()
        .map(|point| point.unexplained_residual_v)
        .collect::<Vec<_>>();
    let mut potential_notes = missing_notes("observed", &time, &observed)?;
    for point in &model.points {
        potential_notes.push(format!(
            "t={}: validity={}; equilibrium={}; uncertainty={}",
            format_public_f64(point.time_s).map_err(staging_number_error)?,
            if point.validity.is_valid {
                "valid"
            } else {
                "invalid"
            },
            token(&point.equilibrium.status),
            token(&point.uncertainty.status)
        ));
    }
    Ok(FigurePayload {
        title: "Model observed versus predicted",
        caption: format!(
            "Model source artifact {}. Time is in seconds and potential/residual are in volts. Observed, predicted, and unexplained residual values are copied from serialized model points. Missing observed or residual values remain NA without plotting coordinates; Phase D never recomputes residuals or maps missing values to zero.",
            source_identity_label(&model.lineage)
        ),
        body: FigureBody::Numeric(vec![
            NumericPanel {
                title: "Observed and predicted potential".into(),
                x_label: "Time [s]".into(),
                y_label: "Potential [V]".into(),
                x_scale: AxisScale::Linear,
                series: vec![
                    optional_numeric_series("observed", &time, &observed, BLUE, true)?,
                    optional_numeric_series("predicted", &time, &predicted, RED, true)?,
                ],
                notes: potential_notes,
            },
            NumericPanel {
                title: "Serialized unexplained residual".into(),
                x_label: "Time [s]".into(),
                y_label: "Residual [V]".into(),
                x_scale: AxisScale::Linear,
                series: vec![optional_numeric_series(
                    "residual", &time, &residual, MAGENTA, true,
                )?],
                notes: missing_notes("residual", &time, &residual)?,
            },
        ]),
    })
}

fn lineage_payload(p: &PublicReportProjection<'_>) -> Result<FigurePayload, PublicReportError> {
    let roots = p
        .supplied_lineages()
        .into_iter()
        .map(|(flag, lineage)| match lineage {
            ArtifactLineageState::Known {
                identity,
                direct_dependencies,
            } => {
                let membership =
                    p.lineage_catalog
                        .map_or("catalog_not_supplied".into(), |catalog| {
                            if catalog.artifacts.contains_key(&identity.artifact_id) {
                                "catalog_member".into()
                            } else {
                                "not_catalog_member".into()
                            }
                        });
                LineageRoot {
                    flag: flag.into(),
                    label: format!(
                        "{} / {} / schema {}",
                        identity.artifact_kind.as_str(),
                        identity.artifact_id.0,
                        identity.schema_version
                    ),
                    catalog_membership: membership,
                    dependencies: direct_dependencies
                        .iter()
                        .map(|dependency| LineageDependency {
                            role: dependency_role(&dependency.role),
                            kind: dependency.artifact_kind.as_str().into(),
                            artifact_id: dependency.artifact_id.0.clone(),
                        })
                        .collect(),
                }
            }
            ArtifactLineageState::LegacyUnknown {
                source_schema_version,
                reason,
            } => LineageRoot {
                flag: flag.into(),
                label: format!(
                    "LegacyUnknown / schema {} / {}",
                    source_schema_version.map_or("NA".into(), |value| value.to_string()),
                    token(reason)
                ),
                catalog_membership: "NA".into(),
                dependencies: Vec::new(),
            },
        })
        .collect();
    Ok(FigurePayload {
        title: "Artifact lineage and provenance",
        caption: "Only supplied roots and their serialized direct dependency edges are shown. Catalog membership is a label, not traversal, resolution, ancestry, or scientific evidence.".into(),
        body: FigureBody::Lineage(roots),
    })
}

fn dependency_role(role: &ArtifactDependencyRole) -> String {
    token(role)
}

fn source_identity_label(lineage: &ArtifactLineageState) -> String {
    match lineage {
        ArtifactLineageState::Known { identity, .. } => identity.artifact_id.0.clone(),
        ArtifactLineageState::LegacyUnknown {
            source_schema_version,
            reason,
        } => format!(
            "LegacyUnknown(schema={}, reason={})",
            source_schema_version.map_or_else(|| "NA".into(), |value| value.to_string()),
            token(reason)
        ),
    }
}

fn eis_warning_notes(eis: &crate::results::EisFitArtifact) -> Vec<String> {
    let mut notes = eis
        .warnings
        .iter()
        .map(|warning| format!("producer warning: {}", warning.message))
        .collect::<Vec<_>>();
    notes.extend(
        eis.diagnostics
            .parameter_at_bound
            .iter()
            .map(|parameter| format!("parameter at bound: {parameter}")),
    );
    if eis.diagnostics.non_identifiable {
        notes.push("producer diagnostic: fit is non-identifiable".into());
    }
    notes
}

fn numeric_series(
    label: &str,
    x: &[f64],
    y: &[f64],
    colour: RGBColor,
    connect: bool,
) -> Result<NumericSeries, PublicReportError> {
    Ok(NumericSeries {
        label: label.into(),
        data: required_data(x, y)?,
        colour,
        connect,
    })
}

fn optional_numeric_series(
    label: &str,
    x: &[f64],
    y: &[Option<f64>],
    colour: RGBColor,
    connect: bool,
) -> Result<NumericSeries, PublicReportError> {
    Ok(NumericSeries {
        label: label.into(),
        data: optional_data(x, y)?,
        colour,
        connect,
    })
}

fn required_data(x: &[f64], y: &[f64]) -> Result<Vec<NumericDatum>, PublicReportError> {
    if x.len() != y.len() {
        return Err(series_length_error());
    }
    x.iter().zip(y).map(|(&x, &y)| datum(x, Some(y))).collect()
}

fn serialized_pairs(x: &[f64], y: &[f64]) -> Result<Vec<NumericDatum>, PublicReportError> {
    x.iter().zip(y).map(|(&x, &y)| datum(x, Some(y))).collect()
}

fn transient_unpaired_notes(
    transient: &crate::results::TransientAnalysisReport,
    prediction: bool,
) -> Vec<String> {
    let field = if prediction {
        "predicted_v"
    } else {
        "residuals_v"
    };
    transient
        .events
        .iter()
        .filter_map(|event| {
            let model = event.selected_model.as_ref()?;
            let matches = event
                .candidate_fits
                .iter()
                .filter(|fit| fit.is_successful() && &fit.model == model)
                .collect::<Vec<_>>();
            let [fit] = matches.as_slice() else {
                return None;
            };
            let value_count = if prediction {
                fit.predicted_v.len()
            } else {
                fit.residuals_v.len()
            };
            (value_count != event.segment.fitted_time_local.len()).then(|| {
                format!(
                    "event {}: {field} has {value_count} serialized values but fitted_time_local has {} coordinates; only serialized coordinate/value pairs are plotted",
                    event.event_index,
                    event.segment.fitted_time_local.len()
                )
            })
        })
        .collect()
}

fn optional_data(x: &[f64], y: &[Option<f64>]) -> Result<Vec<NumericDatum>, PublicReportError> {
    if x.len() != y.len() {
        return Err(series_length_error());
    }
    x.iter().zip(y).map(|(&x, &y)| datum(x, y)).collect()
}

fn datum(x: f64, y: Option<f64>) -> Result<NumericDatum, PublicReportError> {
    if !x.is_finite() || y.is_some_and(|value| !value.is_finite()) {
        return Err(staging_number_error(
            "non-finite number in public projection",
        ));
    }
    Ok(NumericDatum {
        x,
        y,
        x_text: format_public_f64(x).map_err(staging_number_error)?,
        y_text: number_text(y)?,
    })
}

fn missing_notes(
    label: &str,
    x: &[f64],
    y: &[Option<f64>],
) -> Result<Vec<String>, PublicReportError> {
    if x.len() != y.len() {
        return Err(series_length_error());
    }
    x.iter()
        .zip(y)
        .filter(|(_, y)| y.is_none())
        .map(|(&x, _)| {
            format_public_f64(x).map(|x| {
                format!("{label}: NA at serialized x={x}; no plotting coordinate assigned")
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(staging_number_error)
}

fn log_axis_exclusion_notes(label: &str, x: &[f64]) -> Vec<String> {
    let count = x
        .iter()
        .filter(|value| !value.is_finite() || **value <= 0.0)
        .count();
    if count == 0 {
        Vec::new()
    } else {
        vec![format!(
            "{label}: {count} serialized non-positive or invalid x value(s) receive no plotting coordinate on the log display axis"
        )]
    }
}

fn number_text(value: Option<f64>) -> Result<String, PublicReportError> {
    value
        .map(format_public_f64)
        .transpose()
        .map_err(staging_number_error)
        .map(|value| value.unwrap_or_else(|| "NA".into()))
}

fn series_length_error() -> PublicReportError {
    staging_number_error("mismatched serialized coordinate lengths")
}

fn staging_number_error(detail: impl Into<String>) -> PublicReportError {
    PublicReportError::StagingValidation {
        path: Path::new("public report projection").to_path_buf(),
        detail: detail.into(),
    }
}

fn token<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .expect("closed enum serialization")
        .trim_matches('"')
        .to_owned()
}

fn svg_document(id: FigureId, payload: &FigurePayload, width: u32, height: u32) -> String {
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\"><title>{}</title><desc>{}</desc><metadata>phase_d_figure={};threshold_lines=0;missing_values_have_no_plot_coordinates=true</metadata><rect width=\"100%\" height=\"100%\" fill=\"white\"/><text x=\"60\" y=\"52\" font-family=\"sans-serif\" font-size=\"32\" font-weight=\"bold\">{}</text>",
        escape(payload.title),
        escape(&payload.caption),
        id.as_str(),
        escape(payload.title)
    );
    for (index, line) in wrap_text(&payload.caption, 185).iter().enumerate() {
        svg.push_str(&format!(
            "<text data-caption-line=\"{}\" x=\"60\" y=\"{}\" font-family=\"sans-serif\" font-size=\"15\">{}</text>",
            index,
            80 + index * 18,
            escape(line)
        ));
    }
    match &payload.body {
        FigureBody::Numeric(panels) => svg_numeric_panels(&mut svg, panels, width, height),
        FigureBody::Category(panels) => svg_category_panels(&mut svg, panels, width, height),
        FigureBody::HealthGrid(rows) => svg_health_grid(&mut svg, rows, width),
        FigureBody::Lineage(roots) => svg_lineage(&mut svg, roots, width),
    }
    svg.push_str("</svg>");
    svg
}

fn svg_numeric_panels(svg: &mut String, panels: &[NumericPanel], width: u32, height: u32) {
    if panels.is_empty() {
        return;
    }
    let panel_height = (height as f64 - SVG_CONTENT_TOP) / panels.len() as f64;
    for (panel_index, panel) in panels.iter().enumerate() {
        let top = SVG_CONTENT_TOP + panel_index as f64 * panel_height;
        let left = 125.0;
        let right = width as f64 - 290.0;
        let bottom = top + panel_height - 145.0;
        svg.push_str(&format!(
            "<g data-panel=\"{}\"><text x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"19\" font-weight=\"bold\">{}</text><text x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\">y: {}</text><text x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\">x: {}</text>",
            escape(&panel.title),
            top + 18.0,
            escape(&panel.title),
            top + 38.0,
            escape(&panel.y_label),
            bottom + 40.0,
            escape(&panel.x_label)
        ));
        let finite = panel
            .series
            .iter()
            .flat_map(|series| series.data.iter())
            .filter_map(|datum| {
                datum
                    .y
                    .filter(|_| !matches!(panel.x_scale, AxisScale::Log10) || datum.x > 0.0)
                    .map(|y| (datum.x, y))
            })
            .collect::<Vec<_>>();
        if finite.is_empty() {
            svg.push_str(&format!(
                "<text x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"16\">No serialized coordinate series available.</text>",
                top + 75.0
            ));
        } else {
            let (x_min, x_max, y_min, y_max) = numeric_bounds(&finite, panel.x_scale);
            svg.push_str(&format!(
                "<line x1=\"{left}\" y1=\"{bottom}\" x2=\"{right}\" y2=\"{bottom}\" stroke=\"black\"/><line x1=\"{left}\" y1=\"{}\" x2=\"{left}\" y2=\"{bottom}\" stroke=\"black\"/>",
                top + 48.0
            ));
            for (series_index, series) in panel.series.iter().enumerate() {
                let colour = colour_hex(series.colour);
                let mut segment = Vec::new();
                for datum in &series.data {
                    let plotted_y = datum
                        .y
                        .filter(|_| !matches!(panel.x_scale, AxisScale::Log10) || datum.x > 0.0);
                    if let Some(y) = plotted_y {
                        let x = map_x(datum.x, x_min, x_max, left, right, panel.x_scale);
                        let py = map_linear(y, y_min, y_max, bottom, top + 48.0);
                        segment.push((x, py));
                        svg.push_str(&format!(
                            "<circle data-series=\"{}\" data-x=\"{}\" data-y=\"{}\" cx=\"{x}\" cy=\"{py}\" r=\"4\" fill=\"{colour}\"><title>{}: x={} y={}</title></circle>",
                            escape(&series.label),
                            escape(&datum.x_text),
                            escape(&datum.y_text),
                            escape(&series.label),
                            escape(&datum.x_text),
                            escape(&datum.y_text)
                        ));
                    } else {
                        if series.connect {
                            svg_segment(svg, &segment, &colour);
                        }
                        segment.clear();
                    }
                }
                if series.connect {
                    svg_segment(svg, &segment, &colour);
                }
                svg.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\" fill=\"{colour}\">{}</text>",
                    right + 18.0,
                    top + 60.0 + series_index as f64 * 20.0,
                    escape(&series.label)
                ));
            }
        }
        for (note_index, note) in panel.notes.iter().take(6).enumerate() {
            svg.push_str(&format!(
                "<text data-missing-or-note=\"true\" x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"12\">{}</text>",
                bottom + 55.0 + note_index as f64 * 14.0,
                escape(note)
            ));
        }
        svg.push_str("</g>");
    }
}

fn svg_segment(svg: &mut String, segment: &[(f64, f64)], colour: &str) {
    if segment.len() < 2 {
        return;
    }
    let points = segment
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ");
    svg.push_str(&format!(
        "<polyline points=\"{points}\" fill=\"none\" stroke=\"{colour}\" stroke-width=\"2\"/>"
    ));
}

fn svg_category_panels(svg: &mut String, panels: &[CategoryPanel], width: u32, height: u32) {
    if panels.is_empty() {
        return;
    }
    let panel_height = (height as f64 - SVG_CONTENT_TOP) / panels.len() as f64;
    for (panel_index, panel) in panels.iter().enumerate() {
        let top = SVG_CONTENT_TOP + panel_index as f64 * panel_height;
        let left = 125.0;
        let right = width as f64 - 280.0;
        let bottom = top + panel_height - 135.0;
        svg.push_str(&format!(
            "<g data-categorical-panel=\"{}\"><text x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"19\" font-weight=\"bold\">{}</text><text x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\">y: {}</text><text x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\">x: {}</text>",
            escape(&panel.title),
            top + 18.0,
            escape(&panel.title),
            top + 38.0,
            escape(&panel.y_label),
            bottom + 48.0,
            escape(&panel.x_label)
        ));
        let values = panel
            .series
            .iter()
            .flat_map(|series| series.data.iter().map(|datum| datum.y))
            .collect::<Vec<_>>();
        if !values.is_empty() {
            let (y_min, y_max) = linear_extent(values.iter().copied());
            svg.push_str(&format!(
                "<line x1=\"{left}\" y1=\"{bottom}\" x2=\"{right}\" y2=\"{bottom}\" stroke=\"black\"/><line x1=\"{left}\" y1=\"{}\" x2=\"{left}\" y2=\"{bottom}\" stroke=\"black\"/>",
                top + 48.0
            ));
            for (category_index, category) in panel.categories.iter().enumerate() {
                let x = category_x(category_index, panel.categories.len(), left, right);
                svg.push_str(&format!(
                    "<text x=\"{x}\" y=\"{}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"12\">{}</text>",
                    bottom + 18.0,
                    escape(category)
                ));
            }
            for (series_index, series) in panel.series.iter().enumerate() {
                let colour = colour_hex(series.colour);
                for datum in &series.data {
                    if let Some(category_index) = panel
                        .categories
                        .iter()
                        .position(|category| category == &datum.category)
                    {
                        let base = category_x(category_index, panel.categories.len(), left, right);
                        let offset = (series_index as f64
                            - (panel.series.len().saturating_sub(1)) as f64 / 2.0)
                            * 10.0;
                        let y = map_linear(datum.y, y_min, y_max, bottom, top + 48.0);
                        svg.push_str(&format!(
                            "<circle data-series=\"{}\" data-category=\"{}\" data-y=\"{}\" cx=\"{}\" cy=\"{y}\" r=\"5\" fill=\"{colour}\"><title>{}: {} y={}; {}</title></circle>",
                            escape(&series.label),
                            escape(&datum.category),
                            escape(&datum.y_text),
                            base + offset,
                            escape(&series.label),
                            escape(&datum.category),
                            escape(&datum.y_text),
                            escape(&datum.annotation)
                        ));
                    }
                }
                svg.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\" fill=\"{colour}\">{}</text>",
                    right + 18.0,
                    top + 60.0 + series_index as f64 * 20.0,
                    escape(&series.label)
                ));
            }
        }
        for (note_index, note) in panel.notes.iter().take(4).enumerate() {
            svg.push_str(&format!(
                "<text x=\"{left}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"12\">{}</text>",
                bottom + 64.0 + note_index as f64 * 14.0,
                escape(note)
            ));
        }
        svg.push_str("</g>");
    }
}

fn svg_health_grid(svg: &mut String, rows: &[HealthRow], width: u32) {
    svg.push_str("<g data-categorical-grid=\"health-dimensions\"><text x=\"70\" y=\"170\" font-family=\"sans-serif\" font-size=\"15\">Dimension</text><text x=\"500\" y=\"170\" font-family=\"sans-serif\" font-size=\"15\">Stored status</text><text x=\"900\" y=\"170\" font-family=\"sans-serif\" font-size=\"15\">Evidence state / reason count</text>");
    for (index, row) in rows.iter().enumerate() {
        let y = 205 + index as i32 * 78;
        let fill = health_colour(&row.status);
        svg.push_str(&format!(
            "<rect x=\"55\" y=\"{}\" width=\"{}\" height=\"58\" rx=\"6\" fill=\"{}\" opacity=\"0.18\"/><text data-dimension=\"{}\" x=\"70\" y=\"{}\" font-family=\"sans-serif\" font-size=\"17\">{}</text><text data-status=\"{}\" x=\"500\" y=\"{}\" font-family=\"sans-serif\" font-size=\"17\">{}</text><text data-evidence-state=\"{}\" x=\"900\" y=\"{}\" font-family=\"sans-serif\" font-size=\"17\">{}; reasons={}</text>",
            y - 28,
            width - 110,
            fill,
            escape(&row.dimension),
            y,
            escape(&row.dimension),
            escape(&row.status),
            y,
            escape(&row.status),
            escape(&row.evidence_state),
            y,
            escape(&row.evidence_state),
            row.reason_count
        ));
    }
    svg.push_str("</g>");
}

fn svg_lineage(svg: &mut String, roots: &[LineageRoot], width: u32) {
    svg.push_str("<g data-lineage-graph=\"root-direct-only\">");
    let column_width = (width as f64 - 100.0) / roots.len().max(1) as f64;
    for (root_index, root) in roots.iter().enumerate() {
        let x = 55.0 + root_index as f64 * column_width;
        let root_x = x + 8.0;
        svg.push_str(&format!(
            "<rect x=\"{root_x}\" y=\"175\" width=\"{}\" height=\"95\" rx=\"6\" fill=\"#d9edf7\" stroke=\"#31708f\"/><text x=\"{}\" y=\"198\" font-family=\"sans-serif\" font-size=\"15\" font-weight=\"bold\">{}</text><text x=\"{}\" y=\"223\" font-family=\"sans-serif\" font-size=\"10\">{}</text><text x=\"{}\" y=\"248\" font-family=\"sans-serif\" font-size=\"11\">{}</text>",
            column_width - 16.0,
            root_x + 8.0,
            escape(&root.flag),
            root_x + 8.0,
            escape(&root.label),
            root_x + 8.0,
            escape(&root.catalog_membership)
        ));
        for (dependency_index, dependency) in root.dependencies.iter().enumerate() {
            let y = 335.0 + dependency_index as f64 * 95.0;
            let center = root_x + (column_width - 16.0) / 2.0;
            svg.push_str(&format!(
                "<line data-edge-role=\"{}\" x1=\"{center}\" y1=\"270\" x2=\"{center}\" y2=\"{y}\" stroke=\"#555\"/><text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"10\">{}</text><rect x=\"{root_x}\" y=\"{}\" width=\"{}\" height=\"58\" rx=\"5\" fill=\"#f5f5f5\" stroke=\"#777\"/><text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"10\">{} / {}</text><text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"9\">{}</text>",
                escape(&dependency.role),
                center + 4.0,
                y - 20.0,
                escape(&dependency.role),
                y,
                column_width - 16.0,
                root_x + 6.0,
                y + 22.0,
                escape(&dependency.kind),
                escape(&dependency.role),
                root_x + 6.0,
                y + 42.0,
                escape(&dependency.artifact_id)
            ));
        }
    }
    svg.push_str("</g>");
}

fn write_png(
    path: &Path,
    payload: &FigurePayload,
    width: u32,
    height: u32,
    id: FigureId,
) -> Result<(), PublicReportError> {
    let root = BitMapBackend::new(path, (width, height)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|error| plot_error(id, path, error))?;
    root.draw(&Text::new(
        payload.title,
        (50, 35),
        ("sans-serif", 30).into_font().style(FontStyle::Bold),
    ))
    .map_err(|error| plot_error(id, path, error))?;
    let caption_lines = wrap_text(&payload.caption, 185);
    for (index, line) in caption_lines.iter().enumerate() {
        root.draw(&Text::new(
            line.as_str(),
            (50, 66 + index as i32 * 18),
            ("sans-serif", 14).into_font(),
        ))
        .map_err(|error| plot_error(id, path, error))?;
    }
    let content = root.margin(78 + caption_lines.len() as u32 * 18, 35, 45, 45);
    match &payload.body {
        FigureBody::Numeric(panels) => draw_png_numeric(&content, panels, id, path)?,
        FigureBody::Category(panels) => draw_png_category(&content, panels, id, path)?,
        FigureBody::HealthGrid(rows) => draw_png_health(&content, rows, id, path)?,
        FigureBody::Lineage(roots) => draw_png_lineage(&content, roots, id, path)?,
    }
    root.present().map_err(|error| plot_error(id, path, error))
}

fn draw_png_numeric(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    panels: &[NumericPanel],
    id: FigureId,
    path: &Path,
) -> Result<(), PublicReportError> {
    if panels.is_empty() {
        return Ok(());
    }
    for (panel, panel_area) in panels.iter().zip(area.split_evenly((panels.len(), 1))) {
        let finite = panel
            .series
            .iter()
            .flat_map(|series| series.data.iter())
            .filter_map(|datum| {
                datum
                    .y
                    .filter(|_| !matches!(panel.x_scale, AxisScale::Log10) || datum.x > 0.0)
                    .map(|y| (datum.x, y))
            })
            .collect::<Vec<_>>();
        if finite.is_empty() {
            panel_area
                .draw(&Text::new(
                    format!("{} — {}", panel.title, panel.notes.join("; ")),
                    (20, 35),
                    ("sans-serif", 17).into_font(),
                ))
                .map_err(|error| plot_error(id, path, error))?;
            continue;
        }
        match panel.x_scale {
            AxisScale::Linear => draw_png_linear_panel(&panel_area, panel, &finite, id, path)?,
            AxisScale::Log10 => draw_png_log_panel(&panel_area, panel, &finite, id, path)?,
        }
        draw_png_notes(&panel_area, &panel.notes, id, path)?;
    }
    Ok(())
}

fn draw_png_linear_panel(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    panel: &NumericPanel,
    finite: &[(f64, f64)],
    id: FigureId,
    path: &Path,
) -> Result<(), PublicReportError> {
    let (x_min, x_max, y_min, y_max) = numeric_bounds(finite, AxisScale::Linear);
    let mut chart = ChartBuilder::on(area)
        .caption(panel.title.as_str(), ("sans-serif", 18))
        .margin(12)
        .x_label_area_size(44)
        .y_label_area_size(70)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|error| plot_error(id, path, error))?;
    chart
        .configure_mesh()
        .x_desc(panel.x_label.as_str())
        .y_desc(panel.y_label.as_str())
        .draw()
        .map_err(|error| plot_error(id, path, error))?;
    draw_png_series(&mut chart, panel, id, path)?;
    Ok(())
}

fn draw_png_log_panel(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    panel: &NumericPanel,
    finite: &[(f64, f64)],
    id: FigureId,
    path: &Path,
) -> Result<(), PublicReportError> {
    let positives = finite
        .iter()
        .copied()
        .filter(|(x, _)| *x > 0.0)
        .collect::<Vec<_>>();
    if positives.is_empty() {
        area.draw(&Text::new(
            format!("{} — no positive serialized x values", panel.title),
            (20, 35),
            ("sans-serif", 17).into_font(),
        ))
        .map_err(|error| plot_error(id, path, error))?;
        return Ok(());
    }
    let (x_min, x_max, y_min, y_max) = numeric_bounds(&positives, AxisScale::Log10);
    let mut chart = ChartBuilder::on(area)
        .caption(panel.title.as_str(), ("sans-serif", 18))
        .margin(12)
        .x_label_area_size(44)
        .y_label_area_size(70)
        .build_cartesian_2d((x_min..x_max).log_scale(), y_min..y_max)
        .map_err(|error| plot_error(id, path, error))?;
    chart
        .configure_mesh()
        .x_desc(panel.x_label.as_str())
        .y_desc(panel.y_label.as_str())
        .draw()
        .map_err(|error| plot_error(id, path, error))?;
    for series in &panel.series {
        let colour = series.colour;
        if series.connect {
            for segment in finite_segments(&series.data, true) {
                chart
                    .draw_series(LineSeries::new(segment, colour.stroke_width(2)))
                    .map_err(|error| plot_error(id, path, error))?;
            }
        }
        chart
            .draw_series(series.data.iter().filter_map(|datum| {
                datum
                    .y
                    .filter(|_| datum.x > 0.0)
                    .map(|y| Circle::new((datum.x, y), 4, colour.filled()))
            }))
            .map_err(|error| plot_error(id, path, error))?
            .label(series.label.as_str())
            .legend(move |(x, y)| Circle::new((x, y), 4, colour.filled()));
    }
    chart
        .configure_series_labels()
        .border_style(BLACK)
        .background_style(WHITE.mix(0.8))
        .draw()
        .map_err(|error| plot_error(id, path, error))?;
    Ok(())
}

fn draw_png_series<'a, DB: DrawingBackend + 'a>(
    chart: &mut ChartContext<
        'a,
        DB,
        Cartesian2d<plotters::coord::types::RangedCoordf64, plotters::coord::types::RangedCoordf64>,
    >,
    panel: &NumericPanel,
    id: FigureId,
    path: &Path,
) -> Result<(), PublicReportError> {
    for series in &panel.series {
        let colour = series.colour;
        if series.connect {
            for segment in finite_segments(&series.data, false) {
                chart
                    .draw_series(LineSeries::new(segment, colour.stroke_width(2)))
                    .map_err(|error| plot_error(id, path, error))?;
            }
        }
        chart
            .draw_series(series.data.iter().filter_map(|datum| {
                datum
                    .y
                    .map(|y| Circle::new((datum.x, y), 4, colour.filled()))
            }))
            .map_err(|error| plot_error(id, path, error))?
            .label(series.label.as_str())
            .legend(move |(x, y)| Circle::new((x, y), 4, colour.filled()));
    }
    chart
        .configure_series_labels()
        .border_style(BLACK)
        .background_style(WHITE.mix(0.8))
        .draw()
        .map_err(|error| plot_error(id, path, error))?;
    Ok(())
}

fn finite_segments(data: &[NumericDatum], positive_x: bool) -> Vec<Vec<(f64, f64)>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for datum in data {
        match datum.y.filter(|_| !positive_x || datum.x > 0.0) {
            Some(y) => current.push((datum.x, y)),
            None => {
                if current.len() > 1 {
                    result.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
        }
    }
    if current.len() > 1 {
        result.push(current);
    }
    result
}

fn draw_png_category(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    panels: &[CategoryPanel],
    id: FigureId,
    path: &Path,
) -> Result<(), PublicReportError> {
    if panels.is_empty() {
        return Ok(());
    }
    for (panel, panel_area) in panels.iter().zip(area.split_evenly((panels.len(), 1))) {
        let values = panel
            .series
            .iter()
            .flat_map(|series| series.data.iter().map(|datum| datum.y))
            .collect::<Vec<_>>();
        if values.is_empty() {
            continue;
        }
        let (y_min, y_max) = linear_extent(values.into_iter());
        let count = panel.categories.len().max(1) as f64;
        let mut chart = ChartBuilder::on(&panel_area)
            .caption(panel.title.as_str(), ("sans-serif", 18))
            .margin(12)
            .x_label_area_size(55)
            .y_label_area_size(70)
            .build_cartesian_2d(-0.5..count - 0.5, y_min..y_max)
            .map_err(|error| plot_error(id, path, error))?;
        chart
            .configure_mesh()
            .x_desc(panel.x_label.as_str())
            .y_desc(panel.y_label.as_str())
            .x_labels(panel.categories.len())
            .x_label_formatter(&|value| {
                let index = value.round() as isize;
                if index >= 0 {
                    panel
                        .categories
                        .get(index as usize)
                        .cloned()
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            })
            .draw()
            .map_err(|error| plot_error(id, path, error))?;
        for (series_index, series) in panel.series.iter().enumerate() {
            let colour = series.colour;
            chart
                .draw_series(series.data.iter().filter_map(|datum| {
                    panel
                        .categories
                        .iter()
                        .position(|category| category == &datum.category)
                        .map(|index| {
                            let offset = (series_index as f64
                                - (panel.series.len().saturating_sub(1)) as f64 / 2.0)
                                * 0.08;
                            Circle::new((index as f64 + offset, datum.y), 5, colour.filled())
                        })
                }))
                .map_err(|error| plot_error(id, path, error))?
                .label(series.label.as_str())
                .legend(move |(x, y)| Circle::new((x, y), 5, colour.filled()));
        }
        chart
            .configure_series_labels()
            .border_style(BLACK)
            .background_style(WHITE.mix(0.8))
            .draw()
            .map_err(|error| plot_error(id, path, error))?;
        draw_png_notes(&panel_area, &panel.notes, id, path)?;
    }
    Ok(())
}

fn draw_png_notes(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    notes: &[String],
    id: FigureId,
    path: &Path,
) -> Result<(), PublicReportError> {
    for (index, note) in notes.iter().take(4).enumerate() {
        area.draw(&Text::new(
            format!("note: {note}"),
            (95, 52 + index as i32 * 15),
            ("sans-serif", 11).into_font(),
        ))
        .map_err(|error| plot_error(id, path, error))?;
    }
    Ok(())
}

fn draw_png_health(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    rows: &[HealthRow],
    id: FigureId,
    path: &Path,
) -> Result<(), PublicReportError> {
    for (index, row) in rows.iter().enumerate() {
        let y = 25 + index as i32 * 82;
        let colour = health_rgb(&row.status);
        area.draw(&Rectangle::new(
            [(10, y - 20), (area.dim_in_pixel().0 as i32 - 10, y + 42)],
            colour.mix(0.15).filled(),
        ))
        .map_err(|error| plot_error(id, path, error))?;
        area.draw(&Text::new(
            format!(
                "{} | {} | {} | reasons={}",
                row.dimension, row.status, row.evidence_state, row.reason_count
            ),
            (25, y + 12),
            ("sans-serif", 18).into_font(),
        ))
        .map_err(|error| plot_error(id, path, error))?;
    }
    Ok(())
}

fn draw_png_lineage(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    roots: &[LineageRoot],
    id: FigureId,
    path: &Path,
) -> Result<(), PublicReportError> {
    let (width, _) = area.dim_in_pixel();
    let column = width as i32 / roots.len().max(1) as i32;
    for (index, root) in roots.iter().enumerate() {
        let left = index as i32 * column + 5;
        let right = left + column - 10;
        area.draw(&Rectangle::new(
            [(left, 15), (right, 100)],
            RGBColor(217, 237, 247).filled(),
        ))
        .map_err(|error| plot_error(id, path, error))?;
        area.draw(&Text::new(
            format!(
                "{} | {} | {}",
                root.flag, root.label, root.catalog_membership
            ),
            (left + 6, 45),
            ("sans-serif", 11).into_font(),
        ))
        .map_err(|error| plot_error(id, path, error))?;
        for (dependency_index, dependency) in root.dependencies.iter().enumerate() {
            let y = 150 + dependency_index as i32 * 82;
            let center = (left + right) / 2;
            area.draw(&PathElement::new(vec![(center, 100), (center, y)], BLACK))
                .map_err(|error| plot_error(id, path, error))?;
            area.draw(&Rectangle::new(
                [(left, y), (right, y + 54)],
                RGBColor(245, 245, 245).filled(),
            ))
            .map_err(|error| plot_error(id, path, error))?;
            area.draw(&Text::new(
                format!(
                    "{} | {} | {}",
                    dependency.role, dependency.kind, dependency.artifact_id
                ),
                (left + 5, y + 28),
                ("sans-serif", 10).into_font(),
            ))
            .map_err(|error| plot_error(id, path, error))?;
        }
    }
    Ok(())
}

fn numeric_bounds(values: &[(f64, f64)], scale: AxisScale) -> (f64, f64, f64, f64) {
    let xs = values
        .iter()
        .map(|(x, _)| *x)
        .filter(|x| !matches!(scale, AxisScale::Log10) || *x > 0.0);
    let ys = values.iter().map(|(_, y)| *y);
    let (mut x_min, mut x_max) = linear_extent(xs);
    let (y_min, y_max) = linear_extent(ys);
    if matches!(scale, AxisScale::Log10) && x_min <= 0.0 {
        x_min = f64::MIN_POSITIVE;
    }
    if matches!(scale, AxisScale::Log10) && x_min == x_max {
        x_min /= 10.0;
        x_max *= 10.0;
    }
    (x_min, x_max, y_min, y_max)
}

fn linear_extent(values: impl Iterator<Item = f64>) -> (f64, f64) {
    let (mut min, mut max) = values
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), value| {
            (min.min(value), max.max(value))
        });
    if !min.is_finite() || !max.is_finite() {
        return (0.0, 1.0);
    }
    if min == max {
        let padding = if min == 0.0 { 0.5 } else { min.abs() * 0.05 };
        min -= padding;
        max += padding;
    }
    (min, max)
}

fn map_x(value: f64, min: f64, max: f64, left: f64, right: f64, scale: AxisScale) -> f64 {
    let ratio = match scale {
        AxisScale::Linear => (value - min) / (max - min),
        AxisScale::Log10 => (value.log10() - min.log10()) / (max.log10() - min.log10()),
    };
    left + ratio * (right - left)
}

fn map_linear(value: f64, min: f64, max: f64, start: f64, end: f64) -> f64 {
    start + (value - min) / (max - min) * (end - start)
}

fn category_x(index: usize, count: usize, left: f64, right: f64) -> f64 {
    if count <= 1 {
        return (left + right) / 2.0;
    }
    left + index as f64 / (count - 1) as f64 * (right - left)
}

fn health_colour(status: &str) -> &'static str {
    match status {
        "within_baseline" => "#2e7d32",
        "watch" => "#f9a825",
        "degraded" | "critical" => "#b71c1c",
        "data_quality_insufficient" | "indeterminate" => "#6a1b9a",
        _ => "#455a64",
    }
}

fn health_rgb(status: &str) -> RGBColor {
    match status {
        "within_baseline" => RGBColor(46, 125, 50),
        "watch" => RGBColor(249, 168, 37),
        "degraded" | "critical" => RGBColor(183, 28, 28),
        "data_quality_insufficient" | "indeterminate" => RGBColor(106, 27, 154),
        _ => RGBColor(69, 90, 100),
    }
}

fn colour_hex(colour: RGBColor) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.0, colour.1, colour.2)
}

fn plot_error<E: std::fmt::Debug>(id: FigureId, path: &Path, error: E) -> PublicReportError {
    PublicReportError::PlotBackend {
        figure_id: id.as_str().into(),
        path: path.to_path_buf(),
        message: format!("{error:?}"),
    }
}

fn wrap_text(value: &str, maximum_characters: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        let added = usize::from(!current.is_empty()) + word.len();
        if !current.is_empty() && current.len() + added > maximum_characters {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
