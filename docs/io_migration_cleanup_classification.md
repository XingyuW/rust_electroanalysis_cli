# Prompt 3B IO cleanup classification

This classification was prepared before Prompt 3B code deletion. It is based
on `docs/reviews/canonical_io_migration_final_go.md`, whose safe-to-delete
matrix is authoritative. "Current references" are the pre-cleanup search
results for each candidate; an internally unused item is not assumed removable
when it is public.

## Recorded baseline

- Consumer SHA before cleanup: `424f93e334fcf9956fa24fb21d8484f3f0ab0cd6`
- Reviewed consumer SHA: `5f1d73c31419287772a8fdc28093b594596111e4`
- Pinned `electrodata-io` SHA: `dbb6b7d063972114c4208980723e12c807ab199e`
- Baseline commands: locked/offline metadata, `cargo fmt --check`, Clippy with
  warnings denied, `cargo test --all`, and release build were run from the
  clean checkout. Formatting and Clippy passed; the test suite passed.
- Current public data-file re-exports: `EISData`, `EISFitResult`,
  `ElectrochemData`, `DataFileType`, `IntoPlotData`, `LoadedExperimentData`,
  `PlotData`, `PlotDataBuilder`, `PlotDataError`, `PointSelection`, `YSeries`,
  `load_data`, `project_compatibility_read_options`, `read_dataset`,
  `read_dataset_with_sheet`, `InputKind`, measurement-to-plot adapters,
  `load_experiment`, `parse_measurement_file`, `parse_measurement_text`, and
  value-transform types/functions.

## Classification

