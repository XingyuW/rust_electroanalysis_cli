use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentValidityReport {
    pub component_id: String,
    pub status: ValidityStatus,
    pub assumptions_checked: Vec<String>,
    pub validity_domain: String,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
    pub evaluation_rejected: bool,
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
