# 13 — Traceability Matrix

**Identifier:** `DOC-13`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## How to Use This Matrix

For any requirement, workflow, equation, or CLI command, this matrix shows:
- **Where** it is implemented (source file/module)
- **Which equations** it uses (EQ-xxx identifiers)
- **Which CLI command** exposes it
- **Which tests** verify it
- **Which outputs** it generates
- **Current status** (Implemented / Partial / Untested)

---

## Requirements → Implementation

| Requirement | Module | CLI Command | Tests | Output | Evidence Class | Status |
|------------|--------|------------|-------|--------|----------------|--------|
| FR-001 | `electrodata-io` → `data_file/chi_file.rs` domain adapter | (implicit via all commands) | `chi_file` tests, `unified` test | Parsed EISData | Direct | ✅ |
| FR-002 | `electrodata-io` → project domain adapter | (implicit) | `chi_file` tests | Parsed ElectrochemData | Direct | ✅ |
| FR-003 | `electrodata-io` → project domain adapter | (implicit) | `chi_file` tests | Multi-series data | Direct | ✅ |
| FR-004 | `electrodata-io` → `data_file/measurement_parser.rs` domain adapter | (implicit) | `phase1_domain` tests | MeasurementParseResult | Direct | ✅ |
| FR-005 | `electrodata-io` → `data_file/excel_file.rs` compatibility wrapper | (implicit via --sheet) | `xlsx_ingestion` tests | MeasurementParseResult | Direct | ✅ |
| FR-006 | `data_file/excel_file.rs` | (implicit) | `unified_data_loading` test | Error message | Direct | ✅ |
| FR-007 | `data_file/input_kind.rs` | (implicit) | `unified_data_loading` test | Error message | Direct | ✅ |
| FR-008 | `domain/metadata.rs` | All commands with --metadata | `phase1_domain` tests | ExperimentMetadataDocument | Direct | ✅ |
| FR-009 | `impedance/` | `eis fit` | `impedance` tests, `phase0` | CircuitFitResult | Direct | ✅ |
| FR-010 | `impedance/circuit_models.rs` | `eis fit` (implicit) | `chi_file` tests | Circuit model string | Direct | ✅ |
| FR-011 | `impedance/ecm_evolution.rs` | `eis search` | `phase0` integration | Ranked candidates, plots | Direct | ✅ |
| FR-012 | `plottings/eis_plot.rs` | `plot eis` | `phase0` | PNG/SVG figures | Partial | ✅ |
| FR-013 | `plottings/chi_plot.rs` | `plot regular-plot` | `phase0` | PNG/SVG figures | Partial | ✅ |
| FR-014 | `plottings/generic_plot.rs` | `plot generic-plot` | `plot_config` tests | PNG/SVG figures | Partial | ✅ |
| FR-015 | `potentiometry/transient/` | `transient fit` | `phase2_transient` | TransientAnalysisReport | Direct | ✅ |
| FR-016 | `potentiometry/calibration/observations.rs` | `calibration extract` | `phase3_calibration` | CalibrationObservationSet | Direct | ✅ |
| FR-017 | `potentiometry/calibration/` | `calibration fit` | `phase3_calibration` | CalibrationAnalysisReport | Direct | ✅ |
| FR-018 | `potentiometry/calibration/validation.rs` | `calibration validate` | `phase3_calibration` | ValidationResult | Direct | ✅ |
| FR-019 | `potentiometry/calibration/prediction.rs` | `calibration predict` | `phase3_calibration` | CalibrationPrediction | Direct | ✅ |
| FR-020 | `signal/` | `signal characterize` | `phase5` | SignalAnalysisReport | Direct | ✅ |
| FR-021 | `health/baseline.rs` | `health baseline` | `phase5` | SensorHealthBaseline | Partial | ✅ |
| FR-022 | `health/assessment.rs` | `health assess` | `phase5` | SensorHealthAssessment | Partial | ✅ |
| FR-023 | `mechanism/` | `mechanism compare` | `phase4` | Mechanism report | Inferred | ✅ |
| FR-024 | `estimation/` | `estimate run` | `phase6` | StateEstimationReport | Direct | ✅ |
| FR-025 | `domain/provenance.rs` | All commands (implicit) | `provenance` test | AnalysisProvenance in all reports | Direct | ✅ |
| FR-026 | `impedance/reporting.rs` | `eis fit`, `eis export-fit --report` | `phase0`, `chi_file` tests | Human-readable EIS fit report | Direct | ✅ |
| FR-027 | `results/`, `runners/` | All analysis commands (except `plot`) | Integration tests + source inspection | JSON artifacts | Partial | ✅ |
| FR-028 | `runners/transient.rs`, `runners/calibration.rs` | `transient fit`, `calibration fit` | `phase2`, `phase3` | CSV feature tables | Partial | ✅ |
| FR-029 | `model/definition.rs`, `model/registry.rs`, `model/compiler.rs` | None | `model_core` | CompiledIsmModel | Direct | ✅ |
| FR-030 | `model/state.rs`, `model/parameter.rs` | None | `model_core` | Versioned ModelDefinition | Direct | ✅ |
| FR-031 | `model/graph.rs`, `model/compiler.rs` | None | `model_core` | Typed ModelError | Direct | ✅ |
| FR-032 | `model/output.rs`, `model/evidence.rs` | None | `model_core` | ObservationPrediction | Direct | ✅ |
| FR-033 | `model/` | None | Source dependency scan | None | Direct | ✅ |
| FR-034 to FR-035 | `model/builtins.rs`, `model/defaults.rs` | None | `model_builtins` | Built-in model definition | Direct | ✅ |

