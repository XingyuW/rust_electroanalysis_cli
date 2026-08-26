# MHI V1 Phase E — Independent Scientific Validation and Compatibility Certification

**Status:** R5 planning-only amendment; implementation is not authorized by this
document.

**Repository:** `/Users/xingyuwang/ProjectOngoing/rust_electroanalysis_cli`

**Phase-E initialization baseline:** current synchronized `main`

**`PHASE_E_BASELINE_MAIN_SHA`:** `6b76258ff2e8ff71a1b8a68248b47cf224141d73`

**Phase-D provenance release tag:** `ism-mechanism-health-v1-d-complete`

**Phase-D provenance release SHA:** `2316fb1d076d47ac14d3b3c10c55654feb1ffb54`

**Planning workflow:** documentation-only planning commits directly to `main`; no
planning or review branch exists.

**Temporary implementation branch after plan approval only:**
`codex/mhi-v1-e-independent-validation`

**Milestone name:** MHI V1 Phase E

This document defines the next scientific/software milestone after Phase D. It
does not authorize changes to Rust source, configuration, schemas, fixtures, or
existing documentation other than this plan.

## R5 amendment — correct R4 post-freeze E-T30 integration inconsistency

This section is the current Phase-E plan authority. It is a narrow,
documentation-only R5 plan amendment. It supersedes any conflicting R4
normative instruction retained below as historical record. R4 history,
commits, review evidence, and immutable tags are not rewritten.

### R5 defect classification and failed-integration evidence

R5 corrects the R4 post-freeze exception after actual integration validation
disproved its premise. R4 assumed that the frozen implementation SHA did not
need to absorb the R4 plan bytes, that its implementation reviews could carry
forward unchanged, and that integrated `main` would pass the complete locked
suite. The frozen E-T30 implementation test instead contains a literal
assertion that the plan file has the R3 SHA-256
`131dc77dc656952469c77a816a36c847d4f38a018f922577441884396009ed4a`. With R4
plan bytes on integrated `main`, the plan hashes to
`f737233661fd53189d121d53fdea1394f63f227eaa7b584d911ac1024dbaf4e0`, and
E-T30 fails. The observed integration result was 37/38 Phase-E tests and
779 passed / 1 failed in each full-suite run; only E-T30 failed.

Classify this as `P1-R5-001`: the R4 post-freeze exception is incompatible
with locked E-T30 plan-byte authority. This is a plan/test-authority
synchronization defect, not a scientific, security-authority,
publication-state-machine, compatibility-behavior, or Linux-platform defect.
R5 corrects the workflow and does not weaken, skip, or make E-T30
revision-agnostic.

`P1-R5-001 = CLOSED`. It is retained as completed historical defect evidence;
this remediation addresses only the six remaining R5 plan-specification P1s.

The failed local integration commit is diagnostic history only:

```text
b9f1987b392645f59e4adf5962adcac2578bac04
```

It was never pushed, never integrated remotely, never tagged as integrated,
failed mandatory E-T30, and is not release authority. The original working
copy containing it is failed-integration evidence and must remain untouched:
do not push, amend, reset, rebase, revert, force-move `main`, or create an
integrated tag for that commit. R5 recovery starts from a fresh clone of live
remote authority and does not import that local merge.

### R5 remote authority and immutable R4 history

Before recovery, live remote authority must be checked with `git ls-remote`.
The expected authority is:

```text
refs/heads/main = b43c96baca14b1618ce3106fe88c1a09f7d49e0c
refs/heads/codex/mhi-v1-e-independent-validation = 3076b102ed72b46b6c7c5b75f0360ff6f957968b
```

The existing immutable tags must remain unchanged:

```text
ism-mechanism-health-v1-e-plan-approved-r4
ism-mechanism-health-v1-e-implementation-approved
```

The existing historical R4 implementation-approval tag has annotated tag object
`ee3ac21330d53f8d056cab0db6f9ebcf5e2789ff` and peeled target
`3076b102ed72b46b6c7c5b75f0360ff6f957968b`. It remains an immutable
historical approval-evidence record for that `REVIEW_SHA` under the attempted
R4 workflow. Because mandatory integrated E-T30 failed, it did not lead to a
valid integrated release. It is superseded for final integration authority,
not mutated: it must never be deleted, replaced, or retargeted.

The R4 plan tag and all predecessor plan tags are likewise immutable historical
authority. This supersession does not rewrite the R4 review record or claim
that the failed local merge was remotely integrated.

### R5 plan/test authority synchronization

The exact latest independently approved Phase-E plan bytes are authoritative
over locked E-T30. The final implementation candidate must contain those exact
bytes whenever E-T30 validates the plan file. After R5 plan approval,
`tests/phase_e_validation.rs` must be updated from the stale R3 plan SHA-256
to the exact SHA-256 of the approved R5 plan, and wording such as `approved
R3 plan` must identify the exact current R5 plan. This is a test-authority
synchronization change; it does not weaken E-T30.

E-T30 must continue to assert one exact independently approved plan byte
sequence. It must not accept R3/R4/R5 alternatives, any plan hash, a prefix,
existence-only evidence, arbitrary dynamically trusted bytes, or a disabled
assertion. The test adaptation is implementation work authorized only after
this R5 plan is independently approved; this R5 authoring phase modifies no
test.

### R5 supersession of the R4 post-freeze exception

The R4 rule that
`3076b102ed72b46b6c7c5b75f0360ff6f957968b` need not absorb the latest plan
documentation is explicitly superseded. No post-freeze carry-forward
exception applies in R5. The required authority chain is:

```text
latest approved R5 main
    -> normal --no-ff merge into existing implementation branch
    -> exact E-T30 plan-authority adaptation
    -> new frozen implementation SHA
    -> fresh four-role reviews
```

**HISTORICAL R4 EVIDENCE — SUPERSEDED UNDER R5:** the prior R4 review marked
carry-forward as permitted only under the then-reviewed assumption that no
implementation/test synchronization was required. Empirical E-T30 failure
falsified that premise. Under current R5, carry-forward is not a decision
value and the four implementation reviews are fresh. The earlier 3076b102
reviews remain useful historical evidence, but cannot be reused as final
approval evidence after the R5 implementation/test synchronization because
the committed tree changes.

### R5 plan approval and tag contract

After this plan is independently reviewed and receives `P0=0` and `P1=0`,
create the new immutable tag
`ism-mechanism-health-v1-e-plan-approved-r5` at the exact reviewed
`R5_PLAN_REVIEW_SHA`. The existing tags
`ism-mechanism-health-v1-e-plan-approved`,
`ism-mechanism-health-v1-e-plan-approved-r2`,
`ism-mechanism-health-v1-e-plan-approved-r3`, and
`ism-mechanism-health-v1-e-plan-approved-r4` remain unchanged. The exact
ASCII-sorted R5 plan-tag chain is:

```text
ism-mechanism-health-v1-e-plan-approved,ism-mechanism-health-v1-e-plan-approved-r2,ism-mechanism-health-v1-e-plan-approved-r3,ism-mechanism-health-v1-e-plan-approved-r4,ism-mechanism-health-v1-e-plan-approved-r5
```

No R5 plan tag is created before independent plan-review GO.

### R5 implementation and integration sequence

The complete current R5 lifecycle is one chronology. It is the only normative
workflow in this plan:

1. Author one R5 plan-only forward commit on `main`.
2. Push `main` normally.
3. Freeze the exact remote R5 plan commit and exact plan bytes.
4. Perform the independent R5 plan review.
5. If and only if `P0=0` and `P1=0`, create
   `ism-mechanism-health-v1-e-plan-approved-r5` at the exact reviewed plan
   SHA.
6. Merge that exact approved R5 plan SHA into the existing
   `codex/mhi-v1-e-independent-validation` branch with `--no-ff`.
7. Synchronize E-T30 to the exact approved R5 plan SHA-256.
8. Refresh only mechanically required author evidence and fixture authority.
9. Validate the implementation candidate.
10. Commit and push the implementation branch normally.
11. Freeze the new exact remote implementation `REVIEW_SHA`.
12. Obtain fresh independent scientific, architecture, security, and
    compatibility reviews for that exact SHA.
13. Perform fresh macOS exact-SHA validation on that same SHA.
14. Create the exact `ism-mechanism-health-v1-e-implementation-approved-r5`
    tag with the exact V2 body.
15. Merge that exact newly reviewed implementation SHA into `main` with
    normal `--no-ff` history.
16. Require integrated validation, including E-T30, to pass completely.
17. Push `main` normally.
18. Create `ism-mechanism-health-v1-e-implementation-integrated` only after
    integrated validation passes.
19. Verify the remote state and clean up the implementation branch.

No step permits R4 review-evidence carry-forward or skipping the approved R5
plan merge. The old R4 implementation-approval tag remains immutable history;
it never authorizes R5 integration.

The existing historical R4 `ism-mechanism-health-v1-e-implementation-approved`
tag never moves. The new `-r5` implementation-approval tag is required for R5
integration; the old R4 tag cannot authorize it. The integrated tag name was
never created and remains reserved for the first successful R5 integrated
`main`.

### PhaseEImplementationApprovalV2 canonical record

For the final R5 implementation approval tag only, define
`PhaseEImplementationApprovalV2`, replacing V1 as the final approval-record
format. The record is UTF-8 in the ASCII subset, uses LF line endings, has
exactly one final LF, no blank lines, no trailing whitespace, no quoting, no
escaping, fixed field order, no optional fields, no duplicate fields, and no
additional fields. No cryptographic reviewer signatures are introduced.

The following is the complete field order. Angle-bracket values are
parameters filled only from the exact reviewed implementation and validation
results; their lexical forms are closed by the rules that follow. The R5 plan
does not contain its own final SHA-256: E-T30 computes that value over the
final independently approved plan bytes.

```text
PhaseEImplementationApprovalV2
format_version=2
review_sha=<40 lowercase hexadecimal characters>
plan_review_sha=<40 lowercase hexadecimal characters>
plan_sha256=<64 lowercase hexadecimal characters>
plan_git_blob=<40 lowercase hexadecimal characters>
plan_tags=ism-mechanism-health-v1-e-plan-approved,ism-mechanism-health-v1-e-plan-approved-r2,ism-mechanism-health-v1-e-plan-approved-r3,ism-mechanism-health-v1-e-plan-approved-r4,ism-mechanism-health-v1-e-plan-approved-r5
supersedes_approval_tag=ism-mechanism-health-v1-e-implementation-approved
supersession_reason=R4_INTEGRATION_ET30_PLAN_HASH_MISMATCH
scientific_decision=GO
architecture_decision=GO
security_decision=GO
compatibility_decision=GO
p0_count=0
p1_count=0
p2_arch=P2-ARCH-001:accepted_non_blocking
p2_compat=P2-COMPAT-E-T25:accepted_non_blocking
macos_result=PASS
macos_uname=Darwin
macos_arch=<ASCII architecture token>
macos_product_version=<PRODUCT_VERSION>
macos_build_version=<BUILD_VERSION>
rustc_version=<SEMVER_CORE>
cargo_version=<SEMVER_CORE>
diff_worktree=PASS
diff_approved_main=PASS
diff_r5_delta=PASS
fmt=PASS
check=PASS
clippy=PASS
clippy_diagnostics=0
full_suite_run1=PASS
full_suite_run2=PASS
phase_e=<nonnegative decimal integer>/<positive decimal integer>
approval_unit=<nonnegative decimal integer>/<positive decimal integer>
internal_et29_kat=<nonnegative decimal integer>/<positive decimal integer>
compile_fail_authority=<nonnegative decimal integer>/<positive decimal integer>
et04=PASS
et22=PASS
et23=PASS
et29=PASS
et30=PASS
et29_crypto=16/16
et29_substantive=40/40
public_authority_bypass_paths=0
production_trust=UNPROVISIONED
phase_d=<nonnegative decimal integer>/<positive decimal integer>
determinism_files=<nonnegative decimal integer>/<positive decimal integer>
determinism_manifest_records=<nonnegative decimal integer>/<positive decimal integer>
determinism_two_runs=PASS
determinism_golden=PASS
linux_phase_e_gate=NOT_REQUIRED_R5
approval_decision=GO
```

The first two lines are literal. SHA fields are exactly lowercase hexadecimal
of the stated length. Fixed tokens are exactly the values shown. The record
has no optional or extra fields. Every valid approval record has literal
`p0_count=0` and `p1_count=0`.

Every numeric field uses `CANONICAL_DECIMAL` unless it is a SHA field or a
ratio field governed by `CANONICAL_RATIO`.

The closed numeric grammars are:

```text
CANONICAL_DECIMAL = "0" | [1-9][0-9]*
CANONICAL_RATIO = N "/" D
```

For `CANONICAL_RATIO`, `N` and `D` are canonical decimals, `D > 0`, and
`0 <= N <= D`. A required all-pass test-set ratio must additionally have
`N = D`. Thus `00`, `01`, `00038`, `+1`, `-0`, `1 `, and ` 1` are invalid.

The exact platform and compiler field sources are:

| Field | Exact source and normalization | Closed grammar or required value |
|---|---|---|
| `macos_uname` | `uname -s`; remove exactly one trailing LF | literal `Darwin` |
| `macos_arch` | `uname -m`; remove exactly one trailing LF | `[A-Za-z0-9._-]+`; current value `arm64` |
| `macos_product_version` | `sw_vers -productVersion`; remove exactly one trailing LF | `PRODUCT_VERSION = VERSION_COMPONENT ("." VERSION_COMPONENT)*` |
| `macos_build_version` | `sw_vers -buildVersion`; remove exactly one trailing LF | `[A-Za-z0-9._-]+`; current example `25G83` |
| `rustc_version` | `rustc --version`; parse ASCII whitespace, require first token `rustc`, reject fewer than two tokens, and use the second token exactly | `SEMVER_CORE = VERSION_COMPONENT "." VERSION_COMPONENT "." VERSION_COMPONENT`; current value `1.97.0` |
| `cargo_version` | `cargo --version`; parse ASCII whitespace, require first token `cargo`, reject fewer than two tokens, and use the second token exactly | same `SEMVER_CORE`; current value `1.97.0` |

`VERSION_COMPONENT` is `0` or `[1-9][0-9]*`. The macOS product and build
values are separate fields; no parentheses, underscore conversion, or other
normalization is permitted. Compiler commit/date text is not recorded in the
two compiler fields.

The exact diff-field semantics are:

| Field | Exact command | PASS condition |
|---|---|---|
| `diff_worktree` | `git diff --check` in the final frozen implementation candidate worktree | exit status `0` |
| `diff_approved_main` | `git diff --check <R5_PLAN_REVIEW_SHA>..<REVIEW_SHA>` | exit status `0`; the left SHA is the peeled target of `ism-mechanism-health-v1-e-plan-approved-r5` |
| `diff_r5_delta` | `git diff --check 3076b102ed72b46b6c7c5b75f0360ff6f957968b..<REVIEW_SHA>` | exit status `0`; the left SHA is the pre-R5 implementation authority |

`diff_security_remediation` is obsolete and is not a V2 field.

The full-suite fields are exact command-result fields, not aggregate test
counts. `full_suite_run1` and `full_suite_run2` are each `PASS` if and only if
the corresponding command exits `0`:

```text
cargo test --locked --all
cargo test --locked --all
```

Detailed counts remain in the independent validation report.

The exact approval-unit and internal-KAT commands are frozen as:

```text
cargo test --locked --lib phase_e_physical_claim_requires_dual_signature_embedded_trust_and_power
cargo test --locked --lib mhi_validation::approval_kat::phase_e_physical_claim_requires_dual_signature_embedded_trust_and_power
```

The first filter must select exactly `2/2`; the second must select exactly
`1/1`. These filters are derived from `cargo test --locked --lib -- --list`
on the implementation authority and are not replaced by “run approval tests”.

`approval_decision=GO` is valid if and only if all of the following hold:

- `scientific_decision=GO`, `architecture_decision=GO`,
  `security_decision=GO`, and `compatibility_decision=GO`;
- `p0_count=0` and `p1_count=0`;
- both P2 fields are complete and equal their accepted dispositions;
- `macos_result=PASS` and every required static, diff, test, security,
  compatibility, and determinism field is `PASS` or its exact all-pass ratio;
- E-T30 is `PASS`, production trust/state wording is accurate, and the exact
  R5 plan authority fields match the approved R5 tag; and
- no self-approval or third review verdict exists.

Any nonzero P0 or P1 makes the V2 record invalid for approval, regardless of
the other fields.

The complete dynamic-field source table is:

| Field | Authoritative source |
|---|---|
| `review_sha` | `git ls-remote --heads origin refs/heads/codex/mhi-v1-e-independent-validation`; exact live remote 40-lowercase-hex SHA after freeze |
| `plan_review_sha` | peeled target of `ism-mechanism-health-v1-e-plan-approved-r5` |
| `plan_sha256` | SHA-256 of the exact bytes of `docs/engineering_specification/next_milestone_plan.md` at `plan_review_sha` |
| `plan_git_blob` | Git blob object ID of that exact plan file at `plan_review_sha` |
| `plan_tags` | fixed ASCII-sorted R5 chain shown in the record |
| `scientific_decision` | fresh independent scientific review of the exact `REVIEW_SHA`; required `GO` |
| `architecture_decision` | fresh independent architecture review of the exact `REVIEW_SHA`; required `GO` |
| `security_decision` | fresh independent security review of the exact `REVIEW_SHA`; required `GO` |
| `compatibility_decision` | fresh independent compatibility review of the exact `REVIEW_SHA`; required `GO` |
| `p0_count`, `p1_count` | findings from those four exact-SHA reviews; valid R5 approval records require literal `0` for both |
| `p2_arch`, `p2_compat` | the two retained accepted dispositions; no other P2 value is valid |
| `macos_result` | fresh independent macOS validation on the exact `REVIEW_SHA`; required `PASS` |
| `fmt` | `cargo fmt --all --check` exits `0` |
| `check` | `cargo check --locked` exits `0` |
| `clippy` | `cargo clippy --locked --all-targets --all-features -- -D warnings` exits `0` |
| `clippy_diagnostics` | literal `0` for approval |
| `phase_e` | `cargo test --locked --test phase_e_validation`; required `38/38` |
| `approval_unit` | exact first filter above; required `2/2` |
| `internal_et29_kat` | exact second filter above; required `1/1` |
| `compile_fail_authority` | `cargo test --doc --locked` and its exact mapped compile-fail authority set; required `4/4` |
| `et04`, `et22`, `et23`, `et29`, `et30` | named/mapped tests in the Phase-E validation registry; each required `PASS` |
| `et29_crypto` | fixed required result `16/16` |
| `et29_substantive` | fixed required result `40/40` |
| `public_authority_bypass_paths` | exact security scan result; required `0` |
| `production_trust` | embedded production trust state; required literal `UNPROVISIONED` |
| `phase_d` | exact Phase-D regression command/filter; required `73/73` |
| determinism fields | canonical two-run Phase-E deterministic route; required values are `9/9`, `8/8`, `PASS`, `PASS` |

No V2 parameter may be filled by human guesswork. The tag body must be
byte-equal to the instantiated canonical V2 record.

### R5 author-evidence synchronization

After R5 approval and the normal merge into the implementation branch, inspect
`tests/fixtures/phase_e/expected/author_validation_evidence_ledger.md` and
update only fields required to reflect the latest approved plan tag chain
through R5, latest plan authority, exact required command results, and corrected
E-T30 plan-byte authority. If the ledger change mechanically requires fixture
inventory or hash updates, update only those exact authority values. The
ledger must remain author-side evidence: no candidate SHA, independent review,
approval, or fabricated GO may be added.

### Preserved scientific, security, compatibility, and platform contracts

R5 changes no runtime scientific, security, publication, or public
compatibility behavior. Preserve `SCI-P1-001`, `P0-ARCH-001`, `P1-ARCH-002`,
`P1-ARCH-003`, `P1-SEC-001`, `P1-SEC-002`, `SEC-P0-001`, and `COMP-P1-001`
closures; `PUBLIC_AUTHORITY_BYPASS_PATHS=0`; `UNPROVISIONED` production
trust; E-T29 crypto `16/16`; E-T29 substantive `40/40`; the publication state
machine; Wilson mathematics; domain semantics; Phase B/C/D behavior; schemas;
and dependencies.

The R3/R4 macOS-only contract remains in force. macOS is the only
MHI V1 Phase-E supported and release-validated platform. Linux is
`DEFERRED / UNSUPPORTED`, non-gating, and has zero mandatory Linux
requirements. Retain the 15 deferred Linux implementation markers and one
historical/evidence marker. Do not fix either retained non-blocking P2 in this
narrow recovery unless independently required:

```text
P2-ARCH-001: non-UTF-8 early-return DIR* leak; accepted_non_blocking
P2-COMPAT-E-T25: permanent E-T25 coverage weakness; accepted_non_blocking
```

### R5 plan-only scope and consistency audit

This R5 authoring task modifies only
`docs/engineering_specification/next_milestone_plan.md`. It must not yet
modify `src/`, `tests/`, fixtures, `Cargo.toml`, `Cargo.lock`, `.github/`, or
the implementation branch. Implementation adaptation starts only after R5
plan approval.

The final R5 plan is internally audited to require:

```text
LATEST_PLAN_REQUIRED_IN_FINAL_IMPLEMENTATION=YES
ET30_EXACT_LATEST_PLAN_HASH_REQUIRED=YES
R4_POST_FREEZE_EXCEPTION=SUPERSEDED
R4_CARRY_FORWARD_FOR_FINAL_APPROVAL=NO
FRESH_IMPLEMENTATION_SHA_REQUIRED=YES
FRESH_FOUR_ROLE_REVIEWS_REQUIRED=YES
FRESH_MACOS_VALIDATION_REQUIRED=YES
OLD_IMPLEMENTATION_APPROVED_TAG_MOVED=NO
NEW_R5_PLAN_TAG_REQUIRED=YES
NEW_R5_IMPLEMENTATION_APPROVED_TAG_REQUIRED=YES
INTEGRATED_TAG_REMAINS_UNUSED_UNTIL_SUCCESS=YES
PHASE_E_IMPLEMENTATION_APPROVAL_V2_DEFINED=YES
WIRE_GRAMMAR_AMBIGUITY_COUNT=0
SCIENTIFIC_BEHAVIOR_CHANGED=NO
SECURITY_BEHAVIOR_CHANGED=NO
PUBLICATION_BEHAVIOR_CHANGED=NO
COMPATIBILITY_BEHAVIOR_CHANGED=NO
ET30_PLAN_AUTHORITY_SYNCHRONIZATION=YES
MANDATORY_LINUX_REQUIREMENTS=0
```

There are zero normative contradictions in the current R5 authority. Any
contrary R4 sentence retained below is historical R4 record and is superseded
by this section for all current approval and integration decisions.

### R5 commit, push, and review handoff

Before committing, run `git diff --check`. Create one forward plan commit with
subject `docs(plan): close R5 approval-record and workflow gaps` and push
`main` normally; no force-push is permitted. The chronology is AUTHOR PLAN →
PUSH PLAN → FREEZE PLAN → INDEPENDENT PLAN REVIEW → PLAN-APPROVED-R5 TAG →
IMPLEMENTATION. Do not describe review as preceding the plan commit or push.
After the push, record:

```text
R5_PLAN_REVIEW_SHA=<exact pushed plan commit SHA>
R5_PLAN_SHA256=<SHA-256 of the exact plan file bytes>
R5_PLAN_GIT_BLOB=<Git blob ID of the exact plan file>
```

Verify local `main`, remote `main`, and `R5_PLAN_REVIEW_SHA` are identical;
verify the remote implementation branch remains at
`3076b102ed72b46b6c7c5b75f0360ff6f957968b`; verify the working tree is clean;
and verify no R5 plan tag exists before independent plan-review GO. This
handoff creates no R5 approval tag, modifies no implementation, and does not
claim approval or integration success. The plan SHA-256 is computed externally
after the final reviewed plan is frozen; it is never inserted into this plan
as a required literal self-reference.

> **HISTORICAL / SUPERSEDED UNDER R5 — R4 evidence only.** The following
> section is retained as immutable historical record. Its V1 record, old
> implementation-approval tag, carry-forward predicate, and no-plan-merge
> exception are not current R5 requirements and cannot authorize R5 approval
> or integration.

## R4 amendment — simplified post-freeze review evidence

This is a narrow, documentation-only Phase-E R4 amendment. R4 changes only the
post-freeze review-evidence packaging mechanism. It does not change
implementation behavior, scientific requirements, security requirements,
publication requirements, compatibility requirements, the R3 macOS-only
platform scope, P0/P1 severity rules, the requirement for four fresh
independent role reviews, the final macOS exact-SHA validation requirement, the
requirement that all four role decisions be `GO`, or complete P2 disposition.

R4 exists because the R3 evidence mechanism was operationally unsatisfiable:
independently authorized cryptographic reviewer identities do not exist,
`PhaseEReviewAttestationV1` had no exact frozen serialization, and mandatory
server-side remote tag protection is not established. R4 does not fabricate
those controls.

### R4 authority and completed implementation evidence

The R3 plan remains the predecessor authority. The current R3 plan and the
completed implementation-review authority carried forward into R4 are:

```text
R3 plan review SHA: 54a070edf1e2e7bc73731d1b5166816e37b0a8ae
R3 plan tag: ism-mechanism-health-v1-e-plan-approved-r3
R3 plan SHA-256: 131dc77dc656952469c77a816a36c847d4f38a018f922577441884396009ed4a
R3 plan Git blob: a4d32891bab8374bd9d20f80c38ff76e034212fc

IMPLEMENTATION_REVIEW_SHA=3076b102ed72b46b6c7c5b75f0360ff6f957968b
implementation branch: codex/mhi-v1-e-independent-validation
```

The earlier pre-R3 implementation candidate mentioned in the R3 historical
section is historical only. R4 does not modify, merge, or re-review the
implementation branch. The completed exact-SHA evidence carried into the R4
plan review is:

```text
SCIENTIFIC: GO; P0=0; P1=0
ARCHITECTURE: GO; P0=0; P1=0
SECURITY: GO; P0=0; P1=0
COMPATIBILITY: GO; P0=0; P1=0
FINAL_MACOS_EXACT_SHA_VALIDATION: PASS
MACOS_SAME_SHA_EVIDENCE: PASS
```

The retained P2 dispositions are exactly:

```text
P2-ARCH-001: non-UTF-8 early-return DIR* resource leak; accepted_non_blocking
P2-COMPAT-E-T25: permanent E-T25 coverage weakness; accepted_non_blocking
```

### R4 plan-tag contract

The exact applicable immutable Phase-E plan tags are sorted by raw ASCII bytes
and frozen as this value for the R4 approval record:

```text
ism-mechanism-health-v1-e-plan-approved,ism-mechanism-health-v1-e-plan-approved-r2,ism-mechanism-health-v1-e-plan-approved-r3,ism-mechanism-health-v1-e-plan-approved-r4
```

R4 authority is bound externally by the immutable
`ism-mechanism-health-v1-e-plan-approved-r4` tag at the exact pushed R4 plan
review SHA. This planning task creates no implementation-approval tag.

### R4 review-independence contract and carry-forward predicate

The scientific, architecture, security, and compatibility reviews remain four
independent reviews. The implementation author may not self-approve any role.
A reviewer who remediates a blocking defect may not be the final reviewer for
that role on the remediated SHA. Reviewer independence is a procedural review
requirement; cryptographic reviewer identity and cryptographic proof of reviewer
identity are not required.

The already completed exact-SHA evidence may carry forward only when the
independent R4 plan review explicitly confirms every condition below:

1. `REVIEW_SHA` is exactly
   `3076b102ed72b46b6c7c5b75f0360ff6f957968b`.
2. All four completed reviews targeted that exact SHA.
3. All four completed reviews have `P0=0` and `P1=0`.
4. All four canonical decisions are `GO`.
5. Both retained P2 debts have complete dispositions.
6. Final macOS exact-SHA validation targeted that SHA and passed.
7. R4 changes no implementation or review criterion.
8. No repository implementation byte changed after those reviews.

The required R4 plan-review output is exactly
`REVIEW_EVIDENCE_CARRY_FORWARD_VALID = yes/no`. Only `yes` permits reuse. If
the output is `no`, four fresh independent role reviews are required for the
exact implementation SHA.

### R4 approval-evidence tag and exact wire format

R4 replaces the former R3 four-role cryptographic attestation mechanism with
one annotated Git tag named
`ism-mechanism-health-v1-e-implementation-approved`. This single tag is both
the immutable implementation-approval marker and the canonical external
post-freeze review-evidence record. It is created by the project owner/release
coordinator and is not represented as cryptographically signed by the four
reviewers. No reviewer signature, reviewer key, or server-enforced tag
protection is claimed.

The tag must target exactly
`3076b102ed72b46b6c7c5b75f0360ff6f957968b` for the completed evidence named
above. Its message body is the exact `PhaseEImplementationApprovalV1` record
below. The record is UTF-8, uses only the ASCII subset, has LF line endings,
has exactly one final LF byte, contains no blank lines, no trailing spaces, no
optional fields, and no additional fields. The field order and closed values
are frozen exactly; no quoting or escaping is used.

```text
PhaseEImplementationApprovalV1
format_version=1
review_sha=3076b102ed72b46b6c7c5b75f0360ff6f957968b
plan_tags=ism-mechanism-health-v1-e-plan-approved,ism-mechanism-health-v1-e-plan-approved-r2,ism-mechanism-health-v1-e-plan-approved-r3,ism-mechanism-health-v1-e-plan-approved-r4
scientific_decision=GO
architecture_decision=GO
security_decision=GO
compatibility_decision=GO
p0_count=0
p1_count=0
p2_arch=P2-ARCH-001:accepted_non_blocking
p2_compat=P2-COMPAT-E-T25:accepted_non_blocking
macos_result=PASS
macos_uname=Darwin
macos_arch=arm64
macos_version=26.6.2(25G83)
rustc=1.97.0
cargo=1.97.0
diff_worktree=PASS
diff_approved_main=PASS
diff_security_remediation=PASS
fmt=PASS
check=PASS
clippy=PASS
clippy_diagnostics=0
full_suite_run1_passed=780
full_suite_run1_failed=0
full_suite_run2_passed=780
full_suite_run2_failed=0
phase_e=38/38
approval_unit=2/2
internal_et29_kat=1/1
compile_fail_authority=4/4
et04=PASS
et22=PASS
et23=PASS
et29=PASS
et30=PASS
et29_crypto=16/16
et29_substantive=40/40
public_authority_bypass_paths=0
production_trust=UNPROVISIONED
phase_d=73/73
determinism_files=9/9
determinism_manifest_records=8/8
determinism_two_runs=PASS
determinism_golden=PASS
linux_phase_e_gate=NOT_REQUIRED_R4
approval_decision=GO
```

