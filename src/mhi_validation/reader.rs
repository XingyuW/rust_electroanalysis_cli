//! Filesystem boundary for Phase-E inputs.

use super::{MhiValidationError, MhiValidationProtocolV1};
use crate::{
    domain::{
        ArtifactLineageState, StrictArtifactRead, StrictLineageCatalogRead,
        known_lineage_from_artifact, read_artifact_lineage_catalog_strict, read_artifact_strict,
    },
    results::{
        ArtifactSourceExpectationV1, MechanismAnalysisReport, MhiValidationDatasetV1,
        SensorHealthAssessment, ValidationRecordV1,
    },
};
use serde::Serialize;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug)]
pub struct ValidationInputs {
    pub protocol_sha256: String,
    pub dataset: StrictArtifactRead<MhiValidationDatasetV1>,
    pub dataset_directory: PathBuf,
    pub lineage_catalog: StrictLineageCatalogRead,
    pub mechanism_sources: Vec<(String, StrictArtifactRead<MechanismAnalysisReport>)>,
    pub health_sources: Vec<(String, StrictArtifactRead<SensorHealthAssessment>)>,
    pub owner_approval: Option<crate::mhi_validation::approval::OwnerApprovalEvidenceV1>,
    pub approval_trust_store_sha256: Option<String>,
}

impl ValidationInputs {
    pub fn read(
        protocol: &MhiValidationProtocolV1,
        protocol_sha256: &str,
        dataset_path: &Path,
    ) -> Result<Self, MhiValidationError> {
        let dataset = read_artifact_strict::<MhiValidationDatasetV1>(dataset_path)?;
        dataset
            .artifact
            .validate_against_protocol(protocol, protocol_sha256)?;
        let dataset_directory = canonical_regular_file(dataset_path)?
            .parent()
            .expect("canonical file has parent")
            .to_path_buf();
        let lineage_path = safe_dataset_relative_path(
            &dataset_directory,
            &dataset.artifact.lineage_catalog_source.relative_path,
        )?;
        let lineage_catalog =
            read_artifact_lineage_catalog_strict(&lineage_path).map_err(|error| {
                MhiValidationError::Dataset(format!("lineage catalog is invalid: {error}"))
            })?;
        if lineage_catalog.source_file_sha256
            != dataset.artifact.lineage_catalog_source.source_file_sha256
        {
            return Err(MhiValidationError::Dataset(
                "lineage catalog checksum does not match dataset authority".into(),
            ));
        }
        let mut mechanism_sources = Vec::new();
        let mut health_sources = Vec::new();
        for record in &dataset.artifact.records {
            if let Some(source) = &record.mechanism_source {
                if source.expected_artifact_kind != crate::domain::ArtifactKind::MechanismAnalysis
                    || source.expected_schema_version != 4
                {
                    return Err(MhiValidationError::Dataset(
                        "mechanism scientific sources must be schema-4 mechanism_analysis".into(),
                    ));
                }
                let path = safe_dataset_relative_path(&dataset_directory, &source.relative_path)?;
                let artifact = read_artifact_strict::<MechanismAnalysisReport>(&path)?;
                if artifact.source_file_sha256 != source.source_file_sha256
                    || artifact.artifact.schema_version != 4
                {
                    return Err(MhiValidationError::Dataset(
                        "mechanism source checksum or schema mismatch".into(),
                    ));
                }
                validate_source_authority(
                    record,
                    source,
                    &artifact.artifact,
                    &artifact.artifact.lineage,
                    &lineage_catalog,
                )?;
                mechanism_sources.push((record.record_id.clone(), artifact));
            }
            if let Some(source) = &record.health_source {
                if source.expected_artifact_kind != crate::domain::ArtifactKind::HealthAssessment
                    || source.expected_schema_version != 4
                {
                    return Err(MhiValidationError::Dataset(
                        "health scientific sources must be schema-4 health_assessment".into(),
                    ));
                }
                let path = safe_dataset_relative_path(&dataset_directory, &source.relative_path)?;
                let artifact = read_artifact_strict::<SensorHealthAssessment>(&path)?;
                if artifact.source_file_sha256 != source.source_file_sha256
                    || artifact.artifact.schema_version != 4
                {
                    return Err(MhiValidationError::Dataset(
                        "health source checksum or schema mismatch".into(),
                    ));
                }
                validate_source_authority(
                    record,
                    source,
                    &artifact.artifact,
                    &artifact.artifact.lineage,
                    &lineage_catalog,
                )?;
                health_sources.push((record.record_id.clone(), artifact));
            }
        }
        // The protocol is intentionally consumed to make the boundary explicit;
        // all protocol semantic checks run before this source read.
        let _ = protocol;
        Ok(Self {
            protocol_sha256: protocol_sha256.into(),
            dataset,
            dataset_directory,
            lineage_catalog,
            mechanism_sources,
            health_sources,
            owner_approval: None,
            approval_trust_store_sha256: None,
        })
    }

