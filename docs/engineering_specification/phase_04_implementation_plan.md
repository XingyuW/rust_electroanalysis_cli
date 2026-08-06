# Phase 04 — Estimation / Compiled ISM Integration Plan

1. Add `estimation::ism_adapter` to translate the legacy `StateModel` and its
   calibration context into a `ModelDefinition`/`CompiledIsmModel` without
   removing the legacy implementation.
2. Extend estimation configuration with an optional model definition and an
   explicit legacy-compatibility mode; migrate schema only if a durable config
   field is required.
3. Route EKF and UKF transition/observation/Jacobian calls through the same
   adapter, while retaining existing report deserialization defaults.
4. Add contribution, residual, validity, and evidence-aware equilibrium fields
   as optional durable report fields, with schema migration tests.
5. Feed compiled-model observability/identifiability to existing diagnostics;
   add regression tests for legacy equivalence, predict-only rows, relaxation,
   non-equilibrium drift, unobservability, contribution sums, and old artifacts.
