# 09 — Testing & Quality Assurance

## Reduced-order ISM components V1

Parity, analytical decay, decomposition, variance separation, and neutral
equilibrium-recognition coverage are added in `reduced_ism_components_v1` tests.

## Model-core QA

Model-core tests cover graph rejection and stable ordering, state/parameter
round trips and slices, additive reconstruction, variance separation,
Jacobian coverage, covariance propagation, schema serialization, and an
architecture regression check that prohibits high-level imports from
`src/model`.

**Identifier:** `DOC-09`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## 1. Test Inventory

### Unit Tests (In-File)

| Source File | Test Count (approx.) | Coverage Focus |
|-------------|---------------------|----------------|
| `src/cli.rs` | 5 | CLI parsing, legacy normalization, error cases |
| `src/domain/experiment.rs` | 1 | Event ordering |
| `src/domain/measurement.rs` | 3 | Validation, diagnostics, irregular sampling |
| `src/domain/provenance.rs` | 1 | SHA-256 hashing |
| `src/impedance/lib.rs` | 10 | EIS fitting, preprocessing, element limits |
| `src/impedance/circuits.rs` | (inferred) | Parser, evaluation |
| `src/potentiometry/units.rs` | 2 | Unit conversion, molar mass requirements |
| `src/potentiometry/calibration/nernst.rs` | 2 | Nernst slope, charge sign |
| `src/regression_mod.rs` | 6 | Linear fit, error cases, curve generation |
| `src/plot_config.rs` | 20 | Schema, migration, resolution, validation |
| `src/results/mod.rs` | 1 | CircuitFitResult structure |
| `src/model/` | (covered by integration fixture) | Core compilation, validation, and decomposition contracts |

### Integration Tests (`tests/`)

| Test File | Phase | Coverage |
|-----------|-------|----------|
| `tests/phase0_regression.rs` | 0 | End-to-end binary: plot, search, fit CLI |
| `tests/phase1_domain.rs` | 1 | Measurement parsing, metadata, experiment construction |
| `tests/phase2_transient.rs` | 2 | Transient model fitting, model selection, CLI integration |
| `tests/phase3_calibration.rs` | 3 | Calibration model fitting, prediction, validation |
| `tests/phase3_workflow.rs` | 3 | Full calibration workflow |
| `tests/phase4_mechanism.rs` | 4 | Mechanism comparison |
| `tests/phase5_signal_health.rs` | 5 | Signal analysis, health assessment |
| `tests/phase6_estimation.rs` | 6 | State estimation, EKF/UKF |
| `tests/unified_data_loading.rs` | 3 | File format detection, binary rejection |
| `tests/xlsx_ingestion.rs` | (inferred) | Excel file reading |
| `tests/canonical_input_boundary.rs` | IO migration | canonical ownership guard, compatibility API compilation, recovery/time/EIS semantics |
| `tests/canonical_error_types.rs` | IO migration | provider typed errors and worksheet context |
| `tests/model_core.rs` | 02 | ISM graph compilation, schemas, decomposition, and legacy CLI surface |
| `tests/model_builtins.rs` | 03 | Synthetic equilibrium, relaxation, disturbance, reconstruction, and validity coverage |

### Test Classification

| Classification | Count |
|---------------|-------|
| Unit tests (inline `#[cfg(test)]`) | ~50+ |
| Integration tests (`tests/`) | ~10 files |
| CLI tests | 5 (cli.rs) + binary tests |
| Numerical tests | Multiple (fit accuracy, Nernst slope) |
| Error-path tests | Multiple (invalid inputs, edge cases) |
| Cross-platform tests | CI runs on Linux + macOS |

The completed independent legacy parity gate is preserved in
`docs/io_migration_validation_archive.md`; ongoing coverage uses the
`electrodata-io` canonical input boundary, typed errors, XLSX, EIS semantics,
and all-phase scientific regression tests rather than a retained local raw
parser.

## 2. Requirement Test Coverage

Evidence classes:
- **Direct**: explicit assertions in tests for requirement behavior.
- **Partial**: representative workflow behavior covered, but not every requirement facet asserted directly.
- **Inferred**: requirement confidence is primarily from implementation inspection.

| Requirement | Evidence Class | Evidence Summary |
|------------|----------------|------------------|
| FR-001 to FR-011 | Direct | Parsing, fit, model resolution, and ECM search behaviors are asserted in `chi_file`, `unified_data_loading`, `phase0_regression`, and module tests. |
| FR-012 to FR-014 | Partial | Plot workflows are integration-tested for dispatch and artifact creation; visual/semantic plot correctness is not directly asserted. |
| FR-015 | Direct | `phase2_transient` validates event filtering, model fitting, and CLI behavior. |
| FR-016 to FR-019 | Direct | `phase3_calibration` and workflow tests assert extraction, fitting, validation, and prediction paths. |
| FR-020 | Direct | `phase5_signal_health` asserts core signal feature extraction and residual/statistical pathways. |
| FR-021 to FR-022 | Partial | Health baseline/assessment paths are exercised, but complete rule-surface and cross-artifact combinations remain partially inferred. |
| FR-023 | Inferred | Mechanism implementation is verified from source with limited direct test assertions for all comparison/trend branches. |
| FR-024 | Direct | `phase6_estimation` asserts EKF/UKF runtime behavior and artifact outputs. |
| FR-025 to FR-026 | Direct | Provenance hashing and EIS text reporting are directly asserted in unit/integration tests. |
| FR-027 to FR-028 | Partial | Representative JSON/CSV exports are tested; full workflow-by-workflow artifact-name coverage is implementation-verified. |
| FR-029 to FR-033 | Direct | `model_core` covers model compilation, graph/unit failures, bounds, deterministic indices, decomposition, versioned schemas, and CLI compatibility. |
| SCI-001 to SCI-010 | Partial | Core scientific constants/formulas are directly tested; several equations remain code-verified rather than assertion-complete. |
| NUM-001 to NUM-006 | Partial | Numerical safeguards and diagnostics are tested in representative paths; full edge-space coverage is incomplete. |
| DAT-001 to DAT-007 | Partial | Data-model invariants are directly tested; cross-module schema compatibility and migration behavior remain partial. |

