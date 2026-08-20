//! Immutable borrowed presentation state for the Phase-D renderer.

use crate::{
    domain::{ArtifactLineageCatalog, ArtifactLineageState},
    reporting::{AvailabilityReason, reader::ReportInputs},
    results::{
        CalibrationAnalysisReport, CalibrationObservationSet, EisFitArtifact,
        MechanismAnalysisReport, ModelAnalysisReport, SensorHealthAssessment, SignalAnalysisReport,
        StateEstimationReport, TransientAnalysisReport,
    },
};
use std::path::PathBuf;

/// Presentation-only view over canonical inputs. This has no artifact
/// identity, lineage constructor, mutable source access, or science-module
/// dependency.
#[derive(Debug)]
pub(crate) struct PublicReportProjection<'a> {
    pub input_paths: ReportInputPaths,
    pub mechanism: &'a MechanismAnalysisReport,
    pub health: &'a SensorHealthAssessment,
    pub lineage_catalog: Option<&'a ArtifactLineageCatalog>,
    pub eis: Option<&'a EisFitArtifact>,
    pub transient: Option<&'a TransientAnalysisReport>,
    pub calibration: Option<&'a CalibrationAnalysisReport>,
    pub calibration_observations: Option<&'a CalibrationObservationSet>,
    pub signal: Option<&'a SignalAnalysisReport>,
    pub estimation: Option<&'a StateEstimationReport>,
    pub model: Option<&'a ModelAnalysisReport>,
    pub required_compatibility: crate::reporting::reader::CompatibilityOutcome,
    pub optional_compatibility: Vec<(
        &'static str,
        &'static str,
        crate::reporting::reader::CompatibilityOutcome,
    )>,
    pub health_evidence_records: Vec<&'a crate::evidence::EvidenceRecord>,
    pub mechanism_history_count: usize,
    pub history_projection_traversals: usize,
    pub evidence_projection_traversals: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ReportInputPaths {
    pub mechanism: PathBuf,
    pub health: PathBuf,
    pub lineage_catalog: Option<PathBuf>,
    pub eis: Option<PathBuf>,
    pub transient: Option<PathBuf>,
    pub calibration: Option<PathBuf>,
    pub calibration_observations: Option<PathBuf>,
    pub signal: Option<PathBuf>,
    pub estimation: Option<PathBuf>,
    pub model: Option<PathBuf>,
}

impl<'a> PublicReportProjection<'a> {
    pub fn from_inputs(inputs: &'a ReportInputs) -> Self {
        let mechanism_history_count = inputs.mechanism.hypothesis_history.len();
        let mut health_evidence_records = inputs
            .health
            .phase_c
            .as_ref()
            .map(|phase| phase.evidence_bundle.records.iter().collect::<Vec<_>>())
            .unwrap_or_default();
        health_evidence_records.sort_by(|left, right| {
            left.evidence_id
                .cmp(&right.evidence_id)
                .then(evidence_source_key(left).cmp(&evidence_source_key(right)))
        });
        Self {
            input_paths: ReportInputPaths {
                mechanism: inputs.input_paths.mechanism.clone(),
                health: inputs.input_paths.health.clone(),
                lineage_catalog: inputs.input_paths.lineage_catalog.clone(),
                eis: inputs.input_paths.eis.clone(),
                transient: inputs.input_paths.transient.clone(),
                calibration: inputs.input_paths.calibration.clone(),
                calibration_observations: inputs.input_paths.calibration_observations.clone(),
                signal: inputs.input_paths.signal.clone(),
                estimation: inputs.input_paths.estimation.clone(),
                model: inputs.input_paths.model.clone(),
            },
            mechanism: &inputs.mechanism,
            health: &inputs.health,
            lineage_catalog: inputs.lineage_catalog.as_ref(),
            eis: inputs.eis.as_ref(),
            transient: inputs.transient.as_ref(),
            calibration: inputs.calibration.as_ref(),
            calibration_observations: inputs.calibration_observations.as_ref(),
            signal: inputs.signal.as_ref(),
            estimation: inputs.estimation.as_ref(),
            model: inputs.model.as_ref(),
            required_compatibility: inputs.required_compatibility,
            optional_compatibility: inputs.optional_compatibility.clone(),
            health_evidence_records,
            mechanism_history_count,
            history_projection_traversals: 1,
            evidence_projection_traversals: usize::from(inputs.health.phase_c.is_some()),
        }
    }

