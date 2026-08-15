# ISM Model Scientific Contract Final GO Review

**Review date:** 2026-08-08  
**Review decision:** GO — ready for model-core implementation  
**Reviewed consumer SHA:** `36aeff2bc28839a8094683cf60f74afe11a14641`  
**Provider SHA:** `dbb6b7d063972114c4208980723e12c807ab199e`

## Severity findings

### P0

None.

### P1

None. No scoped runtime covariance blocker was reproducible.

### P2

None.

### P3

None.

## Scoped covariance contract

- Missing fitted-parameter runtime covariance remains incomplete: status is
  `Partial`, `parameter_variance_v2` and `total_variance_v2` are `None`, and
  the missing source identifies `parameter:offset runtime covariance missing`.
- Missing estimated-state runtime covariance remains incomplete: status is
  `Partial`, `state_variance_v2` and `total_variance_v2` are `None`, and the
  missing source identifies `state:memory runtime covariance missing`.
- A deterministic quantity with covariance `[[1e-13]]` returns the typed
  `NonzeroCovarianceForDeterministicQuantity` error; it is not reported as
  variance `9e-13`.
- A fitted stochastic parameter with derivative `3` and covariance
  `[[1e-13]]` is accepted as positive covariance and reports parameter
  variance `9e-13` when all other sources are complete.
- Equivalent state cases preserve the same semantics: deterministic state
  covariance is rejected, while the estimated-state fixture accepts the
  positive covariance and reports `4e-13` for its derivative of `2`.
- Covariance zero/nonzero semantics use exact numeric comparisons. The
  absolute tolerances in the compiler are restricted to covariance symmetry
  and PSD numerical checks; reconstruction has its separate numerical
  tolerance.

## Regression preservation

The following named regressions passed: zero covariance row contradiction,
missing derivative coverage, explicit zero derivative coverage, Nernst
cross-covariance, fitted uncertainty validation, estimated-state uncertainty
validation, legacy migration, and the model-core architecture boundary.

## Validation

All required commands passed:

    cargo fmt --check
    cargo check --locked
    cargo clippy --locked --all-targets --all-features -- -D warnings
    cargo test --locked --all
    cargo build --locked --release
    git diff --check

Focused suites also passed: `model_core` (40 tests), `model_builtins` (19
tests), and `model_contracts` (5 tests). The complete locked suite passed with
zero failures. The worktree was clean before the branch merge.

## Architecture

The model-core boundary remains free of high-level CLI, runner, plotting,
health, mechanism, estimation, and results dependencies. Runtime covariance is
caller-supplied; schema-declared uncertainty is not silently converted into a
posterior diagonal covariance.

## Final decision

**GO — ready for model-core implementation**
