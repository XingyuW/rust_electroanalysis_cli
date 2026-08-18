use thiserror::Error;
#[derive(Debug, Error)]
pub enum HealthError {
    #[error("invalid health input: {0}")]
    InvalidInput(String),
    #[error("health artifact I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("health artifact serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid Phase-C health configuration: {message}")]
    InvalidPhaseCConfig { message: String },
    #[error("unsupported Phase-C artifact schema: expected {expected}, got {actual}")]
    UnsupportedPhaseCArtifactSchema { expected: u32, actual: u32 },
    #[error("wrong Phase-C artifact kind: expected {expected}, got {actual}")]
    WrongPhaseCArtifactKind { expected: String, actual: String },
    #[error("Phase-C source scope mismatch: {source_name}")]
    SourceScopeMismatch { source_name: String },
    #[error("Phase-C unit mismatch for {source_name}: expected {expected}, got {actual}")]
    SourceUnitMismatch {
        source_name: String,
        expected: String,
        actual: String,
    },
    #[error("invalid Phase-C evidence at {source_name}.{field}")]
    InvalidEvidence { source_name: String, field: String },
    #[error("invalid lineage catalog: {message}")]
    LineageCatalogInvalid { message: String },
    #[error("conflicting Phase-C evidence input: {left} vs {right}")]
    ConflictingEvidenceInput { left: String, right: String },
    #[error("could not assemble Phase-C report: {message}")]
    ReportAssembly { message: String },
}
impl HealthError {
    pub fn invalid(s: impl Into<String>) -> Self {
        Self::InvalidInput(s.into())
    }
}
