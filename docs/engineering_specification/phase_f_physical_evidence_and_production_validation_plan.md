# MHI V1 Phase F — R4 Physical Evidence and Production Validation Plan

## 1. Identity, status, scope, and chronology

This is the forward R4 planning remediation of R3 commit
`e365b62586810ccfd2c8c6a9231dd970819750aa`, plan SHA-256
`16d37f2a7464ebc9aa8d00a229d27990f4979b96da4ee9ad401c6d0ce1b363ed`,
and Git blob `445496e066a7cb495157e02cf73af3ab4973759a`. Review history is
R1 NO-GO/P1=13, R2 NO-GO/P1=10, R3 NO-GO/P1=19. R4 is forward planning
remediation; independent rereview is PENDING. No Phase-F plan is approved.

Immutable Phase-E authority remains integrated baseline
`14942a30928b88f16914bf0bb103cc0c2a5bfa76`, reviewed implementation
`5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb`, frozen plan SHA-256
`0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`,
and blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

Runtime, external governance, F0 scientific decisions, real physical evidence,
and test/KAT authority are disjoint. Synthetic, constructed, unknown-origin,
test, or KAT material cannot support a physical claim. This document creates no
schema file, decision, tag, branch, key, signature, trust, registry record,
physical evidence, or operational record and changes no runtime behavior.

Chronology is exact: R4 rereview/plan tag; F0 review/tag; F-IMPL-1 checker and
permanent closure of F-MAINT-01/02; readiness review/tag; offline/HSM keys;
enrollment review/tag; genesis; F1 preregistration; F2 evidence/cohort lock; F3
trust provisioning/tag; F4 production run; F5 release/state/final tag. F1–F5
remain blocked until both debts close. P2 temporary disposition is NONE.

```text
F_IMPL_1_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_2_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_3_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_4_BEFORE_F0_EXIT=FORBIDDEN
```

## 2. PHASE-F R4 PRIMITIVE REGISTRY

External files are UTF-8 JCS (RFC 8785), reject duplicate/unknown members, and
have no optional member unless typed `|null`. `schema_version=1` is JSON integer
`1`. Exact objects permit no other fields. Sorted arrays are strictly increasing
by their stated key, so duplicates are forbidden. Complete-file identity is
SHA-256 over exact complete canonical bytes, including IDs/signatures.

| Primitive | Complete grammar |
|---|---|
| `SHA256_V1` | JSON string, exactly 64 bytes from `0-9a-f`; no prefix, uppercase, whitespace. |
| `GIT_SHA_V1` | Exactly 40 lowercase hex characters identifying a Git commit. |
| `GIT_BLOB_V1` | Exactly 40 lowercase hex; `git cat-file -t` returns `blob`. |
| `GIT_TREE_V1` | Exactly 40 lowercase hex; `git cat-file -t` returns `tree`. |
| `RUNTIME_STABLE_ID_V1` | Exact Rust `valid_id()`: nonempty; first byte ASCII alphanumeric; later bytes ASCII alphanumeric or `._:-`; both cases allowed; no Unicode, trim, normalization, or Phase-F max length. |
| `PHASE_F_EXTERNAL_DIGEST_ID_V1` | Exactly `sha256:` plus 64 lowercase hex; external semantic IDs only. |
| `RUNTIME_CANONICAL_TEXT_V1` | Exact Rust `nonempty()`: valid UTF-8, nonempty, no U+0000/U+000D; final character of every Rust `.lines()` logical line, if any, is not Unicode whitespace. No normalization/case change; LF allowed as runtime allows. |
| `RUNTIME_URI_V1` | Exact Rust `valid_uri()`: first byte ASCII alphabetic, contains `:`, all bytes ASCII graphic `0x21..0x7e`; no whitespace/Unicode/normalization. Runtime registration URI only. |
| `IMMUTABLE_EXTERNAL_URI_V1` | `scheme":"remainder`; scheme `[a-z][a-z0-9+.-]*`; nonempty remainder of ASCII graphic bytes; exact bytes; scheme must be in F-OD-20. URI is always accompanied by length/hash. |
| `URI_SCHEME_V1` | `[a-z][a-z0-9+.-]*`. |
| `GITHUB_PRINCIPAL_V1` | 1–39 ASCII bytes from alphanumeric or `-`; first/last alphanumeric; exact case. |
| `UNIT_TEXT_V1` | Nonempty UTF-8; no NUL/CR/LF or leading/trailing Unicode whitespace; no normalization/conversion; exact case. |
| `CANONICAL_INTEGER_V1` | JSON string `0|-?[1-9][0-9]*`; `-0` forbidden. |
| `CANONICAL_UNSIGNED_INTEGER_V1` | JSON string `0|[1-9][0-9]*`. |
| `CANONICAL_POSITIVE_INTEGER_V1` | JSON string `[1-9][0-9]*`. |
| `CANONICAL_DECIMAL_V1` | JSON string `-?(0|[1-9][0-9]*)(\.[0-9]*[1-9])?`; no exponent, plus, leading/trailing zero, bare point, JSON number, or `-0`. |
| `UTC_SECOND_TIMESTAMP_V1` | `YYYY-MM-DDTHH:MM:SSZ`; valid Gregorian UTC, hours 00–23, minutes/seconds 00–59; no leap/fraction/other offset. |
| `DURATION_SECONDS_V1` | `CANONICAL_POSITIVE_INTEGER_V1` elapsed SI seconds; no calendar meaning. |

`RUNTIME_F64_V1` is exactly `{decimal:CANONICAL_DECIMAL_V1,
binary64_bits_hex:<16 lowercase hex>}`. Bits are exact finite IEEE-754 binary64
conversion, round-nearest ties-even; independently recomputed; negative zero
forbidden. `PHASE_F_PARAMETER_VALUE_V1` is the closed union `{type:"integer",
value:CANONICAL_INTEGER_V1}`, `{type:"decimal",value:CANONICAL_DECIMAL_V1}`,
`{type:"runtime_f64",value:RUNTIME_F64_V1}`, `{type:"boolean",value:<boolean>}`,
`{type:"categorical",value:RUNTIME_CANONICAL_TEXT_V1}`, or `{type:"quantity",
value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1}`. No other/free-form value exists.

### 2.1 Numeric domains

| Fields | Exact domain |
|---|---|
| temperature lower/upper | finite `>0`; lower `<` upper; sorted nonoverlapping bands |
| uncertainty maximum/observed | finite `>=0`; exact approved unit |
| rate thresholds/values, prevalence | finite `[0,1]` |
| power target fraction, type-I error | finite `(0,1)`; another method domain must be closed in its interface |
| allocations development/validation/holdout | exact rational `[0,1]`; exact rational sum 1; no binary64 sum authority |
| LOD/LOQ/calibration/QC | exact method-policy range/unit; LOD `<=` LOQ |
| sensor drift/effect size/other quantities | exact interface range and unit; absence is NO-GO |
| u64 counts/minima | unsigned string fitting Rust u64; minima positive and physical endpoint record/family minima `>=2` |

No unlisted decimal/f64 field is permitted. Missing range/unit is NO-GO.

## 3. Exact F0 decision bundle

`PhaseFDecisionBundleV1` is exactly `schema_version,decision_bundle_id,
decisions`. Decisions contain F-OD-01..21 once in ID order, each exactly
`decision_id,value,decision_owner_role,rationale_document_sha256`. Roles are
`SCIENTIFIC_METROLOGY_OWNER|ARCHITECTURE_DATA_OWNER|SECURITY_OWNER|
OPERATIONS_GOVERNANCE_OWNER`. Bundle ID is `"sha256:"+SHA-256(
b"mhi_phase_f_decision_bundle_v1\0"||JCS(object without decision_bundle_id))`.
Arrays below are nonempty sorted unique by first ID unless stated otherwise.

