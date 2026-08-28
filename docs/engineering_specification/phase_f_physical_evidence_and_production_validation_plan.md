# MHI V1 Phase F — R6 planning-only DAG and positive-path closure

## 1. Authority, status, scope, and chronology

This document is the Phase-F R6 planning remediation of the independently
rereviewed R5 plan. It changes only this plan document. It does not create a
schema file, checker, tag, branch, key, signature, trust root, registry record,
physical evidence, monitoring record, claim, production implementation, new
scientific model, or new scientific scope.

The starting authority is exact:

| Authority | Value |
|---|---|
| R5 plan-review SHA | `49b0f92ac3a8c14f84572d6603fd97b7d223f4a0` |
| R5 plan SHA-256 | `507bca050a6c046f536f5244c9e0a0483d4a6fa04b22a004f6886d89685ddc04` |
| R5 plan Git blob | `51f19b6814e0f7ffb42bad647dabe6f804f743d4` |
| R5 independent rereview | `P0=0`, `P1=11`, `P2=0`, `P3=0`, `PLAN_DECISION=NO-GO`, `PLAN_AUTHORITY=FAIL` |
| R6 status | forward remediation; independent R6 rereview `PENDING` |
| plan approval tag | absent; must remain absent in R6 |
| implementation branch | absent; must remain absent in R6 |

The immutable Phase-E authority is not changed: integrated baseline
`14942a30928b88f16914bf0bb103cc0c2a5bfa76`, reviewed implementation
`5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb`, frozen plan SHA-256
`0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`, and
frozen plan blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

R1 was `NO-GO/P1=13`; R2 was `NO-GO/P1=10`; R3 was `NO-GO/P1=19`; R4 was
`NO-GO/P1=14`; R5 was `NO-GO/P1=11`. No rejected version is described as
approved. The exact future order remains: fresh R6 rereview, plan approval, F0, F-IMPL-1 checker and permanent
F-MAINT-01/02 closure, readiness, enrollment, genesis, F1, F2, F3, F4, and F5.
F1-F5 remain blocked until the applicable approved tags and authority objects
exist.

`F_IMPL_1_BEFORE_F0_EXIT`, `F_IMPL_2_BEFORE_F0_EXIT`,
`F_IMPL_3_BEFORE_F0_EXIT`, and `F_IMPL_4_BEFORE_F0_EXIT` are forbidden.
R5 and R6 author audits are not independent approval. No R6 approval tag,
implementation branch, F0 activity, key, signature, trust, registry, evidence,
claim, or monitoring object exists as a result of this plan edit.

R6 foundational rule: `SEMANTIC OBJECT -> COMPLETE IMMUTABLE FILE BYTES ->
COMPLETE FILE SHA-256 -> PhaseFRegistryRecordV1 ATTESTATION -> REGISTRY RECORD
SHA-256 -> OPTIONAL LATER OBJECT`. Registry membership is proven by the signed
registry record, never by a back-pointer inside the subject. Required author
invariants are `REGISTRY_BACK_POINTER_PATHS=0`, `WIRE_IDENTITY_CYCLES=0`, and
`POSITIVE_PATH_HASH_CYCLES=0`.

## 2. Closed primitive and type registry

All external JSON objects use UTF-8 bytes, RFC 8785 JCS, duplicate-member
rejection, unknown-member rejection, no omitted member, and no member typed as
an unqualified primitive. A nullable member is explicitly `T|null`; no other
optional-member convention exists. `schema_version` is the JSON integer `1`.
Arrays are sorted strictly by their stated key and are duplicate-free.

| Primitive | Exact definition |
|---|---|
| `SHA256_V1` | JSON string of exactly 64 lowercase ASCII hexadecimal characters; no prefix. |
| `GIT_SHA_V1` | exactly 40 lowercase ASCII hexadecimal characters naming a Git commit. |
| `GIT_BLOB_V1` | exactly 40 lowercase ASCII hexadecimal characters; `git cat-file -t` is `blob`. |
| `GIT_TREE_V1` | exactly 40 lowercase ASCII hexadecimal characters; `git cat-file -t` is `tree`. |
| `RUNTIME_STABLE_ID_V1` | exact existing Rust `valid_id()`: nonempty; first byte ASCII alphanumeric; later bytes ASCII alphanumeric or `._:-`; uppercase retained; no Unicode, trim, normalization, or undocumented length limit. |
| `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `sha256:` followed by 64 lowercase hexadecimal characters. |
| `RUNTIME_CANONICAL_TEXT_V1` | exact existing Rust `nonempty()`: valid UTF-8, nonempty, no U+0000/U+000D, and no Unicode whitespace as the final character of any Rust `.lines()` logical line; no normalization. |
| `RUNTIME_URI_V1` | exact existing Rust `valid_uri()`: first byte ASCII alphabetic, contains `:`, and every byte ASCII graphic `0x21..0x7e`. |
| `IMMUTABLE_EXTERNAL_URI_V1` | ASCII URI with `scheme` matching `URI_SCHEME_V1`, nonempty ASCII-graphic remainder, exact bytes, and no fragment or whitespace; only immutable object references use this primitive. |
| `LIVE_REGISTRY_HEAD_URI_V1` | ASCII bytes beginning exactly `https://`, followed by at least one ASCII graphic byte; no whitespace, no `#`, and no `@` before the first `/` after the authority. Exact bytes are F0-bound. |
| `URI_SCHEME_V1` | ASCII `[a-z][a-z0-9+.-]*`. |
| `UNIT_TEXT_V1` | nonempty UTF-8 with no NUL, CR, LF, or leading/trailing Unicode whitespace; exact case and spelling. |
| `CANONICAL_INTEGER_V1` | JSON string `0|-?[1-9][0-9]*`; `-0` forbidden. |
| `CANONICAL_UNSIGNED_INTEGER_V1` | JSON string `0|[1-9][0-9]*`. |
| `CANONICAL_POSITIVE_INTEGER_V1` | JSON string `[1-9][0-9]*`. |
| `CANONICAL_DECIMAL_V1` | JSON string `-?(0|[1-9][0-9]*)(\.[0-9]*[1-9])?`; no exponent, plus, leading/trailing zero, bare point, or `-0`. |
| `UTC_SECOND_TIMESTAMP_V1` | `YYYY-MM-DDTHH:MM:SSZ`, valid Gregorian UTC, no leap second, fraction, or alternate offset. |
| `DURATION_SECONDS_V1` | `CANONICAL_POSITIVE_INTEGER_V1` elapsed SI seconds. |
| `BOOLEAN_V1` | JSON `true` or `false`. |
| `NULL_V1` | JSON `null`, permitted only where a field definition says `T|null`. |
| `RUNTIME_F64_V1` | `{decimal:CANONICAL_DECIMAL_V1,binary64_bits_hex:<16 lowercase hex>}`; finite IEEE-754 binary64, round-nearest ties-even, independently recomputed, negative zero forbidden. |
| `PHASE_F_PARAMETER_VALUE_V1` | `{type:"integer",value:CANONICAL_INTEGER_V1}` or `{type:"decimal",value:CANONICAL_DECIMAL_V1}` or `{type:"runtime_f64",value:RUNTIME_F64_V1}` or `{type:"boolean",value:BOOLEAN_V1}` or `{type:"categorical",value:RUNTIME_CANONICAL_TEXT_V1}` or `{type:"quantity",value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1}`. |

The exact existing Rust enums/types used by this plan are
`HealthTargetV1`, `EvidenceOriginV1`, `ReferenceDependencyCompletenessV1`,
`ArtifactExperimentScope`, `ArtifactAcquisitionFamilies`,
`ReferenceDependencyV1`, `ReferenceAuthorityRuleV1`, `ReferenceEndpointV1`,
`ReferenceSourceAuthorityV1`, `ReferenceUncertaintyV1`, `CountMetricV1`,
`RateMetricV1`, `RateTargetV1`, `ComparatorV1`, `BlindingStateV1`,
`MhiValidationProtocolV1`, `ProtocolRegistrationV1`, and
`PhysicalApprovalTrustRootV1`. A field named with one of these types is exactly
that existing Rust type, not a look-alike JSON type.

```text
PHASE_F_DECISION_V1 = GO | NO-GO
PHASE_F_REVIEW_TARGET_KIND_V1 = git_commit_review | external_object_review
PHASE_F_OBJECT_KIND_V1 = decision_bundle | git_tag_message | authority_enrollment
  | registry_record | registry_head | registration_document | protocol
  | power_method_interface | power_analysis | package_manifest | dependency_audit
  | physical_unit_ledger | identity_audit | location_ledger | chain_of_custody
  | deviation_ledger | metrology_policy | metrology_check_result
  | reference_source_descriptor | reference_result | scientific_admissibility_audit
  | cohort_lock | owner_approval | execution_record | release_record | claim_state
  | reinstatement_approval | monitoring_policy | monitoring_record | incident_record
  | retention_audit | independent_review_bundle | trust_provisioning_approval
  | physical_release_approval | emergency_registry_compromise | checker_build_evidence
  | checker_readiness_evidence | f5_release_candidate
PHASE_F_REVIEW_ROLE_V1 = scientific_metrology | architecture_data | security
  | compatibility | operations_governance
PHASE_F_CHECKER_DECISION_V1 = pass | no_go | active | not_active | authority_unavailable
PHASE_F_INCIDENT_STATUS_V1 = open | contained | resolved | superseded
PHASE_F_INCIDENT_TYPE_V1 = key_compromise | key_revocation | registry_equivocation
  | data_integrity | custody_break | undeclared_dependency | monitoring_breach
  | reference_qc_breach | domain_breach | retention_failure | campaign_abandonment
  | other_registered_incident
PHASE_F_INCIDENT_ACTION_V1 = suspend | withdraw | new_release_required | campaign_no_go
PHASE_F_DEVIATION_ACTION_V1 = exclude_before_lock | resolved_no_effect | campaign_no_go
PHASE_F_DEVIATION_EVENT_V1 = reported | resolved_excluded | resolved_no_effect | campaign_no_go
PHASE_F_IDENTITY_BASIS_V1 = issuer_serial | native_specimen_id | registered_barcode
  | custody_created_child | other_registered_identity_basis
PHASE_F_PACKAGE_ROLE_V1 = raw_acquisition | derived_scientific_output | reference_result
  | reference_source_descriptor | protocol | power_analysis | metrology_check_result
  | governance_document | software_kat_support | checker_input | other_documentary
PHASE_F_RELATION_TYPE_V1 = authorized_by | depends_on | registered_after | locks
  | approves | executes | releases | changes_state_of | supersedes | references
  | incident_recorded | retention_audited | scientific_admissibility
PHASE_F_DEPENDENCY_TYPE_V1 = raw_source | sample | sensor | preprocessing | model
  | reference | derived_output
PHASE_F_CUSTODY_EVENT_V1 = acquired | transferred | aliquoted | processed | measured
  | stored | released_to_analysis | destroyed
PHASE_F_VALUE_TYPE_V1 = integer | decimal | runtime_f64 | boolean | categorical | quantity
PHASE_F_RESOLUTION_MODE_V1 = same_release_reinstatement_allowed | new_release_required
  | withdraw_only
PHASE_F_MONITORING_STATUS_METRIC_V1 = domain_compliance | reference_qc_status
  | calibration_status | reference_uncertainty_status
PHASE_F_MONITORING_NUMERIC_METRIC_V1 = sensor_drift | invalid_input_rate
  | indeterminate_rate | data_quality_insufficient_rate | exclusion_rate
PHASE_F_MONITORING_BINDING_METRIC_V1 = software_git_sha | checker_binary_sha256
  | trust_store_sha256 | trust_root_id | owner_approval_id | release_record_id
PHASE_F_MONITORING_METRIC_V1 = PHASE_F_MONITORING_STATUS_METRIC_V1
  | PHASE_F_MONITORING_NUMERIC_METRIC_V1 | PHASE_F_MONITORING_BINDING_METRIC_V1
PHASE_F_MONITORING_STATUS_VALUE_V1 = compliant | out_of_domain | unknown | pass
  | fail | within_limit | above_limit
PHASE_F_RESULT_V1 = pass | no_go
PHASE_F_CHECK_RESULT_V1 = pass | fail
PHASE_F_MONITORING_RESULT_V1 = pass | suspend
PHASE_F_IDENTITY_DETERMINATION_V1 = distinct | same | unknown
PHASE_F_REFERENCE_TYPE_V1 = mechanism | health
PHASE_F_CHECK_KIND_V1 = calibration | qc
PHASE_F_DETECTED_STAGE_V1 = f1 | f2 | f3 | f4 | f5
PHASE_F_NETWORK_MODE_V1 = offline | hsm_isolated
PHASE_F_BREACH_CODE_V1 = missing_metric | unhealthy_status | threshold_failed
  | binding_mismatch | missing_evidence
PHASE_F_CLAIM_STATE_V1 = none | active | suspended | withdrawn | expired | superseded
PHASE_F_CLAIM_REASON_V1 = initial_release | monitoring_breach | reference_qc_breach
  | domain_breach | key_compromise | key_revocation | periodic_expiry
  | manual_withdrawal | superseded_by_new_release | approved_reinstatement
PHASE_F_REGISTRY_RECORD_KIND_V1 = authority_enrolled | protocol_registered
  | power_registered | package_registered | cohort_locked | owner_approval_registered
  | execution_registered | release_registered | claim_state_changed
  | monitoring_recorded | incident_recorded | retention_audit_recorded
PHASE_F_CHECKER_BUILD_ORDINAL_V1 = "1" | "2"
PHASE_F_BUILD_RESULT_V1 = pass | no_go
PHASE_F_MAINTENANCE_STATUS_V1 = closed
PHASE_F_INCIDENT_SCOPE_KIND_V1 = release | campaign | registry_namespace
SCIENTIFIC_EVIDENCE_CATEGORY_V1 = direct_physical_observation
  | orthogonal_physical_measurement | validated_proxy | model_derived
  | same_signal_derived | expert_interpretation | unavailable
SCIENTIFIC_CLAIM_CEILING_V1 = physical | limited | not_assessed | unavailable | none
PHASE_F_TAG_NAME_V1 = ism-mechanism-health-v1-f-plan-approved
  | ism-mechanism-health-v1-f-f0-decisions-approved
  | ism-mechanism-health-v1-f-readiness-approved
  | ism-mechanism-health-v1-f-authority-enrollment-approved
  | ism-mechanism-health-v1-f-trust-provisioning-approved
  | ism-mechanism-health-v1-f-physical-validation-released
PHASE_F_LOCATION_TYPE_V1 = collection_site | laboratory | storage | instrument_station
  | transport_container | other_registered_location
```

Numeric domains are exact: temperature bands are finite positive Kelvin with
lower `<` upper and sorted nonoverlap; uncertainty and drift are finite
nonnegative; rates and prevalence are finite in `[0,1]`; probability and type-I
error are finite in `(0,1)`; allocation values are exact rational decimal
strings in `[0,1]` summing exactly to `1`; counts use the unsigned canonical
integer and minima are positive; every numeric field not listed here has an
explicit interface range and unit. Missing range or unit is NO-GO.

`PhaseFSensitivityOverrideV1` is exactly
`{parameter_id:RUNTIME_STABLE_ID_V1,value:PHASE_F_PARAMETER_VALUE_V1}`.
`PhaseFPowerOutputValueV1` is exactly
`{output_id:RUNTIME_STABLE_ID_V1,value:PHASE_F_PARAMETER_VALUE_V1}`.
`PhaseFDecisionValueV1` is exactly one of the 20 F0 value variants in §4,
selected by its fixed `decision_id`; it is not a free-form value. `PhaseFUnitRuleV1`
is exactly `{type:"none"}` or `{type:"exact",unit:UNIT_TEXT_V1}`.
`PhaseFRangeRuleV1` is exactly one of the six range variants listed in §12,
with every bound typed as `PHASE_F_PARAMETER_VALUE_V1` and every enum member
typed as `PHASE_F_PARAMETER_VALUE_V1`. `PhaseFQuantifiedUncertaintyV1` is
exactly `{type:"quantified",measure_id:RUNTIME_STABLE_ID_V1,
value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1}`.

`PHASE_F_SUBJECT_ID_V1` is a dependent closed type: the record-kind value in
`PHASE_F_REGISTRY_RECORD_KIND_V1` selects exactly the subject ID type shown in
the §9 relation table; no other subject ID is accepted. `FIXED_ORDER<T>` is a
JSON array containing exactly one member of each required `T` in the literal
order stated by its policy, with no duplicate.

The helper values used as nested fields are also closed. `PhaseFObjectDigestV1`
is exactly `{object_kind:PHASE_F_OBJECT_KIND_V1,object_sha256:SHA256_V1}`.
`PhaseFNamedDigestV1` is exactly
`{input_name:RUNTIME_CANONICAL_TEXT_V1,sha256:SHA256_V1}`. A
`PhaseFCommandV1` is either `{name:"verify",kind:PHASE_F_OBJECT_KIND_V1,
input:PATH_V1,context_dir:PATH_V1,report:PATH_V1}` or
`{name:"claim-status",release:PATH_V1,context_dir:PATH_V1,
registry_head_uri:LIVE_REGISTRY_HEAD_URI_V1,now:UTC_SECOND_TIMESTAMP_V1,
report:PATH_V1,prior_head:PATH_V1|null,registry_compromised:PATH_V1|null}`. `PATH_V1` is a valid UTF-8
path string with no NUL, CR, or LF and the §7 path resolution rules; it is not
an untyped string. `PATH_V1` may be absolute only when the command argument
explicitly permits an absolute path; relative paths have no process-CWD meaning.
`prior_head` is the explicit prior-head input and `registry_compromised` is the
explicit fail-closed emergency input. `PhaseFArgvV1` is the exact ordered JSON
array generated from the object. For `verify` it is:
`["phase-f-authority-check","verify","--kind",kind,"--input",input,
"--context-dir",context_dir,"--report",report]`. For `claim-status` it is:
`["phase-f-authority-check","claim-status","--release",release,
"--context-dir",context_dir,"--registry-head-uri",registry_head_uri,
"--now",now,"--report",report]`, followed by exactly
`["--prior-head",prior_head]` when non-null and exactly
`["--registry-compromised",registry_compromised]` when non-null, in that order.
No other pair or ordering is valid. `PhaseFEnvironmentEntryV1` is exactly
`{name:RUNTIME_STABLE_ID_V1,value:RUNTIME_CANONICAL_TEXT_V1}` and its array is
sorted by raw bytes. `PhaseFCheckerStdoutV1` is one exact literal line:
`PASS\n`, `NO-GO\n`, `ACTIVE\n`, `NOT_ACTIVE\n`, or
`AUTHORITY_UNAVAILABLE\n`. `PhaseFCheckerExitCodeV1` is the JSON integer
`0|1|2|64|70` subject to §7 command restrictions.
`PhaseFMonitoringValueV1` is exactly one of
`{type:"status",value:PHASE_F_MONITORING_STATUS_VALUE_V1}`,
`{type:"rate",value:RUNTIME_F64_V1}`,
`{type:"quantity",value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1}`,
`{type:"git_sha",value:GIT_SHA_V1}`,
`{type:"sha256",value:SHA256_V1}`, or
`{type:"stable_id",value:RUNTIME_STABLE_ID_V1}`, or
`{type:"external_digest_id",value:PHASE_F_EXTERNAL_DIGEST_ID_V1}`. A status
variant is legal only for the four status metrics; a rate only for the five
numeric rate metrics; quantity only for sensor drift; and the binding variants
only for the exact metrics named in §14.

`PhaseFUncertaintyPolicyV1` is exactly
`{measure_id:RUNTIME_STABLE_ID_V1,unit:UNIT_TEXT_V1,maximum_inclusive:RUNTIME_F64_V1}`.
`PhaseFCheckListV1` is exactly
`{check_specs:[PhaseFMetrologyCheckSpecV1],failure_action:exclude_before_lock|campaign_no_go}`,
with a sorted nonempty check-spec array. `PhaseFLODLOQPolicyV1` is the exact
tagged union defined in §13. These are nested typed values, not additional
top-level schemas or new scientific models.

`PhaseFIncidentScopeV1` is exactly one of
`{type:"release",release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1}`,
`{type:"campaign",campaign_id:RUNTIME_STABLE_ID_V1}`, or
`{type:"registry_namespace",registry_namespace_id:RUNTIME_STABLE_ID_V1}`.
`PhaseFCheckerBuildEvidenceV1`, `PhaseFCheckerReadinessEvidenceV1`, and
`PhaseFF5ReleaseCandidateV1` are complete external schemas defined in §§5 and
§7, each with one content-derived ID and complete-file hash semantics.

## 3. Content-derived external identity

`CONTENT_DERIVED_EXTERNAL_ID_V1` is the one rule for every content-derived
external ID. Let `semantic_payload` be the complete semantic object excluding
only its own semantic-ID field and a signature field added after semantic
identity. A registry record is a later external attestation and is never a
field of its subject. Then:

```text
semantic_id = "sha256:" + lowercase_hex(
    SHA256(DOMAIN_SEPARATOR_BYTES || JCS(semantic_payload))
)
```

Every domain separator is unique, literal ASCII, and ends with one NUL byte.
The registry pointer is never included in an ID. Runtime-owned IDs remain exact
runtime stable IDs and are never recomputed by this rule.

| Schema | ID field | ID type | Domain separator | Exact excluded fields | Construction stage |
|---|---|---|---|---|---|
| `PhaseFDecisionBundleV1` | `decision_bundle_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_decision_bundle_v1\0` | own ID | F0 |
| `PhaseFIndependentReviewBundleV1` | `review_bundle_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_review_bundle_v1\0` | own ID | each review gate |
| `PhaseFAuthorityEnrollmentV1` | `enrollment_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_authority_enrollment_v1\0` | own ID | enrollment |
| `PhaseFRetrievalVerificationV1` | `retrieval_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_retrieval_v1\0` | own ID | retrieval |
| `PhaseFPackageManifestV1` | `manifest_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_package_manifest_v1\0` | own ID | F2 |
| `PhaseFDependencyAuditV1` | `dependency_audit_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_dependency_audit_v1\0` | own ID | F2 |
| `PhaseFPhysicalUnitLedgerV1` | `unit_ledger_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_unit_ledger_v1\0` | own ID | F2 |
| `PhaseFPhysicalIdentityAuditV1` | `identity_audit_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_identity_audit_v1\0` | own ID | F2 |
| `PhaseFLocationLedgerV1` | `location_ledger_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_location_ledger_v1\0` | own ID | F2 |
| `PhaseFChainOfCustodyV1` | `custody_ledger_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_custody_ledger_v1\0` | own ID | F2-F4 |
| `PhaseFDeviationLedgerRevisionV1` | `revision_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_deviation_revision_v1\0` | own ID | F1-F4 |
| `PhaseFPowerMethodInterfaceV1` | `power_method_interface_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_power_method_interface_v1\0` | own ID | F1 |
| `PhaseFPowerAnalysisRecordV1` | `power_analysis_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_power_analysis_v1\0` | own ID | F1 |
| `PhaseFMetrologyPolicyV1` | `metrology_policy_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_metrology_policy_v1\0` | own ID | F0/F2 |
| `PhaseFMetrologyCheckResultV1` | `check_result_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_metrology_check_result_v1\0` | own ID | F2 |
| `PhaseFReferenceSourceDescriptorV1` | `reference_source_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_reference_source_v1\0` | own ID | F2 |
| `PhaseFReferenceResultV1` | `reference_result_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_reference_result_v1\0` | own ID | F2 |
| `PhaseFCohortLockRecordV1` | `cohort_lock_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_cohort_lock_v1\0` | own ID | F2 |
| `PhaseFExecutionRecordV1` | `execution_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_execution_v1\0` | own ID | F4 |
| `PhaseFReleaseRecordV1` | `release_record_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_release_record_v1\0` | own ID | F5 |
| `PhaseFClaimStateRecordV1` | `claim_state_record_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_claim_state_v1\0` | own ID | F5+ |
| `PhaseFReinstatementApprovalV1` | `reinstatement_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_reinstatement_v1\0` | own ID | F5+ |
| `PhaseFMonitoringPolicyV1` | `monitoring_policy_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_monitoring_policy_v1\0` | own ID | F0 |
| `PhaseFMonitoringRecordV1` | `monitoring_record_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_monitoring_record_v1\0` | own ID | F5+ |
| `PhaseFIncidentRecordV1` | `incident_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_incident_record_v1\0` | own ID | all |
| `PhaseFRetentionAuditV1` | `retention_audit_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_retention_audit_v1\0` | own ID | all |
| `PhaseFScientificAdmissibilityAuditV1` | `scientific_admissibility_audit_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_scientific_admissibility_audit_v1\0` | own ID | F2 |
| `PhaseFRegistryCompromiseEmergencyV1` | `emergency_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_registry_compromise_emergency_v1\0` | own ID | emergency |
| `PhaseFCheckerBuildEvidenceV1` | `build_evidence_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_checker_build_evidence_v1\0` | own ID | readiness |
| `PhaseFCheckerReadinessEvidenceV1` | `readiness_evidence_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_checker_readiness_evidence_v1\0` | own ID | readiness |
| `PhaseFF5ReleaseCandidateV1` | `f5_candidate_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_f5_release_candidate_v1\0` | own ID | F5 |

