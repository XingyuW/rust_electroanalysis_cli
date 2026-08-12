# Model-Based Mechanism and Sensor Health Integration V1 — Final Implementation Contract

**Status:** planning/specification only. This document authorizes no production Rust-code change.
**Repository:** `/Users/xingyuwang/ProjectOngoing/rust_electroanalysis_cli`
**Planning branch / base commit:** `plan/mhi-v1-b-contract-amendment` / `d1fcb5125326d6118fd848bb76c37adc4b5fa7ef`.

This is the normative contract for Model-Based Mechanism and Sensor Health Integration V1 (MHI V1). A normative word is binding. A missing required input or configuration never authorizes an implementation default. Existing public behavior remains unless this contract explicitly defines an additive change.

Frozen contracts: SAR-009 phase split; SAR-010 typed health interpretation; SAR-011 residual signs. No default assigns a fast mode to double layer, a slow mode to adsorption/fouling/water layer, a transduction candidate to proven solid contact, a reference offset to reference-electrode failure, or a residual to sensor failure. Timescale agreement alone is not causal proof. Same-source evidence is not independent confirmation. Transitional, Disturbed, and Indeterminate data are not steady-state evidence unless the exact policy below admits it.

## 1. Repository reconciliation and baseline

The authoritative repository workflow is `.ai/CODE_QUALITY_WORKFLOW.md`; it is present in this checkout. The earlier search missed hidden directories. This plan follows that workflow and does not modify it.

| Finding ID | Original severity | Current classification | Repository evidence | Root cause | Required plan correction | Plan section modified |
|---|---:|---|---|---|---|---|
| F1 A0 scope / compatibility | P1 | REMEDIATED | `src/domain/artifact.rs::validate_value` presently uses legacy-version membership to permit a missing kind; `src/results/artifact_contracts.rs` shows `eis_fit` and `health_baseline` are current 2 / legacy `[1,2]`. | A0 scope excluded the validator while §11 required changing it. | Contract-owned current-kind policy, exact compatibility table, and validator algorithm. | §§2, 11, 15, 19 |
| F2 pairwise evidence independence | P1 | REMEDIATED | Existing repository code has no MHI evidence-bundle type; the prior planned `EvidenceRecord` made `independence` unary. | A relation was modeled as an attribute of one record. | Bundle-owned canonical pair assessments, recomputing validator, and clique algorithm. | §§3, 5 |
| F3 hypothesis history / gate applicability | P1 | REMEDIATED | `src/model/component.rs` owns immutable interpretation status; `src/cli.rs` has no prior-mechanism input. | Aggregate ownership and applicability were implicit. | Serialized definition, required-or-NotApplicable gates, report-owned history, and prior-artifact route. | §§6, 10 |
| F4 uncertainty serialization / propagation | P1 | PARTIALLY CONFIRMED | The committed plan defines a standalone `TimescalePairUncertainty`, but `EvidenceBundle` has no serialized covariance collection; the type has no provenance, construction route, cardinality, or exact lookup rule. | Pair covariance was specified scientifically but not made durable, uniquely owned, or retrievable by a canonical evidence pair. | Make `EvidenceBundle.timescale_pair_uncertainties` the sole serialized V1 owner; define the shared pair key, provenance, builder, validation, lookup, hashing, compatibility, and permanent tests. | §§5, 8, 15–16, 18 |
| F5 workflow path | P2 | REMEDIATED | `.ai/CODE_QUALITY_WORKFLOW.md` exists. | Earlier search omitted hidden directories. | Record the exact authoritative path. | §1 |
| F6 MHI-R2 traceability | P2 | REMEDIATED | A0 requires both `src/results/artifact_contracts.rs` and `src/domain/artifact.rs::validate_value`. | Traceability listed only the contract table. | Name both locations and producer/result modules. | §15 |
| F7 / SAR-007 | P1 | CONFIRMED | `MechanismCompareCommand` accepts EIS/transient/calibration only; `HealthAssessCommand` accepts legacy inputs; existing negatives are not all CLI fixture paths. | Runtime inputs and behavior were unspecified. | Exact additive flags, order and E2E tests. | §10 |
| F8 / SAR-009/A0 | P1 | CONFIRMED | `src/results/artifact_contracts.rs` lists current schemas in `LEGACY_SCHEMA_VERSIONS` for current-2 artifacts; `validate_value` consequently accepts a schema-2 missing kind. | Current and readable versions were not separated. | A0 validation, kind matrix and producers. | §11 |
| F9 / MHI-R14 | P1 | CONFIRMED | `src/results/health.rs::HealthDomain` has seven legacy domains, not nine MHI dimensions. | R14 named but did not enumerate dimensions. | Exact nine-dimension contract. | §12 |
| F10 / plan tracking | P2 | CONFIRMED | The plan was untracked at inspection (`?? docs/...plan.md`). | Prior review inspected a non-Git artifact. | Stage, verify and commit this plan. | §18 |
| A1-C1 aggregate experiment scope | P1 | CONFIRMED | `CalibrationAnalysisReport.source_experiments` is a vector; `health::baseline::build_with_contexts` consumes multiple manifest records; `health trend` and `mechanism trend` consume manifest record collections. These workflows cannot truthfully emit one mandatory experiment ID. | The approved A1 identity modeled every artifact as single-experiment and provided no aggregate identity. | Replace the mandatory field with `ArtifactExperimentScope`; define aggregate IDs, source authority, propagation, Unknown behavior, and temporal narrowing. | §§3, 4, 5, 7, 15, 16 |
| A1-C2 legacy lineage representation | P1 | CONFIRMED | Current result schemas have no `lineage`/`ArtifactIdentity` field. `read_artifact` validates only schema/kind and then uses serde deserialization; no migration object can distinguish absent historical lineage from a current known identity. | “Missing fields yield Unknown” was stated without a serialized state or migration representation. | Add explicit `ArtifactLineageState::LegacyUnknown` with a serde default and prohibit fabricated identity during read or reserialization. | §§3, 4, 15, 16 |
| A1-C3 labeled covariance | P1 | CONFIRMED | `EisFitStatistics.parameter_covariance`, `CalibrationFitStatistics.parameter_covariance`, and estimation covariance fields are `Vec<Vec<f64>>`; `src/mechanism/timescale.rs` and calibration uncertainty code use positional indexing. Model artifacts contain no scientific covariance field. | The approved plan prohibited positional adapter semantics but did not require producer-owned serialized axis labels. | Add producer-owned `LabeledCovarianceMatrix`, exact axis validation/lookup, explicit producer decisions, and unavailable behavior for unlabeled legacy covariance. | §§5.3, 8, 15, 16 |
| A1-RR-01 unique EIS covariance axes | P1 | CONFIRMED | `src/results/eis.rs::element_id_for_name` maps both `Q_CPE1` and `alpha_CPE1` to `CPE1`, while `parameter_covariance` remains an unlabeled positional matrix. | An EIS element instance was incorrectly treated as the individual covariance parameter. | Define producer-owned `EisParameterIdentity`, a complete element/parameter-key table, canonical `eis.parameter:<element_instance_id>:<parameter_key>` axes, and 1:1 labeled construction. | §§3.1, 5.3, 8, 15.1A, 15.3, 16 |
| A1-RR-02 LegacyUnknown lineage root | P1 | CONFIRMED | §3.1 keys `ArtifactLineageCatalog.artifacts` by `ArtifactId`, but §3.2 requires every resolver root to be found in that map; `LegacyUnknown` has no `ArtifactId`. | The catalog lookup model was used as the resolver root model, making the required legacy state unresolvable without fabrication. | Make `resolve_lineage(&ArtifactLineageState, ...)` the primary entrypoint, keep known nodes only in the catalog, and define deterministic Known/LegacyUnknown results plus the separate known-ID `RootMissing` API. | §§3.1–3.2, 5, 15.1A, 15.3, 16 |
| A1-RR-03 acquisition-family identity | P1 | CONFIRMED | `AcquisitionFamilyId` and `acquisition_family_ids` are used throughout §§3–5 and 15 but no type or unknown-family representation is defined. | Family identity was normative before its identity type, validation, producer authority, and Unknown semantics were specified. | Define `AcquisitionFamilyId`, `ArtifactAcquisitionFamilies`, `ResolvedAcquisitionFamilies`, family propagation, independence behavior, and producer authority table. | §§3.1–3.2, 5.2, 15.1A, 15.3, 16 |

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

Each phase has separate review, commit and rollback. **A0 may modify** `src/results/artifact_contracts.rs`, `src/domain/artifact.rs`, the already-enumerated affected producer modules and their artifact result/schema modules, and their production-path tests and fixtures. `src/domain/artifact.rs` is in A0 scope **only** for contract-driven current artifact-kind validation. A0 must not globally tighten or reinterpret artifact kinds whose contracts are outside the A0 repair set. A0 must not modify `ArtifactLineage`, `EvidenceRecord`, `EvidenceBundle`, mechanism assessment, health assessment, new mechanism evidence CLI flags, new health evidence CLI flags, hypothesis assessment, timescale evidence scoring, or identifiability evidence adapters. A1 begins only after an A0 commit.

## 3. Durable lineage contract (A1)

### 3.1 Normative types and serialization

Newtypes serialize as their inner strings. Existing repository types are: `ArtifactKind` at `src/domain/artifact.rs`, `ExperimentId` = new A1 string newtype, and all existing result payload types at `src/results/*.rs`.

```rust
pub struct ArtifactId(pub String); // exactly "sha256:" + 64 lowercase hex
pub struct ExperimentId(pub String); // nonempty UTF-8 identifier
pub struct AggregateExperimentScopeId(pub String); // "sha256:" + 64 lowercase hex
pub struct AcquisitionFamilyId(pub String);
pub enum ArtifactAcquisitionFamilies {
    Known(Vec<AcquisitionFamilyId>),
    Unknown,
}
pub enum ResolvedAcquisitionFamilies {
    Known(Vec<AcquisitionFamilyId>),
    Unknown,
}
pub enum ScopeKey { Specific(String), All, Unspecified }
pub enum ArtifactExperimentScope {
    Single { experiment_id: ExperimentId },
    Aggregate {
        aggregate_scope_id: AggregateExperimentScopeId,
        member_experiment_ids: Vec<ExperimentId>,
    },
    Unknown,
}
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
pub enum LineageResolutionStatus {
    Complete, Incomplete, CycleDetected, Inconsistent, RootMissing,
}
pub struct ResolvedArtifactLineage {
    pub status: LineageResolutionStatus,
    pub root_artifact_id: Option<ArtifactId>,
    pub ancestor_artifact_ids: Vec<ArtifactId>,
    pub missing_artifact_ids: Vec<ArtifactId>,
    pub acquisition_families: ResolvedAcquisitionFamilies,
    pub reasons: Vec<LineageResolutionReason>,
}
pub enum LineageResolutionReason {
    LegacyUnknownRoot,
    MissingDependency(ArtifactId),
    CycleDetected { cycle_artifact_ids: Vec<ArtifactId> },
    CatalogRootInconsistent,
}
pub enum EvidenceIndependence { Independent, PartiallyDependent, SameSource, Unknown }
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
pub enum UnknownLineageReason {
    FieldAbsentInLegacyArtifact,
    ExternalArtifactWithoutLineage,
    MigrationInformationUnavailable,
}
```

`Specific` contains a nonempty string. `All` means deliberately broad scope. `Unspecified` means unavailable, never a wildcard. `AcquisitionFamilyId` is a newtype over a nonempty UTF-8 string. Its canonicalization is: trim leading and trailing Unicode whitespace, reject the empty result, then preserve the remaining string exactly, including case. It is stable across derived artifacts and is never generated from an artifact path, output filename, or processing timestamp. No silent lowercasing is permitted because repository experiment identifiers are case-sensitive.

`ArtifactAcquisitionFamilies::Known` and `ResolvedAcquisitionFamilies::Known` contain deterministically bytewise-sorted, duplicate-free, nonempty family IDs when the source asserts a known family set. `Unknown` is a distinct serialized enum state; it is not the string `"Unknown"` and is not an empty known vector. A raw acquisition with an authoritative family has a nonempty `Known` set; a derived artifact may contain one or more families. An artifact with no family membership because the producer cannot establish it is `Unknown`, not `Known([])`.

An acquisition family is the original independent acquisition campaign, controlled replicate family, sensor exposure, or acquisition session from which an artifact derives. It is not a file, artifact ID, algorithm run, or inferred value. Producers inherit the sorted unique union from every direct dependency and add a raw-family identity only when the producer has an authoritative acquisition-level mapping. Any required `Unknown` source propagates to `Unknown` unless that producer records an independent authoritative family mapping. `ArtifactIdentity` uses `ArtifactAcquisitionFamilies`; lineage resolution uses `ResolvedAcquisitionFamilies`.

Family propagation is exact: `Known(A) + Known(B) -> Known(sorted_unique_union(A,B))`; any required `Unknown` source -> `Unknown`, subject only to the explicitly recorded independent-authority exception above. `Known([])` is not a valid known family state and is never used to mean Unknown.

Direct dependencies sort by role discriminant, then `artifact_kind.as_str()`, then artifact ID bytes. Artifact catalogs serialize `BTreeMap` key order. All listed vectors are serialized in their stated order.

`ArtifactExperimentScope::Single` means the complete artifact belongs to one authoritative experiment and uses that exact `ExperimentId`; no synthetic ID is permitted. `Aggregate` means the artifact combines at least two experiments. Its member IDs are nonempty, bytewise sorted, deduplicated, and contain at least two unique IDs. `AggregateExperimentScopeId` is not an `ExperimentId` and is exactly:

```text
SHA256(UTF-8(
    "aggregate-experiment-scope-v1" || NUL ||
    aggregation_kind || NUL ||
    join(sorted_unique_member_experiment_ids, NUL)
))
```

`aggregation_kind` is a nonempty producer-owned stable string. The NUL separators are part of the hash input; IDs are joined without a trailing separator. `Unknown` is used when historical or external membership cannot be established. Scope propagation is exact: `Single(A)+Single(A) → Single(A)`; distinct singles → `Aggregate(union)`; aggregate plus single or aggregate → aggregate union; any required Unknown dependency → Unknown. A producer may override the last rule only with an independent authoritative membership source, which it records in the artifact derivation/provenance. Unknown never joins automatically with Single or Aggregate.

The current aggregate-capable workflows and their authoritative sources are:

| Artifact kind | Producer | Single/Aggregate capability | Authoritative member-experiment source | `aggregation_kind` |
|---|---|---|---|---|
| `calibration_observations` | `src/potentiometry/calibration/observations.rs::extract_observations` | Single when all observation `experiment_id` values match; Aggregate when at least two unique values occur | `CalibrationObservation.experiment_id` on every retained observation | `calibration-observation-set-v1` |
| `calibration_analysis` | `src/potentiometry/calibration/mod.rs::fit_calibration` and `src/runners/calibration.rs` | Single or Aggregate | retained observation set IDs; current `CalibrationAnalysisReport.source_experiments` is the direct evidence | `calibration-analysis-v1` |
| `health_baseline` | `src/runners/health.rs::baseline` → `src/health/baseline.rs::build_with_contexts` | Aggregate when manifest records resolve to at least two experiments; Single only for one authoritative member | each manifest record’s metadata `experiment_id`, not record count or file path | `health-baseline-v1` |
| `health_assessment` | `src/runners/health.rs::assess` | Single when the signal and all consumed scoped inputs resolve to one experiment; Aggregate when explicitly scoped inputs resolve to multiple | signal artifact scope plus consumed artifact scopes | `health-assessment-v1` |
| `health_trend` | `src/runners/health.rs::trend` → `src/health/trend.rs::report` | Aggregate for multiple assessment records; Single only for one record | selected assessment artifact/member scope; trend manifest membership alone is insufficient | `health-trend-v1` |
| `mechanism_analysis` | `src/runners/mechanism.rs::compare` | Single when EIS/transient/optional inputs agree; Aggregate only when an explicit multi-member source is supplied | source artifact scopes; current compare rejects mismatched EIS/transient IDs | `mechanism-analysis-v1` |
| `mechanism_analysis` | `src/runners/mechanism.rs::trend` → `src/mechanism/trend.rs::calculate_trend` | Aggregate for multiple manifest records; Single only for one record | `MechanismRecordInput.experiment_id` or source EIS/transient artifact scope for each record | `mechanism-trend-v1` |

The trend builders do not infer membership from `record_id`, path, row order, or the independent-variable value. A trend with missing member scopes is `Unknown`, not Aggregate.

The acquisition-family assignment contract for A1 source artifacts is:

| Artifact/source | Authoritative family source | Behavior when authority is absent |
|---|---|---|
| EIS fit raw acquisition | acquisition campaign/session metadata attached to the measured EIS input | `ArtifactAcquisitionFamilies::Unknown`; circuit label, output path, and fit timestamp are rejected as substitutes |
| transient raw acquisition | acquisition session/campaign metadata attached to the measured transient input | `Unknown` |
| calibration observation/raw measurement | controlled acquisition campaign or replicate-family metadata on the retained observation set | `Unknown`; observation ID alone is not a family |
| signal/raw sensor acquisition | sensor exposure/acquisition-session metadata in the source record | `Unknown` |
| imported legacy/external artifact | no current authoritative acquisition identity | `Unknown`; a `LegacySourceFingerprint` is not a family |
| derived A1 artifact | sorted union of authoritative direct-dependency families, plus an independently acquired raw family only when explicitly recorded by the producer | `Unknown` if any required source is Unknown |

Generic A1 adapters never guess a family from a path, filename, timestamp, record ID, or algorithm run. A producer may supply an independent authoritative mapping that resolves an otherwise Unknown family only when it records that mapping and its source field in the artifact provenance.

### 3.2 Closure resolver

`resolve_lineage` has exactly this public interface and is the sole primary resolver for transitive closure:

```rust
pub fn resolve_lineage(
    root: &ArtifactLineageState,
    catalog: &ArtifactLineageCatalog,
) -> ResolvedArtifactLineage
```

The catalog contains known nodes only. Every stored node has `ArtifactLineageState::Known` semantics represented by its `ArtifactIdentity` and direct dependencies. `LegacyUnknown` is never inserted into `artifacts` and no placeholder `ArtifactId` is created.

