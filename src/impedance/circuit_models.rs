#![allow(clippy::collapsible_if)]

//! Circuit-model resolver for EIS fitting workflows.
//!
//! Resolution order:
//! 1) explicit model tag in filename/metadata,
//! 2) first matching configured rule,
//! 3) configured fallback model.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

use crate::domain::ConfigurationError;

/// Fallback circuit expression used when no explicit/model-rule match exists.
pub const DEFAULT_EIS_CIRCUIT_MODEL: &str = "R0-p(CPE1,R1)";
/// Default config filename loaded from the workspace root.
pub const DEFAULT_CIRCUIT_MODEL_CONFIG_PATH: &str = "config/parsing.toml";
pub const LEGACY_CIRCUIT_MODEL_CONFIG_PATH: &str = "circuit_models.toml";

/// Metadata keys scanned for explicit model declarations.
const CIRCUIT_MODEL_METADATA_KEYS: &[&str] =
    &["circuitmodel", "equivalentcircuit", "circuit", "model"];

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FitRankingMetric {
    /// Rank candidates by Akaike information criterion.
    #[default]
    Aic, // ranking_metric = "aic"
    /// Rank candidates by weighted RMSE.
    WeightedRmse, // ranking_metric = "weighted_rmse"
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ModelSelectionConfig {
    /// Metric used to rank multiple feasible circuit fits.
    #[serde(default)]
    pub ranking_metric: FitRankingMetric,
    /// Tie-break threshold for Warburg-inclusive model preference.
    #[serde(default = "default_warburg_aic_threshold")]
    pub warburg_aic_threshold: f64,
}

impl Default for ModelSelectionConfig {
    fn default() -> Self {
        Self {
            ranking_metric: FitRankingMetric::Aic,
            warburg_aic_threshold: default_warburg_aic_threshold(),
        }
    }
}

fn default_warburg_aic_threshold() -> f64 {
    4.0
}

// Normalizes metadata keys for case/whitespace-insensitive lookups.
fn normalize_header_name(name: &str) -> String {
    name.trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect()
}

// Allow characters commonly used in circuit-string syntax.
fn is_valid_circuit_model_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '(' | ')' | ',' | '-' | '.')
}

// Extract inline tags such as "circuit=..." or "model=..." from free text.
fn extract_tagged_circuit_model(text: &str) -> Option<String> {
    let lowered = text.to_ascii_lowercase();

    for marker in ["circuit=", "model="] {
        if let Some(start) = lowered.find(marker) {
            let candidate = text[start + marker.len()..]
                .chars()
                .take_while(|&ch| is_valid_circuit_model_char(ch))
                .collect::<String>();

            if !candidate.is_empty() {
                return Some(candidate);
            }
        }
    }

    None
}

/// One rule mapping filename/metadata predicates to a circuit model string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitModelRule {
    /// Model expression returned when this rule matches.
    pub circuit_model: String,
    /// Case-insensitive substrings that must all appear in filename.
    pub filename_contains: Vec<String>,
    /// Normalized metadata key/value contains predicates.
    pub metadata_contains: Vec<(String, String)>,
}

impl CircuitModelRule {
    /// Create a rule targeting `circuit_model`.
    pub fn new(circuit_model: impl Into<String>) -> Self {
        Self {
            circuit_model: circuit_model.into(),
            filename_contains: Vec::new(),
            metadata_contains: Vec::new(),
        }
    }

    /// Add a filename substring predicate (case-insensitive).
    pub fn with_filename_contains(mut self, pattern: impl Into<String>) -> Self {
        self.filename_contains
            .push(pattern.into().to_ascii_lowercase());
        self
    }

    /// Add a metadata key/value contains predicate (case-insensitive).
    pub fn with_metadata_contains(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata_contains.push((
            normalize_header_name(&key.into()),
            value.into().to_ascii_lowercase(),
        ));
        self
    }

