#![allow(clippy::collapsible_if)]

//! Offline, uncertainty-aware state estimation for time-varying potentiometric
//! measurements.  This module owns the state-space algorithms; runners and
//! plotting adapters only orchestrate or render completed artifacts.
//!
//! Measurement uncertainty is resolved per scalar observation and records its
//! source and calibration-domain inflation.  NIS/NEES and innovation
//! diagnostics are consistency diagnostics with finite-sample limitations, not
//! proof of statistical validity.  Truth validation uses an explicit alignment
//! policy and state-specific tolerances; synthetic fixtures cannot replace
//! independent experimental validation.

pub mod calibration_adapter;
pub mod comparison;
pub mod covariance;
pub mod ekf;
pub mod environment;
pub mod error;
pub mod initialization;
pub mod innovation;
pub mod measurement;
pub mod model;
pub mod observability;
pub mod process;
pub mod simulation;
pub mod smoothing;
pub mod state;
pub mod timestamp;
pub mod ukf;
pub mod validation;

pub use measurement::{AuxiliaryObservation, AuxiliaryObservationKind};

use crate::{
    domain::ElectrochemicalExperiment,
    estimation_config::{FilterKind, ResolvedEstimationConfig, StateModelKind},
    results::{
        CalibrationAnalysisReport, EisFitArtifact, MechanismAnalysisReport, SensorHealthAssessment,
        SensorHealthBaseline, SignalAnalysisReport, StateEstimationReport, TransientAnalysisReport,
    },
};
use calibration_adapter::StoredCalibrationObservationModel;
use covariance::{resolve_measurement_covariance, resolve_process_covariance};
use environment::{
    AlignedEnvironment, align_experiment_with_polarization, resolve_standard_activity,
};
use error::EstimationError;
use initialization::initialize_state;
use measurement::observations;
use model::StateModel;
use observability::diagnose;

#[derive(Default, Clone, Copy)]
pub struct EstimationContext<'a> {
    pub signal: Option<&'a SignalAnalysisReport>,
    pub transient: Option<&'a TransientAnalysisReport>,
    pub calibration_results: Option<&'a CalibrationAnalysisReport>,
    pub eis_fit: Option<&'a EisFitArtifact>,
    pub mechanism: Option<&'a MechanismAnalysisReport>,
    pub health_baseline: Option<&'a SensorHealthBaseline>,
    pub health_assessment: Option<&'a SensorHealthAssessment>,
}

pub fn estimate_experiment(
    experiment: &ElectrochemicalExperiment,
    channel: &str,
    calibration: StoredCalibrationObservationModel,
    config: &ResolvedEstimationConfig,
    context: EstimationContext<'_>,
    filter: FilterKind,
) -> Result<StateEstimationReport, EstimationError> {
    config
        .validate()
        .map_err(|x| EstimationError::config(x.to_string()))?;
    let preprocessed =
        timestamp::preprocess_measurement(experiment.measurement(), &config.timestamp_handling)
            .map_err(EstimationError::invalid)?;
    let mut segment_reports = Vec::with_capacity(preprocessed.segments.len());
    for segment in &preprocessed.segments {
        let segment_measurement = slice_measurement(
            &preprocessed.measurement,
            segment.start_index,
            segment.end_index,
        )?;
        let mut segment_experiment = experiment.clone();
        segment_experiment.measurement_data = segment_measurement;
        let mut report = estimate_single_segment(
            &segment_experiment,
            channel,
            &calibration,
            config,
            context,
            filter,
        )?;
        for (index, point) in report.estimates.iter_mut().enumerate() {
            point.segment_id = segment.segment_index;
            point.original_row_index = preprocessed
                .original_indices
                .get(segment.start_index + index)
                .copied();
        }
        segment_reports.push(report);
    }

    let mut reports_iter = segment_reports.into_iter();
    let mut final_report = reports_iter.next().ok_or_else(|| {
        EstimationError::invalid("no valid timestamp segments remain for estimation")
    })?;
    for report in reports_iter {
        final_report.estimates.extend(report.estimates);
        final_report
            .diagnostics
            .innovations
            .extend(report.diagnostics.innovations);
        final_report.diagnostics.accepted_update_count += report.diagnostics.accepted_update_count;
        final_report.diagnostics.rejected_update_count += report.diagnostics.rejected_update_count;
        final_report.diagnostics.predict_only_count += report.diagnostics.predict_only_count;
        final_report.diagnostics.numerical_failures += report.diagnostics.numerical_failures;
        final_report.diagnostics.domain_excursion_count +=
            report.diagnostics.domain_excursion_count;
        final_report.warnings.extend(report.warnings);
    }
    final_report.timestamp_diagnostics = Some(preprocessed.diagnostics.clone());
    final_report.timestamp_policy = Some(preprocessed.applied_policy.clone());
    final_report.timestamp_segments = preprocessed.segments;
    final_report.skipped_timestamp_segments = preprocessed.skipped_segments;
    final_report.was_preprocessed = preprocessed.was_transformed;
    Ok(final_report)
}