For a `Known { identity, direct_dependencies }` root, the resolver treats the supplied identity and dependencies as authoritative root data and does not require the root to be duplicated in the catalog. It sets `root_artifact_id=Some(identity.artifact_id)`, includes the root family state, and recursively resolves every sorted direct dependency through the catalog. If the root ID is present in the catalog, its node must be semantically identical to the supplied identity/dependencies; otherwise the result is `Inconsistent` with `CatalogRootInconsistent`. A separate `resolve_known_artifact_id(root_id, catalog)` helper may resolve a known ID solely from the catalog; only that explicit ID-root API returns `RootMissing` for an absent root.

Traversal is deterministic depth-first over sorted dependencies, with `Visiting` and `Visited` sets keyed by `ArtifactId`. For each dependency, add its ID to `ancestor_artifact_ids`; if absent, add the exact ID to `missing_artifact_ids` and reason `MissingDependency(id)`; if Visiting, report one deterministic cycle consisting of the active stack slice from the first occurrence of that ID through the repeated ID, and reason `CycleDetected { cycle_artifact_ids }`; otherwise recurse. On every present node, union its `ArtifactAcquisitionFamilies` into the result. Sort and deduplicate ancestors and missing IDs bytewise. Return `CycleDetected` if any back edge, otherwise `Incomplete` if any dependency is missing, otherwise `Complete`. Known ancestors and their family information are retained for both `Incomplete` and `CycleDetected` results.

For a `LegacyUnknown { .. }` root, return exactly `status=Incomplete`, `root_artifact_id=None`, empty ancestor IDs, empty missing IDs, `acquisition_families=Unknown`, and `reasons=[LegacyUnknownRoot]`. It must not return `RootMissing`, inspect a fabricated ID, create a synthetic ID, or classify the root as complete. `RootMissing` is therefore retained only for the explicit known-ID helper and is not a possible LegacyUnknown result.

The resolver neither drops a missing ancestor nor turns missing metadata into a family. `classify_independence(a, b, bundle)` first resolves each evidence source from its `ArtifactLineageState` root. It returns `SameSource` only when known source identity proves equal artifact IDs. Otherwise it returns `Unknown` if either root is `LegacyUnknown`, either result is not `Complete`, either family state is `Unknown`, or either known family set is empty. If both are complete with known nonempty families, it returns `PartiallyDependent` when ancestor closures intersect or family intersections are nonempty, and `Independent` only when source IDs differ, closures are disjoint, and family sets are disjoint. Only `Independent` is independent confirmation; every other value is NotIndependent.

Permanent A1 production-path test MHI-T05d: serialize transient artifact → use it as estimation initialization or prior → serialize `StateEstimationReport` → mechanism evidence adapter reads both plus the catalog. The later transient evidence resolves SameSource or PartiallyDependent, never Independent; JSON and human outputs state the relationship.

### 3.3 Canonical semantic identity and hash ownership

`ArtifactId = "sha256:" + SHA256(canonical_semantic_bytes)` and `semantic_sha256` is the same lowercase hex without prefix. Canonical bytes are UTF-8 RFC 8785 canonical JSON for a named, owned hash-view struct. Reject non-finite numbers before producing bytes. Include kind, schema version, scientific payload, scope, the exact serialized `ArtifactAcquisitionFamilies` state and sorted known family IDs, scientifically meaningful producer algorithm/config identity, and sorted `(role, dependency ID)` pairs. Exclude artifact ID, semantic hash, absolute paths, output directory, generated timestamp, human text, and formatting. A dependency is represented by role + ID only; it is not embedded recursively.

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
| `mechanism_analysis` | `src/results/mechanism.rs` | evidence bundle, assessment/config | prose report/timestamp; all consumed artifacts direct dependencies | evidence §5.3 |
| `health_assessment` | `src/results/health.rs` | evidence bundle, dimensions/findings/config | prose report/timestamp; all consumed artifacts direct dependencies | dimension enum then finding ID |
| `health_trend` | `src/results/health.rs` | trend inputs/results/config | provenance paths/timestamp; direct dependencies | feature then point record ID |

Schema/version is included for every row. Producer modules own their named view and no alternate hasher may create an ID.

## 4. A1 artifact evolution and compatibility

Every A1-participating result artifact adds `#[serde(default = "legacy_unknown_lineage")] pub lineage: ArtifactLineageState`. Current writers emit `Known` only when authoritative identity and direct dependencies are available; otherwise they emit explicit `LegacyUnknown` and a warning/provenance entry. A historical or external artifact never receives a fabricated `ArtifactId`, `ExperimentId`, semantic hash, dependency list, or acquisition family. `Option<ArtifactIdentity>` is prohibited as the legacy representation.

`legacy_unknown_lineage` is exactly:

```text
LegacyUnknown {
    source_schema_version: None,
    reason: FieldAbsentInLegacyArtifact,
}
```

When a reader sees a missing lineage field, it preserves the artifact payload and deserializes this state. If the source schema version is available to the migration boundary, it records `Some(version)`; a current writer that genuinely cannot establish lineage emits `LegacyUnknown { source_schema_version: Some(current_version), reason: MigrationInformationUnavailable }`. Reserializing a legacy artifact preserves `LegacyUnknown`; it does not upgrade it. `LegacyUnknown` makes lineage closure `Incomplete`, evidence independence `Unknown`, and the artifact ineligible to count as independent confirmation.

The exact A1 schema transition is:

| Artifact kind | Current version before A1 | Current version after A1 | Legacy versions | Artifact-kind policy | Lineage behavior | Experiment-scope behavior | Labeled covariance | Backward compatibility |
|---|---:|---:|---|---|---|---|---|---|
| `eis_fit` | 2 | 3 | `[1,2]` | PreserveLegacyOptional | missing lineage → LegacyUnknown | Single/Aggregate/Unknown from authoritative input metadata | current labeled field additive; old positional field readable but not consumed | schemas 1–2 retain historical payload; no fabricated identity/covariance |
| `transient_analysis` | 2 | 3 | `[1,2]` | Required | missing lineage → LegacyUnknown | Single from authoritative experiment ID, otherwise Unknown | not consumed for pair covariance | schemas 1–2 readable with Unknown lineage |
| `calibration_observations` | 2 | 3 | `[1,2]` | Required | missing lineage → LegacyUnknown | source observation IDs determine Single/Aggregate/Unknown | not consumed for pair covariance | schemas 1–2 readable with Unknown lineage |
| `calibration_model` | 2 | 3 | `[1,2]` | Required | missing lineage → LegacyUnknown | inherited from observation/model source; Unknown without it | not consumed for pair covariance | schemas 1–2 readable with Unknown lineage |
| `calibration_analysis` | 2 | 3 | `[1,2]` | Required | missing lineage → LegacyUnknown | `source_experiments` determines Single/Aggregate; absent/empty → Unknown | current calibration covariance remains unavailable in A1 V1 | schemas 1–2 readable with historical payload and Unknown lineage |
| `signal_analysis` | 2 | 3 | `[1,2]` | Required | missing lineage → LegacyUnknown | Single from authoritative input, otherwise Unknown | not consumed for pair covariance | schemas 1–2 readable with Unknown lineage |
| `health_baseline` | 2 | 3 | `[1,2]` | PreserveLegacyOptional | missing lineage → LegacyUnknown | manifest metadata member IDs determine Aggregate; missing IDs → Unknown | not consumed for pair covariance | schemas 1–2 retain current kind-less compatibility and historical payload |
| `health_assessment` | 2 | 3 | `[1,2]` | Required | missing lineage → LegacyUnknown | propagated from signal/consumed artifacts or Unknown | not consumed for pair covariance | schemas 1–2 readable with Unknown lineage |
| `health_trend` | 2 | 3 | `[1,2]` | Required | missing lineage → LegacyUnknown | selected assessment member scopes; manifest alone cannot establish it | not consumed for pair covariance | schemas 1–2 readable with Unknown lineage |
| `mechanism_analysis` | 2 | 3 | `[1,2]` | Required | missing lineage → LegacyUnknown | compare inputs or trend member source scopes | not consumed for pair covariance | schemas 1–2 readable with Unknown lineage |
| `state_estimation` | 3 | 4 | `[1,2,3]` | PreserveLegacyOptional | missing lineage → LegacyUnknown | authoritative input experiment ID, otherwise Unknown | current labeled state/parameter covariance only where IDs resolve | schemas 1–3 retain current accepted/rejected kind matrix |
| `ism_model_compilation` | 4 | 5 | `[1,2,3,4]` | PreserveLegacyOptional | missing lineage → LegacyUnknown | model artifact has no experiment membership; Unknown unless an independent source is recorded | category C; no scientific covariance consumed | schemas 1–4 readable with Unknown lineage |
| `ism_model_analysis` | 4 | 5 | `[1,2,3,4]` | PreserveLegacyOptional | missing lineage → LegacyUnknown | inherited explicit input scope or Unknown | category C; no scientific covariance consumed | schemas 1–4 readable with Unknown lineage |

`ism_model_validation` is not modified by A1: current version 1 / legacy `[1]` and its A0 policy remain unchanged. This explicit exclusion is part of the compatibility contract; no global schema bump is permitted. Every modified row has a permanent read, write, missing-lineage, reserialization, and unsupported-version fixture; A0 artifact-kind semantics remain unchanged.

## 5. Serialized evidence contract (A1)

```rust
pub struct EvidenceId(pub String); // nonempty, stable within a bundle
pub struct EvidencePairKey {
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
}
pub struct HypothesisId(pub String);
pub struct EvidenceRequirementId(pub String);
pub struct HealthFindingId(pub String);
pub struct RequirementId(pub String);
pub struct ComponentId(pub String);
pub struct LegacySourceFingerprint(pub String);
pub enum EvidenceTarget {
    MechanismHypothesis(HypothesisId), HealthFinding(HealthFindingId),
    HealthDimension(HealthDimension), IdentifiabilityRequirement(RequirementId),
    ModelComponent(ComponentId),
}
pub enum EvidenceArtifactSource {
    Known { artifact_id: ArtifactId, artifact_kind: ArtifactKind },
    LegacyUnknown {
        artifact_kind: ArtifactKind,
        source_fingerprint: LegacySourceFingerprint,
    },
}
pub struct EvidenceSourceRef { pub artifact: EvidenceArtifactSource, pub field_path: String }
pub enum EvidenceSourceClass { Observed, ModelDerived, ProducerAssessment, ExternalReference }
pub enum EvidenceScopeDerivation {
    ArtifactScope,
    MemberRecord { experiment_id: ExperimentId, source_field_path: String },
}
pub enum EvidenceExperimentScope {
    Single { experiment_id: ExperimentId, derivation: EvidenceScopeDerivation },
    Aggregate {
        aggregate_scope_id: AggregateExperimentScopeId,
        member_experiment_ids: Vec<ExperimentId>,
    },
    Unknown,
}
pub enum EvidenceDirection { Supports, Contradicts, Neutral, NotApplicable }
pub enum EvidenceAvailability { Available, Missing, NotApplicable }
pub enum EvidenceStrength { NotAssessed, Weak, Moderate, Strong }
pub enum EvidenceValidity { Valid, OutsideDomain, Invalid, NotAssessed }
pub enum EvidenceUncertaintyModel {
    None,
    ExplicitLogInterval {
        lower_ln_tau_s: f64, upper_ln_tau_s: f64, confidence_level: f64,
    },
    LogNormal { variance_ln_tau_s: f64 },
    DeltaMethodTauVariance { variance_tau_s2: f64 },
}
pub struct EvidenceQuantity { pub value: f64, pub unit: String, pub uncertainty: Option<EvidenceUncertaintyModel> }
pub enum StrengthSource { NotAssessed, PreservedProducerAssessment, MechanismAssessor, HealthAssessor }
pub struct EvidenceRef { pub evidence_id: EvidenceId }
pub enum TimescaleCovarianceUse {
    ProducerBacked { pair: EvidencePairKey },
    IndependenceBasedZeroCovariance { pair: EvidencePairKey },
}
pub struct StrengthDerivation {
    pub algorithm_id: String, pub algorithm_version: String,
    pub source_evidence: Vec<EvidenceRef>, pub metric_values: BTreeMap<String, f64>,
    pub timescale_covariance_use: Option<TimescaleCovarianceUse>,
}
pub enum ThresholdSource { UserConfiguration, ValidatedDomain, ProducerContract, PublishedReference }
pub struct ThresholdProvenance {
    pub threshold_id: String, pub source: ThresholdSource, pub value: f64,
    pub unit: String, pub configuration_hash: Option<String>,
}
pub struct EvidenceRecord {
    pub evidence_id: EvidenceId, pub target: EvidenceTarget, pub source: EvidenceSourceRef,
    pub experiment_scope: EvidenceExperimentScope,
    pub source_class: EvidenceSourceClass, pub direction: EvidenceDirection,
    pub availability: EvidenceAvailability,
    pub strength: EvidenceStrength, pub validity: EvidenceValidity,
    pub quantity: Option<EvidenceQuantity>, pub strength_source: StrengthSource,
    pub strength_derivation: Option<StrengthDerivation>,
    pub threshold_provenance: Vec<ThresholdProvenance>,
    pub lineage_artifact_ids: Vec<ArtifactId>, pub warnings: Vec<String>,
}
pub struct EvidenceIndependenceAssessment {
    pub pair: EvidencePairKey,
    pub classification: EvidenceIndependence, pub algorithm_id: String,
    pub left_lineage_status: LineageResolutionStatus,
    pub right_lineage_status: LineageResolutionStatus,
    pub shared_ancestor_artifact_ids: Vec<ArtifactId>,
    pub shared_acquisition_families: Vec<AcquisitionFamilyId>,
    pub reasons: Vec<EvidenceIndependenceReason>,
}
pub enum EvidenceIndependenceReason {
    SameSourceArtifact, SharedAncestor, SharedAcquisitionFamily,
    IncompleteLineage, UnknownAcquisitionFamily, MissingAcquisitionFamily,
}
pub struct EvidenceBundle {
    pub schema_version: u32, pub experiment_scope: EvidenceExperimentScope,
    pub sensor_scope: ScopeKey, pub channel_scope: ScopeKey,
    pub records: Vec<EvidenceRecord>,
    pub independence_assessments: Vec<EvidenceIndependenceAssessment>,
    pub timescale_pair_uncertainties: Vec<TimescalePairUncertainty>,
    pub lineage_catalog: ArtifactLineageCatalog,
    pub warnings: Vec<String>,
}
```

`EvidenceArtifactSource::Known` is used only when the source artifact has a known `ArtifactId`. `EvidenceArtifactSource::LegacyUnknown` represents a readable legacy artifact without an artifact ID and never fabricates one. `LegacySourceFingerprint` is an audit/deduplication locator only: its canonical value is the lowercase hexadecimal SHA-256 of the exact serialized legacy artifact bytes, or, when raw bytes are unavailable at the adapter boundary, the public-reader source-content hash already exposed by the repository. It is not an `ArtifactId`, lineage identity, proof of independence, or acquisition-family identity. The fingerprint and `artifact_kind` remain serializable even though lineage resolution for this source is `LegacyUnknown`.

`EvidenceExperimentScope::Single` carries the actual member `ExperimentId` and a derivation. An adapter may narrow `Aggregate` to `Single` only from the exact selected source record/field that explicitly carries that member ID; membership in the aggregate set alone is insufficient. The adapter preserves the selected ID, `source_field_path`, and `EvidenceScopeDerivation::MemberRecord` in the evidence record. `ArtifactScope` is valid only when the selected artifact itself is Single. An aggregate evidence record is not point-temporally joinable.

`EvidenceUncertainty` is not a separate V1 type: `EvidenceQuantity.uncertainty` is `Option<EvidenceUncertaintyModel>`, and `None` means no uncertainty model. All IDs/paths/units are nonempty. Quantity values and derivation metrics are finite. Units are exact UCUM strings; a unit-bearing and a dimensionless quantity may not be compared unless the assessor explicitly defines conversion. `source_evidence`, threshold provenance, lineage IDs and warnings sort bytewise; empty `source_evidence` is invalid for an assessed strength. Evidence records sort by target discriminant + target ID, source kind, source ID, field path, evidence ID. Evidence references must resolve exactly once in the bundle.

`EvidencePairKey` is the sole V1 representation of an unordered evidence pair. Its invariant is `left_evidence_id < right_evidence_id` in the existing stable bytewise serialized `EvidenceId` ordering. Thus `(A,B)` and `(B,A)` are the same logical pair. `canonical_pair(a,b)` rejects `a == b` and returns the ascending key. Every `EvidenceIndependenceAssessment` and every `TimescalePairUncertainty` uses this exact type and convention; neither may introduce left/right fields or an alternative pair ordering. `TimescaleCovarianceUse::IndependenceBasedZeroCovariance` is required when a timescale strength is assessed using the approved Independent zero-covariance rule; no other strength derivation may claim that use.

### 5.1 Combination validator

The validator returns one typed error per invalid rule: `MissingEvidenceCombination`, `NotApplicableEvidenceCombination`, `AssessedStrengthMissingSource`, `AssessedStrengthMissingDerivation`, `InvalidEvidenceReference`, `QuantityAvailabilityConflict`, `NonFiniteEvidenceValue`, `DuplicateEvidenceId`, `UnknownEvidenceReference`, `SelfIndependenceComparison`, `NonCanonicalEvidencePair`, `DuplicateEvidencePair`, `EvidenceIndependenceMismatch`, `DuplicateTimescalePairUncertainty`, `NonCanonicalTimescalePair`, `UnknownTimescaleEvidenceReference`, `InvalidTimescaleCovarianceSource`, or `TimescaleCovarianceUnitMismatch`.

The builder and deserializer return these typed variants through the closed `EvidenceBundleError` contract:

```rust
pub enum EvidenceBundleError {
    MissingEvidenceCombination, NotApplicableEvidenceCombination,
    AssessedStrengthMissingSource, AssessedStrengthMissingDerivation,
    InvalidEvidenceReference, QuantityAvailabilityConflict, NonFiniteEvidenceValue,
    DuplicateEvidenceId, UnknownEvidenceReference, SelfIndependenceComparison,
    NonCanonicalEvidencePair, DuplicateEvidencePair, EvidenceIndependenceMismatch,
    DuplicateTimescalePairUncertainty, NonCanonicalTimescalePair,
    UnknownTimescaleEvidenceReference, InvalidTimescaleCovarianceSource,
    TimescaleCovarianceUnitMismatch,
}
```

