use super::*;
use crate::domain::{
    ArtifactError, ArtifactKind, CurrentArtifactKindPolicy, VersionedArtifact,
    validate_serialized_finite,
};

macro_rules! contract {
    ($type:ty, $kind:expr, $current:expr, $legacy:expr, $policy:expr) => {
        impl VersionedArtifact for $type {
            const ARTIFACT_KIND: ArtifactKind = $kind;
            const CURRENT_SCHEMA_VERSION: u32 = $current;
            const LEGACY_SCHEMA_VERSIONS: &'static [u32] = $legacy;
            const CURRENT_ARTIFACT_KIND_POLICY: CurrentArtifactKindPolicy = $policy;
            fn schema_version(&self) -> u32 {
                self.schema_version
            }
            fn validate_before_json(&self) -> Result<(), crate::domain::ArtifactError> {
                crate::domain::artifact::validate_serialized_finite(self)
            }
            fn require_kind_for_previous_schema_static() -> bool {
                true
            }
        }
    };
}

contract!(
    EisFitArtifact,
    ArtifactKind::EisFit,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::PreserveLegacyOptional
);
contract!(
    TransientAnalysisReport,
    ArtifactKind::TransientAnalysis,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::Required
);
contract!(
    CalibrationObservationSet,
    ArtifactKind::CalibrationObservations,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::Required
);
contract!(
    StoredCalibrationModel,
    ArtifactKind::CalibrationModel,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::Required
);
contract!(
    CalibrationAnalysisReport,
    ArtifactKind::CalibrationAnalysis,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::Required
);
contract!(
    SignalAnalysisReport,
    ArtifactKind::SignalAnalysis,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::Required
);
contract!(
    SensorHealthBaseline,
    ArtifactKind::HealthBaseline,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::PreserveLegacyOptional
);
contract!(
    SensorHealthAssessment,
    ArtifactKind::HealthAssessment,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::Required
);
contract!(
    HealthTrendReport,
    ArtifactKind::HealthTrend,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::Required
);
contract!(
    MechanismAnalysisReport,
    ArtifactKind::MechanismAnalysis,
    3,
    &[1, 2],
    CurrentArtifactKindPolicy::Required
);
contract!(
    StateEstimationReport,
    ArtifactKind::StateEstimation,
    4,
    &[1, 2, 3],
    CurrentArtifactKindPolicy::PreserveLegacyOptional
);
impl VersionedArtifact for ModelCompilationArtifact {
    const ARTIFACT_KIND: ArtifactKind = ArtifactKind::ModelCompilation;
    const CURRENT_SCHEMA_VERSION: u32 = 5;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32] = &[1, 2, 3, 4];
    const CURRENT_ARTIFACT_KIND_POLICY: CurrentArtifactKindPolicy =
        CurrentArtifactKindPolicy::PreserveLegacyOptional;

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
    const CURRENT_SCHEMA_VERSION: u32 = 5;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32] = &[1, 2, 3, 4];
    const CURRENT_ARTIFACT_KIND_POLICY: CurrentArtifactKindPolicy =
        CurrentArtifactKindPolicy::PreserveLegacyOptional;

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
contract!(
    ValidationResults,
    ArtifactKind::ModelValidation,
    1,
    &[1],
    CurrentArtifactKindPolicy::PreserveLegacyOptional
);
