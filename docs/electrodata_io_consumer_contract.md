# `electrodata-io` consumer contract

Status: proposed cross-crate contract for `rust_electroanalysis_cli`

Consumer repository inspected at: `198a3ee` (`feature/canonical-dataset-api`)

Contract fixture manifest: `tests/fixtures/electrodata_io_contract/expected.toml`

## 1. Purpose and boundary

`electrodata-io` is intended to become the only file/container, delimiter,
worksheet, format-detection, row-decoding, schema, unit, diagnostic, and source
provenance boundary for electrochemical measurement files. The consumer should
eventually reduce its input boundary to:

```rust
let dataset = electrodata_io::read_with_options(path, &options)?;
let measurement = MultiChannelMeasurement::try_from(&dataset)?;
```

After that call, this repository should perform only explicit domain conversion,
scientific validation specific to an analysis, analysis, plotting, and reporting.
It must not guess delimiters, inspect workbook sheets, parse instrument headers,
repair rows, or reinterpret column order.

This document is normative for the future boundary. Descriptions labelled
"current" record the behavior inspected in this repository and do not silently
approve it. No runtime parser is removed and no runtime behavior is changed by
this contract/fixture commit.

An important sequencing fact is that the inspected `Cargo.toml` already contains
an absolute path dependency on `electrodata-io`, and the inspected production
code already calls it. This contract does not add or change that dependency.

## 2. Required dataset types

The dataset must expose a stable scientific kind independent of the physical
container and instrument format:

- `TimeSeries`: one time axis and one or more aligned measurement columns.
- `ImpedanceSpectrum`: frequency plus complex impedance components, with
  optional measured magnitude and phase.
- `GenericTable`: a numeric table whose scientific axis or roles are not known.

Equivalent names are acceptable only when their semantics are stable, public,
documented, serializable, and match the contract fixtures. Instrument/container
identities such as CHI OCPT, CHI EIS, CSV, DAT, and XLSX remain separately
available as detected-format and container metadata; they are not dataset kinds.

## 3. Required column roles

The public schema must provide these roles, or exactly equivalent stable
semantics with lossless conversion to them:

```rust
Time
Frequency
ImpedanceReal
ImpedanceImaginary
ImpedanceMagnitude
ImpedancePhase
Potential
Current
MeasurementChannel(u32)
Unknown
```

Requirements:

- Roles are resolved by normalized semantic headers, not fixed column position.
- The source header and stable canonical name are both retained.
- `MeasurementChannel(n)` retains the positive channel number from forms such as
  `E1/V`, `CH1/V`, `E5/V`, and `CH12`.
- A conventional `Potential/V` column uses `Potential`; numbered electrode
  channels use `MeasurementChannel(n)`.
- `Current/A` uses `Current`, not an application-defined string.
- Unrecognized columns are retained as `Unknown`, never discarded or assigned a
  scientific role solely because they are numeric.
- Duplicate source headers remain addressable by stable column identity and
  source position. Role lookup must return ambiguity rather than silently choose
  the first when a role is not unique.
- During migration, the current `Phase` role may be offered as a deprecated alias
  of `ImpedancePhase`, and `PotentialChannel(u32)` as a deprecated alias of
  `MeasurementChannel(u32)`. The consumer-facing semantics above are canonical.

The current crate exposes `X`, `Y`, `Phase`, `PotentialChannel(u32)`, and
`Custom(String)` instead of several roles above. The project-local
`ProjectTabularHandler` also emits `Custom("channel:n")`. These are contract gaps.

## 4. Required units

Every column descriptor must carry a parsed unit, including an explicit
`Unknown`/unspecified unit rather than absence with ambiguous meaning. At minimum
the API must stably represent:

- seconds (`s`) for `Time`;
- hertz (`Hz`) for `Frequency`;
- ohms (`ohm`) for all impedance roles;
- degrees (`deg`) for `ImpedancePhase`;
- volts (`V`) and millivolts (`mV`) for potential/measurement channels;
- amperes (`A`) and common scaled amperes for `Current`;
- source-preserved other units such as hours, days, degrees Celsius, pH, and
  concentration units;
