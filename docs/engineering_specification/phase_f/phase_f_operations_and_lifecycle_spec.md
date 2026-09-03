# Phase F Operations and Lifecycle Specification

## 1. Authority and adoption

This G2 candidate owns post-construction and operational semantics. It refines
`F-ARCH-010`, `F-ARCH-015..017`, `F-ARCH-021`, and `F-OD-14..20`. Exact JSON
and signatures remain Wire authority; scientific endpoint design remains
Scientific authority.

## 2. Requirements

| ID | derived_from | Normative requirement | Adopted R11 clauses |
|---|---|---|---|
| <a id="F-OPS-001"></a>`F-OPS-001` | `F-ARCH-016,F-ARCH-021` | Initial ACTIVE is valid only after the complete F5 candidate, five-role zero-P0/P1 GO review, final activation-bound claim-state record, and physical release approval. Execution or release alone cannot activate a claim. | §§5.1, 14, 17–19 |
| <a id="F-OPS-002"></a>`F-OPS-002` | `F-ARCH-016,F-OD-16,F-OD-17` | State transitions among ACTIVE, SUSPENDED, WITHDRAWN, EXPIRED, and SUPERSEDED follow the exact trigger/action table. Reinstatement is permitted only by the registered resolution mode and evidence; otherwise a new release or permanent withdrawal is required. | §§14–15 |
| <a id="F-OPS-003"></a>`F-OPS-003` | `F-ARCH-016,F-OD-17,F-OD-19` | Monitoring occurs at the approved cadence and evaluates all 15 exact metrics, source-kind bindings, thresholds, evidence, software/checker/trust/owner/release bindings, and registry acceptance. Missing, stale, unhealthy, mismatched, or unaccepted input suspends; no partial PASS exists. | §14, §§53.5–53.10 R11-07 |
| <a id="F-OPS-004"></a>`F-OPS-004` | `F-ARCH-016,F-OD-16` | Incident detection is append-only. Status at `audited_at` is derived from the ordered registry history; contained-before-terminal progression is exact. Open/uncontained, invalidly resolved, future, contradictory, or compromise incidents fail closed. | §15, §§53.3–53.4 R11-04 |
| <a id="F-OPS-005"></a>`F-OPS-005` | `F-ARCH-015,F-ARCH-016,F-OD-14,F-OD-15,F-OD-16` | Key compromise/revocation and registry compromise immediately remove affected authority. Emergency handling cannot bypass signatures, review, immutable Git publication, state consequences, or subsequent recovery/re-enrollment requirements. | §15 |
| <a id="F-OPS-006"></a>`F-OPS-006` | `F-ARCH-016,F-OD-18,F-OD-19,F-OD-20` | Retention membership is exact-set equality. Campaign is manifest plus every package object; protocol and other static release authorities remain outside the campaign set. Release retention adds every bound static authority, accepted PASS monitoring through `audited_at`, and applicable incident/resolution records, de-duplicated by kind/SHA. | §§15, 53.2–53.6, R11-03..06/R11-15 |
| <a id="F-OPS-007"></a>`F-OPS-007` | `F-ARCH-016,F-OD-20` | Every retained identity has the required primary plus backup copies, distinct immutable URIs, matching SHA/length, availability, freshness, access control, and authorized replacement. Failure suspends or blocks as applicable. | §§15, 44–46, 53.2–53.4 |
| <a id="F-OPS-008"></a>`F-OPS-008` | `F-ARCH-016,F-ARCH-021,F-OD-17,F-OD-18,F-OD-19,F-OD-20` | Claim currentness continuously requires an unexpired authority, a live non-equivocating registry and protected external reviewer-bootstrap head, monitoring PASS, no blocking incident/compromise, and valid retention. Loss of any prerequisite prevents ACTIVE use. | §§14–15 |

## 3. Review gate

P0/P1 must both be zero. Monitoring-result, incident progression, retention
membership, compromise/recovery, transition, or currentness ambiguity is P1.

## 3.1 External reviewer-bootstrap anchor operations

`PhaseFReviewerBootstrapExternalMonotonicHeadV1` is eventually provisioned in
one dedicated Git remote ref, `refs/heads/phase-f-reviewer-bootstrap-head`.
REAL protection pins are trusted only when embedded in an exact independently
reviewed and published normative Git revision. The production resolver
establishes the canonical GitHub repository identity through
`https://api.github.com` and independently verifies the exact pinned ACTIVE
ruleset before reading the ref. The graph-pinned `origin`/SSH alias is only an
operator push transport; live reads do not consult its local configuration.
The following sequence is normative and is required before any REAL reviewer
bootstrap is accepted:

