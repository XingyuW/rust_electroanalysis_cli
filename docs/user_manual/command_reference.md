# Complete CLI command reference

[Master manual](../USER_MANUAL.md) · [Configuration](configuration.md) · [Outputs](outputs.md) · [Workflows](workflows.md)

**VERIFIED IMPLEMENTATION:** 11 top-level commands, 29 nested subcommands, and 30 executable leaf forms counting `plot`. Its four target values are not subcommands. There are no Phase-F authority or private-key commands in this CLI. Every help level was executed.

## Command tree

```text
rust_electroanalysis_cli
├── plot [all|eis|regular-plot|generic-plot]
├── eis
│   ├── fit
│   ├── export-fit
│   ├── search
├── transient
│   ├── fit
├── calibration
│   ├── extract
│   ├── fit
│   ├── validate
│   ├── predict
├── mechanism
│   ├── compare
│   ├── trend
│   ├── report
├── signal
│   ├── characterize
│   ├── compare
│   ├── residuals
├── health
│   ├── baseline
│   ├── assess
│   ├── trend
│   ├── report
├── report
│   ├── render
├── validation
│   ├── run
├── estimate
│   ├── run
│   ├── validate
│   ├── simulate
│   ├── compare
│   ├── report
├── model
│   ├── validate
│   ├── simulate
│   ├── decompose
│   ├── report
```

## Common conventions

The actual binary is `rust_electroanalysis_cli`; runtime usage calls it `electroanalysis`. All flags are command-local. Use `-h`/`--help` on any level and `-V`/`--version` at root. Replace the binary by `cargo run --locked --` for a source-build equivalent. Shell-quote circuit expressions and channel names. Tables below list all application fields; help/version are documented here once.

Root legacy flags remain accepted: `--plot TARGET`, `--plot-config PATH`, `--search-eis PATH`, `--search-config PATH`, `--search-output PATH`, `--search-top N`. They normalize to structured plot/search commands. Do not mix a structured command and legacy selectors or combine plot and search. The generic legacy enum value is `generic-plot` (some old prose says `generic`).

Parent commands `eis`, `transient`, `calibration`, `mechanism`, `signal`, `health`, `report`, `validation`, `estimate`, and `model` group the leaf commands below. Their only own option is help; they perform no standalone analysis without a child.

Syntax blocks show accepted parser usage, not complete runnable shell recipes. Examples labelled illustrative require preparation. Executed examples are preserved in the coverage report; no authority-changing operations are examples.

## plot

### Purpose

Render configured electrochemical and generic figures.

### When to use it

Inspect source data and make configured presentation plots.

### Syntax

```text
rust_electroanalysis_cli plot [OPTIONS] [TARGET]
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| None | — | No | See conditional input requirements below. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `TARGET` | PlotTarget::All, value_name = "TARGET") | all, eis, regular-plot, generic-plot; all by default. Type: `PlotTarget`. | Select a different input/context or destination explicitly. |
| `--plot-config` | Workspace TOML / loader defaults | Plotting TOML override; --config alias supported. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Plotting TOML selects files/directories and column mappings (no worksheet selector); no raw input positional argument.

### Internal workflow

CLI → runners/plot → plot_runner → canonical data adapters → plottings and optional fitting/regression → image files..

### Scientific calculations

Nyquist/Bode conversion, configured regression and axis transforms; see scientific methods. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`plot` output row](outputs.md#plot) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Check units, axes, exclusions, measured-versus-derived Bode channels and fit overlays.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli plot eis --plot-config plotting.toml
```

### Common errors

Missing config/input, ambiguous worksheet, invalid transform range, unsupported columns or font/render errors.

### Limitations

Target selection filters configured jobs. A good-looking plot is not validation; jobs may fit data as part of plotting.