| Condition | Required values | Failure |
|---|---|---|
| `availability=Missing` | `strength=NotAssessed`, `quantity=None`, `direction=Neutral`, `validity=NotAssessed` | `MissingEvidenceCombination` |
| `availability=NotApplicable` | `direction=NotApplicable`, `strength=NotAssessed`, `quantity=None`, `validity=NotAssessed` | `NotApplicableEvidenceCombination` |
| `availability=Available` | source resolves, direction is not `NotApplicable` | `MissingEvidenceCombination` |
| assessed strength | `strength_source != NotAssessed` and derivation exists | `AssessedStrengthMissingSource/Derivation` |
| `strength=NotAssessed` | source is `NotAssessed` and derivation is None | `AssessedStrengthMissingSource` |
| `validity=OutsideDomain or Invalid` | record is retained but excluded from promotion/counts | assessment exclusion |

`strength_source=PreservedProducerAssessment` requires an artifact field recording producer algorithm/version and threshold provenance. `MechanismAssessor` and `HealthAssessor` require a registered algorithm ID in this document. Missing configuration produces `NotAssessed`, never a strength.

### 5.2 Pairwise independence

Independence is a relation owned by `EvidenceBundle`, never a unary `EvidenceRecord` field. Every assessment serializes an `EvidencePairKey`; reversed keys are invalid, not distinct. Both IDs must exist exactly once in the same bundle, must differ, and no pair may occur twice.

For records A and B, the builder recomputes §3.2 lineage resolution. It yields `SameSource` only when both `EvidenceArtifactSource` values are `Known` and their artifact IDs are equal. If either source is `LegacyUnknown`, the result is `Unknown` permanently, even when the other source is complete and even when a second complete source is compared. With distinct known sources, it yields `Unknown` if either closure is not `Complete`, either resolved family state is `Unknown`, or either known family set is empty. It yields `PartiallyDependent` if complete ancestor closures intersect or both known family sets intersect. It yields `Independent` only if sources differ, both closures are complete, both family states are `Known` with nonempty sets, and both ancestor and family sets are disjoint. The builder records sorted intersections and every applicable typed reason. The validator recomputes this algorithm and rejects a serialized classification, status, intersection, or reason that disagrees with it as `EvidenceIndependenceMismatch`; callers cannot serialize an arbitrary `Independent`.

For a requirement needing N independent supporting evidence items, form a graph whose vertices are eligible supporting `EvidenceRecord`s and whose edges are the recomputed pair classifications equal to `Independent`. V1 performs deterministic exhaustive subset search: enumerate subsets in descending cardinality and, within one cardinality, lexicographic `EvidenceId` order; select the first subset for which every pair is an Independent edge. Its cardinality is the independent-confirmation count. A missing pair caused by unresolved lineage is not an edge and those records cannot contribute. Required tests cover same artifact → `SameSource`; shared ancestor → `PartiallyDependent`; shared family → `PartiallyDependent`; complete disjoint lineage/family → `Independent`; missing ancestor → `Unknown`; unknown family → `Unknown`; reversed pair rejection; serialized/computed mismatch rejection; and A-B Independent, A-C Independent, B-C PartiallyDependent → largest subset size 2.

### 5.3 Pairwise timescale-covariance ownership, validation, and construction (A1)

`EvidenceBundle.timescale_pair_uncertainties` is the sole V1 serialized owner of normalized pairwise timescale covariance. It is a sparse collection: for each canonical `EvidencePairKey`, zero or one entry may exist, never more than one. `MechanismAnalysisReport`, `StateEstimationReport`, `ModelAnalysisReport`, a standalone covariance sidecar, a process-global cache, and assessor-performed filesystem lookup are expressly prohibited as V1 covariance owners. Those artifacts may be already-loaded sources from which an adapter extracts covariance, but the only covariance used by the V1 evidence assessor is the normalized entry persisted in `EvidenceBundle.timescale_pair_uncertainties`.

The producer/adapter boundary is labeled, not positional:

```rust
pub struct CovarianceAxisId(pub String);
pub struct CovarianceAxis {
    pub axis_id: CovarianceAxisId,
    pub source_field_path: String,
    pub quantity_kind: CovarianceQuantityKind,
    pub unit: String,
}
pub enum CovarianceQuantityKind { Parameter, State, DerivedQuantity }
pub struct LabeledCovarianceMatrix {
    pub axes: Vec<CovarianceAxis>,
    pub values: Vec<Vec<f64>>,
}
pub enum CovarianceAxisValidationError {
    EisParameterAxisCardinalityMismatch,
    DuplicateCovarianceAxisId,
    UnknownEisParameterKey,
}
```

Axis IDs are nonempty, stable, producer-owned, and unique within a matrix. For every `LabeledCovarianceMatrix`, `(element_instance_id, parameter_key)` is unique among EIS descriptors and all generated `CovarianceAxisId` values are unique; a duplicate is a typed validation failure, never first-wins or last-wins. `values` is square with `axes.len()` rows/columns; every value is finite; every unit is present and valid; and symmetry is checked with the already-approved numerical tolerance. Consumers use only exact `CovarianceAxisId` equality. They may not infer meaning from row/column position, display labels, parameter-name guessing, matrix dimension, component order, neighboring fields, or a fallback axis.

The producer may label rows and columns from its authoritative internal descriptor order at the moment the covariance is constructed. After serialization, that ordering is not a consumer contract. Stable namespaces are `eis.parameter:<element_instance_id>:<parameter_key>`, `calibration.parameter:<stable-id>`, `estimation.state:<StateId>`, `estimation.parameter:<ParameterId>`, `model.parameter:<ParameterId>`, `model.state:<StateId>`, and `derived.timescale:<stable-derived-id>`. An adapter may not generate IDs from display labels, fitted-value order, local vector index, matrix position, or generic element ID.

For EIS, the producer-owned identity is:

```rust
pub struct EisParameterIdentity {
    pub element_instance_id: String,
    pub parameter_key: EisParameterKey,
}
pub struct EisParameterKey(pub String);
```

`element_instance_id` is the canonical machine ID from the parsed circuit instance, preserving the producer’s exact case and form, such as `R1`, `C1`, `CPE1`, or `W1`. `EisParameterKey` is a producer-authoritative lowercase ASCII token. The canonical EIS axis ID is exactly `eis.parameter:<element_instance_id>:<parameter_key>`. Namespace and parameter-key tokens are ASCII lowercase; the element instance ID preserves canonical producer spelling. `:` is reserved as the namespace delimiter. Current circuit labels are the restricted `<ElementType><digits>` form accepted by `src/impedance/circuits.rs`; therefore a producer rejects any element instance ID containing `:` rather than escaping it. No alternate escaping is defined in V1.

The complete mapping for every current `src/impedance/elements.rs::ElementType` parameter is:

| Element kind | Element instance form | Parameter | Stable `parameter_key` | Unit |
|---|---|---|---|---|
| `R` | `R<n>` | `R` | `r` | `Ohm` |
| `C` | `C<n>` | `C` | `c` | `F` |
| `L` | `L<n>` | `L` | `l` | `H` |
| `W` | `W<n>` | `sigma` | `sigma` | `Ohm s^-1/2` |
| `CPE` | `CPE<n>` | `Q` | `q` | `Ohm^-1 s^alpha` |
| `CPE` | `CPE<n>` | `alpha` | `alpha` | dimensionless |
| `Wo` | `Wo<n>` | `Z0` | `z0` | `Ohm` |
| `Wo` | `Wo<n>` | `tau` | `tau` | `s` |
| `Ws` | `Ws<n>` | `Z0` | `z0` | `Ohm` |
| `Ws` | `Ws<n>` | `tau` | `tau` | `s` |
| `La` | `La<n>` | `L` | `l` | `H s^(alpha-1)` |
| `La` | `La<n>` | `alpha` | `alpha` | dimensionless |
| `Gw` | `Gw<n>` | `sigma` | `sigma` | `Ohm s^alpha` |
| `Gw` | `Gw<n>` | `alpha` | `alpha` | dimensionless |
| `G` | `G<n>` | `R_G` | `r_g` | `Ohm` |
| `G` | `G<n>` | `t_G` | `t_g` | `s` |
| `Gs` | `Gs<n>` | `R_G` | `r_g` | `Ohm` |
| `Gs` | `Gs<n>` | `t_G` | `t_g` | `s` |
| `Gs` | `Gs<n>` | `phi` | `phi` | dimensionless |
| `K` | `K<n>` | `R` | `r` | `Ohm` |
| `K` | `K<n>` | `tau_k` | `tau_k` | `s` |
| `Zarc` | `Zarc<n>` | `R` | `r` | `Ohm` |
| `Zarc` | `Zarc<n>` | `tau_k` | `tau_k` | `s` |
| `Zarc` | `Zarc<n>` | `gamma` | `gamma` | dimensionless |
| `TLMQ` | `TLMQ<n>` | `Rion` | `r_ion` | `Ohm` |
| `TLMQ` | `TLMQ<n>` | `Qs` | `q_s` | `Ohm^-1 s^gamma` |
| `TLMQ` | `TLMQ<n>` | `gamma` | `gamma` | dimensionless |
| `T` | `T<n>` | `A` | `a_upper` | `Ohm` |
| `T` | `T<n>` | `B` | `b_upper` | `Ohm` |
| `T` | `T<n>` | `a` | `a_lower` | dimensionless |
| `T` | `T<n>` | `b` | `b_lower` | `s` |

The empty units returned by the current element metadata are represented as `dimensionless` in this contract. Thus a CPE covariance containing both parameters must serialize, for example, as `eis.parameter:CPE1:q` and `eis.parameter:CPE1:alpha`; one `eis.parameter:CPE1` axis is invalid. The producer performs an exhaustive descriptor-to-axis mapping audit: every authoritative ordered fit-parameter descriptor maps to exactly one `EisParameterIdentity` and one covariance axis, and every covariance row/column maps back to exactly one descriptor. A descriptor count different from the axis count, a dropped descriptor, or a duplicate identity is a typed serialization failure (`EisParameterAxisCardinalityMismatch`, `DuplicateCovarianceAxisId`, or `UnknownEisParameterKey`).

The EIS production boundary is the only place allowed to use internal order: authoritative ordered fit descriptor + matching covariance row/column → `EisParameterIdentity` → canonical `CovarianceAxisId`. After labeled serialization, consumers use exact-axis lookup only. `lookup_exact(CovarianceAxisId)` returns the unique axis or no result; a wrong key, including a key differing only by parameter, never falls back to element ID, position, display name, or nearest axis.

For every entry, validation requires: (1) both IDs resolve exactly once in `EvidenceBundle.records`; (2) both resolved records are eligible positive timescale quantities for §8—Available `tau`, finite `value > 0`, exact UCUM `s`, or the existing contract's explicitly normalized compatible timescale representation; (3) the key is canonical; (4) `LogSpace` covariance is finite dimensionless log covariance and `TauSpace` covariance is finite `s^2`; and (5) a `Known` covariance source resolves as a matching identity either in `lineage_catalog` or as one of the two records' direct `EvidenceArtifactSource::Known` identities. A `LegacyUnknown` evidence source cannot provide producer-backed covariance or lineage identity. An unresolvable, mismatched, or non-producer-backed source is `InvalidTimescaleCovarianceSource`; a bad record reference is `UnknownTimescaleEvidenceReference`; a non-timescale record is the same typed validation failure; and incompatible covariance units are `TimescaleCovarianceUnitMismatch`. Duplicate canonical keys fail with `DuplicateTimescalePairUncertainty`; there is no first-wins, latest-wins, merging, averaging, or precedence.

There is exactly one production construction route:

```text
already-loaded producer artifacts
        ↓
artifact-specific evidence adapters
        ↓
EvidenceRecords + optional producer-backed pair covariance
        ↓
EvidenceBundleBuilder
        ↓
validated EvidenceBundle
```

The normative conceptual interface is:

```rust
pub struct EvidenceBundleBuilder { /* bundle state */ }
impl EvidenceBundleBuilder {
    pub fn add_record(&mut self, record: EvidenceRecord);
    pub fn add_independence_assessment(&mut self, assessment: EvidenceIndependenceAssessment);
    pub fn add_timescale_pair_uncertainty(&mut self, uncertainty: TimescalePairUncertainty);
    pub fn build(self) -> Result<EvidenceBundle, EvidenceBundleError>;
}
```

The builder canonicalizes pair keys before insertion, rejects duplicate canonical pairs, validates evidence references, covariance units, and covariance provenance, and deterministically sorts every serialized collection. It may accept a reversed in-memory pair and canonicalize it; a serialized bundle must already be canonical and validation rejects a reversed pair. The builder cannot invent covariance. An adapter may create an entry only when an already-loaded producer artifact explicitly serializes covariance connecting the exact two quantities represented by the records: e.g., a parameter covariance-matrix entry, state/parameter covariance, fit covariance, or another explicitly serialized scientific covariance field. The adapter records the exact left field, right field, and covariance field; it must never infer covariance from parameter-name similarity, vector position, display/component names, nearest timescale, same artifact alone, or default zero.

There are exactly two permitted covariance routes. Route 1 is direct labeled extraction: resolve both exact source `CovarianceAxisId` values and read the corresponding matrix entry, recording `PreservedProducerCovariance` or `ExtractedCovarianceMatrixEntry`. Route 2 is a registered analytic delta method. For registered `tau1=g1(p)` and `tau2=g2(p)` definitions with labeled covariance `Σ`, the adapter computes `Var(tau1)=J1ΣJ1ᵀ`, `Var(tau2)=J2ΣJ2ᵀ`, and `Cov(tau1,tau2)=J1ΣJ2ᵀ`, records `DeltaMethodDerivedCovariance`, source axis IDs, field paths, algorithm ID/version, Jacobian, units, and derived covariance, and validates all exact source axes. `TimescaleJacobianDefinition` is analytical and registry-owned; a consumer may not invent a finite-difference Jacobian or an unregistered transformation.

The collection serializes sorted by `pair.left_evidence_id`, then `pair.right_evidence_id`, never by artifact, magnitude, insertion order, or field path. The canonical `EvidenceBundle` semantic hash is the SHA-256 of the §3.3 RFC 8785 canonical-JSON semantic hash view of the complete bundle, after all required sorting. It includes this sorted collection, including pair IDs, covariance variant and value, source artifact identity, all source field paths, and derivation. Therefore any scientific pair-covariance change changes the bundle identity. Pair covariance and independence are orthogonal collections: the former does not encode independence and the latter does not encode covariance; for every assessed pair the assessor resolves both collections with the same `EvidencePairKey`.

No public `EvidenceBundle` schema has shipped in this repository, so no backward migration is required before A1 creates its first public schema. EvidenceBundle V1 serializes `timescale_pair_uncertainties` explicitly. If a future implementation supports an older representation without that field, it migrates it exactly to `timescale_pair_uncertainties=[]`; it fabricates no covariance, and a dependent pair consequently becomes `NotAssessed / JointUncertaintyUnavailable` where §8 requires covariance.

**F4-AC-01:** the only V1 answer to “where is `TimescalePairUncertainty` persisted and how is the exact covariance for an `EvidenceId` pair found at runtime?” is: it is persisted in `EvidenceBundle.timescale_pair_uncertainties` and found by `lookup_exact(canonical_pair(left_evidence_id, right_evidence_id))`.

## 6. Deterministic hypothesis lifecycle (B)

```rust
pub enum HypothesisEvidenceLevel { Unassessed, Hypothesized, ExperimentallySupported, ValidatedForDomain }
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
pub enum HypothesisGateApplicability<T> {
    Required(T),
    NotApplicable { reason: String },
}
pub struct TimescaleGateDefinition {
    pub evidence_requirement_ids: Vec<EvidenceRequirementId>,
    pub minimum_strength: EvidenceStrength,
}
pub struct EvidenceQuantitySelector { pub source_class: EvidenceSourceClass, pub field_path: String }
pub struct AmplitudeGateDefinition {
    pub predicted_quantity: EvidenceQuantitySelector,
    pub observed_quantity: EvidenceQuantitySelector,
    pub amplitude_floor: f64,
    pub maximum_relative_amplitude_error: f64,
}
pub struct RepeatabilityGateDefinition {
    pub evidence_requirement_ids: Vec<EvidenceRequirementId>,
    pub minimum_replicates: usize,
    pub maximum_log_tau_standard_deviation: f64,
}
pub struct HypothesisEvidenceRequirement {
    pub requirement_id: EvidenceRequirementId,
    pub source_classes: Vec<EvidenceSourceClass>,
    pub required_direction: EvidenceDirection,
    pub minimum_strength: EvidenceStrength,
    pub require_valid: bool,
    pub require_independent_confirmation: bool,
    pub minimum_acquisition_families: usize,
}
pub struct MechanismHypothesisDefinition {
    pub hypothesis_id: HypothesisId, pub name: String,
    pub target_component_ids: Vec<ComponentId>,
    pub timescale_gate: HypothesisGateApplicability<TimescaleGateDefinition>,
    pub amplitude_gate: HypothesisGateApplicability<AmplitudeGateDefinition>,
    pub repeatability_gate: HypothesisGateApplicability<RepeatabilityGateDefinition>,
    pub required_evidence: Vec<HypothesisEvidenceRequirement>,
    pub critical_evidence_requirement_ids: Vec<EvidenceRequirementId>,
    pub required_identifiability_requirement_ids: Vec<RequirementId>,
    pub validation_domain: Option<ValidationDomain>, pub validation_protocol_id: Option<String>,
}
pub struct HypothesisAssessmentEvent {
    pub assessment_run_id: String,
    pub previous_level: HypothesisEvidenceLevel, pub new_level: HypothesisEvidenceLevel,
    pub reason_codes: Vec<HypothesisReasonCode>, pub evidence_bundle_hash: String,
    pub assessed_at: Timestamp,
}
pub struct HypothesisAssessmentRecord {
    pub definition: MechanismHypothesisDefinition,
    pub current: HypothesisAssessment,
    pub history: Vec<HypothesisAssessmentEvent>,
}
pub struct ValidationDomain { pub domain_id: String, pub declared_applicability: String }
pub struct ValidationProtocol { pub protocol_id: String, pub acceptance_criteria: Vec<ValidationAcceptanceCriterion> }
pub struct ValidationAcceptanceCriterion { pub criterion_id: String, pub passed: bool, pub threshold: ThresholdProvenance }
```

All IDs, names, selector field paths, and `NotApplicable.reason` values are nonempty. Requirement IDs are unique within a definition; every gate-reference and critical-requirement ID resolves to exactly one `required_evidence` entry; every listed target component ID is unique. `EvidenceQuantitySelector` selects exactly one eligible quantity by matching its source class and field path; zero or multiple matches make the gate `NotAssessed`. `InterpretationStatus` is the existing immutable type at `src/model/component.rs` and is copied unchanged. `MechanismAnalysisReport` is the existing repository type at `src/results/mechanism.rs::MechanismAnalysisReport`; B changes its existing `hypotheses` field to `Vec<HypothesisAssessmentRecord>` and preserves its other declared fields.