`SEMANTIC_ID_CONSTRUCTION_AMBIGUITIES=0` and `WIRE_IDENTITY_CYCLES=0` require
that every row above be followed literally. A complete-file hash is always
computed only after all semantic fields and any schema signature have been
inserted. `REGISTRY_BACK_POINTER_PATHS=0` means no subject object contains the
hash, sequence, or signature of a registry record attesting that subject.

## 4. F0 decision bundle and runtime projection

`PhaseFDecisionBundleV1` has exactly
`schema_version,decision_bundle_id,decisions`. Each decision has exactly
`decision_id:RUNTIME_STABLE_ID_V1,value:<one exact value below>,
decision_owner_role:PHASE_F_REVIEW_ROLE_V1,rationale_document_sha256:SHA256_V1`.
The 20 IDs F-OD-01 through F-OD-20 occur once in ascending ID order. The
bundle ID follows §3. There is no twenty-first wire or owner-decision value.

F-OD-01 through F-OD-20 retain the R4 scientific, runtime, and governance
choices; R6 only makes their construction and validation exact. There is no
non-authoritative operator coordination decision; coordination cannot enter
security, scientific, approval, tag, registry, or claim authority.

| F-OD | Exact value shape |
|---|---|
| 01 | `{protocol_id:RUNTIME_STABLE_ID_V1,title:RUNTIME_CANONICAL_TEXT_V1}` |
| 02 | `{registration_id:RUNTIME_STABLE_ID_V1,immutable_reference_uri:RUNTIME_URI_V1,document_sha256:SHA256_V1}` |
| 03 | `DomainSelectorDecisionV1`: five ordered axes each `{type:"allowed",ids:[RUNTIME_STABLE_ID_V1]}` plus `temperature:{type:"bands",bands:[{lower_kelvin_inclusive:RUNTIME_F64_V1,upper_kelvin_exclusive:RUNTIME_F64_V1}]}` |
| 04 | `{trust_root_id:RUNTIME_STABLE_ID_V1}` |
| 05 | `{mechanism_endpoints:[MechanismEndpointDecisionV1]}` |
| 06 | `{health_endpoints:[HealthEndpointDecisionV1]}` |
| 07 | `{claims:[{claim_id:RUNTIME_STABLE_ID_V1,statement:RUNTIME_CANONICAL_TEXT_V1,domain:DomainSelectorDecisionV1,supporting_endpoint_ids:[RUNTIME_STABLE_ID_V1]}]}` |
| 08 | one row per `SCIENTIFIC_EVIDENCE_CATEGORY_V1`, each `{category:SCIENTIFIC_EVIDENCE_CATEGORY_V1,may_support:BOOLEAN_V1,may_contradict:BOOLEAN_V1,claim_ceiling:SCIENTIFIC_CLAIM_CEILING_V1}`; `model_derived` and `same_signal_derived` cannot support physical claims; expert interpretation cannot support alone; unavailable is `false,false,unavailable` |
| 09 | `{split_unit:RUNTIME_STABLE_ID_V1,allocations:{development:CANONICAL_DECIMAL_V1,validation:CANONICAL_DECIMAL_V1,holdout:CANONICAL_DECIMAL_V1},stratification_keys:[RUNTIME_STABLE_ID_V1],randomization_algorithm_id:RUNTIME_STABLE_ID_V1,seed_authority:RUNTIME_STABLE_ID_V1,split_execution_authority_id:RUNTIME_STABLE_ID_V1,lock_point:"before_outcome_access",post_hoc_movement:"forbidden"}` |
| 10 | `{unit_kinds:[RUNTIME_STABLE_ID_V1],independent_kind_by_endpoint:[{endpoint_id:RUNTIME_STABLE_ID_V1,unit_kind:RUNTIME_STABLE_ID_V1}],identity_issuance_procedure_sha256:SHA256_V1,parent_child_rules:[{parent_kind:RUNTIME_STABLE_ID_V1,child_kind:RUNTIME_STABLE_ID_V1,procedure_document_sha256:SHA256_V1}],repeat_handling:"same_family_no_increment"}` |
| 11 | complete `PhaseFMetrologyPolicyV1` from §13 |
| 12 | `{power_method_id:RUNTIME_STABLE_ID_V1,power_method_version:RUNTIME_CANONICAL_TEXT_V1,power_method_interface:PhaseFObjectReferenceV1}`; no future `power_analysis_id` is permitted |
| 13 | `{authority_id:RUNTIME_STABLE_ID_V1,authority_role:"production_owner",authority_document:PhaseFObjectReferenceV1}` |
| 14 | `{authority_id:RUNTIME_STABLE_ID_V1,authority_role:"production_registry",registry_namespace_id:RUNTIME_STABLE_ID_V1,registry_head_resolver_uri:LIVE_REGISTRY_HEAD_URI_V1,registry_head_max_validity_seconds:DURATION_SECONDS_V1}` |
| 15 | `{custody_method_id:RUNTIME_STABLE_ID_V1,custody_procedure_document:PhaseFObjectReferenceV1,owner_custodian_role:RUNTIME_STABLE_ID_V1,registry_custodian_role:RUNTIME_STABLE_ID_V1,required_quorum:CANONICAL_POSITIVE_INTEGER_V1,key_input_channel_id:RUNTIME_STABLE_ID_V1,network_mode:"offline"|"hsm_isolated",key_persistence_allowed:false,production_cli_access_allowed:false}` |
| 16 | `{trigger_actions:[{trigger_code:ROTATION_TRIGGER_V1,required_state:PHASE_F_CLAIM_STATE_V1,revalidation_scope:"endpoint"|"full",new_approval_required:BOOLEAN_V1,new_run_required:BOOLEAN_V1,resolution_mode:PHASE_F_RESOLUTION_MODE_V1}],procedure_document_sha256:SHA256_V1,unsupported_lifecycle_action:"f3_no_go"}`; exactly one row for every `ROTATION_TRIGGER_V1`, no missing, duplicate, or extra row |
| 17 | `{claim_validity_seconds:DURATION_SECONDS_V1,periodic_review_seconds:DURATION_SECONDS_V1,suspension_sla_seconds:DURATION_SECONDS_V1}` |
| 18 | `{deviation_actions:[{deviation_code:RUNTIME_STABLE_ID_V1,required_action:PHASE_F_DEVIATION_ACTION_V1}]}` total over permitted deviation codes |
| 19 | complete `PhaseFMonitoringPolicyV1` from §14 |
| 20 | `{allowed_immutable_uri_schemes:[URI_SCHEME_V1],retention_seconds:DURATION_SECONDS_V1,backup_copy_count:CANONICAL_POSITIVE_INTEGER_V1,backup_verification_interval_seconds:DURATION_SECONDS_V1,authorized_access_role_ids:[RUNTIME_STABLE_ID_V1],replacement_authority_role_id:RUNTIME_STABLE_ID_V1,unavailable_object_action:"no_go"}` |

`ROTATION_TRIGGER_V1` is the closed enum
`key_rotation|key_compromise|key_revocation|method_version_change|protocol_revision|
domain_expansion|code_change|sensor_design_change|report_withdrawal|superseding_campaign`.
F-OD-16 `trigger_actions` is exactly the sorted unique ten-row set with trigger
codes `key_rotation`, `key_compromise`, `key_revocation`,
`method_version_change`, `protocol_revision`, `domain_expansion`, `code_change`,
`sensor_design_change`, `report_withdrawal`, and `superseding_campaign`; every
row has `trigger_code`, `required_state`, `revalidation_scope`,
`new_approval_required`, `new_run_required`, and `resolution_mode`, and no
default or extra trigger is legal.
`PHASE_F_RESOLUTION_MODE_V1` is
`same_release_reinstatement_allowed|new_release_required|withdraw_only`.
`ReferenceRuleDecisionV1` uses sorted nonempty
`allowed_methods:[{method_id:RUNTIME_STABLE_ID_V1,method_version:RUNTIME_CANONICAL_TEXT_V1}]`,
sorted nonempty `allowed_authority_ids:[RUNTIME_STABLE_ID_V1]`,
`blinding_rule:"require_blinded"`, and quantified uncertainty with exact
measure ID, unit, and maximum. Runtime authorizes the Cartesian product; a
pair-only rule is not representable.

F0-to-runtime projection constructs TOML and parses exactly
`MhiValidationProtocolV1::from_toml`. Every runtime field and bit is compared:
01 copies protocol ID/title; 02 copies `ProtocolRegistrationV1`; 03 maps only
to `CategoricalSelectorV1::Allowed` and `TemperatureSelectorV1::Bands`; 04
maps to `PhysicalApprovalAuthorityV1::EmbeddedTrustRoot`; 05/06 copy endpoint
fields and invariants; 07 copies claims with `requested_level=Physical`.
Statistics are the fixed `wilson_95_v1,0.95,unavailable,indeterminate,and`.
F-OD-08..21 have no runtime override. Missing, extra, defaulted, normalized,
transformed, or unrepresentable values are F0/F1 NO-GO.

The plan-only `PhaseFProtocolProjectionV1` value is exactly
`{decision_bundle_sha256:SHA256_V1,protocol_toml_sha256:SHA256_V1,
runtime_protocol:MhiValidationProtocolV1,projection_result:PHASE_F_RESULT_V1}`.
It is not a runtime schema, does not create a production route, and exists only
to make the F0-to-runtime comparison auditable.

## 5. Cryptographic primitives and review evidence

`ED25519_PUBLIC_KEY_V1` is exactly 64 lowercase hexadecimal characters decoding
to 32 bytes. The bytes must construct an `ed25519_dalek::VerifyingKey`,
round-trip to the same compressed Edwards bytes, and satisfy the exact
canonical/non-weak checks in `src/mhi_validation/approval.rs`:
`to_edwards().compress().to_bytes()` equals input and `is_weak()` is false.

`ED25519_SIGNATURE_V1` is exactly 128 lowercase hexadecimal characters decoding
to 64 bytes. All verification uses the same strict `ed25519-dalek`
`verify_strict` semantics as the production approval path. No alternate
signature algorithm exists.

`PhaseFIndependentReviewBundleV1` is exactly
`schema_version,review_bundle_id,review_target_kind,review_target_sha256,
review_target_git_sha,reviews,aggregate_p0_count,aggregate_p1_count,
aggregate_decision`. Types are respectively JSON integer, external digest ID,
`PHASE_F_REVIEW_TARGET_KIND_V1`, `SHA256_V1|null`, `GIT_SHA_V1`, exactly five review rows, canonical
unsigned counts, and `GO|NO-GO`. Each row is exactly
`{role:PHASE_F_REVIEW_ROLE_V1,review_instance_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,
decision:PHASE_F_DECISION_V1,p0_count:CANONICAL_UNSIGNED_INTEGER_V1,
p1_count:CANONICAL_UNSIGNED_INTEGER_V1,finding_ids:[RUNTIME_STABLE_ID_V1],
review_artifact_reference:PhaseFObjectReferenceV1}`. Rows are one per role in
enum order; finding IDs are sorted unique. `review_target_sha256=null` is legal
only when `review_target_kind=git_commit_review` and the target has no
independent file bytes. The aggregate is not a reviewer opinion:
`aggregate_p0_count` is the exact arithmetic sum of the five row `p0_count`
values and `aggregate_p1_count` is the exact arithmetic sum of the five row
`p1_count` values. The exact bidirectional rule is:

```text
all five row decisions == GO AND aggregate_p0_count == 0
AND aggregate_p1_count == 0  => aggregate_decision == GO
otherwise                    => aggregate_decision == NO-GO
```

Any mismatch between declared counts and sums, or between the predicate and
`aggregate_decision`, invalidates the bundle. This same rule applies to plan,
F0, readiness, enrollment, F5, trust, reinstatement, and emergency review.

### 5.1 F5 release-candidate review before initial ACTIVE

`PhaseFF5ReleaseCandidateV1` is exactly
`schema_version,f5_candidate_id,release_record_sha256,initial_claim_state_sha256,
execution_record_sha256,cohort_lock_record_sha256,owner_approval_file_sha256,
validation_manifest_sha256,trust_store_sha256,release_code_sha,
package_manifest_sha256,monitoring_policy_sha256,metrology_policy_sha256`.
All fields use `JSON_INTEGER_ONE`, `SHA256_V1`, `GIT_SHA_V1`, or
`RUNTIME_CANONICAL_TEXT_V1` only as named by their field name; its ID uses
`mhi_phase_f_f5_release_candidate_v1\0` and excludes only `f5_candidate_id`.
The candidate is complete before review and has a complete-file SHA-256.
Five independent roles review that exact candidate file hash in one
`PhaseFIndependentReviewBundleV1`. The bundle must have five GO rows, summed
P0/P1 equal to zero, and aggregate GO. The candidate review bundle hash is the
only activation authority needed to construct the initial ACTIVE state.
The F5 review bundle sets `review_target_kind=external_object_review` and
`review_target_sha256` to the complete F5 candidate file hash; its five rows
therefore review one exact candidate, not a moving release or tag.
Because this exact candidate includes `initial_claim_state_sha256`, R6 uses the
following fixed two-stage rule: that field is the complete-file SHA-256 of a
proposed initial-state template with no activation bundle and the template is
not registrable; after the bundle is complete, insert its exact hash into that
template's `activation_review_bundle_sha256`, recompute the final state ID and
complete-file hash, and register only that final state. The insertion rule is
normative text, not a candidate field, so it adds no reverse dependency or
reconstruction choice.

### 5.2 Enrollment is intentionally unsigned

`PhaseFAuthorityEnrollmentV1` is exactly
`schema_version,enrollment_id,phase_f_plan_tag,f0_decisions_tag,readiness_tag,
owner_authority_id,registry_authority_id,owner_public_key,registry_public_key,
owner_public_key_fingerprint,registry_public_key_fingerprint,
owner_authority_document,registry_authority_document,custody_policy_sha256,
created_at`. It has no `owner_signature` and no `registry_signature`; unknown
fields with either name reject. Keys use `ED25519_PUBLIC_KEY_V1`, fingerprints
are SHA-256 over decoded key bytes, references use `PhaseFObjectReferenceV1`,
and `created_at` uses `UTC_SECOND_TIMESTAMP_V1`. The enrollment ID follows §3
and its complete-file SHA-256 includes every listed field.

Enrollment authority is exactly the F0 appointment IDs, readiness-approved
checker authority, exact key bytes/fingerprints, a five-role enrollment review
bundle, and the immutable authority-enrollment approval tag. The first proof of
registry private-key possession is a valid signed registry genesis record. The
first proof of owner private-key possession is the later existing production
owner-approval verification. Enrollment is not signed a second time.

## 6. Durable tags and non-authoritative tag operators

The `git tag` executor is an `OPERATOR`, never an approval authority. Git tagger
name/email, GitHub push actor, and Git commit author are not used for validity.
A tag is valid exactly when its name, annotated type, peeled target, exact body
grammar, preceding references, review-bundle hash, approval decision, and
referenced objects verify. The referenced review bundle must independently
validate as five unique roles, aggregate P0=0, aggregate P1=0, and aggregate
GO; pusher identity is never an input.

Every body is printable ASCII plus one final LF, has the fixed first schema line,
one `name=value` line per listed field in listed order, no blank/duplicate/
unknown/trailing-whitespace line, and no LF or `=` in a value. Every approval
body has `review_bundle_sha256=<SHA256_V1>`.

| Tag / body schema | Target | Required ordered fields after `format_version=1` |
|---|---|---|
| `ism-mechanism-health-v1-f-plan-approved` / `PhaseFPlanApprovalV1` | reviewed R6 main | `plan_review_sha:GIT_SHA_V1,plan_sha256:SHA256_V1,plan_git_blob:GIT_BLOB_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-f0-decisions-approved` / `PhaseFDecisionApprovalV1` | reviewed F0 main | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,plan_review_sha:GIT_SHA_V1,decision_review_sha:GIT_SHA_V1,decision_bundle_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,decision_file_sha256:SHA256_V1,decision_git_blob:GIT_BLOB_V1,decision_count:CANONICAL_UNSIGNED_INTEGER_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-readiness-approved` / `PhaseFReadinessApprovalV1` | integrated F-IMPL-1 | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,f0_decisions_tag:PHASE_F_TAG_NAME_V1,readiness_review_sha:GIT_SHA_V1,readiness_evidence_sha256:SHA256_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-authority-enrollment-approved` / `PhaseFAuthorityEnrollmentApprovalV1` | readiness main | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,f0_decisions_tag:PHASE_F_TAG_NAME_V1,readiness_tag:PHASE_F_TAG_NAME_V1,readiness_main_sha:GIT_SHA_V1,enrollment_sha256:SHA256_V1,owner_authority_id:RUNTIME_STABLE_ID_V1,registry_authority_id:RUNTIME_STABLE_ID_V1,owner_public_key_fingerprint:SHA256_V1,registry_public_key_fingerprint:SHA256_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-trust-provisioning-approved` / `PhaseFTrustProvisioningApprovalV1` | integrated F3 main | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,f0_decisions_tag:PHASE_F_TAG_NAME_V1,readiness_tag:PHASE_F_TAG_NAME_V1,authority_enrollment_tag:PHASE_F_TAG_NAME_V1,enrollment_sha256:SHA256_V1,owner_public_key_fingerprint:SHA256_V1,registry_public_key_fingerprint:SHA256_V1,trust_root_id:RUNTIME_STABLE_ID_V1,trust_review_sha:GIT_SHA_V1,trust_store_git_blob:GIT_BLOB_V1,trust_store_sha256:SHA256_V1,f2_cohort_lock_registry_record_sha256:SHA256_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-physical-validation-released` / `PhaseFPhysicalReleaseApprovalV1` | final F4/F5 main | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,f0_decisions_tag:PHASE_F_TAG_NAME_V1,readiness_tag:PHASE_F_TAG_NAME_V1,authority_enrollment_tag:PHASE_F_TAG_NAME_V1,trust_provisioning_tag:PHASE_F_TAG_NAME_V1,release_code_sha:GIT_SHA_V1,protocol_sha256:SHA256_V1,cohort_lock_registry_record_sha256:SHA256_V1,owner_approval_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,owner_approval_file_sha256:SHA256_V1,validation_manifest_sha256:SHA256_V1,release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,release_file_sha256:SHA256_V1,release_registry_record_sha256:SHA256_V1,initial_claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,initial_claim_state_file_sha256:SHA256_V1,initial_claim_state_registry_record_sha256:SHA256_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |

All values use the named types in §§2-3. Decisions and counts are not duplicated
in any tag; the referenced bundle independently supplies five rows and the
aggregate rule in §5. Each tag is absent before creation, never moved, and
pushed only after its target and review bundle are live. No Phase-F tag is
created during R6.

## 7. Checker build and command authority

### 7.1 Build and readiness evidence

`PhaseFCheckerBuildEvidenceV1` is exactly
`schema_version,build_evidence_id,build_ordinal,checker_source_review_sha,
checker_source_tree,checker_dependency_lock_sha256,rustc_version,cargo_version,
macos_uname,macos_arch,macos_product_version,macos_build_version,environment,
build_command,build_transcript_sha256,checker_binary_sha256,
fresh_source_directory,fresh_target_directory,fresh_home,fresh_cargo_home,
cargo_home_config_absent,result`. `build_ordinal` is exactly `"1"|"2"`;
version/platform values and `build_command` are `RUNTIME_CANONICAL_TEXT_V1`;
the four `fresh_*_directory` members are `BOOLEAN_V1` freshness flags;
`environment` is a sorted unique array of `PhaseFEnvironmentEntryV1` recording
the concrete paths; `result` is `PHASE_F_BUILD_RESULT_V1`. A PASS requires all
four freshness flags to be true, `cargo_home_config_absent=true`, distinct
concrete directories in the environment, and the transcript to prove the exact
reviewed source/tree/lock/toolchain/platform inputs. The ID is content-derived
with `mhi_phase_f_checker_build_evidence_v1\0`; the complete-file hash includes
all fields.

`PhaseFCheckerReadinessEvidenceV1` is exactly
`schema_version,readiness_evidence_id,build1_sha256,build2_sha256,
checker_source_review_sha,checker_source_tree,checker_dependency_lock_sha256,
checker_binary_sha256,f_maint_01_status,f_maint_02_status,result`. The two
build hashes identify complete build-evidence files and must have different
build IDs but identical source/tree/lock/toolchain/platform values and identical
binary SHA. Both build results must be PASS, both maintenance statuses must be
`closed`, and the readiness result is PASS iff every predicate holds. Its ID
uses `mhi_phase_f_checker_readiness_evidence_v1\0` and excludes only its own
ID. The readiness review bundle targets this exact readiness file hash; the
readiness tag binds only `readiness_evidence_sha256` and its review-bundle hash.

The checker subtree is exactly `tools/phase_f_authority_checker/`, with its own
`Cargo.toml`, `Cargo.lock`, and tracked `src/**`. `checker_source_tree` is the
Git tree of exactly that subtree. `checker_dependency_lock_sha256` is SHA-256
of exact checker-local lock bytes; root lock is not an input.

Each independent build uses a fresh empty source directory, target directory,
Cargo home, and home directory under
`$HOME/Library/Caches/Codex/reviews/<repository>/<task>`. The subtree alone is
materialized into the source directory, so no parent `.cargo/config` can be
read. The process begins with `env -i`; whitelist exactly
`PATH,HOME,CARGO_HOME,CARGO_TARGET_DIR,TMPDIR,SDKROOT,
MACOSX_DEPLOYMENT_TARGET`. Every value is recorded. `RUSTFLAGS`,
`RUSTDOCFLAGS`, `CARGO_BUILD_TARGET`, `CARGO_ENCODED_RUSTFLAGS`, every
`CARGO_PROFILE_*`, `CC`, `CXX`, `AR`, `RANLIB`, `RUSTC_WRAPPER`, and
`RUSTC_WORKSPACE_WRAPPER` are unset. `CARGO_HOME` contains no `config` or
`config.toml` before build. The configured repository `CARGO_TARGET_DIR` is
preserved for ordinary validation; independent checker builds use fresh target
directories as required here.

Each build uses exact reviewed source tree, checker-local lock, rustc, Cargo,
macOS product/build/architecture, and platform. Cargo verifies registry
checksums and the transcript records package source/checksum verification where
available. Two builds have different source, target, Cargo-home, and home
directories but identical inputs and byte-identical binaries. No features,
target, linker, profile, manifest override, cache, or parent config is an
authority. Build command:

```text
cargo build --locked --release --manifest-path tools/phase_f_authority_checker/Cargo.toml
```

The checker is read-only and uses one strict parser for KAT and real input.

The only normative command forms use both the semantic `PhaseFCommandV1` and
the derived `PhaseFArgvV1` from §2:

