# 00 — Project Overview

**Identifier:** `DOC-00`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## 1. Project Purpose

`rust_electroanalysis_cli` is a command-line tool for **electrochemical data analysis**. It processes raw instrument data (CH Instruments potentiostat exports, generic CSV, Excel `.xlsx`), performs scientific analysis including electrochemical impedance spectroscopy (EIS) fitting, equivalent-circuit model search, potentiometric transient analysis, calibration curve fitting, signal quality characterization, sensor health assessment, and latent-state estimation, and produces publication-quality figures, JSON/CSV reports, and human-readable summaries.

## 2. Scientific and Engineering Problems Addressed

1. **EIS analysis** — Fit equivalent-circuit models (resistor, capacitor, CPE, Warburg, Gerischer, Zarc, TLMQ, etc.) to frequency-domain impedance data using Levenberg-Marquardt nonlinear least squares.
2. **ECM search** — Automatically discover suitable equivalent-circuit models using genetic-programming-based evolution.
3. **Potentiometric transient analysis** — Fit time-domain exponential models (single, double, double-with-drift, stretched-exponential) to sensor potential transients after concentration steps.
4. **Calibration curve fitting** — Fit Nernst, Nicolsky-Eisenman, and conductivity-empirical models to equilibrium potentiometric calibration observations, with activity-coefficient models and cross-validation.
5. **Signal quality characterization** — Compute PSD, Allan variance, drift, spike detection, descriptive statistics, and channel correlation from time-series measurements.
6. **Sensor health assessment** — Construct baselines from multi-sensor data and assess individual sensor health using rule-based evidence synthesis.
7. **Mechanism comparison** — Compare EIS-derived characteristic timescales with transient-derived time constants.
8. **State estimation** — Estimate latent activity/sensor-response states using Extended and Unscented Kalman Filters.
9. **Publication-ready plotting** — Generate Nyquist, Bode, time-series, calibration-curve, and generic scatter/regression plots at configurable DPI and layout.

## 3. Intended Users

- Electrochemists and analytical chemists performing sensor characterization
- Researchers analysing potentiometric sensor arrays and ISE (ion-selective electrode) data
- Scientists who need reproducible, scriptable analysis pipelines
- Users who have CH Instruments potentiostat data files or generic time-series CSV data

## 4. Supported Use Cases

| Use Case | CLI Command(s) | Output |
|----------|---------------|--------|
| Plot EIS Nyquist/Bode with fitted curves | `plot eis` | PNG figures |
| Plot time-series sensor data | `plot regular-plot` | PNG figures |
| Generate generic scatter/regression plots | `plot generic-plot` | PNG figures |
| Fit ECM to EIS data | `eis fit` | Fit report (stdout/file), JSON artifact |
| Search for optimal ECM | `eis search` | Ranked candidate list, ECM plots |
| Export durable fit artifact | `eis export-fit` | JSON artifact |
| Fit transient response models | `transient fit` | JSON report, CSV features, figures |
| Extract calibration observations | `calibration extract` | JSON observation set |
| Fit calibration models | `calibration fit` | JSON model, CSV summaries, figures |
| Validate calibration model | `calibration validate` | Validation metrics |
| Predict from calibration model | `calibration predict` | Predicted activity/concentration |
| Compare EIS and transient timescales | `mechanism compare` | JSON report |
| Trend mechanism results | `mechanism trend` | JSON report |
| Characterize signal quality | `signal characterize` | JSON report with PSD, Allan, drift |
| Compare signal characteristics | `signal compare` | JSON report |
| Analyze fit residuals | `signal residuals` | JSON report |
| Build health baseline | `health baseline` | JSON baseline |
| Assess sensor health | `health assess` | JSON assessment |
| Trend health assessments | `health trend` | JSON report |
| Run state estimation | `estimate run` | JSON report |
| Validate state estimates | `estimate validate` | Validation report |
| Simulate estimation scenarios | `estimate simulate` | Simulation output |
| Compare estimation filters | `estimate compare` | Comparison report |

## 5. Current Capabilities (Verified)