Every gate is serialized as `Required` or `NotApplicable`; it is never an `Option`, and applicability is determined only by that serialized variant. A `Required` timescale gate evaluates its listed requirements at the stated minimum strength. A `Required` amplitude gate selects the declared predicted and observed quantities, requires finite compatible units and `amplitude_floor > 0`, and evaluates `abs(A_predicted-A_observed)/max(abs(A_predicted),abs(A_observed),amplitude_floor) <= maximum_relative_amplitude_error`, with finite nonnegative maximum. A `Required` repeatability gate selects valid positive tau quantities for its stated requirements, requires at least `minimum_replicates >= 2`, and evaluates the sample standard deviation of `ln(tau_s / 1 second)` against the finite nonnegative maximum. Any missing, invalid, ambiguous, or insufficient required input makes that required gate `NotAssessed`; a present value above a required maximum fails it.

Each requirement applies its declared source classes, direction, strength, validity, and minimum acquisition families. If it requires independent confirmation, use the §5.2 exhaustive clique count; otherwise count eligible records without the pairwise clique criterion. `minimum_acquisition_families` is finite by type and must be at least one. A critical requirement is exactly one whose ID is listed in `critical_evidence_requirement_ids`; every listed one is evaluated. A critical requirement in `Missing`, `NotAssessed`, `Invalid`, or `OutsideDomain` blocks `ExperimentallySupported` and `ValidatedForDomain`. V1 defines no exception field for those states, so no exception is permitted. A valid critical contradiction is one `Strong` contradictory record or the configured `critical_moderate_contradiction_count: usize >= 1` `Moderate` contradictory records, each from the mutually-independent subset required by the corresponding requirement. The configured count is provenance-bearing; missing it is `Unassessed`.

Every `required_identifiability_requirement_id` resolves to an assessment defined in §9, and all listed assessments must be `Satisfied` for `ExperimentallySupported` or higher. `NotAssessed`, `NotSatisfied`, and `NotApplicable` block promotion. `Hypothesized` requires a definition and one valid available association. `ExperimentallySupported` requires Hypothesized plus every required evidence requirement, required gate, critical requirement, and listed identifiability requirement to pass; a required timescale gate does not alone satisfy a non-timescale evidence requirement. `ValidatedForDomain` additionally requires `ExperimentallySupported`, a `ValidationDomain`, a nonempty matching `validation_protocol_id`, `minimum_validation_acquisition_families >= 1`, every matching `ValidationAcceptanceCriterion.passed=true`, and no critical contradiction. No missing protocol authorizes validation.

`MechanismAnalysisReport` is the durable owner of hypothesis history. With no prior mechanism artifact, each current hypothesis starts with `history=[]`, recomputes `current`, then appends exactly one event from `Unassessed` to the current level. With `mechanism compare --prior-mechanism-artifact <PATH>`, the runner validates the prior artifact contract and experiment/sensor/channel scope, matches records by stable `HypothesisId`, copies matched history, recomputes `current` from current evidence, and appends exactly one event from the prior current level (or `Unassessed` if no matching prior record) to the new level. Events are append-only in the newly produced artifact. Prior artifacts lacking history deserialize as `history=[]` and fabricate no event. Current configuration owns the new report: hypotheses absent from it are not carried forward; an unknown prior ID does not initialize a current hypothesis. A future configuration may retain or retire one only through an explicit serialized migration/retirement policy; V1 defines none. Recalculation can increase or decrease level; failure by any required current gate produces the lower level and its event records the corresponding reason codes.

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

Point, window, and event temporal joins require `EvidenceExperimentScope::Single` on both sides with the same `ExperimentId`. `Aggregate` evidence is not point-temporally joinable. It may participate only after narrowing from the exact selected member record/field under §5, and the resulting evidence preserves the member ID, source field path, and `MemberRecord` derivation. `Unknown` scope returns `MissingEvidence` and `Indeterminate`. Scope narrowing by aggregate membership alone is rejected. For non-experiment sensor/channel scopes, Specific/Specific values must equal; Specific/All and All/All are compatible; Unspecified requires two `ScopeManifestBinding` entries with equal concrete values, otherwise identity is Unresolved. The result is MissingEvidence + Indeterminate and cannot produce strong evidence. Equilibrium values are the existing `src/model/equilibrium_recognition.rs::EquilibriumStatus` variants.

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

All values are required: `0.5<confidence_level<1`; `0<=strong<=moderate<=weak`; all resolution thresholds `>0`. Each appears once in provenance with the named field and unit `1`. Missing/invalid configuration means strength NotAssessed. `confidence_level_tolerance = 1e-12` is numerical identity tolerance only, not a scientific threshold; `numerical_variance_tolerance = 1e-15` is numerical roundoff tolerance only.

All timescale quantities assessed here have `quantity=tau`, exact UCUM unit `s`, and finite `value=tau_s>0`; `ln_tau_s = ln(tau_s / 1 second)` is dimensionless. The closed V1 `EvidenceUncertaintyModel` is defined in §5. `None` or absent uncertainty is `NotAssessed` whenever this contract requires uncertainty; there is no point-estimate fallback. `ExplicitLogInterval` requires finite bounds with `lower_ln_tau_s <= upper_ln_tau_s` and `0.5 < confidence_level < 1.0`; its bounds are a two-sided central confidence/credible interval for `ln(tau / 1 second)` at exactly its `confidence_level`. It is not transformed to another confidence level: `abs(interval.confidence_level-config.confidence_level) > 1e-12` produces `NotAssessed` with `ConfidenceLevelMismatch`. `LogNormal.value` is the geometric-mean/median tau and its finite dimensionless variance is nonnegative. `DeltaMethodTauVariance.value` is positive tau and its finite `variance_tau_s2` is nonnegative with units `s^2`; convert it to `Var[ln(tau)] = variance_tau_s2/tau_s^2`.

```rust
pub struct TimescalePairUncertainty {
    pub pair: EvidencePairKey,
    pub covariance: TimescaleCrossCovariance,
    pub source: TimescalePairUncertaintySource,
}
pub struct TimescalePairUncertaintySource {
    pub source_artifact: EvidenceArtifactSource,
    pub left_source_field_path: String,
    pub right_source_field_path: String,
    pub covariance_source_field_path: String,
    pub derivation: PairCovarianceDerivation,
}
pub enum PairCovarianceDerivation {
    PreservedProducerCovariance,
    ExtractedCovarianceMatrixEntry,
    UnitConvertedProducerCovariance,
    DeltaMethodDerivedCovariance,
}
pub struct TimescaleDerivedQuantityDefinition {
    pub derived_axis_id: CovarianceAxisId,
    pub algorithm_id: String,
    pub source_axis_ids: Vec<CovarianceAxisId>,
    pub output_unit: String,
    pub jacobian: TimescaleJacobianDefinition,
}
pub struct TimescaleJacobianDefinition {
    pub source_axis_ids: Vec<CovarianceAxisId>,
    pub coefficients: Vec<f64>,
    pub units: Vec<String>,
}
pub enum TimescaleCrossCovariance {
    LogSpace { covariance_ln_tau: f64 },
    TauSpace { covariance_tau_s2: f64 },
}
```

`TimescalePairUncertainty` is present only for producer-backed covariance; it has no optional covariance field and V1 defines no `UserAssumed`, `DefaultZero`, or `EstimatedByMechanismAssessor` derivation. Its `pair`, source, finite covariance, unit compatibility, cardinality, referential integrity, serialization, and validation behavior are exactly §5.3. `TauSpace` conversion is `Cov[ln(tau1),ln(tau2)] ≈ covariance_tau_s2/(tau1*tau2)`; its covariance has units `s^2`.

The supported pair matrix is exhaustive and model-pair rows are unordered; the calculation always preserves the serialized left/right evidence orientation:

| Left + right models | Required independence/covariance | Result |
|---|---|---|
| `None` + any | n/a | `NotAssessed` when uncertainty is required |
| `ExplicitLogInterval` + `ExplicitLogInterval` | both confidence levels equal configured level within `1e-12`; relation `Independent` | use conservative interval subtraction |
| `ExplicitLogInterval` + `ExplicitLogInterval` | `SameSource`, `PartiallyDependent`, or `Unknown` | `NotAssessed: JointUncertaintyUnavailable` |
| `LogNormal` + `LogNormal` | `Independent`: derived covariance=0; dependent: exact serialized LogSpace or TauSpace covariance; `Unknown`: unsupported | normal-pair direct ratio, otherwise `NotAssessed` |
| `DeltaMethodTauVariance` + `DeltaMethodTauVariance` | same as LogNormal after log-variance conversion | normal-pair direct ratio, otherwise `NotAssessed` |
| `LogNormal` + `DeltaMethodTauVariance` | same as LogNormal after log-variance conversion | normal-pair direct ratio, otherwise `NotAssessed` |
| `ExplicitLogInterval` + `LogNormal` | `Independent` and explicit confidence equal configured level within `1e-12` | transform normal to configured central interval; independent interval subtraction |
| `ExplicitLogInterval` + `DeltaMethodTauVariance` | `Independent` and explicit confidence equal configured level within `1e-12` | transform delta-normal to configured central interval; independent interval subtraction |
| either mixed interval/distribution pair | `SameSource`, `PartiallyDependent`, or `Unknown` | `NotAssessed: MixedModelJointUncertaintyUnavailable` |

For normal-pair direct ratio, `alpha=1-confidence_level`, `z=standard_normal_quantile(1-alpha/2)` using IEEE-754 double inverse standard-normal CDF, `mu_r=mu1-mu2`, `Var[r]=Var1+Var2-2Cov12`, and `[r_low,r_high]=[mu_r-z*sqrt(Var[r]), mu_r+z*sqrt(Var[r])]`. If `-1e-15 <= Var[r] < 0`, clamp to zero; if it is less than `-1e-15`, return typed `InvalidTimescaleCovariance`. For an independent interval pair `[L1,U1]` for the left record and `[L2,U2]` for the right, use `[r_low,r_high]=[L1-U2,U1-L2]`. The normal side of an independent mixed pair becomes `[mu-z*sigma,mu+z*sigma]` at configured confidence, placed on its actual left or right side, before that same oriented subtraction.

For every supported pair, convert the signed ratio interval exactly: if `r_low <= 0 <= r_high`, `d_low=0` and `d_high=max(abs(r_low),abs(r_high))`; otherwise `d_low=min(abs(r_low),abs(r_high))` and `d_high=max(abs(r_low),abs(r_high))`. Strength uses only `d_high`: Strong `<=strong`; Moderate `(strong,moderate]`; Weak `(moderate,weak]`; above weak produces Neutral + NotAssessed, never causal contradiction. In the relevant window every `dt_i=t_i-t_(i-1)` must be finite positive, `effective_sampling_interval_s=max(dt_i)`, `samples_per_tau=tau/effective_sampling_interval_s`, and it passes at least minimum samples. `observation_duration_ratio=(t_last-t_first)/tau` passes at least its minimum. `mode_separation_ratio=max(tau_1,tau_2)/min(tau_1,tau_2)` passes at least its minimum.

For a pair of candidate records, the assessor performs no covariance search other than this exact algorithm:

```text
pair = canonical_pair(left_evidence.evidence_id, right_evidence.evidence_id)
independence = EvidenceBundle.independence_assessments.lookup_exact(pair)
pair_uncertainty = EvidenceBundle.timescale_pair_uncertainties.lookup_exact(pair)
apply the preceding uncertainty-pair matrix
```

`lookup_exact` returns `None` for zero matches and the unique entry for one match. More than one match is impossible in a valid bundle; deserialization/validation returns `DuplicateTimescalePairUncertainty` and invalidates the bundle. The assessor never searches by artifact ID, parameter name, field path, component ID, state ID, array location, nearest match, source-artifact heuristic, or a reversed-pair fallback beyond `canonical_pair`.

No pair-covariance entry is required merely because a pair is considered. Its absence means exactly that no producer-backed pair covariance was normalized into the bundle. For an Independent normal/delta pair where the matrix permits `Cov=0`, no entry is stored: the assessor derives zero solely from the exact `Independent` assessment and records `StrengthDerivation.timescale_covariance_use=IndependenceBasedZeroCovariance { pair }`. For a `SameSource` or `PartiallyDependent` normal/delta pair that requires covariance, `None` yields `NotAssessed / JointUncertaintyUnavailable`; it is not a validation error and it never assumes zero. For `Unknown` independence, an entry never changes the required `NotAssessed` outcome. An entry also cannot make a dependent explicit-interval combination assessable unless the preceding matrix expressly permits that pair.

MHI-T09 additionally covers: explicit+explicit independent; explicit+explicit dependent NotAssessed; lognormal+lognormal independent; dependent with/without covariance; delta+delta independent and dependent TauSpace covariance; lognormal+delta independent and dependent covariance; explicit+lognormal independent; explicit+delta independent; mixed dependent NotAssessed; confidence mismatch; Unknown independence; invalid negative resulting variance; interval crossing zero; entirely positive and negative intervals; and an interval crossing Strong/Moderate where `d_high` controls the result. F4-T01 through F4-T16 are the permanent ownership and production-path requirements in §15.1.

The A1 producer decisions are normative:

| Producer | Classification | Current evidence | A1 decision |
|---|---|---|---|
| EIS | A — add labeled covariance | `EisFitArtifact` stores `parameter_covariance`; `CircuitNode::get_param_names/get_param_units` and `src/results/eis.rs` own the ordered descriptor metadata at fit construction. `EisFittedParameter.element_id` is only an element-instance projection and is insufficient as an axis. | Add `LabeledCovarianceMatrix` at the EIS writer boundary using the complete §5.3 mapping and validated `eis.parameter:<element_instance_id>:<parameter_key>` axes in exact descriptor order. Keep legacy `parameter_covariance` and unlabeled artifacts readable; A1 adapters consume only the labeled field. |
| calibration | A — producer covariance exists, but pair covariance unavailable in A1 V1 | `CalibrationFitStatistics.parameter_covariance` is positional; `CalibrationParameter` has a `name` but no stable producer-owned parameter ID contract | Do not invent IDs from names. Keep old covariance readable, but declare calibration pair covariance unavailable in A1 V1 until a stable serialized calibration parameter-ID contract exists. |
| estimation | A — add labels only where stable IDs resolve | report covariance is positional; model/component contracts provide `StateId`/`ParameterId` only when the serialized model definition/bindings are present | Add labels from exact `estimation.state:<StateId>` / `estimation.parameter:<ParameterId>` bindings at covariance construction. If any required axis cannot resolve to a serialized stable ID, the adapter emits no pair covariance. |
| model | C — not consumed for pair covariance | `ModelCompilationArtifact`/`ModelAnalysisReport` expose model definitions and identifiability metadata but no serialized scientific covariance matrix | Do not add labeled covariance or pair-covariance adapters for model artifacts in A1. |

Legacy EIS, calibration, and estimation positional matrices remain readable for compatibility, but an A1 evidence adapter must treat them as unlabeled and return unavailable pair covariance. The current mechanism implementation's positional lookup is therefore an A1 implementation site to remove from the evidence path, not an approved A1 semantic.

No universal EIS transformation is permitted. A1 must not apply `tau=1/(2*pi*f)` or `tau=R*C` to arbitrary features. `tau=R*C` and `tau_c=(R*Q)^(1/alpha)` remain available only through the existing producer/model semantics in `src/mechanism/timescale.rs`, after the exact circuit branch and parameter axes are authorized. No conversion is permitted for CPE, Warburg, DRT peak, arbitrary Bode feature, or generic fitted frequency without a registered current producer/model transform.

Where the existing approved R-CPE producer/model transform is applicable, its source-axis contract is explicit: `eis.parameter:CPE1:q` for `Q`, `eis.parameter:CPE1:alpha` for `alpha`, and the corresponding exact `eis.parameter:<R-instance>:r` for the resistor term. The delta-method registry must name all required source axes; it may not represent the CPE calculation as one `CPE1` axis. This paragraph does not authorize a new CPE timescale transformation and does not alter the existing CPE/Warburg/DRT restrictions.

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

Preserve current flags. Add to `mechanism compare`: `--estimation-artifact <PATH>`, `--model-artifact <PATH>`, `--lineage-catalog <PATH>`, and `--prior-mechanism-artifact <PATH>`. Add to `health assess`: `--estimation-artifact <PATH>`, `--model-artifact <PATH>`, `--mechanism-artifact <PATH>`, `--lineage-catalog <PATH>`. These are optional. Absent model/estimation yields Missing model-derived evidence; absent catalog yields Unknown independence; neither is fabricated. When supplied, `--prior-mechanism-artifact` follows the exact §6 validation, scope-match, stable-ID history-copy, recompute, and one-event append procedure. Existing `--mechanism-results` remains accepted; `--mechanism-artifact` is its additive explicit artifact alias only if the paths resolve to the same artifact identity, otherwise `ConflictingEvidenceInput`.

Both runners execute exactly: (1) parse CLI; (2) load configuration; (3) load legacy required inputs; (4) load optional model/estimation/mechanism; (5) validate artifact contracts; (6) validate experiment/sensor/channel scope; (7) load/resolve catalog if supplied; (8) build normalized EvidenceBundle; (9) assess; (10) serialize typed output; (11) render human output. A model/estimation identity conflict is typed `ConflictingEvidenceInput`. Any duplicated scientific evidence with unequal IDs is the same error; equal IDs deduplicate by ArtifactId.

MHI-T20a–g run the actual binary/runner with real serialized fixtures and flags. They assert process result, typed report, JSON, human report and forbidden conclusion absence: a timescale match alone does not prove mechanism; slow mode alone does not diagnose fouling; reference offset alone does not diagnose reference failure; missing evidence cannot be Strong; dependent evidence cannot independently confirm; out-of-domain evidence downgrades; Transitional/Disturbed/Indeterminate cannot silently support steady state.

## 11. Phase A0 schema and artifact-kind repair

Phase A0 may modify `src/domain/artifact.rs` **only** to make artifact-kind validation explicitly controlled by each `ArtifactContract`; it must not globally tighten or reinterpret kinds outside the A0 repair set. A0 modifies `src/results/artifact_contracts.rs`, that policy-aware validator, the eight affected producer/result-schema modules already listed below, and their production-path tests/fixtures. It does not modify any A1/B/C type or route named prohibited in §2.

