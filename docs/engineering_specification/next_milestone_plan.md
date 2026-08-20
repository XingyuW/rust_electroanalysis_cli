# MHI V1 Phase E — Independent Scientific Validation and Compatibility Certification

**Status:** planning only; implementation is not authorized by this document.

**Repository:** `/Users/xingyuwang/ProjectOngoing/rust_electroanalysis_cli`

**Phase-E initialization baseline:** current synchronized `main`

**`PHASE_E_BASELINE_MAIN_SHA`:** `23a6710a869bbaccab2bce3f2bfa67af46380755`

**Phase-D provenance release tag:** `ism-mechanism-health-v1-d-complete`

**Phase-D provenance release SHA:** `2316fb1d076d47ac14d3b3c10c55654feb1ffb54`

**Planning workflow:** documentation-only planning commits directly to `main`; no
planning or review branch exists.

**Temporary implementation branch after plan approval only:**
`codex/mhi-v1-e-independent-validation`

**Milestone name:** MHI V1 Phase E

This document defines the next scientific/software milestone after Phase D. It
does not authorize changes to Rust source, configuration, schemas, fixtures, or
existing documentation other than this plan.

## 1. Milestone decision and scientific motivation

### 1.1 Limitation remaining after Phase D

Phase D made the existing scientific state public, deterministic, and
auditable. It projects validated Phase B mechanism evidence and Phase C sensor
health assessments into a certified report bundle without recomputing science.
That closes the communication gap, but not the external-validation gap.

The remaining limitation is that the repository cannot yet quantify whether
those conclusions reproduce on evidence that was not used to configure,
calibrate, fit, initialize, or assess them. In particular, Phase D does not:

- prove that a mechanism assessment generalizes across independent acquisition
  families, sensors, matrices, temperatures, or experimental campaigns;
- measure sensor-health alert performance against independently established
  reference outcomes;
- detect training/validation leakage across lineage or acquisition families;
- calculate validation coverage, exclusion rates, confusion matrices, false
  alarm or miss rates, confidence intervals, or domain-stratified performance;
- turn synthetic tests, model agreement, a fitted timescale, or a rendered
  report into physical validation; or
- certify that all supported legacy schemas, CLI routes, and public artifacts
  remain compatible as one release candidate.

Phase E therefore adds a separate validation and certification layer. It
consumes existing artifacts plus a predeclared protocol and independent
reference manifest. It never mutates a Phase B mechanism conclusion, a Phase C
health status, or a Phase D report. Its result answers a narrower question:
“How did the frozen assessment behavior perform on the declared validation
cohort, under the declared acceptance rules?”

### 1.2 Scientific claim boundary

Phase E may report `meets_protocol`, `does_not_meet_protocol`, or
`indeterminate` for a validation endpoint. These are validation outcomes, not
new mechanism evidence levels, health statuses, or causal findings.

The following ceilings remain binding:

- timescale agreement alone is not physical mechanism identity;
- model residuals alone are not sensor-failure evidence;
- same-source, shared-family, unknown-lineage, synthetic, or training evidence
  is not independent validation;
- an absent denominator, reference outcome, uncertainty, or required domain
  stratum produces an unavailable/indeterminate metric, never zero or pass;
- a Phase E result never upgrades `Hypothesized` to
  `ExperimentallySupported`/`ValidatedForDomain` and never changes a Phase C
  dimension status; and
- real-domain validation claims require owner-approved physical validation
  data. Passing synthetic fixtures certifies software behavior only.

### 1.3 Confirmed repository cause and expected behavior

This is an intentionally deferred capability, not a regression in Phase D.
Repository inspection confirms the current production path ends at
`CommandSpec::ReportRender` → `runners::report` → canonical artifact readers →
`PublicReportProjection` → Phase D writers. The projection consumes one
declared artifact set and has no cohort, reference-outcome, leakage, metric, or
validation-acceptance model. The existing `model validate` path reads a
`ValidationManifest` and evaluates `ism_model_analysis`; it does not validate
Phase B mechanism or Phase C health behavior and must not be repurposed.

The earliest missing boundary is therefore after current mechanism/health
artifacts are serialized and before any cross-cohort scientific claim: there is
no canonical MHI validation dataset, protocol, evaluator, or result artifact.
Expected Phase E behavior is to add that separate boundary while leaving the
current assessment and rendering paths untouched.

## 2. Current capability baseline

All statements in this section describe the tagged baseline at
`2316fb1d076d47ac14d3b3c10c55654feb1ffb54`.

### 2.1 Mechanism evidence

- `mechanism_analysis` schema 4 serializes Phase B hypothesis definitions,
  assessments, temporal joins, timescale assessments, amplitude,
  repeatability, identifiability, contradictions, validation-protocol status,
  component interpretation, reason codes, and deterministic history.
- Evidence is represented by the schema-1 `EvidenceBundle`, including typed
  targets, sources, availability, direction, strength authority, validity,
  quantities, uncertainty, threshold provenance, pairwise independence, and
  exact producer-backed timescale pair covariance.
- Phase B requires explicit hypotheses and gates. It preserves missing,
  contradictory, scope-incompatible, temporally incompatible, and
  non-independent evidence rather than filling gaps.
- Independent confirmation depends on durable lineage and known acquisition
  families. Unknown lineage or family membership cannot become independent.
- Legacy mechanism schemas 1–3 remain readable, but their legacy hypotheses
  are not silently reinterpreted as Phase B assessments.
- The permanent Phase B traceability target records 72 passing mechanism
  evidence tests at the tagged baseline.

### 2.2 Sensor health

- `health_assessment` schema 4 contains exactly one Phase C assessment for each
  of the nine ordered health dimensions, with evidence state, status,
  interpretation category, causal ceiling, reason codes, consumed/excluded
  evidence IDs, source artifact IDs, and an embedded evidence bundle.
- Overall status, interpretation categories, and causal status are derived
  deterministically from the serialized dimension assessments and validated at
  the artifact boundary.
- Data-quality insufficiency, indeterminacy, contradictory evidence, and an
  adequate negative finding remain distinct states.
- Baseline comparison, signal, calibration, dynamic response, reference
  stability, model consistency, observability/identifiability, uncertainty,
  and data sufficiency are explicit; optional-source absence cannot produce a
  healthy finding.
- Schema-3 health assessments remain readable through the legacy route but do
  not contain or synthesize the nine-dimension Phase C payload.
- The Phase C traceability audit covers 110 mandatory contracts; its recorded
  targeted artifact, Phase B, and Phase C suites pass at the tagged baseline.

### 2.3 Reporting

