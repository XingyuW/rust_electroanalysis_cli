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
| <a id="F-WIRE-005"></a>`F-WIRE-005` | `F-ARCH-015,F-ARCH-017,F-OD-04,F-OD-13,F-OD-14,F-OD-15,F-OD-16` | Ed25519 keys/signatures, enrollment, trust bindings, registry record/head, object/record/relation kinds, subject hashes, relation ordering, genesis, sequence, resolver, compromise, emergency wire, and the additive pre-G0 reviewer bootstrap root/currentness/subject-registry and verifier-issued actor-attestation contracts are exact and fail closed. | §§5.2, 8, 9, 15 emergency wire, 53.7 anchors |
| <a id="F-WIRE-006"></a>`F-WIRE-006` | `F-ARCH-012..016,F-ARCH-021` | Retrieval, package, dependency, physical identity/custody, power, metrology, cohort, release, claim-state, monitoring, incident, resolution, and retention schema field closures are exact; scientific/operational interpretation remains with its owning spec. | §§10–15, 53.7 anchors |
| <a id="F-WIRE-007"></a>`F-WIRE-007` | `F-ARCH-008,F-ARCH-009,F-ARCH-010` | Every durable tag is annotated and uses exact target/body/prerequisite validation. Add `ism-mechanism-health-v1-f-specification-bundle-approved` with the exact ordered body fields `phase_f_architecture_plan_tag`, `phase_f_f0_decisions_tag`, `specification_bundle_manifest_sha256`, `aggregate_review_bundle_sha256`, `approval_decision`, `schema_version`; their exact values and byte grammar are defined in §3, and `approval_decision` is `GO`. | §6 plus this row |
| <a id="F-WIRE-008"></a>`F-WIRE-008` | `F-ARCH-004,F-ARCH-005,F-ARCH-017` | The current schema set has exactly the 91 R11 identifiers plus `PhaseFSpecificationBundleApprovalV1`, `PhaseFMigratedFindingReviewV1`, `PhaseFReviewerActorAttestationV1`, `PhaseFReviewerBootstrapTrustRootV1`, `PhaseFReviewerBootstrapCurrentnessProofV1`, and `PhaseFReviewerBootstrapAcceptedHeadCheckpointV1`; every schema has one definition anchor, category, exact field closure, identity/hash rule, producer, validator, stage, registry behavior, and exhaustive nested usage rows. | §§53.7, 53.12 |
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
`PhaseFSpecificationBundleApprovalV1`, `PhaseFMigratedFindingReviewV1`,
`PhaseFReviewerActorAttestationV1`, `PhaseFReviewerBootstrapTrustRootV1`, and
`PhaseFReviewerBootstrapCurrentnessProofV1`, and
`PhaseFReviewerBootstrapAcceptedHeadCheckpointV1` rows below. The generator checks this set
equality and validates every new row's anchor and every non-empty catalog
dimension; there is no wildcard or parallel approval catalog.

### 4.1 Pre-G0 reviewer bootstrap trust domain

The first reviewer cannot be rooted in the normal Phase-F authority enrollment:
that enrollment is deliberately downstream of G4/G5. The normative graph
therefore declares one narrow terminal trust domain at
`PRE_G0_REVIEWER_BOOTSTRAP`. Its graph-pinned genesis root identity and
public-key fingerprint are immutable, while roots and proofs are discovered
from content-addressed history directories. A caller cannot select a different
key or trust root at validation time. The graph contract uses zero placeholders
in this candidate because no real root is provisioned by this remediation; a
subsequent reviewed target must replace them with the externally provisioned
root identity.

<a id="schema-def-PhaseFReviewerBootstrapTrustRootV1"></a>
`SCHEMA_DEF[PhaseFReviewerBootstrapTrustRootV1]` is the immutable terminal
pre-G0 root object. Its exact fields are:

```text
root_id,authority_kind,schema_version,authority_class,stage,root_public_key,
root_public_key_fingerprint,authority_scope,subject_uniqueness_policy,
evidence_retention_policy,rotation_policy,compromise_policy,predecessor_root_id,
predecessor_root_sha256,replacement_authority,effective_sequence,
replacement_reason,replacement_status,replacement_signature,lifecycle,stale,
superseded_by,invalidated
```

