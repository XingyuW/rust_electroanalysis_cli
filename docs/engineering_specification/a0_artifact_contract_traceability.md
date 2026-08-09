# Phase A0 artifact-contract traceability

This document records the implementation evidence for MHI-R2 without
modifying the approved MHI V1 planning contract.

| Requirement | Acceptance criterion | Implementation | Test | Result |
|---|---|---|---|---|
| MHI-R2 | AC2 | `src/results/artifact_contracts.rs`; `src/domain/artifact.rs::validate_value`; affected producer/result modules | `mhi_t02a_current_correct_kind`, `mhi_t02b_current_wrong_kind`, `mhi_t02c_current_missing_kind`, `mhi_t02d_legacy`, `mhi_t02e_unsupported`, `mhi_t02f_producer_roundtrip` | pass |
| MHI-R2 | A0-AC-COMPAT-01 | `CurrentArtifactKindPolicy::PreserveLegacyOptional` for non-A0 contracts | `a0_ac_compat_01_preserves_eis_fit_and_health_baseline_matrices`; tracked fixture inputs | pass |

The eight repair contracts are current schema 2, legacy schema `[1]`, and
`CurrentArtifactKindPolicy::Required`. Legacy schema-1 inputs retain their
existing migration behavior. `eis_fit` and `health_baseline` remain current
schema 2 with legacy `[1, 2]` and `PreserveLegacyOptional`, so missing current
kind remains readable while present wrong kind remains rejected.

The nine producer paths covered by `mhi_t02f_producer_roundtrip` are transient
analysis; calibration observations, analysis, and stored model; signal
analysis; mechanism compare and trend; health assessment; and health trend.
Each path is exercised through its producer/writer and reread with the typed
artifact reader. No A1 lineage/evidence/hypothesis/health-integration types or
CLI flags are part of A0.
