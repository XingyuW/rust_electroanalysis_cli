# Canonical electrodata-io Migration Final Independent Review

**Review decision:** GO WITH DOCUMENTED NON-BLOCKING DEBT — safe to proceed to Prompt 3B

**Consumer SHA:** 5f1d73c31419287772a8fdc28093b594596111e4

**Provider SHA:** dbb6b7d063972114c4208980723e12c807ab199e

## P0 blocking

None.

## P1 major

None. No reproducible runtime, data-integrity, scientific, or compatibility regression was found.

## P2 moderate

### P2-1 — stale IO-ownership documentation

- Files/symbols: docs/engineering_specification/12_change_management_playbook.md Section 7; README.md search pipeline; src/plottings/generic_plot.rs module guidance.
- Exact defect: Documentation still directs format changes to local parser/format-detection modules, says EIS search validates CHI headers locally, and recommends adding parser branches in generic_plot. Production now enumerates regular files and delegates detection, parsing, XLSX selection, schema recognition, and recovery to electrodata-io.
- Consequence: No current runtime or scientific defect, but maintainers could accidentally reintroduce consumer-owned raw parsing or incorrectly exclude XLSX/unusual-extension input.
- Minimum safe correction: State that format work belongs in electrodata-io; consumer directory workflows enumerate files and adapt typed datasets only.
- Regression test: Extend the ownership-documentation test to reject phrases assigning format/header detection to input_kind.rs, search_runner, or plotting parsers, and require canonical XLSX/unusual-extension ownership language.

## P3/minor debt

None beyond the P2 documentation correction.

## Reproducibility

- Original checkout was clean on fix/final-two-migration-p1s.
- A detached clean clone was created at /tmp/electrodata-review.x7Q07F/consumer.
- Locked/offline metadata, dependency resolution, tests, clippy, and release build succeeded there.
- Cargo.toml and Cargo.lock resolve the same immutable provider commit.
- No sibling checkout, path override, or external test-fixture dependency was required.
- The original working tree remained clean and unchanged after review.

## Architecture boundary

All inspected production physical-input paths follow:

    file -> electrodata-io -> Dataset/typed view -> domain adapter -> workflow

The sole production conversion boundary is src/data_file/electrodata_domain_adapter.rs. It calls electrodata_io::read_with_options, then uses typed time-series/EIS views.

Plotting, EIS fit/search, transient, calibration, signal, and estimation all enter through this boundary. No production consumer code independently owns raw detection, CSV/DAT/XLSX parsing, worksheet recognition, EIS-role detection, role assignment, or malformed-row recovery.

The remaining local parser in src/data_file/electrodata_adapter.rs has no production callers and is explicitly legacy/parity-only. The CSV reader in estimation validation handles generated truth/analysis artifacts, which is allowed.

## Directory workflows

Regular/time-series plotting and EIS search were exercised with text DAT, compatible text with an unusual extension, text with a misleading .bin extension, a Mach-O binary renamed .csv, XLSX, and generated analysis artifacts mixed into the directory.

- Compatible DAT, unusual-extension text, and XLSX were ingested canonically.
- Text content was detected independently of its misleading extension.
- Renamed binary produced typed UnsupportedBinary.
- Generated analysis artifacts were rejected without preventing successful physical inputs from completing.
- Partial batches preserved successful artifacts and returned nonzero aggregate status containing typed per-file failures.
- EIS search preserved distinct outputs for accepted inputs with different extensions.

## Time-series domain conversion

The adapter preserves raw coordinate values, coordinate name and unit, source timestamp ordering, channel order, logical and source channel names, channel units, every Option<f64> value, and null positions.

Seconds normalization occurs only in the downstream experiment-loading path that explicitly requests and records it.

Verified cases:

- seconds: preserved at the adapter, identity conversion downstream;
- hours: preserved, then explicitly converted using x3600;
- days: preserved, then explicitly converted using x86400;
- unknown coordinate unit: preserved and not silently converted;
- headerless coordinates: values, inferred roles, and diagnostics retained;
- channel selectors accept both logical and source-header aliases.

## EIS semantics

Canonical conversion was verified for 3-column, 4-column magnitude, 4-column phase, 5-column, reordered-role, and XLSX EIS inputs.

