# ISM Model Schema Migration Policy

## Version 1

Phase 02 introduces `ModelDefinition` and `ModelConfig` schema version `1`,
and the `ism_model_compilation` result artifact schema version `1`. There are no
earlier ISM model artifacts to migrate.

## Compatibility Rules

- Readers reject an unsupported `schema_version` with typed `ModelError`; they
  do not silently reinterpret model states, parameters, equations, or units.
- Additive optional fields may be accepted only when documented defaults retain
  their original scientific meaning.
- A change to equation semantics, units, bounds, state/parameter ordering, or
  contribution ownership requires a new schema version and explicit migration
  tests.
- A compiled model preserves definition-order state and parameter indices. A
  migration that changes either order is breaking and must provide an explicit
  old-to-new mapping.
- Model artifacts use `ModelCompilationArtifact::to_json`, which validates
  finite numeric definition values before JSON serialization.

## Current Migration Behavior

Existing project configuration and result artifacts are untouched. The new
model schema is not consumed by any CLI command or runner in Phase 02.
