use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;

/// Declarative structural information consumed by a later identifiability
/// adapter. It deliberately makes no empirical claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiabilityMetadata {
    pub states_requiring_independent_observations: Vec<String>,
    pub parameter_requirements: Vec<ParameterIdentifiabilityRequirement>,
    pub component_sensitivity_targets: Vec<String>,
    #[serde(default)]
    pub component_requirements: Vec<IdentifiabilityRequirement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnownIdentifiabilityRequirementKind {
    ActivityExcitation,
    TransientExcitation,
    ObservationDurationRelativeToTimescale,
    ModeSeparation,
    ReferenceAnchor,
    IndependentCovariateVariation,
    InterferentVariation,
    TemperatureVariation,
    RepeatedStandards,
    AuxiliaryObservation,
}

/// Open identifiability requirement kind.  The associated constants preserve
/// the source-level API used by pre-A1 model definitions while serialization
/// now accepts and preserves future custom requirement strings.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifiabilityRequirementKind {
    Known(KnownIdentifiabilityRequirementKind),
    Custom(String),
}

impl fmt::Debug for IdentifiabilityRequirementKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(kind) => formatter.write_str(match kind {
                KnownIdentifiabilityRequirementKind::ActivityExcitation => "ActivityExcitation",
                KnownIdentifiabilityRequirementKind::TransientExcitation => "TransientExcitation",
                KnownIdentifiabilityRequirementKind::ObservationDurationRelativeToTimescale => {
                    "ObservationDurationRelativeToTimescale"
                }
                KnownIdentifiabilityRequirementKind::ModeSeparation => "ModeSeparation",
                KnownIdentifiabilityRequirementKind::ReferenceAnchor => "ReferenceAnchor",
                KnownIdentifiabilityRequirementKind::IndependentCovariateVariation => {
                    "IndependentCovariateVariation"
                }
                KnownIdentifiabilityRequirementKind::InterferentVariation => "InterferentVariation",
                KnownIdentifiabilityRequirementKind::TemperatureVariation => "TemperatureVariation",
                KnownIdentifiabilityRequirementKind::RepeatedStandards => "RepeatedStandards",
                KnownIdentifiabilityRequirementKind::AuxiliaryObservation => "AuxiliaryObservation",
            }),
            Self::Custom(value) => formatter.debug_tuple("Custom").field(value).finish(),
        }
    }
}

#[allow(non_upper_case_globals)]
impl IdentifiabilityRequirementKind {
    pub const ActivityExcitation: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::ActivityExcitation);
    pub const TransientExcitation: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::TransientExcitation);
    pub const ObservationDurationRelativeToTimescale: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::ObservationDurationRelativeToTimescale);
    pub const ModeSeparation: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::ModeSeparation);
    pub const ReferenceAnchor: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::ReferenceAnchor);
    pub const IndependentCovariateVariation: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::IndependentCovariateVariation);
    pub const InterferentVariation: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::InterferentVariation);
    pub const TemperatureVariation: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::TemperatureVariation);
    pub const RepeatedStandards: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::RepeatedStandards);
    pub const AuxiliaryObservation: Self =
        Self::Known(KnownIdentifiabilityRequirementKind::AuxiliaryObservation);

    pub fn as_str(&self) -> &str {
        match self {
            Self::Known(kind) => kind.as_str(),
            Self::Custom(value) => value,
        }
    }
}

impl KnownIdentifiabilityRequirementKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ActivityExcitation => "activity_excitation",
            Self::TransientExcitation => "transient_excitation",
            Self::ObservationDurationRelativeToTimescale => {
                "observation_duration_relative_to_timescale"
            }
            Self::ModeSeparation => "mode_separation",
            Self::ReferenceAnchor => "reference_anchor",
            Self::IndependentCovariateVariation => "independent_covariate_variation",
            Self::InterferentVariation => "interferent_variation",
            Self::TemperatureVariation => "temperature_variation",
            Self::RepeatedStandards => "repeated_standards",
            Self::AuxiliaryObservation => "auxiliary_observation",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "activity_excitation" => Self::ActivityExcitation,
            "transient_excitation" => Self::TransientExcitation,
            "observation_duration_relative_to_timescale" => {
                Self::ObservationDurationRelativeToTimescale
            }
            "mode_separation" => Self::ModeSeparation,
            "reference_anchor" => Self::ReferenceAnchor,
            "independent_covariate_variation" => Self::IndependentCovariateVariation,
            "interferent_variation" => Self::InterferentVariation,
            "temperature_variation" => Self::TemperatureVariation,
            "repeated_standards" => Self::RepeatedStandards,
            "auxiliary_observation" => Self::AuxiliaryObservation,
            _ => return None,
        })
    }
}

impl Serialize for IdentifiabilityRequirementKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for IdentifiabilityRequirementKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        if value.is_empty() {
            return Err(de::Error::custom(
                "identifiability requirement kind cannot be empty",
            ));
        }
        Ok(KnownIdentifiabilityRequirementKind::from_str(&value)
            .map(Self::Known)
            .unwrap_or(Self::Custom(value)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementSeverity {
    Required,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiabilityRequirement {
    pub requirement_id: String,
    #[serde(default)]
    pub scope: IdentifiabilityScope,
    pub component_id: String,
    #[serde(default)]
    pub component_ids: Vec<String>,
    pub kind: IdentifiabilityRequirementKind,
    #[serde(default)]
    pub target_states: Vec<String>,
    #[serde(default)]
    pub target_parameters: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub quantitative_criterion: Option<String>,
    pub severity: RequirementSeverity,
}

/// Whether the requirement is emitted by the compiled graph or by an optional
/// capability advertised by a model profile.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IdentifiabilityScope {
    #[default]
    Active,
    Conditional {
        component_kind: String,
        activation_condition: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterIdentifiabilityRequirement {
    pub parameter_id: String,
    pub requirements: Vec<String>,
}

/// Explicit assessment state that preserves a lack of evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatus {
    NotAssessed,
    Supported,
    Contradicted,
    Indeterminate,
}

/// Placeholder report interface for structural/practical identifiability.
/// Phase 02 intentionally does not infer identifiability from a fit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiabilityReport {
    pub structural: AssessmentStatus,
    pub practical: AssessmentStatus,
    pub parameter_ids: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub warnings: Vec<String>,
}

impl IdentifiabilityReport {
    pub(crate) fn not_assessed(parameter_ids: Vec<String>) -> Self {
        Self {
            structural: AssessmentStatus::NotAssessed,
            practical: AssessmentStatus::NotAssessed,
            parameter_ids,
            contradictory_evidence: Vec::new(),
            missing_evidence: vec![
                "No structural or practical identifiability analysis is implemented in Phase 02."
                    .into(),
            ],
            warnings: Vec::new(),
        }
    }
}
