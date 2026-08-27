# MHI V1 Phase F — R3 Physical Evidence and Production Validation Plan

## 1. Identity, status, and authority classes

This is the forward R3 remediation of R2 commit
`79e346e22f95f73d54b5dc55b2787de98d0948ea`, plan SHA-256
`08d7239ba8a88318ddfc0455f9aa8fbbee209915957d896b013bf720a87034b5`,
blob `146398c89c4acf5c78ba778c967d75a5c69410ce`. R1 was NO-GO/P1=13;
R2 was NO-GO/P1=10. R3 is forward planning remediation; independent rereview
is PENDING. Neither R2 nor R3 is approved.

Immutable Phase-E authority remains: integrated baseline
`14942a30928b88f16914bf0bb103cc0c2a5bfa76`, reviewed implementation
`5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb`, frozen plan SHA-256
`0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`,
and blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

Authority classes are disjoint: (1) **PRODUCTION RUNTIME AUTHORITY** is the
existing Rust reader, protocol, evaluator, runner, approval verifier and trust
store; (2) **EXTERNAL PHASE-F GOVERNANCE AUTHORITY** is the future checker,
bundle, tags, registry/head and evidence governance; (3) **SCIENTIFIC OWNER
DECISIONS** are only closed F-OD values; (4) **REAL PHYSICAL EVIDENCE** has
proved identity, custody, metrology and dependencies; (5) **TEST/KAT AUTHORITY**
proves software behavior only. No class substitutes for another. Synthetic,
constructed, unknown-origin, test or KAT material cannot support a physical
claim. This plan creates no schema, decision, key, signature, evidence, record,
trust, tag or branch.

## 2. Chronology and preserved closure

No implementation precedes independently approved plan and F0 tags:

```text
F_IMPL_1_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_2_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_3_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_4_BEFORE_F0_EXIT=FORBIDDEN
```

Order: R3 rereview/plan tag; F0 authoring/review/tag; F-IMPL-1 checker plus
F-MAINT-01 and F-MAINT-02; readiness review/tag; offline/HSM key generation;
frozen enrollment plus five fresh reviews; enrollment tag; genesis record; F1
preregistration; F2 evidence/cohort lock; F3 trust provisioning/tag; F4 exact
production execution; F5 release/state/final tag. F1–F5 are blocked until both
debts are CLOSED in F-IMPL-1. P2 temporary disposition is NONE. All twelve R1
counterexamples remain deterministic PASS. Existing production runner order is
preserved in §13. No implementation occurs before F0 GO/tag.

## 3. Canonical primitives and numeric authority

External JSON is UTF-8 JCS (RFC 8785), rejects duplicate/unknown members, and
separates semantic payload IDs from exact complete-file hashes. Sorted arrays
are strictly increasing by the stated key.

`RUNTIME_STABLE_ID_V1` is the exact runtime grammar: nonempty; first byte ASCII
alphanumeric; later bytes ASCII alphanumeric or `.`, `_`, `:`, `-`; no Unicode,
case folding, truncation, normalization or maximum length. Uppercase is valid
and preserved. `PHASE_F_EXTERNAL_ID_V1` is the same except digest IDs are exactly
`sha256:` plus 64 lowercase hex. External-only IDs never enter runtime fields
unless they also pass `RUNTIME_STABLE_ID_V1`.

`CanonicalUnsignedIntegerV1` is a JSON string `0|[1-9][0-9]*`;
`CanonicalPositiveIntegerV1` excludes zero. Runtime u64 fields must fit u64 and
project by exact base-10 parse. `CanonicalDecimalV1` is a JSON string matching
`-?(0|[1-9][0-9]*)(\.[0-9]*[1-9])?`; exponent, `+`, leading zero, trailing
fractional zero, `-0`, Unicode and JSON numbers are forbidden.

`RuntimeF64V1` contains exactly the field `decimal:CanonicalDecimalV1` and
`binary64_bits_hex` (16 lowercase hex). Bits equal IEEE-754 binary64 conversion
of the exact mathematical value represented by `CanonicalDecimalV1`, using
round-to-nearest, ties-to-even. The
checker independently recomputes conversion; the result is finite, not negative
zero, and within the field domain. The F1 TOML f64 must have identical bits. No
implicit units/defaults or alternate text-versus-bits authority exists.

F-OD-09 allocations interpret `CanonicalDecimalV1` as exact rational values and require
development + validation + holdout = exactly 1. Binary64 equality has no
authority. Numeric grammar, bit, range, fit or sum failure is NO-GO.

## 4. `PhaseFDecisionBundleV1` and closed owner surface

The bundle contains exactly `schema_version=1,decision_bundle_id,decisions`.
Decisions contain F-OD-01..20 once in ID order as `{decision_id,value,
decision_owner_role,rationale_document_sha256}`. `decision_owner_role` is one of
`SCIENTIFIC_METROLOGY_OWNER|ARCHITECTURE_DATA_OWNER|SECURITY_OWNER|
OPERATIONS_GOVERNANCE_OWNER`; assignments are exact: scientific/metrology owns
03,05,06,08,11,12; architecture/data owns 02,09,10; security owns 04,15,16;
operations/governance owns 01,07,13,14,17,18,19,20. `COMPATIBILITY_REVIEWER`
owns no scientific value and independently reviews all projections. All five
closed roles independently review the complete bundle. No null/default/extension exists.
Let `decision_payload` be all fields except `decision_bundle_id`:

```text
decision_bundle_id = "sha256:" + SHA-256(
  b"mhi_phase_f_decision_bundle_v1\0" || JCS(decision_payload))
```

The separator bytes are literal. Complete canonical file SHA-256 includes the
ID and remains separate; there is no identity cycle.

