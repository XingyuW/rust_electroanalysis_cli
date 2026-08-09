# 09 — Testing & Quality Assurance

## Reduced-order ISM components V1

Parity, analytical decay, decomposition, variance separation, and neutral
equilibrium-recognition coverage are added in `reduced_ism_components_v1` tests.

## Model-core QA

Model-core tests cover graph rejection and stable ordering, state/parameter
round trips and slices, additive reconstruction, variance separation,
Jacobian coverage, covariance propagation, schema serialization, and an
architecture regression check that prohibits high-level imports from
`src/model`.

**Identifier:** `DOC-09`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## 1. Test Inventory

### Unit Tests (In-File)

| Source File | Test Count (approx.) | Coverage Focus |
|-------------|---------------------|----------------|
| `src/cli.rs` | 5 | CLI parsing, legacy normalization, error cases |
| `src/domain/experiment.rs` | 1 | Event ordering |
| `src/domain/measurement.rs` | 3 | Validation, diagnostics, irregular sampling |
| `src/domain/provenance.rs` | 1 | SHA-256 hashing |
| `src/impedance/lib.rs` | 10 | EIS fitting, preprocessing, element limits |
| `src/impedance/circuits.rs` | (inferred) | Parser, evaluation |
| `src/potentiometry/units.rs` | 2 | Unit conversion, molar mass requirements |
| `src/potentiometry/calibration/nernst.rs` | 2 | Nernst slope, charge sign |
| `src/regression_mod.rs` | 6 | Linear fit, error cases, curve generation |
| `src/plot_config.rs` | 20 | Schema, migration, resolution, validation |
| `src/results/mod.rs` | 1 | CircuitFitResult structure |
| `src/model/` | (covered by integration fixture) | Core compilation, validation, and decomposition contracts |

### Integration Tests (`tests/`)

| Test File | Phase | Coverage |
|-----------|-------|----------|
| `tests/phase0_regression.rs` | 0 | End-to-end binary: plot, search, fit CLI |
| `tests/phase1_domain.rs` | 1 | Measurement parsing, metadata, experiment construction |
| `tests/phase2_transient.rs` | 2 | Transient model fitting, model selection, CLI integration |
| `tests/phase3_calibration.rs` | 3 | Calibration model fitting, prediction, validation |
| `tests/phase3_workflow.rs` | 3 | Full calibration workflow |
| `tests/phase4_mechanism.rs` | 4 | Mechanism comparison |
| `tests/phase5_signal_health.rs` | 5 | Signal analysis, health assessment |
| `tests/phase6_estimation.rs` | 6 | State estimation, EKF/UKF |
| `tests/unified_data_loading.rs` | 3 | File format detection, binary rejection |
| `tests/xlsx_ingestion.rs` | (inferred) | Excel file reading |
| `tests/canonical_input_boundary.rs` | IO migration | canonical ownership guard, compatibility API compilation, recovery/time/EIS semantics |
| `tests/canonical_error_types.rs` | IO migration | provider typed errors and worksheet context |
| `tests/model_core.rs` | 02 | ISM graph compilation, schemas, decomposition, and legacy CLI surface |
| `tests/model_builtins.rs` | 03 | Synthetic equilibrium, relaxation, disturbance, reconstruction, and validity coverage |

### Test Classification

| Classification | Count |
|---------------|-------|
| Unit tests (inline `#[cfg(test)]`) | ~50+ |
| Integration tests (`tests/`) | ~10 files |
| CLI tests | 5 (cli.rs) + binary tests |
| Numerical tests | Multiple (fit accuracy, Nernst slope) |
| Error-path tests | Multiple (invalid inputs, edge cases) |
| Cross-platform tests | CI runs on Linux + macOS |

The completed independent legacy parity gate is preserved in
`docs/io_migration_validation_archive.md`; ongoing coverage uses the
`electrodata-io` canonical input boundary, typed errors, XLSX, EIS semantics,
and all-phase scientific regression tests rather than a retained local raw
parser.

## 2. Requirement Test Coverage

Evidence classes:
- **Direct**: explicit assertions in tests for requirement behavior.
- **Partial**: representative workflow behavior covered, but not every requirement facet asserted directly.
- **Inferred**: requirement confidence is primarily from implementation inspection.

