# ADR-0001: Establish the Unified ISM Model Scientific Contract

**Status**: Accepted  
**Date**: 2026-08-05  
**Author**: XingyuW/rust_electroanalysis_cli maintainers

---

## Context

The repository currently contains independently useful scientific capabilities:
Nernst and Nicolsky-Eisenman equilibrium response, activity and unit handling,
transient fitting, EIS fitting, signal analysis, and EKF/UKF state estimation.
They must remain authoritative for their present workflows. A future ISM model
must combine evidence from these capabilities without duplicating equations,
silently assigning physical meaning to a fitted parameter, or making the CLI
and rendering layers dependencies of the scientific core.

This ADR is a contract only. No `src/model` module, public Rust item, equation
implementation, component, artifact, configuration field, or command is added
by Phase 01.

## Decision

### Purpose

The unified ISM model framework will provide an explicit, inspectable
composition boundary for predicting a potentiometric observation, recording
component contributions, checking validity and identifiability, and preserving
unexplained residual. It is not a claim that every fitted mode has a unique
physical mechanism.

### Dependency direction

The future dependency graph is strictly acyclic:

```text
CLI / runners / reports / plotting / health
                    ↓
    workflow adapters and artifact boundaries
                    ↓
     current scientific adapters + ISM model core
                    ↓
            domain contracts and units
```

`src/model` (future) may depend on `domain`, stable unit types, and narrow
adapters over existing scientific implementations. It must not depend on CLI,
runners, plotting, health, report generation, or result serialization. The
existing estimation module will consume `CompiledIsmModel` through an adapter;
the model core will not import `estimation`.

### Scientific decomposition

The predicted potential is the sum of named, separately reported contributions:

```text
E_pred = E_equilibrium
       + E_transport
       + E_transduction
       + E_reference
       + E_external
```

| Contribution | Meaning | Explicit non-meaning |
|---|---|---|
| Equilibrium response | Activity- and selectivity-dependent equilibrium response, normally delegated to existing Nernst/Nicolsky-Eisenman and activity adapters | A proof of transport kinetics or sensor health |
| Transport dynamics | Time-dependent state evolution or delayed response | A mechanism label inferred from one fitted time constant |
| Transduction | Membrane/electrode conversion between state and voltage | An implicit reference offset |
| Reference behavior | Reference-electrode/baseline contribution and its uncertainty | A catch-all for arbitrary model error |
| External disturbances | Explicitly observed or declared environmental/experimental influences | Unmeasured error assumed to be disturbance |
| Unexplained residual | `E_observed - E_pred`, reported with units, uncertainty, and diagnostics | A contribution that may be silently absorbed by another component |

The future dynamic contract is:

```text
dx/dt = f(x, u, θ, t) + w
E_pred = h(x, u, θ, t) + v
```

where `x` is `ModelState`, `u` is `ModelInput`, `θ` is declared model
parameters, `w` is process uncertainty, and `v` is observation uncertainty.
The observation equation must expose the five named contributions above and a
separate unexplained residual. It is a framework equation, not a new numerical
equation or an alternative implementation of any existing scientific model.

### Evidence and terminology

The framework shall preserve these distinct statuses:

| Status | Meaning |
|---|---|
| Observed | Directly measured value with provenance and unit. |
| Fitted | Value optimized against a declared objective; not inherently causal. |
| Derived | Value calculated from observed/fitted values with a documented equation. |
| Hypothesized | User- or literature-supplied candidate explanation. |
| Experimentally supported | Supported by stated independent observations and `EvidenceRequirement`; alternatives remain visible. |
| Validated for a defined domain | Tested against stated data, conditions, limits, and acceptance criteria; not universal validation. |

A fitted mode may receive a physical mechanism label only when an
`EvidenceRequirement` records, at minimum: an explicit hypothesis; independent
supporting evidence from at least two distinct domains or experiments; a stated
validity domain; uncertainty/identifiability assessment; contradictory evidence
and alternatives; and an acceptance criterion. A time constant, an EIS element,
or parameter value alone is insufficient.

### Proposed public contracts (declaration-only)

These are the required public contract shapes for a later `src/model` module.
They are names and invariants only in Phase 01; no Rust declarations are added
yet.

