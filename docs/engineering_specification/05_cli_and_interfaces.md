# 05 — CLI & Interfaces

**Identifier:** `DOC-05`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## 1. Binary Name

The installed binary is named `electroanalysis` (configured in `#[command(name = "electroanalysis")]`).

## 2. Command Map

```
electroanalysis
├── plot [TARGET] [--plot-config PATH]          # Generate plots
├── eis
│   ├── fit INPUT [-c EXPR] [-o PATH] [--artifact PATH] [--report PATH]   # Fit ECM
│   ├── search INPUT [--search-config PATH] [--search-output PATH] [--search-top N]  # ECM search
│   └── export-fit INPUT [-c EXPR] --artifact PATH [--report PATH]        # Export fit artifact
├── transient
│   └── fit --input PATH --metadata PATH --channel NAME [--sheet NAME] [--config PATH]
│            [--output PATH] [--event-kind KIND] [--event-index N] [--model MODEL]
│            [--selection CRITERION] [--bootstrap N] [--seed N]
├── calibration
│   ├── extract --input PATH --metadata PATH --channel NAME [--sheet NAME]
│   │           [--transient-results PATH] [--config PATH] [--output PATH]
│   ├── fit --observations PATH [--config PATH] [--output PATH]
│   │        [--model MODEL] [--selection CRITERION] [--bootstrap N] [--seed N]
│   ├── validate --model PATH --observations PATH [--output PATH]
│   └── predict --model PATH [--potential V] [--temperature C]
│               [--input PATH --channel NAME] [--output PATH]
├── mechanism
│   ├── compare --eis-fit PATH --transient-results PATH [--calibration-results PATH]
│   │           [--metadata PATH] [--config PATH] [--output PATH]
│   ├── trend --manifest PATH [--config PATH] [--output PATH]
│   └── report --results PATH [--output PATH]
├── signal
│   ├── characterize --input PATH [--metadata PATH] --channel NAME [--sheet NAME]
│   │                 [--config PATH] [--output PATH]
│   ├── compare --manifest PATH [--config PATH] [--output PATH]
│   └── residuals [--transient-results PATH] [--calibration-results PATH]
│                 [--eis-fit PATH] [--config PATH] [--output PATH]
├── health
│   ├── baseline --manifest PATH [--config PATH] [--output PATH]
│   ├── assess --signal-results PATH [--transient-results PATH] [--calibration-results PATH]
│   │          [--eis-fit PATH] [--mechanism-results PATH] [--baseline PATH]
│   │          [--metadata PATH] [--config PATH] [--output PATH]
│   ├── trend --manifest PATH [--baseline PATH] [--config PATH] [--output PATH]
│   └── report --results PATH [--output PATH]
└── estimate
    ├── run --input PATH --metadata PATH --channel NAME --calibration-model PATH
    │       [--sheet NAME] [--signal-results PATH] [--transient-results PATH]
    │       [--calibration-results PATH] [--eis-fit PATH] [--mechanism-results PATH]
    │       [--health-baseline PATH] [--health-assessment PATH] [--config PATH]
    │       [--output PATH] [--filter NAME] [--model NAME] [--seed N]
    ├── validate --results PATH --truth PATH [--output PATH]
    ├── simulate [--scenario PATH] [--output PATH] [--seed N]
    ├── compare --input PATH --metadata PATH --channel NAME --calibration-model PATH
    │           [--sheet NAME] [--filters NAME] [--config PATH] [--output PATH]
    └── report --results PATH [--output PATH]
```

## 3. Detailed Command Specifications

### 3.1 `plot`

- **Purpose**: Generate EIS, regular (CHI), and/or generic plots.
- **Arguments**:
  - `TARGET` (positional, optional): `all` (default), `eis`, `regular-plot` (alias: `pb`, `pb-sensor`, `chi`), `generic-plot`
  - `--plot-config PATH` (alias: `--config`): Override plotting TOML path
- **Legacy flags**: `--plot TARGET`, `--plot-config PATH` (normalized to structured form)
- **Output**: PNG figures in configured output directory
- **Exit**: 0 on success; non-zero with stderr message on error

### 3.2 `eis fit`

- **Purpose**: Fit one EIS file with resolved or specified circuit model.
- **Arguments**:
  - `INPUT` (positional): EIS data file path
  - `-c/--circuit/--model EXPRESSION`: Circuit expression override (e.g., `R0-p(CPE1,R1)`)
  - `-o/--output PATH`: Report output path (default: stdout)
  - `--artifact PATH`: Durable JSON artifact destination
  - `--report PATH`: Human-readable artifact report path

### 3.3 `eis search`