[Back to command tree](#command-tree)

## eis fit

### Purpose

Fit one complex impedance spectrum to a specified or resolved circuit.

### When to use it

You have a physically motivated candidate ECM.

### Syntax

```text
rust_electroanalysis_cli eis fit [OPTIONS] <INPUT>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `INPUT` | `PathBuf` | Yes | Raw input file; model decompose instead takes scientific ModelInput JSON. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--sheet` | Absent (no CLI default) | Exact compatible XLSX worksheet name; omitted uses provider automatic selection. Type: `Option<String>`. | Match the actual source layout. |
| `--circuit / -c` | Absent (no CLI default) | Circuit expression; omitted resolves filename/metadata/rules/fallback. Type: `Option<String>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--artifact` | Absent (no CLI default) | Durable EIS fit JSON destination. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--report` | Absent (no CLI default) | Human-readable EIS artifact report file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

One canonical EIS file (CSV/TXT/DAT/XLSX), frequency in Hz, signed complex impedance in ohms; optional sheet; circuit resolution uses parsing configuration.

### Internal workflow

CLI → runners/fit → EISData → fit_circuit_detailed → named fit result → optional artifact writer..

### Scientific calculations

Constrained nonlinear fitting, complex residuals, covariance/rank and fit statistics. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`eis fit` output row](outputs.md#eis-fit) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Inspect parameter units/bounds, residual structure, weighted RMSE and covariance validity, not just convergence.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli eis fit tests/fixtures/eis/randles_cpe_weighted_fit.csv --circuit 'R0-p(R1,CPE1)' --artifact output/quickstart/eis_fit.json --report output/quickstart/eis_report.txt
```

### Common errors

Invalid circuit, missing roles, empty/nonfinite data, incompatible worksheet, unwritable destination.

### Limitations

Circuit/parameter nonuniqueness remains. Default stdout report is not a reusable JSON artifact. No automatic figures.

[Back to command tree](#command-tree)

## eis export-fit

### Purpose

Perform a fit and write its durable EIS artifact.

### When to use it

Pass an EIS result into mechanism, health or residual workflows.

### Syntax

```text
rust_electroanalysis_cli eis export-fit [OPTIONS] --artifact <PATH> <INPUT>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `INPUT` | `PathBuf` | Yes | Raw input file; model decompose instead takes scientific ModelInput JSON. |
| `--artifact` | `PathBuf` | Yes | Durable EIS fit JSON destination. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--sheet` | Absent (no CLI default) | Exact compatible XLSX worksheet name; omitted uses provider automatic selection. Type: `Option<String>`. | Match the actual source layout. |
| `--circuit / -c` | Absent (no CLI default) | Circuit expression; omitted resolves filename/metadata/rules/fallback. Type: `Option<String>`. | Select a different input/context or destination explicitly. |
| `--report` | Absent (no CLI default) | Human-readable EIS artifact report file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Same physical EIS input as eis fit; --artifact is required.

### Internal workflow

CLI → runners/fit::export → same fit pipeline as eis fit → JSON and optional text..

### Scientific calculations

Identical fitting to eis fit; this is not conversion of an existing text report. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`eis export-fit` output row](outputs.md#eis-export-fit) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Confirm source identity, circuit and covariance before cross-workflow use.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli eis export-fit tests/fixtures/phase2/eis.csv --circuit 'R0-p(CPE1,R1)' --artifact output/eis/export.json
```

### Common errors

Same fitting errors; wrong output path type or file permissions.

### Limitations

Re-fits the raw spectrum. Existing files may be overwritten.

[Back to command tree](#command-tree)

## eis search

### Purpose

Search a bounded family of equivalent circuits and rank candidate fits.

### When to use it

You want statistical candidate discovery before scientific interpretation.

### Syntax

```text
rust_electroanalysis_cli eis search [OPTIONS] <INPUT>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `INPUT` | `PathBuf` | Yes | Raw input file; model decompose instead takes scientific ModelInput JSON. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--sheet` | Absent (no CLI default) | Exact compatible XLSX worksheet name; omitted uses provider automatic selection. Type: `Option<String>`. | Match the actual source layout. |
| `--search-config` | Workspace TOML / loader defaults | Analysis/search TOML override; --config alias supported. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--search-output` | Absent (no CLI default) | Search report file for a single input or directory of reports. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--search-top` | Absent (no CLI default) | Maximum ranked candidates retained; overrides max_ranked_results. Type: `Option<usize>`. | Select a different input/context or destination explicitly. |

### Input requirements

One EIS file or directory of candidate files; optional common worksheet selector and analysis TOML.

### Internal workflow

CLI → runners/search → search_runner → canonical discovery → ECM evolution/scoring → TXT/CSV ranking and optional plots..

### Scientific calculations

Genetic candidate evolution with numerical fitting; default Gaussian BIC; alternative AIC, weighted_rmse and legacy_penalized_score in evolution.ranking_criterion. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`eis search` output row](outputs.md#eis-search) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Lower criterion is preferred within the same data/objective. Inspect several plausible circuits and residuals.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli eis search tests/fixtures/phase2/eis.csv --search-config search.toml --search-output output/search --search-top 1
```

### Common errors

No usable EIS files, invalid numeric search config, zero top_n (rejected), per-file parse/fit failures.

### Limitations

Search is not exhaustive or proof of uniqueness. No general CLI random-seed option; ranking can vary.

[Back to command tree](#command-tree)

## transient fit

### Purpose

Segment event-driven potentiometric responses and fit relaxation models.

### When to use it

Estimate response times, amplitudes, extrapolated equilibrium and drift.

### Syntax

```text
rust_electroanalysis_cli transient fit [OPTIONS] --input <PATH> --metadata <PATH> --channel <NAME>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--input` | `PathBuf` | Yes | Raw input file; model decompose instead takes scientific ModelInput JSON. |
| `--metadata` | `PathBuf` | Yes | Experiment/context TOML; align identity, events and environmental units. |
| `--channel` | `String` | Yes | Potential/measurement channel by logical name or source-header alias. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--sheet` | Absent (no CLI default) | Exact compatible XLSX worksheet name; omitted uses provider automatic selection. Type: `Option<String>`. | Match the actual source layout. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--event-kind` | TransientEventKindArg::ConcentrationStep) | Eligible event category; default concentration-step; CLI uses hyphens. Type: `TransientEventKindArg`. | Select a different input/context or destination explicitly. |
| `--event-index` | Absent (no CLI default) | Zero-based index among eligible events; omitted processes all eligible events. Type: `Option<usize>`. | Select a different input/context or destination explicitly. |
| `--model` | Absent (no CLI default) | single, double, double-drift, stretched or all (Clap validates). Type: `Option<TransientModelArg>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--selection` | Selected configuration | Candidate ranking criterion; transient aic/bic, calibration aic/aicc/bic/cross_validation. Type: `Option<TransientSelectionArg>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--bootstrap` | Selected configuration | Bootstrap iteration count; 0 disables bootstrap, reducing uncertainty information. Type: `Option<usize>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--seed` | Selected configuration | Unsigned reproducibility seed override. Type: `Option<u64>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |

### Input requirements

Time series with a potential channel plus experiment TOML and eligible events; optional XLSX sheet.

### Internal workflow

CLI → runners/transient → load experiment → segmentation/baseline → multistart fits → diagnostics/selection/bootstrap → JSON/CSV/text/conditional plots..

### Scientific calculations

Single/double exponential, double with drift, or stretched exponential; AIC/BIC and residual bootstrap. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`transient fit` output row](outputs.md#transient-fit) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Selected model may be absent. Examine tau/window ratio, mode separation, amplitudes, bootstrap success, residual autocorrelation and warnings.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli transient fit --input tests/fixtures/phase2/sensor.csv --metadata tests/fixtures/phase2/experiment.toml --channel E1/V --config tests/fixtures/phase2/transient.toml --output output/quickstart/transient --model single --bootstrap 0
```

### Common errors

Missing metadata/channel/event, no baseline/window, too many missing values, duplicate time policy, failed optimizer.

### Limitations

An extrapolated asymptote is not observed equilibrium. Short windows can make slow tau weakly identifiable.

[Back to command tree](#command-tree)

## calibration extract

### Purpose

Create calibration observations from concentration-step experiments.

### When to use it

Prepare standardized observations before calibration fitting.

### Syntax

```text
rust_electroanalysis_cli calibration extract [OPTIONS] --input <PATH> --metadata <PATH> --channel <NAME>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--input` | `PathBuf` | Yes | Raw input file; model decompose instead takes scientific ModelInput JSON. |
| `--metadata` | `PathBuf` | Yes | Experiment/context TOML; align identity, events and environmental units. |
| `--channel` | `String` | Yes | Potential/measurement channel by logical name or source-header alias. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--sheet` | Absent (no CLI default) | Exact compatible XLSX worksheet name; omitted uses provider automatic selection. Type: `Option<String>`. | Match the actual source layout. |
| `--transient-results` | Absent (no CLI default) | Existing transient results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Time series, metadata with concentrations/units/analyte, selected potential channel; optional transient artifact for equilibrium estimates.

### Internal workflow

CLI → runners/calibration::extract → experiment loader → observations extraction → typed observation artifact..

### Scientific calculations

Prefer eligible transient equilibrium estimates; otherwise configured late-window mean/median with missingness/point/slope checks; temperature/activity handling. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`calibration extract` output row](outputs.md#calibration-extract) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Inspect source, warnings, branch, concentration/activity, temperature and potential uncertainty of each observation.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli calibration extract --input standards.csv --metadata standards.toml --channel E1/V --config calibration.toml --output output/calibration
```

### Common errors

No usable concentration steps, unknown units/missing molar mass, too few points or unstable late window.

### Limitations

The extraction command can succeed with unusable observations; inspect before fitting. Default late window is 180–300 s after an event.

[Back to command tree](#command-tree)

## calibration fit

### Purpose

Fit an equilibrium potential-to-activity model to stored observations.

### When to use it

Build a model for interpolation/prediction and quantify residuals/hysteresis.

### Syntax

```text
rust_electroanalysis_cli calibration fit [OPTIONS] --observations <PATH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--observations` | `PathBuf` | Yes | Typed calibration observation-set JSON, not raw time series. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--model` | Absent (no CLI default) | nernst, nicolsky_eisenman, conductivity_empirical or all; runner validates. Type: `Option<String>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--selection` | Selected configuration | Candidate ranking criterion; transient aic/bic, calibration aic/aicc/bic/cross_validation. Type: `Option<String>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--bootstrap` | Selected configuration | Bootstrap iteration count; 0 disables bootstrap, reducing uncertainty information. Type: `Option<usize>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--seed` | Selected configuration | Unsigned reproducibility seed override. Type: `Option<u64>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |

### Input requirements

Typed calibration_observations.json, optional calibration TOML; not a raw voltage CSV.

### Internal workflow

CLI → typed reader → fit_calibration → model selection/validation/bootstrap → stored model + analysis + tables/figures..

### Scientific calculations

Nernst, configured Nicolsky–Eisenman or empirical conductivity correction; weighted regression and AIC/AICc/BIC/CV selection. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`calibration fit` output row](outputs.md#calibration-fit) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Check selected candidate, slope/sign/temperature, domain, uncertainty, held-out errors and branch effects.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli calibration fit --observations output/calibration/calibration_observations.json --config calibration.toml --bootstrap 0 --output output/calibration
```

### Common errors

Insufficient valid/distinct levels, absent activity inputs, unsupported model/criterion, no successful candidate.

### Limitations

A free slope absorbs effects beyond ion charge; extrapolation and matrix transfer need independent validation.

[Back to command tree](#command-tree)

## calibration validate

### Purpose

Evaluate a frozen calibration model against supplied observations.

### When to use it

Test held-out standards without refitting the model.

### Syntax

```text
rust_electroanalysis_cli calibration validate [OPTIONS] --model <PATH> --observations <PATH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--model` | `PathBuf` | Yes | Model selector or artifact/configuration path as described below. |
| `--observations` | `PathBuf` | Yes | Typed calibration observation-set JSON, not raw time series. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Stored calibration model and observation-set JSON artifacts.

### Internal workflow

CLI → typed readers → validate_stored_model → validation JSON/CSV/text..

### Scientific calculations

Prediction/residual statistics for supplied observations. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`calibration validate` output row](outputs.md#calibration-validate) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Prefer independent concentrations/experiments; examine bias and coverage, not training fit alone.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli calibration validate --model output/calibration/calibration_model.json --observations output/calibration/calibration_observations.json --output output/calibration_validation
```

### Common errors

Wrong artifact kind/version, missing model parameters, incompatible or unusable observations.

### Limitations

It does not make the supplied observations independent; using training observations is only a smoke check.

[Back to command tree](#command-tree)

## calibration predict

### Purpose

Invert a stored calibration model for activity and available concentration.

### When to use it

Convert a new potential or channel into model-conditional predictions.

### Syntax

```text
rust_electroanalysis_cli calibration predict [OPTIONS] --model <PATH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--model` | `PathBuf` | Yes | Model selector or artifact/configuration path as described below. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--potential` | Absent (no CLI default) | Scalar potential in volts; takes precedence over --input if both supplied. Type: `Option<f64>`. | Select a different input/context or destination explicitly. |
| `--temperature` | Absent (no CLI default) | Temperature in Celsius; internally converted to kelvin. Type: `Option<f64>`. | Select a different input/context or destination explicitly. |
| `--input` | Absent (no CLI default) | Raw input file; model decompose instead takes scientific ModelInput JSON. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--channel` | Absent (no CLI default) | Potential/measurement channel by logical name or source-header alias. Type: `Option<String>`. | Match the actual source layout. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Stored calibration model; --potential in V or --input with --channel; optional --temperature in °C.

### Internal workflow

CLI → model reader → scalar or parsed channel → potential-unit conversion → inversion → JSON or CSV by output suffix..

### Scientific calculations

Inverse equilibrium model, temperature adjustment and domain/extrapolation diagnostics. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`calibration predict` output row](outputs.md#calibration-predict) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Inspect activity separately from molar concentration, temperature, extrapolated flag and domain distance.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli calibration predict --model output/calibration/calibration_model.json --potential 0.12252 --temperature 25 --output output/prediction.json
```

### Common errors

Neither input mode supplied; --input without channel; invalid model or units; missing output parent.

### Limitations

If both modes are supplied, scalar potential takes precedence. Missing/unconvertible batch cells are skipped and predictions do not preserve the original time axis. No sheet selector or general CLI interferent-input map.

[Back to command tree](#command-tree)

## mechanism compare

### Purpose

Compare EIS and transient evidence, optionally through the explicit Phase-B hypothesis contract.

### When to use it

Assess temporal compatibility and declared evidence gates.

### Syntax

```text
rust_electroanalysis_cli mechanism compare [OPTIONS] --eis-artifact <PATH> --transient-artifact <PATH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--eis-artifact` | `PathBuf` | Yes | Required reusable EIS fit JSON. |
| `--transient-artifact` | `PathBuf` | Yes | Required transient analysis JSON. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--calibration-results` | Absent (no CLI default) | Existing calibration results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--metadata` | Absent (no CLI default) | Experiment/context TOML; align identity, events and environmental units. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--mechanism-evidence-config` | Absent (no CLI default) | Explicit Phase-B hypothesis/evidence TOML; selects a different runner branch. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--state-estimation-artifact` | Absent (no CLI default) | Phase-B optional state-estimation evidence artifact. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--calibration-observations-artifact` | Absent (no CLI default) | Phase-B optional calibration observation evidence. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--prior-mechanism-artifact` | Absent (no CLI default) | Phase-B previous mechanism artifact for explicit hypothesis history. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Required EIS and transient artifacts; legacy context options or a separate Phase-B evidence TOML with optional estimation/observations/prior history.

### Internal workflow

CLI → legacy compare or compare_phase_b branch → typed evidence preparation → timescales/pairs/gates/history → mechanism artifact and tables/plots..

### Scientific calculations

Topology-aware RC/R-CPE/relaxation times; log-distance/ratios; Phase-B amplitude/repeatability/identifiability and promotion gates. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`mechanism compare` output row](outputs.md#mechanism-compare) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Timescale agreement is evidence, not mechanism identity. Read missing/contradictory/independence information before support labels.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli mechanism compare --eis-artifact output/quickstart/eis_fit.json --transient-artifact output/quickstart/transient/transient_results.json --output output/mechanism
```

### Common errors

Artifact mismatch, invalid hypothesis bindings or unavailable temporal/context/covariance evidence.

### Limitations

Legacy --calibration-results is chiefly context presence, not automatic independent validation. Phase-B route does not use every legacy option; keep config families separate.

[Back to command tree](#command-tree)

## mechanism trend

### Purpose

Recompute timescale records and trends from a manifest of EIS/transient pairs.

### When to use it

Study repeated conditions or sensor age.

### Syntax

```text
rust_electroanalysis_cli mechanism trend [OPTIONS] --manifest <PATH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--manifest` | `PathBuf` | Yes | Workflow-specific manifest listing records; paths resolved relative to manifest. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

TOML records containing paths to EIS and transient artifacts and declared condition/age identity.

### Internal workflow

CLI → manifest loader → read paired artifacts per record → extract/match timescales → regress supported trends → report..

### Scientific calculations

Trend summaries and timescale comparisons from replicate records. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`mechanism trend` output row](outputs.md#mechanism-trend) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Check record count, identity, covariate range and consistency of feature roles.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli mechanism trend --manifest mechanism_manifest.toml --output output/mechanism_trend
```

### Common errors

Missing record paths, schema/identity mismatch, too few trend records.

### Limitations

Does not consume a folder of mechanism reports as its main interface; repeated rows are not independent samples.

[Back to command tree](#command-tree)

## mechanism report

### Purpose

Render a human-readable summary of a mechanism artifact.

### When to use it

Review a saved result without refitting raw data.

### Syntax

```text
rust_electroanalysis_cli mechanism report [OPTIONS] --results <PATH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--results` | `PathBuf` | Yes | Saved typed result artifact to read, validate or render. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Typed mechanism_results.json.

### Internal workflow

CLI → typed reader → human_report → text destination..

### Scientific calculations

No new scientific fitting. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`mechanism report` output row](outputs.md#mechanism-report) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Read findings, assumptions and missing evidence in conjunction with JSON.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli mechanism report --results output/mechanism/mechanism_results.json --output output/mechanism/text.txt
```

### Common errors

Wrong artifact kind/version or unavailable output directory.

### Limitations

Legacy text summary is not the governed Phase-D public report bundle.

[Back to command tree](#command-tree)

## signal characterize

### Purpose

Measure time-series quality, noise, stability, drift, spikes and channel relationships.

### When to use it

Choose averaging/acquisition conditions or quantify signal evidence.

### Syntax

```text
rust_electroanalysis_cli signal characterize [OPTIONS] --input <INPUT> --channel <CHANNEL>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--input` | `PathBuf` | Yes | Raw input file; model decompose instead takes scientific ModelInput JSON. |
| `--channel` | `String` | Yes | Potential/measurement channel by logical name or source-header alias. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--metadata` | Absent (no CLI default) | Experiment/context TOML; align identity, events and environmental units. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--sheet` | Absent (no CLI default) | Exact compatible XLSX worksheet name; omitted uses provider automatic selection. Type: `Option<String>`. | Match the actual source layout. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

One time series, selected channel, optional event metadata and sheet; signal TOML controls windows/sampling.

### Internal workflow

CLI → canonical parser → seconds normalization → event windows → sampling checks → signal engines → artifact/tables/PNG..

### Scientific calculations

Statistics, Welch PSD/ASD, overlapping Allan variance, linear/Theil–Sen drift, Hampel spike flags, correlations. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`signal characterize` output row](outputs.md#signal-characterize) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Check actual window length and sampling diagnostics before interpreting spectra; missing outputs mean unavailable, not zero.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli signal characterize --input tests/fixtures/phase2/sensor.csv --channel E1/V --output output/signal
```

### Common errors

Irregular/duplicate/nonmonotonic time under strict policies, empty stable region, insufficient PSD/Allan samples.

### Limitations

Spikes are flagged, not a command to clean or rewrite raw data. Event exclusion may remove most of a short record.

[Back to command tree](#command-tree)

## signal compare

### Purpose

Compare signal summaries across manifest-listed raw measurements.

### When to use it

Compare replicates, conditions or sensor designs.

### Syntax

```text
rust_electroanalysis_cli signal compare [OPTIONS] --manifest <MANIFEST>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--manifest` | `PathBuf` | Yes | Workflow-specific manifest listing records; paths resolved relative to manifest. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

TOML schema_version=1, records with record_id, category, input, channel; paths relative to manifest.

### Internal workflow

CLI → signal comparison manifest → parse/analyze each raw record → comparison JSON/CSV/provenance..

### Scientific calculations

Same scalar noise/drift/spike summaries across records. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`signal compare` output row](outputs.md#signal-compare) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Match units, sampling, duration and preprocessing; inspect category and record count.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli signal compare --manifest signal_manifest.toml --output output/signal_compare
```

### Common errors

Empty manifest, wrong schema, per-record parse/analysis errors.

### Limitations

The manifest metadata field is accepted but the comparison engine passes no events to analysis; it does not reproduce metadata-based event exclusion. No per-record sheet selector.

[Back to command tree](#command-tree)

## signal residuals

### Purpose

Analyze residual structure in existing fit artifacts.

### When to use it

Check whether fitted errors retain systematic structure.

### Syntax

```text
rust_electroanalysis_cli signal residuals [OPTIONS]
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| None | — | No | See conditional input requirements below. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--transient-results` | Absent (no CLI default) | Existing transient results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--calibration-results` | Absent (no CLI default) | Existing calibration results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--eis-fit` | Absent (no CLI default) | Existing eis fit artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Optional transient, calibration and/or EIS artifacts; at least one is scientifically useful.

### Internal workflow

CLI → typed artifact readers → selected-fit residual extraction → time/frequency-domain residual summaries → JSON/text..

### Scientific calculations

Autocorrelation/PSD/spike summaries for time residuals and frequency-dependent EIS residual statistics. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`signal residuals` output row](outputs.md#signal-residuals) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Structured errors suggest model or noise assumptions need revision.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli signal residuals --eis-fit output/quickstart/eis_fit.json --transient-results output/quickstart/transient/transient_results.json --output output/residuals
```

### Common errors

Wrong artifact kind, no selected usable fit, invalid config.

### Limitations

No inputs yields an empty result rather than a useful analysis. Calibration residual coordinates are observation indices, not physical seconds; their PSD must not be interpreted as temporal frequency.

[Back to command tree](#command-tree)

## health baseline

### Purpose

Build a reference feature distribution from a manifest of analysis artifacts.

### When to use it

Establish a sensor/condition baseline before assessing new records.

### Syntax

```text
rust_electroanalysis_cli health baseline [OPTIONS] --manifest <MANIFEST>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--manifest` | `PathBuf` | Yes | Workflow-specific manifest listing records; paths resolved relative to manifest. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

TOML schema_version=1 records; each requires record_id and signal_results; optional other-domain artifacts and metadata.

### Internal workflow

CLI → manifest → domain features/context/lineage → baseline statistics → typed JSON..

### Scientific calculations

Location/spread and context-aware feature summaries. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`health baseline` output row](outputs.md#health-baseline) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Check record count, comparability and robust spread. Reusing one source under multiple IDs does not create independent evidence.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli health baseline --manifest health_manifest.toml --output output/health
```

### Common errors

Empty manifest, wrong schema/kind, unavailable artifact.

### Limitations

Insufficient records can lead to warnings/unavailable normalized features; no universal baseline transfer across matrices.

[Back to command tree](#command-tree)

## health assess

### Purpose

Assess sensor evidence against a baseline and configured rules/dimensions.

### When to use it

Detect changes requiring researcher investigation.

### Syntax

```text
rust_electroanalysis_cli health assess [OPTIONS] --signal-results <SIGNAL_RESULTS>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--signal-results` | `PathBuf` | Yes | Existing signal results artifact; optional context/evidence unless marked required. It is not a raw measurement file. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--transient-results` | Absent (no CLI default) | Existing transient results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--calibration-results` | Absent (no CLI default) | Existing calibration results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--eis-fit` | Absent (no CLI default) | Existing eis fit artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--mechanism-results` | Absent (no CLI default) | Existing mechanism results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--baseline` | Absent (no CLI default) | Stored health_baseline.json used for feature normalization/context. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--metadata` | Absent (no CLI default) | Experiment/context TOML; align identity, events and environmental units. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--phase-c-config` | Absent (no CLI default) | Explicit Phase-C dimension/evidence TOML; selects schema-4 health route. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--estimation-artifact` | Absent (no CLI default) | Phase-C state-estimation evidence. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--model-artifact` | Absent (no CLI default) | Phase-C compiled model analysis evidence. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--mechanism-artifact` | Absent (no CLI default) | Phase-C mechanism hypothesis evidence. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--lineage-catalog` | Absent (no CLI default) | Artifact lineage catalog for ancestry/acquisition-family resolution. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Required signal artifact; optional baseline/transient/calibration/EIS/mechanism/metadata. Phase-C uses --phase-c-config and optional estimation/model/mechanism artifacts and lineage catalog.

### Internal workflow

CLI → legacy feature/rule assessment or Phase-C dimension/evidence path → typed assessment + tables/text/plots..

### Scientific calculations

Relative deviations/robust z scores and rules; Phase-C dimension thresholds and gated causal evidence. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`health assess` output row](outputs.md#health-assess) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Read overall status, each dimension, evidence coverage, missing values and alternatives.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli health assess --signal-results output/signal/signal_results.json --baseline output/health/health_baseline.json --output output/health_assess
```

### Common errors

Context mismatch, absent baseline feature, invalid Phase-C thresholds/bindings/lineage.

### Limitations

Legacy route deliberately writes schema 3; Phase-C writes schema 4. A rule label is not definitive failure diagnosis.

[Back to command tree](#command-tree)

## health trend

### Purpose

Collect saved assessments and summarize feature trends.

### When to use it

Inspect repeated assessments, subject to the known slope defect below.

### Syntax

```text
rust_electroanalysis_cli health trend [OPTIONS] --manifest <MANIFEST>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--manifest` | `PathBuf` | Yes | Workflow-specific manifest listing records; paths resolved relative to manifest. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--baseline` | Absent (no CLI default) | Stored health_baseline.json used for feature normalization/context. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Trend TOML records reference saved assessment JSON and optional independent_value; optional baseline artifact.

### Internal workflow

CLI → trend manifest → saved assessments → raw trend points → summary JSON/CSV; runner does not call the trend plot helper..

### Scientific calculations

Intended feature trends; current summary regression instead uses independent_value versus record index, and spread uses independent_value. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`health trend` output row](outputs.md#health-trend) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Use raw points and recompute trends externally; do not interpret the exported summary slopes as feature drift.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli health trend --manifest health_trend.toml --baseline output/health/health_baseline.json --output output/health_trend
```

### Common errors

Unavailable assessment artifacts, invalid schema, inadequate finite records.

### Limitations

Default trends_filename collides with the CSV writer and overwrites JSON. Set export.trends_filename to health_trends.json in a complete health config. See the coverage report for both defects.

[Back to command tree](#command-tree)

## health report

### Purpose

Write a readable saved-assessment summary.

### When to use it

Review a health artifact without rebuilding evidence.

### Syntax

```text
rust_electroanalysis_cli health report [OPTIONS] --results <RESULTS>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--results` | `PathBuf` | Yes | Saved typed result artifact to read, validate or render. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Typed health assessment JSON.

### Internal workflow

CLI → read_artifact → human_report → text..

### Scientific calculations

No new measurements or inference. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`health report` output row](outputs.md#health-report) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Compare stated findings with evidence and alternatives.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli health report --results output/health_assess/health_assessment.json --output output/health_report.txt
```

### Common errors

Wrong schema/kind, invalid Phase-C structure, invalid destination.

### Limitations

Not equivalent to report render or physical validation.

[Back to command tree](#command-tree)

## estimate run

### Purpose

Estimate latent activity and response states with EKF or UKF.

### When to use it

Separate model-conditional activity, baseline and polarization trajectories.

### Syntax

```text
rust_electroanalysis_cli estimate run [OPTIONS] --input <INPUT> --metadata <METADATA> --channel <CHANNEL> --calibration-model <CALIBRATION_MODEL>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--input` | `PathBuf` | Yes | Raw input file; model decompose instead takes scientific ModelInput JSON. |
| `--metadata` | `PathBuf` | Yes | Experiment/context TOML; align identity, events and environmental units. |
| `--channel` | `String` | Yes | Potential/measurement channel by logical name or source-header alias. |
| `--calibration-model` | `PathBuf` | Yes | Stored calibration_model.json for equilibrium observation mapping. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--sheet` | Absent (no CLI default) | Exact compatible XLSX worksheet name; omitted uses provider automatic selection. Type: `Option<String>`. | Match the actual source layout. |
| `--signal-results` | Absent (no CLI default) | Existing signal results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--transient-results` | Absent (no CLI default) | Existing transient results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--calibration-results` | Absent (no CLI default) | Existing calibration results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--eis-fit` | Absent (no CLI default) | Existing eis fit artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--mechanism-results` | Absent (no CLI default) | Existing mechanism results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--health-baseline` | Absent (no CLI default) | Existing health baseline artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--health-assessment` | Absent (no CLI default) | Existing health assessment artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--filter` | Selected configuration | ekf or ukf; default from estimation config (ukf). Type: `Option<String>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--model` | Absent (no CLI default) | activity, activity_baseline, activity_baseline_polarization or custom; state-model selector, not a file. Type: `Option<String>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--seed` | Selected configuration | Unsigned reproducibility seed override. Type: `Option<u64>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |

### Input requirements

Time series + metadata + channel + stored calibration model; optional noise/prior/context artifacts and estimation config.

### Internal workflow

CLI → ingestion/timestamp policies → calibration/model/environment adaptation → initialization/observability → recursive filter → state artifact/tables/plots..

### Scientific calculations

Nonlinear prediction/update, innovation gate, covariance propagation, operational equilibrium checks. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`estimate run` output row](outputs.md#estimate-run) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Inspect observability, rejected updates, innovations/NIS, uncertainty and extrapolation before accepting state trajectories.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli estimate run --input output/simulation/simulation_measurements.csv --metadata simulation_metadata.toml --channel E1/V --calibration-model output/simulation/simulation_calibration_model.json --config estimation.toml --output output/estimation
```

### Common errors

Missing channel/calibration, ingestion threshold violation, unsupported custom binding, unobservable model, invalid covariance.

### Limitations

Offline model-dependent inference; voltage alone may not identify several latent states. Optional artifacts do not all become direct measurements.

[Back to command tree](#command-tree)

## estimate validate

### Purpose

Compare estimated states with a supplied truth trajectory.

### When to use it

Quantify synthetic or independently known recovery accuracy.

### Syntax

```text
rust_electroanalysis_cli estimate validate [OPTIONS] --results <RESULTS> --truth <TRUTH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--results` | `PathBuf` | Yes | Saved typed result artifact to read, validate or render. |
| `--truth` | `PathBuf` | Yes | Truth trajectory CSV; alignment follows stored config. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Typed state_estimation.json and truth CSV.

### Internal workflow

CLI → result/truth readers → explicit time alignment → state metrics → JSON/text..

### Scientific calculations

Bias/RMSE/coverage/convergence/step response where inputs permit. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`estimate validate` output row](outputs.md#estimate-validate) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Inspect aligned/unmatched counts and per-state evidence availability.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli estimate validate --results output/estimation/state_estimation.json --truth output/simulation/simulation_truth.csv --output output/state_validation
```

### Common errors

Malformed truth, duplicate/nonmonotonic truth or excessive alignment gap.

### Limitations

Truth independence and physical accuracy are external responsibilities. A missing metric is not a passing value.

[Back to command tree](#command-tree)

## estimate simulate

### Purpose

Generate synthetic observations, truth and a compatible calibration model.

### When to use it

Benchmark estimation or learn the workflow.

### Syntax

```text
rust_electroanalysis_cli estimate simulate [OPTIONS]
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| None | — | No | See conditional input requirements below. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--scenario` | Absent (no CLI default) | SimulationScenario TOML; omitted uses built-in synthetic scenario. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--seed` | Selected configuration | Unsigned reproducibility seed override. Type: `Option<u64>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |

### Input requirements

Optional TOML simulation scenario; default seeded scenario otherwise.

### Internal workflow

CLI → scenario/defaults and seed override → model simulation + noise/missing/outlier processes → JSON and CSV..

### Scientific calculations

Declared activity steps/pulses/ramps, baseline/polarization/sensitivity/environment and observation noise. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`estimate simulate` output row](outputs.md#estimate-simulate) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Compare known truth to later filtered estimates.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli estimate simulate --seed 42 --output output/simulation
```

### Common errors

Invalid interval/count/noise/domain or compiled model bindings.

### Limitations

Synthetic recovery validates implementation under those assumptions, not real sensors. No experiment TOML is emitted automatically.

[Back to command tree](#command-tree)

## estimate compare

### Purpose

Run requested filters on the same measurements and compare outputs.

### When to use it

Compare EKF/UKF consistency and behavior.

### Syntax

```text
rust_electroanalysis_cli estimate compare [OPTIONS] --input <INPUT> --metadata <METADATA> --channel <CHANNEL> --calibration-model <CALIBRATION_MODEL>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--input` | `PathBuf` | Yes | Raw input file; model decompose instead takes scientific ModelInput JSON. |
| `--metadata` | `PathBuf` | Yes | Experiment/context TOML; align identity, events and environmental units. |
| `--channel` | `String` | Yes | Potential/measurement channel by logical name or source-header alias. |
| `--calibration-model` | `PathBuf` | Yes | Stored calibration_model.json for equilibrium observation mapping. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--sheet` | Absent (no CLI default) | Exact compatible XLSX worksheet name; omitted uses provider automatic selection. Type: `Option<String>`. | Match the actual source layout. |
| `--filters` | ekf,ukf | Comma-separated filter names (ekf,ukf); default both. Type: `Option<String>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--config` | Workspace TOML / loader defaults | Select this workflow’s TOML; omitted uses its workspace config/default rules. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Same core raw/calibration inputs as estimate run; --filters comma-separated list, default ekf,ukf.

### Internal workflow

CLI → shared run preparation → each selected filter → comparison JSON/text (individual filter reports are computed internally but not exported)..

### Scientific calculations

Filter-state and diagnostic comparisons. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`estimate compare` output row](outputs.md#estimate-compare) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Agreement between filters is not independent truth.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli estimate compare --input output/simulation/simulation_measurements.csv --metadata simulation_metadata.toml --channel E1/V --calibration-model output/simulation/simulation_calibration_model.json --config estimation.toml --filters ekf,ukf --output output/comparison
```

### Common errors

No recognized filter names, or the same input/observability failures as run; unknown names are silently omitted if a recognized filter remains.

### Limitations

This CLI route does not expose run's optional evidence-artifact flags or model/seed overrides.

[Back to command tree](#command-tree)

## estimate report

### Purpose

Render a saved estimation report as text.

### When to use it

Review filter diagnostics without recomputing.

### Syntax

```text
rust_electroanalysis_cli estimate report [OPTIONS] --results <RESULTS>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--results` | `PathBuf` | Yes | Saved typed result artifact to read, validate or render. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Typed state estimation artifact.

### Internal workflow

CLI → typed reader → human_report → text..

### Scientific calculations

No refitting. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`estimate report` output row](outputs.md#estimate-report) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Review model/backend, observability, innovations and warnings.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli estimate report --results output/estimation/state_estimation.json --output output/estimation/report.txt
```

### Common errors

Wrong artifact version/kind, missing results.

### Limitations

Text is a summary; preserve JSON for covariance and full trajectories.

[Back to command tree](#command-tree)

## model validate

### Purpose

Compile a declared ISM model or evaluate a validation-study manifest.

### When to use it

Check structural model contracts; with --manifest evaluate a study.

### Syntax

```text
rust_electroanalysis_cli model validate [OPTIONS]
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| None | — | No | See conditional input requirements below. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--model` | Absent (no CLI default) | Model configuration TOML; omitted uses config/model.toml. Type: `Option<PathBuf>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--manifest` | Absent (no CLI default) | Workflow-specific manifest listing records; paths resolved relative to manifest. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Optional model TOML (default config/model.toml); alternatively JSON ValidationManifest.

### Internal workflow

CLI → model::validate → model compiler/registry, or model_validation::run → evaluate_manifest → artifacts..

### Scientific calculations

Definition/unit/dependency/ownership checks; manifest route evaluates explicitly supplied metrics/evidence. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`model validate` output row](outputs.md#model-validate) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Compilation is structural validity, not identified or physically validated parameters.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli model validate --output output/model_validate
```

### Common errors

Invalid units, bounds, dependency cycles, missing component bindings; manifest input/metric gaps.

### Limitations

When --manifest is present it takes the manifest route, even if --model is also supplied. Missing model path falls back to built-in definition.

[Back to command tree](#command-tree)

## model simulate

### Purpose

Evaluate a compiled ISM model over a deterministic built-in input sequence.

### When to use it

Inspect component/state behavior and output structure.

### Syntax

```text
rust_electroanalysis_cli model simulate [OPTIONS]
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| None | — | No | See conditional input requirements below. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--model` | Absent (no CLI default) | Model configuration TOML; omitted uses config/model.toml. Type: `Option<PathBuf>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--steps` | 10) | Positive number of simulated points. Type: `usize`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--dt-s` | 1.0) | Positive finite simulation interval in seconds. Type: `f64`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |

### Input requirements

Optional model TOML, positive steps and dt_s; no arbitrary scenario file option here.

### Internal workflow

CLI → compile → initialize parameters/states → evaluate then exact component transitions → analysis/tables/plots..

### Scientific calculations

Reduced-order equilibrium and declared dynamic components; named potential contributions. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`model simulate` output row](outputs.md#model-simulate) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Check contributions, validity/equilibrium status and identifiability report.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli model simulate --output output/model_simulate
```

### Common errors

Zero steps, nonpositive/nonfinite dt, invalid definition or incompatible required inputs.

### Limitations

Uses built-in inputs, not a fitted experiment. Default parameters are not sensor calibration.

[Back to command tree](#command-tree)

## model decompose

### Purpose

Evaluate named model contributions and residuals on explicit scientific inputs.

### When to use it

Compare a declared model with measured voltage.

### Syntax

```text
rust_electroanalysis_cli model decompose [OPTIONS]
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| None | — | No | See conditional input requirements below. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--model` | Absent (no CLI default) | Model configuration TOML; omitted uses config/model.toml. Type: `Option<PathBuf>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--input` | Absent (no CLI default) | Raw input file; model decompose instead takes scientific ModelInput JSON. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--measurement` | Absent (no CLI default) | Time-series measurement for model decomposition; needs explicit --input and metadata. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--metadata` | Absent (no CLI default) | Experiment/context TOML; align identity, events and environmental units. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--calibration-model` | Absent (no CLI default) | Stored calibration_model.json for equilibrium observation mapping. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--transient-results` | Absent (no CLI default) | Existing transient results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--eis-fit` | Absent (no CLI default) | Existing eis fit artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--signal-results` | Absent (no CLI default) | Existing signal results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--mechanism-results` | Absent (no CLI default) | Existing mechanism results artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--health-assessment` | Absent (no CLI default) | Existing health assessment artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

Optional model TOML; --input is ModelInput JSON object/array. Measurement+metadata requires explicit input; optional calibration/artifact context.

### Internal workflow

CLI → compile → optional calibration parameter adapter → validate auxiliary artifacts → attach measurements → propagate state → contributions/residual artifact..

### Scientific calculations

Sum voltage contributions, separate observation variance and unexplained residual; no automatic parameter optimization. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`model decompose` output row](outputs.md#model-decompose) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Use residual and evidence availability to decide what is explained; states without identifiability remain phenomenological.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli model decompose --output output/model_decompose
```

### Common errors

Measurement without explicit inputs, decreasing timestamps, wrong units/required input, artifact contract mismatch.

### Limitations

Without inputs uses one synthetic default point. Voltage does not automatically determine concentration. Auxiliary artifacts are often validated evidence, not automatic fitted parameter substitutions.

[Back to command tree](#command-tree)

## model report

### Purpose

Write a concise text summary of a model analysis.

### When to use it

Review simulation/decomposition results.

### Syntax

```text
rust_electroanalysis_cli model report [OPTIONS] --results <RESULTS>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--results` | `PathBuf` | Yes | Saved typed result artifact to read, validate or render. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--output` | See Outputs | Destination; file-versus-directory and default depend on this command (see Outputs). Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |

### Input requirements

model_analysis.json, not model_compilation.json.

### Internal workflow

CLI → typed ModelAnalysisReport → summary text..

### Scientific calculations

No new calculations. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`model report` output row](outputs.md#model-report) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Review model identity, point count, identifiability and evidence statements.

### Example

**EXAMPLE — executed in the audit workspace.** Requires the matching prepared inputs from the [workflow chapter](workflows.md); the exact setup and invocation were executed in the audit workspace.

```bash
rust_electroanalysis_cli model report --results output/model_simulate/model_analysis.json --output output/model_report.txt
```

### Common errors

Wrong artifact kind/schema or unwritable output.

### Limitations

Summary does not contain all contribution arrays; keep JSON and CSV.

[Back to command tree](#command-tree)

## report render

### Purpose

Render a governed Phase-D public bundle from frozen analysis artifacts.

### When to use it

Produce reproducible public tables/figures with evidence availability reasons.

### Syntax

```text
rust_electroanalysis_cli report render [OPTIONS] --mechanism <PATH> --health <PATH> --output-dir <PATH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--mechanism` | `PathBuf` | Yes | Frozen mechanism artifact for Phase-D rendering. |
| `--health` | `PathBuf` | Yes | Frozen health assessment artifact for Phase-D rendering. |
| `--output-dir` | `PathBuf` | Yes | Required dedicated bundle destination; ordinary analysis output behavior does not apply. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--lineage-catalog` | Absent (no CLI default) | Artifact lineage catalog for ancestry/acquisition-family resolution. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--eis` | Absent (no CLI default) | Existing eis artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--transient` | Absent (no CLI default) | Existing transient artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--calibration` | Absent (no CLI default) | Existing calibration artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--calibration-observations` | Absent (no CLI default) | Existing calibration observations artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--signal` | Absent (no CLI default) | Existing signal artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--estimation` | Absent (no CLI default) | Existing estimation artifact; optional context/evidence unless marked required. It is not a raw measurement file. Type: `Option<PathBuf>`. | Select a different input/context or destination explicitly. |
| `--model` | Absent (no CLI default) | Model selector or artifact/configuration path as described below. Type: `Option<PathBuf>`. | Change the scientific model, uncertainty effort or reproducibility setting intentionally. |
| `--format` | ReportFormatArg::All) | all (default), json or markdown; selects summary formats. Type: `ReportFormatArg`. | Select a different input/context or destination explicitly. |
| `--figures` | Absent (no CLI default) | all, none or comma-separated figure IDs; omitted uses source-dependent defaults. Type: `Option<String>`. | Select a different input/context or destination explicitly. |
| `--tables` | Absent (no CLI default) | all, none or comma-separated table IDs; omitted uses default tables. Type: `Option<String>`. | Select a different input/context or destination explicitly. |
| `--overwrite` | false | Replace a recognized, integrity-checked managed output bundle. Defaults false. Type: `bool`. | Select a different input/context or destination explicitly. |

### Input requirements

Required mechanism and health artifacts; optional source artifacts and lineage catalog; explicit destination.

### Internal workflow

Early dispatch → strict inputs/projection/lineage → output preflight → staged figures/tables/summary/manifest → publish bundle..

### Scientific calculations

Presentation of frozen results and evidence; no raw-data fitting or authority promotion. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`report render` output row](outputs.md#report-render) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Read unavailable entries/reasons, source identities and managed file manifest.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli report render --mechanism tests/fixtures/phase_d/base/mechanism.json --health tests/fixtures/phase_d/base/health.json --lineage-catalog tests/fixtures/phase_d/base/lineage_catalog.json --output-dir output/public_report --figures none
```

### Common errors

Wrong/legacy-incompatible artifacts, missing lineage/source dependencies, unknown figure/table ID, existing or modified managed output.

### Limitations

Format controls JSON/Markdown summaries, separately from figure/table selection. --overwrite is managed-bundle replacement, not arbitrary directory deletion.

[Back to command tree](#command-tree)

## validation run

### Purpose

Independently evaluate frozen mechanism/health artifacts against a declared cohort and protocol.

### When to use it

Run the Phase-E software/physical-claim validation boundary.

### Syntax

```text
rust_electroanalysis_cli validation run [OPTIONS] --protocol <PATH> --dataset <PATH> --output-dir <PATH>
```

### Required arguments

| Argument | Type | Required | Meaning |
|---|---|---|---|
| `--protocol` | `PathBuf` | Yes | Phase-E protocol TOML declaring endpoints, cohort rules and release claims. |
| `--dataset` | `PathBuf` | Yes | Phase-E schema-1 dataset manifest and referenced artifacts. |
| `--output-dir` | `PathBuf` | Yes | Required dedicated bundle destination; ordinary analysis output behavior does not apply. |

### Optional arguments

| Option | Default | Meaning | When to change it |
|---|---|---|---|
| `--overwrite` | false | Replace a recognized, integrity-checked managed output bundle. Defaults false. Type: `bool`. | Select a different input/context or destination explicitly. |

### Input requirements

Protocol TOML + dataset schema-1 JSON and its referenced immutable artifacts/labels/lineage; explicit destination.

### Internal workflow

Early dispatch → strict reader/protocol/partition/approval checks → endpoint and release-scope evaluation → staged report/tables/manifest..

### Scientific calculations

Cohort coverage/leakage, mechanism/health endpoint metrics and declared acceptance rules. See [scientific methods](scientific_methods.md).

### Outputs

See the complete [`validation run` output row](outputs.md#validation-run) for defaults, names, formats and downstream consumers. Optional plotting produces only the applicable figures.

### What to look for in the results

Inspect evaluated/excluded/unsupported cohorts and allowed claim level, not just process exit code.

### Example

**EXAMPLE — executed in the audit workspace.** Run from the repository root after building; output paths are intentionally separate.

```bash
rust_electroanalysis_cli validation run --protocol tests/fixtures/phase_e/protocol/software_valid.toml --dataset phase_e/dataset.json --output-dir output/validation
```

### Common errors

Invalid references/protocol, unsafe output, lineage conflicts, untrusted approval, managed-output mismatch.

### Limitations

No Phase-F REAL bootstrap/approval command. Software fixture success is not physical validation; public trust inputs do not authorize private-key access.

[Back to command tree](#command-tree)

## Plot target values

All four share the `plot` options and input/output rules above. These are positional selectors, not another command-parser level.

| Invocation (illustrative configuration-dependent recipe) | Purpose |
|---|---|
| `rust_electroanalysis_cli plot all --plot-config config/plotting.toml` | All configured workflows |
| `rust_electroanalysis_cli plot eis --plot-config config/plotting.toml` | EIS Nyquist/Bode jobs |
| `rust_electroanalysis_cli plot regular-plot --plot-config config/plotting.toml` | Regular/CHI time-series jobs; aliases regular, pb, pb-sensor, chi |
| `rust_electroanalysis_cli plot generic-plot --plot-config config/plotting.toml` | Generic numeric jobs |

## Command cheat sheet

| Goal | Command |
|---|---|
| Render configured electrochemical and generic figures. | [`plot`](#plot) |
| Fit one complex impedance spectrum to a specified or resolved circuit. | [`eis fit`](#eis-fit) |
| Perform a fit and write its durable EIS artifact. | [`eis export-fit`](#eis-export-fit) |
| Search a bounded family of equivalent circuits and rank candidate fits. | [`eis search`](#eis-search) |
| Segment event-driven potentiometric responses and fit relaxation models. | [`transient fit`](#transient-fit) |
| Create calibration observations from concentration-step experiments. | [`calibration extract`](#calibration-extract) |
| Fit an equilibrium potential-to-activity model to stored observations. | [`calibration fit`](#calibration-fit) |
| Evaluate a frozen calibration model against supplied observations. | [`calibration validate`](#calibration-validate) |
| Invert a stored calibration model for activity and available concentration. | [`calibration predict`](#calibration-predict) |
| Compare EIS and transient evidence, optionally through the explicit Phase-B hypothesis contract. | [`mechanism compare`](#mechanism-compare) |
| Recompute timescale records and trends from a manifest of EIS/transient pairs. | [`mechanism trend`](#mechanism-trend) |
| Render a human-readable summary of a mechanism artifact. | [`mechanism report`](#mechanism-report) |
| Measure time-series quality, noise, stability, drift, spikes and channel relationships. | [`signal characterize`](#signal-characterize) |
| Compare signal summaries across manifest-listed raw measurements. | [`signal compare`](#signal-compare) |
| Analyze residual structure in existing fit artifacts. | [`signal residuals`](#signal-residuals) |
| Build a reference feature distribution from a manifest of analysis artifacts. | [`health baseline`](#health-baseline) |
| Assess sensor evidence against a baseline and configured rules/dimensions. | [`health assess`](#health-assess) |
| Collect saved assessments and summarize feature trends. | [`health trend`](#health-trend) |
| Write a readable saved-assessment summary. | [`health report`](#health-report) |
| Estimate latent activity and response states with EKF or UKF. | [`estimate run`](#estimate-run) |
| Compare estimated states with a supplied truth trajectory. | [`estimate validate`](#estimate-validate) |
| Generate synthetic observations, truth and a compatible calibration model. | [`estimate simulate`](#estimate-simulate) |
| Run requested filters on the same measurements and compare outputs. | [`estimate compare`](#estimate-compare) |
| Render a saved estimation report as text. | [`estimate report`](#estimate-report) |
| Compile a declared ISM model or evaluate a validation-study manifest. | [`model validate`](#model-validate) |
| Evaluate a compiled ISM model over a deterministic built-in input sequence. | [`model simulate`](#model-simulate) |
| Evaluate named model contributions and residuals on explicit scientific inputs. | [`model decompose`](#model-decompose) |
| Write a concise text summary of a model analysis. | [`model report`](#model-report) |
| Render a governed Phase-D public bundle from frozen analysis artifacts. | [`report render`](#report-render) |
| Independently evaluate frozen mechanism/health artifacts against a declared cohort and protocol. | [`validation run`](#validation-run) |

## Runtime help snapshots

These are the exact help outputs of the audited build. They complement the explanations above and make parser-level defaults and allowed enum values directly reviewable.

<details>
<summary>Root --help</summary>

```text
Electrochemical data analysis and equivalent-circuit workflows

Usage: electroanalysis [OPTIONS] [COMMAND]

Commands:
  plot         Generate configured EIS, regular, and/or generic plots
  eis          Run an EIS fit or equivalent-circuit search
  transient    Analyze potentiometric transient responses around experimental events
  calibration  Extract, fit, validate, and use equilibrium potentiometric calibrations
  mechanism    Compare EIS-derived and transient-derived characteristic timescales
  signal       Characterize signal quality and residual structure
  health       Construct baselines and assess sensor health
  report       Render the certified Phase-D public scientific-output bundle
  validation   Independently validate frozen Phase-B/Phase-C artifacts against a declared cohort
  estimate     Estimate latent activity and sensor-response states from time-resolved measurements
  model        Validate, simulate, decompose, and report unified ISM models

Options:
      --plot <TARGET>
          Legacy plot selector (`all`, `eis`, `regular-plot`, or `generic`)

          Possible values:
          - all:          Generate EIS, regular, and generic plots
          - eis:          Generate EIS (Nyquist / Bode) plots only
          - regular-plot: Generate regular (Pb-sensor / CHI timeseries) plots only
          - generic-plot: Generate only the generic (`[[generic_plot]]`) plots

      --plot-config <PATH>
          Legacy plotting configuration override

      --search-eis <PATH>
          Legacy EIS-search target

      --search-config <PATH>
          Legacy search configuration override

      --search-output <PATH>
          Legacy search report output override

      --search-top <N>
          Legacy ranked-candidate limit

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

</details>

<details>
<summary>plot --help</summary>

```text
Generate configured EIS, regular, and/or generic plots

Usage: electroanalysis plot [OPTIONS] [TARGET]

Arguments:
  [TARGET]
          Plot category. Defaults to all configured plot workflows

          Possible values:
          - all:          Generate EIS, regular, and generic plots
          - eis:          Generate EIS (Nyquist / Bode) plots only
          - regular-plot: Generate regular (Pb-sensor / CHI timeseries) plots only
          - generic-plot: Generate only the generic (`[[generic_plot]]`) plots

          [default: all]

Options:
      --plot-config <PATH>
          Override the plotting TOML file

  -h, --help
          Print help (see a summary with '-h')
```

</details>

<details>
<summary>eis --help</summary>

```text
Run an EIS fit or equivalent-circuit search

Usage: electroanalysis eis <COMMAND>

Commands:
  fit         Fit one EIS data file with its resolved or explicitly supplied circuit
  search      Search one EIS file or all supported EIS files in a directory
  export-fit  Export a durable JSON artifact for one EIS fit

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>eis fit --help</summary>

```text
Fit one EIS data file with its resolved or explicitly supplied circuit

Usage: electroanalysis eis fit [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input CHI EIS file

Options:
      --sheet <NAME>          Canonical XLSX worksheet name. Without it, electrodata-io selects one compatible EIS worksheet or returns structured ambiguity
  -c, --circuit <EXPRESSION>  Circuit expression override, for example `R0-p(CPE1,R1)`
  -o, --output <PATH>         Write the fit report to this path instead of stdout
      --artifact <PATH>       Optional durable JSON artifact destination
      --report <PATH>         Optional human-readable artifact report destination
  -h, --help                  Print help
```

</details>

<details>
<summary>eis search --help</summary>

```text
Search one EIS file or all supported EIS files in a directory

Usage: electroanalysis eis search [OPTIONS] <INPUT>

Arguments:
  <INPUT>  EIS file or directory to search

Options:
      --sheet <NAME>          Apply this canonical XLSX worksheet selector to every input file
      --search-config <PATH>  Override the analysis TOML file
      --search-output <PATH>  Report file or output directory override
      --search-top <N>        Maximum number of ranked candidates to retain
  -h, --help                  Print help
```

</details>

<details>
<summary>eis export-fit --help</summary>

```text
Export a durable JSON artifact for one EIS fit

Usage: electroanalysis eis export-fit [OPTIONS] --artifact <PATH> <INPUT>

Arguments:
  <INPUT>

Options:
      --sheet <NAME>
  -c, --circuit <EXPRESSION>
      --artifact <PATH>
      --report <PATH>
  -h, --help                  Print help
```

</details>

<details>
<summary>transient --help</summary>

```text
Analyze potentiometric transient responses around experimental events

Usage: electroanalysis transient <COMMAND>

Commands:
  fit  Fit configured transient models to one or more eligible events

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>transient fit --help</summary>

```text
Fit configured transient models to one or more eligible events

Usage: electroanalysis transient fit [OPTIONS] --input <PATH> --metadata <PATH> --channel <NAME>

Options:
      --input <PATH>             Input time-series data file
      --metadata <PATH>          Experiment metadata TOML file
      --channel <NAME>           Measurement channel name, for example `E1/V` or `potential`
      --sheet <NAME>
      --config <PATH>            Transient configuration override
      --output <PATH>            Output directory for transient reports and figures
      --event-kind <EVENT_KIND>  Event category to analyze [default: concentration-step] [possible values: concentration-step, flow-change, temperature-change, ionic-strength-change, interferent-addition, flush-start, reading-start, flush-end, manual-annotation]
      --event-index <N>          Zero-based index among eligible events
      --model <MODEL>            Fit one model or all configured models [possible values: single, double, double-drift, stretched, all]
      --selection <SELECTION>    Information criterion used for model selection [possible values: aic, bic]
      --bootstrap <N>            Residual bootstrap iteration override
      --seed <N>                 Reproducibility seed override
  -h, --help                     Print help
```

</details>

<details>
<summary>calibration --help</summary>

```text
Extract, fit, validate, and use equilibrium potentiometric calibrations

Usage: electroanalysis calibration <COMMAND>

Commands:
  extract   Extract equilibrium calibration observations from concentration events
  fit       Fit configured calibration models to observations
  validate  Validate a stored calibration model against observations
  predict   Predict activity or concentration from a stored calibration model

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>calibration extract --help</summary>

```text
Extract equilibrium calibration observations from concentration events

Usage: electroanalysis calibration extract [OPTIONS] --input <PATH> --metadata <PATH> --channel <NAME>

Options:
      --input <PATH>
      --metadata <PATH>
      --channel <NAME>
      --sheet <NAME>
      --transient-results <PATH>
      --config <PATH>
      --output <PATH>
  -h, --help                      Print help
```

</details>

<details>
<summary>calibration fit --help</summary>

```text
Fit configured calibration models to observations

Usage: electroanalysis calibration fit [OPTIONS] --observations <PATH>

Options:
      --observations <PATH>
      --config <PATH>
      --output <PATH>
      --model <MODEL>
      --selection <CRITERION>
      --bootstrap <N>
      --seed <N>
  -h, --help                   Print help
```

</details>

<details>
<summary>calibration validate --help</summary>

```text
Validate a stored calibration model against observations

Usage: electroanalysis calibration validate [OPTIONS] --model <PATH> --observations <PATH>

Options:
      --model <PATH>
      --observations <PATH>
      --output <PATH>
  -h, --help                 Print help
```

</details>

<details>
<summary>calibration predict --help</summary>

```text
Predict activity or concentration from a stored calibration model

Usage: electroanalysis calibration predict [OPTIONS] --model <PATH>

Options:
      --model <PATH>
      --potential <V>
      --temperature <C>  Temperature in degrees Celsius; converted to kelvin internally
      --input <PATH>
      --channel <NAME>
      --output <PATH>
  -h, --help             Print help
```

</details>

<details>
<summary>mechanism --help</summary>

```text
Compare EIS-derived and transient-derived characteristic timescales

Usage: electroanalysis mechanism <COMMAND>

Commands:
  compare
  trend
  report

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>mechanism compare --help</summary>

```text
Usage: electroanalysis mechanism compare [OPTIONS] --eis-artifact <PATH> --transient-artifact <PATH>

Options:
      --eis-artifact <PATH>

      --transient-artifact <PATH>

      --calibration-results <PATH>

      --metadata <PATH>

      --config <PATH>

      --output <PATH>

      --mechanism-evidence-config <PATH>
          Phase-B evidence contract; intentionally separate from the legacy general mechanism configuration
      --state-estimation-artifact <PATH>

      --calibration-observations-artifact <PATH>

      --prior-mechanism-artifact <PATH>

  -h, --help
          Print help
```

</details>

<details>
<summary>mechanism trend --help</summary>

```text
Usage: electroanalysis mechanism trend [OPTIONS] --manifest <PATH>

Options:
      --manifest <PATH>
      --config <PATH>
      --output <PATH>
  -h, --help             Print help
```

</details>

<details>
<summary>mechanism report --help</summary>

```text
Usage: electroanalysis mechanism report [OPTIONS] --results <PATH>

Options:
      --results <PATH>
      --output <PATH>
  -h, --help            Print help
```

</details>

<details>
<summary>signal --help</summary>

```text
Characterize signal quality and residual structure

Usage: electroanalysis signal <COMMAND>

Commands:
  characterize
  compare
  residuals

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>signal characterize --help</summary>

```text
Usage: electroanalysis signal characterize [OPTIONS] --input <INPUT> --channel <CHANNEL>

Options:
      --input <INPUT>
      --metadata <METADATA>
      --channel <CHANNEL>
      --sheet <NAME>
      --config <CONFIG>
      --output <OUTPUT>
  -h, --help                 Print help
```

</details>

<details>
<summary>signal compare --help</summary>

```text
Usage: electroanalysis signal compare [OPTIONS] --manifest <MANIFEST>

Options:
      --manifest <MANIFEST>
      --config <CONFIG>
      --output <OUTPUT>
  -h, --help                 Print help
```

</details>

<details>
<summary>signal residuals --help</summary>

```text
Usage: electroanalysis signal residuals [OPTIONS]

Options:
      --transient-results <TRANSIENT_RESULTS>
      --calibration-results <CALIBRATION_RESULTS>
      --eis-fit <EIS_FIT>
      --output <OUTPUT>
      --config <CONFIG>
  -h, --help                                       Print help
```

</details>

<details>
<summary>health --help</summary>

```text
Construct baselines and assess sensor health

Usage: electroanalysis health <COMMAND>

Commands:
  baseline
  assess
  trend
  report

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>health baseline --help</summary>

```text
Usage: electroanalysis health baseline [OPTIONS] --manifest <MANIFEST>

Options:
      --manifest <MANIFEST>
      --config <CONFIG>
      --output <OUTPUT>
  -h, --help                 Print help
```

</details>

<details>
<summary>health assess --help</summary>

```text
Usage: electroanalysis health assess [OPTIONS] --signal-results <SIGNAL_RESULTS>

Options:
      --signal-results <SIGNAL_RESULTS>
      --transient-results <TRANSIENT_RESULTS>
      --calibration-results <CALIBRATION_RESULTS>
      --eis-fit <EIS_FIT>
      --mechanism-results <MECHANISM_RESULTS>
      --baseline <BASELINE>
      --metadata <METADATA>
      --config <CONFIG>
      --phase-c-config <PHASE_C_CONFIG>
      --estimation-artifact <ESTIMATION_ARTIFACT>
      --model-artifact <MODEL_ARTIFACT>
      --mechanism-artifact <MECHANISM_ARTIFACT>
      --lineage-catalog <LINEAGE_CATALOG>
      --output <OUTPUT>
  -h, --help                                       Print help
```

</details>

<details>
<summary>health trend --help</summary>

```text
Usage: electroanalysis health trend [OPTIONS] --manifest <MANIFEST>

Options:
      --manifest <MANIFEST>
      --baseline <BASELINE>
      --config <CONFIG>
      --output <OUTPUT>
  -h, --help                 Print help
```

</details>

<details>
<summary>health report --help</summary>

```text
Usage: electroanalysis health report [OPTIONS] --results <RESULTS>

Options:
      --results <RESULTS>
      --output <OUTPUT>
  -h, --help               Print help
```

</details>

<details>
<summary>report --help</summary>

```text
Render the certified Phase-D public scientific-output bundle

Usage: electroanalysis report <COMMAND>

Commands:
  render

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>report render --help</summary>

```text
Usage: electroanalysis report render [OPTIONS] --mechanism <PATH> --health <PATH> --output-dir <PATH>

Options:
      --mechanism <PATH>
      --health <PATH>
      --output-dir <PATH>
      --lineage-catalog <PATH>
      --eis <PATH>
      --transient <PATH>
      --calibration <PATH>
      --calibration-observations <PATH>
      --signal <PATH>
      --estimation <PATH>
      --model <PATH>
      --format <FORMAT>                  [default: all] [possible values: all, json, markdown]
      --figures <all|none|ID[,ID...]>
      --tables <all|none|ID[,ID...]>
      --overwrite
  -h, --help                             Print help
```

</details>

<details>
<summary>validation --help</summary>

```text
Independently validate frozen Phase-B/Phase-C artifacts against a declared cohort

Usage: electroanalysis validation <COMMAND>

Commands:
  run  Run the sole certified Phase-E artifact-only validation route

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>validation run --help</summary>

```text
Run the sole certified Phase-E artifact-only validation route

Usage: electroanalysis validation run [OPTIONS] --protocol <PATH> --dataset <PATH> --output-dir <PATH>

Options:
      --protocol <PATH>
      --dataset <PATH>
      --output-dir <PATH>
      --overwrite
  -h, --help               Print help
```

</details>

<details>
<summary>estimate --help</summary>

```text
Estimate latent activity and sensor-response states from time-resolved measurements

Usage: electroanalysis estimate <COMMAND>

Commands:
  run
  validate
  simulate
  compare
  report

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>estimate run --help</summary>

```text
Usage: electroanalysis estimate run [OPTIONS] --input <INPUT> --metadata <METADATA> --channel <CHANNEL> --calibration-model <CALIBRATION_MODEL>

Options:
      --input <INPUT>
      --metadata <METADATA>
      --channel <CHANNEL>
      --sheet <NAME>
      --calibration-model <CALIBRATION_MODEL>
      --signal-results <SIGNAL_RESULTS>
      --transient-results <TRANSIENT_RESULTS>
      --calibration-results <CALIBRATION_RESULTS>
      --eis-fit <EIS_FIT>
      --mechanism-results <MECHANISM_RESULTS>
      --health-baseline <HEALTH_BASELINE>
      --health-assessment <HEALTH_ASSESSMENT>
      --config <CONFIG>
      --output <OUTPUT>
      --filter <FILTER>
      --model <MODEL>
      --seed <SEED>
  -h, --help                                       Print help
```

</details>

<details>
<summary>estimate validate --help</summary>

```text
Usage: electroanalysis estimate validate [OPTIONS] --results <RESULTS> --truth <TRUTH>

Options:
      --results <RESULTS>
      --truth <TRUTH>
      --output <OUTPUT>
  -h, --help               Print help
```

</details>

<details>
<summary>estimate simulate --help</summary>

```text
Usage: electroanalysis estimate simulate [OPTIONS]

Options:
      --scenario <SCENARIO>
      --output <OUTPUT>
      --seed <SEED>
  -h, --help                 Print help
```

</details>

<details>
<summary>estimate compare --help</summary>

```text
Usage: electroanalysis estimate compare [OPTIONS] --input <INPUT> --metadata <METADATA> --channel <CHANNEL> --calibration-model <CALIBRATION_MODEL>

Options:
      --input <INPUT>
      --metadata <METADATA>
      --channel <CHANNEL>
      --sheet <NAME>
      --calibration-model <CALIBRATION_MODEL>
      --filters <FILTERS>
      --config <CONFIG>
      --output <OUTPUT>
  -h, --help                                   Print help
```

</details>

<details>
<summary>estimate report --help</summary>

```text
Usage: electroanalysis estimate report [OPTIONS] --results <RESULTS>

Options:
      --results <RESULTS>
      --output <OUTPUT>
  -h, --help               Print help
```

</details>

<details>
<summary>model --help</summary>

```text
Validate, simulate, decompose, and report unified ISM models

Usage: electroanalysis model <COMMAND>

Commands:
  validate
  simulate
  decompose
  report

Options:
  -h, --help  Print help
```

</details>

<details>
<summary>model validate --help</summary>

```text
Usage: electroanalysis model validate [OPTIONS]

Options:
      --model <MODEL>
      --manifest <MANIFEST>
      --output <OUTPUT>
  -h, --help                 Print help
```

</details>

<details>
<summary>model simulate --help</summary>

```text
Usage: electroanalysis model simulate [OPTIONS]

Options:
      --model <MODEL>
      --output <OUTPUT>
      --steps <STEPS>    [default: 10]
      --dt-s <DT_S>      [default: 1]
  -h, --help             Print help
```

</details>

<details>
<summary>model decompose --help</summary>

```text
Usage: electroanalysis model decompose [OPTIONS]

Options:
      --model <MODEL>

      --input <INPUT>
          JSON `ModelInput` records; this is the deterministic measurement adapter
      --measurement <MEASUREMENT>

      --metadata <METADATA>

      --calibration-model <CALIBRATION_MODEL>

      --transient-results <TRANSIENT_RESULTS>

      --eis-fit <EIS_FIT>

      --signal-results <SIGNAL_RESULTS>

      --mechanism-results <MECHANISM_RESULTS>

      --health-assessment <HEALTH_ASSESSMENT>

      --output <OUTPUT>

  -h, --help
          Print help
```

</details>

<details>
<summary>model report --help</summary>

```text
Usage: electroanalysis model report [OPTIONS] --results <RESULTS>

Options:
      --results <RESULTS>
      --output <OUTPUT>
  -h, --help               Print help
```

</details>
