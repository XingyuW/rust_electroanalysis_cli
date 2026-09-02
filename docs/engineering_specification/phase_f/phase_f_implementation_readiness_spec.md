# Phase F Implementation and Readiness Specification

## 1. Authority and precondition

This G2 candidate owns how approved contracts become software. It refines
`F-ARCH-003`, `F-ARCH-010..011`, `F-ARCH-015`, `F-ARCH-017`, and
`F-ARCH-021`. It grants no permission to implement before G3 and cannot invent
wire, scientific, operational, or owner-decision semantics.

## 2. Requirements

| ID | derived_from | Normative requirement | Adopted R11 clauses |
|---|---|---|---|
| <a id="F-IMPL-001"></a>`F-IMPL-001` | `F-ARCH-011,F-ARCH-017` | No implementation branch, checker source, production schema file, or implementation change may begin before exact G3 approval. Planning documents, reviews, and conformance fixtures are not implementation. | §§1, 7, 17–19 |
| <a id="F-IMPL-002"></a>`F-IMPL-002` | `F-ARCH-004,F-ARCH-017,F-WIRE-001..009,F-SCI-001..010,F-OPS-001..008` | Every release-relevant code behavior maps to approved child requirements and tests. No behavior is authorized merely because code implements it. | §§19, 53.8/53.12 inverse model |
| <a id="F-IMPL-003"></a>`F-IMPL-003` | `F-ARCH-010,F-ARCH-015` | Checker responsibilities, commands, argv, reports, stdout/stderr, exit codes, and fail-closed results are exact. The command/argv relationship has no shell reinterpretation or ambiguous default. | §7, §53.7 checker anchors |
| <a id="F-IMPL-004"></a>`F-IMPL-004` | `F-ARCH-010,F-ARCH-017` | Readiness requires two fresh-source builds with checker-local locked dependencies, recorded toolchain, clean isolated HOME/CARGO_HOME, closed environment whitelist/exclusions, no network except approved mode, and byte-identical binaries. | §7.1 |
| <a id="F-IMPL-005"></a>`F-IMPL-005` | `F-ARCH-010,F-ARCH-015,F-WIRE-005` | Build and readiness evidence bind source commit/tree, specification-bundle approval tag/manifest, Cargo.lock, toolchain, environment, command transcript, binary SHA/length, tests, and independent review. Readiness precedes enrollment. | §§7.1, 17–19 |
| <a id="F-IMPL-006"></a>`F-IMPL-006` | `F-ARCH-003,F-ARCH-017` | Required checks are `cargo fmt --all --check`, `cargo check --locked`, strict all-target/all-feature Clippy, the full locked test suite, Phase-E validation, Phase-D public-output regression, schema/KAT/traceability audits, and reproducibility comparison. | §53.14 baseline plus this row |
| <a id="F-IMPL-007"></a>`F-IMPL-007` | `F-ARCH-010,F-ARCH-021,F-ARCH-022` | Integration validation parses the authoritative R12 artifact graph, treats its mandatory per-edge binding obligations as the sole binding root, derives the G3 prerequisite closure and all downstream binding projections, proves exact upstream authority bindings, resolves REAL reviewer actor attestations from the graph-pinned bootstrap root through immutable historical proof IDs and the resolver-owned monotonic accepted-head checkpoint, evaluates historical cryptographic validity separately from current subject authorization, validates subject-derived actor identity, anti-alias and role/independence evidence plus migrated-review lifecycle/staleness and aggregate closure, rejects self-Git/future-object/hash cycles or implementation-readiness bypasses, and hands off only the exact independently reviewed SHA through the safe-publication procedure below. | §§17–19, R12 authority graph and conformance §3.2–§3.3 |

## 3. Safe publication handoff

Publication is a controlled handoff of an already independently reviewed
candidate; it is not an approval, review, implementation action, or authority
creation step. A fresh cumulative independent rereview must first report
`GO`, with P0=0 and P1=0, for the exact candidate SHA `REVIEWED_SHA`. The
remediation agent cannot perform or authorize this handoff.

The handoff operator must perform every step below in the canonical repository
checkout:

1. Confirm the review report names `REVIEWED_SHA` exactly and that the checkout
   is clean: `git status --porcelain=v1 --untracked-files=all` emits no bytes.
2. Confirm `git rev-parse HEAD` and
   `git rev-parse refs/heads/main` both equal `REVIEWED_SHA`.
3. Confirm the local tracking ref is safe to compare:
   `git rev-parse refs/remotes/origin/main` succeeds and
   `git merge-base --is-ancestor refs/remotes/origin/main REVIEWED_SHA`
   succeeds. A stale tracking ref is not live remote evidence.
4. Independently read the live remote ref with
   `git ls-remote --heads origin refs/heads/main`, parse exactly one
   `refs/heads/main` row, and record that SHA as `EXPECTED_OLD_SHA`. Require
   `EXPECTED_OLD_SHA` to equal the live value just read and require
   `git merge-base --is-ancestor EXPECTED_OLD_SHA REVIEWED_SHA` to succeed.
5. Publish only with an exact remote compare-and-swap lease and an
   already-proven fast-forward update:

   ```text
   git push --atomic \
     --force-with-lease=refs/heads/main:EXPECTED_OLD_SHA \
     origin REVIEWED_SHA:refs/heads/main
   ```

   `--force-with-lease` here is only the exact expected-old-value race guard;
   it is not permission to overwrite history. The server must reject a
   non-fast-forward update. Unconditional `--force`, `--force-if-includes`,
   ref deletion, a stale lease, or a plain push substituted for the exact
   compare-and-swap is prohibited.
6. After a successful command, fetch and re-read the remote main ref, then
   require all of `HEAD`, local `main`, local `origin/main`, and the live
   remote `main` SHA to equal `REVIEWED_SHA`; require the worktree to remain
   clean. Only this postcondition permits the existing review-start equality
   gate to run.

Any dirty state, missing or ambiguous live response, remote race, lease
failure, non-fast-forward result, authentication/network error, push error, or
post-publication mismatch is a fail-closed `NO-GO` handoff. Do not infer
success from a timeout or retry against a different expected SHA; preserve the
exact candidate, independently re-read live state, and obtain a fresh
independent rereview if the reviewed SHA or target changes. The conformance
self-test exercises a local bare-remote positive publication, dirty-worktree,
remote-race, non-fast-forward, and unavailable-live-state failures without
publishing the checked-in candidate.

## 4. Review and readiness gates

The G2 document review requires P0/P1=0. Build-environment, CLI/result,
reproducibility, or mapping ambiguity is P1. G2 approval only permits bundle
assembly; implementation begins only after G3. G4 requires real implementation
and reproducible evidence independently reviewed GO. The checked-in candidate
has no real approval objects, so readiness remains `NO` even though the
synthetic validator and graph audits pass.
