# 12 — Change Management Playbook

**Identifier:** `DOC-12`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## Who This Guide Is For

This guide is designed for the **project owner** — someone with strong electrochemical and scientific knowledge but limited formal software-engineering experience. Each procedure explains **what to do**, **which documents to read first**, and **how to verify your change is correct**.

---

## Documentation Authority and Conflict Resolution

Use this precedence order whenever documentation conflicts:

1. **Implementation (`src/`) is authoritative for current runtime behavior.**
2. **Engineering specification (`docs/engineering_specification/*.md`) is authoritative for curated requirements/process and must be kept aligned with implementation.**
3. **README is user-facing guidance and must be reconciled to the specification + implementation when drift is found.**

Required action on conflict: **treat it as documentation drift, not a silent behavior change**. Update docs (and traceability) to match implementation, or explicitly approve and implement a behavior change with corresponding tests and spec updates.

---

## Decision Guide

```
What do you want to change?
    |
    ├── Fix a bug ──────────────→ Section 1
    ├── Add a CLI command ──────→ Section 2
    ├── Add a new analysis ─────→ Section 3
    ├── Add a scientific equation → Section 4
    ├── Modify an existing equation → Section 5
    ├── Add an equivalent-circuit element → Section 6
    ├── Change an input format ──→ Section 7
    ├── Change an output format ─→ Section 8
    ├── Add a dependency ───────→ Section 9
    ├── Refactor code ──────────→ Section 10
    ├── Change a public interface → Section 11
    └── Prepare a release ──────→ Section 12
```

---

## Resolved Implementation Clarifications (Moved from DOC-16)

### 1. Runner layering for plot/search

- `main.rs` dispatches `plot`/`eis search` through `src/runners/plot.rs` and `src/runners/search.rs`.
- Those runner files are thin workflow boundaries that currently delegate to `src/plot_runner.rs` and `src/search_runner.rs`.
- Treat this as **intentional layering**, not dead code, unless an explicit consolidation refactor is approved.

### 2. `estimate simulate --scenario` format

- Scenario files are TOML decoded into `estimation::simulation::SimulationScenario` (`src/estimation/simulation.rs`).
- Current scenario schema is version `2` (`schema_version = 2` default).
- Required fields are represented by struct members (sample count, interval, activity/temperature trajectories, polarization inputs, noise/outlier/missing fractions, seed).

### 3. Automatic plotting behavior by workflow

| Workflow | Plot generation behavior |
|----------|--------------------------|
| `transient fit` | Auto-plots per selected event when `transient.plotting.enabled = true` |
| `calibration fit` | Auto-plots calibration report when `calibration.plotting.enabled = true` |
| `mechanism compare` / `mechanism trend` | Calls mechanism plotting during export; files are emitted when compatible data exists |
| `signal characterize` | Auto-plots PNG diagnostics when `signal.plotting.enabled = true` |
| `health assess` | Auto-plots `health_feature_deviations.png` when `health.plotting.enabled = true` |
| `health trend` | No automatic plot call in runner (CSV/JSON export only) |
| `estimate run` | Auto-plots estimation PNG diagnostics when `estimation.plotting.enabled = true` |
| `estimate validate` / `simulate` / `compare` | No automatic plotting in runners |

### 4. Estimation process/measurement noise defaults

- Process noise defaults (configured source):  
  `activity_variance_per_s = 1e-5`, `baseline_variance_v2_per_s = 1e-10`, `polarization_variance_v2_per_s = 1e-8`, `condition_variance_per_s = 1e-9`.
- Measurement noise default source is `signal_robust_variance`, with fallback to configured variance (`configured_variance_v2 = 1e-6`, clamped by `minimum_variance_v2 = 1e-12`, `maximum_variance_v2 = 1.0`).
- Source resolution logic is implemented in `src/estimation/covariance.rs` and records provenance in output diagnostics.

### 5. `TemperatureMode::ReferenceNormalized` behavior

- In calibration fitting/prediction, effective temperature is selected by `effective_temperature_k` in `src/potentiometry/calibration/nernst.rs`.
- `ReferenceNormalized` uses the configured reference temperature (in kelvin) as the slope-evaluation temperature for all points; it is **not** a per-point multiplicative ratio transform.

