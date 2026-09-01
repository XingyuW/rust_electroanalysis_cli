# Phase F Conformance and KAT Specification

## 1. Authority and isolation

This G2 candidate is the sole authority for executable specification tests. It
refines `F-ARCH-002`, `F-ARCH-017`, `F-ARCH-022`, and all implementation-
affecting child requirements. Nothing here is physical evidence. No KAT,
fixture, property, transcript, synthetic value, or constructed graph may
satisfy an `F-EV` oracle.

## 2. Requirements

| ID | derived_from | Normative requirement | Adopted R11 clauses |
|---|---|---|---|
| <a id="F-CNF-001"></a>`F-CNF-001` | `F-ARCH-017,F-WIRE-001..009` | Production-schema KATs use complete schema-valid bytes and verify parsing, JCS, semantic IDs, complete-file SHA, signatures, registry order/relations, field closure, and negative mutations. | §§44, 49, 53.4–53.5, 53.9–53.10 |
| <a id="F-CNF-002"></a>`F-CNF-002` | `F-ARCH-017,F-OPS-006,F-OPS-007` | Storage-copy KATs test opaque bytes only after prevalidated kind/SHA identity and test URI, length, availability, freshness, distinctness, count, and exact membership. They never assert production-schema validity. | §§53.2–53.4, R11-KAT-RETENTION-COPY |
| <a id="F-CNF-003"></a>`F-CNF-003` | `F-ARCH-017,F-OPS-003,F-SCI-002..008` | Properties cover quantified and nonliteral behavior, including complete monitoring mapping, independence, pseudoreplication, power, claim ceilings, retention counts, and cycle freedom. A property cannot claim fixture PASS. | §§20, 53.6, 53.9–53.10 |
| <a id="F-CNF-004"></a>`F-CNF-004` | `F-ARCH-008..010,F-WIRE-007` | Literal parser KATs cover every approval tag including the new specification-bundle tag; the R12 vector uses the strict `(tag_name,body_bytes)` checker below, while separate real-Git properties verify annotated type, peel target, reachability, and upstream tag resolution. | §§53.5, 53.9–53.10 plus R12 tag grammar |
| <a id="F-CNF-005"></a>`F-CNF-005` | `F-ARCH-017,F-ARCH-022,F-WIRE-008` | Constructive audits verify the explicit authority DAG, no future/self/reference cycles, schema/anchor/catalog equality, exhaustive usage matrix, traceability inverse, closed test/evidence/KAT catalogs, owner-decision union, Markdown structure, and no duplicate current traceability graph. | §§53.6–53.13 |
| <a id="F-CNF-006"></a>`F-CNF-006` | `F-ARCH-017` | Every literal KAT declares all required inputs, exact fixture bytes, semantic IDs, SHA-256, byte length, URI when applicable, operation, expected result, decoded fields where applicable, and individually executable negative mutations. Missing data or an incomplete PASS is P1. | §§53.9–53.10 |
| <a id="F-CNF-007"></a>`F-CNF-007` | `F-ARCH-002,F-ARCH-012,F-ARCH-017,F-SCI-009..010` | Automated isolation checks prove zero KAT/test/synthetic/constructed-to-real-evidence promotion paths and preserve every future-real-evidence oracle. | §§53.8 R11-14/R11-19, 53.11 |
| <a id="F-CNF-008"></a>`F-CNF-008` | `F-ARCH-003,F-ARCH-017,F-IMPL-006` | Regression checks preserve frozen Phase-E behavior, the P2 gate, production runner order, and all previously closed safety contracts. | §§19–20, 53.8 R11-17 |

## 3. Current executable catalog

R11 §53.10 is adopted unchanged as the current 28-row test catalog: eight
literal executable KATs, four property tests, and sixteen constructive audits.
Exact fixture bytes, semantic IDs, SHA values, lengths, test-only URIs,
expected results, and mutations in §§44–46 and §§53.2–53.6 remain test
authority here. Historical §§16–52 are provenance only except where a current
§53 row explicitly adopts a literal value.

The current catalog is the 28 adopted R11 rows plus the complete R12 catalog below. The
generator parses the adopted R11 table from the preserved source and this R12
row from this document, then rejects every reference outside that closed
catalog. `R12-POS-SPEC-BUNDLE-TAG` is a literal KAT and its fixture is never a
real approval or an `F-EV`.

