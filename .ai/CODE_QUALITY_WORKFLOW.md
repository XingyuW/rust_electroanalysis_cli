# Model-Independent Software Planning, Implementation, Review, and Delivery Protocol

## 1. Purpose

This protocol governs all software repository planning, implementation, debugging, remediation, engineering review, scientific review, validation, and release-readiness work.

The workflow is intentionally model-independent. Different models may be assigned to different roles, but quality requirements do not change with the selected model.

Model capability, confidence, or reputation must never be treated as evidence of correctness. Correctness must be established through current repository evidence, explicit requirements, executable validation, independent review, requirement traceability, and clearly defined acceptance criteria.

## 2. Required Roles

The workflow distinguishes six roles.

### 2.1 Planning and Architecture Agent

Responsible for:

1. Inspecting the current repository.
2. Establishing the actual problem.
3. Confirming or investigating the root cause.
4. Defining architecture and interfaces.
5. Decomposing requirements.
6. Defining production execution paths.
7. Defining scientific and numerical assumptions when applicable.
8. Designing tests and fixtures.
9. Establishing compatibility requirements.
10. Defining acceptance and failure criteria.
11. Producing the implementation contract.

### 2.2 Implementation Agent

Responsible for:

1. Inspecting the repository independently before editing.
2. Verifying the implementation contract against current code.
3. Implementing the smallest coherent correction.
4. Preserving unrelated behavior.
5. Adding required tests.
6. Exercising production execution paths.
7. Running required validation.
8. Reporting exact implementation and validation evidence.
9. Escalating material conflicts instead of improvising.

### 2.3 Independent Engineering Review Agent

Responsible for independently reviewing:

1. The complete cumulative diff.
2. Runtime integration.
3. Root-cause resolution.
4. Error handling.
5. Data integrity.
6. API and schema compatibility.
7. Test adequacy.
8. Regression risk.
9. Maintainability.
10. Delivery readiness.

### 2.4 Scientific and High-Risk Review Agent

Required when changes affect scientific calculations, mathematical or numerical models, data interpretation, parsing, automatic selection, calibration, physical quantities, public scientific output, or any behavior capable of producing plausible but incorrect results.

### 2.5 Remediation Agent

Responsible for correcting confirmed review findings while preserving scope, architecture, compatibility, and unrelated behavior.

### 2.6 Final Re-review Agent

Responsible for independently reviewing the complete corrected branch and issuing the final evidence-based release verdict.

## 3. Independence Requirements

The following requirements are mandatory.

1. The implementation agent must never be the sole approver of its own work.

2. Initial independent review must occur in a fresh review context whenever practical.

3. If a reviewer modifies production code, that reviewer becomes an implementation agent and another independent review is required.

4. Final review must examine the complete cumulative diff against the intended base branch rather than only the most recent remediation commit.

5. Implementation summaries must not substitute for repository inspection.

6. Test reports must not substitute for examination of the production execution path.

7. No agent may issue GO solely because another agent reported successful implementation.

## 4. Evidence Hierarchy

Agents must distinguish the following categories.

### Confirmed evidence

Evidence directly supported by:

1. Current source code.
2. Current repository structure.
3. Current branch diff.
4. Executed tests.
5. Reproduced runtime behavior.
6. Current configuration.
7. Current documentation when documentation itself is authoritative for a contract.

### Review claims

Statements made by a previous reviewer that require reconciliation against current repository evidence.

### Historical information

Previously correct information that may no longer describe the current branch.

### Assumptions

Unverified statements required for reasoning.

### Hypotheses

Possible explanations requiring investigation.

A root cause must not be described as confirmed unless repository or runtime evidence supports it.

## 5. Repository Inspection Before Planning

Before producing a final implementation plan, inspect:

1. Repository identity.
2. Current branch.
3. Intended base branch.
4. Working tree status when accessible.
5. Relevant recent changes.
6. Repository structure.
7. Runtime entry points.
8. Relevant modules.
9. Public APIs.
10. Data structures.
11. Configuration structures.
12. Serialization formats.
13. Existing tests.
14. Existing fixtures.
15. Feature flags.
16. Dependencies.
17. CI configuration.
18. Formatting requirements.
19. Linting requirements.
20. Build and release commands.