### R4 procedural immutability invariant

The implementation-approval tag must be absent both locally and remotely
before creation; it is created once, pushed normally, and never moved. The
workflow prohibits force update, deletion, or any other tag movement. Before
merge, integration must re-fetch and verify the remote tag immediately; any
target or body mismatch blocks integration. This is procedural immutability,
not server-enforced immutability. No workflow step for this tag may use
`git tag -f`, `git push --force`, or `git push --delete`.

### R4 post-freeze exception and preserved contracts

R4 is an external evidence-mechanism amendment only. The already frozen
implementation SHA
`3076b102ed72b46b6c7c5b75f0360ff6f957968b` does not need to merge the R4
documentation commit and remains unchanged. R4 authority is bound externally
through `ism-mechanism-health-v1-e-plan-approved-r4` and the final
`ism-mechanism-health-v1-e-implementation-approved` tag. This exception applies
only because R4 changes post-freeze evidence packaging and no implementation or
review criterion. Future substantive plan amendments require the normal
merge/new-SHA review discipline.

R4 changes none of the scientific semantics, security authority,
`UNPROVISIONED` production trust, E-T29 `16/16` cryptographic or `40/40`
substantive evidence, publication state machine, Phase B/C/D behavior, schemas,
Cargo dependencies, goldens, Wilson mathematics, domain semantics,
`SCI-P1-001` closure, `SEC-P0-001` closure, or `COMP-P1-001` closure. It also
does not change the R3 macOS-only V1 contract: Linux remains deferred and
unsupported, Linux remains non-gating, the 15 deferred Linux code blocks and
one Linux evidence marker remain, and mandatory Linux requirements remain 0.

> **HISTORICAL / SUPERSEDED UNDER R5 — R3/R4 predecessor context.** The
> following platform and evidence text is retained for provenance only. The
> current R5 authority is defined above and is the sole active approval and
> integration workflow.

## R3 amendment — macOS-only V1 support and deferred Linux support

This is a narrow, documentation-only Phase-E R3 amendment. It changes the
MHI V1 Phase-E supported-platform and approval contract only. It does not
change the scientific mechanism mapping, health mapping, Wilson formula,
domain semantics, `SCI-P1-001`, security verifier semantics, trust
provisioning, the physical/software claim distinction, E-T29, the publication
safety state machine, Phase B/C/D semantics, schemas, dependencies, fixtures,
or implementation code.

### R3 authority and history

R2 remains the predecessor authority and is not rewritten or deleted:

```text
R2 plan commit: e65092088a28a2a9ed61364274dbd6ec46de5eb8
R2 plan tag: ism-mechanism-health-v1-e-plan-approved-r2
R2 plan SHA-256: e6e5195c7f56904afb06dfe937433f3498465fef1df191b8fb6856ee1ac792b6
R2 plan Git blob: 45c1441ac4d6e20c5626b299fe5293b00ea444fb
```

The R3 planning snapshot named
`c848fb17cc925d8d21fca2eefd36accd9348dfff` on
`codex/mhi-v1-e-independent-validation` as its pre-R3 implementation
candidate; that value is historical only. The current completed implementation
authority is the exact R4 `IMPLEMENTATION_REVIEW_SHA` recorded above. R3 is
retained as the predecessor plan authority, and R4 is the current evidence-only
amendment. This planning task does not merge the R4 plan commit into the
implementation branch, create the implementation-approval tag, or authorize
implementation.

### R3 normative platform decision

The MHI V1 Phase-E supported platform is **macOS only**. Required V1 platform
validation is macOS only. Linux is **DEFERRED / UNSUPPORTED FOR MHI V1 PHASE E**.

Linux is not a V1 release platform, V1 approval prerequisite, V1 integration
prerequisite, independent-review evidence requirement, E-T30 evidence
requirement, post-freeze review-evidence requirement, exact numeric-byte
requirement, or publication-state-machine evidence requirement. Linux failure,
absence, CI failure, or an unavailable Linux environment must not block
MHI V1 Phase-E approval or integration under R3.

Release wording for MHI V1 Phase E must not say or imply `Linux-supported`,
`cross-platform-supported`, `Linux-validated`, or `portable across
Linux/macOS`. The only supported V1 wording is **macOS-supported V1
implementation**. Existing Linux-specific code and compilation or generic CI
execution on Linux do not establish support. Only a separately approved later
milestone may restore a Linux release claim.

R3 keeps the exact scientific contract: the Wilson formula, operation order,
finite and zero behavior, independent decimal oracle, exact `to_bits()` and
serialized-byte validation, exact nine-file output, manifest contract, golden
comparison, byte determinism, and publication authority remain unchanged. The
sole platform narrowing is that those V1 validation and approval gates are
macOS-authoritative. Linux exact-bit matching and Linux publication evidence
are no longer Phase-E V1 requirements.

The independently accepted architecture P2 for the non-UTF-8 early-return
`DIR*` leak remains relevant whenever the affected shared/macOS path is
reachable; R3 does not dismiss it as Linux-only. The compatibility P2 for the
E-T25 permanent regression-test coverage weakness also remains documented.
Neither P2 is changed by this amendment; both require explicit R4
approval-evidence disposition and are non-blocking only under the existing P2
policy.

### R3 Linux inventory and classification

The following inventory was produced by read-only repository-wide searches of
the exact implementation candidate
`c848fb17cc925d8d21fca2eefd36accd9348dfff`, including `git grep -i linux`,
`target_os = "linux"`, `renameat2`, `RENAME_NOREPLACE`,
`RENAME_EXCHANGE`, `__errno_location`, `ubuntu`, and cross-platform terms.
Line ranges below refer to that exact candidate, not to this amended plan.

Every row marked **C** is a genuinely Linux-only Phase-E implementation or
retained evidence block. After R3 approval, each such block must have exactly
one nearby searchable
`MHI_V1_DEFERRED_LINUX_SUPPORT` marker. The inventory count, marker count, and
path/function mapping must match exactly in the implementation review.

| Class | Exact candidate path and block | Linux-only surface | Marker required after R3 approval |
|---|---|---|---|
| C | `src/mhi_validation/output.rs:1731-1734` | `native_rename_noreplace_flag()` and Linux `RENAME_NOREPLACE` value | yes |
| C | `src/mhi_validation/output.rs:1735-1738` | `native_rename_exchange_flag()` and Linux `RENAME_EXCHANGE` value | yes |
| C | `src/mhi_validation/output.rs:1743-1746` | Linux `native_flock()` implementation | yes |
| C | `src/mhi_validation/output.rs:2195-2232` | Linux variants of `O_WRONLY`, `O_RDWR`, `O_CREAT`, `O_EXCL`, `O_TRUNC`, `O_CLOEXEC`, `O_DIRECTORY`, `O_NOFOLLOW`, `AT_FDCWD`, and `AT_REMOVEDIR` | yes |
| C | `src/mhi_validation/output.rs:2245-2253` | Linux `NativeDirent` layout | yes |
| C | `src/mhi_validation/output.rs:2281-2293` | Linux extern-C declarations, including `__errno_location`, `openat`, `mkdirat`, `unlinkat`, `flock`, `fchmod`, `fdopendir`, `readdir`, `closedir`, and `syscall` | yes |
| C | `src/mhi_validation/output.rs:2299-2301` | Linux `native_openat()` | yes |
| C | `src/mhi_validation/output.rs:2307-2309` | Linux `native_mkdirat()` | yes |
| C | `src/mhi_validation/output.rs:2315-2317` | Linux `native_unlinkat()` | yes |
| C | `src/mhi_validation/output.rs:2329-2348` | Linux `native_rename_at()`, including `renameat2` syscall IDs for x86_64, aarch64, and arm | yes |
| C | `src/mhi_validation/output.rs:2354-2356` | Linux `native_fchmod()` | yes |
| C | `src/mhi_validation/output.rs:2362-2364` | Linux `native_fdopendir()` | yes |
| C | `src/mhi_validation/output.rs:2370-2372` | Linux `native_readdir()` | yes |
| C | `src/mhi_validation/output.rs:2378-2380` | Linux `native_closedir()` | yes |
| C | `src/mhi_validation/output.rs:2389-2394` | Linux `native_errno_location()` using `__errno_location` | yes |
| C | `src/mhi_validation/output.rs:2453` | Retained Linux-specific EIO test-evidence comment naming both historical targets | yes |

The known minimum Linux implementation inventory is complete: all 15 required
Linux-only implementation blocks are present above. The inventory also includes
one retained Linux-specific Phase-E evidence comment, for 16 total marker
targets. The repository-wide search found no additional Linux-only Phase-E
implementation or retained evidence path outside this file.

Classification boundaries are explicit:

- **A — V1 MACOS AUTHORITY:** the macOS `cfg` branches and `renameatx_np`
  implementation in `src/mhi_validation/output.rs`; these remain authoritative
  and must not be marked deferred.
- **B — SHARED UNIX:** common `#[cfg(unix)]` publication, locking, fsync,
  descriptor, path, and wrapper code, plus `O_RDONLY` and the opaque
  `NativeDir`; macOS uses these exact implementations, so they are not
  deferred.
- **C — DEFERRED LINUX IMPLEMENTATION/EVIDENCE:** the 15 implementation rows
  and one evidence-comment row above; retain them, mark them after R3 approval,
  and exclude them from the V1 support surface.
- **D — HISTORICAL / DOCUMENTARY:** the R2 Linux/macOS contract, prior Linux
  attempts, the retained `cross_platform_numeric_bytes.txt` fixture identifier,
  and the EIO evidence comment; preserve factual history but remove normative
  V1 requirements.
- **E — GENERIC PROJECT-WIDE LINUX SUPPORT OUTSIDE PHASE E:** the Ubuntu leg
  in `.github/workflows/ci.yml`, `Cargo.lock`'s `linux-raw-sys` dependency,
  the project README, general engineering specifications, and non-Phase-E
  plotting/path portability references. These remain outside the Phase-E
  contract. The Ubuntu CI leg is retained as **GENERAL PROJECT CI — RETAIN
  NON-GATING**, unless a later review proves it exists solely as a Phase-E gate.

The Phase-E fixture inventory and golden byte files are not deleted or renamed
merely because a historical filename contains `cross_platform`; their exact
scientific bytes remain authoritative and the R3 expectation is macOS V1
validation. General project documentation and CI are not silently changed by
this narrow plan amendment.

### R3 deferred-code marking and future reactivation contract

The future implementation step must place exactly one nearby marker on every
row C block and use this exact explanatory text:

```text
MHI_V1_DEFERRED_LINUX_SUPPORT
```

```rust
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
```

The post-implementation audit must run
`git grep -n "MHI_V1_DEFERRED_LINUX_SUPPORT"`, compare its marker count to the
16 C-row marker targets (15 implementation blocks plus one evidence path), and
verify exact path/function mapping. Any unmarked
Linux-only Phase-E code is a P1 documentation/scope defect. Any shared Unix or
macOS-authoritative code that is falsely marked is also a P1 scope defect.

Linux support may return only through a separately approved future plan. That
milestone must remove the deferred markers, define supported Linux
distributions/architectures/filesystems, rerun Linux publication tests,
validate `renameat2`, errno and readdir behavior, validate numeric/golden
determinism, complete independent review, and explicitly restore Linux release
wording. Until then, retained Linux implementation is unsupported.

## 1. Milestone decision and scientific motivation

### 1.1 Limitation remaining after Phase D

Phase D made the existing scientific state public, deterministic, and
auditable. It projects validated Phase B mechanism evidence and Phase C sensor
health assessments into a certified report bundle without recomputing science.
That closes the communication gap, but not the external-validation gap.

The remaining limitation is that the repository cannot yet quantify whether
those conclusions reproduce on evidence that was not used to configure,
calibrate, fit, initialize, or assess them. In particular, Phase D does not:

- prove that a mechanism assessment generalizes across independent acquisition
  families, sensors, matrices, temperatures, or experimental campaigns;
- measure sensor-health alert performance against independently established
  reference outcomes;
- detect training/validation leakage across lineage or acquisition families;
- calculate validation coverage, exclusion rates, confusion matrices, false
  alarm or miss rates, confidence intervals, or domain-stratified performance;
- turn synthetic tests, model agreement, a fitted timescale, or a rendered
  report into physical validation; or
- certify that all supported legacy schemas, CLI routes, and public artifacts
  remain compatible as one release candidate.

Phase E therefore adds a separate validation and certification layer. It
consumes existing artifacts plus a predeclared protocol and independent
reference manifest. It never mutates a Phase B mechanism conclusion, a Phase C
health status, or a Phase D report. Its result answers a narrower question:
“How did the frozen assessment behavior perform on the declared validation
cohort, under the declared acceptance rules?”

### 1.2 Scientific claim boundary

Phase E may report `meets_protocol`, `does_not_meet_protocol`, or
`indeterminate` for a validation endpoint. These are validation outcomes, not
new mechanism evidence levels, health statuses, or causal findings.

The following ceilings remain binding:

- timescale agreement alone is not physical mechanism identity;
- model residuals alone are not sensor-failure evidence;
- same-source, shared-family, unknown-lineage, synthetic, or training evidence
  is not independent validation;
- an absent denominator, reference outcome, uncertainty, or required domain
  stratum produces an unavailable/indeterminate metric, never zero or pass;
- a Phase E result never upgrades `Hypothesized` to
  `ExperimentallySupported`/`ValidatedForDomain` and never changes a Phase C
  dimension status; and
- real-domain validation claims require dual-signed physical validation
  approval verified against the embedded owner/registry trust root. Passing
  synthetic fixtures certifies software behavior only.

### 1.3 Confirmed repository cause and expected behavior

This is an intentionally deferred capability, not a regression in Phase D.
Repository inspection confirms the current production path ends at
`CommandSpec::ReportRender` → `runners::report` → canonical artifact readers →
`PublicReportProjection` → Phase D writers. The projection consumes one
declared artifact set and has no cohort, reference-outcome, leakage, metric, or
validation-acceptance model. The existing `model validate` path reads a
`ValidationManifest` and evaluates `ism_model_analysis`; it does not validate
Phase B mechanism or Phase C health behavior and must not be repurposed.

The earliest missing boundary is therefore after current mechanism/health
artifacts are serialized and before any cross-cohort scientific claim: there is
no canonical MHI validation dataset, protocol, evaluator, or result artifact.
Expected Phase E behavior is to add that separate boundary while leaving the
current assessment and rendering paths untouched.

## 2. Current capability baseline

All statements in this section describe the current Phase-E baseline at
`6b76258ff2e8ff71a1b8a68248b47cf224141d73`. The Phase-D release at
`2316fb1d076d47ac14d3b3c10c55654feb1ffb54` remains its required provenance
ancestor and byte/semantic compatibility authority.

### 2.1 Mechanism evidence

- `mechanism_analysis` schema 4 serializes Phase B hypothesis definitions,
  assessments, temporal joins, timescale assessments, amplitude,
  repeatability, identifiability, contradictions, validation-protocol status,
  component interpretation, reason codes, and deterministic history.
- Evidence is represented by the schema-1 `EvidenceBundle`, including typed
  targets, sources, availability, direction, strength authority, validity,
  quantities, uncertainty, threshold provenance, pairwise independence, and
  exact producer-backed timescale pair covariance.
- Phase B requires explicit hypotheses and gates. It preserves missing,
  contradictory, scope-incompatible, temporally incompatible, and
  non-independent evidence rather than filling gaps.
- Independent confirmation depends on durable lineage and known acquisition
  families. Unknown lineage or family membership cannot become independent.
- Legacy mechanism schemas 1–3 remain readable, but their legacy hypotheses
  are not silently reinterpreted as Phase B assessments.
- The permanent Phase B traceability target records 72 passing mechanism
  evidence tests at the tagged baseline.

### 2.2 Sensor health

- `health_assessment` schema 4 contains exactly one Phase C assessment for each
  of the nine ordered health dimensions, with evidence state, status,
  interpretation category, causal ceiling, reason codes, consumed/excluded
  evidence IDs, source artifact IDs, and an embedded evidence bundle.
- Overall status, interpretation categories, and causal status are derived
  deterministically from the serialized dimension assessments and validated at
  the artifact boundary.
- Data-quality insufficiency, indeterminacy, contradictory evidence, and an
  adequate negative finding remain distinct states.
- Baseline comparison, signal, calibration, dynamic response, reference
  stability, model consistency, observability/identifiability, uncertainty,
  and data sufficiency are explicit; optional-source absence cannot produce a
  healthy finding.
- Schema-3 health assessments remain readable through the legacy route but do
  not contain or synthesize the nine-dimension Phase C payload.
- The Phase C traceability audit covers 110 mandatory contracts; its recorded
  targeted artifact, Phase B, and Phase C suites pass at the tagged baseline.

### 2.3 Reporting

- `electroanalysis report render` is the single Phase-D-certified public route.
  It requires mechanism and health artifacts and optionally accepts lineage,
  EIS, transient, calibration, signal, estimation, and model artifacts.
- The route reads canonical artifacts, checks schema/scope compatibility, and
  performs a projection only. It does not discover raw data, refit models,
  reassess evidence, resolve lineage, or manufacture unavailable values.
- The certified bundle contains a schema-1 public summary, schema-1 render
  manifest, Markdown report, seven deterministic CSV table types, and up to
  eleven SVG/PNG figure types.
- Publication is staged and atomic. Selection, unavailability, legacy notices,
  compatibility, and lineage limitations are explicit. Repeated rendering of
  the same accepted inputs is deterministic.
- The Phase D acceptance matrix maps 73 requirements/criteria/tests and records
  all 73 as implemented and passing.

### 2.4 Provenance

- `AnalysisProvenance` records software version, input/configuration paths and
  SHA-256 hashes, generation time, and optional Git commit.
- `ArtifactLineageState` distinguishes authoritative `Known` identity and
  dependencies from `LegacyUnknown`; readers never fabricate historical
  identity.
- Known `ArtifactIdentity` includes artifact kind/schema, producer version,
  experiment scope, sensor/channel scope, acquisition families, and semantic
  SHA-256. Direct dependency roles are typed and deterministically ordered.
- `ArtifactLineageCatalog` supports deterministic closure, missing-ancestor,
  cycle, and inconsistent-root reporting. Unknown lineage cannot satisfy an
  independence requirement.
- Evidence records retain artifact/field sources, experiment scope, temporal
  support, threshold authority, derivation authority, and exact IDs needed to
  reconstruct a conclusion.

### 2.5 Reproducibility

- Rust is pinned to `1.97.0`; generic repository CI uses `Cargo.lock` and may
  run formatting, clippy, all tests, and a release build on Linux and macOS.
  Any Linux/Ubuntu result from that generic CI is informational and
  non-gating for MHI V1 Phase E; macOS is the only V1 release-validation
  platform.
- Scientific collections use stable ordering, typed units, finite-value
  validation, canonical semantic hashes, and explicit residual sign contracts.
- Stochastic workflows expose deterministic seeds where applicable.
- A0/A1/B/C/D compatibility fixtures cover legacy/current schemas, artifact
  kinds, identity, lineage, covariance, missing data, and negative paths.
- Phase D has sealed literal fixtures, exact reader routes, deterministic
  numeric spelling, immutable source-artifact checks, repeat rendering checks,
  and atomic-publication failure tests.

## 3. Scientific objectives

Phase E is complete only when every objective below has a committed test or,
for an actually requested physical claim, a schema-valid dual-signed immutable
owner/registry approval record verified under section 3.8. Scientific thresholds are required
protocol inputs; implementation defaults are forbidden.

| ID | Objective | Measurable acceptance criterion |
|---|---|---|
| E-SCI-01 | Predeclare the validation question and cohort before scoring. | Every run binds exact protocol bytes and one canonical dataset. Every endpoint contains the complete section 4.3 target, cohort role, domain, artifact, family, stratum, reference, metric, confidence, and acceptance authority. Every supporting endpoint domain equals its claim domain by mutual subset proof; no field or domain coverage is defaulted. Physical claims require section 3.8 approval and at least two eligible known-separated families. |
| E-SCI-02 | Prove separation between development and validation evidence. | Every endpoint/view/record receives the exact section 3.2 accounting and section 3.6 separation state using the required canonical lineage catalog and reference-source graph. Holdout known overlap fails; holdout unknown separation is indeterminate; only known separation may pass. |
| E-SCI-03 | Quantify mechanism reproducibility without changing Phase B. | Every declared hypothesis uses the total Phase-B/reference joint mapping in section 3.3 and emits exact category and declared-falsification ID sets/counts, eligible family IDs/count, all three fractions/Wilson intervals where defined, and rule evaluations. `n=s+c+u`; a declared serialized Phase-B contradiction cannot be selected away, and an eligible independent-reference contradiction fails when separation is known. |
| E-SCI-04 | Quantify health classification performance against an independent reference. | Every endpoint uses the exhaustive section 3.4 partitions and emits the six category ID sets/counts, exact eligible/evaluable denominators, coverage/missing-state rates, confusion metrics, Wilson intervals for defined binomial proportions, and point-only balanced accuracy. Every eligible record appears exactly once. |
| E-SCI-05 | Expose domain generalization rather than pooling it away. | Every endpoint emits overall and each closed section 3.6 required-stratum view. Empty or below either declared record/family minimum is indeterminate and forces the parent endpoint indeterminate regardless of aggregate success. |
| E-SCI-06 | Preserve reference-outcome authority and uncertainty. | Every reference endpoint and combined reference/catalog closure satisfies section 3.5. Structurally invalid authority is a hard error; a complete but protocol-ineligible reference is excluded with one precedence-selected typed reason only for software-exclusive support. Reference contradictions are falsifying evidence, derived or unknown-dependent references cannot pass holdout separation, and a physical mechanism outcome cannot be `unavailable` or selectively excluded. |
| E-SCI-07 | Separate software validation from physical scientific validation. | Software requests can never emit `physically_validated`. A physical request against an `UNPROVISIONED` production trust store hard-fails `PhysicalApprovalTrustNotProvisioned` before scoring; a `PROVISIONED` request additionally requires globally distinct real owner/registry authority IDs and keys, dual signatures accepted by the frozen weak-key-rejecting strict verifier against the binary's embedded production store, exact physical/blinded/domain-equal holdout bindings, and usable semantic outcome for every mechanism reference. Test-only known-answer roots certify verifier behavior only. Actual post-partition family underpowering is indeterminate, never silently excluded. Absence of a physical cohort or production provisioning never blocks software implementation or E-SW. |
| E-SCI-08 | Make every result reconstructible and byte-reproducible. | Authority-assisted report validation rereads the exact hashed protocol, dataset, trust store, and consumed sources and reconstructs every set, count, metric, exclusion, separation decision, endpoint/claim outcome, lineage/provenance reference, and report ID. Section 4.4 freezes every JSON field, CSV/Markdown cell, manifest token, and macOS V1 numerical bit vector. |

The protocol may demand stronger sampling, replication, confidence, or domain
requirements. It may not weaken the independence, lineage, missing-data, or
claim-ceiling rules above.

### 3.1 Closed V1 scientific vocabularies

The following wire vocabularies are exhaustive. An implementation may not add
a token, synonym, implicit default, or catch-all variant during Phase E.

| Type | Exact V1 tokens or variants |
|---|---|
| `CohortRoleV1` | `development`, `validation`, `holdout` |
| `EvidenceOriginV1` | `physical`, `synthetic`, `constructed`, `unknown` |
| `EndpointKindV1` | `mechanism`, `health_dimension`, `health_aggregate` |
| `SeparationStatusV1` | `known_separated`, `known_overlap`, `unknown_separation` |
| `RecordDecisionV1` | `eligible`, `excluded`, `not_applicable` |
| `ValidationOutcomeV1` | `meets_protocol`, `does_not_meet_protocol`, `indeterminate` |
| `ReleaseClaimOutcomeV1` | `physically_validated`, `software_validated_only`, `does_not_meet_protocol`, `indeterminate` |
| `RequestedValidationLevelV1` | `software`, `physical` |
| `ComparatorV1` | `greater_than_or_equal`, `less_than_or_equal` |
| `RateTargetV1` | `point_estimate`, `lower_confidence_bound`, `upper_confidence_bound` |
| `BlindingStateV1` | `blinded_to_assessment`, `not_blinded`, `unknown` |
| `BlindingRuleV1` | `require_blinded`, `allow_declared_unblinded` |
| `ReferenceDependencyCompletenessV1` | `complete`, `unknown` |
| `MetricUnavailableReasonV1` | `denominator_zero`, `positive_class_denominator_zero`, `negative_class_denominator_zero`, `sensitivity_or_specificity_unavailable` |
| `SeparationUnknownReasonV1` | `assessed_experiment_scope_unknown`, `assessed_family_unknown`, `reference_experiment_scope_unknown`, `reference_family_unknown`, `reference_dependency_incomplete`, `reference_dependency_node_missing`, `reference_scientific_leaf_missing`, `reference_scientific_leaf_legacy_unknown`, `catalog_ancestor_missing`, `catalog_cycle_reachable`, `development_experiment_scope_unknown`, `development_family_unknown` |
| `RuleEvaluationResultV1` | `true`, `false`, `unavailable` (JSON strings, not booleans) |
| `LegacyLineageReasonV1` | `field_absent_in_legacy_artifact`, `external_artifact_without_lineage`, `migration_information_unavailable` |

`unknown` evidence origin, unknown blinding, and unknown reference dependency
completeness are preserved states. They can never satisfy a physical claim.
No filename, path component, timestamp, sensor ID, campaign ID, or method name
may be used to infer any token above.

### 3.2 Endpoint membership and exact denominator sets

Protocol validation freezes each endpoint's target, one scoreable cohort role
(`validation` or `holdout`), domain selector, required artifact path,
reference rule, overall minimums, required strata, and acceptance rules before
any source artifact is opened. `development` is never a scoreable role; all
development records are leakage comparators.

For every endpoint and every view (`overall` plus each required stratum), the
evaluator constructs these sorted, duplicate-free record-ID sets in this exact
order:

1. `declared_record_ids`: records whose declared `cohort_role` equals the
   endpoint role and whose declared domain satisfies the endpoint domain and
   view predicate. A null endpoint artifact path remains declared and receives
   exclusion reason `missing_endpoint_artifact_path`; it is not a hard error.
2. `excluded_record_ids`: declared records that fail an eligibility rule after
   all source files that are present pass their hard boundary checks. Each pair
   `(endpoint_id, stratum_id, record_id)` has exactly one primary exclusion
   reason selected by section 3.7 precedence and may have sorted secondary
   reasons.
3. `eligible_record_ids = declared_record_ids - excluded_record_ids`.
4. Records outside the endpoint role/domain/view are `not_applicable` for that
   view and appear in the per-record accounting ledger, but not in any of the
   three sets above.

For reporting only, `development_record_ids`, `validation_record_ids`, and
`holdout_record_ids` are separately the records of that exact role whose domain
satisfies the endpoint and view predicate, before source eligibility. They are
not an additional partition: roles other than the endpoint's scoreable role
also appear in `not_applicable_record_ids`. All four reporting sets are derived
from dataset declarations without opening a source.

The invariant is:

```text
declared_record_ids = disjoint_union(eligible_record_ids, excluded_record_ids)
declared_count      = eligible_count + excluded_count
exclusion_rate      = excluded_count / declared_count
```

`exclusion_rate` uses the declared cohort denominator for every endpoint/view,
not the post-exclusion eligible cohort. It is unavailable with
`denominator_zero` when `declared_count=0` and otherwise carries a Wilson 95%
interval under section 3.7.

Record IDs alone do not establish uniqueness. `ScientificSourceKeyV1` is:

```text
Known source         -> (artifact_kind, artifact_id, semantic_sha256)
LegacyUnknown source -> (artifact_kind, schema_version, source_file_sha256)
```

For each endpoint, `AssessedSourceKeyV1 = (endpoint_id,
ScientificSourceKeyV1)`, and the dataset rejects two records with the same
assessed key.

Path and record ID are deliberately absent from this key. Renaming or copying
the same source therefore cannot increase a denominator. The reader also
requires every record-declared experiment scope, sensor/channel scope, and
acquisition-family set to equal the corresponding Known artifact identity.
For `LegacyUnknown`, those declarations must be explicit `unknown`; a manifest
may not assign authoritative scope or family metadata to a legacy source.

### 3.3 Frozen Phase-B mechanism mapping

Phase E reads exactly one schema-4 `HypothesisAssessmentRecord` for the declared
hypothesis from each eligible mechanism artifact. For every record in the
artifact, `definition.hypothesis_id` must equal `current.hypothesis_id` by exact
UTF-8 bytes. Either-side disagreement, a duplicate definition ID, or a duplicate
current ID is a hard artifact-validation failure. After those checks,
`definition.hypothesis_id` is the sole authoritative lookup/deduplication key.
An absent declared hypothesis is an
eligible `not_assessed_or_other` record with reason
`declared_hypothesis_absent`; Phase E does not call the Phase-B assessor.

The protocol field `support_levels` is a nonempty, duplicate-free subset of
`hypothesized`, `experimentally_supported`, and `validated_for_domain`.
`not_assessed` and `contradicted` are invalid members. Mapping is total and uses
only the serialized `evidence_level`:

Mechanism-reference semantic outcomes are not configurable selection filters:
every valid mechanism rule admits `supports`, `contradicts`, and
`not_assessed`; `unavailable` is a software-exclusive exclusion and a physical
pre-scoring hard failure. The final mechanism category is the following total
joint mapping. `policy_support` means the Phase-B level is
one of the endpoint's declared `support_levels`; `policy_other` means it is one
of `hypothesized`, `experimentally_supported`, or `validated_for_domain` but is
not in that set.

| Serialized Phase-B level | Reference `supports` | Reference `contradicts` | Reference `not_assessed` | Reference `unavailable` |
|---|---|---|---|---|
| `contradicted` | `critical_contradiction` | `critical_contradiction` | `critical_contradiction` | software-only support: excluded `reference_outcome_unavailable`; physical support: hard section 3.8 failure |
| `not_assessed` | `not_assessed_or_other` | `critical_contradiction` | `not_assessed_or_other` | software-only support: excluded `reference_outcome_unavailable`; physical support: hard section 3.8 failure |
| `policy_support` | `support` | `critical_contradiction` | `not_assessed_or_other` | software-only support: excluded `reference_outcome_unavailable`; physical support: hard section 3.8 failure |
| `policy_other` | `not_assessed_or_other` | `critical_contradiction` | `not_assessed_or_other` | software-only support: excluded `reference_outcome_unavailable`; physical support: hard section 3.8 failure |

The category retains both source reason codes. `reference_contradicts` is a
critical contradiction even when Phase B serialized a supported state;
`reference_not_assessed` cannot enter a support numerator. This consumes the
independent outcome without reassessing or mutating Phase-B evidence.

