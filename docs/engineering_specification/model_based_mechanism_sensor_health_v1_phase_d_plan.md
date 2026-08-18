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
| `src/reporting/mod.rs` | private façade; exports `render_public_report` and `ReportingError` only |
| `src/reporting/error.rs` | `ReportingError` variants in section 12 |
| `src/reporting/reader.rs` | `ReportInputs::read`; canonical `domain::read_artifact` calls and schema/compatibility gates only |
| `src/reporting/projection.rs` | immutable `PublicReportProjection::from_inputs`; field copies, fixed ordering, and no numeric/statistical operation beyond safe textual formatting |
| `src/reporting/claims.rs` | total functions `mechanism_level_text`, `causal_status_text`, `health_status_text`, `evidence_state_text`, `unavailable_text`; contains no thresholds or branching that changes a result |
| `src/reporting/tables.rs` | seven named CSV writers in section 7 |
| `src/reporting/document.rs` | `write_markdown_report` and `write_public_summary_json` |
| `src/reporting/figures.rs` | eleven figure dispatchers in section 8; takes only `PublicReportProjection` prepared series and uses no analysis module |
| `src/reporting/lineage.rs` | `project_lineage`; copies each serialized root, direct dependency, and supplied catalog node without traversal, resolution, or identity construction |
| `src/report_config.rs` | clap-neutral `ReportFormat`, `ReportSelection`, `ReportRenderOptions`; selection parsing and validation only |
| `src/runners/report.rs` | resolves output directory, calls reader → projection → writers, and performs preflight collision checks |
| `src/runners/mod.rs` | adds `pub mod report;` and `RunnerError::Reporting` conversion only |
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

All reads use `domain::read_artifact`; direct `fs::read`, `serde_json` parsing,
physical-file readers, and raw `plot_config` readers are prohibited in
`src/reporting`.

| CLI flag | ArtifactKind / accepted schema | required | reader and consumer | fields rendered | legacy / absent behaviour |
|---|---|---:|---|---|---|
| `--mechanism` | `mechanism_analysis`, schema 4 | yes | `read_artifact::<MechanismAnalysisReport>` → mechanism section, tables D-TBL-01/03/05, D-FIG-01 | `analysis_id`, `hypothesis_assessments`, `hypothesis_history`, timescales, comparisons, records, warnings, provenance, lineage | schemas 1–3: render only `legacy_hypotheses`, timescales, comparisons, trends, warnings as `legacy mechanism artifact; Phase B V1 hypothesis assessment unavailable`; no inferred Phase B row |
| `--health` | `health_assessment`, schema 4 or 3 | yes | `read_artifact::<SensorHealthAssessment>` → health section, D-TBL-02/03/07, D-FIG-02/03 | identity, overall status, Phase C dimensions/evidence bundle, legacy fields, baseline comparisons, warnings, provenance, lineage | schema 3: exact banner `Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized.`; do not manufacture a dimension row; schemas 1/2 rejected as `UnsupportedSchema` |
| `--lineage-catalog` | `ArtifactLineageCatalog`, schema 1 | no | typed JSON reader with explicit schema 1 check → D-TBL-04/D-FIG-11 | supplied nodes as serialized, plus root direct dependencies | absent: project each root's direct lineage only, with `catalog_not_supplied`; `LegacyUnknown` remains explicit; never resolve or traverse the catalog |
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

