# Model-Based Mechanism and Sensor Health Integration V1 — Final Implementation Contract

**Status:** planning/specification only. This document authorizes no production Rust-code change.
**Repository:** `/Users/xingyuwang/ProjectOngoing/rust_electroanalysis_cli`
**Planning branch / base commit:** `plan/ism-mechanism-health-v1` / `83dbb4bf271e26e8819b48de02f911dc1cc75351`.

This is the normative contract for Model-Based Mechanism and Sensor Health Integration V1 (MHI V1). A normative word is binding. A missing required input or configuration never authorizes an implementation default. Existing public behavior remains unless this contract explicitly defines an additive change.

Frozen contracts: SAR-009 phase split; SAR-010 typed health interpretation; SAR-011 residual signs. No default assigns a fast mode to double layer, a slow mode to adsorption/fouling/water layer, a transduction candidate to proven solid contact, a reference offset to reference-electrode failure, or a residual to sensor failure. Timescale agreement alone is not causal proof. Same-source evidence is not independent confirmation. Transitional, Disturbed, and Indeterminate data are not steady-state evidence unless the exact policy below admits it.

## 1. Repository reconciliation and baseline

`CODE_QUALITY_WORKFLOW.md` was requested but is absent from the checkout and its parent (`rg --files -g CODE_QUALITY_WORKFLOW.md . ..` returns no file). The procedures explicitly required by this contract and the user request govern this work instead.

| Finding ID | Original severity | Current classification | Repository evidence | Root cause | Required plan correction | Plan section modified |
|---|---:|---|---|---|---|---|
| F1 / SAR-001 | P1 | CONFIRMED | `src/domain/provenance.rs` records paths/hashes/timestamp, while `src/results/{estimation,model,transient,calibration,eis}.rs` contain no common durable dependency graph. | Run provenance was used as lineage. | Durable identity, catalog, closure and independence contracts. | §§3–4 |
| F2 / SAR-002 | P1 | CONFIRMED | Existing `HealthEvidence` combines domain, prose and strength; no serialized orthogonal MHI evidence record exists. | Evidence axes and assessor authority are conflated. | Complete evidence schema and validator. | §5 |
| F3 / SAR-003 | P1 | CONFIRMED | `src/model/component.rs` immutable `InterpretationStatus` is distinct from current prose mechanism hypothesis assessment. | Component meaning was confused with current hypothesis support. | Recomputed lifecycle, history and exact gates. | §6 |
| F4 / SAR-004 | P1 | CONFIRMED | Artifact scope/time fields differ; `src/runners/mechanism.rs` has no formal temporal join. | No shared identity, clock or matching policy. | Scope, clocks, joins, fractions and policy. | §7 |
| F5 / SAR-005 | P1 | CONFIRMED | `src/mechanism/timescale.rs` has heuristic comparison/warnings, not an uncertainty-to-strength rule. | Positive-domain uncertainty and resolution were not normative. | Exact log interval and strength algorithm. | §8 |
| F6 / SAR-006 | P1 | CONFIRMED | `src/model/identifiability.rs` has ten closed enum variants and `IdentifiabilityReport::not_assessed`; `src/estimation/observability.rs` assesses filter state, not requirements. | No forward-compatible representation or assessor map. | String serde and complete assessor table. | §9 |
| F7 / SAR-007 | P1 | CONFIRMED | `MechanismCompareCommand` accepts EIS/transient/calibration only; `HealthAssessCommand` accepts legacy inputs; existing negatives are not all CLI fixture paths. | Runtime inputs and behavior were unspecified. | Exact additive flags, order and E2E tests. | §10 |
| F8 / SAR-009/A0 | P1 | CONFIRMED | `src/results/artifact_contracts.rs` lists current schemas in `LEGACY_SCHEMA_VERSIONS` for current-2 artifacts; `validate_value` consequently accepts a schema-2 missing kind. | Current and readable versions were not separated. | A0 validation, kind matrix and producers. | §11 |
| F9 / MHI-R14 | P1 | CONFIRMED | `src/results/health.rs::HealthDomain` has seven legacy domains, not nine MHI dimensions. | R14 named but did not enumerate dimensions. | Exact nine-dimension contract. | §12 |
| F10 / plan tracking | P2 | CONFIRMED | The plan was untracked at inspection (`?? docs/...plan.md`). | Prior review inspected a non-Git artifact. | Stage, verify and commit this plan. | §18 |

### Baseline classification

The following commands must be re-run immediately before an implementation phase starts and in its implementation report:

```bash
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
cargo build --locked --release
```

Observed on the inspected base: `cargo fmt --all --check` fails only on formatting in `src/health/rules.rs` (**existing unrelated baseline**); clippy fails on unused `write_artifact` in `src/runners/fit.rs` and dead `read_json`/`write_json` helpers in calibration, health (two), mechanism, and signal runners (**six existing unrelated baseline warnings**); `cargo test --locked --all` fails only `phase2_transient::transient_cli_creates_machine_and_human_outputs`, because schema-2 `TransientAnalysisReport` is rejected by its current-1 contract (**A0-related**); `cargo build --locked --release` succeeds, retaining those six unrelated warnings. No new regression was introduced by the documentation change. A0-related failures are only failures asserting current-schema/expected-kind behavior for its eight listed producers. Any other failure is baseline-unrelated unless its first failing stack frame is an A0 file changed by A0. No A0 change may repair unrelated formatting or clippy debt.

## 2. Phase boundary and ownership

| Phase | Scope | Exit criterion | Explicit non-goal |
|---|---|---|---|
| A0 | Artifact-contract repair only. | §11 tests pass for all affected kinds. | Lineage, evidence, hypothesis, health, CLI evidence flags. |
| A1 | Durable lineage, evidence schema/adapters, joins and identifiability representation/assessment. | §§3–5, 7, 9 tests pass. | Mechanism/health conclusion promotion. |
| B | Mechanism evidence integration. | §§6, 8, 10 mechanism paths pass. | Health causal conclusion. |
| C | Sensor-health evidence integration. | §12 and health CLI paths pass. | Rendering calculation. |
| D | Reporting/plotting/public scientific output. | Renderers project serialized assessments only. | New scientific assessment rules. |
| E | Full compatibility/scientific validation. | §15 matrix and independent re-review GO. | Reopening frozen semantics. |

Each phase has separate review, commit and rollback. A0 only changes `src/results/artifact_contracts.rs`, A0 tests, and A0 fixtures, except a minimum typed serde migration required to read a documented schema-1 fixture. A1 begins only after an A0 commit.

## 3. Durable lineage contract (A1)

### 3.1 Normative types and serialization

Newtypes serialize as their inner strings. Existing repository types are: `ArtifactKind` at `src/domain/artifact.rs`, `ExperimentId` = new A1 string newtype, and all existing result payload types at `src/results/*.rs`.