| test_id | kat_class | fixture_scope | prevalidated_inputs | literal_inputs | operation | expected_result | negative_mutation | UNBOUND_REQUIRED_INPUT_COUNT |
|---|---|---|---|---|---|---|---|---:|
| R12-POS-SPEC-BUNDLE-TAG | literal_kat | g3_specification_bundle_tag | none | exact tag name, exact 379-byte body, body SHA-256, decoded six fields | `validate_g3_tag(tag_name, body_bytes, synthetic_context)` | PASS with exact decoded fields | the individually defined `R12-NEG-G3-*` rows below | 0 |
| R12-G3-AUTHORITY-CONTEXT-POS | literal_kat | synthetic_complete_g3_authority | complete synthetic R12 graph and prerequisite objects | exact canonical tag body plus all prerequisite identities | `validate_g3_tag(tag_name, body_bytes, synthetic_context)` | PASS | none | 0 |
| R12-G3-ARCHITECTURE-REVIEW-BUNDLE-POSITIVE | constructive_plan_audit | canonical_r11_review_bundle | all nine Phase F review nodes use the exact inherited R11 seven-field bundle, with five ordered role rows, distinct reviewer IDs/actor digests/artifacts, and resolved immutable artifacts; external-object and Git target variants are checked from graph predecessors | canonical direct review bundle fixture and target-type matrix | `validate_g3_tag(...)` | PASS | none | 0 |
| R12-G3-ARCHITECTURE-REVIEW-NEGATIVE-MATRIX | constructive_plan_audit | canonical_r11_review_bundle | exhaustive direct-bundle mutations cover missing/duplicate roles, duplicate actor digests, reviewer and artifact identity/reference mismatches, stale state, every wrong target type/digest shape, counts, fields, and TEST_ONLY classification | direct review-bundle mutation and ten-pair actor-independence matrix | `validate_g3_tag(...)` | REJECT | every direct-bundle mutation | 0 |
| R12-G3-REVIEW-START-GIT-PUBLISHED | constructive_plan_audit | review_start_git_state | review target equals HEAD, local `main`, `origin/main`, and independently supplied live remote `main` SHA after safe exact-SHA publication handoff | exact commit anchors and isolated bare-remote publication fixture | `validate_review_start_git_state(...); publish_reviewed_sha_with_lease(...)` | PASS | none | 0 |
| R12-G3-REVIEW-START-GIT-MISMATCH | constructive_plan_audit | review_start_git_state | each of HEAD, local main, origin/main, and live remote main differs from the reviewed target; publication dirty/race/non-fast-forward/unavailable-live failures also reject | one-anchor mismatch and isolated publication failure matrix | `validate_review_start_git_state(...); validate_safe_publication_preflight(...)` | REJECT | review-start or safe-publication precondition mismatch | 0 |
| R12-G3-MISSING-ARCH-APPROVAL | constructive_plan_audit | g3_prerequisite | complete synthetic context with architecture approval removed | same as positive | `validate_g3_tag(...)` | REJECT | missing architecture approval | 0 |
| R12-G3-STALE-ARCH-APPROVAL | constructive_plan_audit | g3_prerequisite | complete synthetic context with stale architecture approval | same as positive | `validate_g3_tag(...)` | REJECT | stale architecture approval | 0 |
| R12-G3-MISSING-F0-APPROVAL | constructive_plan_audit | g3_prerequisite | complete synthetic context with F0 approval removed | same as positive | `validate_g3_tag(...)` | REJECT | missing F0 approval | 0 |
| R12-G3-WRONG-F0-TARGET | constructive_plan_audit | g3_prerequisite | complete synthetic context with wrong F0 target | same as positive | `validate_g3_tag(...)` | REJECT | wrong F0 target | 0 |
| R12-G3-MISSING-COMPONENT-REVIEW | constructive_plan_audit | g3_prerequisite | complete synthetic context with one component review removed | same as positive | `validate_g3_tag(...)` | REJECT | missing component review | 0 |
| R12-G3-STALE-COMPONENT-REVIEW | constructive_plan_audit | g3_prerequisite | complete synthetic context with stale component review | same as positive | `validate_g3_tag(...)` | REJECT | stale component review | 0 |
| R12-G3-MISSING-MIGRATED-REVIEW | constructive_plan_audit | g3_prerequisite | complete synthetic context with migrated review removed | same as positive | `validate_g3_tag(...)` | REJECT | missing migrated review | 0 |
| R12-G3-MIGRATED-WRONG-BUNDLE | constructive_plan_audit | g3_prerequisite | migrated review targets another bundle-input fingerprint | same as positive | `validate_g3_tag(...)` | REJECT | migrated review target mismatch | 0 |
| R12-G3-MIGRATED-WRONG-LEDGER | constructive_plan_audit | g3_prerequisite | migrated review names another migration-ledger digest | same as positive | `validate_g3_tag(...)` | REJECT | migrated review ledger mismatch | 0 |
| R12-G3-MIGRATED-WRONG-COMMIT | constructive_plan_audit | g3_prerequisite | migrated review targets another Git commit | same as positive | `validate_g3_tag(...)` | REJECT | migrated review target commit mismatch | 0 |
| R12-G3-MIGRATED-HASH-MISMATCH | constructive_plan_audit | g3_prerequisite | migrated review identity does not match its exact bytes | same as positive | `validate_g3_tag(...)` | REJECT | migrated review hash mismatch | 0 |
| R12-G3-MIGRATED-INCOMPLETE-DISPOSITION | constructive_plan_audit | g3_prerequisite | one required migrated finding disposition removed | same as positive | `validate_g3_tag(...)` | REJECT | incomplete migrated finding coverage | 0 |
| R12-G3-MIGRATED-STALE | constructive_plan_audit | g3_prerequisite | migrated review is marked stale | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review | 0 |
| R12-G3-MIGRATED-SUPERSEDED | constructive_plan_audit | g3_prerequisite | migrated review has a superseding authority | same as positive | `validate_g3_tag(...)` | REJECT | superseded migrated review | 0 |
| R12-G3-MIGRATED-NON-INDEPENDENT | constructive_plan_audit | g3_prerequisite | migrated review uses a non-independent producer | same as positive | `validate_g3_tag(...)` | REJECT | non-independent migrated review | 0 |
| R12-G3-MISSING-AGGREGATE | constructive_plan_audit | g3_prerequisite | complete synthetic context with aggregate review removed | same as positive | `validate_g3_tag(...)` | REJECT | missing aggregate review | 0 |
| R12-G3-AGGREGATE-WRONG-BUNDLE | constructive_plan_audit | g3_prerequisite | aggregate review targets another manifest | same as positive | `validate_g3_tag(...)` | REJECT | aggregate target mismatch | 0 |
| R12-G3-AGGREGATE-HASH-MISMATCH | constructive_plan_audit | g3_prerequisite | aggregate review identity does not match its exact bytes | same as positive | `validate_g3_tag(...)` | REJECT | aggregate hash mismatch | 0 |
| R12-G3-MANIFEST-HASH-MISMATCH | constructive_plan_audit | g3_prerequisite | manifest identity does not match its exact bytes | same as positive | `validate_g3_tag(...)` | REJECT | manifest hash mismatch | 0 |
| R12-G3-MANIFEST-CHANGED | constructive_plan_audit | g3_prerequisite | manifest bytes changed after aggregate review | same as positive | `validate_g3_tag(...)` | REJECT | stale manifest | 0 |
| R12-G3-WRONG-COMMIT | constructive_plan_audit | g3_git_authority | annotated tag peels to another commit | same as positive | `validate_g3_tag(...)` | REJECT | target mismatch | 0 |
| R12-G3-LIGHTWEIGHT-TAG | constructive_plan_audit | g3_git_authority | lightweight tag supplied where annotated tag is required | same as positive | `validate_g3_tag(...)` | REJECT | lightweight tag | 0 |
| R12-G3-MISSING-REAL-PREREQUISITES | constructive_plan_audit | real_repository_authority | current repository with no actual approvals | canonical body | `validate_g3_tag(...)` | REJECT | missing real G3 authority | 0 |
| R12-G3-SYNTHETIC-CANNOT-AUTHORIZE-REAL | constructive_plan_audit | authority_isolation | synthetic context marked as real-authority request | same as positive | `validate_g3_tag(...)` | REJECT | synthetic authority isolation | 0 |
| R12-G3-MIGRATED-DISPOSITION-ENUM | constructive_plan_audit | migrated_review_authority | complete synthetic context with an unresolved or unknown finding disposition | same as positive | `validate_g3_tag(...)` | REJECT | closed disposition/count/decision mismatch | 0 |
| R12-G3-MIGRATED-REVIEW-RECORDS | constructive_plan_audit | migrated_review_authority | complete synthetic and real contexts with missing, duplicate, stale, role-mismatched, unresolved, non-independent, or non-GO review rows; exhaustive five-role state and identity probes | same as positive | `validate_g3_tag(...)` | REJECT | review-record closure, identity, or unanimous decision mismatch | 0 |
| R12-G3-MIGRATED-INPUT-FINGERPRINT | constructive_plan_audit | migrated_review_authority | complete synthetic context with a stale derived review-input fingerprint | same as positive | `validate_g3_tag(...)` | REJECT | review-input fingerprint mismatch | 0 |
| R12-G3-REAL-FORMAT-POSITIVE | constructive_plan_audit | isolated_real_repository_authority | isolated repository with canonical authority JSON, annotated tags, and real Git target | real-format fixture | `make_repository_context(...); validate_g3_tag(...)` | PASS | resolved real authority closure | 0 |
| R12-G3-REAL-FORMAT-NEGATIVE-MATRIX | constructive_plan_audit | isolated_real_repository_authority | real-format fixture mutated across missing, malformed, stale, wrong-target, wrong-hash, and tag cases | real-format fixture mutation matrix | `make_repository_context(...); validate_g3_tag(...)` | REJECT | every real-format mutation rejected | 0 |
| R12-G3-MIGRATED-PENDING-P1 | constructive_plan_audit | migrated_review_authority | one P1 finding is pending | same as positive | `validate_g3_tag(...)` | REJECT | pending P1 | 0 |
| R12-G3-MIGRATED-OPEN-P1 | constructive_plan_audit | migrated_review_authority | one P1 finding is open | same as positive | `validate_g3_tag(...)` | REJECT | open P1 | 0 |
| R12-G3-MIGRATED-PARTIALLY-CLOSED-P1 | constructive_plan_audit | migrated_review_authority | one P1 finding is partially closed | same as positive | `validate_g3_tag(...)` | REJECT | partially closed P1 | 0 |
| R12-G3-MIGRATED-COUNT-MISMATCH | constructive_plan_audit | migrated_review_authority | disposition-derived P1/P2 counts disagree with the object | same as positive | `validate_g3_tag(...)` | REJECT | count/disposition mismatch | 0 |
| R12-G3-MIGRATED-MISSING-FINDING | constructive_plan_audit | migrated_review_authority | one required finding ID is absent | same as positive | `validate_g3_tag(...)` | REJECT | missing finding | 0 |
| R12-G3-MIGRATED-DUPLICATE-FINDING | constructive_plan_audit | migrated_review_authority | reviewed finding ID list contains a duplicate | same as positive | `validate_g3_tag(...)` | REJECT | duplicate finding | 0 |
| R12-G3-MIGRATED-UNKNOWN-FINDING | constructive_plan_audit | migrated_review_authority | disposition map contains an unknown finding ID | same as positive | `validate_g3_tag(...)` | REJECT | unknown finding | 0 |
| R12-G3-MIGRATED-MISSING-ROLE | constructive_plan_audit | migrated_review_authority | one required review role is absent | same as positive | `validate_g3_tag(...)` | REJECT | missing review role | 0 |
| R12-G3-MIGRATED-DUPLICATE-ROLE | constructive_plan_audit | migrated_review_authority | two review rows claim one role | same as positive | `validate_g3_tag(...)` | REJECT | duplicate review role | 0 |
| R12-G3-MIGRATED-INDEPENDENCE | constructive_plan_audit | migrated_review_authority | a review row violates the distinct-reviewer independence relation | same as positive | `validate_g3_tag(...)` | REJECT | independence mismatch | 0 |
| R12-G3-MIGRATED-WRONG-REVIEW-TARGET | constructive_plan_audit | migrated_review_authority | a review row targets another input fingerprint | same as positive | `validate_g3_tag(...)` | REJECT | review target mismatch | 0 |
| R12-G3-MIGRATED-WRONG-REVIEW-HASH | constructive_plan_audit | migrated_review_authority | a review row hash does not match its canonical fields | same as positive | `validate_g3_tag(...)` | REJECT | review hash mismatch | 0 |
| R12-G3-MIGRATED-WRONG-BUNDLE-FINGERPRINT | constructive_plan_audit | migrated_review_authority | migrated review targets another bundle-input fingerprint | same as positive | `validate_g3_tag(...)` | REJECT | bundle fingerprint mismatch | 0 |
| R12-G3-MIGRATED-ARCH-APPROVAL-ID | constructive_plan_audit | authority_staleness | architecture approval authority ID changes without refreshing the review | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-ARCH-APPROVAL-DIGEST | constructive_plan_audit | authority_staleness | architecture approval digest changes without refreshing the review | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-ARCH-APPROVAL-TARGET | constructive_plan_audit | authority_staleness | architecture approval target changes without refreshing the review | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-F0-APPROVAL-ID | constructive_plan_audit | authority_staleness | F0 approval authority ID changes without refreshing the review | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-F0-APPROVAL-DIGEST | constructive_plan_audit | authority_staleness | F0 approval digest changes without refreshing the review | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-F0-APPROVAL-TARGET | constructive_plan_audit | authority_staleness | F0 approval target changes without refreshing the review | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-COMPONENT-SPEC-HASH | constructive_plan_audit | authority_staleness | a bound component specification hash changes | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-COMPONENT-REVIEW-ID | constructive_plan_audit | authority_staleness | a bound component review identity changes | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-LEDGER-HASH | constructive_plan_audit | authority_staleness | migration-ledger identity changes | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-NORMATIVE-HASH | constructive_plan_audit | authority_staleness | normative-matrix identity changes | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-TRACEABILITY-HASH | constructive_plan_audit | authority_staleness | generated-traceability identity changes | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-G3-MIGRATED-TARGET-COMMIT | constructive_plan_audit | authority_staleness | reviewed target commit changes | same as positive | `validate_g3_tag(...)` | REJECT | stale migrated review input | 0 |
| R12-DAG-TYPED-EDGE-CONTRACT | constructive_plan_audit | r12_artifact_authority_graph | typed source-kind/relation/destination-kind tuple changed or omitted, or exact node-edge tuple is not authorized | graph mutation | R12 artifact DAG audit | REJECT | typed or exact node-edge contract mismatch | 0 |
| R12-DAG-IDENTITY-RULE-CONTRACT | constructive_plan_audit | r12_artifact_authority_graph | identity rule type or required/optional field closure changed | graph mutation | R12 artifact DAG audit | REJECT | identity-rule contract mismatch | 0 |
| R12-DAG-SEMANTIC-AUDITS | constructive_plan_audit | r12_artifact_authority_graph | computed hash, self-Git, review-target, future-object, and bypass audits mutated | graph mutation | R12 artifact DAG audit | REJECT | semantic audit violation path | 0 |
| R12-DAG-BINDING-EQUALITY | constructive_plan_audit | r12_artifact_authority_graph | every one of the 76 exact edges has a closed explicit `none` or `serialized_binding` obligation; edge obligations are the normative root and node fields, semantic rules, serialized maps, builder/validator expectations, and prerequisites are derived mirrors; root-fixed shrink, malformed-obligation, explicit-`none`, root-identity, and G3 staleness attacks reject | graph/object mutation matrix, exhaustive 44-binding-edge root/downstream closure, obligation-schema matrix, root-change/staleness fixture matrix | R12 graph projection and G3 authority audit | REJECT | missing/malformed edge obligation, downstream mirror shrink, unauthorized `none` binding, unchanged root/fingerprint, or stale-authority acceptance | 0 |
| R12-TRACE-SEMANTIC-SUBSTITUTION | constructive_plan_audit | normative_traceability | catalog-valid but wrong F-OPS-004 relationship | matrix mutation | normative matrix equality audit | REJECT | semantic mapping mismatch | 0 |
| R12-TRACE-WRONG-KAT | constructive_plan_audit | normative_traceability | valid KAT substituted for a different requirement | matrix mutation | normative matrix equality audit | REJECT | semantic mapping mismatch | 0 |
| R12-TRACE-WRONG-EVIDENCE | constructive_plan_audit | normative_traceability | valid evidence substituted for a different requirement | matrix mutation | normative matrix equality audit | REJECT | semantic mapping mismatch | 0 |
| R12-TRACE-WRONG-AUDIT | constructive_plan_audit | normative_traceability | valid constructive audit substituted for a different requirement | matrix mutation | normative matrix equality audit | REJECT | semantic mapping mismatch | 0 |
| R12-TRACE-WRONG-CATEGORY | constructive_plan_audit | normative_traceability | valid test moved to the wrong KAT/audit/property category | matrix mutation | normative matrix category audit | REJECT | test category partition mismatch | 0 |
| R12-TRACE-CROSS-REQUIREMENT | constructive_plan_audit | normative_traceability | two requirement mappings swapped | matrix mutation | normative matrix equality audit | REJECT | semantic mapping mismatch | 0 |
| R12-TRACE-EXTRA-MAPPING | constructive_plan_audit | normative_traceability | extra relationship added to generated output | manifest mutation | normative matrix equality audit | REJECT | extra mapping | 0 |
| R12-TRACE-MISSING-MAPPING | constructive_plan_audit | normative_traceability | required relationship removed from generated output | manifest mutation | normative matrix equality audit | REJECT | missing mapping | 0 |
| R12-TRACE-SCHEMA-INVERSE | constructive_plan_audit | normative_schema_usage | schema-to-requirement inverse changed | matrix mutation | schema usage equality audit | REJECT | schema inverse mismatch | 0 |
| R12-DAG-VALID | constructive_plan_audit | r12_artifact_authority_graph | complete typed R12 graph | graph JSON | R12 artifact DAG audit | PASS | none | 0 |
| R12-DAG-UNKNOWN-NODE | constructive_plan_audit | r12_artifact_authority_graph | unknown node added | graph mutation | R12 artifact DAG audit | REJECT | unknown node | 0 |
| R12-DAG-UNKNOWN-EDGE | constructive_plan_audit | r12_artifact_authority_graph | unknown edge type added | graph mutation | R12 artifact DAG audit | REJECT | unknown edge type | 0 |
| R12-DAG-DUPLICATE-NODE | constructive_plan_audit | r12_artifact_authority_graph | duplicate node added | graph mutation | R12 artifact DAG audit | REJECT | duplicate node | 0 |
| R12-DAG-SELF-EDGE | constructive_plan_audit | r12_artifact_authority_graph | self edge added | graph mutation | R12 artifact DAG audit | REJECT | self edge | 0 |
| R12-DAG-PREREQUISITE-CYCLE | constructive_plan_audit | r12_artifact_authority_graph | prerequisite cycle added | graph mutation | R12 artifact DAG audit | REJECT | prerequisite cycle | 0 |
| R12-DAG-HASH-CYCLE | constructive_plan_audit | r12_artifact_authority_graph | hash cycle added | graph mutation | R12 artifact DAG audit | REJECT | hash cycle | 0 |
| R12-DAG-FUTURE-OBJECT | constructive_plan_audit | r12_artifact_authority_graph | dependent points to later-created object | graph mutation | R12 artifact DAG audit | REJECT | future object | 0 |
| R12-DAG-G3-BYPASS | constructive_plan_audit | r12_artifact_authority_graph | direct bypass omits mandatory G3 predecessor | graph mutation | R12 artifact DAG audit | REJECT | G3 bypass | 0 |
| R12-DAG-IMPLEMENTATION-BYPASS | constructive_plan_audit | r12_artifact_authority_graph | implementation gate disconnected from G3 | graph mutation | R12 artifact DAG audit | REJECT | implementation bypass | 0 |
| R12-DAG-REVIEW-CYCLE | constructive_plan_audit | r12_artifact_authority_graph | review target cycle added | graph mutation | R12 artifact DAG audit | REJECT | review cycle | 0 |
| R12-DAG-SELF-GIT | constructive_plan_audit | r12_artifact_authority_graph | self-Git identity edge added | graph mutation | R12 artifact DAG audit | REJECT | self-Git cycle | 0 |
| R12-DAG-ALTERNATIVE-BYPASS | constructive_plan_audit | r12_artifact_authority_graph | alternative path omits a mandatory G3 predecessor | graph mutation | R12 artifact DAG audit | REJECT | G3 bypass | 0 |
| R12-DAG-G3-BEFORE-AGGREGATE | constructive_plan_audit | r12_artifact_authority_graph | G3 construction precedes aggregate review | graph mutation | R12 artifact DAG audit | REJECT | future object | 0 |

