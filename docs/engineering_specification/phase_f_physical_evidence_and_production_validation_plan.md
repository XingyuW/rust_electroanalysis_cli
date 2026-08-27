# MHI V1 Phase F — R2 Physical Evidence and Production Validation Plan

## 1. Identity, status, and authority

| Field | Value |
|---|---|
| R1 review SHA | `1e9c81b8f23318adf7a0fd46e04e61ec2e7c61bf` |
| R1 plan SHA-256 / blob | `0b10cbc98de044a2c6382e8ddc58660b3aa042e322b2cbbc8efec5bc0c86187d` / `19cf224571b701cdfa25815f95f348ed9ac87803` |
| R1 decision | independent `NO-GO`; P0=0; P1=13 |
| R2 status | forward planning remediation; independent rereview `PENDING` |
| Approval / implementation | `UNAPPROVED` / `FORBIDDEN` |
| Production physical trust | `UNPROVISIONED`; no key, signature, evidence, registry record, claim, tag, or implementation is created here |

Normative terms are MUST, MUST NOT, REQUIRED, FORBIDDEN, and NO-GO. This R2
replaces R1 in full. Only a fresh independent reviewer may mark findings CLOSED;
the author may say REMEDIATED. The immutable Phase-E integrated baseline is
`14942a30928b88f16914bf0bb103cc0c2a5bfa76`; reviewed implementation is
`5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb`. The frozen Phase-E plan remains
SHA-256 `0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`,
blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

## 2. Scope and claim ceiling

Phase F validates one exact claim/domain/cohort/protocol/package/code/trust/
approval/state on macOS. Production scientific/runtime schemas, formulas,
evaluator, and execution order remain unchanged. External Phase-F governance/
evidence contracts and a non-production checker are new. No new `electroanalysis`
route or production signer is allowed. An offline signer remains governed by
F-OD-29, using the already-closed payload contract.

Synthetic, constructed, unknown-origin, test/KAT, same-signal-derived,
Phase-C-derived, incomplete-dependency, or independence-unknown evidence cannot
support a physical claim. Fit/agreement/classification is not causal proof.
`PhysicallyValidated` is a validation result, not an ACTIVE public claim; only
F5 can activate one.

## 3. Closed chronology and gates

```text
PLAN APPROVED -> F0 OWNER DECISIONS -> FIVE-ROLE F0 REVIEW -> F0 DECISION TAG
-> F-IMPL-1 -> F1 -> F2 -> F3/F-IMPL-3 -> F4 -> F5

F_IMPL_1_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_2_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_3_BEFORE_F0_EXIT=FORBIDDEN
F_IMPL_4_BEFORE_F0_EXIT=FORBIDDEN
```

After the future plan tag, the next authorized action is exactly
`CLOSE_F0_OWNER_DECISIONS`. F0 is decide + document + independently review +
tag; no implementation precedes F0 GO.

| Stage | Allowed work | Exit |
|---|---|---|
| F0 | create only the 35-decision bundle | five fresh roles GO, P0=0, P1=0, exact decision tag |
| F-IMPL-1 | checker, schemas, tests; close both P2 debts | readiness GO/tag; otherwise F1 blocked |
| F1 | protocol, power, enrollment, registration | checker PASS and signed linear registration |
| F2 | acquire, audit, and lock real package/cohort | dependencies, units, power and cohort-lock record verified |
| F3/F-IMPL-3 | provision enrolled public roots; dual approval | security+compatibility GO and trust tag |
| F4 | one blind exact-binary run | candidate validation result only |
| F5 | five reviews, release/state records, final tag | all section-13 conditions; otherwise claim NOT ACTIVE |

F-MAINT-01 (non-UTF-8 `DIR*` early-return leak) and F-MAINT-02 (permanent
Phase-D 14/14-equivalent reproduction coverage) MUST both be CLOSED during
F-IMPL-1. No temporary disposition exists. F1–F5 cannot begin with either open.

## 4. Canonical primitives, retrieval, and failure

Every `PhaseF*V1` object uses UTF-8 JSON, no BOM, JCS canonical serialization,
duplicate keys and unknown fields forbidden, finite JSON numbers only, exact
case-sensitive names, and received bytes equal to canonical bytes. SHA-256 is
lowercase 64 hex; Ed25519 signature is lowercase 128 hex; raw public key is
lowercase 64 hex and its fingerprint is SHA-256 of the 32 raw bytes. `id` is
`[a-z0-9][a-z0-9._:-]{0,127}`. `uri` is a nonempty absolute immutable URI using
the F-OD-35 scheme. Canonical decimal strings match
`0|[1-9][0-9]*(\.[0-9]*[1-9])?`; positive decimals exclude zero. Integers are
JSON safe integers. Arrays are sorted by their stated key and unique. Required
arrays cannot be empty; optional empty arrays are stated explicitly. Any invalid
syntax, ordering, duplicate, enum, number, unit, reference, hash, identity,
signature or availability is NO-GO.

Every external reference is `{immutable_uri:uri,sha256:sha256,
byte_length:nonnegative_integer}`; rationale references omit byte length.
Campaign verification MUST materialize every object into a read-only evidence
directory; record URI; recompute length/hash; compare manifest; canonical-parse
structured data using the reviewed checker; verify signatures/registry/bindings;
record checker source SHA, binary SHA-256, rustc/cargo, platform, argv and
transcript SHA-256; and fail NO-GO if unavailable. This covers raw data,
metadata, methods, calibration/QC, custody, power, registrations, enrollment,
decisions, registry, approvals, releases, claim states, monitoring and incidents.
Mutation creates a new object and, when material, a new cohort/approval/run/release.

## 5. F0 bootstrap and `PhaseFDecisionBundleV1`

Layer 1 is repository governance only: exact Git commit bytes, five independent
reviews, and an immutable annotated tag. It is not production cryptographic
authority; Git author identity is not reviewer identity. Layer 2 is future
production authority appointed by Layer 1 and cannot authenticate itself.

