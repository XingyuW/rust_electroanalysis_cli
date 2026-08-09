# Phase A0 artifact-contract traceability

This document records implementation evidence for MHI-R2 without modifying the
approved MHI V1 planning contract.

| Requirement | Acceptance criterion | Permanent fixture evidence | Exact test | Result |
|---|---|---|---|---|
| MHI-R2 | AC2 / current correct kind | `tests/fixtures/a0_artifact_contracts/schema2/*.schema2.json` (all 8 repair kinds) | `tests/a0_producer_roundtrip.rs::mhi_t02a_current_correct_kind` | pass |
| MHI-R2 | current wrong kind | Test-only typed matrix inputs for all 8 current repair contracts | `tests/artifact_contract.rs::mhi_t02b_current_wrong_kind` | pass |
| MHI-R2 | current missing kind | Test-only typed matrix inputs for all 8 current repair contracts | `tests/artifact_contract.rs::mhi_t02c_current_missing_kind` | pass |
| MHI-R2 | legacy schema-1 compatibility | `tests/fixtures/a0_artifact_contracts/schema1/*.schema1.json` (all 8 repair kinds) | `tests/a0_producer_roundtrip.rs::mhi_t02d_legacy` | pass |
| MHI-R2 | unsupported schema | Test-only future-schema inputs for all 8 current repair contracts | `tests/artifact_contract.rs::mhi_t02e_unsupported` | pass |
| MHI-R2 | producer round trips | Temporary output directories only; no fixture-tree writes | `tests/a0_producer_roundtrip.rs::mhi_t02f_producer_roundtrip` | pass |
| MHI-R2 | A0-AC-COMPAT-01 | `tests/fixtures/artifact_contracts/{eis_fit,health_baseline}_schema2_{missing,correct,wrong}_kind.json` | `tests/artifact_contract.rs::a0_ac_compat_01_preserves_eis_fit_and_health_baseline_matrices` | pass |

The eight repair contracts are current schema 2, legacy schema `[1]`, and
`CurrentArtifactKindPolicy::Required`. MHI-T02a loads each tracked schema-2
JSON through the public `read_artifact` reader, deserializes to its intended
typed artifact, and asserts representative scientific payload fields.

MHI-T02d loads each tracked historical schema-1 JSON through the same public
reader, deserializes to its intended typed artifact, and asserts representative
payload fields. The fixture manifest at
`tests/fixtures/a0_artifact_contracts/README.md` records the historical source
commit and result type for each fixture.

`eis_fit` and `health_baseline` remain current schema 2 with legacy `[1, 2]`
and `PreserveLegacyOptional`, so missing current kind remains readable while a
present wrong kind remains rejected. Their compatibility matrix uses only
tracked inputs and performs no runtime writes under `tests/fixtures/`.

The nine producer paths covered by MHI-T02f are transient analysis; calibration
observations, analysis, and stored model; signal analysis; mechanism compare and
trend; health assessment; and health trend. Each path is exercised through its
producer/writer and reread with the typed artifact reader. No A1
lineage/evidence/hypothesis/health-integration types or CLI flags are part of
A0.

Finding reconciliation for this remediation:

| Finding | Classification | Evidence and root cause | Resolution |
|---|---|---|---|
| A0-P1-001 | CONFIRMED | Before remediation, MHI-T02d read only `health_baseline`; the nine-producer helper synthesized schema-1 JSON; MHI-T02a checked constants without reading current JSON. | Added tracked schema-1 and schema-2 fixtures for all 8 repair kinds; MHI-T02a and MHI-T02d now exercise the public reader and assert payload fields; removed dynamic schema-1 generation from MHI-T02f. |
| A0-P2-001 | CONFIRMED | `a0_ac_compat_01_preserves_eis_fit_and_health_baseline_matrices` wrote `.correct.json` and `.wrong.json` beside tracked fixtures. | Added tracked correct/wrong fixtures and changed the test to read missing/correct/wrong inputs without runtime writes. |