The adapter preserves frequency, real and imaginary impedance, source imaginary sign, optional source-measured magnitude, optional source-measured phase, derived magnitude using hypot, and derived phase using atan2(...).to_degrees().

Serialized artifacts distinguish:

- source_measured_magnitude_ohm
- source_measured_phase_deg
- derived_magnitude_ohm
- derived_phase_deg

No source quantity is mislabeled as derived or vice versa.

## Structured errors

Actual variants, fields, nesting, and sources—not just display strings—were inspected.

Verified cases include:

- binary: UnsupportedBinary { path, magic };
- unknown format: UnknownFormat { path, best_confidence, threshold };
- ambiguous workbook: AmbiguousWorksheet { path, candidates };
- worksheet selection: ReadContext retaining worksheet plus MissingWorksheet;
- missing EIS role: MissingRequiredRole { path, role, detected_roles };
- duplicate/ambiguous role: EisSchemaConflict with path and conflicting role;
- wrong typed view: WrongDatasetKind or InvalidDatasetView;
- malformed numeric EIS cell: provider MissingValue retaining role, source row, and column;
- invalid timestamp/numeric/ragged rows: structured recovery diagnostics with code, recovery action, row, and column.

DataParsingError transparently retains provider errors. BatchFileFailure::Canonical retains that wrapper for directory plotting/search. Signal comparison and estimation likewise retain it in their source chains.

## Compatibility profile

The project does not accidentally inherit scientifically important provider defaults. The explicit compatibility profile fixes:

- invalid timestamp -> SkipRow;
- invalid numeric -> Null;
- ragged row -> PadNulls;
- header inference -> Auto;
- missing values -> retained nulls;
- coordinate ordering -> Preserve;
- column naming -> Canonical;
- validation -> Structural;
- lossy UTF-8 -> disabled.

All have regression coverage.

## Estimation diagnostics

estimate run and estimate compare share the same validated ingestion boundary.

Artifacts retain skipped timestamp rows, invalid cells converted to null, ragged-row recovery, header inference, diagnostic code, recovery action, and row/column context.

Configured thresholds are enforced:

- max_skipped_timestamp_rows = 0
- max_missing_measurement_fraction = 0.20
- reject_missing_required_channel = true

Malformed timestamp and selected-channel numeric corruption are rejected. Ragged recovery is accepted only when the selected scientific channel remains valid and diagnostics are retained. Estimation does not silently consume malformed recovered data.

## XLSX

Verified:

- simple time-series workbook auto-selection;
- historical preamble workbook;
- typed ambiguity for multiple compatible worksheets;
- explicit sheet selection;
- missing-sheet error context;
- EIS worksheet selection;
- XLSX signal characterization;
- XLSX estimation run and compare;
- XLSX EIS directory search.

The compatibility wrappers in src/data_file/excel_file.rs call the canonical reader and only project typed values. No production consumer-side Calamine parsing remains.

## Independent parity

tests/legacy_snapshot is a dependency-independent copy traced to pre-migration commit cc6f28379b04616cd54f5e8c94836bd4d14a2107.

It does not reference or call electrodata-io, implements archived text/EIS/XLSX behavior locally, and is compiled only into the parity integration test.

The complete parity corpus passed 6/6. Comparisons cover complete time-series and EIS domain objects, including ordering, names, units, optional values, measured quantities, derived quantities, and metadata.

Canonical-only acceptance—headerless, DAT, 3-column EIS, 4-column magnitude, reordered EIS, and XLSX EIS—is explicitly classified rather than treated as unexplained parity drift.

No unexplained scientifically consequential difference remains.

## Scientific regression

Pre-migration commit cc6f28379b04616cd54f5e8c94836bd4d14a2107 was compared with current 5f1d73c31419287772a8fdc28093b594596111e4 using deterministic inputs.

| Workflow | Result |
|---|---|
| EIS fit | Report byte-identical |
| Transient | Scientific JSON exact after path/timestamp and additive ingestion diagnostics |
| Calibration extraction | Scientific JSON exact after provenance fields |
| Signal characterize | Scientific JSON exact after provenance/config paths |
| Signal compare | Result JSON byte-identical |
| Estimate run | Numeric/scientific JSON exact after documented channel-identity and diagnostic additions |
| Estimate compare | Numeric/scientific JSON exact after schema/diagnostic additions and excluding runtime timing |