fn estimate_single_segment(
    experiment: &ElectrochemicalExperiment,
    channel: &str,
    calibration: &StoredCalibrationObservationModel,
    config: &ResolvedEstimationConfig,
    context: EstimationContext<'_>,
    filter: FilterKind,
) -> Result<StateEstimationReport, EstimationError> {
    let calibration = Box::new(calibration.clone());
    let tau = resolve_tau(config, context.transient)?;
    let model = StateModel::new(config, tau.0, tau.1)?;
    let (obs, timestamp_diagnostics) = observations(experiment.measurement(), channel)?;
    let measurement_source_unit = experiment
        .measurement()
        .channel(channel)
        .map(|channel| channel.unit.clone())
        .unwrap_or_default();
    let mut environments = Vec::with_capacity(obs.len());
    let mut previous: Option<AlignedEnvironment> = None;
    for o in &obs {
        let mut e = align_experiment_with_polarization(
            experiment,
            o.timestamp_s,
            &config.environment,
            previous.as_ref(),
            &config.polarization,
        )?;
        resolve_standard_activity(
            &mut e,
            &calibration.model.configuration.activity,
            calibration.model.configuration.analyte.molar_mass_g_per_mol,
            calibration.model.ion_charge,
        )?;
        previous = Some(e.clone());
        environments.push(e);
    }
    let measurement_covariance = resolve_measurement_covariance(
        config,
        obs.iter()
            .find_map(|observation| observation.observation_variance_v2),
        context.signal,
        context.calibration_results,
    )?;
    let (initial_state, initial_covariance, initialization) = initialize_state(
        experiment,
        channel,
        &model,
        calibration.as_ref(),
        config,
        environments
            .first()
            .ok_or_else(|| EstimationError::invalid("no environment records"))?,
    )?;
    let mut observability = diagnose(
        &model,
        &initial_state,
        &environments,
        calibration.as_ref(),
        config,
    )?;
    let known_standard = environments.iter().any(|e| e.known_standard);
    let baseline_fixed = !model.has_baseline()
        || (config.initial_covariance.baseline_variance_v2 == 0.0
            && config.process_noise.baseline_variance_v2_per_s == 0.0);
    let condition_fixed = !model.has_condition()
        || (config.initial_covariance.condition_variance == 0.0
            && config.process_noise.condition_variance_per_s == 0.0);
    if model.has_condition()
        && !known_standard
        && context.eis_fit.is_none()
        && context.transient.is_none()
        && context.calibration_results.is_none()
        && !condition_fixed
    {
        observability.warnings.push(crate::estimation::state::EstimationWarning::new(crate::estimation::state::EstimationWarningKind::ConditionStateNotIdentifiable,"sensitivity/condition state requires a standard or independent auxiliary observation"));
        if config.auxiliary.condition_requires_auxiliary {
            return Err(EstimationError::config(
                "condition state is not identifiable without configured auxiliary information",
            ));
        }
    }
    if config.observability.reject_unobservable_model
        && observability.numerical_rank < model.dimension()
        && !known_standard
        && !model.has_condition()
        && !baseline_fixed
    {
        return Err(EstimationError::config(format!(
            "selected state model is unobservable: rank {} of {}",
            observability.numerical_rank,
            model.dimension()
        )));
    }
    let input = ekf::FilterInput {
        observations: &obs,
        environments: &environments,
        model: &model,
        calibration: calibration.as_ref(),
        config,
        initial_state,
        initial_covariance,
        measurement_covariance: &measurement_covariance,
        signal: context.signal,
        calibration_results: context.calibration_results,
    };
    let run = match filter {
        FilterKind::Ekf => ekf::run(input)?,
        FilterKind::Ukf => ukf::run(input)?,
    };
    let process = run.process_covariance.unwrap_or_else(|| {
        resolve_process_covariance(config, &model, 1.0)
            .expect("validated default process covariance")
            .1
    });
    let mut warnings = initialization.warnings.clone();
    warnings.extend(observability.warnings.clone());
    if matches!(config.state_model.kind, StateModelKind::Activity) {
        warnings.push(crate::estimation::state::EstimationWarning::new(
            crate::estimation::state::EstimationWarningKind::ModelDiscrepancy,
            "activity-only estimation assumes baseline and dynamic polarization effects are negligible or externally corrected",
        ));
    }
    for e in &environments {
        warnings.extend(e.warnings.clone());
    }
    if tau.2 {
        warnings.push(crate::estimation::state::EstimationWarning::new(
            crate::estimation::state::EstimationWarningKind::TransientPriorUnavailable,
            "configured transient prior was unavailable; configured tau was used",
        ));
    }
    Ok(StateEstimationReport {
        schema_version: 2,
        analysis_id: format!(
            "estimate:{}:{}",
            experiment.provenance.input_sha256, channel
        ),
        experiment_id: experiment.experiment_id.clone(),
        sensor_id: experiment.sensor_metadata.sensor_id.clone(),
        channel: channel.into(),
        measurement_source_unit,
        measurement_conversion:
            "potential converted to V; per-observation variance converted to V²".into(),
        filter,
        model: config.state_model.kind,
        state_definitions: model.definitions,
        initialization,
        process_covariance: process,
        measurement_covariance,
        observability,
        estimates: run.estimates,
        diagnostics: run.diagnostics,
        validation: None,
        configuration: config.clone(),
        provenance: experiment.provenance.clone(),
        timestamp_diagnostics: Some(timestamp_diagnostics),
        timestamp_policy: None,
        timestamp_segments: Vec::new(),
        skipped_timestamp_segments: Vec::new(),
        was_preprocessed: false,
        warnings,
    })
}

