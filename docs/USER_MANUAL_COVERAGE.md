# User manual coverage and audit

[Master manual](USER_MANUAL.md)

## Exact scope and audit counts

**Assessment: COMPREHENSIVE USER MANUAL COMPLETE — ALL IMPLEMENTED USER-FACING WORKFLOWS DOCUMENTED.** This means the public command/configuration/workflow surfaces have documentation; it does not mean every numerical branch was executed, every scientific claim was physically validated, or every historical specification line was reconciled. Known implementation defects remain documented and unfixed.

| Item | Result |
|---|---|
| Original execution-audit commit | `6b1ebb84d31cfbd59fb5fc0f7be83161f940d425` |
| Current release baseline | `v0.2.0` at `c1a76e29d8fe6c66bb6a82d4668f8f06fe0f9c7a`; package version `0.2.0` |
| Date | 2026-09-24 |
| Source / working branch | Initial local main / docs/comprehensive-user-manual |
| Initial dirty state | Cargo.lock: 18 additions, 18 deletions; untracked Phase-F __pycache__ directory. Both preserved |
| Top-level commands | 11 |
| Nested subcommands | 29 |
| Executable command forms | 30, counting plot once; its four targets are values, not subcommands |
| CLI options documented | 173 command-local primary long-option occurrences + 6 root legacy options = **179**; four positional arguments bring application argument occurrences to 183. Same-named options on different commands count separately. Aliases and generated help/version are documented but excluded from 179 |
| Help levels executed | 41: root + 11 parents/top-level + 29 children |
| Config files documented | 12 shipped files: 11 TOML + 1 public trust-store JSON; advanced Phase-B/C/E contracts also catalogued |
| Workflows documented | 11: A–J plus non-equilibrium ISM research; common recipes also indexed |
| Output inventory | 30 producer sections; 99 distinct file/path patterns (includes destination placeholders, not a count of schema kinds) |
| Examples executed | 30 successful scientific/report/plot invocations, one for each executable form; 36 attempts including 6 initial failures corrected during preparation |
| Documentation discrepancies | 7, enumerated below |
| Unresolved implementation issue groups | 6, enumerated below |

The 369-row serialized-default reference distinguishes built-in values from shipped TOML values. Plot styles, model declarations and strict evidence/protocol input types have additional systematic tables. “Covered” describes documentation coverage, not scientific certification.

## Coverage matrix

