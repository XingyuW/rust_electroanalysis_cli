# Reduced-order ISM V1 final review

**Decision: GO — ready for estimation integration**

Reviewed at consumer SHA `64c6bf9202be5464f33934a8a240c39a4e391774`.
Provider SHA: `dbb6b7d063972114c4208980723e12c807ab199e` (`electrodata-io`,
as pinned in `Cargo.lock`).

## Scoped permanent coverage

| Case | Test file and exact function | Tracked / clean-checkout result | Typed error asserted | Exact structural path asserted |
|---|---|---|---|---|
| Applicability interval Infinity | `tests/model_v1_units_charge_serialization_contract.rs` — `applicability_interval_infinity_is_rejected_by_all_public_serializers` | Tracked; passes in a clean checkout | `ModelError::NonFiniteResult` from `to_json`; `ArtifactError::NonFiniteValue` from `write_artifact` | `$.model_definition.components[1].applicability_constraints[0].interval.lower` or `.upper` |
| Nonfinite parameter covariance | `tests/model_v1_units_charge_serialization_contract.rs` — `parameter_covariance_nonfinite_entries_are_rejected_with_exact_paths` | Tracked; passes in a clean checkout | `ArtifactError::NonFiniteValue`; direct API `ModelError::NonFiniteCovariance { subject: "parameter", row, column }` | `$.training_statistics.parameter_covariance[0][0]`, `[1][1]`, or `[0][1]` |
| Raw nonfinite equilibrium evidence | `tests/model_v1_units_charge_serialization_contract.rs` — `raw_equilibrium_evidence_nonfinite_values_are_rejected_with_exact_paths` | Tracked; passes in a clean checkout | `ArtifactError::NonFiniteValue` from both public serializers | `$.evidence.dynamic_state_derivative_norm`, `$.evidence.dynamic_potential_magnitude_v`, or `$.evidence.external_disturbance_potential_v.value` |

The applicability test exercises positive and negative Infinity at interval
endpoints and checks both `to_json` and `write_artifact`, including the
no-output-file condition. The covariance test mutates the public
`training_statistics.parameter_covariance` field and covers NaN and signed
Infinity. The equilibrium test injects nonfinite values into raw numeric
derivative, dynamic-potential, and external-disturbance evidence fields.

## Traceability

Exact test-file/function references are present in:

- `docs/engineering_specification/09_testing_and_quality_assurance.md`
- `docs/engineering_specification/13_traceability_matrix.md`
- `docs/model/reduced_order_ism_v1.md`

The exact model-core functions are also traceable:

- `tests/model_core.rs` — `model_artifact_rejects_nonfinite_definition_values`
- `tests/model_core.rs` — `covariance_matrix_validation_has_typed_failures`

## Validation commands

All required commands passed from a clean checkout:

```text
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
cargo build --locked --release
cargo test --locked applicability_interval_infinity_is_rejected_by_all_public_serializers
cargo test --locked --test model_v1_units_charge_serialization_contract parameter_covariance_nonfinite_entries_are_rejected_with_exact_paths
cargo test --locked --test model_v1_units_charge_serialization_contract raw_equilibrium_evidence_nonfinite_values_are_rejected_with_exact_paths
```

P0: none  
P1: none  
P2: none  
P3: none

