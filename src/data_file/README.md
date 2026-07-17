# data_file subsystem

`data_file` converts raw instrument exports into backend plotting/search data structures.

## Files

- `chi_file.rs`
  - Parses CHI-like text/CSV sources.
  - Produces `ElectrochemData` for regular plots and `EISData` for EIS/search workflows.
  - Extracts metadata used in labels and report outputs.
- `data_op.rs`
  - Defines generic `PlotData` and selection helpers (`PointSelection`).
  - Bridges domain datasets into plotting-ready series.
- `value_transform.rs`
  - Resolves axis transforms (none, log, linear, legacy negative-log).
  - Builds transform terms used in regression annotation equations.

## Integration points

- Consumed by `plot_runner.rs` and `search_runner.rs`.
- Type exports are re-exported from `src/data_file/lib.rs` and `src/lib.rs`.

Parsing failures use the typed `DataParsingError` rather than stringly typed
results. Fit-related errors are preserved through the error chain when a
parsed EIS dataset is passed to the impedance fitter.
