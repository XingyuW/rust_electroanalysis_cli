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
| `src/reporting/mod.rs` | crate-private façade; exposes `render_public_report` and `PublicReportError` only within this crate |
| `src/reporting/error.rs` | crate-private `PublicReportError` and the Phase-D-only error variants frozen in section 18.2; it must not define `ReportingError` |
| `src/reporting/reader.rs` | `ReportInputs::read`; canonical `domain::read_artifact` calls and schema/compatibility gates only |
| `src/reporting/projection.rs` | immutable `PublicReportProjection::from_inputs`; field copies, fixed ordering, and no numeric/statistical operation beyond safe textual formatting |
| `src/reporting/claims.rs` | total functions `mechanism_level_text`, `causal_status_text`, `health_status_text`, `evidence_state_text`, `unavailable_text`; contains no thresholds or branching that changes a result |
| `src/reporting/tables.rs` | seven named CSV writers in section 7 |
| `src/reporting/document.rs` | `write_markdown_report` and `write_public_summary_json` |
| `src/reporting/figures.rs` | eleven figure dispatchers in section 8; takes only `PublicReportProjection` prepared series and uses no analysis module |
| `src/reporting/lineage.rs` | `project_lineage`; copies each serialized root, direct dependency, and supplied catalog node without traversal, resolution, or identity construction |
| `src/report_config.rs` | clap-neutral `ReportFormat`, `ReportSelection`, `ReportRenderOptions`; selection parsing and validation only |
| `src/runners/report.rs` | resolves output directory, calls reader → projection → writers, and performs preflight collision checks |
| `src/runners/mod.rs` | adds `pub mod report;` and a distinct `RunnerError::PublicReport(#[from] PublicReportError)` conversion; the existing `RunnerError::Reporting(#[from] domain::ReportingError)` remains unchanged |
| `src/cli.rs` | adds `Command::Report`, `ReportCommand::Render`, `ReportRenderCommand`, and `CommandSpec::ReportRender` exactly as section 6 |
| `src/main.rs` | one `CommandSpec::ReportRender` match arm; no alternative dispatch |
| `src/lib.rs` | exposes `reporting` only if existing public API conventions require it; otherwise keeps it crate-private |

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
| `--lineage-catalog` | `ArtifactLineageCatalog`, schema 1 (not an `ArtifactKind`) | no | `domain::read_artifact_lineage_catalog` → D-TBL-04/D-FIG-11 | supplied nodes as serialized, plus root direct dependencies | absent: project each root's direct lineage only, with `catalog_not_supplied`; `LegacyUnknown` remains explicit; never resolve or traverse the catalog |
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

