# ISM Model Core Final GO Review

**Review date:** 2026-08-08
**Review decision:** GO — ready for reduced-order ISM components
**Consumer SHA:** `ae97a4b243fb5a7df24408a57261521a43172187`
**Provider SHA:** `dbb6b7d063972114c4208980723e12c807ab199e`

## Scope

This review covers only implementation of the approved extensible ISM model
core contracts. The scientific and uncertainty contracts were already approved;
no new scientific mechanisms or contract redesign was introduced.

## Severity findings

### P0

None.

### P1

None.

### P2

None.

### P3

- `ComponentRegistry::from_static_factories` panics on duplicate static entries;
  the fallible `register` API returns the typed duplicate-kind error. This is a
  programmer-only convenience-constructor concern and does not affect the
  immutable built-in registry.
- `component_validity_reports` does not independently prevalidate global runtime
  vectors. Model-level validity and all evaluation APIs do validate state,
  parameters, and inputs before producing scientific outputs.

Neither observation blocks reduced-order component work.

## Contract verification

| Area | Result | Evidence |
|---|---|---|
| Registry | PASS | Deterministic `BTreeMap` lookup; duplicate registration and unknown kinds are rejected. |
| Graph | PASS | Missing dependencies, self-dependencies, duplicate dependencies, and cycles are rejected; topological ordering is deterministic. |
| Bindings | PASS | State/parameter IDs map to stable definition-order indices; component slices and compiled summaries round-trip deterministically. |
| Evaluation | PASS | Continuous derivatives, discrete transitions, process Jacobians, observation voltage, observation variance, and auxiliary outputs are distinct APIs. |
| Composition | PASS | Only `AdditivePotential` enters voltage; observation variance remains `V²`; noise, auxiliary, and unexplained outputs remain separate. |
| Jacobians | PASS | Stable-ID coverage is validated; explicit zero coverage differs from missing coverage; undeclared numerical fallback is rejected. |
| Uncertainty | PASS | Missing stochastic covariance cannot produce `Complete`; full state/parameter covariance and off-diagonal terms are propagated with first-order quadratic forms. |
| Validity | PASS | Model and component validity outputs preserve warnings, violations, domains, and rejected evaluations. |
| Identifiability | PASS | Explicit `NotAssessed` reports, missing evidence, parameter requirements, observation requirements, and sensitivity targets are exposed. |
| Serialization | PASS | Model definitions and compiled summaries are versioned and deterministic; supported artifact paths reject nonfinite values. |
| Architecture | PASS | `src/model` has no CLI, runner, plotting, results, health, mechanism, or estimation dependency; only approved narrow potentiometry adapters are used. |
| Extensibility | PASS | An independent test component compiled with only implementation, descriptor, registry entry, and tests; no unrelated module edits were required. |

## Independent negative cases

All requested cases passed in a temporary review-only integration fixture:

- duplicate state ID → typed `DuplicateIdentifier`;
- duplicate parameter ID → typed `DuplicateIdentifier`;
- unknown component → typed `UnknownComponentKind`;
- missing dependency → typed `MissingDependency`;
- cycle → typed `CircularDependency`;
- unit mismatch → typed `UnitMismatch`;
- invalid composition → typed `InvalidComponentShape`;
- missing derivative → explicit partial coverage and uncertainty not `Complete`;
- covariance contradiction → typed covariance-contract error;
- reconstruction mismatch → typed `ContributionReconstruction`.

The temporary fixture was removed after execution and the worktree was clean.

## Validation commands

Final validation passed:

    cargo fmt --check
    cargo check --locked
    cargo clippy --locked --all-targets --all-features -- -D warnings
    cargo test --locked --all
    cargo build --locked --release
    git diff --check

Focused model suites also passed:

- `model_core`: 40 tests
- `model_builtins`: 19 tests
- `model_contracts`: 5 tests
- independent negative/extensibility fixture: 3 tests

The complete final locked suite passed with 353 tests and 15 ignored doctests.

## Commands failed

The first `cargo test --locked --all` attempt had one unrelated
`phase3_workflow` calibration extraction failure. The exact test passed on
rerun, and the second complete locked suite passed in full. No model-core test
failed.

## Final decision

**GO — ready for reduced-order ISM components**
