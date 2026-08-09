# Preserved A0 compatibility fixtures

These tracked inputs preserve the pre-A0 compatibility contract for the two
non-repair artifacts whose current `artifact_kind` remains optional. The
missing-kind inputs are historical compatibility cases; the correct-kind and
wrong-kind inputs are fixed matrix cases derived from the same payload.

| Fixture | Kind/version | Kind state | Source | Public-reader result |
|---|---|---|---|---|
| `eis_fit_schema2_missing_kind.json` | `eis_fit` / 2 | missing | Existing tracked A0 fixture | accepted |
| `eis_fit_schema2_correct_kind.json` | `eis_fit` / 2 | correct | Existing tracked fixture plus fixed contract header | accepted |
| `eis_fit_schema2_wrong_kind.json` | `eis_fit` / 2 | `signal_analysis` | Existing tracked fixture plus fixed wrong header | `IncompatibleKind` |
| `health_baseline_schema2_missing_kind.json` | `health_baseline` / 2 | missing | Existing tracked A0 fixture | accepted |
| `health_baseline_schema2_correct_kind.json` | `health_baseline` / 2 | correct | Existing tracked fixture plus fixed contract header | accepted |
| `health_baseline_schema2_wrong_kind.json` | `health_baseline` / 2 | `signal_analysis` | Existing tracked fixture plus fixed wrong header | `IncompatibleKind` |

The matrix is exercised by
`a0_ac_compat_01_preserves_eis_fit_and_health_baseline_matrices` in
`tests/artifact_contract.rs`. No test writes to this directory.
