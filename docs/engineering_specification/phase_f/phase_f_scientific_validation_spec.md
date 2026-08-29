# Phase F Scientific Validation Specification

## 1. Authority and adoption

This G2 candidate owns the scientific meaning of Phase F. It refines
`F-ARCH-002`, `F-ARCH-012..014`, `F-ARCH-017..018`, and applicable F0
decisions. Wire representation is controlled by the Wire specification.

## 2. Requirements

| ID | derived_from | Normative requirement | Adopted R11 clauses |
|---|---|---|---|
| <a id="F-SCI-001"></a>`F-SCI-001` | `F-ARCH-002,F-ARCH-012,F-OD-08` | Admissible categories and claim ceilings are exact. Direct/orthogonal physical evidence may support only within its registered endpoint/domain; validated proxies remain limited; model/same-signal evidence cannot support physical claims; expert interpretation cannot support alone; unavailable supplies no support. | §§4 F-OD-08, 19, 53.8 R11-14/R11-19 |
| <a id="F-SCI-002"></a>`F-SCI-002` | `F-ARCH-013,F-ARCH-014,F-OD-10` | Independence is counted by registered physical unit. Repeats, rows, measurements, runs, aliquots, parents/children, shared samples/sensors/preprocessing/models/references, and undeclared dependencies cannot increment an independent count. Unknown identity or dependency is NO-GO. | §§11, 19, 20 |
| <a id="F-SCI-003"></a>`F-SCI-003` | `F-ARCH-014,F-OD-05,F-OD-06,F-OD-07` | Mechanism/health endpoints, acceptance logic, domains, contradiction rules, and requested claims are fixed before outcome access. A claim cannot exceed the weakest supporting endpoint/category and any qualifying contradiction blocks it. | §§4, 13, 19 |
| <a id="F-SCI-004"></a>`F-SCI-004` | `F-ARCH-014,F-OD-09` | Cohort partitioning occurs at the independent split unit, uses registered stratification/randomization/seed/execution authority, locks before outcome access, and forbids post-hoc movement. Development cannot satisfy validation/holdout evidence. | §§4, 14 |
| <a id="F-SCI-005"></a>`F-SCI-005` | `F-ARCH-014,F-OD-12` | Power uses the registered exact method/version, typed/ranged/unit-qualified parameters and outputs, declared sensitivity cases, independent-unit counts, and pre-data review. Underpowered or indeterminate analysis is NO-GO. | §12, §53.7 power anchors |
| <a id="F-SCI-006"></a>`F-SCI-006` | `F-ARCH-014,F-OD-11` | Reference methods have registered method/version and authority, required blinding, quantified uncertainty, dependency completeness, and endpoint qualification. Same-source or same-signal reference routes cannot establish orthogonal confirmation. | §13 |
| <a id="F-SCI-007"></a>`F-SCI-007` | `F-ARCH-014,F-OD-11` | Metrology closes calibration, QC, traceability, uncertainty, units, LOD/LOQ, validity intervals, and endpoint-qualified acceptance before evidence use. Missing, expired, failed, out-of-range, below-LOD, or ambiguity at LOQ is fail-closed. | §13 |
| <a id="F-SCI-008"></a>`F-SCI-008` | `F-ARCH-012,F-ARCH-014,F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-05,F-OD-06,F-OD-07,F-OD-08,F-OD-09,F-OD-10,F-OD-11,F-OD-12` | Protocol, power, package/dependency audit, unit/identity/location/custody ledgers, metrology/reference results, cohort lock, and scientific admissibility audit must all predate owner approval and production execution. | §§10–14, 17–19 |
| <a id="F-SCI-009"></a>`F-SCI-009` | `F-ARCH-002,F-ARCH-012,F-ARCH-017` | No test/KAT/storage fixture, synthetic/constructed/model material, or earlier experiment is retrospectively promoted. A future revision may define retrospective admission only with explicit equivalence, provenance, independence, custody, metrology, preregistration-bias analysis, and fresh independent GO review. Current default is forbidden. | §53.8 R11-14/R11-19, §53.11 |
| <a id="F-SCI-010"></a>`F-SCI-010` | `F-ARCH-017,F-ARCH-022` | Every physical/scientific requirement has a future-real-evidence oracle distinct from conformance tests. The oracles remain the applicable R11 §53.11 rows, reassigned in the traceability manifest. | §53.11 |

## 3. Review gate

P0/P1 must both be zero. Unclear independent unit, endpoint, power method,
reference authority, metrology rule, pseudoreplication path, retrospective
promotion, contradiction handling, or claim ceiling is P1 and blocks G3.
