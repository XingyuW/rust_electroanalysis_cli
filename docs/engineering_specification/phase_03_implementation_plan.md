# Phase 03 — Built-In Reduced-Order ISM Components Plan

## Scope

Add static built-in component factories to the Phase 02 model core. Built-ins
reuse existing calibration/activity/unit/transient implementations through thin
adapters. No CLI, runner, estimation, health, mechanism, or high-fidelity
Nernst-Planck implementation is added.

## File-Level Plan

| File(s) | Change | Purpose |
|---|---|---|
| `src/model/component.rs`, `compiler.rs`, `registry.rs`, `error.rs` | Extend component metadata, bindings, warnings, and static built-in registry | Require equation/version/assumptions/evidence and support validated component-level Jacobians. |
| `src/model/builtins.rs` | Add equilibrium, activity, transport, transduction, disturbance, and noise factories | Adapt existing equations and expose neutral reduced-order component kinds. |
| `src/model/defaults.rs`, `src/model/mod.rs` | Add and export default reduced-order model definition | Provide equilibrium + fast + slow + baseline drift + noise without a CLI workflow. |
| `src/model/input.rs` | Add structural covariate-unit support | Validate temperature, conductivity, and flow covariate inputs and coefficients. |
| `tests/model_builtins.rs` | Add required synthetic tests | Verify equilibrium, relaxation, disturbances, reconstruction, and validity warnings. |
| Engineering specifications, traceability, risks, QA, migration guide, README | Document public built-ins and deferred limits | Preserve scientific boundaries and migration behavior. |

## Acceptance Checks

- Every built-in descriptor declares equation, version, parameter IDs/units and
  bounds through `ParameterSpec`, assumptions, validity domain, and evidence
  requirements.
- The built-in registry is static; no runtime plugin mechanism is introduced.
- Component Jacobians are implemented for dynamic/observation components.
- Default model remains reduced-order and calls no new high-fidelity transport
  implementation.
