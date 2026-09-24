# Configuration reference

[Master manual](../USER_MANUAL.md) · [Commands](command_reference.md) · [Scientific methods](scientific_methods.md)

**VERIFIED IMPLEMENTATION.** This chapter distinguishes a **built-in default** (Rust default object) from a **shipped setting** (`config/*.toml`). An explicit override file replaces the selected TOML; it does not inherit omitted settings from the shipped file. `null` in the tables means absent/None, not the literal TOML value `null` (TOML has no null). Omit that field to retain absence. Runtime bookkeeping `source_path` is not an analysis tuning field.

The field tables below were checked against both source definitions and a read-only serialization probe of the compiled library. TOML uses snake_case enum values unless a documented alias is given. Types are integer, number, boolean, string, list or table; optional numbers/strings can be omitted. Never change schema numbers simply to make an incompatible file parse.

## Configuration inventory and resolution

| File | Consumers / purpose | Schema and key precedence |
|---|---|---|
| `config/app.toml` | Workspace state and last-run bookkeeping | 1; bootstrap/read/write, not an analysis model |
| `config/plotting.toml` | `plot`, search plot styling | 1; explicit --plot-config, canonical file, legacy file, defaults |
| `config/analysis.toml` | `eis search` | 1; explicit --search-config, canonical/legacy file, defaults; --search-top overrides retention |
| `config/parsing.toml` | Circuit selection for EIS parsing/fit/plots | 1; explicit circuit → filename tag → file metadata → first rule → fallback |
| `config/transient.toml` | `transient fit` | 1; defaults → selected file → model/selection/bootstrap/seed CLI |
| `config/calibration.toml` | extraction/fit | 1; defaults → selected file → fit CLI; validation/prediction read stored model settings |
| `config/mechanism.toml` | Legacy comparison/trend | 1; raw-to-resolved loader; separate Phase-B contract is not an overlay |
| `config/signal.toml` | characterize/compare/residuals | 1; selected file/defaults |
| `config/health.toml` | baseline/assess/trend | 1; selected file/defaults; Phase-C supplied separately |
| `config/estimation.toml` | run/compare; stored settings inform validation | 3; migration of older forms; CLI filter/model/seed override |
| `config/model.toml` | model compiler/simulation/decomposition | wrapper 1, shipped definition 3; current in-memory definition schema 4 with migration |
| `config/mhi_physical_approval_trust_store.schema1.json` | Embedded Phase-E physical approval verification | 1, `UNPROVISIONED`, empty roots; not a user CLI override |

`model validate --manifest`, `estimate simulate --scenario`, Phase-B evidence, Phase-C health, Phase-E protocol/dataset, plotting example files and record manifests are additional explicitly selected input/configuration contracts. They are described below and in workflows. There is no shipped `validation.toml` or generic global `--config` flag.

## app.toml

`schema_version=1`; `[logging].level="info"` is application state. `[last_run]` contains `mode` plus optional `plot_config_override`, `analysis_config_override`, `search_output_override`, `search_top_override`, `calibration_config_override`, `calibration_output_override`. These record the last supported invocation; they are not a substitute for saving commands or a guarantee that a previous override is replayed. Workspace setup updates them, so do not use the file hash as the sole immutable analysis identity. The repository currently has last-run mode `eis-fit`, which is historical state, not a scientific default.

## plotting.toml

`[shared]` controls `input_path`, `output_path`, `input_is_directory`, `output_prefix`, with optional `workspace_dir` and `config_file_path` context. Shipped paths are `../data`, `../output`, directory=true, prefix empty, resolved from `config/`. Workflow overrides are `[eis]`, `[regular_plot]`, `[generic_plot]` and their `.style`, `.individual_style`, `.combined_style`; named `[style_presets.NAME]` supplies reusable style groups. Current definitions use workflow tables; old example prose showing `[[generic_plot]]` is not proof of support for repeated job arrays in the current parser.

Style order is domain/global defaults → `[render]` → shared style → named preset → workflow style → individual/combined specialization. The library exposes an additional style layer, but the current CLI does **not** expose arbitrary color/axis flags. Default axis labels are only substituted for sentinel labels. A nonempty palette takes precedence over a scalar experimental color for covered series indices.

| Field group | Type/default | Meaning and when to change |
|---|---|---|
| `input_path`, `output_path`, `input_is_directory`, `output_prefix` | string/string/bool/string; shipped above | Select raw input and destination; avoid writing figures among original data |
| `dpi`, `width_inches`, `height_inches`, `font_size_pt` | positive numbers; shipped 300, 7.2, 5.2, 21 | Figure dimensions/text; appearance, not analysis precision |
| `line_width`, `experimental_line_width`, `fitted_line_width`, `series_line_width` | nonnegative integer style overrides; shipped 6 | Distinguish traces without concealing uncertainty |
| `marker_radius`, `experimental_marker_radius`, `marker`, `experimental_marker_shape`, `show_points` | radii/enum/bool; shipped radii 8, points false | Marker appearance; show points to reveal sampling density |
| `experimental_color`, `fitted_color`, `series_color`, `series_palette` | color/string/array; shipped blue/orange | Hex RGB/RGBA or supported structured colors; palette wins where specified |
| `legend_position`, `legend_font_ratio`, `png_font`, `svg_font` | enum/number/string | Position/font; availability depends on system fonts |
| `x_label`, `y_label`, `x_lim`, `y_lim` | strings and two-number arrays | Units and display limits; cropping is not evidence exclusion |
| `plot_ratio_x`, `plot_ratio_y`, `x_tick_scale`, `y_tick_scale`, `x_tick_decimals`, `y_tick_decimals` | positive ratios / integer precision | Plot geometry/tick presentation |
| `line_style`, `experimental_line_style`, `fitted_line_style` | supported line-style enum | Distinguish series in monochrome output |
| `plot_type`, `bar_width_ratio`, `category_labels` | enum/number/string list | Shape/category rendering; choose a type matching the meaning of samples |
| `fill_between_mode`, `fill_baseline`, `fill_alpha` | enum/number/fraction | Shading; do not present decorative fill as a confidence interval |
| `pie_value_label_mode`, `pie_min_label_percentage` | enum/percentage | Pie labels only; not appropriate for impedance/time trajectories |
| `x_axis_scale`, `y_axis_scale`, `x_axis_log_base`, `y_axis_log_base` | linear/log selector; unspecified log base 10 | Display scaling; logarithmic axes need valid positive domain |
| `sci_notation_x/y`, `sci_notation_threshold_x/y`, `sci_notation_style_x/y` | bool/number/enum | Tick notation, not numeric rescaling |
| `x_transform`, `y_transform` and `_base`, `_a`, `_b` | optional transform; base 10 if log unspecified | Numeric preprocessing (linear/log/negative-log forms); affects regression quantities and labels |
| `plot_positions`, `plot_values` | index list / coordinate list | Explicit point selection; disclose selected points in publications |
| `assign_x`, `assign_y` | numeric lists | Explicit coordinate reassignment; changes the represented data, not merely labels |
| `aggregate_points_across_files`, `aggregate_sort_by_mtime` | optional booleans | Aggregation and ordering; mtime is not an experimental covariate |
| `regression` | omitted; supported `linear` | Ordinary least-squares overlay only, not nonlinear scientific calibration |
| `reg_info_print`, `reg_metrics_print` | optional [2 booleans], [4 booleans] | Equation/R² and n/RMSE/MAE/r annotations |
| `reg_annotation_layout` | `multi_line` unless overridden; also `single_line` | Regression text layout |
| `[render].png_scale_factor`, `.png_dpi`; style `png_scale_factor` | shipped 2, 300 | Raster scale and PNG metadata; does not increase experimental resolution |

