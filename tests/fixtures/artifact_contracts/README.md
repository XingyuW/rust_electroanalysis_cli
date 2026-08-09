# A0 artifact-contract fixtures

These are tracked compatibility inputs for `A0-AC-COMPAT-01`.

- `eis_fit_schema2_missing_kind.json`: current `eis_fit` artifact, schema 2,
  missing `artifact_kind`; this was accepted before A0 and must remain accepted.
- `health_baseline_schema2_missing_kind.json`: current `health_baseline`
  artifact, schema 2, missing `artifact_kind`; this was accepted before A0 and
  must remain accepted.

The tests add the correct and wrong kind to these fixture values to verify the
complete preserved matrix without changing the tracked historical inputs.
