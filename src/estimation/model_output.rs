//! Shared EKF/UKF decomposition of the legacy observation equation.

use super::{
    calibration_adapter::CalibrationObservationModel, environment::AlignedEnvironment,
    error::EstimationError, model::StateModel, observability::ObservabilityReport,
};
use crate::estimation_config::EquilibriumRecognitionConfig;
use crate::model::{
    AssessmentStatus, ComponentContribution, ComponentRole, EquilibriumAssessment,
    EquilibriumStatus, ObservationPrediction, UnexplainedResidual,
};
use nalgebra::{DMatrix, DVector};

#[derive(Debug, Clone)]
pub struct EquilibriumEvidence {
    pub config: EquilibriumRecognitionConfig,
    pub history_points: usize,
    pub normalized_state_rate_per_s: Option<f64>,
    pub elapsed_time_constants: Option<f64>,
    pub residual_autocorrelation: Option<f64>,
    pub environment_change_fraction: Option<f64>,
    pub maximum_state_uncertainty_fraction: Option<f64>,
    pub observable: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn equilibrium_evidence(
    config: &EquilibriumRecognitionConfig,
    state: &DVector<f64>,
    previous_state: Option<&DVector<f64>>,
    covariance: &DMatrix<f64>,
    environment: &AlignedEnvironment,
    previous_environment: Option<&AlignedEnvironment>,
    dt_s: Option<f64>,
    elapsed_since_event_s: Option<f64>,
    residual_autocorrelation: Option<f64>,
    history_points: usize,
    model: &StateModel,
    observability: &ObservabilityReport,
) -> EquilibriumEvidence {
    let normalized_state_rate_per_s = previous_state
        .zip(dt_s.filter(|value| value.is_finite() && *value > 0.0))
        .map(|(previous, dt)| {
            state
                .iter()
                .zip(previous.iter())
                .zip(&model.definitions)
                .map(|((current, previous), definition)| {
                    let scale = definition
                        .lower_bound
                        .zip(definition.upper_bound)
                        .map(|(lower, upper)| (upper - lower).abs())
                        .filter(|span| span.is_finite() && *span > 0.0)
                        .unwrap_or_else(|| current.abs().max(previous.abs()).max(1.0));
                    (current - previous).abs() / scale / dt
                })
                .fold(0.0, f64::max)
        });
    let elapsed_time_constants = if model.has_polarization() {
        elapsed_since_event_s.map(|elapsed| elapsed.max(0.0) / model.tau_p_s)
    } else {
        Some(f64::INFINITY)
    };
    let maximum_state_uncertainty_fraction = (covariance.nrows() == model.dimension()
        && covariance.ncols() == model.dimension())
    .then(|| {
        model
            .definitions
            .iter()
            .enumerate()
            .map(|(index, definition)| {
                let scale = definition
                    .lower_bound
                    .zip(definition.upper_bound)
                    .map(|(lower, upper)| (upper - lower).abs())
                    .filter(|span| span.is_finite() && *span > 0.0)
                    .unwrap_or_else(|| state[index].abs().max(1.0));
                covariance[(index, index)].max(0.0).sqrt() / scale
            })
            .fold(0.0, f64::max)
    });
    EquilibriumEvidence {
        config: config.clone(),
        history_points,
        normalized_state_rate_per_s,
        elapsed_time_constants,
        residual_autocorrelation,
        environment_change_fraction: previous_environment
            .and_then(|previous| environment_change(environment, previous)),
        maximum_state_uncertainty_fraction,
        observable: observability.numerical_rank == observability.state_count
            && observability.unobservable_states.is_empty()
            && observability.empirical_identifiability_passed,
    }
}

fn environment_change(current: &AlignedEnvironment, previous: &AlignedEnvironment) -> Option<f64> {
    let mut changes = Vec::new();
    let mut compare = |left: Option<f64>, right: Option<f64>, floor: f64| {
        if let (Some(left), Some(right)) = (left, right)
            && left.is_finite()
            && right.is_finite()
        {
            changes.push((left - right).abs() / left.abs().max(right.abs()).max(floor));
        }
    };
    compare(current.temperature_k, previous.temperature_k, 273.15);
    compare(
        current.conductivity_s_per_m,
        previous.conductivity_s_per_m,
        1.0e-12,
    );
    compare(
        current.ionic_strength_mol_l,
        previous.ionic_strength_mol_l,
        1.0e-12,
    );
    compare(current.flow, previous.flow, 1.0e-12);
    for (name, value) in &current.interferent_activities {
        compare(
            Some(*value),
            previous.interferent_activities.get(name).copied(),
            1.0e-12,
        );
    }
    changes.into_iter().reduce(f64::max)
}

pub struct EstimationModelOutput {
    pub predicted_voltage_v: f64,
    pub contributions: Vec<ComponentContribution>,
    pub equilibrium_potential_v: f64,
    pub transport_potential_v: f64,
    pub transduction_potential_v: f64,
    pub reference_potential_v: f64,
    pub external_disturbance_potential_v: f64,
    pub unexplained_residual_v: Option<f64>,
    pub equilibrium: EquilibriumAssessment,
}

pub fn decompose_legacy_observation(
    state: &DVector<f64>,
    environment: &AlignedEnvironment,
    model: &StateModel,
    calibration: &dyn CalibrationObservationModel,
    observed_voltage_v: Option<f64>,
    standardized_innovation: Option<f64>,
    evidence: Option<&EquilibriumEvidence>,
) -> Result<EstimationModelOutput, EstimationError> {
    if let Some(prediction) =
        model.compiled_observation_prediction(state, environment, observed_voltage_v)?
    {
        return assembled_output(
            prediction,
            calibration,
            model.log10_activity(state)?,
            environment,
            standardized_innovation,
            evidence,
        );
    }
    let activity = model.log10_activity(state)?;
    let equilibrium = calibration.predict_potential(activity, environment)?;
    let zero = calibration.predict_potential(0.0, environment)?;
    let transport = model
        .index("polarization")
        .map_or(0.0, |index| state[index]);
    let reference = model
        .index("baseline_offset")
        .map_or(0.0, |index| state[index]);
    let transduction = if let Some(index) = model.index("sensitivity_scale") {
        let scale = model.physical_state_value(state, index).ok_or_else(|| {
            EstimationError::Numerical("sensitivity transform returned a non-finite value".into())
        })?;
        (scale - 1.0) * (equilibrium - zero)
    } else {
        0.0
    };
    let external = 0.0;
    let values = [equilibrium, transport, transduction, reference, external];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(EstimationError::Numerical(
            "component decomposition returned a non-finite value".into(),
        ));
    }
    let mut contributions = vec![contribution(
        "legacy.equilibrium",
        "equilibrium",
        ComponentRole::Equilibrium,
        equilibrium,
    )];
    if model.has_polarization() {
        contributions.push(contribution(
            "legacy.transport.polarization",
            "transport",
            ComponentRole::Transport,
            transport,
        ));
    }
    if model.has_condition() {
        contributions.push(contribution(
            "legacy.transduction.sensitivity",
            "transduction",
            ComponentRole::Transduction,
            transduction,
        ));
    }
    if model.has_baseline() {
        contributions.push(contribution(
            "legacy.reference.baseline",
            "reference",
            ComponentRole::Reference,
            reference,
        ));
    }
    let predicted_voltage_v = contributions
        .iter()
        .filter_map(|item| item.potential_v)
        .sum::<f64>();
    let unexplained_residual_v = observed_voltage_v.map(|observed| observed - predicted_voltage_v);
    let domain = calibration.valid_domain_check(activity, environment);
    let equilibrium_assessment = assess_equilibrium(
        transport,
        transduction,
        reference,
        external,
        standardized_innovation,
        domain,
        evidence,
    );
    Ok(EstimationModelOutput {
        predicted_voltage_v,
        contributions,
        equilibrium_potential_v: equilibrium,
        transport_potential_v: transport,
        transduction_potential_v: transduction,
        reference_potential_v: reference,
        external_disturbance_potential_v: external,
        unexplained_residual_v,
        equilibrium: equilibrium_assessment,
    })
}

