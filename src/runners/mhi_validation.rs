//! Phase-E certified validation runner.

use crate::{
    mhi_validation::{
        MhiValidationError, MhiValidationProtocolV1, ValidationInputs,
        approval::{OwnerApprovalEvidenceV1, PhysicalApprovalTrustStoreV1},
        evaluate_mhi_validation,
        output::{authorize_publication, publish_authorized_bundle},
    },
    validation_config::RequestedValidationLevelV1,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MhiValidationRunOptions {
    pub protocol: PathBuf,
    pub dataset: PathBuf,
    pub output_dir: PathBuf,
    pub overwrite: bool,
}

pub fn run_mhi_validation(options: MhiValidationRunOptions) -> Result<(), MhiValidationError> {
    if options.protocol == options.dataset
        || options.output_dir == options.protocol
        || options.output_dir == options.dataset
    {
        return Err(MhiValidationError::UnsafePath(options.output_dir));
    }
    let protocol_bytes = crate::domain::read_strict_file_bytes(&options.protocol)
        .map_err(crate::mhi_validation::reader::map_reader_artifact_error)?;
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
        if !trust.is_provisioned() {
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
        let approval = OwnerApprovalEvidenceV1::read_and_validate_at(
            &inputs.dataset_directory_authority,
            &source.relative_path,
            &path,
            &source.source_file_sha256,
            trust,
            &protocol,
            &inputs.dataset.artifact,
        )?;
        if approval.approval_record_id() != source.expected_approval_record_id {
            return Err(MhiValidationError::Approval(
                "approval expected record ID mismatch".into(),
            ));
        }
        inputs.attach_verified_approval(approval);
    }
    let report = evaluate_mhi_validation(&protocol, &inputs)?;
    let authorization = authorize_publication(&report, &protocol, &inputs)?;
    publish_authorized_bundle(&options.output_dir, &authorization, options.overwrite)
}
