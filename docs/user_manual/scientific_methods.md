# Scientific methods and interpretation

[Master manual](../USER_MANUAL.md) · [Commands](command_reference.md) · [Outputs](outputs.md)

**VERIFIED IMPLEMENTATION** describes the calculations below; the interpretation advice is **SCIENTIFIC INTERPRETATION**, not an assertion of physical validation.

## EIS fitting and circuit grammar

A circuit is a series/parallel expression: `R0-p(CPE1,R1)` places a resistor in series with a parallel CPE/resistor branch. `-` connects series elements; `p(a,b,...)` connects parallel branches, which can themselves be compound circuits. Quote expressions in a shell. Numeric suffixes label elements; inspect generated parameter names rather than guessing parameter-vector order.

| Element token | Meaning | Parameter names and units |
|---|---|---|
| `R` | Resistance | R, ohm |
| `C` | Ideal capacitance | C, F |
| `L` | Ideal inductance | L, H |
| `W` | Semi-infinite Warburg form | sigma, ohm s^-1/2 |
| `CPE` | Constant-phase element | Q, ohm^-1 s^alpha; alpha, dimensionless |
| `Wo`, `Ws` | Finite-length open/short Warburg forms | Z0, ohm; tau, s |
| `La` | Fractional inductance | L, H s^(alpha-1); alpha |
| `Gw` | Generalized Warburg | sigma, ohm s^alpha; alpha |
| `G`, `Gs` | Gerischer / finite Gerischer forms | R_G, ohm; t_G, s; Gs also phi |
| `K` | Relaxation element | R, ohm; tau_k, s |
| `Zarc` | Distributed relaxation | R, ohm; tau_k, s; gamma |
| `Tlmq` | Transmission-line form | Rion, ohm; Qs, ohm^-1 s^gamma; gamma |
| `T` | Four-parameter transport form | A/B in ohm, a dimensionless, b in s |

The engine evaluates complex impedance at angular frequency `omega = 2*pi*f`, sums impedances in series and admittances in parallel. Examples: `Z_R=R`, `Z_C=1/(j*omega*C)`, `Z_CPE=1/(Q*(j*omega)^alpha)`, and the implemented `Z_W=sigma*(1-j)/sqrt(omega)`. Parameter constraints use transformations and conservative bounds; the reported boundary warnings matter. Q is capacitance only in the special ideal exponent limit, not for an arbitrary fitted CPE.

Direct fitting uses constrained nonlinear least squares with Levenberg–Marquardt machinery, initial guesses and weighted complex residuals. Each frequency contributes a real and imaginary residual. The fitting residual is divided by the prepared weight; see the reported weighted versus unweighted statistics. Covariance is estimated from numerical Jacobian information; rank deficiency/large condition number can make parameter errors unreliable or unavailable.

For candidate ranking, unweighted `RSS = sum((Zre-Zre_fit)^2 + (Zim-Zim_fit)^2)`, with `n=2*N_frequency`. Gaussian criteria are `AIC=n*ln(RSS/n)+2*k`, `BIC=n*ln(RSS/n)+k*ln(n)`. A numerical floor handles exact-zero RSS. The historical modulus-normalized sum is explicitly named `legacy_penalized_score`; it is not a measured-variance chi-square. Do not interpret these different numbers as interchangeable.

ECM evolution uses a finite grammar, seeded candidate families, selection/crossover/mutation, bounded generations and parallel evaluation. Candidate fitness includes a PINN-named optimizer warm start followed by LM polishing; that name does not imply a neural network has discovered a unique physical mechanism. Default ranking is BIC even though old comments mention AIC. Increase population/generations for a broader search, then assess stability across runs and refit scientifically plausible candidates. There is no GPU CLI backend or exhaustive-topology guarantee.

## Potentiometric transient models

The segmenter normalizes time, selects an eligible event, handles duplicate/nonmonotonic policies, establishes a pre-event baseline, and selects a post-event fitting region subject to missingness, point-count and duration requirements. Adjacent events can limit useful interpretation of a response. Baseline mean, median or linear estimation and absolute/baseline-relative response conventions affect amplitude interpretation.

For local elapsed time `t`:

- Single: `E(t)=E_inf + A*exp(-t/tau)`.
- Double: `E(t)=E_inf + A_fast*exp(-t/tau_fast) + A_slow*exp(-t/tau_slow)`.
- Double-drift: the double form plus `drift*t`.
- Stretched: `E(t)=E_inf + A*exp(-(t/tau)^beta)`.

All voltages are V and tau is s. A is a signed displacement relative to the fitted asymptote under this equation; do not assume it means a positive rising-step magnitude. Double-mode constraints maintain ordered positive timescales. Multistart nonlinear optimization chooses a fit; AIC or BIC selects among successful candidates. The default configured model list includes all four models, but the quick-start fixture explicitly uses single.