Future F0 creates
`docs/engineering_specification/phase_f_f0_owner_decisions.schema1.json` with
exactly `schema_version=1`, `record_type="phase_f_f0_owner_decisions"`,
`phase_f_plan_review_sha` (40 lowercase hex), `phase_f_plan_sha256`,
`decision_bundle_id`, and `decisions`. Decisions contain exactly F-OD-01..35,
raw-ASCII sorted. Each contains exactly `decision_id,value,rationale_document,
responsible_role,decision_status`; status is `resolved`; `responsible_role` is
an `id`; rationale has exactly `immutable_uri,sha256`. The file contains no self-hash.
`decision_bundle_sha256=SHA-256(exact canonical file bytes)`.

A `range` is exactly `{lower:decimal,lower_inclusive:boolean,upper:decimal,
upper_inclusive:boolean,unit:id}`, lower < upper. `D` is exactly
`seconds|minutes|hours|days`; `A` is exactly
`hard_error|excluded|indeterminate|does_not_meet_protocol`; `U` is exactly
`sensor|membrane_batch|device|wastewater_sample|experiment|campaign|
reference_measurement|raw_acquisition`. Keys named below are exact; arrays sort
by ID and are unique; missing/extra/invalid values hard-fail.

| Decision | Exact `value` shape |
|---|---|
| F-OD-01 | `{claim_id:id,claim_wording:nonempty_string}` |
| F-OD-02 | `{analyte_ids:[id,...]}` |
| F-OD-03 | `{matrix_ids:[id,...]}` |
| F-OD-04 | `{sensor_design_ids:[id,...]}` |
| F-OD-05 | `{sensor_ids:[id,...],batch_ids:[id,...],campaign_ids:[id,...]}`; all nonempty; no predicates |
| F-OD-06 | `{temperature_ranges:[range,...]}` sorted/nonoverlapping; unit `kelvin` |
| F-OD-07 | `{mechanism_endpoint_bindings:[{endpoint_id:id,method_id:id,method_version:id,authority_id:id}]}` |
| F-OD-08 | `{health_endpoint_bindings:[{endpoint_id:id,method_id:id,method_version:id,authority_id:id}]}` |
| F-OD-09 | `{allowed_reference_authority_ids:[id,...]}` |
| F-OD-10 | `{blinding_by_endpoint:[{endpoint_id:id,required:boolean}]}` total |
| F-OD-11 | `{reference_outcome_codes:[id,...]}` |
| F-OD-12 | `{uncertainty_by_endpoint:[{endpoint_id:id,measure_id:id,unit:id,maximum_inclusive:decimal}]}` total |
| F-OD-13 | `{units_by_measure:[{measure_id:id,unit:id}]}`; no implicit conversion |
| F-OD-14 | `{comparison_by_measure:[{measure_id:id,comparator:"less_than_or_equal"}]}` total |
| F-OD-15 | `{power_analysis_id:id,power_record_sha256:sha256}` |
| F-OD-16 | `{minimum_records:[{endpoint_id:id,value:positive_integer}]}` total |
| F-OD-17 | `{minimum_families:[{endpoint_id:id,value:positive_integer}]}` total |
| F-OD-18 | `{class_minima:[{endpoint_id:id,minimum_positive:positive_integer,minimum_negative:positive_integer}]}` total |
| F-OD-19 | `{critical_policy:"all_critical_must_pass"|"no_critical_contradiction",support_levels:[{endpoint_id:id,minimum_support:decimal}],evidence_category_policy:[{category:E,may_support:boolean,may_contradict:boolean,claim_ceiling:C}]}`; support decimals are in [0,1]; exactly seven E values: `direct_physical_observation,orthogonal_physical_measurement,validated_proxy,model_derived,same_signal_derived,expert_interpretation,unavailable`; C=`physical|limited|not_assessed|unavailable|none`; model/same-signal/expert/unavailable cannot be physical |
| F-OD-20 | `{required_strata:[{stratum_id:id,selector_ids:[id,...],minimum_records:positive_integer,minimum_families:positive_integer}]}` |
| F-OD-21 | `{acceptance_rules:[{rule_id:id,endpoint_id:id,metric_id:id,comparator:"greater_than_or_equal"|"less_than_or_equal",threshold:decimal,critical:boolean}]}` |
| F-OD-22 | `{condition_actions:[{condition_code:K,action:A}]}` exactly once for each K=`missing_source|invalid_origin|incomplete_dependency|known_overlap|unknown_separation|unblinded_reference|unknown_blinding|missing_uncertainty|wrong_measure|wrong_unit|uncertainty_above_max|below_lod|between_lod_loq|failed_calibration|failed_qc|insufficient_records|insufficient_families|missing_positive_class|missing_negative_class|empty_required_stratum|critical_contradiction|domain_mismatch|protocol_mismatch`; no free text |
| F-OD-23 | `{split_unit:U,allocations:{development:decimal,validation:decimal,holdout:decimal},stratification_keys:[id,...],randomization_algorithm_id:id,seed_authority:{kind:"fixed_integer",seed:integer}|{kind:"external_record",record_sha256:sha256},split_execution_authority_id:id,lock_point:"before_outcome_access",discarded_run_action:"retain_and_exclude"|"hard_error",post_hoc_movement:"forbidden"}`; allocations sum exactly 1 |
| F-OD-24 | `{leakage_prohibitions:["outcome_guided_thresholding"|"post_lock_movement"|"holdout_retraining"|"reference_to_assessed_dependency",...]}` |
| F-OD-25 | `{unit_kinds:[U,...],endpoint_independent_kind:[{endpoint_id:id,unit_kind:U}],parent_child_rules:[{parent_kind:U,child_kind:U}],repeat_handling:"same_family_no_increment",sensor_reuse:"shared_family",batch_reuse:"shared_family",sample_reuse:"shared_family",reference_measurement_reuse:"shared_family",campaign_day_grouping:"same_campaign_day_shared_group",effective_count_authority:"phase_f_authority_checker"}` |
| F-OD-26 | `{measurand_id:id,result_unit:id,calibration_check_ids:[id,...],qc_check_ids:[id,...],checks:[{check_id:id,measure_id:id,unit:id,comparator:"less_than_or_equal"|"greater_than_or_equal",threshold:decimal,interval_value:positive_decimal,interval_unit:D,failure_action:A}],lod_loq:{mode:"not_applicable"}|{mode:"required",lod_value:decimal,lod_unit:id,loq_value:decimal,loq_unit:id,below_lod_action:A,between_lod_loq_action:A}}`; ID arrays are nonempty/disjoint and their union equals check IDs; LOQ >= LOD |
| F-OD-27 | `{authority_id:id,authority_role:"production_owner",authority_document:{immutable_uri:uri,sha256:sha256}}` |
| F-OD-28 | `{authority_id:id,authority_role:"production_registry",authority_document:{immutable_uri:uri,sha256:sha256}}` |
| F-OD-29 | `{custody_method_id:id,custody_procedure_document:{immutable_uri:uri,sha256:sha256},owner_custodian_role:id,registry_custodian_role:id,required_quorum:positive_integer,key_input_channel_id:id,network_mode:"offline"|"hsm_isolated",key_persistence_allowed:false,production_cli_access_allowed:false}` |
| F-OD-30 | `{rotation_trigger_codes:[id,...],replacement_procedure:{immutable_uri:uri,sha256:sha256},overlap_allowed:boolean,old_root_claim_action:"suspend"|"withdraw"|"expire",new_approval_required:true,new_run_required:true}`; unsupported V1 lifecycle blocks F3 pending approved V2 plan |
| F-OD-31 | `{revocation_trigger_codes:[id,...],suspension_sla_value:positive_decimal,suspension_sla_unit:D,revocation_authority_role:id,registry_state_required:"suspended"|"withdrawn",replacement_revalidation_action:"endpoint_revalidation"|"full_physical_revalidation"}` |
| F-OD-32 | `{validity:{value:positive_decimal,unit:D},periodic_review:{value:positive_decimal,unit:D}}` |
| F-OD-33 | `{trigger_actions:[{trigger:T,action:X}]}` total for section-13 triggers; X=`documentary_review|endpoint_revalidation|full_physical_revalidation|immediate_suspension_and_endpoint_revalidation|immediate_suspension_and_full_revalidation` and cannot be weaker than section 13 |
| F-OD-34 | `{outcome_wording:[{outcome:"physically_validated"|"software_validated_only"|"does_not_meet_protocol"|"indeterminate",wording:nonempty_string}]}` exactly four |
| F-OD-35 | `{immutable_uri_scheme:id,retention_duration_value:positive_decimal,retention_duration_unit:D,backup_copy_count:positive_integer,backup_verification_interval_value:positive_decimal,backup_verification_interval_unit:D,authorized_access_role_ids:[id,...],replacement_authority_role_id:id,unavailable_object_action:"no_go"}` |

