# rust_electroanalysis_cli User Manual

- **Software version:** `0.2.0` (Cargo package and lockfile package identity on `main`).
- **Original execution-audit commit:** `6b1ebb84d31cfbd59fb5fc0f7be83161f940d425`.
- **Documentation generation date:** 2026-09-24.
- **Source inspected:** the original audit used that commit; release changes through `v0.2.0` (`c1a76e29d8fe6c66bb6a82d4668f8f06fe0f9c7a`) were reviewed during synchronization.

The original audit ran on pre-release source with a reordered working `Cargo.lock`; its command examples describe those executions. The release tag and current public Phase F status were independently verified during synchronization. The v0.2.0 release remains historical and unchanged by this manual.

This guide is for researchers analyzing impedance and potentiometric data. Start with the quick start, then follow the workflow closest to your experiment. The detailed command and configuration chapters are separate to keep this entry page usable. Together these files form the manual.

## Contents

1. [Quick start](#quick-start)
2. [Scope and verification labels](#scope-and-verification-labels)
3. [Installation](#installation)
4. [Project and research workspace](#project-and-research-workspace)
5. [Preparing your data](#preparing-your-data)
6. [Experiment metadata](#experiment-metadata)
7. [Configuration and precedence](#configuration-and-precedence)
8. [Choosing the right command](#choosing-the-right-command)
9. [Complete CLI reference and command tree](user_manual/command_reference.md)
10. [Configuration field reference](user_manual/configuration.md)
11. [Scientific methods and interpretation](user_manual/scientific_methods.md)
12. [End-to-end workflows and common recipes](user_manual/workflows.md)
13. [Output artifacts and JSON field guide](user_manual/outputs.md)
14. [Figures and plotting](#figures-and-plotting)
15. [Provenance and reproducibility](#provenance-and-reproducibility)
16. [What the software can and cannot conclude](#what-the-software-can-and-cannot-conclude)
17. [Phase F REAL authority](#phase-f-real-authority)
18. [Troubleshooting](user_manual/troubleshooting.md)
19. [Architecture and module reference](user_manual/architecture.md)
20. [Glossary](#glossary)
21. [Command cheat sheet](user_manual/command_reference.md#command-cheat-sheet)
22. [Advanced evidence and validation configuration](user_manual/advanced_contracts.md)
23. [Coverage, execution audit, discrepancies and implementation issues](USER_MANUAL_COVERAGE.md)

## Quick start

**EXAMPLE — executed:** Fit a known circuit to a repository EIS fixture and save a reusable JSON result. Run from the repository root. A release compilation may take longer than five minutes; the analysis itself is short.

```bash
cargo build --locked --release
BIN="${CARGO_TARGET_DIR:-target}/release/rust_electroanalysis_cli"
"$BIN" --help
"$BIN" eis fit tests/fixtures/eis/randles_cpe_weighted_fit.csv \
  --circuit 'R0-p(R1,CPE1)' \
  --artifact output/quickstart/eis_fit.json \
  --report output/quickstart/eis_report.txt
```

Cargo uses `target/` only when no target-directory override is configured. The audit reused its configured external `CARGO_TARGET_DIR`. Preserve your environment's configured build/cache directories.

Open `output/quickstart/eis_report.txt`. Check the circuit, fitted parameters with units, RMSE, weighted RMSE and warnings. Open `eis_fit.json` for measured/predicted arrays, residuals, covariance and provenance. Small residuals are useful evidence of agreement; they do not prove a unique physical circuit. The fit command prints its ordinary report to the terminal too. It does not automatically plot a figure.

For time-series work, this executed fixture recipe selects one event and disables bootstrap to keep the demonstration short:

```bash
"$BIN" transient fit --input tests/fixtures/phase2/sensor.csv \
  --metadata tests/fixtures/phase2/experiment.toml --channel 'E1/V' \
  --config tests/fixtures/phase2/transient.toml \
  --output output/quickstart/transient --model single --bootstrap 0
```

Read `transient_report.txt`, then the selected model and warnings in `transient_results.json`. A successful command can contain rejected events or unavailable scientific quantities. Restore an appropriate bootstrap count for uncertainty work; zero iterations deliberately omits bootstrap uncertainty.

## Scope and verification labels

- **VERIFIED IMPLEMENTATION:** derived from CLI definitions, runner dispatch, scientific code, artifact readers/writers and tests at the original audit commit. The release changes were reviewed; runtime checks are enumerated in the coverage report.
- **DOCUMENTED INTENT:** specifications or comments that describe a policy or planned capability; not proof of execution or approval.
- **EXAMPLE — executed:** an analysis invocation run during this audit, with equivalent absolute paths in the audit workspace where needed.
- **EXAMPLE — illustrative:** a template requiring your experiment files, configuration or an independently prepared manifest. Do not paste placeholder paths unchanged.
- **SCIENTIFIC INTERPRETATION:** guidance about interpreting fitted/statistical results; does not represent independent experimental validation.
- **FUTURE/UNIMPLEMENTED:** no implemented public CLI route at this commit.

Command help parsing, an end-to-end example, and physical validation are different levels of evidence. A fixture is usually synthetic or contract-testing data, not a measurement campaign establishing physical accuracy.

## Installation

**VERIFIED IMPLEMENTATION:** the package/binary is `rust_electroanalysis_cli`; Clap displays `electroanalysis` in usage text. These refer to the same executable. There is no separately declared `electroanalysis` binary in Cargo.toml. The toolchain file pins Rust `1.97.0`, edition 2024, with rustfmt and clippy components. Cargo.lock pins dependencies; `electrodata-io` is pinned to Git revision `dbb6b7d063972114c4208980723e12c807ab199e`.

**EXAMPLE — illustrative installation:** with Git and Rust/rustup installed, obtain the published v0.2.0 source:

```bash
git clone https://github.com/XingyuW/rust_electroanalysis_cli.git
cd rust_electroanalysis_cli
git checkout v0.2.0
cargo build --locked --release
```

Dependencies may require network access on the first build. Follow compiler diagnostics for missing native build/font libraries. The original command audit ran on macOS. The repository CI covers Ubuntu and macOS.

**EXAMPLE — separate release checkout:** the published tag was independently verified to peel to `c1a76e29d8fe6c66bb6a82d4668f8f06fe0f9c7a` during synchronization:

```bash
git clone --branch v0.2.0 \
  https://github.com/XingyuW/rust_electroanalysis_cli.git \
  rust_electroanalysis_cli_v0.2.0
```

**EXAMPLE — optional installation, not executed:** `cargo install --path . --locked` normally installs the binary in `$CARGO_HOME/bin`, usually `$HOME/.cargo/bin`. Put that directory on PATH. From the source checkout, `cargo run --locked -- <arguments>` runs the same CLI in the development profile; `cargo run --locked --release -- <arguments>` uses the release profile. For the usual unconfigured Cargo target directory, help is `./target/release/rust_electroanalysis_cli --help`; with a configured target directory use the `BIN` assignment above.

## Project and research workspace

A workspace is the **current working directory**, not necessarily the location of the executable. Most commands call `prepare_workspace`, which creates missing `config/`, `data/`, `output/`, and `logs/` directories and installs missing configuration templates. Several commands update `config/app.toml` last-run state. `--help`, no-argument usage, `report render` and `validation run` bypass normal analysis workspace setup. Help is safe to inspect without preparing a workspace.

```text
rust_electroanalysis_cli/
├── Cargo.toml, Cargo.lock, rust-toolchain.toml
├── README.md, CHANGELOG.md
├── config/                 shipped configuration and public trust-store JSON
├── data/, output/, logs/   workspace locations (may be empty)
├── src/                    CLI, scientific engines, I/O adapters, writers
├── tests/fixtures/         synthetic inputs and versioned artifact contracts
├── examples/               example model material
├── scripts/                development/verification utilities
├── docs/                   this manual, model guides, ADRs, reviews, specifications
├── .github/                automation/CI definitions
└── plot_config.*.toml      extended plotting examples
```

An existing cache directory such as `pycache/` is not a required scientific input. A `target/` directory is not required inside the source tree when Cargo uses an external target directory. The [architecture reference](user_manual/architecture.md) explains each module and its consumers.

For research, use a separate working directory, retain source data unchanged, and assign one output directory per run:

```text
experiment_001/
├── raw/                    original exports, immutable
├── metadata/experiment.toml
├── config/                 copied, reviewed analysis settings
├── results/run_001/         JSON, CSV and text outputs
├── figures/                selected publication copies
└── README_analysis.md      command lines, units, exclusions, scientific notes
```

Ordinary analysis commands generally overwrite same-named outputs. They do not all support `--overwrite`; only the governed report/validation bundles expose that flag. Avoid accidentally combining outputs from two experiments in one directory.

## Preparing your data

**VERIFIED IMPLEMENTATION:** physical reading is delegated to `electrodata-io`. The local adapter consumes canonical datasets rather than recognizing headers independently. Supported ordinary containers are text CSV/TXT/DAT and XLSX; actual content and compatible scientific roles determine usability. Unusual extensions can still be interpreted by provider content detection. A recognized suffix alone is not acceptance.

### EIS

Supply one frequency column and real/imaginary impedance. The canonical roles are `Frequency`, `ImpedanceReal`, `ImpedanceImaginary`, with optional `ImpedanceMagnitude` and `ImpedancePhase`. Use Hz, ohms and degrees; preserve the instrument's signed imaginary values. Do not feed a plotted `-Z''` column as signed `Z''` without converting and documenting it.

**EXAMPLE — illustrative minimal content:**

```csv
Freq/Hz,Z'/ohm,Z''/ohm
1000,100,-10
100,120,-30
10,180,-20
```

This illustrates format only; three points are not a recommended circuit-identification dataset. The repository's `tests/fixtures/electrodata_io_contract/chi_eis_three_column.csv` and reordered/four/five-column fixtures demonstrate recognized roles. Source measured magnitude/phase are retained separately from values derived from real/imaginary components. Absent or missing phase entries are derived using `atan2(Z'',Z')` for the legacy fit vector. Magnitude is `sqrt(Z'^2+Z''^2)`. The EIS adapter copies numeric coordinate/impedance values; do not assume it converts arbitrary frequency or impedance scales into Hz/ohms.

A file is required for direct fit/export; `eis search` also accepts a directory. Batch discovery inspects candidates, excludes known generated artifacts, and reports per-file failures. Review the batch summary even when some inputs succeeded. File traversal is not a general recursive data-lake importer.

### OCPT and multichannel time series

```csv
time/sec,E1/V,E2/V
0,0.100,0.110
1,0.102,0.111
2,,0.112
```

This is a format example, not a sufficient transient/calibration record. The time-series view requires a time coordinate and aligned measurement columns. `Potential`, `Current`, `MeasurementChannel(n)` and `Unknown` measurement roles may survive ingestion; the chosen scientific workflow still requires compatible units. A numeric current channel is not interchangeable with electrode potential.

The domain retains source coordinate values and units. Scientific workflows explicitly normalize supported time units to seconds; metadata event timestamps and window settings are in seconds. Potential workflows convert supported potential quantities to volts. Use explicit headers such as `time/sec` and `E1/V` to remove ambiguity. `--channel 'E1/V'` matches a source header; the logical bare name `E1` is also supported when its suffix is a recognized unit. Real slash-containing identifiers are not indiscriminately truncated.

Missing measurement cells stay aligned as optional values. The normal compatibility reader skips malformed coordinate rows, converts invalid measurement cells to missing values, pads ragged rows, preserves coordinate order, and records diagnostics. It rejects lossy UTF-8 decoding. These ingestion decisions are not permission to ignore missingness: transient, signal and estimation apply different scientific acceptance policies afterward.

### Generic sensor tables and TXT/DAT

Generic plotting can use numeric columns whose roles are not known. Configure column selection, labels and units rather than assuming every numeric channel is potential. Headerless legacy tables may be recoverable with provider diagnostics, but explicit semantic headers are preferable for new work. CSV/TXT/DAT must contain supported tabular text or supported CHI exports; arbitrary instrument binary data renamed `.csv` is not supported. See `generic_text.dat` and `regular_multichannel.csv` under `tests/fixtures/electrodata_io_contract/`.

### XLSX and worksheet selection

Workbook reading belongs to the provider. With automatic selection, one compatible worksheet can be selected; multiple compatible sheets yield structured ambiguity rather than an implicit first-sheet choice. Use the exact worksheet name via `--sheet NAME` on EIS fit/search/export, transient fit, calibration extract, signal characterize and estimate run/compare. Search applies the selector to each candidate workbook. Plot jobs also have no worksheet selector. Calibration **predict** has no `--sheet` flag and uses automatic selection; signal comparison manifests also have no sheet selector. Export the selected sheet as CSV if that route cannot disambiguate your workbook.

Unsupported `.xls` is not made supported by the legacy `InputKind::ExcelXls` enumeration. Binary files, misleading text extensions, missing required roles, malformed workbooks, ambiguous sheets and wholly unusable required coordinates can fail. Inspect ingestion messages rather than silently renaming extensions.

## Experiment metadata

The TOML document separates experimental meaning from the data file and from plot styling. `experiment_id` is required by deserialization. The other top-level sections have defaults, but an analysis can require information inside them. A parsed empty or incomplete metadata document does not establish a scientifically usable experiment.

| Field | Type / requiredness | Units and purpose | Main consumers |
|---|---|---|---|
| `experiment_id` | required string | Stable experimental identity | Transient, calibration, estimation, evidence lineage |
| `sample_matrix` | string; defaults empty | Matrix comparability, e.g. buffer/composition | Calibration and health/context comparisons |
| `[sensor]` | optional table | Working-sensor identity | All metadata-aware analyses |
| `sensor_id`, `name`, `sensor_type`, `analyte`, `manufacturer`, `model` | optional sensor strings | Identity/design/target ion; no implicit charge inference guarantee | Calibration target resolution, evidence and health |
| `sensor.metadata` | optional string-to-string table | Extra declared annotations | Context-aware consumers; not arbitrary automatic model inputs |
| `[reference]` | optional table | Reference electrode description | Experiment record/provenance context |
| `reference_id`, `electrode_type`, `manufacturer`, `model` | optional strings | Reference identity | Context; no automatic universal reference correction |
| `potential`, `potential_unit` | optional number and string | Declared reference potential with explicit unit | Stored reference metadata |
| `reference.metadata` | optional string map | Additional reference context | Metadata consumers |
| `[[environmental_data]]` | optional array; alias `environmental_series` | Temperature/flow/conductivity/ionic strength | Calibration and estimation alignment |
| `name`, `unit`, `time`, `values` | required per environmental series | Time in s, equal-length arrays, finite values; domain allows missing values | Named series chosen by config |
| `environmental_data.metadata` | optional string map | Series annotations | Stored context |
| `[[events]]` | optional array | Timestamped experimental actions | Transient segmentation, calibration, signal exclusion, estimation input |
| `timestamp`, `kind` | required per event | Seconds; kind from list below | Event-aware workflows |
| `value`, `unit`, `analyte`, `annotation`, `metadata` | optional | Quantity/action description, string-valued extra fields | Concentration extraction and explicit event adapters |

Implemented event kinds (TOML uses underscores): `concentration_step`, `flow_change`, `temperature_change`, `ionic_strength_change`, `interferent_addition`, `flush_start`, `reading_start`, `flush_end`, `manual_annotation`. There is **no `reading_end` variant**. The transient CLI spells these with hyphens, e.g. `--event-kind concentration-step`. Events are sorted by timestamp when building an experiment. `--event-index` is zero-based among the eligible events, not necessarily the original TOML row index.

**EXAMPLE — illustrative complete metadata:** align the times and values to your real acquisition; do not reuse these identities for unrelated experiments.

```toml
experiment_id = "ism-buffer-step-001"
sample_matrix = "aqueous buffer; composition recorded in lab notebook"

[sensor]
sensor_id = "ism-01"
name = "working membrane"
sensor_type = "ion_selective_membrane"
analyte = "K+"
manufacturer = "laboratory"
model = "design-A"

[reference]
reference_id = "ref-01"
electrode_type = "Ag/AgCl"
manufacturer = "laboratory"
model = "reference-A"

[[environmental_data]]
name = "temperature"
unit = "degC"
time = [0.0, 300.0, 600.0]
values = [25.0, 25.0, 25.1]

[[events]]
timestamp = 20.0
kind = "concentration_step"
value = 0.001
unit = "mol/L"
analyte = "K+"
annotation = "Final bulk concentration after addition"

[[events]]
timestamp = 400.0
kind = "flush_start"
annotation = "Begin rinse"
```

A concentration-step value means the declared concentration at that step, not automatically the added stock volume or dose. Calibration needs positive concentration/activity and usable response observations across several levels. Mass concentration conversion requires `analyte.molar_mass_g_per_mol`; conductivity is not a substitute for ion activity unless an explicit empirical model is selected. Event metadata strings can carry `concentration_unit`, explicit activity/activity coefficient, temperature/ionic-strength quantities and interferent activities as implemented by calibration extraction. See the detailed [configuration reference](user_manual/configuration.md) before using activity corrections.

Environmental alignment is separately configured (nearest/interpolation/window statistics/hold-previous in estimation). Sparse environmental samples can exceed the maximum alignment gap even if the overall experiment contains a temperature record. Supply a realistic environmental sampling interval and inspect fallbacks in results.

## Configuration and precedence

See the [complete field reference](user_manual/configuration.md) for all 11 shipped TOML files, the public trust-store JSON, advanced fields absent from templates, and separate Phase-B/Phase-C/Phase-E contracts.

There is **no universal merged configuration**. Normal transient/calibration/estimation resolution is: built-in struct defaults → selected TOML (missing members retain defaults) → supported CLI overrides → validation. Explicit config selects a file; it does not merge an additional overlay into the default workspace TOML. Metadata supplies experiment-specific facts to scientific adapters, not a blanket override of every TOML field.

Plot/search/circuit configuration also supports legacy paths. Workspace preparation can migrate legacy `plot_config.toml`, `ecm_search.toml` and `circuit_models.toml` to `config/plotting.toml`, `config/analysis.toml`, and `config/parsing.toml`. Config-relative plot input/output paths are relative to the selected config file's directory. CLI-relative paths generally resolve from the working directory. Manifest record paths generally resolve from the manifest's directory. Use absolute paths when sharing recipes across workspaces.

Circuit selection is special: explicit CLI `--circuit` wins for that fit; otherwise filename `circuit=`/`model=` tags, recognized circuit metadata, first matching parsing rule, then `fallback_model`. Search discovers candidate circuits and uses its own ranking criterion. Its default BIC is not the `parsing.toml` model-selection AIC setting.

Missing explicit files are not handled consistently: plotting/search/transient/calibration can reject explicit missing overrides, signal/health read errors propagate, while model loading falls back to the built-in model even for a nonexistent requested path. Sparse mechanism TOML has a defaulting trap described in troubleshooting. For reproducibility, verify the resolved configuration and warnings in each result rather than relying only on the command's exit code.

## Choosing the right command

```text
What do you have or need?
├─ Raw EIS
│  ├─ Inspect visually → plot eis (configure input/output)
│  ├─ Known circuit → eis fit; --artifact saves reusable JSON
│  ├─ Unknown topology → eis search, then refit selected circuit
│  └─ Durable fit export → eis export-fit (performs a fit)
├─ Time-series with experimental events
│  ├─ Relaxation times/amplitudes → transient fit
│  ├─ Equilibrium standards → calibration extract → fit → validate → predict
│  └─ Latent activity/baseline/polarization → estimate run → validate/compare
├─ Noise, drift, spikes, stability → signal characterize
│  ├─ Compare raw records → signal compare
│  └─ Inspect fit errors → signal residuals
├─ EIS + transient artifacts → mechanism compare; replicate manifest → trend
├─ Sensor performance over time → health baseline → assess → trend
├─ Declared reduced ISM model → model validate → simulate/decompose → report
├─ Frozen mechanism + health evidence → report render
├─ Frozen cohort/protocol assessment → validation run
└─ Synthetic estimation benchmark → estimate simulate
```

| Scientific question | Command / result to inspect |
|---|---|
| Which circuit describes this spectrum statistically? | `eis search`: BIC/AIC, ranked fits and residuals |
| What is its characteristic relaxation time? | `eis fit` then topology-aware `mechanism compare` |
| Is one or two transient modes justified? | `transient fit`: candidate AIC/BIC and diagnostics |
| Does the sensor approach a reproducible equilibrium? | Transient extrapolation + independent late-window calibration checks |
| What concentration corresponds to this voltage? | `calibration predict`, conditional on stored calibration/domain/activity model |
| Is voltage drifting? | `signal characterize`: ordinary-linear/Theil–Sen slopes |
| Which averaging time reduces noise? | Signal Allan deviation, considering drift and available cluster count |
| Are EIS and OCP timescales compatible? | `mechanism compare`, with identity and uncertainty checks |
| Is current sensor behavior unlike its baseline? | `health assess`, inspect dimensions/features and missing evidence |
| Can latent states be separated from voltage? | `estimate run`: observability, innovations and covariance |
| Do a model's declared components reconstruct voltage? | `model decompose`: contributions and unexplained residual |
| Does held-out evidence meet declared acceptance criteria? | `validation run`; distinguish software versus physical claim ceiling |

## Figures and plotting

`plot` takes an optional **target positional value**, not a nested subcommand: `all` (default), `eis`, `regular-plot`, `generic-plot`. It has no raw-file positional argument; set the input in plotting TOML. Configured EIS jobs render Nyquist and Bode views, regular jobs time series, and generic jobs numeric scatter/line/bar-style data and supported regression overlays. Axis transforms and logarithmic axes are distinct controls: transforming numeric values and then using a log display can double-transform data. Default unspecified log base is 10.

Nyquist convention displays real versus negative imaginary impedance; Bode shows magnitude and phase versus frequency. Retained source magnitude/phase and computed complex-data channels have distinct meanings. Regression on plotted data is descriptive, not a dedicated electrochemical calibration. Plotting can invoke circuit fitting/overlays according to job settings; changing appearance does not fix scientifically invalid input.

Transient, calibration and legacy mechanism plotting produce PNG/SVG pairs through shared rendering helpers. Signal, health, estimation and model workflow plots use their dedicated PNG writers. Phase-D report figures produce governed figure outputs with availability/reason tracking. Do not assume a global format switch applies to every plotting module. Exact filenames and conditional figures appear in the [output reference](user_manual/outputs.md).

## Provenance and reproducibility

`AnalysisProvenance` records package version, input path and SHA-256, optional configuration path/hash, Unix generation timestamp and optional Git commit. The Git commit comes from compile-time `GIT_COMMIT` when supplied; it is not guaranteed to be populated by a normal Cargo build. Configuration hashes identify file bytes, not necessarily every effective CLI override. The transient runner replaces the metadata-file provenance configuration identity with the analysis config identity. Keep metadata separately even when a result embeds experiment facts.

Newer artifacts carry `schema_version`, kind where required, and lineage with experiment/sensor/channel scope, dependencies and acquisition-family information. Unknown lineage stays unknown. A readable legacy artifact does not thereby acquire verified independence or current-schema scientific authority. Public report/validation bundles add manifests and managed-output integrity checks; these do not replace raw data preservation.

Preserve raw files, metadata, exact config bytes, exact command and working directory, stdout/stderr, source commit/tag, lockfile, JSON, CSV, figures and interpretation notes. Record pre-existing source or dependency changes. Fix seeds where offered (transient/calibration bootstrap and estimation simulation); ECM evolutionary search has no general CLI seed switch. Parallel numerical execution, optimizer changes, font systems and timestamps can prevent byte-for-byte identical output even with equivalent numerical results.

## What the software can and cannot conclude

**SCIENTIFIC INTERPRETATION:** EIS fitting does not demonstrate a physically unique ECM. A transient time constant is a fitted descriptor, not an automatic chemical or transport assignment. Matching an EIS time constant to an OCP relaxation supports temporal compatibility; double-layer charging, diffusion, binding, mixing, reference drift and other explanations require experimental discrimination.

Correlation does not establish causation. A health finding is conditional evidence against a declared baseline and rule set, not a definitive failure diagnosis. Multiple outputs derived from the same acquisition are not automatically independent evidence. Good prediction can coexist with unidentifiable model parameters or states.

The ISM implementation is a reduced-order component framework. Its phenomenological modes are not a high-fidelity Nernst–Planck transport solver. Kalman uncertainty depends on observation/process noise, observability and model adequacy. A confidence interval from an inadequate model is not a universal accuracy guarantee. Synthetic agreement, software tests, contract conformance and an approved documentation plan do not establish physical validation.

## Phase F REAL authority

**PUBLIC STATUS:** [Phase F REAL Authority Status](engineering_specification/phase_f/phase_f_real_authority_status.md) records a non-authoritative operational snapshot for v0.2.0. Phase F REAL authority is **NO-GO**. Running this CLI or following this manual does not establish governance or physical-validation authority.

At the snapshot date, structurally ready REAL reviewers were `0 / 5`; the REAL verifier was not created; reviewer subjects, identity/equivalence admissions and currentness proof were absent; the protected monotonic ref was absent; and REAL G3 was NO-GO. These are statements from the public snapshot, not a live inspection of private or external authority state. `TEST_ONLY` evidence and fixtures do not promote REAL authority.

Ordinary research analysis remains available subject to the scientific limitations above. No private key, private reviewer material, REAL verifier/subject/proof generation, monotonic-ref creation or quorum change is needed for these workflows. **REAL authority mutations during this task: none.**

## Glossary

| Term | Meaning here |
|---|---|
| EIS | Electrochemical impedance spectroscopy: complex response versus frequency |
| ECM | Equivalent circuit model; a fitted mathematical representation |
| OCP / OCPT | Open-circuit potential / open-circuit potential versus time |
| ISM / ISE | Ion-selective membrane / ion-selective electrode |
| CPE | Constant-phase element; Q and exponent are not generally a simple capacitance |
| Warburg | Diffusion-like impedance element; applicability depends on boundary assumptions |
| Zarc | Distributed relaxation impedance characterized by resistance, timescale and exponent |
| tau | Characteristic relaxation time, in seconds |
| RSS | Sum of squared residuals; distinguish weighted and unweighted definitions |
| AIC / AICc / BIC | Relative model-selection criteria balancing residual agreement and complexity |
| RMSE | Root mean square error; compare only compatible definitions and units |
| PSD / ASD | Power spectral density / its amplitude-density square root |
| Allan variance | Adjacent averaged-block difference statistic across averaging times |
| EKF / UKF | Extended / unscented Kalman filter for nonlinear state estimation |
| NIS | Normalized innovation squared: consistency measure relative to predicted innovation variance |
| Artifact | Persisted analysis result with a reader/writer contract |
| Provenance | Input/configuration/software identity and processing record |
| Lineage | Dependency and acquisition-family relationships among artifacts |
| Currentness | Governed evidence that an authority/result is current under lifecycle rules; not filesystem modification time |
| G3 | Phase F specification-bundle approval gate |
| TEST_ONLY | Test classification, not REAL authority |
| REAL authority | Governed physical/publication authority requiring its own independent evidence and approval chain |

[Back to top](#rust_electroanalysis_cli-user-manual)