fn slice_measurement(
    measurement: &crate::domain::MultiChannelMeasurement,
    start: usize,
    end: usize,
) -> Result<crate::domain::MultiChannelMeasurement, EstimationError> {
    let time = measurement.time[start..end].to_vec();
    let channels = measurement
        .channels
        .iter()
        .map(|channel| crate::domain::MeasurementChannel {
            name: channel.name.clone(),
            unit: channel.unit.clone(),
            values: channel.values[start..end].to_vec(),
            variance: channel
                .variance
                .as_ref()
                .map(|variance| variance[start..end].to_vec()),
            sensor_id: channel.sensor_id.clone(),
            analyte_id: channel.analyte_id.clone(),
            metadata: channel.metadata.clone(),
        })
        .collect::<Vec<_>>();
    crate::domain::MultiChannelMeasurement::new(time, channels)
        .map_err(|error| EstimationError::invalid(error.to_string()))
}

fn resolve_tau(
    config: &ResolvedEstimationConfig,
    transient: Option<&TransientAnalysisReport>,
) -> Result<(f64, Option<f64>, bool), EstimationError> {
    if !matches!(
        config.state_model.kind,
        StateModelKind::ActivityBaselinePolarization | StateModelKind::Custom
    ) {
        return Ok((config.polarization.configured_tau_s, None, false));
    }
    let mut values = Vec::new();
    if matches!(
        config.polarization.tau_source,
        crate::estimation_config::TauSourceKind::Transient
    ) {
        if let Some(report) = transient {
            for event in &report.events {
                if let Some(selected) = event.selected_model {
                    if let Some(fit) = event
                        .candidate_fits
                        .iter()
                        .find(|x| x.model == selected && x.is_successful())
                    {
                        if let Some(parameter) = fit
                            .parameters
                            .iter()
                            .find(|p| p.name == config.polarization.transient_parameter)
                        {
                            if parameter.value.is_finite() && parameter.value > 0.0 {
                                values.push(parameter.value);
                            }
                        }
                    }
                }
            }
        }
    }
    let tau = match config.polarization.aggregation {
        crate::estimation_config::AggregationKind::First => values.first().copied(),
        crate::estimation_config::AggregationKind::Mean => {
            (!values.is_empty()).then_some(values.iter().sum::<f64>() / values.len() as f64)
        }
        crate::estimation_config::AggregationKind::Median => {
            let mut x = values.clone();
            x.sort_by(f64::total_cmp);
            if x.is_empty() {
                None
            } else {
                Some(if x.len() % 2 == 0 {
                    (x[x.len() / 2 - 1] + x[x.len() / 2]) / 2.0
                } else {
                    x[x.len() / 2]
                })
            }
        }
    };
    if let Some(t) = tau {
        return Ok((t, None, false));
    }
    if matches!(
        config.polarization.tau_source,
        crate::estimation_config::TauSourceKind::Configured
    ) {
        return Ok((
            config.polarization.configured_tau_s,
            config.polarization.tau_uncertainty_s,
            false,
        ));
    }
    Ok((
        config.polarization.configured_tau_s,
        config.polarization.tau_uncertainty_s,
        true,
    ))
}
