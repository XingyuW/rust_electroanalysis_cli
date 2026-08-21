use super::error::MhiValidationError;
use crate::validation_config::{
    AcceptanceRuleV1, CohortRoleV1, CountMetricV1, DomainSelectorV1, HealthEndpointV1,
    MechanismEndpointV1, PhysicalApprovalAuthorityV1, RateMetricV1, RateTargetV1,
    ReferenceAuthorityRuleV1, ReleaseClaimV1, RequestedValidationLevelV1, StratumPredicateV1,
    TemperatureSelectorV1,
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
        validate_domain("target_domain", &self.target_domain)?;
        let mut endpoints = BTreeSet::new();
        let mut previous_mechanism_endpoint = None;
        for endpoint in &self.mechanism_endpoints {
            validate_mechanism_endpoint(endpoint)?;
            validate_domain("mechanism endpoint domain", &endpoint.domain)?;
            if !domain_is_subset(&endpoint.domain, &self.target_domain) {
                return Err(protocol("mechanism endpoint domain exceeds target_domain"));
            }
            if previous_mechanism_endpoint
                .as_ref()
                .is_some_and(|previous: &String| previous >= &endpoint.endpoint_id)
            {
                return Err(protocol("mechanism endpoints must be canonically ordered"));
            }
            previous_mechanism_endpoint = Some(endpoint.endpoint_id.clone());
            if !endpoints.insert(endpoint.endpoint_id.clone()) {
                return Err(protocol("endpoint IDs must be globally unique"));
            }
        }
        let mut previous_health_endpoint = None;
        for endpoint in &self.health_endpoints {
            validate_health_endpoint(endpoint)?;
            validate_domain("health endpoint domain", &endpoint.domain)?;
            if !domain_is_subset(&endpoint.domain, &self.target_domain) {
                return Err(protocol("health endpoint domain exceeds target_domain"));
            }
            if previous_health_endpoint
                .as_ref()
                .is_some_and(|previous: &String| previous >= &endpoint.endpoint_id)
            {
                return Err(protocol("health endpoints must be canonically ordered"));
            }
            previous_health_endpoint = Some(endpoint.endpoint_id.clone());
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
        let mut previous_claim = None;
        for claim in &self.release_scope {
            valid_id("claim_id", &claim.claim_id)?;
            nonempty("claim.statement", &claim.statement)?;
            validate_domain("release claim domain", &claim.domain)?;
            if !domain_is_subset(&claim.domain, &self.target_domain) {
                return Err(protocol("release claim domain exceeds target_domain"));
            }
            if previous_claim
                .as_ref()
                .is_some_and(|previous: &String| previous >= &claim.claim_id)
            {
                return Err(protocol("release claims must be canonically ordered"));
            }
            previous_claim = Some(claim.claim_id.clone());
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
                let endpoint_domain = self
                    .mechanism_endpoints
                    .iter()
                    .find(|endpoint| &endpoint.endpoint_id == id)
                    .map(|endpoint| &endpoint.domain)
                    .or_else(|| {
                        self.health_endpoints
                            .iter()
                            .find(|endpoint| &endpoint.endpoint_id == id)
                            .map(|endpoint| &endpoint.domain)
                    })
                    .expect("endpoint exists after membership check");
                if endpoint_domain != &claim.domain {
                    return Err(protocol(
                        "supporting endpoint domain must exactly equal claim domain",
                    ));
                }
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
                    if let Some(endpoint) = mechanism {
                        validate_physical_endpoint_rules(
                            &endpoint.reference_rule,
                            &endpoint.required_strata,
                        )?;
                    } else if let Some(endpoint) = health {
                        validate_physical_endpoint_rules(
                            &endpoint.reference_rule,
                            &endpoint.required_strata,
                        )?;
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
    validate_reference_rule(&endpoint.reference_rule)?;
    validate_acceptance_metrics(&endpoint.acceptance_rules, true)
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
    validate_reference_rule(&endpoint.reference_rule)?;
    validate_acceptance_metrics(&endpoint.acceptance_rules, false)
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
        validate_stratum(stratum)?;
    }
    if strata
        .windows(2)
        .any(|pair| pair[0].stratum_id >= pair[1].stratum_id)
    {
        return Err(protocol("required strata must be canonically ordered"));
    }
    if rules.is_empty() {
        return Err(protocol("acceptance_rules must be nonempty"));
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
    if rules
        .windows(2)
        .any(|pair| rule_id(&pair[0]) >= rule_id(&pair[1]))
    {
        return Err(protocol("acceptance rules must be canonically ordered"));
    }
    validate_rule_constraints(rules)
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

fn validate_physical_endpoint_rules(
    rule: &ReferenceAuthorityRuleV1,
    strata: &[crate::validation_config::RequiredStratumV1],
) -> Result<(), MhiValidationError> {
    let (blinding, uncertainty) = match rule {
        ReferenceAuthorityRuleV1::Mechanism {
            blinding_rule,
            uncertainty_rule,
            ..
        }
        | ReferenceAuthorityRuleV1::Health {
            blinding_rule,
            uncertainty_rule,
            ..
        } => (blinding_rule, uncertainty_rule),
    };
    if *blinding != crate::validation_config::BlindingRuleV1::RequireBlinded
        || !matches!(
            uncertainty,
            crate::validation_config::ReferenceUncertaintyRuleV1::RequireQuantified { .. }
        )
        || strata.iter().any(|stratum| {
            stratum.minimum_eligible_records < 2 || stratum.minimum_independent_families < 2
        })
    {
        return Err(protocol(
            "physical endpoints require blinded quantified references and strata minima of two",
        ));
    }
    Ok(())
}

fn validate_stratum(
    stratum: &crate::validation_config::RequiredStratumV1,
) -> Result<(), MhiValidationError> {
    if stratum.predicates.is_empty() {
        return Err(protocol("required stratum predicates must be nonempty"));
    }
    let mut previous = None;
    for predicate in &stratum.predicates {
        match predicate {
            StratumPredicateV1::AnalyteEquals { id }
            | StratumPredicateV1::MatrixEquals { id }
            | StratumPredicateV1::SensorDesignEquals { id }
            | StratumPredicateV1::SensorEquals { id }
            | StratumPredicateV1::CampaignEquals { id } => valid_id("stratum predicate ID", id)?,
            StratumPredicateV1::TemperatureBand {
                lower_kelvin_inclusive,
                upper_kelvin_exclusive,
            } => validate_temperature_band(
                "stratum temperature band",
                *lower_kelvin_inclusive,
                *upper_kelvin_exclusive,
            )?,
        }
        let current = predicate.discriminant();
        if previous.is_some_and(|last| last >= current) {
            return Err(protocol(
                "stratum predicates must be canonically ordered and use each axis once",
            ));
        }
        previous = Some(current);
    }
    Ok(())
}

fn validate_acceptance_metrics(
    rules: &[AcceptanceRuleV1],
    mechanism: bool,
) -> Result<(), MhiValidationError> {
    let mut support = false;
    let mut coverage = false;
    let mut sensitivity = false;
    let mut specificity = false;
    for rule in rules {
        match rule {
            AcceptanceRuleV1::Count { metric, .. } => {
                let mechanism_only = matches!(
                    metric,
                    CountMetricV1::SupportCount
                        | CountMetricV1::CriticalContradictionCount
                        | CountMetricV1::NotAssessedOrOtherCount
                );
                let health_only = matches!(
                    metric,
                    CountMetricV1::Tp
                        | CountMetricV1::Tn
                        | CountMetricV1::Fp
                        | CountMetricV1::Fn
                        | CountMetricV1::IndeterminateCount
                        | CountMetricV1::DataQualityInsufficientCount
                        | CountMetricV1::EvaluableCount
                );
                if mechanism_only && !mechanism || health_only && mechanism {
                    return Err(protocol("acceptance metric is not valid for endpoint kind"));
                }
            }
            AcceptanceRuleV1::Rate {
                metric,
                target,
                comparator,
                ..
            } => {
                let mechanism_only = matches!(
                    metric,
                    RateMetricV1::SupportFraction
                        | RateMetricV1::ContradictionFraction
                        | RateMetricV1::NotAssessedFraction
                );
                let health_only = matches!(
                    metric,
                    RateMetricV1::Coverage
                        | RateMetricV1::IndeterminateRate
                        | RateMetricV1::DataQualityInsufficientRate
                        | RateMetricV1::Sensitivity
                        | RateMetricV1::Specificity
                        | RateMetricV1::FalsePositiveRate
                        | RateMetricV1::FalseNegativeRate
                        | RateMetricV1::BalancedAccuracy
                );
                if mechanism_only && !mechanism || health_only && mechanism {
                    return Err(protocol("acceptance metric is not valid for endpoint kind"));
                }
                if *metric == RateMetricV1::BalancedAccuracy
                    && *target != RateTargetV1::PointEstimate
                {
                    return Err(protocol("balanced_accuracy has point_estimate only"));
                }
                if mechanism
                    && *metric == RateMetricV1::SupportFraction
                    && *target == RateTargetV1::PointEstimate
                    && *comparator == crate::validation_config::ComparatorV1::GreaterThanOrEqual
                {
                    support = true;
                }
                if !mechanism
                    && *comparator == crate::validation_config::ComparatorV1::GreaterThanOrEqual
                {
                    match metric {
                        RateMetricV1::Coverage => coverage = true,
                        RateMetricV1::Sensitivity => sensitivity = true,
                        RateMetricV1::Specificity => specificity = true,
                        _ => {}
                    }
                }
            }
        }
    }
    if mechanism && !support {
        return Err(protocol(
            "mechanism endpoints require a support_fraction greater_than_or_equal rule",
        ));
    }
    if !mechanism && !(coverage && sensitivity && specificity) {
        return Err(protocol(
            "health endpoints require coverage, sensitivity, and specificity greater_than_or_equal rules",
        ));
    }
    Ok(())
}

fn validate_rule_constraints(rules: &[AcceptanceRuleV1]) -> Result<(), MhiValidationError> {
    use crate::validation_config::ComparatorV1;
    for lower in rules {
        for upper in rules {
            match (lower, upper) {
                (
                    AcceptanceRuleV1::Count {
                        metric: lower_metric,
                        comparator: ComparatorV1::GreaterThanOrEqual,
                        threshold_u64: lower_threshold,
                        ..
                    },
                    AcceptanceRuleV1::Count {
                        metric: upper_metric,
                        comparator: ComparatorV1::LessThanOrEqual,
                        threshold_u64: upper_threshold,
                        ..
                    },
                ) if lower_metric == upper_metric && lower_threshold > upper_threshold => {
                    return Err(protocol("acceptance-rule bounds are contradictory"));
                }
                (
                    AcceptanceRuleV1::Rate {
                        metric: lower_metric,
                        target: lower_target,
                        comparator: ComparatorV1::GreaterThanOrEqual,
                        threshold: lower_threshold,
                        ..
                    },
                    AcceptanceRuleV1::Rate {
                        metric: upper_metric,
                        target: upper_target,
                        comparator: ComparatorV1::LessThanOrEqual,
                        threshold: upper_threshold,
                        ..
                    },
                ) if lower_metric == upper_metric
                    && lower_target == upper_target
                    && lower_threshold > upper_threshold =>
                {
                    return Err(protocol("acceptance-rule bounds are contradictory"));
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn rule_id(rule: &AcceptanceRuleV1) -> &str {
    match rule {
        AcceptanceRuleV1::Count { rule_id, .. } | AcceptanceRuleV1::Rate { rule_id, .. } => rule_id,
    }
}

fn validate_domain(name: &str, domain: &DomainSelectorV1) -> Result<(), MhiValidationError> {
    use crate::validation_config::CategoricalSelectorV1;
    for (axis, selector) in [
        ("analyte", &domain.analyte),
        ("matrix", &domain.matrix),
        ("sensor_design", &domain.sensor_design),
        ("sensor", &domain.sensor),
        ("campaign", &domain.campaign),
    ] {
        if let CategoricalSelectorV1::Allowed { ids } = selector {
            unique_nonempty(&format!("{name}.{axis}.ids"), ids)?;
        }
    }
    if let TemperatureSelectorV1::Bands { bands } = &domain.temperature {
        if bands.is_empty() {
            return Err(protocol(format!(
                "{name}.temperature bands must be nonempty"
            )));
        }
        let mut previous_upper = None;
        for band in bands {
            validate_temperature_band(
                &format!("{name}.temperature band"),
                band.lower_kelvin_inclusive,
                band.upper_kelvin_exclusive,
            )?;
            if previous_upper.is_some_and(|upper| upper >= band.lower_kelvin_inclusive) {
                return Err(protocol(format!(
                    "{name}.temperature bands must be ordered and non-overlapping"
                )));
            }
            previous_upper = Some(band.upper_kelvin_exclusive);
        }
    }
    Ok(())
}

fn validate_temperature_band(name: &str, lower: f64, upper: f64) -> Result<(), MhiValidationError> {
    if !lower.is_finite()
        || !upper.is_finite()
        || lower <= 0.0
        || upper <= 0.0
        || lower.to_bits() == (-0.0f64).to_bits()
        || upper.to_bits() == (-0.0f64).to_bits()
        || lower >= upper
    {
        return Err(protocol(format!(
            "{name} must be finite positive lower < upper"
        )));
    }
    Ok(())
}

fn domain_is_subset(left: &DomainSelectorV1, right: &DomainSelectorV1) -> bool {
    use crate::validation_config::CategoricalSelectorV1;
    fn categorical(left: &CategoricalSelectorV1, right: &CategoricalSelectorV1) -> bool {
        match (left, right) {
            (_, CategoricalSelectorV1::AnyDeclared) => true,
            (CategoricalSelectorV1::AnyDeclared, CategoricalSelectorV1::Allowed { .. }) => false,
            (
                CategoricalSelectorV1::Allowed { ids: left },
                CategoricalSelectorV1::Allowed { ids: right },
            ) => left.iter().all(|id| right.binary_search(id).is_ok()),
        }
    }
    fn temperature(left: &TemperatureSelectorV1, right: &TemperatureSelectorV1) -> bool {
        match (left, right) {
            (_, TemperatureSelectorV1::AnyDeclared) => true,
            (TemperatureSelectorV1::AnyDeclared, TemperatureSelectorV1::Bands { .. }) => false,
            (
                TemperatureSelectorV1::Bands { bands: left },
                TemperatureSelectorV1::Bands { bands: right },
            ) => left.iter().all(|left_band| {
                right.iter().any(|right_band| {
                    left_band.lower_kelvin_inclusive >= right_band.lower_kelvin_inclusive
                        && left_band.upper_kelvin_exclusive <= right_band.upper_kelvin_exclusive
                })
            }),
        }
    }
    categorical(&left.analyte, &right.analyte)
        && categorical(&left.matrix, &right.matrix)
        && categorical(&left.sensor_design, &right.sensor_design)
        && categorical(&left.sensor, &right.sensor)
        && categorical(&left.campaign, &right.campaign)
        && temperature(&left.temperature, &right.temperature)
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
