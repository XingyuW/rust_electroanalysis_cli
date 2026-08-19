//! Immutable copied presentation state for the Phase-D renderer.

use crate::{
    domain::{ArtifactLineageCatalog, ArtifactLineageState},
    reporting::{AvailabilityReason, reader::ReportInputs},
    results::{
        CalibrationAnalysisReport, CalibrationObservationSet, EisFitArtifact,
        MechanismAnalysisReport, ModelAnalysisReport, SensorHealthAssessment, SignalAnalysisReport,
        StateEstimationReport, TransientAnalysisReport,
    },
};
use serde::Serialize;
use std::path::PathBuf;

/// Presentation-only copies of canonical inputs.  This has no artifact
/// identity, lineage constructor, mutable source access, or science-module
/// dependency.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct PublicReportProjection {
    pub input_paths: ReportInputPaths,
    pub mechanism: MechanismAnalysisReport,
    pub health: SensorHealthAssessment,
    pub lineage_catalog: Option<ArtifactLineageCatalog>,
    pub eis: Option<EisFitArtifact>,
    pub transient: Option<TransientAnalysisReport>,
    pub calibration: Option<CalibrationAnalysisReport>,
    pub calibration_observations: Option<CalibrationObservationSet>,
    pub signal: Option<SignalAnalysisReport>,
    pub estimation: Option<StateEstimationReport>,
    pub model: Option<ModelAnalysisReport>,
    pub required_compatibility: crate::reporting::reader::CompatibilityOutcome,
    pub optional_compatibility: Vec<(
        &'static str,
        &'static str,
        crate::reporting::reader::CompatibilityOutcome,
    )>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
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

impl PublicReportProjection {
    pub fn from_inputs(inputs: &ReportInputs) -> Self {
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
            mechanism: inputs.mechanism.clone(),
            health: inputs.health.clone(),
            lineage_catalog: inputs.lineage_catalog.clone(),
            eis: inputs.eis.clone(),
            transient: inputs.transient.clone(),
            calibration: inputs.calibration.clone(),
            calibration_observations: inputs.calibration_observations.clone(),
            signal: inputs.signal.clone(),
            estimation: inputs.estimation.clone(),
            model: inputs.model.clone(),
            required_compatibility: inputs.required_compatibility,
            optional_compatibility: inputs.optional_compatibility.clone(),
        }
    }

    pub fn mechanism_is_legacy(&self) -> bool {
        self.mechanism.schema_version < 4
    }
    pub fn health_is_legacy(&self) -> bool {
        self.health.schema_version < 4
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
            FigureId::CurrentVsBaseline => baseline_reason(&self.health),
            FigureId::EisNyquist | FigureId::EisBode => self
                .eis
                .as_ref()
                .map_or(Some(AvailabilityReason::NotProvided), eis_reason),
            FigureId::TransientResponse => self
                .transient
                .as_ref()
                .map_or(Some(AvailabilityReason::NotProvided), transient_reason),
            FigureId::CalibrationPerformance => calibration_reason(
                self.calibration.as_ref(),
                self.calibration_observations.as_ref(),
            ),
            FigureId::SignalDiagnostics => {
                self.signal
                    .as_ref()
                    .map_or(Some(AvailabilityReason::NotProvided), |value| {
                        value
                            .analysis_timestamps
                            .is_empty()
                            .then_some(AvailabilityReason::SerializedSeriesUnavailable)
                    })
            }
            FigureId::EstimationObservedPredicted => {
                self.estimation
                    .as_ref()
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
                    .as_ref()
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

fn baseline_reason(health: &SensorHealthAssessment) -> Option<AvailabilityReason> {
    if health.baseline_comparison.is_empty() {
        return Some(AvailabilityReason::SerializedSeriesUnavailable);
    }
    for comparison in &health.baseline_comparison {
        if !matches!(
            comparison.comparability,
            crate::results::FeatureComparability::Comparable
                | crate::results::FeatureComparability::ComparableWithWarnings
        ) {
            continue;
        }
        let count = health
            .features
            .iter()
            .filter(|feature| feature.name == comparison.feature && !feature.unit.is_empty())
            .count();
        if count != 1 {
            return Some(AvailabilityReason::UnitAuthorityUnavailable);
        }
        if comparison.current_value.is_some_and(f64::is_finite)
            && comparison.baseline_value.is_some_and(f64::is_finite)
        {
            return None;
        }
    }
    Some(AvailabilityReason::NoComparableFinitePair)
}

fn eis_reason(value: &EisFitArtifact) -> Option<AvailabilityReason> {
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

fn transient_reason(value: &TransientAnalysisReport) -> Option<AvailabilityReason> {
    let mut selected = false;
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
            return Some(AvailabilityReason::SelectedFitNotFound);
        }
        if matches.len() > 1 {
            return Some(AvailabilityReason::SelectedFitAmbiguous);
        }
        let fit = matches[0];
        let predicted_matches_raw = fit.predicted_v.len() == event.segment.raw_time_local.len();
        let predicted_matches_fitted =
            fit.predicted_v.len() == event.segment.fitted_time_local.len();
        let residual_matches_raw = fit.residuals_v.len() == event.segment.raw_time_local.len();
        let residual_matches_fitted =
            fit.residuals_v.len() == event.segment.fitted_time_local.len();
        if (!predicted_matches_raw && !predicted_matches_fitted)
            || (!residual_matches_raw && !residual_matches_fitted)
        {
            return Some(AvailabilityReason::SerializedSeriesInvalid);
        }
    }
    (!selected).then_some(AvailabilityReason::SelectedFitNotFound)
}

fn calibration_reason(
    calibration: Option<&CalibrationAnalysisReport>,
    observations: Option<&CalibrationObservationSet>,
) -> Option<AvailabilityReason> {
    let (Some(calibration), Some(_)) = (calibration, observations) else {
        return Some(AvailabilityReason::PairedInputNotProvided);
    };
    if calibration
        .validation
        .as_ref()
        .is_some_and(|value| !value.predictions.is_empty())
    {
        None
    } else {
        Some(AvailabilityReason::SerializedSeriesUnavailable)
    }
}
