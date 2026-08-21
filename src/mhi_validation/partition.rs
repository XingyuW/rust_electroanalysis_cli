//! Deterministic Phase-E membership, closure, leakage, and accounting.
//!
//! This module deliberately performs no Phase-B/Phase-C interpretation.  It
//! turns immutable dataset/catalog/reference authority into a complete
//! endpoint × view × record ledger which both result mappers consume.

use super::{MhiValidationError, reader::ValidationInputs};
use crate::{
    domain::{
        ArtifactAcquisitionFamilies, ArtifactExperimentScope, ArtifactId, ArtifactLineageCatalog,
    },
    results::{
        ArtifactSourceExpectationV1, ExpectedLineageV1, ReferenceDependencyV1, ReferenceEndpointV1,
        ReferenceSourceAuthorityV1, ScientificSourceKeyV1, ValidationRecordV1,
    },
    validation_config::{
        BlindingRuleV1, BlindingStateV1, CohortRoleV1, DomainSelectorV1, ExclusionReasonV1,
        LeakageNotEvaluatedReasonV1, RecordDecisionV1, ReferenceAuthorityRuleV1,
        ReferenceDependencyCompletenessV1, ReferenceUncertaintyRuleV1, RequiredStratumV1,
        SeparationStatusV1, SeparationUnknownReasonV1,
    },
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointSource {
    Mechanism,
    Health,
}

#[derive(Debug, Clone)]
pub struct EndpointPartitionSpec<'a> {
    pub endpoint_id: &'a str,
    pub cohort_role: CohortRoleV1,
    pub domain: &'a DomainSelectorV1,
    pub required_strata: &'a [RequiredStratumV1],
    pub reference_rule: &'a ReferenceAuthorityRuleV1,
    pub source: EndpointSource,
    pub physical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionRowV1 {
    pub endpoint_id: String,
    pub stratum_id: String,
    pub record_id: String,
    pub decision: RecordDecisionV1,
    pub primary_reason: Option<ExclusionReasonV1>,
    pub secondary_reasons: Vec<ExclusionReasonV1>,
    pub assessed_source_key: Option<ScientificSourceKeyV1>,
    pub reference_endpoint_id: Option<String>,
    pub separation_status: Option<SeparationStatusV1>,
    pub not_evaluated_reason: Option<LeakageNotEvaluatedReasonV1>,
    pub compared_development_record_ids: Vec<String>,
    pub shared_artifact_ids: Vec<ArtifactId>,
    pub shared_source_sha256s: Vec<String>,
    pub shared_experiment_ids: Vec<String>,
    pub shared_family_ids: Vec<String>,
    pub unknown_reasons: Vec<SeparationUnknownReasonV1>,
}

#[derive(Debug, Clone)]
pub struct EndpointPartitionV1 {
    pub rows: Vec<PartitionRowV1>,
}

#[derive(Debug, Default, Clone)]
struct Closure {
    artifact_ids: BTreeSet<ArtifactId>,
    semantic_sha256s: BTreeSet<String>,
    source_sha256s: BTreeSet<String>,
    experiment_ids: BTreeSet<String>,
    family_ids: BTreeSet<String>,
    unknown: BTreeSet<SeparationUnknownReasonV1>,
}

impl Closure {
    fn union(&mut self, other: &Self) {
        self.artifact_ids.extend(other.artifact_ids.iter().cloned());
        self.semantic_sha256s
            .extend(other.semantic_sha256s.iter().cloned());
        self.source_sha256s
            .extend(other.source_sha256s.iter().cloned());
        self.experiment_ids
            .extend(other.experiment_ids.iter().cloned());
        self.family_ids.extend(other.family_ids.iter().cloned());
        self.unknown.extend(other.unknown.iter().copied());
    }
}

#[derive(Debug, Clone, Copy)]
enum ClosureRole {
    Assessed,
    Reference,
    Development,
}

impl ClosureRole {
    fn scope_unknown(self) -> SeparationUnknownReasonV1 {
        match self {
            Self::Assessed => SeparationUnknownReasonV1::AssessedExperimentScopeUnknown,
            Self::Reference => SeparationUnknownReasonV1::ReferenceExperimentScopeUnknown,
            Self::Development => SeparationUnknownReasonV1::DevelopmentExperimentScopeUnknown,
        }
    }
    fn family_unknown(self) -> SeparationUnknownReasonV1 {
        match self {
            Self::Assessed => SeparationUnknownReasonV1::AssessedFamilyUnknown,
            Self::Reference => SeparationUnknownReasonV1::ReferenceFamilyUnknown,
            Self::Development => SeparationUnknownReasonV1::DevelopmentFamilyUnknown,
        }
    }
}

#[derive(Debug, Clone)]
struct Separation {
    status: SeparationStatusV1,
    shared_artifact_ids: Vec<ArtifactId>,
    shared_source_sha256s: Vec<String>,
    shared_experiment_ids: Vec<String>,
    shared_family_ids: Vec<String>,
    unknown_reasons: Vec<SeparationUnknownReasonV1>,
}

pub fn partition_endpoint(
    inputs: &ValidationInputs,
    spec: EndpointPartitionSpec<'_>,
) -> Result<EndpointPartitionV1, MhiValidationError> {
    let catalog = &inputs.lineage_catalog.catalog;
    let reference_sources = inputs
        .dataset
        .artifact
        .reference_sources
        .iter()
        .map(|source| (source.reference_source_id.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    if spec.physical {
        for record in &inputs.dataset.artifact.records {
            if record.cohort_role != spec.cohort_role || !spec.domain.contains(&record.domain) {
                continue;
            }
            if record.evidence_origin != crate::validation_config::EvidenceOriginV1::Physical
                || record_source(record, spec.source).is_none_or(is_not_scoreable)
            {
                return Err(MhiValidationError::Dataset(
                    "PhysicalReferenceAuthorityMismatch".into(),
                ));
            }
            let reference = matching_reference(record, spec.endpoint_id).ok_or_else(|| {
                MhiValidationError::Dataset("PhysicalReferenceAuthorityMismatch".into())
            })?;
            ensure_physical_reference(
                reference_source_id(reference),
                &reference_sources,
                &mut BTreeSet::new(),
            )?;
        }
    }
    let mut rows = Vec::new();
    for view in views(spec.required_strata) {
        let development = inputs
            .dataset
            .artifact
            .records
            .iter()
            .filter(|record| {
                record.cohort_role == CohortRoleV1::Development
                    && spec.domain.contains(&record.domain)
                    && view.contains(record)
            })
            .collect::<Vec<_>>();
        let mut comparator = Closure::default();
        for record in &development {
            if let Some(source) = record_source(record, spec.source) {
                comparator.union(&source_closure(source, catalog, ClosureRole::Development)?);
            }
            if let Some(reference) = matching_reference(record, spec.endpoint_id) {
                comparator.union(&reference_closure(
                    reference,
                    &reference_sources,
                    catalog,
                    ClosureRole::Development,
                )?);
            }
        }
        let comparator_ids = development
            .iter()
            .map(|record| record.record_id.clone())
            .collect::<Vec<_>>();
        for record in &inputs.dataset.artifact.records {
            rows.push(partition_record(
                record,
                &spec,
                &view,
                catalog,
                &reference_sources,
                &comparator,
                &comparator_ids,
            )?);
        }
    }
    rows.sort_by(|left, right| {
        left.endpoint_id
            .cmp(&right.endpoint_id)
            .then_with(|| view_key(&left.stratum_id).cmp(&view_key(&right.stratum_id)))
            .then_with(|| left.record_id.cmp(&right.record_id))
    });
    Ok(EndpointPartitionV1 { rows })
}

fn ensure_physical_reference(
    source_id: &str,
    sources: &BTreeMap<&str, &ReferenceSourceAuthorityV1>,
    visiting: &mut BTreeSet<String>,
) -> Result<(), MhiValidationError> {
    if !visiting.insert(source_id.into()) {
        return Err(MhiValidationError::Dataset("ReferenceSourceCycle".into()));
    }
    let source = sources
        .get(source_id)
        .ok_or_else(|| MhiValidationError::Dataset("PhysicalReferenceAuthorityMismatch".into()))?;
    if source.evidence_origin != crate::validation_config::EvidenceOriginV1::Physical
        || source.dependency_completeness != ReferenceDependencyCompletenessV1::Complete
    {
        return Err(MhiValidationError::Dataset(
            "PhysicalReferenceAuthorityMismatch".into(),
        ));
    }
    for dependency in &source.direct_dependencies {
        if let ReferenceDependencyV1::ReferenceSource {
            reference_source_id,
        } = dependency
        {
            ensure_physical_reference(reference_source_id, sources, visiting)?;
        }
    }
    visiting.remove(source_id);
    Ok(())
}

fn partition_record(
    record: &ValidationRecordV1,
    spec: &EndpointPartitionSpec<'_>,
    view: &View<'_>,
    catalog: &ArtifactLineageCatalog,
    reference_sources: &BTreeMap<&str, &ReferenceSourceAuthorityV1>,
    comparator: &Closure,
    comparator_ids: &[String],
) -> Result<PartitionRowV1, MhiValidationError> {
    let applicable = record.cohort_role == spec.cohort_role
        && spec.domain.contains(&record.domain)
        && view.contains(record);
    if !applicable {
        return Ok(PartitionRowV1 {
            endpoint_id: spec.endpoint_id.into(),
            stratum_id: view.id.into(),
            record_id: record.record_id.clone(),
            decision: RecordDecisionV1::NotApplicable,
            primary_reason: None,
            secondary_reasons: Vec::new(),
            assessed_source_key: None,
            reference_endpoint_id: None,
            separation_status: None,
            not_evaluated_reason: Some(LeakageNotEvaluatedReasonV1::NotApplicable),
            compared_development_record_ids: Vec::new(),
            shared_artifact_ids: Vec::new(),
            shared_source_sha256s: Vec::new(),
            shared_experiment_ids: Vec::new(),
            shared_family_ids: Vec::new(),
            unknown_reasons: Vec::new(),
        });
    }
    let source = record_source(record, spec.source);
    let reference = matching_reference(record, spec.endpoint_id);
    let source_key = source.map(scientific_source_key);
    let reference_id = reference.map(reference_endpoint_id);
    let assessed = source
        .map(|value| source_closure(value, catalog, ClosureRole::Assessed))
        .transpose()?;
    let reference_closure_value = reference
        .map(|value| reference_closure(value, reference_sources, catalog, ClosureRole::Reference))
        .transpose()?;

    let mut reasons = Vec::new();
    if source.is_none() {
        reasons.push(ExclusionReasonV1::MissingEndpointArtifactPath);
    }
    if source.is_some_and(is_not_scoreable) {
        reasons.push(ExclusionReasonV1::SourceNotPhaseBOrCScoreable);
    }
    if reference.is_none() {
        reasons.push(ExclusionReasonV1::MissingReferenceEndpoint);
    }
    if let Some(reference) = reference {
        reasons.extend(reference_exclusion_reasons(
            spec.reference_rule,
            reference,
            spec.physical,
        )?);
    }

    let separation = match (&assessed, &reference_closure_value) {
        (Some(assessed), Some(reference)) => {
            let mut evaluated = assessed.clone();
            evaluated.union(reference);
            Some(classify_separation(
                &evaluated,
                comparator,
                Some((assessed, reference)),
            ))
        }
        _ => None,
    };
    if record.cohort_role == CohortRoleV1::Validation {
        match separation.as_ref().map(|value| value.status) {
            Some(SeparationStatusV1::KnownOverlap) => {
                reasons.push(ExclusionReasonV1::ValidationKnownOverlap)
            }
            Some(SeparationStatusV1::UnknownSeparation) => {
                reasons.push(ExclusionReasonV1::ValidationUnknownSeparation)
            }
            _ => {}
        }
    }
    reasons.sort_by_key(|reason| reason.ordinal());
    reasons.dedup();
    if spec.physical && !reasons.is_empty() {
        return Err(MhiValidationError::Dataset(
            "PhysicalReferenceAuthorityMismatch".into(),
        ));
    }
    let decision = if reasons.is_empty() {
        RecordDecisionV1::Eligible
    } else {
        RecordDecisionV1::Excluded
    };
    let primary_reason = reasons.first().copied();
    let secondary_reasons = reasons.iter().skip(1).copied().collect();
    let (
        separation_status,
        not_evaluated_reason,
        shared_artifact_ids,
        shared_source_sha256s,
        shared_experiment_ids,
        shared_family_ids,
        unknown_reasons,
    ) = if let Some(value) = separation {
        (
            Some(value.status),
            None,
            value.shared_artifact_ids,
            value.shared_source_sha256s,
            value.shared_experiment_ids,
            value.shared_family_ids,
            value.unknown_reasons,
        )
    } else {
        (
            None,
            primary_reason.map(not_evaluated_reason),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    };
    Ok(PartitionRowV1 {
        endpoint_id: spec.endpoint_id.into(),
        stratum_id: view.id.into(),
        record_id: record.record_id.clone(),
        decision,
        primary_reason,
        secondary_reasons,
        assessed_source_key: source_key,
        reference_endpoint_id: reference_id,
        separation_status,
        not_evaluated_reason,
        compared_development_record_ids: comparator_ids.to_vec(),
        shared_artifact_ids,
        shared_source_sha256s,
        shared_experiment_ids,
        shared_family_ids,
        unknown_reasons,
    })
}

fn not_evaluated_reason(reason: ExclusionReasonV1) -> LeakageNotEvaluatedReasonV1 {
    match reason {
        ExclusionReasonV1::MissingEndpointArtifactPath => {
            LeakageNotEvaluatedReasonV1::MissingEndpointArtifactPath
        }
        ExclusionReasonV1::MissingReferenceEndpoint => {
            LeakageNotEvaluatedReasonV1::MissingReferenceEndpoint
        }
        _ => LeakageNotEvaluatedReasonV1::MissingReferenceEndpoint,
    }
}

fn source_closure(
    source: &ArtifactSourceExpectationV1,
    catalog: &ArtifactLineageCatalog,
    role: ClosureRole,
) -> Result<Closure, MhiValidationError> {
    let ExpectedLineageV1::Known {
        artifact_id,
        semantic_sha256,
    } = &source.expected_lineage
    else {
        let mut closure = Closure::default();
        closure.unknown.insert(role.scope_unknown());
        closure.unknown.insert(role.family_unknown());
        return Ok(closure);
    };
    let node = catalog
        .artifacts
        .get(artifact_id)
        .ok_or_else(|| MhiValidationError::Dataset("AssessedRootMissing".into()))?;
    if node.identity.artifact_kind != source.expected_artifact_kind
        || node.identity.semantic_sha256 != *semantic_sha256
    {
        return Err(MhiValidationError::Dataset(
            "AssessedRootIdentityMismatch".into(),
        ));
    }
    let mut closure = Closure::default();
    closure
        .source_sha256s
        .insert(source.source_file_sha256.clone());
    let mut visiting = BTreeSet::new();
    walk_catalog(artifact_id, catalog, role, &mut visiting, &mut closure)?;
    Ok(closure)
}

fn reference_closure(
    endpoint: &ReferenceEndpointV1,
    reference_sources: &BTreeMap<&str, &ReferenceSourceAuthorityV1>,
    catalog: &ArtifactLineageCatalog,
    role: ClosureRole,
) -> Result<Closure, MhiValidationError> {
    let mut closure = Closure::default();
    let mut visiting = BTreeSet::new();
    walk_reference(
        reference_source_id(endpoint),
        reference_sources,
        catalog,
        role,
        &mut visiting,
        &mut closure,
    )?;
    Ok(closure)
}

fn walk_reference(
    source_id: &str,
    reference_sources: &BTreeMap<&str, &ReferenceSourceAuthorityV1>,
    catalog: &ArtifactLineageCatalog,
    role: ClosureRole,
    visiting: &mut BTreeSet<String>,
    closure: &mut Closure,
) -> Result<(), MhiValidationError> {
    if !visiting.insert(source_id.into()) {
        return Err(MhiValidationError::Dataset("ReferenceSourceCycle".into()));
    }
    let Some(source) = reference_sources.get(source_id) else {
        closure
            .unknown
            .insert(SeparationUnknownReasonV1::ReferenceDependencyNodeMissing);
        visiting.remove(source_id);
        return Ok(());
    };
    closure
        .source_sha256s
        .insert(source.source_file_sha256.clone());
    add_scope_and_families(
        &source.experiment_scope,
        &source.acquisition_families,
        role,
        closure,
    );
    if source.dependency_completeness == ReferenceDependencyCompletenessV1::Unknown {
        closure
            .unknown
            .insert(SeparationUnknownReasonV1::ReferenceDependencyIncomplete);
    }
    for dependency in &source.direct_dependencies {
        match dependency {
            ReferenceDependencyV1::ReferenceSource {
                reference_source_id,
            } => walk_reference(
                reference_source_id,
                reference_sources,
                catalog,
                role,
                visiting,
                closure,
            )?,
            ReferenceDependencyV1::ScientificArtifact { source } => {
                walk_scientific_leaf(source, catalog, role, closure)?
            }
        }
    }
    visiting.remove(source_id);
    Ok(())
}

fn walk_scientific_leaf(
    source: &ScientificSourceKeyV1,
    catalog: &ArtifactLineageCatalog,
    role: ClosureRole,
    closure: &mut Closure,
) -> Result<(), MhiValidationError> {
    let ScientificSourceKeyV1::Known {
        artifact_kind,
        artifact_id,
        semantic_sha256,
    } = source
    else {
        closure
            .unknown
            .insert(SeparationUnknownReasonV1::ReferenceScientificLeafLegacyUnknown);
        return Ok(());
    };
    let Some(node) = catalog.artifacts.get(artifact_id) else {
        closure
            .unknown
            .insert(SeparationUnknownReasonV1::ReferenceScientificLeafMissing);
        return Ok(());
    };
    if node.identity.artifact_kind != *artifact_kind
        || node.identity.semantic_sha256 != *semantic_sha256
    {
        return Err(MhiValidationError::Dataset(
            "ReferenceLeafIdentityMismatch".into(),
        ));
    }
    let mut visiting = BTreeSet::new();
    walk_catalog(artifact_id, catalog, role, &mut visiting, closure)
}

fn walk_catalog(
    artifact_id: &ArtifactId,
    catalog: &ArtifactLineageCatalog,
    role: ClosureRole,
    visiting: &mut BTreeSet<ArtifactId>,
    closure: &mut Closure,
) -> Result<(), MhiValidationError> {
    if !visiting.insert(artifact_id.clone()) {
        closure
            .unknown
            .insert(SeparationUnknownReasonV1::CatalogCycleReachable);
        return Ok(());
    }
    let Some(node) = catalog.artifacts.get(artifact_id) else {
        closure
            .unknown
            .insert(SeparationUnknownReasonV1::CatalogAncestorMissing);
        visiting.remove(artifact_id);
        return Ok(());
    };
    closure
        .artifact_ids
        .insert(node.identity.artifact_id.clone());
    closure
        .semantic_sha256s
        .insert(node.identity.semantic_sha256.clone());
    add_scope_and_families(
        &node.identity.experiment_scope,
        &node.identity.acquisition_families,
        role,
        closure,
    );
    for dependency in &node.direct_dependencies {
        match catalog.artifacts.get(&dependency.artifact_id) {
            Some(dependency_node)
                if dependency_node.identity.artifact_kind != dependency.artifact_kind =>
            {
                return Err(MhiValidationError::Dataset(
                    "CatalogDependencyKindMismatch".into(),
                ));
            }
            Some(_) => walk_catalog(&dependency.artifact_id, catalog, role, visiting, closure)?,
            None => {
                closure
                    .unknown
                    .insert(SeparationUnknownReasonV1::CatalogAncestorMissing);
            }
        }
    }
    visiting.remove(artifact_id);
    Ok(())
}

fn add_scope_and_families(
    scope: &ArtifactExperimentScope,
    families: &ArtifactAcquisitionFamilies,
    role: ClosureRole,
    closure: &mut Closure,
) {
    match scope {
        ArtifactExperimentScope::Single { experiment_id } => {
            closure.experiment_ids.insert(experiment_id.0.clone());
        }
        ArtifactExperimentScope::Aggregate {
            member_experiment_ids,
            ..
        } => closure
            .experiment_ids
            .extend(member_experiment_ids.iter().map(|id| id.0.clone())),
        ArtifactExperimentScope::Unknown => {
            closure.unknown.insert(role.scope_unknown());
        }
    }
    match families {
        ArtifactAcquisitionFamilies::Known(values) => closure
            .family_ids
            .extend(values.iter().map(|id| id.0.clone())),
        ArtifactAcquisitionFamilies::Unknown => {
            closure.unknown.insert(role.family_unknown());
        }
    }
}

fn classify_separation(
    evaluated: &Closure,
    comparator: &Closure,
    self_closures: Option<(&Closure, &Closure)>,
) -> Separation {
    let evaluated_vs_development = classify_pair(evaluated, comparator, BTreeSet::new());
    let assessed_vs_reference = self_closures
        .map(|(assessed, reference)| classify_pair(assessed, reference, BTreeSet::new()));
    let mut artifacts = evaluated_vs_development.shared_artifact_ids;
    let mut sources = evaluated_vs_development.shared_source_sha256s;
    let mut experiments = evaluated_vs_development.shared_experiment_ids;
    let mut families = evaluated_vs_development.shared_family_ids;
    if let Some(self_overlap) = assessed_vs_reference {
        artifacts.extend(self_overlap.shared_artifact_ids);
        sources.extend(self_overlap.shared_source_sha256s);
        experiments.extend(self_overlap.shared_experiment_ids);
        families.extend(self_overlap.shared_family_ids);
    }
    artifacts.sort();
    artifacts.dedup();
    sources.sort();
    sources.dedup();
    experiments.sort();
    experiments.dedup();
    families.sort();
    families.dedup();
    let mut unknown = evaluated.unknown.clone();
    unknown.extend(comparator.unknown.iter().copied());
    let status = if !artifacts.is_empty()
        || !sources.is_empty()
        || !experiments.is_empty()
        || !families.is_empty()
    {
        SeparationStatusV1::KnownOverlap
    } else if unknown.is_empty() {
        SeparationStatusV1::KnownSeparated
    } else {
        SeparationStatusV1::UnknownSeparation
    };
    Separation {
        status,
        shared_artifact_ids: artifacts,
        shared_source_sha256s: sources,
        shared_experiment_ids: experiments,
        shared_family_ids: families,
        unknown_reasons: unknown.into_iter().collect(),
    }
}

fn classify_pair(
    left: &Closure,
    right: &Closure,
    mut unknown: BTreeSet<SeparationUnknownReasonV1>,
) -> Separation {
    unknown.extend(left.unknown.iter().copied());
    unknown.extend(right.unknown.iter().copied());
    let artifacts = left
        .artifact_ids
        .intersection(&right.artifact_ids)
        .cloned()
        .collect::<Vec<_>>();
    let sources = left
        .source_sha256s
        .intersection(&right.source_sha256s)
        .cloned()
        .collect::<Vec<_>>();
    let experiments = left
        .experiment_ids
        .intersection(&right.experiment_ids)
        .cloned()
        .collect::<Vec<_>>();
    let families = left
        .family_ids
        .intersection(&right.family_ids)
        .cloned()
        .collect::<Vec<_>>();
    let semantic_overlap = left
        .semantic_sha256s
        .intersection(&right.semantic_sha256s)
        .next()
        .is_some();
    let status = if !artifacts.is_empty()
        || !sources.is_empty()
        || !experiments.is_empty()
        || !families.is_empty()
        || semantic_overlap
    {
        SeparationStatusV1::KnownOverlap
    } else if !unknown.is_empty() {
        SeparationStatusV1::UnknownSeparation
    } else {
        SeparationStatusV1::KnownSeparated
    };
    Separation {
        status,
        shared_artifact_ids: artifacts,
        shared_source_sha256s: sources,
        shared_experiment_ids: experiments,
        shared_family_ids: families,
        unknown_reasons: unknown.into_iter().collect(),
    }
}

fn record_source(
    record: &ValidationRecordV1,
    source: EndpointSource,
) -> Option<&ArtifactSourceExpectationV1> {
    match source {
        EndpointSource::Mechanism => record.mechanism_source.as_ref(),
        EndpointSource::Health => record.health_source.as_ref(),
    }
}

fn scientific_source_key(source: &ArtifactSourceExpectationV1) -> ScientificSourceKeyV1 {
    match &source.expected_lineage {
        ExpectedLineageV1::Known {
            artifact_id,
            semantic_sha256,
        } => ScientificSourceKeyV1::Known {
            artifact_kind: source.expected_artifact_kind,
            artifact_id: artifact_id.clone(),
            semantic_sha256: semantic_sha256.clone(),
        },
        ExpectedLineageV1::LegacyUnknown { schema_version, .. } => {
            ScientificSourceKeyV1::LegacyUnknown {
                artifact_kind: source.expected_artifact_kind,
                schema_version: *schema_version,
                source_file_sha256: source.source_file_sha256.clone(),
            }
        }
    }
}

fn is_not_scoreable(source: &ArtifactSourceExpectationV1) -> bool {
    !matches!(source.expected_lineage, ExpectedLineageV1::Known { .. })
        || source.expected_schema_version != 4
}

pub fn matching_reference<'a>(
    record: &'a ValidationRecordV1,
    endpoint_id: &str,
) -> Option<&'a ReferenceEndpointV1> {
    record
        .reference_endpoints
        .iter()
        .find(|reference| reference_endpoint_binding_id(reference) == endpoint_id)
}
fn reference_endpoint_binding_id(reference: &ReferenceEndpointV1) -> &str {
    match reference {
        ReferenceEndpointV1::Mechanism { endpoint_id, .. }
        | ReferenceEndpointV1::Health { endpoint_id, .. } => endpoint_id,
    }
}
fn reference_endpoint_id(reference: &ReferenceEndpointV1) -> String {
    match reference {
        ReferenceEndpointV1::Mechanism {
            reference_endpoint_id,
            ..
        }
        | ReferenceEndpointV1::Health {
            reference_endpoint_id,
            ..
        } => reference_endpoint_id.clone(),
    }
}
fn reference_source_id(reference: &ReferenceEndpointV1) -> &str {
    match reference {
        ReferenceEndpointV1::Mechanism {
            reference_source_id,
            ..
        }
        | ReferenceEndpointV1::Health {
            reference_source_id,
            ..
        } => reference_source_id,
    }
}

pub fn reference_exclusion_reasons(
    rule: &ReferenceAuthorityRuleV1,
    endpoint: &ReferenceEndpointV1,
    physical: bool,
) -> Result<Vec<ExclusionReasonV1>, MhiValidationError> {
    let (allowed_methods, authorities, blinding, uncertainty) = match rule {
        ReferenceAuthorityRuleV1::Mechanism {
            allowed_methods,
            allowed_authority_ids,
            blinding_rule,
            uncertainty_rule,
        }
        | ReferenceAuthorityRuleV1::Health {
            allowed_methods,
            allowed_authority_ids,
            blinding_rule,
            uncertainty_rule,
        } => (
            allowed_methods,
            allowed_authority_ids,
            blinding_rule,
            uncertainty_rule,
        ),
    };
    let (
        method_id,
        method_version,
        authority_id,
        blinding_state,
        actual_uncertainty,
        unavailable_outcome,
    ) = match endpoint {
        ReferenceEndpointV1::Mechanism {
            method_id,
            method_version,
            authority_id,
            blinding_state,
            uncertainty,
            outcome,
            ..
        } => (
            method_id,
            method_version,
            authority_id,
            blinding_state,
            uncertainty,
            matches!(
                outcome,
                crate::results::MechanismReferenceOutcomeV1::Unavailable
            ),
        ),
        ReferenceEndpointV1::Health {
            method_id,
            method_version,
            authority_id,
            blinding_state,
            uncertainty,
            ..
        } => (
            method_id,
            method_version,
            authority_id,
            blinding_state,
            uncertainty,
            false,
        ),
    };
    if physical && unavailable_outcome {
        return Err(MhiValidationError::Dataset(
            "PhysicalReferenceOutcomeUnavailable".into(),
        ));
    }
    let mut reasons = Vec::new();
    if unavailable_outcome {
        reasons.push(ExclusionReasonV1::ReferenceOutcomeUnavailable);
    }
    if !allowed_methods
        .iter()
        .any(|method| method.method_id == *method_id && method.method_version == *method_version)
    {
        reasons.push(ExclusionReasonV1::ReferenceMethodNotAllowed);
    }
    if !authorities.iter().any(|value| value == authority_id) {
        reasons.push(ExclusionReasonV1::ReferenceAuthorityNotAllowed);
    }
    let blinded = match blinding {
        BlindingRuleV1::RequireBlinded => *blinding_state == BlindingStateV1::BlindedToAssessment,
        BlindingRuleV1::AllowDeclaredUnblinded => matches!(
            blinding_state,
            BlindingStateV1::BlindedToAssessment | BlindingStateV1::NotBlinded
        ),
    };
    if !blinded {
        reasons.push(ExclusionReasonV1::ReferenceBlindingNotAllowed);
    }
    match (uncertainty, actual_uncertainty) {
        (
            ReferenceUncertaintyRuleV1::RequireQuantified { .. },
            crate::results::ReferenceUncertaintyV1::Unavailable { .. },
        ) => reasons.push(ExclusionReasonV1::ReferenceUncertaintyUnavailable),
        (
            ReferenceUncertaintyRuleV1::RequireQuantified {
                measure_id,
                unit,
                maximum_inclusive,
            },
            crate::results::ReferenceUncertaintyV1::Quantified {
                measure_id: actual_measure,
                value,
                unit: actual_unit,
            },
        ) => {
            if measure_id != actual_measure {
                reasons.push(ExclusionReasonV1::ReferenceUncertaintyMeasureMismatch);
            }
            if unit != actual_unit {
                reasons.push(ExclusionReasonV1::ReferenceUncertaintyUnitMismatch);
            }
            if value > maximum_inclusive {
                reasons.push(ExclusionReasonV1::ReferenceUncertaintyAboveMaximum);
            }
        }
        _ => {}
    }
    reasons.sort_by_key(|reason| reason.ordinal());
    reasons.dedup();
    Ok(reasons)
}

#[derive(Clone, Copy)]
struct View<'a> {
    id: &'a str,
    stratum: Option<&'a RequiredStratumV1>,
}
impl<'a> View<'a> {
    fn contains(&self, record: &ValidationRecordV1) -> bool {
        self.stratum.is_none_or(|stratum| {
            stratum
                .predicates
                .iter()
                .all(|predicate| predicate.contains(&record.domain))
        })
    }
}
fn views(strata: &[RequiredStratumV1]) -> Vec<View<'_>> {
    std::iter::once(View {
        id: "overall",
        stratum: None,
    })
    .chain(strata.iter().map(|stratum| View {
        id: &stratum.stratum_id,
        stratum: Some(stratum),
    }))
    .collect()
}
fn view_key(value: &str) -> (u8, &str) {
    (u8::from(value != "overall"), value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AcquisitionFamilyId, ArtifactDependency, ArtifactDependencyRole, ArtifactIdentity,
        ArtifactLineageNode, ExperimentId, ScopeKey,
    };

    fn id(hex: char) -> ArtifactId {
        ArtifactId::new(format!("sha256:{}", hex.to_string().repeat(64))).expect("artifact ID")
    }

    fn node(hex: char, dependencies: Vec<ArtifactDependency>) -> ArtifactLineageNode {
        let semantic = hex.to_string().repeat(64);
        ArtifactLineageNode {
            identity: ArtifactIdentity {
                artifact_id: id(hex),
                artifact_kind: crate::domain::ArtifactKind::MechanismAnalysis,
                schema_version: 4,
                producer_version: "test".into(),
                experiment_scope: ArtifactExperimentScope::Single {
                    experiment_id: ExperimentId::new(format!("experiment_{hex}"))
                        .expect("experiment"),
                },
                sensor_scope: ScopeKey::Unspecified,
                channel_scope: ScopeKey::Unspecified,
                acquisition_families: ArtifactAcquisitionFamilies::Known(vec![
                    AcquisitionFamilyId::new(format!("family_{hex}")).expect("family"),
                ]),
                semantic_sha256: semantic,
            },
            direct_dependencies: dependencies,
        }
    }

    #[test]
    fn phase_e_holdout_unknown_separation_is_indeterminate_without_fabrication() {
        let root = node(
            'a',
            vec![ArtifactDependency {
                artifact_id: id('b'),
                artifact_kind: crate::domain::ArtifactKind::MechanismAnalysis,
                role: ArtifactDependencyRole::DerivedFrom,
            }],
        );
        let mut catalog = ArtifactLineageCatalog::default();
        catalog.artifacts.insert(id('a'), root);
        let mut closure = Closure::default();
        walk_catalog(
            &id('a'),
            &catalog,
            ClosureRole::Assessed,
            &mut BTreeSet::new(),
            &mut closure,
        )
        .expect("walk");
        assert!(
            closure
                .unknown
                .contains(&SeparationUnknownReasonV1::CatalogAncestorMissing)
        );
        assert_eq!(
            classify_pair(&closure, &Closure::default(), BTreeSet::new()).status,
            SeparationStatusV1::UnknownSeparation
        );
    }

    #[test]
    fn phase_e_reachable_catalog_cycle_is_unknown_separation_not_independence() {
        let first = node(
            'a',
            vec![ArtifactDependency {
                artifact_id: id('b'),
                artifact_kind: crate::domain::ArtifactKind::MechanismAnalysis,
                role: ArtifactDependencyRole::DerivedFrom,
            }],
        );
        let second = node(
            'b',
            vec![ArtifactDependency {
                artifact_id: id('a'),
                artifact_kind: crate::domain::ArtifactKind::MechanismAnalysis,
                role: ArtifactDependencyRole::DerivedFrom,
            }],
        );
        let mut catalog = ArtifactLineageCatalog::default();
        catalog.artifacts.insert(id('a'), first);
        catalog.artifacts.insert(id('b'), second);
        let mut closure = Closure::default();
        walk_catalog(
            &id('a'),
            &catalog,
            ClosureRole::Assessed,
            &mut BTreeSet::new(),
            &mut closure,
        )
        .expect("walk");
        assert!(
            closure
                .unknown
                .contains(&SeparationUnknownReasonV1::CatalogCycleReachable)
        );
    }

    #[test]
    fn phase_e_holdout_rejects_known_lineage_scope_and_family_overlap() {
        let root = node('a', Vec::new());
        let mut catalog = ArtifactLineageCatalog::default();
        catalog.artifacts.insert(id('a'), root);
        let mut assessed = Closure::default();
        walk_catalog(
            &id('a'),
            &catalog,
            ClosureRole::Assessed,
            &mut BTreeSet::new(),
            &mut assessed,
        )
        .expect("walk");
        let result = classify_pair(&assessed, &assessed, BTreeSet::new());
        assert_eq!(result.status, SeparationStatusV1::KnownOverlap);
        assert!(!result.shared_artifact_ids.is_empty());
        assert!(!result.shared_experiment_ids.is_empty());
        assert!(!result.shared_family_ids.is_empty());
    }

    #[test]
    fn phase_e_combined_reference_catalog_closure_and_authority_are_total() {
        let root = node('a', Vec::new());
        let mut catalog = ArtifactLineageCatalog::default();
        catalog.artifacts.insert(id('a'), root);
        let scientific = ScientificSourceKeyV1::Known {
            artifact_kind: crate::domain::ArtifactKind::MechanismAnalysis,
            artifact_id: id('a'),
            semantic_sha256: "a".repeat(64),
        };
        let source_b = ReferenceSourceAuthorityV1 {
            reference_source_id: "reference_b".into(),
            source_file_sha256: "b".repeat(64),
            evidence_origin: crate::validation_config::EvidenceOriginV1::Synthetic,
            dependency_completeness: ReferenceDependencyCompletenessV1::Complete,
            experiment_scope: ArtifactExperimentScope::Single {
                experiment_id: ExperimentId::new("reference_experiment").expect("experiment"),
            },
            acquisition_families: ArtifactAcquisitionFamilies::Known(vec![
                AcquisitionFamilyId::new("reference_family").expect("family"),
            ]),
            direct_dependencies: vec![ReferenceDependencyV1::ScientificArtifact {
                source: scientific,
            }],
        };
        let source_a = ReferenceSourceAuthorityV1 {
            reference_source_id: "reference_a".into(),
            source_file_sha256: "c".repeat(64),
            evidence_origin: crate::validation_config::EvidenceOriginV1::Synthetic,
            dependency_completeness: ReferenceDependencyCompletenessV1::Complete,
            experiment_scope: ArtifactExperimentScope::Single {
                experiment_id: ExperimentId::new("reference_experiment_a").expect("experiment"),
            },
            acquisition_families: ArtifactAcquisitionFamilies::Known(vec![
                AcquisitionFamilyId::new("reference_family_a").expect("family"),
            ]),
            direct_dependencies: vec![ReferenceDependencyV1::ReferenceSource {
                reference_source_id: "reference_b".into(),
            }],
        };
        let sources = BTreeMap::from([("reference_a", &source_a), ("reference_b", &source_b)]);
        let mut closure = Closure::default();
        walk_reference(
            "reference_a",
            &sources,
            &catalog,
            ClosureRole::Reference,
            &mut BTreeSet::new(),
            &mut closure,
        )
        .expect("reference closure");
        assert!(closure.artifact_ids.contains(&id('a')));
        assert!(closure.source_sha256s.contains(&"b".repeat(64)));
        assert!(closure.source_sha256s.contains(&"c".repeat(64)));
    }

    #[test]
    fn phase_e_incomplete_reference_dependency_is_unknown_separation() {
        let source = ReferenceSourceAuthorityV1 {
            reference_source_id: "reference".into(),
            source_file_sha256: "d".repeat(64),
            evidence_origin: crate::validation_config::EvidenceOriginV1::Synthetic,
            dependency_completeness: ReferenceDependencyCompletenessV1::Unknown,
            experiment_scope: ArtifactExperimentScope::Single {
                experiment_id: ExperimentId::new("reference_experiment").expect("experiment"),
            },
            acquisition_families: ArtifactAcquisitionFamilies::Known(vec![
                AcquisitionFamilyId::new("reference_family").expect("family"),
            ]),
            direct_dependencies: Vec::new(),
        };
        let sources = BTreeMap::from([("reference", &source)]);
        let mut closure = Closure::default();
        walk_reference(
            "reference",
            &sources,
            &ArtifactLineageCatalog::default(),
            ClosureRole::Reference,
            &mut BTreeSet::new(),
            &mut closure,
        )
        .expect("reference closure");
        assert!(
            closure
                .unknown
                .contains(&SeparationUnknownReasonV1::ReferenceDependencyIncomplete)
        );
    }

    #[test]
    fn phase_e_hidden_duplicate_source_key_ignores_path_and_record_id() {
        let source = |path: &str| ArtifactSourceExpectationV1 {
            relative_path: path.into(),
            expected_artifact_kind: crate::domain::ArtifactKind::MechanismAnalysis,
            expected_schema_version: 4,
            source_file_sha256: "f".repeat(64),
            expected_lineage: ExpectedLineageV1::Known {
                artifact_id: id('f'),
                semantic_sha256: "f".repeat(64),
            },
        };
        assert_eq!(
            scientific_source_key(&source("first.json")),
            scientific_source_key(&source("renamed.json"))
        );
    }

    #[test]
    fn phase_e_complete_disjoint_closures_are_known_separated() {
        let mut left = Closure::default();
        left.artifact_ids.insert(id('a'));
        left.experiment_ids.insert("experiment_a".into());
        left.family_ids.insert("family_a".into());
        let mut right = Closure::default();
        right.artifact_ids.insert(id('b'));
        right.experiment_ids.insert("experiment_b".into());
        right.family_ids.insert("family_b".into());
        assert_eq!(
            classify_pair(&left, &right, BTreeSet::new()).status,
            SeparationStatusV1::KnownSeparated
        );
    }
}