Tau is the exponential relaxation scale, not automatically time to 90% response. For a pure single exponential, `t90=ln(10)*tau`; composite/stretched/drifting cases need their own feature interpretation. A fitted `E_inf` beyond an observation window is extrapolation. A tau near/above the window, negligible amplitude, weak mode separation, bounds or autocorrelated residuals should lower confidence even with good R².

Residual bootstrap generates parameter/feature uncertainty under residual-resampling assumptions. Iteration count, seed, confidence and minimum successful fraction are configurable. Setting zero iterations saves time but removes this uncertainty evidence. Bootstrap does not account for every metadata, calibration, environmental or model-selection uncertainty.

## Calibration and activity

Calibration extraction first needs concentration-step events and usable voltage estimates. Transient equilibrium estimates can be used when a corresponding eligible successful fit is available. Otherwise a configured steady-state source summarizes the late response window and rejects insufficient/unstable data. The shipped config enables a median fallback; built-in defaults used by a sparse override do not. A successful transient fit with a long extrapolation is not necessarily better calibration evidence than an observed stable window.

The Nernst relationship is `E=E0+S*log10(a)`, with theoretical signed slope proportional to `2.303*R*T/(z*F)`. Free, fixed-theoretical and prior-constrained slope modes have different scientific assumptions. Positive/negative response sign and temperature modes are explicit. Compare fitted slope to the theoretical response without treating a mismatch as a unique diagnosis.

Activity is distinguished from concentration. Ideal activity uses the declared concentration convention with coefficient 1; explicit activity or coefficient can override the calculation at an observation. Davies and extended Debye–Hückel use ionic strength and configured constants/validity limits. Ionic strength from composition is `I=0.5*sum(c_i*z_i^2)`. Extended Debye–Hückel additionally needs a positive ion-size parameter and declared size unit. Molality is not silently converted to molarity without density information. Mass-concentration conversion needs molar mass.

The Nicolsky–Eisenman form adds configured interfering-ion contributions inside the logarithmic response, conventionally `a_primary + sum(K_j*a_j^(z_primary/z_j))`. Selectivity fitting requires enough independent variation; a single covarying mixture cannot generally identify all coefficients. The conductivity-empirical activity option is an explicitly empirical correction, not a thermodynamic activity measurement.

Weighting by voltage standard error uses a floor to prevent a near-zero error dominating the fit. Model selection offers AIC, AICc, BIC or cross-validation; cross-validation modes include leave-one-out, leave-one-concentration-level-out, k-fold and leave-one-experiment-out. Choose a holdout unit matching the scientific claim. Randomly partitioning repeated rows is not a test of new-sensor or new-matrix transfer. Hysteresis compares ascending/descending branches near matched log activity; tolerances affect which levels are paired.

Prediction inverts the stored model and reports domain/extrapolation information. It cannot guarantee concentration recovery outside the declared activity/matrix/temperature/interferent regime. The scalar CLI provides voltage and temperature but no general interferent-activity map.

## Timescale and mechanism evidence

Topology determines which EIS quantities can be derived. Recognized parallel RC gives `tau=R*C`; parallel R/CPE gives `tau=(R*Q)^(1/alpha)` and `f_c=1/(2*pi*tau)`. Explicit relaxation elements expose their declared tau. Unsupported or unpaired elements do not automatically yield a unique resistance-capacitance product. Propagated uncertainty and frequency-band boundary checks limit what can be claimed.

Legacy comparison evaluates ratios and logarithmic distances between extracted EIS and selected transient timescales, plus compatibility evidence where uncertainty is available. Configured weak/moderate/strong labels are threshold summaries. The separate Phase-B route prepares typed evidence and explicit requirements/pairs, temporal joins, amplitude gates, replicate independence, identifiability and promotion rules. Unknown acquisition families and shared ancestry must not count as independent support. History is an explicit optional prior artifact, not a directory scan.

A mechanism trend manifest rereads its EIS/transient pairs. The current legacy trend calculation takes its independent coordinate from `sensor_age_days`; changing the configuration's independent-variable label does not select another metadata field. Rank statistics also need caution around ties. Use the CSV points for an independent scientifically specified trend analysis if your covariate is concentration, matrix or ionic composition.

## Signal characterization

Signal analysis chooses windows, checks sampling and applies a policy: require regular sampling, allow irregular time-domain-only analysis, or linearly resample subject to a gap limit. Resampling affects high-frequency content and cannot recover missing bandwidth. Detrending changes the low-frequency PSD; record it.

