# Phase F R11 → R12 Lossless Migration Ledger

## 1. Source and rule

Source: `phase_f_r11_normative_source.md`, SHA-256
`987bc6e06a5c43873b844f864cb1f858c6b57c40c18dd0d4ed4a4edcf32dec3d`,
Git blob `34ab62d094c4cb0bb31a40dc7a192ed304faf981`.

The R11 §53.8 matrix is the single current R11 requirement set. Every row is
mapped below; distributed field closures, anchors, usage rows, ACs, tests,
fixtures, and F-EVs travel with their mapped requirement through the explicit
clause-adoption tables in the destination documents. Historical R1–R10 prose
remains provenance and is not silently promoted to a second current authority.

## 2. Normative-obligation migration

| R11 requirement | Semantic obligation | R12 destination / requirement | Semantics | Further closure | Review status |
|---|---|---|---|---|---|
| R11-01 | Literal plan tag parser plus real-Git property | Conformance `F-CNF-004` | unchanged/refined | add R12 architecture tag candidate | PENDING |
| R11-02 | Literal trust tag/parser/binding | Wire `F-WIRE-005,F-WIRE-007`; Conformance `F-CNF-004` | unchanged | none | PENDING |
| R11-03 | Opaque retention storage bytes after identity validation | Operations `F-OPS-007`; Conformance `F-CNF-002` | unchanged | none | PENDING |
| R11-04 | Literal incident/resolution progression and hashes | Operations `F-OPS-004`; Conformance `F-CNF-001` | unchanged | none | PENDING |
| R11-05 | Exact release retention composition | Operations `F-OPS-006`; Conformance `F-CNF-002` | unchanged | none | PENDING |
| R11-06 | Exact campaign membership, static protocol separation | Operations `F-OPS-006`; Conformance `F-CNF-002` | unchanged | none | PENDING |
| R11-07 | Complete 15-metric KAT or narrow property only | Operations `F-OPS-003`; Conformance `F-CNF-003` | unchanged | none | PENDING |
| R11-08 | Explicit constructive authority DAG | Architecture `F-ARCH-017`; Conformance `F-CNF-005`; Implementation `F-IMPL-007` | refined for G3 | authoritative typed R12 graph with exact source-kind/relation/destination-kind tuples, mandatory per-edge `none`/`serialized_binding` obligations, root SHA-256 binding into specification inputs, derived node/semantic/serialized/builder/validator projections, computed hash/self-Git/self-reference/future-object/target/bypass audits, and isolated root-change/real-format fixture coverage | PENDING |
| R11-09 | One anchor and catalog row per identifier | Wire `F-WIRE-008`; Conformance `F-CNF-005` | refined | add bundle-tag schema | PENDING |
| R11-10 | Exhaustive nested usage rows | Wire `F-WIRE-008`; Conformance `F-CNF-005` | unchanged | reconcile new schema use | PENDING |
| R11-11 | Exact catalog metadata | Wire `F-WIRE-008`; Conformance `F-CNF-005` | unchanged | add new row | PENDING |
| R11-12 | Traceability only from derived inverse | Architecture `F-ARCH-022`; Conformance `F-CNF-005` | refined to JSON normative matrix plus derived manifest | exact bidirectional semantic and schema-usage reconciliation | PENDING |
| R11-13 | Complete literal inputs; honest result class | Conformance `F-CNF-003,F-CNF-006` | unchanged | none | PENDING |
| R11-14 | No KAT/evidence/claim promotion | Architecture `F-ARCH-002,F-ARCH-017`; Scientific `F-SCI-009`; Conformance `F-CNF-007` | unchanged | none | PENDING |
| R11-15 | Campaign/static terminology | Operations `F-OPS-006`; Conformance `F-CNF-002` | unchanged | none | PENDING |
| R11-16 | Structurally valid Markdown | Conformance `F-CNF-005` | unchanged | run across all current docs | PENDING |
| R11-17 | Preserve closed safety/science/Phase-E/P2 | Architecture `F-ARCH-003,F-ARCH-013..017`; Conformance `F-CNF-008`; Implementation `F-IMPL-006` | unchanged | regression replay | PENDING |
| R11-18 | Exact schema set/inverse coverage | Wire `F-WIRE-008`; Conformance `F-CNF-005` | forward-refined | R11 remains 91; the R12 expected set becomes 97 (91 inherited plus `PhaseFSpecificationBundleApprovalV1`, `PhaseFMigratedFindingReviewV1`, `PhaseFReviewerActorAttestationV1`, `PhaseFReviewerBootstrapTrustRootV1`, `PhaseFReviewerBootstrapCurrentnessProofV1`, and `PhaseFReviewerBootstrapAcceptedHeadCheckpointV1`) and is invertible through the R12 matrix `schema_ids` cells | PENDING |
| R11-19 | Future F-EV only real; KATs only tests | Scientific `F-SCI-010`; Conformance `F-CNF-007` | unchanged | none | PENDING |
| R11-20 | Exactly 20 owner decisions | Architecture `F-ARCH-006`; Wire `F-WIRE-004`; Conformance `F-CNF-005` | unchanged | none | PENDING |