fn assembled_output(
    prediction: ObservationPrediction,
    calibration: &dyn CalibrationObservationModel,
    log10_activity: f64,
    environment: &AlignedEnvironment,
    standardized_innovation: Option<f64>,
    evidence: Option<&EquilibriumEvidence>,
) -> Result<EstimationModelOutput, EstimationError> {
    let role_sum = |role| {
        prediction
            .contributions
            .iter()
            .filter(|contribution| contribution.role == role)
            .filter_map(|contribution| contribution.potential_v)
            .sum::<f64>()
    };
    let equilibrium_potential = role_sum(ComponentRole::Equilibrium);
    let transport = role_sum(ComponentRole::Transport);
    let transduction = role_sum(ComponentRole::Transduction);
    let reference = role_sum(ComponentRole::Reference);
    let external = role_sum(ComponentRole::ExternalDisturbance);
    let unexplained_residual_v = match prediction.unexplained_residual {
        UnexplainedResidual::Observed(value) => Some(value),
        UnexplainedResidual::MissingObservedVoltage => None,
    };
    let equilibrium = assess_equilibrium(
        transport,
        transduction,
        reference,
        external,
        standardized_innovation,
        calibration.valid_domain_check(log10_activity, environment),
        evidence,
    );
    Ok(EstimationModelOutput {
        predicted_voltage_v: prediction.predicted_voltage_v,
        contributions: prediction.contributions,
        equilibrium_potential_v: equilibrium_potential,
        transport_potential_v: transport,
        transduction_potential_v: transduction,
        reference_potential_v: reference,
        external_disturbance_potential_v: external,
        unexplained_residual_v,
        equilibrium,
    })
}