| ID / owner | Complete exact value |
|---|---|
| 01 / operations | `{protocol_id:RUNTIME_STABLE_ID_V1,title:RUNTIME_CANONICAL_TEXT_V1}` |
| 02 / architecture | `{registration_id:RUNTIME_STABLE_ID_V1,immutable_reference_uri:RUNTIME_URI_V1,document_sha256:SHA256_V1}` |
| 03 / scientific | `DomainSelectorDecisionV1`: five axes `analytes,matrices,sensor_designs,sensors,campaigns`, each `{type:"allowed",ids:[RUNTIME_STABLE_ID_V1]}`, plus `temperature:{type:"bands",bands:[{lower_kelvin_inclusive:RUNTIME_F64_V1,upper_kelvin_exclusive:RUNTIME_F64_V1}]}` |
| 04 / security | `{trust_root_id:RUNTIME_STABLE_ID_V1}` |
| 05 / scientific | `{mechanism_endpoints:[MechanismEndpointDecisionV1]}` defined below |
| 06 / scientific | `{health_endpoints:[HealthEndpointDecisionV1]}` defined below |
| 07 / operations | `{claims:[{claim_id:RUNTIME_STABLE_ID_V1,statement:RUNTIME_CANONICAL_TEXT_V1,domain:DomainSelectorDecisionV1,supporting_endpoint_ids:[RUNTIME_STABLE_ID_V1]}]}` |
| 08 / scientific | `{evidence_categories:[{category:SCIENTIFIC_EVIDENCE_CATEGORY_V1,may_support:<boolean>,may_contradict:<boolean>,claim_ceiling:SCIENTIFIC_CLAIM_CEILING_V1}]}` once per category in enum order |
| 09 / architecture | `{split_unit:RUNTIME_STABLE_ID_V1,allocations:{development:CANONICAL_DECIMAL_V1,validation:CANONICAL_DECIMAL_V1,holdout:CANONICAL_DECIMAL_V1},stratification_keys:[RUNTIME_STABLE_ID_V1],randomization_algorithm_id:RUNTIME_STABLE_ID_V1,seed_authority:RUNTIME_STABLE_ID_V1,split_execution_authority_id:RUNTIME_STABLE_ID_V1,lock_point:"before_outcome_access",post_hoc_movement:"forbidden"}` |
| 10 / architecture | `{unit_kinds:[RUNTIME_STABLE_ID_V1],independent_kind_by_endpoint:[{endpoint_id:RUNTIME_STABLE_ID_V1,unit_kind:RUNTIME_STABLE_ID_V1}],identity_issuance_procedure_sha256:SHA256_V1,parent_child_rules:[{parent_kind:RUNTIME_STABLE_ID_V1,child_kind:RUNTIME_STABLE_ID_V1,procedure_document_sha256:SHA256_V1}],repeat_handling:"same_family_no_increment"}` |
| 11 / scientific | complete `PhaseFMetrologyPolicyV1` value in §9 |
| 12 / scientific | `{power_analysis_id:RUNTIME_STABLE_ID_V1,power_method_id:RUNTIME_STABLE_ID_V1,power_method_version:RUNTIME_CANONICAL_TEXT_V1,power_method_interface:PhaseFObjectReferenceV1}` |
| 13 / operations | `{authority_id:RUNTIME_STABLE_ID_V1,authority_role:"production_owner",authority_document:PhaseFObjectReferenceV1}` |
| 14 / operations | `{authority_id:RUNTIME_STABLE_ID_V1,authority_role:"production_registry",authority_document:PhaseFObjectReferenceV1,registry_namespace_id:RUNTIME_STABLE_ID_V1,registry_head_resolver_uri:IMMUTABLE_EXTERNAL_URI_V1,registry_head_max_validity_seconds:DURATION_SECONDS_V1}` |
| 15 / security | `{custody_method_id:RUNTIME_STABLE_ID_V1,custody_procedure_document:PhaseFObjectReferenceV1,owner_custodian_role:RUNTIME_STABLE_ID_V1,registry_custodian_role:RUNTIME_STABLE_ID_V1,required_quorum:CANONICAL_POSITIVE_INTEGER_V1,key_input_channel_id:RUNTIME_STABLE_ID_V1,network_mode:"offline"|"hsm_isolated",key_persistence_allowed:false,production_cli_access_allowed:false}` |
| 16 / security | `{trigger_actions:[{trigger_code:ROTATION_TRIGGER_V1,required_state:"suspended"|"withdrawn",revalidation_scope:"endpoint"|"full",new_approval_required:true,new_run_required:true}],procedure_document_sha256:SHA256_V1,unsupported_lifecycle_action:"f3_no_go"}`; one row per closed trigger |
| 17 / operations | `{claim_validity_seconds:DURATION_SECONDS_V1,periodic_review_seconds:DURATION_SECONDS_V1,suspension_sla_seconds:DURATION_SECONDS_V1}` |
| 18 / operations | `{deviation_actions:[{deviation_code:RUNTIME_STABLE_ID_V1,required_action:"exclude_before_lock"|"resolved_no_effect"|"campaign_no_go"}]}` total for allowed codes |
| 19 / operations | complete `PhaseFMonitoringPolicyV1` value in §11 |
| 20 / operations | `{allowed_immutable_uri_schemes:[URI_SCHEME_V1],retention_seconds:DURATION_SECONDS_V1,backup_copy_count:CANONICAL_POSITIVE_INTEGER_V1,backup_verification_interval_seconds:DURATION_SECONDS_V1,authorized_access_role_ids:[RUNTIME_STABLE_ID_V1],replacement_authority_role_id:RUNTIME_STABLE_ID_V1,unavailable_object_action:"no_go"}` |
| 21 / operations | `{release_coordinator_principal:GITHUB_PRINCIPAL_V1,governance_coordinator_principal:GITHUB_PRINCIPAL_V1}` |

`ROTATION_TRIGGER_V1` is `key_rotation|key_compromise|key_revocation|
method_version_change|protocol_revision|domain_expansion|code_change|
sensor_design_change|report_withdrawal|superseding_campaign`.

F-OD-08 category enum is `direct_physical_observation|
orthogonal_physical_measurement|validated_proxy|model_derived|
same_signal_derived|expert_interpretation|unavailable`; claim ceiling is
`physical|limited|not_assessed|unavailable|none`. `model_derived` and
`same_signal_derived` have `may_support=false`; expert interpretation cannot
support alone; unavailable is exactly `false,false,unavailable`. Remaining
category values are reviewed F0 choices. This external F2 gate never changes
runtime support levels, policy, partition, exclusions, or arithmetic.

`ReferenceRuleDecisionV1` is exact `allowed_methods:[{method_id:
RUNTIME_STABLE_ID_V1,method_version:RUNTIME_CANONICAL_TEXT_V1}],
allowed_authority_ids:[RUNTIME_STABLE_ID_V1],blinding_rule:"require_blinded",
uncertainty_rule:{type:"require_quantified",measure_id:RUNTIME_STABLE_ID_V1,
unit:UNIT_TEXT_V1,maximum_inclusive:RUNTIME_F64_V1}`. Runtime authorizes the full
Cartesian product; pair-only policy is unrepresentable/F1 NO-GO.

`StratumDecisionV1` is exact `stratum_id,predicates,minimum_eligible_records,
minimum_independent_families`; predicates use exact Rust tags
`analyte_equals|matrix_equals|sensor_design_equals|sensor_equals|
campaign_equals|temperature_band`, each axis at most once in that order.
`AcceptanceRuleDecisionV1` is isomorphic to exact Rust `AcceptanceRuleV1`:
count fields are `type,rule_id,metric:<CountMetricV1 token>,comparator:
greater_than_or_equal|less_than_or_equal,threshold_u64`; rate fields are
`type,rule_id,metric:<RateMetricV1 token>,target:<RateTargetV1 token>,comparator,
threshold:RUNTIME_F64_V1`.

`MechanismEndpointDecisionV1` exact fields: `endpoint_id,hypothesis_id,domain,
reference_rule,support_levels,minimum_eligible_records,
minimum_independent_families,required_strata,acceptance_rules`; support levels
are `hypothesized|experimentally_supported|validated_for_domain`. Projection
fixes `critical_policy=any_contradicted_record_fails`, `cohort_role=holdout`,
`mechanism_artifact_required=true`.

`HealthEndpointDecisionV1` exact fields: `endpoint_id,target,domain,
reference_rule,predicted_positive_statuses,predicted_negative_statuses,
reference_label_universe,reference_positive_labels,reference_negative_labels,
minimum_eligible_records,minimum_independent_families,required_strata,
acceptance_rules`. `target` is exact Rust `HealthTargetV1`; statuses partition
`within_baseline|watch|degraded|critical`, labels partition their universe.
Projection fixes `cohort_role=holdout`, `health_artifact_required=true`.

## 4. Runtime projection and frozen behavior

`PhaseFProtocolProjectionV1` is a plan contract, not wire. It constructs TOML
from F-OD-01..07, parses exact `MhiValidationProtocolV1::from_toml`, and compares
every field/bit: 01 copies protocol ID/title; 02 copies `ProtocolRegistrationV1`;
03 maps only to `CategoricalSelectorV1::Allowed` and
`TemperatureSelectorV1::Bands`; 04 maps to
`PhysicalApprovalAuthorityV1::EmbeddedTrustRoot`; 05/06 structurally copy exact
endpoint fields plus invariants; 07 copies claims with
`requested_level=Physical`. Invariants are `schema_version=1` and statistics
`wilson_95_v1,0.95,unavailable,indeterminate,and`. F-OD-08..21 have no runtime
projection/override. Missing, extra, defaulted, normalized, transformed, or
unrepresentable values are F0/F1 NO-GO.

Frozen results remain: malformed/binding failures hard-error with no report;
unprovisioned physical approval hard-errors before evaluation; existing ordered
`Excluded`, `Indeterminate`, `DoesNotMeetProtocol`, and passing outcomes remain
unchanged. External policy cannot reclassify them. The four existing runtime
release outcomes and production execution order remain unchanged.

## 5. Six durable tags

`PHASE_F_BOOTSTRAP_COORDINATOR` is procedural GitHub repository owner `XingyuW`
and creates only the plan-approved tag; it is not cryptographic reviewer
identity. After F0, `PHASE_F_RELEASE_COORDINATOR` and
`PHASE_F_GOVERNANCE_COORDINATOR` mean exactly the F-OD-21 principal values.

For every tag: prerequisite GO exists; target is pushed/live; tag is absent
locally/remotely; create annotated tag locally; byte-verify body and peeled
target; push only the tag normally; verify remote tag object, peeled target, and
body equality; never move it. A tag is never pushed before its target commit.

Bodies are printable ASCII plus LF, exactly one final LF, fixed order, no blank,
duplicate, trailing whitespace, optional field, LF/`=` in value. The six creator/
target pairs and exact ordered fields are:

| Tag | Creator | Target | Ordered fields after literal body-type line |
|---|---|---|---|
| `ism-mechanism-health-v1-f-plan-approved` / `PhaseFPlanApprovalV1` | bootstrap | reviewed R4 main | `format_version,plan_review_sha,plan_sha256,plan_git_blob,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-f0-decisions-approved` / `PhaseFDecisionApprovalV1` | governance | reviewed F0 main | `format_version,phase_f_plan_tag,plan_review_sha,decision_review_sha,decision_bundle_id,decision_file_sha256,decision_git_blob,decision_count,release_coordinator_principal,governance_coordinator_principal,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-readiness-approved` / `PhaseFReadinessApprovalV1` | release | integrated F-IMPL-1 | `format_version,phase_f_plan_tag,f0_decisions_tag,readiness_review_sha,checker_source_review_sha,checker_source_tree,checker_dependency_lock_sha256,checker_binary_sha256,macos_uname,macos_arch,macos_product_version,macos_build_version,rustc_version,cargo_version,build1,build2,reproducible_binary,f_maint_01,f_maint_02,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-authority-enrollment-approved` / `PhaseFAuthorityEnrollmentApprovalV1` | governance | readiness main | `format_version,phase_f_plan_tag,f0_decisions_tag,readiness_tag,readiness_main_sha,enrollment_sha256,owner_authority_id,registry_authority_id,owner_public_key_fingerprint,registry_public_key_fingerprint,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-trust-provisioning-approved` / `PhaseFTrustProvisioningApprovalV1` | release | integrated F3 main | `format_version,phase_f_plan_tag,f0_decisions_tag,readiness_tag,authority_enrollment_tag,enrollment_sha256,owner_public_key_fingerprint,registry_public_key_fingerprint,trust_root_id,trust_review_sha,trust_store_git_blob,trust_store_sha256,f2_cohort_lock_registry_record_sha256,macos_uname,macos_arch,macos_product_version,macos_build_version,macos_result,security_decision,compatibility_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-physical-validation-released` / `PhaseFPhysicalReleaseApprovalV1` | release | final F4/F5 main | `format_version,phase_f_plan_tag,f0_decisions_tag,readiness_tag,authority_enrollment_tag,trust_provisioning_tag,release_code_sha,protocol_sha256,cohort_lock_registry_record_sha256,owner_approval_record_id,owner_approval_file_sha256,validation_manifest_sha256,release_record_id,release_file_sha256,release_registry_record_sha256,initial_claim_state_record_id,initial_claim_state_file_sha256,initial_claim_state_registry_record_sha256,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,macos_result,release_decision` |