Establish baseline behavior before modification whenever practical.

Existing failures must be separated from regressions caused by the proposed implementation.

## 6. Root-Cause Requirements

The planning agent must identify:

1. The observed defect.
2. The expected behavior.
3. The current behavior.
4. The production execution path involved.
5. The earliest point where expected and actual behavior diverge.
6. The confirmed root cause when sufficient evidence exists.

When root cause is uncertain:

1. State competing hypotheses.
2. State evidence supporting each hypothesis.
3. Define diagnostic work required to distinguish them.
4. Do not design a permanent implementation around an unverified assumption.

## 7. Implementation Contract Requirements

Every implementation plan must be sufficiently detailed to function as an implementation contract rather than a conceptual description.

The contract must define:

### 7.1 Problem Definition

1. Problem being solved.
2. Observable impact.
3. Expected behavior.
4. Current behavior.
5. Confirmed root cause or clearly identified hypothesis.

### 7.2 Scope

Identify expected changes to:

1. Files.
2. Modules.
3. Functions.
4. Traits or interfaces.
5. Public APIs.
6. Data structures.
7. Configuration.
8. Serialization.
9. Tests.
10. Fixtures.
11. Documentation.
12. CLI behavior.
13. Output artifacts.

### 7.3 Execution Paths

Trace the complete relevant production execution path, including as applicable:

1. User entry point.
2. CLI or API parsing.
3. Configuration loading.
4. Validation.
5. Data loading.
6. Parsing.
7. Transformation.
8. Unit conversion.
9. Runtime binding.
10. Computation.
11. State update.
12. Persistence.
13. Reporting.
14. Output generation.
15. Error propagation.

Explicitly verify that configuration accepted during validation is actually consumed during runtime execution.

### 7.4 Interface Contracts

For every new or modified interface define:

1. Inputs.
2. Outputs.
3. Types.
4. Units.
5. Ownership and lifetime expectations when relevant.
6. Error behavior.
7. Preconditions.
8. Postconditions.
9. Invariants.
10. Compatibility requirements.

### 7.5 Scientific and Numerical Requirements

When applicable define:

1. Equations.
2. Variables.
3. Units.
4. Dimensions.
5. Unit conversions.
6. Sign conventions.
7. Reference states.
8. Initial conditions.
9. Boundary conditions.
10. Parameter constraints.
11. Numerical tolerances.
12. Approximations.
13. Domain of validity.
14. Limiting behavior.
15. Conservation relationships.
16. Calibration assumptions.
17. Identifiability assumptions.
18. Uncertainty treatment.

Scientific assumptions must never remain hidden inside undocumented constants or default values.

### 7.6 Behavioral Invariants

Explicitly identify behavior that must not change.

Examples include:

1. Existing public APIs.
2. Existing output formats.
3. Existing valid input interpretation.
4. Existing error semantics.
5. Ordering guarantees.
6. Unit conventions.
7. Scientific parameter meanings.
8. Default behavior.
9. Supported feature combinations.

### 7.7 Compatibility

Define requirements for:

1. Backward compatibility.
2. Public API compatibility.
3. Configuration compatibility.
4. Serialization compatibility.
5. Stored artifacts.
6. Downstream crates or applications.
7. Migration behavior.

### 7.8 Non-Goals

Explicitly state what is not part of the implementation.

Unrelated refactoring, renaming, dependency upgrades, formatting changes, architectural redesign, and cleanup should normally be prohibited unless required by the root-cause correction.

## 8. Requirement Identification and Traceability

Assign stable identifiers such as:

R1, R2, R3 for requirements.

AC1, AC2, AC3 for acceptance criteria.

T1, T2, T3 for required tests.

Every final implementation must support traceability:

Requirement → implementing code → validating test → validation result.

No requirement may be considered complete merely because relevant code exists.

## 9. Test Requirements

The plan must determine which of the following are required.

### Positive tests

Verify valid intended behavior.

### Negative tests