```rust
pub enum CurrentArtifactKindPolicy { Required, PreserveLegacyOptional }
pub trait VersionedArtifact { // existing repository trait: src/domain/artifact.rs
    const ARTIFACT_KIND: ArtifactKind;
    const CURRENT_SCHEMA_VERSION: u32;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32];
    const CURRENT_ARTIFACT_KIND_POLICY: CurrentArtifactKindPolicy;
}
```

Every artifact contract declares this policy. `Required` means, at current schema only, correct kind passes and missing/wrong kind fails. `PreserveLegacyOptional` means, at current schema only, present correct kind passes, present wrong kind fails, and missing kind preserves the contract’s pre-A0 behavior. It is selected only where currently supported public compatibility requires a kind-less current-schema artifact to remain readable; never merely to pass a test. Legacy schema behavior follows only that contract’s documented migration rules. No missing-current-kind decision may follow merely from `version in LEGACY_SCHEMA_VERSIONS`.

`validate_value(contract, artifact)` is exactly:

```text
version = read schema_version
if version == contract.CURRENT_SCHEMA_VERSION:
    switch contract.CURRENT_ARTIFACT_KIND_POLICY:
        Required:
            if artifact_kind missing: return MissingArtifactKind
            if artifact_kind != contract.expected_artifact_kind: return WrongArtifactKind
            continue current-schema validation
        PreserveLegacyOptional:
            if artifact_kind present and != expected: return WrongArtifactKind
            continue current-schema validation
else if version in contract.LEGACY_SCHEMA_VERSIONS:
    apply that contract's explicitly documented legacy migration rules
else:
    return UnsupportedSchemaVersion
```

The implementation maps `MissingArtifactKind` and `WrongArtifactKind` to the repository-compatible typed `src/domain/artifact.rs::ArtifactError::IncompatibleKind { actual: None/Some(_) }`; this is a naming mapping, not a second behavior. The table is the complete policy assignment for every current `src/results/artifact_contracts.rs` contract relevant to this decision.

| Artifact kind / contract | CURRENT_SCHEMA_VERSION | LEGACY_SCHEMA_VERSIONS | CurrentArtifactKindPolicy | A0 repair set? | Compatibility reason / legacy migration |
|---|---:|---|---|---|---|
| `transient_analysis` / `TransientAnalysisReport` | 2 | `[1]` | Required | yes | schema-1 missing kind remains accepted |
| `calibration_observations` / `CalibrationObservationSet` | 2 | `[1]` | Required | yes | schema-1 missing kind remains accepted |
| `calibration_model` / `StoredCalibrationModel` | 2 | `[1]` | Required | yes | schema-1 missing kind remains accepted |
| `calibration_analysis` / `CalibrationAnalysisReport` | 2 | `[1]` | Required | yes | schema-1 missing kind remains accepted |
| `signal_analysis` / `SignalAnalysisReport` | 2 | `[1]` | Required | yes | schema-1 missing kind remains accepted |
| `mechanism_analysis` / `MechanismAnalysisReport` | 2 | `[1]` | Required | yes | schema-1 missing kind remains accepted |
| `health_assessment` / `SensorHealthAssessment` | 2 | `[1]` | Required | yes | schema-1 missing kind remains accepted |
| `health_trend` / `HealthTrendReport` | 2 | `[1]` | Required | yes | schema-1 missing kind remains accepted |
| `eis_fit` / `EisFitArtifact` | 2 | `[1,2]` | PreserveLegacyOptional | no | inspection confirms kind-less schema-2 reads today; preserve it |
| `health_baseline` / `SensorHealthBaseline` | 2 | `[1,2]` | PreserveLegacyOptional | no | inspection confirms kind-less schema-2 reads today; preserve it |
| `state_estimation` / `StateEstimationReport` | 3 | `[1,2,3]` | PreserveLegacyOptional | no | retain current accepted/rejected matrix, including current-schema missing-kind acceptance |
| `ism_model_compilation` / `ModelCompilationArtifact` | 4 | `[1,2,3,4]` | PreserveLegacyOptional | no | retain current accepted/rejected matrix, including current-schema missing-kind acceptance |
| `ism_model_analysis` / `ModelAnalysisReport` | 4 | `[1,2,3,4]` | PreserveLegacyOptional | no | retain current accepted/rejected matrix, including current-schema missing-kind acceptance |
| `ism_model_validation` / `ValidationResults` | 1 | `[1]` | PreserveLegacyOptional | no | retain current accepted/rejected matrix, including current-schema missing-kind acceptance |

The eight repair kinds have nine producer construction sites because `MechanismAnalysisReport` has `src/runners/mechanism.rs::{compare,trend}` writers. Their affected producer/result-schema modules are `src/potentiometry/transient/mod.rs`, `src/potentiometry/calibration/observations.rs`, `src/potentiometry/calibration/mod.rs`, `src/signal/mod.rs`, `src/runners/mechanism.rs`, `src/health/assessment.rs`, and `src/health/trend.rs`, plus their existing result/schema modules. For every repair-set row, MHI-T02a correct schema-2 kind passes; T02b wrong kind fails; T02c missing schema-2 kind fails; T02d documented schema-1 fixture passes; T02e unsupported version fails; T02f producer serialize→validate→reread passes. Fixtures are real JSON under the existing artifact fixture location; no fixture invents unknown data fields.

**A0-AC-COMPAT-01:** for every contract outside the A0 repair set, the pre-A0 compatibility matrix for schema version, artifact-kind present/missing, and artifact-kind correct/wrong remains unchanged. Fixture regressions specifically prove `eis_fit` and `health_baseline` retain their present correct/pass, present wrong/fail, and schema-2 missing/pass behavior.

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
| `AmplitudeGateDefinition.amplitude_floor` | f64, response unit, >0; `maximum_relative_amplitude_error` f64 ≥0 | exactly when `amplitude_gate=Required` | amplitude NotAssessed |
| `RepeatabilityGateDefinition.minimum_replicates` | usize ≥2; `maximum_log_tau_standard_deviation` f64 ≥0 | exactly when `repeatability_gate=Required` | repeatability NotAssessed |
| `maximum_timestamp_difference_s` | f64 seconds ≥0; `minimum_classified_fraction` f64 [0,1] | temporal evidence | MissingEvidence/Indeterminate |
| `minimum_fraction` | f64 [0,1] | MinimumSteadyFraction | config startup error |
| all `TimescaleEvidenceConfig` fields | §8 ranges, dimensionless except seconds intervals calculated from input | timescale | strength NotAssessed |
| `minimum_covariate_samples` | usize ≥1; `minimum_covariate_range` f64 >0; `maximum_absolute_pearson_correlation` f64 [0,1] | covariate | NotAssessed |
| `minimum_interferent_samples` | usize ≥1; `minimum_interferent_log10_range` f64 >0 | interferent | NotAssessed |
| `minimum_absolute_log10_activity_step` | f64 >0; `minimum_pre_event_points`, `minimum_post_event_points` usize ≥1 | excitation | NotAssessed |

## 15. Complete traceability matrix

| Requirement | Normative behavior | module/symbol | AC | Test IDs | objective failure | compatibility | scientific risk | phase |
|---|---|---|---|---|---|---|---|---|
| MHI-R1 | frozen contracts preserved | all | AC1 | T01a,b | forbidden causal/default output | additive | invalid science | all |
| MHI-R2 | A0 eight contracts fixed and non-A0 matrix preserved | `src/results/artifact_contracts.rs`; `src/domain/artifact.rs::validate_value`; §11 producer/result modules | AC2 / A0-AC-COMPAT-01 | T02a-f, preserved-contract fixtures | any matrix mismatch | schema1 readable; `eis_fit`/`health_baseline` unchanged | broken IO | A0 |
| MHI-R3 | outer adapter only | evidence module | AC3 | T03a,b | model imports evidence | additive | coupling | A1 |
| MHI-R4 | orthogonal evidence | evidence types | AC4 | T04a-d | invalid combo accepted | additive | fabricated strength | A1 |
| MHI-R5 | lineage closure | lineage resolver | AC5 | T05a-d | Unknown Independent | legacy Unknown | false independence | A1 |
| MHI-R6 | absence preserved | adapters | AC6 | T06a-c | missing gains value | legacy retained | false certainty | A1 |
| MHI-R7 | component/status separate | mechanism assessment | AC7 | T07a,b | component mutates | additive | identity conflation | B |
| MHI-R8 | deterministic lifecycle | hypothesis assessor | AC8 | T08a-d | gate skipped | additive | causal overclaim | B |
| MHI-R9 | exact timescale science | timescale assessor | AC9 | T09a-f, F4-T10–14,16 | wrong interval/class or wrong pair used | additive | false match | B |
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
| MHI-R22 | units/signs/variance | adapters | AC22 | T22a-d, F4-T08–09 | unit/sign violation | preserve stored | invalid math | A1–C |
| MHI-R23 | production validation | tests | AC23 | T23a-d, F4-T16 | helper replaces CLI | additive | false coverage | E |
| MHI-R24 | explainable conclusion | reports | AC24 | T24a,b | cannot reconstruct IDs | additive | unauditable | D/E |
| MHI-R25 | named strength authority | evidence | AC25 | T25a,b | raw gets strength | additive | invented authority | A1 |
| MHI-R26 | independent reviews | docs | AC26 | T26a | review absent | none | release risk | E |
| MHI-R27 | pairwise timescale covariance is durable, unique, producer-backed, and exactly retrievable | `EvidenceBundle`; `EvidenceBundleBuilder`; artifact-specific evidence adapters; mechanism timescale assessor | F4-AC-01 | F4-T01–16 | covariance has no owner, invalid pair survives, or a non-exact pair is used | V1 explicit collection; future older-field migration is `[]` only | fabricated/incorrect covariance and false agreement | A1/B |
| MHI-R28 / A1-C1 | artifact and evidence experiment scope is Single, Aggregate, or Unknown with deterministic propagation and no fabricated ID | A1 lineage/scope types; aggregate-capable producers in §3; temporal adapter | A1-C1-AC-01 | A1-T01–08 | aggregate is assigned a single fake ID, scope is inferred from record/path, or Unknown auto-joins | additive scope field; A0 payload and behavior retained | cross-experiment leakage and false temporal association | A1 |
| MHI-R29 / A1-C2 | legacy lineage is explicit and never fabricated or upgraded by reserialization | artifact readers/writers; `ArtifactLineageState`; migration boundary | A1-C2-AC-01 | A1-T09–14 | missing lineage becomes Known, identity is synthesized, or LegacyUnknown counts as independent | schema 1/2/3/4 legacy payloads remain readable with explicit Unknown lineage | false lineage closure and false independence | A1 |
| MHI-R30 / A1-C3 | covariance semantics are producer-labeled, validated, and exact-axis-only | EIS/estimation producers; covariance adapter; `LabeledCovarianceMatrix` | A1-C3-AC-01 | A1-T15–28 | consumer uses position/name/dimension or legacy unlabeled covariance | old covariance fields remain readable but unavailable to A1 | fabricated pair uncertainty and false agreement | A1 |
| MHI-R31 | registered direct/delta covariance is the only route to `TimescalePairUncertainty` | covariance registry and adapter; `PairCovarianceDerivation` | A1-C3-AC-02 | A1-T29–35 | unregistered transform, finite-difference Jacobian, or wrong axis is used | additive labeled covariance; no legacy covariance reinterpretation | invalid uncertainty propagation | A1/B |
| A1-RR-01 / MHI-R32 | every EIS fit parameter has a unique producer-owned covariance axis, including separate CPE Q and alpha axes | `src/impedance/elements.rs`; `src/impedance/circuits.rs`; `src/results/eis.rs`; EIS covariance adapter; `EisParameterIdentity`; `LabeledCovarianceMatrix` | A1-RR-01-AC-01 | EIS-AXIS-01–05 | any descriptor is dropped, duplicated, positionally inferred, or CPE Q/alpha share an axis | legacy unlabeled matrices remain readable but unavailable to A1; current labeled field is additive | wrong cross-parameter covariance and invalid uncertainty propagation | A1 |
| A1-RR-02 / MHI-R33 | lineage resolution accepts a state root, catalogs only known nodes, and deterministically resolves Known and LegacyUnknown | lineage resolver; `ArtifactLineageState`; `ArtifactLineageCatalog`; `ResolvedArtifactLineage`; `EvidenceArtifactSource` | A1-RR-02-AC-01 | LINEAGE-ROOT-01–06 | LegacyUnknown becomes RootMissing/Complete, root is self-looked-up, or an ID is fabricated | legacy lineage remains readable and Unknown; known-ID `RootMissing` is retained only for the explicit ID API | false closure and false independence | A1 |
| A1-RR-03 / MHI-R34 | acquisition family identity and Unknown state are explicit, authoritative, propagated, and used by independence | `src/domain/provenance.rs` as repository hash authority; lineage/evidence schema; `AcquisitionFamilyId`; `ArtifactAcquisitionFamilies`; `ResolvedAcquisitionFamilies` | A1-RR-03-AC-01 | ACQ-FAMILY-01–08 | family ID is undefined, guessed from path/time, encoded as magic string/empty vector, or Unknown is dropped | existing artifacts without authority remain Unknown; no fabricated family identity | false independent confirmation and cross-acquisition scientific overclaim | A1 |

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
| F4-T01 | EvidenceBundle containing pair covariance serializes and rereads with the pair entry intact. |
| F4-T02 | EvidenceBundle with no pair covariance is valid. |
| F4-T03 | One canonical pair has exactly one retrievable entry. |
| F4-T04 | Duplicate entries for the same canonical pair are rejected. |
| F4-T05 | Builder canonicalizes input orientation; serialized validator rejects noncanonical orientation. |
| F4-T06 | Pair covariance referencing an unknown EvidenceId is rejected. |
| F4-T07 | Pair covariance referencing evidence that is not a timescale quantity is rejected. |
| F4-T08 | TauSpace covariance with incompatible units is rejected. |
| F4-T09 | An unresolvable covariance source artifact is rejected. |
| F4-T10 | An Independent LogNormal/Delta pair without stored covariance uses the approved zero-covariance independence assumption and records it in StrengthDerivation. |
| F4-T11 | A PartiallyDependent normal/delta pair with one stored covariance is assessed using that exact entry. |
| F4-T12 | A PartiallyDependent normal/delta pair without stored covariance is `NotAssessed / JointUncertaintyUnavailable`. |
| F4-T13 | Unknown independence remains NotAssessed even when a covariance entry exists where the matrix requires resolved dependence. |
| F4-T14 | Covariance from a different EvidencePairKey is never used as fallback. |
| F4-T15 | Different insertion orders serialize in identical pair order and produce the identical EvidenceBundle semantic hash. |
| F4-T16 | A current producer artifact with labeled covariance serializes → rereads → exact axis lookup → evidence adapter → EvidenceBundleBuilder → `EvidenceBundle.timescale_pair_uncertainties` → mechanism assessor exact lookup; a legacy artifact with only unlabeled covariance rereads successfully but produces no pair-covariance entry and uses conservative unavailable behavior. |

### 15.1A A1 amendment test registry

| Stable ID | Exact variant / assertion |
|---|---|
| A1-T01 | Single scope uses the authoritative ExperimentId and never a synthetic ID. |
| A1-T02 | Two unique observation/member IDs produce Aggregate scope with sorted, deduplicated members. |
| A1-T03 | Identical aggregate inputs produce the deterministic `AggregateExperimentScopeId`; changed aggregation kind or member set changes it. |
| A1-T04 | Scope propagation covers same single, distinct singles, aggregate union, and Unknown dependency. |
| A1-T05 | Current calibration, baseline, health-trend, and mechanism-trend workflows match the §3 producer table. |
| A1-T06 | Aggregate evidence cannot point/window/event join. |
| A1-T07 | An exact member record with explicit experiment ID narrows Aggregate → Single and preserves field path/derivation. |
| A1-T08 | Aggregate membership without an explicit selected-record ID cannot narrow. |
| A1-T09 | Schema-1 missing lineage deserializes to LegacyUnknown. |
| A1-T10 | Supported schema-2 missing lineage deserializes to LegacyUnknown. |
| A1-T11 | Current writer emits Known only with authoritative lineage, otherwise explicit LegacyUnknown and warning/provenance. |
| A1-T12 | LegacyUnknown cannot become Independent. |
| A1-T13 | LegacyUnknown reserialization does not fabricate identity, dependencies, families, or semantic hash. |
| A1-T14 | Every §4 modified artifact row reads its listed legacy versions and rejects unsupported versions. |
| A1-T15 | Labeled covariance round-trips with exact axes and values. |
| A1-T16 | Duplicate axis IDs, dimension mismatch, nonfinite entries, missing units, and nonsymmetric matrices reject. |
| A1-T17 | Exact axis lookup succeeds only for matching `CovarianceAxisId`; wrong axis has no fallback. |
| A1-T18 | EIS producer labels authoritative fit ordering; adapter reads labels after serialize/reread. |
| A1-T19 | Legacy unlabeled EIS covariance is readable but yields unavailable pair covariance. |
| A1-T20 | Calibration positional covariance remains readable and yields unavailable pair covariance; no IDs are invented from names. |
| A1-T21 | Estimation labels resolve only from serialized stable StateId/ParameterId bindings. |
| A1-T22 | Model artifacts do not receive a scientific covariance adapter. |
| A1-T23 | Direct labeled covariance extracts the exact matrix entry and records field provenance. |
| A1-T24 | Registered delta-method covariance uses the declared analytic Jacobian and exact source axes. |
| A1-T25 | Missing source axis, invalid unit, or unsupported transform yields unavailable covariance. |
| A1-T26 | Finite-difference, positional, name-guess, dimension, and neighboring-field covariance inference is rejected. |
| A1-T27 | `PairCovarianceDerivation::DeltaMethodDerivedCovariance` preserves algorithm, Jacobian, axes, units, and derived value. |
| A1-T28 | Current producer → serialize → reread → adapter → exact lookup → EvidenceBundleBuilder → TimescalePairUncertainty is an integration test. |
| A1-T29 | CPE, Warburg, DRT, Bode, and generic fitted-frequency features do not receive universal tau conversion. |
| A1-T30 | Existing approved direct tau and explicit R-C/R-CPE transforms remain available only under registered producer/model semantics. |
| A1-T31 | Schema table is complete for every A1-modified artifact and explicitly excludes model validation. |
| A1-T32 | Traceability and failure criteria are present for A1-C1, A1-C2, and A1-C3. |
| A1-T33 | Type audit finds zero undefined amendment types. |
| A1-T34 | Algorithm audit finds zero unspecified amendment algorithms. |
| A1-T35 | Compatibility/contradiction audits find zero unspecified decisions or contradictions. |
| EIS-AXIS-01 | A valid CPE covariance containing Q and alpha serializes as distinct `eis.parameter:CPE<n>:q` and `eis.parameter:CPE<n>:alpha` axes; both exact lookups succeed and `axis(Q) != axis(alpha)`. |
| EIS-AXIS-02 | Every current EIS fit descriptor in the §5.3 mapping maps 1:1 to one covariance axis; descriptor count, axis count, and matrix dimension agree. |
| EIS-AXIS-03 | Duplicate EIS parameter identities or axis IDs are rejected with a typed validation error. |
| EIS-AXIS-04 | Lookup with a wrong parameter key returns no result and never falls back to element ID, display label, or position. |
| EIS-AXIS-05 | Legacy unlabeled EIS covariance remains readable but is ineligible for A1 pair covariance and is never interpreted by row position. |
| LINEAGE-ROOT-01 | A Known root resolves direct and transitive dependencies without requiring the root to be duplicated in `ArtifactLineageCatalog`. |
| LINEAGE-ROOT-02 | A LegacyUnknown root returns `Incomplete`, not `RootMissing`, with the exact empty-ID/Unknown-family result. |
| LINEAGE-ROOT-03 | A LegacyUnknown root has no fabricated or synthetic `ArtifactId`. |
| LINEAGE-ROOT-04 | A Known root with a missing ancestor returns `Incomplete` and the exact missing dependency ID. |
| LINEAGE-ROOT-05 | A reachable cycle returns `CycleDetected` with deterministic cycle artifact IDs and retains known ancestors. |
| LINEAGE-ROOT-06 | Evidence resolved from a LegacyUnknown root yields `Unknown` independence and cannot become `Independent` through comparison with a complete source. |
| ACQ-FAMILY-01 | A valid `AcquisitionFamilyId` round-trips after trim/canonical validation. |
| ACQ-FAMILY-02 | An empty or whitespace-only family ID is rejected. |
| ACQ-FAMILY-03 | Known family sets sort bytewise and deduplicate deterministically. |
| ACQ-FAMILY-04 | `ResolvedAcquisitionFamilies::Unknown` remains distinct from `Known([])`; the latter is invalid. |
| ACQ-FAMILY-05 | `Known(A) + Known(B)` produces the sorted unique union. |
| ACQ-FAMILY-06 | `Known(A) + Unknown` produces `Unknown`. |
| ACQ-FAMILY-07 | Two complete sources with a shared known family produce `PartiallyDependent`. |
| ACQ-FAMILY-08 | An unknown family blocks `Independent`, even when lineage ancestors are disjoint. |

