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

This amendment is normative for Phase B and supersedes only the Phase-B portions of §§6--10, 15--18 where it differs. It does not change frozen A1 lineage, covariance, or schema-1 `EvidenceBundle` semantic identity, and it does not authorize Phase B implementation. §21 is the final contradiction-remediation addendum: where it names an earlier B contract, the §21 rule replaces it. The current tree confirms all ten findings below: `src/mechanism_config.rs` still defaults its legacy configuration; `src/runners/evidence.rs::EvidenceBundleInputs` has no model input; `src/results/mechanism.rs` has schema 3 and no durable B assessment/history; `src/model/identifiability.rs` serializes requirements but has no mechanism assessor; `EvidenceRecord` has no temporal field; and the current CLI has only legacy mechanism flags. Therefore PB-RR-01 through PB-RR-10 are **CONFIRMED** before this remediation.

### 20.1 Configuration owner, TOML, and scientific thresholds (B-CONFIG)

Phase B owns `src/mechanism/config.rs::MechanismEvidenceConfig` and the loader called by `src/runners/mechanism.rs`; it does not change the legacy `ResolvedMechanismConfig` scientific defaults. Phase B configuration is supplied **only** by `mechanism compare --mechanism-evidence-config <PATH>`. That file is one TOML document whose root is `schema_version=1` followed by the exact top-level tables `[timescale]`, `[amplitude]`, `[repeatability]`, `[temporal]`, `[identifiability]`, `[promotion]`, and, when validation is available, `[validation]`. The legacy `--config` file owns legacy/general mechanism comparison configuration only; it neither contains nor overrides Phase B evidence-evaluation fields. If a B-named field appears in `--config`, it follows the existing legacy parser's unknown-field behavior; it never has Phase B effect and there is no precedence relationship. The legacy no-config fallback remains valid only for legacy comparison output and **must reject a request to perform Phase B assessment** with `MechanismEvidenceInputError::MissingMechanismEvidenceConfig`. The B raw and resolved structs use `#[serde(deny_unknown_fields)]`. Every field of every **present** B section is required and has no default; the `[validation]` section itself is optional. A missing required section/field, non-finite number, range violation, duplicate ID, unknown field, or unsupported `schema_version != 1` is a typed configuration parse/validation error before artifacts are read.

```rust
pub struct MechanismEvidenceConfig {
    pub schema_version: u32, // exactly 1
    pub timescale: TimescaleEvidenceConfig,
    pub amplitude: AmplitudeEvidenceConfig,
    pub repeatability: RepeatabilityEvidenceConfig,
    pub temporal: TemporalJoinConfig,
    pub identifiability: IdentifiabilityGateConfig,
    pub validation: Option<ValidationProtocol>,
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
pub struct HypothesisPromotionConfig { pub critical_moderate_contradiction_count: usize, pub minimum_supporting_evidence: usize, pub minimum_independent_acquisition_families: usize, pub evidence_level_minimum_strength: EvidenceStrength }
pub enum WindowOverlapRule { PositiveDuration }
pub enum EventIdentityRule { Exact }
pub enum ClockMismatchBehavior { Indeterminate }
pub enum ScopeMismatchBehavior { Indeterminate }
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MixedStatePolicy { RequireAllSteady { allow_quasi_equilibrium: bool }, MinimumSteadyFraction { minimum_fraction: f64, allow_quasi_equilibrium: bool, reject_if_disturbed: bool }, WorstCase }
pub struct TemporalJoinConfig { pub point_tolerance_s: f64, pub window_overlap_rule: WindowOverlapRule, pub event_identity_rule: EventIdentityRule, pub minimum_classified_fraction: f64, pub minimum_equilibrium_fraction: f64, pub mixed_state_policy: MixedStatePolicy, pub clock_mismatch_behavior: ClockMismatchBehavior, pub scope_mismatch_behavior: ScopeMismatchBehavior }
pub struct ValidationProtocol { pub protocol_id: String, pub version: String, pub minimum_acquisition_families: usize, pub required_conditions: Vec<ValidationCondition> }
pub enum ValidationProtocolStatus { Satisfied, NotSatisfied, NotAssessed }
pub struct ValidationCondition { pub condition_id: String, pub requirement_ids: Vec<EvidenceRequirementId>, pub experiment_scope: EvidenceExperimentScope, pub sensor_scope: ScopeKey, pub channel_scope: ScopeKey }
```

The exact TOML shape has root `schema_version=1`, mandatory child tables `[timescale]`, `[amplitude]`, `[repeatability]`, `[temporal]`, `[identifiability]`, and `[promotion]`, and optional `[validation]`. `ValidationProtocol` is the direct shape of the present validation table; there is no nested `protocol` member. `ValidationProtocol.required_experiment_scopes` is **RETIRED for Phase B V1**. Experiment-scope eligibility is expressed only by `required_conditions` and validation-family rules; there is no independent numeric experiment-scope-count threshold. Threshold provenance is emitted for every numeric field with the TOML field path and the semantic SHA-256 of the complete `MechanismEvidenceConfig`; that hash is included in a B assessment hash.

| Threshold | equation / unit | config field | allowed range; default | boundary behavior / consumer |
|---|---|---|---|---|
| timescale distance | `r=ln(tau1/tau2)`, `d_tau=abs(r)`; 1 | `timescale.{strong,moderate,weak}_max_log_distance` | `0<=strong<=moderate<=weak`; none | upper boundary belongs to lower-distance level; §8 pair classifier |
| timescale confidence | central interval confidence; 1 | `timescale.confidence_level` | `0.5<c<1`; none | mismatch > `1e-12` is NotAssessed; numerical tolerance is not config |
| duration / sampling / separation | ratios; 1 | three remaining timescale fields | each `>0`; none | equality passes; timescale gate |
| amplitude | `abs(pred-obs)/max(abs(pred),abs(obs),floor)`; quantity unit / 1 | `amplitude.{amplitude_floor,maximum_relative_amplitude_error}` | floor `>0`, error `>=0`; none | equality passes; amplitude gate |
| repeatability | sample SD of `ln(tau/1 s)`; 1 | `repeatability.maximum_log_tau_standard_deviation` | `>=0`; none | equality passes; repeatability gate |
| repeat count | count | `repeatability.minimum_replicates` | integer `>=2`; none | fewer is NotAssessed |
| independent/supporting/contradiction counts | count | `promotion.{minimum_supporting_evidence,minimum_independent_acquisition_families,critical_moderate_contradiction_count}` | supporting/independent `>=1`; moderate-contradiction allowance `>=1`; none | support/independent equality passes; the allowance is evaluated only after the unconditional strong-critical block; promotion engine |
| validation acquisition-family count | count | `validation.minimum_acquisition_families` | `>=1` when `[validation]` is present; none | equality passes; validation protocol |
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
pub struct EvidencePairRequirement { pub requirement_id: EvidenceRequirementId, pub left: EvidenceRequirement, pub right: EvidenceRequirement, pub pair_selector: EvidencePairSelector, pub gate: RequirementGate }
pub enum EvidenceRequirementBinding { Single(EvidenceRequirement), Pair(EvidencePairRequirement) }
```

All selector strings/IDs are nonempty, source classes are sorted/deduplicated and nonempty, `minimum_acquisition_families>=1`, and an exact target comparison is structural enum equality--never a display name, substring, parameter position, or heuristic. `ExactComponent` compares only `EvidenceTarget::ModelComponent(component_id)`; it does not reinterpret a hypothesis target. `EvidenceRequirementBinding` is the only serialized owner of `EvidencePairSelector`: a timescale-pair requirement is `Pair` and therefore requires exactly one selector; every single-record requirement is `Single` and therefore forbids a selector. A `Pair` whose left/right requirements are not eligible timescale quantities, whose IDs are equal, or whose selector does not resolve exactly to the selected left/right candidates fails with `MechanismEvidenceInputError::EvidenceRequirementBindingMismatch`; a selector is never inferred from record order. Resolution is exactly serialized hypothesis definition -> serialized `Pair` binding -> exact left/right candidate selection -> `EvidencePairKey::canonical(left,right)` -> `EvidenceBundle.timescale_pair_uncertainties.lookup_exact(key)`.

The support candidate selection is exactly: `EvidenceBundle.records` -> exact target selector -> source class -> `availability=Available` -> validity requirement -> scope/temporal eligibility -> required quantity kind plus exact UCUM unit -> the requirement's support direction -> `strength >= minimum` -> sort by `(evidence_id bytes, source.artifact sort key, source.field_path bytes)`. It retains every eligible support candidate. Critical contradiction selection is a separate §21.4 pipeline from the unfiltered eligible universe and never reuses this set. The independent subset is the lexicographically first maximum-cardinality mutually Independent clique computed with the existing §5.2 exhaustive clique algorithm; ties are resolved by the sorted evidence-ID vector. Its count and distinct known acquisition-family count decide the `IndependenceRequirement`; an Unknown family never counts. Any other candidate subset is forbidden.

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

`EvidenceBundle.temporal_metadata: Vec<EvidenceTemporalMetadata>` is a schema-2 B field, sorted by `evidence_id`, exactly one entry per referenced evidence ID, and included only in the schema-2 semantic hash. Schema 1 has no temporal metadata semantic field: missing metadata is not fabricated, a schema-1 read remains schema 1 on reserialization, and it yields `MissingEvidence` whenever temporal support is required. The Phase B builder accepts schema-2 metadata only through already loaded `TransientAnalysisReport` event/time fields and `ModelAnalysisReport.points[*].{time_s,equilibrium}`. Equilibrium is exactly `src/model/equilibrium_recognition.rs::EquilibriumStatus`; no new classifier is permitted. The classification fraction, equilibrium fraction, scope/clock conversion, point matching, window semantics, MixedStatePolicy precedence, and failure ordering are replaced by §21.3. Point/Event identities must match exact string IDs under `event_identity_rule=Exact`; window overlap uses `[max(start), min(end))` and requires positive overlap under `PositiveDuration`; clock/scope mismatch returns Indeterminate, aggregate/unknown support returns MissingEvidence, and neither can support a point conclusion.

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

`ValidationProtocol` is the present `MechanismEvidenceConfig.validation` table and is passed only through `--mechanism-evidence-config`--there is no `--validation-protocol` flag. `[validation]` is required when a hypothesis definition permits or requests `ValidatedForDomain`; otherwise `validation=None` is permitted. `validation=None` means `ValidatedForDomain` promotion is unavailable, `ValidationProtocolStatus=NotAssessed`, and the maximum Phase B component status is `ExperimentallySupported`; it never means validation passed. Its sole canonical shape is in §20.1 and has exactly `protocol_id`, `version`, `minimum_acquisition_families`, and `required_conditions: Vec<ValidationCondition>`. `required_experiment_scopes` is retired and MUST NOT be parsed, serialized, inventoried, or implemented. `ValidationProtocol.acceptance_criteria` and all earlier `required_acquisition_families` / `minimum_validation_acquisition_families` shapes are superseded B terminology and MUST NOT be implemented. A validation family is a Known `AcquisitionFamilyId` on an eligible validation-role evidence record, from an exact distinct acquisition family and experiment scope, not present in any supporting/training/calibration candidate used to establish ExperimentalSupport. Unknown lineage/family never counts. Each required condition must have an eligible supporting record in the declared scope; no missing condition passes.

| Result | deterministic requirements |
|---|---|
| Unassessed | missing B config, no valid available candidate, or any Required gate is NotAssessed/NotSatisfied |
| Hypothesized | definition exists and at least one valid Available selected record supports it |
| ExperimentallySupported | Hypothesized; all Required requirements/gates Satisfied; support count at least promotion minimum; independent known family count at least promotion minimum; critical contradiction count below configured limit; all required identifiability Satisfied |
| ValidatedForDomain | ExperimentallySupported plus present validation table, all required conditions, distinct validation families at least `validation.minimum_acquisition_families`, all mandatory domain-validation identifiability gates Satisfied, and no blocking critical contradiction |

`MechanismAnalysisReport` owns `hypothesis_assessments: Vec<HypothesisAssessmentRecord>` and `hypothesis_history: Vec<HypothesisHistoryEntry>` in schema 4. An entry has `history_id=SHA256(hypothesis_id || NUL || prior_level || NUL || new_level || NUL || assessment_hash)`, hypothesis ID, prior/new level, assessment hash, sorted reason codes, sorted source EvidenceIds, sequence number, and optional RFC-3339 timestamp excluded from equality/hash. `assessment_hash` is RFC-8785 SHA-256 of the deterministic scientific assessment view (definition ID, requirement results, component assessments, sorted evidence IDs, config hash, protocol ID/version, identifiability, temporal outcomes); no timestamp/human text. On each run, compute then append only if no prior entry for that hypothesis has equal `(prior_level,new_level,assessment_hash)`; sequence is prior maximum plus one. History content is sorted by hypothesis ID then sequence for serialization. Current assessment and history scientific fields are included in the report semantic hash; timestamps are excluded.

### 20.6 Schema, runner inputs, CLI, and errors (B-COMPAT)

`mechanism_analysis` changes from current schema **3** to **4**, with supported legacy versions `[1,2,3]` (the existing artifact-kind policy remains `Required`). Schema-3 artifacts deserialize with `hypothesis_assessments=[]`, `hypothesis_history=[]`; their absence means NotAssessed, never support. Schema-1/2 retain their documented historical migrations. A schema-4 writer never fabricates a history entry for a legacy read. All current B assessment/history fields participate in semantic hashing as specified above; legacy writers/readers retain old hashes only for their old payload representations.

| Artifact | before / after | legacy | B fields and migration |
|---|---|---|---|
| `mechanism_analysis` | 3 / 4 | `[1,2,3]` | `hypothesis_assessments`, `hypothesis_history`; serde `[]`; missing = NotAssessed |
| `EvidenceBundle` | 1 / 2 | `[1]` | schema 1 has no temporal metadata and retains its schema-1 identity; schema 2 adds `temporal_metadata`; missing schema-2 metadata = `[]`, MissingEvidence only |
| `ism_model_analysis` | 5 / 5 | `[1,2,3,4,5]` | unchanged; accepted read-only B input |

`EvidenceBundleInputs` gains `model_artifact: Option<ModelAnalysisReport>` and includes its known lineage in the catalog. It accepts only artifact kind `ism_model_analysis`, schema versions `[1,2,3,4,5]` readable by its existing contract, and scope compatible with the mechanism target. The complete field-to-evidence mapping, including literal direction and unmapped fields, is §21.6; the earlier "derived from field semantics" wording is superseded. Raw model records never become Strong automatically.

```rust
pub struct ArtifactReference { pub path: String, pub expected_artifact_kind: ArtifactKind, pub expected_schema_versions: Vec<u32> }
pub struct ConflictingEvidenceInput { pub artifact: ArtifactReference, pub expected_scope: EvidenceScopeRequirement, pub role: ConflictingEvidenceRole }
pub enum ConflictingEvidenceRole { EisFit, Transient, Calibration, Estimation, Model, PriorMechanism, EvidenceBundle }
pub enum MechanismEvidenceInputError { ArtifactKindMismatch { role: ConflictingEvidenceRole }, UnsupportedSchemaVersion { role: ConflictingEvidenceRole, schema_version: u32 }, ExperimentScopeConflict, SensorScopeConflict, ChannelScopeConflict, ClockScopeConflict, MissingRequiredEvidence, MissingMechanismEvidenceConfig, EvidenceRequirementBindingMismatch, ConflictingEvidenceInput }
```

Any artifact scope conflict returns the named typed error and is never silently dropped. The complete mechanism CLI, source-assembly exclusivity, `EvidenceBundle` schema acceptance, and validation-protocol ownership are §21.7; this earlier partial "adds exactly" list is superseded and MUST NOT be implemented.

### 20.7 Permanent fixture/data matrix and traceability (B-TEST)

The following original fixture matrix is retained only as traceability history. Its path shorthand is superseded by the literal files, contents, and assertions in §21.8. Immutable upstream source data are `tests/fixtures/a1/current_labeled_covariance.json`, `tests/fixtures/a1/legacy_unlabeled_covariance.json`, `tests/fixtures/a1/current_known_lineage_state.json`, `tests/fixtures/a1/aggregate_scope.json`, `tests/fixtures/a0_artifact_contracts/schema2/transient_analysis.schema2.json`, and `tests/fixtures/a0_artifact_contracts/schema2/mechanism_analysis.schema2.json`. Generated artifacts use public `read_artifact`/`write_artifact` and the existing `assemble_evidence_bundle` route.

| Permanent test ID | exact tracked fixture or deterministic source route |
|---|---|
| `MHI-B-T00-config` through `MHI-B-T07-e2e` | superseded by the literal fixture/data/assertion matrix in §21.8; this retained row carries no fixture-path contract |

| Requirement | normative behavior / public type | implementation module | AC and exact test ID | fixture/data | compatibility / scientific risk |
|---|---|---|---|---|---|
| `MHI-B-R01` | required no-default config; `MechanismEvidenceConfig` | `src/mechanism/config.rs` | `B-CONFIG-AC-01`; `MHI-B-T00-config` | exact file in §21.8 | additive config / hidden threshold |
| `MHI-B-R02` | exact selectors, full candidate retention, exact pair IDs; `EvidenceRequirementBinding` | `src/mechanism/evidence.rs` | `B-BIND-AC-01`; `MHI-B-T01-timescale-independent` | exact files in §21.8 | additive / non-deterministic support |
| `MHI-B-R03` | one conservative result per requirement; `IdentifiabilityAssessment` | `src/mechanism/identifiability.rs` | `B-IDENT-AC-01`; literal `MHI-B-T05` IDs in §21.8 | exact files in §21.8 | additive / false identifiability |
| `MHI-B-R04` | bundle-owned temporal metadata and typed outcome; `TemporalJoinOutcome` | `src/mechanism/temporal.rs` | `B-TEMP-AC-01`; literal `MHI-B-T04` IDs in §21.8 | exact files in §21.8 | schema-2 field / temporal leakage |
| `MHI-B-R05` | per-component outcome; `ComponentInterpretationAssessment` | `src/results/mechanism.rs` | `B-MULTI-AC-01`; `MHI-B-T07-e2e` | exact files in §21.8 | schema 4 / component overclaim |
| `MHI-B-R06` | config-owned protocol and family exclusions; `ValidationProtocol` | `src/mechanism/validation.rs` | `B-VALID-AC-01`; literal `MHI-B-T06` IDs in §21.8 | exact files in §21.8 | additive config / false validation |
| `MHI-B-R07` | schema 3->4 migration and scientific hash; `HypothesisHistoryEntry` | `src/results/mechanism.rs` and `src/results/artifact_contracts.rs` | `B-MIGRATION-AC-01`; `MHI-B-T07-e2e` | exact files in §21.8 | legacy readable / fabricated support |
| `MHI-B-R08` | model adapter and typed scope errors; `ConflictingEvidenceInput` | `src/runners/evidence.rs`, `src/evidence_adapters.rs`, `src/runners/mechanism.rs`, and `src/cli.rs` | `B-INPUT-AC-01`; `MHI-B-T07-e2e` | exact file in §21.8 | additive input / scope-conflicted evidence |
| `MHI-B-R09` | exact test/data matrix and public E2E path | `tests/phase_b_mechanism_evidence.rs` | `B-TEST-AC-01`; literal IDs in §21.8 | every exact path is in §21.8 | test-only files / fixture-only coverage |

### 20.8 Phase B amendment self-audit and delivery

```text
Undefined normative types: 0
Unspecified Phase B algorithms: 0
Unspecified scientific thresholds: 0
Unspecified compatibility decisions: 0
Normative contradictions: 0
Implementation invention still required: no
```

The final Phase B contract is §20.1 plus §21; §20.2--20.8 provide only retained descriptive/traceability context where §21 does not replace them. This amendment modifies documentation only. Before commit, stage only this file, run `git diff --check`, inspect the cached diff, commit the current remediation message, calculate its SHA-256/blob ID, and push only `plan/mhi-v1-b-contract-amendment`. Do not merge or tag it and do not change `main` or `codex/mhi-v1-b-mechanism-evidence-integration`.

## 21. Phase B Contract Remediation II — final executable semantics

### 21.1 Authority, supersession, and review reconciliation

For Phase B V1 implementation, **§20.1 is the sole normative serialized/TOML configuration contract**. The Phase B portions of §§6, 7, 8, 9, 10, 14, and 20.2--20.8 are descriptive only except where this §21 expressly retains an algorithm or type. Any earlier Phase B configuration field, threshold owner, gate configuration, `ValidationProtocol` definition, CLI list, or `EvidenceBundle` schema rule that conflicts with §20.1 or this §21 is superseded and MUST NOT be implemented. In particular, the approved Phase B root is the document loaded only by `--mechanism-evidence-config`, its validation table is optional, and `required_experiment_scopes` is retired. The frozen A1 contracts remain normative for schema-1 parsing and schema-1 semantic identity.

The Phase B mechanism-evidence configuration is a standalone TOML document loaded through `--mechanism-evidence-config`; its top-level key space is owned directly by `MechanismEvidenceConfig`. Its serialized sections are exactly `[timescale]`, `[amplitude]`, `[repeatability]`, `[temporal]`, `[temporal.mixed_state_policy]`, `[identifiability]`, `[promotion]`, and optional `[validation]`. The `[mechanism.evidence.*]` namespace is not part of the serialized Phase B TOML format and MUST NOT be accepted as an alternative Phase B V1 layout.

| Review finding | pre-remediation classification | final disposition |
|---|---|---|
| PB-RR-01 configuration/threshold ownership | CONFIRMED | resolved by §20.1 and §21.2 |
| PB-RR-02 `EvidencePairSelector` owner | CONFIRMED | resolved by `EvidenceRequirementBinding` in §20.2 |
| PB-RR-03 temporal field names | CONFIRMED | resolved by §21.3 |
| PB-RR-04 component transition | CONFIRMED | resolved by §21.4 |
| PB-RR-05 `ValidationProtocol` shapes | CONFIRMED | resolved by §20.1 and §21.4 |
| PB-RR-06 legacy bundle identity | CONFIRMED | resolved by §21.5 |
| PB-RR-07 model direction | CONFIRMED | resolved by §21.6 |
| PB-RR-08 test/fixture bindings | CONFIRMED | resolved by §21.8 |
| PB-RR-09 critical contradiction | CONFIRMED | resolved by §21.4 |
| PB-RR-10 CLI surface | CONFIRMED | resolved by §21.7 |

The occurrence classifications required for retained terminology are: §6 `ValidationProtocol`, `acceptance_criteria`, legacy component promotion, and critical-moderate wording: **SUPERSEDED**; §7 `maximum_timestamp_difference_s` and its old `TemporalJoinConfig`: **SUPERSEDED**; §7 `MixedStatePolicy`: **DESCRIPTIVE**, with the field ownership/range in §20.1/§21.2; §10 `--estimation-artifact` and `--model-artifact`: **SUPERSEDED** by the complete table in §21.7; §20.2 `EvidencePairSelector`: **ACTIVE NORMATIVE** only as the member of `EvidencePairRequirement`; §20.4 `temporal_metadata`: **ACTIVE NORMATIVE** only for schema 2; §20.5 `ValidatedForDomain`: **ACTIVE NORMATIVE** only as refined by §21.4; §20.1 `point_tolerance_s`, `ValidationProtocol`, `required_conditions`, and `minimum_acquisition_families`: **ACTIVE NORMATIVE**. Every occurrence of `required_experiment_scopes`: **SUPERSEDED / RETIRED**. No other occurrence of those terms establishes a competing implementation contract.

### 21.2 Complete scientific threshold inventory

Every threshold below has exactly one owner, TOML key, unit, validation, and consumer. All fields are required and have no default. A comparison not listed here has threshold zero; numerical round-off constants in §8 are implementation tolerances, not scientific configuration.

| ID | owner / TOML key | unit | validation and boundary | sole consumer |
|---|---|---:|---|---|
| TS-01 | `TimescaleEvidenceConfig.timescale.confidence_level` | 1 | `0.5<c<1`; exact configured confidence required | timescale interval compatibility |
| TS-02 | `timescale.strong_max_log_distance` | 1 | `>=0`, `strong<=moderate<=weak`; `d_high<=strong` is Strong | timescale classifier |
| TS-03 | `timescale.moderate_max_log_distance` | 1 | ordered as TS-02; `(strong,moderate]` is Moderate | timescale classifier |
| TS-04 | `timescale.weak_max_log_distance` | 1 | ordered as TS-02; `(moderate,weak]` is Weak | timescale classifier |
| TS-05 | `timescale.minimum_observation_duration_ratio` | 1 | `>0`; equality passes | timescale eligibility |
| TS-06 | `timescale.minimum_samples_per_tau` | 1 | `>0`; equality passes | timescale eligibility |
| TS-07 | `timescale.minimum_mode_separation_ratio` | 1 | `>0`; equality passes | timescale / identifiability eligibility |
| AM-01 | `amplitude.amplitude_floor` | quantity's unit | finite `>0`; denominator floor | amplitude gate |
| AM-02 | `amplitude.maximum_relative_amplitude_error` | 1 | finite `>=0`; equality passes | amplitude gate |
| AM-03 | `amplitude.minimum_strength` | enum | `Weak`, `Moderate`, or `Strong`; equality passes | amplitude candidate gate |
| RP-01 | `repeatability.minimum_replicates` | count | integer `>=2`; fewer is NotAssessed | repeatability gate |
| RP-02 | `repeatability.maximum_log_tau_standard_deviation` | 1 | finite `>=0`; equality passes | repeatability gate |
| RP-03 | `repeatability.minimum_independent_acquisition_families` | count | integer `>=1`; equality passes | repeatability gate |
| PR-01 | `promotion.minimum_supporting_evidence` | count | integer `>=1`; equality passes | ExperimentallySupported promotion |
| PR-02 | `promotion.minimum_independent_acquisition_families` | count | integer `>=1`; equality passes | ExperimentallySupported promotion |
| PR-03 | `promotion.critical_moderate_contradiction_count` | count | integer `>=1`; a count **at or above** this value blocks | moderate critical-contradiction block |
| PR-04 | `promotion.evidence_level_minimum_strength` | enum | `Weak`, `Moderate`, or `Strong`; equality passes | support/validation evidence-level gate |
| TM-01 | `temporal.point_tolerance_s` | s | finite `>=0`; `abs(delta)<=tolerance` is eligible | point temporal join |
| TM-02 | `temporal.minimum_classified_fraction` | 1 | finite `[0,1]`; equality passes | temporal join |
| TM-03 | `temporal.minimum_equilibrium_fraction` | 1 | finite `[0,1]`; equality passes | temporal join |
| TM-04 | `temporal.mixed_state_policy.minimum_fraction` | 1 | required only for `MinimumSteadyFraction`, finite `[0,1]`; equality passes | mixed-state resolution |
| ID-01 | `identifiability.minimum_covariate_samples` | count | integer `>=1`; equality passes | covariate assessor |
| ID-02 | `identifiability.minimum_covariate_range` | source quantity unit | finite `>0`; equality passes | covariate assessor |
| ID-03 | `identifiability.maximum_absolute_pearson_correlation` | 1 | finite `[0,1]`; equality passes | covariate assessor |
| ID-04 | `identifiability.minimum_interferent_samples` | count | integer `>=1`; equality passes | interferent assessor |
| ID-05 | `identifiability.minimum_interferent_log10_range` | log10(activity), 1 | finite `>0`; equality passes | interferent assessor |
| ID-06 | `identifiability.minimum_absolute_log10_activity_step` | log10(activity), 1 | finite `>0`; equality passes | transient-excitation assessor |
| ID-07 | `identifiability.minimum_pre_event_points` | count | integer `>=1`; equality passes | transient-excitation assessor |
| ID-08 | `identifiability.minimum_post_event_points` | count | integer `>=1`; equality passes | transient-excitation assessor |
| VP-01 | `validation.minimum_acquisition_families` | count | present `[validation]`: integer `>=1`; equality passes | ValidatedForDomain protocol |

`MixedStatePolicy::MinimumSteadyFraction { minimum_fraction, ... }` is active. It is owned only by `MechanismEvidenceConfig.temporal.mixed_state_policy.minimum_fraction`; it must appear in the serialized type, the TOML mapping, this inventory, and temporal validation. Its exact TOML is `[temporal.mixed_state_policy]`, `kind="minimum_steady_fraction"`, `minimum_fraction=<f64>`, `allow_quasi_equilibrium=<bool>`, and `reject_if_disturbed=<bool>`. `require_all_steady` has exactly `kind` and `allow_quasi_equilibrium`; `worst_case` has exactly `kind`. Any other member, including `minimum_fraction` on those variants, is a typed configuration error. `maximum_timestamp_difference_s` is retired Phase B terminology and MUST NOT be serialized, parsed, aliased, or implemented.

### 21.3 Temporal contract and one canonical tolerance

The complete Phase B `TemporalJoinConfig` is the §20.1 type. Its TOML table is `[temporal]`; `point_tolerance_s` is the only point-tolerance field. Old Phase B files containing `maximum_timestamp_difference_s` are not Phase B-compatible files: the B parser rejects the unknown key through `deny_unknown_fields`; no migration or alias exists. §7's field name is historical descriptive text only.

The exact algorithm is: (1) validate Single experiment and sensor/channel scope; (2) resolve clocks; (3) select point(s) with `abs(t_source-t_target)<=point_tolerance_s`, with an equal nearest tie returning `AmbiguousNearestPoint`; (4) evaluate `minimum_classified_fraction`; (5) evaluate `minimum_equilibrium_fraction`; (6) apply the serialized mixed-state policy; (7) emit the typed outcome. `Aggregate`/`Unknown` temporal support is `MissingEvidence(AggregateOrUnknownSupport)`; scope/clock mismatch is `Indeterminate`; a failure cannot become support. `minimum_fraction` is applied only in step 6 and only by `MinimumSteadyFraction` using `(Equilibrium + permitted QuasiEquilibrium) / N_classified`.

### 21.4 Lifecycle, validation, and promotion order

`MechanismHypothesisDefinition.required_evidence` is `Vec<EvidenceRequirementBinding>` for Phase B; the earlier `Vec<HypothesisEvidenceRequirement>` representation is superseded. A `Pair` binds the same hypothesis evidence basis to each target component. Component-specific requirements are expressed by the binding's structural `ExactComponent(component_id)` target selector; the same binding cannot be silently projected to a component it does not target.

| hypothesis assessment | validation protocol | current component status | resulting component status |
|---|---|---|---|
| Unassessed / insufficient / NotAssessed | any | any | unchanged |
| Hypothesized conditions pass | unavailable or unsatisfied | Phenomenological | Hypothesized |
| ExperimentallySupported conditions pass | unavailable or unsatisfied | Phenomenological or Hypothesized | ExperimentallySupported |
| ValidatedForDomain conditions pass | Satisfied | Phenomenological, Hypothesized, or ExperimentallySupported | ValidatedForDomain |
| any lower recomputation | any | any B-derived status | the recomputed row's status; a component absent from the definition is unchanged |

For each listed target component independently, `ValidatedForDomain` requires: (1) its corresponding hypothesis is at least `ExperimentallySupported` at `promotion.evidence_level_minimum_strength`; (2) all mandatory hypothesis gates are `Satisfied` or explicitly `NotApplicable`; (3) no Strong critical contradiction blocks promotion; (4) all required identifiability assessments are `Satisfied`; (5) `[validation]` is present; (6) `ValidationProtocolStatus::Satisfied`; (7) at least `validation.minimum_acquisition_families` independent Known validation acquisition families are in the required condition scopes and disjoint from support/training/calibration families; and (8) every `ValidationCondition` is satisfied. This is the only transition to `ValidatedForDomain`. When `[validation]` is absent, `ValidatedForDomain` is impossible and the maximum Phase B component status is `ExperimentallySupported`.

Support and contradiction candidates are intentionally separate. For every requirement, the **support candidate pipeline** is: all evidence records -> exact target selector and source-class selector -> `availability=Available` -> required validity -> scope and temporal eligibility -> required quantity kind and exact UCUM unit -> the requirement's support-direction filter -> required strength -> sorted support candidates. For every CriticalEvidence requirement, the independent **critical contradiction pipeline** is: all evidence records -> exact target selector and source-class selector -> `availability=Available` -> required validity -> scope and temporal eligibility -> required quantity kind and exact UCUM unit -> `direction=Contradicts` -> `strength>=Strong` -> sorted critical-contradiction candidates. The second pipeline MUST NOT consume, reuse, or be derived from a support-direction-filtered set. `eligible strong critical contradictions` is its final candidate count.

Promotion evaluates in exactly this order: (1) validate hypothesis definition; (2) resolve the target/source candidate universe; (3) temporal, scope, and validity eligibility; (4) evaluate critical contradictions from the eligible universe; (5) if blocked, stop promotion; (6) evaluate support requirements; (7) identifiability; (8) timescale, amplitude, and repeatability; (9) independent-support count; (10) validation protocol; (11) final evidence level and per-component interpretation. The V1 constant `critical_contradiction_allowance=0` has no TOML key. If `count(eligible strong critical contradictions) > critical_contradiction_allowance`, promotion is `BLOCKED`; `MechanismAssessmentError::StrongCriticalContradiction { requirement_id, evidence_ids }` is returned and no hypothesis/component/history promotion occurs. Separately, `promotion.critical_moderate_contradiction_count` blocks if the count of mutually Independent matching Moderate-or-strong contradictory records reaches the configured count. The strong rule is evaluated first and cannot be bypassed by support filtering or aggregation.

### 21.5 EvidenceBundle schema and hash compatibility

The additive versioned design is mandatory:

| bundle schema | temporal metadata | semantic hash view | read/reserialize behavior |
|---:|---|---|---|
| 1 | absent; no fabricated `temporal_metadata` | schema-1 semantic view defined before Phase B; excludes every Phase B temporal field | reread and reserialize as schema 1, preserving the original schema-1 semantic identity unless an explicit schema migration action is requested |
| 2 | `temporal_metadata` is allowed and is required for a record used in a temporal-required binding | RFC-8785 canonical JSON of the schema-2 scientific view, including sorted temporal metadata | schema-2 rereads/reserializes as schema 2; missing metadata makes only temporal-required support MissingEvidence |

Schema 1 never changes identity because of a field introduced after schema 1. The normal B runner does not perform an implicit schema-1-to-2 migration. An explicit migration action must create a schema-2 artifact and record `migrated_from_schema=1` plus the source schema-1 semantic hash; it may not claim that the new schema-2 hash equals the schema-1 identity. This is the sole temporal metadata owner/version decision.

### 21.6 Deterministic model evidence adapter mapping

`src/evidence_adapters.rs::adapt_model_analysis` adapts only the following literal source field. All unlisted `ModelAnalysisReport` fields are not adapted; no numeric sign, field name, component role, or model semantic heuristic changes direction.

| source field path | target | source class | direction | availability / validity | strength | quantity / uncertainty | identifiability relevance |
|---|---|---|---|---|---|---|---|
| `$.points[i].contributions[j].potential_v` | `ModelComponent(ComponentId(points[i].contributions[j].component_id))` | `ModelDerived` | `Neutral` | finite value -> `Available` / `Valid`; absent/nonfinite -> `Missing` / `NotAssessed` | `NotAssessed` from `StrengthSource::NotAssessed` | value in `V`; uncertainty `None` | none; component context only |

The following fields are explicitly unmapped: `time_s`, `observed_voltage_v`, `predicted_voltage_v`, every `uncertainty.*`, `state_values`, `contributions[*].{variance_v2,owner,role,semantics,source,validity_domain,interpretation_status,equation_version,validity_status,warnings,uncertainty_status,state_output_ids,auxiliary_outputs}`, `equilibrium`, point `validity`, `unexplained_residual_v`, report `model_definition`, report `identifiability`, and report `evidence`. They produce no evidence record and therefore cannot support, contradict, or become Strong automatically.

### 21.7 Complete Phase B `mechanism compare` CLI

This is the complete authoritative Phase B mechanism input surface; all earlier "adds exactly" lists are superseded. `--validation-protocol` is retired and prohibited: validation is owned only by the optional `[validation]` table in `--mechanism-evidence-config`.

| flag | required | repeatable | owner | artifact/config type | scope validation | failure behavior |
|---|---|---|---|---|---|---|
| `--config <PATH>` | optional | no | legacy mechanism configuration | existing legacy/general mechanism TOML | existing legacy validation only | existing parser error; B-named fields have no B effect |
| `--mechanism-evidence-config <PATH>` | required for a B assessment | no | Phase B only | exact §20.1 TOML root, schema 1 | n/a | absent -> `MechanismEvidenceInputError::MissingMechanismEvidenceConfig`; malformed -> `MechanismEvidenceConfigError` before artifact reads |
| `--transient-artifact <PATH>` | required in source-assembly mode | no | source assembly | `transient_analysis`, readable current/legacy contract | exact compatible experiment/sensor/channel/clock scope | mismatch -> `MechanismEvidenceInputError::{ExperimentScopeConflict,SensorScopeConflict,ChannelScopeConflict,ClockScopeConflict}` |
| `--calibration-artifact <PATH>` | optional | no | source assembly | existing calibration artifact contract | exact compatible experiment/sensor/channel scope | kind/schema/scope -> typed `MechanismEvidenceInputError` |
| `--eis-artifact <PATH>` | required in source-assembly mode | no | source assembly | `eis_fit`, readable current/legacy contract | exact compatible experiment/sensor/channel scope | kind/schema/scope -> typed `MechanismEvidenceInputError` |
| `--estimation-artifact <PATH>` | optional | no | source assembly | `state_estimation`, existing readable contract | exact compatible experiment/sensor/channel scope | kind/schema/scope -> typed `MechanismEvidenceInputError` |
| `--model-artifact <PATH>` | optional | no | source assembly | `ism_model_analysis`, schemas `[1,2,3,4,5]` | exact compatible experiment/sensor/channel scope | kind/schema/scope -> typed `MechanismEvidenceInputError` |
| `--evidence-artifact <PATH>` | required in direct-bundle mode | no | direct bundle | `EvidenceBundle`, schema 1 or 2 | bundle and each selected record must satisfy binding scope | kind/schema/scope -> typed `MechanismEvidenceInputError`; schema 1 cannot satisfy a temporal-required binding |
| `--prior-mechanism-artifact <PATH>` | optional | no | history input | `mechanism_analysis`, schemas `[1,2,3,4]` | report scope compatible with target | typed input error; never evidence |
| `--lineage-catalog <PATH>` | optional | no | lineage resolution | serialized A1 lineage catalog | referenced lineage resolves or remains explicit Unknown | invalid -> typed input error |
| `--metadata <PATH>` | optional | no | legacy scope context | existing legacy metadata | existing parser's scope rules | mismatch -> typed input error |
| `--output <PATH>` | optional | no | report writer | output path | n/a | write error propagates |

For a B request, exactly one assembly mode is required: source assembly requires the pair `--eis-artifact` and `--transient-artifact` and may add calibration, estimation, and model artifacts; direct-bundle mode requires exactly `--evidence-artifact`. The modes are mutually exclusive. `--mechanism-evidence-config` is required in either mode. No alias exists for the retired Phase B spellings `--eis-fit`, `--transient-results`, or `--calibration-results`.

### 21.8 Historical fixture matrix

This retained Phase B II matrix is **SUPERSEDED**. The complete literal contents, typed errors, and exact test-function bindings are §22.8--§22.10; no row below is normative.

| test IDs | exact fixture(s) | literal data and exact assertions |
|---|---|---|
| `MHI-B-T00-config` | `e2e/mechanism_evidence.toml`, `validation/protocol_pass.toml`, `validation/protocol_fail.toml` | config has every §20.1 field, `point_tolerance_s=1.0`, all fraction thresholds `0.80`, `minimum_fraction=0.90`; missing child, unknown key, and defaulted field each return a typed config error |
| `MHI-B-T01-timescale-independent` | `timescale/independent_pair_left.json`, `timescale/independent_pair_right.json` | IDs `b-ts-eis-01`/`b-ts-transient-01`, tau `1.0 s`/`1.0 s`, Independent, zero covariance; pair key `(b-ts-eis-01,b-ts-transient-01)`, Strong |
| `MHI-B-T01-timescale-dependent` | `timescale/dependent_pair_bundle.json` | same IDs/tau, PartiallyDependent and no covariance; exact pair key; `NotAssessed/JointUncertaintyUnavailable` |
| `MHI-B-T01-timescale-with-covariance` | `timescale/with_covariance_pair_bundle.json` | same IDs, PartiallyDependent, LogSpace covariance `0.0`; exact pair key; Strong |
| `MHI-B-T01-timescale-without-covariance` | `timescale/without_covariance_pair_bundle.json` | same IDs, PartiallyDependent, no covariance; exact error/result `JointUncertaintyUnavailable` |
| `MHI-B-T01-timescale-boundary` | `timescale/boundary_pair_bundle.json` | ratio interval `d_high=0.10`, `strong_max_log_distance=0.10`; Strong by inclusive boundary |
| `MHI-B-T01-timescale-out-of-domain` | `timescale/out_of_domain_pair_bundle.json` | left validity `OutsideDomain`; result NotAssessed and no promotion |
| `MHI-B-T02-amplitude-sign` | `amplitude/expected_direction.json` | IDs `b-amp-predicted-01`/`b-amp-observed-01`, values `1.00 V`/`1.05 V`, Supports; error `0.05`, config maximum `0.10`; passes |
| `MHI-B-T02-amplitude-opposite` | `amplitude/opposite_direction.json` | same IDs, values `1.00 V`/`-1.00 V`, Contradicts; amplitude gate fails and critical status is blocking if requirement is critical |
| `MHI-B-T02-amplitude-indeterminate` | `amplitude/indeterminate.json` | observed availability Missing; AmplitudeNotAssessed |
| `MHI-B-T03-repeat-independent` | `repeatability/independent_family_a.json`, `repeatability/independent_family_b.json` | IDs `b-repeat-a-01`/`b-repeat-b-01`, families `b-family-a`/`b-family-b`, tau `1.0 s`/`1.0 s`; count 2 and SD 0, pass |
| `MHI-B-T03-repeat-dependent` | `repeatability/dependent_family_shared.json` | two IDs, shared family `b-family-shared`; independent count 1, NotAssessed |
| `MHI-B-T03-repeat-insufficient` | `repeatability/insufficient.json` | one ID/family `b-family-a`, count 1, NotAssessed |
| `MHI-B-T03-repeat-unknown-family` | `repeatability/unknown_family.json` | serialized family `Unknown`, count 0, NotAssessed |
| `MHI-B-T04-temporal-point` | `temporal/point_join.json` | target `2025-01-01T00:00:00Z`, source `2025-01-01T00:00:01Z`, same clock/event scope, Equilibrium; tolerance `1.0 s`, Eligible |
| `MHI-B-T04-temporal-window` | `temporal/window_join.json` | windows `[00:00:00Z,00:00:10Z)` and `[00:00:05Z,00:00:15Z)`; positive overlap, Eligible |
| `MHI-B-T04-temporal-event` | `temporal/event_join.json` | exact event `b-step-01`, interval `[00:00:00Z,00:00:10Z]`; Eligible |
| `MHI-B-T04-temporal-clock-mismatch` | `temporal/clock_mismatch.json` | same timestamp but incompatible clock IDs; `Indeterminate(ClockMismatch)` |
| `MHI-B-T04-temporal-scope-mismatch` | `temporal/scope_mismatch.json` | sensor `b-sensor-02`; `Indeterminate(ScopeMismatch)` |
| `MHI-B-T04-temporal-aggregate-unknown` | `temporal/aggregate_unknown.json` | Aggregate scope and Unknown support; `MissingEvidence(AggregateOrUnknownSupport)` |
| `MHI-B-T05-ident-satisfied` | `identifiability/satisfied.json` | ID `b-ident-01`, 3 covariate samples, range `2.0`, correlation `0.10`; status Satisfied |
| `MHI-B-T05-ident-not-satisfied` | `identifiability/not_satisfied.json` | ID `b-ident-01`, range `0.10` below config `1.0`; NotSatisfied |
| `MHI-B-T05-ident-not-assessed` | `identifiability/not_assessed.json` | ID `b-ident-01`, missing covariate source; NotAssessed |
| `MHI-B-T05-ident-custom-unsupported` | `identifiability/custom_unsupported.json` | `Custom("b-custom")`; `identifiability.custom.not_assessed`, NotAssessed |
| `MHI-B-T06-validation-pass` | `validation/protocol_pass.toml`, `validation/pass.json` | protocol ID `b-validation-v1`, minimum families 2, validation families `b-family-v1`,`b-family-v2`, distinct from support; Satisfied and ValidatedForDomain |
| `MHI-B-T06-validation-insufficient` | `validation/protocol_fail.toml`, `validation/insufficient.json` | one validation family; ValidationFailed, no ValidatedForDomain |
| `MHI-B-T06-validation-unknown-family` | `validation/unknown_family.json` | validation family Unknown; ValidationFailed, no ValidatedForDomain |
| `MHI-B-T06-validation-training-overlap` | `validation/training_overlap.json` | validation family `b-family-a` also support family; ValidationFailed, no ValidatedForDomain |
| `MHI-B-T08-critical-contradiction` | `promotion/strong_critical_contradiction.json` | ID `b-critical-01`, `direction=Contradicts`, `strength=Strong`, valid/available `1.0 V`, and a critical requirement; promotion is blocked before support counting, level remains Unassessed, component status/history do not promote |
| `MHI-B-T07-e2e` | `e2e/mechanism_evidence.toml`, `e2e/eis_fit.json`, `e2e/transient_analysis.json`, `e2e/state_estimation.json`, `e2e/model_analysis.json`, `e2e/expected_mechanism_analysis.json` | public readers -> source assembly -> B assessment -> schema-4 write/reread; expected IDs/pair key/results are the T01 independent, T02 sign, T03 independent, T05 satisfied, T06 pass results; component `b-component-01` becomes ValidatedForDomain; first run appends exactly one history entry and reread changes none |

`model_analysis.json` contains one mapped contribution at `$.points[0].contributions[0].potential_v=0.25 V`, `component_id="b-component-01"`, and one unmapped `variance_v2=0.01`; the expected bundle has exactly the neutral, NotAssessed model record for the potential and no record for variance. All negative rows assert the named typed code above; no fixture path uses brace expansion, wildcard, or pseudo-path.

### 21.9 Final self-audit

```text
Undefined normative types: 0
Unspecified Phase B algorithms: 0
Unspecified scientific thresholds: 0
Unspecified compatibility decisions: 0
Normative contradictions: 0
Implementation invention still required: no
```

Two competent implementation agents cannot make different choices about threshold ownership, `MixedStatePolicy.minimum_fraction`, pair-selector ownership, temporal tolerance, `ValidatedForDomain`, `ValidationProtocol`, legacy bundle identity, model direction, fixture contents, critical contradiction blocking, or CLI flags: **NO** for each.

## 22. Phase B Contract Remediation III — validation, contradiction, ownership, and fixtures

### 22.1 Finding reconciliation and controlling rules

| finding | classification | current-plan evidence | controlling remediation |
|---|---|---|---|
| PB-FR-01 `required_experiment_scopes` semantics | CONFIRMED | §20.1 declared a numeric field but gave no distinct scientific meaning or counting algorithm | Retire it completely under §20.1 and §22.2. |
| PB-FR-02 critical-contradiction ordering | CONFIRMED | §20.2 selected support direction before the prior contradiction block | Use the two independent pipelines in §21.4 and §22.3. |
| PB-FR-03 config/CLI ownership | CONFIRMED | §20.1 formerly named `--config`; §21.7 named a dedicated flag | §20.1 and §21.7 now give Phase B one owner, one root, and optional validation. |
| PB-FR-04 fixture/test bindings | CONFIRMED | §21.8 named paths/results but did not bind every function to literal data and a typed negative result | §22.8--§22.10 replace that matrix. |

§22 is ACTIVE NORMATIVE for the four findings and is consistent with the directly corrected §§20.1, 20.2, 20.5, 21.2, 21.4, and 21.7. The older §21.8 rows are historical only.

### 22.2 Validation Protocol: one final contract

`ValidationProtocol.required_experiment_scopes` is **RETIRED for Phase B V1**. It is absent from the type definition, TOML mapping, scientific threshold inventory, validation algorithm, fixture requirements, and promotion matrix. Experiment-scope eligibility is expressed by each `ValidationCondition.experiment_scope` and the validation-family eligibility rule. No independent numeric experiment-scope-count threshold exists.

The entire `--mechanism-evidence-config` document is exactly one Phase B root. `[timescale]`, `[amplitude]`, `[repeatability]`, `[temporal]`, `[identifiability]`, and `[promotion]` are required. `[validation]` is optional. Every field of every present section is required unless explicitly optional; no field has a default. A present `[validation]` table has exactly `protocol_id`, `version`, `minimum_acquisition_families`, and `required_conditions`. It is `ValidationProtocolStatus::Satisfied` only when every condition has a matching eligible record and the distinct, Known, independent validation-family count is at least its inclusive minimum. Validation records must be outside every supporting, training, and calibration family used for the corresponding ExperimentalSupport assessment. Unknown family or lineage never counts. A missing table yields `ValidationProtocolStatus::NotAssessed`, not `Satisfied`; `ValidatedForDomain` is forbidden and the component is at most `ExperimentallySupported`.

### 22.3 Critical contradiction algorithm and exact blocking

For an individual requirement, target/source matching means structural equality of its `EvidenceTargetSelector` and membership in its sorted `source_class_selector`; eligibility then means `Available`, allowed validity, scope/temporal eligibility, and (when specified) exact quantity kind plus UCUM unit. This common eligible universe precedes either direction filter.

The support pipeline is: all records -> target/source matching -> availability -> validity -> scope/temporal eligibility -> quantity/unit compatibility -> requirement support-direction filter -> requirement strength threshold -> support candidates.

For every requirement listed by `critical_evidence_requirement_ids`, the contradiction pipeline is: all records -> target/source matching -> availability -> validity -> scope/temporal eligibility -> quantity/unit compatibility -> `direction=Contradicts` -> `strength>=Strong` -> critical contradiction candidates. It is a fresh traversal of all records, not a transform, subset, cache, or reuse of support candidates.

`critical_contradiction_allowance` is the fixed V1 constant `0`; it has no configuration key. If `count(critical contradiction candidates) > 0`, return `MechanismAssessmentError::StrongCriticalContradiction { requirement_id, evidence_ids }`, set promotion to `BLOCKED`, and do not create a promoted hypothesis, component, or history result. Evaluation order is exactly: (1) validate hypothesis definition; (2) resolve target/source candidate universe; (3) temporal/scope/validity eligibility; (4) critical contradictions; (5) if blocked, stop promotion; (6) support requirements; (7) identifiability; (8) timescale/amplitude/repeatability; (9) independent-support count; (10) validation protocol; (11) final evidence level/component interpretation.

### 22.4 Exact typed errors

```rust
pub enum MechanismEvidenceConfigError {
    MissingRequiredField { field: String },
    UnknownField { field: String },
    InvalidThreshold { field: String, value: f64 },
    UnsupportedSchemaVersion { actual: u32 },
}
pub enum HypothesisDefinitionError {
    PairSelectorRequired { requirement_id: EvidenceRequirementId },
    PairSelectorForbidden { requirement_id: EvidenceRequirementId },
}
pub enum ValidationProtocolError {
    InsufficientIndependentAcquisitionFamilies { required: usize, actual: usize },
    RequiredConditionNotSatisfied { condition_id: String },
    ValidationConfigurationMissing,
}
pub enum MechanismAssessmentError {
    StrongCriticalContradiction { requirement_id: EvidenceRequirementId, evidence_ids: Vec<EvidenceId> },
    ValidationProtocol(ValidationProtocolError),
    RequirementNotAssessed { requirement_id: EvidenceRequirementId, reason: HypothesisReasonCode },
    RequirementNotSatisfied { requirement_id: EvidenceRequirementId, reason: HypothesisReasonCode },
    Temporal(TemporalJoinReason),
}
```

The existing §20.6 `MechanismEvidenceInputError` variants remain the sole typed errors for artifact kind/schema/scope/clock/input conflicts. `ExperimentScopeConflict`, `SensorScopeConflict`, `ChannelScopeConflict`, and `ClockScopeConflict` carry the corresponding `ConflictingEvidenceInput` context defined there. No string-only error satisfies a negative stable test.

### 22.5 Final domain transition, threshold inventory, and promotion matrix

`ValidatedForDomain` may occur only when: the corresponding hypothesis is at least `ExperimentallySupported`; all mandatory hypothesis gates are `Satisfied` or explicitly `NotApplicable`; no Strong critical contradiction blocks; required identifiability is `Satisfied`; `[validation]` is present; the validation protocol is `Satisfied`; its independent validation-family count passes; and every required condition passes. The retired experiment-scope count is not a gate. If validation is absent, the status remains at most `ExperimentallySupported`.

The active numeric inventory is exactly TS-01--TS-07, AM-01--AM-03, RP-01--RP-03, PR-01--PR-04, TM-01--TM-04, ID-01--ID-08, and VP-01 in §21.2. `critical_contradiction_allowance=0` is a V1 behavioral constant, not a scientific/configurable threshold. `required_experiment_scopes` appears in no active inventory. Unspecified scientific thresholds: **0**.

| promotion gate | Hypothesized | ExperimentallySupported | ValidatedForDomain |
|---|---|---|---|
| required evidence gate and support strength | required | required | required |
| temporal eligibility | required | required | required |
| critical contradiction block | required | required | required |
| identifiability | required where mandatory | required where mandatory | required where mandatory |
| timescale, amplitude, repeatability | required where declared | required where declared | required where declared |
| independent support-family count | not a separate promotion minimum | `promotion.minimum_independent_acquisition_families` | ExperimentalSupport minimum still passes |
| validation presence/result | not assessed | not assessed | present and `Satisfied` |
| validation-family count | not assessed | not assessed | `validation.minimum_acquisition_families` inclusive |
| validation conditions | not assessed | not assessed | every condition passes |
| numeric experiment-scope count | retired | retired | retired |

### 22.6 Global terminology consistency

| searched term | occurrence classification and final meaning |
|---|---|
| `required_experiment_scopes` | SUPERSEDED / RETIRED everywhere; it is forbidden in active type/TOML/algorithm/fixture/table text. |
| `required_conditions` | ACTIVE NORMATIVE in §20.1 and §22.2; all predicates must pass. |
| `critical_contradiction` | ACTIVE NORMATIVE in §§21.4 and 22.3; independent pipeline and fixed zero allowance. |
| `DirectionRequirement` | SUPERSEDED spelling; `EvidenceDirectionRequirement` is the active type. |
| `--config` | ACTIVE NORMATIVE only as legacy/general config in §§20.1 and 21.7. |
| `--mechanism-evidence-config` | ACTIVE NORMATIVE as the sole Phase B config entry point in §§20.1 and 21.7. |
| `--validation-protocol` | SUPERSEDED / RETIRED; prohibited. |
| `protocol = None` | SUPERSEDED; the active absence representation is `validation=None`. |
| `validation = None` | ACTIVE NORMATIVE for absent optional `[validation]`, yielding NotAssessed and no domain validation. |
| `every field required` | ACTIVE NORMATIVE only as “every field of every present section is required unless explicitly optional.” |

### 22.7 Fixture grammar shared by every JSON artifact

Fixture root is exactly `tests/fixtures/phase_b/`. Every evidence-bundle JSON named below has `artifact_kind="evidence_bundle"`, `schema_version=2`, `artifact_id="b-bundle-01"`, `experiment_scope="Single:b-exp-01"`, `sensor_scope="Specific:b-sensor-01"`, `channel_scope="Specific:b-channel-01"`, and lineage `{"state":"Known","acquisition_family_id":"b-family-a","lineage_artifact_id":"b-lineage-01"}` unless that row explicitly substitutes a value. Each listed evidence record has all of these literal fields: `evidence_id`, `target_component_id`, `source_class`, `direction`, `availability`, `strength`, `validity`, `quantity_kind`, `quantity_value`, `quantity_unit`, `experiment_id`, `acquisition_family_id`, `timestamp`, `clock_id`, `event_id`, `equilibrium`, `covariance_relation`, and `source_field_path`. Values not overridden below are `target_component_id="b-component-01"`, `source_class="Observed"`, `direction="Supports"`, `availability="Available"`, `strength="Strong"`, `validity="Valid"`, `experiment_id="b-exp-01"`, `timestamp="2025-01-01T00:00:00Z"`, `clock_id="b-clock-01"`, `event_id="b-step-01"`, `equilibrium="Equilibrium"`, `covariance_relation="Independent"`, and `source_field_path="$.fixture"`. An expected analysis JSON has `artifact_kind="mechanism_analysis"`, `schema_version=4`, `artifact_id="b-mechanism-analysis-01"`, the same three scopes, lineage `b-lineage-01`, and the stated expected IDs/results. These values are required, not examples.

### 22.8 Literal validation and E2E fixture files

`tests/fixtures/phase_b/validation/protocol_pass.toml` is exactly:

```toml
schema_version = 1
[timescale]
confidence_level = 0.95
strong_max_log_distance = 0.10
moderate_max_log_distance = 0.20
weak_max_log_distance = 0.30
minimum_observation_duration_ratio = 2.0
minimum_samples_per_tau = 5.0
minimum_mode_separation_ratio = 2.0
[amplitude]
amplitude_floor = 0.01
maximum_relative_amplitude_error = 0.10
minimum_strength = "Strong"
[repeatability]
minimum_replicates = 2
maximum_log_tau_standard_deviation = 0.10
minimum_independent_acquisition_families = 2
[temporal]
point_tolerance_s = 1.0
window_overlap_rule = "positive_duration"
event_identity_rule = "exact"
minimum_classified_fraction = 0.80
minimum_equilibrium_fraction = 0.80
clock_mismatch_behavior = "indeterminate"
scope_mismatch_behavior = "indeterminate"
[temporal.mixed_state_policy]
kind = "minimum_steady_fraction"
minimum_fraction = 0.90
allow_quasi_equilibrium = true
reject_if_disturbed = true
[identifiability]
minimum_covariate_samples = 3
minimum_covariate_range = 1.0
maximum_absolute_pearson_correlation = 0.50
minimum_interferent_samples = 3
minimum_interferent_log10_range = 1.0
minimum_absolute_log10_activity_step = 0.50
minimum_pre_event_points = 2
minimum_post_event_points = 2
[promotion]
critical_moderate_contradiction_count = 1
minimum_supporting_evidence = 2
minimum_independent_acquisition_families = 2
evidence_level_minimum_strength = "Strong"
[validation]
protocol_id = "b-validation-v1"
version = "1"
minimum_acquisition_families = 2
[[validation.required_conditions]]
condition_id = "domain_temperature"
requirement_ids = ["b-validation-requirement-temperature"]
experiment_scope = "Single:b-exp-01"
sensor_scope = "Specific:b-sensor-01"
channel_scope = "Specific:b-channel-01"
[[validation.required_conditions]]
condition_id = "matrix_class"
requirement_ids = ["b-validation-requirement-matrix"]
experiment_scope = "Single:b-exp-01"
sensor_scope = "Specific:b-sensor-01"
channel_scope = "Specific:b-channel-01"
```

`tests/fixtures/phase_b/validation/protocol_insufficient_families.toml` is exactly:

```toml
schema_version = 1
[timescale]
confidence_level = 0.95
strong_max_log_distance = 0.10
moderate_max_log_distance = 0.20
weak_max_log_distance = 0.30
minimum_observation_duration_ratio = 2.0
minimum_samples_per_tau = 5.0
minimum_mode_separation_ratio = 2.0
[amplitude]
amplitude_floor = 0.01
maximum_relative_amplitude_error = 0.10
minimum_strength = "Strong"
[repeatability]
minimum_replicates = 2
maximum_log_tau_standard_deviation = 0.10
minimum_independent_acquisition_families = 2
[temporal]
point_tolerance_s = 1.0
window_overlap_rule = "positive_duration"
event_identity_rule = "exact"
minimum_classified_fraction = 0.80
minimum_equilibrium_fraction = 0.80
clock_mismatch_behavior = "indeterminate"
scope_mismatch_behavior = "indeterminate"
[temporal.mixed_state_policy]
kind = "minimum_steady_fraction"
minimum_fraction = 0.90
allow_quasi_equilibrium = true
reject_if_disturbed = true
[identifiability]
minimum_covariate_samples = 3
minimum_covariate_range = 1.0
maximum_absolute_pearson_correlation = 0.50
minimum_interferent_samples = 3
minimum_interferent_log10_range = 1.0
minimum_absolute_log10_activity_step = 0.50
minimum_pre_event_points = 2
minimum_post_event_points = 2
[promotion]
critical_moderate_contradiction_count = 1
minimum_supporting_evidence = 2
minimum_independent_acquisition_families = 2
evidence_level_minimum_strength = "Strong"
[validation]
protocol_id = "b-validation-v1"
version = "1"
minimum_acquisition_families = 3
[[validation.required_conditions]]
condition_id = "domain_temperature"
requirement_ids = ["b-validation-requirement-temperature"]
experiment_scope = "Single:b-exp-01"
sensor_scope = "Specific:b-sensor-01"
channel_scope = "Specific:b-channel-01"
[[validation.required_conditions]]
condition_id = "matrix_class"
requirement_ids = ["b-validation-requirement-matrix"]
experiment_scope = "Single:b-exp-01"
sensor_scope = "Specific:b-sensor-01"
channel_scope = "Specific:b-channel-01"
```

`tests/fixtures/phase_b/validation/protocol_missing.toml` is exactly:

```toml
schema_version = 1
[timescale]
confidence_level = 0.95
strong_max_log_distance = 0.10
moderate_max_log_distance = 0.20
weak_max_log_distance = 0.30
minimum_observation_duration_ratio = 2.0
minimum_samples_per_tau = 5.0
minimum_mode_separation_ratio = 2.0
[amplitude]
amplitude_floor = 0.01
maximum_relative_amplitude_error = 0.10
minimum_strength = "Strong"
[repeatability]
minimum_replicates = 2
maximum_log_tau_standard_deviation = 0.10
minimum_independent_acquisition_families = 2
[temporal]
point_tolerance_s = 1.0
window_overlap_rule = "positive_duration"
event_identity_rule = "exact"
minimum_classified_fraction = 0.80
minimum_equilibrium_fraction = 0.80
clock_mismatch_behavior = "indeterminate"
scope_mismatch_behavior = "indeterminate"
[temporal.mixed_state_policy]
kind = "minimum_steady_fraction"
minimum_fraction = 0.90
allow_quasi_equilibrium = true
reject_if_disturbed = true
[identifiability]
minimum_covariate_samples = 3
minimum_covariate_range = 1.0
maximum_absolute_pearson_correlation = 0.50
minimum_interferent_samples = 3
minimum_interferent_log10_range = 1.0
minimum_absolute_log10_activity_step = 0.50
minimum_pre_event_points = 2
minimum_post_event_points = 2
[promotion]
critical_moderate_contradiction_count = 1
minimum_supporting_evidence = 2
minimum_independent_acquisition_families = 2
evidence_level_minimum_strength = "Strong"
```

These are the only validation protocol fixture TOMLs; the former `protocol_fail.toml` name is retired.

| E2E file | required literal content |
|---|---|
| `e2e/mechanism_evidence.toml` | exact contents of `validation/protocol_pass.toml`; its config hash is the only B config hash accepted by the E2E expected artifact. |
| `e2e/eis_fit.json` | `artifact_kind="eis_fit"`, readable schema version, `artifact_id="b-eis-01"`, three common scopes, Known lineage family `b-family-a`, evidence `b-ts-eis-01`: tau `1.0 s`, Strong Supports, timestamp `2025-01-01T00:00:00Z`, independent. |
| `e2e/transient_analysis.json` | `artifact_kind="transient_analysis"`, readable schema version, `artifact_id="b-transient-01"`, three common scopes, Known lineage family `b-family-b`, evidence `b-ts-transient-01`: tau `1.0 s`, Strong Supports, timestamp `2025-01-01T00:00:01Z`, independent; equilibrium `Equilibrium`; event `b-step-01`. |
| `e2e/state_estimation.json` | `artifact_kind="state_estimation"`, readable schema version, `artifact_id="b-estimation-01"`, common scopes, Known family `b-family-v1`, validation evidence `b-validation-01`: `domain_temperature=298.15 K`, Strong Supports, Valid, Available. |
| `e2e/model_analysis.json` | `artifact_kind="ism_model_analysis"`, schema version `5`, `artifact_id="b-model-01"`, common scopes, Known family `b-family-v2`; `$.points[0].contributions[0].potential_v=0.25 V`, `component_id="b-component-01"`, and `variance_v2=0.01`; only the potential maps to Neutral/NotAssessed evidence. |
| `e2e/expected_mechanism_analysis.json` | common expected-analysis header; hypothesis `b-hypothesis-01`, pair key `(b-ts-eis-01,b-ts-transient-01)`, validation evidence IDs `b-validation-01,b-validation-02`, evidence level `ExperimentallySupported`, component `b-component-01=ValidatedForDomain`, protocol `b-validation-v1/1`, exactly one history entry `sequence=1`. |

### 22.9 Literal non-E2E fixture registry

| file path | required records and exact expected semantics |
|---|---|
| `timescale/independent_pair_left.json` | `b-ts-eis-01`, `tau=1.0 s`, `b-family-a`, Independent; pairs with right fixture. |
| `timescale/independent_pair_right.json` | `b-ts-transient-01`, `tau=1.0 s`, `b-family-b`, Independent; pair is Strong. |
| `timescale/dependent_pair_bundle.json` | `b-ts-eis-01` and `b-ts-transient-01`, both `tau=1.0 s`, relation PartiallyDependent, no covariance entry; `NotAssessed/JointUncertaintyUnavailable`. |
| `timescale/with_covariance_pair_bundle.json` | same pair, PartiallyDependent, exact pair covariance `LogSpace=0.0`; Strong. |
| `timescale/without_covariance_pair_bundle.json` | same pair, PartiallyDependent, no covariance; `JointUncertaintyUnavailable`. |
| `timescale/boundary_pair_bundle.json` | pair IDs above, `d_high=0.10`, configured `strong_max_log_distance=0.10`; Strong by equality. |
| `timescale/out_of_domain_pair_bundle.json` | `b-ts-eis-01` has `validity="OutsideDomain"`; `b-ts-transient-01` otherwise common; NotAssessed and no promotion. |
| `amplitude/expected_direction.json` | `b-amp-predicted-01=1.00 V` and `b-amp-observed-01=1.05 V`; Strong Supports, error `0.05`; amplitude passes. |
| `amplitude/opposite_direction.json` | IDs above, values `1.00 V` and `-1.00 V`; observed record Contradicts; amplitude fails. |
| `amplitude/indeterminate.json` | `b-amp-predicted-01=1.00 V`; `b-amp-observed-01` has `availability="Missing"`; `AmplitudeNotAssessed`. |
| `repeatability/independent_family_a.json` | `b-repeat-a-01`, tau `1.0 s`, family `b-family-a`, Independent. |
| `repeatability/independent_family_b.json` | `b-repeat-b-01`, tau `1.0 s`, family `b-family-b`, Independent; combined count 2, SD 0, pass. |
| `repeatability/dependent_family_shared.json` | `b-repeat-a-01,b-repeat-b-01`, tau `1.0 s`, both family `b-family-shared`; independent count 1, NotAssessed. |
| `repeatability/insufficient.json` | `b-repeat-a-01`, tau `1.0 s`, family `b-family-a`; count 1, NotAssessed. |
| `repeatability/unknown_family.json` | `b-repeat-a-01,b-repeat-b-01`, tau `1.0 s`, both `acquisition_family_id="Unknown"`; count 0, NotAssessed. |
| `temporal/point_join.json` | target `b-temporal-target-01` at `00:00:00Z`, source `b-temporal-source-01` at `00:00:01Z`, clock `b-clock-01`, event `b-step-01`, Equilibrium; Eligible. |
| `temporal/window_join.json` | target window `[00:00:00Z,00:00:10Z)`, source window `[00:00:05Z,00:00:15Z)`, common clock/scope; Eligible. |
| `temporal/event_join.json` | target/source event `b-step-01`, each window `[00:00:00Z,00:00:10Z]`, common clock/scope; Eligible. |
| `temporal/clock_mismatch.json` | common records with source `clock_id="b-clock-02"`; `Indeterminate(ClockMismatch)` and `MechanismEvidenceInputError::ClockScopeConflict`. |
| `temporal/scope_mismatch.json` | common records with source `sensor_scope="Specific:b-sensor-02"`; `Indeterminate(ScopeMismatch)` and `MechanismEvidenceInputError::SensorScopeConflict`. |
| `temporal/aggregate_unknown.json` | source `experiment_scope="Aggregate:b-aggregate-01"`, support `experiment_scope="Unknown"`; `MissingEvidence(AggregateOrUnknownSupport)`. |
| `identifiability/satisfied.json` | `b-ident-01`, covariate values `[0.0,1.0,2.0]`, response values `[0.0,0.1,0.2]`, range `2.0`, correlation `0.10`; Satisfied. |
| `identifiability/not_satisfied.json` | `b-ident-01`, covariate values `[0.0,0.05,0.10]`, response values `[0.0,0.1,0.2]`, range `0.10`; NotSatisfied. |
| `identifiability/not_assessed.json` | `b-ident-01`, `covariate_source=null`; NotAssessed. |
| `identifiability/custom_unsupported.json` | `requirement_kind="Custom:b-custom"`; assessor ID `identifiability.custom.not_assessed`, status NotAssessed. |
| `validation/pass.json` | `b-validation-01`, family `b-family-v1`, condition `domain_temperature`, `298.15 K`; `b-validation-02`, family `b-family-v2`, condition `matrix_class`, quantity `1` unit `1`; both Strong Supports, Valid, Available; Satisfied. |
| `validation/insufficient.json` | only `b-validation-01`, family `b-family-v1`, domain_temperature `298.15 K`; with protocol minimum 3, `ValidationProtocolError::InsufficientIndependentAcquisitionFamilies { required: 3, actual: 1 }`. |
| `validation/unknown_family.json` | `b-validation-01,b-validation-02`, both `acquisition_family_id="Unknown"`, required conditions otherwise pass; `InsufficientIndependentAcquisitionFamilies { required: 2, actual: 0 }`. |
| `validation/training_overlap.json` | validation `b-validation-01` family `b-family-a`, required condition domain_temperature `298.15 K`; it overlaps supporting family `b-family-a`; `InsufficientIndependentAcquisitionFamilies { required: 2, actual: 0 }`. |
| `promotion/strong_critical_contradiction.json` | `b-critical-01`, Strong Contradicts, `1.0 V`, valid/available, family `b-family-c`; `b-critical-support-01,b-critical-support-02`, Strong Supports, `1.0 V`, valid/available, families `b-family-a,b-family-b`; exact error `StrongCriticalContradiction { requirement_id="b-critical-requirement-01", evidence_ids=["b-critical-01"] }`. |

### 22.10 Exact stable-test binding table

All functions are in `tests/phase_b_mechanism_evidence.rs`. The fully qualified stable IDs are `MHI-B-T00-config`, `MHI-B-T01-timescale-independent`, `MHI-B-T01-timescale-dependent`, `MHI-B-T01-timescale-with-covariance`, `MHI-B-T01-timescale-without-covariance`, `MHI-B-T01-timescale-boundary`, `MHI-B-T01-timescale-out-of-domain`, `MHI-B-T02-amplitude-sign`, `MHI-B-T02-amplitude-opposite`, `MHI-B-T02-amplitude-indeterminate`, `MHI-B-T03-repeat-independent`, `MHI-B-T03-repeat-dependent`, `MHI-B-T03-repeat-insufficient`, `MHI-B-T03-repeat-unknown-family`, `MHI-B-T04-temporal-point`, `MHI-B-T04-temporal-window`, `MHI-B-T04-temporal-event`, `MHI-B-T04-temporal-clock-mismatch`, `MHI-B-T04-temporal-scope-mismatch`, `MHI-B-T04-temporal-aggregate-unknown`, `MHI-B-T05-ident-satisfied`, `MHI-B-T05-ident-not-satisfied`, `MHI-B-T05-ident-not-assessed`, `MHI-B-T05-ident-custom-unsupported`, `MHI-B-T06-validation-pass`, `MHI-B-T06-validation-insufficient`, `MHI-B-T06-validation-unknown-family`, `MHI-B-T06-validation-training-overlap`, `MHI-B-T08-critical-contradiction`, and `MHI-B-T07-e2e`, in that table-row order. “Config” means the exact fields in `validation/protocol_pass.toml` unless the listed protocol file says otherwise.

| test ID / function | level | fixture paths | fields consumed | config fields | expected result / typed error |
|---|---|---|---|---|---|
| T00 / `phase_b_config_rejects_missing_unknown_and_defaulted_fields` | integration | `validation/protocol_pass.toml`, `validation/protocol_missing.toml` | root, all section keys, validation presence | all §22.8 keys | missing -> `MissingRequiredField`; unknown -> `UnknownField`; invalid numeric -> `InvalidThreshold`; missing validation -> NotAssessed. |
| T01 / `phase_b_timescale_independent_pair_is_strong` | integration | `timescale/independent_pair_left.json`, `timescale/independent_pair_right.json` | IDs, tau, family, covariance relation | TS-01--TS-07 | Strong. |
| T01 / `phase_b_timescale_dependent_pair_without_covariance_is_not_assessed` | integration | `timescale/dependent_pair_bundle.json` | pair IDs, tau, relation, covariance | TS-01--TS-07 | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-timescale-requirement-01", reason=UncertaintyUnavailable }`. |
| T01 / `phase_b_timescale_dependent_pair_with_covariance_is_strong` | integration | `timescale/with_covariance_pair_bundle.json` | pair IDs, covariance | TS-01--TS-07 | Strong. |
| T01 / `phase_b_timescale_missing_covariance_is_not_assessed` | integration | `timescale/without_covariance_pair_bundle.json` | pair IDs, relation, absent covariance | TS-01--TS-07 | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-timescale-requirement-01", reason=UncertaintyUnavailable }`. |
| T01 / `phase_b_timescale_strong_boundary_is_inclusive` | unit | `timescale/boundary_pair_bundle.json` | `d_high` | TS-02 | Strong. |
| T01 / `phase_b_timescale_outside_domain_cannot_promote` | integration | `timescale/out_of_domain_pair_bundle.json` | validity | TS-01--TS-07 | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-timescale-requirement-01", reason=OutsideDomain }`. |
| T02 / `phase_b_amplitude_expected_direction_passes` | integration | `amplitude/expected_direction.json` | values, direction | AM-01--AM-03 | pass. |
| T02 / `phase_b_amplitude_opposite_direction_fails` | integration | `amplitude/opposite_direction.json` | values, direction | AM-01--AM-03 | `MechanismAssessmentError::RequirementNotSatisfied { requirement_id="b-amplitude-requirement-01", reason=AmplitudeFailed }`. |
| T02 / `phase_b_amplitude_missing_observation_is_not_assessed` | integration | `amplitude/indeterminate.json` | availability | AM-01--AM-03 | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-amplitude-requirement-01", reason=AmplitudeNotAssessed }`. |
| T03 / `phase_b_repeatability_independent_families_pass` | integration | `repeatability/independent_family_a.json`, `repeatability/independent_family_b.json` | IDs, tau, family | RP-01--RP-03 | pass. |
| T03 / `phase_b_repeatability_shared_family_is_not_assessed` | integration | `repeatability/dependent_family_shared.json` | family | RP-01--RP-03 | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-repeatability-requirement-01", reason=InsufficientIndependentFamilies }`. |
| T03 / `phase_b_repeatability_one_family_is_not_assessed` | integration | `repeatability/insufficient.json` | family | RP-01--RP-03 | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-repeatability-requirement-01", reason=InsufficientIndependentFamilies }`. |
| T03 / `phase_b_repeatability_unknown_family_is_not_assessed` | integration | `repeatability/unknown_family.json` | family | RP-01--RP-03 | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-repeatability-requirement-01", reason=InsufficientIndependentFamilies }`. |
| T04 / `phase_b_temporal_point_join_accepts_boundary` | integration | `temporal/point_join.json` | timestamps, clock, event, equilibrium | TM-01--TM-04 | Eligible. |
| T04 / `phase_b_temporal_window_join_requires_overlap` | integration | `temporal/window_join.json` | windows, clock, scope | TM-02--TM-04 | Eligible. |
| T04 / `phase_b_temporal_event_join_requires_exact_event` | integration | `temporal/event_join.json` | event, windows | TM-01--TM-04 | Eligible. |
| T04 / `phase_b_temporal_clock_conflict_is_typed` | integration | `temporal/clock_mismatch.json` | clock IDs | TM-01 | `MechanismEvidenceInputError::ClockScopeConflict`. |
| T04 / `phase_b_temporal_scope_conflict_is_typed` | integration | `temporal/scope_mismatch.json` | sensor scope | TM-01 | `MechanismEvidenceInputError::SensorScopeConflict`. |
| T04 / `phase_b_temporal_aggregate_unknown_is_missing_evidence` | integration | `temporal/aggregate_unknown.json` | experiment scopes | TM-01--TM-04 | `MechanismAssessmentError::Temporal(AggregateOrUnknownSupport)`. |
| T05 / `phase_b_identifiability_covariate_satisfies` | integration | `identifiability/satisfied.json` | covariate/response arrays | ID-01--ID-03 | Satisfied. |
| T05 / `phase_b_identifiability_covariate_below_range_fails` | integration | `identifiability/not_satisfied.json` | covariate array/range | ID-01--ID-03 | `MechanismAssessmentError::RequirementNotSatisfied { requirement_id="b-ident-requirement-01", reason=IdentifiabilityNotSatisfied }`. |
| T05 / `phase_b_identifiability_missing_source_is_not_assessed` | integration | `identifiability/not_assessed.json` | covariate source | ID-01--ID-03 | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-ident-requirement-01", reason=MissingRequiredEvidence }`. |
| T05 / `phase_b_identifiability_custom_is_not_assessed` | integration | `identifiability/custom_unsupported.json` | requirement kind | none | `MechanismAssessmentError::RequirementNotAssessed { requirement_id="b-custom", reason=MissingRequiredEvidence }`. |
| T06 / `phase_b_validation_passes_and_promotes_domain` | integration | `validation/protocol_pass.toml`, `validation/pass.json` | conditions, family, lineage, values | VP-01 | Satisfied; ValidatedForDomain. |
| T06 / `phase_b_validation_insufficient_families_is_typed` | integration | `validation/protocol_insufficient_families.toml`, `validation/insufficient.json` | family, condition | VP-01 | `ValidationProtocolError::InsufficientIndependentAcquisitionFamilies { required: 3, actual: 1 }`. |
| T06 / `phase_b_validation_unknown_family_is_typed` | integration | `validation/protocol_pass.toml`, `validation/unknown_family.json` | unknown family, conditions | VP-01 | `InsufficientIndependentAcquisitionFamilies { required: 2, actual: 0 }`. |
| T06 / `phase_b_validation_training_overlap_is_typed` | integration | `validation/protocol_pass.toml`, `validation/training_overlap.json` | validation/support family | VP-01 | `InsufficientIndependentAcquisitionFamilies { required: 2, actual: 0 }`. |
| T08 / `phase_b_strong_critical_contradiction_blocks_before_support_filtering` | integration | `promotion/strong_critical_contradiction.json` | all three evidence IDs, directions, strengths, validity, availability, quantity, family | PR-01--PR-04 | `StrongCriticalContradiction { requirement_id="b-critical-requirement-01", evidence_ids=["b-critical-01"] }`; proves two Strong Supports cannot hide one Strong Contradicts. |
| T07 / `phase_b_mechanism_compare_e2e_writes_and_rereads_expected_analysis` | end-to-end | all six `e2e/` files in §22.8 | artifact kind/version/scope/lineage, every named evidence value and expected ID | all §22.8 keys | public CLI succeeds; expected schema-4 analysis exactly matches; one history entry. |

### 22.11 Final self-audit

```text
Undefined normative types: 0
Unspecified Phase B algorithms: 0
Unspecified scientific thresholds: 0
Unspecified compatibility decisions: 0
Normative contradictions: 0
Implementation invention still required: no
```

Could two competent implementation agents differ about required experiment scopes, the critical contradiction candidate set, config file ownership, validation optionality, CLI precedence, validation fixture contents, or negative error variants? **NO** for every question.

## 23. Phase B Contract Remediation V — controlling executable contract

**Authority and supersession.** This section is the sole controlling contract for
Phase B. It supersedes every Phase-B rule, type, fixture, CLI table, test table,
and self-audit in §§6--10, 14--18, and 20--22. Earlier A1 sections remain frozen
and are not reinterpreted. In particular, Phase B must not add a field to, change
the serialized meaning of, or change the semantic hash of `EvidenceRecord`,
`EvidenceQuantity`, `EvidencePairKey`, `ArtifactLineageState`,
`TimescalePairUncertainty`, or A1 `EvidenceBundle`.

This remediation is documentation only. It authorizes a later implementation
agent to create the Phase-B types and tests named below, but not to alter A1 or
the legacy mechanism path. The implementation traceability document has one
fixed path: `docs/engineering_specification/phase_b_mechanism_evidence_traceability.md`.
The implementation agent must create or update exactly that path; this planning
amendment does not create a placeholder for it.

### 23.1 PB-HO-01 — one serialized hypothesis owner

`mechanism compare --mechanism-evidence-config <PATH>` is the only serialized
owner of Phase-B hypothesis definitions. It reads exactly one TOML document.
The legacy `--config` remains the owner of legacy comparison settings only;
it must neither deserialize, default, override, nor merge any Phase-B field.
There is no second hypothesis-definition file and no legacy `hypotheses`
ownership for Phase-B assessment.

```rust
pub struct MechanismEvidenceConfig {
    pub schema_version: u32, // exactly 1
    pub hypotheses: Vec<MechanismHypothesisDefinition>,
    pub temporal: TemporalAssessmentPolicy,
}
pub struct MechanismHypothesisDefinition {
    pub hypothesis_id: HypothesisId,
    pub target_components: Vec<ComponentId>,
    pub evidence_requirements: Vec<EvidenceRequirementBinding>,
    pub critical_requirement_ids: Vec<EvidenceRequirementId>,
    pub gates: HypothesisGates,
    pub identifiability_requirements: Vec<IdentifiabilityBinding>,
    pub validation_applicability: ValidationApplicability,
    pub role_bindings: Vec<EvidenceRoleBinding>,
}
pub struct EvidenceRequirementBinding {
    pub requirement_id: EvidenceRequirementId,
    pub role: MechanismEvidenceRole,
    pub target: EvidenceTarget,
    pub source_class: EvidenceSourceClass,
    pub field_path: String,
    pub quantity_semantic: PhaseBQuantitySemantic,
    pub required_unit: String,
    pub direction: RequiredEvidenceDirection,
    pub required: bool,
}
pub struct HypothesisGates {
    pub timescale: Option<TimescaleGate>,
    pub amplitude: Option<AmplitudeGate>,
    pub repeatability: Option<RepeatabilityGate>,
}
```

The remaining closed Phase-B types used above are defined here, rather than
inferred from a legacy similarly named type:

```rust
pub struct TemporalAssessmentPolicy { pub point_tolerance_s: f64 }
pub enum PhaseBQuantitySemantic {
    TimeConstant, ElectricalPotential, CalibrationPotential, ComponentScalar,
}
pub enum RequiredEvidenceDirection { CandidatePresence, TimescalePairMember }
pub struct TimescaleGate {
    pub left_requirement_id: EvidenceRequirementId,
    pub right_requirement_id: EvidenceRequirementId,
    pub maximum_log_distance: f64,
}
pub struct IdentifiabilityBinding {
    pub requirement_id: RequirementId,
    pub kind: IdentifiabilityRequirementKind,
    pub threshold: f64,
}
pub struct ValidationApplicability {
    pub required: bool,
    pub validation_requirement_ids: Vec<EvidenceRequirementId>,
    pub minimum_independent_validation_families: usize,
}
pub enum MechanismEvidenceRole { Support, Validation, Calibration, Training }
pub struct EvidenceRoleBinding {
    pub evidence_id: EvidenceId,
    pub role: MechanismEvidenceRole,
}
pub enum ExpectedEffect { Increase, Decrease, SameSign }
pub enum RepeatabilityStatus { Satisfied, NotSatisfied, NotAssessed }
pub enum AmplitudeReasonCode {
    MissingCandidate, AmbiguousCandidate, InvalidUnit, UnitMismatch,
    DirectionMismatch, RelativeErrorExceeded,
}
pub enum RepeatabilityReasonCode {
    MissingCandidate, ScopeMismatch, IncompatibleUnit,
    InsufficientIndependentRecords, UnknownAcquisitionFamily,
}
```

All TOML fields represented above are required when their containing table is
present; `hypotheses` is nonempty; all IDs are nonempty and unique within their
declared scope; each target component is unique; each `field_path` is a literal
JSONPath emitted by the listed adapter; and unknown TOML fields are rejected.
`None` means that a gate does not apply. A required requirement with no eligible
record is `NotAssessed`, never silently omitted. The exact TOML root is
`schema_version = 1`, `[[hypotheses]]`, then per-hypothesis
`[[hypotheses.evidence_requirements]]`, optional `[hypotheses.gates.timescale]`,
`[hypotheses.gates.amplitude]`, `[hypotheses.gates.repeatability]`, repeated
`[[hypotheses.identifiability_requirements]]`, and
`[hypotheses.validation_applicability]`. The TOML representation uses the
serde snake-case form of the types in this section.

`TimescaleGate.maximum_log_distance` and every identifiability threshold are
finite and `>=0`; a timescale gate selects exactly one left and one right
TimeConstant candidate and is Satisfied iff `abs(ln(left/right)) <=` its
threshold, NotSatisfied iff both candidates exist and exceed it, otherwise
NotAssessed. `ValidationApplicability.required=false` requires an empty
validation requirement list and `minimum_independent_validation_families=0`.
When true, the list is nonempty, unique, and `minimum_independent_validation_families>=1`.
Each role binding ID is unique and resolves to exactly one named requirement.
`TemporalAssessmentPolicy.point_tolerance_s` is finite and `>=0` seconds.
`CandidatePresence` is the only legal direction for a single-value requirement;
`TimescalePairMember` is legal only for one of the two requirements named by a
timescale gate. Directional scientific claims are made only by an
`AmplitudeGate` and its explicit `ExpectedEffect`; Phase B does not inspect or
overwrite A1 `EvidenceRecord.direction` or `EvidenceRecord.strength`.

Promotion is deterministic. Start at `Hypothesized`. A hypothesis is
`ExperimentallySupported` iff every `required=true` Support binding has exactly
one eligible candidate, every present gate is Satisfied, every required
identifiability binding is Satisfied, and no critical requirement is missing or
belongs to a NotSatisfied gate. Otherwise it remains `Hypothesized` when any
required item is NotAssessed and becomes `Unassessed` only when config itself
is invalid. `ValidatedForDomain` is considered only after
`ExperimentallySupported` and follows §23.8. Validation failure never
downgrades the support result below `ExperimentallySupported`.

### 23.2 PB-HO-02 — exact quantity semantics without an A1 change

`EvidenceQuantity` remains exactly `{ value, unit, uncertainty }`; it has no
`quantity_kind`. Phase B therefore uses the following literal, closed mapping.
It is structural enum matching, not a display-name, unit-only, field-path, or
component-name inference:

| `EvidenceTarget` variant | Phase-B target semantic | Eligibility |
|---|---|---|
| `ModelComponent(ComponentId)` | `ComponentScalar` | eligible only when an `EvidenceRequirementBinding` has structural equality for this exact variant and component ID |
| `MechanismHypothesis(HypothesisId)` | unmapped | ineligible |
| `HealthFinding(HealthFindingId)` | unmapped | ineligible |
| `HealthDimension(HealthDimension)` | unmapped | ineligible |
| `IdentifiabilityRequirement(RequirementId)` | unmapped | ineligible as measurement evidence; it is consumed only by the separate `IdentifiabilityBinding` route in §23.9 |

`PhaseBQuantitySemantic` is a Phase-B requirement property, never an A1
field: `TimeConstant`, `ElectricalPotential`, `CalibrationPotential`, or
`ComponentScalar`. A binding is eligible only if its target maps to
`ComponentScalar` above, its exact source class and exact source field path
match, its value is finite and `Available`, its unit validates through
`validate_ucum_unit`, and its declared semantic/unit pair is one of:

| semantic | accepted candidate units | required-unit rule |
|---|---|---|
| `TimeConstant` | `s` | required unit is exactly `s` |
| `ElectricalPotential` or `CalibrationPotential` | `V`, `mV`, `µV` | required unit is one of `V`, `mV`, `µV` |
| `ComponentScalar` | `1`, `dimensionless`, or exactly equal validated unit | `1`/`dimensionless` are mutually convertible; any other unit must equal the required unit byte-for-byte |

Any other target, semantic, unit, availability, duplicate exact candidate, or
unit mismatch is ineligible and records a typed Phase-B reason; there is no
fallback selection. This is the complete Phase-B quantity mapping.

### 23.3 PB-HO-03 and PB-HO-04 — source assembly and legally producible E2E

`--evidence-artifact` is **RETIRED**. `EvidenceBundle` is an in-memory A1
assembly object, not a `VersionedArtifact`; it has no artifact kind, artifact
envelope, independent `ArtifactId`, standalone reader, fixture grammar, or
migration route. Phase B receives only individually versioned source artifacts,
uses `runners::evidence::assemble_evidence_bundle`, then assesses that transient
bundle in the same command invocation.

The only V1 source inputs and adapter outputs are below. IDs are generated by
the cited current adapter, not supplied by a fixture author.

| CLI input / artifact kind | approved adapter and exact `EvidenceId` | exact target / source field | unit / temporal support |
|---|---|---|---|
| `--eis-fit`, `eis_fit` | `adapt_eis_fit`: `eis.parameter.{parameter_index}` | `ModelComponent(parameters[index].element_id)` / `$.parameters[index].value` | serialized parameter unit, or `1` only when it is empty / `Unknown` |
| `--transient-results`, `transient_analysis` | `adapt_transient_analysis`: `transient.event.{event_index}.parameter.{parameter_index}` | `ModelComponent(parameter.name)` / `$.events[event_index].candidate_fits[].parameters[parameter_index].value` | serialized parameter unit / `Window` from the selected event's `time_local` bounds only when nonempty finite and ordered; otherwise `Unknown` |
| `--transient-results`, `transient_analysis` | `adapt_transient_analysis`: `transient.event.{event_index}.tau_fast_s` or `.tau_slow_s` | `ModelComponent(tau_fast_s|tau_slow_s)` / exact `derived_features` path | `s` / same event-window rule |
| `--calibration-observations`, `calibration_observations` | `try_adapt_calibration_observations`: `calibration.observation.{index}` | `ModelComponent(observation.analyte)` / `$.observations[index].potential_v` | `V` / `Unknown` |
| `--estimation-results`, `state_estimation` | `adapt_state_estimation`: `estimation.point.{point_index}.state.{state_index}` | `ModelComponent(filtered_state[state_index].name)` / `$.estimates[point_index].filtered_state[state_index].value` | serialized state unit / `Point` at `timestamp_s` only when finite; clock is the serialized estimation timestamp basis |
| `--model-analysis`, `ism_model_analysis` | `adapt_model_analysis`: `model.point.{point_index}.component.{component_index}` | `ModelComponent(contribution.component_id)` / `$.points[point_index].contributions[component_index].potential_v` | `V` / `Point` at `points[point_index].time_s` only when finite; clock is model relative-time basis |

The current assembly function must be extended only to call the already-approved
model adapter when `--model-analysis` is present. `--calibration-results` and a
stored calibration model remain computational context, not evidence, because
there is no approved calibration-model adapter. Signal, health, arbitrary JSON,
and model-validation artifacts are not Phase-B source inputs. Source order is
irrelevant: the A1 builder's canonical ordering controls the assembled bundle.

Phase B must not manufacture direction, strength, validity, an EvidenceId, a
timestamp, a role, or a model evidence record. It may persist its own assessment
of an immutable record. The production E2E scenarios are:

| scenario | legal inputs and generated IDs | required result |
|---|---|---|
| E2E-1 `ExperimentallySupported` | one EIS parameter with unit `s` (`eis.parameter.0`) and one selected transient derived `tau_fast_s` (`transient.event.0.tau_fast_s`), each bound by exact target/source/field-path requirements; no records have the `Validation` role | all required support gates pass; validation is absent or `NotAssessed`; result is exactly `ExperimentallySupported`, never `ValidatedForDomain` |
| E2E-2 `ValidatedForDomain` | E2E-1 inputs plus one `calibration_observations` record (`calibration.observation.0`) and one `ism_model_analysis` contribution (`model.point.0.component.0`), originating from distinct known independent acquisition families; each is explicitly role-bound as `Validation` and bound to a separate validation requirement | support gates pass, each validation requirement has an eligible Validation-role record, the two validation families are independent from each other and from support families; result is `ValidatedForDomain` |

The E2E test creates real serialized source artifacts through their production
types and reads them through `read_artifact`; it does not serialize an
`EvidenceBundle` or inject IDs/fields that a named adapter cannot emit.

### 23.4 PB-HO-05 — temporal authority

```rust
pub enum EvidenceTemporalSupport {
    Point { timestamp_s: f64, clock_basis: TemporalClockBasis },
    Window { start_s: f64, end_s: f64, clock_basis: TemporalClockBasis },
    Unknown,
}
pub enum TemporalClockBasis { EstimationTimestamp, ModelRelativeTime, TransientRelativeTime }
pub enum TemporalAssessmentReason {
    UnknownSupport, ClockMismatch, ScopeMismatch, PointToleranceExceeded,
    WindowNoPositiveOverlap, PointOutsideWindow, AmbiguousNearestPoint,
}
```

`EisFitArtifact` has no authoritative timestamp, so every EIS-derived record
has `EvidenceTemporalSupport::Unknown`. No implementation may derive an EIS
time from file metadata, wall clock, runner time, neighboring artifacts, or
experiment ID. A point temporal join must never require EIS. Point tests use
only estimation and/or model source records with their documented timestamps.
Transient event support is a window, not a fabricated point; calibration is
unknown. A temporal-required binding accepts only equal clock bases and either
two points within the configured inclusive tolerance, two windows with positive
half-open overlap, or a point lying in a window. Any Unknown, aggregate, clock
mismatch, scope mismatch, or multiple equally-near points is `NotAssessed`.

### 23.5 PB-HO-06/07 — amplitude assessment and unit-bearing threshold

The global bare `amplitude_floor` is retired. Each amplitude gate contains:

```rust
pub struct AmplitudeThreshold { pub value: f64, pub unit: String }
pub struct AmplitudeGate {
    pub predicted_requirement_id: EvidenceRequirementId,
    pub observed_requirement_id: EvidenceRequirementId,
    pub expected_effect: ExpectedEffect, // Increase, Decrease, SameSign
    pub maximum_relative_error: f64,
    pub threshold: AmplitudeThreshold,
}
pub enum AmplitudeStatus { Satisfied, NotSatisfied, NotAssessed }
pub struct AmplitudeAssessment {
    pub status: AmplitudeStatus,
    pub predicted_evidence_id: Option<EvidenceId>,
    pub observed_evidence_id: Option<EvidenceId>,
    pub threshold: AmplitudeThreshold,
    pub predicted_value_in_threshold_unit: Option<f64>,
    pub observed_value_in_threshold_unit: Option<f64>,
    pub relative_error: Option<f64>,
    pub reason_codes: Vec<AmplitudeReasonCode>,
}
```

Threshold value is finite and `>0`; `maximum_relative_error` is finite and
`>=0`; the threshold unit must pass `validate_ucum_unit`. Candidate selection
uses the binding algorithm in §23.2 separately for the predicted and observed
requirement. Exactly one candidate on each side is required; zero or more than
one yields `NotAssessed` (`MissingCandidate` or `AmbiguousCandidate`). Both
units must be compatible with the threshold. Conversion is exact: V→V is 1,
mV→V is `1e-3`, µV→V is `1e-6` (and inverses); `s`, `1`, and
`dimensionless` only use the conversions in §23.2. No other conversion is V1.

Let converted values be `p` and `o`, and threshold value be `f`. The direction
condition is `o-p > 0` for Increase, `o-p < 0` for Decrease, and `p*o > 0` for
SameSign. The error is `abs(p-o) / max(abs(p), abs(o), f)`. Uncertainty does
not enlarge or shrink this deterministic V1 comparison; it is persisted as
source context only. The result is Satisfied iff the direction condition holds
and error `<= maximum_relative_error`; it is NotSatisfied if both candidates
are valid and either condition fails; otherwise NotAssessed. The assessment is
persisted in the Phase-B report only and never writes to A1 evidence.

### 23.6 PB-HO-06B — repeatability assessment

```rust
pub struct RepeatabilityGate {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub minimum_count: usize, // >= 2
    pub maximum_log_tau_sample_standard_deviation: f64, // >= 0
}
pub struct RepeatabilityAssessment {
    pub status: RepeatabilityStatus,
    pub selected_evidence_ids: Vec<EvidenceId>,
    pub grouping_key: RepeatabilityGroupingKey,
    pub sample_standard_deviation_ln_tau: Option<f64>,
    pub reason_codes: Vec<RepeatabilityReasonCode>,
}
pub struct RepeatabilityGroupingKey {
    pub hypothesis_id: HypothesisId,
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub target_components: Vec<ComponentId>,
    pub experiment_scope: EvidenceExperimentScope,
    pub sensor_scope: ScopeKey,
    pub channel_scope: ScopeKey,
}
```

Eligible records are the union of the named bindings after §23.2 selection,
with semantic `TimeConstant`, exact unit `s`, positive finite value, valid
scope equal to the first eligible record, and a known acquisition family. Sort
by EvidenceId. Retain the lexicographically first maximum-cardinality subset
whose pairs have A1 `EvidenceIndependence::Independent` and whose acquisition
family sets are disjoint; ties use the sorted EvidenceId vector. Unknown,
same-source, partially dependent, or missing pair independence never counts.
The grouping key above is exact; records from another target, scope, channel,
or hypothesis do not pool.

With selected values `tau_i`, compute `x_i=ln(tau_i / 1 s)`, mean
`x_bar=sum(x_i)/n`, and sample SD
`sqrt(sum((x_i-x_bar)^2)/(n-1))`. Population SD is prohibited. Fewer than
`minimum_count`, no independent subset, incompatible unit, or absent scope is
`NotAssessed` with the corresponding reason code. Otherwise `<=` threshold is
Satisfied and `>` is NotSatisfied. It is a Phase-B-owned persisted assessment;
no A1 record changes.

### 23.7 PB-HO-08 — exact mechanism schema 3 → 4 migration

`MechanismAnalysisReport` is a `VersionedArtifact` of kind
`mechanism_analysis`, current schema 3 and legacy schemas 1/2 before B. Phase
B advances only this artifact to current schema 4, with readable legacy
versions `[1,2,3]`. Artifact identity remains in the existing outer artifact
envelope/lineage; the report payload must not gain an `artifact_id` field.

```rust
pub enum HypothesisEvidenceLevel {
    Unassessed, Hypothesized, ExperimentallySupported, ValidatedForDomain,
}
pub struct PhaseBHypothesisAssessment {
    pub hypothesis_id: HypothesisId,
    pub level: HypothesisEvidenceLevel,
    pub supporting_evidence_ids: Vec<EvidenceId>,
    pub contradictory_evidence_ids: Vec<EvidenceId>,
    pub excluded_evidence_ids: Vec<EvidenceId>,
    pub amplitude: Option<AmplitudeAssessment>,
    pub repeatability: Option<RepeatabilityAssessment>,
    pub identifiability: Vec<IdentifiabilityAssessment>,
    pub validation_status: ValidationStatus,
    pub reason_codes: Vec<PhaseBHypothesisReasonCode>,
}
pub struct PhaseBHypothesisHistory {
    pub hypothesis_id: HypothesisId,
    pub prior_level: HypothesisEvidenceLevel,
    pub new_level: HypothesisEvidenceLevel,
    pub assessment_index: u64,
    pub reason_codes: Vec<PhaseBHypothesisReasonCode>,
}
pub struct IdentifiabilityAssessment {
    pub requirement_id: RequirementId,
    pub status: IdentifiabilityStatus,
    pub metric: Option<f64>,
    pub unit: String,
    pub source_evidence_ids: Vec<EvidenceId>,
}
pub enum IdentifiabilityStatus { Satisfied, NotSatisfied, NotAssessed, NotApplicable }
pub enum ValidationStatus { NotApplicable, NotAssessed, Satisfied, NotSatisfied }
pub enum PhaseBHypothesisReasonCode {
    MissingRequiredEvidence, GateNotAssessed, GateNotSatisfied,
    CriticalContradiction, IdentifiabilityNotSatisfied,
    ValidationAbsent, ValidationNotAssessed, ValidationNotSatisfied,
}
```

All ID vectors serialize bytewise sorted and duplicate-free. Assessment order
is bytewise hypothesis-ID order. A history row is appended only when a prior
Phase-B assessment for the same hypothesis is an explicit input to a later
Phase-B run; otherwise the first assessment has no history row. Phase B never
rewrites a legacy hypothesis assessment into the new type.

| schema-3 payload field | schema-4 disposition |
|---|---|
| `schema_version` | write `4`; readers accept `1`, `2`, `3`, and `4` |
| `lineage`, `analysis_id`, `records`, `eis_timescales`, `transient_timescales`, `comparisons`, `trends`, `configuration`, `provenance`, `warnings`, `transient_configuration` | retained byte-for-byte in meaning and serde shape |
| `hypotheses: Vec<HypothesisAssessment>` | retained as `legacy_hypotheses: Vec<HypothesisAssessment>` with `#[serde(default)]`; no information is dropped or reinterpreted |
| absent Phase-B assessment | add `hypothesis_assessments: Vec<PhaseBHypothesisAssessment>` with `#[serde(default)]`; legacy reads get `[]` |
| absent Phase-B history | add `hypothesis_history: Vec<PhaseBHypothesisHistory>` with `#[serde(default)]`; legacy reads get `[]` |
| absent role bindings/temporal/amplitude/repeatability detail | reside inside the new Phase-B assessment/history records only; legacy reads contain no fabricated values |

On reading schemas 1--3, serde defaults retain existing fields and produce the
empty new vectors. Writing any report through the Phase-B writer emits schema
4, preserves `legacy_hypotheses`, and emits both new vectors (possibly empty).
The schema-4 semantic hash is computed under the existing artifact semantic
identity policy over every retained and new serialized field. A schema-3 hash
is never claimed to equal its schema-4 rewrite; no historical Phase-B decision,
role, timestamp, or identity is fabricated. Artifact kind remains
`mechanism_analysis` and still follows `CurrentArtifactKindPolicy::Required`.

### 23.8 PB-HO-09 — Phase-B role bindings and validation

`EvidenceRecord` has no role and remains unchanged. The config owns an explicit
binding keyed by generated `EvidenceId`:

`MechanismEvidenceRole` and `EvidenceRoleBinding` are exactly the closed
config-owned types defined in §23.1.

Each role binding is serialized inside the owning hypothesis definition; an ID
may occur once only. Before assessment, every binding must resolve to exactly
one assembled A1 record and that record must also satisfy the corresponding
requirement's exact target/source-class/field-path constraints. A missing,
ambiguous, or mismatched binding is a typed configuration/input error. There is
no artifact-kind role inference. Unbound evidence is `Unknown` for role
purposes and does not count in any validation family.

`ValidatedForDomain` additionally requires a present validation applicability
table, every named validation requirement satisfied by a Validation-role record,
and the selected validation records to form an A1-independent subset with the
configured minimum count and no shared known acquisition family with selected
Support or Training records. Calibration-role evidence can satisfy only a
calibration requirement, not a validation requirement. Failure or absence
downgrades only to `ExperimentallySupported` when all support gates pass; it
does not invalidate support evidence.

### 23.9 Identifiability, conflicting evidence, and exact errors

For each supported identifiability binding, configuration contains the model
`requirement_id`, its kind, threshold, and one declared input. The complete V1
mapping is:

| kind | input artifact / field | metric and threshold | status rules |
|---|---|---|---|
| `ObservationDurationRelativeToTimescale` | selected transient event `$.events[i].segment.fitted_time_local` and selected `TimeConstant` record | duration `(last-first) / tau`, threshold `minimum_duration_ratio` | finite ordered nonempty interval and ratio `>=` threshold → Satisfied; available values below → NotSatisfied; missing/nonfinite/Unknown → NotAssessed |
| `ModeSeparation` | two selected `TimeConstant` records | `max(tau)/min(tau)`, threshold `minimum_mode_separation_ratio` | finite positive values and ratio `>=` threshold → Satisfied; below → NotSatisfied; otherwise NotAssessed |
| `TransientExcitation` | selected transient event `$.events[i].concentration_before` and `.concentration_after` | `abs(log10(after/before))`, threshold `minimum_absolute_log10_concentration_step` | positive finite values and metric `>=` → Satisfied; below → NotSatisfied; otherwise NotAssessed |

All other known kinds (`ActivityExcitation`, `ReferenceAnchor`,
`IndependentCovariateVariation`, `InterferentVariation`, `TemperatureVariation`,
`RepeatedStandards`, `AuxiliaryObservation`) and `Custom(_)` are `NotAssessed`
in V1 because no approved adapter exposes their required metric. Only an
explicitly inapplicable binding is `NotApplicable`; no unavailable metric is
converted to that status.

Conflicting evidence is supplied through the same source flags and assembly
route in §23.3. Accepted kinds are exactly `eis_fit`, `transient_analysis`,
`calibration_observations`, `state_estimation`, and `ism_model_analysis`.
Each input flag accepts exactly one path; a repeated flag is a CLI parse error.
Duplicate `EvidenceId`s from assembled sources are rejected by the A1 builder;
duplicate role bindings are rejected by config validation; direct-conflict
classification is defined exclusively by §24.11. Scope equality is mandatory. The
typed errors are `UnsupportedEvidenceSourceArtifact`, `DuplicateSourceInput`,
`DuplicateEvidenceId`, `RoleBindingUnresolved`, `RoleBindingMismatch`,
`ScopeMismatch`, and `ConflictingEvidence`; no generic bundle-file error exists.

### 23.10 Final Phase-B CLI, fixtures, traceability, and audit

The complete Phase-B `mechanism compare` interface is:

| flag | status and meaning |
|---|---|
| `--mechanism-evidence-config PATH` | required for Phase-B assessment; sole owner of hypotheses, thresholds, roles, validation applicability, and identifiability bindings |
| `--eis-fit PATH` | optional approved EIS source |
| `--transient-results PATH` | optional approved transient source |
| `--calibration-observations PATH` | optional approved calibration-observation source |
| `--estimation-results PATH` | optional approved state-estimation source |
| `--model-analysis PATH` | optional approved model-analysis source |
| `--config PATH` | legacy/general comparison config only; cannot affect Phase B |
| `--output PATH` | mechanism-analysis output |
| `--evidence-artifact` | retired; parse rejection with migration guidance |
| `--validation-protocol` | retired; parse rejection because validation applicability is config-owned |

The legacy `mechanism compare` invocation without
`--mechanism-evidence-config` continues to run only legacy comparison and
schema-3 output. Supplying the evidence config selects Phase B and schema-4
output. A Phase-B config that names a missing required source is a typed input
error, not a legacy fallback.

Fixtures are source-artifact fixtures only. Each fixture registry row in the
implementation traceability document must state the exact envelope
(`artifact_kind`, readable schema version, lineage), real payload fields,
scope, adapter-generated EvidenceIds, temporal support, explicit role,
assessment, and expected typed error. Retire all direct-bundle JSON grammar and
all former fixtures that assign `artifact_id`, timestamp, direction, strength,
or role directly to an evidence record. Required exact test functions are:
`phase_b_e2e_experimentally_supported_from_sources`,
`phase_b_e2e_validated_for_domain_from_sources`,
`phase_b_eis_temporal_support_is_unknown`,
`phase_b_point_temporal_join_uses_estimation_or_model_source`,
`phase_b_amplitude_unit_threshold_and_direction`,
`phase_b_repeatability_uses_sample_sd_and_independent_families`,
`phase_b_schema3_to_schema4_preserves_legacy_hypotheses`, and
`phase_b_validation_counts_only_explicit_roles`.

The traceability file at the exact fixed path in the opening paragraph has
these required columns: `Requirement ID`, `Acceptance Criterion`,
`Implementation Symbol`, `Production Execution Path`, `Exact Test Function`,
`Fixture/Data Source`, `Result`, `Compatibility Impact`, and `Scientific Risk`.

**Final self-audit.**

```text
Undefined normative types: 0
Undefined normative owners: 0
Unspecified Phase B algorithms: 0
Unspecified scientific thresholds/units: 0
Unspecified compatibility decisions: 0
Normative contradictions: 0
Fixture-to-real-schema contradictions: 0
Implementation invention still required: no
Frozen A1 compatibility: YES
```

Two competent implementation agents cannot differ about hypothesis definition
source, quantity semantics, direct EvidenceBundle support, amplitude selection
or units, repeatability grouping, EIS temporal handling, schema-3 hypothesis
migration, validation roles, or the traceability path: **NO** for each.

## 24. Phase B Contract Remediation VI — integrated executable bindings

**Authority and reconciliation.** This section is the controlling Phase-B
amendment after the integration of A1 into `main`.  It supersedes only the
conflicting Phase-B statements in §23 (in particular the one-field
`EvidenceRoleBinding`, deferred fixture registry, and undefined prior
conflict terminology).  All other §23 rules remain in force.
Sections 3--5 remain frozen.  This is a documentation-only amendment: it
does not authorize a change to production Rust, tests, fixtures, the A1
serialized schema, or A1 semantic identity.

### 24.1 Independent-finding reconciliation

| finding | classification | current evidence and disposition |
|---|---|---|
| PB-EX-01 temporal `EvidenceId` association | CONFIRMED | `EvidenceRecord` in `src/evidence.rs` has no temporal member and the §23 temporal policy has no keyed owner. §§24.2--24.4 add the Phase-B-only catalog. |
| PB-EX-02 requirement-scoped roles | CONFIRMED | §23.1/§23.8 key a role by `EvidenceId` alone. §§24.5--24.6 replace it with a hypothesis/requirement/evidence tuple. |
| PB-EX-03 identifiability inputs | CONFIRMED | §23.9 names artifact fields but does not bind exact requirement candidates. §24.7 supplies the closed bindings and metric table. |
| PB-EX-04 fixture/source registry | CONFIRMED | §23.10 assigns fixture semantics to a future traceability file. §§24.8--24.10 make the plan itself the normative registry. |
| PB-EX-05 conflicting-evidence outcomes | CONFIRMED | §23.9 used undefined conflict terminology. §24.11 defines direct and amplitude contradictions separately. |
| PB-HO-01 one hypothesis owner | ALREADY RESOLVED, rechecked | `MechanismEvidenceConfig.hypotheses` remains the sole serialized owner; every new binding below is nested under its owning `MechanismHypothesisDefinition`. |
| PB-HO-02 quantity semantics | ALREADY RESOLVED, rechecked | §23.2 continues to use structural `EvidenceTarget` mapping plus `validate_ucum_unit`; no A1 `quantity_kind` or field-path heuristic is added. |

### 24.2 PB-EX-01 — one exact temporal metadata owner

Phase B owns a separate, additive, non-A1 catalog.  It is constructed by the
**Phase-B preparation / evidence-assembly layer** immediately after
`runners::evidence::assemble_evidence_bundle` returns and before hypothesis
definition validation.  It is not a member of `EvidenceRecord`, is not added
to A1 `EvidenceBundle`, and does not participate in any A1 semantic hash.

```rust
pub struct EvidenceTemporalMetadata {
    pub evidence_id: EvidenceId,
    pub support: EvidenceTemporalSupport,
    pub provenance: TemporalSupportProvenance,
}
pub struct EvidenceTemporalMetadataCatalog {
    pub entries: BTreeMap<EvidenceId, EvidenceTemporalMetadata>,
}
pub enum EvidenceTemporalSupport {
    Point { timestamp_s: f64, clock_basis: TemporalClockBasis },
    Window { start_s: f64, end_s: f64, clock_basis: TemporalClockBasis },
    Unknown,
}
pub enum TemporalClockBasis {
    EstimationTimestamp,
    ModelRelativeTime,
    TransientRelativeTime,
}
pub struct TemporalSupportProvenance {
    pub adapter: String,
    pub source_artifact_kind: ArtifactKind,
    pub source_field_paths: Vec<String>,
    pub fallback: TemporalMetadataFallback,
}
pub enum TemporalMetadataFallback { None, UnknownNoAuthoritativeTime }
pub enum TemporalMetadataError {
    UnknownEvidenceId { evidence_id: EvidenceId },
    KeyValueEvidenceIdMismatch { map_key: EvidenceId, value_id: EvidenceId },
    MissingTemporalMetadata { evidence_id: EvidenceId },
    DuplicateTemporalMetadata { evidence_id: EvidenceId },
    NonFiniteTemporalSupport { evidence_id: EvidenceId },
    InvalidTemporalWindow { evidence_id: EvidenceId },
}
```

The one serialized representation is `BTreeMap<EvidenceId,
EvidenceTemporalMetadata>` in the Phase-B preparation object.  It is never a
parallel vector.  The map key and `value.evidence_id` must be bytewise equal,
each key must resolve to `EvidenceBundle.records`, and every record consumed by
a Phase-B hypothesis has exactly one entry.  A record with no lawful source
gets an explicit `Unknown` entry; omission is never an alias for Unknown.
Map construction from any future deserialized entry list must reject repeated
IDs as `DuplicateTemporalMetadata` before forming the map.  Unknown IDs,
missing entries, non-finite points, and `start_s >= end_s` are the exact typed
errors above.

The only temporal API is therefore:

```rust
pub fn evaluate_temporal_join(
    left: EvidenceId,
    right: EvidenceId,
    temporal_metadata: &EvidenceTemporalMetadataCatalog,
    config: &TemporalJoinConfig,
) -> TemporalJoinAssessment;
```

It may inspect the two supplied IDs, the supplied catalog, and the existing A1
scope/identity facts for those records.  It may not read artifacts, file mtimes,
paths, the wall clock, or a hidden timestamp cache.  Its point/window/equality
rules remain §23.4.  Missing catalog entries yield the typed validation error
above; lawful `Unknown` support yields `NotAssessed(UnknownSupport)`.

### 24.3 Exact source-field temporal mapping

This table is normative for every current adapter output.  Array indices are
the same zero-based indices used in the generated `EvidenceId`; no adapter may
substitute a neighboring record's time.

| artifact kind / adapter | adapter-produced ID target | support and authoritative source field(s) | clock identity | fallback |
|---|---|---|---|---|
| `eis_fit` / `adapt_eis_fit` | `eis.parameter.{i}` | `Unknown`; `EisFitArtifact` has no timestamp field | none | `UnknownNoAuthoritativeTime` |
| `transient_analysis` / `adapt_transient_analysis` selected-fit parameter or `tau_fast_s`/`tau_slow_s` | `transient.event.{i}.parameter.{j}`, `transient.event.{i}.tau_fast_s`, or `.tau_slow_s` | `Window { start_s=fitted_time_local[0], end_s=fitted_time_local[last] }` only when `$.events[i].segment.fitted_time_local` is nonempty, every value is finite, strictly increasing in serialized order, and first `<` last | `TransientRelativeTime` | explicit `Unknown` |
| `state_estimation` / `adapt_state_estimation` | `estimation.point.{i}.state.{j}` | `Point { timestamp_s=$.estimates[i].timestamp_s }` only when finite | `EstimationTimestamp` | explicit `Unknown` |
| `ism_model_analysis` / `adapt_model_analysis` | `model.point.{i}.component.{j}` | `Point { timestamp_s=$.points[i].time_s }` only when finite | `ModelRelativeTime` | explicit `Unknown` |
| `calibration_observations` / `try_adapt_calibration_observations` | `calibration.observation.{i}` | `Unknown`; `$.observations[i].timestamp` has no declared clock identity in the current public contract, so it is not a lawful Phase-B join time | none | `UnknownNoAuthoritativeTime` |

This defines the production flow exactly:

```text
versioned source artifact → A1 EvidenceRecord → artifact-specific temporal binding
→ same generated EvidenceId → temporal_metadata[EvidenceId]
```

No generic hypothesis logic reconstructs timestamps.  In particular, EIS
remains Unknown; it is never given time from an experiment ID, provenance,
file metadata, runner time, or another artifact.

### 24.4 Temporal construction and validation order

The preparation layer performs this deterministic sequence: (1) read the
versioned source artifacts through their public readers; (2) assemble and
validate the A1 bundle; (3) visit `bundle.records` in bytewise `EvidenceId`
order; (4) dispatch by the record's `source.artifact.artifact_kind` and exact
adapter ID grammar in §24.3; (5) insert exactly one catalog entry; (6) validate
the catalog against the immutable bundle.  A record with an unsupported
artifact kind is not Phase-B-consumable and is rejected before assessment as
`UnsupportedEvidenceSourceArtifact`; it is never given guessed temporal
support.

### 24.5 PB-EX-02 — requirement-scoped role binding

`EvidenceRoleBinding` is retired.  The only role type and semantic key are:

```rust
pub type MechanismHypothesisId = HypothesisId;
pub struct MechanismEvidenceRoleBinding {
    pub hypothesis_id: MechanismHypothesisId,
    pub requirement_id: EvidenceRequirementId,
    pub evidence_id: EvidenceId,
    pub role: MechanismEvidenceRole,
}
// semantic key: (hypothesis_id, requirement_id, evidence_id)
```

`MechanismHypothesisDefinition.role_bindings` is
`Vec<MechanismEvidenceRoleBinding>` and is the one owner.  For serialization,
the `hypothesis_id` in every nested row must equal the containing hypothesis;
cross-hypothesis rows are rejected, never silently relocated.  The vector is
sorted on the three-part key before persistence.

Validation is exact: the hypothesis exists; the requirement belongs to that
hypothesis; the evidence ID resolves in the assembled bundle; the record is a
candidate under §23.2 for that exact requirement; and there is one role for a
tuple.  A duplicate tuple, whether its role agrees or disagrees, is
`DuplicateMechanismEvidenceRoleBinding`; a wrong owner is
`RoleBindingHypothesisMismatch`; missing requirement/evidence is
`RoleBindingUnresolved`; and an otherwise ineligible record is
`RoleBindingMismatch`.  These names replace the ambiguous generic role error.

`Support` may satisfy that referenced requirement.  `Validation` may count
only for the referenced validation requirement.  `Calibration` may satisfy a
calibration-only requirement but is excluded from validation-family counting.
`Training` is excluded from validation-family counting.  An unbound tuple is
not Validation and cannot be inferred from artifact kind, source class, or
field path.

### 24.6 Validation-family rule

For a hypothesis, select only bindings where: (a) `hypothesis_id` is that
hypothesis, (b) `requirement_id` is in its
`ValidationApplicability.validation_requirement_ids`, (c) the role is exactly
`Validation`, and (d) the evidence remains eligible after all target, source,
quantity, scope, temporal, and validity filters.  Sort selected IDs bytewise,
deduplicate IDs only after rejecting duplicate tuple bindings, then use the
existing A1 pairwise `EvidenceIndependence::Independent` assessments and known
disjoint acquisition-family sets to choose the lexicographically first maximum
cardinality subset.  It passes only if its cardinality is at least
`minimum_independent_validation_families`.  Support, Calibration, Training,
and unbound records never contribute a validation family.

### 24.7 PB-EX-03 — explicit identifiability inputs

The §23.1 `IdentifiabilityBinding` is replaced by the following closed form;
`IdentifiabilityRequirementId` is a Phase-B alias of frozen A1 `RequirementId`.

```rust
pub type IdentifiabilityRequirementId = RequirementId;
pub struct IdentifiabilityBinding {
    pub identifiability_requirement_id: IdentifiabilityRequirementId,
    pub kind: IdentifiabilityRequirementKind,
    pub threshold: f64,
    pub input: IdentifiabilityInputBinding,
}
pub struct IdentifiabilityInputBinding {
    pub identifiability_requirement_id: IdentifiabilityRequirementId,
    pub input_requirement_ids: Vec<EvidenceRequirementId>,
    pub selection: IdentifiabilityInputSelection,
}
pub enum IdentifiabilityInputSelection {
    AllEligible,
    MutuallyIndependentSubset,
    ExactPair { pair_requirement_id: EvidenceRequirementId },
}
pub struct EvidencePairRequirement {
    pub pair_requirement_id: EvidenceRequirementId,
    pub left_requirement_id: EvidenceRequirementId,
    pub right_requirement_id: EvidenceRequirementId,
}
```

`MechanismHypothesisDefinition` additionally owns
`pair_requirements: Vec<EvidencePairRequirement>`.  IDs are unique across the
hypothesis's evidence and pair requirements.  Each identifiability ID is
unique; its nested and outer IDs must agree; every input requirement belongs to
the same hypothesis and is unique; and the selection must be permitted by the
table below.  `ExactPair` resolves the named pair requirement, whose two sides
must each have exactly one eligible candidate; it then produces the canonical
A1 `EvidencePairKey`.  Any missing or non-unique side is `NotAssessed`, not a
fallback pair.  No assessor may search arbitrary bundle records.

Candidate processing is common to all rows: collect only the named
requirements' eligible, role-authorized records; order by `EvidenceId`; remove
the same ID occurring through two named requirements only once; and reject
non-finite, unavailable, invalid, or unit-incompatible records before
selection. `AllEligible` returns the full ordered set. `MutuallyIndependentSubset`
uses A1 `largest_independent_subset` over that set. `ExactPair` uses only the
canonical pair just defined. Missing input, an Unknown independence result, or
a non-finite metric is `NotAssessed`; no candidate outside the binding enters.

| requirement kind | required input IDs / selection | count | formula and threshold field | Satisfied / NotSatisfied / NotAssessed / NotApplicable |
|---|---|---:|---|---|
| `ObservationDurationRelativeToTimescale` | one `TimeConstant` requirement / `AllEligible` | exactly 1 | `duration_s/tau_s`, where `duration_s=end_s-start_s` from that record's catalog `Window`; `minimum_duration_ratio` | finite positive window and ratio `>=` threshold / below / no exact record, no Window, or nonfinite / only explicitly inapplicable config |
| `ModeSeparation` | the two sides of one `EvidencePairRequirement` / `ExactPair` | exactly 2 | `max(tau)/min(tau)`; `minimum_mode_separation_ratio` | finite positive ratio `>=` threshold / below / missing or non-unique pair side / explicitly inapplicable config |
| `TransientExcitation` | `[]` / `AllEligible` | 0 | no approved A1 evidence adapter exposes before/after concentration pairs | never / never / always `NotAssessed(UnsupportedMetricInput)` / explicitly inapplicable config |
| `ActivityExcitation` | `[]` / `AllEligible` | 0 | no approved adapter | never / never / `NotAssessed(UnsupportedMetricInput)` / explicit only |
| `ReferenceAnchor` | `[]` / `AllEligible` | 0 | no approved adapter | never / never / `NotAssessed(UnsupportedMetricInput)` / explicit only |
| `IndependentCovariateVariation` | `[]` / `AllEligible` | 0 | no approved aligned covariate adapter | never / never / `NotAssessed(UnsupportedMetricInput)` / explicit only |
| `InterferentVariation` | `[]` / `AllEligible` | 0 | no approved adapter | never / never / `NotAssessed(UnsupportedMetricInput)` / explicit only |
| `TemperatureVariation` | `[]` / `AllEligible` | 0 | no approved adapter | never / never / `NotAssessed(UnsupportedMetricInput)` / explicit only |
| `RepeatedStandards` | `[]` / `AllEligible` | 0 | no approved adapter | never / never / `NotAssessed(UnsupportedMetricInput)` / explicit only |
| `AuxiliaryObservation` | `[]` / `AllEligible` | 0 | no approved adapter | never / never / `NotAssessed(UnsupportedMetricInput)` / explicit only |
| `Custom(_)` | `[]` / `AllEligible` | 0 | no registered custom assessor | never / never / `NotAssessed(UnsupportedMetricInput)` / explicit only |

Thresholds are finite and positive ratios.  `MutuallyIndependentSubset` is
reserved for a future registered metric and is invalid for all V1 rows above;
a config using it now is `InvalidIdentifiabilityInputSelection`.  The table is
therefore complete without inventing proxy scientific metrics.

### 24.8 PB-EX-04 — canonical Phase-B fixture registry

The normative fixture root is **`tests/fixtures/phase_b/`**.  These are new
future implementation fixtures; none exists on the frozen A1 branch.  Source
artifacts are serialized through their production types/readers and all
lineage is `Known`.  Every unlisted producer field uses the exact valid value
emitted by the named production constructor; it is not a fixture-only schema
extension.  The listed fields are the complete fields consumed by Phase B.

| fixture ID and literal path | kind/schema/current scope and acquisition family | literal consumed payload and adapter output | temporal / role / expected result | exact test IDs |
|---|---|---|---|---|
| PB-FX-01 `e2e/eis_fit_e2e_1.json` | `eis_fit`/3/current; experiment `b-e2e-1`, sensor `b-sensor-1`, channel `Unspecified`, family `b-family-eis` | `parameters[0]={element_id:"b-eis-tau",unit:"s",value:1.000}` → `eis.parameter.0`, target `ModelComponent("b-eis-tau")` | Unknown; Support for `b-eis-tau`; Independent with PB-FX-02; no direct contradiction | MHI-B-T07-e2e-1, MHI-B-T04-eis-unknown |
| PB-FX-02 `e2e/transient_analysis_e2e_1.json` | `transient_analysis`/3/current; experiment `b-e2e-1`, sensor/channel `b-sensor-1`/`b-channel-1`, family `b-family-transient` | selected event `0`; `segment.fitted_time_local=[0.0,10.0]`; successful fit `derived_features.tau_fast_s=1.000` → `transient.event.0.tau_fast_s`, target `ModelComponent("tau_fast_s")` | Window `[0,10]` TransientRelativeTime; Support for `b-transient-tau`; Independent with PB-FX-01 | MHI-B-T07-e2e-1, MHI-B-T05-duration |
| PB-FX-03 `e2e/calibration_observations_e2e_2.json` | `calibration_observations`/3/current; experiment `b-e2e-1`, sensor/channel `b-sensor-1`/`b-channel-1`, family `b-family-calibration` | `observations[0]={experiment_id:"b-e2e-1",analyte:"b-validation-calibration",potential_v:0.250,timestamp:20.0}` → `calibration.observation.0`, target `ModelComponent("b-validation-calibration")` | Unknown; Validation for `b-validation-calibration`; Independent with PB-FX-04 and support families | MHI-B-T07-e2e-2, MHI-B-T06-validation-pass |
| PB-FX-04 `e2e/model_analysis_e2e_2.json` | `ism_model_analysis`/5/current; experiment `b-e2e-1`, sensor/channel `b-sensor-1`/`b-channel-1`, family `b-family-model` | `points[0]={time_s:5.0,contributions:[{component_id:"b-validation-model",potential_v:0.250}]}` → `model.point.0.component.0` | Point `5.0` ModelRelativeTime; Validation for `b-validation-model` | MHI-B-T07-e2e-2, MHI-B-T06-validation-pass |
| PB-FX-05 `temporal/state_estimation_point.json` | `state_estimation`/4/current; experiment `b-temporal-1`, family `b-family-estimation` | `estimates[0].timestamp_s=4.0`; `filtered_state[0]={name:"b-state",value:2.0,unit:"1"}` → `estimation.point.0.state.0` | Point `4.0` EstimationTimestamp; Support | MHI-B-T04-estimation-point |
| PB-FX-06 `temporal/model_analysis_point.json` | `ism_model_analysis`/5/current; experiment `b-temporal-1`, family `b-family-model-temporal` | `points[0].time_s=4.0`; `contributions[0]={component_id:"b-model",potential_v:0.1}` → `model.point.0.component.0` | Point `4.0` ModelRelativeTime; Support; join is ClockMismatch with PB-FX-05 | MHI-B-T04-clock-mismatch |
| PB-FX-07 `temporal/transient_analysis_window.json` | `transient_analysis`/3/current; experiment `b-temporal-1`, family `b-family-transient-temporal` | event `0`, successful selected fit, `segment.fitted_time_local=[3.0,5.0]`, `tau_fast_s=2.0` → `transient.event.0.tau_fast_s` | Window `[3,5]` TransientRelativeTime; Support | MHI-B-T04-window |
| PB-FX-08 `negative/transient_nonmonotonic_time.json` | `transient_analysis`/3/current; experiment `b-temporal-2`, family `b-family-negative` | selected event `0`, `segment.fitted_time_local=[0.0,0.0,1.0]`, `tau_fast_s=1.0` → `transient.event.0.tau_fast_s` | explicit Unknown, not a synthesized Window; `NotAssessed(UnknownSupport)` | MHI-B-T04-invalid-window |
| PB-FX-09 `config/e2e_experimentally_supported.toml` | Phase-B config/1/current | hypothesis `b-hypothesis`; requirements `b-eis-tau`, `b-transient-tau`; pair `b-timescale-pair`; roles bind PB-FX-01/PB-FX-02 as Support; mode-separation threshold `1.0`; validation `required=false` | E2E-1 level `ExperimentallySupported`, component `ExperimentallySupported`, `ValidatedForDomain=false` | MHI-B-T07-e2e-1 |
| PB-FX-10 `config/e2e_validated_for_domain.toml` | Phase-B config/1/current | PB-FX-09 plus validation requirements `b-validation-calibration`, `b-validation-model`; roles bind PB-FX-03/PB-FX-04 as Validation; minimum independent families `2` | E2E-2 level `ValidatedForDomain`; two independent validation families | MHI-B-T07-e2e-2, MHI-B-T06-validation-pass |
| PB-FX-11 `config/duplicate_role_tuple.toml` | Phase-B config/1/current | two rows with `(b-hypothesis,b-eis-tau,eis.parameter.0)` | `DuplicateMechanismEvidenceRoleBinding` | MHI-B-T06-duplicate-role |
| PB-FX-12 `config/unbound_validation.toml` | Phase-B config/1/current | PB-FX-10 with the model Validation row absent | exactly `ValidationStatus::NotAssessed`; level remains `ExperimentallySupported` | MHI-B-T06-unbound-validation |
| PB-FX-13 `config/invalid_identifiability_input.toml` | Phase-B config/1/current | `ModeSeparation` with `AllEligible` instead of `ExactPair` | `InvalidIdentifiabilityInputSelection` | MHI-B-T05-invalid-selection |

The `Known` lineage records for PB-FX-01--04 have distinct artifact IDs
`b-artifact-eis`, `b-artifact-transient`, `b-artifact-calibration`, and
`b-artifact-model` and exactly the family strings shown above.  Their lineage
catalog has no shared ancestor.  This is the exact independence proof needed
by E2E-2; no fixture claims independence from path names.

The complete Phase-B fields in the two positive config fixtures are literal:

| config | required binding fields | pair/identifiability fields | validation fields |
|---|---|---|---|
| PB-FX-09 | `b-eis-tau`: target `ModelComponent("b-eis-tau")`, `ModelDerived`, `$.parameters[0].value`, `TimeConstant`, `s`, required; `b-transient-tau`: target `ModelComponent("tau_fast_s")`, `ModelDerived`, `$.events[0].candidate_fits[].derived_features.tau_fast_s`, `TimeConstant`, `s`, required | pair `b-timescale-pair=(b-eis-tau,b-transient-tau)`; `ModeSeparation` input `[b-eis-tau,b-transient-tau]`, `ExactPair{b-timescale-pair}`, threshold `1.0` | `required=false`, IDs `[]`, minimum `0` |
| PB-FX-10 | PB-FX-09 plus `b-validation-calibration`: target `ModelComponent("b-validation-calibration")`, `Observed`, `$.observations[0].potential_v`, `CalibrationPotential`, `V`; and `b-validation-model`: target `ModelComponent("b-validation-model")`, `ModelDerived`, `$.points[0].contributions[0].potential_v`, `ElectricalPotential`, `V` | same pair and identifiability fields as PB-FX-09 | `required=true`, IDs `[b-validation-calibration,b-validation-model]`, minimum `2` |

PB-FX-09's role rows are exactly `(b-hypothesis,b-eis-tau,eis.parameter.0,Support)`
and `(b-hypothesis,b-transient-tau,transient.event.0.tau_fast_s,Support)`.
PB-FX-10 contains those rows plus
`(b-hypothesis,b-validation-calibration,calibration.observation.0,Validation)`
and `(b-hypothesis,b-validation-model,model.point.0.component.0,Validation)`.

### 24.9 Normative source-artifact → EvidenceId map

Every E2E ID is in this table.  Directions below are the actual frozen A1
adapter result (`Neutral`), not a Phase-B interpretation.

| source fixture | source field | adapter | generated ID | target / direction / unit | temporal support | hypothesis requirement / role |
|---|---|---|---|---|---|---|
| `e2e/eis_fit_e2e_1.json` | `$.parameters[0].value=1.000` | `adapt_eis_fit` | `eis.parameter.0` | `ModelComponent("b-eis-tau")` / Neutral / `s` | Unknown | `b-eis-tau` / Support |
| `e2e/transient_analysis_e2e_1.json` | `$.events[0].candidate_fits[].derived_features.tau_fast_s=1.000` | `adapt_transient_analysis` | `transient.event.0.tau_fast_s` | `ModelComponent("tau_fast_s")` / Neutral / `s` | Window `[0,10]` TransientRelativeTime | `b-transient-tau` / Support |
| `e2e/calibration_observations_e2e_2.json` | `$.observations[0].potential_v=0.250` | `try_adapt_calibration_observations` | `calibration.observation.0` | `ModelComponent("b-validation-calibration")` / Neutral / `V` | Unknown | `b-validation-calibration` / Validation |
| `e2e/model_analysis_e2e_2.json` | `$.points[0].contributions[0].potential_v=0.250` | `adapt_model_analysis` | `model.point.0.component.0` | `ModelComponent("b-validation-model")` / Neutral / `V` | Point `5.0` ModelRelativeTime | `b-validation-model` / Validation |

Thus E2E-1 is legally producible from PB-FX-01 and PB-FX-02 through public
readers and both approved adapters.  E2E-2 is legally producible by adding
PB-FX-03 and PB-FX-04, whose known acquisition families are pairwise
independent under frozen A1 rules.

### 24.10 Fixture and traceability responsibilities

The plan is the normative fixture/data contract: §§24.8--24.9 define literal
paths, values, mappings, expected errors, and legal production construction.
`docs/engineering_specification/phase_b_mechanism_evidence_traceability.md`
is only the future implementation/result mapping.  It must reference the
fixture IDs above and must not become the first definition of fixture meaning.
No Phase-B test may serialize an `EvidenceBundle`, assign an `EvidenceId`,
temporal support, direction, strength, or role directly to an A1 record, or
use a file mtime/path as data.

### 24.11 PB-EX-05 — direct conflict and amplitude contradiction

A direct source-evidence conflict for a hypothesis requirement is exactly an
assembled A1 `EvidenceRecord` for which: (1) the record is eligible for the
same `EvidenceRequirementBinding`; (2) its temporal, scope, validity, and
quantity filters pass; and (3) `record.direction == EvidenceDirection::Contradicts`.
It is requirement-scoped.  It is not an undefined derived outcome.

The frozen current adapters in `src/evidence_adapters.rs` create Neutral
directions for all five accepted V1 source kinds.  Consequently no current
source-only V1 fixture can legally produce a direct contradiction, and there
is no separate conflicting-bundle input or artificial contradiction fixture.
The configured source flags remain the only route; each accepts one path and
the accepted kinds are exactly `eis_fit`, `transient_analysis`,
`calibration_observations`, `state_estimation`, and `ism_model_analysis`.

For a direct contradiction, persist its sorted ID in
`contradictory_evidence_ids`, increment the requirement's direct contradiction
count once per distinct ID, add `PhaseBHypothesisReasonCode::ConflictingEvidence`,
and exclude it from Support/Validation candidate satisfaction.  A non-critical
direct contradiction does not by itself demote a hypothesis: promotion follows
the existing required-binding/gate matrix using remaining eligible Support
records.  A critical direct contradiction blocks `ExperimentallySupported`
only when it is for a `critical_requirement_id`, has `direction=Contradicts`,
and `strength >= EvidenceStrength::Strong`; it then adds
`CriticalContradiction`.  The critical pipeline consumes no amplitude result.

The distinct amplitude outcome is closed and Phase-B-owned:

```rust
pub enum RequirementAssessmentStatus {
    Satisfied, NotSatisfied, NotAssessed, Contradicted,
}
```

An amplitude gate returns `Contradicted` only when both exact candidates are
valid and the observed-minus-predicted sign is opposite its `ExpectedEffect`.
It returns `NotSatisfied` when the required sign holds but the relative-error
threshold fails, and `NotAssessed` for missing/ambiguous/invalid candidates.
An amplitude `Contradicted` blocks the amplitude gate and is persisted with
`AmplitudeReasonCode::DirectionMismatch`; it is never copied into
`contradictory_evidence_ids`, never counted as a direct A1 contradiction, and
never feeds the critical-direct-contradiction pipeline.

### 24.12 Complete Phase-B production path and compatibility decision

The only integrated Phase-B path is:

```text
mechanism compare CLI source artifacts
→ public readers
→ A1 EvidenceBundle assembly
→ EvidenceId-stable temporal catalog construction
→ MechanismEvidenceConfig.hypotheses
→ requirement candidate binding
→ requirement-scoped role binding validation
→ direct critical-contradiction evaluation
→ timescale/amplitude/repeatability gates
→ identifiability with §24.7 inputs
→ validation-family assessment
→ promotion
→ optional history append
→ schema-4 MechanismAnalysisReport
```

Every stage has one owner.  Frozen A1 compatibility is **YES**: the serialized
meaning of `EvidenceRecord`, `EvidenceQuantity`, `EvidencePairKey`,
`ArtifactLineageState`, `TimescalePairUncertainty`, and A1 semantic identity
does not change.  Temporal metadata is the Phase-B-only additive mechanism
defined in §24.2, never an A1 change.

**Final remediation audit.**

```text
Undefined normative types: 0
Undefined normative owners: 0
Unspecified Phase B algorithms: 0
Unspecified scientific thresholds/units: 0
Unspecified compatibility decisions: 0
Normative contradictions: 0
Fixture-to-real-schema contradictions: 0
Implementation invention still required: no
```

Two competent implementers cannot differ about EvidenceId-to-temporal support,
role-binding scope, validation-role ownership, identifiability inputs,
multi-record selection, fixture contents, source-artifact-to-EvidenceId
mapping, direct conflict, or amplitude-versus-direct contradiction: **NO** for
each question.

## 25. Phase B Contract Remediation VII — final executable type, ownership, and E2E closure

### 25.1 Authority, scope, and current-code reconciliation

This is the **sole active normative Phase-B contract**. It supersedes every
Phase-B statement, type, fixture, algorithm, CLI table, migration statement,
and self-audit in §§6--10, 14--18, and 20--24. Those earlier statements are
historical/descriptive only unless this section expressly preserves them. A1
sections remain frozen. This is a documentation-only amendment: it neither
authorizes nor performs a production, test, fixture, `main`, or A1 change.

The five re-review findings are confirmed against the frozen tree:

| finding | classification | frozen-code evidence | §25 disposition |
|---|---|---|---|
| PB-RR-P1-01 temporal preparation owner/API | CONFIRMED | `src/runners/evidence.rs::assemble_evidence_bundle` returns only `EvidenceBundle`; `src/evidence.rs::EvidenceRecord` has no temporal member. | §§25.2--25.3 define one outer preparation owner, API, source-to-ID key, catalog, and join API. |
| PB-RR-P1-02 competing role declarations | CONFIRMED | §§20--24 contain both a role on an old `EvidenceRequirementBinding` and a role-binding type. | §25.4 retires the former and gives one tuple-keyed owner. |
| PB-RR-P1-03 identifiability applicability/reasons | CONFIRMED | frozen `src/model/identifiability.rs` serializes structural requirements only; it has no B assessor or B reason enum. | §25.5 defines the gate, status behavior, and one reason-code owner. |
| PB-RR-P1-04 conflict/amplitude persistence | CONFIRMED | §§20--24 use both `AmplitudeStatus` and an unattached requirement status. | §25.6 defines evaluator-owned statuses and persisted direct-contradiction summaries. |
| PB-RR-P1-05 fixtures/component output | CONFIRMED | earlier positive fixtures omit fields and incorrectly name model-analysis as an A1 assembled source; frozen `EvidenceBundleInputs` has no model-analysis field. | §§25.7--25.9 give the only legal component output and complete Phase-B fixture contracts using actual A1 inputs. |

The only current A1 source artifacts received by
`runners::evidence::EvidenceBundleInputs` are exactly
`StoredCalibrationModel` (lineage context only), `TransientAnalysisReport`,
`StateEstimationReport`, `EisFitArtifact`, and `CalibrationObservationSet`.
The current A1 assembly adapts the latter four evidence-producing types; it
does **not** adapt `ModelAnalysisReport`. Therefore model-analysis evidence is
not a Phase-B V1 input, is not a fixture source, and is not added by this
contract. This correction is additive to frozen A1: Phase B consumes the
existing assembly rather than changing it.

### 25.2 One preparation owner, input, result, and temporal association

`src/mechanism/preparation.rs` is the sole future owner of the outer
preparation layer. It owns no A1 type and never rereads a path. Its complete
public contract is:

```rust
pub struct PhaseBEvidencePreparationInputs {
    // The exact authoritative source objects passed to A1 assembly.
    pub evidence_inputs: EvidenceBundleInputs,
}

pub struct PhaseBEvidencePreparation {
    pub bundle: EvidenceBundle,
    pub temporal_metadata: EvidenceTemporalMetadataCatalog,
}

pub fn prepare_phase_b_evidence(
    inputs: PhaseBEvidencePreparationInputs,
) -> Result<PhaseBEvidencePreparation, PhaseBEvidencePreparationError>;
```

`EvidenceBundleInputs` is `Clone` in frozen A1. The function first derives
temporal stubs from `&inputs.evidence_inputs`, then moves the unchanged
`inputs.evidence_inputs` into `assemble_evidence_bundle`. Thus the temporal
source is the same authoritative object A1 assembled, not a duplicate input,
file path, cache, mtime, inferred experiment time, or independently reread
artifact. `calibration_model` has no adapter output and contributes no stub.

The one deterministic pre-assembly key and its post-assembly binding are:

```rust
pub struct TemporalEvidenceBindingKey {
    pub source_artifact_id: ArtifactId,
    pub adapter_id: String,
    pub adapter_output_id: String,
}
// adapter_output_id is the exact adapter-generated EvidenceId text:
// eis.parameter.{i}; transient.event.{i}.parameter.{j};
// transient.event.{i}.tau_fast_s; transient.event.{i}.tau_slow_s;
// estimation.point.{i}.state.{j}; calibration.observation.{i}.
```

For every adapted output, preparation derives exactly one
`(TemporalEvidenceBindingKey, EvidenceTemporalSupport)` while the source is
available. After A1 assembly, it binds a stub only when exactly one assembled
record has the same known source `ArtifactId`, adapter grammar, and
`EvidenceId == adapter_output_id`. Duplicate, absent, or mismatched matches
are `PhaseBEvidencePreparationError::TemporalBindingUnresolved`; positional
record order is forbidden. A legacy-unknown lineage cannot form this key and
returns `TemporalBindingLegacyUnknown`. These errors occur before hypothesis
evaluation. This makes source-to-`EvidenceId` association complete.

The required construction order is fixed:

```text
receive validated source artifacts
→ derive temporal stubs keyed by TemporalEvidenceBindingKey
→ assemble_evidence_bundle (unchanged A1) for stable EvidenceIds
→ bind each stub to its exact generated EvidenceId
→ construct EvidenceTemporalMetadataCatalog
→ validate catalog referential integrity
→ return PhaseBEvidencePreparation { bundle, temporal_metadata }
```

The temporal catalog is Phase-B-only and is never a member of `EvidenceRecord`
or `EvidenceBundle`:

```rust
pub struct EvidenceTemporalMetadataCatalog {
    pub entries: BTreeMap<EvidenceId, EvidenceTemporalMetadata>,
}
pub struct EvidenceTemporalMetadata {
    pub evidence_id: EvidenceId,
    pub support: EvidenceTemporalSupport,
    pub binding_key: TemporalEvidenceBindingKey,
}
pub enum EvidenceTemporalSupport {
    Point { timestamp_s: f64, clock: TemporalClockBasis },
    Window { start_s: f64, end_s: f64, clock: TemporalClockBasis },
    Event { event_id: String, start_s: f64, end_s: f64, clock: TemporalClockBasis },
    Aggregate,
    Unknown,
}
pub enum TemporalClockBasis { EstimationTimestamp, TransientRelativeTime }
pub enum PhaseBEvidencePreparationError {
    TemporalBindingLegacyUnknown,
    TemporalBindingUnresolved,
    DuplicateTemporalBinding,
    UnknownTemporalEvidenceId,
    TemporalCatalogKeyValueMismatch,
    InvalidTemporalSupport,
    EvidenceBundle(EvidenceBundleError),
}
```

The map key and `evidence_id` are bytewise equal; keys are unique, every key
exists in `bundle.records`, all used evidence has one entry, and every finite
point/window has `start_s < end_s`. Entries are serialized only inside the
schema-4 `MechanismAnalysisReport` B assessment payload, sorted by `EvidenceId`;
they have no A1 serialization or hash effect. The sole lawful mapping is:

| A1 adapter | output ID | support |
|---|---|---|
| `adapt_eis_fit` | `eis.parameter.{i}` | `Unknown`; EIS has no authoritative timestamp. |
| `adapt_transient_analysis` | all selected-fit parameter and tau outputs for event `{i}` | `Event { event_id: i.to_string(), start_s: fitted_time_local[0], end_s: fitted_time_local[last], clock: TransientRelativeTime }` only when all values are finite and strictly increasing; otherwise `Unknown`. |
| `adapt_state_estimation` | `estimation.point.{i}.state.{j}` | `Point { timestamp_s: estimates[i].timestamp_s, clock: EstimationTimestamp }` only when finite; otherwise `Unknown`. |
| `try_adapt_calibration_observations` | `calibration.observation.{i}` | `Unknown`; its timestamp has no declared Phase-B clock basis. |

### 25.3 One temporal configuration, assessment, and join algorithm

`src/mechanism/temporal.rs` owns every temporal evaluator type. `TemporalAssessmentPolicy`
is **RETIRED** and must not be parsed, serialized, inventoried as active, or
implemented. `MechanismEvidenceConfig.temporal` has exactly this type:

```rust
pub struct TemporalJoinConfig {
    pub point_tolerance_s: f64,
    pub minimum_classified_fraction: f64,
    pub minimum_equilibrium_fraction: f64,
    pub mixed_state_policy: MixedStatePolicy,
}
pub enum MixedStatePolicy { RequireAllSteady, MinimumSteadyFraction, WorstCase }
pub struct TemporalJoinAssessment {
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
    pub outcome: TemporalJoinOutcome,
    pub classified_fraction: Option<f64>,
    pub equilibrium_fraction: Option<f64>,
    pub reasons: Vec<TemporalJoinReasonCode>,
}
pub enum TemporalJoinOutcome { Eligible, Ineligible, NotAssessed }
pub enum TemporalJoinReasonCode {
    MissingMetadata, UnknownSupport, AggregateSupport, ClockMismatch,
    ScopeMismatch, PointToleranceExceeded, WindowNoPositiveOverlap,
    PointOutsideWindow, EventIdentityMismatch, ClassifiedFractionBelowMinimum,
    EquilibriumFractionBelowMinimum, MixedStateRejected,
}
pub enum TemporalJoinError { SameEvidenceId, UnknownEvidenceId, InvalidConfig }

pub fn evaluate_temporal_join(
    left: EvidenceId,
    right: EvidenceId,
    metadata: &EvidenceTemporalMetadataCatalog,
    config: &TemporalJoinConfig,
) -> Result<TemporalJoinAssessment, TemporalJoinError>;
```

All config numbers are finite; `point_tolerance_s >= 0` seconds and both
fractions are in `[0,1]`. The function may inspect only the supplied IDs,
catalog, and their A1 record scopes. Equal IDs are an error. Missing catalog
or `Unknown`/`Aggregate` support returns `NotAssessed`; clock/scope mismatch,
nonmatching event IDs, no positive half-open window overlap, or an out-of-window
point returns `Ineligible`; otherwise the relation is `Eligible`. Two points
are eligible iff equal clock and `abs(left-right) <= point_tolerance_s`; two
windows use `[max(start), min(end))`; point/window is eligible iff the point is
in `[start,end]`. Classification/equilibrium fractions are calculated only for
the selected records, using their producer-owned fields; if unavailable they
are `None` and add no reason. This V1 catalog has no aggregate producer, so
`Aggregate` is reserved but fully defined.

### 25.4 One role owner and no implicit role

`src/mechanism/config.rs` owns the serialized role schema. `EvidenceRequirementBinding.role`,
`EvidenceRoleBinding`, and every one-field role declaration are **RETIRED**.
`EvidenceRequirementBinding` owns selectors, expected direction, threshold,
pair semantics, and `RequirementGate`; it has no role field. The only role
owner is nested `MechanismHypothesisDefinition.role_bindings`:

```rust
pub type MechanismHypothesisId = HypothesisId; // frozen A1 ID alias
pub struct IdentifiabilityRequirementId(pub String);
pub struct MechanismEvidenceConfig {
    pub schema_version: u32,
    pub timescale: TimescaleEvidenceConfig,
    pub amplitude: AmplitudeEvidenceConfig,
    pub repeatability: RepeatabilityEvidenceConfig,
    pub temporal: TemporalJoinConfig,
    pub mixed_state: MixedStateConfig,
    pub identifiability: IdentifiabilityGateConfig,
    pub promotion: HypothesisPromotionConfig,
    pub validation: Option<ValidationProtocol>,
    pub hypotheses: Vec<MechanismHypothesisDefinition>,
}
pub struct TimescaleEvidenceConfig { pub algorithm: TimescaleAlgorithm }
pub struct AmplitudeEvidenceConfig { pub algorithm: AmplitudeAlgorithm }
pub struct RepeatabilityEvidenceConfig { pub algorithm: RepeatabilityAlgorithm }
pub struct MixedStateConfig { pub classification_source: ClassificationSource }
pub struct IdentifiabilityGateConfig { pub algorithm: IdentifiabilityAlgorithm }
pub struct HypothesisPromotionConfig { pub minimum_independent_support: usize }
pub enum TimescaleAlgorithm { LogRatioV1 }
pub enum AmplitudeAlgorithm { SignedRelativeErrorV1 }
pub enum RepeatabilityAlgorithm { IndependentLnTauSampleSdV1 }
pub enum ClassificationSource { ProducerOwnedOnly }
pub enum IdentifiabilityAlgorithm { BoundInputsV1 }
pub struct MechanismHypothesisDefinition {
    pub hypothesis_id: MechanismHypothesisId,
    pub display_name: String,
    pub target_components: Vec<ComponentId>,
    pub evidence_requirements: Vec<EvidenceRequirementBinding>,
    pub pair_requirements: Vec<EvidencePairRequirement>,
    pub critical_requirement_ids: Vec<EvidenceRequirementId>,
    pub timescale_gate: Option<TimescaleGate>,
    pub amplitude_gates: Vec<AmplitudeGate>,
    pub repeatability_gates: Vec<RepeatabilityGate>,
    pub identifiability_bindings: Vec<IdentifiabilityBinding>,
    pub validation_applicability: ValidationApplicability,
    pub role_bindings: Vec<MechanismEvidenceRoleBinding>,
}
pub struct EvidenceRequirementBinding {
    pub requirement_id: EvidenceRequirementId,
    pub target_selector: EvidenceTargetSelector,
    pub source_class_selectors: Vec<EvidenceSourceClass>,
    pub source_field_path: String,
    pub quantity_semantic: PhaseBQuantitySemantic,
    pub required_unit: String,
    pub expected_direction: RequiredEvidenceDirection,
    pub validity_requirement: EvidenceValidityRequirement,
    pub gate: RequirementGate,
}
pub enum EvidenceTargetSelector { ExactComponent(ComponentId) }
pub enum PhaseBQuantitySemantic { TimeConstant, ElectricalPotential, CalibrationPotential, ComponentScalar }
pub enum RequiredEvidenceDirection { CandidatePresence }
pub enum EvidenceValidityRequirement { Valid, ValidOrNotAssessed }
pub struct EvidencePairRequirement {
    pub requirement_id: EvidenceRequirementId,
    pub left_requirement_id: EvidenceRequirementId,
    pub right_requirement_id: EvidenceRequirementId,
    pub gate: RequirementGate,
}
pub struct TimescaleGate { pub pair_requirement_id: EvidenceRequirementId, pub maximum_log_distance: f64 }
pub struct AmplitudeThreshold { pub value: f64, pub unit: String }
pub struct AmplitudeGate {
    pub predicted_requirement_id: EvidenceRequirementId,
    pub observed_requirement_id: EvidenceRequirementId,
    pub expected_effect: ExpectedEffect,
    pub maximum_relative_error: f64,
    pub threshold: AmplitudeThreshold,
    pub gate: RequirementGate,
}
pub enum ExpectedEffect { Increase, Decrease, SameSign }
pub struct RepeatabilityGate {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub minimum_count: usize,
    pub maximum_log_tau_sample_standard_deviation: f64,
    pub gate: RequirementGate,
}
pub struct ValidationProtocol {
    pub protocol_id: String,
    pub version: String,
    pub minimum_acquisition_families: usize,
    pub required_conditions: Vec<ValidationCondition>,
}
pub struct ValidationCondition {
    pub condition_id: String,
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub experiment_scope: ExperimentId,
}
pub enum ValidationApplicability { NotApplicable, Required }
pub struct MechanismEvidenceRoleBinding {
    pub hypothesis_id: MechanismHypothesisId,
    pub requirement_id: EvidenceRequirementId,
    pub evidence_id: EvidenceId,
    pub role: MechanismEvidenceRole,
}
pub enum MechanismEvidenceRole { Support, Validation, Calibration, Training }
// semantic key: (hypothesis_id, requirement_id, evidence_id)
```

The vector is bytewise sorted by that complete key; a duplicate is a typed
configuration error. A nested row's hypothesis ID must equal its parent.
`Support` is eligible to satisfy its requirement; `Validation` is eligible for
validation counting only; `Calibration` and `Training` are not independent
support or validation unless the same record has an explicit separate Support
or Validation binding for a different requirement. Nothing infers a role from
artifact kind, adapter, direction, or requirement. Unbound evidence remains in
the bundle but cannot satisfy support or validation. Every positive E2E fixture
in §25.9 supplies a binding for every used ID.

All configuration IDs and selector paths are nonempty; declared ID collections
are sorted and duplicate-free; every pair and gate resolves within its owning
hypothesis; `schema_version == 1`; `minimum_independent_support >= 1`; and all
scientific thresholds are finite with the evaluator's stated nonnegative or
positive bound. An unknown field or invalid cross-reference is a typed
configuration error.

A `CandidatePresence` requirement accepts only a role-authorized A1 record
whose structural target, source class, source field path, quantity semantic,
and UCUM unit match its binding, whose availability is `Available`, whose value
is finite, and whose validity meets `validity_requirement`. `Valid` accepts
only `EvidenceValidity::Valid`; `ValidOrNotAssessed` accepts only `Valid` or
`NotAssessed`. It deliberately permits the current state-estimation adapter's
`NotAssessed` validity without inventing a validity claim. Both positive E2E
time-constant requirements and the calibration requirement use `Valid`; only
the estimation validation requirement uses `ValidOrNotAssessed`.

### 25.5 Identifiability applicability and one reason-code owner

`src/mechanism/identifiability.rs` owns the B assessor. Frozen
`src/model/identifiability.rs` remains a serializer of model requirements only.

```rust
pub enum RequirementGate { Required, NotApplicable }
pub struct IdentifiabilityBinding {
    pub requirement_id: IdentifiabilityRequirementId,
    pub gate: RequirementGate,
    pub kind: IdentifiabilityRequirementKind,
    pub threshold: f64,
    pub input: IdentifiabilityInputBinding,
}
pub struct IdentifiabilityInputBinding {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub selection: IdentifiabilityInputSelection,
}
pub enum IdentifiabilityInputSelection { ExactPair { pair_requirement_id: EvidenceRequirementId }, AllEligible }
pub enum IdentifiabilityAssessmentStatus { Satisfied, NotSatisfied, NotAssessed, NotApplicable }
pub enum IdentifiabilityAssessmentReasonCode {
    ThresholdSatisfied, ThresholdNotSatisfied, MissingInput,
    InsufficientEvidenceCount, UnsupportedMetricInput, NonFiniteInput,
    NotApplicableByDefinition,
}
pub struct IdentifiabilityAssessment {
    pub requirement_id: IdentifiabilityRequirementId,
    pub status: IdentifiabilityAssessmentStatus,
    pub metric_value: Option<f64>,
    pub evidence_ids: Vec<EvidenceId>,
    pub reasons: Vec<IdentifiabilityAssessmentReasonCode>,
}
```

For `Required`, the assessor always attempts evaluation and may return only
`Satisfied`, `NotSatisfied`, or `NotAssessed`: missing/unsupported/nonfinite
inputs produce `NotAssessed` with the declared reason code. For `NotApplicable`,
it performs no metric evaluation, returns only `NotApplicable`, and records
`NotApplicableByDefinition`; the configuration is the sole applicability
authority. Required `Satisfied` passes, `NotSatisfied` fails, and `NotAssessed`
blocks a promotion that requires it. `NotApplicable` skips that gate. V1 supports
only `ModeSeparation`: exactly one `ExactPair`, two finite positive `s` values,
metric `max(tau)/min(tau)`, and finite positive threshold; `>= threshold` is
Satisfied, below is NotSatisfied. Every other current model requirement kind is
Required → `NotAssessed(UnsupportedMetricInput)` or NotApplicable →
`NotApplicable(NotApplicableByDefinition)`. `UnsupportedMetricInput` therefore
belongs only to `IdentifiabilityAssessmentReasonCode`.

### 25.6 Amplitude, conflicts, reasons, and persistence

`src/mechanism/evaluation.rs` owns timescale, amplitude, repeatability, direct
contradiction evaluation, and their evaluator-specific statuses. There is no
generic `RequirementAssessmentStatus`; it is **RETIRED**.

```rust
pub enum TimescaleStatus { Satisfied, NotSatisfied, NotAssessed, NotApplicable }
pub struct TimescaleAssessment {
    pub pair_requirement_id: EvidenceRequirementId,
    pub status: TimescaleStatus,
    pub evidence_ids: Vec<EvidenceId>,
    pub log_distance: Option<f64>,
}
pub enum AmplitudeStatus { Satisfied, Contradicted, Inconclusive, NotAssessed, NotApplicable }
pub enum AmplitudeReasonCode {
    MissingCandidate, AmbiguousCandidate, InvalidUnit, UnitMismatch,
    DirectionMismatch, RelativeErrorExceeded,
}
pub struct AmplitudeAssessment {
    pub predicted_requirement_id: EvidenceRequirementId,
    pub observed_requirement_id: EvidenceRequirementId,
    pub status: AmplitudeStatus,
    pub predicted_evidence_id: Option<EvidenceId>,
    pub observed_evidence_id: Option<EvidenceId>,
    pub threshold: AmplitudeThreshold,
    pub relative_error: Option<f64>,
    pub reasons: Vec<AmplitudeReasonCode>,
}
pub enum RepeatabilityStatus { Satisfied, NotSatisfied, NotAssessed, NotApplicable }
pub struct RepeatabilityAssessment {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub status: RepeatabilityStatus,
    pub evidence_ids: Vec<EvidenceId>,
    pub sample_standard_deviation_ln_tau: Option<f64>,
}
pub struct RequirementContradictionSummary {
    pub requirement_id: EvidenceRequirementId,
    pub evidence_ids: Vec<EvidenceId>,
    pub contradiction_count: usize,
    pub strong_critical_count: usize,
}
pub enum PhaseBHypothesisReasonCode {
    ConflictingEvidence, StrongCriticalContradiction, MissingRequiredEvidence,
    InvalidEvidence, TemporalIneligible, IdentifiabilityNotSatisfied,
    IdentifiabilityNotAssessed, TimescaleGateFailed, AmplitudeGateContradicted,
    AmplitudeGateInconclusive, RepeatabilityGateFailed,
    InsufficientIndependentSupport, ValidationNotAssessed, ValidationNotSatisfied,
}
```

The amplitude threshold is finite, positive, and unit-bearing. After exact
candidate selection and conversion into that threshold unit, an opposite
required sign beyond the threshold returns `Contradicted`; it adds
`AmplitudeGateContradicted`. A correct sign outside the allowed relative-error
limit returns `Inconclusive`; missing/ambiguous/invalid input is `NotAssessed`.
No amplitude gate is `NotApplicable` unless its configuration gate is
NotApplicable. Amplitude status never alters `EvidenceRecord.direction`.

A direct contradiction is an otherwise eligible record for that requirement
with `direction == EvidenceDirection::Contradicts`. For each requirement,
persist the bytewise sorted/deduplicated IDs. `contradiction_count` equals that
unique-ID length. `strong_critical_count` counts only `Strong` direct
contradictions for a CriticalEvidence requirement. Any noncritical direct
contradiction adds `ConflictingEvidence`; a strong critical one adds
`StrongCriticalContradiction` and blocks promotion. An amplitude contradiction
does not enter either count, evidence-ID vector, or critical pipeline.

### 25.7 Hypothesis and per-component output

`src/mechanism/promotion.rs` owns promotion and `src/results/mechanism.rs`
owns schema-4 persistence. The complete output is:

```rust
pub enum HypothesisEvidenceLevel {
    Unassessed, Hypothesized, ExperimentallySupported, ValidatedForDomain,
}
pub enum ValidationProtocolStatus { NotApplicable, NotAssessed, Satisfied, NotSatisfied }
pub struct PhaseBHypothesisHistory {
    pub hypothesis_id: MechanismHypothesisId,
    pub prior_level: HypothesisEvidenceLevel,
    pub new_level: HypothesisEvidenceLevel,
    pub assessment_index: u64,
    pub reason_codes: Vec<PhaseBHypothesisReasonCode>,
}
pub enum ComponentInterpretationReasonCode {
    HypothesisSupported, ExperimentallySupported, ValidationSatisfied,
    ValidationUnavailable, PromotionBlockedByContradiction, IdentifiabilityBlocked,
}
pub struct ComponentInterpretationAssessment {
    pub component_id: ComponentId,
    pub prior_status: InterpretationStatus,
    pub resulting_status: InterpretationStatus,
    pub supporting_hypothesis_id: MechanismHypothesisId,
    pub evidence_ids: Vec<EvidenceId>,
    pub reasons: Vec<ComponentInterpretationReasonCode>,
}
pub struct PhaseBHypothesisAssessment {
    pub hypothesis_id: MechanismHypothesisId,
    pub evidence_level: HypothesisEvidenceLevel,
    pub temporal_join_assessments: Vec<TemporalJoinAssessment>,
    pub timescale_assessments: Vec<TimescaleAssessment>,
    pub amplitude_assessments: Vec<AmplitudeAssessment>,
    pub repeatability_assessments: Vec<RepeatabilityAssessment>,
    pub identifiability_assessments: Vec<IdentifiabilityAssessment>,
    pub contradiction_summaries: Vec<RequirementContradictionSummary>,
    pub reason_codes: Vec<PhaseBHypothesisReasonCode>,
    pub component_assessments: Vec<ComponentInterpretationAssessment>,
    pub validation_status: ValidationProtocolStatus,
    pub history: Vec<PhaseBHypothesisHistory>,
}
```

`component_assessments` is the only component-level Phase-B output. It has
exactly one row for every distinct declared target component, sorted by
canonical `ComponentId`; extra or missing rows are typed validation errors.
Each row applies the approved matrix to its own prior status: evidence support
can advance `Hypothesized` to `ExperimentallySupported`; only a satisfied
applicable validation protocol advances it to `ValidatedForDomain`; failed
blocking gates leave it unchanged and record the applicable reason. A
hypothesis-level result never overrides that component rule. History appends
only when a prior Phase-B assessment is explicitly supplied to a later run;
the first run has `history=[]`.

### 25.8 Complete controlling-type inventory and supersession audit

| Type | Serialized? | Owner module | Owning parent | Schema | Purpose | Supersedes |
|---|---|---|---|---:|---|---|
| MechanismEvidenceConfig | yes | `mechanism/config.rs` | CLI config | 1 | B root | all prior B roots |
| MechanismHypothesisDefinition | yes | `mechanism/config.rs` | config | 1 | hypothesis definition | prior definition variants |
| EvidenceRequirementBinding | yes | `mechanism/config.rs` | hypothesis | 1 | selectors/gates/pairs, no role | role-bearing binding |
| EvidencePairRequirement | yes | `mechanism/config.rs` | hypothesis | 1 | exact pair selector | prior pair forms |
| MechanismEvidenceRoleBinding | yes | `mechanism/config.rs` | hypothesis | 1 | sole role owner | EvidenceRoleBinding |
| PhaseBEvidencePreparationInputs | no | `mechanism/preparation.rs` | CLI runner | n/a | authoritative inputs | open preparation input |
| PhaseBEvidencePreparation | no | `mechanism/preparation.rs` | preparation | n/a | bundle plus catalog | post-assembly owner |
| EvidenceTemporalMetadataCatalog | yes, B output only | `mechanism/preparation.rs` | preparation | 4 | ID-keyed temporal metadata | bundle temporal fields |
| EvidenceTemporalMetadata | yes, B output only | `mechanism/preparation.rs` | catalog | 4 | one metadata row | vector-only forms |
| EvidenceTemporalSupport | yes, B output only | `mechanism/preparation.rs` | metadata | 4 | time support | prior temporal support |
| TemporalJoinConfig | yes | `mechanism/temporal.rs` | config | 1 | temporal policy | TemporalAssessmentPolicy |
| TemporalJoinAssessment | yes, B output only | `mechanism/temporal.rs` | hypothesis assessment | 4 | join result | incomplete temporal result |
| TemporalJoinOutcome | yes, B output only | `mechanism/temporal.rs` | join assessment | 4 | join disposition | prior outcome forms |
| TemporalJoinReasonCode | yes, B output only | `mechanism/temporal.rs` | join assessment | 4 | machine reason | TemporalJoinReason |
| IdentifiabilityBinding | yes | `mechanism/config.rs` | hypothesis | 1 | metric plus gate | gate-less binding |
| IdentifiabilityRequirementId | yes | `mechanism/config.rs` | identifiability binding | 1 | B metric ID | untyped metric ID |
| IdentifiabilityInputBinding | yes | `mechanism/config.rs` | identifiability binding | 1 | exact inputs | implicit input selection |
| IdentifiabilityAssessment | yes, B output only | `mechanism/identifiability.rs` | hypothesis assessment | 4 | metric outcome | prior assessment forms |
| IdentifiabilityAssessmentReasonCode | yes, B output only | `mechanism/identifiability.rs` | assessment | 4 | reasons | free `UnsupportedMetricInput` |
| AmplitudeAssessment | yes, B output only | `mechanism/evaluation.rs` | hypothesis assessment | 4 | amplitude outcome | prior forms |
| AmplitudeStatus | yes, B output only | `mechanism/evaluation.rs` | amplitude assessment | 4 | amplitude disposition | old status |
| RepeatabilityAssessment | yes, B output only | `mechanism/evaluation.rs` | hypothesis assessment | 4 | repeatability outcome | prior forms |
| RequirementContradictionSummary | yes, B output only | `mechanism/evaluation.rs` | hypothesis assessment | 4 | direct conflicts | unpersisted counts |
| PhaseBHypothesisAssessment | yes | `results/mechanism.rs` | report | 4 | full B result | HypothesisAssessment B use |
| PhaseBHypothesisReasonCode | yes, B output only | `mechanism/promotion.rs` | hypothesis assessment | 4 | hypothesis reasons | prior reason enums |
| ComponentInterpretationAssessment | yes, B output only | `mechanism/promotion.rs` | hypothesis assessment | 4 | component result | old component output |
| ValidationProtocol | yes | `mechanism/config.rs` | config | 1 | domain validation | prior validation forms |

The remaining declared controlling and supporting types have this same single
ownership; this continuation makes the inventory exhaustive for §25:

| Type | Serialized? | Owner module | Owning parent | Schema | Purpose | Supersedes |
|---|---|---|---|---:|---|---|
| TemporalEvidenceBindingKey | yes, B output only | `mechanism/preparation.rs` | metadata | 4 | pre/post assembly association | index association |
| TemporalClockBasis | yes, B output only | `mechanism/preparation.rs` | support | 4 | clock identity | prior clock variants |
| PhaseBEvidencePreparationError | no | `mechanism/preparation.rs` | API | n/a | preparation failures | untyped preparation errors |
| TemporalJoinError | no | `mechanism/temporal.rs` | join API | n/a | join failures | untyped join errors |
| MixedStatePolicy | yes | `mechanism/temporal.rs` | temporal config | 1 | state policy | prior policy |
| RequirementGate | yes | `mechanism/config.rs` | requirement/gate | 1 | applicability | option applicability |
| EvidenceTargetSelector | yes | `mechanism/config.rs` | requirement | 1 | structural target | field-name matching |
| PhaseBQuantitySemantic | yes | `mechanism/config.rs` | requirement | 1 | quantity interpretation | A1 quantity_kind |
| RequiredEvidenceDirection | yes | `mechanism/config.rs` | requirement | 1 | candidate direction | inferred direction |
| EvidenceValidityRequirement | yes | `mechanism/config.rs` | requirement | 1 | explicit validity acceptance | inferred validity |
| TimescaleGate | yes | `mechanism/config.rs` | hypothesis | 1 | timescale threshold | prior gate forms |
| TimescaleEvidenceConfig | yes | `mechanism/config.rs` | config | 1 | algorithm identity | implicit algorithm |
| AmplitudeEvidenceConfig | yes | `mechanism/config.rs` | config | 1 | algorithm identity | implicit algorithm |
| RepeatabilityEvidenceConfig | yes | `mechanism/config.rs` | config | 1 | algorithm identity | implicit algorithm |
| MixedStateConfig | yes | `mechanism/config.rs` | config | 1 | classification source | inferred classifier |
| IdentifiabilityGateConfig | yes | `mechanism/config.rs` | config | 1 | algorithm identity | implicit algorithm |
| HypothesisPromotionConfig | yes | `mechanism/config.rs` | config | 1 | support floor | implicit promotion floor |
| TimescaleAlgorithm | yes | `mechanism/config.rs` | timescale config | 1 | closed algorithm tag | free string |
| AmplitudeAlgorithm | yes | `mechanism/config.rs` | amplitude config | 1 | closed algorithm tag | free string |
| RepeatabilityAlgorithm | yes | `mechanism/config.rs` | repeatability config | 1 | closed algorithm tag | free string |
| ClassificationSource | yes | `mechanism/config.rs` | mixed-state config | 1 | source authority | inferred classifier |
| IdentifiabilityAlgorithm | yes | `mechanism/config.rs` | identifiability config | 1 | closed algorithm tag | free string |
| AmplitudeThreshold | yes | `mechanism/config.rs` | amplitude gate | 1 | value and UCUM unit | bare floor |
| AmplitudeGate | yes | `mechanism/config.rs` | hypothesis | 1 | amplitude gate | prior gate forms |
| ExpectedEffect | yes | `mechanism/config.rs` | amplitude gate | 1 | expected sign | inferred sign |
| RepeatabilityGate | yes | `mechanism/config.rs` | hypothesis | 1 | repeatability gate | prior gate forms |
| ValidationCondition | yes | `mechanism/config.rs` | validation protocol | 1 | validation condition | prior condition forms |
| ValidationApplicability | yes | `mechanism/config.rs` | hypothesis | 1 | validation gate | boolean applicability |
| TimescaleAssessment | yes, B output only | `mechanism/evaluation.rs` | hypothesis assessment | 4 | timescale result | prior result forms |
| TimescaleStatus | yes, B output only | `mechanism/evaluation.rs` | timescale assessment | 4 | timescale disposition | generic status |
| AmplitudeReasonCode | yes, B output only | `mechanism/evaluation.rs` | amplitude assessment | 4 | amplitude reasons | free strings |
| RepeatabilityStatus | yes, B output only | `mechanism/evaluation.rs` | repeatability assessment | 4 | repeatability disposition | generic status |
| HypothesisEvidenceLevel | yes, B output only | `mechanism/promotion.rs` | hypothesis assessment | 4 | promotion level | old B level forms |
| ValidationProtocolStatus | yes, B output only | `mechanism/promotion.rs` | hypothesis assessment | 4 | validation disposition | prior validation status |
| PhaseBHypothesisHistory | yes, B output only | `results/mechanism.rs` | hypothesis assessment | 4 | history row | prior history form |
| ComponentInterpretationReasonCode | yes, B output only | `mechanism/promotion.rs` | component assessment | 4 | component reasons | free strings |

The complete-plan occurrence audit is: `TemporalAssessmentPolicy`, old
`EvidenceRequirementBinding.role`, `EvidenceRoleBinding`, and
`RequirementAssessmentStatus` are **SUPERSEDED** wherever they occur before
§25 and have zero active uses. `TemporalJoinConfig`, `TemporalJoinAssessment`,
`AmplitudeStatus`, `IdentifiabilityBinding`, `MechanismEvidenceRoleBinding`,
`PhaseBHypothesisReasonCode`, and `component_assessments` before §25 are
**SUPERSEDED**; only their §25 definition is **ACTIVE NORMATIVE**.
`UnsupportedMetricInput` before §25 is **SUPERSEDED**; its §25.5 enum member
is **ACTIVE NORMATIVE**. Historical explanations and migration prose are
**DESCRIPTIVE**; schema-3-to-4 read compatibility is **MIGRATION-ONLY**. No
other occurrence supplies an active definition.

### 25.9 PB-FX-09 and PB-FX-10 complete literal E2E definitions

Fixture root is `tests/fixtures/phase_b/`. All paths below are exact future
fixture paths. All four sources are generated through their production types,
read through public artifact readers, have `Known` lineage with no shared
ancestor, and use the exact artifact/family pairs shown. These are the complete
Phase-B-consumed source fields; no test may inject an `EvidenceBundle`, ID,
temporal support, direction, strength, or role.

| ID/path | artifact/family | exact consumed payload → generated EvidenceId | temporal |
|---|---|---|---|
| PB-FX-01 `e2e/eis_fit_e2e_1.json` | `eis_fit`, `b-artifact-eis` / `b-family-eis` | `parameters[0]={element_id:"b-eis-tau", value:1.0, unit:"s"}` → `eis.parameter.0` | Unknown |
| PB-FX-02 `e2e/transient_analysis_e2e_1.json` | `transient_analysis`, `b-artifact-transient` / `b-family-transient` | selected event `0`, successful fit, `fitted_time_local=[0.0,10.0]`, `tau_fast_s=1.0` → `transient.event.0.tau_fast_s` | Event `"0"`, `[0.0,10.0]`, TransientRelativeTime |
| PB-FX-03 `e2e/calibration_observations_e2e_2.json` | `calibration_observations`, `b-artifact-calibration` / `b-family-calibration` | `observations[0]={experiment_id:"b-e2e-1", analyte:"b-validation-calibration", potential_v:0.25}` → `calibration.observation.0` | Unknown |
| PB-FX-04 `e2e/state_estimation_e2e_2.json` | `state_estimation`, `b-artifact-estimation` / `b-family-estimation` | `estimates[0]={timestamp_s:5.0, filtered_state:[{name:"b-validation-estimation",value:0.25,unit:"V"}]}` → `estimation.point.0.state.0` | Point `5.0`, EstimationTimestamp |

PB-FX-09 is `config/e2e_experimentally_supported.toml`; its complete B config
is the following literal (an absent `[validation]` table means validation is
not applicable):

```toml
schema_version = 1
[timescale]
algorithm = "log_ratio_v1"
[amplitude]
algorithm = "signed_relative_error_v1"
[repeatability]
algorithm = "independent_ln_tau_sample_sd_v1"
[temporal]
point_tolerance_s = 0.0
minimum_classified_fraction = 0.0
minimum_equilibrium_fraction = 0.0
mixed_state_policy = "require_all_steady"
[mixed_state]
classification_source = "producer_owned_only"
[identifiability]
algorithm = "bound_inputs_v1"
[promotion]
minimum_independent_support = 2

[[hypotheses]]
hypothesis_id = "b-hypothesis"
display_name = "B E2E support"
target_components = ["b-eis-tau", "tau_fast_s"]
critical_requirement_ids = []
validation_applicability = "not_applicable"
[[hypotheses.evidence_requirements]]
requirement_id = "b-eis-tau"
target_component_id = "b-eis-tau"
source_classes = ["model_derived"]
source_field_path = "$.parameters[0].value"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
[[hypotheses.evidence_requirements]]
requirement_id = "b-transient-tau"
target_component_id = "tau_fast_s"
source_classes = ["model_derived"]
source_field_path = "$.events[0].candidate_fits[].derived_features.tau_fast_s"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
[[hypotheses.pair_requirements]]
requirement_id = "b-timescale-pair"
left_requirement_id = "b-eis-tau"
right_requirement_id = "b-transient-tau"
gate = "required"
[hypotheses.timescale_gate]
pair_requirement_id = "b-timescale-pair"
maximum_log_distance = 0.0
[[hypotheses.identifiability_bindings]]
requirement_id = "b-mode-separation"
gate = "required"
kind = "mode_separation"
threshold = 1.0
input_requirement_ids = ["b-eis-tau", "b-transient-tau"]
input_selection = "exact_pair"
pair_requirement_id = "b-timescale-pair"
[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-eis-tau"
evidence_id = "eis.parameter.0"
role = "support"
[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-transient-tau"
evidence_id = "transient.event.0.tau_fast_s"
role = "support"
```

PB-FX-09 invokes the CLI with PB-FX-01 and PB-FX-02. The expected exact B
output is: `evidence_level=ExperimentallySupported`; one timescale assessment
for `b-timescale-pair` is Satisfied; temporal join, amplitude, and repeatability
assessment lists are empty; identifiability is `[b-mode-separation:Satisfied,
metric_value=1.0,reasons=[ThresholdSatisfied]]`; contradiction summaries and
hypothesis reasons are empty; validation is NotApplicable; component rows are
`b-eis-tau` and `tau_fast_s`, each prior `Hypothesized`, resulting
`ExperimentallySupported`, supporting hypothesis `b-hypothesis`, evidence IDs
`[eis.parameter.0,transient.event.0.tau_fast_s]`, reasons
`[HypothesisSupported,ExperimentallySupported]`; history is empty.

PB-FX-10 is `config/e2e_validated_for_domain.toml`; its complete B config is
the following literal:

```toml
schema_version = 1
[timescale]
algorithm = "log_ratio_v1"
[amplitude]
algorithm = "signed_relative_error_v1"
[repeatability]
algorithm = "independent_ln_tau_sample_sd_v1"
[temporal]
point_tolerance_s = 0.0
minimum_classified_fraction = 0.0
minimum_equilibrium_fraction = 0.0
mixed_state_policy = "require_all_steady"
[mixed_state]
classification_source = "producer_owned_only"
[identifiability]
algorithm = "bound_inputs_v1"
[promotion]
minimum_independent_support = 2

[[hypotheses]]
hypothesis_id = "b-hypothesis"
display_name = "B E2E validation"
target_components = ["b-eis-tau", "b-validation-calibration", "b-validation-estimation", "tau_fast_s"]
critical_requirement_ids = []
validation_applicability = "required"
[[hypotheses.evidence_requirements]]
requirement_id = "b-eis-tau"
target_component_id = "b-eis-tau"
source_classes = ["model_derived"]
source_field_path = "$.parameters[0].value"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
[[hypotheses.evidence_requirements]]
requirement_id = "b-transient-tau"
target_component_id = "tau_fast_s"
source_classes = ["model_derived"]
source_field_path = "$.events[0].candidate_fits[].derived_features.tau_fast_s"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
[[hypotheses.evidence_requirements]]
requirement_id = "b-validation-calibration"
target_component_id = "b-validation-calibration"
source_classes = ["observed"]
source_field_path = "$.observations[0].potential_v"
quantity_semantic = "calibration_potential"
required_unit = "V"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
[[hypotheses.evidence_requirements]]
requirement_id = "b-validation-estimation"
target_component_id = "b-validation-estimation"
source_classes = ["model_derived"]
source_field_path = "$.estimates[0].filtered_state[0].value"
quantity_semantic = "electrical_potential"
required_unit = "V"
expected_direction = "candidate_presence"
validity_requirement = "valid_or_not_assessed"
gate = "required"
[[hypotheses.pair_requirements]]
requirement_id = "b-timescale-pair"
left_requirement_id = "b-eis-tau"
right_requirement_id = "b-transient-tau"
gate = "required"
[hypotheses.timescale_gate]
pair_requirement_id = "b-timescale-pair"
maximum_log_distance = 0.0
[[hypotheses.identifiability_bindings]]
requirement_id = "b-mode-separation"
gate = "required"
kind = "mode_separation"
threshold = 1.0
input_requirement_ids = ["b-eis-tau", "b-transient-tau"]
input_selection = "exact_pair"
pair_requirement_id = "b-timescale-pair"
[validation]
protocol_id = "b-e2e-validation"
version = "1"
minimum_acquisition_families = 2
[[validation.required_conditions]]
condition_id = "b-calibration-condition"
requirement_ids = ["b-validation-calibration"]
experiment_scope = "b-e2e-1"
[[validation.required_conditions]]
condition_id = "b-estimation-condition"
requirement_ids = ["b-validation-estimation"]
experiment_scope = "b-e2e-1"
[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-eis-tau"
evidence_id = "eis.parameter.0"
role = "support"
[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-transient-tau"
evidence_id = "transient.event.0.tau_fast_s"
role = "support"
[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-validation-calibration"
evidence_id = "calibration.observation.0"
role = "validation"
[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-validation-estimation"
evidence_id = "estimation.point.0.state.0"
role = "validation"
```

PB-FX-10 invokes the CLI with PB-FX-01 through PB-FX-04. Its exact validation
IDs are `calibration.observation.0` and `estimation.point.0.state.0`; their
families are respectively `b-family-calibration` and `b-family-estimation`,
they are pairwise Independent under A1, neither is a support/training/
calibration-role candidate used for experimental support, and the exact
minimum-family calculation is `2 eligible distinct known families >= 2`.
The expected exact B output is: `evidence_level=ValidatedForDomain`; one
timescale assessment for `b-timescale-pair` is Satisfied; temporal join,
amplitude, and repeatability assessment lists are empty; identifiability is
`[b-mode-separation:Satisfied,metric_value=1.0,reasons=[ThresholdSatisfied]]`;
contradiction summaries and hypothesis reasons are empty; validation is
Satisfied; history is empty. The four component rows, in bytewise ID order,
are `b-eis-tau`, `b-validation-calibration`, `b-validation-estimation`, and
`tau_fast_s`. Every row has prior `Hypothesized`, resulting
`ValidatedForDomain`, supporting hypothesis `b-hypothesis`, evidence IDs
`[calibration.observation.0,eis.parameter.0,estimation.point.0.state.0,transient.event.0.tau_fast_s]`,
and reasons `[HypothesisSupported,ExperimentallySupported,ValidationSatisfied]`.

### 25.10 Final route, compatibility, and self-audit

The complete production route is:

```text
PhaseBEvidencePreparationInputs → prepare_phase_b_evidence
→ PhaseBEvidencePreparation { bundle, temporal_metadata }
→ MechanismEvidenceConfig.hypotheses → EvidenceRequirementBinding
→ MechanismEvidenceRoleBinding → temporal join → direct contradiction
→ timescale → amplitude → repeatability → IdentifiabilityBinding → validation
→ hypothesis promotion → component assessments → history
→ MechanismAnalysisReport schema 4
```

Phase B can be implemented without changing serialized meaning of
`EvidenceRecord`, `EvidenceQuantity`, `EvidencePairKey`, `ArtifactLineageState`,
`TimescalePairUncertainty`, or A1 semantic identity: **YES**. Previously passed
hypothesis ownership, quantity semantics, direct `EvidenceBundle` retirement,
EIS `Unknown` time, unit-bearing amplitude threshold, repeatability algorithm,
schema-3-to-4 migration, validation roles, traceability, critical conflict
pipeline, TOML root, CLI, legacy bundle compatibility, and history remain PASS
as constrained by this amendment.

```text
Undefined normative types: 0
Undefined normative owners: 0
Unspecified Phase B algorithms: 0
Unspecified scientific thresholds/units: 0
Unspecified compatibility decisions: 0
Normative contradictions: 0
Fixture-to-real-schema contradictions: 0
Incomplete normative positive fixtures: 0
Implementation invention still required: no
```

Two competent implementers can differ about the preparation object, temporal
types/API, timestamp retention, role precedence/ownership, applicability,
`UnsupportedMetricInput` ownership, amplitude contradiction, direct-conflict
persistence, hypothesis reasons, component output, PB-FX-09, or PB-FX-10:
**NO** for every item.

## 26. Phase B Contract Remediation VIII — final temporal, gate, promotion, wire-format, and API closure

### 26.1 Authority and independent reconciliation

This section is the sole active normative Phase-B contract. It supersedes the
active portions of §§6--10, 14--18, and 20--25 wherever they differ. Earlier
material remains historical/descriptive or migration-only. A1 remains frozen;
this section is documentation-only and authorizes no production Rust, test,
fixture, `main`, or A1 change.

| finding | classification | current evidence | controlling correction |
|---|---|---|---|
| PB-FR-P1-01 temporal evaluator executability | CONFIRMED | `EvidenceBundle` owns records/scopes/lineage, but the §25 evaluator receives only metadata and no classification fields | §26.2 adds one metadata row per `EvidenceId`, passes `&EvidenceBundle`, and defines the full support matrix |
| PB-FR-P1-02 role/gate precedence | CONFIRMED | §25 has a gate but no requirement stage and no complete role/phase rule | §26.3 adds `EvidenceRequirementStage`, keeps gate separate, and defines role compatibility |
| PB-FR-P1-03 amplitude/conflict mathematics | CONFIRMED | §23.5 has the approved error equation, but active §25 omits signed-relative-error symbols and still uses `CriticalEvidence` | §26.4 activates the approved equation and uses `critical_requirement_ids` only |
| PB-FR-P1-04 complete component promotion | CONFIRMED | §25 has partial upward prose but no complete status matrix or current target field | §26.5 defines monotonic V1 behavior, a 4×4 matrix, and `assessment_target` |
| PB-FR-P1-05 fixture wire/schema correctness | CONFIRMED | §25.9 has invalid ArtifactId placeholders and fields not present on the declared structs | §26.6 makes IDs runtime-derived and replaces fixtures with actual struct-shaped wire |
| PB-FR-P1-06 exhaustive types and APIs | CONFIRMED | §25.8 omits active IDs/statuses/errors and post-preparation production functions | §§26.7--26.9 add the exhaustive inventory and exact stage APIs |

### 26.2 Executable temporal contract

Phase B owns exactly one temporal metadata row per assembled `EvidenceId`. It
is additive and never changes the serialized meaning or semantic identity of
A1 `EvidenceRecord`, `EvidenceQuantity`, `EvidencePairKey`,
`ArtifactLineageState`, or `TimescalePairUncertainty`.

```rust
pub struct ClockId(pub String);

pub struct TemporalClassificationMetadata {
    pub classified_fraction: Option<f64>,
    pub equilibrium_fraction: Option<f64>,
    pub steady_state_fraction: Option<f64>,
    pub classification_source: TemporalClassificationSource,
}
pub enum TemporalClassificationSource {
    StateEstimationEquilibriumAssessment,
    ModelAnalysisEquilibriumAssessment,
    Unavailable,
}
pub enum EvidenceTemporalSupport {
    Point { timestamp_s: f64 },
    Window { start_s: f64, end_s: f64 },
    Event { event_id: String, start_s: f64, end_s: f64 },
    Unknown,
}
pub struct TemporalSupportProvenance {
    pub adapter_id: String,
    pub source_artifact_kind: ArtifactKind,
    pub source_field_paths: Vec<String>,
}
pub struct EvidenceTemporalMetadata {
    pub evidence_id: EvidenceId,
    pub support: EvidenceTemporalSupport,
    pub clock_id: Option<ClockId>,
    pub classification: TemporalClassificationMetadata,
    pub provenance: TemporalSupportProvenance,
}
pub struct EvidenceTemporalMetadataCatalog {
    pub entries: BTreeMap<EvidenceId, EvidenceTemporalMetadata>,
}

pub struct TemporalJoinConfig {
    pub point_tolerance_s: f64,
    pub minimum_classified_fraction: f64,
    pub minimum_equilibrium_fraction: f64,
    pub mixed_state_policy: MixedStatePolicy,
}
pub enum MixedStatePolicy {
    RequireAllSteady { allow_quasi_equilibrium: bool },
    MinimumSteadyFraction {
        minimum_fraction: f64,
        allow_quasi_equilibrium: bool,
        reject_if_disturbed: bool,
    },
    WorstCase,
}
pub struct TemporalJoinAssessment {
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
    pub join_mode: TemporalJoinMode,
    pub outcome: TemporalJoinOutcome,
    pub classified_fraction: Option<f64>,
    pub equilibrium_fraction: Option<f64>,
    pub steady_state_fraction: Option<f64>,
    pub reasons: Vec<TemporalJoinReasonCode>,
}
pub enum TemporalJoinOutcome { Eligible, Ineligible, Indeterminate }
pub enum TemporalJoinReasonCode {
    MissingMetadata,
    UnknownSupport,
    ClockMismatch,
    ClockUnknown,
    ScopeMismatch,
    ScopeAmbiguous,
    PointToleranceExceeded,
    WindowNoPositiveOverlap,
    PointOutsideWindow,
    EventIdentityMismatch,
    ClassificationUnavailable,
    ClassifiedFractionBelowMinimum,
    EquilibriumFractionBelowMinimum,
    UnsupportedTemporalSupportCombination,
}
pub enum TemporalJoinError {
    SameEvidenceId,
    UnknownEvidenceId,
    MissingMetadata,
    InvalidConfig,
}
```

Fractions are finite and in `[0,1]`. `Unavailable` requires all three
fractions to be `None`; no source without a producer classification receives a
fabricated value. A single producer point with
`EquilibriumAssessment::classification` is represented as one classified
observation: `classified_fraction=1.0`, `equilibrium_fraction=1.0` only for
`Equilibrium`, and `steady_state_fraction=1.0` for `Equilibrium` or permitted
`QuasiEquilibrium`; otherwise the applicable fraction is `0.0`. This is a
count-preserving projection of a real producer classification, not an inferred
one.

For a producer-owned sequence selected by a window, the exact approved count
algorithm is: `N_target` is the number of expected target observations;
`N_classified` is the number with an Available producer classification;
`classified_fraction=N_classified/N_target`; `N_equilibrium` counts
`Equilibrium`; `N_quasi` counts `QuasiEquilibrium`;
`equilibrium_fraction=N_equilibrium/N_classified`; and
`steady_state_fraction=(N_equilibrium+N_quasi)/N_classified`. If `N_target=0`
the result is missing; if `N_classified=0`, all fractions are unavailable, not
zero. `RequireAllSteady` requires every classified state to be Equilibrium or,
when configured, QuasiEquilibrium. `MinimumSteadyFraction` applies its
inclusive `minimum_fraction` to the allowed steady numerator and optionally
rejects any Disturbed state. `WorstCase` uses the fixed precedence
`Indeterminate > Disturbed > Transitional > QuasiEquilibrium > Equilibrium`.

| adapter/source | generated ID and real source field | support/clock | classification source | unavailable behavior |
|---|---|---|---|---|
| `adapt_eis_fit` | `eis.parameter.{i}` from `EisFitArtifact.parameters[i]` | `Unknown`; no timestamp or clock field | none | both `None`, `Unavailable`; required gate is `Indeterminate` |
| `adapt_transient_analysis` | selected event parameter/tau from `events[i].segment.fitted_time_local` and `candidate_fits` | `Event { event_id=i.to_string(), start_s=first, end_s=last }` only for finite strict-increasing real values; no current clock ID | none | `Unknown`, both `None`, `Unavailable` |
| `adapt_state_estimation` | `estimation.point.{i}.state.{j}` from `estimates[i].timestamp_s` and `filtered_state[j]` | `Point { timestamp_s }` when finite; no current serialized clock ID | `estimates[i].equilibrium_assessment.classification` | absent classification → both `None`; invalid time → `Unknown` |
| `try_adapt_calibration_observations` | `calibration.observation.{i}` from complete `CalibrationObservation` | `Unknown`; `timestamp` has no declared comparable clock | no `EquilibriumStatus` field | both `None`, `Unavailable` |
| model analysis | not in current `EvidenceBundleInputs` | no Phase B V1 source | not applicable | never fabricated or fixture-injected |

The current production input set is unchanged: transient, state estimation,
EIS, and calibration observations produce records; stored calibration model is
lineage-only. Preparation derives stubs from the same `EvidenceBundleInputs`
that it moves into A1 `assemble_evidence_bundle`, binds by known source
`ArtifactId`, adapter ID, and exact generated `EvidenceId`, and rejects
positional or path-based matching.

```rust
pub struct PhaseBEvidencePreparationInputs { pub evidence_inputs: EvidenceBundleInputs }
pub struct PhaseBEvidencePreparation {
    pub bundle: EvidenceBundle,
    pub temporal_metadata: EvidenceTemporalMetadataCatalog,
}
pub fn prepare_phase_b_evidence(
    inputs: PhaseBEvidencePreparationInputs,
) -> Result<PhaseBEvidencePreparation, PhaseBEvidencePreparationError>;
```

The sole temporal requirement owner is the pair:

```rust
pub enum TemporalRequirement {
    NotApplicable,
    Required {
        counterpart_requirement_id: EvidenceRequirementId,
        join_mode: TemporalJoinMode,
    },
}
pub enum TemporalJoinMode { PointPoint, PointWindow, WindowPoint, WindowWindow, EventEvent }
pub struct EvidencePairRequirement {
    pub requirement_id: EvidenceRequirementId,
    pub left_requirement_id: EvidenceRequirementId,
    pub right_requirement_id: EvidenceRequirementId,
    pub temporal: TemporalRequirement,
    pub gate: RequirementGate,
}
```

There is no global join-every-pair behavior. The controlling evaluator is:

```rust
pub fn evaluate_temporal_join(
    left: EvidenceId,
    right: EvidenceId,
    bundle: &EvidenceBundle,
    temporal_metadata: &EvidenceTemporalMetadataCatalog,
    config: &TemporalJoinConfig,
) -> Result<TemporalJoinAssessment, TemporalJoinError>;
```

Scope comes from the two bundle records. Sensor/channel scope comes from the
known source ArtifactIds through `bundle.lineage_catalog`, falling back to
concrete bundle-level scope only when available. `Specific` must equal
`Specific`; `Specific`/`All` and `All`/`All` are compatible; `Unspecified` needs
the same concrete resolved scope. Only identical `Single { experiment_id }`
experiment scopes are compatible. Aggregate or Unknown scope is
`Indeterminate`, never narrowed by membership.

If both clock IDs are known and unequal, return `Indeterminate(ClockMismatch)`.
If either is unknown, return `Indeterminate(ClockUnknown)`; V1 never compares
numeric times without a shared clock. Scope mismatch is
`Indeterminate(ScopeMismatch)`.

| left | right | mode | eligible rule | ineligible boundary |
|---|---|---|---|---|
| Point | Point | `PointPoint` | same scope/clock and `abs(left-right) <= point_tolerance_s` | `>` → `PointToleranceExceeded` |
| Point | Window | `PointWindow` | point in inclusive `[start,end]` | outside → `PointOutsideWindow` |
| Window | Point | `WindowPoint` | point in inclusive `[start,end]` | outside → `PointOutsideWindow` |
| Window | Window | `WindowWindow` | `max(start) < min(end)` for `[start,end)` | zero/negative → `WindowNoPositiveOverlap` |
| Event | Event | `EventEvent` | same scope/clock and exact `event_id` equality | unequal → `EventIdentityMismatch` |
| Event | Point | unsupported | none | `Indeterminate(UnsupportedTemporalSupportCombination)` |
| Event | Window | unsupported | none | `Indeterminate(UnsupportedTemporalSupportCombination)` |
| Point | Event | unsupported | none | `Indeterminate(UnsupportedTemporalSupportCombination)` |
| Window | Event | unsupported | none | `Indeterminate(UnsupportedTemporalSupportCombination)` |

`TemporalJoinOutcome` is exactly `Eligible`, `Ineligible`, or `Indeterminate`.
Unknown support, unsupported combinations, unknown clocks, ambiguous scope,
and unavailable classification required by a positive configured minimum are
always `Indeterminate`. Fractions are the minimum of the two present fractions;
configured minimum comparisons are inclusive. `TemporalJoinAssessment` stores
both IDs, mode, outcome, optional fractions, and sorted reason codes.

`MixedStatePolicy` is internally tagged with `kind`, not externally tagged:
`require_all_steady` has only `allow_quasi_equilibrium`,
`minimum_steady_fraction` has `minimum_fraction`,
`allow_quasi_equilibrium`, and `reject_if_disturbed`, and `worst_case` has no
payload. The canonical TOML is therefore
`[temporal.mixed_state_policy]`, `kind="require_all_steady"`, and
`allow_quasi_equilibrium=false` (or the exact fields of the selected variant).

### 26.3 Requirement stage, gate, and role precedence

```rust
pub enum EvidenceRequirementStage { Support, Validation, SupportAndValidation }
pub struct EvidenceRequirementBinding {
    pub requirement_id: EvidenceRequirementId,
    pub target_selector: EvidenceTargetSelector,
    pub source_class_selectors: Vec<EvidenceSourceClass>,
    pub source_field_path: String,
    pub quantity_semantic: PhaseBQuantitySemantic,
    pub required_unit: String,
    pub expected_direction: RequiredEvidenceDirection,
    pub validity_requirement: EvidenceValidityRequirement,
    pub gate: RequirementGate,
    pub stage: EvidenceRequirementStage,
}
```

`gate` means whether the requirement applies; `stage` means when it is
enforced. `NotApplicable` is not evaluated in any stage; its serialized stage
is ignored for schema stability. `Required + Support` is enforced for
ExperimentallySupported and higher. `Required + Validation` does not block
Hypothesized or ExperimentallySupported and is enforced only for
ValidatedForDomain. `Required + SupportAndValidation` is enforced in both.

| role | legal stage | support candidate? | validation candidate? |
|---|---|---:|---:|
| `Support` | Support, SupportAndValidation | yes | no |
| `Validation` | Validation, SupportAndValidation | no | yes |
| `Calibration` | none | no | no |
| `Training` | none | no | no |

The role owner remains `MechanismEvidenceRoleBinding`; an illegal role/stage
tuple is typed `RoleStageMismatch`. Gate/stage precedence is evaluated before
role candidate selection, and role is never inferred from artifact kind,
source class, direction, or field path. PB-FX-10 declares every validation-only
row literally as `gate="required"`, `stage="validation"`, and
`role="validation"`; therefore it cannot block ExperimentallySupported.

### 26.4 Active amplitude mathematics and critical definition

The exact previously approved §23.5 equation is active. After UCUM conversion
to the requirement threshold unit, let `p` be predicted and `o` observed:

```text
d       = o - p
D       = max(abs(p), abs(o), f)
r_signed = d / D
r_abs    = abs(r_signed) = abs(p - o) / D
```

Here `f` is the finite, positive, unit-bearing requirement threshold; `D` is
the normalization denominator. `r_abs` is the exact approved error; the signed
form makes its sign convention explicit without changing the science. The
order is: parse UCUM, convert, calculate `d/D`, then apply boundaries.

`Increase` requires `d > 0`, `Decrease` requires `d < 0`, and `SameSign`
requires `p*o > 0`; equality to zero fails the direction predicate. With two
valid candidates, a direction failure is `Contradicted`/`DirectionMismatch`; a
direction pass with `r_abs <= maximum_relative_error` is `Satisfied`; a pass
with `r_abs > maximum_relative_error` is `Inconclusive`/
`RelativeErrorExceeded`. Missing, ambiguous, invalid, or incompatible inputs
are `NotAssessed`. The relative-error boundary is inclusive only at `<=`.

No normative type or category named `CriticalEvidence` exists. A requirement
is critical iff and only if its ID is in
`MechanismHypothesisDefinition.critical_requirement_ids`. A strong critical
contradiction is an eligible direct record for such an ID with
`direction=Contradicts` and `strength >= Strong`. Amplitude contradiction is
never part of that direct contradiction pipeline.

### 26.5 Complete component promotion and history contract

Phase B V1 never automatically demotes a component. A weaker current
assessment is persisted as evidence/reason history, while the component's
resulting status is the monotonic maximum of prior status and current target.
The current-run target is `NoPromotion` when blocked or insufficient,
`Hypothesized` when the hypothesis threshold is met,
`ExperimentallySupported` when support gates pass, and
`ValidatedForDomain` only when all validation requirements pass.

```rust
pub struct ComponentInterpretationAssessment {
    pub component_id: ComponentId,
    pub prior_status: InterpretationStatus,
    pub assessment_target: Option<InterpretationStatus>,
    pub resulting_status: InterpretationStatus,
    pub supporting_hypothesis_id: MechanismHypothesisId,
    pub evidence_ids: Vec<EvidenceId>,
    pub reasons: Vec<ComponentInterpretationReasonCode>,
}
```

`assessment_target=None` is exactly `NoPromotion`; otherwise it is the
current-run target. The complete frozen-status matrix is:

| prior status \ target | NoPromotion | Hypothesized | ExperimentallySupported | ValidatedForDomain |
|---|---|---|---|---|
| Phenomenological | Phenomenological | Hypothesized | ExperimentallySupported | ValidatedForDomain |
| Hypothesized | Hypothesized | Hypothesized | ExperimentallySupported | ValidatedForDomain |
| ExperimentallySupported | ExperimentallySupported | ExperimentallySupported | ExperimentallySupported | ValidatedForDomain |
| ValidatedForDomain | ValidatedForDomain | ValidatedForDomain | ValidatedForDomain | ValidatedForDomain |

Rows and columns are exhaustive. Every declared target component has one
sorted row; extra or missing rows are typed errors. History stores prior status,
target, resulting status, and uses the semantic duplicate key
`(hypothesis_id, component_id, prior_status, target, resulting_status,
sorted evidence_ids, sorted reason_codes)`.

### 26.6 Real wire types, ArtifactIds, and PB-FX-09/10

`ArtifactId` is frozen A1 `sha256:` plus exactly 64 lowercase hexadecimal
characters. Source-artifact fixtures generate identity through the current
production writer, reread it through the public reader, and bind the returned
`Known.identity.artifact_id` to a runtime variable such as `EIS_ARTIFACT_ID`.
No `b-artifact-*` value is serialized. Literal scalar fields are exact;
content-derived identity fields are generated and asserted after reread.

| fixture source | real Rust type and consumed fields | prohibited invented shape |
|---|---|---|
| PB-FX-01 | `EisFitArtifact.parameters[i].element_id`, `.unit`, `.value`, plus all producer-required fields | no placeholder identity or model-analysis fields |
| PB-FX-02 | `TransientAnalysisReport.events[i].segment.fitted_time_local`, selected `candidate_fits`, and `derived_features.tau_fast_s` | no nonexistent top-level `derived_features` |
| PB-FX-03 | complete `CalibrationObservationSet` and `CalibrationObservation` row | no reduced ad hoc observation object |
| PB-FX-04 | `StateEstimationReport.estimates[i].timestamp_s` and `.filtered_state[j].{name,value,unit}` | no `filtered_state` outside `estimates` |
| model analysis | not in current `EvidenceBundleInputs` | no Phase-B V1 source fixture |

The active configuration field names are exact: `target_selector`,
`source_class_selectors`, `input`, and `stage`. `MechanismEvidenceConfig`,
`MechanismHypothesisDefinition`, `EvidenceRequirementBinding`,
`EvidencePairRequirement`, `MechanismEvidenceRoleBinding`,
`IdentifiabilityBinding`, `IdentifiabilityInputBinding`, and
`ValidationProtocol` are all ordinary serde structs with the Rust field names
shown in §§25.4--25.5 and 26.3. `EvidenceRequirementStage`, `RequirementGate`,
roles, and algorithm tags use `#[serde(rename_all="snake_case")]` scalars.
`EvidenceTargetSelector`, `IdentifiabilityInputSelection`, and
`TemporalRequirement` use `#[serde(tag="type", rename_all="snake_case")]`.
No active enum is externally tagged or untagged.

The corrected PB-FX-09 requirement literal is structurally:

```toml
[temporal]
point_tolerance_s = 0.0
minimum_classified_fraction = 0.0
minimum_equilibrium_fraction = 0.0
[temporal.mixed_state_policy]
kind = "require_all_steady"
allow_quasi_equilibrium = false

[[hypotheses.evidence_requirements]]
requirement_id = "b-eis-tau"
target_selector = { type = "exact_component", value = "b-eis-tau" }
source_class_selectors = ["model_derived"]
source_field_path = "$.parameters[0].value"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
stage = "support"

[[hypotheses.evidence_requirements]]
requirement_id = "b-transient-tau"
target_selector = { type = "exact_component", value = "tau_fast_s" }
source_class_selectors = ["model_derived"]
source_field_path = "$.events[0].candidate_fits[].derived_features.tau_fast_s"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
stage = "support"

[[hypotheses.pair_requirements]]
requirement_id = "b-timescale-pair"
left_requirement_id = "b-eis-tau"
right_requirement_id = "b-transient-tau"
temporal = { type = "not_applicable" }
gate = "required"

[[hypotheses.identifiability_bindings]]
requirement_id = "b-mode-separation"
gate = "required"
kind = "mode_separation"
threshold = 1.0
input = { requirement_ids = ["b-eis-tau", "b-transient-tau"], selection = { type = "exact_pair", pair_requirement_id = "b-timescale-pair" } }
```

PB-FX-10 uses the same shapes and adds each validation-only requirement with
`gate="required"`, `stage="validation"`, and a matching
`MechanismEvidenceRoleBinding.role="validation"`. It does not use
`target_component_id`, `source_classes`, `input_requirement_ids`,
`input_selection`, or a flat `pair_requirement_id`. Both fixtures deserialize
with serde and declared attributes alone; no custom syntax-preserving logic is
permitted.

The PB-FX-10-only wire rows are:

```toml
[[hypotheses.evidence_requirements]]
requirement_id = "b-validation-calibration"
target_selector = { type = "exact_component", value = "b-validation-calibration" }
source_class_selectors = ["observed"]
source_field_path = "$.observations[0].potential_v"
quantity_semantic = "calibration_potential"
required_unit = "V"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
stage = "validation"

[[hypotheses.evidence_requirements]]
requirement_id = "b-validation-estimation"
target_selector = { type = "exact_component", value = "b-validation-estimation" }
source_class_selectors = ["model_derived"]
source_field_path = "$.estimates[0].filtered_state[0].value"
quantity_semantic = "electrical_potential"
required_unit = "V"
expected_direction = "candidate_presence"
validity_requirement = "valid_or_not_assessed"
gate = "required"
stage = "validation"

[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-validation-calibration"
evidence_id = "calibration.observation.0"
role = "validation"

[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-validation-estimation"
evidence_id = "estimation.point.0.state.0"
role = "validation"

[validation]
protocol_id = "b-e2e-validation"
version = "1"
minimum_acquisition_families = 2
[[validation.required_conditions]]
condition_id = "b-calibration-condition"
requirement_ids = ["b-validation-calibration"]
experiment_scope = "b-e2e-1"
[[validation.required_conditions]]
condition_id = "b-estimation-condition"
requirement_ids = ["b-validation-estimation"]
experiment_scope = "b-e2e-1"
```

The canonical wire table is:

| Rust type | serde field names | wire shape |
|---|---|---|
| `MechanismEvidenceConfig` | `schema_version`, `timescale`, `amplitude`, `repeatability`, `temporal`, `mixed_state`, `identifiability`, `promotion`, `validation`, `hypotheses` | root TOML struct; validation optional |
| `MechanismHypothesisDefinition` | `hypothesis_id`, `display_name`, `target_components`, `evidence_requirements`, `pair_requirements`, `critical_requirement_ids`, `timescale_gate`, `amplitude_gates`, `repeatability_gates`, `identifiability_bindings`, `validation_applicability`, `role_bindings` | `[[hypotheses]]` array-of-table |
| `EvidenceRequirementBinding` | `requirement_id`, `target_selector`, `source_class_selectors`, `source_field_path`, `quantity_semantic`, `required_unit`, `expected_direction`, `validity_requirement`, `gate`, `stage` | `[[hypotheses.evidence_requirements]]` |
| `EvidencePairRequirement` | `requirement_id`, `left_requirement_id`, `right_requirement_id`, `temporal`, `gate` | `[[hypotheses.pair_requirements]]` |
| `MechanismEvidenceRoleBinding` | `hypothesis_id`, `requirement_id`, `evidence_id`, `role` | `[[hypotheses.role_bindings]]` |
| `IdentifiabilityBinding` | `requirement_id`, `gate`, `kind`, `threshold`, `input` | `[[hypotheses.identifiability_bindings]]` |
| `IdentifiabilityInputBinding` | `requirement_ids`, `selection` | nested `input` table/inline table |
| `ValidationProtocol` | `protocol_id`, `version`, `minimum_acquisition_families`, `required_conditions` | optional `[validation]` plus child tables |

All eight rows use Rust names because no `serde(rename=...)` is declared.
Enums in these rows are snake_case scalar strings except the three internally
tagged selectors, whose exact `type` tag and payload are shown above.

### 26.7 Exhaustive active type and error inventory

The following is the controlling inventory for every active normative Phase-B
type declared in §§25--26. Every declaration has exactly one row; `B1` is the
config schema, `B4` is schema-4 report output, and `n/a` is an internal API
type. Frozen A1 declarations are not re-inventoried.

| type | kind | serialized? | owner module | owning parent | schema | purpose | wire representation | supersedes |
|---|---|---:|---|---|---|---|---|---|
| `MechanismEvidenceConfig` | struct | yes | `src/mechanism/config.rs` | CLI config | B1 | config root | struct/root TOML | prior B roots |
| `MechanismHypothesisDefinition` | struct | yes | `src/mechanism/config.rs` | config | B1 | hypothesis definition | array-of-table | prior definitions |
| `MechanismHypothesisId` | type alias | yes | `src/mechanism/config.rs` | hypothesis | B1 | hypothesis ID | frozen `HypothesisId` scalar | untyped ID |
| `IdentifiabilityRequirementId` | newtype | yes | `src/mechanism/config.rs` | identifiability binding | B1 | metric ID | scalar string | untyped metric ID |
| `EvidenceRequirementBinding` | struct | yes | `src/mechanism/config.rs` | hypothesis | B1 | candidate and phase rule | struct | role-bearing binding |
| `EvidenceRequirementStage` | enum | yes | `src/mechanism/config.rs` | requirement | B1 | support/validation phase | snake_case scalar | implicit phase |
| `RequirementGate` | enum | yes | `src/mechanism/config.rs` | requirement/pair | B1 | applicability | snake_case scalar | option gate |
| `EvidenceTargetSelector` | enum | yes | `src/mechanism/config.rs` | requirement | B1 | structural target | tagged `{type,value}` | field-name matching |
| `PhaseBQuantitySemantic` | enum | yes | `src/mechanism/config.rs` | requirement | B1 | quantity meaning | snake_case scalar | inferred meaning |
| `RequiredEvidenceDirection` | enum | yes | `src/mechanism/config.rs` | requirement | B1 | candidate direction | snake_case scalar | inferred direction |
| `EvidenceValidityRequirement` | enum | yes | `src/mechanism/config.rs` | requirement | B1 | accepted validity | snake_case scalar | inferred validity |
| `EvidencePairRequirement` | struct | yes | `src/mechanism/config.rs` | hypothesis | B1 | pair and temporal owner | struct | old pair forms |
| `TemporalRequirement` | enum | yes | `src/mechanism/config.rs` | pair | B1 | pair-scoped invocation | tagged `{type,...}` | global joining |
| `TemporalJoinMode` | enum | yes | `src/mechanism/config.rs` | temporal requirement | B1 | support dispatch | snake_case scalar | implicit combinations |
| `MechanismEvidenceRoleBinding` | struct | yes | `src/mechanism/config.rs` | hypothesis | B1 | sole record-role owner | array-of-table | `EvidenceRoleBinding` |
| `MechanismEvidenceRole` | enum | yes | `src/mechanism/config.rs` | role binding | B1 | record role | snake_case scalar | inferred role |
| `IdentifiabilityBinding` | struct | yes | `src/mechanism/config.rs` | hypothesis | B1 | metric gate/inputs | struct with `input` | implicit inputs |
| `IdentifiabilityInputBinding` | struct | yes | `src/mechanism/config.rs` | identifiability binding | B1 | candidate requirements | nested struct | flat input fields |
| `IdentifiabilityInputSelection` | enum | yes | `src/mechanism/config.rs` | input binding | B1 | exact/all selection | tagged `{type,...}` | implicit selection |
| `TimescaleGate` | struct | yes | `src/mechanism/config.rs` | hypothesis | B1 | pair threshold | struct | old gate |
| `AmplitudeThreshold` | struct | yes | `src/mechanism/config.rs` | amplitude gate | B1 | unit-bearing threshold | `{value,unit}` | bare floor |
| `AmplitudeGate` | struct | yes | `src/mechanism/config.rs` | hypothesis | B1 | amplitude rule | struct | old amplitude gate |
| `ExpectedEffect` | enum | yes | `src/mechanism/config.rs` | amplitude gate | B1 | sign convention | snake_case scalar | inferred sign |
| `RepeatabilityGate` | struct | yes | `src/mechanism/config.rs` | hypothesis | B1 | replicate rule | struct | old repeatability gate |
| `ValidationProtocol` | struct | yes | `src/mechanism/validation.rs` | config | B1 | domain protocol | struct/child tables | old protocol |
| `ValidationCondition` | struct | yes | `src/mechanism/validation.rs` | validation protocol | B1 | condition binding | child table | old condition |
| `ValidationApplicability` | enum | yes | `src/mechanism/config.rs` | hypothesis | B1 | validation applicability | snake_case scalar | boolean applicability |
| `TimescaleEvidenceConfig` | struct | yes | `src/mechanism/config.rs` | config | B1 | algorithm tag | struct | implicit algorithm |
| `AmplitudeEvidenceConfig` | struct | yes | `src/mechanism/config.rs` | config | B1 | algorithm tag | struct | implicit algorithm |
| `RepeatabilityEvidenceConfig` | struct | yes | `src/mechanism/config.rs` | config | B1 | algorithm tag | struct | implicit algorithm |
| `MixedStateConfig` | struct | yes | `src/mechanism/config.rs` | config | B1 | classification source | struct | inferred classifier |
| `IdentifiabilityGateConfig` | struct | yes | `src/mechanism/config.rs` | config | B1 | algorithm tag | struct | implicit algorithm |
| `HypothesisPromotionConfig` | struct | yes | `src/mechanism/config.rs` | config | B1 | support floor | struct | implicit promotion |
| `TimescaleAlgorithm` | enum | yes | `src/mechanism/config.rs` | timescale config | B1 | closed algorithm | snake_case scalar | free string |
| `AmplitudeAlgorithm` | enum | yes | `src/mechanism/config.rs` | amplitude config | B1 | closed algorithm | snake_case scalar | free string |
| `RepeatabilityAlgorithm` | enum | yes | `src/mechanism/config.rs` | repeatability config | B1 | closed algorithm | snake_case scalar | free string |
| `ClassificationSource` | enum | yes | `src/mechanism/config.rs` | mixed-state config | B1 | source authority | snake_case scalar | inferred source |
| `IdentifiabilityAlgorithm` | enum | yes | `src/mechanism/config.rs` | identifiability config | B1 | closed algorithm | snake_case scalar | free string |
| `EvidenceTemporalMetadataCatalog` | struct | yes | `src/mechanism/preparation.rs` | preparation | B4 | ID-keyed metadata | map | vector metadata |
| `EvidenceTemporalMetadata` | struct | yes | `src/mechanism/preparation.rs` | catalog | B4 | one temporal row | struct | missing row |
| `EvidenceTemporalSupport` | enum | yes | `src/mechanism/preparation.rs` | metadata | B4 | point/window/event | snake_case payload | prior support |
| `ClockId` | newtype | yes | `src/mechanism/preparation.rs` | metadata | B4 | comparable clock | scalar string | clock basis |
| `TemporalClassificationMetadata` | struct | yes | `src/mechanism/preparation.rs` | metadata | B4 | producer fractions | struct | inferred fractions |
| `TemporalClassificationSource` | enum | yes | `src/mechanism/preparation.rs` | classification | B4 | source authority | snake_case scalar | inferred source |
| `TemporalSupportProvenance` | struct | yes | `src/mechanism/preparation.rs` | metadata | B4 | field trace | struct | path guessing |
| `PhaseBEvidencePreparationInputs` | struct | no | `src/mechanism/preparation.rs` | preparation API | n/a | authoritative input | n/a | open input |
| `PhaseBEvidencePreparation` | struct | no | `src/mechanism/preparation.rs` | preparation API | n/a | bundle plus metadata | n/a | post-assembly owner |
| `PhaseBSourceArtifactRefs` | struct | no | `src/runners/mechanism.rs` | runner API | n/a | explicit source paths | n/a | hidden path lookup |
| `TemporalJoinConfig` | struct | yes | `src/mechanism/temporal.rs` | config | B1 | temporal thresholds | struct | `TemporalAssessmentPolicy` |
| `MixedStatePolicy` | enum | yes | `src/mechanism/temporal.rs` | temporal config | B1 | classification policy | internally tagged `{kind,...}` | prior policy |
| `TemporalJoinAssessment` | struct | yes | `src/mechanism/temporal.rs` | hypothesis assessment | B4 | temporal result | struct | incomplete result |
| `TemporalJoinOutcome` | enum | yes | `src/mechanism/temporal.rs` | temporal assessment | B4 | disposition | snake_case scalar | old outcome |
| `TemporalJoinReasonCode` | enum | yes | `src/mechanism/temporal.rs` | temporal assessment | B4 | reason | snake_case scalar | old reason |
| `BoundHypothesisEvidence` | struct | no | `src/mechanism/evidence.rs` | binding API | n/a | deterministic candidates/pairs | n/a | implicit candidates |
| `BoundEvidencePair` | struct | no | `src/mechanism/evidence.rs` | bound evidence | n/a | oriented pair | n/a | implicit pair |
| `TimescaleAssessment` | struct | yes | `src/mechanism/timescale.rs` | hypothesis assessment | B4 | timescale result | struct | old result |
| `TimescaleStatus` | enum | yes | `src/mechanism/timescale.rs` | timescale assessment | B4 | disposition | snake_case scalar | generic status |
| `AmplitudeAssessment` | struct | yes | `src/mechanism/amplitude.rs` | hypothesis assessment | B4 | amplitude result | struct | old result |
| `AmplitudeStatus` | enum | yes | `src/mechanism/amplitude.rs` | amplitude assessment | B4 | disposition | snake_case scalar | generic status |
| `AmplitudeReasonCode` | enum | yes | `src/mechanism/amplitude.rs` | amplitude assessment | B4 | reason | snake_case scalar | free reason |
| `RepeatabilityAssessment` | struct | yes | `src/mechanism/repeatability.rs` | hypothesis assessment | B4 | repeatability result | struct | old result |
| `RepeatabilityStatus` | enum | yes | `src/mechanism/repeatability.rs` | repeatability assessment | B4 | disposition | snake_case scalar | generic status |
| `RequirementContradictionSummary` | struct | yes | `src/mechanism/evaluation.rs` | hypothesis assessment | B4 | direct conflict | struct | unpersisted count |
| `IdentifiabilityAssessment` | struct | yes | `src/mechanism/identifiability.rs` | hypothesis assessment | B4 | metric result | struct | old result |
| `IdentifiabilityAssessmentStatus` | enum | yes | `src/mechanism/identifiability.rs` | identifiability assessment | B4 | metric disposition | snake_case scalar | generic status |
| `IdentifiabilityAssessmentReasonCode` | enum | yes | `src/mechanism/identifiability.rs` | identifiability assessment | B4 | metric reason | snake_case scalar | free reason |
| `ValidationAssessment` | struct | yes | `src/mechanism/validation.rs` | hypothesis assessment | B4 | validation result | struct | missing result |
| `ValidationReasonCode` | enum | yes | `src/mechanism/validation.rs` | validation assessment | B4 | validation reason | snake_case scalar | free validation reason |
| `ValidationProtocolStatus` | enum | yes | `src/mechanism/validation.rs` | validation assessment | B4 | disposition | snake_case scalar | old status |
| `HypothesisEvidenceLevel` | enum | yes | `src/mechanism/promotion.rs` | hypothesis assessment | B4 | level | snake_case scalar | old level |
| `PhaseBHypothesisReasonCode` | enum | yes | `src/mechanism/promotion.rs` | hypothesis assessment | B4 | promotion reason | snake_case scalar | old reasons |
| `ComponentInterpretationAssessment` | struct | yes | `src/mechanism/promotion.rs` | hypothesis assessment | B4 | component matrix row | struct | partial output |
| `ComponentInterpretationReasonCode` | enum | yes | `src/mechanism/promotion.rs` | component assessment | B4 | component reason | snake_case scalar | free reason |
| `PhaseBHypothesisAssessment` | struct | yes | `src/results/mechanism.rs` | report | B4 | complete B result | struct | old output |
| `HypothesisHistoryEntry` | struct | yes | `src/mechanism/history.rs` | report | B4 | append-only event | struct | old history |
| `PhaseBEvidencePreparationError` | error enum | no | `src/mechanism/preparation.rs` | preparation API | n/a | preparation failures | n/a | untyped errors |
| `TemporalJoinError` | error enum | no | `src/mechanism/temporal.rs` | temporal API | n/a | join failures | n/a | generic errors |
| `EvidenceBindingError` | error enum | no | `src/mechanism/evidence.rs` | binding API | n/a | binding failures | n/a | implicit failures |
| `HypothesisDefinitionError` | error enum | no | `src/mechanism/config.rs` | definition validation | n/a | config failures | n/a | string errors |
| `MechanismAssessmentError` | error enum | no | `src/mechanism/evaluation.rs` | assessment APIs | n/a | cross-stage failures | n/a | scattered errors |
| `AmplitudeAssessmentError` | error enum | no | `src/mechanism/amplitude.rs` | amplitude API | n/a | amplitude failures | n/a | generic errors |
| `RepeatabilityAssessmentError` | error enum | no | `src/mechanism/repeatability.rs` | repeatability API | n/a | repeatability failures | n/a | generic errors |
| `IdentifiabilityAssessmentError` | error enum | no | `src/mechanism/identifiability.rs` | identifiability API | n/a | identifiability failures | n/a | generic errors |
| `ValidationAssessmentError` | error enum | no | `src/mechanism/validation.rs` | validation API | n/a | validation failures | n/a | generic errors |
| `ComponentAssessmentError` | error enum | no | `src/mechanism/promotion.rs` | component API | n/a | matrix failures | n/a | generic errors |
| `PhaseBReportAssemblyError` | error enum | no | `src/runners/mechanism.rs` | report/runner APIs | n/a | assembly/serialization failures | n/a | string errors |

The inventory rule is normative: every active Phase-B `struct`, `enum`, type
alias, ID newtype, error enum, reason-code enum, config, assessment, input, or
output declaration has exactly one row above. A missing row or duplicate owner
is a contract error.

### 26.8 Exact production APIs and typed errors

The future implementation must provide these exact signatures. Every input is
explicit; no function reads global state, paths, hidden caches, or an unpassed
artifact.

```rust
// src/mechanism/evidence.rs
pub fn bind_hypothesis_evidence(
    hypothesis: &MechanismHypothesisDefinition,
    preparation: &PhaseBEvidencePreparation,
) -> Result<BoundHypothesisEvidence, EvidenceBindingError>;
pub fn evaluate_direct_contradictions(
    hypothesis: &MechanismHypothesisDefinition,
    bound: &BoundHypothesisEvidence,
    bundle: &EvidenceBundle,
) -> Result<Vec<RequirementContradictionSummary>, MechanismAssessmentError>;

// src/mechanism/timescale.rs
pub fn evaluate_timescale_requirement(
    requirement: &EvidencePairRequirement,
    bound: &BoundHypothesisEvidence,
    bundle: &EvidenceBundle,
    temporal_assessments: &[TemporalJoinAssessment],
    config: &TimescaleEvidenceConfig,
) -> Result<TimescaleAssessment, MechanismAssessmentError>;

// src/mechanism/amplitude.rs
pub fn evaluate_amplitude_requirement(
    gate: &AmplitudeGate,
    bound: &BoundHypothesisEvidence,
    bundle: &EvidenceBundle,
    config: &AmplitudeEvidenceConfig,
) -> Result<AmplitudeAssessment, AmplitudeAssessmentError>;

// src/mechanism/repeatability.rs
pub fn evaluate_repeatability_requirement(
    gate: &RepeatabilityGate,
    bound: &BoundHypothesisEvidence,
    bundle: &EvidenceBundle,
    config: &RepeatabilityEvidenceConfig,
) -> Result<RepeatabilityAssessment, RepeatabilityAssessmentError>;

// src/mechanism/identifiability.rs
pub fn evaluate_identifiability_binding(
    binding: &IdentifiabilityBinding,
    bound: &BoundHypothesisEvidence,
    bundle: &EvidenceBundle,
) -> Result<IdentifiabilityAssessment, IdentifiabilityAssessmentError>;

// src/mechanism/validation.rs
pub fn evaluate_validation_protocol(
    protocol: &ValidationProtocol,
    hypothesis: &MechanismHypothesisDefinition,
    bound: &BoundHypothesisEvidence,
    bundle: &EvidenceBundle,
) -> Result<ValidationAssessment, ValidationAssessmentError>;

// src/mechanism/promotion.rs
pub fn assess_hypothesis(
    hypothesis: &MechanismHypothesisDefinition,
    bound: &BoundHypothesisEvidence,
    temporal_assessments: &[TemporalJoinAssessment],
    contradiction_summaries: &[RequirementContradictionSummary],
    timescale_assessments: &[TimescaleAssessment],
    amplitude_assessments: &[AmplitudeAssessment],
    repeatability_assessments: &[RepeatabilityAssessment],
    identifiability_assessments: &[IdentifiabilityAssessment],
    validation: &ValidationAssessment,
    config: &HypothesisPromotionConfig,
) -> Result<PhaseBHypothesisAssessment, MechanismAssessmentError>;
pub fn assess_components(
    hypothesis: &MechanismHypothesisDefinition,
    hypothesis_assessment: &PhaseBHypothesisAssessment,
    prior_component_statuses: &BTreeMap<ComponentId, InterpretationStatus>,
) -> Result<Vec<ComponentInterpretationAssessment>, ComponentAssessmentError>;

// src/mechanism/history.rs
pub fn update_hypothesis_history(
    previous_history: &[HypothesisHistoryEntry],
    current_assessment: &PhaseBHypothesisAssessment,
) -> Result<Vec<HypothesisHistoryEntry>, MechanismAssessmentError>;

// src/runners/mechanism.rs
pub fn assemble_phase_b_mechanism_report(
    config: &MechanismEvidenceConfig,
    preparation: &PhaseBEvidencePreparation,
    assessments: &[PhaseBHypothesisAssessment],
    prior_report: Option<&MechanismAnalysisReport>,
) -> Result<MechanismAnalysisReport, PhaseBReportAssemblyError>;
```

`BoundHypothesisEvidence` contains the owning hypothesis ID, deterministic
`BTreeMap<EvidenceRequirementId, Vec<EvidenceId>> candidate_evidence_ids`,
sorted `Vec<BoundEvidencePair> pair_bindings`, and validated role rows.
`ValidationAssessment` contains `protocol_id`, status, sorted eligible
validation EvidenceIds, sorted known acquisition-family IDs, passed condition
IDs, and reason codes. `PhaseBHypothesisAssessment` contains all temporal,
timescale, amplitude, repeatability, identifiability, contradiction,
validation, promotion, component, and history outputs. It omits no stage.
Its active schema-4 `history` field is `Vec<HypothesisHistoryEntry>`; the
earlier `PhaseBHypothesisHistory` spelling is migration-only and is not a
controlling type.

The omitted supporting output types are complete:

```rust
pub struct ValidationAssessment {
    pub protocol_id: String,
    pub status: ValidationProtocolStatus,
    pub evidence_ids: Vec<EvidenceId>,
    pub acquisition_family_ids: Vec<AcquisitionFamilyId>,
    pub passed_condition_ids: Vec<String>,
    pub reasons: Vec<ValidationReasonCode>,
}
pub enum ValidationReasonCode {
    ProtocolMissing,
    ConditionNotSatisfied,
    InsufficientIndependentFamilies,
    UnknownAcquisitionFamily,
    Passed,
}
pub struct HypothesisHistoryEntry {
    pub hypothesis_id: MechanismHypothesisId,
    pub prior_level: HypothesisEvidenceLevel,
    pub new_level: HypothesisEvidenceLevel,
    pub assessment_target: Option<InterpretationStatus>,
    pub assessment_index: u64,
    pub reason_codes: Vec<PhaseBHypothesisReasonCode>,
}
pub struct PhaseBSourceArtifactRefs {
    pub eis_fit: PathBuf,
    pub transient_results: PathBuf,
    pub calibration_results: Option<PathBuf>,
    pub estimation_artifact: Option<PathBuf>,
    pub prior_mechanism_artifact: Option<PathBuf>,
}

pub fn run_phase_b_mechanism_compare(
    cli: &CliArgs,
    config: &MechanismEvidenceConfig,
    source_refs: &PhaseBSourceArtifactRefs,
    prior_report: Option<&MechanismAnalysisReport>,
) -> Result<MechanismAnalysisReport, PhaseBReportAssemblyError>;
```

The runner receives parsed CLI, the parsed Phase-B config, source artifact
references, and optional current history/report input. It validates and reads
each source through the current public reader, then calls the preparation,
binding, evaluator, promotion, history, assembly, and serializer stages in the
table below. It does not invent a model-analysis source or bypass A1.

The error architecture is singular by owner. `MechanismAssessmentError` owns
cross-stage evaluation failures; module-specific errors own their module;
`PhaseBReportAssemblyError` owns schema-4 construction, deterministic order,
prior-scope/lineage/dependency handling, and public serialization. The named
error inventory is complete:

```text
PhaseBEvidencePreparationError
TemporalJoinError
EvidenceBindingError
HypothesisDefinitionError
MechanismAssessmentError
AmplitudeAssessmentError
RepeatabilityAssessmentError
IdentifiabilityAssessmentError
ValidationAssessmentError
ComponentAssessmentError
PhaseBReportAssemblyError
```

At minimum, the variants are typed and owned as follows: preparation owns
`TemporalBindingUnresolved`, `TemporalBindingLegacyUnknown`,
`DuplicateTemporalBinding`, `UnknownTemporalEvidenceId`, and
`TemporalCatalogKeyValueMismatch`; temporal owns `SameEvidenceId`,
`UnknownEvidenceId`, `MissingMetadata`, and `InvalidConfig`; binding owns
`RoleBindingMismatch`, `RoleStageMismatch`, `DuplicateRoleBinding`, and
`UnresolvedRequirement`; assessment owns `StrongCriticalContradiction`,
`RequirementNotAssessed`, `RequirementNotSatisfied`, and typed temporal/gate
wrappers; amplitude, repeatability, identifiability, validation, and component
modules own their invalid-input and algorithm-specific variants; report
assembly owns `SchemaMismatch`, `ScopeMismatch`, `LineageFailure`,
`NondeterministicOrder`, and `Serialization`. No stage returns an untyped
string error.

### 26.9 Canonical production API table and fixture exercise map

| stage | function | owner | inputs | output | errors | next |
|---|---|---|---|---|---|---|
| CLI/runner entry | `run_phase_b_mechanism_compare` | `src/runners/mechanism.rs` | parsed CLI, config, current source refs, optional prior report | report | `PhaseBReportAssemblyError` | preparation |
| preparation | `prepare_phase_b_evidence` | `src/mechanism/preparation.rs` | preparation inputs | preparation | preparation error | binding |
| binding | `bind_hypothesis_evidence` | `src/mechanism/evidence.rs` | definition, preparation | bound evidence | binding error | temporal/contradiction |
| temporal | `evaluate_temporal_join` | `src/mechanism/temporal.rs` | two IDs, bundle, metadata, config | temporal assessment | temporal error | gates |
| contradiction | `evaluate_direct_contradictions` | `src/mechanism/evidence.rs` | definition, bound, bundle | summaries | mechanism error | timescale |
| timescale | `evaluate_timescale_requirement` | `src/mechanism/timescale.rs` | pair, bound, bundle, temporal results, config | assessment | mechanism error | amplitude |
| amplitude | `evaluate_amplitude_requirement` | `src/mechanism/amplitude.rs` | gate, bound, bundle, config | assessment | amplitude error | repeatability |
| repeatability | `evaluate_repeatability_requirement` | `src/mechanism/repeatability.rs` | gate, bound, bundle, config | assessment | repeatability error | identifiability |
| identifiability | `evaluate_identifiability_binding` | `src/mechanism/identifiability.rs` | binding, bound, bundle | assessment | identifiability error | validation |
| validation | `evaluate_validation_protocol` | `src/mechanism/validation.rs` | protocol, hypothesis, bound, bundle | assessment | validation error | promotion |
| promotion | `assess_hypothesis` | `src/mechanism/promotion.rs` | all gate results, summaries, config | hypothesis assessment | mechanism error | components |
| components | `assess_components` | `src/mechanism/promotion.rs` | definition, assessment, prior statuses | component rows | component error | history |
| history | `update_hypothesis_history` | `src/mechanism/history.rs` | prior history, current assessment | history entries | mechanism error | assembly |
| assembly | `assemble_phase_b_mechanism_report` | `src/runners/mechanism.rs` | config, preparation, assessments, prior | schema-4 report | report error | serialization |
| serialization | existing public artifact writer/rereader | runner + artifact reader | schema-4 report, output path | persisted report | report error | none |

Every arrow from CLI through serialization has one exact row, owner, output,
and error boundary. PB-FX-09 exercises preparation, binding, temporal
`NotApplicable`, contradictions, timescale, identifiability, promotion,
components, history, and report assembly; its validation-only rows are not
evaluated. PB-FX-10 exercises the same path plus validation and asserts the
`ValidatedForDomain` result. Each fixture field maps to one API input and one
expected intermediate output; no hidden intermediate behavior is allowed.

### 26.10 Supersession, regression, and final self-audit

| searched term | classification |
|---|---|
| `TemporalAssessmentPolicy` | SUPERSEDED in pre-§26 text; zero active definitions or wire uses |
| `TemporalJoinConfig`, `TemporalJoinAssessment` | ACTIVE NORMATIVE only in §26 and their exact owner modules |
| `TemporalEvidenceBindingKey`, `TemporalClockBasis` | SUPERSEDED by the §26 provenance and `ClockId` fields; zero active owners |
| `PhaseBHypothesisHistory` | SUPERSEDED by `HypothesisHistoryEntry`; zero active output fields |
| `CriticalEvidence` | historical/invalid terminology; zero active uses; criticality is only `critical_requirement_ids` |
| `EvidenceRequirementBinding.role` | SUPERSEDED; zero active fields; role owner is `MechanismEvidenceRoleBinding` |
| `RequirementAssessmentStatus` | SUPERSEDED; zero active uses; evaluator-specific statuses are active |
| `target_component_id`, `source_classes`, `input_requirement_ids` | invalid examples; zero active fixture/wire fields |
| `b-artifact-*` | invalid examples; zero serialized fixture uses |

Previously passed areas remain PASS without redesign: hypothesis ownership,
quantity semantics, direct `EvidenceBundle` retirement, EIS temporal Unknown,
unit-bearing amplitude threshold, repeatability algorithm, identifiability
applicability, schema 3→4, validation role ownership, traceability, critical
contradiction pipeline, TOML root, CLI, legacy bundle compatibility, history,
and A1 compatibility. Phase B can be implemented without changing serialized
meaning of `EvidenceRecord`, `EvidenceQuantity`, `EvidencePairKey`,
`ArtifactLineageState`, `TimescalePairUncertainty`, or A1 semantic identity:
**YES**.

```text
Undefined normative types = 0
Undefined normative owners = 0
Unspecified Phase B algorithms = 0
Unspecified scientific thresholds/units = 0
Unspecified compatibility decisions = 0
Normative contradictions = 0
Fixture-to-real-schema contradictions = 0
Incomplete normative positive fixtures = 0
Unmapped active normative types = 0
Pipeline stages without exact API = 0
Invalid ArtifactId placeholders = 0
Wrong active wire-field names = 0
PB-FX-09 serde-valid = yes
PB-FX-10 serde-valid = yes
Implementation invention still required = no
Production Rust modified = NONE
Tests modified = NONE
Fixtures modified = NONE
```

Two competent implementers cannot materially differ about temporal scope,
classification/equilibrium source, pair invocation, event behavior, role/stage
precedence, validation-only requirements, amplitude equation or boundaries,
critical definition, any component status transition, ArtifactId generation,
TOML field names, serde enum shape, type ownership, API signatures, or error
ownership: **NO** for every item.

Main and `codex/mhi-v1-b-mechanism-evidence-integration` remain unchanged.

READY_FOR_PHASE_B_WIRE_EXECUTION_REREVIEW = yes

## 27. Phase B Contract Remediation IX — final temporal invocation, eligibility, wire, and fixture contract

### 27.1 Authority, scope, and finding reconciliation

This section is the sole active normative Phase-B contract for the four
remaining P1 findings. It supersedes every earlier Phase-B definition,
signature, wire example, fixture fragment, API-table row, error list, route,
self-audit, and readiness statement in §§6--10, 14--18, and 20--26 wherever
this section differs. Earlier material is historical, descriptive, or
migration-only. The frozen A1 interfaces, serialized A1 meaning, A1 semantic
hashes, legacy mechanism path, `main`, and
`codex/mhi-v1-b-mechanism-evidence-integration` remain unchanged. This is a
documentation-only amendment and authorizes no production Rust, test, or
fixture change.

| finding | classification | controlling resolution |
|---|---|---|
| PB-WR-01 temporal mode invocation | CONFIRMED | `TemporalJoinRequest` carries the serialized requirement ID, both bound EvidenceIds, and the configured `TemporalJoinMode`; the evaluator accepts only that request. |
| PB-WR-02 temporally eligible contradiction input | CONFIRMED | `EligibleRequirementEvidence` is produced after temporal resolution and is the sole direct-contradiction input; raw candidate sets and independent temporal matching are forbidden there. |
| PB-WR-03 `EvidenceTemporalSupport` wire shape | CONFIRMED | The support enum is internally tagged with `kind`, exact payload fields, exact JSON examples, validation, and a top-level map catalog representation. |
| PB-WR-04 PB-FX-09/PB-FX-10 completeness | CONFIRMED | The complete standalone TOML, source registry, intermediate outputs, and schema-4 output contracts below are the sole active fixture definitions. |

The following supersession sentence is normative:

> The complete PB-FX-09 and PB-FX-10 definitions in §27.7 are the sole ACTIVE
> NORMATIVE definitions. Every earlier partial PB-FX-09/PB-FX-10 block is
> SUPERSEDED and MUST NOT be combined with the canonical fixtures.

### 27.2 PB-WR-01 — explicit temporal invocation contract

The serialized temporal declaration remains owned by the exact pair
requirement. `EvidencePairRequirement` is therefore the exact active
equivalent of the brief's `EvidenceRequirementBinding` parameter in request
construction; no second temporal owner is introduced.

```rust
pub struct TemporalJoinRequest {
    pub requirement_id: EvidenceRequirementId,
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
    pub mode: TemporalJoinMode,
}

pub enum TemporalRequirement {
    NotApplicable,
    Required { join_mode: TemporalJoinMode },
}

pub struct EvidencePairRequirement {
    pub requirement_id: EvidenceRequirementId,
    pub left_requirement_id: EvidenceRequirementId,
    pub right_requirement_id: EvidenceRequirementId,
    pub temporal: TemporalRequirement,
    pub gate: RequirementGate,
}

pub fn build_temporal_join_request(
    requirement: &EvidencePairRequirement,
    bound: &BoundEvidencePair,
) -> Result<Option<TemporalJoinRequest>, EvidenceBindingError>;

pub fn evaluate_temporal_join(
    request: &TemporalJoinRequest,
    bundle: &EvidenceBundle,
    temporal_metadata: &EvidenceTemporalMetadataCatalog,
    config: &TemporalJoinConfig,
) -> Result<TemporalJoinAssessment, TemporalJoinError>;

pub struct TemporalJoinAssessment {
    pub requirement_id: EvidenceRequirementId,
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
    pub mode: TemporalJoinMode,
    pub outcome: TemporalJoinOutcome,
    pub classified_fraction: Option<f64>,
    pub equilibrium_fraction: Option<f64>,
    pub steady_state_fraction: Option<f64>,
    pub reasons: Vec<TemporalJoinReasonCode>,
}
```

`build_temporal_join_request` is the only production construction route.
`NotApplicable` returns `Ok(None)`. `Required { join_mode }` returns exactly
one request whose `requirement_id` is the pair requirement ID, whose left and
right IDs are the oriented IDs in `BoundEvidencePair`, and whose `mode` is the
serialized `join_mode`. The builder never derives a mode from support
variants, timestamps, scopes, or record order. An absent or ambiguous bound
pair is `EvidenceBindingError::UnresolvedPairBinding`.

The evaluator receives the configured mode as data and never changes it. It
first resolves both IDs in the bundle and the temporal catalog, validates
scope/clock/classification prerequisites, then applies the exact support
matrix. The assessment persists the same request mode even when the result is
indeterminate or ineligible.

| requested mode | left support | right support | allowed | algorithm |
|---|---|---|---:|---|
| `PointPoint` | `Point` | `Point` | yes | same clock/scope and `abs(left.timestamp_s-right.timestamp_s) <= point_tolerance_s` |
| `PointWindow` | `Point` | `Window` | yes | point in inclusive `[start_s,end_s]` |
| `WindowPoint` | `Window` | `Point` | yes | point in inclusive `[start_s,end_s]` |
| `WindowWindow` | `Window` | `Window` | yes | positive half-open overlap: `max(start_s) < min(end_s)` |
| `EventEvent` | `Event` | `Event` | yes | same clock/scope and exact nonempty `event_id` equality |
| `PointPoint` | `Window` | `Point` | no | `Indeterminate(ModeSupportMismatch)` |
| `PointWindow` | `Point` | `Point` | no | `Indeterminate(ModeSupportMismatch)` |
| `EventEvent` | `Point` | `Point` | no | `Indeterminate(ModeSupportMismatch)` |
| `WindowWindow` | `Event` | `Event` | no | `Indeterminate(ModeSupportMismatch)` |
| any | `Event` | `Point`/`Window` | no | `Indeterminate(ModeSupportMismatch)` |
| any | `Point`/`Window` | `Event` | no | `Indeterminate(ModeSupportMismatch)` |
| any | `Aggregate` or `Unknown` | any | no | `Indeterminate(ModeSupportMismatch)` |
| any | any | `Aggregate` or `Unknown` | no | `Indeterminate(ModeSupportMismatch)` |
| any | support variant not matching the requested row | any | no | `Indeterminate(ModeSupportMismatch)` |

The matrix is exact; runtime support may not override the configured mode.
`ModeSupportMismatch` is the canonical mismatch behavior: the evaluator
returns `Ok(TemporalJoinAssessment { outcome: Indeterminate, reasons:
['ModeSupportMismatch'] })`, with no typed error. `UnknownEvidenceId`,
`MissingTemporalMetadata`, same-ID requests, invalid configuration, and
invalid catalog data remain typed errors as specified in §27.4. Scope or
clock mismatch uses the existing `Indeterminate` reason path only after the
mode/support row is compatible.

### 27.3 PB-WR-02 — temporal eligibility precedes contradiction

Binding is structural only. It selects candidate IDs, validates the declared
target/source/quantity/availability/validity shape, resolves exact pair
bindings, and validates role tuples. It does not decide final temporal
eligibility and it does not classify direct contradictions.

```rust
pub struct BoundRequirementEvidence {
    pub requirement_id: EvidenceRequirementId,
    pub candidate_evidence_ids: Vec<EvidenceId>,
}

pub struct EligibleRequirementEvidence {
    pub requirement_id: EvidenceRequirementId,
    pub support_evidence_ids: Vec<EvidenceId>,
    pub contradictory_evidence_ids: Vec<EvidenceId>,
    pub temporally_ineligible_evidence_ids: Vec<EvidenceId>,
    pub indeterminate_evidence_ids: Vec<EvidenceId>,
    pub temporal_assessments: Vec<TemporalJoinAssessment>,
}

pub fn evaluate_requirement_eligibility(
    hypothesis: &MechanismHypothesisDefinition,
    bound: &BoundHypothesisEvidence,
    preparation: &PhaseBEvidencePreparation,
    config: &MechanismEvidenceConfig,
) -> Result<Vec<EligibleRequirementEvidence>, MechanismAssessmentError>;

pub fn evaluate_direct_contradictions(
    hypothesis: &MechanismHypothesisDefinition,
    eligible: &[EligibleRequirementEvidence],
    bundle: &EvidenceBundle,
) -> Result<Vec<RequirementContradictionSummary>, MechanismAssessmentError>;
```

Eligibility is evaluated in this frozen order for every requirement:

```text
1. requirement structural candidate selection
2. availability
3. validity
4. experiment/sensor/channel scope
5. quantity semantic compatibility
6. required temporal assessment
7. temporal outcome
8. support or contradiction direction classification
9. strength filtering where the requirement requires it
→ EligibleRequirementEvidence
```

An `Eligible` temporal outcome permits the record to proceed. `Ineligible`
places its ID only in `temporally_ineligible_evidence_ids`.
`Indeterminate` places its ID only in `indeterminate_evidence_ids`.
`MissingEvidence`/missing structural input is excluded from all eligible ID
vectors and persists the applicable `MissingRequiredEvidence` or
`TemporalAssessmentMissing` reason. A required temporal assessment that is
absent from the preparation results is the typed
`MechanismAssessmentError::TemporalAssessmentMissing { requirement_id }`;
an existing `Ineligible` or `Indeterminate` assessment is a normal persisted
eligibility result, not an exception.

The direct contradiction evaluator consumes only the `eligible` slice. It
must not rerun temporal matching, infer eligibility from raw support, inspect
`BoundHypothesisEvidence.candidate_evidence_ids`, or inspect an unfiltered
bundle candidate set. For each requirement it considers only
`EligibleRequirementEvidence.contradictory_evidence_ids`, resolves those IDs
in the supplied bundle, sorts and deduplicates them, and emits a summary only
when at least one eligible direct contradiction exists. A critical
contradiction is counted only when the requirement ID is in
`critical_requirement_ids`, the eligible record direction is
`Contradicts`, and strength is at least `Strong`. Amplitude contradictions
never enter this direct path.

### 27.4 PB-WR-03 — exact temporal support and catalog wire contract

The active serialized form is internally tagged JSON/TOML data. Phase-B
schema types use `deny_unknown_fields`; no externally tagged or untagged
alternative is accepted. The scalar timestamp type is the already-approved
`f64`; no new timestamp type is introduced.

```rust
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EvidenceTemporalSupport {
    Point { timestamp_s: f64 },
    Window { start_s: f64, end_s: f64 },
    Event { event_id: String, start_s: f64, end_s: f64 },
    Aggregate,
    Unknown,
}

#[serde(transparent)]
pub struct EvidenceTemporalMetadataCatalog {
    pub entries: BTreeMap<EvidenceId, EvidenceTemporalMetadata>,
}
```

The exact JSON representations are:

```json
{"kind":"point","timestamp_s":120.0}
{"kind":"window","start_s":100.0,"end_s":150.0}
{"kind":"event","event_id":"event-01","start_s":100.0,"end_s":150.0}
{"kind":"aggregate"}
{"kind":"unknown"}
```

`EvidenceTemporalMetadataCatalog` serializes as a top-level JSON object whose
keys are the string form of `EvidenceId`, not as an object with an `entries`
wrapper and not as an ordered vector. Each value is the complete metadata
object:

```json
{
  "eis.parameter.0": {
    "evidence_id": "eis.parameter.0",
    "support": {"kind":"unknown"},
    "clock_id": null,
    "classification": {
      "classified_fraction": null,
      "equilibrium_fraction": null,
      "steady_state_fraction": null,
      "classification_source": "unavailable"
    },
    "provenance": {
      "adapter_id": "adapt_eis_fit",
      "source_artifact_kind": "eis_fit",
      "source_field_paths": ["$.parameters[0].value"]
    }
  }
}
```

The map key and `value.evidence_id` must be bytewise equal. Every consumed
bundle record has exactly one catalog entry. Duplicate source entries are
rejected as `PhaseBEvidencePreparationError::DuplicateTemporalMetadata`;
unknown IDs are rejected as `PhaseBEvidencePreparationError::UnknownTemporalEvidenceId`;
missing required entries are rejected as
`TemporalJoinError::MissingTemporalMetadata`; a catalog key that does not
match its value is `PhaseBEvidencePreparationError::TemporalCatalogKeyValueMismatch`.

Validation is normative: every timestamp is finite; `Window` and `Event`
require finite `start_s <= end_s`; `Event.event_id` is a nonempty valid
string ID; `Aggregate` and `Unknown` have no payload fields. Invalid support
bounds are `PhaseBEvidencePreparationError::InvalidTemporalSupportBounds`;
invalid event IDs are `PhaseBEvidencePreparationError::InvalidTemporalEventId`.
Unknown fields, duplicate fields, wrong `kind`, wrong payload, non-finite
numbers, and map/vector shape changes are serde/configuration failures.

### 27.5 Exact error ownership

The active error architecture remains singular by owner. The required
temporal/eligibility cases are:

| condition | owner and exact representation |
|---|---|
| mode/support mismatch | `TemporalJoinReasonCode::ModeSupportMismatch`; result `Indeterminate`, not an exception |
| missing temporal metadata | `TemporalJoinError::MissingTemporalMetadata` |
| unknown temporal EvidenceId | `TemporalJoinError::UnknownEvidenceId` |
| duplicate temporal metadata | `PhaseBEvidencePreparationError::DuplicateTemporalMetadata` |
| invalid temporal support bounds | `PhaseBEvidencePreparationError::InvalidTemporalSupportBounds` |
| catalog key/value mismatch | `PhaseBEvidencePreparationError::TemporalCatalogKeyValueMismatch` |
| missing required eligibility assessment | `MechanismAssessmentError::TemporalAssessmentMissing` |
| eligible record excluded by temporal outcome | persisted in `EligibleRequirementEvidence`, no exception |

These variants are added to the exhaustive inventory in §27.8; no parallel
generic temporal or contradiction error hierarchy is permitted.

### 27.6 Canonical source-fixture rules and runtime identities

The fixture root remains `tests/fixtures/phase_b/`. PB-FX-09 and PB-FX-10 are
source-artifact scenarios. Their JSON files are created later through the
public production writers and reread through public readers. No fixture
serializes an `EvidenceBundle`, `EvidenceId`, direction, strength, role, or
temporal catalog entry directly. All producer-required fields are present in
the production-typed source object; fields not consumed by Phase B retain the
exact valid value emitted by that object's public fixture constructor and are
not omitted by a hand-written reduced schema.

The following runtime variables are generated by writing and rereading the
source artifacts and asserting their `Known.identity.artifact_id` values:
`EIS_ARTIFACT_ID`, `TRANSIENT_ARTIFACT_ID`, `CALIBRATION_ARTIFACT_ID`,
`ESTIMATION_ARTIFACT_ID`, and, for the output, `MECHANISM_ARTIFACT_ID`.
Each is exactly `sha256:` followed by 64 lowercase hexadecimal characters.
The variables are runtime-derived identities, not serialized placeholders.

#### Canonical complete source registry for PB-FX-09 and PB-FX-10

| fixture path | artifact kind/schema | runtime ArtifactId variable | exact source field(s) | adapter | EvidenceId | EvidenceTarget | adapter direction | unit | temporal metadata | role | requirement ID |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `e2e/eis_fit_e2e_1.json` | `eis_fit` / 3 / current | `EIS_ARTIFACT_ID` | `$.parameters[0].element_id="b-eis-tau"`, `$.parameters[0].unit="s"`, `$.parameters[0].value=1.0`; all remaining `EisFitArtifact` fields are valid constructor output | `adapt_eis_fit` | `eis.parameter.0` | `ModelComponent("b-eis-tau")` | `Neutral` | `s` | `Unknown`; `clock_id=null`; all fractions unavailable | `Support` | `b-eis-tau` |
| `e2e/transient_analysis_e2e_1.json` | `transient_analysis` / 3 / current | `TRANSIENT_ARTIFACT_ID` | selected `events[0]`; `segment.fitted_time_local=[0.0,10.0]`; selected successful fit `derived_features.tau_fast_s=1.0`; all remaining `TransientAnalysisReport` fields are valid constructor output | `adapt_transient_analysis` | `transient.event.0.tau_fast_s` | `ModelComponent("tau_fast_s")` | `Neutral` | `s` | `Window { start_s=0.0, end_s=10.0 }`; `clock_id=null`; fractions unavailable | `Support` | `b-transient-tau` |
| `e2e/calibration_observations_e2e_2.json` | `calibration_observations` / 3 / current | `CALIBRATION_ARTIFACT_ID` | `$.observations[0].experiment_id="b-e2e-1"`, `analyte="b-validation-calibration"`, `potential_v=0.25`; all remaining `CalibrationObservationSet` fields are valid constructor output | `try_adapt_calibration_observations` | `calibration.observation.0` | `ModelComponent("b-validation-calibration")` | `Neutral` | `V` | `Unknown`; `clock_id=null`; all fractions unavailable | `Validation` | `b-validation-calibration` |
| `e2e/state_estimation_e2e_2.json` | `state_estimation` / 4 / current | `ESTIMATION_ARTIFACT_ID` | `$.estimates[0].timestamp_s=5.0`, `filtered_state[0].name="b-validation-estimation"`, `.value=0.25`, `.unit="V"`; all remaining `StateEstimationReport` fields are valid constructor output | `adapt_state_estimation` | `estimation.point.0.state.0` | `ModelComponent("b-validation-estimation")` | `Neutral` | `V` | `Point { timestamp_s=5.0 }`; `clock_id=null`; no producer classification means all fractions unavailable | `Validation` | `b-validation-estimation` |

PB-FX-09 consumes exactly the first two rows. PB-FX-10 consumes all four.
The source family IDs are `b-family-eis`, `b-family-transient`,
`b-family-calibration`, and `b-family-estimation`; every source has Known
lineage, scope `b-e2e-1`, no shared ancestor, and pairwise A1 independence
where a pair is evaluated. The A1 adapter direction is `Neutral`; the
serialized requirement's `expected_direction="candidate_presence"` means
availability/candidate acceptance, not a fabricated A1 `Supports` direction.

### 27.7 Canonical PB-FX-09 and PB-FX-10 contracts

#### PB-FX-09 — COMPLETE CANONICAL CONTRACT

Path: `config/e2e_experimentally_supported.toml`. This is a complete,
standalone TOML document; it does not inherit fields from another fixture.

```toml
schema_version = 1

[timescale]
algorithm = "log_ratio_v1"

[amplitude]
algorithm = "signed_relative_error_v1"

[repeatability]
algorithm = "independent_ln_tau_sample_sd_v1"

[temporal]
point_tolerance_s = 0.0
window_overlap_rule = "positive_duration"
event_identity_rule = "exact"
minimum_classified_fraction = 0.0
minimum_equilibrium_fraction = 0.0
clock_mismatch_behavior = "indeterminate"
scope_mismatch_behavior = "indeterminate"

[temporal.mixed_state_policy]
kind = "require_all_steady"
allow_quasi_equilibrium = false

[mixed_state]
classification_source = "producer_owned_only"

[identifiability]
algorithm = "bound_inputs_v1"

[promotion]
minimum_independent_support = 2

[[hypotheses]]
hypothesis_id = "b-hypothesis"
display_name = "B E2E experimentally supported"
target_components = ["b-eis-tau", "tau_fast_s"]
critical_requirement_ids = []
validation_applicability = "not_applicable"
amplitude_gates = []
repeatability_gates = []

[[hypotheses.evidence_requirements]]
requirement_id = "b-eis-tau"
target_selector = { type = "exact_component", value = "b-eis-tau" }
source_class_selectors = ["model_derived"]
source_field_path = "$.parameters[0].value"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
stage = "support"

[[hypotheses.evidence_requirements]]
requirement_id = "b-transient-tau"
target_selector = { type = "exact_component", value = "tau_fast_s" }
source_class_selectors = ["model_derived"]
source_field_path = "$.events[0].candidate_fits[].derived_features.tau_fast_s"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
stage = "support"

[[hypotheses.pair_requirements]]
requirement_id = "b-timescale-pair"
left_requirement_id = "b-eis-tau"
right_requirement_id = "b-transient-tau"
temporal = { type = "not_applicable" }
gate = "required"

[hypotheses.timescale_gate]
pair_requirement_id = "b-timescale-pair"
maximum_log_distance = 0.0

[[hypotheses.identifiability_bindings]]
requirement_id = "b-mode-separation"
gate = "required"
kind = "mode_separation"
threshold = 1.0
input = { requirement_ids = ["b-eis-tau", "b-transient-tau"], selection = { type = "exact_pair", pair_requirement_id = "b-timescale-pair" } }

[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-eis-tau"
evidence_id = "eis.parameter.0"
role = "support"

[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-transient-tau"
evidence_id = "transient.event.0.tau_fast_s"
role = "support"
```

PB-FX-09 intermediate outputs are complete and deterministic:

| API | exact expected output |
|---|---|
| `prepare_phase_b_evidence` | `bundle.records=[eis.parameter.0,transient.event.0.tau_fast_s]` in bytewise ID order; `temporal_metadata` has exactly those two map keys with the registry rows above; no errors. |
| `bind_hypothesis_evidence` | `candidate_evidence_ids={b-eis-tau:[eis.parameter.0], b-transient-tau:[transient.event.0.tau_fast_s]}`; one pair binding `{pair_requirement_id:b-timescale-pair,left:eis.parameter.0,right:transient.event.0.tau_fast_s}`; the two Support role rows exactly match TOML. |
| `build_temporal_join_request` | `None` for `b-timescale-pair`, because its serialized `TemporalRequirement` is `NotApplicable`; no temporal evaluator call is made. |
| `evaluate_temporal_join` | empty assessment list; there is no implicit join and no inferred mode. |
| `evaluate_requirement_eligibility` | two rows; support IDs are respectively `[eis.parameter.0]` and `[transient.event.0.tau_fast_s]`; contradictory, temporally-ineligible, and indeterminate ID vectors are empty; `temporal_assessments=[]`. |
| `evaluate_direct_contradictions` | `[]`; no raw candidate or non-eligible record is inspected. |
| `evaluate_timescale_requirement` | `{pair_requirement_id:b-timescale-pair,status:satisfied,evidence_ids:[eis.parameter.0,transient.event.0.tau_fast_s],log_distance:0.0}`. |
| `evaluate_amplitude_requirement` | `[]`; no amplitude gate is configured. |
| `evaluate_repeatability_requirement` | `[]`; no repeatability gate is configured. |
| `evaluate_identifiability_binding` | `{requirement_id:b-mode-separation,status:satisfied,metric_value:1.0,evidence_ids:[eis.parameter.0,transient.event.0.tau_fast_s],reasons:[threshold_satisfied]}`. |
| `evaluate_validation_protocol` | `{protocol_id:"not-applicable",status:not_applicable,evidence_ids:[],acquisition_family_ids:[],passed_condition_ids:[],reasons:[]}`. |
| `assess_hypothesis` | `hypothesis_id=b-hypothesis`, `evidence_level=ExperimentallySupported`, timescale list as above, amplitude/repeatability/temporal/contradiction lists empty, identifiability list as above, `validation_status=NotApplicable`, `reason_codes=[]`. |
| `assess_components` | exactly two rows sorted by component ID; both prior `Hypothesized`, target/result `ExperimentallySupported`, supporting hypothesis `b-hypothesis`, evidence IDs `[eis.parameter.0,transient.event.0.tau_fast_s]`, reasons `[HypothesisSupported,ExperimentallySupported]`. |
| `update_hypothesis_history` | `[]` when no prior schema-4 report is supplied. |
| `assemble_phase_b_mechanism_report` | schema 4, one hypothesis assessment, `hypothesis_history=[]`, exact B fields below, runtime-derived Known output lineage, and no serialization error. |

The complete Phase-B portion of the expected schema-4 report is:

```json
{
  "schema_version": 4,
  "lineage": {"Known":{"identity":{"artifact_id":"MECHANISM_ARTIFACT_ID","artifact_kind":"mechanism_analysis","schema_version":4,"producer_version":"RUNTIME_PACKAGE_VERSION","experiment_scope":{"Single":{"experiment_id":"b-e2e-1"}},"sensor_scope":"Unspecified","channel_scope":"Unspecified","acquisition_families":{"Known":["b-family-eis","b-family-transient"]},"semantic_sha256":"MECHANISM_SEMANTIC_SHA256"},"direct_dependencies":[{"artifact_id":"EIS_ARTIFACT_ID","artifact_kind":"eis_fit","role":"TransformationInput"},{"artifact_id":"TRANSIENT_ARTIFACT_ID","artifact_kind":"transient_analysis","role":"TransformationInput"}]}},
  "analysis_id": "mechanism:b-hypothesis",
  "records": [],
  "eis_timescales": [],
  "transient_timescales": [],
  "comparisons": [],
  "legacy_hypotheses": [],
  "trends": [],
  "configuration": {
    "schema_version": 1,
    "confidence_level": 0.95,
    "allow_warning_fits": true,
    "ratio_weak": 10.0,
    "ratio_moderate": 3.0,
    "ratio_strong": 1.5,
    "log_distance_weak": 1.0,
    "log_distance_moderate": 0.5,
    "log_distance_strong": 0.1761,
    "minimum_fit_quality": 0.0,
    "compatibility_ratio_lower": 0.5,
    "compatibility_ratio_upper": 2.0,
    "require_experiment_id": true,
    "require_sensor_id": false,
    "minimum_replicates_for_strong": 3,
    "trend_minimum_records": 3,
    "trend_independent_variable": "sensor_age_days",
    "frequency_boundary_margin": 0.1,
    "monte_carlo_samples": 10000,
    "seed": 42,
    "selected_model_only": true,
    "hypotheses": []
  },
  "provenance": null,
  "warnings": [],
  "transient_configuration": null,
  "hypothesis_assessments": [{
    "definition": {
      "hypothesis_id": "b-hypothesis",
      "display_name": "B E2E experimentally supported",
      "target_components": ["b-eis-tau", "tau_fast_s"],
      "evidence_requirements": [
        {"requirement_id":"b-eis-tau","target_selector":{"type":"exact_component","value":"b-eis-tau"},"source_class_selectors":["model_derived"],"source_field_path":"$.parameters[0].value","quantity_semantic":"time_constant","required_unit":"s","expected_direction":"candidate_presence","validity_requirement":"valid","gate":"required","stage":"support"},
        {"requirement_id":"b-transient-tau","target_selector":{"type":"exact_component","value":"tau_fast_s"},"source_class_selectors":["model_derived"],"source_field_path":"$.events[0].candidate_fits[].derived_features.tau_fast_s","quantity_semantic":"time_constant","required_unit":"s","expected_direction":"candidate_presence","validity_requirement":"valid","gate":"required","stage":"support"}
      ],
      "pair_requirements": [{"requirement_id":"b-timescale-pair","left_requirement_id":"b-eis-tau","right_requirement_id":"b-transient-tau","temporal":{"type":"not_applicable"},"gate":"required"}],
      "critical_requirement_ids": [],
      "timescale_gate": {"pair_requirement_id":"b-timescale-pair","maximum_log_distance":0.0},
      "amplitude_gates": [],
      "repeatability_gates": [],
      "identifiability_bindings": [{"requirement_id":"b-mode-separation","gate":"required","kind":"mode_separation","threshold":1.0,"input":{"requirement_ids":["b-eis-tau","b-transient-tau"],"selection":{"type":"exact_pair","pair_requirement_id":"b-timescale-pair"}}}],
      "validation_applicability": "not_applicable",
      "role_bindings": [
        {"hypothesis_id":"b-hypothesis","requirement_id":"b-eis-tau","evidence_id":"eis.parameter.0","role":"support"},
        {"hypothesis_id":"b-hypothesis","requirement_id":"b-transient-tau","evidence_id":"transient.event.0.tau_fast_s","role":"support"}
      ]
    },
    "current": {
      "hypothesis_id": "b-hypothesis",
      "evidence_level": "experimentally_supported",
      "temporal_join_assessments": [],
      "timescale_assessments": [{"pair_requirement_id":"b-timescale-pair","status":"satisfied","evidence_ids":["eis.parameter.0","transient.event.0.tau_fast_s"],"log_distance":0.0}],
      "amplitude_assessments": [],
      "repeatability_assessments": [],
      "identifiability_assessments": [{"requirement_id":"b-mode-separation","status":"satisfied","metric_value":1.0,"evidence_ids":["eis.parameter.0","transient.event.0.tau_fast_s"],"reasons":["threshold_satisfied"]}],
      "contradiction_summaries": [],
      "reason_codes": [],
      "component_assessments": [
        {"component_id":"b-eis-tau","prior_status":"hypothesized","assessment_target":"experimentally_supported","resulting_status":"experimentally_supported","supporting_hypothesis_id":"b-hypothesis","evidence_ids":["eis.parameter.0","transient.event.0.tau_fast_s"],"reasons":["hypothesis_supported","experimentally_supported"]},
        {"component_id":"tau_fast_s","prior_status":"hypothesized","assessment_target":"experimentally_supported","resulting_status":"experimentally_supported","supporting_hypothesis_id":"b-hypothesis","evidence_ids":["eis.parameter.0","transient.event.0.tau_fast_s"],"reasons":["hypothesis_supported","experimentally_supported"]}
      ],
      "validation_status":"not_applicable",
      "history":[]
    }
  }],
  "hypothesis_history": []
}
```

The uppercase values in the `lineage` object are runtime-derived variables,
not serialized placeholders: `MECHANISM_ARTIFACT_ID` and
`MECHANISM_SEMANTIC_SHA256` are the same valid `sha256:` identity and
64-hex semantic digest, and `RUNTIME_PACKAGE_VERSION` is the exact package
producer version. The schema-3 legacy fields retained by
`MechanismAnalysisReport` are fully literal above: empty `records`,
`eis_timescales`, `transient_timescales`, `comparisons`, legacy `hypotheses`,
`trends`, `provenance=null`, `warnings=[]`, and `transient_configuration=null`,
plus the complete `ResolvedMechanismConfig` default object. No ellipsis or
unresolved reference is permitted in the serialized report.

The immutable `PB-LEGACY-EMPTY-MECHANISM-REPORT-01` object referenced by the
PB-FX-10 prior input is fully defined as the schema-3 object with
`analysis_id="mechanism:b-hypothesis"`, empty `records`,
`eis_timescales`, `transient_timescales`, `comparisons`, legacy `hypotheses`,
and `trends`, `provenance=null`, `warnings=[]`,
`transient_configuration=null`, and the complete `ResolvedMechanismConfig`
object shown above; its `schema_version=3` and its public-writer-generated
legacy lineage are the only differences from the listed schema-4 envelope.

#### PB-FX-10 — COMPLETE CANONICAL CONTRACT

Path: `config/e2e_validated_for_domain.toml`. This is a complete standalone
TOML document and repeats every root, section, hypothesis, gate, selector,
stage, direction, threshold, critical-ID, identifiability, validation, and
role field; it does not use “same as PB-FX-09”.

```toml
schema_version = 1

[timescale]
algorithm = "log_ratio_v1"

[amplitude]
algorithm = "signed_relative_error_v1"

[repeatability]
algorithm = "independent_ln_tau_sample_sd_v1"

[temporal]
point_tolerance_s = 0.0
window_overlap_rule = "positive_duration"
event_identity_rule = "exact"
minimum_classified_fraction = 0.0
minimum_equilibrium_fraction = 0.0
clock_mismatch_behavior = "indeterminate"
scope_mismatch_behavior = "indeterminate"

[temporal.mixed_state_policy]
kind = "require_all_steady"
allow_quasi_equilibrium = false

[mixed_state]
classification_source = "producer_owned_only"

[identifiability]
algorithm = "bound_inputs_v1"

[promotion]
minimum_independent_support = 2

[[hypotheses]]
hypothesis_id = "b-hypothesis"
display_name = "B E2E validated for domain"
target_components = ["b-eis-tau", "b-validation-calibration", "b-validation-estimation", "tau_fast_s"]
critical_requirement_ids = []
validation_applicability = "required"
amplitude_gates = []
repeatability_gates = []

[[hypotheses.evidence_requirements]]
requirement_id = "b-eis-tau"
target_selector = { type = "exact_component", value = "b-eis-tau" }
source_class_selectors = ["model_derived"]
source_field_path = "$.parameters[0].value"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
stage = "support"

[[hypotheses.evidence_requirements]]
requirement_id = "b-transient-tau"
target_selector = { type = "exact_component", value = "tau_fast_s" }
source_class_selectors = ["model_derived"]
source_field_path = "$.events[0].candidate_fits[].derived_features.tau_fast_s"
quantity_semantic = "time_constant"
required_unit = "s"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
stage = "support"

[[hypotheses.evidence_requirements]]
requirement_id = "b-validation-calibration"
target_selector = { type = "exact_component", value = "b-validation-calibration" }
source_class_selectors = ["observed"]
source_field_path = "$.observations[0].potential_v"
quantity_semantic = "calibration_potential"
required_unit = "V"
expected_direction = "candidate_presence"
validity_requirement = "valid"
gate = "required"
stage = "validation"

[[hypotheses.evidence_requirements]]
requirement_id = "b-validation-estimation"
target_selector = { type = "exact_component", value = "b-validation-estimation" }
source_class_selectors = ["model_derived"]
source_field_path = "$.estimates[0].filtered_state[0].value"
quantity_semantic = "electrical_potential"
required_unit = "V"
expected_direction = "candidate_presence"
validity_requirement = "valid_or_not_assessed"
gate = "required"
stage = "validation"

[[hypotheses.pair_requirements]]
requirement_id = "b-timescale-pair"
left_requirement_id = "b-eis-tau"
right_requirement_id = "b-transient-tau"
temporal = { type = "not_applicable" }
gate = "required"

[hypotheses.timescale_gate]
pair_requirement_id = "b-timescale-pair"
maximum_log_distance = 0.0

[[hypotheses.identifiability_bindings]]
requirement_id = "b-mode-separation"
gate = "required"
kind = "mode_separation"
threshold = 1.0
input = { requirement_ids = ["b-eis-tau", "b-transient-tau"], selection = { type = "exact_pair", pair_requirement_id = "b-timescale-pair" } }

[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-eis-tau"
evidence_id = "eis.parameter.0"
role = "support"

[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-transient-tau"
evidence_id = "transient.event.0.tau_fast_s"
role = "support"

[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-validation-calibration"
evidence_id = "calibration.observation.0"
role = "validation"

[[hypotheses.role_bindings]]
hypothesis_id = "b-hypothesis"
requirement_id = "b-validation-estimation"
evidence_id = "estimation.point.0.state.0"
role = "validation"

[validation]
protocol_id = "b-e2e-validation"
version = "1"
minimum_acquisition_families = 2

[[validation.required_conditions]]
condition_id = "b-calibration-condition"
requirement_ids = ["b-validation-calibration"]
experiment_scope = "b-e2e-1"

[[validation.required_conditions]]
condition_id = "b-estimation-condition"
requirement_ids = ["b-validation-estimation"]
experiment_scope = "b-e2e-1"
```

PB-FX-10 intermediate outputs are complete and deterministic:

| API | exact expected output |
|---|---|
| `prepare_phase_b_evidence` | four records exactly `[calibration.observation.0,eis.parameter.0,estimation.point.0.state.0,transient.event.0.tau_fast_s]` in bytewise ID order; four matching catalog map entries; the registry temporal values above; no errors. |
| `bind_hypothesis_evidence` | one candidate ID for every requirement; one exact timescale pair; four role tuples exactly as TOML; validation rows are stage `Validation` and role `Validation`. |
| `build_temporal_join_request` | `None` for `b-timescale-pair` because the serialized pair temporal requirement is `NotApplicable`; no inferred temporal request. |
| `evaluate_temporal_join` | empty assessment list; no temporal stage is silently inserted for a NotApplicable pair. |
| `evaluate_requirement_eligibility` | four rows; the two support IDs appear only in support vectors; the two validation IDs appear only in validation requirement support vectors; all contradictory, temporally-ineligible, and indeterminate vectors are empty. |
| `evaluate_direct_contradictions` | `[]`; critical requirement IDs are empty and no non-eligible record is considered. |
| `evaluate_timescale_requirement` | `{pair_requirement_id:b-timescale-pair,status:satisfied,evidence_ids:[eis.parameter.0,transient.event.0.tau_fast_s],log_distance:0.0}`. |
| `evaluate_amplitude_requirement` | `[]`. |
| `evaluate_repeatability_requirement` | `[]`. |
| `evaluate_identifiability_binding` | `{requirement_id:b-mode-separation,status:satisfied,metric_value:1.0,evidence_ids:[eis.parameter.0,transient.event.0.tau_fast_s],reasons:[threshold_satisfied]}`. |
| `evaluate_validation_protocol` | `{protocol_id:b-e2e-validation,status:satisfied,evidence_ids:[calibration.observation.0,estimation.point.0.state.0],acquisition_family_ids:[b-family-calibration,b-family-estimation],passed_condition_ids:[b-calibration-condition,b-estimation-condition],reasons:[passed]}`; exact distinct Known family count `2 >= 2`; the two validation families are pairwise A1 `Independent`. |
| `assess_hypothesis` | `hypothesis_id=b-hypothesis`, `evidence_level=ValidatedForDomain`, validation `Satisfied`, no temporal/amplitude/repeatability/contradiction reasons, and the timescale/identifiability results above. |
| `assess_components` | four rows sorted by component ID; each prior status is `ExperimentallySupported` from the supplied canonical prior schema-4 report, each assessment target and resulting status is `ValidatedForDomain`, supporting hypothesis is `b-hypothesis`, evidence IDs are `[calibration.observation.0,eis.parameter.0,estimation.point.0.state.0,transient.event.0.tau_fast_s]`, and reasons are `[HypothesisSupported,ExperimentallySupported,ValidationSatisfied]`. |
| `update_hypothesis_history` | exactly one entry for the supplied prior report: prior level `ExperimentallySupported`, new level `ValidatedForDomain`, target `ValidatedForDomain`, sequence `1`, sorted source IDs equal the four eligible IDs, sorted reasons equal `[ValidationSatisfied]`, and `history_id` is the exact SHA-256 of the declared history-key bytes. |
| `assemble_phase_b_mechanism_report` | schema 4, one current assessment, exactly one history entry, four ValidatedForDomain component rows, runtime-derived Known output lineage, and no serialization error. |

The PB-FX-10 prior report is the immutable canonical input
`PB-FX-10-PRIOR-EXPERIMENTALLY-SUPPORTED-01`: it contains the same complete
hypothesis definition and source IDs, current level
`ExperimentallySupported`, four component statuses
`ExperimentallySupported`, empty prior history, and the exact schema-4 legacy
fields from `PB-LEGACY-EMPTY-MECHANISM-REPORT-01`. It is a complete canonical
prior object, not “PB-FX-09 plus changes”. The history key bytes are exactly
`hypothesis_id || NUL || prior_level || NUL || new_level || NUL || assessment_hash`;
`assessment_hash` is the RFC-8785 SHA-256 of the complete deterministic
PB-FX-10 assessment view, with no timestamps or human text. The serialized
history entry contains every field:

```json
{
  "history_id": "SHA256(exact declared history-key bytes)",
  "hypothesis_id": "b-hypothesis",
  "prior_level": "experimentally_supported",
  "new_level": "validated_for_domain",
  "assessment_target": "validated_for_domain",
  "assessment_index": 1,
  "reason_codes": ["validation_satisfied"],
  "source_evidence_ids": ["calibration.observation.0","eis.parameter.0","estimation.point.0.state.0","transient.event.0.tau_fast_s"]
}
```

`history_id` above is a deterministic derived value, like the runtime
ArtifactId variables; it is not a free-form placeholder. The implementation
must calculate it from the exact bytes specified and the fixture assertion
compares the resulting 64-hex digest. The complete PB-FX-10 B report has the
following complete schema-4 envelope. Its embedded `definition` is the
complete PB-FX-10 hypothesis object formed from every field in the standalone
TOML immediately above; that canonical TOML is immutable and is the only
permitted cross-reference.

```json
{
  "schema_version": 4,
  "lineage": {"Known":{"identity":{"artifact_id":"MECHANISM_ARTIFACT_ID","artifact_kind":"mechanism_analysis","schema_version":4,"producer_version":"RUNTIME_PACKAGE_VERSION","experiment_scope":{"Single":{"experiment_id":"b-e2e-1"}},"sensor_scope":"Unspecified","channel_scope":"Unspecified","acquisition_families":{"Known":["b-family-calibration","b-family-eis","b-family-estimation","b-family-transient"]},"semantic_sha256":"MECHANISM_SEMANTIC_SHA256"},"direct_dependencies":[{"artifact_id":"CALIBRATION_ARTIFACT_ID","artifact_kind":"calibration_observations","role":"TransformationInput"},{"artifact_id":"EIS_ARTIFACT_ID","artifact_kind":"eis_fit","role":"TransformationInput"},{"artifact_id":"ESTIMATION_ARTIFACT_ID","artifact_kind":"state_estimation","role":"TransformationInput"},{"artifact_id":"TRANSIENT_ARTIFACT_ID","artifact_kind":"transient_analysis","role":"TransformationInput"}]}},
  "analysis_id": "mechanism:b-hypothesis",
  "records": [],
  "eis_timescales": [],
  "transient_timescales": [],
  "comparisons": [],
  "legacy_hypotheses": [],
  "trends": [],
  "configuration": {"schema_version":1,"confidence_level":0.95,"allow_warning_fits":true,"ratio_weak":10.0,"ratio_moderate":3.0,"ratio_strong":1.5,"log_distance_weak":1.0,"log_distance_moderate":0.5,"log_distance_strong":0.1761,"minimum_fit_quality":0.0,"compatibility_ratio_lower":0.5,"compatibility_ratio_upper":2.0,"require_experiment_id":true,"require_sensor_id":false,"minimum_replicates_for_strong":3,"trend_minimum_records":3,"trend_independent_variable":"sensor_age_days","frequency_boundary_margin":0.1,"monte_carlo_samples":10000,"seed":42,"selected_model_only":true,"hypotheses":[]},
  "provenance": null,
  "warnings": [],
  "transient_configuration": null,
  "hypothesis_assessments": [{"definition":{"hypothesis_id":"b-hypothesis","display_name":"B E2E validated for domain","target_components":["b-eis-tau","b-validation-calibration","b-validation-estimation","tau_fast_s"],"evidence_requirements":[{"requirement_id":"b-eis-tau","target_selector":{"type":"exact_component","value":"b-eis-tau"},"source_class_selectors":["model_derived"],"source_field_path":"$.parameters[0].value","quantity_semantic":"time_constant","required_unit":"s","expected_direction":"candidate_presence","validity_requirement":"valid","gate":"required","stage":"support"},{"requirement_id":"b-transient-tau","target_selector":{"type":"exact_component","value":"tau_fast_s"},"source_class_selectors":["model_derived"],"source_field_path":"$.events[0].candidate_fits[].derived_features.tau_fast_s","quantity_semantic":"time_constant","required_unit":"s","expected_direction":"candidate_presence","validity_requirement":"valid","gate":"required","stage":"support"},{"requirement_id":"b-validation-calibration","target_selector":{"type":"exact_component","value":"b-validation-calibration"},"source_class_selectors":["observed"],"source_field_path":"$.observations[0].potential_v","quantity_semantic":"calibration_potential","required_unit":"V","expected_direction":"candidate_presence","validity_requirement":"valid","gate":"required","stage":"validation"},{"requirement_id":"b-validation-estimation","target_selector":{"type":"exact_component","value":"b-validation-estimation"},"source_class_selectors":["model_derived"],"source_field_path":"$.estimates[0].filtered_state[0].value","quantity_semantic":"electrical_potential","required_unit":"V","expected_direction":"candidate_presence","validity_requirement":"valid_or_not_assessed","gate":"required","stage":"validation"}],"pair_requirements":[{"requirement_id":"b-timescale-pair","left_requirement_id":"b-eis-tau","right_requirement_id":"b-transient-tau","temporal":{"type":"not_applicable"},"gate":"required"}],"critical_requirement_ids":[],"timescale_gate":{"pair_requirement_id":"b-timescale-pair","maximum_log_distance":0.0},"amplitude_gates":[],"repeatability_gates":[],"identifiability_bindings":[{"requirement_id":"b-mode-separation","gate":"required","kind":"mode_separation","threshold":1.0,"input":{"requirement_ids":["b-eis-tau","b-transient-tau"],"selection":{"type":"exact_pair","pair_requirement_id":"b-timescale-pair"}}}],"validation_applicability":"required","role_bindings":[{"hypothesis_id":"b-hypothesis","requirement_id":"b-eis-tau","evidence_id":"eis.parameter.0","role":"support"},{"hypothesis_id":"b-hypothesis","requirement_id":"b-transient-tau","evidence_id":"transient.event.0.tau_fast_s","role":"support"},{"hypothesis_id":"b-hypothesis","requirement_id":"b-validation-calibration","evidence_id":"calibration.observation.0","role":"validation"},{"hypothesis_id":"b-hypothesis","requirement_id":"b-validation-estimation","evidence_id":"estimation.point.0.state.0","role":"validation"}]},"current":{"hypothesis_id":"b-hypothesis","evidence_level":"validated_for_domain","temporal_join_assessments":[],"timescale_assessments":[{"pair_requirement_id":"b-timescale-pair","status":"satisfied","evidence_ids":["eis.parameter.0","transient.event.0.tau_fast_s"],"log_distance":0.0}],"amplitude_assessments":[],"repeatability_assessments":[],"identifiability_assessments":[{"requirement_id":"b-mode-separation","status":"satisfied","metric_value":1.0,"evidence_ids":["eis.parameter.0","transient.event.0.tau_fast_s"],"reasons":["threshold_satisfied"]}],"contradiction_summaries":[],"reason_codes":[],"component_assessments":[{"component_id":"b-eis-tau","prior_status":"experimentally_supported","assessment_target":"validated_for_domain","resulting_status":"validated_for_domain","supporting_hypothesis_id":"b-hypothesis","evidence_ids":["calibration.observation.0","eis.parameter.0","estimation.point.0.state.0","transient.event.0.tau_fast_s"],"reasons":["hypothesis_supported","experimentally_supported","validation_satisfied"]},{"component_id":"b-validation-calibration","prior_status":"experimentally_supported","assessment_target":"validated_for_domain","resulting_status":"validated_for_domain","supporting_hypothesis_id":"b-hypothesis","evidence_ids":["calibration.observation.0","eis.parameter.0","estimation.point.0.state.0","transient.event.0.tau_fast_s"],"reasons":["hypothesis_supported","experimentally_supported","validation_satisfied"]},{"component_id":"b-validation-estimation","prior_status":"experimentally_supported","assessment_target":"validated_for_domain","resulting_status":"validated_for_domain","supporting_hypothesis_id":"b-hypothesis","evidence_ids":["calibration.observation.0","eis.parameter.0","estimation.point.0.state.0","transient.event.0.tau_fast_s"],"reasons":["hypothesis_supported","experimentally_supported","validation_satisfied"]},{"component_id":"tau_fast_s","prior_status":"experimentally_supported","assessment_target":"validated_for_domain","resulting_status":"validated_for_domain","supporting_hypothesis_id":"b-hypothesis","evidence_ids":["calibration.observation.0","eis.parameter.0","estimation.point.0.state.0","transient.event.0.tau_fast_s"],"reasons":["hypothesis_supported","experimentally_supported","validation_satisfied"]}],"validation_status":"satisfied","history":[{"history_id":"SHA256(exact declared history-key bytes)","hypothesis_id":"b-hypothesis","prior_level":"experimentally_supported","new_level":"validated_for_domain","assessment_target":"validated_for_domain","assessment_index":1,"reason_codes":["validation_satisfied"],"source_evidence_ids":["calibration.observation.0","eis.parameter.0","estimation.point.0.state.0","transient.event.0.tau_fast_s"]}]}}],
  "hypothesis_history": [{"history_id":"SHA256(exact declared history-key bytes)","hypothesis_id":"b-hypothesis","prior_level":"experimentally_supported","new_level":"validated_for_domain","assessment_target":"validated_for_domain","assessment_index":1,"reason_codes":["validation_satisfied"],"source_evidence_ids":["calibration.observation.0","eis.parameter.0","estimation.point.0.state.0","transient.event.0.tau_fast_s"]}]
}
```

The uppercase lineage values and the `SHA256(...)` history value in this
wire listing are deterministic generated identities whose exact derivation is
specified above; they are not arbitrary fixture text. The complete embedded
PB-FX-10 definition, all four component rows, all validation fields, and the
history entry are required in the actual serialized object.

### 27.8 Active type inventory additions and exact API table

The exhaustive active inventory in §26.7 is retained and gains these rows;
the old rows that name superseded temporal/eligibility forms are no longer
active:

| type | kind | serialized? | owner module | owning parent | schema | purpose | wire representation | supersedes |
|---|---|---:|---|---|---:|---|---|---|
| `TemporalJoinRequest` | struct | no | `src/mechanism/temporal.rs` | temporal evaluator input | n/a | explicit requirement/mode invocation | internal typed request | two-ID implicit invocation |
| `BoundRequirementEvidence` | struct | no | `src/mechanism/evidence.rs` | bound hypothesis evidence | n/a | structural candidates before eligibility | internal typed result | raw candidate ambiguity |
| `EligibleRequirementEvidence` | struct | no | `src/mechanism/evidence.rs` | eligibility output | n/a | temporally resolved direction buckets | internal typed result | contradiction over bound candidates |
| `TemporalJoinReasonCode::ModeSupportMismatch` | reason enum member | yes, B output only | `src/mechanism/temporal.rs` | temporal assessment | B4 | configured mode/support mismatch | `"mode_support_mismatch"` | implicit mode switching |
| `EvidenceBindingError::UnresolvedPairBinding` | error member | no | `src/mechanism/evidence.rs` | request builder | n/a | absent/ambiguous oriented pair | typed error | inferred pair order |
| `PhaseBEvidencePreparationError::DuplicateTemporalMetadata` | error member | no | `src/mechanism/preparation.rs` | preparation | n/a | duplicate catalog input | typed error | duplicate overwrite |
| `PhaseBEvidencePreparationError::InvalidTemporalSupportBounds` | error member | no | `src/mechanism/preparation.rs` | preparation | n/a | invalid point/window/event bounds | typed error | unchecked support |
| `PhaseBEvidencePreparationError::InvalidTemporalEventId` | error member | no | `src/mechanism/preparation.rs` | preparation | n/a | empty/invalid event identity | typed error | unchecked event ID |
| `PhaseBEvidencePreparationError::UnknownTemporalEvidenceId` | error member | no | `src/mechanism/preparation.rs` | preparation | n/a | catalog ID absent from bundle | typed error | implicit catalog membership |
| `PhaseBEvidencePreparationError::TemporalCatalogKeyValueMismatch` | error member | no | `src/mechanism/preparation.rs` | preparation | n/a | map key differs from metadata ID | typed error | unchecked duplicate map key |
| `TemporalJoinError::MissingTemporalMetadata` | error member | no | `src/mechanism/temporal.rs` | temporal evaluator | n/a | absent required catalog row | typed error | silent Unknown |
| `TemporalJoinError::UnknownEvidenceId` | error member | no | `src/mechanism/temporal.rs` | temporal evaluator | n/a | request ID absent from bundle | typed error | unchecked request ID |
| `MechanismAssessmentError::TemporalAssessmentMissing` | error member | no | `src/mechanism/evidence.rs` | eligibility | n/a | required temporal result absent | typed error | contradiction fallback |

The exact production order and API ownership is:

| order | stage | exact function | owner | inputs | output | errors | next |
|---:|---|---|---|---|---|---|---|
| 1 | binding | `bind_hypothesis_evidence` | `src/mechanism/evidence.rs` | hypothesis, preparation | `BoundHypothesisEvidence` | `EvidenceBindingError` | temporal request |
| 2 | temporal request | `build_temporal_join_request` | `src/mechanism/evidence.rs` | pair requirement, bound pair | `Option<TemporalJoinRequest>` | `EvidenceBindingError` | temporal assessment or eligibility |
| 3 | temporal assessment | `evaluate_temporal_join` | `src/mechanism/temporal.rs` | request, bundle, catalog, config | `TemporalJoinAssessment` | `TemporalJoinError` | eligibility |
| 4 | eligibility | `evaluate_requirement_eligibility` | `src/mechanism/evidence.rs` | hypothesis, bound, preparation, config | `Vec<EligibleRequirementEvidence>` | `MechanismAssessmentError` | direct contradiction |
| 5 | direct contradiction | `evaluate_direct_contradictions` | `src/mechanism/evidence.rs` | hypothesis, eligible, bundle | summaries | `MechanismAssessmentError` | timescale/gates |
| 6 | gates | existing exact evaluator APIs | their controlling modules | eligible evidence and temporal results | gate assessments | their typed errors | promotion |
| 7 | promotion | `assess_hypothesis` | `src/mechanism/promotion.rs` | all resolved assessments | hypothesis assessment | `MechanismAssessmentError` | components |
| 8 | components/history/assembly | existing exact APIs in §26.8 | controlling modules | resolved assessment and prior report | schema-4 report | inventoried typed errors | serialization |

There is no pipeline stage without an exact API, and no direct contradiction
stage before temporal eligibility.

### 27.9 Supersession audit and regression preservation

The complete-plan search audit classifies every conflicting occurrence as
follows. Only the definitions in §27 are active for the four findings:

| searched term or old definition | classification |
|---|---|
| `evaluate_temporal_join(left, right, bundle, metadata, config)` | SUPERSEDED; the sole active signature takes `&TemporalJoinRequest` |
| `TemporalJoinAssessment` without `requirement_id` or persisted `mode` | SUPERSEDED |
| `TemporalJoinAssessment.join_mode` | SUPERSEDED by `mode` |
| implicit support-variant mode inference | SUPERSEDED and forbidden |
| `EvidenceTemporalSupport` without an internally tagged wire shape | SUPERSEDED |
| vector temporal catalog | SUPERSEDED; the sole active catalog is the top-level EvidenceId-keyed map |
| `evaluate_direct_contradictions(..., &BoundHypothesisEvidence, ...)` | SUPERSEDED |
| direct contradiction over raw/unfiltered bundle candidates | SUPERSEDED and forbidden |
| `PB-FX-09`/`PB-FX-10` in §§21--26 | SUPERSEDED fixture fragments; §27.7 is sole active definition |
| `b-artifact-*` serialized fixture IDs | SUPERSEDED; runtime-derived ArtifactId variables are required |
| `TemporalAssessmentPolicy`, `TemporalClockBasis`, `PhaseBHypothesisHistory`, `CriticalEvidence`, old role-bearing bindings, and old generic status types | SUPERSEDED or migration-only exactly as §26.10 states |

The following previously passed areas remain PASS without redesign: role/gate/
stage matrix; validation-only requirement semantics; signed-relative-error
amplitude mathematics and inclusive boundaries; `critical_requirement_ids`;
the 4×4 `InterpretationStatus` matrix; ArtifactId fixture-generation policy;
wire field names; controlling type inventory; error ownership; frozen A1
compatibility; hypothesis ownership; quantity semantics; direct
`EvidenceBundle` retirement; EIS temporal `Unknown`; repeatability;
identifiability applicability; schema 3→4; validation role ownership;
traceability; critical contradiction pipeline; TOML root; CLI; history.

The following questions have one required answer: **NO**. Could two competent
implementers differ about where `TemporalJoinMode` comes from; whether runtime
support overrides configured mode; mode/support mismatch behavior; whether
eligibility precedes contradiction; which contradictory records are eligible;
the support enum serde representation; catalog representation; PB-FX-09
TOML/intermediate/final report; PB-FX-10 TOML/validation/intermediate/final
report; or any field-to-type mapping? No.

### 27.10 Final contract self-audit

```text
Undefined normative types = 0
Undefined normative owners = 0
Unspecified Phase B algorithms = 0
Unspecified scientific thresholds/units = 0
Unspecified compatibility decisions = 0
Normative contradictions = 0
Fixture-to-real-schema contradictions = 0
Incomplete normative positive fixtures = 0
Unmapped active normative types = 0
Pipeline stages without exact API = 0
Serialized active types without exact wire shape = 0
Implementation invention still required = no
Production Rust modified = NONE
Tests modified = NONE
Fixtures modified = NONE
Frozen A1 compatibility = YES
Main unchanged = YES
Phase B implementation branch unchanged = YES
```

### 27.11 Acceptance and round-trip contract

The canonical acceptance contract is:

```text
canonical TOML
→ deserialize MechanismEvidenceConfig
→ serialize through the supported production config path
→ reload
→ semantically identical configuration

source artifacts
→ public production reader
→ expected A1 records and runtime-derived ArtifactIds

schema-4 expected report
→ public writer
→ public reader
→ identical Phase-B assessment, history, validation, component, and lineage fields
```

The field-by-field serde audit is closed: every field in both standalone
TOML documents maps to the Rust field/type named in §26.6/§27.7; every active
enum uses its declared scalar or `type`/`kind` representation; no unknown or
missing required field is accepted; all ArtifactIds are runtime-derived and
validated; the temporal catalog has one exact map shape; and both fixture
reports contain every Phase-B-added field. Only the planning document may be
changed by this remediation.

READY_FOR_PHASE_B_FINAL_WIRE_REREVIEW = yes

## 28. Phase B Contract Remediation X — canonical production pipeline and eligible gate inputs

### 28.1 Finding reconciliation and authority

The remaining independent-review finding is **CONFIRMED**. The controlling
production/API contract was inconsistent: the prior exact-order lists omitted
preparation, the later gates and report stages were only indirectly named,
the prose required temporally eligible evidence, and retained gate signatures
still accepted `BoundHypothesisEvidence`.

This section is the sole active normative contract for Phase B evidence state,
scientific-gate inputs, promotion wiring, and the complete production order.
It supersedes every earlier Phase-B production-order definition, API
signature, gate input list, fixture API label, type-inventory status, and
readiness statement in §§6--10, 14--18, and 20--27 wherever this section
differs. In particular, the §26.9 table and the §27.8 eight-row table are
SUPERSEDED; their stages and APIs are not additive. Earlier material is
historical, descriptive, or migration-only. The frozen A1 interfaces,
serialized A1 meaning, A1 semantic hashes, legacy mechanism behavior,
`main`, and `codex/mhi-v1-b-mechanism-evidence-integration` remain unchanged.
This is a documentation-only amendment and authorizes no production Rust,
test, fixture, `main`, or A1 change.

### 28.2 One authoritative evidence-state model

Phase B has exactly three evidence states. They are distinct types with a
one-way dataflow:

```text
EvidenceBundle
  → bind_hypothesis_evidence
  → BoundHypothesisEvidence
  → evaluate_hypothesis_evidence_eligibility
  → EligibleHypothesisEvidence
  → scientific gates and promotion
```

`EvidenceBundle` is the neutral A1 evidence state. It owns A1
`EvidenceRecord`s, pairwise independence, pair covariance, scope, lineage,
and warnings. It does not contain a hypothesis-specific candidate decision.

`BoundHypothesisEvidence` is structural only. Its sole purpose is:

```text
EvidenceBundle + MechanismHypothesisDefinition
  → BoundHypothesisEvidence
```

The active internal shape is:

```rust
pub struct BoundHypothesisEvidence {
    pub hypothesis_id: MechanismHypothesisId,
    pub requirements: Vec<BoundRequirementEvidence>,
    pub pair_bindings: Vec<BoundEvidencePair>,
    pub role_bindings: Vec<MechanismEvidenceRoleBinding>,
}

pub struct BoundRequirementEvidence {
    pub requirement_id: EvidenceRequirementId,
    pub candidate_evidence_ids: Vec<EvidenceId>,
}
```

Binding selects deterministic per-requirement candidate IDs by exact target,
source class, quantity semantic/unit, pair selector, and structural
requirement binding. It validates exact pair orientation and role/stage tuple
shape. It does **not** imply temporal eligibility, support eligibility,
contradiction eligibility, final strength eligibility, or validation-role
eligibility. No scientific evaluator may search the bundle or choose a raw
candidate after this stage.

`EligibleHypothesisEvidence` is the one canonical aggregate passed to
scientific gates and promotion. Its exact active shape is:

```rust
pub struct EligibleHypothesisEvidence {
    pub hypothesis_id: MechanismHypothesisId,
    pub requirements: Vec<EligibleRequirementEvidence>,
}

pub struct EligibleRequirementEvidence {
    pub requirement_id: EvidenceRequirementId,
    pub support_evidence_ids: Vec<EvidenceId>,
    pub contradictory_evidence_ids: Vec<EvidenceId>,
    pub temporally_ineligible_evidence_ids: Vec<EvidenceId>,
    pub indeterminate_evidence_ids: Vec<EvidenceId>,
    pub temporal_assessments: Vec<TemporalJoinAssessment>,
}
```

The aggregate is complete and sorted by requirement ID. Eligibility owns
availability, validity, experiment/sensor/channel scope, quantity
compatibility, serialized temporal requests, temporal joins, direction
separation, and the minimum structural strength filtering required before a
downstream gate. An `Eligible` temporal result is the only route into an
eligible ID vector. `Ineligible` and `Indeterminate` IDs remain in their
dedicated vectors and cannot be counted by a gate. Validation-role selection
uses the exact eligible requirement slice together with the declared role
bindings; it never resurrects an ID from `BoundHypothesisEvidence`.

The one canonical eligibility API is:

```rust
pub fn evaluate_hypothesis_evidence_eligibility(
    hypothesis: &MechanismHypothesisDefinition,
    bound: &BoundHypothesisEvidence,
    preparation: &PhaseBEvidencePreparation,
    config: &MechanismEvidenceConfig,
) -> Result<EligibleHypothesisEvidence, MechanismAssessmentError>;
```

This is the only generic validity/scope/temporal/direction filter. It calls
the §27 `build_temporal_join_request` and `evaluate_temporal_join` helpers
internally when a pair requirement is temporal, preserving the configured
`TemporalJoinMode`; those helpers are not additional production-order stages.
The eligibility result is the only input on which the direct contradiction
and scientific gate APIs may operate.

### 28.3 Exact post-eligibility gate APIs

All scientific Phase B evaluators consume `EligibleHypothesisEvidence` or an
exact per-requirement slice derived from it. No post-eligibility scientific
API accepts `BoundHypothesisEvidence`.

Direct contradictions use exactly:

```rust
pub fn evaluate_direct_contradictions(
    hypothesis: &MechanismHypothesisDefinition,
    eligible: &EligibleHypothesisEvidence,
    bundle: &EvidenceBundle,
) -> Result<Vec<RequirementContradictionSummary>, MechanismAssessmentError>;
```

The evaluator reads only `eligible.requirements[*].contradictory_evidence_ids`,
resolves those IDs in the supplied neutral bundle, and emits sorted summaries.
It never reruns temporal matching, reads bound candidate vectors, or searches
an unfiltered bundle. A critical contradiction remains exactly an eligible
direct `Contradicts` record for an ID in `critical_requirement_ids` with
strength at least `Strong`; amplitude contradictions remain outside this
direct path.

The exact scientific gate signatures are:

```rust
// src/mechanism/timescale.rs
pub fn evaluate_timescale_requirement(
    hypothesis: &MechanismHypothesisDefinition,
    requirement: &EvidencePairRequirement,
    eligible_pair: (&EligibleRequirementEvidence, &EligibleRequirementEvidence),
    bundle: &EvidenceBundle,
    config: &TimescaleEvidenceConfig,
) -> Result<TimescaleAssessment, MechanismAssessmentError>;

// src/mechanism/amplitude.rs
pub fn evaluate_amplitude_requirement(
    hypothesis: &MechanismHypothesisDefinition,
    gate: &AmplitudeGate,
    eligible: &EligibleRequirementEvidence,
    bundle: &EvidenceBundle,
    config: &AmplitudeEvidenceConfig,
) -> Result<AmplitudeAssessment, AmplitudeAssessmentError>;

// src/mechanism/repeatability.rs
pub fn evaluate_repeatability_requirement(
    hypothesis: &MechanismHypothesisDefinition,
    gate: &RepeatabilityGate,
    eligible: &EligibleRequirementEvidence,
    bundle: &EvidenceBundle,
    config: &RepeatabilityEvidenceConfig,
) -> Result<RepeatabilityAssessment, RepeatabilityAssessmentError>;

// src/mechanism/identifiability.rs
pub fn evaluate_identifiability_binding(
    hypothesis: &MechanismHypothesisDefinition,
    binding: &IdentifiabilityBinding,
    eligible: &EligibleHypothesisEvidence,
    bundle: &EvidenceBundle,
    pairwise_independence: &[EvidenceIndependenceAssessment],
    config: &IdentifiabilityGateConfig,
) -> Result<IdentifiabilityAssessment, IdentifiabilityAssessmentError>;

// src/mechanism/validation.rs
pub fn evaluate_validation_protocol(
    hypothesis: &MechanismHypothesisDefinition,
    eligible: &EligibleHypothesisEvidence,
    role_bindings: &[MechanismEvidenceRoleBinding],
    bundle: &EvidenceBundle,
    protocol: Option<&ValidationProtocol>,
) -> Result<ValidationAssessment, ValidationAssessmentError>;
```

The timescale evaluator receives the exact two eligible requirement slices
named by `EvidencePairRequirement`; it may not derive a pair from raw bound
IDs or record order. Amplitude and repeatability receive the exact eligible
slice for their named requirement. Identifiability resolves every
`IdentifiabilityInputBinding.requirement_ids` against the corresponding
eligible slices in the aggregate and uses the supplied A1 independence
assessments/bundle covariance only as required by the already-approved
algorithm. Validation intersects declared `Validation` role bindings only
with eligible IDs in the aggregate, then applies the existing protocol and
family rules. A `None` protocol is `NotApplicable` only when the hypothesis
declares validation not applicable; it cannot turn an absent protocol into a
pass. No gate performs generic availability, validity, scope, quantity, or
temporal filtering a second time.

### 28.4 Explicit promotion aggregate and downstream APIs

All gate results and contradiction results pass through one explicit
aggregate:

```rust
pub struct HypothesisGateAssessments {
    pub contradiction_summaries: Vec<RequirementContradictionSummary>,
    pub timescale_assessments: Vec<TimescaleAssessment>,
    pub amplitude_assessments: Vec<AmplitudeAssessment>,
    pub repeatability_assessments: Vec<RepeatabilityAssessment>,
    pub identifiability_assessments: Vec<IdentifiabilityAssessment>,
    pub validation_assessment: Option<ValidationAssessment>,
}

pub fn assess_hypothesis(
    hypothesis: &MechanismHypothesisDefinition,
    eligible: &EligibleHypothesisEvidence,
    gates: &HypothesisGateAssessments,
    config: &MechanismEvidenceConfig,
) -> Result<PhaseBHypothesisAssessment, MechanismAssessmentError>;
```

`assess_hypothesis` consumes the aggregate and the eligible evidence only.
It does not call a scientific evaluator, recompute a gate, inspect raw bound
IDs, or read global state. Temporal assessments are already carried by the
eligible requirement slices and are copied into the Phase B assessment. A
missing or failed gate result remains a typed assessment outcome and cannot
be silently replaced by promotion logic.

The complete downstream signatures are:

```rust
// src/mechanism/promotion.rs
pub fn assess_components(
    hypothesis: &MechanismHypothesisDefinition,
    hypothesis_assessment: &PhaseBHypothesisAssessment,
    prior_component_statuses: &BTreeMap<ComponentId, InterpretationStatus>,
) -> Result<Vec<ComponentInterpretationAssessment>, ComponentAssessmentError>;

// src/mechanism/history.rs
pub fn update_hypothesis_history(
    previous_history: &[HypothesisHistoryEntry],
    current_assessment: &PhaseBHypothesisAssessment,
    component_assessments: &[ComponentInterpretationAssessment],
) -> Result<Vec<HypothesisHistoryEntry>, MechanismAssessmentError>;

// src/runners/mechanism.rs
pub fn assemble_phase_b_mechanism_report(
    legacy_current_mechanism_payload: &MechanismAnalysisReport,
    hypothesis_assessments: &[PhaseBHypothesisAssessment],
    component_assessments: &[ComponentInterpretationAssessment],
    history: &[HypothesisHistoryEntry],
    lineage: &ArtifactLineageState,
    direct_dependencies: &[ArtifactDependency],
) -> Result<MechanismAnalysisReport, PhaseBReportAssemblyError>;

// src/domain/artifact.rs
pub fn write_artifact<T: VersionedArtifact>(
    path: &Path,
    value: &T,
) -> Result<(), ArtifactError>;
```

`assess_components` applies the already-approved 4×4 monotonic status matrix
using the supplied prior status map. `update_hypothesis_history` is the sole
history transition owner and receives the component rows explicitly.
`assemble_phase_b_mechanism_report` is the sole schema-4 report assembler;
its legacy/current payload preserves the existing legacy fields, while the
explicit hypothesis, component, history, lineage, and dependency arguments
populate the Phase B fields. `write_artifact::<MechanismAnalysisReport>` is
the final canonical public writer. No component, history, report, or writer
stage is hidden behind prose.

### 28.5 Sole authoritative 18-stage production pipeline

The exact semantic production order is:

```text
1.  parse CLI
2.  load MechanismEvidenceConfig
3.  load versioned source artifacts through public readers
4.  construct PhaseBEvidencePreparationInputs
5.  prepare_phase_b_evidence → PhaseBEvidencePreparation { bundle, temporal_metadata }
6.  for each hypothesis: bind_hypothesis_evidence → BoundHypothesisEvidence
7.  evaluate_hypothesis_evidence_eligibility → EligibleHypothesisEvidence
8.  evaluate_direct_contradictions
9.  evaluate timescale requirements from eligible evidence
10.  evaluate amplitude requirements from eligible evidence
11. evaluate repeatability requirements from eligible evidence
12. evaluate identifiability bindings from eligible evidence
13. evaluate validation protocol from eligible Validation-role evidence
14. assess_hypothesis using all gate assessments and contradiction summaries
    → PhaseBHypothesisAssessment
15. assess_components → Vec<ComponentInterpretationAssessment>
16. update_hypothesis_history
17. assemble_phase_b_mechanism_report → MechanismAnalysisReport schema 4
18. serialize/write through the canonical mechanism artifact writer
```

Preparation is explicitly before binding. Temporal request construction and
temporal joins are internal operations owned by stage 7; they are not omitted
and they are not competing top-level stages. Eligibility is complete before
contradictions, every scientific gate consumes eligible evidence, and
validation uses only eligible records bound to the Validation role.

The sole complete production API table is:

| Stage | Function | Owner module | Input types | Output type | Error type | Consumes Bound or Eligible? | Next stage |
|---:|---|---|---|---|---|---|---|
| 1 | `parse_cli_args(&[String])` | `src/cli.rs` | process argument vector | `CliArgs` | `CliError` | neither | 2 |
| 2 | `load_mechanism_evidence_config(&Path)` | `src/mechanism/config.rs` | Phase B config path from `CliArgs` | `MechanismEvidenceConfig` | `MechanismEvidenceConfigError` | neither | 3 |
| 3 | `load_phase_b_source_artifacts(&PhaseBSourceArtifactRefs)` | `src/runners/mechanism.rs`, using `src/domain/artifact.rs::read_artifact` | versioned source paths/direct-bundle reference | `EvidenceBundleInputs` | `MechanismEvidenceInputError` / public reader `ArtifactError` | neither | 4 |
| 4 | `construct_phase_b_evidence_preparation_inputs(EvidenceBundleInputs)` | `src/mechanism/preparation.rs` | `EvidenceBundleInputs` | `PhaseBEvidencePreparationInputs` | infallible construction; no error | neither | 5 |
| 5 | `prepare_phase_b_evidence(PhaseBEvidencePreparationInputs)` | `src/mechanism/preparation.rs` | preparation inputs and public-reader outputs | `PhaseBEvidencePreparation` | `PhaseBEvidencePreparationError` | neither | 6 |
| 6 | `bind_hypothesis_evidence(&MechanismHypothesisDefinition, &PhaseBEvidencePreparation)` | `src/mechanism/evidence.rs` | one hypothesis and preparation | `BoundHypothesisEvidence` | `EvidenceBindingError` | produces Bound; no Eligible input | 7 |
| 7 | `evaluate_hypothesis_evidence_eligibility(&MechanismHypothesisDefinition, &BoundHypothesisEvidence, &PhaseBEvidencePreparation, &MechanismEvidenceConfig)` | `src/mechanism/evidence.rs` | hypothesis, structural bound evidence, preparation, config | `EligibleHypothesisEvidence` | `MechanismAssessmentError` | Bound input; produces canonical Eligible | 8 |
| 8 | `evaluate_direct_contradictions(&MechanismHypothesisDefinition, &EligibleHypothesisEvidence, &EvidenceBundle)` | `src/mechanism/evidence.rs` | hypothesis, eligible aggregate, neutral bundle | `Vec<RequirementContradictionSummary>` | `MechanismAssessmentError` | Eligible | 9 |
| 9 | `evaluate_timescale_requirement` once per declared pair gate | `src/mechanism/timescale.rs` | hypothesis, `EvidencePairRequirement`, exact eligible pair `(&EligibleRequirementEvidence, &EligibleRequirementEvidence)`, bundle, `TimescaleEvidenceConfig` | `TimescaleAssessment` | `MechanismAssessmentError` | Eligible | 10 |
| 10 | `evaluate_amplitude_requirement` once per declared amplitude gate | `src/mechanism/amplitude.rs` | hypothesis, `AmplitudeGate`, exact eligible requirement slice, bundle, `AmplitudeEvidenceConfig` | `AmplitudeAssessment` | `AmplitudeAssessmentError` | Eligible | 11 |
| 11 | `evaluate_repeatability_requirement` once per declared repeatability gate | `src/mechanism/repeatability.rs` | hypothesis, `RepeatabilityGate`, exact eligible requirement slice, bundle, `RepeatabilityEvidenceConfig` | `RepeatabilityAssessment` | `RepeatabilityAssessmentError` | Eligible | 12 |
| 12 | `evaluate_identifiability_binding` once per declared binding | `src/mechanism/identifiability.rs` | hypothesis, `IdentifiabilityBinding`, eligible aggregate, bundle, A1 independence assessments, `IdentifiabilityGateConfig` | `IdentifiabilityAssessment` | `IdentifiabilityAssessmentError` | Eligible | 13 |
| 13 | `evaluate_validation_protocol` once per hypothesis | `src/mechanism/validation.rs` | hypothesis, eligible aggregate, role bindings, bundle, optional `ValidationProtocol` | `ValidationAssessment` | `ValidationAssessmentError` | Eligible Validation-role evidence | 14 |
| 14 | `assess_hypothesis` | `src/mechanism/promotion.rs` | hypothesis, eligible aggregate, `HypothesisGateAssessments`, `MechanismEvidenceConfig` | `PhaseBHypothesisAssessment` | `MechanismAssessmentError` | Eligible-derived results | 15 |
| 15 | `assess_components` | `src/mechanism/promotion.rs` | hypothesis, current `PhaseBHypothesisAssessment`, prior component status map | `Vec<ComponentInterpretationAssessment>` | `ComponentAssessmentError` | Eligible-derived assessment; no raw Bound | 16 |
| 16 | `update_hypothesis_history` | `src/mechanism/history.rs` | previous history, current assessment, component assessments | updated `Vec<HypothesisHistoryEntry>` | `MechanismAssessmentError` | Eligible-derived assessment; no raw Bound | 17 |
| 17 | `assemble_phase_b_mechanism_report` | `src/runners/mechanism.rs` | legacy/current mechanism payload, hypothesis assessments, component assessments, history, lineage, direct dependencies | `MechanismAnalysisReport` schema 4 | `PhaseBReportAssemblyError` | Eligible-derived outputs; no raw Bound | 18 |
| 18 | `write_artifact::<MechanismAnalysisReport>(&Path, &MechanismAnalysisReport)` | `src/domain/artifact.rs` | schema-4 report and output path | persisted mechanism artifact | `ArtifactError` | neither; report only | done |

The table has exactly 18 stages, all 18 have an exact function or API owner,
and zero stages are delegated to an indirect “remaining gates” description.
Stages 9--13 invoke the named function once for each declared gate or
binding; their exact signatures and all input types are listed in §28.3, so
no API or stage is omitted.

### 28.6 PB-FX-09/PB-FX-10 final stage mapping

The scientific values, source artifacts, runtime-derived ArtifactIds, wire
shapes, and expected assessment values in the canonical fixtures remain
unchanged. The API-stage labels in the earlier fixture blocks are
SUPERSEDED by this mapping and must not retain the old
`BoundHypothesisEvidence` gate signatures.

| final pipeline stage | PB-FX-09 mapping | PB-FX-10 mapping |
|---|---|---|
| prepare | `prepare_phase_b_evidence` returns the two source-derived records and temporal catalog | same preparation returns all four source-derived records and catalog |
| bind | `bind_hypothesis_evidence` binds the two support requirements and exact pair | same plus two validation-role requirement bindings |
| eligibility | `evaluate_hypothesis_evidence_eligibility` returns eligible support slices; no temporal request for the NotApplicable pair | same; validation slices are eligible but do not block support promotion |
| contradiction | `evaluate_direct_contradictions` consumes the eligible aggregate and returns `[]` | same; critical IDs are empty and returns `[]` |
| timescale | `evaluate_timescale_requirement` consumes the exact eligible pair and returns the approved satisfied result | same |
| amplitude | `evaluate_amplitude_requirement` consumes eligible amplitude slices; no gate, `[]` | same |
| repeatability | `evaluate_repeatability_requirement` consumes eligible repeatability slices; no gate, `[]` | same |
| identifiability | `evaluate_identifiability_binding` consumes the named eligible pair and returns the approved satisfied result | same |
| validation | `evaluate_validation_protocol` consumes no protocol/eligible Validation role and returns `NotApplicable` | consumes eligible Validation-role slices and returns the approved satisfied result |
| hypothesis | `assess_hypothesis` consumes one `HypothesisGateAssessments` aggregate and yields `ExperimentallySupported` | same aggregate path yields `ValidatedForDomain` |
| components | `assess_components` yields the two approved component rows | yields the four approved component rows |
| history | `update_hypothesis_history` receives component rows and yields empty history without a prior report | receives component rows and appends the approved single transition |
| report | `assemble_phase_b_mechanism_report` produces schema 4 with the approved PB-FX-09 fields | same schema-4 assembly with approved validation/component/history fields |

PB-FX-09 and PB-FX-10 therefore both map to
`prepare → bind → eligibility → contradiction → timescale → amplitude →
repeatability → identifiability → validation → hypothesis → components →
history → report`, followed by canonical serialization. Neither fixture
references a scientific gate taking `BoundHypothesisEvidence`.

### 28.7 Controlling type and error inventory closure

The §26.7 inventory remains exhaustive after removing its superseded rows and
gains exactly these active Phase B rows:

| type | kind | serialized? | owner module | owning parent | schema | purpose | wire representation | supersedes |
|---|---|---:|---|---|---|---|---|---|
| `EligibleHypothesisEvidence` | struct | no | `src/mechanism/evidence.rs` | eligibility output/promotion input | n/a | canonical aggregate of eligible requirement evidence | internal typed aggregate | gates selecting raw bound candidates |
| `HypothesisGateAssessments` | struct | no | `src/mechanism/promotion.rs` | `assess_hypothesis` input | n/a | explicit contradiction and gate-result aggregate | internal typed aggregate | variadic/hidden promotion wiring |
| `MechanismEvidenceConfigError` | error enum | no | `src/mechanism/config.rs` | config loading | n/a | typed Phase B config parse/validation failures | internal typed error | implicit/defaulted B config |
| `MechanismEvidenceInputError` | error enum | no | `src/runners/mechanism.rs` | source loading | n/a | typed source mode/kind/schema/scope failures | internal typed error | untyped source-loading failures |

No new scientific threshold, wire field, A1 type, or error category is
introduced by this remediation. `CliArgs`, `CliError`, `EvidenceBundleInputs`,
`ArtifactLineageState`, `ArtifactDependency`, `MechanismAnalysisReport`, and
`ArtifactError` are existing repository-native boundary types; they are named
explicitly in the 18-stage table and are not duplicate Phase B definitions.
Every active type declared by this section has one inventory row or is one of
those existing/frozen repository-native boundary types.

### 28.8 Full supersession audit

The complete-plan search for the controlling terms has one active normative
interpretation:

| occurrence/class | classification | controlling rule |
|---|---|---|
| `BoundHypothesisEvidence` in `bind_hypothesis_evidence` output | ACTIVE NORMATIVE | structural binding result only |
| `BoundHypothesisEvidence` in `evaluate_hypothesis_evidence_eligibility` input | ACTIVE NORMATIVE | only allowed post-binding input; generic eligibility produces the canonical eligible state |
| `BoundHypothesisEvidence` in any timescale, amplitude, repeatability, identifiability, validation, contradiction, promotion, component, history, or report API | SUPERSEDED | no scientific gate or downstream stage may consume raw Bound evidence |
| `EligibleHypothesisEvidence` definition and eligibility output | ACTIVE NORMATIVE | sole canonical scientific-gate aggregate |
| `EligibleRequirementEvidence` definition and exact per-requirement gate slices | ACTIVE NORMATIVE | sole eligible slice type for requirement-specific gates |
| old `evaluate_requirement_eligibility` name/signature | SUPERSEDED | replaced by `evaluate_hypothesis_evidence_eligibility` |
| old `evaluate_timescale_requirement` signature accepting `BoundHypothesisEvidence` | SUPERSEDED | §28.3 exact eligible-pair signature |
| old amplitude/repeatability/identifiability/validation signatures accepting Bound evidence | SUPERSEDED | §28.3 exact Eligible signatures |
| old `evaluate_direct_contradictions(..., &BoundHypothesisEvidence, ...)` | SUPERSEDED | §28.3 exact `EligibleHypothesisEvidence` signature |
| old `assess_hypothesis` signature with raw Bound evidence or separate hidden gate parameters | SUPERSEDED | §28.4 explicit `HypothesisGateAssessments` signature |
| every earlier “exact production order” or “production pipeline” list | SUPERSEDED | §28.5 sole 18-stage order and table |
| “remaining gates,” “later stages,” or indirect component/history/report prose | SUPERSEDED | §28.5 rows 9--18 list each stage and API explicitly |
| old PB-FX-09/PB-FX-10 API labels | SUPERSEDED | §28.6 final stage mapping; scientific values remain frozen |

The audit counts are: one active normative production order, one active
normative production API table, zero active scientific gate APIs consuming
`BoundHypothesisEvidence` after eligibility, and zero omitted stage APIs.

### 28.9 Regression and compatibility audit

The following previously passed contract areas remain **PASS** without
redesign: `TemporalJoinRequest`/mode; temporal support matrix; temporal
serde; contradiction eligibility; role/stage/gate semantics;
validation-only semantics; amplitude equation and boundaries;
`critical_requirement_ids`; the 4×4 `InterpretationStatus` matrix;
`ArtifactId` fixture policy; PB-FX-09/10 canonical fixture values and wire
formats; hypothesis ownership; quantity semantics; direct `EvidenceBundle`
retirement; EIS temporal `Unknown`; repeatability; identifiability
applicability; schema 3→4; validation-role ownership; traceability path;
critical contradiction pipeline; TOML root; CLI; history; legacy
`EvidenceBundle` compatibility; and frozen A1 compatibility.

Phase B remains implementable without changing the serialized meaning of
`EvidenceRecord`, `EvidenceQuantity`, `EvidencePairKey`,
`ArtifactLineageState`, `TimescalePairUncertainty`, or A1 semantic identity:
**PASS**.

### 28.10 Pipeline completeness and contract self-audit

```text
Pipeline stages required = 18
Pipeline stages with exact API/owner = 18
Pipeline stages missing exact API = 0
Undefined normative types = 0
Undefined normative owners = 0
Unspecified Phase B algorithms = 0
Unspecified scientific thresholds/units = 0
Unspecified compatibility decisions = 0
Normative contradictions = 0
Fixture-to-real-schema contradictions = 0
Incomplete normative positive fixtures = 0
Unmapped active normative types = 0
Pipeline stages without exact API = 0
Serialized active types without exact wire shape = 0
Competing active production-order definitions = 0
Scientific gate APIs consuming BoundHypothesisEvidence after eligibility = 0
Implementation invention still required = no
Production Rust modified = NONE
Tests modified = NONE
Fixtures modified = NONE
```

Could two competent implementation agents differ about whether preparation
happens before binding? **NO.** Could they differ about whether temporal
eligibility happens before contradictions? **NO.** Could they differ about
whether timescale, amplitude, repeatability, identifiability, or validation
consume Bound or Eligible evidence? **NO.** Could they differ about which
evidence identifiability sees? **NO.** Could they differ about which evidence
validation sees? **NO.** Could they differ about what object promotion
consumes? **NO.** Could they differ about where component assessment occurs?
**NO.** Could they differ about where history is updated? **NO.** Could they
differ about where schema-4 report assembly occurs? **NO.**

The implementation agent must not infer an omitted step, invent a competing
gate input, refilter generic eligibility, or modify any previously passed
scientific, wire, compatibility, fixture, or A1 contract area.

READY_FOR_PHASE_B_PIPELINE_REREVIEW = yes

## 29. Phase B Contract Remediation XI — final source-route compatibility

### 29.1 Authority and finding verification

This section is the final and sole active Phase B V1 source-route contract. It
supersedes every earlier Phase-B statement in §§20--28, and every earlier
source-route, temporal-source, CLI, fixture, test, API-table, and readiness
statement, **only** where this section addresses the two source-route
contradictions resolved here. Earlier occurrences that differ are historical,
descriptive, future/non-V1, or superseded as classified by §29.9. Frozen A1
behavior, serialized meaning, semantic hashes, legacy mechanism behavior,
`main`, and `codex/mhi-v1-b-mechanism-evidence-integration` remain unchanged.
This is a documentation-only amendment and authorizes no production Rust,
test, or fixture change.

The frozen-tree verification is:

| finding | classification | repository evidence | final V1 disposition |
|---|---|---|---|
| `EvidenceBundleInputs` fields | CONFIRMED | `src/runners/evidence.rs::EvidenceBundleInputs` contains `calibration_model`, `transient`, `estimation`, `eis_fit`, and `calibration_observations`; it has no model field. | The exact existing A1 fields are frozen. |
| `assemble_evidence_bundle(...)` | CONFIRMED | It calls the transient, estimation, calibration-observation, and EIS adapters; it does not call `adapt_model_analysis`. | Phase B reuses it unchanged. |
| `adapt_model_analysis(...)` | CONFIRMED | `src/evidence_adapters.rs` exposes a standalone adapter for `ModelAnalysisReport`. | It remains available but is outside the Phase B V1 production path. |
| `ModelAnalysisReport` artifact contract | CONFIRMED | `src/results/model.rs` defines the report, and `ArtifactKind::ModelAnalysis` is readable as a public artifact type. | Public readability does not make it a Phase B V1 source. |
| `EvidenceBundle` artifact status | CONFIRMED | `EvidenceBundle` is returned by `assemble_evidence_bundle(...)` and is not a `VersionedArtifact`; it has no public artifact envelope/reader contract. | It is internal assembled evidence only, never an external Phase B V1 input. |

The final compatibility rule is therefore: Phase B V1 adds no model route and
no direct-bundle route. It loads only source artifacts already represented by
the frozen A1 assembly boundary, then creates one in-memory `EvidenceBundle`.

### 29.2 Authoritative ModelAnalysisReport policy

`ModelAnalysisReport` is **NOT** a Phase B V1 production source artifact.
`adapt_model_analysis(...)` may remain in `src/evidence_adapters.rs` for
future additive work, but it is not called by Phase B V1 source loading,
preparation, or any production pipeline stage. Phase B V1 MUST NOT:

1. add a model field to frozen `EvidenceBundleInputs`;
2. change `assemble_evidence_bundle(...)` to consume model reports;
3. create a Phase-B-private model evidence path that bypasses A1 assembly; or
4. fabricate model evidence, timestamps, equilibrium classifications, or
   `EvidenceId`s into the bundle.

Model evidence, model-derived temporal metadata, model-source fixture rows, and
model-source acceptance criteria are **OUT OF SCOPE FOR PHASE B V1**. A future
phase may define an additive versioned contract, but that future possibility is
not an active V1 requirement.

### 29.3 Authoritative direct EvidenceBundle policy

Phase B V1 does **NOT** accept a standalone `EvidenceBundle` file, path,
artifact ID, or public reference as CLI or production input. `EvidenceBundle`
exists only as the in-memory output of the canonical source preparation path:

```text
CLI source-artifact references
  → public VersionedArtifact readers
  → loaded EvidenceBundleInputs source objects
  → PhaseBEvidencePreparationInputs
  → frozen assemble_evidence_bundle(...)
  → in-memory EvidenceBundle
```

`PhaseBEvidencePreparationInputs` contains only the loaded source objects (or
their source-object container) needed by the frozen A1 assembly. It contains
no `EvidenceBundle`, `PathBuf` to a bundle, direct bundle artifact ID, bundle
reader result, or direct-bundle mode. A bundle received from a previous
preparation invocation is not a valid preparation input; preparation begins
with source artifacts and creates the bundle itself.

The `--evidence-artifact` flag and every spelling of direct-bundle mode are
**RETIRED FROM PHASE B V1**. They are not aliases and are not accepted by the
active Phase B CLI. The active CLI has only the source-artifact references in
§29.4. Any stale earlier table that lists `--evidence-artifact` as required,
optional, or mutually exclusive is SUPERSEDED and is not an implementation
requirement.

### 29.4 Phase B V1 accepted production source artifacts

The following is the sole canonical source table. “Accepted” means accepted as
an external Phase B V1 source artifact and loaded into the corresponding
frozen-A1 input field. The `calibration_model` row is included to distinguish
an existing A1 lineage/context field from an evidence-producing source.

| Source artifact type | Accepted in Phase B V1? | Frozen A1 `EvidenceBundleInputs` field | A1 adapter | Public reader | Temporal capability | Used by PB-FX-09? | Used by PB-FX-10? | Notes |
|---|---|---|---|---|---|---:|---:|---|
| `EisFitArtifact` / `eis_fit` | YES | `eis_fit` | `adapt_eis_fit` | `read_artifact::<EisFitArtifact>` | `Unknown`; no lawful timestamp/clock field | YES | YES | Records use `eis.parameter.{index}`; pair covariance comes only from the frozen labeled EIS covariance path. |
| `TransientAnalysisReport` / `transient_analysis` | YES | `transient` | `adapt_transient_analysis` | `read_artifact::<TransientAnalysisReport>` | `Window` only from the selected event’s finite, ordered `segment.fitted_time_local`; otherwise `Unknown` | YES | YES | Records include selected fit parameters and finite `tau_fast_s`/`tau_slow_s` derived fields. |
| `StateEstimationReport` / `state_estimation` | YES | `estimation` | `adapt_state_estimation` | `read_artifact::<StateEstimationReport>` | `Point` from finite `estimates[index].timestamp_s`; otherwise `Unknown`; equilibrium classification only from producer `equilibrium_assessment` | NO | YES | Records use `estimation.point.{point}.state.{state}`. |
| `CalibrationObservationSet` / `calibration_observations` | YES | `calibration_observations` | `try_adapt_calibration_observations` | `read_artifact::<CalibrationObservationSet>` | `Point` is not lawful for V1 because `timestamp` has no declared comparable clock; use `Unknown` | NO | YES | Records use `calibration.observation.{index}` and retain producer observation scope. |
| `StoredCalibrationModel` / `calibration_model` | NO as an evidence source | `calibration_model` (context/lineage only) | none | Public reader may load it for existing computational context | no Phase B evidence temporal support | NO | NO | Frozen A1 catalogs lineage context but does not adapt it into an `EvidenceRecord`. |
| `ModelAnalysisReport` / `ism_model_analysis` | NO | NONE | `adapt_model_analysis` exists but is not wired into this field or assembly | Public reader may read the artifact outside Phase B | NONE for Phase B V1 | NO | NO | Explicitly out of V1 production scope; no model fixture or model EvidenceId is legal. |
| `EvidenceBundle` | NO as an external source | NONE | none | no `VersionedArtifact` reader | NONE as an input | NO | NO | Internal output of A1 assembly only; no standalone bundle grammar. |

The accepted source set is therefore exactly `eis_fit`, `transient`,
`state_estimation`, and `calibration_observations`. Every accepted source has
one real frozen-A1 field, one real adapter, one public reader, and one
adapter-generated EvidenceId family. No accepted source is selected by path,
position, file timestamp, or fixture-supplied identity.

The active Phase B CLI source flags are exactly:

| flag | loaded type | A1 field | status |
|---|---|---|---|
| `--eis-artifact PATH` | `EisFitArtifact` | `eis_fit` | accepted |
| `--transient-artifact PATH` | `TransientAnalysisReport` | `transient` | accepted |
| `--estimation-artifact PATH` | `StateEstimationReport` | `estimation` | optional accepted source |
| `--calibration-observations-artifact PATH` | `CalibrationObservationSet` | `calibration_observations` | optional accepted source |
| `--evidence-artifact PATH` | `EvidenceBundle` | none | retired and rejected; never read |
| `--model-analysis PATH` / `--model-artifact PATH` | `ModelAnalysisReport` | none | out of scope and rejected for Phase B V1 |

The Phase B configuration flag and legacy mechanism flags retain the ownership
rules already frozen in §28; this table changes only the source-route details.

### 29.5 Canonical temporal source map

The active V1 temporal map uses only the accepted source set above:

| source artifact | adapter-generated EvidenceId | authoritative time field | classification/equilibrium field | temporal support | fallback |
|---|---|---|---|---|---|
| `EisFitArtifact` | `eis.parameter.{i}` | none | none | `Unknown` | `Unknown`; EIS cannot satisfy a required temporal join |
| `TransientAnalysisReport` | `transient.event.{i}.parameter.{j}`, `transient.event.{i}.tau_fast_s`, or `transient.event.{i}.tau_slow_s` | selected event `segment.fitted_time_local` | no producer equilibrium classification in this artifact | `Window` only when values are finite, ordered, and nonempty | `Unknown`; never infer a clock or equilibrium state |
| `StateEstimationReport` | `estimation.point.{i}.state.{j}` | `estimates[i].timestamp_s` | `estimates[i].equilibrium_assessment.classification` when present | `Point` only for a finite timestamp; clock is the producer timestamp basis | `Unknown` time; unavailable classification remains unavailable |
| `CalibrationObservationSet` | `calibration.observation.{i}` | `observations[i].timestamp` is not a comparable V1 clock | none | `Unknown` | `Unknown`; no fabricated point support |

No row for `ModelAnalysisReport` is active. No model timestamp, model
EvidenceId, model temporal support, or model equilibrium field participates in
Phase B V1. EIS remains `Unknown` exactly because no lawful timestamp exists.

### 29.6 Canonical stage 3 and unchanged 18-stage pipeline

The sole active 18-stage pipeline remains the §28 pipeline, with its source
details replaced by this section:

```text
1.  parse CLI
2.  load MechanismEvidenceConfig
3.  load ONLY accepted Phase B V1 versioned source artifacts through public readers
4.  construct PhaseBEvidencePreparationInputs from those loaded source objects
5.  prepare_phase_b_evidence
      → PhaseBEvidencePreparation { bundle, temporal_metadata }
6.  bind_hypothesis_evidence
7.  evaluate_hypothesis_evidence_eligibility
8.  evaluate_direct_contradictions
9.  timescale
10. amplitude
11. repeatability
12. identifiability
13. validation
14. assess_hypothesis
15. assess_components
16. update_hypothesis_history
17. assemble_phase_b_mechanism_report
18. serialize/write
```

Stage 3 loads only `EisFitArtifact`, `TransientAnalysisReport`,
`StateEstimationReport`, and `CalibrationObservationSet`, subject to the
optional-source rules in §29.4. It never loads `EvidenceBundle` or
`ModelAnalysisReport`. Stages 4 and 5 receive the actual loaded
`EvidenceBundleInputs` source objects and call/reuse frozen
`assemble_evidence_bundle(...)` unchanged. All stages 6--18 and their exact
APIs remain as specified in §28, except that any stale source-route wording is
classified by §29.9.

The affected production API rows are therefore:

| Stage | Exact active API contract |
|---:|---|
| 3 | `load_phase_b_source_artifacts(&PhaseBSourceArtifactRefs)` reads only the four accepted public artifact types and returns loaded source objects suitable for frozen `EvidenceBundleInputs`; direct bundle and model routes are rejected. |
| 4 | `construct_phase_b_evidence_preparation_inputs(EvidenceBundleInputs)` constructs `PhaseBEvidencePreparationInputs` containing source objects only; it does not accept a bundle. |
| 5 | `prepare_phase_b_evidence(PhaseBEvidencePreparationInputs)` invokes unchanged A1 assembly and returns the in-memory `EvidenceBundle` plus Phase-B temporal metadata derived only from accepted source fields. |

The other fifteen stages are unchanged. Thus the canonical stage count is 18,
the stage/API mapping is complete, and there is no competing production order.

### 29.7 PB-FX-09 and PB-FX-10 source regression

PB-FX-09 uses exactly two accepted source artifacts:

```text
EisFitArtifact → adapt_eis_fit → eis.parameter.0
TransientAnalysisReport → adapt_transient_analysis → transient.event.0.tau_fast_s
```

PB-FX-10 uses exactly four accepted source artifacts:

```text
EisFitArtifact → adapt_eis_fit → eis.parameter.0
TransientAnalysisReport → adapt_transient_analysis → transient.event.0.tau_fast_s
CalibrationObservationSet → try_adapt_calibration_observations → calibration.observation.0
StateEstimationReport → adapt_state_estimation → estimation.point.0.state.0
```

PB-FX-09 contains zero `ModelAnalysisReport` fixtures and zero standalone
`EvidenceBundle` inputs. PB-FX-10 contains zero `ModelAnalysisReport` fixtures
and zero standalone `EvidenceBundle` inputs. The PB-FX-10 validation rows are
the calibration-observation and state-estimation rows above, with their two
real source artifact IDs and two known acquisition families. The earlier
`b-validation-model` requirement, `model.point.0.component.0` role, and
`model_analysis_e2e_2.json` fixture are **SUPERSEDED and retired from Phase B
V1**; they are not replaced by a model-shaped placeholder.

Every PB-FX-09/PB-FX-10 EvidenceId is generated by the named adapter during
the actual frozen A1 assembly. Fixture authors provide complete real
versioned source artifacts and producer lineage only; they do not provide an
EvidenceBundle, EvidenceId, temporal metadata, direction, strength, role, or
model evidence record. Fixture-to-real-schema contradictions and
fixture-to-source-route contradictions are both zero.

### 29.8 Test and acceptance-contract correction

The mandatory Phase B V1 test plan contains no model-source preparation,
model-source temporal metadata, model evidence assembly, standalone bundle
reading, or direct-bundle CLI support. The exact former test name
`phase_b_point_temporal_join_uses_estimation_or_model_source` is **RETIRED
FROM PHASE B V1** because its model alternative is unsupported. Its supported
replacement is `phase_b_point_temporal_join_uses_estimation_source`, which
tests the same point-join behavior using `StateEstimationReport` only.

The retained canonical positive tests are:

```text
phase_b_e2e_experimentally_supported_from_sources
phase_b_e2e_validated_for_domain_from_sources
phase_b_eis_temporal_support_is_unknown
phase_b_point_temporal_join_uses_estimation_source
phase_b_amplitude_unit_threshold_and_direction
phase_b_repeatability_uses_sample_sd_and_independent_families
phase_b_schema3_to_schema4_preserves_legacy_hypotheses
phase_b_validation_counts_only_explicit_roles
```

Their source fixtures are limited to the accepted table in §29.4. No exact
mandatory test name remains mapped to a model source or direct bundle. Missing
exact test names = 0 and unmapped active acceptance criteria = 0.

### 29.9 Supersession audit and compatibility decision

The complete active-plan audit classifies all occurrences of the required
terms as follows:

| term/occurrence | classification after §29 | rule |
|---|---|---|
| `ModelAnalysisReport` in A1 covariance/schema inventories | DESCRIPTIVE or FUTURE/NON-V1 | The artifact remains a repository type, but is not a Phase B V1 source. |
| `ModelAnalysisReport` or `adapt_model_analysis` in prior Phase-B source, temporal, fixture, CLI, or acceptance text | SUPERSEDED | §29.2 and §29.4 control; active V1 count is zero. |
| `adapt_model_analysis` as a standalone repository adapter | OUT OF SCOPE / FUTURE | It remains callable only outside the Phase B V1 production path. |
| `model source`, `model evidence`, `model-derived`, `model point`, or `model temporal` in retained explanatory text | DESCRIPTIVE or OUT OF SCOPE / FUTURE | No retained occurrence is an active V1 source requirement. |
| `direct-bundle`, `direct bundle`, `EvidenceBundle input`, or `--evidence-artifact` in older Phase-B text | SUPERSEDED | Standalone/direct bundle input is retired and unsupported. |
| `EvidenceBundle` as the output of `assemble_evidence_bundle` or preparation | ACTIVE V1 INTERNAL OUTPUT | It is never an external source artifact. |

The authoritative compatibility decision is:

```text
PHASE B V1 SOURCE COMPATIBILITY DECISION
1. ModelAnalysisReport is NOT a Phase B V1 production evidence source.
2. adapt_model_analysis may remain in the repository but is outside the
   Phase B V1 production path.
3. Frozen EvidenceBundleInputs and assemble_evidence_bundle are not expanded
   for model evidence in Phase B V1.
4. Standalone/direct EvidenceBundle input is RETIRED and unsupported.
5. EvidenceBundle exists only as the in-memory result of canonical source
   assembly/preparation.
6. Phase B V1 fixtures, CLI, tests, temporal mappings, and acceptance criteria
   must use only the frozen-A1-supported source set.
```

Could two competent implementers differ about whether `ModelAnalysisReport` is
supported? **NO.** Could they differ about whether `adapt_model_analysis`
belongs in the Phase B V1 production route? **NO.** Could they differ about
whether `EvidenceBundleInputs` gains a model field? **NO.** Could they differ
about whether Phase B accepts a direct `EvidenceBundle` input? **NO.** Could
they differ about what stage 3 loads, how `EvidenceBundle` is created, or which
source types PB-FX-09/PB-FX-10 use? **NO** for every item.

### 29.10 Final remediation self-audit

```text
Undefined normative types = 0
Undefined normative owners = 0
Unspecified Phase B algorithms = 0
Unspecified scientific thresholds/units = 0
Unspecified compatibility decisions = 0
Normative contradictions = 0
Fixture-to-real-schema contradictions = 0
Fixture-to-source-route contradictions = 0
Incomplete normative positive fixtures = 0
Unmapped active normative types = 0
Pipeline stages without exact API = 0
Serialized active types without exact wire shape = 0
Competing active production-order definitions = 0
Scientific gate APIs consuming Bound after eligibility = 0
ACTIVE V1 model production source references = 0
ACTIVE V1 direct EvidenceBundle source references = 0
Model temporal source references remaining = 0
Unmapped acceptance criteria = 0
Missing exact test names = 0
Frozen A1 semantic/API changes required = no
Implementation invention still required = no
Production Rust modified = NONE
Tests modified = NONE
Fixtures modified = NONE
Main unchanged = YES
Phase B implementation branch unchanged = YES
```

The previously passed Phase B areas listed in §28.9 remain PASS without
redesign, including the evidence-state model, eligibility ordering, temporal
support/serde, contradiction eligibility, all scientific gates, validation,
promotion, components, history, report assembly, traceability, wire formats,
CLI consistency, and frozen A1 evidence semantics.

READY_FOR_PHASE_B_SOURCE_COMPATIBILITY_REREVIEW = yes

## 30. Phase B Contract Remediation XII — final wire, history, schema-4, and transient-temporal reconciliation

### 30.1 Authority and blocker classification

This section is the final documentation-only amendment for the four remaining
wire/history/schema/source contradictions. It supersedes earlier Phase-B
definitions, fixture fragments, wire tables, API rows, test plans, and
self-audits only where this section defines `TemporalJoinConfig`,
`HypothesisHistoryEntry`, schema-4 legacy hypothesis naming, or transient
temporal support. Sections 27--29 remain controlling for all other Phase-B
areas. Frozen A1 APIs and serialized meaning, the legacy mechanism path,
`main`, and `codex/mhi-v1-b-mechanism-evidence-integration` remain unchanged.
This section authorizes no production Rust, test, or fixture modification.

| blocker | classification before this amendment | repository/specification evidence | final disposition |
|---|---|---|---|
| PB-FINAL-WIRE-01 `TemporalJoinConfig` mismatch | CONFIRMED | The final temporal algorithm uses overlap, event identity, clock mismatch, and scope mismatch policies; the later active declaration omitted those four fields while PB-FX-09/10 serialized them. | One eight-field config below is authoritative; PB-FX-09/10 use it exactly. |
| PB-FINAL-WIRE-02 `HypothesisHistoryEntry` mismatch | CONFIRMED | The approved history algorithm defines deterministic identity and provenance, while the later six-field declaration omitted `history_id` and `source_evidence_ids`, which PB-FX-10 serializes. | One eight-field history type below is authoritative; both fixture locations use it exactly. |
| PB-FINAL-WIRE-03 schema-4 hypothesis naming | CONFIRMED | The migration table already assigns schema-3 `hypotheses` to schema-4 `legacy_hypotheses`, but the canonical schema-4 report examples still emitted `hypotheses`. | Schema 3 reads `hypotheses`; schema 4 reads/writes `legacy_hypotheses`; new `hypothesis_assessments` is distinct. |
| PB-FINAL-WIRE-04 transient temporal support | CONFIRMED | `TransientEventResult` exposes `event_index`, an `ExperimentEvent` without a stable event ID, and `segment.fitted_time_local` start/end values. | Transient is `Window` when the fitted time vector is valid; otherwise `Unknown`; it is never `Event` in V1. |

No production or test inspection changed this classification. The actual
transient source fields are `TransientEventResult.event_index`,
`TransientEventResult.event`, and `TransientEventResult.segment.fitted_time_local`;
`ExperimentEvent` has timestamp/kind/value/annotation metadata but no stable
`event_id`. An array index is not an event identity contract.

### 30.2 PB-FINAL-WIRE-01 — one authoritative `TemporalJoinConfig`

`src/mechanism/temporal.rs::TemporalJoinConfig` is the sole active owner. Its
exact complete declaration is:

```rust
#[serde(deny_unknown_fields)]
pub struct TemporalJoinConfig {
    pub point_tolerance_s: f64,
    pub window_overlap_rule: WindowOverlapRule,
    pub event_identity_rule: EventIdentityRule,
    pub minimum_classified_fraction: f64,
    pub minimum_equilibrium_fraction: f64,
    pub mixed_state_policy: MixedStatePolicy,
    pub clock_mismatch_behavior: ClockMismatchBehavior,
    pub scope_mismatch_behavior: ScopeMismatchBehavior,
}

#[serde(rename_all = "snake_case")]
pub enum WindowOverlapRule { PositiveDuration }
#[serde(rename_all = "snake_case")]
pub enum EventIdentityRule { Exact }
#[serde(rename_all = "snake_case")]
pub enum ClockMismatchBehavior { Indeterminate }
#[serde(rename_all = "snake_case")]
pub enum ScopeMismatchBehavior { Indeterminate }
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MixedStatePolicy {
    RequireAllSteady { allow_quasi_equilibrium: bool },
    MinimumSteadyFraction {
        minimum_fraction: f64,
        allow_quasi_equilibrium: bool,
        reject_if_disturbed: bool,
    },
    WorstCase,
}
```

All eight fields are required in the present `[temporal]` TOML table. No
field has a default. The four policy fields are active algorithm inputs, not
descriptive annotations or fixture-only fields. The exact wire table is:

| Rust field | wire name | type / serde representation | required/default | validation | consumer |
|---|---|---|---|---|---|
| `point_tolerance_s` | `point_tolerance_s` | finite `f64` | required / none | `>= 0` seconds | Point–Point join |
| `window_overlap_rule` | `window_overlap_rule` | `WindowOverlapRule`, snake-case scalar | required / none | only `positive_duration` | Window–Window join |
| `event_identity_rule` | `event_identity_rule` | `EventIdentityRule`, snake-case scalar | required / none | only `exact` | Event–Event join when an Event source exists |
| `minimum_classified_fraction` | `minimum_classified_fraction` | finite `f64` | required / none | `[0,1]` | classification gate |
| `minimum_equilibrium_fraction` | `minimum_equilibrium_fraction` | finite `f64` | required / none | `[0,1]` | equilibrium gate |
| `mixed_state_policy` | `mixed_state_policy` | internally tagged table `{kind=...}` | required / none | exact variant fields; no unknown fields | mixed-state evaluation |
| `clock_mismatch_behavior` | `clock_mismatch_behavior` | `ClockMismatchBehavior`, snake-case scalar | required / none | only `indeterminate` | clock mismatch result |
| `scope_mismatch_behavior` | `scope_mismatch_behavior` | `ScopeMismatchBehavior`, snake-case scalar | required / none | only `indeterminate` | scope mismatch result |

The complete PB-FX-09 and PB-FX-10 temporal table is therefore:

```toml
[temporal]
point_tolerance_s = 0.0
window_overlap_rule = "positive_duration"
event_identity_rule = "exact"
minimum_classified_fraction = 0.0
minimum_equilibrium_fraction = 0.0
clock_mismatch_behavior = "indeterminate"
scope_mismatch_behavior = "indeterminate"

[temporal.mixed_state_policy]
kind = "require_all_steady"
allow_quasi_equilibrium = false
```

`MechanismEvidenceConfig` and all nested Phase-B configuration structs use
`deny_unknown_fields` or the equivalent strict parser policy. Missing fields,
unknown fields, duplicate fields, invalid enum values, invalid tagged-table
shape, non-finite numbers, and range violations are configuration errors.
PB-FX-09 and PB-FX-10 must each deserialize this table into
`TemporalJoinConfig`, serialize it, reload it, and compare semantic equality;
the round-trip does not add, drop, or rename a field. The temporal config /
fixture contradiction count is zero.

### 30.3 PB-FINAL-WIRE-02 — one authoritative `HypothesisHistoryEntry`

`src/mechanism/history.rs::HypothesisHistoryEntry` is the sole active
history-entry type. The complete final declaration is:

```rust
#[serde(deny_unknown_fields)]
pub struct HypothesisHistoryEntry {
    pub history_id: String,
    pub hypothesis_id: MechanismHypothesisId,
    pub prior_level: HypothesisEvidenceLevel,
    pub new_level: HypothesisEvidenceLevel,
    pub assessment_target: Option<InterpretationStatus>,
    pub assessment_index: u64,
    pub reason_codes: Vec<PhaseBHypothesisReasonCode>,
    pub source_evidence_ids: Vec<EvidenceId>,
}
```

Every field is serialized with its Rust snake-case name. There are no defaults
for a persisted history entry. The field contract is:

| field | type | source and deterministic construction | semantic-hash participation |
|---|---|---|---|
| `history_id` | `String` | lowercase 64-hex SHA-256 of `hypothesis_id || NUL || prior_level || NUL || new_level || NUL || assessment_hash`; no random UUID | persisted schema-4 field; derived from the same scientific transition |
| `hypothesis_id` | `MechanismHypothesisId` | current assessment hypothesis ID | yes |
| `prior_level` | `HypothesisEvidenceLevel` | explicit prior schema-4 assessment state | yes |
| `new_level` | `HypothesisEvidenceLevel` | current assessment result | yes |
| `assessment_target` | `Option<InterpretationStatus>` | current component-promotion target; `None` means no promotion | yes |
| `assessment_index` | `u64` | prior maximum for the hypothesis plus one; first appended transition is `1` | yes |
| `reason_codes` | `Vec<PhaseBHypothesisReasonCode>` | reasons that caused the transition, bytewise sorted and duplicate-free | yes |
| `source_evidence_ids` | `Vec<EvidenceId>` | only EvidenceIds actually consumed by the persisted transition: the sorted, duplicate-free union of evidence IDs used by the current gate/validation assessment and its component assessment rows; unused eligible candidates are excluded | yes |

`assessment_hash` is the existing internal RFC-8785 SHA-256 of the
deterministic scientific assessment view. It is an input to `history_id` and
duplicate suppression but is not an additional serialized history field in
the final six-field declaration plus the two reconciled fields. A transition
is appended only when no prior entry for the hypothesis has the same
`(prior_level, new_level, assessment_hash)`. History entries serialize sorted
by `hypothesis_id`, then `assessment_index`.

PB-FX-09 has no prior schema-4 transition and therefore serializes empty
history at both the current-assessment and report-history locations. PB-FX-10
has exactly one entry. Its `source_evidence_ids` are exactly the four consumed
IDs, sorted bytewise:

```json
{
  "history_id": "SHA256(exact declared history-key bytes)",
  "hypothesis_id": "b-hypothesis",
  "prior_level": "experimentally_supported",
  "new_level": "validated_for_domain",
  "assessment_target": "validated_for_domain",
  "assessment_index": 1,
  "reason_codes": ["validation_satisfied"],
  "source_evidence_ids": [
    "calibration.observation.0",
    "eis.parameter.0",
    "estimation.point.0.state.0",
    "transient.event.0.tau_fast_s"
  ]
}
```

The placeholder notation above denotes the deterministic digest assertion,
not arbitrary fixture text. PB-FX-09 and PB-FX-10 deserialize and reserialize
the exact same `HypothesisHistoryEntry` type; the history type / fixture
contradiction count is zero. Earlier declarations containing a different
history shape (`PhaseBHypothesisHistory`, serialized `assessment_hash`,
`sequence`, or `assessed_at`) are historical/superseded for this final wire
contract; the existing internal assessment hash and duplicate rule remain.

### 30.4 PB-FINAL-WIRE-03 — singular schema-4 legacy hypothesis name

The current schema-3 `MechanismAnalysisReport` payload fields, verified in
`src/results/mechanism.rs`, are:

```text
schema_version, lineage, analysis_id, records, eis_timescales,
transient_timescales, comparisons, hypotheses, trends, configuration,
provenance, warnings, transient_configuration
```

The final ownership rule is singular:

| concept | schema-3 read field | schema-4 canonical field | meaning |
|---|---|---|---|
| preserved legacy payload | `hypotheses` | `legacy_hypotheses` | exact schema-3 `Vec<HypothesisAssessment>` payload; preserved, not reinterpreted |
| new Phase-B output | absent | `hypothesis_assessments` | `Vec<HypothesisAssessmentRecord>` containing the new definition/current assessment |
| new Phase-B history | absent | `hypothesis_history` | `Vec<HypothesisHistoryEntry>` |

Reader and writer behavior is exact:

1. Reading schema 3 accepts `hypotheses` and maps it in memory to
   `legacy_hypotheses`; missing new Phase-B vectors become empty and mean
   `NotAssessed`.
2. Reading schema 4 expects `legacy_hypotheses`,
   `hypothesis_assessments`, and `hypothesis_history`. An input-only serde
   alias for schema-3 `hypotheses` may be used by the compatibility reader,
   but both names in one payload are rejected as ambiguous.
3. Writing schema 4 always emits `legacy_hypotheses`; it never emits the old
   top-level `hypotheses` name. It writes `hypothesis_assessments` and
   `hypothesis_history`, including empty vectors when no Phase-B result exists.
4. The nested `configuration.hypotheses` field is unrelated legacy
   `ResolvedMechanismConfig` data and remains unchanged; it is not the
   schema-4 report field.

The schema-4 report wire table is:

| Rust field | wire name | type | required/default | serde representation and validation |
|---|---|---|---|---|
| `schema_version` | `schema_version` | `u32` | required; writer `4` | exact integer; readable versions `1..=4` per artifact policy |
| `lineage` | `lineage` | `ArtifactLineageState` | required in canonical output; legacy reader default remains existing A1 default | existing lineage representation and validation |
| `analysis_id` | `analysis_id` | `String` | required | nonempty existing report ID rules |
| `records` | `records` | `Vec<MechanismRecordSummary>` | required; empty allowed | array; existing element validation |
| `eis_timescales` | `eis_timescales` | `Vec<CharacteristicTimescale>` | required; empty allowed | array; existing element validation |
| `transient_timescales` | `transient_timescales` | `Vec<CharacteristicTimescale>` | required; empty allowed | array; existing element validation |
| `comparisons` | `comparisons` | `Vec<TimescaleComparison>` | required; empty allowed | array; existing element validation |
| legacy payload field | `legacy_hypotheses` | `Vec<HypothesisAssessment>` | required in schema-4 writer; empty allowed | array; read alias `hypotheses` is migration-only; no dual names |
| `trends` | `trends` | `Vec<MechanismTrendResult>` | required; empty allowed | array; existing element validation |
| `configuration` | `configuration` | `ResolvedMechanismConfig` | required | existing legacy config shape, including nested `hypotheses` |
| `provenance` | `provenance` | `Option<AnalysisProvenance>` | required; `null` allowed | nullable object |
| `warnings` | `warnings` | `Vec<MechanismWarning>` | required; empty allowed | array |
| `transient_configuration` | `transient_configuration` | `Option<ResolvedTransientConfig>` | required; `null` allowed | nullable object |
| `hypothesis_assessments` | `hypothesis_assessments` | `Vec<HypothesisAssessmentRecord>` | required; empty allowed | array; complete Phase-B record shape |
| `hypothesis_history` | `hypothesis_history` | `Vec<HypothesisHistoryEntry>` | required; empty allowed | array; complete final history shape above |

PB-FX-09 and PB-FX-10 canonical schema-4 report examples now use
`"legacy_hypotheses": []` at the report level. They contain zero report-level
`"hypotheses"` fields and retain only the nested legacy configuration field.
The schema-3 prior input alone uses report-level `"hypotheses": []`.
Schema-3 migration, schema-4 writer naming, and fixture naming are therefore
singular and contradiction-free.

### 30.5 PB-FINAL-WIRE-04 — transient is `Window`, not `Event`

The canonical transient route is:

```text
TransientAnalysisReport.events[i].segment.fitted_time_local
  → start_s = first value, end_s = last value
  → EvidenceTemporalSupport::Window { start_s, end_s }
```

This support is emitted only when the vector is nonempty, every value is
finite, values are strictly increasing in serialized order, and
`start_s < end_s`. Otherwise the support is `Unknown`. The event's
`event_index` is used only to construct the adapter EvidenceId family; it is
not an `EventId`, and the embedded `ExperimentEvent` has no stable event ID.
The transient adapter therefore cannot satisfy the contracted Event identity
semantics. It must not emit `Event { event_id: i.to_string(), ... }`.

The single active V1 source map is:

| accepted source | adapter EvidenceId | authoritative temporal field | support |
|---|---|---|---|
| `EisFitArtifact` | `eis.parameter.{i}` | none | `Unknown` |
| `TransientAnalysisReport` | `transient.event.{i}.parameter.{j}`, `.tau_fast_s`, `.tau_slow_s` | `events[i].segment.fitted_time_local` | `Window` when valid; otherwise `Unknown` |
| `StateEstimationReport` | `estimation.point.{i}.state.{j}` | `estimates[i].timestamp_s` | `Point` when finite; otherwise `Unknown` |
| `CalibrationObservationSet` | `calibration.observation.{i}` | no comparable V1 clock | `Unknown` |

The PB-FX-09 registry uses the transient `Window { start_s=0.0,
end_s=10.0 }` row, and PB-FX-10 uses the same row. Every active source table,
fixture registry, temporal catalog expectation, and route map uses this one
mapping. Event remains a defined support variant for a future source that
provides a stable event ID and interval; no current V1 source claims it.

### 30.6 Canonical fixture re-audit, acceptance mapping, and regression

The final PB-FX-09/PB-FX-10 checks are exact:

| acceptance criterion | PB-FX-09 | PB-FX-10 | exact test |
|---|---|---|---|
| temporal config deserializes and round-trips semantically | yes | yes | `phase_b_fx09_temporal_config_deserializes`, `phase_b_fx10_temporal_config_deserializes` |
| history type matches serialized fixture | empty history | one complete eight-field entry | `phase_b_history_entry_roundtrip`, `phase_b_history_entry_matches_fixture` |
| schema-3 legacy payload migrates | read `hypotheses` → in-memory `legacy_hypotheses` | same prior-input rule | `phase_b_schema3_hypotheses_migrate_to_legacy_hypotheses` |
| schema-4 writer uses canonical legacy name | `legacy_hypotheses`; no report-level `hypotheses` | `legacy_hypotheses`; no report-level `hypotheses` | `phase_b_schema4_writer_emits_legacy_hypotheses` |
| transient source route matches actual fields | `Window` | `Window` | `phase_b_transient_temporal_support_matches_source_contract` |

No test is retained for the retired transient Event route or for a
fixture-only history field. Missing exact test names = 0 and unmapped
acceptance criteria = 0.

The 18-stage pipeline remains exactly the §29/§28 pipeline: source loading,
preparation, binding, eligibility, contradiction assessment, all scientific
gates, validation, promotion, component assessment, history update, schema-4
assembly, and writing are unchanged. Only the affected config input, history
wire shape, schema-4 legacy field name, and transient source-support row are
updated. The production API table remains complete; no A1 API is changed.

Previously passed Phase-B areas remain PASS: accepted source set;
`ModelAnalysisReport` exclusion; direct `EvidenceBundle` retirement;
canonical evidence-state model; Bound structural-only; eligible gate input;
eligibility API; temporal mode and temporal-before-contradiction ordering;
role/stage/gate; amplitude; repeatability; identifiability; validation;
critical contradictions; 4×4 component matrix; PB-FX-09; PB-FX-10; ArtifactId
policy; production API table; 18-stage pipeline; promotion; components;
history algorithm; traceability; CLI; source compatibility; and frozen A1
compatibility.

### 30.7 Supersession audit and final self-audit

The required terminology has one active interpretation:

| occurrence | classification | controlling rule |
|---|---|---|
| `TemporalJoinConfig` declarations before §30 | SUPERSEDED where their field set differs | §30.2 eight-field declaration is sole active type |
| `window_overlap_rule`, `event_identity_rule`, `clock_mismatch_behavior`, `scope_mismatch_behavior` in PB-FX-09/10 | ACTIVE NORMATIVE | required fields and exact enum values in §30.2 |
| older temporal policy names | SUPERSEDED or DESCRIPTIVE | no second config owner or wire shape |
| `HypothesisHistoryEntry` declarations before §30 | SUPERSEDED where their field set differs | §30.3 eight-field declaration is sole active type |
| `history_id` and `source_evidence_ids` | ACTIVE NORMATIVE | deterministic identity and consumed-evidence provenance in §30.3 |
| schema-3 report-level `hypotheses` | MIGRATION-ONLY | accepted on schema-3 read and mapped to `legacy_hypotheses` |
| schema-4 report-level `legacy_hypotheses` | ACTIVE NORMATIVE | canonical writer field and canonical fixture field |
| nested `configuration.hypotheses` | ACTIVE legacy field | not the report-level migration field |
| `EvidenceTemporalSupport::Window` for transient | ACTIVE NORMATIVE | derived from fitted time interval |
| `EvidenceTemporalSupport::Event` for transient | SUPERSEDED / unsupported V1 route | no stable event identity in the actual source |
| generic `Event` support variant | ACTIVE type, not transient mapping | reserved for a source with the contracted event ID |

Final self-audit:

```text
Undefined normative types = 0
Undefined normative owners = 0
Unspecified Phase B algorithms = 0
Unspecified scientific thresholds/units = 0
Unspecified compatibility decisions = 0
Normative contradictions = 0
Fixture-to-real-schema contradictions = 0
Fixture-to-source-route contradictions = 0
Fixture-to-wire-contract contradictions = 0
Incomplete normative positive fixtures = 0
Unmapped active normative types = 0
Pipeline stages without exact API = 0
Serialized active types without exact wire shape = 0
Competing active production-order definitions = 0
Unmapped acceptance criteria = 0
Missing exact test names = 0
Frozen A1 semantic/API changes required = no
Implementation invention still required = no
Production Rust modified = NONE
Tests modified = NONE
Fixtures modified = NONE
Main unchanged = YES
Phase B implementation branch unchanged = YES
```

Could two competent implementers differ about which fields
`TemporalJoinConfig` contains? **NO.** Could they differ about whether PB-FX
temporal config deserializes? **NO.** Could they differ about whether
`HypothesisHistoryEntry` contains `history_id` or `source_evidence_ids`?
**NO.** Could they differ about whether the schema-4 writer emits
`hypotheses` or `legacy_hypotheses`? **NO.** Could they differ about whether
transient support is `Event` or `Window`? **NO.**

READY_FOR_PHASE_B_FINAL_WIRE_CONTRACT_REREVIEW = yes

## 31. Phase B Contract Remediation XIII — canonical assessment-hash preimage and history identity

### 31.1 Authority, confirmed finding, and scope

This documentation-only section resolves **PB-HISTORY-HASH-01** and is the
sole ACTIVE NORMATIVE Phase-B definition of the `assessment_hash` preimage,
`assessment_hash` algorithm, `history_id` preimage/algorithm, and history
duplicate rule. The P1 is **CONFIRMED**: §30 correctly makes `history_id`
depend on `assessment_hash`, but its phrase “deterministic scientific
assessment view” does not define one canonical payload, ordering, null rule,
or float rule. No production Rust, test, fixture, frozen A1 API, serialized
schema-4 report field, pipeline stage count, `main`, or
`codex/mhi-v1-b-mechanism-evidence-integration` is changed by this amendment.

`src/mechanism/history.rs` owns this Phase-B-only identity contract. It may
reuse the repository's existing `serde_jcs` plus SHA-256 utility, but it MUST
NOT repurpose an A1 semantic-hash type or change A1 identity semantics.
There is exactly one assessment-hash preimage type:

```rust
// Non-wire helper: not an independently persisted schema-4 object and not a
// replacement for PhaseBHypothesisAssessment.
pub struct HypothesisAssessmentHashView {
    pub hypothesis_id: MechanismHypothesisId,
    pub evidence_level: HypothesisEvidenceLevel,
    pub temporal_join_assessments: Vec<TemporalJoinAssessmentHashView>,
    pub timescale_assessments: Vec<TimescaleAssessmentHashView>,
    pub amplitude_assessments: Vec<AmplitudeAssessmentHashView>,
    pub repeatability_assessments: Vec<RepeatabilityAssessmentHashView>,
    pub identifiability_assessments: Vec<IdentifiabilityAssessmentHashView>,
    pub contradiction_summaries: Vec<RequirementContradictionSummaryHashView>,
    pub validation_assessment: Option<ValidationAssessmentHashView>,
    pub reason_codes: Vec<PhaseBHypothesisReasonCode>,
    pub component_assessments: Vec<ComponentInterpretationAssessmentHashView>,
    pub source_evidence_ids: Vec<EvidenceId>,
}

// Non-wire typed string. Its serde representation, when required by a helper
// or test, is exactly 64 lowercase hexadecimal digits (no prefix), matching
// the already-approved internal semantic-hash convention.
pub struct AssessmentHash(pub String);

// Non-wire canonical preimage for history identity.
pub struct HypothesisHistoryIdView {
    pub hypothesis_id: MechanismHypothesisId,
    pub prior_level: HypothesisEvidenceLevel,
    pub new_level: HypothesisEvidenceLevel,
    pub assessment_hash: AssessmentHash,
}
```

No `hash_domain` field is added. The approved identity architecture did not
already use a Phase-B domain-separation wrapper; adding one would be a new
cryptographic design rather than a remediation of the ambiguous payload.

### 31.2 Exact semantic field set and nested views

The view contains semantic assessment results and their semantic inputs. It
excludes presentation, runtime, and recursive identity fields. In particular,
`history_id`, `assessment_hash`, sequence/index, wall-clock time, paths, CLI
strings, map insertion order, display labels, diagnostic prose, serializer
formatting, process time, and random values are excluded. `history_id` and
`assessment_hash` can therefore never recursively participate in their own
preimages.

The complete nested helper types are exact; a nested persisted object is never
described merely as “its semantic parts.” All fields below serialize with their
shown snake-case names and use the existing snake-case serde token of each
existing enum.

```rust
pub struct TemporalJoinAssessmentHashView {
    pub requirement_id: EvidenceRequirementId,
    pub left_evidence_id: EvidenceId,
    pub right_evidence_id: EvidenceId,
    pub mode: TemporalJoinMode,
    pub outcome: TemporalJoinOutcome,
    pub classified_fraction: Option<f64>,
    pub equilibrium_fraction: Option<f64>,
    pub steady_state_fraction: Option<f64>,
    pub reasons: Vec<TemporalJoinReasonCode>,
}
pub struct TimescaleAssessmentHashView {
    pub pair_requirement_id: EvidenceRequirementId,
    pub status: TimescaleStatus,
    pub evidence_ids: Vec<EvidenceId>,
    pub log_distance: Option<f64>,
}
pub struct AmplitudeThresholdHashView { pub value: f64, pub unit: String }
pub struct AmplitudeAssessmentHashView {
    pub predicted_requirement_id: EvidenceRequirementId,
    pub observed_requirement_id: EvidenceRequirementId,
    pub status: AmplitudeStatus,
    pub predicted_evidence_id: Option<EvidenceId>,
    pub observed_evidence_id: Option<EvidenceId>,
    pub threshold: AmplitudeThresholdHashView,
    pub relative_error: Option<f64>,
    pub reasons: Vec<AmplitudeReasonCode>,
}
pub struct RepeatabilityAssessmentHashView {
    pub requirement_ids: Vec<EvidenceRequirementId>,
    pub status: RepeatabilityStatus,
    pub evidence_ids: Vec<EvidenceId>,
    pub sample_standard_deviation_ln_tau: Option<f64>,
}
pub struct IdentifiabilityAssessmentHashView {
    pub requirement_id: IdentifiabilityRequirementId,
    pub status: IdentifiabilityAssessmentStatus,
    pub metric_value: Option<f64>,
    pub evidence_ids: Vec<EvidenceId>,
    pub reasons: Vec<IdentifiabilityAssessmentReasonCode>,
}
pub struct RequirementContradictionSummaryHashView {
    pub requirement_id: EvidenceRequirementId,
    pub evidence_ids: Vec<EvidenceId>,
    pub contradiction_count: usize,
    pub strong_critical_count: usize,
}
pub struct ValidationAssessmentHashView {
    pub protocol_id: String,
    pub status: ValidationProtocolStatus,
    pub evidence_ids: Vec<EvidenceId>,
    pub acquisition_family_ids: Vec<AcquisitionFamilyId>,
    pub passed_condition_ids: Vec<String>,
    pub reasons: Vec<ValidationReasonCode>,
}
pub struct ComponentInterpretationAssessmentHashView {
    pub component_id: ComponentId,
    pub prior_status: InterpretationStatus,
    pub assessment_target: Option<InterpretationStatus>,
    pub resulting_status: InterpretationStatus,
    pub supporting_hypothesis_id: MechanismHypothesisId,
    pub evidence_ids: Vec<EvidenceId>,
    pub reasons: Vec<ComponentInterpretationReasonCode>,
}
```

The source-to-view inclusion decision is exhaustive:

| Source field | Included? | hash-view path / normalization | Reason |
|---|---:|---|---|
| `PhaseBHypothesisAssessment.hypothesis_id` | yes | `hypothesis_id`; canonical ID string | identifies the assessed hypothesis |
| `.evidence_level` | yes | `evidence_level`; existing enum token | resulting semantic level |
| `.temporal_join_assessments` | yes | `temporal_join_assessments`; nested view | gate outcome is scientific meaning |
| `.timescale_assessments` | yes | `timescale_assessments`; nested view | gate outcome is scientific meaning |
| `.amplitude_assessments` | yes | `amplitude_assessments`; nested view | gate outcome is scientific meaning |
| `.repeatability_assessments` | yes | `repeatability_assessments`; nested view | gate outcome is scientific meaning |
| `.identifiability_assessments` | yes | `identifiability_assessments`; nested view | gate outcome is scientific meaning |
| `.contradiction_summaries` | yes | `contradiction_summaries`; nested view | direct contradiction result is scientific meaning |
| `.reason_codes` | yes | `reason_codes`; enum-token sorted/deduplicated | machine explanation of the result |
| `.component_assessments` | yes | `component_assessments`; nested view | persisted component result is assessment meaning |
| `.validation_status` / the active validation result | yes | `validation_assessment`; `Some` exact nested view, `None` is JSON `null` | protocol result, families, conditions, and reasons are semantic |
| `.history` / report `hypothesis_history` | no | excluded | prior transition log and derived IDs are not the current assessment; including it would make identity history-position dependent |
| `MechanismEvidenceConfig`, including `display_name`, CLI and source-path metadata | no | excluded | config is report/config identity, not a field of the persisted assessment result; every resulting gate/config consequence is represented by the included assessment rows |
| consumed transition evidence, derived from the preceding included nested rows | yes | `source_evidence_ids`; sorted/deduplicated union | history provenance is part of transition identity and appears once as the canonical root collection |
| `HypothesisHistoryEntry.history_id` | no | excluded | recursive derived identity |
| `HypothesisHistoryEntry.assessment_index` | no | excluded | append position, not assessment meaning |
| generated timestamp, `assessed_at`, current process time | no | excluded | runtime/presentation metadata |
| paths, CLI invocation, labels, diagnostic prose, formatting/order | no | excluded | incidental presentation/runtime state |

Every nested field is likewise decided; “same as the persisted type” means the
corresponding exact helper declaration above, not implicit direct serialization.

| Nested source | Included fields | Excluded fields |
|---|---|---|
| `TemporalJoinAssessment` | `requirement_id,left_evidence_id,right_evidence_id,mode,outcome,classified_fraction,equilibrium_fraction,steady_state_fraction,reasons` | none |
| `TimescaleAssessment` | `pair_requirement_id,status,evidence_ids,log_distance` | none |
| `AmplitudeAssessment` and `AmplitudeThreshold` | `predicted_requirement_id,observed_requirement_id,status,predicted_evidence_id,observed_evidence_id,threshold.value,threshold.unit,relative_error,reasons` | none |
| `RepeatabilityAssessment` | `requirement_ids,status,evidence_ids,sample_standard_deviation_ln_tau` | none |
| `IdentifiabilityAssessment` | `requirement_id,status,metric_value,evidence_ids,reasons` | none |
| `RequirementContradictionSummary` | `requirement_id,evidence_ids,contradiction_count,strong_critical_count` | none |
| `ValidationAssessment` | `protocol_id,status,evidence_ids,acquisition_family_ids,passed_condition_ids,reasons` | none |
| `ComponentInterpretationAssessment` | `component_id,prior_status,assessment_target,resulting_status,supporting_hypothesis_id,evidence_ids,reasons` | none |

### 31.3 Normalization, null, map, and float rules

RFC 8785 canonicalizes object members only. The builder MUST apply these rules
before JCS serialization; a duplicate row with a semantic key is a typed error
rather than a traversal-order-dependent choice.

| Collection | Exact canonical order and duplicate rule |
|---|---|
| root `source_evidence_ids`; every nested `evidence_ids` | canonical `EvidenceId` ascending; deduplicate |
| `temporal_join_assessments` | `(requirement_id,left_evidence_id,right_evidence_id,mode-token)` ascending; duplicate tuple rejected |
| `timescale_assessments` | `pair_requirement_id` ascending; duplicate rejected |
| `amplitude_assessments` | `(predicted_requirement_id,observed_requirement_id)` ascending; duplicate rejected |
| `repeatability_assessments` | lexicographic sorted `requirement_ids` tuple ascending; duplicate tuple rejected |
| `repeatability_assessments[*].requirement_ids` | requirement ID ascending; deduplicate before the row key is formed |
| `identifiability_assessments` | `requirement_id` ascending; duplicate rejected |
| `contradiction_summaries` | `requirement_id` ascending; duplicate rejected |
| `reason_codes` and every nested `reasons` vector | canonical serialized snake-case enum token ascending; deduplicate |
| `component_assessments` | `component_id` ascending; duplicate rejected |
| validation `acquisition_family_ids`, `passed_condition_ids`, and `evidence_ids` | canonical ID/string ascending; deduplicate |

`source_evidence_ids` is exactly the sorted, duplicate-free union of every
`EvidenceId` present in the canonical nested gate, validation, and component
views (including temporal left/right and populated amplitude options). The
builder rejects a caller-provided source list that differs from that union.
Thus source evidence is included exactly once as a root identity collection,
while each assessment row retains its own source reference as required to
express that row's meaning.

The hash view contains no map. If a future revision introduces one, it MUST
use a `BTreeMap<String, _>` in the hash view, convert each typed key to its
canonical string before insertion, reject duplicate semantic keys, and only
then rely on RFC-8785 object-member ordering; `HashMap` iteration is forbidden.

Every `Option<T>` in every hash-view declaration serializes with normal serde
as an explicit JSON `null` for `None`; `skip_serializing_if` is forbidden on a
hash-view type. Every hashed float, including floats nested in optional fields,
MUST be finite. `NaN`, `+∞`, and `-∞` are invalid hash input.
Before constructing the view, `-0.0` is normalized to `0.0`; no approved
Phase-B semantic distinguishes them. JCS/`serde_jcs` solely owns final JSON
number rendering after this validation and normalization.

### 31.4 Canonical procedures, errors, and wire form

The only assessment procedure is:

```rust
pub fn build_hypothesis_assessment_hash_view(
    assessment: &PhaseBHypothesisAssessment,
    component_assessments: &[ComponentInterpretationAssessment],
) -> Result<HypothesisAssessmentHashView, HypothesisAssessmentHashError>;

pub fn compute_assessment_hash(
    view: &HypothesisAssessmentHashView,
) -> Result<AssessmentHash, HypothesisAssessmentHashError>;

pub fn compute_history_id(
    view: &HypothesisHistoryIdView,
) -> Result<String, HypothesisAssessmentHashError>;
```

`build_hypothesis_assessment_hash_view` validates all semantic values,
verifies the separately supplied component slice is equal to the assessment's
canonical component rows, derives/verifies `source_evidence_ids`, applies every
rule in §31.3, and returns the one view above. It neither reads a config hash
or clock nor accepts a history ID/hash from its caller.

`compute_assessment_hash` serializes that view with the repository's existing
RFC-8785/JCS implementation (`serde_jcs::to_vec`), hashes those exact UTF-8
bytes with SHA-256, and returns:

```text
assessment_hash = lowercase_hex(SHA-256(jcs_bytes))
```

`AssessmentHash` validation rejects a prefix, uppercase, non-hex data, and any
length other than 64 characters. This preserves the already-approved internal
semantic-hash representation; it is not the frozen A1 `ArtifactId` wrapper.
JCS
serialization is not replaceable with `serde_json` or implementation-specific
float formatting.

The complete current approved history distinction is hypothesis, prior level,
new level, and the canonical assessment result. Therefore `compute_history_id`
JCS-serializes exactly `HypothesisHistoryIdView`, then returns:

```text
history_id = lowercase_hex(SHA-256(history_id_jcs_bytes))
```

The typed failures are added to the existing Phase-B history/assessment error
surface, not to A1: `NonFiniteHashedFloat { path }`,
`DuplicateSemanticKey { collection, key }`, `InvalidAssessmentHashEncoding`,
`JcsSerialization { detail }`, and `HistoryHashConstruction { detail }`.

Two history entries are semantically duplicate **iff** their
`(hypothesis_id, prior_level, new_level, assessment_hash)` tuples are equal.
`update_hypothesis_history` computes the candidate `HypothesisHistoryIdView`
and uses its `history_id` to recognize that tuple among persisted history rows;
this is the required implementation of the canonical tuple test because the
wire history row intentionally does not persist `assessment_hash` separately.
It appends no new entry when the candidate ID already exists for the same
hypothesis. `assessment_index` is assigned only after the no-duplicate decision
as the prior maximum index for that hypothesis plus one. The serialized history
remains sorted by `hypothesis_id`, then `assessment_index`.

### 31.5 PB-HASH-01 fixed RFC-8785 vector

PB-HASH-01 is the complete normalized semantic
`HypothesisAssessmentHashView` with `hypothesis_id="pb-hash-01"`,
`evidence_level=NotAssessed`, every collection empty, and the one optional
member explicitly `None`/`null`. Every root member is present. Its exact
snake-case enum token is therefore `not_assessed`:

```json
{"amplitude_assessments":[],"component_assessments":[],"contradiction_summaries":[],"evidence_level":"not_assessed","hypothesis_id":"pb-hash-01","identifiability_assessments":[],"reason_codes":[],"repeatability_assessments":[],"source_evidence_ids":[],"temporal_join_assessments":[],"timescale_assessments":[],"validation_assessment":null}
```

That literal JSON string is the exact UTF-8 JCS byte sequence (no newline).
It was computed with this repository's locked `serde_jcs = 0.2.0` semantics.
Its expected SHA-256 is
`c0d071e535bef8d14993e3a3e5b2a8209e38771655d2582e605c93096e7c6bd1`,
and its expected `assessment_hash` is
`c0d071e535bef8d14993e3a3e5b2a8209e38771655d2582e605c93096e7c6bd1`.

The PB-HASH-01 history preimage is exactly:

```json
{"assessment_hash":"c0d071e535bef8d14993e3a3e5b2a8209e38771655d2582e605c93096e7c6bd1","hypothesis_id":"pb-hash-01","new_level":"not_assessed","prior_level":"hypothesized"}
```

That literal JSON string is likewise the exact UTF-8 JCS byte sequence (no
newline). Its expected SHA-256 and `history_id` are both
`4de00157ee00baa8f1ff5dc89297464668d39f08392293ca48446fa9e6127bd3`,
respectively.

### 31.6 Fixture, API, inventory, and test alignment

PB-FX-09 remains an empty-history first run. It nonetheless builds its
canonical assessment view with source IDs
`[eis.parameter.0, transient.event.0.tau_fast_s]`, computes
`assessment_hash` through `compute_assessment_hash`, and asserts the view's
sorted source union and hash match the production result.

PB-FX-10 remains the one-transition run. It builds its full canonical
assessment view from the declared current assessment and four component rows,
asserts source IDs
`[calibration.observation.0, eis.parameter.0, estimation.point.0.state.0,
transient.event.0.tau_fast_s]`, computes `assessment_hash`, constructs the
exact `HypothesisHistoryIdView { hypothesis_id: "b-hypothesis",
prior_level: ExperimentallySupported, new_level: ValidatedForDomain,
assessment_hash }`, and asserts the persisted `history_id` equals
`compute_history_id`. No fixture contains a placeholder digest. Fixture tests
may derive these two hashes through the canonical production functions because
PB-HASH-01 is the fixed byte-level cross-implementation vector.

The controlling inventory gains the following non-wire items, all owned by
`src/mechanism/history.rs`: `HypothesisAssessmentHashView`, every named nested
`*HashView`, `AssessmentHash`, `HypothesisHistoryIdView`,
`build_hypothesis_assessment_hash_view`, `compute_assessment_hash`, and
`compute_history_id`. Each has serialized report field **no**; its purpose,
input, output, and error are exactly §§31.2–31.4. The stage-16 API row is
superseded only as follows:

```text
previous history + current PhaseBHypothesisAssessment + canonical component rows
→ build_hypothesis_assessment_hash_view → assessment_hash
→ HypothesisHistoryIdView → history_id → semantic duplicate check
→ update_hypothesis_history(...) → updated Vec<HypothesisHistoryEntry>
```

The 18-stage pipeline and all other stage APIs are unchanged. Required exact
tests are `phase_b_assessment_hash_view_normalizes_order`,
`phase_b_assessment_hash_rfc8785_vector`,
`phase_b_assessment_hash_rfc8785_validated_vector`,
`phase_b_assessment_hash_rejects_non_finite_float`,
`phase_b_history_id_is_deterministic`,
`phase_b_history_duplicate_suppression_uses_semantic_identity`,
`phase_b_fx09_history_hash_matches_canonical_view`, and
`phase_b_fx10_history_hash_matches_canonical_view`.

**PB-HASH-VECTOR-INTEGRITY.** For every fixed Phase-B hash vector,
`SHA256(exact documented RFC-8785/JCS UTF-8 bytes)` MUST equal the documented
expected SHA-256. The documented history-ID preimage MUST likewise hash to the
documented history ID. A mismatch is a P1 contract error, not an
implementation failure. The exact tests are
`phase_b_assessment_hash_rfc8785_vector`,
`phase_b_assessment_hash_rfc8785_validated_vector`, and
`phase_b_history_id_is_deterministic`; they MUST hash the literal documented
JCS bytes and MUST NOT hard-code an unrelated expected digest.

### 31.7 Supersession and final audit

| occurrence | classification | controlling rule |
|---|---|---|
| pre-§31 phrases “deterministic scientific assessment view” / category-only `assessment_hash` descriptions | SUPERSEDED | §31.2–31.5 are the one payload and algorithm |
| pre-§31 `history_id` concatenated-key construction | SUPERSEDED | §31.4 `HypothesisHistoryIdView` JCS construction |
| §30 eight-field `HypothesisHistoryEntry` wire shape and source-ID meaning | ACTIVE NORMATIVE | unchanged by this section |
| PB-FX-09/10 placeholder history digest prose | SUPERSEDED | §31.6 canonical-function assertions |
| PB-HASH-01's pre-§31 `Unassessed`/`unassessed` vector spelling | SUPERSEDED | §31.5's canonical `NotAssessed`/`not_assessed` semantic vector |
| RFC-8785/JCS mentions outside this identity contract | DESCRIPTIVE or their separately owned active contract | no second Phase-B assessment/history preimage |

All previously passed Phase-B areas remain PASS without redesign: temporal
config/serde, transient Window mapping, schema 3→4 naming, source
compatibility, `ModelAnalysisReport` exclusion, direct `EvidenceBundle`
retirement, evidence-state/eligibility/contradiction/role/stage/gate behavior,
amplitude, repeatability, identifiability, validation, component matrix,
source-evidence semantics, the 18-stage pipeline, production API table,
PB-FX-09/10 source routes, traceability, CLI, and frozen A1 compatibility.

```text
Undefined normative types = 0
Undefined normative owners = 0
Unspecified Phase B algorithms = 0
Unspecified scientific thresholds/units = 0
Unspecified compatibility decisions = 0
Normative contradictions = 0
Fixture-to-real-schema contradictions = 0
Fixture-to-source-route contradictions = 0
Fixture-to-wire-contract contradictions = 0
Incomplete normative positive fixtures = 0
Unmapped active normative types = 0
Pipeline stages without exact API = 0
Serialized active types without exact wire shape = 0
Competing active production-order definitions = 0
Unmapped acceptance criteria = 0
Missing exact test names = 0
Fixed hash vectors whose literal JCS bytes do not match documented digest = 0
Fixed history vectors whose literal JCS preimage does not match documented history ID = 0
PB-HASH-01 mathematical inconsistencies = 0
PB-HASH-02 mathematical inconsistencies = 0
Assessment-hash fields with unspecified inclusion/exclusion = 0
Assessment-hash vectors with unspecified ordering = 0
Assessment-hash Option/null semantics unspecified = 0
Assessment-hash float semantics unspecified = 0
Canonical fixed hash test vectors = 1
Frozen A1 semantic/API changes required = no
Implementation invention still required = no
```

Could two conforming implementations produce different `assessment_hash`
values from the same semantic Phase-B assessment? **NO.** Could two conforming
implementations produce different `history_id` values from the same semantic
history transition? **NO.**

READY_FOR_PHASE_B_HISTORY_IDENTITY_REREVIEW = yes

## 32. Phase B Contract Remediation XIV — wire full validation assessment into history identity

### 32.1 Authority, finding reconciliation, scope, and ownership

This documentation-only section resolves **PB-HISTORY-HASH-02**. The finding is
**CONFIRMED**. The active §31 builder accepted only
`PhaseBHypothesisAssessment` plus component rows, while the complete
`ValidationAssessment` is produced before promotion and is retained by
`HypothesisGateAssessments`. `validation_status` cannot recreate its protocol,
consumed EvidenceIds, acquisition families, condition IDs, or reasons. The
earliest divergence is therefore the old stage-16 call boundary, not JCS,
SHA-256, the history-ID preimage, duplicate suppression, or a scientific gate.

This section is the sole ACTIVE NORMATIVE amendment for the Phase-B
validation-to-history data path. It changes no production Rust, tests,
fixtures, frozen A1 API, A1 semantic identity, serialized Phase-B wire field,
CLI behavior, pipeline-stage count, `main`, or
`codex/mhi-v1-b-mechanism-evidence-integration`. It neither redesigns
validation nor adds a second persisted copy of `ValidationAssessment`.

The following ownership model is literal and exclusive:

```text
ValidationAssessment
= complete output of the validation gate.

HypothesisGateAssessments
= complete transient/deterministic collection of scientific gate outputs,
  including the complete ValidationAssessment when validation is evaluated.

PhaseBHypothesisAssessment
= durable hypothesis-level summarized result. It retains validation_status but
  does NOT duplicate the entire ValidationAssessment payload merely for
  history hashing.

HypothesisAssessmentHashView
= canonical semantic history-hash preimage. It is constructed using BOTH the
  durable PhaseBHypothesisAssessment and the full gate results required for
  canonical history identity.
```

Existing serialized assessment rows remain outputs required by their already
approved schema, but they are projections, not an alternative authoritative
owner for history construction. For the hash builder, the canonical owner of
every full gate row is exactly `HypothesisGateAssessments`; the canonical owner
of the durable summary is exactly `PhaseBHypothesisAssessment`.

The already-active aggregate is sufficient and remains exact:

```rust
pub struct HypothesisGateAssessments {
    pub contradiction_summaries: Vec<RequirementContradictionSummary>,
    pub timescale_assessments: Vec<TimescaleAssessment>,
    pub amplitude_assessments: Vec<AmplitudeAssessment>,
    pub repeatability_assessments: Vec<RepeatabilityAssessment>,
    pub identifiability_assessments: Vec<IdentifiabilityAssessment>,
    pub validation_assessment: Option<ValidationAssessment>,
}
```

It must remain alive after `assess_hypothesis(...)` returns and until history
identity has been constructed. No new type, module, global lookup, report
reread, artifact search, or hidden runner state is permitted.

The controlling §28.7 type-inventory row is amended in place as follows:

| type | serialized? | owner module | owning parent | purpose |
|---|---:|---|---|---|
| `HypothesisGateAssessments` | no | `src/mechanism/promotion.rs` | promotion and history-identity inputs | authoritative transient/full gate-result owner used by both promotion and history identity construction |

No new type is required for this P1.

### 32.2 Summary/full-result invariant and canonical source mapping

The builder first enforces this invariant using the existing
`HypothesisAssessmentHashError` surface (the history module's existing
Phase-B assessment/history error type):

```text
If gates.validation_assessment = Some(v),
assessment.validation_status MUST equal v.status.

If gates.validation_assessment = None,
assessment.validation_status MUST equal ValidationProtocolStatus::NotAssessed.
```

The exact failure is
`ValidationAssessmentStatusMismatch { summary, full_status }`; `full_status`
is `None` for the no-validation case. A required validation result missing from
the aggregate, or malformed full validation semantic input, remains a typed
error at the existing validation/history boundary only when illegal under the
active validation applicability configuration. No new error architecture is
created.

This is the controlling source map. A hash-view field must never be
reconstructed from a summary if its full authoritative gate object exists.

| HypothesisAssessmentHashView field | Canonical source object | Canonical source field | Transformation / normalization |
|---|---|---|---|
| `hypothesis_id` | `PhaseBHypothesisAssessment` | `hypothesis_id` | canonical ID string |
| `evidence_level` | `PhaseBHypothesisAssessment` | `evidence_level` | existing snake-case enum token |
| `reason_codes` | `PhaseBHypothesisAssessment` | `reason_codes` | canonical token ascending; deduplicate |
| `temporal_join_assessments` | `PhaseBHypothesisAssessment` | existing durable temporal-join rows | §31.3 stable key; duplicate rejected; no separate full temporal aggregate exists in the approved type |
| `timescale_assessments` | `HypothesisGateAssessments` | `timescale_assessments` | §31.3 stable key; duplicate rejected |
| `amplitude_assessments` | `HypothesisGateAssessments` | `amplitude_assessments` | §31.3 stable key; duplicate rejected |
| `repeatability_assessments` | `HypothesisGateAssessments` | `repeatability_assessments` | §31.3 stable key; duplicate rejected |
| `identifiability_assessments` | `HypothesisGateAssessments` | `identifiability_assessments` | §31.3 stable key; duplicate rejected |
| `contradiction_summaries` | `HypothesisGateAssessments` | `contradiction_summaries` | §31.3 stable key; duplicate rejected |
| `validation_assessment` | `HypothesisGateAssessments` | `validation_assessment` | exact nested `ValidationAssessmentHashView`; `None` is explicit JSON `null` |
| `component_assessments` | stage-15 argument | `component_assessments` | §31.3 `component_id` order; duplicate rejected |
| `source_evidence_ids` | stage-16 argument | canonical consumed source list | §32.4 sorted/unique exact-union verification |

The temporal rows remain the already-approved durable temporal-join output:
the approved `HypothesisGateAssessments` type has no separately owned temporal
field, so no full temporal object is reconstructed or moved by this amendment.
This is not a new scientific evaluation or a source traversal.

### 32.3 Validation hash view, inclusion, normalization, and absence

`ValidationAssessmentHashView` in §31.2 remains the one canonical nested
representation, with no field added or removed. Its inclusion decisions are
exhaustive:

| ValidationAssessment field | Included in hash view? | Hash-view field | Normalization |
|---|---:|---|---|
| `protocol_id` | yes | `protocol_id` | exact UTF-8 string; no display transformation |
| `status` | yes | `status` | existing snake-case enum token |
| `evidence_ids` | yes | `evidence_ids` | canonical `EvidenceId` ascending; deduplicate |
| `acquisition_family_ids` | yes | `acquisition_family_ids` | canonical serialized ID ascending; deduplicate because families are a semantic set |
| `passed_condition_ids` | yes | `passed_condition_ids` | canonical condition-ID ascending; deduplicate because conditions are a semantic set |
| `reasons` | yes | `reasons` | existing canonical validation-reason token ascending; deduplicate |

Any future validation family/condition assessment vector must declare a stable
semantic key before it can enter this view; source traversal order is forbidden.
All §31 finite-float, `-0.0`, map, duplicate-key, Option, and RFC-8785/JCS
rules remain unchanged. In particular, when
`gates.validation_assessment == None`,
`HypothesisAssessmentHashView.validation_assessment == None` and normal serde
emits the already-approved explicit JSON `null`; it must not synthesize a
`ValidationAssessment { status: NotAssessed, ... }`.

### 32.4 Canonical consumed source evidence and consistency invariants

The caller supplies `source_evidence_ids` as the already-approved canonical
consumed-evidence set. The builder independently computes and requires exact
equality to this algorithm:

```text
sorted_unique_union(
  EvidenceIds used by eligible temporal assessments,
  EvidenceIds used by direct contradiction summaries,
  EvidenceIds used by timescale assessments,
  EvidenceIds used by amplitude assessments,
  EvidenceIds used by repeatability assessments,
  EvidenceIds used by identifiability assessments,
  EvidenceIds consumed by ValidationAssessment
)
```

Only eligible evidence actually consumed by those assessments enters the
union. It excludes temporally ineligible candidates, indeterminate candidates
that did not contribute, loaded training/calibration records that were not
consumed, and the remainder of `EvidenceBundle`. Component-row EvidenceIds are
verified to be members of this union when they refer to the same derived gate
evidence; they do not add a second source or alter the union. The root vector
is canonical `EvidenceId` ascending and duplicate-free.

Every EvidenceId in `ValidationAssessmentHashView.evidence_ids` representing
consumed validation evidence MUST occur in canonical `source_evidence_ids`.
Protocol, family, and condition IDs are not EvidenceIds and never enter that
root vector. A missing validation EvidenceId is a typed
`SourceEvidenceIdsMismatch` error, not a silently accepted incomplete
provenance list.

### 32.5 Superseding history APIs and the unchanged identity algorithms

The only active hash-builder and history-stage APIs are:

```rust
pub fn build_hypothesis_assessment_hash_view(
    assessment: &PhaseBHypothesisAssessment,
    gates: &HypothesisGateAssessments,
    component_assessments: &[ComponentInterpretationAssessment],
    source_evidence_ids: &[EvidenceId],
) -> Result<HypothesisAssessmentHashView, HypothesisAssessmentHashError>;

pub fn compute_assessment_hash(
    view: &HypothesisAssessmentHashView,
) -> Result<AssessmentHash, HypothesisAssessmentHashError>;

pub fn compute_history_id(
    view: &HypothesisHistoryIdView,
) -> Result<String, HypothesisAssessmentHashError>;

pub fn update_hypothesis_history(
    previous_history: &[HypothesisHistoryEntry],
    assessment: &PhaseBHypothesisAssessment,
    gates: &HypothesisGateAssessments,
    component_assessments: &[ComponentInterpretationAssessment],
    source_evidence_ids: &[EvidenceId],
) -> Result<Vec<HypothesisHistoryEntry>, MechanismAssessmentError>;
```

`update_hypothesis_history` calls the builder, then the unchanged
`compute_assessment_hash`, constructs the unchanged `HypothesisHistoryIdView`,
calls the unchanged `compute_history_id`, and applies the unchanged semantic
duplicate suppression. `serde_jcs::to_vec`, SHA-256 encoding, the history-ID
preimage, Option/null wire semantics, map rules, float rules, and the approved
PB-HASH-01 input all remain frozen.

### 32.6 Stage 13--16 flow and production API table amendment

The pipeline remains exactly 18 stages. Only its stage 13--16 data flow is
amended:

```text
Stage 13: evaluate validation -> ValidationAssessment
gate aggregation: all gate outputs -> HypothesisGateAssessments
Stage 14: assess_hypothesis(hypothesis, eligible, &gates, config)
          -> PhaseBHypothesisAssessment; gates remains available
Stage 15: assess_components(...) -> component assessments
Stage 16: history identity/update receives assessment + gates + component
          assessments + canonical source_evidence_ids + prior history
```

The authoritative stage-16 production API row is:

| Stage | Function | Owner | Inputs | Output | Internal deterministic operations |
|---:|---|---|---|---|---|
| 16 | `update_hypothesis_history` | `src/mechanism/history.rs` | prior history; `PhaseBHypothesisAssessment`; `HypothesisGateAssessments`; `Vec<ComponentInterpretationAssessment>`; canonical `source_evidence_ids` | updated history | `build_hypothesis_assessment_hash_view`; `compute_assessment_hash`; `compute_history_id`; duplicate suppression |

No runner may consume or drop the aggregate after stage 14. Thus every one of
the 18 stages retains an exact API.

### 32.7 Fixed vectors and PB-FX regression paths

PB-HASH-01 has absent full validation input. Its canonical JCS assessment
bytes and history-ID preimage are the literal corrected `not_assessed` vectors
in §31.5. Their SHA-256 values, and therefore its `assessment_hash` and
`history_id`, are respectively
`c0d071e535bef8d14993e3a3e5b2a8209e38771655d2582e605c93096e7c6bd1` and
`4de00157ee00baa8f1ff5dc89297464668d39f08392293ca48446fa9e6127bd3`.

PB-HASH-02 is the second fixed, validated-domain cross-implementation vector.
Its complete literal durable summary, full gate aggregate, component input,
and canonical source input are:

```text
PhaseBHypothesisAssessment {
  hypothesis_id: "pb-hash-02", evidence_level: ValidatedForDomain,
  temporal_join_assessments: [], timescale_assessments: [],
  amplitude_assessments: [], repeatability_assessments: [],
  identifiability_assessments: [], contradiction_summaries: [],
  reason_codes: [ValidationSatisfied], component_assessments: [],
  validation_status: Satisfied, history: []
}
HypothesisGateAssessments {
  contradiction_summaries: [], timescale_assessments: [],
  amplitude_assessments: [], repeatability_assessments: [],
  identifiability_assessments: [],
  validation_assessment: Some(ValidationAssessment {
    protocol_id: "pb-hash-02-protocol", status: Satisfied,
    evidence_ids: ["validation.evidence.a", "validation.evidence.b"],
    acquisition_family_ids: ["family-a", "family-b"],
    passed_condition_ids: ["condition-a", "condition-b"], reasons: [Passed]
  })
}
component_assessments = []
source_evidence_ids = ["validation.evidence.a", "validation.evidence.b"]
```

Its complete normalized `HypothesisAssessmentHashView` is represented by this
exact RFC-8785/JCS UTF-8 string (no newline):

```json
{"amplitude_assessments":[],"component_assessments":[],"contradiction_summaries":[],"evidence_level":"validated_for_domain","hypothesis_id":"pb-hash-02","identifiability_assessments":[],"reason_codes":["validation_satisfied"],"repeatability_assessments":[],"source_evidence_ids":["validation.evidence.a","validation.evidence.b"],"temporal_join_assessments":[],"timescale_assessments":[],"validation_assessment":{"acquisition_family_ids":["family-a","family-b"],"evidence_ids":["validation.evidence.a","validation.evidence.b"],"passed_condition_ids":["condition-a","condition-b"],"protocol_id":"pb-hash-02-protocol","reasons":["passed"],"status":"satisfied"}}
```

The exact SHA-256 and canonical `assessment_hash` are both
`6a540a332d57d763cefaa05ba46a663ba97e019649df1d531e8c430da047d4ec`.
Its exact history-ID view/JCS UTF-8 string is:

```json
{"assessment_hash":"6a540a332d57d763cefaa05ba46a663ba97e019649df1d531e8c430da047d4ec","hypothesis_id":"pb-hash-02","new_level":"validated_for_domain","prior_level":"experimentally_supported"}
```

Its exact SHA-256 and `history_id` are both
`0f4f48e1bd076897520a1e6a43a870cf22bec202ac272fc3ab4d3fea707cd70c`.
These values were computed from the above exact JCS bytes using the locked
`serde_jcs = 0.2.0` semantic procedure and SHA-256; they are not placeholders.

PB-FX-09 continues to pass: its no-validation path supplies the aggregate and
uses the already-approved `NotApplicable` full assessment representation when
the validation evaluator ran, or the `None`/`NotAssessed` invariant only when
validation was not evaluated. It retains its exact absence/null behavior and
unrelated fixture content.

PB-FX-10 now traces the executable path:

```text
accepted source artifacts -> eligible Validation-role evidence
-> ValidationAssessment -> HypothesisGateAssessments.validation_assessment
-> build_hypothesis_assessment_hash_view(..., &gates, ...)
-> ValidationAssessmentHashView -> assessment_hash -> history_id
-> persisted HypothesisHistoryEntry
```

Each arrow is owned by stages 3/7/13, gate aggregation, stage 16's explicit
history API, or the persisted history writer respectively. PB-FX-10 compares
runtime-derived production hashes with its fully specified semantic view; it
does not retain a placeholder digest and does not reconstruct validation from
`validation_status`.

### 32.8 Tests, supersession, and final audit

The exact required implementation tests are:

```text
phase_b_assessment_hash_includes_full_validation_assessment
phase_b_validation_summary_must_match_full_assessment
phase_b_validated_source_evidence_ids_include_consumed_validation_evidence
phase_b_hash_builder_accepts_gate_assessments
phase_b_hash_builder_validation_none_matches_not_assessed_summary
phase_b_assessment_hash_rfc8785_validated_vector
phase_b_fx10_validation_payload_reaches_history_hash
phase_b_fx10_history_hash_matches_canonical_validation_view
```

They supplement, rather than replace, every approved PB-HASH-01, PB-FX-09,
and history duplicate-suppression test. Requirement-to-test mapping is one to
one: full payload, summary invariant, source evidence, builder API, absence,
fixed vector, E2E propagation, and E2E canonical hash respectively.

| occurrence | classification | controlling rule |
|---|---|---|
| §31.2 source-map row `.validation_status / the active validation result` | SUPERSEDED | §32.2 separates the durable summary from the full gate-owned result |
| §31.3 `source_evidence_ids` union that adds component-row IDs as an independent source | SUPERSEDED | §32.4 gate/validation consumed-evidence union and component-subset verification |
| §31.4 two-argument `build_hypothesis_assessment_hash_view` | SUPERSEDED | §32.5 four-input builder |
| §31.6 previous-history plus assessment/component-only stage-16 flow | SUPERSEDED | §32.5--32.6 explicit gates and source IDs |
| §28.4 three-input `update_hypothesis_history` | SUPERSEDED | §32.5 explicit gates and source IDs |
| §28.5 stage-16 row without gates/source IDs | SUPERSEDED | §32.6 stage-16 row |
| §28.6 PB-FX history descriptions that pass only component rows | SUPERSEDED | §32.7 explicit aggregate flow |
| §28.7 inventory description limiting `HypothesisGateAssessments` to promotion input | SUPERSEDED | §32.1 inventory amendment |
| any active instruction reconstructing validation from `validation_status` | SUPERSEDED | §32.1--32.3 |
| all other §31 hash schemas, ordering, JCS, float, null, hash, history-ID, and duplicate rules | ACTIVE NORMATIVE | unchanged except for the explicit input path |

```text
Undefined normative types = 0
Undefined normative owners = 0
Unspecified Phase B algorithms = 0
Unspecified scientific thresholds/units = 0
Unspecified compatibility decisions = 0
Normative contradictions = 0
Fixture-to-real-schema contradictions = 0
Fixture-to-source-route contradictions = 0
Fixture-to-wire-contract contradictions = 0
Incomplete normative positive fixtures = 0
Unmapped active normative types = 0
Pipeline stages without exact API = 0
Serialized active types without exact wire shape = 0
Competing active production-order definitions = 0
Unmapped acceptance criteria = 0
Missing exact test names = 0
Assessment-hash fields with unspecified inclusion/exclusion = 0
Assessment-hash vectors with unspecified ordering = 0
Assessment-hash Option/null semantics unspecified = 0
Assessment-hash float semantics unspecified = 0
Hash-view fields with unavailable canonical source = 0
Validation hash fields without canonical data path = 0
Active hash-builder APIs lacking full validation input = 0
Canonical fixed hash test vectors = 2
Validated fixed hash test vectors = 1
Fixed hash vectors whose literal JCS bytes do not match documented digest = 0
Fixed history vectors whose literal JCS preimage does not match documented history ID = 0
PB-HASH-01 mathematical inconsistencies = 0
PB-HASH-02 mathematical inconsistencies = 0
Frozen A1 semantic/API changes required = no
Implementation invention still required = no
```

Can conforming implementations disagree on `assessment_hash`? **NO.** Can
they disagree on `history_id`? **NO.** Can they disagree on where full
validation data used by the hash comes from? **NO.**

READY_FOR_PHASE_B_VALIDATION_TO_HISTORY_REREVIEW = yes
