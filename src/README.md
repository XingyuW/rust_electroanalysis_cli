# src module map (CLI-only)

- `main.rs`: CLI entrypoint and command dispatch.
- `cli.rs`: clap derive command tree, validation, and legacy-flag normalization.
- `domain/`: scientific measurements, experiment metadata, provenance,
  diagnostics, and typed errors shared across workflows.
- `fitting/`: stable façade for the scientific circuit-fit pipeline.
- `results/`: named result structures, including `CircuitFitResult`.
- `runners/`: thin plot, fit, and search workflow boundaries.
- `workspace.rs`: workspace bootstrap and TOML config lifecycle.
- `plot_config.rs`: plotting TOML schema/load/migration/resolution.
- `search_config.rs`: analysis TOML schema/load/validation.
- `plot_runner.rs`: EIS/regular/generic plotting workflows.
- `search_runner.rs`: ECM search workflow and exports.
- `data_file/`: CHI/generic file parsing, diagnostics, and adapters into the
  existing plotting data container.
- `impedance/`: circuit models, fitting, scoring, and evolution.
- `plottings/`: rendering backends and plot styles.

The codebase is intentionally CLI-focused; GUI/Tauri bridge modules are removed.
Scientific equations and ECM evolution remain in `impedance/`; runners only
coordinate existing modules. `data_file/`, `impedance/`, and `plottings/` are
preserved as the implementation subsystems. `domain/` does not depend on the
plotting renderer.
