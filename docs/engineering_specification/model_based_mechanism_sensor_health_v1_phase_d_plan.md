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
| PD-P1-05 | 48 named tests did not substantively cover manifest, D-TBL-04/05, Bode, compatibility, catalog reading, publication, numeric format, or all figure defects; fixture labels were conceptual | `tests/fixtures` uses literal directory/file contracts | the reviewed predecessor expanded this to 66; this remediation replaces it with the 71-test literal fixture contract in 18.11–18.12 |
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

PublicInputReferenceV1 {
  input_flag: InputFlagV1,
  supplied_path_basename: Option<String>, artifact_kind: Option<ArtifactKindV1>,
  schema_version: Option<u32>, lineage: LineagePresentationV1,
  acquisition_families: AcquisitionFamilyPresentationV1,
  availability: AvailabilityV1
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
| `ArtifactKindV1` | `eis_fit`, `transient_analysis`, `calibration_observations`, `calibration_model`, `calibration_analysis`, `signal_analysis`, `health_baseline`, `health_assessment`, `health_trend`, `mechanism_analysis`, `state_estimation`, `ism_model_compilation`, `ism_model_analysis`, `ism_model_validation`, `artifact_lineage_catalog` | complete `ArtifactKind` vocabulary plus catalog contract |
| `AvailabilityV1` | `available`, `available_with_warnings`, `not_provided`, `not_selected`, `unavailable` | reader/projection/selection outcome; only the first two permit populated source-derived detail |
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
has no `ArtifactIdentity`, no `artifact_kind`, no lineage root, no dependency
registration, and cannot substitute for A1 lineage.  Its complete closed type
graph is:

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

ManifestInputReferenceV1 { input_flag: InputFlagV1,
  artifact_kind: Option<ArtifactKindV1>, schema_version: Option<u32>,
  lineage: LineagePresentationV1,
  acquisition_families: AcquisitionFamilyPresentationV1,
  availability: AvailabilityV1, compatibility: CompatibilityStatusV1 }
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

`input_references` use the fixed input-flag order. `formats`, `figures`, and
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
complete normative fixture contract is section 18.11.1.

| fixture set / exact files | literal relevant content / purpose |
|---|---|
| `current/` — `mechanism.json`, `health.json`, `eis.json`, `transient.json`, `calibration.json`, `calibration_observations.json`, `signal.json`, `estimation.json`, `model.json`, `lineage_catalog.json` | all current schemas; every known identity has experiment `Single(exp-alpha)`, sensor `Specific(sensor-A)`, channel `Specific(potential-V)`, families `Known([eis_sweep,transient_step])`; mechanism `analysis_id=mech-current`, hypothesis `h-transport`, evidence level `experimentally_supported`, comparison `cmp-01` with serialized `log10_distance=0.041`; health `assessment_id=health-current`, all nine dimensions in `HealthDimension::ALL`, feature `{name:"slope_v_per_decade",value:0.058,unit:"V/decade"}`, comparison `{feature:"slope_v_per_decade",current_value:0.058,baseline_value:0.059,comparability:comparable}`; EIS frequency `[1,10]`, real `[10,5]`, imag `[-2,-1]`, fitted same; transient event 0 selects one converged `Exponential`, raw time `[0,1]`, raw V `[0.10,0.20]`, fitted time `[0,1]`, predicted `[0.11,0.19]`, residual `[-0.01,0.01]`; catalog contains the ten corresponding IDs in lexical map order. |
| `legacy/health_schema3.json`, `legacy/mechanism_schema3.json`, `legacy/unknown_lineage.json` | valid schema-3 health with `phase_c` absent; valid schema-3 mechanism with `hypothesis_assessments=[]`; each lineage is `LegacyUnknown { source_schema_version: 3, reason: FieldAbsentInLegacyArtifact }`. |
| `edge/baseline_no_unit.json`, `edge/baseline_duplicate_unit.json` | clone `current/health.json`; respectively zero matching `features.name` and two matching features with units `V/decade` and `mV/decade`; comparison remains literal. |
| `edge/transient_zero_match.json`, `edge/transient_duplicate_match.json` | clone current transient; respectively selected model has no converged candidate and has exactly two converged `Exponential` candidates with different literal predicted series `[0.11,0.19]` and `[0.12,0.18]`. |
| `edge/eis_bode.json`, `edge/eis_nyquist_sign.json` | EIS Bode adds source magnitude `[10.198...,5.099...]`, phase `[-11.309..., -11.309...]`, fitted magnitude/phase literal arrays; Nyquist uses the current negative serialized imag values to prove no sign change. |
| `edge/incompatible_sensor.json`, `edge/incompatible_experiment.json`, `edge/incompatible_optional.json` | clone the named current artifact changing only `sensor_scope=Specific(sensor-B)`, `experiment_scope=Single(exp-beta)`, or optional EIS `channel_scope=Specific(other-channel)` respectively. |
| reviewed scope/legacy examples | retracted: section 18.4 now reuses Phase-C admissibility exactly and section 18.11.1 provides the normative mutation records. |
| `edge/catalog_schema2.json`, `edge/catalog_bad_key.json`, `edge/catalog_duplicate_key.json`, `edge/catalog_malformed.json` | schema 2; schema 1 with key `sha256:` ID different from node identity; schema 1 raw JSON text containing the same artifact map key twice; and text `{not-json}`. |
| `edge/numeric_values.json` | valid source values `0.0`, `-0.0`, `0.000001`, `100000000000000000000.0`, `1.25`, and threshold `0.041`; expected formatted values are produced by section 18.9, not a renderer golden. |
| `edge/dqi_health.json`, `edge/indeterminate_health.json`, `edge/signal_missing.json`, `edge/model_missing.json`, `edge/large_history.json` | each is a literal clone of `current` changing only: first `data_quality` dimension to `data_quality_insufficient` with reason `required_quantity_absent`; second `observability` to `indeterminate` with `insufficient_evidence`; third `analysis_values=[0.10,null,0.20]`; fourth one model point's `observed_voltage_v` and `unexplained_residual_v` to null; fifth mechanism `hypothesis_history` to exactly 1,000 entries `history-0000` through `history-0999` and Phase-C evidence records to exactly 10,000 IDs `evidence-00000` through `evidence-09999`, each otherwise the same valid typed value. |
| `failure/write_denied/`, `failure/unmanaged_output/` | a test-only injected writer returns `io::ErrorKind::PermissionDenied` for staged `tables/mechanism_evidence.csv`; unmanaged output contains literal `keep.txt` with `do not delete`. |

The preceding matrix is superseded by section 18.11.1 and creates no
implementation permission.

#### 18.11.1 Normative literal fixture capsules

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

### 18.12 Mandatory test inventory — exactly 71 unique tests

All tests live in `tests/phase_d_reporting_public_output.rs` unless marked
unit; a test name appears once.  `R` is the requirement in section 18.13,
`AC` is its acceptance criterion, and each cell states target/fixture/expected
falsification result.  Status `ok` means successful complete publication;
`err(X)` means exact `PublicReportError::X` unless explicitly `CliError::Parse`.

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
| 20 | `phase_d_optional_legacy_unknown_is_limited_not_inferred` integration | R07/AC20; compatibility | legacy_optional EIS → `ok`, compatibility `legacy_unknown` |
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

### 18.13 Traceability, two-implementer audit, and readiness

There are exactly **22 requirements**, **71 acceptance criteria**, and **71
mandatory tests**.  Requirement IDs are `D-R01` route/parser, `R02` selection,
`R03` atomic output, `R04` artifact readers, `R05` catalog reader, `R06`
required compatibility, `R07` optional compatibility, `R08` legacy projection,
`R09` public summary, `R10` manifest, `R11` Markdown, `R12` tables, `R13`
scientific figures, `R14` figure validity, `R15` format/selection semantics,
`R16` numeric determinism, `R17` failure publication, `R18` immutability,
`R19` repeatability, `R20` scale, `R21` literal non-circular fixtures, and
`R22` public error reachability. Acceptance criteria AC01–AC71 map one-to-one
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
verbatim Phase-C gate and AC70; PD-RR-P1-03 by the closed graphs/enums in
18.5–18.6; PD-RR-P1-04 by D-TBL-04 and AC71; and PD-RR-P1-05 by 18.11.1 plus
AC68–AC69. PD-P1-06 is unchanged by 18.9. The final planning audit requires
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
identity/provenance = the literal manifest and canonical hash recipe in
18.11.1; malformed bytes/errors = the two code blocks in 18.11.1; and the
mandatory test count = 71. Two conforming implementers therefore have zero
material disagreements and zero implementation inventions. Any discovered
choice outside those answers is a planning defect and blocks implementation.
