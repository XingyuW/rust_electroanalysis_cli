# 07 — Scientific Models & Equations

## Reduced-order ISM components V1

The V1 adapter equations, domains, and explicit non-mechanistic interpretation
limits are recorded in [`docs/model/reduced_order_ism_v1.md`](../model/reduced_order_ism_v1.md).

## Composition and uncertainty

The prediction is the sum of outputs explicitly typed
`additive_potential`. Observation-noise components provide `R` in V² and do
not enter voltage. With supplied runtime covariance, propagated variance is
`Jx Px Jxᵀ + Jθ Pθ Jθᵀ + R`, retaining off-diagonal terms. Missing derivative
coverage or covariance remains Partial or Unavailable, never zero by default.

**Identifier:** `DOC-07`  
**Status:** Verified from source code inspection  
**Last Updated:** 2026-07-19

All equations documented below are **verified against the source code**. If an apparent inconsistency is detected, it is recorded in `16_open_questions.md` and `14_risk_and_technical_debt_register.md`.

---

## Part A: EIS Circuit Elements

### EQ-EIS-001: Resistor (R)

**Expression**: Z = R  
**Parameters**: 1 — R [Ohm]  
**Constraint**: Positive  
**Bounds**: [1e-12, 1e12]  
**Source**: `src/impedance/elements.rs` L310

### EQ-EIS-002: Capacitor (C)

**Expression**: Z = 1/(jωC) = −j/(ωC) (ω > 1e-9); Z = 1e12 (DC limit)  
**Parameters**: 1 — C [F]  
**Constraint**: Positive  
**Bounds**: [1e-15, 1e3]  
**Source**: `src/impedance/elements.rs` L311-L317

### EQ-EIS-003: Inductor (L)

**Expression**: Z = jωL  
**Parameters**: 1 — L [H]  
**Constraint**: Positive  
**Bounds**: [1e-15, 1e6]  
**Source**: `src/impedance/elements.rs` L318

### EQ-EIS-004: Warburg (Infinite, W)

**Expression**: Z = σ(1−j)/√ω (ω > 1e-9); Z = 1e6 − j·1e6 (DC limit)  
**Parameters**: 1 — σ [Ohm·s^(-1/2)]  
**Constraint**: Positive  
**Bounds**: [1e-12, 1e12]  
**Source**: `src/impedance/elements.rs` L319-L327

### EQ-EIS-005: Constant Phase Element (CPE)

**Expression**: Z = 1/(Q·(jω)^α)  
**Implementation**: Uses polar form: magnitude = Q·ω^α, phase = πα/2  
**Parameters**: 2 — Q [Ohm^(-1)·s^α], α [dimensionless]  
**Constraints**: Q > 0, 0 < α < 1  
**Bounds**: Q ∈ [1e-15, 1e3], α ∈ [0.05, 1.0]  
**Source**: `src/impedance/elements.rs` L328-L340  
**Test**: `cpe_matches_ideal_capacitor_when_alpha_is_one`

### EQ-EIS-006: Finite-Length Warburg — Open (Wo)

**Expression**: Z = Z₀·coth(√(jωτ)) / √(jωτ)  
**Parameters**: 2 — Z₀ [Ohm], τ [s]  
**Constraints**: Both positive  
**Bounds**: Both [1e-12, 1e12]  
**DC limit**: 1e12 Ω (open circuit)  
**Source**: `src/impedance/elements.rs` L341-L356

### EQ-EIS-007: Finite-Length Warburg — Short (Ws)

**Expression**: Z = Z₀·tanh(√(jωτ)) / √(jωτ)  
**Parameters**: 2 — Z₀ [Ohm], τ [s]  
**DC limit**: Z₀ (resistive)  
**Source**: `src/impedance/elements.rs` L357-L371

### EQ-EIS-008: Modified Inductance (La)

**Expression**: Z = L·(jω)^α  
**Parameters**: 2 — L [H·s^(α-1)], α [dimensionless]  
**Bounds**: L ∈ [1e-15, 1e6], α ∈ [0.05, 2.0]  
**Source**: `src/impedance/elements.rs` L372-L378

### EQ-EIS-009: Generalized Warburg (Gw)

**Expression**: Z = σ·(jω)^(-α)  
**Parameters**: 2 — σ [Ohm·s^α], α [dimensionless]  
**Constraints**: σ > 0, 0 < α < 1  
**Bounds**: σ ∈ [1e-12, 1e12], α ∈ [0.05, 1.0]  
**Source**: `src/impedance/elements.rs` L379-L390

