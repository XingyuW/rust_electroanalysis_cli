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

### Phase 07 validation artifacts

`ValidationResults` is schema version 1 with artifact kind
`ism_model_validation`. A reader must reject an incompatible schema or kind.

Phase 02 introduces `ModelDefinition` and `ModelConfig` schema version `1`,
and the `ism_model_compilation` result artifact schema version `2`. Model schema
v1 remains readable: legacy additive composition is migrated to typed semantics,
legacy `external` deserializes as `external_disturbance`, and legacy numeric
uncertainty remains explicitly unknown/incomplete rather than zero. There are no
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

Model configuration schema 1 is consumed by the Phase 06 model commands. Empty
or partial checked-in model definitions are rejected; they are not silently
replaced with a default model. `model_definition_resolved.json` contains the
resolved definition itself, while compilation diagnostics use the distinct
`ism_model_compilation` artifact contract.

Phase 03 adds built-in component kinds under the existing v1 definition
schema. Their `equation_version`, assumptions, and evidence requirements are
serialized in each descriptor; semantic changes require an explicit migration.

All cross-workflow result readers now use `VersionedArtifact`. A historical
artifact may omit `artifact_kind` only when its schema is listed by that
specific result contract. On successful read it is migrated in memory by
stamping the expected kind; writers always emit both kind and current schema.
Future schemas and mismatched kinds are rejected with `ArtifactError`, never
coerced. Additive Phase 04 estimation fields default only for old schema-1/2
reports, preserving existing `StateEstimationReport` deserialization.

The additive `[equilibrium_recognition]` estimation configuration section uses
documented defaults under schema version 3, so existing schema-3 files require
no migration. Timestamp-level result fields were already optional; no result
schema increment is required.

## Version 3: uncertainty and derivative coverage

Model-definition schema 3 adds direct observation state/parameter declarations,
typed Jacobian coverage/method records, and strict uncertainty compatibility.
Model compilation and analysis artifacts increment from schema 2 to 3 because
prediction uncertainty now serializes Jacobian methods and follows stricter
status semantics. Older model definitions still deserialize. Legacy numeric
uncertainty becomes `Unknown` with a migration reason; it is never reinterpreted
as zero.

The configuration adapter can migrate unambiguous composition and built-in
dependency declarations in memory, but `uncertainty_incomplete` no longer
bypasses validation. A legacy fitted parameter or estimated state with missing,
zero, deterministic, or unknown uncertainty returns typed
`InvalidUncertainty` until the user supplies a positive finite prior/covariance
or reclassifies a truly fixed quantity. Direct compilation of an unmigrated
legacy definition returns `LegacyMigrationRequired` once its uncertainty is
otherwise valid. Writers emit schema 3; model artifact readers retain explicit
legacy schema 1/2 support.