| Requirement | Evidence Class | Evidence Summary |
|------------|----------------|------------------|
| FR-001 to FR-011 | Direct | Parsing, fit, model resolution, and ECM search behaviors are asserted in `chi_file`, `unified_data_loading`, `phase0_regression`, and module tests. |
| FR-012 to FR-014 | Partial | Plot workflows are integration-tested for dispatch and artifact creation; visual/semantic plot correctness is not directly asserted. |
| FR-015 | Direct | `phase2_transient` validates event filtering, model fitting, and CLI behavior. |
| FR-016 to FR-019 | Direct | `phase3_calibration` and workflow tests assert extraction, fitting, validation, and prediction paths. |
| FR-020 | Direct | `phase5_signal_health` asserts core signal feature extraction and residual/statistical pathways. |
| FR-021 to FR-022 | Partial | Health baseline/assessment paths are exercised, but complete rule-surface and cross-artifact combinations remain partially inferred. |
| FR-023 | Inferred | Mechanism implementation is verified from source with limited direct test assertions for all comparison/trend branches. |
| FR-024 | Direct | `phase6_estimation` asserts EKF/UKF runtime behavior and artifact outputs. |
| FR-025 to FR-026 | Direct | Provenance hashing and EIS text reporting are directly asserted in unit/integration tests. |
| FR-027 to FR-028 | Partial | Representative JSON/CSV exports are tested; full workflow-by-workflow artifact-name coverage is implementation-verified. |
| FR-029 to FR-033 | Direct | `model_core` covers model compilation, graph/unit failures, bounds, deterministic indices, decomposition, versioned schemas, and CLI compatibility. |
| SCI-001 to SCI-010 | Partial | Core scientific constants/formulas are directly tested; several equations remain code-verified rather than assertion-complete. |
| NUM-001 to NUM-006 | Partial | Numerical safeguards and diagnostics are tested in representative paths; full edge-space coverage is incomplete. |
| DAT-001 to DAT-007 | Partial | Data-model invariants are directly tested; cross-module schema compatibility and migration behavior remain partial. |

## 3. Test-Gap Summary

| Gap | Description | Priority |
|-----|-------------|----------|
| GAP-001 | No performance regression tests | Low |
| GAP-002 | Cross-version artifact compatibility is covered for the common contract; payload-specific migrations remain to be added as schemas evolve | Low |
| GAP-003 | Many workflow tests validate artifact presence/shape but not full file-content semantics for every exported artifact variant | Medium |
| GAP-004 | No fuzz testing of circuit parser | Low |
| GAP-005 | No tests for NaN/Inf edge cases in all numerical paths | Medium |
| GAP-006 | Kalman filter process/measurement noise model tests are limited | Medium |
| GAP-007 | No Windows CI testing | Low |
| GAP-008 | No integration test for `estimate simulate` | Closed by the permanent compiled-truth tests below |
| GAP-009 | No deterministic reproducibility tests (fixed seed comparison) | Medium |
| GAP-010 | Plotting output is tested for file existence, not visual correctness | Low |
| GAP-011 | No real Nernst, transport, transduction, reference, or external ISM component implementation yet | High (planned Phase 03) |
| GAP-012 | Identifiability and equilibrium contracts intentionally report not-assessed rather than numerical conclusions | Medium (planned later validation phase) |
| GAP-013 | Built-ins are reduced-order adapters; no high-fidelity Nernst-Planck transport or mechanism confirmation is implemented | High (deferred) |

## 4. Run Commands

| Action | Command |
|--------|---------|
| Format check | `cargo fmt --check` |
| Lint (strict) | `cargo clippy --all-targets --all-features -- -D warnings` |
| All tests | `cargo test --all` |
| Specific test | `cargo test <test_name>` |
| Release build | `cargo build --release` |
| CI simulation | `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all && cargo build --release` |

## 5. CI Configuration (`.github/workflows/ci.yml`)

- **Triggers**: push, pull_request
- **Matrix**: ubuntu-latest, macos-latest
- **Toolchain**: pinned by `rust-toolchain.toml`, including rustfmt and clippy
- **Locked steps**: cargo clippy, test, and release build use `--locked`; format is checked before compilation

## Phase 05 Regression Coverage

`tests/phase05_model_health.rs` covers explicit EIS component-ID mapping,
missing mappings, unreplicated timescale evidence, contradictory evidence,
model residual health features, and multi-domain mechanistic health guards.

`phase06_model_workflow` covers model CLI parsing, invalid configuration,
deterministic simulation, decomposition exports, JSON compatibility, finite
JSON output, report regeneration, and estimate-command parsing compatibility.

`phase07_validation` verifies manifest evaluation, contribution reconstruction,
reproducible result contracts, and the synthetic-data non-validation boundary.