    // Evaluate all configured predicates against a context.
    fn matches(&self, context: &CircuitModelContext) -> bool {
        let filename_lower = context.filename_lower();
        let filename_matches = self.filename_contains.is_empty()
            || self
                .filename_contains
                .iter()
                .all(|pattern| filename_lower.contains(pattern));

        let metadata_matches = self.metadata_contains.is_empty()
            || self.metadata_contains.iter().all(|(key, expected)| {
                context
                    .metadata
                    .get(key)
                    .map(|actual| actual.to_ascii_lowercase().contains(expected))
                    .unwrap_or(false)
            });

        filename_matches && metadata_matches
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct CircuitModelResolverConfig {
    #[serde(default)]
    fallback_model: Option<String>,
    #[serde(default)]
    model_selection: ModelSelectionConfig,
    #[serde(default)]
    rules: Vec<CircuitModelRuleConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct CircuitModelRuleConfig {
    circuit_model: String,
    #[serde(default)]
    filename_contains: Vec<String>,
    #[serde(default)]
    metadata_contains: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CircuitModelResolver {
    /// Fallback model used when no explicit/rule match is found.
    pub fallback_model: String,
    /// Secondary model-selection policy used downstream in ranking.
    pub model_selection: ModelSelectionConfig,
    /// Ordered rules evaluated first-match-wins.
    pub rules: Vec<CircuitModelRule>,
}

impl CircuitModelResolver {
    /// Create a resolver with explicit fallback model and no rules.
    pub fn new(fallback_model: impl Into<String>) -> Self {
        Self {
            fallback_model: fallback_model.into(),
            model_selection: ModelSelectionConfig::default(),
            rules: Vec::new(),
        }
    }

    /// Append a rule to evaluation order.
    pub fn with_rule(mut self, rule: CircuitModelRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Load resolver from a TOML config file.
    pub fn from_config_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigurationError> {
        let path = path.as_ref();
        let config_text =
            std::fs::read_to_string(path).map_err(|error| ConfigurationError::io(path, error))?;
        let config: CircuitModelResolverConfig =
            toml::from_str(&config_text).map_err(|error| ConfigurationError::parse(path, error))?;

        Ok(Self::from_config(config))
    }

    /// Load resolver from the default workspace config path.
    pub fn from_default_config_file() -> Result<Self, ConfigurationError> {
        Self::from_config_file(DEFAULT_CIRCUIT_MODEL_CONFIG_PATH)
    }

    /// Load resolver from default path if present; otherwise use defaults.
    pub fn load_or_default() -> Result<Self, ConfigurationError> {
        let default_path = Path::new(DEFAULT_CIRCUIT_MODEL_CONFIG_PATH);
        if default_path.exists() {
            Self::from_default_config_file()
        } else {
            let legacy_path = Path::new(LEGACY_CIRCUIT_MODEL_CONFIG_PATH);
            if legacy_path.exists() {
                Self::from_config_file(legacy_path)
            } else {
                Ok(Self::default())
            }
        }
    }

    /// Load resolver from default path, then fallback to legacy root path.
    pub fn from_default_or_legacy_config_file() -> Result<Self, ConfigurationError> {
        let default_path = Path::new(DEFAULT_CIRCUIT_MODEL_CONFIG_PATH);
        if default_path.exists() {
            Self::from_config_file(default_path)
        } else {
            Self::from_config_file(LEGACY_CIRCUIT_MODEL_CONFIG_PATH)
        }
    }

    // Convert deserialized raw config into normalized runtime resolver.
    fn from_config(config: CircuitModelResolverConfig) -> Self {
        let mut resolver = Self::new(
            config
                .fallback_model
                .unwrap_or_else(|| DEFAULT_EIS_CIRCUIT_MODEL.to_string()),
        );
        resolver.model_selection = config.model_selection;

        for rule in config.rules {
            let mut normalized_rule = CircuitModelRule::new(rule.circuit_model);

            for pattern in rule.filename_contains {
                normalized_rule = normalized_rule.with_filename_contains(pattern);
            }

            for (key, value) in rule.metadata_contains {
                normalized_rule = normalized_rule.with_metadata_contains(key, value);
            }

            resolver = resolver.with_rule(normalized_rule);
        }

        resolver
    }

    /// Resolve the effective circuit model for one dataset context.
    pub fn resolve(&self, context: &CircuitModelContext) -> String {
        if let Some(explicit_model) = context.explicit_circuit_model() {
            return explicit_model;
        }

        if let Some(rule_match) = self.rules.iter().find(|rule| rule.matches(context)) {
            return rule_match.circuit_model.clone();
        }

        self.fallback_model.clone()
    }
}

impl Default for CircuitModelResolver {
    fn default() -> Self {
        Self::new(DEFAULT_EIS_CIRCUIT_MODEL)
    }
}

#[derive(Debug, Clone)]
pub struct CircuitModelContext {
    /// Source filename (or stem) used for filename predicates.
    pub filename: String,
    /// Parsed metadata map used for metadata predicates.
    pub metadata: BTreeMap<String, String>,
}

impl CircuitModelContext {
    // Lowercased filename cache helper.
    fn filename_lower(&self) -> String {
        self.filename.to_ascii_lowercase()
    }

    // Prefer explicit model tags in filename/metadata over configured rules.
    fn explicit_circuit_model(&self) -> Option<String> {
        if let Some(model) = extract_tagged_circuit_model(&self.filename) {
            return Some(model);
        }

        for key in CIRCUIT_MODEL_METADATA_KEYS {
            if let Some(value) = self.metadata.get(*key) {
                if !value.is_empty() {
                    return Some(value.clone());
                }
            }
        }

        None
    }
}
