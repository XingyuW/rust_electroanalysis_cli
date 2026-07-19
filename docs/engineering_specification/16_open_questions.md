# 16 — Open Questions

**Identifier:** `DOC-16`  
**Status:** Requires project-owner input  
**Last Updated:** 2026-07-19

---

## How to Use This Document

Each open question represents something that **cannot be resolved from the repository alone**. The project owner must review each question and provide a decision. Once resolved, move the item to the relevant specification document and remove it from this list.

---

## Unresolved Questions

Code-answerable items from earlier revisions were moved to `DOC-12` (Change Management Playbook).  
This list now contains policy-level decisions that still require owner input.

### OQ-001: Crate Naming Discrepancy

**Evidence**: `src/lib.rs` L6 declares `extern crate self as rust_plots;`, but `Cargo.toml` names the crate `rust_electroanalysis_cli`. The README and CLI binary use the longer name.

**Question**: Should the internal alias `rust_plots` be removed/renamed to match the crate name?

**Impact**: Low. Only affects internal naming and potential confusion for new developers.

**Decision Required**: Yes — rename or document as intentional.

---

### OQ-002: Plotting Configuration Scope

**Evidence**: `plot_config.rs` contains ~200 configurable fields (lines, markers, colors, transforms, scientific notation, regression, axis scales, etc.) in `RawPlotStyle`. Some fields may be unused or only partially supported.

**Question**: Are all ~200 style fields actively used in all plot backends, or are some aspirational? Should unused fields be removed or documented as "reserved for future use"?

**Impact**: Medium. Large config surface increases maintenance burden and user confusion.

**Decision Required**: Yes — audit and prune or document.

---

### OQ-003: PINN Optimizer Status

**Evidence**: `src/impedance/pinn_optimizer.rs` exists but its relationship to the main LM-based fitting pipeline is unclear. It appears to be an experimental/alternative optimizer.

**Question**: Is the PINN optimizer functional? Should it be exposed as a CLI option, or is it research code that should be in a separate branch?

**Impact**: Medium. Unused code increases maintenance burden.

**Decision Required**: Yes — expose, remove, or document as experimental.

---

### OQ-007: Schema Version Evolution Strategy

**Evidence**: Schema handling is now heterogeneous: estimation config is v3 with in-loader migration from older versions; app config auto-migrates to v1; signal/health/mechanism configs reject unsupported versions; plotting/search/transient/calibration configs warn on mismatches; and emitted result schemas span v1 and v2 (for example health baseline and estimation outputs are v2).

**Question**: Should schema evolution policy be standardized across modules (reject vs auto-migrate vs warning-only), and should that policy be made explicit for both config and result artifacts?

**Impact**: High for long-term maintainability.

**Decision Required**: Yes — define migration policy before first breaking change.

---

### OQ-008: Windows Platform Support

**Evidence**: CI tests only Linux and macOS. No Windows-specific code exists, but plotters may require font configuration differences.

**Question**: Is Windows support a goal? If so, what testing infrastructure is needed?

**Impact**: Low. Current users appear to be on macOS/Linux.

**Decision Required**: Optional — state explicitly whether Windows is supported.

---

### OQ-010: CPE Initial Alpha Guess

**Evidence**: `impedance/fitting.rs` L110 clamps initial CPE α to [0.45, 0.98]. For systems with α far outside this range, the optimizer may converge slowly or to a local minimum.

**Question**: Should the α clamp range be configurable, or is [0.45, 0.98] sufficient for all expected use cases?

**Impact**: Low. Only affects convergence speed for unusual systems.

**Decision Required**: Optional.

---

### OQ-011: Logging Infrastructure

**Evidence**: `config/app.toml` has a `[logging]` section with `level = "info"`, but no actual logging implementation (no log crate dependency, no log statements in production code). The `logs/` directory is created but not used.

**Question**: Is logging intended to be implemented (using the `log` + `env_logger` crates, for example), or should this config be removed?

**Impact**: Low. Dead configuration.

**Decision Required**: Optional — implement logging or remove config.

---

## Summary

| ID | Priority | Category |
|----|----------|----------|
| OQ-001 | Low | Naming |
| OQ-002 | Medium | Config scope |
| OQ-003 | Medium | Unused code |
| OQ-007 | High | Architecture |
| OQ-008 | Low | Platform |
| OQ-010 | Low | Numerical |
| OQ-011 | Low | Infrastructure |
