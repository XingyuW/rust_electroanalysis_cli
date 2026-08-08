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
    #[error(
        "invalid {subject} covariance dimensions; expected {expected}x{expected}, found {actual}"
    )]
    CovarianceDimension {
        subject: &'static str,
        expected: usize,
        actual: String,
    },
    #[error("non-finite {subject} covariance entry at ({row}, {column})")]
    NonFiniteCovariance {
        subject: &'static str,
        row: usize,
        column: usize,
    },
    #[error("{subject} covariance is not symmetric at ({row}, {column})")]
    AsymmetricCovariance {
        subject: &'static str,
        row: usize,
        column: usize,
    },
    #[error("{subject} covariance is not positive semidefinite")]
    NonPositiveSemidefiniteCovariance { subject: &'static str },
    #[error(
        "covariance conflicts with declared {declared_uncertainty:?} uncertainty for '{quantity_id}': diagonal {covariance_diagonal:?}; {reason}"
    )]
    CovarianceUncertaintyConflict {
        quantity_id: String,
        declared_uncertainty: super::state::DeclaredUncertaintyClass,
        covariance_diagonal: Option<f64>,
        reason: String,
    },
    #[error("stochastic quantity '{quantity_id}' is missing covariance")]
    MissingCovarianceForStochasticQuantity { quantity_id: String },
    #[error(
        "deterministic quantity '{quantity_id}' has nonzero covariance (diagonal {covariance_diagonal})"
    )]
    NonzeroCovarianceForDeterministicQuantity {
        quantity_id: String,
        covariance_diagonal: f64,
    },
    #[error("invalid Jacobian coverage from component '{component}': {message}")]
    JacobianCoverage { component: String, message: String },
    #[error(
        "model schema version {found} must be explicitly migrated to version {expected} before compilation"
    )]
    LegacyMigrationRequired { found: u32, expected: u32 },
    #[error("component '{component}' emitted a non-finite voltage contribution")]
    NonFiniteContribution { component: String },
    #[error("component '{component}' emitted a voltage without a declared contribution owner")]
    UndeclaredVoltageContribution { component: String },
    #[error("component '{component}' requested unsupported composition semantics '{semantics}'")]
    UnsupportedCompositionSemantics {
        component: String,
        semantics: String,
    },
    #[error(
        "component '{component}' emitted an output incompatible with declared {semantics:?} semantics"
    )]
    IncompatibleContributionOutput {
        component: String,
        semantics: super::component::ContributionSemantics,
    },
    #[error("invalid or missing uncertainty declaration for {subject}")]
    InvalidUncertainty { subject: String },
    #[error("invalid potential reconstruction tolerance {tolerance_v} V")]
    InvalidTolerance { tolerance_v: f64 },
    #[error(
        "component contributions reconstruct {reconstructed_v} V, not predicted {predicted_v} V within {tolerance_v} V"
    )]
    ContributionReconstruction {
        predicted_v: f64,
        reconstructed_v: f64,
        tolerance_v: f64,
    },
    #[error("component '{component}' evaluation failed: {message}")]
    ComponentEvaluation { component: String, message: String },
    #[error("component '{component}' has an invalid descriptor shape: {message}")]
    InvalidComponentShape { component: String, message: String },
    #[error("model configuration I/O error: {0}")]
    Io(String),
    #[error("model configuration TOML error: {0}")]
    Toml(String),
    #[error("model artifact JSON error: {0}")]
    Json(String),
}