| ID | Exact owner value |
|---|---|
| F-OD-01 | `{protocol_id:RUNTIME_STABLE_ID_V1,title:canonical_nonempty_text}` |
| F-OD-02 | `{registration_id:RUNTIME_STABLE_ID_V1,immutable_reference_uri:canonical_uri,document_sha256:sha256}` |
| F-OD-03 | `DomainSelectorDecisionV1`: five categorical axes `{type:"allowed",ids:[RUNTIME_STABLE_ID_V1,...]}` sorted unique nonempty; temperature `{type:"bands",bands:[{lower_kelvin_inclusive:RuntimeF64V1,upper_kelvin_exclusive:RuntimeF64V1},...]}` positive, sorted, nonoverlapping, half-open |
| F-OD-04 | `{trust_root_id:RUNTIME_STABLE_ID_V1}` |
| F-OD-05 | sorted nonempty mechanism endpoints with owner fields `endpoint_id,hypothesis_id,domain,reference_rule,support_levels,minimum_eligible_records,minimum_independent_families,required_strata,acceptance_rules` |
| F-OD-06 | sorted nonempty health endpoints with owner fields `endpoint_id,target,domain,reference_rule,predicted_positive_statuses,predicted_negative_statuses,reference_label_universe,reference_positive_labels,reference_negative_labels,minimum_eligible_records,minimum_independent_families,required_strata,acceptance_rules` |
| F-OD-07 | sorted nonempty release claims `{claim_id,statement,domain,supporting_endpoint_ids}` |
| F-OD-08 | external scientific admissibility `{evidence_categories:[{category,may_support,may_contradict,claim_ceiling}]}`; cannot alter runtime policy |
| F-OD-09 | campaign split `{split_unit,allocations:{development,validation,holdout},stratification_keys,randomization_algorithm_id,seed_authority,split_execution_authority_id,lock_point:"before_outcome_access",post_hoc_movement:"forbidden"}` |
| F-OD-10 | physical identity `{unit_kinds,independent_kind_by_endpoint,identity_issuance_procedure_sha256,parent_child_rules,repeat_handling:"same_family_no_increment"}` |
| F-OD-11 | metrology eligibility with method/QC/calibration/LOD-LOQ document identities, exact units, checks and failure actions; §10 closes wire |
| F-OD-12 | `{power_analysis_id,power_method_id,power_method_version,power_method_document_sha256}`; §10 closes parameters |
| F-OD-13 | owner appointment `{authority_id,authority_role:"production_owner",authority_document:{immutable_uri,sha256}}` |
| F-OD-14 | registry appointment `{authority_id,authority_role:"production_registry",authority_document:{immutable_uri,sha256},registry_namespace_id,registry_head_resolver_uri}` |
| F-OD-15 | custody `{custody_method_id,custody_procedure_document,owner_custodian_role,registry_custodian_role,required_quorum,key_input_channel_id,network_mode:"offline"|"hsm_isolated",key_persistence_allowed:false,production_cli_access_allowed:false}` |
| F-OD-16 | rotation/revocation policy: closed trigger codes, immutable procedure hash, required state/revalidation, new approval/run; unsupported lifecycle blocks F3 |
| F-OD-17 | `{validity:{value:CanonicalPositiveIntegerV1,unit:"D"},periodic_review:{value:CanonicalPositiveIntegerV1,unit:"D"}}` |
| F-OD-18 | campaign-deviation total map `deviation_code -> exclude_affected|document_no_effect|campaign_no_go`; cannot overwrite runtime behavior |
| F-OD-19 | `PhaseFMonitoringPolicyV1` owner values in §12, including cadence and scientifically required thresholds |
| F-OD-20 | retention/access with positive integer day intervals, backup count, sorted roles, replacement authority, `unavailable_object_action:"no_go"` |

F-OD-05/06 `reference_rule` is the matching exact
`ReferenceAuthorityRuleV1` shape: sorted unique nonempty `allowed_methods`
`{method_id:RUNTIME_STABLE_ID_V1,method_version:canonical_nonempty_text}`, sorted
unique nonempty `allowed_authority_ids:RUNTIME_STABLE_ID_V1[]`,
invariant `blinding_rule:"require_blinded"`, and
`uncertainty_rule:{type:"require_quantified",measure_id,unit,
maximum_inclusive:RuntimeF64V1}`. Runtime authorizes the Cartesian product of
methods and authorities; the owner approves that exact space. Pairwise-only
policy is unrepresentable and blocks F1 pending reviewed runtime-schema plan.

Mechanism support levels are sorted unique nonempty members of
`hypothesized|experimentally_supported|validated_for_domain`.
`critical_policy:"any_contradicted_record_fails"`, `cohort_role:"holdout"`,
and `mechanism_artifact_required:true` are invariants. Health `target` is
`{type:"dimension",dimension_id}` or `{type:"aggregate"}`; status arrays form
an exact disjoint partition of `within_baseline,watch,degraded,critical` and
label arrays exactly partition their universe. `cohort_role:"holdout"` and
`health_artifact_required:true` are invariants.

Required strata contain `stratum_id,predicates,minimum_eligible_records,
minimum_independent_families`, with positive runtime u64 values and minima >=2
for physical endpoints. Predicate tags are exactly `analyte_equals,
matrix_equals,sensor_design_equals,sensor_equals,campaign_equals,
temperature_band`, in that order and each axis once; temperature fields use
`RuntimeF64V1`.

Acceptance rules mirror `AcceptanceRuleV1`. Count has `type:"count",rule_id,
metric,comparator,threshold_u64`; exact CountMetric token; comparator
`greater_than_or_equal|less_than_or_equal`; exact u64 string. Rate has
`type:"rate",rule_id,metric,target,comparator,threshold`; exact RateMetric and
RateTarget tokens; RuntimeF64 in [0,1]. Runtime metric-kind restrictions and
noncontradictory bounds hold. Balanced accuracy uses point_estimate only.
Mechanism requires support_fraction/point_estimate/>=; health requires coverage,
sensitivity and specificity />=.

## 5. F0 decision to runtime mapping and total projection

`PhaseFProtocolProjectionV1` is a future plan contract, not a file. It reads
approved F0 bytes, constructs TOML, parses through
`MhiValidationProtocolV1::from_toml`, and compares every field and f64 bit.

| F-OD | Decision field | Bound | Exact Rust field/type | Transformation / failure | F1 constructor |
|---|---|---|---|---|---|
| 01 | protocol_id,title | yes | `MhiValidationProtocolV1.protocol_id,title` | byte copy; grammar/equality else NO-GO | projection |
| 02 | registration | yes | `ProtocolRegistrationV1` | byte copy from immutable F1 registration; mismatch NO-GO | registration + projection |
| 03 | axes | yes | `DomainSelectorV1.target_domain` | only `CategoricalSelectorV1::Allowed`; byte copy IDs | projection |
| 03 | bands | yes | `TemperatureSelectorV1::Bands`/`TemperatureBandV1` | CanonicalDecimalV1-to-bits once; exact TOML bits; half-open | projection |
| 04 | trust_root_id | yes | `PhysicalApprovalAuthorityV1::EmbeddedTrustRoot` | byte copy; NotRequested forbidden | projection |
| 05 | all mechanism owner fields | yes | `Vec<MechanismEndpointV1>` | structural copy; exact tags/tokens/f64/u64/order | projection |
| invariant | mechanism role/artifact/critical | yes | respective fields | `Holdout,true,any_contradicted_record_fails` only | projection invariant |
| 06 | all health owner fields | yes | `Vec<HealthEndpointV1>` | structural copy; exact partitions/tags/numbers/order | projection |
| invariant | health role/artifact | yes | respective fields | `Holdout,true` only | projection invariant |
| 07 | claims | yes | `Vec<ReleaseClaimV1>` | structural copy; `requested_level=Physical` invariant | projection |
| invariant | schema/statistics | yes | `schema_version,statistics` | `1`; `wilson_95_v1,0.95,unavailable,indeterminate,and` | projection invariant |
| 08–20 | external governance/science | no | none | no runtime projection/override; attempt is NO-GO | external records/checker |

Nested aggregate rows cover every member exactly once. Domains obey subset and
supporting-endpoint semantic-equality constraints; every endpoint supports a
claim; all global IDs, canonical orders, minima and current validator rules
remain authoritative. Missing/extra/defaulted/transformed/unrepresentable value
is F0/F1 NO-GO. Thus `PROTOCOL_PROJECTION_UNBOUND_FIELDS=0`,
`DECISION_TO_RUNTIME_MAPPING_AMBIGUITIES=0`,
`UNREPRESENTABLE_DECISION_VALUES=0`, and
`HIDDEN_TRANSFORMATION_DEFAULTS=0`.

## 6. Frozen runtime failure behavior

`FROZEN_RUNTIME_FAILURE_BEHAVIOR_V1` is not configurable:

| Condition | Existing result |
|---|---|
| malformed/unknown/noncanonical input or hash/binding failure | hard error; no report |
| physical request with unprovisioned/unverified trust or approval | hard error before evaluation |
| ordered exclusion reason or known overlap | existing `Excluded` and exact reason |
| unknown separation or unavailable required rule | existing exclusion/`Indeterminate` semantics |
| endpoint rule false | `DoesNotMeetProtocol` |
| complete eligible evidence/all rules true | existing outcome; never claim activation by itself |