```rust
pub struct ArtifactId(pub String); // exactly "sha256:" + 64 lowercase hex
pub struct ExperimentId(pub String); // nonempty UTF-8 identifier
pub enum ScopeKey { Specific(String), All, Unspecified }
pub enum AcquisitionFamilyId { Known(String), Unknown }
pub struct ArtifactScope {
    pub experiment_id: ExperimentId,
    pub sensor: ScopeKey,
    pub channel: ScopeKey,
}
pub struct ArtifactIdentity {
    pub artifact_id: ArtifactId,
    pub artifact_kind: ArtifactKind,
    pub schema_version: u32,
    pub producer_version: String,
    pub experiment_id: ExperimentId,
    pub sensor_scope: ScopeKey,
    pub channel_scope: ScopeKey,
    pub acquisition_family_ids: Vec<AcquisitionFamilyId>,
    pub semantic_sha256: String,
}
pub enum ArtifactDependencyRole {
    Initialization, Calibration, Prior, Constraint, TransformationInput,
    AuxiliaryInput, ValidationInput, DerivedFrom,
}
pub struct ArtifactDependency {
    pub artifact_id: ArtifactId,
    pub artifact_kind: ArtifactKind,
    pub role: ArtifactDependencyRole,
}
pub struct ArtifactLineageNode {
    pub identity: ArtifactIdentity,
    pub direct_dependencies: Vec<ArtifactDependency>,
}
pub struct ArtifactLineageCatalog {
    pub schema_version: u32,
    pub artifacts: BTreeMap<ArtifactId, ArtifactLineageNode>,
}
pub enum LineageResolutionStatus { Complete, Incomplete, CycleDetected, RootMissing }
pub struct LineageResolution {
    pub root_artifact_id: ArtifactId,
    pub ancestor_artifact_ids: Vec<ArtifactId>,
    pub acquisition_family_ids: Vec<AcquisitionFamilyId>,
    pub missing_artifact_ids: Vec<ArtifactId>,
    pub status: LineageResolutionStatus,
}
pub enum EvidenceIndependence { Independent, PartiallyDependent, SameSource, Unknown }
```

`Specific` contains a nonempty string. `All` means deliberately broad scope. `Unspecified` means unavailable, never a wildcard. `Known` contains a nonempty string. `Unknown` is serialized literally as `"unknown"`, is retained after deserialization, and is never independent. An acquisition family is the original independent acquisition campaign, experiment, specimen, sensor exposure, or controlled acquisition source from which an artifact derives. It is not a file, artifact ID, algorithm run, or inferred value. Producers inherit the sorted unique union from every direct dependency and append their independently acquired raw-family identity; if that identity is unavailable they append `Unknown`.

`acquisition_family_ids` are sorted `Known` bytewise ascending, then one `Unknown`; duplicate values and duplicate Unknowns are validation errors. Direct dependencies sort by role discriminant, then `artifact_kind.as_str()`, then artifact ID bytes. Artifact catalogs serialize `BTreeMap` key order. All listed vectors are serialized in their stated order.

### 3.2 Closure resolver

`resolve_lineage(root, catalog)` is the sole resolver for transitive closure. It performs deterministic depth-first traversal over sorted dependencies, with `Visiting` and `Visited` sets keyed by `ArtifactId`.

1. If root is absent, return `RootMissing`, empty ancestors/families, and `missing_artifact_ids=[root]`.
2. Mark root Visiting. For every sorted direct dependency, add its ID to ancestors. If absent, add it to missing IDs and continue. If Visiting, retain all accumulated ancestors and mark a cycle. If unvisited, recurse.
3. On every present node, union its `acquisition_family_ids`; retain `Unknown`; then mark Visited.
4. Sort/deduplicate ancestors, families and missing IDs using §3.1 ordering.
5. Return `CycleDetected` if any back edge; otherwise `Incomplete` if any missing ID; otherwise `Complete`.

The resolver neither drops an absent ancestor nor turns missing metadata into a family. A caller receives all known ancestors/families even when status is not Complete.

`classify_independence(a_source_id, b_source_id, catalog)` first resolves both source identities. It returns `SameSource` for equal source IDs. It returns `Unknown` if either source is missing, either result is not Complete, either family set contains Unknown, or required identity is absent. Otherwise it returns `PartiallyDependent` if ancestor closures intersect or known family sets intersect; it returns `Independent` only for distinct source IDs, complete disjoint ancestor closures, and nonempty disjoint known-family sets. An empty known family set is Unknown. Only `Independent` is independent confirmation; every other value is NotIndependent.

Permanent A1 production-path test MHI-T05d: serialize transient artifact → use it as estimation initialization or prior → serialize `StateEstimationReport` → mechanism evidence adapter reads both plus the catalog. The later transient evidence resolves SameSource or PartiallyDependent, never Independent; JSON and human outputs state the relationship.

### 3.3 Canonical semantic identity and hash ownership

`ArtifactId = "sha256:" + SHA256(canonical_semantic_bytes)` and `semantic_sha256` is the same lowercase hex without prefix. Canonical bytes are UTF-8 RFC 8785 canonical JSON for a named, owned hash-view struct. Reject non-finite numbers before producing bytes. Include kind, schema version, scientific payload, scope, sorted family IDs, scientifically meaningful producer algorithm/config identity, and sorted `(role, dependency ID)` pairs. Exclude artifact ID, semantic hash, absolute paths, output directory, generated timestamp, human text, and formatting. A dependency is represented by role + ID only; it is not embedded recursively.

| Artifact kind | Hash-view owner | Included payload | Excluded / dependency treatment | Ordering |
|---|---|---|---|---|
| `eis_fit` | `src/results/eis.rs` | circuit, source/fitted/residual data, diagnostics, fit config | provenance paths/timestamp; direct dependencies by role+ID | arrays input order; dependencies §3.1 |
| `transient_analysis` | `src/results/transient.rs` | experiment/channel, events/fits/config | provenance paths/timestamp; direct dependencies | event index order; dependencies §3.1 |
| `calibration_observations` | `src/results/calibration.rs` | observations, units, steady-state/context | provenance paths/timestamp; direct dependencies | observation ID bytes |
| `calibration_model` | `src/results/calibration.rs` | parameters/domain/training/config | provenance paths/timestamp; direct dependencies | parameter ID bytes |
| `calibration_analysis` | `src/results/calibration.rs` | model results/validation/config | provenance paths/timestamp; direct dependencies | candidate kind bytes |
| `signal_analysis` | `src/results/signal.rs` | scientific features/config/identity | provenance paths/timestamp; direct dependencies | feature name bytes |
| `health_baseline` | `src/results/health.rs` | baseline context, distributions, records/config | provenance paths/timestamp; direct dependencies | feature then record ID |
| `state_estimation` | `src/results/estimation.rs` | state definitions, initialization, estimates, covariance, config | provenance paths/timestamp; every consumed artifact direct dependency | timestamps increasing; dependencies §3.1 |
| `ism_model_compilation` | `src/results/model.rs` | definition, validity, declarative requirements | generated location; direct dependencies | model canonical order; dependencies §3.1 |
| `ism_model_analysis` | `src/results/model.rs` | definition, points, equilibrium | generated location; direct dependencies | point time increasing; dependencies §3.1 |
| `mechanism_analysis` | `src/results/mechanism.rs` | evidence bundle, assessment/config | prose report/timestamp; all consumed artifacts direct dependencies | evidence §5.4 |
| `health_assessment` | `src/results/health.rs` | evidence bundle, dimensions/findings/config | prose report/timestamp; all consumed artifacts direct dependencies | dimension enum then finding ID |
| `health_trend` | `src/results/health.rs` | trend inputs/results/config | provenance paths/timestamp; direct dependencies | feature then point record ID |

Schema/version is included for every row. Producer modules own their named view and no alternate hasher may create an ID.

## 4. A1 artifact evolution and compatibility

Every listed producer adds `identity: ArtifactIdentity` and `direct_dependencies: Vec<ArtifactDependency>` in A1, wrapped in a new `ArtifactLineageNode` at catalog emission. New fields are additive. Existing artifacts with no fields yield `Unknown` lineage, never inferred from provenance paths. A1 schema contracts are exact: transient, all three calibration artifacts, signal, EIS, and health baseline current 3 / legacy `[1,2]`; state estimation current 4 / legacy `[1,2,3]`; model compilation and analysis current 5 / legacy `[1,2,3,4]`; mechanism analysis, health assessment, and health trend current 3 / legacy `[1,2]`. Current schema is never listed as legacy. A1 tests prove each legacy read → Unknown lineage and reject unknown schemas.

## 5. Serialized evidence contract (A1)

