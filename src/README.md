# src module map (CLI-only)

- `main.rs`: CLI entrypoint and command dispatch.
- `cli.rs`: clap derive command tree, validation, and legacy-flag normalization.
- `domain/`: scientific measurements, experiment metadata, provenance,
  diagnostics, and typed errors shared across workflows.
- `fitting/`: stable façade for the scientific circuit-fit pipeline.
- `potentiometry/`: event segmentation, constrained transient models, fitting,
  diagnostics, model selection, and typed transient errors.
- `transient_config.rs`: independent transient TOML schema, validation, and
  CLI override resolution.
- `results/`: named result structures, including `CircuitFitResult` and the
  serializable transient report types.
- `runners/`: thin plot, fit, search, and transient workflow boundaries.
- `reporting/`: certified Phase-D public-summary, Markdown, CSV, SVG/PNG,
  lineage-presentation, and atomic-publication components. These modules only
  project validated serialized artifacts and do not perform scientific
  reassessment.
- `report_config.rs`: closed format, figure/table selection, and render-option
  contracts for `electroanalysis report render`.
- `workspace.rs`: workspace bootstrap and TOML config lifecycle.
- `plot_config.rs`: plotting TOML schema/load/migration/resolution.
- `search_config.rs`: analysis TOML schema/load/validation.
- `plot_runner.rs`: EIS/regular/generic plotting workflows.
- `search_runner.rs`: ECM search workflow and exports.
- `data_file/`: CHI/generic file parsing, diagnostics, and adapters into the
  existing plotting data container.
- `impedance/`: circuit models, fitting, scoring, and evolution.
- `plottings/`: rendering backends and plot styles.

The transient runner loads an `ElectrochemicalExperiment`, delegates all
scientific work to `potentiometry`, exports `results::transient`, and uses
`plottings::transient_plot` only as an adapter to the existing `PlotSeries`
and publication renderer. No transient-fitting logic is placed in the CLI or
renderer.

The codebase is intentionally CLI-focused; GUI/Tauri bridge modules are removed.
Scientific equations and ECM evolution remain in `impedance/`; runners only
coordinate existing modules. `data_file/`, `impedance/`, and `plottings/` are
preserved as the implementation subsystems. `domain/` does not depend on the
plotting renderer.

The certified public-output entry point is `electroanalysis report render`.
It requires mechanism and health artifacts plus an output directory; lineage,
EIS, transient, paired calibration inputs, signal, estimation, and model
artifacts are optional. Format selection controls JSON and Markdown documents,
while table and figure selections independently accept `all`, `none`, or a
closed comma-separated ID list. Successful publication always includes the
render manifest and uses private staging followed by an atomic directory
rename. Existing scientific commands and legacy plotting paths remain separate
and retain their established behavior.