### EQ-EIS-010: Gerischer (G)

**Expression**: Z = R_G / √(1 + jωt_G)  
**Parameters**: 2 — R_G [Ohm], t_G [s]  
**Source**: `src/impedance/elements.rs` L391-L397

### EQ-EIS-011: Finite-Length Gerischer (Gs)

**Expression**: Z = R_G / (√(1 + jωt_G)·tanh(φ·√(1 + jωt_G)))  
**Parameters**: 3 — R_G [Ohm], t_G [s], φ [dimensionless]  
**Bounds**: R_G, t_G ∈ [1e-12, 1e12], φ ∈ [1e-6, 1e6]  
**Source**: `src/impedance/elements.rs` L398-L410

### EQ-EIS-012: K Element

**Expression**: Z = R / (1 + jωτ_k)  
**Parameters**: 2 — R [Ohm], τ_k [s]  
**Source**: `src/impedance/elements.rs` L411-L417

### EQ-EIS-013: Zarc

**Expression**: Z = R / (1 + (jωτ_k)^γ)  
**Parameters**: 3 — R [Ohm], τ_k [s], γ [dimensionless]  
**Constraint**: 0 < γ < 1  
**Bounds**: R, τ_k ∈ [1e-12, 1e12], γ ∈ [0.05, 1.0]  
**Source**: `src/impedance/elements.rs` L418-L426

### EQ-EIS-014: Transmission Line Model (TLMQ)

**Expression**: Z = √(R_ion·Z_s)·coth(√(R_ion/Z_s)), where Z_s = 1/(Qs·(jω)^γ)  
**Parameters**: 3 — R_ion [Ohm], Qs [Ohm^(-1)·s^γ], γ [dimensionless]  
**Source**: `src/impedance/elements.rs` L427-L451

### EQ-EIS-015: Porous Electrode Model (T)

**Expression**: Z = A·coth(β)/β + B/(β·sinh(β)), where β = √(a + jωb)  
**Parameters**: 4 — A [Ohm], B [Ohm], a [dimensionless], b [s]  
**Source**: `src/impedance/elements.rs` L452-L482

---

## Part B: Circuit Composition Rules

### EQ-CCT-001: Series

Z_total = Σ Z_i (implemented as complex sum)  
**Source**: `src/impedance/circuits.rs` L129-L132

### EQ-CCT-002: Parallel

Y_total = Σ Y_i = Σ (1/Z_i), Z_total = 1/Y_total  
Admittance near zero: uses 1e12 Ω fallback  
**Source**: `src/impedance/circuits.rs` L133-L152

---

## Part C: Fitting Methods

### EQ-FIT-001: Parameter Transformation

**Positive constraint** (R, C, L, W, etc.):
- Forward: internal = ln(physical) (physical ≤ 0 → internal = −23)
- Backward: physical = exp(internal)

**ZeroOne constraint** (α, γ exponents):
- Forward: internal = ln(physical/(1−physical))
- Backward: physical = 1/(1 + exp(−internal))

**Source**: `src/impedance/fitting.rs` L259-L288

### EQ-FIT-002: Initial Guess Heuristics

- **Rs (solution resistance)**: Minimum of Z′ at highest 5 frequencies
- **Rct (charge-transfer resistance)**: 2× arc real span at −Z″ peak, or max(arc_real, 0.15×span)
- **CPE α**: (4/π)·atan(2·|Z″_peak|/Rct), clamped to [0.45, 0.98]
- **CPE Q**: 1/(Rct·ω_char^α)
- **GW α**: atan(|Z″_tail|/Z′_tail) / (π/2), clamped [0.25, 0.75]
- **GW σ**: |Z_tail|·ω_tail^α

**Source**: `src/impedance/fitting.rs` L29-L145

### EQ-FIT-003: Weighting

Residual normalization denominator per point:

d_i = max(|Z_measured_i|, 1.0)

Real/imaginary residual channels are normalized by this denominator:

r_re,i = (Z′_model_i − Z′_measured_i) / d_i  
r_im,i = (Z″_model_i − Z″_measured_i) / d_i

**Source**: `src/impedance/lib.rs` L214-L273, `src/impedance/fitting.rs` L341-L390

### EQ-FIT-004: Optimization

Levenberg-Marquardt via `levenberg_marquardt` crate, minimizing:

