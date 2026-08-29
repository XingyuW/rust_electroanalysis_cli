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
| <a id="F-CNF-004"></a>`F-CNF-004` | `F-ARCH-008..010,F-WIRE-007` | Literal parser KATs cover every approval tag including the new specification-bundle tag; separate real-Git properties verify annotated type, peel target, reachability, and upstream tag resolution. | §§53.5, 53.9–53.10 plus R12 tag grammar |
| <a id="F-CNF-005"></a>`F-CNF-005` | `F-ARCH-017,F-ARCH-022,F-WIRE-008` | Constructive audits verify the explicit authority DAG, no future/self/reference cycles, schema/anchor/catalog equality, exhaustive usage matrix, traceability inverse, owner-decision union, Markdown structure, and no duplicate current traceability graph. | §§53.6–53.13 |
| <a id="F-CNF-006"></a>`F-CNF-006` | `F-ARCH-017` | Every literal KAT declares all required inputs, exact fixture bytes, semantic IDs, SHA-256, byte length, URI when applicable, operation, expected result, and at least one exact negative mutation. Missing data or an incomplete PASS is P1. | §§53.9–53.10 |
| <a id="F-CNF-007"></a>`F-CNF-007` | `F-ARCH-002,F-ARCH-012,F-ARCH-017,F-SCI-009..010` | Automated isolation checks prove zero KAT/test/synthetic/constructed-to-real-evidence promotion paths and preserve every future-real-evidence oracle. | §§53.8 R11-14/R11-19, 53.11 |
| <a id="F-CNF-008"></a>`F-CNF-008` | `F-ARCH-003,F-ARCH-017,F-IMPL-006` | Regression checks preserve frozen Phase-E behavior, the P2 gate, production runner order, and all previously closed safety contracts. | §§19–20, 53.8 R11-17 |

## 3. Current executable catalog

R11 §53.10 is adopted unchanged as the current 28-row test catalog: eight
literal executable KATs, four property tests, and sixteen constructive audits.
Exact fixture bytes, semantic IDs, SHA values, lengths, test-only URIs,
expected results, and mutations in §§44–46 and §§53.2–53.6 remain test
authority here. Historical §§16–52 are provenance only except where a current
§53 row explicitly adopts a literal value.

Add `R12-POS-SPEC-BUNDLE-TAG` and negative mutations for every ordered field,
hash, tag resolution, approval value, target, and extra/missing line. It cannot
claim real approval or satisfy any F-EV.

## 4. Review gate

P0/P1 must both be zero. Wrong SHA, undeclared URI, incomplete PASS, missing
mutation, schema/test contradiction, opaque-to-schema promotion, test-only F0
leakage, missing usage row, or evidence promotion is P1 and blocks G3.