Phase-B reason codes, gate assessments, contradiction summaries, component
interpretations, and validation status are retained as source authority but do
not change this joint mapping. V1's only critical-contradiction policy token is
`any_contradicted_record_fails`; no threshold may waive it.

Exclusion cannot select away a serialized Phase-B contradiction. Independently
of the eligible denominator, `declared_critical_falsification_record_ids` is the
sorted set of declared records whose present, strict-reader-valid, scoreable
Phase-B artifact serializes `contradicted`, plus records whose otherwise
protocol-eligible mechanism reference serializes `contradicts`. Missing or
ineligible reference authority never erases the Phase-B member. Any nonempty
set is `does_not_meet_protocol` after separation precedence and before power or
ordinary acceptance rules. Malformed/hash-invalid sources remain hard errors.
The eligible `critical_contradiction_record_ids` remains the category set used
in `n=s+c+u`, so excluded records never enter a metric denominator.

For a mechanism endpoint/view:

```text
n = eligible_record_ids.len()
s = support_record_ids.len()
c = critical_contradiction_record_ids.len()
u = not_assessed_or_other_record_ids.len()
n = s + c + u

support_fraction       = s / n
contradiction_fraction = c / n
not_assessed_fraction  = u / n
```

All three fractions and Wilson intervals are unavailable when `n = 0`.

### 3.4 Frozen Phase-C health mapping

Each health endpoint declares two duplicate-free predicted-status sets. Their
intersection must be empty and their union must equal exactly:

```text
within_baseline, watch, degraded, critical
```

`indeterminate` and `data_quality_insufficient` are forbidden in either set.
For a dimension target, Phase E reads only the serialized status of the unique
matching `PhaseCHealthDimensionAssessment`. For an aggregate target, it reads
only serialized `phase_c.overall_status`. Schema-4 artifact validation must
already prove exactly nine unique dimensions and consistency of the serialized
aggregate; Phase E does not derive or repair either status.
Each endpoint also declares a nonempty `reference_label_universe` and disjoint,
nonempty `reference_positive_labels` and `reference_negative_labels`; their
union must exactly equal the universe. A dataset label outside the universe is
a hard dataset/protocol binding failure. No class direction is inferred.

For every eligible health record, apply this total mapping:

| Serialized Phase-C status | Reference class | Category |
|---|---|---|
| predicted positive | positive | `tp` |
| predicted positive | negative | `fp` |
| predicted negative | positive | `fn` |
| predicted negative | negative | `tn` |
| `indeterminate` | either | `indeterminate` |
| `data_quality_insufficient` | either | `data_quality_insufficient` |

The reference label is required and protocol-eligible even when the Phase-C
status is Indeterminate or DQI. Exact record-ID invariants are:

```text
eligible_record_ids = disjoint_union(
    tp_record_ids,
    tn_record_ids,
    fp_record_ids,
    fn_record_ids,
    indeterminate_record_ids,
    data_quality_insufficient_record_ids,
)

eligible_count = TP + TN + FP + FN + indeterminate
                 + data_quality_insufficient
evaluable      = TP + TN + FP + FN
```

The denominator called `declared_eligible_cohort` in earlier planning text is
replaced by the exact `eligible_count` above. Metrics are:

```text
coverage                           = evaluable / eligible_count
indeterminate_rate                 = indeterminate / eligible_count
data_quality_insufficient_rate     = data_quality_insufficient / eligible_count
sensitivity                        = TP / (TP + FN)
specificity                        = TN / (TN + FP)
false_positive_rate                = FP / (FP + TN)
false_negative_rate                = FN / (FN + TP)
balanced_accuracy                  = (sensitivity + specificity) / 2
```

`balanced_accuracy` is available only when both sensitivity and specificity
are available and has no interval in V1. A zero denominator stores an
unavailable reason and never a numeric zero, pass, imputation, correction,
weight, or pooled substitute.

### 3.5 Reference authority and derived-reference exclusion

Every endpoint declares `ReferenceAuthorityRuleV1`: allowed method
ID/version pairs, allowed assessor/instrument authority IDs, blinding rule,
uncertainty rule, and required reference outcome kind. A mechanism rule has no
allowed-outcome subset: `supports`, `contradicts`, and `not_assessed` are all
admitted to the joint mapping in section 3.3, while `unavailable` is never
eligible: it is a software-exclusive exclusion or physical hard failure as
section 3.8 specifies. Dataset reference records are closed tagged variants:

- `MechanismReferenceV1`: hypothesis ID and one of `supports`, `contradicts`,
  `not_assessed`, or `unavailable`. Its outcome and the Phase-B state jointly
  determine the section 3.3 category; it never rewrites Phase-B state.
- `HealthReferenceV1`: health target and one label from the protocol's exact
  label universe.

Both variants carry `reference_source_id`, method ID/version, authority ID,
blinding state, `ReferenceUncertaintyV1`, and limitations. Their referenced
`ReferenceSourceAuthorityV1` node, not the endpoint outcome record, carries the
authoritative experiment scope and acquisition families.
`ReferenceUncertaintyV1` is exactly one of:

```text
quantified { measure_id, finite_nonnegative_value, unit }
unavailable { nonempty_reason }
```

The protocol uncertainty rule is exactly one of:

```text
require_quantified { measure_id, unit, maximum_inclusive }
allow_unavailable_with_limitation
```

No uncertainty value changes a label or class. Under `require_quantified`, a
missing, mismatched-unit/measure, nonfinite, or above-maximum uncertainty is a
typed exclusion. Under `allow_unavailable_with_limitation`, a quantified
reference is eligible; an explicitly unavailable uncertainty remains eligible
for descriptive accounting but forces its endpoint/view `indeterminate` and
retains the reason/limitation. It can never contribute to a passing endpoint.
A missing reference endpoint or disallowed method/authority/blinding is a typed
scientific exclusion. A structurally missing required field, unknown token,
wrong binding, or nonfinite serialized value is a hard schema failure before
scoring.

For an endpoint that supports any physical claim, a mechanism reference
outcome of `unavailable` is hard `PhysicalReferenceOutcomeUnavailable` in the
section 3.8 pre-scoring gate, even when method, authority, blinding,
quantification, origin, and dependencies are otherwise valid. It is never an
exclusion and therefore cannot shrink a physical performance denominator. The
ordinary `reference_outcome_unavailable` exclusion applies only to endpoints
that support software claims exclusively.

`require_blinded` accepts only `blinded_to_assessment`.
`allow_declared_unblinded` accepts `blinded_to_assessment` or `not_blinded` but
never `unknown`. The protocol never infers blinding from assessor or method.

`ReferenceSourceAuthorityV1` nodes form a closed, canonical directed graph in
the dataset. Every node contains source ID, source-file SHA-256, evidence
origin, dependency completeness, scope, family, and sorted direct dependencies
tagged as another reference source or a `ScientificSourceKeyV1`.

Reference independence uses one combined graph, never a reference-only graph:

1. Start at the record's `reference_source_id` and traverse reference-source
   dependencies depth-first in canonical dependency order.
2. A `reference_source` dependency must resolve to exactly one dataset node.
   An absent node or `dependency_completeness=unknown` marks the combined
   closure incomplete.
3. A Known `scientific_artifact` leaf must resolve by exact
   `(artifact_kind, artifact_id, semantic_sha256)` to exactly one lineage-
   catalog node. The node's artifact kind and semantic identity must match the
   leaf. An absent leaf is incomplete reason `reference_scientific_leaf_missing`;
   a mismatching leaf is the hard error in section 3.6. Recursively traverse every catalog `direct_dependency`; a missing
   ancestor or reachable cycle marks the combined closure incomplete. A
   LegacyUnknown scientific leaf marks it incomplete and is never treated as
   an empty terminal.
4. Build the assessed-source closure by the same recursive catalog algorithm.
   Intersection at any Known artifact ID, semantic SHA-256, available
   source-file SHA-256, experiment ID, or family ID yields `known_overlap`,
   including the exact chain `reference -> scientific artifact X -> assessed
   artifact A`. “Available source-file SHA-256” is exactly the hash on a
   present `ArtifactSourceExpectationV1` root or
   `ReferenceSourceAuthorityV1` node. The existing catalog has no source-file
   hash field, so a catalog-only ancestor contributes artifact identity/scope/
   family but never an invented file hash.
5. Otherwise any incomplete closure/scope/family yields
   `unknown_separation`; only complete disjoint closures yield
   `known_separated`.

Reference-node cycles are closed-schema invalid and hard-fail; reachable
catalog cycles have the treatment frozen in section 3.6. Changing filenames,
endpoint IDs, or inserting a derived intermediate cannot erase a dependency.

### 3.6 Lineage, scope, family separation, and strata

The dataset references exactly one canonical schema-1
`ArtifactLineageCatalog` by safe relative path and exact file SHA-256. It is
read with the additive `domain::read_artifact_lineage_catalog_strict`; Phase-E
module-local/ad-hoc parsing is forbidden. The existing non-strict reader remains
the unchanged Phase-D compatibility route.
Root and closure conditions have exactly these treatments:

| Catalog condition | Exact treatment |
|---|---|
| malformed JSON, unknown/duplicate field/key, unsupported schema, invalid node identity/dependency shape | hard error |
| Known assessed root absent | hard `AssessedRootMissing` |
| root catalog key/identity differs from the embedded assessed identity | hard `AssessedRootIdentityMismatch` |
| root direct dependency set/role/kind/ID differs from the embedded assessed direct dependencies | hard `AssessedRootDependencyMismatch` |
| a dependency repeats an artifact ID with a different artifact kind | hard `CatalogDependencyKindMismatch` |
| a Known scientific leaf from the reference graph has a matching artifact ID but mismatched kind or semantic SHA-256 | hard `ReferenceLeafIdentityMismatch` |
| a required transitive ancestor is absent | `unknown_separation` reason `catalog_ancestor_missing` |
| a cycle is reachable below a validated root | `unknown_separation` reason `catalog_cycle_reachable` |
| any reachable experiment scope or family is Unknown | `unknown_separation` with the exact scope/family reason |

Thus root authority failure is never downgraded, while lack of transitive
knowledge is never fabricated as separation. Complete closure requires every
transitive node. In this table, “assessed root” includes every present
mechanism/health source on validation, holdout, and development records; role
does not weaken root identity/dependency binding.

For a scoreable mechanism record, its evaluated closure is the union of its
assessed mechanism-source closure and its matching reference-endpoint closure.
The development comparator closure is the union, over every `development`
record, of that record's non-null mechanism-source closure and every mechanism
reference closure on that record bound to the same protocol endpoint. For a
scoreable health record, use the analogous health sources/references. A null
development source contributes no assessed root; a present development
reference still contributes its closure. Both validation/holdout and
development closures are recursively expanded; development data are
comparators only and are never scored.
For the source-hash comparison below, each assessed/development source root
contributes its verified expectation file hash and each traversed reference
node contributes its declared `source_file_sha256`; catalog-only ancestors
contribute no source hash because the existing catalog schema has no such
field. Separation is:

1. `known_overlap` if any artifact ID, semantic SHA-256, available source-file
   SHA-256, experiment ID, or acquisition-family ID is shared between the
   evaluated closure and development comparator closure, or between the
   evaluated record's assessed and reference closures. Reference descent from
   the assessed source and reference descent from a development source are
   therefore both overlap;
2. otherwise `unknown_separation` if any required closure, scope, family, or
   reference dependency is unknown/incomplete;
3. otherwise `known_separated`.

An evaluated `known_overlap` row has at least one nonempty shared-ID collection;
an evaluated `unknown_separation` row has a nonempty canonical
`SeparationUnknownReasonV1` list; a `known_separated` row has all shared and
unknown collections empty. For every evaluated row,
`compared_development_record_ids` equals the exact comparator set above, even
when the assessed/reference self-intersection alone establishes overlap.

Missing assessed-source or matching-reference prerequisites do not erase a
known overlap visible from the closures that are present. The evaluator first
constructs every present strict-reader-valid assessed/reference/development
closure and tests all available intersections. If any intersection is known,
the row is evaluated `known_overlap` even though its accounting decision is
also excluded for the missing prerequisite. Only when no overlap is visible
and either the assessed or reference prerequisite is absent is separation null
with the lowest applicable missing-prerequisite exclusion token as
`not_evaluated_reason`. `unknown_separation`/`known_separated` require both
evaluated-record prerequisites to be present.

For a holdout endpoint, separation precedence scans every declared record whose
closure was evaluable, whether or not another scientific exclusion also applies;
an exclusion cannot hide leakage. Any `known_overlap` forces
`does_not_meet_protocol`; otherwise any `unknown_separation` forces
`indeterminate`. A record with no visible overlap for which missing source/
reference prerequisites leave separation non-evaluable has null separation and
is handled by exclusion and empty/underpowered precedence. For a validation endpoint, known-overlap and
unknown records are excluded with typed reasons; remaining known-separated
records may be scored. A physical claim can reference only a holdout endpoint.

For each endpoint/view, `eligible_family_ids` is the sorted set union of the
Known assessed-artifact acquisition-family IDs for `eligible_record_ids`;
`independent_family_count = eligible_family_ids.len()`. Reference-source family
IDs never increase this count, although they must be Known and disjoint for
known separation. Repeating a family across records counts once. A Known
aggregate artifact contributes every family serialized in its authoritative
identity; an unknown family makes holdout separation unknown and cannot satisfy
a minimum.

`DomainSelectorV1` contains an explicit selector for analyte, matrix, sensor
design, sensor, campaign, and temperature. Each categorical selector is
exactly `any_declared` or `allowed { nonempty_sorted_ids }`. Temperature is
exactly `any_declared` or `bands { nonempty_sorted_nonoverlapping_bands }`;
each band uses `lower_kelvin_inclusive < upper_kelvin_exclusive`. A record must
declare every domain axis; a missing/empty/nonfinite axis is a hard closed-
dataset or domain-binding error, never an exclusion or inferred value. A
complete record outside the selector is `not_applicable`. Membership is the
conjunction of all six axes. For subset validation,
`allowed` is narrower than `any_declared`; one categorical allowed set is
narrower than another iff it is a set subset; temperature bands are interpreted
as their half-open union and are narrower iff that union is a subset. Equality
uses mutual subset. Empty allowed sets and overlapping bands are rejected;
adjacent bands may touch because upper is exclusive and lower is inclusive.

Each `RequiredStratumV1` has a unique `stratum_id`, a nonempty conjunction of
the following internally tagged `predicate` variants in this exact
discriminant/canonical order:

```text
analyte_equals       { id }
matrix_equals        { id }
sensor_design_equals { id }
sensor_equals        { id }
campaign_equals      { id }
temperature_band     { lower_kelvin_inclusive, upper_kelvin_exclusive }
```

At most one predicate per axis is allowed; a repeated axis, repeated predicate,
empty ID, nonfinite/nonpositive temperature, or non-increasing band is a hard
protocol error. Because equality predicates are single-valued, two conditions
on one categorical axis are contradictory and rejected rather than combined.
Predicates use only declared dataset metadata and are a conjunction in the
order above. `minimum_eligible_records` and
`minimum_independent_families` are integer `u64` values at least one. Overall
has the same two positive minima directly on the endpoint and is the reserved
`stratum_id="overall"`; it cannot be declared by the protocol. Required strata
may overlap as separate views but are never summed. A view (overall or stratum)
is:

- empty when `eligible_count = 0`;
- underpowered when eligible count or independent-family count is below its
  declared minimum; and
- `indeterminate` in either case.

An empty/underpowered overall view makes the endpoint indeterminate. Any
required stratum that is indeterminate also makes its parent endpoint
indeterminate even if the overall view or another stratum passes. For a
physical supporting endpoint, protocol validation requires both minima to be
at least two on overall and every stratum; after partitioning, falling below
either declared minimum is indeterminate, not a hard error and not an
exclusion.

### 3.7 Wilson calculation, acceptance, and total precedence

Every binomial fraction `p = x / n`, `n > 0`, uses the two-sided Wilson 95%
interval `wilson_95_v1`, with `z = 1.959963984540054`:

All count operands must be at most `9_007_199_254_740_992` (`2^53`) so their
binary64 conversion is exact. A larger declared/derived collection is a hard
`CountExceedsExactF64Range` error before metric computation.
Every numerator is a reconstructed subset count and must satisfy `x <= n`;
violation is hard `InvalidBinomialCount` before conversion.

```text
z2          = z * z
p           = x_as_f64 / n_as_f64
denominator = 1.0 + z2 / n_as_f64
center      = (p + z2 / (2.0 * n_as_f64)) / denominator
radicand    = p * (1.0 - p) / n_as_f64
              + z2 / (4.0 * n_as_f64 * n_as_f64)
half_width  = z / denominator * sqrt(radicand)
lower       = max(0.0, center - half_width)
upper       = min(1.0, center + half_width)
```

Rust V1 uses IEEE-754 binary64, the operation order above, no reassociation,
no fused multiply-add, and `f64::sqrt`. Fixed vectors are checked on macOS
within `1e-12` of independently derived decimal references and for exact
`to_bits()` and serialized-byte equality. The independent decimal oracle and
the exact macOS bit contract remain mandatory; Linux exact-bit matching is not
a Phase-E V1 approval requirement.

`AcceptanceRuleV1` is a tagged union:

```text
count { rule_id, CountMetricV1, ComparatorV1, threshold_u64 }
rate  { rule_id, RateMetricV1, RateTargetV1, ComparatorV1,
        finite_threshold_inclusive_0_to_1 }
```

`CountMetricV1` is limited to the serialized counts named in sections 3.3 and
3.4 plus independent-family count. `RateMetricV1` is limited to the named
fractions. `balanced_accuracy` permits only `point_estimate`; count metrics
have no confidence-bound target. Invalid metric/target pairs fail protocol
validation. Rules must have unique IDs. The constraint-group key is exactly
`(count, metric)` for a count rule and `(rate, metric, target)` for a rate rule.
Within one key, the greatest `greater_than_or_equal` threshold may equal but
must not exceed the least `less_than_or_equal` threshold; a strict exceedance
rejects the protocol as contradictory. Every rule remains serialized and is
evaluated; validation does not collapse redundant bounds. No implicit
tolerance or rounding is used.

Before endpoint outcomes, record decisions use this exhaustive
`ExclusionReasonV1` precedence. The integer is the wire precedence ordinal;
the exact token is serialized.

| Ordinal | Condition | Exact primary/secondary reason token |
|---:|---|---|
| 1 | declared endpoint artifact path is null | `missing_endpoint_artifact_path` |
| 2 | present artifact is readable legacy or current LegacyUnknown rather than scoreable Phase B/C schema 4 | `source_not_phase_b_or_c_scoreable` |
| 3 | required reference endpoint is absent | `missing_reference_endpoint` |
| 4 | software-exclusive mechanism reference outcome is `unavailable` | `reference_outcome_unavailable` |
| 5 | method ID/version not in the allowed pair set | `reference_method_not_allowed` |
| 6 | authority ID not allowed | `reference_authority_not_allowed` |
| 7 | blinding state fails the declared blinding rule | `reference_blinding_not_allowed` |
| 8 | `require_quantified` receives unavailable uncertainty | `reference_uncertainty_unavailable` |
| 9 | quantified uncertainty measure ID differs | `reference_uncertainty_measure_mismatch` |
| 10 | quantified uncertainty unit differs | `reference_uncertainty_unit_mismatch` |
| 11 | quantified uncertainty exceeds the inclusive maximum | `reference_uncertainty_above_maximum` |
| 12 | validation-role record has known overlap | `validation_known_overlap` |
| 13 | validation-role record has unknown separation | `validation_unknown_separation` |

An outside-role/domain/view record is `not_applicable`, with null primary
reason and an empty secondary list; no source is opened solely for that view.
For a declared record, every condition whose prerequisite input exists is
evaluated without short-circuiting. If no condition is true the decision is
`eligible` and both reason fields are empty. Otherwise the decision is
`excluded`, the lowest ordinal true condition is primary, and every other true
token is a duplicate-free secondary list sorted by ordinal. A null source path
does not fabricate source-dependent reasons, but an independently present
reference may still supply its own reasons. Structurally missing fields,
malformed sources, unsafe paths, hash/identity/binding mismatch, physical-origin
failure, physical-reference-authority failure, and trust/signature failure are
hard errors and never appear in this vocabulary. For any endpoint supporting a
physical claim, the section 3.8 pre-scoring gate supersedes the otherwise
available source/reference exclusions: a record cannot be excluded to evade a
physical authority requirement. Holdout known/unknown separation does not
exclude an otherwise eligible record; it controls the endpoint outcome below.

All acceptance rules are evaluated without short-circuiting and combined with
logical AND. Total endpoint precedence is:

1. hard boundary error: abort run, no report;
2. holdout known overlap: `does_not_meet_protocol`;
3. holdout unknown separation: `indeterminate`;
4. nonempty mechanism `declared_critical_falsification_record_ids`:
   `does_not_meet_protocol`;
5. empty/underpowered overall or required-stratum view: `indeterminate`;
6. any eligible reference with explicitly unavailable uncertainty:
   `indeterminate`;
7. if any required acceptance target is unavailable: `indeterminate`, even if
   another defined rule is false;
8. otherwise if any rule is false: `does_not_meet_protocol`;
9. otherwise: `meets_protocol`.

Reasons are all retained and sorted by precedence then rule/reason ID. Overall
and release-support composition is independent of input order:

```text
any required indeterminate -> indeterminate
else any does_not_meet_protocol -> does_not_meet_protocol
else -> meets_protocol
```

### 3.8 Immutable physical-approval authority and production provisioning

Every release claim declares `requested_level`. Software claims may be scored
from synthetic, constructed, physical, or unknown-origin data but can emit only
`software_validated_only`, `does_not_meet_protocol`, or `indeterminate`.
Physical claims require the following pre-scoring authority checks:

1. endpoint role is `holdout`;
2. every record declared for any supporting endpoint has a non-null, current
   schema-4 Known assessed source and exactly one bound reference endpoint; every
   such record and every reference source reachable from it has origin
   `physical`; absence/LegacyUnknown/nonphysical is an approval-binding hard
   failure, not an exclusion that can be dropped;
3. all required reference rules demand `require_blinded` and
   `require_quantified`, and every actual matching reference has an allowed
   method/authority, `blinded_to_assessment`, a matching finite quantified value
   at or below the maximum, and `dependency_completeness=complete` on every
   reachable reference node. Every matching mechanism reference must also have
   a semantic outcome in `supports|contradicts|not_assessed`; `unavailable` is
   hard `PhysicalReferenceOutcomeUnavailable` before partitioning. Any other
   failure in this item is hard `PhysicalReferenceAuthorityMismatch`, not a
   selectively excluded record.
   Catalog transitive missing/cycle/scope/family treatment remains the section
   3.6 scientific indeterminate/failure path;
4. every supporting endpoint overall and required-stratum protocol record and
   family minimum is at least two;
5. a closed schema-1 `OwnerApprovalEvidenceV1` file is referenced by safe
   relative path and exact SHA-256 from the dataset; and
6. that approval binds the exact protocol SHA-256, dataset cohort semantic
   SHA-256, release claim IDs, endpoint IDs, reference authority IDs, target
   domain, physical-origin assertion, and the protocol's trusted approval root.

Consequently every declared record on a physical supporting endpoint either
passes the pre-scoring gate or aborts the run: none can receive section 3.7
exclusion ordinals 1–11. After a successful gate,
`eligible_record_ids=declared_record_ids` for those endpoints. Known/unknown
holdout separation still controls fail/indeterminate, and actual record/family
power still controls indeterminate; neither operation selectively removes a
record from the physical metric denominator.

The production runtime trust authority is never supplied by the protocol,
dataset, approval, environment, CLI, feature flag, or network. Phase E embeds
the exact bytes of the one reviewed production file
`config/mhi_physical_approval_trust_store.schema1.json` with `include_bytes!`.
It is the only authority selectable by the production CLI. The strict approval
reader validates those bytes during physical-path initialization, after
protocol validation and before opening the dataset; an all-software run does
not parse or report the store. The complete closed JSON wire schema is:

```text
PhysicalApprovalTrustStoreV1 {
  schema_version:1,
  trust_store_id:"mhi_physical_approval_trust_store_v1",
  provisioning_state:"UNPROVISIONED" | "PROVISIONED",
  trust_roots:[canonical PhysicalApprovalTrustRootV1]
}

PhysicalApprovalTrustRootV1 {
  trust_root_id,
  project_owner_authority_id,
  owner_ed25519_public_key_hex,
  registry_authority_id,
  registry_ed25519_public_key_hex
}
```

The JSON field names and displayed order are exact; no alias such as `roots`,
`approval_roots`, or a state alias is accepted. The two provisioning states
are closed and have the following exact representation and behavior:

| `provisioning_state` | Required `trust_roots` value | Physical-request behavior |
|---|---|---|
| `UNPROVISIONED` | The only valid value is `[]`. | Hard-fail `PhysicalApprovalTrustNotProvisioned` before scientific scoring, dataset opening, approval parsing, or report creation. There is no fallback to a software request. |
| `PROVISIONED` | A nonempty canonical array of `PhysicalApprovalTrustRootV1`. | Continue with the strict selected-root, approval-binding, and scientific gates in this section. |

`UNPROVISIONED` is a supported production state: E-SW/software validation and
software release behavior remain fully usable, but no physical claim can be
authorized. The initial Phase-E production file is `UNPROVISIONED`; it contains
no public trust root. A physical request against it returns the exact typed
error above regardless of any protocol, dataset, approval, or test fixture
contents. Implementation must remove the currently embedded public test-vector
keys from this production file rather than merely ignore them. Its absence must
never block Phase-E software implementation or a software-only release.

In `PROVISIONED`, root entries sort uniquely by `trust_root_id`. The union of
every `project_owner_authority_id` and `registry_authority_id` is globally
unique, so the two authority IDs in one root are necessarily different. The
union of every canonically recompressed owner and registry public-key byte
array is also globally unique, so one mathematical key can never occupy both
roles in one root or any roles in different roots. Each public key is exactly
32 bytes encoded as 64 lowercase hex characters.

Production provisioning requires real independently controlled owner and
registry keys. Their private material is never committed and must not be
publicly known. Public Ed25519 test-vector keys, including keys whose matching
private seed/material is published, are prohibited as `PROVISIONED` production
roots. Replacing the initial `UNPROVISIONED` file, changing a provisioned root,
or changing any provisioning/public-root custody evidence requires a forward
plan amendment and independent security review before deployment.

After closed field validation in displayed schema order, semantic checks run
exactly: (1) state/array consistency; (2) for `PROVISIONED`, roots sorted
uniquely by `trust_root_id`; (3) authority-ID global uniqueness in root order,
owner before registry; (4) key hex/length decode in that same order; (5) the
point/canonical/weak-key checks below in that same order; (6) canonical-key
global uniqueness in that same order; and (7) selected-root lookup. The first
failure stops the stage. `UNPROVISIONED` stops after its empty-array check with
`PhysicalApprovalTrustNotProvisioned` when and only when a physical request is
being processed.

Phase E uses exactly
`ed25519-dalek = { version = "=2.2.0", default-features = false }` with no
features; the `std`, `legacy_compatibility`, `batch`, `digest`, `hazmat`,
signing, key-generation, PKCS#8, PEM, random, and Serde surfaces are not used.
At `PROVISIONED` physical-path startup, after store ID/root/authority/key-shape
checks, every decoded key is passed to `ed25519_dalek::VerifyingKey::from_bytes`
in canonical root order and owner before registry order. Its exact input bytes
must then equal `verifying_key.to_edwards().compress().to_bytes()`, and
`is_weak()` must be false. A parse failure is hard
`PhysicalApprovalPublicKeyInvalid`, a recompression mismatch is hard
`PhysicalApprovalNoncanonicalPublicKey`, and `is_weak()==true` is hard
`PhysicalApprovalWeakPublicKey`. Global key distinctness compares these
canonical recompressed arrays only after every key passes those checks. This
validation applies to selected and unused roots and requires no fabricated
signature. For each selected approval signature, exact 64 decoded bytes are
passed to `ed25519_dalek::Signature::try_from`, then
`VerifyingKey::verify_strict(approval_signing_bytes, signature)` must return
`Ok(())`; ordinary `verify`, `verify_batch`, and `ring::signature::ED25519`
are forbidden on this authority path. A signature parse/strict-verification
failure is the role-specific hard `PhysicalApprovalOwnerSignatureInvalid` or
`PhysicalApprovalRegistrySignatureInvalid`. Thus malformed/noncanonical
public-key/signature encodings, noncanonical scalars, weak/low-order public
keys, and weak-key forgeries do not authorize a physical claim. V1 has no
runtime key discovery, environment override, network lookup, validity clock,
dataset-supplied key, signing capability, or test-root selection path.

E-T29's cryptographic known answers use a literal `PROVISIONED`
`PhysicalApprovalTrustStoreV1` only through a pure verifier API exposed to
tests behind an approved test-only boundary. It is neither embedded nor
loadable by the production binary. Tests may supply that literal authority
directly to the pure verifier; the production CLI must always use only the
embedded production store and must prove that no CLI/configuration/environment
route selects or loads a test root. The fixture authority is a software
conformance authority, not a physical-approval authority. It may use published
test-vector public keys only because it cannot authorize a real physical
claim.

Passing E-T29 therefore means only: “the software correctly enforces the
physical-approval contract.” It does not mean that a real physical cohort was
approved or that Phase E is physically validated. `physically_validated`
release language requires real `PROVISIONED` production roots, a real
dual-signed approval, an actual approved physical cohort, passing scientific
endpoints/strata, and the same exact domain authority. No test-only signature
or authority can satisfy a real release gate.

Changing the production store, root field name, key separation rule, verifier
version or features, cryptographic algorithm, or provisioning/public-root
custody requires a forward plan revision, independent security/scientific
review, and a new frozen implementation SHA.

`OwnerApprovalEvidenceV1` contains exactly:

| Field | Exact content |
|---|---|
| `schema_version` | Integer `1`. |
| `approval_record_id` | Domain-separated ID below. |
| `approval_status` | Exact token `approved`. |
| `approval_purpose` | Exact token `pre_scoring_physical_validation_cohort_lock`. |
| `trust_store_id` | Exact token `mhi_physical_approval_trust_store_v1`. |
| `trust_root_id` | Exact ID of one embedded `PhysicalApprovalTrustRootV1`; must equal the protocol's `physical_approval_authority` binding. |
| `project_owner_authority_id` | Exact owner ID from the selected embedded root. |
| `registry_authority_id` | Exact registry ID from the selected embedded root. |
| `owner_authority_document` | `{ immutable_reference_uri, document_sha256 }`, signed as payload authority metadata. |
| `registry_record` | `{ immutable_reference_uri, document_sha256 }`, signed as payload authority metadata. |
| `protocol_sha256` | Exact approved protocol bytes. |
| `cohort_semantic_sha256` | Exact approved section 3.8 cohort preimage. |
| `claim_ids` | Canonical nonempty physical claim IDs; equals the dataset protocol's physical claim set. |
| `endpoint_ids` | Canonical union of all supporting endpoint IDs for `claim_ids`. |
| `reference_authority_ids` | Canonical set union of every allowed authority ID in those endpoint rules and every actual matching reference-endpoint `authority_id` on declared supporting records; actual IDs must already be allowed. |
| `target_domain` | Exact protocol target domain; each physical claim may be narrower but not broader. |
| `physical_origin_confirmed` | Literal `true`. |
| `limitations` | Canonical string list; empty is allowed and means none declared. |
| `owner_signature_ed25519_hex` | Detached Ed25519 signature by the selected owner key over `approval_signing_bytes`; exactly 64 bytes/128 lowercase hex. |
| `registry_signature_ed25519_hex` | Detached Ed25519 signature by the selected registry key over the same bytes; exactly 64 bytes/128 lowercase hex. |

