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
| <a id="F-WIRE-007"></a>`F-WIRE-007` | `F-ARCH-008,F-ARCH-009,F-ARCH-010` | Every durable tag is annotated and uses exact target/body/prerequisite validation. Add `ism-mechanism-health-v1-f-specification-bundle-approved` with the exact ordered body fields `phase_f_architecture_plan_tag`, `phase_f_f0_decisions_tag`, `specification_bundle_manifest_sha256`, `aggregate_review_bundle_sha256`, `approval_decision`, `schema_version`; their exact values and byte grammar are defined in §3, and `approval_decision` is `GO`. | §6 plus this row |
| <a id="F-WIRE-008"></a>`F-WIRE-008` | `F-ARCH-004,F-ARCH-005,F-ARCH-017` | The current schema set has exactly the 91 R11 identifiers plus `PhaseFSpecificationBundleApprovalV1`; every schema has one definition anchor, category, exact field closure, identity/hash rule, producer, validator, stage, registry behavior, and exhaustive nested usage rows. | §§53.7, 53.12 |
| <a id="F-WIRE-009"></a>`F-WIRE-009` | `F-ARCH-017,F-ARCH-021` | `PhaseFCheckerBuildEvidenceV1` and `PhaseFCheckerReadinessEvidenceV1` each add required `specification_bundle_approval_tag:RUNTIME_CANONICAL_TEXT_V1` and `specification_bundle_manifest_sha256:SHA256_V1` fields. Enrollment binds readiness, and every later authority binds that chain transitively. A missing or mismatched G3 binding invalidates readiness and every descendant. | New R12 closure |

## 3. Specification-bundle tag grammar

`PhaseFSpecificationBundleApprovalV1` is the `TAG_BODY` contract for the
annotated tag named exactly
`ism-mechanism-health-v1-f-specification-bundle-approved`. The exact message is
six LF-terminated ASCII lines in this order:

```text
phase_f_architecture_plan_tag=<annotated tag name>
phase_f_f0_decisions_tag=<annotated tag name>
specification_bundle_manifest_sha256=<SHA256_V1>
aggregate_review_bundle_sha256=<SHA256_V1>
approval_decision=GO
schema_version=1
```

No CR, blank line, alternate order, duplicate, omitted, or additional field is
valid. The final LF is required and is part of the message bytes. Field names
are ASCII and values contain no LF, CR, `=`, or leading/trailing whitespace.
The two tag fields must contain the exact Phase-F architecture-plan and F0
approval tag names; the two hash fields are exactly 64 lowercase hexadecimal
characters; `approval_decision` is exactly `GO`; and `schema_version` is
exactly the JSON/text value `1`. Unknown, missing, duplicate, reordered, or
legacy unprefixed fields are rejected; they are not aliases. The parser input
is the pair `(tag_name, exact_body_bytes)`, so the tag name is checked in
addition to the body.

The tag peels to the reviewed commit containing the exact manifest and all five
component files. The architecture and F0 tag names must resolve to valid
annotated approval tags and exact authorities named by the manifest. This
contract defines syntax and binding prerequisites only; it does not create an
approval, review, tag, or implementation authority.

## 4. Current R12 schema catalog closure

The 91 R11 catalog rows in the preserved source's §53.12 remain the inherited
baseline. The current R12 schema set is exactly those 91 identifiers plus the
single row below. The generator checks this set equality and validates the
new row's anchor and every non-empty catalog dimension; there is no wildcard or
parallel approval catalog.

<a id="schema-def-PhaseFSpecificationBundleApprovalV1"></a>
`SCHEMA_DEF[PhaseFSpecificationBundleApprovalV1]` is the exact six-line
`TAG_BODY` in §3. All six fields are required, have the order and value rules
specified there, and have no nullable, unknown, omitted, duplicate, or
additional-member form. Its complete identity is the SHA-256 of the exact tag
message bytes; it has no JSON semantic ID and no complete-file JSON hash.

| identifier | category | exact field-closure pointer | semantic identity / complete-file hash meaning | concrete producer | actual validator | exact stage/set | exact registry behavior | traceability |
|---|---|---|---|---|---|---|---|---|
| PhaseFSpecificationBundleApprovalV1 | TAG_BODY | #schema-def-PhaseFSpecificationBundleApprovalV1 | no JSON semantic ID; SHA-256 of the exact six-line annotated tag-message bytes including the final LF | independent five-role specification-bundle approval gate | exact §3 tag-name/body parser plus target, architecture approval, F0 approval, five component-review, traceability, migrated-finding, aggregate-review, and `approval_decision=GO` validator | G3 specification-bundle approval, after architecture/F0 approvals and all five component reviews | TAG_BODY; Git annotated-tag message only; no registry subject and no registry record | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFSpecificationBundleApprovalV1) |

The approval object binds the exact architecture-plan tag, F0-decisions tag,
specification-bundle manifest SHA, aggregate review-bundle SHA, and all five
component review rows through the manifest and its prerequisite validator. It
does not duplicate those review objects, create a registry subject, or permit
the synthetic KAT to satisfy any prerequisite. The object is created only by
the independent G3 approval gate; this planning remediation creates none.

Its exhaustive current usage set is `F-WIRE-007` (tag grammar and binding),
`F-WIRE-008` (schema/catalog closure), `F-CNF-004` (literal parser KAT),
`F-CNF-005` (catalog and traceability audit), `F-CNF-006` (literal KAT
completeness), `F-IMPL-005` (downstream readiness binding), and `F-IMPL-007`
(authority-DAG integration). The object is immutable after creation; a changed
upstream tag, manifest, review bundle, component review, or migrated-finding
review invalidates the candidate and requires a forward bundle revision. The
R11 approval bodies and all Phase-D/Phase-E bindings remain unchanged.

## 5. Review gate

P0/P1 must both be zero. Ambiguous schema/hash/ID/relation/tag grammar, missing
usage row, conflicting closure, or an absent upstream bundle binding is P1.
This candidate grants no authority until independently reviewed GO and included
in an approved G3 manifest.
