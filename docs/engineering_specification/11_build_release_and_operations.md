# 11 — Build, Release & Operations

**Identifier:** `DOC-11`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## 1. Supported Rust Toolchain

- **Channel**: Stable (specified in CI via `dtolnay/rust-toolchain@stable`)
- **Edition**: 2024 (specified in `Cargo.toml`)
- **Components**: rustfmt, clippy (required for CI)

## 2. Build Commands

| Action | Command |
|--------|---------|
| Development build | `cargo build` |
| Release build | `cargo build --release` |
| Check (no codegen) | `cargo check` |
| Format | `cargo fmt` |
| Format check | `cargo fmt --check` |
| Lint | `cargo clippy` |
| Strict lint | `cargo clippy --all-targets --all-features -- -D warnings` |

## 3. Development Commands

| Action | Command |
|--------|---------|
| Run all tests | `cargo test --all` |
| Run specific test | `cargo test <name>` |
| Run with backtrace | `RUST_BACKTRACE=1 cargo run -- <args>` |

## 4. Installation

The binary is built from source:
```sh
git clone <repository>
cd rust_electroanalysis_cli
cargo build --release
# Binary at: target/release/rust_electroanalysis_cli
```

No pre-built binaries, package managers, or install scripts are currently provided.

## 5. Platform Assumptions

- **Tested**: Linux (ubuntu-latest), macOS (macos-latest)
- **Not tested**: Windows
- **Native dependencies**: None declared (pure Rust except for system libraries used by plotters font rendering)
- **Architecture**: x86_64 (CI default); no ARM-specific config

## 6. Environment Variables

| Variable | Purpose | Required |
|----------|---------|----------|
| `GIT_COMMIT` | Embedded in provenance at build time | No |
| `RUST_BACKTRACE` | Enables panic backtraces | No |

## 7. Release Build Configuration

From `Cargo.toml` `[profile.release]`:

| Setting | Value | Effect |
|---------|-------|--------|
| `opt-level` | 3 | Maximum optimization |
| `panic` | "abort" | Smaller binary, immediate abort on panic |
| `codegen-units` | 1 | Better optimization, slower compilation |
| `lto` | "fat" | Full link-time optimization |
| `strip` | "symbols" | Strip debug symbols |
| `debug` | false | No debug info |

## 8. Versioning

- Current version: `0.1.0` (in `Cargo.toml`)
- Version is embedded in provenance records via `env!("CARGO_PKG_VERSION")`
- No changelog or release notes file exists

## 9. Packaging

No packaging configuration (`.deb`, `.rpm`, Homebrew, etc.) exists. Distribution is source-only.

## 10. CI/CD (`.github/workflows/ci.yml`)

- **Triggers**: Every push and pull request
- **OS Matrix**: ubuntu-latest, macos-latest
- **Caching**: `~/.cargo/registry`, `~/.cargo/git`, `target` keyed by `Cargo.lock` hash
- **Steps**:
  1. Checkout (`actions/checkout@v4`)
  2. Cache (`actions/cache@v4`)
  3. Install Rust (`dtolnay/rust-toolchain@stable` with rustfmt, clippy)
  4. `cargo fmt --check`
  5. `cargo clippy --all-targets --all-features -- -D warnings`
  6. `cargo test --all`
  7. `cargo build --release`
- **No deployment step**: CI only validates; no artifact uploads or releases

## 11. Reproducibility

- **Deterministic builds**: Not guaranteed (Cargo.lock pins versions, but no `--frozen` flag)
- **Provenance tracking**: SHA-256 of input and config files + timestamp
- **No Docker/container configuration** exists
- **No Nix flake or Guix package** exists

## 12. Workspace Layout

```
<project_root>/
├── config/          # TOML configuration files (auto-generated defaults)
├── data/            # Input data files (user-provided)
├── output/          # Generated figures and reports
├── logs/            # Log files (currently unused by code)
├── src/             # Rust source code (~164 files)
├── tests/           # Integration tests
├── Cargo.toml       # Manifest
├── Cargo.lock       # Lock file
└── README.md        # User documentation
```

