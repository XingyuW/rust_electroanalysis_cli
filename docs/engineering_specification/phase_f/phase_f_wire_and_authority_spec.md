# Phase F Wire and Authority Specification

## 1. Authority and adoption

This G2 candidate owns exact machine-facing Phase-F authority contracts. It
refines `F-ARCH-004..010`, `F-ARCH-015..021` and the applicable F0 decisions.
It does not own scientific acceptance thresholds, evidence interpretation, KAT
values, or implementation layout.

The exact R11 source bytes at `phase_f_r11_normative_source.md` are incorporated
only through the clauses listed below. Their wire semantics remain unchanged;
references to “plan authority” in those clauses now mean this specification
unless the architecture plan explicitly retains ownership. R11 remains the
immutable source of the independent-review bundle wire; this document records
the R12 graph placement and resolver closure without redefining that wire.

## 2. Requirements

| ID | derived_from | Normative requirement | Adopted R11 clauses |
|---|---|---|---|
| <a id="F-WIRE-001"></a>`F-WIRE-001` | `F-ARCH-004,F-ARCH-017` | External JSON uses UTF-8, RFC 8785 JCS, duplicate/unknown-member rejection, no omitted members, and the exact closed primitive/type registry. | §§2, 53.7 |
| <a id="F-WIRE-002"></a>`F-WIRE-002` | `F-ARCH-017` | Every content-derived ID uses the unique NUL-terminated domain separator, complete semantic payload, exact exclusions, and no registry back pointer or future-object cycle. Complete-file SHA is computed after the file is complete. | §3 |
| <a id="F-WIRE-003"></a>`F-WIRE-003` | `F-ARCH-007,F-ARCH-008` | Every R12 graph node claiming `PhaseFIndependentReviewBundleV1` uses the exact inherited R11 seven-field bundle: `schema_version`, `review_bundle_id`, `target`, `reviews`, `aggregate_p0_count`, `aggregate_p1_count`, and `aggregate_decision`. Its five rows are in canonical role order, use the exact R11 six-field row, and derive arithmetic aggregates and the GO predicate. | R11 §§3, 5 plus R12 graph/refinement |
| <a id="F-WIRE-004"></a>`F-WIRE-004` | `F-ARCH-006,F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-05,F-OD-06,F-OD-07,F-OD-08,F-OD-09,F-OD-10,F-OD-11,F-OD-12,F-OD-13,F-OD-14,F-OD-15,F-OD-16,F-OD-17,F-OD-18,F-OD-19,F-OD-20` | `PhaseFDecisionBundleV1`, its 20 value variants, ordering, runtime projection wire, and no-future-F1-object rules are exact. | §4, §53.7 decision anchors |
| <a id="F-WIRE-005"></a>`F-WIRE-005` | `F-ARCH-015,F-ARCH-017,F-OD-04,F-OD-13,F-OD-14,F-OD-15,F-OD-16` | Ed25519 keys/signatures, enrollment, trust bindings, registry record/head, object/record/relation kinds, subject hashes, relation ordering, genesis, sequence, resolver, compromise, and emergency wire are exact and fail closed. | §§5.2, 8, 9, 15 emergency wire, 53.7 anchors |
| <a id="F-WIRE-006"></a>`F-WIRE-006` | `F-ARCH-012..016,F-ARCH-021` | Retrieval, package, dependency, physical identity/custody, power, metrology, cohort, release, claim-state, monitoring, incident, resolution, and retention schema field closures are exact; scientific/operational interpretation remains with its owning spec. | §§10–15, 53.7 anchors |
| <a id="F-WIRE-007"></a>`F-WIRE-007` | `F-ARCH-008,F-ARCH-009,F-ARCH-010` | Every durable tag is annotated and uses exact target/body/prerequisite validation. Add `ism-mechanism-health-v1-f-specification-bundle-approved` with the exact ordered body fields `phase_f_architecture_plan_tag`, `phase_f_f0_decisions_tag`, `specification_bundle_manifest_sha256`, `aggregate_review_bundle_sha256`, `approval_decision`, `schema_version`; their exact values and byte grammar are defined in §3, and `approval_decision` is `GO`. | §6 plus this row |
| <a id="F-WIRE-008"></a>`F-WIRE-008` | `F-ARCH-004,F-ARCH-005,F-ARCH-017` | The current schema set has exactly the 91 R11 identifiers plus `PhaseFSpecificationBundleApprovalV1` and `PhaseFMigratedFindingReviewV1`; every schema has one definition anchor, category, exact field closure, identity/hash rule, producer, validator, stage, registry behavior, and exhaustive nested usage rows. | §§53.7, 53.12 |
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
`PhaseFSpecificationBundleApprovalV1` and `PhaseFMigratedFindingReviewV1`
rows below. The generator checks this set equality and validates every new
row's anchor and every non-empty catalog dimension; there is no wildcard or
parallel approval catalog.

