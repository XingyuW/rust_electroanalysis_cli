//! Filesystem boundary for Phase-E inputs.

use super::{MhiValidationError, MhiValidationProtocolV1};
use crate::{
    domain::{StrictArtifactRead, read_artifact_strict},
    results::{MechanismAnalysisReport, MhiValidationDatasetV1, SensorHealthAssessment},
};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug)]
pub struct ValidationInputs {
    pub dataset: StrictArtifactRead<MhiValidationDatasetV1>,
    pub dataset_directory: PathBuf,
    pub mechanism_sources: Vec<(String, StrictArtifactRead<MechanismAnalysisReport>)>,
    pub health_sources: Vec<(String, StrictArtifactRead<SensorHealthAssessment>)>,
}

impl ValidationInputs {
    pub fn read(
        protocol: &MhiValidationProtocolV1,
        protocol_sha256: &str,
        dataset_path: &Path,
    ) -> Result<Self, MhiValidationError> {
        let dataset = read_artifact_strict::<MhiValidationDatasetV1>(dataset_path)?;
        if dataset.artifact.protocol_sha256 != protocol_sha256 {
            return Err(MhiValidationError::Dataset(
                "dataset protocol_sha256 does not bind the exact protocol bytes".into(),
            ));
        }
        let dataset_directory = canonical_regular_file(dataset_path)?
            .parent()
            .expect("canonical file has parent")
            .to_path_buf();
        let mut mechanism_sources = Vec::new();
        let mut health_sources = Vec::new();
        for record in &dataset.artifact.records {
            if let Some(source) = &record.mechanism_source {
                if source.expected_artifact_kind != "mechanism_analysis"
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
                mechanism_sources.push((record.record_id.clone(), artifact));
            }
            if let Some(source) = &record.health_source {
                if source.expected_artifact_kind != "health_assessment"
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
                health_sources.push((record.record_id.clone(), artifact));
            }
        }
        // The protocol is intentionally consumed to make the boundary explicit;
        // all protocol semantic checks run before this source read.
        let _ = protocol;
        Ok(Self {
            dataset,
            dataset_directory,
            mechanism_sources,
            health_sources,
        })
    }
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