```text
phase-f-authority-check verify --kind <OBJECT_KIND> --input <PATH> \
  --context-dir <PATH> --report <PATH>
phase-f-authority-check claim-status --release <PATH> --context-dir <PATH> \
  --registry-head-uri <LIVE_REGISTRY_HEAD_URI_V1> --now <UTC_SECOND_TIMESTAMP_V1> \
  --report <PATH> [--prior-head <PATH>] [--registry-compromised <PATH>]
```

`--kind` is exactly one `PHASE_F_OBJECT_KIND_V1`; `verify` takes exactly one
input, one context directory, and one report path. `claim-status` takes exactly
one release, context directory, live URI, UTC timestamp, and report path. Each
optional argument is one exact pair, serialized only when its corresponding
object member is non-null; `--registry-compromised` is permitted only for §15's
exact emergency schema.
Paths are UTF-8, relative paths resolve only from the named context directory,
absolute paths are permitted only when explicitly passed, and symlinks,
directories in file position, traversal, and unsafe files reject. The checker
writes only the requested report.

Stdout is exactly one line: `PASS\n` or `NO-GO\n` for `verify`, and
`ACTIVE\n`, `NOT_ACTIVE\n`, or `AUTHORITY_UNAVAILABLE\n` for `claim-status`.
For exits 0/1/2, stderr is sorted lines of
`diagnostic_code=<DIAGNOSTIC_CODE_V1>\n` and the report must agree with stdout
and exit code. Exit codes are exact: `0` PASS/ACTIVE, `1` validated
NO-GO/NOT_ACTIVE, and `2` AUTHORITY_UNAVAILABLE for claim-status only. Exit
`64` is a usage error with empty stdout, exactly `USAGE_ERROR\n` on stderr, and
no valid report. Exit `70` is an internal failure with empty stdout, exactly
`INTERNAL_ERROR\n` on stderr, and no valid report. `USAGE_CODE_V1`, 65, and 66
do not exist.

`PhaseFCheckerReportV1` is exactly
`schema_version,checker_binary_sha256,command,argv,input_sha256s,decision,
diagnostic_codes,stdout,exit_code`. `command:PhaseFCommandV1` is the semantic
object and `argv:PhaseFArgvV1` is its deterministic ordered array.
`input_sha256s` is a sorted array of
`{input_name:RUNTIME_CANONICAL_TEXT_V1,sha256:SHA256_V1}`; `stdout` is
`PhaseFCheckerStdoutV1` and `exit_code` is `PhaseFCheckerExitCodeV1`. The
decision uses `PHASE_F_CHECKER_DECISION_V1`; diagnostic codes are closed:
`MalformedJson,UnknownMember,DuplicateMember,InvalidType,InvalidSemanticId,
InvalidCompleteFileHash,InvalidSignature,InvalidRelation,InvalidTransition,
MissingInput,UnsafePath,ResolverUnavailable,HeadExpired,RegistryEquivocation,
RegistryRegression,MonitoringBreach,RetentionFailure,ProjectionMismatch,
BuildInputMismatch,CommandResultMismatch`. Report decision, stdout, argv,
command, and exit code must agree; no report is valid for exit 64 or 70.

## 8. Live registry resolver and cryptographic wire

`registry_head_resolver_uri` is `LIVE_REGISTRY_HEAD_URI_V1`, never
`IMMUTABLE_EXTERNAL_URI_V1`. Its exact bytes are F0-bound, but no response hash
is bound to that URI. HTTPS is transport only. Normal claim-status authority is
the live HTTPS response, a signed fresh head, and the complete verified chain
from genesis through that head; the head `valid_until` is mandatory. No hidden
watermark, persisted current sequence, cache, or implicit prior head exists.
Authority is the verified
`PhaseFRegistryHeadV1`, including namespace, registry ID, key fingerprint,
freshness interval, and every verified record in the chain.

`PhaseFRegistryRecordV1` is exactly
`schema_version,registry_namespace_id,registry_authority_id,sequence,
predecessor_record_sha256,record_kind,subject_id,subject_sha256,relations,
created_at,registry_key_fingerprint,signature`. Sequence is canonical unsigned,
predecessor is `SHA256_V1|null`, record kind is `PHASE_F_REGISTRY_RECORD_KIND_V1`, subject types and hash
meaning are §9, relations are sorted `PhaseFRegistryRelationV1`, time is UTC,
key fingerprint is SHA-256, and signature is `ED25519_SIGNATURE_V1`.

The exact signing payload is the record object excluding only `signature`.
Signing bytes are:

```text
ASCII("mhi_phase_f_registry_record_v1") || byte(0) || JCS(signing_payload)
```

Verification uses the approved registry public key and `verify_strict`; mismatch
is NO-GO. Complete-file SHA-256 includes the signature.

`PhaseFRegistryHeadV1` is exactly
`schema_version,registry_namespace_id,registry_authority_id,sequence,
registry_record_sha256,issued_at,valid_until,registry_key_fingerprint,signature`.
Head signing bytes are:

```text
ASCII("mhi_phase_f_registry_head_v1") || byte(0) || JCS(head_without_signature)
```

Head signatures use `verify_strict`; complete-file SHA-256 includes signature.
`issued_at<=now<valid_until`, and the interval is no greater than F0's maximum.
Same sequence/digest is the same head. Same sequence/different digest is
`REGISTRY_EQUIVOCATION` and NOT_ACTIVE. If `prior_head=null`, no cross-invocation
regression or unseen-equivocation claim is made. If `prior_head` is supplied,
its complete file must validate: lower sequence is AUTHORITY_UNAVAILABLE;
same sequence with different record hash is REGISTRY_EQUIVOCATION/NOT_ACTIVE;
same sequence and same hash is the same head; higher sequence requires every
intervening registry record. Unavailable resolver, expired head, bad signature,
missing chain object, or equivocation never uses a cache as ACTIVE authority.

## 9. Registry object kinds, hashes, and relations

The exact object-kind set is the enum in §2. The hash meaning is exhaustive:

| Object kind | `object_sha256` means |
|---|---|
| `decision_bundle` | SHA-256 of complete canonical `PhaseFDecisionBundleV1` bytes |
| `git_tag_message` | SHA-256 of exact annotated-tag message bytes from first body byte through one final LF |
| `authority_enrollment` | SHA-256 of complete canonical unsigned enrollment bytes |
| `registry_record` | SHA-256 of complete canonical signed registry-record bytes |
| `registry_head` | SHA-256 of complete canonical signed head bytes |
| `registration_document` | SHA-256 of exact original registered-document bytes |
| `protocol` | SHA-256 of exact original protocol TOML bytes |
| `power_method_interface` | SHA-256 of complete canonical power-interface bytes |
| `power_analysis` | SHA-256 of complete canonical power-analysis bytes |
| `package_manifest` | SHA-256 of complete canonical package-manifest bytes |
| `dependency_audit` | SHA-256 of complete canonical dependency-audit bytes |
| `physical_unit_ledger` | SHA-256 of complete canonical unit-ledger bytes |
| `identity_audit` | SHA-256 of complete canonical identity-audit bytes |
| `location_ledger` | SHA-256 of complete canonical location-ledger bytes |
| `chain_of_custody` | SHA-256 of complete canonical custody-ledger bytes |
| `deviation_ledger` | SHA-256 of complete canonical latest deviation-revision bytes |
| `metrology_policy` | SHA-256 of complete canonical metrology-policy bytes |
| `metrology_check_result` | SHA-256 of complete canonical check-result bytes |
| `reference_source_descriptor` | SHA-256 of complete canonical source-descriptor bytes |
| `reference_result` | SHA-256 of complete canonical reference-result bytes |
| `scientific_admissibility_audit` | SHA-256 of complete canonical scientific-audit bytes |
| `cohort_lock` | SHA-256 of complete canonical cohort-lock bytes |
| `owner_approval` | SHA-256 of complete canonical owner-approval bytes |
| `execution_record` | SHA-256 of complete canonical execution bytes |
| `release_record` | SHA-256 of complete canonical release-record bytes |
| `claim_state` | SHA-256 of complete canonical claim-state bytes |
| `reinstatement_approval` | SHA-256 of complete canonical reinstatement bytes |
| `monitoring_policy` | SHA-256 of complete canonical monitoring-policy bytes |
| `monitoring_record` | SHA-256 of complete canonical monitoring-record bytes |
| `incident_record` | SHA-256 of complete canonical incident bytes |
| `retention_audit` | SHA-256 of complete canonical retention-audit bytes |
| `independent_review_bundle` | SHA-256 of complete canonical review-bundle bytes |
| `trust_provisioning_approval` | SHA-256 of complete canonical trust-provisioning approval bytes |
| `physical_release_approval` | SHA-256 of complete canonical physical-release approval bytes |
| `emergency_registry_compromise` | SHA-256 of complete canonical emergency bytes |
| `checker_build_evidence` | SHA-256 of complete canonical `PhaseFCheckerBuildEvidenceV1` bytes |
| `checker_readiness_evidence` | SHA-256 of complete canonical `PhaseFCheckerReadinessEvidenceV1` bytes |
| `f5_release_candidate` | SHA-256 of complete canonical `PhaseFF5ReleaseCandidateV1` bytes |

`PhaseFRegistryRelationV1` is exactly
`{relation_type:PHASE_F_RELATION_TYPE_V1,object_kind:PHASE_F_OBJECT_KIND_V1,
object_sha256:SHA256_V1}`. Relation type is
`authorized_by|depends_on|registered_after|locks|approves|executes|releases|
changes_state_of|supersedes|references|incident_recorded|retention_audited|
scientific_admissibility`. Every relation is validated against kind and hash
meaning; a bare hash never supplies a subject.

Relations are canonicalized by raw ASCII lexical ascending tuple
`(relation_type literal bytes, object_kind literal bytes, object_sha256 literal
bytes)`. Duplicate tuples are forbidden. Enum declaration order and insertion
order have no effect. Every package relation below includes its explicit
relation type; prose cannot supply a missing relation.

For every ordinary external JSON subject, `subject_sha256` is the SHA-256 of
the complete canonical subject-file bytes. The exhaustive exceptions are
`protocol` (exact original TOML bytes), `git_tag_message` (exact annotated-tag
message bytes), and `owner_approval` only when it is an existing certified
owner-approval JSON whose documented certification defines its original bytes.
No semantic digest is ever substituted for a complete-file subject hash.

