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

The current catalog is the 28 adopted R11 rows plus the one R12 row below. The
generator parses the adopted R11 table from the preserved source and this R12
row from this document, then rejects every reference outside that closed
catalog. `R12-POS-SPEC-BUNDLE-TAG` is a literal KAT and its fixture is never a
real approval or an `F-EV`.

| test_id | kat_class | fixture_scope | prevalidated_inputs | literal_inputs | operation | expected_result | negative_mutation | UNBOUND_REQUIRED_INPUT_COUNT |
|---|---|---|---|---|---|---|---|---:|
| R12-POS-SPEC-BUNDLE-TAG | literal_kat | g3_specification_bundle_tag | none | exact tag name, exact 379-byte body, body SHA-256, decoded six fields | `validate_g3_tag(tag_name, body_bytes, synthetic_context)` | PASS with exact decoded fields | the individually defined `R12-NEG-G3-*` rows below | 0 |

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
separate predicates and are not fabricated by this synthetic KAT.

## 4. Review gate

P0/P1 must both be zero. Wrong SHA, undeclared URI, incomplete PASS, missing
mutation, schema/test contradiction, opaque-to-schema promotion, test-only F0
leakage, missing usage row, or evidence promotion is P1 and blocks G3.
