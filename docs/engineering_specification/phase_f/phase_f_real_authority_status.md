# Phase F REAL Authority Status

Status date: 2026-09-24  
Software release: v0.2.0  
Status type: non-authoritative operational snapshot

## Current decision

**Phase F REAL authority: NO-GO.**

This status is deliberately separate from software release readiness. v0.2.0 may be
used as research software, but its publication does not create Phase F REAL authority,
G3 approval, physical-validation approval, or reviewer-bootstrap authority.

## Current REAL bootstrap state

| Item | State |
|---|---|
| Required initial natural-person reviewers | 5 distinct eligible people |
| Structurally ready REAL reviewer packages | 0 / 5 |
| REAL reviewer verifier | NOT CREATED |
| REAL reviewer subjects | NONE |
| REAL identity/equivalence admissions | NONE |
| REAL currentness proof | NONE |
| Protected monotonic ref `refs/heads/phase-f-reviewer-bootstrap-head` | ABSENT |
| REAL G3 | NO-GO |
| REAL bootstrap-root private key used for reviewer currentness | NO |

The public bootstrap root may exist in the repository without making reviewer
authority operational. The protected monotonic ref is the live freshness authority and
has not been created.

## What has been validated

The TEST_ONLY reviewer-bootstrap path has been exercised end-to-end and independently
reviewed. Those results establish software/conformance behavior only.

TEST_ONLY material:

- cannot be counted as a REAL reviewer;
- cannot authorize REAL review artifacts;
- cannot replace the required natural-person identity/equivalence process;
- cannot create G3 authority; and
- must never be relabeled or promoted into REAL authority.

## Why REAL remains blocked

The current published governance contract requires five distinct natural persons for
the five canonical roles:

1. `scientific_metrology`
2. `architecture_data`
3. `security`
4. `compatibility`
5. `operations_governance`

The remediation author cannot occupy those five reviewer slots. Identity anti-alias
and private equivalence evidence remain mandatory.

A governance reassessment concluded that lower-person quorums may be technically
plausible for a future steady-state profile, but the current bootstrap has no
non-circular authority to adopt a lower quorum. Therefore the five-person requirement
remains in force.

## Future-work gate

Do not start the REAL bootstrap signing ceremony until all of the following are true:

1. five genuine, willing, qualified natural-person reviewers have been identified;
2. all five private intake/evidence packages are complete;
3. identity/equivalence and remediation-author independence checks are complete;
4. the existing private readiness checker reports exactly `5 / 5`;
5. a REAL verifier key is provisioned under the published storage policy;
6. all non-time genesis inputs are frozen and independently reviewed;
7. only then is `valid_from` frozen and the REAL bootstrap-root private key accessed
   for the dedicated genesis-proof signing ceremony;
8. monotonic genesis is published only after the signed proof and all external
   protection checks succeed.

No release tag, software version, operator action, TEST_ONLY result, or publication of
this status document waives those requirements.

## v0.2.0 use policy

v0.2.0 can be used for:

- exploratory and production-like research analysis;
- paper figures and reproducible analysis artifacts;
- method development and software validation.

Results should not be represented as having Phase F REAL governance approval or
physical-validation authority until the corresponding gates are completed.

## Preservation

Historical Phase F specifications, hashes, reviews, and TEST_ONLY artifacts retain
their original meaning. This snapshot does not amend the normative governance policy.
