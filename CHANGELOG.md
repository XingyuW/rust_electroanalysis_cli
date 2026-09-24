# Changelog

All notable changes to this project are documented here.

## 0.2.0 — 2026-09-24

### Added

- Release-ready electrochemical analysis workflows for EIS, potentiometric transients,
  calibration, mechanism comparison, signal characterization, sensor health, state
  estimation, plotting, and reduced-order ISM model analysis.
- Provider-backed CSV/TXT/DAT/XLSX ingestion through the pinned `electrodata-io`
  revision with typed diagnostics and provenance.
- Phase F specifications, protected bootstrap-ref policy, public bootstrap root,
  currentness/identity rules, and independently reviewed TEST_ONLY conformance evidence.
- Durable release documentation that separates software usability from Phase F REAL
  governance authority.

### Changed

- Package version advanced to `0.2.0`; generated provenance now records
  `rust_electroanalysis_cli@0.2.0`.
- Release documentation now records the current Phase F REAL-authority state and the
  exact future prerequisites for continuing the REAL bootstrap.

### Governance status

- **Phase F REAL authority is NO-GO for v0.2.0.**
- REAL reviewer readiness is `0 / 5`.
- No REAL reviewer verifier, REAL reviewer subjects, REAL currentness proof, or
  reviewer-bootstrap monotonic head exists.
- TEST_ONLY rehearsal/review evidence is non-authoritative and is not promoted by this
  release.
- v0.2.0 is a software/research-tool release; it is not a G3 approval, physical
  validation approval, or Phase F REAL authorization.

### Scientific scope

- Analysis outputs remain evidence, not automatic physical-mechanism diagnoses.
- Reduced-order ISM modes remain phenomenological unless separately supported by
  physical evidence.
- Phase F REAL governance and physical-validation claims remain fail-closed until their
  published prerequisites are satisfied.

## 0.1.0 — 2026-08-05

### Added

- Versioned, typed cross-workflow JSON artifact validation with explicit legacy migration.
- Unified reduced-order ISM definitions, static registry, graph compiler,
  built-in components, model workflows, and validation manifests.
- Compiled-model integration for EKF and UKF with named voltage contributions
  and visible unexplained residuals.
- Context-aware health/mechanism evidence and operational equilibrium recognition.

### Corrected

- Exact first-order relaxation stepping and runtime ion-charge handling.
- Panic-prone malformed-input paths in model, transient, unit, spreadsheet,
  fitting, and estimation workflows.
- Unsupported proxy values in scientific-validation metrics.
- Model decomposition input handling, state propagation, and plot contents.

### Compatibility

- Existing CLI commands are preserved.
- Historical kind-less artifacts are accepted only through typed legacy contracts.
- Existing state-estimation reports remain readable through additive defaults.

### Scientific scope

- Synthetic evidence is not physical validation.
- Phenomenological modes receive no automatic physical identity.
- High-fidelity Nernst–Planck transport and real parameter/state recovery are
  intentionally deferred to separately validated work.