## 13. Expected Output Artifacts

The inventory below is derived from runner write paths and export-config defaults.

| Workflow group | Default output root | Artifacts |
|----------------|---------------------|-----------|
| Plot (`WF-001..WF-003`) | Job-specific `output_dir` from `plot` config | Base-path rendering emits `.svg` and `.png` pairs; EIS plotting additionally emits `<base>_fit_report.txt`. |
| EIS fit/export (`WF-004`, `WF-006`) | No implicit directory; caller-specified paths | Text report to stdout or `--output`; optional JSON artifact (`--artifact`) and text artifact report (`--report`). |
| ECM search (`WF-005`) | Report defaults beside input; optional configured plot directory | `<stem>_ecm_search.txt` + `<stem>_ecm_search.csv`; optional top-N `.svg/.png` ranked/overlay plot sets and optional combined overlay for multi-input search. |
| Transient (`WF-007`) | `output/` | `transient_results.json`, `transient_features.csv`, `transient_model_comparison.csv`, `transient_report.txt`, plus optional per-event `.svg/.png` plots when plotting is enabled. |
| Calibration extract/fit/validate (`WF-008..WF-010`) | `output/calibration` | `calibration_observations.json` (extract), `calibration_model.json`, `calibration_results.json`, `calibration_summary.csv`, `calibration_residuals.csv`, `calibration_validation.csv`, `calibration_report.txt` (fit), `calibration_validation_results.json`, `calibration_validation.csv`, `calibration_validation_report.txt` (validate), optional `.svg/.png` calibration figures (fit). |
| Calibration predict (`WF-011`) | Workspace root by default | `prediction.json` by default, or CSV table when output extension is `.csv`. |
| Mechanism (`WF-012..WF-013`) | `output/mechanism` | `mechanism_results.json`, `characteristic_timescales.csv`, `timescale_comparisons.csv`, `mechanism_trends.csv`, `mechanism_report.txt`, and conditional `timescale_map.{svg,png}` / `timescale_ratio.{svg,png}`. |
| Signal characterize (`WF-014`) | `output/signal` | `signal_results.json`, `signal_summary.csv`, `signal_psd.csv`, `signal_allan.csv`, `signal_drift.csv`, `signal_spikes.csv`, `signal_correlations.csv`, `signal_report.txt`, and optional PNG plots when enabled. |
| Signal compare/residuals (`WF-015..WF-016`) | `output/signal_comparison`, `output/residual_analysis` | `signal_comparison_results.json`, `signal_comparison.csv`, `signal_comparison_provenance.json`; `residual_analysis_results.json`, `residual_analysis_report.txt`. |
| Health baseline/assess/trend (`WF-017..WF-019`) | `output/health`, `output/health_trend` | `health_baseline.json`; `health_assessment.json`, `health_features.csv`, `health_findings.csv`, `health_report.txt`, optional `health_feature_deviations.png`; trend JSON at `trends_filename` (default `health_trends.csv`) plus fixed `health_trends.csv`. |
| Estimation run (`WF-020`) | `output/estimation` | `state_estimation.json`, `state_diagnostics.json`, `state_validation.json`, `state_estimates.csv`, `state_innovations.csv`, `state_estimation_report.txt`, plus optional PNG plots when enabled. |
| Estimation validate/simulate/compare (`WF-021..WF-023`) | `output/estimation_validation`, `output/estimation_simulation`, `output/estimation_comparison` | `state_validation.json`, `state_validation_report.txt`; `simulation.json`, `simulation_calibration_model.json`, `simulation_measurements.csv`, `simulation_truth.csv`; `state_filter_comparison.json`, `state_filter_comparison_report.txt`. |

For per-workflow naming patterns and suffix-level plot outputs, see `DOC-06` "Workflow Output Artifact Matrix (Code-Verified)".