### 3.1 Executable R12 G3 KAT

The positive fixture is a synthetic conformance object. Its input is the exact
tag name plus the exact body bytes below; the code block has six lines and the
final displayed newline is required. The body is ASCII, not JSON:

~~~text
phase_f_architecture_plan_tag=ism-mechanism-health-v1-f-plan-approved
phase_f_f0_decisions_tag=ism-mechanism-health-v1-f-f0-decisions-approved
specification_bundle_manifest_sha256=0000000000000000000000000000000000000000000000000000000000000000
aggregate_review_bundle_sha256=1111111111111111111111111111111111111111111111111111111111111111
approval_decision=GO
schema_version=1
~~~

`fixture_id=R12-POS-SPEC-BUNDLE-TAG`

`fixture_byte_length=379`

`fixture_sha256=af3f94a1a5ae85f2e62d8a0ad54e66b3bd985cd150805a5750528befa15027b6`

`operation=validate_g3_tag(tag_name,body_bytes,synthetic_context)`

The exact synthetic semantic object is:

~~~json
{
  "tag_name": "ism-mechanism-health-v1-f-specification-bundle-approved",
  "phase_f_architecture_plan_tag": "ism-mechanism-health-v1-f-plan-approved",
  "phase_f_f0_decisions_tag": "ism-mechanism-health-v1-f-f0-decisions-approved",
  "specification_bundle_manifest_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
  "aggregate_review_bundle_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
  "approval_decision": "GO",
  "schema_version": "1"
}
~~~