The JSON summary has exactly these top-level fields, in this order:

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
| D-TBL-04 `artifact_lineage.csv` | `root_scope,root_artifact_kind,root_artifact_id,lineage_state,direct_dependency_role,direct_dependency_kind,direct_dependency_id,catalog_supplied,catalog_entry_present` | source-artifact order, then existing canonical dependency order | A1 lineage projection only; no traversal, resolution, or inferred missing dependency |
| D-TBL-05 `timescale_comparison.csv` | `comparison_id,record_id,eis_timescale_id,eis_value_s,eis_standard_error_s,transient_timescale_id,transient_value_s,transient_standard_error_s,ratio,log10_distance,symmetric_relative_difference,confidence_interval_overlap,compatibility_probability,evidence_level,supporting_evidence,contradictory_evidence,alternative_explanations,warnings` | `comparison_id` lexical | serialized Phase B timescale values and limitations |
| D-TBL-06 `model_consistency.csv` | `availability,time_s,observed_voltage_v,predicted_voltage_v,unexplained_residual_v,uncertainty_status,validity_status,equilibrium_status` | `time_s`, preserving equal-time source order | model observed/predicted evidence only; no residual recomputation or binary evidence conversion |
| D-TBL-07 `current_vs_baseline.csv` | `availability,feature,unit,current_value,baseline_value,comparability,absolute_difference,relative_difference,log_ratio,z_score,robust_z_score,empirical_percentile,baseline_sample_count,override_reason` | feature lexical then unit lexical | serialized health comparison; neither ranking nor new status is calculated |

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
| D-FIG-03 Current versus baseline `current_vs_baseline` | Shows serialized current/baseline comparison without reclassification | x: `BaselineComparison.feature`; y: a pair is plotted only when exactly one `SensorHealthAssessment.features` entry has the same `name` and a nonempty `unit`; its `unit` is the y-unit and the pair's `current_value` / `baseline_value` are the series values | no error bars unless a stored uncertainty field exists (none in `BaselineComparison`, therefore none); zero or multiple matching feature-unit entries make that pair unavailable with `unit_authority_unavailable`; **threshold lines: none**; unavailable if no comparable finite pair |
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
| PD-P1-05 | 48 named tests did not substantively cover manifest, D-TBL-04/05, Bode, compatibility, catalog reading, publication, numeric format, or all figure defects; fixture labels were conceptual | `tests/fixtures` uses literal directory/file contracts | a 66-test inventory and literal fixture matrix |
| PD-P1-06 | section 10 said `NA` but left finite-number and cross-format representation unspecified | artifact readers reject non-finite numeric source data before projection | one exact finite formatter and serialization rules |

The Phase-D projection-only boundary in sections 1, 2, 7–11 is retained.  A
Phase-D transformation is permitted only when this section expressly calls it
presentation formatting or display geometry; it must not alter a scientific
coordinate, select an assessment, calculate an absent quantity, infer
compatibility, or create new evidence.

### 18.2 One error architecture and normative boundary matrix

`domain::ReportingError` remains public, re-exported, and **unchanged**.  It
continues to cover pre-existing fit/search reporting only.  Phase D adds the
crate-private type `reporting::error::PublicReportError`:

