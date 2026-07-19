# 08 — Validation & Constraints

**Identifier:** `DOC-08`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## 1. Input Validation

| ID | Condition | Behaviour | Location | Severity |
|----|-----------|-----------|----------|----------|
| VAL-001 | Measurement time axis empty | Error: "measurement time axis is empty" | `domain/measurement.rs` | Error |
| VAL-002 | No channels in measurement | Error: "measurement has no channels" | `domain/measurement.rs` | Error |
| VAL-003 | Non-finite timestamp | Error | `domain/measurement.rs` | Error |
| VAL-004 | Channel length ≠ time axis length | Error with channel name and counts | `domain/measurement.rs` | Error |
| VAL-005 | Non-finite channel values | Error | `domain/measurement.rs` | Error |
| VAL-006 | Negative or non-finite variance | Error | `domain/measurement.rs` | Error |
| VAL-007 | Environmental series length mismatch | Error | `domain/experiment.rs` | Error |
| VAL-008 | Non-finite event timestamp | Error | `domain/experiment.rs` | Error |
| VAL-009 | Non-finite event value | Error | `domain/experiment.rs` | Error |
| VAL-010 | Empty frequency array in EIS | Error: "frequencies cannot be empty" | `impedance/lib.rs` | Error |
| VAL-018 | Fewer than 3 valid EIS rows after preprocessing | Error: "not enough valid impedance points after preprocessing" | `impedance/lib.rs` | Error |
| VAL-011 | Fewer than 2 data points for regression | Error: "requires at least 2" | `regression_mod.rs` | Error |
| VAL-012 | Identical x values in regression | Error: "indeterminate" | `regression_mod.rs` | Error |
| VAL-013 | --search-top = 0 | CLI error: "must be greater than zero" | `cli.rs` | Error |
| VAL-014 | Mutually exclusive --potential/--input | CLI error | `cli.rs` | Error |
| VAL-015 | --input without --channel in predict | CLI error | `cli.rs` | Error |
| VAL-016 | Binary content in CSV file | Error: "unsupported" | `data_file/input_kind.rs` | Error |
| VAL-017 | Legacy .xls file | Error: ".xls" reference | `data_file/excel_file.rs` | Error |

## 2. Parameter Bound Enforcement

| ID | Parameter | Bounds | Enforcement | Location |
|----|-----------|--------|-------------|----------|
| BND-001 | All R elements | [1e-12, 1e12] Ω | clamp_to_bounds + transform | `impedance/elements.rs` |
| BND-002 | C elements | [1e-15, 1e3] F | clamp_to_bounds | `impedance/elements.rs` |
| BND-003 | CPE α | [0.05, 1.0] | transform_forward/backward (ZeroOne) | `impedance/elements.rs` |
| BND-004 | Transient τ | τ > 0 (finite) | validate_tau | `potentiometry/transient/models.rs` |
| BND-005 | Transient β | β > 0 (finite) | Explicit check | `potentiometry/transient/models.rs` |
| BND-006 | Double-exponential τ ordering | 0 < τ_fast < τ_slow | validate_ordered_taus | `potentiometry/transient/models.rs` |
| BND-007 | Nernst temperature | T > 0 K | effective_temperature_k | `potentiometry/calibration/nernst.rs` |
| BND-008 | Nernst ion charge | z ≠ 0 | theoretical_slope check | `potentiometry/calibration/nernst.rs` |
| BND-009 | Nernst slope for inversion | |slope| ≥ 1e-15 | activity_from_potential | `potentiometry/calibration/nernst.rs` |

## 3. Solver Convergence Validation

| ID | Check | Location |
|----|-------|----------|
| CNV-001 | Levenberg-Marquardt convergence status recorded | `impedance/fitting.rs` |
| CNV-002 | Transient fit status (Converged/Failed/Invalid) | `results/transient.rs` |
| CNV-003 | Calibration fit status (Converged/Failed/Invalid) | `results/calibration.rs` |
| CNV-004 | Bootstrap minimum success fraction (default 0.80) | Config validation |
| CNV-005 | All-models-failed detection for transient | `potentiometry/transient/mod.rs` |
| CNV-006 | All-models-failed detection for calibration | `potentiometry/calibration/mod.rs` |

## 4. Scientific Plausibility Checks

| ID | Check | Severity | Location |
|----|-------|----------|----------|
| PLS-001 | Fewer than 3 distinct activity levels → warning | Warning | `potentiometry/calibration/mod.rs` |
| PLS-002 | High hysteresis (> threshold) → warning | Warning | `potentiometry/calibration/mod.rs` |
| PLS-003 | Non-Nernstian slope → warning | Warning | `results/calibration.rs` |
| PLS-004 | Slope sign inconsistent → warning | Warning | `results/calibration.rs` |
| PLS-005 | Poor tau separation (< configurable ratio) → warning | Warning | `potentiometry/transient/` |
| PLS-006 | Long time constant (> window ratio) → warning | Warning | `potentiometry/transient/` |
| PLS-007 | Negligible amplitude → warning | Warning | `potentiometry/transient/` |
| PLS-008 | Parameter at bound → warning | Warning | (Inferred) |
| PLS-009 | High residual autocorrelation → warning | Warning | `potentiometry/transient/` |
| PLS-010 | Singular covariance → warning | Warning | `potentiometry/transient/`, `potentiometry/calibration/` |
| PLS-011 | Prediction extrapolation → warning | Warning | `results/calibration.rs` |

## 5. NaN and Infinity Handling

- **Rejection during validation**: `is_finite()` checks on all measurement data
- **DC limit fallbacks**: Large real values (1e6, 1e12 Ω) instead of infinity
- **Division safeguards**: `norm_sqr() > 1e-16` checks before division in circuit impedance and admittance
- **Non-finite optimization parameters**: Clamped to lower bounds

## 6. Missing-Data Handling

| Scenario | Behaviour |
|----------|-----------|
| Missing channel cell | Represented as `None` in `Vec<Option<f64>>` |
| Missing environmental value | `None` in environmental series |
| Configurable max missing fraction | Transient: 0.20, Calibration: 0.20 |
| Transient segment with excessive missing | Warning or rejection based on fraction |

## 7. Duplicate Timestamp Handling

| Workflow | Policy | Behaviour |
|----------|--------|-----------|
| Transient segmentation | `error` (default) | Reject duplicate groups with `DuplicateTimestamps` error |
| Transient segmentation | `average` | Average each duplicate timestamp group before fitting |
| Signal sampling | `error` (default) | Reject duplicate groups with sampling error |
| Signal sampling | `average` | Replace duplicate group with mean finite value |
| Signal sampling | `first` | Replace duplicate group with first value |
| Signal sampling | `last` | Replace duplicate group with last value |

Related timestamp controls are separate from duplicate policy:
- Transient: `non_monotonic_policy` (`sort`/`error`), `irregular_sampling_policy` (`allow`/`error`)
- Signal: `non_monotonic_timestamp_policy` (`sort_paired`/`error`)

## 8. Output Validation

- All output directories are created via `fs::create_dir_all` before writing
- JSON serialization errors are propagated as `RunnerError::Json`
- CSV writing errors are propagated as `RunnerError::Csv`
- Plot rendering failures are propagated as `RunnerError::Backend`

## 9. Cross-Workflow Input Validation

When a workflow consumes results from another workflow (e.g., health consuming signal results), the input JSON is deserialized with serde. Missing fields use `Default` or `Option` types. Schema version mismatches are **not** currently checked at runtime — this is a deferred concern.
