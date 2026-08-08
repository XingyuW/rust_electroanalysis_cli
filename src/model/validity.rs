use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Explicit evaluation status. A warning is not a health diagnosis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidityStatus {
    Valid,
    ValidWithWarnings,
    Invalid,
    Unavailable,
}

/// Per-component validity information retained alongside model-level results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentValidityReport {
    pub component_id: String,
    pub status: ValidityStatus,
    pub assumptions_checked: Vec<String>,
    pub validity_domain: String,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
    pub evaluation_rejected: bool,
    #[serde(default)]
    pub physical_valid: bool,
    #[serde(default)]
    pub domain_status: DomainStatus,
    #[serde(default)]
    pub extrapolation_distance: Option<f64>,
    #[serde(default)]
    pub violated_domain_fields: Vec<String>,
    #[serde(default)]
    pub domain_source: DomainSource,
}

/// Closed interval with explicitly inclusive endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericInterval {
    pub lower: f64,
    pub upper: f64,
}

impl NumericInterval {
    pub fn validate(&self) -> bool {
        self.lower.is_finite() && self.upper.is_finite() && self.lower <= self.upper
    }

    pub fn distance(&self, value: f64) -> Option<f64> {
        if !self.validate() || !value.is_finite() {
            return None;
        }
        Some(if value < self.lower {
            self.lower - value
        } else if value > self.upper {
            value - self.upper
        } else {
            0.0
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DomainSource {
    CalibrationArtifact,
    UserConfiguration,
    ValidatedExperiment,
    LiteratureOrUserDeclaration,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DomainEnforcement {
    #[default]
    Warn,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DomainStatus {
    InsideDomain,
    NearBoundary,
    OutsideDomain,
    #[default]
    DomainUnavailable,
}

/// Calibrated applicability limits. Absence means no in-domain claim, not a
/// universal range. Intervals include their endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ComponentApplicabilityDomain {
    pub target_activity: Option<NumericInterval>,
    pub temperature_k: Option<NumericInterval>,
    #[serde(default)]
    pub interferent_activities: BTreeMap<String, NumericInterval>,
    #[serde(default)]
    pub environmental_inputs: BTreeMap<String, NumericInterval>,
    #[serde(default)]
    pub source: DomainSource,
    #[serde(default)]
    pub enforcement: DomainEnforcement,
}

impl ComponentApplicabilityDomain {
    /// Reads an explicit serialized domain declaration from component metadata.
    /// `applicability_domain` contains this structure as JSON so the stable
    /// schema-v1 descriptor stays backward compatible; no default limits are
    /// inferred when the key is absent.
    pub fn from_metadata(metadata: &BTreeMap<String, String>) -> Result<Option<Self>, String> {
        let Some(serialized) = metadata.get("applicability_domain") else {
            return Ok(None);
        };
        let domain: Self = serde_json::from_str(serialized)
            .map_err(|error| format!("applicability_domain must be valid JSON: {error}"))?;
        let intervals = domain
            .target_activity
            .iter()
            .chain(domain.temperature_k.iter())
            .chain(domain.interferent_activities.values())
            .chain(domain.environmental_inputs.values());
        if intervals.clone().next().is_none()
            || intervals.into_iter().any(|interval| !interval.validate())
        {
            return Err(
                "a domain must contain at least one finite inclusive interval with lower <= upper"
                    .into(),
            );
        }
        Ok(Some(domain))
    }
}

/// A declaration of the domain over which a state, parameter, or model is valid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidityDomain {
    pub description: String,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub excluded_conditions: Vec<String>,
}

/// A non-claiming validity result. Violations remain visible to consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityReport {
    pub is_valid: bool,
    pub checked_domain: String,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidityReport {
    pub(crate) fn valid(domain: impl Into<String>) -> Self {
        Self {
            is_valid: true,
            checked_domain: domain.into(),
            violations: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub(crate) fn invalid(domain: impl Into<String>, violations: Vec<String>) -> Self {
        Self {
            is_valid: false,
            checked_domain: domain.into(),
            violations,
            warnings: Vec::new(),
        }
    }
}