`artifact_contract` verifies typed kind/version rejection, legacy kind-less
migration, write-time headers, and semantic contracts for every exported
cross-workflow result. `model_builtins` verifies malformed-descriptor rejection,
runtime ion charge, exact relaxation subdivision invariance, and contribution
reconstruction. `phase6_estimation` verifies that the compiled compatibility
adapter reproduces legacy equations and that EKF/UKF expose identical named
scientific contributions.

Phase A0 artifact-contract evidence is permanent and fixture-based:

- `tests/a0_producer_roundtrip.rs::mhi_t02a_current_correct_kind` reads all
  eight tracked schema-2 fixtures in
  `tests/fixtures/a0_artifact_contracts/schema2/` through `read_artifact` and
  asserts representative payload fields.
- `tests/a0_producer_roundtrip.rs::mhi_t02d_legacy` reads all eight tracked
  schema-1 fixtures in `tests/fixtures/a0_artifact_contracts/schema1/` through
  `read_artifact` and asserts representative payload fields.
- `tests/a0_producer_roundtrip.rs::mhi_t02f_producer_roundtrip` covers the nine
  producer paths without generating schema-1 compatibility evidence.
- `tests/artifact_contract.rs::a0_ac_compat_01_preserves_eis_fit_and_health_baseline_matrices`
  reads tracked missing/correct/wrong-kind fixtures for `eis_fit` and
  `health_baseline`; it performs no runtime writes below `tests/fixtures/`.

Fixture source and historical schema evidence are recorded in
`tests/fixtures/a0_artifact_contracts/README.md` and the detailed A0 mapping is
in `docs/engineering_specification/a0_artifact_contract_traceability.md`.

`estimation::model_output::equilibrium_tests` covers a fully evidenced stable
timestamp, rejection of slow reference drift, and indeterminate classification
when history or residual evidence is missing.

### ADR-0002 contract coverage

`tests/model_contracts.rs` guards stable component IDs, duplicate state and
parameter rejection, required state/parameter/component metadata, the separation
of noise/residuals from deterministic voltage, and forbidden high-level imports
from `src/model`. Existing `model_core` tests cover acyclic dependencies and
unique contribution ownership. These tests are architectural invariants, not
evidence that a high-fidelity physical transport model has been validated.

Model-contract tests cover deterministic contribution ordering, typed voltage
reconstruction, observation-variance exclusion, canonical external-role alias
migration, uncertainty validation, covariance propagation, and non-finite
prediction serialization rejection.

## Compiled estimation integration matrix

`tests/phase6_estimation.rs` is the permanent integration gate for compiled
estimation. The runtime binding plan is resolved once per compiled model and is
then executed by the shared `StateModel::compiled_input` path used by EKF, UKF,
compiled simulation, estimate, and compare. Explicit custom target IDs take
precedence over standard defaults; required unbound inputs fail; optional
unbound inputs remain absent; source declarations, units, conversions, and
model identity are retained in `StateEstimationReport.resolved_input_bindings`.