F-OD-18 governs campaign handling only and cannot reclassify runtime error,
exclusion, Indeterminate or DoesNotMeetProtocol.

## 7. Annotated tag grammar and six durable authorities

Every body is ASCII-subset UTF-8, LF only, exactly one final LF, no blank lines,
trailing whitespace, duplicates or optional fields; order is fixed and values
contain neither LF nor `=`. Scalars: `GIT_SHA` and `GIT_TAG_OBJECT` are 40
lowercase hex; `GIT_BLOB` and `GIT_TREE` are 40 lowercase hex Git object IDs of
the named kind; `SHA256` is 64 lowercase hex; `COUNT` is canonical unsigned
integer; `DECISION=GO|NO-GO`; `PASS_RESULT=PASS|FAIL`; `STABLE_ID` names either
ID grammar in §3; `TAG_TEXT` is nonempty printable ASCII bytes `0x20..0x7e`
excluding `=`, with neither leading nor trailing space. A tag reference equals
the literal expected tag name and the
checker peels it to the required target. Tags are annotated, immutable, normally
pushed and never moved.

Creators are procedural repository operators, not owner/registry cryptographic
authorities. `PHASE_F_RELEASE_COORDINATOR` creates plan, readiness, trust and
physical-release tags. `PHASE_F_GOVERNANCE_COORDINATOR` creates F0 and enrollment
tags.

| Exact tag | Creator / target / prerequisite | Literal body type and exact ordered fields |
|---|---|---|
| `ism-mechanism-health-v1-f-plan-approved` | release; reviewed R3 main SHA; five fresh GO, P0=0/P1=0 | `PhaseFPlanApprovalV1`; `format_version,plan_review_sha,plan_sha256,plan_git_blob,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-f0-decisions-approved` | governance; F0 review main SHA; plan tag peels; five GO | `PhaseFDecisionApprovalV1`; `format_version,phase_f_plan_tag,plan_review_sha,decision_review_sha,decision_bundle_id,decision_file_sha256,decision_git_blob,decision_count,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-readiness-approved` | release; integrated F-IMPL-1 main SHA after debts/reviews | exact block below |
| `ism-mechanism-health-v1-f-authority-enrollment-approved` | governance; readiness main SHA after exact bytes/five fresh reviews | exact block below |
| `ism-mechanism-health-v1-f-trust-provisioning-approved` | release; F3 integrated main SHA after enrollment/F2 lock | exact block below |
| `ism-mechanism-health-v1-f-physical-validation-released` | release; final F4/F5 integrated main SHA after five GO | exact block below |

For plan/F0 bodies `format_version=1`, `decision_count=20`, all decision fields
are GO, counts are zero, and final approval is GO. Exact full remaining bodies:

```text
PhaseFPlanApprovalV1
format_version=1
plan_review_sha=<GIT_SHA>
plan_sha256=<SHA256>
plan_git_blob=<GIT_BLOB>
scientific_decision=GO
architecture_decision=GO
security_decision=GO
compatibility_decision=GO
operations_decision=GO
p0_count=0
p1_count=0
approval_decision=GO
```

```text
PhaseFDecisionApprovalV1
format_version=1
phase_f_plan_tag=ism-mechanism-health-v1-f-plan-approved
plan_review_sha=<GIT_SHA>
decision_review_sha=<GIT_SHA>
decision_bundle_id=<PHASE_F_EXTERNAL_ID_V1>
decision_file_sha256=<SHA256>
decision_git_blob=<GIT_BLOB>
decision_count=20
scientific_decision=GO
architecture_decision=GO
security_decision=GO
compatibility_decision=GO
operations_decision=GO
p0_count=0
p1_count=0
approval_decision=GO
```

```text
PhaseFReadinessApprovalV1
format_version=1
phase_f_plan_tag=ism-mechanism-health-v1-f-plan-approved
f0_decisions_tag=ism-mechanism-health-v1-f-f0-decisions-approved
readiness_review_sha=<GIT_SHA>
checker_source_review_sha=<GIT_SHA>
checker_source_tree=<GIT_TREE>
checker_dependency_lock_sha256=<SHA256>
checker_binary_sha256=<SHA256>
macos_uname=<TAG_TEXT>
macos_arch=<TAG_TEXT>
macos_product_version=<TAG_TEXT>
macos_build_version=<TAG_TEXT>
rustc_version=<TAG_TEXT>
cargo_version=<TAG_TEXT>
build1=PASS
build2=PASS
reproducible_binary=PASS
f_maint_01=CLOSED
f_maint_02=CLOSED
scientific_decision=GO
architecture_decision=GO
security_decision=GO
compatibility_decision=GO
operations_decision=GO
p0_count=0
p1_count=0
approval_decision=GO
```

```text
PhaseFAuthorityEnrollmentApprovalV1
format_version=1
phase_f_plan_tag=ism-mechanism-health-v1-f-plan-approved
f0_decisions_tag=ism-mechanism-health-v1-f-f0-decisions-approved
readiness_tag=ism-mechanism-health-v1-f-readiness-approved
readiness_main_sha=<GIT_SHA>
enrollment_sha256=<SHA256>
owner_authority_id=<RUNTIME_STABLE_ID_V1>
registry_authority_id=<RUNTIME_STABLE_ID_V1>
owner_public_key_fingerprint=<SHA256>
registry_public_key_fingerprint=<SHA256>
scientific_decision=GO
architecture_decision=GO
security_decision=GO
compatibility_decision=GO
operations_decision=GO
p0_count=0
p1_count=0
approval_decision=GO
```

```text
PhaseFTrustProvisioningApprovalV1
format_version=1
phase_f_plan_tag=ism-mechanism-health-v1-f-plan-approved
f0_decisions_tag=ism-mechanism-health-v1-f-f0-decisions-approved
readiness_tag=ism-mechanism-health-v1-f-readiness-approved
authority_enrollment_tag=ism-mechanism-health-v1-f-authority-enrollment-approved
enrollment_sha256=<SHA256>
owner_public_key_fingerprint=<SHA256>
registry_public_key_fingerprint=<SHA256>
trust_root_id=<RUNTIME_STABLE_ID_V1>
trust_store_sha256=<SHA256>
trust_store_embedded_source_sha=<GIT_SHA>
trust_review_sha=<GIT_SHA>
f2_cohort_lock_registry_record_sha256=<SHA256>
macos_uname=<TAG_TEXT>
macos_arch=<TAG_TEXT>
macos_product_version=<TAG_TEXT>
macos_build_version=<TAG_TEXT>
macos_result=PASS
security_decision=GO
compatibility_decision=GO
p0_count=0
p1_count=0
approval_decision=GO
```

```text
PhaseFPhysicalReleaseApprovalV1
format_version=1
phase_f_plan_tag=ism-mechanism-health-v1-f-plan-approved
f0_decisions_tag=ism-mechanism-health-v1-f-f0-decisions-approved
readiness_tag=ism-mechanism-health-v1-f-readiness-approved
authority_enrollment_tag=ism-mechanism-health-v1-f-authority-enrollment-approved
trust_provisioning_tag=ism-mechanism-health-v1-f-trust-provisioning-approved
release_code_sha=<GIT_SHA>
protocol_sha256=<SHA256>
cohort_lock_registry_record_sha256=<SHA256>
owner_approval_file_sha256=<SHA256>
validation_manifest_sha256=<SHA256>
release_record_id=<PHASE_F_EXTERNAL_ID_V1>
release_file_sha256=<SHA256>
release_registry_record_sha256=<SHA256>
initial_claim_state_registry_record_sha256=<SHA256>
scientific_decision=GO
architecture_decision=GO
security_decision=GO
compatibility_decision=GO
operations_decision=GO
p0_count=0
p1_count=0
macos_result=PASS
release_decision=GO
```

