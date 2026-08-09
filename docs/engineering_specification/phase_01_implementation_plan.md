# Phase 01 — Unified ISM Scientific Contract Plan

## Scope

This phase establishes only the scientific and architectural contract for the
future unified ion-selective membrane (ISM) model. It adds no Rust module,
CLI command, runtime dependency, numerical equation, serialization schema, or
workflow behavior.

| File | Change |
|---|---|
| `docs/engineering_specification/adr/0001-unified-ism-model-contract.md` | Decision record defining boundaries, state/observation equations, evidence policy, contract shapes, schema policy, and future extraction path. |
| `01_system_requirements.md` | Add Phase-01 contract requirements and explicit non-runtime status. |
| `02_architecture.md` | Reserve an acyclic future model-core boundary and compatibility adapter direction. |
| `03_module_specifications.md` | Specify the future `src/model/` ownership and proposed public contracts without creating it. |
| `07_scientific_models_and_equations.md` | Record the compositional observation equation and prevent interpretation as an implemented numerical equation. |
| `13_traceability_matrix.md` | Trace the planned contract to its ADR and planned validation. |
| `14_risk_and_technical_debt_register.md` | Record residual opacity, mechanism-label, and contract-drift risks. |

## Completion conditions

- Existing scientific modules remain authoritative adapters in future phases.
- `src/model` is absent after this phase.
- No CLI, runner, result, configuration, or artifact schema changes occur.
- Every future state and parameter contract declares units, bounds, source, and
  validity domain; unexplained residual remains explicit.