1. **Stage A — mechanism approval.** The current generic `UNPROVISIONED`
   mechanism must receive fresh cumulative independent technical review with
   `P0=0` and `P1=0`, and that exact generic mechanism SHA must be published.
2. **Stage B — external ruleset creation.** Create and verify the real GitHub
   ruleset using the approved policy. Do not create the monotonic genesis ref
   yet. Through the canonical authenticated API, obtain the exact repository
   identity, ruleset ID, initial `version_id`, complete canonical ruleset
   state, and canonical state digest.
3. **Stage C — forward normative pinning revision.** Create a NEW forward
   normative Git commit replacing only the `UNPROVISIONED`/null pin values
   with the exact observed ruleset ID, `version_id`, state digest, and other
   required binding values. Regenerate every affected authority graph,
   specification-bundle manifest, traceability manifest, normative
   mirror/hash, and generated identity artifact. Run complete validation, but
   do not trust the pins yet. Submit that exact pinning SHA for fresh cumulative
   independent technical review; require `P0=0` and `P1=0`; then safely publish
   exactly the independently reviewed pinning SHA.
4. **Stage D — post-publication revalidation.** Re-query GitHub with the
   canonical authenticated API and require the real ruleset to retain the same
   repository identity, ruleset ID, `version_id`, complete digest, active
   policy, and complete bypass visibility. If anything changed, stop and make
   another forward pinning revision requiring fresh review and exact
   republication.
5. **Stage E — monotonic genesis.** Only after exact pinning-commit
   publication and successful post-publication revalidation may the dedicated
   monotonic ref/genesis head be created. Verify that it is protected by the
   already pinned ruleset, then continue the existing bootstrap-root and
   currentness ceremony.

The pin-authority contract is explicit:

```text
PIN_AUTHORITY_MODEL=FORWARD_NORMATIVE_GIT_COMMIT
PINNING_STAGE_A=INDEPENDENT_GO_PUBLISH_GENERIC_UNPROVISIONED_MECHANISM
PINNING_STAGE_B=CREATE_VERIFY_REAL_GITHUB_RULESET_BEFORE_MONOTONIC_GENESIS
PINNING_STAGE_C=NEW_FORWARD_COMMIT_REGENERATE_MIRRORS_FRESH_CUMULATIVE_REVIEW_PUBLISH_EXACT_SHA
PINNING_GATE=P0=0,P1=0
PINNING_STAGE_D=POST_PUBLICATION_REVALIDATE_SAME_ID_VERSION_DIGEST_POLICY_BYPASS_VISIBILITY
PINNING_STAGE_E=ONLY_AFTER_PUBLICATION_AND_REVALIDATION_CREATE_MONOTONIC_GENESIS
PIN_AUTHORITY_EQUALITY=LOCAL_REVIEWED_PINNING_SHA=PUBLISHED_MAIN_SHA=LIVE_REMOTE_MAIN_SHA
PIN_TRUST_PROHIBITION=NO_LOCAL_ENV_UNCOMMITTED_RUNTIME_EXTERNAL_MUTATION
```

No local edit, environment variable, operator configuration, uncommitted graph
change, runtime argument, or external mutable file may replace `UNPROVISIONED`
pins and create REAL authority. The exact pinning commit is authoritative only
when `local reviewed pinning SHA = published main SHA = live remote main SHA`
under the existing safe-publication procedure. A caller-selected local SHA,
tracking ref, or checkpoint cannot authorize freshness.

After Stage E, advance only by publishing one canonical successor with exactly
one Git parent, `sequence + 1`, and predecessor commit/head/hash bindings. The
server-side fast-forward/CAS result is final. Keep the local accepted-head
checkpoint outside the repository in an owner-only state directory. It is a
cache: a missing cache is reconstructed from the live external head, a lower
cache is repaired, and a fork or cache ahead of the live head rejects. State
paths use no-follow opens, ownership and mode checks, and an interprocess
exclusive lock around the read/compare/write transaction.

The lifecycle is therefore `GENESIS → ACTIVE successor → ACTIVE successor` on
the external ref. Deletion, rewind, force update, invalidation, protection
weakening, ruleset replacement, or server policy drift is an operational
incident and blocks REAL resolution until the external authority is restored.
Any ruleset history version after the pinned initial version invalidates trust,
even if a later version restores the original visible rules. Root rotation
remains predecessor-signed and history-preserving; the external head advances
to the replacement proof only after the replacement root and proof are
present. TEST_ONLY fixtures inject a deterministic API transport and may use
an isolated bare remote with the same policy, but they never provision or
publish production authority.