Exactly six durable Phase-F tags exist; amendment review is required to change
them. No tag is created by this plan revision.

## 8. Checker source-to-binary authority

F-IMPL-1 may create `tools/phase_f_authority_checker`; this plan does not.
`checker_source_review_sha` is its exact reviewed source commit;
`checker_source_tree` is that directory's Git tree ID;
`checker_dependency_lock_sha256` hashes its exact Cargo.lock or approved shared
lock. Exact build command:

```text
cargo build --locked --release --manifest-path tools/phase_f_authority_checker/Cargo.toml
```

Readiness freezes macOS architecture, uname, product/build versions, rustc and
cargo. Two independent clean builds from the source SHA must have identical
executable SHA-256. Each real F1–F5 invocation first matches the readiness
binary hash and records it, readiness tag, argv, input hashes, transcript hash
and exit status. Correct source with a different executable is NO-GO.

Commands are `phase-f-authority-check decisions|enrollment|registry|package|
power|metrology|cohort-lock|approval-package|execution|release|claim-status`.
The read-only checker does not score, sign, replace runtime or create evidence.
KAT and campaign use identical reviewed parsers; no human parser fallback.

## 9. Enrollment, registry subjects and current head

`PhaseFAuthorityEnrollmentV1` contains exactly `schema_version=1,enrollment_id,
owner_authority_id,registry_authority_id,owner_public_key_ed25519_hex,
registry_public_key_ed25519_hex,owner_public_key_fingerprint,
registry_public_key_fingerprint,owner_authority_document_sha256,
registry_authority_document_sha256,key_generation_attestation_sha256,
key_custody_attestation_sha256,f0_decision_bundle_id,f0_decision_file_sha256`.
After readiness, keys are generated offline/HSM under F-OD-15; exact bytes are
frozen and five fresh reviews occur before enrollment tag and genesis. Genesis
key trust requires F0 ID match, tag byte hash, fingerprint match and correctly
peeled readiness/enrollment tags. Genesis never self-authenticates.

`PhaseFRegistryRecordV1` contains exactly `schema_version=1,
registry_namespace_id,sequence_number,record_kind,predecessor_record_sha256,
subject_id,subject_sha256,related_record_sha256s,registry_authority_id,
registry_public_key_fingerprint,registry_signature_ed25519_hex`. Genesis is
sequence 0/null predecessor; later sequence is predecessor+1 and hashes exact
predecessor bytes. Signature preimage is
`b"mhi_phase_f_registry_record_v1\0" + JCS(excluding signature)`.

| Kind / order | Subject ID; hash | Required related hashes | Forbidden related hashes |
|---|---|---|---|
| `authority_enrolled` genesis after enrollment tag | enrollment_id; exact enrollment bytes | F0 decision file; SHA-256 of exact readiness tag body bytes | future/self |
| `protocol_registered` after power/registration | protocol_id; exact TOML bytes | power record; registration document; enrollment record | cohort/execution/release/state |
| `cohort_locked` after all F2 inputs | cohort-lock ID; canonical lock bytes | protocol, enrollment, package and every lock component below | approval/execution/release/state |
| `approval_registered` after lock | approval_record_id; exact approval file | cohort-lock registry record | execution/release/state |
| `execution_registered` after run | execution_id; canonical execution bytes | cohort-lock and approval registry records | release/state |
| `release_registered` after semantic payload | release_record_id; release semantic digest | execution and approval records | final release file/state/self |
| `claim_state_changed` after semantic payload | state record ID; state semantic digest | release record and prior state record if noninitial | final state file/self |
| `supersession_registered` after replacement | supersession ID; digest of canonical old/new/reason payload | old/new release records and latest old state | unrelated cohort/execution/self |

Runtime object IDs use runtime grammar; digest subjects use external grammar.
Missing/extra/forbidden relationships, order error, fork, gap, rollback,
signature or unavailable predecessor is NO-GO.

`PhaseFCohortLockRecordV1` contains exactly `schema_version=1,cohort_lock_id,
protocol_sha256,dataset_file_sha256,cohort_semantic_sha256,
package_manifest_sha256,dependency_audit_sha256,physical_unit_ledger_sha256,
physical_unit_identity_audit_sha256,power_analysis_sha256,
metrology_package_sha256,chain_of_custody_sha256,deviation_ledger_sha256,
decision_bundle_file_sha256,code_baseline_sha,limitations_document_sha256`.

`PhaseFExecutionRecordV1` contains exactly `schema_version=1,execution_id,
code_git_sha,binary_sha256,protocol_sha256,dataset_sha256,approval_file_sha256,
trust_store_sha256,trust_root_id,checker_source_review_sha,
checker_binary_sha256,readiness_tag,checker_argv,checker_input_sha256s,
checker_transcript_sha256,macos_uname,macos_arch,macos_product_version,
macos_build_version,validation_manifest_sha256`.

`PhaseFRegistryHeadV1` contains exactly `schema_version=1,
registry_namespace_id,head_sequence_number,head_record_sha256,
registry_authority_id,registry_public_key_fingerprint,head_signature_ed25519_hex`.
Signature preimage is `b"mhi_phase_f_registry_head_v1\0" + JCS(excluding
signature)`. Claim-status retrieves F-OD-14 resolver, verifies head and complete
chain, and never accepts sequence regression. Unavailable/unverifiable resolver
means NOT ACTIVE. Cached old heads cannot establish public ACTIVE. The production
CLI never uses this resolver; the external checker does.

## 10. Physical identity, custody, power and reference results

`PhaseFPackageManifestV1` contains exactly `schema_version=1,manifest_id,
protocol_sha256,cohort_semantic_sha256,entries`. Entries sort by `logical_id`
and contain `logical_id,role,immutable_uri,sha256,byte_length,media_type,
format_or_schema,producing_authority_id,physical,test_only,generated,
direct_dependency_ids,physical_unit_ids,retention_class_id`. Role is exactly
`raw_data|experiment_metadata|assessed_artifact|reference_artifact|
preprocessing_output|calibration_input|fit_output|model_derived_reference|
method_document|calibration_document|qc_document|chain_of_custody|
deviation_ledger|physical_unit_ledger|identity_audit|power_analysis|protocol|
dataset|lineage_catalog|approval|registry|release|claim_state|monitoring|
incident`. IDs resolve; arrays sort unique; byte/path/native identities cannot
conflict. `PhaseFDependencyAuditV1` contains exactly `schema_version=1,
manifest_sha256,auditor_authority_id,checked_logical_ids,
missing_dependency_findings,undeclared_real_world_dependency_findings,
cycle_findings,audit_result`; result is `pass|no_go`. Any omitted or unproved
real-world dependency is UNKNOWN SEPARATION and NO-GO.

`PhaseFPhysicalUnitLedgerV1` contains `schema_version=1,unit_ledger_id,entries`.
Each entry contains exactly `unit_id,unit_kind,identity_issuer_authority_id,
native_identifier,identity_basis,identity_evidence,parent_unit_ids,
derived_from_unit_ids,acquisition_family_id`. Basis is
`manufacturer_serial|fabrication_batch_record|sample_collection_identifier|
instrument_run_identifier|campaign_identifier|reference_measurement_identifier|
other_registered_identity_basis`; the last requires immutable method/identity
document SHA-256 and has no free-text semantics.