#[allow(clippy::too_many_arguments)]
fn assess_equilibrium(
    transport_v: f64,
    transduction_v: f64,
    reference_v: f64,
    external_v: f64,
    standardized_innovation: Option<f64>,
    calibration_domain: crate::estimation::state::CalibrationDomainStatus,
    evidence: Option<&EquilibriumEvidence>,
) -> EquilibriumAssessment {
    let Some(evidence) = evidence else {
        return EquilibriumAssessment {
            status: AssessmentStatus::Indeterminate,
            classification: EquilibriumStatus::Indeterminate,
            supporting_evidence: Vec::new(),
            contradictory_evidence: Vec::new(),
            missing_evidence: vec![
                "timestamp-level equilibrium evidence context is unavailable".into(),
            ],
            validity_domain: "legacy estimator calibration domain; no physical equilibrium claim"
                .into(),
        };
    };
    if !evidence.config.enabled {
        return EquilibriumAssessment {
            status: AssessmentStatus::NotAssessed,
            classification: EquilibriumStatus::Indeterminate,
            supporting_evidence: Vec::new(),
            contradictory_evidence: Vec::new(),
            missing_evidence: vec!["equilibrium recognition is disabled by configuration".into()],
            validity_domain: "equilibrium recognition disabled".into(),
        };
    }
    let config = &evidence.config;
    let mut supporting = Vec::new();
    let mut contradictory = Vec::new();
    let mut missing = Vec::new();
    record_maximum_threshold(
        "normalized dynamic-state rate",
        evidence.normalized_state_rate_per_s,
        config.maximum_normalized_state_rate_per_s,
        "fraction/s",
        &mut supporting,
        &mut contradictory,
        &mut missing,
    );
    record_maximum_threshold(
        "dynamic potential magnitude",
        Some(transport_v.abs() + transduction_v.abs()),
        config.maximum_dynamic_potential_v,
        "V",
        &mut supporting,
        &mut contradictory,
        &mut missing,
    );
    record_maximum_threshold(
        "equilibrium gap magnitude",
        Some((transport_v + transduction_v + reference_v + external_v).abs()),
        config.maximum_equilibrium_gap_v,
        "V",
        &mut supporting,
        &mut contradictory,
        &mut missing,
    );
    match evidence
        .elapsed_time_constants
        .filter(|value| !value.is_nan() && *value >= 0.0)
    {
        Some(value) if value >= config.minimum_elapsed_time_constants => supporting.push(format!(
            "elapsed time {value:.3} time constants meets minimum {:.3}",
            config.minimum_elapsed_time_constants
        )),
        Some(value) => contradictory.push(format!(
            "elapsed time {value:.3} time constants is below minimum {:.3}",
            config.minimum_elapsed_time_constants
        )),
        None => {
            missing.push("elapsed time relative to dynamic time constants is unavailable".into())
        }
    }
    record_maximum_threshold(
        "absolute standardized innovation",
        standardized_innovation.map(f64::abs),
        config.maximum_absolute_standardized_innovation,
        "standard deviations",
        &mut supporting,
        &mut contradictory,
        &mut missing,
    );
    record_maximum_threshold(
        "absolute residual autocorrelation",
        evidence.residual_autocorrelation.map(f64::abs),
        config.maximum_absolute_residual_autocorrelation,
        "fraction",
        &mut supporting,
        &mut contradictory,
        &mut missing,
    );
    record_maximum_threshold(
        "environmental change",
        evidence.environment_change_fraction,
        config.maximum_environment_change_fraction,
        "fraction",
        &mut supporting,
        &mut contradictory,
        &mut missing,
    );
    match calibration_domain {
        crate::estimation::state::CalibrationDomainStatus::Inside => {
            supporting.push("calibration input is inside its declared domain".into())
        }
        crate::estimation::state::CalibrationDomainStatus::NearBoundary => {
            contradictory.push("calibration input is near the declared domain boundary".into())
        }
        crate::estimation::state::CalibrationDomainStatus::Outside => {
            contradictory.push("calibration input is outside the declared domain".into())
        }
        crate::estimation::state::CalibrationDomainStatus::Missing => {
            missing.push("calibration-domain status is unavailable".into())
        }
    }
    record_maximum_threshold(
        "maximum normalized state uncertainty",
        evidence.maximum_state_uncertainty_fraction,
        config.maximum_state_uncertainty_fraction,
        "fraction",
        &mut supporting,
        &mut contradictory,
        &mut missing,
    );
    if config.require_observable {
        if evidence.observable {
            supporting
                .push("local observability and empirical identifiability checks passed".into());
        } else {
            contradictory.push(
                "local observability or empirical identifiability requirement did not pass".into(),
            );
        }
    } else if evidence.observable {
        supporting.push("observability evidence is available".into());
    } else {
        missing.push("observability was not required and did not pass".into());
    }
    if evidence.history_points < config.minimum_history_points {
        missing.push(format!(
            "history has {} points; at least {} are required",
            evidence.history_points, config.minimum_history_points
        ));
    } else {
        supporting.push(format!(
            "history length {} meets minimum {}",
            evidence.history_points, config.minimum_history_points
        ));
    }
    let status = if !contradictory.is_empty() {
        AssessmentStatus::Contradicted
    } else if !missing.is_empty() {
        AssessmentStatus::Indeterminate
    } else {
        AssessmentStatus::Supported
    };
    EquilibriumAssessment {
        status,
        classification: match status {
            AssessmentStatus::Supported => EquilibriumStatus::Equilibrium,
            AssessmentStatus::Contradicted => EquilibriumStatus::Disturbed,
            AssessmentStatus::NotAssessed | AssessmentStatus::Indeterminate => {
                EquilibriumStatus::Indeterminate
            }
        },
        supporting_evidence: supporting,
        contradictory_evidence: contradictory,
        missing_evidence: missing,
        validity_domain: "configured thresholds within stored calibration and estimator domains; this is an operational equilibrium classification, not physical mechanism validation".into(),
    }
}

