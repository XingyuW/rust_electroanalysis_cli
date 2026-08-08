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