Every `format_version=1`, tag-reference value is the literal listed name, SHA/
Git/count/ID use §2, `decision_count=21`, decisions/approval/release are GO,
P0/P1 are 0, build/maintenance/macOS values are PASS/CLOSED as named.

## 6. Checker build authority

The checker owns `tools/phase_f_authority_checker/Cargo.toml`, its own
`Cargo.lock`, and `src/**`. `checker_source_tree` is the Git tree of exactly the
tool directory, including every tracked file. `checker_dependency_lock_sha256`
is SHA-256 of exact checker-local lock bytes; root/shared lock is forbidden.

```sh
cargo build --locked --release --manifest-path tools/phase_f_authority_checker/Cargo.toml
```

Expected executable is `tools/phase_f_authority_checker/target/release/
phase-f-authority-check` unless readiness freezes `CARGO_TARGET_DIR`. Each clean
build creates new empty source/target directories; materializes exact
`checker_source_review_sha`; requires empty `git status --short`; verifies tree
and lock; uses exact reviewed rustc/cargo/macOS product/build/architecture; runs
the exact command; hashes executable; records command and exact whitelist
`PATH,HOME,CARGO_HOME,CARGO_TARGET_DIR,TMPDIR,SDKROOT,
MACOSX_DEPLOYMENT_TARGET`. `RUSTFLAGS`, target, features, linker, profile, and
manifest overrides are absent. Builds use distinct source/target directories;
shared Cargo registry cache is allowed because lock checksums bind packages.
Binary hashes must match and readiness binds the result. The checker is read-only
and uses one parser for KAT and real input.

## 7. Enrollment, typed registry, and current head

`PhaseFAuthorityEnrollmentV1` exact fields are `schema_version,enrollment_id,
phase_f_plan_tag,f0_decisions_tag,readiness_tag,owner_authority_id,
registry_authority_id,owner_public_key,registry_public_key,
owner_public_key_fingerprint,registry_public_key_fingerprint,
owner_authority_document,registry_authority_document,custody_policy_sha256,
created_at,owner_signature,registry_signature`. Keys/signatures are respectively
64/128 lowercase hex characters; fingerprints hash decoded key bytes. Semantic
ID excludes itself and signatures and uses domain `mhi_phase_f_enrollment_v1\0`.
`enrollment_sha256` unequivocally hashes exact canonical COMPLETE file bytes,
including ID/signatures. The enrollment tag and genesis subject use this one
meaning; never payload, key, registry-subject, blob, or tree hash.

`REGISTRY_OBJECT_KIND_V1` is closed: `decision_bundle|git_tag_body|
authority_enrollment|registry_record|registry_head|registration_document|
protocol|power_analysis|power_method_interface|package_manifest|
dependency_audit|physical_unit_ledger|identity_audit|location_ledger|
chain_of_custody|deviation_ledger|metrology_policy|reference_result|
reference_source_descriptor|cohort_lock|owner_approval|execution_record|
release_record|claim_state|reinstatement_approval|monitoring_policy|
monitoring_record|retention_audit|incident_record|retrieval_verification`.
`REGISTRY_RELATION_TYPE_V1` is `authorized_by|depends_on|registered_after|locks|
approves|executes|releases|changes_state_of|supersedes|references`.

`PhaseFRegistryRelationV1` is exactly `relation_type,object_kind,object_sha256`,
sorted unique by those fields. `PhaseFRegistryRecordV1` is exactly
`schema_version,registry_namespace_id,registry_authority_id,sequence,
predecessor_record_sha256,record_kind,subject_id,subject_sha256,relations,
created_at,registry_key_fingerprint,signature`; sequence is unsigned string,
predecessor `SHA256_V1|null`, signature 128 lowercase hex over domain-separated
JCS excluding signature. The table is exhaustive; all unlisted tuples forbidden.

| record kind | Subject ID / subject SHA-256 meaning | Exact relations | Construction |
|---|---|---|---|
| `authority_enrolled` | enrollment ID / complete enrollment file | `authorized_by+decision_bundle` once; `references+git_tag_body` exactly plan/F0/readiness/enrollment tags | sequence 0 |
| `protocol_registered` | runtime protocol ID / complete protocol | `authorized_by+decision_bundle` once; `depends_on+registration_document` once | after genesis |
| `power_registered` | analysis ID / complete power analysis | `authorized_by+decision_bundle`, `depends_on+power_method_interface`, `depends_on+protocol`, each once | after protocol |
| `package_registered` | manifest ID / complete manifest | once each `depends_on+dependency_audit`, `physical_unit_ledger`, `identity_audit`, `location_ledger`, `chain_of_custody`, `deviation_ledger`, `metrology_policy`; one or more each `reference_result`, `reference_source_descriptor` | after dependencies |
| `cohort_locked` | cohort-lock ID / complete lock | once each `locks+package_manifest`, `depends_on+protocol`, `power_analysis`, `deviation_ledger` | after package |
| `owner_approval_registered` | approval digest ID / complete approval | once `approves+cohort_lock`, once `authorized_by+authority_enrollment` | after lock |
| `execution_registered` | execution ID / complete execution | once `executes+cohort_lock`, `authorized_by+owner_approval`, `depends_on+deviation_ledger`, `depends_on+protocol` | after approval |
| `release_registered` | release digest ID / release semantic digest | once `releases+execution_record`, `authorized_by+owner_approval`, `depends_on+monitoring_policy`, `depends_on+metrology_policy` | after execution |
| `claim_state_changed` | state digest ID / state semantic digest | once `changes_state_of+release_record`; zero/one `registered_after+claim_state`, `depends_on+reinstatement_approval`, `supersedes+release_record` as transition requires | after release/prior state |
| `monitoring_recorded` | monitoring ID / complete monitoring file | once `references+release_record`, `depends_on+monitoring_policy`; zero/one `registered_after+monitoring_record` | after active state |
| `incident_recorded` | incident ID / complete incident file | once `references+release_record`; zero or more `references+monitoring_record` | after evidence |
| `retention_audited` | audit ID / complete audit file | one or more `references+package_manifest`; once `references+release_record` | after release |

The chain is linear. Genesis sequence is 0/null predecessor. Later sequence is
previous+1 and predecessor hashes exact previous complete record bytes. Genesis
is accepted only after plan, F0, readiness, enrollment tags; enrollment complete
hash; F0 namespace/registry ID; enrollment registry fingerprint; then signature.

`PhaseFRegistryHeadV1` exact fields are `schema_version,
registry_namespace_id,registry_authority_id,sequence,registry_record_sha256,
issued_at,valid_until,registry_key_fingerprint,signature`. Valid-until is later
than issued-at by at most F-OD-14 max validity. Evaluation requires
`issued_at<=now<valid_until`. Same sequence/same hash is same head; same sequence/
different hash is EQUIVOCATION, NO-GO, not ACTIVE; lower than a supplied verified
watermark is REGRESSION/AUTHORITY_UNAVAILABLE; higher verifies all intervening
records. Resolver unavailable, bad signature, or expiry is
AUTHORITY_UNAVAILABLE. V1 persists no local state: first-use currentness relies
on authenticated live resolver and signed freshness. It accepts an optional
caller watermark exact tuple `(registry_namespace_id,sequence,
registry_record_sha256)` and rejects regression. No stronger replay protection
is claimed.

## 8. Retrieval, package, physical identity, custody, and deviation

`PhaseFObjectReferenceV1` is exactly `immutable_uri,sha256,byte_length` using
`IMMUTABLE_EXTERNAL_URI_V1`, `SHA256_V1`, and unsigned integer. Retrieval
failure is NO-GO. `PhaseFRetrievalVerificationV1` is exactly `schema_version,
retrieval_id,object_reference,retrieved_sha256,retrieved_byte_length,
checker_binary_sha256,checker_source_review_sha,retrieved_at,
verification_result`; result `pass|no_go`. Unavailable objects never receive a
fabricated pass. This is operational, not scientific evidence.

`PhaseFPackageManifestV1` exact fields: `schema_version,manifest_id,objects,
bindings`. Objects sorted by object ID are exactly `object_id,object_reference,
media_type,format_or_schema,producing_authority_id,physical,test_only,generated,
retention_class_id`. Bindings sorted by binding ID are exactly `binding_id,role,
object_id,physical_unit_ids,direct_dependency_binding_ids`; both ID arrays sorted
unique. Object IDs are `RUNTIME_STABLE_ID_V1`; text uses
`RUNTIME_CANONICAL_TEXT_V1`; booleans are JSON. Every binding references a known
object/binding/unit and dependency graph is acyclic. Duplicate SHA-256 under
different object IDs is NO-GO; multiple roles reuse the same object ID.

`PhaseFDependencyAuditV1` exact fields: `schema_version,dependency_audit_id,
manifest_id,edges,undeclared_dependency_count,unknown_separation_count,result`.
Edges sorted by `(from_binding_id,to_binding_id)` are exactly
`from_binding_id,to_binding_id,dependency_type,source_document_sha256`; type is
`raw_source|sample|sensor|preprocessing|model|reference|derived_output`; counts
are unsigned; result `pass|no_go`; pass requires both counts 0 and exact equality
to manifest dependencies.