`root_id` is the SHA-256 of the domain-separated JCS payload excluding
`root_id` and `replacement_signature`; the complete-file SHA is computed after
the object is complete. A genesis root has null predecessor/replacement fields,
`effective_sequence=0`, and `replacement_status=GENESIS`. A replacement has an
exact predecessor ID and complete-file SHA, `effective_sequence` exactly one
greater than its predecessor, a non-empty reason, and an Ed25519
`replacement_signature` made by the predecessor root key. Root history has one
genesis, one child per predecessor, and no disconnected or competing lineage.
`stage` is exactly `PRE_G0_REVIEWER_BOOTSTRAP`, `authority_scope` is exactly
`reviewer_actor_attestation`, `reviewer_currentness`, and
`reviewer_subject_registry`, and the root is limited to those purposes. The
root is not a reviewer, architecture authority, approval, release, or general
registry mutation authority. `rotation_policy` is
`forward_signed_replacement_with_predecessor_signature`; `compromise_policy` is
`immediate_reject`. A root that is stale, superseded, invalidated, revoked, or
compromised cannot authorize a proof. The external authority retains identity
evidence outside this file and exposes only evidence hashes here.

<a id="schema-def-PhaseFReviewerBootstrapCurrentnessProofV1"></a>
`SCHEMA_DEF[PhaseFReviewerBootstrapCurrentnessProofV1]` is the signed current
snapshot rooted in that terminal key. Its exact fields are:

```text
currentness_proof_id,authority_kind,schema_version,authority_class,stage,
root_id,root_sha256,sequence,previous_proof_id,previous_proof_sha256,head_id,
current_verifier_authority_id,current_verifier_public_key,
current_verifier_public_key_fingerprint,subject_registry_head_sha256,
subject_bindings,valid_from,valid_until,root_lifecycle,root_revoked,
root_compromised,root_superseded_by,verifier_lifecycle,verifier_revoked,
verifier_compromised,verifier_superseded_by,lifecycle,stale,superseded_by,
invalidated,signature
```

The proof ID excludes only `currentness_proof_id` and `signature`; `head_id`
is derived from the same proof payload with both identity fields excluded.
The root signs `currentness_proof_id` and every other field except
`signature` using Ed25519 and the domain
`mhi_phase_f_reviewer_bootstrap_currentness_proof_v1`. `sequence=0` is the
pre-G0 genesis proof and has no predecessor or predecessor hash; later proofs
require an exact predecessor ID and complete-file SHA and a strictly newer
sequence. The resolver validates every proof in the history, not only the
selected head. `subject_bindings` is sorted and contains the
closed fields `actor_subject_id`, `identity_evidence_sha256`, and
`subject_status`; one evidence hash may map to only one active subject. The
subject-head hash is recomputed from the exact sorted bindings and sequence.
`valid_from <= validation_time <= valid_until` is required for the current
head. Root and verifier lifecycle, revocation, compromise, and supersession
fields are signed state, not caller-provided booleans. A signed sequence is not
itself freshness: the resolver-owned accepted-head checkpoint is required, is
never lowered, and advances only after complete-history validation. A missing,
malformed, stale, forked, revoked,
compromised, superseded, or incorrectly signed proof fails closed.

The selected transition policy is permanent bootstrap provenance for reviewer
identities. Normal G5 enrollment may establish later Phase-F authorities, but
it cannot replace the trust source of an existing reviewer actor. Therefore
historical review artifacts remain verifiable across normal-registry creation,
bootstrap verifier rotation, or later authority enrollment.

<a id="schema-def-PhaseFReviewerBootstrapAcceptedHeadCheckpointV1"></a>
`SCHEMA_DEF[PhaseFReviewerBootstrapAcceptedHeadCheckpointV1]` is the
resolver-owned persistent accepted-head watermark. Its exact fields are:

```text
checkpoint_id,authority_kind,schema_version,authority_class,stage,root_id,
current_proof_id,current_proof_sha256,current_sequence,
current_subject_registry_head_sha256,accepted_at,lifecycle,stale,
superseded_by,invalidated
```

The checkpoint ID is a domain-separated SHA-256 of the canonical payload
excluding `checkpoint_id`; it binds the current root, proof, complete-file SHA,
sequence, and subject-registry head. It is stored in resolver-controlled
persistent state outside the authority repository and is advanced with an
atomic same-directory replace after validation. Missing, malformed, lower, or
same-sequence/different-proof checkpoints fail closed. A new verifier therefore
requires explicit trusted checkpoint initialization; it never infers the
present head from an old signed proof alone.

### 4.2 REAL reviewer actor attestation and identity derivation

