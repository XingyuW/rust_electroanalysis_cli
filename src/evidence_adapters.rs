//! A1 adapters from public producer result contracts into neutral evidence.
//!
//! Adapters preserve producer fields and provenance.  They do not score
//! mechanisms, diagnose health causes, infer acquisition families, or assign
//! evidence strength from raw values.

use crate::{
    domain::{ArtifactKind, ArtifactLineageState},
    evidence::{
        ComponentId, EvidenceArtifactSource, EvidenceAvailability, EvidenceDirection,
        EvidenceExperimentScope, EvidenceId, EvidenceQuantity, EvidenceRecord, EvidenceSourceClass,
        EvidenceSourceRef, EvidenceStrength, EvidenceTarget, EvidenceValidity, HealthFindingId,
        StrengthSource,
    },
    results::{
        CalibrationObservationSet, EisFitArtifact, ModelAnalysisReport, SignalAnalysisReport,
        StateEstimationReport, TransientAnalysisReport,
    },
};

#[derive(Debug, Clone)]
pub struct AdapterContext {
    pub source: EvidenceArtifactSource,
    pub experiment_scope: EvidenceExperimentScope,
    pub lineage_artifact_ids: Vec<crate::domain::ArtifactId>,
}

impl AdapterContext {
    pub fn new(source: EvidenceArtifactSource, experiment_scope: EvidenceExperimentScope) -> Self {
        Self {
            source,
            experiment_scope,
            lineage_artifact_ids: Vec::new(),
        }
    }