There is no worksheet selector in current plotting TOML/CLI definitions. A workbook requiring manual disambiguation should be exported to a single compatible sheet/file for plotting. See the [plot recipe](workflows.md#workflow-a--basic-eis-analysis). The full shipped style file is linked in the snapshot appendix below; additional type/enum spellings are inventoried there.

## analysis.toml and parsing.toml

| Field | Built-in / shipped | Allowed values and scientific consequence |
|---|---|---|
| `max_ranked_results` | 12 / 12 | Positive integer; controls retained candidate count, CLI search-top wins |
| `evolution.population_size` | 24 / 24 | Positive integer; more candidates increases cost and exploration |
| `evolution.generation_limit` | 12 / 12 | Positive integer; more evolution does not prove convergence |
| `evolution.num_individuals_per_parents` | 2 / 2 | Positive integer; offspring generation |
| `evolution.selection_ratio` | 0.7 / 0.7 | (0,1]; breeding selection pressure |
| `evolution.mutation_rate` | 0.35 / 0.35 | [0,1]; exploration versus preservation |
| `evolution.reinsertion_ratio` | 0.75 / 0.75 | (0,1]; generational replacement |
| `evolution.ranking_criterion` | bic / omitted | bic, aic, weighted_rmse, legacy_penalized_score; changes the objective |
| `plotting.top_n` | omitted disables / 3 | Positive when supplied. **0 is rejected**, despite shipped comment |
| `plotting.output_dir` | optional / ../output | Config-relative figure destination; separate from report override |
| `parsing.fallback_model` | see shipped template / R0-p(CPE1,R1) | Valid circuit expression when no higher-priority selection exists |
| `parsing.model_selection.ranking_metric` | shipped aic | aic or weighted_rmse for the applicable circuit-selection path, not search GA criterion |
| `parsing.model_selection.warburg_aic_threshold` | shipped 4 | Preference threshold for Warburg alternatives in that selection path |
| `parsing.rules[].circuit_model` | required per rule | Circuit returned when all rule conditions match |
| `parsing.rules[].filename_contains` | string list | All configured tokens must match the filename |
| `parsing.rules[].metadata_contains` | string map | Required metadata matches; rules evaluated first-match-first |

Circuit config does not define physical file parsing grammar despite its filename. Physical parsing belongs to the pinned provider. The shipped rules select `R0-p(CPE1,R1)-Gw2` for `ism`, `R0-p(CPE1,R1)` for `qd`, and `R0-W1` for matching equivalent-circuit metadata.

## How to tune the scientific sections

The exact field/default tables follow this guide. Ranges described as scientific requirements are not a promise that every loader enforces every invalid combination at deserialization; several signal/health decisions are checked later in engines.

| Section | What it controls / when to change / consequence |
|---|---|
| Transient segmentation | Seconds before/after event, baseline interval, minimum data/duration and missing/order policies. Match acquisition/event spacing; a short post-window biases slow-mode identifiability |
| Transient baseline/models | Baseline method and response convention; candidate exponential families and beta bounds. Change for a declared model comparison, not to force a preferred mechanism |
| Transient optimizer | Positive iteration/start/patience/tolerance controls. More starts can reduce local-minimum dependence; tighter tolerances do not fix insufficient data |
| Transient validation | Tau ratio/window ratio, negligible amplitude, autocorrelation and bound proximity warning thresholds. Altering thresholds changes evidence flags |
| Calibration extraction | Preferred/fallback voltage source, late window, point/missing/slope constraints. Validate actual equilibrium; sparse defaults have no fallback |
| Calibration analyte/activity | Name, nonzero integer charge, optional positive molar mass, activity models and composition. Changes alter the physical quantity inferred |
| Calibration temperature/nernst | Temperature source/alignment and free/fixed/prior slope/sign. Prior/fixed slope encodes a stronger assumption than free regression |
| Calibration interferents | Explicit ion names/charges/coefficients; fitting coefficients requires identifiable variation |
| Calibration weighting/selection/validation | Error floor, branch, ranking and holdout unit. Avoid zero-error domination and leakage across repeated levels |
| Signal windowing/sampling | Stable/event/explicit/entire windows and resampling. Exclusion windows can empty a short dataset; resampling changes bandwidth |
| Signal PSD/Allan | Resolution, overlap, detrending, frequency bands and averaging-time support. Match stationary record duration and acquisition rate |
| Signal drift/spikes/correlation | Drift estimator, local MAD threshold, lag support/counts. A flag is not automatic data removal |
| Health baseline/comparability | Record count and matched analyte/matrix/design/temperature. Relaxing comparability permits broader but less defensible baselines |
| Health normalization/rules | Relative or robust normalization and explicit feature predicates/domain gates. Threshold choice determines findings; missing features need review |
| Estimation filter/state_model | EKF/UKF and state dimensionality; too many confounded states may be unobservable |
| Estimation initialization/covariance | Starting activity/offset/polarization/condition and variances; overconfident initialization delays correction |
| Estimation process/measurement noise | Q versus R sources/scales. Changes determine tracking smoothness and interval width, not just numerical cosmetics |
| Estimation polarization/environment | Tau prior/input drive and environmental alignment/fallbacks. Fallbacks must remain visible in interpretation |
| Estimation observability/equilibrium | Rank/sensitivity and operational equilibrium thresholds; disabling checks does not establish identifiability |
| Estimation timestamp_handling/ingestion | Reject/deduplicate/sort/segment behavior and data-loss thresholds. Segment resets rather than joining physically discontinuous records blindly |
| Estimation compiled model | Backend/profile/definition/input bindings/variance combination. `add_independent` is an explicit independence assumption |
| Uncertainty, all workflows | Confidence in (0,1), nonnegative bootstrap count, seed and success fraction. Zero bootstrap means no bootstrap evidence |
| Export/plotting, all workflows | Filenames and figure requests. Use distinct run directories; not every stored include_* flag is consumed by a dedicated plot writer |

## Important loader and writer caveats

1. Sparse calibration overrides use `fallback_source=None`, while the shipped TOML specifies a steady-state median fallback.
2. Signal built-in defaults analyze `entire_measurement`, with event exclusion margins 0 and no resample interval. The shipped file requests `stable_experiment_region`, margins 10/300 s and interval 1 s.
3. Health built-in `rules=[]`; the shipped file supplies elevated-noise and probable-fouling rules. A sparse override will not inherit them.
4. Legacy mechanism config derives defaults on nested raw structs. Omitting whole sections can produce zero values rejected by validation, even though per-field serde defaults exist. Start from the complete shipped file.
5. `eis.require_uncertainty`, `eis.seed`, `transient.allow_warning_fits`, `trend.enabled` and empty plotting/export groups in legacy mechanism TOML are not all wired into the resolved runner behavior. In particular the runner uses resolved allow_warning_fits from the EIS section for transient extraction. Do not treat accepted fields as guaranteed scientific controls.
6. **Health trend output collision:** built-in `export.trends_filename="health_trends.csv"` is first used for JSON, then overwritten by the fixed CSV writer. Set `trends_filename="health_trends.json"` in a complete health config to retain both files. This is an unresolved implementation issue, not fixed by this manual.
7. Missing model paths fall back to defaults. Confirm the resolved definition and model ID.


## transient field table

| Field | Type | Built-in default | Shipped setting if different | Meaning / valid range / when to change |
|---|---|---|---|---|
| `baseline.method` | string / enum | `"median"` | same / omitted | Select the baseline strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `baseline.response_mode` | string / enum | `"baseline_relative"` | same / omitted | Select the baseline strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `export.features_filename` | string / enum | `"transient_features.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.json_filename` | string / enum | `"transient_results.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.model_comparison_filename` | string / enum | `"transient_model_comparison.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.report_filename` | string / enum | `"transient_report.txt"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `models.beta_max` | number | `1.0` | same / omitted | Upper stretched-exponential beta bound; inspect boundary hits. |
| `models.beta_min` | number | `0.05` | same / omitted | Lower positive stretched-exponential beta bound; must be below beta_max. |
| `models.enabled` | list | `["single", "double", "double_drift", "stretched"]` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `optimizer.ftol` | number | `1e-10` | same / omitted | Positive objective-reduction stopping tolerance. |
| `optimizer.gtol` | number | `1e-10` | same / omitted | Positive gradient stopping tolerance. |
| `optimizer.maximum_iterations` | integer | `400` | same / omitted | Positive optimization effort limit. |
| `optimizer.multiple_starts` | integer | `8` | same / omitted | Positive number of initial guesses to reduce local optimum dependence. |
| `optimizer.patience` | integer | `400` | same / omitted | Positive optimizer patience/effort control. |
| `optimizer.step_bound` | number | `50.0` | same / omitted | Numerical optimization step-size bound. |
| `optimizer.xtol` | number | `1e-10` | same / omitted | Positive parameter-step stopping tolerance. |
| `plotting.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `plotting.include_components` | boolean | `true` | same / omitted | Requested components content; plotting output depends on the specific writer and available results. |
| `plotting.include_model_comparison` | boolean | `true` | same / omitted | Requested model comparison content; plotting output depends on the specific writer and available results. |
| `plotting.include_residuals` | boolean | `true` | same / omitted | Requested residuals content; plotting output depends on the specific writer and available results. |
| `schema_version` | integer | `1` | same / omitted | Contract version; preserve the supported version, not a scientific tuning parameter. |
| `segmentation.baseline_window_s` | number | `20.0` | same / omitted | Pre-event baseline averaging interval in seconds. |
| `segmentation.duplicate_timestamp_policy` | string / enum | `"error"` | same / omitted | Select the segmentation strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `segmentation.irregular_sampling_policy` | string / enum | `"allow"` | same / omitted | Select the segmentation strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `segmentation.maximum_missing_fraction` | number | `0.2` | same / omitted | Largest accepted missing-cell fraction [0,1]. |
| `segmentation.minimum_duration_s` | number | `10.0` | same / omitted | Minimum analysis duration in seconds; longer windows support slower effects. |
| `segmentation.minimum_points` | integer | `20` | same / omitted | Minimum usable sample count; reducing it weakens fitting/statistical support. |
| `segmentation.non_monotonic_policy` | string / enum | `"sort"` | same / omitted | Select the segmentation strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `segmentation.post_event_s` | number | `300.0` | same / omitted | Post-event observation horizon, seconds; length controls which slow relaxations can be identified. |
| `segmentation.pre_event_s` | number | `30.0` | same / omitted | Pre-event context retained, seconds; extend only if baseline remains comparable. |
| `selection.criterion` | string / enum | `"aic"` | same / omitted | Workflow-specific candidate objective; enum list below. |
| `uncertainty.bootstrap_iterations` | integer | `500` | same / omitted | Integer >=0; 0 skips residual/observation bootstrap. |
| `uncertainty.confidence_level` | number | `0.95` | same / omitted | Interval probability between 0 and 1. |
| `uncertainty.minimum_success_fraction` | number | `0.8` | same / omitted | Required successful bootstrap fraction [0,1]. |
| `uncertainty.seed` | integer | `42` | same / omitted | Unsigned random seed; preserve for repeated uncertainty/simulation runs. |
| `validation.bound_proximity_fraction` | number | `0.01` | same / omitted | Fractional distance to a parameter bound that triggers concern. |
| `validation.high_autocorrelation_threshold` | number | `0.8` | same / omitted | Residual autocorrelation warning threshold. |
| `validation.maximum_tau_to_window_ratio` | number | `1.0` | same / omitted | Positive tau/window warning threshold; a larger value accepts more extrapolation. |
| `validation.minimum_tau_ratio` | number | `3.0` | same / omitted | Required mode separation (>1) before interpreting two distinct scales. |
| `validation.negligible_amplitude_fraction` | number | `0.05` | same / omitted | Fraction [0,1] below which an apparent mode is weak. |

## calibration field table

| Field | Type | Built-in default | Shipped setting if different | Meaning / valid range / when to change |
|---|---|---|---|---|
| `activity.conductivity_empirical.b0` | number | `0.0` | same / omitted | Empirical conductivity/log-activity correction intercept. |
| `activity.conductivity_empirical.b1` | number | `0.0` | same / omitted | Empirical conductivity/log-activity correction slope. |
| `activity.conductivity_empirical.conductivity_series` | string / enum | `"conductivity"` | same / omitted | Named conductivity environmental series for empirical correction. |
| `activity.conductivity_empirical.enabled` | boolean | `false` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `activity.conductivity_empirical.fit_b1` | boolean | `false` | same / omitted | Fit empirical slope rather than hold it fixed. |
| `activity.conductivity_empirical.form` | string / enum | `"linear_log_activity_correction"` | same / omitted | **Accepted but inactive in calibration analysis**; see applicability notes below. |
| `activity.conductivity_empirical.maximum_conductivity_s_per_m` | optional | `null` | same / omitted | Optional nonnegative empirical domain upper bound. |
| `activity.conductivity_empirical.minimum_conductivity_s_per_m` | optional | `null` | same / omitted | Optional nonnegative empirical domain lower bound. |
| `activity.davies.a_constant` | number | `0.509` | same / omitted | Positive activity-model A coefficient, with the model’s unit convention. |
| `activity.davies.maximum_ionic_strength_mol_l` | number | `0.5` | same / omitted | Positive activity-model validity ceiling; exceeds it with warnings rather than claiming unlimited validity. |
| `activity.extended_debye_huckel.a_constant` | number | `0.509` | same / omitted | Positive activity-model A coefficient, with the model’s unit convention. |
| `activity.extended_debye_huckel.b_constant` | number | `0.328` | same / omitted | Positive extended Debye–Hückel B coefficient. |
| `activity.extended_debye_huckel.ion_size_parameter` | optional | `null` | same / omitted | Positive ion size required for extended Debye–Hückel. |
| `activity.extended_debye_huckel.ion_size_unit` | string / enum | `"angstrom"` | same / omitted | angstrom or nm variants; do not mix size convention with B constant. |
| `activity.extended_debye_huckel.maximum_ionic_strength_mol_l` | number | `0.1` | same / omitted | Positive activity-model validity ceiling; exceeds it with warnings rather than claiming unlimited validity. |
| `activity.model` | string / enum | `"ideal"` | same / omitted | Model: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `activity.solution_composition` | list | `[]` | same / omitted | List of name, concentration_mol_l>=0 and nonzero charge records used for ionic strength. |
| `activity.user_provided_activity_field` | string / enum | `"activity"` | same / omitted | **Accepted but inactive in calibration analysis**; see applicability notes below. |
| `analyte.charge` | integer | `1` | same / omitted | Signed nonzero integer ion charge; affects theoretical slope/activity. |
| `analyte.molar_mass_g_per_mol` | optional | `null` | same / omitted | Positive molar mass needed for mass-to-molar concentration conversion. |
| `analyte.name` | string / enum | `"auto"` | same / omitted | Declared scientific name; auto resolves from experiment metadata where implemented. |
| `export.features_filename` | string / enum | `"calibration_summary.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.model_filename` | string / enum | `"calibration_model.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.observations_filename` | string / enum | `"calibration_observations.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.report_filename` | string / enum | `"calibration_report.txt"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.residuals_filename` | string / enum | `"calibration_residuals.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.results_filename` | string / enum | `"calibration_results.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.validation_filename` | string / enum | `"calibration_validation.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `hysteresis.analyze` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `hysteresis.log_activity_matching_tolerance` | number | `0.05` | same / omitted | Positive log-activity distance allowed when pairing hysteresis branches. |
| `hysteresis.warning_threshold_v` | number | `0.01` | same / omitted | Nonnegative hysteresis warning threshold in V. |
| `models.enabled` | list | `["nernst"]` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `nernst.prior_slope_v_per_decade` | optional | `null` | same / omitted | Optional signed prior slope for constrained calibration. |
| `nernst.prior_standard_deviation_v_per_decade` | optional | `null` | same / omitted | Optional positive prior uncertainty, not a fitted standard error. |
| `nernst.response_sign` | string / enum | `"auto"` | same / omitted | auto, positive or negative response convention. |
| `nernst.slope_mode` | string / enum | `"free"` | same / omitted | free, fixed_theoretical or prior_constrained Nernst slope. |
| `nicolsky_eisenman.enabled` | boolean | `false` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `nicolsky_eisenman.fit_selectivity_coefficients` | boolean | `false` | same / omitted | Enable coefficient estimation; independent variation is needed for identifiability. |
| `nicolsky_eisenman.interferents` | list | `[]` | same / omitted | List of name, charge, selectivity_coefficient and source; coefficients need explicit information or fitting. |
| `observation_extraction.allow_warning_fits` | boolean | `true` | same / omitted | Whether warning-bearing fits remain eligible; inspect flags rather than hiding them. |
| `observation_extraction.fallback_source` | optional | `null` | `"steady_state_median"` | Optional alternate source if the preferred observation is unavailable. |
| `observation_extraction.maximum_absolute_slope_v_per_s` | number | `1e-05` | same / omitted | Nonnegative late-window drift ceiling for equilibrium observations. |
| `observation_extraction.maximum_missing_fraction` | number | `0.2` | same / omitted | Largest accepted missing-cell fraction [0,1]. |
| `observation_extraction.minimum_points` | integer | `20` | same / omitted | Minimum usable sample count; reducing it weakens fitting/statistical support. |
| `observation_extraction.preferred_source` | string / enum | `"transient_equilibrium"` | same / omitted | Primary voltage observation source; enum list below. |
| `observation_extraction.steady_state_end_s` | number | `300.0` | same / omitted | Window end after concentration event, greater than start. |
| `observation_extraction.steady_state_start_s` | number | `180.0` | same / omitted | Window start after concentration event, seconds, nonnegative. |
| `plotting.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `plotting.include_confidence_band` | boolean | `true` | same / omitted | Requested confidence band content; plotting output depends on the specific writer and available results. |
| `plotting.include_hysteresis` | boolean | `true` | same / omitted | Requested hysteresis content; plotting output depends on the specific writer and available results. |
| `plotting.include_residuals` | boolean | `true` | same / omitted | Requested residuals content; plotting output depends on the specific writer and available results. |
| `plotting.include_validation` | boolean | `true` | same / omitted | Requested validation content; plotting output depends on the specific writer and available results. |
| `schema_version` | integer | `1` | same / omitted | Contract version; preserve the supported version, not a scientific tuning parameter. |
| `selection.branch` | string / enum | `"mixed"` | same / omitted | ascending, descending, mixed or unknown; selects calibration population. |
| `selection.criterion` | string / enum | `"aicc"` | same / omitted | Workflow-specific candidate objective; enum list below. |
| `temperature.alignment` | string / enum | `"linear_interpolation"` | same / omitted | Select the temperature strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `temperature.default_celsius` | number | `25.0` | same / omitted | Fallback temperature above absolute zero; does not measure actual temperature. |
| `temperature.environmental_series` | string / enum | `"temperature"` | same / omitted | Named metadata series used for temperature alignment. |
| `temperature.maximum_gap_s` | number | `30.0` | same / omitted | Positive largest permitted environmental alignment gap in seconds. |
| `temperature.mode` | string / enum | `"observation_specific"` | same / omitted | Select the temperature strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `temperature.reference_celsius` | number | `25.0` | same / omitted | Temperature used by reference normalization. |
| `uncertainty.bootstrap_iterations` | integer | `1000` | same / omitted | Integer >=0; 0 skips residual/observation bootstrap. |
| `uncertainty.confidence_level` | number | `0.95` | same / omitted | Interval probability between 0 and 1. |
| `uncertainty.minimum_success_fraction` | number | `0.8` | same / omitted | Required successful bootstrap fraction [0,1]. |
| `uncertainty.seed` | integer | `42` | same / omitted | Unsigned random seed; preserve for repeated uncertainty/simulation runs. |
| `validation.folds` | integer | `5` | same / omitted | Cross-validation fold count (at least 2 for k-fold). |
| `validation.mode` | string / enum | `"leave_one_concentration_level_out"` | same / omitted | Select the validation strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `validation.prediction_interval_confidence` | number | `0.95` | same / omitted | **Accepted but inactive in calibration analysis**; see applicability notes below. |
| `validation.seed` | integer | `42` | same / omitted | Unsigned random seed; preserve for repeated uncertainty/simulation runs. |
| `weighting.minimum_standard_error_v` | number | `1e-06` | same / omitted | Positive weight floor; avoids a single near-zero error dominating. |
| `weighting.mode` | string / enum | `"potential_standard_error"` | same / omitted | Select the weighting strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |

## signal field table

| Field | Type | Built-in default | Shipped setting if different | Meaning / valid range / when to change |
|---|---|---|---|---|
| `allan.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `allan.minimum_clusters` | integer | `8` | same / omitted | Minimum Allan blocks supporting an averaging scale; positive count. |
| `allan.tau_points` | integer | `30` | same / omitted | Number of averaging scales requested for Allan analysis. |
| `correlation.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `correlation.lag_step_s` | optional | `null` | same / omitted | Optional positive lag spacing in seconds. |
| `correlation.maximum_lag_s` | number | `60.0` | same / omitted | Maximum correlation lag in seconds. |
| `correlation.minimum_observations` | integer | `3` | same / omitted | Minimum paired samples for correlation. |
| `drift.minimum_duration_s` | number | `300.0` | same / omitted | Minimum analysis duration in seconds; longer windows support slower effects. |
| `drift.models` | list | `["ordinary_linear", "theil_sen"]` | same / omitted | Estimator/model list; select from the implemented families in the enum/catalog below. |
| `export.allan_filename` | string / enum | `"signal_allan.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.correlations_filename` | string / enum | `"signal_correlations.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.drift_filename` | string / enum | `"signal_drift.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.psd_filename` | string / enum | `"signal_psd.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.report_filename` | string / enum | `"signal_report.txt"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.results_filename` | string / enum | `"signal_results.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.spikes_filename` | string / enum | `"signal_spikes.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.summary_filename` | string / enum | `"signal_summary.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `plotting.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `psd.detrend` | string / enum | `"linear"` | same / omitted | none, mean or linear; changes low-frequency spectral content. |
| `psd.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `psd.fft_length` | optional | `null` | same / omitted | Optional FFT length; at least segment length; padding does not add information. |
| `psd.frequency_bands` | list | `[]` | same / omitted | List of name/minimum_hz/maximum_hz band-power ranges. |
| `psd.maximum_frequency_hz` | optional | `null` | same / omitted | **Accepted but inactive in signal analysis**; see applicability notes below. |
| `psd.minimum_frequency_hz` | optional | `null` | same / omitted | **Accepted but inactive in signal analysis**; see applicability notes below. |
| `psd.overlap_fraction` | number | `0.5` | same / omitted | Welch overlap fraction in [0,1); larger overlap increases computation/correlation. |
| `psd.parseval_tolerance` | number | `0.1` | same / omitted | Relative discrepancy tolerance between integrated PSD and time-domain variance. |
| `psd.segment_duration_s` | optional | `null` | same / omitted | **Accepted but inactive in signal analysis**; see applicability notes below. |
| `psd.segment_points` | integer | `256` | same / omitted | Welch segment sample count; length sets spectral resolution. |
| `psd.window` | string / enum | `"hann"` | same / omitted | PSD window string; implemented Hann or supported alternatives checked by engine. |
| `sampling.duplicate_timestamp_policy` | string / enum | `"error"` | same / omitted | Select the sampling strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `sampling.maximum_interpolation_gap_s` | number | `5.0` | same / omitted | Positive maximum gap that may be interpolated. |
| `sampling.non_monotonic_timestamp_policy` | string / enum | `"error"` | same / omitted | Select the sampling strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `sampling.policy` | string / enum | `"require_regular"` | same / omitted | Select the sampling strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `sampling.regularity_relative_tolerance` | number | `0.01` | same / omitted | Relative sampling-interval tolerance; smaller is stricter. |
| `sampling.resample_interval_s` | optional | `null` | `1.0` | Positive target interval when linear resampling is chosen; absent uses engine resolution. |
| `schema_version` | integer | `1` | same / omitted | Contract version; preserve the supported version, not a scientific tuning parameter. |
| `spikes.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `spikes.mad_threshold` | number | `4.0` | same / omitted | Positive multiple of robust local MAD scale for spike flags. |
| `spikes.maximum_flagged_fraction` | number | `0.25` | same / omitted | Warning ceiling for flagged fraction [0,1]. |
| `spikes.method` | string / enum | `"hampel"` | same / omitted | Select the spikes strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `spikes.minimum_local_observations` | integer | `5` | same / omitted | Minimum finite values supporting a local spike decision. |
| `spikes.window_duration_s` | optional | `null` | same / omitted | **Accepted but inactive in signal analysis**; see applicability notes below. |
| `spikes.window_points` | integer | `11` | same / omitted | Positive local sample window for Hampel detection. |
| `statistics.confidence_level` | number | `0.95` | same / omitted | Interval probability between 0 and 1. |
| `statistics.quantiles` | list | `[0.01, 0.05, 0.25, 0.5, 0.75, 0.95, 0.99]` | same / omitted | Requested probabilities in [0,1]. |
| `windowing.eligible_event_kinds` | list | `["concentration_step", "flow_change", "temperature_change", "interferent_addition"]` | same / omitted | Metadata snake_case event names included in window selection. |
| `windowing.end_s` | optional | `null` | same / omitted | Absolute window end in seconds, after start. |
| `windowing.exclude_after_event_s` | number | `0.0` | `300.0` | Nonnegative time after eligible events excluded from stable windows. |
| `windowing.exclude_before_event_s` | number | `0.0` | `10.0` | Nonnegative time before eligible events excluded from stable windows. |
| `windowing.relative_end_s` | optional | `null` | same / omitted | **Accepted but inactive in signal analysis**; see applicability notes below. |
| `windowing.relative_start_s` | optional | `null` | same / omitted | **Accepted but inactive in signal analysis**; see applicability notes below. |
| `windowing.source` | string / enum | `"entire_measurement"` | `"stable_experiment_region"` | Select the windowing strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `windowing.start_s` | optional | `null` | same / omitted | Absolute window start in seconds; optional unless explicit interval selected. |

## health field table

| Field | Type | Built-in default | Shipped setting if different | Meaning / valid range / when to change |
|---|---|---|---|---|
| `assessment.allow_warning_artifacts` | boolean | `true` | same / omitted | **Accepted but inactive in health analysis**; see applicability notes below. |
| `assessment.minimum_domains_for_assessment` | integer | `2` | same / omitted | Evidence-domain count needed for assessment availability. |
| `assessment.minimum_domains_for_mechanistic_finding` | integer | `2` | same / omitted | Domain count for a stronger legacy finding; not causal proof. |
| `baseline.minimum_required_records` | integer | `3` | same / omitted | Minimum baseline record count, not independent-domain count. |
| `baseline.robust_statistics` | boolean | `true` | same / omitted | **Accepted but inactive in health analysis**; see applicability notes below. |
| `comparability.maximum_temperature_difference_k` | number | `2.0` | same / omitted | Nonnegative allowed context temperature mismatch in K. |
| `comparability.require_same_analyte` | boolean | `true` | same / omitted | Require same analyte before accepting comparable/usable evidence; relax only with a declared justification. |
| `comparability.require_same_sample_matrix` | boolean | `true` | same / omitted | Require same sample matrix before accepting comparable/usable evidence; relax only with a declared justification. |
| `comparability.require_same_sensor_design` | boolean | `true` | same / omitted | Require same sensor design before accepting comparable/usable evidence; relax only with a declared justification. |
| `export.assessment_filename` | string / enum | `"health_assessment.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.baseline_filename` | string / enum | `"health_baseline.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.features_filename` | string / enum | `"health_features.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.findings_filename` | string / enum | `"health_findings.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.report_filename` | string / enum | `"health_report.txt"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.trends_filename` | string / enum | `"health_trends.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `normalization.minimum_baseline_records_for_z_score` | integer | `5` | same / omitted | Count required before robust normalization is accepted. |
| `normalization.use_relative_difference` | boolean | `true` | same / omitted | Use relative difference: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `normalization.use_robust_z_score` | boolean | `true` | same / omitted | Use robust z score: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `plotting.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `rules` | list | `[]` | `[{"rule_id": "elevated-noise", "finding": "elevated_noise", "severity": "moderate", "minimum_evidence_domains": 1, "all_of": [{"feature": "signal.robust_noise_standard_deviation", "operator": "robust_z_greater_than", "value": 3.0}]}, {"rule_id": "probable-fouling", "finding": "probable_fouling", "severity": "major", "minimum_evidence_domains": 2, "alternative_explanations": ["environmental mismatch", "incomplete baseline context"], "all_of": [{"feature": "transient.tau_slow", "operator": "relative_increase_greater_than", "value": 1.0}], "any_of": [{"feature": "calibration.slope_efficiency", "operator": "relative_decrease_greater_than", "value": 0.2}, {"feature": "eis.role.transport.relaxation_timescale", "operator": "relative_increase_greater_than", "value": 1.0}]}]` | Feature rules; built-in empty, shipped nonempty. See rule grammar below. |
| `schema_version` | integer | `1` | same / omitted | Contract version; preserve the supported version, not a scientific tuning parameter. |

## estimation field table

| Field | Type | Built-in default | Shipped setting if different | Meaning / valid range / when to change |
|---|---|---|---|---|
| `auxiliary.allow_known_standard_events` | boolean | `true` | same / omitted | Permit known standard events; enabling broadens fallback/eligibility and can weaken evidence. |
| `auxiliary.allow_reference_activity` | boolean | `true` | same / omitted | **Accepted but inactive in estimation analysis**; see applicability notes below. |
| `auxiliary.condition_requires_auxiliary` | boolean | `true` | same / omitted | Condition requires auxiliary: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `auxiliary.known_log10_activity_variance` | number | `1e-08` | same / omitted | Positive variance assigned to auxiliary known standards. |
| `auxiliary.standard_variance_v2` | optional | `null` | same / omitted | standard variance v2; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `ekf.numerical_jacobian_relative_step` | number | `1e-06` | same / omitted | **Accepted but inactive in estimation analysis**; see applicability notes below. |
| `ekf.use_joseph_update` | boolean | `true` | same / omitted | **Accepted but inactive in estimation analysis**; see applicability notes below. |
| `environment.alignment` | string / enum | `"linear_interpolation"` | same / omitted | Select the environment strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `environment.allow_configured_fallback` | boolean | `true` | same / omitted | Permit configured fallback; enabling broadens fallback/eligibility and can weaken evidence. |
| `environment.conductivity_series` | string / enum | `"conductivity"` | same / omitted | Named conductivity environmental series for empirical correction. |
| `environment.fallback_conductivity_s_per_m` | optional | `null` | same / omitted | Explicit fallback conductivity s per m used when aligned evidence is unavailable; report its use. |
| `environment.fallback_ionic_strength_mol_l` | optional | `null` | same / omitted | Explicit fallback ionic strength mol l used when aligned evidence is unavailable; report its use. |
| `environment.fallback_temperature_celsius` | number | `25.0` | same / omitted | Explicit fallback temperature celsius used when aligned evidence is unavailable; report its use. |
| `environment.flow_series` | string / enum | `"flow"` | same / omitted | Name of the environmental metadata series used for flow. |
| `environment.interferent_series` | table | `{}` | same / omitted | Name of the environmental metadata series used for interferent. |
| `environment.ionic_strength_series` | string / enum | `"ionic_strength"` | same / omitted | Name of the environmental metadata series used for ionic strength. |
| `environment.maximum_gap_s` | number | `60.0` | same / omitted | Positive largest permitted environmental alignment gap in seconds. |
| `environment.temperature_series` | string / enum | `"temperature"` | same / omitted | Name of the environmental metadata series used for temperature. |
| `environment.window_half_width_s` | number | `5.0` | same / omitted | window half width s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `equilibrium_recognition.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `equilibrium_recognition.maximum_absolute_residual_autocorrelation` | number | `0.2` | same / omitted | Upper threshold for absolute residual autocorrelation; increasing it makes this gate more permissive. |
| `equilibrium_recognition.maximum_absolute_standardized_innovation` | number | `2.0` | same / omitted | Upper threshold for absolute standardized innovation; increasing it makes this gate more permissive. |
| `equilibrium_recognition.maximum_dynamic_potential_v` | number | `0.0001` | same / omitted | Upper threshold for dynamic potential v; increasing it makes this gate more permissive. |
| `equilibrium_recognition.maximum_environment_change_fraction` | number | `0.01` | same / omitted | Upper threshold for environment change fraction; increasing it makes this gate more permissive. |
| `equilibrium_recognition.maximum_equilibrium_gap_v` | number | `0.0001` | same / omitted | Upper threshold for equilibrium gap v; increasing it makes this gate more permissive. |
| `equilibrium_recognition.maximum_normalized_state_rate_per_s` | number | `0.0001` | same / omitted | Upper threshold for normalized state rate per s; increasing it makes this gate more permissive. |
| `equilibrium_recognition.maximum_state_uncertainty_fraction` | number | `0.05` | same / omitted | Upper threshold for state uncertainty fraction; increasing it makes this gate more permissive. |
| `equilibrium_recognition.minimum_elapsed_time_constants` | number | `5.0` | same / omitted | Required relaxation multiples before equilibrium recognition. |
| `equilibrium_recognition.minimum_history_points` | integer | `5` | same / omitted | Number of historical points needed for equilibrium evidence. |
| `equilibrium_recognition.require_observable` | boolean | `true` | same / omitted | Require observable before accepting comparable/usable evidence; relax only with a declared justification. |
| `export.diagnostics_filename` | string / enum | `"state_diagnostics.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.innovations_filename` | string / enum | `"state_innovations.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.report_filename` | string / enum | `"state_estimation_report.txt"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.results_filename` | string / enum | `"state_estimation.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.states_filename` | string / enum | `"state_estimates.csv"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `export.validation_filename` | string / enum | `"state_validation.json"` | same / omitted | Output filename within the workflow destination; changing it changes downstream paths. See output inventory. |
| `extrapolation.inflate_measurement_variance` | boolean | `false` | same / omitted | inflate measurement variance; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `extrapolation.near_boundary_fraction` | number | `0.05` | same / omitted | near boundary fraction between 0 and 1; alters simulated missingness/outliers or threshold support. |
| `extrapolation.near_boundary_variance_inflation_factor` | number | `1.25` | same / omitted | near boundary variance inflation factor; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `extrapolation.variance_inflation_factor` | number | `4.0` | same / omitted | variance inflation factor; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `extrapolation.warn_outside_domain` | boolean | `true` | same / omitted | **Accepted but inactive in estimation analysis**; see applicability notes below. |
| `filter.confidence_level` | number | `0.95` | same / omitted | Interval probability between 0 and 1. |
| `filter.innovation_gate_probability` | number | `0.997` | same / omitted | Probability threshold for innovation/outlier gating, between 0 and 1. |
| `filter.kind` | string / enum | `"ukf"` | same / omitted | Select the filter strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `filter.reject_outliers` | boolean | `true` | same / omitted | Reject gated measurement updates when enabled. |
| `ingestion.max_missing_measurement_fraction` | number | `0.2` | same / omitted | Maximum missing selected-channel fraction [0,1]. |
| `ingestion.max_skipped_timestamp_rows` | integer | `0` | same / omitted | Maximum accepted number of skipped coordinates (integer >=0). |
| `ingestion.reject_missing_required_channel` | boolean | `true` | same / omitted | Reject a wholly missing required measurement channel. |
| `initial_covariance.baseline_variance_v2` | number | `0.0001` | same / omitted | baseline variance v2; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `initial_covariance.condition_variance` | number | `0.01` | same / omitted | condition variance; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `initial_covariance.log10_activity_variance` | number | `0.25` | same / omitted | log10 activity variance; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `initial_covariance.polarization_variance_v2` | number | `0.0001` | same / omitted | polarization variance v2; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `initialization.activity_source` | string / enum | `"calibration_inversion"` | same / omitted | Initialization strategy, e.g. calibration_inversion or configured. |
| `initialization.baseline_v` | number | `0.0` | same / omitted | baseline v in volts; use a finite value appropriate to the selected state/scenario. |
| `initialization.condition_value` | number | `1.0` | same / omitted | Condition value: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `initialization.initial_activity` | number | `0.001` | same / omitted | Positive configured activity/concentration seed, interpreted with declared unit/source. |
| `initialization.initial_activity_unit` | string / enum | `"mol/L"` | same / omitted | Unit for configured initial activity; do not mix activity and mass concentration. |
| `initialization.polarization_v` | number | `0.0` | same / omitted | polarization v in volts; use a finite value appropriate to the selected state/scenario. |
| `initialization.previous_artifact` | optional | `null` | same / omitted | **Accepted but inactive in estimation analysis**; see applicability notes below. |
| `initialization.steady_window_s` | number | `30.0` | same / omitted | **Accepted but inactive in estimation analysis**; see applicability notes below. |
| `measurement_noise.configured_variance_v2` | number | `1e-06` | same / omitted | Positive configured observation variance in V². |
| `measurement_noise.inflate_outside_domain` | boolean | `false` | same / omitted | Reduce trust in out-of-domain observations by inflating R where consumed. |
| `measurement_noise.maximum_variance_v2` | number | `1.0` | same / omitted | Variance cap, at least the floor. |
| `measurement_noise.minimum_variance_v2` | number | `1e-12` | same / omitted | Positive measurement variance floor. |
| `measurement_noise.per_observation_variance` | optional | `null` | same / omitted | **Accepted but inactive in estimation analysis**; see applicability notes below. |
| `measurement_noise.source` | string / enum | `"signal_robust_variance"` | same / omitted | Select the measurement noise strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `model.backend` | string / enum | `"legacy"` | same / omitted | legacy (default) or compiled model adapter. |
| `model.definition` | optional | `null` | same / omitted | Model-definition path for a custom compiled profile; no default. |
| `model.input_bindings.conductivity` | string / enum | `"environment:conductivity"` | same / omitted | Explicit compiled-model input source binding; must agree with model-required units/IDs. |
| `model.input_bindings.custom` | table | `{}` | same / omitted | Map of required custom model input IDs to explicit sources. |
| `model.input_bindings.delta_log10_activity` | string / enum | `"experiment_activity_step"` | same / omitted | Explicit compiled-model input source binding; must agree with model-required units/IDs. |
| `model.input_bindings.target_activity` | string / enum | `"estimated_activity"` | same / omitted | Explicit compiled-model input source binding; must agree with model-required units/IDs. |
| `model.input_bindings.temperature` | string / enum | `"environment:temperature"` | same / omitted | Explicit compiled-model input source binding; must agree with model-required units/IDs. |
| `model.observation_variance.combination` | string / enum | `"estimation_only"` | same / omitted | estimation_only, model_only or add_independent variance sources. |
| `model.profile` | string / enum | `"legacy_equivalent_v1"` | same / omitted | legacy_equivalent_v1, reduced_ism_v1 or custom. |
| `model.transduction_drive.source` | string / enum | `"none"` | same / omitted | Select the model.transduction drive strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `observability.empirical_perturbation` | number | `0.001` | same / omitted | Finite perturbation used for empirical sensitivity checks. |
| `observability.empirical_sensitivity_tolerance` | number | `1e-08` | same / omitted | Threshold below which sensitivity is considered weak. |
| `observability.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `observability.horizon_steps` | integer | `20` | same / omitted | Observation horizon for observability checks. |
| `observability.maximum_condition_number` | number | `10000000000.0` | same / omitted | Positive maximum acceptable observability matrix condition number. |
| `observability.rank_tolerance` | number | `1e-08` | same / omitted | Positive numerical rank tolerance; threshold choice affects identifiability gates. |
| `observability.reject_unobservable_model` | boolean | `true` | same / omitted | Fail if observability checks reject the model rather than silently estimating. |
| `plotting.enabled` | boolean | `true` | same / omitted | Enable this section’s calculation or output; false removes corresponding evidence/output. |
| `plotting.include_covariance` | boolean | `true` | same / omitted | Requested covariance content; plotting output depends on the specific writer and available results. |
| `plotting.include_environment` | boolean | `true` | same / omitted | Requested environment content; plotting output depends on the specific writer and available results. |
| `plotting.include_innovations` | boolean | `true` | same / omitted | Requested innovations content; plotting output depends on the specific writer and available results. |
| `plotting.include_nis` | boolean | `true` | same / omitted | Requested nis content; plotting output depends on the specific writer and available results. |
| `plotting.include_state_uncertainty` | boolean | `true` | same / omitted | Requested state uncertainty content; plotting output depends on the specific writer and available results. |
| `polarization.aggregation` | string / enum | `"median"` | same / omitted | Select the polarization strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `polarization.configured_tau_s` | number | `30.0` | same / omitted | Positive fallback/selected polarization decay time in seconds. |
| `polarization.gain` | number | `1.0` | same / omitted | Declared polarization input gain; sign/scale determine driven response, not an automatically fitted coefficient. |
| `polarization.gain_v_per_log10_activity` | number | `0.0` | same / omitted | Declared polarization input gain; sign/scale determine driven response, not an automatically fitted coefficient. |
| `polarization.input_event_kind` | string / enum | `"concentration_step"` | same / omitted | Event category eligible to drive polarization input. |
| `polarization.input_model` | string / enum | `"none"` | same / omitted | Select the polarization strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `polarization.tau_source` | string / enum | `"transient"` | same / omitted | configured, transient or mechanism source of polarization prior. |
| `polarization.tau_uncertainty_s` | optional | `null` | same / omitted | Optional positive prior uncertainty of the polarization time constant. |
| `polarization.transient_parameter` | string / enum | `"tau_slow"` | same / omitted | Named transient parameter used for polarization tau prior. |
| `process_noise.activity_variance_per_s` | number | `1e-05` | same / omitted | activity variance per s; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `process_noise.baseline_variance_v2_per_s` | number | `1e-10` | same / omitted | baseline variance v2 per s; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `process_noise.condition_variance_per_s` | number | `1e-09` | same / omitted | condition variance per s; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `process_noise.polarization_variance_v2_per_s` | number | `1e-08` | same / omitted | polarization variance v2 per s; nonnegative covariance scale in the stated units; increasing it expresses less certainty. |
| `process_noise.source` | string / enum | `"configured"` | same / omitted | Select the process noise strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `schema_version` | integer | `3` | same / omitted | Contract version; preserve the supported version, not a scientific tuning parameter. |
| `state_model.activity_transform` | string / enum | `"identity_log10"` | same / omitted | Select the state model strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `state_model.condition_initial` | number | `1.0` | same / omitted | Initial condition-state value within its bounds. |
| `state_model.condition_lower` | number | `0.5` | same / omitted | Lower condition-state bound; below upper. |
| `state_model.condition_upper` | number | `1.5` | same / omitted | Upper condition-state bound. |
| `state_model.include_condition_state` | boolean | `false` | same / omitted | Adds a condition/sensitivity state; needs auxiliary information and observability. |
| `state_model.kind` | string / enum | `"activity_baseline_polarization"` | same / omitted | Select the state model strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `timestamp_handling.duplicate_policy` | string / enum | `"deduplicate_identical"` | same / omitted | Select the timestamp handling strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `timestamp_handling.minimum_segment_points` | integer | `2` | `10` | Minimum samples retained per timestamp segment. |
| `timestamp_handling.minor_reversal_threshold_s` | number | `1.0` | same / omitted | Small backward-time tolerance threshold, seconds. |
| `timestamp_handling.non_finite_policy` | string / enum | `"reject"` | same / omitted | Select the timestamp handling strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `timestamp_handling.non_monotonic_policy` | string / enum | `"segment_on_reset"` | same / omitted | Select the timestamp handling strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `timestamp_handling.reset_threshold_fraction` | number | `0.5` | same / omitted | Relative reset threshold; changes segmentation decisions. |
| `timestamp_handling.reset_threshold_s` | number | `10.0` | same / omitted | Absolute threshold for recognizing an acquisition clock reset. |
| `ukf.alpha` | number | `0.001` | same / omitted | UKF sigma-point spread control; must produce valid scaling. |
| `ukf.beta` | number | `2.0` | same / omitted | UKF prior distribution correction, commonly 2 for Gaussian assumptions. |
| `ukf.initial_jitter` | number | `1e-12` | same / omitted | Positive diagonal regularization for factorization failures. |
| `ukf.jitter_multiplier` | number | `10.0` | same / omitted | Growth factor for repeated jitter attempts. |
| `ukf.kappa` | number | `0.0` | same / omitted | UKF secondary scaling; must produce valid sigma weights. |
| `ukf.maximum_jitter_attempts` | integer | `5` | same / omitted | Bounded retry count for covariance factorization. |
| `validation.alignment_policy` | string / enum | `"nearest_within_tolerance"` | same / omitted | Select the validation strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `validation.allow_truth_reuse` | boolean | `false` | same / omitted | Allow one truth observation to match multiple estimates; affects validation independence. |
| `validation.maximum_alignment_gap_s` | number | `0.5` | same / omitted | Nonnegative allowed truth/estimate timestamp mismatch. |
| `validation.states.baseline_offset.absolute_convergence_tolerance` | number | `0.001` | same / omitted | Per-state absolute truth-error threshold in that state’s units. |
| `validation.states.baseline_offset.minimum_consecutive_converged_points` | integer | `1` | same / omitted | Consecutive eligible points needed to declare convergence. |
| `validation.states.baseline_offset.step_detection_threshold` | number | `1e-06` | same / omitted | Threshold for identifying changes in truth. |
| `validation.states.baseline_offset.step_response_fraction` | number | `0.9` | same / omitted | Fraction of truth step used for response-time detection. |
| `validation.states.log10_activity.absolute_convergence_tolerance` | number | `0.05` | same / omitted | Per-state absolute truth-error threshold in that state’s units. |
| `validation.states.log10_activity.minimum_consecutive_converged_points` | integer | `1` | same / omitted | Consecutive eligible points needed to declare convergence. |
| `validation.states.log10_activity.step_detection_threshold` | number | `1e-06` | same / omitted | Threshold for identifying changes in truth. |
| `validation.states.log10_activity.step_response_fraction` | number | `0.9` | same / omitted | Fraction of truth step used for response-time detection. |
| `validation.states.polarization.absolute_convergence_tolerance` | number | `0.001` | same / omitted | Per-state absolute truth-error threshold in that state’s units. |
| `validation.states.polarization.minimum_consecutive_converged_points` | integer | `1` | same / omitted | Consecutive eligible points needed to declare convergence. |
| `validation.states.polarization.step_detection_threshold` | number | `1e-06` | same / omitted | Threshold for identifying changes in truth. |
| `validation.states.polarization.step_response_fraction` | number | `0.9` | same / omitted | Fraction of truth step used for response-time detection. |
| `validation.states.sensitivity_scale.absolute_convergence_tolerance` | number | `0.02` | same / omitted | Per-state absolute truth-error threshold in that state’s units. |
| `validation.states.sensitivity_scale.minimum_consecutive_converged_points` | integer | `1` | same / omitted | Consecutive eligible points needed to declare convergence. |
| `validation.states.sensitivity_scale.step_detection_threshold` | number | `1e-06` | same / omitted | Threshold for identifying changes in truth. |
| `validation.states.sensitivity_scale.step_response_fraction` | number | `0.9` | same / omitted | Fraction of truth step used for response-time detection. |

## simulation field table

| Field | Type | Built-in default | Shipped setting if different | Meaning / valid range / when to change |
|---|---|---|---|---|
| `activity_pulse_duration_s` | number | `10.0` | same / omitted | activity pulse duration s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `activity_pulse_log10` | number | `0.0` | same / omitted | Activity pulse log10: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `activity_pulse_time_s` | optional | `null` | same / omitted | activity pulse time s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `activity_ramp_rate_log10_per_s` | number | `0.0` | same / omitted | activity ramp rate log10 per s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `activity_step_log10` | number | `1.0` | same / omitted | Activity step log10: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `activity_step_time_s` | number | `80.0` | same / omitted | activity step time s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `baseline_drift_v_per_s` | number | `0.0` | same / omitted | baseline drift v per s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `baseline_initial_v` | number | `0.0` | same / omitted | baseline initial v in volts; use a finite value appropriate to the selected state/scenario. |
| `baseline_random_walk_sd_v` | number | `0.0` | same / omitted | baseline random walk sd v in volts; use a finite value appropriate to the selected state/scenario. |
| `initial_log10_activity` | number | `-3.0` | same / omitted | Initial log10 activity: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `interval_s` | number | `1.0` | same / omitted | interval s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `irregular_jitter_s` | number | `0.0` | same / omitted | irregular jitter s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `measurement_noise_sd_v` | number | `0.0005` | same / omitted | measurement noise sd v in volts; use a finite value appropriate to the selected state/scenario. |
| `missing_fraction` | number | `0.0` | same / omitted | missing fraction between 0 and 1; alters simulated missingness/outliers or threshold support. |
| `model.backend` | string / enum | `"legacy"` | same / omitted | legacy (default) or compiled model adapter. |
| `model.definition` | optional | `null` | same / omitted | Model-definition path for a custom compiled profile; no default. |
| `model.input_bindings.conductivity` | string / enum | `"environment:conductivity"` | same / omitted | Explicit compiled-model input source binding; must agree with model-required units/IDs. |
| `model.input_bindings.custom` | table | `{}` | same / omitted | Map of required custom model input IDs to explicit sources. |
| `model.input_bindings.delta_log10_activity` | string / enum | `"experiment_activity_step"` | same / omitted | Explicit compiled-model input source binding; must agree with model-required units/IDs. |
| `model.input_bindings.target_activity` | string / enum | `"estimated_activity"` | same / omitted | Explicit compiled-model input source binding; must agree with model-required units/IDs. |
| `model.input_bindings.temperature` | string / enum | `"environment:temperature"` | same / omitted | Explicit compiled-model input source binding; must agree with model-required units/IDs. |
| `model.observation_variance.combination` | string / enum | `"estimation_only"` | same / omitted | estimation_only, model_only or add_independent variance sources. |
| `model.profile` | string / enum | `"legacy_equivalent_v1"` | same / omitted | legacy_equivalent_v1, reduced_ism_v1 or custom. |
| `model.transduction_drive.source` | string / enum | `"none"` | same / omitted | Select the model.transduction drive strategy; enum values below. Changing it changes the accepted data or calculation, not just display. |
| `outlier_fraction` | number | `0.0` | same / omitted | outlier fraction between 0 and 1; alters simulated missingness/outliers or threshold support. |
| `outlier_magnitude_v` | number | `0.05` | same / omitted | outlier magnitude v in volts; use a finite value appropriate to the selected state/scenario. |
| `polarization_activity_step_gain_v_per_log10` | number | `0.0` | same / omitted | Polarization activity step gain v per log10: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `polarization_initial_v` | number | `0.0` | same / omitted | polarization initial v in volts; use a finite value appropriate to the selected state/scenario. |
| `polarization_input_model` | string / enum | `"none"` | same / omitted | Polarization input model: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `polarization_pulse_time_s` | optional | `null` | same / omitted | polarization pulse time s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `polarization_pulse_v` | number | `0.0` | same / omitted | polarization pulse v in volts; use a finite value appropriate to the selected state/scenario. |
| `polarization_tau_s` | number | `30.0` | same / omitted | polarization tau s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `sample_count` | integer | `200` | same / omitted | Sample count: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `schema_version` | integer | `2` | same / omitted | Contract version; preserve the supported version, not a scientific tuning parameter. |
| `seed` | integer | `42` | same / omitted | Unsigned random seed; preserve for repeated uncertainty/simulation runs. |
| `sensitivity_drift_per_s` | number | `0.0` | same / omitted | sensitivity drift per s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `sensitivity_initial` | number | `1.0` | same / omitted | Sensitivity initial: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `start_time_s` | number | `0.0` | same / omitted | start time s in seconds; a duration/interval must be positive, optional timing can be absent. |
| `temperature_celsius` | number | `25.0` | same / omitted | Temperature celsius: section-specific control; use the tuning guide and selected algorithm contract above before changing it. |
| `temperature_ramp_celsius_per_s` | number | `0.0` | same / omitted | temperature ramp celsius per s in seconds; a duration/interval must be positive, optional timing can be absent. |

## Scientific enum vocabulary

Enum names identify the configuration type, not a TOML key. Field tables associate them by meaning. Values are serialized snake_case, with explicit renames where present. Some enum choices need additional evidence and can be rejected later by a workflow.

| Domain / type | Values |
|---|---|
| transient_config.rs / `DuplicateTimestampPolicy` | `error`, `average` |
| transient_config.rs / `NonMonotonicPolicy` | `sort`, `error` |
| transient_config.rs / `IrregularSamplingPolicy` | `allow`, `error` |
| transient_config.rs / `SelectionCriterion` | `aic`, `bic` |
| potentiometry / `TransientModelKind` | `single`, `double`, `double_drift`, `stretched` |
| potentiometry / `BaselineMethod` | `mean`, `median`, `linear` |
| potentiometry / `ResponseMode` | `absolute`, `baseline_relative` |
| results / `CalibrationBranch` | `ascending`, `descending`, `mixed`, `unknown` |
| results / `CalibrationPotentialSource` | `transient_equilibrium`, `steady_state_window_mean`, `steady_state_window_median`, `explicit_observation` |
| results / `ActivityModelKind` | `ideal`, `davies`, `extended_debye_huckel`, `conductivity_empirical`, `user_provided` |
| results / `NernstSlopeMode` | `free`, `fixed_theoretical`, `prior_constrained` |
| results / `TemperatureMode` | `constant`, `observation_specific`, `reference_normalized` |
| results / `ResponseSign` | `auto`, `positive`, `negative` |
| results / `CalibrationModelKind` | `nernst`, `nicolsky_eisenman`, `conductivity_empirical` |
| results / `CalibrationFitStatus` | `converged`, `failed`, `invalid` |
| results / `CalibrationSelectionCriterion` | `aic`, `aicc`, `bic`, `cross_validation` |
| results / `CrossValidationMode` | `none`, `leave_one_out`, `leave_one_concentration_level_out`, `k_fold`, `leave_one_experiment_out` |
| results / `EnvironmentalAlignment` | `nearest`, `linear_interpolation`, `window_mean`, `window_median` |
| results / `WeightingMode` | `uniform`, `potential_standard_error`, `user_provided` |
| signal_config.rs / `SamplingPolicy` | `require_regular`, `allow_irregular_time_domain_only`, `resample_linear` |
| signal_config.rs / `DuplicateTimestampPolicy` | `error`, `average`, `first`, `last` |
| signal_config.rs / `NonMonotonicTimestampPolicy` | `error`, `sort_paired` |
| signal_config.rs / `SignalWindowSource` | `entire_measurement`, `explicit_interval`, `event_relative`, `stable_experiment_region`, `residual_artifact` |
| signal_config.rs / `DetrendKind` | `none`, `mean`, `linear` |
| health_config.rs / `FeatureOperator` | `greater_than`, `less_than`, `relative_increase_greater_than`, `relative_decrease_greater_than`, `log_ratio_greater_than`, `robust_z_greater_than`, `warning_present`, `trend_increasing`, `trend_decreasing`, `evidence_level_present` |
| health_config.rs / `HealthSeverity` | `informational`, `minor`, `moderate`, `major`, `critical` |
| health_config.rs / `HealthFindingKind` | `elevated_noise`, `excessive_drift`, `frequent_spikes`, `slow_response`, `reduced_response_amplitude`, `reduced_sensitivity`, `high_hysteresis`, `poor_calibration_prediction`, `eis_parameter_shift`, `poor_model_identifiability`, `probable_fouling`, `probable_reference_instability`, `probable_contact_issue`, `environmental_mismatch`, `data_quality_problem` |
| health_config.rs / `EnvironmentalCovariate` | `temperature_k`, `conductivity_s_per_m`, `ionic_strength_mol_l`, `flow` |
| health_config.rs / `PhaseCHypothesisRelationship` | `possible_physical_degradation` |
| estimation_config.rs / `FilterKind` | `ekf`, `ukf` |
| estimation_config.rs / `StateModelKind` | `activity`, `activity_baseline`, `activity_baseline_polarization`, `custom` |
| estimation_config.rs / `EstimationModelBackend` | `legacy`, `compiled` |
| estimation_config.rs / `CompiledEstimationProfile` | `legacy_equivalent_v1`, `reduced_ism_v1`, `custom` |
| estimation_config.rs / `TransductionDriveSource` | `none`, `activity_step`, `explicit_event_field` |
| estimation_config.rs / `ObservationVarianceCombination` | `estimation_only`, `model_only`, `add_independent` |
| estimation_config.rs / `StateTransformKind` | `identity_log10`, `log10_positive`, `log_positive`, `logistic_bounded` |
| estimation_config.rs / `CovarianceSourceKind` | `configured`, `signal_artifact`, `calibration_artifact`, `transient_artifact`, `estimated_from_training_data` |
| estimation_config.rs / `AlignmentKind` | `nearest`, `linear_interpolation`, `window_mean`, `window_median`, `hold_previous` |
| estimation_config.rs / `TauSourceKind` | `transient`, `configured`, `mechanism` |
| estimation_config.rs / `AggregationKind` | `median`, `mean`, `first` |
| estimation_config.rs / `PolarizationInputModel` | `none`, `explicit_event_voltage`, `activity_step_gain` |
| estimation_config.rs / `MeasurementNoiseSourceKind` | `configured`, `per_observation`, `signal_robust_variance`, `stable_window_variance`, `calibration_residual_variance`, `calibration_prediction_uncertainty` |
| estimation_config.rs / `TruthAlignmentPolicy` | `exact`, `nearest_within_tolerance`, `linear_interpolation` |
| estimation / `DuplicatePolicy` | `deduplicate_identical`, `reject`, `keep` |
| estimation / `NonMonotonicPolicy` | `segment_on_reset`, `stable_sort_within_segment`, `reject` |
| estimation / `NonFiniteTimestampPolicy` | `reject`, `drop_rows` |
| regression_mod.rs / `RegressionKind` | `linear` |
| plottings / `PlotSeriesKind` | `experimental`, `fitted`, `regression_fit` |
| plottings / `PlotAxisScale` | `linear`, `log10` |
| plottings / `PlotLegendPosition` | `upper_left`, `middle_left`, `lower_left`, `upper_middle`, `middle_middle`, `lower_middle`, `upper_right`, `middle_right`, `lower_right` |
| plottings / `PlotMarkerShape` | `circle`, `square`, `triangle`, `cross` |
| plottings / `PlotLineStyle` | `solid`, `dashed` |
| plottings / `PlotType` | `line`, `scatter`, `vertical_bar`, `horizontal_bar`, `grouped_bar`, `stacked_bar`, `fill_between`, `stack_plot`, `pie` |
| plottings / `FillBetweenMode` | `between_curves`, `to_zero`, `to_baseline` |
| plottings / `PieValueLabelMode` | `none`, `percentage`, `value`, `value_and_percentage` |
| plottings / `RegressionAnnotationLayout` | `multi_line`, `single_line` |
| plottings / `AxisScaleKind` | `linear`, `log` |
| plottings / `ScientificNotationStyle` | `full`, `normalized` |
| plottings / `AxisScale` | `linear`, `log` |

## Legacy mechanism fields

The shipped values are the safest starting point because omitted raw sections do not always obtain the intended per-field defaults. Types derive directly from TOML. These are declared settings; applicability is constrained by the loader caveats above.

| Field | Type | Shipped value | Meaning |
|---|---|---|---|
| `schema_version` | integer | `1` | Schema contract version. |
| `eis.allow_warning_fits` | boolean | `true` | EIS confidence/eligibility declaration; require_uncertainty and seed are not propagated as controls by this loader. |
| `eis.require_uncertainty` | boolean | `false` | EIS confidence/eligibility declaration; require_uncertainty and seed are not propagated as controls by this loader. |
| `eis.confidence_level` | number | `0.95` | EIS confidence/eligibility declaration; require_uncertainty and seed are not propagated as controls by this loader. |
| `eis.seed` | integer | `42` | EIS confidence/eligibility declaration; require_uncertainty and seed are not propagated as controls by this loader. |
| `transient.allow_warning_fits` | boolean | `true` | Selected-model declaration; legacy extractor uses selected fits; allow_warning_fits comes from EIS section. |
| `transient.selected_model_only` | boolean | `true` | Selected-model declaration; legacy extractor uses selected fits; allow_warning_fits comes from EIS section. |
| `timescale.monte_carlo_samples` | integer | `10000` | Positive Monte Carlo count, seed and frequency-boundary margin [0,1]; verify which extraction route consumes them. |
| `timescale.seed` | integer | `42` | Positive Monte Carlo count, seed and frequency-boundary margin [0,1]; verify which extraction route consumes them. |
| `timescale.frequency_boundary_margin` | number | `0.1` | Positive Monte Carlo count, seed and frequency-boundary margin [0,1]; verify which extraction route consumes them. |
| `matching.require_experiment_id` | boolean | `true` | Require declared experiment/sensor identity where the route checks it; missing/mismatch can prevent comparison. |
| `matching.require_sensor_id` | boolean | `false` | Require declared experiment/sensor identity where the route checks it; missing/mismatch can prevent comparison. |
| `comparison.ratio_weak` | number | `10.0` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `comparison.ratio_moderate` | number | `3.0` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `comparison.ratio_strong` | number | `1.5` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `comparison.log_distance_weak` | number | `1.0` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `comparison.log_distance_moderate` | number | `0.5` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `comparison.log_distance_strong` | number | `0.1761` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `comparison.minimum_fit_quality` | number | `0.0` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `comparison.compatibility_ratio_lower` | number | `0.5` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `comparison.compatibility_ratio_upper` | number | `2.0` | Ratio/log-distance compatibility/evidence thresholds; require ordered strong <= moderate <= weak, ratio strong >=1; compatibility lower >0 and < upper. |
| `evidence.minimum_replicates_for_strong` | integer | `3` | Declared replicate support threshold; never replaces provenance-based independence. |
| `trend.enabled` | boolean | `true` | Minimum records and independent-variable label; engine currently reads sensor_age_days. |
| `trend.minimum_records` | integer | `3` | Minimum records and independent-variable label; engine currently reads sensor_age_days. |
| `trend.independent_variable` | string / enum | `"sensor_age_days"` | Minimum records and independent-variable label; engine currently reads sensor_age_days. |

## model.toml: declaring a reduced-order model

The wrapper `schema_version=1` contains `[model]`. The definition has its own schema number; shipped definition 3 migrates to current definition 4. `model_id`, `description` and `validity_domain` describe its scientific scope. `uncertainty_incomplete` records an incomplete uncertainty declaration, not a way to suppress missing uncertainty. Arrays `states`, `parameters`, `inputs`, `components` define the model; IDs must be stable and bindings must refer to existing entries.

| Record / fields | Type, default and allowed constraints | Scientific use |
|---|---|---|
| State `id`, `name`, `description`, `unit`, `source`, `validity_domain` | Strings; explicit ID/unit/source/domain; descriptive defaults/migrations where supported | Identify a latent quantity and its scope |
| State `transformation` | identity, log, logit or custom according to the model state enum | Enforce the numerical/physical state domain |
| State `initialization_source` | declared_default or supported explicit source enum | Distinguish assumed from externally supplied initialization |
| State `lower_bound`, `upper_bound`, `initial_value` | Finite numbers, ordered bounds and value in bounds | Avoid invalid model states; not a fitted confidence interval |
| State `initial_uncertainty` | Tagged UncertaintySpec; deterministic/unknown/quantified variants | Explicit uncertainty; unknown is not zero |
| State `process_equation_version`, `observability_requirements` | Version integer and string list | Versioned evolution and declared interpretation prerequisites |
| Parameter `id`, `name`, `description`, `unit`, `source`, `validity_domain` | String identity/context | Stable binding and calibration provenance |
| Parameter `lower_bound`, `upper_bound`, `default_value` | Finite bounded values | Default values are assumptions, not estimated sensor properties |
| Parameter `uncertainty`, `value_source`, `characteristic` | Tagged uncertainty / enums | Fixed, supplied or estimated semantics; discrete integer charge is different from a continuous fitted parameter |
| Parameter `equation_version`, `identifiability_requirements` | Integer/list | Trace equation version and requirements for inference |
| Input `id`, `unit`, `required`, `source`, `validity_domain` | String/string/bool/string/string | Typed external scientific quantities; required inputs must be bound |
| Component `id`, `kind`, `role`, `interpretation_status` | Stable ID, registered kind, role, interpretation enum | Select implemented equation and limit physical claims |
| `depends_on`, `required_inputs`, `state_ids`, `parameter_ids` | Lists of IDs or {id,unit} inputs | Dependency graph and equation bindings; no cycles/unresolved IDs |
| `observation_state_ids`, `observation_parameter_ids`, `numerical_jacobian_supported` | ID lists/bool | Declare observation dependence for derivatives/filter integration |
| `output_unit`, `voltage_contribution_owner`, `contribution_semantics` | Optional strings and semantic enum | Separate additive voltage from variance; prevent double counting |
| `legacy_composition_rule` | Optional migration context | Compatibility only, not a new physical equation |
| `source`, `validity_domain`, `equation`, `equation_version`, `assumptions` | Strings/version/list | Scientific declaration accompanying the implementation |
| `evidence_requirements`, `applicability_constraints`, `metadata` | Typed lists/string map | Evidence gates and scope; metadata is not executable code |

The shipped model declares fast/slow voltage states at 0 V, tau defaults 1/30 s, gains 1/0.2, standard potential 0 V, charge +1, drift 0 V/s and observation noise SD 0.001 V. Several parameter uncertainties are explicitly unknown. Inspect the complete [shipped model](../../config/model.toml) before using it scientifically. Compiling it does not make those values calibrated.

Runtime ModelInput is JSON, for example (**illustrative**, units must match your model):

```json
{"time_s":0.0,"values":{"primary_concentration":{"value":0.001,"unit":"mol/L"},"temperature":{"value":298.15,"unit":"K"},"driving_step_v":{"value":0.01,"unit":"V"},"observed_voltage_v":{"value":-0.17,"unit":"V"}}}
```

An array represents successive timestamps. Decomposition advances from the previous input over the next interval. This is not a voltage-to-concentration inverse solver.

## Health rule grammar

`[[rules]]` requires a `rule_id`, `finding`, `severity`; optional `all_of`, `any_of`, `minimum_evidence_domains`, `minimum_baseline_records`, `alternative_explanations` default to empty/zero. Each condition has `feature`, `operator`, optional numeric `value`. Operators are `greater_than`, `less_than`, `relative_increase_greater_than`, `relative_decrease_greater_than`, `log_ratio_greater_than`, `robust_z_greater_than`, `warning_present`, `trend_increasing`, `trend_decreasing`, `evidence_level_present`. Choose a feature actually exported by its upstream domain; a plausible name is not automatically available.

Severity is informational/minor/moderate/major/critical. Findings include elevated_noise, excessive_drift, frequent_spikes, slow_response, reduced_response_amplitude, reduced_sensitivity, high_hysteresis, poor_calibration_prediction, eis_parameter_shift, poor_model_identifiability, probable_fouling, probable_reference_instability, probable_contact_issue, environmental_mismatch and data_quality_problem. These labels remain evidence summaries.

## Phase-B hypothesis configuration

`--mechanism-evidence-config` selects a **closed explicit contract**, distinct from `config/mechanism.toml`. Start with a repository contract fixture under [Phase-B fixtures](../../tests/fixtures/phase_b/). This is an advanced scientific configuration, not a switch that automatically assigns a mechanism.

`schema_version`, timescale/amplitude/repeatability/identifiability algorithm IDs, temporal join policy and promotion minimum support are explicit. Each hypothesis declares stable ID/display name/target component IDs, evidence requirements, pair requirements, critical IDs, optional timescale gate, amplitude/repeatability gates, identifiability bindings, validation applicability and role bindings. Requirements name source classes, source-field paths, quantity semantic, required unit, expected direction, validity, gate and stage. Pair requirements link requirement IDs and temporal compatibility. Amplitude floors are unit-qualified; repeatability counts independent acquisition families; identifiability input selection is explicit. A prior mechanism artifact adds history, not automatic new support.

Changing a threshold or binding changes the evidence question and should be versioned before inspecting confirmatory data. Wrong component IDs, missing source paths, unit mismatch and unknown independence are not repaired by guessing. Algorithm strings and the full nested field/type catalog are in [advanced contracts](advanced_contracts.md); mandatory fields have no implicit scientific default.

## Phase-C health configuration

`--phase-c-config` selects the explicit dimension contract. The fixture [valid_phase_c.toml](../../tests/fixtures/phase_c/config/valid_phase_c.toml) is a complete syntax example, not recommended thresholds for every sensor. Required groups cover data_quality, signal_integrity, calibration_health, dynamic_response_health, environmental_robustness, model_consistency, observability, uncertainty_health and causal_promotion, with optional hypothesis bindings.

`LevelThreshold` carries ordered `watch <= degraded <= critical` values. Signal amplitudes/residuals are volts, drift V/s, temperatures kelvin, times seconds; ratio/condition-number/standard-error thresholds must meet explicit positivity checks. Data quality controls finite count, missing fraction, interval CV, duplicate/reset counts and interpolation-gap acceptance. Dynamic-response settings select an event and named baseline features, not averages over arbitrary incompatible events. Environmental evidence needs a selected covariate, enough points/range, residual RMS and Spearman thresholds. Causal promotion requires independent support and declared Phase-B relationships; it cannot infer an absent hypothesis.

## Phase-E protocol, dataset and trust configuration

`validation run` requires a complete protocol; unknown fields and semantic defaults are rejected. Its registration identifies an immutable reference and hash. Target-domain selectors cover analyte, matrix, sensor design, sensor, campaign and temperature; `any_declared` is a declaration, not permission to accept missing identity. Mechanism/health endpoints specify the target, reference method/authority/blinding/uncertainty rule, sample support, strata and acceptance rules. Statistics freeze interval method/confidence/undefined-metric behavior/composition. Release claims declare software or physical scope and supporting endpoints.

A dataset contains its protocol SHA-256, cohort semantic identity, lineage-catalog reference, source artifacts with exact file hashes/kind/schema/lineage, record cohort role and declared scope/domain, evidence origin, and reference endpoints. Paths are safe dataset-relative references; the repository's individual JSON fixture is not self-contained until its companion lineage/source files are staged in the expected layout. Do not edit hashes merely to bypass a mismatch: reconstruct the dataset from the actual declared inputs.

The public trust-store JSON has `schema_version`, `trust_store_id`, `provisioning_state`, `trust_roots`. Shipped values are 1, `mhi_physical_approval_trust_store_v1`, `UNPROVISIONED`, `[]`. It is embedded in production, not selected by a CLI flag. Physical requests fail early with this unprovisioned store. This manual describes no provisioning/private-key workflow and does not alter authority.

## Shipped configuration snapshots

These files are the exact repository examples, not universally optimal settings. Preserve whole-file context when copying; see the differences from built-in defaults above.

### analysis.toml

[Repository file](../../config/analysis.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1
max_ranked_results = 12

# ── Genetic Algorithm Evolution Parameters ──────────────────────────────
[evolution]
# Number of circuit candidates in each generation
population_size = 24
# Maximum number of GA generations before search terminates
generation_limit = 12
# Offspring produced per parent pair
num_individuals_per_parents = 2
# Fraction of the population selected for breeding (0.0–1.0)
selection_ratio = 0.7
# Probability of mutation when producing offspring (0.0–1.0)
mutation_rate = 0.35
# Fraction of offspring reinserted into the next generation (0.0–1.0)
reinsertion_ratio = 0.75

# ── Search Result Plotting ──────────────────────────────────────────────
[plotting]
# Number of top-ranked candidates to render as figures.
# Set to 0 (or omit) to disable search-result plots entirely.
top_n = 3
# Output directory for search result plots.
# Relative paths are resolved from the config/ directory.
output_dir = "../output"
```

</details>

### app.toml

[Repository file](../../config/app.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1

[logging]
level = "info"

[last_run]
mode = "eis-fit"
```

</details>

### calibration.toml

[Repository file](../../config/calibration.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1

[observation_extraction]
preferred_source = "transient_equilibrium"
allow_warning_fits = true
fallback_source = "steady_state_median"
steady_state_start_s = 180.0
steady_state_end_s = 300.0
minimum_points = 20
maximum_missing_fraction = 0.20
maximum_absolute_slope_v_per_s = 0.00001

[analyte]
name = "auto"
charge = 1

[temperature]
mode = "observation_specific"
default_celsius = 25.0
reference_celsius = 25.0
environmental_series = "temperature"
alignment = "linear_interpolation"
maximum_gap_s = 30.0

[activity]
model = "ideal"

[nernst]
slope_mode = "free"
response_sign = "auto"

[hysteresis]
analyze = true
log_activity_matching_tolerance = 0.05
warning_threshold_v = 0.010

[weighting]
mode = "potential_standard_error"
minimum_standard_error_v = 0.000001

[selection]
criterion = "aicc"
branch = "mixed"

[validation]
mode = "leave_one_concentration_level_out"
folds = 5
seed = 42
prediction_interval_confidence = 0.95

[uncertainty]
confidence_level = 0.95
bootstrap_iterations = 1000
seed = 42
minimum_success_fraction = 0.80

[plotting]
enabled = true
include_residuals = true
include_hysteresis = true
include_validation = true
include_confidence_band = true

[export]
observations_filename = "calibration_observations.json"
model_filename = "calibration_model.json"
results_filename = "calibration_results.json"
features_filename = "calibration_summary.csv"
residuals_filename = "calibration_residuals.csv"
validation_filename = "calibration_validation.csv"
report_filename = "calibration_report.txt"
```

</details>

### estimation.toml

[Repository file](../../config/estimation.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 3

[ingestion]
# Canonical compatibility recovery is retained in every estimate artifact.
# Reject skipped coordinate rows by default; allow at most 20% missing values
# in the selected measurement channel, and reject a wholly missing channel.
max_skipped_timestamp_rows = 0
max_missing_measurement_fraction = 0.20
reject_missing_required_channel = true

[filter]
kind = "ukf"
confidence_level = 0.95
innovation_gate_probability = 0.997
reject_outliers = true

[state_model]
kind = "activity_baseline_polarization"
activity_transform = "identity_log10"
include_condition_state = false

[initialization]
activity_source = "calibration_inversion"
initial_activity = 0.001
initial_activity_unit = "mol/L"
baseline_v = 0.0
polarization_v = 0.0

[initial_covariance]
log10_activity_variance = 0.25
baseline_variance_v2 = 0.0001
polarization_variance_v2 = 0.0001
condition_variance = 0.01

[process_noise]
activity_variance_per_s = 1.0e-5
baseline_variance_v2_per_s = 1.0e-10
polarization_variance_v2_per_s = 1.0e-8
condition_variance_per_s = 1.0e-9
source = "configured"

[measurement_noise]
source = "signal_robust_variance"
configured_variance_v2 = 1.0e-6
minimum_variance_v2 = 1.0e-12
maximum_variance_v2 = 1.0

[polarization]
tau_source = "transient"
transient_parameter = "tau_slow"
aggregation = "median"
configured_tau_s = 30.0
gain = 1.0

[environment]
temperature_series = "temperature"
conductivity_series = "conductivity"
ionic_strength_series = "ionic_strength"
flow_series = "flow"
alignment = "linear_interpolation"
maximum_gap_s = 60.0
window_half_width_s = 5.0
allow_configured_fallback = true
fallback_temperature_celsius = 25.0

[observability]
enabled = true
horizon_steps = 20
rank_tolerance = 1.0e-8
maximum_condition_number = 1.0e10
reject_unobservable_model = true
empirical_perturbation = 1.0e-3
empirical_sensitivity_tolerance = 1.0e-8

[equilibrium_recognition]
enabled = true
minimum_history_points = 5
minimum_elapsed_time_constants = 5.0
maximum_normalized_state_rate_per_s = 1.0e-4
maximum_dynamic_potential_v = 1.0e-4
maximum_equilibrium_gap_v = 1.0e-4
maximum_absolute_standardized_innovation = 2.0
maximum_absolute_residual_autocorrelation = 0.2
maximum_environment_change_fraction = 0.01
maximum_state_uncertainty_fraction = 0.05
require_observable = true

[ekf]
numerical_jacobian_relative_step = 1.0e-6
use_joseph_update = true

[ukf]
alpha = 0.001
beta = 2.0
kappa = 0.0
initial_jitter = 1.0e-12
jitter_multiplier = 10.0
maximum_jitter_attempts = 5

[extrapolation]
warn_outside_domain = true
inflate_measurement_variance = false
variance_inflation_factor = 4.0
near_boundary_fraction = 0.05
near_boundary_variance_inflation_factor = 1.25

[validation]
alignment_policy = "nearest_within_tolerance"
maximum_alignment_gap_s = 0.5
allow_truth_reuse = false

[validation.states.log10_activity]
absolute_convergence_tolerance = 0.05
minimum_consecutive_converged_points = 1
step_detection_threshold = 1.0e-6
step_response_fraction = 0.9

[validation.states.baseline_offset]
absolute_convergence_tolerance = 0.001

[validation.states.polarization]
absolute_convergence_tolerance = 0.001

[validation.states.sensitivity_scale]
absolute_convergence_tolerance = 0.02

[auxiliary]
condition_requires_auxiliary = true
allow_known_standard_events = true
allow_reference_activity = true
known_log10_activity_variance = 1.0e-8

[plotting]
enabled = true
include_state_uncertainty = true
include_innovations = true
include_nis = true
include_environment = true
include_covariance = true

[export]
results_filename = "state_estimation.json"
states_filename = "state_estimates.csv"
innovations_filename = "state_innovations.csv"
diagnostics_filename = "state_diagnostics.json"
validation_filename = "state_validation.json"
report_filename = "state_estimation_report.txt"

[timestamp_handling]
duplicate_policy = "deduplicate_identical"
non_monotonic_policy = "segment_on_reset"
non_finite_policy = "reject"
minor_reversal_threshold_s = 1.0
reset_threshold_s = 10.0
reset_threshold_fraction = 0.5
minimum_segment_points = 10

# Existing workflows intentionally remain on the direct legacy backend.
# Set backend = "compiled" to opt in to either compiled profile.
[model]
backend = "legacy"
profile = "legacy_equivalent_v1"

[model.input_bindings]
target_activity = "estimated_activity"
delta_log10_activity = "experiment_activity_step"
temperature = "environment:temperature"
conductivity = "environment:conductivity"

[model.transduction_drive]
source = "none"

[model.observation_variance]
combination = "estimation_only"
```

</details>

### health.toml

[Repository file](../../config/health.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1

[baseline]
minimum_required_records = 3
robust_statistics = true

[comparability]
require_same_analyte = true
require_same_sample_matrix = true
maximum_temperature_difference_k = 2.0
require_same_sensor_design = true

[normalization]
use_relative_difference = true
use_robust_z_score = true
minimum_baseline_records_for_z_score = 5

[assessment]
minimum_domains_for_assessment = 2
minimum_domains_for_mechanistic_finding = 2
allow_warning_artifacts = true

[[rules]]
rule_id = "elevated-noise"
finding = "elevated_noise"
severity = "moderate"
minimum_evidence_domains = 1

[[rules.all_of]]
feature = "signal.robust_noise_standard_deviation"
operator = "robust_z_greater_than"
value = 3.0

[[rules]]
rule_id = "probable-fouling"
finding = "probable_fouling"
severity = "major"
minimum_evidence_domains = 2
alternative_explanations = ["environmental mismatch", "incomplete baseline context"]

[[rules.all_of]]
feature = "transient.tau_slow"
operator = "relative_increase_greater_than"
value = 1.0

[[rules.any_of]]
feature = "calibration.slope_efficiency"
operator = "relative_decrease_greater_than"
value = 0.20

[[rules.any_of]]
feature = "eis.role.transport.relaxation_timescale"
operator = "relative_increase_greater_than"
value = 1.0

[plotting]
enabled = true
```

</details>

### mechanism.toml

[Repository file](../../config/mechanism.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1

[eis]
allow_warning_fits = true
require_uncertainty = false
confidence_level = 0.95
seed = 42

[transient]
allow_warning_fits = true
selected_model_only = true

[timescale]
monte_carlo_samples = 10000
seed = 42
frequency_boundary_margin = 0.1

[matching]
require_experiment_id = true
require_sensor_id = false

[comparison]
ratio_weak = 10.0
ratio_moderate = 3.0
ratio_strong = 1.5
log_distance_weak = 1.0
log_distance_moderate = 0.5
log_distance_strong = 0.1761
minimum_fit_quality = 0.0
compatibility_ratio_lower = 0.5
compatibility_ratio_upper = 2.0

[evidence]
minimum_replicates_for_strong = 3

[trend]
enabled = true
minimum_records = 3
independent_variable = "sensor_age_days"
```

</details>

### mhi_physical_approval_trust_store.schema1.json

[Repository file](../../config/mhi_physical_approval_trust_store.schema1.json)

<details>
<summary>Exact shipped contents</summary>

```json
{
  "schema_version": 1,
  "trust_store_id": "mhi_physical_approval_trust_store_v1",
  "provisioning_state": "UNPROVISIONED",
  "trust_roots": []
}
```

</details>

### model.toml

[Repository file](../../config/model.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1

[model]
schema_version = 3
model_id = "ism-reduced-order-v1"
description = "Reduced-order ISM baseline with explicit phenomenological modes."
validity_domain = "Aqueous potentiometric steps within declared calibration/activity domains."
uncertainty_incomplete = false

[[model.states]]
id = "fast_mode_v"
name = "fast mode voltage"
description = "Phenomenological reduced-order voltage state; no physical mechanism is implied."
unit = "V"
transformation = "identity"
initialization_source = "declared_default"
lower_bound = -10.0
upper_bound = 10.0
initial_value = 0.0
source = "default reduced-order model"
process_equation_version = 1
observability_requirements = ["State observability must be assessed before interpretation."]
validity_domain = "reduced-order voltage mode"
initial_uncertainty = { kind = "deterministic" }

[[model.states]]
id = "slow_mode_v"
name = "slow mode voltage"
description = "Phenomenological reduced-order voltage state; no physical mechanism is implied."
unit = "V"
transformation = "identity"
initialization_source = "declared_default"
lower_bound = -10.0
upper_bound = 10.0
initial_value = 0.0
source = "default reduced-order model"
process_equation_version = 1
observability_requirements = ["State observability must be assessed before interpretation."]
validity_domain = "reduced-order voltage mode"
initial_uncertainty = { kind = "deterministic" }

[[model.parameters]]
id = "standard_potential_v"
name = "standard potential"
description = "Reduced-order model parameter; domain-specific calibration is required."
unit = "V"
lower_bound = -2.0
upper_bound = 2.0
default_value = 0.0
uncertainty = { kind = "unknown", reason = "standard potential requires calibration covariance or an explicit prior" }
source = "default reduced-order model"
equation_version = 1
identifiability_requirements = ["Structural and practical identifiability must be assessed before interpretation."]
value_source = "externally_supplied"
validity_domain = "must be calibrated for the experimental domain"

[[model.parameters]]
id = "ion_charge"
name = "ion charge"
description = "Reduced-order model parameter; domain-specific calibration is required."
unit = "dimensionless"
lower_bound = -4.0
upper_bound = 4.0
default_value = 1.0
uncertainty = { kind = "deterministic" }
source = "default reduced-order model"
equation_version = 1
identifiability_requirements = ["Structural and practical identifiability must be assessed before interpretation."]
value_source = "fixed"
characteristic = "discrete_integer"
validity_domain = "must be declared for the analyte"

[[model.parameters]]
id = "fast_tau_s"
name = "fast mode time constant"
description = "Reduced-order model parameter; domain-specific calibration is required."
unit = "s"
lower_bound = 0.0001
upper_bound = 100000.0
default_value = 1.0
uncertainty = { kind = "unknown", reason = "fast time constant has no calibration covariance or explicit prior" }
source = "default reduced-order model"
equation_version = 1
identifiability_requirements = ["Structural and practical identifiability must be assessed before interpretation."]
value_source = "externally_supplied"
validity_domain = "must be calibrated for the experimental domain"

[[model.parameters]]
id = "fast_gain"
name = "fast mode gain"
description = "Reduced-order model parameter; domain-specific calibration is required."
unit = "dimensionless"
lower_bound = -10.0
upper_bound = 10.0
default_value = 1.0
uncertainty = { kind = "unknown", reason = "fast gain has no calibration covariance or explicit prior" }
source = "default reduced-order model"
equation_version = 1
identifiability_requirements = ["Structural and practical identifiability must be assessed before interpretation."]
value_source = "externally_supplied"
validity_domain = "must be calibrated for the experimental domain"

[[model.parameters]]
id = "slow_tau_s"
name = "slow mode time constant"
description = "Reduced-order model parameter; domain-specific calibration is required."
unit = "s"
lower_bound = 0.0001
upper_bound = 1000000.0
default_value = 30.0
uncertainty = { kind = "unknown", reason = "slow time constant has no calibration covariance or explicit prior" }
source = "default reduced-order model"
equation_version = 1
identifiability_requirements = ["Structural and practical identifiability must be assessed before interpretation."]
value_source = "externally_supplied"
validity_domain = "must be calibrated for the experimental domain"

[[model.parameters]]
id = "slow_gain"
name = "slow mode gain"
description = "Reduced-order model parameter; domain-specific calibration is required."
unit = "dimensionless"
lower_bound = -10.0
upper_bound = 10.0
default_value = 0.2
uncertainty = { kind = "unknown", reason = "slow gain has no calibration covariance or explicit prior" }
source = "default reduced-order model"
equation_version = 1
identifiability_requirements = ["Structural and practical identifiability must be assessed before interpretation."]
value_source = "externally_supplied"
validity_domain = "must be calibrated for the experimental domain"

[[model.parameters]]
id = "baseline_drift_v_per_s"
name = "baseline drift"
description = "Reduced-order model parameter; domain-specific calibration is required."
unit = "V/s"
lower_bound = -1.0
upper_bound = 1.0
default_value = 0.0
uncertainty = { kind = "unknown", reason = "drift rate has no calibration covariance or explicit prior" }
source = "default reduced-order model"
equation_version = 1
identifiability_requirements = ["Structural and practical identifiability must be assessed before interpretation."]
value_source = "externally_supplied"
validity_domain = "must be calibrated for the experimental domain"

[[model.parameters]]
id = "observation_noise_std_v"
name = "observation noise standard deviation"
description = "Reduced-order model parameter; domain-specific calibration is required."
unit = "V"
lower_bound = 0.0
upper_bound = 1.0
default_value = 0.001
uncertainty = { kind = "deterministic" }
source = "default reduced-order model"
equation_version = 1
identifiability_requirements = ["Structural and practical identifiability must be assessed before interpretation."]
value_source = "externally_supplied_fixed"
validity_domain = "must be estimated for the measurement system"

[[model.inputs]]
id = "primary_concentration"
unit = "mol/L"
required = true
source = "experiment input"
validity_domain = "positive finite concentration within calibration domain"

[[model.inputs]]
id = "temperature"
unit = "K"
required = true
source = "experiment input"
validity_domain = "positive absolute temperature within calibration domain"

[[model.inputs]]
id = "driving_step_v"
unit = "V"
required = true
source = "experiment input"
validity_domain = "finite declared step drive"

[[model.components]]
id = "equilibrium"
kind = "equilibrium.nernst"
role = "equilibrium"
interpretation_status = "phenomenological"
depends_on = []
required_inputs = [{ id = "primary_concentration", unit = "mol/L" }, { id = "temperature", unit = "K" }]
state_ids = []
parameter_ids = ["standard_potential_v", "ion_charge"]
observation_state_ids = []
observation_parameter_ids = ["standard_potential_v", "ion_charge"]
numerical_jacobian_supported = false
output_unit = "V"
voltage_contribution_owner = "equilibrium"
contribution_semantics = "additive_potential"
source = "Phase 03 built-in component"
validity_domain = "reduced-order component; no mechanism confirmation implied"
equation = "EQ-CAL-001 adapter"
equation_version = 1
assumptions = ["Phenomenological component terms are not mechanism labels."]
evidence_requirements = [{ hypothesis_id = "equilibrium-mechanism", proposed_mechanism_label = "unassigned", independent_evidence_types = ["independent experiment"], minimum_independent_observations = 2, validity_domain = "declared model domain", alternatives_to_consider = ["other reduced-order explanations"], required_uncertainty_statement = "parameter uncertainty is required before interpretation" }]
metadata = { activity_model = "ideal" }

[[model.components]]
id = "fast_mode"
kind = "transport.first_order_relaxation"
role = "transport"
interpretation_status = "phenomenological"
depends_on = []
required_inputs = [{ id = "driving_step_v", unit = "V" }]
state_ids = ["fast_mode_v"]
parameter_ids = ["fast_tau_s", "fast_gain"]
observation_state_ids = ["fast_mode_v"]
observation_parameter_ids = []
numerical_jacobian_supported = false
output_unit = "V"
voltage_contribution_owner = "fast_mode"
contribution_semantics = "additive_potential"
source = "Phase 03 built-in component"
validity_domain = "reduced-order component; no mechanism confirmation implied"
equation = "EQ-TR-001 adapter"
equation_version = 1
assumptions = ["Fast is a relative phenomenological label, not a physical identity."]
evidence_requirements = [{ hypothesis_id = "fast-mode-mechanism", proposed_mechanism_label = "unassigned", independent_evidence_types = ["independent experiment"], minimum_independent_observations = 2, validity_domain = "declared model domain", alternatives_to_consider = ["other reduced-order explanations"], required_uncertainty_statement = "parameter uncertainty is required before interpretation" }]
metadata = {}

[[model.components]]
id = "slow_mode"
kind = "transport.first_order_relaxation"
role = "transport"
interpretation_status = "phenomenological"
depends_on = []
required_inputs = [{ id = "driving_step_v", unit = "V" }]
state_ids = ["slow_mode_v"]
parameter_ids = ["slow_tau_s", "slow_gain"]
observation_state_ids = ["slow_mode_v"]
observation_parameter_ids = []
numerical_jacobian_supported = false
output_unit = "V"
voltage_contribution_owner = "slow_mode"
contribution_semantics = "additive_potential"
source = "Phase 03 built-in component"
validity_domain = "reduced-order component; no mechanism confirmation implied"
equation = "EQ-TR-001 adapter"
equation_version = 1
assumptions = ["Slow is a relative phenomenological label, not a physical identity."]
evidence_requirements = [{ hypothesis_id = "slow-mode-mechanism", proposed_mechanism_label = "unassigned", independent_evidence_types = ["independent experiment"], minimum_independent_observations = 2, validity_domain = "declared model domain", alternatives_to_consider = ["other reduced-order explanations"], required_uncertainty_statement = "parameter uncertainty is required before interpretation" }]
metadata = {}

[[model.components]]
id = "baseline_drift"
kind = "disturbance.linear_drift"
role = "external_disturbance"
interpretation_status = "phenomenological"
depends_on = []
required_inputs = []
state_ids = []
parameter_ids = ["baseline_drift_v_per_s"]
observation_state_ids = []
observation_parameter_ids = ["baseline_drift_v_per_s"]
numerical_jacobian_supported = false
output_unit = "V"
voltage_contribution_owner = "baseline_drift"
contribution_semantics = "additive_potential"
source = "Phase 03 built-in component"
validity_domain = "reduced-order component; no mechanism confirmation implied"
equation = "linear covariate offset"
equation_version = 1
assumptions = ["Drift is not assigned to a physical failure mechanism."]
evidence_requirements = [{ hypothesis_id = "baseline-drift-mechanism", proposed_mechanism_label = "unassigned", independent_evidence_types = ["independent experiment"], minimum_independent_observations = 2, validity_domain = "declared model domain", alternatives_to_consider = ["other reduced-order explanations"], required_uncertainty_statement = "parameter uncertainty is required before interpretation" }]
metadata = {}

[[model.components]]
id = "observation_noise"
kind = "disturbance.stochastic_observation_noise"
role = "observation_noise"
interpretation_status = "phenomenological"
depends_on = []
required_inputs = []
state_ids = []
parameter_ids = ["observation_noise_std_v"]
observation_state_ids = []
observation_parameter_ids = []
numerical_jacobian_supported = false
output_unit = "V^2"
contribution_semantics = "observation_variance"
source = "Phase 03 built-in component"
validity_domain = "reduced-order component; no mechanism confirmation implied"
equation = "zero-mean observation-noise declaration"
equation_version = 1
assumptions = ["Zero-mean noise is uncertainty, not a deterministic voltage contribution."]
evidence_requirements = [{ hypothesis_id = "observation-noise-mechanism", proposed_mechanism_label = "unassigned", independent_evidence_types = ["independent experiment"], minimum_independent_observations = 2, validity_domain = "declared model domain", alternatives_to_consider = ["other reduced-order explanations"], required_uncertainty_statement = "parameter uncertainty is required before interpretation" }]
metadata = {}
```

</details>

### parsing.toml

[Repository file](../../config/parsing.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1
fallback_model = "R0-p(CPE1,R1)"

# ── Model Selection ─────────────────────────────────────────────────────
[model_selection]
# Ranking metric: "aic" or "weighted_rmse"
ranking_metric = "aic"
# AIC threshold for preferring Warburg-based models
warburg_aic_threshold = 4.0

# ── Circuit Model Selection Rules ───────────────────────────────────────
# Rules are evaluated in order (first match wins).
# When a rule's filename_contains and metadata_contains conditions are all
# satisfied for a given input file, its circuit_model is used.
#
# Resolution order:
#   1. Explicit circuit= or model= tag in the filename
#   2. Explicit equivalentcircuit / circuitmodel / circuit / model value
#      in file metadata
#   3. First matching [[rules]] entry below
#   4. fallback_model above

[[rules]]
circuit_model = "R0-p(CPE1,R1)-Gw2"
filename_contains = ["ism"]

[[rules]]
circuit_model = "R0-p(CPE1,R1)"
filename_contains = ["qd"]

[[rules]]
circuit_model = "R0-W1"

[rules.metadata_contains]
equivalentcircuit = "R0-W1"
```

</details>

### plotting.toml

[Repository file](../../config/plotting.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1

# ── Shared paths and global style baseline ──────────────────────────────
# All paths are resolved relative to the config file's directory
# (i.e., the config/ subdirectory under the workspace root).
# Absolute paths are also accepted.
[shared]
# input_path: Directory or single file containing input data.
# Relative paths are resolved from config/ (so "../data" → <workspace>/data).
input_path = "../data"

# output_path: Directory where generated figures and reports are written.
output_path = "../output"

# output_prefix: Optional prefix prepended to every output filename.
# Use "" (empty) for no prefix.
output_prefix = ""

# input_is_directory: When true, input_path is treated as a directory
# and all supported files within it are processed. When false, input_path
# is treated as a single file.
input_is_directory = true

# Global style defaults inherited by all plot workflows (EIS, regular, generic).
# Per-job style overrides and named presets take precedence over these values.
[shared.style]
# Figure resolution (DPI)
dpi = 300.0
# Figure dimensions in inches
width_inches = 7.2
height_inches = 5.2
# Base font size in points
font_size_pt = 21.0
# General line width
line_width = 6
# Marker sizes
experimental_marker_radius = 8
marker_radius = 8
# Line widths for different data types
experimental_line_width = 6
fitted_line_width = 6
series_line_width = 6
# Colors
experimental_color = "#0000ff"
fitted_color = "#ff6a00"
# Legend placement
legend_position = "upper_right"
# PNG supersampling factor
png_scale_factor = 2
# Show/hide individual data point markers
show_points = false

# ── Global rendering knobs ──────────────────────────────────────────────
# These settings apply on top of domain defaults but below per-job styles.
[render]
# PNG supersampling factor (default: 2)
png_scale_factor = 2
# PNG output DPI (overrides shared.style.dpi for PNG output)
png_dpi = 300.0

# ── Named style presets ─────────────────────────────────────────────────
# Presets are referenced by a workflow's `style_preset` field.
# They follow the same structure: [style_presets.<name>.style],
# [style_presets.<name>.individual_style],
# [style_presets.<name>.combined_style]

[style_presets.paper.individual_style]
experimental_color = "#000dff"
fitted_color = "#ff6a00cc"

[style_presets.paper.combined_style]
series_palette = [
    "#000dff",
    "#B25019",
    "#2E7D32",
    "#fb0606",
    "#0b4282",
    "#6A1B9A",
    "#F9A825",
    "#f20ce6",
    "#4E342E",
]
legend_position = "lower_right"
```

</details>

### signal.toml

[Repository file](../../config/signal.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1

[windowing]
source = "stable_experiment_region"
exclude_before_event_s = 10.0
exclude_after_event_s = 300.0
eligible_event_kinds = ["concentration_step", "flow_change", "temperature_change", "interferent_addition"]

[sampling]
policy = "require_regular"
regularity_relative_tolerance = 0.01
resample_interval_s = 1.0
maximum_interpolation_gap_s = 5.0

[statistics]
confidence_level = 0.95

[psd]
enabled = true
segment_points = 256
overlap_fraction = 0.5
window = "hann"
detrend = "linear"
parseval_tolerance = 0.10

[allan]
enabled = true
minimum_clusters = 8
tau_points = 30

[drift]
models = ["ordinary_linear", "theil_sen"]
minimum_duration_s = 300.0

[spikes]
enabled = true
method = "hampel"
window_points = 11
mad_threshold = 4.0

[correlation]
enabled = true
maximum_lag_s = 60.0

[plotting]
enabled = true
```

</details>

### transient.toml

[Repository file](../../config/transient.toml)

<details>
<summary>Exact shipped contents</summary>

```toml
schema_version = 1

[segmentation]
pre_event_s = 30.0
post_event_s = 300.0
baseline_window_s = 20.0
minimum_points = 20
minimum_duration_s = 10.0
maximum_missing_fraction = 0.20
duplicate_timestamp_policy = "error"
non_monotonic_policy = "sort"
irregular_sampling_policy = "allow"

[baseline]
method = "median"
response_mode = "baseline_relative"

[models]
enabled = ["single", "double", "double_drift", "stretched"]
beta_min = 0.05
beta_max = 1.0

[optimizer]
maximum_iterations = 400
ftol = 1e-10
xtol = 1e-10
gtol = 1e-10
patience = 400
step_bound = 50.0
multiple_starts = 8

[selection]
criterion = "aic"

[validation]
minimum_tau_ratio = 3.0
maximum_tau_to_window_ratio = 1.0
negligible_amplitude_fraction = 0.05
high_autocorrelation_threshold = 0.8
bound_proximity_fraction = 0.01

[uncertainty]
bootstrap_iterations = 500
confidence_level = 0.95
seed = 42
minimum_success_fraction = 0.80

[plotting]
enabled = true
include_components = true
include_residuals = true
include_model_comparison = true

[export]
json_filename = "transient_results.json"
features_filename = "transient_features.csv"
model_comparison_filename = "transient_model_comparison.csv"
report_filename = "transient_report.txt"
```

</details>


## Accepted fields with limited or no effect

**VERIFIED IMPLEMENTATION — audit caveat:** a deserialized setting is not necessarily connected to an analysis. The following fields have no corresponding production analysis read in this revision. The defaults table describes their serialized values, not a promise of effective control. Preserve them for round-tripping if needed, but do not rely on changing them to change results.

| Workflow | Accepted fields without an effective analysis control |
|---|---|
| calibration | `activity.conductivity_empirical.form`, `plotting.include_confidence_band`, `plotting.include_hysteresis`, `plotting.include_validation`, `validation.prediction_interval_confidence`, `activity.user_provided_activity_field` |
| signal | `psd.minimum_frequency_hz`, `psd.maximum_frequency_hz`, `psd.segment_duration_s`, `windowing.relative_start_s`, `windowing.relative_end_s`, `spikes.window_duration_s` |
| health | `assessment.allow_warning_artifacts`, `baseline.robust_statistics` |
| estimation | `auxiliary.allow_reference_activity`, `ekf.numerical_jacobian_relative_step`, `ekf.use_joseph_update`, `plotting.include_covariance`, `plotting.include_environment`, `plotting.include_innovations`, `plotting.include_nis`, `plotting.include_state_uncertainty`, `measurement_noise.per_observation_variance`, `initialization.previous_artifact`, `initialization.steady_window_s`, `extrapolation.warn_outside_domain` |

For Phase-C health, `maximum_reference_alignment_difference_s`, `causal_promotion.minimum_independent_supporting_evidence` and the hypothesis binding `relationship` are declared/validated contract inputs, but no configurable evaluation use was found. Do not infer stronger alignment or independent-support enforcement merely by changing them.

Signal `windowing.source` labels the source but does not implement different relative-event window algorithms: the current selector uses absolute `start_s`/`end_s` and event exclusions. PSD `window` applies Hann only for case-insensitive `hann`; any other string uses rectangular weights. `allan.minimum_clusters` triggers a warning rather than removing points or excluding them from minimum selection. Check cluster counts yourself. PSD frequency range fields above do not crop the spectrum.

Estimation initialization uses known standard events when allowed and in standard context; otherwise an `activity_source` containing `configured` selects the configured activity, and other values attempt calibration inversion of the first valid measurement with configured fallback. It does not resume `previous_artifact` or average `steady_window_s`. The EKF settings above do not select a different update equation or Jacobian step in the current engine. Plot include flags listed above do not independently enable/disable those figures.

These limitations are recorded as an implementation issue group in the [coverage report](../USER_MANUAL_COVERAGE.md); no implementation was changed.

The complete nested [Phase-B, Phase-C and Phase-E field and enum catalog](advanced_contracts.md) supplements these shipped configurations.