`PhaseFPhysicalIdentityAuditV1` contains exactly `schema_version=1,
unit_ledger_sha256,manifest_sha256,auditor_authority_id,
identity_issuance_procedure_sha256,checked_unit_ids,
duplicate_or_alias_findings,unresolved_identity_findings,audit_result`, where
result is `pass|no_go`. An independent data/governance reviewer is required.
Duplicate native issuer identity, conflicting evidence, or suspected but
unresolved sameness is NO-GO/UNKNOWN PHYSICAL IDENTITY. Different IDs never
prove different real units. Cohort lock binds ledger and audit.

`PhaseFChainOfCustodyV1` contains `schema_version,custody_ledger_id,
package_manifest_sha256,physical_unit_ledger_sha256,events`. Each event contains
`event_id,unit_ids,event_type,actor_authority_id,source_location_id,
destination_location_id,input_object_sha256s,output_object_sha256s,
procedure_document_sha256,deviation_ids`; type is exactly `acquired|transferred|
aliquoted|processed|measured|stored|released_to_analysis|destroyed`.

`PhaseFDeviationLedgerV1` contains `schema_version,deviation_ledger_id,
campaign_id,entries`. Entry fields are `deviation_id,affected_unit_ids,
affected_object_sha256s,deviation_code,detected_stage,required_action,
decision_authority_id,rationale_document_sha256,status`; status is
`open|resolved_excluded|resolved_no_effect|campaign_no_go`. Required action must
match F-OD-18. Each registered revision is immutable; update is append or
supersession only. Undocumented deviation is NO-GO. Cohort lock binds custody
and deviation hashes.

`PhaseFParameterValueV1` is a closed tagged union `integer|decimal|runtime_f64|
boolean|categorical|quantity`; quantity contains `value,unit`. Values use §3
primitives. `PhaseFPowerAnalysisRecordV1` contains exactly `schema_version=1,
analysis_id,protocol_id,endpoint_id,primary_metric_id,power_method_id,
power_method_version,power_method_document_sha256,null_hypothesis,
alternative_hypothesis,effect_size_definition,effect_size_value,
type_i_error_criterion,power_target,cluster_unit_kind,intra_cluster_assumption,
correlation_or_icc_model,positive_class_prevalence_assumption,
negative_class_prevalence_assumption,missingness_assumption,
attrition_assumption,required_strata,minimum_eligible_records,
minimum_independent_families,minimum_positive_records,minimum_negative_records,
rounding_rule,parameters,sensitivity_cases,software_name,software_version,
software_source_sha256_or_release_id,stochastic_algorithm,random_seed,
analyst_authority_id,reviewer_authority_id,limitations,
supporting_document_sha256s`. Parameters are sorted `{parameter_id,value}`, never a map;
cases are sorted `{case_id,parameter_overrides}` using the same type. Immutable
method document defines semantics, required IDs, units, algorithm/formula,
cluster/correlation/ICC model, rounding and software procedure. Checker requires
all and only declared parameters and complete cases; no default.

`PhaseFReferenceResultV1` contains exactly `schema_version,
reference_result_id,reference_source_id,reference_endpoint_id,endpoint_id,
reference_type,physical_unit_ids,method_id,method_version,authority_id,
measurand_id,result_value,result_unit,blinding_state,uncertainty,lod_loq_status,
calibration_status,qc_status,method_document_sha256,
calibration_document_sha256,qc_document_sha256,traceability_document_sha256,
chain_of_custody_sha256,limitations_document_sha256,limitations,
reference_outcome`. Type is `mechanism|health`. Mechanism outcome is exactly
`supports|contradicts|not_assessed|unavailable`; health outcome is an approved
label. `result_value` is `PhaseFParameterValueV1`; `result_unit` is an exact
unit token; blinding state is `blinded_to_assessment|not_blinded|unknown`;
uncertainty is exactly `{measure_id,value:RuntimeF64V1,unit}`; each LOD/LOQ,
calibration and QC status is `pass|fail|not_applicable`; arrays are sorted
unique and no status or limitation supplies a runtime default.

Projection to runtime `ReferenceEndpointV1` copies endpoint/reference source
IDs, hypothesis/target, outcome/label, method ID/version, authority ID, blinding,
uncertainty and limitations without aliases/conversion. Projection to
`ReferenceSourceAuthorityV1` sets `evidence_origin=physical` and copies exact
dependency completeness, experiment scope, acquisition families and direct
dependencies. External QC/calibration/LOD eligibility decides preregistered
admission to the locked dataset only; it invents no runtime behavior. Failed
eligibility is F2 NO-GO or omission only under F-OD-11.

## 11. Release, claim state and latest-state semantics

`PhaseFReleaseRecordV1` contains exactly `schema_version=1,release_record_id,
claim_id,claim_outcome,claim_wording,target_domain,supporting_endpoint_ids,
protocol_sha256,cohort_semantic_sha256,package_manifest_sha256,
cohort_lock_registry_record_sha256,reference_method_bindings,code_git_sha,
binary_sha256,platform,trust_store_sha256,trust_root_id,
owner_approval_record_id,owner_approval_file_sha256,validation_report_id,
validation_manifest_sha256,limitations,validity_duration_value,
validity_duration_unit,periodic_review_value,periodic_review_unit,
registry_record_sha256`. Domains and endpoint IDs are exact F0 projection;
method bindings sort by endpoint/reference and contain `endpoint_id,
reference_result_id,method_id,method_version,authority_id,
method_document_sha256`. Platform is `macos`; duration units are `D`; claim
outcome is `physically_validated|software_validated_only|
does_not_meet_protocol|indeterminate`. Define
`release_semantic_payload` as every semantic field excluding
`release_record_id` and `registry_record_sha256`:

```text
release_semantic_sha256 = SHA-256(
  b"mhi_phase_f_release_record_v1\0" || JCS(release_semantic_payload))
release_record_id = "sha256:" + release_semantic_sha256
```

The `release_registered` subject is that ID/digest, intentionally not the final
file hash. After registry creation, insert `registry_record_sha256` and compute
`release_file_sha256` over exact canonical final file bytes. The final tag binds
all three: ID, file hash and registry-record hash. No cycle exists.

`claim_state_payload` contains `schema_version,claim_id,release_record_id,
previous_claim_state_record_id,state,reason_code,effective_at,
superseding_release_record_id,reinstatement_evidence_sha256s,limitations`, and
excludes `claim_state_record_id,registry_record_sha256`. `effective_at` is
RFC3339 UTC second precision `YYYY-MM-DDTHH:MM:SSZ`.

```text
claim_state_semantic_sha256 = SHA-256(
  b"mhi_phase_f_claim_state_v1\0" || JCS(claim_state_payload))
claim_state_record_id = "sha256:" + claim_state_semantic_sha256
```

A signed `claim_state_changed` registry record binds ID/digest; its hash is then
inserted into the final state file. There is no second state signature or cycle.

Legal transitions are only NONE→active; active→suspended|withdrawn|expired|
superseded; suspended→active only with exact approved reinstatement/revalidation
evidence; suspended→withdrawn|expired|superseded. Withdrawn and superseded are
terminal. Expired is terminal; a supersession may cite history but cannot
reactivate it. Reason compatibility is exact: `initial_release→active`,
`key_compromise|key_revocation|monitoring_breach|reference_qc_breach|
domain_breach→suspended|withdrawn`, `periodic_expiry→expired`,
`superseded_by_new_release→superseded`, `manual_withdrawal→withdrawn`, and
`approved_reinstatement→active` from suspended only with evidence. Invalid
transition/reason is NO-GO.

