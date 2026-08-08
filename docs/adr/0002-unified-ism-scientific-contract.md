# ADR-0002: ISM scientific-contract framework

Status: Accepted  
Date: 2026-08-07

## Decision

The ISM model framework is a component-based scientific contract for a future
unified ion-selective-membrane model. It represents equilibrium recognition,
transport, transduction, reference/interface terms, external disturbances,
observation noise, and unexplained residuals without claiming that a fitted
term identifies a physical mechanism. It does not introduce a high-fidelity
transport solver in this decision.

The state-space contract is `dx/dt = f(x, u, theta, t) + w` and the
observation contract is `E_pred = h(x, u, theta, t) + v`. Deterministic
potential reconstruction is explicitly additive:

```text
E_pred = E_equilibrium + E_transport + E_transduction + E_reference + E_external
E_unexplained = E_measured - E_pred
```

`E_unexplained` remains a separate residual record. It cannot be a voltage
component or be automatically assigned to any mechanism.

## Component architecture and direction

Components are independently describable, composable, and registry-resolved so
new scientific hypotheses do not require independent match statements in
estimation, health, mechanism, plotting, or reports. Each component declares a
stable `ComponentId`, role, interpretation status, state/parameter/input
bindings, equation/version, validity domain, evidence requirements, and an
explicit voltage composition rule when it contributes to the observation.

The permitted dependency direction is:

```text
domain / units / small numerical abstractions / equation adapters
                         ↓
                      model core
                         ↓
                  workflow adapters
                         ↓
             estimation / mechanism / health
                         ↓
                runners / plotting / reports
```

The core must not import CLI, runners, plotting, report generation, health,
mechanism, or estimation. Static registry factories are used; runtime dynamic
library plugins are explicitly outside this architecture.

## Scientific metadata

`ModelInput` is the measured or externally supplied `u`; `ModelState` is the
ordered latent `x`; `ParameterSpec` defines `theta`; and `ModelPrediction` is
the evaluated observation. State specifications include stable ID, name,
description, unit, transformation, initialization source, bounds, process
equation version, observability requirements, validity domain, and uncertainty
representation. Parameter specifications include stable ID, name, description,
unit, bounds, default/prior value, source, equation version, uncertainty,
identifiability requirements, whether a value is fixed/fitted/externally
supplied, and validity domain.

Roles are `Equilibrium`, `Transport`, `Transduction`, `Reference`,
`ExternalDisturbance`, `ObservationNoise`, `Auxiliary`, and `Unexplained`.
Only named deterministic terms with an `additive_voltage` composition rule can
participate in `E_pred`; observation noise and unexplained residuals cannot.
Every component declares `Phenomenological`, `Hypothesized`,
`ExperimentallySupported`, or `ValidatedForDomain`. A fitted exponential begins
as `Phenomenological`.

## Evidence, uncertainty, and equilibrium recognition

Model-form uncertainty, unidentifiability, missing evidence, alternatives, and
contradictory evidence are first-class outputs. A mechanism assignment requires
an explicit hypothesis, independent supporting evidence, uncertainty and
identifiability evidence, domain validity, alternatives, and an acceptance
criterion. A good fit, a fitted time constant, or an EIS match is insufficient.

Equilibrium recognition reports `Equilibrium`, `QuasiEquilibrium`,
`Transitional`, `Disturbed`, or `Indeterminate`. Its future algorithm must
consider state derivatives, dynamic voltage, measured-equilibrium gap, elapsed
time relative to time constants, innovation statistics, residual
autocorrelation, environmental stability, calibration-domain validity,
uncertainty, and observability. This ADR defines the evidence interface only.

## Versioning, compatibility, and extension

Model definitions are schema-versioned independently from workflow config and
result artifacts. Compatible additive fields use explicit serde defaults;
changes to units, equation meaning, state/parameter semantics, or validity
semantics require a schema-version decision and migration tests. Unknown future
versions are rejected.

A new component requires implementation, descriptor, state/parameter specs,
static-registry entry, unit tests, validity tests, synthetic behavior test,
equation documentation, and traceability update. Existing equations are reused
only through adapters for Nernst, Nicolsky-Eisenman, activity, transient/EIS
timescales, and state-estimation observation models; equations are not copied.

Legacy estimation remains an outer compatibility adapter until regression
equivalence is shown. The future extraction path is a mechanical move of
`src/model` to `crates/ism-model-core`, retained by a compatibility re-export;
the extracted crate must not reverse the dependency direction.

### Schema-v2 composition and uncertainty remediation

Composition is a closed typed contract: `additive_potential`,
`observation_variance`, `state_only`, or `auxiliary`. Only additive potential
terms reconstruct prediction; observation variance remains V². The sole
runtime external role is `external_disturbance`; legacy `external` is an input
alias only. Predictions report complete, partial, unavailable, or
not-requested uncertainty. First-order propagation uses available `J P Jᵀ`
terms and records diagonal-independence assumptions. It does not claim full
Bayesian or model-form uncertainty propagation.

### Schema-v3 derivative-coverage and uncertainty remediation

Schema v3 separates a covered derivative whose analytical value is zero from
an omitted derivative. Each additive component declares its direct observation
state and parameter IDs. Local Jacobians return stable IDs, values, coverage
status (`complete`, `partial`, `unavailable`, or `not_applicable`), and method;
the compiler validates the local-to-global mapping. Built-ins use analytical
derivatives. Numerical derivatives require an explicit component declaration
and recorded positive relative and absolute steps; there is no silent finite-
difference fallback.

`Complete` now requires covariance and derivative coverage for every
non-deterministic influencing state and parameter, an available observation
variance, finite values, and no unresolved source. Missing covariance or
coverage produces `Partial` when some meaningful uncertainty is available and
`Unavailable` otherwise. `NotRequested` is emitted only by an explicit request
flag. Full covariance uses all off-diagonal terms; per-item uncertainty creates
a documented diagonal covariance and independence assumption.

Fitted parameters and estimated states require positive finite uncertainty.
`Deterministic`, zero, and `Unknown` are invalid for those sources.
`uncertainty_incomplete` is retained only as legacy/migration metadata and
cannot bypass validation. Legacy numeric zero deserializes as `Unknown` and
must be enriched before compilation. This remains first-order propagation and
makes no Bayesian or structural/model-form uncertainty claim.

### Covariance uncertainty consistency

Schema declarations are authoritative: `Deterministic`, `StochasticKnown`, or
`StochasticUnknown` is derived from the value/initialization source and typed
uncertainty declaration, never from a covariance row. A full covariance matrix
quantifies magnitude and correlation but cannot turn a declared stochastic
quantity into deterministic or vice versa. Known stochastic entries require a
finite positive diagonal; deterministic entries require an all-zero row and
column. Dimension, finiteness, symmetry, and PSD are validated before
propagation, and contradictions are typed errors. A zero derivative is valid
only when the Jacobian explicitly covers the ID and returns numeric zero;
missing coverage remains missing even for a zero covariance row. `Complete`
requires valid covariance and explicit derivative coverage for each relevant
stochastic source, observation variance, and finite propagated values.
