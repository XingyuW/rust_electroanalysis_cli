//! Stable, validated JSON boundaries between analysis workflows.

use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, DeserializeSeed, Visitor},
    ser,
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fmt, fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    #[serde(rename = "ism_model_compilation")]
    ModelCompilation,
    #[serde(rename = "ism_model_analysis")]
    ModelAnalysis,
    #[serde(rename = "ism_model_validation")]
    ModelValidation,
    #[serde(rename = "mhi_validation_dataset")]
    MhiValidationDataset,
    #[serde(rename = "mhi_validation_report")]
    MhiValidationReport,
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
            Self::MhiValidationDataset => "mhi_validation_dataset",
            Self::MhiValidationReport => "mhi_validation_report",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentArtifactKindPolicy {
    Required,
    PreserveLegacyOptional,
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
    const CURRENT_ARTIFACT_KIND_POLICY: CurrentArtifactKindPolicy;
    fn schema_version(&self) -> u32;
    /// Must validate the complete typed artifact before JSON can erase a
    /// non-finite float. There is intentionally no accepting default.
    fn validate_before_json(&self) -> Result<(), ArtifactError>;
    /// Validation that requires the typed representation after a public read.
    fn validate_after_read(&self) -> Result<(), ArtifactError> {
        Ok(())
    }
    /// A1's migration boundary can always preserve an explicit lineage state
    /// even for historical result structs that predate the typed field.
    fn lineage_state(&self) -> crate::domain::ArtifactLineageState {
        crate::domain::current_unknown_lineage(self.schema_version())
    }
    fn require_kind_for_previous_schema_static() -> bool {
        false
    }
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
    #[error("artifact {path} contains a non-finite value at {field_path}")]
    NonFiniteValue { path: PathBuf, field_path: String },
    #[error("artifact schema validation failed: {message}")]
    Validation { message: String },
    #[error("artifact {path} must be a regular non-symlink file")]
    UnsafeFile { path: PathBuf },
    #[error("artifact {path} contains a duplicate JSON key {key}")]
    DuplicateJsonKey { path: PathBuf, key: String },
    #[error("artifact {path} is not valid UTF-8")]
    InvalidUtf8 { path: PathBuf },
    #[error("artifact {path} contains a UTF-8 byte-order mark")]
    Utf8Bom { path: PathBuf },
}

/// A strict read deliberately keeps both the parsed value and the exact bytes
/// that were validated.  Validation workflows must never reread a pathname
/// after checking its checksum.
#[derive(Debug, Clone)]
pub struct StrictArtifactRead<T> {
    pub artifact: T,
    pub source_bytes: Vec<u8>,
    pub source_file_sha256: String,
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
    let artifact: T = serde_json::from_value(value).map_err(|source| {
        if T::ARTIFACT_KIND == ArtifactKind::HealthAssessment {
            ArtifactError::Validation {
                message: format!("invalid health assessment payload: {source}"),
            }
        } else {
            ArtifactError::Json {
                path: path.into(),
                source,
            }
        }
    })?;
    artifact.validate_after_read()?;
    Ok(artifact)
}

/// Reads an artifact at the Phase-E boundary.  Unlike the historic public
/// reader this rejects duplicate object keys at every nesting level and
/// returns the exact checked bytes and their file hash.  `read_artifact`
/// intentionally remains unchanged for stored-artifact compatibility.
pub fn read_artifact_strict<T: VersionedArtifact>(
    path: &Path,
) -> Result<StrictArtifactRead<T>, ArtifactError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| ArtifactError::Io {
        path: path.into(),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(ArtifactError::UnsafeFile { path: path.into() });
    }
    let source_bytes = fs::read(path).map_err(|source| ArtifactError::Io {
        path: path.into(),
        source,
    })?;
    read_artifact_strict_bytes(path, &source_bytes)
}

pub(crate) fn read_artifact_strict_bytes<T: VersionedArtifact>(
    path: &Path,
    source_bytes: &[u8],
) -> Result<StrictArtifactRead<T>, ArtifactError> {
    if source_bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(ArtifactError::Utf8Bom { path: path.into() });
    }
    let text = std::str::from_utf8(source_bytes)
        .map_err(|_| ArtifactError::InvalidUtf8 { path: path.into() })?;
    reject_nonfinite_tokens(path, text)?;
    scan_duplicate_json_keys(text).map_err(|error| match error {
        DuplicateScanError::Json(source) => ArtifactError::Json {
            path: path.into(),
            source,
        },
        DuplicateScanError::Duplicate(key) => ArtifactError::DuplicateJsonKey {
            path: path.into(),
            key,
        },
    })?;
    let value: Value = serde_json::from_str(text).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })?;
    validate_value::<T>(path, &value)?;
    let artifact: T = serde_json::from_value(value).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })?;
    artifact.validate_after_read()?;
    let source_file_sha256 = hex_sha256(source_bytes);
    Ok(StrictArtifactRead {
        artifact,
        source_bytes: source_bytes.to_vec(),
        source_file_sha256,
    })
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

