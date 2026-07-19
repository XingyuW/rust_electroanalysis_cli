# Engineering Specification — rust_electroanalysis_cli

**Version:** 1.0  
**Generated:** 2026-07-18  
**Scope:** Complete repository (`052564` lines of Rust across `164` source files)  
**Status:** Descriptive — documents current system behaviour as verified from source

---

## 1. Purpose

This specification is the **authoritative source of truth** for understanding, reviewing, maintaining, debugging, extending, and validating the `rust_electroanalysis_cli` project. It is a descriptive document: it records what the system **actually does**, not what it was intended to do or should do in the future.

The **source code remains the authority** for current behaviour. This specification becomes the authority for intended behaviour **after** a proposed change has been reviewed, approved, and implemented. Until then, any discrepancy between this specification and the code is resolved in favour of the code.

## 2. Scope

This specification covers every major aspect of the system:

- Project purpose and scientific context
- System requirements derived from implemented behaviour
- Architecture, module decomposition, and data flow
- Module-level specifications with source-to-document mapping
- Data models, formats, units, and serialization
- CLI interface and command reference
- End-to-end workflows
- Scientific equations, equivalent-circuit models, and numerical methods
- Validation rules and constraints
- Test inventory and coverage gaps
- Error handling and diagnostics
- Build, release, and operations procedures
- Change-management playbook for non-engineer maintainers
- Traceability matrix connecting requirements to implementations
- Risk and technical-debt register
- Glossary of software and domain terminology
- Open questions and unresolved issues
- Architecture decision record (ADR) template

## 3. Documentation Map

| Document | Title | Purpose |
|----------|-------|---------|
| `README.md` | **this file** | Master entry point and index |
| `00_project_overview.md` | Project Overview | Scientific purpose, use cases, capabilities, "system in one page" |
| `01_system_requirements.md` | System Requirements | FR, NFR, SCI, VAL, CLI requirements with identifiers |
| `02_architecture.md` | Architecture | Module decomposition, data/control flow, Mermaid diagrams |
| `03_module_specifications.md` | Module Specifications | Per-module purpose, interfaces, invariants, source mapping |
| `04_data_models_and_units.md` | Data Models & Units | Structs, enums, file formats, unit registry, serialization |
| `05_cli_and_interfaces.md` | CLI & Interfaces | Every command, argument, default, and exit behaviour |
| `06_workflows.md` | Workflows | End-to-end pipelines with Mermaid diagrams |
| `07_scientific_models_and_equations.md` | Scientific Models | Equations, ECM topologies, parameter bounds, fitting methods |
| `08_validation_and_constraints.md` | Validation & Constraints | Input validation, parameter enforcement, convergence checks |
| `09_testing_and_quality_assurance.md` | Testing & QA | Test inventory, classification, coverage gaps, run commands |
| `10_error_handling_and_diagnostics.md` | Error Handling | Error types, propagation, unwrap/expect audit, logging |
| `11_build_release_and_operations.md` | Build, Release & Operations | Build commands, CI/CD, platform assumptions, packaging |
| `12_change_management_playbook.md` | Change Management | Practical procedures for non-engineer maintainers |
| `13_traceability_matrix.md` | Traceability Matrix | Requirement → module → CLI → test → output mapping |
| `14_risk_and_technical_debt_register.md` | Risk Register | Categorized risks with evidence and remediation guidance |
| `15_glossary.md` | Glossary | Software, Rust, electrochemistry, and domain terminology |
| `16_open_questions.md` | Open Questions | Unresolved issues requiring project-owner decisions |
| `adr/README.md` | ADR Template | How to record future architecture decisions |

## 4. Source-of-Truth Policy

| Question | Answer |
|----------|--------|
| What determines current behaviour? | **The source code** (this spec describes it) |
| What determines intended future behaviour? | **This specification** after a change is approved |
| What happens when code and spec disagree? | **The code is correct** — flag the discrepancy in `16_open_questions.md` |
| Who can change this specification? | The project owner, with input from domain/software reviewers |

## 5. Normative vs Descriptive Content

- **Descriptive** statements describe what the system currently does. They are labelled "**Verified**" when confirmed by source inspection, or "**Inferred**" when deduced from context.
- **Normative** statements (using "shall") describe required behaviour. In this specification, "shall" is used **only** when the behaviour is directly supported by implementation, tests, or explicit project documentation.
- **Recommendations** are clearly labelled and do not imply that the current system is broken.

## 6. Instructions for Reviewing the Current System

1. Start with `00_project_overview.md` for context
2. Read `02_architecture.md` for component relationships
3. Consult `05_cli_and_interfaces.md` for command-level behaviour
4. Use `03_module_specifications.md` to locate source files for any feature
5. Cross-reference with `13_traceability_matrix.md` to find tests and outputs

## 7. Instructions for Proposing a Change

1. Identify which documents are affected using the table below
2. Review those documents and the corresponding source files
3. Answer the questions in `12_change_management_playbook.md` for your change type
4. Record open decisions in `16_open_questions.md`
5. If the change affects architecture, create an ADR in `adr/`

## 8. Instructions for Implementing a Change

1. Follow the procedure in `12_change_management_playbook.md`
2. Write or update tests as described in `09_testing_and_quality_assurance.md`
3. Update affected specification documents
4. Run the full validation suite (see `11_build_release_and_operations.md`)

## 9. Instructions for Validating a Change

1. Run `cargo fmt --check`
2. Run `cargo clippy --all-targets --all-features -- -D warnings`
3. Run `cargo test --all`
4. Run `cargo build --release`
5. Review `git diff --stat` to confirm only intended files changed

## 10. Instructions for Releasing a Change

1. Confirm all tests pass on both Linux and macOS (CI matrix)
2. Update version in `Cargo.toml` if applicable
3. Update specification documents that describe changed behaviour
4. Tag the release commit

## 11. Specification Maintenance Rules

1. Every code change that alters behaviour must be accompanied by specification updates
2. Every new module must be documented in `03_module_specifications.md`
3. Every new CLI command must be documented in `05_cli_and_interfaces.md`
4. Every new scientific equation must be documented in `07_scientific_models_and_equations.md`
5. Every new test file must be recorded in `09_testing_and_quality_assurance.md`
6. Risks discovered during development must be added to `14_risk_and_technical_debt_register.md`

## 12. Quick-Reference Table

| I want to… | Read first |
|------------|-----------|
| Understand the project | `00`, `02`, `06` |
| Review a module | `03` |
| Review a scientific equation | `07`, `13` |
| Fix a bug | `12`, `13`, `14` |
| Add a new CLI command | `05`, `06`, `09`, `12`, `13` |
| Add a new equation | `07`, `08`, `09`, `12`, `13` |
| Add an equivalent-circuit model element | `07`, `03` (impedance), `12` |
| Add a new input format | `04`, `03` (data_file), `12` |
| Refactor without changing behaviour | `02`, `03`, `09`, `12` |
| Prepare a release | `09`, `11`, `12`, `13` |
| Understand a data structure | `04` |
| Debug a test failure | `09`, `10`, `13` |
| Check for known risks | `14` |

## 13. Project Owner Checklist

- [ ] Read `00_project_overview.md` for a non-technical summary
- [ ] Review `16_open_questions.md` for decisions needed
- [ ] Review `14_risk_and_technical_debt_register.md` for prioritised remediation
- [ ] Use `12_change_management_playbook.md` whenever modifying the system
- [ ] Keep this specification up to date with each release
