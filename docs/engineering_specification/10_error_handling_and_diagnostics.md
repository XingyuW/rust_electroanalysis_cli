# 10 — Error Handling & Diagnostics

**Identifier:** `DOC-10`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## 1. Error Type Hierarchy

```
ApplicationError (main.rs)
├── CliError (cli.rs)
├── ConfigurationError (domain/errors.rs)
├── WorkspaceError (domain/errors.rs)
└── RunnerError (runners/mod.rs)
    ├── ConfigurationError
    ├── DataParsingError
    ├── FittingError
    ├── ReportingError
    ├── WorkspaceError
    ├── PotentiometryError
    ├── CalibrationError
    ├── SignalError
    ├── HealthError
    ├── EstimationError
    ├── Json (serde_json::Error)
    ├── Toml (toml::de::Error)
    ├── Io (io::Error)
    ├── Csv (csv::Error)
    ├── Backend (Box<dyn Error>)
    └── Message (String)
```

## 2. Domain Error Types

### ConfigurationError
- `Io { path, source }` — File I/O failure
- `Parse { path, source }` — TOML deserialization failure
- `Serialize` — TOML serialization failure
- `Invalid(String)` — Semantic validation failure

### DataParsingError
- `Io { path, source }` — Data file I/O failure
- `Invalid { path, message }` — Structural/semantic failure
- `Configuration` — Configuration-dependent error
- `Fitting` — Fitting-dependent error
- `Provenance` — Provenance-dependent error

### FittingError
- `InvalidInput`, `CircuitParse`, `Optimizer`, `Search`, `Regression`, `Io`

### ReportingError
- `Fitting`, `ParameterCountMismatch`, `Io`, `Invalid`

### WorkspaceError
- `Io`, `Configuration`, `Invalid`

### PlottingError
- `Fitting`, `Data`

### ProvenanceError
- `Io`, `Timestamp`

## 3. Workflow-Specific Error Types

| Error Type | Location | Variants |
|-----------|----------|----------|
| `PotentiometryError` | `potentiometry/error.rs` | Invalid, MissingChannel, NoEligibleEvents, InvalidEventWindow, AllCandidateModelsFailed, etc. |
| `CalibrationError` | `potentiometry/calibration/error.rs` | NoObservations, AllModelsFailed, InvalidObservation, InvalidConfiguration, InvalidPrediction, etc. |
| `SignalError` | `signal/error.rs` | Various signal-specific errors |
| `HealthError` | `health/error.rs` | Various health-specific errors |
| `EstimationError` | `estimation/error.rs` | Various estimation-specific errors |
| `UnitError` | `potentiometry/units.rs` | Unknown, Incompatible, MissingMolarMass, NonFinite, NonPhysicalTemperature |

## 4. Error Propagation Paths

1. **Scientific modules** → domain errors or workflow-specific errors
2. **Runners** → wrap everything into `RunnerError` via `From` impls
3. **main.rs** → wraps `RunnerError` + `CliError` + `ConfigurationError` + `WorkspaceError` into `ApplicationError`
4. **User sees**: `Error: <message>` on stderr, exit code 1

## 5. User-Facing Messages

- All errors implement `Display` via `thiserror`
- Messages include contextual information (paths, channel names, event kinds)
- Warnings are emitted to stderr via `eprintln!("Warning: {warning}")` during workspace setup and config loading
- Plot configuration warnings are printed after loading

## 6. Warning Behaviour

### Workflow Warnings (Non-Fatal)

Warnings are collected in result structures (e.g., `TransientEventResult.warnings`, `CalibrationAnalysisReport.warnings`) and serialized into JSON output. They do **not** cause non-zero exit codes.

| Warning Type | Trigger |
|-------------|---------|
| Irregular sampling | Non-uniform time intervals |
| Duplicate timestamps | Repeated timestamps in data |
| Long time constant | τ > configured ratio × window |
| Poor tau separation | τ_fast/τ_slow < configured ratio |
| Negligible amplitude | A < configured fraction of response |
| Parameter at bound | Fitted value within 1% of bound |
| Singular covariance | Non-invertible Jacobian |
| High residual autocorrelation | lag-1 autocorrelation > 0.8 |
| Bootstrap unavailable | < minimum_success_fraction successful iterations |
| All models failed | No candidate model converged |
| High hysteresis | > configured threshold |
| Insufficient concentration levels | < 3 distinct activity levels |
| Prediction extrapolation | Input outside training domain |
| Nicolsky non-identifiable | Selectivity coefficient cannot be determined |