| Record kind | Subject ID / hash | Required relations | Optional relations; all others forbidden |
|---|---|---|---|
| `authority_enrolled` | `enrollment_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `authority_enrollment` | `authorized_by+decision_bundle`; `references+git_tag_message` for plan/F0/readiness/enrollment tags | none |
| `protocol_registered` | `protocol_id:RUNTIME_STABLE_ID_V1` / `protocol` | `authorized_by+decision_bundle`; `depends_on+registration_document` | none |
| `power_registered` | `power_analysis_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `power_analysis` | `authorized_by+decision_bundle`; `depends_on+power_method_interface`; `depends_on+protocol` | none |
| `package_registered` | `manifest_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `package_manifest` | exactly once each: `depends_on+dependency_audit`, `depends_on+physical_unit_ledger`, `depends_on+identity_audit`, `depends_on+location_ledger`, `depends_on+chain_of_custody`, `depends_on+deviation_ledger`, `depends_on+metrology_policy`, `depends_on+scientific_admissibility_audit`; at least once each: `references+reference_result`, `references+reference_source_descriptor` | none; `locks`, `releases`, and untyped relations forbidden |
| `cohort_locked` | `cohort_lock_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `cohort_lock` | `locks+package_manifest`; `depends_on+protocol`; `depends_on+power_analysis`; `depends_on+deviation_ledger`; `depends_on+scientific_admissibility_audit` | none |
| `owner_approval_registered` | `approval_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `owner_approval` | `approves+cohort_lock`; `authorized_by+authority_enrollment` | none |
| `execution_registered` | `execution_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `execution_record` | `executes+cohort_lock`; `authorized_by+owner_approval`; `depends_on+deviation_ledger`; `depends_on+protocol` | none |
| `release_registered` | `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `release_record` | `releases+execution_record`; `authorized_by+owner_approval`; `depends_on+monitoring_policy`; `depends_on+metrology_policy` | none |
| `claim_state_changed` | `claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `claim_state` | `changes_state_of+release_record` | one prior-state `registered_after+claim_state`; one reinstatement dependency; one superseding-release relation only when applicable |
| `monitoring_recorded` | `monitoring_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `monitoring_record` | `references+release_record`; `depends_on+monitoring_policy` | one prior `registered_after+monitoring_record` after first |
| `incident_recorded` | `incident_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `incident_record` | for release scope exactly `incident_recorded+release_record`; for campaign or registry-namespace scope no release relation is permitted | references only to listed affected evidence; campaign closure is authorized by its independently reviewed incident |
| `retention_audit_recorded` | `retention_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `retention_audit` | exactly once: `references+release_record`; exactly once: `authorized_by+decision_bundle` | exactly one `registered_after+retention_audit` for every audit after the first; no per-checked-object `references+package_manifest` relation |

Genesis is sequence `0` with null predecessor. Later records are prior sequence
plus one and prior complete-file hash. No gap, fork, or rollback is legal.

## 10. Retrieval and package construction

`PhaseFObjectReferenceV1` is exactly
`{immutable_uri:IMMUTABLE_EXTERNAL_URI_V1,sha256:SHA256_V1,
byte_length:CANONICAL_UNSIGNED_INTEGER_V1}`. Retrieval verifies URI bytes,
length, and hash. `PhaseFRetrievalVerificationV1` is exactly
`schema_version,retrieval_id,object_reference,retrieved_sha256,
retrieved_byte_length,checker_binary_sha256,checker_source_review_sha,
retrieved_at,verification_result`, with §3 ID and `pass|no_go`. No unavailable
object receives a fabricated pass.

`PhaseFPackageManifestV1` has exactly `schema_version,manifest_id,objects,bindings`.
Each object is exactly
`{object_id:RUNTIME_STABLE_ID_V1,object_reference:PhaseFObjectReferenceV1,
media_type:RUNTIME_CANONICAL_TEXT_V1,format_or_schema:RUNTIME_CANONICAL_TEXT_V1,
producing_authority_id:RUNTIME_STABLE_ID_V1,physical:BOOLEAN_V1,
test_only:BOOLEAN_V1,generated:BOOLEAN_V1,retention_class_id:RUNTIME_STABLE_ID_V1}`.
Each binding is exactly
`{binding_id:RUNTIME_STABLE_ID_V1,role:PHASE_F_PACKAGE_ROLE_V1,
object_id:RUNTIME_STABLE_ID_V1,physical_unit_ids:[RUNTIME_STABLE_ID_V1],
direct_dependency_binding_ids:[RUNTIME_STABLE_ID_V1]}`. Objects/bindings sort
by ID and arrays are unique. Object bytes are one-to-one with object IDs; a
duplicate SHA under two IDs rejects. Binding dependencies are acyclic.

| Role | `physical` | `test_only` | `generated` | physical-unit IDs |
|---|---:|---:|---:|---|
| `raw_acquisition` | true | false | false | required and nonempty |
| `derived_scientific_output` | true | false | true | required and nonempty |
| `reference_result` | true | false | false | required and nonempty |
| `reference_source_descriptor` | true | false | false | required for physical source |
| `protocol` | false | false | false | empty |
| `power_analysis` | false | false | true | empty |
| `metrology_check_result` | true | false | false | required and nonempty |
| `governance_document` | false | false | false | empty |
| `software_kat_support` | false | true | true | empty; cannot enter cohort scoring |
| `checker_input` | false | true | true | empty; cannot enter cohort scoring |
| `other_documentary` | false | false | true | empty |

`test_only=true` implies `physical=false`; test-only objects cannot enter a
real scoring package except explicitly labeled software/KAT-support inventory.
Physical campaign observation/derived roles require a unit binding. No
contradictory triple is accepted.

## 11. Physical identity, dependency, and custody

`PhaseFDependencyAuditV1` is exactly
`schema_version,dependency_audit_id,manifest_id,edges,undeclared_dependency_count,
unknown_separation_count,result`. Each edge is exactly
`{from_binding_id:RUNTIME_STABLE_ID_V1,to_binding_id:RUNTIME_STABLE_ID_V1,
dependency_type:PHASE_F_DEPENDENCY_TYPE_V1,source_document_sha256:SHA256_V1}`;
dependency type is `raw_source|sample|sensor|preprocessing|model|reference|
derived_output`. Counts are canonical unsigned; result is pass/no_go; pass
requires zero counts and exact manifest equality.

`PhaseFPhysicalUnitLedgerV1` is exactly `schema_version,unit_ledger_id,entries`.
Each entry is exactly
`{unit_id:RUNTIME_STABLE_ID_V1,unit_kind:RUNTIME_STABLE_ID_V1,
identity_issuer_authority_id:RUNTIME_STABLE_ID_V1,
native_identifier:RUNTIME_CANONICAL_TEXT_V1,identity_basis:PHASE_F_IDENTITY_BASIS_V1,
identity_basis_document_sha256:SHA256_V1,parent_unit_ids:[RUNTIME_STABLE_ID_V1],
independent_family_id:RUNTIME_STABLE_ID_V1,source_object_ids:[RUNTIME_STABLE_ID_V1]}`.
Entries and arrays sort unique. Mechanical key is
`(identity_issuer_authority_id,identity_basis,native_identifier)`, globally
unique in the campaign ledger. Duplicate key is alias/NO-GO. External evidence
is checked for cross-key possible aliases; distinct keys do not prove distinctness.

`PhaseFPhysicalIdentityAuditV1` is exactly
`schema_version,identity_audit_id,unit_ledger_sha256,comparisons,
unknown_identity_count,alias_count,result`. A comparison is exactly
`{left_unit_id:RUNTIME_STABLE_ID_V1,right_unit_id:RUNTIME_STABLE_ID_V1,
determination:PHASE_F_IDENTITY_DETERMINATION_V1,evidence_sha256:SHA256_V1}`. Pass requires
zero unknown/alias counts and proof of all claimed independent families.

`PhaseFLocationLedgerV1` is exactly `schema_version,location_ledger_id,locations`;
each location is exactly
`{location_id:RUNTIME_STABLE_ID_V1,location_type:PHASE_F_LOCATION_TYPE_V1,
authority_id:RUNTIME_STABLE_ID_V1,identity_document_sha256:SHA256_V1}`. Location
type is `collection_site|laboratory|storage|instrument_station|
transport_container|other_registered_location`; `other` is defined by its hash.

`PhaseFChainOfCustodyV1` is exactly
`schema_version,custody_ledger_id,campaign_id,unit_ledger_sha256,
location_ledger_sha256,events`. Each event is exactly
`{event_id:RUNTIME_STABLE_ID_V1,event_type:PHASE_F_CUSTODY_EVENT_V1,
occurred_at:UTC_SECOND_TIMESTAMP_V1,source_location_id:RUNTIME_STABLE_ID_V1|null,
destination_location_id:RUNTIME_STABLE_ID_V1|null,input_unit_ids:[RUNTIME_STABLE_ID_V1],
output_unit_ids:[RUNTIME_STABLE_ID_V1],procedure_document_sha256:SHA256_V1|null,
deviation_id:RUNTIME_STABLE_ID_V1|null}`. Events sort by `(occurred_at,event_id)`.

| Event | Source / destination | Input / output | Exact rule |
|---|---|---|---|
| `acquired` | null / required | empty / nonempty | creates ledger units; procedure required |
| `transferred` | required distinct / required | nonempty / identical | same units; no child; procedure required |
| `aliquoted` | required / required | one parent / nonempty new children | every child link exists in ledger; procedure required |
| `processed` | required / required | nonempty / identical | exact same continuing IDs; no implicit partial consumption or child creation |
| `measured` | required / same | nonempty / identical | no creation; procedure required |
| `stored` | required / required | nonempty / identical | same units; procedure required |
| `released_to_analysis` | required / required | nonempty / identical | same units; procedure required |
| `destroyed` | required / null | nonempty / empty | terminal for every input unit; procedure required |

For every continuing unit, next-event source equals previous destination; an
unknown gap is NO-GO. Parent-child events must match ledger links. A valid
destroyed event is terminal and cannot later be used. Incorrect destruction is
campaign/custody NO-GO, not retroactively invalidated through a deviation. A
corrected package requires new custody, package identity, applicable audits,
and a new cohort lock when already locked.

`deviation_ledger_id` is a campaign-scoped
`RUNTIME_STABLE_ID_V1` allocated during F1 campaign registration. It is not a
content-derived digest. The only wire revision schema is
`PhaseFDeviationLedgerRevisionV1`, exactly
`schema_version,deviation_ledger_id,revision_id,campaign_id,revision_number,
previous_revision_sha256,events`. Genesis revision is zero/null; later revision
is prior plus one and prior complete-file hash. Prior event JSON values are
byte-identical and new events append only. Each event is exactly
`{event_id:RUNTIME_STABLE_ID_V1,deviation_id:RUNTIME_STABLE_ID_V1,
event_type:PHASE_F_DEVIATION_EVENT_V1,affected_unit_ids:[RUNTIME_STABLE_ID_V1],
affected_object_sha256s:[SHA256_V1],deviation_code:RUNTIME_STABLE_ID_V1,
detected_stage:f1|f2|f3|f4|f5,required_action:PHASE_F_DEVIATION_ACTION_V1,
decision_authority_id:RUNTIME_STABLE_ID_V1,rationale_document_sha256:SHA256_V1}`.
The first event is `reported`, followed by exactly one terminal resolution.
`exclude_before_lock` resolves only as `resolved_excluded` and requires affected
records absent from the locked cohort; `resolved_no_effect` requires rationale;
`campaign_no_go` resolves only as `campaign_no_go` and blocks F2/F4/F5.
`revision_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` is the content identity of the
complete revision object and includes the pre-existing stable
`deviation_ledger_id`; it excludes only `revision_id`. The stable ledger ID
exists first, so no reverse dependency exists. Package, cohort, and execution
objects bind the exact applicable revision file SHA-256. The obsolete
`PhaseFDeviationLedgerV1` top-level schema does not exist.

## 12. Power interface and analysis

`PhaseFPowerMethodInterfaceV1` is exactly
`schema_version,power_method_interface_id,power_method_id,power_method_version,
method_document_sha256,primary_metric_ids,parameter_specs,
required_sensitivity_case_ids,output_spec`. A parameter spec is exactly
`{parameter_id:RUNTIME_STABLE_ID_V1,value_type:<closed parameter value type>,
unit_rule:PHASE_F_UNIT_RULE_V1,required:BOOLEAN_V1,range_rule:PHASE_F_RANGE_RULE_V1}`.
Unit rule is only `{type:"none"}` or `{type:"exact",unit:UNIT_TEXT_V1}`;
`owner_selected_exact` does not exist. Exact unit is fixed by the reviewed
interface; a different unit requires a new interface/version. Range rules are
`unbounded|nonnegative|positive|{type:"closed_interval",minimum:PHASE_F_PARAMETER_VALUE_V1,maximum:PHASE_F_PARAMETER_VALUE_V1}|{type:"open_interval",minimum:PHASE_F_PARAMETER_VALUE_V1,maximum:PHASE_F_PARAMETER_VALUE_V1}|{type:"enum_values",values:[PHASE_F_PARAMETER_VALUE_V1]}`; endpoints match type/unit.

`output_spec` is a sorted nonempty array of
`{output_id:RUNTIME_STABLE_ID_V1,value_type:<closed value type>,
unit_rule:PHASE_F_UNIT_RULE_V1,range_rule:PHASE_F_RANGE_RULE_V1}`. Standard
rows `minimum_eligible_records`, `minimum_independent_families`,
`minimum_positive_records`, `minimum_negative_records`,
`required_stratum_minimum_records`, and `required_stratum_minimum_families`
all use `value_type=integer`, `unit_rule={type:"none"}`, and
`range_rule=positive`. Their actual values are
`PHASE_F_PARAMETER_VALUE_V1` with `type="integer"` and positive-range
validation. No
arbitrary output ID affects F1 protocol generation.

`PhaseFPowerAnalysisRecordV1` is exactly
`schema_version,power_analysis_id,power_method_id,power_method_version,
power_method_interface_sha256,software_source_sha,software_binary_sha256,
parameters,sensitivity_cases,outputs,created_at`. Parameter rows are
`{parameter_id:RUNTIME_STABLE_ID_V1,value:PHASE_F_PARAMETER_VALUE_V1}`; output
rows are `PhaseFPowerOutputValueV1`; case rows are exactly
`{case_id:RUNTIME_STABLE_ID_V1,parameter_overrides:[PhaseFSensitivityOverrideV1],
outputs:[PhaseFPowerOutputValueV1]}`. Each case has at least one override, only declared
parameter IDs, no duplicate override, base type/unit/range, all required
unchanged base parameters, unchanged method ID/version/output semantics, and
unchanged power target unless explicitly declared. Checker constructs base plus
overrides and validates again. Scientific reviewer decides adequacy. Construction
is ordered: retrieve/verify the interface; choose exact approved parameters;
validate required, unknown, type, unit, and range; execute exact method/software;
create outputs; validate output IDs/types/units/ranges; evaluate every required
sensitivity case; construct the complete analysis object; calculate its
content-derived `power_analysis_id`; obtain independent scientific review; then
create the `power_registered` registry attestation. `analysis_id` is not a
second wire field and means `power_analysis_id` only in legacy prose.

## 13. Metrology and reference result projection

`PhaseFMetrologyPolicyV1` is exactly
`schema_version,metrology_policy_id,endpoint_policies`. Each endpoint policy is
exactly
`{endpoint_id:RUNTIME_STABLE_ID_V1,reference_type:PHASE_F_REFERENCE_TYPE_V1,
allowed_methods:[{method_id:RUNTIME_STABLE_ID_V1,method_version:RUNTIME_CANONICAL_TEXT_V1}],
allowed_authority_ids:[RUNTIME_STABLE_ID_V1],measurand_id:RUNTIME_STABLE_ID_V1,
result_unit:UNIT_TEXT_V1,blinding_requirement:"blinded_to_assessment",
uncertainty_policy:PhaseFUncertaintyPolicyV1,lod_loq_policy:PhaseFLODLOQPolicyV1,
calibration_policy:PhaseFCheckListV1,qc_policy:PhaseFCheckListV1,
chain_of_custody_required:true,traceability_document_required:true,
limitations_document_required:true}`. Method and authority arrays are sorted
nonempty and exactly equal the corresponding runtime `ReferenceAuthorityRuleV1`
allowed lists; semantics are Cartesian.

`PhaseFMetrologyCheckSpecV1` is exactly
`{check_id:RUNTIME_STABLE_ID_V1,check_kind:PHASE_F_CHECK_KIND_V1,
method_id:RUNTIME_STABLE_ID_V1,method_version:RUNTIME_CANONICAL_TEXT_V1,
authority_id:RUNTIME_STABLE_ID_V1,procedure_document:PhaseFObjectReferenceV1,
measurand_id:RUNTIME_STABLE_ID_V1,result_unit:UNIT_TEXT_V1,
comparator:greater_than_or_equal|less_than_or_equal,threshold:RUNTIME_F64_V1,
failure_action:exclude_before_lock|campaign_no_go}`. Policy endpoints contain
sorted check-spec arrays.

`PhaseFMetrologyCheckResultV1` is exactly
`{schema_version:JSON_INTEGER_ONE,check_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,check_id:RUNTIME_STABLE_ID_V1,
reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,performed_at:UTC_SECOND_TIMESTAMP_V1,
method_id:RUNTIME_STABLE_ID_V1,method_version:RUNTIME_CANONICAL_TEXT_V1,
authority_id:RUNTIME_STABLE_ID_V1,measurand_id:RUNTIME_STABLE_ID_V1,
value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1,result:PHASE_F_CHECK_RESULT_V1}`. Checker recomputes
the comparator; manually inconsistent result rejects. Every required calibration
and QC result must pass.

`PhaseFLODLOQPolicyV1` is `{type:"not_applicable"}` or
`{type:"required",lod_value:RUNTIME_F64_V1,lod_unit:UNIT_TEXT_V1,
loq_value:RUNTIME_F64_V1,loq_unit:UNIT_TEXT_V1,below_lod_action:PHASE_F_DEVIATION_ACTION_V1,
between_lod_loq_action:PHASE_F_DEVIATION_ACTION_V1}`. Units are never converted
by the checker; `lod_value<=loq_value`. R6 permits no implicit or explicit unit
conversion inside the Phase-F checker:
`lod_unit == loq_unit == endpoint policy result_unit` by exact byte equality.
A laboratory using another unit must convert upstream under an independently
validated method before constructing the Phase-F reference object. No untyped
LOD/LOQ exists.

`PhaseFReferenceSourceDescriptorV1` is exactly
`schema_version,reference_source_id,source_file_sha256,evidence_origin,
dependency_completeness,experiment_scope,acquisition_families,direct_dependencies`
using exact Rust types from §2. Physical admissibility requires
`evidence_origin=physical` and `dependency_completeness=complete`. Result source
ID and descriptor hash must resolve to the same descriptor.

`PhaseFReferenceResultV1` has exactly common fields
`schema_version,reference_result_id,endpoint_id,reference_endpoint_id,
reference_source_id,reference_source_descriptor_sha256,reference_type,method_id,
method_version,authority_id,blinding_state,uncertainty,
limitations,limitations_document_sha256,traceability_document_sha256,
chain_of_custody_sha256`. Types are schema integer, external ID, runtime ID,
runtime stable ID, external source ID, SHA-256, tagged type, runtime ID,
canonical text, runtime ID, exact `BlindingStateV1`, exact quantified
`{type:"quantified",measure_id:RUNTIME_STABLE_ID_V1,value:RUNTIME_F64_V1,
unit:UNIT_TEXT_V1}`, sorted unique `[RUNTIME_CANONICAL_TEXT_V1]`, and three
SHA-256 values. Mechanism adds
`hypothesis_id:RUNTIME_STABLE_ID_V1,outcome:supports|contradicts|not_assessed|unavailable`;
health adds `target:HealthTargetV1,label:RUNTIME_CANONICAL_TEXT_V1`.
`reference_endpoint_id` is never aliased to `reference_result_id`. The fields
`result_value` and `result_unit` do not exist in this schema.

Projection is total and exact: endpoint ID, reference endpoint ID, source ID,
mechanism hypothesis/outcome or health target/label, method ID/version,
authority ID, blinding state, quantified uncertainty, and limitations copy
field-for-field into the current `ReferenceEndpointV1`. Evidence metadata
(`reference_source_descriptor_sha256`, document hashes, and reference type)
remains external. Physical measurement values and units are
`METROLOGY_PROVENANCE_ONLY`; they remain in raw reference files, metrology
checks, or traceability evidence and are never projected because the runtime
endpoint has no such fields. No result can produce an endpoint with a missing
field, default, target mismatch, or source mismatch.

## 14. Cohort, release, claim state, and monitoring

`PhaseFCohortLockRecordV1` is exactly
`schema_version,cohort_lock_id,protocol_sha256,package_manifest_sha256,
power_analysis_sha256,dependency_audit_sha256,physical_unit_ledger_sha256,
identity_audit_sha256,location_ledger_sha256,chain_of_custody_sha256,
deviation_ledger_sha256,metrology_policy_sha256,scientific_admissibility_audit_sha256,
reference_result_sha256s,reference_source_descriptor_sha256s,locked_at`.
Hash arrays sort unique; ID follows §3.

`PhaseFExecutionRecordV1` is exactly
`schema_version,execution_id,cohort_lock_record_sha256,owner_approval_file_sha256,
protocol_sha256,deviation_ledger_sha256,release_code_sha,checker_binary_sha256,
validation_manifest_sha256,started_at,completed_at,result`, with exact UTC
times, `completed_at>started_at`, and pass/no_go result.

`PhaseFReleaseRecordV1` is exactly
`schema_version,release_record_id,claim_id,claim_statement,release_code_sha,
protocol_sha256,cohort_lock_record_sha256,owner_approval_file_sha256,
execution_record_sha256,validation_manifest_sha256,monitoring_policy_sha256,
metrology_policy_sha256,valid_from,valid_until,limitations`.
Claim ID is runtime stable; statement is canonical text; hashes are SHA-256;
times are UTC with `valid_from<valid_until`; limitations sort unique. The ID
excludes only `release_record_id`; the complete release file is then hashed.
Registration is a later signed `release_registered` record with
`subject_id=release_record_id` and `subject_sha256=release_file_sha256`, followed
by the registry-record hash. No registry pointer, sequence, signature, or tag
field is present in the release file.

`PhaseFClaimStateRecordV1` is exactly
`schema_version,claim_state_record_id,claim_id,release_record_id,
previous_claim_state_record_id,state,reason_code,effective_at,
superseding_release_record_id,activation_review_bundle_sha256,
reinstatement_approval_sha256,limitations`. `previous_claim_state_record_id`
is nullable only for the initial state; `superseding_release_record_id` is
non-null only for superseded; `activation_review_bundle_sha256` is non-null
only for initial ACTIVE; `reinstatement_approval_sha256` is non-null only for a
suspended-to-active reinstatement. State is
`PHASE_F_CLAIM_STATE_V1`; `reason_code:PHASE_F_CLAIM_REASON_V1` is the exact table:

| Reason | Legal prior → next | Extra authority |
|---|---|---|
| `initial_release` | none → active | release record and exact F5 review bundle |
| `monitoring_breach` | active → suspended | valid monitoring incident |
| `reference_qc_breach` | active → suspended | failed required QC |
| `domain_breach` | active → suspended | domain evidence |
| `key_compromise` | active → suspended or withdrawn per F0 row | un-compromised path |
| `key_revocation` | active → suspended or withdrawn per F0 row | un-compromised path |
| `periodic_expiry` | active or suspended → expired | no shortcut |
| `manual_withdrawal` | active or suspended → withdrawn | governance record |
| `superseded_by_new_release` | active or suspended → superseded | new release |
| `approved_reinstatement` | suspended → active | valid five-role approval and same-release mode |

No other transition is legal. `new_release_required` forbids old-release
reinstatement; `withdraw_only` forbids reinstatement. If an incident or breach
exists without its required state record, claim-status returns NOT_ACTIVE.

An initial ACTIVE state is constructed only after the release file and exact
F5 candidate are complete and the five-role F5 review bundle is aggregate GO.
The candidate's `initial_claim_state_sha256` is the non-registrable proposed
template hash; the final state is made by the single fixed insertion rule in
§5.1. The final state contains `activation_review_bundle_sha256` for that exact
bundle; its complete file is hashed and then a later `claim_state_changed`
registry record attests it. The state never references its future registry
record or final tag. `INITIAL_ACTIVE_CONSTRUCTION_AMBIGUITIES=0`.

`PhaseFReinstatementApprovalV1` is exactly
`schema_version,reinstatement_id,claim_id,suspended_state_record_id,
suspension_reason,required_corrective_action,corrective_evidence_sha256s,
execution_record_sha256,review_bundle_sha256`. It requires the referenced
five-role bundle to validate as aggregate GO/P0=0/P1=0 and the exact allowed
trigger row; reviewer decisions and counts exist only in that bundle.

Monitoring policy `PhaseFMonitoringPolicyV1` is exactly
`schema_version,monitoring_policy_id,monitoring_interval_seconds,required_metrics,
metric_thresholds,missing_monitoring_action,domain_breach_action,
reference_qc_breach_action`. Status vocabularies are
`domain_compliance={compliant,out_of_domain,unknown}` healthy `compliant`;
`reference_qc_status={pass,fail,unknown}` healthy `pass`;
`calibration_status={pass,fail,unknown}` healthy `pass`; and
`reference_uncertainty_status={within_limit,above_limit,unknown}` healthy
`within_limit`. `required_metrics` is exactly the sorted list
`domain_compliance,reference_qc_status,calibration_status,sensor_drift,
invalid_input_rate,indeterminate_rate,data_quality_insufficient_rate,
exclusion_rate,reference_uncertainty_status,software_git_sha,
checker_binary_sha256,trust_store_sha256,trust_root_id,owner_approval_id,
release_record_id`. Numeric metrics are only sensor drift and the four rates
`invalid_input_rate,indeterminate_rate,data_quality_insufficient_rate,
exclusion_rate`. Rate units are null; drift uses exact unit. Threshold is
`{metric_id:PHASE_F_MONITORING_NUMERIC_METRIC_V1,comparator:greater_than_or_equal|less_than_or_equal,
value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1|null}`; there are exactly five threshold
rows, one each for `sensor_drift`, `invalid_input_rate`, `indeterminate_rate`,
`data_quality_insufficient_rate`, and `exclusion_rate`; no status, hash, or ID
metric has a threshold. F0 supplies all five values and no threshold is
optional. All actions are `suspend`.

The binding metrics use exact variants: `software_git_sha` uses
`{type:"git_sha",value:GIT_SHA_V1}`; `checker_binary_sha256` and
`trust_store_sha256` use `{type:"sha256",value:SHA256_V1}`;
`trust_root_id` uses `{type:"stable_id",value:RUNTIME_STABLE_ID_V1}`; and
`owner_approval_id` and `release_record_id` use
`{type:"external_digest_id",value:PHASE_F_EXTERNAL_DIGEST_ID_V1}`. Status
values use the metric-specific status enum; no arbitrary stable-ID status is
accepted.

`PhaseFMonitoringRecordV1` is exactly
`schema_version,monitoring_record_id,release_record_id,claim_id,window_start,
window_end,policy_sha256,measurements,breaches,result`.
Every required metric appears once in policy order. Measurement is
`{metric_id:PHASE_F_MONITORING_METRIC_V1,value:PhaseFMonitoringValueV1}`; status uses the
metric vocabulary, numeric uses `RUNTIME_F64_V1`, and bindings use named ID/hash
types. Breach is
`{metric_id:PHASE_F_MONITORING_METRIC_V1,breach_code:PHASE_F_BREACH_CODE_V1,
evidence_sha256:SHA256_V1}` sorted by metric ID. All required metrics exist
exactly once, with exact value types, healthy statuses, release bindings,
passing thresholds, and evidence. `breaches` must equal the exact recomputed failed-metric set: one row
for every failed required metric and zero rows for every passing metric. The
only breach codes are `missing_metric`, `unhealthy_status`, `threshold_failed`,
`binding_mismatch`, and `missing_evidence`; no duplicate, extra, or missing
metric is valid. The code is derived from the first applicable failed reason in
this fixed order: absent metric -> `missing_metric`; unhealthy status ->
`unhealthy_status`; failed numeric comparison -> `threshold_failed`; wrong
release/policy binding -> `binding_mismatch`; absent evidence ->
`missing_evidence`. Declared result is pass iff `breaches` is empty, otherwise
suspend, and must equal recomputation. Before any accepted PASS
window, `initial_due = initial_active_effective_at +
monitoring_interval_seconds`; `now<initial_due` is CURRENT and `now>=initial_due`
is OVERDUE/NOT_ACTIVE with suspension required. After an accepted PASS,
`next_due = latest_accepted_window_end + monitoring_interval_seconds` and the
same strict comparison applies.

An accepted monitoring window is exactly a structurally valid, recomputed-pass,
registry-bound, current-chain record for the correct release and policy,
submitted no later than its due boundary. Only its `window_end` anchors the
next due time. `window_start < window_end`; the first PASS starts at initial
ACTIVE `effective_at`; each later PASS starts at the prior accepted PASS
`window_end`; and `window_end <= due` for the period satisfied. A late PASS can
provide corrective evidence but cannot retroactively make an overdue interval
ACTIVE; reinstatement is required. A suspend record is never accepted.

## 15. Retention, incidents, and compromise

`PhaseFIncidentRecordV1` is exactly
`schema_version,incident_id,scope,incident_type,detected_at,
affected_object_sha256s,affected_unit_ids,evidence_references,required_action,
incident_status`. Affected object entries are sorted
`PhaseFObjectDigestV1`; unit IDs are
sorted runtime IDs; evidence references are sorted `PhaseFObjectReferenceV1`;
type/action/status use §2 enums. `other_registered_incident` requires an
immutable incident-type definition document. `scope` is the exact tagged union
`PhaseFIncidentScopeV1`: release scope carries a release ID, campaign
abandonment carries a campaign ID and no release ID, and registry compromise
uses registry-namespace scope. ID and complete-file hash follow §3/§9.
Campaign-abandonment closure requires an independent five-role review bundle
in `evidence_references`; it is not inferred from the operator that records the
incident.

`PhaseFRetentionAuditV1` is exactly
`schema_version,retention_audit_id,release_record_id,policy_sha256,audited_at,
object_checks,result`. Each check is exactly
`{object_sha256:PhaseFObjectDigestV1,
primary_available:BOOLEAN_V1,primary_verified:BOOLEAN_V1,
verified_backup_count:CANONICAL_UNSIGNED_INTEGER_V1,
last_backup_verification_at:UTC_SECOND_TIMESTAMP_V1,result:PHASE_F_RESULT_V1}`.
Pass requires primary available and exact, backup count at least F0, and backup
age below F0 interval. A `retention_audited` record binds every current audit.

Retention starts when an authority object is first referenced by valid registry
record. After terminal state, deadline is terminal `effective_at` plus F0
retention seconds; retrieval is required while `now<deadline`, unless another
 nonterminal release references it. Pre-release campaigns end retention only by
an independently reviewed `campaign_abandonment` incident with campaign scope;
the deadline is `incident.detected_at + retention_seconds`, and all campaign
authority objects are retained until that deadline. Deletion is never
silent. Replacement is an additional copy with identical bytes, SHA, and length
recorded in the next audit; different bytes are a new object. If every copy
disappears before deadline, retention failure is required and claim-status is
NOT_ACTIVE.

Default consequences are exact: key compromise suspend; key revocation suspend
or withdraw per F0; registry equivocation NOT_ACTIVE immediately; data
integrity, custody break, undeclared dependency, monitoring breach, reference
QC breach, domain breach, and retention failure suspend. Incident alone does
not change state; missing transition is NOT_ACTIVE.

Registry-key compromise uses one exact independent path.
`PhaseFRegistryCompromiseEmergencyV1` is exactly
`schema_version,emergency_id,registry_namespace_id,incident_id,declared_at,
affected_claim_ids,incident_record_sha256,review_bundle_sha256,
repository_commit_sha,repository_blob,action`, where action is the literal
`suspend_all_active_claims`. `repository_blob:GIT_BLOB_V1` is the Git blob of
the emergency file at `repository_commit_sha:GIT_SHA_V1`; the commit must be
reachable from the live remote `main` used as repository governance authority.
If either Git fact cannot be verified, the emergency object is invalid.
IDs/hashes use named types and ID follows §3. One full five-role review bundle,
using the same aggregate rule as every other bundle, is required; the registry
key is not used. `--registry-compromised` accepts only this object, its exact
Git commit/blob, and aggregate-GO bundle and immediately returns NOT_ACTIVE for
affected claims. No unsigned flag or revoked-root bypass exists.

To keep the emergency path acyclic, the emergency review bundle targets the
exact incident file hash plus the repository commit/blob tuple and affected
claim set; it does not target the completed emergency object that contains the
bundle hash. The emergency object is complete only after that bundle hash is
inserted.

Owner-key compromise uses the un-compromised registry/governance path to append
the exact suspend/withdraw state; owner signature is not required for that
non-active transition. Recovery requires a new owner key, root, owner approval,
run, and release according to the exact F-OD-16 `resolution_mode`; compromised
key never revokes itself.

## 16. Master schema catalog

This is the one catalog for every external R6 schema. Each row includes exact
fields, identity, complete-file hash, producer, validator, stage, requirement,
AC, test, evidence, and registry relation. The canonical substantive
requirement/AC/test/evidence procedures are §19.1; legacy R5 identifiers in the
final column are historical cross-references only and are not current R6
acceptance authority.

| Schema | Field closure / identity | Producer; validator; stage | Registry relation | Requirement / AC / test / evidence |
|---|---|---|---|---|
| `PhaseFDecisionBundleV1` | §4; §3 ID; complete hash; unsigned | F0; checker; F0 | protocol authority | R5-01 / AC5-01 / T5-01 / EV5-01 |
| `PhaseFIndependentReviewBundleV1` | §5 exact five rows; §3 ID; complete hash | independent roles; checker; approvals | tag evidence | R5-02 / AC5-02 / T5-02 / EV5-02 |
| `PhaseFProtocolProjectionV1` | §4 exact plan contract; no wire ID | checker; projection; F1 | protocol | R5-03 / AC5-03 / T5-03 / EV5-03 |
| `PhaseFAuthorityEnrollmentV1` | §5.1 unsigned; §3 ID/file hash | governance; enrollment; readiness | authority_enrolled | R5-04 / AC5-04 / T5-04 / EV5-04 |
| `PhaseFCheckerBuildEvidenceV1` | §7 exact fields; §3 ID; complete hash | checker builder; independent verifier; readiness | readiness evidence | R6-03 / AC6-03 / T6-03 / EV6-03 |
| `PhaseFCheckerReadinessEvidenceV1` | §7 exact fields; §3 ID; complete hash | checker readiness; independent verifier; readiness | readiness tag evidence | R6-03 / AC6-03 / T6-03 / EV6-03 |
| `PhaseFCheckerReportV1` | §7 exact fields including semantic command and argv; complete hash | checker; report validator; all | none | R6-03 / AC6-03 / T6-03 / EV6-03 |
| `PhaseFF5ReleaseCandidateV1` | §5.1 exact fields; §3 ID; complete hash | F5 release; independent reviewers; F5 | F5 review evidence | R6-08 / AC6-08 / T6-08 / EV6-08 |
| `PhaseFRegistryRelationV1` | §9 typed tuple | registry; chain; F1-F5 | record-contained | R5-06 / AC5-06 / T5-06 / EV5-06 |
| `PhaseFRegistryRecordV1` | §8 strict signed bytes; complete hash | registry; chain; F1-F5 | chain | R5-07 / AC5-07 / T5-07 / EV5-07 |
| `PhaseFRegistryHeadV1` | §8 strict signed bytes; complete hash | registry; head; all | resolver | R5-08 / AC5-08 / T5-08 / EV5-08 |
| `PhaseFObjectReferenceV1` | §10 exact three fields | all; retrieval; all | target | R5-09 / AC5-09 / T5-09 / EV5-09 |
| `PhaseFRetrievalVerificationV1` | §10 exact fields; §3 ID/hash | checker; retrieval; all | evidence | R5-10 / AC5-10 / T5-10 / EV5-10 |
| `PhaseFPackageManifestV1` | §10 exact roles/bindings; §3 ID/hash | campaign; package; F2 | package_registered | R5-11 / AC5-11 / T5-11 / EV5-11 |
| `PhaseFDependencyAuditV1` | §11 exact edges/result; §3 ID/hash | auditor; dependency; F2 | package dependency | R5-12 / AC5-12 / T5-12 / EV5-12 |
| `PhaseFPhysicalUnitLedgerV1` | §11 native identity; §3 ID/hash | campaign; identity; F2 | package dependency | R5-13 / AC5-13 / T5-13 / EV5-13 |
| `PhaseFPhysicalIdentityAuditV1` | §11 exact comparisons/result; §3 ID/hash | auditor; identity; F2 | package dependency | R5-14 / AC5-14 / T5-14 / EV5-14 |
| `PhaseFLocationLedgerV1` | §11 exact locations; §3 ID/hash | operations; custody; F2 | package dependency | R5-15 / AC5-15 / T5-15 / EV5-15 |
| `PhaseFChainOfCustodyV1` | §11 exact event matrix; §3 ID/hash | custodians; custody; F2-F4 | package dependency | R5-16 / AC5-16 / T5-16 / EV5-16 |
| `PhaseFDeviationLedgerRevisionV1` | §11 stable-ledger/revision construction; §3 revision ID/hash | campaign; deviation; F2-F4 | package/execution | R6-01 / AC6-01 / T6-01 / EV6-01 |
| `PhaseFPowerMethodInterfaceV1` | §12 unit/range/output rows; §3 ID/hash | statistician; power; F1 | power dependency | R5-18 / AC5-18 / T5-18 / EV5-18 |
| `PhaseFPowerAnalysisRecordV1` | §12 params/cases; §3 ID/hash | statistician; power; F1 | power subject | R5-19 / AC5-19 / T5-19 / EV5-19 |
| `PhaseFMetrologyPolicyV1` | §13 Cartesian methods/checks; §3 ID/hash | metrology; policy; F0/F2 | package/release | R5-20 / AC5-20 / T5-20 / EV5-20 |
| `PhaseFMetrologyCheckSpecV1` | §13 exact fields | metrology; policy; F2 | nested policy | R5-21 / AC5-21 / T5-21 / EV5-21 |
| `PhaseFMetrologyCheckResultV1` | §13 exact fields/math; complete hash | laboratory; result; F2 | package evidence | R5-22 / AC5-22 / T5-22 / EV5-22 |
| `PhaseFReferenceSourceDescriptorV1` | §13 runtime types; §3 ID/hash | laboratory/data; source; F2 | package dependency | R5-23 / AC5-23 / T5-23 / EV5-23 |
| `PhaseFReferenceResultV1` | §13 adjudicated fields and exact runtime projection; §3 ID/hash | laboratory; reference; F2 | package dependency | R6-07 / AC6-07 / T6-07 / EV6-07 |
| `PhaseFScientificAdmissibilityAuditV1` | exact fields below; §3 ID/hash | scientific reviewer/checker; F2 | scientific_admissibility | R5-25 / AC5-25 / T5-25 / EV5-25 |
| `PhaseFCohortLockRecordV1` | §14 exact hashes; §3 ID/hash | campaign; cohort; F2 | cohort_locked | R5-26 / AC5-26 / T5-26 / EV5-26 |
| `PhaseFExecutionRecordV1` | §14 exact time/result; §3 ID/hash | release; execution; F4 | execution_registered | R5-27 / AC5-27 / T5-27 / EV5-27 |
| `PhaseFReleaseRecordV1` | §14 semantic-only fields; §3 ID; complete hash | release; release; F5 | release_registered external attestation | R6-01/R6-08 / AC6-01/08 / T6-01/08 / EV6-01/08 |
| `PhaseFClaimStateRecordV1` | §14 exact nullable fields and transition; §3 ID; complete hash | governance; state; F5+ | claim_state_changed external attestation | R6-01/R6-08 / AC6-01/08 / T6-01/08 / EV6-01/08 |
| `PhaseFReinstatementApprovalV1` | §14 review-bundle reference and trigger; §3 ID/hash | governance; reinstatement; F5+ | state dependency | R6-08 / AC6-08 / T6-08 / EV6-08 |
| `PhaseFMonitoringPolicyV1` | §14 metric vocabulary; §3 ID/hash | F0; monitoring; F5+ | release dependency | R5-31 / AC5-31 / T5-31 / EV5-31 |
| `PhaseFMonitoringRecordV1` | §14 derived result/window; §3 ID; complete hash | operations; monitoring; F5+ | monitoring_recorded external attestation | R6-09 / AC6-09 / T6-09 / EV6-09 |
| `PhaseFIncidentScopeV1` | §15 exact tagged union | governance; incident; all | nested in incident | R6-10 / AC6-10 / T6-10 / EV6-10 |
| `PhaseFIncidentRecordV1` | §15 exact scoped fields/enums; §3 ID/hash | operations/governance; incident; all | incident_recorded | R6-10 / AC6-10 / T6-10 / EV6-10 |
| `PhaseFRetentionAuditV1` | §15 exact checks/result and typed relations; §3 ID/hash | operations; retention; all | retention_audit_recorded | R6-10 / AC6-10 / T6-10 / EV6-10 |
| `PhaseFRegistryCompromiseEmergencyV1` | §15 independent Git/blob path; §3 ID/hash | security/operations; emergency; claim-status | emergency input | R6-10 / AC6-10 / T6-10 / EV6-10 |
| six `PhaseF*ApprovalV1` bodies | §6 exact ordered ASCII | five roles; tag validator; gates | tag message hash | R5-36 / AC5-36 / T5-36 / EV5-36 |

`PhaseFScientificAdmissibilityAuditV1` is exactly
`schema_version,scientific_admissibility_audit_id,protocol_sha256,
package_manifest_sha256,dependency_audit_sha256,identity_audit_sha256,
reference_assessments,reviewer_role,result`. Assessment is exactly
`{reference_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,endpoint_id:RUNTIME_STABLE_ID_V1,
evidence_category:SCIENTIFIC_EVIDENCE_CATEGORY_V1,claim_ceiling:SCIENTIFIC_CLAIM_CEILING_V1,
dependency_status:known_separated|known_overlap|unknown,
identity_status:distinct|same|unknown,
admissibility:physical_support_allowed|limited_only|not_assessed|unavailable|not_admissible}`.
Reviewer role is `scientific_metrology`; result is pass/no_go. Known overlap,
unknown dependency, and same/unknown identity where independence is required
are not admissible; category ceilings cannot be exceeded. Checker verifies
structure/category/relations; reviewer supplies category/assessment. Cohort
lock binds exact audit hash and package registry relation binds it.

### 16.1 Normative field-type audit

The following is the complete field audit. `JSON_INTEGER_ONE` is the literal
JSON integer `1`; `SORTED_UNIQUE<T>` is a strictly increasing JSON array whose
member type is exactly `T`; `NONEMPTY_SORTED_UNIQUE<T>` adds nonempty; and
`JCS_OBJECT<T>` means the complete canonical object type `T`. These are closed
Phase-F constructions, not untyped containers. Every field in every normative
object is listed here or in the exact nested-row definition beside it.

| Object | Every field and exact type |
|---|---|
| `PhaseFDecisionBundleV1` | `schema_version:JSON_INTEGER_ONE`; `decision_bundle_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `decisions:NONEMPTY_SORTED_UNIQUE<PhaseFDecisionRowV1>` |
| `PhaseFProtocolProjectionV1` | plan-only `decision_bundle_sha256:SHA256_V1`; `protocol_toml_sha256:SHA256_V1`; `runtime_protocol:MhiValidationProtocolV1`; `projection_result:PHASE_F_RESULT_V1` |
| `PhaseFDecisionRowV1` | `decision_id:RUNTIME_STABLE_ID_V1`; `value:PhaseFDecisionValueV1`; `decision_owner_role:PHASE_F_REVIEW_ROLE_V1`; `rationale_document_sha256:SHA256_V1` |
| `PhaseFIndependentReviewBundleV1` | `schema_version:JSON_INTEGER_ONE`; `review_bundle_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `review_target_kind:PHASE_F_REVIEW_TARGET_KIND_V1`; `review_target_sha256:SHA256_V1|null`; `review_target_git_sha:GIT_SHA_V1`; `reviews:NONEMPTY_SORTED_UNIQUE<PhaseFIndependentReviewV1>`; `aggregate_p0_count:CANONICAL_UNSIGNED_INTEGER_V1`; `aggregate_p1_count:CANONICAL_UNSIGNED_INTEGER_V1`; `aggregate_decision:PHASE_F_DECISION_V1` |
| `PhaseFIndependentReviewV1` | `role:PHASE_F_REVIEW_ROLE_V1`; `review_instance_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `decision:PHASE_F_DECISION_V1`; `p0_count:CANONICAL_UNSIGNED_INTEGER_V1`; `p1_count:CANONICAL_UNSIGNED_INTEGER_V1`; `finding_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `review_artifact_reference:PhaseFObjectReferenceV1` |
| `PhaseFAuthorityEnrollmentV1` | `schema_version:JSON_INTEGER_ONE`; `enrollment_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `phase_f_plan_tag:PHASE_F_TAG_NAME_V1`; `f0_decisions_tag:PHASE_F_TAG_NAME_V1`; `readiness_tag:PHASE_F_TAG_NAME_V1`; `owner_authority_id:RUNTIME_STABLE_ID_V1`; `registry_authority_id:RUNTIME_STABLE_ID_V1`; `owner_public_key:ED25519_PUBLIC_KEY_V1`; `registry_public_key:ED25519_PUBLIC_KEY_V1`; `owner_public_key_fingerprint:SHA256_V1`; `registry_public_key_fingerprint:SHA256_V1`; `owner_authority_document:PhaseFObjectReferenceV1`; `registry_authority_document:PhaseFObjectReferenceV1`; `custody_policy_sha256:SHA256_V1`; `created_at:UTC_SECOND_TIMESTAMP_V1` |
| `PhaseFCheckerBuildEvidenceV1` | `schema_version:JSON_INTEGER_ONE`; `build_evidence_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `build_ordinal:PHASE_F_CHECKER_BUILD_ORDINAL_V1`; `checker_source_review_sha:GIT_SHA_V1`; `checker_source_tree:GIT_TREE_V1`; `checker_dependency_lock_sha256:SHA256_V1`; `rustc_version:RUNTIME_CANONICAL_TEXT_V1`; `cargo_version:RUNTIME_CANONICAL_TEXT_V1`; `macos_uname:RUNTIME_CANONICAL_TEXT_V1`; `macos_arch:RUNTIME_CANONICAL_TEXT_V1`; `macos_product_version:RUNTIME_CANONICAL_TEXT_V1`; `macos_build_version:RUNTIME_CANONICAL_TEXT_V1`; `environment:SORTED_UNIQUE<PhaseFEnvironmentEntryV1>`; `build_command:RUNTIME_CANONICAL_TEXT_V1`; `build_transcript_sha256:SHA256_V1`; `checker_binary_sha256:SHA256_V1`; `fresh_source_directory:BOOLEAN_V1`; `fresh_target_directory:BOOLEAN_V1`; `fresh_home:BOOLEAN_V1`; `fresh_cargo_home:BOOLEAN_V1`; `cargo_home_config_absent:BOOLEAN_V1`; `result:PHASE_F_BUILD_RESULT_V1` |
| `PhaseFCheckerReadinessEvidenceV1` | `schema_version:JSON_INTEGER_ONE`; `readiness_evidence_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `build1_sha256:SHA256_V1`; `build2_sha256:SHA256_V1`; `checker_source_review_sha:GIT_SHA_V1`; `checker_source_tree:GIT_TREE_V1`; `checker_dependency_lock_sha256:SHA256_V1`; `checker_binary_sha256:SHA256_V1`; `f_maint_01_status:PHASE_F_MAINTENANCE_STATUS_V1`; `f_maint_02_status:PHASE_F_MAINTENANCE_STATUS_V1`; `result:PHASE_F_BUILD_RESULT_V1` |
| `PhaseFCheckerReportV1` | `schema_version:JSON_INTEGER_ONE`; `checker_binary_sha256:SHA256_V1`; `command:PhaseFCommandV1`; `argv:PhaseFArgvV1`; `input_sha256s:SORTED_UNIQUE<PhaseFNamedDigestV1>`; `decision:PHASE_F_CHECKER_DECISION_V1`; `diagnostic_codes:SORTED_UNIQUE<DIAGNOSTIC_CODE_V1>`; `stdout:PhaseFCheckerStdoutV1`; `exit_code:PhaseFCheckerExitCodeV1` |
| `PhaseFRegistryRelationV1` | `relation_type:PHASE_F_RELATION_TYPE_V1`; `object_kind:PHASE_F_OBJECT_KIND_V1`; `object_sha256:SHA256_V1` |
| `PhaseFRegistryRecordV1` | `schema_version:JSON_INTEGER_ONE`; `registry_namespace_id:RUNTIME_STABLE_ID_V1`; `registry_authority_id:RUNTIME_STABLE_ID_V1`; `sequence:CANONICAL_UNSIGNED_INTEGER_V1`; `predecessor_record_sha256:SHA256_V1|null`; `record_kind:PHASE_F_REGISTRY_RECORD_KIND_V1`; `subject_id:PHASE_F_SUBJECT_ID_V1`; `subject_sha256:SHA256_V1`; `relations:SORTED_UNIQUE<PhaseFRegistryRelationV1>`; `created_at:UTC_SECOND_TIMESTAMP_V1`; `registry_key_fingerprint:SHA256_V1`; `signature:ED25519_SIGNATURE_V1` |
| `PhaseFRegistryHeadV1` | `schema_version:JSON_INTEGER_ONE`; `registry_namespace_id:RUNTIME_STABLE_ID_V1`; `registry_authority_id:RUNTIME_STABLE_ID_V1`; `sequence:CANONICAL_UNSIGNED_INTEGER_V1`; `registry_record_sha256:SHA256_V1`; `issued_at:UTC_SECOND_TIMESTAMP_V1`; `valid_until:UTC_SECOND_TIMESTAMP_V1`; `registry_key_fingerprint:SHA256_V1`; `signature:ED25519_SIGNATURE_V1` |
| `PhaseFObjectReferenceV1` | `immutable_uri:IMMUTABLE_EXTERNAL_URI_V1`; `sha256:SHA256_V1`; `byte_length:CANONICAL_UNSIGNED_INTEGER_V1` |
| `PhaseFRetrievalVerificationV1` | `schema_version:JSON_INTEGER_ONE`; `retrieval_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `object_reference:PhaseFObjectReferenceV1`; `retrieved_sha256:SHA256_V1`; `retrieved_byte_length:CANONICAL_UNSIGNED_INTEGER_V1`; `checker_binary_sha256:SHA256_V1`; `checker_source_review_sha:GIT_SHA_V1`; `retrieved_at:UTC_SECOND_TIMESTAMP_V1`; `verification_result:PHASE_F_RESULT_V1` |
| `PhaseFPackageManifestV1` | `schema_version:JSON_INTEGER_ONE`; `manifest_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `objects:NONEMPTY_SORTED_UNIQUE<PhaseFPackageObjectV1>`; `bindings:NONEMPTY_SORTED_UNIQUE<PhaseFPackageBindingV1>` |
| `PhaseFPackageObjectV1` | `object_id:RUNTIME_STABLE_ID_V1`; `object_reference:PhaseFObjectReferenceV1`; `media_type:RUNTIME_CANONICAL_TEXT_V1`; `format_or_schema:RUNTIME_CANONICAL_TEXT_V1`; `producing_authority_id:RUNTIME_STABLE_ID_V1`; `physical:BOOLEAN_V1`; `test_only:BOOLEAN_V1`; `generated:BOOLEAN_V1`; `retention_class_id:RUNTIME_STABLE_ID_V1` |
| `PhaseFPackageBindingV1` | `binding_id:RUNTIME_STABLE_ID_V1`; `role:PHASE_F_PACKAGE_ROLE_V1`; `object_id:RUNTIME_STABLE_ID_V1`; `physical_unit_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `direct_dependency_binding_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>` |
| `PhaseFDependencyAuditV1` | `schema_version:JSON_INTEGER_ONE`; `dependency_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `manifest_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `edges:SORTED_UNIQUE<PhaseFDependencyEdgeV1>`; `undeclared_dependency_count:CANONICAL_UNSIGNED_INTEGER_V1`; `unknown_separation_count:CANONICAL_UNSIGNED_INTEGER_V1`; `result:PHASE_F_RESULT_V1` |
| `PhaseFDependencyEdgeV1` | `from_binding_id:RUNTIME_STABLE_ID_V1`; `to_binding_id:RUNTIME_STABLE_ID_V1`; `dependency_type:PHASE_F_DEPENDENCY_TYPE_V1`; `source_document_sha256:SHA256_V1` |
| `PhaseFPhysicalUnitLedgerV1` | `schema_version:JSON_INTEGER_ONE`; `unit_ledger_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `entries:NONEMPTY_SORTED_UNIQUE<PhaseFUnitEntryV1>` |
| `PhaseFUnitEntryV1` | `unit_id:RUNTIME_STABLE_ID_V1`; `unit_kind:RUNTIME_STABLE_ID_V1`; `identity_issuer_authority_id:RUNTIME_STABLE_ID_V1`; `native_identifier:RUNTIME_CANONICAL_TEXT_V1`; `identity_basis:PHASE_F_IDENTITY_BASIS_V1`; `identity_basis_document_sha256:SHA256_V1`; `parent_unit_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `independent_family_id:RUNTIME_STABLE_ID_V1`; `source_object_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>` |
| `PhaseFPhysicalIdentityAuditV1` | `schema_version:JSON_INTEGER_ONE`; `identity_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `unit_ledger_sha256:SHA256_V1`; `comparisons:SORTED_UNIQUE<PhaseFIdentityComparisonV1>`; `unknown_identity_count:CANONICAL_UNSIGNED_INTEGER_V1`; `alias_count:CANONICAL_UNSIGNED_INTEGER_V1`; `result:PHASE_F_RESULT_V1` |
| `PhaseFIdentityComparisonV1` | `left_unit_id:RUNTIME_STABLE_ID_V1`; `right_unit_id:RUNTIME_STABLE_ID_V1`; `determination:PHASE_F_IDENTITY_DETERMINATION_V1`; `evidence_sha256:SHA256_V1` |
| `PhaseFLocationLedgerV1` | `schema_version:JSON_INTEGER_ONE`; `location_ledger_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `locations:NONEMPTY_SORTED_UNIQUE<PhaseFLocationV1>` |
| `PhaseFLocationV1` | `location_id:RUNTIME_STABLE_ID_V1`; `location_type:PHASE_F_LOCATION_TYPE_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `identity_document_sha256:SHA256_V1` |
| `PhaseFChainOfCustodyV1` | `schema_version:JSON_INTEGER_ONE`; `custody_ledger_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `campaign_id:RUNTIME_STABLE_ID_V1`; `unit_ledger_sha256:SHA256_V1`; `location_ledger_sha256:SHA256_V1`; `events:NONEMPTY_SORTED_UNIQUE<PhaseFCustodyEventV1>` |
| `PhaseFCustodyEventV1` | `event_id:RUNTIME_STABLE_ID_V1`; `event_type:PHASE_F_CUSTODY_EVENT_V1`; `occurred_at:UTC_SECOND_TIMESTAMP_V1`; `source_location_id:RUNTIME_STABLE_ID_V1|null`; `destination_location_id:RUNTIME_STABLE_ID_V1|null`; `input_unit_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `output_unit_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `procedure_document_sha256:SHA256_V1|null`; `deviation_id:RUNTIME_STABLE_ID_V1|null` |
| `PhaseFDeviationLedgerRevisionV1` | `schema_version:JSON_INTEGER_ONE`; `deviation_ledger_id:RUNTIME_STABLE_ID_V1`; `revision_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `campaign_id:RUNTIME_STABLE_ID_V1`; `revision_number:CANONICAL_UNSIGNED_INTEGER_V1`; `previous_revision_sha256:SHA256_V1|null`; `events:SORTED_UNIQUE<PhaseFDeviationEventV1>` |
| `PhaseFDeviationEventV1` | `event_id:RUNTIME_STABLE_ID_V1`; `deviation_id:RUNTIME_STABLE_ID_V1`; `event_type:PHASE_F_DEVIATION_EVENT_V1`; `affected_unit_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `affected_object_sha256s:SORTED_UNIQUE<SHA256_V1>`; `deviation_code:RUNTIME_STABLE_ID_V1`; `detected_stage:PHASE_F_DETECTED_STAGE_V1`; `required_action:PHASE_F_DEVIATION_ACTION_V1`; `decision_authority_id:RUNTIME_STABLE_ID_V1`; `rationale_document_sha256:SHA256_V1` |
| `PhaseFPowerMethodInterfaceV1` | `schema_version:JSON_INTEGER_ONE`; `power_method_interface_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `power_method_id:RUNTIME_STABLE_ID_V1`; `power_method_version:RUNTIME_CANONICAL_TEXT_V1`; `method_document_sha256:SHA256_V1`; `primary_metric_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `parameter_specs:NONEMPTY_SORTED_UNIQUE<PhaseFParameterSpecV1>`; `required_sensitivity_case_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `output_spec:NONEMPTY_SORTED_UNIQUE<PhaseFOutputSpecV1>` |
| `PhaseFParameterSpecV1` | `parameter_id:RUNTIME_STABLE_ID_V1`; `value_type:PHASE_F_VALUE_TYPE_V1`; `unit_rule:PhaseFUnitRuleV1`; `required:BOOLEAN_V1`; `range_rule:PhaseFRangeRuleV1` |
| `PhaseFOutputSpecV1` | `output_id:RUNTIME_STABLE_ID_V1`; `value_type:PHASE_F_VALUE_TYPE_V1`; `unit_rule:PhaseFUnitRuleV1`; `range_rule:PhaseFRangeRuleV1` |
| `PhaseFPowerAnalysisRecordV1` | `schema_version:JSON_INTEGER_ONE`; `power_analysis_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `power_method_id:RUNTIME_STABLE_ID_V1`; `power_method_version:RUNTIME_CANONICAL_TEXT_V1`; `power_method_interface_sha256:SHA256_V1`; `software_source_sha:GIT_SHA_V1`; `software_binary_sha256:SHA256_V1`; `parameters:NONEMPTY_SORTED_UNIQUE<PhaseFParameterValueRowV1>`; `sensitivity_cases:SORTED_UNIQUE<PhaseFSensitivityCaseV1>`; `outputs:NONEMPTY_SORTED_UNIQUE<PhaseFPowerOutputValueV1>`; `created_at:UTC_SECOND_TIMESTAMP_V1` |
| `PhaseFParameterValueRowV1` | `parameter_id:RUNTIME_STABLE_ID_V1`; `value:PHASE_F_PARAMETER_VALUE_V1` |
| `PhaseFSensitivityCaseV1` | `case_id:RUNTIME_STABLE_ID_V1`; `parameter_overrides:NONEMPTY_SORTED_UNIQUE<PhaseFSensitivityOverrideV1>`; `outputs:NONEMPTY_SORTED_UNIQUE<PhaseFPowerOutputValueV1>` |
| `PhaseFSensitivityOverrideV1` | `parameter_id:RUNTIME_STABLE_ID_V1`; `value:PHASE_F_PARAMETER_VALUE_V1` |
| `PhaseFPowerOutputValueV1` | `output_id:RUNTIME_STABLE_ID_V1`; `value:PHASE_F_PARAMETER_VALUE_V1` |
| `PhaseFMetrologyPolicyV1` | `schema_version:JSON_INTEGER_ONE`; `metrology_policy_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_policies:NONEMPTY_SORTED_UNIQUE<PhaseFEndpointMetrologyPolicyV1>` |
| `PhaseFEndpointMetrologyPolicyV1` | `endpoint_id:RUNTIME_STABLE_ID_V1`; `reference_type:PHASE_F_REFERENCE_TYPE_V1`; `allowed_methods:NONEMPTY_SORTED_UNIQUE<PhaseFMethodVersionV1>`; `allowed_authority_ids:NONEMPTY_SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `measurand_id:RUNTIME_STABLE_ID_V1`; `result_unit:UNIT_TEXT_V1`; `blinding_requirement:blinded_to_assessment`; `uncertainty_policy:PhaseFUncertaintyPolicyV1`; `lod_loq_policy:PhaseFLODLOQPolicyV1`; `calibration_policy:PhaseFCheckListV1`; `qc_policy:PhaseFCheckListV1`; `chain_of_custody_required:true`; `traceability_document_required:true`; `limitations_document_required:true` |
| `PhaseFMethodVersionV1` | `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1` |
| `PhaseFMetrologyCheckSpecV1` | `check_id:RUNTIME_STABLE_ID_V1`; `check_kind:PHASE_F_CHECK_KIND_V1`; `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `procedure_document:PhaseFObjectReferenceV1`; `measurand_id:RUNTIME_STABLE_ID_V1`; `result_unit:UNIT_TEXT_V1`; `comparator:greater_than_or_equal|less_than_or_equal`; `threshold:RUNTIME_F64_V1`; `failure_action:PHASE_F_DEVIATION_ACTION_V1` |
| `PhaseFMetrologyCheckResultV1` | `schema_version:JSON_INTEGER_ONE`; `check_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `check_id:RUNTIME_STABLE_ID_V1`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `performed_at:UTC_SECOND_TIMESTAMP_V1`; `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `measurand_id:RUNTIME_STABLE_ID_V1`; `value:RUNTIME_F64_V1`; `unit:UNIT_TEXT_V1`; `result:PHASE_F_CHECK_RESULT_V1` |
| `PhaseFReferenceSourceDescriptorV1` | `schema_version:JSON_INTEGER_ONE`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `source_file_sha256:SHA256_V1`; `evidence_origin:EvidenceOriginV1`; `dependency_completeness:ReferenceDependencyCompletenessV1`; `experiment_scope:ArtifactExperimentScope`; `acquisition_families:ArtifactAcquisitionFamilies`; `direct_dependencies:SORTED_UNIQUE<ReferenceDependencyV1>` |
| `PhaseFReferenceResultV1` | `schema_version:JSON_INTEGER_ONE`; `reference_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_id:RUNTIME_STABLE_ID_V1`; `reference_endpoint_id:RUNTIME_STABLE_ID_V1`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `reference_source_descriptor_sha256:SHA256_V1`; `reference_type:PHASE_F_REFERENCE_TYPE_V1`; `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `blinding_state:BlindingStateV1`; `uncertainty:PhaseFQuantifiedUncertaintyV1`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>`; `limitations_document_sha256:SHA256_V1`; `traceability_document_sha256:SHA256_V1`; `chain_of_custody_sha256:SHA256_V1`; mechanism branch adds `hypothesis_id:RUNTIME_STABLE_ID_V1,outcome:supports|contradicts|not_assessed|unavailable`; health branch adds `target:HealthTargetV1,label:RUNTIME_CANONICAL_TEXT_V1` |
| `PhaseFScientificAdmissibilityAuditV1` | `schema_version:JSON_INTEGER_ONE`; `scientific_admissibility_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `protocol_sha256:SHA256_V1`; `package_manifest_sha256:SHA256_V1`; `dependency_audit_sha256:SHA256_V1`; `identity_audit_sha256:SHA256_V1`; `reference_assessments:NONEMPTY_SORTED_UNIQUE<PhaseFReferenceAssessmentV1>`; `reviewer_role:scientific_metrology`; `result:PHASE_F_RESULT_V1` |
| `PhaseFReferenceAssessmentV1` | `reference_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_id:RUNTIME_STABLE_ID_V1`; `evidence_category:SCIENTIFIC_EVIDENCE_CATEGORY_V1`; `claim_ceiling:SCIENTIFIC_CLAIM_CEILING_V1`; `dependency_status:known_separated|known_overlap|unknown`; `identity_status:distinct|same|unknown`; `admissibility:physical_support_allowed|limited_only|not_assessed|unavailable|not_admissible` |
| `PhaseFCohortLockRecordV1` | `schema_version:JSON_INTEGER_ONE`; `cohort_lock_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `protocol_sha256:SHA256_V1`; `package_manifest_sha256:SHA256_V1`; `power_analysis_sha256:SHA256_V1`; `dependency_audit_sha256:SHA256_V1`; `physical_unit_ledger_sha256:SHA256_V1`; `identity_audit_sha256:SHA256_V1`; `location_ledger_sha256:SHA256_V1`; `chain_of_custody_sha256:SHA256_V1`; `deviation_ledger_sha256:SHA256_V1`; `metrology_policy_sha256:SHA256_V1`; `scientific_admissibility_audit_sha256:SHA256_V1`; `reference_result_sha256s:SORTED_UNIQUE<SHA256_V1>`; `reference_source_descriptor_sha256s:SORTED_UNIQUE<SHA256_V1>`; `locked_at:UTC_SECOND_TIMESTAMP_V1` |
| `PhaseFExecutionRecordV1` | `schema_version:JSON_INTEGER_ONE`; `execution_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `cohort_lock_record_sha256:SHA256_V1`; `owner_approval_file_sha256:SHA256_V1`; `protocol_sha256:SHA256_V1`; `deviation_ledger_sha256:SHA256_V1`; `release_code_sha:GIT_SHA_V1`; `checker_binary_sha256:SHA256_V1`; `validation_manifest_sha256:SHA256_V1`; `started_at:UTC_SECOND_TIMESTAMP_V1`; `completed_at:UTC_SECOND_TIMESTAMP_V1`; `result:PHASE_F_RESULT_V1` |
| `PhaseFReleaseRecordV1` | `schema_version:JSON_INTEGER_ONE`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `claim_statement:RUNTIME_CANONICAL_TEXT_V1`; `release_code_sha:GIT_SHA_V1`; `protocol_sha256:SHA256_V1`; `cohort_lock_record_sha256:SHA256_V1`; `owner_approval_file_sha256:SHA256_V1`; `execution_record_sha256:SHA256_V1`; `validation_manifest_sha256:SHA256_V1`; `monitoring_policy_sha256:SHA256_V1`; `metrology_policy_sha256:SHA256_V1`; `valid_from:UTC_SECOND_TIMESTAMP_V1`; `valid_until:UTC_SECOND_TIMESTAMP_V1`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>` |
| `PhaseFClaimStateRecordV1` | `schema_version:JSON_INTEGER_ONE`; `claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `previous_claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1|null`; `state:PHASE_F_CLAIM_STATE_V1`; `reason_code:PHASE_F_CLAIM_REASON_V1`; `effective_at:UTC_SECOND_TIMESTAMP_V1`; `superseding_release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1|null`; `activation_review_bundle_sha256:SHA256_V1|null`; `reinstatement_approval_sha256:SHA256_V1|null`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>` |
| `PhaseFReinstatementApprovalV1` | `schema_version:JSON_INTEGER_ONE`; `reinstatement_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `suspended_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `suspension_reason:PHASE_F_CLAIM_REASON_V1`; `required_corrective_action:RUNTIME_CANONICAL_TEXT_V1`; `corrective_evidence_sha256s:SORTED_UNIQUE<SHA256_V1>`; `execution_record_sha256:SHA256_V1`; `review_bundle_sha256:SHA256_V1` |
| `PhaseFMonitoringPolicyV1` | `schema_version:JSON_INTEGER_ONE`; `monitoring_policy_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `monitoring_interval_seconds:DURATION_SECONDS_V1`; `required_metrics:FIXED_ORDER<PHASE_F_MONITORING_METRIC_V1>`; `metric_thresholds:SORTED_UNIQUE<PhaseFMetricThresholdV1>`; `missing_monitoring_action:suspend`; `domain_breach_action:suspend`; `reference_qc_breach_action:suspend` |
| `PhaseFMetricThresholdV1` | `metric_id:PHASE_F_MONITORING_NUMERIC_METRIC_V1`; `comparator:greater_than_or_equal|less_than_or_equal`; `value:RUNTIME_F64_V1`; `unit:UNIT_TEXT_V1|null` |
| `PhaseFMonitoringRecordV1` | `schema_version:JSON_INTEGER_ONE`; `monitoring_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `window_start:UTC_SECOND_TIMESTAMP_V1`; `window_end:UTC_SECOND_TIMESTAMP_V1`; `policy_sha256:SHA256_V1`; `measurements:NONEMPTY_SORTED_UNIQUE<PhaseFMonitoringMeasurementV1>`; `breaches:SORTED_UNIQUE<PhaseFMonitoringBreachV1>`; `result:PHASE_F_MONITORING_RESULT_V1` |
| `PhaseFMonitoringMeasurementV1` | `metric_id:PHASE_F_MONITORING_METRIC_V1`; `value:PhaseFMonitoringValueV1` |
| `PhaseFMonitoringBreachV1` | `metric_id:PHASE_F_MONITORING_METRIC_V1`; `breach_code:PHASE_F_BREACH_CODE_V1`; `evidence_sha256:SHA256_V1` |
| `PhaseFIncidentScopeV1` | tagged union: `{type:"release",release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1}` or `{type:"campaign",campaign_id:RUNTIME_STABLE_ID_V1}` or `{type:"registry_namespace",registry_namespace_id:RUNTIME_STABLE_ID_V1}` |
| `PhaseFIncidentRecordV1` | `schema_version:JSON_INTEGER_ONE`; `incident_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `scope:PhaseFIncidentScopeV1`; `incident_type:PHASE_F_INCIDENT_TYPE_V1`; `detected_at:UTC_SECOND_TIMESTAMP_V1`; `affected_object_sha256s:SORTED_UNIQUE<PhaseFObjectDigestV1>`; `affected_unit_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `evidence_references:SORTED_UNIQUE<PhaseFObjectReferenceV1>`; `required_action:PHASE_F_INCIDENT_ACTION_V1`; `incident_status:PHASE_F_INCIDENT_STATUS_V1` |
| `PhaseFRetentionAuditV1` | `schema_version:JSON_INTEGER_ONE`; `retention_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `policy_sha256:SHA256_V1`; `audited_at:UTC_SECOND_TIMESTAMP_V1`; `object_checks:NONEMPTY_SORTED_UNIQUE<PhaseFRetentionObjectCheckV1>`; `result:PHASE_F_RESULT_V1` |
| `PhaseFRetentionObjectCheckV1` | `object_sha256:PhaseFObjectDigestV1`; `primary_available:BOOLEAN_V1`; `primary_verified:BOOLEAN_V1`; `verified_backup_count:CANONICAL_UNSIGNED_INTEGER_V1`; `last_backup_verification_at:UTC_SECOND_TIMESTAMP_V1`; `result:PHASE_F_RESULT_V1` |
| `PhaseFF5ReleaseCandidateV1` | `schema_version:JSON_INTEGER_ONE`; `f5_candidate_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `release_record_sha256:SHA256_V1`; `initial_claim_state_sha256:SHA256_V1`; `execution_record_sha256:SHA256_V1`; `cohort_lock_record_sha256:SHA256_V1`; `owner_approval_file_sha256:SHA256_V1`; `validation_manifest_sha256:SHA256_V1`; `trust_store_sha256:SHA256_V1`; `release_code_sha:GIT_SHA_V1`; `package_manifest_sha256:SHA256_V1`; `monitoring_policy_sha256:SHA256_V1`; `metrology_policy_sha256:SHA256_V1` |
| `PhaseFRegistryCompromiseEmergencyV1` | `schema_version:JSON_INTEGER_ONE`; `emergency_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `registry_namespace_id:RUNTIME_STABLE_ID_V1`; `incident_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `declared_at:UTC_SECOND_TIMESTAMP_V1`; `affected_claim_ids:NONEMPTY_SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `incident_record_sha256:SHA256_V1`; `review_bundle_sha256:SHA256_V1`; `repository_commit_sha:GIT_SHA_V1`; `repository_blob:GIT_BLOB_V1`; `action:suspend_all_active_claims` |