---

## Scientific Equations → Implementation

| Equation ID | Name | Source File | Tests | Status |
|------------|------|------------|-------|--------|
| EQ-EIS-001 | Resistor Z=R | `impedance/elements.rs` L310 | `impedance` tests | ✅ |
| EQ-EIS-002 | Capacitor Z=-j/ωC | `impedance/elements.rs` L311 | (implicit) | ✅ |
| EQ-EIS-003 | Inductor Z=jωL | `impedance/elements.rs` L318 | (implicit) | ✅ |
| EQ-EIS-004 | Warburg Z=σ(1-j)/√ω | `impedance/elements.rs` L319 | `impedance` tests | ✅ |
| EQ-EIS-005 | CPE Z=1/Q(jω)^α | `impedance/elements.rs` L328 | `cpe_matches_ideal_capacitor` | ✅ |
| EQ-EIS-006 | Wo Z=Z₀coth(√jωτ)/√jωτ | `impedance/elements.rs` L341 | (implicit) | ✅ |
| EQ-EIS-007 | Ws Z=Z₀tanh(√jωτ)/√jωτ | `impedance/elements.rs` L357 | (implicit) | ✅ |
| EQ-EIS-008 | La Z=L(jω)^α | `impedance/elements.rs` L372 | (implicit) | ✅ |
| EQ-EIS-009 | Gw Z=σ(jω)^(-α) | `impedance/elements.rs` L379 | `generalized_warburg_outperforms` | ✅ |
| EQ-EIS-010 | G Z=R_G/√(1+jωt_G) | `impedance/elements.rs` L391 | (implicit) | ✅ |
| EQ-EIS-011 | Gs finite Gerischer | `impedance/elements.rs` L398 | (implicit) | ✅ |
| EQ-EIS-012 | K Z=R/(1+jωτ_k) | `impedance/elements.rs` L411 | (implicit) | ✅ |
| EQ-EIS-013 | Zarc Z=R/(1+(jωτ)^γ) | `impedance/elements.rs` L418 | (implicit) | ✅ |
| EQ-EIS-014 | TLMQ transmission line | `impedance/elements.rs` L427 | (implicit) | ✅ |
| EQ-EIS-015 | T porous electrode | `impedance/elements.rs` L452 | (implicit) | ✅ |
| EQ-CCT-001 | Series Z=ΣZ_i | `impedance/circuits.rs` L129 | (implicit) | ✅ |
| EQ-CCT-002 | Parallel 1/Z=Σ1/Z_i | `impedance/circuits.rs` L133 | (implicit) | ✅ |
| EQ-FIT-001 | Parameter transforms | `impedance/fitting.rs` L259 | (implicit) | ✅ |
| EQ-FIT-002 | Initial guesses | `impedance/fitting.rs` L29 | `impedance` tests | ✅ |
| EQ-FIT-003 | Residual normalization d=max(|Z|,1) | `impedance/lib.rs` L214, `impedance/fitting.rs` L341 | `impedance` tests | ✅ |
| EQ-FIT-004 | LM objective Σ[(ΔRe/d)^2+(ΔIm/d)^2] | `impedance/fitting.rs` L341, L513 | `impedance` tests | ✅ |
| EQ-TR-001 | Single exponential | `potentiometry/transient/models.rs` L129 | `phase2` | ✅ |
| EQ-TR-002 | Double exponential | `potentiometry/transient/models.rs` L143 | `phase2` | ✅ |
| EQ-TR-003 | Double with drift | `potentiometry/transient/models.rs` L165 | `phase2` | ✅ |
| EQ-TR-004 | Stretched exponential | `potentiometry/transient/models.rs` L189 | `phase2` | ✅ |
| EQ-CAL-001 | Nernst equation | `potentiometry/calibration/nernst.rs` | `phase3`, unit tests | ✅ |
| EQ-CAL-002 | Nicolsky-Eisenman | `potentiometry/calibration/nicolsky_eisenman.rs` | (implicit) | ✅ |
| EQ-CAL-003 | Activity models | `potentiometry/calibration/activity.rs` | unit tests | ✅ |
| EQ-CAL-004 | Conductivity-empirical calibration equation | `potentiometry/calibration/fitting.rs` | `phase3` | ✅ |
| EQ-ECM-001 | Candidate ranking objective (BIC default) | `impedance/ecm_scoring.rs`, `impedance/ecm_evolution.rs` | `phase0` | ✅ |
| EQ-ECM-002 | Evolution defaults (population/generation/mutation/etc.) | `impedance/ecm_evolution.rs`, `search_config.rs` | `phase0`, config tests | ✅ |
| EQ-ECM-003 | Circuit mutation operators | `impedance/ecm_evolution.rs` | `phase0` | ✅ |
| EQ-SIG-001 | PSD (Welch) | `signal/psd.rs` | `phase5` | ✅ |
| EQ-SIG-002 | Allan variance | `signal/allan.rs` | `phase5` | ✅ |
| EQ-SIG-003 | Linear regression | `regression_mod.rs` | unit tests | ✅ |
| EQ-EST-001 | EKF prediction | `estimation/ekf.rs` | `phase6` | ✅ |
| EQ-EST-002 | EKF update | `estimation/ekf.rs` | `phase6` | ✅ |
| EQ-EST-003 | UKF | `estimation/ukf.rs` | `phase6` | ✅ |
| EQ-ISM-001 | ISM state-space interface | `model/` | `model_core` | ✅ (contract only) |
| EQ-ISM-002 | ISM contribution/residual decomposition | `model/output.rs` | `model_core` | ✅ |
| EQ-ISM-003 | Reduced-order built-in adapters | `model/builtins.rs` | `model_builtins` | ✅ |
| EQ-ISM-004 | Runtime covariance propagation and semantic-zero contract | `model/compiler.rs` | `model_core`, `model_builtins` | ✅ |