### 4.1 Canonical independent-review bundle wire

`PhaseFIndependentReviewBundleV1` is inherited from the immutable R11 source;
it is not a new R12 schema identifier. Its complete top-level field set is
exactly:

```text
schema_version,review_bundle_id,target,reviews,
aggregate_p0_count,aggregate_p1_count,aggregate_decision
```

`schema_version` is `1`. `review_bundle_id` is the R11 semantic ID computed
with domain `mhi_phase_f_review_bundle_v1\0` over the complete payload with its
own ID excluded, using RFC 8785/JCS bytes. `target` is the exact R11 tagged
union. A Git target is exactly
`{"type":"git_commit","git_sha":"<40 lowercase hex>"}`. An external
authority-object target is exactly
`{"type":"external_object","object_kind":"<R11 enum>","object_sha256":"<64 lowercase hex>"}`;
`object_sha256` is the complete canonical object digest defined by the
immutable R11 source. `reviews` has exactly one row for each role, in this
order: `scientific_metrology`, `architecture_data`, `security`,
`compatibility`, `operations_governance`. Each row has exactly `role`,
`decision`, `p0_count`, `p1_count`, `finding_ids`, and
`review_artifact_reference`; the reference has exactly `immutable_uri`,
`sha256`, and `byte_length`. Counts are canonical unsigned integer strings,
finding IDs are sorted and unique, and the aggregate counts are arithmetic
sums. `aggregate_decision` is `GO` exactly when every row is `GO` and both
blocking aggregates are zero; otherwise it is `NO-GO`.

The target is derived from the graph's single incoming `reviews` edge, never
from the review-node name and never from a global commit override. For the
current graph, `architecture_review`, all five component review nodes,
`aggregate_review`, and `readiness_review` review repository-owned revisions,
so their target is the reviewed Git commit. `f0_review` reviews the external
`PhaseFDecisionBundleV1` authority object, so its target is
`{"type":"external_object","object_kind":"decision_bundle",
"object_sha256":"<f0 decision-bundle SHA>"}`. The same rule applies to every
future direct review whose graph predecessor is an R11 external authority
object: use that predecessor's exact object kind and complete-object SHA;
otherwise use the reviewed Git commit. The artifact's `reviewed_target` keeps
the source-specific evidence binding and must equal the source object/file
SHA. Bundle lifecycle, staleness, supersession, and invalidation are
resolver-derived state and are intentionally not serialized in this R11 wire.

The validator accepts only the exact two R11 target shapes, rejects nullable,
extra, omitted, uppercase, wrong-kind, and wrong-digest forms, and compares
the complete target object with the graph-derived expectation. This preserves
R11 plan/repository semantics while preventing a direct external-object review
from being silently retargeted to the review-start commit.

The auxiliary resolver contracts are support records, not additions to the 93
schema catalog. `PhaseFReviewerIdentityV1` is a canonical JSON record with
`reviewer_authority_id`, `authority_kind`, `schema_version`,
`authority_class` (`REAL` or `TEST_ONLY`), `actor_identity_digest`,
`permitted_review_roles`, and derived lifecycle fields. `PhaseFReviewArtifactV1`
has the corresponding identity/class fields plus `reviewer_authority_id`,
`role`, `reviewed_target`, `decision`, canonical `p0_count`/`p1_count`/`p2_count`
strings, `finding_ids`, and `independence_relation`. Both are canonical,
content-addressed, active, and resolved through the graph-declared paths.
`PhaseFImplementationAuthorIdentityV1` supplies the explicit remediation actor
for independence checks. A `TEST_ONLY` record is accepted only by the
explicitly opted-in `real_test` fixture mode and can never authorize real
production mode.

For every direct five-role bundle, the five `reviewer_authority_id` values,
the five review-artifact IDs, and the five resolved `actor_identity_digest`
values are each pairwise distinct. The reviewer IDs and artifact IDs prove
separate authority records; actor-digest uniqueness proves that those records
do not merely represent different IDs for the same underlying actor. A direct
bundle is rejected if either uniqueness layer fails, even when all role,
artifact, and aggregate fields otherwise validate. The migrated-review path
uses the same actor-digest uniqueness predicate.

<a id="schema-def-PhaseFSpecificationBundleApprovalV1"></a>
`SCHEMA_DEF[PhaseFSpecificationBundleApprovalV1]` is the exact six-line
`TAG_BODY` in §3. All six fields are required, have the order and value rules
specified there, and have no nullable, unknown, omitted, duplicate, or
additional-member form. Its complete identity is the SHA-256 of the exact tag
message bytes; it has no JSON semantic ID and no complete-file JSON hash.