    /// Constructs the source reference from the serialized A1 lineage state.
    /// A legacy state remains a legacy source fingerprint; it is never
    /// upgraded to a Known artifact ID by an adapter.
    pub fn from_artifact<T: serde::Serialize>(
        artifact: &T,
        artifact_kind: ArtifactKind,
        lineage: &ArtifactLineageState,
    ) -> Self {
        match lineage {
            ArtifactLineageState::Known {
                identity,
                direct_dependencies,
            } => Self {
                source: EvidenceArtifactSource::Known {
                    artifact_id: identity.artifact_id.clone(),
                    artifact_kind,
                },
                experiment_scope: EvidenceExperimentScope::from_artifact_scope(
                    &identity.experiment_scope,
                ),
                lineage_artifact_ids: direct_dependencies
                    .iter()
                    .map(|dependency| dependency.artifact_id.clone())
                    .collect(),
            },
            ArtifactLineageState::LegacyUnknown { .. } => {
                let serialized = serde_json::to_vec(artifact).unwrap_or_default();
                legacy_context(artifact_kind, &serialized, EvidenceExperimentScope::Unknown)
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn record(
    id: impl Into<String>,
    target: EvidenceTarget,
    context: &AdapterContext,
    field_path: impl Into<String>,
    source_class: EvidenceSourceClass,
    availability: EvidenceAvailability,
    quantity: Option<EvidenceQuantity>,
    validity: EvidenceValidity,
) -> EvidenceRecord {
    EvidenceRecord {
        evidence_id: EvidenceId(id.into()),
        target,
        source: EvidenceSourceRef {
            artifact: context.source.clone(),
            field_path: field_path.into(),
        },
        experiment_scope: context.experiment_scope.clone(),
        source_class,
        direction: match availability {
            EvidenceAvailability::Available | EvidenceAvailability::Missing => {
                EvidenceDirection::Neutral
            }
            EvidenceAvailability::NotApplicable => EvidenceDirection::NotApplicable,
        },
        availability,
        strength: EvidenceStrength::NotAssessed,
        validity,
        quantity,
        strength_source: StrengthSource::NotAssessed,
        strength_derivation: None,
        threshold_provenance: Vec::new(),
        lineage_artifact_ids: context.lineage_artifact_ids.clone(),
        warnings: Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn scalar_record(
    id: impl Into<String>,
    target: EvidenceTarget,
    context: &AdapterContext,
    field_path: String,
    source_class: EvidenceSourceClass,
    value: Option<f64>,
    unit: String,
    validity: EvidenceValidity,
) -> EvidenceRecord {
    match value.filter(|value| value.is_finite()) {
        Some(value) => record(
            id,
            target,
            context,
            field_path,
            source_class,
            EvidenceAvailability::Available,
            Some(EvidenceQuantity {
                value,
                unit,
                uncertainty: None,
            }),
            validity,
        ),
        None => record(
            id,
            target,
            context,
            field_path,
            source_class,
            EvidenceAvailability::Missing,
            None,
            EvidenceValidity::NotAssessed,
        ),
    }
}

/// Adapts fitted EIS parameters without interpreting them as mechanisms or
/// converting arbitrary frequencies into timescales.
pub fn adapt_eis_fit(artifact: &EisFitArtifact, context: &AdapterContext) -> Vec<EvidenceRecord> {
    artifact
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            scalar_record(
                format!("eis.parameter.{index}"),
                EvidenceTarget::ModelComponent(ComponentId(parameter.element_id.clone())),
                context,
                format!("$.parameters[{index}].value"),
                EvidenceSourceClass::ModelDerived,
                Some(parameter.value),
                if parameter.unit.is_empty() {
                    "1".into()
                } else {
                    parameter.unit.clone()
                },
                if parameter.value.is_finite() {
                    EvidenceValidity::Valid
                } else {
                    EvidenceValidity::NotAssessed
                },
            )
        })
        .collect()
}

/// Adapts transient fit parameters and derived tau features as neutral model
/// evidence, retaining the selected fit's public field paths.
pub fn adapt_transient_analysis(
    artifact: &TransientAnalysisReport,
    context: &AdapterContext,
) -> Vec<EvidenceRecord> {
    let mut records = Vec::new();
    for (event_index, event) in artifact.events.iter().enumerate() {
        if let Some(fit) = event
            .candidate_fits
            .iter()
            .find(|fit| Some(fit.model) == event.selected_model)
        {
            for (parameter_index, parameter) in fit.parameters.iter().enumerate() {
                records.push(scalar_record(format!("transient.event.{event_index}.parameter.{parameter_index}"), EvidenceTarget::ModelComponent(ComponentId(parameter.name.clone())), context, format!("$.events[{event_index}].candidate_fits[].parameters[{parameter_index}].value"), EvidenceSourceClass::ModelDerived, Some(parameter.value), parameter.unit.clone(), if fit.is_successful() { EvidenceValidity::Valid } else { EvidenceValidity::NotAssessed }));
            }
            for (name, value) in [
                ("tau_fast_s", fit.derived_features.tau_fast_s),
                ("tau_slow_s", fit.derived_features.tau_slow_s),
            ] {
                if value.is_some() {
                    records.push(scalar_record(
                        format!("transient.event.{event_index}.{name}"),
                        EvidenceTarget::ModelComponent(ComponentId(name.into())),
                        context,
                        format!("$.events[{event_index}].candidate_fits[].derived_features.{name}"),
                        EvidenceSourceClass::ModelDerived,
                        value,
                        "s".into(),
                        if fit.is_successful() {
                            EvidenceValidity::Valid
                        } else {
                            EvidenceValidity::NotAssessed
                        },
                    ));
                }
            }
        }
    }
    records
}

/// Adapts retained calibration observations.  Source observation IDs remain
/// in the field path; aggregate scope is never silently narrowed.
pub fn adapt_calibration_observations(
    artifact: &CalibrationObservationSet,
    context: &AdapterContext,
) -> Vec<EvidenceRecord> {
    try_adapt_calibration_observations(artifact, context).unwrap_or_default()
}

/// Checked calibration adapter used by production assembly. Aggregate scopes
/// may only narrow when the producer-owned observation record proves the
/// member experiment; mismatches are returned as typed contract errors.
pub fn try_adapt_calibration_observations(
    artifact: &CalibrationObservationSet,
    context: &AdapterContext,
) -> Result<Vec<EvidenceRecord>, crate::evidence::EvidenceBundleError> {
    artifact
        .observations
        .iter()
        .enumerate()
        .map(
            |(index, observation)| -> Result<EvidenceRecord, crate::evidence::EvidenceBundleError> {
                let mut record = scalar_record(
                    format!("calibration.observation.{index}"),
                    EvidenceTarget::ModelComponent(ComponentId(observation.analyte.clone())),
                    context,
                    format!("$.observations[{index}].potential_v"),
                    EvidenceSourceClass::Observed,
                    Some(observation.potential_v),
                    "V".into(),
                    EvidenceValidity::Valid,
                );
                if context.experiment_scope.is_aggregate() {
                    // A `Single` scope is permitted only after this adapter has
                    // read the authoritative observation.experiment_id itself.
                    let selected =
                        crate::evidence::SelectedExperimentRecord::calibration_observation(
                            observation,
                            index,
                        )?;
                    record.experiment_scope =
                        context.experiment_scope.narrow_selected_record(selected)?;
                }
                Ok(record)
            },
        )
        .collect()
}

/// Adapts model component outputs while retaining interpretation status in the
/// source artifact.  No component is promoted by this adapter.
pub fn adapt_model_analysis(
    artifact: &ModelAnalysisReport,
    context: &AdapterContext,
) -> Vec<EvidenceRecord> {
    artifact.points.iter().enumerate().flat_map(|(point_index, point)| {
        point.contributions.iter().enumerate().filter_map(move |(component_index, contribution)| {
            let value = contribution.potential_v;
            value.map(|value| scalar_record(format!("model.point.{point_index}.component.{component_index}"), EvidenceTarget::ModelComponent(ComponentId(contribution.component_id.clone())), context, format!("$.points[{point_index}].contributions[{component_index}].potential_v"), EvidenceSourceClass::ModelDerived, Some(value), "V".into(), EvidenceValidity::Valid))
        })
    }).collect()
}

/// Adapts state estimates using stable serialized state names as field
/// provenance.  Positional covariance remains unavailable to A1 consumers.
pub fn adapt_state_estimation(
    artifact: &StateEstimationReport,
    context: &AdapterContext,
) -> Vec<EvidenceRecord> {
    artifact
        .estimates
        .iter()
        .enumerate()
        .flat_map(|(point_index, point)| {
            point
                .filtered_state
                .iter()
                .enumerate()
                .map(move |(state_index, state)| {
                    scalar_record(
                        format!("estimation.point.{point_index}.state.{state_index}"),
                        EvidenceTarget::ModelComponent(ComponentId(state.name.clone())),
                        context,
                        format!("$.estimates[{point_index}].filtered_state[{state_index}].value"),
                        EvidenceSourceClass::ModelDerived,
                        state.value,
                        state.unit.clone(),
                        EvidenceValidity::NotAssessed,
                    )
                })
        })
        .collect()
}

/// Signal analysis is adapted only when a scalar feature is explicitly
/// selected by the caller.  The generic result contains heterogeneous feature
/// payloads, so no field-name guessing is performed here.
pub fn adapt_signal_scalar(
    _artifact: &SignalAnalysisReport,
    context: &AdapterContext,
    feature_name: &str,
    value: Option<f64>,
    unit: &str,
) -> Result<EvidenceRecord, crate::evidence::EvidenceBundleError> {
    if feature_name.is_empty() || unit.is_empty() {
        return Err(crate::evidence::EvidenceBundleError::EmptyIdentifier);
    }
    Ok(scalar_record(
        format!("signal.feature.{feature_name}"),
        EvidenceTarget::HealthFinding(HealthFindingId(feature_name.into())),
        context,
        format!("$.features[{feature_name}]"),
        EvidenceSourceClass::Observed,
        value,
        unit.into(),
        EvidenceValidity::NotAssessed,
    ))
}

/// Utility for adapters that intentionally cannot establish source lineage.
pub fn legacy_context(
    artifact_kind: crate::domain::ArtifactKind,
    serialized_bytes: &[u8],
    scope: EvidenceExperimentScope,
) -> AdapterContext {
    AdapterContext::new(
        EvidenceArtifactSource::LegacyUnknown {
            artifact_kind,
            source_fingerprint: crate::evidence::LegacySourceFingerprint::from_bytes(
                serialized_bytes,
            ),
        },
        scope,
    )
}