fn record_maximum_threshold(
    name: &str,
    value: Option<f64>,
    maximum: f64,
    unit: &str,
    supporting: &mut Vec<String>,
    contradictory: &mut Vec<String>,
    missing: &mut Vec<String>,
) {
    match value.filter(|value| value.is_finite()) {
        Some(value) if value <= maximum => supporting.push(format!(
            "{name} {value:.6e} {unit} is within threshold {maximum:.6e} {unit}"
        )),
        Some(value) => contradictory.push(format!(
            "{name} {value:.6e} {unit} exceeds threshold {maximum:.6e} {unit}"
        )),
        None => missing.push(format!("{name} evidence is unavailable")),
    }
}

pub struct WeightedObservation<'a> {
    pub states: &'a [DVector<f64>],
    pub weights: &'a [f64],
    pub reference_state: &'a DVector<f64>,
    pub environment: &'a AlignedEnvironment,
    pub model: &'a StateModel,
    pub calibration: &'a dyn CalibrationObservationModel,
    pub observed_voltage_v: Option<f64>,
    pub standardized_innovation: Option<f64>,
    pub equilibrium_evidence: Option<&'a EquilibriumEvidence>,
}

pub fn decompose_weighted_observation(
    observation: WeightedObservation<'_>,
) -> Result<EstimationModelOutput, EstimationError> {
    let WeightedObservation {
        states,
        weights,
        reference_state,
        environment,
        model,
        calibration,
        observed_voltage_v,
        standardized_innovation,
        equilibrium_evidence,
    } = observation;
    if states.len() != weights.len() || states.is_empty() {
        return Err(EstimationError::Numerical(
            "weighted component decomposition has invalid dimensions".into(),
        ));
    }
    let mut totals = [0.0; 5];
    for (state, weight) in states.iter().zip(weights) {
        let output = decompose_legacy_observation(
            state,
            environment,
            model,
            calibration,
            None,
            standardized_innovation,
            None,
        )?;
        for (total, value) in totals.iter_mut().zip([
            output.equilibrium_potential_v,
            output.transport_potential_v,
            output.transduction_potential_v,
            output.reference_potential_v,
            output.external_disturbance_potential_v,
        ]) {
            *total += weight * value;
        }
    }
    let mut result = decompose_legacy_observation(
        reference_state,
        environment,
        model,
        calibration,
        observed_voltage_v,
        standardized_innovation,
        equilibrium_evidence,
    )?;
    result.equilibrium_potential_v = totals[0];
    result.transport_potential_v = totals[1];
    result.transduction_potential_v = totals[2];
    result.reference_potential_v = totals[3];
    result.external_disturbance_potential_v = totals[4];
    let mut index = 0;
    result.contributions[index].potential_v = Some(totals[0]);
    index += 1;
    if model.has_polarization() {
        result.contributions[index].potential_v = Some(totals[1]);
        index += 1;
    }
    if model.has_condition() {
        result.contributions[index].potential_v = Some(totals[2]);
        index += 1;
    }
    if model.has_baseline() {
        result.contributions[index].potential_v = Some(totals[3]);
    }
    result.predicted_voltage_v = result
        .contributions
        .iter()
        .filter_map(|item| item.potential_v)
        .sum();
    result.unexplained_residual_v =
        observed_voltage_v.map(|value| value - result.predicted_voltage_v);
    result.equilibrium = assess_equilibrium(
        totals[1],
        totals[2],
        totals[3],
        totals[4],
        standardized_innovation,
        calibration.valid_domain_check(model.log10_activity(reference_state)?, environment),
        equilibrium_evidence,
    );
    Ok(result)
}