The existing identity/trust capability inventory is closed as follows. The
reviewer actor contract reuses the Ed25519, lifecycle, and compromise
primitives, while the pre-G0 bootstrap root owns only reviewer attestation,
currentness, and subject-registry trust. Normal authority enrollment and the
Phase-F registry remain downstream primitives and cannot bootstrap the first
reviewer.

| Existing primitive | Stable actor anchor? | Enrollment approval? | Role eligibility? | Alias prevention? | Lifecycle/currentness? | Suitable for reviewer identity? |
|---|---|---|---|---|---|---|
| `PhaseFAuthorityEnrollmentV1` | Enrolled authority/key subjects; no reviewer subject by itself | No; unsigned payload | No | No | Bound by the existing enrollment/tag/registry prerequisites | Trust root input, not sufficient alone |
| `PhaseFAuthorityEnrollmentApprovalV1` annotated tag | No subject; approves exact enrollment bytes and keys | Yes | No | Indirectly, through exact enrollment binding | Existing tag/target/review validation | Approval prerequisite, not actor identity |
| Ed25519 public keys/signatures | Cryptographic key, not an underlying natural-person subject | No | No | No; key rotation must not define a new actor | Signature validity only | Issuer authentication primitive |
| `PhaseFRegistryRecordV1` subjects/relations | Persistent registry subject when enrolled | Via the enrolled registry authority | Existing relation semantics, not reviewer role semantics | Registry subject uniqueness/currentness | Yes; sequence, predecessor, revocation, compromise, and supersession rules | Issuer/currentness primitive |
| `PhaseFRegistryHeadV1` and live resolver | Registry state anchor, not actor identity | Resolves the current enrolled registry state | No reviewer role by itself | Prevents stale/forked registry state | Yes; freshness, chain, equivocation, regression, and compromise rules | Current trust-state prerequisite |
| Existing revocation/compromise/supersession controls | No new subject | No | No | Removes invalid/stale authority | Yes | Required rejection inputs |
| `PhaseFReviewerIdentityV1` | No; reviewer authority ID is role-specific | No | Previously caller-declared only | No in the former contract | Yes | Requires the additive attestation binding below |
| `PhaseFReviewArtifactV1` / `PhaseFIndependentReviewBundleV1` | No; references reviewer authorities | No | No | Pairwise reviewer/actor checks only after identity resolution | Yes | Downstream consumers only |
| Git commit/tagger/pusher/session identity | No trusted actor anchor | No | No | No | No | Explicitly unsuitable |

The single terminal REAL actor trust root is the graph-pinned
`PhaseFReviewerBootstrapTrustRootV1`, followed by a valid signed
`PhaseFReviewerBootstrapCurrentnessProofV1` and one verifier-issued
`PhaseFReviewerActorAttestationV1`. No caller, Git identity, reviewer record,
normal G5 enrollment, or parallel reviewer-specific trust root can replace that
chain.

`PhaseFReviewerActorAttestationV1` is the one additive R12 reviewer-identity
contract. It is an external signed authority object, not a new R11 registry
record kind and not a reviewer back-pointer. Its exact closed field set is:

```text
attestation_id,authority_kind,schema_version,actor_subject_id,actor_class,
actor_identity_evidence_sha256,trust_source,eligible_role,
role_eligibility_evidence_sha256,
independence_evidence_sha256,independence_excluded_actor_identity_digest,
eligibility_verifier_authority_id,independence_verifier_authority_id,
created_at,lifecycle,stale,superseded_by,invalidated,signature
```

`authority_kind` is exactly `PhaseFReviewerActorAttestationV1`,
`schema_version` is `1`, and `actor_class` is exactly `natural_person`.
Software, AI, Git, commit, tagger, pusher, email, display name, organization,
or session-generated values are not eligible actor subjects. The
`actor_subject_id` is an opaque, stable, runtime-stable ID issued by the
pre-G0 bootstrap subject registry. The authority keeps the
identity and anti-alias evidence outside this canonical file, issues one
subject for one underlying natural person, preserves that subject across key
or enrollment rotation, and never places PII in this contract. The two
identity, role, and independence evidence fields are opaque SHA-256 references to
that authority's retained evidence; hashes alone are not identity authority.

The attestation binds one exact tagged `trust_source` object:

```text
trust_source={
  "type":"reviewer_bootstrap",
  "root_id":"sha256:<root semantic ID>",
  "root_sha256":"<complete root-file SHA-256>",
  "currentness_proof_id":"sha256:<proof semantic ID>",
  "currentness_proof_sha256":"<complete proof-file SHA-256>"
}
```