The unsigned approval payload is every field above except
`approval_record_id`, `owner_signature_ed25519_hex`, and
`registry_signature_ed25519_hex`. Signing and identity bytes are exact:

```text
approval_payload_jcs = JCS({
  "identity_domain": "mhi_owner_approval_evidence_v1",
  "schema_version": 1,
  "approval_status": "approved",
  "approval_purpose": "pre_scoring_physical_validation_cohort_lock",
  "trust_store_id": "mhi_physical_approval_trust_store_v1",
  "trust_root_id": <exact ID>,
  "project_owner_authority_id": <exact ID>,
  "registry_authority_id": <exact ID>,
  "owner_authority_document": <exact URI/hash object>,
  "registry_record": <exact URI/hash object>,
  "protocol_sha256": <exact hash>,
  "cohort_semantic_sha256": <exact hash>,
  "claim_ids": <canonical array>,
  "endpoint_ids": <canonical array>,
  "reference_authority_ids": <canonical array>,
  "target_domain": <exact DomainSelectorV1>,
  "physical_origin_confirmed": true,
  "limitations": <canonical array>
})
approval_signing_bytes = UTF8("mhi_owner_approval_signature_v1\0")
                         || approval_payload_jcs
approval_record_id = "sha256:" + SHA256_HEX(approval_signing_bytes)
```

Both signatures are parsed and verified over exactly `approval_signing_bytes`
with the section 3.8 `ed25519-dalek 2.2.0` `verify_strict` sequence; no other
verification API can satisfy this check. The owner ID/key and registry ID/key
come only from the embedded root selected by `trust_root_id`, and the store's
global ID/key distinctness invariants make those authorities cryptographically
separate. Consequently a dataset author cannot create a self-authenticating
physical approval by inventing authority IDs or keys, or satisfy both roles
with one embedded key and copied signature bytes.

The cohort semantic SHA-256 is computed before approval-reference attachment:

```text
SHA256_HEX(JCS({
  "identity_domain": "mhi_validation_cohort_v1",
  "schema_version": 1,
  "dataset_id": ...,
  "protocol_sha256": ...,
  "records": canonical records,
  "reference_sources": canonical reference-source graph,
  "lineage_catalog_source_sha256": ...
}))
```

The dedicated approval reader checks the exact file SHA-256, closed schema,
record ID, embedded trust-store hash/root, both signatures, all bindings, and
source containment before evaluation. A physical
claim with missing, unapproved, mismatched, wrong-origin, or wrong-scope
approval is a hard pre-scoring failure; it cannot silently fall back to a
physical or software pass. If no physical cohort/approval exists, Phase-E
software implementation and E-SW validation remain permitted using a protocol
whose requested claims are explicitly `software`. No actual physical protocol,
dataset, or owner approval is required to authorize software implementation.

After pre-scoring authority succeeds, ordinary partitioning determines actual
overall/stratum record and family counts. Falling below either declared
physical minimum (including the one-family case) makes that endpoint and its
claim `indeterminate` by section 3.7; it is not an approval error and records
are not dropped to manufacture a stronger claim.

Each claim then composes its supporting
endpoint outcomes using section 3.7 and maps them exactly:

| Requested level | Composed endpoint outcome | Release claim outcome |
|---|---|---|
| `software` | `meets_protocol` | `software_validated_only` |
| `software` | `does_not_meet_protocol` | `does_not_meet_protocol` |
| `software` | `indeterminate` | `indeterminate` |
| `physical` | `meets_protocol` | `physically_validated` |
| `physical` | `does_not_meet_protocol` | `does_not_meet_protocol` |
| `physical` | `indeterminate` | `indeterminate` |

The report `overall_status` composes all supporting endpoint outcomes by
section 3.7 over the sorted set union of every claim's supporting endpoint IDs,
evaluating a multiply referenced endpoint once. It remains a
`ValidationOutcomeV1`; it is not inferred from the strongest release-claim
wording.

Two independent families are a minimum claim gate, not evidence of universal
external validity. Every physical claim statement and report row is qualified
by its exact approved domain and required strata; no wording may generalize to
an untested sensor, analyte, matrix, temperature, or campaign.

## 4. Technical objectives

### 4.1 Module plan

All paths in this section are proposed implementation paths. They must not be
created until the review gate in section 8 passes.

| Module | Responsibility |
|---|---|
| `src/mhi_validation/mod.rs` | Narrow facade for protocol validation and the pure evaluation API. No filesystem access in the evaluator. |
| `src/mhi_validation/error.rs` | Closed typed error vocabulary for protocol, dataset, compatibility, overlap, reference, metric, and publication failures. |
| `src/mhi_validation/protocol.rs` | Parse and validate the exact closed TOML structs in section 4.3; reject absent scientific fields, invalid mapping partitions, contradictory rules, and unknown fields. |
| `src/mhi_validation/approval.rs` | Strictly read the embedded production physical-approval trust store and `OwnerApprovalEvidenceV1`; enforce the closed provisioning state, then verify file hash, record ID, both Ed25519 signatures, selected owner/registry root, and exact protocol/cohort/claim bindings. Its pure verifier has one approved test-only literal-trust boundary; production never accepts dataset/protocol/environment/test-supplied keys. |
| `src/mhi_validation/reader.rs` | Use additive `domain::read_artifact_strict` for the dataset and referenced mechanism/health artifacts plus additive `domain::read_artifact_lineage_catalog_strict` for the required catalog; verify duplicate-free/unknown-free bytes, kind, schema, recomputed semantic identity, file hash, declaration equality, combined graph closure, and canonical path containment. |
| `src/mhi_validation/partition.rs` | Apply sections 3.2, 3.5, and 3.6 exactly: endpoint/view membership, source-key deduplication, lineage/reference closure, experiment/family overlap, strata, and total eligible/excluded/not-applicable accounting. No filename or timestamp inference. |
| `src/mhi_validation/mechanism.rs` | Project already-serialized Phase B outcomes into declared validation endpoints and exact counts. It must not call the Phase B assessor. |
| `src/mhi_validation/health.rs` | Compare already-serialized Phase C outcomes with independently declared reference endpoints. It must not call the Phase C assessor. |
| `src/mhi_validation/statistics.rs` | Deterministic count/rate calculations and 95% Wilson intervals; no imputation, weighting, smoothing, or unregistered metric. |
| `src/mhi_validation/assessment.rs` | Apply only explicit protocol acceptance rules and return `meets_protocol`, `does_not_meet_protocol`, or `indeterminate`. |
| `src/mhi_validation/output.rs` | Write the typed report, execution manifest, Markdown summary, and CSV tables to a private staging directory; validate checksums before atomic publication. |
| `src/results/mhi_validation.rs` | Own the two new `VersionedArtifact` payloads and their validation invariants. |
| `src/validation_config.rs` | Clap-neutral protocol/options types and closed wire enums. |
| `src/runners/mhi_validation.rs` | Orchestrate read → compatibility/leakage checks → pure evaluation → atomic publication. |
| `src/cli.rs`, `src/main.rs` | Add the single certified route described below, without changing any existing command. |
| `src/results/artifact_contracts.rs`, `src/domain/artifact.rs` | Register the two additive artifact kinds and add `read_artifact_strict` while leaving existing `read_artifact` behavior/bytes unchanged. |
| `src/domain/lineage.rs`, `src/domain/mod.rs` | Add `read_artifact_lineage_catalog_strict` and extract the current deserialization/validation into a shared internal exact-text catalog parser as section 4.3 specifies; leave existing `read_artifact_lineage_catalog` public read/error behavior unchanged. |
| `config/mhi_physical_approval_trust_store.schema1.json` | Reviewed owner/registry Ed25519 trust roots embedded byte-for-byte; no runtime override or discovery. Required for physical claims only. |
| `Cargo.toml`, `Cargo.lock` | Add direct `ed25519-dalek = { version = "=2.2.0", default-features = false }` solely for `VerifyingKey::{from_bytes,to_edwards,is_weak,verify_strict}`, canonical point recompression, and `Signature::try_from`; add exactly the six frozen lock entries in section 4.6 and no other package/version/checksum drift. Do not promote/use `ring`, enable a Dalek feature, or add signer/private-key API. |
| `tests/`, `tests/fixtures/phase_e/` | Implement exactly the section 7.3 registry, literal fixture inventory, mutation oracles, golden outputs, two-process publication tests, and compatibility guards. Existing tests/fixtures are not rewritten. |

Existing `src/model_validation.rs` and the `ism_model_validation` artifact are
model-workflow validation and remain unchanged. Phase E must not rename or
silently repurpose them.

### 4.2 Proposed certified CLI and APIs

The sole new certified route is proposed as:

```text
electroanalysis validation run \
  --protocol validation_protocol.toml \
  --dataset mhi_validation_dataset.schema1.json \
  --output-dir validation_output \
  [--overwrite]
```

The dataset manifest, not arbitrary directory discovery, declares every input
artifact. The CLI must not accept raw electrochemical data or infer reference
labels.

The minimum library surface is:

```text
domain::read_artifact_strict<T: VersionedArtifact>(
    path: &Path,
) -> Result<StrictArtifactRead<T>, ArtifactError>
StrictArtifactRead<T> {
    artifact: T,
    source_bytes: Vec<u8>,
    source_file_sha256: String,
}
domain::read_artifact_lineage_catalog_strict(
    path: &Path,
) -> Result<StrictLineageCatalogRead, LineageCatalogReadError>
StrictLineageCatalogRead {
    catalog: ArtifactLineageCatalog,
    source_bytes: Vec<u8>,
    source_file_sha256: String,
}
MhiValidationProtocolV1::from_toml(&str) -> Result<Self, MhiValidationError>
MhiValidationProtocolV1::validate(&self) -> Result<(), MhiValidationError>
MhiValidationDatasetV1::validate_structure(&self) -> Result<(), ArtifactError>
MhiValidationDatasetV1::validate_against_protocol(
    &self,
    protocol: &MhiValidationProtocolV1,
    protocol_sha256: &str,
) -> Result<(), MhiValidationError>
PhysicalApprovalTrustStoreV1::from_embedded_bytes(
) -> Result<VerifiedEmbeddedTrustStore, MhiValidationError>
VerifiedEmbeddedTrustStore {
    store: PhysicalApprovalTrustStoreV1,
    source_file_sha256: String,
}
OwnerApprovalEvidenceV1::read_and_validate(
    path,
    expected_file_sha256,
    production_trust_store,
    protocol,
    dataset,
) -> Result<Self, MhiValidationError>
#[cfg(test)]
verify_physical_approval_known_answer_test(
    literal_test_trust_store: &PhysicalApprovalTrustStoreV1,
    approval_signing_bytes: &[u8],
    owner_signature: &[u8],
    registry_signature: &[u8],
) -> Result<(), MhiValidationError>
ValidationInputs::read(protocol, dataset_path) -> Result<Self, MhiValidationError>
evaluate_mhi_validation(
    protocol: &MhiValidationProtocolV1,
    inputs: &ValidationInputs,
) -> Result<MhiValidationReportV1, MhiValidationError>
MhiValidationReportV1::validate_structure(&self) -> Result<(), ArtifactError>
MhiValidationReportV1::validate_against(
    &self,
    protocol: &MhiValidationProtocolV1,
    inputs: &ValidationInputs,
    embedded_trust_store: &PhysicalApprovalTrustStoreV1,
) -> Result<(), MhiValidationError>
run_mhi_validation(options: MhiValidationRunOptions) -> Result<(), RunnerError>
```

`evaluate_mhi_validation` is deterministic and filesystem-free. The reader and
runner own I/O. The production writer uses `domain::write_artifact`; direct
ad-hoc JSON parsing of scientific artifacts is prohibited.

The complete production execution path is frozen as:

1. Clap parses `validation run`; unknown options and missing required paths
   fail before the runner.
2. `MhiValidationRunOptions` performs lexical path-shape checks and rejects
   invalid input/output combinations without opening scientific inputs. It
   never uses lexical normalization as proof of filesystem containment.
3. `protocol::from_toml` loads the exact protocol bytes, calculates their
   SHA-256, and validates the closed schema and every scientific rule. Protocol
   and dataset CLI paths must be existing regular non-symlink files, and their
   canonical paths must be distinct from each other and outside the canonical
   output/stage/backup trees. If and only if at least one claim is physical, the
   binary then strictly validates its embedded production trust-store bytes. An
   `UNPROVISIONED` store returns `PhysicalApprovalTrustNotProvisioned`; only a
   `PROVISIONED` store continues to resolve the declared root ID, before opening
   the dataset or any scientific source.
4. `reader::ValidationInputs::read` uses `domain::read_artifact_strict` for the
   dataset and every present declared mechanism/health path, and
   `domain::read_artifact_lineage_catalog_strict` for the one required catalog. It
   resolves every path against the canonical dataset directory, rejects an
   absolute path, `..`, symlink component, or canonical target outside that
   directory, then verifies kind, allowed schema, file hash, expected identity,
   source-specific semantic-identity recomputation, declaration-to-identity
   equality, combined catalog/reference closure authority, and exact protocol
   hash. Missing referenced files are hard errors; a null endpoint
   path is an explicit endpoint exclusion under section 3.2.
5. For a physical requested claim, `approval` performs every section 3.8 signature and
   binding check before source scoring. For an all-software protocol, approval
   evidence must be null and is not read; the trust-store hash is still
   available as binary configuration but is absent from the software report.
6. `partition` applies the exact membership, deduplication, reference graph,
   separation, and stratum rules. It produces one accounting decision for
   every `(endpoint, view, record)` and retains every primary/secondary reason.
7. `mechanism` and `health` project only serialized Phase B/C outcomes into the
   mutually exclusive count sets defined in sections 3.3 and 3.4.
8. `statistics` derives registered rates and Wilson intervals; `assessment`
   evaluates all explicit rules and applies section 3.7 precedence.
9. `MhiValidationReportV1::validate_structure` checks closed self-contained
   invariants. `validate_against` then rereads the normalized protocol authority,
   inputs, and embedded trust store to reconstruct membership, counts, rules,
   outcomes, claims, provenance, and lineage from consumed dependencies.
10. `output` writes the exact section 4.4 bytes to the fixed private staging
   path, rereads the scientific artifact with `read_artifact_strict`, and verifies every
   managed path/checksum before publication.
11. Publication follows the section 4.5 exclusive-lock, no-replace/exchange,
    directory-fsync, staging-cleanup, residue, and committed-cleanup outcomes.

Hard-error precedence is deterministic and completes one numbered stage before
the next:

1. CLI option and lexical input/output path shape;
2. protocol UTF-8/TOML parse, closed-schema, then semantic validation;
3. for physical protocols only, embedded production trust-store structure/hash
   in exact field order, then provisioning-state/array consistency; an
   `UNPROVISIONED` store returns `PhysicalApprovalTrustNotProvisioned`, while a
   `PROVISIONED` store continues through canonical roots, globally distinct
   authority IDs, globally canonical point-recompression plus weak-key
   validation, globally distinct canonical public-key bytes, then selected-root
   lookup;
4. dataset I/O, duplicate scan, kind/schema, source-file SHA-256 computation,
   closed structure,
   protocol binding, and cohort-hash recomputation, in that order;
5. lexical/canonical containment and symlink checks in this reference order:
   lineage catalog, scientific sources by the stage-7 key, then physical
   approval;
6. lineage-catalog exact-byte read → recursive duplicate scan → nested
   unknown-field/tag scan → existing canonical parse/validation → file hash;
7. present scientific source files in `(record_id, source_role)` order, where
   `mechanism_source < health_source`; for each file complete I/O → duplicate
   scan → kind/schema → file hash → producer semantic-identity recomputation →
   expectation/declaration equality before advancing;
8. catalog root and combined reference-graph structural bindings;
9. for physical claims, declared-record Known-source/reference presence →
   physical origin → mechanism semantic-outcome availability → reference
   method → authority → blinding →
   quantified uncertainty → reference-node completeness, followed by
   approval I/O → duplicate scan → schema/file hash/record ID → owner
   signature → registry signature → protocol/cohort/claim/endpoint/domain/
   origin/authority bindings, in that order; and
10. output lock/recovery/preflight.

Within a stage, canonical collection order from section 4.3 selects the first
error. A raw JSON/TOML syntax error uses the parser's lowest byte offset; a JSON
duplicate uses the lowest byte offset of a repeated key's second occurrence;
after successful parsing, field checks use the field order explicitly listed in
the corresponding schema. The first hard error stops the run. Scientific exclusions and outcomes
use section 3.7 and are serialized; the exhaustive condition matrix in section
4.3 prevents a condition from being both a hard error and an exclusion.

### 4.3 Schema definitions

All schema-1 objects and every nested object are closed
(`deny_unknown_fields` or an equivalent duplicate-key-aware custom reader).
Every JSON field listed by a schema is present: nullable fields serialize as
explicit `null`, never omission, and no `skip_serializing_if` behavior is
permitted. TOML has no null; protocol alternatives use the required tagged
objects below, and no unlisted field may be omitted as an implicit default.
All owner-assigned IDs match `^[A-Za-z0-9][A-Za-z0-9._:-]*$`. Human-readable
text is valid UTF-8, has no NUL or CR, and has no trailing whitespace per line. Every
SHA-256 is exactly 64 lowercase hexadecimal characters; artifact IDs use
`sha256:<64 lowercase hex>`. All f64 values must be finite, and an input f64
whose bits represent negative zero is invalid in every field. Tokens are exact
snake case. Authority URI/reference strings are opaque printable ASCII and must
match `^[A-Za-z][A-Za-z0-9+.-]*:[!-~]+$`; no network lookup or URI
normalization changes identity. Unit and method/version strings compare by
exact UTF-8 bytes; Phase E performs no unit conversion or method aliasing.

Canonical order for every unordered vector is ascending lexicographic order of
the stated tuple, comparing strings by raw UTF-8 bytes and enums by their wire
token. Adjacent equal keys are duplicates and invalid; readers do not sort or
deduplicate malformed input. The common keys are:

| Collection | Exact canonical sort/uniqueness key |
|---|---|
| endpoints | `(endpoint_id)` |
| release claims | `(claim_id)` |
| rules | `(rule_id)` |
| required strata | `(stratum_id)` |
| dataset records | `(record_id)` |
| reference endpoints | `(endpoint_id, reference_endpoint_id)` |
| reference-source nodes | `(reference_source_id)` |
| direct dependencies | `(dependency_variant, source_id_or_scientific_source_key)` |
| result views | `(endpoint_id, stratum_id)` with `overall` first, then UTF-8 order |
| exclusions/accounting/leakage | `(endpoint_id, stratum_id, record_id, reason_token)` |
| compatibility rows | `(source_role discriminant, record_id-or-empty, relative_path)`; role order is protocol, dataset, lineage_catalog, owner_approval, mechanism_source, health_source |
| source references | `(variant_discriminant, kind_or_source_type, stable_id, sha256)` |
| warnings | `(code, related_id, detail)` |
| limitations | raw UTF-8 string bytes |
| exclusion reasons | `(section 3.7 exclusion ordinal, wire token, stable related ID)` |
| separation unknown reasons | `(section 3.1 token-list discriminant, stable related ID)` |
| endpoint/outcome reasons | `(section 3.7 endpoint ordinal, rule ID or wire token, stable related ID)` |

TOML field order does not change parsed meaning, but `protocol_sha256` always
hashes the exact original UTF-8 bytes, including whitespace and comments.
Duplicate TOML keys, invalid UTF-8, or a byte-order mark are hard parse errors.

Phase E does not change the legacy behavior of `domain::read_artifact`.
Instead it adds `domain::read_artifact_strict<T: VersionedArtifact>`, which:

1. opens one regular non-symlink file and reads its exact bytes once;
2. rejects BOM/invalid UTF-8/nonfinite tokens;
3. uses a recursive JSON map visitor that retains a key set for every object
   and rejects a duplicate before constructing any `serde_json::Value`;
4. on the resulting duplicate-free value invokes the same existing
   `validate_value::<T>`, typed deserialization, and `validate_after_read`
   sequence used by `read_artifact`; and
5. returns the typed value, exact source bytes, and their SHA-256 so no second
   path read or time-of-check/time-of-use substitution is possible.

All Phase-E JSON artifacts, including consumed Phase-B/C artifacts, use this
strict function. Phase E also adds
`domain::read_artifact_lineage_catalog_strict`, because the existing catalog
reader rejects root unknown fields and duplicate keys but the existing nested
lineage structs intentionally do not reject unknown fields. The strict catalog
reader:

1. reads one regular non-symlink file into exact bytes once and computes its
   SHA-256;
2. runs the same recursive duplicate-key scanner used by
   `read_artifact_strict` over every object;
3. rejects any object key outside this exact existing-wire grammar:
   - catalog root: `schema_version`, `artifacts`;
   - each arbitrary artifact-ID map value:
     `identity`, `direct_dependencies`;
   - `ArtifactIdentity`: `artifact_id`, `artifact_kind`, `schema_version`,
     `producer_version`, `experiment_scope`, `sensor_scope`, `channel_scope`,
     `acquisition_families`, `semantic_sha256`;
   - each `ArtifactDependency`: `artifact_id`, `artifact_kind`, `role`;
   - `ArtifactExperimentScope`: exact unit string `"Unknown"`, or a one-key
     external tag `"Single"` whose payload contains only `experiment_id`, or
     `"Aggregate"` whose payload contains only `aggregate_scope_id` and
     `member_experiment_ids`;
   - `ScopeKey`: exact unit string `"All"`/`"Unspecified"`, or a one-key
     external tag `"Specific"` whose value is a string and introduces no
     nested object; and
   - `ArtifactAcquisitionFamilies`: exact unit string `"Unknown"`, or a
     one-key external tag `"Known"` whose value is an array and introduces no
     nested object;
4. validates UTF-8 without transforming it (invalid UTF-8 maps to the existing
   `LineageCatalogReadError::Io` with `ErrorKind::InvalidData`, matching
   `read_to_string`) and passes the `&str` backed by those same exact bytes to a shared internal
   `parse_artifact_lineage_catalog_text` containing the current canonical
   reader's existing type/shape/schema/key-identity/semantic validation; and
5. returns catalog, bytes, and hash as `StrictLineageCatalogRead`.

This is domain-owned strict validation, not Phase-E ad-hoc parsing. The
existing public `read_artifact_lineage_catalog(path)` retains its current
`fs::read_to_string` boundary and invokes the same internal text parser without
step 3; its acceptance, errors, and output remain variant-for-variant
compatible. The
strict function alone rejects nested unknown fields. Compatibility tests call
both readers on the nested-unknown mutations and prove the old API retains its
baseline result while the Phase-E API rejects. Tests likewise prove the old
`read_artifact` public API, existing accept/reject matrix, and writer bytes are
unchanged.

For every scoreable Known mechanism/health source, the reader recomputes the
producer-owned identity by calling `known_lineage_from_artifact` with the typed
artifact, its embedded identity's artifact kind/schema/producer version/scope/
family fields, and its embedded direct dependencies. That existing function's
scientific view removes only `lineage`, `schema_version`, `artifact_kind`,
`warnings`, and the three documented operational provenance path/time fields.
The recomputed `ArtifactIdentity` and sorted direct dependencies must equal the
embedded values byte-for-byte and must equal `ArtifactSourceExpectationV1`;
otherwise `SourceSemanticIdentityMismatch` is a hard error. Comparing an
embedded hash to itself is forbidden.

#### Closed common and protocol wire types

Every new Phase-E tagged union in sections 3.8, 4.3, and 4.4 is an internally
tagged object whose discriminator field is exactly `type`; shorthand such as
`available { ... }` means `{ type="available", ... }`. No external/untagged
representation is allowed. Only explicitly labeled existing Phase-A/B/C/D
wire enums retain their existing representation.

```text
CategoricalSelectorV1 =
  { type="any_declared" }
  | { type="allowed", ids=[nonempty canonical IDs] }

TemperatureSelectorV1 =
  { type="any_declared" }
  | { type="bands", bands=[canonical TemperatureBandV1] }

TemperatureBandV1 = {
  lower_kelvin_inclusive: finite positive f64,
  upper_kelvin_exclusive: finite positive f64
}

DomainSelectorV1 = {
  analyte, matrix, sensor_design, sensor, campaign: CategoricalSelectorV1,
  temperature: TemperatureSelectorV1
}

DomainKeyV1 = {
  analyte_id, matrix_id, sensor_design_id, sensor_id, campaign_id,
  temperature_kelvin: finite positive f64
}

HealthTargetV1 =
  { type="dimension", dimension_id: one of signal_integrity,
    calibration_health, dynamic_response_health, reference_stability,
    environmental_robustness, model_consistency, observability,
    uncertainty_health, data_quality }
  | { type="aggregate" }

ReferenceMethodV1 = { method_id:nonempty ID, method_version:nonempty text }

ReferenceUncertaintyRuleV1 =
  { type="require_quantified", measure_id:nonempty ID, unit:nonempty text,
    maximum_inclusive: finite nonnegative f64 }
  | { type="allow_unavailable_with_limitation" }

PhysicalApprovalAuthorityV1 =
  { type="not_requested" }
  | { type="embedded_trust_root", trust_root_id }

MechanismReferenceRuleV1 = {
  type="mechanism", allowed_methods:[ReferenceMethodV1],
  allowed_authority_ids:[ID], blinding_rule:BlindingRuleV1,
  uncertainty_rule:ReferenceUncertaintyRuleV1
}

HealthReferenceRuleV1 = {
  type="health", allowed_methods:[ReferenceMethodV1],
  allowed_authority_ids:[ID], blinding_rule:BlindingRuleV1,
  uncertainty_rule:ReferenceUncertaintyRuleV1
}

StratumPredicateV1 =
  { type="analyte_equals", id }
  | { type="matrix_equals", id }
  | { type="sensor_design_equals", id }
  | { type="sensor_equals", id }
  | { type="campaign_equals", id }
  | { type="temperature_band", lower_kelvin_inclusive,
      upper_kelvin_exclusive }

RequiredStratumV1 = {
  stratum_id, predicates:[StratumPredicateV1],
  minimum_eligible_records:u64,
  minimum_independent_families:u64
}

CountMetricV1 = one of:
  declared_count, eligible_count, excluded_count, not_applicable_count,
  independent_family_count, support_count,
  critical_contradiction_count, not_assessed_or_other_count, tp, tn, fp, fn,
  indeterminate_count, data_quality_insufficient_count, evaluable_count

RateMetricV1 = one of:
  exclusion_rate, support_fraction, contradiction_fraction,
  not_assessed_fraction, coverage,
  indeterminate_rate, data_quality_insufficient_rate, sensitivity, specificity,
  false_positive_rate, false_negative_rate, balanced_accuracy

AcceptanceRuleV1 =
  { type="count", rule_id, metric:CountMetricV1,
    comparator:ComparatorV1, threshold_u64:u64 }
  | { type="rate", rule_id, metric:RateMetricV1,
    target:RateTargetV1, comparator:ComparatorV1,
    threshold:finite f64 in [0,1] }

MechanismEndpointV1 = {
  endpoint_id, hypothesis_id, cohort_role, domain,
  mechanism_artifact_required:true, reference_rule:MechanismReferenceRuleV1,
  support_levels:[HypothesisEvidenceLevel],
  critical_policy="any_contradicted_record_fails",
  minimum_eligible_records:u64, minimum_independent_families:u64,
  required_strata:[RequiredStratumV1], acceptance_rules:[AcceptanceRuleV1]
}

HealthEndpointV1 = {
  endpoint_id, target:HealthTargetV1, cohort_role, domain,
  health_artifact_required:true, reference_rule:HealthReferenceRuleV1,
  predicted_positive_statuses:[existing OverallHealthStatus tokens],
  predicted_negative_statuses:[existing OverallHealthStatus tokens],
  reference_label_universe:[ID], reference_positive_labels:[ID],
  reference_negative_labels:[ID], minimum_eligible_records:u64,
  minimum_independent_families:u64, required_strata:[RequiredStratumV1],
  acceptance_rules:[AcceptanceRuleV1]
}

ReleaseClaimV1 = {
  claim_id, requested_level:RequestedValidationLevelV1, statement,
  domain:DomainSelectorV1, supporting_endpoint_ids:[ID]
}
```

Every listed vector is required, duplicate-free, and canonical by section 4.3.
Allowed methods sort by `(method_id, method_version)`; bands sort by lower then
upper bit value; stratum predicates sort by the discriminant order in section
3.6. Endpoint IDs are globally unique across mechanism and health lists.
Mechanism-only metrics are invalid on health endpoints and conversely; the
shared `eligible_count`/`independent_family_count` are allowed on both.
Acceptance-rule lists are nonempty and cannot make a performance endpoint
vacuously pass. Every mechanism endpoint contains at least one
`support_fraction` rule with comparator `greater_than_or_equal`. Every health
endpoint contains at least one `coverage`, one `sensitivity`, and one
`specificity` rule, each with comparator `greater_than_or_equal`. Their
registered target and threshold remain explicit protocol authority; additional
valid rules are allowed and all rules compose by AND. This makes a zero class
denominator indeterminate and prevents an eligible-count-only protocol from
authorizing a validation claim.

#### `MhiValidationProtocolV1` — TOML schema 1

| Field | Required content |
|---|---|
| `schema_version` | Exactly `1`. |
| `protocol_id` | Stable nonempty owner-assigned ID. |
| `title` | Human-readable protocol title. |
| `registration` | `ProtocolRegistrationV1 { registration_id, immutable_reference_uri, document_sha256 }`; all fields nonempty/valid. This registers the question before scoring, not owner approval of a physical cohort. |
| `physical_approval_authority` | Exact `PhysicalApprovalAuthorityV1`; `not_requested` iff all claims are software, otherwise `embedded_trust_root` with one ID that must exist in the binary store. The protocol supplies only the ID, never a key. |
| `target_domain` | One exact `DomainSelectorV1` from section 3.6; every axis present. |
| `mechanism_endpoints` | Sorted nonempty `MechanismEndpointV1` list. |
| `health_endpoints` | Sorted nonempty `HealthEndpointV1` list. |
| `statistics` | Exactly the registered interval method (`wilson_95_v1`) and explicit missing/indeterminate handling. |
| `release_scope` | Sorted nonempty `ReleaseClaimV1` list. |