```rust
pub struct EvidenceId(pub String); // nonempty, stable within a bundle
pub struct HypothesisId(pub String);
pub struct HealthFindingId(pub String);
pub struct RequirementId(pub String);
pub struct ComponentId(pub String);
pub enum EvidenceTarget {
    MechanismHypothesis(HypothesisId), HealthFinding(HealthFindingId),
    HealthDimension(HealthDimension), IdentifiabilityRequirement(RequirementId),
    ModelComponent(ComponentId),
}
pub struct EvidenceSourceRef { pub artifact_id: ArtifactId, pub artifact_kind: ArtifactKind, pub field_path: String }
pub enum EvidenceSourceClass { Observed, ModelDerived, ProducerAssessment, ExternalReference }
pub enum EvidenceDirection { Supports, Contradicts, Neutral, NotApplicable }
pub enum EvidenceAvailability { Available, Missing, NotApplicable }
pub enum EvidenceStrength { NotAssessed, Weak, Moderate, Strong }
pub enum EvidenceValidity { Valid, OutsideDomain, Invalid, NotAssessed }
pub struct EvidenceUncertainty { pub model: EvidenceUncertaintyModel, pub parameters: BTreeMap<String, f64> }
pub enum EvidenceUncertaintyModel { ExplicitLogInterval, LogNormal, DeltaMethodFromTauCovariance }
pub struct EvidenceQuantity { pub value: f64, pub unit: String, pub uncertainty: Option<EvidenceUncertainty> }
pub enum StrengthSource { NotAssessed, PreservedProducerAssessment, MechanismAssessor, HealthAssessor }
pub struct EvidenceRef { pub evidence_id: EvidenceId }
pub struct StrengthDerivation {
    pub algorithm_id: String, pub algorithm_version: String,
    pub source_evidence: Vec<EvidenceRef>, pub metric_values: BTreeMap<String, f64>,
}
pub enum ThresholdSource { UserConfiguration, ValidatedDomain, ProducerContract, PublishedReference }
pub struct ThresholdProvenance {
    pub threshold_id: String, pub source: ThresholdSource, pub value: f64,
    pub unit: String, pub configuration_hash: Option<String>,
}
pub struct EvidenceRecord {
    pub evidence_id: EvidenceId, pub target: EvidenceTarget, pub source: EvidenceSourceRef,
    pub source_class: EvidenceSourceClass, pub direction: EvidenceDirection,
    pub independence: EvidenceIndependence, pub availability: EvidenceAvailability,
    pub strength: EvidenceStrength, pub validity: EvidenceValidity,
    pub quantity: Option<EvidenceQuantity>, pub strength_source: StrengthSource,
    pub strength_derivation: Option<StrengthDerivation>,
    pub threshold_provenance: Vec<ThresholdProvenance>,
    pub lineage_artifact_ids: Vec<ArtifactId>, pub warnings: Vec<String>,
}
pub struct EvidenceBundle {
    pub schema_version: u32, pub experiment_id: ExperimentId,
    pub sensor_scope: ScopeKey, pub channel_scope: ScopeKey,
    pub records: Vec<EvidenceRecord>, pub lineage_catalog: ArtifactLineageCatalog,
    pub warnings: Vec<String>,
}
```

All IDs/paths/units are nonempty. Quantity values and derivation metrics are finite. Units are exact UCUM strings; a unit-bearing and a dimensionless quantity may not be compared unless the assessor explicitly defines conversion. `source_evidence`, threshold provenance, lineage IDs and warnings sort bytewise; empty `source_evidence` is invalid for an assessed strength. Evidence records sort by target discriminant + target ID, source kind, source ID, field path, evidence ID. Evidence references must resolve exactly once in the bundle.

### 5.1 Combination validator

The validator returns one typed error per invalid rule: `MissingEvidenceCombination`, `NotApplicableEvidenceCombination`, `AssessedStrengthMissingSource`, `AssessedStrengthMissingDerivation`, `IndependentLineageFailure`, `InvalidEvidenceReference`, `QuantityAvailabilityConflict`, `NonFiniteEvidenceValue`, or `DuplicateEvidenceId`.

| Condition | Required values | Failure |
|---|---|---|
| `availability=Missing` | `strength=NotAssessed`, `quantity=None`, `direction=Neutral`, `validity=NotAssessed` | `MissingEvidenceCombination` |
| `availability=NotApplicable` | `direction=NotApplicable`, `strength=NotAssessed`, `quantity=None`, `validity=NotAssessed` | `NotApplicableEvidenceCombination` |
| `availability=Available` | source resolves, direction is not `NotApplicable` | `MissingEvidenceCombination` |
| assessed strength | `strength_source != NotAssessed` and derivation exists | `AssessedStrengthMissingSource/Derivation` |
| `strength=NotAssessed` | source is `NotAssessed` and derivation is None | `AssessedStrengthMissingSource` |
| `independence=Independent` | both closures Complete, known nonempty disjoint families, classifier passes | `IndependentLineageFailure` |
| `validity=OutsideDomain or Invalid` | record is retained but excluded from promotion/counts | assessment exclusion |

`strength_source=PreservedProducerAssessment` requires an artifact field recording producer algorithm/version and threshold provenance. `MechanismAssessor` and `HealthAssessor` require a registered algorithm ID in this document. Missing configuration produces `NotAssessed`, never a strength. `Unknown`, `SameSource`, and `PartiallyDependent` remain auditable but only Independent counts as independent confirmation.

## 6. Deterministic hypothesis lifecycle (B)

```rust
pub enum HypothesisEvidenceLevel { Unassessed, Hypothesized, ExperimentallySupported, ValidatedForDomain }
pub enum InterpretationStatus { /* existing exact type: src/model/component.rs */ }
pub enum HypothesisReasonCode {
    Declared, MissingConfiguration, MissingRequiredEvidence, InsufficientIndependentFamilies,
    MissingNonTimescaleEvidence, TimescaleNotAssessed, AmplitudeNotAssessed,
    AmplitudeFailed, RepeatabilityNotAssessed, RepeatabilityFailed, CriticalContradiction,
    IdentifiabilityNotSatisfied, InvalidEvidence, OutsideDomain, LineageReclassified,
    UncertaintyUnavailable, ValidationProtocolMissing, ValidationFailed,
}
pub struct HypothesisAssessment {
    pub hypothesis_id: HypothesisId,
    pub component_interpretation_status: InterpretationStatus,
    pub hypothesis_evidence_level: HypothesisEvidenceLevel,
    pub supporting_evidence: Vec<EvidenceRef>, pub contradictory_evidence: Vec<EvidenceRef>,
    pub excluded_evidence: Vec<EvidenceRef>, pub reason_codes: Vec<HypothesisReasonCode>,
    pub assessed_at: Timestamp,
}
pub struct Timestamp(pub String); // RFC 3339 UTC instant
pub struct HypothesisAssessmentEvent {
    pub previous_level: HypothesisEvidenceLevel, pub new_level: HypothesisEvidenceLevel,
    pub reason_codes: Vec<HypothesisReasonCode>, pub evidence_bundle_hash: String,
    pub assessed_at: Timestamp,
}
pub struct ValidationDomain { pub domain_id: String, pub declared_applicability: String }
pub struct ValidationProtocol { pub protocol_id: String, pub acceptance_criteria: Vec<ValidationAcceptanceCriterion> }
pub struct ValidationAcceptanceCriterion { pub criterion_id: String, pub passed: bool, pub threshold: ThresholdProvenance }
```

`InterpretationStatus` is the existing immutable component field and is copied unchanged. The current assessment is recomputed from the current validated bundle, thus can increase or decrease. An append-only history event records each level change. A decrease is required when new valid contradiction, invalidation, domain invalidation, lineage reclassification, lost identifiability, lost required evidence, or updated uncertainty causes a gate failure.