The exact expected result is `PASS`, with the six decoded body fields equal to
the lines above. The fixture tests canonical serialization, not authority: the
synthetic hashes, absent real tags, absent reviews, and absence of any registry
record are intentional. The canonical byte vector is also represented by this
hex string, with no omitted or implicit bytes:

~~~text
70686173655f665f6172636869746563747572655f706c616e5f7461673d69736d2d6d656368616e69736d2d6865616c74682d76312d662d706c616e2d617070726f7665640a70686173655f665f66305f6465636973696f6e735f7461673d69736d2d6d656368616e69736d2d6865616c74682d76312d662d66302d6465636973696f6e732d617070726f7665640a73706563696669636174696f6e5f62756e646c655f6d616e69666573745f7368613235363d303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030300a6167677265676174655f7265766965775f62756e646c655f7368613235363d313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131310a617070726f76616c5f6465636973696f6e3d474f0a736368656d615f76657273696f6e3d310a
~~~

Each negative mutation is applied to the positive input by the exact operation
shown. The expected checker result for every row is `REJECT`; categories are
stable diagnostic classes, not approval decisions.

| mutation_id | exact deterministic mutation operation | expected result | expected failure category |
|---|---|---|---|
| R12-NEG-G3-WRONG-FIELD-NAME | Replace the first key `phase_f_architecture_plan_tag` with `phase_f_architecture_plan` | REJECT | unknown_field |
| R12-NEG-G3-LEGACY-FIELD-NAME | Replace the first key with the legacy unprefixed `architecture_plan_tag` | REJECT | legacy_field_name |
| R12-NEG-G3-MISSING-REQUIRED-FIELD | Remove the complete `aggregate_review_bundle_sha256=...` line and its LF | REJECT | missing_required_field |
| R12-NEG-G3-DUPLICATE-FIELD | Insert a second complete `approval_decision=GO` line immediately before `schema_version=1` | REJECT | duplicate_field |
| R12-NEG-G3-UNEXPECTED-FIELD | Replace the final `schema_version=1` line with `unexpected_field=x` | REJECT | unexpected_field |
| R12-NEG-G3-WRONG-LINE-ORDER | Swap the first and second complete lines | REJECT | wrong_field_order |
| R12-NEG-G3-SCHEMA-VERSION | Replace `schema_version=1` with `schema_version=2` | REJECT | invalid_schema_version |
| R12-NEG-G3-MALFORMED-TAG-NAME | Replace the input tag name with `ism-mechanism-health-v1-f-specification-bundl-approved`; leave body unchanged | REJECT | invalid_tag_name |
| R12-NEG-G3-WRONG-ARCHITECTURE-BINDING | Replace the first value with `ism-mechanism-health-v1-f-f0-decisions-approved` | REJECT | wrong_architecture_plan_binding |
| R12-NEG-G3-WRONG-F0-BINDING | Replace the second value with `ism-mechanism-health-v1-f-plan-approved` | REJECT | wrong_f0_decisions_binding |
| R12-NEG-G3-WRONG-BUNDLE-HASH | Replace the first zero in `specification_bundle_manifest_sha256` with `a` | REJECT | wrong_bundle_hash |
| R12-NEG-G3-MALFORMED-SHA | Replace the first zero in `aggregate_review_bundle_sha256` with `z` | REJECT | malformed_sha256 |
| R12-NEG-G3-TRAILING-WHITESPACE | Replace `approval_decision=GO` with `approval_decision=GO ` | REJECT | trailing_whitespace |
| R12-NEG-G3-MISSING-DELIMITER | Replace the first `=` delimiter with one ASCII space | REJECT | missing_delimiter |
| R12-NEG-G3-INVALID-NEWLINE | Replace the LF after the first line with CRLF | REJECT | invalid_newline |
| R12-NEG-G3-EXTRA-TRAILING-CONTENT | Append `trailing` plus LF after the required final LF | REJECT | extra_trailing_content |
| R12-NEG-G3-TRUNCATED-CONTENT | Remove the final ten bytes, producing a partial final field | REJECT | truncated_content |
| R12-NEG-G3-MISSING-FINAL-NEWLINE | Remove exactly the required final LF byte | REJECT | missing_final_newline |
| R12-NEG-G3-WRONG-APPROVAL-VALUE | Replace `approval_decision=GO` with `approval_decision=NO-GO` | REJECT | invalid_approval_decision |