`PhaseFPhysicalUnitLedgerV1` exact fields: `schema_version,unit_ledger_id,
entries`. Entries sorted by unit ID are exactly `unit_id,unit_kind,
identity_basis,identity_basis_document_sha256,parent_unit_ids,
independent_family_id,source_object_ids`; arrays sorted unique. Identity basis is
`issuer_serial|native_specimen_id|registered_barcode|custody_created_child|
other_registered_identity_basis`. Basis-document hash is mandatory for every
basis and defines issuance/source procedure; especially `other` has no prose
fallback. Different IDs do not prove different units. Unresolved alias is
UNKNOWN PHYSICAL IDENTITY/F2 NO-GO.

`PhaseFPhysicalIdentityAuditV1` exact fields: `schema_version,
identity_audit_id,unit_ledger_sha256,comparisons,unknown_identity_count,
alias_count,result`. Comparisons sorted by `(left_unit_id,right_unit_id)` are
exactly `left_unit_id,right_unit_id,determination,evidence_sha256`; determination
`distinct|same|unknown`. Pass requires unknown/alias counts zero and all claimed
independent families proved distinct.

`PhaseFLocationLedgerV1` exact fields: `schema_version,location_ledger_id,
locations`; each sorted location is exactly `location_id,location_type,
authority_id,identity_document_sha256`. Type is `collection_site|laboratory|
storage|instrument_station|transport_container|other_registered_location`;
identity document defines every location and all `other` semantics.

`PhaseFChainOfCustodyV1` exact fields: `schema_version,custody_ledger_id,
campaign_id,unit_ledger_sha256,location_ledger_sha256,events`. Events sorted by
`(occurred_at,event_id)` are exactly `event_id,event_type,occurred_at,
source_location_id,destination_location_id,input_unit_ids,output_unit_ids,
procedure_document_sha256,deviation_id`. Location/deviation fields are
`RUNTIME_STABLE_ID_V1|null`; procedure hash is `SHA256_V1|null`; unit arrays
sorted unique. All nonnull locations exist. Per-type constraints are exhaustive:

| Type | source / destination | input / output | continuation/creation/procedure/deviation |
|---|---|---|---|
| acquired | null / required | empty / nonempty | creates ledger units; procedure required; deviation null unless reported |
| transferred | required distinct / required | nonempty / identical | same units; no child; procedure required |
| aliquoted | required / required | one parent / nonempty new children | child links must match ledger; procedure required |
| processed | required / required | nonempty / nonempty | same continuation or registered children; procedure required |
| measured | required / required same | nonempty / identical | no creation; procedure required |
| stored | required / required | nonempty / identical | same units; procedure required |
| released_to_analysis | required / required | nonempty / identical | same units; procedure required |
| destroyed | required / null | nonempty / empty | no later use unless a later deviation-ledger revision explicitly invalidates this event; procedure required |

`PhaseFDeviationLedgerV1` exact fields: `schema_version,deviation_ledger_id,
campaign_id,revision_number,previous_revision_sha256,events`. Genesis is 0/null;
later is previous+1 and previous hash is complete prior file. Later files contain
all prior event JSON values byte-identically and append only. Events sorted by
append order are exactly `event_id,deviation_id,event_type,affected_unit_ids,
affected_object_sha256s,deviation_code,detected_stage,required_action,
decision_authority_id,rationale_document_sha256`; affected arrays sorted unique;
type is `reported|resolved_excluded|resolved_no_effect|campaign_no_go`; stage is
`f1|f2|f3|f4|f5`; action is `exclude_before_lock|resolved_no_effect|
campaign_no_go`. First event per deviation is reported; resolution follows; no
incompatible second terminal resolution. Undocumented deviation is NO-GO.
Cohort lock binds exact F2 revision; execution binds latest F4 revision; material
post-lock deviation applies F-OD-18 and may invalidate cohort.

## 9. Power and metrology/reference interface

`PhaseFPowerMethodInterfaceV1` exact fields: `schema_version,power_method_id,
power_method_version,method_document_sha256,primary_metric_ids,parameter_specs,
required_sensitivity_case_ids,output_spec`. ID arrays sorted unique. Parameter
specs sorted by ID are exactly `parameter_id,value_type,unit_rule,required,
range_rule`; value type `integer|decimal|runtime_f64|boolean|categorical|
quantity`; unit rule `{type:"none"}`, `{type:"exact",unit:UNIT_TEXT_V1}`, or
`{type:"owner_selected_exact"}`. Range rule is exactly one of `{type:
"unbounded"}`, `nonnegative`, `positive`, `{type:"closed_interval",minimum:
PHASE_F_PARAMETER_VALUE_V1,maximum:PHASE_F_PARAMETER_VALUE_V1}`,
`open_interval` with same bounds, or `{type:"enum_values",values:[
PHASE_F_PARAMETER_VALUE_V1]}`; endpoints match value type/unit. Output spec exact
fields are `minimum_eligible_records_output_id,
minimum_independent_families_output_id,minimum_positive_records_output_id,
minimum_negative_records_output_id,required_strata_output_id`.

`PhaseFPowerAnalysisRecordV1` exact fields: `schema_version,power_analysis_id,
power_method_id,power_method_version,power_method_interface_sha256,
software_source_sha,software_binary_sha256,parameters,sensitivity_cases,outputs,
created_at`. Parameters sorted by ID are exact `parameter_id,value`; cases sorted
by case ID are exact `case_id,parameters,outputs`; outputs sorted by output ID are
exact `output_id,value`. Checker enforces method ID/version, all and only required
parameters, type/unit/range, sensitivity IDs, software identity and output minima.
Scientific reviewer alone decides method adequacy.

`PhaseFMetrologyPolicyV1` exact fields: `schema_version,metrology_policy_id,
endpoint_policies`. Entries sorted by endpoint ID are exactly `endpoint_id,
reference_type,allowed_method_id,allowed_method_version,allowed_authority_ids,
measurand_id,result_unit,blinding_requirement,uncertainty_policy,lod_loq_policy,
calibration_policy,qc_policy,chain_of_custody_required,
traceability_document_required,limitations_document_required`. Reference type is
`mechanism|health`; blinding is `blinded_to_assessment`; booleans are true for
physical validation. Uncertainty exact fields are `measure_id,unit,
maximum_inclusive`. LOD/LOQ is `{type:"not_applicable"}` or `{type:"required",
lod_value,lod_unit,loq_value,loq_unit,below_lod_action,
between_lod_loq_action}`; actions `exclude_before_lock|campaign_no_go`.
Calibration and QC each are exactly `check_ids:[RUNTIME_STABLE_ID_V1],
failure_action:"exclude_before_lock"|"campaign_no_go"`. These actions are
external eligibility only. Policy must agree with F-OD-05/06 reference rule;
unrepresentable method-authority pairing is F1 NO-GO.

`PhaseFReferenceSourceDescriptorV1` exact fields: `schema_version,
reference_source_id,source_file_sha256,evidence_origin,dependency_completeness,
experiment_scope,acquisition_families,direct_dependencies`. The last five use
exact existing Rust `EvidenceOriginV1`, `ReferenceDependencyCompletenessV1`,
`ArtifactExperimentScope`, `ArtifactAcquisitionFamilies`, and sorted exact
`ReferenceDependencyV1` values. Physical admissibility requires origin physical
and completeness complete. Checker projects it isomorphically to exact runtime
`ReferenceSourceAuthorityV1`.

`PhaseFReferenceResultV1` exact fields: `schema_version,reference_result_id,
endpoint_id,hypothesis_id_or_health_target,reference_source_descriptor_sha256,
reference_source_id,outcome_or_label,method_id,method_version,authority_id,
blinding_status,uncertainty_measure_id,uncertainty_value,uncertainty_unit,
result_value,result_unit,limitations_document_sha256,traceability_document_sha256,
chain_of_custody_sha256`. Target is `{type:"mechanism",hypothesis_id:
RUNTIME_STABLE_ID_V1}` or `{type:"health",target:<exact HealthTargetV1>}`.
Outcome/label and blinding use exact runtime tokens/text; result value is
`PHASE_F_PARAMETER_VALUE_V1`. Mechanism hypothesis must byte-equal protocol;
health target must semantically equal protocol. Descriptor supplies dependency
completeness, scope, acquisition families, dependencies; result supplies the
remaining runtime authority fields. Missing/unprojectable source is F2 NO-GO.

Physical origin alone never proves independence. Before F2 lock dependency,
identity, custody, and scientific-admissibility audits all PASS. Unproved shared
source, preprocessing, sample, sensor, or model lineage is UNKNOWN SEPARATION,
cannot count as independent physical support, and is F2 NO-GO where required.

## 10. Cohort, execution, release, and claim state

`PhaseFCohortLockRecordV1` exact fields: `schema_version,cohort_lock_id,
protocol_sha256,package_manifest_sha256,power_analysis_sha256,
dependency_audit_sha256,physical_unit_ledger_sha256,identity_audit_sha256,
location_ledger_sha256,chain_of_custody_sha256,deviation_ledger_sha256,
metrology_policy_sha256,reference_result_sha256s,
reference_source_descriptor_sha256s,locked_at`; arrays sorted unique. ID is the
domain-separated semantic digest excluding itself.

`PhaseFExecutionRecordV1` exact fields: `schema_version,execution_id,
cohort_lock_record_sha256,owner_approval_file_sha256,protocol_sha256,
deviation_ledger_sha256,release_code_sha,checker_binary_sha256,
validation_manifest_sha256,started_at,completed_at,result`; result `pass|no_go`,
completed later than started. ID is semantic digest excluding itself.

`PhaseFReleaseRecordV1` exact fields: `schema_version,release_record_id,
claim_id,claim_statement,release_code_sha,protocol_sha256,
cohort_lock_record_sha256,owner_approval_file_sha256,execution_record_sha256,
validation_manifest_sha256,monitoring_policy_sha256,metrology_policy_sha256,
valid_from,valid_until,limitations,registry_record_sha256`. Semantic payload
excludes ID and registry pointer. ID/digest is domain
`mhi_phase_f_release_record_v1\0`. Registry subject binds semantic digest; then
pointer is inserted and complete-file hash computed. Final tag binds ID,
complete file, and registry record; no identity cycle.

