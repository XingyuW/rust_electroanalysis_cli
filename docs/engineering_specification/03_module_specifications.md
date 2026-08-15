# 03 — Module Specifications

## Reduced-order ISM components V1

V1 adds activity, equilibrium, first-order dynamic, reference, covariate, and
observation-variance adapters under the existing model contracts. Details are
in [`docs/model/reduced_order_ism_v1.md`](../model/reduced_order_ism_v1.md).

## `model`

The model module exposes compiled state/parameter lookup and component-local
index slices, discrete transitions, optional continuous derivatives, process
Jacobians, observation reconstruction, covariance propagation, validity, and
declarative identifiability metadata. `CompiledModelSummary` is serializable;
trait-object component implementations are intentionally not serialized.

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
| `src/domain/artifact.rs` | VersionedArtifact contract, artifact-kind/schema validation, legacy migration, finite JSON guards | ✅ |
| `src/data_file/lib.rs` | Data ingestion module root | ✅ |
| `src/data_file/chi_file.rs` | Canonical Dataset → EIS/plot domain adapter | ✅ |
| `src/data_file/data_op.rs` | PlotData container, IntoPlotData trait | ✅ |
| `src/data_file/excel_file.rs` | Compatibility wrapper over canonical XLSX ingestion | ✅ |
| `src/data_file/input_kind.rs` | Legacy/reference batch classification only | ✅ |
| `src/data_file/measurement_adapter.rs` | Conversion from domain measurements to PlotData | ✅ |
| `src/data_file/measurement_parser.rs` | Canonical file-to-measurement domain adapter; deprecated text compatibility API | ✅ |
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
| `src/results/model.rs` | Versioned compiled-model artifact schema | ✅ |
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
| `src/model_config.rs` | ISM model TOML configuration wrapper | ✅ |
| `src/model/mod.rs` | ISM core public façade and stable re-exports | ✅ |
| `src/model/error.rs` | Typed ISM model errors | ✅ |
| `src/model/definition.rs` | Versioned model definition schema | ✅ |
| `src/model/component.rs` | Component descriptors, roles, and component trait | ✅ |
| `src/model/registry.rs` | Immutable static component-factory registry | ✅ |
| `src/model/graph.rs` | Deterministic dependency ordering and cycle detection | ✅ |
| `src/model/compiler.rs` | Definition validation, index resolution, and compiled-model execution | ✅ |
| `src/model/parameter.rs` | Parameter metadata, bounds, and compiled indices | ✅ |
| `src/model/state.rs` | State metadata, bounds, and compiled indices | ✅ |
| `src/model/input.rs` | Input schemas and existing-unit adapter | ✅ |
| `src/model/output.rs` | Contributions, prediction, and explicit residual status | ✅ |
| `src/model/validity.rs` | Validity-domain and validity-report contracts | ✅ |
| `src/model/identifiability.rs` | Non-claiming identifiability-report contract | ✅ |
| `src/model/evidence.rs` | Mechanism-evidence requirement contract | ✅ |
| `src/model/equilibrium_recognition.rs` | Evidence-preserving equilibrium-assessment contract | ✅ |
| `src/model/builtins.rs` | Static reduced-order equilibrium/activity/transport/transduction/disturbance adapters | ✅ |
| `src/model/defaults.rs` | Default reduced-order ISM definition | ✅ |
| `src/plot_runner.rs` | (Legacy) Plot orchestration adapter | ✅ |
| `src/search_runner.rs` | (Legacy) Search orchestration adapter | ✅ |
| `src/model/` | **Planned only** unified ISM scientific core; no source module exists in Phase 01 | 📋 |

**Coverage check:** 159/159 Rust source paths under `src/**/*.rs` are mapped above.

---

## Key Module Specifications

### `domain/` — Shared Application-Domain Contracts

- **Purpose**: Type-safe shared contracts for measurements, experiments, errors, and provenance.
- **Public types**: `ElectrochemicalExperiment`, `MultiChannelMeasurement`, `MeasurementChannel`, `AnalysisProvenance`, `VersionedArtifact`, `ArtifactKind`, `ArtifactError`, `ExperimentEvent`, typed error enums.
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

- **Canonical boundary**: `electrodata-io` owns physical/raw data detection, parsing, worksheet handling, scientific input roles, recovery diagnostics, and raw input errors.
- **Domain adapter**: this project owns domain conversion, scientific calculations, modeling, estimation, mechanism, health, plotting, reporting, and analysis artifacts. The completed legacy parser/parity gate is archived; retained `parse_measurement_text`, Excel wrappers, and `InputKind` are public compatibility surfaces only.

### `results/` — Result Structures

- **Cross-workflow result types implement `VersionedArtifact`**. Serialized files carry both `schema_version` and `artifact_kind`; unsupported kind/schema combinations fail before serde payload deserialization.
- **Key types**: `CircuitFitResult`, `TransientAnalysisReport`, `CalibrationAnalysisReport`, `StoredCalibrationModel`, `SignalAnalysisReport`, `SensorHealthAssessment`, `StateEstimationReport`.

