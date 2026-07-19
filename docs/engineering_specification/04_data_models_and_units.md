# 04 — Data Models, Formats & Units

**Identifier:** `DOC-04`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## 1. Core Data Structures

### 1.1 Measurement Types

**`MeasurementChannel`** (`src/domain/measurement.rs`)
- `name: String` — Channel name (e.g., "potential", "E1")
- `unit: String` — Unit string (e.g., "V", "C", "mol/L")
- `values: Vec<Option<f64>>` — Per-sample values; `None` = missing
- `variance: Option<Vec<Option<f64>>>` — Optional per-sample variance
- `sensor_id: Option<String>`, `analyte_id: Option<String>`
- `metadata: Option<ChannelMetadata>` (alias for `BTreeMap<String, String>`)

**`MultiChannelMeasurement`** (`src/domain/measurement.rs`)
- `time: Vec<f64>` — Shared time axis
- `channels: Vec<MeasurementChannel>` — Named signal channels
- **Channel lookup**: Supports `"name"`, `"name/unit"`, and `"name [unit]"` formats

### 1.2 Experiment Types

**`ElectrochemicalExperiment`** (`src/domain/experiment.rs`)
- `experiment_id: String`
- `sensor_metadata: SensorMetadata` — Sensor identity and type
- `reference_metadata: Option<ReferenceMetadata>` — Reference electrode info
- `measurement_data: MultiChannelMeasurement`
- `environmental_data: Vec<EnvironmentalSeries>` — Temperature, flow, etc.
- `events: Vec<ExperimentEvent>` — Sorted by timestamp
- `sample_matrix: String`
- `provenance: AnalysisProvenance`

**`ExperimentEvent`** — `{ timestamp: f64, kind: ExperimentEventKind, value: Option<f64>, unit: Option<String>, analyte: Option<String>, annotation: Option<String> }`  
**`ExperimentEventKind`** — Enum: `ConcentrationStep`, `FlowChange`, `TemperatureChange`, `IonicStrengthChange`, `InterferentAddition`, `FlushStart`, `ReadingStart`, `FlushEnd`, `ManualAnnotation`

**`SensorMetadata`** — `{ sensor_id, name, sensor_type, analyte, manufacturer, model, metadata }`  
**`ReferenceMetadata`** — `{ reference_id, electrode_type, manufacturer, model, potential, potential_unit, metadata }`  
**`EnvironmentalSeries`** — `{ name, unit, time: Vec<f64>, values: Vec<Option<f64>>, metadata }`

### 1.3 Provenance

**`AnalysisProvenance`** (`src/domain/provenance.rs`)
- `software_version: String` — From `CARGO_PKG_VERSION`
- `input_path: PathBuf`
- `input_sha256: String` — Hex-encoded SHA-256
- `configuration_path: Option<PathBuf>`
- `configuration_sha256: Option<String>`
- `generation_timestamp: u64` — Unix epoch seconds
- `git_commit: Option<String>` — From `GIT_COMMIT` env at build time

## 2. Result Types

### CircuitFitResult (`src/results/mod.rs`)
- `fitted_parameters: Vec<f64>` — Physical-space parameter values
- `parameter_names: Vec<String>`, `parameter_units: Vec<String>`
- `fitted_z_re: Vec<f64>`, `fitted_z_im: Vec<f64>`
- `fitted_magnitude: Vec<f64>`, `fitted_phase: Vec<f64>` (degrees)

### TransientAnalysisReport (`src/results/transient.rs`)
- Schema version 1, experiment/channel identity, parse diagnostics, full configuration clone
- `events: Vec<TransientEventResult>` — Per-event results with candidate fits, selection, warnings

### CalibrationAnalysisReport (`src/results/calibration.rs`)
- Schema version 1, calibration ID, analyte, ion charge
- `candidate_models: Vec<CalibrationModelResult>`, `selected_model: Option<CalibrationModelKind>`
- Optional hysteresis, validation, provenance, warnings

### StoredCalibrationModel (`src/results/calibration.rs`)
- Portable model for prediction: schema, analyte, charge, model kind, activity model, parameters, selectivity coefficients, valid domain, training statistics, configuration clone, provenance

