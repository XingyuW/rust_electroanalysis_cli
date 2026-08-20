//! Durable artifact lineage and experiment-scope contracts.
//!
//! This module is deliberately independent of the mechanism and health
//! assessors.  It records what an artifact is derived from and preserves
//! uncertainty when historical provenance is unavailable.

use super::artifact::ArtifactKind;
use serde::{Deserialize, Serialize, de};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ArtifactId(pub String);

impl ArtifactId {
    pub fn new(value: impl Into<String>) -> Result<Self, LineageError> {
        let value = value.into();
        if is_sha256_id(&value) {
            Ok(Self(value))
        } else {
            Err(LineageError::InvalidArtifactId(value))
        }
    }

    pub fn from_semantic_bytes(bytes: &[u8]) -> Self {
        Self(format!("sha256:{}", hex_sha256(bytes)))
    }
}

impl<'de> Deserialize<'de> for ArtifactId {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ExperimentId(pub String);

impl ExperimentId {
    pub fn new(value: impl Into<String>) -> Result<Self, LineageError> {
        let value = value.into();
        if value.is_empty() {
            return Err(LineageError::EmptyIdentifier("experiment_id"));
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for ExperimentId {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct AggregateExperimentScopeId(pub String);

impl AggregateExperimentScopeId {
    pub fn derive(
        aggregation_kind: &str,
        member_experiment_ids: &[ExperimentId],
    ) -> Result<Self, LineageError> {
        if aggregation_kind.is_empty() {
            return Err(LineageError::EmptyIdentifier("aggregation_kind"));
        }
        let members = unique_sorted_experiment_ids(member_experiment_ids)?;
        if members.len() < 2 {
            return Err(LineageError::AggregateNeedsTwoMembers);
        }
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"aggregate-experiment-scope-v1");
        bytes.push(0);
        bytes.extend_from_slice(aggregation_kind.as_bytes());
        bytes.push(0);
        for (index, member) in members.iter().enumerate() {
            if index != 0 {
                bytes.push(0);
            }
            bytes.extend_from_slice(member.0.as_bytes());
        }
        Ok(Self(ArtifactId::from_semantic_bytes(&bytes).0))
    }
}

impl<'de> Deserialize<'de> for AggregateExperimentScopeId {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        if is_sha256_id(&value) {
            Ok(Self(value))
        } else {
            Err(de::Error::custom("invalid aggregate scope ID"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct AcquisitionFamilyId(pub String);

impl AcquisitionFamilyId {
    pub fn new(value: impl Into<String>) -> Result<Self, LineageError> {
        let value = value.into();
        let canonical = value.trim().to_string();
        if canonical.is_empty() {
            return Err(LineageError::EmptyIdentifier("acquisition_family_id"));
        }
        Ok(Self(canonical))
    }
}

impl<'de> Deserialize<'de> for AcquisitionFamilyId {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactAcquisitionFamilies {
    Known(Vec<AcquisitionFamilyId>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolvedAcquisitionFamilies {
    Known(Vec<AcquisitionFamilyId>),
    Unknown,
}

impl ArtifactAcquisitionFamilies {
    pub fn known(
        values: impl IntoIterator<Item = AcquisitionFamilyId>,
    ) -> Result<Self, LineageError> {
        let values = normalize_families(values)?;
        if values.is_empty() {
            return Err(LineageError::EmptyKnownFamilySet);
        }
        Ok(Self::Known(values))
    }

    pub fn union(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Known(left), Self::Known(right)) => {
                let mut values = left.clone();
                values.extend(right.clone());
                Self::Known(normalize_families(values).unwrap_or_default())
            }
            _ => Self::Unknown,
        }
    }

    pub fn validate(&self) -> Result<(), LineageError> {
        match self {
            Self::Known(values) if values.is_empty() => Err(LineageError::EmptyKnownFamilySet),
            Self::Known(values) => validate_normalized_families(values),
            Self::Unknown => Ok(()),
        }
    }
}

impl ResolvedAcquisitionFamilies {
    pub fn known(
        values: impl IntoIterator<Item = AcquisitionFamilyId>,
    ) -> Result<Self, LineageError> {
        let values = normalize_families(values)?;
        if values.is_empty() {
            return Err(LineageError::EmptyKnownFamilySet);
        }
        Ok(Self::Known(values))
    }

    pub fn union(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Known(left), Self::Known(right)) => {
                let mut values = left.clone();
                values.extend(right.clone());
                Self::Known(normalize_families(values).unwrap_or_default())
            }
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeKey {
    Specific(String),
    All,
    Unspecified,
}

impl ScopeKey {
    pub fn specific(value: impl Into<String>) -> Result<Self, LineageError> {
        let value = value.into();
        if value.is_empty() {
            return Err(LineageError::EmptyIdentifier("scope"));
        }
        Ok(Self::Specific(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactExperimentScope {
    Single {
        experiment_id: ExperimentId,
    },
    Aggregate {
        aggregate_scope_id: AggregateExperimentScopeId,
        member_experiment_ids: Vec<ExperimentId>,
    },
    Unknown,
}

impl ArtifactExperimentScope {
    pub fn single(experiment_id: ExperimentId) -> Result<Self, LineageError> {
        ExperimentId::new(experiment_id.0.clone())?;
        Ok(Self::Single { experiment_id })
    }

    pub fn aggregate(
        aggregation_kind: &str,
        member_experiment_ids: impl IntoIterator<Item = ExperimentId>,
    ) -> Result<Self, LineageError> {
        let members = unique_sorted_experiment_ids(
            member_experiment_ids
                .into_iter()
                .collect::<Vec<_>>()
                .as_slice(),
        )?;
        let aggregate_scope_id = AggregateExperimentScopeId::derive(aggregation_kind, &members)?;
        Ok(Self::Aggregate {
            aggregate_scope_id,
            member_experiment_ids: members,
        })
    }

    pub fn validate(&self) -> Result<(), LineageError> {
        match self {
            Self::Single { experiment_id } => {
                ExperimentId::new(experiment_id.0.clone()).map(|_| ())
            }
            Self::Aggregate {
                aggregate_scope_id,
                member_experiment_ids,
            } => {
                let members = unique_sorted_experiment_ids(member_experiment_ids)?;
                if members.len() < 2 || members.as_slice() != member_experiment_ids.as_slice() {
                    return Err(LineageError::NonCanonicalAggregateMembers);
                }
                let expected = AggregateExperimentScopeId::derive("validation", &members)?;
                if aggregate_scope_id.0.is_empty() || !is_sha256_id(&aggregate_scope_id.0) {
                    return Err(LineageError::InvalidAggregateScopeId(
                        aggregate_scope_id.0.clone(),
                    ));
                }
                // The aggregation kind is intentionally producer-owned and is
                // not recoverable from the serialized digest.  Structural
                // validation therefore checks shape here; constructors verify
                // the exact kind before serialization.
                let _ = expected;
                Ok(())
            }
            Self::Unknown => Ok(()),
        }
    }

    /// Propagate input scopes into this producer's persisted scope.  The
    /// producer-owned aggregation kind is mandatory: a generic propagated
    /// scope is not a durable artifact identity.
    pub fn propagate_with_kind(
        aggregation_kind: &str,
        scopes: impl IntoIterator<Item = ArtifactExperimentScope>,
    ) -> Self {
        let mut singles = BTreeSet::new();
        let mut has_aggregate = false;
        for scope in scopes {
            match scope {
                Self::Single { experiment_id } => {
                    singles.insert(experiment_id);
                }
                Self::Aggregate {
                    member_experiment_ids,
                    ..
                } => {
                    has_aggregate = true;
                    singles.extend(member_experiment_ids);
                }
                Self::Unknown => return Self::Unknown,
            }
        }
        let members = singles.into_iter().collect::<Vec<_>>();
        match members.len() {
            0 => Self::Unknown,
            1 if !has_aggregate => Self::Single {
                experiment_id: members[0].clone(),
            },
            _ => Self::aggregate(aggregation_kind, members).unwrap_or(Self::Unknown),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactDependencyRole {
    Initialization,
    Calibration,
    Prior,
    Constraint,
    TransformationInput,
    AuxiliaryInput,
    ValidationInput,
    DerivedFrom,
}

impl ArtifactDependencyRole {
    pub fn discriminant(&self) -> u8 {
        match self {
            Self::Initialization => 0,
            Self::Calibration => 1,
            Self::Prior => 2,
            Self::Constraint => 3,
            Self::TransformationInput => 4,
            Self::AuxiliaryInput => 5,
            Self::ValidationInput => 6,
            Self::DerivedFrom => 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDependency {
    pub artifact_id: ArtifactId,
    pub artifact_kind: ArtifactKind,
    pub role: ArtifactDependencyRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactIdentity {
    pub artifact_id: ArtifactId,
    pub artifact_kind: ArtifactKind,
    pub schema_version: u32,
    pub producer_version: String,
    pub experiment_scope: ArtifactExperimentScope,
    pub sensor_scope: ScopeKey,
    pub channel_scope: ScopeKey,
    pub acquisition_families: ArtifactAcquisitionFamilies,
    pub semantic_sha256: String,
}

impl ArtifactIdentity {
    pub fn validate(&self) -> Result<(), LineageError> {
        if !is_sha256_id(&self.artifact_id.0)
            || !is_sha256_hex(&self.semantic_sha256)
            || self.artifact_id.0 != format!("sha256:{}", self.semantic_sha256)
        {
            return Err(LineageError::InvalidArtifactIdentity);
        }
        if self.producer_version.is_empty() || self.schema_version == 0 {
            return Err(LineageError::InvalidArtifactIdentity);
        }
        self.experiment_scope.validate()?;
        self.acquisition_families.validate()
    }
}

/// Creates an artifact identity from an owned semantic payload.  The hash
/// view includes only stable scientific identity and dependency descriptors;
/// callers do not pass paths, timestamps, prose, artifact IDs, or hashes.
#[allow(clippy::too_many_arguments)]
pub fn artifact_identity_from_payload<T: Serialize>(
    artifact_kind: ArtifactKind,
    schema_version: u32,
    producer_version: impl Into<String>,
    experiment_scope: ArtifactExperimentScope,
    sensor_scope: ScopeKey,
    channel_scope: ScopeKey,
    acquisition_families: ArtifactAcquisitionFamilies,
    dependencies: &[ArtifactDependency],
    scientific_payload: &T,
) -> Result<ArtifactIdentity, LineageError> {
    // `serde_json::Value` maps non-finite floats to null; validate the source
    // payload with the RFC 8785 serializer before constructing that hash view.
    serde_jcs::to_vec(scientific_payload)
        .map_err(|error| LineageError::Serialization(error.to_string()))?;
    let mut dependency_view = dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.role.discriminant(),
                dependency.artifact_kind.as_str(),
                dependency.artifact_id.0.clone(),
            )
        })
        .collect::<Vec<_>>();
    dependency_view.sort();
    let view = serde_json::json!({
        "artifact_kind": artifact_kind,
        "schema_version": schema_version,
        "producer_version": producer_version.into(),
        "experiment_scope": experiment_scope,
        "sensor_scope": sensor_scope,
        "channel_scope": channel_scope,
        "acquisition_families": acquisition_families,
        "dependencies": dependency_view,
        "scientific_payload": scientific_payload,
    });
    let semantic_sha256 = semantic_sha256(&view)?;
    let identity = ArtifactIdentity {
        artifact_id: ArtifactId(format!("sha256:{semantic_sha256}")),
        artifact_kind,
        schema_version,
        producer_version: view["producer_version"].as_str().unwrap_or_default().into(),
        experiment_scope: serde_json::from_value(view["experiment_scope"].clone())
            .map_err(|error| LineageError::Serialization(error.to_string()))?,
        sensor_scope: serde_json::from_value(view["sensor_scope"].clone())
            .map_err(|error| LineageError::Serialization(error.to_string()))?,
        channel_scope: serde_json::from_value(view["channel_scope"].clone())
            .map_err(|error| LineageError::Serialization(error.to_string()))?,
        acquisition_families: serde_json::from_value(view["acquisition_families"].clone())
            .map_err(|error| LineageError::Serialization(error.to_string()))?,
        semantic_sha256,
    };
    identity.validate()?;
    Ok(identity)
}

/// Builds a current `Known` lineage state from a producer-owned artifact
/// payload. The stable view deliberately drops output location, generation
/// time, warnings, and the lineage field itself before hashing.
#[allow(clippy::too_many_arguments)]
pub fn known_lineage_from_artifact<T: Serialize>(
    artifact_kind: ArtifactKind,
    schema_version: u32,
    producer_version: impl Into<String>,
    experiment_scope: ArtifactExperimentScope,
    sensor_scope: ScopeKey,
    channel_scope: ScopeKey,
    acquisition_families: ArtifactAcquisitionFamilies,
    direct_dependencies: impl IntoIterator<Item = ArtifactDependency>,
    artifact: &T,
) -> Result<ArtifactLineageState, LineageError> {
    serde_jcs::to_vec(artifact).map_err(|error| LineageError::Serialization(error.to_string()))?;
    let mut scientific_payload = serde_json::to_value(artifact)
        .map_err(|error| LineageError::Serialization(error.to_string()))?;
    if let Value::Object(object) = &mut scientific_payload {
        object.remove("lineage");
        object.remove("schema_version");
        object.remove("artifact_kind");
        object.remove("warnings");
        if let Some(Value::Object(provenance)) = object.get_mut("provenance") {
            provenance.remove("input_path");
            provenance.remove("configuration_path");
            provenance.remove("generation_timestamp");
        }
    }
    let mut direct_dependencies = direct_dependencies.into_iter().collect::<Vec<_>>();
    direct_dependencies.sort_by(|left, right| {
        left.role
            .discriminant()
            .cmp(&right.role.discriminant())
            .then_with(|| {
                left.artifact_kind
                    .as_str()
                    .cmp(right.artifact_kind.as_str())
            })
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
    });
    direct_dependencies.dedup();
    let identity = artifact_identity_from_payload(
        artifact_kind,
        schema_version,
        producer_version,
        experiment_scope,
        sensor_scope,
        channel_scope,
        acquisition_families,
        &direct_dependencies,
        &scientific_payload,
    )?;
    Ok(ArtifactLineageState::Known {
        identity,
        direct_dependencies,
    })
}

/// Preserves a known upstream artifact as one sorted direct dependency. A
/// legacy upstream artifact has no fabricable ID and therefore contributes no
/// dependency descriptor.
pub fn dependency_from_lineage(
    lineage: &ArtifactLineageState,
    role: ArtifactDependencyRole,
) -> Option<ArtifactDependency> {
    match lineage {
        ArtifactLineageState::Known { identity, .. } => Some(ArtifactDependency {
            artifact_id: identity.artifact_id.clone(),
            artifact_kind: identity.artifact_kind,
            role,
        }),
        ArtifactLineageState::LegacyUnknown { .. } => None,
    }
}

/// Propagates scope and family identity only from explicit serialized lineage.
/// Any legacy/unknown source remains Unknown rather than being guessed from a
/// file path or result identifier.
pub fn lineage_scope_and_families<'a>(
    aggregation_kind: &str,
    lineages: impl IntoIterator<Item = &'a ArtifactLineageState>,
) -> (ArtifactExperimentScope, ArtifactAcquisitionFamilies) {
    let mut scopes = Vec::new();
    let mut families = Vec::new();
    for lineage in lineages {
        let ArtifactLineageState::Known { identity, .. } = lineage else {
            return (
                ArtifactExperimentScope::Unknown,
                ArtifactAcquisitionFamilies::Unknown,
            );
        };
        scopes.push(identity.experiment_scope.clone());
        let ArtifactAcquisitionFamilies::Known(values) = &identity.acquisition_families else {
            return (
                ArtifactExperimentScope::propagate_with_kind(aggregation_kind, scopes),
                ArtifactAcquisitionFamilies::Unknown,
            );
        };
        families.extend(values.clone());
    }
    let scope = ArtifactExperimentScope::propagate_with_kind(aggregation_kind, scopes);
    let families = ArtifactAcquisitionFamilies::known(families)
        .unwrap_or(ArtifactAcquisitionFamilies::Unknown);
    (scope, families)
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ArtifactLineageState {
    Known {
        identity: ArtifactIdentity,
        direct_dependencies: Vec<ArtifactDependency>,
    },
    LegacyUnknown {
        source_schema_version: Option<u32>,
        reason: UnknownLineageReason,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize)]
enum ArtifactLineageStateWire {
    Known {
        identity: ArtifactIdentity,
        direct_dependencies: Vec<ArtifactDependency>,
    },
    LegacyUnknown {
        source_schema_version: Option<u32>,
        reason: UnknownLineageReason,
    },
}

impl<'de> Deserialize<'de> for ArtifactLineageState {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match ArtifactLineageStateWire::deserialize(deserializer)? {
            ArtifactLineageStateWire::Known {
                identity,
                direct_dependencies,
            } => {
                identity.validate().map_err(de::Error::custom)?;
                validate_dependencies(&direct_dependencies).map_err(de::Error::custom)?;
                Ok(Self::Known {
                    identity,
                    direct_dependencies,
                })
            }
            ArtifactLineageStateWire::LegacyUnknown {
                source_schema_version,
                reason,
            } => Ok(Self::LegacyUnknown {
                source_schema_version,
                reason,
            }),
        }
    }
}

impl Default for ArtifactLineageState {
    fn default() -> Self {
        Self::LegacyUnknown {
            source_schema_version: None,
            reason: UnknownLineageReason::FieldAbsentInLegacyArtifact,
        }
    }
}

pub fn legacy_unknown_lineage() -> ArtifactLineageState {
    ArtifactLineageState::default()
}

/// Conservative lineage state for a current producer that cannot establish a
/// complete authoritative identity. This is intentionally not an identity
/// synthesis path: callers retain an explicit, schema-versioned unknown.
pub fn current_unknown_lineage(schema_version: u32) -> ArtifactLineageState {
    ArtifactLineageState::LegacyUnknown {
        source_schema_version: Some(schema_version),
        reason: UnknownLineageReason::MigrationInformationUnavailable,
    }
}

pub fn artifact_scope_from_experiment_ids(
    aggregation_kind: &str,
    experiment_ids: impl IntoIterator<Item = ExperimentId>,
) -> ArtifactExperimentScope {
    let ids = experiment_ids.into_iter().collect::<Vec<_>>();
    let unique = unique_sorted_experiment_ids(&ids).unwrap_or_default();
    match unique.as_slice() {
        [] => ArtifactExperimentScope::Unknown,
        [experiment_id] => ArtifactExperimentScope::Single {
            experiment_id: experiment_id.clone(),
        },
        _ => ArtifactExperimentScope::aggregate(aggregation_kind, unique)
            .unwrap_or(ArtifactExperimentScope::Unknown),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownLineageReason {
    FieldAbsentInLegacyArtifact,
    ExternalArtifactWithoutLineage,
    MigrationInformationUnavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactLineageNode {
    pub identity: ArtifactIdentity,
    pub direct_dependencies: Vec<ArtifactDependency>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactLineageCatalog {
    pub schema_version: u32,
    pub artifacts: BTreeMap<ArtifactId, ArtifactLineageNode>,
}

impl Default for ArtifactLineageCatalog {
    fn default() -> Self {
        Self {
            schema_version: 1,
            artifacts: BTreeMap::new(),
        }
    }
}

impl ArtifactLineageCatalog {
    pub fn insert(&mut self, node: ArtifactLineageNode) -> Result<(), LineageError> {
        node.identity.validate()?;
        if node.identity.artifact_id.0.is_empty() {
            return Err(LineageError::InvalidArtifactIdentity);
        }
        validate_dependencies(&node.direct_dependencies)?;
        self.artifacts
            .insert(node.identity.artifact_id.clone(), node);
        Ok(())
    }
}

/// Errors from the canonical JSON reader for an artifact-lineage catalog.
///
/// A catalog deliberately is not a [`crate::domain::VersionedArtifact`]: it
/// has no `ArtifactKind` and it is provenance metadata rather than a
/// scientific artifact.  Keeping this vocabulary separate makes malformed
/// JSON distinguishable from a syntactically-valid closed-schema violation.
#[derive(Debug, Error)]
pub enum LineageCatalogReadError {
    #[error("artifact lineage catalog I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("artifact lineage catalog JSON error for {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("artifact lineage catalog {path} must be a JSON object")]
    InvalidRoot { path: PathBuf },
    #[error("artifact lineage catalog {path} contains unknown field {field}")]
    UnknownField { path: PathBuf, field: String },
    #[error("artifact lineage catalog {path} repeats field {field}")]
    DuplicateField { path: PathBuf, field: String },
    #[error("artifact lineage catalog {path} repeats artifact key {key}")]
    DuplicateArtifactKey { path: PathBuf, key: String },
    #[error("artifact lineage catalog {path} has unsupported schema version {actual}")]
    UnsupportedSchemaVersion { path: PathBuf, actual: u32 },
    #[error("artifact lineage catalog {path} key {key} does not match identity {identity}")]
    KeyIdentityMismatch {
        path: PathBuf,
        key: String,
        identity: String,
    },
    #[error("artifact lineage catalog validation failed for {path}: {source}")]
    Validation {
        path: PathBuf,
        #[source]
        source: LineageError,
    },
}

const CATALOG_INVALID_ROOT: &str = "__phase_d_catalog_invalid_root";
const CATALOG_UNKNOWN_FIELD: &str = "__phase_d_catalog_unknown_field:";
const CATALOG_DUPLICATE_FIELD: &str = "__phase_d_catalog_duplicate_field:";
const CATALOG_DUPLICATE_ARTIFACT_KEY: &str = "__phase_d_catalog_duplicate_artifact_key:";

/// Reads the one canonical, closed-schema artifact-lineage catalog.
///
/// The custom map visitors intentionally see duplicate JSON keys before a
/// `BTreeMap` can overwrite one.  Consumers such as Phase D must use this
/// reader rather than deserialize catalog content locally.
pub fn read_artifact_lineage_catalog(
    path: &Path,
) -> Result<ArtifactLineageCatalog, LineageCatalogReadError> {
    let text = fs::read_to_string(path).map_err(|source| LineageCatalogReadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut deserializer = serde_json::Deserializer::from_str(&text);
    let wire = CatalogWire::deserialize(&mut deserializer)
        .map_err(|source| map_catalog_deserialize_error(path, source))?;
    deserializer
        .end()
        .map_err(|source| LineageCatalogReadError::Json {
            path: path.to_path_buf(),
            source,
        })?;

    if wire.schema_version != 1 {
        return Err(LineageCatalogReadError::UnsupportedSchemaVersion {
            path: path.to_path_buf(),
            actual: wire.schema_version,
        });
    }

    let mut catalog = ArtifactLineageCatalog::default();
    let mut keys = BTreeSet::new();
    for (key, node) in wire.artifacts {
        if !keys.insert(key.clone()) {
            return Err(LineageCatalogReadError::DuplicateArtifactKey {
                path: path.to_path_buf(),
                key: key.0,
            });
        }
        if key != node.identity.artifact_id {
            return Err(LineageCatalogReadError::KeyIdentityMismatch {
                path: path.to_path_buf(),
                key: key.0,
                identity: node.identity.artifact_id.0,
            });
        }
        catalog
            .insert(node)
            .map_err(|source| LineageCatalogReadError::Validation {
                path: path.to_path_buf(),
                source,
            })?;
    }
    Ok(catalog)
}

fn map_catalog_deserialize_error(
    path: &Path,
    source: serde_json::Error,
) -> LineageCatalogReadError {
    // `serde_json` appends source coordinates to visitor errors.  Strip that
    // transport detail before matching the closed reader vocabulary while
    // retaining the original `serde_json::Error` for genuine JSON failures.
    let message = source.to_string();
    let marker = message.split(" at line ").next().unwrap_or(&message);
    let location = path.to_path_buf();
    if marker == CATALOG_INVALID_ROOT {
        LineageCatalogReadError::InvalidRoot { path: location }
    } else if let Some(field) = marker.strip_prefix(CATALOG_UNKNOWN_FIELD) {
        LineageCatalogReadError::UnknownField {
            path: location,
            field: field.to_string(),
        }
    } else if let Some(field) = marker.strip_prefix(CATALOG_DUPLICATE_FIELD) {
        LineageCatalogReadError::DuplicateField {
            path: location,
            field: field.to_string(),
        }
    } else if let Some(key) = marker.strip_prefix(CATALOG_DUPLICATE_ARTIFACT_KEY) {
        LineageCatalogReadError::DuplicateArtifactKey {
            path: location,
            key: key.to_string(),
        }
    } else {
        LineageCatalogReadError::Json {
            path: location,
            source,
        }
    }
}

struct CatalogWire {
    schema_version: u32,
    artifacts: Vec<(ArtifactId, ArtifactLineageNode)>,
}

impl<'de> Deserialize<'de> for CatalogWire {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(CatalogWireVisitor)
    }
}

struct CatalogWireVisitor;

impl<'de> de::Visitor<'de> for CatalogWireVisitor {
    type Value = CatalogWire;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a closed artifact-lineage catalog object")
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut schema_version = None;
        let mut artifacts = None;
        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "schema_version" => {
                    if schema_version.is_some() {
                        return Err(de::Error::custom(format!(
                            "{CATALOG_DUPLICATE_FIELD}schema_version"
                        )));
                    }
                    schema_version = Some(map.next_value()?);
                }
                "artifacts" => {
                    if artifacts.is_some() {
                        return Err(de::Error::custom(format!(
                            "{CATALOG_DUPLICATE_FIELD}artifacts"
                        )));
                    }
                    artifacts = Some(map.next_value_seed(ArtifactEntriesSeed)?);
                }
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                    return Err(de::Error::custom(format!("{CATALOG_UNKNOWN_FIELD}{field}")));
                }
            }
        }
        Ok(CatalogWire {
            schema_version: schema_version
                .ok_or_else(|| de::Error::missing_field("schema_version"))?,
            artifacts: artifacts.ok_or_else(|| de::Error::missing_field("artifacts"))?,
        })
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
        Err(E::custom(CATALOG_INVALID_ROOT))
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
        Err(E::custom(CATALOG_INVALID_ROOT))
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
        Err(E::custom(CATALOG_INVALID_ROOT))
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
        Err(E::custom(CATALOG_INVALID_ROOT))
    }

    fn visit_str<E: de::Error>(self, _: &str) -> Result<Self::Value, E> {
        Err(E::custom(CATALOG_INVALID_ROOT))
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Err(E::custom(CATALOG_INVALID_ROOT))
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, _: A) -> Result<Self::Value, A::Error> {
        Err(de::Error::custom(CATALOG_INVALID_ROOT))
    }
}

struct ArtifactEntriesSeed;

impl<'de> de::DeserializeSeed<'de> for ArtifactEntriesSeed {
    type Value = Vec<(ArtifactId, ArtifactLineageNode)>;

    fn deserialize<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(ArtifactEntriesVisitor)
    }
}

struct ArtifactEntriesVisitor;

impl<'de> de::Visitor<'de> for ArtifactEntriesVisitor {
    type Value = Vec<(ArtifactId, ArtifactLineageNode)>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an artifact-lineage catalog artifacts object")
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries = Vec::new();
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<ArtifactId>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!(
                    "{CATALOG_DUPLICATE_ARTIFACT_KEY}{}",
                    key.0
                )));
            }
            entries.push((key, map.next_value()?));
        }
        Ok(entries)
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
        Err(E::custom(CATALOG_INVALID_ROOT))
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, _: A) -> Result<Self::Value, A::Error> {
        Err(de::Error::custom(CATALOG_INVALID_ROOT))
    }
}

fn validate_dependencies(dependencies: &[ArtifactDependency]) -> Result<(), LineageError> {
    for dependency in dependencies {
        ArtifactId::new(dependency.artifact_id.0.clone())?;
    }
    let mut expected = dependencies.to_vec();
    expected.sort_by(|left, right| {
        left.role
            .discriminant()
            .cmp(&right.role.discriminant())
            .then_with(|| {
                left.artifact_kind
                    .as_str()
                    .cmp(right.artifact_kind.as_str())
            })
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
    });
    if expected != dependencies {
        return Err(LineageError::NonCanonicalDependencies);
    }
    if dependencies.windows(2).any(|window| window[0] == window[1]) {
        return Err(LineageError::DuplicateDependency);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageResolutionStatus {
    Complete,
    Incomplete,
    CycleDetected,
    Inconsistent,
    RootMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageResolutionReason {
    LegacyUnknownRoot,
    MissingDependency(ArtifactId),
    CycleDetected { cycle_artifact_ids: Vec<ArtifactId> },
    CatalogRootInconsistent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedArtifactLineage {
    pub status: LineageResolutionStatus,
    pub root_artifact_id: Option<ArtifactId>,
    pub ancestor_artifact_ids: Vec<ArtifactId>,
    pub missing_artifact_ids: Vec<ArtifactId>,
    pub acquisition_families: ResolvedAcquisitionFamilies,
    pub reasons: Vec<LineageResolutionReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceIndependence {
    Independent,
    PartiallyDependent,
    SameSource,
    Unknown,
}

pub fn resolve_lineage(
    root: &ArtifactLineageState,
    catalog: &ArtifactLineageCatalog,
) -> ResolvedArtifactLineage {
    let ArtifactLineageState::Known {
        identity,
        direct_dependencies,
    } = root
    else {
        return ResolvedArtifactLineage {
            status: LineageResolutionStatus::Incomplete,
            root_artifact_id: None,
            ancestor_artifact_ids: Vec::new(),
            missing_artifact_ids: Vec::new(),
            acquisition_families: ResolvedAcquisitionFamilies::Unknown,
            reasons: vec![LineageResolutionReason::LegacyUnknownRoot],
        };
    };

    let mut result = ResolvedArtifactLineage {
        status: LineageResolutionStatus::Complete,
        root_artifact_id: Some(identity.artifact_id.clone()),
        ancestor_artifact_ids: Vec::new(),
        missing_artifact_ids: Vec::new(),
        acquisition_families: match &identity.acquisition_families {
            ArtifactAcquisitionFamilies::Known(values) => {
                ResolvedAcquisitionFamilies::Known(values.clone())
            }
            ArtifactAcquisitionFamilies::Unknown => ResolvedAcquisitionFamilies::Unknown,
        },
        reasons: Vec::new(),
    };
    if let Some(node) = catalog.artifacts.get(&identity.artifact_id)
        && (node.identity != *identity || node.direct_dependencies != *direct_dependencies)
    {
        result.status = LineageResolutionStatus::Inconsistent;
        result
            .reasons
            .push(LineageResolutionReason::CatalogRootInconsistent);
        return result;
    }
    let mut visiting = vec![identity.artifact_id.clone()];
    let mut visited = BTreeSet::new();
    walk_dependencies(
        direct_dependencies,
        catalog,
        &mut visiting,
        &mut visited,
        &mut result,
    );
    visiting.pop();
    result.ancestor_artifact_ids.sort();
    result.ancestor_artifact_ids.dedup();
    result.missing_artifact_ids.sort();
    result.missing_artifact_ids.dedup();
    result.reasons.sort_by_key(reason_sort_key);
    result.reasons.dedup();
    if result
        .reasons
        .iter()
        .any(|reason| matches!(reason, LineageResolutionReason::CycleDetected { .. }))
    {
        result.status = LineageResolutionStatus::CycleDetected;
    } else if !result.missing_artifact_ids.is_empty() {
        result.status = LineageResolutionStatus::Incomplete;
    }
    result
}

fn walk_dependencies(
    dependencies: &[ArtifactDependency],
    catalog: &ArtifactLineageCatalog,
    visiting: &mut Vec<ArtifactId>,
    visited: &mut BTreeSet<ArtifactId>,
    result: &mut ResolvedArtifactLineage,
) {
    let mut dependencies = dependencies.to_vec();
    dependencies.sort_by(|left, right| {
        left.role
            .discriminant()
            .cmp(&right.role.discriminant())
            .then_with(|| {
                left.artifact_kind
                    .as_str()
                    .cmp(right.artifact_kind.as_str())
            })
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
    });
    for dependency in dependencies {
        result
            .ancestor_artifact_ids
            .push(dependency.artifact_id.clone());
        if let Some(start) = visiting.iter().position(|id| id == &dependency.artifact_id) {
            let mut cycle = visiting[start..].to_vec();
            cycle.push(dependency.artifact_id.clone());
            result.reasons.push(LineageResolutionReason::CycleDetected {
                cycle_artifact_ids: cycle,
            });
            continue;
        }
        let Some(node) = catalog.artifacts.get(&dependency.artifact_id) else {
            result
                .missing_artifact_ids
                .push(dependency.artifact_id.clone());
            result
                .reasons
                .push(LineageResolutionReason::MissingDependency(
                    dependency.artifact_id,
                ));
            continue;
        };
        if visited.contains(&dependency.artifact_id) {
            continue;
        }
        visited.insert(dependency.artifact_id.clone());
        if let ArtifactAcquisitionFamilies::Known(families) = &node.identity.acquisition_families {
            result.acquisition_families = result
                .acquisition_families
                .union(&ResolvedAcquisitionFamilies::Known(families.clone()));
        } else {
            result.acquisition_families = ResolvedAcquisitionFamilies::Unknown;
        }
        visiting.push(dependency.artifact_id.clone());
        walk_dependencies(
            &node.direct_dependencies,
            catalog,
            visiting,
            visited,
            result,
        );
        visiting.pop();
    }
}

pub fn resolve_known_artifact_id(
    root_id: &ArtifactId,
    catalog: &ArtifactLineageCatalog,
) -> ResolvedArtifactLineage {
    let Some(node) = catalog.artifacts.get(root_id) else {
        return ResolvedArtifactLineage {
            status: LineageResolutionStatus::RootMissing,
            root_artifact_id: Some(root_id.clone()),
            ancestor_artifact_ids: Vec::new(),
            missing_artifact_ids: vec![root_id.clone()],
            acquisition_families: ResolvedAcquisitionFamilies::Unknown,
            reasons: vec![LineageResolutionReason::MissingDependency(root_id.clone())],
        };
    };
    resolve_lineage(
        &ArtifactLineageState::Known {
            identity: node.identity.clone(),
            direct_dependencies: node.direct_dependencies.clone(),
        },
        catalog,
    )
}

pub fn semantic_sha256<T: Serialize>(value: &T) -> Result<String, LineageError> {
    // serde_jcs implements RFC 8785/JCS, including ECMAScript number
    // formatting and UTF-16 property ordering.  It rejects non-finite floats
    // before any semantic bytes or artifact ID can be produced.
    let canonical =
        serde_jcs::to_vec(value).map_err(|error| LineageError::Serialization(error.to_string()))?;
    Ok(hex_sha256(&canonical))
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_sha256_id(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(is_sha256_hex)
}

fn unique_sorted_experiment_ids(
    values: &[ExperimentId],
) -> Result<Vec<ExperimentId>, LineageError> {
    let mut result = values.to_vec();
    if result.iter().any(|value| value.0.is_empty()) {
        return Err(LineageError::EmptyIdentifier("experiment_id"));
    }
    result.sort();
    result.dedup();
    Ok(result)
}

fn normalize_families(
    values: impl IntoIterator<Item = AcquisitionFamilyId>,
) -> Result<Vec<AcquisitionFamilyId>, LineageError> {
    let mut values = values
        .into_iter()
        .map(|value| AcquisitionFamilyId::new(value.0))
        .collect::<Result<Vec<_>, _>>()?;
    values.sort();
    values.dedup();
    Ok(values)
}

fn validate_normalized_families(values: &[AcquisitionFamilyId]) -> Result<(), LineageError> {
    if values.windows(2).any(|window| window[0] >= window[1])
        || values
            .iter()
            .any(|value| value.0.trim() != value.0 || value.0.is_empty())
    {
        Err(LineageError::NonCanonicalFamilySet)
    } else {
        Ok(())
    }
}

fn reason_sort_key(reason: &LineageResolutionReason) -> (u8, String) {
    match reason {
        LineageResolutionReason::LegacyUnknownRoot => (0, String::new()),
        LineageResolutionReason::MissingDependency(id) => (1, id.0.clone()),
        LineageResolutionReason::CycleDetected { cycle_artifact_ids } => {
            (2, format!("{cycle_artifact_ids:?}"))
        }
        LineageResolutionReason::CatalogRootInconsistent => (3, String::new()),
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LineageError {
    #[error("invalid ArtifactId: {0}")]
    InvalidArtifactId(String),
    #[error("invalid aggregate scope ID: {0}")]
    InvalidAggregateScopeId(String),
    #[error("empty {0}")]
    EmptyIdentifier(&'static str),
    #[error("aggregate scope requires at least two unique members")]
    AggregateNeedsTwoMembers,
    #[error("aggregate members are not canonical")]
    NonCanonicalAggregateMembers,
    #[error("known acquisition-family set must be nonempty")]
    EmptyKnownFamilySet,
    #[error("acquisition-family set is not canonical")]
    NonCanonicalFamilySet,
    #[error("artifact identity is invalid")]
    InvalidArtifactIdentity,
    #[error("dependencies are not in canonical order")]
    NonCanonicalDependencies,
    #[error("duplicate artifact dependency")]
    DuplicateDependency,
    #[error("semantic value serialization failed: {0}")]
    Serialization(String),
}