`MechanismEndpointV1` and `HealthEndpointV1` use exactly the closed definitions
above. The health class partitions must satisfy section 3.4 exactly.

Every endpoint reference rule contains the exact allowed method ID/version
pairs, authority IDs, blinding rule, and uncertainty rule from section 3.5.
Physical release claims require `require_blinded` and `require_quantified`;
software claims may use either declared rule. Every endpoint `cohort_role` is
validation or holdout; development is rejected. Allowed methods and authorities
are nonempty canonical lists. A mechanism rule's admitted semantic outcomes are
fixed by section 3.3 and therefore have no wire field.

`ReleaseClaimV1` contains exactly: claim ID, `requested_level`, claim statement,
claim domain selector, and a sorted nonempty set of supporting endpoint IDs.
Every endpoint domain and claim domain must be equal to or narrower than the
protocol `target_domain`. In addition, each supporting endpoint domain must be
exactly equal to its claim domain: protocol validation proves both
`endpoint_domain <= claim_domain` and `claim_domain <= endpoint_domain` using
section 3.6's categorical-set/temperature-union subset relation. One-sided
containment in either direction is rejected. This prevents a broader claim
from inheriting evidence for an untested analyte/range and prevents a narrower
claim from inheriting a pooled endpoint result whose unreported complement
could conceal failure. Physical claims may reference only holdout endpoints;
software claims may reference validation or holdout endpoints. A supporting
endpoint may serve multiple claims only when those claim domains are exactly
equal, and every endpoint must support at least one claim. Protocol validation rejects a
physical claim if either `minimum_eligible_records` or
`minimum_independent_families` is below two on any supporting endpoint overall
or required stratum. All software minima must still be at least one.

`StatisticsV1` contains exactly `interval_method="wilson_95_v1"`,
`confidence_level="0.95"`, `undefined_metric="unavailable"`,
`required_rule_unavailable="indeterminate"`, and
`rule_composition="and"`. These fields are required equality guards, not
configurable scientific alternatives. There are no scientific defaults.

#### `MhiValidationDatasetV1` — artifact kind `mhi_validation_dataset`, schema 1

| Field | Required content |
|---|---|
| `schema_version`, `artifact_kind` | Exactly `1` and `mhi_validation_dataset`. |
| `dataset_id` | Stable nonempty ID. |
| `protocol_sha256` | SHA-256 of the exact protocol bytes used to construct the manifest. |
| `cohort_semantic_sha256` | Exact section 3.8 domain-separated JCS hash. |
| `lineage_catalog_source` | Required `LineageCatalogSourceV1 { relative_path, schema_version=1, source_file_sha256 }`. |
| `reference_sources` | Canonical closed `ReferenceSourceAuthorityV1` graph from section 3.5. |
| `records` | Canonically ordered `ValidationRecordV1` values. |
| `owner_approval_source` | Null iff every release claim requests software; otherwise required `OwnerApprovalSourceV1 { relative_path, schema_version=1, source_file_sha256, expected_approval_record_id }`. |
| `lineage` | Known aggregate identity and direct dependencies when authoritative; otherwise explicit `LegacyUnknown`. |
| `provenance` | Manifest input/config hashes, software version, generation time, and optional Git commit. |
| `warnings` | Typed, ordered construction warnings; warnings cannot waive invalidity. |

`ValidationRecordV1` contains exactly:

| Field | Exact type/meaning |
|---|---|
| `record_id` | Unique nonempty ID. |
| `cohort_role` | One `CohortRoleV1`; no role lists or aliases. |
| `mechanism_source` | Null or `ArtifactSourceExpectationV1` fixed to kind `mechanism_analysis`. |
| `health_source` | Null or `ArtifactSourceExpectationV1` fixed to kind `health_assessment`. |
| `declared_scope` | `DeclaredScopeV1 { experiment_scope, sensor_scope, channel_scope, acquisition_families }`; Known source values must equal serialized identity; all fields `unknown` for LegacyUnknown. |
| `domain` | `DomainKeyV1 { analyte_id, matrix_id, sensor_design_id, sensor_id, temperature_kelvin, campaign_id }`; every value present and temperature finite/positive. |
| `evidence_origin` | One explicit `EvidenceOriginV1`; `physical` requires reference-source physical origin and physical approval binding before a physical claim. |
| `reference_endpoints` | Canonical list of closed mechanism/health reference variants, each bound to one protocol endpoint and one `reference_source_id`. |

The nested dataset wire types are exactly:

```text
Existing ArtifactExperimentScope wire =
  { "Single": { "experiment_id": ID } }
  | { "Aggregate": { "aggregate_scope_id": "sha256:<64 lowercase hex>",
                       "member_experiment_ids": [at least two canonical IDs] } }
  | "Unknown"

Existing ScopeKey wire = { "Specific": ID } | "All" | "Unspecified"

Existing ArtifactAcquisitionFamilies wire =
  { "Known": [nonempty canonical acquisition-family IDs] } | "Unknown"

DeclaredScopeV1 = {
  experiment_scope: existing ArtifactExperimentScope wire,
  sensor_scope: existing ScopeKey wire,
  channel_scope: existing ScopeKey wire,
  acquisition_families: existing ArtifactAcquisitionFamilies wire
}

ScientificSourceKeyV1 =
  { type="known", artifact_kind, artifact_id, semantic_sha256 }
  | { type="legacy_unknown", artifact_kind, schema_version,
      source_file_sha256 }

ReferenceDependencyV1 =
  { type="reference_source", reference_source_id }
  | { type="scientific_artifact", source:ScientificSourceKeyV1 }

ReferenceSourceAuthorityV1 = {
  reference_source_id, source_file_sha256, evidence_origin,
  dependency_completeness,
  experiment_scope: existing ArtifactExperimentScope wire,
  acquisition_families: existing ArtifactAcquisitionFamilies wire,
  direct_dependencies:[ReferenceDependencyV1]
}

ReferenceUncertaintyV1 =
  { type="quantified", measure_id:nonempty ID,
    finite_value:finite nonnegative f64,
    unit:nonempty text }
  | { type="unavailable", nonempty_reason }

MechanismReferenceV1 = {
  type="mechanism", endpoint_id, reference_endpoint_id, hypothesis_id,
  outcome: one of supports|contradicts|not_assessed|unavailable,
  reference_source_id, method:ReferenceMethodV1, authority_id,
  blinding_state, uncertainty:ReferenceUncertaintyV1,
  limitations:[string]
}

HealthReferenceV1 = {
  type="health", endpoint_id, reference_endpoint_id, target:HealthTargetV1,
  label, reference_source_id, method:ReferenceMethodV1, authority_id,
  blinding_state, uncertainty:ReferenceUncertaintyV1,
  limitations:[string]
}

ReferenceEndpointV1 = MechanismReferenceV1 | HealthReferenceV1
```

For each `(record_id, endpoint_id)`, zero or one matching reference endpoint is
allowed structurally. Zero yields `missing_reference_endpoint` when that
record is declared for the endpoint/view; section 3.8 instead makes zero a hard
pre-scoring failure when the endpoint supports a physical claim. More than one
is always a hard duplicate. For every present endpoint, the pair
`(endpoint_id, reference_endpoint_id)` is unique within the record, and
`reference_endpoint_id` is globally unique within that record. Mechanism
`hypothesis_id` must equal the protocol endpoint hypothesis; health target must
exactly equal the protocol endpoint target. Variant and endpoint kind must
agree. A present cross-bound or mismatched reference endpoint is hard
`ReferenceEndpointBindingMismatch`; no case is resolved by list order.

The existing lineage wire enums above retain their exact Phase-A/Phase-D
serialization. A scoreable Known source and physical reference authority must
use non-Unknown scope/families. LegacyUnknown declarations must use the existing
Unknown variants exactly; a dataset may not relabel them Known.

`ArtifactSourceExpectationV1` contains exactly: safe relative path, expected
artifact kind, expected schema version, source-file SHA-256, and
`ExpectedLineageV1`. The latter is tagged as:

```text
known { artifact_id, semantic_sha256 }
legacy_unknown { schema_version, legacy_source_fingerprint,
                 reason:LegacyLineageReasonV1 }
```

`legacy_source_fingerprint` is exactly the lowercase 64-hex SHA-256 of the
source bytes. The three V1 reason tokens map one-to-one, in listed order, to the
existing `UnknownLineageReason` variants without changing their existing wire
serialization.

Current scoreable sources must be Known schema 4. Readable legacy sources may
use `legacy_unknown` and receive typed exclusions; a current source serialized
as LegacyUnknown is not scoreable and produces unknown separation. For Known,
the reader performs the producer-owned semantic recomputation defined above;
artifact ID must equal `sha256:<recomputed semantic_sha256>`, embedded identity,
catalog root, expectation, dependencies, and all declared scope/family values
must agree exactly.

`ReferenceSourceAuthorityV1` and reference endpoint variants contain exactly
the fields above. They are provenance/outcome authority embedded in the dataset;
they do not authorize physical origin without section 3.8 signatures. Reference
source dependencies sort by the variant order shown, then reference-source ID
or the complete `ScientificSourceKeyV1` tuple.

All relative input paths must use `/`, must not be empty, `.`, contain an empty
component, `.` component, `..` component, Windows prefix, or NUL, and must not
name a symlink at any component. After opening the canonical dataset directory,
the reader resolves the canonical target and requires it to remain beneath
that directory. Absolute paths and lexical/canonical escapes are hard failures.

#### `MhiValidationReportV1` — artifact kind `mhi_validation_report`, schema 1

| Field | Required content |
|---|---|
| `schema_version`, `artifact_kind` | Exactly `1` and `mhi_validation_report`. |
| `report_id` | Exact domain-separated semantic ID defined below. |
| `protocol` | Exact `ProtocolAuthorityV1`: protocol ID, schema, SHA-256, registration, tagged physical-approval authority, and the complete normalized `MhiValidationProtocolV1`. |
| `dataset` | Exact `DatasetAuthorityV1`: dataset ID/schema, protocol/cohort hashes, and the canonical tagged source reference. |
| `approval` | Null when all requests are software; otherwise exact `ApprovalAuthorityV1` below, including embedded trust-store/root and both verified signature bindings. |
| `compatibility` | Canonical `CompatibilityRowV1` array; a published report is compatible iff every row has one of the three non-hard results below. |
| `record_accounting` | One canonical row for every endpoint/view/record with decision, primary/secondary reasons, assessed source key, and reference endpoint ID or null. |
| `cohorts` | Per endpoint/view exact declared, eligible, excluded, not-applicable, development, validation, and holdout ID sets and counts. |
| `leakage_assessment` | Per endpoint/view/record separation status, compared development IDs, shared artifact/source/experiment/family IDs, unknown reasons, and resulting decision. |
| `mechanism_results` | One result per mechanism endpoint/view with exact category ID sets/counts, family IDs/count, rate values/intervals, rule evaluations, exhaustive ordered outcome reasons, limitations, and outcome. |
| `health_results` | One result per health endpoint/view with exact six category ID sets/counts, family IDs/count, every metric, rule evaluations, exhaustive ordered outcome reasons, limitations, and outcome. |
| `exclusions` | Canonical endpoint/view/record primary and secondary reason projection containing exactly accounting rows whose decision is `excluded`. |
| `release_claims` | Per requested claim: `physically_validated`, `software_validated_only`, `does_not_meet_protocol`, or `indeterminate`, plus exact supporting endpoint IDs. |
| `overall_status` | Exact section 3.7 composition. |
| `lineage`, `provenance`, `warnings` | Direct dependencies for every actually consumed artifact; deterministic software/Git/configuration identity with no wall clock or output path; closed typed warnings. |

The report's nested wire records are exactly:

```text
ProtocolAuthorityV1 = {
  protocol_id, schema_version:1, source_file_sha256, registration,
  physical_approval_authority:PhysicalApprovalAuthorityV1,
  normalized_protocol:MhiValidationProtocolV1
}

DatasetAuthorityV1 = {
  dataset_id, schema_version:1, protocol_sha256, cohort_semantic_sha256,
  source:DatasetSourceReferenceV1
}

ApprovalAuthorityV1 = {
  approval_source_file_sha256, approval_record_id, trust_store_id,
  approval_purpose:"pre_scoring_physical_validation_cohort_lock",
  trust_store_sha256, trust_root_id, project_owner_authority_id,
  registry_authority_id,
  owner_authority_document:{immutable_reference_uri,document_sha256},
  registry_record:{immutable_reference_uri,document_sha256},
  owner_signature_verified:true,
  registry_signature_verified:true, binding_status:"verified",
  limitations:[string]
}

CompatibilityResultV1 = one of:
  compatible, readable_legacy_excluded, current_legacy_unknown_excluded

CompatibilityRowV1 = {
  record_id:null|ID,
  source_role: one of protocol|dataset|lineage_catalog|owner_approval|
               mechanism_source|health_source,
  relative_path,
  expected_kind:null|ArtifactKind, actual_kind:null|ArtifactKind,
  expected_schema:u64, actual_schema:u64,
  expected_file_sha256, actual_file_sha256,
  expected_artifact_id:null|ArtifactId, actual_artifact_id:null|ArtifactId,
  expected_semantic_sha256:null|sha256, actual_semantic_sha256:null|sha256,
  result:CompatibilityResultV1
}

RecordAccountingRowV1 = {
  endpoint_id, stratum_id, record_id, decision:RecordDecisionV1,
  primary_reason:null|ExclusionReasonV1,
  secondary_reasons:[ExclusionReasonV1],
  assessed_source_key:null|ScientificSourceKeyV1,
  reference_endpoint_id:null|ID
}

CohortRowV1 = {
  endpoint_id, stratum_id, endpoint_kind, cohort_role,
  declared_record_ids:[ID], eligible_record_ids:[ID],
  excluded_record_ids:[ID], not_applicable_record_ids:[ID],
  development_record_ids:[ID], validation_record_ids:[ID],
  holdout_record_ids:[ID], declared_count, eligible_count, excluded_count,
  not_applicable_count, exclusion_rate:MetricValueV1,
  evaluable_count:null|u64, indeterminate_count:null|u64,
  data_quality_insufficient_count:null|u64,
  coverage:null|MetricValueV1, indeterminate_rate:null|MetricValueV1,
  data_quality_insufficient_rate:null|MetricValueV1,
  outcome:ValidationOutcomeV1
}

LeakageRowV1 = {
  endpoint_id, stratum_id, record_id,
  separation_status:null|SeparationStatusV1,
  not_evaluated_reason:null|one of not_applicable|
    missing_endpoint_artifact_path|missing_reference_endpoint,
  compared_development_record_ids:[ID], shared_artifact_ids:[ArtifactId],
  shared_source_sha256s:[sha256], shared_experiment_ids:[ID],
  shared_family_ids:[ID], unknown_reasons:[closed reason token],
  decision:RecordDecisionV1
}

RuleEvaluationV1 = {
  rule:AcceptanceRuleV1,
  actual: { type="count", value:u64 }
          | { type="binomial_rate", value:MetricValueV1 }
          | { type="balanced_accuracy", value:BalancedAccuracyV1 },
  result: RuleEvaluationResultV1
}

OutcomeReasonV1 =
  { type="holdout_known_overlap", record_id }
  | { type="holdout_unknown_separation", record_id }
  | { type="declared_critical_falsification", record_id }
  | { type="empty_view" }
  | { type="eligible_record_minimum_not_met", actual:u64, minimum:u64 }
  | { type="independent_family_minimum_not_met", actual:u64, minimum:u64 }
  | { type="required_stratum_indeterminate", stratum_id }
  | { type="reference_uncertainty_unavailable", record_id }
  | { type="required_rule_unavailable", rule_id }
  | { type="required_rule_false", rule_id }

MechanismResultV1 = {
  endpoint_id, stratum_id, eligible_record_ids:[ID],
  eligible_family_ids:[ID], support_record_ids:[ID],
  critical_contradiction_record_ids:[ID],
  declared_critical_falsification_record_ids:[ID],
  not_assessed_or_other_record_ids:[ID],
  eligible_count, independent_family_count, support_count,
  critical_contradiction_count, declared_critical_falsification_count,
  not_assessed_or_other_count,
  support_fraction:MetricValueV1, contradiction_fraction:MetricValueV1,
  not_assessed_fraction:MetricValueV1, rule_evaluations:[RuleEvaluationV1],
  outcome_reasons:[OutcomeReasonV1], limitations:[string],
  outcome:ValidationOutcomeV1
}

HealthResultV1 = {
  endpoint_id, stratum_id, eligible_record_ids:[ID],
  eligible_family_ids:[ID], tp_record_ids:[ID], tn_record_ids:[ID],
  fp_record_ids:[ID], fn_record_ids:[ID], indeterminate_record_ids:[ID],
  data_quality_insufficient_record_ids:[ID], eligible_count,
  independent_family_count, tp, tn, fp, fn, indeterminate,
  data_quality_insufficient, evaluable,
  coverage:MetricValueV1, indeterminate_rate:MetricValueV1,
  data_quality_insufficient_rate:MetricValueV1,
  sensitivity:MetricValueV1, specificity:MetricValueV1,
  false_positive_rate:MetricValueV1, false_negative_rate:MetricValueV1,
  balanced_accuracy:BalancedAccuracyV1,
  rule_evaluations:[RuleEvaluationV1], outcome_reasons:[OutcomeReasonV1],
  limitations:[string],
  outcome:ValidationOutcomeV1
}

ExclusionRowV1 = {
  endpoint_id, stratum_id, record_id, primary_reason:ExclusionReasonV1,
  secondary_reasons:[ExclusionReasonV1],
  assessed_source_key:null|ScientificSourceKeyV1,
  reference_endpoint_id:null|ID
}

ReleaseClaimResultV1 = {
  claim_id, requested_level, statement, domain:DomainSelectorV1,
  supporting_endpoint_ids:[ID], approval_record_id:null|ArtifactId,
  outcome:ReleaseClaimOutcomeV1
}

ValidationWarningV1 = {
  code: one of legacy_source_excluded|reference_uncertainty_unavailable|
               declared_source_missing|physical_scope_limitation,
  related_id, detail
}
```

`outcome_reasons` is the duplicate-free exhaustive projection of every true
condition at section 3.7 endpoint ordinals 2–8; a pass has an empty list. Emit
one record-bearing reason per affected record, one minimum reason for each
failed minimum (with `empty_view` additionally when eligible count is zero),
one `required_stratum_indeterminate` per affected stratum on the overall
result, and one rule reason per affected rule. Sort by section 3.7 ordinal;
within ordinal 5 use `empty_view`, eligible-record minimum, independent-family
minimum, then required stratum; within a variant sort by record, stratum, or
rule ID and then numeric fields. Thus a higher-precedence outcome never erases
lower-precedence diagnostic reasons.

For `RecordAccountingRowV1`, `assessed_source_key` is null exactly when the row
is `not_applicable` or its declared endpoint source path is null; otherwise a
present strict-reader-valid Known or LegacyUnknown source supplies its exact
tagged key even when the row is excluded. `reference_endpoint_id` is null
exactly when the row is `not_applicable` or no matching endpoint is present;
otherwise the present bound ID is retained even when another eligibility rule
excludes the row. Physical cases that section 3.8 classifies as hard never
produce an accounting row or report.

The top-level arrays use the names in the field table and the exact nested
types above. `compatibility`, `record_accounting`, `cohorts`,
`leakage_assessment`, `mechanism_results`, `health_results`, `exclusions`,
`release_claims`, and `warnings` are always present, including as empty arrays.
`ProtocolAuthorityV1.normalized_protocol` must round-trip to the same normalized
value as the hashed TOML; report validation rejects a hash/snapshot mismatch.

Every derived count is validated against its sorted ID set. Every rate is a
tagged `MetricValueV1`:

```text
available { numerator, denominator, point_estimate,
            interval: { method="wilson_95_v1",
                        confidence="0.95", lower, upper } }
unavailable { numerator, denominator, reason }
```

Available numerator/denominator must reproduce the point estimate exactly
under section 3.7 f64 operations. An unavailable metric has no numeric point or
interval and its reason is the applicable exact `MetricUnavailableReasonV1`:
generic fractions use `denominator_zero`; sensitivity/FNR use
`positive_class_denominator_zero`; specificity/FPR use
`negative_class_denominator_zero`. `BalancedAccuracyV1` is tagged as
`available { sensitivity_metric_id, specificity_metric_id, point_estimate }`
or `unavailable { sensitivity_metric_id, specificity_metric_id,
reason="sensitivity_or_specificity_unavailable" }`; it
never has an interval.
`validate_structure` reconstructs all set unions/counts, metric arithmetic,
exclusion projection, rule-result shape, canonical order, and report identity
that are self-contained. `validate_against` additionally re-normalizes the exact
hashed protocol, validates the dataset against it, replays the combined closure
and partition from `ValidationInputs`, re-evaluates every rule/endpoint/claim,
and, for physical claims, re-verifies the embedded trust-store hash and approval
signatures. Only `validate_against` may authorize publication; a successful
generic `VersionedArtifact::validate_after_read` is structural, not scientific
approval.

`report_id` is exactly:

```text
"sha256:" + SHA256_HEX(JCS({
  "identity_domain": "mhi_validation_report_id_v1",
  "protocol_sha256": <exact protocol hash>,
  "dataset_source": <canonical tagged dataset source reference>,
  "consumed_sources": <canonical SourceReferenceV1 list>
}))
```

`DatasetSourceReferenceV1` is exactly one of:

```text
known { dataset_id, schema=1, artifact_id, semantic_sha256,
        source_file_sha256 }
legacy_unknown { dataset_id, schema=1, legacy_fingerprint,
                 source_file_sha256, reason:LegacyLineageReasonV1 }
```

The dataset field and report-ID preimage use this same tagged representation.
For all tagged source unions, variant discriminants are the zero-based order in
which variants are listed in this section; the wire tag is the exact snake-case
variant name. `consumed_sources` excludes the dataset and protocol because they
occupy dedicated preimage fields; it includes every consumed scientific
artifact, lineage catalog, reference authority, and, for a physical report,
the embedded approval trust store and physical approval exactly once.

This preimage contains no report results because results are a deterministic
function of these authorities; report validation rejects any inconsistent
result. It contains no clock, hostname, absolute/output path, or prose.

`ValidationProvenanceV1` contains exactly software version, optional Git
commit, protocol SHA-256, a tagged dataset source reference, and sorted
`SourceReferenceV1` records. The closed variants are:

```text
known_artifact { kind, schema, artifact_id, semantic_sha256, source_file_sha256 }
legacy_artifact { kind, schema, legacy_fingerprint, source_file_sha256,
                  reason:LegacyLineageReasonV1 }
lineage_catalog { schema=1, source_file_sha256 }
reference_authority { reference_source_id, source_file_sha256, origin }
approval_trust_store { trust_store_id, source_file_sha256 }
owner_approval { approval_record_id, source_file_sha256,
                 registry_record_sha256 }
```

Every source actually used in a numerator, denominator, exclusion, leakage
decision, reference decision, or claim gate appears exactly once. Declared but
unread not-applicable paths do not. Report artifact lineage is Known only when
the dataset and every consumed mechanism/health artifact are Known; its direct
dependencies are those artifacts sorted by existing dependency rules. Any
consumed LegacyUnknown artifact makes report lineage LegacyUnknown. Catalog,
reference, trust-store, approval, and protocol authorities remain typed provenance/config
references and never receive fabricated `ArtifactId` values.

#### Hard error versus scientific decision matrix

| Condition | Exact treatment |
|---|---|
| malformed/unknown/duplicate/nonfinite protocol or dataset field | hard error |
| unsafe path, symlink, missing referenced file, wrong kind/schema/file hash/identity | hard error |
| Known declaration differs from serialized scope/family | hard error |
| malformed/closed-schema-invalid catalog, trust store, or approval file | hard error |
| Known assessed root absent/identity mismatch/direct-dependency mismatch, dependency-kind mismatch, or reference-leaf identity mismatch | hard error |
| structurally valid catalog with missing transitive ancestor, reachable cycle, or Unknown transitive scope/family | `unknown_separation` under section 3.6 |
| reference-source graph cycle | hard error |
| for software-only support: absent reference dependency node, incomplete reference node, absent Known scientific leaf, or LegacyUnknown scientific leaf | `unknown_separation` with the exact section 3.1 reason |
| physical claim missing/mismatching approval, unknown embedded root, or failed owner/registry signature | hard error |
| physical supporting mechanism reference has semantic outcome `unavailable` | hard `PhysicalReferenceOutcomeUnavailable` before any exclusion or score |
| physical supporting record lacks a current Known assessed source or exactly one bound reference, has nonphysical origin, or fails actual reference method/authority/blinding/quantification/reference-node-completeness checks | hard `PhysicalReferenceAuthorityMismatch` before any exclusion or score |
| null endpoint artifact path | typed exclusion `missing_endpoint_artifact_path` |
| for software-only support: readable legacy/current-LegacyUnknown scientific source | typed exclusion `source_not_phase_b_or_c_scoreable`; unknown separation retained |
| for software-only support: missing reference endpoint or complete but protocol-ineligible reference | typed exclusion with exact reference reason |
| known development/holdout overlap | holdout endpoint `does_not_meet_protocol` |
| unknown holdout separation | holdout endpoint `indeterminate` |
| validation-role known/unknown overlap | typed exclusion |
| zero/underpowered view | `indeterminate` |
| unavailable metric required by a rule | `indeterminate` |

No row may be reclassified by implementation convenience.

### 4.4 Output artifacts

One successful run atomically publishes:

```text
OUTPUT/
  mhi_validation_report.schema1.json
  validation_execution_manifest.schema1.json
  validation_summary.md
  tables/
    cohort_coverage.csv
    leakage_assessment.csv
    mechanism_validation.csv
    health_validation.csv
    exclusion_ledger.csv
    compatibility_matrix.csv
```

The execution manifest records input identities/hashes, protocol hash,
generated paths and SHA-256 checksums, and software/Git identity.
It contains no scientific result not already present in the validation report.
Its closed schema is exactly:

```text
ValidationExecutionManifestV1 = {
  schema_version:1,
  output_kind:"mhi_validation_execution_manifest",
  report_id,
  protocol_sha256,
  dataset_source:DatasetSourceReferenceV1,
  generated_files:[GeneratedFileRecordV1],
  publication_mode: one of create_new|replace_managed_bundle,
  software_version,
  git_commit:null|string
}

GeneratedFileRecordV1 = {
  relative_path, output_kind, byte_length:u64, sha256
}
```

`publication_mode=create_new` iff output did not exist when the exclusive
publication lock was acquired; `replace_managed_bundle` iff the atomic exchange
replaced a verified managed bundle. There is no third token and no inference
from output path. V1 omits the
timestamp field entirely; wall-clock timing belongs only to process logs outside
the managed bundle. Generated
file records sort by relative path and contain path, output kind, byte length,
and SHA-256. The generated-file list contains the report, summary, and six CSV
tables; it deliberately excludes the execution manifest itself, avoiding a
self-referential checksum. Managed-bundle validation separately requires the
manifest as the ninth exact file.

Exact generated-file `output_kind` mapping is:

```text
mhi_validation_report.schema1.json -> mhi_validation_report
validation_summary.md              -> validation_summary_markdown
tables/cohort_coverage.csv         -> cohort_coverage_csv
tables/leakage_assessment.csv      -> leakage_assessment_csv
tables/mechanism_validation.csv    -> mechanism_validation_csv
tables/health_validation.csv       -> health_validation_csv
tables/exclusion_ledger.csv        -> exclusion_ledger_csv
tables/compatibility_matrix.csv    -> compatibility_matrix_csv
```

No figure contract is introduced in Phase E; validation plots are out of scope
until their statistical and accessibility requirements are separately
reviewed.

#### Byte-level serialization rules

- JSON is UTF-8 without BOM, two-space indented, uses LF only, and has no final
  newline, exactly as produced by `domain::write_artifact`/
  `serde_json::to_string_pretty` on a validated `serde_json::Value`. Object keys
  are lexicographic UTF-8 order. Integers are shortest base-10. f64 values use
  Serde/Ryu shortest round-trippable spelling from the exact section 3.7 bits;
  a serialized/input negative zero is rejected, while a computed value equal
  to zero has its sign bit cleared to positive `0.0` before hashing/writing.
- CSV uses RFC 4180 field quoting with comma delimiter, `"` quoting doubled as
  `""`, UTF-8, deliberately fixed LF (not CRLF) record terminators, one header
  row, and one final LF. Integer and float
  tokens follow JSON spelling. Any unavailable/null scalar cell is `NA`.
  Collection cells are compact JSON arrays with canonically sorted string
  elements; an empty collection is `[]`. Any non-null object-valued cell, such
  as an assessed source key, is compact RFC-8785 JCS with no whitespace.
- Markdown is UTF-8/LF with one final LF, no trailing spaces, and canonical
  float/`NA` tokens. Rows use the same ordering as the corresponding CSV.
- Managed JSON, CSV, and Markdown have no structural field for an operational
  timestamp, hostname, absolute path, staging/backup name, or output directory,
  and the writer never injects one. Opaque registered statements/limitations
  are copied exactly and are not rejected merely because their human text
  happens to contain a path- or date-shaped substring.

The six exact CSV headers are:

```text
cohort_coverage.csv:
endpoint_id,stratum_id,endpoint_kind,cohort_role,declared_count,eligible_count,excluded_count,not_applicable_count,exclusion_rate,exclusion_lower,exclusion_upper,evaluable_count,indeterminate_count,data_quality_insufficient_count,coverage,coverage_lower,coverage_upper,indeterminate_rate,indeterminate_lower,indeterminate_upper,data_quality_insufficient_rate,data_quality_insufficient_lower,data_quality_insufficient_upper,outcome

leakage_assessment.csv:
endpoint_id,stratum_id,record_id,separation_status,not_evaluated_reason,compared_development_record_ids,shared_artifact_ids,shared_source_sha256s,shared_experiment_ids,shared_family_ids,unknown_reasons,decision

mechanism_validation.csv:
endpoint_id,stratum_id,eligible_count,independent_family_count,support_count,critical_contradiction_count,declared_critical_falsification_count,not_assessed_or_other_count,support_fraction,support_lower,support_upper,contradiction_fraction,contradiction_lower,contradiction_upper,not_assessed_fraction,not_assessed_lower,not_assessed_upper,outcome

health_validation.csv:
endpoint_id,stratum_id,eligible_count,independent_family_count,tp,tn,fp,fn,indeterminate,data_quality_insufficient,evaluable,coverage,coverage_lower,coverage_upper,indeterminate_rate,indeterminate_lower,indeterminate_upper,data_quality_insufficient_rate,data_quality_insufficient_lower,data_quality_insufficient_upper,sensitivity,sensitivity_lower,sensitivity_upper,specificity,specificity_lower,specificity_upper,false_positive_rate,false_positive_lower,false_positive_upper,false_negative_rate,false_negative_lower,false_negative_upper,balanced_accuracy,outcome

exclusion_ledger.csv:
endpoint_id,stratum_id,record_id,primary_reason,secondary_reasons,assessed_source_key,reference_endpoint_id

compatibility_matrix.csv:
record_id,source_role,relative_path,expected_kind,actual_kind,expected_schema,actual_schema,expected_file_sha256,actual_file_sha256,expected_artifact_id,actual_artifact_id,expected_semantic_sha256,actual_semantic_sha256,result
```

