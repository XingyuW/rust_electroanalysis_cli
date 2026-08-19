//! Closed public lineage presentation types.
//!
//! These are copied presentation records.  They neither construct identities
//! nor traverse the supplied lineage catalog.

use crate::domain::{
    AnalysisProvenance, ArtifactAcquisitionFamilies, ArtifactDependencyRole,
    ArtifactExperimentScope, ArtifactKind, ArtifactLineageState, ScopeKey, UnknownLineageReason,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineagePresentationStatusV1 {
    Known,
    LegacyUnknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionFamilyStatusV1 {
    Known,
    Unknown,
    LegacyUnknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionFamilyPresentationV1 {
    pub status: AcquisitionFamilyStatusV1,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ExperimentScopeV1 {
    Single {
        experiment_id: String,
    },
    Aggregate {
        aggregate_scope_id: String,
        member_experiment_ids: Vec<String>,
    },
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ScopeKeyV1 {
    Specific { value: String },
    All,
    Unspecified,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicArtifactIdentityV1 {
    pub artifact_id: String,
    pub artifact_kind: ArtifactKind,
    pub schema_version: u32,
    pub producer_version: String,
    pub experiment_scope: ExperimentScopeV1,
    pub sensor_scope: ScopeKeyV1,
    pub channel_scope: ScopeKeyV1,
    pub acquisition_families: AcquisitionFamilyPresentationV1,
    pub semantic_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicDependencyV1 {
    pub artifact_id: String,
    pub artifact_kind: ArtifactKind,
    pub role: ArtifactDependencyRole,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineagePresentationV1 {
    pub status: LineagePresentationStatusV1,
    pub identity: Option<PublicArtifactIdentityV1>,
    pub legacy_source_schema_version: Option<u32>,
    pub legacy_reason: Option<UnknownLineageReason>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenancePresentationV1 {
    pub software_version: String,
    pub input_sha256: String,
    pub configuration_sha256: Option<String>,
    pub git_commit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicLineageRootV1 {
    pub input_flag: crate::reporting::document::InputFlagV1,
    pub lineage: LineagePresentationV1,
    pub direct_dependencies: Vec<PublicDependencyV1>,
    pub root_catalog_entry_present: Option<bool>,
}

pub(crate) fn project_lineage(lineage: &ArtifactLineageState) -> LineagePresentationV1 {
    match lineage {
        ArtifactLineageState::Known { identity, .. } => LineagePresentationV1 {
            status: LineagePresentationStatusV1::Known,
            identity: Some(PublicArtifactIdentityV1 {
                artifact_id: identity.artifact_id.0.clone(),
                artifact_kind: identity.artifact_kind,
                schema_version: identity.schema_version,
                producer_version: identity.producer_version.clone(),
                experiment_scope: project_experiment_scope(&identity.experiment_scope),
                sensor_scope: project_scope_key(&identity.sensor_scope),
                channel_scope: project_scope_key(&identity.channel_scope),
                acquisition_families: project_families(&identity.acquisition_families, false),
                semantic_sha256: identity.semantic_sha256.clone(),
            }),
            legacy_source_schema_version: None,
            legacy_reason: None,
        },
        ArtifactLineageState::LegacyUnknown {
            source_schema_version,
            reason,
        } => LineagePresentationV1 {
            status: LineagePresentationStatusV1::LegacyUnknown,
            identity: None,
            legacy_source_schema_version: *source_schema_version,
            legacy_reason: Some(*reason),
        },
    }
}

pub(crate) fn project_dependencies(lineage: &ArtifactLineageState) -> Vec<PublicDependencyV1> {
    match lineage {
        ArtifactLineageState::Known {
            direct_dependencies,
            ..
        } => direct_dependencies
            .iter()
            .map(|dependency| PublicDependencyV1 {
                artifact_id: dependency.artifact_id.0.clone(),
                artifact_kind: dependency.artifact_kind,
                role: dependency.role.clone(),
            })
            .collect(),
        ArtifactLineageState::LegacyUnknown { .. } => Vec::new(),
    }
}

pub(crate) fn project_families(
    families: &ArtifactAcquisitionFamilies,
    legacy: bool,
) -> AcquisitionFamilyPresentationV1 {
    if legacy {
        return AcquisitionFamilyPresentationV1 {
            status: AcquisitionFamilyStatusV1::LegacyUnknown,
            values: Vec::new(),
        };
    }
    match families {
        ArtifactAcquisitionFamilies::Known(values) => AcquisitionFamilyPresentationV1 {
            status: AcquisitionFamilyStatusV1::Known,
            values: values.iter().map(|value| value.0.clone()).collect(),
        },
        ArtifactAcquisitionFamilies::Unknown => AcquisitionFamilyPresentationV1 {
            status: AcquisitionFamilyStatusV1::Unknown,
            values: Vec::new(),
        },
    }
}

pub(crate) fn project_provenance(
    provenance: Option<&AnalysisProvenance>,
) -> ProvenancePresentationV1 {
    let provenance = provenance.cloned().unwrap_or_else(|| AnalysisProvenance {
        software_version: "not_serialized".into(),
        input_path: "not_serialized".into(),
        input_sha256: "not_serialized".into(),
        configuration_path: None,
        configuration_sha256: None,
        generation_timestamp: 0,
        git_commit: None,
    });
    ProvenancePresentationV1 {
        software_version: provenance.software_version,
        input_sha256: provenance.input_sha256,
        configuration_sha256: provenance.configuration_sha256,
        git_commit: provenance.git_commit,
    }
}

fn project_experiment_scope(value: &ArtifactExperimentScope) -> ExperimentScopeV1 {
    match value {
        ArtifactExperimentScope::Single { experiment_id } => ExperimentScopeV1::Single {
            experiment_id: experiment_id.0.clone(),
        },
        ArtifactExperimentScope::Aggregate {
            aggregate_scope_id,
            member_experiment_ids,
        } => ExperimentScopeV1::Aggregate {
            aggregate_scope_id: aggregate_scope_id.0.clone(),
            member_experiment_ids: member_experiment_ids
                .iter()
                .map(|value| value.0.clone())
                .collect(),
        },
        ArtifactExperimentScope::Unknown => ExperimentScopeV1::Unknown,
    }
}
fn project_scope_key(value: &ScopeKey) -> ScopeKeyV1 {
    match value {
        ScopeKey::Specific(value) => ScopeKeyV1::Specific {
            value: value.clone(),
        },
        ScopeKey::All => ScopeKeyV1::All,
        ScopeKey::Unspecified => ScopeKeyV1::Unspecified,
    }
}
