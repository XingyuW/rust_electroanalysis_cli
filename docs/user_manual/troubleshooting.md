# Troubleshooting

[Master manual](../USER_MANUAL.md) · [Configuration](configuration.md) · [Coverage/issues](../USER_MANUAL_COVERAGE.md)

Start with the complete stderr, selected file/config paths, schema versions and current working directory. A successful export can still contain unavailable/failed scientific results. A batch command can write successful results and return a partial-failure error for other files.

| Symptom | Likely cause | How to verify | Fix / safe response |
|---|---|---|---|
| Binary cannot be found under target/release | CARGO_TARGET_DIR changes Cargo output location | Check environment/build output | Use configured target directory; do not relocate build outputs into /tmp |
| Usage says electroanalysis but installed binary has another name | Clap display name differs from Cargo package binary | Cargo.toml and root help | Invoke rust_electroanalysis_cli; usage arguments still apply |
| Requested v0.2.0 cannot be reproduced locally | Wrong source checkout, missing tag, or a dependency/build mismatch | Check `git rev-parse 'v0.2.0^{}'`, Cargo package version and build diagnostics | Fetch the published tag, verify its commit, then build with `--locked` |
| File unrecognized | Content/schema unsupported even if suffix is .csv | Inspect headers and provider diagnostics | Export supported tabular text/CHI/XLSX with explicit roles/units |
| Binary or .xls rejected | Unsupported legacy/binary container | Compare content with real export format | Export .xlsx or text; renaming binary is not conversion |
| XLSX ambiguity | Several compatible worksheets | Error lists candidate selection context | Use --sheet on supported routes; for others export one compatible sheet |
| Requested worksheet not found/incompatible | Misspelling or sheet has wrong scientific roles | Check exact sheet name and columns | Select the proper EIS/time-series sheet |
| Missing EIS columns | Frequency/real/imaginary roles not resolved | Compare with committed reordered-column fixtures | Use canonical semantic headers and complete required values |
| Implausible EIS scale/sign | Units or -imaginary plotted data supplied as signed imaginary | Check raw instrument export and source values | Convert explicitly to Hz/ohm and signed imaginary; preserve original |
| Selected channel does not exist | Alias/header spelling mismatch | Inspect source header and logical unit-aware aliases | Quote exact source name, e.g. E1/V |
| Unknown/wrong units | Numeric column has absent/incompatible unit | Inspect channel unit and conversion error | Export explicit supported units; do not relabel values without conversion |
| Rows skipped/missing values created | Compatibility reader recovered invalid coordinates/cells/ragged rows | Parse diagnostics and source-row details | Correct source export or accept only under a declared quality policy |
| Duplicate time error | Scientific workflow policy rejects duplicates | Count/source diagnostics | Choose justified average/first/last/deduplicate policy where available, retaining raw data |
| Nonmonotonic time/reset | Acquisition restart or unsorted series | Inspect source sequence and reset magnitude | Use supported paired-sort/segment policy; do not silently stitch unrelated time segments |
| PSD/Allan unavailable | Irregular sampling, short window, excessive gaps or too few clusters | Signal sampling/window diagnostics | Acquire longer regular data or explicitly choose resampling/time-domain-only policy |
| No transient event result | Wrong event kind, missing metadata or invalid event index | Compare metadata underscore kind with CLI hyphen spelling | Use eligible event category and index among eligible events |
| No usable transient window | Insufficient baseline/duration/points or too much missingness | Segment diagnostics and event spacing | Extend acquisition/adjust justified window, not just lower every threshold |
| Fitting failed/parameters at bounds | Invalid model, weak data support or local optimum | Candidate diagnostics, units and bounds | Simplify/test candidates, expand data range or change justified starts/settings |
| Invalid circuit | Unsupported token or malformed series/parallel expression | Quote string; consult element table | Use supported circuit grammar and unique intended labels |
| Calibration says no observations despite concentration events | No usable voltage source, not necessarily absent concentration | Check sparse override, fallback_source, late-window stability | Explicitly select steady_state_median or supply eligible transient results; inspect excluded observations |
| Calibration insufficient/invalid | Too few distinct usable levels, missing temperature/activity/interferents | Observation artifact warnings | Add independent standards and required chemical context |
| Calibration prediction loses rows | Missing or unconvertible batch potential values are filtered | Compare raw finite count with prediction count | Validate units and retain external row correspondence; output is not a timestamp-aligned replacement trace |
| Prediction ignores a supplied input file | --potential also supplied | Exact command | Use one mode deliberately; scalar branch wins |
| Search rejects top_n=0 | Validator contradicts old shipped comment | Error `plotting.top_n must be greater than zero` | Omit top_n in an explicit search config to disable plotting |
| Sparse mechanism config rejected | Nested raw defaults yield invalid zero fields | Confidence/ratio validation error | Copy the complete shipped mechanism TOML and modify needed fields |
| Model path typo does not fail | Loader falls back to built-in definition | Resolved model ID and definition output | Verify path exists before running; do not assume supplied path was read |
| Decomposition refuses measurements | Explicit scientific inputs missing | Error says --input required | Supply ModelInput JSON with actual concentration/activity/environment; voltage alone is insufficient |
| State estimate rejected as unobservable | Too many states for available independent observations | Observability rank/condition diagnostics | Use identifiable simpler model or add real auxiliary standards/context |
| EKF/UKF disagreement or large NIS | Model/noise/initialization assumptions differ from data | Innovations, covariance, outlier/predict-only counts | Revisit Q/R/initialization and dynamics; validate with independent truth |
| Some --filters entries disappear | Compare filters out unknown tokens if valid ones remain | Reported filters versus command | Spell ekf,ukf exactly; invalid-only list fails |
| Health baseline unavailable | Empty/few/incomparable records or zero spread | Baseline record/context/warnings | Add genuinely comparable reference acquisitions; do not duplicate IDs to claim independence |
| Health trend complains missing assessment | Baseline manifest used for trend | Trend record requires assessment, not signal_results | Create a saved-assessment trend manifest |
| Health trends JSON missing | Default JSON destination health_trends.csv overwritten by CSV | Export config and resulting first file line | Set export.trends_filename to health_trends.json in a complete health config |
| Health trend slope scientifically implausible | Known feature/independent-value summary calculation issue | Compare raw points to reported slope | Fit exported value versus independent_value externally; see audit issue |
| Mechanism trend ignores alternate covariate name | Legacy calculation uses sensor_age_days | Inspect manifest ages | Use age or perform desired covariate model externally |
| Artifact kind/schema mismatch | Wrong result family, unsupported schema or manual relabeling | Producer and typed artifact contract | Regenerate/migrate through implemented path; do not edit schema tag to force acceptance |
| Phase-D output unavailable | Missing source, lineage or incompatible evidence | Manifest unavailable_outputs and reasons | Supply compatible exact source artifacts; preserve honest unavailable status |
| Phase-E fixture fails on missing lineage/source path | Fixture JSON expects a staged relative layout | Resolve referenced relative_path from dataset directory | Stage companion files unchanged as in Workflow J |
| Phase-E hash mismatch | Changed protocol/source bytes or wrong dataset | Compare expected versus file SHA-256 | Rebuild a consistent dataset declaration; never bypass validation by unreviewed edits |
| Physical validation rejected before input evaluation | Embedded trust store UNPROVISIONED | Public shipped store and error | Software-only work remains available; do not attempt REAL bootstrap in this workflow |
| Output already exists | Governed bundle requires explicit managed overwrite | Inspect existing destination and manifest | Use a new directory; use --overwrite only for the intended recognized bundle |
| Overwrite still rejected | Foreign/modified files, symlink/path safety or manifest mismatch | Integrity/path diagnostics | Preserve existing content and choose a new destination |
| Output path cannot be written | Missing parent, wrong file/directory distinction or permissions | Command-specific output table | Prepare parent where needed; choose explicit writable destination |

## Reporting an implementation issue

Preserve the exact command, software commit/package version, input/config hashes, minimal nonprivate reproducer, stdout/stderr and result artifact. Report expected scientific behavior separately from observed behavior. Do not alter source data or authority records to make a failing case disappear. The [coverage report](../USER_MANUAL_COVERAGE.md) lists issues discovered during this audit; none were silently fixed.