The aliases used by this audit are defined exactly in §§2, 3, 5, 7, 9-15;
there is no free-form `string`, `integer`, `hash`, `value`, `status`, `policy`,
`record`, `document`, `object`, `relation`, or `role` field. The audit result is
`UNTYPED_NORMATIVE_FIELDS=0`.

## 17. Valid object construction order

| Object | Inputs and canonicalization | ID / signature | Complete-file hash; relation; next |
|---|---|---|---|
| F0 decision bundle | F0 values, JCS, exact 20 IDs | §3; unsigned | hash; review/tag; protocol |
| review bundle | target bytes/commit, five rows, JCS | §3; unsigned | hash; tag `review_bundle_sha256`; approval tag |
| approval tag | exact annotated message/peeled target | no ID/signature | `git_tag_message` hash; preceding refs; next gate |
| checker readiness | two fresh builds/closed env/transcript | no ID in body | binary hash; readiness review/tag; enrollment |
| authority enrollment | F0 IDs, readiness, key bytes, JCS | §3; intentionally unsigned | hash; enrollment tag; genesis |
| registry genesis | enrollment/F0 authority, sequence 0 | §8 signing bytes | signed hash; authority_enrolled; protocol |
| protocol registration | exact TOML/registration document | runtime protocol ID | hash; protocol_registered; power |
| power method/analysis | reviewed interface then typed values | §3 IDs | hashes; power_registered; package |
| package/physical/custody/metrology | retrieved objects, role matrix, audits | §3 IDs | hashes; package relations; cohort |
| scientific audit | package/dependency/identity/ref assessments | §3 audit ID | hash; scientific_admissibility; cohort |
| cohort lock | exact prior hashes/lock time | §3 cohort ID | hash; cohort_locked; owner approval |
| owner approval | owner approval/cohort/enrollment | owner approval ID | hash; owner_approval_registered; execution |
| F3 trust provisioning | reviewed store blob/file/hash | no new root in plan | hashes; trust approval; F4 |
| F4 execution | locked cohort/approval/latest deviation/checker | §3 execution ID | hash; execution_registered; release |
| release record | semantic release payload excluding only `release_record_id` | §3 release ID | complete release-file hash; later `release_registered` subject is release ID/file hash; state |
| F5 candidate | frozen release/evidence hashes and exact candidate fields | §5.1 candidate ID | complete candidate-file hash; five-role F5 review bundle |
| initial active state | release/no prior/initial_release plus exact F5 `activation_review_bundle_sha256` | §3 claim-state ID | complete state-file hash; later `claim_state_changed` subject is state ID/file hash |
| monitoring pass | correct window/all metrics/recomputed pass | §3 monitoring ID | complete monitoring-file hash; later `monitoring_recorded` subject is monitoring ID/file hash |
| incident | verified evidence/exact consequence | §3 incident ID | hash; incident_recorded; suspension |
| suspension | incident/monitoring evidence/legal transition | §3 state ID | pointer then hash; state; remediation |
| reinstatement | permitted trigger/five GO/corrective evidence | §3 reinstatement ID | hash; state dependency; active state |
| retention audit | object hashes/primary-backup verification/policy | §3 audit ID | hash; retention_audited; next audit |