For every declared hypothesis, configuration provides `critical_moderate_contradiction_count: usize >= 1`, `minimum_independent_supporting_families: usize >= 1`, and `minimum_non_timescale_supporting_families: usize >= 1`; all have `ThresholdProvenance`. Missing any is `Unassessed`. A critical contradiction is one Valid + Independent + Strong contradictory record, or at least `critical_moderate_contradiction_count` Valid + Independent + Moderate contradictory records from distinct acquisition families.

When amplitude is predicted, `relative_amplitude_error = abs(A1-A2)/max(abs(A1),abs(A2),amplitude_floor)`. `amplitude_floor > 0` has the amplitude unit; `max_relative_amplitude_error >= 0` is dimensionless. Both are required and provenance-bearing. Missing/invalid amplitudes or configuration return NotAssessed; error above maximum fails. For positive replicate tau values `x_i=ln(tau_i)` and `repeatability_sd = sqrt(sum((x_i-mean(x))^2)/(R-1))`. `minimum_repeatability_replicates >= 2` and `maximum_log_tau_repeatability_sd >= 0` are required. Fewer R returns NotAssessed; above maximum fails.

`Hypothesized` requires explicit declared hypothesis and at least one Valid available association. `ExperimentallySupported` requires all: declared hypothesis; at least configured independent supporting families; at least configured independent non-timescale supporting families; applicable timescale passes; predicted amplitude passes; repeatable dynamic-mode repeatability passes; no critical contradiction; every critical `src/model/identifiability.rs::RequirementSeverity::Required` requirement Satisfied; critical evidence Valid; required uncertainty available. Timescale alone never meets the non-timescale gate. `ValidatedForDomain` additionally requires ExperimentallySupported, `ValidationDomain`, nonempty `ValidationProtocol.protocol_id`, configured `minimum_validation_acquisition_families >= 1`, every `ValidationAcceptanceCriterion.passed=true`, and no critical contradiction. With no protocol it is forbidden.

## 7. Temporal/equilibrium joins (A1/B/C)

```rust
pub enum ClockBasis { ExperimentElapsedSeconds, AbsoluteUtcSeconds }
pub struct ClockConversionProvenance { pub source_artifact_id: ArtifactId, pub method_id: String, pub shared_experiment_start_ref: String }
pub struct ScopeManifestBinding { pub artifact_id: ArtifactId, pub sensor: Option<String>, pub channel: Option<String> }
pub struct ClockConversion { pub source: ClockBasis, pub target: ClockBasis, pub experiment_start_ref: String, pub conversion_provenance: ClockConversionProvenance }
pub enum JoinIdentityStatus { Resolved, Unresolved }
pub enum JoinOutcome { Joined, MissingEvidence, Ambiguous, Indeterminate }
pub enum MixedStatePolicy {
    RequireAllSteady { allow_quasi_equilibrium: bool },
    MinimumSteadyFraction { minimum_fraction: f64, allow_quasi_equilibrium: bool, reject_if_disturbed: bool },
    WorstCase,
}
pub struct TemporalJoinConfig {
    pub maximum_timestamp_difference_s: f64,
    pub minimum_classified_fraction: f64,
    pub mixed_state_policy: MixedStatePolicy,
    pub threshold_provenance: Vec<ThresholdProvenance>,
}
```

Cross-artifact joins require equal `experiment_id`. Specific/Specific scopes must equal; Specific/All and All/All are compatible. Either Unspecified requires two `ScopeManifestBinding` entries, one for each artifact ID, whose relevant concrete values are equal; without them identity is Unresolved. Sensor and channel apply the same rule. The result is MissingEvidence + Indeterminate and cannot produce strong evidence. Equilibrium values are the existing `src/model/equilibrium_recognition.rs::EquilibriumStatus` variants.

Normalize to elapsed seconds from experiment start. Absolute timestamps convert only through an identical shared experiment-start reference and serialized `ClockConversion`; otherwise clocks are unresolved. Target point `t` selects a source point minimizing `abs(t_source-t)` when at most `maximum_timestamp_difference_s`. An exact tie is Ambiguous → MissingEvidence. Window join is `[start_s,end_s)`. Event window is `[event_time-pre_event_s,event_time+post_event_s]` (both endpoints included); pre/post must be finite and nonnegative.

For a target window, `N_target` is expected target observations, `N_classified` is successfully joined target observations with Available equilibrium classification, `classified_fraction=N_classified/N_target`. If `N_target=0`, evidence is Missing. `N_equilibrium` counts Equilibrium; `N_quasi` counts QuasiEquilibrium. If `N_classified=0`, all fractions are unavailable, not zero. `equilibrium_fraction=N_equilibrium/N_classified`; `steady_state_fraction=(N_equilibrium+N_quasi)/N_classified`. `minimum_classified_fraction` and MinimumSteadyFraction threshold lie in `[0,1]` and are required/provenance-bearing.

Processing is exactly: (1) identity scope; (2) clock resolution; (3) point/window/event matching; (4) minimum classified fraction; (5) MixedStatePolicy; (6) scientific use. `RequireAllSteady` requires every classified state Equilibrium or, if allowed, Equilibrium/Quasi. `MinimumSteadyFraction` uses its allowed numerator, requires it ≥ minimum, and rejects any Disturbed if flag set. `WorstCase` returns precedence Indeterminate > Disturbed > Transitional > QuasiEquilibrium > Equilibrium. No hard-coded precedence may bypass the selected policy.

MHI-T11a–k exercise: perfect alignment, partial alignment, below minimum, equilibrium+quasi, equilibrium+transitional, disturbed, Indeterminate, clock mismatch, scope mismatch, missing experiment ID, and nearest-time tie. Each unresolved fixture asserts MissingEvidence + Indeterminate.

## 8. Timescale evidence algorithm (B)

```rust
pub struct TimescaleEvidenceConfig {
    pub confidence_level: f64,
    pub strong_max_log_distance: f64, pub moderate_max_log_distance: f64,
    pub weak_max_log_distance: f64, pub minimum_observation_duration_ratio: f64,
    pub minimum_samples_per_tau: f64, pub minimum_mode_separation_ratio: f64,
    pub threshold_provenance: Vec<ThresholdProvenance>,
}
```

All values are required: `0.5<confidence_level<1`; `0<=strong<=moderate<=weak`; all resolution thresholds `>0`. Each appears once in provenance with the named field and unit `1`. Missing/invalid configuration means strength NotAssessed.

For finite positive `tau_1,tau_2`, `r=ln(tau_1/tau_2)` and `d_tau=abs(r)`; otherwise no comparison. In the relevant window every `dt_i=t_i-t_(i-1)` must be finite positive, `effective_sampling_interval_s=max(dt_i)`, `samples_per_tau=tau/effective_sampling_interval_s`, and it passes at least minimum samples. `observation_duration_ratio=(t_last-t_first)/tau` passes at least its minimum. `mode_separation_ratio=max(tau_1,tau_2)/min(tau_1,tau_2)` passes at least its minimum.

Only `ExplicitLogInterval`, `LogNormal`, and `DeltaMethodFromTauCovariance` are supported. The first supplies ln(tau) bounds; LogNormal supplies ln(tau) variance; delta gives `Var[ln(tau)]≈Var[tau]/tau²` and `Cov[ln(tau1),ln(tau2)]≈Cov[tau1,tau2]/(tau1*tau2)`. Same-fit taus without cross covariance are NotAssessed. Zero cross covariance is permitted solely for proven Independent acquisition families and only when the decision records that lineage assumption. Other uncertainty is NotAssessed.

