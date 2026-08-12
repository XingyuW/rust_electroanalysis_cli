# Phase A1 fixtures

These tracked fixtures cover the durable migration boundary and covariance
representation:

- `legacy_lineage_state.json` is explicit `LegacyUnknown` lineage.
- `current_known_lineage_state.json` is a current known-lineage artifact.
- `aggregate_scope.json` preserves aggregate membership without a synthetic
  `ExperimentId`.
- `legacy_unlabeled_covariance.json` remains readable but has no A1 axis
  semantics.
- `current_labeled_covariance.json` uses producer-owned, parameter-specific
  EIS axes; CPE `q` and `alpha` are distinct.
