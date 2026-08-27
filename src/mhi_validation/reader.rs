//! Filesystem boundary for Phase-E inputs.

use super::{MhiValidationError, MhiValidationProtocolV1};
use crate::{
    domain::{
        ArtifactLineageState, StrictArtifactRead, StrictLineageCatalogRead,
        known_lineage_from_artifact, open_strict_directory,
        read_artifact_lineage_catalog_strict_at, read_artifact_strict_at,
    },
    results::{
        ArtifactSourceExpectationV1, MechanismAnalysisReport, MhiValidationDatasetV1,
        SensorHealthAssessment, ValidationRecordV1,
    },
};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

/// Reader-owned validation state.  Approval authority is private and can only
/// be attached by the crate after strict embedded-authority verification.
///
/// ```compile_fail
/// use rust_electroanalysis_cli::mhi_validation::ValidationInputs;
///
/// let _forged = ValidationInputs {
///     protocol_sha256: String::new(),
///     ..todo!()
/// };
/// ```
///
/// ```compile_fail
/// use rust_electroanalysis_cli::mhi_validation::ValidationInputs;
///
/// let _attach = ValidationInputs::attach_verified_approval;
/// ```
#[derive(Debug, Clone)]
pub struct ValidationInputs {
    pub protocol_sha256: String,
    pub dataset: StrictArtifactRead<MhiValidationDatasetV1>,
    pub dataset_directory: PathBuf,
    pub(crate) dataset_directory_authority: Arc<fs::File>,
    pub lineage_catalog: StrictLineageCatalogRead,
    pub mechanism_sources: Vec<(String, StrictArtifactRead<MechanismAnalysisReport>)>,
    pub health_sources: Vec<(String, StrictArtifactRead<SensorHealthAssessment>)>,
    pub(crate) owner_approval: Option<crate::mhi_validation::approval::VerifiedOwnerApproval>,
}

impl ValidationInputs {
    pub fn read(
        protocol: &MhiValidationProtocolV1,
        protocol_sha256: &str,
        dataset_path: &Path,
    ) -> Result<Self, MhiValidationError> {
        let dataset_directory = dataset_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let dataset_directory_authority =
            Arc::new(open_strict_directory(&dataset_directory).map_err(map_reader_artifact_error)?);
        let dataset_name = dataset_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| MhiValidationError::UnsafePath(dataset_path.into()))?;
        let dataset: StrictArtifactRead<MhiValidationDatasetV1> =
            read_artifact_strict_at(&dataset_directory_authority, dataset_name, dataset_path)
                .map_err(map_reader_artifact_error)?;
        dataset
            .artifact
            .validate_against_protocol(protocol, protocol_sha256)?;
        let lineage_path = safe_dataset_relative_path(
            &dataset_directory,
            &dataset.artifact.lineage_catalog_source.relative_path,
        )?;
        let lineage_catalog = read_artifact_lineage_catalog_strict_at(
            &dataset_directory_authority,
            &dataset.artifact.lineage_catalog_source.relative_path,
            &lineage_path,
        )
        .map_err(|error| {
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
                let artifact = read_phase_e_mechanism_source(
                    &dataset_directory_authority,
                    &source.relative_path,
                    &path,
                )?;
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
                let artifact = read_phase_e_health_source(
                    &dataset_directory_authority,
                    &source.relative_path,
                    &path,
                )?;
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
            dataset_directory_authority,
            lineage_catalog,
            mechanism_sources,
            health_sources,
            owner_approval: None,
        })
    }

    pub(crate) fn attach_verified_approval(
        &mut self,
        approval: crate::mhi_validation::approval::VerifiedOwnerApproval,
    ) {
        self.owner_approval = Some(approval);
    }
}

fn read_phase_e_mechanism_source(
    directory: &fs::File,
    relative: &str,
    path: &Path,
) -> Result<StrictArtifactRead<MechanismAnalysisReport>, MhiValidationError> {
    let artifact = read_artifact_strict_at::<MechanismAnalysisReport>(directory, relative, path)
        .map_err(map_reader_artifact_error)?;
    if artifact.artifact.schema_version != 4 {
        return Err(MhiValidationError::Dataset(
            "mechanism scientific sources must be schema-4 mechanism_analysis".into(),
        ));
    }
    validate_phase_b_assessment_integrity(&artifact.artifact)?;
    Ok(artifact)
}

fn read_phase_e_health_source(
    directory: &fs::File,
    relative: &str,
    path: &Path,
) -> Result<StrictArtifactRead<SensorHealthAssessment>, MhiValidationError> {
    let artifact = read_artifact_strict_at::<SensorHealthAssessment>(directory, relative, path)
        .map_err(map_reader_artifact_error)?;
    if artifact.artifact.schema_version != 4 {
        return Err(MhiValidationError::Dataset(
            "health scientific sources must be schema-4 health_assessment".into(),
        ));
    }
    Ok(artifact)
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
        if !declared_scope_is_unknown(&record.declared_scope) {
            return Err(MhiValidationError::Dataset(
                "LegacyUnknown source requires unknown declared scope".into(),
            ));
        }
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
    Ok(dataset_directory.join(path))
}

pub(crate) fn validate_phase_b_assessment_integrity(
    report: &MechanismAnalysisReport,
) -> Result<(), MhiValidationError> {
    let mut definition_ids = BTreeSet::new();
    let mut current_ids = BTreeSet::new();
    for row in &report.hypothesis_assessments {
        if row.definition.hypothesis_id != row.current.hypothesis_id
            || !definition_ids.insert(row.definition.hypothesis_id.clone())
            || !current_ids.insert(row.current.hypothesis_id.clone())
        {
            return Err(MhiValidationError::Dataset(
                "Phase-B hypothesis ID mismatch or duplicate".into(),
            ));
        }
    }
    Ok(())
}

fn declared_scope_is_unknown(scope: &crate::results::DeclaredScopeV1) -> bool {
    matches!(
        (
            &scope.experiment_scope,
            &scope.sensor_scope,
            &scope.channel_scope,
            &scope.acquisition_families,
        ),
        (
            crate::domain::ArtifactExperimentScope::Unknown,
            crate::domain::ScopeKey::Unspecified,
            crate::domain::ScopeKey::Unspecified,
            crate::domain::ArtifactAcquisitionFamilies::Unknown,
        )
    )
}

pub(crate) fn map_reader_artifact_error(error: crate::domain::ArtifactError) -> MhiValidationError {
    match error {
        crate::domain::ArtifactError::UnsafeFile { path } => MhiValidationError::UnsafePath(path),
        other => MhiValidationError::Artifact(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn phase_e_reader_hard_fails_wrong_future_and_explicitly_excludes_legacy() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let legacy_mechanism = root.join("tests/fixtures/phase_d/legacy/mechanism_v1.json");
        let legacy_health = root.join("tests/fixtures/phase_d/legacy/health_v3.json");
        let mechanism_directory = open_strict_directory(legacy_mechanism.parent().expect("parent"))
            .expect("mechanism parent");
        let health_directory =
            open_strict_directory(legacy_health.parent().expect("parent")).expect("health parent");
        assert!(
            read_phase_e_mechanism_source(
                &mechanism_directory,
                legacy_mechanism
                    .file_name()
                    .expect("file")
                    .to_str()
                    .expect("UTF-8"),
                &legacy_mechanism,
            )
            .is_err()
        );
        assert!(
            read_phase_e_health_source(
                &health_directory,
                legacy_health
                    .file_name()
                    .expect("file")
                    .to_str()
                    .expect("UTF-8"),
                &legacy_health,
            )
            .is_err()
        );
    }
}
