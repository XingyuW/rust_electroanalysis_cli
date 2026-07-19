# 03 — Module Specifications

**Identifier:** `DOC-03`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## Source-to-Specification Mapping

| Source Path | Module Purpose | Documented |
|------------|---------------|-----------|
| `src/main.rs` | CLI binary entrypoint, command dispatch | ✅ |
| `src/cli.rs` | clap derive command tree, validation, legacy normalization | ✅ |
| `src/lib.rs` | Crate root, public re-exports, module declarations | ✅ |
| `src/workspace.rs` | Workspace bootstrap, config lifecycle, directory management | ✅ |
| `src/domain/mod.rs` | Domain module root | ✅ |
| `src/domain/errors.rs` | Typed errors (Configuration, DataParsing, Fitting, Workspace, Plotting, Reporting, Provenance) | ✅ |
| `src/domain/experiment.rs` | ElectrochemicalExperiment, SensorMetadata, ReferenceMetadata, ExperimentEvent, EnvironmentalSeries | ✅ |
| `src/domain/measurement.rs` | MultiChannelMeasurement, MeasurementChannel | ✅ |
| `src/domain/metadata.rs` | ExperimentMetadataDocument, TOML loading | ✅ |
| `src/domain/provenance.rs` | AnalysisProvenance (SHA-256, timestamps) | ✅ |
| `src/domain/diagnostics.rs` | ParseDiagnostics, MeasurementParseResult | ✅ |
| `src/data_file/lib.rs` | Data ingestion module root | ✅ |
| `src/data_file/chi_file.rs` | CHI-format parser (EIS, OCPT) | ✅ |
| `src/data_file/data_op.rs` | PlotData container, IntoPlotData trait | ✅ |
| `src/data_file/excel_file.rs` | Excel .xlsx reader (calamine) | ✅ |
| `src/data_file/input_kind.rs` | File format detection | ✅ |
| `src/data_file/measurement_adapter.rs` | Conversion from domain measurements to PlotData | ✅ |
| `src/data_file/measurement_parser.rs` | Generic CSV measurement parser | ✅ |
| `src/data_file/value_transform.rs` | Axis transform resolution for plotting | ✅ |
| `src/impedance/lib.rs` | Impedance module root, fit_circuit, lin_kk | ✅ |
| `src/impedance/elements.rs` | 15 circuit element types with impedance equations | ✅ |
| `src/impedance/circuits.rs` | CircuitNode AST, nom-based parser | ✅ |
| `src/impedance/circuit_models.rs` | Circuit model resolver (rules, metadata, fallback) | ✅ |
| `src/impedance/fitting.rs` | Levenberg-Marquardt fitter, parameter transforms | ✅ |
| `src/impedance/pinn_optimizer.rs` | PINN-based optimizer (experimental) | ✅ |
| `src/impedance/ecm_candidate.rs` | Genetic encoding of circuit candidates | ✅ |
| `src/impedance/ecm_evolution.rs` | Genetic algorithm for ECM search | ✅ |
| `src/impedance/ecm_scoring.rs` | Candidate fitness scoring | ✅ |
| `src/impedance/ecm_search.rs` | Search report assembly | ✅ |
| `src/impedance/reporting.rs` | Human-readable fit reports | ✅ |
| `src/potentiometry/mod.rs` | Potentiometry module root | ✅ |
| `src/potentiometry/error.rs` | PotentiometryError | ✅ |
| `src/potentiometry/units.rs` | Quantity, QuantityUnit, unit conversion | ✅ |
| `src/potentiometry/transient/mod.rs` | Transient analysis orchestration | ✅ |
| `src/potentiometry/transient/models.rs` | Transient model equations (single, double, double-drift, stretched) | ✅ |
| `src/potentiometry/transient/fitting.rs` | Transient model fitting | ✅ |
| `src/potentiometry/transient/segmentation.rs` | Event-based data segmentation | ✅ |
| `src/potentiometry/transient/selection.rs` | Model selection (AIC, BIC) | ✅ |
| `src/potentiometry/transient/diagnostics.rs` | Fit statistics computation | ✅ |
| `src/potentiometry/calibration/mod.rs` | Calibration orchestration | ✅ |
| `src/potentiometry/calibration/error.rs` | CalibrationError definitions and typed failure mapping | ✅ |
| `src/potentiometry/calibration/nernst.rs` | Nernst equation, slope, activity inversion | ✅ |
| `src/potentiometry/calibration/nicolsky_eisenman.rs` | Nicolsky-Eisenman equation | ✅ |
| `src/potentiometry/calibration/activity.rs` | Activity coefficient models | ✅ |
| `src/potentiometry/calibration/observations.rs` | Observation extraction | ✅ |
| `src/potentiometry/calibration/fitting.rs` | Calibration model fitting | ✅ |
| `src/potentiometry/calibration/validation.rs` | Cross-validation | ✅ |
| `src/potentiometry/calibration/prediction.rs` | Activity/concentration prediction | ✅ |
| `src/potentiometry/calibration/uncertainty.rs` | Bootstrap uncertainty | ✅ |
| `src/potentiometry/calibration/environment.rs` | Environmental data alignment | ✅ |
| `src/potentiometry/calibration/ionic_strength.rs` | Ionic strength computation | ✅ |
| `src/signal/mod.rs` | Signal analysis module root | ✅ |
| `src/signal/error.rs` | SignalError | ✅ |
| `src/signal/statistics.rs` | Descriptive statistics | ✅ |
| `src/signal/psd.rs` | Power spectral density (Welch/FFT) | ✅ |
| `src/signal/allan.rs` | Allan variance analysis | ✅ |
| `src/signal/drift.rs` | Drift analysis (linear, Theil-Sen) | ✅ |
| `src/signal/spikes.rs` | Spike/outlier detection | ✅ |
| `src/signal/correlation.rs` | Channel correlation | ✅ |
| `src/signal/residuals.rs` | Residual analysis | ✅ |
| `src/signal/sampling.rs` | Sampling analysis | ✅ |
| `src/signal/windows.rs` | Signal windowing | ✅ |
| `src/signal/comparison.rs` | Signal comparison | ✅ |
| `src/health/mod.rs` | Health module root | ✅ |
| `src/health/baseline.rs` | Baseline construction | ✅ |
| `src/health/assessment.rs` | Health assessment | ✅ |
| `src/health/features.rs` | Feature extraction | ✅ |
| `src/health/rules.rs` | Health assessment rules | ✅ |
| `src/health/evidence.rs` | Evidence synthesis | ✅ |
| `src/health/normalization.rs` | Normalization | ✅ |
| `src/health/trend.rs` | Trend analysis | ✅ |
| `src/health/error.rs` | HealthError | ✅ |
| `src/estimation/mod.rs` | Estimation module root | ✅ |
| `src/estimation/ekf.rs` | Extended Kalman Filter | ✅ |
| `src/estimation/ukf.rs` | Unscented Kalman Filter | ✅ |
| `src/estimation/state.rs` | State vector definitions | ✅ |
| `src/estimation/model.rs` | Process/measurement models | ✅ |
| `src/estimation/initialization.rs` | Filter initialization | ✅ |
| `src/estimation/measurement.rs` | Measurement ingestion | ✅ |
| `src/estimation/process.rs` | Process noise models | ✅ |
| `src/estimation/covariance.rs` | Covariance management | ✅ |
| `src/estimation/innovation.rs` | Innovation monitoring | ✅ |
| `src/estimation/observability.rs` | Observability analysis | ✅ |
| `src/estimation/smoothing.rs` | State smoothing | ✅ |
| `src/estimation/timestamp.rs` | Timestamp handling | ✅ |
| `src/estimation/simulation.rs` | Simulation | ✅ |
| `src/estimation/validation.rs` | Validation | ✅ |
| `src/estimation/comparison.rs` | Filter comparison | ✅ |
| `src/estimation/calibration_adapter.rs` | Calibration model adapter | ✅ |
| `src/estimation/environment.rs` | Environmental input handling | ✅ |
| `src/estimation/error.rs` | EstimationError | ✅ |
| `src/mechanism/mod.rs` | Mechanism module root | ✅ |
| `src/mechanism/timescale.rs` | Timescale extraction | ✅ |
| `src/mechanism/matching.rs` | Timescale matching | ✅ |
| `src/mechanism/evidence.rs` | Evidence synthesis | ✅ |
| `src/mechanism/interpretation.rs` | Mechanism interpretation | ✅ |
| `src/mechanism/trend.rs` | Trend analysis | ✅ |
| `src/mechanism/uncertainty.rs` | Uncertainty handling | ✅ |
| `src/mechanism/error.rs` | MechanismError | ✅ |
| `src/plottings/lib.rs` | Plotting module root | ✅ |
| `src/plottings/plotting.rs` | Core renderer, PlotSeries, publication pipeline | ✅ |
| `src/plottings/eis_plot.rs` | Nyquist/Bode plot pipeline | ✅ |
| `src/plottings/chi_plot.rs` | CHI time-series plot pipeline | ✅ |
| `src/plottings/generic_plot.rs` | Generic scatter/regression plot pipeline | ✅ |
| `src/plottings/transient_plot.rs` | Transient analysis plot pipeline | ✅ |
| `src/plottings/calibration_plot.rs` | Calibration plot pipeline | ✅ |
| `src/plottings/signal_plot.rs` | Signal analysis plot pipeline | ✅ |
| `src/plottings/health_plot.rs` | Health assessment plot pipeline | ✅ |
| `src/plottings/mechanism_plot.rs` | Mechanism comparison plot pipeline | ✅ |
| `src/plottings/estimation_plot.rs` | State estimation plot pipeline | ✅ |
| `src/results/mod.rs` | Results module root, CircuitFitResult | ✅ |
| `src/results/eis.rs` | EIS result types | ✅ |
| `src/results/transient.rs` | Transient result types (event, fit, report) | ✅ |
| `src/results/calibration.rs` | Calibration result types (observation, model, report) | ✅ |
| `src/results/signal.rs` | Signal analysis result types | ✅ |
| `src/results/health.rs` | Health assessment result types | ✅ |
| `src/results/estimation.rs` | State estimation result types | ✅ |
| `src/results/mechanism.rs` | Mechanism comparison result types | ✅ |
| `src/runners/mod.rs` | Runner module root, RunnerError | ✅ |
| `src/runners/plot.rs` | Plot workflow coordinator | ✅ |
| `src/runners/fit.rs` | EIS fit workflow coordinator | ✅ |
| `src/runners/search.rs` | ECM search workflow coordinator | ✅ |
| `src/runners/transient.rs` | Transient analysis workflow coordinator | ✅ |
| `src/runners/calibration.rs` | Calibration workflow coordinator | ✅ |
| `src/runners/mechanism.rs` | Mechanism comparison workflow coordinator | ✅ |
| `src/runners/signal.rs` | Signal analysis workflow coordinator | ✅ |
| `src/runners/health.rs` | Health assessment workflow coordinator | ✅ |
| `src/runners/estimation.rs` | State estimation workflow coordinator | ✅ |
| `src/fitting/mod.rs` | Public fitting façade | ✅ |
| `src/regression_mod.rs` | Linear regression for plot overlays | ✅ |
| `src/plot_config.rs` | Plotting TOML schema, loading, migration | ✅ |
| `src/search_config.rs` | ECM search TOML schema | ✅ |
| `src/transient_config.rs` | Transient analysis TOML schema | ✅ |
| `src/calibration_config.rs` | Calibration TOML schema | ✅ |
| `src/mechanism_config.rs` | Mechanism comparison TOML schema | ✅ |
| `src/signal_config.rs` | Signal analysis TOML schema | ✅ |
| `src/health_config.rs` | Health assessment TOML schema | ✅ |
| `src/estimation_config.rs` | State estimation TOML schema | ✅ |
| `src/plot_runner.rs` | (Legacy) Plot orchestration adapter | ✅ |
| `src/search_runner.rs` | (Legacy) Search orchestration adapter | ✅ |