Both `eligibility_verifier_authority_id` and
`independence_verifier_authority_id` must equal the current verifier authority
ID in that signed proof. The proof binds the verifier key to the graph-pinned
bootstrap root; the verifier is the sole attestation issuer. The actor does not
self-attest, and an implementation author cannot issue or materialize a
reviewer attestation. `eligible_role` is exactly one of the five
independent-review roles. `independence_excluded_actor_identity_digest` must
equal the separately resolved remediation-author digest, so an actor cannot
review its own remediation. Root/proof currentness, subject-head uniqueness,
sequence/predecessor, lifecycle, revocation, compromise, supersession, and
signature checks are prerequisites; a stale, revoked, superseded, invalidated,
or untrusted bootstrap object or attestation is rejected.

The attestation ID is content-derived using the inherited R11 §3 form with a
new unique domain and exact exclusions:

```text
attestation_id = "sha256:" + lowercase_hex(
    SHA256(ASCII("mhi_phase_f_reviewer_actor_attestation_v1") || 0x00 ||
           JCS(attestation_without_attestation_id_and_signature))
)
```

The signature is lowercase 64-byte Ed25519 and covers
`ASCII("mhi_phase_f_reviewer_actor_attestation_v1") || 0x00 ||
JCS(attestation_without_signature)`. Verification uses the existing strict
Ed25519 semantics and the exact current verifier public key from the signed
bootstrap proof. Complete-file SHA-256 is computed only after the signature is
present. The reviewer record
must carry both `actor_attestation_id` and an immutable reference with the
attestation's complete-file SHA and byte length. Its existing raw
`actor_identity_digest` is no longer caller-chosen in REAL mode; it must equal
exactly:

```text
lowercase_hex(SHA256(
    ASCII("mhi_phase_f_reviewer_actor_identity_v1") || 0x00 ||
    JCS({"actor_subject_id": actor_subject_id})
))
```

Only the opaque subject participates in this derivation. Role, display name,
email, organization, key bytes, enrollment ID, evidence hashes, lifecycle,
and timestamps do not, so legitimate metadata and key rotation preserve the
actor identity. Different reviewer IDs, keys, or attestations carrying one
subject therefore collide on the derived actor digest and fail the existing
five-role pairwise-independence predicate. A fake/missing attestation,
arbitrary digest, subject substitution, cross-wired role, five-key alias,
self-declaration, or materializer-supplied evidence cannot satisfy REAL
review.

The R12 resolver loads the graph-pinned bootstrap root and exact currentness
proof from `.phase_f_authority/reviewer_bootstrap/`, validates complete-file
hashes, root/proof IDs, the root signature, subject registry head, validity
window, lifecycle, and verifier key, and then resolves each verifier-signed
attestation. The canonical repository intentionally contains no real root or
proof; production `real` resolution therefore fails closed until an
externally provisioned, independently reviewed target supplies those exact
objects. The isolated `real_test` fixture supplies explicitly marked
`TEST_ONLY` root/proof/attestation material only for conformance. TEST_ONLY
material can never authorize REAL mode. This additive contract changes no R11
field, R11 registry record kind, registry relation enum, or 28-node/76-edge
R12 authority graph.

### 4.3 Canonical independent-review bundle wire

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

`PhaseFReviewerIdentityV1` and `PhaseFReviewArtifactV1` remain auxiliary
resolver records; the signed actor attestation above is the one additive R12
schema. `PhaseFReviewerIdentityV1` is a canonical JSON record with
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
values derived from verified attestation subjects are each pairwise distinct.
The reviewer IDs and artifact IDs prove separate authority records;
attestation-derived actor-digest uniqueness proves that those records do not
merely represent different IDs, keys, or attestations for the same underlying
actor. A direct bundle is rejected if either uniqueness layer fails, even when
all role, artifact, and aggregate fields otherwise validate. The
migrated-review path uses the same actor-digest and attestation predicates.

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

<a id="schema-def-PhaseFReviewerActorAttestationV1"></a>
`SCHEMA_DEF[PhaseFReviewerActorAttestationV1]` is the exact signed external
authority object defined in §4.2. It has no omitted, additional, or unsigned
alternate form; its semantic ID, signature preimage, complete-file SHA, and
bootstrap-root/currentness requirements are exact.

