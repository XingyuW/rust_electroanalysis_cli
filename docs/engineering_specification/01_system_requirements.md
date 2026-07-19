# 01 — System Requirements

**Identifier:** `DOC-01`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

All requirements are **derived from implemented behaviour**. "Shall" is used only when behaviour is directly supported by the implementation, tests, or project README.

---

## 1. Functional Requirements

Verification confidence labels used in **Verified By**:
- **Direct**: explicit test assertions for the requirement behavior.
- **Partial**: integration/behavior coverage exists, but not every requirement facet is asserted directly.
- **Inferred**: requirement is confirmed from implementation inspection.

| ID | Requirement | Priority | Location | Verified By | Status |
|----|------------|----------|----------|-------------|--------|
| FR-001 | System shall parse CHI-format EIS files and extract frequency, Z′, Z″, Z, and Phase columns | High | `src/data_file/chi_file.rs` | **Direct** — `chi_file` tests, `unified_data_loading` test | ✅ Implemented |
| FR-002 | System shall parse CHI-format OCPT files with automatic header detection | High | `src/data_file/chi_file.rs` | **Direct** — `chi_file` tests | ✅ Implemented |
| FR-003 | System shall parse multi-column OCPT files into independent measurement series | High | `src/data_file/chi_file.rs` | **Direct** — `chi_file` tests | ✅ Implemented |
| FR-004 | System shall parse generic CSV measurement files with automatic column-name detection | High | `src/data_file/measurement_parser.rs` | **Direct** — `phase1_domain` tests | ✅ Implemented |
| FR-005 | System shall parse Excel `.xlsx` workbooks with worksheet selection | High | `src/data_file/excel_file.rs` | **Direct** — `xlsx_ingestion` tests | ✅ Implemented |
| FR-006 | System shall reject legacy `.xls` binary files with a clear error message | Medium | `src/data_file/excel_file.rs` | **Direct** — `unified_data_loading` test | ✅ Implemented |
| FR-007 | System shall reject binary content disguised as CSV | Medium | `src/data_file/input_kind.rs` | **Direct** — `unified_data_loading` test | ✅ Implemented |
| FR-008 | System shall load experiment metadata from TOML files | High | `src/domain/metadata.rs` | **Direct** — `phase1_domain` tests | ✅ Implemented |
| FR-009 | System shall fit a user-specified circuit expression to EIS data using Levenberg-Marquardt | High | `src/impedance/fitting.rs` | **Direct** — `impedance::tests` | ✅ Implemented |
| FR-010 | System shall resolve circuit models from filename/metadata tags and configured rules | High | `src/impedance/circuit_models.rs` | **Direct** — `chi_file` tests | ✅ Implemented |
| FR-011 | System shall search for optimal equivalent-circuit models using a genetic algorithm | High | `src/impedance/ecm_evolution.rs` | **Direct** — `phase0_regression` integration tests | ✅ Implemented |
| FR-012 | System shall generate Nyquist and Bode plots from EIS data with optional fitted curves | High | `src/plottings/eis_plot.rs` | **Partial** — `phase0_regression` covers workflow/file generation; no direct visual-correctness assertions | ✅ Implemented |
| FR-013 | System shall generate time-series plots from CHI/CSV data | High | `src/plottings/chi_plot.rs` | **Partial** — `phase0_regression` covers workflow/file generation; no direct visual-correctness assertions | ✅ Implemented |
| FR-014 | System shall generate generic scatter/regression plots with configurable axis transforms | Medium | `src/plottings/generic_plot.rs` | **Partial** — `plot_config` tests cover config and dispatch behavior | ✅ Implemented |
| FR-015 | System shall fit transient exponential models to potentiometric event responses | High | `src/potentiometry/transient/` | **Direct** — `phase2_transient` tests | ✅ Implemented |
| FR-016 | System shall extract equilibrium calibration observations from transient fit results | High | `src/potentiometry/calibration/observations.rs` | **Direct** — `phase3_calibration` tests | ✅ Implemented |
| FR-017 | System shall fit Nernst, Nicolsky-Eisenman, and conductivity-empirical calibration models | High | `src/potentiometry/calibration/` | **Direct** — `phase3_calibration` tests | ✅ Implemented |
| FR-018 | System shall validate stored calibration models against observation sets | High | `src/potentiometry/calibration/validation.rs` | **Direct** — `phase3_calibration` tests | ✅ Implemented |
| FR-019 | System shall predict activity/concentration from a calibration model and potential | High | `src/potentiometry/calibration/prediction.rs` | **Direct** — `phase3_calibration` tests | ✅ Implemented |
| FR-020 | System shall characterize signal quality (PSD, Allan, drift, spikes, statistics) | Medium | `src/signal/` | **Direct** — `phase5_signal_health` tests | ✅ Implemented |
| FR-021 | System shall construct sensor health baselines from multiple signal analysis results | Medium | `src/health/baseline.rs` | **Partial** — `phase5_signal_health` covers baseline creation; not all cross-artifact combinations are asserted directly | ✅ Implemented |
| FR-022 | System shall assess individual sensor health using evidence-based rules | Medium | `src/health/assessment.rs` | **Partial** — `phase5_signal_health` covers representative rule paths; full rule-surface combinations are inferred | ✅ Implemented |
| FR-023 | System shall compare EIS-derived and transient-derived characteristic timescales | Medium | `src/mechanism/` | **Inferred** — implementation inspection with partial integration coverage | ✅ Implemented |
| FR-024 | System shall estimate latent states using Extended and Unscented Kalman Filters | Low | `src/estimation/` | **Direct** — `phase6_estimation` tests | ✅ Implemented |
| FR-025 | System shall track input/config file provenance via SHA-256 hashing | Medium | `src/domain/provenance.rs` | **Direct** — `provenance` unit tests | ✅ Implemented |
| FR-026 | System shall generate human-readable text reports for EIS fits | Medium | `src/impedance/reporting.rs` | **Direct** — `chi_file` tests, `phase0_regression` | ✅ Implemented |
| FR-027 | System shall produce JSON artifacts for all major analysis results | Medium | `src/results/` | **Partial** — integration tests cover representative JSON artifacts; cross-workflow completeness is implementation-verified | ✅ Implemented |
| FR-028 | System shall produce CSV feature tables for transient and calibration results | Medium | Various runners | **Partial** — `phase2_transient` and `phase3_calibration` cover core exports; configurable filename variants are implementation-verified | ✅ Implemented |

