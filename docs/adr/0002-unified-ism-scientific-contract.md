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