`PhaseFReinstatementApprovalV1` exact fields: `schema_version,reinstatement_id,
claim_id,suspended_state_record_id,suspension_reason,
required_corrective_action,corrective_evidence_sha256s,execution_record_sha256,
scientific_decision,architecture_decision,security_decision,
compatibility_decision,operations_decision,p0_count,p1_count,approval_decision`.
Evidence hashes sorted unique. Reinstatement requires five GO, P0/P1 zero, GO.
If F-OD-16 requires new release, this object is forbidden.

`PhaseFClaimStateRecordV1` exact fields: `schema_version,
claim_state_record_id,claim_id,release_record_id,previous_claim_state_record_id,
state,reason_code,effective_at,superseding_release_record_id,
reinstatement_approval_sha256,limitations,registry_record_sha256`. Nullable:
previous, superseding, reinstatement, registry pointer; no others. Semantic
payload excludes ID/pointer; ID uses `mhi_phase_f_claim_state_v1\0`; registry
then binds digest and pointer completes file. Registry sequence is authoritative
ordering; it strictly increases, and `effective_at>=previous effective_at`.
Timestamp regression is invalid; timestamp never reorders sequence.

Transitions: NONE→active only initial release; active→suspended/withdrawn/
expired/superseded; suspended→active only with exact reinstatement approval;
suspended→withdrawn/expired/superseded. Withdrawn, expired, superseded terminal.
Reasons are exact `initial_release|key_compromise|key_revocation|
monitoring_breach|reference_qc_breach|domain_breach|periodic_expiry|
superseded_by_new_release|manual_withdrawal|approved_reinstatement` and must match
transition. Suspended→active references reinstatement hash; no prose evidence.

Public status uses current signed head, complete chain, latest state, monitoring,
expiry, and release/final-tag binding; returns `ACTIVE|NOT_ACTIVE|
AUTHORITY_UNAVAILABLE`. Authority-unavailable is not active. Historical tags do
not override later governance. Runtime CLI output cannot override live state.

## 11. Monitoring and retention

`PhaseFMonitoringValueV1` is exactly one of `{type:"status",value:
RUNTIME_STABLE_ID_V1}`, `{type:"rate",value:RUNTIME_F64_V1}` in `[0,1]`,
`{type:"quantity",value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1}`,
`{type:"git_sha",value:GIT_SHA_V1}`, `{type:"sha256",value:SHA256_V1}`,
`{type:"stable_id",value:RUNTIME_STABLE_ID_V1}`, or
`{type:"external_digest_id",value:PHASE_F_EXTERNAL_DIGEST_ID_V1}`.

Metric/type map is exact: domain compliance/status; reference QC/status;
calibration/status; sensor drift/quantity; invalid input, indeterminate, data
quality insufficient, and exclusion/rate; reference uncertainty/status; software
Git SHA/git_sha; binary and trust store/sha256; trust root/stable_id; owner
approval and release record/external_digest_id.

`PhaseFMonitoringPolicyV1` exact fields: `schema_version,monitoring_policy_id,
monitoring_interval_seconds,required_metrics,metric_thresholds,
missing_monitoring_action,domain_breach_action,reference_qc_breach_action`.
Required metrics are exactly the 15 above in stated order. Thresholds sorted by
metric ID are exactly `metric_id,comparator,value,unit`; allowed only for sensor
drift and four rate metrics; comparator `greater_than_or_equal|
less_than_or_equal`; value `RUNTIME_F64_V1`; unit is `UNIT_TEXT_V1|null`, null
for rate and exact policy unit for drift. All actions literal `suspend`.

`PhaseFMonitoringRecordV1` exact fields: `schema_version,monitoring_record_id,
release_record_id,claim_id,window_start,window_end,policy_sha256,measurements,
breaches,result`. Measurements contain every metric once in policy order, each
exact `metric_id,value:PhaseFMonitoringValueV1`. Breaches sorted by metric ID are
exact `metric_id,breach_code,evidence_sha256`; result `pass|suspend`.

Initial due = initial ACTIVE effective-at plus interval SI seconds. Later due =
previous accepted window-end plus interval. `now<due` current; `now>=due`
overdue. At exact due instant claim is not ACTIVE and a monitoring-breach
suspension must append. No calendar-day interpretation.

F-OD-20 is the complete storage policy. Every authority object is retained for
its `retention_seconds`, has the positive backup count, and is verified at the
backup interval. Access roles and replacement authority are exact stable IDs.
Unavailable object action is always `no_go`; URI schemes are sorted lowercase.

## 12. Trust provisioning and production order

Trust authority separates: `trust_review_sha:GIT_SHA_V1` is independently
reviewed F-IMPL-3 candidate; `trust_store_git_blob:GIT_BLOB_V1` is the blob of
`config/mhi_physical_approval_trust_store.schema1.json` at that commit;
`trust_store_sha256:SHA256_V1` hashes exact file bytes. After integration, main
must contain the same blob/bytes. Trust tag targets integrated F3 main and
records review SHA/blob/file hash. No private key, production signer, new
production CLI route, or test-to-production authority path exists.

Existing runner order is preserved: validate option relationships; strict-read/
parse/validate/hash protocol; determine physical request; load embedded trust
and fail before dataset open if unprovisioned; `ValidationInputs::read` all
scientific sources/bindings; locate pinned approval and verify file hash,
trust/root/authorities/protocol/cohort/claim/endpoint/reference/domain, both
signatures and expected record ID, attach opaque `VerifiedOwnerApproval`;
evaluate, authorize, atomically publish. Approval precedes scoring, not source
reading. macOS is the only release-gating platform.

## 13. MASTER EXTERNAL SCHEMA CATALOG

This is the single catalog occurrence for every external `PhaseF*V1` type named
by R4. `file` means complete-file SHA-256. Validators are future checker
subcommands; runtime consumes only already-authorized projected artifacts.

| Schema v1 | Exact fields/null/array rule | Identity/signature | Producer; validator; stage | Req/AC/Test/Evidence |
|---|---|---|---|---|
| `PhaseFDecisionBundleV1` | §3; no null; decision-ID order/unique | domain ID; file; unsigned | F0; `decisions`; F0 | R02/AC02/T02/EV02 |
| `PhaseFProtocolProjectionV1` | §4 plan contract, not wire | none | checker; `protocol`; F1 | R03/AC03/T03/EV03 |
| `PhaseFAuthorityEnrollmentV1` | §7; no null | domain ID; file; dual signed | authorities; `enrollment`; genesis | R07/AC07/T07/EV07 |
| `PhaseFRegistryRelationV1` | §7 exact 3; tuple order/unique | containing file | registry; `registry`; F1–F5 | R08/AC08/T08/EV08 |
| `PhaseFRegistryRecordV1` | §7; predecessor null genesis only | file; registry signed | registry; `registry`; F1–F5 | R08/AC08/T08/EV08 |
| `PhaseFRegistryHeadV1` | §7; no null | file; registry signed | registry; `head`; all | R09/AC09/T09/EV09 |
| `PhaseFObjectReferenceV1` | §8 exact 3; no null | referenced bytes | all; `retrieve`; all | R10/AC10/T10/EV10 |
| `PhaseFRetrievalVerificationV1` | §8 exact 9; no null | ID/file; unsigned operational | checker; `retrieve`; all | R10/AC10/T10/EV10 |
| `PhaseFPackageManifestV1` | §8; no null; ID arrays sorted unique | domain ID/file | data; `package`; F2 | R11/AC11/T11/EV11 |
| `PhaseFDependencyAuditV1` | §8; no null; edge tuple sorted unique | ID/file | data/science; `dependencies`; F2 | R12/AC12/T12/EV12 |
| `PhaseFPhysicalUnitLedgerV1` | §8; no null; unit-ID sorted unique | ID/file | data; `identity`; F2 | R13/AC13/T13/EV13 |
| `PhaseFPhysicalIdentityAuditV1` | §8; no null; pair sorted unique | ID/file | auditor; `identity`; F2 | R13/AC13/T13/EV13 |
| `PhaseFLocationLedgerV1` | §8; no null; location-ID sorted unique | ID/file | operations; `custody`; F2 | R14/AC14/T14/EV14 |
| `PhaseFChainOfCustodyV1` | §8; four explicit nullable fields; event tuple unique | ID/file | custodians; `custody`; F2–F4 | R14/AC14/T14/EV14 |
| `PhaseFDeviationLedgerV1` | §8; previous null genesis only; append order | revision ID/file | campaign; `deviations`; F2–F4 | R15/AC15/T15/EV15 |
| `PhaseFParameterValueV1` | §2 closed union; no null | containing file | all; relevant parser; all | R01/AC01/T01/EV01 |
| `PhaseFPowerMethodInterfaceV1` | §9; no null; ID arrays sorted unique | method ID/file | science; `power`; F1 | R16/AC16/T16/EV16 |
| `PhaseFPowerAnalysisRecordV1` | §9; no null; ID arrays sorted unique | analysis ID/file | statistician; `power`; F1 | R16/AC16/T16/EV16 |
| `PhaseFMetrologyPolicyV1` | §9; no null; endpoint-ID sorted unique | policy ID/file | metrology; `metrology`; F0/F2 | R17/AC17/T17/EV17 |
| `PhaseFReferenceSourceDescriptorV1` | §9; no null; Rust dependency canonical unique | source ID/file | metrology/data; `reference`; F2 | R18/AC18/T18/EV18 |
| `PhaseFReferenceResultV1` | §9; no null | result ID/file | laboratory; `reference`; F2 | R19/AC19/T19/EV19 |
| `PhaseFCohortLockRecordV1` | §10; no null; hashes sorted unique | domain ID/file; registry bound | data/science; `cohort`; F2 | R20/AC20/T20/EV20 |
| `PhaseFExecutionRecordV1` | §10; no null | domain ID/file; registry bound | release; `execution`; F4 | R21/AC21/T21/EV21 |
| `PhaseFReleaseRecordV1` | §10; registry pointer null only during construction | domain ID/final file; registry bound | release; `release`; F5 | R22/AC22/T22/EV22 |
| `PhaseFClaimStateRecordV1` | §10 four explicit nullable fields | domain ID/final file; registry bound | governance; `claim-state`; F5+ | R23/AC23/T23/EV23 |
| `PhaseFReinstatementApprovalV1` | §10; no null; evidence sorted unique | ID/file; five decisions | reviewers; `reinstatement`; F5+ | R24/AC24/T24/EV24 |
| `PhaseFMonitoringValueV1` | §11 closed union; no null | containing file | operations; `monitoring`; F5+ | R25/AC25/T25/EV25 |
| `PhaseFMonitoringPolicyV1` | §11; threshold unit nullable; fixed metric order | domain ID/file | F0; `monitoring`; F0/F5 | R25/AC25/T25/EV25 |
| `PhaseFMonitoringRecordV1` | §11; no null; fixed measurements/breach sort | ID/file; registry bound | operations; `monitoring`; F5+ | R26/AC26/T26/EV26 |
| `PhaseFPlanApprovalV1` | §5 exact body | tag bytes/annotated tag | bootstrap; `tags`; plan | R04/AC04/T04/EV04 |
| `PhaseFDecisionApprovalV1` | §5 exact body | same | governance; `tags`; F0 | R04/AC04/T04/EV04 |
| `PhaseFReadinessApprovalV1` | §5 exact body | same | release; `tags`; readiness | R04/AC04/T04/EV04 |
| `PhaseFAuthorityEnrollmentApprovalV1` | §5 exact body | same | governance; `tags`; enrollment | R04/AC04/T04/EV04 |
| `PhaseFTrustProvisioningApprovalV1` | §5 exact body | same | release; `tags`; F3 | R27/AC27/T27/EV27 |
| `PhaseFPhysicalReleaseApprovalV1` | §5 exact body | same | release; `tags`; F5 | R28/AC28/T28/EV28 |