Verify explicit rejection or failure behavior.

### Malformed-input tests

Verify incorrect or incomplete external inputs.

### Boundary tests

Verify values at domain, numerical, indexing, time, size, or configuration boundaries.

### Limiting-case tests

Verify mathematically or scientifically meaningful limits.

### Regression tests

Reproduce the original defect and prove that the correction prevents recurrence.

### Integration tests

Exercise interactions among real production components.

### End-to-end tests

Exercise behavior through the actual user-facing production path.

### Compatibility tests

Verify existing configurations, APIs, serialized artifacts, or downstream consumers.

Tests must not validate only helper functions when the defect occurs in production integration.

A test suite passing against disconnected helper logic is insufficient evidence.

## 10. Implementation Agent Requirements

Before editing, the implementation agent must:

1. Inspect the current repository.
2. Confirm branch and scope.
3. Verify all referenced paths and symbols.
4. Verify assumptions in the plan.
5. Run relevant baseline validation when practical.
6. Identify discrepancies between plan and repository.

During implementation it must:

1. Make the smallest coherent change.
2. Preserve behavioral invariants.
3. Avoid unrelated refactoring.
4. Preserve public behavior unless explicitly changed.
5. Implement required error handling.
6. Add required tests.
7. Exercise production paths.
8. Preserve structured error context.
9. Avoid silent fallback unless explicitly specified.
10. Avoid silent data loss.
11. Avoid ambiguous implicit selection.
12. Avoid test-specific production logic.

The agent must never:

1. Weaken a valid test merely to obtain a passing suite.
2. Delete a valid regression test without justification.
3. Suppress warnings to conceal an unresolved problem.
4. Introduce placeholder behavior while claiming completion.
5. Hardcode expected test values into production logic.
6. Invent architectural decisions silently.
7. Change scientific assumptions silently.
8. Change numerical methods silently.
9. claim an unexecuted validation command passed.

## 11. Escalation Conditions

Implementation must escalate when current repository evidence conflicts materially with the approved contract.

Examples include:

1. Required symbol does not exist.
2. Runtime architecture differs materially from the plan.
3. Correct implementation requires a public API change that was not approved.
4. Scientific assumptions are incomplete or contradictory.
5. Required units or reference states are undefined.
6. Correct implementation requires compatibility breakage.
7. Existing behavior makes the requested invariant impossible.
8. The root-cause hypothesis is contradicted by current code.

The escalation report must state:

1. Approved assumption.
2. Actual evidence.
3. Conflict.
4. Affected requirements.
5. Available alternatives.
6. Risks and tradeoffs.
7. Decision required.

## 12. Review Feedback Reconciliation

When review feedback is supplied, never automatically accept or reject it.

For every finding:

1. Assign a stable finding ID.
2. Preserve the reviewer’s original severity and concern.
3. Locate the cited code.
4. Trace the affected runtime path.
5. Verify the reproduction when practical.
6. Compare the finding with current repository state.
7. Determine the actual root cause.

Classify every finding as one of:

CONFIRMED

PARTIALLY CONFIRMED

ALREADY RESOLVED

STALE

CONTRADICTED BY CURRENT CODE

DUPLICATE

OUT OF SCOPE

UNVERIFIABLE WITH AVAILABLE EVIDENCE

Identify interactions among findings when several symptoms originate from the same root cause.

## 13. Independent Engineering Review Requirements

The engineering reviewer must review the complete branch diff against the intended base branch.

The first review pass should be read-only.

The reviewer must verify:

1. Every requirement and acceptance criterion.
2. Root-cause resolution.
3. Production runtime execution.
4. Configuration propagation.
5. Data propagation.
6. Error propagation.
7. Structured error context.
8. Silent fallback behavior.
9. Silent data loss.
10. Ambiguous automatic selection.
11. Default substitution.
12. Partial execution.
13. Unexpected API changes.
14. Unexpected schema changes.
15. Unexpected serialization changes.
16. Feature-flag behavior.
17. Compatibility.
18. Regression risk.
19. Resource handling.
20. Concurrency when relevant.
21. Performance when relevant.
22. Security when relevant.
23. Documentation consistency.
24. Test adequacy.
25. Validation evidence.

