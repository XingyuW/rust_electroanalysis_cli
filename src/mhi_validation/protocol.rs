use super::error::MhiValidationError;
use crate::validation_config::{
    AcceptanceRuleV1, CohortRoleV1, HealthEndpointV1, MechanismEndpointV1,
    PhysicalApprovalAuthorityV1, ReferenceAuthorityRuleV1, ReleaseClaimV1,
    RequestedValidationLevelV1,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolRegistrationV1 {
    pub registration_id: String,
    pub immutable_reference_uri: String,
    pub document_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatisticsV1 {
    pub interval_method: String,
    pub confidence_level: String,
    pub undefined_metric: String,
    pub required_rule_unavailable: String,
    pub rule_composition: String,
}

/// The complete, closed TOML protocol boundary.  Hashes intentionally cover
/// the original bytes, not a reserialized TOML representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MhiValidationProtocolV1 {
    pub schema_version: u32,
    pub protocol_id: String,
    pub title: String,
    pub registration: ProtocolRegistrationV1,
    pub physical_approval_authority: PhysicalApprovalAuthorityV1,
    pub target_domain: crate::validation_config::DomainSelectorV1,
    pub mechanism_endpoints: Vec<MechanismEndpointV1>,
    pub health_endpoints: Vec<HealthEndpointV1>,
    pub statistics: StatisticsV1,
    pub release_scope: Vec<ReleaseClaimV1>,
}

impl MhiValidationProtocolV1 {
    pub fn from_toml(text: &str) -> Result<Self, MhiValidationError> {
        if text.as_bytes().starts_with(&[0xef, 0xbb, 0xbf]) {
            return Err(MhiValidationError::Protocol(
                "UTF-8 BOM is forbidden".into(),
            ));
        }
        let protocol: Self = toml::from_str(text)?;
        protocol.validate()?;
        Ok(protocol)
    }

    pub fn sha256_of_bytes(bytes: &[u8]) -> String {
        let mut hash = Sha256::new();
        hash.update(bytes);
        format!("{:x}", hash.finalize())
    }

    pub fn validate(&self) -> Result<(), MhiValidationError> {
        if self.schema_version != 1 {
            return Err(protocol("schema_version must be 1"));
        }
        valid_id("protocol_id", &self.protocol_id)?;
        nonempty("title", &self.title)?;
        valid_id(
            "registration.registration_id",
            &self.registration.registration_id,
        )?;
        valid_uri(
            "registration.immutable_reference_uri",
            &self.registration.immutable_reference_uri,
        )?;
        sha(
            "registration.document_sha256",
            &self.registration.document_sha256,
        )?;
        if self.mechanism_endpoints.is_empty()
            || self.health_endpoints.is_empty()
            || self.release_scope.is_empty()
        {
            return Err(protocol(
                "mechanism_endpoints, health_endpoints, and release_scope must be nonempty",
            ));
        }
        if self.statistics.interval_method != "wilson_95_v1"
            || self.statistics.confidence_level != "0.95"
            || self.statistics.undefined_metric != "unavailable"
            || self.statistics.required_rule_unavailable != "indeterminate"
            || self.statistics.rule_composition != "and"
        {
            return Err(protocol(
                "statistics must use the frozen wilson_95_v1 contract",
            ));
        }
        let mut endpoints = BTreeSet::new();
        for endpoint in &self.mechanism_endpoints {
            validate_mechanism_endpoint(endpoint)?;
            if !endpoints.insert(endpoint.endpoint_id.clone()) {
                return Err(protocol("endpoint IDs must be globally unique"));
            }
        }
        for endpoint in &self.health_endpoints {
            validate_health_endpoint(endpoint)?;
            if !endpoints.insert(endpoint.endpoint_id.clone()) {
                return Err(protocol("endpoint IDs must be globally unique"));
            }
        }
        let physical = self
            .release_scope
            .iter()
            .any(|claim| claim.requested_level == RequestedValidationLevelV1::Physical);
        match (&self.physical_approval_authority, physical) {
            (PhysicalApprovalAuthorityV1::NotRequested, false) => {}
            (PhysicalApprovalAuthorityV1::EmbeddedTrustRoot { trust_root_id }, true) => {
                valid_id("physical_approval_authority.trust_root_id", trust_root_id)?
            }
            _ => {
                return Err(protocol(
                    "physical approval authority must be requested iff a claim is physical",
                ));
            }
        }
        let mut claims = BTreeSet::new();
        let mut supported = BTreeSet::new();
        for claim in &self.release_scope {
            valid_id("claim_id", &claim.claim_id)?;
            nonempty("claim.statement", &claim.statement)?;
            if !claims.insert(claim.claim_id.clone()) {
                return Err(protocol("release claim IDs must be unique"));
            }
            unique_nonempty(
                "claim.supporting_endpoint_ids",
                &claim.supporting_endpoint_ids,
            )?;
            for id in &claim.supporting_endpoint_ids {
                if !endpoints.contains(id) {
                    return Err(protocol("release claim references an unknown endpoint"));
                }
                supported.insert(id.clone());
                if claim.requested_level == RequestedValidationLevelV1::Physical {
                    let mechanism = self
                        .mechanism_endpoints
                        .iter()
                        .find(|endpoint| &endpoint.endpoint_id == id);
                    let health = self
                        .health_endpoints
                        .iter()
                        .find(|endpoint| &endpoint.endpoint_id == id);
                    let (role, min_records, min_families, domain) =
                        if let Some(endpoint) = mechanism {
                            (
                                endpoint.cohort_role,
                                endpoint.minimum_eligible_records,
                                endpoint.minimum_independent_families,
                                &endpoint.domain,
                            )
                        } else if let Some(endpoint) = health {
                            (
                                endpoint.cohort_role,
                                endpoint.minimum_eligible_records,
                                endpoint.minimum_independent_families,
                                &endpoint.domain,
                            )
                        } else {
                            unreachable!("endpoint was checked above")
                        };
                    if role != CohortRoleV1::Holdout
                        || min_records < 2
                        || min_families < 2
                        || domain != &claim.domain
                    {
                        return Err(protocol(
                            "physical claims require domain-equal holdout endpoints with minima of two",
                        ));
                    }
                }
            }
        }
        if supported != endpoints {
            return Err(protocol(
                "every endpoint must support at least one release claim",
            ));
        }
        Ok(())
    }
}

fn validate_mechanism_endpoint(endpoint: &MechanismEndpointV1) -> Result<(), MhiValidationError> {
    valid_common_endpoint(
        &endpoint.endpoint_id,
        endpoint.cohort_role,
        endpoint.minimum_eligible_records,
        endpoint.minimum_independent_families,
        &endpoint.required_strata,
        &endpoint.acceptance_rules,
    )?;
    valid_id("hypothesis_id", &endpoint.hypothesis_id)?;
    if !endpoint.mechanism_artifact_required
        || endpoint.critical_policy != "any_contradicted_record_fails"
    {
        return Err(protocol(
            "mechanism endpoint has an invalid frozen authority",
        ));
    }
    if !matches!(
        endpoint.reference_rule,
        ReferenceAuthorityRuleV1::Mechanism { .. }
    ) {
        return Err(protocol(
            "mechanism endpoint requires a mechanism reference rule",
        ));
    }
    unique_nonempty("support_levels", &endpoint.support_levels)?;
    if endpoint.support_levels.iter().any(|level| {
        !matches!(
            level.as_str(),
            "hypothesized" | "experimentally_supported" | "validated_for_domain"
        )
    }) {
        return Err(protocol(
            "support_levels contains an invalid evidence level",
        ));
    }
    validate_reference_rule(&endpoint.reference_rule)
}

fn validate_health_endpoint(endpoint: &HealthEndpointV1) -> Result<(), MhiValidationError> {
    valid_common_endpoint(
        &endpoint.endpoint_id,
        endpoint.cohort_role,
        endpoint.minimum_eligible_records,
        endpoint.minimum_independent_families,
        &endpoint.required_strata,
        &endpoint.acceptance_rules,
    )?;
    if !endpoint.health_artifact_required
        || !matches!(
            endpoint.reference_rule,
            ReferenceAuthorityRuleV1::Health { .. }
        )
    {
        return Err(protocol(
            "health endpoint requires health artifact and reference rule",
        ));
    }
    let all = ["within_baseline", "watch", "degraded", "critical"];
    unique_nonempty(
        "predicted_positive_statuses",
        &endpoint.predicted_positive_statuses,
    )?;
    unique_nonempty(
        "predicted_negative_statuses",
        &endpoint.predicted_negative_statuses,
    )?;
    let set = endpoint
        .predicted_positive_statuses
        .iter()
        .chain(&endpoint.predicted_negative_statuses)
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if set.len() != all.len() || all.iter().any(|value| !set.contains(value)) {
        return Err(protocol(
            "health status sets must be a disjoint exact partition",
        ));
    }
    unique_nonempty(
        "reference_label_universe",
        &endpoint.reference_label_universe,
    )?;
    unique_nonempty(
        "reference_positive_labels",
        &endpoint.reference_positive_labels,
    )?;
    unique_nonempty(
        "reference_negative_labels",
        &endpoint.reference_negative_labels,
    )?;
    let labels = endpoint
        .reference_positive_labels
        .iter()
        .chain(&endpoint.reference_negative_labels)
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if labels.len() != endpoint.reference_label_universe.len()
        || endpoint
            .reference_label_universe
            .iter()
            .any(|label| !labels.contains(label.as_str()))
    {
        return Err(protocol(
            "health reference classes must be a disjoint exact partition",
        ));
    }
    validate_reference_rule(&endpoint.reference_rule)
}

fn valid_common_endpoint(
    endpoint_id: &str,
    role: CohortRoleV1,
    minimum_records: u64,
    minimum_families: u64,
    strata: &[crate::validation_config::RequiredStratumV1],
    rules: &[AcceptanceRuleV1],
) -> Result<(), MhiValidationError> {
    valid_id("endpoint_id", endpoint_id)?;
    if role == CohortRoleV1::Development {
        return Err(protocol("development is not scoreable"));
    }
    if minimum_records == 0 || minimum_families == 0 {
        return Err(protocol("endpoint minima must be positive"));
    }
    let mut ids = BTreeSet::new();
    for stratum in strata {
        valid_id("stratum_id", &stratum.stratum_id)?;
        if stratum.minimum_eligible_records == 0
            || stratum.minimum_independent_families == 0
            || !ids.insert(stratum.stratum_id.clone())
        {
            return Err(protocol("strata must be unique and have positive minima"));
        }
    }
    let mut rule_ids = BTreeSet::new();
    for rule in rules {
        match rule {
            AcceptanceRuleV1::Count { rule_id, .. } => valid_id("rule_id", rule_id)?,
            AcceptanceRuleV1::Rate {
                rule_id, threshold, ..
            } => {
                valid_id("rule_id", rule_id)?;
                finite_unit_interval("rate threshold", *threshold)?;
            }
        }
        let id = match rule {
            AcceptanceRuleV1::Count { rule_id, .. } | AcceptanceRuleV1::Rate { rule_id, .. } => {
                rule_id
            }
        };
        if !rule_ids.insert(id.clone()) {
            return Err(protocol("acceptance-rule IDs must be unique"));
        }
    }
    Ok(())
}

fn validate_reference_rule(rule: &ReferenceAuthorityRuleV1) -> Result<(), MhiValidationError> {
    let (methods, authorities, uncertainty) = match rule {
        ReferenceAuthorityRuleV1::Mechanism {
            allowed_methods,
            allowed_authority_ids,
            uncertainty_rule,
            ..
        }
        | ReferenceAuthorityRuleV1::Health {
            allowed_methods,
            allowed_authority_ids,
            uncertainty_rule,
            ..
        } => (allowed_methods, allowed_authority_ids, uncertainty_rule),
    };
    if methods.is_empty() {
        return Err(protocol("allowed_methods must be nonempty"));
    }
    unique_nonempty("allowed_authority_ids", authorities)?;
    for method in methods {
        valid_id("method_id", &method.method_id)?;
        nonempty("method_version", &method.method_version)?;
    }
    if let crate::validation_config::ReferenceUncertaintyRuleV1::RequireQuantified {
        measure_id,
        unit,
        maximum_inclusive,
    } = uncertainty
    {
        valid_id("uncertainty.measure_id", measure_id)?;
        nonempty("uncertainty.unit", unit)?;
        if !maximum_inclusive.is_finite()
            || *maximum_inclusive < 0.0
            || maximum_inclusive.to_bits() == (-0.0f64).to_bits()
        {
            return Err(protocol(
                "uncertainty maximum must be finite and nonnegative",
            ));
        }
    }
    Ok(())
}

fn valid_id(name: &str, value: &str) -> Result<(), MhiValidationError> {
    if value.is_empty()
        || !value.bytes().enumerate().all(|(index, byte)| {
            (byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
                && (index != 0 || byte.is_ascii_alphanumeric())
        })
    {
        return Err(protocol(format!("{name} must be a stable ID")));
    }
    Ok(())
}
fn nonempty(name: &str, value: &str) -> Result<(), MhiValidationError> {
    if value.is_empty()
        || value.contains('\0')
        || value.contains('\r')
        || value
            .lines()
            .any(|line| line.ends_with(char::is_whitespace))
    {
        Err(protocol(format!("{name} must be nonempty canonical text")))
    } else {
        Ok(())
    }
}
fn sha(name: &str, value: &str) -> Result<(), MhiValidationError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Err(protocol(format!("{name} must be lowercase SHA-256")))
    } else {
        Ok(())
    }
}
fn valid_uri(name: &str, value: &str) -> Result<(), MhiValidationError> {
    if value
        .bytes()
        .next()
        .is_none_or(|byte| !byte.is_ascii_alphabetic())
        || !value.contains(':')
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        Err(protocol(format!("{name} must be an opaque printable URI")))
    } else {
        Ok(())
    }
}
fn finite_unit_interval(name: &str, value: f64) -> Result<(), MhiValidationError> {
    if !value.is_finite() || value.to_bits() == (-0.0f64).to_bits() || !(0.0..=1.0).contains(&value)
    {
        Err(protocol(format!("{name} must be finite in [0,1]")))
    } else {
        Ok(())
    }
}
fn unique_nonempty(name: &str, values: &[String]) -> Result<(), MhiValidationError> {
    if values.is_empty()
        || values.iter().any(|value| valid_id(name, value).is_err())
        || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        Err(protocol(format!(
            "{name} must be sorted, nonempty, and duplicate-free"
        )))
    } else {
        Ok(())
    }
}
fn protocol(message: impl Into<String>) -> MhiValidationError {
    MhiValidationError::Protocol(message.into())
}