#[derive(Debug)]
enum DuplicateScanError {
    Json(serde_json::Error),
    Duplicate(String),
}

struct DuplicateScanSeed;

impl<'de> DeserializeSeed<'de> for DuplicateScanSeed {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DuplicateScanVisitor)
    }
}

struct DuplicateScanVisitor;

impl<'de> Visitor<'de> for DuplicateScanVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, _: bool) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }
    fn visit_i64<E>(self, _: i64) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }
    fn visit_u64<E>(self, _: u64) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }
    fn visit_f64<E>(self, _: f64) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }
    fn visit_str<E>(self, _: &str) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }
    fn visit_string<E>(self, _: String) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }
    fn visit_none<E>(self) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }
    fn visit_unit<E>(self) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<(), A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        while sequence.next_element_seed(DuplicateScanSeed)?.is_some() {}
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<(), A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(serde::de::Error::custom(format!(
                    "__duplicate_json_key__{key}"
                )));
            }
            map.next_value_seed(DuplicateScanSeed)?;
        }
        Ok(())
    }
}

fn scan_duplicate_json_keys(text: &str) -> Result<(), DuplicateScanError> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let result = DuplicateScanSeed.deserialize(&mut deserializer);
    match result {
        Ok(()) => deserializer.end().map_err(DuplicateScanError::Json),
        Err(error) => {
            let message = error.to_string();
            let marker = message.split(" at line ").next().unwrap_or(&message);
            if let Some(key) = marker.strip_prefix("__duplicate_json_key__") {
                Err(DuplicateScanError::Duplicate(key.to_string()))
            } else {
                Err(DuplicateScanError::Json(error))
            }
        }
    }
}

/// Shared by the strict lineage-catalog reader.  It deliberately exposes no
/// `serde_json::Value`, because callers must perform their own closed grammar
/// validation after duplicate detection.
pub(crate) fn ensure_duplicate_free_json(text: &str) -> Result<(), String> {
    scan_duplicate_json_keys(text).map_err(|error| match error {
        DuplicateScanError::Json(error) => error.to_string(),
        DuplicateScanError::Duplicate(key) => format!("duplicate JSON key {key}"),
    })
}

pub fn write_artifact<T: VersionedArtifact>(
    path: &Path,
    artifact: &T,
) -> Result<(), ArtifactError> {
    let bytes = serialize_artifact(path, artifact)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ArtifactError::Io {
            path: parent.into(),
            source,
        })?;
    }
    fs::write(path, bytes).map_err(|source| ArtifactError::Io {
        path: path.into(),
        source,
    })
}

pub(crate) fn serialize_artifact<T: VersionedArtifact>(
    path: &Path,
    artifact: &T,
) -> Result<Vec<u8>, ArtifactError> {
    if artifact.schema_version() != T::CURRENT_SCHEMA_VERSION
        && !T::LEGACY_SCHEMA_VERSIONS.contains(&artifact.schema_version())
    {
        return Err(ArtifactError::UnsupportedSchemaVersion {
            path: path.into(),
            expected: T::ARTIFACT_KIND,
            actual: artifact.schema_version(),
        });
    }
    artifact
        .validate_before_json()
        .map_err(|error| match error {
            ArtifactError::NonFiniteValue { field_path, .. } => ArtifactError::NonFiniteValue {
                path: path.into(),
                field_path,
            },
            other => other,
        })?;
    let mut value = serde_json::to_value(artifact).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| ArtifactError::InvalidRoot { path: path.into() })?;
    // A supported legacy value is migrated at the public writer boundary.
    // Its scientific payload remains untouched; A1's additive lineage field
    // is inserted below and the artifact contract version is advanced here.
    object.insert(
        "schema_version".into(),
        Value::Number(serde_json::Number::from(T::CURRENT_SCHEMA_VERSION)),
    );
    object.insert(
        "artifact_kind".into(),
        Value::String(T::ARTIFACT_KIND.as_str().into()),
    );
    object.entry("lineage").or_insert_with(|| {
        serde_json::to_value(artifact.lineage_state()).unwrap_or_else(|_| {
            serde_json::json!({
                "LegacyUnknown": {
                    "source_schema_version": artifact.schema_version(),
                    "reason": "MigrationInformationUnavailable"
                }
            })
        })
    });
    // A typed A1 artifact carries lineage directly. If it was read from a
    // historical payload with the field absent, retain that explicit state
    // while recording the schema observed at this writer boundary. This never
    // upgrades a legacy artifact into a fabricated identity.
    if let Some(lineage) = object.get_mut("lineage") {
        let parsed = serde_json::from_value::<crate::domain::ArtifactLineageState>(lineage.clone())
            .map_err(|source| ArtifactError::Json {
                path: path.into(),
                source,
            })?;
        if let crate::domain::ArtifactLineageState::LegacyUnknown {
            source_schema_version: None,
            reason,
        } = parsed
        {
            *lineage = serde_json::to_value(crate::domain::ArtifactLineageState::LegacyUnknown {
                source_schema_version: Some(artifact.schema_version()),
                reason,
            })
            .map_err(|source| ArtifactError::Json {
                path: path.into(),
                source,
            })?;
        }
    }
    validate_value::<T>(path, &value)?;
    let text = serde_json::to_string_pretty(&value).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })?;
    reject_nonfinite_tokens(path, &text)?;
    Ok(text.into_bytes())
}