All unsigned files receive independent review and exact registry relations where
§7 requires them. All unlisted optional fields, duplicate rows, and registry
bindings are forbidden. `ORPHAN_EXTERNAL_SCHEMAS=0`.

## 14. Requirement and traceability rebuild

Each requirement has one primary AC and its same-number executable KAT/evidence.

| Requirement | Normative requirement / primary AC | Test/evidence | ODs |
|---|---|---|---|
| F-R01 | Terminal primitives/numeric domains; AC01 all token/range mutations reject. | F-T01/F-EV01 | 01–21 |
| F-R02 | Closed decisions/ID; AC02 missing/extra/transformed value rejects. | F-T02/F-EV02 | 01–21 |
| F-R03 | Total isomorphic projection/frozen behavior; AC03 every field/bit equals. | F-T03/F-EV03 | 01–07 |
| F-R04 | Six exact tag lifecycles; AC04 creator/body/target/timing mutation rejects. | F-T04/F-EV04 | 13–21 |
| F-R05 | Reproducible checker provenance; AC05 two clean builds byte-match. | F-T05/F-EV05 | 15 |
| F-R06 | One read-only real/KAT parser; AC06 alternate authority path absent. | F-T06/F-EV06 | 01–21 |
| F-R07 | Complete-file enrollment bootstrap; AC07 payload/key/file substitution rejects. | F-T07/F-EV07 | 13–15 |
| F-R08 | Typed linear registry/genesis; AC08 forbidden tuple/gap/fork rejects. | F-T08/F-EV08 | 14–16 |
| F-R09 | Fresh head/currentness; AC09 stale/regressed/equivocated not ACTIVE. | F-T09/F-EV09 | 14,17–19 |
| F-R10 | Exact retrieval; AC10 URI/hash/length/unavailable mutation rejects. | F-T10/F-EV10 | 02,20 |
| F-R11 | Object/binding manifest; AC11 byte alias/unknown binding rejects. | F-T11/F-EV11 | 09–11,20 |
| F-R12 | Dependency closure; AC12 omitted/unknown dependency blocks independence. | F-T12/F-EV12 | 08,10–11 |
| F-R13 | Unit basis/audit; AC13 missing basis hash/unknown alias rejects. | F-T13/F-EV13 | 10 |
| F-R14 | Location/event custody; AC14 invalid endpoint/event transition rejects. | F-T14/F-EV14 | 10–11,18 |
| F-R15 | Append-only deviations; AC15 edited prior/incompatible resolution rejects. | F-T15/F-EV15 | 18 |
| F-R16 | Machine power interface; AC16 missing/type/unit/range/output rejects. | F-T16/F-EV16 | 12 |
| F-R17 | Exact metrology policy; AC17 incomplete/inconsistent endpoint rejects. | F-T17/F-EV17 | 05–06,11 |
| F-R18 | Runtime source projection; AC18 incomplete/unprojectable source rejects. | F-T18/F-EV18 | 05–06,11 |
| F-R19 | Result target/source projection; AC19 target/source/unit mutation rejects. | F-T19/F-EV19 | 05–06,11 |
| F-R20 | Exact cohort lock; AC20 package/revision substitution rejects. | F-T20/F-EV20 | 03,08–12,18 |
| F-R21 | Exact execution/latest deviation; AC21 mutation/order blocks F4. | F-T21/F-EV21 | 01–21 |
| F-R22 | Acyclic release identities; AC22 ID/file/registry verify independently. | F-T22/F-EV22 | 01–21 |
| F-R23 | Exact claim-state order/transitions; AC23 time/reason mutation rejects. | F-T23/F-EV23 | 16–19 |
| F-R24 | Five-GO reinstatement; AC24 missing approval/new-release trigger rejects. | F-T24/F-EV24 | 16–19 |
| F-R25 | Exact monitoring types/policy; AC25 wrong variant/threshold/unit rejects. | F-T25/F-EV25 | 19 |
| F-R26 | SI-second cadence/live state; AC26 exact due instant not ACTIVE. | F-T26/F-EV26 | 17,19–20 |
| F-R27 | Review/blob/byte trust; AC27 mismatch/private/test path rejects. | F-T27/F-EV27 | 04,13–16 |
| F-R28 | F5 five GO/zero P0-P1/macOS/final tag; AC28 omission blocks ACTIVE. | F-T28/F-EV28 | 01–21 |
| F-R29 | Runtime order/Phase-E compatibility; AC29 guards and 38/38,73/73 pass. | F-T29/F-EV29 | 01–12 |
| F-R30 | Both debts permanently close in F-IMPL-1; AC30 regression/inventory passes. | F-T30/F-EV30 | — |
| F-R31 | Cumulative counterexamples replay; AC31 every §15 result is deterministic. | F-T31/F-EV31 | 01–21 |

Counts: requirements=31, ACs=31, tests=31, evidence=31, owner decisions=21,
external schemas=35, tags=6. F-T01–F-T31/F-EV01–F-EV31 are contiguous and
mean the executable KAT/transcript for their row. Every schema, decision, tag,
and counterexample maps above or in §13/§15.

## 15. Cumulative normative counterexample catalog

Every row is independently replayable. `NO-GO` is checker failure; public claim
is NOT_ACTIVE unless explicitly ACTIVE or AUTHORITY_UNAVAILABLE. Object names
refer to the catalog. All map to F-R31/F-AC31/F-T31/F-EV31 in addition to the
listed domain requirement.