---

## Section 1: Fixing a Bug

### Before coding
1. Read `00_project_overview.md` for system context
2. Read `02_architecture.md` to locate the affected module
3. Read `03_module_specifications.md` for the module's purpose and interfaces
4. Check `14_risk_and_technical_debt_register.md` — is this a known issue?
5. Read the relevant source files (find them via `03_module_specifications.md`)

### Questions to answer
- Is this a scientific bug (wrong equation) or a software bug (crash, wrong output)?
- Does the fix change any numerical results?
- Which existing tests should catch this? (Check `09_testing_and_quality_assurance.md`)
- Does the fix need a new test?

### Likely affected files
- The source file containing the bug
- Test files that cover the affected module
- `13_traceability_matrix.md` (update if needed)

### Required tests
- If a test should have caught this bug but didn't: add a regression test
- If the fix changes behaviour: update existing tests

### Scientific validation
- For equation changes: manually verify with known input/output pairs
- For fitting changes: verify on a synthetic dataset with known parameters

### Specification updates
- `07_scientific_models_and_equations.md` if an equation changed
- `14_risk_and_technical_debt_register.md` if a risk is resolved
- `16_open_questions.md` if the fix raises new questions

### Completion criteria
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all` passes
- [ ] New regression test added (if applicable)
- [ ] Specification updated

---

## Section 2: Adding a CLI Command

### Before coding
1. Read `05_cli_and_interfaces.md` for existing command patterns
2. Read `02_architecture.md` for the runner layer design
3. Read an existing runner (e.g., `src/runners/fit.rs`) as a template
4. Read `12_change_management_playbook.md` (this document)

### Questions to answer
- Which workflow does this command expose?
- Is there an existing scientific module, or does one need to be created?
- What configuration does it need? (New TOML config file or reuse existing?)
- What are the required vs optional arguments?

### Likely affected files
| Layer | Files |
|-------|-------|
| CLI | `src/cli.rs` (add subcommand, args, CommandSpec variant) |
| Dispatch | `src/main.rs` (add match arm) |
| Runner | `src/runners/<new>.rs` (new file) |
| Runner module | `src/runners/mod.rs` (add `pub mod <new>`) |
| Config | `src/<new>_config.rs` (if new config needed) |
| Results | `src/results/<new>.rs` (if new result types) |
| Workspace | `src/workspace.rs` (add config path constant, LastRunMode variant) |

### Required tests
- Unit test for CLI argument parsing in `cli.rs`
- Integration test in `tests/` for the new command
- Scientific validation test if the command produces numerical results

### Specification updates
- `05_cli_and_interfaces.md` — document new command
- `06_workflows.md` — add new workflow
- `03_module_specifications.md` — document new module
- `13_traceability_matrix.md` — add new entries

### Compatibility
- New command must not break existing commands
- Legacy flags should not be needed for new commands unless back-compat is required
- Existing config files must remain valid

---

## Section 3: Adding a New Analysis Workflow

Same steps as Section 2, plus:
- Create a new scientific module if the analysis is novel
- Define result types in `src/results/`
- Create configuration module if needed
- Create default config file content (embedded in `workspace.rs`)
- Add plotting support in `src/plottings/` if figures are generated

---

## Section 4: Adding a Scientific Equation

### Before coding
1. Read `07_scientific_models_and_equations.md` for existing equation patterns
2. Identify the module where the equation belongs (impedance, potentiometry, signal, etc.)
3. Read `08_validation_and_constraints.md` for parameter constraints

### Questions to answer
- What is the mathematical expression (with units)?
- What are the parameter names, units, and valid ranges?
- What are the domain assumptions (e.g., ω > 0, t ≥ 0)?
- Is there an existing test pattern to follow?

### For circuit elements specifically
1. Add variant to `ElementType` enum in `src/impedance/elements.rs`
2. Implement `code()`, `display_name()`, `param_count()`, `constraints()`, `param_names()`, `param_units()`, `parameter_bounds()`, `calculate()`
3. Add parser token in `src/impedance/circuits.rs` (`parse_element_type`)
4. Add initial guess logic in `src/impedance/fitting.rs` (`fill_guesses`)
5. Update README circuit element table

### Scientific validation
- Verify DC limit behaviour (ω → 0)
- Verify high-frequency limit (ω → ∞)
- Test with known special cases (e.g., CPE with α=1 → ideal capacitor)

---

## Section 5: Modifying an Existing Equation

### Before coding
⚠️ **This is the highest-risk change.** Read everything below first.

1. Read `07_scientific_models_and_equations.md` — find the equation ID
2. Read `13_traceability_matrix.md` — find every test, command, and output affected
3. Read `14_risk_and_technical_debt_register.md` — check for related risks
4. Read `16_open_questions.md` — check for unresolved issues

### Questions to answer
- Why is the change needed? (Scientific correction? Better model? New parameter?)
- Does this change the meaning of existing parameters?
- Will existing results change? By how much?
- Is this backward-compatible, or does it require a schema version bump?

### Required tests
- Update all existing tests that depend on the equation output
- Add comparison tests showing old vs new behaviour
- Validate against known analytical solutions if available

### Specification updates
- Update `07_scientific_models_and_equations.md` with a version note
- Update `13_traceability_matrix.md`
- If backward-incompatible: create an ADR in `adr/`

---

## Section 6: Adding an Equivalent-Circuit Element

Follow Section 4 "For circuit elements specifically."

---

## Section 7: Changing an Input Format

### Likely affected files
- `src/data_file/` — parser modules
- `src/data_file/input_kind.rs` — format detection
- Tests that use the changed format
- Configuration that references the format

### Questions
- Is this a new format or a change to an existing format parser?
- Will existing data files still parse correctly?
- Do column names/units change?

---

## Section 8: Changing an Output Format

### Likely affected files
- `src/results/` — if result structures change
- Runner files — if output writing logic changes
- Tests that validate output content

### Questions
- Is this backward-compatible (additive change)?
- Does it require a schema version bump?
- Do downstream consumers (other workflow commands) need updates?

---

## Section 9: Adding a Dependency

### Before coding
1. Check if functionality already exists in current dependencies
2. Verify the dependency is actively maintained
3. Check license compatibility

### Steps
1. Add to `Cargo.toml` under `[dependencies]`
2. Run `cargo update` to refresh lock file
3. Verify `cargo build` and `cargo test --all` still pass
4. Document in `03_module_specifications.md` (external dependencies section)

---

## Section 10: Refactoring Code Without Changing Behaviour

### Before coding
1. Read `02_architecture.md` for dependency direction
2. Run the full test suite to establish a baseline

### Rules
- Do not change public interfaces without a deprecation path
- Do not move scientific equations to different modules without updating traceability
- Run tests after every refactoring step

### Required tests
- All existing tests must continue to pass
- No new tests are strictly required for pure refactoring
- Consider adding tests for any behaviour that was previously untested

---

## Section 11: Changing a Public Interface

### Before coding
1. Read `src/lib.rs` to see all public re-exports
2. Identify all consumers (CLI, tests, other modules)

### Rules
- Deprecate before removing
- Use type aliases during transition
- Update all call sites

---

## Section 12: Preparing a Release

### Steps
1. Run full CI pipeline: `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all && cargo build --release`
2. Update version in `Cargo.toml` (if applicable)
3. Review `16_open_questions.md` — anything resolved?
4. Review `14_risk_and_technical_debt_register.md` — any new items?
5. Update `00_project_overview.md` if capabilities changed
6. Update this specification for any changed behaviour
7. Commit all specification changes alongside code changes
8. Tag the release commit: `git tag v<version>`

### Release checklist
- [ ] All CI checks pass on both Linux and macOS
- [ ] Release build succeeds
- [ ] Version updated (if applicable)
- [ ] Specification documents updated
- [ ] Breaking changes documented
- [ ] New features added to `00_project_overview.md`
- [ ] `13_traceability_matrix.md` updated