- `electroanalysis report render` is the single Phase-D-certified public route.
  It requires mechanism and health artifacts and optionally accepts lineage,
  EIS, transient, calibration, signal, estimation, and model artifacts.
- The route reads canonical artifacts, checks schema/scope compatibility, and
  performs a projection only. It does not discover raw data, refit models,
  reassess evidence, resolve lineage, or manufacture unavailable values.
- The certified bundle contains a schema-1 public summary, schema-1 render
  manifest, Markdown report, seven deterministic CSV table types, and up to
  eleven SVG/PNG figure types.
- Publication is staged and atomic. Selection, unavailability, legacy notices,
  compatibility, and lineage limitations are explicit. Repeated rendering of
  the same accepted inputs is deterministic.
- The Phase D acceptance matrix maps 73 requirements/criteria/tests and records
  all 73 as implemented and passing.

### 2.4 Provenance

- `AnalysisProvenance` records software version, input/configuration paths and
  SHA-256 hashes, generation time, and optional Git commit.
- `ArtifactLineageState` distinguishes authoritative `Known` identity and
  dependencies from `LegacyUnknown`; readers never fabricate historical
  identity.
- Known `ArtifactIdentity` includes artifact kind/schema, producer version,
  experiment scope, sensor/channel scope, acquisition families, and semantic
  SHA-256. Direct dependency roles are typed and deterministically ordered.
- `ArtifactLineageCatalog` supports deterministic closure, missing-ancestor,
  cycle, and inconsistent-root reporting. Unknown lineage cannot satisfy an
  independence requirement.
- Evidence records retain artifact/field sources, experiment scope, temporal
  support, threshold authority, derivation authority, and exact IDs needed to
  reconstruct a conclusion.

### 2.5 Reproducibility

- Rust is pinned to `1.97.0`; CI uses `Cargo.lock` and runs formatting, clippy,
  all tests, and a release build on Linux and macOS.
- Scientific collections use stable ordering, typed units, finite-value
  validation, canonical semantic hashes, and explicit residual sign contracts.
- Stochastic workflows expose deterministic seeds where applicable.
- A0/A1/B/C/D compatibility fixtures cover legacy/current schemas, artifact
  kinds, identity, lineage, covariance, missing data, and negative paths.
- Phase D has sealed literal fixtures, exact reader routes, deterministic
  numeric spelling, immutable source-artifact checks, repeat rendering checks,
  and atomic-publication failure tests.

## 3. Scientific objectives

Phase E is complete only when every objective below has a committed test or an
owner-approved validation record. Scientific thresholds are required protocol
inputs; implementation defaults are forbidden.

| ID | Objective | Measurable acceptance criterion |
|---|---|---|
| E-SCI-01 | Predeclare the validation question and cohort before scoring. | Every run references one schema-1 protocol hash and one schema-1 dataset ID plus canonical source reference. Every endpoint declares its target, cohort role, domain, minimum eligible independent families, metric, confidence rule, and pass/fail threshold. Missing declarations fail before evaluation. A claim requested as physically validated requires at least two eligible independent acquisition families. |
| E-SCI-02 | Prove separation between development and validation evidence. | One hundred percent of consumed records receive a deterministic overlap classification using artifact lineage, experiment scope, and acquisition families. Any known overlap in a holdout endpoint fails that endpoint. Unknown separation makes it indeterminate. Zero overlap is permitted for a passing holdout endpoint. |
| E-SCI-03 | Quantify mechanism reproducibility without changing Phase B. | For every declared hypothesis, report eligible artifact count, eligible independent-family count, support/contradiction/not-assessed counts, support fraction, contradiction fraction, and 95% Wilson interval where defined. The protocol result passes only when all declared minimums/thresholds are met and no protocol-defined critical contradiction is present. |
| E-SCI-04 | Quantify health classification performance against an independent reference. | For every declared dimension/aggregate endpoint, emit the exact confusion matrix, evaluable denominator, coverage, indeterminate rate, data-quality-insufficient rate, sensitivity, specificity, false-positive rate, false-negative rate, balanced accuracy, and 95% Wilson intervals for each binomial proportion with a valid denominator. Balanced accuracy is the arithmetic mean of defined sensitivity and specificity and has no interval in V1. Zero-denominator metrics are unavailable. |
| E-SCI-05 | Expose domain generalization rather than pooling it away. | Every scored endpoint emits overall and protocol-required strata (for example sensor design, sensor, analyte, matrix, temperature band, and acquisition campaign). A required empty or underpowered stratum is indeterminate and cannot be hidden by an aggregate pass. |
| E-SCI-06 | Preserve reference-outcome authority and uncertainty. | Every reference label records method, unit where applicable, source hash/identity, assessor or instrument authority, blinding state, and uncertainty/limitations. A label lacking a protocol-required authority field is excluded with a typed reason. |
| E-SCI-07 | Separate software validation from physical scientific validation. | Synthetic fixtures may satisfy software gates only. A real-domain validation status requires at least one owner-approved, protocol-conforming physical holdout cohort for every claim named in the release scope. Claims without such a cohort remain `not_physically_validated`. |
| E-SCI-08 | Make every result reconstructible. | For every numerator, denominator, exclusion, and validation outcome, the report lists the contributing record IDs and source artifact identities. Counts reconstructed from those IDs must exactly equal the serialized metrics. |

The protocol may demand stronger sampling, replication, confidence, or domain
requirements. It may not weaken the independence, lineage, missing-data, or
claim-ceiling rules above.

### 3.1 Frozen V1 metric definitions

All counts are nonnegative integers and all rates are dimensionless. A record
appears at most once in a given endpoint/stratum denominator. Required strata
may overlap only when the protocol explicitly defines them as separate views;
their counts are never summed to construct the overall result.

For a mechanism endpoint, let `n` be the count of eligible, non-duplicate
records after compatibility, scope, role, independence, reference, and leakage
checks. Let `s`, `c`, and `u` be the mutually exclusive counts whose serialized
Phase B result is accepted support, protocol-defined critical contradiction,
or not assessed/other, respectively. The invariant is `n = s + c + u`.

```text
support_fraction       = s / n
contradiction_fraction = c / n
not_assessed_fraction  = u / n
```

For a health endpoint, predicted-positive/negative and
reference-positive/negative sets are declared by the protocol. An eligible
assessment contributes exactly one of `TP`, `TN`, `FP`, or `FN`. Phase C
`Indeterminate` and `DataQualityInsufficient` results do not enter that
confusion matrix; they remain in the declared cohort denominator and are
reported separately.