| Classification | File / symbol | Visibility | Production callers | Test callers / current references | Public or external implication | Canonical replacement | Final-review status | Proposed action |
|---|---|---|---|---|---|---|---|---|
| SafeToDeleteNow | `src/data_file/electrodata_adapter.rs` / module | `pub(crate)` | None | No code caller; module declaration at `src/data_file/lib.rs:16`; review and traceability docs name it | None | `electrodata_domain_adapter::{read_dataset, measurement_parse_result}` | Explicitly safe now | Delete file and module declaration; remove stale docs. |
| SafeToDeleteNow | `electrodata_adapter::ProjectTabularHandler` and its custom registration/helpers | private | None | Contained only in the removable module; debt-register reference | None | Provider built-in format detection and typed datasets | Explicitly safe through the enclosing module | Delete with the module; do not replace local detection. |
| SafeToDeleteAfterArchivingEvidence | `tests/legacy_snapshot/` | integration-test private | None | Imported only by `tests/io_migration_parity.rs`; documented in review, validation doc, architecture doc | None | Permanent canonical-input and scientific regression tests | Safe after final gate is preserved | Write the requested archive, then delete. |
| SafeToDeleteAfterArchivingEvidence | `tests/io_migration_parity.rs` archived-side comparison machinery | integration-test private | None | This test is the sole snapshot importer; review calls its archived-side machinery safe after the gate record | None | Retained canonical boundary, XLSX, EIS semantics, typed-error, and workflow tests | Explicitly safe after preserving final gate result | Archive evidence, then delete the parity test as a completed migration gate. |
| SafeToDeleteAfterArchivingEvidence | direct dev-dependency `calamine` | test-only Cargo dependency | None | `tests/legacy_snapshot/mod.rs:8`; documentation and ownership guard mention it | No public API | Provider-owned XLSX reader plus permanent XLSX tests | Explicitly safe after snapshot removal | Remove from `Cargo.toml` and lockfile only after the snapshot is removed. |
| RetainPublicCompatibility | `measurement_parser::parse_measurement_text` | `pub`, re-exported | None | unit test plus `tests/phase1_domain.rs`; re-export in `data_file/lib.rs` | Deprecated public compatibility API for in-memory text; must remain without a provider buffer API and scheduled deprecation release | `parse_measurement_file` for physical files; future provider buffer API for memory input | Explicitly unsafe | Retain `#[deprecated]`; document migration debt and compile-level public API coverage. |
| RetainProduction | `measurement_parser::parse_measurement_table` | `pub(crate)` | Called by retained `parse_measurement_text` | Unit coverage through retained text API | Required implementation detail of public compatibility API | Future removal only together with `parse_measurement_text` | Explicitly unsafe alone | Retain unchanged. |
| RetainPublicCompatibility | `excel_file::{ExcelTable, ExcelMeasurementParseResult, parse_excel_measurement, read_worksheet}` | `pub` module contents | No internal direct caller found | Public surface; source definitions in `excel_file.rs` | Public table-shaped compatibility API; no local workbook parsing contract | `read_dataset_with_sheet` and typed dataset/domain APIs | Explicitly unsafe | Retain; document provider-backed projection and compile-level API coverage. |
| RetainPublicCompatibility | `input_kind::InputKind` | `pub`, re-exported | No internal production caller found | Module unit tests; re-export at `data_file/lib.rs:33` | Public compatibility/reference classification; unused internally does not authorize removal | Provider detection/read APIs | Explicitly unsafe | Retain without new deprecation schedule; document as non-production detector and compile-test it. |
| RetainPublicCompatibility | `chi_file::EISData` public fields, `from_impedance`, `with_source_bode` | `pub` | EIS plot/search/fit and results use type/file loaders | canonical boundary, XLSX, phase 4/5 tests; struct literals in phase 4/5 tests | Public construction and field compatibility | Canonical `Dataset` conversion for file input; constructors for direct construction | Explicitly unsafe | Retain fields and supported constructors; compile-test constructors. |
| RetainProduction | `EISData::{parse_file, parse_file_with_sheet, parse_file_with_resolver*}` | `pub` | fit runner, search runner, EIS plotting, `data_op` | canonical boundary, XLSX, unit tests | Supported public canonical EIS file loader | `read_dataset[_with_sheet]` → `TryFrom<&Dataset>` | Explicitly production-required | Retain. |
| RetainProduction | `electrodata_domain_adapter::{read_dataset, read_dataset_with_sheet, measurement_parse_result, TryFrom<&Dataset> for EISData}` | `pub` | `chi_file`, measurement parser, Excel wrappers, data-op, search runner | canonical boundary/error/XLSX/domain tests | Public canonical adapter boundary | None; this is the required consumer boundary | Explicitly unsafe | Retain and keep as sole physical-input reader. |
| RetainProduction | file-based measurement loaders: `parse_measurement_file[_with_sheet]`, `load_experiment[_with_sheet]`, `ElectrochemData::parse_file*`, `load_data` | public / re-exported as applicable | plotting, model, calibration, signal, comparison workflows | phase 1, canonical input, unified loading, XLSX tests | Active public and production file-loading API | Provider read → canonical domain adapter | Explicitly required by architecture and compatibility matrix | Retain; document canonical role. |
| RetainProduction | analysis-artifact readers: `domain::read_artifact` and runner wrappers | `pub` / private wrappers | model, mechanism, calibration, signal, health, estimation, validation | `artifact_contract` and workflow tests | Public artifact contract; analysis-artifact JSON/CSV/TOML remains consumer-owned | None; separate from physical input | Outside deletion matrix; allowed consumer ownership | Retain. |

## Required retained-surface state

| Surface | State | Purpose and replacement |
|---|---|---|
| `parse_measurement_text` | deprecated compatibility | Preserves in-memory text callers. Use `parse_measurement_file` for physical files; a provider buffer API is required before removal. |
| Excel wrappers | active compatibility | Preserve table-shaped workbook projections. Use canonical dataset/sheet APIs for new code. |
| `InputKind` | active compatibility | Preserves public reference classification; it is not the production format detector. New production code uses provider read/detection. |
| `EISData` fields and constructors | active compatibility | Preserve direct domain construction. File input uses provider datasets then the domain adapter. |
| file-based canonical loaders and Dataset adapters | active production API | They form the consumer side of the canonical physical-input boundary. |

No additional `#[deprecated]` attribute is proposed: only
`parse_measurement_text` already meets the documented migration-path condition,
and no release deprecation schedule has been approved for the remaining public
surfaces.