| Contract | Exact test function(s) | Profile/filter/state model | Tolerance or policy |
|----------|-------------------------|----------------------------|---------------------|
| `flow_drive` target execution, provenance, hash, report, and normal runtime observation | `custom_flow_drive_binding_executes_in_normal_estimation_runtime` | `Compiled` / `Custom` / EKF / Activity | Exact target-ID and source assertions; fixture predictions are checked |
| Standard/custom sources, named environment, event field, constants, and typed failures | `compiled_bindings_cover_standard_custom_target_and_constant_sources`, `compiled_bindings_support_named_environment_and_event_field_sources`, `compiled_bindings_reject_typed_target_source_and_unit_failures`, `compiled_bindings_report_missing_and_optional_sources_without_position_assumptions` | `Compiled` / `Custom` / shared input plan | Typed error variants and fields; typed unit conversion |
| Legacy/compiled parity matrix | `compiled_legacy_equivalent_permanent_parity_matrix_covers_states_filters_and_scenarios`, `compiled_legacy_parity_covers_condition_sensitivity_state_for_ekf_and_ukf` | `Legacy` vs `Compiled` + `LegacyEquivalentV1`; EKF and UKF; Activity, ActivityBaseline, ActivityBaselinePolarization, plus sensitivity/condition | Ordinary matrix `1e-10`; condition/sensitivity compatibility `1e-8`; warning metadata differences are explicitly allowlisted only |
| Nicolsky-Eisenman legacy-equivalent parity | `compiled_legacy_equivalent_nicolsky_interferent_parity_runs_ekf_and_ukf` | Legacy vs Compiled + `LegacyEquivalentV1`; EKF and UKF; target Ca2+, interferent Cl-, signed charges, active selectivity, temperature series | Normal estimation path; timestamps, named states, covariances, predicted potential, innovation/NIS, update and domain statuses; `1e-10` numeric tolerance |
| Irregular, multiple, initial, and absent activity events | `compiled_activity_events_are_applied_once_at_irregular_transitions_with_provenance` | Reduced transition binding | Events in `(previous_timestamp, current_timestamp]`; interval sums and all event timestamps are asserted |
| Transduction drive modes and active truth | `compiled_transduction_drive_modes_cover_none_activity_step_event_field_and_failures`, `compiled_reduced_active_transduction_truth_and_validation_use_stable_state_ids` | ReducedIsmV1; EKF plus simulation truth/validation | None/ActivityStep/ExplicitEventField; missing and incompatible event fields remain unavailable or warn |
| Stable-ID simulation truth and validation metrics | `compiled_reduced_active_transduction_truth_and_validation_use_stable_state_ids` | ReducedIsmV1 with active transduction | IDs, deterministic ordering, RMSE, MAE, bias, and interval coverage are asserted by name |
| Human reports and migration breadth | `estimation_report_matrix_renders_honest_backend_profile_and_custom_definition`, `old_estimation_configuration_fixture_matrix_preserves_legacy_defaults`, `old_estimation_artifact_migration_matrix_keeps_identity_honest_and_deterministic`, `legacy_state_estimation_report_fixture_migrates`, `legacy_simulation_truth_fixture_migrates`, `legacy_validation_fixture_migrates`, `legacy_filter_comparison_fixture_migrates` | Legacy, compiled legacy-equivalent, reduced, custom; tracked old config/truth/report/validation/comparison schemas | Backend/profile/model identity never defaults to Compiled; old fields deserialize as absent/unavailable; fixture reserialization is deterministic |
| Absent compiled truth | `compiled_validation_keeps_absent_truth_metrics_unavailable` | ReducedIsmV1 / EKF / stable `reference_offset_v` ID with truth intentionally omitted | Available truth still scores; omitted state has zero matched samples and `None` RMSE/MAE/bias/coverage |
| Typed duplicate binding error | `duplicate_binding_key_returns_typed_error_with_target_and_declaration` | Compiled binding resolver with malformed duplicate target fixture | Pattern-matches `EstimationError::DuplicateModelInputBinding` and preserves target, declaration, and model ID |

This matrix does not compare `ReducedIsmV1` numerically with Legacy. It
compares only the approved LegacyEquivalentV1 adapter with the historical
backend and keeps reduced-profile truth and contribution assertions separate.

Schema-v3 regressions additionally cover Nernst E0 and slope covariance and
their cross term, Nicolsky-Eisenman E0/slope/selectivity derivatives and full
covariance, transduction gain/offset, centered covariate coefficients, drift,
stable-ID mapping, missing derivative/covariance status, explicit disabling,
strict fitted/estimated compatibility matrices, legacy numeric-zero migration,
and enrichment of legacy estimator states from resolved initial covariance.

Covariance-consistency regressions cover the reviewer Nernst positive-charge
uncertainty plus zero-row reproduction, a generic missing derivative with a
zero row, missing fitted-parameter and estimated-state runtime covariance,
deterministic and stochastic `1e-13` state/parameter covariance, covered
numerical zero derivatives, state equivalents, and dimension, non-finite,
asymmetric, and non-PSD matrices. The existing Nernst 0.76 V² cross-covariance
calculation remains covered.

`tests/model_components_v1_contract.rs` permanently retains V1 guardrails for
exact charge rejection, calibrated-domain Warn/Reject behavior, candidate
transduction status, missing disturbance evidence, structured
identifiability, and nested nonfinite serialization.

The permanent V1 matrix also covers legacy-only and typed-only domains, the
legacy-plus-typed two-constraint reviewer reproduction, exact duplicate
provenance, typed conflicts, deterministic constraint ordering, mixed
Warn/Reject independence, and shared `to_json`/`write_artifact` rejection of a
nested nonfinite applicability interval. The wider model-core suites retain the
charge, equation, dynamic, covariance, identifiability, unit, equilibrium, and
decomposition matrices.

### Permanent V1 regression-matrix traceability

The following normal integration-test targets are committed regression gates;
the function names are intentionally recorded so a future review can identify
the scientific assertion without relying on a filename alone.