### SignalAnalysisReport (`src/results/signal.rs`)
- Schema version 1, channel identity, window summary, sampling analysis, descriptive statistics, optional PSD/Allan/drift/spike/correlation/residual analyses

### SensorHealthAssessment (`src/results/health.rs`)
- Baseline comparison, feature scores, rule-based evidence, health classification

### StateEstimationReport (`src/results/estimation.rs`)
- Timestamped state estimates with uncertainty, innovation sequence, filter diagnostics

## 3. File Formats

### 3.1 Input Formats

| Format | Detection | Parser |
|--------|-----------|--------|
| CHI EIS CSV | Header contains "A.C. Impedance" or "Freq/Hz" column | `chi_file.rs` → `EISData` |
| CHI OCPT CSV | Header contains "Technique" metadata or time/potential columns | `chi_file.rs` → `ElectrochemData` |
| Generic sensor CSV | CSV with header row containing time column + value columns | `measurement_parser.rs` → `MeasurementParseResult` |
| Excel .xlsx | File extension `.xlsx` | `excel_file.rs` (calamine) → `MeasurementParseResult` |
| Experiment metadata | TOML file with `experiment_id`, `sensor`, `events` | `metadata.rs` → `ExperimentMetadataDocument` |

### 3.2 Output Formats

| Format | Extension | Content |
|--------|-----------|---------|
| JSON result | `.json` | Schema-versioned analysis reports |
| CSV table | `.csv` | Feature tables, model comparisons |
| TXT report | `.txt` | Human-readable text summaries |
| PNG figure | `.png` | Publication-quality plots |

### 3.3 Configuration and Schema-Version Formats

All configuration files are TOML, but schema versions are **module-specific** (not universally `1`).

#### Configuration schema versions and compatibility behavior

| File | Module | Current schema | Compatibility behavior |
|------|--------|----------------|------------------------|
| `config/plotting.toml` | `plot_config.rs` | 1 (optional field) | Missing field is accepted; mismatched version emits warning |
| `config/analysis.toml` | `search_config.rs` | 1 (optional field) | Missing field is accepted; mismatched version emits warning |
| `config/parsing.toml` | `impedance/circuit_models.rs` | N/A | Resolver config currently has no explicit `schema_version` field |
| `config/transient.toml` | `transient_config.rs` | 1 | Mismatched version emits warning after validation |
| `config/calibration.toml` | `calibration_config.rs` | 1 | Mismatched version emits warning after validation |
| `config/mechanism.toml` | `mechanism_config.rs` | 1 | Mismatched version is rejected (`unsupported mechanism config schema version`) |
| `config/signal.toml` | `signal_config.rs` | 1 | Mismatched version is rejected (`unsupported signal configuration schema`) |
| `config/health.toml` | `health_config.rs` | 1 | Mismatched version is rejected (`unsupported health configuration schema`) |
| `config/estimation.toml` | `estimation_config.rs` | 3 | `<3` auto-migrates to 3 with warnings; `>3` is unsupported and rejected during validation |
| `config/app.toml` | `workspace.rs` | 1 | Mismatched version auto-migrates to 1 with warning and rewrite |

#### Result/artifact schema versions (currently emitted)

| Output artifact | Module | Emitted `schema_version` |
|-----------------|--------|--------------------------|
| EIS fit artifact (`EisFitArtifact`) | `results/eis.rs` | 1 |
| Transient analysis report | `potentiometry/transient/mod.rs` | 1 |
| Calibration observations/report/stored model | `potentiometry/calibration/mod.rs`, `potentiometry/calibration/observations.rs` | 1 |
| Signal analysis report | `signal/mod.rs` | 1 |
| Mechanism comparison/trend reports | `runners/mechanism.rs`, `results/mechanism.rs` | 1 |
| Health baseline | `health/baseline.rs` | 2 |
| Health assessment/trend reports | `health/assessment.rs`, `health/trend.rs` | 1 |
| Estimation run/compare/simulate outputs | `estimation/mod.rs`, `estimation/comparison.rs`, `estimation/simulation.rs` | 2 |

## 4. Unit Registry

### Core Physical Constants