Every public/citation check retrieves and verifies the current registry head and
complete chain, finds latest state, verifies release/final-tag binding, checks
monitoring currency and expiry, and returns only `ACTIVE|NOT_ACTIVE|
AUTHORITY_UNAVAILABLE`. Authority unavailable behaves as NOT ACTIVE for public
physical use. Historical release tags never override later state.

## 12. Monitoring authority

F-OD-19 is `PhaseFMonitoringPolicyV1`, containing exactly `schema_version=1,
monitoring_policy_id,monitoring_interval_value:CanonicalPositiveIntegerV1,
monitoring_interval_unit:"D",required_metrics,metric_thresholds,
missing_monitoring_action:"suspend",domain_breach_action:"suspend",
reference_qc_breach_action:"suspend"`. Required metrics are exactly
`domain_compliance,reference_qc_status,calibration_status,sensor_drift,
invalid_input_rate,indeterminate_rate,data_quality_insufficient_rate,
exclusion_rate,reference_uncertainty_status,software_git_sha,binary_sha256,
trust_store_sha256,trust_root_id,owner_approval_record_id,release_record_id`.
`metric_thresholds` is a sorted array `{metric_id,comparator,value,unit}` only
for scientifically required thresholds supplied by the owner; no threshold is
invented. Document/status/hash metrics have no numeric threshold.

`PhaseFMonitoringRecordV1` contains exactly `schema_version=1,
monitoring_record_id,release_record_id,claim_id,window_start,window_end,
policy_sha256,measurements,breaches,result`; timestamps are RFC3339 UTC second
precision with `Z`; result is `pass|suspend`. Measurements contain every
required metric once as sorted `{metric_id,value:PhaseFParameterValueV1,unit}`;
breaches are sorted `{metric_id,breach_code,evidence_sha256}` and reference metric IDs
and immutable evidence hashes. The checker validates structure/cadence; an
operations reviewer judges real evidence. Missing due monitoring makes the
claim NOT ACTIVE and requires suspension under policy.

## 13. Exact production order and implementation scopes

The unchanged production runner order is:

1. validate option path relationships;
2. strict-read, UTF-8 parse, validate and hash protocol bytes;
3. determine whether any claim requests Physical;
4. for Physical load embedded trust; if UNPROVISIONED fail before dataset open;
5. `ValidationInputs::read` strictly reads/validates dataset, lineage, Phase-B/C
   sources, references, protocol/data bindings and source authority;
6. for Physical locate approval at the pinned dataset-directory authority,
   strict-read, verify file hash, trust/root/owner/registry/protocol/cohort/claim/
   endpoint/reference/domain bindings, both signatures and expected record ID,
   then attach opaque `VerifiedOwnerApproval`;
7. evaluate, authorize publication and atomically publish.

Approval verification precedes scientific scoring, not scientific-source read;
campaign chronology is separate. F-IMPL-1 scope remains checker, Phase-F KATs,
F-MAINT-01 output-only closure, F-MAINT-02 permanent reproduction coverage and
direct docs. It cannot change scientific/evaluator/CLI/trust/signer logic.
F-IMPL-3 begins only after plan/F0/readiness/enrollment tags, F1 GO, F2 lock and
debt closure, and may embed reviewed public keys only; never private keys.

## 14. Requirements, primary ACs, tests and evidence

Each row is one normative requirement with exactly one primary AC. Test IDs and
evidence IDs are catalogued below; ranges mean every member.

| Requirement | Normative requirement | Primary AC | Test(s) | Evidence | ODs |
|---|---|---|---|---|---|
| F-R01 | Enforce chronology and pre-F0 prohibition. | F-AC01 stage checker accepts only §2 order. | F-T01 | F-EV01 | 01–20 |
| F-R02 | Use exactly six tags with §7 grammar/roles/peeling. | F-AC02 every body/target/creator mutation rejects. | F-T02 | F-EV01 | 13–20 |
| F-R03 | Bundle semantic ID and file identity are exact/acyclic. | F-AC03 mutation changes/rejects correct authority. | F-T03 | F-EV02 | 01–20 |
| F-R04 | Runtime IDs/numbers use §3 exact grammars. | F-AC04 case/bits/range/rational KATs are exact. | F-T04 | F-EV02 | 01–20 |
| F-R05 | Projection is total and isomorphic. | F-AC05 all TOML fields/bits equal or F1 forbidden. | F-T05 | F-EV03 | 01–07 |
| F-R06 | Frozen runtime behavior cannot be overridden. | F-AC06 each attempted external override rejects. | F-T06 | F-EV03 | 18 |
| F-DATA-01 | Manifest binds every actual byte/dependency. | F-AC07 omission/hash/path alias is NO-GO. | F-T07 | F-EV10 | 09–11,20 |
| F-DATA-02 | Physical identity ledger/audit closes independence. | F-AC08 duplicates/conflicts/unknown sameness are NO-GO. | F-T08 | F-EV08 | 10 |
| F-DATA-03 | Custody events are closed and hash-bound. | F-AC09 missing/unknown event or unit rejects. | F-T09 | F-EV09 | 10–11,18 |
| F-DATA-04 | Deviation revisions are immutable and complete. | F-AC10 undocumented/edit-in-place deviation is NO-GO. | F-T10 | F-EV09 | 18 |
| F-DATA-05 | Power method/parameters are complete. | F-AC11 missing/unknown/wrong-unit parameter rejects. | F-T11 | F-EV07 | 12 |
| F-DATA-06 | Reference results project exactly after eligibility. | F-AC12 each metrology/projection mutation rejects. | F-T12 | F-EV06 | 05–06,11 |
| F-HOLD-01 | Split is prospective and exact rational. | F-AC13 bad sum/seed/lock/movement rejects. | F-T13 | F-EV07 | 09 |
| F-HOLD-02 | Unproved independence never increments counts. | F-AC14 alias/unknown ancestry prevents F2 lock. | F-T08 | F-EV08 | 10 |
| F-TRUST-01 | F0 appointments plus enrollment tag bootstrap keys. | F-AC15 key/hash/fingerprint/review substitution rejects. | F-T14 | F-EV04 | 13–15 |
| F-TRUST-02 | Registry subjects/relations/order are per-kind exact. | F-AC16 kind mutation/fork/gap/rollback rejects. | F-T15 | F-EV11 | 14–16 |
| F-TRUST-03 | Current head is signed, complete and nonregressing. | F-AC17 unavailable/stale/regressed head is not ACTIVE. | F-T16 | F-EV12 | 14,16–19 |
| F-TRUST-04 | Trust provisioning binds enrollment/public keys only. | F-AC18 exact tag/store/source hashes pass; private path zero. | F-T17 | F-EV13 | 04,13–16 |
| F-SEC-01 | Checker binary is reproducibly source-bound. | F-AC19 two builds and execution hash are identical. | F-T18 | F-EV05 | 15 |
| F-SEC-02 | Real checker is read-only and same parser as KAT. | F-AC20 command/transcript/parser provenance is exact. | F-T19 | F-EV05 | 01–20 |
| F-SEC-03 | Production order remains §13. | F-AC21 source guard proves exact route/order. | F-T20 | F-EV14 | 04,13–15 |
| F-OPS-01 | Release semantic/file/registry identities are acyclic. | F-AC22 all three identities validate independently. | F-T21 | F-EV15 | 01–20 |
| F-OPS-02 | Claim states use only legal transitions/reasons. | F-AC23 invalid transition/reason rejects. | F-T22 | F-EV16 | 16–19 |
| F-OPS-03 | Latest state overrides historical tags. | F-AC24 suspended/expired/withdrawn/superseded is not ACTIVE. | F-T23 | F-EV16 | 16–19 |
| F-OPS-04 | Monitoring metrics/cadence are complete. | F-AC25 missing metric/cadence/breach causes suspension. | F-T24 | F-EV17 | 17,19 |
| F-OPS-05 | Retention/retrieval fails closed. | F-AC26 missing/hash/parser/retention failure is NO-GO. | F-T25 | F-EV18 | 20 |
| F-OPS-06 | F5 needs five GO, zero P0/P1, macOS PASS and final tag. | F-AC27 omitted prerequisite prevents ACTIVE/tag. | F-T26 | F-EV19 | 01–20 |
| F-COMPAT-01 | Runtime scientific schemas/logic remain unchanged. | F-AC28 prohibited diff count is zero. | F-T20 | F-EV14 | 01–12 |
| F-COMPAT-02 | Phase E 38/38 and Phase D 73/73 remain exact. | F-AC29 exact-SHA suites pass. | F-T27 | F-EV14 | — |
| F-COMPAT-03 | F-MAINT-01 closes in F-IMPL-1 without output change. | F-AC30 goldens pass before readiness tag. | F-T28 | F-EV20 | — |
| F-COMPAT-04 | F-MAINT-02 closes with permanent 14/14 coverage. | F-AC31 inventory/reproduction passes before readiness. | F-T29 | F-EV20 | — |
| F-R07 | Four claim outcomes and physical evidence ceiling persist. | F-AC32 prohibited origins never produce physical support. | F-T30 | F-EV14 | 07–08 |
| F-R08 | Every external object has deterministic retrieval. | F-AC33 unavailable/hash/length/parse mismatch is NO-GO. | F-T25 | F-EV18 | 02,11–20 |
| F-R09 | Human review starts only after checker PASS. | F-AC34 no alternate parser or identity fallback exists. | F-T19 | F-EV05 | 01–20 |
| F-R10 | All old/new adversarial cases have exact results. | F-AC35 §15 table passes without interpretation. | F-T01–T30 | F-EV01–EV20 | 01–20 |