`Var[r]=Var[ln(tau1)]+Var[ln(tau2)]-2Cov[...]`; `sigma_r=sqrt(Var[r])`; `alpha=1-confidence_level`; `z=standard_normal_quantile(1-alpha/2)` using IEEE-754 double inverse standard normal CDF; `r_low=r-z*sigma_r`, `r_high=r+z*sigma_r`. If interval contains zero, `d_low=0`, `d_high=max(abs(r_low),abs(r_high))`; otherwise `d_low=min(abs(...))`, `d_high=max(abs(...))`. Classify by conservative `d_high`: Strong `<=strong`; Moderate `(strong,moderate]`; Weak `(moderate,weak]`; above weak produces Neutral + NotAssessed, never causal contradiction. Unavailable required uncertainty is NotAssessed without point-estimate fallback.

Current repository permitted EIS sources are direct fitted time parameters in `src/mechanism/timescale.rs` for `ElementType::{Wo,Ws,G,Gs,K,Zarc}` and derived `tau=R*C` / `tau_c=(R*Q)^(1/alpha)` in an explicitly modeled parallel R-C/R-CPE branch. No current serialized EIS feature kind declares approved single-relaxation frequency semantics; therefore frequency-to-tau conversion is NotApplicable in V1. It remains forbidden for Nyquist extrema, Bode extrema, DRT peaks and generic fitted frequencies until a future artifact adds approved feature metadata.

## 9. Identifiability contract (A1/B)

Replace the closed `src/model/identifiability.rs::IdentifiabilityRequirementKind` only in A1 with:

```rust
pub enum IdentifiabilityRequirementKind { Known(KnownIdentifiabilityRequirementKind), Custom(String) }
pub enum KnownIdentifiabilityRequirementKind {
    ActivityExcitation, TransientExcitation, ObservationDurationRelativeToTimescale,
    ModeSeparation, ReferenceAnchor, IndependentCovariateVariation,
    InterferentVariation, TemperatureVariation, RepeatedStandards, AuxiliaryObservation,
}
pub enum IdentifiabilityAssessmentStatus { Satisfied, NotSatisfied, NotAssessed, NotApplicable }
pub struct IdentifiabilityAssessment {
    pub requirement_id: RequirementId, pub kind: IdentifiabilityRequirementKind,
    pub assessor_id: String, pub algorithm_id: String, pub source_fields: Vec<String>,
    pub metrics: BTreeMap<String, f64>, pub thresholds: Vec<ThresholdProvenance>,
    pub status: IdentifiabilityAssessmentStatus, pub reasons: Vec<String>,
}
```

Serde serializes one string using the current known spellings; any nonempty unknown string deserializes to `Custom(original_string)`. Empty string is an error. Existing known strings round-trip byte-for-byte. Tests cover every known old string, an unknown string, reserialization, and empty rejection.

| Current kind | assessor / algorithm | source fields and required input | threshold/result/missing behavior |
|---|---|---|---|
| ActivityExcitation | `identifiability.not_assessed` / `not_assessed.v1` | `[]` | NotAssessed, `UnsupportedRequirementKind` |
| TransientExcitation | `identifiability.transient_excitation` / `transient_excitation.v1` | event ID/type/time, log10 activity step, pre/post point counts | required absolute log10 step, pre/post counts; passing eligible event Satisfied; missing NotAssessed; otherwise NotSatisfied |
| ObservationDurationRelativeToTimescale | `identifiability.observation_duration` / `timescale_resolution.v1` | time window, positive tau | §8 duration ratio; missing NotAssessed; below NotSatisfied |
| ModeSeparation | `identifiability.mode_separation` / `timescale_resolution.v1` | two positive taus | §8 separation ratio; missing NotAssessed; below NotSatisfied |
| ReferenceAnchor | `identifiability.reference_anchor` / `reference_anchor.v1` | known-standard/reference-control/approved anchor artifact through lineage | valid declared domain + complete lineage + anchor present Satisfied; unknown lineage/missing input NotAssessed; invalid/out-of-domain NotSatisfied |
| IndependentCovariateVariation | `identifiability.independent_covariate` / `covariate_variation.v1` | aligned `(u_i,target_activity_i)` | N, range, Pearson limit below; missing NotAssessed; zero variance or gate fail NotSatisfied |
| InterferentVariation | `identifiability.interferent_variation` / `interferent_variation.v1` | positive interferent activities | log10 N/range below; missing NotAssessed; gate fail NotSatisfied |
| TemperatureVariation | `identifiability.not_assessed` / `not_assessed.v1` | `[]` | NotAssessed, `UnsupportedRequirementKind` |
| RepeatedStandards | `identifiability.not_assessed` / `not_assessed.v1` | `[]` | NotAssessed, `UnsupportedRequirementKind` |
| AuxiliaryObservation | `identifiability.not_assessed` / `not_assessed.v1` | `[]` | NotAssessed, `UnsupportedRequirementKind` |

Covariate requires finite aligned data, `N>=minimum_covariate_samples`, `max(u)-min(u)>=minimum_covariate_range`, and `abs(Pearson(u,target_activity))<=maximum_absolute_pearson_correlation`. Zero variance is NotSatisfied. Interferent uses `log10(activity)`, `N>=minimum_interferent_samples`, and `max-min>=minimum_interferent_log10_range`. Transient requires an explicit event, `abs(log10(post/pre))>=minimum_absolute_log10_activity_step`, and counts at least configured pre/post points. All named values are required configuration/provenance; sample counts are integers ≥1, ranges/steps >0, correlation `[0,1]`. A Custom uses `identifiability.custom.not_assessed` / `not_assessed.v1`, NotAssessed, unless a future registered custom assessor is explicitly added. It never auto-satisfies.

## 10. Exact CLI production interfaces (B/C)

Preserve current flags. Add to `mechanism compare`: `--estimation-artifact <PATH>`, `--model-artifact <PATH>`, `--lineage-catalog <PATH>`. Add to `health assess`: `--estimation-artifact <PATH>`, `--model-artifact <PATH>`, `--mechanism-artifact <PATH>`, `--lineage-catalog <PATH>`. These are optional. Absent model/estimation yields Missing model-derived evidence; absent catalog yields Unknown independence; neither is fabricated. Existing `--mechanism-results` remains accepted; `--mechanism-artifact` is its additive explicit artifact alias only if the paths resolve to the same artifact identity, otherwise `ConflictingEvidenceInput`.

Both runners execute exactly: (1) parse CLI; (2) load configuration; (3) load legacy required inputs; (4) load optional model/estimation/mechanism; (5) validate artifact contracts; (6) validate experiment/sensor/channel scope; (7) load/resolve catalog if supplied; (8) build normalized EvidenceBundle; (9) assess; (10) serialize typed output; (11) render human output. A model/estimation identity conflict is typed `ConflictingEvidenceInput`. Any duplicated scientific evidence with unequal IDs is the same error; equal IDs deduplicate by ArtifactId.

MHI-T20a–g run the actual binary/runner with real serialized fixtures and flags. They assert process result, typed report, JSON, human report and forbidden conclusion absence: a timescale match alone does not prove mechanism; slow mode alone does not diagnose fouling; reference offset alone does not diagnose reference failure; missing evidence cannot be Strong; dependent evidence cannot independently confirm; out-of-domain evidence downgrades; Transitional/Disturbed/Indeterminate cannot silently support steady state.

## 11. Phase A0 schema and artifact-kind repair

For every affected artifact contract: `CURRENT_SCHEMA_VERSION=2`, `LEGACY_SCHEMA_VERSIONS=&[1]`. Do not include 2 in legacy. Validator order is: read schema; identify current/legacy; schema 2 requires `artifact_kind` and exact expected kind; schema 1 follows documented legacy migration; otherwise reject. This changes `src/domain/artifact.rs::validate_value` so the missing-kind allowance applies only to schema 1 explicitly, not membership in a list containing current.

