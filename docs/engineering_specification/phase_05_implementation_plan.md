# Phase 05 Implementation Plan — Mechanism and Health Integration

## Scope

Connect the existing, reduced-order ISM contracts to mechanism and health analysis without
changing CLI commands, estimation runtime behaviour, or adding new physical diagnoses.

## File-level plan

| File | Change |
|---|---|
| `src/mechanism/model_mapping.rs` | Add explicit stable-component-ID mappings and neutral component-targeted hypothesis assessments. |
| `src/mechanism/mod.rs` | Export the mapping API. |
| `src/results/mechanism.rs` | Add serializable mapping, prior, and component-hypothesis result contracts. |
| `src/health/features.rs` | Keep transient events separate by scientific context and add model-derived feature adapters. |
| `src/health/rules.rs` | Preserve contradictory evidence and prevent single-domain mechanistic findings. |
| `src/health/assessment.rs` | Derive domain status from deviations, rules, and findings as well as feature warnings. |
| `src/results/health.rs` | Add a health warning for insufficient model evidence. |
| `tests/phase05_model_health.rs` | Add explicit mapping, evidence, residual, context, and multi-domain regression tests. |
| Engineering specifications and README | Record scope, safety boundary, traceability, risk, and migration behavior. |

## Verification

Run format, strict Clippy, complete tests, release build, diff check, and status check. Existing
workflows remain covered by the full suite.
