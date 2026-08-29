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
| <a id="F-IMPL-007"></a>`F-IMPL-007` | `F-ARCH-010,F-ARCH-021` | Integration validation follows the approved authority DAG and proves each downstream artifact binds its exact upstream authority, with no self-Git/future-object cycle or bypass. | §§17–19, 53.6 |

## 3. Review and readiness gates

The G2 document review requires P0/P1=0. Build-environment, CLI/result,
reproducibility, or mapping ambiguity is P1. G2 approval only permits bundle
assembly; implementation begins only after G3. G4 requires real implementation
and reproducible evidence independently reviewed GO.