The checker rejects CR, blank lines, alternate order, duplicate keys, omitted
keys, additional keys, legacy keys, malformed values, and bytes after the final
required LF before any authority prerequisite is considered. Real Git tag
type, peel, reachability, manifest, review, and upstream-authority checks are
part of the same reusable validator and are exercised by the prerequisite
catalog below; the synthetic KAT supplies synthetic equivalents for those
predicates and never creates real authority.

### 3.2 Canonical G3 authority validator

The normative operation is the single reusable function
`validate_g3_tag(tag_name, body_bytes, context)`. `parse_g3_tag(...)` is only
the wire-validation layer called by that function; `check_g3_kat(...)` is a
reporting adapter that supplies a synthetic context and catches the same
typed validation errors. Real repository validation supplies a real context.
The validator obtains its mandatory prerequisite list from
`phase_f_r12_authority_graph.json`; it does not maintain a second hard-coded
G3 prerequisite list.

The context has three closed modes. `synthetic` resolves complete synthetic
objects through the same object, target, lifecycle, digest, graph, and
staleness interfaces as `real`, but its prevalidated digest records are
explicitly test-only. `real` resolves exact repository bytes and Git objects;
`real_test` is an explicit isolated-fixture mode that permits records marked
`authority_class=TEST_ONLY`. A synthetic or `real_test` context marked for
real-authority authorization rejects before any approval result is returned.