## 2. Scientific Requirements

| ID | Requirement | Location | Verified By |
|----|------------|----------|-------------|
| SCI-001 | Nernst equation shall use R = 8.31446261815324 J/(mol·K), F = 96485.33212 C/mol | `src/potentiometry/calibration/nernst.rs` | Unit tests |
| SCI-002 | Theoretical Nernst slope shall be (RT ln 10)/(zF) volts per decade | `src/potentiometry/calibration/nernst.rs` | Unit test (≈59.16 mV at 298.15 K, z=1) |
| SCI-003 | Temperature input in Celsius shall be converted to Kelvin by adding 273.15 | `src/potentiometry/units.rs` | Unit test |
| SCI-004 | CPE impedance shall be Z = 1/(Q·(jω)^α) | `src/impedance/elements.rs` | Unit test (α=1 → ideal capacitor) |
| SCI-005 | Warburg (infinite) impedance shall be Z = σ(1−j)/√ω | `src/impedance/elements.rs` | (Inferred) |
| SCI-006 | Finite-length Warburg (Open) shall be Z = Z₀·coth(√(jωτ))/√(jωτ) | `src/impedance/elements.rs` | (Inferred) |
| SCI-007 | Circuit parallel combination shall use admittance summation: 1/Z = Σ(1/Zᵢ) | `src/impedance/circuits.rs` | (Verified from code) |
| SCI-008 | Linear regression shall use ordinary least squares | `src/regression_mod.rs` | Unit tests |
| SCI-009 | Transient single-exponential shall be E(t) = E∞ + A·exp(−t/τ) | `src/potentiometry/transient/models.rs` | (Verified from code) |
| SCI-010 | Transient stretched-exponential shall be E(t) = E∞ + A·exp(−(t/τ)^β) | `src/potentiometry/transient/models.rs` | (Verified from code) |

## 3. Numerical Requirements

| ID | Requirement | Location |
|----|------------|----------|
| NUM-001 | Circuit impedance evaluation at DC (ω → 0) shall use fallback real impedance (1e12 Ω) instead of division by zero | `src/impedance/elements.rs` |
| NUM-002 | Parallel admittance near zero shall use fallback real impedance (1e12 Ω) | `src/impedance/circuits.rs` |
| NUM-003 | Parameter constraints (Positive, ZeroOne, None) shall be enforced via log/exp transforms during optimization | `src/impedance/fitting.rs` |
| NUM-004 | Levenberg-Marquardt shall use weighted residuals with configurable weights | `src/impedance/fitting.rs`, `src/impedance/lib.rs` |
| NUM-005 | FFT-based PSD (Welch method) for signal analysis shall use configurable segment size and overlap | `src/signal/psd.rs` |
| NUM-006 | Kalman filter innovation sequences shall be monitored for consistency | `src/estimation/innovation.rs` |

