# Phase 07 Implementation Plan — Scientific Validation Infrastructure

| File | Change |
|---|---|
| `src/results/validation.rs` | Versioned validation manifest, dataset taxonomy, metrics, comparison, and result contracts. |
| `src/model_validation.rs` | Compute finite recovery, coverage, reconstruction, and comparison metrics from declared experiments. |
| `src/runners/model_validation.rs` | Export reproducible JSON, CSV, identifiability, comparison, and text reports. |
| `src/cli.rs`, `src/main.rs` | Let `model validate --manifest` execute an experimental validation study. |
| `tests/phase07_validation.rs` | Confirm synthetic evidence remains explicitly non-physical validation. |