The reviewer must specifically determine whether tests can pass while production behavior remains broken.

The reviewer must determine whether configuration can validate successfully while never affecting actual execution.

The reviewer must determine whether parsed or transformed data can fail to reach the component that is supposed to consume it.

## 14. Review Finding Format

Every finding must include:

1. Finding ID.
2. Severity.
3. File.
4. Precise code location.
5. Affected execution path.
6. Supporting evidence.
7. User or project impact.
8. Reproduction or failure scenario when possible.
9. Required correction.
10. Affected requirement or acceptance criterion.

Reviewers should prioritize demonstrated correctness defects over stylistic preference.

Speculative findings must be clearly identified rather than presented as confirmed defects.

## 15. Scientific and High-Risk Review Triggers

Dedicated scientific or high-risk review is required when changes affect:

1. Scientific equations.
2. Physical models.
3. Chemical models.
4. Biological models.
5. Electrochemical models.
6. Statistical models.
7. Mathematical models.
8. Numerical algorithms.
9. Unit conversions.
10. Calibration.
11. Parameter estimation.
12. Uncertainty.
13. Scientific interpretation.
14. Sensor data processing.
15. Temporal alignment.
16. Interpolation.
17. Resampling.
18. Filtering.
19. Event detection.
20. Automatic file detection.
21. Automatic worksheet selection.
22. Parsing.
23. Data truncation or reordering.
24. Scientific figures or tables.
25. Public scientific output.
26. Cross-repository scientific contracts.
27. Any behavior capable of producing plausible but incorrect results.

## 16. Scientific Review Requirements

Scientific review must evaluate:

1. Model validity.
2. Domain of applicability.
3. Equation correctness.
4. Transformation correctness.
5. Dimensional consistency.
6. Unit consistency.
7. Conversion direction.
8. Sign conventions.
9. Reference states.
10. Initial conditions.
11. Boundary conditions.
12. Limiting behavior.
13. Parameter constraints.
14. Approximation validity.
15. Identifiability.
16. Confounding.
17. Calibration assumptions.
18. Numerical stability.
19. Convergence.
20. Conditioning.
21. Discretization.
22. Overflow and underflow.
23. Numerical tolerance.
24. Measurement error.
25. Missing data.
26. Outliers.
27. Censoring.
28. Sensitivity.
29. Uncertainty propagation.
30. Temporal leakage.
31. Artificial agreement introduced through preprocessing.
32. Whether scientifically invalid input fails explicitly.

Scientific review must inspect the production code implementing the equations, not merely documentation describing them.

Where feasible, validation should include:

1. Analytical reference cases.
2. Hand calculations.
3. Synthetic cases.
4. Limiting cases.
5. Conservation relationships.
6. Established reference outputs.
7. Independent calculation.

## 17. Remediation Requirements

The remediation agent must:

1. Address every confirmed P0 finding.
2. Address every confirmed P1 finding.
3. Address required P2 findings.
4. Preserve approved architecture.
5. Preserve unrelated behavior.
6. Avoid scope expansion.
7. Add or strengthen regression tests.
8. Execute affected validation.
9. Inspect the cumulative diff.
10. Update requirement traceability.
11. Report every finding and its resolution status.

A finding is not resolved merely because code associated with it changed.

Resolution requires both:

1. Corrective implementation.
2. Validation evidence demonstrating the corrected behavior.

## 18. Final Re-review Requirements

After remediation, the final reviewer must:

1. Review the complete corrected branch against the intended base branch.
2. Re-evaluate the original requirements.
3. Re-evaluate prior findings.
4. Confirm P0 and P1 resolution.
5. Verify remediation tests.
6. Verify production execution paths.
7. Check for new regressions.
8. Check cumulative compatibility.
9. Verify scientific review where required.
10. Confirm final validation corresponds to the current commit.
11. Identify accidental or unrelated modifications.
12. Issue the final verdict.

The reviewer must not rely on the remediation agent’s assertion that findings were resolved.

## 19. Validation Requirements

