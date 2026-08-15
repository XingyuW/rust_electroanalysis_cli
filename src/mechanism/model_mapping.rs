//! Explicit adapters from legacy analysis artifacts to ISM component priors.
//!
//! These adapters intentionally require a caller-provided stable component ID
//! and stable source path.  No target component is selected from parameter
//! ordering, labels, or a fitted time constant.

use crate::results::{
    CalibrationAnalysisReport, ComponentHypothesisAssessment, ComponentHypothesisDefinition,
    ComponentParameterPrior, ComponentPriorMapping, EisFitArtifact, EvidenceLevel,
    SignalAnalysisReport, TransientAnalysisReport,
};

pub fn eis_prior(
    artifact: &EisFitArtifact,
    mapping: &ComponentPriorMapping,
) -> Option<ComponentParameterPrior> {
    let parameter = artifact
        .parameters
        .iter()
        .find(|parameter| parameter.element_id == mapping.source_path)?;
    finite_prior(
        mapping,
        parameter.value,
        parameter.unit.clone(),
        parameter.standard_error,
        format!("eis:{}", artifact.fit_id),
    )
}

pub fn transient_prior(
    artifact: &TransientAnalysisReport,
    mapping: &ComponentPriorMapping,
) -> Option<ComponentParameterPrior> {
    let parameter = artifact.events.iter().find_map(|event| {
        event.selected_model.and_then(|selected| {
            event
                .candidate_fits
                .iter()
                .find(|fit| fit.model == selected && fit.is_successful())
                .and_then(|fit| {
                    fit.parameters
                        .iter()
                        .find(|p| p.name == mapping.source_path)
                })
        })
    })?;
    finite_prior(
        mapping,
        parameter.value,
        parameter.unit.clone(),
        None,
        format!("transient:{}", artifact.experiment_id),
    )
}

pub fn calibration_prior(
    artifact: &CalibrationAnalysisReport,
    mapping: &ComponentPriorMapping,
) -> Option<ComponentParameterPrior> {
    let parameter = artifact.selected_model.and_then(|selected| {
        artifact
            .candidate_models
            .iter()
            .find(|model| model.model_kind == selected)
            .and_then(|model| {
                model
                    .parameters
                    .iter()
                    .find(|p| p.name == mapping.source_path)
            })
    })?;
    finite_prior(
        mapping,
        parameter.value,
        parameter.unit.clone(),
        parameter.standard_error,
        format!("calibration:{}", artifact.calibration_id),
    )
}

pub fn signal_prior(
    artifact: &SignalAnalysisReport,
    mapping: &ComponentPriorMapping,
) -> Option<ComponentParameterPrior> {
    let (value, unit) = match mapping.source_path.as_str() {
        "theil_sen_drift_rate_v_per_s" => (
            artifact
                .drift
                .iter()
                .find(|drift| matches!(drift.model, crate::results::DriftModelKind::TheilSen))
                .and_then(|drift| drift.slope_v_per_s),
            "V/s".to_string(),
        ),
        "rms_noise" => (artifact.descriptive.rms, artifact.unit.clone()),
        _ => return None,
    };
    finite_prior(
        mapping,
        value?,
        unit,
        None,
        format!("signal:{}", artifact.analysis_id),
    )
}

fn finite_prior(
    mapping: &ComponentPriorMapping,
    value: f64,
    unit: String,
    standard_error: Option<f64>,
    source_artifact: String,
) -> Option<ComponentParameterPrior> {
    value.is_finite().then(|| ComponentParameterPrior {
        component_id: mapping.component_id.clone(),
        parameter_id: mapping.component_parameter_id.clone(),
        value,
        unit,
        standard_error: standard_error.filter(|value| value.is_finite() && *value >= 0.0),
        source_artifact,
        source_path: mapping.source_path.clone(),
    })
}

/// Assess a component hypothesis without converting numerical similarity into
/// a physical mechanism label. A timescale match with insufficient independent
/// replicates is deliberately capped at weak evidence.
pub fn assess_component_hypothesis(
    definition: &ComponentHypothesisDefinition,
    supporting: Vec<String>,
    contradictory: Vec<String>,
    missing: Vec<String>,
    independent_replicates: usize,
) -> ComponentHypothesisAssessment {
    let evidence_level = if !contradictory.is_empty() {
        EvidenceLevel::Contradictory
    } else if !missing.is_empty()
        || independent_replicates < definition.minimum_independent_replicates
    {
        EvidenceLevel::Weak
    } else if supporting.is_empty() {
        EvidenceLevel::Insufficient
    } else {
        // Strong identification requires a dedicated future validation study.
        EvidenceLevel::Moderate
    };
    let mut missing_evidence = missing;
    if independent_replicates < definition.minimum_independent_replicates {
        missing_evidence.push(format!(
            "{} independent replicates required; only {} available",
            definition.minimum_independent_replicates, independent_replicates
        ));
    }
    ComponentHypothesisAssessment {
        hypothesis_id: definition.hypothesis_id.clone(),
        component_ids: definition.component_ids.clone(),
        evidence_level,
        supporting_evidence: supporting,
        contradictory_evidence: contradictory,
        missing_evidence,
        alternative_explanations: vec![
            "similar fitted timescales can arise from distinct processes".to_string(),
            "model misspecification or unmeasured disturbances can explain the observation"
                .to_string(),
        ],
        applicability_domain: definition.applicability_domain.clone(),
    }
}