`MHI-T14a-signal` through `MHI-T14i-data`, `MHI-T15a-mechanism-cli` through `MHI-T15g-health-cli`, `MHI-T16a-baseline` through `MHI-T16c-domain`, `MHI-T18a-residual` through `MHI-T18d-confounded`, `MHI-T19a-schema` through `MHI-T19f-future`, `MHI-T20a-timescale` through `MHI-T20g-steady-state`, `MHI-T22a-units` through `MHI-T22d-variance`, and `MHI-T23a-unit` through `MHI-T23d-cli` are literal individual IDs, not parameterized test names.

### 15.2 F4 ownership traceability

| Requirement | Acceptance criterion | Implementation owner | Serialized owner | Construction path | Lookup path | Test IDs | Failure criterion | Compatibility implication | Scientific risk | Phase |
|---|---|---|---|---|---|---|---|---|---|---|
| MHI-R9 | AC9: the timescale matrix uses only the exact covariance for the assessed canonical pair | B mechanism timescale assessor | `EvidenceBundle.timescale_pair_uncertainties` | A1 artifact-specific evidence adapters → A1 EvidenceBundleBuilder | `canonical_pair` → exact independence and covariance collection lookups | MHI-T09a-f, F4-T10–14,16 | wrong pair, fallback, unknown-dependence override, or missing-required covariance is used | additive V1 collection; no historical covariance is fabricated | false agreement or causal overclaim | A1/B |
| MHI-R22 | AC22: covariance representation and units remain dimensionally valid | A1 evidence adapters + EvidenceBundleBuilder | `EvidenceBundle.timescale_pair_uncertainties` | producer explicit covariance field → adapter records source/units → builder validates | exact key lookup after unit-valid bundle construction | MHI-T22a-d, F4-T08–09 | invalid units or unverifiable source is retained | explicit V1 field; future missing field migrates to `[]` | invalid uncertainty propagation | A1/B |
| MHI-R23 | AC23: real producer-to-assessor execution—not helper-only coverage—uses normalized covariance | A1 adapters/Builder and B timescale assessor | `EvidenceBundle.timescale_pair_uncertainties` | serialized producer artifact → adapter → builder → validated bundle | B assessor `lookup_exact(canonical_pair(...))` | MHI-T23a-d, F4-T16 | an integration path can bypass the bundle or use a helper-only result | no sidecar/global/assessor filesystem route | unwired scientific input | A1/B/E |
| MHI-R27 | F4-AC-01: one durable owner, 0-or-1 canonical entry, producer-backed provenance, deterministic serialization/hash, and exact lookup | A1 EvidenceBundleBuilder + artifact-specific evidence adapters; B mechanism timescale assessor consumes | `EvidenceBundle.timescale_pair_uncertainties` | already-loaded producer artifact → adapter → builder → validated bundle | exact canonical `EvidencePairKey`, no heuristics | F4-T01–16 | duplicate, noncanonical, unknown/non-timescale reference, invalid source/unit, or non-exact selection survives | V1 field explicit; pre-public-schema no migration; future old form is `[]` | fabricated covariance, false independence, or non-reproducible conclusion | A1/B |

### 15.3 Re-review defect closure traceability

| Finding | Exact requirement ID | Acceptance criterion | Implementation modules / public schema | Test IDs | Explicit failure criterion | Compatibility impact | Scientific risk | Phase |
|---|---|---|---|---|---|---|---|---|
| A1-RR-01 | `A1-RR-01-AC-01` | `EisParameterIdentity` maps every current descriptor exactly once; `CPE<n>:q` and `CPE<n>:alpha` serialize as distinct exact axes; duplicate and count mismatch fail | `src/impedance/elements.rs`, `src/impedance/circuits.rs`, `src/results/eis.rs`; `EisParameterIdentity`, `EisParameterKey`, `CovarianceAxisId`, `LabeledCovarianceMatrix` | EIS-AXIS-01–05 | dropped/duplicated descriptor, shared element-only axis, positional/name fallback, or wrong-key lookup succeeds | legacy unlabeled EIS covariance remains readable and unavailable to A1; current labeled covariance is additive | cross-parameter covariance can be assigned to the wrong scientific quantity | A1 |
| A1-RR-02 | `A1-RR-02-AC-01` | `resolve_lineage(&ArtifactLineageState, ...)` returns the exact Known/LegacyUnknown result; root self-duplication is optional; catalog stores known nodes only; RootMissing exists only on explicit known-ID lookup | lineage resolver and migration boundary; `ArtifactLineageState`, `ArtifactLineageCatalog`, `ArtifactLineageNode`, `ResolvedArtifactLineage`, `LineageResolutionReason`, `EvidenceArtifactSource` | LINEAGE-ROOT-01–06 | LegacyUnknown is RootMissing/Complete, a root is required in catalog, a synthetic ID is created, or a cycle/missing ancestor is hidden | historical lineage remains readable as Incomplete/Unknown; known root behavior is additive | false transitive closure and false evidence independence | A1 |
| A1-RR-03 | `A1-RR-03-AC-01` | family IDs validate/canonicalize exactly; Unknown is explicit; propagation and independence follow §§3.1–3.2/5.2; authority table is followed | lineage/evidence schema; existing `src/domain/provenance.rs` SHA-256 utility for legacy fingerprints; `AcquisitionFamilyId`, `ArtifactAcquisitionFamilies`, `ResolvedAcquisitionFamilies` | ACQ-FAMILY-01–08 | undefined/magic/empty family state, guessed family authority, dropped Unknown, or Independent emitted with unknown family | artifacts lacking authoritative family identity remain Unknown; no fabricated family ID | false independent confirmation across shared or unknown acquisition sources | A1 |

## 16. Pre-review self-audit

| Review blocker | Exact plan section | Exact defined interface/algorithm | Remaining invention required? |
|---|---|---|---|
| Lineage transitive closure | §3.2 | `resolve_lineage(&ArtifactLineageState, &ArtifactLineageCatalog) -> ResolvedArtifactLineage` | NO |
| AcquisitionFamilyId | §3.1 | `pub struct AcquisitionFamilyId(pub String)` with trim/preserve-case validation | NO |
| ArtifactAcquisitionFamilies / ResolvedAcquisitionFamilies | §§3.1–3.2 | explicit `Known(Vec<AcquisitionFamilyId>)` or `Unknown` enums and union/propagation rules | NO |
| Canonical hash ownership | §3.3 | owner hash-view table | NO |
| Dependency ordering | §3.1 | role/kind/ID sorting | NO |
| Missing-ancestor behavior | §3.2 | `Incomplete` plus missing IDs | NO |
| EvidenceRecord | §5 | full struct | NO |
| EvidencePairKey | §§5, 5.3 | full struct, canonical ordering, and shared pair semantics | NO |
| EvidenceRef | §5 | full struct and resolver rule | NO |
| EvidenceBundle | §§5, 5.3 | full struct, sole covariance ownership, ordering, and hash behavior | NO |
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

### 16.1 Final normative-type audit

| Normative type/reference | Definition or exact existing source | Undefined? |
|---|---|---|
| `CurrentArtifactKindPolicy` | §11 complete enum | NO |
| `AcquisitionFamilyId` / `ArtifactAcquisitionFamilies` / `ResolvedAcquisitionFamilies` | §§3.1–3.2 complete definitions, canonicalization, and propagation | NO |
| `EisParameterIdentity` / `EisParameterKey` | §5.3 complete producer-owned EIS identity and exhaustive mapping table | NO |
| `CovarianceAxisValidationError` | §5.3 complete typed EIS axis validation errors | NO |
| `ResolvedArtifactLineage` / `LineageResolutionReason` | §3.2 complete result and deterministic reason variants | NO |
| `EvidenceArtifactSource` / `LegacySourceFingerprint` | §5 complete Known/LegacyUnknown source representation and SHA-256 locator rule | NO |
| `EvidenceIndependenceAssessment` | §5 complete struct | NO |
| `EvidenceIndependenceReason` | §5 complete enum | NO |
| `EvidenceRecord` / `EvidenceBundle` | §§5, 5.3 complete structs, sole covariance owner, and ordering | NO |
| `EvidencePairKey` / `TimescaleCovarianceUse` | §5 complete shared-key and derivation-recording types | NO |
| `EvidenceUncertaintyModel` | §5 complete closed enum | NO |
| `TimescalePairUncertainty` / `TimescaleCrossCovariance` | §§5.3, 8 complete types, sole owner, units, and exact lookup | NO |
| `TimescalePairUncertaintySource` / `PairCovarianceDerivation` | §8 complete producer-backed provenance types | NO |
| `EvidenceBundleBuilder` / `EvidenceBundleError` | §5.3 conceptual interface, construction behavior, and typed errors | NO |
| `MechanismHypothesisDefinition` | §6 complete struct | NO |
| `HypothesisAssessmentRecord` / `HypothesisAssessmentEvent` | §6 complete structs | NO |
| `HypothesisGateApplicability` | §6 complete generic enum | NO |
| `TimescaleGateDefinition` / `AmplitudeGateDefinition` / `RepeatabilityGateDefinition` | §6 complete structs | NO |
| `HypothesisEvidenceRequirement` / `EvidenceRequirementId` | §§5–6 complete types | NO |
| `MechanismAnalysisReport` | existing `src/results/mechanism.rs::MechanismAnalysisReport`; §6 names the sole changed field | NO |
| `ArtifactKind`, `VersionedArtifact`, `ArtifactError` | existing `src/domain/artifact.rs` | NO |
| `IdentifiabilityAssessmentStatus` / `IdentifiabilityAssessment` | §9 complete types | NO |

### 16.2 Final contradiction audit

| Topic | Normative sections checked | Contradiction found? | Resolution |
|---|---|---|---|
| A0 allowed files | §§2, 11, 15, 19 | NO | validator explicitly limited to policy-aware validation |
| A0 prohibited files | §§2, 11, 19 | NO | A1/B types and routes are explicitly excluded |
| schema-2 artifact-kind behavior | §§2, 11, 15, 19 | NO | behavior is contract policy, not legacy-list membership |
| `eis_fit` compatibility | §§1, 11, 15, 19 | NO | `PreserveLegacyOptional`, missing schema-2 kind remains accepted |
| `health_baseline` compatibility | §§1, 11, 15, 19 | NO | `PreserveLegacyOptional`, missing schema-2 kind remains accepted |
| EvidenceRecord independence | §§3, 5 | NO | no unary field is serialized |
| EvidenceBundle independence | §§3, 5, 8 | NO | canonical pair assessments own all relational decisions |
| history ownership | §§6, 10 | NO | report owns append-only per-hypothesis history |
| prior mechanism artifact | §§6, 10 | NO | input, validation, matching, and append behavior are exact |
| hypothesis gate applicability | §§6, 14 | NO | every gate is `Required` or `NotApplicable` |
| uncertainty pair behavior | §§5, 8 | NO | closed models and exhaustive pair matrix |
| F4 covariance ownership and lookup | §§5.3, 8, 15.2 | NO | EvidenceBundle is sole owner; builder is sole route; assessor uses exact canonical key only |
| F4 compatibility | §§5.3, 15.2 | NO | V1 explicit field; no pre-public-schema migration; any future old form migrates to `[]` |
| confidence semantics | §§5, 8 | NO | central interval, exact configured level, and numerical tolerance stated |
| A1-RR-01 EIS identity | §§5.3, 8, 15.1A, 15.3 | NO | complete mapping table and exact three-token axis prevent element-only CPE axes and positional fallback |
| A1-RR-02 LegacyUnknown root | §§3.1–3.2, 5, 15.1A, 15.3 | NO | state-root resolver handles LegacyUnknown; catalog remains known-only; RootMissing is ID-API-only |
| A1-RR-03 acquisition family | §§3.1–3.2, 5.2, 15.1A, 15.3 | NO | family identity, Unknown variant, producer authority, propagation, and independence all use one definition |

### 16.3 Final implementation-discretion audit

| Prior blocker | Could two compliant implementation agents make materially different scientific/interface/compatibility decisions? | Result |
|---|---|---|
| F1 A0 scope / compatibility | No: scope, per-contract table, validator branch order, and compatibility invariant are exact. | NO |
| F2 evidence independence | No: owner, canonical ordering, recomputation, and exhaustive clique algorithm are exact. | NO |
| F3 hypothesis lifecycle / applicability | No: aggregate owner, declared gates, prior load path, and promotion checks are exact. | NO |
| F4 uncertainty propagation | No: sole serialized owner, shared pair key, 0-or-1 cardinality, provenance, builder, units/references, exact lookup, missing behavior, hashing, compatibility, and production test are exact. | NO |
| A1-RR-01 EIS parameter identity | No: element instance, complete parameter table, canonical axis syntax, producer boundary, duplicate/count failures, and exact lookup are fixed. | NO |
| A1-RR-02 lineage root | No: root input, known-only catalog, Known traversal, LegacyUnknown result, cycle reporting, and RootMissing distinction are fixed. | NO |
| A1-RR-03 acquisition family | No: ID canonicalization, authoritative assignment, explicit Unknown representation, propagation, and independence rule are fixed. | NO |

### 16.4 Final planning self-review

```text
Undefined normative types: 0
Unspecified scientific algorithms: 0
Unspecified compatibility decisions: 0
Normative contradictions: 0
Implementation invention still required: no

A0 scope: unambiguous yes
Evidence independence: pairwise and serializable yes
Hypothesis history owner: defined yes
Gate applicability: fully serialized yes
Uncertainty pair matrix: complete yes
EIS per-parameter covariance axes: complete yes
LegacyUnknown root resolution: complete yes
Acquisition-family identity and propagation: complete yes
Workflow path corrected: yes
MHI-R2 traceability corrected: yes
A0 regression check: PASS — this F4 clarification changes only future A1/B planning; A0 remains artifact-contract repair only and still prohibits EvidenceBundle, TimescalePairUncertainty, evidence-adapter, mechanism-scoring, health-integration, and timescale-assessment implementation.
```

### 16.5 F4 P1 closure self-audit

```text
Where is TimescalePairUncertainty serialized?
EvidenceBundle.timescale_pair_uncertainties

How many entries may exist per unordered EvidenceId pair?
0 or 1

What key identifies the entry?
canonical EvidencePairKey

What happens with duplicate entries?
bundle validation fails

Who constructs entries?
artifact-specific evidence adapters, through EvidenceBundleBuilder

Can EvidenceBundleBuilder invent covariance?
No

How does the assessor find covariance?
exact canonical EvidencePairKey lookup

Can it search by parameter/artifact/name heuristics?
No

What happens if a dependent pair requires covariance and none exists?
NotAssessed / JointUncertaintyUnavailable

What happens for an Independent pair where the approved matrix permits zero covariance?
no stored covariance required; zero derived from proven independence and recorded in StrengthDerivation

Does covariance override Unknown independence?
No

Is pair covariance included in EvidenceBundle semantic hashing?
Yes

Is F4 behavior still implementation-defined anywhere?
No

F4 status after remediation: RESOLVED
READY_FOR_FINAL_PLAN_REVIEW = yes
```

### 16.6 A1 contract-amendment self-audit

