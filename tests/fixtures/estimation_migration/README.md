# Historical estimation artifact fixtures

These files are hand-maintained compatibility fixtures, not current artifacts
rewritten with an older version number.

| File | Artifact kind | Historical schema | Omitted/new fields | Expected migration |
|---|---|---:|---|---|
| `legacy_state_estimation_report_v1.json` | `StateEstimationReport` | 1 | Artifact kind header and compiled-model fields | Deserializes through `read_artifact`; backend/profile/model identity remain unavailable (`None`). |
| `legacy_simulation_truth_v2.json` | compiled/legacy simulation output | 2 | `scenario.model` and per-point `compiled` truth | Deserializes as the legacy backend; compiled truth remains `None`. |
| `legacy_state_validation_v1.json` | `StateValidationResult` | pre-contribution-metrics | `contribution_metrics` | New contribution metrics default to an empty list; absent state metrics remain absent. |
| `legacy_state_filter_comparison_v2.json` | `StateFilterComparison` | 2 | record backend/profile and ingestion diagnostics | Backend/profile remain `None`; ingestion diagnostics default to empty. |