J = Σ[(r_re,i)² + (r_im,i)²]  
= Σ[((Z′_model_i − Z′_measured_i)/d_i)² + ((Z″_model_i − Z″_measured_i)/d_i)²]

Reported weighted RMSE uses the same normalized channels:

weighted_RMSE = sqrt(J / (2N))

---

## Part D: Transient Models

### EQ-TR-001: Single Exponential

E(t) = E_∞ + A·exp(−t/τ)  
**Parameters**: E_∞ [V], A [V], τ [s]  
**Constraint**: τ > 0  
**Initial response rate**: −A/τ  
**Source**: `src/potentiometry/transient/models.rs` L129-L142

### EQ-TR-002: Double Exponential

E(t) = E_∞ + A_fast·exp(−t/τ_fast) + A_slow·exp(−t/τ_slow)  
**Parameters**: E_∞ [V], A_fast [V], A_slow [V], τ_fast [s], τ_slow [s]  
**Constraint**: 0 < τ_fast < τ_slow  
**Initial response rate**: −A_fast/τ_fast − A_slow/τ_slow  
**Source**: `src/potentiometry/transient/models.rs` L143-L164

### EQ-TR-003: Double with Drift

E(t) = E_∞ + A_fast·exp(−t/τ_fast) + A_slow·exp(−t/τ_slow) + drift·t  
**Parameters**: 6 — adds drift [V/s]  
**Source**: `src/potentiometry/transient/models.rs` L165-L188

### EQ-TR-004: Stretched Exponential

E(t) = E_∞ + A·exp(−(t/τ)^β)  
**Parameters**: E_∞ [V], A [V], τ [s], β [dimensionless]  
**Constraint**: τ > 0, β > 0  
**Configurable β range**: [0.05, 1.0]  
**Source**: `src/potentiometry/transient/models.rs` L189-L207

---

## Part E: Calibration Models

### EQ-CAL-001: Nernst Equation

E = E⁰ + S·log₁₀(a)  
where S = (RT ln 10)/(zF) [V/decade]

**Theoretical slope at 298.15 K, z = ±1**: ≈ ±0.05916 V/decade  
**Constants**: R = 8.31446261815324 J/(mol·K), F = 96485.33212 C/mol  
**Source**: `src/potentiometry/calibration/nernst.rs`

**Activity from potential**:
a = 10^((E − E⁰)/S)  
**Source**: `src/potentiometry/calibration/nernst.rs` L93-L115

### EQ-CAL-002: Nicolsky-Eisenman Equation

E = E⁰ + S·log₁₀(a_i + Σ K_ij·a_j^(z_i/z_j))  
(Implementation verified from `src/potentiometry/calibration/nicolsky_eisenman.rs`)

### EQ-CAL-003: Activity Models

| Model | Source |
|-------|--------|
| Ideal | γ = 1 |
| Davies | log₁₀(γ) = −A·z²·(√I/(1+√I) − 0.3·I) |
| Extended Debye-Hückel | log₁₀(γ) = −A·z²·√I/(1 + B·a₀·√I) |
| Conductivity Empirical | Empirical mapping |
| User-Provided | User-specified γ values |

**Source**: `src/potentiometry/calibration/activity.rs`

### EQ-CAL-004: Conductivity-Empirical Calibration Equation

E = E0 + S·[log₁₀(c) + b0 + b1·κ]

Where:
- c is concentration (mol/L)
- κ is conductivity (S/m)
- b0 and b1 are empirical conductivity correction coefficients

When `fit_b1 = false`, `b1` is fixed from configuration; when `fit_b1 = true`, it is estimated during regression.

**Source**: `src/potentiometry/calibration/fitting.rs` (model equation string and regression implementation)

---

## Part F: Signal Analysis Methods

### EQ-SIG-001: PSD (Welch Method)

Windowed FFT with configurable segment size and overlap via `rustfft`.  
**Source**: `src/signal/psd.rs`

### EQ-SIG-002: Allan Variance

σ²_y(τ) = (1/2)·⟨(ȳ_{k+1} − ȳ_k)²⟩  
**Source**: `src/signal/allan.rs`

### EQ-SIG-003: Linear Regression (OLS)

y = slope·x + intercept  
R² = 1 − SS_res/SS_tot  
Correlation: r = cov(x,y)/√(var(x)·var(y))  
**Source**: `src/regression_mod.rs`

---

## Part G: ECM Genetic Search

### EQ-ECM-001: Fitness Function

