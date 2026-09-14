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
| <a id="F-OPS-008"></a>`F-OPS-008` | `F-ARCH-016,F-ARCH-021,F-OD-17,F-OD-18,F-OD-19,F-OD-20` | Claim currentness continuously requires an unexpired authority, a live non-equivocating registry and protected external reviewer-bootstrap head, monitoring PASS, no blocking incident/compromise, and valid retention. Loss of any prerequisite prevents ACTIVE use. Reviewer currentness additionally requires a root-signed key-bound verifier authority, distinct root/verifier keys, `valid_from < valid_until`, and a REAL lifetime no greater than 604800 UTC seconds; renewal and subject/verifier/root changes use immutable successor proofs. | §§14–15 |

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
change, runtime argument, or external mutable file may make candidate pin
values authoritative or create REAL authority. The exact pinning commit is
authoritative only when `local reviewed pinning SHA = published main SHA = live
remote main SHA` under the existing safe-publication procedure. A
caller-selected local SHA, tracking ref, or checkpoint cannot authorize
freshness.

### 3.1.1 Canonical publication identity gate

Every production REAL resolution has an explicit purpose. For
`CURRENT_AUTHORIZATION`, the production-derived `published_normative_target`
resolver authenticates the fixed GitHub repository object
`provider=github`, `api_origin=https://api.github.com`,
`repository_id=1273879958`, and
`repository_full_name=XingyuW/rust_electroanalysis_cli`, then reads exactly
`refs/heads/main`. The selected normative target must equal that canonical
head exactly; ancestry is insufficient. A missing/malformed ref, unavailable
authentication/API, repository-identity mismatch, or unequal target fails
closed. There is no fallback to local `HEAD`, local `main`, `origin/main`, an
SSH alias, a remote URL, or a caller-declared publication SHA.

The resolver binds the repository identity, publication ref, published SHA,
selected SHA, and resolution purpose into the REAL runtime context. It reads
canonical main once at resolution start and again immediately before current
operational `GO`; the two heads and selected target must all be equal. A head
change during resolution fails closed.

`HISTORICAL_VALIDATION` is a separate non-authorizing purpose. Historical
publication validity is established only through canonical GitHub commit
existence plus canonical-main ancestry/equality. Local Git ancestry is
non-authoritative. The same registry-owned `published_normative_target`
resolver constructs the fixed repository API paths internally, using exact
40-hex target and main SHAs. It reads `/git/commits/{target}` and requires the
exact returned target SHA and a well-formed commit object. It then reads
`/compare/{target}...{main_start}` and requires both `base_commit.sha` and
`merge_base_commit.sha` to equal the target, `behind_by=0`, and canonical
nonnegative integer counts. Equality requires `status=identical` and zero
counts; a strict ancestor requires `status=ahead`, positive `ahead_by`, and
`total_commits=ahead_by`. Missing, malformed, contradictory, diverged, and
other-branch-only evidence fails closed. Commit existence alone is insufficient.

The resolver re-reads authenticated canonical main after comparison, and
historical validators re-read it immediately before returning validity. A
changed head fails closed without retry. The binding retains
`purpose=HISTORICAL_VALIDATION`; it cannot authorize current G3 even if a caller
changes the context purpose. Current authorization requires a fresh exact-main
proof. REAL entrypoints select `GitHubApiTransport`; response fixtures are
available only through private test entrypoints.

`refs/replace/*`, grafts (including `.git/info/grafts`), local refs,
remote-tracking refs, alternate object stores, `origin/main`, remote URLs, and
SSH aliases cannot establish historical publication. Remaining local Git
support reads disable replacement-object semantics. No local fetch, commit
parent, or ancestry command participates in the historical publication proof.

Historical graph validation recognizes the exact earlier R12 external catalog
as a historical representation, without executing its obsolete local-commit
resolver or changing the original graph bytes/hash. All other graph structural
checks still apply. Historical validation does not retroactively require later
provisioning state: a target's valid `UNPROVISIONED` binding remains meaningful.
`validate_historical_normative_target` validates the committed graph and draft
normative bundle, source hashes, and component bindings under that contract;
it reports current operational authority as false. Issued review artifacts
continue to require their historical signature, reviewer, root, and issuance
proof checks. Neither path advances currentness or the accepted-head checkpoint.