- `Unknown` when the source provides no defensible unit.

The `Dataset` must preserve numeric values and source units without silently
scaling them. Typed accessors may return values converted to requested units, but
the conversion must be explicit, checked, deterministic, and recorded in
conversion provenance. `MultiChannelMeasurement::try_from(&Dataset)` must create
its shared time axis in seconds because downstream segmentation, calibration,
signal, and estimation code interprets time as seconds. Measurement channels
retain a declared unit string and aligned optional values.

Strict conversion must fail with a typed unit error when a required role has an
unknown or dimensionally incompatible unit. Compatibility conversion may apply
a caller-supplied unit hint, but must diagnose and record that hint; it must not
invent `s`, `V`, `A`, `Hz`, `ohm`, or `deg` from column position alone.

## 5. Required missing-value behavior

- Empty numeric cells and documented sentinels (`NA`, `N/A`, `NaN`, `null`,
  `missing`, case-insensitive) are represented as missing values, not IEEE NaN.
- Missing channel values preserve the row and alignment. This is required by
  `MeasurementChannel.values: Vec<Option<f64>>`.
- The dataset exposes total missing count and missing counts by column.
- A missing time/frequency coordinate is not a usable analysis row. Strict mode
  rejects it. Compatibility mode may skip it only under an explicit row policy
  and must report the original row number and every skipped cell.
- Entirely missing required columns are errors in both modes.
- Dataset-to-plot conversion may omit missing y-values only after domain
  conversion; the canonical dataset and measurement remain unchanged.

Missing channel cells are valid-but-incomplete scientific observations and are
therefore accepted in strict mode with structured diagnostics. Individual
analyses retain responsibility for maximum missing fractions and interpolation
rules; the IO layer must never interpolate.

## 6. Required row-recovery policies

`ReadOptions` must make row recovery explicit. A boolean such as `strict` is not
sufficient; stable policies are required for at least:

```rust
RowPolicy::RejectInvalid                         // strict default
RowPolicy::Compatibility {
    invalid_coordinate: SkipRow,
    invalid_measurement: ReplaceWithMissing,
    short_row: PadWithMissing,
    extra_cells: IgnoreTrailing,
}
```

Recovery rules:

- Strict mode stops at the first malformed numeric coordinate, malformed numeric
  channel, non-finite number, or ragged row and returns a typed error.
- Compatibility mode skips an invalid timestamp row, retains a row containing an
  invalid channel value as missing, pads missing trailing cells, and ignores
  extra trailing cells. Every action produces a diagnostic.
- Blank lines before/between header and data may be ignored. A blank line inside
  a data section must not silently terminate reading if later data exists.
- No mode sorts, deduplicates, averages, interpolates, resamples, changes an
  impedance sign, or changes source row order.
- The dataset must expose source-row identity for every retained row so later
  domain errors can refer to the original CSV/DAT line or XLSX worksheet row.

These policies extract the public `parse_measurement_text` compatibility behavior
without making it the strict default.

## 7. Required diagnostics

Diagnostics must be structured, stable, machine-readable, serializable, and
available even when a compatibility read succeeds. Each diagnostic requires:

- stable code;
- severity (`Info`, `Warning`, or `Error`);
- human-readable message;
- physical path and, for workbooks, worksheet;
- one-based source row when known;
- source column name and/or one-based column when known;
- action taken (`None`, `RowSkipped`, `ValueReplacedWithMissing`, `RowPadded`,
  `ExtraCellsIgnored`, or equivalent);
- original token for malformed cells, subject to a bounded length;
- detected format and active policy/profile.

Stable diagnostic coverage is required for malformed timestamps, malformed
numeric cells, missing values, ragged rows, duplicate timestamps, nonmonotonic
timestamps, unknown units, inferred headerless roles, ambiguous format/sheet,
lossy decoding, non-positive/nonmonotonic EIS frequency, and measured-versus-
derived magnitude/phase mismatch.

Counts must be directly available for total source data rows, retained rows,
skipped rows, malformed rows, missing cells, duplicate timestamps, and
nonmonotonic transitions. The current consumer's `ParseDiagnostics` can then be
constructed without re-scanning the time axis or flattening diagnostic details
into strings.

