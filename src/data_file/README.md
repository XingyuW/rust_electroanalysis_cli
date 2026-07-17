# data_file subsystem

`data_file` converts raw instrument exports into backend plotting/search data structures.

## Files

- `chi_file.rs`
  - Parses CHI-like text/CSV sources.
  - Produces `ElectrochemData` for regular plots and `EISData` for EIS/search workflows.
  - Extracts metadata used in labels and report outputs.
- `measurement_parser.rs`
  - Parses CHI-style or generic time-series text into
    `MultiChannelMeasurement`.
  - Returns `MeasurementParseResult` with explicit `ParseDiagnostics`.
- `measurement_adapter.rs`
  - Projects scientific measurements into the existing `PlotData` type.
  - Keeps missing scientific values out of rendered point pairs without
    changing the source measurement.
- `data_op.rs`
  - Defines generic `PlotData` and selection helpers (`PointSelection`).
  - Bridges domain datasets into plotting-ready series.
- `value_transform.rs`
  - Resolves axis transforms (none, log, linear, legacy negative-log).
  - Builds transform terms used in regression annotation equations.

## Integration points

- Consumed by `plot_runner.rs` and `search_runner.rs`.
- Type exports are re-exported from `src/data_file/lib.rs` and `src/lib.rs`.

The existing `ElectrochemData`, `EISData`, and plotting conversions remain
available. New callers can use `parse_measurement_file` or
`ElectrochemData::to_multi_channel_measurement` incrementally.

Parsing failures use the typed `DataParsingError` rather than stringly typed
results. Fit-related errors are preserved through the error chain when a
parsed EIS dataset is passed to the impedance fitter.
