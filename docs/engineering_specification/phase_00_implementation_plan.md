# Phase 00 — Platform Hardening Implementation Plan

## Scope

This phase hardens existing workflow boundaries before the unified ISM model is introduced. It deliberately does not add `src/model` or alter scientific equations.

| File area | Change |
|---|---|
| `src/domain/artifact.rs`, `src/domain/{mod,errors}.rs` | Add the stable `VersionedArtifact` contract, artifact kinds, finite JSON validation, legacy-header migration handling, and typed errors. |
| `src/results/*.rs` | Implement the contract for cross-workflow artifacts while preserving their existing payload schemas. |
| `src/runners/{fit,transient,calibration,signal,health,mechanism,estimation}.rs` | Replace direct JSON deserialization/serialization at cross-workflow boundaries with the validated adapter. |
| `src/health/{features,baseline,assessment,rules}.rs` | Preserve transient event context; classify drift correctly; enforce baseline minimums; add trend operators and contradictory evidence; derive statuses from evidence and deviations. |
| User-data paths under `src/` | Replace reachable `unwrap`/`expect`/`unreachable!` sites with typed fallible paths or guarded alternatives. |
| `rust-toolchain.toml`, `.github/workflows/ci.yml` | Pin the toolchain and use `Cargo.lock` via `--locked`. |
| `tests/artifact_contract.rs`, `tests/phase5_signal_health.rs` | Add migration, artifact semantic, and health-rule regression coverage. |
| `README.md`, DOC-01, DOC-03, DOC-08, DOC-09, DOC-13, DOC-14 | Document contract, migration policy, validation, traceability, and resolved risks. |

## Compatibility policy

New artifacts contain `artifact_kind` at the JSON root. The existing `schema_version` payload field remains authoritative and legacy artifacts with a supported prior schema but no kind are accepted only through the typed expected-artifact loader. Artifacts with a present but incompatible kind, unsupported schema, malformed root, or non-finite numeric value are rejected before deserialization.