<a id="schema-def-PhaseFMigratedFindingReviewV1"></a>
`SCHEMA_DEF[PhaseFMigratedFindingReviewV1]` is the exact top-level object
defined in §5. It has no omitted or additional member form. Its complete-file
identity is computed only after all bound inputs and five finding dispositions
are complete; the identity field is excluded from that computation. The
review object is not a G3 approval and cannot be produced by this remediation.

| identifier | category | exact field-closure pointer | semantic identity / complete-file hash meaning | concrete producer | actual validator | exact stage/set | exact registry behavior | traceability |
|---|---|---|---|---|---|---|---|---|
| PhaseFSpecificationBundleApprovalV1 | TAG_BODY | #schema-def-PhaseFSpecificationBundleApprovalV1 | no JSON semantic ID; SHA-256 of the exact six-line annotated tag-message bytes including the final LF | independent five-role specification-bundle approval gate | exact §3 tag-name/body parser plus target, architecture approval, F0 approval, five component-review, traceability, migrated-finding, aggregate-review, and `approval_decision=GO` validator | G3 specification-bundle approval, after architecture/F0 approvals and all five component reviews | TAG_BODY; Git annotated-tag message only; no registry subject and no registry record | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFSpecificationBundleApprovalV1) |
| PhaseFMigratedFindingReviewV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFMigratedFindingReviewV1 | no registry subject before G3; SHA-256 of the complete canonical review object excluding its own ID field | independent migrated-finding review panel | strict migrated-review schema, closed finding-disposition/count/decision validator, exact bundle-input target, concrete five-role review records and independence, lifecycle, staleness, and hash validator | G2 review prerequisite for the specification bundle | external authority object; registry publication is prohibited before later gate authority | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMigratedFindingReviewV1) |

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

## 5. Migrated-finding review authority

`PhaseFMigratedFindingReviewV1` is the R12 authority object used by
`F-ARCH-022`. It remains a specialized migration object with its own
eight-field `review_records` rows; it does not redefine or replace the exact
seven-field `PhaseFIndependentReviewBundleV1` wire used by the nine graph review
nodes. Its exact top-level fields are:

```text
schema_version,migrated_finding_review_id,target_git_commit,
target_bundle_inputs_sha256,
reviewed_migration_ledger_sha256,reviewed_normative_traceability_matrix_sha256,
reviewed_traceability_manifest_sha256,
reviewed_component_sha256s,reviewed_finding_ids,finding_dispositions,
reviewer_roles,review_records,review_input_fingerprint,
p0_count,p1_count,p2_count,decision,created_stage,producer,validator,
lifecycle,stale,superseded_by,invalidated
```

The ID is the SHA-256 of the complete canonical object with the
`migrated_finding_review_id` value excluded. The target is the exact
target Git commit plus the exact `PhaseFSpecificationBundleInputsV1`
fingerprint, not the final bundle manifest, so the review can exist before the
bundle binds its own review hash.
`reviewed_migration_ledger_sha256`,
`reviewed_normative_traceability_matrix_sha256`,
`reviewed_traceability_manifest_sha256`, and the sorted five-entry
`reviewed_component_sha256s` must equal the inputs used to compute that
fingerprint. `review_input_fingerprint` is the canonical SHA-256 of the target
commit, bundle-input identity, all reviewed source identities, the exact five
finding IDs, and their dispositions. `reviewed_finding_ids` is exactly
`F-PLAN-R11-P1-01`, `F-PLAN-R11-P1-02`, `F-PLAN-R11-P1-03`,
`F-PLAN-R11-P1-04`, and `F-PLAN-R11-P3-01`; `finding_dispositions` contains
exactly one value for each of those IDs from the closed enum
`OPEN|PARTIALLY_CLOSED|PENDING|TECHNICALLY_CLOSED|NON_BLOCKING_DEBT|SUPERSEDED|INVALIDATED`.
`TECHNICALLY_CLOSED` contributes no unresolved count; `NON_BLOCKING_DEBT` is
valid only for a P2 finding and contributes to `p2_count`; all other values
remain unresolved and derive the severity count. Counts are non-negative
integers and are derived from the disposition map, so `decision` is exactly
`NO-GO` for unresolved findings, `GO_WITH_DOCUMENTED_NON_BLOCKING_DEBT` for
P2 debt, and `GO` only when all five findings are technically closed.
`reviewer_roles` is exactly the five roles `scientific_metrology`,
`architecture_data`, `security`, `compatibility`, and
`operations_governance`. `review_records` contains exactly one concrete row per
role with the closed fields `role`, `reviewer_authority_id`,
`reviewed_target`, `review_artifact_id`, `decision`, `review_sha256`,
`lifecycle`, and `independence_relation`; reviewer and artifact identities are
unique, active, distinct, target the exact `review_input_fingerprint`, and
hash their canonical row content. A `GO` migrated review requires every row to
have `decision=GO`; a row with `NO-GO` makes the derived migrated decision
`NO-GO`, regardless of the finding dispositions. The serialized `decision` is
accepted only when it equals the decision derived from both the finding
dispositions and the complete five-row state. The producer is an independent
review panel and never the remediation agent.