This permits published `A` to remain historically verifiable after main
advances `A -> B`, but never makes `A` current, returns operational `GO`, or
establishes G3 eligibility. The current graph catalog and current authority
lifecycle requirements remain strict.

Historical-publication traceability (part of `F-OPS-008`):

| Requirement | Production implementation | Production-path regression | Evidence |
| --- | --- | --- | --- |
| AC-HIST-007–030, 041–046 | `_resolve_publication_dependency`, `GitHubApiTransport.get_commit/compare_commits`, `_verify_historical_publication_lineage` | `tests/phase_f_historical_publication.py`: H1–H13, resolver/transport omissions | Canonical raw-response acceptance/rejection and local-manipulation invariance |
| AC-HIST-031–037 | `_historical_external_dependency_contract`, `validate_historical_normative_target`, `validate_historical_review_artifact` | H14–H16 and malformed graph/bundle cases | Published e5deb189 and 76561751 target-relative bundle checks; current authority false |
| AC-HIST-038–042, 053 | `validate_g3_tag`, purpose-bound `PublicationBinding` | H17–H18 and existing generator `--self-test` publication matrix | Exact-main current positive; unpublished/current and mode escalation reject |

The external graph remains 22 nodes and 27 edges: commit lookup, comparison,
and main re-read are mandatory operations within the existing atomic
publication resolver, with the same canonical repository prerequisite. They
are not independently selectable authority nodes. Removing either transport
operation or substituting a local resolver fails the publication gate.

### 3.1.2 Reviewer-bootstrap currentness and provisioning operations

The signed `PhaseFReviewerBootstrapCurrentnessProofV1` is the persisted
representation of the current verifier authority. No separate verifier
authority object or external DAG node is introduced. The verifier's sole
authority is to sign `PhaseFReviewerActorAttestationV1`; it cannot sign a
currentness proof, replace the root, advance the monotonic head, approve a
specification, become a reviewer merely by holding the verifier key, or create
G3 authority. Currentness proofs remain root-signed, root replacements remain
predecessor-root-signed, and actor attestations remain verifier-signed.

The verifier authority ID is derived, never selected by an operator. The
closed formula used by Wire, Operations, Conformance, graph metadata, and
production code is:

```text
PHASE_F_VERIFIER_ID_FORMULA_V1
domain = ASCII("mhi_phase_f_reviewer_bootstrap_verifier_authority_v1") || 0x00
preimage = JCS({"current_verifier_public_key":"<64 lowercase hex>","current_verifier_public_key_fingerprint":"<64 lowercase hex>"})
current_verifier_authority_id = "sha256:" || lowercase_hex(SHA256(domain || preimage))
sha256_operations = 1
```

The single SHA-256 result is lower-case hexadecimal and is prefixed with
`sha256:`. The hexadecimal digest is never hashed again.

The fingerprint is SHA-256 over the verifier's 32-byte public key. REAL
validation rejects reuse of either the root public key or root fingerprint.
Changing verifier key material necessarily changes the authority ID. A
rotation is only a root-signed immutable successor proof containing the new
ID/key/fingerprint triple; historical attestations remain cryptographically
verifiable under their issuance proof, while the old verifier cannot issue new
currently authorized attestations after monotonic-head advancement.

If a current verifier is suspected compromised or revoked, issuance under it
stops immediately. A replacement proof must be root-signed and externally
advanced before issuance resumes. Historical validity does not imply current
authorization. There is no automatic renewal, unsigned extension, or
in-place modification. A successor proof is required before expiry for
continuous availability, whenever the verifier, root, active subject set, or
any signed currentness state changes, and after verifier compromise/revocation
or subject removal/revocation.