## 3. R11 finding migration

No author disposition is upgraded to CLOSED by this refactor.

| Finding | New owner/gate | Status |
|---|---|---|
| F-PLAN-R11-P1-01 storage/schema conflation | `F-CNF-001,F-CNF-002`, G2 Conformance | OPEN pending independent review |
| F-PLAN-R11-P1-02 catalog/usage metadata | `F-WIRE-008,F-CNF-005`, G2 Wire/Conformance | OPEN pending independent review |
| F-PLAN-R11-P1-03 incomplete parser/DAG/monitoring positives | `F-CNF-003..006`, G2 Conformance | OPEN pending independent review |
| F-PLAN-R11-P1-04 campaign/static membership | `F-OPS-006,F-CNF-002`, G2 Operations/Conformance | OPEN pending independent review |
| F-PLAN-R11-P3-01 Markdown fence integrity | `F-CNF-005`, G2 Conformance | OPEN pending independent review |

## 4. Completeness result

```text
R11_CURRENT_NORMATIVE_OBLIGATIONS=20
R12_ARCHITECTURE_SELF_CLOSED=3
R12_WIRE_OWNED=5
R12_SCI_OWNED=3
R12_OPS_OWNED=5
R12_CNF_OWNED=3
R12_IMPL_OWNED=1
missing=0
unowned=0
duplicated_conflicting_ownership=0
migrated_findings_missing=0
```

Rows with multiple refining documents have one semantic owner and one or more
verification/implementation consumers; that is traceability, not conflicting
normative ownership. Independent reviews remain PENDING, so G2 and G3 are not
approved.

## 5. R12 authority-closure remediation

The current R12 closure adds no approval result. The immutable
`PhaseFMigratedFindingReviewV1` schema is a future independent-review object;
the five migrated findings remain pending fresh independent disposition. The
bundle binds a typed nullable reference to that object and stays fail closed
until a genuine external review exists. The exact R12 artifact-level graph,
semantic traceability matrix, shared G3 validator, real-format resolver,
serialized binding checks, and all lifecycle/staleness checks are defined as
planning contracts only. This remediation does not fabricate a review artifact
for its own work. `PhaseFReviewerActorAttestationV1`,
`PhaseFReviewerBootstrapTrustRootV1`, and
`PhaseFReviewerBootstrapCurrentnessProofV1`, and
`PhaseFReviewerBootstrapAcceptedHeadCheckpointV1` are forward R12 additive schemas:
the bootstrap root and signed currentness proof establish the narrow pre-G0
reviewer trust domain, the resolver checkpoint establishes monotonic accepted
head state, and the attestation binds each REAL reviewer support record to that
chain without changing R11 fields, registry record kinds,
relation enums, or the authority graph. This remediation creates no real root,
actor identity, enrollment, attestation, review, approval, or publication
authority.
