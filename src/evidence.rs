//! Conservative A1 evidence contracts and producer-backed covariance.
//!
//! Evidence is an outer adapter concern.  These types intentionally preserve
//! missing, unavailable, and unresolved provenance rather than interpreting
//! raw scientific values as mechanism or health conclusions.

pub use crate::domain::EvidenceIndependence;
use crate::domain::{
    ArtifactId, ArtifactKind, ArtifactLineageCatalog, ExperimentId, LineageResolutionStatus,
    ResolvedAcquisitionFamilies, ScopeKey, resolve_known_artifact_id,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, EvidenceBundleError> {
                let value = value.into();
                if value.is_empty() {
                    return Err(EvidenceBundleError::EmptyIdentifier);
                }
                Ok(Self(value))
            }
        }
    };
}

string_id!(EvidenceId);
string_id!(HypothesisId);
string_id!(EvidenceRequirementId);
string_id!(HealthFindingId);
string_id!(RequirementId);
string_id!(ComponentId);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LegacySourceFingerprint(pub String);

impl LegacySourceFingerprint {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        )
    }
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceBundleError> {
        let value = value.into();
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(EvidenceBundleError::InvalidLegacyFingerprint);
        }
        Ok(Self(value))
    }
}

pub type HealthDimension = crate::results::HealthDomain;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceTarget {
    MechanismHypothesis(HypothesisId),
    HealthFinding(HealthFindingId),
    HealthDimension(HealthDimension),
    IdentifiabilityRequirement(RequirementId),
    ModelComponent(ComponentId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceArtifactSource {
    Known {
        artifact_id: ArtifactId,
        artifact_kind: ArtifactKind,
    },
    LegacyUnknown {
        artifact_kind: ArtifactKind,
        source_fingerprint: LegacySourceFingerprint,
    },
}

impl EvidenceArtifactSource {
    fn sort_key(&self) -> (u8, String, String) {
        match self {
            Self::Known {
                artifact_id,
                artifact_kind,
            } => (0, artifact_kind.as_str().into(), artifact_id.0.clone()),
            Self::LegacyUnknown {
                artifact_kind,
                source_fingerprint,
            } => (
                1,
                artifact_kind.as_str().into(),
                source_fingerprint.0.clone(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSourceRef {
    pub artifact: EvidenceArtifactSource,
    pub field_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceSourceClass {
    Observed,
    ModelDerived,
    ProducerAssessment,
    ExternalReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceScopeDerivation {
    ArtifactScope,
    MemberRecord {
        experiment_id: ExperimentId,
        source_field_path: String,
    },
}

/// A producer-owned source path.  Its constructor is private so aggregate
/// membership and an arbitrary caller string cannot prove member scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceFieldPath(String);

impl EvidenceFieldPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Proof that a selected public record actually carries an experiment ID.
/// The only current constructor is the calibration-observation adapter route.
pub struct SelectedExperimentRecord<'a, T> {
    record: &'a T,
    experiment_id: ExperimentId,
    source_field_path: EvidenceFieldPath,
}

impl<'a> SelectedExperimentRecord<'a, crate::results::CalibrationObservation> {
    pub fn calibration_observation(
        record: &'a crate::results::CalibrationObservation,
        index: usize,
    ) -> Result<Self, EvidenceBundleError> {
        let experiment_id = ExperimentId::new(record.experiment_id.clone())
            .map_err(|_| EvidenceBundleError::ScopeRecordMissingExperimentId)?;
        Ok(Self {
            record,
            experiment_id,
            source_field_path: EvidenceFieldPath(format!("$.observations[{index}].experiment_id")),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceExperimentScope {
    Single {
        experiment_id: ExperimentId,
        derivation: EvidenceScopeDerivation,
    },
    Aggregate {
        aggregate_scope_id: crate::domain::AggregateExperimentScopeId,
        member_experiment_ids: Vec<ExperimentId>,
    },
    Unknown,
}

impl EvidenceExperimentScope {
    pub fn from_artifact_scope(scope: &crate::domain::ArtifactExperimentScope) -> Self {
        match scope {
            crate::domain::ArtifactExperimentScope::Single { experiment_id } => Self::Single {
                experiment_id: experiment_id.clone(),
                derivation: EvidenceScopeDerivation::ArtifactScope,
            },
            crate::domain::ArtifactExperimentScope::Aggregate {
                aggregate_scope_id,
                member_experiment_ids,
            } => Self::Aggregate {
                aggregate_scope_id: aggregate_scope_id.clone(),
                member_experiment_ids: member_experiment_ids.clone(),
            },
            crate::domain::ArtifactExperimentScope::Unknown => Self::Unknown,
        }
    }

    pub fn narrow_selected_record<T>(
        &self,
        selected: SelectedExperimentRecord<'_, T>,
    ) -> Result<Self, EvidenceBundleError> {
        let _record = selected.record;
        match self {
            Self::Aggregate {
                member_experiment_ids,
                ..
            } if member_experiment_ids.contains(&selected.experiment_id) => Ok(Self::Single {
                experiment_id: selected.experiment_id.clone(),
                derivation: EvidenceScopeDerivation::MemberRecord {
                    experiment_id: selected.experiment_id,
                    source_field_path: selected.source_field_path.0,
                },
            }),
            Self::Aggregate { .. } => Err(EvidenceBundleError::ScopeMemberRecordMismatch),
            _ => Err(EvidenceBundleError::ScopeCannotBeNarrowed),
        }
    }

    pub fn is_aggregate(&self) -> bool {
        matches!(self, Self::Aggregate { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceDirection {
    Supports,
    Contradicts,
    Neutral,
    NotApplicable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceAvailability {
    Available,
    Missing,
    NotApplicable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceStrength {
    NotAssessed,
    Weak,
    Moderate,
    Strong,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceValidity {
    Valid,
    OutsideDomain,
    Invalid,
    NotAssessed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvidenceUncertaintyModel {
    None,
    ExplicitLogInterval {
        lower_ln_tau_s: f64,
        upper_ln_tau_s: f64,
        confidence_level: f64,
    },
    LogNormal {
        variance_ln_tau_s: f64,
    },
    DeltaMethodTauVariance {
        variance_tau_s2: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceQuantity {
    pub value: f64,
    pub unit: String,
    pub uncertainty: Option<EvidenceUncertaintyModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceUnitDimension {
    Dimensionless,
    Time,
    TimeSquared,
    Potential,
    PotentialSquared,
    Temperature,
    Concentration,
    Conductivity,
    Impedance,
    OtherApproved,
}

/// Validates the project-approved UCUM vocabulary used by A1 artifacts.
/// Existing potentiometry units are delegated to `QuantityUnit`; EIS's
/// declared parameter grammar is retained as an explicit producer contract.
pub fn validate_ucum_unit(unit: &str) -> Result<EvidenceUnitDimension, EvidenceBundleError> {
    if unit.is_empty() {
        return Err(EvidenceBundleError::InvalidUnitSyntax { unit: unit.into() });
    }
    let exact = match unit {
        "1" | "dimensionless" | "dimensionless^2" => Some(EvidenceUnitDimension::Dimensionless),
        "s" => Some(EvidenceUnitDimension::Time),
        "s^2" => Some(EvidenceUnitDimension::TimeSquared),
        "V" | "mV" | "µV" => Some(EvidenceUnitDimension::Potential),
        "V^2" => Some(EvidenceUnitDimension::PotentialSquared),
        "K" | "°C" | "degC" => Some(EvidenceUnitDimension::Temperature),
        "Ohm" => Some(EvidenceUnitDimension::Impedance),
        "F" | "H" | "Hz" | "V/s" | "V/decade" | "Ohm s^-1/2" | "Ohm^-1 s^alpha"
        | "H s^(alpha-1)" | "Ohm s^alpha" | "Ohm^-1 s^gamma" => {
            Some(EvidenceUnitDimension::OtherApproved)
        }
        _ => None,
    };
    if let Some(dimension) = exact {
        return Ok(dimension);
    }
    match unit.parse::<crate::potentiometry::units::QuantityUnit>() {
        Ok(parsed) => Ok(match parsed.dimension() {
            crate::potentiometry::units::QuantityDimension::Concentration => {
                EvidenceUnitDimension::Concentration
            }
            crate::potentiometry::units::QuantityDimension::Activity => {
                EvidenceUnitDimension::Dimensionless
            }
            crate::potentiometry::units::QuantityDimension::Potential => {
                EvidenceUnitDimension::Potential
            }
            crate::potentiometry::units::QuantityDimension::Temperature => {
                EvidenceUnitDimension::Temperature
            }
            crate::potentiometry::units::QuantityDimension::Conductivity => {
                EvidenceUnitDimension::Conductivity
            }
        }),
        Err(_) if unit.bytes().any(|byte| byte.is_ascii_whitespace()) => {
            Err(EvidenceBundleError::InvalidUnitSyntax { unit: unit.into() })
        }
        Err(_) => Err(EvidenceBundleError::UnknownUnit { unit: unit.into() }),
    }
}

fn require_unit_dimension(
    unit: &str,
    expected: EvidenceUnitDimension,
) -> Result<(), EvidenceBundleError> {
    let actual = validate_ucum_unit(unit)?;
    if actual == expected {
        Ok(())
    } else {
        Err(EvidenceBundleError::UnitDimensionMismatch {
            unit: unit.into(),
            expected: format!("{expected:?}"),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrengthSource {
    NotAssessed,
    PreservedProducerAssessment,
    MechanismAssessor,
    HealthAssessor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EvidencePairKey {
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
}

impl EvidencePairKey {
    pub fn canonical(left: EvidenceId, right: EvidenceId) -> Result<Self, EvidenceBundleError> {
        if left == right {
            return Err(EvidenceBundleError::SelfIndependenceComparison);
        }
        if left < right {
            Ok(Self {
                left_evidence_id: left,
                right_evidence_id: right,
            })
        } else {
            Ok(Self {
                left_evidence_id: right,
                right_evidence_id: left,
            })
        }
    }
    pub fn is_canonical(&self) -> bool {
        self.left_evidence_id < self.right_evidence_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimescaleCovarianceUse {
    ProducerBacked { pair: EvidencePairKey },
    IndependenceBasedZeroCovariance { pair: EvidencePairKey },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrengthDerivation {
    pub algorithm_id: String,
    pub algorithm_version: String,
    pub source_evidence: Vec<EvidenceRef>,
    pub metric_values: BTreeMap<String, f64>,
    pub timescale_covariance_use: Option<TimescaleCovarianceUse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdSource {
    UserConfiguration,
    ValidatedDomain,
    ProducerContract,
    PublishedReference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThresholdProvenance {
    pub threshold_id: String,
    pub source: ThresholdSource,
    pub value: f64,
    pub unit: String,
    pub configuration_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub evidence_id: EvidenceId,
    pub target: EvidenceTarget,
    pub source: EvidenceSourceRef,
    pub experiment_scope: EvidenceExperimentScope,
    pub source_class: EvidenceSourceClass,
    pub direction: EvidenceDirection,
    pub availability: EvidenceAvailability,
    pub strength: EvidenceStrength,
    pub validity: EvidenceValidity,
    pub quantity: Option<EvidenceQuantity>,
    pub strength_source: StrengthSource,
    pub strength_derivation: Option<StrengthDerivation>,
    pub threshold_provenance: Vec<ThresholdProvenance>,
    pub lineage_artifact_ids: Vec<ArtifactId>,
    pub warnings: Vec<String>,
}

impl EvidenceRecord {
    fn canonicalize(&mut self) {
        self.lineage_artifact_ids.sort();
        self.lineage_artifact_ids.dedup();
        self.warnings.sort();
        self.warnings.dedup();
        self.threshold_provenance
            .sort_by(|left, right| left.threshold_id.cmp(&right.threshold_id));
        if let Some(derivation) = &mut self.strength_derivation {
            derivation
                .source_evidence
                .sort_by(|left, right| left.evidence_id.cmp(&right.evidence_id));
            derivation
                .source_evidence
                .dedup_by(|left, right| left.evidence_id == right.evidence_id);
            derivation.metric_values = derivation.metric_values.clone().into_iter().collect();
        }
    }
    pub fn validate(&self) -> Result<(), EvidenceBundleError> {
        if self.evidence_id.0.is_empty() || self.source.field_path.is_empty() {
            return Err(EvidenceBundleError::EmptyIdentifier);
        }
        match self.availability {
            EvidenceAvailability::Missing => {
                if self.strength != EvidenceStrength::NotAssessed
                    || self.quantity.is_some()
                    || self.direction != EvidenceDirection::Neutral
                    || self.validity != EvidenceValidity::NotAssessed
                {
                    return Err(EvidenceBundleError::MissingEvidenceCombination);
                }
            }
            EvidenceAvailability::NotApplicable => {
                if self.direction != EvidenceDirection::NotApplicable
                    || self.strength != EvidenceStrength::NotAssessed
                    || self.quantity.is_some()
                    || self.validity != EvidenceValidity::NotAssessed
                {
                    return Err(EvidenceBundleError::NotApplicableEvidenceCombination);
                }
            }
            EvidenceAvailability::Available => {
                if self.direction == EvidenceDirection::NotApplicable {
                    return Err(EvidenceBundleError::MissingEvidenceCombination);
                }
                if let Some(quantity) = &self.quantity {
                    validate_quantity(quantity)?;
                }
            }
        }
        if self.strength == EvidenceStrength::NotAssessed {
            if self.strength_source != StrengthSource::NotAssessed
                || self.strength_derivation.is_some()
            {
                return Err(EvidenceBundleError::AssessedStrengthMissingSource);
            }
        } else {
            if self.strength_source == StrengthSource::NotAssessed {
                return Err(EvidenceBundleError::AssessedStrengthMissingSource);
            }
            let Some(derivation) = &self.strength_derivation else {
                return Err(EvidenceBundleError::AssessedStrengthMissingDerivation);
            };
            if derivation.algorithm_id.is_empty()
                || derivation.algorithm_version.is_empty()
                || derivation.source_evidence.is_empty()
            {
                return Err(EvidenceBundleError::AssessedStrengthMissingDerivation);
            }
            if derivation
                .metric_values
                .values()
                .any(|value| !value.is_finite())
            {
                return Err(EvidenceBundleError::NonFiniteEvidenceValue);
            }
        }
        for threshold in &self.threshold_provenance {
            if threshold.threshold_id.is_empty() || !threshold.value.is_finite() {
                return Err(EvidenceBundleError::NonFiniteEvidenceValue);
            }
            validate_ucum_unit(&threshold.unit)?;
        }
        if self
            .lineage_artifact_ids
            .windows(2)
            .any(|window| window[0] >= window[1])
        {
            return Err(EvidenceBundleError::InvalidEvidenceReference);
        }
        Ok(())
    }
}

fn validate_quantity(quantity: &EvidenceQuantity) -> Result<(), EvidenceBundleError> {
    if !quantity.value.is_finite() {
        return Err(EvidenceBundleError::NonFiniteEvidenceValue);
    }
    validate_ucum_unit(&quantity.unit)?;
    if let Some(model) = &quantity.uncertainty {
        match model {
            EvidenceUncertaintyModel::None => {}
            EvidenceUncertaintyModel::ExplicitLogInterval {
                lower_ln_tau_s,
                upper_ln_tau_s,
                confidence_level,
            } => {
                if !lower_ln_tau_s.is_finite()
                    || !upper_ln_tau_s.is_finite()
                    || lower_ln_tau_s > upper_ln_tau_s
                    || !confidence_level.is_finite()
                    || !(0.5..1.0).contains(confidence_level)
                {
                    return Err(EvidenceBundleError::NonFiniteEvidenceValue);
                }
            }
            EvidenceUncertaintyModel::LogNormal { variance_ln_tau_s } => {
                if !variance_ln_tau_s.is_finite() || *variance_ln_tau_s < 0.0 {
                    return Err(EvidenceBundleError::NonFiniteEvidenceValue);
                }
            }
            EvidenceUncertaintyModel::DeltaMethodTauVariance { variance_tau_s2 } => {
                if !variance_tau_s2.is_finite() || *variance_tau_s2 < 0.0 {
                    return Err(EvidenceBundleError::NonFiniteEvidenceValue);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceIndependenceReason {
    SameSourceArtifact,
    SharedAncestor,
    SharedAcquisitionFamily,
    IncompleteLineage,
    UnknownAcquisitionFamily,
    MissingAcquisitionFamily,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceIndependenceAssessment {
    pub pair: EvidencePairKey,
    pub classification: EvidenceIndependence,
    pub algorithm_id: String,
    pub left_lineage_status: LineageResolutionStatus,
    pub right_lineage_status: LineageResolutionStatus,
    pub shared_ancestor_artifact_ids: Vec<ArtifactId>,
    pub shared_acquisition_families: Vec<crate::domain::AcquisitionFamilyId>,
    pub reasons: Vec<EvidenceIndependenceReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CovarianceAxisId(pub String);

impl CovarianceAxisId {
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceBundleError> {
        let value = value.into();
        if value.is_empty() {
            return Err(EvidenceBundleError::EmptyIdentifier);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CovarianceQuantityKind {
    Parameter,
    State,
    DerivedQuantity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CovarianceAxis {
    pub axis_id: CovarianceAxisId,
    pub source_field_path: String,
    pub quantity_kind: CovarianceQuantityKind,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LabeledCovarianceMatrix {
    pub axes: Vec<CovarianceAxis>,
    pub values: Vec<Vec<f64>>,
}

#[derive(Deserialize)]
struct LabeledCovarianceMatrixWire {
    axes: Vec<CovarianceAxis>,
    values: Vec<Vec<f64>>,
}

impl<'de> Deserialize<'de> for LabeledCovarianceMatrix {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = LabeledCovarianceMatrixWire::deserialize(deserializer)?;
        Self::new(wire.axes, wire.values).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CovarianceAxisValidationError {
    #[error("EIS parameter axis cardinality mismatch")]
    EisParameterAxisCardinalityMismatch,
    #[error("duplicate covariance axis ID")]
    DuplicateCovarianceAxisId,
    #[error("unknown EIS parameter key")]
    UnknownEisParameterKey,
    #[error("covariance matrix is not square")]
    NotSquare,
    #[error("covariance matrix contains a non-finite value")]
    NonFinite,
    #[error("covariance axis is missing a unit")]
    MissingUnit,
    #[error("covariance axis has an invalid unit: {0}")]
    InvalidUnit(String),
    #[error("covariance matrix is not symmetric")]
    NotSymmetric,
}

impl LabeledCovarianceMatrix {
    pub fn new(
        axes: Vec<CovarianceAxis>,
        values: Vec<Vec<f64>>,
    ) -> Result<Self, CovarianceAxisValidationError> {
        if values.len() != axes.len() || values.iter().any(|row| row.len() != axes.len()) {
            return Err(CovarianceAxisValidationError::NotSquare);
        }
        let mut ids = BTreeSet::new();
        for axis in &axes {
            if axis.axis_id.0.is_empty() || axis.source_field_path.is_empty() {
                return Err(CovarianceAxisValidationError::MissingUnit);
            }
            validate_ucum_unit(&axis.unit)
                .map_err(|_| CovarianceAxisValidationError::InvalidUnit(axis.unit.clone()))?;
            if !ids.insert(axis.axis_id.clone()) {
                return Err(CovarianceAxisValidationError::DuplicateCovarianceAxisId);
            }
        }
        if values.iter().flatten().any(|value| !value.is_finite()) {
            return Err(CovarianceAxisValidationError::NonFinite);
        }
        for (row_index, row) in values.iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                if (*value - values[column_index][row_index]).abs() > 1e-10 {
                    return Err(CovarianceAxisValidationError::NotSymmetric);
                }
            }
        }
        Ok(Self { axes, values })
    }
    pub fn lookup_exact(&self, axis_id: &CovarianceAxisId) -> Option<&CovarianceAxis> {
        self.axes.iter().find(|axis| &axis.axis_id == axis_id)
    }
    pub fn value_exact(&self, left: &CovarianceAxisId, right: &CovarianceAxisId) -> Option<f64> {
        let left = self.axes.iter().position(|axis| &axis.axis_id == left)?;
        let right = self.axes.iter().position(|axis| &axis.axis_id == right)?;
        self.values.get(left)?.get(right).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EisParameterKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EisParameterIdentity {
    pub element_instance_id: String,
    pub parameter_key: EisParameterKey,
}

impl EisParameterIdentity {
    pub fn from_descriptor(
        element_instance_id: impl Into<String>,
        parameter_name: &str,
    ) -> Result<Self, CovarianceAxisValidationError> {
        let element_instance_id = element_instance_id.into();
        if element_instance_id.is_empty() || element_instance_id.contains(':') {
            return Err(CovarianceAxisValidationError::UnknownEisParameterKey);
        }
        let split = element_instance_id
            .find(|character: char| character.is_ascii_digit())
            .ok_or(CovarianceAxisValidationError::UnknownEisParameterKey)?;
        if split == 0
            || !element_instance_id[split..]
                .bytes()
                .all(|byte| byte.is_ascii_digit())
        {
            return Err(CovarianceAxisValidationError::UnknownEisParameterKey);
        }
        let element_type = &element_instance_id[..split];
        let allowed = match element_type {
            "R" => &["R"][..],
            "C" => &["C"][..],
            "L" => &["L"][..],
            "W" => &["sigma"][..],
            "CPE" => &["Q", "alpha"][..],
            "Wo" | "Ws" => &["Z0", "tau"][..],
            "La" => &["L", "alpha"][..],
            "Gw" => &["sigma", "alpha"][..],
            "G" => &["R_G", "t_G"][..],
            "Gs" => &["R_G", "t_G", "phi"][..],
            "K" => &["R", "tau_k"][..],
            "Zarc" => &["R", "tau_k", "gamma"][..],
            "TLMQ" => &["Rion", "Qs", "gamma"][..],
            "T" => &["A", "B", "a", "b"][..],
            _ => return Err(CovarianceAxisValidationError::UnknownEisParameterKey),
        };
        if !allowed.contains(&parameter_name) {
            return Err(CovarianceAxisValidationError::UnknownEisParameterKey);
        }
        let parameter_key = match parameter_name {
            "R" => "r",
            "C" => "c",
            "L" => "l",
            "sigma" => "sigma",
            "Q" => "q",
            "alpha" => "alpha",
            "Z0" => "z0",
            "tau" => "tau",
            "R_G" => "r_g",
            "t_G" => "t_g",
            "phi" => "phi",
            "tau_k" => "tau_k",
            "gamma" => "gamma",
            "Rion" => "r_ion",
            "Qs" => "q_s",
            "A" => "a_upper",
            "B" => "b_upper",
            "a" => "a_lower",
            "b" => "b_lower",
            _ => return Err(CovarianceAxisValidationError::UnknownEisParameterKey),
        };
        Ok(Self {
            element_instance_id,
            parameter_key: EisParameterKey(parameter_key.into()),
        })
    }
    pub fn axis_id(&self) -> CovarianceAxisId {
        CovarianceAxisId(format!(
            "eis.parameter:{}:{}",
            self.element_instance_id, self.parameter_key.0
        ))
    }
}

pub fn labeled_eis_covariance(
    descriptors: &[(String, String, String)],
    values: Vec<Vec<f64>>,
) -> Result<LabeledCovarianceMatrix, CovarianceAxisValidationError> {
    if descriptors.len() != values.len() || values.iter().any(|row| row.len() != descriptors.len())
    {
        return Err(CovarianceAxisValidationError::EisParameterAxisCardinalityMismatch);
    }
    let mut axes = Vec::with_capacity(descriptors.len());
    for (element, parameter, unit) in descriptors {
        let identity = EisParameterIdentity::from_descriptor(element, parameter)?;
        axes.push(CovarianceAxis {
            axis_id: identity.axis_id(),
            source_field_path: format!("parameters[{element}:{parameter}]"),
            quantity_kind: CovarianceQuantityKind::Parameter,
            unit: if unit.is_empty() {
                "dimensionless".into()
            } else {
                unit.clone()
            },
        });
    }
    LabeledCovarianceMatrix::new(axes, values)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimescaleCrossCovariance {
    LogSpace { covariance_ln_tau: f64 },
    TauSpace { covariance_tau_s2: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairCovarianceDerivation {
    PreservedProducerCovariance,
    ExtractedCovarianceMatrixEntry,
    UnitConvertedProducerCovariance,
    DeltaMethodDerivedCovariance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimescalePairUncertaintySource {
    pub source_artifact: EvidenceArtifactSource,
    pub left_source_field_path: String,
    pub right_source_field_path: String,
    pub covariance_source_field_path: String,
    pub derivation: PairCovarianceDerivation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimescalePairUncertainty {
    pub pair: EvidencePairKey,
    pub covariance: TimescaleCrossCovariance,
    pub source: TimescalePairUncertaintySource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimescaleDerivedQuantityDefinition {
    pub derived_axis_id: CovarianceAxisId,
    pub algorithm_id: String,
    pub source_axis_ids: Vec<CovarianceAxisId>,
    pub output_unit: String,
    pub jacobian: TimescaleJacobianDefinition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimescaleJacobianDefinition {
    pub source_axis_ids: Vec<CovarianceAxisId>,
    pub coefficients: Vec<f64>,
    pub units: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TimescaleTransformRegistry {
    algorithm_ids: BTreeSet<String>,
}

impl TimescaleTransformRegistry {
    pub fn register(&mut self, algorithm_id: impl Into<String>) -> Result<(), EvidenceBundleError> {
        let algorithm_id = algorithm_id.into();
        if algorithm_id.is_empty() {
            return Err(EvidenceBundleError::EmptyIdentifier);
        }
        self.algorithm_ids.insert(algorithm_id);
        Ok(())
    }
    pub fn contains(&self, algorithm_id: &str) -> bool {
        self.algorithm_ids.contains(algorithm_id)
    }
}

/// Route 1: exact labeled producer covariance. No positional or nearest-axis
/// fallback is permitted.
pub fn extract_direct_covariance(
    matrix: &LabeledCovarianceMatrix,
    left: &CovarianceAxisId,
    right: &CovarianceAxisId,
) -> Result<f64, EvidenceBundleError> {
    matrix
        .value_exact(left, right)
        .ok_or(EvidenceBundleError::InvalidTimescaleCovarianceSource)
}

/// Route 2: an explicitly registered analytic delta method. Finite
/// differences and universal EIS transformations are intentionally absent.
pub fn analytic_delta_method_covariance(
    registry: &TimescaleTransformRegistry,
    algorithm_id: &str,
    matrix: &LabeledCovarianceMatrix,
    left: &TimescaleJacobianDefinition,
    right: &TimescaleJacobianDefinition,
) -> Result<TimescaleCrossCovariance, EvidenceBundleError> {
    if !registry.contains(algorithm_id)
        || left.source_axis_ids.len() != left.coefficients.len()
        || right.source_axis_ids.len() != right.coefficients.len()
        || left.units.len() != left.coefficients.len()
        || right.units.len() != right.coefficients.len()
        || left.units.iter().any(String::is_empty)
        || right.units.iter().any(String::is_empty)
        || left
            .coefficients
            .iter()
            .chain(right.coefficients.iter())
            .any(|value| !value.is_finite())
    {
        return Err(EvidenceBundleError::InvalidTimescaleCovarianceSource);
    }
    for axis in left
        .source_axis_ids
        .iter()
        .chain(right.source_axis_ids.iter())
    {
        if matrix.lookup_exact(axis).is_none() {
            return Err(EvidenceBundleError::InvalidTimescaleCovarianceSource);
        }
    }
    let mut covariance = 0.0;
    for (left_index, left_axis) in left.source_axis_ids.iter().enumerate() {
        for (right_index, right_axis) in right.source_axis_ids.iter().enumerate() {
            covariance += left.coefficients[left_index]
                * matrix
                    .value_exact(left_axis, right_axis)
                    .ok_or(EvidenceBundleError::InvalidTimescaleCovarianceSource)?
                * right.coefficients[right_index];
        }
    }
    if !covariance.is_finite() {
        return Err(EvidenceBundleError::NonFiniteEvidenceValue);
    }
    Ok(TimescaleCrossCovariance::TauSpace {
        covariance_tau_s2: covariance,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvidenceBundle {
    pub schema_version: u32,
    pub experiment_scope: EvidenceExperimentScope,
    pub sensor_scope: ScopeKey,
    pub channel_scope: ScopeKey,
    pub records: Vec<EvidenceRecord>,
    pub independence_assessments: Vec<EvidenceIndependenceAssessment>,
    pub timescale_pair_uncertainties: Vec<TimescalePairUncertainty>,
    pub lineage_catalog: ArtifactLineageCatalog,
    pub warnings: Vec<String>,
}

#[derive(Deserialize)]
struct EvidenceBundleWire {
    schema_version: u32,
    experiment_scope: EvidenceExperimentScope,
    sensor_scope: ScopeKey,
    channel_scope: ScopeKey,
    records: Vec<EvidenceRecord>,
    independence_assessments: Vec<EvidenceIndependenceAssessment>,
    timescale_pair_uncertainties: Vec<TimescalePairUncertainty>,
    lineage_catalog: ArtifactLineageCatalog,
    warnings: Vec<String>,
}

impl<'de> Deserialize<'de> for EvidenceBundle {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = EvidenceBundleWire::deserialize(deserializer)?;
        let mut builder = EvidenceBundleBuilder::new(
            wire.experiment_scope,
            wire.sensor_scope,
            wire.channel_scope,
            wire.lineage_catalog,
        );
        builder.schema_version = wire.schema_version;
        for record in wire.records {
            record.validate().map_err(serde::de::Error::custom)?;
            builder.add_record(record);
        }
        for assessment in wire.independence_assessments {
            if !assessment.pair.is_canonical() {
                return Err(serde::de::Error::custom(
                    EvidenceBundleError::NonCanonicalEvidencePair,
                ));
            }
            builder.add_independence_assessment(assessment);
        }
        for uncertainty in wire.timescale_pair_uncertainties {
            if !uncertainty.pair.is_canonical() {
                return Err(serde::de::Error::custom(
                    EvidenceBundleError::NonCanonicalTimescalePair,
                ));
            }
            builder.add_timescale_pair_uncertainty(uncertainty);
        }
        for warning in wire.warnings {
            builder.warning(warning);
        }
        builder.build().map_err(serde::de::Error::custom)
    }
}

impl EvidenceBundle {
    pub fn lookup_independence(
        &self,
        pair: &EvidencePairKey,
    ) -> Option<&EvidenceIndependenceAssessment> {
        self.independence_assessments
            .iter()
            .find(|item| &item.pair == pair)
    }
    pub fn lookup_timescale_pair_uncertainty(
        &self,
        pair: &EvidencePairKey,
    ) -> Option<&TimescalePairUncertainty> {
        self.timescale_pair_uncertainties
            .iter()
            .find(|item| &item.pair == pair)
    }
    pub fn semantic_hash(&self) -> Result<String, EvidenceBundleError> {
        self.validate()?;
        let canonical = serde_jcs::to_vec(self)
            .map_err(|error| EvidenceBundleError::Serialization(error.to_string()))?;
        Ok(Sha256::digest(&canonical)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect())
    }
    pub fn validate(&self) -> Result<(), EvidenceBundleError> {
        let mut builder = EvidenceBundleBuilder::new(
            self.experiment_scope.clone(),
            self.sensor_scope.clone(),
            self.channel_scope.clone(),
            self.lineage_catalog.clone(),
        );
        for record in self.records.clone() {
            builder.add_record(record);
        }
        for assessment in self.independence_assessments.clone() {
            builder.add_independence_assessment(assessment);
        }
        for uncertainty in self.timescale_pair_uncertainties.clone() {
            builder.add_timescale_pair_uncertainty(uncertainty);
        }
        for warning in &self.warnings {
            builder.warning(warning.clone());
        }
        builder.build().map(|_| ())
    }
}

pub struct EvidenceBundleBuilder {
    schema_version: u32,
    experiment_scope: EvidenceExperimentScope,
    sensor_scope: ScopeKey,
    channel_scope: ScopeKey,
    records: Vec<EvidenceRecord>,
    independence_assessments: Vec<EvidenceIndependenceAssessment>,
    timescale_pair_uncertainties: Vec<TimescalePairUncertainty>,
    lineage_catalog: ArtifactLineageCatalog,
    warnings: Vec<String>,
}

impl EvidenceBundleBuilder {
    pub fn new(
        experiment_scope: EvidenceExperimentScope,
        sensor_scope: ScopeKey,
        channel_scope: ScopeKey,
        lineage_catalog: ArtifactLineageCatalog,
    ) -> Self {
        Self {
            schema_version: 1,
            experiment_scope,
            sensor_scope,
            channel_scope,
            records: Vec::new(),
            independence_assessments: Vec::new(),
            timescale_pair_uncertainties: Vec::new(),
            lineage_catalog,
            warnings: Vec::new(),
        }
    }
    pub fn add_record(&mut self, mut record: EvidenceRecord) {
        record.canonicalize();
        self.records.push(record);
    }
    pub fn add_independence_assessment(&mut self, mut assessment: EvidenceIndependenceAssessment) {
        if assessment.pair.left_evidence_id > assessment.pair.right_evidence_id {
            std::mem::swap(
                &mut assessment.pair.left_evidence_id,
                &mut assessment.pair.right_evidence_id,
            );
            std::mem::swap(
                &mut assessment.left_lineage_status,
                &mut assessment.right_lineage_status,
            );
        }
        self.independence_assessments.push(assessment);
    }
    pub fn add_timescale_pair_uncertainty(&mut self, mut uncertainty: TimescalePairUncertainty) {
        if uncertainty.pair.left_evidence_id > uncertainty.pair.right_evidence_id {
            std::mem::swap(
                &mut uncertainty.pair.left_evidence_id,
                &mut uncertainty.pair.right_evidence_id,
            );
            std::mem::swap(
                &mut uncertainty.source.left_source_field_path,
                &mut uncertainty.source.right_source_field_path,
            );
        }
        self.timescale_pair_uncertainties.push(uncertainty);
    }
    pub fn warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
    pub(crate) fn lineage_catalog(&self) -> &ArtifactLineageCatalog {
        &self.lineage_catalog
    }
    pub fn build(mut self) -> Result<EvidenceBundle, EvidenceBundleError> {
        self.records.sort_by(record_order);
        let mut ids = BTreeSet::new();
        for record in &self.records {
            record.validate()?;
            if !ids.insert(record.evidence_id.clone()) {
                return Err(EvidenceBundleError::DuplicateEvidenceId);
            }
        }
        let index = self
            .records
            .iter()
            .map(|record| (record.evidence_id.clone(), record))
            .collect::<BTreeMap<_, _>>();
        self.independence_assessments
            .sort_by(|left, right| pair_order(&left.pair, &right.pair));
        let mut pairs = BTreeSet::new();
        for assessment in &self.independence_assessments {
            if !assessment.pair.is_canonical() {
                return Err(EvidenceBundleError::NonCanonicalEvidencePair);
            }
            if !pairs.insert(assessment.pair.clone()) {
                return Err(EvidenceBundleError::DuplicateEvidencePair);
            }
            let left = index
                .get(&assessment.pair.left_evidence_id)
                .ok_or(EvidenceBundleError::UnknownEvidenceReference)?;
            let right = index
                .get(&assessment.pair.right_evidence_id)
                .ok_or(EvidenceBundleError::UnknownEvidenceReference)?;
            let expected = classify_independence(left, right, &self.lineage_catalog);
            if assessment.classification != expected.classification
                || assessment.left_lineage_status != expected.left_lineage_status
                || assessment.right_lineage_status != expected.right_lineage_status
                || assessment.shared_ancestor_artifact_ids != expected.shared_ancestor_artifact_ids
                || assessment.shared_acquisition_families != expected.shared_acquisition_families
                || assessment.reasons != expected.reasons
            {
                return Err(EvidenceBundleError::EvidenceIndependenceMismatch);
            }
        }
        self.timescale_pair_uncertainties
            .sort_by(|left, right| pair_order(&left.pair, &right.pair));
        let mut covariance_pairs = BTreeSet::new();
        for uncertainty in &self.timescale_pair_uncertainties {
            if !uncertainty.pair.is_canonical() {
                return Err(EvidenceBundleError::NonCanonicalTimescalePair);
            }
            if !covariance_pairs.insert(uncertainty.pair.clone()) {
                return Err(EvidenceBundleError::DuplicateTimescalePairUncertainty);
            }
            let left = index
                .get(&uncertainty.pair.left_evidence_id)
                .ok_or(EvidenceBundleError::UnknownTimescaleEvidenceReference)?;
            let right = index
                .get(&uncertainty.pair.right_evidence_id)
                .ok_or(EvidenceBundleError::UnknownTimescaleEvidenceReference)?;
            validate_timescale_record(left)?;
            validate_timescale_record(right)?;
            validate_covariance(&uncertainty.covariance)?;
            if matches!(
                uncertainty.source.source_artifact,
                EvidenceArtifactSource::LegacyUnknown { .. }
            ) {
                return Err(EvidenceBundleError::InvalidTimescaleCovarianceSource);
            }
            if let EvidenceArtifactSource::Known { artifact_id, .. } =
                &uncertainty.source.source_artifact
                && !self.lineage_catalog.artifacts.contains_key(artifact_id)
                && !matches!(&left.source.artifact, EvidenceArtifactSource::Known { artifact_id: left_id, .. } if left_id == artifact_id)
                && !matches!(&right.source.artifact, EvidenceArtifactSource::Known { artifact_id: right_id, .. } if right_id == artifact_id)
            {
                return Err(EvidenceBundleError::InvalidTimescaleCovarianceSource);
            }
        }
        self.warnings.sort();
        self.warnings.dedup();
        Ok(EvidenceBundle {
            schema_version: self.schema_version,
            experiment_scope: self.experiment_scope,
            sensor_scope: self.sensor_scope,
            channel_scope: self.channel_scope,
            records: self.records,
            independence_assessments: self.independence_assessments,
            timescale_pair_uncertainties: self.timescale_pair_uncertainties,
            lineage_catalog: self.lineage_catalog,
            warnings: self.warnings,
        })
    }
}

fn validate_timescale_record(record: &EvidenceRecord) -> Result<(), EvidenceBundleError> {
    if record.availability != EvidenceAvailability::Available
        || record.quantity.as_ref().is_none_or(|quantity| {
            require_unit_dimension(&quantity.unit, EvidenceUnitDimension::Time).is_err()
                || !quantity.value.is_finite()
                || quantity.value <= 0.0
        })
    {
        return Err(EvidenceBundleError::InvalidTimescaleCovarianceSource);
    }
    Ok(())
}
fn validate_covariance(covariance: &TimescaleCrossCovariance) -> Result<(), EvidenceBundleError> {
    match covariance {
        TimescaleCrossCovariance::LogSpace { covariance_ln_tau }
            if covariance_ln_tau.is_finite() =>
        {
            Ok(())
        }
        TimescaleCrossCovariance::TauSpace { covariance_tau_s2 }
            if covariance_tau_s2.is_finite() =>
        {
            Ok(())
        }
        _ => Err(EvidenceBundleError::NonFiniteEvidenceValue),
    }
}

pub struct ClassifiedIndependence {
    pub classification: EvidenceIndependence,
    pub left_lineage_status: LineageResolutionStatus,
    pub right_lineage_status: LineageResolutionStatus,
    pub shared_ancestor_artifact_ids: Vec<ArtifactId>,
    pub shared_acquisition_families: Vec<crate::domain::AcquisitionFamilyId>,
    pub reasons: Vec<EvidenceIndependenceReason>,
}

pub fn classify_independence(
    left: &EvidenceRecord,
    right: &EvidenceRecord,
    catalog: &ArtifactLineageCatalog,
) -> ClassifiedIndependence {
    let same_source = match (&left.source.artifact, &right.source.artifact) {
        (
            EvidenceArtifactSource::Known {
                artifact_id: left_id,
                ..
            },
            EvidenceArtifactSource::Known {
                artifact_id: right_id,
                ..
            },
        ) => left_id == right_id,
        _ => false,
    };
    let left_resolved = source_lineage(&left.source.artifact, catalog);
    let right_resolved = source_lineage(&right.source.artifact, catalog);
    let shared_ancestors = left_resolved
        .ancestor_artifact_ids
        .iter()
        .filter(|id| right_resolved.ancestor_artifact_ids.contains(id))
        .cloned()
        .collect::<Vec<_>>();
    let left_families = families(&left_resolved.acquisition_families);
    let right_families = families(&right_resolved.acquisition_families);
    let shared_families = left_families
        .intersection(&right_families)
        .cloned()
        .collect::<Vec<_>>();
    let mut reasons = Vec::new();
    let classification = if same_source {
        reasons.push(EvidenceIndependenceReason::SameSourceArtifact);
        EvidenceIndependence::SameSource
    } else if matches!(
        left.source.artifact,
        EvidenceArtifactSource::LegacyUnknown { .. }
    ) || matches!(
        right.source.artifact,
        EvidenceArtifactSource::LegacyUnknown { .. }
    ) || left_resolved.status != LineageResolutionStatus::Complete
        || right_resolved.status != LineageResolutionStatus::Complete
    {
        reasons.push(EvidenceIndependenceReason::IncompleteLineage);
        EvidenceIndependence::Unknown
    } else if matches!(
        left_resolved.acquisition_families,
        ResolvedAcquisitionFamilies::Unknown
    ) || matches!(
        right_resolved.acquisition_families,
        ResolvedAcquisitionFamilies::Unknown
    ) {
        reasons.push(EvidenceIndependenceReason::UnknownAcquisitionFamily);
        EvidenceIndependence::Unknown
    } else if left_families.is_empty() || right_families.is_empty() {
        reasons.push(EvidenceIndependenceReason::MissingAcquisitionFamily);
        EvidenceIndependence::Unknown
    } else if !shared_ancestors.is_empty() {
        reasons.push(EvidenceIndependenceReason::SharedAncestor);
        EvidenceIndependence::PartiallyDependent
    } else if !shared_families.is_empty() {
        reasons.push(EvidenceIndependenceReason::SharedAcquisitionFamily);
        EvidenceIndependence::PartiallyDependent
    } else {
        EvidenceIndependence::Independent
    };
    ClassifiedIndependence {
        classification,
        left_lineage_status: left_resolved.status,
        right_lineage_status: right_resolved.status,
        shared_ancestor_artifact_ids: shared_ancestors,
        shared_acquisition_families: shared_families,
        reasons,
    }
}

fn source_lineage(
    source: &EvidenceArtifactSource,
    catalog: &ArtifactLineageCatalog,
) -> crate::domain::ResolvedArtifactLineage {
    match source {
        EvidenceArtifactSource::Known { artifact_id, .. } => {
            resolve_known_artifact_id(artifact_id, catalog)
        }
        EvidenceArtifactSource::LegacyUnknown { .. } => crate::domain::ResolvedArtifactLineage {
            status: LineageResolutionStatus::Incomplete,
            root_artifact_id: None,
            ancestor_artifact_ids: Vec::new(),
            missing_artifact_ids: Vec::new(),
            acquisition_families: ResolvedAcquisitionFamilies::Unknown,
            reasons: vec![crate::domain::LineageResolutionReason::LegacyUnknownRoot],
        },
    }
}
fn families(value: &ResolvedAcquisitionFamilies) -> BTreeSet<crate::domain::AcquisitionFamilyId> {
    match value {
        ResolvedAcquisitionFamilies::Known(values) => values.iter().cloned().collect(),
        ResolvedAcquisitionFamilies::Unknown => BTreeSet::new(),
    }
}

pub fn largest_independent_subset<'a>(
    records: &'a [EvidenceRecord],
    bundle: &EvidenceBundle,
) -> Vec<&'a EvidenceRecord> {
    let mut eligible = records
        .iter()
        .filter(|record| {
            record.availability == EvidenceAvailability::Available
                && record.direction == EvidenceDirection::Supports
                && record.validity == EvidenceValidity::Valid
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.evidence_id.cmp(&right.evidence_id));
    let n = eligible.len();
    for cardinality in (1..=n).rev() {
        let mut selected = Vec::with_capacity(cardinality);
        if let Some(indices) =
            find_independent_combination(&eligible, bundle, cardinality, 0, &mut selected)
        {
            return indices.into_iter().map(|index| eligible[index]).collect();
        }
    }
    Vec::new()
}

fn find_independent_combination(
    eligible: &[&EvidenceRecord],
    bundle: &EvidenceBundle,
    desired_len: usize,
    next_index: usize,
    selected: &mut Vec<usize>,
) -> Option<Vec<usize>> {
    if selected.len() == desired_len {
        return Some(selected.clone());
    }
    let remaining = desired_len - selected.len();
    if eligible.len().saturating_sub(next_index) < remaining {
        return None;
    }
    let last_start = eligible.len() - remaining;
    for index in next_index..=last_start {
        let independent_of_selected = selected.iter().all(|selected_index| {
            EvidencePairKey::canonical(
                eligible[index].evidence_id.clone(),
                eligible[*selected_index].evidence_id.clone(),
            )
            .ok()
            .and_then(|pair| bundle.lookup_independence(&pair))
            .is_some_and(|assessment| {
                assessment.classification == EvidenceIndependence::Independent
            })
        });
        if !independent_of_selected {
            continue;
        }
        selected.push(index);
        if let Some(result) =
            find_independent_combination(eligible, bundle, desired_len, index + 1, selected)
        {
            return Some(result);
        }
        selected.pop();
    }
    None
}

fn record_order(left: &EvidenceRecord, right: &EvidenceRecord) -> std::cmp::Ordering {
    target_key(&left.target)
        .cmp(&target_key(&right.target))
        .then_with(|| {
            left.source
                .artifact
                .sort_key()
                .cmp(&right.source.artifact.sort_key())
        })
        .then_with(|| left.source.field_path.cmp(&right.source.field_path))
        .then_with(|| left.evidence_id.cmp(&right.evidence_id))
}
fn target_key(target: &EvidenceTarget) -> (u8, String) {
    match target {
        EvidenceTarget::MechanismHypothesis(id) => (0, id.0.clone()),
        EvidenceTarget::HealthFinding(id) => (1, id.0.clone()),
        EvidenceTarget::HealthDimension(id) => (2, format!("{id:?}")),
        EvidenceTarget::IdentifiabilityRequirement(id) => (3, id.0.clone()),
        EvidenceTarget::ModelComponent(id) => (4, id.0.clone()),
    }
}
fn pair_order(left: &EvidencePairKey, right: &EvidencePairKey) -> std::cmp::Ordering {
    left.left_evidence_id
        .cmp(&right.left_evidence_id)
        .then_with(|| left.right_evidence_id.cmp(&right.right_evidence_id))
}
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EvidenceBundleError {
    #[error("missing evidence combination")]
    MissingEvidenceCombination,
    #[error("not-applicable evidence combination")]
    NotApplicableEvidenceCombination,
    #[error("assessed strength has no source")]
    AssessedStrengthMissingSource,
    #[error("assessed strength has no derivation")]
    AssessedStrengthMissingDerivation,
    #[error("invalid evidence reference")]
    InvalidEvidenceReference,
    #[error("quantity availability conflict")]
    QuantityAvailabilityConflict,
    #[error("non-finite evidence value")]
    NonFiniteEvidenceValue,
    #[error("duplicate evidence ID")]
    DuplicateEvidenceId,
    #[error("unknown evidence reference")]
    UnknownEvidenceReference,
    #[error("self independence comparison")]
    SelfIndependenceComparison,
    #[error("non-canonical evidence pair")]
    NonCanonicalEvidencePair,
    #[error("duplicate evidence pair")]
    DuplicateEvidencePair,
    #[error("evidence independence mismatch")]
    EvidenceIndependenceMismatch,
    #[error("duplicate timescale pair uncertainty")]
    DuplicateTimescalePairUncertainty,
    #[error("non-canonical timescale pair")]
    NonCanonicalTimescalePair,
    #[error("unknown timescale evidence reference")]
    UnknownTimescaleEvidenceReference,
    #[error("invalid timescale covariance source")]
    InvalidTimescaleCovarianceSource,
    #[error("timescale covariance units mismatch")]
    TimescaleCovarianceUnitMismatch,
    #[error("invalid UCUM unit syntax: {unit}")]
    InvalidUnitSyntax { unit: String },
    #[error("unknown UCUM unit: {unit}")]
    UnknownUnit { unit: String },
    #[error("unit '{unit}' is incompatible with {expected}")]
    UnitDimensionMismatch { unit: String, expected: String },
    #[error("covariance unit mismatch: {unit}")]
    CovarianceUnitMismatch { unit: String },
    #[error("empty identifier")]
    EmptyIdentifier,
    #[error("invalid legacy source fingerprint")]
    InvalidLegacyFingerprint,
    #[error("scope cannot be narrowed without an explicit member record")]
    ScopeCannotBeNarrowed,
    #[error("selected record's experiment ID is not a member of the aggregate scope")]
    ScopeMemberRecordMismatch,
    #[error("selected record has no trustworthy experiment ID")]
    ScopeRecordMissingExperimentId,
    #[error("serialization failed: {0}")]
    Serialization(String),
}
