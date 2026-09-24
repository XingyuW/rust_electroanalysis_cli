# End-to-end workflows and common recipes

[Master manual](../USER_MANUAL.md) · [Command reference](command_reference.md) · [Configuration](configuration.md) · [Outputs](outputs.md)

The recipes below connect implemented commands. **EXAMPLE — illustrative unless explicitly marked executed:** file paths such as `raw/ocp.csv`, `metadata/experiment.toml`, manifests and custom configs must be prepared for your experiment. Use the actual binary on PATH, or replace it with the `BIN` variable in the quick start. All 30 leaf command forms were also executed in the audit workspace; those successful invocations and failed preliminary attempts are recorded in the coverage report.

Choose separate output directories for steps that share filenames. Review warnings after each step; do not automate advancement solely on exit status. Repeatability and independence require different evidence: replicates derived from one acquisition do not become independent by receiving different record IDs.

## Workflow A — Basic EIS analysis

**Question:** how well does a justified circuit explain this spectrum?

1. Prepare Hz/ohm signed-complex data, preserve raw exports, and inspect the frequency span and missing points.
2. Configure a single-file plot. With this TOML saved as `config/eis_plot.toml`, paths resolve from `config/`:

```toml
schema_version = 1
[shared]
input_path = "../raw/eis.csv"
output_path = "../results/eis_plot"
input_is_directory = false
output_prefix = "sample01"
```

3. Plot and fit, explicitly saving the JSON needed downstream:

```bash
rust_electroanalysis_cli plot eis --plot-config config/eis_plot.toml
rust_electroanalysis_cli eis fit raw/eis.csv --circuit 'R0-p(R1,CPE1)' \
  --artifact results/eis/eis_fit.json --report results/eis/eis_report.txt
rust_electroanalysis_cli signal residuals --eis-fit results/eis/eis_fit.json \
  --output results/eis_residuals
```