### `model/` — Unified ISM Model Core

- **Purpose**: Versioned, extensible model definitions and deterministic graph
  compilation without implementing scientific equations in this phase.
- **Public contracts**: `IsmModel`, `IsmComponent`, `ComponentDescriptor`,
  `ComponentRole`, `StateSpec`, `ParameterSpec`, `ModelDefinition`,
  `CompiledIsmModel`, `ModelInput`, `ModelState`, `ComponentContribution`,
  `ValidityReport`, `IdentifiabilityReport`, `EvidenceRequirement`, and
  `EquilibriumAssessment`.
- **Dependencies**: only core Rust/serde/thiserror and a narrow adapter to the
  existing potentiometry unit taxonomy; never CLI, runners, plotting, health,
  mechanism, or estimation.
- **Invariant**: components cannot own an unexplained residual; state/parameter
  positions preserve definition order; factories are static and keyed by kind.

### Configuration Modules (8 modules)

Each workflow has its own TOML config schema with:
- `schema_version` field
- Workflow-specific sections
- Default values embedded in the Rust struct (via `Default` impl)
- CLI override resolution in the corresponding runner

### Phase 05 Adapter Contracts

`mechanism/model_mapping.rs` exposes explicit EIS, transient, calibration, and
signal mappings to model priors. Every mapping carries a stable component ID
and explicit source path; an unavailable path produces no assignment.
`health/features.rs` creates context-partitioned transient and model-derived
health features. `health/rules.rs` retains contradictory evidence, and
`health/assessment.rs` incorporates findings and baseline deviations into
domain status. These adapters do not introduce a new CLI command.

### Phase 06 Model Runner

`runners/model.rs` resolves a versioned definition, compiles the static model
registry, performs deterministic evaluation, and exports contributions,
states, validity, equilibrium evidence, and a report. It rejects unsupported
analysis artifact schema/kind values and validates finite values before JSON.

### Phase 07 Validation Infrastructure

`results/validation.rs` declares a versioned manifest spanning stable standards,
steps, environmental variation, interferents, flow, reference, construction,
fouling, aging, and paired EIS/transient studies. `model_validation.rs` records
missing state/profile-likelihood evidence instead of fabricating validation.

### Platform and Estimation Adapters

- `domain/artifact.rs`: `VersionedArtifact`, typed kind/schema errors, and the
  generic read/write boundary for cross-workflow JSON.
- `results/artifact_contracts.rs`: explicit artifact contracts and permitted
  legacy versions for every exported cross-workflow result.
- `estimation/ism_adapter.rs`: compiles the legacy state/calibration contract
  into stable component IDs without moving legacy equations into the core.
- `estimation/model_output.rs`: common contribution categories, visible
  residual, and conservative equilibrium assessment shared by EKF and UKF.

Malformed built-in descriptors, parameter/state arity errors, and missing
runtime inputs return `ModelError`; a factory may not index user-controlled
descriptor content before validating its shape.

## `data_file/electrodata_domain_adapter.rs`

This module owns no raw parsing. It applies the explicit project compatibility `ReadOptions`, calls `electrodata_io::read_with_options`, converts typed time-series/EIS views into project types, preserves source channel names, units, ordering and null cells, and passes `electrodata_io::Error` transparently through `DataParsingError`.

### ISM scientific-contract additions (ADR-0002)

`ComponentId`, `InterpretationStatus`, `EvidenceAssessment`, `ModelPrediction`,
`ModelWarning`, and `EquilibriumStatus` make model-form uncertainty and
interpretation boundaries explicit. `StateSpec` and `ParameterSpec` retain
scientific metadata beyond bounds and units, including transformations,
initialization/value source, equation version, observability/identifiability,
validity, and uncertainty representation. These are contracts only: they do
not introduce a new numerical transport equation.

`component.rs` owns closed output semantics and canonical external role;
`output.rs` records categorized potential/variance and uncertainty status;
`compiler.rs` enforces output compatibility and applies available first-order
covariance propagation.

In schema v3, `component.rs` also owns typed state/parameter Jacobian results.
Covered values are paired with stable IDs; `Complete`, `Partial`,
`Unavailable`, and `NotApplicable` coverage cannot be inferred from numeric
zeros. `compiler.rs` validates declared direct-observation dependencies,
coverage sets, finite values, numerical-step policy, and local-to-global ID
mapping before applying caller-supplied full runtime covariance. `defaults.rs`
classifies values by the current artifact (fixed, externally supplied, or
fitted), rather than by whether they might be fitted in a future workflow.

`validity.rs` supplies explicit calibrated-domain status and enforcement;
`evidence.rs` represents Present/NotApplicable/Missing evidence;
`identifiability.rs` holds structured component requirements; and
`results/model.rs` recursively validates numeric report leaves before JSON.

`compiler.rs` normalizes metadata domains and typed constraints together,
records declaration provenance, sorts deterministically, and returns typed
conflicts rather than silently composing same-subject constraints. `validity.rs`
retains each observed value, interval, source, enforcement, and warning; its
aggregate status is independent of fatality.
