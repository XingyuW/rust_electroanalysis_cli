//! Closed, configuration-neutral wire types for MHI Phase-E validation.
//!
//! The evaluator consumes these types after the protocol reader has rejected
//! unknown fields and semantic defaults.  They are intentionally separate
//! from clap so library callers and the CLI exercise the same contract.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CohortRoleV1 {
    Development,
    Validation,
    Holdout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOriginV1 {
    Physical,
    Synthetic,
    Constructed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedValidationLevelV1 {
    Software,
    Physical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationOutcomeV1 {
    MeetsProtocol,
    DoesNotMeetProtocol,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseClaimOutcomeV1 {
    PhysicallyValidated,
    SoftwareValidatedOnly,
    DoesNotMeetProtocol,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparatorV1 {
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateTargetV1 {
    PointEstimate,
    LowerConfidenceBound,
    UpperConfidenceBound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CategoricalSelectorV1 {
    AnyDeclared,
    Allowed { ids: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemperatureBandV1 {
    pub lower_kelvin_inclusive: f64,
    pub upper_kelvin_exclusive: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TemperatureSelectorV1 {
    AnyDeclared,
    Bands { bands: Vec<TemperatureBandV1> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainSelectorV1 {
    pub analyte: CategoricalSelectorV1,
    pub matrix: CategoricalSelectorV1,
    pub sensor_design: CategoricalSelectorV1,
    pub sensor: CategoricalSelectorV1,
    pub campaign: CategoricalSelectorV1,
    pub temperature: TemperatureSelectorV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainKeyV1 {
    pub analyte_id: String,
    pub matrix_id: String,
    pub sensor_design_id: String,
    pub sensor_id: String,
    pub campaign_id: String,
    pub temperature_kelvin: f64,
}

impl DomainSelectorV1 {
    pub fn contains(&self, key: &DomainKeyV1) -> bool {
        fn categorical(selector: &CategoricalSelectorV1, value: &str) -> bool {
            match selector {
                CategoricalSelectorV1::AnyDeclared => true,
                CategoricalSelectorV1::Allowed { ids } => ids.iter().any(|id| id == value),
            }
        }
        let temperature = match &self.temperature {
            TemperatureSelectorV1::AnyDeclared => true,
            TemperatureSelectorV1::Bands { bands } => bands.iter().any(|band| {
                key.temperature_kelvin >= band.lower_kelvin_inclusive
                    && key.temperature_kelvin < band.upper_kelvin_exclusive
            }),
        };
        categorical(&self.analyte, &key.analyte_id)
            && categorical(&self.matrix, &key.matrix_id)
            && categorical(&self.sensor_design, &key.sensor_design_id)
            && categorical(&self.sensor, &key.sensor_id)
            && categorical(&self.campaign, &key.campaign_id)
            && temperature
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum HealthTargetV1 {
    Dimension { dimension_id: String },
    Aggregate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceMethodV1 {
    pub method_id: String,
    pub method_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferenceUncertaintyRuleV1 {
    RequireQuantified {
        measure_id: String,
        unit: String,
        maximum_inclusive: f64,
    },
    AllowUnavailableWithLimitation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PhysicalApprovalAuthorityV1 {
    NotRequested,
    EmbeddedTrustRoot { trust_root_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferenceAuthorityRuleV1 {
    Mechanism {
        allowed_methods: Vec<ReferenceMethodV1>,
        allowed_authority_ids: Vec<String>,
        blinding_rule: String,
        uncertainty_rule: ReferenceUncertaintyRuleV1,
    },
    Health {
        allowed_methods: Vec<ReferenceMethodV1>,
        allowed_authority_ids: Vec<String>,
        blinding_rule: String,
        uncertainty_rule: ReferenceUncertaintyRuleV1,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AcceptanceRuleV1 {
    Count {
        rule_id: String,
        metric: String,
        comparator: ComparatorV1,
        threshold_u64: u64,
    },
    Rate {
        rule_id: String,
        metric: String,
        target: RateTargetV1,
        comparator: ComparatorV1,
        threshold: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredStratumV1 {
    pub stratum_id: String,
    pub predicates: Vec<serde_json::Value>,
    pub minimum_eligible_records: u64,
    pub minimum_independent_families: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MechanismEndpointV1 {
    pub endpoint_id: String,
    pub hypothesis_id: String,
    pub cohort_role: CohortRoleV1,
    pub domain: DomainSelectorV1,
    pub mechanism_artifact_required: bool,
    pub reference_rule: ReferenceAuthorityRuleV1,
    pub support_levels: Vec<String>,
    pub critical_policy: String,
    pub minimum_eligible_records: u64,
    pub minimum_independent_families: u64,
    pub required_strata: Vec<RequiredStratumV1>,
    pub acceptance_rules: Vec<AcceptanceRuleV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthEndpointV1 {
    pub endpoint_id: String,
    pub target: HealthTargetV1,
    pub cohort_role: CohortRoleV1,
    pub domain: DomainSelectorV1,
    pub health_artifact_required: bool,
    pub reference_rule: ReferenceAuthorityRuleV1,
    pub predicted_positive_statuses: Vec<String>,
    pub predicted_negative_statuses: Vec<String>,
    pub reference_label_universe: Vec<String>,
    pub reference_positive_labels: Vec<String>,
    pub reference_negative_labels: Vec<String>,
    pub minimum_eligible_records: u64,
    pub minimum_independent_families: u64,
    pub required_strata: Vec<RequiredStratumV1>,
    pub acceptance_rules: Vec<AcceptanceRuleV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseClaimV1 {
    pub claim_id: String,
    pub requested_level: RequestedValidationLevelV1,
    pub statement: String,
    pub domain: DomainSelectorV1,
    pub supporting_endpoint_ids: Vec<String>,
}
