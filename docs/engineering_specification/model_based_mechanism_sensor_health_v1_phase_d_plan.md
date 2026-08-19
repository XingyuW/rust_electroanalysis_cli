# MHI V1 Phase D — Reporting, Plotting, and Public Scientific Output

## 1. Authority, decision, and scope

This is the authoritative implementation contract for MHI V1 Phase D.  It was
written against `main` commit
`1b04f22b0588e48e39808a870eb55b254272a88c`.

**Phase D is a renderer-projection phase.**  Its only scientific inputs are
validated, serialized artifacts.  It may select fields, give them fixed labels,
sort them, format values, and draw them.  It must not calculate or replace a
Phase B hypothesis conclusion, a Phase C dimension conclusion, threshold,
causal conclusion, evidence-independence result, lineage result, residual,
fit, prediction, baseline statistic, or model evaluation.

The certified Phase D public route is exactly:

```text
electroanalysis report render
```

Existing `mechanism report`, `health report`, `estimate report`, and `model
report` commands are retained with their current behaviour for compatibility.
They are not Phase-D-certified public reports and must not be extended into a
second Phase D route.

### 1.1 Explicit non-goals

- No new scientific assessment rule, threshold, score, normalisation, fit,
  model evaluation, causal inference, or mechanism inference.
- No raw physical-file parsing, directory discovery, CSV/XLSX role detection,
  model fitting, model selection, or calculation of a source-derived Bode
  channel in the Phase D route.
- No change to artifact schemas owned by A0/A1/Phase B/Phase C.
- No change to legacy producer behaviour, Phase C health semantics, existing
  plot configuration semantics, or existing report filenames.
- No durable `ArtifactIdentity` for a presentation file and no second lineage
  graph.

### 1.2 Observable problem and confirmed causes

The repository has useful report and plot producers, but they are workflow
sidecars rather than one public scientific-output contract.  The current
`mechanism report` and `health report` text omit the serialized Phase B
`hypothesis_assessments` and Phase C `phase_c.dimension_assessments`,
respectively.  Existing plots are primarily direct analysis plots and accept
raw input or perform renderer-side transforms.  Thus an implementation cannot
currently produce a complete, deterministic Phase B/Phase C public report
without inventing conventions at implementation time.

The following are confirmed by the current code, not hypotheses:

1. `src/plottings/health_plot.rs::plot_health_assessment` selects
   `robust_z_score.or(z_score)`, omits unavailable values, and returns with no
   output when all values are unavailable; it does not represent the nine
   Phase C dimensions.
2. `health_plot::plot_health_trend` renders only the first serialized trend.
3. `src/plottings/mechanism_plot.rs::plot_mechanism_report` computes
   `log10()` and silently filters non-positive timescales and ratios.
4. `src/plottings/transient_plot.rs::event_components` re-evaluates transient
   model components using `potentiometry::transient::models::evaluate`.
5. `src/plottings/calibration_plot.rs::line_points` calculates a theoretical
   line, and its input mapping recomputes `log10(activity)`.
6. `src/plottings/model_plot.rs::plot_model_analysis` converts the presence of
   equilibrium supporting evidence into an invented `0/1` series and converts
   missing component potential into `0.0` with `unwrap_or(0.0)`.
7. `src/plottings/estimation_plot.rs::plot_estimation_report` takes
   `log10(activity)` without a defined unavailable-data presentation policy.
8. `src/plottings/plotting.rs::augment_with_regression` can fit an additional
   regression curve when used by the generic raw-input pathway.
9. `src/plottings/{chi_plot,eis_plot,generic_plot}.rs` read physical input
   and/or use raw-input helpers; that is deliberately outside Phase D.

None of these is to be copied into the Phase D route.

## 2. Complete existing reporting and visualization inventory

| Component / source file | Current CLI/workflow route | Current input | Current output | Current scientific work | status | Phase D disposition |
|---|---|---|---|---|---|---|
| `plottings/plotting.rs` | `plot` | prepared/raw `PlotData` | SVG, PNG | optional regression augmentation | PRODUCTION | KEEP only as a finite series drawing primitive; Phase D sets regression off and supplies prepared serialized series |
| `plottings/chi_plot.rs`, `plot_runner.rs` | `plot regular-plot` | canonical physical files | SVG, PNG | coordinate transforms | PRODUCTION | OUT-OF-SCOPE raw-input visualisation |
| `plottings/generic_plot.rs` | `plot generic-plot` | canonical physical files | SVG, PNG | selection / optional regression backend | PRODUCTION | OUT-OF-SCOPE; never a Phase D reader |
| `plottings/eis_plot.rs` | `plot eis`, EIS search | physical EIS files/search result | Nyquist/Bode SVG/PNG, fit text | source/derived channels and ranked candidate conversion | PRODUCTION | OUT-OF-SCOPE; replace for Phase D with artifact-only EIS figure projectors |
| `plottings/transient_plot.rs` | `transient fit` sidecar | transient report | SVG, PNG | component re-evaluation; event geometry | PRODUCTION | KEEP legacy sidecar; REPLACE for Phase D |
| `plottings/calibration_plot.rs` | `calibration fit` sidecar | calibration report + observations | SVG, PNG | activity log and theoretical line | PRODUCTION | KEEP legacy sidecar; REPLACE for Phase D |
| `plottings/signal_plot.rs` | `signal characterize` sidecar | signal report | PNG | axis ranges only | PRODUCTION | KEEP legacy sidecar; REPLACE for Phase D to add missing-data policy and SVG |
| `plottings/health_plot.rs` | `health assess`, `health trend` sidecar | health assessment/trend | PNG | z-score selection and first-trend selection | PARTIAL | KEEP legacy sidecar; REPLACE for Phase D |
| `plottings/mechanism_plot.rs` | `mechanism compare/trend` sidecar | mechanism report | SVG, PNG | `log10` and invalid-value filtering | PARTIAL | KEEP legacy sidecar; REPLACE for Phase D |
| `plottings/estimation_plot.rs` | `estimate run` sidecar | state estimation report | PNG | activity logarithm | PRODUCTION | KEEP legacy sidecar; REPLACE for Phase D |
| `plottings/model_plot.rs` | `model decompose` sidecar | model analysis report | PNG | binary evidence and missing-to-zero conversion | PARTIAL | KEEP legacy sidecar; REPLACE for Phase D |
| `runners/mechanism.rs` | `mechanism compare`, `trend`, `report` | artifacts/config | JSON, CSV, TXT, plots | Phase B assessor plus legacy report | PRODUCTION | Phase B assessment is input authority; legacy text remains compatibility-only |
| `runners/health.rs` | `health baseline/assess/trend/report` | artifacts/config | JSON, CSV, TXT, plots | Phase C assessor plus legacy report | PRODUCTION | Phase C assessment is input authority; legacy text remains compatibility-only |
| `runners/{transient,calibration,signal,estimation,model}.rs` | respective analysis/report commands | raw inputs and artifacts | JSON, CSV, TXT, plots | scientific analyses plus sidecars | PRODUCTION | source artifact producers only; no Phase D recomputation |
| `runners/fit.rs`, `impedance/reporting.rs` | `eis fit/export-fit` | raw EIS file | fit text, JSON artifact | fit and diagnostics | PRODUCTION | EIS artifact source only |
| `search_runner.rs`, `impedance/ecm_search.rs` | `eis search` | physical EIS file/directory | text, CSV, plot outputs | ranking/search scoring | PRODUCTION | OUT-OF-SCOPE; a search report is not a Phase B assessment |
| `cli.rs`, `main.rs` | all current commands | CLI arguments | terminal messages/errors | parsing and dispatch | PRODUCTION | EXTEND only with one `report render` route |
| `README.md`, `src/README.md`, `docs/engineering_specification/*` | documentation | n/a | Markdown | n/a | PRODUCTION DOCS | EXTEND with the public-output contract and CLI documentation |

Current terminal `println!` output is operational progress only and is not a
public scientific report.  Existing JSON/CSV/TXT outputs listed above are
analysis sidecars, not a substitute for the Phase D public-output manifest.

## 3. Exact Phase D architecture and file-level work plan

The implementation must add these production modules and no scientific module
may depend on them:

| File | Exact responsibility / key symbols |
|---|---|
| `src/reporting/mod.rs` | public, narrow façade: re-exports the public Phase-D runtime-error vocabulary required by `RunnerError`; keeps `render_public_report` crate-private |
| `src/reporting/error.rs` | public, reachable `PublicReportError` plus the public payload enums frozen in section 18.2; it must not define or alter `ReportingError` |
| `src/reporting/reader.rs` | `ReportInputs::read`; canonical `domain::read_artifact` calls and schema/compatibility gates only |
| `src/reporting/projection.rs` | immutable `PublicReportProjection::from_inputs`; field copies, fixed ordering, and no numeric/statistical operation beyond safe textual formatting |
| `src/reporting/claims.rs` | total functions `mechanism_level_text`, `causal_status_text`, `health_status_text`, `evidence_state_text`, `unavailable_text`; contains no thresholds or branching that changes a result |
| `src/reporting/tables.rs` | seven named CSV writers in section 7 |
| `src/reporting/document.rs` | `write_markdown_report` and `write_public_summary_json` |
| `src/reporting/figures.rs` | eleven figure dispatchers in section 8; takes only `PublicReportProjection` prepared series and uses no analysis module |
| `src/reporting/lineage.rs` | `project_lineage`; copies each serialized root and direct dependency, and projects only the supplied catalog's root-membership bit, without catalog-node output, traversal, resolution, or identity construction |
| `src/report_config.rs` | clap-neutral `ReportFormat`, `ReportSelection`, `ReportRenderOptions`; selection parsing and validation only |
| `src/runners/report.rs` | resolves output directory, calls reader → projection → writers, and performs preflight collision checks |
| `src/runners/mod.rs` | adds `pub mod report;` and a distinct `RunnerError::PublicReport(#[from] reporting::PublicReportError)` conversion; the existing `RunnerError::Reporting(#[from] domain::ReportingError)` remains unchanged |
| `src/cli.rs` | adds `Command::Report`, `ReportCommand::Render`, `ReportRenderCommand`, and `CommandSpec::ReportRender` exactly as section 6 |
| `src/main.rs` | one `CommandSpec::ReportRender` match arm; no alternative dispatch |
| `src/lib.rs` | adds `pub mod reporting;` solely so the public `RunnerError::PublicReport` payload is reachable; `render_public_report` itself remains crate-private |

The implementation must not modify the existing rendering functions named in
section 2 except to add a non-scientific error conversion if compilation
requires it.  In particular it must not "fix" legacy renderer semantics as
part of Phase D.  That preserves Phase A0/A1/B/C behavior and avoids making a
legacy command silently mean something new.

`PublicReportProjection` is a temporary in-memory presentation view.  It must
derive `Clone, Debug, PartialEq, Serialize` and own only copied serialized
fields plus availability markers.  It has no `ArtifactIdentity`, no lineage
constructor, no science dependency, and no mutating access to its input
artifacts.  Tests must deserialize input JSON before rendering and compare the
typed values afterward.

## 4. Frozen input-artifact contract

All artifact reads use `domain::read_artifact`.  The lineage catalog is the
one deliberate non-`VersionedArtifact` input and is read only through
`domain::read_artifact_lineage_catalog`, as frozen in section 18.4.  Direct
`fs::read`, `serde_json` parsing, physical-file readers, and raw `plot_config`
readers are prohibited in `src/reporting`.

| CLI flag | ArtifactKind / accepted schema | required | reader and consumer | fields rendered | legacy / absent behaviour |
|---|---|---:|---|---|---|
| `--mechanism` | `mechanism_analysis`, schema 4 | yes | `read_artifact::<MechanismAnalysisReport>` → mechanism section, tables D-TBL-01/03/05, D-FIG-01 | `analysis_id`, `hypothesis_assessments`, `hypothesis_history`, timescales, comparisons, records, warnings, provenance, lineage | schemas 1–3: render only `legacy_hypotheses`, timescales, comparisons, trends, warnings as `legacy mechanism artifact; Phase B V1 hypothesis assessment unavailable`; no inferred Phase B row |
| `--health` | `health_assessment`, schema 4 or 3 | yes | `read_artifact::<SensorHealthAssessment>` → health section, D-TBL-02/03/07, D-FIG-02/03 | identity, overall status, Phase C dimensions/evidence bundle, legacy fields, baseline comparisons, warnings, provenance, lineage | schema 3: exact banner `Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized.`; do not manufacture a dimension row; schemas 1/2 rejected as `UnsupportedSchema` |
| `--lineage-catalog` | `ArtifactLineageCatalog`, schema 1 (not an `ArtifactKind`) | no | `domain::read_artifact_lineage_catalog` → D-TBL-04/D-FIG-11 | root membership bit plus each root's serialized direct dependencies | absent: project each root's direct lineage only, with `catalog_not_supplied`; `LegacyUnknown` remains explicit; never emit catalog-only nodes, resolve, or traverse the catalog |
| `--eis` | `eis_fit`, schema 3 | no | `read_artifact::<EisFitArtifact>` → D-FIG-04/05, D-TBL-03 | source, fitted, residuals, parameters, statistics, CIs, warnings, provenance, lineage | schemas 1/2 rejected as `UnsupportedSchema`; no raw EIS fallback |
| `--transient` | `transient_analysis`, schema 3 | no | `read_artifact::<TransientAnalysisReport>` → D-FIG-06, D-TBL-03 | selected fit, serialized raw/fitted time series, residuals, CIs, event/warnings/provenance/lineage | schemas 1/2 rejected; no component re-evaluation |
| `--calibration-observations` | `calibration_observations`, schema 3 | paired optional | canonical reader → D-FIG-07 only | observation ID, potential, activity/log activity only when serialized positive, branch, uncertainty | must be paired with `--calibration`; absence makes D-FIG-07 unavailable |
| `--calibration` | `calibration_analysis`, schema 3 | paired optional | canonical reader → D-FIG-07, D-TBL-03 | selected candidate's serialized predictions/residuals/validation/warnings/provenance/lineage | must be paired with `--calibration-observations`; schemas 1/2 rejected |
| `--signal` | `signal_analysis`, schema 3 | no | canonical reader → D-FIG-08, D-TBL-03 | analysis time/value, PSD, Allan points, spike flags, warnings/provenance/lineage | schemas 1/2 rejected; absent subseries displayed as unavailable, not zero |
| `--estimation` | `state_estimation`, schema 4 | no | canonical reader → D-FIG-09, D-TBL-03 | timestamps, measured/predicted potential, innovation variance, activity SE, observability, warnings/provenance/lineage | schemas 1–3 rejected for figures that need schema-4 fields; no compiled-model inference from absent backend/profile |
| `--model` | `ism_model_analysis`, schema 5 | no | canonical reader → D-FIG-10, D-TBL-06/03 | time, observed/predicted potential, serialized residual, uncertainty, validity/equilibrium records, warnings via input evidence, lineage | schemas 1–4 rejected; missing observed/residual remains unavailable |
| embedded in health | `health_baseline`, schema 3 | not independently accepted | no second read; projection uses `SensorHealthAssessment.baseline_comparison` only | current/baseline values, units through matched health feature, comparability, sample count, override reason | no baseline comparison means D-TBL-07 and D-FIG-03 say `not_serialized` |
| embedded in health | A1 `EvidenceBundle` and copied lineage catalog | schema owned by health schema 4 | no re-evaluation → D-TBL-03 | IDs, target, source, source class, direction, availability, validity, quantity, uncertainty, temporal support | only copied records; non-Phase-C health artifacts have no invented bundle |

`--calibration` and `--calibration-observations` are a required pair.  No
other optional input depends on another optional input.  The report never
opens the file paths present in `AnalysisProvenance`; paths are displayed as
provenance strings only.

## 5. Public output and identity contract

The output directory is required.  The all-or-nothing publication model in
section 18.3 supersedes the earlier per-file temporary-file wording: no final
certified bundle path becomes visible until the complete staged bundle has
passed validation.  A preflight determines every requested path before the
staging directory is created.  With `--overwrite`, an output directory that
contains anything other than the complete prior Phase-D contract set is an
`UnmanagedOutputEntry` collision and is left untouched.

| Output class | path and format | source | deterministic writer / overwrite |
|---|---|---|---|
| machine-readable public projection | `OUTPUT/public_summary.schema1.json`, UTF-8 pretty JSON | required assessments + supplied optional artifacts | `write_public_summary_json`; schema 1, struct field order, no timestamp; replace only with `--overwrite` |
| human-readable report | `OUTPUT/scientific_report.md`, UTF-8 LF Markdown | same projection | `write_markdown_report`; fixed section order; replace only with `--overwrite` |
| render manifest | `OUTPUT/render_manifest.schema1.json`, UTF-8 pretty JSON | command options + availability only | `write_render_manifest`; records paths/statuses, no clock; replace only with `--overwrite` |
| CSV tables | `OUTPUT/tables/<table-id>.csv` | section 7 | CSV writer with header and LF; all seven paths preflight together |
| SVG figures | `OUTPUT/figures/<figure-id>.svg` | section 8 | Phase-D figure dispatch; only when renderable |
| PNG figures | `OUTPUT/figures/<figure-id>.png` | section 8 | same payload as SVG; only when renderable |

The reviewed conceptual JSON list below is retained only as historical context;
it is superseded in full by the closed `PublicSummaryV1` and `RenderManifestV1`
graphs in sections 18.5–18.6. It grants no implementation choice.

```text
schema_version: 1
output_kind: "phase_d_public_scientific_output"
renderer_contract: "mhi_v1_phase_d_public_output_v1"
source_artifacts: [PublicSourceArtifact]
mechanism: PublicMechanismProjection
sensor_health: PublicHealthProjection
optional_projections: PublicOptionalProjections
lineage: PublicLineageProjection
tables: [PublicOutputStatus]
figures: [PublicOutputStatus]
```

`PublicSourceArtifact` contains exactly `input_flag`, `artifact_kind`,
`schema_version`, `artifact_id`, `semantic_sha256`, `lineage_state`, and
`availability`.  `artifact_id`/`semantic_sha256` are null for
`LegacyUnknown`; the reason is displayed instead.  `PublicOutputStatus`
contains exactly `id`, `relative_path`, `format`, `status`, and `reason`.
Status is one of `written`, `unavailable`, or `not_selected`.  The projection
copies assessment enum values as their serde snake-case strings; it never
maps them to a new scientific state.

These files are **derived presentation files**, not durable scientific
artifacts.  They receive neither `ArtifactIdentity` nor an A1 dependency
entry.  Their output path is not semantic identity.  Re-rendering equal
serialized artifacts and equal options must produce byte-equivalent JSON/CSV/
Markdown and scientifically equivalent SVG/PNG.  The manifest records the
same input identities so a consumer can trace the presentation back through
the existing A1 lineage.  It must not synthesize a report identity.

## 6. Exact CLI contract

```text
electroanalysis report render \
  --mechanism PATH --health PATH --output-dir PATH \
  [--lineage-catalog PATH] [--eis PATH] [--transient PATH] \
  [--calibration PATH --calibration-observations PATH] [--signal PATH] \
  [--estimation PATH] [--model PATH] \
  [--format all|json|markdown] \
  [--figures all|none|mechanism_timescale,health_dimension_status,current_vs_baseline,eis_nyquist,eis_bode,transient_response,calibration_performance,signal_diagnostics,estimation_observed_predicted,model_observed_predicted,lineage] \
  [--tables all|none|mechanism_evidence,health_dimensions,evidence_provenance,artifact_lineage,timescale_comparison,model_consistency,current_vs_baseline] \
  [--overwrite]
```

Defaults are `--format all --figures all --tables all`; `all` and `none` are
mutually exclusive with a comma-list.  `--format json` still writes the render
manifest and selected tables/figures; it only omits Markdown.  `--format
markdown` still writes the manifest and selected tables/figures; it only omits
`public_summary.schema1.json`.  At least one of JSON or Markdown is required.

Invalid combinations and exact failures:

| condition | error variant |
|---|---|
| one of the two required artifacts absent | `CliError::Parse(clap::Error)` before runner dispatch |
| only one calibration pair flag supplied | `PublicReportError::InvalidCombination { detail: "--calibration and --calibration-observations must be supplied together" }` |
| requested EIS/transient/calibration/signal/estimation/model figure without its source | section 18.8: omitted/default selection records `not_selected` or `unavailable`; an explicit ID or `all` returns `PublicReportError::RequestedOutputUnavailable` |
| a selected table requires optional source that was not supplied | CSV is written with the fixed `not_provided` availability row |
| an input has a wrong `ArtifactKind` | propagated `ArtifactError::IncompatibleKind` |
| an input has unsupported schema | `PublicReportError::Artifact { flag, path, source: ArtifactError::UnsupportedSchemaVersion { .. } }` |
| `--format` has a value outside `all|json|markdown` | `CliError::Parse(clap::Error)` before runner dispatch |
| selected figure/table ID unknown or duplicate | `PublicReportError::InvalidSelection { selector, value }` after successful parsing |
| output path exists without `--overwrite`, or output dir is a file | `PublicReportError::OutputCollision` / `InvalidOutputDirectory` |
| writer or plot backend fails | `PublicReportError::Write` / `PlotBackend`, with path and source |

The runner must not create a workspace last-run mode or modify an analysis
configuration.  It prints only the output directory and count of written/
unavailable files after every requested writer succeeds.

## 7. Frozen table contract

All CSV headers are written even for unavailable optional inputs.  Missing
numeric/text data is the literal `NA`; a missing collection is `[]`; a missing
artifact is `not_provided`.  A text field that itself equals `NA` is CSV quoted
normally.  No empty string has scientific meaning.

| ID / filename | exact columns | row order | source / purpose |
|---|---|---|---|
| D-TBL-01 `mechanism_evidence.csv` | `hypothesis_id,display_name,evidence_level,reason_codes,validation_status,temporal_statuses,timescale_statuses,amplitude_statuses,repeatability_statuses,identifiability_statuses,contradiction_requirement_ids,component_ids,history_ids,legacy_status` | current hypotheses by `definition.hypothesis_id`; legacy rows by `hypothesis_id` after current | Phase B conclusion projection; shows support and contradiction without promotion |
| D-TBL-02 `sensor_health_dimensions.csv` | `dimension,display_label,status,evidence_state,reason_codes,interpretation_category,causal_status,source_evidence_ids,excluded_evidence_ids,source_artifact_ids,legacy_status` | exact `HealthDimension::ALL` order | all nine Phase C dimensions; schema-3 writes one `legacy_phase_c_not_serialized` row, never nine synthetic rows |
| D-TBL-03 `evidence_provenance.csv` | `assessment_scope,evidence_id,target,source_class,direction,availability,validity,quantity_value,quantity_unit,uncertainty,source_artifact_kind,source_artifact_id_or_fingerprint,source_field_path,experiment_scope,acquisition_families,temporal_support` | `assessment_scope`, then evidence ID, then source sort key | copied Phase C bundle and Phase B evidence IDs when serialized; exposes contradictions and missing data |
| D-TBL-04 `artifact_lineage.csv` | `root_input_flag,row_kind,root_artifact_kind,root_artifact_id,lineage_state,direct_dependency_role,direct_dependency_kind,direct_dependency_id,catalog_supplied,root_catalog_entry_present` | supplied input flag order; each root row first, then its direct-dependency rows in A1 canonical dependency order | only each certified input root and its serialized direct dependencies; catalog-only nodes never appear, and there is no traversal, resolution, or inferred missing dependency |
| D-TBL-05 `timescale_comparison.csv` | `comparison_id,record_id,eis_timescale_id,eis_value_s,eis_standard_error_s,transient_timescale_id,transient_value_s,transient_standard_error_s,ratio,log10_distance,symmetric_relative_difference,confidence_interval_overlap,compatibility_probability,evidence_level,supporting_evidence,contradictory_evidence,alternative_explanations,warnings` | `comparison_id` lexical | serialized Phase B timescale values and limitations |
| D-TBL-06 `model_consistency.csv` | `availability,time_s,observed_voltage_v,predicted_voltage_v,unexplained_residual_v,uncertainty_status,validity_status,equilibrium_status` | `time_s`, preserving equal-time source order | model observed/predicted evidence only; no residual recomputation or binary evidence conversion |
| D-TBL-07 `current_vs_baseline.csv` | `availability,feature,unit,current_value,baseline_value,comparability,absolute_difference,relative_difference,log_ratio,z_score,robust_z_score,empirical_percentile,baseline_sample_count,override_reason,warnings` | feature lexical then unit lexical | serialized health comparison; `warnings` is `[]` or the closed projection warning list; neither ranking nor new status is calculated |

For D-TBL-01 `display_name` is copied from
`HypothesisAssessmentRecord.definition.display_name`; no fallback label is
created.  For all nested enum lists, individual values are joined with `;` in
the serialized vector order, except IDs which use existing canonical sorted
order.  This rule applies identically to CSV, Markdown, and JSON.

## 8. Frozen scientific-figure contract

There are **11 planned figures**.  Every plotted point is a serialized source
field.  Axis extent padding, non-overlapping text layout, and stable colour/
shape selection are display geometry, not scientific calculation.  A figure
does not render unless its stated minimum serialized data exists.  Its status
is then `unavailable` in the manifest; no blank chart or substituted zero is
created.  Every renderable figure writes both SVG and PNG.

| ID / title / filename | scientific purpose and source | axes, units, series / uncertainty | annotations, thresholds, failures |
|---|---|---|---|
| D-FIG-01 Mechanism timescale comparison `mechanism_timescale_comparison` | Shows serialized cross-method timing agreement and disagreement; `MechanismAnalysisReport.comparisons` plus referenced timescale records | x: comparison ID (categorical); y: stored `log10_distance` (dimensionless); series grouped by stored `evidence_level`; direct labels identify EIS/transient IDs | labels include stored evidence level and warning symbol; **threshold lines: none**; unavailable when no comparison has a finite stored distance |
| D-FIG-02 Sensor-health dimension status `sensor_health_dimension_status` | Makes the complete schema-4 nine-dimension assessment visible | categorical grid: dimension label × stored status; no numeric axis; one row for each `HealthDimension::ALL` | cell text shows status/evidence state/reason count; DQI and Indeterminate have named glyphs; **threshold lines: none**; schema 3 unavailable with legacy notice |
| D-FIG-03 Current versus baseline `current_vs_baseline` | Shows serialized current/baseline comparison without reclassification | x: `BaselineComparison.feature`; y: a pair is plotted only when `comparability` is `comparable` or `comparable_with_warnings`, exactly one `SensorHealthAssessment.features` entry has the same `name` and a nonempty `unit`, and both serialized values are finite; its `unit` is the y-unit and the pair's `current_value` / `baseline_value` are the series values | `comparable_with_warnings` remains rendered and carries the stored `override_reason` as a manifest warning and caption limitation; `not_comparable` and `unknown`, zero/multiple matching feature-unit entries, or no finite pair are unavailable with their closed reason; **threshold lines: none** |
| D-FIG-04 EIS Nyquist `eis_nyquist` | Shows observed and fitted impedance correspondence | x: serialized `source.z_real_ohm` / `fitted.z_real_ohm` (Ohm); y: serialized `source.z_imag_ohm` / `fitted.z_imag_ohm` (Ohm), direct and unmodified; observed and serialized fitted series | y-axis label exactly `Im(Z) [Ohm]`; caption exactly states `Imaginary impedance is plotted with its serialized sign; Phase D performs no Nyquist sign transform.`; parameter-at-bound and non-identifiable warning marker; **threshold lines: none**; unavailable for mismatched/non-finite arrays |
| D-FIG-05 EIS Bode `eis_bode` | Shows serialized impedance magnitude/phase versus frequency | x: serialized positive `frequency_hz`, logarithmic display axis; y panels: serialized source-measured magnitude/phase if present, otherwise serialized `derived_*`, explicitly captioned; fitted magnitude/phase | no values are derived; source-null points shown as missing markers; **threshold lines: none**; unavailable if no positive finite frequency or no matching series |
| D-FIG-06 Transient selected-fit response `transient_selected_fit_response` | Shows observed response, selected serialized fitted response, and serialized residuals for each event whose selected representation is unique | x: `event.segment.raw_time_local` with `event.segment.raw_potential_v`; fitted x: `event.segment.fitted_time_local`; fitted y and residual: only the one successful `event.candidate_fits` element whose `model == event.selected_model`; y panels: potential (V), residual (V) | count of successful matching candidates = 1 renders that serialized representation; 0 records `selected_fit_not_found`; >1 records `selected_fit_ambiguous`; never choose first, rank, refit, evaluate, or recalculate.  Explicit selection failure and default behavior are frozen in section 18.8. |
| D-FIG-07 Calibration performance `calibration_performance` | Shows serialized calibration observations versus the selected candidate's serialized predictions/residuals | x: serialized positive activity displayed as log10 activity (only if that log value is serialized in a validation point; otherwise use `ValidationPredictionPoint.observed_log10_activity`); y panels potential/residual (V) | validation `extrapolated` marker; **threshold lines: none**; unavailable without paired artifacts and serialized aligned validation predictions; no theoretical line |
| D-FIG-08 Signal diagnostics `signal_diagnostics` | Shows retained raw analysis signal and serialized PSD/Allan diagnostic evidence | panels: time (s) vs signal unit; frequency (Hz, log display) vs PSD unit; averaging time (s, log display) vs deviation in signal unit | missing samples/spike flags visible; no resampling, Welch recomputation, or ASDs; **threshold lines: none**; panels independently unavailable |
| D-FIG-09 Estimation observed versus predicted `estimation_observed_predicted` | Shows measured and predicted potential and makes serialized measurement-variance availability visible | x: timestamp (s); y: potential (V); measurement and prediction series; no error bars because only variance, not a serialized potential uncertainty interval, is available | rejected/predict-only update markers and `applied_measurement_variance_v2` availability marker; **threshold lines: none**; unavailable when fewer than two finite timestamped series points |
| D-FIG-10 Model observed versus predicted `model_observed_predicted` | Shows model output against observations without treating prediction as observation | x: `time_s`; y: potential (V) plus residual (V) panel; observed, predicted, serialized unexplained residual; serialized uncertainty interval only when its semantics are available in `PredictionUncertainty` | validity/equilibrium is a labelled marker from its serialized enum/record, never a 0/1 series; **threshold lines: none**; unavailable if fewer than two finite prediction points |
| D-FIG-11 Artifact lineage and provenance `artifact_lineage` | Shows exactly supplied A1 root and direct-dependency information | directed root/direct-edge diagram, no scientific axes; nodes labelled kind/ID prefix/schema, edges labelled existing dependency role | `LegacyUnknown` is shown explicitly; catalog membership is labelled only when directly serialized; the renderer does not discover missing dependencies, cycles, or consistency; **threshold lines: none**; always renderable from required roots |

Figure captions must state (1) the artifact IDs or explicit `LegacyUnknown`,
(2) units, (3) whether a series is observed, model-derived, or producer
assessment according to serialized labels, and (4) limitations/warnings.  A
logarithmic display axis is a coordinate representation only; the stored values
remain unchanged and are available in the corresponding table.  It is never a
threshold transform or a way to discard non-positive evidence: those points
produce an unavailable/invalid marker and a caption count.

## 9. Human-readable report contract and language safety

`scientific_report.md` has this exact section order:

1. Analysis identity and renderer boundary
2. Input artifacts and compatibility state
3. Mechanism assessment
4. Sensor-health assessment
5. Key evidence and contradictions
6. Uncertainty and data-quality limitations
7. Current-versus-baseline comparison
8. Optional analysis projections
9. Figures
10. Tables
11. Lineage and provenance
12. Reproducibility metadata

The mechanism section contains one subsection per current hypothesis ordered
by hypothesis ID.  It displays ID, display name, target components, evidence
level, every gate status, reason codes, contradictions, validation status,
component interpretation, and history.  It displays the serialized text of
supporting/missing/contradictory evidence and alternative explanations where
present.  The health section contains exactly nine subsections in
`HealthDimension::ALL` order for schema 4 and never calls a legacy
`HealthDomain` a Phase C dimension.

The following total mapping is mandatory for Markdown, table labels, figure
captions, and JSON display fields:

| serialized value | required wording | prohibited wording |
|---|---|---|
| `not_assessed` / `indeterminate` | `not assessed` / `indeterminate; evidence is unavailable or insufficient as stated` | `healthy`, `normal`, `no issue` |
| `hypothesized` | `hypothesized` | `proved`, `established cause` |
| `experimentally_supported` | `experimentally supported within the serialized evidence` | `proven`, `caused by` |
| `validated_for_domain` | `validated for the serialized domain` | `universally validated`, `causal proof` |
| `contradicted` | `contradicted by the serialized evidence` | `ruled out in all conditions` |
| health `within_baseline` | `within serialized baseline` | `healthy` |
| health `watch` / `degraded` / `critical` | `watch` / `degraded` / `critical` exactly | `failed because of <mechanism>` |
| `data_quality_insufficient` | `data-quality insufficient; no normal-state conclusion follows` | `normal`, `within baseline` |
| `adequate_evidence` / `no_evidence` / `insufficient_evidence` / `poor_data_quality` / `contradictory_evidence` | exact serialized phrase | a stronger evidence claim |
| causal `observed` / `associated` / `hypothesized` | `observed`, `associated with`, `hypothesized` exactly | `caused by`, `causes` |
| causal `experimentally_supported` / `validated_for_domain` | `experimentally supported` / `validated for domain` exactly | `causal proof` |
| `LegacyUnknown` | `lineage unknown: <serialized reason>` | any guessed input/dependency identity |
| absent optional field | `not serialized` or `not provided` as applicable | `0`, `none detected`, `negative finding` |

The exact fixed disclaimer, placed before all conclusions, is: **“This report
projects serialized assessments.  Support, association, consistency, and
model-derived prediction do not by themselves establish causal proof.”**

## 10. Determinism, missing-data, and scientific-integrity rules

1. Current Phase C dimension order is `HealthDimension::ALL`; legacy health
   never enters that order.
2. Artifact rows sort by `ArtifactKind::as_str`, then known `artifact_id`; a
   `LegacyUnknown` row sorts after known IDs by kind then fingerprint/reason.
3. Hypotheses sort by ID; nested requirement/gate/contradiction/history arrays
   retain their serialized canonical order, with strings sorted only where A1
   or producer contracts already require sorting.
4. Tables use the order in section 7. Figures use the order in section 8.
   Legends are observed, model-derived/fitted, producer assessment, warning,
   unavailable.  No palette conveys a status without its textual label.
5. Missing numeric values use a visible `NA` marker and a missing-data count;
   they are never filtered silently, coerced to zero, interpolated, or joined
   across timestamps.  The exact cross-implementation numeric, CSV, Markdown,
   JSON, and annotation format contract is section 18.9.  Invalid/non-finite
   values are rejected by the canonical reader before projection.
6. DQI uses the literal label `Data quality insufficient (DQI)`, an `DQI`
   glyph, and its reason codes.  Indeterminate uses the literal label
   `Indeterminate`, a `?` glyph, and its evidence state/reasons.  Neither
   shares the green/within-baseline presentation.
7. Contradictory evidence is rendered in every relevant hypothesis/dimension
   subsection and D-TBL-03.  It cannot be hidden by a selected positive
   status.  Excluded evidence is labelled `excluded` rather than dropped.
8. Event order is the artifact's serialized event order.  Timestamped figures
   use original serialized order; equal timestamps retain source order.  No
   sort by measurement value is allowed.
9. Display axis padding, categorical placement, typeface, colour, and line
   style are fixed constants documented in `reporting/figures.rs`; they must
   not depend on a scientific result.  No threshold line is permitted in any
   Phase D figure because thresholds are assessment inputs, not display rules.
10. SVG metadata must contain no creation date, host path, random ID, or
    process ID.  PNG image dimensions are fixed at 1600×1000 for every
    single-panel figure; D-FIG-08/09/10 panels are 1600×1400.  SVG viewBox is
    the same dimension.  PNG pixel equality is not required, but parsed
    dimensions and figure payload must be deterministic.

## 11. Legacy and backward compatibility

- Schema-4 `SensorHealthAssessment` is the complete Phase C input.  Schema-3
  is renderable only as an explicit pre-Phase-C legacy health assessment; the
  phrase `phase_c` and all nine labels are absent from its detail body except
  for the compatibility banner.  The `health_dimensions.csv` single legacy
  row is not a dimension assessment.
- Current Phase B uses `MechanismAnalysisReport` schema 4.  Schemas 1–3 are
  legacy mechanism outputs: only fields actually serialized are shown, and
  D-TBL-01 marks `legacy_status=phase_b_v1_not_serialized`.
- A1 `LegacyUnknown` remains exactly its serialized variant and reason.  The
  renderer does not derive a lineage identity from a report ID, pathname,
  provenance path, metadata, or content hash, and it does not traverse or
  resolve an `ArtifactLineageCatalog`.
- No existing command, public Rust function, existing report path, output
  filename, plot function, TOML key, or current artifact schema changes.
  `report render` is additive.  The existing `plot` command continues to
  serve raw-input graphics and is documented as outside this contract.

## 12. Superseded error and atomicity text

The reviewed error and per-file publication description is superseded in full
by sections 18.2 and 18.3.  In particular, it does not introduce a second
`ReportingError`, does not reclassify Clap parser failures as reporting errors,
and does not permit a failed command to leave a partial final certified bundle.

## 13. Performance and scale acceptance criteria

The renderer reads each supplied artifact once and keeps only the projection
and one figure's prepared series at a time.  It must not clone a full raw
series per output format or retain pixel buffers after each figure.  Acceptance
criteria on the Phase D reference corpus are: one experiment with all inputs
and all eleven figures completes in under 10 s and stays below 512 MiB RSS;
an aggregate with 10,000 evidence records and 1,000 history entries completes
tables/Markdown/JSON in under 15 s and stays below 768 MiB RSS; a request for
one table does not prepare unselected figure series.  These are renderer
limits, not scientific timeouts.

## 14. Superseded mandatory implementation test plan

The reviewed 48-test inventory and conceptual fixture paragraph below are
superseded in full by sections 18.12 and 18.13.  The preserved rows are
historical traceability only and do not set the final test count.

| # | exact test name | requirement | fixture/input | expected falsification purpose |
|---:|---|---|---|---|
| 1 | `phase_d_cli_requires_mechanism_and_health` | D-R01 | `current_complete` | no incomplete public path |
| 2 | `phase_d_cli_rejects_unpaired_calibration_inputs` | D-R01 | `current_complete` | no ambiguous calibration plot |
| 3 | `phase_d_cli_rejects_unknown_selection` | D-R02 | `current_complete` | no invented figure/table |
| 4 | `phase_d_cli_rejects_existing_output_without_overwrite` | D-R03 | empty output plus collision | no implicit overwrite |
| 5 | `phase_d_cli_overwrites_only_contract_paths` | D-R03 | collision plus unrelated sentinel | no collateral deletion |
| 6 | `phase_d_reads_only_canonical_artifacts` | D-R04 | wrong-kind JSON | no bypass of canonical reader |
| 7 | `phase_d_rejects_unsupported_optional_schema` | D-R04 | `legacy_optional_schema2` | no legacy reinterpretation |
| 8 | `phase_d_schema4_health_projects_exactly_nine_dimensions` | D-R05 | `current_complete` | complete Phase C projection |
| 9 | `phase_d_schema3_health_does_not_synthesize_phase_c` | D-R05 | existing `phase_c/writer_boundary/legacy_health_assessment_v3.json` | no invented health findings |
| 10 | `phase_d_legacy_mechanism_marks_phase_b_assessment_unavailable` | D-R05 | `legacy_mechanism_schema3` | no invented Phase B assessment |
| 11 | `phase_d_legacy_unknown_lineage_remains_unknown` | D-R06 | `legacy_unknown_lineage` | no guessed ancestry |
| 12 | `phase_d_known_lineage_projects_a1_direct_dependencies_only` | D-R06 | `current_complete` + catalog | no competing lineage graph or renderer traversal |
| 13 | `phase_d_missing_catalog_is_explicit_not_complete` | D-R06 | `current_complete` no catalog | no false complete lineage |
| 14 | `phase_d_mechanism_table_projects_serialized_gate_statuses` | D-R07 | `current_complete` | no gate recalculation |
| 15 | `phase_d_mechanism_report_preserves_contradictions` | D-R07 | `contradictory_mechanism` | no hidden contradiction |
| 16 | `phase_d_mechanism_report_preserves_history` | D-R07 | `history_mechanism` | no re-promotion |
| 17 | `phase_d_health_table_preserves_dqi_reason_codes` | D-R08 | `dqi_health` | DQI not normal |
| 18 | `phase_d_health_table_preserves_indeterminate_reason_codes` | D-R08 | `indeterminate_health` | Indeterminate not healthy |
| 19 | `phase_d_health_report_preserves_excluded_evidence` | D-R08 | `excluded_evidence_health` | exclusion not silent drop |
| 20 | `phase_d_claim_mapping_never_strengthens_mechanism_status` | D-R09 unit | enum matrix | no proof wording |
| 21 | `phase_d_claim_mapping_never_strengthens_causal_status` | D-R09 unit | enum matrix | association not causality |
| 22 | `phase_d_claim_mapping_distinguishes_prediction_from_observation` | D-R09 unit | `current_complete` | model output not observed fact |
| 23 | `phase_d_claim_mapping_marks_missing_evidence_explicitly` | D-R09 unit | availability matrix | missing not negative |
| 24 | `phase_d_public_json_schema_and_order_are_stable` | D-R10 | `current_complete` | machine output contract |
| 25 | `phase_d_markdown_sections_and_order_are_stable` | D-R10 | `current_complete` | public report contract |
| 26 | `phase_d_mechanism_csv_columns_and_order_are_stable` | D-R11 | `current_complete` | table contract |
| 27 | `phase_d_health_csv_columns_and_dimension_order_are_stable` | D-R11 | `current_complete` | table contract |
| 28 | `phase_d_evidence_provenance_csv_is_deterministic` | D-R11 | `current_complete` | provenance order |
| 29 | `phase_d_current_baseline_csv_never_substitutes_zero` | D-R11 | `missing_baseline_values` | missing values stay NA |
| 30 | `phase_d_model_consistency_csv_never_recomputes_residual` | D-R11 | `model_residual_sign` | projection only |
| 31 | `phase_d_figure_mechanism_uses_stored_log_distance_only` | D-R12 | `mechanism_nonpositive_timescale` | no `log10`/filtering in renderer |
| 32 | `phase_d_figure_health_shows_all_nine_statuses` | D-R12 | `current_complete` | no first/available-only selection |
| 33 | `phase_d_figure_baseline_groups_units_without_conversion` | D-R12 | `mixed_units_baseline` | no unit conversion |
| 34 | `phase_d_figure_eis_uses_serialized_source_and_fit_series` | D-R12 | `current_complete` | no physical-file read |
| 35 | `phase_d_figure_transient_never_evaluates_components` | D-R12 | `transient_selected_fit` | no transient model execution |
| 36 | `phase_d_figure_calibration_has_no_theoretical_line` | D-R12 | `calibration_validation` | no equation evaluation |
| 37 | `phase_d_figure_signal_marks_missing_samples` | D-R12 | `signal_missing_values` | no silent drop |
| 38 | `phase_d_figure_estimation_shows_serialized_uncertainty_only` | D-R12 | `estimation_uncertainty` | no covariance recomputation |
| 39 | `phase_d_figure_model_never_maps_missing_to_zero` | D-R12 | `model_missing_contribution` | missing is not zero |
| 40 | `phase_d_figure_lineage_marks_legacy_unknown` | D-R12 | `legacy_unknown_lineage` | explicit uncertainty |
| 41 | `phase_d_selected_figure_files_are_valid_svg_and_png` | D-R13 | `current_complete` | real, nonzero valid formats |
| 42 | `phase_d_figure_metadata_has_labels_units_series_and_dqi_visibility` | D-R13 | `current_complete` | semantic figure requirements |
| 43 | `phase_d_rendering_does_not_mutate_health_assessment` | D-R14 | `current_complete` | rendering cannot change health |
| 44 | `phase_d_rendering_does_not_mutate_mechanism_assessment` | D-R14 | `current_complete` | rendering cannot change mechanism |
| 45 | `phase_d_format_selection_does_not_change_projection` | D-R14 | `current_complete` | format is non-scientific |
| 46 | `phase_d_plot_backend_failure_does_not_modify_sources` | D-R14 | forced backend failure | failure has no state mutation |
| 47 | `phase_d_repeated_render_is_deterministic` | D-R15 | `current_complete` | repeated results equivalent |
| 48 | `phase_d_large_history_does_not_duplicate_artifact_series_unboundedly` | D-R16 | `large_history` | scale/memory guard |

`current_complete` is one internally consistent set of schema-4 mechanism,
schema-4 health, and all optional current artifacts.  Each named exceptional
fixture is a minimal mutation of that set, documented by a fixture README with
the exact source field and expected availability status.  Golden files are
permitted only for `public_summary.schema1.json`, stable Markdown fragments,
CSV headers/rows, and figure semantic metadata.  Pixel equality is prohibited
as a correctness oracle; tests 31–42 assert axes, units, series counts,
required text, DQI/Indeterminate visibility, and parsed SVG/PNG validity.

## 15. Superseded requirements and acceptance traceability

| Requirement | acceptance criterion | implementation location | mandatory tests | scientific / compatibility risk |
|---|---|---|---|---|
| D-R01 single CLI route | only `report render` produces certified output | `cli.rs`, `main.rs`, `runners/report.rs` | 1–2 | competing paths |
| D-R02 explicit selections | selections are exhaustive and invalid IDs fail | `report_config.rs` | 3 | accidental output omission |
| D-R03 collision safety | no overwrite without explicit flag; no unrelated write | `runners/report.rs` | 4–5 | data loss |
| D-R04 canonical reader boundary | every input uses typed artifact reader/version gate | `reporting/reader.rs` | 6–7 | raw-data reinterpretation |
| D-R05 current/legacy projection | schema 4 and schema 3 are visibly distinct | `projection.rs`, `document.rs` | 8–10 | fabricated Phase B/C information |
| D-R06 A1 lineage reuse | no identity construction, traversal, or resolution; unknown stays unknown | `lineage.rs` | 11–13 | false provenance |
| D-R07 mechanism disclosure | all serialized support, contradictions, gates, history project | `projection.rs`, `tables.rs`, `document.rs` | 14–16 | causal overstatement |
| D-R08 health disclosure | nine dimensions, DQI, Indeterminate, exclusions project | `projection.rs`, `tables.rs`, `document.rs` | 17–19 | missing evidence appears healthy |
| D-R09 claim language safety | wording is total, deterministic, and never stronger | `claims.rs` | 20–23 | public scientific overclaim |
| D-R10 document/JSON contract | exact schemas and section ordering | `document.rs` | 24–25 | downstream incompatibility |
| D-R11 table contract | seven complete deterministic CSVs | `tables.rs` | 26–30 | ambiguity/data loss |
| D-R12 figure contract | eleven artifact-only scientific figures | `figures.rs` | 31–40 | scientific recalculation |
| D-R13 figure verification | valid format, labels, units, semantics | `figures.rs` | 41–42 | plausible but misleading figures |
| D-R14 no state mutation | source values unchanged on success/failure | `runners/report.rs` | 43–46 | changed conclusion |
| D-R15 deterministic output | repeated logical output equivalence | all reporting modules | 47 | reproducibility |
| D-R16 bounded scale | no unbounded duplicate series | projection/figures | 48 | operational reliability |

The reviewed requirement count, acceptance-criterion count, and 48-test count
are superseded by section 18.13.  Its traceability matrix is normative.

## 16. Implementation sequencing and review gates

1. Add reader and error tests (1–13) before any writer; prove all artifacts are
   canonical-reader-only.
2. Add immutable projection and claim mapping tests (14–23); scientific review
   checks that every branch copies a serialized conclusion rather than making
   one.
3. Add document/table writers and tests 24–30.
4. Add one figure at a time with tests 31–42.  Each uses serialized prepared
   series and has no regression, fit, evaluation, threshold, or source-file
   call.
5. Add end-to-end CLI/error/determinism/scale tests 43–48.
6. An independent engineering reviewer must inspect the cumulative diff against
   `1b04f22`; a scientific reviewer must inspect every public label, axis,
   caption, status mapping, and claim-language mapping before GO.

The final two-implementer and implementation-invention audit is in section
18.13, which supersedes this paragraph.

## 17. Completion checklist for the Phase D implementation branch

- [ ] Production changes conform exactly to sections 1–18 and do not edit
      Phase A0/A1/B/C semantics.
- [ ] All section 18.12 named tests exist exactly once and are substantive.
- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo check --locked` passes.
- [ ] `cargo clippy --locked --all-targets --all-features -- -D warnings`
      passes with zero diagnostics.
- [ ] `cargo test --locked --all` passes twice with no recurring instability.
- [ ] `cargo test --doc --locked` passes.
- [ ] `cargo build --locked --release` passes.
- [ ] `git diff --check` passes.
- [ ] Engineering review and scientific/high-risk review both approve the full
      cumulative diff, not merely the final commit.

Planning readiness audit: use sections 18.1–18.13, including its explicit P1
closure audit, rather than the reviewed count and error wording above.

## 18. P1 remediation contract (normative; supersedes the reviewed text where stated)

### 18.1 Review reproduction and retained boundary

This section is a forward remediation of independently reviewed commit
`916305e4606d68bce4e538f6c52f8e7fbc2c2774`, not a redesign of a PASS area.
The following reproduction record is part of the implementation contract:

| finding | reproduced plan evidence | repository evidence | required correction |
|---|---|---|---|
| PD-P1-01 | sections 3, 6, and 12 named a new `reporting::ReportingError`; section 6 assigned an invalid Clap enum to it | `src/domain/errors.rs` already exports `domain::ReportingError`; `src/runners/mod.rs` already transports it as `RunnerError::Reporting`; `src/cli.rs` owns `CliError::Parse` | one distinct Phase-D runtime error; retain the existing type and parser boundary |
| PD-P1-02 | section 4 required the generic reader for every input but gave the catalog an ad-hoc exception; no pairwise compatibility rule existed | `ArtifactLineageCatalog` is not `VersionedArtifact`; `src/runners/health.rs::load_lineage_catalog` contains the only local parse/validate pattern; A1 identities contain scope/family information | dedicated domain reader and a literal compatibility matrix |
| PD-P1-03 | section 5 only listed summary top-level names; no manifest schema existed | public artifacts have typed nested payloads and stable ordering, but no Phase-D types | typed, closed summary and manifest schemas |
| PD-P1-04 | D-FIG-03 invented a unit, D-FIG-04 negated stored imaginary impedance, and D-FIG-06 implicitly selected a candidate | `BaselineComparison` has no unit; `HealthFeature.unit` is the authority; EIS stores `z_imag_ohm`; transient results have a model but no selected-fit index | source-authority and representation-validation rules only |
| PD-P1-05 | 48 named tests did not substantively cover manifest, D-TBL-04/05, Bode, compatibility, catalog reading, publication, numeric format, or all figure defects; fixture labels were conceptual | `tests/fixtures` uses literal directory/file contracts | the reviewed predecessor expanded this to 66; the final remediation replaces it with the 73-test sealed fixture ledger and test contract in 18.11.2–18.12 |
| PD-P1-06 | section 10 said `NA` but left finite-number and cross-format representation unspecified | artifact readers reject non-finite numeric source data before projection | one exact finite formatter and serialization rules |

The second independent re-review reproduced the five remaining P1s against
the preceding remediation text. They are independently confirmed and this
section is their only replacement authority:

| finding | reproduced plan evidence | repository evidence | exact correction |
|---|---|---|---|
| PD-RR-P1-01 | a crate-private `PublicReportError` was proposed as the payload of public `RunnerError::PublicReport` | `runners::RunnerError` is public; Rust private-interface linting rejects an inaccessible payload | public reachable error/payload enums, explicit conversion proof, AC67 and strict Clippy |
| PD-RR-P1-02 | the compatibility paragraph added equality of acquisition-family sets | `health::phase_c::scope_compatible` compares only experiment, sensor, and channel scope and permits legacy lineage | extract/reuse its exact three-axis algorithm; project families only |
| PD-RR-P1-03 | the public summary and manifest used undefined copied-payload shorthand and undeclared status records | public serialized sources have concrete fields but no Phase-D public type graph | sections 18.5–18.6 define every public field, nullable form, ordering, enum token, and source authority |
| PD-RR-P1-04 | D-TBL-04 mixed source/dependency and catalog-node ordering; D-FIG-03 left warning comparability unresolved | `FeatureComparability::ComparableWithWarnings` is produced by within-tolerance context difference in `health::normalization::comparable` | one tagged root/dependency row model and one rendered-with-warning policy shared by D-FIG-03/D-TBL-07 |
| PD-RR-P1-05 | fixture text permitted omitted values and named a malformed catalog without a required syntax-error test | canonical readers validate actual artifact payload/lineage; `read_artifact_lineage_catalog` must distinguish parse/shape failure | literal fixture manifest, identity/provenance revalidation, exact malformed bytes, AC68 and AC69 |

PD-P1-06 is already closed and is deliberately reproduced only as the
unchanged section 18.9 regression contract.

The Phase-D projection-only boundary in sections 1, 2, 7–11 is retained.  A
Phase-D transformation is permitted only when this section expressly calls it
presentation formatting or display geometry; it must not alter a scientific
coordinate, select an assessment, calculate an absent quantity, infer
compatibility, or create new evidence.

### 18.2 One error architecture and normative boundary matrix

`domain::ReportingError` remains public, re-exported, and **unchanged**.  It
continues to cover pre-existing fit/search reporting only.  Phase D adds one
new public error vocabulary because `runners::RunnerError` is public and its
new payload must be publicly nameable.  `src/reporting/mod.rs` is `pub mod
reporting`; `src/reporting/error.rs` is a public child module; and
`reporting` re-exports `PublicReportError`, `CompatibilityAxis`,
`PublicationPhase`, and `AvailabilityReason`.  Thus the canonical external
path is `rust_electroanalysis_cli::reporting::PublicReportError` and every
named payload type below is public and reachable at that path or through the
already-public `domain` module.

```text
pub enum PublicReportError {
  InvalidCombination { detail: &'static str },
  InvalidSelection { selector: &'static str, value: String },
  Artifact { flag: &'static str, path: PathBuf, source: ArtifactError },
  LineageCatalog { path: PathBuf, source: LineageCatalogReadError },
  RequiredInputsIncompatible { left_flag: &'static str, right_flag: &'static str,
                               axis: CompatibilityAxis, left: String, right: String },
  OptionalInputIncompatible { flag: &'static str, required_flag: &'static str,
                               axis: CompatibilityAxis, actual: String, expected: String },
  InvalidOutputDirectory { path: PathBuf },
  OutputCollision { path: PathBuf },
  UnmanagedOutputEntry { path: PathBuf },
  RequestedOutputUnavailable { output_id: String, reason: AvailabilityReason },
  Staging { path: PathBuf, source: io::Error },
  Write { path: PathBuf, source: io::Error },
  Csv { path: PathBuf, source: csv::Error },
  Serialization { path: PathBuf, source: serde_json::Error },
  PlotBackend { figure_id: String, path: PathBuf, message: String },
  StagingValidation { path: PathBuf, detail: String },
  Publication { phase: PublicationPhase, staging_path: PathBuf,
                backup_path: Option<PathBuf>, source: io::Error },
  Cleanup { path: PathBuf, source: io::Error }
}
```

`CompatibilityAxis`, `PublicationPhase`, and `AvailabilityReason` are public
closed enums with these exact Rust variants/JSON tokens:

```text
CompatibilityAxis { ExperimentScope = "experiment_scope",
  SensorScope = "sensor_scope", ChannelScope = "channel_scope" }
PublicationPhase { BackupRename = "backup_rename", PublishRename = "publish_rename",
  RestoreRename = "restore_rename", BackupCleanup = "backup_cleanup" }
AvailabilityReason { NotProvided = "not_provided", NotSelected = "not_selected",
  LegacyPhaseCNotSerialized = "legacy_phase_c_not_serialized",
  LegacyMechanismAssessmentNotSerialized = "legacy_mechanism_assessment_not_serialized",
  LineageLegacyUnknown = "lineage_legacy_unknown",
  UnitAuthorityUnavailable = "unit_authority_unavailable", NotComparable = "not_comparable",
  ComparisonUnknown = "comparison_unknown", NoComparableFinitePair = "no_comparable_finite_pair",
  SelectedFitNotFound = "selected_fit_not_found", SelectedFitAmbiguous = "selected_fit_ambiguous",
  SerializedSeriesInvalid = "serialized_series_invalid",
  SerializedSeriesUnavailable = "serialized_series_unavailable",
  PairedInputNotProvided = "paired_input_not_provided",
  CatalogNotSupplied = "catalog_not_supplied" }
```

They map one-for-one to the similarly named public JSON output vocabulary in
section 18.5. `RunnerError` gains
exactly `PublicReport(#[from] reporting::PublicReportError)`; its existing
`Reporting(#[from] domain::ReportingError)` variant stays unchanged.
`ApplicationError` continues to receive this through its existing
`Runner(#[from] RunnerError)` conversion.  No public API gains a second type
called `ReportingError`.

Reachability is intentionally single-path and compile-checkable:
`RunnerError::PublicReport` has payload `reporting::PublicReportError`; the
`#[from]` implementation is generated from that same public type; its
`Artifact` source is public `domain::ArtifactError`; its `LineageCatalog`
source is public `domain::LineageCatalogReadError`; and its remaining payloads
are `String`, `PathBuf`, `io::Error`, `csv::Error`, `serde_json::Error`, or the
three public reporting enums.  No `pub` item mentions a `pub(crate)` type.
The planned integration test imports both `runners::RunnerError` and
`reporting::PublicReportError`, constructs `RunnerError::from` the latter, and
pattern-matches `RunnerError::PublicReport`; `cargo clippy --all-targets
--all-features -- -D warnings` is the required strict private-interface
falsification check.

| stage / failure | exact owner and variant | payload | final certified-output visibility / terminal behavior |
|---|---|---|---|
| Clap enum/value unknown; a required clap positional/flag absent | `CliError::Parse(clap::Error)` | Clap diagnostic | runner not called; no output path examined; normal clap error exit |
| parsed calibration pair absent, or no text format remains | `PublicReportError::InvalidCombination` | fixed detail string | no staging; runtime error exit |
| parsed figure/table selector unknown or repeated | `PublicReportError::InvalidSelection` | selector and offending token | no staging; runtime error exit |
| required file missing, JSON invalid, wrong kind, unsupported schema, or semantic validation fails | `PublicReportError::Artifact` | flag, path, original `ArtifactError` | no staging; runtime error exit |
| catalog missing, malformed, duplicate-keyed, schema != 1, key/identity mismatch, or lineage-invalid | `PublicReportError::LineageCatalog` | path and `LineageCatalogReadError` | no staging; runtime error exit |
| known required inputs disagree on a required axis | `RequiredInputsIncompatible` | both flags, axis, canonical values | no staging; runtime error exit |
| a supplied optional input or paired calibration input disagrees on a required axis | `OptionalInputIncompatible` | flags, axis, actual/expected | no staging; runtime error exit |
| final root invalid, existing without overwrite, or unmanaged entry under overwrite | `InvalidOutputDirectory`, `OutputCollision`, or `UnmanagedOutputEntry` | final path | no staging; runtime error exit |
| creation/write/render/CSV/JSON/staging verification fails | `Staging`, `Write`, `Csv`, `Serialization`, `PlotBackend`, or `StagingValidation` | exact staging-relative path and source/detail | staging removed if cleanup succeeds; final bundle unchanged or absent |
| an explicitly selected unavailable output is discovered after projection | `RequestedOutputUnavailable` | ID and closed availability reason | staging removed; final bundle unchanged or absent |
| backup/publish/restore/backup-cleanup rename fails | `Publication` | phase, staging path, optional backup path, source | no partial final bundle; previous bundle is restored when possible; retained recovery paths are not certified output |
| removal of a non-certified staging/backup directory fails | `Cleanup` | path and source | no partial final bundle; retained path is reported as non-certified recovery material |

No `completed_paths` field exists or is required: all successful generated
files live in staging until the one directory-level publication operation.

### 18.3 Certified output atomicity

The publication model is exactly **preflight → private staging → complete
validation → directory publication**.

1. Preflight canonical-reads every supplied input, validates all compatibility,
   resolves default/explicit selection, builds the complete output path set,
   and tests collision policy before creating a directory.
2. `OUTPUT` is the final bundle root.  Its staging sibling is
   `PARENT/.<output-file-name>.phase-d-staging-<pid>-<attempt>` where `pid` is
   the current process ID and `attempt` starts at 0 and increments only after
   a `create_dir` collision.  It is process-local, is never a final path, and
   is never recorded as an output in a successful manifest.
3. Every requested artifact, including `render_manifest.schema1.json`, is
   written below staging using the final relative path.  The manifest is
   constructed last from the already determined statuses, then the staging
   tree is verified to contain exactly the expected relative paths, UTF-8
   text outputs, two image formats for every written figure, and no temporary
   writer file.
4. Without `--overwrite`, `OUTPUT` must be absent and `rename(STAGING, OUTPUT)`
   is the only publication operation.  A rename failure returns
   `Publication { phase: publish_rename, ... }`; no final bundle exists.
5. With `--overwrite`, `OUTPUT` must be a directory whose complete recursive
   file set is exactly a prior Phase-D contract set: public summary if prior
   JSON was selected, Markdown if prior Markdown was selected, one manifest,
   and only recognized `tables/` and `figures/` paths.  Any extra file,
   directory, symlink, or unrecognized path fails `UnmanagedOutputEntry` and
   is not touched.  The final root is renamed to sibling
   `.<output-file-name>.phase-d-backup-<pid>-<attempt>`, staging is renamed to
   `OUTPUT`, then the backup is removed.  If publish fails, the runner renames
   backup back to `OUTPUT`; a failed restore reports `Publication` with both
   recovery paths.  A failed backup cleanup reports `Cleanup` but the new
   complete bundle remains valid.
6. Staging is removed after every failure before publication when removal
   succeeds.  A retained staging or backup is explicitly non-certified and
   must not be consumed as an output.  Inputs are read-only.  There is no
   partial certified output state.

### 18.4 Canonical input readers, catalog reader, and compatibility

The canonical reader table replaces all reader ambiguity:

| flag | type / exact accepted schema | canonical reader | required | compatibility target | failure |
|---|---|---|---:|---|---|
| `--mechanism` | `MechanismAnalysisReport`, 4; 1–3 legacy | `domain::read_artifact` | yes | health | `Artifact` |
| `--health` | `SensorHealthAssessment`, 4 or legacy 3; 1–2 rejected | `domain::read_artifact` | yes | mechanism | `Artifact` |
| `--lineage-catalog` | `ArtifactLineageCatalog`, exactly 1; no `ArtifactKind` | `domain::read_artifact_lineage_catalog` | no | none; catalog is provenance-only | `LineageCatalog` |
| `--eis` | `EisFitArtifact`, exactly 3 | `domain::read_artifact` | no | mechanism and health | `Artifact` |
| `--transient` | `TransientAnalysisReport`, exactly 3 | `domain::read_artifact` | no | mechanism and health | `Artifact` |
| `--calibration` | `CalibrationAnalysisReport`, exactly 3 | `domain::read_artifact` | paired | mechanism, health, observations | `Artifact` |
| `--calibration-observations` | `CalibrationObservationSet`, exactly 3 | `domain::read_artifact` | paired | mechanism, health, calibration | `Artifact` |
| `--signal` | `SignalAnalysisReport`, exactly 3 | `domain::read_artifact` | no | mechanism and health | `Artifact` |
| `--estimation` | `StateEstimationReport`, exactly 4 | `domain::read_artifact` | no | mechanism and health | `Artifact` |
| `--model` | `ModelAnalysisReport`, exactly 5 | `domain::read_artifact` | no | mechanism and health | `Artifact` |

The new dedicated reader is `pub fn
read_artifact_lineage_catalog(path: &Path) -> Result<ArtifactLineageCatalog,
LineageCatalogReadError>` in `src/domain/lineage.rs`, re-exported by
`src/domain/mod.rs`.  `LineageCatalogReadError` is public and closed:
`Io { path, source }`, `Json { path, source }`, `InvalidRoot { path }`,
`UnknownField { path, field }`, `DuplicateField { path, field }`,
`DuplicateArtifactKey { path, key }`, `UnsupportedSchemaVersion { path,
actual }`, `KeyIdentityMismatch { path, key, identity }`, and `Validation {
path, source: LineageError }`.  It reads UTF-8 inside `domain`, uses a custom
serde map visitor for the root and `artifacts` map so duplicate JSON keys fail
before construction, permits only root fields `schema_version` and `artifacts`,
requires schema 1, requires every map key equal to the node identity ID, and
inserts every node into a fresh `ArtifactLineageCatalog` so existing identity,
family, dependency order, and dependency-duplicate validation runs.  Reporting
calls this one function and contains no `serde_json::from_*` call for catalog
content.

Compatibility does not reassess science.  The Phase-D gate is the exact
Phase-C `src/health/phase_c.rs::scope_compatible` algorithm, factored into a
crate-private shared helper without changing its branches: when both lineage
states are `Known`, reject only if `experiment_scope`, `sensor_scope`, or
`channel_scope` differs; otherwise return true.  When either state is
`LegacyUnknown`, return true.  In particular, equal `Unknown` experiment scope
or equal `Unspecified` scope keys remain admissible exactly as Phase C treats
them.  Phase D must not introduce a `not_verifiable` branch or a fourth
scientific compatibility axis.

`ArtifactAcquisitionFamilies` is never read by the gate.  It is projected as
provenance only: known values appear in their stored normalized order, unknown
is displayed as `unknown`, and a legacy source as `legacy_unknown`.  Equal or
different known sets never cause rejection and never cause the renderer to say
`independent`, `dependent`, or `same_source`; only an already serialized A1 or
Phase-C independence conclusion may be displayed where its source field is
explicitly listed.

| pair | exact gate | legacy behavior | supplied but unselected |
|---|---|---|---|
| mechanism ↔ health | apply the shared Phase-C helper once | permit and record `legacy_unknown`; make no cross-artifact claim | n/a |
| EIS, transient, signal, estimation, model ↔ mechanism, then health | apply the helper against each required artifact, in that order | permit and record `legacy_unknown` | read, gate, and record before selection |
| calibration ↔ calibration-observations, then each ↔ mechanism, then health | apply the helper in that exact order | permit and record `legacy_unknown`; pair output rules still apply | read, gate, and record before selection |
| supplied catalog ↔ any artifact | no gate | catalog may contain nodes outside the bundle | same |

For two known inputs, the first unequal axis in the ordered sequence
`experiment_scope`, `sensor_scope`, `channel_scope` raises
`RequiredInputsIncompatible` or `OptionalInputIncompatible`; no second axis is
reported.  A successful known comparison serializes `compatible`; a successful
comparison containing a legacy source serializes `legacy_unknown`; omitted
optional inputs serialize `not_provided`; the catalog serializes
`not_applicable`.  All supplied optional inputs are read and checked before
selection, so a known scope mismatch aborts even when its figure is not
selected.  These are presentation records of the reused gate, not a new
compatibility theory.

### 18.5 Closed `public_summary.schema1.json`

`public_summary.schema1.json` is UTF-8 LF pretty JSON from one typed serializer
in declaration order.  It is a deliberately bounded public projection: the
complete detailed series remain in the named CSV/figure outputs, while this
file contains every public assessment, source reference, compatibility result,
availability result, and limitation needed to interpret those outputs.  It
does not embed an unbounded or opaque upstream artifact payload.

```text
PublicSummaryV1 {
  schema_version: u32 = 1,
  output_kind: String = "phase_d_public_scientific_output",
  renderer_contract: String = "mhi_v1_phase_d_public_output_v1",
  route: String = "electroanalysis report render",
  input_references: Vec<PublicInputReferenceV1>,
  compatibility: PublicCompatibilityV1,
  mechanism: PublicMechanismSectionV1,
  sensor_health: PublicHealthSectionV1,
  optional_sources: Vec<PublicOptionalSourceV1>,
  lineage: PublicLineageSectionV1,
  outputs: PublicOutputIndexV1,
  limitations: Vec<PublicLimitationV1>,
  rendering: PublicRenderingMetadataV1
}

PublicInputReferenceV1 =
  Artifact(PublicArtifactInputReferenceV1) |
  LineageCatalog(PublicLineageCatalogInputReferenceV1)

PublicArtifactInputReferenceV1 {
  input_kind: PublicInputReferenceKindV1 = artifact,
  input_flag: ArtifactInputFlagV1,
  supplied_path_basename: Option<String>, artifact_kind: Option<ArtifactKindV1>,
  schema_version: Option<u32>, lineage: Option<LineagePresentationV1>,
  acquisition_families: Option<AcquisitionFamilyPresentationV1>,
  availability: AvailabilityV1
}

PublicLineageCatalogInputReferenceV1 {
  input_kind: PublicInputReferenceKindV1 = lineage_catalog,
  supplied_path_basename: Option<String>, schema_version: Option<u32>,
  availability: AvailabilityV1, validation: CatalogValidationV1
}

PublicCompatibilityV1 { required_pair: CompatibilityStatusV1,
  optional: Vec<CompatibilityRecordV1> }
CompatibilityRecordV1 { input_flag: InputFlagV1, against_flag: InputFlagV1,
  status: CompatibilityStatusV1, mismatch_axis: Option<CompatibilityAxisV1> }

PublicMechanismSectionV1 { availability: AvailabilityV1, analysis_id: String,
  hypotheses: Vec<PublicHypothesisV1>, comparisons: Vec<PublicTimescaleComparisonV1>,
  warning_messages: Vec<PublicMessageV1>, lineage: LineagePresentationV1,
  provenance: ProvenancePresentationV1 }
PublicHypothesisV1 { hypothesis_id: String, display_name: String,
  target_components: Vec<String>, evidence_level: HypothesisEvidenceLevelV1,
  reason_codes: Vec<PhaseBHypothesisReasonCodeV1> }
PublicTimescaleComparisonV1 { comparison_id: String, record_id: String,
  eis_timescale_id: String, transient_timescale_id: String, ratio: Option<f64>,
  log10_distance: Option<f64>, symmetric_relative_difference: Option<f64>,
  confidence_interval_overlap: Option<bool>, compatibility_probability: Option<f64>,
  evidence_level: MechanismEvidenceLevelV1, warnings: Vec<PublicMessageV1> }

PublicHealthSectionV1 { availability: AvailabilityV1, assessment_id: String,
  sensor_id: Option<String>, experiment_id: Option<String>,
  overall_status: OverallHealthStatusV1, dimensions: Vec<PublicHealthDimensionV1>,
  features: Vec<PublicHealthFeatureV1>, baseline_comparisons: Vec<PublicBaselineComparisonV1>,
  warning_codes: Vec<HealthWarningV1>, lineage: LineagePresentationV1,
  provenance: ProvenancePresentationV1 }
PublicHealthDimensionV1 { dimension: HealthDimensionV1, status: OverallHealthStatusV1,
  evidence_state: HealthEvidenceStateV1, interpretation_category: HealthInterpretationCategoryV1,
  causal_status: CausalStatusV1, reason_codes: Vec<PhaseCHealthReasonCodeV1>,
  source_evidence_ids: Vec<String>, source_artifact_ids: Vec<String>,
  excluded_evidence_ids: Vec<String> }
PublicHealthFeatureV1 { name: String, value: Option<f64>, unit: String,
  domain: HealthDomainV1, source: String, warning: Option<String> }
PublicBaselineComparisonV1 { feature: String, current_value: Option<f64>,
  baseline_value: Option<f64>, comparability: FeatureComparabilityV1,
  absolute_difference: Option<f64>, relative_difference: Option<f64>, log_ratio: Option<f64>,
  z_score: Option<f64>, robust_z_score: Option<f64>, empirical_percentile: Option<f64>,
  range_position_percent: Option<f64>, override_reason: Option<String>,
  baseline_sample_count: u64 }

PublicOptionalSourceV1 { kind: OptionalSourceKindV1, availability: AvailabilityV1,
  compatibility: CompatibilityStatusV1, input: Option<PublicInputReferenceV1>,
  detail: Option<OptionalSourceDetailV1> }
OptionalSourceDetailV1 { analysis_id: Option<String>, record_count: u64,
  measurement_unit: Option<String>, lineage: LineagePresentationV1,
  provenance: ProvenancePresentationV1 }

PublicLineageSectionV1 { catalog_supplied: bool, roots: Vec<PublicLineageRootV1> }
PublicLineageRootV1 { input_flag: InputFlagV1, lineage: LineagePresentationV1,
  direct_dependencies: Vec<PublicDependencyV1>, root_catalog_entry_present: Option<bool> }
LineagePresentationV1 { status: LineagePresentationStatusV1,
  identity: Option<PublicArtifactIdentityV1>, legacy_source_schema_version: Option<u32>,
  legacy_reason: Option<LegacyLineageReasonV1> }
PublicArtifactIdentityV1 { artifact_id: String, artifact_kind: ArtifactKindV1,
  schema_version: u32, producer_version: String, experiment_scope: ExperimentScopeV1,
  sensor_scope: ScopeKeyV1, channel_scope: ScopeKeyV1,
  acquisition_families: AcquisitionFamilyPresentationV1, semantic_sha256: String }
PublicDependencyV1 { artifact_id: String, artifact_kind: ArtifactKindV1,
  role: DependencyRoleV1 }
AcquisitionFamilyPresentationV1 { status: AcquisitionFamilyStatusV1,
  values: Vec<String> }
ProvenancePresentationV1 { software_version: String, input_sha256: String,
  configuration_sha256: Option<String>, git_commit: Option<String> }

PublicOutputIndexV1 { tables: Vec<PublicOutputStatusV1>, figures: Vec<PublicOutputStatusV1> }
PublicOutputStatusV1 { output_kind: GeneratedOutputKindV1, output_id: String,
  relative_path: Option<String>, format: Option<RenderFormatV1>, status: RenderStatusV1,
  reason: Option<AvailabilityReasonV1> }
PublicLimitationV1 { code: WarningCodeV1, message: String,
  input_flag: Option<InputFlagV1>, output_id: Option<String> }
PublicMessageV1 { code: WarningCodeV1, message: String }
PublicRenderingMetadataV1 { json_schema: String = "public_summary.schema1",
  numeric_format: String = "rust_display_normalized_negative_zero_v1",
  csv_newline: String = "LF", timestamp: Option<String> = null }
```

`PublicInputReferenceV1` is serialized as one object, not as a Rust externally
tagged enum: it uses `#[serde(tag = "input_kind", rename_all = "snake_case")]`.
The exact artifact object is

```text
{ "input_kind": "artifact", "input_flag": ArtifactInputFlagV1,
  "supplied_path_basename": String|null, "artifact_kind": ArtifactKindV1|null,
  "schema_version": u32|null, "lineage": LineagePresentationV1|null,
  "acquisition_families": AcquisitionFamilyPresentationV1|null,
  "availability": AvailabilityV1 }
```

and the exact catalog object is

```text
{ "input_kind": "lineage_catalog", "supplied_path_basename": String|null,
  "schema_version": u32|null, "availability": AvailabilityV1,
  "validation": CatalogValidationV1 }
```

`input_references` contains nine artifact objects in `ArtifactInputFlagV1`
order followed by exactly one lineage-catalog object.  An absent artifact has
`supplied_path_basename`, `artifact_kind`, `schema_version`, `lineage`, and
`acquisition_families` as `null`; it is not a legacy artifact and no state is
invented for it. A supplied legacy artifact has its actual
`LegacyUnknown` lineage and `legacy_unknown` families presentation. A
selected, unavailable artifact retains the source-derived fields that were
read before the unavailable projection; an unread absent source has only the
null form. A supplied artifact copies all five artifact-specific fields from
the canonical reader.  A catalog that is
supplied and successfully read has `availability="available"`,
`validation="validated"`, its supplied basename, and `schema_version=1`.  An
absent catalog has `availability="not_provided"`,
`validation="not_applicable"`, and both nullable fields `null`.  A catalog
whose schema is unsupported, whose JSON or structure is invalid, whose root
has duplicate keys, whose map key does not match its node identity, or whose
node/dependency validation fails raises `PublicReportError::LineageCatalog`;
therefore it has no successful-summary reference.  A structurally valid
catalog may have a dependency not represented by another catalog node; that is
node-level resolution information and still uses the supplied/validated
catalog reference.  `LegacyUnknown` is a state of an input artifact, not a
catalog node or catalog reference.

The catalog object deliberately has no `input_flag`, `artifact_id`,
`artifact_kind`, `lineage`, `acquisition_families`, direct dependencies,
aggregate families, or synthetic identity.  `ArtifactLineageCatalog` supplies
only its own `schema_version` and its node map; family and lineage data belong
to individual `ArtifactLineageNode`s and are projected only through the
dedicated artifact lineage roots/table.  A catalog is provenance metadata, not
scientific evidence and not an `ArtifactIdentity` root.

All fields above are required. `Option` is encoded as JSON `null`; `Vec` is
encoded as `[]`; no key is omitted and no `serde_json::Value`, map, or generic
wrapper is allowed.  `analysis_id` and `assessment_id` are the validated
required source values; a legacy source still supplies its source value and is
distinguished only by `availability`/`lineage`, never by a fabricated ID.
`u64` values are serialized as JSON integral numbers.  Numeric values are JSON
numbers subject to section 18.9, not formatted strings.

The closed enum vocabulary is as follows.  The listed token is both the Rust
`#[serde(rename_all = "snake_case")]` spelling and the JSON value:

| enum | tokens | source authority |
|---|---|---|
| `InputFlagV1` | `mechanism`, `health`, `eis`, `transient`, `calibration`, `calibration_observations`, `signal`, `estimation`, `model`, `lineage_catalog` | fixed CLI flag inventory in 18.4 |
| `ArtifactInputFlagV1` | `mechanism`, `health`, `eis`, `transient`, `calibration`, `calibration_observations`, `signal`, `estimation`, `model` | `InputFlagV1` excluding `lineage_catalog`; only this type may select the `Artifact` input-reference variant |
| `PublicInputReferenceKindV1` | `artifact`, `lineage_catalog` | closed tagged-union discriminator; `#[serde(tag = "input_kind")]`, never inferred from a nullable field |
| `ArtifactKindV1` | `eis_fit`, `transient_analysis`, `calibration_observations`, `calibration_model`, `calibration_analysis`, `signal_analysis`, `health_baseline`, `health_assessment`, `health_trend`, `mechanism_analysis`, `state_estimation`, `ism_model_compilation`, `ism_model_analysis`, `ism_model_validation` | exact one-for-one projection of `src/domain/artifact.rs::ArtifactKind`: `EisFit/eis_fit`, `TransientAnalysis/transient_analysis`, `CalibrationObservations/calibration_observations`, `CalibrationModel/calibration_model`, `CalibrationAnalysis/calibration_analysis`, `SignalAnalysis/signal_analysis`, `HealthBaseline/health_baseline`, `HealthAssessment/health_assessment`, `HealthTrend/health_trend`, `MechanismAnalysis/mechanism_analysis`, `StateEstimation/state_estimation`, `ModelCompilation/ism_model_compilation`, `ModelAnalysis/ism_model_analysis`, and `ModelValidation/ism_model_validation` |
| `AvailabilityV1` | `available`, `available_with_warnings`, `not_provided`, `not_selected`, `unavailable` | reader/projection/selection outcome; only the first two permit populated source-derived detail |
| `CatalogValidationV1` | `validated`, `not_applicable` | `validated` only after `read_artifact_lineage_catalog` succeeds; `not_applicable` only when the catalog flag is absent. Parse, schema, duplicate-key, key/identity, and node-validation failures abort before either public document exists. |
| `CompatibilityStatusV1` | `compatible`, `legacy_unknown`, `not_provided`, `not_applicable` | exact Phase-C scope gate in 18.4; `incompatible` never serializes because it aborts |
| `CompatibilityAxisV1` | `experiment_scope`, `sensor_scope`, `channel_scope` | first unequal Phase-C axis in that order; acquisition families are excluded |
| `LineagePresentationStatusV1` | `known`, `legacy_unknown` | `ArtifactLineageState` tag |
| `LegacyLineageReasonV1` | `field_absent_in_legacy_artifact`, `external_artifact_without_lineage`, `migration_information_unavailable` | `UnknownLineageReason` |
| `AcquisitionFamilyStatusV1` | `known`, `unknown`, `legacy_unknown` | `ArtifactAcquisitionFamilies` or legacy lineage; `values=[]` unless status is `known` |
| `OptionalSourceKindV1` | `eis`, `transient`, `calibration`, `signal`, `estimation`, `model`, `lineage_catalog` | supplied optional source kind; observations are represented by the paired calibration record |
| `RenderStatusV1` | `written`, `unavailable`, `not_selected` | completed writer/selection result |
| `GeneratedOutputKindV1` | `summary`, `markdown`, `table`, `figure`, `manifest` | output path class |
| `RenderFormatV1` | `json`, `markdown`, `csv`, `svg`, `png` | output extension/writer |
| `AvailabilityReasonV1` | `not_provided`, `not_selected`, `legacy_phase_c_not_serialized`, `legacy_mechanism_assessment_not_serialized`, `lineage_legacy_unknown`, `unit_authority_unavailable`, `not_comparable`, `comparison_unknown`, `no_comparable_finite_pair`, `selected_fit_not_found`, `selected_fit_ambiguous`, `serialized_series_invalid`, `serialized_series_unavailable`, `paired_input_not_provided`, `catalog_not_supplied` | exact source/selection condition |
| `FeatureComparabilityV1` | `comparable`, `comparable_with_warnings`, `not_comparable`, `unknown` | `FeatureComparability` |
| `HypothesisEvidenceLevelV1` | `not_assessed`, `hypothesized`, `experimentally_supported`, `validated_for_domain`, `contradicted` | `HypothesisEvidenceLevel` |
| `MechanismEvidenceLevelV1` | `none`, `weak`, `moderate`, `strong`, `contradictory`, `insufficient` | `EvidenceLevel` |
| `PhaseBHypothesisReasonCodeV1` | `missing_required_evidence`, `critical_contradiction`, `timescale_satisfied`, `amplitude_satisfied`, `repeatability_satisfied`, `identifiability_satisfied`, `validation_satisfied` | `PhaseBHypothesisReasonCode` |
| `HealthDimensionV1` | `signal_integrity`, `calibration_health`, `dynamic_response_health`, `reference_stability`, `environmental_robustness`, `model_consistency`, `observability`, `uncertainty_health`, `data_quality` | `HealthDimension::ALL` |
| `OverallHealthStatusV1` | `within_baseline`, `watch`, `degraded`, `critical`, `data_quality_insufficient`, `indeterminate` | `OverallHealthStatus` |
| `HealthEvidenceStateV1` | `adequate_evidence`, `no_evidence`, `insufficient_evidence`, `poor_data_quality`, `contradictory_evidence` | `HealthEvidenceState` |
| `PhaseCHealthReasonCodeV1` | `optional_source_absent`, `required_quantity_absent`, `invalid_quantity`, `unit_mismatch`, `source_incompatible`, `scope_incompatible`, `temporal_incompatible`, `incomplete_lineage`, `independence_unknown`, `insufficient_independent_families`, `baseline_absent`, `baseline_insufficient`, `baseline_incomparable`, `quality_gate_failed`, `threshold_within_limit`, `threshold_watch`, `threshold_degraded`, `threshold_critical`, `contradictory_evidence`, `model_outside_domain`, `model_validity_unavailable`, `observability_failed`, `uncertainty_incomplete`, `mechanism_noncausal`, `mechanism_contradicted`, `reference_anchor_unavailable`, `phase_b_hypothesis_unmapped`, `selected_transient_event_absent`, `selected_transient_event_ambiguous`, `selected_transient_event_invalid`, `baseline_feature_absent`, `baseline_statistic_absent`, `baseline_denominator_zero`, `baseline_denominator_near_zero`, `optional_invalid_source_excluded` | `PhaseCHealthReasonCode` |
| `HealthInterpretationCategoryV1` | `observed_behavior`, `model_inconsistency`, `environmental_effect`, `calibration_issue`, `possible_physical_degradation` | `HealthInterpretationCategory` |
| `CausalStatusV1` | `observed`, `associated`, `hypothesized`, `experimentally_supported`, `validated_for_domain`, `indeterminate` | `CausalStatus` |
| `HealthDomainV1` | `data_quality`, `signal_noise`, `drift`, `dynamic_response`, `calibration`, `impedance`, `mechanism_evidence` | `HealthDomain` |
| `HealthWarningV1` | `missing_baseline`, `insufficient_baseline_records`, `baseline_variance_unavailable`, `feature_noncomparable`, `missing_signal_artifact`, `missing_transient_artifact`, `missing_calibration_artifact`, `missing_eis_artifact`, `missing_mechanism_artifact`, `artifact_schema_mismatch`, `artifact_configuration_mismatch`, `environmental_mismatch`, `insufficient_evidence_domains`, `contradictory_evidence`, `rule_condition_unavailable`, `semantic_role_unavailable`, `assessment_based_on_warning_bearing_fits`, `invalid_rule`, `non_finite_artifact`, `mixed_analyte_context`, `mixed_sample_matrix_context`, `mixed_sensor_design_context`, `mixed_sensor_type_context`, `mixed_temperature_context` | `SensorHealthAssessment.warnings`; no renderer-created status |
| `WarningCodeV1` | `source_warning`, `baseline_comparable_with_warnings`, `legacy_input`, `legacy_lineage`, `catalog_not_supplied`, `output_unavailable` | source warning or the listed presentation condition |
| `ExperimentScopeV1` | `single { experiment_id: String }`, `aggregate { aggregate_scope_id: String, member_experiment_ids: Vec<String> }`, `unknown` | `ArtifactExperimentScope` tagged representation |
| `ScopeKeyV1` | `specific { value: String }`, `all`, `unspecified` | `ScopeKey` tagged representation |
| `DependencyRoleV1` | `initialization`, `calibration`, `prior`, `constraint`, `transformation_input`, `auxiliary_input`, `validation_input`, `derived_from` | one-for-one `ArtifactDependencyRole` projection; order uses its discriminant 0–7 |

Source order is the fixed input-flag order above; mechanism hypotheses sort by
`hypothesis_id`; comparisons sort by `comparison_id`; health dimensions are
`HealthDimension::ALL`; features and comparisons sort by `feature` then unit;
messages retain producer order; direct dependencies retain A1 canonical order;
tables and figures retain their inventory order.  Every value is copied from
the named serialized source field in the type graph, except availability,
compatibility, output status, limitation, and rendering metadata, whose source
authority is stated in the enum table.  The serializer may not add a field,
omit a field, convert an enum to a free-form string, or derive a scientific
conclusion.

### 18.6 Closed `render_manifest.schema1.json`

The render manifest is presentation provenance, not a scientific artifact: it
has no manifest-level `ArtifactIdentity`, artifact kind, lineage root, or
dependency registration, and cannot substitute for A1 lineage. Its artifact
input-reference variant copies an input artifact kind only where a real
artifact was read. Its complete closed type graph is:

```text
RenderManifestV1 {
  schema_version: u32 = 1,
  output_kind: String = "phase_d_render_manifest",
  renderer_contract: String = "mhi_v1_phase_d_public_output_v1",
  route: String = "electroanalysis report render",
  final_output_status: FinalOutputStatusV1,
  input_references: Vec<ManifestInputReferenceV1>,
  requested: RequestedOutputSelectionV1,
  render_order: Vec<ManifestRenderStepV1>,
  generated_files: Vec<ManifestGeneratedFileV1>,
  unavailable_outputs: Vec<ManifestUnavailableOutputV1>,
  warnings: Vec<ManifestWarningV1>,
  legacy_input_notices: Vec<ManifestLegacyNoticeV1>,
  optional_compatibility: Vec<ManifestCompatibilityOutcomeV1>,
  determinism: ManifestDeterminismV1
}

ManifestInputReferenceV1 =
  Artifact(ManifestArtifactInputReferenceV1) |
  LineageCatalog(ManifestLineageCatalogInputReferenceV1)
ManifestArtifactInputReferenceV1 {
  input_kind: ManifestInputReferenceKindV1 = artifact,
  input_flag: ArtifactInputFlagV1, supplied_path_basename: Option<String>,
  artifact_kind: Option<ArtifactKindV1>, schema_version: Option<u32>,
  lineage: Option<LineagePresentationV1>,
  acquisition_families: Option<AcquisitionFamilyPresentationV1>,
  availability: AvailabilityV1, compatibility: CompatibilityStatusV1 }
ManifestLineageCatalogInputReferenceV1 {
  input_kind: ManifestInputReferenceKindV1 = lineage_catalog,
  supplied_path_basename: Option<String>, schema_version: Option<u32>,
  availability: AvailabilityV1, validation: CatalogValidationV1,
  compatibility: CompatibilityStatusV1 = not_applicable }
RequestedOutputSelectionV1 { formats: Vec<RenderFormatV1>, figures: Vec<FigureIdV1>,
  tables: Vec<TableIdV1>, figures_mode: SelectionModeV1,
  tables_mode: SelectionModeV1, overwrite: bool }
ManifestRenderStepV1 { ordinal: u32, output_kind: GeneratedOutputKindV1,
  output_id: Option<String>, relative_path: String }
ManifestGeneratedFileV1 { relative_path: String, output_kind: GeneratedOutputKindV1,
  output_id: Option<String>, format: RenderFormatV1, status: RenderStatusV1,
  source_input_flags: Vec<InputFlagV1> }
ManifestUnavailableOutputV1 { output_kind: GeneratedOutputKindV1,
  output_id: String, reason: AvailabilityReasonV1 }
ManifestWarningV1 { code: WarningCodeV1, message: String,
  input_flag: Option<InputFlagV1>, output_id: Option<String> }
ManifestLegacyNoticeV1 { input_flag: InputFlagV1, schema_version: u32,
  notice: LegacyNoticeV1 }
ManifestCompatibilityOutcomeV1 { input_flag: InputFlagV1,
  against_flag: InputFlagV1, status: CompatibilityStatusV1,
  mismatch_axis: Option<CompatibilityAxisV1> }
ManifestDeterminismV1 { json_object_order: JsonObjectOrderV1,
  array_order: ArrayOrderV1, numeric_format: String = "rust_display_normalized_negative_zero_v1",
  csv: String = "rfc4180_lf_utf8_v1", path_separator: String = "/",
  clock: Option<String> = null }
```

`FinalOutputStatusV1` has only `published`; `SelectionModeV1` has `default`
and `explicit`; `JsonObjectOrderV1` has only `declaration_order`; and
`ArrayOrderV1` has only `contract_order`.  `FigureIdV1` has exactly
`mechanism_timescale`, `sensor_health_dimension_status`, `current_vs_baseline`,
`eis_nyquist`, `eis_bode`, `transient_response`, `calibration_performance`,
`signal_diagnostics`, `estimation_observed_predicted`,
`model_observed_predicted`, and `lineage`.  `TableIdV1` has exactly
`mechanism_evidence`, `health_dimensions`, `evidence_provenance`,
`artifact_lineage`, `timescale_comparison`, `model_consistency`, and
`current_vs_baseline`.  `LegacyNoticeV1` has exactly
`legacy_phase_c_not_serialized`, `legacy_mechanism_assessment_not_serialized`,
and `legacy_lineage_unknown`.  These, plus the section 18.5 enums, are every
semantic token in the manifest; there are no free-form status strings.

`ManifestInputReferenceV1` uses the same `#[serde(tag = "input_kind",
rename_all = "snake_case")]` object encoding as the summary, with the same
two literal discriminator values: `artifact` and `lineage_catalog`.
`ManifestInputReferenceKindV1` is exactly those two tokens and has the same
source authority as `PublicInputReferenceKindV1`; the two named enums remain
separate because this is a separately-versioned public document. Its artifact
object has, in declaration order, `input_kind`, `input_flag`,
`supplied_path_basename`, `artifact_kind`, `schema_version`, `lineage`,
`acquisition_families`, `availability`, and `compatibility`. Its catalog
object has, in declaration order, `input_kind`, `supplied_path_basename`,
`schema_version`, `availability`, `validation`, and `compatibility`. The
summary's nullability rules apply verbatim. The catalog `compatibility` is
always `not_applicable`, because section 18.4 forbids a catalog/artifact scope
gate. A supplied catalog has `available`/`validated`/schema `1`; an absent
catalog has `not_provided`/`not_applicable`/schema `null`; a catalog reader
failure produces no manifest. No manifest catalog object contains an artifact
ID or kind, lineage state, direct dependencies, acquisition-family state, or
aggregate scientific claim. Catalog node information is represented only by
the separate root/provenance projections.

`input_references` use the nine-value `ArtifactInputFlagV1` order followed by
the one catalog object. `formats`, `figures`, and
`tables` use the requested order after duplicate validation; all other arrays
use the section 18.5 ordering rule. `render_order` is ordinal 0 upward;
`generated_files` has the same order but omits unavailable paths, with SVG
immediately before PNG for one figure. A generated file always has
`status=written`, a non-null `relative_path`, and a non-null `format`; an
unavailable output is present only in `unavailable_outputs` and has no
generated-file record.  Summary/Markdown/manifest `output_id` is null; table
and figure `output_id` is the closed table/figure token. `source_input_flags`
are the input flags from which that file was projected, in fixed flag order.

All paths are UTF-8 relative paths using `/`, never `..`, leading `/`, a drive
prefix, or a platform separator. `null` occurs only at an `Option` field,
which is an explicit nullable field in the graph. `availability` is always
present: `available` means supplied/read and no projected warning,
`available_with_warnings` means supplied/read with at least one listed warning,
`not_provided` means no CLI input, `not_selected` means a supplied source has
no requested output, and `unavailable` means a closed reason is present in the
corresponding output/limitation record. Explicit unavailable selection raises
`RequestedOutputUnavailable`; default selection writes the unavailable record
and continues. No manifest field is inferred from a path, host, process, or
clock.

### 18.7 JSON and non-finite determinism

Typed serializers must serialize fields in declaration order.  A supplied
BTreeMap-backed catalog is projected only through the per-root membership bit
and serialized direct dependencies in section 18.10; catalog nodes are not a
public JSON collection. No output type may use a HashMap. Enum tokens are their
existing serde snake-case token unless a
literal token is specified above.  JSON strings use serde_json's standard JSON
escaping and LF pretty indentation of two spaces.  JSON scientific values are
JSON numbers, not formatted strings.  Canonical artifact readers reject NaN,
`Infinity`, `-Infinity`, and non-finite values before projection; therefore a
Phase-D JSON writer never encounters one.  Encountering one in an in-memory
projection is `StagingValidation`, not JSON `null`, a token, omission, or an
alternate numeric value.

### 18.8 Availability, format flags, and selection

`AvailabilityReason` is the closed `AvailabilityReasonV1` vocabulary in
section 18.5: `not_provided`, `not_selected`,
`legacy_phase_c_not_serialized`, `legacy_mechanism_assessment_not_serialized`,
`lineage_legacy_unknown`, `unit_authority_unavailable`, `not_comparable`,
`comparison_unknown`, `no_comparable_finite_pair`, `selected_fit_not_found`,
`selected_fit_ambiguous`, `serialized_series_invalid`,
`serialized_series_unavailable`, `paired_input_not_provided`, and
`catalog_not_supplied`. This set contains no compatibility-result token because
Phase D does not add a branch that Phase C does not have.

`--format` defaults to `all`, which means `[json, markdown]`; `json` writes
public summary and manifest but no Markdown; `markdown` writes Markdown and
manifest but no public summary.  Tables and figures are independently governed
by their selectors for every format.  `--tables` omitted selects all seven
tables in D-TBL order.  `--figures` omitted selects D-FIG-01, 02, 03, and 11,
plus each optional-source figure only when its required optional input(s) are
supplied; unavailable default selections are recorded in the manifest and do
not fail the run.  `--figures all` and every comma-list are explicit: an
unavailable selected figure/table fails with `RequestedOutputUnavailable` and
publishes no bundle.  `none` selects no item; it cannot coexist with another
ID.  Thus an absent transient input with omitted figures is not selected,
whereas `--figures transient_response` or `--figures all` fails.

### 18.9 Exact numeric, CSV, Markdown, and figure text contract

`format_public_f64` is exactly: reject non-finite; if `value == 0.0`, emit
`"0"` (normalizes both signs); otherwise emit Rust stable `f64::Display` of the
finite value with no precision argument.  This shortest-round-trip algorithm
is used unchanged for Markdown numeric cells, CSV numeric cells, figure
annotation strings, controlled tick labels, and textual axis metadata.
Integers held as `f64` therefore use the Display result (for example `1.0` is
`1`); `0.000001` and `100000000000000000000` use exactly the Display spelling
chosen by the supported Rust toolchain.  No locale, thousands separator,
percentage re-scaling, fixed precision, or renderer default numeric formatter
is permitted.  Units are adjacent text, never part of the numeric string.

`None` is `NA` in Markdown/CSV and JSON `null` only where the typed field is
`Option`.  DQI is `Data quality insufficient (DQI)`; Indeterminate is
`Indeterminate`; a missing optional artifact is `not_provided`; LegacyUnknown
is the fixed wording from section 9.  NaN and both infinities are impossible
after reading; should one reach a writer it causes `StagingValidation` rather
than a representation.  There is no percentage formatter.

CSV is UTF-8 RFC 4180 with comma delimiter, `"` quote, doubled embedded quote,
quoted fields whenever RFC 4180 requires it, LF (`\n`) record terminator, one
header record in section 7 order, no BOM, `true`/`false` booleans, serde
snake-case enum tokens, `NA` for `Option`, and `[]` for an empty collection.
Rows and numeric cells use the contracts above.  Markdown uses LF, one ASCII
pipe table, header then `---` divider, left alignment, no padding-dependent
meaning, and exactly the same cell tokens.  PNG/SVG pixel/tick placement need
not be identical, but coordinates, series order, axis units, thresholds
(none), and every controlled annotation/tick numeric string must match this
formatter.

### 18.10 Re-audited figure and table source authority

Every figure writes SVG then PNG when renderable.  `default unavailable` means
manifest record and successful bundle; `explicit unavailable` has the behavior
in section 18.8.  No figure may run a scientific module.

| figure | x / y exact authority and unit | series / order | unavailable or special rule |
|---|---|---|---|
| D-FIG-01 | x `TimescaleComparison.comparison_id`; y stored `log10_distance`, dimensionless | stored comparison order, legend evidence-level token order | no finite stored distance |
| D-FIG-02 | categorical `PhaseCHealthDimensionAssessment.dimension` / stored `status`, no numeric unit | `HealthDimension::ALL` | schema-3 legacy notice |
| D-FIG-03 | x `BaselineComparison.feature`; y current/baseline only when `comparability` is `comparable` or `comparable_with_warnings`, exactly one matching `HealthFeature{name,unit}` supplies a nonempty unit, and both serialized values are finite | `current`, then `baseline`; feature source order | `comparable_with_warnings` is rendered without conversion and emits warning code `baseline_comparable_with_warnings`, message equal to serialized `override_reason` when non-null otherwise the fixed message `Comparable with upstream context warning.`, in the manifest and Markdown caption/limitations; `not_comparable` = `not_comparable`, `unknown` = `comparison_unknown`, zero/multiple match = `unit_authority_unavailable`; no unit inference or conversion |
| D-FIG-04 | x `EisSourceData.z_real_ohm` / `EisFittedData.z_real_ohm`; y `z_imag_ohm`, all Ohm | observed, fitted | direct serialized y, `Im(Z) [Ohm]`, zero renderer negations |
| D-FIG-05 | x `source.frequency_hz` (Hz); y source-provided magnitude/phase when present, otherwise source `derived_*`; fitted magnitude/phase | observed then fitted, source array order | no frequency transform, no magnitude/phase derivation; log axis display only |
| D-FIG-06 | x raw/fitted serialized local times; y raw potential, unique candidate `predicted_v`, unique candidate `residuals_v`, V | event source order; observed, fitted, residual | unique successful model match required; 0/duplicate reason exact; no first/rank/refit/evaluate |
| D-FIG-07 | x `ValidationPredictionPoint.observed_log10_activity` only; y observed/predicted potential and serialized residual, V | validation source order | no `CalibrationObservation::log10_activity` call; paired input and aligned validation required |
| D-FIG-08 | serialized analysis timestamp/value, PSD frequency/value, Allan averaging-time/value with source units | time, PSD, Allan panels in order | panels independently unavailable; no resampling/PSD/Allan calculation |
| D-FIG-09 | `StateEstimatePoint.timestamp_s`; measured/predicted potential, V | observed then predicted, estimate source order | no variance-to-interval conversion; fewer than two finite pairs unavailable |
| D-FIG-10 | `ModelAnalysisPoint.time_s`; stored observed/predicted/unexplained residual, V | observed, predicted, residual, point source order | no residual recomputation or missing-to-zero |
| D-FIG-11 | typed root/direct dependency nodes and edges; no scientific axis | flag order then direct dependency canonical order | always roots; catalog membership only, no resolution/traversal |

All figure labels, legend order, warning, DQI, Indeterminate, and
LegacyUnknown language retain sections 8–10.  There are no threshold lines or
uncertainty transforms.  D-TBL-01 through D-TBL-03 and D-TBL-05 through
D-TBL-06 retain section 7 headers, types, source fields, order, and tokens.
D-TBL-04 has exactly one tagged-row model. Its columns are
`root_input_flag,row_kind,root_artifact_kind,root_artifact_id,lineage_state,direct_dependency_role,direct_dependency_kind,direct_dependency_id,catalog_supplied,root_catalog_entry_present`.
For every supplied certified input in fixed flag order, emit exactly one
`root` row; if its lineage is known, the root kind/ID are copied from its
identity and `root_catalog_entry_present` is `true` or `false` when a catalog
was supplied, otherwise `NA`. If its lineage is legacy unknown, root kind/ID
and entry-present are `NA`, lineage state is `legacy_unknown`, and it has no
dependency rows. For a known root, emit one `direct_dependency` row after the
root row for each serialized direct dependency, ordered by A1 role
discriminant, artifact kind, then artifact ID. A dependency row copies only
the three dependency columns; its root catalog-entry field is `NA`. The
`catalog_supplied` column is `true`/`false` in every row. Catalog nodes not
reachable as an emitted root or direct dependency never appear; there is no
catalog-node row, ancestor lookup, membership lookup for dependencies, or
traversal. D-TBL-05 uses
only `MechanismAnalysisReport.comparisons` and its referenced serialized
timescales, in lexical `comparison_id` order; it never searches a better pair.
D-TBL-07 gets its `unit` using the same unique `HealthFeature{name,unit}` rule
as D-FIG-03. `comparable` writes its literal row. `comparable_with_warnings`
writes the same literal row plus `warnings=baseline_comparable_with_warnings`
in the existing warnings field/Markdown limitation and manifest warning.
`not_comparable`, `unknown`, a non-authoritative unit, or non-finite current or
baseline writes its exact availability reason and `NA` in every numeric/unit
cell. All seven tables use section 18.9 CSV.

### 18.11 Literal fixture contract

The reviewed fixture matrix immediately below is retained only to explain the
finding; it is not normative and must not be implemented. In particular,
neither an omitted field nor a phrase such as "clone current" authorizes an
implementation to select a value, a provenance record, or an identity. The
complete normative fixture contract is section 18.11.2.

| fixture set / exact files | literal relevant content / purpose |
|---|---|
| `current/` — `mechanism.json`, `health.json`, `eis.json`, `transient.json`, `calibration.json`, `calibration_observations.json`, `signal.json`, `estimation.json`, `model.json`, `lineage_catalog.json` | all current schemas; every known identity has experiment `Single(exp-alpha)`, sensor `Specific(sensor-A)`, channel `Specific(potential-V)`, families `Known([eis_sweep,transient_step])`; mechanism `analysis_id=mech-current`, hypothesis `h-transport`, evidence level `experimentally_supported`, comparison `cmp-01` with serialized `log10_distance=0.041`; health `assessment_id=health-current`, all nine dimensions in `HealthDimension::ALL`, feature `{name:"slope_v_per_decade",value:0.058,unit:"V/decade"}`, comparison `{feature:"slope_v_per_decade",current_value:0.058,baseline_value:0.059,comparability:comparable}`; EIS frequency `[1,10]`, real `[10,5]`, imag `[-2,-1]`, fitted same; transient event 0 selects one converged `Exponential`, raw time `[0,1]`, raw V `[0.10,0.20]`, fitted time `[0,1]`, predicted `[0.11,0.19]`, residual `[-0.01,0.01]`; catalog contains the ten corresponding IDs in lexical map order. |
| `legacy/health_schema3.json`, `legacy/mechanism_schema3.json`, `legacy/unknown_lineage.json` | valid schema-3 health with `phase_c` absent; valid schema-3 mechanism with `hypothesis_assessments=[]`; each lineage is `LegacyUnknown { source_schema_version: 3, reason: FieldAbsentInLegacyArtifact }`. |
| `edge/baseline_no_unit.json`, `edge/baseline_duplicate_unit.json` | clone `current/health.json`; respectively zero matching `features.name` and two matching features with units `V/decade` and `mV/decade`; comparison remains literal. |
| `edge/transient_zero_match.json`, `edge/transient_duplicate_match.json` | clone current transient; respectively selected model has no converged candidate and has exactly two converged `Exponential` candidates with different literal predicted series `[0.11,0.19]` and `[0.12,0.18]`. |
| `edge/eis_bode.json`, `edge/eis_nyquist_sign.json` | EIS Bode adds source magnitude `[10.198...,5.099...]`, phase `[-11.309..., -11.309...]`, fitted magnitude/phase literal arrays; Nyquist uses the current negative serialized imag values to prove no sign change. |
| `edge/incompatible_sensor.json`, `edge/incompatible_experiment.json`, `edge/incompatible_optional.json` | clone the named current artifact changing only `sensor_scope=Specific(sensor-B)`, `experiment_scope=Single(exp-beta)`, or optional EIS `channel_scope=Specific(other-channel)` respectively. |
| reviewed scope/legacy examples | retracted: section 18.4 now reuses Phase-C admissibility exactly and section 18.11.2 is the sole normative fixture authority. |
| `edge/catalog_schema2.json`, `edge/catalog_bad_key.json`, `edge/catalog_duplicate_key.json`, `edge/catalog_malformed.json` | schema 2; schema 1 with key `sha256:` ID different from node identity; schema 1 raw JSON text containing the same artifact map key twice; and text `{not-json}`. |
| `edge/numeric_values.json` | valid source values `0.0`, `-0.0`, `0.000001`, `100000000000000000000.0`, `1.25`, and threshold `0.041`; expected formatted values are produced by section 18.9, not a renderer golden. |
| `edge/dqi_health.json`, `edge/indeterminate_health.json`, `edge/signal_missing.json`, `edge/model_missing.json`, `edge/large_history.json` | each is a literal clone of `current` changing only: first `data_quality` dimension to `data_quality_insufficient` with reason `required_quantity_absent`; second `observability` to `indeterminate` with `insufficient_evidence`; third `analysis_values=[0.10,null,0.20]`; fourth one model point's `observed_voltage_v` and `unexplained_residual_v` to null; fifth mechanism `hypothesis_history` to exactly 1,000 entries `history-0000` through `history-0999` and Phase-C evidence records to exactly 10,000 IDs `evidence-00000` through `evidence-09999`, each otherwise the same valid typed value. |
| `failure/write_denied/`, `failure/unmanaged_output/` | a test-only injected writer returns `io::ErrorKind::PermissionDenied` for staged `tables/mechanism_evidence.csv`; unmanaged output contains literal `keep.txt` with `do not delete`. |

The preceding matrix is superseded by section 18.11.2 and creates no
implementation permission.

#### 18.11.1 Historical, non-normative fixture capsules (superseded)

**HISTORICAL ONLY — MUST NOT BE USED FOR IMPLEMENTATION OR TEST AUTHORITY.**
This entire subsection is retained solely as review history. Its old producer
routes, mutation recipes, fixture-manifest shape, and “future” language are
superseded and may be incomplete or contradictory. It creates no implementation
permission, fixture source, mutation recipe, producer choice, or expected-output
contract. Section 18.11.2 replaces it in full; a mandatory test must never
cite an identifier, path, byte stream, or instruction from this subsection.

Phase D adds no opaque, partly-described artifact fixture. Every future input
artifact fixture is either a byte-for-byte copy of one named committed source
fixture, or is emitted once by `domain::write_artifact` from the named source
fixture plus the exact mutation record below. The test then rereads it through
`domain::read_artifact`; the bytes produced by the report renderer are never a
fixture source. The fixture manifest at
`tests/fixtures/phase_d/fixture_manifest.schema1.json` is itself a literal,
hand-authored JSON file with one record per file and exactly these fields:

```text
FixtureManifestV1 { schema_version: u32 = 1, fixtures: Vec<FixtureRecordV1> }
FixtureRecordV1 { fixture_id: String, relative_path: String,
  source_kind: FixtureSourceKindV1, source_path: Option<String>,
  source_sha256: Option<String>, artifact_kind: Option<ArtifactKindV1>,
  schema_version: Option<u32>, identity: Option<FixtureIdentityV1>,
  provenance: Option<FixtureProvenanceV1>, mutation: Option<FixtureMutationV1>,
  expected_reader: FixtureReaderV1, expected_result: FixtureResultV1 }
FixtureIdentityV1 { artifact_id: String, semantic_sha256: String,
  producer_version: String, experiment_scope: ExperimentScopeV1,
  sensor_scope: ScopeKeyV1, channel_scope: ScopeKeyV1,
  acquisition_families: AcquisitionFamilyPresentationV1 }
FixtureProvenanceV1 { software_version: String, input_path: String,
  input_sha256: String, configuration_path: Option<String>,
  configuration_sha256: Option<String>, generation_timestamp: u64,
  git_commit: Option<String> }
FixtureMutationV1 { target_path: String, before_sha256: String,
  exact_json_pointer_replacements: Vec<FixtureReplacementV1> }
FixtureReplacementV1 { pointer: String, old_json: String, new_json: String }
```

`FixtureSourceKindV1` is `committed_copy`, `canonical_mutation`, or
`literal_bytes`; `FixtureReaderV1` is `artifact`, `lineage_catalog`, or `none`;
and `FixtureResultV1` is `accept`, `artifact_error`, or
`lineage_catalog_error`.  `FixtureMutationV1` is present only for a canonical
mutation and has at least one replacement. The `old_json` and `new_json`
strings are exact RFC 8259 fragments, including object keys, array members,
and number spelling; no test helper may use `Default`, `..`, a missing field,
or a run-time-generated placeholder. A future implementation must check the
manifest hash/identity/provenance record against each constructed fixture
before its Phase-D test runs.

The committed-copy source rows are fixed now: `tests/fixtures/phase_b/e2e/eis_fit_e2e_1.json`
with SHA-256 `352dbbd578437a2260d066f7b59795a036f37be89ce1bf4edae55cd00d5e0e8c`,
`tests/fixtures/phase_b/e2e/transient_analysis_e2e_1.json` with
`cb79b7ddccacf91fb27a44f2e7a791a096e2c7b068dc0f310b426fab5792bc75`,
`tests/fixtures/phase_b/e2e/calibration_observations_e2e_2.json` with
`d743434c21a19ba77a98247c29c2f2cc9d2b1617ecd046e426895ae5a7d1ff5b`,
and `tests/fixtures/phase_b/e2e/state_estimation_e2e_2.json` with
`3d843cdae227db31ddfcbe97b05c4f035e9b72e29aef86168782f34e41d942e2`.
Their full literal payloads are the committed files at those paths; the
manifest copies their exact artifact kind, schema, complete provenance, and
known/legacy lineage value rather than restating a partial subset.

For the current health fixture, the only permitted producer route is the
existing `health assess` public route using the byte-for-byte source
`tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json`
and `tests/fixtures/phase_c/config/valid_phase_c.toml`; the manifest records
the complete emitted health artifact bytes, its Phase-C fields, its provenance,
and the `write_artifact`-validated identity before that emitted artifact is
committed as `phase_d/current/health.json`. For current mechanism,
calibration-analysis, signal, and model artifacts, the implementation must use
the corresponding existing producing public route, record every required
serialized field in the committed resulting fixture, and record the same
identity/provenance values in the manifest. It may not replace this evidence
with hand-selected defaults. This is a one-time fixture setup step; subsequent
tests consume only the committed literal files and manifest.

For every known-identity fixture, `semantic_sha256` is computed by the current
`known_lineage_from_artifact` algorithm from the fully populated artifact,
producer version, scope, acquisition families, and canonical direct
dependencies. `artifact_id` is exactly `sha256:` plus that 64-character hash.
The manifest records both literal strings and the test recomputes them before
rendering; arbitrary, all-zero, or merely shape-valid IDs are forbidden. A
legacy fixture records no identity and instead records its exact
`LegacyUnknown` source schema and reason. The manifest's recorded provenance
is complete `AnalysisProvenance`, including all nullable fields and timestamp;
it is not abbreviated to a path label.

For a byte-for-byte committed copy, `producer_version` and provenance are the
literal values already serialized in that source file. Every new known-lineage
fixture or canonical mutation uses the fixed producer version
`phase-d-fixture-v1`; it must appear verbatim both in the artifact identity and
the manifest before its hash is calculated. Thus neither the package version
nor a local Git state is a fixture-time choice.

The required mutation rows are fully constrained as follows: baseline-no-unit
and baseline-duplicate-unit replace only the `features` array and recompute the
health identity; transient-zero-match and transient-duplicate-match replace
only `events[0].candidate_fits` and recompute the transient identity;
incompatible sensor/experiment/channel replace the indicated identity scope
and recompute the modified artifact identity; different-families replaces only
the known family vector and recomputes identity; and comparable-with-warnings
replaces only the baseline comparison `comparability` token with
`"ComparableWithWarnings"` and `override_reason` with
`"temperature differs within configured tolerance"`, then recomputes the
health identity. The literal manifest must give the old and new JSON for every
one of those replacements, all other bytes remain the named committed copy,
and a pre-test SHA-256 check proves that fact.

The two catalog-error fixtures are literal bytes, not canonical artifacts:

```text
tests/fixtures/phase_d/failure/catalog_malformed.json
{not-json}\n
tests/fixtures/phase_d/failure/catalog_invalid_structure.json
{"schema_version":1,"artifacts":[]}\n
```

The former must return `LineageCatalogReadError::Json`; the latter must return
`LineageCatalogReadError::Json` because `artifacts` is an array where the
closed reader requires an object map. The existing catalog schema/key/duplicate
fixtures remain separate structural cases and their manifest rows include their
complete valid node identities and direct dependencies. A malformed catalog
never reaches staging or publication.

#### 18.11.2 Sealed Phase-D fixture ledger (normative)

This subsection is the only fixture authority. `tests/fixtures/phase_d/` is
materialized exactly from this ledger; no report, health, mechanism, model, or
other producer command is a fixture source. Each row is either an exact
byte-identical copy of an existing committed file at base
`1b04f22b0588e48e39808a870eb55b254272a88c`, or a literal UTF-8 JSON byte
stream embedded below. A `copy` is made by copying the named source bytes with
no parse/reformat/rewrite operation. A `literal-gzip-base64` is made by base64
decoding while ignoring ASCII whitespace, gzip-decompressing, verifying the
listed uncompressed SHA-256, and writing those bytes unchanged. This is an
encoding of complete file content, not a recipe, template, default, or
producer invocation. All JSON readers use the listed relative destination.

The one canonical base bundle is `phase_d_b_e2e_v1`. Every *Known* identity in
that bundle has `experiment_scope={"Single":{"experiment_id":"b-e2e-1"}}`,
`sensor_scope="Unspecified"`, and `channel_scope="Unspecified"`. Thus the
approved Phase-C three-axis gate accepts every known/known comparison in the
base. Only the model-analysis source is intentionally a serialized legacy
input; it has no invented scope, family, producer version, or identity and is
handled by the already-approved `legacy_unknown` path. Acquisition families
remain explicit provenance only: mechanism and health are `Unknown`; EIS is
`Known(["b-family-eis"])`; transient is
`Known(["b-family-transient"])`; calibration observations is
`Known(["b-family-calibration"])`; estimation is
`Known(["b-family-estimation"])`; calibration analysis and signal analysis
are `Known(["phase-d-fixture-family"])`.

| fixture ID / destination | model and exact source or complete literal SHA-256 | canonical reader outcome and immutable identity/provenance facts |
|---|---|---|
| `base.mechanism` → `base/mechanism.json` | `literal-gzip-base64`; `b24422b8e1ec3f99fcea4a9f7c7f225dfe6f77550b0365b6cda80447fd306b8b` | `MechanismAnalysisReport`, schema 4, `analysis_id="mechanism-phase-b:b-e2e-1"`, identity `sha256:a9e888019fd01dee61c98390a27bd9c6ca80eafe6b6379b77ca41f6a42a8c5b0`, semantic SHA `a9e888019fd01dee61c98390a27bd9c6ca80eafe6b6379b77ca41f6a42a8c5b0`, producer `phase-d-fixture-v1`; provenance `{software_version:"phase-b-fixture-generator",input_path:"phase-b-fixture-input",input_sha256:"0000000000000000000000000000000000000000000000000000000000000000",configuration_path:null,configuration_sha256:null,generation_timestamp:0,git_commit:null}`; four direct dependencies exactly as in the literal catalog row below. |
| `base.health` → `base/health.json` | `literal-gzip-base64`; `4265b48a0a70ff6ec89eb214a2cc8c2194cbd43bb7b7098482a7686e2eee73b3` | `SensorHealthAssessment`, schema 4, `assessment_id="health:signal:a0-test:E1"`, identity `sha256:4717ab60c11af2a14fb665ff07427530861d2eb52773b288a733b9e814562964`, semantic SHA `4717ab60c11af2a14fb665ff07427530861d2eb52773b288a733b9e814562964`, producer `phase-d-fixture-v1`; provenance `{software_version:"a0-test",input_path:"fixture-input.json",input_sha256:"a0-test",configuration_path:null,configuration_sha256:null,generation_timestamp:1,git_commit:null}`. Its complete Phase-C report has config schema `1` and SHA `946901d36fc742952c6e03f068b08c3547d1b328f031fb606cfa74093e0be8a4`. |
| `base.eis` → `base/eis.json` | `copy` of `tests/fixtures/phase_b/e2e/eis_fit_e2e_1.json`; `352dbbd578437a2260d066f7b59795a036f37be89ce1bf4edae55cd00d5e0e8c` | `EisFitReport`, schema 3, identity/semantic SHA `sha256:325483a1050eb603dd7b15c9587cfae97fa41aaf29a393a71c6082725b028e44` / `325483a1050eb603dd7b15c9587cfae97fa41aaf29a393a71c6082725b028e44`, producer `phase-b-fixture-generator`; serialized provenance is copied byte-for-byte. |
| `base.transient` → `base/transient.json` | `copy` of `tests/fixtures/phase_b/e2e/transient_analysis_e2e_1.json`; `cb79b7ddccacf91fb27a44f2e7a791a096e2c7b068dc0f310b426fab5792bc75` | `TransientAnalysisReport`, schema 3, identity/semantic SHA `sha256:d9465a5deff1224c5190dae21a674c34e9eb293f88055973491616ea2ba02b5c` / `d9465a5deff1224c5190dae21a674c34e9eb293f88055973491616ea2ba02b5c`, producer `phase-b-fixture-generator`; serialized provenance is copied byte-for-byte. |
| `base.calibration_observations` → `base/calibration_observations.json` | `copy` of `tests/fixtures/phase_b/e2e/calibration_observations_e2e_2.json`; `d743434c21a19ba77a98247c29c2f2cc9d2b1617ecd046e426895ae5a7d1ff5b` | `CalibrationObservations`, schema 3, identity/semantic SHA `sha256:927c0d3e846978f80e964fb040bfcca3e15cfffaf79bd712e223b6cf6d71c4f3` / `927c0d3e846978f80e964fb040bfcca3e15cfffaf79bd712e223b6cf6d71c4f3`, producer `phase-b-fixture-generator`; serialized provenance is copied byte-for-byte. |
| `base.estimation` → `base/estimation.json` | `copy` of `tests/fixtures/phase_b/e2e/state_estimation_e2e_2.json`; `3d843cdae227db31ddfcbe97b05c4f035e9b72e29aef86168782f34e41d942e2` | `StateEstimationReport`, schema 4, identity/semantic SHA `sha256:12b73e011b71dfe35bf5e6d88ba15ecf4767a7fc1e2c95820602e6c120dc5ddf` / `12b73e011b71dfe35bf5e6d88ba15ecf4767a7fc1e2c95820602e6c120dc5ddf`, producer `phase-b-fixture-generator`; serialized provenance is copied byte-for-byte. |
| `base.calibration` → `base/calibration.json` | exact embedded base64 literal `N-L05` below; decoded SHA-256 `7232a587edba2942aa8845fa44b4c9bee2383fda0e41e880f9bc89b5f06ce37e` | `CalibrationAnalysisReport`, schema **3**, identity/semantic SHA `sha256:f781422adad11c6adca9037fa2b8340c00b23f138d8551ca51416abf47a2a01a` / `f781422adad11c6adca9037fa2b8340c00b23f138d8551ca51416abf47a2a01a`, producer `phase-d-fixture-v1`; complete provenance is `{software_version:"phase-d-fixture-v1",input_path:"phase-d-calibration.csv",input_sha256:"phase-d-calibration-input",configuration_path:"phase-d-calibration.toml",configuration_sha256:"phase-d-calibration-config",generation_timestamp:0,git_commit:null}`. |
| `base.signal` → `base/signal.json` | exact embedded base64 literal `N-L06` below; decoded SHA-256 `e354b35e6a61f8fe6a041e07a06deec7e5f7df191190efe7cf4a418e28f5f65a` | `SignalAnalysisReport`, schema **3**, identity/semantic SHA `sha256:0c4c7f89787d26002ccbedad8c9336cd190a2a721b1e0bc0f319c4df420a4733` / `0c4c7f89787d26002ccbedad8c9336cd190a2a721b1e0bc0f319c4df420a4733`, producer `phase-d-fixture-v1`; complete provenance is `{software_version:"phase-d-fixture-v1",input_path:"phase-d-signal.csv",input_sha256:"phase-d-signal-input",configuration_path:"phase-d-signal.toml",configuration_sha256:"phase-d-signal-config",generation_timestamp:0,git_commit:null}`. The serialized series is exactly timestamps `[0,1,2,3,4,5,6,7]`, values `[0.1,0.11,null,0.13,0.14,0.15,0.16,0.17]`; it is the required missing-sample source. |
| `base.model` → `base/model.json` | `literal-gzip-base64`; `f01a0360afb6a36e1d3c3649e01f56602f7ba9e7ab105eea4d75c76fcd595b0a` | `ModelAnalysisReport`, schema 5, `artifact_kind="ism_model_analysis"`, actual `LegacyUnknown { source_schema_version: 5, reason: FieldAbsentInLegacyArtifact }`; it has three ordered points `(time_s, observed_voltage_v, predicted_voltage_v, unexplained_residual_v)=(0,0.002,0,0.002),(1,0.002,0,0.002),(2,0.002,0,0.002)` and complete `0.00000025 V^2`/`0.0005 V` uncertainty in each point. |
| `base.catalog` → `base/lineage_catalog.json` | `literal-gzip-base64`; `6b06d3a7a8b530d1acd4471d7bfc28de95e592a6726a9e72a81015c1ac0db320` | `ArtifactLineageCatalog`, schema 1, exactly six lexical map keys and nodes; it is not an artifact, has no identity, no provenance, no producer version, and no acquisition-family state of its own. |

The `base.catalog` six keys, which are also its complete node set, are exactly
`sha256:12b73e011b71dfe35bf5e6d88ba15ecf4767a7fc1e2c95820602e6c120dc5ddf`,
`sha256:325483a1050eb603dd7b15c9587cfae97fa41aaf29a393a71c6082725b028e44`,
`sha256:4717ab60c11af2a14fb665ff07427530861d2eb52773b288a733b9e814562964`,
`sha256:927c0d3e846978f80e964fb040bfcca3e15cfffaf79bd712e223b6cf6d71c4f3`,
`sha256:a9e888019fd01dee61c98390a27bd9c6ca80eafe6b6379b77ca41f6a42a8c5b0`,
and `sha256:d9465a5deff1224c5190dae21a674c34e9eb293f88055973491616ea2ba02b5c`.
Only the mechanism node has dependencies, in this exact canonical order:
calibration-observations / EIS-fit / state-estimation / transient-analysis,
each role `TransformationInput` and each artifact ID exactly as the named base
file. Every other node has `direct_dependencies=[]`. The catalog intentionally
does not add nodes for the three legacy inputs.

The complete bytes for the four `literal-gzip-base64` entries follow. They are
normative Unicode-free ASCII transport of the complete UTF-8 JSON content;
their uncompressed SHA-256 values above, not gzip metadata, are authoritative.

```text
base/mechanism.json
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
```

The following concatenated stream is retained only as a non-normative
transport check; the individually labelled streams below are authoritative:

```text
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
```

The following individually labelled streams supersede the preceding
non-normative concatenated transport dump:

```text
health_phase_d_scope.json
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
model.json
H4sICAQHhWoAA21vZGVsLmpzb24A7VhNi9swEL3vrxA+1yEbuhTaU6EUCqUU2p5KEYo0WU9XllxJ9m5Y9r93LMeKP5K0hz2UYsglmjdfb95MII9XjGXCBdwJGfgdGpW9Zhn6kpdWgebCCL336LMXLRAaVGAkEOY7faeXL0EElCyC2Q4fQu3gDTOWVQW5SaHzEmQhDEVk0hqpa4/WrDLy/hFDtgEpO4otagx7ivzYRSZ0cEKhDNbt+TBzdCREid6jueWzqsj2iSoQTpQQwOWTHExqgWWs4VAFOSQ0R+WHWSpHzLSdtMQYG7jwHuijsgPAB1dL6vs84l44Q4V2YenpKXau0YC4hWPHH+FWyP03c2fsvUnPZHAgvG1fsvcIWr3deurng+nwbw/DOyRrC7K1k8C9LKAUvAHXUk7eNxHwlAroJqxghwZDB0ncl5U1lGTEhAIvHVYHaPa1cAB5ZdGEdvKgmAOPqhaaCaNYTRNxQZB13wtj1ROCpqrHsbtaMKqvKoSHXOXxLW+us+mIRp6zNl+muYgAI+igJI6mbVJTOELshPZwADVCoyKVcGVLgs4K6pth6Jnfm1BAVIfeMzR55xOl1XEc6fFJmWmkUd24rVsyhzW2S/arJpluHdblQAStjyZhkYyl6EdA69ryUaKhVpMAuvC7tBTr1XpsurxXZ3draPdUg98hKE6SoBJQzBBEf+0vVunrqrIk3wt5nmUaw3hoNVVxvuzprsaVSbOxtHquIffG6kDLy5uO3/UmQSoHLblzzHE/W2Fy6qyGyegDlrS3E/hAtWM9kBrqskoKSgYydfv4OTL1bnydh4uZJZ9Ts+/uiJ8y1HEQNUhNEIUUjzcbgm1WN5C/GkCPR/WnkHZLUE5fC6tmQY/Iccixdok5o4RTHJyzLpG/vpnoDv6Yr5/BxVwH/aZDMbAGG4Q+2f5MM7WBh4p+cQxJor+QJ3TTK32y8wXIO3J8Jvmj5zEPBaJfLZgtxuwanduIq0GPy1FbjtrfHLXr5agtR40tR205av/RUdssR205auyfP2pX/f8tJ/4VePoNFQ0ZNQUSAAA=
catalog.json
H4sICAQHhWoAA2NhdGFsb2cuanNvbgDtV01PGzEQvfMrUM5E8vcH/6Dqse0JVauxPSYWyW66XigI8d/rJTRaKA1hN/TSHCI59qzfzPObGfv+5PR0lv0CV1DdYJtTU8/OT+lZPw1tlyL4LpeZ+zLRWy6ASXVOmdMcCaVO0xCRSxclqmCMAyrRR6GVBh09ReatNIwowlB5ykjwMoS43bBsmQLWXeruBnMD7CqFsnAw3LNXEK5SvcHooMMKc5dW0PU8DIz/YEgMFtdtE649toPl2XoBGeduHtNtd93i/BJrbKFr2uGueLvGNq1K/FX2zRqfUVDWv6T6cvly9vl3G3rcHBnO6Wxg9rAdPwzjwDo37RZt9q3Oa/QpJgxDx/wC6hqXb9qB/3GdcurZqiKs0jJhfhnE57r52VNy8TyGQk3/wd18QPjA4vtf3F9BEYuvNnroPZssiZMXMLOQWiyyCLjGumjTb2K62Hj0ZPZbkJxJYThQIgk6RXgI2lHZA2kfAa2OIChAZBa45aCpV8QwzaQjzKAQYxNhMu6uRMCUq5i6nfrnR/0fSv8pjxL+ZAlMEr7QVEOB9ZQWCKAiOqVkjEQLpiUnRtHA0EmmNXfMGNCcO4uGCqmYVaOFPxl3l/AXCMtuUUHOmHMvrwO0gLBNgRv6P2i/mF89Kn63eief4yT1WqY9CRyNUFabaAiWLaMjgrjoPXAsiRRjhKitC5oyZIw75aMqf7yIfKx6J+PuUq+HZXLtYyetGpexvXkc52Md/yd1fED/qHo+WRuTMgJKbhlDqI2B0ICoqLeGWwJMu2C98lAcgojKKV4c0NqX7hIVCAbGl5YyNiMm4+7KiBX2Akh5VUENy7ucdufCsaBPKOiTD3Jf+W69eMbuh9fbkRW3fNY2j8c++9pCnWPTbh47n+r1dTd77Vz3iOuA1//9HgAfE8YBn/PvfNB/TDzBCiVBBoyRMia8pJYEQEZBaeG5QIuOWR5LnkhpNReWKqoQmAPCnPRvxNP1nqa+ULxSzvaO6Gn0aheYHMDILnBA4t5J2/FG9HE3oi3xo+5DkzXxzvvQSf97+AU6PqV1EBUAAA==
```

No fixture construction may choose or synthesize a field omitted from these
four byte streams.

`N-L05` and `N-L06` are the remaining two base-bundle artifact literals. The
encoding is canonical base64 of the complete UTF-8 LF JSON byte stream (not a
producer instruction); decode it without whitespace folding and verify the
row checksum before invoking the corresponding certified reader.

```text
N-L05 base/calibration.json
ewogICJhbmFseXRlIjogIk5hKyIsCiAgImFydGlmYWN0X2tpbmQiOiAiY2FsaWJyYXRpb25fYW5hbHlzaXMiLAogICJjYWxpYnJhdGlvbl9pZCI6ICJOYSstY2FsaWJyYXRpb24iLAogICJjYW5kaWRhdGVfbW9kZWxzIjogWwogICAgewogICAgICAiYWN0aXZpdHlfbW9kZWwiOiAiaWRlYWwiLAogICAgICAiY29uZmlkZW5jZV9pbnRlcnZhbHMiOiBbXSwKICAgICAgImVxdWF0aW9uIjogIkUgPSBFMCArIFMgbG9nMTAoYWN0aXZpdHkpOyBhY3Rpdml0eSBtb2RlbCBpcyBjb25maWd1cmVkIHNlcGFyYXRlbHkiLAogICAgICAiZml0dGVkX3Nsb3BlX3ZfcGVyX2RlY2FkZSI6IDAuMDU5OTk1NjUzMzE5NjgzNDQsCiAgICAgICJtb2RlbF9raW5kIjogIm5lcm5zdCIsCiAgICAgICJwYXJhbWV0ZXJzIjogWwogICAgICAgIHsKICAgICAgICAgICJsb3dlcl9ib3VuZCI6IG51bGwsCiAgICAgICAgICAibmFtZSI6ICJFMCIsCiAgICAgICAgICAic291cmNlIjogbnVsbCwKICAgICAgICAgICJzdGFuZGFyZF9lcnJvciI6IDQuMzQ2NjgwMzE2MzQ1Njg4ZS02LAogICAgICAgICAgInVuaXQiOiAiViIsCiAgICAgICAgICAidXBwZXJfYm91bmQiOiBudWxsLAogICAgICAgICAgInZhbHVlIjogMC4yNzk5OTU2NTMzMTk2ODM0NgogICAgICAgIH0sCiAgICAgICAgewogICAgICAgICAgImxvd2VyX2JvdW5kIjogbnVsbCwKICAgICAgICAgICJuYW1lIjogInNsb3BlIiwKICAgICAgICAgICJzb3VyY2UiOiBudWxsLAogICAgICAgICAgInN0YW5kYXJkX2Vycm9yIjogNC4zNDY2ODAzMTYzMjE3ODFlLTYsCiAgICAgICAgICAidW5pdCI6ICJWL2RlY2FkZSIsCiAgICAgICAgICAidXBwZXJfYm91bmQiOiBudWxsLAogICAgICAgICAgInZhbHVlIjogMC4wNTk5OTU2NTMzMTk2ODM0NAogICAgICAgIH0KICAgICAgXSwKICAgICAgInByZWRpY3RlZF9wb3RlbnRpYWxfdiI6IFsKICAgICAgICAwLjEwMDAwODY5MzM2MDYzMzEyLAogICAgICAgIDAuMTYwMDA0MzQ2NjgwMzE2NTcsCiAgICAgICAgMC4yMjAwMDAwMDAwMDAwMDAwMwogICAgICBdLAogICAgICAicmVzaWR1YWxzX3YiOiBbCiAgICAgICAgLTguNjkzMzYwNjMzMTEwNzdlLTYsCiAgICAgICAgLTQuMzQ2NjgwMzE2NTY5MjYyNWUtNiwKICAgICAgICAwLjAKICAgICAgXSwKICAgICAgInNlbGVjdGl2aXR5X2NvZWZmaWNpZW50cyI6IFtdLAogICAgICAic2xvcGVfZWZmaWNpZW5jeSI6IDEuMDE0MTM2NDU3NTA1MDA5NCwKICAgICAgInN0YW5kYXJkaXplZF9yZXNpZHVhbHMiOiBbCiAgICAgICAgLTAuODk0NDI3MTkwOTk5MzQ0OCwKICAgICAgICAtMC40NDcyMTM1OTU1MDExMDAyLAogICAgICAgIDAuMAogICAgICBdLAogICAgICAic3RhdGlzdGljcyI6IHsKICAgICAgICAiYWRqdXN0ZWRfcl9zcXVhcmVkIjogbnVsbCwKICAgICAgICAiYWljIjogLTY4LjU0NDExMjAyNzY2MDEsCiAgICAgICAgImFpY2MiOiBudWxsLAogICAgICAgICJiaWMiOiAtNzAuMzQ2ODg3NDUwMzIzODksCiAgICAgICAgImNvbmRpdGlvbl9udW1iZXIiOiA4OTQ0MjcuMTkwOTUxOTYxNiwKICAgICAgICAiY29udmVyZ2VuY2VfcmVhc29uIjogInN0YWJsZSBTVkQgbGluZWFyIGxlYXN0LXNxdWFyZXMgc29sdXRpb24iLAogICAgICAgICJjb29rc19kaXN0YW5jZSI6IFsKICAgICAgICAgIDguMDAwMDAwMDAyMDczMTcyLAogICAgICAgICAgMC4wMzEyNDk5OTk5OTQzMTQwMjQsCiAgICAgICAgICAxMTIwMDk1MDI4NDY4MTguNjIKICAgICAgICBdLAogICAgICAgICJjcml0ZXJpb25fZGVsdGEiOiAwLjAsCiAgICAgICAgImR1cmJpbl93YXRzb24iOiAwLjM5OTk5OTk5OTk5OTQ4OTIsCiAgICAgICAgImZpdHRlZF9wYXJhbWV0ZXJzIjogMiwKICAgICAgICAibGV2ZXJhZ2UiOiBbCiAgICAgICAgICAwLjgwMDAwMDAwMDAyMzk0NTgsCiAgICAgICAgICAwLjE5OTk5OTk5OTk3NjI1NDIzLAogICAgICAgICAgMC45OTk5OTk5OTk5OTk3OTk5CiAgICAgICAgXSwKICAgICAgICAibWFlX3YiOiA0LjM0NjY4MDMxNjU2MDAxZS02LAogICAgICAgICJtb2RlbF93ZWlnaHQiOiAxLjAsCiAgICAgICAgIm9ic2VydmF0aW9ucyI6IDMsCiAgICAgICAgInBhcmFtZXRlcl9jb3ZhcmlhbmNlIjogWwogICAgICAgICAgWwogICAgICAgICAgICAxLjg4OTM2Mjk3NzI1MDcwNWUtMTEsCiAgICAgICAgICAgIDEuODg5MzYyOTc3MjM1NTkwMmUtMTEKICAgICAgICAgIF0sCiAgICAgICAgICBbCiAgICAgICAgICAgIDEuODg5MzYyOTc3MjM1NTkwMmUtMTEsCiAgICAgICAgICAgIDEuODg5MzYyOTc3MjI5OTIxOGUtMTEKICAgICAgICAgIF0KICAgICAgICBdLAogICAgICAgICJyX3NxdWFyZWQiOiAwLjk5OTk5OTk4Njg3OTQyMzgsCiAgICAgICAgInJtc2VfdiI6IDUuNjExNTQwMTU5MTA1Nzc5ZS02LAogICAgICAgICJyc3MiOiA5LjQ0NjgxNDg4NzE3NzA3NGUtMTEsCiAgICAgICAgIndlaWdodGVkX3JzcyI6IDkuNDQ2ODE0ODg3MTc3MDc0ZS0xMQogICAgICB9LAogICAgICAic3RhdHVzIjogImNvbnZlcmdlZCIsCiAgICAgICJ0aGVvcmV0aWNhbF9zbG9wZV92X3Blcl9kZWNhZGUiOiAwLjA1OTE1OTM0OTY4NjgxMTg0MywKICAgICAgInZhbGlkX2RvbWFpbiI6IHsKICAgICAgICAiY29uZHVjdGl2aXR5X21heF9zX3Blcl9tIjogbnVsbCwKICAgICAgICAiY29uZHVjdGl2aXR5X21pbl9zX3Blcl9tIjogbnVsbCwKICAgICAgICAibG9nMTBfYWN0aXZpdHlfbWF4IjogLTEuMCwKICAgICAgICAibG9nMTBfYWN0aXZpdHlfbWluIjogLTMuMCwKICAgICAgICAibW9sYXJfY29uY2VudHJhdGlvbl9tYXgiOiAwLjEsCiAgICAgICAgIm1vbGFyX2NvbmNlbnRyYXRpb25fbWluIjogMC4wMDEsCiAgICAgICAgInRlbXBlcmF0dXJlX21heF9rIjogMjk4LjE1LAogICAgICAgICJ0ZW1wZXJhdHVyZV9taW5fayI6IDI5OC4xNQogICAgICB9LAogICAgICAid2FybmluZ3MiOiBbCiAgICAgICAgewogICAgICAgICAgImtpbmQiOiAiaW5mbHVlbnRpYWxfb2JzZXJ2YXRpb24iLAogICAgICAgICAgIm1lc3NhZ2UiOiAiYXQgbGVhc3Qgb25lIGNhbGlicmF0aW9uIG9ic2VydmF0aW9uIGlzIGluZmx1ZW50aWFsIGJ5IENvb2sncyBkaXN0YW5jZSIsCiAgICAgICAgICAib2JzZXJ2YXRpb25faWQiOiBudWxsCiAgICAgICAgfSwKICAgICAgICB7CiAgICAgICAgICAia2luZCI6ICJib290c3RyYXBfdW5hdmFpbGFibGUiLAogICAgICAgICAgIm1lc3NhZ2UiOiAiYm9vdHN0cmFwX2l0ZXJhdGlvbnMgaXMgemVybzsgY29uZmlkZW5jZSBpbnRlcnZhbHMgYXJlIHVuYXZhaWxhYmxlIiwKICAgICAgICAgICJvYnNlcnZhdGlvbl9pZCI6IG51bGwKICAgICAgICB9CiAgICAgIF0KICAgIH0KICBdLAogICJjb25maWd1cmF0aW9uIjogewogICAgImFjdGl2aXR5IjogewogICAgICAiY29uZHVjdGl2aXR5X2VtcGlyaWNhbCI6IHsKICAgICAgICAiYjAiOiAwLjAsCiAgICAgICAgImIxIjogMC4wLAogICAgICAgICJjb25kdWN0aXZpdHlfc2VyaWVzIjogImNvbmR1Y3Rpdml0eSIsCiAgICAgICAgImVuYWJsZWQiOiBmYWxzZSwKICAgICAgICAiZml0X2IxIjogZmFsc2UsCiAgICAgICAgImZvcm0iOiAibGluZWFyX2xvZ19hY3Rpdml0eV9jb3JyZWN0aW9uIiwKICAgICAgICAibWF4aW11bV9jb25kdWN0aXZpdHlfc19wZXJfbSI6IG51bGwsCiAgICAgICAgIm1pbmltdW1fY29uZHVjdGl2aXR5X3NfcGVyX20iOiBudWxsCiAgICAgIH0sCiAgICAgICJkYXZpZXMiOiB7CiAgICAgICAgImFfY29uc3RhbnQiOiAwLjUwOSwKICAgICAgICAibWF4aW11bV9pb25pY19zdHJlbmd0aF9tb2xfbCI6IDAuNQogICAgICB9LAogICAgICAiZXh0ZW5kZWRfZGVieWVfaHVja2VsIjogewogICAgICAgICJhX2NvbnN0YW50IjogMC41MDksCiAgICAgICAgImJfY29uc3RhbnQiOiAwLjMyOCwKICAgICAgICAiaW9uX3NpemVfcGFyYW1ldGVyIjogbnVsbCwKICAgICAgICAiaW9uX3NpemVfdW5pdCI6ICJhbmdzdHJvbSIsCiAgICAgICAgIm1heGltdW1faW9uaWNfc3RyZW5ndGhfbW9sX2wiOiAwLjEKICAgICAgfSwKICAgICAgIm1vZGVsIjogImlkZWFsIiwKICAgICAgInNvbHV0aW9uX2NvbXBvc2l0aW9uIjogW10sCiAgICAgICJ1c2VyX3Byb3ZpZGVkX2FjdGl2aXR5X2ZpZWxkIjogImFjdGl2aXR5IgogICAgfSwKICAgICJhbmFseXRlIjogewogICAgICAiY2hhcmdlIjogMSwKICAgICAgIm1vbGFyX21hc3NfZ19wZXJfbW9sIjogbnVsbCwKICAgICAgIm5hbWUiOiAiTmErIgogICAgfSwKICAgICJleHBvcnQiOiB7CiAgICAgICJmZWF0dXJlc19maWxlbmFtZSI6ICJjYWxpYnJhdGlvbl9zdW1tYXJ5LmNzdiIsCiAgICAgICJtb2RlbF9maWxlbmFtZSI6ICJjYWxpYnJhdGlvbl9tb2RlbC5qc29uIiwKICAgICAgIm9ic2VydmF0aW9uc19maWxlbmFtZSI6ICJjYWxpYnJhdGlvbl9vYnNlcnZhdGlvbnMuanNvbiIsCiAgICAgICJyZXBvcnRfZmlsZW5hbWUiOiAiY2FsaWJyYXRpb25fcmVwb3J0LnR4dCIsCiAgICAgICJyZXNpZHVhbHNfZmlsZW5hbWUiOiAiY2FsaWJyYXRpb25fcmVzaWR1YWxzLmNzdiIsCiAgICAgICJyZXN1bHRzX2ZpbGVuYW1lIjogImNhbGlicmF0aW9uX3Jlc3VsdHMuanNvbiIsCiAgICAgICJ2YWxpZGF0aW9uX2ZpbGVuYW1lIjogImNhbGlicmF0aW9uX3ZhbGlkYXRpb24uY3N2IgogICAgfSwKICAgICJoeXN0ZXJlc2lzIjogewogICAgICAiYW5hbHl6ZSI6IHRydWUsCiAgICAgICJsb2dfYWN0aXZpdHlfbWF0Y2hpbmdfdG9sZXJhbmNlIjogMC4wNSwKICAgICAgIndhcm5pbmdfdGhyZXNob2xkX3YiOiAwLjAxCiAgICB9LAogICAgIm1vZGVscyI6IHsKICAgICAgImVuYWJsZWQiOiBbCiAgICAgICAgIm5lcm5zdCIKICAgICAgXQogICAgfSwKICAgICJuZXJuc3QiOiB7CiAgICAgICJwcmlvcl9zbG9wZV92X3Blcl9kZWNhZGUiOiBudWxsLAogICAgICAicHJpb3Jfc3RhbmRhcmRfZGV2aWF0aW9uX3ZfcGVyX2RlY2FkZSI6IG51bGwsCiAgICAgICJyZXNwb25zZV9zaWduIjogImF1dG8iLAogICAgICAic2xvcGVfbW9kZSI6ICJmcmVlIgogICAgfSwKICAgICJuaWNvbHNreV9laXNlbm1hbiI6IHsKICAgICAgImVuYWJsZWQiOiBmYWxzZSwKICAgICAgImZpdF9zZWxlY3Rpdml0eV9jb2VmZmljaWVudHMiOiBmYWxzZSwKICAgICAgImludGVyZmVyZW50cyI6IFtdCiAgICB9LAogICAgIm9ic2VydmF0aW9uX2V4dHJhY3Rpb24iOiB7CiAgICAgICJhbGxvd193YXJuaW5nX2ZpdHMiOiB0cnVlLAogICAgICAiZmFsbGJhY2tfc291cmNlIjogbnVsbCwKICAgICAgIm1heGltdW1fYWJzb2x1dGVfc2xvcGVfdl9wZXJfcyI6IDAuMDAwMDEsCiAgICAgICJtYXhpbXVtX21pc3NpbmdfZnJhY3Rpb24iOiAwLjIsCiAgICAgICJtaW5pbXVtX3BvaW50cyI6IDIsCiAgICAgICJwcmVmZXJyZWRfc291cmNlIjogInN0ZWFkeV9zdGF0ZV93aW5kb3dfbWVhbiIsCiAgICAgICJzdGVhZHlfc3RhdGVfZW5kX3MiOiA0LjAsCiAgICAgICJzdGVhZHlfc3RhdGVfc3RhcnRfcyI6IDAuMAogICAgfSwKICAgICJwbG90dGluZyI6IHsKICAgICAgImVuYWJsZWQiOiB0cnVlLAogICAgICAiaW5jbHVkZV9jb25maWRlbmNlX2JhbmQiOiB0cnVlLAogICAgICAiaW5jbHVkZV9oeXN0ZXJlc2lzIjogdHJ1ZSwKICAgICAgImluY2x1ZGVfcmVzaWR1YWxzIjogdHJ1ZSwKICAgICAgImluY2x1ZGVfdmFsaWRhdGlvbiI6IHRydWUKICAgIH0sCiAgICAic2NoZW1hX3ZlcnNpb24iOiAxLAogICAgInNlbGVjdGlvbiI6IHsKICAgICAgImJyYW5jaCI6ICJtaXhlZCIsCiAgICAgICJjcml0ZXJpb24iOiAiYWljYyIKICAgIH0sCiAgICAic291cmNlX3BhdGgiOiBudWxsLAogICAgInRlbXBlcmF0dXJlIjogewogICAgICAiYWxpZ25tZW50IjogImxpbmVhcl9pbnRlcnBvbGF0aW9uIiwKICAgICAgImRlZmF1bHRfY2Vsc2l1cyI6IDI1LjAsCiAgICAgICJlbnZpcm9ubWVudGFsX3NlcmllcyI6ICJ0ZW1wZXJhdHVyZSIsCiAgICAgICJtYXhpbXVtX2dhcF9zIjogMzAuMCwKICAgICAgIm1vZGUiOiAib2JzZXJ2YXRpb25fc3BlY2lmaWMiLAogICAgICAicmVmZXJlbmNlX2NlbHNpdXMiOiAyNS4wCiAgICB9LAogICAgInVuY2VydGFpbnR5IjogewogICAgICAiYm9vdHN0cmFwX2l0ZXJhdGlvbnMiOiAwLAogICAgICAiY29uZmlkZW5jZV9sZXZlbCI6IDAuOTUsCiAgICAgICJtaW5pbXVtX3N1Y2Nlc3NfZnJhY3Rpb24iOiAwLjgsCiAgICAgICJzZWVkIjogNDIKICAgIH0sCiAgICAidmFsaWRhdGlvbiI6IHsKICAgICAgImZvbGRzIjogNSwKICAgICAgIm1vZGUiOiAibGVhdmVfb25lX2NvbmNlbnRyYXRpb25fbGV2ZWxfb3V0IiwKICAgICAgInByZWRpY3Rpb25faW50ZXJ2YWxfY29uZmlkZW5jZSI6IDAuOTUsCiAgICAgICJzZWVkIjogNDIKICAgIH0sCiAgICAid2VpZ2h0aW5nIjogewogICAgICAibWluaW11bV9zdGFuZGFyZF9lcnJvcl92IjogMWUtNiwKICAgICAgIm1vZGUiOiAicG90ZW50aWFsX3N0YW5kYXJkX2Vycm9yIgogICAgfQogIH0sCiAgImh5c3RlcmVzaXMiOiB7CiAgICAiYWN0aXZpdHlfc3BlY2lmaWNfaHlzdGVyZXNpcyI6IFtdLAogICAgIm1hdGNoaW5nX3RvbGVyYW5jZV9sb2cxMF9hY3Rpdml0eSI6IDAuMDUsCiAgICAibWF4aW11bV9hYnNvbHV0ZV9oeXN0ZXJlc2lzX3YiOiBudWxsLAogICAgIm1lYW5faHlzdGVyZXNpc192IjogbnVsbCwKICAgICJtZWRpYW5faHlzdGVyZXNpc192IjogbnVsbCwKICAgICJwYWlyZWRfb2JzZXJ2YXRpb25zIjogMCwKICAgICJ3YXJuaW5ncyI6IFtdCiAgfSwKICAiaW9uX2NoYXJnZSI6IDEsCiAgImxpbmVhZ2UiOiB7CiAgICAiS25vd24iOiB7CiAgICAgICJkaXJlY3RfZGVwZW5kZW5jaWVzIjogW10sCiAgICAgICJpZGVudGl0eSI6IHsKICAgICAgICAiYWNxdWlzaXRpb25fZmFtaWxpZXMiOiB7CiAgICAgICAgICAiS25vd24iOiBbCiAgICAgICAgICAgICJwaGFzZS1kLWZpeHR1cmUtZmFtaWx5IgogICAgICAgICAgXQogICAgICAgIH0sCiAgICAgICAgImFydGlmYWN0X2lkIjogInNoYTI1NjpmNzgxNDIyYWRhZDExYzZhZGNhOTAzN2ZhMmI4MzQwYzAwYjIzZjEzOGQ4NTUxY2E1MTQxNmFiZjQ3YTJhMDFhIiwKICAgICAgICAiYXJ0aWZhY3Rfa2luZCI6ICJjYWxpYnJhdGlvbl9hbmFseXNpcyIsCiAgICAgICAgImNoYW5uZWxfc2NvcGUiOiAiVW5zcGVjaWZpZWQiLAogICAgICAgICJleHBlcmltZW50X3Njb3BlIjogewogICAgICAgICAgIlNpbmdsZSI6IHsKICAgICAgICAgICAgImV4cGVyaW1lbnRfaWQiOiAiYi1lMmUtMSIKICAgICAgICAgIH0KICAgICAgICB9LAogICAgICAgICJwcm9kdWNlcl92ZXJzaW9uIjogInBoYXNlLWQtZml4dHVyZS12MSIsCiAgICAgICAgInNjaGVtYV92ZXJzaW9uIjogMywKICAgICAgICAic2VtYW50aWNfc2hhMjU2IjogImY3ODE0MjJhZGFkMTFjNmFkY2E5MDM3ZmEyYjgzNDBjMDBiMjNmMTM4ZDg1NTFjYTUxNDE2YWJmNDdhMmEwMWEiLAogICAgICAgICJzZW5zb3Jfc2NvcGUiOiAiVW5zcGVjaWZpZWQiCiAgICAgIH0KICAgIH0KICB9LAogICJvYnNlcnZhdGlvbl9zdW1tYXJ5IjogewogICAgImFzY2VuZGluZ19vYnNlcnZhdGlvbnMiOiAyLAogICAgImNvbmNlbnRyYXRpb25fbGV2ZWxzIjogMywKICAgICJkZXNjZW5kaW5nX29ic2VydmF0aW9ucyI6IDAsCiAgICAiZXhwZXJpbWVudHMiOiAxLAogICAgImZpbml0ZV9hY3Rpdml0aWVzIjogMywKICAgICJwb3RlbnRpYWxfcmFuZ2VfdiI6IFsKICAgICAgMC4xLAogICAgICAwLjIyMDAwMDAwMDAwMDAwMDAzCiAgICBdLAogICAgInRvdGFsX29ic2VydmF0aW9ucyI6IDMsCiAgICAidW5rbm93bl9icmFuY2hfb2JzZXJ2YXRpb25zIjogMQogIH0sCiAgInByb3ZlbmFuY2UiOiB7CiAgICAiY29uZmlndXJhdGlvbl9wYXRoIjogInBoYXNlLWQtY2FsaWJyYXRpb24udG9tbCIsCiAgICAiY29uZmlndXJhdGlvbl9zaGEyNTYiOiAicGhhc2UtZC1jYWxpYnJhdGlvbi1jb25maWciLAogICAgImdlbmVyYXRpb25fdGltZXN0YW1wIjogMCwKICAgICJnaXRfY29tbWl0IjogbnVsbCwKICAgICJpbnB1dF9wYXRoIjogInBoYXNlLWQtY2FsaWJyYXRpb24uY3N2IiwKICAgICJpbnB1dF9zaGEyNTYiOiAicGhhc2UtZC1jYWxpYnJhdGlvbi1pbnB1dCIsCiAgICAic29mdHdhcmVfdmVyc2lvbiI6ICJwaGFzZS1kLWZpeHR1cmUtdjEiCiAgfSwKICAic2NoZW1hX3ZlcnNpb24iOiAzLAogICJzZWxlY3RlZF9tb2RlbCI6ICJuZXJuc3QiLAogICJzb3VyY2VfZXhwZXJpbWVudHMiOiBbCiAgICAiYi1lMmUtMSIKICBdLAogICJ2YWxpZGF0aW9uIjogewogICAgImNvbmNlbnRyYXRpb25fcmVsYXRpdmVfZXJyb3IiOiBudWxsLAogICAgImV4dHJhcG9sYXRpb25fY291bnQiOiAyLAogICAgImZhaWxlZF9wcmVkaWN0aW9ucyI6IDAsCiAgICAiZm9sZHMiOiBbCiAgICAgIHsKICAgICAgICAiY292ZXJhZ2UiOiBudWxsLAogICAgICAgICJleHRyYXBvbGF0aW9uX2NvdW50IjogMSwKICAgICAgICAiZmFpbGVkX3ByZWRpY3Rpb25zIjogMCwKICAgICAgICAiZm9sZF9pZCI6ICJmb2xkLTAiLAogICAgICAgICJoZWxkX291dF9vYnNlcnZhdGlvbnMiOiAxLAogICAgICAgICJtYWVfbG9nMTBfYWN0aXZpdHkiOiAwLjAwMDU0NTgwNDY1ODgzNzEzNDYsCiAgICAgICAgIm1hZV9wb3RlbnRpYWxfdiI6IDAuMDAwMDMyNzU3MjE5MDUxNjMyMzE0LAogICAgICAgICJwcmVkaWN0aW9uX2JpYXNfdiI6IC0wLjAwMDAzMjc1NzIxOTA1MTYzMjMxNCwKICAgICAgICAicm1zZV9sb2cxMF9hY3Rpdml0eSI6IDAuMDAwNTQ1ODA0NjU4ODM3MTM0NiwKICAgICAgICAicm1zZV9wb3RlbnRpYWxfdiI6IDAuMDAwMDMyNzU3MjE5MDUxNjMyMzE0CiAgICAgIH0sCiAgICAgIHsKICAgICAgICAiY292ZXJhZ2UiOiBudWxsLAogICAgICAgICJleHRyYXBvbGF0aW9uX2NvdW50IjogMCwKICAgICAgICAiZmFpbGVkX3ByZWRpY3Rpb25zIjogMCwKICAgICAgICAiZm9sZF9pZCI6ICJmb2xkLTEiLAogICAgICAgICJoZWxkX291dF9vYnNlcnZhdGlvbnMiOiAxLAogICAgICAgICJtYWVfbG9nMTBfYWN0aXZpdHkiOiAwLjAwMDAxNDg3MzczMjY1NDQwNzk0NSwKICAgICAgICAibWFlX3BvdGVudGlhbF92IjogOC45MjQxMDY4NTc5MzI2NTdlLTcsCiAgICAgICAgInByZWRpY3Rpb25fYmlhc192IjogOC45MjQxMDY4NTc5MzI2NTdlLTcsCiAgICAgICAgInJtc2VfbG9nMTBfYWN0aXZpdHkiOiAwLjAwMDAxNDg3MzczMjY1NDQwNzk0NSwKICAgICAgICAicm1zZV9wb3RlbnRpYWxfdiI6IDguOTI0MTA2ODU3OTMyNjU3ZS03CiAgICAgIH0sCiAgICAgIHsKICAgICAgICAiY292ZXJhZ2UiOiBudWxsLAogICAgICAgICJleHRyYXBvbGF0aW9uX2NvdW50IjogMSwKICAgICAgICAiZmFpbGVkX3ByZWRpY3Rpb25zIjogMCwKICAgICAgICAiZm9sZF9pZCI6ICJmb2xkLTIiLAogICAgICAgICJoZWxkX291dF9vYnNlcnZhdGlvbnMiOiAxLAogICAgICAgICJtYWVfbG9nMTBfYWN0aXZpdHkiOiAxLjExMDIyMzAyNDYyNTE1NjVlLTE2LAogICAgICAgICJtYWVfcG90ZW50aWFsX3YiOiAwLjAsCiAgICAgICAgInByZWRpY3Rpb25fYmlhc192IjogMC4wLAogICAgICAgICJybXNlX2xvZzEwX2FjdGl2aXR5IjogMS4xMTAyMjMwMjQ2MjUxNTY1ZS0xNiwKICAgICAgICAicm1zZV9wb3RlbnRpYWxfdiI6IDAuMAogICAgICB9CiAgICBdLAogICAgImludGVydmFsX2NvdmVyYWdlIjogbnVsbCwKICAgICJtYWVfbG9nMTBfYWN0aXZpdHkiOiAwLjAwMDE4Njg5Mjc5NzE2Mzg4NDUyLAogICAgIm1hZV9wb3RlbnRpYWxfdiI6IDAuMDAwMDExMjE2NTQzMjQ1ODA4NTI3LAogICAgIm1vZGUiOiAibGVhdmVfb25lX2NvbmNlbnRyYXRpb25fbGV2ZWxfb3V0IiwKICAgICJwcmVkaWN0aW9uX2JpYXNfdiI6IC0wLjAwMDAxMDYyMTYwMjc4ODYxMzAxNiwKICAgICJwcmVkaWN0aW9ucyI6IFsKICAgICAgewogICAgICAgICJleHRyYXBvbGF0ZWQiOiB0cnVlLAogICAgICAgICJvYnNlcnZhdGlvbl9pZCI6ICJiLWUyZS0xLWV2ZW50LTAiLAogICAgICAgICJvYnNlcnZlZF9sb2cxMF9hY3Rpdml0eSI6IC0zLjAsCiAgICAgICAgIm9ic2VydmVkX3BvdGVudGlhbF92IjogMC4xLAogICAgICAgICJwcmVkaWN0ZWRfbG9nMTBfYWN0aXZpdHkiOiAtMi45OTk0NTQxOTUzNDExNjMsCiAgICAgICAgInByZWRpY3RlZF9wb3RlbnRpYWxfdiI6IDAuMDk5OTY3MjQyNzgwOTQ4MzcKICAgICAgfSwKICAgICAgewogICAgICAgICJleHRyYXBvbGF0ZWQiOiBmYWxzZSwKICAgICAgICAib2JzZXJ2YXRpb25faWQiOiAiYi1lMmUtMS1ldmVudC0xIiwKICAgICAgICAib2JzZXJ2ZWRfbG9nMTBfYWN0aXZpdHkiOiAtMi4wLAogICAgICAgICJvYnNlcnZlZF9wb3RlbnRpYWxfdiI6IDAuMTYsCiAgICAgICAgInByZWRpY3RlZF9sb2cxMF9hY3Rpdml0eSI6IC0yLjAwMDAxNDg3MzczMjY1NDQsCiAgICAgICAgInByZWRpY3RlZF9wb3RlbnRpYWxfdiI6IDAuMTYwMDAwODkyNDEwNjg1OAogICAgICB9LAogICAgICB7CiAgICAgICAgImV4dHJhcG9sYXRlZCI6IHRydWUsCiAgICAgICAgIm9ic2VydmF0aW9uX2lkIjogImItZTJlLTEtZXZlbnQtMiIsCiAgICAgICAgIm9ic2VydmVkX2xvZzEwX2FjdGl2aXR5IjogLTEuMCwKICAgICAgICAib2JzZXJ2ZWRfcG90ZW50aWFsX3YiOiAwLjIyMDAwMDAwMDAwMDAwMDAzLAogICAgICAgICJwcmVkaWN0ZWRfbG9nMTBfYWN0aXZpdHkiOiAtMC45OTk5OTk5OTk5OTk5OTk5LAogICAgICAgICJwcmVkaWN0ZWRfcG90ZW50aWFsX3YiOiAwLjIyMDAwMDAwMDAwMDAwMDAzCiAgICAgIH0KICAgIF0sCiAgICAicm1zZV9sb2cxMF9hY3Rpdml0eSI6IDAuMDAwMzE1MjM3NDUyMDUzNDE5NCwKICAgICJybXNlX3BvdGVudGlhbF92IjogMC4wMDAwMTg5MTk0MDYyMzQ3Njk4MTgsCiAgICAid2FybmluZ3MiOiBbXQogIH0sCiAgIndhcm5pbmdzIjogWwogICAgewogICAgICAia2luZCI6ICJtaXNzaW5nX3RlbXBlcmF0dXJlIiwKICAgICAgIm1lc3NhZ2UiOiAidXNpbmcgY29uZmlndXJlZCBkZWZhdWx0IHRlbXBlcmF0dXJlOyBubyBhbGlnbmVkIHRlbXBlcmF0dXJlIG9ic2VydmF0aW9uIHdhcyBhdmFpbGFibGUiLAogICAgICAib2JzZXJ2YXRpb25faWQiOiAiZXZlbnQtMCIKICAgIH0sCiAgICB7CiAgICAgICJraW5kIjogIm1pc3NpbmdfdGVtcGVyYXR1cmUiLAogICAgICAibWVzc2FnZSI6ICJ1c2luZyBjb25maWd1cmVkIGRlZmF1bHQgdGVtcGVyYXR1cmU7IG5vIGFsaWduZWQgdGVtcGVyYXR1cmUgb2JzZXJ2YXRpb24gd2FzIGF2YWlsYWJsZSIsCiAgICAgICJvYnNlcnZhdGlvbl9pZCI6ICJldmVudC0xIgogICAgfSwKICAgIHsKICAgICAgImtpbmQiOiAibWlzc2luZ190ZW1wZXJhdHVyZSIsCiAgICAgICJtZXNzYWdlIjogInVzaW5nIGNvbmZpZ3VyZWQgZGVmYXVsdCB0ZW1wZXJhdHVyZTsgbm8gYWxpZ25lZCB0ZW1wZXJhdHVyZSBvYnNlcnZhdGlvbiB3YXMgYXZhaWxhYmxlIiwKICAgICAgIm9ic2VydmF0aW9uX2lkIjogImV2ZW50LTIiCiAgICB9CiAgXQp9Cg==
N-L06 base/signal.json
ewogICJhbGxhbiI6IG51bGwsCiAgImFuYWx5c2lzX2lkIjogInNpZ25hbDpwaGFzZS1kLXNpZ25hbC1pbnB1dDpFMSIsCiAgImFuYWx5c2lzX3RpbWVzdGFtcHMiOiBbCiAgICAwLjAsCiAgICAxLjAsCiAgICAyLjAsCiAgICAzLjAsCiAgICA0LjAsCiAgICA1LjAsCiAgICA2LjAsCiAgICA3LjAKICBdLAogICJhbmFseXNpc192YWx1ZXMiOiBbCiAgICAwLjEsCiAgICAwLjExLAogICAgbnVsbCwKICAgIDAuMTMsCiAgICAwLjE0LAogICAgMC4xNSwKICAgIDAuMTYsCiAgICAwLjE3CiAgXSwKICAiYXJ0aWZhY3Rfa2luZCI6ICJzaWduYWxfYW5hbHlzaXMiLAogICJjaGFubmVsIjogIkUxIiwKICAiY29uZmlndXJhdGlvbiI6IHsKICAgICJhbGxhbiI6IHsKICAgICAgImVuYWJsZWQiOiBmYWxzZSwKICAgICAgIm1pbmltdW1fY2x1c3RlcnMiOiA4LAogICAgICAidGF1X3BvaW50cyI6IDMwCiAgICB9LAogICAgImNvcnJlbGF0aW9uIjogewogICAgICAiZW5hYmxlZCI6IGZhbHNlLAogICAgICAibGFnX3N0ZXBfcyI6IG51bGwsCiAgICAgICJtYXhpbXVtX2xhZ19zIjogNjAuMCwKICAgICAgIm1pbmltdW1fb2JzZXJ2YXRpb25zIjogMwogICAgfSwKICAgICJkcmlmdCI6IHsKICAgICAgIm1pbmltdW1fZHVyYXRpb25fcyI6IDMwMC4wLAogICAgICAibW9kZWxzIjogWwogICAgICAgICJvcmRpbmFyeV9saW5lYXIiLAogICAgICAgICJ0aGVpbF9zZW4iCiAgICAgIF0KICAgIH0sCiAgICAiZXhwb3J0IjogewogICAgICAiYWxsYW5fZmlsZW5hbWUiOiAic2lnbmFsX2FsbGFuLmNzdiIsCiAgICAgICJjb3JyZWxhdGlvbnNfZmlsZW5hbWUiOiAic2lnbmFsX2NvcnJlbGF0aW9ucy5jc3YiLAogICAgICAiZHJpZnRfZmlsZW5hbWUiOiAic2lnbmFsX2RyaWZ0LmNzdiIsCiAgICAgICJwc2RfZmlsZW5hbWUiOiAic2lnbmFsX3BzZC5jc3YiLAogICAgICAicmVwb3J0X2ZpbGVuYW1lIjogInNpZ25hbF9yZXBvcnQudHh0IiwKICAgICAgInJlc3VsdHNfZmlsZW5hbWUiOiAic2lnbmFsX3Jlc3VsdHMuanNvbiIsCiAgICAgICJzcGlrZXNfZmlsZW5hbWUiOiAic2lnbmFsX3NwaWtlcy5jc3YiLAogICAgICAic3VtbWFyeV9maWxlbmFtZSI6ICJzaWduYWxfc3VtbWFyeS5jc3YiCiAgICB9LAogICAgInBsb3R0aW5nIjogewogICAgICAiZW5hYmxlZCI6IHRydWUKICAgIH0sCiAgICAicHNkIjogewogICAgICAiZGV0cmVuZCI6ICJsaW5lYXIiLAogICAgICAiZW5hYmxlZCI6IGZhbHNlLAogICAgICAiZmZ0X2xlbmd0aCI6IG51bGwsCiAgICAgICJmcmVxdWVuY3lfYmFuZHMiOiBbXSwKICAgICAgIm1heGltdW1fZnJlcXVlbmN5X2h6IjogbnVsbCwKICAgICAgIm1pbmltdW1fZnJlcXVlbmN5X2h6IjogbnVsbCwKICAgICAgIm92ZXJsYXBfZnJhY3Rpb24iOiAwLjUsCiAgICAgICJwYXJzZXZhbF90b2xlcmFuY2UiOiAwLjEsCiAgICAgICJzZWdtZW50X2R1cmF0aW9uX3MiOiBudWxsLAogICAgICAic2VnbWVudF9wb2ludHMiOiAyNTYsCiAgICAgICJ3aW5kb3ciOiAiaGFubiIKICAgIH0sCiAgICAic2FtcGxpbmciOiB7CiAgICAgICJkdXBsaWNhdGVfdGltZXN0YW1wX3BvbGljeSI6ICJlcnJvciIsCiAgICAgICJtYXhpbXVtX2ludGVycG9sYXRpb25fZ2FwX3MiOiA1LjAsCiAgICAgICJub25fbW9ub3RvbmljX3RpbWVzdGFtcF9wb2xpY3kiOiAiZXJyb3IiLAogICAgICAicG9saWN5IjogInJlcXVpcmVfcmVndWxhciIsCiAgICAgICJyZWd1bGFyaXR5X3JlbGF0aXZlX3RvbGVyYW5jZSI6IDAuMDEsCiAgICAgICJyZXNhbXBsZV9pbnRlcnZhbF9zIjogbnVsbAogICAgfSwKICAgICJzY2hlbWFfdmVyc2lvbiI6IDEsCiAgICAic3Bpa2VzIjogewogICAgICAiZW5hYmxlZCI6IHRydWUsCiAgICAgICJtYWRfdGhyZXNob2xkIjogNC4wLAogICAgICAibWF4aW11bV9mbGFnZ2VkX2ZyYWN0aW9uIjogMC4yNSwKICAgICAgIm1ldGhvZCI6ICJoYW1wZWwiLAogICAgICAibWluaW11bV9sb2NhbF9vYnNlcnZhdGlvbnMiOiA1LAogICAgICAid2luZG93X2R1cmF0aW9uX3MiOiBudWxsLAogICAgICAid2luZG93X3BvaW50cyI6IDExCiAgICB9LAogICAgInN0YXRpc3RpY3MiOiB7CiAgICAgICJjb25maWRlbmNlX2xldmVsIjogMC45NSwKICAgICAgInF1YW50aWxlcyI6IFsKICAgICAgICAwLjAxLAogICAgICAgIDAuMDUsCiAgICAgICAgMC4yNSwKICAgICAgICAwLjUsCiAgICAgICAgMC43NSwKICAgICAgICAwLjk1LAogICAgICAgIDAuOTkKICAgICAgXQogICAgfSwKICAgICJ3aW5kb3dpbmciOiB7CiAgICAgICJlbGlnaWJsZV9ldmVudF9raW5kcyI6IFsKICAgICAgICAiY29uY2VudHJhdGlvbl9zdGVwIiwKICAgICAgICAiZmxvd19jaGFuZ2UiLAogICAgICAgICJ0ZW1wZXJhdHVyZV9jaGFuZ2UiLAogICAgICAgICJpbnRlcmZlcmVudF9hZGRpdGlvbiIKICAgICAgXSwKICAgICAgImVuZF9zIjogbnVsbCwKICAgICAgImV4Y2x1ZGVfYWZ0ZXJfZXZlbnRfcyI6IDAuMCwKICAgICAgImV4Y2x1ZGVfYmVmb3JlX2V2ZW50X3MiOiAwLjAsCiAgICAgICJyZWxhdGl2ZV9lbmRfcyI6IG51bGwsCiAgICAgICJyZWxhdGl2ZV9zdGFydF9zIjogbnVsbCwKICAgICAgInNvdXJjZSI6ICJlbnRpcmVfbWVhc3VyZW1lbnQiLAogICAgICAic3RhcnRfcyI6IG51bGwKICAgIH0KICB9LAogICJjb3JyZWxhdGlvbnMiOiBbXSwKICAiZGVzY3JpcHRpdmUiOiB7CiAgICAiY29uZmlkZW5jZV9pbnRlcnZhbCI6IFsKICAgICAgMC4xMTgxNTIzMzMxOTAxMDY0LAogICAgICAwLjE1NjEzMzM4MTA5NTYwNzkKICAgIF0sCiAgICAiY291bnQiOiA3LAogICAgImV4Y2Vzc19rdXJ0b3NpcyI6IC0xLjcwNTAyNjgxMjIzNzE4MjMsCiAgICAiaW50ZXJxdWFydGlsZV9yYW5nZSI6IDAuMDM1LAogICAgIm1heGltdW0iOiAwLjE3LAogICAgIm1lYW4iOiAwLjEzNzE0Mjg1NzE0Mjg1NzE1LAogICAgIm1lZGlhbiI6IDAuMTQsCiAgICAibWVkaWFuX2Fic29sdXRlX2RldmlhdGlvbiI6IDAuMDE5OTk5OTk5OTk5OTk5OTksCiAgICAibWluaW11bSI6IDAuMSwKICAgICJwZWFrX3RvX3BlYWsiOiAwLjA3LAogICAgInF1YW50aWxlcyI6IFsKICAgICAgewogICAgICAgICJwcm9iYWJpbGl0eSI6IDAuMDEsCiAgICAgICAgInZhbHVlIjogMC4xMDA2MDAwMDAwMDAwMDAwMQogICAgICB9LAogICAgICB7CiAgICAgICAgInByb2JhYmlsaXR5IjogMC4wNSwKICAgICAgICAidmFsdWUiOiAwLjEwMzAwMDAwMDAwMDAwMDAxCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAicHJvYmFiaWxpdHkiOiAwLjI1LAogICAgICAgICJ2YWx1ZSI6IDAuMTIKICAgICAgfSwKICAgICAgewogICAgICAgICJwcm9iYWJpbGl0eSI6IDAuNSwKICAgICAgICAidmFsdWUiOiAwLjE0CiAgICAgIH0sCiAgICAgIHsKICAgICAgICAicHJvYmFiaWxpdHkiOiAwLjc1LAogICAgICAgICJ2YWx1ZSI6IDAuMTU1CiAgICAgIH0sCiAgICAgIHsKICAgICAgICAicHJvYmFiaWxpdHkiOiAwLjk1LAogICAgICAgICJ2YWx1ZSI6IDAuMTY3CiAgICAgIH0sCiAgICAgIHsKICAgICAgICAicHJvYmFiaWxpdHkiOiAwLjk5LAogICAgICAgICJ2YWx1ZSI6IDAuMTY5NAogICAgICB9CiAgICBdLAogICAgInJtcyI6IDAuMTM5MTgxMjc5NTI5MzU1NCwKICAgICJyb2J1c3Rfc3RhbmRhcmRfZGV2aWF0aW9uIjogMC4wMjk2NTE5OTk5OTk5OTk5ODQsCiAgICAic2FtcGxlX3ZhcmlhbmNlIjogMC4wMDA2NTcxNDI4NTcxNDI4NTczLAogICAgInNrZXduZXNzIjogLTAuMTg2OTEzMTg5MzEwOTg3ODgsCiAgICAic3RhbmRhcmRfZGV2aWF0aW9uIjogMC4wMjU2MzQ3OTc3Nzg0NjYyMzMKICB9LAogICJkcmlmdCI6IFtdLAogICJleHBlcmltZW50X2lkIjogbnVsbCwKICAibGluZWFnZSI6IHsKICAgICJLbm93biI6IHsKICAgICAgImRpcmVjdF9kZXBlbmRlbmNpZXMiOiBbXSwKICAgICAgImlkZW50aXR5IjogewogICAgICAgICJhY3F1aXNpdGlvbl9mYW1pbGllcyI6IHsKICAgICAgICAgICJLbm93biI6IFsKICAgICAgICAgICAgInBoYXNlLWQtZml4dHVyZS1mYW1pbHkiCiAgICAgICAgICBdCiAgICAgICAgfSwKICAgICAgICAiYXJ0aWZhY3RfaWQiOiAic2hhMjU2OjBjNGM3Zjg5Nzg3ZDI2MDAyY2NiZWRhZDhjOTMzNmNkMTkwYTJhNzIxYjFlMGJjMGYzMTljNGRmNDIwYTQ3MzMiLAogICAgICAgICJhcnRpZmFjdF9raW5kIjogInNpZ25hbF9hbmFseXNpcyIsCiAgICAgICAgImNoYW5uZWxfc2NvcGUiOiAiVW5zcGVjaWZpZWQiLAogICAgICAgICJleHBlcmltZW50X3Njb3BlIjogewogICAgICAgICAgIlNpbmdsZSI6IHsKICAgICAgICAgICAgImV4cGVyaW1lbnRfaWQiOiAiYi1lMmUtMSIKICAgICAgICAgIH0KICAgICAgICB9LAogICAgICAgICJwcm9kdWNlcl92ZXJzaW9uIjogInBoYXNlLWQtZml4dHVyZS12MSIsCiAgICAgICAgInNjaGVtYV92ZXJzaW9uIjogMywKICAgICAgICAic2VtYW50aWNfc2hhMjU2IjogIjBjNGM3Zjg5Nzg3ZDI2MDAyY2NiZWRhZDhjOTMzNmNkMTkwYTJhNzIxYjFlMGJjMGYzMTljNGRmNDIwYTQ3MzMiLAogICAgICAgICJzZW5zb3Jfc2NvcGUiOiAiVW5zcGVjaWZpZWQiCiAgICAgIH0KICAgIH0KICB9LAogICJwcm92ZW5hbmNlIjogewogICAgImNvbmZpZ3VyYXRpb25fcGF0aCI6ICJwaGFzZS1kLXNpZ25hbC50b21sIiwKICAgICJjb25maWd1cmF0aW9uX3NoYTI1NiI6ICJwaGFzZS1kLXNpZ25hbC1jb25maWciLAogICAgImdlbmVyYXRpb25fdGltZXN0YW1wIjogMCwKICAgICJnaXRfY29tbWl0IjogbnVsbCwKICAgICJpbnB1dF9wYXRoIjogInBoYXNlLWQtc2lnbmFsLmNzdiIsCiAgICAiaW5wdXRfc2hhMjU2IjogInBoYXNlLWQtc2lnbmFsLWlucHV0IiwKICAgICJzb2Z0d2FyZV92ZXJzaW9uIjogInBoYXNlLWQtZml4dHVyZS12MSIKICB9LAogICJwc2QiOiBudWxsLAogICJyZXNpZHVhbF9hbmFseXNpcyI6IFtdLAogICJzYW1wbGluZyI6IHsKICAgICJkdXBsaWNhdGVfdGltZXN0YW1wcyI6IDAsCiAgICAiZHVyYXRpb25fcyI6IDcuMCwKICAgICJlZmZlY3RpdmVfZnJlcXVlbmN5X2h6IjogMS4wLAogICAgImVuZF90aW1lX3MiOiA3LjAsCiAgICAiZmluaXRlX3NhbXBsZV9jb3VudCI6IDcsCiAgICAiaW50ZXJwb2xhdGVkX2luZGljZXMiOiBbXSwKICAgICJpbnRlcnBvbGF0aW9uX2NvdW50IjogMCwKICAgICJpbnRlcnBvbGF0aW9uX2dhcF9leGNlZWRlZCI6IGZhbHNlLAogICAgImludGVydmFsX2N2IjogMC4wLAogICAgImludGVydmFsX3N0ZGRldl9zIjogMC4wLAogICAgImlzX3JlZ3VsYXIiOiB0cnVlLAogICAgIm1heGltdW1faW50ZXJ2YWxfcyI6IDEuMCwKICAgICJtZWFuX2ludGVydmFsX3MiOiAxLjAsCiAgICAibWVkaWFuX2ludGVydmFsX3MiOiAxLjAsCiAgICAibWluaW11bV9pbnRlcnZhbF9zIjogMS4wLAogICAgIm1pc3NpbmdfZnJhY3Rpb24iOiAwLjEyNSwKICAgICJub25fbW9ub3RvbmljX3RpbWVzdGFtcHMiOiAwLAogICAgIm91dHB1dF9taXNzaW5nX2luZGljZXMiOiBbXSwKICAgICJyZXNvbHZlZF9kdXBsaWNhdGVfZ3JvdXBzIjogMCwKICAgICJzYW1wbGVfY291bnQiOiA4LAogICAgInNvcnRlZF9yb3dzIjogMCwKICAgICJzdGFydF90aW1lX3MiOiAwLjAsCiAgICAidGFyZ2V0X2ludGVydmFsX3MiOiBudWxsLAogICAgInRyYW5zZm9ybWF0aW9ucyI6IFtdCiAgfSwKICAic2NoZW1hX3ZlcnNpb24iOiAzLAogICJzZW5zb3JfaWQiOiBudWxsLAogICJzcGlrZXMiOiB7CiAgICAiZmxhZ2dlZCI6IFtdLAogICAgImZsYWdnZWRfZnJhY3Rpb24iOiAwLjAsCiAgICAibWF4aW11bV9mbGFnZ2VkX2ZyYWN0aW9uIjogMC4yNSwKICAgICJtZXRob2QiOiAiaGFtcGVsIgogIH0sCiAgInVuaXQiOiAiViIsCiAgIndhcm5pbmdzIjogWwogICAgInJlY29yZF90b29fc2hvcnQiLAogICAgImRyaWZ0X2R1cmF0aW9uX2luc3VmZmljaWVudCIsCiAgICAiZHJpZnRfZHVyYXRpb25faW5zdWZmaWNpZW50IgogIF0sCiAgIndpbmRvdyI6IHsKICAgICJkZXRyZW5kaW5nX21ldGhvZCI6ICJMaW5lYXIiLAogICAgImVuZF9zIjogNy4wLAogICAgImV4Y2x1ZGVkX2ludGVydmFscyI6IFtdLAogICAgImV4Y2x1ZGVkX29ic2VydmF0aW9ucyI6IDAsCiAgICAibWlzc2luZ19vYnNlcnZhdGlvbnMiOiAxLAogICAgInJlc2FtcGxpbmdfbWV0aG9kIjogbnVsbCwKICAgICJzZWxlY3RlZF9vYnNlcnZhdGlvbl9jb3VudCI6IDgsCiAgICAic291cmNlIjogImVudGlyZV9tZWFzdXJlbWVudCIsCiAgICAic291cmNlX29ic2VydmF0aW9uX2NvdW50IjogOCwKICAgICJzb3VyY2VfdGltZXN0YW1wcyI6IFsKICAgICAgMC4wLAogICAgICAxLjAsCiAgICAgIDIuMCwKICAgICAgMy4wLAogICAgICA0LjAsCiAgICAgIDUuMCwKICAgICAgNi4wLAogICAgICA3LjAKICAgIF0sCiAgICAic3RhcnRfcyI6IDAuMAogIH0KfQo=
```

##### 18.11.2.a Complete Phase-D normative fixture authority

The base ledger above and this completion are one single **PHASE D NORMATIVE
FIXTURE LEDGER**. They are the sole authority for every fixture, bundle, and
expected-output contract used by a mandatory Phase-D test. Every fixture ID in
section 18.12 resolves exactly once to an `N-F*` or `N-X*` record below or to
its declared `N-F01`–`N-F10` base alias. Historical prose, an execution path,
temporary output, an uncreated output manifest, a test-helper default, or an
implementation-selected repository file is never fixture authority.

Exactly three classes exist: **1 exact committed copy** (ID, Phase-D
destination, source path/base, source/final SHA-256, and byte-for-byte=yes);
**2 exact embedded literal** (ID, destination, encoding, complete bytes,
decoded SHA-256, logical type, reader result); and **3 exact derived fixture**
(ID, destination, base ID, ordered JSON-pointer old/new operations, complete
identity/provenance consequence, and final SHA-256). There is no fourth class.
For Class 3, materialize the base, apply every listed operation in order,
verify the final SHA, and only then call its reader. Running a producer,
deferring capture until generation, and any mutation at test time are prohibited.

The certified Phase-D input policy is intentionally narrower than the generic
`VersionedArtifact` migration reader: mechanism 4/current plus 1–3 legacy;
health 4/current plus 3 legacy; catalog exactly 1; EIS, transient,
calibration, calibration-observations, and signal exactly 3; estimation
exactly 4; model exactly 5. The Phase-D boundary checks that policy before its
`read_artifact` projection; catalogs use only `read_artifact_lineage_catalog`.
Accordingly the schema-3 `N-F07` calibration and `N-F08` signal literals
replace the prior invalid schema-1 base entries.

| certified input | current schema / accepted legacy schemas | base fixture schema | result |
|---|---|---:|---|
| mechanism | 4 / 1–3 | 4 | accepted |
| health | 4 / 3 | 4 | accepted |
| lineage catalog | 1 / none | 1 | accepted by catalog reader only |
| EIS | 3 / none | 3 | accepted |
| transient | 3 / none | 3 | accepted |
| calibration | 3 / none | 3 | accepted |
| calibration observations | 3 / none | 3 | accepted |
| signal | 3 / none | 3 | accepted |
| estimation | 4 / none | 4 | accepted |
| model | 5 / none | 5 | accepted as its exact legacy-lineage source |

Thus base-fixture/reader mismatches are zero. The generic repository reader's
additional migration capability is not a Phase-D admission rule.

##### 18.11.2.b Complete file ledger

The base-table aliases are fixed: `N-F01=base.mechanism`, `N-F02=base.health`,
`N-F03=base.eis`, `N-F04=base.transient`,
`N-F05=base.calibration_observations`, `N-F06=base.estimation`,
`N-F07=base.calibration` (`N-L05`), `N-F08=base.signal` (`N-L06`),
`N-F09=base.model`, and `N-F10=base.catalog`. They are not additional files.

| ID → destination | Class and exact source/operation | exact reader result, identity/state, final SHA-256 |
|---|---|---|
| `N-F11` → `legacy/health_v3.json` | 1; copy `tests/fixtures/phase_c/writer_boundary/legacy_health_assessment_v3.json` at base; source/final `47aecec55b6a35d352ec349c8d6c7c35485a4b86b063a6be33920887c550cb7c`; byte-for-byte yes | health reader accepts schema 3, exact `LegacyUnknown`, no Phase-C dimensions |
| `N-F12` → `legacy/mechanism_v1.json` | 1; copy `tests/fixtures/a0_artifact_contracts/schema1/mechanism_analysis.schema1.json`; source/final `1f306be35576f813347ad4906ead8296bf6d7a391547b2dfcdb9aef74d9d30e0`; byte-for-byte yes | mechanism reader accepts schema 1, exact `LegacyUnknown`, Phase-B unavailable |
| `N-F13` → `failure/eis_schema2.json` | 1; copy `tests/fixtures/a0_artifact_contracts/eis_fit_schema2_correct_kind.json`; source/final `7c7bc97b0a83040077bfb11bab3caaa0a2176f2e85b3580d4d75388a7e36478f`; byte-for-byte yes | EIS policy rejects `UnsupportedSchemaVersion { actual:2 }` |
| `N-F14` → `failure/wrong_kind.json` | 1; copy `tests/fixtures/a0_artifact_contracts/eis_fit_schema2_wrong_kind.json`; source/final `e66762f2eb6828a8ac79f1f41857d30e66e77b8819387f11b0d1b36761950097`; byte-for-byte yes | requested reader rejects `IncompatibleKind` |
| `N-F15` → `compat/health_sensor_mismatch.json` | 3 from `N-F02`: replace only `/lineage/Known/identity` with `N-ID01` | known health, scope `b-e2e-1/sensor-mismatch/Unspecified`; `1b55ca385f049b370b58ae599f69449e0fac45f40d1355e43873c59d04f95ad3` |
| `N-F16` → `compat/mechanism_experiment_mismatch.json` | 3 from `N-F01`: identity → `N-ID02` | known mechanism, scope `experiment-mismatch/Unspecified/Unspecified`; `95830d771fa9e6c7fc244ecbb2344080309012ccf66bb3bb503cb69fc0f082a7` |
| `N-F17,N-F18` → `compat/health_unknown_scope.json`, `compat/mechanism_unknown_scope.json` | 3 from `N-F02,N-F01`: identity → `N-ID03,N-ID04` | known values with `experiment_scope=Unknown`; `3437195132fe85d9c3f05914c9e5b455c064e0f7ba94654b90f5379840af748b`, `84b3ce786d054af341b6ddfe483fa35c33589cba6159550b0d556da8f7900c8e` |
| `N-F19` → `compat/eis_sensor_mismatch.json` | 3 from `N-F03`: identity → `N-ID05` | known EIS, scope `b-e2e-1/sensor-mismatch/Unspecified`; `680ef043edb581b525e52794d87f165e4c51b8ba18024858914372379b84b17e` |
| `N-F20,N-F21` → `compat/health_family.json`, `compat/mechanism_family.json` | 3 from `N-F02,N-F01`: identity → `N-ID06,N-ID07` | known values, only `family-health`, only `family-mechanism`; `79f1d3efa7955129612cf78202283448b7279ba75a1429c8585d3cd0bffeeb65`, `f2277af261b80b283f0f510659c61311b0429e8efe0509474ef021ddf7c0b526` |
| `N-F22` → `health/missing_unit.json` | 3 from `N-F02`: `/features/0/unit` old `"V"` → new `""`; then identity → `N-ID08` | known health; `805af582264595f306915365ecea4848e735535714e25662f9a6ee7fa1d1eafa` |
| `N-F23` → `health/comparable_with_warnings.json` | 2; complete base64 `N-L07` below | known health `N-ID09`; source current `0.21472615802499273`, baseline `0.058`, comparability `comparable_with_warnings`, absolute `0.15672615802499273`, relative `2.702175138361943`, reason `temperature differs within configured tolerance`; `0b23a8cf4a60d9a136abd6fe89715c1a9cfe826c78b068f4f5d7428b81fb8082` |
| `N-F24` → `eis/nyquist_bode.json` | 3 from `N-F03`: frequency `[1.0]` → `[1.0,10.0]`; source real `[1.0]` → `[1.0,2.0]`; source imaginary `[0.0]` → `[-2.0,-1.0]`; derived magnitude `[1.0]` → `[2.23606797749979,2.23606797749979]`; phase `[0.0]` → `[-63.43494882292201,-26.56505117707799]`; fitted real/imag/magnitude/phase → `[1.5,2.5]`, `[-1.5,-0.5]`, `[2.1213203435596424,2.5495097567963922]`, `[-45.0,-11.309932474020215]`; identity → `N-ID10` | known EIS; `b03c5b651ebd1980ed564e03ff88aa6ca14481c6ac1f39856d91dc956ee2aaf0` |
| `N-F25,N-F26` → `transient/zero_selected_fit.json`, `transient/duplicate_selected_fit.json` | 3 from `N-F04`: `/events/0/candidate_fits` old one exact `single/converged` value → `[]`; or append an exact second copy of element 0; identities → `N-ID11,N-ID12` | known transient; `811fd379536f89802fff7446ad9356162a787a320027c20c7d58815617ace312`, `659768ae6748fbbfb460ce7b9f59d160dd5bc3c532a9ffb4a108da9158cd859a` |
| `N-F27` → `model/missing_values.json` | 3 from `N-F09`: `/points/0/observed_voltage_v` `0.002` → `null`; `/points/0/unexplained_residual_v` `0.002` → `null`; legacy lineage/provenance unchanged | schema-5 legacy model; `6134d17ae62c52cc931188d9fc55d2c1d10c7fdbfc4b0c79a821bbd28a59aec4` |
| `N-F28` → `output/keep.txt` | 2 literal bytes `keep\n`; `f660a7996deacfbc7560e4240054a8ad82eb02fe25a95064257e07084bcacb85` | unmanaged-output sentinel; reader none |
| `N-F29` → `scale/history_seed.json` | 2 literal bytes `{"schema":"phase_d_history_seed_v1","artifact_id":"history-root","evidence_id":"evidence-root"}\n`; `925fea5c7dcdb78097b8f094cb1664eb9c09ca0baf4892877059fde6491f3be6` | exact scale seed; reader none |
| `N-F30` → `scale/large_history.json` | 3 from `N-F29`: compact no-whitespace UTF-8 JSON with one LF, object key order exactly `schema,history_ids,evidence_ids`; `history_ids[j]` is the ASCII concatenation `history-` + base-10 `j` left-padded with zeroes to width 4 for every integer `j=0..999`; `evidence_ids[j]` is `evidence-` + base-10 `j` left-padded to width 5 for every `j=0..9999`; arrays use increasing `j` | exact 1,000/10,000 input; reader none; `f105c609a9b2ae098f0f4546312f951af80b516c7da6f8bfacf82ba559857d1b` |
| `N-F31` → `mechanism/timescale_cmp01.json` | 2; complete base64 literal `N-L08` below | mechanism schema 4, known identity `sha256:03487f7022a2fbb77bb85bfbd1e3c30a35aff1d1efca7d231d7b8943fd7a349e`, producer `phase-d-fixture-v1`, complete inherited provenance; `d0a373578981f8db5f69e722d484c3be32e78e2f55d563d22125b3692332aee6` |

`N-ID01`=`sha256:7aeb641c739a87e4cbd55cb75b71feec966ec881e42bf22da2226c9b487394f7`,
`N-ID02`=`sha256:37f3d20e43cb576fe42e34362ca02882a2489b8299c38bbf7764ec8d8678aceb`,
`N-ID03`=`sha256:b10b0fea4fd512718d46c0b97a7bbe97df5e6d44aa0164c8632e68679b679ffb`,
`N-ID04`=`sha256:3cfe838b269eeb9e489581fa21fac756b190f717e118abfc5ada2c26cccef47b`,
`N-ID05`=`sha256:224abd8f108152be50a884380542734c99ff1fa24b46b9fa216454f3ef5e0567`,
`N-ID06`=`sha256:f8bf2bef2da2ecd929c6bfd7875ba23201e0fc66b7a4a78ec3ee2ecabc8304aa`,
`N-ID07`=`sha256:cb4828f9b8dda5edd22257dcbe2ee23347486b4674e2da3b03eea90c1d819986`,
`N-ID08`=`sha256:f9df94298d341369af3be17e8f937e521efc290a63f8173afae9faf1d17dcf55`,
`N-ID09`=`sha256:d9346376ff3bbd4fa4f3a4387c04c3a8c4187400f653cd45a6e27cad4412bac8`,
`N-ID10`=`sha256:fa7240f2519026d238801b7e53dba95bdba5d46364df2dd9f9cd19316dd7d138`,
`N-ID11`=`sha256:c84120086e1dcf8d30ecc084599fad4fba7de95d671bfc50b1133bf78e5c00fd`,
`N-ID12`=`sha256:19d957ea6d8c33e4afb5b3a87123befcb984cd811a6a7f57ef7b8f42fa39ee79`.
Every `N-ID*` has the kind/schema/scopes/family in its row, producer
`phase-d-fixture-v1`, semantic SHA equal to its 64 digits, and the unchanged
complete serialized base provenance. No derived valid artifact has a deferred
producer, identity, dependency, scope, family, or provenance value.

```text
N-L07 health/comparable_with_warnings.json
ewogICJhcnRpZmFjdF9raW5kIjogImhlYWx0aF9hc3Nlc3NtZW50IiwKICAiYXNzZXNzbWVudF9pZCI6ICJoZWFsdGg6c2lnbmFsOmEwLXRlc3Q6RTEiLAogICJiYXNlbGluZV9jb21wYXJpc29uIjogWwogICAgewogICAgICAiYWJzb2x1dGVfZGlmZmVyZW5jZSI6IDAuMTU2NzI2MTU4MDI0OTkyNzMsCiAgICAgICJiYXNlbGluZV9zYW1wbGVfY291bnQiOiAwLAogICAgICAiYmFzZWxpbmVfdmFsdWUiOiAwLjA1OCwKICAgICAgImNvbXBhcmFiaWxpdHkiOiAiQ29tcGFyYWJsZVdpdGhXYXJuaW5ncyIsCiAgICAgICJjdXJyZW50X3ZhbHVlIjogMC4yMTQ3MjYxNTgwMjQ5OTI3MywKICAgICAgImVtcGlyaWNhbF9wZXJjZW50aWxlIjogbnVsbCwKICAgICAgImZlYXR1cmUiOiAic2lnbmFsLnJtc19ub2lzZSIsCiAgICAgICJsb2dfcmF0aW8iOiBudWxsLAogICAgICAib3ZlcnJpZGVfcmVhc29uIjogInRlbXBlcmF0dXJlIGRpZmZlcnMgd2l0aGluIGNvbmZpZ3VyZWQgdG9sZXJhbmNlIiwKICAgICAgInJhbmdlX3Bvc2l0aW9uX3BlcmNlbnQiOiBudWxsLAogICAgICAicmVsYXRpdmVfZGlmZmVyZW5jZSI6IDIuNzAyMTc1MTM4MzYxOTQzLAogICAgICAicm9idXN0X3pfc2NvcmUiOiBudWxsLAogICAgICAiel9zY29yZSI6IG51bGwKICAgIH0sCiAgICB7CiAgICAgICJhYnNvbHV0ZV9kaWZmZXJlbmNlIjogbnVsbCwKICAgICAgImJhc2VsaW5lX3NhbXBsZV9jb3VudCI6IDAsCiAgICAgICJiYXNlbGluZV92YWx1ZSI6IG51bGwsCiAgICAgICJjb21wYXJhYmlsaXR5IjogInVua25vd24iLAogICAgICAiY3VycmVudF92YWx1ZSI6IDEuODUxMzYyNDQ4MDIzODk2NGUtNiwKICAgICAgImVtcGlyaWNhbF9wZXJjZW50aWxlIjogbnVsbCwKICAgICAgImZlYXR1cmUiOiAic2lnbmFsLnJvYnVzdF9ub2lzZV9zdGFuZGFyZF9kZXZpYXRpb24iLAogICAgICAibG9nX3JhdGlvIjogbnVsbCwKICAgICAgIm92ZXJyaWRlX3JlYXNvbiI6ICJiYXNlbGluZSB1bmF2YWlsYWJsZSIsCiAgICAgICJyYW5nZV9wb3NpdGlvbl9wZXJjZW50IjogbnVsbCwKICAgICAgInJlbGF0aXZlX2RpZmZlcmVuY2UiOiBudWxsLAogICAgICAicm9idXN0X3pfc2NvcmUiOiBudWxsLAogICAgICAiel9zY29yZSI6IG51bGwKICAgIH0sCiAgICB7CiAgICAgICJhYnNvbHV0ZV9kaWZmZXJlbmNlIjogbnVsbCwKICAgICAgImJhc2VsaW5lX3NhbXBsZV9jb3VudCI6IDAsCiAgICAgICJiYXNlbGluZV92YWx1ZSI6IG51bGwsCiAgICAgICJjb21wYXJhYmlsaXR5IjogInVua25vd24iLAogICAgICAiY3VycmVudF92YWx1ZSI6IDAuMDk5OTk5OTk5OTk4NjExMTYsCiAgICAgICJlbXBpcmljYWxfcGVyY2VudGlsZSI6IG51bGwsCiAgICAgICJmZWF0dXJlIjogInNpZ25hbC5wZWFrX3RvX3BlYWsiLAogICAgICAibG9nX3JhdGlvIjogbnVsbCwKICAgICAgIm92ZXJyaWRlX3JlYXNvbiI6ICJiYXNlbGluZSB1bmF2YWlsYWJsZSIsCiAgICAgICJyYW5nZV9wb3NpdGlvbl9wZXJjZW50IjogbnVsbCwKICAgICAgInJlbGF0aXZlX2RpZmZlcmVuY2UiOiBudWxsLAogICAgICAicm9idXN0X3pfc2NvcmUiOiBudWxsLAogICAgICAiel9zY29yZSI6IG51bGwKICAgIH0sCiAgICB7CiAgICAgICJhYnNvbHV0ZV9kaWZmZXJlbmNlIjogbnVsbCwKICAgICAgImJhc2VsaW5lX3NhbXBsZV9jb3VudCI6IDAsCiAgICAgICJiYXNlbGluZV92YWx1ZSI6IG51bGwsCiAgICAgICJjb21wYXJhYmlsaXR5IjogInVua25vd24iLAogICAgICAiY3VycmVudF92YWx1ZSI6IG51bGwsCiAgICAgICJlbXBpcmljYWxfcGVyY2VudGlsZSI6IG51bGwsCiAgICAgICJmZWF0dXJlIjogInNpZ25hbC5hbGxhbl9taW5pbXVtIiwKICAgICAgImxvZ19yYXRpbyI6IG51bGwsCiAgICAgICJvdmVycmlkZV9yZWFzb24iOiAiYmFzZWxpbmUgdW5hdmFpbGFibGUiLAogICAgICAicmFuZ2VfcG9zaXRpb25fcGVyY2VudCI6IG51bGwsCiAgICAgICJyZWxhdGl2ZV9kaWZmZXJlbmNlIjogbnVsbCwKICAgICAgInJvYnVzdF96X3Njb3JlIjogbnVsbCwKICAgICAgInpfc2NvcmUiOiBudWxsCiAgICB9LAogICAgewogICAgICAiYWJzb2x1dGVfZGlmZmVyZW5jZSI6IG51bGwsCiAgICAgICJiYXNlbGluZV9zYW1wbGVfY291bnQiOiAwLAogICAgICAiYmFzZWxpbmVfdmFsdWUiOiBudWxsLAogICAgICAiY29tcGFyYWJpbGl0eSI6ICJ1bmtub3duIiwKICAgICAgImN1cnJlbnRfdmFsdWUiOiBudWxsLAogICAgICAiZW1waXJpY2FsX3BlcmNlbnRpbGUiOiBudWxsLAogICAgICAiZmVhdHVyZSI6ICJzaWduYWwuYWxsYW5fbWluaW11bV9hdmVyYWdpbmdfdGltZSIsCiAgICAgICJsb2dfcmF0aW8iOiBudWxsLAogICAgICAib3ZlcnJpZGVfcmVhc29uIjogImJhc2VsaW5lIHVuYXZhaWxhYmxlIiwKICAgICAgInJhbmdlX3Bvc2l0aW9uX3BlcmNlbnQiOiBudWxsLAogICAgICAicmVsYXRpdmVfZGlmZmVyZW5jZSI6IG51bGwsCiAgICAgICJyb2J1c3Rfel9zY29yZSI6IG51bGwsCiAgICAgICJ6X3Njb3JlIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImFic29sdXRlX2RpZmZlcmVuY2UiOiBudWxsLAogICAgICAiYmFzZWxpbmVfc2FtcGxlX2NvdW50IjogMCwKICAgICAgImJhc2VsaW5lX3ZhbHVlIjogbnVsbCwKICAgICAgImNvbXBhcmFiaWxpdHkiOiAidW5rbm93biIsCiAgICAgICJjdXJyZW50X3ZhbHVlIjogLTMuMzc0ODg1NjExMTI3NDkxN2UtNiwKICAgICAgImVtcGlyaWNhbF9wZXJjZW50aWxlIjogbnVsbCwKICAgICAgImZlYXR1cmUiOiAic2lnbmFsLnJvYnVzdF9kcmlmdF9yYXRlIiwKICAgICAgImxvZ19yYXRpbyI6IG51bGwsCiAgICAgICJvdmVycmlkZV9yZWFzb24iOiAiYmFzZWxpbmUgdW5hdmFpbGFibGUiLAogICAgICAicmFuZ2VfcG9zaXRpb25fcGVyY2VudCI6IG51bGwsCiAgICAgICJyZWxhdGl2ZV9kaWZmZXJlbmNlIjogbnVsbCwKICAgICAgInJvYnVzdF96X3Njb3JlIjogbnVsbCwKICAgICAgInpfc2NvcmUiOiBudWxsCiAgICB9LAogICAgewogICAgICAiYWJzb2x1dGVfZGlmZmVyZW5jZSI6IG51bGwsCiAgICAgICJiYXNlbGluZV9zYW1wbGVfY291bnQiOiAwLAogICAgICAiYmFzZWxpbmVfdmFsdWUiOiBudWxsLAogICAgICAiY29tcGFyYWJpbGl0eSI6ICJ1bmtub3duIiwKICAgICAgImN1cnJlbnRfdmFsdWUiOiAwLjAsCiAgICAgICJlbXBpcmljYWxfcGVyY2VudGlsZSI6IG51bGwsCiAgICAgICJmZWF0dXJlIjogInNpZ25hbC5zcGlrZV9mcmFjdGlvbiIsCiAgICAgICJsb2dfcmF0aW8iOiBudWxsLAogICAgICAib3ZlcnJpZGVfcmVhc29uIjogImJhc2VsaW5lIHVuYXZhaWxhYmxlIiwKICAgICAgInJhbmdlX3Bvc2l0aW9uX3BlcmNlbnQiOiBudWxsLAogICAgICAicmVsYXRpdmVfZGlmZmVyZW5jZSI6IG51bGwsCiAgICAgICJyb2J1c3Rfel9zY29yZSI6IG51bGwsCiAgICAgICJ6X3Njb3JlIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImFic29sdXRlX2RpZmZlcmVuY2UiOiBudWxsLAogICAgICAiYmFzZWxpbmVfc2FtcGxlX2NvdW50IjogMCwKICAgICAgImJhc2VsaW5lX3ZhbHVlIjogbnVsbCwKICAgICAgImNvbXBhcmFiaWxpdHkiOiAidW5rbm93biIsCiAgICAgICJjdXJyZW50X3ZhbHVlIjogMC4wLAogICAgICAiZW1waXJpY2FsX3BlcmNlbnRpbGUiOiBudWxsLAogICAgICAiZmVhdHVyZSI6ICJzaWduYWwubWlzc2luZ19mcmFjdGlvbiIsCiAgICAgICJsb2dfcmF0aW8iOiBudWxsLAogICAgICAib3ZlcnJpZGVfcmVhc29uIjogImJhc2VsaW5lIHVuYXZhaWxhYmxlIiwKICAgICAgInJhbmdlX3Bvc2l0aW9uX3BlcmNlbnQiOiBudWxsLAogICAgICAicmVsYXRpdmVfZGlmZmVyZW5jZSI6IG51bGwsCiAgICAgICJyb2J1c3Rfel9zY29yZSI6IG51bGwsCiAgICAgICJ6X3Njb3JlIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImFic29sdXRlX2RpZmZlcmVuY2UiOiBudWxsLAogICAgICAiYmFzZWxpbmVfc2FtcGxlX2NvdW50IjogMCwKICAgICAgImJhc2VsaW5lX3ZhbHVlIjogbnVsbCwKICAgICAgImNvbXBhcmFiaWxpdHkiOiAidW5rbm93biIsCiAgICAgICJjdXJyZW50X3ZhbHVlIjogMC4wNTQ5NjQ3MTg2Mjc5MzA3MiwKICAgICAgImVtcGlyaWNhbF9wZXJjZW50aWxlIjogbnVsbCwKICAgICAgImZlYXR1cmUiOiAic2lnbmFsLnNhbXBsaW5nX2lycmVndWxhcml0eSIsCiAgICAgICJsb2dfcmF0aW8iOiBudWxsLAogICAgICAib3ZlcnJpZGVfcmVhc29uIjogImJhc2VsaW5lIHVuYXZhaWxhYmxlIiwKICAgICAgInJhbmdlX3Bvc2l0aW9uX3BlcmNlbnQiOiBudWxsLAogICAgICAicmVsYXRpdmVfZGlmZmVyZW5jZSI6IG51bGwsCiAgICAgICJyb2J1c3Rfel9zY29yZSI6IG51bGwsCiAgICAgICJ6X3Njb3JlIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImFic29sdXRlX2RpZmZlcmVuY2UiOiBudWxsLAogICAgICAiYmFzZWxpbmVfc2FtcGxlX2NvdW50IjogMCwKICAgICAgImJhc2VsaW5lX3ZhbHVlIjogbnVsbCwKICAgICAgImNvbXBhcmFiaWxpdHkiOiAidW5rbm93biIsCiAgICAgICJjdXJyZW50X3ZhbHVlIjogbnVsbCwKICAgICAgImVtcGlyaWNhbF9wZXJjZW50aWxlIjogbnVsbCwKICAgICAgImZlYXR1cmUiOiAic2lnbmFsLmNvbW1vbl9tb2RlX2ZyYWN0aW9uIiwKICAgICAgImxvZ19yYXRpbyI6IG51bGwsCiAgICAgICJvdmVycmlkZV9yZWFzb24iOiAiYmFzZWxpbmUgdW5hdmFpbGFibGUiLAogICAgICAicmFuZ2VfcG9zaXRpb25fcGVyY2VudCI6IG51bGwsCiAgICAgICJyZWxhdGl2ZV9kaWZmZXJlbmNlIjogbnVsbCwKICAgICAgInJvYnVzdF96X3Njb3JlIjogbnVsbCwKICAgICAgInpfc2NvcmUiOiBudWxsCiAgICB9LAogICAgewogICAgICAiYWJzb2x1dGVfZGlmZmVyZW5jZSI6IG51bGwsCiAgICAgICJiYXNlbGluZV9zYW1wbGVfY291bnQiOiAwLAogICAgICAiYmFzZWxpbmVfdmFsdWUiOiBudWxsLAogICAgICAiY29tcGFyYWJpbGl0eSI6ICJ1bmtub3duIiwKICAgICAgImN1cnJlbnRfdmFsdWUiOiBudWxsLAogICAgICAiZW1waXJpY2FsX3BlcmNlbnRpbGUiOiBudWxsLAogICAgICAiZmVhdHVyZSI6ICJtZWNoYW5pc20udGltZXNjYWxlX3JhdGlvIiwKICAgICAgImxvZ19yYXRpbyI6IG51bGwsCiAgICAgICJvdmVycmlkZV9yZWFzb24iOiAiYmFzZWxpbmUgdW5hdmFpbGFibGUiLAogICAgICAicmFuZ2VfcG9zaXRpb25fcGVyY2VudCI6IG51bGwsCiAgICAgICJyZWxhdGl2ZV9kaWZmZXJlbmNlIjogbnVsbCwKICAgICAgInJvYnVzdF96X3Njb3JlIjogbnVsbCwKICAgICAgInpfc2NvcmUiOiBudWxsCiAgICB9LAogICAgewogICAgICAiYWJzb2x1dGVfZGlmZmVyZW5jZSI6IG51bGwsCiAgICAgICJiYXNlbGluZV9zYW1wbGVfY291bnQiOiAwLAogICAgICAiYmFzZWxpbmVfdmFsdWUiOiBudWxsLAogICAgICAiY29tcGFyYWJpbGl0eSI6ICJ1bmtub3duIiwKICAgICAgImN1cnJlbnRfdmFsdWUiOiAwLjAsCiAgICAgICJlbXBpcmljYWxfcGVyY2VudGlsZSI6IG51bGwsCiAgICAgICJmZWF0dXJlIjogIm1lY2hhbmlzbS5zdHJvbmdfY29tcGFyaXNvbnMiLAogICAgICAibG9nX3JhdGlvIjogbnVsbCwKICAgICAgIm92ZXJyaWRlX3JlYXNvbiI6ICJiYXNlbGluZSB1bmF2YWlsYWJsZSIsCiAgICAgICJyYW5nZV9wb3NpdGlvbl9wZXJjZW50IjogbnVsbCwKICAgICAgInJlbGF0aXZlX2RpZmZlcmVuY2UiOiBudWxsLAogICAgICAicm9idXN0X3pfc2NvcmUiOiBudWxsLAogICAgICAiel9zY29yZSI6IG51bGwKICAgIH0sCiAgICB7CiAgICAgICJhYnNvbHV0ZV9kaWZmZXJlbmNlIjogbnVsbCwKICAgICAgImJhc2VsaW5lX3NhbXBsZV9jb3VudCI6IDAsCiAgICAgICJiYXNlbGluZV92YWx1ZSI6IG51bGwsCiAgICAgICJjb21wYXJhYmlsaXR5IjogInVua25vd24iLAogICAgICAiY3VycmVudF92YWx1ZSI6IDAuMCwKICAgICAgImVtcGlyaWNhbF9wZXJjZW50aWxlIjogbnVsbCwKICAgICAgImZlYXR1cmUiOiAibWVjaGFuaXNtLmNvbnRyYWRpY3RvcnlfY29tcGFyaXNvbnMiLAogICAgICAibG9nX3JhdGlvIjogbnVsbCwKICAgICAgIm92ZXJyaWRlX3JlYXNvbiI6ICJiYXNlbGluZSB1bmF2YWlsYWJsZSIsCiAgICAgICJyYW5nZV9wb3NpdGlvbl9wZXJjZW50IjogbnVsbCwKICAgICAgInJlbGF0aXZlX2RpZmZlcmVuY2UiOiBudWxsLAogICAgICAicm9idXN0X3pfc2NvcmUiOiBudWxsLAogICAgICAiel9zY29yZSI6IG51bGwKICAgIH0KICBdLAogICJjb25maWd1cmF0aW9uIjogewogICAgImFzc2Vzc21lbnQiOiB7CiAgICAgICJhbGxvd193YXJuaW5nX2FydGlmYWN0cyI6IHRydWUsCiAgICAgICJtaW5pbXVtX2RvbWFpbnNfZm9yX2Fzc2Vzc21lbnQiOiAyLAogICAgICAibWluaW11bV9kb21haW5zX2Zvcl9tZWNoYW5pc3RpY19maW5kaW5nIjogMgogICAgfSwKICAgICJiYXNlbGluZSI6IHsKICAgICAgIm1pbmltdW1fcmVxdWlyZWRfcmVjb3JkcyI6IDMsCiAgICAgICJyb2J1c3Rfc3RhdGlzdGljcyI6IHRydWUKICAgIH0sCiAgICAiY29tcGFyYWJpbGl0eSI6IHsKICAgICAgIm1heGltdW1fdGVtcGVyYXR1cmVfZGlmZmVyZW5jZV9rIjogMi4wLAogICAgICAicmVxdWlyZV9zYW1lX2FuYWx5dGUiOiB0cnVlLAogICAgICAicmVxdWlyZV9zYW1lX3NhbXBsZV9tYXRyaXgiOiB0cnVlLAogICAgICAicmVxdWlyZV9zYW1lX3NlbnNvcl9kZXNpZ24iOiB0cnVlCiAgICB9LAogICAgImV4cG9ydCI6IHsKICAgICAgImFzc2Vzc21lbnRfZmlsZW5hbWUiOiAiaGVhbHRoX2Fzc2Vzc21lbnQuanNvbiIsCiAgICAgICJiYXNlbGluZV9maWxlbmFtZSI6ICJoZWFsdGhfYmFzZWxpbmUuanNvbiIsCiAgICAgICJmZWF0dXJlc19maWxlbmFtZSI6ICJoZWFsdGhfZmVhdHVyZXMuY3N2IiwKICAgICAgImZpbmRpbmdzX2ZpbGVuYW1lIjogImhlYWx0aF9maW5kaW5ncy5jc3YiLAogICAgICAicmVwb3J0X2ZpbGVuYW1lIjogImhlYWx0aF9yZXBvcnQudHh0IiwKICAgICAgInRyZW5kc19maWxlbmFtZSI6ICJoZWFsdGhfdHJlbmRzLmNzdiIKICAgIH0sCiAgICAibm9ybWFsaXphdGlvbiI6IHsKICAgICAgIm1pbmltdW1fYmFzZWxpbmVfcmVjb3Jkc19mb3Jfel9zY29yZSI6IDUsCiAgICAgICJ1c2VfcmVsYXRpdmVfZGlmZmVyZW5jZSI6IHRydWUsCiAgICAgICJ1c2Vfcm9idXN0X3pfc2NvcmUiOiB0cnVlCiAgICB9LAogICAgInBsb3R0aW5nIjogewogICAgICAiZW5hYmxlZCI6IHRydWUKICAgIH0sCiAgICAicnVsZXMiOiBbCiAgICAgIHsKICAgICAgICAiYWxsX29mIjogWwogICAgICAgICAgewogICAgICAgICAgICAiZmVhdHVyZSI6ICJzaWduYWwucm9idXN0X25vaXNlX3N0YW5kYXJkX2RldmlhdGlvbiIsCiAgICAgICAgICAgICJvcGVyYXRvciI6ICJyb2J1c3Rfel9ncmVhdGVyX3RoYW4iLAogICAgICAgICAgICAidmFsdWUiOiAzLjAKICAgICAgICAgIH0KICAgICAgICBdLAogICAgICAgICJhbHRlcm5hdGl2ZV9leHBsYW5hdGlvbnMiOiBbXSwKICAgICAgICAiYW55X29mIjogW10sCiAgICAgICAgImZpbmRpbmciOiAiZWxldmF0ZWRfbm9pc2UiLAogICAgICAgICJtaW5pbXVtX2Jhc2VsaW5lX3JlY29yZHMiOiAwLAogICAgICAgICJtaW5pbXVtX2V2aWRlbmNlX2RvbWFpbnMiOiAxLAogICAgICAgICJydWxlX2lkIjogImVsZXZhdGVkLW5vaXNlIiwKICAgICAgICAic2V2ZXJpdHkiOiAibW9kZXJhdGUiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAiYWxsX29mIjogWwogICAgICAgICAgewogICAgICAgICAgICAiZmVhdHVyZSI6ICJ0cmFuc2llbnQudGF1X3Nsb3ciLAogICAgICAgICAgICAib3BlcmF0b3IiOiAicmVsYXRpdmVfaW5jcmVhc2VfZ3JlYXRlcl90aGFuIiwKICAgICAgICAgICAgInZhbHVlIjogMS4wCiAgICAgICAgICB9CiAgICAgICAgXSwKICAgICAgICAiYWx0ZXJuYXRpdmVfZXhwbGFuYXRpb25zIjogWwogICAgICAgICAgImVudmlyb25tZW50YWwgbWlzbWF0Y2giLAogICAgICAgICAgImluY29tcGxldGUgYmFzZWxpbmUgY29udGV4dCIKICAgICAgICBdLAogICAgICAgICJhbnlfb2YiOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJmZWF0dXJlIjogImNhbGlicmF0aW9uLnNsb3BlX2VmZmljaWVuY3kiLAogICAgICAgICAgICAib3BlcmF0b3IiOiAicmVsYXRpdmVfZGVjcmVhc2VfZ3JlYXRlcl90aGFuIiwKICAgICAgICAgICAgInZhbHVlIjogMC4yCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAiZmVhdHVyZSI6ICJlaXMucm9sZS50cmFuc3BvcnQucmVsYXhhdGlvbl90aW1lc2NhbGUiLAogICAgICAgICAgICAib3BlcmF0b3IiOiAicmVsYXRpdmVfaW5jcmVhc2VfZ3JlYXRlcl90aGFuIiwKICAgICAgICAgICAgInZhbHVlIjogMS4wCiAgICAgICAgICB9CiAgICAgICAgXSwKICAgICAgICAiZmluZGluZyI6ICJwcm9iYWJsZV9mb3VsaW5nIiwKICAgICAgICAibWluaW11bV9iYXNlbGluZV9yZWNvcmRzIjogMCwKICAgICAgICAibWluaW11bV9ldmlkZW5jZV9kb21haW5zIjogMiwKICAgICAgICAicnVsZV9pZCI6ICJwcm9iYWJsZS1mb3VsaW5nIiwKICAgICAgICAic2V2ZXJpdHkiOiAibWFqb3IiCiAgICAgIH0KICAgIF0sCiAgICAic2NoZW1hX3ZlcnNpb24iOiAxCiAgfSwKICAiZG9tYWluX2Fzc2Vzc21lbnRzIjogWwogICAgewogICAgICAiYXZhaWxhYmxlX2ZlYXR1cmVzIjogMiwKICAgICAgImNvbmZpZGVuY2UiOiAibW9kZXJhdGUiLAogICAgICAiZG9tYWluIjogImRhdGFfcXVhbGl0eSIsCiAgICAgICJmZWF0dXJlX2NvdW50IjogMiwKICAgICAgInN0YXR1cyI6ICJ3aXRoaW5fYmFzZWxpbmUiLAogICAgICAid2FybmluZ19jb3VudCI6IDAKICAgIH0sCiAgICB7CiAgICAgICJhdmFpbGFibGVfZmVhdHVyZXMiOiA0LAogICAgICAiY29uZmlkZW5jZSI6ICJtb2RlcmF0ZSIsCiAgICAgICJkb21haW4iOiAic2lnbmFsX25vaXNlIiwKICAgICAgImZlYXR1cmVfY291bnQiOiA3LAogICAgICAic3RhdHVzIjogIndpdGhpbl9iYXNlbGluZSIsCiAgICAgICJ3YXJuaW5nX2NvdW50IjogMAogICAgfSwKICAgIHsKICAgICAgImF2YWlsYWJsZV9mZWF0dXJlcyI6IDEsCiAgICAgICJjb25maWRlbmNlIjogIm1vZGVyYXRlIiwKICAgICAgImRvbWFpbiI6ICJkcmlmdCIsCiAgICAgICJmZWF0dXJlX2NvdW50IjogMSwKICAgICAgInN0YXR1cyI6ICJ3aXRoaW5fYmFzZWxpbmUiLAogICAgICAid2FybmluZ19jb3VudCI6IDAKICAgIH0sCiAgICB7CiAgICAgICJhdmFpbGFibGVfZmVhdHVyZXMiOiAwLAogICAgICAiY29uZmlkZW5jZSI6ICJpbnN1ZmZpY2llbnQiLAogICAgICAiZG9tYWluIjogImR5bmFtaWNfcmVzcG9uc2UiLAogICAgICAiZmVhdHVyZV9jb3VudCI6IDAsCiAgICAgICJzdGF0dXMiOiAid2l0aGluX2Jhc2VsaW5lIiwKICAgICAgIndhcm5pbmdfY291bnQiOiAwCiAgICB9LAogICAgewogICAgICAiYXZhaWxhYmxlX2ZlYXR1cmVzIjogMCwKICAgICAgImNvbmZpZGVuY2UiOiAiaW5zdWZmaWNpZW50IiwKICAgICAgImRvbWFpbiI6ICJjYWxpYnJhdGlvbiIsCiAgICAgICJmZWF0dXJlX2NvdW50IjogMCwKICAgICAgInN0YXR1cyI6ICJ3aXRoaW5fYmFzZWxpbmUiLAogICAgICAid2FybmluZ19jb3VudCI6IDAKICAgIH0sCiAgICB7CiAgICAgICJhdmFpbGFibGVfZmVhdHVyZXMiOiAwLAogICAgICAiY29uZmlkZW5jZSI6ICJpbnN1ZmZpY2llbnQiLAogICAgICAiZG9tYWluIjogImltcGVkYW5jZSIsCiAgICAgICJmZWF0dXJlX2NvdW50IjogMCwKICAgICAgInN0YXR1cyI6ICJ3aXRoaW5fYmFzZWxpbmUiLAogICAgICAid2FybmluZ19jb3VudCI6IDAKICAgIH0sCiAgICB7CiAgICAgICJhdmFpbGFibGVfZmVhdHVyZXMiOiAyLAogICAgICAiY29uZmlkZW5jZSI6ICJtb2RlcmF0ZSIsCiAgICAgICJkb21haW4iOiAibWVjaGFuaXNtX2V2aWRlbmNlIiwKICAgICAgImZlYXR1cmVfY291bnQiOiAzLAogICAgICAic3RhdHVzIjogIndpdGhpbl9iYXNlbGluZSIsCiAgICAgICJ3YXJuaW5nX2NvdW50IjogMAogICAgfQogIF0sCiAgImV4cGVyaW1lbnRfaWQiOiBudWxsLAogICJmZWF0dXJlcyI6IFsKICAgIHsKICAgICAgImRvbWFpbiI6ICJzaWduYWxfbm9pc2UiLAogICAgICAibmFtZSI6ICJzaWduYWwucm1zX25vaXNlIiwKICAgICAgInNvdXJjZSI6ICJzaWduYWwiLAogICAgICAidW5pdCI6ICJWIiwKICAgICAgInZhbHVlIjogMC4yMTQ3MjYxNTgwMjQ5OTI3MywKICAgICAgIndhcm5pbmciOiBudWxsCiAgICB9LAogICAgewogICAgICAiZG9tYWluIjogInNpZ25hbF9ub2lzZSIsCiAgICAgICJuYW1lIjogInNpZ25hbC5yb2J1c3Rfbm9pc2Vfc3RhbmRhcmRfZGV2aWF0aW9uIiwKICAgICAgInNvdXJjZSI6ICJzaWduYWwiLAogICAgICAidW5pdCI6ICJWIiwKICAgICAgInZhbHVlIjogMS44NTEzNjI0NDgwMjM4OTY0ZS02LAogICAgICAid2FybmluZyI6IG51bGwKICAgIH0sCiAgICB7CiAgICAgICJkb21haW4iOiAic2lnbmFsX25vaXNlIiwKICAgICAgIm5hbWUiOiAic2lnbmFsLnBlYWtfdG9fcGVhayIsCiAgICAgICJzb3VyY2UiOiAic2lnbmFsIiwKICAgICAgInVuaXQiOiAiViIsCiAgICAgICJ2YWx1ZSI6IDAuMDk5OTk5OTk5OTk4NjExMTYsCiAgICAgICJ3YXJuaW5nIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImRvbWFpbiI6ICJzaWduYWxfbm9pc2UiLAogICAgICAibmFtZSI6ICJzaWduYWwuYWxsYW5fbWluaW11bSIsCiAgICAgICJzb3VyY2UiOiAic2lnbmFsIiwKICAgICAgInVuaXQiOiAiViIsCiAgICAgICJ2YWx1ZSI6IG51bGwsCiAgICAgICJ3YXJuaW5nIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImRvbWFpbiI6ICJzaWduYWxfbm9pc2UiLAogICAgICAibmFtZSI6ICJzaWduYWwuYWxsYW5fbWluaW11bV9hdmVyYWdpbmdfdGltZSIsCiAgICAgICJzb3VyY2UiOiAic2lnbmFsIiwKICAgICAgInVuaXQiOiAicyIsCiAgICAgICJ2YWx1ZSI6IG51bGwsCiAgICAgICJ3YXJuaW5nIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImRvbWFpbiI6ICJkcmlmdCIsCiAgICAgICJuYW1lIjogInNpZ25hbC5yb2J1c3RfZHJpZnRfcmF0ZSIsCiAgICAgICJzb3VyY2UiOiAic2lnbmFsIiwKICAgICAgInVuaXQiOiAiVi9zIiwKICAgICAgInZhbHVlIjogLTMuMzc0ODg1NjExMTI3NDkxN2UtNiwKICAgICAgIndhcm5pbmciOiBudWxsCiAgICB9LAogICAgewogICAgICAiZG9tYWluIjogInNpZ25hbF9ub2lzZSIsCiAgICAgICJuYW1lIjogInNpZ25hbC5zcGlrZV9mcmFjdGlvbiIsCiAgICAgICJzb3VyY2UiOiAic2lnbmFsIiwKICAgICAgInVuaXQiOiAiZnJhY3Rpb24iLAogICAgICAidmFsdWUiOiAwLjAsCiAgICAgICJ3YXJuaW5nIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImRvbWFpbiI6ICJkYXRhX3F1YWxpdHkiLAogICAgICAibmFtZSI6ICJzaWduYWwubWlzc2luZ19mcmFjdGlvbiIsCiAgICAgICJzb3VyY2UiOiAic2lnbmFsIiwKICAgICAgInVuaXQiOiAiZnJhY3Rpb24iLAogICAgICAidmFsdWUiOiAwLjAsCiAgICAgICJ3YXJuaW5nIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImRvbWFpbiI6ICJkYXRhX3F1YWxpdHkiLAogICAgICAibmFtZSI6ICJzaWduYWwuc2FtcGxpbmdfaXJyZWd1bGFyaXR5IiwKICAgICAgInNvdXJjZSI6ICJzaWduYWwiLAogICAgICAidW5pdCI6ICJmcmFjdGlvbiIsCiAgICAgICJ2YWx1ZSI6IDAuMDU0OTY0NzE4NjI3OTMwNzIsCiAgICAgICJ3YXJuaW5nIjogbnVsbAogICAgfSwKICAgIHsKICAgICAgImRvbWFpbiI6ICJzaWduYWxfbm9pc2UiLAogICAgICAibmFtZSI6ICJzaWduYWwuY29tbW9uX21vZGVfZnJhY3Rpb24iLAogICAgICAic291cmNlIjogInNpZ25hbCIsCiAgICAgICJ1bml0IjogImZyYWN0aW9uIiwKICAgICAgInZhbHVlIjogbnVsbCwKICAgICAgIndhcm5pbmciOiBudWxsCiAgICB9LAogICAgewogICAgICAiZG9tYWluIjogIm1lY2hhbmlzbV9ldmlkZW5jZSIsCiAgICAgICJuYW1lIjogIm1lY2hhbmlzbS50aW1lc2NhbGVfcmF0aW8iLAogICAgICAic291cmNlIjogIm1lY2hhbmlzbSIsCiAgICAgICJ1bml0IjogImZyYWN0aW9uIiwKICAgICAgInZhbHVlIjogbnVsbCwKICAgICAgIndhcm5pbmciOiBudWxsCiAgICB9LAogICAgewogICAgICAiZG9tYWluIjogIm1lY2hhbmlzbV9ldmlkZW5jZSIsCiAgICAgICJuYW1lIjogIm1lY2hhbmlzbS5zdHJvbmdfY29tcGFyaXNvbnMiLAogICAgICAic291cmNlIjogIm1lY2hhbmlzbSIsCiAgICAgICJ1bml0IjogImZyYWN0aW9uIiwKICAgICAgInZhbHVlIjogMC4wLAogICAgICAid2FybmluZyI6IG51bGwKICAgIH0sCiAgICB7CiAgICAgICJkb21haW4iOiAibWVjaGFuaXNtX2V2aWRlbmNlIiwKICAgICAgIm5hbWUiOiAibWVjaGFuaXNtLmNvbnRyYWRpY3RvcnlfY29tcGFyaXNvbnMiLAogICAgICAic291cmNlIjogIm1lY2hhbmlzbSIsCiAgICAgICJ1bml0IjogImZyYWN0aW9uIiwKICAgICAgInZhbHVlIjogMC4wLAogICAgICAid2FybmluZyI6IG51bGwKICAgIH0KICBdLAogICJmaW5kaW5ncyI6IFtdLAogICJsaW5lYWdlIjogewogICAgIktub3duIjogewogICAgICAiZGlyZWN0X2RlcGVuZGVuY2llcyI6IFtdLAogICAgICAiaWRlbnRpdHkiOiB7CiAgICAgICAgImFjcXVpc2l0aW9uX2ZhbWlsaWVzIjogewogICAgICAgICAgIktub3duIjogWwogICAgICAgICAgICAicGhhc2UtZC1maXh0dXJlLWZhbWlseSIKICAgICAgICAgIF0KICAgICAgICB9LAogICAgICAgICJhcnRpZmFjdF9pZCI6ICJzaGEyNTY6ZDkzNDYzNzZmZjNiYmQ0ZmE0ZjNhNDM4N2MwNGMzYThjNDE4NzQwMGY2NTNjZDQ1YTZlMjdjYWQ0NDEyYmFjOCIsCiAgICAgICAgImFydGlmYWN0X2tpbmQiOiAiaGVhbHRoX2Fzc2Vzc21lbnQiLAogICAgICAgICJjaGFubmVsX3Njb3BlIjogIlVuc3BlY2lmaWVkIiwKICAgICAgICAiZXhwZXJpbWVudF9zY29wZSI6IHsKICAgICAgICAgICJTaW5nbGUiOiB7CiAgICAgICAgICAgICJleHBlcmltZW50X2lkIjogImItZTJlLTEiCiAgICAgICAgICB9CiAgICAgICAgfSwKICAgICAgICAicHJvZHVjZXJfdmVyc2lvbiI6ICJwaGFzZS1kLWZpeHR1cmUtdjEiLAogICAgICAgICJzY2hlbWFfdmVyc2lvbiI6IDQsCiAgICAgICAgInNlbWFudGljX3NoYTI1NiI6ICJkOTM0NjM3NmZmM2JiZDRmYTRmM2E0Mzg3YzA0YzNhOGM0MTg3NDAwZjY1M2NkNDVhNmUyN2NhZDQ0MTJiYWM4IiwKICAgICAgICAic2Vuc29yX3Njb3BlIjogIlVuc3BlY2lmaWVkIgogICAgICB9CiAgICB9CiAgfSwKICAibWlzc2luZ19kb21haW5zIjogWwogICAgImR5bmFtaWNfcmVzcG9uc2UiLAogICAgImNhbGlicmF0aW9uIiwKICAgICJpbXBlZGFuY2UiCiAgXSwKICAib3ZlcmFsbF9zdGF0dXMiOiAiY3JpdGljYWwiLAogICJwaGFzZV9jIjogewogICAgImNvbmZpZ19zY2hlbWFfdmVyc2lvbiI6IDEsCiAgICAiY29uZmlnX3NoYTI1NiI6ICI5NDY5MDFkMzZmYzc0Mjk1MmM2ZTAzZjA2OGIwOGMzNTQ3ZDFiMzI4ZjAzMWZiNjA2Y2ZhNzQwOTNlMGJlOGE0IiwKICAgICJkaW1lbnNpb25fYXNzZXNzbWVudHMiOiBbCiAgICAgIHsKICAgICAgICAiY2F1c2FsX3N0YXR1cyI6ICJvYnNlcnZlZCIsCiAgICAgICAgImRpbWVuc2lvbiI6ICJzaWduYWxfaW50ZWdyaXR5IiwKICAgICAgICAiZXZpZGVuY2Vfc3RhdGUiOiAiYWRlcXVhdGVfZXZpZGVuY2UiLAogICAgICAgICJleGNsdWRlZF9ldmlkZW5jZV9pZHMiOiBbXSwKICAgICAgICAiaW50ZXJwcmV0YXRpb25fY2F0ZWdvcnkiOiAib2JzZXJ2ZWRfYmVoYXZpb3IiLAogICAgICAgICJyZWFzb25fY29kZXMiOiBbCiAgICAgICAgICAidGhyZXNob2xkX2NyaXRpY2FsIgogICAgICAgIF0sCiAgICAgICAgInNvdXJjZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAic291cmNlX2V2aWRlbmNlX2lkcyI6IFsKICAgICAgICAgICJzaWduYWwuZGVzY3JpcHRpdmUucm1zIiwKICAgICAgICAgICJzaWduYWwuZGVzY3JpcHRpdmUucm9idXN0X3N0YW5kYXJkX2RldmlhdGlvbiIsCiAgICAgICAgICAic2lnbmFsLmRyaWZ0LnRoZWlsX3Nlbi5zbG9wZV92X3Blcl9zIiwKICAgICAgICAgICJzaWduYWwuc3Bpa2VzLmZsYWdnZWRfZnJhY3Rpb24iCiAgICAgICAgXSwKICAgICAgICAic3RhdHVzIjogImNyaXRpY2FsIgogICAgICB9LAogICAgICB7CiAgICAgICAgImNhdXNhbF9zdGF0dXMiOiAiaW5kZXRlcm1pbmF0ZSIsCiAgICAgICAgImRpbWVuc2lvbiI6ICJjYWxpYnJhdGlvbl9oZWFsdGgiLAogICAgICAgICJldmlkZW5jZV9zdGF0ZSI6ICJub19ldmlkZW5jZSIsCiAgICAgICAgImV4Y2x1ZGVkX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJpbnRlcnByZXRhdGlvbl9jYXRlZ29yeSI6ICJjYWxpYnJhdGlvbl9pc3N1ZSIsCiAgICAgICAgInJlYXNvbl9jb2RlcyI6IFsKICAgICAgICAgICJvcHRpb25hbF9zb3VyY2VfYWJzZW50IgogICAgICAgIF0sCiAgICAgICAgInNvdXJjZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAic291cmNlX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJzdGF0dXMiOiAiaW5kZXRlcm1pbmF0ZSIKICAgICAgfSwKICAgICAgewogICAgICAgICJjYXVzYWxfc3RhdHVzIjogImluZGV0ZXJtaW5hdGUiLAogICAgICAgICJkaW1lbnNpb24iOiAiZHluYW1pY19yZXNwb25zZV9oZWFsdGgiLAogICAgICAgICJldmlkZW5jZV9zdGF0ZSI6ICJub19ldmlkZW5jZSIsCiAgICAgICAgImV4Y2x1ZGVkX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJpbnRlcnByZXRhdGlvbl9jYXRlZ29yeSI6ICJvYnNlcnZlZF9iZWhhdmlvciIsCiAgICAgICAgInJlYXNvbl9jb2RlcyI6IFsKICAgICAgICAgICJvcHRpb25hbF9zb3VyY2VfYWJzZW50IgogICAgICAgIF0sCiAgICAgICAgInNvdXJjZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAic291cmNlX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJzdGF0dXMiOiAiaW5kZXRlcm1pbmF0ZSIKICAgICAgfSwKICAgICAgewogICAgICAgICJjYXVzYWxfc3RhdHVzIjogImluZGV0ZXJtaW5hdGUiLAogICAgICAgICJkaW1lbnNpb24iOiAicmVmZXJlbmNlX3N0YWJpbGl0eSIsCiAgICAgICAgImV2aWRlbmNlX3N0YXRlIjogIm5vX2V2aWRlbmNlIiwKICAgICAgICAiZXhjbHVkZWRfZXZpZGVuY2VfaWRzIjogW10sCiAgICAgICAgImludGVycHJldGF0aW9uX2NhdGVnb3J5IjogIm9ic2VydmVkX2JlaGF2aW9yIiwKICAgICAgICAicmVhc29uX2NvZGVzIjogWwogICAgICAgICAgInJlZmVyZW5jZV9hbmNob3JfdW5hdmFpbGFibGUiCiAgICAgICAgXSwKICAgICAgICAic291cmNlX2FydGlmYWN0X2lkcyI6IFtdLAogICAgICAgICJzb3VyY2VfZXZpZGVuY2VfaWRzIjogW10sCiAgICAgICAgInN0YXR1cyI6ICJpbmRldGVybWluYXRlIgogICAgICB9LAogICAgICB7CiAgICAgICAgImNhdXNhbF9zdGF0dXMiOiAiaW5kZXRlcm1pbmF0ZSIsCiAgICAgICAgImRpbWVuc2lvbiI6ICJlbnZpcm9ubWVudGFsX3JvYnVzdG5lc3MiLAogICAgICAgICJldmlkZW5jZV9zdGF0ZSI6ICJub19ldmlkZW5jZSIsCiAgICAgICAgImV4Y2x1ZGVkX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJpbnRlcnByZXRhdGlvbl9jYXRlZ29yeSI6ICJlbnZpcm9ubWVudGFsX2VmZmVjdCIsCiAgICAgICAgInJlYXNvbl9jb2RlcyI6IFsKICAgICAgICAgICJvcHRpb25hbF9zb3VyY2VfYWJzZW50IgogICAgICAgIF0sCiAgICAgICAgInNvdXJjZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAic291cmNlX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJzdGF0dXMiOiAiaW5kZXRlcm1pbmF0ZSIKICAgICAgfSwKICAgICAgewogICAgICAgICJjYXVzYWxfc3RhdHVzIjogImluZGV0ZXJtaW5hdGUiLAogICAgICAgICJkaW1lbnNpb24iOiAibW9kZWxfY29uc2lzdGVuY3kiLAogICAgICAgICJldmlkZW5jZV9zdGF0ZSI6ICJub19ldmlkZW5jZSIsCiAgICAgICAgImV4Y2x1ZGVkX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJpbnRlcnByZXRhdGlvbl9jYXRlZ29yeSI6ICJtb2RlbF9pbmNvbnNpc3RlbmN5IiwKICAgICAgICAicmVhc29uX2NvZGVzIjogWwogICAgICAgICAgIm9wdGlvbmFsX3NvdXJjZV9hYnNlbnQiCiAgICAgICAgXSwKICAgICAgICAic291cmNlX2FydGlmYWN0X2lkcyI6IFtdLAogICAgICAgICJzb3VyY2VfZXZpZGVuY2VfaWRzIjogW10sCiAgICAgICAgInN0YXR1cyI6ICJpbmRldGVybWluYXRlIgogICAgICB9LAogICAgICB7CiAgICAgICAgImNhdXNhbF9zdGF0dXMiOiAiaW5kZXRlcm1pbmF0ZSIsCiAgICAgICAgImRpbWVuc2lvbiI6ICJvYnNlcnZhYmlsaXR5IiwKICAgICAgICAiZXZpZGVuY2Vfc3RhdGUiOiAibm9fZXZpZGVuY2UiLAogICAgICAgICJleGNsdWRlZF9ldmlkZW5jZV9pZHMiOiBbXSwKICAgICAgICAiaW50ZXJwcmV0YXRpb25fY2F0ZWdvcnkiOiAibW9kZWxfaW5jb25zaXN0ZW5jeSIsCiAgICAgICAgInJlYXNvbl9jb2RlcyI6IFsKICAgICAgICAgICJvcHRpb25hbF9zb3VyY2VfYWJzZW50IgogICAgICAgIF0sCiAgICAgICAgInNvdXJjZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAic291cmNlX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJzdGF0dXMiOiAiaW5kZXRlcm1pbmF0ZSIKICAgICAgfSwKICAgICAgewogICAgICAgICJjYXVzYWxfc3RhdHVzIjogImluZGV0ZXJtaW5hdGUiLAogICAgICAgICJkaW1lbnNpb24iOiAidW5jZXJ0YWludHlfaGVhbHRoIiwKICAgICAgICAiZXZpZGVuY2Vfc3RhdGUiOiAibm9fZXZpZGVuY2UiLAogICAgICAgICJleGNsdWRlZF9ldmlkZW5jZV9pZHMiOiBbXSwKICAgICAgICAiaW50ZXJwcmV0YXRpb25fY2F0ZWdvcnkiOiAibW9kZWxfaW5jb25zaXN0ZW5jeSIsCiAgICAgICAgInJlYXNvbl9jb2RlcyI6IFsKICAgICAgICAgICJvcHRpb25hbF9zb3VyY2VfYWJzZW50IgogICAgICAgIF0sCiAgICAgICAgInNvdXJjZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAic291cmNlX2V2aWRlbmNlX2lkcyI6IFtdLAogICAgICAgICJzdGF0dXMiOiAiaW5kZXRlcm1pbmF0ZSIKICAgICAgfSwKICAgICAgewogICAgICAgICJjYXVzYWxfc3RhdHVzIjogImluZGV0ZXJtaW5hdGUiLAogICAgICAgICJkaW1lbnNpb24iOiAiZGF0YV9xdWFsaXR5IiwKICAgICAgICAiZXZpZGVuY2Vfc3RhdGUiOiAicG9vcl9kYXRhX3F1YWxpdHkiLAogICAgICAgICJleGNsdWRlZF9ldmlkZW5jZV9pZHMiOiBbXSwKICAgICAgICAiaW50ZXJwcmV0YXRpb25fY2F0ZWdvcnkiOiAib2JzZXJ2ZWRfYmVoYXZpb3IiLAogICAgICAgICJyZWFzb25fY29kZXMiOiBbCiAgICAgICAgICAicXVhbGl0eV9nYXRlX2ZhaWxlZCIKICAgICAgICBdLAogICAgICAgICJzb3VyY2VfYXJ0aWZhY3RfaWRzIjogW10sCiAgICAgICAgInNvdXJjZV9ldmlkZW5jZV9pZHMiOiBbCiAgICAgICAgICAic2lnbmFsLnNhbXBsaW5nLmR1cGxpY2F0ZV90aW1lc3RhbXBzIiwKICAgICAgICAgICJzaWduYWwuc2FtcGxpbmcuZmluaXRlX3NhbXBsZV9jb3VudCIsCiAgICAgICAgICAic2lnbmFsLnNhbXBsaW5nLmludGVycG9sYXRpb25fZ2FwX2V4Y2VlZGVkIiwKICAgICAgICAgICJzaWduYWwuc2FtcGxpbmcuaW50ZXJ2YWxfY3YiLAogICAgICAgICAgInNpZ25hbC5zYW1wbGluZy5taXNzaW5nX2ZyYWN0aW9uIiwKICAgICAgICAgICJzaWduYWwuc2FtcGxpbmcubm9uX21vbm90b25pY190aW1lc3RhbXBzIgogICAgICAgIF0sCiAgICAgICAgInN0YXR1cyI6ICJkYXRhX3F1YWxpdHlfaW5zdWZmaWNpZW50IgogICAgICB9CiAgICBdLAogICAgImV2aWRlbmNlX2J1bmRsZSI6IHsKICAgICAgImNoYW5uZWxfc2NvcGUiOiAiVW5zcGVjaWZpZWQiLAogICAgICAiZXhwZXJpbWVudF9zY29wZSI6ICJVbmtub3duIiwKICAgICAgImluZGVwZW5kZW5jZV9hc3Nlc3NtZW50cyI6IFtdLAogICAgICAibGluZWFnZV9jYXRhbG9nIjogewogICAgICAgICJhcnRpZmFjdHMiOiB7fSwKICAgICAgICAic2NoZW1hX3ZlcnNpb24iOiAxCiAgICAgIH0sCiAgICAgICJyZWNvcmRzIjogWwogICAgICAgIHsKICAgICAgICAgICJhdmFpbGFiaWxpdHkiOiAiQXZhaWxhYmxlIiwKICAgICAgICAgICJkaXJlY3Rpb24iOiAiTmV1dHJhbCIsCiAgICAgICAgICAiZXZpZGVuY2VfaWQiOiAic2lnbmFsLnNhbXBsaW5nLmR1cGxpY2F0ZV90aW1lc3RhbXBzIiwKICAgICAgICAgICJleHBlcmltZW50X3Njb3BlIjogIlVua25vd24iLAogICAgICAgICAgImxpbmVhZ2VfYXJ0aWZhY3RfaWRzIjogW10sCiAgICAgICAgICAicXVhbnRpdHkiOiB7CiAgICAgICAgICAgICJ1bmNlcnRhaW50eSI6IG51bGwsCiAgICAgICAgICAgICJ1bml0IjogIjEiLAogICAgICAgICAgICAidmFsdWUiOiAwLjAKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlIjogewogICAgICAgICAgICAiYXJ0aWZhY3QiOiB7CiAgICAgICAgICAgICAgIkxlZ2FjeVVua25vd24iOiB7CiAgICAgICAgICAgICAgICAiYXJ0aWZhY3Rfa2luZCI6ICJzaWduYWxfYW5hbHlzaXMiLAogICAgICAgICAgICAgICAgInNvdXJjZV9maW5nZXJwcmludCI6ICI3NDE5YjY5OWFkMjdhN2Y2NDY1ODE4ZGMyYzc5MGU4OTA5ZWJjZDM2NWIyNjlkZTEwNzUxMTgyYzU2NzI0MWZlIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgfSwKICAgICAgICAgICAgImZpZWxkX3BhdGgiOiAiJC5zYW1wbGluZy5kdXBsaWNhdGVfdGltZXN0YW1wcyIKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlX2NsYXNzIjogIk9ic2VydmVkIiwKICAgICAgICAgICJzdHJlbmd0aCI6ICJOb3RBc3Nlc3NlZCIsCiAgICAgICAgICAic3RyZW5ndGhfZGVyaXZhdGlvbiI6IG51bGwsCiAgICAgICAgICAic3RyZW5ndGhfc291cmNlIjogIk5vdEFzc2Vzc2VkIiwKICAgICAgICAgICJ0YXJnZXQiOiB7CiAgICAgICAgICAgICJIZWFsdGhEaW1lbnNpb24iOiAiZGF0YV9xdWFsaXR5IgogICAgICAgICAgfSwKICAgICAgICAgICJ0aHJlc2hvbGRfcHJvdmVuYW5jZSI6IFtdLAogICAgICAgICAgInZhbGlkaXR5IjogIlZhbGlkIiwKICAgICAgICAgICJ3YXJuaW5ncyI6IFtdCiAgICAgICAgfSwKICAgICAgICB7CiAgICAgICAgICAiYXZhaWxhYmlsaXR5IjogIkF2YWlsYWJsZSIsCiAgICAgICAgICAiZGlyZWN0aW9uIjogIk5ldXRyYWwiLAogICAgICAgICAgImV2aWRlbmNlX2lkIjogInNpZ25hbC5zYW1wbGluZy5maW5pdGVfc2FtcGxlX2NvdW50IiwKICAgICAgICAgICJleHBlcmltZW50X3Njb3BlIjogIlVua25vd24iLAogICAgICAgICAgImxpbmVhZ2VfYXJ0aWZhY3RfaWRzIjogW10sCiAgICAgICAgICAicXVhbnRpdHkiOiB7CiAgICAgICAgICAgICJ1bmNlcnRhaW50eSI6IG51bGwsCiAgICAgICAgICAgICJ1bml0IjogIjEiLAogICAgICAgICAgICAidmFsdWUiOiAzMzAuMAogICAgICAgICAgfSwKICAgICAgICAgICJzb3VyY2UiOiB7CiAgICAgICAgICAgICJhcnRpZmFjdCI6IHsKICAgICAgICAgICAgICAiTGVnYWN5VW5rbm93biI6IHsKICAgICAgICAgICAgICAgICJhcnRpZmFjdF9raW5kIjogInNpZ25hbF9hbmFseXNpcyIsCiAgICAgICAgICAgICAgICAic291cmNlX2ZpbmdlcnByaW50IjogIjc0MTliNjk5YWQyN2E3ZjY0NjU4MThkYzJjNzkwZTg5MDllYmNkMzY1YjI2OWRlMTA3NTExODJjNTY3MjQxZmUiCiAgICAgICAgICAgICAgfQogICAgICAgICAgICB9LAogICAgICAgICAgICAiZmllbGRfcGF0aCI6ICIkLnNhbXBsaW5nLmZpbml0ZV9zYW1wbGVfY291bnQiCiAgICAgICAgICB9LAogICAgICAgICAgInNvdXJjZV9jbGFzcyI6ICJPYnNlcnZlZCIsCiAgICAgICAgICAic3RyZW5ndGgiOiAiTm90QXNzZXNzZWQiLAogICAgICAgICAgInN0cmVuZ3RoX2Rlcml2YXRpb24iOiBudWxsLAogICAgICAgICAgInN0cmVuZ3RoX3NvdXJjZSI6ICJOb3RBc3Nlc3NlZCIsCiAgICAgICAgICAidGFyZ2V0IjogewogICAgICAgICAgICAiSGVhbHRoRGltZW5zaW9uIjogImRhdGFfcXVhbGl0eSIKICAgICAgICAgIH0sCiAgICAgICAgICAidGhyZXNob2xkX3Byb3ZlbmFuY2UiOiBbXSwKICAgICAgICAgICJ2YWxpZGl0eSI6ICJWYWxpZCIsCiAgICAgICAgICAid2FybmluZ3MiOiBbXQogICAgICAgIH0sCiAgICAgICAgewogICAgICAgICAgImF2YWlsYWJpbGl0eSI6ICJBdmFpbGFibGUiLAogICAgICAgICAgImRpcmVjdGlvbiI6ICJOZXV0cmFsIiwKICAgICAgICAgICJldmlkZW5jZV9pZCI6ICJzaWduYWwuc2FtcGxpbmcuaW50ZXJwb2xhdGlvbl9nYXBfZXhjZWVkZWQiLAogICAgICAgICAgImV4cGVyaW1lbnRfc2NvcGUiOiAiVW5rbm93biIsCiAgICAgICAgICAibGluZWFnZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAgICJxdWFudGl0eSI6IHsKICAgICAgICAgICAgInVuY2VydGFpbnR5IjogbnVsbCwKICAgICAgICAgICAgInVuaXQiOiAiMSIsCiAgICAgICAgICAgICJ2YWx1ZSI6IDAuMAogICAgICAgICAgfSwKICAgICAgICAgICJzb3VyY2UiOiB7CiAgICAgICAgICAgICJhcnRpZmFjdCI6IHsKICAgICAgICAgICAgICAiTGVnYWN5VW5rbm93biI6IHsKICAgICAgICAgICAgICAgICJhcnRpZmFjdF9raW5kIjogInNpZ25hbF9hbmFseXNpcyIsCiAgICAgICAgICAgICAgICAic291cmNlX2ZpbmdlcnByaW50IjogIjc0MTliNjk5YWQyN2E3ZjY0NjU4MThkYzJjNzkwZTg5MDllYmNkMzY1YjI2OWRlMTA3NTExODJjNTY3MjQxZmUiCiAgICAgICAgICAgICAgfQogICAgICAgICAgICB9LAogICAgICAgICAgICAiZmllbGRfcGF0aCI6ICIkLnNhbXBsaW5nLmludGVycG9sYXRpb25fZ2FwX2V4Y2VlZGVkIgogICAgICAgICAgfSwKICAgICAgICAgICJzb3VyY2VfY2xhc3MiOiAiT2JzZXJ2ZWQiLAogICAgICAgICAgInN0cmVuZ3RoIjogIk5vdEFzc2Vzc2VkIiwKICAgICAgICAgICJzdHJlbmd0aF9kZXJpdmF0aW9uIjogbnVsbCwKICAgICAgICAgICJzdHJlbmd0aF9zb3VyY2UiOiAiTm90QXNzZXNzZWQiLAogICAgICAgICAgInRhcmdldCI6IHsKICAgICAgICAgICAgIkhlYWx0aERpbWVuc2lvbiI6ICJkYXRhX3F1YWxpdHkiCiAgICAgICAgICB9LAogICAgICAgICAgInRocmVzaG9sZF9wcm92ZW5hbmNlIjogW10sCiAgICAgICAgICAidmFsaWRpdHkiOiAiVmFsaWQiLAogICAgICAgICAgIndhcm5pbmdzIjogW10KICAgICAgICB9LAogICAgICAgIHsKICAgICAgICAgICJhdmFpbGFiaWxpdHkiOiAiQXZhaWxhYmxlIiwKICAgICAgICAgICJkaXJlY3Rpb24iOiAiTmV1dHJhbCIsCiAgICAgICAgICAiZXZpZGVuY2VfaWQiOiAic2lnbmFsLnNhbXBsaW5nLmludGVydmFsX2N2IiwKICAgICAgICAgICJleHBlcmltZW50X3Njb3BlIjogIlVua25vd24iLAogICAgICAgICAgImxpbmVhZ2VfYXJ0aWZhY3RfaWRzIjogW10sCiAgICAgICAgICAicXVhbnRpdHkiOiB7CiAgICAgICAgICAgICJ1bmNlcnRhaW50eSI6IG51bGwsCiAgICAgICAgICAgICJ1bml0IjogIjEiLAogICAgICAgICAgICAidmFsdWUiOiAwLjA1NDk2NDcxODYyNzkzMDcyCiAgICAgICAgICB9LAogICAgICAgICAgInNvdXJjZSI6IHsKICAgICAgICAgICAgImFydGlmYWN0IjogewogICAgICAgICAgICAgICJMZWdhY3lVbmtub3duIjogewogICAgICAgICAgICAgICAgImFydGlmYWN0X2tpbmQiOiAic2lnbmFsX2FuYWx5c2lzIiwKICAgICAgICAgICAgICAgICJzb3VyY2VfZmluZ2VycHJpbnQiOiAiNzQxOWI2OTlhZDI3YTdmNjQ2NTgxOGRjMmM3OTBlODkwOWViY2QzNjViMjY5ZGUxMDc1MTE4MmM1NjcyNDFmZSIKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIH0sCiAgICAgICAgICAgICJmaWVsZF9wYXRoIjogIiQuc2FtcGxpbmcuaW50ZXJ2YWxfY3YiCiAgICAgICAgICB9LAogICAgICAgICAgInNvdXJjZV9jbGFzcyI6ICJPYnNlcnZlZCIsCiAgICAgICAgICAic3RyZW5ndGgiOiAiTm90QXNzZXNzZWQiLAogICAgICAgICAgInN0cmVuZ3RoX2Rlcml2YXRpb24iOiBudWxsLAogICAgICAgICAgInN0cmVuZ3RoX3NvdXJjZSI6ICJOb3RBc3Nlc3NlZCIsCiAgICAgICAgICAidGFyZ2V0IjogewogICAgICAgICAgICAiSGVhbHRoRGltZW5zaW9uIjogImRhdGFfcXVhbGl0eSIKICAgICAgICAgIH0sCiAgICAgICAgICAidGhyZXNob2xkX3Byb3ZlbmFuY2UiOiBbXSwKICAgICAgICAgICJ2YWxpZGl0eSI6ICJWYWxpZCIsCiAgICAgICAgICAid2FybmluZ3MiOiBbXQogICAgICAgIH0sCiAgICAgICAgewogICAgICAgICAgImF2YWlsYWJpbGl0eSI6ICJBdmFpbGFibGUiLAogICAgICAgICAgImRpcmVjdGlvbiI6ICJOZXV0cmFsIiwKICAgICAgICAgICJldmlkZW5jZV9pZCI6ICJzaWduYWwuc2FtcGxpbmcubWlzc2luZ19mcmFjdGlvbiIsCiAgICAgICAgICAiZXhwZXJpbWVudF9zY29wZSI6ICJVbmtub3duIiwKICAgICAgICAgICJsaW5lYWdlX2FydGlmYWN0X2lkcyI6IFtdLAogICAgICAgICAgInF1YW50aXR5IjogewogICAgICAgICAgICAidW5jZXJ0YWludHkiOiBudWxsLAogICAgICAgICAgICAidW5pdCI6ICIxIiwKICAgICAgICAgICAgInZhbHVlIjogMC4wCiAgICAgICAgICB9LAogICAgICAgICAgInNvdXJjZSI6IHsKICAgICAgICAgICAgImFydGlmYWN0IjogewogICAgICAgICAgICAgICJMZWdhY3lVbmtub3duIjogewogICAgICAgICAgICAgICAgImFydGlmYWN0X2tpbmQiOiAic2lnbmFsX2FuYWx5c2lzIiwKICAgICAgICAgICAgICAgICJzb3VyY2VfZmluZ2VycHJpbnQiOiAiNzQxOWI2OTlhZDI3YTdmNjQ2NTgxOGRjMmM3OTBlODkwOWViY2QzNjViMjY5ZGUxMDc1MTE4MmM1NjcyNDFmZSIKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIH0sCiAgICAgICAgICAgICJmaWVsZF9wYXRoIjogIiQuc2FtcGxpbmcubWlzc2luZ19mcmFjdGlvbiIKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlX2NsYXNzIjogIk9ic2VydmVkIiwKICAgICAgICAgICJzdHJlbmd0aCI6ICJOb3RBc3Nlc3NlZCIsCiAgICAgICAgICAic3RyZW5ndGhfZGVyaXZhdGlvbiI6IG51bGwsCiAgICAgICAgICAic3RyZW5ndGhfc291cmNlIjogIk5vdEFzc2Vzc2VkIiwKICAgICAgICAgICJ0YXJnZXQiOiB7CiAgICAgICAgICAgICJIZWFsdGhEaW1lbnNpb24iOiAiZGF0YV9xdWFsaXR5IgogICAgICAgICAgfSwKICAgICAgICAgICJ0aHJlc2hvbGRfcHJvdmVuYW5jZSI6IFtdLAogICAgICAgICAgInZhbGlkaXR5IjogIlZhbGlkIiwKICAgICAgICAgICJ3YXJuaW5ncyI6IFtdCiAgICAgICAgfSwKICAgICAgICB7CiAgICAgICAgICAiYXZhaWxhYmlsaXR5IjogIkF2YWlsYWJsZSIsCiAgICAgICAgICAiZGlyZWN0aW9uIjogIk5ldXRyYWwiLAogICAgICAgICAgImV2aWRlbmNlX2lkIjogInNpZ25hbC5zYW1wbGluZy5ub25fbW9ub3RvbmljX3RpbWVzdGFtcHMiLAogICAgICAgICAgImV4cGVyaW1lbnRfc2NvcGUiOiAiVW5rbm93biIsCiAgICAgICAgICAibGluZWFnZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAgICJxdWFudGl0eSI6IHsKICAgICAgICAgICAgInVuY2VydGFpbnR5IjogbnVsbCwKICAgICAgICAgICAgInVuaXQiOiAiMSIsCiAgICAgICAgICAgICJ2YWx1ZSI6IDAuMAogICAgICAgICAgfSwKICAgICAgICAgICJzb3VyY2UiOiB7CiAgICAgICAgICAgICJhcnRpZmFjdCI6IHsKICAgICAgICAgICAgICAiTGVnYWN5VW5rbm93biI6IHsKICAgICAgICAgICAgICAgICJhcnRpZmFjdF9raW5kIjogInNpZ25hbF9hbmFseXNpcyIsCiAgICAgICAgICAgICAgICAic291cmNlX2ZpbmdlcnByaW50IjogIjc0MTliNjk5YWQyN2E3ZjY0NjU4MThkYzJjNzkwZTg5MDllYmNkMzY1YjI2OWRlMTA3NTExODJjNTY3MjQxZmUiCiAgICAgICAgICAgICAgfQogICAgICAgICAgICB9LAogICAgICAgICAgICAiZmllbGRfcGF0aCI6ICIkLnNhbXBsaW5nLm5vbl9tb25vdG9uaWNfdGltZXN0YW1wcyIKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlX2NsYXNzIjogIk9ic2VydmVkIiwKICAgICAgICAgICJzdHJlbmd0aCI6ICJOb3RBc3Nlc3NlZCIsCiAgICAgICAgICAic3RyZW5ndGhfZGVyaXZhdGlvbiI6IG51bGwsCiAgICAgICAgICAic3RyZW5ndGhfc291cmNlIjogIk5vdEFzc2Vzc2VkIiwKICAgICAgICAgICJ0YXJnZXQiOiB7CiAgICAgICAgICAgICJIZWFsdGhEaW1lbnNpb24iOiAiZGF0YV9xdWFsaXR5IgogICAgICAgICAgfSwKICAgICAgICAgICJ0aHJlc2hvbGRfcHJvdmVuYW5jZSI6IFtdLAogICAgICAgICAgInZhbGlkaXR5IjogIlZhbGlkIiwKICAgICAgICAgICJ3YXJuaW5ncyI6IFtdCiAgICAgICAgfSwKICAgICAgICB7CiAgICAgICAgICAiYXZhaWxhYmlsaXR5IjogIkF2YWlsYWJsZSIsCiAgICAgICAgICAiZGlyZWN0aW9uIjogIk5ldXRyYWwiLAogICAgICAgICAgImV2aWRlbmNlX2lkIjogInNpZ25hbC5kZXNjcmlwdGl2ZS5ybXMiLAogICAgICAgICAgImV4cGVyaW1lbnRfc2NvcGUiOiAiVW5rbm93biIsCiAgICAgICAgICAibGluZWFnZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAgICJxdWFudGl0eSI6IHsKICAgICAgICAgICAgInVuY2VydGFpbnR5IjogbnVsbCwKICAgICAgICAgICAgInVuaXQiOiAiViIsCiAgICAgICAgICAgICJ2YWx1ZSI6IDAuMjE0NzI2MTU4MDI0OTkyNzMKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlIjogewogICAgICAgICAgICAiYXJ0aWZhY3QiOiB7CiAgICAgICAgICAgICAgIkxlZ2FjeVVua25vd24iOiB7CiAgICAgICAgICAgICAgICAiYXJ0aWZhY3Rfa2luZCI6ICJzaWduYWxfYW5hbHlzaXMiLAogICAgICAgICAgICAgICAgInNvdXJjZV9maW5nZXJwcmludCI6ICI3NDE5YjY5OWFkMjdhN2Y2NDY1ODE4ZGMyYzc5MGU4OTA5ZWJjZDM2NWIyNjlkZTEwNzUxMTgyYzU2NzI0MWZlIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgfSwKICAgICAgICAgICAgImZpZWxkX3BhdGgiOiAiJC5kZXNjcmlwdGl2ZS5ybXMiCiAgICAgICAgICB9LAogICAgICAgICAgInNvdXJjZV9jbGFzcyI6ICJPYnNlcnZlZCIsCiAgICAgICAgICAic3RyZW5ndGgiOiAiTm90QXNzZXNzZWQiLAogICAgICAgICAgInN0cmVuZ3RoX2Rlcml2YXRpb24iOiBudWxsLAogICAgICAgICAgInN0cmVuZ3RoX3NvdXJjZSI6ICJOb3RBc3Nlc3NlZCIsCiAgICAgICAgICAidGFyZ2V0IjogewogICAgICAgICAgICAiSGVhbHRoRGltZW5zaW9uIjogInNpZ25hbF9pbnRlZ3JpdHkiCiAgICAgICAgICB9LAogICAgICAgICAgInRocmVzaG9sZF9wcm92ZW5hbmNlIjogWwogICAgICAgICAgICB7CiAgICAgICAgICAgICAgImNvbmZpZ3VyYXRpb25faGFzaCI6ICI5NDY5MDFkMzZmYzc0Mjk1MmM2ZTAzZjA2OGIwOGMzNTQ3ZDFiMzI4ZjAzMWZiNjA2Y2ZhNzQwOTNlMGJlOGE0IiwKICAgICAgICAgICAgICAic291cmNlIjogIlVzZXJDb25maWd1cmF0aW9uIiwKICAgICAgICAgICAgICAidGhyZXNob2xkX2lkIjogInNpZ25hbC5kZXNjcmlwdGl2ZS5ybXMudGhyZXNob2xkLmNyaXRpY2FsIiwKICAgICAgICAgICAgICAidW5pdCI6ICJWIiwKICAgICAgICAgICAgICAidmFsdWUiOiAwLjAwNQogICAgICAgICAgICB9LAogICAgICAgICAgICB7CiAgICAgICAgICAgICAgImNvbmZpZ3VyYXRpb25faGFzaCI6ICI5NDY5MDFkMzZmYzc0Mjk1MmM2ZTAzZjA2OGIwOGMzNTQ3ZDFiMzI4ZjAzMWZiNjA2Y2ZhNzQwOTNlMGJlOGE0IiwKICAgICAgICAgICAgICAic291cmNlIjogIlVzZXJDb25maWd1cmF0aW9uIiwKICAgICAgICAgICAgICAidGhyZXNob2xkX2lkIjogInNpZ25hbC5kZXNjcmlwdGl2ZS5ybXMudGhyZXNob2xkLmRlZ3JhZGVkIiwKICAgICAgICAgICAgICAidW5pdCI6ICJWIiwKICAgICAgICAgICAgICAidmFsdWUiOiAwLjAwMgogICAgICAgICAgICB9LAogICAgICAgICAgICB7CiAgICAgICAgICAgICAgImNvbmZpZ3VyYXRpb25faGFzaCI6ICI5NDY5MDFkMzZmYzc0Mjk1MmM2ZTAzZjA2OGIwOGMzNTQ3ZDFiMzI4ZjAzMWZiNjA2Y2ZhNzQwOTNlMGJlOGE0IiwKICAgICAgICAgICAgICAic291cmNlIjogIlVzZXJDb25maWd1cmF0aW9uIiwKICAgICAgICAgICAgICAidGhyZXNob2xkX2lkIjogInNpZ25hbC5kZXNjcmlwdGl2ZS5ybXMudGhyZXNob2xkLndhdGNoIiwKICAgICAgICAgICAgICAidW5pdCI6ICJWIiwKICAgICAgICAgICAgICAidmFsdWUiOiAwLjAwMQogICAgICAgICAgICB9CiAgICAgICAgICBdLAogICAgICAgICAgInZhbGlkaXR5IjogIlZhbGlkIiwKICAgICAgICAgICJ3YXJuaW5ncyI6IFtdCiAgICAgICAgfSwKICAgICAgICB7CiAgICAgICAgICAiYXZhaWxhYmlsaXR5IjogIkF2YWlsYWJsZSIsCiAgICAgICAgICAiZGlyZWN0aW9uIjogIk5ldXRyYWwiLAogICAgICAgICAgImV2aWRlbmNlX2lkIjogInNpZ25hbC5kZXNjcmlwdGl2ZS5yb2J1c3Rfc3RhbmRhcmRfZGV2aWF0aW9uIiwKICAgICAgICAgICJleHBlcmltZW50X3Njb3BlIjogIlVua25vd24iLAogICAgICAgICAgImxpbmVhZ2VfYXJ0aWZhY3RfaWRzIjogW10sCiAgICAgICAgICAicXVhbnRpdHkiOiB7CiAgICAgICAgICAgICJ1bmNlcnRhaW50eSI6IG51bGwsCiAgICAgICAgICAgICJ1bml0IjogIlYiLAogICAgICAgICAgICAidmFsdWUiOiAxLjg1MTM2MjQ0ODAyMzg5NjRlLTYKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlIjogewogICAgICAgICAgICAiYXJ0aWZhY3QiOiB7CiAgICAgICAgICAgICAgIkxlZ2FjeVVua25vd24iOiB7CiAgICAgICAgICAgICAgICAiYXJ0aWZhY3Rfa2luZCI6ICJzaWduYWxfYW5hbHlzaXMiLAogICAgICAgICAgICAgICAgInNvdXJjZV9maW5nZXJwcmludCI6ICI3NDE5YjY5OWFkMjdhN2Y2NDY1ODE4ZGMyYzc5MGU4OTA5ZWJjZDM2NWIyNjlkZTEwNzUxMTgyYzU2NzI0MWZlIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgfSwKICAgICAgICAgICAgImZpZWxkX3BhdGgiOiAiJC5kZXNjcmlwdGl2ZS5yb2J1c3Rfc3RhbmRhcmRfZGV2aWF0aW9uIgogICAgICAgICAgfSwKICAgICAgICAgICJzb3VyY2VfY2xhc3MiOiAiT2JzZXJ2ZWQiLAogICAgICAgICAgInN0cmVuZ3RoIjogIk5vdEFzc2Vzc2VkIiwKICAgICAgICAgICJzdHJlbmd0aF9kZXJpdmF0aW9uIjogbnVsbCwKICAgICAgICAgICJzdHJlbmd0aF9zb3VyY2UiOiAiTm90QXNzZXNzZWQiLAogICAgICAgICAgInRhcmdldCI6IHsKICAgICAgICAgICAgIkhlYWx0aERpbWVuc2lvbiI6ICJzaWduYWxfaW50ZWdyaXR5IgogICAgICAgICAgfSwKICAgICAgICAgICJ0aHJlc2hvbGRfcHJvdmVuYW5jZSI6IFsKICAgICAgICAgICAgewogICAgICAgICAgICAgICJjb25maWd1cmF0aW9uX2hhc2giOiAiOTQ2OTAxZDM2ZmM3NDI5NTJjNmUwM2YwNjhiMDhjMzU0N2QxYjMyOGYwMzFmYjYwNmNmYTc0MDkzZTBiZThhNCIsCiAgICAgICAgICAgICAgInNvdXJjZSI6ICJVc2VyQ29uZmlndXJhdGlvbiIsCiAgICAgICAgICAgICAgInRocmVzaG9sZF9pZCI6ICJzaWduYWwuZGVzY3JpcHRpdmUucm9idXN0X3N0YW5kYXJkX2RldmlhdGlvbi50aHJlc2hvbGQuY3JpdGljYWwiLAogICAgICAgICAgICAgICJ1bml0IjogIlYiLAogICAgICAgICAgICAgICJ2YWx1ZSI6IDAuMDA1CiAgICAgICAgICAgIH0sCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAiY29uZmlndXJhdGlvbl9oYXNoIjogIjk0NjkwMWQzNmZjNzQyOTUyYzZlMDNmMDY4YjA4YzM1NDdkMWIzMjhmMDMxZmI2MDZjZmE3NDA5M2UwYmU4YTQiLAogICAgICAgICAgICAgICJzb3VyY2UiOiAiVXNlckNvbmZpZ3VyYXRpb24iLAogICAgICAgICAgICAgICJ0aHJlc2hvbGRfaWQiOiAic2lnbmFsLmRlc2NyaXB0aXZlLnJvYnVzdF9zdGFuZGFyZF9kZXZpYXRpb24udGhyZXNob2xkLmRlZ3JhZGVkIiwKICAgICAgICAgICAgICAidW5pdCI6ICJWIiwKICAgICAgICAgICAgICAidmFsdWUiOiAwLjAwMgogICAgICAgICAgICB9LAogICAgICAgICAgICB7CiAgICAgICAgICAgICAgImNvbmZpZ3VyYXRpb25faGFzaCI6ICI5NDY5MDFkMzZmYzc0Mjk1MmM2ZTAzZjA2OGIwOGMzNTQ3ZDFiMzI4ZjAzMWZiNjA2Y2ZhNzQwOTNlMGJlOGE0IiwKICAgICAgICAgICAgICAic291cmNlIjogIlVzZXJDb25maWd1cmF0aW9uIiwKICAgICAgICAgICAgICAidGhyZXNob2xkX2lkIjogInNpZ25hbC5kZXNjcmlwdGl2ZS5yb2J1c3Rfc3RhbmRhcmRfZGV2aWF0aW9uLnRocmVzaG9sZC53YXRjaCIsCiAgICAgICAgICAgICAgInVuaXQiOiAiViIsCiAgICAgICAgICAgICAgInZhbHVlIjogMC4wMDEKICAgICAgICAgICAgfQogICAgICAgICAgXSwKICAgICAgICAgICJ2YWxpZGl0eSI6ICJWYWxpZCIsCiAgICAgICAgICAid2FybmluZ3MiOiBbXQogICAgICAgIH0sCiAgICAgICAgewogICAgICAgICAgImF2YWlsYWJpbGl0eSI6ICJBdmFpbGFibGUiLAogICAgICAgICAgImRpcmVjdGlvbiI6ICJOZXV0cmFsIiwKICAgICAgICAgICJldmlkZW5jZV9pZCI6ICJzaWduYWwuZHJpZnQudGhlaWxfc2VuLnNsb3BlX3ZfcGVyX3MiLAogICAgICAgICAgImV4cGVyaW1lbnRfc2NvcGUiOiAiVW5rbm93biIsCiAgICAgICAgICAibGluZWFnZV9hcnRpZmFjdF9pZHMiOiBbXSwKICAgICAgICAgICJxdWFudGl0eSI6IHsKICAgICAgICAgICAgInVuY2VydGFpbnR5IjogbnVsbCwKICAgICAgICAgICAgInVuaXQiOiAiVi9zIiwKICAgICAgICAgICAgInZhbHVlIjogLTMuMzc0ODg1NjExMTI3NDkxN2UtNgogICAgICAgICAgfSwKICAgICAgICAgICJzb3VyY2UiOiB7CiAgICAgICAgICAgICJhcnRpZmFjdCI6IHsKICAgICAgICAgICAgICAiTGVnYWN5VW5rbm93biI6IHsKICAgICAgICAgICAgICAgICJhcnRpZmFjdF9raW5kIjogInNpZ25hbF9hbmFseXNpcyIsCiAgICAgICAgICAgICAgICAic291cmNlX2ZpbmdlcnByaW50IjogIjc0MTliNjk5YWQyN2E3ZjY0NjU4MThkYzJjNzkwZTg5MDllYmNkMzY1YjI2OWRlMTA3NTExODJjNTY3MjQxZmUiCiAgICAgICAgICAgICAgfQogICAgICAgICAgICB9LAogICAgICAgICAgICAiZmllbGRfcGF0aCI6ICIkLmRyaWZ0W3RoZWlsX3Nlbl0uc2xvcGVfdl9wZXJfcyIKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlX2NsYXNzIjogIk9ic2VydmVkIiwKICAgICAgICAgICJzdHJlbmd0aCI6ICJOb3RBc3Nlc3NlZCIsCiAgICAgICAgICAic3RyZW5ndGhfZGVyaXZhdGlvbiI6IG51bGwsCiAgICAgICAgICAic3RyZW5ndGhfc291cmNlIjogIk5vdEFzc2Vzc2VkIiwKICAgICAgICAgICJ0YXJnZXQiOiB7CiAgICAgICAgICAgICJIZWFsdGhEaW1lbnNpb24iOiAic2lnbmFsX2ludGVncml0eSIKICAgICAgICAgIH0sCiAgICAgICAgICAidGhyZXNob2xkX3Byb3ZlbmFuY2UiOiBbCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAiY29uZmlndXJhdGlvbl9oYXNoIjogIjk0NjkwMWQzNmZjNzQyOTUyYzZlMDNmMDY4YjA4YzM1NDdkMWIzMjhmMDMxZmI2MDZjZmE3NDA5M2UwYmU4YTQiLAogICAgICAgICAgICAgICJzb3VyY2UiOiAiVXNlckNvbmZpZ3VyYXRpb24iLAogICAgICAgICAgICAgICJ0aHJlc2hvbGRfaWQiOiAic2lnbmFsLmRyaWZ0LnRoZWlsX3Nlbi5zbG9wZV92X3Blcl9zLnRocmVzaG9sZC5jcml0aWNhbCIsCiAgICAgICAgICAgICAgInVuaXQiOiAiVi9zIiwKICAgICAgICAgICAgICAidmFsdWUiOiAwLjAxCiAgICAgICAgICAgIH0sCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAiY29uZmlndXJhdGlvbl9oYXNoIjogIjk0NjkwMWQzNmZjNzQyOTUyYzZlMDNmMDY4YjA4YzM1NDdkMWIzMjhmMDMxZmI2MDZjZmE3NDA5M2UwYmU4YTQiLAogICAgICAgICAgICAgICJzb3VyY2UiOiAiVXNlckNvbmZpZ3VyYXRpb24iLAogICAgICAgICAgICAgICJ0aHJlc2hvbGRfaWQiOiAic2lnbmFsLmRyaWZ0LnRoZWlsX3Nlbi5zbG9wZV92X3Blcl9zLnRocmVzaG9sZC5kZWdyYWRlZCIsCiAgICAgICAgICAgICAgInVuaXQiOiAiVi9zIiwKICAgICAgICAgICAgICAidmFsdWUiOiAwLjAwMQogICAgICAgICAgICB9LAogICAgICAgICAgICB7CiAgICAgICAgICAgICAgImNvbmZpZ3VyYXRpb25faGFzaCI6ICI5NDY5MDFkMzZmYzc0Mjk1MmM2ZTAzZjA2OGIwOGMzNTQ3ZDFiMzI4ZjAzMWZiNjA2Y2ZhNzQwOTNlMGJlOGE0IiwKICAgICAgICAgICAgICAic291cmNlIjogIlVzZXJDb25maWd1cmF0aW9uIiwKICAgICAgICAgICAgICAidGhyZXNob2xkX2lkIjogInNpZ25hbC5kcmlmdC50aGVpbF9zZW4uc2xvcGVfdl9wZXJfcy50aHJlc2hvbGQud2F0Y2giLAogICAgICAgICAgICAgICJ1bml0IjogIlYvcyIsCiAgICAgICAgICAgICAgInZhbHVlIjogMC4wMDAxCiAgICAgICAgICAgIH0KICAgICAgICAgIF0sCiAgICAgICAgICAidmFsaWRpdHkiOiAiVmFsaWQiLAogICAgICAgICAgIndhcm5pbmdzIjogW10KICAgICAgICB9LAogICAgICAgIHsKICAgICAgICAgICJhdmFpbGFiaWxpdHkiOiAiQXZhaWxhYmxlIiwKICAgICAgICAgICJkaXJlY3Rpb24iOiAiTmV1dHJhbCIsCiAgICAgICAgICAiZXZpZGVuY2VfaWQiOiAic2lnbmFsLnNwaWtlcy5mbGFnZ2VkX2ZyYWN0aW9uIiwKICAgICAgICAgICJleHBlcmltZW50X3Njb3BlIjogIlVua25vd24iLAogICAgICAgICAgImxpbmVhZ2VfYXJ0aWZhY3RfaWRzIjogW10sCiAgICAgICAgICAicXVhbnRpdHkiOiB7CiAgICAgICAgICAgICJ1bmNlcnRhaW50eSI6IG51bGwsCiAgICAgICAgICAgICJ1bml0IjogIjEiLAogICAgICAgICAgICAidmFsdWUiOiAwLjAKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlIjogewogICAgICAgICAgICAiYXJ0aWZhY3QiOiB7CiAgICAgICAgICAgICAgIkxlZ2FjeVVua25vd24iOiB7CiAgICAgICAgICAgICAgICAiYXJ0aWZhY3Rfa2luZCI6ICJzaWduYWxfYW5hbHlzaXMiLAogICAgICAgICAgICAgICAgInNvdXJjZV9maW5nZXJwcmludCI6ICI3NDE5YjY5OWFkMjdhN2Y2NDY1ODE4ZGMyYzc5MGU4OTA5ZWJjZDM2NWIyNjlkZTEwNzUxMTgyYzU2NzI0MWZlIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgfSwKICAgICAgICAgICAgImZpZWxkX3BhdGgiOiAiJC5zcGlrZXMuZmxhZ2dlZF9mcmFjdGlvbiIKICAgICAgICAgIH0sCiAgICAgICAgICAic291cmNlX2NsYXNzIjogIk9ic2VydmVkIiwKICAgICAgICAgICJzdHJlbmd0aCI6ICJOb3RBc3Nlc3NlZCIsCiAgICAgICAgICAic3RyZW5ndGhfZGVyaXZhdGlvbiI6IG51bGwsCiAgICAgICAgICAic3RyZW5ndGhfc291cmNlIjogIk5vdEFzc2Vzc2VkIiwKICAgICAgICAgICJ0YXJnZXQiOiB7CiAgICAgICAgICAgICJIZWFsdGhEaW1lbnNpb24iOiAic2lnbmFsX2ludGVncml0eSIKICAgICAgICAgIH0sCiAgICAgICAgICAidGhyZXNob2xkX3Byb3ZlbmFuY2UiOiBbCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAiY29uZmlndXJhdGlvbl9oYXNoIjogIjk0NjkwMWQzNmZjNzQyOTUyYzZlMDNmMDY4YjA4YzM1NDdkMWIzMjhmMDMxZmI2MDZjZmE3NDA5M2UwYmU4YTQiLAogICAgICAgICAgICAgICJzb3VyY2UiOiAiVXNlckNvbmZpZ3VyYXRpb24iLAogICAgICAgICAgICAgICJ0aHJlc2hvbGRfaWQiOiAic2lnbmFsLnNwaWtlcy5mbGFnZ2VkX2ZyYWN0aW9uLnRocmVzaG9sZC5jcml0aWNhbCIsCiAgICAgICAgICAgICAgInVuaXQiOiAiMSIsCiAgICAgICAgICAgICAgInZhbHVlIjogMC4xCiAgICAgICAgICAgIH0sCiAgICAgICAgICAgIHsKICAgICAgICAgICAgICAiY29uZmlndXJhdGlvbl9oYXNoIjogIjk0NjkwMWQzNmZjNzQyOTUyYzZlMDNmMDY4YjA4YzM1NDdkMWIzMjhmMDMxZmI2MDZjZmE3NDA5M2UwYmU4YTQiLAogICAgICAgICAgICAgICJzb3VyY2UiOiAiVXNlckNvbmZpZ3VyYXRpb24iLAogICAgICAgICAgICAgICJ0aHJlc2hvbGRfaWQiOiAic2lnbmFsLnNwaWtlcy5mbGFnZ2VkX2ZyYWN0aW9uLnRocmVzaG9sZC5kZWdyYWRlZCIsCiAgICAgICAgICAgICAgInVuaXQiOiAiMSIsCiAgICAgICAgICAgICAgInZhbHVlIjogMC4wNQogICAgICAgICAgICB9LAogICAgICAgICAgICB7CiAgICAgICAgICAgICAgImNvbmZpZ3VyYXRpb25faGFzaCI6ICI5NDY5MDFkMzZmYzc0Mjk1MmM2ZTAzZjA2OGIwOGMzNTQ3ZDFiMzI4ZjAzMWZiNjA2Y2ZhNzQwOTNlMGJlOGE0IiwKICAgICAgICAgICAgICAic291cmNlIjogIlVzZXJDb25maWd1cmF0aW9uIiwKICAgICAgICAgICAgICAidGhyZXNob2xkX2lkIjogInNpZ25hbC5zcGlrZXMuZmxhZ2dlZF9mcmFjdGlvbi50aHJlc2hvbGQud2F0Y2giLAogICAgICAgICAgICAgICJ1bml0IjogIjEiLAogICAgICAgICAgICAgICJ2YWx1ZSI6IDAuMDEKICAgICAgICAgICAgfQogICAgICAgICAgXSwKICAgICAgICAgICJ2YWxpZGl0eSI6ICJWYWxpZCIsCiAgICAgICAgICAid2FybmluZ3MiOiBbXQogICAgICAgIH0KICAgICAgXSwKICAgICAgInNjaGVtYV92ZXJzaW9uIjogMSwKICAgICAgInNlbnNvcl9zY29wZSI6ICJVbnNwZWNpZmllZCIsCiAgICAgICJ0aW1lc2NhbGVfcGFpcl91bmNlcnRhaW50aWVzIjogW10sCiAgICAgICJ3YXJuaW5ncyI6IFtdCiAgICB9LAogICAgIm92ZXJhbGxfY2F1c2FsX3N0YXR1cyI6ICJvYnNlcnZlZCIsCiAgICAib3ZlcmFsbF9pbnRlcnByZXRhdGlvbl9jYXRlZ29yaWVzIjogWwogICAgICAib2JzZXJ2ZWRfYmVoYXZpb3IiCiAgICBdLAogICAgIm92ZXJhbGxfc3RhdHVzIjogImNyaXRpY2FsIgogIH0sCiAgInByb3ZlbmFuY2UiOiB7CiAgICAiY29uZmlndXJhdGlvbl9wYXRoIjogbnVsbCwKICAgICJjb25maWd1cmF0aW9uX3NoYTI1NiI6IG51bGwsCiAgICAiZ2VuZXJhdGlvbl90aW1lc3RhbXAiOiAxLAogICAgImdpdF9jb21taXQiOiBudWxsLAogICAgImlucHV0X3BhdGgiOiAiZml4dHVyZS1pbnB1dC5qc29uIiwKICAgICJpbnB1dF9zaGEyNTYiOiAiYTAtdGVzdCIsCiAgICAic29mdHdhcmVfdmVyc2lvbiI6ICJhMC10ZXN0IgogIH0sCiAgInJ1bGVfZXZhbHVhdGlvbnMiOiBbCiAgICB7CiAgICAgICJjb25kaXRpb25zX25vdF9zYXRpc2ZpZWQiOiBbXSwKICAgICAgImNvbmRpdGlvbnNfc2F0aXNmaWVkIjogW10sCiAgICAgICJjb25kaXRpb25zX3VuYXZhaWxhYmxlIjogWwogICAgICAgICJzaWduYWwucm9idXN0X25vaXNlX3N0YW5kYXJkX2RldmlhdGlvbiIKICAgICAgXSwKICAgICAgImNvbmZpZGVuY2UiOiAiaW5zdWZmaWNpZW50IiwKICAgICAgImNvbnRyYWRpY3RvcnlfZXZpZGVuY2UiOiBbXSwKICAgICAgImV2aWRlbmNlX2RvbWFpbnMiOiBbXSwKICAgICAgInJ1bGVfaWQiOiAiZWxldmF0ZWQtbm9pc2UiLAogICAgICAic2V2ZXJpdHkiOiAibW9kZXJhdGUiLAogICAgICAic3VwcG9ydGluZ19ldmlkZW5jZSI6IFtdLAogICAgICAidHJpZ2dlcmVkIjogZmFsc2UKICAgIH0sCiAgICB7CiAgICAgICJjb25kaXRpb25zX25vdF9zYXRpc2ZpZWQiOiBbXSwKICAgICAgImNvbmRpdGlvbnNfc2F0aXNmaWVkIjogW10sCiAgICAgICJjb25kaXRpb25zX3VuYXZhaWxhYmxlIjogWwogICAgICAgICJ0cmFuc2llbnQudGF1X3Nsb3ciLAogICAgICAgICJjYWxpYnJhdGlvbi5zbG9wZV9lZmZpY2llbmN5IiwKICAgICAgICAiZWlzLnJvbGUudHJhbnNwb3J0LnJlbGF4YXRpb25fdGltZXNjYWxlIgogICAgICBdLAogICAgICAiY29uZmlkZW5jZSI6ICJpbnN1ZmZpY2llbnQiLAogICAgICAiY29udHJhZGljdG9yeV9ldmlkZW5jZSI6IFtdLAogICAgICAiZXZpZGVuY2VfZG9tYWlucyI6IFtdLAogICAgICAicnVsZV9pZCI6ICJwcm9iYWJsZS1mb3VsaW5nIiwKICAgICAgInNldmVyaXR5IjogIm1ham9yIiwKICAgICAgInN1cHBvcnRpbmdfZXZpZGVuY2UiOiBbXSwKICAgICAgInRyaWdnZXJlZCI6IGZhbHNlCiAgICB9CiAgXSwKICAic2NoZW1hX3ZlcnNpb24iOiA0LAogICJzZW5zb3JfaWQiOiBudWxsLAogICJ3YXJuaW5ncyI6IFsKICAgICJBc3Nlc3NtZW50QmFzZWRPbldhcm5pbmdCZWFyaW5nRml0cyIsCiAgICAiTWlzc2luZ0Jhc2VsaW5lIiwKICAgICJJbnN1ZmZpY2llbnRFdmlkZW5jZURvbWFpbnMiCiAgXQp9Cg==
```

`N-F31` → `mechanism/timescale_cmp01.json` is Class 2, encoded by `N-L08`.
It is schema 4 with producer `phase-d-fixture-v1`, base scopes, family
`phase-d-fixture-family`, inherited complete provenance, semantic/artifact ID
`sha256:03487f7022a2fbb77bb85bfbd1e3c30a35aff1d1efca7d231d7b8943fd7a349e`,
and final SHA-256 `d0a373578981f8db5f69e722d484c3be32e78e2f55d563d22125b3692332aee6`.
Its complete literal records exactly the `cmp-01` comparison and timescale
values restated by `E22`; no component is generated by a future producer.

```text
N-L08 mechanism/timescale_cmp01.json
ewogICJhbmFseXNpc19pZCI6ICJtZWNoYW5pc20tcGhhc2UtYjpiLWUyZS0xIiwKICAiYXJ0aWZhY3Rfa2luZCI6ICJtZWNoYW5pc21fYW5hbHlzaXMiLAogICJjb21wYXJpc29ucyI6IFsKICAgIHsKICAgICAgImFsdGVybmF0aXZlX2V4cGxhbmF0aW9ucyI6IFtdLAogICAgICAiYXNzdW1wdGlvbnMiOiBbXSwKICAgICAgImNvbXBhcmlzb25faWQiOiAiY21wLTAxIiwKICAgICAgImNvbXBhdGliaWxpdHlfcHJvYmFiaWxpdHkiOiAwLjksCiAgICAgICJjb25maWRlbmNlX2ludGVydmFsX292ZXJsYXAiOiB0cnVlLAogICAgICAiY29udHJhZGljdG9yeV9ldmlkZW5jZSI6IFtdLAogICAgICAiZWlzX3RpbWVzY2FsZV9pZCI6ICJlaXMtdGF1IiwKICAgICAgImV2aWRlbmNlX2xldmVsIjogIm1vZGVyYXRlIiwKICAgICAgImxvZzEwX2Rpc3RhbmNlIjogMC4wNDEsCiAgICAgICJyYXRpbyI6IDEuMSwKICAgICAgInJlY29yZF9pZCI6ICJyZWMtMDEiLAogICAgICAic3VwcG9ydGluZ19ldmlkZW5jZSI6IFtdLAogICAgICAic3ltbWV0cmljX3JlbGF0aXZlX2RpZmZlcmVuY2UiOiAwLjA5NTIzODA5NTIzODA5NTIzLAogICAgICAidHJhbnNpZW50X3RpbWVzY2FsZV9pZCI6ICJ0cmFuc2llbnQtdGF1IiwKICAgICAgIndhcm5pbmdzIjogW10KICAgIH0KICBdLAogICJjb25maWd1cmF0aW9uIjogewogICAgImFsbG93X3dhcm5pbmdfZml0cyI6IHRydWUsCiAgICAiY29tcGF0aWJpbGl0eV9yYXRpb19sb3dlciI6IDAuNSwKICAgICJjb21wYXRpYmlsaXR5X3JhdGlvX3VwcGVyIjogMi4wLAogICAgImNvbmZpZGVuY2VfbGV2ZWwiOiAwLjk1LAogICAgImZyZXF1ZW5jeV9ib3VuZGFyeV9tYXJnaW4iOiAwLjEsCiAgICAiaHlwb3RoZXNlcyI6IFtdLAogICAgImxvZ19kaXN0YW5jZV9tb2RlcmF0ZSI6IDAuNSwKICAgICJsb2dfZGlzdGFuY2Vfc3Ryb25nIjogMC4xNzYxLAogICAgImxvZ19kaXN0YW5jZV93ZWFrIjogMS4wLAogICAgIm1pbmltdW1fZml0X3F1YWxpdHkiOiAwLjAsCiAgICAibWluaW11bV9yZXBsaWNhdGVzX2Zvcl9zdHJvbmciOiAzLAogICAgIm1vbnRlX2NhcmxvX3NhbXBsZXMiOiAxMDAwMCwKICAgICJyYXRpb19tb2RlcmF0ZSI6IDMuMCwKICAgICJyYXRpb19zdHJvbmciOiAxLjUsCiAgICAicmF0aW9fd2VhayI6IDEwLjAsCiAgICAicmVxdWlyZV9leHBlcmltZW50X2lkIjogdHJ1ZSwKICAgICJyZXF1aXJlX3NlbnNvcl9pZCI6IGZhbHNlLAogICAgInNjaGVtYV92ZXJzaW9uIjogMSwKICAgICJzZWVkIjogNDIsCiAgICAic2VsZWN0ZWRfbW9kZWxfb25seSI6IHRydWUsCiAgICAidHJlbmRfaW5kZXBlbmRlbnRfdmFyaWFibGUiOiAic2Vuc29yX2FnZV9kYXlzIiwKICAgICJ0cmVuZF9taW5pbXVtX3JlY29yZHMiOiAzCiAgfSwKICAiZWlzX3RpbWVzY2FsZXMiOiBbCiAgICB7CiAgICAgICJjb25maWRlbmNlX2ludGVydmFsX3MiOiBbCiAgICAgICAgOC4wLAogICAgICAgIDEyLjAKICAgICAgXSwKICAgICAgImRlcml2YXRpb24iOiB7CiAgICAgICAgImNpcmN1aXRfcGF0aCI6IG51bGwsCiAgICAgICAgImNvbnZlbnRpb24iOiBudWxsLAogICAgICAgICJlcXVhdGlvbiI6ICJzdG9yZWQiCiAgICAgIH0sCiAgICAgICJsYWJlbCI6ICJFSVMgdGF1IiwKICAgICAgInNlbWFudGljX3JvbGUiOiBudWxsLAogICAgICAic291cmNlIjogImVpc19jaXJjdWl0IiwKICAgICAgInNvdXJjZV9wYXJhbWV0ZXJzIjogW10sCiAgICAgICJzdGFuZGFyZF9lcnJvcl9zIjogMS4wLAogICAgICAidGltZXNjYWxlX2lkIjogImVpcy10YXUiLAogICAgICAidmFsaWRpdHkiOiAidmFsaWQiLAogICAgICAidmFsdWVfcyI6IDEwLjAsCiAgICAgICJ3YXJuaW5ncyI6IFtdCiAgICB9CiAgXSwKICAiaHlwb3RoZXNpc19hc3Nlc3NtZW50cyI6IFsKICAgIHsKICAgICAgImN1cnJlbnQiOiB7CiAgICAgICAgImFtcGxpdHVkZV9hc3Nlc3NtZW50cyI6IFtdLAogICAgICAgICJjb21wb25lbnRfYXNzZXNzbWVudHMiOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJhc3Nlc3NtZW50X3RhcmdldCI6ICJ2YWxpZGF0ZWRfZm9yX2RvbWFpbiIsCiAgICAgICAgICAgICJjb21wb25lbnRfaWQiOiAiYi1laXMtdGF1IiwKICAgICAgICAgICAgImV2aWRlbmNlX2lkcyI6IFsKICAgICAgICAgICAgICAiY2FsaWJyYXRpb24ub2JzZXJ2YXRpb24uMCIsCiAgICAgICAgICAgICAgImVpcy5wYXJhbWV0ZXIuMCIsCiAgICAgICAgICAgICAgImVzdGltYXRpb24ucG9pbnQuMC5zdGF0ZS4wIiwKICAgICAgICAgICAgICAidHJhbnNpZW50LmV2ZW50LjAudGF1X2Zhc3RfcyIKICAgICAgICAgICAgXSwKICAgICAgICAgICAgInByaW9yX3N0YXR1cyI6ICJoeXBvdGhlc2l6ZWQiLAogICAgICAgICAgICAicmVhc29ucyI6IFsKICAgICAgICAgICAgICAiaHlwb3RoZXNpc19ldmlkZW5jZSIKICAgICAgICAgICAgXSwKICAgICAgICAgICAgInJlc3VsdGluZ19zdGF0dXMiOiAidmFsaWRhdGVkX2Zvcl9kb21haW4iLAogICAgICAgICAgICAic3VwcG9ydGluZ19oeXBvdGhlc2lzX2lkIjogImItaHlwb3RoZXNpcyIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJhc3Nlc3NtZW50X3RhcmdldCI6ICJ2YWxpZGF0ZWRfZm9yX2RvbWFpbiIsCiAgICAgICAgICAgICJjb21wb25lbnRfaWQiOiAiYi12YWxpZGF0aW9uLWNhbGlicmF0aW9uIiwKICAgICAgICAgICAgImV2aWRlbmNlX2lkcyI6IFsKICAgICAgICAgICAgICAiY2FsaWJyYXRpb24ub2JzZXJ2YXRpb24uMCIsCiAgICAgICAgICAgICAgImVpcy5wYXJhbWV0ZXIuMCIsCiAgICAgICAgICAgICAgImVzdGltYXRpb24ucG9pbnQuMC5zdGF0ZS4wIiwKICAgICAgICAgICAgICAidHJhbnNpZW50LmV2ZW50LjAudGF1X2Zhc3RfcyIKICAgICAgICAgICAgXSwKICAgICAgICAgICAgInByaW9yX3N0YXR1cyI6ICJoeXBvdGhlc2l6ZWQiLAogICAgICAgICAgICAicmVhc29ucyI6IFsKICAgICAgICAgICAgICAiaHlwb3RoZXNpc19ldmlkZW5jZSIKICAgICAgICAgICAgXSwKICAgICAgICAgICAgInJlc3VsdGluZ19zdGF0dXMiOiAidmFsaWRhdGVkX2Zvcl9kb21haW4iLAogICAgICAgICAgICAic3VwcG9ydGluZ19oeXBvdGhlc2lzX2lkIjogImItaHlwb3RoZXNpcyIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJhc3Nlc3NtZW50X3RhcmdldCI6ICJ2YWxpZGF0ZWRfZm9yX2RvbWFpbiIsCiAgICAgICAgICAgICJjb21wb25lbnRfaWQiOiAiYi12YWxpZGF0aW9uLWVzdGltYXRpb24iLAogICAgICAgICAgICAiZXZpZGVuY2VfaWRzIjogWwogICAgICAgICAgICAgICJjYWxpYnJhdGlvbi5vYnNlcnZhdGlvbi4wIiwKICAgICAgICAgICAgICAiZWlzLnBhcmFtZXRlci4wIiwKICAgICAgICAgICAgICAiZXN0aW1hdGlvbi5wb2ludC4wLnN0YXRlLjAiLAogICAgICAgICAgICAgICJ0cmFuc2llbnQuZXZlbnQuMC50YXVfZmFzdF9zIgogICAgICAgICAgICBdLAogICAgICAgICAgICAicHJpb3Jfc3RhdHVzIjogImh5cG90aGVzaXplZCIsCiAgICAgICAgICAgICJyZWFzb25zIjogWwogICAgICAgICAgICAgICJoeXBvdGhlc2lzX2V2aWRlbmNlIgogICAgICAgICAgICBdLAogICAgICAgICAgICAicmVzdWx0aW5nX3N0YXR1cyI6ICJ2YWxpZGF0ZWRfZm9yX2RvbWFpbiIsCiAgICAgICAgICAgICJzdXBwb3J0aW5nX2h5cG90aGVzaXNfaWQiOiAiYi1oeXBvdGhlc2lzIgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgImFzc2Vzc21lbnRfdGFyZ2V0IjogInZhbGlkYXRlZF9mb3JfZG9tYWluIiwKICAgICAgICAgICAgImNvbXBvbmVudF9pZCI6ICJ0YXVfZmFzdF9zIiwKICAgICAgICAgICAgImV2aWRlbmNlX2lkcyI6IFsKICAgICAgICAgICAgICAiY2FsaWJyYXRpb24ub2JzZXJ2YXRpb24uMCIsCiAgICAgICAgICAgICAgImVpcy5wYXJhbWV0ZXIuMCIsCiAgICAgICAgICAgICAgImVzdGltYXRpb24ucG9pbnQuMC5zdGF0ZS4wIiwKICAgICAgICAgICAgICAidHJhbnNpZW50LmV2ZW50LjAudGF1X2Zhc3RfcyIKICAgICAgICAgICAgXSwKICAgICAgICAgICAgInByaW9yX3N0YXR1cyI6ICJoeXBvdGhlc2l6ZWQiLAogICAgICAgICAgICAicmVhc29ucyI6IFsKICAgICAgICAgICAgICAiaHlwb3RoZXNpc19ldmlkZW5jZSIKICAgICAgICAgICAgXSwKICAgICAgICAgICAgInJlc3VsdGluZ19zdGF0dXMiOiAidmFsaWRhdGVkX2Zvcl9kb21haW4iLAogICAgICAgICAgICAic3VwcG9ydGluZ19oeXBvdGhlc2lzX2lkIjogImItaHlwb3RoZXNpcyIKICAgICAgICAgIH0KICAgICAgICBdLAogICAgICAgICJjb250cmFkaWN0aW9uX3N1bW1hcmllcyI6IFtdLAogICAgICAgICJldmlkZW5jZV9sZXZlbCI6ICJ2YWxpZGF0ZWRfZm9yX2RvbWFpbiIsCiAgICAgICAgImhpc3RvcnkiOiBbXSwKICAgICAgICAiaHlwb3RoZXNpc19pZCI6ICJiLWh5cG90aGVzaXMiLAogICAgICAgICJpZGVudGlmaWFiaWxpdHlfYXNzZXNzbWVudHMiOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJldmlkZW5jZV9pZHMiOiBbCiAgICAgICAgICAgICAgImVpcy5wYXJhbWV0ZXIuMCIsCiAgICAgICAgICAgICAgInRyYW5zaWVudC5ldmVudC4wLnRhdV9mYXN0X3MiCiAgICAgICAgICAgIF0sCiAgICAgICAgICAgICJtZXRyaWNfdmFsdWUiOiAxLjAsCiAgICAgICAgICAgICJyZWFzb25zIjogWwogICAgICAgICAgICAgICJ0aHJlc2hvbGRfc2F0aXNmaWVkIgogICAgICAgICAgICBdLAogICAgICAgICAgICAicmVxdWlyZW1lbnRfaWQiOiAiYi1tb2RlLXNlcGFyYXRpb24iLAogICAgICAgICAgICAic3RhdHVzIjogInNhdGlzZmllZCIKICAgICAgICAgIH0KICAgICAgICBdLAogICAgICAgICJyZWFzb25fY29kZXMiOiBbCiAgICAgICAgICAidmFsaWRhdGlvbl9zYXRpc2ZpZWQiLAogICAgICAgICAgInRpbWVzY2FsZV9zYXRpc2ZpZWQiLAogICAgICAgICAgImlkZW50aWZpYWJpbGl0eV9zYXRpc2ZpZWQiCiAgICAgICAgXSwKICAgICAgICAicmVwZWF0YWJpbGl0eV9hc3Nlc3NtZW50cyI6IFtdLAogICAgICAgICJ0ZW1wb3JhbF9qb2luX2Fzc2Vzc21lbnRzIjogW10sCiAgICAgICAgInRpbWVzY2FsZV9hc3Nlc3NtZW50cyI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgImV2aWRlbmNlX2lkcyI6IFsKICAgICAgICAgICAgICAiZWlzLnBhcmFtZXRlci4wIiwKICAgICAgICAgICAgICAidHJhbnNpZW50LmV2ZW50LjAudGF1X2Zhc3RfcyIKICAgICAgICAgICAgXSwKICAgICAgICAgICAgImxvZ19kaXN0YW5jZSI6IDAuMCwKICAgICAgICAgICAgInBhaXJfcmVxdWlyZW1lbnRfaWQiOiAiYi10aW1lc2NhbGUtcGFpciIsCiAgICAgICAgICAgICJzdGF0dXMiOiAic2F0aXNmaWVkIgogICAgICAgICAgfQogICAgICAgIF0sCiAgICAgICAgInZhbGlkYXRpb25fc3RhdHVzIjogInNhdGlzZmllZCIKICAgICAgfSwKICAgICAgImRlZmluaXRpb24iOiB7CiAgICAgICAgImFtcGxpdHVkZV9nYXRlcyI6IFtdLAogICAgICAgICJjcml0aWNhbF9yZXF1aXJlbWVudF9pZHMiOiBbXSwKICAgICAgICAiZGlzcGxheV9uYW1lIjogIkIgRTJFIHZhbGlkYXRlZCBmb3IgZG9tYWluIiwKICAgICAgICAiZXZpZGVuY2VfcmVxdWlyZW1lbnRzIjogWwogICAgICAgICAgewogICAgICAgICAgICAiZXhwZWN0ZWRfZGlyZWN0aW9uIjogImNhbmRpZGF0ZV9wcmVzZW5jZSIsCiAgICAgICAgICAgICJnYXRlIjogInJlcXVpcmVkIiwKICAgICAgICAgICAgInF1YW50aXR5X3NlbWFudGljIjogInRpbWVfY29uc3RhbnQiLAogICAgICAgICAgICAicmVxdWlyZWRfdW5pdCI6ICJzIiwKICAgICAgICAgICAgInJlcXVpcmVtZW50X2lkIjogImItZWlzLXRhdSIsCiAgICAgICAgICAgICJzb3VyY2VfY2xhc3Nfc2VsZWN0b3JzIjogWwogICAgICAgICAgICAgICJtb2RlbF9kZXJpdmVkIgogICAgICAgICAgICBdLAogICAgICAgICAgICAic291cmNlX2ZpZWxkX3BhdGgiOiAiJC5wYXJhbWV0ZXJzWzBdLnZhbHVlIiwKICAgICAgICAgICAgInN0YWdlIjogInN1cHBvcnQiLAogICAgICAgICAgICAidGFyZ2V0X3NlbGVjdG9yIjogewogICAgICAgICAgICAgICJ0eXBlIjogImV4YWN0X2NvbXBvbmVudCIsCiAgICAgICAgICAgICAgInZhbHVlIjogImItZWlzLXRhdSIKICAgICAgICAgICAgfSwKICAgICAgICAgICAgInZhbGlkaXR5X3JlcXVpcmVtZW50IjogInZhbGlkIgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgImV4cGVjdGVkX2RpcmVjdGlvbiI6ICJjYW5kaWRhdGVfcHJlc2VuY2UiLAogICAgICAgICAgICAiZ2F0ZSI6ICJyZXF1aXJlZCIsCiAgICAgICAgICAgICJxdWFudGl0eV9zZW1hbnRpYyI6ICJ0aW1lX2NvbnN0YW50IiwKICAgICAgICAgICAgInJlcXVpcmVkX3VuaXQiOiAicyIsCiAgICAgICAgICAgICJyZXF1aXJlbWVudF9pZCI6ICJiLXRyYW5zaWVudC10YXUiLAogICAgICAgICAgICAic291cmNlX2NsYXNzX3NlbGVjdG9ycyI6IFsKICAgICAgICAgICAgICAibW9kZWxfZGVyaXZlZCIKICAgICAgICAgICAgXSwKICAgICAgICAgICAgInNvdXJjZV9maWVsZF9wYXRoIjogIiQuZXZlbnRzWzBdLmNhbmRpZGF0ZV9maXRzW10uZGVyaXZlZF9mZWF0dXJlcy50YXVfZmFzdF9zIiwKICAgICAgICAgICAgInN0YWdlIjogInN1cHBvcnQiLAogICAgICAgICAgICAidGFyZ2V0X3NlbGVjdG9yIjogewogICAgICAgICAgICAgICJ0eXBlIjogImV4YWN0X2NvbXBvbmVudCIsCiAgICAgICAgICAgICAgInZhbHVlIjogInRhdV9mYXN0X3MiCiAgICAgICAgICAgIH0sCiAgICAgICAgICAgICJ2YWxpZGl0eV9yZXF1aXJlbWVudCI6ICJ2YWxpZCIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJleHBlY3RlZF9kaXJlY3Rpb24iOiAiY2FuZGlkYXRlX3ByZXNlbmNlIiwKICAgICAgICAgICAgImdhdGUiOiAicmVxdWlyZWQiLAogICAgICAgICAgICAicXVhbnRpdHlfc2VtYW50aWMiOiAicG90ZW50aWFsIiwKICAgICAgICAgICAgInJlcXVpcmVkX3VuaXQiOiAiViIsCiAgICAgICAgICAgICJyZXF1aXJlbWVudF9pZCI6ICJiLXZhbGlkYXRpb24tY2FsaWJyYXRpb24iLAogICAgICAgICAgICAic291cmNlX2NsYXNzX3NlbGVjdG9ycyI6IFsKICAgICAgICAgICAgICAib2JzZXJ2ZWQiCiAgICAgICAgICAgIF0sCiAgICAgICAgICAgICJzb3VyY2VfZmllbGRfcGF0aCI6ICIkLm9ic2VydmF0aW9uc1swXS5wb3RlbnRpYWxfdiIsCiAgICAgICAgICAgICJzdGFnZSI6ICJ2YWxpZGF0aW9uIiwKICAgICAgICAgICAgInRhcmdldF9zZWxlY3RvciI6IHsKICAgICAgICAgICAgICAidHlwZSI6ICJleGFjdF9jb21wb25lbnQiLAogICAgICAgICAgICAgICJ2YWx1ZSI6ICJiLXZhbGlkYXRpb24tY2FsaWJyYXRpb24iCiAgICAgICAgICAgIH0sCiAgICAgICAgICAgICJ2YWxpZGl0eV9yZXF1aXJlbWVudCI6ICJ2YWxpZCIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJleHBlY3RlZF9kaXJlY3Rpb24iOiAiY2FuZGlkYXRlX3ByZXNlbmNlIiwKICAgICAgICAgICAgImdhdGUiOiAicmVxdWlyZWQiLAogICAgICAgICAgICAicXVhbnRpdHlfc2VtYW50aWMiOiAicG90ZW50aWFsIiwKICAgICAgICAgICAgInJlcXVpcmVkX3VuaXQiOiAiViIsCiAgICAgICAgICAgICJyZXF1aXJlbWVudF9pZCI6ICJiLXZhbGlkYXRpb24tZXN0aW1hdGlvbiIsCiAgICAgICAgICAgICJzb3VyY2VfY2xhc3Nfc2VsZWN0b3JzIjogWwogICAgICAgICAgICAgICJtb2RlbF9kZXJpdmVkIgogICAgICAgICAgICBdLAogICAgICAgICAgICAic291cmNlX2ZpZWxkX3BhdGgiOiAiJC5lc3RpbWF0ZXNbMF0uZmlsdGVyZWRfc3RhdGVbMF0udmFsdWUiLAogICAgICAgICAgICAic3RhZ2UiOiAidmFsaWRhdGlvbiIsCiAgICAgICAgICAgICJ0YXJnZXRfc2VsZWN0b3IiOiB7CiAgICAgICAgICAgICAgInR5cGUiOiAiZXhhY3RfY29tcG9uZW50IiwKICAgICAgICAgICAgICAidmFsdWUiOiAiYi12YWxpZGF0aW9uLWVzdGltYXRpb24iCiAgICAgICAgICAgIH0sCiAgICAgICAgICAgICJ2YWxpZGl0eV9yZXF1aXJlbWVudCI6ICJ2YWxpZF9vcl9ub3RfYXNzZXNzZWQiCiAgICAgICAgICB9CiAgICAgICAgXSwKICAgICAgICAiaHlwb3RoZXNpc19pZCI6ICJiLWh5cG90aGVzaXMiLAogICAgICAgICJpZGVudGlmaWFiaWxpdHlfYmluZGluZ3MiOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJnYXRlIjogInJlcXVpcmVkIiwKICAgICAgICAgICAgImlucHV0IjogewogICAgICAgICAgICAgICJyZXF1aXJlbWVudF9pZHMiOiBbCiAgICAgICAgICAgICAgICAiYi1laXMtdGF1IiwKICAgICAgICAgICAgICAgICJiLXRyYW5zaWVudC10YXUiCiAgICAgICAgICAgICAgXSwKICAgICAgICAgICAgICAic2VsZWN0aW9uIjogewogICAgICAgICAgICAgICAgInBhaXJfcmVxdWlyZW1lbnRfaWQiOiAiYi10aW1lc2NhbGUtcGFpciIsCiAgICAgICAgICAgICAgICAidHlwZSI6ICJleGFjdF9wYWlyIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgfSwKICAgICAgICAgICAgImtpbmQiOiAibW9kZV9zZXBhcmF0aW9uIiwKICAgICAgICAgICAgInJlcXVpcmVtZW50X2lkIjogImItbW9kZS1zZXBhcmF0aW9uIiwKICAgICAgICAgICAgInRocmVzaG9sZCI6IDEuMAogICAgICAgICAgfQogICAgICAgIF0sCiAgICAgICAgInBhaXJfcmVxdWlyZW1lbnRzIjogWwogICAgICAgICAgewogICAgICAgICAgICAiZ2F0ZSI6ICJyZXF1aXJlZCIsCiAgICAgICAgICAgICJsZWZ0X3JlcXVpcmVtZW50X2lkIjogImItZWlzLXRhdSIsCiAgICAgICAgICAgICJyZXF1aXJlbWVudF9pZCI6ICJiLXRpbWVzY2FsZS1wYWlyIiwKICAgICAgICAgICAgInJpZ2h0X3JlcXVpcmVtZW50X2lkIjogImItdHJhbnNpZW50LXRhdSIsCiAgICAgICAgICAgICJ0ZW1wb3JhbCI6IHsKICAgICAgICAgICAgICAidHlwZSI6ICJub3RfYXBwbGljYWJsZSIKICAgICAgICAgICAgfQogICAgICAgICAgfQogICAgICAgIF0sCiAgICAgICAgInJlcGVhdGFiaWxpdHlfZ2F0ZXMiOiBbXSwKICAgICAgICAicm9sZV9iaW5kaW5ncyI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgImV2aWRlbmNlX2lkIjogImVpcy5wYXJhbWV0ZXIuMCIsCiAgICAgICAgICAgICJoeXBvdGhlc2lzX2lkIjogImItaHlwb3RoZXNpcyIsCiAgICAgICAgICAgICJyZXF1aXJlbWVudF9pZCI6ICJiLWVpcy10YXUiLAogICAgICAgICAgICAicm9sZSI6ICJzdXBwb3J0IgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgImV2aWRlbmNlX2lkIjogInRyYW5zaWVudC5ldmVudC4wLnRhdV9mYXN0X3MiLAogICAgICAgICAgICAiaHlwb3RoZXNpc19pZCI6ICJiLWh5cG90aGVzaXMiLAogICAgICAgICAgICAicmVxdWlyZW1lbnRfaWQiOiAiYi10cmFuc2llbnQtdGF1IiwKICAgICAgICAgICAgInJvbGUiOiAic3VwcG9ydCIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJldmlkZW5jZV9pZCI6ICJjYWxpYnJhdGlvbi5vYnNlcnZhdGlvbi4wIiwKICAgICAgICAgICAgImh5cG90aGVzaXNfaWQiOiAiYi1oeXBvdGhlc2lzIiwKICAgICAgICAgICAgInJlcXVpcmVtZW50X2lkIjogImItdmFsaWRhdGlvbi1jYWxpYnJhdGlvbiIsCiAgICAgICAgICAgICJyb2xlIjogInZhbGlkYXRpb24iCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAiZXZpZGVuY2VfaWQiOiAiZXN0aW1hdGlvbi5wb2ludC4wLnN0YXRlLjAiLAogICAgICAgICAgICAiaHlwb3RoZXNpc19pZCI6ICJiLWh5cG90aGVzaXMiLAogICAgICAgICAgICAicmVxdWlyZW1lbnRfaWQiOiAiYi12YWxpZGF0aW9uLWVzdGltYXRpb24iLAogICAgICAgICAgICAicm9sZSI6ICJ2YWxpZGF0aW9uIgogICAgICAgICAgfQogICAgICAgIF0sCiAgICAgICAgInRhcmdldF9jb21wb25lbnRzIjogWwogICAgICAgICAgImItZWlzLXRhdSIsCiAgICAgICAgICAiYi12YWxpZGF0aW9uLWNhbGlicmF0aW9uIiwKICAgICAgICAgICJiLXZhbGlkYXRpb24tZXN0aW1hdGlvbiIsCiAgICAgICAgICAidGF1X2Zhc3RfcyIKICAgICAgICBdLAogICAgICAgICJ0aW1lc2NhbGVfZ2F0ZSI6IHsKICAgICAgICAgICJtYXhpbXVtX2xvZ19kaXN0YW5jZSI6IDAuMCwKICAgICAgICAgICJwYWlyX3JlcXVpcmVtZW50X2lkIjogImItdGltZXNjYWxlLXBhaXIiCiAgICAgICAgfSwKICAgICAgICAidmFsaWRhdGlvbl9hcHBsaWNhYmlsaXR5IjogInJlcXVpcmVkIgogICAgICB9CiAgICB9CiAgXSwKICAiaHlwb3RoZXNpc19oaXN0b3J5IjogW10sCiAgImxlZ2FjeV9oeXBvdGhlc2VzIjogW10sCiAgImxpbmVhZ2UiOiB7CiAgICAiS25vd24iOiB7CiAgICAgICJkaXJlY3RfZGVwZW5kZW5jaWVzIjogWwogICAgICAgIHsKICAgICAgICAgICJhcnRpZmFjdF9pZCI6ICJzaGEyNTY6OTI3YzBkM2U4NDY5NzhmODBlOTY0ZmIwNDBiZmNjYTNlMTVjZmZmYWY3OWJkNzEyZTIyM2I2Y2Y2ZDcxYzRmMyIsCiAgICAgICAgICAiYXJ0aWZhY3Rfa2luZCI6ICJjYWxpYnJhdGlvbl9vYnNlcnZhdGlvbnMiLAogICAgICAgICAgInJvbGUiOiAiVHJhbnNmb3JtYXRpb25JbnB1dCIKICAgICAgICB9LAogICAgICAgIHsKICAgICAgICAgICJhcnRpZmFjdF9pZCI6ICJzaGEyNTY6MzI1NDgzYTEwNTBlYjYwM2RkN2IxNWM5NTg3Y2ZhZTk3ZmE0MWFhZjI5YTM5M2E3MWM2MDgyNzI1YjAyOGU0NCIsCiAgICAgICAgICAiYXJ0aWZhY3Rfa2luZCI6ICJlaXNfZml0IiwKICAgICAgICAgICJyb2xlIjogIlRyYW5zZm9ybWF0aW9uSW5wdXQiCiAgICAgICAgfSwKICAgICAgICB7CiAgICAgICAgICAiYXJ0aWZhY3RfaWQiOiAic2hhMjU2OjEyYjczZTAxMWI3MWRmZTM1YmY1ZTZkODhiYTE1ZWNmNDc2N2E3ZmMxZTJjOTU4MjA2MDJlNmMxMjBkYzVkZGYiLAogICAgICAgICAgImFydGlmYWN0X2tpbmQiOiAic3RhdGVfZXN0aW1hdGlvbiIsCiAgICAgICAgICAicm9sZSI6ICJUcmFuc2Zvcm1hdGlvbklucHV0IgogICAgICAgIH0sCiAgICAgICAgewogICAgICAgICAgImFydGlmYWN0X2lkIjogInNoYTI1NjpkOTQ2NWE1ZGVmZjEyMjRjNTE5MGRhZTIxYTY3NGMzNGU5ZWIyOTNmODgwNTU5NzM0OTE2MTZlYTJiYTAyYjVjIiwKICAgICAgICAgICJhcnRpZmFjdF9raW5kIjogInRyYW5zaWVudF9hbmFseXNpcyIsCiAgICAgICAgICAicm9sZSI6ICJUcmFuc2Zvcm1hdGlvbklucHV0IgogICAgICAgIH0KICAgICAgXSwKICAgICAgImlkZW50aXR5IjogewogICAgICAgICJhY3F1aXNpdGlvbl9mYW1pbGllcyI6IHsKICAgICAgICAgICJLbm93biI6IFsKICAgICAgICAgICAgInBoYXNlLWQtZml4dHVyZS1mYW1pbHkiCiAgICAgICAgICBdCiAgICAgICAgfSwKICAgICAgICAiYXJ0aWZhY3RfaWQiOiAic2hhMjU2OjAzNDg3ZjcwMjJhMmZiYjc3YmI4NWJmYmQxZTNjMzBhMzVhZmYxZDFlZmNhN2QyMzFkN2I4OTQzZmQ3YTM0OWUiLAogICAgICAgICJhcnRpZmFjdF9raW5kIjogIm1lY2hhbmlzbV9hbmFseXNpcyIsCiAgICAgICAgImNoYW5uZWxfc2NvcGUiOiAiVW5zcGVjaWZpZWQiLAogICAgICAgICJleHBlcmltZW50X3Njb3BlIjogewogICAgICAgICAgIlNpbmdsZSI6IHsKICAgICAgICAgICAgImV4cGVyaW1lbnRfaWQiOiAiYi1lMmUtMSIKICAgICAgICAgIH0KICAgICAgICB9LAogICAgICAgICJwcm9kdWNlcl92ZXJzaW9uIjogInBoYXNlLWQtZml4dHVyZS12MSIsCiAgICAgICAgInNjaGVtYV92ZXJzaW9uIjogNCwKICAgICAgICAic2VtYW50aWNfc2hhMjU2IjogIjAzNDg3ZjcwMjJhMmZiYjc3YmI4NWJmYmQxZTNjMzBhMzVhZmYxZDFlZmNhN2QyMzFkN2I4OTQzZmQ3YTM0OWUiLAogICAgICAgICJzZW5zb3Jfc2NvcGUiOiAiVW5zcGVjaWZpZWQiCiAgICAgIH0KICAgIH0KICB9LAogICJwcm92ZW5hbmNlIjogewogICAgImNvbmZpZ3VyYXRpb25fcGF0aCI6IG51bGwsCiAgICAiY29uZmlndXJhdGlvbl9zaGEyNTYiOiBudWxsLAogICAgImdlbmVyYXRpb25fdGltZXN0YW1wIjogMCwKICAgICJnaXRfY29tbWl0IjogbnVsbCwKICAgICJpbnB1dF9wYXRoIjogInBoYXNlLWItZml4dHVyZS1pbnB1dCIsCiAgICAiaW5wdXRfc2hhMjU2IjogIjAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAiLAogICAgInNvZnR3YXJlX3ZlcnNpb24iOiAicGhhc2UtYi1maXh0dXJlLWdlbmVyYXRvciIKICB9LAogICJyZWNvcmRzIjogW10sCiAgInNjaGVtYV92ZXJzaW9uIjogNCwKICAidHJhbnNpZW50X2NvbmZpZ3VyYXRpb24iOiB7CiAgICAiYmFzZWxpbmUiOiB7CiAgICAgICJtZXRob2QiOiAibWVkaWFuIiwKICAgICAgInJlc3BvbnNlX21vZGUiOiAiYmFzZWxpbmVfcmVsYXRpdmUiCiAgICB9LAogICAgImV4cG9ydCI6IHsKICAgICAgImZlYXR1cmVzX2ZpbGVuYW1lIjogInRyYW5zaWVudF9mZWF0dXJlcy5jc3YiLAogICAgICAianNvbl9maWxlbmFtZSI6ICJ0cmFuc2llbnRfcmVzdWx0cy5qc29uIiwKICAgICAgIm1vZGVsX2NvbXBhcmlzb25fZmlsZW5hbWUiOiAidHJhbnNpZW50X21vZGVsX2NvbXBhcmlzb24uY3N2IiwKICAgICAgInJlcG9ydF9maWxlbmFtZSI6ICJ0cmFuc2llbnRfcmVwb3J0LnR4dCIKICAgIH0sCiAgICAibW9kZWxzIjogewogICAgICAiYmV0YV9tYXgiOiAxLjAsCiAgICAgICJiZXRhX21pbiI6IDAuMDUsCiAgICAgICJlbmFibGVkIjogWwogICAgICAgICJzaW5nbGUiCiAgICAgIF0KICAgIH0sCiAgICAib3B0aW1pemVyIjogewogICAgICAiZnRvbCI6IDFlLTEwLAogICAgICAiZ3RvbCI6IDFlLTEwLAogICAgICAibWF4aW11bV9pdGVyYXRpb25zIjogNDAwLAogICAgICAibXVsdGlwbGVfc3RhcnRzIjogOCwKICAgICAgInBhdGllbmNlIjogNDAwLAogICAgICAic3RlcF9ib3VuZCI6IDUwLjAsCiAgICAgICJ4dG9sIjogMWUtMTAKICAgIH0sCiAgICAicGxvdHRpbmciOiB7CiAgICAgICJlbmFibGVkIjogZmFsc2UsCiAgICAgICJpbmNsdWRlX2NvbXBvbmVudHMiOiB0cnVlLAogICAgICAiaW5jbHVkZV9tb2RlbF9jb21wYXJpc29uIjogdHJ1ZSwKICAgICAgImluY2x1ZGVfcmVzaWR1YWxzIjogdHJ1ZQogICAgfSwKICAgICJzY2hlbWFfdmVyc2lvbiI6IDEsCiAgICAic2VnbWVudGF0aW9uIjogewogICAgICAiYmFzZWxpbmVfd2luZG93X3MiOiAyMC4wLAogICAgICAiZHVwbGljYXRlX3RpbWVzdGFtcF9wb2xpY3kiOiAiZXJyb3IiLAogICAgICAiaXJyZWd1bGFyX3NhbXBsaW5nX3BvbGljeSI6ICJhbGxvdyIsCiAgICAgICJtYXhpbXVtX21pc3NpbmdfZnJhY3Rpb24iOiAwLjIsCiAgICAgICJtaW5pbXVtX2R1cmF0aW9uX3MiOiAxMC4wLAogICAgICAibWluaW11bV9wb2ludHMiOiAyMCwKICAgICAgIm5vbl9tb25vdG9uaWNfcG9saWN5IjogInNvcnQiLAogICAgICAicG9zdF9ldmVudF9zIjogMzAwLjAsCiAgICAgICJwcmVfZXZlbnRfcyI6IDMwLjAKICAgIH0sCiAgICAic2VsZWN0aW9uIjogewogICAgICAiY3JpdGVyaW9uIjogImFpYyIKICAgIH0sCiAgICAic291cmNlX3BhdGgiOiBudWxsLAogICAgInVuY2VydGFpbnR5IjogewogICAgICAiYm9vdHN0cmFwX2l0ZXJhdGlvbnMiOiAwLAogICAgICAiY29uZmlkZW5jZV9sZXZlbCI6IDAuOTUsCiAgICAgICJtaW5pbXVtX3N1Y2Nlc3NfZnJhY3Rpb24iOiAwLjgsCiAgICAgICJzZWVkIjogNDIKICAgIH0sCiAgICAidmFsaWRhdGlvbiI6IHsKICAgICAgImJvdW5kX3Byb3hpbWl0eV9mcmFjdGlvbiI6IDAuMDEsCiAgICAgICJoaWdoX2F1dG9jb3JyZWxhdGlvbl90aHJlc2hvbGQiOiAwLjgsCiAgICAgICJtYXhpbXVtX3RhdV90b193aW5kb3dfcmF0aW8iOiAxMDAuMCwKICAgICAgIm1pbmltdW1fdGF1X3JhdGlvIjogMy4wLAogICAgICAibmVnbGlnaWJsZV9hbXBsaXR1ZGVfZnJhY3Rpb24iOiAwLjA1CiAgICB9CiAgfSwKICAidHJhbnNpZW50X3RpbWVzY2FsZXMiOiBbCiAgICB7CiAgICAgICJjb25maWRlbmNlX2ludGVydmFsX3MiOiBbCiAgICAgICAgOS4wLAogICAgICAgIDEzLjAKICAgICAgXSwKICAgICAgImRlcml2YXRpb24iOiB7CiAgICAgICAgImNpcmN1aXRfcGF0aCI6IG51bGwsCiAgICAgICAgImNvbnZlbnRpb24iOiBudWxsLAogICAgICAgICJlcXVhdGlvbiI6ICJzdG9yZWQiCiAgICAgIH0sCiAgICAgICJsYWJlbCI6ICJUcmFuc2llbnQgdGF1IiwKICAgICAgInNlbWFudGljX3JvbGUiOiBudWxsLAogICAgICAic291cmNlIjogInRyYW5zaWVudF9maXQiLAogICAgICAic291cmNlX3BhcmFtZXRlcnMiOiBbXSwKICAgICAgInN0YW5kYXJkX2Vycm9yX3MiOiAxLjAsCiAgICAgICJ0aW1lc2NhbGVfaWQiOiAidHJhbnNpZW50LXRhdSIsCiAgICAgICJ2YWxpZGl0eSI6ICJ2YWxpZCIsCiAgICAgICJ2YWx1ZV9zIjogMTEuMCwKICAgICAgIndhcm5pbmdzIjogW10KICAgIH0KICBdLAogICJ0cmVuZHMiOiBbXSwKICAid2FybmluZ3MiOiBbXQp9Cg==
```

##### 18.11.2.c Catalog failures and closed bundles

| ID → destination | Class; exact complete bytes/operation; SHA-256 | exact reader result |
|---|---|---|
| `N-X01` → `failure/catalog_malformed.json` | 2; `{not-json}\n`; `8c69fc307fed3936d6a8ac679c0079c9bfd11f9de2a43e20ae25ff2a899d9776` | `LineageCatalogReadError::Json` |
| `N-X02` → `failure/catalog_invalid_structure.json` | 2; `{"schema_version":1,"artifacts":[]}\n`; `81a6dbddac1e825c74477d5b077a895d8f1db6e7022fd695d025cc6377a9ee9c` | `Json` (array not map) |
| `N-X03` → `failure/catalog_schema2.json` | 2; `{"schema_version":2,"artifacts":{}}\n`; `6fb4379a5a30eb3e959dbb873dab858ee583ae782acf2b9d8160653141eda3d4` | `UnsupportedSchemaVersion { actual:2 }` |
| `N-X04` → `failure/catalog_duplicate_root_key.json` | 2; `{"schema_version":1,"schema_version":1,"artifacts":{}}\n`; `2989e7f1c3b87e294c817e046496cbd8a3feb600ef988f32ddaeb938a1128cc2` | `DuplicateField { field:"schema_version" }` |
| `N-X05` → `failure/catalog_key_identity_mismatch.json` | 3 from `N-F10`: replace only first artifact-map key `sha256:12b73e011b71dfe35bf5e6d88ba15ecf4767a7fc1e2c95820602e6c120dc5ddf` by `sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`; node identity unchanged | `KeyIdentityMismatch`, no output |

`phase_d_b_e2e_v1` maps mechanism=`N-F01`, health=`N-F02`, EIS=`N-F03`,
transient=`N-F04`, calibration-observations=`N-F05`, estimation=`N-F06`,
calibration=`N-F07`, signal=`N-F08`, model=`N-F09`, catalog=`N-F10`.
`phase_d_b_e2e_v1_without_transient` removes only `N-F04`; `_without_signal`
removes only `N-F08`; `phase_d_legacy_required_v1` maps `N-F12,N-F11`;
`phase_d_scope_sensor_mismatch_v1` replaces health with `N-F15`;
`phase_d_scope_experiment_mismatch_v1` replaces mechanism with `N-F16`;
`phase_d_scope_unknown_v1` uses `N-F17,N-F18`; optional mismatch adds `N-F19`;
different-families uses `N-F20,N-F21`; missing-unit uses `N-F22`;
comparable-warning uses `N-F23`; EIS-plot uses `N-F24`; transient zero/duplicate
use `N-F25/N-F26`; model-missing uses `N-F27`. Unlisted roles retain their
base mappings. Base is scope-compatible=yes; mismatch bundles reject at their
named first axis; unknown-scope and different-family bundles are accepted.

### 18.12 Mandatory test inventory — exactly 73 unique tests

All tests live in `tests/phase_d_reporting_public_output.rs` unless marked
unit; a test name appears once.  `R` is the requirement in section 18.13,
`AC` is its acceptance criterion, and each cell states target/fixture/expected
falsification result.  Status `ok` means successful complete publication;
`err(X)` means exact `PublicReportError::X` unless explicitly `CliError::Parse`.

The table immediately below freezes the unique name, owner, requirement,
criterion, and falsification topic. Its legacy shorthand such as `current`,
`mutation`, or a descriptive fixture nickname is an index only and has **no
fixture authority**. The sole per-test fixture/bundle and expected-output
authority is the same-numbered row in 18.12.1, which resolves exclusively to
the 18.11.2 ledger.

| # | exact name / class | R/AC; owner target | fixture/input → exact expected result and falsification |
|---:|---|---|---|
| 1 | `phase_d_cli_requires_mechanism_and_health` integration | R01/AC01; cli/report | current missing each flag → `CliError::Parse`; no final root |
| 2 | `phase_d_clap_rejects_unknown_format_before_runner` integration | R01/AC02; cli | `--format yaml` → `CliError::Parse`; no runner call |
| 3 | `phase_d_cli_rejects_unpaired_calibration_inputs` integration | R02/AC03; report options | one pair flag → `err(InvalidCombination)` |
| 4 | `phase_d_cli_rejects_unknown_selection` integration | R02/AC04; report_config | `unknown` figure/table → `err(InvalidSelection)` |
| 5 | `phase_d_cli_rejects_duplicate_selection` integration | R02/AC05; report_config | duplicate ID → `err(InvalidSelection)` |
| 6 | `phase_d_cli_rejects_existing_output_without_overwrite` integration | R03/AC06; runner publish | current + output root → `err(OutputCollision)` |
| 7 | `phase_d_cli_overwrite_rejects_unmanaged_entry` integration | R03/AC07; runner publish | failure/unmanaged_output → `err(UnmanagedOutputEntry)` and `keep.txt` retained |
| 8 | `phase_d_reads_only_canonical_artifacts` integration | R04/AC08; reader | wrong-kind current JSON → `err(Artifact)` with `IncompatibleKind` |
| 9 | `phase_d_rejects_unsupported_optional_schema` integration | R04/AC09; reader | schema-2 optional → `err(Artifact)` with `UnsupportedSchemaVersion` |
| 10 | `phase_d_catalog_reader_accepts_schema1_and_canonical_order` integration | R05/AC10; domain catalog reader | current catalog → `ok`, D-TBL-04 root row then canonical direct-dependency rows |
| 11 | `phase_d_catalog_reader_rejects_schema2` integration | R05/AC11; domain catalog reader | catalog_schema2 → `err(LineageCatalog)` |
| 12 | `phase_d_catalog_reader_rejects_key_identity_mismatch` integration | R05/AC12; domain catalog reader | catalog_bad_key → `err(LineageCatalog)` |
| 13 | `phase_d_catalog_reader_rejects_duplicate_json_key` integration | R05/AC13; domain catalog reader | catalog_duplicate_key → `err(LineageCatalog)` |
| 14 | `phase_d_reporting_never_ad_hoc_parses_catalog` unit | R05/AC14; reporting reader | source-level forbidden-call guard → no `serde_json::from_*` catalog parse |
| 15 | `phase_d_required_known_scope_mismatch_is_rejected` integration | R06/AC15; compatibility | incompatible_sensor health → `err(RequiredInputsIncompatible)` sensor |
| 16 | `phase_d_required_experiment_mismatch_is_rejected` integration | R06/AC16; compatibility | incompatible_experiment mechanism → exact experiment error |
| 17 | `phase_d_required_equal_unknown_scope_reuses_phase_c_admissibility` integration | R06/AC17; compatibility | both known identities use equal serialized unknown/unspecified scope → `ok`, summary token `compatible`; proves no Phase-D unverifiable branch |
| 18 | `phase_d_required_legacy_unknown_is_explicit` integration | R06/AC18; compatibility | legacy unknown required → `ok`, no compatible token |
| 19 | `phase_d_optional_known_mismatch_is_rejected_when_unselected` integration | R07/AC19; compatibility | incompatible_optional plus figures none → `err(OptionalInputIncompatible)` |
| 20 | `phase_d_optional_legacy_unknown_is_limited_not_inferred` integration | R07/AC20; compatibility | supplied legacy optional mechanism `N-F12` with the base required pair → `ok`, compatibility `legacy_unknown`; it does not infer scope, family, or independence |
| 21 | `phase_d_schema4_health_projects_exactly_nine_dimensions` integration | R08/AC21; projection | current → nine ordered dimensions |
| 22 | `phase_d_schema3_health_does_not_synthesize_phase_c` integration | R08/AC22; projection | legacy health → zero dimensions and exact legacy token |
| 23 | `phase_d_legacy_mechanism_marks_phase_b_assessment_unavailable` integration | R08/AC23; projection | legacy mechanism → exact legacy token |
| 24 | `phase_d_public_summary_schema1_is_closed_and_ordered` integration | R09/AC24; document | current → all section 18.5 keys/order, no unknown key |
| 25 | `phase_d_public_summary_field_authorities_are_typed_copies` unit | R09/AC25; projection | current typed mutations → copied value proves no free map/recompute |
| 26 | `phase_d_render_manifest_schema1_records_semantic_fields` integration | R10/AC26; manifest | current → schema, refs, generated, unavailable, warnings, IDs |
| 27 | `phase_d_render_manifest_orders_paths_and_legacy_notices` integration | R10/AC27; manifest | current+legacy → `/` paths, deterministic orders/notices |
| 28 | `phase_d_markdown_sections_and_order_are_stable` integration | R11/AC28; document | current → exact twelve section order |
| 29 | `phase_d_mechanism_table_projects_serialized_gate_statuses` integration | R12/AC29; tables | current → D-TBL-01 copied gate/status fields |
| 30 | `phase_d_health_table_preserves_dqi_reason_codes` integration | R12/AC30; tables | DQI mutation → literal DQI/reasons |
| 31 | `phase_d_health_table_preserves_indeterminate_reason_codes` integration | R12/AC31; tables | indeterminate mutation → literal token/reasons |
| 32 | `phase_d_evidence_provenance_csv_is_deterministic` integration | R12/AC32; tables | current → canonical row sort |
| 33 | `phase_d_artifact_lineage_table_projects_root_and_direct_dependency_rows` integration | R12/AC33; tables | current+legacy → D-TBL-04 one root row then canonical dependency rows; no catalog-only node/traversal |
| 34 | `phase_d_timescale_table_uses_only_serialized_comparisons` integration | R12/AC34; tables | current misleading optional → D-TBL-05 `cmp-01` only |
| 35 | `phase_d_current_baseline_csv_uses_unique_feature_unit_authority` integration | R12/AC35; tables | current → `V/decade`, 0.058/0.059 |
| 36 | `phase_d_current_baseline_csv_marks_missing_unit_authority` integration | R12/AC36; tables | baseline_no_unit/duplicate → availability and `NA` |
| 37 | `phase_d_model_consistency_csv_never_recomputes_residual` integration | R12/AC37; tables | model residual literal → copied residual |
| 38 | `phase_d_figure_mechanism_uses_stored_log_distance_only` integration | R13/AC38; figures | current → 0.041 only; no `log10` call |
| 39 | `phase_d_figure_health_shows_all_nine_statuses` integration | R13/AC39; figures | current → nine dimension labels |
| 40 | `phase_d_figure_baseline_uses_unique_feature_unit_authority` integration | R13/AC40; figures | current/duplicate → exact unit/unavailable |
| 41 | `phase_d_figure_eis_nyquist_uses_direct_serialized_imaginary_values` integration | R13/AC41; figures | eis_nyquist_sign → y `-2,-1`, label/caption exact |
| 42 | `phase_d_figure_eis_bode_projects_serialized_frequency_magnitude_phase` integration | R13/AC42; figures | eis_bode → data, labels, observed/fitted order |
| 43 | `phase_d_figure_transient_renders_one_unique_selected_fit` integration | R13/AC43; figures | current transient → exactly selected candidate series |
| 44 | `phase_d_figure_transient_zero_match_default_is_manifest_unavailable` integration | R13/AC44; figures | zero match + default → `ok`, reason exact |
| 45 | `phase_d_figure_transient_zero_match_explicit_fails_atomically` integration | R13/AC45; figures/publish | zero + explicit ID → `err(RequestedOutputUnavailable)`, no root |
| 46 | `phase_d_figure_transient_duplicate_match_is_never_first_selected` integration | R13/AC46; figures | duplicate → reason ambiguous; no rendered fitted points |
| 47 | `phase_d_figure_calibration_has_no_theoretical_line` integration | R13/AC47; figures | current → only validation predictions |
| 48 | `phase_d_figure_signal_marks_missing_samples` integration | R13/AC48; figures | signal missing value → marker, no drop |
| 49 | `phase_d_figure_estimation_shows_serialized_uncertainty_only` integration | R13/AC49; figures | current → no variance conversion |
| 50 | `phase_d_figure_model_never_maps_missing_to_zero` integration | R13/AC50; figures | missing observed/residual → unavailable/NA |
| 51 | `phase_d_figure_lineage_marks_legacy_unknown` integration | R13/AC51; figures | legacy → exact unknown label |
| 52 | `phase_d_selected_figure_files_are_valid_svg_and_png` integration | R14/AC52; figures | current → parse SVG/PNG each selected figure |
| 53 | `phase_d_figure_metadata_has_labels_units_series_and_dqi_visibility` integration | R14/AC53; figures | current + DQI → semantic text/series |
| 54 | `phase_d_format_json_writes_summary_manifest_and_selected_visuals` integration | R15/AC54; runner | json/current → no Markdown, other requested files |
| 55 | `phase_d_format_markdown_writes_report_manifest_and_selected_visuals` integration | R15/AC55; runner | markdown/current → no summary, other requested files |
| 56 | `phase_d_default_selection_is_best_effort_and_explicit_all_is_strict` integration | R15/AC56; selection | absent transient → default ok/not selected; explicit all error |
| 57 | `phase_d_public_float_format_is_exact` unit | R16/AC57; format helper | numeric_values → exact 0 normalized and Display strings |
| 58 | `phase_d_csv_markdown_and_figure_annotations_share_float_format` integration | R16/AC58; writers/figures | numeric_values → equal strings across outputs |
| 59 | `phase_d_nonfinite_projection_fails_before_serialization` unit | R16/AC59; staging validation | injected NaN → `err(StagingValidation)` |
| 60 | `phase_d_staging_write_failure_publishes_no_final_bundle` integration | R17/AC60; publish | denied writer → `err(Write)`, final absent, staging cleanup |
| 61 | `phase_d_publication_failure_restores_previous_complete_bundle` integration | R17/AC61; publish | injected publish rename failure → previous manifest still present |
| 62 | `phase_d_rendering_does_not_mutate_health_assessment` integration | R18/AC62; projection | current clone equality after ok/failure |
| 63 | `phase_d_rendering_does_not_mutate_mechanism_assessment` integration | R18/AC63; projection | current clone equality after ok/failure |
| 64 | `phase_d_repeated_render_is_deterministic` integration | R19/AC64; all writers | two fresh roots → byte-identical JSON/CSV/Markdown/metadata |
| 65 | `phase_d_large_history_does_not_duplicate_artifact_series_unboundedly` integration | R20/AC65; projection | literal 1,000 histories/10,000 evidence → bounded behavior |
| 66 | `phase_d_golden_expectations_are_hand_derived_from_fixture_literals` unit | R21/AC66; test fixtures | repository fixture audit → no renderer-generated golden |
| 67 | `phase_d_public_report_error_is_publicly_reachable` integration | R22/AC67; reporting/runners | external integration test imports `reporting::PublicReportError`, converts it with `RunnerError::from`, and matches `RunnerError::PublicReport`; falsifies inaccessible public payload |
| 68 | `phase_d_catalog_reader_rejects_syntactically_malformed_json` integration | R05/AC68; domain catalog reader | `failure/catalog_malformed.json` exact bytes → `err(LineageCatalog::Json)` and no final root; falsifies accidental acceptance/late publication |
| 69 | `phase_d_catalog_reader_rejects_structurally_invalid_catalog` integration | R05/AC69; domain catalog reader | `failure/catalog_invalid_structure.json` exact bytes → `err(LineageCatalog::Json)` and no final root; falsifies array-as-map fallback |
| 70 | `phase_d_different_known_acquisition_families_are_projected_not_rejected` integration | R06/AC70; compatibility/projection | manifest different-families row plus required pair → `ok`, compatibility `compatible`, both literal family lists projected, no independence label; falsifies family-equality gating |
| 71 | `phase_d_comparable_with_warnings_is_rendered_and_disclosed` integration | R13/AC71; figure/table/document/manifest | manifest comparable-with-warnings health row → D-FIG-03 and D-TBL-07 written with literal values and closed warning in manifest plus Markdown; falsifies silent drop or divergence |
| 72 | `phase_d_lineage_catalog_input_reference_is_catalog_variant_without_artifact_fields` integration | R09/R10/AC72; summary/manifest | `base.catalog` in `phase_d_b_e2e_v1` → both documents contain exactly one catalog-tagged object: summary keys are exactly `input_kind,supplied_path_basename,schema_version,availability,validation`; manifest keys append only `compatibility`. Values are `lineage_catalog`, `lineage_catalog.json`, `1`, `available`, `validated`, and manifest `not_applicable`. The test asserts the absence of `input_flag`, `artifact_id`, `artifact_kind`, `lineage`, and `acquisition_families` in both values, then injects `artifact_kind:"artifact_lineage_catalog"` into an otherwise identical value and requires typed deserialization/closed-key validation to fail. |
| 73 | `phase_d_fixture_ledger_materializes_exact_literal_files_and_canonical_readers_accept_them` integration | R23/AC73; fixture ledger/readers | materialize **every valid** `N-F*` entry in the complete section-18.11.2 ledger, verify its final byte SHA-256, exact reader/schema policy, known identity (including producer, semantic hash, scopes, families, and direct dependencies) or exact legacy state, and complete serialized provenance. It separately materializes rejected `N-F13,N-F14` and every `N-X*` catalog/error entry and asserts each named reader error. It also verifies every defined bundle is scope-compatible where its contract says it must be. A schema-1 calibration or signal presented to the certified Phase-D policy fails this test. |

#### 18.12.1 Complete test → fixture/bundle → expectation chain

The following matrix is normative and supplements the preceding name/owner
inventory. Its ordinal is the exact one and only test name on the same-numbered
row above; `E##` is the complete expected-output contract in 18.12.2. `none`
means the test has no file fixture because it stops before reading (parser,
in-memory typed value, or injected I/O seam); it must not invent one.

| # | exact fixture/bundle authority | expected-output contract |
|---:|---|---|
| 1 | none; omit each required CLI flag | E01 parse/no-root |
| 2 | none; `--format yaml` | E02 parse/no-runner |
| 3 | `N-F07` alone, then `N-F05` alone | E03 invalid-combination |
| 4–5 | none; unknown/duplicate selection tokens | E04 invalid-selection |
| 6 | `phase_d_b_e2e_v1` + existing certified root | E05 output-collision |
| 7 | `phase_d_b_e2e_v1` + `N-F28` | E06 unmanaged-entry |
| 8 | `N-F14` | E07 incompatible-kind |
| 9 | `N-F13` | E08 unsupported-schema |
| 10 | `N-F10` / `phase_d_b_e2e_v1` | E09 catalog/D-TBL-04 |
| 11–13 | `N-X03`, `N-X05`, `N-X04` respectively | E10 catalog-errors |
| 14 | none; reporting source guard | E11 only-catalog-reader |
| 15–16 | `phase_d_scope_sensor_mismatch_v1`; `phase_d_scope_experiment_mismatch_v1` | E12 first-axis-error |
| 17 | `phase_d_scope_unknown_v1` | E13 compatible-unknown |
| 18 | `phase_d_legacy_required_v1` | E14 legacy-required |
| 19 | `phase_d_optional_mismatch_v1` | E15 optional-error |
| 20 | `N-F12` as supplied legacy optional mechanism with base required pair | E16 legacy-optional |
| 21, 30–31, 39, 53 | `phase_d_b_e2e_v1` / `N-F02` | E17 health-nine/DQI/Indeterminate |
| 22–23 | `N-F11`; `N-F12` | E18 legacy-projection |
| 24–26 | `phase_d_b_e2e_v1` | E19 base-summary-manifest |
| 27 | `phase_d_legacy_required_v1` | E20 legacy-manifest-order |
| 28–29, 32–33 | `phase_d_b_e2e_v1` | E21 base-markdown/tables |
| 34, 38 | `phase_d_b_e2e_v1` with the exact stored comparison fixture `N-F31` | E22 stored-timescale |
| 35, 40 | `phase_d_comparable_warning_v1` | E23 baseline-pair |
| 36 | `phase_d_missing_unit_v1` | E24 unit-unavailable |
| 37, 50 | base model; `phase_d_model_missing_v1` | E25 model-projection |
| 41–42 | `phase_d_eis_plot_v1` | E26 Nyquist/Bode |
| 43 | `phase_d_b_e2e_v1` | E27 unique-transient |
| 44–45 | `phase_d_transient_zero_v1` | E28 zero-transient |
| 46 | `phase_d_transient_duplicate_v1` | E29 duplicate-transient |
| 47 | `phase_d_b_e2e_v1` | E30 calibration |
| 48 | `phase_d_b_e2e_v1` / `N-F08` | E31 missing-signal-sample |
| 49 | `phase_d_b_e2e_v1` | E32 estimation |
| 51 | `phase_d_legacy_required_v1` | E33 legacy-lineage |
| 52 | `phase_d_b_e2e_v1` | E34 valid-SVG/PNG |
| 54–55 | `phase_d_b_e2e_v1` with JSON; then Markdown | E35 format-files |
| 56 | `phase_d_b_e2e_v1_without_transient` | E36 default/explicit-selection |
| 57–59 | none; exact typed numeric matrix / injected NaN | E37 numeric/staging |
| 60–61 | `phase_d_b_e2e_v1` + injected writer/rename seam | E38 atomic-publication |
| 62–64 | `phase_d_b_e2e_v1` | E39 immutable/repeatable |
| 65 | `N-F30` | E40 large-history |
| 66 | every `N-F*`, `N-X*`, and `E*` literal | E41 non-circular-golden |
| 67 | none; public external integration import | E42 public-error |
| 68–69 | `N-X01`; `N-X02` | E43 malformed-catalogs |
| 70 | `phase_d_different_families_v1` | E44 families-projection |
| 71 | `phase_d_comparable_warning_v1` | E45 comparable-warning |
| 72 | `phase_d_b_e2e_v1` / `N-F10` | E46 catalog-reference |
| 73 | every valid `N-F*` and every `N-X*` | E47 full-materialization |

`N-F31` is a Class-2 literal `mechanism/timescale_cmp01.json`: it is the
complete `N-F01` literal with exactly one serialized comparison
`{comparison_id:"cmp-01",record_id:"rec-01",eis_timescale_id:"eis-tau",
transient_timescale_id:"transient-tau",ratio:1.1,log10_distance:0.041,
symmetric_relative_difference:0.09523809523809523,
confidence_interval_overlap:true,compatibility_probability:0.9,
evidence_level:"moderate",warnings:[]}` and its referenced serialized EIS
and transient timescale records `{id:"eis-tau",value_s:10.0,standard_error_s:1.0}` and
`{id:"transient-tau",value_s:11.0,standard_error_s:1.0}`. Its exact literal
is encoded in `N-L08`, its final SHA is `d0a373578981f8db5f69e722d484c3be32e78e2f55d563d22125b3692332aee6`,
and its known identity is producer `phase-d-fixture-v1`, schema 4, base scopes,
family `phase-d-fixture-family`, semantic/artifact ID
`sha256:03487f7022a2fbb77bb85bfbd1e3c30a35aff1d1efca7d231d7b8943fd7a349e`.

#### 18.12.2 Exact expected-output ledger

`E19` is the complete semantic `PublicSummaryV1`/`RenderManifestV1` contract
for `phase_d_b_e2e_v1`, JSON format, default selections. The summary envelope
is exactly schema `1`, output kind `phase_d_public_scientific_output`, renderer
contract `mhi_v1_phase_d_public_output_v1`, route `electroanalysis report
render`, then the section-18.5 declaration-order fields. Its ten input
references are the nine artifact variants in fixed flag order plus the catalog
variant: every supplied artifact has `input_kind="artifact"`, basename from
its `base/*.json` path, actual kind/schema/lineage/families from `N-F01`–
`N-F09`, and `available`; the catalog object has exactly
`{input_kind:"lineage_catalog",supplied_path_basename:"lineage_catalog.json",
schema_version:1,availability:"available",validation:"validated"}`. The
required compatibility is `compatible`; every supplied known optional is
`compatible`, legacy model is `legacy_unknown`, and catalog is absent from the
compatibility vector. Mechanism is the complete `N-F01` serialized projection;
health is the complete `N-F02.phase_c` nine-dimension projection in
`HealthDimension::ALL` order; optional details are the typed copied records;
lineage roots are the supplied artifact roots in input-flag order and direct
dependencies only. There are no omitted fields, map-valued extensions, or
invented values. The manifest has the section-18.6 declaration order,
`final_output_status="published"`, the same ten references (catalog appends
only `compatibility:"not_applicable"`), and requested exactly
`{formats:[json],figures:[mechanism_timescale,sensor_health_dimension_status,
current_vs_baseline,eis_nyquist,eis_bode,transient_response,
calibration_performance,signal_diagnostics,estimation_observed_predicted,
model_observed_predicted,lineage],tables:[mechanism_evidence,health_dimensions,
evidence_provenance,artifact_lineage,timescale_comparison,model_consistency,
current_vs_baseline],figures_mode:default,tables_mode:default,overwrite:false}`.
Render order is summary, each selected table in that exact list, then each
selected figure in that exact list with SVG immediately before PNG; generated
files retain that order, unavailable figures occur only in unavailable_outputs,
and paths are `public_summary.schema1.json`, `tables/<table-id>.csv`, and
`figures/<figure-id>.svg|png`. Warnings, notices, compatibility outcomes, and
determinism clock `null` are then in their closed section-18.6 orders. This
paragraph fixes every field by
the explicit type graph plus a named source literal; no renderer output may be
used as its own oracle.

`E21` fixes table rows. D-TBL-04 has one root row for each supplied artifact in
flag order; `N-F01` is followed by its four direct-dependency rows exactly
`calibration_observations/sha256:927c0d3e846978f80e964fb040bfcca3e15cfffaf79bd712e223b6cf6d71c4f3`,
`eis_fit/sha256:325483a1050eb603dd7b15c9587cfae97fa41aaf29a393a71c6082725b028e44`,
`state_estimation/sha256:12b73e011b71dfe35bf5e6d88ba15ecf4767a7fc1e2c95820602e6c120dc5ddf`,
and `transient_analysis/sha256:d9465a5deff1224c5190dae21a674c34e9eb293f88055973491616ea2ba02b5c`,
all role `transformation_input`; every other known base root has zero dependency
rows; legacy model root has `legacy_unknown` and `NA` root ID/kind. Catalog-only
nodes never produce a row. D-TBL-07 for `E23/E45` is exactly one available row
`signal.rms_noise,V,0.21472615802499273,0.058,comparable_with_warnings,
0.15672615802499273,2.702175138361943,NA,NA,NA,NA,0,
temperature differs within configured tolerance,baseline_comparable_with_warnings`;
`E24` has `unit_authority_unavailable` and `NA` in every value/unit cell.
`E22` D-TBL-05 is exactly the one `cmp-01` row with the values in `N-F31`, in
that order, and no recomputation. `E25` D-TBL-06 copies model points exactly;
the `N-F27` null values become `NA`, never `0`.

`E26` fixes figure semantics: D-FIG-04 observed `(1,-2),(2,-1)` then fitted
`(1.5,-1.5),(2.5,-0.5)`, unit Ohm, caption `Imaginary impedance is plotted
with its serialized sign; Phase D performs no Nyquist sign transform.`;
D-FIG-05 frequencies `[1,10]`, observed magnitude
`[2.23606797749979,2.23606797749979]`, observed phase
`[-63.43494882292201,-26.56505117707799]`, then fitted magnitude
`[2.1213203435596424,2.5495097567963922]`, fitted phase
`[-45,-11.309932474020215]`, units Hz/Ohm/degrees. `E27` is observed, fitted,
residual series from the one `N-F04.events[0].candidate_fits[0]` in that
order. `E28` declares D-FIG-06 unavailable `selected_fit_not_found` by default
and `RequestedOutputUnavailable`/no root explicitly; `E29` declares
`selected_fit_ambiguous` with no fitted points. `E30` plots only serialized
validation prediction points; `E31` preserves the exact null at timestamp 2
as a missing marker; `E32` uses serialized estimation values only; `E33` emits
the exact LegacyUnknown label. `E17` lists all nine health status/evidence
tokens, including `data_quality_insufficient/poor_data_quality` and every
`indeterminate/no_evidence` token from `N-F02`; `E45` adds the one closed
baseline warning to figure, table, Markdown, and manifest.

`E01`–`E18`, `E20`, `E24`, and `E28`–`E47` mean the exact error/no-root,
availability, files-present/files-absent, and warning outcomes named in their
corresponding preceding inventory row. `E34` requires each selected SVG/PNG
parse and nonzero length; `E35` specifies JSON writes summary+manifest and no
Markdown, Markdown writes report+manifest and no summary; `E36` fixes default
best-effort versus explicit failure; `E37` uses `0`, `-0`, finite displays and
NaN failure from 18.9; `E38` preserves/omits roots exactly; `E39` requires
byte-identical second render; `E40` is exactly 1,000 histories/10,000 evidence;
`E43` uses `N-X01,N-X02`; `E46` is the exact closed catalog objects; `E47`
checks every final checksum, reader, schema, identity, provenance, base-bundle
compatibility, and the expected rejection of `N-F13,N-F14,N-X01`–`N-X05`.
Thus every test has an exact fixture/bundle,
expectation, and falsification chain.

### 18.13 Traceability, two-implementer audit, and readiness

There are exactly **23 requirements**, **73 acceptance criteria**, and **73
mandatory tests**.  Requirement IDs are `D-R01` route/parser, `R02` selection,
`R03` atomic output, `R04` artifact readers, `R05` catalog reader, `R06`
required compatibility, `R07` optional compatibility, `R08` legacy projection,
`R09` public summary, `R10` manifest, `R11` Markdown, `R12` tables, `R13`
scientific figures, `R14` figure validity, `R15` format/selection semantics,
`R16` numeric determinism, `R17` failure publication, `R18` immutability,
`R19` repeatability, `R20` scale, `R21` literal non-circular fixtures,
`R22` public error reachability, and `R23` sealed fixture materialization.
Acceptance criteria AC01–AC73 map one-to-one
to the corresponding inventory
row, named owner target, fixture/input, expected status/error, and
falsification purpose.  Thus unmapped requirements = 0, unmapped criteria =
0, tests without owner = 0, tests without expected result = 0, and orphan
tests = 0.

Two independent conforming implementers have no material choice on error name
or transport, public reachability, parser ownership, catalog reader, Phase-C
scope compatibility, acquisition-family presentation, public-summary/manifest
field shape, availability tokens, D-TBL-04 row/order/candidate universe,
ComparableWithWarnings behavior, Nyquist sign, baseline unit, transient fit
uniqueness, numeric spelling, fixture paths/bytes/identities/provenance,
malformed-catalog errors, test names/count, or publication failure. Material
disagreement axes = 0.

The second independent-review findings are closed in the plan as follows:
PD-RR-P1-01 by the public reachability proof and AC67; PD-RR-P1-02 by the
verbatim Phase-C gate and AC70; PD-RR-P1-03 by the tagged catalog variants in
18.5–18.6 and AC72; PD-RR-P1-04 by D-TBL-04 and AC71; and PD-RR-P1-05 by the
sealed ledger in 18.11.2 and AC73. PD-P1-06 is unchanged by 18.9. The final
planning audit requires
zero inaccessible public payloads, error transport ambiguities, new
compatibility theories, family-equality gates, undefined type references or
tokens, availability/lineage/manifest ambiguity, D-TBL-04 contradictions,
ComparableWithWarnings ambiguity, fixture field/identity/provenance/hash
omissions, malformed-catalog ambiguity, duplicate names, traceability gaps,
or implementation inventions. This author does not approve, certify, or
self-review the remediation; independent planning re-review remains required.

The required two-implementer and invention audit has these predetermined
answers: error visibility/path = `reporting::PublicReportError`; error payload
and conversion = `RunnerError::PublicReport(#[from])`; scope gate = the three
Phase-C comparisons in 18.4; acquisition families = projection only; summary
and manifest = the closed graphs in 18.5–18.6; availability/lineage = their
listed enums and null rules; D-TBL-04 = root plus direct-dependency tagged
rows in 18.10; catalog-only nodes = never; `ComparableWithWarnings` = render
with the stated manifest/Markdown warning in both figure and table; fixture
identity/provenance = the sealed bytes and canonical hash recipe in 18.11.2;
malformed bytes/errors = the two code blocks in 18.11.2; and the mandatory
test count = 73. Two conforming implementers therefore have zero
material disagreements and zero implementation inventions. Any discovered
choice outside those answers is a planning defect and blocks implementation.