| identifier | category | exact field-closure pointer | semantic identity / complete-file hash meaning | concrete producer | actual validator | exact stage/set | exact registry behavior | traceability |
|---|---|---|---|---|---|---|---|---|
| PhaseFSpecificationBundleApprovalV1 | TAG_BODY | #schema-def-PhaseFSpecificationBundleApprovalV1 | no JSON semantic ID; SHA-256 of the exact six-line annotated tag-message bytes including the final LF | independent five-role specification-bundle approval gate | exact §3 tag-name/body parser plus target, architecture approval, F0 approval, five component-review, traceability, migrated-finding, aggregate-review, and `approval_decision=GO` validator | G3 specification-bundle approval, after architecture/F0 approvals and all five component reviews | TAG_BODY; Git annotated-tag message only; no registry subject and no registry record | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFSpecificationBundleApprovalV1) |
| PhaseFMigratedFindingReviewV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFMigratedFindingReviewV1 | no registry subject before G3; SHA-256 of the complete canonical review object excluding its own ID field | independent migrated-finding review panel | strict migrated-review schema, closed finding-disposition/count/decision validator, exact bundle-input target, concrete five-role review records and independence, lifecycle, staleness, and hash validator | G2 review prerequisite for the specification bundle | external authority object; registry publication is prohibited before later gate authority | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMigratedFindingReviewV1) |
| PhaseFReviewerActorAttestationV1 | SIGNED_EXTERNAL_AUTHORITY | #schema-def-PhaseFReviewerActorAttestationV1 | sha256:<lowercase_hex>; SHA-256 of the domain-separated JCS semantic payload excluding attestation_id and signature; complete-file SHA-256 covers every field including signature | reviewer-bootstrap-verifier-issued natural-person reviewer actor eligibility and independence attestation | strict schema, domain-separated identity derivation, tagged trust-source binding, subject-registry anti-alias, role evidence, lifecycle, currentness, and strict Ed25519 signature verification | REAL reviewer identity prerequisite for every five-role review bundle | external signed authority object; rooted in the permanent pre-G0 bootstrap domain; no reviewer back-pointer or downstream registry enrollment is permitted | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReviewerActorAttestationV1) |
| PhaseFReviewerBootstrapTrustRootV1 | EXTERNAL_TRUST_ANCHOR | #schema-def-PhaseFReviewerBootstrapTrustRootV1 | sha256:<lowercase_hex>; SHA-256 of the domain-separated canonical semantic payload excluding root_id and replacement_signature; complete-file SHA-256 covers every field | normative terminal pre-G0 reviewer bootstrap trust root and subject-uniqueness policy | strict schema, graph-pinned root identity and key fingerprint, narrow purpose scope, lifecycle, rotation, and compromise validation | PRE_G0_REVIEWER_BOOTSTRAP; before G0 and every downstream review gate | immutable external trust anchor; not a Phase F registry record and cannot authorize scientific, architecture, release, or unrelated registry mutations | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReviewerBootstrapTrustRootV1) |
| PhaseFReviewerBootstrapCurrentnessProofV1 | SIGNED_EXTERNAL_AUTHORITY | #schema-def-PhaseFReviewerBootstrapCurrentnessProofV1 | sha256:<lowercase_hex>; SHA-256 of the domain-separated canonical semantic payload excluding currentness_proof_id and signature; complete-file SHA-256 covers every field including signature | root-signed pre-G0 reviewer verifier, subject-registry, and currentness snapshot | strict schema, root signature, root binding, sequence/head, validity window, verifier key, subject-head uniqueness, revocation, compromise, and supersession validation | PRE_G0_REVIEWER_BOOTSTRAP; current proof required before every REAL reviewer identity | external signed authority object; bootstrap reviewer trust only and no architecture, release, or downstream approval authority | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReviewerBootstrapCurrentnessProofV1) |
| PhaseFReviewerBootstrapAcceptedHeadCheckpointV1 | RESOLVER_STATE | #schema-def-PhaseFReviewerBootstrapAcceptedHeadCheckpointV1 | sha256:<lowercase_hex>; SHA-256 of the domain-separated canonical semantic payload excluding checkpoint_id; complete-file SHA-256 covers every field | resolver-owned accepted currentness-head watermark | strict schema, checkpoint identity, root/proof/complete-file binding, monotonic sequence, fork detection, and atomic persistence validation | PRE_G0_REVIEWER_BOOTSTRAP; resolver state required before every REAL reviewer identity | resolver state outside the authority repository; never a registry subject and never an approval or signing authority | INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReviewerBootstrapAcceptedHeadCheckpointV1) |

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
