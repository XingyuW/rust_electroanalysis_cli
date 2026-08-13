use crate::{
    evidence::EvidenceId,
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
            let support = event
                .segment
                .fitted_time_local
                .first()
                .zip(event.segment.fitted_time_local.last())
                .filter(|(a, b)| a.is_finite() && b.is_finite() && a < b)
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
    })
}