Validation order is fail-closed: exact six-field tag grammar; real annotated
tag object type, exact peeled commit, and exact message bytes; parsed graph
closure; exact manifest bytes/hash/target/status; architecture and F0 approval
objects; every direct review bundle already resolved in the validation context
using the inherited seven-field R11 wire; the complete immutable migrated-
finding review; and the aggregate bundle whose graph-derived scope is the
exact final manifest. The graph intentionally places `readiness_review` at its
later readiness stage outside the 13-node G3 prerequisite closure; when that
node is resolved, it is validated by the same direct-bundle validator and is
covered by the isolated real-format readiness fixture. Every object must have
the expected kind, identity, bound target, ACTIVE lifecycle, no
stale/superseded/invalidated state, and exact dependency closure. The migrated
review must cover all five migrated findings with one disposition each and
bind the exact bundle-input fingerprint, migration ledger, traceability
manifest, and component content identities. Any absent, malformed, stale,
superseded, mismatched, future, or contradictory object returns `REJECT`.

The migrated-review decision is derived, never trusted from serialized input:

```text
finding closure
AND exact five-role completeness
AND unique/resolvable reviewer identities
AND pairwise-distinct reviewer actor identity digests
AND unique/resolvable review-artifact identities
AND reviewer role/target/decision/lifecycle bindings
AND reviewer independence from the explicit remediation-author identity
```