Candidates are ranked by one configured objective:
- BIC
- AIC
- weighted RMSE
- legacy penalized score

**Default ranking**: BIC (`EcmRankingCriterion::Bic`)  
**Source**: `src/impedance/ecm_scoring.rs`, `src/impedance/ecm_evolution.rs`

### EQ-ECM-002: Default Evolution Parameters

| Parameter | Default | Configurable |
|-----------|---------|-------------|
| Population size | 24 | Yes |
| Generation limit | 12 | Yes |
| Individuals per parents | 2 | Yes |
| Selection ratio | 0.7 | Yes |
| Mutation rate | 0.35 | Yes |
| Reinsertion ratio | 0.75 | Yes |
| Ranking criterion | bic | Yes |

### EQ-ECM-003: Mutation Operators

1. Leaf kind mutation (R↔C↔CPE↔W, etc.)
2. Series element insertion
3. Parallel wrapper insertion
4. Subtree swap crossover
5. Node insertion
6. Leaf pruning

**Source**: `src/impedance/ecm_evolution.rs`

---

## Part H: Kalman Filter Methods

### EQ-EST-001: EKF Prediction

x̂_k|k-1 = f(x̂_{k-1|k-1})  
P_k|k-1 = F_k·P_{k-1|k-1}·F_k^T + Q_k

### EQ-EST-002: EKF Update

K_k = P_k|k-1·H_k^T·(H_k·P_k|k-1·H_k^T + R_k)^(-1)  
x̂_k|k = x̂_k|k-1 + K_k·(z_k − h(x̂_k|k-1))  
P_k|k = (I − K_k·H_k)·P_k|k-1

**Source**: `src/estimation/ekf.rs`

### EQ-EST-003: UKF

Unscented transform with sigma points.  
**Source**: `src/estimation/ukf.rs`

---

## Part I: Unified ISM Model Contract

### EQ-ISM-001: State-Space Interface

`dx/dt = f(x, u, θ, t) + w`

`src/model/` defines the interface and validates state/parameter metadata. It
does not implement a new state equation in Phase 02.

### EQ-ISM-002: Explicit Observation Decomposition

`E_pred = E_equilibrium + E_transport + E_transduction + E_reference + E_external`

`E_unexplained_residual = E_observed - E_pred`

`ComponentContribution` records named contributions and
`UnexplainedResidual` records either the residual or missing observed-voltage
evidence. No component may own a residual and no fitted result receives a
mechanism label from this contract alone.

### EQ-ISM-003: Reduced-Order Built-In Adapters

Phase 03 delegates Nernst, Nicolsky-Eisenman, activity, and relaxation
evaluation to the existing calibration and transient modules. First/two-mode
and stretched terms are phenomenological; candidate names are not mechanism
assignments. High-fidelity Nernst-Planck transport is not implemented.

### EQ-ISM-004: Evidence and Health Integration

Legacy fitted parameters are not new equations. They may form a prior only
through an explicit component-ID/source-path mapping. A numerical timescale
match without independent replication is weak evidence and no health or
mechanism adapter changes a phenomenological mode into a physical diagnosis.
The unexplained-residual RMS remains a separate quality feature.

### EQ-ISM-005: Workflow Evaluation

Phase 06 evaluates compiled component equations already supplied by the core.
Deterministic simulation is synthetic rather than calibration or physical
validation; equilibrium remains indeterminate without dynamic and innovation evidence.

### EQ-ISM-006: Validation Metrics

Parameter/state recovery, contribution reconstruction, coverage, calibration
transfer, cross-sensor comparisons, and model criteria are computed only from
manifest-declared observations. Synthetic-only scenarios are reported as such
and cannot substantiate physical validation.

### EQ-ISM-007: Discrete Relaxation and Legacy Estimation Adapter

For a constant target over a time interval, the first-order reduced state uses
the exact recurrence

`x(t + dt) = exp(-dt/tau) x(t) + (1 - exp(-dt/tau)) target`.

This makes a fixed elapsed-time prediction invariant to numerical subdivision;
it is not a physical-mechanism assignment. The compiled legacy estimation
adapter preserves the pre-existing equilibrium, polarization, baseline, and
sensitivity equations and exposes them as stable contribution IDs. Their sum
is `E_pred`; `E_observed - E_pred` remains a separate unexplained residual and
is absent rather than zero when no measurement exists.

### EQ-ISM-008: Operational Equilibrium Recognition

