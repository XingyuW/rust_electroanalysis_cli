//! Production A1 normalization boundary for already-read public artifacts.
//!
//! This module deliberately assembles neutral provenance/evidence only.  It
//! does not assess mechanisms or health and is therefore safe to call from
//! production preparation paths before Phase B/C exist.

use crate::{
    domain::{ArtifactLineageCatalog, ArtifactLineageNode, ArtifactLineageState, ScopeKey},
    evidence::{
        EvidenceBundle, EvidenceBundleBuilder, EvidenceBundleError, EvidenceExperimentScope,
        EvidencePairKey, EvidenceRecord, PairCovarianceDerivation, TimescaleCrossCovariance,
        TimescalePairUncertainty, TimescalePairUncertaintySource, classify_independence,
    },
    evidence_adapters::{
        AdapterContext, adapt_eis_fit, adapt_state_estimation, adapt_transient_analysis,
        try_adapt_calibration_observations,
    },
    results::{
        CalibrationObservationSet, EisFitArtifact, StateEstimationReport, TransientAnalysisReport,
    },
};

#[derive(Debug, Default, Clone)]
pub struct EvidenceBundleInputs {
    pub transient: Option<TransientAnalysisReport>,
    pub estimation: Option<StateEstimationReport>,
    pub eis_fit: Option<EisFitArtifact>,
    pub calibration_observations: Option<CalibrationObservationSet>,
}

pub fn assemble_evidence_bundle(
    inputs: EvidenceBundleInputs,
) -> Result<EvidenceBundle, EvidenceBundleError> {
    let mut catalog = ArtifactLineageCatalog::default();
    for lineage in [
        inputs.transient.as_ref().map(|v| &v.lineage),
        inputs.estimation.as_ref().map(|v| &v.lineage),
        inputs.eis_fit.as_ref().map(|v| &v.lineage),
        inputs.calibration_observations.as_ref().map(|v| &v.lineage),
    ]
    .into_iter()
    .flatten()
    {
        insert_known_lineage(&mut catalog, lineage)?;
    }

    let mut builder = EvidenceBundleBuilder::new(
        EvidenceExperimentScope::Unknown,
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        catalog,
    );
    let mut records = Vec::new();
    if let Some(artifact) = &inputs.transient {
        let context = AdapterContext::from_artifact(
            artifact,
            crate::domain::ArtifactKind::TransientAnalysis,
            &artifact.lineage,
        );
        records.extend(adapt_transient_analysis(artifact, &context));
    }
    if let Some(artifact) = &inputs.estimation {
        let context = AdapterContext::from_artifact(
            artifact,
            crate::domain::ArtifactKind::StateEstimation,
            &artifact.lineage,
        );
        records.extend(adapt_state_estimation(artifact, &context));
    }
    if let Some(artifact) = &inputs.calibration_observations {
        let context = AdapterContext::from_artifact(
            artifact,
            crate::domain::ArtifactKind::CalibrationObservations,
            &artifact.lineage,
        );
        records.extend(try_adapt_calibration_observations(artifact, &context)?);
    }
    let mut eis_records = Vec::new();
    if let Some(artifact) = &inputs.eis_fit {
        let context = AdapterContext::from_artifact(
            artifact,
            crate::domain::ArtifactKind::EisFit,
            &artifact.lineage,
        );
        eis_records = adapt_eis_fit(artifact, &context);
        records.extend(eis_records.clone());
    }
    for record in &records {
        builder.add_record(record.clone());
    }
    for (index, left) in records.iter().enumerate() {
        for right in records.iter().skip(index + 1) {
            let pair =
                EvidencePairKey::canonical(left.evidence_id.clone(), right.evidence_id.clone())?;
            let classified = classify_independence(left, right, builder_lineage_catalog(&builder));
            builder.add_independence_assessment(crate::evidence::EvidenceIndependenceAssessment {
                pair,
                classification: classified.classification,
                algorithm_id: "lineage.v1".into(),
                left_lineage_status: classified.left_lineage_status,
                right_lineage_status: classified.right_lineage_status,
                shared_ancestor_artifact_ids: classified.shared_ancestor_artifact_ids,
                shared_acquisition_families: classified.shared_acquisition_families,
                reasons: classified.reasons,
            });
        }
    }
    if let Some(artifact) = &inputs.eis_fit {
        add_eis_pair_covariances(&mut builder, artifact, &eis_records)?;
    }
    builder.build()
}

// The builder owns the catalog; this small accessor keeps assembly from
// maintaining a second, potentially divergent, lineage catalog.
fn builder_lineage_catalog(builder: &EvidenceBundleBuilder) -> &ArtifactLineageCatalog {
    builder.lineage_catalog()
}

fn insert_known_lineage(
    catalog: &mut ArtifactLineageCatalog,
    lineage: &ArtifactLineageState,
) -> Result<(), EvidenceBundleError> {
    if let ArtifactLineageState::Known {
        identity,
        direct_dependencies,
    } = lineage
    {
        catalog
            .insert(ArtifactLineageNode {
                identity: identity.clone(),
                direct_dependencies: direct_dependencies.clone(),
            })
            .map_err(|error| EvidenceBundleError::Serialization(error.to_string()))?;
    }
    Ok(())
}

fn add_eis_pair_covariances(
    builder: &mut EvidenceBundleBuilder,
    artifact: &EisFitArtifact,
    records: &[EvidenceRecord],
) -> Result<(), EvidenceBundleError> {
    let Some(matrix) = &artifact.statistics.labeled_parameter_covariance else {
        return Ok(());
    };
    let crate::domain::ArtifactLineageState::Known { identity, .. } = &artifact.lineage else {
        return Ok(());
    };
    let timescales = artifact
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| {
            (parameter.unit == "s")
                .then(|| {
                    let axis = crate::evidence::EisParameterIdentity::from_descriptor(
                        &parameter.element_id,
                        &parameter.name,
                    )
                    .ok()?
                    .axis_id();
                    Some((index, axis))
                })
                .flatten()
        })
        .collect::<Vec<_>>();
    for (left_position, (left_index, left_axis)) in timescales.iter().enumerate() {
        for (right_index, right_axis) in timescales.iter().skip(left_position + 1) {
            let covariance = matrix
                .value_exact(left_axis, right_axis)
                .ok_or(EvidenceBundleError::InvalidTimescaleCovarianceSource)?;
            let left = records
                .get(*left_index)
                .ok_or(EvidenceBundleError::InvalidTimescaleCovarianceSource)?;
            let right = records
                .get(*right_index)
                .ok_or(EvidenceBundleError::InvalidTimescaleCovarianceSource)?;
            builder.add_timescale_pair_uncertainty(TimescalePairUncertainty {
                pair: EvidencePairKey::canonical(
                    left.evidence_id.clone(),
                    right.evidence_id.clone(),
                )?,
                covariance: TimescaleCrossCovariance::TauSpace {
                    covariance_tau_s2: covariance,
                },
                source: TimescalePairUncertaintySource {
                    source_artifact: crate::evidence::EvidenceArtifactSource::Known {
                        artifact_id: identity.artifact_id.clone(),
                        artifact_kind: crate::domain::ArtifactKind::EisFit,
                    },
                    left_source_field_path: format!("$.parameters[{left_index}].value"),
                    right_source_field_path: format!("$.parameters[{right_index}].value"),
                    covariance_source_field_path: format!(
                        "$.statistics.labeled_parameter_covariance[{left_axis:?},{right_axis:?}]"
                    ),
                    derivation: PairCovarianceDerivation::ExtractedCovarianceMatrixEntry,
                },
            });
        }
    }
    Ok(())
}