---

## Workflow IDs (DOC-06) → Implementation

| Workflow ID | CLI Command | Runner | Scientific Module | Status |
|-------------|-------------|--------|-------------------|--------|
| WF-001 | `plot eis` | `runners/plot.rs` | `plottings/eis_plot.rs` | ✅ |
| WF-002 | `plot regular-plot` | `runners/plot.rs` | `plottings/chi_plot.rs` | ✅ |
| WF-003 | `plot generic-plot` | `runners/plot.rs` | `plottings/generic_plot.rs` | ✅ |
| WF-004 | `eis fit` | `runners/fit.rs` | `impedance/` | ✅ |
| WF-005 | `eis search` | `runners/search.rs` | `impedance/ecm_evolution.rs` | ✅ |
| WF-006 | `eis export-fit` | `runners/fit.rs` | `impedance/` | ✅ |
| WF-007 | `transient fit` | `runners/transient.rs` | `potentiometry/transient/` | ✅ |
| WF-008 | `calibration extract` | `runners/calibration.rs` | `potentiometry/calibration/observations.rs` | ✅ |
| WF-009 | `calibration fit` | `runners/calibration.rs` | `potentiometry/calibration/` | ✅ |
| WF-010 | `calibration validate` | `runners/calibration.rs` | `potentiometry/calibration/validation.rs` | ✅ |
| WF-011 | `calibration predict` | `runners/calibration.rs` | `potentiometry/calibration/prediction.rs` | ✅ |
| WF-012 | `mechanism compare` | `runners/mechanism.rs` | `mechanism/` | ✅ |
| WF-013 | `mechanism trend` | `runners/mechanism.rs` | `mechanism/trend.rs` | ✅ |
| WF-014 | `signal characterize` | `runners/signal.rs` | `signal/` | ✅ |
| WF-015 | `signal compare` | `runners/signal.rs` | `signal/comparison.rs` | ✅ |
| WF-016 | `signal residuals` | `runners/signal.rs` | `signal/residuals.rs` | ✅ |
| WF-017 | `health baseline` | `runners/health.rs` | `health/baseline.rs` | ✅ |
| WF-018 | `health assess` | `runners/health.rs` | `health/assessment.rs` | ✅ |
| WF-019 | `health trend` | `runners/health.rs` | `health/trend.rs` | ✅ |
| WF-020 | `estimate run` | `runners/estimation.rs` | `estimation/` | ✅ |
| WF-021 | `estimate validate` | `runners/estimation.rs` | `estimation/validation.rs` | ✅ |
| WF-022 | `estimate simulate` | `runners/estimation.rs` | `estimation/simulation.rs` | ✅ |
| WF-023 | `estimate compare` | `runners/estimation.rs` | `estimation/comparison.rs` | ✅ — shared canonical ingestion validation and evidence |

