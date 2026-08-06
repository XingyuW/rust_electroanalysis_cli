# ISM Model Schema Migration Policy

## Version 1

### Phase 05 adapter contracts

Phase 05 adds serializable mapping and component-hypothesis contracts without
changing an existing emitted artifact schema or CLI output. Existing health and
mechanism artifacts remain readable. Missing explicit mappings produce no prior
or mechanism assignment rather than a guessed conversion.

### Phase 06 analysis artifacts

`ModelAnalysisReport` is schema version 1 with artifact kind
`ism_model_analysis`. Consumers reject a different version or kind rather than
silently treating a non-model JSON artifact as a model analysis.

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

Phase 03 adds built-in component kinds under the existing v1 definition
schema. Their `equation_version`, assumptions, and evidence requirements are
serialized in each descriptor; semantic changes require an explicit migration.