| Artifact kind / contract | Producer sites (all schema-2 writers) | current contract → desired | schema-1 migration |
|---|---|---|---|
| `transient_analysis` / `TransientAnalysisReport` | `src/potentiometry/transient/mod.rs` | 1/[1] → 2/[1] | missing kind accepted |
| `calibration_observations` / `CalibrationObservationSet` | `src/potentiometry/calibration/observations.rs` | 1/[1] → 2/[1] | missing kind accepted |
| `calibration_model` / `StoredCalibrationModel` | `src/potentiometry/calibration/mod.rs` | 1/[1] → 2/[1] | missing kind accepted |
| `calibration_analysis` / `CalibrationAnalysisReport` | `src/potentiometry/calibration/mod.rs` | 1/[1] → 2/[1] | missing kind accepted |
| `signal_analysis` / `SignalAnalysisReport` | `src/signal/mod.rs` | 1/[1] → 2/[1] | missing kind accepted |
| `mechanism_analysis` / `MechanismAnalysisReport` | `src/runners/mechanism.rs::{compare,trend}` | 1/[1] → 2/[1] | missing kind accepted |
| `health_assessment` / `SensorHealthAssessment` | `src/health/assessment.rs` | 1/[1] → 2/[1] | missing kind accepted |
| `health_trend` / `HealthTrendReport` | `src/health/trend.rs` | 1/[1] → 2/[1] | missing kind accepted |

The table contains eight artifact kinds and nine code construction sites because `MechanismAnalysisReport` has compare and trend writers; it is not eight producer sites literally. `eis_fit` and `health_baseline` already contract at current 2 and are outside A0. For each row MHI-T02a correct schema-2 kind passes; T02b wrong kind fails; T02c missing schema-2 kind fails; T02d documented schema-1 fixture passes; T02e unsupported version fails; T02f producer serialize→validate→reread passes and writer output validates its contract. Fixtures are real JSON under the existing artifact test fixture location selected by A0; no fixture invents unknown data fields.

## 12. MHI-R14: nine health dimensions (C)

```rust
pub enum HealthDimension {
    SignalIntegrity, CalibrationHealth, DynamicResponseHealth, ReferenceStability,
    EnvironmentalRobustness, ModelConsistency, Observability, UncertaintyHealth, DataQuality,
}
pub enum HealthInterpretationCategory { ObservedBehavior, ModelInconsistency, EnvironmentalEffect, CalibrationIssue, PossiblePhysicalDegradation }
pub enum CausalStatus { Observed, Associated, Hypothesized, ExperimentallySupported, ValidatedForDomain, Indeterminate }
```

Possible statuses are existing `OverallHealthStatus::{WithinBaseline,Watch,Degraded,Critical,DataQualityInsufficient,Indeterminate}`; no missing dimension is silently WithinBaseline. Legacy domain mapping is an input view, not a causal proof.

| Dimension | Meaning / allowed source | minimum evidence and missing behavior | legacy mapping / limitation |
|---|---|---|---|
| SignalIntegrity | noise, spikes, drift from signal artifact | one Valid signal feature; absent → Indeterminate | SignalNoise/Drift; does not prove degradation |
| CalibrationHealth | calibration validity/prediction | Valid calibration artifact + applicable domain; absent → Indeterminate | Calibration; does not identify cause |
| DynamicResponseHealth | transient amplitude/tau/recovery | Valid transient evidence and §7 state policy; absent → Indeterminate | DynamicResponse; slow mode is noncausal |
| ReferenceStability | independently anchored reference behavior | Valid reference anchor + independent confirmation; absent → Indeterminate | no direct domain; offset alone is insufficient |
| EnvironmentalRobustness | environmental covariate response | Valid environment/covariate artifact; absent → Indeterminate | DataQuality; correlation is not cause |
| ModelConsistency | observed-predicted residual, model validity | Valid model/estimation plus residual sign contract; absent → Indeterminate | MechanismEvidence; residual is not sensor failure |
| Observability | §9 requirement results | all declared critical results; absent → Indeterminate | MechanismEvidence; filter observability alone is insufficient |
| UncertaintyHealth | uncertainty completeness/conditioning | Valid uncertainty metadata; absent → Indeterminate | DataQuality; incomplete uncertainty is no defect diagnosis |
| DataQuality | finite, aligned, complete input | Valid parse/alignment diagnostics; absent → DataQualityInsufficient | DataQuality; no physical inference |

Every health finding retains separate `HealthInterpretationCategory` and `CausalStatus` per frozen SAR-010. Physical degradation is at most Hypothesized without §6 independent gates. MHI-T14a–i are one stable assertion per listed dimension; MHI-T14j asserts a missing dimension is Indeterminate.

## 13. Frozen residual contract

For model/estimation potentiometric data only, `unexplained_residual_v=measured_potential_v-predicted_potential_v`. Legacy EIS residual `real`, `imaginary`, `magnitude`, and `phase_deg` remains fitted minus measured; no EIS value is sign-inverted. Any future cross-domain normalization adds a separately named value containing original convention and explicit transform.

## 14. Configuration registry and absence behavior

Every scientific threshold above is configuration, finite, serialized, and recorded by `ThresholdProvenance`; no code constant is a scientific threshold.

| Name | type/unit/range | required | absent/invalid behavior |
|---|---|---|---|
| `critical_moderate_contradiction_count` | usize, ≥1 | B | Unassessed |
| `minimum_independent_supporting_families`, `minimum_non_timescale_supporting_families`, `minimum_validation_acquisition_families` | usize, ≥1 | B | Unassessed |
| `amplitude_floor` | f64, response unit, >0; `max_relative_amplitude_error` f64 ≥0 | when amplitude predicted | amplitude NotAssessed |
| `minimum_repeatability_replicates` usize ≥2; `maximum_log_tau_repeatability_sd` f64 ≥0 | repeatable mode | repeatability NotAssessed |
| `maximum_timestamp_difference_s` f64 seconds ≥0; `minimum_classified_fraction` f64 [0,1] | temporal evidence | MissingEvidence/Indeterminate |
| `minimum_fraction` | f64 [0,1] | MinimumSteadyFraction | config startup error |
| all `TimescaleEvidenceConfig` fields | §8 ranges, dimensionless except seconds intervals calculated from input | timescale | strength NotAssessed |
| `minimum_covariate_samples` usize ≥1; `minimum_covariate_range` f64 >0; `maximum_absolute_pearson_correlation` f64 [0,1] | covariate | NotAssessed |
| `minimum_interferent_samples` usize ≥1; `minimum_interferent_log10_range` f64 >0 | interferent | NotAssessed |
| `minimum_absolute_log10_activity_step` f64 >0; `minimum_pre_event_points`, `minimum_post_event_points` usize ≥1 | excitation | NotAssessed |

## 15. Complete traceability matrix