REAL proof construction uses a maximum validity lifetime of exactly `604800`
UTC seconds. At a genesis or renewal ceremony, the operator obtains one
trustworthy UTC-second timestamp, freezes it as `valid_from`, sets
`valid_until = valid_from + 604800 seconds`, freezes all other fields, computes
the subject-registry head, currentness head ID, and proof ID, freezes the exact
signing bytes, and only then accesses the root private key. After root-key
access begins, only `signature` may change. Current validation requires
`valid_from < valid_until` and
`valid_from <= validation_time <= valid_until`; there is no grace period after
expiry, and expired or future authority fails closed. Historical validation
checks the encoded ordering and REAL lifetime limit without requiring today's
time to lie inside the old window.

Current authorization is target-relative and binds every reviewer attestation
to the exact root ID/complete-file hash, currentness-proof ID/complete-file
hash, and verifier authority ID selected by the live protected monotonic head.
An attestation issued under an earlier proof remains historically verifiable
under that proof, but it is not current authority after the head advances,
even when its subject is still ACTIVE or its signed `created_at` is backdated.
`created_at` is syntax-validated and must lie inside its issuance proof window;
it is not the verifier-rotation or current-authority cutoff.

Historical resolution propagates `HISTORICAL_VALIDATION` through every nested
authority context. The selected historical target owns the currentness-window
policy: legacy targets use their former
`valid_from <= validation_time <= valid_until` structural semantics without a
604800-second issuance ceiling, while the closed current profile requires
`valid_from < valid_until` and REAL encoded lifetime at most 604800 seconds.
Historical checks validate encoded timestamps and signatures without requiring
today's wall clock to be inside an expired historical window; historical
validity never creates current authorization.

The seven-day REAL lifetime bounds stale verifier, subject, and root exposure
without requiring daily root-key access or daily monotonic-head churn. Normal
renewal therefore requires at most approximately weekly root-signing access;
all payload construction and validation occurs before root-key access, which
is limited to the final signature operation. Renewal begins no later than 48
hours before `valid_until`, with a target monotonic-head completion no later
than 24 hours before expiry. Missing that objective does not extend authority.
At `validation_time > valid_until` validation fails closed with no grace period
and no unsigned emergency extension. If renewal fails before expiry, reviewer
bootstrap authority becomes unavailable until a valid root-signed successor is
published and the protected monotonic head advances normally. Compromise or
revocation follows the applicable rotation procedure; availability never
weakens the fail-closed rule.

The first REAL proof intended to bootstrap the first independent five-role
review requires at least five distinct natural-person subjects capable of the
canonical roles `scientific_metrology`, `architecture_data`, `security`,
`compatibility`, and `operations_governance`. Role names remain in later
reviewer attestation records, not in `subject_bindings`. More than five active
subjects is permitted. Enrollment as a bootstrap subject is not independent
reviewer status; the remediation author is excluded from the five reviewer
slots, so at least five reviewer-eligible natural persons distinct from that
author must be provisioned.

Each `subject_bindings` entry has the exact fields
`actor_subject_id`, `identity_evidence_sha256`, `subject_status`,
`equivalence_decision`, `equivalence_checked_subject_ids`, and
`equivalence_audit_sha256`. Subjects are sorted and unique, each current
status is `ACTIVE` or `INACTIVE`, and one retained evidence hash cannot map to
multiple admitted subjects, including one later marked `INACTIVE`. For each new
subject, the provisioning validator receives a subject-independent canonical
private evidence manifest and a separate canonical private equivalence-audit
manifest, derives the complete historical-plus-same-proof comparison set,
requires the exact `NO_NATURAL_PERSON_MATCH` decision, and recomputes both
digests. The root signature over these binding fields is the authoritative
subject-registry admission; a caller-declared result is not authority. The
evidence digest is the domain-separated SHA-256 of the evidence manifest, and
the audit digest uses
`mhi_phase_f_reviewer_person_equivalence_audit_v1\0`; neither private manifest
nor retained evidence is stored in public Git. The public proof contains only
opaque IDs, hashes, statuses, the decision, and checked opaque IDs, not
identity documents or PII. File-backed verifier secret storage
uses an operator-readable-only `0700` parent directory and `0600` private-key
file, remains outside Git/generated artifacts and `/tmp`, is never logged, and
is separate from root-key storage; keychain/HSM storage may provide an
equivalent or stronger control. This revision provisions no verifier key or
REAL subject.

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