```text
evaluable                  = TP + TN + FP + FN
coverage                   = evaluable / declared_eligible_cohort
indeterminate_rate         = indeterminate / declared_eligible_cohort
data_quality_insufficient_rate
                           = data_quality_insufficient / declared_eligible_cohort
sensitivity                = TP / (TP + FN)
specificity                = TN / (TN + FP)
false_positive_rate        = FP / (FP + TN)
false_negative_rate        = FN / (FN + TP)
balanced_accuracy          = (sensitivity + specificity) / 2
```

`balanced_accuracy` is available only when both sensitivity and specificity
are available. It has no confidence interval in V1. No continuity correction,
weighting, imputation, pooling across missing strata, or replacement of an
undefined rate by zero is permitted.

Every binomial fraction `p = x / n`, `n > 0`, uses the two-sided Wilson 95%
interval registered as `wilson_95_v1`, with
`z = 1.959963984540054`:

```text
denominator = 1 + z^2 / n
center      = (p + z^2 / (2n)) / denominator
half_width  = z / denominator
              * sqrt(p(1 - p) / n + z^2 / (4n^2))
lower       = max(0, center - half_width)
upper       = min(1, center + half_width)
```

Protocol acceptance is a closed ordered list of `AcceptanceRuleV1` records.
Each record names a metric, a comparison target (`point_estimate`,
`lower_confidence_bound`, `upper_confidence_bound`, or integer `count`), a
comparator (`greater_than_or_equal` or `less_than_or_equal`), and a finite
threshold. All required rules use logical AND. An unavailable target makes the
endpoint `indeterminate`; a defined rule that evaluates false makes it
`does_not_meet_protocol`. No implicit tolerance or rounding is used for the
comparison; public formatting occurs only after evaluation.

## 4. Technical objectives

### 4.1 Module plan

All paths in this section are proposed implementation paths. They must not be
created until the review gate in section 8 passes.

| Module | Responsibility |
|---|---|
| `src/mhi_validation/mod.rs` | Narrow facade for protocol validation and the pure evaluation API. No filesystem access in the evaluator. |
| `src/mhi_validation/error.rs` | Closed typed error vocabulary for protocol, dataset, compatibility, overlap, reference, metric, and publication failures. |
| `src/mhi_validation/protocol.rs` | Parse and validate the closed TOML protocol schema; reject absent scientific thresholds and unknown fields. |
| `src/mhi_validation/reader.rs` | Canonical `domain::read_artifact` boundary for the dataset manifest and referenced mechanism/health artifacts; verify expected kind, schema, identity, semantic hash, and path containment. |
| `src/mhi_validation/partition.rs` | Classify development/validation/holdout membership and leakage from declared roles, lineage, experiment scopes, and acquisition families. No filename or timestamp inference. |
| `src/mhi_validation/mechanism.rs` | Project already-serialized Phase B outcomes into declared validation endpoints and exact counts. It must not call the Phase B assessor. |
| `src/mhi_validation/health.rs` | Compare already-serialized Phase C outcomes with independently declared reference endpoints. It must not call the Phase C assessor. |
| `src/mhi_validation/statistics.rs` | Deterministic count/rate calculations and 95% Wilson intervals; no imputation, weighting, smoothing, or unregistered metric. |
| `src/mhi_validation/assessment.rs` | Apply only explicit protocol acceptance rules and return `meets_protocol`, `does_not_meet_protocol`, or `indeterminate`. |
| `src/mhi_validation/output.rs` | Write the typed report, execution manifest, Markdown summary, and CSV tables to a private staging directory; validate checksums before atomic publication. |
| `src/results/mhi_validation.rs` | Own the two new `VersionedArtifact` payloads and their validation invariants. |
| `src/validation_config.rs` | Clap-neutral protocol/options types and closed wire enums. |
| `src/runners/mhi_validation.rs` | Orchestrate read → compatibility/leakage checks → pure evaluation → atomic publication. |
| `src/cli.rs`, `src/main.rs` | Add the single certified route described below, without changing any existing command. |
| `src/results/artifact_contracts.rs`, `src/domain/artifact.rs` | Register the two additive artifact kinds and exact schema-1 contracts. No other contract changes. |

Existing `src/model_validation.rs` and the `ism_model_validation` artifact are
model-workflow validation and remain unchanged. Phase E must not rename or
silently repurpose them.

### 4.2 Proposed certified CLI and APIs

The sole new certified route is proposed as:

```text
electroanalysis validation run \
  --protocol validation_protocol.toml \
  --dataset mhi_validation_dataset.schema1.json \
  --output-dir validation_output \
  [--overwrite]
```

The dataset manifest, not arbitrary directory discovery, declares every input
artifact. The CLI must not accept raw electrochemical data or infer reference
labels.

The minimum library surface is:

```text
MhiValidationProtocolV1::from_toml(&str) -> Result<Self, MhiValidationError>
MhiValidationProtocolV1::validate(&self) -> Result<(), MhiValidationError>
MhiValidationDatasetV1::validate(&self) -> Result<(), ArtifactError>
ValidationInputs::read(protocol, dataset_path) -> Result<Self, MhiValidationError>
evaluate_mhi_validation(
    protocol: &MhiValidationProtocolV1,
    inputs: &ValidationInputs,
) -> Result<MhiValidationReportV1, MhiValidationError>
MhiValidationReportV1::validate(&self) -> Result<(), ArtifactError>
run_mhi_validation(options: MhiValidationRunOptions) -> Result<(), RunnerError>
```

`evaluate_mhi_validation` is deterministic and filesystem-free. The reader and
runner own I/O. The production writer uses `domain::write_artifact`; direct
ad-hoc JSON parsing of scientific artifacts is prohibited.

The complete production execution path is frozen as:

1. Clap parses `validation run`; unknown options and missing required paths
   fail before the runner.
2. `MhiValidationRunOptions` normalizes paths and rejects invalid
   input/output combinations without opening scientific inputs.
3. `protocol::from_toml` loads the exact protocol bytes, calculates their
   SHA-256, and validates the closed schema and every scientific rule.
4. `reader::ValidationInputs::read` uses `domain::read_artifact` for the dataset
   and every declared mechanism/health path, then checks kind, allowed schema,
   expected identity/hash, relative path containment, and exact protocol hash.
5. `partition` classifies cohort role, scope compatibility, lineage closure,
   acquisition-family independence, reference independence, and development /
   holdout leakage. It produces an explicit eligible/excluded/error decision
   for every declared endpoint record.
6. `mechanism` and `health` project only serialized Phase B/C outcomes into the
   mutually exclusive count sets defined in section 3.1.
7. `statistics` derives registered rates and Wilson intervals; `assessment`
   evaluates the ordered explicit acceptance rules.
8. `MhiValidationReportV1::validate` reconstructs counts from IDs, checks every
   invariant, and constructs lineage from the actually consumed dependencies.