- **CHI file parsing**: OCPT (open-circuit potential vs time), multi-column OCPT, and EIS files with automatic header detection.
- **15 circuit elements**: R, C, L, W (Warburg), CPE, Wo, Ws, La, Gw, G, Gs, K, Zarc, TLMQ, T (porous electrode).
- **Circuit string parser**: Nom-based parser for expressions like `R0-p(CPE1,R1)`.
- **ECM genetic search**: Configurable population size, generations, mutation rate.
- **4 transient models**: Single-exponential, Double-exponential, Double-with-drift, Stretched-exponential.
- **3 calibration models**: Nernst, Nicolsky-Eisenman, Conductivity-empirical.
- **5 activity models**: Ideal, Davies, Extended Debye-Hückel, Conductivity-empirical, User-provided.
- **5 signal analysis domains**: Sampling statistics, Descriptive statistics, PSD (Welch method via FFT), Allan variance, Drift analysis.
- **2 Kalman filter variants**: EKF and UKF.
- **Provence tracking**: SHA-256 hashing of input and configuration files with timestamps.
- **Configurable plotting**: DPI, figure size, font size, line width, marker size, colors, palettes, axis transforms, scientific notation, regression overlays.
- **Workspace management**: Auto-creation of `config/`, `data/`, `output/`, `logs/` directories; legacy config migration warnings.

## 6. System Boundaries

- **Inputs**: CSV/TSV files (CHI format or generic), Excel `.xlsx` files, TOML configuration files, JSON result files (for cross-workflow integration).
- **Outputs**: PNG figures, JSON reports, CSV feature tables, TXT human-readable reports.
- **Not included**: GUI, real-time data acquisition, database storage, network/cloud services, instrument control.

## 7. Explicitly Unsupported Capabilities (Verified)

- Legacy `.xls` (BIFF) binary files are explicitly rejected
- Binary content disguised as CSV is rejected
- No CV (cyclic voltammetry) analysis beyond CHI file parsing of its time-series output
- No chronoamperometry or coulometry analysis
- No 3D or interactive plots
- No Windows-specific testing (CI only runs Linux + macOS)
- No GPU acceleration

## 8. Major Inputs and Outputs

### Inputs

| Format | Extension | Parser |
|--------|-----------|--------|
| CHI EIS | `.csv`, `.txt` | `data_file::chi_file` |
| CHI OCPT | `.csv`, `.txt` | `data_file::chi_file` |
| Generic sensor CSV | `.csv` | `data_file::measurement_parser` |
| Excel workbook | `.xlsx` | `data_file::excel_file` (calamine) |
| Experiment metadata | `.toml` | `domain::metadata` |
| Configuration | `.toml` | Various config modules |
| JSON results | `.json` | Cross-workflow inputs |

### Outputs

| Format | Purpose |
|--------|---------|
| PNG | Publication-quality figures |
| JSON | Machine-readable results, calibration models, reports |
| CSV | Feature tables, model comparisons |
| TXT | Human-readable reports |

## 9. High-Level Execution Model

1. User invokes CLI with a subcommand and arguments
2. `main.rs` parses arguments via `cli.rs` (clap derive), normalizes legacy flags
3. Workspace is prepared: directories created, default config files generated if missing
4. The selected runner coordinates parsing → analysis → reporting → plotting
5. Results are written to `output/` (configurable) or specified paths
6. Exit code 0 on success, 1 on error (error printed to stderr)

## 10. Current Maturity and Limitations

- **Maturity**: Active development (version 0.1.0). All 8 top-level commands have implementations. Tests exist for all major workflows.
- **Limitations**:
  - Only linear regression implemented for plot overlays (no polynomial, exponential, etc.)
  - ECM search uses a genetic algorithm with fixed seed circuits; no exhaustive enumeration option
  - Transient fitting uses a custom optimizer, not the Levenberg-Marquardt crate
  - Kalman filter implementations have limited process/measurement noise models documented
  - No formal uncertainty propagation through multi-step workflows
  - Plot configuration schema is large and complex (~200 fields in `RawPlotStyle`)
  - No dry-run or simulation mode for most commands

## 11. System in One Page

> **rust_electroanalysis_cli** reads electrochemical instrument files and configuration TOML files, then runs analysis commands selected by the user: fitting equivalent-circuit models to EIS data, searching for new circuit models with a genetic algorithm, analysing potentiometric transients with exponential models, building calibration curves with Nernst/Nicolsky-Eisenman equations, characterizing signal quality with PSD/Allan/drift analysis, assessing sensor health from multi-sensor data, comparing EIS and transient timescales, and estimating latent states with Kalman filters. All results are written as JSON/CSV/TXT reports and publication-quality PNG figures to a configurable output directory. A provenance record (SHA-256 + timestamp) is stored with every result for reproducibility.