| Requirement | Normative behavior | module/symbol | AC | Test IDs | objective failure | compatibility | scientific risk | phase |
|---|---|---|---|---|---|---|---|---|
| MHI-R1 | frozen contracts preserved | all | AC1 | T01a,b | forbidden causal/default output | additive | invalid science | all |
| MHI-R2 | A0 eight contracts fixed | artifact_contracts | AC2 | T02a-f | any matrix mismatch | schema1 readable | broken IO | A0 |
| MHI-R3 | outer adapter only | evidence module | AC3 | T03a,b | model imports evidence | additive | coupling | A1 |
| MHI-R4 | orthogonal evidence | evidence types | AC4 | T04a-d | invalid combo accepted | additive | fabricated strength | A1 |
| MHI-R5 | lineage closure | lineage resolver | AC5 | T05a-d | Unknown Independent | legacy Unknown | false independence | A1 |
| MHI-R6 | absence preserved | adapters | AC6 | T06a-c | missing gains value | legacy retained | false certainty | A1 |
| MHI-R7 | component/status separate | mechanism assessment | AC7 | T07a,b | component mutates | additive | identity conflation | B |
| MHI-R8 | deterministic lifecycle | hypothesis assessor | AC8 | T08a-d | gate skipped | additive | causal overclaim | B |
| MHI-R9 | exact timescale science | timescale assessor | AC9 | T09a-f | wrong interval/class | additive | false match | B |
| MHI-R10 | EIS authorization | timescale adapter | AC10 | T10a,b | generic frequency converts | existing EIS preserved | invalid conversion | B |
| MHI-R11 | temporal join | temporal adapter | AC11 | T11a-k | unresolved supports | additive | temporal misuse | A1 |
| MHI-R12 | residual signs | result adapters | AC12 | T12a,b | sign inversion | EIS preserved | wrong diagnosis | B/C |
| MHI-R13 | identifiability | identifiability assessor | AC13 | T13a-l | unknown deserialize fails/satisfies | strings stable | false ID | A1/B |
| MHI-R14 | nine dimensions | health assessment | AC14 | T14a-j | dimension omitted | domain views retained | health gap | C |
| MHI-R15 | stable diagnostics | runners | AC15 | T15a-g | helper-only pass | additive | unwired behavior | B/C |
| MHI-R16 | provenance baselines | health adapters | AC16 | T16a-c | absent baseline healthy | old baseline indeterminate | false health | C |
| MHI-R17 | category/causality | results health | AC17 | T17a,b | physical cause lacks gate | legacy map | causal overclaim | C |
| MHI-R18 | confounding separation | health rules | AC18 | T18a-d | residual→failure | additive | wrong cause | C |
| MHI-R19 | compatibility | artifact readers | AC19 | T19a-f | promised input fails | explicit migration | data loss | A0–E |
| MHI-R20 | exact CLI | cli/runners | AC20 | T20a-g | flags ignored/conflict wins | old invocation works | hidden path | B/C |
| MHI-R21 | render-only | plottings | AC21 | T21a | renderer assesses | additive | opaque science | D |
| MHI-R22 | units/signs/variance | adapters | AC22 | T22a-d | unit/sign violation | preserve stored | invalid math | A1–C |
| MHI-R23 | production validation | tests | AC23 | T23a-d | helper replaces CLI | additive | false coverage | E |
| MHI-R24 | explainable conclusion | reports | AC24 | T24a,b | cannot reconstruct IDs | additive | unauditable | D/E |
| MHI-R25 | named strength authority | evidence | AC25 | T25a,b | raw gets strength | additive | invented authority | A1 |
| MHI-R26 | independent reviews | docs | AC26 | T26a | review absent | none | release risk | E |

### 15.1 Stable test-variant registry

The abbreviated references in §15 expand to the following exact IDs; a range in §15 is not a substitute for this registry.

| Stable ID | Exact variant / assertion |
|---|---|
| MHI-T01a-frozen | no newly implemented default produces a forbidden causal assignment |
| MHI-T01b-residual | both frozen residual sign fixtures retain their declared signs |
| MHI-T02a-current-correct-kind | every A0 kind accepts schema 2 with its exact kind |
| MHI-T02b-current-wrong-kind | every A0 kind rejects schema 2 with another kind |
| MHI-T02c-current-missing-kind | every A0 kind rejects schema 2 without kind |
| MHI-T02d-legacy | every A0 documented schema-1 fixture reads |
| MHI-T02e-unsupported | every A0 kind rejects unsupported schema |
| MHI-T02f-producer-roundtrip | each A0 producer serializes, validates and rereads |
| MHI-T03a-dependency | model core has no evidence-module dependency |
| MHI-T03b-adapter | outer adapter consumes only public result contracts |
| MHI-T04a-valid | every permitted EvidenceRecord combination round-trips |
| MHI-T04b-missing | invalid Missing combination returns typed error |
| MHI-T04c-not-applicable | invalid NotApplicable combination returns typed error |
| MHI-T04d-strength | invalid strength/derivation combination returns typed error |
| MHI-T05a-closure | sorted transitive closure contains all ancestors |
| MHI-T05b-cycle | cycle is reported without lost known ancestors |
| MHI-T05c-missing | missing ancestor yields Incomplete and Unknown independence |
| MHI-T05d-initialization | production prior/initialization reuse is SameSource or PartiallyDependent |
| MHI-T06a-missing | absent input yields Missing/NotAssessed, no quantity |
| MHI-T06b-na | non-applicable input yields NotApplicable only |
| MHI-T06c-legacy | legacy lineage stays Unknown |
| MHI-T07a-immutable | hypothesis level never mutates component interpretation |
| MHI-T07b-independent | supported hypothesis does not upgrade component physical identity |
| MHI-T08a-timescale-only | match alone stops below ExperimentallySupported |
| MHI-T08b-contradiction | critical contradiction lowers current level and appends history |
| MHI-T08c-amplitude | amplitude missing/failure blocks its required gate |
| MHI-T08d-validation | missing protocol forbids ValidatedForDomain |
| MHI-T09a-log-distance | signed log ratio and absolute distance match reference values |
| MHI-T09b-resolution | sampling/duration/separation failures are deterministic |
| MHI-T09c-interval-zero | zero-crossing signed interval converts to specified d bounds |
| MHI-T09d-threshold | conservative upper bound selects exact strength bin |
| MHI-T09e-covariance | same-fit omitted covariance returns NotAssessed |
| MHI-T09f-above-weak | above weak emits Neutral/NotAssessed |
| MHI-T10a-direct-parameter | permitted direct fitted EIS tau remains usable |
| MHI-T10b-frequency | unapproved EIS frequency is NotApplicable |
| MHI-T11a-perfect | perfect point alignment joins |
| MHI-T11b-partial | partial alignment yields the exact classified fraction |
| MHI-T11c-below-minimum | below minimum fraction is Missing/Indeterminate |
| MHI-T11d-quasi | equilibrium plus quasi obeys both allow settings |
| MHI-T11e-transitional | equilibrium plus transitional obeys selected policy |
| MHI-T11f-disturbed | disturbed obeys reject/worst-case policy |
| MHI-T11g-indeterminate | indeterminate obeys worst-case policy |
| MHI-T11h-clock | unresolved clocks are Missing/Indeterminate |
| MHI-T11i-scope | unresolved scopes are Missing/Indeterminate |
| MHI-T11j-experiment | missing experiment ID is Missing/Indeterminate |
| MHI-T11k-tie | nearest-time tie is Ambiguous/MissingEvidence |
| MHI-T12a-model | model/estimation residual equals measured minus predicted |
| MHI-T12b-eis | stored EIS residual equals fitted minus measured |
| MHI-T13a-known | all ten known strings deserialize and serialize unchanged |
| MHI-T13b-custom | unknown nonempty string becomes Custom unchanged |
| MHI-T13c-empty | empty unknown string rejects |
| MHI-T13d-mode | ModeSeparation exact assessor behavior |
| MHI-T13e-anchor | ReferenceAnchor complete-lineage/domain behavior |
| MHI-T13f-duration | ObservationDuration exact assessor behavior |
| MHI-T13g-covariate | IndependentCovariateVariation exact assessor behavior |
| MHI-T13h-interferent | InterferentVariation exact assessor behavior |
| MHI-T13i-excitation | TransientExcitation exact assessor behavior |
| MHI-T13j-unsupported | each unsupported known kind is deterministic NotAssessed |
| MHI-T13k-custom-assessor | custom without registration is deterministic NotAssessed |
| MHI-T13l-observability | filter observability alone does not satisfy a requirement |
| MHI-T14a-signal through MHI-T14i-data | one test per ordered §12 dimension verifies minimum/missing behavior |
| MHI-T14j-missing-dimension | absent dimension is never WithinBaseline |
| MHI-T15a-mechanism-cli through MHI-T15g-health-cli | each §10 negative executes a real runner, JSON and human output |
| MHI-T16a-baseline through MHI-T16c-domain | missing/inadequate/out-of-domain baseline cannot make a healthy or causal conclusion |
| MHI-T17a-category | finding serializes category and causal status separately |
| MHI-T17b-physical | physical finding without §6 gates does not exceed Hypothesized |
| MHI-T18a-residual through MHI-T18d-confounded | anomaly alternatives stay separated from sensor failure |
| MHI-T19a-schema through MHI-T19f-future | every §4 compatibility row has read/reject fixture |
| MHI-T20a-timescale through MHI-T20g-steady-state | seven named §10 CLI negatives |
| MHI-T21a-renderer | renderer has no assessment calculation and DTO projection matches |
| MHI-T22a-units through MHI-T22d-variance | V, s, Hz, unit and covariance semantics retained |
| MHI-T23a-unit through MHI-T23d-cli | full suite retains production rather than helper-only coverage |
| MHI-T24a-json | consumer reconstructs conclusion from serialized IDs/lineage |
| MHI-T24b-text | human report projects, rather than substitutes for, serialized evidence |
| MHI-T25a-raw | raw adapter evidence is NotAssessed |
| MHI-T25b-derived | assessed strength records authority/version/configuration |
| MHI-T26a-rereview | independent scientific and architecture GO records are attached |