The only valid lifecycle is `ACTIVE` with `stale=false`, `invalidated=false`,
and `superseded_by=null`. A changed target commit, architecture plan, F0
authority, component specification, normative matrix, generated traceability
manifest, migration ledger, or bundle-input fingerprint makes the object
stale. Supersession or invalidation is terminal and cannot be repaired in
place; the next bundle requires a new review object and exact hash. The
generator validates identity, target, bound inputs, role independence,
coverage, counts, decision, lifecycle, and staleness. A missing or malformed
object is not an intentional null success: it leaves the checked-in bundle
`DRAFT_NO_AUTHORITY` and blocks G3.

The real resolver discovers this closure from the exact authority graph. Source
specification, matrix, ledger, and generated-manifest nodes are read from the
target commit; all nine direct review bundle files are canonical UTF-8 JSON
under `.phase_f_authority/{node_id}.json` and must have the exact inherited
seven-field wire. Each direct row's
`phase-f-authority://review-artifacts/<64 lowercase hex>` reference resolves to
the graph-declared `.phase_f_authority/review_artifacts/<id>.json` object, whose
full bytes must match the reference SHA and byte length. Reviewer identities
resolve from `.phase_f_authority/reviewer_identities/<id>.json`; every artifact
must bind exactly one permitted role, reviewer, reviewed-target hash, decision,
finding list, and independence relation. The validation context resolves the
explicit canonical remediation-author identity from
`.phase_f_authority/remediation_authority.json` and compares its actor digest
with each reviewer actor digest; no name or prefix convention is authority.
Synthetic records are accepted only by the conformance KAT or explicit
`real_test` fixture opt-in and can never authorize the real mode. The resolver
reports every requested, resolved, missing, and malformed reference in a
structured resolution record.

`PhaseFSpecificationBundleInputsV1.authority_bindings` and the manifest's
`bound_authority_sha256s` are exact maps of the graph's `binds` edges. The
complete `edges` inventory is the single normative binding trust root: all 76
exact edge records have a mandatory `binding_obligation` whose kind is either
`none` or `serialized_binding`. A serialized obligation closes the destination
field, binding category, value semantics, cardinality, and destination object
kind, and the referenced field must exist in that destination's object schema.
There is no implicit absence meaning `none` and no separate selected-membership
authority.

`node.binding_fields`, `binding_semantics`, `serialized_binding_fields`, and
their `all`/`selected`/`none` relation policies are derived mirrors. The
generator derives semantic rules, concrete serialized bindings, builder
expectations, and validator expectations from the edge obligations and then
requires each checked-in mirror to equal that projection. Removing or adding a
downstream mirror entry while the edge root is unchanged rejects. A valid
change to an edge obligation is a normative specification revision: its exact
graph bytes and SHA-256 change, the specification-bundle input fingerprint
changes, and prior dependent review/approval authority must be refreshed.

All seven edge relations remain prerequisite inputs, independently derived as
the complete incoming-edge predecessor set; `hashes` is represented by the
closed G3 tag wire fields rather than an authority-object binding map. The
graph's `edge_contract` is the normative closed set of complete `(from node
ID, relation, to node ID)` tuples, and the serialized `edges` set must equal it
exactly. The graph also carries the closed typed source-kind/relation/
destination-kind contract and closed per-node identity rules used by the
resolver. Exact graph membership, rather than kind-level admissibility,
determines cardinality and authority.

The bundle binds the object through the typed nullable reference
`migrated_finding_review`, whose `schema`, `authority_id`, `sha256`,
`target_git_commit`,
`target_bundle_inputs_sha256`, `reviewed_migration_ledger_sha256`,
`reviewed_normative_traceability_matrix_sha256`,
`reviewed_traceability_manifest_sha256`, `review_records`,
`review_input_fingerprint`, and `review_status` fields are all required. A
future complete bundle must populate every field from one immutable review
object; the present candidate keeps them null/`ABSENT`.
The aggregate review's R11 bundle target is the review-start Git commit; its
source-specific manifest hash is bound through the graph-derived auxiliary
artifact scope. The aggregate's required dependency closure, including the
migrated-review identity, is derived from the graph rather than serialized as
extra fields in the canonical R11 bundle. G3 validates these relationships
through the R12 authority graph.

## 6. Review gate

P0/P1 must both be zero. Ambiguous schema/hash/ID/relation/tag grammar, missing
usage row, conflicting closure, or an absent upstream bundle binding is P1.
This candidate grants no authority until independently reviewed GO and included
in an approved G3 manifest.
