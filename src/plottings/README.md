# plottings subsystem

`plottings` contains domain plot defaults and shared rendering code.

## Files

- `plotting.rs`
  - Core renderer abstraction and `PublicationConfig` style model.
  - Handles PNG/SVG output, supersampling, legends, axis formatting, regression overlays, and plot-type dispatch.
- `chi_plot.rs`
  - Regular (time/potential) plotting pipeline and directory orchestration.
- `eis_plot.rs`
  - Nyquist/Bode rendering pipeline for single-file, directory, and ranked-search outputs.
- `generic_plot.rs`
  - Domain-agnostic plotting for generic `x/y` datasets.
- `lib.rs`
  - Re-export façade for consumers.

## Supported generic plot types

The shared renderer now supports these generic geometries through `PublicationConfig.plot_type`:

- `line`
- `scatter`
- `vertical_bar`
- `horizontal_bar`
- `grouped_bar`
- `stacked_bar`
- `fill_between`
- `stack_plot`
- `pie`

All of them reuse the same output and styling infrastructure:

- SVG + supersampled PNG export
- legend font sizing and placement
- shared color palette handling
- axis labels and figure sizing
- TOML + GUI state resolution through `RawPlotStyle -> PublicationConfig`

## Rendering workflow

For generic plots the data/render separation remains:

```text
parser -> PlotData -> PlotSeries -> plot_hq -> draw_plot_area -> export files
```

Important extension points:

- `PlotType`
  - selects the renderer branch
- `FillBetweenMode`
  - controls how lower bounds are derived for area shading
- `PieValueLabelMode`
  - controls absolute/percentage slice text
- `RawPlotStyle`
  - optional config surface from TOML and GUI state
- `PublicationConfig`
  - fully resolved runtime render contract

## Notes for maintainers

- Existing line/scatter behavior should always remain the default when `plot_type` is omitted.
- New render branches should prefer consuming `PlotSeries` rather than introducing a second data abstraction.
- Export-path smoke tests in `plotting.rs` should be updated whenever a new plot geometry or legend rule is added.

## Data dependencies

Plot pipelines consume parsed datasets from `data_file/` and style/job resolution from `plot_config.rs`.