## 7. Runtime Panic-Site Audit (`unwrap`/`expect`/`unreachable!`)

Audit scope is **runtime code only** (test modules excluded).  
Current inventory: **43 runtime sites** (`unwrap`: 21, `expect`: 13, `unreachable!`: 9).

### Distribution by subsystem

| Subsystem | Site count | Representative files |
|-----------|------------|----------------------|
| `potentiometry/*` | 12 | `transient/models.rs`, `units.rs`, `calibration/environment.rs` |
| `impedance/*` | 10 | `ecm_candidate.rs`, `ecm_evolution.rs`, `fitting.rs` |
| `estimation/*` | 9 | `environment.rs`, `ekf.rs`, `ukf.rs`, `validation.rs` |
| `plottings/*` | 6 | `estimation_plot.rs`, `health_plot.rs`, `signal_plot.rs` |
| `data_file/*` | 3 | `data_op.rs`, `excel_file.rs` |
| `runners/*` | 2 | `runners/fit.rs` |
| `signal/*` | 1 | `signal/sampling.rs` |

### Runtime impact tiers

| Tier | Description | Representative locations | Current assessment |
|------|-------------|--------------------------|--------------------|
| Tier A | Potentially user-triggerable panic if runtime data/invariants become empty or malformed unexpectedly | `plottings/{estimation,health,signal}_plot.rs`, `signal/sampling.rs`, `data_file/excel_file.rs` | **Medium impact / Low likelihood** (most paths are pre-guarded, but failures would abort release binary because `panic = "abort"`). |
| Tier B | Invariant-guarded assumptions in scientific/model code | `impedance/ecm_candidate.rs`, `potentiometry/transient/models.rs`, `potentiometry/units.rs`, `estimation/environment.rs` | **Low-medium impact / Very low to low likelihood** (callers validate dimensions/branches before these sites). |
| Tier C | Internal consistency and fail-fast guards (poisoned locks, impossible branches, artifact construction assumptions) | `impedance/ecm_evolution.rs`, `runners/fit.rs`, `data_file/data_op.rs`, `estimation/mod.rs` | **Low impact / Very low likelihood** under normal operation; still panic-abort when triggered. |

### Notes

- This inventory supersedes the earlier plot-only panic note.
- Test-only `unwrap`/`expect` usage remains out of scope.

## 8. Recoverable vs Unrecoverable Failures

| Type | Behaviour |
|------|-----------|
| Recoverable | Most operational failures are structured errors (`Result`), reported to stderr with exit code 1 |
| Unrecoverable (panic-abort) | Runtime `unwrap`/`expect`/`unreachable!` sites listed in Section 7 can abort process in invariant-violation paths |
| Graceful degradation | Warnings are collected in artifacts; partial results are returned where workflow design permits |

## 9. Partial-Output Behaviour

- **Transient analysis**: If some events fail, successful events are still returned in the report
- **Calibration**: If some models fail, converged models are still reported; failure only if all models fail
- **ECM search**: Empty search result is valid (not an error)
- **Signal analysis**: Individual analysis failures produce warnings, not errors

## 10. Debugging Artifacts

- `ParseDiagnostics` is stored in `TransientAnalysisReport` for inspection
- Provenance records (SHA-256 + timestamps) allow reproducibility verification
- Configuration is cloned into result reports for audit trail
- Last-run state is persisted in `config/app.toml`

## 11. Context Loss in Error Chains

| Error Path | Context Preserved? |
|------------|-------------------|
| File I/O | ✅ Path included |
| Config parse | ✅ Path + TOML error |
| Channel lookup | ✅ Channel name included |
| Circuit parse | ✅ Expression string |
| Optimizer failure | ✅ Termination reason |
| ECM search | ✅ Error message |
| Calibration fit | ✅ Model kind, convergence reason |
| Transient fit | ✅ Model kind, event index |

**No identified context-loss issues** in the error propagation chain.