## 8. Required typed accessors

The consumer must not depend on Polars column-name fallback or reproduce null
iteration. The dataset API must provide typed, borrow-friendly accessors with
typed errors:

```rust
dataset.kind() -> DatasetKind
dataset.column_descriptor(id) -> &ColumnDescriptor
dataset.columns_by_role(&ColumnRole) -> Vec<ColumnId>
dataset.column_by_role(&ColumnRole) -> Result<TypedColumn<'_>>
dataset.optional_f64_by_role(&ColumnRole) -> Result<Vec<Option<f64>>>
dataset.required_f64_by_role(&ColumnRole) -> Result<Vec<f64>>
dataset.time_seconds() -> Result<Vec<Option<f64>>>
dataset.measurement_channels() -> Result<Vec<MeasurementColumn<'_>>>
dataset.eis() -> Result<EisView<'_>>
dataset.missing_value_count() -> usize
dataset.source_row(dataset_row) -> Option<SourceRow>
```

Names may differ, but the semantics may not. `required_f64_by_role` fails on any
missing value and identifies the role and source location. Accessors must detect
duplicate roles. `EisView` must provide frequency, real, imaginary, magnitude,
and phase in contract units, deriving only the two optional quantities as
specified below.

## 9. Required EIS behavior

The following are supported input shapes:

- three columns: frequency, real impedance, imaginary impedance;
- four columns: the three required components plus magnitude or phase;
- five columns: frequency, real, imaginary, magnitude, phase;
- any of those semantic columns in a different source order.

Requirements:

- Detection and mapping are header-role based, not positional.
- `Frequency`, `ImpedanceReal`, and `ImpedanceImaginary` are required.
- Missing magnitude is derived as `hypot(real, imaginary)` by the typed EIS view.
- Missing phase is derived as `atan2(imaginary, real)` in degrees by the typed
  EIS view.
- A measured magnitude or phase remains a source column. It is validated against
  the derived quantity within configured tolerances but never overwritten.
- Imaginary impedance sign is preserved exactly. Nyquist sign transformation is
  a plotting concern, not IO behavior.
- Frequency and source row order are preserved. Strict mode rejects non-positive
  and nonmonotonic frequency; compatibility mode preserves them with diagnostics
  when explicitly requested.
- Required EIS components cannot contain missing values at domain conversion.
- CHI metadata preamble, instrument model, technique, parameters, and raw rows
  remain available.

The current built-in reader recognizes only fixed-order five-column CHI EIS;
the project handler recognizes fixed-order three/four-column compact EIS, while
the current `EISData` conversion still requires phase. Reordered and three-column
inputs are therefore migration gaps captured by the fixtures.

## 10. Required multichannel time-series behavior

- A time-series has exactly one resolved `Time` role and at least one non-time
  column.
- `E1`, `CH1`, `E5`, and noncontiguous channel numbers map to
  `MeasurementChannel(n)` without renumbering.
- Conventional potential and current columns retain `Potential` and `Current`.
- Unknown numeric channels are preserved as `Unknown` with source header, unit,
  position, and values. The consumer may convert them to named domain channels;
  it must not lose them.
- All channel vectors remain aligned to the retained time axis, including nulls.
- Duplicate timestamps and timestamp resets are preserved in compatibility mode
  with exact counts and source locations. No averaging occurs in IO. The current
  calibration layer's duplicate averaging and estimation timestamp preprocessing
  remain explicit downstream scientific policies.
- Strict mode rejects duplicate or decreasing time by default. A caller may
  select compatibility policies for legacy CHI runs and timestamp-reset data.

## 11. Required XLSX behavior

- `.xlsx` uses the same detection, role, unit, recovery, diagnostic, and typed
  accessor pipeline as delimited text.
- Auto-selection succeeds only when exactly one worksheet is compatible.
- `SheetSelector::Name` and zero-based `SheetSelector::Index` are supported.
- Multiple compatible sheets without a selector return a typed ambiguity error
  listing candidate sheet names; a missing named sheet is a typed error.