A timestamp may be classified as operationally supported equilibrium only when
all configured evidence gates are available and pass: normalized state-rate
stability, dynamic-potential magnitude, full equilibrium gap, elapsed time in
units of the fitted dynamic time constant, standardized innovation, residual
autocorrelation, environmental stability, calibration-domain status,
normalized state uncertainty, local observability, empirical identifiability,
and minimum history length. A failed gate produces contradictory evidence; a
missing gate produces an indeterminate assessment. The result is not physical
mechanism validation.

High-fidelity Nernst–Planck transport remains outside the reduced-order model
contract. Adding it requires a separate scientific phase with new state,
boundary-condition, numerical-stability, unit, identifiability, and experimental
validation requirements; it is not a release blocker for this framework.

### EQ-ISM-009: Contract-only extension policy

ADR-0002 adds no equation. It standardizes the state/parameter/component
metadata and requires any future component to state its equation/version,
composition, validity, interpretation status, and evidence requirements. Nernst,
Nicolsky-Eisenman, activity, transient, EIS-timescale, and estimation-observation
implementations remain existing adapters until a separately reviewed equation
change is approved.

For scalar voltage prediction, `E_pred = sum(E_additive)`. Observation noise
is `R` in V² and cannot be a voltage term. Available uncertainty is propagated
as `J_x P_x J_x^T + J_theta P_theta J_theta^T + R`; caller-supplied full
runtime covariance retains correlation. Declared uncertainty is prior or
initialization metadata and is not an implicit runtime diagonal covariance.
Structural and model-form uncertainty remain unquantified.

Runtime covariance supplies the numerical joint distribution only: it may refine a
declared stochastic magnitude and cross-correlation, but may not change a
schema-declared stochastic quantity into deterministic or make a deterministic
quantity stochastic. Consequently a declared stochastic diagonal must be
positive when a full matrix is supplied, while deterministic rows/columns must
be exactly zero. Symmetry and PSD tolerances apply only to numerical matrix
validation, never to this semantic-zero rule: `1e-13` is nonzero for a
deterministic row and remains positive for a stochastic diagonal. A singular
valid covariance remains permitted. Missing Jacobian
coverage is different from an explicitly covered zero derivative and cannot be
converted to zero by inspecting covariance.

### EQ-ISM-010: Analytical Observation-Parameter Jacobians

For an explicitly parameterized Nernst form
`E = E0 + S log10(a)`, the built-in returns `dE/dE0 = 1` and
`dE/dS = log10(a)`. The default Nernst descriptor instead uses the theoretical
temperature/charge slope, so it does not pretend that slope is a fitted
uncertain parameter. Ion charge is a rounded structural parameter; its
continuous derivative is unavailable, and an uncertain charge therefore makes
uncertainty incomplete.

The built-in ion charge remains `Fixed` plus `Deterministic`: it is a discrete
valence, not a continuously fitted physical coefficient. A custom definition
that marks charge stochastic has no continuous first-order derivative in this
adapter and therefore cannot receive `Complete` differential uncertainty; it
must use an explicitly supported non-differential treatment outside this
adapter rather than silently rounding and propagating a fake derivative.

For Nicolsky-Eisenman,
`Aeff = ai + sum(Kij aj^(zi/zj))` and `E = E0 + S log10(Aeff)` when an empirical
slope is declared. The derivatives are `dE/dE0 = 1`,
`dE/dS = log10(Aeff)`, and
`dE/dKij = S aj^(zi/zj) / (ln(10) Aeff)`. With the theoretical slope, the same
selectivity derivative is evaluated as
`R T aj^(zi/zj) / (zi F Aeff)`. Interferent parameters are resolved by stable
parameter ID, never by descriptor-local vector position.

For `E = offset + gain x`, the derivatives are `1` and `x`. For the implemented
centered covariate form `E = beta (u - uref)`, they are `u - uref` and `-beta`.
For linear drift `E = k t`, the derivative is `t`. State-output components
explicitly cover each directly observed state with derivative one, including a
mathematically zero value when that is the actual analytical result.

Prediction variance is
`Var(E) = Jx Px Jx^T + Jtheta Ptheta Jtheta^T + R`. Supplied full runtime
matrices retain off-diagonal cross terms. Absent runtime covariance leaves a
stochastic component unavailable rather than creating a diagonal matrix from
declared uncertainty. Structural and model-form uncertainty are still not
quantified, and no Bayesian propagation claim is made.