| Requirement | Test file | Exact test function | Scientific assertion protected |
|-------------|-----------|---------------------|--------------------------------|
| ISM-013 declaration migration, deterministic ordering, duplicate provenance, and typed conflicts | `tests/model_v1_applicability_contract.rs` | `legacy_only_constraint_is_migrated_losslessly`, `typed_only_constraint_is_resolved_and_preserved`, `mixed_legacy_and_typed_constraints_survive_with_stable_ordering`, `exact_duplicate_retains_both_provenance_sources_and_conflicts_name_fields`, `conflicting_intervals_and_policies_return_typed_conflict_context` | Legacy and typed domains are lossless, deterministic, source-provenanced, and never silently composed. |
| ISM-014 per-constraint Warn/Reject enforcement and unavailable evidence | `tests/model_v1_applicability_contract.rs` | `independent_warn_and_reject_policies_keep_each_constraint_outcome`, `unavailable_warn_and_reject_are_independent_and_ordered` | Each applicability constraint retains its own typed result, warning, and enforcement policy. |
| ISM-012 equilibrium evidence statuses | `tests/model_v1_equilibrium_identifiability_contract.rs` | `equilibrium_recognition_classifies_every_v1_status_with_auditable_criteria`, `equilibrium_recognition_preserves_missing_not_applicable_and_unobservable_evidence` | Equilibrium, quasi-equilibrium, transitional, disturbed, and indeterminate assessments preserve satisfied, violated, and missing evidence. |
| ISM-015 topology and optional-component identifiability metadata | `tests/model_v1_equilibrium_identifiability_contract.rs` | `identifiability_metadata_distinguishes_one_and_two_active_dynamic_modes`, `optional_requirements_promote_to_active_and_serialized_order_is_stable` | Active versus conditional requirements, IDs, scopes, targets, and serialized ordering are stable. |
| ISM-010 discrete ion-charge contract | `tests/model_v1_units_charge_serialization_contract.rs` | `nernst_component_rejects_every_invalid_discrete_charge_without_coercion`, `nicolsky_component_validates_target_and_interferent_charges_on_its_actual_path` | Nernst and Nicolsky-Eisenman only accept fixed, finite, nonzero integral ion charges. |
| ISM-011 covariate dimensions | `tests/model_v1_units_charge_serialization_contract.rs` | `covariate_units_accept_exact_contracts_and_report_precise_mismatches` | Temperature, conductivity, flow, `%RH`, and ppm inputs require matching reference and V-per-input sensitivity units with typed mismatch context. |
| ISM-015 finite public artifacts | `tests/model_v1_units_charge_serialization_contract.rs` | `public_serialization_paths_reject_nested_nonfinite_values_without_creating_files`, `analysis_serialization_rejects_each_uncertainty_and_nested_numeric_path` | `to_json` and `write_artifact` reject nonfinite nested fields at the same structural path and do not create an output file. |
| ISM-015 model-definition parameter finiteness | `tests/model_core.rs` | `model_artifact_rejects_nonfinite_definition_values` | `ModelCompilationArtifact::to_json` rejects a nonfinite `model_definition.parameters[0].default_value` before JSON can substitute `null`. |
| ISM-009 state-covariance matrix validation | `tests/model_core.rs` | `covariance_matrix_validation_has_typed_failures` | The direct prediction API rejects nonfinite, asymmetric, and non-PSD **state** covariance matrices with typed errors; it does not claim parameter-covariance coverage. |
| ISM-015 applicability-interval infinity | `tests/model_v1_units_charge_serialization_contract.rs` | `applicability_interval_infinity_is_rejected_by_all_public_serializers` | Positive and negative infinity at declared interval lower/upper endpoints yield the same exact structural path from `ModelCompilationArtifact::to_json` and `write_artifact`, with no output file. |
| ISM-009 / ISM-015 parameter-covariance finiteness | `tests/model_v1_units_charge_serialization_contract.rs` | `parameter_covariance_nonfinite_entries_are_rejected_with_exact_paths` | NaN and signed infinity in public `training_statistics.parameter_covariance` retain matrix coordinates through serialization, while the direct model API identifies the `parameter` covariance subject and coordinates. |
| ISM-012 / ISM-015 raw equilibrium-evidence finiteness | `tests/model_v1_units_charge_serialization_contract.rs` | `raw_equilibrium_evidence_nonfinite_values_are_rejected_with_exact_paths` | Raw derivative, dynamic-potential, and external-disturbance evidence values are rejected at their exact paths before JSON serialization or artifact writes. |