9. `output` writes all requested files to a sibling private staging directory,
   rereads the scientific artifact canonically, verifies every path/checksum,
   and publishes the complete directory by atomic rename.
10. Any error before publication leaves no final bundle. Managed overwrite
    restores the previous complete bundle if publication fails; unmanaged
    entries and symlinks are rejected unchanged.

Error precedence is deterministic: invalid CLI option/path shape → protocol
parse/schema failure → dataset schema or referenced-path containment failure →
source kind/schema failure → expected identity/hash mismatch → missing required
artifact/reference → scientific incompatibility/exclusion →
overlap/unknown-separation endpoint outcome → unavailable metric → failed
acceptance rule → output failure. Hard errors stop the run; exclusions and
scientific outcomes are serialized. The same condition cannot be both silently
excluded and returned as a hard error.

### 4.3 Schema definitions

All schema-1 objects are closed (`deny_unknown_fields` or equivalent), contain
finite numeric values only, use snake-case tokens, and use deterministically
sorted duplicate-free collections where order is not scientifically
meaningful.

#### `MhiValidationProtocolV1` — TOML schema 1

| Field | Required content |
|---|---|
| `schema_version` | Exactly `1`. |
| `protocol_id` | Stable nonempty owner-assigned ID. |
| `title` | Human-readable protocol title. |
| `registration_reference` | Nonempty URI/registry/document reference proving the frozen protocol authority. |
| `target_domain` | Explicit analyte, matrix, sensor design/type, temperature bounds, and other protocol-required scope axes; an axis may be explicitly unrestricted but not omitted silently. |
| `development_roles` | Roles treated as development/training evidence. |
| `validation_roles` | Roles eligible for validation and holdout evaluation. |
| `mechanism_endpoints` | Hypothesis ID, accepted input evidence levels, minimum independent-family count, required strata, support/contradiction thresholds, and critical contradiction policy. |
| `health_endpoints` | Dimension or aggregate target, predicted positive/negative status sets, reference positive/negative label sets, minimum denominator, required strata, and metric thresholds. |
| `statistics` | Exactly the registered interval method (`wilson_95_v1`) and explicit missing/indeterminate handling. |
| `release_scope` | Claims for which physical-validation certification is requested. |

There are no scientific defaults for endpoints, thresholds, family counts,
strata, positive classes, or release claims.

#### `MhiValidationDatasetV1` — artifact kind `mhi_validation_dataset`, schema 1

| Field | Required content |
|---|---|
| `schema_version`, `artifact_kind` | Exactly `1` and `mhi_validation_dataset`. |
| `dataset_id` | Stable nonempty ID. |
| `protocol_sha256` | SHA-256 of the exact protocol bytes used to construct the manifest. |
| `records` | Canonically ordered `ValidationRecordV1` values. |
| `lineage` | Known aggregate identity and direct dependencies when authoritative; otherwise explicit `LegacyUnknown`. |
| `provenance` | Manifest input/config hashes, software version, generation time, and optional Git commit. |
| `warnings` | Typed, ordered construction warnings; warnings cannot waive invalidity. |

Each `ValidationRecordV1` contains exactly: record ID, cohort role, relative
mechanism-artifact path or null, relative health-artifact path or null,
expected artifact kind/schema and source-file SHA-256 for every non-null path,
expected artifact ID/semantic SHA-256 when the source has Known lineage,
declared experiment/sensor/domain keys, declared acquisition-family identity,
reference endpoints, and reference provenance. Absolute paths and paths that
escape the manifest directory are invalid.

Each reference endpoint contains exactly: endpoint ID, endpoint kind,
hypothesis ID or health dimension/aggregate target, reference label/value,
unit when numerical, method ID/version, source identity/hash, blinding state,
uncertainty or an explicit unavailable reason, limitations, and the acquisition
family that generated the reference. A reference derived from the assessed
artifact is not independent and is ineligible for a holdout pass.

#### `MhiValidationReportV1` — artifact kind `mhi_validation_report`, schema 1

| Field | Required content |
|---|---|
| `schema_version`, `artifact_kind` | Exactly `1` and `mhi_validation_report`. |
| `report_id` | Semantic ID derived from protocol hash, the canonical dataset source reference, and canonical consumed source references: Known artifact ID/semantic SHA-256 or explicit legacy source fingerprint/file SHA-256; no clock, fabricated identity, or output path. |
| `protocol` | Protocol ID, schema, SHA-256, and registration reference. |
| `dataset` | Dataset ID, schema, source-file SHA-256, and its tagged Known identity/semantic SHA-256 or explicit LegacyUnknown fingerprint/reason. |
| `compatibility` | Per-input kind/schema/identity/hash result and aggregate status. |
| `cohorts` | Declared, eligible, excluded, development, validation, and holdout counts. |
| `leakage_assessment` | Per-record separation status, shared ancestors/families, unknowns, and aggregate status. |
| `mechanism_results` | One result per declared mechanism endpoint and required stratum, with exact counts, Wilson interval, source IDs, exclusions, limitations, and protocol outcome. |
| `health_results` | One result per declared health endpoint and required stratum, with confusion matrix, coverage and error metrics, Wilson intervals, source IDs, exclusions, limitations, and protocol outcome. |
| `exclusions` | Canonically ordered record/endpoint/reason ledger; no silent row loss. |
| `release_claims` | Per requested claim: `physically_validated`, `software_validated_only`, `does_not_meet_protocol`, or `indeterminate`, plus exact supporting endpoint IDs. |
| `overall_status` | `meets_protocol`, `does_not_meet_protocol`, or `indeterminate`; any required indeterminate endpoint prevents an overall pass. |
| `lineage`, `provenance`, `warnings` | Direct dependencies for every actually consumed artifact; deterministic software/Git/configuration identity with no wall clock or output path; typed warnings. |

All rates store numerator and denominator. Every interval stores method,
confidence level, lower, and upper values. A metric with no valid denominator
stores an unavailable reason and no numeric value or interval.

`ValidationProvenanceV1` contains exactly software version, optional Git
commit, protocol SHA-256, a tagged dataset source reference, and sorted
`SourceReferenceV1` records. A source reference is a tagged Known variant with
artifact kind/ID/semantic SHA-256 or a LegacyUnknown variant with artifact kind,
schema, `LegacySourceFingerprint`, source-file SHA-256, and unknown-lineage
reason. It contains no generation clock, hostname, absolute path, fabricated
ID, or output path, so the scientific report is byte-reproducible. Operational
timing, if required, belongs only to the execution manifest and is excluded
from scientific semantic identity.

### 4.4 Output artifacts

