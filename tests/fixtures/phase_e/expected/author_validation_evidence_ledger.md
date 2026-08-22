# Phase E R2 author-side validation evidence

## Authority

| item | immutable identity |
| --- | --- |
| original approved plan tag | ism-mechanism-health-v1-e-plan-approved |
| R2 approved plan tag | ism-mechanism-health-v1-e-plan-approved-r2 |
| R2 plan commit | e65092088a28a2a9ed61364274dbd6ec46de5eb8 |
| R2 plan SHA-256 | e6e5195c7f56904afb06dfe937433f3498465fef1df191b8fb6856ee1ac792b6 |
| R2 plan Git blob | 45c1441ac4d6e20c5626b299fe5293b00ea444fb |
| implementation branch | codex/mhi-v1-e-independent-validation |
| fixture ledger | expected/phase_e_fixture_inventory.schema1.json |

## Traceability

The required registry is complete: E-R01 through E-R18, E-AC01 through
E-AC18, and E-T01 through E-T30.  The closed fixture inventory supplies every
fixture-to-test mapping and its mutation case and expected oracle identifier.
Traceability totals are 18 requirements, 18 acceptance criteria, and 30
test/evidence records.

| requirement group | acceptance criterion | tests |
| --- | --- | --- |
| E-R01 | E-AC01 | E-T01, E-T02 |
| E-R02 | E-AC02 | E-T03, E-T04 |
| E-R03 | E-AC03 | E-T05, E-T06 |
| E-R04 | E-AC04 | E-T07, E-T08 |
| E-R05 | E-AC05 | E-T09 |
| E-R06 | E-AC06 | E-T10, E-T11 |
| E-R07 | E-AC07 | E-T12 |
| E-R08 | E-AC08 | E-T13, E-T14 |
| E-R09 | E-AC09 | E-T15, E-T16 |
| E-R10 | E-AC10 | E-T17 |
| E-R11 | E-AC11 | E-T18 |
| E-R12 | E-AC12 | E-T19 |
| E-R13 | E-AC13 | E-T20, E-T21 |
| E-R14 | E-AC14 | E-T22, E-T23 |
| E-R15 | E-AC15 | E-T24, E-T25 |
| E-R16 | E-AC16 | E-T26, E-T27 |
| E-R17 | E-AC17 | E-T28, E-T29 |
| E-R18 | E-AC18 | E-T30 |

## Dependency and lock audit

The only new direct cryptographic dependency is
`ed25519-dalek = { version = "=2.2.0", default-features = false }`.
The lock delta is limited to six new packages:
`curve25519-dalek 4.1.3`, `curve25519-dalek-derive 0.1.1`,
`ed25519 2.2.3`, `ed25519-dalek 2.2.0`, `fiat-crypto 0.2.9`, and
`signature 2.2.0`.  Existing locked package entries are unchanged.

## Required author-side command registry

```text
git diff --check
cargo fmt --all --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --test phase_e_validation
cargo test --locked --all
cargo test --doc --locked
cargo build --locked --release
```

The exact command results, platform, and cumulative-diff audit are recorded at
candidate freeze by the author workflow; this document contains no candidate
commit identity and no approval decision.

## Local author command results

Host: Darwin arm64.

| command or evidence group | result |
| --- | --- |
| git diff --check | PASS |
| cargo fmt --all --check | PASS |
| cargo check --locked | PASS |
| cargo clippy --locked --all-targets --all-features -- -D warnings | PASS; zero diagnostics |
| Phase-E integration, approval, reader, and publication suites | PASS |
| Phase-D targeted and historical compatibility coverage | PASS |
| cargo test --locked --all, run 1 | PASS |
| cargo test --locked --all, run 2 | PASS |
| cargo test --doc --locked | PASS |
| cargo build --locked --release | PASS |
| production trust is UNPROVISIONED with zero roots | PASS |
| production physical request fails before dataset/scoring | PASS: PhysicalApprovalTrustNotProvisioned |
| deterministic software golden bundle | PASS |

## Author-side dispositions

| severity | disposition |
| --- | --- |
| P0 | no unresolved author-side issue |
| P1 | no unresolved author-side issue |
| P2 | no unresolved author-side issue |

## Post-freeze external fields

Independent scientific review: PENDING_POST_FREEZE  
Independent architecture review: PENDING_POST_FREEZE  
Independent security review: PENDING_POST_FREEZE  
Independent compatibility review: PENDING_POST_FREEZE  
Other-platform exact-commit validation: PENDING_POST_FREEZE  
Implementation approval: PENDING_POST_FREEZE  
Integration approval: PENDING_POST_FREEZE