No step signs enrollment, trusts a tag pusher, binds a live response to URI
hash, or treats a future tag or registry record as an input to the initial
claim-state file. Claim-status requires current head, chain, release, initial
state, and F5 review authority; the final tag is created only after the release
and state registrations. Initial ACTIVE may exist during its
grace period before the first monitoring window is due; an accepted monitoring
PASS is required only by the first due boundary and thereafter by each due
boundary. Release/state/monitoring files are complete before their external
registry attestations, so the positive-path graph is acyclic.

The final physical-release tag is created only after both the release file and
initial ACTIVE state file exist, each has its corresponding registry record, and
the final review bundle is complete. It binds independently:
`release_record_id`, `release_file_sha256`, `release_registry_record_sha256`,
`initial_claim_state_record_id`, `initial_claim_state_file_sha256`,
`initial_claim_state_registry_record_sha256`, and `review_bundle_sha256`.

## 18. R6 positive controls and complete DAG construction audit

The foundational external-authority DAG is frozen as:

```text
semantic object
  -> complete immutable file bytes
  -> complete file SHA-256
  -> signed registry record attesting that file
  -> registry-record SHA-256
  -> optional later tag or object referencing the registry record
```

The subject never contains the hash, sequence, or signature of its attesting
registry record. R1-CX-02 is therefore constructible with concrete symbolic
hashes: release file `H1` -> `release_registered` subject `H1` -> registry
record `H2`, with no `H2` in the release; state file `H3` ->
`claim_state_changed` subject `H3` -> registry record `H4`, with no `H4` in
the state; the final tag binds `H1,H2,H3,H4` separately.

The complete positive path is: plan file -> five-role plan bundle -> plan tag;
F0 decision bundle -> five-role F0 bundle -> F0 tag; two build-evidence files
-> readiness evidence -> five-role readiness bundle -> readiness tag;
enrollment -> five-role enrollment bundle -> enrollment tag -> signed genesis;
protocol, power interface, completed power analysis, metrology policy/checks,
reference descriptors/results, unit/identity/location/custody ledgers,
deviation revision, dependency audit, scientific admissibility audit, package
manifest -> package registry record -> cohort lock -> cohort registry record;
owner approval -> approval registry record -> trust candidate -> five-role
trust bundle -> trust tag; execution -> execution registry record; release file
-> F5 candidate -> five-role F5 bundle -> final initial ACTIVE state -> release
registry record -> state registry record -> final release tag; monitoring PASS
-> monitoring registry record -> live fresh head -> ACTIVE. No step requires a
future object hash, future tag, future registry record, or undefined decision.

The ten mandatory positive-path probes are constructible before counterexample
review: release, state, and monitoring files each complete before their own
registry record; deviation revision ID is one-way from stable ledger ID;
F5 review precedes initial-state registration; initial ACTIVE precedes first
monitoring due; and a reference result projects exactly to the current runtime
endpoint. Failure of any probe is P1.

Monitoring KAT: for initial ACTIVE at `T0` with interval `3600`, no record at
`T0+100`, `T0+1800`, and `T0+3599` is CURRENT; no accepted PASS at `T0+3600` is OVERDUE
and NOT_ACTIVE. If a PASS window ends at `T0+3500`, the next due is exactly
`T0+7100` using SI-second arithmetic. A monitoring record is accepted only when
its file, recomputed result, registry record, current chain, release/policy,
continuity, and due boundary all validate.

Reference KAT: a mechanism `PhaseFReferenceResultV1` supplies endpoint ID,
reference endpoint ID, source ID, hypothesis, outcome, method/version,
authority, blinding, quantified uncertainty, and limitations. The runtime
projection contains exactly the corresponding current `ReferenceEndpointV1`
fields. It contains no measurement value or unit; those remain metrology
provenance only.

Review KAT: five rows each equal `GO,0,0` produce aggregate counts `0,0` and
aggregate `GO`. Mutating only aggregate decision to `NO-GO` invalidates the
bundle; mutating one row to `NO-GO` requires aggregate `NO-GO`. A pre-release
abandonment incident uses campaign scope and needs no release ID. A post-release
monitoring breach uses release scope; until its suspension state is present,
claim-status is NOT_ACTIVE even if the latest ACTIVE state remains in the
registry.

## 19. Historical regression repair and traceability substance

The cumulative historical table is the complete case accounting. The existing
case rows in §20 are the authoritative immutable descriptions: `case_id` is the
case label, `origin_version` is its prefix (`R1`, `R2`, `R3`, `R3R`, `R4`, `R5`,
or `R6`), `origin_expected_result` is the stated deterministic result, and R6
copies that result into `R6_expected_result` unless a row below explicitly marks
it superseded. `superseded_by` is `none` for retained cases.
`R6_status` is exactly the closed enum `retained|superseded`.

| case_id | origin_version | origin_expected_result | R6_status | R6_expected_result | superseded_by |
|---|---|---|---|---|---|
| R1-CX-01 | R1 | software outcome; physical NOT_ACTIVE | retained | software outcome; physical NOT_ACTIVE | none |
| R1-CX-02 | R1 | PASS; ACTIVE positive control | retained | PASS; ACTIVE positive control | none |
| R1-CX-03 | R1 | hard error before dataset | retained | hard error before dataset | none |
| R1-CX-04 | R1 | NO-GO; evidence ceiling | retained | NO-GO; evidence ceiling | none |
| R1-CX-05 | R1 | hard error | retained | hard error | none |
| R1-CX-06 | R1 | hard error | retained | hard error | none |
| R1-CX-07 | R1 | existing exclusion/DNP | retained | existing exclusion/DNP | none |
| R1-CX-08 | R1 | UNKNOWN/NO-GO | retained | UNKNOWN/NO-GO | none |
| R1-CX-09 | R1 | existing Indeterminate | retained | existing Indeterminate | none |
| R1-CX-10 | R1 | eligible / excluded boundary | retained | eligible / excluded boundary | none |
| R1-CX-11 | R1 | hard error/no publication | retained | hard error/no publication | none |
| R1-CX-12 | R1 | byte-identical governed outputs | retained | byte-identical governed outputs | none |
| R2-CX-01 | R2 | semantic mismatch NO-GO | retained | semantic mismatch NO-GO | none |
| R2-CX-02 | R2 | fingerprint/file/tag mismatch | retained | fingerprint/file/tag mismatch | none |
| R2-CX-03 | R2 | complete-file hash mismatch | retained | complete-file hash mismatch | none |
| R2-CX-04 | R2 | forbidden | retained | forbidden | none |
| R2-CX-05 | R2 | NO-GO | retained | NO-GO | none |
| R2-CX-06 | R2 | NO-GO | retained | NO-GO | none |
| R2-CX-07 | R2 | NO-GO/AUTHORITY_UNAVAILABLE | retained | NO-GO/AUTHORITY_UNAVAILABLE | none |
| R2-CX-08 | R2 | alias/NO-GO | retained | alias/NO-GO | none |
| R2-CX-09 | R2 | interface mismatch NO-GO | retained | interface mismatch NO-GO | none |
| R2-CX-10 | R2 | NO-GO | retained | NO-GO | none |
| R2-CX-11 | R2 | NO-GO | retained | NO-GO | none |
| R2-CX-12 | R2 | NOT_ACTIVE | retained | NOT_ACTIVE | none |
| R2-CX-13 | R2 | forbidden/readiness NO-GO | retained | forbidden/readiness NO-GO | none |
| R2-CX-14 | R2 | NO-GO | retained | NO-GO | none |
| R2-CX-15 | R2 | exact rational PASS | retained | exact rational PASS | none |
| R3-CX-01 | R3 | PASS, exact case preserved | retained | PASS, exact case preserved | none |
| R3-CX-02 | R3 | unrepresentable NO-GO | retained | unrepresentable NO-GO | none |
| R3-CX-03 | R3 | invariant NO-GO | retained | invariant NO-GO | none |
| R3-CX-04 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-05 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-06 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-07 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-08 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-09 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-10 | R3 | AUTHORITY_UNAVAILABLE | retained | AUTHORITY_UNAVAILABLE | none |
| R3-CX-11 | R3 | NOT_ACTIVE | retained | NOT_ACTIVE | none |
| R3-CX-12 | R3 | suspend; NOT_ACTIVE | retained | suspend; NOT_ACTIVE | none |
| R3-CX-13 | R3 | NO-GO/suspend | retained | NO-GO/suspend | none |
| R3-CX-14 | R3 | UNKNOWN/no count/NO-GO | retained | UNKNOWN/no count/NO-GO | none |
| R3-CX-15 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-16 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-17 | R3 | invalid transition/NO-GO | retained | invalid transition/NO-GO | none |
| R3-CX-18 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-19 | R3 | NO-GO | retained | NO-GO | none |
| R3-CX-20 | R3 | NO-GO | retained | NO-GO | none |
| R3R-CX-01 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-02 | R3R | NO-GO; ceiling | retained | NO-GO; ceiling | none |
| R3R-CX-03 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-04 | R3R | NO-GO; principal non-authority | superseded | pusher identity is irrelevant if tag bytes/target/review authority validate | R6-CX-28 |
| R3R-CX-05 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-06 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-07 | R3R | NO-GO; wrong hash meaning | retained | NO-GO; wrong hash meaning | none |
| R3R-CX-08 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-09 | R3R | AUTHORITY_UNAVAILABLE | retained | AUTHORITY_UNAVAILABLE | none |
| R3R-CX-10 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-11 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-12 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-13 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-14 | R3R | NO-GO | retained | NO-GO | none |
| R3R-CX-15 | R3R | NO-GO | retained | NO-GO | none |
| R4-CX-01 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-02 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-03 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-04 | R4 | NO-GO despite sum | retained | NO-GO despite sum | none |
| R4-CX-05 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-06 | R4 | equivocation; NOT_ACTIVE | retained | equivocation; NOT_ACTIVE | none |
| R4-CX-07 | R4 | AUTHORITY_UNAVAILABLE | retained | AUTHORITY_UNAVAILABLE | none |
| R4-CX-08 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-09 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-10 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-11 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-12 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-13 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-14 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-15 | R4 | NO-GO; NOT_ACTIVE | retained | NO-GO; NOT_ACTIVE | none |
| R4-CX-16 | R4 | NO-GO; NOT_ACTIVE | retained | NO-GO; NOT_ACTIVE | none |
| R4-CX-17 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-18 | R4 | overdue; NOT_ACTIVE | retained | overdue; NOT_ACTIVE | none |
| R4-CX-19 | R4 | NO-GO | retained | NO-GO | none |
| R4-CX-20 | R4 | AUTHORITY_UNAVAILABLE | retained | AUTHORITY_UNAVAILABLE | none |
| R5-CX-01 | R5 | tag validity unchanged; pusher non-authoritative | retained | tag validity unchanged; pusher non-authoritative | none |
| R5-CX-02 | R5 | schema reject | retained | schema reject | none |
| R5-CX-03 | R5 | reject | retained | reject | none |
| R5-CX-04 | R5 | readiness build invalid | retained | readiness build invalid | none |
| R5-CX-05 | R5 | checker evidence invalid | retained | checker evidence invalid | none |
| R5-CX-06 | R5 | reject | retained | reject | none |
| R5-CX-07 | R5 | reject | retained | reject | none |
| R5-CX-08 | R5 | reject | retained | reject | none |
| R5-CX-09 | R5 | alias/NO-GO | retained | alias/NO-GO | none |
| R5-CX-10 | R5 | NO-GO | retained | NO-GO | none |
| R5-CX-11 | R5 | NO-GO | retained | NO-GO | none |
| R5-CX-12 | R5 | reject | retained | reject | none |
| R5-CX-13 | R5 | schema reject | retained | schema reject | none |
| R5-CX-14 | R5 | reject | retained | reject | none |
| R5-CX-15 | R5 | reject | retained | reject | none |
| R5-CX-16 | R5 | reject | retained | reject | none |
| R5-CX-17 | R5 | reject | retained | reject | none |
| R5-CX-18 | R5 | reject | retained | reject | none |
| R5-CX-19 | R5 | breach; suspend | retained | breach; suspend | none |
| R5-CX-20 | R5 | invalid; suspension required | retained | invalid; suspension required | none |
| R5-CX-21 | R5 | not an accepted window | retained | not an accepted window | none |
| R5-CX-22 | R5 | retention failure; NOT_ACTIVE | retained | retention failure; NOT_ACTIVE | none |
| R5-CX-23 | R5 | emergency path; NOT_ACTIVE | retained | emergency path; NOT_ACTIVE | none |
| R5-CX-24 | R5 | NOT_ACTIVE | retained | NOT_ACTIVE | none |
| R5-CX-25 | R5 | reject | retained | reject | none |
| R6-CX-01 | R6 | schema reject; field does not exist | retained | schema reject; field does not exist | none |
| R6-CX-02 | R6 | reject | retained | reject | none |
| R6-CX-03 | R6 | schema reject | retained | schema reject | none |
| R6-CX-04 | R6 | schema reject | retained | schema reject | none |
| R6-CX-05 | R6 | reject; stable runtime ID required | retained | reject; stable runtime ID required | none |
| R6-CX-06 | R6 | reject | retained | reject | none |
| R6-CX-07 | R6 | invalid bundle | retained | invalid bundle | none |
| R6-CX-08 | R6 | invalid bundle | retained | invalid bundle | none |
| R6-CX-09 | R6 | reject | retained | reject | none |
| R6-CX-10 | R6 | reject | retained | reject | none |
| R6-CX-11 | R6 | invalid specification; token removed | retained | invalid specification; token removed | none |
| R6-CX-12 | R6 | reject | retained | reject | none |
| R6-CX-13 | R6 | reject | retained | reject | none |
| R6-CX-14 | R6 | reject; integer plus positive range required | retained | reject; integer plus positive range required | none |
| R6-CX-15 | R6 | reject; field does not exist | retained | reject; field does not exist | none |
| R6-CX-16 | R6 | reject | retained | reject | none |
| R6-CX-17 | R6 | reject | retained | reject | none |
| R6-CX-18 | R6 | reject; fields removed | retained | reject; fields removed | none |
| R6-CX-19 | R6 | reject | retained | reject | none |
| R6-CX-20 | R6 | reject; F5 bundle is authority | retained | reject; F5 bundle is authority | none |
| R6-CX-21 | R6 | ACTIVE | retained | ACTIVE | none |
| R6-CX-22 | R6 | reject; exactly five required | retained | reject; exactly five required | none |
| R6-CX-23 | R6 | reject | retained | reject | none |
| R6-CX-24 | R6 | reject | retained | reject | none |
| R6-CX-25 | R6 | reject; campaign scope has no release ID | retained | reject; campaign scope has no release ID | none |
| R6-CX-26 | R6 | reject; only `repository_blob` exists | retained | reject; only `repository_blob` exists | none |
| R6-CX-27 | R6 | reject; five-role bundle required | retained | reject; five-role bundle required | none |
| R6-CX-28 | R6 | mark superseded; pusher is irrelevant | retained | mark superseded; pusher is irrelevant | none |
| R6-CX-29 | R6 | reject | retained | reject | none |
| R6-CX-30 | R6 | reject as noncanonical unless ASCII tuple-sorted | retained | reject as noncanonical unless ASCII tuple-sorted | none |