Test catalog: F-T01 chronology; T02 tag grammar/targets/roles; T03 bundle ID;
T04 ID/numeric; T05 protocol projection; T06 frozen failures; T07 package;
T08 identity; T09 custody; T10 deviation; T11 power; T12 metrology/reference;
T13 split; T14 enrollment; T15 registry; T16 head; T17 trust; T18 reproducible
checker; T19 real command/parser; T20 production/source guards; T21 release;
T22 transition; T23 latest state; T24 monitoring; T25 retrieval; T26 F5; T27
Phase-E/D; T28 maintenance-01; T29 maintenance-02; T30 claim ceiling.

Evidence catalog: F-EV01 plan/tag/stage; EV02 decisions; EV03 projected protocol;
EV04 enrollment; EV05 checker builds/transcripts; EV06 metrology/reference; EV07
power/split; EV08 unit ledger/audit; EV09 custody/deviation; EV10 package/lock;
EV11 registry; EV12 current head; EV13 trust; EV14 compatibility/execution; EV15
release; EV16 claim state; EV17 monitoring; EV18 retention/retrieval; EV19 five
F5 reviews/final tag; EV20 maintenance closure.

External schema mapping: decision/numeric/projection→F-R03–05; tag→F-R02;
enrollment/registry/head/cohort/execution→F-TRUST-01–03; physical ledger/audit→
F-DATA-02; custody/deviation→F-DATA-03–04; parameter/power→F-DATA-05;
reference result→F-DATA-06; release/state→F-OPS-01–03; monitoring→F-OPS-04.
All six tags map to F-R02 and their stage-specific requirements.

Catalog closure: requirements=35; acceptance criteria=35; tests=30; evidence
items=20; owner decisions=20. `UNMAPPED_REQUIREMENTS=0`, `UNMAPPED_ACS=0`,
`UNMAPPED_TESTS=0`, `UNMAPPED_EVIDENCE=0`, `UNMAPPED_ODS=0`,
`ORPHAN_FIXTURES=0`, `LOST_R1_NORMATIVE_OBLIGATIONS=0`.

## 15. Adversarial authority and counterexample closure

All currently passing R1 boundary, trust, origin, signature, lineage, minima,
uncertainty, deterministic-output, publication and outcome cases keep passing.
R2 failed cases are exact: R2-CX-01 mutated bundle ID→semantic mismatch/NO-GO;
02 substituted key→enrollment tag hash/fingerprint mismatch/NO-GO; 03 different
review enrollment hash→tag mismatch/NO-GO; 09 undeclared power parameter→method
document mismatch/NO-GO; 14 unprojectable decision→projection FAIL/F1 forbidden;
15 exact rational allocation sum=1→PASS regardless of binary64 sum.

| Case | Input | Deterministic result |
|---|---|---|
| R3-CX-01 | uppercase runtime-valid ID | PASS; uppercase preserved byte-for-byte |
| R3-CX-02 | A only/X and B only/Y pairwise policy | Cartesian runtime cannot express it; F1 NO-GO unless owner approves full Cartesian set |
| R3-CX-03 | allow-unblinded physical reference | NO-GO; RequireBlinded invariant |
| R3-CX-04 | alternate mechanism critical policy | NO-GO |
| R3-CX-05 | two predicates on same axis | NO-GO |
| R3-CX-06 | rate rule missing RateTarget | NO-GO |
| R3-CX-07 | RuntimeF64 text/bits disagree | NO-GO |
| R3-CX-08 | source SHA correct, binary hash different | NO-GO |
| R3-CX-09 | valid enrollment tag, unavailable enrollment file | NO-GO |
| R3-CX-10 | valid chain, resolver unavailable | AUTHORITY_UNAVAILABLE; public claim NOT ACTIVE |
| R3-CX-11 | historical physical release, latest suspended | NOT ACTIVE |
| R3-CX-12 | missed monitoring interval | NOT ACTIVE and suspension required |
| R3-CX-13 | undocumented deviation after lock | NO-GO/suspension by stage |
| R3-CX-14 | two sample IDs, distinctness unproved | UNKNOWN PHYSICAL IDENTITY; no independent count; F2 NO-GO |
| R3-CX-15 | QC passes but result cannot project | F2 NO-GO |
| R3-CX-16 | valid release semantic digest, wrong final registry pointer | NO-GO |
| R3-CX-17 | withdrawn→active | invalid transition/NO-GO |
| R3-CX-18 | external-valid but runtime-invalid ID enters protocol | F0/F1 NO-GO |
| R3-CX-19 | unsupported temperature boundary semantics | F0 NO-GO |
| R3-CX-20 | runtime-required health partition omitted | projection FAIL/F1 forbidden |

## 16. R3 remediation ledger

Author disposition is not independent closure; only `REMEDIATED|OPEN` is used.