- Worksheet name, one-based source row/cell coordinates, cached formula status,
  and physical container are retained in provenance/diagnostics.
- Numeric cells remain numeric. Empty cells are missing. Formula cells without a
  usable cached numeric result follow strict/compatibility policy and diagnose
  `UnusableFormula` or equivalent.
- The consumer requires XLSX time-series ingestion now. XLSX EIS remains rejected
  by current CLI time-series workflows for backward compatibility, but the IO
  crate should still return an `ImpedanceSpectrum`; workflow-level rejection
  belongs in this consumer after canonicalization.
- Legacy `.xls` remains a typed unsupported-container error for this consumer.

`timeseries.xlsx` contains one worksheet named `measurement`; its expected
schema is in the manifest.

## 12. Required DAT behavior

`.dat` is a supported delimited-text container alias and must use content-based
delimiter detection. Comma, tab, and semicolon delimiters used by existing text
inputs must be supported consistently for `.csv`, `.txt`, and `.dat`. Strict
UTF-8 is the default; optional lossy decoding must be explicit and diagnosed.

The inspected `InputKind` advertises `.dat`, but the current `electrodata-io`
container dispatch accepts only `.csv`, `.txt`, and `.tsv`. This is a concrete
contract gap represented by `generic_text.dat`.

## 13. Required error behavior

All failures are typed and non-panicking. At minimum, callers must distinguish:

- IO and permission failure;
- unsupported container/extension and invalid UTF-8;
- unknown or ambiguous scientific format;
- missing/ambiguous worksheet;
- missing header or ambiguous headerless schema;
- missing required role, duplicate role, invalid role signature;
- unknown/incompatible required unit;
- empty data;
- malformed numeric or non-finite value;
- ragged row;
- missing required coordinate/value;
- duplicate/nonmonotonic coordinate in strict mode;
- spreadsheet/formula decoding failure.

Errors must include path, worksheet if any, source row/column if known, stable
error code/variant, offending role, and a concise message. Compatibility actions
are diagnostics, not errors. Ambiguity must never be collapsed into "invalid
data" before the consumer can present an actionable `--sheet`/format message.

## 14. Required provenance

The dataset must retain:

- caller path and basename;
- physical container and selected worksheet;
- detected delimiter/encoding for text;
- detected scientific format, confidence, and evidence;
- source header text, canonical name, role, unit, and one-based position for
  every column;
- raw metadata rows with original one-based row numbers;
- parsed acquisition metadata and parameters without discarding raw spellings;
- source-row mapping for every retained dataset row;
- active read options, validation level, and recovery policy;
- every recovery/derivation/unit-conversion action;
- a content digest, or sufficient byte-level source identity for the consumer to
  populate `AnalysisProvenance.input_sha256` without a second read.

The consumer continues to add its software version, git commit, configuration
hash, and analysis generation time. IO provenance must be value-semantic and
serializable so derived artifacts can embed it.

## 15. Expected domain conversions

### Time series

`MultiChannelMeasurement::try_from(&Dataset)` must:

1. require `DatasetKind::TimeSeries`;
2. obtain one time coordinate converted to seconds;
3. retain all rows supplied by the selected IO policy in source order;
4. convert each non-time dataset column to a `MeasurementChannel` with its source
   or canonical name, declared unit, and `Vec<Option<f64>>`;
5. preserve numbered channel identity in channel metadata;
6. preserve missing channel values and reject required-coordinate missingness;
7. attach or return diagnostics/provenance without flattening them to strings;
8. validate vector alignment but perform no sorting, interpolation, resampling,
   duplicate averaging, or timestamp segmentation.

Unknown-role columns are not discarded. The consumer may require an explicit
channel selection for an analysis, but generic plotting must still be able to
project every numeric channel.

### EIS

The EIS domain conversion must require `DatasetKind::ImpedanceSpectrum`, obtain
frequency in Hz, real/imaginary impedance in ohms, and phase in degrees from
measured or derived accessors. It must preserve metadata and source sign/order.
Magnitude remains available for validation/Bode plotting even though current
`EISData` stores only frequency, real, imaginary, and phase.