`MHI-T14a-signal` through `MHI-T14i-data`, `MHI-T15a-mechanism-cli` through `MHI-T15g-health-cli`, `MHI-T16a-baseline` through `MHI-T16c-domain`, `MHI-T18a-residual` through `MHI-T18d-confounded`, `MHI-T19a-schema` through `MHI-T19f-future`, `MHI-T20a-timescale` through `MHI-T20g-steady-state`, `MHI-T22a-units` through `MHI-T22d-variance`, and `MHI-T23a-unit` through `MHI-T23d-cli` are literal individual IDs, not parameterized test names.

## 16. Pre-review self-audit

| Review blocker | Exact plan section | Exact defined interface/algorithm | Remaining invention required? |
|---|---|---|---|
| Lineage transitive closure | §3.2 | `resolve_lineage` | NO |
| AcquisitionFamilyId | §3.1 | enum, inheritance and Unknown rule | NO |
| Canonical hash ownership | §3.3 | owner hash-view table | NO |
| Dependency ordering | §3.1 | role/kind/ID sorting | NO |
| Missing-ancestor behavior | §3.2 | `Incomplete` plus missing IDs | NO |
| EvidenceRecord | §5 | full struct | NO |
| EvidenceRef | §5 | full struct and resolver rule | NO |
| EvidenceBundle | §5 | full struct and ordering | NO |
| ThresholdProvenance | §5 | full struct/source enum | NO |
| Combination validator | §5.1 | exhaustive combination table/errors | NO |
| Hypothesis downgrade | §6 | recomputation/history causes | NO |
| Critical contradiction | §6 | Strong or configured Moderate rule | NO |
| Amplitude agreement | §6 | relative-error equation | NO |
| Repeatability | §6 | log-space sample SD | NO |
| ExperimentallySupported gate | §6 | ten required gates | NO |
| ValidatedForDomain gate | §6 | protocol/domain/criteria gate | NO |
| Scope identity | §7 | ScopeKey and bindings | NO |
| MixedStatePolicy | §7 | three exact enum variants | NO |
| equilibrium_fraction denominator | §7 | `N_equilibrium/N_classified` | NO |
| classified_fraction denominator | §7 | `N_classified/N_target` | NO |
| Temporal precedence | §7 | six-step order | NO |
| confidence level | §8 | range and alpha equation | NO |
| z quantile | §8 | inverse standard-normal CDF | NO |
| signed log-ratio interval | §8 | r/sigma/r_low/r_high equations | NO |
| d_tau interval conversion | §8 | zero-containing/nonzero rules | NO |
| threshold-crossing rule | §8 | conservative `d_high` bins | NO |
| unknown identifiability serialization | §9 | `Custom(String)` serde | NO |
| unsupported kind result construction | §9 | `identifiability.not_assessed` rows | NO |
| custom kind result construction | §9 | custom not-assessed rule | NO |
| mechanism CLI estimation/model route | §10 | three optional mechanism flags | NO |
| health CLI estimation/model/mechanism route | §10 | four optional health flags | NO |
| lineage catalog route | §10 | catalog flag/load step | NO |
| schema-2 current version | §11 | current `2` rule | NO |
| legacy version list | §11 | `&[1]` rule | NO |
| schema-2 missing-kind rejection | §11 | validator order/test T02c | NO |
| nine health dimensions | §12 | enum/nine-dimension matrix | NO |
| plan tracked in Git | §18 | staged commit procedure | NO |

Acceptance-critical text contains no TBD, suitable, reasonable, or implementation-choice threshold. The type audit confirms every new normative type is defined in §§3, 5–9, 12 or cited as an existing exact source path.

## 17. Implementation acceptance and reporting

Each phase report gives changed files, test IDs/results, exact command output classification, compatibility fixtures, remaining known baseline failures, commit, and rollback target. No phase reports GO with a failed required test. E runs all §1 commands, full CLI negatives, migration tests, and independent Scientific and Architecture re-review against the committed plan and committed implementation.

## 18. Plan tracking and final validation

After this document is finalized, execute:

```bash
git add docs/engineering_specification/model_based_mechanism_sensor_health_v1_plan.md
git ls-files --error-unmatch docs/engineering_specification/model_based_mechanism_sensor_health_v1_plan.md
git diff --cached --check
git commit -m "docs(plan): finalize mechanism-health V1 implementation contract"
git status
git rev-parse HEAD
shasum -a 256 docs/engineering_specification/model_based_mechanism_sensor_health_v1_plan.md
git hash-object docs/engineering_specification/model_based_mechanism_sensor_health_v1_plan.md
```

## 19. Phase A0 — Artifact Contract Repair implementation prompt

Implement Phase A0 only in `/Users/xingyuwang/ProjectOngoing/rust_electroanalysis_cli`. Inspect Git status, branch, base commit, `src/domain/artifact.rs`, `src/results/artifact_contracts.rs`, all eight artifact types and the nine producer construction sites in §11, existing artifact tests and fixtures. Confirm the current contracts and producer schemas before changing files. The root cause is that current schema-2 writers are paired with contract current version 1, and `validate_value` uses `LEGACY_SCHEMA_VERSIONS` to permit a missing kind; this becomes unsafe when a current version is included in legacy. For exactly the eight §11 affected artifact kinds set `CURRENT_SCHEMA_VERSION = 2` and `LEGACY_SCHEMA_VERSIONS = &[1]`; do not set `[1,2]`. Enforce validator order: read schema version; schema 2 requires artifact kind and exact expected kind; schema 1 applies only its documented missing-kind migration; unsupported versions reject. Preserve current EIS and health-baseline contracts. Add real fixture and production tests per affected kind: correct schema-2 kind pass, wrong-kind fail, missing-kind schema-2 fail, documented schema-1 form pass, unsupported version fail, and producer serialize → validate → reread pass. Verify every current producer output is accepted by its declared contract. Do not add durable-lineage types, `EvidenceRecord`, `EvidenceBundle`, hypothesis assessment, mechanism/health assessment, or evidence CLI flags. Do not modify unrelated baseline formatting/clippy debt. Run the four §1 validation commands and classify failures as existing unrelated, A0-related, new regression, or resolved. Provide traceability to MHI-R2/T02a-f, exact compatibility behavior, all changed paths, commands/results, commit ID, and rollback target.
