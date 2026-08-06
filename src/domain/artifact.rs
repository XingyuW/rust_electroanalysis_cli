//! Stable, validated JSON boundaries between analysis workflows.

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    EisFit,
    TransientAnalysis,
    CalibrationObservations,
    CalibrationModel,
    CalibrationAnalysis,
    SignalAnalysis,
    HealthBaseline,
    HealthAssessment,
    HealthTrend,
    MechanismAnalysis,
    StateEstimation,
    ModelCompilation,
    ModelAnalysis,
    ModelValidation,
}

impl ArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EisFit => "eis_fit",
            Self::TransientAnalysis => "transient_analysis",
            Self::CalibrationObservations => "calibration_observations",
            Self::CalibrationModel => "calibration_model",
            Self::CalibrationAnalysis => "calibration_analysis",
            Self::SignalAnalysis => "signal_analysis",
            Self::HealthBaseline => "health_baseline",
            Self::HealthAssessment => "health_assessment",
            Self::HealthTrend => "health_trend",
            Self::MechanismAnalysis => "mechanism_analysis",
            Self::StateEstimation => "state_estimation",
            Self::ModelCompilation => "ism_model_compilation",
            Self::ModelAnalysis => "ism_model_analysis",
            Self::ModelValidation => "ism_model_validation",
        }
    }
}

impl fmt::Display for ArtifactKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub trait VersionedArtifact: Serialize + DeserializeOwned {
    const ARTIFACT_KIND: ArtifactKind;
    const CURRENT_SCHEMA_VERSION: u32;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32];
    fn schema_version(&self) -> u32;
}

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("artifact I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("artifact JSON error for {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("artifact {path} must be a JSON object")]
    InvalidRoot { path: PathBuf },
    #[error("artifact {path} has no valid schema_version")]
    InvalidSchemaVersion { path: PathBuf },
    #[error("artifact {path} has unsupported schema_version {actual} for {expected}")]
    UnsupportedSchemaVersion {
        path: PathBuf,
        expected: ArtifactKind,
        actual: u32,
    },
    #[error("artifact {path} has kind {actual:?}; expected {expected}")]
    IncompatibleKind {
        path: PathBuf,
        expected: ArtifactKind,
        actual: Option<String>,
    },
    #[error("artifact {path} contains a non-finite numeric token")]
    NonFiniteValue { path: PathBuf },
}

pub fn read_artifact<T: VersionedArtifact>(path: &Path) -> Result<T, ArtifactError> {
    let text = fs::read_to_string(path).map_err(|source| ArtifactError::Io {
        path: path.into(),
        source,
    })?;
    reject_nonfinite_tokens(path, &text)?;
    let value: Value = serde_json::from_str(&text).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })?;
    validate_value::<T>(path, &value)?;
    serde_json::from_value(value).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })
}

pub fn write_artifact<T: VersionedArtifact>(
    path: &Path,
    artifact: &T,
) -> Result<(), ArtifactError> {
    if artifact.schema_version() != T::CURRENT_SCHEMA_VERSION {
        return Err(ArtifactError::UnsupportedSchemaVersion {
            path: path.into(),
            expected: T::ARTIFACT_KIND,
            actual: artifact.schema_version(),
        });
    }
    let mut value = serde_json::to_value(artifact).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| ArtifactError::InvalidRoot { path: path.into() })?;
    object.insert(
        "artifact_kind".into(),
        Value::String(T::ARTIFACT_KIND.as_str().into()),
    );
    validate_value::<T>(path, &value)?;
    let text = serde_json::to_string_pretty(&value).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })?;
    reject_nonfinite_tokens(path, &text)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ArtifactError::Io {
            path: parent.into(),
            source,
        })?;
    }
    fs::write(path, text).map_err(|source| ArtifactError::Io {
        path: path.into(),
        source,
    })
}

fn validate_value<T: VersionedArtifact>(path: &Path, value: &Value) -> Result<(), ArtifactError> {
    let object = value
        .as_object()
        .ok_or_else(|| ArtifactError::InvalidRoot { path: path.into() })?;
    let schema = schema_version(path, object)?;
    if schema != T::CURRENT_SCHEMA_VERSION && !T::LEGACY_SCHEMA_VERSIONS.contains(&schema) {
        return Err(ArtifactError::UnsupportedSchemaVersion {
            path: path.into(),
            expected: T::ARTIFACT_KIND,
            actual: schema,
        });
    }
    let kind = object.get("artifact_kind").and_then(Value::as_str);
    if let Some(actual) = kind {
        if actual != T::ARTIFACT_KIND.as_str() {
            return Err(ArtifactError::IncompatibleKind {
                path: path.into(),
                expected: T::ARTIFACT_KIND,
                actual: Some(actual.into()),
            });
        }
    } else if !T::LEGACY_SCHEMA_VERSIONS.contains(&schema) {
        return Err(ArtifactError::IncompatibleKind {
            path: path.into(),
            expected: T::ARTIFACT_KIND,
            actual: None,
        });
    }
    Ok(())
}

fn schema_version(path: &Path, object: &Map<String, Value>) -> Result<u32, ArtifactError> {
    object
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| ArtifactError::InvalidSchemaVersion { path: path.into() })
}

fn reject_nonfinite_tokens(path: &Path, text: &str) -> Result<(), ArtifactError> {
    if text.contains("NaN") || text.contains("Infinity") {
        Err(ArtifactError::NonFiniteValue { path: path.into() })
    } else {
        Ok(())
    }
}