After the exact file is committed/pushed, five fresh F0 reviews—SCIENTIFIC/
METROLOGY, ARCHITECTURE/DATA, SECURITY, COMPATIBILITY, OPERATIONS/GOVERNANCE—
must each return GO, with P0=0/P1=0. Only then may its tag be created. All
appointments derive from this bootstrap, never self-signature.

## 6. Immutable annotated tag authorities

All tags are annotated and immutable; the named governance/release role creates
them only after prerequisites and after the exact target is on `main`, then
pushes normally. Body bytes are UTF-8, LF only, fixed field order, one final LF,
no blank lines/trailing whitespace; values contain neither LF nor `=`. Byte
equality is checked before push. A tag is never moved, renamed, or reused.
The body type shown below is the literal first line; each listed field is one
subsequent `name=value` line in the displayed order.
The body type shown below is the literal first line; each listed field is one
subsequent `name=value` line in the displayed order.

| Exact tag | Target / prerequisite | Fixed body type and ordered fields |
|---|---|---|
| `ism-mechanism-health-v1-f-plan-approved` | reviewed R2 plan commit; fresh plan GO P0=0 P1=0 | `PhaseFPlanApprovalV1`; `format_version=1,plan_review_sha,plan_sha256,plan_git_blob,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count=0,p1_count=0,approval_decision=GO` |
| `ism-mechanism-health-v1-f-f0-decisions-approved` | reviewed decision commit; all five F0 GO | `PhaseFDecisionApprovalV1`; `format_version=1,phase_f_plan_review_sha,decision_review_sha,decision_file_sha256,decision_git_blob,decision_count=35,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count=0,p1_count=0,approval_decision=GO` |
| `ism-mechanism-health-v1-f-readiness-approved` | integrated F-IMPL-1 main SHA; checker reviewed; both debts closed | `PhaseFReadinessApprovalV1`; `format_version=1,phase_f_plan_tag,f0_decision_tag,readiness_review_sha,checker_source_sha,checker_binary_sha256,wire_schema_test_manifest_sha256,f_maint_01=CLOSED,f_maint_02=CLOSED,scientific_decision,architecture_decision,security_decision,compatibility_decision,p0_count=0,p1_count=0,approval_decision=GO` |
| `ism-mechanism-health-v1-f-trust-provisioning-approved` | integrated F-IMPL-3 main SHA; F2 lock; security+compatibility GO | `PhaseFTrustProvisioningApprovalV1`; `format_version=1,trust_review_sha,phase_f_plan_tag,f0_decision_tag,readiness_tag,trust_store_sha256,owner_authority_id,registry_authority_id,owner_public_key_fingerprint,registry_public_key_fingerprint,f2_cohort_lock_record_sha256,security_decision=GO,compatibility_decision=GO,p0_count=0,p1_count=0,macos_result=PASS,approval_decision=GO` |
| `ism-mechanism-health-v1-f-physical-validation-released` | exact final integrated main SHA used for F4/F5; section-13 GO | `PhaseFPhysicalReleaseApprovalV1`; exact body in section 13 |

## 7. One real external-authority checker

