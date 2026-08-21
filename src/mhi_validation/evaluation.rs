//! Filesystem-free projection of frozen Phase-B and Phase-C outcomes.

use super::{MhiValidationError, MhiValidationProtocolV1};
use crate::{
    mhi_validation::reader::ValidationInputs,
    results::{MhiValidationReportV1, ReleaseClaimResultV1},
    validation_config::{ReleaseClaimOutcomeV1, RequestedValidationLevelV1, ValidationOutcomeV1},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn evaluate_mhi_validation(
    protocol: &MhiValidationProtocolV1,
    inputs: &ValidationInputs,
) -> Result<MhiValidationReportV1, MhiValidationError> {
    let dataset = &inputs.dataset.artifact;
    let mut endpoint_outcomes = BTreeMap::new();
    let mut endpoint_payload = Vec::new();
    for endpoint in &protocol.mechanism_endpoints {
        let declared = dataset
            .records
            .iter()
            .filter(|record| {
                record.cohort_role == endpoint.cohort_role
                    && endpoint.domain.contains(&record.domain)
            })
            .collect::<Vec<_>>();
        let eligible = declared
            .iter()
            .filter(|record| record.mechanism_source.is_some())
            .count() as u64;
        let outcome = if eligible < endpoint.minimum_eligible_records {
            ValidationOutcomeV1::Indeterminate
        } else {
            ValidationOutcomeV1::MeetsProtocol
        };
        endpoint_outcomes.insert(endpoint.endpoint_id.clone(), outcome);
        endpoint_payload.push(serde_json::json!({ "endpoint_id": endpoint.endpoint_id, "endpoint_kind": "mechanism", "declared_count": declared.len(), "eligible_count": eligible, "outcome": outcome }));
    }
    for endpoint in &protocol.health_endpoints {
        let declared = dataset
            .records
            .iter()
            .filter(|record| {
                record.cohort_role == endpoint.cohort_role
                    && endpoint.domain.contains(&record.domain)
            })
            .collect::<Vec<_>>();
        let eligible = declared
            .iter()
            .filter(|record| record.health_source.is_some())
            .count() as u64;
        let outcome = if eligible < endpoint.minimum_eligible_records {
            ValidationOutcomeV1::Indeterminate
        } else {
            ValidationOutcomeV1::MeetsProtocol
        };
        endpoint_outcomes.insert(endpoint.endpoint_id.clone(), outcome);
        endpoint_payload.push(serde_json::json!({ "endpoint_id": endpoint.endpoint_id, "endpoint_kind": "health", "declared_count": declared.len(), "eligible_count": eligible, "outcome": outcome }));
    }
    let release_claims = protocol
        .release_scope
        .iter()
        .map(|claim| {
            let outcome = compose(
                claim
                    .supporting_endpoint_ids
                    .iter()
                    .filter_map(|id| endpoint_outcomes.get(id).copied()),
            );
            let outcome = match (claim.requested_level, outcome) {
                (RequestedValidationLevelV1::Software, ValidationOutcomeV1::MeetsProtocol) => {
                    ReleaseClaimOutcomeV1::SoftwareValidatedOnly
                }
                (RequestedValidationLevelV1::Physical, ValidationOutcomeV1::MeetsProtocol) => {
                    ReleaseClaimOutcomeV1::PhysicallyValidated
                }
                (_, ValidationOutcomeV1::DoesNotMeetProtocol) => {
                    ReleaseClaimOutcomeV1::DoesNotMeetProtocol
                }
                (_, ValidationOutcomeV1::Indeterminate) => ReleaseClaimOutcomeV1::Indeterminate,
            };
            ReleaseClaimResultV1 {
                claim_id: claim.claim_id.clone(),
                requested_level: claim.requested_level,
                outcome,
            }
        })
        .collect::<Vec<_>>();
    let overall_status = compose(endpoint_outcomes.values().copied());
    let payload = serde_json::json!({ "endpoint_results": endpoint_payload, "mechanism_source_count": inputs.mechanism_sources.len(), "health_source_count": inputs.health_sources.len() });
    let preimage = serde_jcs::to_vec(&serde_json::json!({ "identity_domain": "mhi_validation_report_v1", "protocol_sha256": protocol_sha(&inputs.dataset.artifact.protocol_sha256), "dataset_id": dataset.dataset_id, "payload": payload, "release_claims": release_claims, "overall_status": overall_status })).map_err(|error| MhiValidationError::Dataset(error.to_string()))?;
    let mut hash = Sha256::new();
    hash.update(preimage);
    let report = MhiValidationReportV1 {
        schema_version: 1,
        artifact_kind: "mhi_validation_report".into(),
        report_id: format!("sha256:{:x}", hash.finalize()),
        protocol_sha256: dataset.protocol_sha256.clone(),
        dataset_id: dataset.dataset_id.clone(),
        dataset_source_file_sha256: inputs.dataset.source_file_sha256.clone(),
        approval_trust_store_sha256: None,
        release_claims,
        overall_status,
        payload,
        lineage: dataset.lineage.clone(),
        provenance: serde_json::json!({ "software_version": env!("CARGO_PKG_VERSION") }),
        warnings: Vec::new(),
    };
    report.validate_structure()?;
    Ok(report)
}

fn protocol_sha(value: &str) -> &str {
    value
}
fn compose(outcomes: impl Iterator<Item = ValidationOutcomeV1>) -> ValidationOutcomeV1 {
    let outcomes = outcomes.collect::<Vec<_>>();
    if outcomes.contains(&ValidationOutcomeV1::Indeterminate) {
        ValidationOutcomeV1::Indeterminate
    } else if outcomes.contains(&ValidationOutcomeV1::DoesNotMeetProtocol) {
        ValidationOutcomeV1::DoesNotMeetProtocol
    } else {
        ValidationOutcomeV1::MeetsProtocol
    }
}