CSV row membership and cell spelling are total:

| File | Exact row source and special-cell rules |
|---|---|
| `cohort_coverage.csv` | Exactly one `CohortRowV1` per endpoint/view. Every endpoint kind writes exclusion point/lower/upper from declared-count authority. Mechanism rows put `NA` in all three health counts and all nine health metric point/interval cells; health rows use exact values. Any unavailable metric spells `NA` in point/lower/upper. Counts are never `NA` on the endpoint kind that owns them. |
| `leakage_assessment.csv` | Exactly every `LeakageRowV1`, including `not_applicable`. For evaluated rows (including LegacyUnknown → `unknown_separation` and a visible known overlap despite a missing prerequisite), `separation_status` is the token and `not_evaluated_reason=NA`. An outside-view row uses `separation_status=NA`, `not_evaluated_reason=not_applicable`, and empty comparison collections. A missing-source/reference row with no visible overlap uses `separation_status=NA`, the lowest section 3.7 missing-prerequisite exclusion ordinal as `not_evaluated_reason`, and empty shared/unknown collections; its `compared_development_record_ids` still equals the section 3.6 comparator ID set. Decision is always retained. |
| `mechanism_validation.csv` | Exactly one row per `MechanismResultV1`; each `MetricValueV1.available` supplies point/lower/upper, while `unavailable` supplies `NA` to all three cells. |
| `health_validation.csv` | Exactly one row per `HealthResultV1`; the same metric projection applies. Balanced accuracy is its point or `NA` and has no interval columns. |
| `exclusion_ledger.csv` | Exactly one row per `ExclusionRowV1`, hence only `decision=excluded` accounting rows. Nullable source/reference cells are `NA`; secondary reasons are a compact canonical JSON string array. |
| `compatibility_matrix.csv` | Exactly one row for protocol, dataset, lineage catalog, optional approval, and each present mechanism/health source opened by the run. Embedded reference-source nodes and embedded trust-store bytes are represented in report provenance, not as path rows. `relative_path` is the literal logical locator `@protocol` or `@dataset` for the two CLI roots and the exact safe dataset-relative `/` path for every other row; an absolute/canonical host path is forbidden. Protocol/catalog/approval rows use `NA` for artifact kind/ID/semantic fields; absent optional actual fields use `NA`. `source_role` and `result` use only the closed tokens in `CompatibilityRowV1`. |

All CSV rows sort by the report collection keys in section 4.3. Rate
numerator/denominator are retained in JSON and need not be duplicated in CSV. JSON null is never
spelled as an empty CSV cell.

For compatibility rows, protocol and dataset have no upstream expected file
hash, so their `expected_file_sha256` is definitionally the exact computed hash
registered in `ProtocolAuthorityV1`/`DatasetSourceReferenceV1` and equals
`actual_file_sha256`; this records the run root rather than asserting an
external comparison. Catalog, approval, and scientific-source expected values
come only from the dataset declarations. A mismatch is a hard error, so a
published row records the exact verified expected/actual values rather than a
failed continuation. `record_id` is null for protocol/dataset/catalog/approval
and is the owning dataset record ID for each scientific-source row.

`validation_summary.md` has exactly this section order:

```text
# MHI Validation Summary

## Identity
## Cohort Coverage
## Leakage
## Mechanism Endpoints
## Health Endpoints
## Exclusions
## Release Claims
## Overall Status
## Limitations
```

Identity uses the exact header/delimiter lines
`| key | value |` and `| --- | --- |`. The next five sections project
the corresponding CSV columns without omission. Release claims sort by claim
ID and list requested level, outcome, supporting endpoints, approval record ID
or `NA`, and domain. Overall Status contains exactly one outcome token.
Limitations is a sorted bullet list or the literal `- NONE`. No generated prose
or implementation-chosen sentence is permitted.

The Identity table rows are exactly, in order: `report_id`, `protocol_id`,
`protocol_sha256`, `dataset_id`, `dataset_source_file_sha256`,
`approval_record_id`, `approval_trust_store_sha256`, `software_version`,
`git_commit`; nullable values spell `NA`. Every projected section uses a GitHub pipe table with one leading/trailing
pipe and one ASCII space around each cell, the exact CSV header as its header
row, and delimiter cells `---`. Markdown text cells escape backslash as `\\`,
pipe as `\|`, and LF as `<br>` in that order. Collection cells remain compact
JSON arrays. Empty projected tables retain header/delimiter rows and no data
row. The Release Claims header line is exactly
`| claim_id | requested_level | statement | domain | supporting_endpoint_ids | approval_record_id | outcome |`
and its delimiter line has seven `---` cells under the common pipe-table rule;
`domain` is compact RFC-8785 JCS for the exact `DomainSelectorV1`, then Markdown-
escaped, and endpoint IDs are a compact canonical JSON array. `Overall Status`
contains the exact two lines `outcome: <token>` and a following blank line.
Each limitation bullet is `- ` plus the source limitation after escaping
backslash as `\\` and LF as `<br>` in that order; no wrapping is allowed.
The unescaped limitation source set is exactly the duplicate-free union of
`approval:<text>` for every approval limitation,
`endpoint:<endpoint_id>:<stratum_id>:<text>` for every mechanism/health result
limitation, and `warning:<code>:<related_id>:<detail>` for every report warning,
sorted by raw UTF-8 bytes. Empty union emits `- NONE`. The title and each
section heading are followed by exactly one blank line; each table or bullet
block is followed by exactly one blank line before the next heading. The final
Limitations line has the document's single final LF and no extra blank line.
These rules plus the fixed section order determine all Markdown bytes.

The committed golden authority is the exact nine-file tree under
`tests/fixtures/phase_e/expected/golden_bundle/`, with a sibling
`golden_bundle_file_sha256s.txt` listing each relative path, byte length, and
SHA-256. It is the `create_new` case. The exact overwrite-mode replacement for
its manifest is
`tests/fixtures/phase_e/expected/golden_replace_execution_manifest.schema1.json`;
the other eight files must remain byte-identical. Tests compare bytes, not parsed equivalence, and independently validate
JSON fields, CSV row/cell projections, Markdown escaping, manifest mode tokens,
and the absence of timestamps/host/output paths.

### 4.5 Atomic publication state machine

For output directory basename `B` in canonical parent `P`, the fixed private
paths are `P/.B.phase-e-stage`, `P/.B.phase-e-backup`, and
`P/.B.phase-e-publish.lock`. No output, parent, private path, or managed-tree
component may be a symlink. macOS is the only supported and release-validated
V1 publication platform when the exact atomic operations below are available
on the output filesystem; otherwise the run hard-fails
`UnsupportedAtomicPublicationFilesystem` before any publication
rename/exchange and applies pre-commit stage cleanup. The Linux `renameat2`
path is retained as deferred future-support code and is not authoritative for
MHI V1 Phase E.
`P` must already exist as a canonical directory; `B` must be a nonempty
single filename component other than `.`/`..`, with no `/`, NUL, or platform
prefix, and output/private names must be pairwise distinct.

Atomic primitives are:

- `rename_noreplace(a,b)`: macOS `renameatx_np(RENAME_EXCL)`; destination
  existence must fail without change. The retained Linux
  `renameat2(RENAME_NOREPLACE)` implementation is deferred and non-authoritative.
- `rename_exchange(a,b)`: macOS `renameatx_np(RENAME_SWAP)`; both directory
  names change in one atomic metadata operation. The retained Linux
  `renameat2(RENAME_EXCHANGE)` implementation is deferred and non-authoritative.
- `fsync_dir(path)`: open the directory itself and require successful `fsync`;
  an unsupported/error result is not ignored.

The deterministic state machine is:

1. Open/create the lock file with no-follow semantics, require a regular file
   with link count one and byte length zero, and acquire a nonblocking exclusive
   OS advisory lock. A nonempty/nonregular/multilink lock returns
   `PublicationLockFileInvalid` without modification.
   Lock contention returns `PublicationLocked` without inspecting/mutating
   output. The empty lock file persists; the lock is released only when its file
   descriptor closes, avoiding unlink/recreate races. On first creation,
   `fsync` the lock file and `fsync_dir(P)`.
2. Under the lock, classify exact names. Any existing stage or backup is
   `PublicationRecoveryResidue { output_state, stage_state, backup_state,
   remaining_paths }` and no path is removed or renamed. Each state is one of
   `absent`, `valid_managed_bundle`, `unmanaged`, or `symlink`; paths are sorted.
   `output + valid stage/backup` is explicitly a previously committed cleanup
   residue, not an invitation to restore over output. `output absent + backup`
   is ambiguous residue and is likewise preserved for an operator.
3. Preflight output under the lock. Without `--overwrite`, any existing output
   is `OutputAlreadyExists`. With overwrite, output must be a complete managed
   bundle: exactly the nine paths, valid closed manifest, all eight recorded
   lengths/hashes correct, no extra entry/symlink. Otherwise return
   `OutputNotManaged` without change. Managed validation is descriptor-relative
   with no-follow opens. Keep the opened output-directory descriptor alive and
   bind `OutputGenerationV1 { st_dev_u64, st_ino_u64,
   bundle_fingerprint }`, where the native `fstat` device/inode values are
   converted losslessly with `u64::try_from` (conversion failure is
   `UnsupportedAtomicPublicationFilesystem`) and
   `bundle_fingerprint` is
   `SHA256(UTF8("mhi_managed_bundle_fingerprint_v1\0") || entries)`. `entries`
   concatenates all nine paths in section 4.4 relative-path byte order as
   `U64_BE(path_utf8_byte_length) || path_utf8 || U64_BE(file_byte_length) ||
   SHA256_RAW(file_bytes)`. An unavailable/nonstable directory identity is
   `UnsupportedAtomicPublicationFilesystem`. This internal binding is never a
   report/manifest field.
4. Create stage with a non-recursive create-new operation and immediately
   `fsync_dir(P)`. Write files in exact relative-path order, fsync each file,
   then fsync `tables/`, stage, and `P` in that order. Strictly reread report and
   manifest, call authority-assisted report validation, and verify paths,
   lengths, hashes, and byte rules. Keep the stage directory descriptor open
   and bind `StageGenerationV1` with the same identity/fingerprint algorithm as
   step 3 after this verification. Staging failure performs canonical cleanup
   only before any publication primitive; cleanup failure returns
   `PublicationStagingCleanupFailed { primary_error, remaining_paths }`. For
   overwrite, immediately before the exchange, reopen `output` no-follow,
   require its `fstat(st_dev,st_ino)` to equal the held descriptor and its
   recomputed fingerprint/exact managed validation to equal the step-3 binding.
   A missing, replaced, symlinked, mutated, or unreadable output returns
   `PublicationConcurrentManagedOutputChanged { output_state,
   identity_result, fingerprint_result }`, preserves the competing output,
   and applies only pre-commit cleanup to the new stage.
5. Create-new publication calls `rename_noreplace(stage, output)` and then
   `fsync_dir(P)`. A no-replace collision returns
   `PublicationConcurrentDestinationCreated` and invokes pre-commit stage
   cleanup; it never replaces the competing path. Any other rename failure has
   the same no-rename guarantee and cleanup. If rename succeeds but parent fsync
   fails, the visible verified output is preserved and the command returns
   `PublicationDurabilityUnconfirmed { output, operation="create_new",
   fsync_error }`; it must not claim a durable publication. Successful parent
   fsync is the commit point.
6. Overwrite publication calls `rename_exchange(stage, output)` immediately
   after the step-4 recheck and then `fsync_dir(P)`. The new verified bundle is
   now `output` and the directory found at the destination by the atomic
   exchange is `stage`; there is never a namespace instant with output absent.
   Exchange-call failure guarantees no swap: old output remains and the new
   stage follows pre-commit cleanup. Exchange success followed by parent-fsync
   failure preserves new output and the entire swapped stage and returns
   `PublicationDurabilityUnconfirmed { output, operation="replace_managed_bundle",
   fsync_error, remaining_paths=[stage] }`; it never guesses that the old bundle
   should overwrite a valid output.
7. After a committed exchange and before any cleanup rename/unlink, first open
   `output` no-follow and require its identity, fingerprint, and exact
   descriptor-relative managed validation to equal the held step-4
   `StageGenerationV1`. A mismatch returns
   `PublicationCommittedVisibleOutputChanged { output_state, identity_result,
   fingerprint_result, remaining_paths=[stage] }`; no stage entry is inspected,
   renamed, or deleted and no success is reported. Only after the visible
   output passes, open `stage` no-follow and require its identity, fingerprint,
   and exact managed validation to equal the held step-3 `OutputGenerationV1`.
   If that old-generation proof differs, do not rename or delete any stage
   entry: the newly verified output remains committed, the entire swapped
   object is preserved at `stage`, and return
   `PublicationCommittedForeignSwapDetected { output, stage_state,
   identity_result, fingerprint_result, remaining_paths=[stage] }`. This
   ordered two-generation proof detects replacement or same-inode mutation of
   either the newly visible output or the swapped old destination through that
   path's final proof.
8. Only after step 7 proves exact equality, call
   `rename_noreplace(stage, backup)` and `fsync_dir(P)`, then remove backup
   entries in reverse canonical relative-path order. After every file unlink or
   directory removal, fsync its containing directory; finally fsync `P`. If
   rename or deletion fails, the new output remains committed and the command returns
   `PublicationCommittedCleanupFailed { output, stage_state, backup_state,
   remaining_paths, cleanup_error }`. A partially deleted backup is permitted
   only in this typed state; `remaining_paths` is an exact sorted snapshot and
   later runs preserve it under step 2.
9. During a pre-commit cleanup, entries are removed in the same reverse
   canonical order with directory fsync after each transition. Private residue
   is never accepted as final output, silently removed on a later run, or used
   as authority. Process logs may describe recovery; managed scientific bytes
   do not.

The concurrency guarantee covers every Phase-E publisher (all obey the
persistent lock), replacement/same-inode mutation of either exchange generation
through that path's ordered step-7 proof, and writers quiescent immediately
after their path's final proof. POSIX advisory locks cannot exclude a hostile
process that already has write permission and mutates a path after its final
proof; that noncooperating post-proof writer is explicitly outside V1's threat
model and deployment must deny it write access to `P` and managed outputs.
Even for a foreign replacement in the precheck-to-exchange interval, Phase E
never unlinks or changes the swapped object's bytes: it preserves the whole
object at `stage` and reports the committed typed state. A changed newly visible
output likewise prevents cleanup of the old stage. No success is reported in
either case.

For all three race errors, `output_state`/`stage_state` uses step 2's exact state
tokens. `identity_result` is exactly `match`, `mismatch`, or `unavailable`.
Identity is checked first. `fingerprint_result` is exactly `match`, `mismatch`,
or `not_evaluated`, and is `not_evaluated` unless identity is `match`; exact
managed validation runs only after fingerprint `match`. A failed exact-managed
validation after both matches is represented by the already classified
`output_state`/`stage_state` plus `identity_result=match` and
`fingerprint_result=match`; it is still the same race error. These tokens and
the sorted `remaining_paths` snapshot, not OS error prose, are the test oracle.

Tests run two synchronized publisher processes to force lock contention,
create-new destination races, overwrite precheck/exchange boundaries,
same-inode mutations of the old and new generations, an old-destination
namespace replacement after the final precheck, and a newly visible output
replacement before its post-exchange proof; inject every write/fsync/noreplace/
exchange/rename/delete failure; and assert the exact error,
identity/fingerprint tokens, commit classification, output bytes, preserved
stage bytes, and sorted residue snapshot after each operation and simulated
crash boundary.

### 4.6 Compatibility objectives

- Existing CLI parsing and behavior, including `report render`, is unchanged.
- Existing mechanism schema 4, health schema 4, public-summary schema 1,
  render-manifest schema 1, evidence bundle schema 1, lineage catalog schema 1,
  and model-validation schema 1 are byte/semantic compatibility baselines.
- Only mechanism schema 4 and health schema 4 are scientifically scoreable in
  Phase E. Readable legacy mechanism schemas 1–3 and health schema 3 may be
  listed in the exclusion ledger but cannot be upgraded or scored as Phase B/C.
- Wrong kinds, unsupported future schemas, identity/hash mismatch, missing
  required files, and path escape are typed hard failures before publication.
- The two new artifact kinds are additive. Every exhaustive `ArtifactKind`
  match must be updated without changing existing wire tokens.
- The additive catalog-strict reader rejects nested unknown fields only for
  Phase E. The existing `read_artifact_lineage_catalog` route, public error
  variants, and Phase-D acceptance matrix remain unchanged and are tested on
  the same nested-unknown bytes.
- No new dependency may alter existing numerical results. A statistics
  dependency, if proposed, requires locked-version review and fixed-vector
  parity tests; a small reviewed in-tree Wilson implementation is preferred.
- The only approved dependency-surface change is the no-feature direct
  `ed25519-dalek 2.2.0` strict-verification dependency. Relative to the current
  Phase-D `Cargo.lock`, the complete permitted new lock entries are exactly:

  | Package | Version | crates.io checksum |
  |---|---|---|
  | `curve25519-dalek` | `4.1.3` | `97fb8b7c4503de7d6ae7b42ab72a5a59857b4c937ec27a3d4539dba95b5ab2be` |
  | `curve25519-dalek-derive` | `0.1.1` | `f46882e17999c6cc590af592290432be3bce0428cb0d5f8b6715e4dc7b383eb3` |
  | `ed25519` | `2.2.3` | `115531babc129696a58c64a4fef0a8bf9e9698629fb97e9e40767d235cfbcd53` |
  | `ed25519-dalek` | `2.2.0` | `70e796c081cee67dc755e1a36a0a172b897fab85fc3f6bc48307991f64e4eca9` |
  | `fiat-crypto` | `0.2.9` | `28dea519a9695b9977216879a3ebfddf92f1c08c05d984f8996aecd6ecdc811d` |
  | `signature` | `2.2.0` | `77549399552de45a898a580c1b41d445bf730df867cc44e6c0233bbc4b8329de` |

  The complete sorted `Cargo.lock` dependency arrays for those entries are
  exactly:

  | Package | Exact dependency array |
  |---|---|
  | `curve25519-dalek` | `["cfg-if","cpufeatures 0.2.17","curve25519-dalek-derive","digest","fiat-crypto","rustc_version","subtle"]` |
  | `curve25519-dalek-derive` | `["proc-macro2","quote","syn 2.0.118"]` |
  | `ed25519` | `["signature"]` |
  | `ed25519-dalek` | `["curve25519-dalek","ed25519","sha2","subtle"]` |
  | `fiat-crypto` | `[]` |
  | `signature` | `[]` |

  The existing `rust_electroanalysis_cli` package dependency array gains only
  `"ed25519-dalek"` in Cargo's canonical name order. Every other existing
  package version, checksum, dependency array, and feature selection remains
  byte-for-byte unchanged. Any resolution difference is a plan-revision and
  independent security-review stop, not implementation discretion.

## 5. Scope

### IN SCOPE

- The schema-1 validation protocol, validation dataset artifact, validation
  report artifact, embedded physical-approval trust store, dual-signed owner-
  approval evidence input, lineage-catalog reference, reference-source graph,
  and execution manifest defined above.
- Canonical reading of existing Phase B schema-4 mechanism and Phase C
  schema-4 health artifacts referenced by a closed manifest.
- Deterministic cohort eligibility, lineage/family leakage checks, exact
  counts, registered rates, Wilson 95% intervals, stratification, exclusions,
  and protocol outcomes.
- Independent reference endpoints with explicit authority, blinding,
  uncertainty, domain, and provenance.
- A single additive CLI route and atomic JSON/Markdown/CSV validation bundle.
- Verification-only use of the exact no-feature `ed25519-dalek 2.2.0` strict
  verifier and frozen six-package lock delta above; no signing, key generation,
  network trust, runtime key loading, or other crypto dependency.
- Schema/artifact migration fixtures, full regression validation, and
  independent scientific and architecture review.
- At least one dual-signed, embedded-root-verified physical holdout cohort per
  release claim that is to be described as physically validated.

### OUT OF SCOPE

- Any Phase B hypothesis reassessment, promotion, threshold change, or new
  mechanism mapping.
- Any Phase C health reassessment, new dimension, threshold change, causal
  diagnosis, alarm action, maintenance recommendation, or prognosis.
- Changes to Phase D report semantics, paths, tables, figures, public summary,
  or manifest.
- Raw CSV/XLSX/CHI ingestion, experimental data cleaning, model fitting,
  calibration, state estimation, or source artifact production in the Phase E
  route.
- Automatic train/test splitting, inferred labels, inferred acquisition
  families, inferred sensor/domain metadata, or filename-based cohort roles.
- Bayesian model selection, structural/model-form uncertainty, new covariance
  approximations, high-fidelity Nernst–Planck transport, or physical mechanism
  discovery.
- Online monitoring, alert dispatch, fleet telemetry, database/service APIs,
  GUI work, or cloud deployment.
- Validation figures, performance optimization, Windows enablement, cleanup of
  unrelated technical debt, or standardization of every repository config
  migration policy.

## 6. Data model impact

### 6.1 New artifacts

| Artifact | Schema | Owner | Migration at introduction |
|---|---:|---|---|
| `mhi_validation_dataset` | 1 | `src/results/mhi_validation.rs` | New artifact; no legacy form. |
| `mhi_validation_report` | 1 | `src/results/mhi_validation.rs` | New artifact; no legacy form. |
| `validation_execution_manifest` | 1 | Phase E output writer; presentation manifest, not a `VersionedArtifact` | New output; no legacy form. |
| `OwnerApprovalEvidenceV1` | 1 | `src/mhi_validation/approval.rs`; closed authority input, not a `VersionedArtifact` | New input authority; no legacy form and no migration. |
| `PhysicalApprovalTrustStoreV1` | 1 | reviewed `config/mhi_physical_approval_trust_store.schema1.json`, embedded in the binary | New non-overridable configuration authority; forward review only. |

The validation protocol is closed TOML configuration schema 1, not a scientific
result artifact. Its exact byte hash is stored in both new artifacts and the
execution manifest. The trust store, owner approval, and lineage catalog are
separately typed configuration/input authorities, not scientific result
artifacts, and never receive a fabricated `ArtifactKind` or `ArtifactId`.

### 6.2 Existing schema changes

No existing artifact payload or schema version changes in Phase E. Shared
changes are limited to adding `MhiValidationDataset` and `MhiValidationReport`
variants to `ArtifactKind`, registering their `VersionedArtifact` contracts,
adding `read_artifact_strict`, and adding the nested-strict catalog companion
plus shared internal text parser described in section 4.3. Existing public
reader/writer behavior, errors, and call sites remain unchanged; Phase E alone
uses the two strict APIs. The frozen no-feature `ed25519-dalek 2.2.0` lock delta
is not a data-model change.

When all consumed scientific artifacts have Known lineage, the new report
lineage directly depends on the dataset artifact and every mechanism/health
artifact actually used in a numerator, denominator, exclusion, reference, or
leakage decision. If any such artifact has `LegacyUnknown`, the report lineage
is also `LegacyUnknown`; its `SourceReferenceV1` fingerprint remains explicit,
and no dependency ID is fabricated. Every consumed catalog, reference-source,
trust-store, approval, and protocol authority remains a typed provenance/config reference.
Merely declared but unread not-applicable inputs are neither dependencies nor
provenance sources.

### 6.3 Migration strategy

1. Introduce both new artifact kinds at current schema 1 with no legacy schema
   list and required `artifact_kind`.
2. Preserve every existing kind token, schema table, Serde default, reader,
   writer, and CLI route unchanged.
3. Accept only exact schema-1 Phase E artifacts. Reject future versions until a
   separately reviewed migration contract exists.
4. Read existing mechanism/health artifacts in Phase E through the additive
   `domain::read_artifact_strict`; retain `domain::read_artifact` unchanged for
   existing callers.
   Schema-4 artifacts may be scored; readable legacy artifacts receive a typed
   exclusion and never an in-memory upgrade to missing Phase B/C content.
5. Add the complete section 7.3 fixture/mutation ledger, including literal
   schema-1 round trips, wrong/missing/future kind/schema, unknown/duplicate/
   nonfinite fields, lexical/symlink escape, identity/declaration mismatch,
   renamed-source duplication, combined catalog/reference closure, trust/
   signature/approval binding, classification partitions, precedence, byte
   vectors, and no-clobber crash-durable publication.
6. Before any future schema-2 work, archive canonical schema-1 fixtures and
   define field-by-field preservation, rejection, and semantic-hash behavior.

Rollback is additive: removing the Phase E command/modules, embedded trust-store
config, direct `ed25519-dalek` declaration and its six newly locked packages,
strict-reader entry point, and two new artifact-kind registrations
returns to the exact Phase D behavior; no existing artifact needs rewriting.

## 7. Validation strategy

### 7.1 Unit tests

- Closed protocol parsing covers every section 4.3 field and token, missing
  fields, unknown/duplicate keys, nonfinite/range errors, endpoint-role/domain/
  family/stratum completeness, total class partitions, invalid metric/target
  pairs, contradictory bounds, and release-claim bindings.
- Dataset invariants cover canonical sort keys, duplicate IDs and duplicate
  assessed-source keys, safe lexical/canonical paths and symlinks, exact
  file/recomputed-identity/declaration equality, exact catalog root treatments,
  combined reference/catalog closure, dual-signature owner/registry approval
  bindings, embedded trust-root isolation, globally distinct authority IDs and
  key bytes, every-key weak-point rejection, strict signature verification,
  exact endpoint/claim domain equality, and cohort semantic hash.
- Leakage classification covers disjoint/shared/unknown artifact lineage,
  experiment scope, families, renamed files, aggregate scopes, missing
  ancestors, cycles, incomplete reference dependencies, reference-to-scientific-
  artifact expansion, and direct/indirect self-derived references.
- Exact count/rate calculations, zero denominators, confusion matrices,
  balanced accuracy, complete Phase-B/Phase-C mapping tables, and fixed Wilson
  95% decimal plus exact-bit vectors including boundary counts.
- Status tests evaluate all combinations in section 3.7, including simultaneous
  unavailable/false rules, overlap plus missing metrics, critical contradiction,
  empty/underpowered strata, and claim/overall composition.
- Report invariants: count reconstruction from IDs, canonical ordering,
  complete accounting/exclusion/leakage equality, consumed-source provenance,
  physical approval, exact report-ID preimage, and stable scientific bytes.

### 7.2 Integration and CLI tests

- Canonical artifact read/write round trips for both new kinds.
- A complete manifest → reader → evaluator → writer → rereader path using
  literal, independently derived expected metrics.
- Wrong kind/schema/hash/identity/declaration, missing file, absolute/lexical/
  canonical/symlink escape, duplicate renamed source, overlapping holdout,
  unknown independence, missing/ineligible reference, invalid approval, and
  empty/underpowered stratum paths.
- CLI parsing for required flags, unknown options, output collision,
  `--overwrite`, and no raw-file/directory-discovery alternative.
- Atomic publication injects every section 4.5 failure, including stage/backup
  residue, lock contention, invalid managed bundle, concurrent no-clobber race,
  exchange, managed generation replacement/mutation immediately before and
  during exchange, the post-exchange identity/fingerprint proof, every parent-
  directory fsync, partial backup deletion, and stage cleanup; exact commit,
  preserved foreign bytes, and private filesystem state are asserted.
- Repeated runs from the same protocol and artifacts produce identical
  scientific JSON/CSV/Markdown bytes. macOS V1 validation compares exact Wilson
  bits and sealed report/table hashes. Operational timestamps are absent from
  every managed file and their presence is a rejecting mutation.
- Source guard tests prove Phase E does not call Phase B/Phase C assessors and
  Phase D reporting does not import Phase E.

### 7.3 Required fixtures

This is the normative fixture/mutation traceability ledger. Every path named in
the `Fixture(s)` column is a required committed literal fixture under
`tests/fixtures/phase_e/`. No other permanent Phase-E fixture may be added
without a row that maps it to requirement, acceptance criterion, test,
mutation, and exact expected result. A mutation is applied in a test-owned
temporary copy; production code never generates an oracle.