F-IMPL-1 implements `tools/phase_f_authority_checker/`, suggested binary
`phase-f-authority-check`, optionally with its own Cargo manifest. It is a
separate, non-production, read-only deterministic Rust tool: never part of the
production CLI, never a signer or physical evidence source. It owns parsing,
canonicalization, identity, signatures, retrieval, unit/dependency closure,
registry order, releases and claim states for every contract here. F-T tests use
the identical library/parser invoked by the real command. The real campaign runs
the exact independently reviewed binary and records source SHA, binary SHA-256,
rustc/cargo, platform, argv, and transcript hash. Human scientific review begins
only after checker success; no separately implemented human parser or fallback exists.

## 8. `PhaseFPackageManifestV1`, physical units, dependencies, metrology

The manifest contains exactly `schema_version=1,manifest_id,protocol_sha256,
cohort_semantic_sha256,entries,physical_units,metrology_methods`.

Entries sort by `logical_id` and contain exactly `logical_id,role,immutable_uri,
sha256,byte_length,media_type,format_or_schema,producing_authority_id,physical,
test_only,generated,direct_dependency_ids,physical_unit_ids,retention_class_id`.
IDs/URIs/hashes/lengths cannot conflict; same bytes under another path remain
one dependency identity. Dependency/unit IDs sort unique and resolve. Role is
exactly `raw_data|experiment_metadata|assessed_artifact|reference_artifact|
preprocessing_output|calibration_input|fit_output|model_derived_reference|
method_document|calibration_document|qc_document|chain_of_custody|power_analysis|
protocol|dataset|lineage_catalog|approval|registry|release|claim_state|monitoring|
incident`.

Physical-unit entries sort by `unit_id` and contain exactly `unit_id,unit_kind,
issuer_authority_id,native_identifier,identity_evidence_sha256,parent_unit_ids,
derived_from_unit_ids,acquisition_family_id`. Unit kind is `U` from section 5.
`(issuer_authority_id,unit_kind,native_identifier)` is globally unique;
identity-evidence hashes cannot map to conflicting IDs; references exist; graph
is acyclic; raw and validation records bind exact unit IDs. The checker detects
same file under new path, same hash under new logical ID, same native identity
under new unit ID, conflicting identity evidence, missing ancestry, and duplicate
family attribution. A custody audit also checks that a real unit was not issued
multiple native identities. Uncertain sameness means unknown independence/NO-GO.

Metrology entries contain exactly `method_id,method_version,measurand_id,
result_unit,uncertainty_measure_id,uncertainty_unit,uncertainty_maximum,
method_document_sha256,calibration_document_sha256,qc_document_sha256,
chain_of_custody_record_sha256,limitations_document_sha256,lod_loq`.
`lod_loq` is exactly the F-OD-26 tagged union. Result/uncertainty units are
separate; no implicit conversion or version inference exists.

Before F2 lock, checker inventory plus independent data review cross-check raw
data to assessed/reference artifacts, preprocessing, calibration, fits, model
references, sample/sensor/batch/experiment IDs and acquisition families. The
signed registered package binds the inventory. Omitted/unproved real-world
dependency is UNKNOWN SEPARATION and NO-GO; software does not claim to prove an
undeclared real-world dependency.

## 9. `PhaseFPowerAnalysisRecordV1`

The record contains exactly `schema_version=1,analysis_id,protocol_id,
endpoint_id,primary_metric_id,null_hypothesis,alternative_hypothesis,
effect_size_definition,effect_size_value,type_i_error_criterion,power_target,
cluster_unit_kind,intra_cluster_assumption,positive_class_prevalence_assumption,
negative_class_prevalence_assumption,missingness_assumption,attrition_assumption,
required_strata,minimum_eligible_records,minimum_independent_families,
minimum_positive_records,minimum_negative_records,rounding_rule,
sensitivity_analysis_cases,software_name,software_version,
software_source_sha256_or_release_id,stochastic_algorithm,random_seed,
analyst_authority_id,reviewer_authority_id,limitations,
supporting_document_sha256s`.

Hypotheses/definitions/assumptions/limitations are nonempty; dimensionless
values are canonical decimals in [0,1], other values are `{value,unit}`;
strata/cases are sorted unique closed objects with IDs and parameter maps;
minima are positive integers; rounding is exactly `ceil_to_integer`.
`stochastic_algorithm` is boolean; seed is null when false and a safe integer
when true. Supporting hashes sort unique. Exact canonical bytes are identified
in F-EV08 and a `protocol_registered` record. No production power formula is added.

## 10. Authority enrollment and linear registry

F0 closes identity/custody policy. After F0 GO and before the first signed F1
campaign record, external keys may be generated under F-OD-29; this is not
runtime trust provisioning. `PhaseFAuthorityEnrollmentV1` contains exactly
`schema_version=1,enrollment_id,owner_authority_id,registry_authority_id,
owner_public_key_ed25519_hex,registry_public_key_ed25519_hex,
owner_public_key_fingerprint,registry_public_key_fingerprint,
owner_authority_document_sha256,registry_authority_document_sha256,
key_generation_attestation_sha256,key_custody_attestation_sha256,
f0_decision_bundle_sha256`. Identities/keys/fingerprints are distinct and match
F-OD-27/28/29. Reviewed enrollment is the subject of the first
`authority_enrolled` record. Production trust remains UNPROVISIONED until F3.
For that genesis record only, the checker authenticates the registry key from
the F0 appointment plus exact five-role-reviewed enrollment bytes, then verifies
the genesis signature with it. Later records use the verified enrollment and
predecessor chain. The registry never self-appoints.
For that genesis record only, the checker authenticates the registry key from
the F0 appointment plus exact five-role-reviewed enrollment bytes, then verifies
the genesis signature with it. Later records use the verified enrollment and
predecessor chain. The registry never self-appoints.

