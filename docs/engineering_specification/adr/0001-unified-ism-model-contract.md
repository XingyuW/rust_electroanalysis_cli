# ADR-0001: Unified ISM model scientific contract

Status: Accepted
Date: 2026-08-05

## Context and purpose

The unified ISM framework composes existing equilibrium, activity, transient,
EIS, signal, and estimation capabilities into an inspectable reduced-order
model. It is a prediction and evidence-accounting framework, not proof that a
fitted mode has one physical cause.

## Dependency decision

Dependencies point from CLI, runners, plotting, reports, mechanism, health, and
estimation toward workflow adapters and the model core. The core may use domain
contracts, units, and narrow adapters over established scientific equations. It
must never import CLI, runners, plotting, report generation, health, mechanism,
or estimation. This direction is acyclic.

## Scientific decomposition

The state and observation equations are

```text
dx/dt = f(x, u, theta, t) + w
E_pred = h(x, u, theta, t) + v
```

Every deterministic prediction must expose

```text
E_pred = E_equilibrium
       + E_transport
       + E_transduction
       + E_reference
       + E_external
```

- Equilibrium response is the delegated activity/selectivity-dependent response.
- Transport dynamics are time-dependent states or delays, without an automatic
  mechanism identity.
- Transduction converts declared internal states or drives to voltage.
- Reference behavior is an explicit reference-electrode or baseline term.
- External disturbances are declared measured covariates or stochastic terms.
- Unexplained residual is `E_observed - E_pred`; it is never a component and no
  component may absorb or overwrite it.

## Evidence vocabulary

The framework keeps observed, fitted, derived, hypothesized, experimentally
supported, and validated-for-a-defined-domain as distinct statuses. A fitted
mode may receive a physical mechanism label only after an explicit hypothesis,
independent support from at least two experiments or evidence domains,
uncertainty and identifiability assessment, a stated applicability domain,
contradictory evidence, alternatives, and an acceptance criterion are recorded.
A single fitted time constant, EIS element, or parameter match is insufficient.

## Public contracts

`IsmModel`, `IsmComponent`, `ComponentDescriptor`, `ComponentRole`, `StateSpec`,
`ParameterSpec`, `ModelDefinition`, `CompiledIsmModel`, `ModelInput`,
`ModelState`, `ComponentContribution`, `ValidityReport`,
`IdentifiabilityReport`, `EvidenceRequirement`, and `EquilibriumAssessment`
form the stable public vocabulary. Every state and parameter is invalid without
a unit, finite bounds, source/provenance, and validity domain; parameters also
carry uncertainty. Contributions carry stable component identity, role, unit,
source, and validity. Missing diagnostics remain explicitly unavailable.

## Schema and extensibility policy

Model-definition, workflow-configuration, and result-artifact versions are
independent. Unknown future versions are rejected. Additive compatible changes
need serde defaults and migration tests. Changes to units, equations, state or
parameter meaning, or validity semantics require a version increment and an
explicit migration decision. New equations, states, parameters, or mechanism
hypotheses require provenance, units, bounds, uncertainty, validity, tests,
specification updates, and traceability; established scientific equations are
adapted rather than copied. Plugins use the static registry; runtime dynamic
libraries are outside the contract.

## Extraction and estimation compatibility

After the API stabilizes, `src/model` may be moved mechanically to
`crates/ism-model-core` with a compatibility re-export and no dependency back to
the application. The legacy `estimation::model::StateModel` remains available
until regression equivalence is demonstrated. Estimation may construct a
`ModelDefinition` and call `CompiledIsmModel`; the model core must not import
estimation. Existing estimation artifacts remain readable through explicit
schema migration and compatibility mode.

## Consequences

Residuals, limitations, uncertainty, contradictory evidence, and missing
evidence remain visible. The framework deliberately cannot turn a good fit or a
timescale match into physical validation.

Schema-v3 clarification: a derivative value of zero is valid only when its
stable state/parameter ID is explicitly covered. Missing coverage has a typed
status and prevents complete uncertainty for any non-deterministic influencing
quantity. Fitted parameters and estimated states require positive finite
uncertainty; legacy `uncertainty_incomplete` metadata cannot bypass that rule.
First-order covariance retains off-diagonal terms but does not quantify
structural/model-form uncertainty or imply Bayesian propagation.