**Coverage check:** 140/140 Rust source paths under `src/**/*.rs` are mapped above.

---

## Key Module Specifications

### `domain/` — Shared Application-Domain Contracts

- **Purpose**: Type-safe shared contracts for measurements, experiments, errors, and provenance.
- **Public types**: `ElectrochemicalExperiment`, `MultiChannelMeasurement`, `MeasurementChannel`, `AnalysisProvenance`, `ExperimentEvent`, 7 error enums.
- **No dependencies on CLI, plotting, or scientific modules**.
- **Invariant**: All constructed `ElectrochemicalExperiment` instances have validated measurements, events sorted by timestamp, and provenance attached.

### `impedance/` — EIS Circuit Models and Fitting

- **Purpose**: Circuit element impedance equations, circuit AST parsing, Levenberg-Marquardt fitting, genetic model search.
- **15 elements**: R, C, L, W, CPE, Wo, Ws, La, Gw, G, Gs, K, Zarc, TLMQ, T.
- **Circuit parser**: Nom-based, supports series (`-`) and parallel (`p(…)`) composition.
- **Fitting**: log/exp transforms for parameter constraints, weighted residuals, modulus weighting.
- **ECM search**: genevo-based genetic programming with seed circuits, crossover, mutation, fitness scoring.

### `potentiometry/` — Potentiometric Analysis