### Error ownership

Format/container/row/schema/unit errors originate in `electrodata-io`. Domain
conversion errors cover only incompatibility between a valid dataset and the
requested domain type. Scientific workflow errors cover analysis-specific
requirements such as minimum points, permitted missing fraction, timestamp
segmentation, calibration window validity, or positive-frequency requirements
that a compatibility caller elected to retain.

## 16. Backward-compatibility expectations

Compatibility mode must preserve these observable behaviors until callers and
tests migrate deliberately:

- CHI metadata and numeric source order are preserved.
- Negative imaginary impedance is preserved.
- valid rows with missing or invalid channel cells retain alignment using nulls;
- invalid timestamp rows can be skipped with visible diagnostics;
- short rows can be padded and extra trailing cells ignored with diagnostics;
- duplicate and decreasing timestamps remain in source order with diagnostics;
- numbered/noncontiguous channels retain their channel numbers;
- CSV/TXT/DAT and compatible XLSX time-series inputs reach the same domain shape;
- multi-series regular/OCPT plotting retains each named source channel;
- current `DataParsingError` messages can wrap the new typed source error while
  CLI-facing wording is migrated.

Backward compatibility does not require preserving implementation types such as
`ElectrochemData`, `ExcelTable`, `DataFileType`, `ProjectTabularHandler`, Polars
column names, or duplicate parsing passes. It does not authorize silent repair.
During migration, compatibility should be the explicit profile for legacy
workflows; strict should be used for new validation and contract tests. Changing
the default profile is a separate user-visible change.

## 17. Parity expectations and policy decisions

| Condition | Current file-backed behavior | Required compatibility policy | Required strict policy | Classification |
|---|---|---|---|---|
| Invalid timestamp | Current `electrodata-io` numeric loading rejects it. Public legacy `parse_measurement_text` skips the row and diagnoses it. | Skip the row, retain source-row mapping, emit `MalformedTimestamp` and `RowSkipped`. | Reject at the offending cell. | Skip behavior should become explicit compatibility policy. |
| Invalid numeric channel | Current file-backed path rejects it. Legacy text parsing retains the row as `None` and diagnoses it. | Replace the cell with missing and emit both malformed/recovery diagnostics. | Reject at the offending cell. | Null replacement should become explicit compatibility policy. |
| Ragged row | Current file-backed path rejects it. Legacy parsing pads short rows and ignores extra fields while diagnosing the row. | Pad short rows; ignore only trailing extra cells; diagnose every action. | Reject the first ragged row. | Recovery should become explicit compatibility policy. |
| Missing channel value | Current dataset retains nulls; `MultiChannelMeasurement` retains `None`; plotting later omits the point. | Preserve null and alignment; diagnose/count it. | Accept valid rows with null channel cells; downstream analysis decides admissibility. | Must be preserved. |
| Missing timestamp | Current dataset can contain a null coordinate, and `measurement_from_dataset` silently removes that row except for aggregate skipped counts. | Skip only with row-level diagnostics and provenance. | Reject. | Existing aggregate-only behavior is insufficient and should become explicit policy. |
| Duplicate timestamp | Current domain diagnostics count duplicates; source order is preserved. Calibration may average duplicates inside a selected window; IO does not. | Preserve all rows and emit stable diagnostics. | Reject by default. | Preservation must be available; averaging remains downstream policy. |
| Nonmonotonic timestamp/reset | Current domain diagnostics count backward transitions; estimation may segment/preprocess later. | Preserve source order and emit stable diagnostics. | Reject by default. | Preservation must be available; segmentation remains downstream policy. |

The strict decisions above are intentionally stronger than current permissive
domain construction. They prevent a canonical IO boundary from silently blessing
scientifically risky time axes. Compatibility keeps historical data loadable
without pretending it is clean.

## 18. Current ingestion-path inventory

