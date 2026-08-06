# 14 — Risk & Technical Debt Register

**Identifier:** `DOC-14`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## Classification Key

| Code | Category |
|------|----------|
| SCI | Scientific correctness |
| NUM | Numerical stability |
| UNIT | Unit consistency |
| DATA | Data integrity |
| ERR | Error handling |
| ARCH | Architecture |
| MAINT | Maintainability |
| PERF | Performance |
| PORT | Portability |
| REPRO | Reproducibility |
| TEST | Testing |
| DOC | Documentation |
| SEC | Security |

---

## Risk Register

| ID | Category | Description | Evidence | Files | Impact | Likelihood | Severity | Priority |
|----|----------|-------------|----------|-------|--------|-----------|----------|----------|
| RISK-001 | NUM | DC limit fallback (1e12 Ω) may mask physically meaningful low-frequency behaviour | Hardcoded in `elements.rs` for 7+ elements | `impedance/elements.rs` | Medium | Low | Low | P4 |
| RISK-002 | NUM | Division-by-zero guard in parallel admittance (1e12 Ω fallback) may produce misleading Nyquist plots at extreme parameters | `circuits.rs` L138-150 | `impedance/circuits.rs` | Low | Low | Low | P4 |
| RISK-003 | UNIT | CPE, La, Gw parameter units depend on the fitted α value, making units dimensionally dependent on the parameter | `elements.rs` param_units strings contain α | `impedance/elements.rs` | Low | — | Low | P4 |
| RISK-004 | ERR | Runtime panic-abort footprint from `unwrap`/`expect`/`unreachable!` is broader than plotting-only assumptions (43 runtime sites) | Runtime-only inventory: 43 sites (`unwrap` 21, `expect` 13, `unreachable!` 9); Tier-A user-data-adjacent sites concentrated in plotting/data-alignment paths | `plottings/`, `signal/sampling.rs`, `data_file/`, `estimation/`, `impedance/`, `potentiometry/` | Medium | Low | Medium | P2 |
| RISK-005 | ARCH | `rust_plots` crate alias (`extern crate self as rust_plots`) is a historical artifact; internal naming differs from crate name | `lib.rs` L6 | `src/lib.rs` | Low | — | Low | P4 |
| RISK-006 | REPRO | No deterministic build flag (`--frozen`); Cargo.lock pins versions but toolchain is not pinned | `ci.yml` uses `@stable` | `.github/`, `Cargo.toml` | Medium | Medium | Medium | P2 |
| RISK-007 | TEST | Several estimate subcommands and mechanism subcommands have inferred test coverage but no explicit integration tests | Coverage gaps noted in traceability | `tests/` | Medium | Medium | Medium | P2 |
| RISK-008 | TEST | No visual-correctness tests for plot output | Plots only tested for file existence | `tests/phase0` | Low | — | Low | P4 |
| RISK-009 | TEST | No cross-version schema compatibility tests | Schema version field exists but no migration tests | All result modules | Medium | Medium | Medium | P2 |
| RISK-010 | PORT | No Windows CI testing | `ci.yml` matrix: ubuntu + macos only | `.github/` | Medium | Low | Medium | P3 |
| RISK-011 | DOC | Embedded default config strings in `workspace.rs` may drift from actual config file defaults | Duplicated defaults | `workspace.rs`, `config/*.toml` | Low | Medium | Low | P4 |
| RISK-012 | ARCH | `plot_runner.rs` and `search_runner.rs` exist alongside newer `runners/plot.rs` and `runners/search.rs` — potential duplication | Two files for same responsibility | `src/plot_runner.rs`, `src/search_runner.rs` | Low | — | Low | P4 |
| RISK-013 | NUM | Genetic algorithm for ECM search uses fixed seed circuits; no option for exhaustive enumeration of simple circuits | `ecm_evolution.rs` seeding | `impedance/ecm_evolution.rs` | Low | — | Low | P4 |
| RISK-014 | NUM | Transient fit uses a custom optimizer rather than the Levenberg-Marquardt crate used for EIS — different convergence characteristics | `potentiometry/transient/fitting.rs` | `potentiometry/transient/fitting.rs` | Medium | Low | Low | P4 |
| RISK-015 | DATA | No schema version compatibility check when loading cross-workflow JSON inputs | JSON deserialization uses serde with defaults | All cross-workflow inputs | Medium | Medium | Medium | P2 |
| RISK-016 | UNIT | Some config fields have implicit units (e.g., temperature "default_celsius" vs internal kelvin) | Config uses Celsius, internal uses Kelvin | `calibration_config.rs`, `units.rs` | Low | — | Low | P4 |
| RISK-017 | SCI | Inverse Nernst slope check (|slope| ≥ 1e-15) may be too permissive for near-zero slopes in pathological cases | `nernst.rs` L101 | `potentiometry/calibration/nernst.rs` | Low | Low | Low | P4 |
| RISK-018 | NUM | Initial guess for Warburg α defaults to 0.5 when phase angle is non-finite; this may bias fits toward diffusion-like behaviour | `fitting.rs` L125-127 | `impedance/fitting.rs` | Low | Low | Low | P4 |
| RISK-019 | ERR | Invariant-guarded `unreachable!()` branches remain in transient model destructuring and dimensional unit conversions | `transient/models.rs` (4 sites), `potentiometry/units.rs` (4 sites) rely on prior validation guards | `potentiometry/transient/models.rs`, `potentiometry/units.rs` | Low | Very Low | Low | P4 |
| RISK-020 | MAINT | `plot_config.rs` has ~200 fields in `RawPlotStyle` — very large configuration surface | File size, field count | `plot_config.rs` | Medium | — | Low | P4 |
| RISK-021 | SCI | Fitted modes could be over-interpreted as physical mechanisms when concrete ISM components arrive | A core evidence contract exists, but no independent-evidence evaluator exists yet | `model/evidence.rs`, `model/equilibrium_recognition.rs` | High | Medium | High | P1 |
| RISK-022 | SCI | A future component could obscure unexplained voltage error | Core prevents residual ownership, but workflow-level reporting is deferred | `model/output.rs` | High | Low | Medium | P2 |
| RISK-023 | ARCH | Future built-in component additions could introduce forbidden workflow dependencies | Phase 02 core is clean; dependency enforcement is currently source/test reviewed | `src/model/` | Medium | Low | Low | P3 |
| RISK-024 | DATA | Model schema v1 has no historical artifact migration because it is newly introduced | `model_schema_migration.md` defines reject-and-migrate policy for future versions | `model_config.rs`, `results/model.rs` | Low | Low | Low | P4 |
| RISK-025 | SCI | Reduced-order modes may be mistaken for specific physical mechanisms | Built-ins use neutral/candidate names and evidence requirements, but no independent confirmation workflow exists | `model/builtins.rs` | High | Medium | High | P1 |
| RISK-026 | SCI | Explicit mapping prevents name/order inference, but mapping applicability and independent replication remain user-supplied scientific evidence | Phase 05 mappings preserve missing/contradictory evidence and cap unreplicated matches at weak | `mechanism/model_mapping.rs` | Medium | Medium | Medium | P2 |
| RISK-027 | DATA | Legacy transient events can omit matrix or temperature metadata; such events remain partitioned as `unknown` rather than being treated as comparable | Context-preserving feature keys | `health/features.rs` | Medium | Medium | Medium | P2 |
| RISK-028 | SCI | Deterministic model workflow outputs can be mistaken for fitted validation | Reports label synthetic outputs and preserve indeterminate equilibrium/identifiability evidence | `runners/model.rs` | Medium | Medium | Medium | P2 |
| RISK-029 | SCI | A validation manifest can reference synthetic or incomplete reference data | Results label non-real experiments and preserve unavailable profile-likelihood/state-recovery evidence | `model_validation.rs` | High | Medium | High | P1 |

---

## Priority Summary

| Priority | Count | Action |
|----------|-------|--------|
| P1 (Critical) | 0 | None |
| P2 (Medium) | 5 | Consider addressing in next release cycle |
| P3 (Low-Medium) | 1 | Address when touching related code |
| P4 (Low) | 14 | Documented; no immediate action required |