fn contribution(
    component_id: &str,
    owner: &str,
    role: ComponentRole,
    voltage_v: f64,
) -> ComponentContribution {
    ComponentContribution {
        component_id: component_id.into(),
        owner: Some(owner.into()),
        role,
        semantics: crate::model::ContributionSemantics::AdditivePotential,
        potential_v: Some(voltage_v),
        variance_v2: None,
        source: "legacy estimation compatibility adapter".into(),
        validity_domain: "stored calibration and configured state-model domain".into(),
        interpretation_status: crate::model::InterpretationStatus::Phenomenological,
        equation_version: 1,
        validity_status: crate::model::ValidityStatus::Valid,
        warnings: Vec::new(),
        uncertainty_status: crate::model::UncertaintyStatus::NotRequested,
        state_output_ids: Vec::new(),
        auxiliary_outputs: std::collections::BTreeMap::new(),
    }
}

#[cfg(test)]
mod equilibrium_tests {
    use super::{EquilibriumEvidence, assess_equilibrium};
    use crate::{
        estimation::state::CalibrationDomainStatus,
        estimation_config::EquilibriumRecognitionConfig, model::AssessmentStatus,
    };

    fn complete_evidence() -> EquilibriumEvidence {
        EquilibriumEvidence {
            config: EquilibriumRecognitionConfig::default(),
            history_points: 10,
            normalized_state_rate_per_s: Some(0.0),
            elapsed_time_constants: Some(10.0),
            residual_autocorrelation: Some(0.0),
            environment_change_fraction: Some(0.0),
            maximum_state_uncertainty_fraction: Some(0.001),
            observable: true,
        }
    }

    #[test]
    fn complete_stable_evidence_supports_operational_equilibrium() {
        let evidence = complete_evidence();
        let assessment = assess_equilibrium(
            0.0,
            0.0,
            0.0,
            0.0,
            Some(0.1),
            CalibrationDomainStatus::Inside,
            Some(&evidence),
        );
        assert_eq!(assessment.status, AssessmentStatus::Supported);
        assert!(assessment.missing_evidence.is_empty());
    }

    #[test]
    fn slow_reference_drift_is_not_classified_as_equilibrium() {
        let mut evidence = complete_evidence();
        evidence.normalized_state_rate_per_s = Some(1.0e-2);
        let assessment = assess_equilibrium(
            0.0,
            0.0,
            0.01,
            0.0,
            Some(0.1),
            CalibrationDomainStatus::Inside,
            Some(&evidence),
        );
        assert_eq!(assessment.status, AssessmentStatus::Contradicted);
        assert!(assessment.contradictory_evidence.len() >= 2);
    }

    #[test]
    fn incomplete_history_remains_indeterminate() {
        let mut evidence = complete_evidence();
        evidence.history_points = 1;
        evidence.residual_autocorrelation = None;
        let assessment = assess_equilibrium(
            0.0,
            0.0,
            0.0,
            0.0,
            Some(0.1),
            CalibrationDomainStatus::Inside,
            Some(&evidence),
        );
        assert_eq!(assessment.status, AssessmentStatus::Indeterminate);
        assert!(!assessment.missing_evidence.is_empty());
    }
}