Numeric comparison was exact for serialized scientific values; no tolerance-only or unexplained numerical difference was needed.

## Documentation

Canonical ownership is correctly documented in the architecture, workflow, data-model, validation, and technical-debt documents.

The stale statements described under P2 remain in the change-management playbook, README search description, and generic_plot.rs rustdoc. There are no remaining production claims of local Calamine ownership, but those passages still imply local format/header parser ownership or understate supported directory inputs.

## Legacy/reference code usage matrix

| Surface | Classification | Migration path |
|---|---|---|
| EISData::parse_file[_with_sheet] | production-required | Retain canonical typed adapter |
| EISData::from_impedance / with_source_bode | compatibility-required | Preferred public construction path; migrate direct literals to these methods |
| Public EISData fields | compatibility-required | Retain until a documented breaking API transition |
| parse_measurement_text | deprecated compatibility-required | Move callers to parse_measurement_file; in-memory users need a future provider buffer API |
| parse_measurement_table | compatibility-helper | Remove only together with parse_measurement_text |
| excel_file wrappers | compatibility-required | Migrate callers to canonical dataset/sheet APIs before deprecation |
| InputKind | compatibility/reference | Migrate callers to provider detection/read APIs before removal |
| electrodata_adapter | dead legacy/reference | No callers; removable |
| tests/legacy_snapshot | parity-test-only | Removable after this gate is recorded |
| Test-only Calamine dependency | parity-test-only | Removable with legacy_snapshot |

## Safe-to-delete matrix

| Prompt 3B target | Safe now? | Qualification |
|---|---:|---|
| src/data_file/electrodata_adapter.rs | Yes | No production or compatibility callers |
| tests/legacy_snapshot/ | Yes | Final independent parity completed |
| tests/io_migration_parity.rs archived-side machinery | Yes | Preserve final gate result in review records |
| Direct test-only calamine dependency | Yes | After removing legacy snapshot |
| parse_measurement_text | No | Deprecated public compatibility API |
| parse_measurement_table alone | No | Required by the retained text API |
| InputKind | No | Public compatibility surface; document/deprecate migration first |
| Excel wrappers | No | Public compatibility surface backed by canonical ingestion |
| Public EISData construction fields | No | Retain supported migration path/API compatibility |
| Production canonical adapters | No | Required boundary |

## Commands executed

    git status
    git rev-parse HEAD
    git log --oneline -10
    git clone --no-local …
    git checkout --detach 5f1d73c…
    cargo metadata --locked --offline
    cargo tree --locked --offline -i electrodata-io
    cargo fmt --check
    cargo clippy --all-targets --all-features --locked --offline -- -D warnings
    cargo test --all --locked --offline
    cargo build --release --locked --offline

Also executed source/caller audits, typed-error harnesses, directory mixed-input runs, the complete parity corpus, and representative CLI smoke tests for plot, EIS fit/search, transient fit, calibration extraction, signal characterize/compare, and estimate run/compare.

Validation result: all 302 executable tests passed; zero failed. Fifteen documentation tests were intentionally ignored.

## Commands failed

No required formatting, clippy, test, or build command failed.

Expected/non-gate nonzero cases:

- Mixed-input directory runs returned nonzero because deliberately injected binary/artifact files produced typed partial-batch failures while valid outputs completed.
- An EIS XLSX fit using a deliberately oversized circuit for the small fixture reported insufficient post-preprocessing points; canonical XLSX EIS parsing and search succeeded.
- An initial calibration smoke used an incompatible default analyte configuration and correctly rejected the concentration metadata; rerunning with the matching configuration passed.

## Consumer SHA

5f1d73c31419287772a8fdc28093b594596111e4

## Provider SHA

dbb6b7d063972114c4208980723e12c807ab199e

## Prompt 3B recommendation

Proceed with Prompt 3B, limited to the entries marked safe above. Retain public compatibility surfaces until their documented migration/deprecation paths are completed. Correct the P2 documentation during or immediately after cleanup; it does not block parser removal.

GO WITH DOCUMENTED NON-BLOCKING DEBT — safe to proceed to Prompt 3B