| Requirement | AC | Test | Fixture(s) | Required mutation/falsification | Exact expected result |
|---|---|---|---|---|---|
| E-R01 | E-AC01 | E-T01 | `protocol/software_valid.toml`; `dataset/software_valid.schema1.json`; `lineage/complete.schema1.json`; `reference/complete_sources.schema1.json`; `expected/golden_bundle_file_sha256s.txt` | none; invoke exact certified route | exit success; exactly nine managed relative paths and each length/SHA-256 equals the named list |
| E-R01 | E-AC01 | E-T02 | `protocol/software_valid.toml`; `dataset/software_valid.schema1.json` | remove `--protocol`, `--dataset`, or `--output-dir` one at a time; add unknown flag, raw-input flag, or alias `validation run`; make protocol/dataset path a symlink | exact Clap/path error before scientific reader; no output/private path |
| E-R02 | E-AC02 | E-T03 | `protocol/software_valid.toml`; `protocol/physical_valid.toml` | parse and serialize typed value; reparse | exact value equality; exact raw-byte hash retained separately |
| E-R02 | E-AC02 | E-T04 | `protocol/software_valid.toml`; `protocol/physical_valid.toml` | one at a time: missing field; unknown/duplicate key; nonfinite/negative-zero threshold; incomplete/overlapping class partition; invalid endpoint kind/target/metric pair; empty rules or remove/wrong-direction the mandatory mechanism support or health coverage/sensitivity/specificity rule; duplicate endpoint/reference/rule/stratum ID; empty/zero minimum; duplicate/conflicting stratum axis; overlapping domain bands; endpoint broader than claim, then claim broader than endpoint (including claim analyte set `{Pb,Cd}` against supporting endpoint `{Pb}`); contradictory rule bounds; development role; physical record/family minimum 1; missing/unknown physical trust-root ID; add mechanism allowed-outcomes field | each one-sided domain case hard-fails `SupportingEndpointClaimDomainMismatch`; every other mutation returns its exact precedence-selected protocol error; no dataset/source read |
| E-R03 | E-AC03 | E-T05 | `dataset/software_valid.schema1.json`; `lineage/complete.schema1.json`; `reference/complete_sources.schema1.json` | reorder one canonical vector; duplicate record ID | noncanonical/duplicate typed error; unmodified fixture round-trips exactly |
| E-R03 | E-AC03 | E-T06 | `dataset/software_valid.schema1.json`; `mechanism/supported.schema4.json`; `health/within_baseline.schema4.json`; `lineage/complete.schema1.json` | absolute, `..`, empty component, symlink escape, missing file, duplicate JSON key, wrong file hash, false embedded semantic hash, recomputed-vs-embedded identity mismatch, expectation mismatch, root absence/identity/direct-dependency mismatch, declaration scope/family mismatch, duplicate assessed source under renamed path, wrong cohort semantic hash | exact hard error by section 4.2/3.6 precedence; no scoring |
| E-R04 | E-AC04 | E-T07 | `mechanism/supported.schema4.json`; `mechanism/contradicted.schema4.json`; `health/within_baseline.schema4.json`; `health/alert.schema4.json` | none | canonical readers accept exact schema 4 and identities |
| E-R04 | E-AC04 | E-T08 | `mechanism/supported.schema4.json`; `health/within_baseline.schema4.json`; `mechanism/legacy.schema3.json`; `health/legacy.schema3.json` | set wrong kind; schema 5; duplicate nested JSON key; use each legacy source; run each source through existing non-Phase-E reader regression | wrong/future/duplicate hard-fail in strict reader; legacy exact exclusion; existing reader behavior unchanged |
| E-R05 | E-AC05 | E-T09 | `dataset/accounting.schema1.json`; `expected/accounting_ledger.md` | null required path; out-of-domain record; in-domain reference exclusion; zero-declared view; simultaneous exclusion reasons | every endpoint/view record has one decision; declared=eligible+excluded; exclusion rate uses excluded/declared or exact unavailable; literal ledger/precedence match |
| E-R06 | E-AC06 | E-T10 | `dataset/known_overlap.schema1.json`; `lineage/shared_ancestor.schema1.json` | rename source; change family labels while retaining shared ancestor; same family/different sensor; same experiment/different family; remove assessed source or reference while another present closure retains a known development/self overlap | holdout `does_not_meet_protocol`; exact shared IDs retained; missing-prerequisite exclusion never hides the visible overlap |
| E-R06 | E-AC06 | E-T11 | `dataset/unknown_separation.schema1.json`; `lineage/missing_ancestor.schema1.json`; `lineage/cycle.schema1.json`; `lineage/root_mismatch.schema1.json` | absent transitive ancestor; Unknown scope/family; reachable cycle; absent assessed root; root identity mismatch; root direct-dependency mismatch; dependency-kind mismatch; add an unknown field/tag separately to catalog node, identity, dependency, each object-valued experiment-scope payload, `ScopeKey::Specific`, and `ArtifactAcquisitionFamilies::Known` | transitive/Unknown/cycle cases yield exact unknown reason; validation excludes and holdout indeterminate; root/kind cases hard-fail; every nested unknown fails the additive strict reader; an explicit same-byte call to existing `read_artifact_lineage_catalog` retains its exact baseline result—acceptance for the currently permissive nested struct fields and existing rejection where the external-tag parser already rejects; no fabricated ID/family |
| E-R07 | E-AC07 | E-T12 | `dataset/reference_authority.schema1.json`; `reference/complete_sources.schema1.json`; `lineage/reference_intermediate.schema1.json` | direct assessed dependency; reference→reference→assessed; reference→Known artifact X→assessed; reference→development source; development reference→evaluated source; missing X; X identity mismatch; incomplete graph; unknown node; unblinded; wrong method/authority; quantified measure/unit/maximum mismatch; unavailable uncertainty under each rule | exact combined-closure evaluated-vs-development and assessed-vs-reference overlap/unknown/hard result; complete ineligible authority receives the exact section 3.7 exclusion ordinal; unavailable-allowed forces indeterminate |
| E-R08 | E-AC08 | E-T13 | `mechanism/all_levels.schema4.json`; `dataset/mechanism_reference_cross_product.schema1.json`; `expected/mechanism_mapping.md` | enumerate five Phase-B levels × four reference outcomes under software-exclusive support; absent hypothesis; definition/current ID mismatch; duplicate under either ID; change support-level set | exact section 3.3 joint category/reason/exclusion mapping; reference contradiction falsifies, reference not-assessed never supports, unavailable excludes only in this software-exclusive fixture; physical unavailable is the E-T29 hard case; no reassessment |
| E-R08 | E-AC08 | E-T14 | `mechanism/all_levels.schema4.json`; `dataset/mechanism_reference_cross_product.schema1.json`; `expected/mechanism_mapping.md`; `expected/fixed_metric_ledger.md` | zero eligible; one Phase-B contradiction with missing/ineligible reference; one independent-reference contradiction among high support; repeated renamed source | exact fractions/intervals or unavailable; Phase-B contradiction remains in declared falsification set despite exclusion; eligible reference contradiction fails; duplicate rejected |
| E-R09 | E-AC09 | E-T15 | `health/all_status_reference_pairs.schema4.json`; `dataset/health_confusion.schema1.json`; `expected/health_mapping.md` | enumerate four evaluable statuses × two reference classes plus Indeterminate/DQI | exact six disjoint ID sets and eligible/evaluable invariants |
| E-R09 | E-AC09 | E-T16 | `health/all_status_reference_pairs.schema4.json`; `dataset/health_confusion.schema1.json`; `expected/health_mapping.md`; `expected/fixed_metric_ledger.md` | zero positives; zero negatives; all Indeterminate; all DQI; threshold equality; label outside universe | exact unavailable/defined metrics, missing-state rates, balanced accuracy, and IDs; outside-universe hard binding error |
| E-R10 | E-AC10 | E-T17 | `expected/wilson_vectors.schema1.json`; `expected/cross_platform_numeric_bytes.txt` | x=0; x=n; n=1; 5/10; 50/100; n=2^53; n=2^53+1; x>n; input/computed negative zero; perturb expected bit/decimal/string | macOS V1 exact decimal <=1e-12 and exact `to_bits`/serialized tokens through 2^53; larger/x>n/negative-zero input hard-fail; computed zero is +0; perturbation fails |
| E-R11 | E-AC11 | E-T18 | `dataset/strata.schema1.json`; `protocol/software_valid.toml`; `protocol/physical_valid.toml`; `expected/strata_ledger.md` | each of six predicate variants; repeated/conflicting axis; invalid/overlapping temperature band; zero minimum; empty overall/stratum; record-underpowered; family-underpowered; actual one-family physical view; overlapping strata; aggregate pass | invalid protocol cases hard-fail; each membership literal matches; every overall/stratum underpowering including one-family physical is indeterminate and forces parent; no pooling/rescue |
| E-R12 | E-AC12 | E-T19 | `expected/acceptance_truth_table.schema1.json`; `expected/exclusion_precedence.schema1.json`; `protocol/software_valid.toml` | each exclusion condition alone and every pair; not-applicable; holdout overlap/unknown; equal threshold; false only; unavailable only; false+unavailable in both rule orders; critical contradiction | exact primary/secondary reason ordinals, decision transition, endpoint result, and sorted reasons independent of input/rule order |
| E-R13 | E-AC13 | E-T20 | `report/valid.schema1.json`; `expected/accounting_ledger.md`; `expected/fixed_metric_ledger.md` | change each count, ID set, exclusion, family count, rule result, claim, or overall status separately | report validation rejects every inconsistent mutation |
| E-R13 | E-AC13 | E-T21 | `report/valid.schema1.json`; `protocol/software_valid.toml`; `dataset/software_valid.schema1.json`; `expected/report_identity_preimage.jcs`; `expected/golden_bundle_file_sha256s.txt`; `expected/escaping_vectors.schema1.json` | change clock/path/output/staging; reorder source; change normalized protocol/hash/source/trust authority; perturb numeric bit; exercise comma/quote/LF/backslash/pipe/non-ASCII/empty/null/NA/negative-zero cells; mechanism cohort row in health columns; not-applicable leakage; domain JSON | operational values absent; authority/numeric changes alter identity or fail validation; exact JSON/CSV/Markdown bytes/escaping/NA projections on macOS V1 |
| E-R14 | E-AC14 | E-T22 | `expected/golden_bundle/mhi_validation_report.schema1.json`; `expected/golden_bundle/validation_execution_manifest.schema1.json`; `expected/golden_replace_execution_manifest.schema1.json`; `expected/golden_bundle/validation_summary.md`; `expected/golden_bundle/tables/cohort_coverage.csv`; `expected/golden_bundle/tables/leakage_assessment.csv`; `expected/golden_bundle/tables/mechanism_validation.csv`; `expected/golden_bundle/tables/health_validation.csv`; `expected/golden_bundle/tables/exclusion_ledger.csv`; `expected/golden_bundle/tables/compatibility_matrix.csv`; `expected/golden_bundle_file_sha256s.txt` | fail each staging write/fsync/checksum/reread/authority validation; add manifest self-record; substitute wrong `create_new`/`replace_managed_bundle` golden; add timestamp/extra field/file | no final partial output; exact create-new or replace manifest plus the same eight scientific files; every mutation rejected; manifest records exactly eight non-self files |
| E-R14 | E-AC14 | E-T23 | `expected/golden_bundle/mhi_validation_report.schema1.json`; `expected/golden_bundle/validation_execution_manifest.schema1.json`; `expected/golden_replace_execution_manifest.schema1.json`; `expected/golden_bundle/validation_summary.md`; `expected/golden_bundle/tables/cohort_coverage.csv`; `expected/golden_bundle/tables/leakage_assessment.csv`; `expected/golden_bundle/tables/mechanism_validation.csv`; `expected/golden_bundle/tables/health_validation.csv`; `expected/golden_bundle/tables/exclusion_ledger.csv`; `expected/golden_bundle/tables/compatibility_matrix.csv`; `publication/unmanaged_bundle/sentinel.txt`; `expected/publication_state_table.schema1.json` | copy the nine create-new goldens to the test-owned managed output, then force lock contention; existing stage/backup/symlink; unmanaged output; concurrent create after preflight; no-replace unsupported/failure; exchange unsupported/failure; replace the managed output namespace before the final precheck; after that precheck but before exchange, atomically replace it with the unmanaged sentinel tree; separately mutate a managed old-generation file through the held identity in that interval; after exchange and before the ordered proof, separately replace then same-inode-mutate the newly visible output; crash before/after every parent fsync and each generation proof; stage→backup failure; each reverse-order deletion failure including partial backup | precheck replacement hard-fails uncommitted `PublicationConcurrentManagedOutputChanged` and leaves the competitor at output; precheck-to-exchange old-generation replacement/mutation returns `PublicationCommittedForeignSwapDetected` and preserves it whole at stage; post-exchange new-generation replacement/mutation returns `PublicationCommittedVisibleOutputChanged` and preserves the entire old stage without cleanup; each returns exact identity/fingerprint tokens and no success; every other case matches the section 4.5 error/commit/residue table; successful overwrite uses the replacement manifest; exchange never leaves output absent; every metadata transition is directory-fsynced |
| E-R15 | E-AC15 | E-T24 | `source_guards/forbidden_dependencies.txt` | add each forbidden assessor/reverse import token to temporary source sample | guard fails for Phase-B assessor, Phase-C assessor, and Phase-D→E dependency |
| E-R15 | E-AC15 | E-T25 | `phase_d_golden/input_manifest.txt`; `phase_d_golden/file_sha256s.txt` | run Phase D before/after additive Phase-E build; mutate one expected byte | identical bytes pass; mutation fails |
| E-R16 | E-AC16 | E-T26 | `dataset/software_valid.schema1.json`; `report/valid.schema1.json`; `approval/valid.schema1.json`; `trust/test_only_known_answer_trust_store.schema1.json`; `trust/test_only_invalid_identity_weak_key.schema1.json` | missing/wrong kind; schema 0/2; unknown/duplicate nested field; nonfinite; negative zero; invalid tag/token/signature length; missing/unknown `provisioning_state`; `UNPROVISIONED` with roots or `PROVISIONED` with empty roots; rename `trust_roots` to `roots`/`approval_roots`; unsorted/duplicate provisioned roots; same owner/registry authority ID in one root; duplicate authority ID across roots; same owner/registry canonical key in one root; duplicate canonical key across roots/roles; 31/33-byte key; exact nondecompressible y=2 point `0200000000000000000000000000000000000000000000000000000000000000`; exact ZIP-215-decodable noncanonical identity alias `eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f`; unused canonical identity key `0100000000000000000000000000000000000000000000000000000000000000` | exact schema-1/strict-reader acceptance and typed rejection matrix; only `UNPROVISIONED` with `[]`, or exact `PROVISIONED` `trust_roots` bytes with globally distinct IDs/canonical keys and every key passing `from_bytes`, recompression equality, and `is_weak()==false`, pass; y=2 hard-fails `PhysicalApprovalPublicKeyInvalid`, the alias hard-fails `PhysicalApprovalNoncanonicalPublicKey`, and unused identity hard-fails `PhysicalApprovalWeakPublicKey`; no old reader behavior change |
| E-R16 | E-AC16 | E-T27 | `compatibility/existing_artifact_fixture_inventory.schema1.json`; `compatibility/existing_artifact_matrix.md` | inventory must equal the literal historical set below; flip one expected kind/schema/acceptance/byte-hash cell at a time | every historical fixture retains exact baseline result/bytes; any expectation mutation fails |
| E-R17 | E-AC17 | E-T28 | `protocol/software_valid.toml`; `dataset/synthetic_perfect.schema1.json`; `approval/none.txt` | perfect metrics; relabel filename/method as physical | only `software_validated_only`; origin is never inferred; physical outcome impossible |
| E-R17 | E-AC17 | E-T29 | `protocol/physical_valid.toml`; `dataset/physical_valid.schema1.json`; `dataset/physical_selective_unavailable.schema1.json`; `approval/valid.schema1.json`; `approval/valid_selective_unavailable.schema1.json`; `approval/invalid_self_signed.schema1.json`; `approval/invalid_identity_forgery.schema1.json`; `trust/test_only_known_answer_trust_store.schema1.json`; `trust/test_only_invalid_identity_weak_key.schema1.json`; `reference/complete_sources.schema1.json` | invoke the production CLI physical route while its embedded production store is `UNPROVISIONED`; then, only through the approved test-only pure-verifier boundary, use the literal known-answer store and mutate: missing approval; wrong purpose; unknown root; dataset/protocol-supplied attacker key; wrong owner/registry key; copy the owner signature into the registry field; malformed/noncanonical-scalar/one-signature payload; set the selected owner key to identity encoding `0100000000000000000000000000000000000000000000000000000000000000` and its signature to identity-R/zero-S `01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000` over an arbitrary altered payload, then apply the same identity-key mutation separately to the registry role; replace either selected role separately with nondecompressible y=2 key `0200000000000000000000000000000000000000000000000000000000000000`; wrong file/record/cohort/protocol/claim/endpoint/domain/origin/authority binding; missing/legacy assessed source or reference; synthetic/unknown origin; disallowed reference method/authority; unblinded/unquantified/over-limit reference; incomplete reference node; use the precommitted mutually hash-bound dataset/approval pair containing 100 fully authoritative/physical/blinded/quantified/complete mechanism records, 98 semantic outcomes `unavailable`, and only two supporting records/families; declared minimum <2; actual one family/missing stratum; valid dual-signed two-family case; attempt every CLI/config/environment/protocol test-root-selection route | the production CLI always hard-fails a physical request with `PhysicalApprovalTrustNotProvisioned` before dataset/scoring and cannot select test roots; only the exact no-feature `ed25519-dalek 2.2.0` `from_bytes`/recompression/`is_weak`/`Signature::try_from`/`verify_strict` sequence is accepted by the test-only pure verifier; each literal identity forgery hard-fails `PhysicalApprovalWeakPublicKey` before signature verification, each y=2 key hard-fails `PhysicalApprovalPublicKeyInvalid`, copied/malformed/strict-invalid role signatures hard-fail their role-specific error, and all test-store/schema/binding/origin/physical-reference-authority failures hard-fail before exclusion or scoring; the independently valid dual-signed 98-outcome fixture hard-fails `PhysicalReferenceOutcomeUnavailable` with no report, proving the result is not merely an approval-hash mismatch; actual family/record underpowering is indeterminate; only the exact test-boundary dual-verified named passing case emits `physically_validated`, and it is software conformance evidence only |
| E-R18 | E-AC18 | E-T30 | `expected/phase_e_fixture_inventory.schema1.json`; `expected/author_validation_evidence_ledger.md` | compare inventory to every regular file under `tests/fixtures/phase_e`; require each inventory row's E-R/E-AC/E-T/mutation/oracle to equal this section; omit/stale one committed author command/result, fixture row, mapping, dependency/lock audit, or P0/P1 author disposition; add a candidate commit SHA or an independent GO to the committed author ledger; change the direct Dalek declaration/features, any of the six section-4.6 new lock versions/checksums/edges, or any pre-existing lock entry; after freeze, make the canonical approval record stale, incomplete, mismatched, self-approving, or unrelated to the exact `REVIEW_SHA`; mutate any required role decision, P0/P1 count, P2 disposition, macOS result, command result, tag target, tag absence-before-creation check, or remote-tag verification | any missing/extra/duplicate/unmapped fixture, stale committed author evidence, self-approval, forbidden feature, dependency drift, old-lock drift, or incomplete/mismatched approval evidence fails its applicable gate; each role decision is exactly `GO` or `NO-GO`, all four must be `GO`, zero P0/P1 and complete P2 disposition are required, the canonical `PhaseEImplementationApprovalV2` body must match byte-for-byte, the `ism-mechanism-health-v1-e-implementation-approved-r5` tag must target the exact `REVIEW_SHA`, the tag must be absent before creation, the remote tag must be verified after creation, and any stale SHA or target/body mismatch blocks |

For E-T29, “the same identity-key mutation separately to the registry role”
means replacing both that role's key and that role's signature with the same
literal identity-key and identity-R/zero-S bytes displayed in the row; the
other role remains the unmodified valid fixture role. The owner case is
constructed analogously. Store weak-key validation must select the named role's
`PhysicalApprovalWeakPublicKey` before either signature is evaluated.

`expected/phase_e_fixture_inventory.schema1.json` is a closed canonical array
with one row for itself and every other Phase-E fixture: `{ relative_path,
mappings:[{ requirement_id, acceptance_criterion_id, test_id,
mutation_case_ids, expected_result_id }] }`. Mappings are nonempty and sort
uniquely by the complete five-field tuple, so a shared golden may map to every
test that consumes it. E-T30 uses a deterministic no-follow recursive walk,
rejects any symlink/non-file leaf, and compares its sorted regular-file
relative-path set byte-for-byte with the
inventory; reviewers independently cross-check with
`rg --files tests/fixtures/phase_e`. Aliases, directories, globs, “same as,”
and “all fixtures” are forbidden values.

`expected/author_validation_evidence_ledger.md` is the other committed E-T30
artifact. It contains the exact Phase-E fixture inventory reference; complete
requirement/AC/test/fixture/mutation/oracle mapping; exact dependency and lock
audit; exact commands required for author validation; author-side command
results; and author-side P0/P1/P2 dispositions. It contains no implementation
candidate commit SHA, no `REVIEW_SHA`, no independent-review record, and no GO
approval. It is evidence frozen with the candidate, not self-approval.

The required post-freeze E-T30 record is one external, non-self-referential,
annotated Git tag on `origin`:
`ism-mechanism-health-v1-e-implementation-approved-r5`. It is never committed
back into `REVIEW_SHA`. The tag is created by the project owner/release
coordinator and is both the R5 implementation-approval marker and the
canonical external review-evidence record. It is not represented as four
reviewer signatures, and it does not claim cryptographic reviewer identity or
server-enforced tag protection. The old R4 tag is historical only.

E-T30 requires all of the following for the exact same `REVIEW_SHA`:

- four independent scientific, architecture, security, and compatibility
  reviews, each with a canonical `GO` decision;
- zero unresolved P0/P1 findings and complete disposition of both retained P2
  findings;
- final macOS exact-SHA validation with result `PASS`;
- the exact canonical `PhaseEImplementationApprovalV2` body, including its
  exact sorted R5 `plan_tags` value, exact latest-plan SHA-256, and exact
  implementation SHA;
- the exact latest approved R5 plan bytes merged into the implementation
  branch with `--no-ff`;
- the approval tag targeting exactly `REVIEW_SHA`;
- proof that the approval tag was absent locally and remotely before creation;
- normal push followed by immediate remote target/body verification;
- no implementation-author self-approval; and
- no stale review SHA, tag target, tag body, plan-tag list, role decision, or
  validation result.

The former R3 four-role cryptographic tag mechanism and the R4 V1 record are
historical context only. Under current R5, a missing, stale, retargeted,
mismatched, self-approving, or non-`GO` V2 approval record blocks approval and
integration. A complete accepted non-blocking P2 disposition remains visible in
the exact P2 fields while all four role decisions remain `GO`. Linux remains
deferred and non-gating under the preserved R3 macOS-only contract.

`trust/test_only_known_answer_trust_store.schema1.json` is a literal test-only
`PROVISIONED` store passed directly to the pure verifier by E-T26/E-T29. It is
not a copy of `config/mhi_physical_approval_trust_store.schema1.json`, is never
embedded in or opened by the production CLI, and cannot change which authority
the binary trusts. The embedded production file is `UNPROVISIONED` until a real
separately reviewed production provisioning occurs. The test-only file may
contain public test-vector keys solely as software-conformance inputs; those
keys never become production physical authority.

No owner/registry private key, seed, signer, signing capability, or
fixture-regeneration helper is committed. `approval/valid.schema1.json` and
`approval/valid_selective_unavailable.schema1.json` are immutable literal
known-answer signatures created offline, outside the production and test
runtimes, and bound to their exact cohort fixtures. Only public test keys,
signed literal approvals, literal protocol/dataset/reference files, and
expected hashes/errors/results may be committed. The fixture authority retains
private material outside the repository or destroys it after authoring; tests
verify the literals and never regenerate them.

Golden scientific and output fixtures are independent authorities, not an
output-update mechanism. Literal source fixtures are constructed from this
approved plan; expected scientific ledgers are hand-derived or independently
derived; and byte goldens may be assembled by an independent fixture-authoring
process. Production output may be compared against them but must never
overwrite, update, or regenerate their oracle values. `UPDATE_GOLDENS=1` or
any equivalent production-generated-oracle path is forbidden. Independent
review confirms golden bytes against this plan.

The historical compatibility set mapped to E-R16 → E-AC16 → E-T27 is exactly:

```text
tests/fixtures/a0_artifact_contracts/eis_fit_schema2_correct_kind.json
tests/fixtures/a0_artifact_contracts/eis_fit_schema2_wrong_kind.json
tests/fixtures/a0_artifact_contracts/health_baseline_schema2_correct_kind.json
tests/fixtures/a0_artifact_contracts/health_baseline_schema2_wrong_kind.json
tests/fixtures/a0_artifact_contracts/schema1/calibration_analysis.schema1.json
tests/fixtures/a0_artifact_contracts/schema1/calibration_model.schema1.json
tests/fixtures/a0_artifact_contracts/schema1/calibration_observations.schema1.json
tests/fixtures/a0_artifact_contracts/schema1/health_assessment.schema1.json
tests/fixtures/a0_artifact_contracts/schema1/health_trend.schema1.json
tests/fixtures/a0_artifact_contracts/schema1/mechanism_analysis.schema1.json
tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json
tests/fixtures/a0_artifact_contracts/schema1/transient_analysis.schema1.json
tests/fixtures/a0_artifact_contracts/schema2/calibration_analysis.schema2.json
tests/fixtures/a0_artifact_contracts/schema2/calibration_model.schema2.json
tests/fixtures/a0_artifact_contracts/schema2/calibration_observations.schema2.json
tests/fixtures/a0_artifact_contracts/schema2/health_assessment.schema2.json
tests/fixtures/a0_artifact_contracts/schema2/health_trend.schema2.json
tests/fixtures/a0_artifact_contracts/schema2/mechanism_analysis.schema2.json
tests/fixtures/a0_artifact_contracts/schema2/signal_analysis.schema2.json
tests/fixtures/a0_artifact_contracts/schema2/transient_analysis.schema2.json
tests/fixtures/artifact_contracts/eis_fit_schema2_missing_kind.json
tests/fixtures/artifact_contracts/health_baseline_schema2_missing_kind.json
tests/fixtures/a1/current_known_lineage_state.json
tests/fixtures/a1/legacy_lineage_state.json
tests/fixtures/estimation_migration/legacy_simulation_truth_v2.json
tests/fixtures/estimation_migration/legacy_state_estimation_report_v1.json
tests/fixtures/estimation_migration/legacy_state_filter_comparison_v2.json
tests/fixtures/estimation_migration/legacy_state_validation_v1.json
tests/fixtures/phase_c/writer_boundary/legacy_health_assessment_v3.json
tests/fixtures/phase_d/base/calibration.json
tests/fixtures/phase_d/base/calibration_observations.json
tests/fixtures/phase_d/base/eis.json
tests/fixtures/phase_d/base/estimation.json
tests/fixtures/phase_d/base/health.json
tests/fixtures/phase_d/base/lineage_catalog.json
tests/fixtures/phase_d/base/mechanism.json
tests/fixtures/phase_d/base/model.json
tests/fixtures/phase_d/base/signal.json
tests/fixtures/phase_d/base/transient.json
tests/fixtures/phase_d/failure/catalog_duplicate_root_key.json
tests/fixtures/phase_d/failure/catalog_invalid_structure.json
tests/fixtures/phase_d/failure/catalog_key_identity_mismatch.json
tests/fixtures/phase_d/failure/catalog_malformed.json
tests/fixtures/phase_d/failure/catalog_schema2.json
tests/fixtures/phase_d/failure/eis_schema2.json
tests/fixtures/phase_d/failure/wrong_kind.json
tests/fixtures/phase_d/legacy/health_v3.json
tests/fixtures/phase_d/legacy/mechanism_v1.json
```

All Markdown ledgers, JCS preimages, numerical vectors, truth tables, trust/
signature vectors, golden bytes, and expected statuses are hand-derived and
independently reviewed from literal fixture content. Production implementation
must not generate or update them. The mutation list above is exhaustive for
every normative condition in sections 3 and 4; adding a condition requires an
explicit ledger mutation and inventory row in the same main-only plan revision.

### 7.4 Scientific validation

Scientific validation has two non-substitutable gates:

1. **E-SW — algorithmic/software gate.** Frozen synthetic and constructed
   fixtures establish exact metric math, leakage handling, missing-data
   behavior, determinism, and artifact compatibility. Passing E-SW authorizes
   only “software validated for the tested contract.”
2. **E-SCI — physical holdout gate.** The section 3.8 immutable approval file
   binds the exact pre-scoring registered protocol, cohort semantic hash,
   physical/blinded reference authorities, claims, endpoints, and domain. The
   runtime verifies canonical, nonweak, globally distinct owner and registry
   Ed25519 keys and both strict signatures against its reviewed embedded trust
   store before scoring; reviewers additionally verify the frozen store/review
   evidence. Only strict dual-verified fully bound, domain-equal claims
   whose required endpoints/strata pass may be described as physically
   validated for that declared domain.

If no qualifying physical cohort is available, implementation may still
finish E-SW using explicit software claims. Such a report may say
`software_validated_only`, `does_not_meet_protocol`, or `indeterminate`, never
`physically_validated`. A physical-request protocol without approval is
rejected before scoring rather than silently downgraded.

### 7.5 Regression validation

Before review, run from a clean worktree pinned to the approved implementation
commit:

```bash
git diff --check
cargo fmt --all --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
cargo test --locked --all
cargo test --doc --locked
cargo build --locked --release
```

Also require:

- all permanent A0/A1/Phase B/Phase C/Phase D tests pass unchanged;
- all historical artifact migration fixtures retain their prior accept/reject
  behavior;
- golden Phase D bundles from identical baseline inputs remain byte-identical;
- existing CLI help/parse snapshots and legacy commands remain compatible;
- the two new artifact-kind variants do not alter serialization of any existing
  `ArtifactKind`; and
- macOS exact-SHA validation and the locked regression suite pass. Generic
  repository Linux/Ubuntu CI may remain, but its result is informational and
  non-gating for Phase-E approval.

Any baseline test modification requires a written compatibility justification
and independent approval. Deleting, weakening, renaming, or replacing a
baseline scientific test is a release blocker.

## 8. Git workflow and review gate

### 8.1 Phase-E baseline and durable branch policy

Phase E is based on the current synchronized `main` at Phase-E
initialization:

```text
PHASE_E_BASELINE_MAIN_SHA=6b76258ff2e8ff71a1b8a68248b47cf224141d73
```

The historical Phase-D release tag
`ism-mechanism-health-v1-d-complete` at
`2316fb1d076d47ac14d3b3c10c55654feb1ffb54` remains a provenance ancestor of
this baseline. Phase E must verify that ancestry, but must not require
`main` to equal the release SHA, check out that historical commit as its base,
or reset `main` backward to it.

`main` is the only durable development branch. The steady state before,
during planning, and after Phase-E integration is:

```text
local branches:  main only
remote branches: main only
```

### 8.2 Main-only planning and frozen plan review

Planning occurs directly on `main`. There is no
`plan/mhi-v1-e-independent-validation` branch, review branch, or planning
remediation branch. The Phase-E plan is one coherent documentation-only commit
on `main`; it must not modify `src/`, `config/`, `tests/`, `Cargo.toml`,
`Cargo.lock`, CI, or existing artifacts/fixtures.

The R5 plan-only commit is authored and pushed before any independent review.
After that push, freeze the exact review target from the actual remote state:

```bash
git fetch origin --prune
PLAN_REVIEW_SHA="$(git rev-parse origin/main)"
```

Independent reviewers must review exactly `PLAN_REVIEW_SHA`. They may use
detached `HEAD` for isolation, but no review branch is created. No additional
commit is made on `main` until this review completes. The review must record
all of the following as GO:

- independent scientific review of endpoint definitions, reference authority,
  independence/leakage rules, metric denominators, Wilson method, strata,
  physical holdout design, and claim wording;
- independent architecture review of module boundaries, canonical readers,
  closed schemas, semantic identity, lineage, atomic publication, and rollback;
- independent security review of embedded trust-store provenance, exact
  Ed25519 verification/signing bytes, key isolation, malformed-signature
  rejection, and dual owner/registry authority;
- compatibility review proving no existing schema, artifact, route, output, or
  test behavior is changed;
- project-owner approval of the claim vocabulary and section 3.8 approval
  schema. Approval of an actual physical protocol/cohort is required later
  only for a physical claim and is not a prerequisite for software-only
  implementation;