    pub fn mechanism_is_legacy(&self) -> bool {
        self.mechanism.schema_version < 4
    }
    pub fn health_is_legacy(&self) -> bool {
        self.health.schema_version < 4
    }

    pub fn traversal_audit(&self) -> (usize, usize, usize, usize) {
        (
            self.history_projection_traversals,
            self.evidence_projection_traversals,
            self.mechanism_history_count,
            self.health_evidence_records.len(),
        )
    }

    pub fn figure_reason(
        &self,
        figure: crate::report_config::FigureId,
    ) -> Option<AvailabilityReason> {
        use crate::report_config::FigureId;
        match figure {
            FigureId::MechanismTimescale => (!self
                .mechanism
                .comparisons
                .iter()
                .any(|row| row.log10_distance.is_some_and(f64::is_finite)))
            .then_some(AvailabilityReason::SerializedSeriesUnavailable),
            FigureId::SensorHealthDimensionStatus => self
                .health
                .phase_c
                .is_none()
                .then_some(AvailabilityReason::LegacyPhaseCNotSerialized),
            FigureId::CurrentVsBaseline => baseline_reason(self.health),
            FigureId::EisNyquist => self
                .eis
                .map_or(Some(AvailabilityReason::NotProvided), eis_nyquist_reason),
            FigureId::EisBode => self
                .eis
                .map_or(Some(AvailabilityReason::NotProvided), eis_bode_reason),
            FigureId::TransientResponse => self
                .transient
                .map_or(Some(AvailabilityReason::NotProvided), transient_reason),
            FigureId::CalibrationPerformance => {
                calibration_reason(self.calibration, self.calibration_observations)
            }
            FigureId::SignalDiagnostics => self
                .signal
                .map_or(Some(AvailabilityReason::NotProvided), signal_reason),
            FigureId::EstimationObservedPredicted => {
                self.estimation
                    .map_or(Some(AvailabilityReason::NotProvided), |value| {
                        (value
                            .estimates
                            .iter()
                            .filter(|point| {
                                point.timestamp_s.is_finite()
                                    && (point.measurement_v.is_some()
                                        || point.predicted_measurement_v.is_some())
                            })
                            .count()
                            < 2)
                        .then_some(AvailabilityReason::SerializedSeriesUnavailable)
                    })
            }
            FigureId::ModelObservedPredicted => {
                self.model
                    .map_or(Some(AvailabilityReason::NotProvided), |value| {
                        (value
                            .points
                            .iter()
                            .filter(|point| {
                                point.time_s.is_finite() && point.predicted_voltage_v.is_finite()
                            })
                            .count()
                            < 2)
                        .then_some(AvailabilityReason::SerializedSeriesUnavailable)
                    })
            }
            FigureId::Lineage => None,
        }
    }

    pub fn supplied_lineages(&self) -> Vec<(&'static str, &ArtifactLineageState)> {
        let mut rows = vec![
            ("mechanism", &self.mechanism.lineage),
            ("health", &self.health.lineage),
        ];
        if let Some(value) = &self.eis {
            rows.push(("eis", &value.lineage));
        }
        if let Some(value) = &self.transient {
            rows.push(("transient", &value.lineage));
        }
        if let Some(value) = &self.calibration {
            rows.push(("calibration", &value.lineage));
        }
        if let Some(value) = &self.calibration_observations {
            rows.push(("calibration_observations", &value.lineage));
        }
        if let Some(value) = &self.signal {
            rows.push(("signal", &value.lineage));
        }
        if let Some(value) = &self.estimation {
            rows.push(("estimation", &value.lineage));
        }
        if let Some(value) = &self.model {
            rows.push(("model", &value.lineage));
        }
        rows
    }
}