| Input | Current dispatch and conversion | Current limitations relevant to the contract |
|---|---|---|
| CHI EIS CSV/TXT | `EISData::parse_file` -> project `read_dataset` -> role extraction; used by fit/search/EIS plotting and by `load_data`. | Five-column built-in is positional; three/four-column support is project-local; `EISData` requires phase; `load_data` reads an EIS file twice. |
| CHI OCPT | `ElectrochemData::parse_file[_series]` for regular CHI plots; `parse_measurement_file_with_sheet` for scientific workflows. | Two parallel domain shapes and repeated reads for diagnostics. |
| Multichannel OCPT | Built-in numbered CHI channels or project generic time-series handler -> `MultiChannelMeasurement`; CHI plotting expands to `ElectrochemData` series. | Generic channels become `Custom` roles; duplicate headers and channel identity are handled differently by wrappers. |
| Generic time-series CSV/TXT | Project handler finds `time`/`timestamp`; conversion removes rows whose time is null and retains other nulls. | Handler duplicates header/unit logic; malformed/ragged file rows fail before legacy recovery code can run. |
| Headerless table | Built-in regular reader yields `X`/`Y`; project conversion treats `X` as time. | Scientific role/unit inference is implicit; tables wider than two columns are not supported by the built-in regular handler. |
| DAT text | `InputKind` and docs advertise `.dat`; file path eventually reaches `electrodata-io`. | Current dependency rejects the `.dat` container extension. |
| XLSX time series | Same dataset adapter with automatic or explicit sheet selection; consumed by transient, calibration, signal, estimation, model workflows. | Project-local handler is still required for wide sheets; XLSX EIS is rejected by the time-series conversion. |
| Malformed/missing/ragged | File-backed reads use dependency strict numeric/ragged behavior; public in-memory text parser has permissive recovery. | Behavior depends on entrypoint rather than explicit options. |
| Duplicate/nonmonotonic time | `ParseDiagnostics::from_measurement` rescans the time axis. | Diagnostic logic duplicates dependency scientific validation and loses source rows after conversion. |

Raw-file callers are:

- `runners/fit.rs` and `search_runner.rs` for EIS;
- `plot_runner.rs`, `plottings/chi_plot.rs`, and
  `plottings/generic_plot.rs` for plots/directories;
- `runners/transient.rs`, `calibration.rs`, `signal.rs`, `estimation.rs`, and
  `model.rs` for time-series/domain workflows.

`runners/health.rs` and `runners/mechanism.rs` primarily ingest versioned JSON
artifacts/metadata rather than raw electrochemical tables. `potentiometry/` and
`impedance/` consume already-converted measurements/arrays, apart from an
impedance test that parses an external fixture.

## 19. Duplicate IO logic found

1. `ProjectTabularHandler` repeats time-header normalization, unit parsing,
   numeric-row probing, and compact EIS recognition that belong in the IO crate.
2. `measurement_parser::parse_measurement_text/table` independently splits CSV,
   recognizes headers/sentinels, parses numbers, repairs ragged rows, and builds
   diagnostics.
3. `electrodata_adapter::measurement_from_dataset` repeats role lookup, typed
   numeric extraction, missing-row filtering, header/unit parsing, and time-axis
   diagnostics.
4. `chi_file::{ElectrochemData,EISData}` perform another metadata/role-to-vector
   projection. `parse_file_with_diagnostics` and
   `parse_file_series_with_diagnostics` read the same path twice.
5. `data_op::load_data` reads once to classify, then reads CHI EIS again through
   `EISData::parse_file`.
6. `InputKind::classify_path`, search eligibility, and plot directory walkers
   independently combine extension filtering and content detection.
7. `excel_file` adapts the canonical numeric frame back into strings solely to
   preserve the former table API.
8. Duplicate/nonmonotonic/missing counts exist in both dependency diagnostics and
   the consumer's `ParseDiagnostics`, with source locations flattened/lost.
9. `AnalysisProvenance` reads the source again to hash it after parsing.

## 20. Recommended `electrodata-io` API

```rust
let options = electrodata_io::ReadOptions::new()
    .with_profile(electrodata_io::ReadProfile::Compatibility)
    .with_validation(electrodata_io::ValidationLevel::Scientific)
    .with_sheet(electrodata_io::SheetSelector::Auto);

let dataset = electrodata_io::read_with_options(path, &options)?;
let measurement = MultiChannelMeasurement::try_from(&dataset)?;
```