No historical case is deleted. The R3R-CX-04 change is an intentional,
reviewed architecture supersession, not an unaccounted result change.

| Historical finding | R5 section | Requirement / AC / test / evidence | Author disposition |
|---|---|---|---|
| R1 real manifest checker | §7, §10, §16 | R5-05/R5-11; AC5-05/11; T5-05/11; EV5-05/11 | REMEDIATED |
| R1 physical-unit/pseudoreplication | §10-11 | R5-13/R5-14/R5-16; AC5-13/14/16; T5-13/14/16; EV5-13/14/16 | REMEDIATED |
| R1 power | §12 | R5-18/R5-19; AC5-18/19; T5-18/19; EV5-18/19 | REMEDIATED |
| R1 metrology | §13 | R5-20/R5-22; AC5-20/22; T5-20/22; EV5-20/22 | REMEDIATED |
| R1 operational state | §14-15 | R5-29/R5-32/R5-33; AC5-29/32/33; T5-29/32/33; EV5-29/32/33 | REMEDIATED |
| R2 enrollment approval | §5.1-6 | R5-02/R5-04; AC5-02/04; T5-02/04; EV5-02/04 | REMEDIATED |
| R2 checker source/binary binding | §7 | R5-05/R5-36; AC5-05/36; T5-05/36; EV5-05/36 | REMEDIATED |
| R2 monitoring/deviation restoration | §11, §14-15 | R5-17/R5-31/R5-32; AC5-17/31/32; T5-17/31/32; EV5-17/31/32 | REMEDIATED |

The R6 requirements and traceability rows are the canonical operational
requirements for this document. Each has one primary AC, one test, and one
evidence item with an executable KAT/transcript and deterministic oracle. Every
F-OD-01 through F-OD-20 is mapped below; no additional owner decision exists.

### 19.1 Executable R6 requirement traceability

Every row below is normative and contains the complete requirement-to-path
mapping. `owner_decision_ids` is `none` only when no F0 decision is involved.

| requirement ID | normative statement | owner-decision IDs | external schemas | production/checker path | primary AC ID | test ID(s) | evidence ID(s) | review role | stage |
|---|---|---|---|---|---|---|---|---|---|
| `F-PLAN-R6-P1-01` | External authority is an acyclic semantic-object -> complete-file-hash -> registry-attestation graph; release, state, monitoring, and deviation identities have no reverse pointer. | F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-05,F-OD-06,F-OD-07,F-OD-08,F-OD-09,F-OD-10 | ReleaseRecord, ClaimStateRecord, MonitoringRecord, RegistryRecord, DeviationLedgerRevision | canonicalize/hash/register each file, then bind later registry hash or tag | AC6-01 | T6-01 | EV6-01 | architecture_data | F1-F5 |
| `F-PLAN-R6-P1-02` | Review bundles contain exactly five unique roles and derive aggregate counts/decision by the mandatory bidirectional sum rule; tags contain only the bundle hash and GO. | none | IndependentReviewBundle, six approval tag bodies | review bundle validator and tag validator | AC6-02 | T6-02 | EV6-02 | architecture_data | all gates |
| `F-PLAN-R6-P1-03` | Two fresh checker builds and readiness evidence bind exact source/tree/lock/toolchain/platform, transcript, binary, and closed maintenance status; command object and argv are equivalent. | none | CheckerBuildEvidence, CheckerReadinessEvidence, CheckerReport, Command, Argv | readiness gate and checker report route | AC6-03 | T6-03 | EV6-03 | compatibility | readiness |
| `F-PLAN-R6-P1-04` | Registry subject hashes mean complete subject-file bytes, relations are explicitly typed and ASCII-tuple sorted, and package/retention relation sets are exact. | F-OD-14,F-OD-17,F-OD-20 | RegistryRecord, RegistryRelation, PackageManifest, RetentionAudit | registry chain and package/retention registration | AC6-04 | T6-04 | EV6-04 | security | F1-F5 |
| `F-PLAN-R6-P1-05` | Power interface defines all parameter/output types, units, ranges, and sensitivities before analysis construction; integer-positive values use `integer` plus positive range. | F-OD-12 | PowerMethodInterface, PowerAnalysisRecord | interface retrieval, validation, execution, analysis registration | AC6-05 | T6-05 | EV6-05 | scientific_metrology | F1 |
| `F-PLAN-R6-P1-06` | Metrology check results are full schema-versioned external objects and LOD/LOQ/result units match by exact bytes with no checker conversion. | F-OD-11 | MetrologyPolicy, MetrologyCheckSpec, MetrologyCheckResult | metrology policy/check construction and package binding | AC6-06 | T6-06 | EV6-06 | scientific_metrology | F0-F2 |
| `F-PLAN-R6-P1-07` | Adjudicated reference results contain only fields projectable to current runtime endpoints plus external provenance; measurement values/units never enter runtime projection. | F-OD-05,F-OD-06,F-OD-07 | ReferenceSourceDescriptor, ReferenceResult, ReferenceEndpoint | reference adjudication and total runtime projection | AC6-07 | T6-07 | EV6-07 | scientific_metrology | F2 |
| `F-PLAN-R6-P1-08` | Every rotation trigger has one exact action row; initial ACTIVE is authorized by the completed five-role F5 candidate review, and compromise/reinstatement follows the exact transition matrix. | F-OD-16 | F5ReleaseCandidate, ClaimStateRecord, ReinstatementApproval | F5 activation and claim-state transitions | AC6-08 | T6-08 | EV6-08 | operations_governance | F3-F5+ |
| `F-PLAN-R6-P1-09` | Monitoring has five numeric thresholds, exact breach-set/result derivation, due-boundary windows, and a pre-first-window grace period. | F-OD-19 | MonitoringPolicy, MonitoringRecord | monitoring producer, registry, currentness evaluator | AC6-09 | T6-09 | EV6-09 | operations_governance | F5+ |
| `F-PLAN-R6-P1-10` | Retention and incidents use constructible release/campaign/registry scopes; pre-release abandonment has no release ID; registry and owner compromise have exact authority paths. | F-OD-13,F-OD-14,F-OD-15,F-OD-17,F-OD-20 | IncidentScope, IncidentRecord, RetentionAudit, RegistryCompromiseEmergency | incident/retention recorder and claim-status fail-closed path | AC6-10 | T6-10 | EV6-10 | security | all stages |
| `F-PLAN-R6-P1-11` | Every external schema, requirement, AC, test, evidence item, owner decision, and historical case is substantively mapped and replayable, including intentional supersession. | F-OD-18 | Master schema catalog, historical case table | plan-review traceability and KAT replay | AC6-11 | T6-11 | EV6-11 | architecture_data | plan review |

Every acceptance criterion contains preconditions, exact inputs, operation,
expected result/output, and a failure oracle:

| AC ID | preconditions | exact input object(s) | exact operation/command | expected exit/result | expected output object | failure oracle |
|---|---|---|---|---|---|---|
| AC6-01 | complete subject files and stable ledger ID | release/state/monitoring files and exact bytes | compute ID/file hash, then construct typed registry record | valid record subject equals complete file hash | registry records and later tag bindings | any back-pointer, semantic/file substitution, or cycle -> NO-GO |
| AC6-02 | five unique role rows | review rows and target hash | validate aggregate and tag bodies | GO only under exact predicate | review bundle and minimal tags | count/decision mismatch or per-role tag field -> invalid |
| AC6-03 | two isolated builds | build evidence and readiness files | verify readiness object and deterministic argv | PASS, exit 0, report agrees | readiness evidence/report | mismatch, omitted transcript, or usage/report disagreement -> reject |
| AC6-04 | typed package/retention relations | package, audit, registry records | canonicalize tuple and validate record kind | sorted, typed relations and complete-file subjects | signed registry records | missing type, wrong kind, duplicate, semantic hash -> reject |
| AC6-05 | approved interface | interface, parameters, sensitivity cases | construct/recompute analysis | analysis ID and registry subject valid | power analysis/record | future ID, unknown value, wrong unit/range, missing case -> reject |
| AC6-06 | endpoint policy | check specs/results and LOD/LOQ | recompute comparator and exact units | required checks pass or exact NO-GO | check-result files | schema/version/conversion/result mismatch -> reject |
| AC6-07 | current runtime endpoint type | adjudicated reference result | project common and branch runtime fields | exact runtime endpoint | `ReferenceEndpointV1` | extra/missing/defaulted field or measurement value/unit -> reject |
| AC6-08 | F5 candidate and five GO rows | candidate, bundle, initial state | construct state then register release/state | ACTIVE valid before first due | state, registry records, tag inputs | future tag/registry dependency or missing trigger -> reject |
| AC6-09 | T0 and 3600-second policy | monitoring files and current head | recompute threshold/breach/window due | CURRENT before due; OVERDUE/NOT_ACTIVE at due | status/report | cardinality, breach, interval, or late-PASS error -> reject |
| AC6-10 | selected campaign/release/registry scope | incident, retention, emergency objects | validate scope and fail-closed authority | closure/suspension constructible | incident/audit/emergency files | release ID in campaign, wrong blob, or weak review -> reject |
| AC6-11 | complete catalog/case tables | requirement/AC/test/evidence/OD/case rows | replay positive controls and counterexamples | all mapping counters zero | traceability report | identifier-only or missing case/OD mapping -> P1 |

Every test contains fixture construction, command/function, exact result, and a
negative mutation:

| test ID | fixture construction | command/function | expected exact result | negative mutation |
|---|---|---|---|---|
| T6-01 | JCS subjects and H1-H4 registry chain | ID/hash and registry validator | H1/H3 subjects verify; H2/H4 remain external | insert H2/H4 into subjects -> reject |
| T6-02 | five GO rows with 0/0 counts | review/tag validator | GO aggregate and minimal tags | flip aggregate or one row -> invalid |
| T6-03 | two isolated transcripts and same binary | build/readiness checker | PASS and report/argv agreement | omit transcript or reorder argv -> reject |
| T6-04 | package/retention relation arrays | registry canonicalizer | exact typed tuple ordering | omit type or add per-object retention relation -> reject |
| T6-05 | complete interface and sensitivity cases | power constructor | analysis registered after review | add future ID or `positive_integer` -> reject |
| T6-06 | checks plus LOD/LOQ | metrology checker | exact unit/comparator pass | change unit or schema version -> reject |
| T6-07 | mechanism and health references | runtime projection | byte-for-byte valid endpoint | add result unit to endpoint -> reject |
| T6-08 | F5 candidate/bundle before state registration | state constructor | initial ACTIVE has no future dependency | state references final tag -> reject |
| T6-09 | T0, 1800, 3599, 3600, 3500-end windows | currentness function | CURRENT/CURRENT/CURRENT/OVERDUE and next due 7100 | four thresholds or omitted breach -> reject |
| T6-10 | campaign, release, emergency incidents | incident/retention checker | scope and fail-closed action valid | campaign has release ID or legacy blob field -> reject |
| T6-11 | all catalog/case/traceability rows | independent R6 review | zero gaps; R3R-CX-04 superseded | delete case or leave OD unmapped -> P1 |

Every evidence item contains a real artifact, producer/authority, immutable
identity, and acceptance oracle:

| evidence ID | real-world artifact | producer/authority | immutable identity | acceptance oracle |
|---|---|---|---|---|
| EV6-01 | canonical subject files and registry records | release/governance/registry authorities | complete file and registry hashes | no back-pointer; subject equals bytes |
| EV6-02 | five-role bundle and tag messages | independent reviewers/Git | bundle SHA and exact tag bytes | aggregate predicate/minimal grammar |
| EV6-03 | build directories, transcripts, binaries, readiness file | checker builder/readiness reviewers | source/tree/lock/toolchain/transcript/binary | fresh isolation, same binary, maintenance closed |
| EV6-04 | package/retention registry records | registry authority | sequence/predecessor/signature | explicit relation table and tuple order |
| EV6-05 | power interface, analysis, outputs | statistician/scientific reviewer | content IDs and complete hashes | types/ranges/cases all validate |
| EV6-06 | laboratory check-result files/policy | metrology laboratory | check-result ID and complete hash | version/comparator/exact units |
| EV6-07 | laboratory reference/result and runtime projection | laboratory/runtime authority | source/result/endpoint bytes | exact projection and external-only measurements |
| EV6-08 | candidate, bundle, state, registry records | reviewers/governance | complete candidate/state/review/registry hashes | activation precedes registration |
| EV6-09 | monitoring files, policy, live heads | operations/registry authority | file/registry hashes and UTC times | thresholds/breaches/due cadence |
| EV6-10 | incident, retention, emergency Git artifacts | operations/security/live remote | complete hashes plus Git commit/blob | scope and compromise action fail closed |
| EV6-11 | R6 plan/catalog/KAT transcripts | independent R6 reviewer | plan SHA/blob and transcript identity | all IDs mapped and cases accounted |

## 20. Cumulative normative counterexamples

Every historical case remains independently replayable. `NO-GO` means checker
failure; public claim is NOT_ACTIVE except exact ACTIVE or
AUTHORITY_UNAVAILABLE. R6 cases are appended, not substituted.

| Case | Exact mutation/input | Deterministic result |
|---|---|---|
| R1-CX-01 | valid software-only request | software outcome; physical NOT_ACTIVE |
| R1-CX-02 | all physical gates exact PASS | PASS; ACTIVE positive control |
| R1-CX-03 | UNPROVISIONED trust | hard error before dataset |
| R1-CX-04 | synthetic/constructed/unknown/test origin | NO-GO; evidence ceiling |
| R1-CX-05 | missing/wrong/duplicate authority signature/key | hard error |
| R1-CX-06 | wrong root/protocol/cohort/claim/endpoint/reference/domain | hard error |
| R1-CX-07 | known holdout overlap | existing exclusion/DNP |
| R1-CX-08 | unknown separation | UNKNOWN/NO-GO |
| R1-CX-09 | record/family/stratum/class below minimum | existing Indeterminate |
| R1-CX-10 | uncertainty at max / next f64 above | eligible / excluded boundary |
| R1-CX-11 | malformed/duplicate JSON, unsafe path, or TOCTOU | hard error/no publication |
| R1-CX-12 | identical rerun | byte-identical governed outputs |
| R2-CX-01 | mutate decision payload, retain ID | semantic mismatch NO-GO |
| R2-CX-02 | substitute enrollment public key | fingerprint/file/tag mismatch |
| R2-CX-03 | review different enrollment file | complete-file hash mismatch |
| R2-CX-04 | start F-IMPL-1 before F0 | forbidden |
| R2-CX-05 | authority self-appoints outside F0 | NO-GO |
| R2-CX-06 | real manifest uses alternate human parser | NO-GO |
| R2-CX-07 | broken predecessor/gap/fork/rollback | NO-GO/AUTHORITY_UNAVAILABLE |
| R2-CX-08 | same material under two unit IDs | alias/NO-GO |
| R2-CX-09 | undeclared power parameter | interface mismatch NO-GO |
| R2-CX-10 | missing LOD/LOQ/QC/custody authority | NO-GO |
| R2-CX-11 | alternate release serialization | NO-GO |
| R2-CX-12 | old tag valid, latest state suspended | NOT_ACTIVE |
| R2-CX-13 | temporary P2 waiver | forbidden/readiness NO-GO |
| R2-CX-14 | decision cannot project | NO-GO |
| R2-CX-15 | rational allocation sum 1, f64 sum differs | exact rational PASS |
| R3-CX-01 | uppercase runtime-valid ID | PASS, exact case preserved |
| R3-CX-02 | pair-only A/X and B/Y authorization | unrepresentable NO-GO |
| R3-CX-03 | allow unblinded physical reference | invariant NO-GO |
| R3-CX-04 | alternate mechanism critical policy | NO-GO |
| R3-CX-05 | duplicate predicate axis | NO-GO |
| R3-CX-06 | rate rule lacks exact `RateTargetV1` | NO-GO |
| R3-CX-07 | f64 decimal/bits disagree | NO-GO |
| R3-CX-08 | checker source correct, binary differs | NO-GO |
| R3-CX-09 | enrollment tag valid, file unavailable | NO-GO |
| R3-CX-10 | valid chain, resolver unavailable | AUTHORITY_UNAVAILABLE |
| R3-CX-11 | historical release, latest suspended | NOT_ACTIVE |
| R3-CX-12 | missed monitoring interval | suspend; NOT_ACTIVE |
| R3-CX-13 | undocumented post-lock deviation | NO-GO/suspend |
| R3-CX-14 | distinct IDs, distinctness unproved | UNKNOWN/no count/NO-GO |
| R3-CX-15 | QC PASS but result unprojectable | NO-GO |
| R3-CX-16 | release digest valid, wrong registry pointer | NO-GO |
| R3-CX-17 | withdrawn→active | invalid transition/NO-GO |
| R3-CX-18 | external-valid/runtime-invalid ID projected | NO-GO |
| R3-CX-19 | unsupported temperature boundary | NO-GO |
| R3-CX-20 | required health partition omitted | NO-GO |
| R3R-CX-01 | schema uses undefined `canonical_text` | NO-GO |
| R3R-CX-02 | model-derived category supports physical | NO-GO; ceiling |
| R3R-CX-03 | temperature lower is zero | NO-GO |
| R3R-CX-04 | unauthorized principal creates plan tag | NO-GO; principal non-authority |
| R3R-CX-05 | build reuses dirty source checkout | NO-GO |
| R3R-CX-06 | checker uses repository root Cargo.lock | NO-GO |
| R3R-CX-07 | enrollment payload hash put in tag | NO-GO; wrong hash meaning |
| R3R-CX-08 | `depends_on` has wrong object kind | NO-GO |
| R3R-CX-09 | cached head used after expiry | AUTHORITY_UNAVAILABLE |
| R3R-CX-10 | same bytes duplicated by object IDs | NO-GO |
| R3R-CX-11 | `other` identity basis lacks document hash | NO-GO |
| R3R-CX-12 | custody source location absent | NO-GO |
| R3R-CX-13 | old deviation event mutated | NO-GO |
| R3R-CX-14 | power method has prose-only range | NO-GO |
| R3R-CX-15 | suspended→active cites prose evidence | NO-GO |
| R4-CX-01 | undefined primitive token | NO-GO |
| R4-CX-02 | runtime canonical text contains CR | NO-GO |
| R4-CX-03 | external URI scheme unapproved | NO-GO |
| R4-CX-04 | allocations `-0.1,0.5,0.6` | NO-GO despite sum |
| R4-CX-05 | relation kind mismatches type | NO-GO |
| R4-CX-06 | same sequence/different head hash | equivocation; NOT_ACTIVE |
| R4-CX-07 | registry head expired | AUTHORITY_UNAVAILABLE |
| R4-CX-08 | two builds use different lock bytes | NO-GO |
| R4-CX-09 | identity basis lacks document hash | NO-GO |
| R4-CX-10 | aliquot lacks child unit | NO-GO |
| R4-CX-11 | revision changes old event byte | NO-GO |
| R4-CX-12 | required power parameter absent | NO-GO |
| R4-CX-13 | result hypothesis differs from protocol | NO-GO |
| R4-CX-14 | source descriptor incomplete | NO-GO |
| R4-CX-15 | suspended→active lacks reinstatement | NO-GO; NOT_ACTIVE |
| R4-CX-16 | higher sequence regresses effective-at | NO-GO; NOT_ACTIVE |
| R4-CX-17 | monitoring hash encoded quantity | NO-GO |
| R4-CX-18 | now equals due timestamp | overdue; NOT_ACTIVE |
| R4-CX-19 | trust review SHA right, blob differs | NO-GO |
| R4-CX-20 | historical release, live head unavailable | AUTHORITY_UNAVAILABLE |
| R5-CX-01 | different Git writer pushes exact body/target/review bundle | tag validity unchanged; pusher non-authoritative |
| R5-CX-02 | enrollment contains owner/registry signatures | schema reject |
| R5-CX-03 | signature passes non-strict but fails `verify_strict` | reject |
| R5-CX-04 | `CARGO_HOME` contains `config.toml` | readiness build invalid |
| R5-CX-05 | exit 0 with report `decision=no_go` | checker evidence invalid |
| R5-CX-06 | immutable-valid but non-HTTPS live resolver | reject |
| R5-CX-07 | `test_only=true,physical=true` | reject |
| R5-CX-08 | physical raw object has no unit binding | reject |
| R5-CX-09 | two unit IDs share issuer/native key | alias/NO-GO |
| R5-CX-10 | custody source differs from prior destination | NO-GO |
| R5-CX-11 | destroyed unit later measured | NO-GO |
| R5-CX-12 | action `campaign_no_go`, resolution `resolved_no_effect` | reject |
| R5-CX-13 | power unit rule `owner_selected_exact` | schema reject |
| R5-CX-14 | sensitivity override undeclared | reject |
| R5-CX-15 | declared calibration pass fails threshold math | reject |
| R5-CX-16 | reference result omits `reference_endpoint_id` | reject |
| R5-CX-17 | reference result duplicate limitation | reject |
| R5-CX-18 | new-release trigger attempts old-release reinstatement | reject |
| R5-CX-19 | domain status `unknown` | breach; suspend |
| R5-CX-20 | monitoring pass but threshold fails | invalid; suspension required |
| R5-CX-21 | monitoring pass not registry-bound | not an accepted window |
| R5-CX-22 | retention object disappears before deadline | retention failure; NOT_ACTIVE |
| R5-CX-23 | registry compromised while latest head says active | emergency path; NOT_ACTIVE |
| R5-CX-24 | incident exists without suspension state | NOT_ACTIVE |
| R5-CX-25 | same byte object appears under two package IDs | reject |
| R6-CX-01 | release file contains `registry_record_sha256` | schema reject; field does not exist |
| R6-CX-02 | release registry subject uses semantic ID instead of complete file hash | reject |
| R6-CX-03 | claim-state file contains its attesting registry hash | schema reject |
| R6-CX-04 | monitoring file contains its attesting registry hash | schema reject |
| R6-CX-05 | deviation ledger ID is supplied as a content-derived digest | reject; stable runtime ID required |
| R6-CX-06 | prior deviation event is mutated in a later revision | reject |
| R6-CX-07 | five review rows are GO but aggregate is NO-GO | invalid bundle |
| R6-CX-08 | one review row is NO-GO but aggregate is GO | invalid bundle |
| R6-CX-09 | build evidence omits build transcript SHA | reject |
| R6-CX-10 | command object maps to different argv ordering | reject |
| R6-CX-11 | usage error references `USAGE_CODE_V1` | invalid specification; token removed |
| R6-CX-12 | package relation omits explicit relation type | reject |
| R6-CX-13 | retention record uses per-object package-manifest relation | reject |
| R6-CX-14 | power output uses `value_type=positive_integer` | reject; integer plus positive range required |
| R6-CX-15 | F-OD-12 includes future power-analysis ID | reject; field does not exist |
| R6-CX-16 | metrology check result omits schema version | reject |
| R6-CX-17 | LOD unit differs from LOQ or endpoint result unit | reject |
| R6-CX-18 | reference result contains `result_value` or `result_unit` | reject; fields removed |
| R6-CX-19 | F-OD-16 omits one rotation trigger | reject |
| R6-CX-20 | initial ACTIVE references a final tag not yet created | reject; F5 bundle is authority |
| R6-CX-21 | initial ACTIVE has no monitoring at T0+100 before due | ACTIVE |
| R6-CX-22 | monitoring threshold array contains four numeric rows | reject; exactly five required |
| R6-CX-23 | breach array omits one recomputed failed metric | reject |
| R6-CX-24 | monitoring window has `window_start == window_end` | reject |
| R6-CX-25 | pre-release abandonment requires a release ID | reject; campaign scope has no release ID |
| R6-CX-26 | emergency object uses `repository_blob_sha` | reject; only `repository_blob` exists |
| R6-CX-27 | emergency review has only security and operations rows | reject; five-role bundle required |
| R6-CX-28 | historical R3R-CX-04 requires pusher identity | mark superseded; pusher is irrelevant |
| R6-CX-29 | runtime reference endpoint projection contains result unit | reject |
| R6-CX-30 | relation array differs only by insertion order | reject as noncanonical unless ASCII tuple-sorted |