`PhaseFRegistryRecordV1` contains exactly `schema_version=1,
registry_namespace_id,sequence_number,record_kind,predecessor_record_sha256,
subject_id,subject_sha256,related_record_sha256s,registry_authority_id,
registry_public_key_fingerprint,registry_signature_ed25519_hex`. Record kind is
exactly `protocol_registered|authority_enrolled|cohort_locked|
approval_registered|execution_registered|release_registered|
claim_state_changed|supersession_registered`. Related hashes sort unique.
Genesis has sequence 0/null predecessor. Later sequence equals predecessor+1
and predecessor hash is SHA-256 of exact canonical predecessor bytes. Signature
preimage is literal `mhi_phase_f_registry_record_v1\0` bytes plus JCS bytes
excluding only the signature. The enrolled key verifies every later record.

The checker recomputes length, hash, parse, signature, predecessor, sequence,
subject and related bindings. Unavailable/unknown predecessor, broken signature,
gap, rollback, mismatch or fork is NO-GO. V1 permits one linear chain only;
correction/supersession appends and never branches or mutates.

## 11. Release and claim-state contracts

`PhaseFReleaseRecordV1` contains exactly `schema_version=1,release_record_id,
claim_id,claim_outcome,claim_wording,target_domain,supporting_endpoint_ids,
protocol_sha256,cohort_semantic_sha256,package_manifest_sha256,
cohort_lock_registry_record_sha256,reference_method_bindings,code_git_sha,
binary_sha256,platform,trust_store_sha256,trust_root_id,owner_approval_record_id,
owner_approval_file_sha256,validation_report_id,validation_manifest_sha256,
limitations,validity_duration_value,validity_duration_unit,periodic_review_value,
periodic_review_unit,registry_record_sha256`. Endpoints/limitations sort unique.
Method bindings sort endpoint/reference and contain exactly `endpoint_id,
reference_id,method_id,method_version,authority_id,method_document_sha256`.
Platform is `macos`; duration units are `D`; outcome uses the four lowercase results.
`target_domain` contains exactly sorted nonempty `analyte_ids`, `matrix_ids`,
`sensor_design_ids`, `sensor_ids`, `batch_ids`, `campaign_ids`, and sorted
nonoverlapping `temperature_ranges` in kelvin, byte-equal to F-OD-02–06.
`target_domain` contains exactly sorted nonempty `analyte_ids`, `matrix_ids`,
`sensor_design_ids`, `sensor_ids`, `batch_ids`, `campaign_ids`, and sorted
nonoverlapping `temperature_ranges` in kelvin, byte-equal to F-OD-02–06.

To avoid self-reference, `release_payload` excludes `release_record_id` and has
`registry_record_sha256=null`. `release_record_id="sha256:" + SHA-256(
b"mhi_phase_f_release_record_v1\0" || JCS(release_payload))`. A
`release_registered` record binds that digest as subject hash; its exact hash is
then inserted as `registry_record_sha256`. The checker validates semantic ID and
final canonical file hash.

`PhaseFClaimStateRecordV1` contains exactly `schema_version=1,claim_id,
release_record_id,previous_state_record_sha256,state,reason_code,
registry_sequence_number,effective_order_record_sha256,
superseding_release_record_id,limitations,registry_authority_id,
registry_signature_ed25519_hex`. State is `active|suspended|withdrawn|expired|
superseded`. Reason is `initial_release|periodic_expiry|key_compromise|
key_revocation|trust_store_change|protocol_change|domain_change|code_change|
method_change|lineage_correction|data_integrity_incident|leakage_discovered|
operational_drift|manual_withdrawal|superseded_by_new_release`.

Initial active has null previous/superseder. Later states reference exact prior
canonical state hash; superseded requires a new release ID and other states
require null. Semantic subject hash is SHA-256 of JCS excluding
`effective_order_record_sha256` and signature; a companion `claim_state_changed`
record binds it. Its hash/sequence are inserted, then registry signs literal
`mhi_phase_f_claim_state_record_v1\0` plus JCS excluding only signature. Checker
verifies signature, subject, chain, prior state, monotonic order, latest state,
and release. Missing/unverifiable latest state means NOT ACTIVE.

An old binary may reproduce an old validation result after revocation, but cannot
activate it. Every deployment/citation runs `phase-f-authority-check claim-status`;
use requires valid release, latest active state, no later supersession, and exact
final tag binding.

## 12. Actual production execution order

The plan does not modify code. The repository runner authority is:

1. validate option path relationships;
2. strict-read protocol bytes, UTF-8 parse, validate, and hash;
3. determine whether any claim requests Physical;
4. for Physical load embedded trust; if UNPROVISIONED fail before dataset opening;
5. call `ValidationInputs::read`, strictly reading/validating dataset, lineage,
   Phase-B/C sources, references, protocol/data bindings, and source authority;
6. for Physical locate approval source; strict-read from pinned dataset-directory
   authority; verify file hash, trust/root/owner/registry/protocol/cohort/claim/
   endpoint/reference/domain bindings, both signatures and expected record ID;
   attach opaque `VerifiedOwnerApproval`;
7. evaluate, authorize publication, and atomically publish.

Owner approval MUST be verified before scientific scoring/evaluation, not before
scientific source reading. Externally it must exist and be registered before F4.
Runtime read order and campaign chronology are separate.

## 13. Implementation scopes, F4/F5, and final tag

F-IMPL-1 may modify only `tools/phase_f_authority_checker/**`,
`tests/phase_f_validation.rs`, `tests/fixtures/phase_f/**`,
`src/mhi_validation/output.rs` only for F-MAINT-01, Phase-D permanent
reproduction tests/fixtures only for F-MAINT-02, and directly required checker/
wire documentation. No physical evidence/root/signature/signer/evaluator/
Phase-B/C logic/CLI change is allowed.

F-IMPL-3 starts only after plan/F0/readiness tags, F1 GO, F2 lock, enrollment,
and debt closure. It may change only
`config/mhi_physical_approval_trust_store.schema1.json`, provisioning-specific
tests, and direct provisioning documentation/evidence. It embeds already-reviewed
public keys only; no private key or scientific change.