| ID | R2 P1 finding | R3 authoritative section / remediation | Requirement | AC | Test/KAT | F-EV | Author disposition |
|---|---|---|---|---|---|---|---|
| F-PLAN-R3-P1-01 | bundle/numeric identity incomplete | §§3–4 exact primitives, semantic/file identity | F-R03–04 | AC03–04 | T03–04 | EV02 | REMEDIATED |
| F-PLAN-R3-P1-02 | tag grammar/roles/enrollment authority incomplete | §7 scalar grammar, peeling, roles, six bodies | F-R02 | AC02 | T02 | EV01 | REMEDIATED |
| F-PLAN-R3-P1-03 | unrepresentable F-OD/runtime mappings | §§4–6 exact shapes, mapping, projection/invariants | F-R05–06 | AC05–06 | T05–06 | EV03 | REMEDIATED |
| F-PLAN-R3-P1-04 | enrollment not immutably byte-bound | §§7,9 enrollment tag before genesis | F-TRUST-01 | AC15 | T14 | EV04 | REMEDIATED |
| F-PLAN-R3-P1-05 | registry kind/genesis semantics incomplete | §9 per-kind subjects/relations/order/head | F-TRUST-02–03 | AC16–17 | T15–16 | EV11–12 | REMEDIATED |
| F-PLAN-R3-P1-06 | checker source/binary provenance incomplete | §8 tree/lock/build/platform/two builds/hash | F-SEC-01–02 | AC19–20 | T18–19 | EV05 | REMEDIATED |
| F-PLAN-R3-P1-07 | physical aliases rely on unclosed identity | §10 ledger, basis, audit, unknown→NO-GO | F-DATA-02 | AC08 | T08 | EV08 | REMEDIATED |
| F-PLAN-R3-P1-08 | power method interface incomplete | §10 typed parameters/method document/cases | F-DATA-05 | AC11 | T11 | EV07 | REMEDIATED |
| F-PLAN-R3-P1-09 | metrology/reference interface incomplete | §10 complete result and two runtime projections | F-DATA-06 | AC12 | T12 | EV06 | REMEDIATED |
| F-PLAN-R3-P1-10 | release/claim and lost monitoring/deviation | §§10–12 identities, transitions, head, monitoring, deviation | F-DATA-04,F-OPS-01–04 | AC10,22–25 | T10,21–24 | EV09,15–17 | REMEDIATED |

The original R1 ledger remains historically REMEDIATED/OPEN as recorded by R2;
no verdict is promoted. Its 13 closure states are preserved by §§2,7–14 and
F-R01–R10, F-DATA-01–06, F-HOLD-01–02, F-TRUST-01–04, F-SEC-01–03,
F-OPS-01–06 and F-COMPAT-01–04. Monitoring cadence/metrics and immutable
deviation requirements are explicitly restored in F-OPS-04/AC25/T24/EV17 and
F-DATA-04/AC10/T10/EV09.

## 17. R3 author internal audit

This author audit is not approval.

```text
SCIENTIFIC_DEFAULTS_INVENTED=0
HIDDEN_DEFAULTS=0
MISSING_VALUE_GRAMMARS=0
MISSING_UNITS=0
AMBIGUOUS_DECISION_AUTHORITIES=0
OWNER_DECISION_BOOTSTRAP_CIRCULARITY=0
AUTHORITY_ENROLLMENT_BOOTSTRAP_CIRCULARITY=0
EXTERNAL_AUTHORITY_ENFORCEMENT_AMBIGUITIES=0
CHECKER_BINARY_SOURCE_BINDING_AMBIGUITIES=0
STAGE_IMPLEMENTATION_ORDER_AMBIGUITY=0
UNSPECIFIED_DURABLE_TAG_AUTHORITIES=0
GIT_AUTHORITY_AMBIGUITIES=0
WIRE_CONTRACT_AMBIGUITIES=0
WIRE_IDENTITY_CYCLES=0
DECISION_BUNDLE_ID_AMBIGUITY=0
DECISION_TO_RUNTIME_MAPPING_AMBIGUITIES=0
UNREPRESENTABLE_DECISION_VALUES=0
HIDDEN_TRANSFORMATION_DEFAULTS=0
EXTERNAL_RUNTIME_ID_GRAMMAR_MISMATCHES=0
NUMERIC_REPRESENTATION_AMBIGUITIES=0
PROTOCOL_PROJECTION_UNBOUND_FIELDS=0
REGISTRY_SUBJECT_SEMANTICS_AMBIGUITIES=0
RELEASE_RECORD_SELF_REFERENCE_CYCLES=0
RELEASE_RECORD_SUBJECT_SEMANTICS_AMBIGUITIES=0
CLAIM_STATE_SELF_REFERENCE_CYCLES=0
CLAIM_STATE_CONSTRUCTION_ORDER_AMBIGUITIES=0
OPERATIONAL_STATE_AMBIGUITIES=0
FINAL_TAG_VS_LIVE_STATE_AMBIGUITIES=0
PRIVATE_KEY_REPOSITORY_PATHS=0
TEST_AUTHORITY_TO_PRODUCTION_PATHS=0
TEST_TO_PHYSICAL_EVIDENCE_PROMOTION_PATHS=0
SYNTHETIC_TO_PHYSICAL_CLAIM_PATHS=0
CONSTRUCTED_TO_PHYSICAL_CLAIM_PATHS=0
UNKNOWN_TO_PHYSICAL_CLAIM_PATHS=0
SAME_SOURCE_REFERENCE_INDEPENDENCE_PATHS=0
UNDECLARED_DEPENDENCY_INDEPENDENCE_PATHS=0
PHYSICAL_PSEUDOREPLICATION_PATHS=0
PACKAGE_IDENTITY_AMBIGUITIES=0
POWER_METHOD_INTERFACE_AMBIGUITIES=0
METROLOGY_INTERFACE_GAPS=0
REFERENCE_RESULT_TO_RUNTIME_MAPPING_AMBIGUITIES=0
P2_TEMPORARY_DISPOSITION_AMBIGUITY=0
P2_PUBLIC_RELEASE_BYPASS_PATHS=0
REVOKED_ROOT_PUBLIC_CLAIM_BYPASS_PATHS=0
PRODUCTION_EXECUTION_ORDER_CONTRADICTIONS=0
LOST_R1_NORMATIVE_OBLIGATIONS=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
DUPLICATED_NORMATIVE_CLAUSES=0
CONFLICTING_DUPLICATED_CLAUSES=0
NORMATIVE_CONTRADICTIONS=0
```

The three repeated R2 normative clauses (tag-body type sentence, enrollment
genesis authentication paragraph, and target-domain paragraph) are each stated
once in R3. Repeated field names inside distinct literal wire bodies are required
wire definitions, not duplicated normative prose.

## 18. Validation, commit, push and rereview workflow

Before and after authoring: `git diff --check`; `cargo fmt --all --check`;
`cargo check --locked`; strict all-target/all-feature Clippy; Phase E 38/38;
Phase D 73/73; frozen Phase-E SHA/blob; and exact one-file diff. Create one
forward commit `docs(plan): align Phase F authority with runtime contracts`.
Never amend/reset/rebase/squash/force-push, create Phase-F tags/branch, provision
trust, generate keys/signatures/evidence or claim approval. Immediately before
normal push, live origin/main must still equal the R2 SHA or STOP. After push,
record R3 commit, plan SHA-256 and blob; local/main/origin/live must match and
worktree be clean. No later commit precedes rereview.

A new independent R3 reviewer must inspect the full plan, R2→R3 delta, all ten
R2 P1s, all thirteen R1 closure states, mapping/numeric/tag/enrollment/checker/
registry/identity/custody/power/metrology/release/state/head/monitoring/deviation
contracts and all counterexamples. Prior verdicts do not carry forward.

`READY_FOR_PHASE_F_PLAN_APPROVAL_TAG=NO` and
`READY_FOR_PHASE_F_IMPLEMENTATION=NO` pending fresh independent R3 GO. After a
future plan tag, the next action is `CLOSE_F0_OWNER_DECISIONS`.