## 3. Test-Gap Summary

| Gap | Description | Priority |
|-----|-------------|----------|
| GAP-001 | No performance regression tests | Low |
| GAP-002 | No cross-version schema compatibility tests | Medium |
| GAP-003 | Many workflow tests validate artifact presence/shape but not full file-content semantics for every exported artifact variant | Medium |
| GAP-004 | No fuzz testing of circuit parser | Low |
| GAP-005 | No tests for NaN/Inf edge cases in all numerical paths | Medium |
| GAP-006 | Kalman filter process/measurement noise model tests are limited | Medium |
| GAP-007 | No Windows CI testing | Low |
| GAP-008 | No integration test for `estimate simulate` | Low |
| GAP-009 | No deterministic reproducibility tests (fixed seed comparison) | Medium |
| GAP-010 | Plotting output is tested for file existence, not visual correctness | Low |
| GAP-011 | No real Nernst, transport, transduction, reference, or external ISM component implementation yet | High (planned Phase 03) |
| GAP-012 | Identifiability and equilibrium contracts intentionally report not-assessed rather than numerical conclusions | Medium (planned later validation phase) |
| GAP-013 | Built-ins are reduced-order adapters; no high-fidelity Nernst-Planck transport or mechanism confirmation is implemented | High (deferred) |

## 4. Run Commands

| Action | Command |
|--------|---------|
| Format check | `cargo fmt --check` |
| Lint (strict) | `cargo clippy --all-targets --all-features -- -D warnings` |
| All tests | `cargo test --all` |
| Specific test | `cargo test <test_name>` |
| Release build | `cargo build --release` |
| CI simulation | `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all && cargo build --release` |

## 5. CI Configuration (`.github/workflows/ci.yml`)

- **Triggers**: push, pull_request
- **Matrix**: ubuntu-latest, macos-latest
- **Toolchain**: pinned by `rust-toolchain.toml`, including rustfmt and clippy
- **Locked steps**: cargo clippy, test, and release build use `--locked`; format is checked before compilation

## Phase 05 Regression Coverage

`tests/phase05_model_health.rs` covers explicit EIS component-ID mapping,
missing mappings, unreplicated timescale evidence, contradictory evidence,
model residual health features, and multi-domain mechanistic health guards.

`phase06_model_workflow` covers model CLI parsing, invalid configuration,
deterministic simulation, decomposition exports, JSON compatibility, finite
JSON output, report regeneration, and estimate-command parsing compatibility.

`phase07_validation` verifies manifest evaluation, contribution reconstruction,
reproducible result contracts, and the synthetic-data non-validation boundary.

`artifact_contract` verifies typed kind/version rejection, legacy kind-less
migration, write-time headers, and semantic contracts for every exported
cross-workflow result. `model_builtins` verifies malformed-descriptor rejection,
runtime ion charge, exact relaxation subdivision invariance, and contribution
reconstruction. `phase6_estimation` verifies that the compiled compatibility
adapter reproduces legacy equations and that EKF/UKF expose identical named
scientific contributions.

`estimation::model_output::equilibrium_tests` covers a fully evidenced stable
timestamp, rejection of slow reference drift, and indeterminate classification
when history or residual evidence is missing.

### ADR-0002 contract coverage

`tests/model_contracts.rs` guards stable component IDs, duplicate state and
parameter rejection, required state/parameter/component metadata, the separation
of noise/residuals from deterministic voltage, and forbidden high-level imports
from `src/model`. Existing `model_core` tests cover acyclic dependencies and
unique contribution ownership. These tests are architectural invariants, not
evidence that a high-fidelity physical transport model has been validated.

Model-contract tests cover deterministic contribution ordering, typed voltage
reconstruction, observation-variance exclusion, canonical external-role alias
migration, uncertainty validation, covariance propagation, and non-finite
prediction serialization rejection.

Schema-v3 regressions additionally cover Nernst E0 and slope covariance and
their cross term, Nicolsky-Eisenman E0/slope/selectivity derivatives and full
covariance, transduction gain/offset, centered covariate coefficients, drift,
stable-ID mapping, missing derivative/covariance status, explicit disabling,
strict fitted/estimated compatibility matrices, legacy numeric-zero migration,
and enrichment of legacy estimator states from resolved initial covariance.

Covariance-consistency regressions cover the reviewer Nernst positive-charge
uncertainty plus zero-row reproduction, a generic missing derivative with a
zero row, missing fitted-parameter and estimated-state runtime covariance,
deterministic and stochastic `1e-13` state/parameter covariance, covered
numerical zero derivatives, state equivalents, and dimension, non-finite,
asymmetric, and non-PSD matrices. The existing Nernst 0.76 V² cross-covariance
calculation remains covered.
