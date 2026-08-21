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
        let trust = PhysicalApprovalTrustStoreV1::from_embedded_bytes()?;
        // This gate is intentionally before dataset opening, approval parsing,
        // evaluation, or report creation.  Production never falls back to a
        // software claim and cannot accept a runtime-supplied test root.
        if !trust.store.is_provisioned() {
            return Err(MhiValidationError::PhysicalApprovalTrustNotProvisioned);
        }
        Some(trust)
    } else {
        None
    };
    // A physical request against the shipped UNPROVISIONED authority stops
    // above this line.  In particular, an attacker cannot obtain a different
    // error (or make the reader inspect a supplied path) by choosing a missing
    // or malformed dataset.
    validate_root_file(&options.dataset)?;
    let mut inputs = ValidationInputs::read(&protocol, &protocol_sha256, &options.dataset)?;
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
        let approval = OwnerApprovalEvidenceV1::read_and_validate(
            &path,
            &source.source_file_sha256,
            trust,
            &protocol,
            &inputs.dataset.artifact,
        )?;
        if approval.approval_record_id != source.expected_approval_record_id {
            return Err(MhiValidationError::Approval(
                "approval expected record ID mismatch".into(),
            ));
        }
        inputs.attach_verified_approval(approval, trust.source_file_sha256.clone());
    }
    let report = evaluate_mhi_validation(&protocol, &inputs)?;
    report.validate_against(&protocol, &inputs, trust.as_ref())?;
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