/// Writes the one frozen legacy health-assessment representation.  This is
/// deliberately not a generic old-schema escape hatch: only the no-Phase-C
/// health runner calls it, and it can only ever emit schema 3.
pub(crate) fn write_legacy_sensor_health_assessment_v3(
    path: &Path,
    assessment: &crate::results::SensorHealthAssessment,
) -> Result<(), ArtifactError> {
    if assessment.schema_version != 3 {
        return Err(ArtifactError::UnsupportedSchemaVersion {
            path: path.to_path_buf(),
            expected: ArtifactKind::HealthAssessment,
            actual: assessment.schema_version,
        });
    }
    if assessment.phase_c.is_some() {
        return Err(ArtifactError::Validation {
            message: "schema-3 health assessment must not contain phase_c".into(),
        });
    }
    validate_serialized_finite(assessment).map_err(|error| match error {
        ArtifactError::NonFiniteValue { field_path, .. } => ArtifactError::NonFiniteValue {
            path: path.into(),
            field_path,
        },
        other => other,
    })?;
    let mut value = serde_json::to_value(assessment).map_err(|source| ArtifactError::Json {
        path: path.into(),
        source,
    })?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| ArtifactError::InvalidRoot { path: path.into() })?;
    object.remove("phase_c");
    object.insert(
        "schema_version".into(),
        Value::Number(serde_json::Number::from(3)),
    );
    object.insert(
        "artifact_kind".into(),
        Value::String(ArtifactKind::HealthAssessment.as_str().into()),
    );
    // The legacy typed assessment deliberately does not carry a generic
    // `artifact_kind` field. Add its schema-3 wire discriminator before the
    // common artifact validator checks the public boundary.
    validate_value::<crate::results::SensorHealthAssessment>(path, &value)?;
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
    // `hypotheses` is the schema-3 report field.  Schema 4 uses the distinct
    // `legacy_hypotheses` name; accepting the retired spelling there would
    // make a malformed current artifact indistinguishable from a migration.
    if T::ARTIFACT_KIND == ArtifactKind::MechanismAnalysis
        && schema == T::CURRENT_SCHEMA_VERSION
        && object.contains_key("hypotheses")
    {
        return Err(ArtifactError::Validation {
            message: "schema-4 mechanism report must use legacy_hypotheses, not hypotheses".into(),
        });
    }
    let kind = object.get("artifact_kind").and_then(Value::as_str);
    if schema == T::CURRENT_SCHEMA_VERSION {
        if let Some(actual) = kind {
            if actual != T::ARTIFACT_KIND.as_str() {
                return Err(ArtifactError::IncompatibleKind {
                    path: path.into(),
                    expected: T::ARTIFACT_KIND,
                    actual: Some(actual.into()),
                });
            }
        } else if matches!(
            T::CURRENT_ARTIFACT_KIND_POLICY,
            CurrentArtifactKindPolicy::Required
        ) {
            return Err(ArtifactError::IncompatibleKind {
                path: path.into(),
                expected: T::ARTIFACT_KIND,
                actual: None,
            });
        }
    } else {
        if let Some(actual) = kind {
            if actual != T::ARTIFACT_KIND.as_str() {
                return Err(ArtifactError::IncompatibleKind {
                    path: path.into(),
                    expected: T::ARTIFACT_KIND,
                    actual: Some(actual.into()),
                });
            }
        } else if schema + 1 == T::CURRENT_SCHEMA_VERSION
            && matches!(
                T::CURRENT_ARTIFACT_KIND_POLICY,
                CurrentArtifactKindPolicy::Required
            )
            && T::require_kind_for_previous_schema_static()
        {
            // A1 keeps the A0 schema-2 kind requirement intact while adding a
            // new current schema. Older historical versions remain readable.
            return Err(ArtifactError::IncompatibleKind {
                path: path.into(),
                expected: T::ARTIFACT_KIND,
                actual: None,
            });
        }
    }
    if T::ARTIFACT_KIND == ArtifactKind::HealthAssessment
        && schema == T::CURRENT_SCHEMA_VERSION
        && object.get("phase_c").is_none_or(Value::is_null)
    {
        return Err(ArtifactError::Validation {
            message: "schema-4 health assessment requires a non-null phase_c".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_c_legacy_schema3_writer_rejects_non_schema3_input() {
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/phase_c/writer_boundary/legacy_health_assessment_v3.json");
        let mut assessment: crate::results::SensorHealthAssessment =
            read_artifact(&fixture).expect("legacy fixture reads");
        assessment.schema_version = 4;
        let output = std::env::temp_dir().join(format!(
            "phase_c_legacy_writer_rejects_schema4_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        match write_legacy_sensor_health_assessment_v3(&output, &assessment) {
            Err(ArtifactError::UnsupportedSchemaVersion {
                path,
                expected: ArtifactKind::HealthAssessment,
                actual: 4,
            }) => assert_eq!(path, output),
            other => panic!("expected schema-3 writer rejection, got {other:?}"),
        }
        assert!(!output.exists());
    }
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
        Err(ArtifactError::NonFiniteValue {
            path: path.into(),
            field_path: "$json".into(),
        })
    } else {
        Ok(())
    }
}

/// A lossless validation serializer: serde calls `serialize_f64` before any
/// JSON serializer has an opportunity to convert NaN or infinity to null.
pub fn validate_serialized_finite<T: Serialize>(value: &T) -> Result<(), ArtifactError> {
    value
        .serialize(FiniteSerializer { path: "$".into() })
        .map_err(
            |error: FiniteSerializeError| ArtifactError::NonFiniteValue {
                path: PathBuf::new(),
                field_path: error.path,
            },
        )
}

#[derive(Debug)]
struct FiniteSerializeError {
    path: String,
}
impl std::fmt::Display for FiniteSerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path)
    }
}
impl std::error::Error for FiniteSerializeError {}
impl ser::Error for FiniteSerializeError {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        Self {
            path: "$serialization".into(),
        }
    }
}