Recommended additions/changes in the provider crate:

- `DatasetKind` and the canonical roles/units in this contract;
- explicit `ReadProfile`/`RowPolicy`, with no entrypoint-dependent recovery;
- typed, null-aware role accessors and `EisView`;
- stable diagnostic/error codes with source cells and recovery actions;
- source-row mapping, read-options snapshot, delimiter/encoding, and digest in
  provenance;
- built-in wide generic time-series, DAT, reordered/three/four/five-column EIS,
  `Current`, and `Unknown` support;
- measured-versus-derived EIS accessors without adding synthetic source columns;
- a provider-side test that loads this directory and asserts `expected.toml`.

## 21. Later deletion candidates in this repository

Delete only after provider contract tests pass and every caller is migrated:

- project-local detection/reader construction and `ProjectTabularHandler` in
  `src/data_file/electrodata_adapter.rs`;
- file parsing/recovery portions of `src/data_file/measurement_parser.rs` (retain
  experiment-metadata composition somewhere domain-appropriate);
- compatibility workbook/string-table wrappers in `src/data_file/excel_file.rs`;
- content-level parsing classification in `src/data_file/input_kind.rs` (retain a
  small CLI/batch support policy only if still needed);
- `ElectrochemData` parsing methods and eventually the type once CHI plotting
  consumes `MultiChannelMeasurement`/`PlotData` directly;
- `EISData` parsing methods once `EISData::try_from(&Dataset)` exists; retain the
  analysis type if useful;
- `DataFileType` dispatch and re-read logic in `data_op::load_data`;
- duplicate directory eligibility checks in plot/search modules;
- duplicate row/time diagnostic reconstruction once provider diagnostics map
  losslessly to the domain.

Do not delete scientific timestamp preprocessing, calibration duplicate
averaging, missing-fraction checks, domain validation, plot projection, or
analysis provenance. Those are downstream policies, not file IO.

## 22. Migration risks and gates

- The current absolute dependency path is nonportable and already precedes this
  planning run; dependency/version strategy must be resolved separately.
- Role renames (`Phase`, `PotentialChannel`, `Custom`) can break exhaustive
  matches and serialized expectations.
- Strict-mode defaults would newly reject historical duplicate/reset datasets;
  migration must choose profiles per workflow and report them in artifacts.
- Three-column EIS needs derived phase; tolerance/sign conventions must be tested
  before fit parity is claimed.
- Reordered EIS can expose code that still indexes columns positionally.
- Wide generic tables include non-potential channels; treating every y column as
  voltage would be scientifically wrong.
- Headerless data has no defensible unit/role evidence; compatibility inference
  must remain diagnosed and configurable.
- XLSX formula/cache behavior and automatic sheet ambiguity can differ across
  workbook producers.
- Recovery changes row counts and therefore hashes, event alignment, sampling
  statistics, calibration windows, and estimation segments.
- Replacing consumer provenance hashing with provider digests requires agreement
  on exact bytes and digest algorithm.
- Public legacy parser APIs may have external callers even when repository call
  sites disappear; deprecation should precede removal.

Migration gates are: provider fixture parity in strict and compatibility modes;
domain conversion parity; existing CLI integration tests; representative real
CHI/CSV/XLSX smoke tests without checking confidential data into either crate;
and explicit approval for any default-policy or CLI behavior change.

## 23. Fixture-manifest interpretation

`expected.toml` is normative. `row_count` and `missing_value_count` describe the
dataset returned in compatibility mode when strict mode rejects a fixture.
`column_count` describes source columns; EIS derived magnitude/phase accessors do
not increase it. Source row numbers include headers/preambles exactly as stored.
`Unknown` is an explicit role or unit, not missing manifest data.

The required fixture set is deliberately small and synthetic. It contains no
large or confidential experimental data. Additional fixtures for invalid
numeric cells, duplicate timestamps, and nonmonotonic timestamps make the parity
decisions executable rather than prose-only.