must hold before `GO` is possible. The only five-row decision vector that can
produce `GO` is `(GO, GO, GO, GO, GO)`; any vector containing `NO-GO` produces
`NO-GO`. The serialized decision must equal that derived result.

The implementation's self-test exercises a complete synthetic PASS, the exact
direct R11 bundle positive path, the graph-derived Git/external-object target
matrix for all nine direct review nodes (including the readiness review outside
the G3 closure), ten pairwise actor-identity mutations for every exercised
direct bundle, the direct-bundle negative matrix, every authority-prerequisite
negative in the catalog, real checked-in repository NO-GO with absent
approvals, review-start Git-anchor equality/mismatch, and
synthetic-to-real isolation. This is a conformance path only; it does not
create an approval tag or change the candidate bundle's fail-closed state.

The self-test also constructs a disposable isolated Git repository containing
the real-format source files, nine canonical direct R11 review bundles, five
canonical reviewer identities reused across those bundles, their immutable
review artifacts, five migrated-review artifacts, one explicit
remediation-author identity, annotated architecture/F0/G3 tags, and a target
commit. The positive path resolves that repository through
`make_repository_context(..., allow_test_only=True)` and passes the same
`validate_g3_tag` function used by the real path. Its negative matrix mutates
missing objects, malformed/noncanonical objects, wrong digests, wrong targets,
stale/superseded records, incomplete migrated dispositions, all five row
decisions, every missing role, every pairwise duplicate reviewer/artifact
identity, every unresolved reviewer/artifact reference, direct-bundle role and
artifact mismatches, TEST_ONLY-in-real-mode classification, author
substitution, serialized bindings, and annotated-tag type/peel/message fields;
every mutation must reject. The review-start test independently requires the
reviewed target to equal `HEAD`, local `main`, `origin/main`, and a supplied
live remote-main SHA. A separate local bare-remote fixture proves exact-SHA
lease publication, clean-worktree and fast-forward preconditions, remote-race
rejection, non-fast-forward rejection, and unavailable-live-state rejection.
The disposable fixtures are never copied into the checked-in candidate and
are removed after the self-test.