The output directory is required and must be either absent or empty unless
`--overwrite` is supplied.  A preflight determines every requested path before
the first file is written.  With `--overwrite`, only the exact paths in the
following table may be replaced; unrelated directory entries are untouched.

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
| one of the two required artifacts absent | `ReportingError::MissingRequiredInput { flag }` |
| only one calibration pair flag supplied | `ReportingError::IncompatibleInputCombination { detail: "--calibration and --calibration-observations must be supplied together" }` |
| requested EIS/transient/calibration/signal/estimation/model figure without its source | no command error; selected figure status is `unavailable` with `required input not supplied` |
| a selected table requires optional source that was not supplied | CSV is written with the fixed `not_provided` availability row |
| an input has a wrong `ArtifactKind` | propagated `ArtifactError::IncompatibleKind` |
| an input has unsupported schema | `ReportingError::UnsupportedSchema { path, artifact_kind, schema_version }` |
| `--format` has a value outside `all|json|markdown` | `ReportingError::UnsupportedOutputFormat { value }` (the clap value parser maps its invalid-value diagnostic to this typed command error) |
| selected figure/table ID unknown or duplicate | `ReportingError::InvalidSelection { value }` |
| output path exists without `--overwrite`, or output dir is a file | `ReportingError::OutputCollision` / `InvalidOutputDirectory` |
| writer or plot backend fails | `ReportingError::Write` / `PlotBackend`, with path and source |

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
| D-FIG-03 Current versus baseline `current_vs_baseline` | Shows serialized current/baseline comparison without reclassification | x: feature, faceted strictly by matching unit; y: stored value in that unit; current and baseline series | no error bars unless a stored uncertainty field exists (none in `BaselineComparison`, therefore none); comparability/NA marker; **threshold lines: none**; unavailable if no comparable finite pair |
| D-FIG-04 EIS Nyquist `eis_nyquist` | Shows observed and fitted impedance correspondence | x: `z_real_ohm` (Ohm); y: `-z_imag_ohm` (Ohm), where the sign is a fixed Nyquist display convention stated in caption; observed and serialized fitted series | parameter-at-bound and non-identifiable warning marker; **threshold lines: none**; unavailable for mismatched/non-finite arrays |
| D-FIG-05 EIS Bode `eis_bode` | Shows serialized impedance magnitude/phase versus frequency | x: serialized positive `frequency_hz`, logarithmic display axis; y panels: serialized source-measured magnitude/phase if present, otherwise serialized `derived_*`, explicitly captioned; fitted magnitude/phase | no values are derived; source-null points shown as missing markers; **threshold lines: none**; unavailable if no positive finite frequency or no matching series |
| D-FIG-06 Transient selected-fit response `transient_selected_fit_response` | Shows observed response, selected serialized fitted response, and serialized residuals for each selected converged event | x: local time (s); y panels: potential (V), residual (V); observed, selected fitted, residual series | event index/model/status/CI availability; **threshold lines: none**; unavailable per event when no selected converged fit or unequal serialized arrays; other events still render |
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
   across timestamps.  Invalid/non-finite values should be impossible after
   canonical reader validation; if encountered in an optional series, its
   figure is unavailable and the report records `invalid serialized value`.
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

## 12. Error and atomicity contract

`ReportingError` must contain exactly these variants: `MissingRequiredInput`,
`IncompatibleInputCombination`, `UnsupportedSchema`, `InvalidSelection`,
`UnsupportedOutputFormat`, `InvalidOutputDirectory`, `OutputCollision`, `Artifact { path, source:
ArtifactError }`, `Write { path, source: io::Error }`, `Csv { path, source: csv::Error }`, `Serialization { path,
source: serde_json::Error }`, and `PlotBackend { figure_id, path, message }`.

The runner validates all input paths, readers, selection IDs, compatibility,
and target collisions before creating `OUTPUT`.  It writes each output to an
exact sibling temporary file in `OUTPUT/.phase-d-tmp-<filename>` and renames
only after that writer succeeds.  A failed writer leaves no final file for
that output; already completed selected outputs remain intact and are reported
in the error's `completed_paths` context.  It must never modify input
artifacts.  A backend failure does not re-run analysis; the render manifest is
written only if its own selected output succeeded.

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

## 14. Mandatory implementation test plan

Phase D implementation must add `tests/phase_d_reporting_public_output.rs`
and only the fixtures named below under `tests/fixtures/phase_d/`.  The exact
mandatory total is **48 tests**; names are unique and each is an integration
test unless marked unit.  Semantic assertions are required in addition to any
golden comparison.

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

## 15. Requirements and acceptance traceability

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

There are 16 requirements, 16 mapped acceptance criteria, 48 mapped mandatory
tests, **0 unmapped requirements, 0 unmapped acceptance criteria, and 0
orphan mandatory tests**.

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

Two independent implementers have no material decision left open: the
canonical inputs, supported schemas, required/optional flags, output paths,
schema, table/figure IDs, figure purpose, CLI route, selection behavior,
ordering, DQI/Indeterminate/legacy presentation, language policy, A1
lineage policy, errors, and all 48 test names are fixed above.  Required
implementation inventions: **0**.

## 17. Completion checklist for the Phase D implementation branch

- [ ] Production changes conform exactly to sections 1–16 and do not edit
      Phase A0/A1/B/C semantics.
- [ ] All 48 named tests exist exactly once and are substantive.
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

Planning readiness audit: unspecified outputs 0; unspecified paths 0;
unspecified figures 0; unspecified tables 0; unspecified CLI behavior 0;
unspecified legacy behavior 0; scientific-recalculation ambiguity 0;
claim-language ambiguity 0; lineage ambiguity 0; test ambiguity 0;
implementation inventions required 0.
