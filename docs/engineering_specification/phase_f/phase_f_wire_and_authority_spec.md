# Phase F Wire and Authority Specification

## 1. Authority and adoption

This G2 candidate owns exact machine-facing Phase-F authority contracts. It
refines `F-ARCH-004..010`, `F-ARCH-015..021` and the applicable F0 decisions.
It does not own scientific acceptance thresholds, evidence interpretation, KAT
values, or implementation layout.

The exact R11 source bytes at `phase_f_r11_normative_source.md` are incorporated
only through the clauses listed below. Their wire semantics remain unchanged;
references to “plan authority” in those clauses now mean this specification
unless the architecture plan explicitly retains ownership.

## 2. Requirements

| ID | derived_from | Normative requirement | Adopted R11 clauses |
|---|---|---|---|
| <a id="F-WIRE-001"></a>`F-WIRE-001` | `F-ARCH-004,F-ARCH-017` | External JSON uses UTF-8, RFC 8785 JCS, duplicate/unknown-member rejection, no omitted members, and the exact closed primitive/type registry. | §§2, 53.7 |
| <a id="F-WIRE-002"></a>`F-WIRE-002` | `F-ARCH-017` | Every content-derived ID uses the unique NUL-terminated domain separator, complete semantic payload, exact exclusions, and no registry back pointer or future-object cycle. Complete-file SHA is computed after the file is complete. | §3 |
| <a id="F-WIRE-003"></a>`F-WIRE-003` | `F-ARCH-007,F-ARCH-008` | Review targets, five rows, arithmetic aggregates, and bidirectional GO predicate use the exact review wire. Specification component and aggregate bundle reviews use the same rule. | §5 excluding F5 scientific meaning |
| <a id="F-WIRE-004"></a>`F-WIRE-004` | `F-ARCH-006,F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-05,F-OD-06,F-OD-07,F-OD-08,F-OD-09,F-OD-10,F-OD-11,F-OD-12,F-OD-13,F-OD-14,F-OD-15,F-OD-16,F-OD-17,F-OD-18,F-OD-19,F-OD-20` | `PhaseFDecisionBundleV1`, its 20 value variants, ordering, runtime projection wire, and no-future-F1-object rules are exact. | §4, §53.7 decision anchors |
| <a id="F-WIRE-005"></a>`F-WIRE-005` | `F-ARCH-015,F-ARCH-017,F-OD-04,F-OD-13,F-OD-14,F-OD-15,F-OD-16` | Ed25519 keys/signatures, enrollment, trust bindings, registry record/head, object/record/relation kinds, subject hashes, relation ordering, genesis, sequence, resolver, compromise, and emergency wire are exact and fail closed. | §§5.2, 8, 9, 15 emergency wire, 53.7 anchors |
| <a id="F-WIRE-006"></a>`F-WIRE-006` | `F-ARCH-012..016,F-ARCH-021` | Retrieval, package, dependency, physical identity/custody, power, metrology, cohort, release, claim-state, monitoring, incident, resolution, and retention schema field closures are exact; scientific/operational interpretation remains with its owning spec. | §§10–15, 53.7 anchors |
| <a id="F-WIRE-007"></a>`F-WIRE-007` | `F-ARCH-008,F-ARCH-009,F-ARCH-010` | Every durable tag is annotated and uses exact target/body/prerequisite validation. Add `ism-mechanism-health-v1-f-specification-bundle-approved` with ordered fields `architecture_plan_tag`, `f0_decisions_tag`, `specification_bundle_manifest_sha256`, `aggregate_review_bundle_sha256`, `approval_decision`; the last value must be `GO`. | §6 plus this row |
| <a id="F-WIRE-008"></a>`F-WIRE-008` | `F-ARCH-004,F-ARCH-005,F-ARCH-017` | The current schema set has exactly the 91 R11 identifiers plus `PhaseFSpecificationBundleApprovalV1`; every schema has one definition anchor, category, exact field closure, identity/hash rule, producer, validator, stage, registry behavior, and exhaustive nested usage rows. | §§53.7, 53.12 |
| <a id="F-WIRE-009"></a>`F-WIRE-009` | `F-ARCH-017,F-ARCH-021` | `PhaseFCheckerBuildEvidenceV1` and `PhaseFCheckerReadinessEvidenceV1` each add required `specification_bundle_approval_tag:RUNTIME_CANONICAL_TEXT_V1` and `specification_bundle_manifest_sha256:SHA256_V1` fields. Enrollment binds readiness, and every later authority binds that chain transitively. A missing or mismatched G3 binding invalidates readiness and every descendant. | New R12 closure |

## 3. Specification-bundle tag grammar

The exact annotated message is six LF-terminated ASCII lines in this order:

```text
phase_f_architecture_plan_tag=<annotated tag name>
phase_f_f0_decisions_tag=<annotated tag name>
specification_bundle_manifest_sha256=<SHA256_V1>
aggregate_review_bundle_sha256=<SHA256_V1>
approval_decision=GO
schema_version=1
```

No CR, blank line, alternate order, duplicate, omitted, or additional field is
valid. The tag peels to the reviewed commit containing the exact manifest and
all five component files. The architecture and F0 tag names must resolve to
valid annotated approval tags and exact authorities named by the manifest.

## 4. Review gate

P0/P1 must both be zero. Ambiguous schema/hash/ID/relation/tag grammar, missing
usage row, conflicting closure, or an absent upstream bundle binding is P1.
This candidate grants no authority until independently reviewed GO and included
in an approved G3 manifest.