One successful run atomically publishes:

```text
OUTPUT/
  mhi_validation_report.schema1.json
  validation_execution_manifest.schema1.json
  validation_summary.md
  tables/
    cohort_coverage.csv
    leakage_assessment.csv
    mechanism_validation.csv
    health_validation.csv
    exclusion_ledger.csv
    compatibility_matrix.csv
```

The execution manifest records input identities/hashes, protocol hash,
generated paths and SHA-256 checksums, availability, and software/Git identity.
It contains no scientific result not already present in the validation report.
Its closed schema contains exactly `schema_version=1`,
`output_kind="mhi_validation_execution_manifest"`, report ID, protocol hash,
tagged dataset source reference, generated-file records, publication mode, software version,
optional Git commit, and optional operational generation timestamp. Generated
file records sort by relative path and contain path, output kind, byte length,
and SHA-256.
No figure contract is introduced in Phase E; validation plots are out of scope
until their statistical and accessibility requirements are separately
reviewed.

### 4.5 Compatibility objectives

- Existing CLI parsing and behavior, including `report render`, is unchanged.
- Existing mechanism schema 4, health schema 4, public-summary schema 1,
  render-manifest schema 1, evidence bundle schema 1, lineage catalog schema 1,
  and model-validation schema 1 are byte/semantic compatibility baselines.
- Only mechanism schema 4 and health schema 4 are scientifically scoreable in
  Phase E. Readable legacy mechanism schemas 1–3 and health schema 3 may be
  listed in the exclusion ledger but cannot be upgraded or scored as Phase B/C.
- Wrong kinds, unsupported future schemas, identity/hash mismatch, missing
  required files, and path escape are typed hard failures before publication.
- The two new artifact kinds are additive. Every exhaustive `ArtifactKind`
  match must be updated without changing existing wire tokens.
- No new dependency may alter existing numerical results. A statistics
  dependency, if proposed, requires locked-version review and fixed-vector
  parity tests; a small reviewed in-tree Wilson implementation is preferred.

## 5. Scope

### IN SCOPE

- The schema-1 validation protocol, validation dataset artifact, validation
  report artifact, and execution manifest defined above.
- Canonical reading of existing Phase B schema-4 mechanism and Phase C
  schema-4 health artifacts referenced by a closed manifest.
- Deterministic cohort eligibility, lineage/family leakage checks, exact
  counts, registered rates, Wilson 95% intervals, stratification, exclusions,
  and protocol outcomes.
- Independent reference endpoints with explicit authority, blinding,
  uncertainty, domain, and provenance.
- A single additive CLI route and atomic JSON/Markdown/CSV validation bundle.
- Schema/artifact migration fixtures, full regression validation, and
  independent scientific and architecture review.
- At least one owner-approved physical holdout cohort per release claim that is
  to be described as physically validated.

### OUT OF SCOPE

- Any Phase B hypothesis reassessment, promotion, threshold change, or new
  mechanism mapping.
- Any Phase C health reassessment, new dimension, threshold change, causal
  diagnosis, alarm action, maintenance recommendation, or prognosis.
- Changes to Phase D report semantics, paths, tables, figures, public summary,
  or manifest.
- Raw CSV/XLSX/CHI ingestion, experimental data cleaning, model fitting,
  calibration, state estimation, or source artifact production in the Phase E
  route.
- Automatic train/test splitting, inferred labels, inferred acquisition
  families, inferred sensor/domain metadata, or filename-based cohort roles.
- Bayesian model selection, structural/model-form uncertainty, new covariance
  approximations, high-fidelity Nernst–Planck transport, or physical mechanism
  discovery.
- Online monitoring, alert dispatch, fleet telemetry, database/service APIs,
  GUI work, or cloud deployment.
- Validation figures, performance optimization, Windows enablement, cleanup of
  unrelated technical debt, or standardization of every repository config
  migration policy.

## 6. Data model impact

### 6.1 New artifacts

| Artifact | Schema | Owner | Migration at introduction |
|---|---:|---|---|
| `mhi_validation_dataset` | 1 | `src/results/mhi_validation.rs` | New artifact; no legacy form. |
| `mhi_validation_report` | 1 | `src/results/mhi_validation.rs` | New artifact; no legacy form. |
| `validation_execution_manifest` | 1 | Phase E output writer; presentation manifest, not a `VersionedArtifact` | New output; no legacy form. |

The validation protocol is closed TOML configuration schema 1, not a scientific
result artifact. Its exact byte hash is stored in both new artifacts and the
execution manifest.

### 6.2 Existing schema changes

No existing artifact payload or schema version changes in Phase E. The only
shared data-model change is adding `MhiValidationDataset` and
`MhiValidationReport` variants to `ArtifactKind` and registering their
`VersionedArtifact` contracts.

When all consumed sources have Known lineage, the new report lineage directly
depends on the dataset artifact and every mechanism/health artifact actually
used in a numerator, denominator, exclusion, or leakage decision. If any such
source has `LegacyUnknown`, the report lineage is also `LegacyUnknown`; its
`SourceReferenceV1` fingerprint remains explicit, and no dependency ID is
fabricated. Merely declared but unread optional inputs are not dependencies.
Protocol identity is recorded as configuration SHA-256 rather than a fabricated
artifact ID.

### 6.3 Migration strategy

1. Introduce both new artifact kinds at current schema 1 with no legacy schema
   list and required `artifact_kind`.
2. Preserve every existing kind token, schema table, Serde default, reader,
   writer, and CLI route unchanged.
3. Accept only exact schema-1 Phase E artifacts. Reject future versions until a
   separately reviewed migration contract exists.
4. Read existing mechanism/health artifacts through `domain::read_artifact`.
   Schema-4 artifacts may be scored; readable legacy artifacts receive a typed
   exclusion and never an in-memory upgrade to missing Phase B/C content.
5. Add literal schema-1 round-trip, wrong-kind, missing-kind, future-schema,
   unknown-field, nonfinite, duplicate-ID, path-escape, and identity/hash
   mismatch fixtures.
6. Before any future schema-2 work, archive canonical schema-1 fixtures and
   define field-by-field preservation, rejection, and semantic-hash behavior.

Rollback is additive: removing the Phase E command/modules and the two new
artifact-kind registrations returns to the exact Phase D behavior; no existing
artifact needs rewriting.

## 7. Validation strategy

### 7.1 Unit tests

- Closed protocol parsing: required fields, no scientific defaults, enum wire
  tokens, finite thresholds, valid probability ranges, duplicate endpoints,
  and contradictory positive/negative label sets.
- Dataset invariants: canonical ordering, unique IDs, relative path
  containment, expected identity/hash fields, reference authority, and exact
  endpoint binding.
