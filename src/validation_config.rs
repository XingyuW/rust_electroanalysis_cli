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

/// The protocol's reference blinding requirement.  This is deliberately an
/// enum rather than free text: a spelling change must not weaken a cohort
/// authority boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlindingRuleV1 {
    RequireBlinded,
    AllowDeclaredUnblinded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlindingStateV1 {
    BlindedToAssessment,
    NotBlinded,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceDependencyCompletenessV1 {
    Complete,
    Unknown,
}

/// The only count authorities which can appear in a Phase-E acceptance rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CountMetricV1 {
    DeclaredCount,
    EligibleCount,
    ExcludedCount,
    NotApplicableCount,
    IndependentFamilyCount,
    SupportCount,
    CriticalContradictionCount,
    NotAssessedOrOtherCount,
    Tp,
    Tn,
    Fp,
    Fn,
    IndeterminateCount,
    DataQualityInsufficientCount,
    EvaluableCount,
}

/// The only rate authorities which can appear in a Phase-E acceptance rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateMetricV1 {
    ExclusionRate,
    SupportFraction,
    ContradictionFraction,
    NotAssessedFraction,
    Coverage,
    IndeterminateRate,
    DataQualityInsufficientRate,
    Sensitivity,
    Specificity,
    FalsePositiveRate,
    FalseNegativeRate,
    BalancedAccuracy,
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
        blinding_rule: BlindingRuleV1,
        uncertainty_rule: ReferenceUncertaintyRuleV1,
    },
    Health {
        allowed_methods: Vec<ReferenceMethodV1>,
        allowed_authority_ids: Vec<String>,
        blinding_rule: BlindingRuleV1,
        uncertainty_rule: ReferenceUncertaintyRuleV1,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AcceptanceRuleV1 {
    Count {
        rule_id: String,
        metric: CountMetricV1,
        comparator: ComparatorV1,
        threshold_u64: u64,
    },
    Rate {
        rule_id: String,
        metric: RateMetricV1,
        target: RateTargetV1,
        comparator: ComparatorV1,
        threshold: f64,
    },
}

/// A required stratum can select exactly the six axes predeclared by the
/// Phase-E protocol.  A `serde_json::Value` here would silently accept an
/// unrecognised scientific axis, so it is intentionally not used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum StratumPredicateV1 {
    AnalyteEquals {
        id: String,
    },
    MatrixEquals {
        id: String,
    },
    SensorDesignEquals {
        id: String,
    },
    SensorEquals {
        id: String,
    },
    CampaignEquals {
        id: String,
    },
    TemperatureBand {
        lower_kelvin_inclusive: f64,
        upper_kelvin_exclusive: f64,
    },
}

impl StratumPredicateV1 {
    pub fn contains(&self, key: &DomainKeyV1) -> bool {
        match self {
            Self::AnalyteEquals { id } => key.analyte_id == *id,
            Self::MatrixEquals { id } => key.matrix_id == *id,
            Self::SensorDesignEquals { id } => key.sensor_design_id == *id,
            Self::SensorEquals { id } => key.sensor_id == *id,
            Self::CampaignEquals { id } => key.campaign_id == *id,
            Self::TemperatureBand {
                lower_kelvin_inclusive,
                upper_kelvin_exclusive,
            } => {
                key.temperature_kelvin >= *lower_kelvin_inclusive
                    && key.temperature_kelvin < *upper_kelvin_exclusive
            }
        }
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::AnalyteEquals { .. } => 0,
            Self::MatrixEquals { .. } => 1,
            Self::SensorDesignEquals { .. } => 2,
            Self::SensorEquals { .. } => 3,
            Self::CampaignEquals { .. } => 4,
            Self::TemperatureBand { .. } => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredStratumV1 {
    pub stratum_id: String,
    pub predicates: Vec<StratumPredicateV1>,
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
