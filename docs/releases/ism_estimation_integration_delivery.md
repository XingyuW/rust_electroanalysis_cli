# Compiled ISM Estimation Integration — Delivery Record

## Delivered milestone

The compiled ISM estimation integration is frozen and approved for delivery at
consumer commit `2ae647739bbc826e11ece6b20400fecf40324a2f` on branch
`test/compiled-estimation-final-evidence`.

Intended base: `13b1cf309c635e5794b661ee103d8d6ed73658c8`.

Final review: `docs/reviews/ism_estimation_integration_final_go.md`.

## Delivered capabilities

- Runtime custom input bindings with typed source, unit, missing, and duplicate errors.
- Compiled LegacyEquivalentV1 parity for EKF and UKF, including Nicolsky–Eisenman interferents.
- Reduced ISM V1 compiled simulation with stable state-ID truth and validation.
- Explicit activity-event and transduction-drive timing/provenance.
- Component-specific dynamic time constants and covariance handling.
- Honest backend/profile/model provenance in reports.
- Backward-compatible estimation configurations and historical artifacts.
- Absent-truth metrics represented as unavailable rather than fabricated zeros.

## Acceptance and validation record

The permanent integration matrix is in `tests/phase6_estimation.rs`; migration
fixtures are in `tests/fixtures/estimation_migration/`. R1–R4 remediation tests
were each run independently. The complete clean-checkout validation passed:

```text
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
cargo build --locked --release
```

Final result: **GO**. No P0, P1, or P2 findings remain within the approved
scope. The pinned external provider is `electrodata-io` at
`dbb6b7d063972114c4208980723e12c807ab199e`.

## Freeze metadata

Tag to be created:

```text
ism-estimation-integration-approved
```

The working tree must be clean after the documentation commit and tag are
created. Any pre-existing unrelated local change must be resolved separately
before release publication.