| Audit | Result | Evidence |
|---|---:|---|
| Undefined normative types | 0 | `AcquisitionFamilyId`, `ArtifactAcquisitionFamilies`, `ResolvedAcquisitionFamilies`, `EisParameterIdentity`, `EisParameterKey`, `CovarianceAxisValidationError`, `ResolvedArtifactLineage`, `LineageResolutionReason`, `EvidenceArtifactSource`, `LegacySourceFingerprint`, `ArtifactExperimentScope`, `AggregateExperimentScopeId`, `EvidenceExperimentScope`, `EvidenceScopeDerivation`, `ArtifactLineageState`, `UnknownLineageReason`, `CovarianceAxisId`, `CovarianceAxis`, `CovarianceQuantityKind`, `LabeledCovarianceMatrix`, `TimescaleDerivedQuantityDefinition`, and `TimescaleJacobianDefinition` are defined in §§3, 5.3, and 8 or mapped to exact existing IDs. |
| Unspecified A1 algorithms | 0 | aggregate ID bytes, family ID canonicalization/union, scope propagation, member narrowing, legacy default, Known/LegacyUnknown root resolution, deterministic cycle reporting, exact axis construction/lookup, direct extraction, delta method, unsupported transform, and unavailable behavior are exact. |
| Unspecified A1 compatibility decisions | 0 | §4 and §§3.2/5 define legacy EIS readability, unlabeled covariance unavailability, legacy lineage Unknown, non-fabricated IDs/fingerprints/families, root `RootMissing` scope, and `EvidenceArtifactSource` representation; `ism_model_validation` is explicitly unchanged. |
| Normative contradictions | 0 | unique per-parameter axes agree with the complete mapping table; LegacyUnknown is state-rooted and never catalog/fabricated-ID resolved; every AcquisitionFamilyId use has one definition and Unknown is not an empty set. |
| Implementation invention still required | 0 | producer ownership, exact source fields, complete EIS table, root interface/results, family authority table, registered transform boundary, tests, failure criteria, and traceability are enumerated. |

`A1-C1-AC-01` is satisfied only when aggregate workflows use `Aggregate` or `Unknown` according to §3 and no synthetic `ExperimentId` exists. `A1-C2-AC-01` is satisfied only when every missing lineage field deserializes to explicit `LegacyUnknown` and reserialization preserves it. `A1-C3-AC-01` is satisfied only when A1 adapters consume labeled axes or return unavailable; `A1-C3-AC-02` additionally requires the direct/registered-delta production path. These acceptance criteria do not authorize mechanism scoring, health interpretation, or Phase B/C implementation.

`A1-RR-01-AC-01` is satisfied only when the complete §5.3 EIS mapping is exhaustive, every current descriptor maps 1:1 to a unique axis, CPE Q and alpha are independently addressable, and wrong-key lookup has no positional fallback. `A1-RR-02-AC-01` is satisfied only when `resolve_lineage` accepts the state root, handles Known without root self-lookup, handles LegacyUnknown as the exact Incomplete result, and reserves RootMissing for the explicit known-ID API. `A1-RR-03-AC-01` is satisfied only when the typed family ID, explicit Unknown state, producer authority table, propagation, and independence behavior are all implemented as written. These three acceptance criteria authorize planning only; they do not authorize production Rust changes in this documentation remediation.

## 17. Implementation acceptance and reporting

Each phase report gives changed files, test IDs/results, exact command output classification, compatibility fixtures, remaining known baseline failures, commit, and rollback target. No phase reports GO with a failed required test. E runs all §1 commands, full CLI negatives, migration tests, and independent Scientific and Architecture re-review against the committed plan and committed implementation.

## 18. Plan tracking and final validation

For this A1 contract amendment, the only staged path is this planning document. Before commit, execute:

```bash
git add docs/engineering_specification/model_based_mechanism_sensor_health_v1_plan.md
git ls-files --error-unmatch docs/engineering_specification/model_based_mechanism_sensor_health_v1_plan.md
git diff --cached --check
git diff --cached --stat
git diff --cached
git commit -m "docs(plan): resolve A1 scope lineage and covariance contracts"
git status --short
git rev-parse HEAD
shasum -a 256 docs/engineering_specification/model_based_mechanism_sensor_health_v1_plan.md
git hash-object docs/engineering_specification/model_based_mechanism_sensor_health_v1_plan.md
git push origin plan/mhi-v1-a1-contract-amendment
git ls-remote --heads origin plan/mhi-v1-a1-contract-amendment
```

The required post-commit state is a clean tree, no production Rust changes, an amendment commit based directly on `f6e18bfed97a399b6e20de09f7348d7ffe910c77`, and a published non-force-pushed amendment branch. The empty A1 implementation branch remains untouched and is not merged or fast-forwarded by this amendment.

## 19. Phase A0 — Artifact Contract Repair implementation prompt

Implement Phase A0 only in `/Users/xingyuwang/ProjectOngoing/rust_electroanalysis_cli`. Inspect Git status, branch, base commit, `src/domain/artifact.rs`, `src/results/artifact_contracts.rs`, all eight artifact types and nine producer construction sites in §11, existing artifact tests and fixtures. **`src/domain/artifact.rs` is in A0 scope only for contract-driven current artifact-kind validation.** Add the §11 `CurrentArtifactKindPolicy` declaration to every contract and implement the exact §11 `validate_value` algorithm. For exactly the eight A0 repair-set contracts set `CURRENT_SCHEMA_VERSION=2`, `LEGACY_SCHEMA_VERSIONS=&[1]`, and `CurrentArtifactKindPolicy=Required`. Preserve non-A0 contracts’ prior accepted/rejected matrix; in particular, retain `eis_fit` and `health_baseline` schema-2 behavior and fixture-regress present-correct/pass, present-wrong/fail, and missing/pass. Add real production-path fixtures/tests per repair-set kind: correct schema-2 kind pass, wrong-kind fail, missing-kind schema-2 fail, documented schema-1 form pass, unsupported version fail, and producer serialize → validate → reread pass. Verify every current producer output is accepted by its declared contract. Do not add durable-lineage types, `EvidenceRecord`, `EvidenceBundle`, `EvidenceIndependenceAssessment`, `HypothesisAssessmentRecord`, `TimescalePairUncertainty`, hypothesis assessment, mechanism/health assessment, or evidence CLI flags. Do not modify unrelated baseline formatting/clippy debt. Run the four §1 validation commands and classify failures as existing unrelated, A0-related, new regression, or resolved. Provide traceability to MHI-R2/T02a-f and A0-AC-COMPAT-01, exact compatibility behavior, all changed paths, commands/results, commit ID, and rollback target.

## 20. Phase B contract amendment — mechanism evidence integration

This amendment is normative for Phase B and supersedes only the Phase-B portions of §§6--10, 15--18 where it differs. It neither changes the frozen A1 lineage, `EvidenceBundle`, covariance, artifact-identity, or schema semantics, nor authorizes Phase B implementation. The current tree confirms all nine findings below: `src/mechanism_config.rs` still defaults its legacy configuration; `src/runners/evidence.rs::EvidenceBundleInputs` has no model input; `src/results/mechanism.rs` has schema 3 and no durable B assessment/history; `src/model/identifiability.rs` serializes requirements but has no mechanism assessor; `EvidenceRecord` has no temporal field; and the current CLI has only legacy mechanism flags. Therefore the classifications are **CONFIRMED** for gaps 1--9.

### 20.1 Configuration owner, TOML, and scientific thresholds (B-CONFIG)

Phase B owns `src/mechanism/config.rs::MechanismEvidenceConfig` and the loader called by `src/runners/mechanism.rs`; it does not change the legacy `ResolvedMechanismConfig` scientific defaults. The serialized root is exactly `[mechanism.evidence]` in the same TOML passed through `mechanism compare --config`; the legacy no-config fallback remains valid only for legacy comparison output and **must reject a request to perform Phase B assessment** with `MechanismEvidenceInputError::MissingMechanismEvidenceConfig`. The B raw and resolved structs use `#[serde(deny_unknown_fields)]`; every B section is required; every listed field is required and has no default. A missing section/field, non-finite number, range violation, duplicate ID, unknown field, or unsupported `schema_version != 1` is a typed configuration parse/validation error before artifacts are read.

```rust
pub struct MechanismEvidenceConfig {
    pub schema_version: u32, // exactly 1
    pub timescale: TimescaleEvidenceConfig,
    pub amplitude: AmplitudeEvidenceConfig,
    pub repeatability: RepeatabilityEvidenceConfig,
    pub temporal: TemporalJoinConfig,
    pub identifiability: IdentifiabilityGateConfig,
    pub validation: ValidationProtocolConfig,
    pub promotion: HypothesisPromotionConfig,
}
pub struct TimescaleEvidenceConfig {
    pub confidence_level: f64, pub strong_max_log_distance: f64,
    pub moderate_max_log_distance: f64, pub weak_max_log_distance: f64,
    pub minimum_observation_duration_ratio: f64, pub minimum_samples_per_tau: f64,
    pub minimum_mode_separation_ratio: f64,
}
pub struct AmplitudeEvidenceConfig { pub amplitude_floor: f64, pub maximum_relative_amplitude_error: f64, pub minimum_strength: EvidenceStrength }
pub struct RepeatabilityEvidenceConfig { pub minimum_replicates: usize, pub maximum_log_tau_standard_deviation: f64, pub minimum_independent_acquisition_families: usize }
pub struct IdentifiabilityGateConfig { pub minimum_covariate_samples: usize, pub minimum_covariate_range: f64, pub maximum_absolute_pearson_correlation: f64, pub minimum_interferent_samples: usize, pub minimum_interferent_log10_range: f64, pub minimum_absolute_log10_activity_step: f64, pub minimum_pre_event_points: usize, pub minimum_post_event_points: usize }
pub struct HypothesisPromotionConfig { pub critical_moderate_contradiction_count: usize, pub minimum_supporting_evidence: usize, pub minimum_independent_acquisition_families: usize, pub minimum_validation_acquisition_families: usize, pub evidence_level_minimum_strength: EvidenceStrength }
pub enum WindowOverlapRule { PositiveDuration }
pub enum EventIdentityRule { Exact }
pub enum ClockMismatchBehavior { Indeterminate }
pub enum ScopeMismatchBehavior { Indeterminate }
pub struct TemporalJoinConfig { pub point_tolerance_s: f64, pub window_overlap_rule: WindowOverlapRule, pub event_identity_rule: EventIdentityRule, pub minimum_classified_fraction: f64, pub minimum_equilibrium_fraction: f64, pub mixed_state_policy: MixedStatePolicy, pub clock_mismatch_behavior: ClockMismatchBehavior, pub scope_mismatch_behavior: ScopeMismatchBehavior }
pub struct ValidationProtocolConfig { pub protocol: Option<ValidationProtocol> }
pub struct ValidationProtocol { pub protocol_id: String, pub version: String, pub required_acquisition_families: usize, pub minimum_validation_acquisition_families: usize, pub required_experiment_scopes: usize, pub required_conditions: Vec<ValidationCondition> }
pub struct ValidationCondition { pub condition_id: String, pub requirement_ids: Vec<EvidenceRequirementId>, pub experiment_scope: EvidenceExperimentScope, pub sensor_scope: ScopeKey, pub channel_scope: ScopeKey }
```

The exact TOML shape is `[mechanism.evidence]` with `schema_version=1`, and mandatory child tables `[mechanism.evidence.timescale]`, `.amplitude`, `.repeatability`, `.temporal`, `.identifiability`, `.validation`, and `.promotion`. `ValidationProtocolConfig` owns `protocol` directly under the validation table. Threshold provenance is emitted for every numeric field with the TOML field path and the semantic SHA-256 of the complete `MechanismEvidenceConfig`; that hash is included in a B assessment hash.

| Threshold | equation / unit | config field | allowed range; default | boundary behavior / consumer |
|---|---|---|---|---|
| timescale distance | `r=ln(tau1/tau2)`, `d_tau=abs(r)`; 1 | `timescale.{strong,moderate,weak}_max_log_distance` | `0<=strong<=moderate<=weak`; none | upper boundary belongs to lower-distance level; §8 pair classifier |
| timescale confidence | central interval confidence; 1 | `timescale.confidence_level` | `0.5<c<1`; none | mismatch > `1e-12` is NotAssessed; numerical tolerance is not config |
| duration / sampling / separation | ratios; 1 | three remaining timescale fields | each `>0`; none | equality passes; timescale gate |
| amplitude | `abs(pred-obs)/max(abs(pred),abs(obs),floor)`; quantity unit / 1 | `amplitude.{amplitude_floor,maximum_relative_amplitude_error}` | floor `>0`, error `>=0`; none | equality passes; amplitude gate |
| repeatability | sample SD of `ln(tau/1 s)`; 1 | `repeatability.maximum_log_tau_standard_deviation` | `>=0`; none | equality passes; repeatability gate |
| repeat count | count | `repeatability.minimum_replicates` | integer `>=2`; none | fewer is NotAssessed |
| independent/supporting/contradiction counts | count | `promotion.*` excluding strength | independent/validation/supporting `>=1`; contradiction `>=1`; none | exact count passes; promotion engine |
| temporal fractions | classified/equilibrium fractions; 1 | `temporal.{minimum_classified_fraction,minimum_equilibrium_fraction}` | `[0,1]`; none | equality passes; temporal join |
| temporal tolerance | seconds | `temporal.point_tolerance_s` | `>=0`; none | equality is eligible |
| identifiability thresholds | counts, source units, `log10(activity)`, correlation 1 | every `identifiability.*` field | counts `>=1`, ranges/step `>0`, correlation `[0,1]`; none | equality passes; §20.3 assessor |

Every `ValidationCondition` ID is nonempty and unique; its requirement IDs are nonempty, sorted, and resolve in the owning hypothesis. Protocol IDs/versions are nonempty; all its counts are integers `>=1`. The closed temporal enums above are required and have no defaults. This table is the complete B scientific-threshold inventory: an unconfigured threshold never yields support.

### 20.2 Deterministic evidence-requirement binding (B-BIND)

Replace the underspecified `HypothesisEvidenceRequirement` use in §6 with the following complete B requirement. It retains the existing `EvidenceRequirementId` newtype and `EvidenceTarget`/`EvidenceSourceClass` enums from `src/evidence.rs`.

```rust
pub enum RequirementGate { Required, NotApplicable { reason: String } }
pub enum EvidenceTargetSelector { ExactTarget(EvidenceTarget), ExactComponent(ComponentId), ExactIdentifiabilityRequirement(RequirementId), ExactDerivedQuantity { target: EvidenceTarget, quantity_kind: String, unit: String } }
pub enum EvidenceDirectionRequirement { Supports, Contradicts, Neutral }
pub enum EvidenceStrengthRequirement { AtLeast(EvidenceStrength) }
pub enum EvidenceValidityRequirement { ValidOnly, ValidOrWarning }
pub struct EvidenceQuantityRequirement { pub quantity_kind: String, pub unit: String }
pub struct IndependenceRequirement { pub required: bool, pub minimum_acquisition_families: usize }
pub struct EvidenceScopeRequirement { pub experiment_scope: EvidenceExperimentScope, pub sensor_scope: ScopeKey, pub channel_scope: ScopeKey, pub temporal_required: bool }
pub struct EvidenceRequirement { pub requirement_id: EvidenceRequirementId, pub target_selector: EvidenceTargetSelector, pub source_class_selector: Vec<EvidenceSourceClass>, pub direction_requirement: EvidenceDirectionRequirement, pub minimum_strength: EvidenceStrengthRequirement, pub validity_requirement: EvidenceValidityRequirement, pub quantity_requirement: Option<EvidenceQuantityRequirement>, pub independence_requirement: IndependenceRequirement, pub scope_requirement: EvidenceScopeRequirement, pub gate: RequirementGate }
pub struct EvidencePairSelector { pub left_evidence_id: EvidenceId, pub right_evidence_id: EvidenceId }
```

All selector strings/IDs are nonempty, source classes are sorted/deduplicated and nonempty, `minimum_acquisition_families>=1`, and an exact target comparison is structural enum equality--never a display name, substring, parameter position, or heuristic. `ExactComponent` compares only `EvidenceTarget::ModelComponent(component_id)`; it does not reinterpret a hypothesis target. A timescale requirement carries `EvidencePairSelector`; it is canonicalized through `EvidencePairKey::canonical` and requires those exact two eligible record IDs. A pair is never inferred from record ordering.

Candidate selection is exactly: `EvidenceBundle.records` -> exact target selector -> source class -> `availability=Available` -> validity requirement -> direction -> required quantity kind plus exact UCUM unit -> scope/temporal eligibility -> `strength >= minimum` -> sort by `(evidence_id bytes, source.artifact sort key, source.field_path bytes)`. It retains every eligible candidate. The independent subset is the lexicographically first maximum-cardinality mutually Independent clique computed with the existing §5.2 exhaustive clique algorithm; ties are resolved by the sorted evidence-ID vector. Its count and distinct known acquisition-family count decide the `IndependenceRequirement`; an Unknown family never counts. Any other candidate subset is forbidden.

### 20.3 Identifiability assessment and production inputs (B-IDENT)

Phase B owns `src/mechanism/identifiability.rs::assess_identifiability`; A1's `src/model/identifiability.rs` remains the serializer of requirements only. The assessor emits one assessment per declared requirement, in bytewise requirement-ID order.

```rust
pub enum IdentifiabilityAssessmentStatus { Satisfied, NotSatisfied, NotAssessed, NotApplicable }
pub struct IdentifiabilityMetric { pub name: String, pub value: f64, pub unit: String, pub threshold_field: String }
pub struct IdentifiabilityAssessment { pub requirement_id: RequirementId, pub requirement_kind: IdentifiabilityRequirementKind, pub status: IdentifiabilityAssessmentStatus, pub assessor_id: String, pub assessor_version: String, pub source_artifact_ids: Vec<ArtifactId>, pub evidence_ids: Vec<EvidenceId>, pub metric: Option<IdentifiabilityMetric>, pub reasons: Vec<String> }
```

The assessor uses existing artifact fields only: `TransientAnalysisReport` retained time/event observations for `TransientExcitation` and duration; exact eligible tau `EvidenceRecord`s for duration/mode separation; `ModelAnalysisReport.model_definition` and `points[*].equilibrium` only as declared model context; and `EvidenceBundle` records/family lineage for covariate/interferent/anchor evidence. `ReferenceAnchor` requires a valid `ExternalReference` record with complete Known lineage and matching scope. For `TransientExcitation`, a selected event needs finite positive pre/post activity, an exact event ID, enough pre/post points, and `abs(log10(post/pre))` at least config. Covariate/interferent conditions use the exact §9 equations and named config fields. A present applicable input below a threshold is NotSatisfied; missing/invalid/unknown lineage input is NotAssessed. `ActivityExcitation`, `TemperatureVariation`, `RepeatedStandards`, and `AuxiliaryObservation` remain NotAssessed until a registered B assessor is added. `Custom(_)` always yields `assessor_id="identifiability.custom.not_assessed"`, `assessor_version="not_assessed.v1"`, status NotAssessed, never Satisfied. Only `RequirementGate::NotApplicable` produces NotApplicable.