| Contract | Required responsibility and invariants |
|---|---|
| `IsmModel` | Compiles a `ModelDefinition`, evaluates declared contributions, and returns `ValidityReport`/`IdentifiabilityReport`; cannot hide residual. |
| `IsmComponent` | Declares a `ComponentDescriptor`, consumes only declared inputs/states/parameters, and contributes one `ComponentRole`. |
| `ComponentDescriptor` | Stable identifier, role, version, inputs/states/parameters, equation provenance, validity domain, and evidence requirements. |
| `ComponentRole` | `Equilibrium`, `Transport`, `Transduction`, `Reference`, or `External`; no residual role. |
| `StateSpec` | Identifier, unit, physical bounds, initial/source provenance, transform, uncertainty, and validity domain. |
| `ParameterSpec` | Identifier, unit, bounds, source/provenance, fixed/fitted status, prior/uncertainty, and validity domain. |
| `ModelDefinition` | Versioned declarative component graph plus `StateSpec`/`ParameterSpec`; validates acyclicity and unique identifiers. |
| `CompiledIsmModel` | Validated immutable executable graph produced from `ModelDefinition`; exposes no CLI/runtime policy. |
| `ModelInput` | Timestamped observed inputs, environmental context, declared units, provenance, and missing-data representation. |
| `ModelState` | Named state vector aligned to `StateSpec`, with units, finite values, uncertainty, and timestamps. |
| `ComponentContribution` | Role, component identity, predicted voltage contribution, unit, uncertainty, validity status, and source evidence. |
| `ValidityReport` | Declared domain checks, extrapolations, missing inputs, violated constraints, and warnings; absence of warnings is not proof of truth. |
| `IdentifiabilityReport` | Rank/conditioning/correlation or other declared diagnostics and unavailable diagnostics; never converts non-identifiability into a mechanism label. |
| `EvidenceRequirement` | Minimum independent support, accepted/contradictory evidence, alternatives, and domain-specific acceptance criteria for an interpretive label. |
| `EquilibriumAssessment` | Documents whether equilibrium is assumed, observed, fitted, or unsupported; names the delegated equilibrium adapter and its domain. |

Every `StateSpec` and `ParameterSpec` is invalid without all of: unit, finite
bounds (or a documented physically unbounded policy), source/provenance, and a
validity domain. Components must produce an explicit unavailable/invalid status
when their required inputs are absent or outside domain. No component can write
to the unexplained residual; the framework computes and reports it after
summation.

### Model schema-versioning policy

`ModelDefinition` will own `model_schema_version`, separate from configuration
and result-artifact schema versions. Readers will reject unknown future major
versions. Additive compatible fields require defaults and migration tests;
semantic, unit, equation, state, parameter, or validity-domain changes require
a new version and an explicit migration/compatibility decision. Every compiled
model and derived artifact will record the model definition identifier, schema
version, component versions, and equation provenance.

### Extensibility policy

New equations, states, parameters, or mechanism hypotheses must be added as
new declared contracts/components with unit, bounds, source, validity domain,
uncertainty, tests, specification update, and traceability entry. Existing
Nernst, Nicolsky-Eisenman, activity, unit, transient, EIS, signal, and
estimation calculations are reused through stable adapters rather than copied.
Mechanism hypotheses are extensible evidence metadata, never automatically
discovered labels.

### Future extraction path

Phase 02 may introduce `src/model/` as a dependency-clean module. Once its API
is stable, move it mechanically to `crates/ism-model-core` with the same
dependency restrictions and a thin compatibility re-export from the root crate.
The extraction must preserve serialized definition compatibility and add no
dependency from `ism-model-core` back to application modules.

### Compatibility with current estimation

Current EKF/UKF APIs, state models, configuration, artifacts, and commands
remain unchanged. A future `estimation::ism_adapter` may map a
`CompiledIsmModel` to existing process/measurement callbacks while retaining
legacy `StateModel` behavior as the default. No automatic migration of existing
estimation configurations or interpretation of legacy states is authorized by
this ADR.

## Alternatives considered

| Alternative | Why rejected |
|---|---|
| Build components immediately | Would mix scientific design decisions with numerical behavior and violate Phase 01 scope. |
| Put the core in runners or estimation | Reverses dependency direction and prevents reuse outside the CLI. |
| Treat every fitted mode as a mechanism | Overstates evidence and conflicts with existing mechanism safeguards. |
| Add a generic residual component | Allows unexplained error to be hidden rather than audited. |

## Consequences

### Positive

- Establishes a reviewable scientific vocabulary before numerical implementation.
- Preserves existing equations and current workflows as stable adapter targets.
- Makes uncertainty, domain validity, identifiability, contradictory evidence,
  and residual visibility mandatory design concerns.

### Negative

- Later phases must satisfy a deliberately detailed contract before shipping
  components.
- The framework does not itself improve current numerical estimates in this
  phase.

### Neutral

- No CLI behavior, result schema, configuration schema, or runtime dependency
  changes in Phase 01.
