//! Phase-E certified validation runner.

use crate::{
    mhi_validation::{
        MhiValidationError, MhiValidationProtocolV1, ValidationInputs,
        approval::{OwnerApprovalEvidenceV1, PhysicalApprovalTrustStoreV1},
        evaluate_mhi_validation,
        output::publish_bundle,
    },
    validation_config::RequestedValidationLevelV1,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct MhiValidationRunOptions {
    pub protocol: PathBuf,
    pub dataset: PathBuf,
    pub output_dir: PathBuf,
    pub overwrite: bool,
}

pub fn run_mhi_validation(options: MhiValidationRunOptions) -> Result<(), MhiValidationError> {
    validate_root_file(&options.protocol)?;
    validate_root_file(&options.dataset)?;
    if options.protocol == options.dataset
        || options.output_dir == options.protocol
        || options.output_dir == options.dataset
    {
        return Err(MhiValidationError::UnsafePath(options.output_dir));
    }
    let protocol_bytes = fs::read(&options.protocol).map_err(|source| MhiValidationError::Io {
        path: options.protocol.clone(),
        source,
    })?;
    let protocol_text = std::str::from_utf8(&protocol_bytes)
        .map_err(|_| MhiValidationError::Protocol("protocol must be UTF-8".into()))?;
    let protocol = MhiValidationProtocolV1::from_toml(protocol_text)?;
    let protocol_sha256 = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);
    let physical = protocol
        .release_scope
        .iter()
        .any(|claim| claim.requested_level == RequestedValidationLevelV1::Physical);
    let trust = if physical {
        Some(PhysicalApprovalTrustStoreV1::from_embedded_bytes()?)
    } else {
        None
    };
    let inputs = ValidationInputs::read(&protocol, &protocol_sha256, &options.dataset)?;
    if let Some(trust) = &trust {
        let source = inputs
            .dataset
            .artifact
            .owner_approval_source
            .as_ref()
            .ok_or_else(|| {
                MhiValidationError::Approval(
                    "physical validation requires owner_approval_source".into(),
                )
            })?;
        let path = crate::mhi_validation::reader::safe_dataset_relative_path(
            &inputs.dataset_directory,
            &source.relative_path,
        )?;
        OwnerApprovalEvidenceV1::read_and_validate(
            &path,
            &source.source_file_sha256,
            trust,
            &protocol,
            &inputs.dataset.artifact,
        )?;
    }
    let mut report = evaluate_mhi_validation(&protocol, &inputs)?;
    if let Some(trust) = trust {
        report.approval_trust_store_sha256 = Some(trust.source_file_sha256);
    }
    publish_bundle(
        &options.output_dir,
        &report,
        &protocol.protocol_id,
        options.overwrite,
    )
}

fn validate_root_file(path: &Path) -> Result<(), MhiValidationError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| MhiValidationError::Io {
        path: path.into(),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(MhiValidationError::UnsafePath(path.into()));
    }
    Ok(())
}
