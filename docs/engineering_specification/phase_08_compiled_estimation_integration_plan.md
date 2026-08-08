# Compiled ISM model estimation integration

## Audited boundary

- `src/estimation/model.rs` owns the historical direct state and observation
  equations: log10 activity, optional baseline, optional polarization, and
  optional sensitivity state.  Its historical state order is activity,
  baseline, polarization, sensitivity.
- `src/estimation/ekf.rs` and `src/estimation/ukf.rs` share that model boundary
  for process/observation evaluation, but retain their respective numerical
  update implementations.
- `src/estimation/ism_adapter.rs` is an estimation-owned calibration bridge to
  the model core.  `src/model/` has no estimation dependency.
- `src/results/estimation.rs` is an artifact schema with serde defaults for
  additive fields; pre-integration artifacts therefore remain readable.

## Integration order

1. Preserve the direct legacy backend as the configuration default.
2. Route explicit compiled compatibility and reduced-V1 profiles through the
   same `StateModel` calls used by EKF and UKF.
3. Bind `log10_activity` to positive `target_activity` only at the compiled
   input boundary; it is never a second estimator state.
4. Resolve model observation variance only under an explicit policy and retain
   the estimator process-covariance ownership.
5. Record selected backend/profile and compiled graph summary in reports;
   plotting consumes those artifacts without calculating models.
6. Keep legacy-parity fixtures and reduced-V1 scenarios separate because their
   state vectors are intentionally different.

## Resolved runtime input bindings

Compiled configuration is resolved into a deterministic
`ResolvedModelInputBindings` plan before estimation begins. Each entry retains
the model-required target input ID, typed source, compiled target unit, source
unit, conversion, source declaration, and model identity. Runtime evaluation
executes this plan once per timestamp or transition and inserts the converted
value under the target ID. For example,
`custom.flow_drive = "environment:flow"` produces `flow_drive -> value`; it
does not emit only the built-in `flow` ID.

Precedence is explicit custom target binding, then configured standard binding,
then the automatic standard default. Duplicate custom targets and unknown
compiled targets are typed errors. Required unavailable sources and dimension
mismatches are typed errors; optional unavailable sources remain absent. Flow,
temperature, conductivity, ionic strength, named environment series, event
fields, activity steps, transduction drive, and finite constants are represented
by typed source variants. Environmental flow is normalized through the typed
flow-unit boundary before it enters the shared plan.

## Permanent integration evidence

The checked-in `phase6_estimation` matrix protects the complete approved
boundary:

- `custom_flow_drive_binding_executes_in_normal_estimation_runtime` is the P1
  reproduction and exercises compiled definition loading, SHA-256 provenance,
  normal EKF observation evaluation, custom target insertion, and report output.
- `compiled_legacy_equivalent_permanent_parity_matrix_covers_states_filters_and_scenarios`
  and `compiled_legacy_parity_covers_condition_sensitivity_state_for_ekf_and_ukf`
  compare Legacy only with Compiled + LegacyEquivalentV1. The ordinary numeric
  tolerance is `1e-10`; condition/sensitivity UKF compatibility uses `1e-8`.
- `compiled_activity_events_are_applied_once_at_irregular_transitions_with_provenance`
  covers irregular, multiple-in-one-interval, multiple-across-interval,
  initial-time, and no-event behavior with dynamic event provenance.
- `compiled_transduction_drive_modes_cover_none_activity_step_event_field_and_failures`
  covers None, ActivityStep, and ExplicitEventField, including missing and
  incompatible values. Active simulation truth and validation are covered by
  `compiled_reduced_active_transduction_truth_and_validation_use_stable_state_ids`.
- `estimation_report_matrix_renders_honest_backend_profile_and_custom_definition`,
  `old_estimation_configuration_fixture_matrix_preserves_legacy_defaults`, and
  `old_estimation_artifact_migration_matrix_keeps_identity_honest_and_deterministic`
  protect human-readable model narratives and old config/report/truth/validation/
  comparison compatibility.

The exact profile, filter, state model, scenario, protected contract, and test
function mapping is maintained in the testing QA specification and the
traceability matrix.
