//! Stable, validated JSON boundaries between analysis workflows.

use serde::{Deserialize, Serialize, de::DeserializeOwned, ser};
use serde_json::{Map, Value};
use std::{
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