## 4. Data Requirements

| ID | Requirement | Location |
|----|------------|----------|
| DAT-001 | Missing values in measurement channels shall be represented as `Option<f64>` (None) | `src/domain/measurement.rs` |
| DAT-002 | Measurement time axis must be non-empty, with all finite timestamps | `src/domain/measurement.rs` |
| DAT-003 | Channel values must align with the shared time axis length | `src/domain/measurement.rs` |
| DAT-004 | Environmental series must have matching time/value lengths | `src/domain/experiment.rs` |
| DAT-005 | Experiment events must have finite timestamps | `src/domain/experiment.rs` |
| DAT-006 | Configuration files shall use TOML format with schema_version field | Various config modules |
| DAT-007 | JSON results shall include schema_version for forward compatibility | All `src/results/` modules |

## 5. Performance Requirements

| ID | Requirement | Status |
|----|------------|--------|
| PERF-001 | ECM search shall be configurable for population size and generation count | ✅ |
| PERF-002 | Release builds shall use LTO, single codegen unit, and symbol stripping | ✅ (Cargo.toml) |
| PERF-003 | Parallel evaluation in ECM search via rayon | ✅ |

## 6. Reliability Requirements

| ID | Requirement | Status |
|----|------------|--------|
| REL-001 | Workspace directories shall be created automatically on startup | ✅ |
| REL-002 | Default configuration files shall be generated when absent | ✅ |
| REL-003 | Legacy configuration file names shall be checked with migration warnings | ✅ |
| REL-004 | Last-run state shall be persisted in `config/app.toml` | ✅ |

## 7. Maintainability Requirements

| ID | Requirement | Status |
|----|------------|--------|
| MNT-001 | Scientific equations shall reside in dedicated modules (impedance/, potentiometry/) | ✅ |
| MNT-002 | CLI parsing shall be separated from workflow orchestration | ✅ |
| MNT-003 | Result structures shall be serializable and independent of rendering | ✅ |
| MNT-004 | All public error types shall implement `std::error::Error` via thiserror | ✅ |
| MNT-005 | Code shall compile without warnings under `cargo clippy -- -D warnings` | ✅ (CI enforced) |

## 8. Portability Requirements

| ID | Requirement | Status |
|----|------------|--------|
| POR-001 | System shall build on Linux (ubuntu-latest) and macOS (macos-latest) | ✅ (CI matrix) |
| POR-002 | No platform-specific `unsafe` code shall exist | ✅ (Verified: zero unsafe blocks) |
| POR-003 | File paths shall use `Path`/`PathBuf` for cross-platform compatibility | ✅ |

## 9. Reproducibility Requirements

| ID | Requirement | Status |
|----|------------|--------|
| REP-001 | Every analysis result shall include an `AnalysisProvenance` record | ✅ |
| REP-002 | Input file SHA-256 hash shall be stored in provenance | ✅ |
| REP-003 | Configuration file SHA-256 shall be stored when a config file is used | ✅ |
| REP-004 | Software version shall be recorded in provenance | ✅ |

## 10. User-Interface and CLI Requirements

| ID | Requirement | Status |
|----|------------|--------|
| CLI-001 | Help text shall be displayed when no arguments are provided | ✅ |
| CLI-002 | Version shall be displayed with `--version` | ✅ |
| CLI-003 | Legacy flat flags (--plot, --search-eis) shall be normalized to structured subcommands | ✅ |
| CLI-004 | Mutually exclusive flag combinations shall produce clear error messages | ✅ (`cli.rs` tests) |
| CLI-005 | All subcommands shall accept `--help` for argument documentation | ✅ (clap derive) |
| CLI-006 | Configuration paths shall be overridable via CLI arguments | ✅ |

## 11. Uncertainty and Limitations

- **REQ-UNC-001**: Performance requirements are not benchmarked; no performance regression tests exist.
- **REQ-UNC-002**: No explicit requirements exist for numerical precision beyond IEEE 754 f64.
- **REQ-UNC-003**: No formal specification of acceptable fitting residual thresholds exists; these are configuration-driven.
- **REQ-UNC-004**: Estimation module (EKF/UKF) requirements are the least mature; process/measurement noise models are not fully verified from the source.
