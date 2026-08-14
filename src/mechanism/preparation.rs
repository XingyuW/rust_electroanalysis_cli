use crate::{
    domain::{ArtifactExperimentScope, ArtifactLineageState, ScopeKey},
    evidence::{EvidenceExperimentScope, EvidenceId},
    mechanism::temporal::*,
    runners::evidence::{EvidenceBundleInputs, assemble_evidence_bundle},
};
use std::collections::BTreeMap;
use thiserror::Error;
pub struct PhaseBEvidencePreparationInputs {
    pub evidence_inputs: EvidenceBundleInputs,
}
pub struct PhaseBEvidencePreparation {
    pub bundle: crate::evidence::EvidenceBundle,
    pub temporal_metadata: EvidenceTemporalMetadataCatalog,
    /// The Phase-B analysis scope is established by the required EIS input.
    /// Record-level eligibility compares every candidate against this scope;
    /// the A1 bundle remains a neutral aggregation boundary.
    pub analysis_scope: PhaseBAnalysisScope,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseBAnalysisScope {
    pub experiment_scope: EvidenceExperimentScope,
    pub sensor_scope: ScopeKey,
    pub channel_scope: ScopeKey,
}
#[derive(Debug, Error)]
pub enum PhaseBEvidencePreparationError {
    #[error("evidence bundle: {0}")]
    Bundle(#[from] crate::evidence::EvidenceBundleError),
    #[error("duplicate temporal metadata {0}")]
    DuplicateTemporalMetadata(String),
    #[error("invalid temporal support bounds {0}")]
    InvalidTemporalSupportBounds(String),
    #[error("invalid temporal event id {0}")]
    InvalidTemporalEventId(String),
    #[error("catalog key/value mismatch {0}")]
    TemporalCatalogKeyValueMismatch(String),
    #[error("Phase B requires an EIS analysis scope")]
    MissingAnalysisScope,
    #[error("Phase B EIS analysis scope is unresolved")]
    UnresolvedAnalysisScope,
}

fn analysis_scope(
    inputs: &EvidenceBundleInputs,
) -> Result<PhaseBAnalysisScope, PhaseBEvidencePreparationError> {
    let eis = inputs
        .eis_fit
        .as_ref()
        .ok_or(PhaseBEvidencePreparationError::MissingAnalysisScope)?;
    let ArtifactLineageState::Known { identity, .. } = &eis.lineage else {
        return Err(PhaseBEvidencePreparationError::UnresolvedAnalysisScope);
    };
    if !matches!(
        identity.experiment_scope,
        ArtifactExperimentScope::Single { .. }
    ) {
        return Err(PhaseBEvidencePreparationError::UnresolvedAnalysisScope);
    }
    Ok(PhaseBAnalysisScope {
        experiment_scope: EvidenceExperimentScope::from_artifact_scope(&identity.experiment_scope),
        sensor_scope: identity.sensor_scope.clone(),
        channel_scope: identity.channel_scope.clone(),
    })
}
fn unavailable(
    kind: crate::domain::ArtifactKind,
    adapter: &str,
    path: String,
    id: EvidenceId,
) -> EvidenceTemporalMetadata {
    EvidenceTemporalMetadata {
        evidence_id: id,
        support: EvidenceTemporalSupport::Unknown,
        clock_id: None,
        classification: TemporalClassificationMetadata {
            classified_fraction: None,
            equilibrium_fraction: None,
            steady_state_fraction: None,
            classification_source: TemporalClassificationSource::Unavailable,
        },
        provenance: TemporalSupportProvenance {
            adapter_id: adapter.into(),
            source_artifact_kind: kind,
            source_field_paths: vec![path],
        },
    }
}
pub fn prepare_phase_b_evidence(
    inputs: PhaseBEvidencePreparationInputs,
) -> Result<PhaseBEvidencePreparation, PhaseBEvidencePreparationError> {
    let refs = &inputs.evidence_inputs;
    let analysis_scope = analysis_scope(refs)?;
    let mut entries = BTreeMap::new();
    let mut add = |m: EvidenceTemporalMetadata| -> Result<(), PhaseBEvidencePreparationError> {
        if entries.insert(m.evidence_id.clone(), m).is_some() {
            Err(PhaseBEvidencePreparationError::DuplicateTemporalMetadata(
                "duplicate".into(),
            ))
        } else {
            Ok(())
        }
    };
    if let Some(eis) = &refs.eis_fit {
        for (i, _) in eis.parameters.iter().enumerate() {
            add(unavailable(
                crate::domain::ArtifactKind::EisFit,
                "adapt_eis_fit",
                format!("$.parameters[{i}].value"),
                EvidenceId(format!("eis.parameter.{i}")),
            ))?;
        }
    }
    if let Some(t) = &refs.transient {
        for (i, event) in t.events.iter().enumerate() {
            let times = &event.segment.fitted_time_local;
            let support = times
                .first()
                .zip(times.last())
                .filter(|(a, b)| {
                    a.is_finite()
                        && b.is_finite()
                        && a < b
                        && times.windows(2).all(|pair| {
                            pair[0].is_finite() && pair[1].is_finite() && pair[0] < pair[1]
                        })
                })
                .map(|(start_s, end_s)| EvidenceTemporalSupport::Window {
                    start_s: *start_s,
                    end_s: *end_s,
                })
                .unwrap_or(EvidenceTemporalSupport::Unknown);
            for name in ["tau_fast_s", "tau_slow_s"] {
                add(EvidenceTemporalMetadata {
                    evidence_id: EvidenceId(format!("transient.event.{i}.{name}")),
                    support: support.clone(),
                    clock_id: None,
                    classification: TemporalClassificationMetadata {
                        classified_fraction: None,
                        equilibrium_fraction: None,
                        steady_state_fraction: None,
                        classification_source: TemporalClassificationSource::Unavailable,
                    },
                    provenance: TemporalSupportProvenance {
                        adapter_id: "adapt_transient_analysis".into(),
                        source_artifact_kind: crate::domain::ArtifactKind::TransientAnalysis,
                        source_field_paths: vec![format!(
                            "$.events[{i}].segment.fitted_time_local"
                        )],
                    },
                })?;
            }
        }
    }
    if let Some(e) = &refs.estimation {
        for (i, p) in e.estimates.iter().enumerate() {
            for (j, _) in p.filtered_state.iter().enumerate() {
                let support = if p.timestamp_s.is_finite() {
                    EvidenceTemporalSupport::Point {
                        timestamp_s: p.timestamp_s,
                    }
                } else {
                    EvidenceTemporalSupport::Unknown
                };
                add(EvidenceTemporalMetadata {
                    evidence_id: EvidenceId(format!("estimation.point.{i}.state.{j}")),
                    support,
                    clock_id: None,
                    classification: TemporalClassificationMetadata {
                        classified_fraction: None,
                        equilibrium_fraction: None,
                        steady_state_fraction: None,
                        classification_source: TemporalClassificationSource::Unavailable,
                    },
                    provenance: TemporalSupportProvenance {
                        adapter_id: "adapt_state_estimation".into(),
                        source_artifact_kind: crate::domain::ArtifactKind::StateEstimation,
                        source_field_paths: vec![format!("$.estimates[{i}].timestamp_s")],
                    },
                })?;
            }
        }
    }
    if let Some(c) = &refs.calibration_observations {
        for (i, _) in c.observations.iter().enumerate() {
            add(unavailable(
                crate::domain::ArtifactKind::CalibrationObservations,
                "try_adapt_calibration_observations",
                format!("$.observations[{i}].potential_v"),
                EvidenceId(format!("calibration.observation.{i}")),
            ))?;
        }
    }
    let bundle = assemble_evidence_bundle(inputs.evidence_inputs)?;
    Ok(PhaseBEvidencePreparation {
        bundle,
        temporal_metadata: EvidenceTemporalMetadataCatalog { entries },
        analysis_scope,
    })
}