- Leakage classification: disjoint/shared/unknown lineage, experiment scope,
  and acquisition families; aggregate scopes; missing ancestors; cycles; and
  same-source reference outcomes.
- Exact count/rate calculations, zero denominators, confusion matrices,
  balanced accuracy, and fixed Wilson 95% vectors including boundary counts.
- Status precedence: hard failure versus exclusion versus indeterminate versus
  protocol failure; no empty endpoint can pass.
- Report invariants: count reconstruction from IDs, canonical ordering,
  complete exclusion ledger, finite values, and semantic report ID stability.

### 7.2 Integration and CLI tests

- Canonical artifact read/write round trips for both new kinds.
- A complete manifest → reader → evaluator → writer → rereader path using
  literal, independently derived expected metrics.
- Wrong kind/schema/hash/identity, missing file, absolute/path-escape,
  overlapping holdout, unknown independence, missing reference, and
  underpowered stratum paths.
- CLI parsing for required flags, unknown options, output collision,
  `--overwrite`, and no raw-file/directory-discovery alternative.
- Atomic publication: staging failure publishes nothing; overwrite failure
  preserves the prior complete managed bundle; unmanaged entries are not
  modified.
- Repeated runs from the same protocol and artifacts produce identical
  scientific report bytes and table bytes. Operational timestamps, if retained
  in the execution manifest, are excluded from semantic identity and tested
  separately.
- Source guard tests prove Phase E does not call Phase B/Phase C assessors and
  Phase D reporting does not import Phase E.

### 7.3 Required fixtures

```text
tests/fixtures/phase_e/
  protocol/
    valid.toml
    missing_threshold.toml
    unknown_field.toml
  dataset/
    valid_holdout.schema1.json
    shared_family.schema1.json
    unknown_lineage.schema1.json
    missing_reference.schema1.json
    path_escape.schema1.json
    wrong_hash.schema1.json
  mechanism/
    supported.schema4.json
    contradicted.schema4.json
    not_assessed.schema4.json
    legacy.schema3.json
  health/
    within_baseline.schema4.json
    alert.schema4.json
    indeterminate.schema4.json
    data_quality_insufficient.schema4.json
    legacy.schema3.json
  expected/
    fixed_metric_ledger.md
```

The expected ledger must be hand-derived from fixture literals, not generated
by the implementation under test. At least one fixture mutation must falsify
each scientific rule.

### 7.4 Scientific validation

Scientific validation has two non-substitutable gates:

1. **E-SW — algorithmic/software gate.** Frozen synthetic and constructed
   fixtures establish exact metric math, leakage handling, missing-data
   behavior, determinism, and artifact compatibility. Passing E-SW authorizes
   only “software validated for the tested contract.”
2. **E-SCI — physical holdout gate.** The project owner approves a protocol
   registered before scoring and an independently acquired/blinded physical
   holdout cohort. The Phase E artifact must report every declared endpoint and
   stratum, all exclusions, and all acceptance outcomes. Independent reviewers
   verify protocol adherence and reference authority. Only claims explicitly
   passing E-SCI may be described as physically validated for the declared
   domain.

If no qualifying physical cohort is available, implementation may still
finish E-SW, but Phase E remains scientifically incomplete and the report must
say `software_validated_only` or `indeterminate`.

### 7.5 Regression validation

Before review, run from a clean worktree pinned to the approved implementation
commit:

```bash
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
cargo build --locked --release
```

Also require:

- all permanent A0/A1/Phase B/Phase C/Phase D tests pass unchanged;
- all historical artifact migration fixtures retain their prior accept/reject
  behavior;
- golden Phase D bundles from identical baseline inputs remain byte-identical;
- existing CLI help/parse snapshots and legacy commands remain compatible;
- the two new artifact-kind variants do not alter serialization of any existing
  `ArtifactKind`; and
- Linux and macOS CI both pass with the locked dependency graph.

Any baseline test modification requires a written compatibility justification
and independent approval. Deleting, weakening, renaming, or replacing a
baseline scientific test is a release blocker.

## 8. Git workflow and review gate

### 8.1 Phase-E baseline and durable branch policy

Phase E is based on the current synchronized `main` at Phase-E
initialization:

```text
PHASE_E_BASELINE_MAIN_SHA=23a6710a869bbaccab2bce3f2bfa67af46380755
```

The historical Phase-D release tag
`ism-mechanism-health-v1-d-complete` at
`2316fb1d076d47ac14d3b3c10c55654feb1ffb54` remains a provenance ancestor of
this baseline. Phase E must verify that ancestry, but must not require
`main` to equal the release SHA, check out that historical commit as its base,
or reset `main` backward to it.

`main` is the only durable development branch. The steady state before,
during planning, and after Phase-E integration is:

```text
local branches:  main only
remote branches: main only
```

### 8.2 Main-only planning and frozen plan review

Planning occurs directly on `main`. There is no
`plan/mhi-v1-e-independent-validation` branch, review branch, or planning
remediation branch. The Phase-E plan is one coherent documentation-only commit
on `main`; it must not modify `src/`, `config/`, `tests/`, `Cargo.toml`,
`Cargo.lock`, CI, or existing artifacts/fixtures.

After that plan commit is pushed normally, freeze the exact review target from
the actual remote state:

```bash
git fetch origin --prune
PLAN_REVIEW_SHA="$(git rev-parse origin/main)"
```

Independent reviewers must review exactly `PLAN_REVIEW_SHA`. They may use
detached `HEAD` for isolation, but no review branch is created. No additional
commit is made on `main` until this review completes. The review must record
all of the following as GO:

- independent scientific review of endpoint definitions, reference authority,
  independence/leakage rules, metric denominators, Wilson method, strata,
  physical holdout design, and claim wording;
- independent architecture review of module boundaries, canonical readers,
  closed schemas, semantic identity, lineage, atomic publication, and rollback;
- compatibility review proving no existing schema, artifact, route, output, or
  test behavior is changed;
- project-owner approval of the release claims and the physical validation
  protocol/data authority;
- a requirement-to-test/fixture traceability matrix with zero unmapped
  requirements, criteria, fixtures, or tests; and
- zero open P0/P1 findings and explicit disposition of all P2 findings.

After independent GO, create the immutable
`ism-mechanism-health-v1-e-plan-approved` tag at exactly the independently
reviewed `PLAN_REVIEW_SHA`. No plan-integration merge is needed because the
approved plan is already committed to `main`. A documentation self-review is
not sufficient for this gate.

### 8.3 The single temporary implementation branch