---

## CLI Commands → Implementation

| Command | Runner | Main Dispatch | Tests |
|---------|--------|--------------|-------|
| `plot` | `runners/plot.rs` | `main.rs` L70-87 | `phase0`, `plot_config` tests |
| `eis fit` | `runners/fit.rs` | `main.rs` L109-125 | `phase0`, `cli` tests |
| `eis search` | `runners/search.rs` | `main.rs` L88-108 | `phase0`, `cli` tests |
| `eis export-fit` | `runners/fit.rs` | `main.rs` L126-140 | (inferred) |
| `transient fit` | `runners/transient.rs` | `main.rs` L141-178 | `phase2` |
| `calibration extract` | `runners/calibration.rs` | `main.rs` L179-205 | `phase3` |
| `calibration fit` | `runners/calibration.rs` | `main.rs` L206-230 | `phase3` |
| `calibration validate` | `runners/calibration.rs` | `main.rs` L231-241 | `phase3` |
| `calibration predict` | `runners/calibration.rs` | `main.rs` L243-265 | `phase3` |
| `mechanism compare` | `runners/mechanism.rs` | `main.rs` L266-290 | `phase4` |
| `mechanism trend` | `runners/mechanism.rs` | `main.rs` L291-309 | (inferred) |
| `mechanism report` | `runners/mechanism.rs` | `main.rs` L310-319 | (inferred) |
| `signal characterize` | `runners/signal.rs` | `main.rs` L320-337 | `phase5` |
| `signal compare` | `runners/signal.rs` | `main.rs` L338-349 | `phase5` |
| `signal residuals` | `runners/signal.rs` | `main.rs` L350-365 | (inferred) |
| `health baseline` | `runners/health.rs` | `main.rs` L366-377 | `phase5` |
| `health assess` | `runners/health.rs` | `main.rs` L378-401 | `phase5` |
| `health trend` | `runners/health.rs` | `main.rs` L402-415 | (inferred) |
| `health report` | `runners/health.rs` | `main.rs` L416-418 | (inferred) |
| `estimate run` | `runners/estimation.rs` | `main.rs` L419-467 | `phase6` |
| `estimate validate` | `runners/estimation.rs` | `main.rs` L468-481 | (inferred) |
| `estimate simulate` | `runners/estimation.rs` | `main.rs` L482-495 | (inferred) |
| `estimate compare` | `runners/estimation.rs` | `main.rs` L496-536 | (inferred) |
| `estimate report` | `runners/estimation.rs` | `main.rs` L537-546 | (inferred) |
| FR-036 / SCI-013 | `mechanism/model_mapping.rs` | `phase05_model_health` | Explicit stable-ID mapping and weak-evidence guard |
| FR-037 | `health/features.rs`, `health/assessment.rs` | `phase05_model_health` | Context-aware/model-derived health features |
| FR-038 | `cli.rs`, `runners/model.rs`, `results/model.rs` | `phase06_model_workflow` | Model workflow and finite artifact exports |
| FR-039 | `model_validation.rs`, `results/validation.rs` | `phase07_validation` | Reproducible experimental validation evidence |
| ART-001 / ART-002 | `domain/artifact.rs`, `results/artifact_contracts.rs` | all cross-workflow runners | `artifact_contract` kind/version/migration tests |
| ISM-001 / ISM-002 | `model/compiler.rs`, `model/builtins.rs`, `model/output.rs` | static model compiler | `model_core`, `model_builtins` |
| EST-ISM-001 | `estimation/ism_adapter.rs`, `estimation/model.rs`, `estimation/model_output.rs` | EKF and UKF compatibility facade | `phase6_estimation` compiled-equivalence and contribution tests |
| VAL-ISM-001 | `model_validation.rs`, `results/validation.rs` | `runners/model_validation.rs` | `phase07_validation` missing-evidence assertions |
| EST-EQ-001 | `estimation/model_output.rs`, `estimation_config.rs` | EKF and UKF timestamp outputs | equilibrium unit tests and `phase6_estimation` |
| IO-CANON-001 | `data_file/electrodata_domain_adapter.rs` | typed `Dataset` conversion | `canonical_input_boundary` |
| IO-CANON-002 | `docs/io_migration_validation_archive.md` | completed independent parity evidence | canonical boundary, XLSX, EIS semantics, and scientific workflow regressions |
| ISM-003 / ISM-004 / ISM-005 | `model/component.rs`, `model/state.rs`, `model/parameter.rs`, `model/evidence.rs`, `model/equilibrium_recognition.rs` | ADR-0002 scientific-contract boundary | `model_contracts`, `model_core` | Contract-only; no high-fidelity transport claim |
| ISM-006 | `model/component.rs`, `model/compiler.rs`, `model/output.rs` | typed composition, reconstruction, uncertainty status | `model_core`, `model_builtins` | Schema-v3 |
| ISM-007 / FR-040 / FR-041 | `model/component.rs`, `model/compiler.rs`, `model/builtins.rs` | stable-ID derivative coverage and complete covariance propagation | built-in/model-core uncertainty regressions | Schema-v3 |
| ISM-008 / FR-042 | `model/definition.rs`, `model/state.rs`, `model/parameter.rs`, `estimation/ism_adapter.rs` | fitted/estimated validation and legacy enrichment | model-core and phase6 estimation regressions | Schema-v3 |
| ISM-009 | `model/compiler.rs`, `model/error.rs`, `model/state.rs`, `model/parameter.rs` | declared uncertainty class and covariance-contract enforcement | zero-row, state, derivative, and matrix validation regressions | Schema-v3 covariance consistency |