- **Purpose**: ECM search on a file or directory.
- **Arguments**:
  - `INPUT` (positional): EIS file or directory
  - `--search-config/--config PATH`: Analysis TOML override
  - `--search-output PATH`: Report/output directory override
  - `--search-top N`: Max ranked candidates (>0)
- **Validation**: `--search-top 0` is rejected

### 3.4 `eis export-fit`

- **Purpose**: Export durable JSON artifact for one EIS fit (no stdout report).
- **Arguments**: Same as `eis fit` but `--artifact` is required.

### 3.5 `transient fit`

- **Purpose**: Fit transient models to potentiometric event responses.
- **Required**: `--input`, `--metadata`, `--channel`
- **Optional**: `--sheet`, `--config`, `--output`, `--event-kind` (default: `concentration-step`), `--event-index`, `--model` (single/double/double-drift/stretched/all), `--selection` (aic/bic), `--bootstrap`, `--seed`
- **Event kinds**: concentration-step, flow-change, temperature-change, ionic-strength-change, interferent-addition, flush-start, reading-start, flush-end, manual-annotation

### 3.6 `calibration` subcommands

- **`extract`**: Extract equilibrium observations from concentration events. Requires `--input`, `--metadata`, `--channel`.
- **`fit`**: Fit calibration models to observations. Requires `--observations`. Models: Nernst, Nicolsky-Eisenman, Conductivity-empirical.
- **`fit --model` accepted values/aliases**: `nernst` (`linear`), `nicolsky-eisenman` (`nicolsky_eisenman`, `ne`), `conductivity-empirical` (`conductivity_empirical`, `empirical-conductivity`), `all`.
- **`validate`**: Validate stored model against observations. Requires `--model`, `--observations`.
- **`predict`**: Predict activity/concentration. Requires `--model` and either `--potential` or `--input`+`--channel` (mutually exclusive). Temperature in °C, converted to K.

### 3.7 `mechanism`

- **`compare`**: Compare EIS-derived and transient-derived timescales. Requires `--eis-fit`, `--transient-results`.
- **`trend`**: Trend analysis from manifest. Requires `--manifest`.
- **`report`**: Generate report from results. Requires `--results`.

### 3.8 `signal`

- **`characterize`**: Characterize signal quality. Requires `--input`, `--channel`.
- **`compare`**: Compare signals from manifest. Requires `--manifest`.
- **`residuals`**: Analyze fit residuals. Optional inputs from transient, calibration, EIS.

### 3.9 `health`

- **`baseline`**: Build baseline from manifest. Requires `--manifest`.
- **`assess`**: Assess sensor health. Requires `--signal-results`. Optional inputs from all other workflows.
- **`trend`**: Trend health assessments. Requires `--manifest`.
- **`report`**: Generate health report. Requires `--results`.

### 3.10 `estimate`

- **`run`**: Run state estimation. Requires `--input`, `--metadata`, `--channel`, `--calibration-model`.
- **`validate`**: Validate against truth. Requires `--results`, `--truth`.
- **`simulate`**: Simulate estimation scenario. Optional `--scenario`, `--output`, `--seed`.
- **`compare`**: Compare filter configurations. Requires `--input`, `--metadata`, `--channel`, `--calibration-model`.
- **`report`**: Generate estimation report. Requires `--results`.

## 4. Configuration Path Resolution

All configuration paths are resolved relative to the workspace root (current directory). Legacy config file names are checked when the default path doesn't exist:
- `plot_config.toml` → `config/plotting.toml`
- `ecm_search.toml` → `config/analysis.toml`
- `circuit_models.toml` → `config/parsing.toml`

## 5. Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (including help/version display) |
| 1 | Application error (printed to stderr) |

## 6. Legacy Flag Compatibility

The system supports flat legacy flags (`--plot`, `--search-eis`, `--search-config`, `--search-output`, `--search-top`, `--plot-config`). These are normalized into `CommandSpec` values. Mixed structured+legacy invocations are rejected with a clear error.

## 7. CLI Test Coverage

| Test | File | Coverage |
|------|------|----------|
| `structured_plot_command_defaults_to_all` | `cli.rs` test | ✅ |
| `legacy_plot_flags_normalize_to_structured_command` | `cli.rs` test | ✅ |
| `structured_search_preserves_all_search_overrides` | `cli.rs` test | ✅ |
| `invalid_legacy_plot_search_combination_is_clear` | `cli.rs` test | ✅ |
| `fit_command_exposes_named_fit_options` | `cli.rs` test | ✅ |
| End-to-end binary tests | `tests/phase0_regression.rs` | ✅ |