Only after the plan-approval tag exists, create exactly one temporary branch,
`codex/mhi-v1-e-independent-validation`, from current `main` containing the
approved plan. No implementation-remediation, review, or integration branch
is permitted. If remediation is required, it is made as forward commits on
this one temporary implementation branch.

Implementation is limited to the approved Phase E paths and contracts. Any
change to Phase B/C/D scientific semantics, any new metric, any schema field,
or any compatibility exception requires a new independent plan review of the
forward documentation change on `main`, rather than a planning branch.

Implementation should use coherent reviewable commits in this order:

1. artifact kinds, result schemas, and negative schema fixtures;
2. protocol/dataset readers and leakage classification;
3. deterministic metrics and pure evaluator;
4. CLI runner and atomic outputs; and
5. full fixtures, traceability, compatibility evidence, and documentation.

### 8.4 Implementation review freeze

After the temporary implementation branch is pushed, the reviewer freezes the
exact remote commit to review:

```bash
git fetch origin --prune
REVIEW_SHA="$(
  git rev-parse origin/codex/mhi-v1-e-independent-validation
)"
```

Independent reviewers must review exactly `REVIEW_SHA`. If the branch advances
after that freeze, the review remains valid for `REVIEW_SHA`; later commits are
simply unreviewed. No stale-SHA stop loop is permitted.

The frozen implementation commit may be approved only when:

- E-SW passes in full on Linux and macOS;
- E-SCI passes for every claim proposed as physically validated, or those
  claims are removed and the output remains explicitly software-only;
- the full locked regression suite and Phase D byte-compatibility checks pass;
- source, schema, fixture, and generated-output diffs match the approved scope;
- independent scientific and architecture reviewers both record GO; and
- the validated commit has an explicit rollback target at the pre-merge
  `main` commit.

### 8.5 Integration and cleanup

After independent GO, create an immutable
`ism-mechanism-health-v1-e-implementation-approved` tag at the exact reviewed
implementation SHA. Merge that exact reviewed SHA into `main` using:

```bash
git merge --no-ff <EXACT_REVIEWED_SHA>
```

Validate the merge, push `main` normally, and create the immutable
`ism-mechanism-health-v1-e-implementation-integrated` tag at the integrated
`main` commit. After remote main and all approval/integration tags are
verified, delete the local and remote
`codex/mhi-v1-e-independent-validation` branch. The resulting steady state is
again local `main` only and remote `main` only.

Planning should use one coherent documentation commit before initial review;
if plan remediation is required, use as few forward documentation commits as
practical. Implementation should likewise prefer coherent commits rather than
a commit for every minor test or fix. Branches must not be created to
compensate for commit organization.

No Phase-E workflow step may use reset, rebase, amend, squash, force-push,
tag movement, or stash deletion. No implementation branch, approval tag, or
merge is created as part of this planning task.

## 9. Milestone exit criteria

Phase E is complete only when all of these statements are true:

1. Every E-SCI objective has a passing test or approved physical-validation
   record, with no silent unavailable data.
2. Both new schemas and the output bundle are closed, canonical, deterministic,
   provenance-complete, and atomically published.
3. Every validation result is reconstructible from serialized IDs, counts,
   exclusions, and source identities.
4. Phase B/C/D artifacts remain immutable and their scientific semantics are
   unchanged.
5. All baseline and Phase E test/CI gates pass.
6. Independent scientific, architecture, and compatibility reviews record GO.
7. Release language distinguishes software validation from physical validation
   and names the exact validated domain.

Until then, the correct milestone status is `planning`, `software-only`, or
`indeterminate` as applicable—never implied scientific validation.

## 10. Planning traceability contract

The implementation review must preserve these IDs. Splitting a test is
permitted only when every resulting test remains mapped; combining requirements
into one helper-only test is not sufficient.

