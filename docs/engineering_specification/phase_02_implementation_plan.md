# Phase 02 — Extensible ISM Model Core Implementation Plan

## Scope

Implement the dependency-clean, equation-free model-core contracts required for
future unified ISM work. Existing analysis workflows, CLI commands, numerical
models, and result formats remain unchanged.

## File-Level Plan

| File(s) | Change | Purpose |
|---|---|---|
| `src/model/mod.rs` | Add public model-core façade | Re-export the stable core contracts without importing application layers. |
| `src/model/error.rs` | Add `ModelError` | Return typed validation, compilation, and evaluation errors. |
| `src/model/definition.rs`, `component.rs`, `parameter.rs`, `state.rs`, `input.rs` | Add serializable definitions and traits | Describe versioned models, components, units, states, parameters, and inputs. |
| `src/model/registry.rs`, `graph.rs`, `compiler.rs` | Add static factory registry and deterministic compiler | Resolve indices, validate graphs/units, and reject invalid model definitions. |
| `src/model/output.rs`, `validity.rs`, `identifiability.rs`, `evidence.rs`, `equilibrium_recognition.rs` | Add explicit output, validity, evidence, and assessment contracts | Preserve uncertainty, validity limits, missing evidence, and unexplained residuals. |
| `src/model_config.rs` | Add versioned model configuration schema | Keep configuration outside the core and validate its schema version. |
| `src/results/model.rs`, `src/results/mod.rs` | Add durable model compilation artifact schema | Serialize only validated finite model definitions and reports. |
| `src/lib.rs` | Expose `model` and `model_config` | Make the new core available without adding a CLI command. |
| `tests/model_core.rs` | Add mock-component contract tests | Cover graph failures, bounds, deterministic indices/compilation, decomposition, invalid models, and existing CLI parsing. |
| Engineering specifications | Update requirements, architecture, modules, equations, validation, QA, traceability, risks, and migration guidance | Trace the public contract and its deferred scientific components. |

## Acceptance Checks

- The model module has no dependency on `cli`, `runners`, `plottings`, `health`,
  `mechanism`, or `estimation`.
- The compiler rejects duplicate IDs, missing/circular dependencies, missing
  inputs, unit mismatches, non-finite values, invalid bounds, and duplicate
  voltage contribution ownership.
- Mock components prove deterministic compilation and explicit voltage
  decomposition; no Nernst, transient, or EIS equation is reimplemented.
- Existing commands and workflow tests continue to pass unchanged.