fn evidence_source_key(record: &crate::evidence::EvidenceRecord) -> (&str, &str, &str) {
    match &record.source.artifact {
        crate::evidence::EvidenceArtifactSource::Known {
            artifact_id,
            artifact_kind,
        } => (
            artifact_kind.as_str(),
            artifact_id.0.as_str(),
            record.source.field_path.as_str(),
        ),
        crate::evidence::EvidenceArtifactSource::LegacyUnknown {
            artifact_kind,
            source_fingerprint,
        } => (
            artifact_kind.as_str(),
            source_fingerprint.0.as_str(),
            record.source.field_path.as_str(),
        ),
    }
}

fn baseline_reason(health: &SensorHealthAssessment) -> Option<AvailabilityReason> {
    if health.baseline_comparison.is_empty() {
        return Some(AvailabilityReason::SerializedSeriesUnavailable);
    }
    let mut saw_unit_failure = false;
    let mut saw_comparable = false;
    let mut saw_unknown = false;
    let mut saw_not_comparable = false;
    for comparison in &health.baseline_comparison {
        let count = health
            .features
            .iter()
            .filter(|feature| feature.name == comparison.feature && !feature.unit.is_empty())
            .count();
        if count != 1 {
            saw_unit_failure = true;
            continue;
        }
        match comparison.comparability {
            crate::results::FeatureComparability::Comparable
            | crate::results::FeatureComparability::ComparableWithWarnings => {
                saw_comparable = true;
            }
            crate::results::FeatureComparability::Unknown => {
                saw_unknown = true;
                continue;
            }
            crate::results::FeatureComparability::NotComparable => {
                saw_not_comparable = true;
                continue;
            }
        }
        if comparison.current_value.is_some_and(f64::is_finite)
            && comparison.baseline_value.is_some_and(f64::is_finite)
        {
            return None;
        }
    }
    if saw_unit_failure {
        Some(AvailabilityReason::UnitAuthorityUnavailable)
    } else if saw_comparable {
        Some(AvailabilityReason::NoComparableFinitePair)
    } else if saw_unknown {
        Some(AvailabilityReason::ComparisonUnknown)
    } else if saw_not_comparable {
        Some(AvailabilityReason::NotComparable)
    } else {
        Some(AvailabilityReason::SerializedSeriesUnavailable)
    }
}

fn eis_nyquist_reason(value: &EisFitArtifact) -> Option<AvailabilityReason> {
    let count = value.source.frequency_hz.len();
    if count == 0
        || value.source.z_real_ohm.len() != count
        || value.source.z_imag_ohm.len() != count
        || value.fitted.z_real_ohm.len() != count
        || value.fitted.z_imag_ohm.len() != count
    {
        return Some(AvailabilityReason::SerializedSeriesInvalid);
    }
    if value
        .source
        .frequency_hz
        .iter()
        .chain(value.source.z_real_ohm.iter())
        .chain(value.source.z_imag_ohm.iter())
        .chain(value.fitted.z_real_ohm.iter())
        .chain(value.fitted.z_imag_ohm.iter())
        .all(|value| value.is_finite())
    {
        None
    } else {
        Some(AvailabilityReason::SerializedSeriesInvalid)
    }
}