| Area | Source inspected | Manual section | Status |
|---|---|---|---|
| CLI and dispatch | src/cli.rs; src/main.rs; src/runners/mod.rs | [Chapter](user_manual/command_reference.md) | COVERED |
| EIS | src/runners/fit.rs; search.rs; src/impedance/; src/search_config.rs; EIS tests | [Chapter](user_manual/scientific_methods.md) | COVERED |
| Transient | src/runners/transient.rs; src/potentiometry/; src/transient_config.rs; transient tests | [Chapter](user_manual/scientific_methods.md) | COVERED |
| Calibration | src/runners/calibration.rs; src/potentiometry/; src/calibration_config.rs; calibration tests | [Chapter](user_manual/workflows.md) | COVERED |
| Mechanism | src/runners/mechanism.rs; src/mechanism/; both config families; Phase-B tests | [Chapter](user_manual/advanced_contracts.md) | COVERED |
| Signal | src/runners/signal.rs; src/signal/; src/signal_config.rs; signal tests | [Chapter](user_manual/configuration.md) | COVERED |
| Health | src/runners/health.rs; src/health/; src/health_config.rs; Phase-C tests | [Chapter](user_manual/workflows.md) | COVERED |
| Estimation | src/runners/estimation.rs; src/estimation/; src/estimation_config.rs; estimator tests | [Chapter](user_manual/scientific_methods.md) | COVERED |
| Model | src/runners/model.rs; src/model/; src/model_config.rs; model tests | [Chapter](user_manual/configuration.md) | COVERED |
| Public reporting | src/reporting/; src/runners/report.rs; Phase-D fixtures/tests | [Chapter](user_manual/outputs.md) | COVERED |
| Validation | src/mhi_validation/; src/validation_config.rs; Phase-E fixtures/tests | [Chapter](user_manual/advanced_contracts.md) | COVERED |
| Configurations | All 12 config files; loader/default types; serialized default probe | [Chapter](user_manual/configuration.md) | COVERED |
| Outputs and compatibility | src/results/; src/domain/ artifact readers/writers; runners; emitted examples | [Chapter](user_manual/outputs.md) | COVERED |
| Architecture | 221 tracked Rust source files structurally inventoried; main/lib and major module boundaries | [Chapter](user_manual/architecture.md) | COVERED |
| Scientific boundaries | Scientific engines, evidence contracts, reporting projections and validation release rules | [Chapter](user_manual/scientific_methods.md) | COVERED |
| Troubleshooting | Parser/config errors, tests, six failed preparation attempts, discovered issues | [Chapter](user_manual/troubleshooting.md) | COVERED |
| Input data and metadata | src/data_file/; src/domain/; pinned electrodata-io integration and canonical-input tests | [Data](USER_MANUAL.md#preparing-your-data), [metadata](USER_MANUAL.md#experiment-metadata) | COVERED |
| Phase-F governance boundary | Public phase_f specifications/scripts; embedded public trust store | [Phase F](USER_MANUAL.md#phase-f-real-authority) | COVERED |
| Public REAL status snapshot | v0.2.0 public status file | [Phase F status](#phase-f-status) | COVERED |
| Private reviewer/key material and REAL mutation | Outside task scope; not accessed/performed | No operational recipe | NOT_APPLICABLE |

The original audit predates the v0.2.0 status document. There are no public REAL bootstrap/proof commands in this application.

## Complete inventories

**Commands:** plot; eis fit/search/export-fit; transient fit; calibration extract/fit/validate/predict; mechanism compare/trend/report; signal characterize/compare/residuals; health baseline/assess/trend/report; estimate run/validate/simulate/compare/report; model validate/simulate/decompose/report; report render; validation run. Every executable form has the requested 13 reference subsections. Parent grouping commands and all help snapshots are included.

**Configuration files:** `analysis.toml`, `app.toml`, `calibration.toml`, `estimation.toml`, `health.toml`, `mechanism.toml`, `mhi_physical_approval_trust_store.schema1.json`, `model.toml`, `parsing.toml`, `plotting.toml`, `signal.toml`, `transient.toml`.

**Workflow inventory:** A known-circuit EIS; B ECM discovery; C transient; D calibration; E mechanism comparison/trends; F signal quality; G health baseline/assessment/trend; H state estimation; I reduced-order ISM models; J public reporting and software/physical-validation boundary; non-equilibrium ISM research.

**Output inventory:** the [producer/file/schema/consumer tables](user_manual/outputs.md) are the complete operational inventory. Major families are EIS fits and rankings; transient results/features; calibration observations/models/analysis/validation/predictions; mechanism comparisons/timescales/trends; signal characterization/comparison/residuals; health baseline/assessment/trends; state estimates/validation/filter comparisons/simulation; model analysis/decomposition/validation; public report bundles; validation report bundles; figures/text/CSV/JSON and provenance companions. A filename pattern is not a schema version or promise that a conditional plot is always produced.

## Repository inventory

The recursive tracked inventory at the documented commit contains **745 files**: 221 under src, 435 under tests, 66 under docs, 12 under config, and 11 other tracked files. Runtime data/output/logs are workspace products; untracked caches are not canonical source. The [architecture chapter](user_manual/architecture.md) explains responsibilities and inter-module data flow.

| Location | Tracked files | Audit role |
|---|---|---|
| src/(source root) | 22 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/data_file | 10 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/domain | 10 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/estimation | 28 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/fitting | 1 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/health | 10 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/impedance | 12 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/mechanism | 19 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/mhi_validation | 10 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/model | 17 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/plottings | 13 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/potentiometry | 21 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/reporting | 9 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/results | 12 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/runners | 15 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |
| src/signal | 12 | CLI/configuration, adapter, scientific, artifact or presentation layer; see module reference |

Other inspected surfaces: Cargo.toml/Cargo.lock/rust-toolchain.toml (dependencies/version/toolchain), README/CHANGELOG (claims and history), .github workflow (Ubuntu/macOS CI), examples, scripts, top-level plotting example TOMLs, tests and fixtures, docs and engineering specifications. The inventory is recursive; detailed inspection concentrated on executable paths, input/output contracts, configuration loaders, scientific engines and relevant tests rather than treating historical prose as executable authority.

## Executed examples

All commands below exited **0** in the audit workspace after preparation. Paths in the [command reference](user_manual/command_reference.md) preserve their exact argument values; fixture paths are made repository-relative for readability. The audit log records full commands, stdout, stderr and exit statuses. Plot execution used EIS; other plot targets were parser/help/source checked and their recipes are labelled illustrative. Optional Phase-B/C branches rely on source and the full conformance test suite rather than an additional CLI smoke invocation.

| Command | Important observed output / scope |
|---|---|
| `eis fit` | Reusable fit JSON and report for known Randles/CPE fixture. |
| `eis export-fit` | Schema-3 EIS fit JSON. |
| `eis search` | One-generation small-population smoke search; ranking TXT/CSV, not search-quality validation. |
| `transient fit` | One event result; single model, bootstrap disabled. |
| `signal characterize` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `signal residuals` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `mechanism compare` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `mechanism report` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `model validate` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `model simulate` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `model decompose` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `model report` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `estimate simulate` | Seed 42 synthetic measurements/truth/calibration artifact. |
| `estimate run` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `estimate compare` | Comparison JSON/text; no individual filter result export. |
| `estimate validate` | CSV truth alignment and validation JSON/text. |
| `estimate report` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `signal compare` | One synthetic record; no independent cohort conclusion. |
| `health baseline` | Three IDs referencing the same source; software smoke only. |
| `health assess` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `health report` | Expected saved result/report written; see producer inventory. No physical validation inferred. |
| `report render` | Managed bundle: 10 written, 0 unavailable; figures disabled. |
| `plot` | 15 files including Nyquist/Bode comparisons, overlays and fit report. |
| `calibration extract` | Six observations from explicitly synthetic standards. |
| `calibration fit` | Stored model and analysis, bootstrap disabled. |
| `calibration validate` | Training-observation smoke validation; not held-out evidence. |
| `calibration predict` | Scalar synthetic prediction JSON. |
| `health trend` | CSV emitted; confirmed default JSON collision. Slopes are affected by I1. |
| `mechanism trend` | Repeated fixture pairs at declared ages; smoke only. |
| `validation run` | Software fixture dataset after staging companion lineage/source files; no physical approval. |

Six initial failed attempts: calibration extract lacked a late-window fallback in a sparse config; calibration fit/validate/predict then lacked its prerequisite artifacts; health trend was initially given a baseline-style manifest instead of saved assessments; validation was initially given a fixture without its companion dataset-relative files. These were preparation failures, corrected without implementation changes.

Raw audit files remain in an external review cache and are not required to read the manual. Illustrative installation, user-data recipes and governance-boundary examples were not represented as executed physical experiments.

## Documentation discrepancies

| ID | Disagreement | Documented resolution |
|---|---|
| D1 | Original audit checkout was pre-release 0.1.0 and had no local v0.2.0 tag | Resolved for publication: fetched and verified the unchanged v0.2.0 tag and version 0.2.0 on `main` |
| D2 | Original audit checkout lacked the Phase-F status markdown | Resolved for publication: linked the public v0.2.0 snapshot, retaining its NO-GO boundary |
| D3 | Illustrative command tree makes plot targets look like subcommands | Implementation has one plot command with four positional target values |
| D4 | Runtime usage label electroanalysis differs from Cargo binary name | Commands use actual rust_electroanalysis_cli binary; discrepancy explained |
| D5 | Analysis configuration comment describes top_n=0 as disabling plots | src/search_config.rs rejects explicit zero; omit optional top_n to disable |
| D6 | Legacy generic selector prose does not consistently match structured CLI spelling | Structured selector is generic-plot; accepted aliases documented from Clap |
| D7 | Historical electrodata-io migration/consumer prose describes normalization at adapter boundary | Current adapter preserves source units; scientific time-series workflows explicitly normalize to seconds. EIS inputs must use stated canonical numeric units |

Shipped TOML values versus sparse built-in defaults are additionally tabulated. They are distinct configuration layers, not silently treated as identical defaults.

## IMPLEMENTATION ISSUE DISCOVERED DURING MANUAL AUDIT

No issues below were fixed. Severity describes impact on a researcher using the affected route.

| ID / severity | File and function | Observed behavior | Expected/intended behavior; documented handling |
|---|---|---|---|
| I1 / high scientific correctness | src/health/trend.rs:5 calculate (regressions around lines 36–44; spread at final construction) | Regresses independent coordinates against record index and computes their SD, rather than feature values against independent coordinate. Rank helper also assigns sequential ranks to ties | Feature slopes/spread should describe measured feature evolution; use raw points and recompute externally, including tie-aware correlation |
| I2 / high output loss | src/health_config.rs:173 default; src/runners/health.rs:759–760 trend writer | Default JSON filename is health_trends.csv; subsequent CSV writer overwrites it | Separate JSON/CSV destinations; use a complete config with trends_filename="health_trends.json" |
| I3 / medium scientific control | src/calibration_config.rs; signal_config.rs; health_config.rs; estimation_config.rs; mechanism_config.rs and corresponding engines | Several accepted fields are unused or only labels/warnings; omitted legacy mechanism sections can resolve unusable zero defaults | Field-specific applicability list in configuration chapter; preserve full shipped config and do not rely on inactive controls |
| I4 / medium context omission | src/signal/comparison.rs:113 comparison analysis | Manifest metadata accepted but analysis receives no events | Metadata-based event exclusion is not applied; use characterize for that context and compare equivalent windows |
| I5 / medium configuration surprise | src/runners/model.rs:215 load_config | Missing explicit model path falls back to built-in definition | A typo can analyze the default model; verify existence and emitted model identity |
| I6 / medium covariate mismatch | src/mechanism/trend.rs:20 calculate_trend | Uses sensor_age_days even though independent-variable label is configurable | Use sensor age only or compute other covariate trends externally; do not relabel as a different measured axis |

## Validation and QA

| Check | Result |
|---|---|
| cargo build --locked --release | PASS; executable in configured external CARGO_TARGET_DIR |
| cargo fmt --all --check | PASS |
| cargo check --locked | PASS |
| cargo test --locked --all | PASS; 15 documentation examples ignored by the existing suite, four compile-fail doctests passed |
| Recursive runtime help | PASS, all 41 levels; all 70 distinct printed long flags (including generated flags) appear in the reference |
| Executable command examples | PASS after preparation, 30/30 forms |
| Reference structure | PASS: all 30 executable commands contain each of the 13 requested subsections |
| Second source pass | Completed: reviewed config applicability, schemas, filenames, manifests, output consumers, version and scientific limitations |
| Local Markdown links | PASS: 349 local links checked for file existence and heading targets across the manual and coverage report |
| git diff --check | PASS at final documentation verification |

The table records the original pre-release audit with its reordered working Cargo.lock. Synchronization reruns the required validation against the release baseline. No new tests were added because this is documentation only. Numerical correctness was not established by exit status alone; I1 is an example of a defect not caught by existing tests.

## Phase-F status

**REAL authority = NO-GO. REAL authority mutations = none.** The [public v0.2.0 status snapshot](engineering_specification/phase_f/phase_f_real_authority_status.md) records `0 / 5` structurally ready reviewers, no REAL verifier, subjects, identity/equivalence admissions or currentness proof, an absent protected monotonic ref, and G3 NO-GO. It is expressly a non-authoritative operational snapshot, not a live inspection of private or external state. The embedded production physical-approval trust store is UNPROVISIONED with no trust roots. Ordinary research analyses and software/conformance validation remain usable within their interpretation limits. No private key, private reviewer material, bootstrap, verifier, subject, proof or monotonic ref was accessed or created for this manual.

## Changed files and external storage

Manual changes: README.md (guide link and two pre-existing broken-link corrections), docs/USER_MANUAL.md, this coverage report, and eight chapters under docs/user_manual/. A narrow .gitignore rule excludes Python bytecode/cache artifacts. No application, tests, schemas, contracts or authority configuration were changed. Cargo.lock had a pre-existing package-order-only diff and the Phase-F Python cache was untracked; neither belongs in the published manual commit.

One substantial audit directory was created outside the canonical repository under the configured Codex review cache. It contains synthetic fixtures, outputs, logs and a small path-dependency configuration probe; it remains useful for audit reproduction, but normal application/manual use does not require it. The configured external Cargo target directory was reused for Rust builds. No repository clone, Git worktree, virtual environment or substantial artifact was created under a temporary directory. No repository/worktree was deleted.