| Constant | Value | Location |
|----------|-------|----------|
| Gas constant R | 8.31446261815324 J/(mol·K) | `potentiometry/calibration/nernst.rs` |
| Faraday constant F | 96485.33212 C/mol | `potentiometry/calibration/nernst.rs` |
| Celsius-to-Kelvin offset | +273.15 | `potentiometry/units.rs` |

### QuantityUnit Enum (`src/potentiometry/units.rs`)

| Quantity | Symbol | Internal Unit | Accepted Input Units | Conversion |
|----------|--------|--------------|----------------------|------------|
| Concentration | c | mol/L | mol/L, mmol/L, µmol/L, mg/L, g/L | mmol/L → ×1e-3; µmol/L → ×1e-6; mg/L → ÷(molar_mass) ÷1000; g/L → ÷(molar_mass) |
| Activity | a | dimensionless | activity, dimensionless, unitless, 1 | Direct |
| Potential | E | V | V, mV, µV | mV → ×0.001; µV → ×1e-6 |
| Temperature | T | K | K, °C | °C → +273.15 |
| Conductivity | κ | S/m | S/m, S/cm, mS/cm, µS/cm | S/cm → ×100; mS/cm → ×0.1; µS/cm → ×1e-4 |

### Impedance Element Parameter Units

| Element | Parameter | Unit |
|---------|-----------|------|
| R | R | Ohm |
| C | C | F |
| L | L | H |
| W | σ | Ohm·s^(-1/2) |
| CPE | Q, α | Ohm^(-1)·s^α, dimensionless |
| Wo, Ws | Z₀, τ | Ohm, s |
| La | L, α | H·s^(α-1), dimensionless |
| Gw | σ, α | Ohm·s^α, dimensionless |
| G | R_G, t_G | Ohm, s |
| Gs | R_G, t_G, φ | Ohm, s, dimensionless |
| K | R, τ_k | Ohm, s |
| Zarc | R, τ_k, γ | Ohm, s, dimensionless |
| TLMQ | R_ion, Qs, γ | Ohm, Ohm^(-1)·s^γ, dimensionless |
| T | A, B, a, b | Ohm, Ohm, dimensionless, s |

### Transient Model Parameter Units

| Model | Parameters | Units |
|-------|-----------|-------|
| Single | E∞, A, τ | V, V, s |
| Double | E∞, A_fast, A_slow, τ_fast, τ_slow | V, V, V, s, s |
| DoubleDrift | E∞, A_fast, A_slow, τ_fast, τ_slow, drift | V, V, V, s, s, V/s |
| Stretched | E∞, A, τ, β | V, V, s, dimensionless |

## 5. Serialization and Precision

- **JSON**: All result types serialize via `serde_json` with default precision (f64 → JSON number)
- **CSV**: Written via the `csv` crate with default formatting
- **TOML**: Configuration via the `toml` crate
- **No explicit rounding behaviour** is defined; IEEE 754 f64 precision is used throughout

## 6. Missing-Value Behaviour

- **Measurement channels**: `None` in `Vec<Option<f64>>` represents missing values
- **Missing value count** is tracked in `ParseDiagnostics`
- **Environmental series**: `Option<f64>` for potentially missing values
- **NaN/Infinity**: Explicitly rejected during validation (`is_finite()` checks)
- **Negative variance**: Rejected during channel validation

## 7. Flagged Unit Ambiguities

| Location | Issue | Severity |
|----------|-------|----------|
| `elements.rs` CPE unit `"Ohm^-1 s^alpha"` | α present in unit specifier makes units dimensionally dependent on the fitted parameter | Low (standard notation) |
| `elements.rs` La unit `"H s^(alpha-1)"` | Same issue — unit depends on fitted α | Low |
| Various config fields | Units sometimes inferred from context rather than stated explicitly | Medium (documentation gap) |

## 8. Configuration Precedence

For all workflows:
1. CLI argument overrides (highest priority)
2. User-specified config file (`--config` / `--plot-config` / etc.)
3. Workspace default config file (`config/*.toml`)
4. Hardcoded defaults in Rust struct `Default` impls (lowest priority)

For plotting styles specifically:
1. Per-job `style` block
2. Per-job `individual_style` / `combined_style` blocks
3. Named `style_preset`
4. `[shared.style]` / `[shared.individual_style]` / `[shared.combined_style]`
5. Global `[render]` settings