### 20.4 Temporal/equilibrium metadata and join (B-TEMP)

Temporal support is additive bundle-owned metadata, never a field added to frozen A1 `EvidenceRecord`:

```rust
pub enum EvidenceTemporalSupport { Point { timestamp: Timestamp, clock: ClockBasis }, Window { start: Timestamp, end: Timestamp, clock: ClockBasis }, Event { event_id: String, start: Timestamp, end: Timestamp, clock: ClockBasis }, Aggregate, Unknown }
pub struct EvidenceTemporalMetadata { pub evidence_id: EvidenceId, pub support: EvidenceTemporalSupport, pub equilibrium_source_artifact_id: Option<ArtifactId>, pub equilibrium_field_path: Option<String>, pub phase_event_id: Option<String>, pub clock_id: Option<String> }
pub enum TemporalJoinOutcome { Eligible, Ineligible { reason: TemporalJoinReason }, Indeterminate { reason: TemporalJoinReason }, MissingEvidence { reason: TemporalJoinReason } }
pub enum TemporalJoinReason { MissingMetadata, AggregateOrUnknownSupport, ScopeMismatch, ClockMismatch, EventIdentityMismatch, PointToleranceExceeded, WindowNoOverlap, AmbiguousNearestPoint, ClassifiedFractionBelowMinimum, EquilibriumFractionBelowMinimum, MixedStateRejected, NoTargetObservations, NoClassifiedObservations }
```

`EvidenceBundle.temporal_metadata: Vec<EvidenceTemporalMetadata>` is schema-1 additive B metadata, sorted by `evidence_id`, exactly one entry per referenced evidence ID, and included in the bundle semantic hash. It is optional only for A1-created bundles; missing metadata deserializes to `[]` and is treated as MissingEvidence, never temporal support. The Phase B builder accepts it only through already loaded `TransientAnalysisReport` event/time fields and `ModelAnalysisReport.points[*].{time_s,equilibrium}`. Equilibrium is exactly `src/model/equilibrium_recognition.rs::EquilibriumStatus`; no new classifier is permitted. The classification fraction, equilibrium fraction, scope/clock conversion, point matching, window semantics, MixedStatePolicy precedence, and failure ordering remain §7; B adds the required `minimum_equilibrium_fraction` check after classified fraction and before mixed-state policy. Point/Event identities must match exact string IDs under `event_identity_rule=Exact`; window overlap uses `[max(start), min(end))` and requires positive overlap under `PositiveDuration`; clock/scope mismatch returns Indeterminate, aggregate/unknown support returns MissingEvidence, and neither can support a point conclusion.

### 20.5 Multi-component, validation, promotion, and history (B-LIFECYCLE)

V1 preserves existing multi-component definitions without collapsing component meaning. `HypothesisAssessment` contains `component_assessments: Vec<ComponentInterpretationAssessment>`, one per sorted distinct `target_component_id`; it removes the single `component_interpretation_status` field. The complete replacement types are:

```rust
pub enum ComponentInterpretationBasis { HypothesisEvidence { hypothesis_id: HypothesisId, evidence_level: HypothesisEvidenceLevel, assessment_hash: String } }
pub struct ComponentInterpretationAssessment { pub component_id: ComponentId, pub interpretation_status: InterpretationStatus, pub basis: ComponentInterpretationBasis }
pub struct HypothesisAssessment { pub hypothesis_id: HypothesisId, pub hypothesis_evidence_level: HypothesisEvidenceLevel, pub component_assessments: Vec<ComponentInterpretationAssessment>, pub supporting_evidence: Vec<EvidenceRef>, pub contradictory_evidence: Vec<EvidenceRef>, pub excluded_evidence: Vec<EvidenceRef>, pub identifiability_assessments: Vec<IdentifiabilityAssessment>, pub temporal_outcomes: Vec<TemporalJoinOutcome>, pub reason_codes: Vec<HypothesisReasonCode>, pub assessment_hash: String }
pub struct HypothesisHistoryEntry { pub history_id: String, pub hypothesis_id: HypothesisId, pub prior_level: HypothesisEvidenceLevel, pub new_level: HypothesisEvidenceLevel, pub assessment_hash: String, pub reason_codes: Vec<HypothesisReasonCode>, pub source_evidence_ids: Vec<EvidenceId>, pub sequence: u64, pub assessed_at: Option<Timestamp> }
pub struct HypothesisAssessmentRecord { pub definition: MechanismHypothesisDefinition, pub current: HypothesisAssessment }
```

The only B component effect is the existing component's transition to `Hypothesized` when the hypothesis reaches `Hypothesized`, and to `ExperimentallySupported` when it reaches that level; it never sets `ValidatedForDomain` until the validation row below passes. A component absent from the definition is untouched. A shared hypothesis result is copied to each listed component; no aggregate or inferred component status exists.

`ValidationProtocol` is embedded as `MechanismEvidenceConfig.validation.protocol` and passed only through the B config--there is no `--validation-protocol` flag. It is required when a definition names `validation_protocol_id`; otherwise `protocol=None` is permitted and validation is unavailable. Its exact fields are `protocol_id`, `version`, `required_acquisition_families`, `minimum_validation_acquisition_families`, `required_experiment_scopes`, and `required_conditions: Vec<ValidationCondition>`. A validation family is a Known `AcquisitionFamilyId` on an eligible validation-role evidence record, from an exact distinct acquisition family and experiment scope, not present in any supporting/training/calibration candidate used to establish ExperimentalSupport. Unknown lineage/family never counts. Each required condition must have an eligible supporting record in the declared scope; no missing condition passes.

| Result | deterministic requirements |
|---|---|
| Unassessed | missing B config, no valid available candidate, or any Required gate is NotAssessed/NotSatisfied |
| Hypothesized | definition exists and at least one valid Available selected record supports it |
| ExperimentallySupported | Hypothesized; all Required requirements/gates Satisfied; support count at least promotion minimum; independent known family count at least promotion minimum; critical contradiction count below configured limit; all required identifiability Satisfied |
| ValidatedForDomain | ExperimentallySupported plus matching embedded protocol, all required conditions, distinct validation families at least `max(protocol.required_acquisition_families, protocol.minimum_validation_acquisition_families, promotion.minimum_validation_acquisition_families)`, required experiment scopes, and no critical contradiction |

`MechanismAnalysisReport` owns `hypothesis_assessments: Vec<HypothesisAssessmentRecord>` and `hypothesis_history: Vec<HypothesisHistoryEntry>` in schema 4. An entry has `history_id=SHA256(hypothesis_id || NUL || prior_level || NUL || new_level || NUL || assessment_hash)`, hypothesis ID, prior/new level, assessment hash, sorted reason codes, sorted source EvidenceIds, sequence number, and optional RFC-3339 timestamp excluded from equality/hash. `assessment_hash` is RFC-8785 SHA-256 of the deterministic scientific assessment view (definition ID, requirement results, component assessments, sorted evidence IDs, config hash, protocol ID/version, identifiability, temporal outcomes); no timestamp/human text. On each run, compute then append only if no prior entry for that hypothesis has equal `(prior_level,new_level,assessment_hash)`; sequence is prior maximum plus one. History content is sorted by hypothesis ID then sequence for serialization. Current assessment and history scientific fields are included in the report semantic hash; timestamps are excluded.

### 20.6 Schema, runner inputs, CLI, and errors (B-COMPAT)

`mechanism_analysis` changes from current schema **3** to **4**, with supported legacy versions `[1,2,3]` (the existing artifact-kind policy remains `Required`). Schema-3 artifacts deserialize with `hypothesis_assessments=[]`, `hypothesis_history=[]`; their absence means NotAssessed, never support. Schema-1/2 retain their documented historical migrations. A schema-4 writer never fabricates a history entry for a legacy read. All current B assessment/history fields participate in semantic hashing as specified above; legacy writers/readers retain old hashes only for their old payload representations.

| Artifact | before / after | legacy | B fields and migration |
|---|---|---|---|
| `mechanism_analysis` | 3 / 4 | `[1,2,3]` | `hypothesis_assessments`, `hypothesis_history`; serde `[]`; missing = NotAssessed |
| `EvidenceBundle` | 1 / 1 | `[1]` | additive `temporal_metadata`; missing = `[]`, MissingEvidence only |
| `ism_model_analysis` | 5 / 5 | `[1,2,3,4,5]` | unchanged; accepted read-only B input |

`EvidenceBundleInputs` gains `model_artifact: Option<ModelAnalysisReport>` and includes its known lineage in the catalog. It accepts only artifact kind `ism_model_analysis`, schema versions `[1,2,3,4,5]` readable by its existing contract, and scope compatible with the mechanism target. `src/evidence_adapters.rs::adapt_model_analysis` maps a model point only to `source_class=ModelDerived`, its exact `ModelComponent(ComponentId)` target, direction derived solely from the declared requirement/field semantics, `availability=Available` only for finite source values, `strength=NotAssessed` until a B assessor derives it, `validity` from the point validity/domain, exact quantity/unit from the source state/contribution, producer uncertainty if serialized, model identifiability relevance only through §20.3, and artifact/member scope. Raw model records never become Strong automatically.

```rust
pub struct ArtifactReference { pub path: String, pub expected_artifact_kind: ArtifactKind, pub expected_schema_versions: Vec<u32> }
pub struct ConflictingEvidenceInput { pub artifact: ArtifactReference, pub expected_scope: EvidenceScopeRequirement, pub role: ConflictingEvidenceRole }
pub enum ConflictingEvidenceRole { EisFit, Transient, Calibration, Estimation, Model, PriorMechanism, EvidenceBundle }
pub enum MechanismEvidenceInputError { ArtifactKindMismatch { role: ConflictingEvidenceRole }, UnsupportedSchemaVersion { role: ConflictingEvidenceRole, schema_version: u32 }, ExperimentScopeConflict, SensorScopeConflict, ChannelScopeConflict, ClockScopeConflict, MissingRequiredEvidence, MissingMechanismEvidenceConfig, ConflictingEvidenceInput }
```

Any artifact scope conflict returns the named typed error and is never silently dropped. Phase B adds exactly `--mechanism-evidence-config <PATH>` (required for B assessment), `--model-artifact <PATH>` (optional), `--evidence-artifact <PATH>` (optional, at most once), `--prior-mechanism-artifact <PATH>` (optional), and `--lineage-catalog <PATH>` (optional); it retains existing `--config`, EIS, transient, calibration, metadata, and output flags. `--evidence-artifact` accepts only an `EvidenceBundle` schema 1 artifact; if present it is the only bundle source and may not be combined with `--model-artifact`/legacy evidence inputs except as an exact same-ID deduplication, otherwise `ConflictingEvidenceInput`. A missing optional model yields Missing model-derived evidence. `--validation-protocol` is expressly prohibited in B because protocol ownership is config. No Phase-E flag is introduced.

### 20.7 Permanent fixture/data matrix and traceability (B-TEST)

Phase B creates only the following tracked files during implementation; each is an exact data binding, not an instruction to invent appropriate evidence. `tests/fixtures/a1/current_labeled_covariance.json`, `legacy_unlabeled_covariance.json`, `current_known_lineage_state.json`, `aggregate_scope.json`, and `tests/fixtures/a0_artifact_contracts/schema2/{transient_analysis,mechanism_analysis}.schema2.json` are the immutable upstream source data. Generated artifacts use public `read_artifact`/`write_artifact` and the existing `assemble_evidence_bundle` route.

| Permanent test ID | exact tracked fixture or deterministic source route |
|---|---|
| `MHI-B-T00-config` | `tests/fixtures/phase_b/e2e/mechanism.toml`; assert missing child/unknown field/no-default rejection through the public config loader |
| `MHI-B-T01-timescale-independent`, `MHI-B-T01-timescale-dependent`, `MHI-B-T01-timescale-with-covariance`, `MHI-B-T01-timescale-without-covariance`, `MHI-B-T01-timescale-boundary`, `MHI-B-T01-timescale-out-of-domain` | `tests/fixtures/phase_b/timescale/{independent,dependent,with_covariance,without_covariance,boundary,out_of_domain}.json`; covariance source is `tests/fixtures/a1/current_labeled_covariance.json`; unavailable case is `legacy_unlabeled_covariance.json`; IDs are literal `b-ts-eis-01` / `b-ts-transient-01` |
| `MHI-B-T02-amplitude-sign`, `MHI-B-T02-amplitude-opposite`, `MHI-B-T02-amplitude-indeterminate` | `tests/fixtures/phase_b/amplitude/{expected_sign,opposite_sign,indeterminate}.json`; literal quantity IDs `b-amp-predicted-01`, `b-amp-observed-01` |
| `MHI-B-T03-repeat-independent`, `MHI-B-T03-repeat-dependent`, `MHI-B-T03-repeat-insufficient`, `MHI-B-T03-repeat-unknown-family` | `tests/fixtures/phase_b/repeatability/{independent,dependent,insufficient,unknown_family}.json`; literal families `b-family-a`, `b-family-b`, `b-family-shared`, and serialized `Unknown` |
| `MHI-B-T04-temporal-point`, `MHI-B-T04-temporal-window`, `MHI-B-T04-temporal-event`, `MHI-B-T04-temporal-clock-mismatch`, `MHI-B-T04-temporal-scope-mismatch`, `MHI-B-T04-temporal-aggregate-unknown` | respectively `tests/fixtures/phase_b/temporal/{point,window,event,clock_mismatch,scope_mismatch,aggregate_unknown}.json`; literal UTC timestamps `2025-01-01T00:00:00Z`, `2025-01-01T00:00:10Z`, and event `b-step-01`; equilibrium source route is `ModelAnalysisReport.points[*].equilibrium` |
| `MHI-B-T05-ident-satisfied`, `MHI-B-T05-ident-not-satisfied`, `MHI-B-T05-ident-not-assessed`, `MHI-B-T05-ident-custom-unsupported` | respectively `tests/fixtures/phase_b/identifiability/{satisfied,not_satisfied,not_assessed,custom_unsupported}.json`, each naming the §20.3 field path and literal requirement ID |
| `MHI-B-T06-validation-pass`, `MHI-B-T06-validation-insufficient`, `MHI-B-T06-validation-unknown-family`, `MHI-B-T06-validation-training-overlap` | respectively `tests/fixtures/phase_b/validation/{pass,insufficient,unknown_family,training_overlap}.json`, literal family sets and protocol count |
| `MHI-B-T07-e2e` | `tests/fixtures/phase_b/e2e/{mechanism.toml,eis_fit.json,transient.json,model_analysis.json}.json`; public readers -> `assemble_evidence_bundle` -> B assessment -> schema-4 mechanism artifact -> public reread -> exact assessment/history assertion |

| Requirement | normative behavior / public type | implementation module | AC and exact test ID | fixture/data | compatibility / scientific risk |
|---|---|---|---|---|---|
| `MHI-B-R01` | required no-default config; `MechanismEvidenceConfig` | `src/mechanism/config.rs` | `B-CONFIG-AC-01`; `MHI-B-T00-config` | `phase_b/e2e/mechanism.toml` | additive config / hidden threshold |
| `MHI-B-R02` | exact selectors, full candidate retention, exact pair IDs; `EvidenceRequirement` | `src/mechanism/evidence.rs` | `B-BIND-AC-01`; `MHI-B-T01-timescale-independent` | `phase_b/timescale/independent.json` | additive / non-deterministic support |
| `MHI-B-R03` | one conservative result per requirement; `IdentifiabilityAssessment` | `src/mechanism/identifiability.rs` | `B-IDENT-AC-01`; four literal `MHI-B-T05-ident-*` IDs above | `phase_b/identifiability/*.json` | additive / false identifiability |
| `MHI-B-R04` | bundle-owned temporal metadata and typed outcome; `TemporalJoinOutcome` | `src/mechanism/temporal.rs` | `B-TEMP-AC-01`; six literal `MHI-B-T04-temporal-*` IDs above | `phase_b/temporal/*.json` | additive field / temporal leakage |
| `MHI-B-R05` | per-component outcome; `ComponentInterpretationAssessment` | `src/results/mechanism.rs` | `B-MULTI-AC-01`; `MHI-B-T07-e2e` | `phase_b/e2e/*.json` | schema 4 / component overclaim |
| `MHI-B-R06` | config-owned protocol and family exclusions; `ValidationProtocol` | `src/mechanism/validation.rs` | `B-VALID-AC-01`; four literal `MHI-B-T06-validation-*` IDs above | `phase_b/validation/*.json` | additive config / false validation |
| `MHI-B-R07` | schema 3->4 migration and scientific hash; `HypothesisHistoryEntry` | `src/results/{mechanism,artifact_contracts}.rs` | `B-MIGRATION-AC-01`; `MHI-B-T07-e2e` | `phase_b/e2e/*.json`, existing schema2 mechanism fixture | legacy readable / fabricated support |
| `MHI-B-R08` | model adapter and typed scope errors; `ConflictingEvidenceInput` | `src/{runners/evidence.rs,evidence_adapters.rs,runners/mechanism.rs,cli.rs}` | `B-INPUT-AC-01`; `MHI-B-T07-e2e` | `phase_b/e2e/model_analysis.json` | additive input / scope-conflicted evidence |
| `MHI-B-R09` | exact test/data matrix and public E2E path | `tests/phase_b_mechanism_evidence.rs` | `B-TEST-AC-01`; `MHI-B-T01`--`MHI-B-T07` literals above | all `tests/fixtures/phase_b/**` paths above | test-only files / fixture-only coverage |

### 20.8 Phase B amendment self-audit and delivery

```text
Undefined normative types: 0
Unspecified Phase B algorithms: 0
Unspecified scientific thresholds: 0
Unspecified compatibility decisions: 0
Normative contradictions: 0
Implementation invention still required: no
```

Two compliant implementation agents cannot make materially different choices about configuration defaults, candidate/pair selection, identifiability status, temporal join, multi-component semantics, validation, migration, model input behavior, or fixture binding: all are fixed in §§20.1--20.7. This amendment modifies documentation only. Before commit, stage only this file, run `git diff --check`, inspect the cached diff, commit `docs(plan): close Phase B mechanism evidence contract gaps`, calculate its SHA-256/blob ID, and push only `plan/mhi-v1-b-contract-amendment`. Do not merge or tag it and do not change `main` or `codex/mhi-v1-b-mechanism-evidence-integration`.