- **Transient models**: Single-exponential, Double-exponential, Double-with-drift, Stretched-exponential.
- **Calibration**: Nernst (`E = E⁰ + S·log₁₀(a)`), Nicolsky-Eisenman, activity coefficient models (Ideal, Davies, Extended Debye-Hückel).
- **Unit handling**: `Quantity` + `QuantityUnit` for concentration, potential, temperature, conductivity with conversion.

### `signal/` — Signal Quality Analysis

- **Analyses**: Sampling statistics, descriptive statistics, PSD (Welch via FFT), Allan variance, drift (linear, Theil-Sen), spike detection, channel correlation, residual analysis.
- **Output**: `SignalAnalysisReport` with JSON serialization.

### `estimation/` — Latent State Estimation

- **Filters**: Extended Kalman Filter (EKF), Unscented Kalman Filter (UKF).
- **Features**: State initialization, innovation monitoring, observability analysis, smoothing, simulation.
- **Adapter**: Calibration model integration for measurement prediction.

### `plottings/` — Rendering Backend

- **Renderer**: plotters-based, supports PNG output with configurable DPI, size, fonts, colors.
- **Plot types**: Nyquist, Bode, time-series, calibration curves, signal analysis, health, transient, mechanism comparison, estimation.
- **Style system**: Cascading styles from shared → workflow → job → individual/combined, with named presets.

### `data_file/` — Data Ingestion

- **CHI parser**: Header-based detection of EIS vs OCPT, multi-column handling.
- **Generic CSV**: Automatic delimiter detection, header parsing, channel name/unit extraction.
- **Excel**: calamine-based `.xlsx` reading with worksheet selection.
- **Format detection**: `InputKind` discriminates CHI, generic CSV, Excel, and rejects binary/legacy `.xls`.

### `results/` — Result Structures

- **All result types are serializable** with `schema_version` for forward compatibility.
- **Key types**: `CircuitFitResult`, `TransientAnalysisReport`, `CalibrationAnalysisReport`, `StoredCalibrationModel`, `SignalAnalysisReport`, `SensorHealthAssessment`, `StateEstimationReport`.

### Configuration Modules (8 modules)

Each workflow has its own TOML config schema with:
- `schema_version` field
- Workflow-specific sections
- Default values embedded in the Rust struct (via `Default` impl)
- CLI override resolution in the corresponding runner