| Case | Exact input | Stage / authority object | Result; claim; class | Domain req |
|---|---|---|---|---|
| R1-CX-01 | valid software-only request | F4/runtime | software outcome; NOT_ACTIVE physical; ceiling | R03 |
| R1-CX-02 | all physical gates exact PASS | F5/release/state/tag | PASS; ACTIVE; positive control | R28 |
| R1-CX-03 | UNPROVISIONED trust | F3/runtime | hard error before dataset; trust | R27 |
| R1-CX-04 | synthetic/constructed/unknown/test origin | F2/source descriptor | NO-GO; evidence ceiling | R18 |
| R1-CX-05 | missing/wrong/duplicate authority signature/key | F3/enrollment/approval | hard error; signature | R07 |
| R1-CX-06 | wrong root/protocol/cohort/claim/endpoint/reference/domain | F4/approval | hard error; binding | R21 |
| R1-CX-07 | known holdout overlap | F4/runtime | existing exclusion/DNP; leakage | R12 |
| R1-CX-08 | unknown separation | F2/dependency audit | UNKNOWN/NO-GO; independence | R12 |
| R1-CX-09 | record/family/stratum/class below minimum | F4/runtime | existing Indeterminate; minima | R03 |
| R1-CX-10 | uncertainty exact max / next f64 above | F4/reference | eligible / excluded; boundary | R17 |
| R1-CX-11 | malformed/duplicate JSON, unsafe path, or TOCTOU | F1–F4/input | hard error/no publication; I/O | R06 |
| R1-CX-12 | identical rerun | F4/execution | byte-identical governed outputs; determinism | R21 |
| R2-CX-01 | mutate decision payload, retain ID | F0/decision bundle | semantic mismatch NO-GO; identity | R02 |
| R2-CX-02 | substitute enrollment public key | genesis/enrollment | fingerprint/file/tag mismatch; bootstrap | R07 |
| R2-CX-03 | review different enrollment file | genesis/enrollment tag | complete-file hash mismatch; identity | R07 |
| R2-CX-04 | start F-IMPL-1 before F0 | workflow/stage | FORBIDDEN; chronology | R30 |
| R2-CX-05 | authority self-appoints outside F0 | genesis/registry | NO-GO; bootstrap | R08 |
| R2-CX-06 | real manifest uses alternate human parser | F2/package | NO-GO; parser authority | R06 |
| R2-CX-07 | broken predecessor/gap/fork/rollback | registry | NO-GO/AUTHORITY_UNAVAILABLE; chain | R08 |
| R2-CX-08 | same material under two unit IDs | F2/identity | UNKNOWN alias/NO-GO; pseudoreplication | R13 |
| R2-CX-09 | undeclared power parameter | F1/power | interface mismatch NO-GO | R16 |
| R2-CX-10 | missing LOD/LOQ/QC/custody authority | F2/metrology | NO-GO; metrology | R17 |
| R2-CX-11 | alternate release serialization | F5/release | semantic/file mismatch NO-GO | R22 |
| R2-CX-12 | old tag valid, latest state suspended | public/state | NOT_ACTIVE; lifecycle | R23 |
| R2-CX-13 | temporary P2 waiver | readiness/workflow | forbidden/readiness NO-GO | R30 |
| R2-CX-14 | decision cannot project | F0/F1/projection | NO-GO/F1 forbidden | R03 |
| R2-CX-15 | rational allocation sum 1, f64 sum differs | F0/decision | PASS exact rational; boundary | R01 |
| R3-CX-01 | uppercase runtime-valid ID | F1/projection | PASS, exact case preserved | R03 |
| R3-CX-02 | pair-only A/X and B/Y authorization | F0/reference rule | unrepresentable NO-GO | R03 |
| R3-CX-03 | allow unblinded physical reference | F0/metrology | invariant NO-GO | R17 |
| R3-CX-04 | alternate mechanism critical policy | F1/projection | NO-GO | R03 |
| R3-CX-05 | duplicate predicate axis | F1/projection | NO-GO | R03 |
| R3-CX-06 | rate rule lacks exact `RateTargetV1` | F1/projection | NO-GO | R03 |
| R3-CX-07 | f64 decimal/bits disagree | any/numeric | NO-GO | R01 |
| R3-CX-08 | checker source correct, binary differs | readiness/checker | NO-GO | R05 |
| R3-CX-09 | enrollment tag valid, file unavailable | genesis/retrieval | NO-GO | R07 |
| R3-CX-10 | valid chain, resolver unavailable | public/head | AUTHORITY_UNAVAILABLE | R09 |
| R3-CX-11 | historical release, latest suspended | public/state | NOT_ACTIVE | R23 |
| R3-CX-12 | missed monitoring interval | public/monitoring | suspend; NOT_ACTIVE | R26 |
| R3-CX-13 | undocumented post-lock deviation | F4/deviation | NO-GO/suspend | R15 |
| R3-CX-14 | distinct IDs, distinctness unproved | F2/identity | UNKNOWN/no count/NO-GO | R13 |
| R3-CX-15 | QC PASS but result unprojectable | F2/reference | NO-GO | R19 |
| R3-CX-16 | release digest valid, wrong registry pointer | F5/release | NO-GO | R22 |
| R3-CX-17 | withdrawn→active | state | invalid transition/NO-GO | R23 |
| R3-CX-18 | external-valid/runtime-invalid ID projected | F0/F1 | NO-GO | R03 |
| R3-CX-19 | unsupported temperature boundary | F0/domain | NO-GO | R01 |
| R3-CX-20 | required health partition omitted | F1/projection | NO-GO | R03 |
| R3R-CX-01 | schema uses undefined `canonical_text` | plan/schema lint | NO-GO; undefined primitive | R01 |
| R3R-CX-02 | model-derived category may support physical | F0/F-OD-08 | NO-GO; ceiling | R02 |
| R3R-CX-03 | temperature lower is zero | F0/domain | NO-GO; numeric range | R01 |
| R3R-CX-04 | unauthorized principal creates plan tag | plan/tag | NO-GO; creator | R04 |
| R3R-CX-05 | build reuses dirty source checkout | readiness/checker | NO-GO; reproducibility | R05 |
| R3R-CX-06 | checker uses repository root Cargo.lock | readiness/checker | NO-GO; input set | R05 |
| R3R-CX-07 | enrollment payload hash put in tag | genesis/enrollment | NO-GO; complete-file identity | R07 |
| R3R-CX-08 | relation says `depends_on` with wrong kind | registry relation | NO-GO; typed relation | R08 |
| R3R-CX-09 | cached head used after expiry | public/head | AUTHORITY_UNAVAILABLE | R09 |
| R3R-CX-10 | same byte object duplicated by object IDs | F2/package | NO-GO; package alias | R11 |
| R3R-CX-11 | unit uses `other` basis without document hash | F2/unit ledger | NO-GO; identity wire | R13 |
| R3R-CX-12 | custody source location absent from ledger | F2/custody | NO-GO; location | R14 |
| R3R-CX-13 | old deviation event mutated in revision | F4/deviation | NO-GO; append-only | R15 |
| R3R-CX-14 | power method has prose-only range | F1/power | NO-GO; interface | R16 |
| R3R-CX-15 | suspended→active cites prose evidence | state/reinstatement | NO-GO; transition | R24 |
| R4-CX-01 | undefined primitive token attempted | plan/schema lint | NO-GO; undefined primitive | R01 |
| R4-CX-02 | runtime canonical text contains CR | F0/projection | NO-GO; text grammar | R01 |
| R4-CX-03 | external URI scheme unapproved | retrieval | NO-GO; retention/URI | R10 |
| R4-CX-04 | allocations `-0.1,0.5,0.6` | F0/split | NO-GO despite sum 1; domain | R01 |
| R4-CX-05 | relation object kind mismatches type | registry | NO-GO; typed relation | R08 |
| R4-CX-06 | same head sequence/different hash | public/head | EQUIVOCATION/NO-GO; NOT_ACTIVE | R09 |
| R4-CX-07 | registry head expired | public/head | AUTHORITY_UNAVAILABLE | R09 |
| R4-CX-08 | two builds use different lock bytes | readiness/checker | NO-GO | R05 |
| R4-CX-09 | identity basis lacks document hash | F2/unit ledger | NO-GO | R13 |
| R4-CX-10 | aliquot lacks child unit | F2/custody | NO-GO | R14 |
| R4-CX-11 | deviation revision changes old event byte | F4/deviation | NO-GO | R15 |
| R4-CX-12 | required power parameter absent | F1/power analysis | NO-GO | R16 |
| R4-CX-13 | result hypothesis differs from protocol | F2/reference result | NO-GO | R19 |
| R4-CX-14 | source descriptor incomplete | F2/reference source | NO-GO | R18 |
| R4-CX-15 | suspended→active lacks reinstatement | claim state | NO-GO; NOT_ACTIVE | R24 |
| R4-CX-16 | higher sequence, regressed effective-at | claim state | NO-GO; NOT_ACTIVE | R23 |
| R4-CX-17 | monitoring hash encoded quantity | monitoring | NO-GO; wrong variant | R25 |
| R4-CX-18 | now exactly equals due timestamp | monitoring | overdue/suspend; NOT_ACTIVE | R26 |
| R4-CX-19 | trust review SHA right, blob differs | F3/trust tag | NO-GO | R27 |
| R4-CX-20 | historical release tag, live head unavailable | public/head | AUTHORITY_UNAVAILABLE, never ACTIVE | R09 |

Historical counts are R1=12, R2=15, R3-author=20, R3-review=15, R4=20.
Where two cases are semantically identical, both IDs remain: R3R-CX-07 shares
the canonical result with R2-CX-03; R4-CX-09 with R3R-CX-11; R4-CX-11 with
R3R-CX-13; R4-CX-12 with R2-CX-09; R4-CX-15 with R3R-CX-15.

## 16. R4 remediation ledger

Author disposition is not closure; only a new reviewer may close findings.

| Finding | R4 section and exact remediation | Requirement / AC / test / evidence | Author disposition |
|---|---|---|---|
| F-PLAN-R4-P1-01 | §2 terminal primitive/text/URI/hash grammars | R01/AC01/T01/EV01 | REMEDIATED |
| F-PLAN-R4-P1-02 | §3 exact F-OD-08 enums/invariants | R02/AC02/T02/EV02 | REMEDIATED |
| F-PLAN-R4-P1-03 | §2.1 numeric ranges and §3–4 nested mapping | R01,R03/AC01,03/T01,03/EV01,03 | REMEDIATED |
| F-PLAN-R4-P1-04 | §5 bootstrap/F0 principals and push lifecycle | R04/AC04/T04/EV04 | REMEDIATED |
| F-PLAN-R4-P1-05 | §6 procedural two-directory clean builds | R05/AC05/T05/EV05 | REMEDIATED |
| F-PLAN-R4-P1-06 | §6 checker tree and checker-local lock | R05/AC05/T05/EV05 | REMEDIATED |
| F-PLAN-R4-P1-07 | §7 enrollment exact complete-file hash | R07/AC07/T07/EV07 | REMEDIATED |
| F-PLAN-R4-P1-08 | §7 typed relation/object enums and per-kind tuples | R08/AC08/T08/EV08 | REMEDIATED |
| F-PLAN-R4-P1-09 | §7 head freshness/equivocation/currentness model | R09/AC09/T09/EV09 | REMEDIATED |
| F-PLAN-R4-P1-10 | §8 reference/retrieval/object-binding manifest | R10,R11/AC10,11/T10,11/EV10,11 | REMEDIATED |
| F-PLAN-R4-P1-11 | §8 mandatory identity-basis document hash | R13/AC13/T13/EV13 | REMEDIATED |
| F-PLAN-R4-P1-12 | §8 location ledger and event constraint table | R14/AC14/T14/EV14 | REMEDIATED |
| F-PLAN-R4-P1-13 | §8 append-only deviation revisions/resolutions | R15/AC15/T15/EV15 | REMEDIATED |
| F-PLAN-R4-P1-14 | §9 closed power interface/analysis | R16/AC16/T16/EV16 | REMEDIATED |
| F-PLAN-R4-P1-15 | §9 complete F-OD-11 metrology policy | R17/AC17/T17/EV17 | REMEDIATED |
| F-PLAN-R4-P1-16 | §9 result target and source descriptor projection | R18,R19/AC18,19/T18,19/EV18,19 | REMEDIATED |
| F-PLAN-R4-P1-17 | §10 sequence/time order and five-GO reinstatement | R23,R24/AC23,24/T23,24/EV23,24 | REMEDIATED |
| F-PLAN-R4-P1-18 | §11 metric types, thresholds, SI cadence, retention | R25,R26/AC25,26/T25,26/EV25,26 | REMEDIATED |
| F-PLAN-R4-P1-19 | §12 commit/blob/file trust; §13/§15 full catalogs | R27,R31/AC27,31/T27,31/EV27,31 | REMEDIATED |

## 17. Literal tag body templates

Angle-bracket values are replaced by exactly one value of the named primitive;
all fixed GO/PASS/zero/count values are literal. These templates, not prose
abbreviations, are the six complete bodies.