```text
pub(crate) enum PublicReportError {
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

`CompatibilityAxis` is exactly `experiment_scope`, `sensor_scope`,
`channel_scope`, or `acquisition_families`; `PublicationPhase` is exactly
`backup_rename`, `publish_rename`, `restore_rename`, or `backup_cleanup`.
`AvailabilityReason` is the closed enum in section 18.8.  `RunnerError` gains
exactly `PublicReport(#[from] PublicReportError)`; its existing
`Reporting(#[from] domain::ReportingError)` variant stays unchanged.
`ApplicationError` continues to receive this through its existing
`Runner(#[from] RunnerError)` conversion.  No public API gains a second type
called `ReportingError`.

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

Compatibility does not reassess science.  It checks whether a report may be
certified as one coherent input bundle, using only `ArtifactLineageState`.
For two `Known` identities, `Compatible` requires all of: equal
`ArtifactExperimentScope` (a `Single` never matches an `Aggregate`, even if
its ID is a member), equal `ScopeKey` sensor scope, equal `ScopeKey` channel
scope, and equal normalized `ArtifactAcquisitionFamilies`.  This is the
conservative equality form of Phase C's existing known-lineage
`scope_compatible` check; it adds no physical or statistical rule.  A known
`Unknown` experiment scope, `Unspecified` sensor/channel scope, or unknown
family set yields `NotVerifiable`, not a fabricated match.  A
`LegacyUnknown` lineage likewise yields `LegacyUnknown`, never `Compatible`.

| pair | known mismatch | `NotVerifiable` / `LegacyUnknown` | supplied but not selected |
|---|---|---|---|
| mechanism ↔ health | fail `RequiredInputsIncompatible` | render independent sections; `input_compatibility=not_verifiable` or `legacy_unknown`; no cross-artifact claim, comparison, or inferred relation | n/a |
| EIS, transient, signal, estimation, model ↔ each required input | fail `OptionalInputIncompatible` | project only that artifact's own fields; manifest/summary carry the exact state and limitation | still validate and record; never silently ignore a mismatch |
| calibration ↔ calibration-observations, then each ↔ each required input | fail `OptionalInputIncompatible` | paired figure/table is unavailable with the exact state; other independent optional projections may remain eligible | still validate and record |
| supplied catalog ↔ any artifact | not applicable | catalog may contain entries outside this bundle; only supplied direct nodes are projected | same |

`CompatibilityState` tokens are exactly `compatible`, `not_verifiable`,
`legacy_unknown`, `not_applicable`, and `not_provided`.  `NotVerifiable` and
`LegacyUnknown` are never described as compatible; they are limitation states.
All supplied optional inputs are read and checked before selection.  A known
mismatch aborts the render, even if no output that uses the input was selected.

### 18.5 Closed `public_summary.schema1.json`

`public_summary.schema1.json` is UTF-8 LF pretty JSON emitted by one typed
serializer in declaration order.  Its complete top-level structure is:

```text
PublicSummaryV1 {
  schema_version: 1,
  output_kind: "phase_d_public_scientific_output",
  renderer_contract: "mhi_v1_phase_d_public_output_v1",
  route: "electroanalysis report render",
  input_references: Vec<PublicInputReference>,
  compatibility: PublicCompatibility,
  mechanism: PublicMechanismSection,
  sensor_health: PublicHealthSection,
  optional_sources: PublicOptionalSources,
  lineage: PublicLineageSection,
  outputs: PublicOutputIndex,
  limitations: Vec<PublicLimitation>,
  rendering: PublicRenderingMetadata
}
```

Every field is required and non-null unless `Option<T>` is written below.
`Vec` is `[]` when empty; fields are never omitted; a source absence is an
availability enum, not a missing key.  Display prose uses section 9 mappings;
scientific numeric values remain JSON numbers.  The only embedded upstream
payloads are the named, typed `Copy<T>` fields below; each is copied unchanged
from its validated source type, is not a free-form JSON value/map, and has the
same upstream JSON shape.

| type / exact fields | source authority and ordering |
|---|---|
| `PublicInputReference { input_flag, supplied_path_basename: Option<String>, artifact_kind: Option<String>, schema_version: Option<u32>, lineage: PublicLineageState, compatibility: CompatibilityState, availability }` | flag order from section 18.4; basename only, never an absolute path; known identity copied, legacy reason copied |
| `PublicCompatibility { required_pair: CompatibilityState, optional: Vec<PublicOptionalCompatibility> }`; item `{ input_flag, against_flag, state, axis: Option<CompatibilityAxis>, detail: Option<String> }` | canonical flag order then required target order; state is reader result only |
| `PublicMechanismSection { availability, analysis_id: Option<String>, hypotheses: Vec<PublicHypothesis>, legacy_hypotheses: Vec<Copy<HypothesisAssessment>>, timescales: PublicTimescales, comparisons: Vec<Copy<TimescaleComparison>>, warnings: Vec<Copy<MechanismWarning>>, lineage }` | current artifact fields; current hypotheses sort by `definition.hypothesis_id`; legacy array retains source order; no evidence level recomputation |
| `PublicHypothesis { definition: Copy<MechanismHypothesisDefinition>, assessment: Copy<PhaseBHypothesisAssessment> }` | each is the corresponding `HypothesisAssessmentRecord.definition/current`, copied unchanged; `definition.hypothesis_id` order |
| `PublicTimescales { eis: Vec<Copy<CharacteristicTimescale>>, transient: Vec<Copy<CharacteristicTimescale>> }` | `MechanismAnalysisReport.eis_timescales` and `transient_timescales`, source order |
| `PublicHealthSection { availability, assessment_id: Option<String>, sensor_id: Option<String>, experiment_id: Option<String>, overall_status: Option<OverallHealthStatus>, dimensions: Vec<Copy<PhaseCHealthDimensionAssessment>>, phase_c_summary: Option<PublicPhaseCSummary>, features: Vec<Copy<HealthFeature>>, baseline_comparisons: Vec<Copy<BaselineComparison>>, warnings: Vec<Copy<HealthWarning>>, lineage }` | schema-4 dimensions exactly `HealthDimension::ALL`; schema-3 uses `dimensions=[]`, `phase_c_summary=null`, and `availability=legacy_phase_c_not_serialized` |
| `PublicPhaseCSummary { config_schema_version, config_sha256, overall_status, overall_interpretation_categories: Vec<HealthInterpretationCategory>, overall_causal_status, evidence_bundle: Copy<EvidenceBundle> }` | exact `SensorHealthAssessment.phase_c` fields; nested bundle is sealed upstream typed content |
| `PublicOptionalSources { eis: PublicOptional<EisPublicProjection>, transient: PublicOptional<TransientPublicProjection>, calibration: PublicOptional<CalibrationPublicProjection>, signal: PublicOptional<SignalPublicProjection>, estimation: PublicOptional<EstimationPublicProjection>, model: PublicOptional<ModelPublicProjection>, lineage_catalog: PublicOptional<CatalogPublicProjection> }` | fixed member order; each wrapper is `{ availability, compatibility, reason: Option<AvailabilityReason>, value: Option<T> }`; `value` is null unless availability is `projected` |
| `EisPublicProjection { fit_id, source: Copy<EisSourceData>, fitted: Copy<EisFittedData>, parameters: Vec<Copy<EisFittedParameter>>, statistics: Copy<EisFitStatistics>, confidence_intervals: Vec<Copy<EisParameterConfidenceInterval>>, diagnostics: Copy<EisFitDiagnostics>, warnings: Vec<Copy<EisFitWarning>>, lineage }` | exactly the named `EisFitArtifact` fields, source order |
| `TransientPublicProjection { experiment_id, channel, channel_unit, events: Vec<PublicTransientEvent>, lineage }`; event `{ event_index, selected_model, segment: Copy<SegmentSummary>, candidate_fits: Vec<Copy<TransientFitResult>>, failure: Option<Copy<TransientFitFailure>>, warnings: Vec<Copy<TransientWarning>> }` | exact `TransientAnalysisReport` and `TransientEventResult` fields; source event/candidate order; selected-fit availability is representation validation, not a replacement value |
| `CalibrationPublicProjection { observations: Vec<Copy<CalibrationObservation>>, analysis: PublicCalibrationAnalysis, observation_warnings, analysis_warnings, lineage }`; analysis `{ calibration_id, analyte, ion_charge, source_experiments, selected_model, candidate_models: Vec<Copy<CalibrationModelResult>>, validation: Option<Copy<CalibrationValidationResult>> }` | paired source fields; observations source order, candidates source order |
| `SignalPublicProjection { analysis_id, experiment_id, sensor_id, channel, unit, analysis_timestamps, analysis_values, psd: Option<Copy<PsdAnalysis>>, allan: Option<Copy<AllanAnalysis>>, spikes: Copy<SpikeAnalysis>, warnings, lineage }` | exact named `SignalAnalysisReport` fields and source order |
| `EstimationPublicProjection { analysis_id, experiment_id, sensor_id, channel, measurement_source_unit, estimates: Vec<Copy<StateEstimatePoint>>, observability: Copy<ObservabilityReport>, diagnostics: Copy<FilterDiagnostics>, warnings, lineage }` | exact named `StateEstimationReport` fields, estimate source order |
| `ModelPublicProjection { model_definition: Copy<ModelDefinition>, points: Vec<Copy<ModelAnalysisPoint>>, identifiability: Copy<IdentifiabilityReport>, evidence: Vec<String>, lineage }` | exact `ModelAnalysisReport` fields, point/evidence source order |
| `CatalogPublicProjection { schema_version: 1, nodes: Vec<PublicCatalogNode> }`; node `{ artifact_id, identity: Copy<ArtifactIdentity>, direct_dependencies: Vec<Copy<ArtifactDependency>> }` | catalog BTreeMap ascending `artifact_id`; node dependency canonical order; no traversal |
| `PublicLineageSection { roots: Vec<PublicLineageRoot>, catalog_supplied }`; root `{ input_flag, lineage: PublicLineageState, direct_dependencies: Vec<Copy<ArtifactDependency>>, catalog_entry_present: Option<bool> }` | flag order; direct dependency order copied; `PublicLineageState` is `known { identity }` or `legacy_unknown { source_schema_version, reason }` |
| `PublicOutputIndex { tables: Vec<PublicOutputStatus>, figures: Vec<PublicOutputStatus> }`; status `{ id, relative_path: Option<String>, format: Option<String>, status, reason: Option<AvailabilityReason> }` | table/figure contract order; no absolute path |
| `PublicLimitation { code, message, input_flag: Option<String>, output_id: Option<String> }` and `PublicRenderingMetadata { json_schema: "public_summary.schema1", numeric_format: "rust_display_normalized_negative_zero_v1", csv_newline: "LF", timestamp: null }` | fixed sort `(input_flag, output_id, code)` with `None` last; no clock/host/process path |

The fixed `availability` tokens are `projected`, `not_provided`,
`legacy_phase_c_not_serialized`, `legacy_mechanism_assessment_not_serialized`,
`not_verifiable`, `legacy_unknown`, and `unavailable`.  This is the full
schema; no serializer may add a field, omit a field, or use `serde_json::Value`.

### 18.6 Closed `render_manifest.schema1.json`

The render manifest is presentation provenance, not a scientific artifact: it
has no `ArtifactIdentity`, no `artifact_kind`, no lineage root, no dependency
registration, and cannot substitute for A1 lineage.  Its closed schema is:

```text
RenderManifestV1 {
  schema_version: 1,
  output_kind: "phase_d_render_manifest",
  renderer_contract: "mhi_v1_phase_d_public_output_v1",
  route: "electroanalysis report render",
  final_output_status: "published",
  input_references: Vec<ManifestInputReference>,
  requested: RequestedOutputSelection,
  render_order: Vec<ManifestRenderStep>,
  generated_files: Vec<ManifestGeneratedFile>,
  unavailable_outputs: Vec<ManifestUnavailableOutput>,
  warnings: Vec<ManifestWarning>,
  legacy_input_notices: Vec<ManifestLegacyNotice>,
  optional_compatibility: Vec<ManifestCompatibilityOutcome>,
  determinism: ManifestDeterminism
}
```

`ManifestInputReference` is `PublicInputReference` without
`supplied_path_basename`; `RequestedOutputSelection` is `{ formats: Vec<json|
markdown>, figures: Vec<FigureId>, tables: Vec<TableId>, figures_mode:
default|explicit, tables_mode: default|explicit, overwrite }`; IDs use
section 6 order. `ManifestRenderStep` is `{ ordinal: u32, kind:
summary|markdown|table|figure|manifest, id: Option<String>, relative_path }`.
`ManifestGeneratedFile` is `{ relative_path, output_kind:
summary|markdown|table|figure|manifest, report_id: Option<String>, format:
json|markdown|csv|svg|png, status: "written", source_artifact_ids:
Vec<String> }`.  `ManifestUnavailableOutput` is `{ output_kind, report_id,
reason }`; `ManifestWarning` is `{ code, message, input_flag: Option<String>,
output_id: Option<String> }`; `ManifestLegacyNotice` is `{ input_flag,
schema_version, notice }`; `ManifestCompatibilityOutcome` is `{ input_flag,
against_flag, state, axis: Option<CompatibilityAxis> }`; and
`ManifestDeterminism` is `{ json_object_order: "declaration_order",
array_order: "contract_order", numeric_format:
"rust_display_normalized_negative_zero_v1", csv: "rfc4180_lf_utf8_v1",
path_separator: "/", clock: null }`.

Arrays are ordered exactly as their section 18.5 source order; generated files
are `render_order` order with SVG before PNG for each figure.  All paths are
UTF-8 relative paths using `/`, never `..`, leading `/`, a drive prefix, or a
platform separator.  `null` is used only for fields typed `Option`; all absent
optional source values use their wrapper, not a missing manifest record.

### 18.7 JSON and non-finite determinism

Typed serializers must serialize fields in declaration order.  BTreeMap-backed
catalog nodes are converted to the ordered `nodes` array; no output type may
use a HashMap.  Enum tokens are their existing serde snake-case token unless a
literal token is specified above.  JSON strings use serde_json's standard JSON
escaping and LF pretty indentation of two spaces.  JSON scientific values are
JSON numbers, not formatted strings.  Canonical artifact readers reject NaN,
`Infinity`, `-Infinity`, and non-finite values before projection; therefore a
Phase-D JSON writer never encounters one.  Encountering one in an in-memory
projection is `StagingValidation`, not JSON `null`, a token, omission, or an
alternate numeric value.

### 18.8 Availability, format flags, and selection

`AvailabilityReason` is exactly `not_provided`, `not_selected`,
`legacy_phase_c_not_serialized`, `legacy_mechanism_assessment_not_serialized`,
`compatibility_not_verifiable`, `lineage_legacy_unknown`,
`unit_authority_unavailable`, `no_comparable_finite_pair`,
`selected_fit_not_found`, `selected_fit_ambiguous`, `serialized_series_invalid`,
`serialized_series_unavailable`, `paired_input_not_provided`, and
`catalog_not_supplied`.

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
| D-FIG-03 | x `BaselineComparison.feature`; y current/baseline only when one matching `HealthFeature{name,unit}` supplies nonempty unit | `current`, then `baseline`; feature source order | zero/multiple match = `unit_authority_unavailable`; no unit inference or conversion |
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
uncertainty transforms.  D-TBL-01 through D-TBL-06 retain section 7 headers,
types, source fields, order, and tokens.  D-TBL-04 projects catalog nodes in
ascending artifact ID and direct dependencies in canonical order; missing
ancestor is only `catalog_entry_present=false`, never resolved.  D-TBL-05 uses
only `MechanismAnalysisReport.comparisons` and its referenced serialized
timescales, in lexical `comparison_id` order; it never searches a better pair.
D-TBL-07 gets its `unit` using the same unique `HealthFeature{name,unit}` rule
as D-FIG-03; a non-authoritative pair writes `availability=unit_authority_unavailable`
and `NA` in every numeric/unit cell.  All seven tables use section 18.9 CSV.

### 18.11 Literal fixture contract

Phase D adds only the following future fixture directory; this planning task
does not create it: `tests/fixtures/phase_d/{current,legacy,edge,failure}/`.
Each listed JSON file is a canonical artifact with its exact schema/kind.  Any
field not listed is its type's valid zero/empty/default value: every `Vec` is
`[]`, every `Option` is `null`, every warning is `[]`, provenance uses
`input_path="fixture"`, `configuration_path=null`, and no float is non-finite.
This rule plus the literals below is the complete fixture content contract;
fixtures are hand-authored from these values, never emitted by the renderer.

| fixture set / exact files | literal relevant content / purpose |
|---|---|
| `current/` — `mechanism.json`, `health.json`, `eis.json`, `transient.json`, `calibration.json`, `calibration_observations.json`, `signal.json`, `estimation.json`, `model.json`, `lineage_catalog.json` | all current schemas; every known identity has experiment `Single(exp-alpha)`, sensor `Specific(sensor-A)`, channel `Specific(potential-V)`, families `Known([eis_sweep,transient_step])`; mechanism `analysis_id=mech-current`, hypothesis `h-transport`, evidence level `experimentally_supported`, comparison `cmp-01` with serialized `log10_distance=0.041`; health `assessment_id=health-current`, all nine dimensions in `HealthDimension::ALL`, feature `{name:"slope_v_per_decade",value:0.058,unit:"V/decade"}`, comparison `{feature:"slope_v_per_decade",current_value:0.058,baseline_value:0.059,comparability:comparable}`; EIS frequency `[1,10]`, real `[10,5]`, imag `[-2,-1]`, fitted same; transient event 0 selects one converged `Exponential`, raw time `[0,1]`, raw V `[0.10,0.20]`, fitted time `[0,1]`, predicted `[0.11,0.19]`, residual `[-0.01,0.01]`; catalog contains the ten corresponding IDs in lexical map order. |
| `legacy/health_schema3.json`, `legacy/mechanism_schema3.json`, `legacy/unknown_lineage.json` | valid schema-3 health with `phase_c` absent; valid schema-3 mechanism with `hypothesis_assessments=[]`; each lineage is `LegacyUnknown { source_schema_version: 3, reason: FieldAbsentInLegacyArtifact }`. |
| `edge/baseline_no_unit.json`, `edge/baseline_duplicate_unit.json` | clone `current/health.json`; respectively zero matching `features.name` and two matching features with units `V/decade` and `mV/decade`; comparison remains literal. |
| `edge/transient_zero_match.json`, `edge/transient_duplicate_match.json` | clone current transient; respectively selected model has no converged candidate and has exactly two converged `Exponential` candidates with different literal predicted series `[0.11,0.19]` and `[0.12,0.18]`. |
| `edge/eis_bode.json`, `edge/eis_nyquist_sign.json` | EIS Bode adds source magnitude `[10.198...,5.099...]`, phase `[-11.309..., -11.309...]`, fitted magnitude/phase literal arrays; Nyquist uses the current negative serialized imag values to prove no sign change. |
| `edge/incompatible_sensor.json`, `edge/incompatible_experiment.json`, `edge/incompatible_optional.json` | clone the named current artifact changing only `sensor_scope=Specific(sensor-B)`, `experiment_scope=Single(exp-beta)`, or optional EIS `channel_scope=Specific(other-channel)` respectively. |
| `edge/not_verifiable_known.json`, `edge/legacy_optional.json` | clone current source changing only known identity families to `Unknown` / scope to `Unspecified`, or its whole lineage to the literal LegacyUnknown variant. |
| `edge/catalog_schema2.json`, `edge/catalog_bad_key.json`, `edge/catalog_duplicate_key.json`, `edge/catalog_malformed.json` | schema 2; schema 1 with key `sha256:` ID different from node identity; schema 1 raw JSON text containing the same artifact map key twice; and text `{not-json}`. |
| `edge/numeric_values.json` | valid source values `0.0`, `-0.0`, `0.000001`, `100000000000000000000.0`, `1.25`, and threshold `0.041`; expected formatted values are produced by section 18.9, not a renderer golden. |
| `edge/dqi_health.json`, `edge/indeterminate_health.json`, `edge/signal_missing.json`, `edge/model_missing.json`, `edge/large_history.json` | each is a literal clone of `current` changing only: first `data_quality` dimension to `data_quality_insufficient` with reason `required_quantity_absent`; second `observability` to `indeterminate` with `insufficient_evidence`; third `analysis_values=[0.10,null,0.20]`; fourth one model point's `observed_voltage_v` and `unexplained_residual_v` to null; fifth mechanism `hypothesis_history` to exactly 1,000 entries `history-0000` through `history-0999` and Phase-C evidence records to exactly 10,000 IDs `evidence-00000` through `evidence-09999`, each otherwise the same valid typed value. |
| `failure/write_denied/`, `failure/unmanaged_output/` | a test-only injected writer returns `io::ErrorKind::PermissionDenied` for staged `tables/mechanism_evidence.csv`; unmanaged output contains literal `keep.txt` with `do not delete`. |

`current` is the only complete bundle.  Exceptional fixtures are exact
mutations above; they do not inherit an unspecified value.  Expected public
output is asserted semantically against these literals; a golden may only be a
hand-authored expected header/row/field fragment derived from this table, never
the current renderer's output committed as truth.

### 18.12 Mandatory test inventory — exactly 66 unique tests

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
| 10 | `phase_d_catalog_reader_accepts_schema1_and_canonical_order` integration | R05/AC10; domain catalog reader | current catalog → `ok`, D-TBL-04 ascending IDs |
| 11 | `phase_d_catalog_reader_rejects_schema2` integration | R05/AC11; domain catalog reader | catalog_schema2 → `err(LineageCatalog)` |
| 12 | `phase_d_catalog_reader_rejects_key_identity_mismatch` integration | R05/AC12; domain catalog reader | catalog_bad_key → `err(LineageCatalog)` |
| 13 | `phase_d_catalog_reader_rejects_duplicate_json_key` integration | R05/AC13; domain catalog reader | catalog_duplicate_key → `err(LineageCatalog)` |
| 14 | `phase_d_reporting_never_ad_hoc_parses_catalog` unit | R05/AC14; reporting reader | source-level forbidden-call guard → no `serde_json::from_*` catalog parse |
| 15 | `phase_d_required_known_scope_mismatch_is_rejected` integration | R06/AC15; compatibility | incompatible_sensor health → `err(RequiredInputsIncompatible)` sensor |
| 16 | `phase_d_required_experiment_mismatch_is_rejected` integration | R06/AC16; compatibility | incompatible_experiment mechanism → exact experiment error |
| 17 | `phase_d_required_unknown_scope_is_not_claimed_compatible` integration | R06/AC17; compatibility | not_verifiable_known required → `ok`, summary token `not_verifiable` |
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
| 33 | `phase_d_artifact_lineage_table_projects_known_and_legacy_unknown` integration | R12/AC33; tables | current+legacy → D-TBL-04 columns/order/no traversal |
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

### 18.13 Traceability, two-implementer audit, and readiness

There are exactly **21 requirements**, **66 acceptance criteria**, and **66
mandatory tests**.  Requirement IDs are `D-R01` route/parser, `R02` selection,
`R03` atomic output, `R04` artifact readers, `R05` catalog reader, `R06`
required compatibility, `R07` optional compatibility, `R08` legacy projection,
`R09` public summary, `R10` manifest, `R11` Markdown, `R12` tables, `R13`
scientific figures, `R14` figure validity, `R15` format/selection semantics,
`R16` numeric determinism, `R17` failure publication, `R18` immutability,
`R19` repeatability, `R20` scale, and `R21` literal non-circular fixtures.
Acceptance criteria AC01–AC66 map one-to-one to the corresponding inventory
row, named owner target, fixture/input, expected status/error, and
falsification purpose.  Thus unmapped requirements = 0, unmapped criteria =
0, tests without owner = 0, tests without expected result = 0, and orphan
tests = 0.

Two independent conforming implementers have no material choice on error name
or transport, parser ownership, catalog reader, known/not-verifiable/legacy
compatibility, public-summary/manifest field shape, Nyquist sign, baseline
unit, transient fit uniqueness, numeric spelling, fixture paths/literals,
test names/count, or publication failure.  Material disagreement axes = 0.

Proposed closure evidence for the next independent reviewer is: PD-P1-01 is
addressed by 18.2–18.3; PD-P1-02 by 18.4; PD-P1-03 by 18.5–18.7; PD-P1-04 by
18.10; PD-P1-05 by 18.11–18.13; and PD-P1-06 by 18.7 and 18.9.  The plan
targets zero error-name, parse/runtime, publication, catalog-reader,
canonical-reader-bypass, compatibility, schema-field, unit, figure-source,
scientific-transformation, numeric, test-family, fixture, traceability, and
implementation-invention ambiguities.  This author does not approve, certify,
or self-review the remediation; independent planning re-review remains
required.