- a requirement-to-test/fixture traceability matrix with zero unmapped
  requirements, criteria, fixtures, or tests; and
- zero open P0/P1 findings and explicit disposition of all P2 findings.

The current R5 plan review must record `P0=0` and `P1=0`, complete both P2
dispositions, and independently review the exact frozen plan bytes. R4
review-evidence carry-forward is not a current option: the R5 implementation
requires four fresh role reviews after the implementation branch absorbs the
approved R5 plan. A documentation self-review is not sufficient for either
gate.

After independent R5 GO, create the immutable
`ism-mechanism-health-v1-e-plan-approved-r5` tag at exactly the independently
reviewed `PLAN_REVIEW_SHA`. The approved R5 plan SHA must then be merged into
the existing implementation branch with normal `git merge --no-ff`; no current
R5 workflow skips that plan merge. The earlier base, R2, R3, and R4 plan tags
remain immutable historical predecessor authorities.

At the end of plan review, fetch `origin` again. If `origin/main` advanced,
record the later commits as unreviewed; the frozen review remains valid only
for `PLAN_REVIEW_SHA` and no approval tag moves to the later SHA. Before an
implementation branch is created, reconcile current main by independently
reviewing or explicitly excluding every later commit from Phase-E authority.
The implementation base must contain the exact approved plan plus only later
commits with recorded acceptance; otherwise implementation remains blocked.

### 8.3 The single temporary implementation branch

Only after the plan-approval tag exists, create exactly one temporary branch,
`codex/mhi-v1-e-independent-validation`, from current `main` containing the
approved plan. No implementation-remediation, review, or integration branch
is permitted. If remediation is required, it is made as forward commits on
this one temporary implementation branch.

Implementation is limited to the approved Phase E paths and contracts. Any
change to Phase B/C/D scientific semantics, any new metric, any schema field,
or any compatibility exception requires a new independent plan review of the
forward documentation change on `main`, rather than a planning branch.

Implementation should use coherent reviewable commits in this order:

1. artifact kinds, additive strict reader, result schemas, and negative schema
   fixtures;
2. protocol/dataset readers, embedded trust store, dual-signature approval, and
   leakage classification;
3. deterministic metrics and pure evaluator;
4. CLI runner and atomic outputs; and
5. full fixtures, traceability, compatibility evidence, and documentation.

### 8.4 Forward plan changes discovered during implementation

If implementation discovers that any Phase-E scientific rule, schema field,
metric, output byte, compatibility exception, or other approved contract must
change, this exact forward-only procedure is mandatory:

1. Pause implementation before making the contract-dependent code change.
2. Commit and push the current safe implementation work on the existing
   `codex/mhi-v1-e-independent-validation` branch. Do not create another branch
   and do not reset, rebase, amend, squash, or force-push it.
3. Switch to synchronized `main`, make the smallest forward documentation-only
   plan commit, push normally, freeze its exact `origin/main` SHA, and run a new
   independent plan review. No implementation code is committed to main.
4. The immutable `ism-mechanism-health-v1-e-plan-approved-r5` tag is the
   current R5 plan authority and never moves. Any later approved revision must
   receive a new explicitly numbered immutable plan tag at its reviewed SHA;
   the implementation review ledger names that latest tag and every
   predecessor. Every such revision is merged into the implementation branch
   with normal `--no-ff` history before implementation resumes.
5. Fetch the existing implementation branch, merge the newly approved `main`
   into it with a normal non-fast-forward merge, and push. Do not rebase.
6. Verify the merge contains the exact newly approved plan SHA and no
   unreviewed main commits. If main also contained unrelated later commits,
   each must have separate recorded acceptance before this merge. A
   no-plan-merge or carry-forward exception is invalid under the current R5
   workflow.
7. Resume implementation on the same branch. Its eventual review freezes a new
   exact remote implementation SHA and validates the entire cumulative diff
   against the pre-implementation main baseline under the latest plan tag.

A plan-review NO-GO leaves implementation paused. No stale-SHA loop is created:
each review remains authoritative for its exact immutable SHA, and later
commits are merely unreviewed until separately frozen.

### 8.5 Implementation review freeze

After the temporary implementation branch is pushed, the reviewer freezes the
exact remote commit to review:

```bash
git fetch origin --prune
REVIEW_SHA="$(
  git rev-parse origin/codex/mhi-v1-e-independent-validation
)"
```

Independent reviewers must review exactly `REVIEW_SHA`. If the branch advances
after that freeze, the review remains valid for `REVIEW_SHA`; later commits are
simply unreviewed. No stale-SHA stop loop is permitted.

Two non-interchangeable gates apply to this frozen candidate.

`READY_FOR_PHASE_E_IMPLEMENTATION_REVIEW=YES` is permitted when and only when:

- all production implementation required by the approved plan is complete;
- all author-executable E-SW tests are complete, all required literal fixtures
  and mutation oracles are committed, and author-side traceability is complete;
- local/macOS validation passes wherever it is executed, including the exact
  results recorded in the committed author evidence ledger;
- the implementation branch is committed, pushed, and clean;
- `REVIEW_SHA` is frozen at the exact remote branch commit; and
- the committed E-T30 author-side evidence is complete and contains zero
  self-approval.

This review-candidate gate does not require any independent GO or external
approval evidence. The author completes available macOS validation and pushes
the exact candidate; CI and independent validation then execute the required
macOS commands on that same `REVIEW_SHA`. The author must not fabricate or
imply a Linux V1 result. Generic Linux/Ubuntu CI, if it remains, is
informational and non-gating for Phase-E.

`READY_FOR_PHASE_E_IMPLEMENTATION_APPROVAL_AND_INTEGRATION=YES` is permitted
only after all of the following hold for the same exact `REVIEW_SHA`:

- four fresh independent scientific, architecture, security, and compatibility
  role reviews target the exact same `REVIEW_SHA`; R4 carry-forward is not
  permitted; all four canonical decisions are `GO`, with zero unresolved P0/P1
  and complete P2 disposition;
- macOS exact-SHA validation and every required macOS command pass, and the
  exact results are recorded in the closed `PhaseEImplementationApprovalV2`
  body;
- `ism-mechanism-health-v1-e-plan-approved-r5` targets the exact approved R5
  plan, and the implementation branch contains that exact plan SHA through a
  normal `--no-ff` merge;
- the single annotated `ism-mechanism-health-v1-e-implementation-approved-r5`
  tag is absent locally and remotely before creation, is created by the
  project owner/release coordinator exactly once, targets the exact
  `REVIEW_SHA`, is pushed normally, and is immediately re-fetched and verified
  for exact target and byte-equal body;
- no reviewer signature, reviewer key, or server-enforced tag protection is
  required or claimed;
- for every requested physical claim, E-SCI passes with a real dual-signed
  section 3.8 `PROVISIONED` production-root approval; an all-software release
  has no physical claim and is explicitly limited to E-SW wording;
- the full locked regression suite and Phase D byte-compatibility checks pass;
- source, schema, fixture, and generated-output diffs match the approved scope;
- no P0 or P1 remains; and
- the validated commit has an explicit rollback target at the pre-merge
  `main` commit.

Linux absence, failure, CI failure, or an unavailable Linux environment is not
an R5 blocker for approval or integration. Any voluntarily supplied Linux
result is informational and cannot override the macOS gate.

### 8.6 Integration and cleanup

After the four fresh role reviews and all other R5 gates give GO, create exactly
once the annotated
`ism-mechanism-health-v1-e-implementation-approved-r5` tag at the exact
reviewed implementation SHA, using the exact V2 body and the procedural
immutability invariant above. Re-fetch and verify the remote tag immediately
before merge. The old R4 implementation-approval tag remains untouched and
historical; it cannot authorize this merge. Merge that exact reviewed
implementation SHA into `main` using:

```bash
git merge --no-ff <EXACT_REVIEWED_SHA>
```

Validate the merge, push `main` normally, and create the immutable
`ism-mechanism-health-v1-e-implementation-integrated` tag at the integrated
`main` commit. After remote main and all approval/integration tags are
verified, delete the local and remote
`codex/mhi-v1-e-independent-validation` branch. The resulting steady state is
again local `main` only and remote `main` only.

Planning should use one coherent documentation commit before initial review;
if plan remediation is required, use as few forward documentation commits as
practical. Implementation should likewise prefer coherent commits rather than
a commit for every minor test or fix. Branches must not be created to
compensate for commit organization.

No Phase-E workflow step may use reset, rebase, amend, squash, force-push,
tag movement, or stash deletion. No implementation branch, approval tag, or
merge is created as part of this planning task.

## 9. Milestone exit criteria

Phase E is complete only when all of these statements are true:

1. Every E-SCI objective has a passing test or, only where a physical claim is
   requested, exact dual-signed section 3.8 approval evidence verified against
   real `PROVISIONED` embedded production roots and a passing physical-validation
   record, with no silent unavailable data. Test-only known-answer evidence is
   not a physical-validation record.
2. Both new artifact schemas, protocol/trust/approval authorities, source graphs,
   and the output bundle are closed, canonical, byte-deterministic,
   provenance-complete, and published under the section 4.5 state machine.
3. Every validation result is reconstructible by the authority-assisted API
   from serialized IDs/counts/exclusions plus the exact hashed protocol,
   dataset, trust store, and consumed sources.
4. Phase B/C/D artifacts remain immutable and their scientific semantics are
   unchanged.
5. All baseline and Phase E locked regression gates pass, with macOS exact-SHA
   validation as the only required V1 platform evidence. Generic Linux/Ubuntu
   CI, if retained, is informational and non-gating.
6. Independent scientific, architecture, security, and compatibility reviews
   freshly record `GO` for the exact reviewed SHA in the canonical
   `PhaseEImplementationApprovalV2` body and the
   `ism-mechanism-health-v1-e-implementation-approved-r5` tag. Accepted
   non-blocking P2 debt, if any, remains visible only in the complete P2
   fields.
7. Release language distinguishes software validation from physical validation,
   names the exact validated domain, and identifies the result as a
   macOS-supported V1 implementation without implying Linux or cross-platform
   support.

Until then, the correct milestone status is `planning`, `software-only`, or
`indeterminate` as applicable—never implied scientific validation.

## 10. Planning traceability contract

The implementation review must preserve these IDs. Splitting a test is
permitted only when every resulting test remains mapped; combining requirements
into one helper-only test is not sufficient.

| Requirement | Normative behavior | Planned owner | Acceptance criterion | Required tests | Explicit failure criterion |
|---|---|---|---|---|---|
| E-R01 | One additive certified `validation run` route with three required paths and no raw-input alternative. | `cli.rs`, `main.rs`, `runners/mhi_validation.rs` | E-AC01: Clap/runner accepts the exact valid route and rejects missing/unknown/conflicting input before evaluation. | E-T01, E-T02 | Existing command changes, a required flag is ignored, or a second Phase E route exists. |
| E-R02 | Protocol schema is the exact closed section 4.3 contract, hashed from exact bytes, with total class/rule/claim bindings, exact supporting-endpoint/claim domain equality, and no scientific defaults. | `protocol.rs`, `validation_config.rs` | E-AC02: every endpoint/claim field is present; invalid token, partition, metric-target pair, contradictory rule, role/domain/stratum, one-sided endpoint/claim domain containment, unknown/duplicate/nonfinite value fails before source read. | E-T03, E-T04 | Any endpoint, threshold, family minimum, class, stratum, reference rule, claim, or untested domain coverage is inferred/defaulted. |
| E-R03 | Dataset has exact canonical records, assessed-source deduplication, safe canonical non-symlink paths, combined catalog/reference graph, producer-owned semantic identity recomputation, declaration equality, cohort hash, and approval reference. | `results/mhi_validation.rs`, `reader.rs`, `approval.rs` | E-AC03: schema-1 round-trip is exact; every unsafe, duplicate, self-asserted semantic hash, mismatched root/declaration, malformed, or unbound authority fails at the section 4.2 hard boundary. | E-T05, E-T06 | Renaming double-counts a source; a path escapes; or an embedded hash is trusted without recomputation/equality to catalog, expectation, scope, and dependencies. |
| E-R04 | Scientific inputs use only the additive canonical duplicate-aware reader and exact Phase B/C schemas while the existing reader remains unchanged. | `reader.rs`, `domain/artifact.rs` | E-AC04: strict schema-4 mechanism/health inputs read; duplicate/wrong/future hard-fail; readable legacy is explicitly excluded; old API regressions are identical. | E-T07, E-T08 | A duplicate key is lost in `Value`, ad-hoc parsing occurs, existing reader semantics change, or future/legacy data are scored. |
| E-R05 | Every endpoint/view/record receives the exact section 3.2 declared/eligible/excluded/not-applicable decision, declared-denominator exclusion rate, and source-key uniqueness check. | `partition.rs` | E-AC05: sorted IDs prove declared=eligible+excluded; every record has one decision/reason; exclusion rate is excluded/declared or unavailable only at zero declared. | E-T09 | A record is dropped/double-counted, exclusion uses an eligible denominator, or role/domain is inferred from path/time. |
| E-R06 | Holdout leakage uses the required additive nested-strict canonical catalog reader plus the recursively combined assessed/reference/scientific-artifact closure, exact root-condition matrix, development comparators, scope, and families. | `partition.rs`, `reader.rs`, `domain/lineage.rs` | E-AC06: strict-vs-existing reader compatibility, root hard errors, and transitive shared/unknown/missing/cycle/aggregate/renamed/cross-graph cases return exactly one frozen state/reason. | E-T10, E-T11 | A nested catalog field bypasses Phase-E closure, an existing reader result changes, a reference scientific leaf is terminal, root failure is downgraded, shared/unknown holdout evidence passes, or an ancestor is fabricated/hidden. |
| E-R07 | Reference outcomes use exact method/authority/blinding/uncertainty rules, complete combined dependency closure, and non-selective mechanism outcome admission. | `reader.rs`, `partition.rs` | E-AC07: direct/cross-graph self-derivation cannot pass; unknown remains unknown; each protocol-ineligible authority receives its exact exclusion ordinal. | E-T12 | An inferred/unbound/unblinded/excessive/derived label enters a holdout numerator or a contradiction is selected away. |
| E-R08 | Mechanism validation applies the total Phase-B × independent-reference section 3.3 mapping without inspecting/recomputing Phase-B gates. | `mechanism.rs`, `statistics.rs` | E-AC08: all five states × four outcomes, ID binding/dedup, eligible category sets, declared falsification set, n=s+c+u, rates, absence, and both contradiction sources match literals. | E-T13, E-T14 | Assessor logic is called, definition/current IDs diverge, a Phase-B contradiction is selected away, reference contradiction passes, or unavailable becomes pass/zero. |
| E-R09 | Health validation applies exhaustive predicted/reference partitions and six-category section 3.4 accounting. | `health.rs`, `statistics.rs` | E-AC09: exact ID sets prove eligible=evaluable+Indeterminate+DQI; metrics/zero-class boundaries match hand vectors. | E-T15, E-T16 | Any status/label disappears, enters two categories, changes sign/class by inference, or undefined balanced accuracy is invented. |
| E-R10 | Wilson 95% is the sole V1 interval method and exact operation/bit/serialization contract in section 3.7. | `statistics.rs` | E-AC10: macOS V1 vectors agree within `1e-12` with independent decimals and exactly in `to_bits`/serialized bytes; the scientific formula and order are unchanged. | E-T17 | Another interval/correction/order appears, exact macOS V1 bits differ, or output is nonfinite/unclamped. |
| E-R11 | Closed six-variant strata and per-overall/stratum positive record/family minima remain visible and cannot be rescued by pooling. | `partition.rs`, `assessment.rs` | E-AC11: predicates/minima validate exactly; empty/below either minimum on overall or stratum makes endpoint indeterminate, including actual one-family physical evidence. | E-T18 | Duplicate/conflicting predicates pass or aggregate success hides empty/record/family underpowering. |
| E-R12 | Record exclusions and endpoint acceptance each use their complete ordered condition tables independent of rule/input order. | `partition.rs`, `assessment.rs` | E-AC12: every exclusion alone/pair, equality, unavailable+false, overlap, contradiction, endpoint/claim/overall composition returns exact primary/secondary reasons and status. | E-T19 | Short-circuit/order, omitted reason, rounding/tolerance, OR/default, or mixed-state ambiguity changes a decision. |
| E-R13 | Validation report is closed, finite, lineage/provenance-complete, byte-deterministic, and reconstructible through the explicit authority-assisted API. | `results/mhi_validation.rs` | E-AC13: structure plus exact hashed protocol/input/trust replay validates every set/count/metric/reason/outcome/source and JCS/golden bytes or rejects a one-field mutation. | E-T20, E-T21 | Standalone structure validation is mistaken for scientific approval, authority is absent, or clock/path/platform changes scientific bytes. |
| E-R14 | Exact nine-file output obeys closed field/cell bytes and the locked no-clobber/two-generation-bound exchange/fsync publication state machine. | `output.rs`, runner | E-AC14: success publishes exact bytes; every write/fsync/noreplace/exchange/race/partial-cleanup failure returns exact commit/residue state; unmanaged output present at preflight is untouched, a foreign old generation is preserved whole at stage, and a changed newly visible generation prevents all old-stage cleanup under its typed committed error. | E-T22, E-T23 | Manifest self-hash, timestamp, either exchange generation is unbound, foreign cleanup, output namespace gap, un-fsynced metadata, ambiguous residue, or unverified bytes occur. |
| E-R15 | Phase B/C assessors and Phase D projection/output remain unchanged and independent of Phase E. | source dependency guards; full baseline suite | E-AC15: source guards and baseline/golden tests prove no reverse dependency or output drift. | E-T24, E-T25 | Phase E reassesses/mutates sources or identical Phase D input produces changed public output. |
| E-R16 | New artifact kinds are additive and schema-1-only; existing serialization/migration remains exact. | `artifact.rs`, `artifact_contracts.rs` | E-AC16: new round trips and negative matrices pass while all existing fixture matrices retain prior results. | E-T26, E-T27 | Existing token/schema behavior changes or a Phase E future/legacy schema is silently accepted. |
| E-R17 | Software and physical requests are distinct. The production trust store is explicitly `UNPROVISIONED` or `PROVISIONED`; production roots are real independently controlled authority only, while literal test roots are accessible solely through the test-only pure verifier. A provisioned physical claim requires exact origin/blinding/quantification, usable mechanism semantic outcomes, domain-equal endpoints, globally separate owner/registry IDs and keys, and immutable dual signatures accepted only by the frozen strict verifier before scoring. | protocol/report/trust/approval schemas, assessment | E-AC17: an unprovisioned production request hard-fails before scoring; self-signed/synthetic/constructed/unknown/unapproved, weak-key-forged, same-key-dual-role, domain-overbroad, or semantic-outcome-unavailable input cannot emit `physically_validated`; test-only signatures cannot satisfy a release gate; actual underpowering is indeterminate; only a real provisioned strict dual-verified named passing claim can. | E-T28, E-T29 | A production test-vector root, dataset/protocol self-authentication, test-root selection route, weak/same key, one signature, approval/domain downgrade, or synthetic/unblinded/outcome-unavailable/underpowered evidence obtains a physical claim. |
| E-R18 | Full same-`REVIEW_SHA` CI and independent review evidence, exact Phase-E fixture inventory, committed author-side evidence, historical literal set, frozen dependency delta, and exhaustive mutation/oracle ledger gate approval/integration. | CI, committed author evidence, and the single canonical R5 implementation-approval tag | E-AC18: all 18 requirements, 18 ACs, 30 tests, every literal fixture/mutation/oracle and required command are mapped; committed author evidence has no candidate SHA or approval; and the canonical `PhaseEImplementationApprovalV2` record binds four fresh independent `GO` role decisions, macOS validation result, exact approved R5 plan authority, complete P0/P1/P2 disposition, and required command results to one exact `REVIEW_SHA`; approval/integration requires all four `GO`, zero P0/P1, complete P2 disposition, exact tag target/body, absent-before-creation proof, and remote verification. | E-T30 | Any filesystem fixture is absent/extra/aliased/unmapped, dependency drifts, author evidence self-approves or names its candidate SHA, or any command/review/tag/SHA is stale, mismatched, retargeted, self-approving, `NO-GO`, missing/unknown/third decision token, incomplete P2 disposition, nonzero P0/P1, or missing remote verification. |

### 10.1 Exact required test registry

| Test ID | Exact required test or evidence record |
|---|---|
| E-T01 | `phase_e_cli_runs_exact_certified_route` |
| E-T02 | `phase_e_cli_rejects_missing_unknown_and_raw_input_routes` |
| E-T03 | `phase_e_protocol_roundtrip_preserves_all_scientific_rules` |
| E-T04 | `phase_e_protocol_rejects_incomplete_conflicting_untrusted_and_nondeterministic_authority` |
| E-T05 | `phase_e_dataset_schema1_roundtrip_is_closed_and_canonical` |
| E-T06 | `phase_e_dataset_recomputes_semantic_identity_and_rejects_root_or_path_mismatch` |
| E-T07 | `phase_e_reader_accepts_only_canonical_schema4_scientific_inputs` |
| E-T08 | `phase_e_reader_hard_fails_wrong_future_and_explicitly_excludes_legacy` |
| E-T09 | `phase_e_partition_accounts_for_every_declared_record_exactly_once` |
| E-T10 | `phase_e_holdout_rejects_known_lineage_scope_and_family_overlap` |
| E-T11 | `phase_e_holdout_unknown_separation_is_indeterminate_without_fabrication` |
| E-T12 | `phase_e_combined_reference_catalog_closure_and_authority_are_total` |
| E-T13 | `phase_e_mechanism_phase_b_reference_cross_product_matches_hand_oracle` |
| E-T14 | `phase_e_mechanism_rates_intervals_and_ids_are_exact` |
| E-T15 | `phase_e_health_confusion_and_missing_state_counts_are_exact` |
| E-T16 | `phase_e_health_rates_boundaries_and_balanced_accuracy_are_exact` |
| E-T17 | `phase_e_wilson_95_decimal_bits_and_serialized_vectors_are_exact` — macOS V1 exact numeric authority plus independent decimal oracle |
| E-T18 | `phase_e_overall_and_closed_strata_apply_exact_record_and_family_minima` |
| E-T19 | `phase_e_exclusions_and_acceptance_use_complete_ordered_precedence` |
| E-T20 | `phase_e_report_reconstructs_every_count_from_source_ids` |
| E-T21 | `phase_e_authority_assisted_report_and_all_scientific_bytes_are_exact` — macOS V1 deterministic scientific-byte authority |
| E-T22 | `phase_e_publication_is_atomic_and_checksum_verified` — macOS V1 publication-state-machine authority |
| E-T23 | `phase_e_publication_is_locked_no_clobber_crash_durable_and_residue_exact` — macOS V1 publication-state-machine authority |
| E-T24 | `phase_e_source_guards_prohibit_reassessment_and_reverse_dependencies` |
| E-T25 | `phase_e_preserves_phase_d_golden_outputs_byte_for_byte` |
| E-T26 | `phase_e_artifact_contracts_accept_exact_schema1_and_reject_invalid_variants` |
| E-T27 | `phase_e_preserves_all_existing_artifact_migration_contracts` |
| E-T28 | `phase_e_synthetic_only_run_is_software_validated_only` |
| E-T29 | `phase_e_physical_claim_requires_dual_signature_embedded_trust_and_power` |
| E-T30 | committed exact-fixture inventory plus author-validation evidence; one exact `PhaseEImplementationApprovalV2` body in the annotated `ism-mechanism-health-v1-e-implementation-approved-r5` tag tying four fresh independent GO role reviews, exact R5 plan authority, and macOS platform/command results to exact `REVIEW_SHA` |

Planning traceability totals are 18 requirements, 18 acceptance criteria, and
30 required tests/evidence records. Unmapped requirements = 0; acceptance
criteria without a required test = 0; implementation is still prohibited until
the independent planning review confirms these mappings and the full schema.

### 10.2 Independent-review remediation ledger

This author-side ledger identifies where each blocking review concern must be
re-evaluated; it is not approval. Every row remains pending until a fresh
independent reviewer examines the same frozen plan bytes and records zero open
P0/P1 findings.

| Finding | Contract remediation requiring independent verification |
|---|---|
| P0-E-001 | Section 3.8 removes self-authentication: physical approval requires owner and registry Ed25519 signatures verified only against byte-embedded, independently reviewed trust roots; signed bytes bind protocol, cohort, claims, endpoints, authorities, domain, origin, immutable-document hashes, and purpose before scoring. E-T29 includes attacker-key, one-signature, malformed-signature, self-signed, and binding mutations. |
| P1-E-001 | Sections 3.5–3.6 define recursively complete assessed, reference, scientific-leaf, catalog, and development closures with exact hard/unknown/overlap treatment and available-source-hash authority. E-T10–E-T12 include cross-graph and development-reference derivation. |
| P1-E-002 | Sections 3.2, 3.3, and 4.3 define source-key deduplication, producer semantic recomputation, complete equality bindings, and exact Phase-B definition/current ID authority and duplicate rejection. |
| P1-E-003 | Section 3.5 makes a Known scientific reference leaf resolve into and recursively expand through the lineage catalog; the chain `reference -> X -> assessed/development` cannot terminate early or pass separation. |
| P1-E-004 | Sections 3.8 and 4.3 give the exact `trust_roots` field plus closed tagged endpoint, target, reference-rule, reference-endpoint, method, authority, uncertainty, dependency, metric, and binding schemas with canonical keys; third-round rows additionally freeze canonical/weak key validation and role separation. |
| P1-E-005 | Sections 3.6 and 4.3 freeze six stratum predicates, axis-conflict rejection, domain subset math, exact endpoint/claim domain equality, positive minima, physical minima of two, and overall/required-stratum underpower precedence. |
| P1-E-006 | Section 3.3 gives the total five-state Phase-B × four-state independent-reference mapping, prevents reference contradiction selection, retains declared Phase-B falsifications across exclusions, and never calls the Phase-B assessor. |
| P1-E-007 | Section 3.4 gives the exhaustive six-category Phase-C mapping, ID-set invariants, exact denominators, all rates, zero-class behavior, Wilson eligibility, and point-only balanced accuracy without calling the Phase-C assessor. |
| P1-E-008 | Sections 3.7–3.8 and the section 4.3 matrix freeze exclusion ordinals, secondary reasons, visible-leakage precedence, endpoint/outcome reason mappings, hard-boundary order, physical pre-scoring checks including mechanism semantic-outcome availability, and post-partition underpower treatment. |
| P1-E-009 | Sections 3.7 and 4.3 require additive raw-byte duplicate-aware artifact and nested-strict catalog readers, producer-owned semantic identity recomputation, exact count range, Wilson operation order, exact bits, and negative-zero rejection; structure-only validation cannot authorize publication. |
| P1-E-010 | Section 4.4 defines the complete report/manifest nested records, exact nine-file set, JSON/CSV/Markdown byte rules, every row/cell/null mapping, Markdown projection/escaping, and literal golden authority. |
| P1-E-011 | Section 4.5 defines a persistent exclusive lock, macOS V1 no-replace and exchange primitives, file/directory fsync points, commit boundaries, both-generation identity/fingerprint binding, foreign-swap/old-stage preservation, exact no-clobber behavior, and typed deterministic residue/cleanup states; the retained Linux `renameat2` path is explicitly deferred and non-authoritative. |
| P1-E-012 | Sections 3.8 and 7.4 separate software and physical gates: synthetic perfection and selectively excluded unavailable mechanism outcomes cannot produce a physical claim; only exact dual-verified, semantic-outcome-available physical holdout authority plus passing powered views can. |
| P1-E-013 | Section 8 preserves main-only durable workflow, forward-only plan changes on `main`, immutable review SHAs/tags, one post-approval temporary implementation branch, the distinct review-candidate versus approval/integration gates, and no stale-SHA loop; this R5 planning task creates only its prescribed forward documentation commit, not an implementation branch, approval tag, or merge. |
| P1-E-014 | Sections 7.3 and 10 map 18 requirements, 18 criteria, and 30 tests to literal fixture paths, mutation/oracle rows including selective physical outcome exclusion, nested catalog fields, strict-key forgeries/aliases/role reuse, both domain-containment directions, and overwrite replacement/mutation races, a filesystem-exact fixture inventory, the literal historical compatibility set, committed non-self-approving author evidence, and the single exact `PhaseEImplementationApprovalV2` record/tag with aliases/globs forbidden. |
| P0-R2-001 | Sections 3.3, 3.5, and 3.8 hard-fail any physical supporting mechanism reference whose semantic outcome is `unavailable` before partitioning; E-T29's exact 98-of-100 mutation proves such records cannot be selected out to manufacture a 2/2 physical pass. |
| P1-R2-001 | Section 3.8 names the closed root array field exactly `trust_roots`, forbids aliases, and freezes canonical root/ID/key validation for every selected and unused key; the third-round strict-verifier remediation below supersedes the rejected `ring` assumption. |
| P1-R2-002 | Sections 3.6 and 4.3 authorize the additive domain-owned `read_artifact_lineage_catalog_strict`, enumerate every permitted nested existing-wire key/tag, reuse one internal canonical text parser over the unchanged exact bytes, preserve the existing reader's result, and add same-byte strict-vs-existing mutations in E-T11. |
| P1-R3-001 (`P1-NEW-001`) | Section 3.8 replaces ordinary `ring` verification with the exact no-feature `ed25519-dalek 2.2.0` `from_bytes` → canonical recompression equality → `is_weak` → `Signature::try_from` → `verify_strict` contract, freezes the complete six-package lock delta/checksums, and makes E-T29 reject the literal identity-key/identity-R/zero-S arbitrary-message forgery. |
| P1-R3-002 (`P1-NEW-002`) | Section 3.8 requires owner and registry authority IDs and canonically recompressed public-key bytes to be globally unique across every role/root; noncanonical aliases fail before uniqueness, E-T26 mutates intra-root/cross-root reuse and a noncanonical point encoding, and E-T29 proves a copied role signature cannot satisfy the other distinct key. |
| P1-R3-003 (`P1-NEW-003`) | Section 4.3 requires mutual subset proof—exact equality—between every supporting endpoint domain and its claim domain; E-T04 rejects both one-sided directions, including a `{Pb}` endpoint supporting a `{Pb,Cd}` claim, so untested or pooled domains cannot inherit validation. |
| P1-R3-004 (`P1-NEW-004`) | Section 4.5 binds both staged-new and preflight-old generations to held directory identities and exact nine-file fingerprints, rechecks old immediately before exchange, validates newly visible output then swapped-old stage before cleanup, preserves old stage on either mismatch, freezes all three typed race errors, states the per-path post-proof writer threat boundary, and makes E-T23 force namespace-replacement and same-inode mutation races against both generations. |
