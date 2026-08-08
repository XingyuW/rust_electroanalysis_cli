use super::*;
use crate::domain::{ArtifactError, ArtifactKind, VersionedArtifact, validate_serialized_finite};

macro_rules! contract {
    ($type:ty, $kind:expr, $current:expr, $legacy:expr) => {
        impl VersionedArtifact for $type {
            const ARTIFACT_KIND: ArtifactKind = $kind;
            const CURRENT_SCHEMA_VERSION: u32 = $current;
            const LEGACY_SCHEMA_VERSIONS: &'static [u32] = $legacy;
            fn schema_version(&self) -> u32 {
                self.schema_version
            }
            fn validate_before_json(&self) -> Result<(), crate::domain::ArtifactError> {
                crate::domain::artifact::validate_serialized_finite(self)
            }
        }
    };
}

contract!(EisFitArtifact, ArtifactKind::EisFit, 2, &[1, 2]);
contract!(
    TransientAnalysisReport,
    ArtifactKind::TransientAnalysis,
    1,
    &[1]
);
contract!(
    CalibrationObservationSet,
    ArtifactKind::CalibrationObservations,
    1,
    &[1]
);
contract!(
    StoredCalibrationModel,
    ArtifactKind::CalibrationModel,
    1,
    &[1]
);
contract!(
    CalibrationAnalysisReport,
    ArtifactKind::CalibrationAnalysis,
    1,
    &[1]
);
contract!(SignalAnalysisReport, ArtifactKind::SignalAnalysis, 1, &[1]);
contract!(
    SensorHealthBaseline,
    ArtifactKind::HealthBaseline,
    2,
    &[1, 2]
);
contract!(
    SensorHealthAssessment,
    ArtifactKind::HealthAssessment,
    1,
    &[1]
);
contract!(HealthTrendReport, ArtifactKind::HealthTrend, 1, &[1]);
contract!(
    MechanismAnalysisReport,
    ArtifactKind::MechanismAnalysis,
    1,
    &[1]
);
contract!(
    StateEstimationReport,
    ArtifactKind::StateEstimation,
    2,
    &[1, 2]
);
impl VersionedArtifact for ModelCompilationArtifact {
    const ARTIFACT_KIND: ArtifactKind = ArtifactKind::ModelCompilation;
    const CURRENT_SCHEMA_VERSION: u32 = 4;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32] = &[1, 2, 3, 4];

    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn validate_before_json(&self) -> Result<(), ArtifactError> {
        self.model_definition
            .validate_schema()
            .map_err(|error| ArtifactError::Validation {
                message: error.to_string(),
            })?;
        validate_serialized_finite(self)
    }
}

impl VersionedArtifact for ModelAnalysisReport {
    const ARTIFACT_KIND: ArtifactKind = ArtifactKind::ModelAnalysis;
    const CURRENT_SCHEMA_VERSION: u32 = 4;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32] = &[1, 2, 3, 4];

    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn validate_before_json(&self) -> Result<(), ArtifactError> {
        self.model_definition
            .validate_schema()
            .map_err(|error| ArtifactError::Validation {
                message: error.to_string(),
            })?;
        validate_serialized_finite(self)
    }
}
contract!(ValidationResults, ArtifactKind::ModelValidation, 1, &[1]);
