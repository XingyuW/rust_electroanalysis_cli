# domain subsystem

`domain` contains the scientific data contracts shared by parsers, metadata
loading, and future analysis workflows. It deliberately does not depend on
the plotting renderer or on the impedance AST.

## Core types

- `MultiChannelMeasurement` owns one shared time axis and aligned
  `MeasurementChannel` values. Missing numeric values are represented as
  `None`, not removed from the scientific record.
- `ElectrochemicalExperiment` combines measurement data with sensor and
  reference metadata, sample-matrix information, environmental series,
  timestamped `ExperimentEvent`s, and `AnalysisProvenance`.
- `ParseDiagnostics` records row and sampling-quality findings produced while
  parsing input data.
- `ExperimentMetadataDocument` is the TOML-facing schema. It is kept separate
  from plot configuration so experimental context cannot silently become a
  rendering setting.

## Boundaries

`data_file` converts source files into these types and supplies adapters back
to the existing `PlotData` container. `plottings` remains unaware of the
scientific model. No fitting, filtering, calibration, or transient analysis is
implemented in this module.