fn eis_bode_reason(value: &EisFitArtifact) -> Option<AvailabilityReason> {
    let count = value.source.frequency_hz.len();
    if count == 0
        || !value
            .source
            .frequency_hz
            .iter()
            .any(|frequency| frequency.is_finite() && *frequency > 0.0)
        || value.fitted.magnitude_ohm.len() != count
        || value.fitted.phase_deg.len() != count
    {
        return Some(AvailabilityReason::SerializedSeriesInvalid);
    }
    let source_magnitude_len = value
        .source
        .source_measured_magnitude_ohm
        .as_ref()
        .map_or(value.source.derived_magnitude_ohm.len(), Vec::len);
    let source_phase_len = value
        .source
        .source_measured_phase_deg
        .as_ref()
        .map_or(value.source.derived_phase_deg.len(), Vec::len);
    if source_magnitude_len != count || source_phase_len != count {
        return Some(AvailabilityReason::SerializedSeriesInvalid);
    }
    None
}

fn transient_reason(value: &TransientAnalysisReport) -> Option<AvailabilityReason> {
    let mut selected = false;
    let mut valid = false;
    let mut missing = false;
    let mut ambiguous = false;
    let mut invalid = false;
    for event in &value.events {
        let Some(model) = &event.selected_model else {
            continue;
        };
        selected = true;
        let matches = event
            .candidate_fits
            .iter()
            .filter(|fit| fit.is_successful() && &fit.model == model)
            .collect::<Vec<_>>();
        if matches.is_empty() {
            missing = true;
            continue;
        }
        if matches.len() > 1 {
            ambiguous = true;
            continue;
        }
        let fit = matches[0];
        if event.segment.raw_time_local.len() != event.segment.raw_potential_v.len()
            || event.segment.fitted_time_local.is_empty()
            || fit.predicted_v.is_empty()
            || fit.residuals_v.is_empty()
        {
            invalid = true;
            continue;
        }
        valid = true;
    }
    if valid {
        None
    } else if ambiguous {
        Some(AvailabilityReason::SelectedFitAmbiguous)
    } else if invalid {
        Some(AvailabilityReason::SerializedSeriesInvalid)
    } else if missing || !selected {
        Some(AvailabilityReason::SelectedFitNotFound)
    } else {
        Some(AvailabilityReason::SerializedSeriesUnavailable)
    }
}

fn calibration_reason(
    calibration: Option<&CalibrationAnalysisReport>,
    observations: Option<&CalibrationObservationSet>,
) -> Option<AvailabilityReason> {
    let (Some(calibration), Some(_)) = (calibration, observations) else {
        return Some(AvailabilityReason::PairedInputNotProvided);
    };
    if calibration.validation.as_ref().is_some_and(|value| {
        value.predictions.iter().any(|point| {
            point.observed_log10_activity.is_some_and(f64::is_finite)
                && point.observed_potential_v.is_finite()
                && point.predicted_potential_v.is_some_and(f64::is_finite)
        })
    }) {
        None
    } else {
        Some(AvailabilityReason::SerializedSeriesUnavailable)
    }
}

fn signal_reason(value: &SignalAnalysisReport) -> Option<AvailabilityReason> {
    let time_available = value.analysis_timestamps.len() == value.analysis_values.len()
        && value
            .analysis_timestamps
            .iter()
            .zip(&value.analysis_values)
            .any(|(x, y)| x.is_finite() && y.is_some_and(f64::is_finite));
    let psd_available = value.psd.as_ref().is_some_and(|psd| {
        psd.frequency_hz.len() == psd.psd.len()
            && psd
                .frequency_hz
                .iter()
                .zip(&psd.psd)
                .any(|(x, y)| x.is_finite() && *x > 0.0 && y.is_finite())
    });
    let allan_available = value.allan.as_ref().is_some_and(|allan| {
        allan.points.iter().any(|point| {
            point.averaging_time_s.is_finite()
                && point.averaging_time_s > 0.0
                && point.deviation.is_some_and(f64::is_finite)
        })
    });
    (!time_available && !psd_available && !allan_available)
        .then_some(AvailabilityReason::SerializedSeriesUnavailable)
}
