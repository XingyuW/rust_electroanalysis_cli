# 09 — Testing & Quality Assurance

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

### Test Classification

| Classification | Count |
|---------------|-------|
| Unit tests (inline `#[cfg(test)]`) | ~50+ |
| Integration tests (`tests/`) | ~10 files |
| CLI tests | 5 (cli.rs) + binary tests |
| Numerical tests | Multiple (fit accuracy, Nernst slope) |
| Error-path tests | Multiple (invalid inputs, edge cases) |
| Cross-platform tests | CI runs on Linux + macOS |

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
- **Steps**: checkout, cache, rust-toolchain (stable + rustfmt + clippy), cargo fmt --check, cargo clippy -- -D warnings, cargo test --all, cargo build --release