Validation commands must be determined from repository evidence rather than assumed blindly.

For a Rust workspace, commonly applicable checks include:

cargo fmt --all --check

cargo check --workspace --all-targets --all-features

cargo clippy --workspace --all-targets --all-features -- -D warnings

cargo test --workspace --all-features

cargo test --doc --workspace

For release readiness, where applicable:

cargo build --release --workspace --all-features

Additional validation may include:

1. No-default-feature testing.
2. Feature matrix testing.
3. Integration suites.
4. CLI end-to-end execution.
5. Fixture-based tests.
6. Downstream compatibility tests.
7. Benchmarking.
8. Reproducibility tests.

No command may be reported as passing unless it was actually executed successfully.

Compilation and unit tests are necessary but not sufficient evidence of runtime correctness.

## 20. Severity Framework

### P0

Catastrophic correctness, safety, security, data-loss, scientific-validity, or release-integrity failure.

### P1

Major correctness, runtime-integration, compatibility, scientific-validity, data-integrity, or delivery blocker.

### P2

Non-blocking defect, maintainability issue, documentation problem, limited test gap, or documented technical debt that does not invalidate required behavior.

## 21. Verdict Framework

### NO-GO

NO-GO is mandatory when any of the following remains:

1. Unresolved P0.
2. Unresolved P1.
3. Root cause not demonstrated as resolved.
4. Mandatory acceptance criterion unverified.
5. Required validation failure.
6. Required production execution path unverified.
7. Unresolved scientific or numerical validity.
8. Silent data-integrity risk.
9. Broken compatibility.
10. Insufficient evidence supporting required behavior.

### GO WITH DOCUMENTED NON-BLOCKING DEBT

Permitted only when:

1. All P0 findings are resolved.
2. All P1 findings are resolved.
3. Required behavior is correct.
4. Required scientific behavior is valid.
5. Required validation passes.
6. Remaining findings are exclusively P2.
7. Remaining P2 debt is explicitly documented.

### GO

Requires:

1. Every required acceptance criterion satisfied.
2. Required validation successfully executed.
3. All P0 findings resolved.
4. All P1 findings resolved.
5. Required engineering review complete.
6. Required scientific review complete.
7. No unresolved correctness, compatibility, scientific-validity, or data-integrity risk.

## 22. Required Implementation Report

The implementation agent must report:

1. Repository.
2. Base branch.
3. Target branch or commit.
4. Confirmed root cause.
5. Implementation approach.
6. Changed files.
7. Changed APIs.
8. Changed schemas.
9. Tests added or modified.
10. Baseline validation.
11. Final validation.
12. Commands not executed and reasons.
13. Remaining risks.
14. Requirement traceability.

The traceability table should contain:

Requirement | Acceptance criterion | Implementation location | Test | Result

## 23. Required Final Delivery Report

Final delivery must include:

1. Repository and branch scope.
2. Confirmed root cause.
3. Final implementation approach.
4. Complete changed-file summary.
5. Requirement-to-code traceability.
6. Requirement-to-test traceability.
7. Validation evidence.
8. Engineering review findings.
9. Scientific review findings where applicable.
10. Resolution status.
11. Remaining P2 debt.
12. Commands not executed.
13. Compatibility implications.
14. Current limitations.
15. Final GO, GO WITH DOCUMENTED NON-BLOCKING DEBT, or NO-GO verdict.
16. Evidence supporting the verdict.

## 24. Core Quality Principles

Every task must follow this sequence:

1. Inspect before planning.
2. Confirm before diagnosing.
3. Decompose before implementing.
4. Verify assumptions before editing.
5. Implement the smallest coherent correction.
6. Test production behavior rather than only helpers.
7. Trace requirements before declaring completion.
8. Review the complete cumulative diff.
9. Keep implementation and approval independent.
10. Escalate unresolved scientific and high-consequence decisions.
11. Re-review after material remediation.
12. Base final decisions on evidence rather than model confidence.

The workflow may be scaled according to task complexity, but repository inspection, evidence-based reasoning, executable validation, role separation, requirement traceability, and independent final approval must not be removed.
