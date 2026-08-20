//! Phase-C sensor-health evidence preparation and deterministic evaluation.
//!
//! This module is crate-visible on purpose.  It owns the typed inputs and all
//! evaluators; the health runner only reads versioned artifacts and forwards
//! them through these APIs.

#![allow(clippy::collapsible_if)]

use crate::{
    domain::{ArtifactKind, ArtifactLineageCatalog, ArtifactLineageState, ScopeKey},
    evidence::{
        EvidenceAvailability, EvidenceBundle, EvidenceBundleBuilder, EvidenceDirection, EvidenceId,
        EvidenceQuantity, EvidenceRecord, EvidenceSourceClass, EvidenceSourceRef, EvidenceStrength,
        EvidenceTarget, EvidenceValidity, StrengthSource, ThresholdProvenance, ThresholdSource,
    },
    evidence_adapters::AdapterContext,
    health::{baseline::Context, error::HealthError},
    health_config::{
        ComparabilityConfig, EnvironmentalCovariate, LevelThreshold,
        LoadedPhaseCHealthEvidenceConfig, PhaseCHealthEvidenceConfig,
    },
    results::{
        CalibrationAnalysisReport, CausalStatus, HealthDimension, HealthEvidenceState,
        HealthInterpretationCategory, MechanismAnalysisReport, ModelAnalysisReport,
        OverallHealthStatus, PhaseCHealthDimensionAssessment, PhaseCHealthReasonCode,
        PhaseCSensorHealthEvidenceReport, SensorHealthBaseline, SignalAnalysisReport,
        StateEstimationReport, TransientAnalysisReport,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct PhaseCHealthInputs {
    signal: SignalAnalysisReport,
    baseline: Option<SensorHealthBaseline>,
    transient: Option<TransientAnalysisReport>,
    calibration: Option<CalibrationAnalysisReport>,
    estimation: Option<StateEstimationReport>,
    model: Option<ModelAnalysisReport>,
    mechanism: Option<MechanismAnalysisReport>,
    lineage_catalog: Option<ArtifactLineageCatalog>,
    current_context: Context,
    comparability: ComparabilityConfig,
}

#[derive(Debug, Clone)]
pub(crate) struct PhaseCEligibleInputs {
    signal: SignalAnalysisReport,
    baseline: Option<SensorHealthBaseline>,
    transient: Option<TransientAnalysisReport>,
    calibration: Option<CalibrationAnalysisReport>,
    estimation: Option<StateEstimationReport>,
    model: Option<ModelAnalysisReport>,
    mechanism: Option<MechanismAnalysisReport>,
    lineage_catalog: Option<ArtifactLineageCatalog>,
    current_context: Context,
    comparability: ComparabilityConfig,
    transient_compatible: bool,
    calibration_compatible: bool,
    estimation_compatible: bool,
    model_compatible: bool,
    mechanism_compatible: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn assemble_phase_c_inputs(
    signal: SignalAnalysisReport,
    baseline: Option<SensorHealthBaseline>,
    transient: Option<TransientAnalysisReport>,
    calibration: Option<CalibrationAnalysisReport>,
    estimation: Option<StateEstimationReport>,
    model: Option<ModelAnalysisReport>,
    mechanism: Option<MechanismAnalysisReport>,
    lineage_catalog: Option<ArtifactLineageCatalog>,
    current_context: Context,
    comparability: ComparabilityConfig,
) -> PhaseCHealthInputs {
    PhaseCHealthInputs {
        signal,
        baseline,
        transient,
        calibration,
        estimation,
        model,
        mechanism,
        lineage_catalog,
        current_context,
        comparability,
    }
}

pub(crate) fn validate_source_compatibility(
    inputs: &PhaseCHealthInputs,
    catalog: Option<&ArtifactLineageCatalog>,
) -> Result<PhaseCEligibleInputs, HealthError> {
    let compatible =
        |lineage: &ArtifactLineageState| scope_compatible(&inputs.signal.lineage, lineage);
    Ok(PhaseCEligibleInputs {
        signal: inputs.signal.clone(),
        baseline: inputs.baseline.clone(),
        transient: inputs.transient.clone(),
        calibration: inputs.calibration.clone(),
        estimation: inputs.estimation.clone(),
        model: inputs.model.clone(),
        mechanism: inputs.mechanism.clone(),
        lineage_catalog: catalog.cloned().or_else(|| inputs.lineage_catalog.clone()),
        current_context: inputs.current_context.clone(),
        comparability: inputs.comparability.clone(),
        transient_compatible: inputs.transient.as_ref().is_none_or(|item| {
            compatible(&item.lineage)
                && inputs
                    .signal
                    .experiment_id
                    .as_deref()
                    .is_none_or(|id| id == item.experiment_id)
                && inputs.signal.channel == item.channel
        }),
        calibration_compatible: inputs
            .calibration
            .as_ref()
            .is_none_or(|item| compatible(&item.lineage)),
        estimation_compatible: inputs
            .estimation
            .as_ref()
            .is_none_or(|item| compatible(&item.lineage)),
        model_compatible: inputs
            .model
            .as_ref()
            .is_none_or(|item| compatible(&item.lineage)),
        mechanism_compatible: inputs.mechanism.as_ref().is_none_or(|item| {
            item.schema_version == 4
                && matches!(
                    (&inputs.signal.lineage, &item.lineage),
                    (
                        ArtifactLineageState::Known { identity: left, .. },
                        ArtifactLineageState::Known { identity: right, .. },
                    ) if left.experiment_scope == right.experiment_scope
                        && left.sensor_scope == right.sensor_scope
                        && left.channel_scope == right.channel_scope
                )
        }),
    })
}

pub(crate) fn prepare_phase_c_evidence(
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> Result<EvidenceBundle, HealthError> {
    let _mechanism_is_compatible = inputs.mechanism_compatible;
    let catalog = inputs.lineage_catalog.clone().unwrap_or_default();
    let mut builder = EvidenceBundleBuilder::new(
        scope_from_lineage(&inputs.signal.lineage),
        scope_key_from_lineage(&inputs.signal.lineage, true),
        scope_key_from_lineage(&inputs.signal.lineage, false),
        catalog,
    );
    let context = AdapterContext::from_artifact(
        &inputs.signal,
        ArtifactKind::SignalAnalysis,
        &inputs.signal.lineage,
    );
    add_scalar(
        &mut builder,
        "signal.descriptive.rms",
        HealthDimension::SignalIntegrity,
        &context,
        "$.descriptive.rms",
        inputs.signal.descriptive.rms,
        &inputs.signal.unit,
    );
    add_scalar(
        &mut builder,
        "signal.descriptive.robust_standard_deviation",
        HealthDimension::SignalIntegrity,
        &context,
        "$.descriptive.robust_standard_deviation",
        inputs.signal.descriptive.robust_standard_deviation,
        &inputs.signal.unit,
    );
    add_scalar(
        &mut builder,
        "signal.spikes.flagged_fraction",
        HealthDimension::SignalIntegrity,
        &context,
        "$.spikes.flagged_fraction",
        inputs.signal.spikes.flagged_fraction,
        "1",
    );
    if let Some(drift) = inputs
        .signal
        .drift
        .iter()
        .find(|row| matches!(row.model, crate::results::DriftModelKind::TheilSen))
    {
        add_scalar(
            &mut builder,
            "signal.drift.theil_sen.slope_v_per_s",
            HealthDimension::SignalIntegrity,
            &context,
            "$.drift[theil_sen].slope_v_per_s",
            drift.slope_v_per_s,
            "V/s",
        );
    }
    add_scalar(
        &mut builder,
        "signal.sampling.finite_sample_count",
        HealthDimension::DataQuality,
        &context,
        "$.sampling.finite_sample_count",
        Some(inputs.signal.sampling.finite_sample_count as f64),
        "1",
    );
    add_scalar(
        &mut builder,
        "signal.sampling.missing_fraction",
        HealthDimension::DataQuality,
        &context,
        "$.sampling.missing_fraction",
        inputs.signal.sampling.missing_fraction,
        "1",
    );
    add_scalar(
        &mut builder,
        "signal.sampling.interval_cv",
        HealthDimension::DataQuality,
        &context,
        "$.sampling.interval_cv",
        inputs.signal.sampling.interval_cv,
        "1",
    );
    add_scalar(
        &mut builder,
        "signal.sampling.duplicate_timestamps",
        HealthDimension::DataQuality,
        &context,
        "$.sampling.duplicate_timestamps",
        Some(inputs.signal.sampling.duplicate_timestamps as f64),
        "1",
    );
    add_scalar(
        &mut builder,
        "signal.sampling.non_monotonic_timestamps",
        HealthDimension::DataQuality,
        &context,
        "$.sampling.non_monotonic_timestamps",
        Some(inputs.signal.sampling.non_monotonic_timestamps as f64),
        "1",
    );
    add_scalar(
        &mut builder,
        "signal.sampling.interpolation_gap_exceeded",
        HealthDimension::DataQuality,
        &context,
        "$.sampling.interpolation_gap_exceeded",
        Some(inputs.signal.sampling.interpolation_gap_exceeded as u8 as f64),
        "1",
    );
    if let Some(calibration) = &inputs.calibration {
        let context = AdapterContext::from_artifact(
            calibration,
            ArtifactKind::CalibrationAnalysis,
            &calibration.lineage,
        );
        let selected = calibration.selected_model.and_then(|kind| {
            calibration
                .candidate_models
                .iter()
                .find(|model| model.model_kind == kind)
        });
        if let Some(model) = selected {
            add_scalar(
                &mut builder,
                "calibration.selected.slope_efficiency",
                HealthDimension::CalibrationHealth,
                &context,
                "$.selected.slope_efficiency",
                model.slope_efficiency,
                "1",
            );
            add_scalar(
                &mut builder,
                "calibration.selected.rmse_v",
                HealthDimension::CalibrationHealth,
                &context,
                "$.selected.statistics.rmse_v",
                model.statistics.rmse_v,
                "V",
            );
        }
        add_scalar(
            &mut builder,
            "calibration.validation.prediction_bias_v",
            HealthDimension::CalibrationHealth,
            &context,
            "$.validation.prediction_bias_v",
            calibration
                .validation
                .as_ref()
                .and_then(|row| row.prediction_bias_v),
            "V",
        );
        add_scalar(
            &mut builder,
            "calibration.hysteresis.mean_hysteresis_v",
            HealthDimension::CalibrationHealth,
            &context,
            "$.hysteresis.mean_hysteresis_v",
            calibration
                .hysteresis
                .as_ref()
                .and_then(|row| row.mean_hysteresis_v),
            "V",
        );
    }
    if let Some(transient) = &inputs.transient {
        let context = AdapterContext::from_artifact(
            transient,
            ArtifactKind::TransientAnalysis,
            &transient.lineage,
        );
        // The configured selector names the producer ordinal, while the
        // EvidenceId uses the field's serialized-array position exactly
        // as required by the Phase-C wire contract.
        for (index, event) in transient.events.iter().enumerate() {
            if event.event_index != config.dynamic_response_health.selected_event_index {
                continue;
            }
            let Some(model) = event.selected_model else {
                continue;
            };
            let matching_successful_fits = event
                .candidate_fits
                .iter()
                .filter(|fit| fit.model == model && fit.is_successful())
                .collect::<Vec<_>>();
            if matching_successful_fits.len() != 1 {
                continue;
            }
            let fit = matching_successful_fits[0];
            add_scalar(
                &mut builder,
                format!("transient.event.{index}.tau_fast_s"),
                HealthDimension::DynamicResponseHealth,
                &context,
                format!("$.events[{index}].selected_fit.derived_features.tau_fast_s"),
                fit.derived_features.tau_fast_s,
                "s",
            );
            add_scalar(
                &mut builder,
                format!("transient.event.{index}.tau_slow_s"),
                HealthDimension::DynamicResponseHealth,
                &context,
                format!("$.events[{index}].selected_fit.derived_features.tau_slow_s"),
                fit.derived_features.tau_slow_s,
                "s",
            );
            add_scalar(
                &mut builder,
                format!("transient.event.{index}.time_to_90_percent_s"),
                HealthDimension::DynamicResponseHealth,
                &context,
                format!("$.events[{index}].selected_fit.derived_features.time_to_90_percent_s"),
                fit.derived_features.time_to_90_percent_s,
                "s",
            );
            add_scalar(
                &mut builder,
                format!("transient.event.{index}.response_amplitude_v"),
                HealthDimension::DynamicResponseHealth,
                &context,
                format!(
                    "$.events[{index}].selected_fit.derived_features.total_response_amplitude_v"
                ),
                fit.derived_features.total_response_amplitude_v,
                "V",
            );
            add_scalar(
                &mut builder,
                format!("transient.event.{index}.fit_rmse_v"),
                HealthDimension::DynamicResponseHealth,
                &context,
                format!("$.events[{index}].selected_fit.statistics.rmse_v"),
                fit.statistics.rmse_v,
                "V",
            );
        }
    }
    if let Some(estimation) = &inputs.estimation {
        let context = AdapterContext::from_artifact(
            estimation,
            ArtifactKind::StateEstimation,
            &estimation.lineage,
        );
        for (index, point) in estimation.estimates.iter().enumerate() {
            add_scalar(
                &mut builder,
                format!("estimation.point.{index}.unexplained_residual_v"),
                HealthDimension::ModelConsistency,
                &context,
                format!("$.estimates[{index}].unexplained_residual_v"),
                point.unexplained_residual_v,
                "V",
            );
            add_scalar(
                &mut builder,
                format!("estimation.point.{index}.environment.temperature_k"),
                HealthDimension::EnvironmentalRobustness,
                &context,
                format!("$.estimates[{index}].environmental_context.temperature_k"),
                point.environmental_context.temperature_k,
                "K",
            );
            add_scalar(
                &mut builder,
                format!("estimation.point.{index}.environment.conductivity_s_per_m"),
                HealthDimension::EnvironmentalRobustness,
                &context,
                format!("$.estimates[{index}].environmental_context.conductivity_s_per_m"),
                point.environmental_context.conductivity_s_per_m,
                "S/m",
            );
            add_scalar(
                &mut builder,
                format!("estimation.point.{index}.environment.ionic_strength_mol_l"),
                HealthDimension::EnvironmentalRobustness,
                &context,
                format!("$.estimates[{index}].environmental_context.ionic_strength_mol_l"),
                point.environmental_context.ionic_strength_mol_l,
                "mol/L",
            );
            let flow_unit = point
                .environmental_context
                .source_records
                .iter()
                .filter_map(|record| {
                    record
                        .source_unit
                        .as_deref()
                        .filter(|unit| !unit.is_empty())
                })
                .next()
                .unwrap_or("1");
            add_scalar(
                &mut builder,
                format!("estimation.point.{index}.environment.flow"),
                HealthDimension::EnvironmentalRobustness,
                &context,
                format!("$.estimates[{index}].environmental_context.flow"),
                point.environmental_context.flow,
                flow_unit,
            );
        }
        let observable = &estimation.observability;
        add_scalar(
            &mut builder,
            "estimation.observability.numerical_rank",
            HealthDimension::Observability,
            &context,
            "$.observability.numerical_rank",
            Some(observable.numerical_rank as f64),
            "1",
        );
        add_scalar(
            &mut builder,
            "estimation.observability.state_count",
            HealthDimension::Observability,
            &context,
            "$.observability.state_count",
            Some(observable.state_count as f64),
            "1",
        );
        add_scalar(
            &mut builder,
            "estimation.observability.condition_number",
            HealthDimension::Observability,
            &context,
            "$.observability.condition_number",
            observable.condition_number,
            "1",
        );
    }
    if let Some(model) = &inputs.model {
        let context =
            AdapterContext::from_artifact(model, ArtifactKind::ModelAnalysis, &model.lineage);
        for (index, point) in model.points.iter().enumerate() {
            add_scalar(
                &mut builder,
                format!("model.point.{index}.unexplained_residual_v"),
                HealthDimension::ModelConsistency,
                &context,
                format!("$.points[{index}].unexplained_residual_v"),
                point.unexplained_residual_v,
                "V",
            );
            add_scalar(
                &mut builder,
                format!("model.point.{index}.validity.is_valid"),
                HealthDimension::ModelConsistency,
                &context,
                format!("$.points[{index}].validity.is_valid"),
                Some(if point.validity.is_valid { 1.0 } else { 0.0 }),
                "1",
            );
            add_scalar(
                &mut builder,
                format!("model.point.{index}.uncertainty.standard_error_v"),
                HealthDimension::UncertaintyHealth,
                &context,
                format!("$.points[{index}].uncertainty.standard_error_v"),
                point.uncertainty.standard_error_v,
                "V",
            );
            add_scalar(
                &mut builder,
                format!("model.point.{index}.uncertainty.total_variance_v2"),
                HealthDimension::UncertaintyHealth,
                &context,
                format!("$.points[{index}].uncertainty.total_variance_v2"),
                point.uncertainty.total_variance_v2,
                "V^2",
            );
        }
    }
    if let Some(mechanism) = &inputs.mechanism {
        if inputs.mechanism_compatible && mechanism.schema_version == 4 {
            let context = AdapterContext::from_artifact(
                mechanism,
                ArtifactKind::MechanismAnalysis,
                &mechanism.lineage,
            );
            for binding in &config.phase_b_hypothesis_bindings {
                if let Some((index, _)) = mechanism
                    .hypothesis_assessments
                    .iter()
                    .enumerate()
                    .find(|(_, row)| row.current.hypothesis_id == binding.hypothesis_id)
                {
                    add_marker(
                        &mut builder,
                        format!("mechanism.hypothesis.{}.assessment", binding.hypothesis_id),
                        binding.health_dimension,
                        &context,
                        format!("$.hypothesis_assessments[{index}].current"),
                    );
                }
            }
        }
    }
    let mut bundle = builder
        .build()
        .map_err(|error| HealthError::InvalidEvidence {
            source_name: "phase_c".into(),
            field: error.to_string(),
        })?;
    for record in &mut bundle.records {
        record.threshold_provenance = threshold_provenance_for(&record.evidence_id.0, config);
    }
    bundle
        .validate()
        .map_err(|error| HealthError::InvalidEvidence {
            source_name: "phase_c".into(),
            field: error.to_string(),
        })?;
    Ok(bundle)
}

pub(crate) fn evaluate_data_quality(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> PhaseCHealthDimensionAssessment {
    let s = &inputs.signal.sampling;
    let missing = s.missing_fraction;
    let interval = s.interval_cv;
    if missing.is_none() || interval.is_none() {
        return finding(
            HealthDimension::DataQuality,
            OverallHealthStatus::DataQualityInsufficient,
            HealthEvidenceState::PoorDataQuality,
            HealthInterpretationCategory::ObservedBehavior,
            CausalStatus::Indeterminate,
            vec![PhaseCHealthReasonCode::RequiredQuantityAbsent],
            ids_for(bundle, HealthDimension::DataQuality),
        );
    }
    if !missing.is_some_and(f64::is_finite) || !interval.is_some_and(f64::is_finite) {
        return dqi(
            HealthDimension::DataQuality,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::InvalidQuantity,
            ids_for(bundle, HealthDimension::DataQuality),
        );
    }
    let bad = s.finite_sample_count < config.data_quality.minimum_finite_samples
        || missing.unwrap() > config.data_quality.maximum_missing_fraction
        || interval.unwrap() > config.data_quality.maximum_interval_cv
        || s.duplicate_timestamps > config.data_quality.maximum_duplicate_timestamps
        || s.non_monotonic_timestamps > config.data_quality.maximum_non_monotonic_timestamps
        || (!config.data_quality.allow_interpolation_gap_exceeded && s.interpolation_gap_exceeded);
    finding(
        HealthDimension::DataQuality,
        if bad {
            OverallHealthStatus::DataQualityInsufficient
        } else {
            OverallHealthStatus::WithinBaseline
        },
        if bad {
            HealthEvidenceState::PoorDataQuality
        } else {
            HealthEvidenceState::AdequateEvidence
        },
        HealthInterpretationCategory::ObservedBehavior,
        if bad {
            CausalStatus::Indeterminate
        } else {
            CausalStatus::Observed
        },
        vec![if bad {
            PhaseCHealthReasonCode::QualityGateFailed
        } else {
            PhaseCHealthReasonCode::ThresholdWithinLimit
        }],
        ids_for(bundle, HealthDimension::DataQuality),
    )
}

pub(crate) fn evaluate_dimension(
    dimension: HealthDimension,
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> Result<PhaseCHealthDimensionAssessment, HealthError> {
    Ok(match dimension {
        HealthDimension::DataQuality => evaluate_data_quality(bundle, inputs, config),
        HealthDimension::SignalIntegrity => evaluate_signal(bundle, inputs, config),
        HealthDimension::CalibrationHealth => evaluate_calibration(bundle, inputs, config),
        HealthDimension::DynamicResponseHealth => evaluate_dynamic(bundle, inputs, config),
        HealthDimension::ReferenceStability => finding(
            dimension,
            OverallHealthStatus::Indeterminate,
            HealthEvidenceState::NoEvidence,
            HealthInterpretationCategory::ObservedBehavior,
            CausalStatus::Indeterminate,
            vec![PhaseCHealthReasonCode::ReferenceAnchorUnavailable],
            Vec::new(),
        ),
        HealthDimension::EnvironmentalRobustness => evaluate_environment(bundle, inputs, config),
        HealthDimension::ModelConsistency => evaluate_model_consistency(bundle, inputs, config),
        HealthDimension::Observability => evaluate_observability(bundle, inputs, config),
        HealthDimension::UncertaintyHealth => evaluate_uncertainty(bundle, inputs, config),
    })
}

pub(crate) fn evaluate_all_dimensions(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> Result<Vec<PhaseCHealthDimensionAssessment>, HealthError> {
    HealthDimension::ALL
        .into_iter()
        .map(|dimension| evaluate_dimension(dimension, bundle, inputs, config))
        .collect()
}

pub(crate) fn derive_interpretation_category(
    assessment: &mut PhaseCHealthDimensionAssessment,
    mechanism: Option<&MechanismAnalysisReport>,
) -> Result<(), HealthError> {
    let Some(mechanism) = mechanism else {
        return Ok(());
    };
    if !matches!(
        assessment.dimension,
        HealthDimension::SignalIntegrity
            | HealthDimension::CalibrationHealth
            | HealthDimension::DynamicResponseHealth
    ) || !matches!(
        assessment.status,
        OverallHealthStatus::Degraded | OverallHealthStatus::Critical
    ) || assessment.evidence_state != HealthEvidenceState::AdequateEvidence
    {
        return Ok(());
    }
    if let Some(row) = mechanism.hypothesis_assessments.iter().find(|row| {
        matches!(
            row.current.evidence_level,
            crate::mechanism::promotion::HypothesisEvidenceLevel::Hypothesized
                | crate::mechanism::promotion::HypothesisEvidenceLevel::ExperimentallySupported
                | crate::mechanism::promotion::HypothesisEvidenceLevel::ValidatedForDomain
        ) && !row.current.reason_codes.contains(
            &crate::mechanism::promotion::PhaseBHypothesisReasonCode::CriticalContradiction,
        )
    }) {
        assessment.interpretation_category =
            HealthInterpretationCategory::PossiblePhysicalDegradation;
        assessment.source_evidence_ids.push(EvidenceId(format!(
            "mechanism.hypothesis.{}.assessment",
            row.current.hypothesis_id
        )));
        canonicalize(assessment);
    }
    Ok(())
}

pub(crate) fn derive_causal_status(
    assessment: &mut PhaseCHealthDimensionAssessment,
    _bundle: &EvidenceBundle,
    mechanism: Option<&MechanismAnalysisReport>,
    _config: &PhaseCHealthEvidenceConfig,
) -> Result<(), HealthError> {
    if let Some(row) = mechanism.and_then(|report| {
        report.hypothesis_assessments.iter().find(|row| {
            row.current.evidence_level
                == crate::mechanism::promotion::HypothesisEvidenceLevel::Contradicted
                || row.current.reason_codes.contains(
                    &crate::mechanism::promotion::PhaseBHypothesisReasonCode::CriticalContradiction,
                )
        })
    }) {
        assessment.causal_status = CausalStatus::Indeterminate;
        assessment
            .reason_codes
            .push(PhaseCHealthReasonCode::MechanismContradicted);
        assessment.source_evidence_ids.push(EvidenceId(format!(
            "mechanism.hypothesis.{}.assessment",
            row.current.hypothesis_id
        )));
        canonicalize(assessment);
        return Ok(());
    }
    assessment.causal_status = if matches!(
        assessment.evidence_state,
        HealthEvidenceState::AdequateEvidence | HealthEvidenceState::ContradictoryEvidence
    ) {
        CausalStatus::Observed
    } else {
        CausalStatus::Indeterminate
    };
    Ok(())
}

pub(crate) fn compose_phase_c_report(
    config: &LoadedPhaseCHealthEvidenceConfig,
    mut dimensions: Vec<PhaseCHealthDimensionAssessment>,
    bundle: EvidenceBundle,
) -> Result<PhaseCSensorHealthEvidenceReport, HealthError> {
    if dimensions.len() != HealthDimension::ALL.len()
        || dimensions
            .iter()
            .zip(HealthDimension::ALL)
            .any(|(row, expected)| row.dimension != expected)
    {
        return Err(HealthError::ReportAssembly {
            message: "Phase C requires exactly nine declaration-order dimensions".into(),
        });
    }
    bundle
        .validate()
        .map_err(|error| HealthError::ReportAssembly {
            message: format!("invalid Phase-C evidence bundle: {error}"),
        })?;
    for row in &dimensions {
        validate_dimension_assessment(row, &bundle)?;
    }
    let overall_status = if dimensions
        .iter()
        .any(|row| row.status == OverallHealthStatus::Critical)
    {
        OverallHealthStatus::Critical
    } else if dimensions
        .iter()
        .any(|row| row.status == OverallHealthStatus::Degraded)
    {
        OverallHealthStatus::Degraded
    } else if dimensions
        .iter()
        .any(|row| row.status == OverallHealthStatus::Watch)
    {
        OverallHealthStatus::Watch
    } else if dimensions
        .iter()
        .any(|row| row.status == OverallHealthStatus::DataQualityInsufficient)
    {
        OverallHealthStatus::DataQualityInsufficient
    } else if dimensions
        .iter()
        .any(|row| row.status == OverallHealthStatus::Indeterminate)
    {
        OverallHealthStatus::Indeterminate
    } else {
        OverallHealthStatus::WithinBaseline
    };
    let positive = dimensions
        .iter()
        .filter(|row| {
            matches!(
                row.status,
                OverallHealthStatus::Watch
                    | OverallHealthStatus::Degraded
                    | OverallHealthStatus::Critical
            )
        })
        .collect::<Vec<_>>();
    let mut categories = Vec::new();
    for category in positive.iter().map(|row| row.interpretation_category) {
        if !categories.contains(&category) {
            categories.push(category);
        }
    }
    let causal = positive
        .iter()
        .map(|row| row.causal_status)
        .min_by_key(|status| status.strength())
        .unwrap_or(CausalStatus::Indeterminate);
    for row in &mut dimensions {
        canonicalize(row);
    }
    Ok(PhaseCSensorHealthEvidenceReport {
        config_schema_version: config.config.schema_version,
        config_sha256: config.config_sha256.clone(),
        dimension_assessments: dimensions,
        overall_status,
        overall_interpretation_categories: categories,
        overall_causal_status: causal,
        evidence_bundle: bundle,
    })
}

fn validate_dimension_assessment(
    row: &PhaseCHealthDimensionAssessment,
    bundle: &EvidenceBundle,
) -> Result<(), HealthError> {
    let state_matches_status = match row.evidence_state {
        HealthEvidenceState::AdequateEvidence => matches!(
            row.status,
            OverallHealthStatus::WithinBaseline
                | OverallHealthStatus::Watch
                | OverallHealthStatus::Degraded
                | OverallHealthStatus::Critical
        ),
        HealthEvidenceState::NoEvidence | HealthEvidenceState::InsufficientEvidence => {
            row.status == OverallHealthStatus::Indeterminate
        }
        HealthEvidenceState::PoorDataQuality => {
            row.status == OverallHealthStatus::DataQualityInsufficient
        }
        HealthEvidenceState::ContradictoryEvidence => matches!(
            row.status,
            OverallHealthStatus::Watch
                | OverallHealthStatus::Degraded
                | OverallHealthStatus::Critical
        ),
    };
    if !state_matches_status {
        return Err(HealthError::ReportAssembly {
            message: format!(
                "{} has incompatible status and evidence_state",
                serde_json::to_string(&row.dimension).unwrap_or_default()
            ),
        });
    }
    if row.interpretation_category == HealthInterpretationCategory::PossiblePhysicalDegradation
        && (!matches!(
            row.dimension,
            HealthDimension::SignalIntegrity
                | HealthDimension::CalibrationHealth
                | HealthDimension::DynamicResponseHealth
        ) || !matches!(
            row.status,
            OverallHealthStatus::Degraded | OverallHealthStatus::Critical
        ) || row.evidence_state != HealthEvidenceState::AdequateEvidence)
    {
        return Err(HealthError::ReportAssembly {
            message: "possible_physical_degradation has an invalid Phase-C predicate".into(),
        });
    }
    if row.reason_codes.is_empty()
        || row
            .reason_codes
            .get(1..)
            .is_some_and(|secondary| secondary.contains(&row.reason_codes[0]))
        || row.reason_codes.get(1..).is_some_and(|secondary| {
            secondary
                .windows(2)
                .any(|pair| pair[0].as_str() >= pair[1].as_str())
        })
    {
        return Err(HealthError::ReportAssembly {
            message: "Phase-C reason codes are not canonical".into(),
        });
    }
    let known_ids = bundle
        .records
        .iter()
        .map(|record| &record.evidence_id)
        .collect::<std::collections::BTreeSet<_>>();
    if row
        .source_evidence_ids
        .iter()
        .chain(row.excluded_evidence_ids.iter())
        .any(|id| !known_ids.contains(id))
    {
        return Err(HealthError::ReportAssembly {
            message: "Phase-C finding references evidence that is not in the bundle".into(),
        });
    }
    Ok(())
}

pub(crate) fn populate_consumed_artifact_ids(
    dimensions: &mut [PhaseCHealthDimensionAssessment],
    bundle: &EvidenceBundle,
) {
    for dimension in dimensions {
        let mut ids = Vec::new();
        for evidence_id in &dimension.source_evidence_ids {
            if let Some(record) = bundle
                .records
                .iter()
                .find(|record| &record.evidence_id == evidence_id)
                && let crate::evidence::EvidenceArtifactSource::Known { artifact_id, .. } =
                    &record.source.artifact
            {
                ids.push(artifact_id.clone());
            }
        }
        dimension.source_artifact_ids = ids;
        canonicalize(dimension);
    }
}

pub(crate) fn consumed_lineage_sources(
    inputs: &PhaseCEligibleInputs,
    dimensions: &[PhaseCHealthDimensionAssessment],
) -> Vec<(ArtifactLineageState, crate::domain::ArtifactDependencyRole)> {
    let consumed = |kind: ArtifactKind| {
        dimensions.iter().any(|dimension| {
        dimension.source_artifact_ids.iter().any(|artifact_id| match kind {
            ArtifactKind::SignalAnalysis => matches!(&inputs.signal.lineage, ArtifactLineageState::Known { identity, .. } if identity.artifact_id == *artifact_id),
            ArtifactKind::HealthBaseline => inputs.baseline.as_ref().is_some_and(|item| matches!(&item.lineage, ArtifactLineageState::Known { identity, .. } if identity.artifact_id == *artifact_id)),
            ArtifactKind::TransientAnalysis => inputs.transient.as_ref().is_some_and(|item| matches!(&item.lineage, ArtifactLineageState::Known { identity, .. } if identity.artifact_id == *artifact_id)),
            ArtifactKind::CalibrationAnalysis => inputs.calibration.as_ref().is_some_and(|item| matches!(&item.lineage, ArtifactLineageState::Known { identity, .. } if identity.artifact_id == *artifact_id)),
            ArtifactKind::StateEstimation => inputs.estimation.as_ref().is_some_and(|item| matches!(&item.lineage, ArtifactLineageState::Known { identity, .. } if identity.artifact_id == *artifact_id)),
            ArtifactKind::ModelAnalysis => inputs.model.as_ref().is_some_and(|item| matches!(&item.lineage, ArtifactLineageState::Known { identity, .. } if identity.artifact_id == *artifact_id)),
            ArtifactKind::MechanismAnalysis => inputs.mechanism.as_ref().is_some_and(|item| matches!(&item.lineage, ArtifactLineageState::Known { identity, .. } if identity.artifact_id == *artifact_id)),
            _ => false,
        })
    })
    };
    let mut sources = vec![(
        inputs.signal.lineage.clone(),
        crate::domain::ArtifactDependencyRole::DerivedFrom,
    )];
    // Baseline distributions are comparison context rather than scalar A1
    // evidence.  A selected DynamicResponse source nevertheless consumes the
    // baseline as a `Prior` dependency.
    let baseline_consumed = dimensions.iter().any(|dimension| {
        dimension.dimension == HealthDimension::DynamicResponseHealth
            && !dimension.source_evidence_ids.is_empty()
    });
    if consumed(ArtifactKind::HealthBaseline) || baseline_consumed {
        if let Some(item) = &inputs.baseline {
            sources.push((
                item.lineage.clone(),
                crate::domain::ArtifactDependencyRole::Prior,
            ));
        }
    }
    for (kind, lineage) in [
        (
            ArtifactKind::TransientAnalysis,
            inputs.transient.as_ref().map(|item| &item.lineage),
        ),
        (
            ArtifactKind::CalibrationAnalysis,
            inputs.calibration.as_ref().map(|item| &item.lineage),
        ),
        (
            ArtifactKind::StateEstimation,
            inputs.estimation.as_ref().map(|item| &item.lineage),
        ),
        (
            ArtifactKind::ModelAnalysis,
            inputs.model.as_ref().map(|item| &item.lineage),
        ),
        (
            ArtifactKind::MechanismAnalysis,
            inputs.mechanism.as_ref().map(|item| &item.lineage),
        ),
    ] {
        if consumed(kind) {
            if let Some(lineage) = lineage {
                sources.push((
                    lineage.clone(),
                    crate::domain::ArtifactDependencyRole::TransformationInput,
                ));
            }
        }
    }
    sources
}

fn evaluate_signal(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> PhaseCHealthDimensionAssessment {
    let potential_scale = match inputs.signal.unit.as_str() {
        "V" => 1.0,
        "mV" => 1.0e-3,
        "µV" => 1.0e-6,
        _ => {
            return dqi(
                HealthDimension::SignalIntegrity,
                HealthInterpretationCategory::ObservedBehavior,
                PhaseCHealthReasonCode::UnitMismatch,
                ids_for(bundle, HealthDimension::SignalIntegrity),
            );
        }
    };
    let values = [
        (
            inputs
                .signal
                .descriptive
                .rms
                .map(|value| value * potential_scale),
            &config.signal_integrity.maximum_rms_noise_v,
        ),
        (
            inputs
                .signal
                .descriptive
                .robust_standard_deviation
                .map(|value| value * potential_scale),
            &config
                .signal_integrity
                .maximum_robust_noise_standard_deviation_v,
        ),
        (
            inputs.signal.spikes.flagged_fraction,
            &config.signal_integrity.maximum_spike_fraction,
        ),
        (
            inputs
                .signal
                .drift
                .iter()
                .find(|row| matches!(row.model, crate::results::DriftModelKind::TheilSen))
                .and_then(|row| row.slope_v_per_s)
                .map(|value| value.abs() * potential_scale),
            &config.signal_integrity.maximum_absolute_drift_v_per_s,
        ),
    ];
    if values.iter().any(|(value, _)| value.is_none()) {
        return dqi(
            HealthDimension::SignalIntegrity,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::RequiredQuantityAbsent,
            ids_for(bundle, HealthDimension::SignalIntegrity),
        );
    }
    if values.iter().any(|(value, _)| !value.unwrap().is_finite()) {
        return dqi(
            HealthDimension::SignalIntegrity,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::InvalidQuantity,
            ids_for(bundle, HealthDimension::SignalIntegrity),
        );
    }
    let status = values
        .into_iter()
        .map(|(value, threshold)| severity(value.unwrap(), threshold))
        .max_by_key(|status| status_rank(*status))
        .unwrap();
    normal(
        HealthDimension::SignalIntegrity,
        status,
        HealthInterpretationCategory::ObservedBehavior,
        ids_for(bundle, HealthDimension::SignalIntegrity),
    )
}

fn evaluate_calibration(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> PhaseCHealthDimensionAssessment {
    let Some(calibration) = &inputs.calibration else {
        return absent(
            HealthDimension::CalibrationHealth,
            HealthInterpretationCategory::CalibrationIssue,
        );
    };
    if !inputs.calibration_compatible {
        return incompatible_with_ids(
            HealthDimension::CalibrationHealth,
            HealthInterpretationCategory::CalibrationIssue,
            ids_for(bundle, HealthDimension::CalibrationHealth),
        );
    }
    let Some(selected) = calibration.selected_model.and_then(|kind| {
        calibration
            .candidate_models
            .iter()
            .find(|row| row.model_kind == kind)
    }) else {
        return dqi(
            HealthDimension::CalibrationHealth,
            HealthInterpretationCategory::CalibrationIssue,
            PhaseCHealthReasonCode::RequiredQuantityAbsent,
            Vec::new(),
        );
    };
    let values = [
        (
            selected.slope_efficiency.map(|v| (1.0 - v).abs()),
            &config
                .calibration_health
                .maximum_absolute_slope_efficiency_error,
        ),
        (
            selected.statistics.rmse_v,
            &config.calibration_health.maximum_rmse_v,
        ),
        (
            calibration
                .validation
                .as_ref()
                .and_then(|v| v.prediction_bias_v)
                .map(f64::abs),
            &config.calibration_health.maximum_absolute_prediction_bias_v,
        ),
        (
            calibration
                .hysteresis
                .as_ref()
                .and_then(|v| v.mean_hysteresis_v)
                .map(f64::abs),
            &config.calibration_health.maximum_hysteresis_v,
        ),
    ];
    if values.iter().any(|(v, _)| v.is_none()) {
        return dqi(
            HealthDimension::CalibrationHealth,
            HealthInterpretationCategory::CalibrationIssue,
            PhaseCHealthReasonCode::RequiredQuantityAbsent,
            ids_for(bundle, HealthDimension::CalibrationHealth),
        );
    }
    if values.iter().any(|(v, _)| !v.unwrap().is_finite()) {
        return dqi(
            HealthDimension::CalibrationHealth,
            HealthInterpretationCategory::CalibrationIssue,
            PhaseCHealthReasonCode::InvalidQuantity,
            ids_for(bundle, HealthDimension::CalibrationHealth),
        );
    }
    normal(
        HealthDimension::CalibrationHealth,
        values
            .into_iter()
            .map(|(v, t)| severity(v.unwrap(), t))
            .max_by_key(|status| status_rank(*status))
            .unwrap(),
        HealthInterpretationCategory::CalibrationIssue,
        ids_for(bundle, HealthDimension::CalibrationHealth),
    )
}

fn evaluate_dynamic(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> PhaseCHealthDimensionAssessment {
    let Some(transient) = &inputs.transient else {
        return absent(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
        );
    };
    let Some(baseline) = &inputs.baseline else {
        return absent(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
        );
    };
    if !inputs.transient_compatible {
        return incompatible_with_ids(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
            ids_for(bundle, HealthDimension::DynamicResponseHealth),
        );
    }
    let baseline_context = baseline_context(baseline);
    if matches!(
        crate::health::normalization::comparable(
            &inputs.current_context,
            &baseline_context,
            &inputs.comparability,
        )
        .0,
        crate::results::FeatureComparability::NotComparable
    ) {
        return finding(
            HealthDimension::DynamicResponseHealth,
            OverallHealthStatus::Indeterminate,
            HealthEvidenceState::InsufficientEvidence,
            HealthInterpretationCategory::ObservedBehavior,
            CausalStatus::Indeterminate,
            vec![PhaseCHealthReasonCode::BaselineIncomparable],
            Vec::new(),
        );
    }
    if transient
        .events
        .windows(2)
        .any(|pair| pair[0].event_index >= pair[1].event_index)
    {
        return dqi(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::SelectedTransientEventAmbiguous,
            Vec::new(),
        );
    }
    let matches = transient
        .events
        .iter()
        .enumerate()
        .filter(|(_, event)| {
            event.event_index == config.dynamic_response_health.selected_event_index
        })
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return finding(
            HealthDimension::DynamicResponseHealth,
            OverallHealthStatus::Indeterminate,
            HealthEvidenceState::InsufficientEvidence,
            HealthInterpretationCategory::ObservedBehavior,
            CausalStatus::Indeterminate,
            vec![PhaseCHealthReasonCode::SelectedTransientEventAbsent],
            Vec::new(),
        );
    }
    if matches.len() != 1 {
        return dqi(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::SelectedTransientEventAmbiguous,
            Vec::new(),
        );
    }
    let (serialized_index, event) = matches[0];
    let source_evidence_ids = dynamic_ids(bundle, serialized_index);
    let Some(model) = event.selected_model else {
        return dqi(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::SelectedTransientEventInvalid,
            source_evidence_ids.clone(),
        );
    };
    let matching_successful_fits = event
        .candidate_fits
        .iter()
        .filter(|fit| fit.model == model && fit.is_successful())
        .collect::<Vec<_>>();
    if matching_successful_fits.len() != 1 {
        return dqi(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::SelectedTransientEventInvalid,
            source_evidence_ids.clone(),
        );
    }
    let fit = matching_successful_fits[0];
    if event.failure.is_some() {
        return dqi(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::SelectedTransientEventInvalid,
            source_evidence_ids.clone(),
        );
    }
    let metric_inputs = [
        (
            fit.derived_features.tau_fast_s,
            &config.dynamic_response_health.baseline_tau_fast_feature,
            "s",
            &config.dynamic_response_health.maximum_tau_fast_ratio,
            false,
        ),
        (
            fit.derived_features.tau_slow_s,
            &config.dynamic_response_health.baseline_tau_slow_feature,
            "s",
            &config.dynamic_response_health.maximum_tau_slow_ratio,
            false,
        ),
        (
            fit.derived_features.time_to_90_percent_s,
            &config
                .dynamic_response_health
                .baseline_time_to_90_percent_feature,
            "s",
            &config
                .dynamic_response_health
                .maximum_time_to_90_percent_ratio,
            false,
        ),
        (
            fit.derived_features.total_response_amplitude_v,
            &config
                .dynamic_response_health
                .baseline_response_amplitude_feature,
            "V",
            &config
                .dynamic_response_health
                .maximum_response_amplitude_relative_loss,
            true,
        ),
    ];
    let mut values = Vec::with_capacity(metric_inputs.len());
    for (current, feature, required_unit, threshold, amplitude) in metric_inputs {
        let Some(distribution) = baseline
            .feature_distributions
            .iter()
            .find(|row| row.feature == *feature)
        else {
            return finding(
                HealthDimension::DynamicResponseHealth,
                OverallHealthStatus::Indeterminate,
                HealthEvidenceState::InsufficientEvidence,
                HealthInterpretationCategory::ObservedBehavior,
                CausalStatus::Indeterminate,
                vec![PhaseCHealthReasonCode::BaselineFeatureAbsent],
                source_evidence_ids.clone(),
            );
        };
        if distribution.unit != required_unit {
            return dqi(
                HealthDimension::DynamicResponseHealth,
                HealthInterpretationCategory::ObservedBehavior,
                PhaseCHealthReasonCode::UnitMismatch,
                source_evidence_ids.clone(),
            );
        }
        if distribution.sample_count == 0 {
            return dqi(
                HealthDimension::DynamicResponseHealth,
                HealthInterpretationCategory::ObservedBehavior,
                PhaseCHealthReasonCode::InvalidQuantity,
                source_evidence_ids.clone(),
            );
        }
        let Some(denominator) = distribution.mean else {
            return finding(
                HealthDimension::DynamicResponseHealth,
                OverallHealthStatus::Indeterminate,
                HealthEvidenceState::InsufficientEvidence,
                HealthInterpretationCategory::ObservedBehavior,
                CausalStatus::Indeterminate,
                vec![PhaseCHealthReasonCode::BaselineStatisticAbsent],
                source_evidence_ids.clone(),
            );
        };
        let Some(current) = current else {
            return dqi(
                HealthDimension::DynamicResponseHealth,
                HealthInterpretationCategory::ObservedBehavior,
                PhaseCHealthReasonCode::RequiredQuantityAbsent,
                source_evidence_ids.clone(),
            );
        };
        if !current.is_finite() || !denominator.is_finite() {
            return dqi(
                HealthDimension::DynamicResponseHealth,
                HealthInterpretationCategory::ObservedBehavior,
                PhaseCHealthReasonCode::InvalidQuantity,
                source_evidence_ids.clone(),
            );
        }
        values.push((current, denominator, threshold, amplitude));
    }
    let mut statuses = Vec::new();
    let mut contradictory_amplitude = false;
    for (current, denominator, threshold, amplitude) in values {
        if (!amplitude && denominator <= 0.0) || (amplitude && denominator.abs() < 1e-12) {
            return dqi(
                HealthDimension::DynamicResponseHealth,
                HealthInterpretationCategory::ObservedBehavior,
                if denominator == 0.0 {
                    PhaseCHealthReasonCode::BaselineDenominatorZero
                } else {
                    PhaseCHealthReasonCode::BaselineDenominatorNearZero
                },
                source_evidence_ids.clone(),
            );
        }
        let metric = if amplitude {
            contradictory_amplitude |= current * denominator < 0.0;
            ((denominator - current) / denominator.abs()).max(0.0)
        } else {
            current / denominator
        };
        statuses.push(severity(metric, threshold));
    }
    let Some(rmse) = fit.statistics.rmse_v else {
        return dqi(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::RequiredQuantityAbsent,
            source_evidence_ids.clone(),
        );
    };
    if !rmse.is_finite() {
        return dqi(
            HealthDimension::DynamicResponseHealth,
            HealthInterpretationCategory::ObservedBehavior,
            PhaseCHealthReasonCode::InvalidQuantity,
            source_evidence_ids.clone(),
        );
    }
    statuses.push(severity(
        rmse,
        &config.dynamic_response_health.maximum_fit_rmse_v,
    ));
    let status = statuses
        .into_iter()
        .max_by_key(|status| status_rank(*status))
        .unwrap();
    if contradictory_amplitude {
        return finding(
            HealthDimension::DynamicResponseHealth,
            if status == OverallHealthStatus::WithinBaseline {
                OverallHealthStatus::Watch
            } else {
                status
            },
            HealthEvidenceState::ContradictoryEvidence,
            HealthInterpretationCategory::ObservedBehavior,
            CausalStatus::Observed,
            vec![PhaseCHealthReasonCode::ContradictoryEvidence],
            source_evidence_ids,
        );
    }
    normal(
        HealthDimension::DynamicResponseHealth,
        status,
        HealthInterpretationCategory::ObservedBehavior,
        source_evidence_ids,
    )
}

fn evaluate_environment(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> PhaseCHealthDimensionAssessment {
    let Some(report) = &inputs.estimation else {
        return absent(
            HealthDimension::EnvironmentalRobustness,
            HealthInterpretationCategory::EnvironmentalEffect,
        );
    };
    if !inputs.estimation_compatible {
        return incompatible_with_ids(
            HealthDimension::EnvironmentalRobustness,
            HealthInterpretationCategory::EnvironmentalEffect,
            ids_for(bundle, HealthDimension::EnvironmentalRobustness),
        );
    }
    if config.environmental_robustness.covariate == EnvironmentalCovariate::Flow {
        let source_units = report
            .estimates
            .iter()
            .flat_map(|point| point.environmental_context.source_records.iter())
            .map(|record| record.source_unit.as_deref())
            .collect::<Vec<_>>();
        let units = source_units
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        if source_units
            .iter()
            .any(|unit| unit.is_none_or(str::is_empty))
            || units.len() != 1
        {
            return dqi(
                HealthDimension::EnvironmentalRobustness,
                HealthInterpretationCategory::EnvironmentalEffect,
                PhaseCHealthReasonCode::UnitMismatch,
                ids_for(bundle, HealthDimension::EnvironmentalRobustness),
            );
        }
    }
    let mut pairs = Vec::new();
    let mut previous = None;
    for point in &report.estimates {
        if !point.timestamp_s.is_finite()
            || previous.is_some_and(|value| point.timestamp_s <= value)
        {
            return dqi(
                HealthDimension::EnvironmentalRobustness,
                HealthInterpretationCategory::EnvironmentalEffect,
                PhaseCHealthReasonCode::InvalidQuantity,
                ids_for(bundle, HealthDimension::EnvironmentalRobustness),
            );
        }
        previous = Some(point.timestamp_s);
        let covariate = match config.environmental_robustness.covariate {
            EnvironmentalCovariate::TemperatureK => point.environmental_context.temperature_k,
            EnvironmentalCovariate::ConductivitySPerM => {
                point.environmental_context.conductivity_s_per_m
            }
            EnvironmentalCovariate::IonicStrengthMolL => {
                point.environmental_context.ionic_strength_mol_l
            }
            EnvironmentalCovariate::Flow => point.environmental_context.flow,
        };
        match (point.unexplained_residual_v, covariate) {
            (None, _) | (_, None) => {
                return finding(
                    HealthDimension::EnvironmentalRobustness,
                    OverallHealthStatus::Indeterminate,
                    HealthEvidenceState::InsufficientEvidence,
                    HealthInterpretationCategory::EnvironmentalEffect,
                    CausalStatus::Indeterminate,
                    vec![PhaseCHealthReasonCode::RequiredQuantityAbsent],
                    ids_for(bundle, HealthDimension::EnvironmentalRobustness),
                );
            }
            (Some(residual), Some(covariate))
                if !residual.is_finite() || !covariate.is_finite() =>
            {
                return dqi(
                    HealthDimension::EnvironmentalRobustness,
                    HealthInterpretationCategory::EnvironmentalEffect,
                    PhaseCHealthReasonCode::InvalidQuantity,
                    ids_for(bundle, HealthDimension::EnvironmentalRobustness),
                );
            }
            (Some(residual), Some(covariate)) => pairs.push((residual, covariate)),
        }
    }
    if pairs.len() < config.environmental_robustness.minimum_points {
        return finding(
            HealthDimension::EnvironmentalRobustness,
            OverallHealthStatus::Indeterminate,
            HealthEvidenceState::InsufficientEvidence,
            HealthInterpretationCategory::EnvironmentalEffect,
            CausalStatus::Indeterminate,
            vec![PhaseCHealthReasonCode::RequiredQuantityAbsent],
            ids_for(bundle, HealthDimension::EnvironmentalRobustness),
        );
    }
    let range = pairs
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max)
        - pairs.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
    if range < config.environmental_robustness.minimum_covariate_range {
        return finding(
            HealthDimension::EnvironmentalRobustness,
            OverallHealthStatus::Indeterminate,
            HealthEvidenceState::InsufficientEvidence,
            HealthInterpretationCategory::EnvironmentalEffect,
            CausalStatus::Indeterminate,
            vec![PhaseCHealthReasonCode::RequiredQuantityAbsent],
            ids_for(bundle, HealthDimension::EnvironmentalRobustness),
        );
    }
    let rms = (pairs
        .iter()
        .map(|(residual, _)| residual * residual)
        .sum::<f64>()
        / pairs.len() as f64)
        .sqrt();
    let status = if rms < config.environmental_robustness.minimum_residual_rms_v {
        OverallHealthStatus::WithinBaseline
    } else {
        severity(
            spearman(&pairs).abs(),
            &config
                .environmental_robustness
                .minimum_absolute_spearman_correlation,
        )
    };
    normal(
        HealthDimension::EnvironmentalRobustness,
        status,
        HealthInterpretationCategory::EnvironmentalEffect,
        ids_for(bundle, HealthDimension::EnvironmentalRobustness),
    )
}

fn evaluate_model_consistency(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> PhaseCHealthDimensionAssessment {
    let mut candidates = Vec::<(OverallHealthStatus, Vec<EvidenceId>)>::new();
    let mut failures = Vec::<(PhaseCHealthReasonCode, Vec<EvidenceId>)>::new();
    let mut incompatible_source = false;

    if let Some(estimation) = &inputs.estimation {
        let ids = ids_for_source(
            bundle,
            HealthDimension::ModelConsistency,
            ArtifactKind::StateEstimation,
        );
        if !inputs.estimation_compatible {
            incompatible_source = true;
        } else if estimation.estimates.is_empty() {
            failures.push((PhaseCHealthReasonCode::RequiredQuantityAbsent, ids));
        } else {
            let mut values = Vec::with_capacity(estimation.estimates.len());
            let mut failure = None;
            for point in &estimation.estimates {
                if point.update_status != crate::estimation::state::MeasurementUpdateStatus::Updated
                {
                    failure = Some(PhaseCHealthReasonCode::QualityGateFailed);
                    break;
                }
                match point.unexplained_residual_v {
                    None => {
                        failure = Some(PhaseCHealthReasonCode::RequiredQuantityAbsent);
                        break;
                    }
                    Some(value) if !value.is_finite() => {
                        failure = Some(PhaseCHealthReasonCode::InvalidQuantity);
                        break;
                    }
                    Some(value) => values.push(value),
                }
            }
            if let Some(reason) = failure {
                failures.push((reason, ids));
            } else {
                candidates.push((model_status(&values, config), ids));
            }
        }
    }

    if let Some(model) = &inputs.model {
        let ids = ids_for_source(
            bundle,
            HealthDimension::ModelConsistency,
            ArtifactKind::ModelAnalysis,
        );
        if !inputs.model_compatible {
            incompatible_source = true;
        } else if model.points.is_empty() {
            failures.push((PhaseCHealthReasonCode::RequiredQuantityAbsent, ids));
        } else {
            let mut values = Vec::with_capacity(model.points.len());
            let mut failure = None;
            for point in &model.points {
                if !point.validity.is_valid {
                    failure = Some(PhaseCHealthReasonCode::ModelOutsideDomain);
                    break;
                }
                let Some(observed) = point.observed_voltage_v else {
                    failure = Some(PhaseCHealthReasonCode::RequiredQuantityAbsent);
                    break;
                };
                if !point.time_s.is_finite()
                    || !observed.is_finite()
                    || !point.predicted_voltage_v.is_finite()
                {
                    failure = Some(PhaseCHealthReasonCode::InvalidQuantity);
                    break;
                }
                let residual = observed - point.predicted_voltage_v;
                if point
                    .unexplained_residual_v
                    .is_some_and(|stored| !stored.is_finite() || stored != residual)
                {
                    failure = Some(PhaseCHealthReasonCode::InvalidQuantity);
                    break;
                }
                values.push(residual);
            }
            if let Some(reason) = failure {
                failures.push((reason, ids));
            } else {
                candidates.push((model_status(&values, config), ids));
            }
        }
    }

    if candidates.is_empty() {
        if let Some((reason, ids)) = failures.into_iter().next() {
            return dqi(
                HealthDimension::ModelConsistency,
                HealthInterpretationCategory::ModelInconsistency,
                reason,
                ids,
            );
        }
        if incompatible_source {
            return incompatible_with_ids(
                HealthDimension::ModelConsistency,
                HealthInterpretationCategory::ModelInconsistency,
                ids_for(bundle, HealthDimension::ModelConsistency),
            );
        }
        return absent(
            HealthDimension::ModelConsistency,
            HealthInterpretationCategory::ModelInconsistency,
        );
    }

    let mut consumed_ids = candidates
        .iter()
        .flat_map(|(_, ids)| ids.clone())
        .collect::<Vec<_>>();
    consumed_ids.sort();
    consumed_ids.dedup();
    let strongest = candidates
        .iter()
        .map(|(status, _)| *status)
        .max_by_key(|status| status_rank(*status))
        .expect("non-empty candidates");
    if candidates
        .iter()
        .map(|(status, _)| *status)
        .any(|status| status != strongest)
    {
        return finding(
            HealthDimension::ModelConsistency,
            if matches!(
                strongest,
                OverallHealthStatus::Degraded | OverallHealthStatus::Critical
            ) {
                strongest
            } else {
                OverallHealthStatus::Watch
            },
            HealthEvidenceState::ContradictoryEvidence,
            HealthInterpretationCategory::ModelInconsistency,
            CausalStatus::Observed,
            vec![PhaseCHealthReasonCode::ContradictoryEvidence],
            consumed_ids,
        );
    }
    let mut result = normal(
        HealthDimension::ModelConsistency,
        strongest,
        HealthInterpretationCategory::ModelInconsistency,
        consumed_ids,
    );
    if !failures.is_empty() {
        result
            .reason_codes
            .push(PhaseCHealthReasonCode::OptionalInvalidSourceExcluded);
        result.excluded_evidence_ids = failures.into_iter().flat_map(|(_, ids)| ids).collect();
        canonicalize(&mut result);
    }
    result
}

fn evaluate_observability(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> PhaseCHealthDimensionAssessment {
    let Some(report) = &inputs.estimation else {
        return absent(
            HealthDimension::Observability,
            HealthInterpretationCategory::ModelInconsistency,
        );
    };
    if !inputs.estimation_compatible {
        return incompatible_with_ids(
            HealthDimension::Observability,
            HealthInterpretationCategory::ModelInconsistency,
            ids_for(bundle, HealthDimension::Observability),
        );
    }
    if report.observability.state_count == 0 {
        return dqi(
            HealthDimension::Observability,
            HealthInterpretationCategory::ModelInconsistency,
            PhaseCHealthReasonCode::RequiredQuantityAbsent,
            ids_for(bundle, HealthDimension::Observability),
        );
    }
    let value = report.observability.condition_number;
    let Some(condition) = value else {
        return dqi(
            HealthDimension::Observability,
            HealthInterpretationCategory::ModelInconsistency,
            PhaseCHealthReasonCode::RequiredQuantityAbsent,
            ids_for(bundle, HealthDimension::Observability),
        );
    };
    if !condition.is_finite() {
        return dqi(
            HealthDimension::Observability,
            HealthInterpretationCategory::ModelInconsistency,
            PhaseCHealthReasonCode::InvalidQuantity,
            ids_for(bundle, HealthDimension::Observability),
        );
    }
    let mut status = severity(condition, &config.observability.maximum_condition_number);
    if report.observability.numerical_rank < report.observability.state_count
        || !report.observability.unobservable_states.is_empty()
    {
        status = OverallHealthStatus::Critical;
    } else if !report.observability.weakly_observable_states.is_empty()
        && status == OverallHealthStatus::WithinBaseline
    {
        status = OverallHealthStatus::Watch;
    } else if config.observability.require_empirical_identifiability
        && !report.observability.empirical_identifiability_passed
        && status_rank(OverallHealthStatus::Degraded) > status_rank(status)
    {
        status = OverallHealthStatus::Degraded;
    }
    normal(
        HealthDimension::Observability,
        status,
        HealthInterpretationCategory::ModelInconsistency,
        ids_for(bundle, HealthDimension::Observability),
    )
}

fn evaluate_uncertainty(
    bundle: &EvidenceBundle,
    inputs: &PhaseCEligibleInputs,
    config: &PhaseCHealthEvidenceConfig,
) -> PhaseCHealthDimensionAssessment {
    let Some(model) = &inputs.model else {
        return absent(
            HealthDimension::UncertaintyHealth,
            HealthInterpretationCategory::ModelInconsistency,
        );
    };
    if !inputs.model_compatible {
        return incompatible_with_ids(
            HealthDimension::UncertaintyHealth,
            HealthInterpretationCategory::ModelInconsistency,
            ids_for(bundle, HealthDimension::UncertaintyHealth),
        );
    }
    if model.points.is_empty() {
        return absent(
            HealthDimension::UncertaintyHealth,
            HealthInterpretationCategory::ModelInconsistency,
        );
    }
    let mut partial = 0usize;
    let mut maximum_se: f64 = 0.0;
    for point in &model.points {
        match point.uncertainty.status {
            crate::model::UncertaintyStatus::NotRequested
            | crate::model::UncertaintyStatus::Unavailable => {
                return absent(
                    HealthDimension::UncertaintyHealth,
                    HealthInterpretationCategory::ModelInconsistency,
                );
            }
            crate::model::UncertaintyStatus::Partial => partial += 1,
            crate::model::UncertaintyStatus::Complete => {
                let (Some(variance), Some(se)) = (
                    point.uncertainty.total_variance_v2,
                    point.uncertainty.standard_error_v,
                ) else {
                    return dqi(
                        HealthDimension::UncertaintyHealth,
                        HealthInterpretationCategory::ModelInconsistency,
                        PhaseCHealthReasonCode::RequiredQuantityAbsent,
                        ids_for(bundle, HealthDimension::UncertaintyHealth),
                    );
                };
                if !variance.is_finite() || !se.is_finite() || variance < 0.0 || se < 0.0 {
                    return dqi(
                        HealthDimension::UncertaintyHealth,
                        HealthInterpretationCategory::ModelInconsistency,
                        PhaseCHealthReasonCode::InvalidQuantity,
                        ids_for(bundle, HealthDimension::UncertaintyHealth),
                    );
                }
                maximum_se = maximum_se.max(se);
            }
        }
    }
    let partial_status = severity(
        partial as f64 / model.points.len() as f64,
        &config
            .uncertainty_health
            .maximum_partial_uncertainty_fraction,
    );
    let standard_error_status = severity(
        maximum_se,
        &config.uncertainty_health.maximum_standard_error_v,
    );
    let status = if status_rank(partial_status) >= status_rank(standard_error_status) {
        partial_status
    } else {
        standard_error_status
    };
    normal(
        HealthDimension::UncertaintyHealth,
        status,
        HealthInterpretationCategory::ModelInconsistency,
        ids_for(bundle, HealthDimension::UncertaintyHealth),
    )
}

fn threshold_provenance_for(
    evidence_id: &str,
    config: &PhaseCHealthEvidenceConfig,
) -> Vec<ThresholdProvenance> {
    let level = match evidence_id {
        "signal.descriptive.rms" => Some((&config.signal_integrity.maximum_rms_noise_v, "V")),
        "signal.descriptive.robust_standard_deviation" => Some((
            &config
                .signal_integrity
                .maximum_robust_noise_standard_deviation_v,
            "V",
        )),
        "signal.spikes.flagged_fraction" => {
            Some((&config.signal_integrity.maximum_spike_fraction, "1"))
        }
        "signal.drift.theil_sen.slope_v_per_s" => Some((
            &config.signal_integrity.maximum_absolute_drift_v_per_s,
            "V/s",
        )),
        "calibration.selected.slope_efficiency" => Some((
            &config
                .calibration_health
                .maximum_absolute_slope_efficiency_error,
            "1",
        )),
        "calibration.selected.rmse_v" => Some((&config.calibration_health.maximum_rmse_v, "V")),
        "calibration.validation.prediction_bias_v" => Some((
            &config.calibration_health.maximum_absolute_prediction_bias_v,
            "V",
        )),
        "calibration.hysteresis.mean_hysteresis_v" => {
            Some((&config.calibration_health.maximum_hysteresis_v, "V"))
        }
        id if id.ends_with(".tau_fast_s") => {
            Some((&config.dynamic_response_health.maximum_tau_fast_ratio, "1"))
        }
        id if id.ends_with(".tau_slow_s") => {
            Some((&config.dynamic_response_health.maximum_tau_slow_ratio, "1"))
        }
        id if id.ends_with(".time_to_90_percent_s") => Some((
            &config
                .dynamic_response_health
                .maximum_time_to_90_percent_ratio,
            "1",
        )),
        id if id.ends_with(".response_amplitude_v") => Some((
            &config
                .dynamic_response_health
                .maximum_response_amplitude_relative_loss,
            "1",
        )),
        id if id.ends_with(".fit_rmse_v") => {
            Some((&config.dynamic_response_health.maximum_fit_rmse_v, "V"))
        }
        id if id.starts_with("model.point.") && id.ends_with(".unexplained_residual_v")
            || id.starts_with("estimation.point.") && id.ends_with(".unexplained_residual_v") =>
        {
            // RMS and bias are both evaluated from the same residual field.
            return level_threshold_provenance(
                evidence_id,
                [
                    (
                        "maximum_residual_bias_v",
                        &config.model_consistency.maximum_residual_bias_v,
                        "V",
                    ),
                    (
                        "maximum_residual_rms_v",
                        &config.model_consistency.maximum_residual_rms_v,
                        "V",
                    ),
                ],
                config.configuration_hash(),
            );
        }
        "estimation.observability.condition_number" => {
            Some((&config.observability.maximum_condition_number, "1"))
        }
        id if id.ends_with(".uncertainty.standard_error_v") => {
            Some((&config.uncertainty_health.maximum_standard_error_v, "V"))
        }
        _ => None,
    };
    let Some((levels, unit)) = level else {
        return Vec::new();
    };
    level_threshold_provenance(
        evidence_id,
        [("threshold", levels, unit)],
        config.configuration_hash(),
    )
}

fn level_threshold_provenance<const N: usize>(
    evidence_id: &str,
    thresholds: [(&str, &LevelThreshold, &str); N],
    configuration_hash: Option<&str>,
) -> Vec<ThresholdProvenance> {
    let mut provenance = Vec::with_capacity(N * 3);
    for (name, threshold, unit) in thresholds {
        for (level, value) in [
            ("watch", threshold.watch),
            ("degraded", threshold.degraded),
            ("critical", threshold.critical),
        ] {
            provenance.push(ThresholdProvenance {
                threshold_id: format!("{evidence_id}.{name}.{level}"),
                source: ThresholdSource::UserConfiguration,
                value,
                unit: unit.into(),
                configuration_hash: configuration_hash.map(str::to_owned),
            });
        }
    }
    provenance.sort_by(|left, right| left.threshold_id.cmp(&right.threshold_id));
    provenance
}

fn add_scalar(
    builder: &mut EvidenceBundleBuilder,
    id: impl Into<String>,
    dimension: HealthDimension,
    context: &AdapterContext,
    field_path: impl Into<String>,
    value: Option<f64>,
    unit: &str,
) {
    let Some(value) = value.filter(|value| value.is_finite()) else {
        return;
    };
    builder.add_record(EvidenceRecord {
        evidence_id: EvidenceId(id.into()),
        target: EvidenceTarget::HealthDimension(dimension),
        source: EvidenceSourceRef {
            artifact: context.source.clone(),
            field_path: field_path.into(),
        },
        experiment_scope: context.experiment_scope.clone(),
        source_class: EvidenceSourceClass::Observed,
        direction: EvidenceDirection::Neutral,
        availability: EvidenceAvailability::Available,
        strength: EvidenceStrength::NotAssessed,
        validity: EvidenceValidity::Valid,
        quantity: Some(EvidenceQuantity {
            value,
            unit: unit.into(),
            uncertainty: None,
        }),
        strength_source: StrengthSource::NotAssessed,
        strength_derivation: None,
        threshold_provenance: Vec::new(),
        lineage_artifact_ids: context.lineage_artifact_ids.clone(),
        warnings: Vec::new(),
    });
}

fn add_marker(
    builder: &mut EvidenceBundleBuilder,
    id: impl Into<String>,
    dimension: HealthDimension,
    context: &AdapterContext,
    field_path: impl Into<String>,
) {
    builder.add_record(EvidenceRecord {
        evidence_id: EvidenceId(id.into()),
        target: EvidenceTarget::HealthDimension(dimension),
        source: EvidenceSourceRef {
            artifact: context.source.clone(),
            field_path: field_path.into(),
        },
        experiment_scope: context.experiment_scope.clone(),
        source_class: EvidenceSourceClass::ProducerAssessment,
        direction: EvidenceDirection::Neutral,
        availability: EvidenceAvailability::Available,
        strength: EvidenceStrength::NotAssessed,
        validity: EvidenceValidity::Valid,
        quantity: None,
        strength_source: StrengthSource::NotAssessed,
        strength_derivation: None,
        threshold_provenance: Vec::new(),
        lineage_artifact_ids: context.lineage_artifact_ids.clone(),
        warnings: Vec::new(),
    });
}

fn scope_from_lineage(lineage: &ArtifactLineageState) -> crate::evidence::EvidenceExperimentScope {
    match lineage {
        ArtifactLineageState::Known { identity, .. } => {
            crate::evidence::EvidenceExperimentScope::from_artifact_scope(
                &identity.experiment_scope,
            )
        }
        ArtifactLineageState::LegacyUnknown { .. } => {
            crate::evidence::EvidenceExperimentScope::Unknown
        }
    }
}

fn scope_key_from_lineage(lineage: &ArtifactLineageState, sensor: bool) -> ScopeKey {
    match lineage {
        ArtifactLineageState::Known { identity, .. } if sensor => identity.sensor_scope.clone(),
        ArtifactLineageState::Known { identity, .. } => identity.channel_scope.clone(),
        ArtifactLineageState::LegacyUnknown { .. } => ScopeKey::Unspecified,
    }
}

pub(crate) fn scope_compatible(left: &ArtifactLineageState, right: &ArtifactLineageState) -> bool {
    match (left, right) {
        (
            ArtifactLineageState::Known { identity: left, .. },
            ArtifactLineageState::Known {
                identity: right, ..
            },
        ) => {
            left.experiment_scope == right.experiment_scope
                && left.sensor_scope == right.sensor_scope
                && left.channel_scope == right.channel_scope
        }
        // Unknown provenance cannot prove a mismatch and remains admissible
        // for direct evidence; it cannot support independence promotion.
        _ => true,
    }
}

fn ids_for(bundle: &EvidenceBundle, dimension: HealthDimension) -> Vec<EvidenceId> {
    let mut ids = bundle
        .records
        .iter()
        .filter_map(|record| {
            matches!(record.target, EvidenceTarget::HealthDimension(target) if target == dimension)
                .then_some(record.evidence_id.clone())
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

fn ids_for_source(
    bundle: &EvidenceBundle,
    dimension: HealthDimension,
    artifact_kind: ArtifactKind,
) -> Vec<EvidenceId> {
    let mut ids = bundle
        .records
        .iter()
        .filter_map(|record| {
            (matches!(record.target, EvidenceTarget::HealthDimension(target) if target == dimension)
                && matches!(
                    record.source.artifact,
                    crate::evidence::EvidenceArtifactSource::Known { artifact_kind: kind, .. }
                        | crate::evidence::EvidenceArtifactSource::LegacyUnknown { artifact_kind: kind, .. }
                        if kind == artifact_kind
                ))
                .then_some(record.evidence_id.clone())
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

fn dynamic_ids(bundle: &EvidenceBundle, serialized_event_index: usize) -> Vec<EvidenceId> {
    let prefix = format!("transient.event.{serialized_event_index}.");
    let mut ids = bundle
        .records
        .iter()
        .filter_map(|record| {
            record
                .evidence_id
                .0
                .starts_with(&prefix)
                .then_some(record.evidence_id.clone())
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

fn finding(
    dimension: HealthDimension,
    status: OverallHealthStatus,
    evidence_state: HealthEvidenceState,
    interpretation_category: HealthInterpretationCategory,
    causal_status: CausalStatus,
    reason_codes: Vec<PhaseCHealthReasonCode>,
    source_evidence_ids: Vec<EvidenceId>,
) -> PhaseCHealthDimensionAssessment {
    let mut result = PhaseCHealthDimensionAssessment {
        dimension,
        status,
        evidence_state,
        interpretation_category,
        causal_status,
        reason_codes,
        source_evidence_ids,
        source_artifact_ids: Vec::new(),
        excluded_evidence_ids: Vec::new(),
    };
    canonicalize(&mut result);
    result
}

fn canonicalize(result: &mut PhaseCHealthDimensionAssessment) {
    if let Some(primary) = result.reason_codes.first().copied() {
        let mut secondary = result.reason_codes[1..].to_vec();
        secondary.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        secondary.dedup();
        secondary.retain(|reason| *reason != primary);
        result.reason_codes.clear();
        result.reason_codes.push(primary);
        result.reason_codes.extend(secondary);
    }
    result.source_evidence_ids.sort();
    result.source_evidence_ids.dedup();
    result.source_artifact_ids.sort();
    result.source_artifact_ids.dedup();
    result.excluded_evidence_ids.sort();
    result.excluded_evidence_ids.dedup();
}

fn absent(
    dimension: HealthDimension,
    category: HealthInterpretationCategory,
) -> PhaseCHealthDimensionAssessment {
    finding(
        dimension,
        OverallHealthStatus::Indeterminate,
        HealthEvidenceState::NoEvidence,
        category,
        CausalStatus::Indeterminate,
        vec![PhaseCHealthReasonCode::OptionalSourceAbsent],
        Vec::new(),
    )
}

fn incompatible(
    dimension: HealthDimension,
    category: HealthInterpretationCategory,
) -> PhaseCHealthDimensionAssessment {
    finding(
        dimension,
        OverallHealthStatus::Indeterminate,
        HealthEvidenceState::InsufficientEvidence,
        category,
        CausalStatus::Indeterminate,
        vec![PhaseCHealthReasonCode::ScopeIncompatible],
        Vec::new(),
    )
}

fn incompatible_with_ids(
    dimension: HealthDimension,
    category: HealthInterpretationCategory,
    excluded_evidence_ids: Vec<EvidenceId>,
) -> PhaseCHealthDimensionAssessment {
    let mut result = incompatible(dimension, category);
    result.excluded_evidence_ids = excluded_evidence_ids;
    canonicalize(&mut result);
    result
}

fn dqi(
    dimension: HealthDimension,
    category: HealthInterpretationCategory,
    reason: PhaseCHealthReasonCode,
    ids: Vec<EvidenceId>,
) -> PhaseCHealthDimensionAssessment {
    finding(
        dimension,
        OverallHealthStatus::DataQualityInsufficient,
        HealthEvidenceState::PoorDataQuality,
        category,
        CausalStatus::Indeterminate,
        vec![reason],
        ids,
    )
}

fn normal(
    dimension: HealthDimension,
    status: OverallHealthStatus,
    category: HealthInterpretationCategory,
    ids: Vec<EvidenceId>,
) -> PhaseCHealthDimensionAssessment {
    let reason = match status {
        OverallHealthStatus::WithinBaseline => PhaseCHealthReasonCode::ThresholdWithinLimit,
        OverallHealthStatus::Watch => PhaseCHealthReasonCode::ThresholdWatch,
        OverallHealthStatus::Degraded => PhaseCHealthReasonCode::ThresholdDegraded,
        OverallHealthStatus::Critical => PhaseCHealthReasonCode::ThresholdCritical,
        _ => PhaseCHealthReasonCode::RequiredQuantityAbsent,
    };
    finding(
        dimension,
        status,
        HealthEvidenceState::AdequateEvidence,
        category,
        CausalStatus::Observed,
        vec![reason],
        ids,
    )
}

fn severity(value: f64, thresholds: &LevelThreshold) -> OverallHealthStatus {
    if value >= thresholds.critical {
        OverallHealthStatus::Critical
    } else if value >= thresholds.degraded {
        OverallHealthStatus::Degraded
    } else if value >= thresholds.watch {
        OverallHealthStatus::Watch
    } else {
        OverallHealthStatus::WithinBaseline
    }
}

fn baseline_context(baseline: &SensorHealthBaseline) -> Context {
    Context {
        sensor_id: None,
        sensor_type: baseline.sensor_type.clone(),
        sensor_design: baseline.sensor_design.clone(),
        analyte: baseline.analyte.clone(),
        sample_matrix: baseline.sample_matrix.clone(),
        temperature_k: baseline
            .temperature_domain_k
            .map(|(minimum, maximum)| (minimum + maximum) / 2.0),
        temperature_values_k: baseline
            .temperature_domain_k
            .map(|(minimum, maximum)| vec![minimum, maximum])
            .unwrap_or_default(),
        experiment_id: None,
        metadata_source: None,
    }
}

fn status_rank(status: OverallHealthStatus) -> u8 {
    match status {
        OverallHealthStatus::WithinBaseline => 0,
        OverallHealthStatus::Watch => 1,
        OverallHealthStatus::Degraded => 2,
        OverallHealthStatus::Critical => 3,
        OverallHealthStatus::DataQualityInsufficient => 4,
        OverallHealthStatus::Indeterminate => 5,
    }
}

fn model_status(values: &[f64], config: &PhaseCHealthEvidenceConfig) -> OverallHealthStatus {
    let rms = (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt();
    let bias = values.iter().sum::<f64>() / values.len() as f64;
    let one = severity(rms, &config.model_consistency.maximum_residual_rms_v);
    let two = severity(
        bias.abs(),
        &config.model_consistency.maximum_residual_bias_v,
    );
    if status_rank(one) >= status_rank(two) {
        one
    } else {
        two
    }
}

fn spearman(values: &[(f64, f64)]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let ranks = |values: Vec<f64>| {
        let mut ordered = values.iter().enumerate().collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.1.total_cmp(right.1));
        let mut ranks = vec![0.0; values.len()];
        let mut start = 0;
        while start < ordered.len() {
            let mut end = start + 1;
            while end < ordered.len() && ordered[start].1 == ordered[end].1 {
                end += 1;
            }
            let average_rank = (start + 1 + end) as f64 / 2.0;
            for (index, _) in &ordered[start..end] {
                ranks[*index] = average_rank;
            }
            start = end;
        }
        ranks
    };
    let x = ranks(values.iter().map(|(x, _)| *x).collect());
    let y = ranks(values.iter().map(|(_, y)| *y).collect());
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let numerator = x
        .iter()
        .zip(&y)
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum::<f64>();
    let denominator = (x.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>()
        * y.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>())
    .sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::read_artifact,
        results::{BaselineFeatureDistribution, HealthDomain},
    };
    use std::path::PathBuf;

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
    }

    fn config() -> PhaseCHealthEvidenceConfig {
        PhaseCHealthEvidenceConfig::load(&fixture(
            "tests/fixtures/phase_c/config/valid_phase_c.toml",
        ))
        .expect("valid Phase-C test configuration")
        .config
    }

    fn signal() -> SignalAnalysisReport {
        read_artifact(&fixture(
            "tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json",
        ))
        .expect("signal fixture")
    }

    fn eligible(
        signal: SignalAnalysisReport,
        baseline: Option<SensorHealthBaseline>,
        transient: Option<TransientAnalysisReport>,
        estimation: Option<StateEstimationReport>,
    ) -> PhaseCEligibleInputs {
        PhaseCEligibleInputs {
            signal,
            baseline,
            transient,
            calibration: None,
            estimation,
            model: None,
            mechanism: None,
            lineage_catalog: None,
            current_context: Context::default(),
            comparability: ComparabilityConfig::default(),
            transient_compatible: true,
            calibration_compatible: true,
            estimation_compatible: true,
            model_compatible: true,
            mechanism_compatible: true,
        }
    }

    #[test]
    fn phase_c_nonfinite_signal_metric_is_dqi() {
        let config = config();
        let mut source = signal();
        source.descriptive.rms = Some(f64::NAN);
        let inputs = eligible(source, None, None, None);
        let bundle = prepare_phase_c_evidence(&inputs, &config).expect("bundle");
        let result =
            evaluate_dimension(HealthDimension::SignalIntegrity, &bundle, &inputs, &config)
                .expect("evaluation");
        assert_eq!(result.status, OverallHealthStatus::DataQualityInsufficient);
        assert_eq!(
            result.reason_codes,
            vec![PhaseCHealthReasonCode::InvalidQuantity]
        );
    }

    #[test]
    fn phase_c_observability_nonfinite_condition_number_is_dqi() {
        let config = config();
        let mut estimation: StateEstimationReport = read_artifact(&fixture(
            "tests/fixtures/phase_b/e2e/state_estimation_e2e_2.json",
        ))
        .expect("estimation fixture");
        estimation.observability.state_count = 1;
        estimation.observability.numerical_rank = 1;
        estimation.observability.condition_number = Some(f64::NAN);
        let inputs = eligible(signal(), None, None, Some(estimation));
        let bundle = prepare_phase_c_evidence(&inputs, &config).expect("bundle");
        let result = evaluate_dimension(HealthDimension::Observability, &bundle, &inputs, &config)
            .expect("evaluation");
        assert_eq!(result.status, OverallHealthStatus::DataQualityInsufficient);
        assert_eq!(
            result.reason_codes,
            vec![PhaseCHealthReasonCode::InvalidQuantity]
        );
    }

    #[test]
    fn phase_c_dynamic_response_nonfinite_baseline_denominator_is_dqi() {
        let config = config();
        let mut transient: TransientAnalysisReport = read_artifact(&fixture(
            "tests/fixtures/phase_b/e2e/transient_analysis_e2e_1.json",
        ))
        .expect("transient fixture");
        let selected = transient
            .events
            .iter_mut()
            .find(|event| event.event_index == config.dynamic_response_health.selected_event_index)
            .expect("selected transient event");
        let model = selected.selected_model.expect("selected model");
        let fit = selected
            .candidate_fits
            .iter_mut()
            .find(|fit| fit.model == model && fit.is_successful())
            .expect("successful selected fit");
        fit.derived_features.tau_slow_s = Some(1.0);
        let mut baseline: SensorHealthBaseline = read_artifact(&fixture(
            "tests/fixtures/artifact_contracts/health_baseline_schema2_missing_kind.json",
        ))
        .expect("baseline fixture");
        for (feature, unit, mean) in [
            (
                config
                    .dynamic_response_health
                    .baseline_tau_fast_feature
                    .clone(),
                "s".to_string(),
                f64::NAN,
            ),
            (
                config
                    .dynamic_response_health
                    .baseline_tau_slow_feature
                    .clone(),
                "s".to_string(),
                1.0,
            ),
            (
                config
                    .dynamic_response_health
                    .baseline_time_to_90_percent_feature
                    .clone(),
                "s".to_string(),
                1.0,
            ),
            (
                config
                    .dynamic_response_health
                    .baseline_response_amplitude_feature
                    .clone(),
                "V".to_string(),
                1.0,
            ),
        ] {
            baseline
                .feature_distributions
                .push(BaselineFeatureDistribution {
                    feature,
                    unit,
                    domain: HealthDomain::DynamicResponse,
                    sample_count: 1,
                    mean: Some(mean),
                    standard_deviation: None,
                    median: None,
                    mad: None,
                    quantiles: Vec::new(),
                    minimum: None,
                    maximum: None,
                    reference_direction: None,
                    comparison_context: None,
                    empirical_values: Vec::new(),
                });
        }
        let inputs = eligible(signal(), Some(baseline), Some(transient), None);
        let bundle = prepare_phase_c_evidence(&inputs, &config).expect("bundle");
        let result = evaluate_dimension(
            HealthDimension::DynamicResponseHealth,
            &bundle,
            &inputs,
            &config,
        )
        .expect("evaluation");
        assert_eq!(result.status, OverallHealthStatus::DataQualityInsufficient);
        assert_eq!(
            result.reason_codes,
            vec![PhaseCHealthReasonCode::InvalidQuantity]
        );
    }
}