struct FiniteSerializer {
    path: String,
}
impl FiniteSerializer {
    fn child(&self, segment: impl AsRef<str>) -> Self {
        Self {
            path: format!("{}{}", self.path, segment.as_ref()),
        }
    }
}

macro_rules! finite_scalar {
    ($($name:ident($ty:ty)),* $(,)?) => {$(
        fn $name(self, _value: $ty) -> Result<Self::Ok, Self::Error> { Ok(()) }
    )*};
}
impl ser::Serializer for FiniteSerializer {
    type Ok = ();
    type Error = FiniteSerializeError;
    type SerializeSeq = FiniteSeq;
    type SerializeTuple = FiniteSeq;
    type SerializeTupleStruct = FiniteSeq;
    type SerializeTupleVariant = FiniteSeq;
    type SerializeMap = FiniteMap;
    type SerializeStruct = FiniteStruct;
    type SerializeStructVariant = FiniteStruct;
    finite_scalar!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8])
    );
    fn serialize_f32(self, value: f32) -> Result<(), FiniteSerializeError> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(FiniteSerializeError { path: self.path })
        }
    }
    fn serialize_f64(self, value: f64) -> Result<(), FiniteSerializeError> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(FiniteSerializeError { path: self.path })
        }
    }
    fn serialize_none(self) -> Result<(), FiniteSerializeError> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), FiniteSerializeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), FiniteSerializeError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), FiniteSerializeError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
    ) -> Result<(), FiniteSerializeError> {
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), FiniteSerializeError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), FiniteSerializeError> {
        value.serialize(self.child(format!(".{variant}")))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<FiniteSeq, FiniteSerializeError> {
        Ok(FiniteSeq {
            path: self.path,
            index: 0,
        })
    }
    fn serialize_tuple(self, _len: usize) -> Result<FiniteSeq, FiniteSerializeError> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<FiniteSeq, FiniteSerializeError> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<FiniteSeq, FiniteSerializeError> {
        Ok(FiniteSeq {
            path: format!("{}.{}", self.path, variant),
            index: 0,
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<FiniteMap, FiniteSerializeError> {
        Ok(FiniteMap {
            path: self.path,
            key: None,
        })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<FiniteStruct, FiniteSerializeError> {
        Ok(FiniteStruct { path: self.path })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<FiniteStruct, FiniteSerializeError> {
        Ok(FiniteStruct {
            path: format!("{}.{}", self.path, variant),
        })
    }
}

struct FiniteSeq {
    path: String,
    index: usize,
}
impl FiniteSeq {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), FiniteSerializeError> {
        let path = format!("{}[{}]", self.path, self.index);
        self.index += 1;
        value.serialize(FiniteSerializer { path })
    }
}
impl ser::SerializeSeq for FiniteSeq {
    type Ok = ();
    type Error = FiniteSerializeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl ser::SerializeTuple for FiniteSeq {
    type Ok = ();
    type Error = FiniteSerializeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl ser::SerializeTupleStruct for FiniteSeq {
    type Ok = ();
    type Error = FiniteSerializeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl ser::SerializeTupleVariant for FiniteSeq {
    type Ok = ();
    type Error = FiniteSerializeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct FiniteStruct {
    path: String,
}
impl ser::SerializeStruct for FiniteStruct {
    type Ok = ();
    type Error = FiniteSerializeError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteSerializer {
            path: format!("{}.{}", self.path, key),
        })
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl ser::SerializeStructVariant for FiniteStruct {
    type Ok = ();
    type Error = FiniteSerializeError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteSerializer {
            path: format!("{}.{}", self.path, key),
        })
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct FiniteMap {
    path: String,
    key: Option<String>,
}
impl ser::SerializeMap for FiniteMap {
    type Ok = ();
    type Error = FiniteSerializeError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self.key.take().unwrap_or_else(|| "?".into());
        value.serialize(FiniteSerializer {
            path: format!("{}[{key:?}]", self.path),
        })
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
struct MapKeySerializer;
macro_rules! key_scalar { ($($name:ident($ty:ty)),* $(,)?) => {$(fn $name(self, value: $ty) -> Result<String, FiniteSerializeError> { Ok(value.to_string()) })*}; }
impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = FiniteSerializeError;
    type SerializeSeq = ser::Impossible<String, FiniteSerializeError>;
    type SerializeTuple = ser::Impossible<String, FiniteSerializeError>;
    type SerializeTupleStruct = ser::Impossible<String, FiniteSerializeError>;
    type SerializeTupleVariant = ser::Impossible<String, FiniteSerializeError>;
    type SerializeMap = ser::Impossible<String, FiniteSerializeError>;
    type SerializeStruct = ser::Impossible<String, FiniteSerializeError>;
    type SerializeStructVariant = ser::Impossible<String, FiniteSerializeError>;
    key_scalar!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char)
    );
    fn serialize_str(self, value: &str) -> Result<String, FiniteSerializeError> {
        Ok(value.into())
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<String, FiniteSerializeError> {
        Err(ser::Error::custom("unsupported map key"))
    }
    fn serialize_none(self) -> Result<String, FiniteSerializeError> {
        Ok("null".into())
    }
    fn serialize_some<T: ?Sized + Serialize>(
        self,
        value: &T,
    ) -> Result<String, FiniteSerializeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<String, FiniteSerializeError> {
        Ok("unit".into())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, FiniteSerializeError> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<String, FiniteSerializeError> {
        Ok(variant.into())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<String, FiniteSerializeError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<String, FiniteSerializeError> {
        Ok(variant.into())
    }
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeSeq, FiniteSerializeError> {
        Err(ser::Error::custom("unsupported map key"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, FiniteSerializeError> {
        Err(ser::Error::custom("unsupported map key"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, FiniteSerializeError> {
        Err(ser::Error::custom("unsupported map key"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, FiniteSerializeError> {
        Err(ser::Error::custom("unsupported map key"))
    }
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeMap, FiniteSerializeError> {
        Err(ser::Error::custom("unsupported map key"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, FiniteSerializeError> {
        Err(ser::Error::custom("unsupported map key"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, FiniteSerializeError> {
        Err(ser::Error::custom("unsupported map key"))
    }
}