```text
PhaseFPlanApprovalV1
format_version=1
plan_review_sha=<GIT_SHA_V1>
plan_sha256=<SHA256_V1>
plan_git_blob=<GIT_BLOB_V1>
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
plan_review_sha=<GIT_SHA_V1>
decision_review_sha=<GIT_SHA_V1>
decision_bundle_id=<PHASE_F_EXTERNAL_DIGEST_ID_V1>
decision_file_sha256=<SHA256_V1>
decision_git_blob=<GIT_BLOB_V1>
decision_count=21
release_coordinator_principal=<GITHUB_PRINCIPAL_V1>
governance_coordinator_principal=<GITHUB_PRINCIPAL_V1>
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
readiness_review_sha=<GIT_SHA_V1>
checker_source_review_sha=<GIT_SHA_V1>
checker_source_tree=<GIT_TREE_V1>
checker_dependency_lock_sha256=<SHA256_V1>
checker_binary_sha256=<SHA256_V1>
macos_uname=<RUNTIME_CANONICAL_TEXT_V1>
macos_arch=<RUNTIME_CANONICAL_TEXT_V1>
macos_product_version=<RUNTIME_CANONICAL_TEXT_V1>
macos_build_version=<RUNTIME_CANONICAL_TEXT_V1>
rustc_version=<RUNTIME_CANONICAL_TEXT_V1>
cargo_version=<RUNTIME_CANONICAL_TEXT_V1>
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
readiness_main_sha=<GIT_SHA_V1>
enrollment_sha256=<SHA256_V1>
owner_authority_id=<RUNTIME_STABLE_ID_V1>
registry_authority_id=<RUNTIME_STABLE_ID_V1>
owner_public_key_fingerprint=<SHA256_V1>
registry_public_key_fingerprint=<SHA256_V1>
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
enrollment_sha256=<SHA256_V1>
owner_public_key_fingerprint=<SHA256_V1>
registry_public_key_fingerprint=<SHA256_V1>
trust_root_id=<RUNTIME_STABLE_ID_V1>
trust_review_sha=<GIT_SHA_V1>
trust_store_git_blob=<GIT_BLOB_V1>
trust_store_sha256=<SHA256_V1>
f2_cohort_lock_registry_record_sha256=<SHA256_V1>
macos_uname=<RUNTIME_CANONICAL_TEXT_V1>
macos_arch=<RUNTIME_CANONICAL_TEXT_V1>
macos_product_version=<RUNTIME_CANONICAL_TEXT_V1>
macos_build_version=<RUNTIME_CANONICAL_TEXT_V1>
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
release_code_sha=<GIT_SHA_V1>
protocol_sha256=<SHA256_V1>
cohort_lock_registry_record_sha256=<SHA256_V1>
owner_approval_record_id=<PHASE_F_EXTERNAL_DIGEST_ID_V1>
owner_approval_file_sha256=<SHA256_V1>
validation_manifest_sha256=<SHA256_V1>
release_record_id=<PHASE_F_EXTERNAL_DIGEST_ID_V1>
release_file_sha256=<SHA256_V1>
release_registry_record_sha256=<SHA256_V1>
initial_claim_state_record_id=<PHASE_F_EXTERNAL_DIGEST_ID_V1>
initial_claim_state_file_sha256=<SHA256_V1>
initial_claim_state_registry_record_sha256=<SHA256_V1>
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

## 18. R4 author audit

This author audit follows a full-plan token/schema/traceability scan and is not
approval. Every counter required by the R4 brief is independently represented:

```text
SCIENTIFIC_DEFAULTS_INVENTED=0
HIDDEN_DEFAULTS=0
UNDEFINED_PRIMITIVE_TOKENS=0
PARTIAL_PRIMITIVE_GRAMMARS=0
MISSING_VALUE_GRAMMARS=0
MISSING_UNITS=0
DURATION_UNIT_AMBIGUITIES=0
SCIENTIFIC_ADMISSIBILITY_VOCABULARY_AMBIGUITIES=0
NUMERIC_REPRESENTATION_AMBIGUITIES=0
DECISION_BUNDLE_ID_AMBIGUITY=0
DECISION_TO_RUNTIME_MAPPING_AMBIGUITIES=0
UNREPRESENTABLE_DECISION_VALUES=0
HIDDEN_TRANSFORMATION_DEFAULTS=0
EXTERNAL_RUNTIME_ID_GRAMMAR_MISMATCHES=0
PROTOCOL_PROJECTION_UNBOUND_FIELDS=0
RUNTIME_FAILURE_OVERRIDE_PATHS=0
STAGE_IMPLEMENTATION_ORDER_AMBIGUITY=0
UNSPECIFIED_DURABLE_TAG_AUTHORITIES=0
TAG_BODY_GRAMMAR_AMBIGUITIES=0
TAG_CREATOR_AMBIGUITIES=0
TAG_PUSH_TIMING_AMBIGUITIES=0
CHECKER_BINARY_SOURCE_BINDING_AMBIGUITIES=0
CHECKER_BUILD_INPUT_SET_AMBIGUITIES=0
ENROLLMENT_HASH_SEMANTICS_AMBIGUITIES=0
OWNER_DECISION_BOOTSTRAP_CIRCULARITY=0
AUTHORITY_ENROLLMENT_BOOTSTRAP_CIRCULARITY=0
REGISTRY_SUBJECT_SEMANTICS_AMBIGUITIES=0
REGISTRY_RELATED_HASH_TYPE_AMBIGUITIES=0
REGISTRY_GENESIS_AMBIGUITIES=0
REGISTRY_HEAD_CURRENTNESS_AMBIGUITIES=0
EXTERNAL_AUTHORITY_ENFORCEMENT_AMBIGUITIES=0
PACKAGE_IDENTITY_AMBIGUITIES=0
PHYSICAL_PSEUDOREPLICATION_PATHS=0
PHYSICAL_IDENTITY_WIRE_AMBIGUITIES=0
CHAIN_OF_CUSTODY_WIRE_AMBIGUITIES=0
DEVIATION_AUTHORITY_AMBIGUITIES=0
POWER_METHOD_INTERFACE_AMBIGUITIES=0
METROLOGY_POLICY_AMBIGUITIES=0
METROLOGY_INTERFACE_GAPS=0
REFERENCE_RESULT_TO_RUNTIME_MAPPING_AMBIGUITIES=0
SAME_SOURCE_REFERENCE_INDEPENDENCE_PATHS=0
UNDECLARED_DEPENDENCY_INDEPENDENCE_PATHS=0
RELEASE_RECORD_SELF_REFERENCE_CYCLES=0
RELEASE_RECORD_SUBJECT_SEMANTICS_AMBIGUITIES=0
CLAIM_STATE_SELF_REFERENCE_CYCLES=0
CLAIM_STATE_CONSTRUCTION_ORDER_AMBIGUITIES=0
CLAIM_STATE_TRANSITION_AMBIGUITIES=0
CLAIM_STATE_TIME_ORDER_AMBIGUITIES=0
MONITORING_POLICY_AMBIGUITIES=0
MONITORING_RECORD_VALUE_TYPE_AMBIGUITIES=0
MONITORING_CADENCE_AMBIGUITIES=0
RETENTION_POLICY_AMBIGUITIES=0
TRUST_EMBEDDED_SOURCE_AUTHORITY_AMBIGUITIES=0
TRUST_RUNTIME_VS_EXTERNAL_LIFECYCLE_AMBIGUITIES=0
PRIVATE_KEY_REPOSITORY_PATHS=0
TEST_AUTHORITY_TO_PRODUCTION_PATHS=0
TEST_TO_PHYSICAL_EVIDENCE_PROMOTION_PATHS=0
SYNTHETIC_TO_PHYSICAL_CLAIM_PATHS=0
CONSTRUCTED_TO_PHYSICAL_CLAIM_PATHS=0
UNKNOWN_TO_PHYSICAL_CLAIM_PATHS=0
REVOKED_ROOT_PUBLIC_CLAIM_BYPASS_PATHS=0
PHYSICAL_CLAIM_BEFORE_F5_PATHS=0
FINAL_TAG_VS_LIVE_STATE_AMBIGUITIES=0
P2_TEMPORARY_DISPOSITION_AMBIGUITY=0
P2_RELEASE_BYPASS_PATHS=0
PRODUCTION_EXECUTION_ORDER_CONTRADICTIONS=0
PHASE_E_PROVISIONING_COMPATIBILITY_AMBIGUITIES=0
UNMAPPED_REQUIREMENTS=0
UNMAPPED_ACS=0
UNMAPPED_TESTS=0
UNMAPPED_EVIDENCE=0
UNMAPPED_ODS=0
ORPHAN_FIXTURES=0
ORPHAN_EXTERNAL_SCHEMAS=0
TRACEABILITY_SUBSTANCE_GAPS=0
LOST_R1_NORMATIVE_OBLIGATIONS=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
CONFLICTING_DUPLICATED_CLAUSES=0
NORMATIVE_CONTRADICTIONS=0
```

## 19. Validation, commit, push, and rereview workflow

Before/after authoring run diff check, fmt, locked check, strict all-target/all-
feature Clippy, Phase E 38/38, Phase D 73/73; verify the frozen Phase-E SHA/blob
and exact one-file diff. Create exactly one forward commit
`docs(plan): close Phase F wire and operational authority`; never amend, reset,
rebase, squash, force-push, tag, create implementation branch, start F0, create
keys/signatures/trust/evidence/records/claims, or change production behavior.

Immediately before normal push, live remote main must still equal
`e365b62586810ccfd2c8c6a9231dd970819750aa` or STOP. After push record R4 commit,
plan SHA-256/blob and require local/main/origin/live equality and clean tree. No
later commit precedes a new independent R4 rereview of full plan, delta, all
R1/R2/R3 findings, primitives, decisions, six bodies, checker, enrollment,
registry, physical/custody/deviation/power/metrology/reference/release/state/
monitoring/retention/trust contracts, all historical cases, and traceability.

`READY_FOR_PHASE_F_PLAN_APPROVAL_TAG=NO` pending fresh R4 GO.
`READY_FOR_PHASE_F_IMPLEMENTATION=NO`.