Inspect Nyquist/Bode overlays, all parameter units, fit boundaries, covariance rank and frequency-dependent residuals. `eis export-fit raw/eis.csv --circuit 'R0-p(R1,CPE1)' --artifact results/eis/export.json` is an alternative that **fits again**; it does not convert the text report. See [fit reference](command_reference.md#eis-fit).

## Workflow B — ECM discovery

**Question:** which candidates are statistically competitive before physical discrimination?

```bash
rust_electroanalysis_cli eis search raw/eis.csv --search-config config/analysis.toml \
  --search-output results/search --search-top 5
```

Read the `<input-identity>_ecm_search.csv` ranking and TXT report. Compare BIC/AIC only across the same input/preprocessing/objective. Reject candidates with implausible bounds or structured residuals even if ranked first. Copy a selected circuit expression into an explicit `eis fit --circuit ... --artifact ...` invocation, then use Workflow A's residual check.

A directory search uses a common worksheet selector if supplied and records partial failures. Search reports default beside raw inputs if no output override is supplied; provide one to keep the raw directory immutable. The finite GA search is not exhaustive. A small population/generation count is useful for learning, not a guarantee of robust circuit identification. Shipped search plots are enabled; omit `plotting.top_n` in an explicit custom search file to disable them—zero is rejected.

## Workflow C — Potentiometric transient analysis

**Question:** which relaxation descriptors are supported by the observed OCP response?

Prepare a potential channel, seconds-based event times and metadata identifying each concentration step. Ensure enough pre-step baseline and post-step acquisition for the slowest anticipated response. Then:

```bash
rust_electroanalysis_cli transient fit --input raw/ocp.csv \
  --metadata metadata/experiment.toml --channel 'E1/V' \
  --config config/transient.toml --model all --selection bic \
  --seed 42 --output results/transient
```

Use `--sheet` for a workbook with multiple compatible sheets, and `--event-index` only after confirming eligible event ordering. Read `transient_model_comparison.csv` and each selected fit's status. Compare tau, amplitudes, baseline/asymptote, drift, confidence intervals, mode separation and residual autocorrelation. Reject short-window slow-mode interpretations that depend mainly on extrapolation.

The quick-start fixture is an **executed** single-model example with bootstrap disabled. For real uncertainty analysis use justified bootstrap settings and inspect failed resample counts. The output JSON feeds calibration equilibrium extraction, mechanism comparison, health and estimation priors.

## Workflow D — Calibration

**Question:** does potential predict activity/concentration over a declared experimental domain?

Use multiple concentration levels with known final concentration, analyte charge, temperature and matrix. Preserve ascending/descending branches if studying hysteresis. Provide transient results for model-derived equilibrium or explicitly choose a stable-window source. A minimal explicit extraction override must include the source; missing fields do not inherit the shipped fallback:

```toml
schema_version = 1
[observation_extraction]
preferred_source = "steady_state_median"
steady_state_start_s = 180.0
steady_state_end_s = 300.0
minimum_points = 20
[analyte]
name = "K+"
charge = 1
```

Save that as `config/calibration_windows.toml`, or copy the full shipped file and change only justified fields.

```bash
rust_electroanalysis_cli calibration extract --input raw/standards.csv \
  --metadata metadata/standards.toml --channel 'E1/V' \
  --config config/calibration_windows.toml --output results/calibration
rust_electroanalysis_cli calibration fit \
  --observations results/calibration/calibration_observations.json \
  --config config/calibration.toml --model nernst --seed 42 \
  --output results/calibration
rust_electroanalysis_cli calibration validate \
  --model results/calibration/calibration_model.json \
  --observations results/heldout/calibration_observations.json \
  --output results/calibration_validation
rust_electroanalysis_cli calibration predict \
  --model results/calibration/calibration_model.json --potential 0.12252 \
  --temperature 25 --output results/prediction.json
```

Generate held-out observations by running extract on an independent held-out experiment. Reusing training observations tests plumbing, not generalization. For transient-derived calibration add `--transient-results results/transient/transient_results.json` to extract and use a config permitting that source.

Inspect observation exclusions/source, calibration slope versus theoretical slope, valid domain, residuals, CV errors, hysteresis and prediction extrapolation. Do not transfer the calibration to another matrix merely because the CLI accepts a voltage. `--output prediction.csv` selects CSV; batch prediction needs `--input` and `--channel` and has no worksheet selector.

### Synthetic calibration exercise

The audit executed a six-level synthetic exercise: 400 samples per level at 1 s intervals, levels `1e-6` through `1e-1 mol/L`, potential `0.3 + 0.05916*log10(c) + 1e-5*sin(local_second)`, events at 0/400/800/1200/1600/2000 s. A `steady_state_median` config, bootstrap=0 and plots disabled yielded six extracted observations and successful fit/validate/predict. This is a reproducible mathematical exercise, not measured sensor validation. Its exact generated CSV/TOML and execution log remain in the audit cache named in the coverage report.

## Workflow E — Mechanism timescale analysis

**Question:** are selected EIS and OCP characteristic times compatible under matched conditions?

```bash
rust_electroanalysis_cli mechanism compare \
  --eis-artifact results/eis/eis_fit.json \
  --transient-artifact results/transient/transient_results.json \
  --metadata metadata/experiment.toml --config config/mechanism.toml \
  --output results/mechanism
```

Read `characteristic_timescales.csv` (including derivation, source parameter and uncertainty) and `timescale_comparisons.csv`, then inspect timescale-map and ratio plots. A pair near unity supports temporal compatibility, **not** an automatic double-layer/diffusion/binding assignment. EIS and transient measurements need defensibly comparable sensor, matrix, temperature and operational state.

For repeated experiments, a legacy mechanism manifest (**illustrative**) looks like:

```toml
schema_version = 1
[[records]]
record_id = "day1"
experiment_id = "exp-day1"
sensor_id = "sensor-01"
eis_fit = "day1/eis/eis_fit.json"
transient_results = "day1/transient/transient_results.json"
condition = "buffer-A"
sensor_age_days = 1.0
[[records]]
record_id = "day2"
experiment_id = "exp-day2"
sensor_id = "sensor-01"
eis_fit = "day2/eis/eis_fit.json"
transient_results = "day2/transient/transient_results.json"
condition = "buffer-A"
sensor_age_days = 2.0
```

Add enough records for the configured minimum (shipped 3), save it alongside its `day*/` folders, and run:

```bash
rust_electroanalysis_cli mechanism trend --manifest results/mechanism_manifest.toml \
  --config config/mechanism.toml --output results/mechanism_trend
```

This rereads paired EIS/transient artifacts; it is not a manifest of mechanism reports. The legacy trend uses sensor_age_days even if an alternative independent-variable label is configured. For explicit Phase-B evidence, design a separate hypothesis contract and supply `--mechanism-evidence-config`; do not assume the legacy timescale chart already satisfies its identity, temporal, amplitude, independence or identifiability gates.

## Workflow F — Signal quality analysis

**Question:** how much noise, drift, temporal instability and contamination are present?

```bash
rust_electroanalysis_cli signal characterize --input raw/ocp.csv \
  --metadata metadata/experiment.toml --channel 'E1/V' \
  --config config/signal.toml --output results/signal
```

First inspect `sampling` and selected window. Then check robust noise, PSD/ASD, Allan curves, drift and spikes. A long exclusion interval after each event can leave too little data for the PSD or Allan analysis. Choose a justified stable region explicitly if needed; never interpret an empty table as zero noise.

Comparison manifest (**illustrative**):

```toml
schema_version = 1
[[records]]
record_id = "replicate-1"
category = "buffer-A"
input = "../raw/replicate1.csv"
channel = "E1/V"
```

Add records and run `signal compare --manifest metadata/signal_manifest.toml --output results/signal_comparison`. This uses raw measurements. Its optional metadata field is currently not used for event windows; ensure comparable windows through the selected signal config/inputs. To inspect fit errors, run `signal residuals` with the typed transient/calibration/EIS artifact flags. Calibration residual PSD uses observation index, not acquisition seconds.

## Workflow G — Sensor health

**Question:** is the sensor's current behavior unlike comparable reference behavior?

Create independent reference acquisitions, run the upstream analyses, and write a baseline manifest:

```toml
schema_version = 1
[[records]]
record_id = "reference-1"
signal_results = "reference1/signal/signal_results.json"
transient_results = "reference1/transient/transient_results.json"
metadata = "../metadata/reference1.toml"
```

Add at least the configured number of comparable records (shipped minimum 3; robust z scoring normally needs 5). Do not repeat the same artifact to meet a scientific minimum.

```bash
rust_electroanalysis_cli health baseline --manifest results/baseline_manifest.toml \
  --config config/health.toml --output results/baseline
rust_electroanalysis_cli health assess --signal-results results/new/signal/signal_results.json \
  --transient-results results/new/transient/transient_results.json \
  --baseline results/baseline/health_baseline.json --metadata metadata/new.toml \
  --config config/health.toml --output results/new/health
```

Inspect available domains and baseline comparability before findings. Add calibration/EIS/mechanism evidence only when actually applicable. To use Phase-C dimension reporting, supply the separate `--phase-c-config` contract and its matching evidence; the legacy route writes schema 3 by design.

Health **trend uses a different manifest** of saved assessments:

```toml
schema_version = 1
[[records]]
record_id = "new-1"
assessment = "new1/health/health_assessment.json"
independent_value = 1.0
[[records]]
record_id = "new-2"
assessment = "new2/health/health_assessment.json"
independent_value = 2.0
```

Before running it, copy the full health config and add `[export] trends_filename="health_trends.json"` to avoid the default JSON/CSV collision. Run `health trend --manifest results/health_trend_manifest.toml --baseline results/baseline/health_baseline.json --config config/health_trend.toml --output results/health_trend`. Use exported per-record values for external trend fitting because the current summary-slope implementation has the issue documented in [scientific methods](scientific_methods.md#health-evidence).

## Workflow H — State estimation

**Question:** can activity and response states be estimated under an explicit observation/dynamics/noise model?

```bash
rust_electroanalysis_cli estimate run --input raw/ocp.csv \
  --metadata metadata/experiment.toml --channel 'E1/V' \
  --calibration-model results/calibration/calibration_model.json \
  --signal-results results/signal/signal_results.json \
  --transient-results results/transient/transient_results.json \
  --config config/estimation.toml --filter ukf --output results/estimation
rust_electroanalysis_cli estimate validate --results results/estimation/state_estimation.json \
  --truth raw/independent_truth.csv --output results/state_validation
rust_electroanalysis_cli estimate compare --input raw/ocp.csv \
  --metadata metadata/experiment.toml --channel 'E1/V' \
  --calibration-model results/calibration/calibration_model.json \
  --config config/estimation.toml --filters ekf,ukf --output results/filter_comparison
```

Truth must use the CSV schema accepted by state validation; the simulation truth CSV is a concrete example. Current CLI validation calls the CSV reader, not a general arbitrary JSON truth importer. Filter comparison has fewer optional artifact flags than estimate run; choose a config with appropriate configured noise/prior fallbacks for a fair comparison. Mixed unknown filter tokens are silently skipped if a valid token remains, so inspect which filters are reported.

### Executed synthetic estimation exercise

Run `estimate simulate --seed 42 --output output/simulation`. It emits a compatible calibration model, `simulation_measurements.csv` and `simulation_truth.csv`, but no metadata TOML. The audit supplied:

```toml
experiment_id = "manual-simulation"
sample_matrix = "synthetic"
[sensor]
analyte = "Na+"
```

and an explicit estimator override:

```toml
schema_version = 3
[state_model]
kind = "activity"
[measurement_noise]
source = "configured"
[plotting]
enabled = false
```

The run/compare examples in the command reference use those prepared inputs. This activity-only exercise avoids implying that one voltage channel identifies arbitrary simultaneous baseline, polarization and activity changes. Inspect validation errors and innovations; synthetic recoverability is not physical validation.

## Workflow I — ISM model workflow

**Question:** is a declared reduced-order model structurally valid, and how do its contributions behave?

```bash
rust_electroanalysis_cli model validate --model config/model.toml --output results/model_check
rust_electroanalysis_cli model simulate --model config/model.toml --steps 100 --dt-s 1 \
  --output results/model_simulation
rust_electroanalysis_cli model decompose --model config/model.toml \
  --input metadata/model_inputs.json --measurement raw/ocp.csv \
  --metadata metadata/experiment.toml \
  --calibration-model results/calibration/calibration_model.json \
  --output results/model_decomposition
rust_electroanalysis_cli model report --results results/model_decomposition/model_analysis.json \
  --output results/model_decomposition/summary.txt
```

Provide explicit time-aligned scientific ModelInput quantities (concentration/activity, temperature and any required drive); voltage alone is not converted into concentrations by decompose. Inspect validity, identifiability, named contributions and unexplained residual. `model validate` without a manifest checks compilation; a JSON study manifest is a separate evaluation route:

```json
{"schema_version":1,"study_id":"study-001","experiments":[{"experiment_id":"exp-001","category":"concentration_steps","sensor_id":"sensor-01","analysis_path":"model_decomposition/model_analysis.json","is_real_experiment":false}],"model_comparison_paths":[]}
```

Save relative to its referenced analysis and run `model validate --manifest results/study.json --output results/model_validation`. Supported study categories include stable standards, steps/reverse steps, ionic strength, temperature, interferents, flow, reference substitution, membrane thickness, solid contact, controlled fouling, aging and paired EIS/transient. A declared category or is_real_experiment flag is not independent verification of physical validity.

## Workflow J — Public reporting and Phase-E validation

**Question:** can frozen evidence be presented and assessed without recomputing measurements?

```bash
rust_electroanalysis_cli report render --mechanism results/phase_b/mechanism_results.json \
  --health results/phase_c/health_assessment.json --lineage-catalog metadata/lineage.json \
  --eis results/eis/eis_fit.json --transient results/transient/transient_results.json \
  --output-dir results/public_report --format all
rust_electroanalysis_cli validation run --protocol metadata/protocol.toml \
  --dataset validation_cohort/dataset.json --output-dir results/validation
```

The report renderer reads frozen artifacts and records available/unavailable figures/tables. The validation runner needs a complete source-hashed cohort, reference labels and protocol; it does not infer physical truth from a mechanism label. Both use managed output directories. Do not add `--overwrite` until the destination is the intended existing managed bundle; edited/foreign files can prevent replacement.

The audit executed a repository Phase-D fixture render and a **software-only** Phase-E fixture. To reproduce the Phase-E layout, copy `tests/fixtures/phase_e/dataset/software_valid.schema1.json` as `dataset.json`, `lineage/complete.schema1.json` into sibling `lineage/`, and these fixtures into sibling `sources/`: `mechanism/supported.schema4.json` → `mechanism_a.schema4.json`; `mechanism/all_levels.schema4.json` → `mechanism_c.schema4.json`; `health/within_baseline.schema4.json` → `health_a.schema4.json`; `health/alert.schema4.json` → `health_c.schema4.json`. Keep bytes unchanged so source hashes remain valid. Select `tests/fixtures/phase_e/protocol/software_valid.toml`. Fixture-declared physical labels are test content, not a real experimental claim.

Software/conformance success, physical validation and Phase-F REAL authority are distinct. The production physical trust store is unprovisioned; no instruction in this workflow provisions it or advances REAL authority.

## Recommended workflow for non-equilibrium ISM research

**Question:** how do fast, medium and slow voltage processes change with concentration, matrix and ionic composition?

1. Plan controls and independent experimental units before fitting. Record sensor design/age, membrane/reference, matrix/composition, temperature, flow, concentration values and intervention times. Include stable plateaus and reverse steps where feasible.
2. Preserve raw OCP and EIS separately; create shared identities only when they truly refer to comparable experimental conditions. EIS collected under one state and OCP under another can have similar numbers without comparable mechanisms.
3. Run Workflow C for each potential channel. Start with candidate single/double/stretched/drift models; use observation duration, sampling rate and residuals to decide whether fast and slow modes are resolvable. A third “medium” mechanism is not automatically an implemented three-exponential transient model: the current transient candidates have at most two exponential modes.
4. Run Workflows A/B for paired EIS. Inspect several plausible circuits and derive only topology-supported characteristic times. Preserve uncertainty and fit warnings.
5. Run Workflow E to compare timescales. Keep amplitude, temporal support, matrix and environmental context in the interpretation. Use explicit Phase-B contracts when testing predeclared evidence requirements.
6. Run Workflow F on stable control segments and fit residuals. Distinguish drift/noise from a long relaxation. Event masking must not discard the very kinetics you seek to characterize.
7. Run Workflow D if equilibrium/late-window standards justify calibration. Non-equilibrium voltage is not automatically eligible for equilibrium concentration inversion.
8. Use Workflows H/I only with a specified reduced model, justified priors/noise and sufficient observability. Treat named fast/slow components as phenomenological unless independent evidence supports a physical assignment.
9. Assemble JSON, CSV, figures, metadata and notes. Analyze concentration/matrix/composition dependence externally where the CLI lacks the required multi-factor experimental model. The legacy mechanism trend is age-based; health trend summary slopes have a known issue.
10. Use controlled perturbations, reference substitutions, thickness/composition changes, replicates and external physical measurements to discriminate hypotheses. “Tau_EIS approximately equals tau_OCP” does not imply “therefore double-layer charging.” That assignment belongs to the researcher and independent evidence.

## Common recipes index

| Need | Recipe | Where to look afterward |
|---|---|---|
| Plot one EIS file | Workflow A | Nyquist/Bode plus fit text |
| Fit a known circuit | Workflow A / quick start | parameters, residuals, covariance |
| Search circuits | Workflow B | ranking CSV and candidate stability |
| Fit an OCP transient | Workflow C | event feature and model-comparison tables |
| Build/predict calibration | Workflow D | observation quality, stored domain and prediction flags |
| Characterize noise/drift | Workflow F | sampling/window before PSD/Allan |
| Compare EIS/OCP scales | Workflow E | derivation, context, uncertainty, evidence limits |
| Assess health | Workflow G | domain availability, baseline comparability, alternatives |
| Estimate latent states | Workflow H | observability, innovations and truth validation |
| Validate/decompose ISM model | Workflow I | definition validity, contributions and unexplained residual |
| Render/validate frozen evidence | Workflow J | manifests, unavailable evidence and claim ceiling |