    pub fn attach_verified_approval(
        &mut self,
        approval: crate::mhi_validation::approval::OwnerApprovalEvidenceV1,
        trust_store_sha256: String,
    ) {
        self.owner_approval = Some(approval);
        self.approval_trust_store_sha256 = Some(trust_store_sha256);
    }
}

fn validate_source_authority<T: Serialize>(
    record: &ValidationRecordV1,
    expectation: &ArtifactSourceExpectationV1,
    artifact: &T,
    lineage: &ArtifactLineageState,
    catalog: &StrictLineageCatalogRead,
) -> Result<(), MhiValidationError> {
    let expected_matches = match (&expectation.expected_lineage, lineage) {
        (
            crate::results::ExpectedLineageV1::Known {
                artifact_id,
                semantic_sha256,
            },
            ArtifactLineageState::Known { identity, .. },
        ) => identity.artifact_id == *artifact_id && identity.semantic_sha256 == *semantic_sha256,
        (
            crate::results::ExpectedLineageV1::LegacyUnknown {
                schema_version,
                legacy_source_fingerprint,
                reason,
            },
            ArtifactLineageState::LegacyUnknown {
                source_schema_version,
                reason: actual_reason,
            },
        ) => {
            source_schema_version == &Some(*schema_version)
                && legacy_source_fingerprint == &expectation.source_file_sha256
                && legacy_reason_matches(reason.clone(), *actual_reason)
        }
        _ => false,
    };
    if !expected_matches {
        return Err(MhiValidationError::Dataset(
            "source embedded lineage does not match the dataset expectation".into(),
        ));
    }
    let ArtifactLineageState::Known {
        identity,
        direct_dependencies,
    } = lineage
    else {
        return Ok(());
    };
    let recomputed = known_lineage_from_artifact(
        identity.artifact_kind,
        identity.schema_version,
        identity.producer_version.clone(),
        identity.experiment_scope.clone(),
        identity.sensor_scope.clone(),
        identity.channel_scope.clone(),
        identity.acquisition_families.clone(),
        direct_dependencies.clone(),
        artifact,
    )
    .map_err(|error| MhiValidationError::Dataset(format!("source semantic identity: {error}")))?;
    if &recomputed != lineage {
        return Err(MhiValidationError::Dataset(
            "source semantic identity recomputation differs from embedded lineage".into(),
        ));
    }
    let Some(node) = catalog.catalog.artifacts.get(&identity.artifact_id) else {
        return Err(MhiValidationError::Dataset(
            "scoreable source root is absent from the lineage catalog".into(),
        ));
    };
    if node.identity != *identity || node.direct_dependencies != *direct_dependencies {
        return Err(MhiValidationError::Dataset(
            "scoreable source root does not match the lineage catalog".into(),
        ));
    }
    let declared_scope = crate::results::DeclaredScopeV1 {
        experiment_scope: identity.experiment_scope.clone(),
        sensor_scope: identity.sensor_scope.clone(),
        channel_scope: identity.channel_scope.clone(),
        acquisition_families: identity.acquisition_families.clone(),
    };
    if record.declared_scope != declared_scope {
        return Err(MhiValidationError::Dataset(
            "record declared scope differs from the known source identity".into(),
        ));
    }
    Ok(())
}

fn legacy_reason_matches(
    expected: crate::results::LegacyLineageReasonV1,
    actual: crate::domain::UnknownLineageReason,
) -> bool {
    matches!(
        (expected, actual),
        (
            crate::results::LegacyLineageReasonV1::FieldAbsentInLegacyArtifact,
            crate::domain::UnknownLineageReason::FieldAbsentInLegacyArtifact,
        ) | (
            crate::results::LegacyLineageReasonV1::ExternalArtifactWithoutLineage,
            crate::domain::UnknownLineageReason::ExternalArtifactWithoutLineage,
        ) | (
            crate::results::LegacyLineageReasonV1::MigrationInformationUnavailable,
            crate::domain::UnknownLineageReason::MigrationInformationUnavailable,
        )
    )
}

pub fn safe_dataset_relative_path(
    dataset_directory: &Path,
    relative: &str,
) -> Result<PathBuf, MhiValidationError> {
    if relative.is_empty() || relative.contains('\0') || relative.contains('\\') {
        return Err(MhiValidationError::UnsafePath(relative.into()));
    }
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(MhiValidationError::UnsafePath(path.into()));
    }
    let candidate = dataset_directory.join(path);
    let mut cursor = dataset_directory.to_path_buf();
    for component in path.components() {
        cursor.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&cursor).map_err(|source| MhiValidationError::Io {
            path: cursor.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(MhiValidationError::UnsafePath(cursor));
        }
    }
    let canonical = canonical_regular_file(&candidate)?;
    if !canonical.starts_with(dataset_directory) {
        return Err(MhiValidationError::UnsafePath(canonical));
    }
    Ok(canonical)
}

fn canonical_regular_file(path: &Path) -> Result<PathBuf, MhiValidationError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| MhiValidationError::Io {
        path: path.into(),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(MhiValidationError::UnsafePath(path.into()));
    }
    fs::canonicalize(path).map_err(|source| MhiValidationError::Io {
        path: path.into(),
        source,
    })
}
