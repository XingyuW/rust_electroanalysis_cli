use thiserror::Error;

/// Typed failures produced while defining, compiling, or evaluating an ISM model.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ModelError {
    #[error("unsupported model schema version {found}; expected {expected}")]
    UnsupportedSchemaVersion { found: u32, expected: u32 },
    #[error("unsupported model-config schema version {found}; expected {expected}")]
    UnsupportedConfigSchemaVersion { found: u32, expected: u32 },
    #[error("{kind} identifier must not be empty")]
    EmptyIdentifier { kind: &'static str },
    #[error("duplicate {kind} identifier '{id}'")]
    DuplicateIdentifier { kind: &'static str, id: String },
    #[error("component '{component}' depends on missing component '{dependency}'")]
    MissingDependency {
        component: String,
        dependency: String,
    },
    #[error("component dependency cycle detected: {components:?}")]
    CircularDependency { components: Vec<String> },
    #[error("component '{component}' requires missing input '{input}'")]
    MissingInput { component: String, input: String },
    #[error("component '{component}' references missing {kind} '{id}'")]
    MissingReference {
        component: String,
        kind: &'static str,
        id: String,
    },
    #[error(
        "component '{component}' input '{input}' has incompatible units: expected '{expected}', found '{found}'"
    )]
    UnitMismatch {
        component: String,
        input: String,
        expected: String,
        found: String,
    },
    #[error("invalid unit '{unit}' for {subject}")]
    InvalidUnit { subject: String, unit: String },
    #[error("component '{component}' has no factory for kind '{kind}'")]
    UnknownComponentKind { component: String, kind: String },
    #[error("component factory returned a descriptor inconsistent with '{component}'")]
    FactoryDescriptorMismatch { component: String },
    #[error("duplicate voltage contribution owner '{owner}'")]
    DuplicateContributionOwner { owner: String },
    #[error("{kind} '{id}' has invalid bounds [{lower}, {upper}]")]
    InvalidBounds {
        kind: &'static str,
        id: String,
        lower: f64,
        upper: f64,
    },
    #[error("{kind} '{id}' value {value} is outside bounds [{lower}, {upper}]")]
    BoundViolation {
        kind: &'static str,
        id: String,
        value: f64,
        lower: f64,
        upper: f64,
    },
    #[error("non-finite value for {subject}")]
    NonFinite { subject: String },
    #[error("state vector length {actual} does not match model state dimension {expected}")]
    StateDimension { expected: usize, actual: usize },
    #[error("parameter vector length {actual} does not match model parameter dimension {expected}")]
    ParameterDimension { expected: usize, actual: usize },
    #[error("invalid transition interval {dt_s} s")]
    InvalidTimeStep { dt_s: f64 },
    #[error("invalid Jacobian dimensions from component '{component}'")]
    JacobianDimension { component: String },
    #[error("component '{component}' emitted a non-finite voltage contribution")]
    NonFiniteContribution { component: String },
    #[error("component '{component}' emitted a voltage without a declared contribution owner")]
    UndeclaredVoltageContribution { component: String },
    #[error("model configuration I/O error: {0}")]
    Io(String),
    #[error("model configuration TOML error: {0}")]
    Toml(String),
    #[error("model artifact JSON error: {0}")]
    Json(String),
}