F-OD-33 trigger `T` is exactly `phase_b_logic|phase_c_logic|evaluator_logic|
protocol|claim_wording|reference_method|reference_authority|uncertainty_rule|
domain|sensor_design|membrane_formulation|fabrication_process|solid_contact|
reference_electrode|instrument|sampling_process|matrix_shift|temperature_range|
key_rotation|key_revocation|trust_store|approval|lineage|leakage|data_integrity|
operational_drift`. Logic/protocol/uncertainty/domain/design/formulation/process/
lineage/leakage/integrity require immediate suspension + full revalidation;
method/authority/electrode/instrument require endpoint revalidation, full if
comparability is unproved; revocation requires immediate suspension, replacement
approval and run; trust/approval/code require at least endpoint revalidation and
new approval/run; expansion requires full revalidation. Documentary review is
only for bytes outside all bound authority. Unsupported V1 lifecycle blocks F3.

F4 creates a candidate only. F5 requires five final independent GO, P0=0,
P1=0, exact macOS PASS, complete F-EV package, verified release, verified initial
ACTIVE state, and exact final tag. Only then is the claim ACTIVE.

```text
PhaseFPhysicalReleaseApprovalV1
format_version=1
phase_f_plan_review_sha=<40 lowercase hex>
f0_decision_review_sha=<40 lowercase hex>
readiness_review_sha=<40 lowercase hex>
trust_review_sha=<40 lowercase hex>
release_code_sha=<40 lowercase hex>
protocol_sha256=<sha256>
cohort_semantic_sha256=<sha256>
package_manifest_sha256=<sha256>
cohort_lock_registry_record_sha256=<sha256>
owner_approval_record_id=<id>
owner_approval_file_sha256=<sha256>
validation_manifest_sha256=<sha256>
release_record_sha256=<sha256>
initial_claim_state_record_sha256=<sha256>
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

The first line is literal. Section-6 byte grammar applies. The tag targets the
exact final integrated production main SHA used for F4/F5 and cannot precede F5 GO.

## 14. Requirements, ACs, tests, evidence, and traceability

This is the complete R2 catalog. Each requirement has exactly one primary AC;
every test/evidence/OD is mapped. Severity is review severity; P2 debts are still
stage blockers. `checker` is section 7; `external` is institutional evidence,
not production runtime.

| Requirement | Normative requirement | Primary AC | Test(s) | Evidence | ODs | Role / severity / gate / implementation |
|---|---|---|---|---|---|---|
| F-R01 | Execute exact chronology; no pre-F0 implementation. | F-AC01: stage ledger accepts only section-3 order. | F-T01 | F-EV01 | F-OD-01–35 | governance/P0/all/workflow |
| F-R02 | Use only five exact immutable tag authorities/bodies. | F-AC02: target/body/prerequisite mutations reject. | F-T02 | F-EV01 | F-OD-27–35 | governance/P0/all/checker |
| F-R03 | Separate validation result from active claim. | F-AC03: candidate without latest state/tag is NOT ACTIVE. | F-T03 | F-EV16 | F-OD-32–34 | operations/P0/F5/checker |
| F-R04 | Preserve four outcomes and physical claim ceiling. | F-AC04: prohibited evidence never activates physical claim. | F-T04 | F-EV15 | F-OD-01–26 | scientific/P0/F2–F5/runtime unchanged |
| F-R05 | Apply deterministic retrieval to every external object. | F-AC05: missing/hash/length/parser mismatch is NO-GO. | F-T05 | F-EV10 | F-OD-35 | data/P0/F1–F5/checker |
| F-R06 | Human review follows exact checker PASS. | F-AC06: no human parser/identity/order fallback. | F-T06 | F-EV04,F-EV10 | F-OD-15,F-OD-26 | all/P1/F1–F5/checker |
| F-DATA-01 | Canonical manifest binds every actual package byte. | F-AC07: real package passes reviewed parser/transcript. | F-T07 | F-EV10 | F-OD-02–14,F-OD-35 | data/P0/F2/checker |
| F-DATA-02 | Unit ledger prevents path/hash/native/evidence aliases and cycles. | F-AC08: each alias/ancestry mutation is NO-GO. | F-T08 | F-EV10 | F-OD-05,F-OD-25 | data/P0/F2/checker+COC |
| F-DATA-03 | Dependency inventory is complete across raw/assessed/reference paths. | F-AC09: omission/unproved dependency is UNKNOWN/NO-GO. | F-T09 | F-EV10 | F-OD-23–26 | data+scientific/P0/F2/checker |
| F-DATA-04 | Metrology binds exact result/uncertainty/LOD/LOQ/QC/COC. | F-AC10: unit/document/check/LOD mutation rejects. | F-T10 | F-EV09 | F-OD-07–14,F-OD-26 | metrology/P0/F1–F2/checker |
| F-DATA-05 | Power is canonical, reproducible and preregistered. | F-AC11: all fields/seed/minima bind F-EV08 bytes. | F-T11 | F-EV08 | F-OD-15–21,F-OD-23,F-OD-25 | scientific/P1/F1/checker |
| F-DATA-06 | Material mutation creates new campaign identities. | F-AC12: silent replacement/reuse rejects. | F-T12 | F-EV10 | F-OD-22,F-OD-35 | data/P0/F2–F5/checker |
| F-HOLD-01 | Split/lock is prospective; post-hoc movement forbidden. | F-AC13: bad allocation/seed/lock/movement rejects. | F-T13 | F-EV08,F-EV10 | F-OD-23–25 | scientific/P0/F1–F2/checker |
| F-HOLD-02 | Repeated IDs cannot raise independent count. | F-AC14: counts follow F-OD-25 units/families. | F-T08 | F-EV10 | F-OD-25 | data/P0/F2/checker |
| F-HOLD-03 | Known overlap fails; unknown never defaults independent. | F-AC15: shared/omitted dependency maps via F-OD-22. | F-T09 | F-EV10 | F-OD-22,F-OD-25 | scientific/P0/F2–F4/checker+runtime |
| F-TRUST-01 | F0 Git/tag bootstrap alone appoints authorities. | F-AC16: bundle/tag mutation or self-appointment rejects. | F-T14 | F-EV02 | F-OD-27–29 | governance+security/P0/F0/checker |
| F-TRUST-02 | Enrollment binds distinct identities/keys/docs/custody/F0. | F-AC17: mismatch/duplicate/key mismatch rejects. | F-T15 | F-EV03 | F-OD-27–29 | security/P0/F1/checker |
| F-TRUST-03 | Registry is one signed predecessor-bound linear chain. | F-AC18: signature/gap/predecessor/fork/rollback rejects. | F-T16 | F-EV07 | F-OD-28–31 | security+ops/P0/F1–F5/checker |
| F-TRUST-04 | Provision only enrolled public keys at F3; no private/test path. | F-AC19: hashes/fingerprints/provenance pass; forbidden paths zero. | F-T17 | F-EV11 | F-OD-27–31 | security/P0/F3/config+tests |
| F-SEC-01 | Checker is separate/read-only/pinned and shares campaign/test parser. | F-AC20: library/source/binary identities match. | F-T06,F-T18 | F-EV04 | F-OD-29 | architecture+security/P0/F-IMPL-1/checker |
| F-SEC-02 | Runtime retains actual order; approval precedes evaluation. | F-AC21: source guard/route match section 12. | F-T19 | F-EV15 | F-OD-27–29 | architecture/P0/F-IMPL-1/source guard |
| F-OPS-01 | Release has closed identity/bindings/units/registry authority. | F-AC22: canonical record/mutations pass or reject. | F-T20 | F-EV12 | F-OD-01–21,F-OD-27–35 | operations/P0/F5/checker |
| F-OPS-02 | Claim-state is signed/latest/rollback-safe. | F-AC23: rollback/unavailable/revocation returns NOT ACTIVE. | F-T03,F-T21 | F-EV13 | F-OD-30–33 | ops+security/P0/F5/checker |
| F-OPS-03 | Trigger actions meet minima; lifecycle overflow blocks F3. | F-AC24: total mapping, none weaker than section 13. | F-T21 | F-EV14 | F-OD-30–33 | all/P0/F3–F5/checker+review |
| F-OPS-04 | F5 alone activates with five GO, zero P0/P1, macOS PASS, tag. | F-AC25: omitted prerequisite prevents ACTIVE/tag. | F-T02,F-T03 | F-EV16 | F-OD-32–35 | all/P0/F5/workflow |
| F-OPS-05 | Retention/backup/access/unavailability follow F-OD-35. | F-AC26: all values/units/roles/NO-GO validate. | F-T05 | F-EV14 | F-OD-35 | governance/P1/F1–F5/external+checker |
| F-COMPAT-01 | Runtime schemas/evaluator/CLI remain unchanged. | F-AC27: diff/source guards show zero prohibited change. | F-T19 | F-EV15 | F-OD-01–26 | compatibility/P0/F-IMPL-1–F5 |
| F-COMPAT-02 | Phase E 38/38 and Phase D 73/73 remain exact. | F-AC28: both pass on exact review SHA. | F-T18 | F-EV15 | — | compatibility/P1/F-IMPL-1,F3/macOS |
| F-COMPAT-03 | Close F-MAINT-01 in F-IMPL-1 without output change. | F-AC29: regression/goldens pass before readiness tag. | F-T18 | F-EV05 | — | architecture/P2-block/F-IMPL-1/output.rs |
| F-COMPAT-04 | Close F-MAINT-02 by permanent 14/14 coverage. | F-AC30: permanent inventory/reproduction passes. | F-T18 | F-EV06 | — | compatibility/P2-block/F-IMPL-1/tests+fixtures |

Test catalog: F-T01 chronology; F-T02 five tag parsers/targets; F-T03 claim
activation/revocation/unavailability; F-T04 claim ceiling; F-T05 retrieval;
F-T06 checker identity/shared parser; F-T07 package schema; F-T08 physical-unit
dedup; F-T09 dependency closure; F-T10 metrology; F-T11 power; F-T12 mutation/
supersession; F-T13 split; F-T14 decision bundle/bootstrap; F-T15 enrollment;
F-T16 registry signature/order/fork; F-T17 trust provisioning; F-T18 complete
KAT/regression suites plus checker binary mismatch; F-T19 production-order source
guard; F-T20 release identity/mutation; F-T21 claim-state rollback/trigger minima.
F-T01–F-T21 are contiguous, unique, and coherent.

Evidence catalog: F-EV01 plan/F0 tag approvals; F-EV02 exact F0 decision approval;
F-EV03 enrollment review; F-EV04 checker source/binary/toolchain identity;
F-EV05 F-MAINT-01 closure; F-EV06 F-MAINT-02 closure; F-EV07 registry-chain
transcript; F-EV08 power/split record; F-EV09 metrology/QC/COC review; F-EV10
real package transcript; F-EV11 trust provisioning; F-EV12 release verification;
F-EV13 latest claim-state verification; F-EV14 monitoring/incident/governance;
F-EV15 exact-SHA macOS compatibility/scientific validation; F-EV16 five-role F5
approval/final tag. F-EV01–F-EV16 are contiguous and unique.

Catalog closure: requirements=30; acceptance criteria=30; tests=21; evidence
items=16; owner decisions=35. Unmapped requirements=0; unmapped ACs=0; unmapped
tests=0; unmapped evidence=0; unmapped ODs=0; orphan fixtures=0.

## 15. Adversarial authority

| Case | Deterministic result |
|---|---|
| REVIEW-CX-02 real manifest vs KAT parser | same reviewed checker parser; PASS |
| REVIEW-CX-03 authority self-appointment | only F0 Git/tag bootstrap appoints; self-appointment rejects; PASS |
| REVIEW-CX-04 F-IMPL-1 before F0 | FORBIDDEN; PASS |
| REVIEW-CX-06 old binary after revocation | result may reproduce; latest state non-active; use forbidden; PASS |
| REVIEW-CX-07 different release serialization | JCS/semantic/final hash mismatch rejects; PASS |
| REVIEW-CX-08 same material under new ID | unit/native/evidence/COC controls; uncertainty NO-GO; PASS |
| REVIEW-CX-11 sequence without registry identity | F0 appointment + enrollment + signature required; PASS |
| Broken predecessor/gap/fork/rollback | registry NO-GO |
| Enrollment key/document/F0 mismatch | NO-GO |
| Decision bundle/tag mutation | NO-GO |
| Release mutation | identity mismatch; NO-GO |
| Claim-state rollback or unavailable latest | NOT ACTIVE / public-use NO-GO |
| Checker binary SHA mismatch | campaign NO-GO |
| Same identity evidence under two unit IDs | independence unknown; NO-GO |

All R1 already-passing boundary, trust, signature, origin, lineage, minima,
uncertainty, deterministic-output, atomic-publication and outcome cases remain
required by F-T04/F-T18.

## 16. R2 remediation ledger

Rereview `OPEN` means not yet independently adjudicated.

| ID | R1 finding | R2 remediation | Acceptance / test / evidence | Author / rereview |
|---|---|---|---|---|
| F-PLAN-R2-P1-01 | F0 bootstrap undefined | §§5–6 bundle and Git/tag bootstrap | F-AC16/F-T14/F-EV02 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-02 | F-IMPL-1 timing ambiguous | §3 chronology | F-AC01/F-T01/F-EV01 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-03 | durable tags unnamed | §§6,13 exact tags/bodies | F-AC02/F-T02/F-EV01,F-EV16 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-04 | validator unspecified | §§7–8 reviewed real checker | F-AC20/F-T06,F-T07/F-EV04,F-EV10 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-05 | registry contract absent | §10 enrollment/signed linear chain | F-AC17–18/F-T15–16/F-EV03,F-EV07 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-06 | human parsing | §§4,7 one parser/retrieval | F-AC05–06/F-T05–06/F-EV04,F-EV10 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-07 | pseudoreplication | §8 unit ledger/COC | F-AC08,F-AC14/F-T08/F-EV10 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-08 | power incomplete | §9 closed wire | F-AC11/F-T11/F-EV08 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-09 | metrology gaps | §§5,8 units/LOD/LOQ/docs/COC | F-AC10/F-T10/F-EV09 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-10 | release ambiguity | §11 non-self-referential identity | F-AC22/F-T20/F-EV12 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-11 | state/revocation ambiguity | §§11,13 latest-state rule | F-AC23–25/F-T03,F-T21/F-EV13,F-EV16 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-12 | temporary P2 authority | §3 removes disposition | F-AC29–30/F-T18/F-EV05–06 | REMEDIATED/OPEN |
| F-PLAN-R2-P1-13 | runner order contradiction | §12 actual order | F-AC21/F-T19/F-EV15 | REMEDIATED/OPEN |

## 17. Internal R2 author audit

```text
SCIENTIFIC_DEFAULTS_INVENTED=0
HIDDEN_DEFAULTS=0
MISSING_VALUE_GRAMMARS=0
MISSING_UNITS=0
AMBIGUOUS_DECISION_AUTHORITIES=0
OWNER_DECISION_BOOTSTRAP_CIRCULARITY=0
EXTERNAL_AUTHORITY_ENFORCEMENT_AMBIGUITIES=0
STAGE_IMPLEMENTATION_ORDER_AMBIGUITY=0
UNSPECIFIED_DURABLE_TAG_AUTHORITIES=0
GIT_AUTHORITY_AMBIGUITIES=0
REAL_MANIFEST_VALIDATION_AUTHORITY_OPEN=0
PREREGISTRATION_ORDERING_AUTHORITY_OPEN=0
RELEASE_RECORD_AUTHORITY_OPEN=0
OPERATIONAL_STATE_AUTHORITY_OPEN=0
P2_TEMPORARY_DISPOSITION_AMBIGUITY=0
P2_PUBLIC_RELEASE_BYPASS_PATHS=0
TRUST_RUNTIME_VS_EXTERNAL_LIFECYCLE_AMBIGUITIES=0
REVOKED_ROOT_PUBLIC_CLAIM_BYPASS_PATHS=0
TEST_TO_PHYSICAL_EVIDENCE_PROMOTION_PATHS=0
PRIVATE_KEY_REPOSITORY_PATHS=0
TEST_AUTHORITY_TO_PRODUCTION_PATHS=0
SYNTHETIC_TO_PHYSICAL_CLAIM_PATHS=0
CONSTRUCTED_TO_PHYSICAL_CLAIM_PATHS=0
UNKNOWN_TO_PHYSICAL_CLAIM_PATHS=0
SAME_SOURCE_REFERENCE_INDEPENDENCE_PATHS=0
PHYSICAL_PSEUDOREPLICATION_PATHS=0
POWER_METHOD_INTERFACE_AMBIGUITIES=0
METROLOGY_INTERFACE_GAPS=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
PRODUCTION_EXECUTION_ORDER_CONTRADICTIONS=0
NORMATIVE_CONTRADICTIONS=0
```

This audit is not approval. R2 remains unapproved and implementation forbidden
pending a new full independent rereview of the plan, delta, 13 findings,
contracts, bootstrap, chronology, tags, actual runner order and adversarial cases.

## 18. Validation and forward-authoring workflow

Before/after authoring run `git diff --check`, `cargo fmt --all --check`,
`cargo check --locked`, strict all-target/all-feature Clippy, Phase E 38/38, and
Phase D 73/73. Verify frozen Phase-E hashes, exact one-file diff, contiguous
catalogs and zero unmapped/orphans. Create one forward commit
`docs(plan): close Phase F external authority contracts`; do not amend/reset/
rebase/squash/force-push, create any Phase-F tag/implementation branch, provision
trust, generate keys/signatures/evidence, or claim readiness. Immediately before
normal push, live `origin/main` must still equal the R1 SHA or STOP. After push,
freeze R2 review SHA/plan SHA-256/blob; verify local/main/origin/live equality and
clean worktree. Next action is a NEW independent R2 rereview.
`READY_FOR_PHASE_F_PLAN_APPROVAL_TAG=NO` and
`READY_FOR_PHASE_F_IMPLEMENTATION=NO` pending that GO.
