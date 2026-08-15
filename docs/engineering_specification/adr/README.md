# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the `rust_electroanalysis_cli` project.

## What Is an ADR?

An Architecture Decision Record is a short document that captures a significant architectural choice, the context in which it was made, the alternatives considered, and the consequences of the decision. ADRs provide a historical record that helps future maintainers understand **why** the system is designed the way it is.

## When to Write an ADR

Write an ADR when you make a decision that:
- Changes the architecture (module structure, dependency direction, data flow)
- Affects the public API or CLI interface in a breaking way
- Introduces a new external dependency with significant implications
- Changes how scientific equations are implemented across multiple modules
- Establishes a new convention or pattern that other parts of the codebase should follow
- Resolves a question documented in `16_open_questions.md`

You do **not** need an ADR for:
- Bug fixes
- Adding a new circuit element (follow `12_change_management_playbook.md` Section 4 instead)
- Routine refactoring that preserves behaviour
- Test additions

## How to Write an ADR

1. Copy the template below
2. Fill in each section
3. Name the file `NNNN-title-with-dashes.md` (e.g., `0001-use-levenberg-marquardt-for-eis-fitting.md`)
4. Use sequential numbers; check existing ADRs for the next number
5. Set status to "Proposed" initially; change to "Accepted" after review

## ADR Template

```markdown
# ADR-NNNN: [Title]

**Status**: [Proposed | Accepted | Deprecated | Superseded by ADR-NNNN]

**Date**: YYYY-MM-DD

**Author**: [Name / GitHub handle]

---

## Context

[Describe the problem or situation that motivated this decision. What constraints exist? What scientific or engineering requirements drive this?]

## Decision

[Clearly state what was decided. Use active voice: "We will use X for Y."]

## Alternatives Considered

| Alternative | Pros | Cons | Why Rejected |
|-------------|------|------|-------------|
| Alternative A | ... | ... | ... |
| Alternative B | ... | ... | ... |

## Scientific Implications

[How does this affect scientific correctness, numerical accuracy, or interpretability of results? Do equations change? Do parameter meanings change?]

## Software Implications

[How does this affect the codebase? New dependencies? Module restructuring? Breaking API changes?]

## Compatibility Implications

[Does this break backward compatibility? Do old config files, result files, or CLI invocations still work? Is a migration path needed?]

## Testing Requirements

[What new tests are needed? Which existing tests must be updated?]

## Consequences

### Positive
- [List benefits of this decision]

### Negative
- [List drawbacks, limitations, or risks introduced]

### Neutral
- [List side effects that are neither clearly good nor bad]
```