Welch PSD averages windowed overlapping segment periodograms; ASD is its square root. For a voltage channel the units are V²/Hz and V/sqrt(Hz). Segment length sets frequency resolution; zero padding increases displayed bins, not physical resolution. The Hann window reduces leakage; detrend can be none, mean or linear. Parseval checks compare spectral power with time-domain variance and may flag inconsistency. Missing PSD due to unsupported sampling is not evidence of zero noise.

Allan variance compares averages separated by an averaging interval, using overlapping estimates and a minimum cluster count. Allan deviation is the square root in the signal's units. Its minimum can suggest a useful averaging scale for the measured conditions; it is not a universal integration-time prescription.

Ordinary linear drift estimates least-squares slope; Theil–Sen uses robust pairwise slopes. Hampel detection compares a local value with a median/MAD-derived threshold, flags candidates and reports fraction/density; the original trace remains available. Correlation and lagged cross-correlation describe association between channels after alignment, not causation or a uniquely identified delay mechanism.

Time residuals use autocorrelation and related diagnostics. EIS residuals remain frequency-domain quantities. Calibration residual analysis currently constructs artificial index coordinates `0,1,...`: PSD there is per observation index, not hertz from an actual acquisition clock.

## Health evidence

Baselines aggregate domain features with context and lineage. Normalization can use relative change and robust z scores. A relative difference needs a nonzero baseline; robust normalization needs adequate spread/count. Rules combine `all_of`, `any_of`, domain/count gates and explicit alternatives. Phase-C assessment adds separate dimension statuses and context-aware evidence/causal promotion. Missing evidence must not be converted to “healthy.”

**IMPLEMENTATION ISSUE DISCOVERED DURING MANUAL AUDIT:** `src/health/trend.rs::calculate` collects `(feature_value, independent_value)` pairs but computes `ordinary_slope`/`theil_sen_slope` and `replicate_standard_deviation` from the independent values against record index. Consequently those fields are not feature-versus-independent-variable slopes/spread as their context suggests. Use the correctly retained per-record `value`, `independent_value` and baseline-change CSV columns for external trend fitting. Do not use these summary slopes for scientific claims until separately corrected. This task does not modify the algorithm.

## State estimation and ISM models

EKF linearizes a nonlinear observation/transition model with a numerical Jacobian; UKF propagates sigma points. Both combine prior covariance, process noise and measurement variance, then compare an innovation (observed minus predicted potential) against its predicted variance. Rejected outliers, large NIS and serial innovations can indicate a bad noise model or missing dynamics. A Joseph-form EKF covariance update and UKF jitter controls address numerical stability, not scientific identifiability.

Default legacy states represent log10 activity, baseline offset and polarization; simpler activity-only and activity-plus-baseline alternatives are available. Additional condition/sensitivity states need auxiliary evidence and observability. The default backend remains `legacy`. Explicit compiled profiles `legacy_equivalent_v1`, `reduced_ism_v1` and `custom` bind named model states/inputs to the filter; do not confuse `--model` state-model choice with `[model].definition` path.

Measurement noise can come from a configured variance, observation field, signal robust variance, stable-window variance or calibration uncertainty/residuals, subject to floors/caps and availability rules. Process-noise choices represent dynamics not in the deterministic model. Increasing them allows faster changes but can let noise masquerade as a state. Multiple latent voltage contributions can be confounded; observability and empirical sensitivity checks are necessary.

Model compilation checks declared units, dependencies, state/parameter bindings, ownership of voltage contributions and validity contracts. Built-in models include equilibrium adapters and phenomenological dynamics; first-order modes use exact relaxation stepping. Decomposition sums named additive potentials, keeps variance contributions distinct, and reports unexplained residuals when observed voltage is supplied. Optional artifacts supply only the adapters/evidence implemented by the runner; merely attaching an EIS fit does not optimize all model parameters.

Operational equilibrium recognition considers elapsed time constants, state rates/dynamic potential, equilibrium gap, innovations, residual structure, environment, uncertainty and observability. A small measured voltage slope alone is insufficient. Structural and practical identifiability can remain not assessed after successful compilation.

## Validation boundaries

`model validate --manifest` evaluates a model-study manifest; `estimate validate` compares state trajectories; `calibration validate` scores a stored calibration; `validation run` is the separate strict Phase-E artifact/cohort/protocol route. They are not synonyms.

Phase-E reads exact source hashes/identities, partitions and leakage information, reference labels/methods/blinding/uncertainty, endpoint acceptance rules and requested release scopes. Software-only claims do not require REAL authority. The shipped physical-approval trust store is unprovisioned, and production physical requests fail before dataset evaluation. Tests with test-only roots cannot establish production authority. Phase-F specifications and their approval gates remain a separate governance layer.