## 21. R6 remediation ledger

Only a fresh independent R6 reviewer may close a finding. Author dispositions are
limited to `REMEDIATED` or `OPEN`; no author row uses `CLOSED`.

| R6 remediation ID / R5 finding | R6 exact section | root cause | R6 remediation | requirement ID | AC | test | evidence | AUTHOR DISPOSITION |
|---|---|---|---|---|---|---|---|---|---|
| F-PLAN-R6-P1-01 | §3,§14,§17 | subjects pointed to future registry hashes | remove pointers; externalize attestation | F-PLAN-R6-P1-01 | AC6-01 | T6-01 | EV6-01 | REMEDIATED |
| F-PLAN-R6-P1-02 | §5,§6 | aggregate/tag decision duplication | exact five-row aggregate and minimal tags | F-PLAN-R6-P1-02 | AC6-02 | T6-02 | EV6-02 | REMEDIATED |
| F-PLAN-R6-P1-03 | §7 | build/report/argv authority was split | build/readiness schemas and exact command wire | F-PLAN-R6-P1-03 | AC6-03 | T6-03 | EV6-03 | REMEDIATED |
| F-PLAN-R6-P1-04 | §8,§9 | subject/relation/currentness meanings were ambiguous | complete-file hashes, typed tuple order, prior-head model | F-PLAN-R6-P1-04 | AC6-04 | T6-04 | EV6-04 | REMEDIATED |
| F-PLAN-R6-P1-05 | §4,§12 | power ID was bound before analysis | interface-first construction and integer-positive encoding | F-PLAN-R6-P1-05 | AC6-05 | T6-05 | EV6-05 | REMEDIATED |
| F-PLAN-R6-P1-06 | §13 | check-result and unit conversion interface incomplete | schema-versioned result and exact-unit rule | F-PLAN-R6-P1-06 | AC6-06 | T6-06 | EV6-06 | REMEDIATED |
| F-PLAN-R6-P1-07 | §13,§18 | external result carried non-runtime scalar fields | exact runtime projection and external provenance | F-PLAN-R6-P1-07 | AC6-07 | T6-07 | EV6-07 | REMEDIATED |
| F-PLAN-R6-P1-08 | §4,§14,§18 | trigger and initial-state authority were circular | total triggers and pre-tag F5 review | F-PLAN-R6-P1-08 | AC6-08 | T6-08 | EV6-08 | REMEDIATED |
| F-PLAN-R6-P1-09 | §14,§18 | first due window and breach semantics conflicted | grace period, five thresholds, exact windows/breaches | F-PLAN-R6-P1-09 | AC6-09 | T6-09 | EV6-09 | REMEDIATED |
| F-PLAN-R6-P1-10 | §15 | incident/retention/compromise schemas were not constructible | typed scopes, retention relations, Git blob emergency path | F-PLAN-R6-P1-10 | AC6-10 | T6-10 | EV6-10 | REMEDIATED |
| F-PLAN-R6-P1-11 | §19-§21 | traceability was identifier-only and history lacked supersession | substantive procedures, OD map, cumulative case accounting | F-PLAN-R6-P1-11 | AC6-11 | T6-11 | EV6-11 | REMEDIATED |

## 22. Internal author audit

The author audit is plan lint and constructive audit, not approval. Every
counter is required to be zero after the rewrite; the independent reviewer must
recompute it.

```text
UNTYPED_NORMATIVE_FIELDS=0
SEMANTIC_ID_CONSTRUCTION_AMBIGUITIES=0
WIRE_IDENTITY_CYCLES=0
ENROLLMENT_CRYPTOGRAPHIC_WIRE_AMBIGUITIES=0
REGISTRY_CRYPTOGRAPHIC_WIRE_AMBIGUITIES=0
TAG_CREATOR_AMBIGUITIES=0
TAG_CREATOR_BOOTSTRAP_CIRCULARITY=0
TAG_REVIEW_EVIDENCE_AMBIGUITIES=0
CHECKER_BUILD_INPUT_AMBIGUITIES=0
CHECKER_CARGO_HOME_AMBIGUITIES=0
CHECKER_COMMAND_AUTHORITY_AMBIGUITIES=0
REAL_MANIFEST_VALIDATION_AUTHORITY=0
REGISTRY_RESOLVER_URI_CONTRADICTION=0
REGISTRY_OBJECT_HASH_SEMANTICS_AMBIGUITIES=0
INCIDENT_WIRE_AMBIGUITIES=0
RETENTION_AUDIT_WIRE_AMBIGUITIES=0
REGISTRY_RELATION_TYPE_AMBIGUITIES=0
REGISTRY_SUBJECT_AMBIGUITIES=0
SCIENTIFIC_ADMISSIBILITY_ENFORCEMENT_AMBIGUITIES=0
PACKAGE_CLASSIFICATION_AMBIGUITIES=0
PHYSICAL_NATIVE_IDENTITY_AMBIGUITIES=0
PHYSICAL_PSEUDOREPLICATION_PATHS=0
CUSTODY_CONTINUITY_AMBIGUITIES=0
CUSTODY_DESTROYED_UNIT_AMBIGUITIES=0
DEVIATION_ACTION_COMPATIBILITY_AMBIGUITIES=0
POWER_METHOD_INTERFACE_AMBIGUITIES=0
METROLOGY_POLICY_AMBIGUITIES=0
METROLOGY_INTERFACE_GAPS=0
REFERENCE_RESULT_TO_RUNTIME_MAPPING_AMBIGUITIES=0
CLAIM_STATE_TRANSITION_AMBIGUITIES=0
REINSTATEMENT_AUTHORITY_AMBIGUITIES=0
MONITORING_POLICY_AMBIGUITIES=0
MONITORING_RECORD_VALUE_TYPE_AMBIGUITIES=0
ACCEPTED_MONITORING_WINDOW_AMBIGUITIES=0
MONITORING_CADENCE_AMBIGUITIES=0
RETENTION_POLICY_AMBIGUITIES=0
REVOKED_ROOT_PUBLIC_CLAIM_BYPASS_PATHS=0
ORPHAN_EXTERNAL_SCHEMAS=0
TRACEABILITY_SUBSTANCE_GAPS=0
LOST_R1_NORMATIVE_OBLIGATIONS=0
UNBOUND_ABSTRACT_AUTHORITY_TERMS=0
NORMATIVE_CONTRADICTIONS=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES=0
UNMAPPED_REQUIREMENTS=0
UNMAPPED_ACS=0
UNMAPPED_TESTS=0
UNMAPPED_EVIDENCE=0
UNMAPPED_ODS=0
CONFLICTING_DUPLICATED_CLAUSES=0
SCIENTIFIC_DEFAULTS_INVENTED=0
HIDDEN_DEFAULTS=0
PARTIAL_PRIMITIVE_GRAMMARS=0
MISSING_VALUE_GRAMMARS=0
MISSING_UNITS=0
DECISION_TO_RUNTIME_MAPPING_AMBIGUITIES=0
UNREPRESENTABLE_DECISION_VALUES=0
HIDDEN_TRANSFORMATION_DEFAULTS=0
STAGE_IMPLEMENTATION_ORDER_AMBIGUITY=0
TAG_BODY_GRAMMAR_AMBIGUITIES=0
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
PHYSICAL_IDENTITY_WIRE_AMBIGUITIES=0
CHAIN_OF_CUSTODY_WIRE_AMBIGUITIES=0
DEVIATION_AUTHORITY_AMBIGUITIES=0
RELEASE_RECORD_SELF_REFERENCE_CYCLES=0
RELEASE_RECORD_SUBJECT_SEMANTICS_AMBIGUITIES=0
CLAIM_STATE_SELF_REFERENCE_CYCLES=0
CLAIM_STATE_CONSTRUCTION_ORDER_AMBIGUITIES=0
CLAIM_STATE_TIME_ORDER_AMBIGUITIES=0
TRUST_EMBEDDED_SOURCE_AUTHORITY_AMBIGUITIES=0
TRUST_RUNTIME_VS_EXTERNAL_LIFECYCLE_AMBIGUITIES=0
PRIVATE_KEY_REPOSITORY_PATHS=0
TEST_AUTHORITY_TO_PRODUCTION_PATHS=0
TEST_TO_PHYSICAL_EVIDENCE_PROMOTION_PATHS=0
SYNTHETIC_TO_PHYSICAL_CLAIM_PATHS=0
CONSTRUCTED_TO_PHYSICAL_CLAIM_PATHS=0
UNKNOWN_TO_PHYSICAL_CLAIM_PATHS=0
PHYSICAL_CLAIM_BEFORE_F5_PATHS=0
FINAL_TAG_VS_LIVE_STATE_AMBIGUITIES=0
P2_TEMPORARY_DISPOSITION_AMBIGUITY=0
P2_RELEASE_BYPASS_PATHS=0
PRODUCTION_EXECUTION_ORDER_CONTRADICTIONS=0
PHASE_E_PROVISIONING_COMPATIBILITY_AMBIGUITIES=0
REGISTRY_BACK_POINTER_PATHS=0
POSITIVE_PATH_HASH_CYCLES=0
RELEASE_RECORD_SELF_REFERENCE_CYCLES=0
CLAIM_STATE_SELF_REFERENCE_CYCLES=0
MONITORING_RECORD_SELF_REFERENCE_CYCLES=0
DEVIATION_IDENTITY_CYCLES=0
REVIEW_BUNDLE_AGGREGATION_AMBIGUITIES=0
TAG_DECISION_DUPLICATION_PATHS=0
TAG_REVIEW_BUNDLE_AMBIGUITIES=0
CHECKER_BUILD_EVIDENCE_AMBIGUITIES=0
REGISTRY_RELATION_ORDER_AMBIGUITIES=0
POWER_CONSTRUCTION_ORDER_AMBIGUITIES=0
METROLOGY_CHECK_RESULT_ID_AMBIGUITIES=0
LOD_LOQ_AMBIGUITIES=0
CLAIM_TRIGGER_POLICY_AMBIGUITIES=0
INITIAL_ACTIVE_CONSTRUCTION_AMBIGUITIES=0
INITIAL_MONITORING_POSITIVE_PATH_AMBIGUITIES=0
MONITORING_RESULT_DERIVATION_AMBIGUITIES=0
INCIDENT_SCOPE_AMBIGUITIES=0
REGISTRY_KEY_COMPROMISE_AUTHORITY_AMBIGUITIES=0
OWNER_KEY_COMPROMISE_AUTHORITY_AMBIGUITIES=0
HISTORICAL_CASE_ACCOUNTING_GAPS=0
```

The constructive audit asks for one valid instance of every catalog schema,
every semantic ID, every complete-file hash, both signing payloads, one genesis-
through-active chain, monitoring pass/breach, permitted reinstatement,
retention audit, reference/runtime projection, power analysis, metrology check,
checker invocation/report, and pusher-independent tag. Any guess is a P1.

## 23. Required validation and handoff

Before and after authoring, run exactly:

```text
git diff --check
cargo fmt --all --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --test phase_e_validation
cargo test --locked --test phase_d_reporting_public_output
```

Required results are Clippy diagnostics zero, Phase-E `38/38`, Phase-D
`73/73`, frozen Phase-E SHA/blob unchanged, and exactly one changed path:
`docs/engineering_specification/phase_f_physical_evidence_and_production_validation_plan.md`.
No source, test, fixture, config, tool, Cargo, README, CHANGELOG, or
`next_milestone_plan.md` change is allowed.

Create one forward commit with subject
`docs(plan): remove Phase F authority cycles`. Do not amend, reset, rebase,
squash, force-push, tag, create an implementation branch, start F0, generate
keys/signatures, provision trust, or create evidence/registry/monitoring/claim
records. Immediately before push, verify local `main`, `origin/main`, and live
remote `main` all equal the required R5 starting SHA; if live remote cannot be
verified, stop before push. Then push `main` normally. After a successful push,
record the R6 commit SHA,
plan SHA-256, and Git blob and require a clean worktree. No later commit occurs
before fresh independent R6 rereview.

## 24. Required R6 planning-remediation handoff

```text
MHI V1 PHASE F
R6 PLANNING REMEDIATION HANDOFF

STARTING R5 SHA: 49b0f92ac3a8c14f84572d6603fd97b7d223f4a0
R5 PLAN SHA-256: 507bca050a6c046f536f5244c9e0a0483d4a6fa04b22a004f6886d89685ddc04
R5 PLAN BLOB: 51f19b6814e0f7ffb42bad647dabe6f804f743d4
R6 PLAN REVIEW SHA: <filled only by fresh independent R6 reviewer>
R6 PLAN SHA-256: <filled after final plan bytes>
R6 PLAN GIT BLOB: <filled after final plan bytes>
CHANGED FILES: 1 expected

F-PLAN-R6-P1-01: REMEDIATED
F-PLAN-R6-P1-02: REMEDIATED
F-PLAN-R6-P1-03: REMEDIATED
F-PLAN-R6-P1-04: REMEDIATED
F-PLAN-R6-P1-05: REMEDIATED
F-PLAN-R6-P1-06: REMEDIATED
F-PLAN-R6-P1-07: REMEDIATED
F-PLAN-R6-P1-08: REMEDIATED
F-PLAN-R6-P1-09: REMEDIATED
F-PLAN-R6-P1-10: REMEDIATED
F-PLAN-R6-P1-11: REMEDIATED

TAG_PUSHER_IDENTITY_AUTHORITATIVE: NO
ENROLLMENT_FILE_SIGNATURES: NO
REVIEW_BUNDLE_AUTHORITY: COMPLETE
REGISTRY_CRYPTOGRAPHY: COMPLETE
LIVE_RESOLVER_TYPE: COMPLETE
CHECKER_FRESH_CARGO_HOME: yes
CHECKER_PARENT_CARGO_CONFIG_EXCLUDED: yes
CHECKER_BUILD_ENV_CLOSED: yes
CHECKER_COMMAND_GRAMMAR: yes
CHECKER_EXIT_CODES: yes
CHECKER_REPORT_SCHEMA: yes
REGISTRY_OBJECT_HASH_SEMANTICS: COMPLETE
REGISTRY_TYPED_RELATIONS: COMPLETE
INCIDENT_SCHEMA: COMPLETE
RETENTION_AUDIT: COMPLETE
SCIENTIFIC_ADMISSIBILITY: COMPLETE
PACKAGE_CLASSIFICATION: COMPLETE
NATIVE_IDENTITY: COMPLETE
CUSTODY_CONTINUITY: COMPLETE
DESTROYED_UNIT_SEMANTICS: COMPLETE
DEVIATION_ACTION_MATRIX: COMPLETE
POWER_UNIT_RULE: COMPLETE
POWER_OUTPUT_SPEC: COMPLETE
POWER_SENSITIVITY_CASES: COMPLETE
METROLOGY_ALLOWED_METHODS: COMPLETE
METROLOGY_CHECK_SPEC: COMPLETE
METROLOGY_CHECK_RESULT: COMPLETE
LOD_LOQ: COMPLETE
REFERENCE_ENDPOINT_ID: COMPLETE
REFERENCE_LIMITATIONS: COMPLETE
REFERENCE_UNCERTAINTY: COMPLETE
REFERENCE_RUNTIME_PROJECTION: COMPLETE
CLAIM_TRIGGER_RESOLUTION_MODE: COMPLETE
CLAIM_REASON_TRANSITION_MATRIX: COMPLETE
REINSTATEMENT_AUTHORITY: COMPLETE
MONITORING_STATUS_VOCABULARY: COMPLETE
MONITORING_THRESHOLD_DERIVATION: COMPLETE
MONITORING_RESULT_DERIVATION: COMPLETE
ACCEPTED_MONITORING_WINDOW: COMPLETE
MONITORING_CADENCE: COMPLETE
RETENTION_START_END: COMPLETE
RETENTION_REPLACEMENT: COMPLETE
RETENTION_AUDIT: COMPLETE
INCIDENT_EFFECTS: COMPLETE
REGISTRY_KEY_COMPROMISE: COMPLETE
OWNER_KEY_COMPROMISE: COMPLETE
CATALOG_REQUIREMENTS: 11 canonical R6 requirement rows plus schema mappings
CATALOG_ACS: 11 substantive R6 acceptance criteria
CATALOG_TESTS: 11 executable R6 tests
CATALOG_EVIDENCE: 11 substantive R6 evidence items
CATALOG_OWNER_DECISIONS: 20
CATALOG_EXTERNAL_SCHEMAS_UNMAPPED: 0
TRACEABILITY_SUBSTANCE_GAPS: 0
LOST_R1_NORMATIVE_OBLIGATIONS: 0
ORPHAN_EXTERNAL_SCHEMAS: 0

DAG
registry back-pointers: 0 expected
release cycle: 0 expected
state cycle: 0 expected
monitoring cycle: 0 expected
deviation cycle: 0 expected

REVIEW / TAG
aggregate rule: COMPLETE
per-role decisions removed from tags: yes
review-bundle hash in every approval tag: yes

CHECKER
build evidence: COMPLETE
readiness evidence: COMPLETE
command object: COMPLETE
argv mapping: COMPLETE
exit codes: COMPLETE

REGISTRY
relation order: COMPLETE
package relations: COMPLETE
retention relations: COMPLETE
head/prior-head model: COMPLETE

POWER
positive integer representation: COMPLETE
F-OD future analysis ID removed: yes
construction path: COMPLETE

METROLOGY
check-result ID: COMPLETE
schema version: COMPLETE
LOD exact-unit rule: COMPLETE

REFERENCE
measurement fields external-only: yes
runtime projection exact: COMPLETE

CLAIM
F-OD trigger totality: COMPLETE
F5 candidate: COMPLETE
initial ACTIVE review authority: COMPLETE

MONITORING
initial grace period: COMPLETE
threshold count: 5 expected
breach-set equality: COMPLETE
window positive interval: COMPLETE
accepted window: COMPLETE

RETENTION / INCIDENT
incident scope: COMPLETE
pre-release abandonment: COMPLETE
retention path: COMPLETE
registry compromise: COMPLETE
owner compromise: COMPLETE

TRACEABILITY
requirements: 11 substantive rows
ACs: 11 substantive rows
tests: 11 executable rows
evidence: 11 substantive rows
ODs: F-OD-01..20 all mapped
unmapped ODs: 0
traceability substance gaps: 0
lost R1 obligations: 0

HISTORICAL CASES
retained: all cumulative rows except R3R-CX-04
superseded: R3R-CX-04 only
accounting gaps: 0

POSITIVE PATH
complete DAG constructible: yes
construction ambiguities: 0

AUTHOR AUDIT
all R6 counters in §22: 0

R6 INDEPENDENT REREVIEW: PENDING
FROZEN PHASE-E PLAN: PASS if SHA/blob verify unchanged
PLAN TAG CREATED: NO
IMPLEMENTATION BRANCH: NO
F0 STARTED: NO
TRUST CHANGED: NO
KEYS CREATED: NO
EVIDENCE CREATED: NO
CLAIM CREATED: NO
READY_FOR_FRESH_PHASE_F_R6_PLAN_REREVIEW: yes
READY_FOR_PHASE_F_PLAN_APPROVAL_TAG: NO pending fresh R6 GO
READY_FOR_PHASE_F_IMPLEMENTATION: NO
```