The R12 authority graph is a closed exact contract, not only an edge-membership
check. `edge_contract` is the normative finite set of complete node-level
`from|relation|to` tuples, and serialized `edges` must equal that set exactly;
the typed source-kind/relation/destination-kind contract remains a general
admissibility check. The self-test enumerates all 28-node, seven-relation,
non-self candidate triples and requires zero accepted undeclared edges. It also
proves every authorized edge by removal, retyping, and destination redirection
mutations. Every exact edge has one explicit binding obligation; the 44
serialized obligations drive the derived node, semantic, serialized-map,
builder, validator, and G3 projections, while 32 edges explicitly declare
`none`. Computed audits emit named pass records and violation paths for hash
cycles, self-Git cycles, review-target cycles, future-object dependencies,
self-reference, G3 bypass, and implementation-readiness bypass. Mutations of
any contract or audit input reject before an authority result can be returned.

### 3.3 Normative semantic traceability and schema usage

`phase_f_r12_normative_traceability_matrix.json` is the single parseable
authority for the 64 R12 requirement-to-validation relationships. Every row
contains the exact requirement/anchor, dependencies, test IDs, KAT IDs,
constructive-audit IDs, property-test IDs, future-real-evidence IDs, expected
stage, validation category, and schema IDs. Empty arrays are explicit
intentional absence. The generator derives the traceability manifest from
these rows and verifies exact bidirectional set equality; it never chooses a
mapping from a requirement-number switch. It also derives schema-to-
requirement and requirement-to-schema projections from the same `schema_ids`
cells and verifies both directions against the 93-schema catalog.

Catalog-valid but semantically wrong substitutions, including replacing the
`F-OPS-004` mapping with `R11-CAT` and `EV11-01`, are rejected because the
generated relationship no longer equals the normative row. Valid wrong KAT,
evidence, audit, cross-requirement, extra, missing, category, and schema
inverse mutations are covered by the R12 constructive audit IDs.

## 4. Review gate

P0/P1 must both be zero. Wrong SHA, undeclared URI, incomplete PASS, missing
mutation, schema/test contradiction, opaque-to-schema promotion, test-only F0
leakage, missing usage row, or evidence promotion is P1 and blocks G3.