| Requirement | Normative behavior | Planned owner | Acceptance criterion | Required tests | Explicit failure criterion |
|---|---|---|---|---|---|
| E-R01 | One additive certified `validation run` route with three required paths and no raw-input alternative. | `cli.rs`, `main.rs`, `runners/mhi_validation.rs` | E-AC01: Clap/runner accepts the exact valid route and rejects missing/unknown/conflicting input before evaluation. | E-T01, E-T02 | Existing command changes, a required flag is ignored, or a second Phase E route exists. |
| E-R02 | Protocol schema is closed, hashed from exact bytes, and has no scientific defaults. | `protocol.rs`, `validation_config.rs` | E-AC02: every endpoint is complete and invalid/unknown/duplicate/nonfinite rules fail deterministically. | E-T03, E-T04 | A missing threshold, family minimum, class set, stratum, or acceptance rule gains an implicit value. |
| E-R03 | Dataset artifact has canonical records, safe relative paths, exact expected source identity/hash, and authoritative references. | `results/mhi_validation.rs`, `reader.rs` | E-AC03: schema-1 round-trip is exact; unsafe/mismatched/malformed inputs fail before scoring. | E-T05, E-T06 | A path escapes, an identity/hash mismatch is consumed, or a reference label lacks required authority. |
| E-R04 | Scientific inputs use only canonical artifact readers and exact Phase B/C schemas. | `reader.rs`, `domain/artifact.rs` | E-AC04: schema-4 mechanism/health inputs read; wrong kind/future schema hard-fail; readable legacy is explicitly excluded. | E-T07, E-T08 | Ad-hoc JSON, future-schema acceptance, legacy synthesis, or wrong-kind consumption occurs. |
| E-R05 | Every declared record receives one deterministic cohort/eligibility decision. | `partition.rs` | E-AC05: declared = eligible + excluded for every endpoint/stratum, with reconstructible IDs. | E-T09 | A record is silently dropped, double-counted, or assigned a role from path/time. |
| E-R06 | Holdout leakage uses lineage, experiment scope, and acquisition families; known overlap fails and unknown separation is indeterminate. | `partition.rs` | E-AC06: all disjoint/shared/unknown/missing/cycle/aggregate cases produce the exact closed result. | E-T10, E-T11 | Shared/unknown evidence passes, or a family/ID is fabricated. |
| E-R07 | Reference outcomes are method-authoritative, provenance-complete, and independent of assessor inputs. | `reader.rs`, `partition.rs` | E-AC07: same-derived reference is ineligible; independent blinded authority is retained exactly. | E-T12 | A label inferred from assessed output or missing required authority enters a holdout numerator. |
| E-R08 | Mechanism validation projects serialized Phase B state into the exact mutually exclusive counts and Wilson rates. | `mechanism.rs`, `statistics.rs` | E-AC08: hand-derived support/contradiction/not-assessed fixtures match exact counts, fractions, intervals, and IDs. | E-T13, E-T14 | The Phase B assessor is called, categories overlap, or a missing denominator becomes pass/zero. |
| E-R09 | Health validation uses protocol-declared classes and the exact confusion/coverage equations. | `health.rs`, `statistics.rs` | E-AC09: hand-derived TP/TN/FP/FN, missing-state rates, derived metrics, and boundaries match fixed vectors. | E-T15, E-T16 | Indeterminate/DQI enters the confusion matrix, balanced accuracy is invented, or sign/class mapping reverses. |
| E-R10 | Wilson 95% is the sole V1 interval method and matches section 3.1. | `statistics.rs` | E-AC10: boundary and reference vectors agree within a frozen `1e-12` absolute test tolerance; serialized results remain finite and clamped. | E-T17 | Another interval/continuity correction appears, or numerical output exceeds tolerance/nonfinite bounds. |
| E-R11 | Required domain strata remain visible and cannot be rescued by aggregate pooling. | `partition.rs`, `assessment.rs` | E-AC11: empty/underpowered required stratum makes its endpoint and required aggregate indeterminate. | E-T18 | Aggregate success hides a failed, empty, or unavailable required stratum. |
| E-R12 | Acceptance applies only the explicit ordered AND rules and closed status precedence. | `assessment.rs` | E-AC12: point/bound/count comparators, unavailable targets, failed rules, and all-pass cases return exact statuses/reasons. | E-T19 | Rounding/tolerance/default OR behavior changes an outcome. |
| E-R13 | Validation report is closed, finite, lineage-complete, deterministic, and reconstructible. | `results/mhi_validation.rs` | E-AC13: report validation reconstructs every count and stable report ID from serialized authority. | E-T20, E-T21 | Counts disagree with IDs, a consumed dependency is absent, or clock/path changes scientific identity. |
| E-R14 | Publication is complete, atomic, checksum-verified, and safe on overwrite. | `output.rs`, runner | E-AC14: success publishes the exact managed set; staged/publication failures leave zero partial output and preserve a prior bundle. | E-T22, E-T23 | Partial files appear, unmanaged data changes, symlink traversal occurs, or checksums are unverified. |
| E-R15 | Phase B/C assessors and Phase D projection/output remain unchanged and independent of Phase E. | source dependency guards; full baseline suite | E-AC15: source guards and baseline/golden tests prove no reverse dependency or output drift. | E-T24, E-T25 | Phase E reassesses/mutates sources or identical Phase D input produces changed public output. |
| E-R16 | New artifact kinds are additive and schema-1-only; existing serialization/migration remains exact. | `artifact.rs`, `artifact_contracts.rs` | E-AC16: new round trips and negative matrices pass while all existing fixture matrices retain prior results. | E-T26, E-T27 | Existing token/schema behavior changes or a Phase E future/legacy schema is silently accepted. |
| E-R17 | Software validation and physical validation are separate release states. | protocol/report schema, assessment | E-AC17: synthetic-only input cannot emit `physically_validated`; qualifying protocol-conforming physical holdout can do so only for named passing claims and at least two independent families. | E-T28, E-T29 | Synthetic or under-independent evidence obtains a physical-validation claim. |
| E-R18 | Full CI, scientific review, architecture review, compatibility review, and final re-review gate release. | CI and review records | E-AC18: all commands/reviews apply to the same final commit with zero unresolved P0/P1. | E-T30 | A command/review is absent/stale, required tests fail, or implementation self-approves. |

### 10.1 Exact required test registry

| Test ID | Exact required test or evidence record |
|---|---|
| E-T01 | `phase_e_cli_runs_exact_certified_route` |
| E-T02 | `phase_e_cli_rejects_missing_unknown_and_raw_input_routes` |
| E-T03 | `phase_e_protocol_roundtrip_preserves_all_scientific_rules` |
| E-T04 | `phase_e_protocol_rejects_defaults_unknowns_duplicates_and_nonfinite_values` |
| E-T05 | `phase_e_dataset_schema1_roundtrip_is_closed_and_canonical` |
| E-T06 | `phase_e_dataset_rejects_unsafe_paths_identity_hash_and_reference_authority` |
| E-T07 | `phase_e_reader_accepts_only_canonical_schema4_scientific_inputs` |
| E-T08 | `phase_e_reader_hard_fails_wrong_future_and_explicitly_excludes_legacy` |
| E-T09 | `phase_e_partition_accounts_for_every_declared_record_exactly_once` |
| E-T10 | `phase_e_holdout_rejects_known_lineage_scope_and_family_overlap` |
| E-T11 | `phase_e_holdout_unknown_separation_is_indeterminate_without_fabrication` |
| E-T12 | `phase_e_reference_requires_independent_blinded_provenance_authority` |
| E-T13 | `phase_e_mechanism_counts_match_hand_derived_fixture` |
| E-T14 | `phase_e_mechanism_rates_intervals_and_ids_are_exact` |
| E-T15 | `phase_e_health_confusion_and_missing_state_counts_are_exact` |
| E-T16 | `phase_e_health_rates_boundaries_and_balanced_accuracy_are_exact` |
| E-T17 | `phase_e_wilson_95_fixed_vectors_and_limits_are_exact` |
| E-T18 | `phase_e_required_stratum_cannot_be_hidden_by_aggregate_pass` |
| E-T19 | `phase_e_acceptance_rules_use_exact_targets_comparators_and_precedence` |
| E-T20 | `phase_e_report_reconstructs_every_count_from_source_ids` |
| E-T21 | `phase_e_report_semantic_identity_excludes_clock_and_path` |
| E-T22 | `phase_e_publication_is_atomic_and_checksum_verified` |
| E-T23 | `phase_e_failed_overwrite_restores_prior_bundle_and_preserves_unmanaged_entries` |
| E-T24 | `phase_e_source_guards_prohibit_reassessment_and_reverse_dependencies` |
| E-T25 | `phase_e_preserves_phase_d_golden_outputs_byte_for_byte` |
| E-T26 | `phase_e_artifact_contracts_accept_exact_schema1_and_reject_invalid_variants` |
| E-T27 | `phase_e_preserves_all_existing_artifact_migration_contracts` |
| E-T28 | `phase_e_synthetic_only_run_is_software_validated_only` |
| E-T29 | `phase_e_physical_claim_requires_named_passing_holdout_and_two_independent_families` |
| E-T30 | committed validation/review ledger tying all required commands and independent GO records to the final implementation SHA |

Planning traceability totals are 18 requirements, 18 acceptance criteria, and
30 required tests/evidence records. Unmapped requirements = 0; acceptance
criteria without a required test = 0; implementation is still prohibited until
the independent planning review confirms these mappings and the full schema.
