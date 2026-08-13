use thiserror::Error;
#[derive(Debug, Error)]
pub enum MechanismAssessmentError {
    #[error("missing temporal assessment for {requirement_id}")]
    TemporalAssessmentMissing { requirement_id: String },
    #[error("evidence binding: {0}")]
    Binding(String),
    #[error("assessment: {0}")]
    Invalid(String),
}
