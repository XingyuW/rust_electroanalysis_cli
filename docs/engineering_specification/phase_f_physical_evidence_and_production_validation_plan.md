# MHI V1 Phase F — R8 planning-only final contract reconciliation

## 1. Authority, status, scope, and chronology

This document is the Phase-F R8 planning remediation of the independently
rereviewed R7 plan. It changes only this plan document. It does not create a
schema file, checker, tag, branch, key, signature, trust root, registry record,
physical evidence, monitoring record, claim, production implementation, new
scientific model, or new scientific scope.

This is planning only: no architecture expansion, no self-Git identity, and no
future-file authority in F0.

The starting authority is exact:

| Authority | Value |
|---|---|
| R7 plan-review SHA | `e9cef7d7370b084f64eb91a628fb47b0b868dc63` |
| R7 plan SHA-256 | `ab4acec5c9f8f16e8c35d14f2ca83b977a16cacc4ac2505cc5e3bacdf9980c8b` |
| R7 plan Git blob | `625413873fab712961e38f6e20b98d00a5110b52` |
| R7 independent rereview | `P0=0`, `P1=5 grouped findings`, `P2=0`, `P3=0`, `PLAN_DECISION=NO-GO`, `PLAN_AUTHORITY=FAIL` |
| R8 status | forward remediation; independent R8 rereview `PENDING` |
| plan approval tag | absent; must remain absent in R8 |
| implementation branch | absent; must remain absent in R8 |

The immutable Phase-E authority is not changed: integrated baseline
`14942a30928b88f16914bf0bb103cc0c2a5bfa76`, reviewed implementation
`5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb`, frozen plan SHA-256
`0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`, and
frozen plan blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

R1 was `NO-GO/P1=13`; R2 was `NO-GO/P1=10`; R3 was `NO-GO/P1=19`; R4 was
`NO-GO/P1=14`; R5 was `NO-GO/P1=11`; R6 was `NO-GO/P1=13`; R7 was
`NO-GO/P1=5 grouped findings`. No rejected version is described as approved.
The exact future order remains: fresh R8
rereview, plan approval, F0, F-IMPL-1 checker and permanent F-MAINT-01/02
closure, readiness, enrollment, genesis, F1, F2, F3, F4, and F5.
F1-F5 remain blocked until the applicable approved tags and authority objects
exist.

`F_IMPL_1_BEFORE_F0_EXIT`, `F_IMPL_2_BEFORE_F0_EXIT`,
`F_IMPL_3_BEFORE_F0_EXIT`, and `F_IMPL_4_BEFORE_F0_EXIT` are forbidden.
R5, R6, R7, and R8 author audits are not independent approval. No R6, R7, or R8 approval tag,
implementation branch, F0 activity, key, signature, trust, registry, evidence,
claim, or monitoring object exists as a result of this plan edit.

R7 foundational rule: `SUBJECT CONTENT -> COMPLETE SUBJECT FILE -> SUBJECT
FILE SHA-256 -> LATER REGISTRY ATTESTATION`. Release, claim-state, monitoring,
and emergency files never contain their own registry-record hashes. Git
publication is an outer attestation: `OBJECT FILE -> REVIEW -> LATER GIT
COMMIT CONTAINING THE ALREADY-COMPLETE FILE`. Required author invariants are
`REGISTRY_BACK_POINTER_PATHS=0`, `WIRE_IDENTITY_CYCLES=0`,
`SELF_GIT_IDENTITY_CYCLES=0`, `POSITIVE_PATH_HASH_CYCLES=0`,
`RELEASE_RECORD_SELF_REFERENCE_CYCLES=0`,
`CLAIM_STATE_SELF_REFERENCE_CYCLES=0`,
`MONITORING_RECORD_SELF_REFERENCE_CYCLES=0`, and
`DEVIATION_IDENTITY_CYCLES=0`.

## 2. Closed primitive and type registry

All external JSON objects use UTF-8 bytes, RFC 8785 JCS, duplicate-member
rejection, unknown-member rejection, no omitted member, and no member typed as
an unqualified primitive. A nullable member is explicitly `T|null`; no other
optional-member convention exists. `schema_version` is the JSON integer `1`.
Arrays are duplicate-free. An array is `SORTED_UNIQUE<T>` only when its field
definition explicitly declares sorted order. An array is `FIXED_ORDER<T>` only
when its field definition supplies an explicit literal order. There is no
default rule that sorts every array.

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
PHASE_F_REVIEW_TARGET_V1 = {type:"git_commit",git_sha:GIT_SHA_V1}
  | {type:"external_object",object_kind:PHASE_F_OBJECT_KIND_V1,object_sha256:SHA256_V1}
PHASE_F_OBJECT_KIND_V1 = decision_bundle | git_tag_message | authority_enrollment
  | registry_record | registry_head | registration_document | protocol
  | power_method_interface | power_analysis | package_manifest | dependency_audit
  | physical_unit_ledger | identity_audit | location_ledger | chain_of_custody
  | deviation_ledger | metrology_policy | metrology_check_result
  | reference_source_descriptor | reference_result | scientific_admissibility_audit
  | cohort_lock | owner_approval | execution_record | release_record | claim_state
  | reinstatement_approval | monitoring_policy | monitoring_record | incident_record
  | monitoring_evidence | retention_audit | independent_review_bundle | trust_provisioning_approval
  | physical_release_approval | emergency_registry_compromise | checker_build_evidence
  | checker_readiness_evidence | f5_release_candidate
PHASE_F_REVIEW_ROLE_V1 = scientific_metrology | architecture_data | security
  | compatibility | operations_governance
PHASE_F_CHECKER_DECISION_V1 = pass | no_go | active | not_active | authority_unavailable
PHASE_F_INCIDENT_STATUS_V1 = open | contained | resolved | superseded
PHASE_F_INCIDENT_TYPE_V1 = key_compromise | key_revocation | registry_equivocation
  | data_integrity | custody_break | undeclared_dependency | monitoring_breach
  | reference_qc_breach | domain_breach | retention_failure | campaign_abandonment
  | manual_withdrawal
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
report:PATH_V1,prior_head:PATH_V1|null,
registry_compromised_emergency:PATH_V1|null,
registry_compromised_review:PATH_V1|null,
registry_compromised_commit:GIT_SHA_V1|null}`. `PATH_V1` is a valid UTF-8
path string with no NUL, CR, or LF and the §7 path resolution rules; it is not
an untyped string. `PATH_V1` may be absolute only when the command argument
explicitly permits an absolute path; relative paths have no process-CWD meaning.
`prior_head` and the three `registry_compromised_*` members are the four nullable
fields. Either all three emergency members are null or all three are non-null;
partial emergency triples are invalid. `PhaseFArgvV1` is the exact ordered JSON
array generated from the object. For `verify` it is:
`["phase-f-authority-check","verify","--kind",kind,"--input",input,
"--context-dir",context_dir,"--report",report]`. For `claim-status` it is:
`["phase-f-authority-check","claim-status","--release",release,
"--context-dir",context_dir,"--registry-head-uri",registry_head_uri,
"--now",now,"--report",report]`, followed by exactly
`["--prior-head",prior_head]` when non-null and, when the emergency triple is
non-null, exactly
`["--registry-compromised-emergency",emergency,
"--registry-compromised-review",review,
"--registry-compromised-commit",commit]`. No legacy emergency flag exists.
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
variant is legal only for the four status metrics; a rate only for the four
rate metrics; quantity only for sensor drift; and the binding variants
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
| `PhaseFMonitoringEvidenceV1` | `monitoring_evidence_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_monitoring_evidence_v1\0` | own ID | F5+ |
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
| 12 | `{power_method_id:RUNTIME_STABLE_ID_V1,power_method_version:RUNTIME_CANONICAL_TEXT_V1}`; no power-interface identity, URI, or byte length exists during F0 |
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
F-OD-08..20 have no runtime override. Missing, extra, defaulted, normalized,
transformed, or unrepresentable values are F0/F1 NO-GO.

The plan-only `PhaseFProtocolProjectionV1` value is exactly
`{decision_bundle_sha256:SHA256_V1,protocol_toml_sha256:SHA256_V1,
runtime_protocol:MhiValidationProtocolV1,projection_result:PHASE_F_RESULT_V1}`.
It is not a runtime schema, does not create a production route, and exists only
to make the F0-to-runtime comparison auditable.

F-OD-12 is exactly `{power_method_id:RUNTIME_STABLE_ID_V1,
power_method_version:RUNTIME_CANONICAL_TEXT_V1}` and nothing else. F0 selects
the method ID/version only. During F1, construct
`PhaseFPowerMethodInterfaceV1`, require exact equality of its method ID/version
to F-OD-12, then construct and review the power analysis. No interface hash,
URI, byte length, analysis ID, or other future-object reference is permitted in
F0. Therefore `F0_F1_FUTURE_OBJECT_DEPENDENCY_PATHS=0` and
`F0_FUTURE_OBJECT_REFERENCE_PATHS=0`.

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
`schema_version,review_bundle_id,target,reviews,aggregate_p0_count,
aggregate_p1_count,aggregate_decision`. `target` is exactly
`PhaseFReviewTargetV1`: either `{type:"git_commit",git_sha:GIT_SHA_V1}` or
`{type:"external_object",object_kind:PHASE_F_OBJECT_KIND_V1,
object_sha256:SHA256_V1}`. There are no nullable target SHA fields and no
review-instance ID. Each row is exactly
`{role:PHASE_F_REVIEW_ROLE_V1,decision:PHASE_F_DECISION_V1,
p0_count:CANONICAL_UNSIGNED_INTEGER_V1,
p1_count:CANONICAL_UNSIGNED_INTEGER_V1,finding_ids:[RUNTIME_STABLE_ID_V1],
review_artifact_reference:PhaseFObjectReferenceV1}`. Rows are exactly one per
role in enum order; finding IDs are sorted unique. The immutable review
artifact reference identifies the actual review evidence, so a second row ID
is neither needed nor permitted. The aggregate is not a reviewer opinion:
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
The F5 review bundle sets `target={type:"external_object",
object_kind:"f5_release_candidate",object_sha256:<complete candidate SHA>}`;
its five rows therefore review one exact candidate, not a moving release or
tag. The same target form is used for F0 decisions, readiness, enrollment,
power analysis, and emergency objects. A plan review may use the Git-commit
variant, for example `{type:"git_commit",git_sha:<R7 plan review SHA>}`.
The normative external-object examples are:
`{type:"external_object",object_kind:"decision_bundle",object_sha256:<decision SHA>}`;
`{type:"external_object",object_kind:"checker_readiness_evidence",object_sha256:<readiness SHA>}`;
`{type:"external_object",object_kind:"authority_enrollment",object_sha256:<enrollment SHA>}`;
`{type:"external_object",object_kind:"power_analysis",object_sha256:<analysis SHA>}`;
and `{type:"external_object",object_kind:"f5_release_candidate",object_sha256:<candidate SHA>}`.
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
| `ism-mechanism-health-v1-f-plan-approved` / `PhaseFPlanApprovalV1` | reviewed R7 main | `plan_review_sha:GIT_SHA_V1,plan_sha256:SHA256_V1,plan_git_blob:GIT_BLOB_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
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
  --report <PATH> [--prior-head <PATH>] \
  [--registry-compromised-emergency <PATH> \
   --registry-compromised-review <PATH> \
   --registry-compromised-commit <GIT_SHA>]
```

`--kind` is exactly one `PHASE_F_OBJECT_KIND_V1`; `verify` takes exactly one
input, one context directory, and one report path. `claim-status` takes exactly
one release, context directory, live URI, UTC timestamp, and report path. Each
optional argument is one exact pair, serialized only when its corresponding
object member is non-null. The three emergency members are nullable as one
all-null or all-non-null triple; a partial triple is a usage error. When
non-null, argv appends the emergency path, review path, and commit SHA in
exactly that order. The emergency path accepts only §15's exact emergency
schema and its separately supplied review and commit.
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
| `monitoring_evidence` | SHA-256 of complete canonical `PhaseFMonitoringEvidenceV1` bytes |
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
| `power_registered` | `power_analysis_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `power_analysis` | `authorized_by+decision_bundle`; `authorized_by+independent_review_bundle`; `depends_on+power_method_interface`; `depends_on+protocol` | the independent-review-bundle relation names the five-role bundle whose `target={type:"external_object",object_kind:"power_analysis",object_sha256:<exact subject_sha256>}` |
| `package_registered` | `manifest_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `package_manifest` | exactly once each: `depends_on+dependency_audit`, `depends_on+physical_unit_ledger`, `depends_on+identity_audit`, `depends_on+location_ledger`, `depends_on+chain_of_custody`, `depends_on+deviation_ledger`, `depends_on+metrology_policy`, `depends_on+scientific_admissibility_audit`; at least once each: `references+reference_result`, `references+reference_source_descriptor` | none; `locks`, `releases`, and untyped relations forbidden |
| `cohort_locked` | `cohort_lock_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `cohort_lock` | `locks+package_manifest`; `depends_on+protocol`; `depends_on+power_analysis`; `depends_on+deviation_ledger`; `depends_on+scientific_admissibility_audit` | none |
| `owner_approval_registered` | `approval_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `owner_approval` | `approves+cohort_lock`; `authorized_by+authority_enrollment` | none |
| `execution_registered` | `execution_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `execution_record` | `executes+cohort_lock`; `authorized_by+owner_approval`; `depends_on+deviation_ledger`; `depends_on+protocol` | none |
| `release_registered` | `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `release_record` | `releases+execution_record`; `authorized_by+owner_approval`; `depends_on+monitoring_policy`; `depends_on+metrology_policy` | none |
| `claim_state_changed` | `claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `claim_state` | `changes_state_of+release_record` | initial ACTIVE also requires `authorized_by+independent_review_bundle` naming the exact F5 activation bundle; every noninitial state requires exactly one `registered_after+claim_state`; incident-driven reasons require exactly one `depends_on+incident_record` whose hash equals `cause_incident_sha256`; approved reinstatement requires exactly one `depends_on+reinstatement_approval`; superseded state requires exactly one `supersedes+release_record`; periodic expiry has none of those |
| `monitoring_recorded` | `monitoring_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `monitoring_record` | `references+release_record`; `depends_on+monitoring_policy`; exactly 15 `depends_on+monitoring_evidence` relations, whose hashes equal exactly the 15 measurement `monitoring_evidence_sha256` values, one per measurement | one prior `registered_after+monitoring_record` after first; no missing, extra, or duplicate monitoring-evidence relation |
| `incident_recorded` | `incident_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `incident_record` | for release scope exactly `incident_recorded+release_record`; for campaign or registry-namespace scope no release relation is permitted; campaign abandonment additionally requires exactly `authorized_by+decision_bundle`, `authorized_by+independent_review_bundle`, and `references+package_manifest` | references only to listed affected evidence; ordinary campaign incidents use the separately defined ordinary set |
| `retention_audit_recorded` | `retention_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `retention_audit` | exactly once: `authorized_by+decision_bundle`; release scope exactly once `references+release_record`; campaign scope exactly once `references+package_manifest` | exactly one `registered_after+retention_audit` for every later audit of the same scope; no release relation for campaign scope and no package-manifest relation for release scope |

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
is ordered: F0 selects only method ID/version; F1 retrieves/verifies and
constructs the interface; the interface method ID/version must equal F-OD-12;
then choose exact approved parameters;
validate required, unknown, type, unit, and range; execute exact method/software;
create outputs; validate output IDs/types/units/ranges; evaluate every required
sensitivity case; construct the complete analysis object; calculate its
content-derived `power_analysis_id`; create a normal five-role independent
review bundle whose single target is `{type:"external_object",
object_kind:"power_analysis",object_sha256:<complete analysis file SHA>}`;
require aggregate GO/P0=0/P1=0; then create the `power_registered` registry
attestation. That record requires `authorized_by+decision_bundle`,
`authorized_by+independent_review_bundle`, `depends_on+power_method_interface`,
and `depends_on+protocol`; the review relation must name the bundle targeting
the exact analysis subject file hash. No review hash is embedded in the
analysis file. `analysis_id` is not a second wire field and means
`power_analysis_id` only in historical prose.

The scientific/metrology role must substantively assess power adequacy, the
analysis file, and the exact referenced interface identified by
`power_method_interface_sha256`. No separate interface review bundle is
required unless another independent requirement demands it.

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
`{endpoint_id:RUNTIME_STABLE_ID_V1,check_id:RUNTIME_STABLE_ID_V1,
check_kind:PHASE_F_CHECK_KIND_V1,
method_id:RUNTIME_STABLE_ID_V1,method_version:RUNTIME_CANONICAL_TEXT_V1,
authority_id:RUNTIME_STABLE_ID_V1,procedure_document:PhaseFObjectReferenceV1,
measurand_id:RUNTIME_STABLE_ID_V1,result_unit:UNIT_TEXT_V1,
comparator:greater_than_or_equal|less_than_or_equal,threshold:RUNTIME_F64_V1,
failure_action:exclude_before_lock|campaign_no_go}`. Policy endpoints contain
sorted check-spec arrays.

`PhaseFMetrologyCheckResultV1` is exactly
`{schema_version:JSON_INTEGER_ONE,check_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,
endpoint_id:RUNTIME_STABLE_ID_V1,metrology_policy_sha256:SHA256_V1,
check_id:RUNTIME_STABLE_ID_V1,reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,
performed_at:UTC_SECOND_TIMESTAMP_V1,
method_id:RUNTIME_STABLE_ID_V1,method_version:RUNTIME_CANONICAL_TEXT_V1,
authority_id:RUNTIME_STABLE_ID_V1,measurand_id:RUNTIME_STABLE_ID_V1,
value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1,result:PHASE_F_CHECK_RESULT_V1}`. Checker recomputes
the comparator; manually inconsistent result rejects. Every required calibration
and QC result must pass.

Check IDs need not be globally unique. The exact specification lookup is
`(metrology_policy_sha256,endpoint_id,check_id)`: load the policy named by the
result, select exactly one endpoint policy, then exactly one check specification
within that endpoint. The checker recomputes comparator, threshold, method,
authority, measurand, and unit from that specification. Duplicate `check_id`
values under different endpoints are therefore deterministic; an endpoint
mutation that selects no spec or the wrong spec is NO-GO. The check-result ID
uses `mhi_phase_f_metrology_check_result_v1\0` and excludes only
`check_result_id`.

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
previous_claim_state_record_id,state,reason_code,cause_incident_sha256,effective_at,
superseding_release_record_id,activation_review_bundle_sha256,
reinstatement_approval_sha256,limitations`. `previous_claim_state_record_id`
is nullable only for the initial state; `superseding_release_record_id` is
non-null only for superseded; `activation_review_bundle_sha256` is non-null
only for initial ACTIVE; `reinstatement_approval_sha256` is non-null only for a
suspended-to-active reinstatement. State is
`PHASE_F_CLAIM_STATE_V1`; `reason_code:PHASE_F_CLAIM_REASON_V1` and the exact
cause rule are:

| Reason | Legal prior → next | Extra authority |
|---|---|---|
| `initial_release` | none → active | `cause_incident_sha256=null`; release record and exact F5 review bundle |
| `monitoring_breach` | active → suspended | non-null exact incident SHA; valid monitoring incident |
| `reference_qc_breach` | active → suspended | non-null exact incident SHA; failed required QC |
| `domain_breach` | active → suspended | non-null exact incident SHA; domain evidence |
| `key_compromise` | active → suspended or withdrawn per F0 row | non-null exact incident SHA; un-compromised registry/governance path |
| `key_revocation` | active → suspended or withdrawn per F0 row | non-null exact incident SHA; un-compromised registry/governance path |
| `periodic_expiry` | active or suspended → expired | `cause_incident_sha256=null`; no shortcut |
| `manual_withdrawal` | active or suspended → withdrawn | non-null exact incident SHA; governance incident |
| `superseded_by_new_release` | active or suspended → superseded | `cause_incident_sha256=null`; new release |
| `approved_reinstatement` | suspended → active | `cause_incident_sha256=null`; valid five-role approval and same-release mode |

For every non-null cause, the referenced incident file is complete and its
scope matches the claim/release or applicable campaign/registry authority. The
incident SHA is the exact complete-file SHA, never a semantic ID or a registry
record hash.

No other transition is legal. `new_release_required` forbids old-release
reinstatement; `withdraw_only` forbids reinstatement. If an incident or breach
exists without its required state record, claim-status returns NOT_ACTIVE.

The claim-state registry relation contract is exact. Every state has
`changes_state_of+release_record`. Initial ACTIVE additionally has exactly
`authorized_by+independent_review_bundle` naming the F5 activation bundle and
has no prior-state, incident, reinstatement, or supersession relation. Every
noninitial state has exactly one `registered_after+claim_state`. The six
incident-driven reasons (`monitoring_breach`, `reference_qc_breach`,
`domain_breach`, `key_compromise`, `key_revocation`, and `manual_withdrawal`)
have exactly one `depends_on+incident_record`, whose object hash equals
`cause_incident_sha256`. `approved_reinstatement` has exactly one
`depends_on+reinstatement_approval`; `superseded_by_new_release` has exactly
one `supersedes+release_record` identifying `superseding_release_record_id`;
`periodic_expiry` has none of those extra relations. All inapplicable tuples
are forbidden. `CLAIM_STATE_CAUSE_BINDING_AMBIGUITIES=0` and
`CLAIM_STATE_RELATION_AMBIGUITIES=0`.

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
reference_qc_breach_action`. Define the exact fixed order
`PHASE_F_MONITORING_METRIC_ORDER_V1` as
`domain_compliance,reference_qc_status,calibration_status,reference_uncertainty_status,
sensor_drift,invalid_input_rate,indeterminate_rate,data_quality_insufficient_rate,
exclusion_rate,software_git_sha,checker_binary_sha256,trust_store_sha256,
trust_root_id,owner_approval_id,release_record_id`. Status vocabularies are
`domain_compliance={compliant,out_of_domain,unknown}` healthy `compliant`;
`reference_qc_status={pass,fail,unknown}` healthy `pass`;
`calibration_status={pass,fail,unknown}` healthy `pass`; and
`reference_uncertainty_status={within_limit,above_limit,unknown}` healthy
`within_limit`. `required_metrics:FIXED_ORDER<PHASE_F_MONITORING_METRIC_V1>` is
exactly the fixed 15-member order above.
The four status metrics are
`domain_compliance,reference_qc_status,calibration_status,reference_uncertainty_status`;
the one quantity metric is `sensor_drift`; the four rate metrics are
`invalid_input_rate,indeterminate_rate,data_quality_insufficient_rate,exclusion_rate`;
and the six binding metrics are
`software_git_sha,checker_binary_sha256,trust_store_sha256,trust_root_id,owner_approval_id,release_record_id`.
Rate units are null; drift uses exact unit. `metric_thresholds` is
`FIXED_ORDER<PhaseFMetricThresholdV1>` in the literal five-member order
`sensor_drift,invalid_input_rate,indeterminate_rate,data_quality_insufficient_rate,exclusion_rate`.
Threshold is
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

`PhaseFMonitoringEvidenceV1` is the complete external evidence object:
`{schema_version:JSON_INTEGER_ONE,monitoring_evidence_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,
metric_id:PHASE_F_MONITORING_METRIC_V1,window_start:UTC_SECOND_TIMESTAMP_V1,
window_end:UTC_SECOND_TIMESTAMP_V1,value:PhaseFMonitoringValueV1,
source_references:NONEMPTY_SORTED_UNIQUE<PhaseFMonitoringSourceReferenceV1>,
derivation_document_sha256:SHA256_V1,produced_at:UTC_SECOND_TIMESTAMP_V1}`.
It uses semantic ID domain `mhi_phase_f_monitoring_evidence_v1\0`, excludes
only `monitoring_evidence_id`, and its complete canonical bytes define its
complete-file SHA-256. `window_start<window_end` and
`produced_at>=window_end` are required.

`PHASE_F_MONITORING_SOURCE_KIND_V1` is exactly
`domain_observation|reference_qc_record|calibration_record|sensor_drift_series|
input_validation_summary|runtime_validation_summary|reference_uncertainty_record|
execution_record|trust_provisioning_approval|owner_approval|release_record`.
`PhaseFMonitoringSourceReferenceV1` is exactly
`{source_kind:PHASE_F_MONITORING_SOURCE_KIND_V1,object_reference:PhaseFObjectReferenceV1}`.
Source references sort by source-kind literal ASCII bytes, then
`object_reference.sha256`, then `object_reference.immutable_uri`, with no
duplicate tuple.

The required source-kind mapping is exact: `domain_compliance` requires
`domain_observation`; `reference_qc_status` requires `reference_qc_record`;
`calibration_status` requires `calibration_record`; `sensor_drift` requires
`sensor_drift_series`; `invalid_input_rate` requires `input_validation_summary`;
`indeterminate_rate`, `data_quality_insufficient_rate`, and `exclusion_rate`
each require `runtime_validation_summary`; `reference_uncertainty_status`
requires `reference_uncertainty_record`; `software_git_sha` and
`checker_binary_sha256` each require exactly one `execution_record`;
`trust_store_sha256` and `trust_root_id` each require exactly one
`trust_provisioning_approval`; `owner_approval_id` requires exactly one
`owner_approval`; and `release_record_id` requires exactly one
`release_record`. No source kind outside the listed mapping is permitted.
For binding metrics the checker parses the named authority and compares exact
fields: `PhaseFExecutionRecordV1.release_code_sha`,
`PhaseFExecutionRecordV1.checker_binary_sha256`,
`PhaseFTrustProvisioningApprovalV1.trust_store_sha256`,
`PhaseFTrustProvisioningApprovalV1.trust_root_id`,
`OwnerApprovalEvidenceV1.approval_record_id`, and
`PhaseFReleaseRecordV1.release_record_id`, respectively. For observational
metrics the checker validates syntax, ID/hash, source references, metric,
window, value type, and measurement equality; it does not infer scientific
truth from the underlying source.

`PhaseFMonitoringRecordV1` is exactly
`schema_version,monitoring_record_id,release_record_id,claim_id,window_start,
window_end,policy_sha256,measurements,breaches,result`.
Every required metric appears exactly once in the fixed canonical order.
`measurements` is `FIXED_ORDER<PhaseFMonitoringMeasurementV1>` and is not a
lexically sorted or generic sorted-unique array. Measurement is exactly
`{metric_id:PHASE_F_MONITORING_METRIC_V1,value:PhaseFMonitoringValueV1,
monitoring_evidence_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,monitoring_evidence_sha256:SHA256_V1}`;
status uses the metric vocabulary, numeric uses `RUNTIME_F64_V1`, and bindings
use named ID/hash types. The evidence object is a complete
`PhaseFMonitoringEvidenceV1` file. Breach is exactly
`{metric_id:PHASE_F_MONITORING_METRIC_V1,breach_code:PHASE_F_BREACH_CODE_V1}`
and its evidence object satisfies `evidence.monitoring_evidence_id =
measurement.monitoring_evidence_id`, `evidence` complete-file SHA equals
`measurement.monitoring_evidence_sha256`, `evidence.metric_id = measurement.metric_id`,
`evidence.value = measurement.value`, and
`evidence.window_start = monitoring_record.window_start` plus
`evidence.window_end = monitoring_record.window_end`.
ordered by the same canonical metric order; evidence is never repeated in a breach row. All required metrics exist
exactly once, with exact value types, healthy statuses, release bindings,
passing thresholds, and evidence. `breaches` must equal the exact recomputed failed-metric set: one row
for every failed required metric and zero rows for every passing metric. The
only breach codes are `missing_metric`, `unhealthy_status`, `threshold_failed`,
`binding_mismatch`, and `missing_evidence`; no duplicate, extra, or missing
metric is valid. The code is derived from the first applicable failed reason in
this fixed order: absent metric -> `missing_metric`; invalid/unavailable evidence
-> `missing_evidence`; unhealthy status ->
`unhealthy_status`; failed numeric comparison -> `threshold_failed`; wrong
release/policy binding -> `binding_mismatch`. Declared result is pass iff `breaches` is empty, otherwise
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

For `missing_metric`, no measurement row exists. For `missing_evidence`, the
measurement row exists but its evidence object is unavailable or invalid. The
breach array contains the exact recomputed failed metric set in either case and
never carries a second evidence hash.

The PASS predicate is exact: every required metric appears once; no unrequired
metric appears; every measurement type is correct; every measurement has a
verifiable `monitoring_evidence_id` and `monitoring_evidence_sha256`; every
required evidence object verifies; all four status metrics are healthy; all five
numeric thresholds pass; all six release/build/trust/owner binding metrics equal
release authority; the recomputed
breach set is empty; and `result=pass`. SUSPEND is required for one or more
recomputed failures, and the declared result must equal recomputation.
`MONITORING_RESULT_DERIVATION_AMBIGUITIES=0`,
`MONITORING_EVIDENCE_OBJECT_AMBIGUITIES=0`, and
`MONITORING_METRIC_EVIDENCE_MAPPING_AMBIGUITIES=0`.

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
`PhaseFIncidentRecordV1` contains no `review_bundle_sha256`, review-artifact
reference, or future review hash. Campaign-abandonment construction is exactly
complete incident file -> incident SHA -> independent review bundle with
`target={type:"external_object",object_kind:"incident_record",
object_sha256:<exact incident file SHA>}` -> `incident_recorded` attestation.
There is no reverse pointer. For ordinary non-abandonment campaign incidents,
the permitted relation set is separately defined as only the applicable
`authorized_by+decision_bundle`, package/evidence references, and the required
`incident_recorded` subject relation; the abandonment review relation is not
inherited.

`PhaseFRetentionObjectV1` is exactly the tagged union
`{type:"package_object",object_id:RUNTIME_STABLE_ID_V1,object_reference:PhaseFObjectReferenceV1}`
or `{type:"authority_object",object_kind:PHASE_F_OBJECT_KIND_V1,
object_reference:PhaseFObjectReferenceV1}`. `PhaseFRetentionScopeV1` is exactly the tagged union
`{type:"release",release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1}` or
`{type:"campaign",campaign_id:RUNTIME_STABLE_ID_V1,
package_manifest_sha256:SHA256_V1}`. `PhaseFRetentionAuditV1` is exactly
`schema_version,retention_audit_id,scope,policy_sha256,audited_at,object_checks,
result`. There is no top-level release ID. Each check is exactly
`{object:PhaseFRetentionObjectV1,
primary_available:BOOLEAN_V1,primary_verified:BOOLEAN_V1,
verified_backup_count:CANONICAL_UNSIGNED_INTEGER_V1,
last_backup_verification_at:UTC_SECOND_TIMESTAMP_V1,result:PHASE_F_RESULT_V1}`.
Rows sort by the raw ASCII lexical identity key: package object
`("package_object",object_id,object_reference.sha256)` or authority object
`("authority_object",object_kind,object_reference.sha256)`.
Pass requires primary available and exact, backup count at least F0, and backup
age below F0 interval. A `retention_audited` record is authorized by the
decision bundle and has exactly one scope-specific relation: release scope
uses `references+release_record`; campaign scope uses
`references+package_manifest`. Every later audit of the same scope has exactly
one `registered_after+retention_audit`. A campaign audit never requires a
release relation, and a release audit never requires a package-manifest
relation merely because its objects came from a package.

Retention starts when an authority object is first referenced by valid registry
record. After terminal state, deadline is terminal `effective_at` plus F0
retention seconds; retrieval is required while `now<deadline`, unless another
 nonterminal release references it. Pre-release campaigns end retention only by
an independently reviewed `campaign_abandonment` incident with campaign scope;
the deadline is `incident.detected_at + retention_seconds`, and all campaign
authority objects are retained until that deadline. Campaign-scoped audits
continue until that deadline without a release ID. For campaign scope, the
required retention set is exactly one authority-object row for the exact
package manifest plus one package-object row for every
`PhaseFPackageManifestV1.objects` entry, copying exactly its `object_id` and
`object_reference`; no missing, duplicate, or extra package object is legal.
Deletion is never silent.
Replacement is an additional copy with identical bytes, SHA, and length
recorded in the next audit; different bytes are a new object. If every copy
disappears before deadline, retention failure is required and claim-status is
NOT_ACTIVE.

`CampaignRetentionSetV1` is the exact campaign set above. `ReleaseRetentionSetV1`
is the deterministic union of: (A) the complete campaign set for the
release-bound package; (B) one authority object each for protocol,
power_analysis, cohort_lock, owner_approval, execution_record, release_record,
monitoring_policy, metrology_policy, trust_provisioning_approval, the F5
independent review bundle, the initial claim state, and the latest claim state
at `audited_at`; (C) every accepted `monitoring_record` through `audited_at`;
and (D) every unresolved incident affecting that release at `audited_at`.
The current retention audit is excluded, registry records are excluded from
retained objects, and every reference is derived from verified release,
registry, and context bindings. Human-selected extras are forbidden.
For campaign scope, `object_checks` equals `CampaignRetentionSetV1`; for
release scope it equals `ReleaseRetentionSetV1`, by exact set equality with no
missing, extra, or duplicate object. `RETENTION_COVERAGE_AMBIGUITIES=0`.

Default consequences are exact: key compromise suspend; key revocation suspend
or withdraw per F0; registry equivocation NOT_ACTIVE immediately; data
integrity, custody break, undeclared dependency, monitoring breach, reference
QC breach, domain breach, and retention failure suspend. Incident alone does
not change state; missing transition is NOT_ACTIVE.

Registry-key compromise uses one exact independent path.
`PhaseFRegistryCompromiseEmergencyV1` is exactly
`schema_version,emergency_id,registry_namespace_id,incident_record_sha256,
declared_at,affected_claim_ids,action`, where action is the literal
`suspend_all_active_claims`. It contains no review hash, repository commit,
repository blob, or repository blob hash. Its ID uses
`mhi_phase_f_registry_compromise_emergency_v1\0`, excluding only its own ID;
its complete file is therefore constructible immediately after the incident
file is complete.

After the emergency file is complete, create a normal five-role
`PhaseFIndependentReviewBundleV1` with exactly
`target={type:"external_object",object_kind:"emergency_registry_compromise",
object_sha256:<complete emergency file SHA>}` and aggregate GO/P0=0/P1=0.
The complete emergency file already contains the registry namespace, incident
hash, affected claims, declaration time, and action, so no composite review
target is permitted.

The completed emergency and review files are then published together in a
later repository commit at deterministic paths
`phase_f_governance/emergencies/<DIGEST>/emergency.json` and
`phase_f_governance/emergencies/<DIGEST>/review.json`, where `<DIGEST>` is the
64 lowercase hexadecimal body of `emergency_id` after `sha256:`. Neither file
contains that commit SHA. Claim-status accepts the emergency path, review path,
and commit SHA as three paired inputs and verifies canonical/semantic hashes,
review target/aggregate, commit existence and reachability from live remote
`main`, deterministic tree paths, and byte equality with the local files. The
sequence is incident -> emergency file -> emergency review -> later Git commit
-> NOT_ACTIVE; `REGISTRY_COMPROMISE_GIT_FIXED_POINT_CYCLES=0`.

The emergency checker order is exact: (1) validate the emergency file and
semantic ID; (2) validate the five-role review bundle; (3) verify the review
target equals the exact emergency complete-file SHA; (4) verify aggregate
GO/P0=0/P1=0; (5) verify the supplied commit exists; (6) verify the commit is
reachable from live remote `main`; (7) resolve the Git tree at that exact
commit; (8) fetch exactly
`phase_f_governance/emergencies/<DIGEST>/emergency.json`; (9) fetch exactly
`phase_f_governance/emergencies/<DIGEST>/review.json`; and (10) verify the Git
tree bytes equal the supplied emergency and review bytes. Any failure rejects
the triple or returns the specified fail-closed non-active result.

Owner-key compromise uses the un-compromised registry/governance path to append
the exact suspend/withdraw state; owner signature is not required for that
non-active transition. Recovery requires a new owner key, root, owner approval,
run, and release according to the exact F-OD-16 `resolution_mode`; compromised
key never revokes itself.

## 16. Historical R7 master-schema catalog snapshot (non-normative)

This is the retained R7 catalog snapshot. It is historical and non-normative;
the one current catalog is the R8 MASTER SCHEMA CATALOG in §26. Each row includes exact
fields, identity, complete-file hash, producer, validator, stage, requirement,
AC, test, evidence, and registry relation. The canonical substantive
requirement/AC/test/evidence procedures were §19.2; every R7 identifier in this
snapshot is a historical cross-reference only and is not current R8 acceptance
authority.

| Schema | Field closure / identity | Producer; validator; stage | Registry relation | Requirement / AC / test / evidence |
|---|---|---|---|---|
| `PhaseFDecisionBundleV1` | §4; §3 ID; complete hash; unsigned | F0; checker; F0 | protocol authority | R5-01 / AC5-01 / T5-01 / EV5-01 |
| `PhaseFIndependentReviewBundleV1` | §5 exact tagged target and five rows; §3 ID; complete hash | independent roles; checker; approvals | tag evidence | R7-02/R7-05 / AC7-02/05 / T7-02/05 / EV7-02/05 |
| `PhaseFReviewTargetV1` | §2 exact `git_commit` or `external_object` tagged union | reviewer; target validator; all review gates | nested in review bundle | R7-02 / AC7-02 / T7-02 / EV7-02 |
| `PhaseFIndependentReviewV1` | §5 exact five role fields with no row ID | independent reviewer; review validator; all gates | nested in review bundle | R7-05 / AC7-05 / T7-05 / EV7-05 |
| `PhaseFProtocolProjectionV1` | §4 exact plan contract; no wire ID | checker; projection; F1 | protocol | R5-03 / AC5-03 / T5-03 / EV5-03 |
| `PhaseFAuthorityEnrollmentV1` | §5.1 unsigned; §3 ID/file hash | governance; enrollment; readiness | authority_enrolled | R5-04 / AC5-04 / T5-04 / EV5-04 |
| `PhaseFCheckerBuildEvidenceV1` | §7 exact fields; §3 ID; complete hash | checker builder; independent verifier; readiness | readiness evidence | R6-03 / AC6-03 / T6-03 / EV6-03 |
| `PhaseFCheckerReadinessEvidenceV1` | §7 exact fields; §3 ID in body; complete hash | checker readiness; independent verifier; readiness | readiness tag evidence | R7-06 / AC7-06 / T7-06 / EV7-06 |
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
| `PhaseFPowerAnalysisRecordV1` | §12 params/cases; §3 ID/hash; complete before review | statistician; power; F1 | power subject | R7-03/R7-10 / AC7-03/10 / T7-03/10 / EV7-03/10 |
| `PhaseFMetrologyPolicyV1` | §13 Cartesian methods/checks; §3 ID/hash | metrology; policy; F0/F2 | package/release | R5-20 / AC5-20 / T5-20 / EV5-20 |
| `PhaseFMetrologyCheckSpecV1` | §13 endpoint-qualified exact fields | metrology; policy; F2 | nested policy | R7-09 / AC7-09 / T7-09 / EV7-09 |
| `PhaseFMetrologyCheckResultV1` | §13 endpoint/policy-qualified fields/math; complete hash | laboratory; result; F2 | package evidence | R7-09 / AC7-09 / T7-09 / EV7-09 |
| `PhaseFReferenceSourceDescriptorV1` | §13 runtime types; §3 ID/hash | laboratory/data; source; F2 | package dependency | R5-23 / AC5-23 / T5-23 / EV5-23 |
| `PhaseFReferenceResultV1` | §13 adjudicated fields and exact runtime projection; §3 ID/hash | laboratory; reference; F2 | package dependency | R6-07 / AC6-07 / T6-07 / EV6-07 |
| `PhaseFScientificAdmissibilityAuditV1` | exact fields below; §3 ID/hash | scientific reviewer/checker; F2 | scientific_admissibility | R5-25 / AC5-25 / T5-25 / EV5-25 |
| `PhaseFCohortLockRecordV1` | §14 exact hashes; §3 ID/hash | campaign; cohort; F2 | cohort_locked | R5-26 / AC5-26 / T5-26 / EV5-26 |
| `PhaseFExecutionRecordV1` | §14 exact time/result; §3 ID/hash | release; execution; F4 | execution_registered | R5-27 / AC5-27 / T5-27 / EV5-27 |
| `PhaseFReleaseRecordV1` | §14 semantic-only fields; §3 ID; complete hash | release; release; F5 | release_registered external attestation | R6-01/R6-08 / AC6-01/08 / T6-01/08 / EV6-01/08 |
| `PhaseFClaimStateRecordV1` | §14 exact nullable fields, cause binding, and transition; §3 ID; complete hash | governance; state; F5+ | claim_state_changed external attestation | R7-07/R7-11 / AC7-07/11 / T7-07/11 / EV7-07/11 |
| `PhaseFReinstatementApprovalV1` | §14 review-bundle reference and trigger; §3 ID/hash | governance; reinstatement; F5+ | state dependency | R6-08 / AC6-08 / T6-08 / EV6-08 |
| `PhaseFMonitoringPolicyV1` | §14 metric vocabulary; §3 ID/hash | F0; monitoring; F5+ | release dependency | R5-31 / AC5-31 / T5-31 / EV5-31 |
| `PhaseFMonitoringRecordV1` | §14 derived result/window; §3 ID; complete hash | operations; monitoring; F5+ | monitoring_recorded external attestation | R7-04 / AC7-04 / T7-04 / EV7-04 |
| `PhaseFMonitoringMeasurementV1` | §14 exactly metric, typed value, and evidence SHA | operations; measurement validator; F5+ | nested in monitoring record | R7-04 / AC7-04 / T7-04 / EV7-04 |
| `PhaseFMonitoringBreachV1` | §14 exactly metric and derived breach code; no evidence field | monitoring checker; breach derivation; F5+ | nested in monitoring record | R7-04 / AC7-04 / T7-04 / EV7-04 |
| `PhaseFIncidentScopeV1` | §15 incident tagged union | governance; incident; all | nested in incident | R7-07/R7-08/R7-11 / AC7-07/08/11 / T7-07/08/11 / EV7-07/08/11 |
| `PhaseFIncidentRecordV1` | §15 exact scoped fields/enums; §3 ID/hash | operations/governance; incident; all | incident_recorded | R6-10 / AC6-10 / T6-10 / EV6-10 |
| `PhaseFRetentionScopeV1` | §15 exact release/campaign tagged union | retention auditor; scope validator; all | nested in retention audit | R7-08 / AC7-08 / T7-08 / EV7-08 |
| `PhaseFRetentionAuditV1` | §15 exact scope/check/result and typed relations; §3 ID/hash | operations; retention; all | retention_audit_recorded | R7-08 / AC7-08 / T7-08 / EV7-08 |
| `PhaseFRegistryCompromiseEmergencyV1` | §15 exact acyclic fields; §3 ID/hash; no Git self-fields | security/operations; emergency; claim-status | emergency input | R7-01 / AC7-01 / T7-01 / EV7-01 |
| `PhaseFPlanApprovalV1` | §6 exact ordered ASCII body fields | five roles; tag validator; plan gate | tag message hash | R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFDecisionApprovalV1` | §6 exact ordered ASCII body fields | five roles; tag validator; F0 gate | tag message hash | R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFReadinessApprovalV1` | §6 exact ordered ASCII body fields | five roles; tag validator; readiness gate | tag message hash | R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFAuthorityEnrollmentApprovalV1` | §6 exact ordered ASCII body fields | five roles; tag validator; enrollment gate | tag message hash | R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFTrustProvisioningApprovalV1` | §6 exact ordered ASCII body fields | five roles; tag validator; trust gate | tag message hash | R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFPhysicalReleaseApprovalV1` | §6 exact ordered ASCII body fields | five roles; tag validator; physical-release gate | tag message hash | R7-12 / AC7-12 / T7-12 / EV7-12 |

The six approval schemas are individually closed in the catalog below; the
wire is the exact ordered ASCII `format_version=1` body in §6. Each target is
the named gate target and each review bundle is a five-role aggregate-GO bundle
whose hash is the body field `review_bundle_sha256`.

| approval schema | exact field closure / wire encoding | producer / validator / stage | tag name / target | review bundle / requirement / AC / test / evidence |
|---|---|---|---|---|
| `PhaseFPlanApprovalV1` | §6 plan body fields, fixed order, ASCII plus final LF | independent reviewer / tag validator / plan gate | `ism-mechanism-health-v1-f-plan-approved` / reviewed R7 main | five-role plan review / R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFDecisionApprovalV1` | §6 F0 body fields, fixed order, ASCII plus final LF | independent reviewer / tag validator / F0 gate | `ism-mechanism-health-v1-f-f0-decisions-approved` / reviewed F0 main | five-role F0 review / R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFReadinessApprovalV1` | §6 readiness body fields, fixed order, ASCII plus final LF | independent reviewer / tag validator / readiness gate | `ism-mechanism-health-v1-f-readiness-approved` / integrated checker | five-role readiness review / R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFAuthorityEnrollmentApprovalV1` | §6 enrollment body fields, fixed order, ASCII plus final LF | independent reviewer / tag validator / enrollment gate | `ism-mechanism-health-v1-f-authority-enrollment-approved` / readiness main | five-role enrollment review / R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFTrustProvisioningApprovalV1` | §6 trust body fields, fixed order, ASCII plus final LF | independent reviewer / tag validator / trust gate | `ism-mechanism-health-v1-f-trust-provisioning-approved` / integrated F3 main | five-role trust review / R7-12 / AC7-12 / T7-12 / EV7-12 |
| `PhaseFPhysicalReleaseApprovalV1` | §6 physical-release body fields, fixed order, ASCII plus final LF | independent reviewer / tag validator / physical-release gate | `ism-mechanism-health-v1-f-physical-validation-released` / final F4/F5 main | five-role release review / R7-12 / AC7-12 / T7-12 / EV7-12 |

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

### 16.1 Historical R7 normative field-type audit

The following was the R7 field audit. It is retained for historical accounting;
the complete current R8 closure and metadata are in §26. `JSON_INTEGER_ONE` is the literal
JSON integer `1`; `SORTED_UNIQUE<T>` is a strictly increasing JSON array whose
member type is exactly `T`; `NONEMPTY_SORTED_UNIQUE<T>` adds nonempty; and
`JCS_OBJECT<T>` means the complete canonical object type `T`. These were closed
R7 constructions, retained for regression accounting. The current R8 catalog
is the single source of catalog authority in §26.

| Object | Every field and exact type |
|---|---|
| `PhaseFDecisionBundleV1` | `schema_version:JSON_INTEGER_ONE`; `decision_bundle_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `decisions:NONEMPTY_SORTED_UNIQUE<PhaseFDecisionRowV1>` |
| `PhaseFProtocolProjectionV1` | plan-only `decision_bundle_sha256:SHA256_V1`; `protocol_toml_sha256:SHA256_V1`; `runtime_protocol:MhiValidationProtocolV1`; `projection_result:PHASE_F_RESULT_V1` |
| `PhaseFDecisionRowV1` | `decision_id:RUNTIME_STABLE_ID_V1`; `value:PhaseFDecisionValueV1`; `decision_owner_role:PHASE_F_REVIEW_ROLE_V1`; `rationale_document_sha256:SHA256_V1` |
| `PhaseFReviewTargetV1` | tagged union: `{type:"git_commit",git_sha:GIT_SHA_V1}` or `{type:"external_object",object_kind:PHASE_F_OBJECT_KIND_V1,object_sha256:SHA256_V1}` |
| `PhaseFIndependentReviewBundleV1` | `schema_version:JSON_INTEGER_ONE`; `review_bundle_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `target:PhaseFReviewTargetV1`; `reviews:NONEMPTY_SORTED_UNIQUE<PhaseFIndependentReviewV1>`; `aggregate_p0_count:CANONICAL_UNSIGNED_INTEGER_V1`; `aggregate_p1_count:CANONICAL_UNSIGNED_INTEGER_V1`; `aggregate_decision:PHASE_F_DECISION_V1` |
| `PhaseFIndependentReviewV1` | `role:PHASE_F_REVIEW_ROLE_V1`; `decision:PHASE_F_DECISION_V1`; `p0_count:CANONICAL_UNSIGNED_INTEGER_V1`; `p1_count:CANONICAL_UNSIGNED_INTEGER_V1`; `finding_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `review_artifact_reference:PhaseFObjectReferenceV1` |
| `PhaseFAuthorityEnrollmentV1` | `schema_version:JSON_INTEGER_ONE`; `enrollment_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `phase_f_plan_tag:PHASE_F_TAG_NAME_V1`; `f0_decisions_tag:PHASE_F_TAG_NAME_V1`; `readiness_tag:PHASE_F_TAG_NAME_V1`; `owner_authority_id:RUNTIME_STABLE_ID_V1`; `registry_authority_id:RUNTIME_STABLE_ID_V1`; `owner_public_key:ED25519_PUBLIC_KEY_V1`; `registry_public_key:ED25519_PUBLIC_KEY_V1`; `owner_public_key_fingerprint:SHA256_V1`; `registry_public_key_fingerprint:SHA256_V1`; `owner_authority_document:PhaseFObjectReferenceV1`; `registry_authority_document:PhaseFObjectReferenceV1`; `custody_policy_sha256:SHA256_V1`; `created_at:UTC_SECOND_TIMESTAMP_V1` |
| `PhaseFCheckerBuildEvidenceV1` | `schema_version:JSON_INTEGER_ONE`; `build_evidence_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `build_ordinal:PHASE_F_CHECKER_BUILD_ORDINAL_V1`; `checker_source_review_sha:GIT_SHA_V1`; `checker_source_tree:GIT_TREE_V1`; `checker_dependency_lock_sha256:SHA256_V1`; `rustc_version:RUNTIME_CANONICAL_TEXT_V1`; `cargo_version:RUNTIME_CANONICAL_TEXT_V1`; `macos_uname:RUNTIME_CANONICAL_TEXT_V1`; `macos_arch:RUNTIME_CANONICAL_TEXT_V1`; `macos_product_version:RUNTIME_CANONICAL_TEXT_V1`; `macos_build_version:RUNTIME_CANONICAL_TEXT_V1`; `environment:SORTED_UNIQUE<PhaseFEnvironmentEntryV1>`; `build_command:RUNTIME_CANONICAL_TEXT_V1`; `build_transcript_sha256:SHA256_V1`; `checker_binary_sha256:SHA256_V1`; `fresh_source_directory:BOOLEAN_V1`; `fresh_target_directory:BOOLEAN_V1`; `fresh_home:BOOLEAN_V1`; `fresh_cargo_home:BOOLEAN_V1`; `cargo_home_config_absent:BOOLEAN_V1`; `result:PHASE_F_BUILD_RESULT_V1` |
| `PhaseFCheckerReadinessEvidenceV1` | `schema_version:JSON_INTEGER_ONE`; `readiness_evidence_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `build1_sha256:SHA256_V1`; `build2_sha256:SHA256_V1`; `checker_source_review_sha:GIT_SHA_V1`; `checker_source_tree:GIT_TREE_V1`; `checker_dependency_lock_sha256:SHA256_V1`; `checker_binary_sha256:SHA256_V1`; `f_maint_01_status:PHASE_F_MAINTENANCE_STATUS_V1`; `f_maint_02_status:PHASE_F_MAINTENANCE_STATUS_V1`; `result:PHASE_F_BUILD_RESULT_V1` |
| `PhaseFCheckerReportV1` | `schema_version:JSON_INTEGER_ONE`; `checker_binary_sha256:SHA256_V1`; `command:PhaseFCommandV1`; `argv:PhaseFArgvV1`; `input_sha256s:SORTED_UNIQUE<PhaseFNamedDigestV1>`; `decision:PHASE_F_CHECKER_DECISION_V1`; `diagnostic_codes:SORTED_UNIQUE<DIAGNOSTIC_CODE_V1>`; `stdout:PhaseFCheckerStdoutV1`; `exit_code:PhaseFCheckerExitCodeV1` |
| `PhaseFCommandV1` | verify: `{name:"verify",kind:PHASE_F_OBJECT_KIND_V1,input:PATH_V1,context_dir:PATH_V1,report:PATH_V1}`; claim-status: `{name:"claim-status",release:PATH_V1,context_dir:PATH_V1,registry_head_uri:LIVE_REGISTRY_HEAD_URI_V1,now:UTC_SECOND_TIMESTAMP_V1,report:PATH_V1,prior_head:PATH_V1|null,registry_compromised_emergency:PATH_V1|null,registry_compromised_review:PATH_V1|null,registry_compromised_commit:GIT_SHA_V1|null}`; emergency fields all null or all non-null |
| `PhaseFArgvV1` | exact ordered array derived from `PhaseFCommandV1`; prior-head pair precedes the emergency path/review/commit triples |
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
| `PhaseFPowerAnalysisRecordV1` | `schema_version:JSON_INTEGER_ONE`; `power_analysis_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `power_method_id:RUNTIME_STABLE_ID_V1`; `power_method_version:RUNTIME_CANONICAL_TEXT_V1`; `power_method_interface_sha256:SHA256_V1`; `software_source_sha:GIT_SHA_V1`; `software_binary_sha256:SHA256_V1`; `parameters:NONEMPTY_SORTED_UNIQUE<PhaseFParameterValueRowV1>`; `sensitivity_cases:SORTED_UNIQUE<PhaseFSensitivityCaseV1>`; `outputs:NONEMPTY_SORTED_UNIQUE<PhaseFPowerOutputValueV1>`; `created_at:UTC_SECOND_TIMESTAMP_V1`; complete file reviewed before registration |
| `PhaseFParameterValueRowV1` | `parameter_id:RUNTIME_STABLE_ID_V1`; `value:PHASE_F_PARAMETER_VALUE_V1` |
| `PhaseFSensitivityCaseV1` | `case_id:RUNTIME_STABLE_ID_V1`; `parameter_overrides:NONEMPTY_SORTED_UNIQUE<PhaseFSensitivityOverrideV1>`; `outputs:NONEMPTY_SORTED_UNIQUE<PhaseFPowerOutputValueV1>` |
| `PhaseFSensitivityOverrideV1` | `parameter_id:RUNTIME_STABLE_ID_V1`; `value:PHASE_F_PARAMETER_VALUE_V1` |
| `PhaseFPowerOutputValueV1` | `output_id:RUNTIME_STABLE_ID_V1`; `value:PHASE_F_PARAMETER_VALUE_V1` |
| `PhaseFMetrologyPolicyV1` | `schema_version:JSON_INTEGER_ONE`; `metrology_policy_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_policies:NONEMPTY_SORTED_UNIQUE<PhaseFEndpointMetrologyPolicyV1>` |
| `PhaseFEndpointMetrologyPolicyV1` | `endpoint_id:RUNTIME_STABLE_ID_V1`; `reference_type:PHASE_F_REFERENCE_TYPE_V1`; `allowed_methods:NONEMPTY_SORTED_UNIQUE<PhaseFMethodVersionV1>`; `allowed_authority_ids:NONEMPTY_SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `measurand_id:RUNTIME_STABLE_ID_V1`; `result_unit:UNIT_TEXT_V1`; `blinding_requirement:blinded_to_assessment`; `uncertainty_policy:PhaseFUncertaintyPolicyV1`; `lod_loq_policy:PhaseFLODLOQPolicyV1`; `calibration_policy:PhaseFCheckListV1`; `qc_policy:PhaseFCheckListV1`; `chain_of_custody_required:true`; `traceability_document_required:true`; `limitations_document_required:true` |
| `PhaseFMethodVersionV1` | `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1` |
| `PhaseFMetrologyCheckSpecV1` | `endpoint_id:RUNTIME_STABLE_ID_V1`; `check_id:RUNTIME_STABLE_ID_V1`; `check_kind:PHASE_F_CHECK_KIND_V1`; `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `procedure_document:PhaseFObjectReferenceV1`; `measurand_id:RUNTIME_STABLE_ID_V1`; `result_unit:UNIT_TEXT_V1`; `comparator:greater_than_or_equal|less_than_or_equal`; `threshold:RUNTIME_F64_V1`; `failure_action:PHASE_F_DEVIATION_ACTION_V1` |
| `PhaseFMetrologyCheckResultV1` | `schema_version:JSON_INTEGER_ONE`; `check_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_id:RUNTIME_STABLE_ID_V1`; `metrology_policy_sha256:SHA256_V1`; `check_id:RUNTIME_STABLE_ID_V1`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `performed_at:UTC_SECOND_TIMESTAMP_V1`; `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `measurand_id:RUNTIME_STABLE_ID_V1`; `value:RUNTIME_F64_V1`; `unit:UNIT_TEXT_V1`; `result:PHASE_F_CHECK_RESULT_V1` |
| `PhaseFReferenceSourceDescriptorV1` | `schema_version:JSON_INTEGER_ONE`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `source_file_sha256:SHA256_V1`; `evidence_origin:EvidenceOriginV1`; `dependency_completeness:ReferenceDependencyCompletenessV1`; `experiment_scope:ArtifactExperimentScope`; `acquisition_families:ArtifactAcquisitionFamilies`; `direct_dependencies:SORTED_UNIQUE<ReferenceDependencyV1>` |
| `PhaseFReferenceResultV1` | `schema_version:JSON_INTEGER_ONE`; `reference_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_id:RUNTIME_STABLE_ID_V1`; `reference_endpoint_id:RUNTIME_STABLE_ID_V1`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `reference_source_descriptor_sha256:SHA256_V1`; `reference_type:PHASE_F_REFERENCE_TYPE_V1`; `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `blinding_state:BlindingStateV1`; `uncertainty:PhaseFQuantifiedUncertaintyV1`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>`; `limitations_document_sha256:SHA256_V1`; `traceability_document_sha256:SHA256_V1`; `chain_of_custody_sha256:SHA256_V1`; mechanism branch adds `hypothesis_id:RUNTIME_STABLE_ID_V1,outcome:supports|contradicts|not_assessed|unavailable`; health branch adds `target:HealthTargetV1,label:RUNTIME_CANONICAL_TEXT_V1` |
| `PhaseFScientificAdmissibilityAuditV1` | `schema_version:JSON_INTEGER_ONE`; `scientific_admissibility_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `protocol_sha256:SHA256_V1`; `package_manifest_sha256:SHA256_V1`; `dependency_audit_sha256:SHA256_V1`; `identity_audit_sha256:SHA256_V1`; `reference_assessments:NONEMPTY_SORTED_UNIQUE<PhaseFReferenceAssessmentV1>`; `reviewer_role:scientific_metrology`; `result:PHASE_F_RESULT_V1` |
| `PhaseFReferenceAssessmentV1` | `reference_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_id:RUNTIME_STABLE_ID_V1`; `evidence_category:SCIENTIFIC_EVIDENCE_CATEGORY_V1`; `claim_ceiling:SCIENTIFIC_CLAIM_CEILING_V1`; `dependency_status:known_separated|known_overlap|unknown`; `identity_status:distinct|same|unknown`; `admissibility:physical_support_allowed|limited_only|not_assessed|unavailable|not_admissible` |
| `PhaseFCohortLockRecordV1` | `schema_version:JSON_INTEGER_ONE`; `cohort_lock_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `protocol_sha256:SHA256_V1`; `package_manifest_sha256:SHA256_V1`; `power_analysis_sha256:SHA256_V1`; `dependency_audit_sha256:SHA256_V1`; `physical_unit_ledger_sha256:SHA256_V1`; `identity_audit_sha256:SHA256_V1`; `location_ledger_sha256:SHA256_V1`; `chain_of_custody_sha256:SHA256_V1`; `deviation_ledger_sha256:SHA256_V1`; `metrology_policy_sha256:SHA256_V1`; `scientific_admissibility_audit_sha256:SHA256_V1`; `reference_result_sha256s:SORTED_UNIQUE<SHA256_V1>`; `reference_source_descriptor_sha256s:SORTED_UNIQUE<SHA256_V1>`; `locked_at:UTC_SECOND_TIMESTAMP_V1` |
| `PhaseFExecutionRecordV1` | `schema_version:JSON_INTEGER_ONE`; `execution_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `cohort_lock_record_sha256:SHA256_V1`; `owner_approval_file_sha256:SHA256_V1`; `protocol_sha256:SHA256_V1`; `deviation_ledger_sha256:SHA256_V1`; `release_code_sha:GIT_SHA_V1`; `checker_binary_sha256:SHA256_V1`; `validation_manifest_sha256:SHA256_V1`; `started_at:UTC_SECOND_TIMESTAMP_V1`; `completed_at:UTC_SECOND_TIMESTAMP_V1`; `result:PHASE_F_RESULT_V1` |
| `PhaseFReleaseRecordV1` | `schema_version:JSON_INTEGER_ONE`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `claim_statement:RUNTIME_CANONICAL_TEXT_V1`; `release_code_sha:GIT_SHA_V1`; `protocol_sha256:SHA256_V1`; `cohort_lock_record_sha256:SHA256_V1`; `owner_approval_file_sha256:SHA256_V1`; `execution_record_sha256:SHA256_V1`; `validation_manifest_sha256:SHA256_V1`; `monitoring_policy_sha256:SHA256_V1`; `metrology_policy_sha256:SHA256_V1`; `valid_from:UTC_SECOND_TIMESTAMP_V1`; `valid_until:UTC_SECOND_TIMESTAMP_V1`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>` |
| `PhaseFClaimStateRecordV1` | `schema_version:JSON_INTEGER_ONE`; `claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `previous_claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1|null`; `state:PHASE_F_CLAIM_STATE_V1`; `reason_code:PHASE_F_CLAIM_REASON_V1`; `cause_incident_sha256:SHA256_V1|null`; `effective_at:UTC_SECOND_TIMESTAMP_V1`; `superseding_release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1|null`; `activation_review_bundle_sha256:SHA256_V1|null`; `reinstatement_approval_sha256:SHA256_V1|null`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>` |
| `PhaseFReinstatementApprovalV1` | `schema_version:JSON_INTEGER_ONE`; `reinstatement_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `suspended_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `suspension_reason:PHASE_F_CLAIM_REASON_V1`; `required_corrective_action:RUNTIME_CANONICAL_TEXT_V1`; `corrective_evidence_sha256s:SORTED_UNIQUE<SHA256_V1>`; `execution_record_sha256:SHA256_V1`; `review_bundle_sha256:SHA256_V1` |
| `PhaseFMonitoringPolicyV1` | `schema_version:JSON_INTEGER_ONE`; `monitoring_policy_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `monitoring_interval_seconds:DURATION_SECONDS_V1`; `required_metrics:FIXED_ORDER<PHASE_F_MONITORING_METRIC_V1>`; `metric_thresholds:SORTED_UNIQUE<PhaseFMetricThresholdV1>`; `missing_monitoring_action:suspend`; `domain_breach_action:suspend`; `reference_qc_breach_action:suspend` |
| `PhaseFMetricThresholdV1` | `metric_id:PHASE_F_MONITORING_NUMERIC_METRIC_V1`; `comparator:greater_than_or_equal|less_than_or_equal`; `value:RUNTIME_F64_V1`; `unit:UNIT_TEXT_V1|null` |
| `PhaseFMonitoringRecordV1` | `schema_version:JSON_INTEGER_ONE`; `monitoring_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `window_start:UTC_SECOND_TIMESTAMP_V1`; `window_end:UTC_SECOND_TIMESTAMP_V1`; `policy_sha256:SHA256_V1`; `measurements:NONEMPTY_SORTED_UNIQUE<PhaseFMonitoringMeasurementV1>`; `breaches:SORTED_UNIQUE<PhaseFMonitoringBreachV1>`; `result:PHASE_F_MONITORING_RESULT_V1` |
| `PhaseFMonitoringMeasurementV1` | `metric_id:PHASE_F_MONITORING_METRIC_V1`; `value:PhaseFMonitoringValueV1`; `evidence_sha256:SHA256_V1` |
| `PhaseFMonitoringBreachV1` | `metric_id:PHASE_F_MONITORING_METRIC_V1`; `breach_code:PHASE_F_BREACH_CODE_V1` |
| `PhaseFIncidentScopeV1` | tagged union: `{type:"release",release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1}` or `{type:"campaign",campaign_id:RUNTIME_STABLE_ID_V1}` or `{type:"registry_namespace",registry_namespace_id:RUNTIME_STABLE_ID_V1}` |
| `PhaseFIncidentRecordV1` | `schema_version:JSON_INTEGER_ONE`; `incident_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `scope:PhaseFIncidentScopeV1`; `incident_type:PHASE_F_INCIDENT_TYPE_V1`; `detected_at:UTC_SECOND_TIMESTAMP_V1`; `affected_object_sha256s:SORTED_UNIQUE<PhaseFObjectDigestV1>`; `affected_unit_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `evidence_references:SORTED_UNIQUE<PhaseFObjectReferenceV1>`; `required_action:PHASE_F_INCIDENT_ACTION_V1`; `incident_status:PHASE_F_INCIDENT_STATUS_V1` |
| `PhaseFRetentionScopeV1` | tagged union: `{type:"release",release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1}` or `{type:"campaign",campaign_id:RUNTIME_STABLE_ID_V1,package_manifest_sha256:SHA256_V1}` |
| `PhaseFRetentionAuditV1` | `schema_version:JSON_INTEGER_ONE`; `retention_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `scope:PhaseFRetentionScopeV1`; `policy_sha256:SHA256_V1`; `audited_at:UTC_SECOND_TIMESTAMP_V1`; `object_checks:NONEMPTY_SORTED_UNIQUE<PhaseFRetentionObjectCheckV1>`; `result:PHASE_F_RESULT_V1` |
| `PhaseFRetentionObjectCheckV1` | `object_sha256:PhaseFObjectDigestV1`; `primary_available:BOOLEAN_V1`; `primary_verified:BOOLEAN_V1`; `verified_backup_count:CANONICAL_UNSIGNED_INTEGER_V1`; `last_backup_verification_at:UTC_SECOND_TIMESTAMP_V1`; `result:PHASE_F_RESULT_V1` |
| `PhaseFF5ReleaseCandidateV1` | `schema_version:JSON_INTEGER_ONE`; `f5_candidate_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `release_record_sha256:SHA256_V1`; `initial_claim_state_sha256:SHA256_V1`; `execution_record_sha256:SHA256_V1`; `cohort_lock_record_sha256:SHA256_V1`; `owner_approval_file_sha256:SHA256_V1`; `validation_manifest_sha256:SHA256_V1`; `trust_store_sha256:SHA256_V1`; `release_code_sha:GIT_SHA_V1`; `package_manifest_sha256:SHA256_V1`; `monitoring_policy_sha256:SHA256_V1`; `metrology_policy_sha256:SHA256_V1` |
| `PhaseFRegistryCompromiseEmergencyV1` | `schema_version:JSON_INTEGER_ONE`; `emergency_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `registry_namespace_id:RUNTIME_STABLE_ID_V1`; `incident_record_sha256:SHA256_V1`; `declared_at:UTC_SECOND_TIMESTAMP_V1`; `affected_claim_ids:NONEMPTY_SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `action:suspend_all_active_claims` |

The aliases used by this audit are defined exactly in §§2, 3, 5, 7, 9-15;
there is no free-form `string`, `integer`, `hash`, `value`, `status`, `policy`,
`record`, `document`, `object`, `relation`, or `role` field. The audit result is
`UNTYPED_NORMATIVE_FIELDS=0`.

For the historical readiness object, the field closure included
`readiness_evidence_id`; the construction is semantic payload -> derived ID ->
complete file -> file SHA. For claim state, `cause_incident_sha256` is
`SHA256_V1|null` with the reason-specific nullability in §14. For monitoring,
the measurement closure included `evidence_sha256` and the breach closure did
not. For retention, the audit closure included `scope:PhaseFRetentionScopeV1`
and never a top-level release ID. These are historical catalog entries; §26 is
the current normative catalog.

## 17. Valid object construction order

| Object | Inputs and canonicalization | ID / signature | Complete-file hash; relation; next |
|---|---|---|---|
| F0 decision bundle | F0 values, JCS, exact 20 IDs | §3; unsigned | hash; review/tag; protocol |
| review bundle | target bytes/commit, five rows, JCS | §3; unsigned | hash; tag `review_bundle_sha256`; approval tag |
| approval tag | exact annotated message/peeled target | no ID/signature | `git_tag_message` hash; preceding refs; next gate |
| checker readiness | two fresh builds/closed env/transcript; construct semantic payload first | semantic payload -> `readiness_evidence_id` using `mhi_phase_f_checker_readiness_evidence_v1\0` | complete readiness-evidence file hash; readiness review/tag; enrollment |
| authority enrollment | F0 IDs, readiness, key bytes, JCS | §3; intentionally unsigned | hash; enrollment tag; genesis |
| registry genesis | enrollment/F0 authority, sequence 0 | §8 signing bytes | signed hash; authority_enrolled; protocol |
| protocol registration | exact TOML/registration document | runtime protocol ID | hash; protocol_registered; power |
| power method/analysis | F0 method ID/version only; F1 interface; typed values; complete analysis before review | §3 IDs | interface hash; analysis hash; five-role review target equals analysis hash; `power_registered` with review relation; package |
| package/physical/custody/metrology | retrieved objects, role matrix, audits | §3 IDs | hashes; package relations; cohort |
| scientific audit | package/dependency/identity/ref assessments | §3 audit ID | hash; scientific_admissibility; cohort |
| cohort lock | exact prior hashes/lock time | §3 cohort ID | hash; cohort_locked; owner approval |
| owner approval | owner approval/cohort/enrollment | owner approval ID | hash; owner_approval_registered; execution |
| F3 trust provisioning | reviewed store blob/file/hash | no new root in plan | hashes; trust approval; F4 |
| F4 execution | locked cohort/approval/latest deviation/checker | §3 execution ID | hash; execution_registered; release |
| release record | semantic release payload excluding only `release_record_id` | §3 release ID | complete release-file hash; later `release_registered` subject is release ID/file hash; state |
| F5 candidate | frozen release/evidence hashes and exact candidate fields | §5.1 candidate ID | complete candidate-file hash; five-role F5 review bundle |
| initial active state | release/no prior/initial_release plus exact F5 `activation_review_bundle_sha256` | §3 claim-state ID | complete state-file hash; later `claim_state_changed` subject is state ID/file hash |
| monitoring pass | exact 15 metrics in `PHASE_F_MONITORING_METRIC_ORDER_V1`, typed values, one complete `PhaseFMonitoringEvidenceV1` object per measurement, evidence verification, recomputed pass | §3 monitoring ID | complete monitoring-file hash; exactly 15 later `depends_on+monitoring_evidence` relations in `monitoring_recorded` |
| incident | verified evidence/exact consequence | §3 incident ID | hash; incident_recorded; suspension |
| registry-compromise emergency | incident file complete -> emergency file with incident SHA -> five-role review targeting emergency file SHA -> later commit at deterministic paths | emergency ID excludes only own ID; no Git identity in file | local emergency/review bytes and later commit SHA are supplied separately; claim-status verifies tree and bytes -> NOT_ACTIVE |
| campaign retention | campaign package/manifest exists before release -> campaign-scoped audit -> reviewed abandonment incident -> deadline | §3 audit/incident IDs | campaign retention audit uses package-manifest relation and no release ID |
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

## 18. Historical R7 positive controls and complete DAG construction audit

This section is retained for R7 accounting only and is non-normative. The
current positive-path authority is §26; where this historical text differs,
§26 controls.

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

The complete R7 audit sequence is also explicit for exceptional paths:

```text
PLAN -> plan review -> plan tag
F0 -> decision bundle -> review -> F0 tag
READINESS -> build evidence -> readiness evidence -> review -> readiness tag
ENROLLMENT -> review -> enrollment tag -> genesis
F1 -> protocol -> power interface -> power analysis -> power review -> registry
F2 -> package/scientific/metrology authority -> cohort lock
F3 -> owner approval -> trust
F4 -> execution
F5 -> release -> candidate -> review -> initial ACTIVE state -> registries -> tag
OPERATIONS -> monitoring PASS -> registry -> live head -> ACTIVE
EMERGENCY -> incident -> emergency file -> review -> later Git commit -> NOT_ACTIVE
PRE-RELEASE RETENTION -> campaign audit -> abandonment incident -> deadline
```

No edge in this sequence requires a future object hash, self Git identity,
untyped review target, missing monitoring evidence, or a release ID before a
release exists. Therefore `COMPLETE_VALID_DAG_CONSTRUCTIBLE=yes` and
`POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES=0`.

The R7 positive-path probes are constructible before counterexample review:
release, state, monitoring, emergency, and retention files each complete before
their later attestations; deviation revision ID is one-way from stable ledger
ID; F5 review precedes initial-state registration; initial ACTIVE precedes first
monitoring due; power analysis review precedes `power_registered`; and a
reference result projects exactly to the current runtime endpoint. Failure of
any probe is P1.

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

### 18.1 Historical R7 positive controls

Monitoring PASS control: use `release_001`, policy `policy_001`,
`window_start=2026-01-01T00:00:00Z`, `window_end=2026-01-01T00:30:00Z`, and a
due boundary after the window. Construct exactly these 15 measurement rows,
each with the named typed value and a present, immutable evidence object:
In this fixture, every `<...>` denotes a concrete value already materialized
in the named context (not an omitted member); all five thresholds are exact
`less_than_or_equal` comparisons to binary64 `1` and all five numeric values
are binary64 `0` (`decimal:"0",binary64_bits_hex:"0000000000000000"`).

| metric | typed passing value | evidence object |
|---|---|---|
| `domain_compliance` | `{type:"status",value:"compliant"}` | domain QC result |
| `reference_qc_status` | `{type:"status",value:"pass"}` | reference QC result |
| `calibration_status` | `{type:"status",value:"pass"}` | calibration result |
| `sensor_drift` | `{type:"quantity",value:{decimal:"0",binary64_bits_hex:"0000000000000000"},unit:<policy unit>}` | drift calculation |
| `invalid_input_rate` | `{type:"rate",value:<RUNTIME_F64_V1>}` | input-quality calculation |
| `indeterminate_rate` | `{type:"rate",value:<RUNTIME_F64_V1>}` | indeterminate-rate calculation |
| `data_quality_insufficient_rate` | `{type:"rate",value:<RUNTIME_F64_V1>}` | data-quality calculation |
| `exclusion_rate` | `{type:"rate",value:<RUNTIME_F64_V1>}` | exclusion calculation |
| `reference_uncertainty_status` | `{type:"status",value:"within_limit"}` | uncertainty assessment |
| `software_git_sha` | `{type:"git_sha",value:<GIT_SHA_V1>}` | release build record |
| `checker_binary_sha256` | `{type:"sha256",value:<SHA256_V1>}` | checker build evidence |
| `trust_store_sha256` | `{type:"sha256",value:<SHA256_V1>}` | trust-store bytes |
| `trust_root_id` | `{type:"stable_id",value:<RUNTIME_STABLE_ID_V1>}` | approved trust object |
| `owner_approval_id` | `{type:"external_digest_id",value:<approval ID>}` | owner approval object |
| `release_record_id` | `{type:"external_digest_id",value:<release ID>}` | exact release object |

The historical numeric values are on the passing side of their exact policy
comparators. The historical binding values equal the exact release authority;
all 15 evidence references resolve and validate; `breaches=[]`, `result=pass`, and
`window_start < window_end <= due`. The complete monitoring file is hashed,
then registered by `monitoring_recorded`; this control has no missing evidence
object and therefore `MONITORING_PASS_CONSTRUCTIBLE=yes`.

Monitoring failure control: keep the complete measurement/evidence set but set
`domain_compliance` to `out_of_domain`. Recompute exactly
`breaches=[{metric_id:"domain_compliance",breach_code:"unhealthy_status"}]`
and `result=suspend`. The evidence remains on the measurement row and is not
duplicated in the breach row.

Pre-release retention control: with `campaign_id=campaign_001`, a present
package manifest hash, and no release, construct a PASS
`PhaseFRetentionAuditV1` with
`scope={type:"campaign",campaign_id:"campaign_001",
package_manifest_sha256:<manifest SHA>}`. Register it with exactly
`authorized_by+decision_bundle` and `references+package_manifest`, with no
release relation. Add a reviewed `campaign_abandonment` incident; continue
campaign-scoped audits through `detected_at + retention_seconds`. This path
passes without constructing a release ID.

Metrology lookup control: place `check_id=qc-01` under endpoint A and endpoint
B in one policy. Results carry the policy SHA, endpoint ID, and `qc-01`; each
selects exactly its endpoint's specification and passes. Mutating a result to
endpoint B while retaining endpoint A's threshold selects the wrong spec and
is NO-GO.

Power review control: construct and hash the complete F1 analysis, create the
five-role bundle targeting exactly that analysis file hash, require aggregate
GO/P0=0/P1=0, and only then create `power_registered` with both
`authorized_by+independent_review_bundle` and `depends_on+power_method_interface`.
An analysis registered without that bundle is rejected.

Owner-compromise control: construct a release-scoped `key_compromise` incident,
then a suspended state with `reason_code=key_compromise` and
`cause_incident_sha256=<exact incident file SHA>`. Its registry relations are
exactly `changes_state_of+release_record`, `registered_after+claim_state`, and
`depends_on+incident_record` naming that SHA. No owner signature is required;
the registry/governance authority supplies the non-active transition.

Registry-compromise control: complete the incident, emergency file, and
five-role emergency review in that order. Publish the two already-complete
files in a later commit at the deterministic digest paths, then pass their
local paths and commit SHA to claim-status. It verifies ancestry, paths, and
byte equality and returns `NOT_ACTIVE`; adding the commit SHA to either file
is a schema failure. This proves
`REGISTRY_COMPROMISE_GIT_FIXED_POINT_CYCLES=0` and
`COMPOSITE_REVIEW_TARGET_PATHS=0`.

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

The following R6 requirements and traceability rows are retained as historical
cross-references only. The R7 rows in §19.2 are also historical. The current
operational authority is the R8 requirement matrix in §27. Each historical row
has one primary AC, one test, and one evidence item; no historical row changes
the R8 contract. Every F-OD-01 through F-OD-20 is mapped only by the current
R8 matrix; no additional owner decision exists.

### 19.1 Historical R6 executable requirement traceability

Every historical R6 row below contains the complete requirement-to-path mapping;
it is retained for regression accounting, while §19.2 is also historical.
`owner_decision_ids` is `none` only when no F0 decision is involved.

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
| `F-PLAN-R6-P1-09` | Historical R6 monitoring threshold and breach-set contract, due-boundary windows, and pre-first-window grace period. | F-OD-19 | MonitoringPolicy, MonitoringRecord | monitoring producer, registry, currentness evaluator | AC6-09 | T6-09 | EV6-09 | operations_governance | F5+ |
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

### 19.2 Historical R7 executable requirement traceability

The following R7 requirements were normative in R7 and are retained only for
history. Each row has one complete
requirement/AC/test/evidence chain; the detailed tables that follow are part of
the requirement, not identifier-only cross-references.

| requirement ID | normative statement | F-OD mapping | schemas | stage | review role | primary AC | test | evidence |
|---|---|---|---|---|---|---|---|---|
| `R7-01` | Emergency files are complete, reviewed as one external object, and published only by a later Git outer attestation at deterministic paths. | F-OD-14,F-OD-17 | PhaseFIncidentRecordV1, PhaseFRegistryCompromiseEmergencyV1, PhaseFIndependentReviewBundleV1 | emergency/claim-status | security | AC7-01 | T7-01 | EV7-01 |
| `R7-02` | Every review bundle has exactly one tagged target union: Git commit or external object; external objects never require a Git SHA. | none | PhaseFReviewTargetV1, PhaseFIndependentReviewBundleV1 | all review gates | architecture_data | AC7-02 | T7-02 | EV7-02 |
| `R7-03` | F0 contains only power method ID/version; F1 constructs and binds the interface before analysis. | F-OD-12 | PhaseFDecisionBundleV1, PhaseFPowerMethodInterfaceV1, PhaseFPowerAnalysisRecordV1 | F0-F1 | scientific_metrology | AC7-03 | T7-03 | EV7-03 |
| `R7-04` | Every monitoring measurement has a typed value and evidence SHA; PASS is derived only when all evidence verifies. | F-OD-19 | PhaseFMonitoringPolicyV1, PhaseFMonitoringRecordV1, PhaseFMonitoringMeasurementV1, PhaseFMonitoringBreachV1 | F5+ | operations_governance | AC7-04 | T7-04 | EV7-04 |
| `R7-05` | Review rows use immutable artifact references and role uniqueness; no undefined review-row identity exists. | none | PhaseFIndependentReviewV1, PhaseFIndependentReviewBundleV1 | all review gates | architecture_data | AC7-05 | T7-05 | EV7-05 |
| `R7-06` | Readiness evidence derives its ID from the semantic payload, inserts it into the complete file, and then hashes/reviews that file. | none | PhaseFCheckerReadinessEvidenceV1 | readiness | compatibility | AC7-06 | T7-06 | EV7-06 |
| `R7-07` | Claim-state cause hashes and registry relations are exact for every reason, with all inapplicable tuples forbidden. | F-OD-16 | PhaseFClaimStateRecordV1, PhaseFIncidentRecordV1, PhaseFRegistryRecordV1, PhaseFRegistryRelationV1 | F5+ | operations_governance | AC7-07 | T7-07 | EV7-07 |
| `R7-08` | Retention has release and campaign scopes, and campaign audits are constructible before any release exists. | F-OD-20 | PhaseFRetentionScopeV1, PhaseFRetentionAuditV1, PhaseFIncidentRecordV1, PhaseFRegistryRecordV1 | all stages | security | AC7-08 | T7-08 | EV7-08 |
| `R7-09` | Metrology check specifications and results resolve by policy, endpoint, and check ID, so duplicate check IDs across endpoints are deterministic. | F-OD-11 | PhaseFMetrologyPolicyV1, PhaseFMetrologyCheckSpecV1, PhaseFMetrologyCheckResultV1 | F0-F2 | scientific_metrology | AC7-09 | T7-09 | EV7-09 |
| `R7-10` | Power analysis is complete and scientifically reviewed before its registry record; the registry relation binds the review target to the exact analysis file hash. | F-OD-12 | PhaseFPowerAnalysisRecordV1, PhaseFIndependentReviewBundleV1, PhaseFRegistryRecordV1 | F1 | scientific_metrology | AC7-10 | T7-10 | EV7-10 |
| `R7-11` | Owner-key compromise uses an incident-bound governance state transition and does not require the compromised owner signature. | F-OD-16 | PhaseFIncidentRecordV1, PhaseFClaimStateRecordV1, PhaseFRegistryRecordV1 | F5+ | security | AC7-11 | T7-11 | EV7-11 |
| `R7-12` | The master catalog individually enumerates all six approval schemas and every normative nested schema with closure, wire, authority, stage, and traceability. | F-OD-01..F-OD-20 | all catalogued PhaseF schemas and six approval bodies | plan review | architecture_data | AC7-12 | T7-12 | EV7-12 |
| `R7-13` | The current F0 contract is exactly 20 owner decisions and no normative stale 21-decision or F-OD-21 reference remains. | F-OD-01..F-OD-20 | PhaseFDecisionBundleV1, PhaseFDecisionRowV1 | F0 | architecture_data | AC7-13 | T7-13 | EV7-13 |

| AC ID | preconditions | exact input files/objects | exact operation | expected result | expected output | failure oracle |
|---|---|---|---|---|---|---|
| `AC7-01` | incident, emergency, review, and later commit are complete | exact incident/emergency/review bytes and commit tree | validate emergency ID, review target/aggregate, ancestry, deterministic paths, and byte equality | NOT_ACTIVE | verified emergency command report | any Git self-field, composite target, wrong path, ancestry, or byte mismatch -> reject |
| `AC7-02` | one review target and five role rows exist | Git commit target or external object target plus bundle | parse closed tagged union and validate target hash | exactly one deterministic target | valid review bundle | nullable/meaningless Git SHA or multiple targets -> reject |
| `AC7-03` | F0 decision bundle is complete and F1 interface exists | F0 row 12, interface, analysis | assert F0 row closure; build interface; compare ID/version; construct analysis | zero future F0 references | F0 projection and F1 interface/analysis files | interface hash/length in F0 or chronology reversal -> reject |
| `AC7-04` | policy defines 15 metrics and context contains evidence | monitoring record, 15 evidence objects, policy, release | validate cardinality/types/evidence, recompute thresholds/status/bindings/breaches | PASS with empty breach set | accepted monitoring record | missing/invalid evidence or declared/recomputed mismatch -> `missing_evidence`/suspend |
| `AC7-05` | five roles and immutable review artifacts exist | review rows with and without proposed row IDs | parse exact row closure and role uniqueness | rows validate without row ID | five-row review bundle | any review-row ID or ambiguous target -> schema reject |
| `AC7-06` | readiness semantic payload and two build hashes exist | readiness payload/file and build evidence | derive ID excluding only own ID, insert, hash complete file, review exact hash | readiness file and tag bind same hash | readiness evidence | construction text omits ID or hash precedes insertion -> plan consistency failure |
| `AC7-07` | release, incident, prior state, and cause file are valid | state file and exact registry relations | recompute reason transition and exact relation set | incident-driven state validates only with matching incident relation | suspended state and registry record | missing cause hash/relation or inapplicable tuple -> reject |
| `AC7-08` | campaign package/manifest exists and no release exists | campaign retention audit, package manifest, decision bundle | validate campaign scope and register package relation; append reviewed abandonment | PASS without release ID | campaign audit/incident chain | campaign audit requires release relation or release-only scope -> reject |
| `AC7-09` | one policy has endpoint A and B, each with `qc-01` | policy plus endpoint-qualified specs/results | resolve `(policy SHA, endpoint ID, check ID)` and recompute result | both endpoint results PASS | endpoint-qualified check results | endpoint mutation selects wrong/missing spec -> reject |
| `AC7-10` | complete analysis and independent scientific review exist | analysis, five-role bundle, power registry record | compare bundle target to subject hash and validate exact relations | registration accepted only after review | bound power registry record | missing review relation or target mismatch -> reject |
| `AC7-11` | key-compromise incident and active prior state exist | incident, suspended state, prior-state and incident registry records | validate cause hash, reason, transition, and governance signature authority | suspended transition validates without owner signature | incident/state registry chain | missing incident hash/relation or owner-signature requirement -> reject |
| `AC7-12` | catalog and schema list are present | master catalog, field audit, approval rows | enumerate and compare every required schema and closure/traceability field | six approvals and all new types individually listed | catalog consistency report | wildcard approval row or absent schema -> catalog KAT fail |
| `AC7-13` | normative document text is available | F0 table, catalog, full plan text | count F-OD rows and scan forbidden stale tokens | exactly 20; no F-OD-21/21-decision normative token | plan consistency report | any stale 21 reference -> plan consistency failure |

| test ID | fixture construction | checker/function invocation | exact expected result | negative mutation |
|---|---|---|---|---|
| `T7-01` | complete incident/emergency/review, then commit both files at digest paths | emergency claim-status verifier | NOT_ACTIVE; acyclic PASS | add repository commit to emergency or publish wrong path -> reject |
| `T7-02` | external target with object kind/hash only; Git target separately | review-bundle validator | external target PASS; Git target PASS | external target adds `git_sha` or nullable target SHA -> reject |
| `T7-03` | F0 row 12 plus later interface/analysis fixtures | F0 consistency and F1 chronology validator | F0/F1 positive path PASS | add interface hash/length to F0 -> reject |
| `T7-04` | 15 passing measurements with 15 valid evidence objects | monitoring recomputation | PASS, `breaches=[]` | remove one evidence SHA/object -> `missing_evidence`, suspend |
| `T7-05` | five role rows with artifact references and no row IDs | review schema validator | five-row aggregate GO | add `review_instance_id` -> schema reject |
| `T7-06` | semantic readiness payload and complete ID/file hash | readiness construction audit | ID in body, exact file hash, review/tag binding | omit readiness ID from construction table -> plan consistency fail |
| `T7-07` | incident-driven state plus exact release/prior-state relations | state/registry validator | PASS | remove `depends_on+incident_record` or alter cause hash -> reject |
| `T7-08` | campaign audit for `campaign_001`, manifest, no release, reviewed abandonment | retention/registry validator | campaign audit PASS | require release ID or add release relation -> reject |
| `T7-09` | endpoint A/B both define `qc-01`; results carry endpoint and policy SHA | metrology lookup/recompute | both correct results PASS | endpoint B result uses endpoint A threshold -> reject |
| `T7-10` | complete power analysis, exact-target five-role review, registry record | power registration validator | registration PASS | register analysis without review relation -> reject |
| `T7-11` | key-compromise incident, active prior state, suspended next state | claim-state transition validator | suspended state PASS without owner signature | remove cause hash or incident relation -> reject |
| `T7-12` | catalog contains six separate approval rows and new nested types | plan catalog KAT | zero orphan schemas | replace six rows with wildcard -> catalog fail |
| `T7-13` | full plan text and F0 table | normative token/count scanner | exactly 20 and no stale token | inject normative `F-OD-21` or `decision_count=21` -> plan fail |

| evidence ID | real artifact | producer | immutable identity | review/verification oracle |
|---|---|---|---|---|
| `EV7-01` | incident, emergency, review files and later Git commit/tree | security/operations and repository governance | complete file SHAs, review SHA, commit SHA, deterministic tree paths | checker proves target, ancestry, paths, and local byte equality |
| `EV7-02` | canonical review bundles and immutable review artifacts | five independent reviewers | bundle file SHA and object reference SHA/length | strict tagged-union and five-row validation |
| `EV7-03` | F0 decision bundle, F1 interface, analysis chronology | decision authority/statistician | complete canonical file SHAs and method IDs | no future object identity in F0; interface ID/version equality |
| `EV7-04` | 15 metric evidence objects and monitoring record | operations and domain/QC/build authorities | evidence SHA per measurement and monitoring file SHA | context retrieval plus exact metric recomputation |
| `EV7-05` | five review artifacts with role assignments | independent review roles | immutable `PhaseFObjectReferenceV1` values | no row ID, one row per role, artifact resolves |
| `EV7-06` | readiness payload, derived-ID transcript, complete file, review/tag | checker builder and readiness reviewer | readiness ID/file SHA/review SHA | recompute domain hash and exact tag binding |
| `EV7-07` | incident, state, prior state, signed registry record | governance/registry authority | incident/state/registry complete hashes | cause SHA and exact relation tuple equality |
| `EV7-08` | campaign manifest, campaign audit, abandonment incident | retention auditor/governance | manifest/audit/incident complete hashes | campaign relation has package manifest and no release |
| `EV7-09` | endpoint policy, duplicate-ID specs, endpoint-qualified results | metrology authority/laboratory | policy/check-result complete hashes | resolve one spec by policy+endpoint+check and recompute comparator |
| `EV7-10` | complete power analysis, five-role review, power registry record | statistician/scientific reviewers/registry | analysis file SHA, bundle SHA, registry record SHA | target SHA equals subject SHA and relation is present |
| `EV7-11` | key-compromise incident and governance state transition | security/governance/registry authority | incident/state/registry hashes | exact cause binding and no compromised-key signature dependency |
| `EV7-12` | master catalog, field audit, approval-schema rows | plan author and independent catalog reviewer | R7 plan SHA/blob and catalog transcript | every required schema has closure, wire, producer, validator, stage, target, review, requirement, AC, test, evidence |
| `EV7-13` | full R7 plan and F0 decision table | plan author and independent plan reviewer | plan SHA/blob and consistency transcript | exact 20 rows and forbidden-token scan |

`TRACEABILITY_SUBSTANCE_GAPS=0`, `UNMAPPED_REQUIREMENTS=0`, `UNMAPPED_ACS=0`,
`UNMAPPED_TESTS=0`, `UNMAPPED_EVIDENCE=0`, and `UNMAPPED_ODS=0` are required.

### 19.3 Historical R7 owner-decision mapping

This R7 owner-decision table is retained only as historical accounting. It is
not a current mapping source; the only current owner-decision edges are the
`owner_decision_ids` cells in §27. The F0 table remains the decision contract,
not a requirement mapping.

| owner decision | current requirement IDs |
|---|---|
| `F-OD-01` | `R7-03,R7-12,R7-13` |
| `F-OD-02` | `R7-12,R7-13` |
| `F-OD-03` | `R7-04,R7-12,R7-13` |
| `F-OD-04` | `R7-04,R7-12,R7-13` |
| `F-OD-05` | `R7-09,R7-12,R7-13` |
| `F-OD-06` | `R7-09,R7-12,R7-13` |
| `F-OD-07` | `R7-07,R7-12,R7-13` |
| `F-OD-08` | `R7-12,R7-13` |
| `F-OD-09` | `R7-12,R7-13` |
| `F-OD-10` | `R7-07,R7-12,R7-13` |
| `F-OD-11` | `R7-09,R7-12,R7-13` |
| `F-OD-12` | `R7-03,R7-10,R7-12,R7-13` |
| `F-OD-13` | `R7-11,R7-12,R7-13` |
| `F-OD-14` | `R7-01,R7-07,R7-12,R7-13` |
| `F-OD-15` | `R7-12,R7-13` |
| `F-OD-16` | `R7-07,R7-11,R7-12,R7-13` |
| `F-OD-17` | `R7-07,R7-12,R7-13` |
| `F-OD-18` | `R7-07,R7-12,R7-13` |
| `F-OD-19` | `R7-04,R7-12,R7-13` |
| `F-OD-20` | `R7-08,R7-12,R7-13` |

`UNMAPPED_ODS=0`.

## 20. Cumulative normative counterexamples

Every historical case remains independently replayable. `NO-GO` means checker
failure; public claim is NOT_ACTIVE except exact ACTIVE or
AUTHORITY_UNAVAILABLE. R6 cases are appended, not substituted.
R1-R6 rows are historical, non-normative regression fixtures; they do not
reintroduce superseded fields or relations into the current R8 schemas.
The exact stale-token strings in the R7 negative fixtures below are
non-normative test inputs only; they do not define the F0 contract.

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
| R7-CX-01 | emergency file contains `repository_commit_sha` | schema reject |
| R7-CX-02 | emergency file contains `repository_blob` | schema reject |
| R7-CX-03 | external review target contains `git_sha` | schema reject |
| R7-CX-04 | external review target contains object kind/hash only | PASS |
| R7-CX-05 | F-OD-12 contains a power-interface object reference | schema reject |
| R7-CX-06 | F-OD-12 contains only method ID/version | PASS |
| R7-CX-07 | monitoring measurement has no evidence SHA | reject; `missing_evidence` |
| R7-CX-08 | all 15 monitoring measurements have valid evidence and pass | PASS |
| R7-CX-09 | readiness construction table omits `readiness_evidence_id` | plan consistency failure |
| R7-CX-10 | reinstatement lacks `depends_on+reinstatement_approval` | registry record reject |
| R7-CX-11 | superseded state lacks `supersedes+release_record` | reject |
| R7-CX-12 | key-compromise state lacks cause incident SHA | reject |
| R7-CX-13 | campaign-scoped retention audit has no release ID | PASS |
| R7-CX-14 | release-scoped retention audit lacks release relation | reject |
| R7-CX-15 | endpoints A and B both use `qc-01`, and result A includes endpoint A | PASS |
| R7-CX-16 | endpoint B result uses endpoint A threshold | reject |
| R7-CX-17 | power registry record lacks analysis review relation | reject |
| R7-CX-18 | review target equals exact power-analysis SHA and registry relation binds bundle | PASS |
| R7-CX-19 | master catalog uses wildcard `six approval schemas` | catalog consistency failure |
| R7-CX-20 | all six approval schemas are individually enumerated | PASS |
| R7-CX-21 | normative text contains `F-OD-08..21` | plan consistency failure |
| R7-CX-22 | normative text contains `F-OD-08..20` | PASS |
| R7-CX-23 | emergency and review files are committed after both are complete | acyclic PASS |
| R7-CX-24 | `review_instance_id` is present without a defined derivation | schema reject because field is removed |
| R7-CX-25 | owner compromise incident/state/relation triple is exact | PASS |

## 21. Historical R6 remediation ledger

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

### 21.1 R7 remediation ledger

Only a fresh independent R7 reviewer may close a finding. The author
disposition is limited to `REMEDIATED` or `OPEN`; no author row uses `CLOSED`.

| R7 remediation ID | R6 finding | R7 exact section | root cause | R7 remediation | requirement | AC | test | evidence | AUTHOR DISPOSITION |
|---|---|---|---|---|---|---|---|---|---|
| `F-PLAN-R7-P1-01` | registry-compromise emergency Git fixed-point cycle | §15,§17,§18.1 | emergency file carried later commit/blob attestations and review linkage | remove all later attestations from emergency; review complete file; publish both files in later deterministic commit | R7-01 | AC7-01 | T7-01 | EV7-01 | REMEDIATED |
| `F-PLAN-R7-P1-02` | review bundle target not deterministic for external objects | §2,§5,§19.2 | split nullable target fields allowed meaningless combinations | replace with exactly one tagged `PhaseFReviewTargetV1` union | R7-02 | AC7-02 | T7-02 | EV7-02 | REMEDIATED |
| `F-PLAN-R7-P1-03` | F-OD-12 required a future F1 interface reference | §4,§12,§17 | F0 selected an object that does not exist until F1 | retain only method ID/version in F0; construct interface in F1 | R7-03 | AC7-03 | T7-03 | EV7-03 | REMEDIATED |
| `F-PLAN-R7-P1-04` | monitoring PASS required evidence rows could not carry | §14,§18.1 | measurement closure had only metric/value | add exactly one evidence SHA to every measurement | R7-04 | AC7-04 | T7-04 | EV7-04 | REMEDIATED |
| `F-PLAN-R7-P1-05` | review target Git and review-instance identity undefined | §2,§5,§16.1,§19.2 | review rows had an unconstructed second identity | remove row ID and make artifact reference the immutable evidence identity | R7-05 | AC7-05 | T7-05 | EV7-05 | REMEDIATED |
| `F-PLAN-R7-P1-06` | readiness evidence ID contradicted construction table | §7,§16.1,§17 | table said the derived ID was absent from the body | document semantic-payload -> ID -> complete-file -> SHA construction | R7-06 | AC7-06 | T7-06 | EV7-06 | REMEDIATED |
| `F-PLAN-R7-P1-07` | claim-state incident/reinstatement/supersession tuples were not exact | §9,§14,§16.1,§18.1 | transition authority was distributed across prose | add cause SHA and exact relation contract per reason | R7-07 | AC7-07 | T7-07 | EV7-07 | REMEDIATED |
| `F-PLAN-R7-P1-08` | pre-release retention was release-only | §15,§17,§18.1 | retention audit had only a release ID | add release/campaign scope union and scope-specific relations | R7-08 | AC7-08 | T7-08 | EV7-08 | REMEDIATED |
| `F-PLAN-R7-P1-09` | metrology check endpoint/spec identity was ambiguous | §13,§16.1,§18.1 | check ID was treated as globally unique | add endpoint/policy fields and exact tuple lookup | R7-09 | AC7-09 | T7-09 | EV7-09 | REMEDIATED |
| `F-PLAN-R7-P1-10` | power scientific review was not bound to analysis authority | §9,§12,§17,§18.1 | registry record omitted an independent-review relation | require five-role review target equal to analysis subject SHA before registration | R7-10 | AC7-10 | T7-10 | EV7-10 | REMEDIATED |
| `F-PLAN-R7-P1-11` | owner-key compromise lacked incident/evidence binding | §14,§18.1 | non-active transition could be asserted without exact cause | add cause incident SHA and `depends_on+incident_record`; use governance authority | R7-11 | AC7-11 | T7-11 | EV7-11 | REMEDIATED |
| `F-PLAN-R7-P1-12` | master catalog orphaned approval schemas and new types | §16,§16.1,§19.2 | one wildcard row hid six independent wire contracts | enumerate six approval rows and every new normative type | R7-12 | AC7-12 | T7-12 | EV7-12 | REMEDIATED |
| `F-PLAN-R7-P1-13` | stale 21-decision F0 language contradicted contract | §4,§19.2,§20 | prior prose was not globally reconciled | use exact F-OD-08..20 and enforce 20-row/no-stale-token scan | R7-13 | AC7-13 | T7-13 | EV7-13 | REMEDIATED |

## 22. Historical R7 author audit (non-normative)

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
SELF_GIT_IDENTITY_CYCLES=0
REGISTRY_COMPROMISE_GIT_FIXED_POINT_CYCLES=0
EMERGENCY_REVIEW_TARGET_AMBIGUITIES=0
COMPOSITE_REVIEW_TARGET_PATHS=0
REVIEW_TARGET_AMBIGUITIES=0
REVIEW_INSTANCE_ID_AMBIGUITIES=0
F0_F1_FUTURE_OBJECT_DEPENDENCY_PATHS=0
F0_FUTURE_OBJECT_REFERENCE_PATHS=0
READINESS_ID_CONTRADICTIONS=0
MONITORING_EVIDENCE_AMBIGUITIES=0
CLAIM_STATE_CAUSE_BINDING_AMBIGUITIES=0
CLAIM_STATE_RELATION_AMBIGUITIES=0
RETENTION_REGISTRY_RELATION_AMBIGUITIES=0
PRE_RELEASE_RETENTION_POSITIVE_PATH_AMBIGUITIES=0
METROLOGY_CHECK_LOOKUP_AMBIGUITIES=0
POWER_SCIENTIFIC_REVIEW_BINDING_AMBIGUITIES=0
ORPHAN_EXTERNAL_SCHEMAS=0
STALE_F0_NORMATIVE_REFERENCES=0
NORMATIVE_F0_COUNT_CONTRADICTIONS=0
```

The constructive audit asks for one valid instance of every catalog schema,
every semantic ID, every complete-file hash, both signing payloads, one genesis-
through-active chain, monitoring pass/breach, permitted reinstatement,
retention audit, reference/runtime projection, power analysis, metrology check,
checker invocation/report, and pusher-independent tag. Any guess is a P1.

## 23. Historical R7 validation and handoff

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
`docs(plan): close Phase F external positive paths`. Do not amend, reset, rebase,
squash, force-push, tag, create an implementation branch, start F0, generate
keys/signatures, provision trust, or create evidence/registry/monitoring/claim
records. Immediately before push, verify local `main`, `origin/main`, and live
remote `main` all equal the required R6 starting SHA; if live remote cannot be
verified, stop before push. Then push `main` normally. After a successful push,
record the R7 commit SHA, plan SHA-256, and Git blob and require a clean
worktree. No later commit occurs before fresh independent R7 rereview.

## 24. Historical R7 rereview gate

R7 remains unapproved pending a new independent reviewer. That reviewer must
begin by constructing these positive controls: (1) an external-object review
bundle with no Git SHA; (2) F0 with no F1 power-file identity; (3) monitoring
PASS with evidence for all 15 measurements; (4) pre-release campaign retention;
(5) duplicate check IDs across separate endpoints; (6) reviewed power analysis
and registry relation; (7) incident-driven owner compromise state; (8) complete
emergency object, review, and later Git publication; (9) all six approval
schemas independently catalogued; and (10) the exact 20-decision contract. If
any control cannot be built without guessing, it is P1. The reviewer then
rechecks all thirteen R6 findings and all preserved historical authority.

## 25. Historical R7 planning-remediation handoff

```text
MHI V1 PHASE F
R7 PLANNING REMEDIATION HANDOFF

STARTING R6 SHA: b70a068afc0c2a2458dfee61281a455fd657400f
R6 PLAN SHA-256: 776880c916366fd7cda6a5075ee8fc7df5ba70d5c5f4a8dd431a27b382862bde
R6 PLAN BLOB: 558c11260f5fcec1e711b90931481cbd012db643
R7 PLAN REVIEW SHA: <filled only by fresh independent R7 reviewer>
R7 PLAN SHA-256: <filled after final plan bytes>
R7 PLAN GIT BLOB: <filled after final plan bytes>
CHANGED FILES: 1 expected
R7 forward remediation: R6 NO-GO / P1=13; R7 independent rereview PENDING

R6 P1 REMEDIATION
F-PLAN-R7-P1-01: REMEDIATED
F-PLAN-R7-P1-02: REMEDIATED
F-PLAN-R7-P1-03: REMEDIATED
F-PLAN-R7-P1-04: REMEDIATED
F-PLAN-R7-P1-05: REMEDIATED
F-PLAN-R7-P1-06: REMEDIATED
F-PLAN-R7-P1-07: REMEDIATED
F-PLAN-R7-P1-08: REMEDIATED
F-PLAN-R7-P1-09: REMEDIATED
F-PLAN-R7-P1-10: REMEDIATED
F-PLAN-R7-P1-11: REMEDIATED
F-PLAN-R7-P1-12: REMEDIATED
F-PLAN-R7-P1-13: REMEDIATED

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
EMERGENCY
self Git identity: REMOVED
external-object review target: COMPLETE
later Git publication: COMPLETE
fixed-point cycles: 0

REVIEW
target tagged union: COMPLETE
review_instance_id removed: yes
aggregate rule: COMPLETE

F0 / POWER
owner decision count: 20
F-OD-12 future object: REMOVED
future-object dependency paths: 0
power-analysis review binding: COMPLETE

READINESS
readiness ID construction: COMPLETE
construction-table contradiction: REMOVED

MONITORING
measurement evidence: COMPLETE
PASS constructible: yes
breach schema: COMPLETE
result derivation: COMPLETE

CLAIM STATE
cause incident: COMPLETE
exact relations: COMPLETE
owner compromise: COMPLETE

RETENTION
scope union: COMPLETE
campaign audit: COMPLETE
release audit: COMPLETE
pre-release path constructible: yes

METROLOGY
endpoint-qualified check: COMPLETE
policy-qualified result: COMPLETE
duplicate check IDs across endpoints: DETERMINISTIC

SCHEMA CATALOG
six approval schemas individually listed: yes
orphan schemas: 0
CATALOG_REQUIREMENTS: 13 canonical R7 requirement rows plus schema mappings
CATALOG_ACS: 13 substantive R7 acceptance criteria
CATALOG_TESTS: 13 executable R7 tests
CATALOG_EVIDENCE: 13 substantive R7 evidence items
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
requirements: 13 substantive rows
ACs: 13 substantive rows
tests: 13 executable rows
evidence: 13 substantive rows
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
all counters in §22 and the R7 audit list below: 0
REGISTRY_BACK_POINTER_PATHS: 0
WIRE_IDENTITY_CYCLES: 0
SELF_GIT_IDENTITY_CYCLES: 0
REGISTRY_COMPROMISE_GIT_FIXED_POINT_CYCLES: 0
EMERGENCY_REVIEW_TARGET_AMBIGUITIES: 0
COMPOSITE_REVIEW_TARGET_PATHS: 0
REVIEW_TARGET_AMBIGUITIES: 0
REVIEW_INSTANCE_ID_AMBIGUITIES: 0
F0_F1_FUTURE_OBJECT_DEPENDENCY_PATHS: 0
F0_FUTURE_OBJECT_REFERENCE_PATHS: 0
READINESS_ID_CONTRADICTIONS: 0
MONITORING_EVIDENCE_AMBIGUITIES: 0
MONITORING_RESULT_DERIVATION_AMBIGUITIES: 0
CLAIM_STATE_CAUSE_BINDING_AMBIGUITIES: 0
CLAIM_STATE_RELATION_AMBIGUITIES: 0
OWNER_KEY_COMPROMISE_AUTHORITY_AMBIGUITIES: 0
RETENTION_REGISTRY_RELATION_AMBIGUITIES: 0
PRE_RELEASE_RETENTION_POSITIVE_PATH_AMBIGUITIES: 0
METROLOGY_CHECK_LOOKUP_AMBIGUITIES: 0
POWER_SCIENTIFIC_REVIEW_BINDING_AMBIGUITIES: 0
ORPHAN_EXTERNAL_SCHEMAS: 0
STALE_F0_NORMATIVE_REFERENCES: 0
NORMATIVE_F0_COUNT_CONTRADICTIONS: 0
UNMAPPED_REQUIREMENTS: 0
UNMAPPED_ACS: 0
UNMAPPED_TESTS: 0
UNMAPPED_EVIDENCE: 0
UNMAPPED_ODS: 0
TRACEABILITY_SUBSTANCE_GAPS: 0
LOST_R1_NORMATIVE_OBLIGATIONS: 0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN: 0
NORMATIVE_CONTRADICTIONS: 0
POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES: 0

BASELINE
diff: PASS
fmt: PASS
check: PASS
strict Clippy: PASS
Clippy diagnostics: 0
Phase E: 38/38
Phase D: 73/73
FROZEN PHASE-E PLAN: PASS
CHANGED FILES: 1 expected
WORKTREE CLEAN: yes after commit

R7 INDEPENDENT REREVIEW: PENDING
FROZEN PHASE-E PLAN: PASS if SHA/blob verify unchanged
PLAN TAG CREATED: NO
IMPLEMENTATION BRANCH: NO
F0 STARTED: NO
TRUST CHANGED: NO
KEYS CREATED: NO
EVIDENCE CREATED: NO
CLAIM CREATED: NO
READY_FOR_FRESH_PHASE_F_R7_PLAN_REREVIEW: yes
READY_FOR_PHASE_F_PLAN_APPROVAL_TAG: NO pending fresh R7 GO
READY_FOR_PHASE_F_IMPLEMENTATION: NO
```

## 26. R8 MASTER SCHEMA CATALOG

This section is the only current schema catalog. The historical R7 catalog in
§16 is not current authority. The catalog is set-equal to every distinct
normative identifier matching `PhaseF[A-Za-z0-9_]*V1` in this complete plan;
there are no wildcard, grouped, or implicit rows. Every row has all thirteen
required metadata columns, including explicit containing-object-only values for
nested schemas.

### 26.1 SCHEMA_IDENTIFIER_SET

The post-edit mechanical enumeration is:

```text
PhaseFArgvV1
PhaseFAuthorityEnrollmentApprovalV1
PhaseFAuthorityEnrollmentV1
PhaseFChainOfCustodyV1
PhaseFCheckListV1
PhaseFCheckerBuildEvidenceV1
PhaseFCheckerExitCodeV1
PhaseFCheckerReadinessEvidenceV1
PhaseFCheckerReportV1
PhaseFCheckerStdoutV1
PhaseFClaimStateRecordV1
PhaseFCohortLockRecordV1
PhaseFCommandV1
PhaseFCustodyEventV1
PhaseFDecisionApprovalV1
PhaseFDecisionBundleV1
PhaseFDecisionRowV1
PhaseFDecisionValueV1
PhaseFDependencyAuditV1
PhaseFDependencyEdgeV1
PhaseFDeviationEventV1
PhaseFDeviationLedgerRevisionV1
PhaseFDeviationLedgerV1
PhaseFEndpointMetrologyPolicyV1
PhaseFEnvironmentEntryV1
PhaseFExecutionRecordV1
PhaseFF5ReleaseCandidateV1
PhaseFIdentityComparisonV1
PhaseFIncidentRecordV1
PhaseFIncidentScopeV1
PhaseFIndependentReviewBundleV1
PhaseFIndependentReviewV1
PhaseFLODLOQPolicyV1
PhaseFLocationLedgerV1
PhaseFLocationV1
PhaseFMethodVersionV1
PhaseFMetricThresholdV1
PhaseFMetrologyCheckResultV1
PhaseFMetrologyCheckSpecV1
PhaseFMetrologyPolicyV1
PhaseFMonitoringBreachV1
PhaseFMonitoringEvidenceV1
PhaseFMonitoringMeasurementV1
PhaseFMonitoringPolicyV1
PhaseFMonitoringRecordV1
PhaseFMonitoringSourceReferenceV1
PhaseFMonitoringValueV1
PhaseFNamedDigestV1
PhaseFObjectDigestV1
PhaseFObjectReferenceV1
PhaseFOutputSpecV1
PhaseFPackageBindingV1
PhaseFPackageManifestV1
PhaseFPackageObjectV1
PhaseFParameterSpecV1
PhaseFParameterValueRowV1
PhaseFPhysicalIdentityAuditV1
PhaseFPhysicalReleaseApprovalV1
PhaseFPhysicalUnitLedgerV1
PhaseFPlanApprovalV1
PhaseFPowerAnalysisRecordV1
PhaseFPowerMethodInterfaceV1
PhaseFPowerOutputValueV1
PhaseFProtocolProjectionV1
PhaseFQuantifiedUncertaintyV1
PhaseFRangeRuleV1
PhaseFReadinessApprovalV1
PhaseFReferenceAssessmentV1
PhaseFReferenceResultV1
PhaseFReferenceSourceDescriptorV1
PhaseFRegistryCompromiseEmergencyV1
PhaseFRegistryHeadV1
PhaseFRegistryRecordV1
PhaseFRegistryRelationV1
PhaseFReinstatementApprovalV1
PhaseFReleaseRecordV1
PhaseFRetentionAuditV1
PhaseFRetentionObjectCheckV1
PhaseFRetentionObjectV1
PhaseFRetentionScopeV1
PhaseFRetrievalVerificationV1
PhaseFReviewTargetV1
PhaseFScientificAdmissibilityAuditV1
PhaseFSensitivityCaseV1
PhaseFSensitivityOverrideV1
PhaseFTrustProvisioningApprovalV1
PhaseFUncertaintyPolicyV1
PhaseFUnitEntryV1
PhaseFUnitRuleV1
```

### 26.2 CATALOG_IDENTIFIER_SET

| exact identifier | category | exact field-closure section | semantic-ID rule | complete-file hash | producer | validator | stage | registry behavior | CURRENT requirement IDs | CURRENT primary AC IDs | CURRENT test IDs | CURRENT F-EV IDs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `PhaseFArgvV1` | NESTED_WIRE | §7 exact argv derivation | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | containing object's stage | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFAuthorityEnrollmentApprovalV1` | TAG_BODY | §6 exact ASCII body | none | exact tag-body bytes | independent five-role reviewers | tag validator | enrollment gate | tag message hash; no object subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFAuthorityEnrollmentV1` | TOP_LEVEL_WIRE | §5.1 exact enrollment object | §3 exact domain and own ID | exact canonical file bytes | governance authority | enrollment strict parser | enrollment | subject `authority_enrollment` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFChainOfCustodyV1` | TOP_LEVEL_WIRE | §11 exact custody ledger | §3 exact domain and own ID | exact canonical file bytes | custody authority | custody strict parser | F2-F4 | subject `chain_of_custody` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCheckListV1` | NESTED_WIRE | §13 exact checklist | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F0-F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCheckerBuildEvidenceV1` | TOP_LEVEL_WIRE | §7 exact build evidence | §3 exact domain and own ID | exact canonical file bytes | checker builder | readiness strict parser | readiness | subject `checker_build_evidence` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCheckerExitCodeV1` | NESTED_WIRE | §7 exact exit-code grammar | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | checker execution | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCheckerReadinessEvidenceV1` | TOP_LEVEL_WIRE | §7 exact readiness evidence | §3 exact domain and own ID | exact canonical file bytes | checker readiness authority | readiness strict parser | readiness | subject `checker_readiness_evidence` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCheckerReportV1` | TOP_LEVEL_WIRE | §7 exact report | none | exact canonical report bytes | checker | report strict parser | all checker routes | none; report is evidence | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCheckerStdoutV1` | NESTED_WIRE | §7 exact stdout literals | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | checker execution | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFClaimStateRecordV1` | TOP_LEVEL_WIRE | §14 exact claim-state fields | §3 exact domain and own ID | exact canonical file bytes | governance authority | claim-state strict parser | F5+ | subject `claim_state` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCohortLockRecordV1` | TOP_LEVEL_WIRE | §14 exact cohort lock | §3 exact domain and own ID | exact canonical file bytes | campaign authority | cohort strict parser | F2 | subject `cohort_lock` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCommandV1` | NESTED_WIRE | §2/§7 exact command union | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | checker execution | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFCustodyEventV1` | NESTED_WIRE | §11 exact custody event | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2-F4 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDecisionApprovalV1` | TAG_BODY | §6 exact ASCII body | none | exact tag-body bytes | independent five-role reviewers | tag validator | F0 gate | tag message hash; no object subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDecisionBundleV1` | TOP_LEVEL_WIRE | §4 exact decision bundle | §3 exact domain and own ID | exact canonical file bytes | F0 authority | decision strict parser | F0 | subject `decision_bundle` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDecisionRowV1` | NESTED_WIRE | §4 exact decision row | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F0 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDecisionValueV1` | NESTED_WIRE | §2/§4 fixed decision variants | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F0 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDependencyAuditV1` | TOP_LEVEL_WIRE | §11 exact dependency audit | §3 exact domain and own ID | exact canonical file bytes | dependency auditor | dependency strict parser | F2 | subject `dependency_audit` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDependencyEdgeV1` | NESTED_WIRE | §11 exact dependency edge | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDeviationEventV1` | NESTED_WIRE | §11 exact deviation event | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1-F4 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDeviationLedgerRevisionV1` | TOP_LEVEL_WIRE | §11 stable-ledger revision | §3 exact domain and own ID | exact canonical file bytes | campaign authority | deviation strict parser | F1-F4 | subject `deviation_ledger` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFDeviationLedgerV1` | PLAN_ONLY_CONTRACT | §11 stable-ledger contract; no top-level wire | none | none | campaign authority | plan-review strict validator | F1-F4 | none; represented by revisions | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFEndpointMetrologyPolicyV1` | NESTED_WIRE | §13 endpoint policy | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F0-F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFEnvironmentEntryV1` | NESTED_WIRE | §7 exact environment entry | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | readiness | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFExecutionRecordV1` | TOP_LEVEL_WIRE | §14 exact execution record | §3 exact domain and own ID | exact canonical file bytes | release authority | execution strict parser | F4 | subject `execution_record` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFF5ReleaseCandidateV1` | TOP_LEVEL_WIRE | §5.1 exact F5 candidate | §3 exact domain and own ID | exact canonical file bytes | F5 release authority | candidate strict parser | F5 | subject `f5_release_candidate` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFIdentityComparisonV1` | NESTED_WIRE | §11 exact identity comparison | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFIncidentRecordV1` | TOP_LEVEL_WIRE | §15 exact incident object | §3 exact domain and own ID | exact canonical file bytes | operations/governance authority | incident strict parser | all | subject `incident_record` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFIncidentScopeV1` | NESTED_WIRE | §15 exact incident scope union | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | all | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFIndependentReviewBundleV1` | TOP_LEVEL_WIRE | §5 exact five-role bundle | §3 exact domain and own ID | exact canonical file bytes | independent reviewers | review strict parser | every gate | subject `independent_review_bundle` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFIndependentReviewV1` | NESTED_WIRE | §5 exact review row | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | every gate | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFLODLOQPolicyV1` | NESTED_WIRE | §13 exact LOD/LOQ union | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F0-F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFLocationLedgerV1` | TOP_LEVEL_WIRE | §11 exact location ledger | §3 exact domain and own ID | exact canonical file bytes | operations authority | location strict parser | F2 | subject `location_ledger` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFLocationV1` | NESTED_WIRE | §11 exact location row | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMethodVersionV1` | NESTED_WIRE | §12 exact method/version pair | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1-F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMetricThresholdV1` | NESTED_WIRE | §14 exact threshold row | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F0/F5+ | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMetrologyCheckResultV1` | TOP_LEVEL_WIRE | §13 exact check result | §3 exact domain and own ID | exact canonical file bytes | laboratory authority | metrology strict parser | F2 | subject `metrology_check_result` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMetrologyCheckSpecV1` | NESTED_WIRE | §13 exact check spec | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMetrologyPolicyV1` | TOP_LEVEL_WIRE | §13 exact metrology policy | §3 exact domain and own ID | exact canonical file bytes | metrology authority | metrology strict parser | F0-F2 | subject `metrology_policy` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMonitoringBreachV1` | NESTED_WIRE | §14 exact breach row | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F5+ | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMonitoringEvidenceV1` | TOP_LEVEL_WIRE | §14 exact evidence object | §3 exact domain and own ID | exact canonical file bytes | metric evidence producer | evidence strict parser | F5+ | subject `monitoring_evidence` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMonitoringMeasurementV1` | NESTED_WIRE | §14 exact measurement row | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F5+ | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMonitoringPolicyV1` | TOP_LEVEL_WIRE | §14 exact monitoring policy | §3 exact domain and own ID | exact canonical file bytes | F0 authority | monitoring strict parser | F0/F5+ | subject `monitoring_policy` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMonitoringRecordV1` | TOP_LEVEL_WIRE | §14 exact monitoring record | §3 exact domain and own ID | exact canonical file bytes | operations authority | monitoring strict parser | F5+ | subject `monitoring_record` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMonitoringSourceReferenceV1` | NESTED_WIRE | §14 exact source reference | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F5+ | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFMonitoringValueV1` | NESTED_WIRE | §2/§14 exact value union | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F5+ | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFNamedDigestV1` | NESTED_WIRE | §2 exact named digest | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | checker execution | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFObjectDigestV1` | NESTED_WIRE | §2 exact object digest | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | all | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFObjectReferenceV1` | NESTED_WIRE | §10 exact object reference | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | all | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFOutputSpecV1` | NESTED_WIRE | §12 exact output specification | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPackageBindingV1` | NESTED_WIRE | §10 exact package binding | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPackageManifestV1` | TOP_LEVEL_WIRE | §10 exact package manifest | §3 exact domain and own ID | exact canonical file bytes | campaign authority | package strict parser | F2 | subject `package_manifest` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPackageObjectV1` | NESTED_WIRE | §10 exact package object | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFParameterSpecV1` | NESTED_WIRE | §12 exact parameter specification | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFParameterValueRowV1` | NESTED_WIRE | §12 exact parameter value row | containing-object-only | containing-object-only | containing object's producer | containing-object's strict parser | F1 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPhysicalIdentityAuditV1` | TOP_LEVEL_WIRE | §11 exact identity audit | §3 exact domain and own ID | exact canonical file bytes | identity auditor | identity strict parser | F2 | subject `identity_audit` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPhysicalReleaseApprovalV1` | TAG_BODY | §6 exact ASCII body | none | exact tag-body bytes | independent five-role reviewers | tag validator | physical-release gate | tag message hash; no object subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPhysicalUnitLedgerV1` | TOP_LEVEL_WIRE | §11 exact unit ledger | §3 exact domain and own ID | exact canonical file bytes | campaign authority | unit strict parser | F2 | subject `physical_unit_ledger` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPlanApprovalV1` | TAG_BODY | §6 exact ASCII body | none | exact tag-body bytes | independent five-role reviewers | tag validator | plan gate | tag message hash; no object subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPowerAnalysisRecordV1` | TOP_LEVEL_WIRE | §12 exact analysis record | §3 exact domain and own ID | exact canonical file bytes | statistician | power strict parser | F1 | subject `power_analysis` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPowerMethodInterfaceV1` | TOP_LEVEL_WIRE | §12 exact method interface | §3 exact domain and own ID | exact canonical file bytes | statistician | power strict parser | F1 | subject `power_method_interface` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFPowerOutputValueV1` | NESTED_WIRE | §12 exact output value | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFProtocolProjectionV1` | PLAN_ONLY_CONTRACT | §4 exact projection | none | none | checker/projection authority | plan-review strict validator | F0-F1 | none; audit projection only | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFQuantifiedUncertaintyV1` | NESTED_WIRE | §2/§13 exact uncertainty | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRangeRuleV1` | NESTED_WIRE | §2/§12 exact range union | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFReadinessApprovalV1` | TAG_BODY | §6 exact ASCII body | none | exact tag-body bytes | independent five-role reviewers | tag validator | readiness gate | tag message hash; no object subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFReferenceAssessmentV1` | NESTED_WIRE | §13 exact assessment row | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested; no independent registry subject | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFReferenceResultV1` | TOP_LEVEL_WIRE | §13 exact reference result | §3 exact domain and own ID | exact canonical file bytes | laboratory authority | reference strict parser | F2 | subject `reference_result` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFReferenceSourceDescriptorV1` | TOP_LEVEL_WIRE | §13 exact source descriptor | §3 exact domain and own ID | exact canonical file bytes | laboratory/data authority | source strict parser | F2 | subject `reference_source_descriptor` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRegistryCompromiseEmergencyV1` | TOP_LEVEL_WIRE | §15 exact emergency object | §3 exact domain and own ID | exact canonical file bytes | security/operations authority | emergency strict parser | claim-status | external emergency input; later Git outer attestation | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRegistryHeadV1` | TOP_LEVEL_WIRE | §8 exact registry head | none | exact canonical head bytes | registry authority | head strict parser | all | resolver object; no subject row | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRegistryRecordV1` | TOP_LEVEL_WIRE | §8 exact signed record | none | exact canonical signed bytes | registry authority | registry strict parser | F1-F5 | chain record; subject and relation fields exact | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRegistryRelationV1` | NESTED_WIRE | §9 exact typed relation | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1-F5 | nested in registry record | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFReinstatementApprovalV1` | TOP_LEVEL_WIRE | §14 exact reinstatement approval | §3 exact domain and own ID | exact canonical file bytes | governance authority | reinstatement strict parser | F5+ | subject `reinstatement_approval` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFReleaseRecordV1` | TOP_LEVEL_WIRE | §14 exact release record | §3 exact domain and own ID | exact canonical file bytes | release authority | release strict parser | F5 | subject `release_record` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRetentionAuditV1` | TOP_LEVEL_WIRE | §15 exact retention audit | §3 exact domain and own ID | exact canonical file bytes | retention authority | retention strict parser | all | subject `retention_audit` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRetentionObjectCheckV1` | NESTED_WIRE | §15 exact retention check | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | all | nested in retention audit | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRetentionObjectV1` | NESTED_WIRE | §15 exact package/authority union | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | all | nested in retention check | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRetentionScopeV1` | NESTED_WIRE | §15 exact release/campaign union | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | all | nested in retention audit | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFRetrievalVerificationV1` | TOP_LEVEL_WIRE | §10 exact retrieval verification | §3 exact domain and own ID | exact canonical file bytes | retrieval authority | retrieval strict parser | all | subject `retrieval_verification` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFReviewTargetV1` | NESTED_WIRE | §2/§5 exact target union | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | every gate | nested in review bundle | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFScientificAdmissibilityAuditV1` | TOP_LEVEL_WIRE | §13 exact admissibility audit | §3 exact domain and own ID | exact canonical file bytes | scientific reviewer | admissibility strict parser | F2 | subject `scientific_admissibility_audit` | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFSensitivityCaseV1` | NESTED_WIRE | §12 exact sensitivity case | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1 | nested in power analysis | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFSensitivityOverrideV1` | NESTED_WIRE | §2/§12 exact sensitivity override | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1 | nested in sensitivity case | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFTrustProvisioningApprovalV1` | TOP_LEVEL_WIRE | §6 exact trust approval body/object | none | exact tag-body bytes | independent five-role reviewers | tag validator | trust gate | tag message hash; trust object is separately authoritative | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFUncertaintyPolicyV1` | NESTED_WIRE | §2/§13 exact uncertainty policy | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F0-F2 | nested in metrology policy | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFUnitEntryV1` | NESTED_WIRE | §11 exact unit entry | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F2 | nested in unit ledger | R8-22 | AC8-22 | T8-16 | EV8-22 |
| `PhaseFUnitRuleV1` | NESTED_WIRE | §2/§12 exact unit rule | containing-object-only | containing-object-only | containing object's producer | containing object's strict parser | F1-F2 | nested in parameter/output specification | R8-22 | AC8-22 | T8-16 | EV8-22 |

The six approval schemas have six individual rows above: `PhaseFPlanApprovalV1`,
`PhaseFDecisionApprovalV1`, `PhaseFReadinessApprovalV1`,
`PhaseFAuthorityEnrollmentApprovalV1`, `PhaseFTrustProvisioningApprovalV1`,
and `PhaseFPhysicalReleaseApprovalV1`. `PhaseFCheckListV1`,
`PhaseFCheckerExitCodeV1`, `PhaseFCheckerStdoutV1`,
`PhaseFDecisionValueV1`, `PhaseFEnvironmentEntryV1`, `PhaseFLODLOQPolicyV1`,
`PhaseFMonitoringValueV1`, `PhaseFNamedDigestV1`, `PhaseFObjectDigestV1`,
`PhaseFQuantifiedUncertaintyV1`, `PhaseFRangeRuleV1`,
`PhaseFUncertaintyPolicyV1`, and `PhaseFUnitRuleV1` are individually listed,
not represented by a grouped helper row.

## 27. R8 CURRENT NORMATIVE REQUIREMENT MATRIX

This is the only current requirement-to-owner-decision mapping source. The R6
and R7 tables in §19 are historical only. The `owner_decision_ids` column is
the only normative F-OD-to-requirement edge list; any OD coverage summary is
derived and non-normative.

| requirement_id | normative_statement | owner_decision_ids | schema_ids | stage | review_roles | primary_ac_id | test_ids | evidence_ids |
|---|---|---|---|---|---|---|---|---|
| `R8-01` | Emergency publication uses exactly one full-prefixed repository path pair; claim-status validates the ten-step sequence, exact tree paths, reachability, and bytes with no fallback. | F-OD-14,F-OD-17 | PhaseFIncidentRecordV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFIndependentReviewBundleV1,PhaseFReviewTargetV1,PhaseFObjectReferenceV1,PhaseFCommandV1,PhaseFArgvV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFCheckerExitCodeV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1 | emergency/claim-status | security,architecture_data | AC8-01 | T8-01,T8-02 | EV8-01 |
| `R8-02` | Monitoring has exactly 15 required metrics partitioned 4 status/1 quantity/4 rate/6 binding, five thresholds, fixed metric order, fixed threshold order, and exact result derivation. | F-OD-19 | PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringBreachV1,PhaseFMetricThresholdV1,PhaseFMonitoringValueV1 | F0/F5+ | operations_governance,architecture_data | AC8-02 | T8-03,T8-04,T8-05,T8-06 | EV8-02 |
| `R8-03` | Every monitoring measurement is backed by one complete typed evidence object with exact source-kind mapping, ID/hash binding, window equality, and value equality. | F-OD-19 | PhaseFMonitoringEvidenceV1,PhaseFMonitoringSourceReferenceV1,PhaseFObjectReferenceV1,PhaseFMonitoringValueV1 | F5+ | operations_governance,scientific_metrology | AC8-03 | T8-07,T8-08,T8-09 | EV8-03 |
| `R8-04` | Monitoring registry attestation contains exactly the release, policy, and 15 evidence dependencies, and binding values equal named authoritative fields. | F-OD-19 | PhaseFMonitoringRecordV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFExecutionRecordV1,PhaseFTrustProvisioningApprovalV1,PhaseFReleaseRecordV1 | F5+ | operations_governance,security | AC8-04 | T8-10 | EV8-04 |
| `R8-05` | Campaign-abandonment incident files contain no review hash/reference; the completed incident is independently reviewed afterward and the incident registry record binds that review to the exact incident SHA. | F-OD-13,F-OD-15 | PhaseFIncidentRecordV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFReviewTargetV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFObjectReferenceV1 | all pre-release campaign operations | security,operations_governance | AC8-05 | T8-11,T8-12 | EV8-05 |
| `R8-06` | Retention checks use the exact package/authority tagged object union and canonical identity key, never an untyped digest. | F-OD-20 | PhaseFRetentionObjectV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionScopeV1,PhaseFRetentionAuditV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1 | all | security | AC8-06 | T8-13 | EV8-06 |
| `R8-07` | Campaign and release retention object sets are deterministic exact unions; campaign coverage includes the manifest and every manifest object exactly once, and release coverage includes only verified authority/context bindings. | F-OD-20 | PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFRetentionAuditV1,PhaseFRetentionObjectV1,PhaseFRetentionObjectCheckV1 | all | security,operations_governance | AC8-07 | T8-14,T8-15 | EV8-07 |
| `R8-08` | The complete external-authority DAG remains one-way from semantic object to complete file to hash to later registry attestation, preserving all previously closed release/state/deviation and Phase-E contracts. | none | PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReleaseRecordV1,PhaseFClaimStateRecordV1,PhaseFMonitoringRecordV1,PhaseFDeviationLedgerRevisionV1 | F1-F5 | architecture_data,security | AC8-08 | T8-17 | EV8-08 |
| `R8-09` | F0 remains exactly 20 decision rows, F-OD-12 remains method ID/version only, and F0-to-F1 projection and power chronology remain exact. | F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-08,F-OD-09,F-OD-12 | PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFProtocolProjectionV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerAnalysisRecordV1 | F0-F1 | architecture_data,scientific_metrology | AC8-09 | T8-18 | EV8-09 |
| `R8-10` | Review target, five-role review rows, aggregate decision/counts, immutable artifact references, and six approval tag bodies remain closed and independently bound. | none | PhaseFReviewTargetV1,PhaseFIndependentReviewV1,PhaseFIndependentReviewBundleV1,PhaseFPlanApprovalV1,PhaseFDecisionApprovalV1,PhaseFReadinessApprovalV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFTrustProvisioningApprovalV1,PhaseFPhysicalReleaseApprovalV1 | all gates | architecture_data | AC8-10 | T8-19 | EV8-10 |
| `R8-11` | Checker build evidence, readiness ID construction, command object, argv, stdout, exit code, and fresh-environment closure remain exact and executable. | none | PhaseFCheckerBuildEvidenceV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCommandV1,PhaseFArgvV1,PhaseFEnvironmentEntryV1,PhaseFNamedDigestV1,PhaseFCheckerStdoutV1,PhaseFCheckerExitCodeV1 | readiness | compatibility | AC8-11 | T8-20 | EV8-11 |
| `R8-12` | Registry subjects, complete-file hash meanings, typed relations, relation tuple order, head currentness, and external registry attestation remain exact. | F-OD-14,F-OD-17 | PhaseFRegistryRelationV1,PhaseFRegistryRecordV1,PhaseFRegistryHeadV1,PhaseFObjectReferenceV1 | F1-F5 | security,architecture_data | AC8-12 | T8-17 | EV8-12 |
| `R8-13` | Retrieval, manifest, package classification, dependency and package relation contracts remain complete and deterministic. | none | PhaseFRetrievalVerificationV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFPackageBindingV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1 | F2 | security,scientific_metrology | AC8-13 | T8-17 | EV8-13 |
| `R8-14` | Physical native identity, pseudoreplication, location, custody continuity, and destroyed-unit controls remain exact. | F-OD-10 | PhaseFPhysicalUnitLedgerV1,PhaseFUnitEntryV1,PhaseFPhysicalIdentityAuditV1,PhaseFIdentityComparisonV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFChainOfCustodyV1,PhaseFCustodyEventV1 | F2-F4 | scientific_metrology,operations_governance | AC8-14 | T8-17 | EV8-14 |
| `R8-15` | Deviation ledger revisions are stable-ID, immutable-history, action-compatible, and acyclic. | F-OD-18 | PhaseFDeviationLedgerV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationEventV1 | F1-F4 | operations_governance,security | AC8-15 | T8-17 | EV8-15 |
| `R8-16` | Power interface, method/version, typed parameter/output values, ranges, units, sensitivity cases, and scientific review remain complete before analysis registration. | F-OD-12 | PhaseFPowerMethodInterfaceV1,PhaseFMethodVersionV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFOutputSpecV1,PhaseFPowerOutputValueV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFRangeRuleV1,PhaseFUnitRuleV1,PhaseFPowerAnalysisRecordV1 | F1 | scientific_metrology | AC8-16 | T8-17 | EV8-16 |
| `R8-17` | Metrology policy is endpoint-qualified and check-result math, LOD/LOQ, uncertainty, and units are exact without checker conversion. | F-OD-11 | PhaseFMetrologyPolicyV1,PhaseFEndpointMetrologyPolicyV1,PhaseFCheckListV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyCheckResultV1,PhaseFLODLOQPolicyV1,PhaseFUncertaintyPolicyV1 | F0-F2 | scientific_metrology | AC8-17 | T8-17 | EV8-17 |
| `R8-18` | Reference source/result/admissibility objects retain external provenance, independence controls, exact uncertainty, and total runtime projection without measurement scalars. | F-OD-05,F-OD-06,F-OD-07 | PhaseFReferenceSourceDescriptorV1,PhaseFReferenceResultV1,PhaseFReferenceAssessmentV1,PhaseFQuantifiedUncertaintyV1 | F2 | scientific_metrology | AC8-18 | T8-17 | EV8-18 |
| `R8-19` | Claim-state causes, exact relations, owner-key compromise, reinstatement, trigger resolution modes, and F4/F5 boundary remain closed. | F-OD-16 | PhaseFClaimStateRecordV1,PhaseFReinstatementApprovalV1,PhaseFIncidentScopeV1,PhaseFIncidentRecordV1,PhaseFRegistryRelationV1 | F5+ | operations_governance,security | AC8-19 | T8-17 | EV8-19 |
| `R8-20` | F5 release candidate, physical release approval, release/state chronology, P2 hard gate, production runner order, and final release authority remain exact. | none | PhaseFF5ReleaseCandidateV1,PhaseFPhysicalReleaseApprovalV1,PhaseFReleaseRecordV1,PhaseFClaimStateRecordV1,PhaseFIndependentReviewBundleV1 | F4-F5 | architecture_data,operations_governance | AC8-20 | T8-17,T8-19 | EV8-20 |
| `R8-21` | All closed R7/R6 safety, scientific, identity, custody, trust, DAG, runtime compatibility, and production-order contracts remain preserved unless directly changed by R8 P1 remediation. | none | PhaseFDecisionBundleV1,PhaseFPowerMethodInterfaceV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFRetentionAuditV1,PhaseFClaimStateRecordV1 | all | architecture_data,scientific_metrology,security | AC8-21 | T8-17 | EV8-21 |
| `R8-22` | The master schema catalog contains one fully populated row for every identifier in `SCHEMA_IDENTIFIER_SET`, and every catalog schema is bidirectionally present in the current R8 matrix. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data | AC8-22 | T8-14,T8-16 | EV8-22 |
| `R8-23` | The single R8 requirement graph is the only current OD mapping and its derived owner-decision union is exactly F-OD-01 through F-OD-20 with no extra owner decision. | none | PhaseFDecisionBundleV1,PhaseFDecisionRowV1 | plan review | architecture_data | AC8-23 | T8-19 | EV8-23 |
| `R8-24` | Every current R8 requirement has one substantive AC, exact test and evidence references, and no identifier-only traceability row remains. | none | PhaseFIndependentReviewBundleV1,PhaseFMonitoringEvidenceV1,PhaseFRetentionAuditV1,PhaseFPackageManifestV1 | plan review | architecture_data | AC8-24 | T8-19 | EV8-24 |

`owner_decision_ids` is not repeated in a second current table. The derived
coverage computation is `union(R8 requirement.owner_decision_ids)` and must
equal the exact set `{F-OD-01,...,F-OD-20}`.

## 28. R8 AC, TEST, AND F-EV EXECUTABILITY

Every current R8 acceptance criterion has preconditions, exact inputs, exact
operation, expected result/output, and a failure oracle.

| AC ID | preconditions | exact inputs | exact operation | expected result | expected output | failure oracle |
|---|---|---|---|---|---|---|
| `AC8-01` | incident, emergency, review, local files, supplied commit, and live remote main are available | complete emergency/review bytes, paths, commit SHA, remote main | validate schema/IDs, review target, aggregate, commit existence/reachability, exact full-prefixed tree paths, and tree-byte equality | exact ten-step procedure returns NOT_ACTIVE | claim-status report with matched paths and bytes | any prefix omission, fallback lookup, ancestry failure, or byte mismatch rejects |
| `AC8-02` | monitoring policy and fixed metric order are present | policy, 15-member required array, 5 thresholds, 15 measurements | count/partition/order and recompute breaches/result | cardinality/order valid; healthy fixture yields pass | canonical policy/record | any count, category, threshold, or sort contradiction rejects |
| `AC8-03` | 15 evidence files are complete and addressable | evidence files, source references, measurements, monitoring window | parse/hash each evidence file and compare ID/hash/metric/value/window/source mapping | every measurement has exactly one matching evidence object | evidence-backed measurement set | absent/mismatched evidence, source kind, window, or value rejects |
| `AC8-04` | monitoring record and authority objects are valid | record, release/policy, authority objects, registry relations | enforce exact relation set and binding-field equality | 15 evidence dependencies and exact bindings validate | signed monitoring registry record | missing/extra/duplicate relation or field mismatch rejects |
| `AC8-05` | campaign abandonment incident is complete before review | incident file, incident SHA, review bundle, registry record | reject review fields in incident; then bind review target and registry relation | acyclic authority path validates | incident, review, and registry artifacts | any reverse review pointer or wrong target SHA rejects |
| `AC8-06` | retention policy and typed object references are valid | package/authority object union and check rows | parse tagged union and compute identity key/order | every check has typed object and canonical key | canonical retention checks | untyped digest, wrong key, duplicate, or noncanonical order rejects |
| `AC8-07` | complete package manifest exists | manifest M and objects O1..On; campaign or release context | derive exact set and compare audit rows by set equality | campaign/release coverage passes only on exact set | retention audit with no extras or omissions | missing, duplicate, extra, current-audit, or human-selected object rejects |
| `AC8-08` | historical safety and DAG fixtures are available | complete subject files and later registry records | replay one-way construction and preserved closed contracts | no cycle or reopened closed contract | preservation audit | any back-pointer, cycle, changed closed contract, or Phase-E mismatch rejects |
| `AC8-09` | exact F0 decision fixture and F1 interface exist | 20 decision rows, F-OD-12 row, projection, interface, analysis | validate chronology and exact value equality | F0/F1 positive path passes | projection and power authority objects | future object in F0, stale 21st row, or chronology reversal rejects |
| `AC8-10` | five unique role rows and approval body fixtures exist | review bundle, target, six tag-body fixtures | validate target union, aggregate rule, role uniqueness, tag minimal grammar | every gate review/tag validates | review bundle and tag-body bytes | row ID, target ambiguity, count mismatch, or per-role tag field rejects |
| `AC8-11` | two fresh builds and checker invocation fixtures exist | build/readiness/report/command/argv/environment objects | recompute readiness ID and exact argv/output | readiness PASS with exit/output agreement | readiness and checker report | omitted transcript, ID construction mismatch, or argv mismatch rejects |
| `AC8-12` | registry chain/head fixture exists | typed registry records, head, relation tuples | validate subject/file hash meaning, signature, sequence, tuple order/currentness | chain and head validate | registry chain report | wrong subject hash, duplicate/gap/fork, or tuple order rejects |
| `AC8-13` | package/retrieval fixtures are complete | immutable references, retrieval record, manifest, dependency graph | verify URI/hash/length and package relations | package authority validates | retrieval and package records | unavailable object, wrong bytes, missing relation, or classification gap rejects |
| `AC8-14` | physical unit/location/custody fixtures are complete | ledgers, identity comparisons, custody events | recompute native identity and continuity | no alias/pseudoreplication/destroyed-unit path | physical audit set | shared native identity, custody discontinuity, or post-destroy measure rejects |
| `AC8-15` | deviation ledger and revisions are complete | stable ledger ID, prior revision, new event, action row | verify immutable prior bytes and action compatibility | revision is one-way and valid | deviation revision | prior mutation, incompatible action, or derived stable ID rejects |
| `AC8-16` | approved method interface exists | interface, typed parameter/output specs, sensitivity cases, analysis | validate all ranges/units and review before registration | complete power path passes | interface, analysis, review relation | missing range/unit, undeclared parameter, or early registration rejects |
| `AC8-17` | endpoint policies and check results exist | policy, duplicate check IDs on endpoints, LOD/LOQ/uncertainty | resolve by endpoint/policy/check and recompute exact math/units | required checks pass | metrology result files | endpoint collision, unit conversion, or schema omission rejects |
| `AC8-18` | admissible reference files exist | source descriptor, result, assessment, runtime projection | verify provenance/independence and project only allowed endpoint fields | exact runtime projection passes | reference and audit objects | measurement scalar in runtime projection or unknown independence rejects |
| `AC8-19` | state/incident/reinstatement fixtures exist | state chain, cause incident, exact relations, trigger | recompute transition and relation set | permitted state is accepted | state and registry records | missing cause/relation, owner-signature bypass misuse, or bad mode rejects |
| `AC8-20` | F5 candidate and release inputs are complete | candidate, review, release, state, tags, P2 result | enforce F4/F5 order and P2 hard gate | release path is valid only after all prerequisites | release authority set | early tag, P2 bypass, or final-state chronology failure rejects |
| `AC8-21` | frozen Phase-E and closed R7/R6 fixture manifests exist | authority hashes, source tree, compatibility fixtures | compare preserved contracts and frozen hashes | all preservation checks pass | preservation report | any frozen hash change or reopened closed contract rejects |
| `AC8-22` | complete plan bytes and current catalog are available | regex identifier set and catalog rows | compute set equality, row uniqueness, and all metadata cells | 89 identifiers; missing/extra/duplicate/incomplete all zero | catalog audit | absent identifier, duplicate row, wildcard/grouped row, or blank cell rejects |
| `AC8-23` | current R8 matrix is parsed | all `owner_decision_ids` cells and matrix rows | derive union and scan for a second current mapping source | exact F-OD-01..20 union | derived OD coverage summary | missing/extra OD or contradictory current table rejects |
| `AC8-24` | current R8 graph and artifact tables are parsed | requirements, ACs, tests, evidence, schemas | resolve every reference and inspect substantive fields | all traceability counters are zero | traceability audit | identifier-only artifact row or unresolved reference rejects |

Every current R8 test has fixture construction, exact invocation, expected result,
and a negative mutation.

| test ID | fixture construction | exact checker/function invocation | expected exact result | negative mutation |
|---|---|---|---|---|
| `T8-01` | emergency ID `sha256:<DIGEST>` and both complete local files | claim-status emergency verifier with unprefixed emergency path | FAIL; wrong canonical path | replace full path with `emergencies/<DIGEST>/emergency.json` |
| `T8-02` | same emergency/review bytes committed under full-prefixed paths | claim-status emergency verifier with exact paths and reachable commit | PASS path verification; fail-closed NOT_ACTIVE outcome | alter one tree byte or path component -> FAIL |
| `T8-03` | policy with 4/1/4/6 partition and five thresholds | monitoring policy cardinality/order audit | PASS; 15 total | write five binding or five rate -> plan consistency FAIL |
| `T8-04` | fixed-order 15 measurements and complete evidence files | monitoring record validator | PASS | alphabetically sort measurements -> FAIL |
| `T8-05` | 14 measurements or one absent evidence file | monitoring PASS constructor | FAIL; pass impossible | declare `result=pass` -> reject |
| `T8-06` | 15 complete evidence objects and matching measurements | monitoring record recomputation and registry-attestation validator | PASS with `breaches=[]` | alter one evidence value/hash -> FAIL |
| `T8-07` | evidence object with metric/window/value/source fields | evidence strict parser and measurement comparator | PASS when all equal | change evidence metric or window -> FAIL |
| `T8-08` | one source reference per metric according to mapping | source-kind mapping validator | PASS | use `owner_approval` for software metric -> FAIL |
| `T8-09` | valid evidence and measurement with wrong complete-file SHA | evidence hash verifier | FAIL | replace hash with actual complete-file SHA -> PASS |
| `T8-10` | monitoring record plus exactly 15 evidence relations | `monitoring_recorded` relation-set validator | PASS | omit or add one evidence relation -> FAIL |
| `T8-11` | abandonment incident with review hash/reference field | incident strict parser and authority validator | FAIL schema/authority | remove field and construct incident first -> eligible for review |
| `T8-12` | completed incident SHA followed by review targeting that SHA | incident/review/registry DAG validator | PASS | target another incident SHA -> FAIL |
| `T8-13` | typed package and authority retention objects | retention object/check parser | PASS | supply bare object digest -> FAIL |
| `T8-14` | M,O1,O2,O3 exact manifest/object set | campaign retention-set derivation and audit | PASS | omit O3 -> FAIL |
| `T8-15` | M,O1,O2,O3,O4 with O4 absent from manifest | campaign retention-set equality audit | FAIL | remove O4 -> PASS |
| `T8-16` | plan identifier extraction and 89 populated catalog rows | catalog set/metadata audit | PASS; missing/extra/duplicate/incomplete zero | delete `PhaseFRangeRuleV1` row or blank validator -> FAIL |
| `T8-17` | preserved R6/R7 closure fixtures and Phase-E hashes | preservation/DAG replay audit | PASS | introduce a registry back-pointer or reopen a closed contract -> FAIL |
| `T8-18` | exact 20 F0 rows with F-OD-12 method ID/version only | F0/F1 chronology and projection validator | PASS | add future interface hash or a twenty-first owner decision -> FAIL |
| `T8-19` | current R8 matrix and historical tables labeled non-current | current-source and OD-union audit | PASS; exact F-OD-01..20 union | add a second current mapping for F-OD-12 -> FAIL |
| `T8-20` | two fresh checker build transcripts and command/report pair | readiness/checker invocation validator | PASS | reorder argv or omit transcript -> FAIL |

Every current R8 F-EV row names a real artifact, producer/authority, immutable
identity, and acceptance/review oracle.

| F-EV ID | real artifact | producer/authority | immutable identity | acceptance/review oracle |
|---|---|---|---|---|
| `EV8-01` | emergency/review local files and later Git tree | security authority and independent reviewers | complete file SHA, review SHA, commit/tree SHA | exact ten-step path/ancestry/byte verifier |
| `EV8-02` | monitoring policy, thresholds, measurements, and result | F0 owner and operations authority | policy/record complete-file hashes | 15-cardinality, partition, order, and recomputation audit |
| `EV8-03` | 15 complete monitoring evidence files | metric producers and operations authority | evidence IDs and complete-file SHAs | exact source/metric/window/value comparison |
| `EV8-04` | monitoring registry record and 15 evidence relations | registry authority | signed record and relation hashes | exact relation set and authority-field equality |
| `EV8-05` | completed campaign-abandonment incident, review bundle, registry record | campaign operator, independent reviewers, registry authority | incident/review/registry complete-file SHAs | review target equals incident SHA and no reverse pointer |
| `EV8-06` | typed retention object/check records | retention authority | canonical identity key and complete audit SHA | tagged-union and key-order parser |
| `EV8-07` | package manifest, package objects, campaign/release audit | campaign/release retention authority | manifest/object references and audit SHA | exact set equality |
| `EV8-08` | preserved subject files, registry chains, Phase-E authority files | architecture/security authority | frozen SHA/blob values and replay transcript | no cycle and no change to closed contracts |
| `EV8-09` | F0 decision bundle, projection, power interface/analysis | F0 owner and statistician | decision/interface/analysis complete hashes | exact chronology and F-OD-12 equality |
| `EV8-10` | five-role reviews and six approval tag messages | independent reviewers/Git | review bundle and tag-body bytes | aggregate rule, role uniqueness, and minimal tag grammar |
| `EV8-11` | two build directories, transcripts, binaries, readiness/report files | checker builder and readiness reviewers | source/tree/lock/transcript/binary hashes | fresh-build equivalence and exact command/report output |
| `EV8-12` | signed registry records and resolver head | registry authority | sequence, predecessor, signature, subject hashes | chain, head currentness, tuple ordering |
| `EV8-13` | retrieved external objects, manifest, dependency audit | retrieval/package authorities | immutable URI, byte length, SHA | retrieval and classification oracle |
| `EV8-14` | physical unit/location/custody ledgers and audits | campaign/laboratory/custody authorities | native identity, child, location, and custody hashes | no alias, discontinuity, or destroyed-unit use |
| `EV8-15` | deviation ledger revisions and event records | campaign/deviation authority | stable ledger ID plus revision complete SHA | immutable prior history and action compatibility |
| `EV8-16` | power interface, typed specs, cases, analysis, review | statistician and scientific reviewers | content IDs and review target SHA | range/unit/case completeness and pre-registration review |
| `EV8-17` | metrology policies, endpoint checks, LOD/LOQ and uncertainty files | metrology laboratory | policy/check result IDs and complete hashes | exact endpoint lookup, math, and units |
| `EV8-18` | source descriptors, reference results, assessments, runtime projections | laboratory/runtime authority | source/result/audit complete hashes | external provenance and exact runtime projection |
| `EV8-19` | incidents, claim states, reinstatement approvals, registry relations | governance/operations authority | state/incident/approval complete hashes | exact transition cause and relation contract |
| `EV8-20` | F5 candidate, release, initial state, approvals, tags | release authority and independent reviewers | candidate/release/state/tag bytes | F4/F5 order, P2 gate, and final binding |
| `EV8-21` | frozen Phase-E authority and closed R6/R7 contract fixtures | architecture/security authority | specified Phase-E SHA/blob and replay transcript | preservation comparison |
| `EV8-22` | complete plan text and current schema catalog | plan author and independent reviewer | plan SHA/blob and catalog row text | set equality and all metadata columns |
| `EV8-23` | current R8 requirement matrix | plan author and independent reviewer | plan SHA/blob and matrix row bytes | derived OD union exactly 01..20 |
| `EV8-24` | current requirements, AC/test/evidence tables | plan author and independent reviewer | plan SHA/blob and row identities | bidirectional reference and substance audit |

## 29. R8 POSITIVE CONTROLS AND COUNTEREXAMPLES

### 29.1 Emergency positive control

For `emergency_id=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`,
the only accepted repository paths are:

```text
phase_f_governance/emergencies/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/emergency.json
phase_f_governance/emergencies/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/review.json
```

The lookup of `emergencies/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/emergency.json`
fails. The claim-status checker performs, in order: validate emergency file;
validate five-role review; compare review target to the complete emergency
SHA; verify GO/P0=0/P1=0; verify commit exists; verify reachability from live
remote `main`; resolve the exact commit tree; fetch exactly the full-prefixed
emergency path; fetch exactly the full-prefixed review path; and compare both
tree byte sequences to the supplied files. There is no fallback, prefix
stripping, or basename lookup. `EMERGENCY_REPOSITORY_PATH_AMBIGUITIES=0`,
`EMERGENCY_GIT_PUBLICATION_AMBIGUITIES=0`, and
`CHECKER_EMERGENCY_COMMAND_AMBIGUITIES=0`.

### 29.2 Full monitoring PASS positive control

Use a release-bound window with concrete fixed fixture bytes:
`window_start=2026-01-01T00:00:00Z`, `window_end=2026-01-01T00:30:00Z`,
`produced_at=2026-01-01T00:30:00Z`, and a due boundary after the window.
For each row, `E01` through `E15` means the exact content-derived
`monitoring_evidence_id` and `H01` through `H15` means the exact complete-file
SHA computed from the fully specified evidence object; these are computed
symbols, not omitted fields or guessed values.

| order | metric | value | source kind | evidence ID/hash |
|---:|---|---|---|---|
| 1 | `domain_compliance` | `{type:"status",value:"compliant"}` | `domain_observation` | E01/H01 |
| 2 | `reference_qc_status` | `{type:"status",value:"pass"}` | `reference_qc_record` | E02/H02 |
| 3 | `calibration_status` | `{type:"status",value:"pass"}` | `calibration_record` | E03/H03 |
| 4 | `reference_uncertainty_status` | `{type:"status",value:"within_limit"}` | `reference_uncertainty_record` | E04/H04 |
| 5 | `sensor_drift` | `{type:"quantity",value:{decimal:"0",binary64_bits_hex:"0000000000000000"},unit:"fixture_unit"}` | `sensor_drift_series` | E05/H05 |
| 6 | `invalid_input_rate` | `{type:"rate",value:{decimal:"0",binary64_bits_hex:"0000000000000000"}}` | `input_validation_summary` | E06/H06 |
| 7 | `indeterminate_rate` | `{type:"rate",value:{decimal:"0",binary64_bits_hex:"0000000000000000"}}` | `runtime_validation_summary` | E07/H07 |
| 8 | `data_quality_insufficient_rate` | `{type:"rate",value:{decimal:"0",binary64_bits_hex:"0000000000000000"}}` | `runtime_validation_summary` | E08/H08 |
| 9 | `exclusion_rate` | `{type:"rate",value:{decimal:"0",binary64_bits_hex:"0000000000000000"}}` | `runtime_validation_summary` | E09/H09 |
| 10 | `software_git_sha` | `{type:"git_sha",value:<exact PhaseFExecutionRecordV1.release_code_sha>}` | `execution_record` | E10/H10 |
| 11 | `checker_binary_sha256` | `{type:"sha256",value:<exact PhaseFExecutionRecordV1.checker_binary_sha256>}` | `execution_record` | E11/H11 |
| 12 | `trust_store_sha256` | `{type:"sha256",value:<exact PhaseFTrustProvisioningApprovalV1.trust_store_sha256>}` | `trust_provisioning_approval` | E12/H12 |
| 13 | `trust_root_id` | `{type:"stable_id",value:<exact PhaseFTrustProvisioningApprovalV1.trust_root_id>}` | `trust_provisioning_approval` | E13/H13 |
| 14 | `owner_approval_id` | `{type:"external_digest_id",value:<exact OwnerApprovalEvidenceV1.approval_record_id>}` | `owner_approval` | E14/H14 |
| 15 | `release_record_id` | `{type:"external_digest_id",value:<exact PhaseFReleaseRecordV1.release_record_id>}` | `release_record` | E15/H15 |

The policy `required_metrics`, `measurements`, and evidence relations all use
the exact 15-row order above. Thresholds use fixed order
`[sensor_drift,invalid_input_rate,indeterminate_rate,data_quality_insufficient_rate,exclusion_rate]`,
each with comparator `less_than_or_equal`, exact threshold value `1`, and
`fixture_unit` only for drift. The four statuses are healthy, drift is zero,
all four rates are zero, all six binding values equal their named authority
fields, every `E##/H##` resolves, `breaches=[]`, and `result=pass`.
The monitoring registry record has exactly 15 `depends_on+monitoring_evidence`
relations, one for each `H##`, plus its release and policy relations. Thus
`MONITORING_PASS_CONSTRUCTIBLE=yes` and
`MONITORING_REGISTRY_EVIDENCE_AMBIGUITIES=0`.

### 29.3 Campaign retention and abandonment positive control

For package manifest M with exactly objects O1, O2, and O3,
`CampaignRetentionSetV1=[M,O1,O2,O3]`. An audit with M,O1,O2,O3 passes; an
audit with M,O1,O2 fails; and an audit with M,O1,O2,O3,O4 fails. Construct
abandonment incident I completely, compute SHA(I), then construct the
five-role independent review whose target is exactly
`{type:"external_object",object_kind:"incident_record",object_sha256:SHA(I)}`.
The later `incident_recorded` relation contains exactly
`authorized_by+decision_bundle`, `authorized_by+independent_review_bundle`,
and `references+package_manifest` for campaign abandonment. The retention
deadline is `I.detected_at + retention_seconds`; no review hash occurs inside I,
no release ID is required, and no cycle exists.

### 29.4 R8 counterexamples

| case ID | exact mutation/input | deterministic result |
|---|---|---|
| `R8-CX-01` | checker searches `emergencies/<DIGEST>/emergency.json` | FAIL; wrong canonical path |
| `R8-CX-02` | checker searches full `phase_f_governance/emergencies/<DIGEST>/emergency.json` and bytes match | PASS path verification |
| `R8-CX-03` | metric count is 15 but text says five binding metrics | plan consistency failure |
| `R8-CX-04` | measurements are alphabetically sorted instead of fixed metric order | reject |
| `R8-CX-05` | measurement evidence ID/hash resolves to evidence with another metric | reject |
| `R8-CX-06` | evidence window differs from monitoring record window | reject |
| `R8-CX-07` | `software_git_sha` source kind is not `execution_record` | reject |
| `R8-CX-08` | `software_git_sha` parses valid execution record and matches release code SHA | PASS |
| `R8-CX-09` | campaign-abandonment incident contains a review hash | schema/authority reject |
| `R8-CX-10` | complete incident is followed by review targeting exact incident SHA | PASS |
| `R8-CX-11` | campaign audit omits a manifest object | FAIL |
| `R8-CX-12` | campaign audit includes manifest and every object exactly once | PASS |
| `R8-CX-13` | release audit includes human-selected extra object | FAIL |
| `R8-CX-14` | normative `PhaseFRangeRuleV1` has no catalog row | catalog failure |
| `R8-CX-15` | catalog duplicates `PhaseFRangeRuleV1` | catalog failure |
| `R8-CX-16` | catalog validator metadata is blank | catalog failure |
| `R8-CX-17` | R8 requirement references schema absent from catalog | traceability failure |
| `R8-CX-18` | catalog schema maps to no R8 requirement | traceability failure |
| `R8-CX-19` | F-OD-12 has different mappings in two current tables | impossible; current-source failure |
| `R8-CX-20` | derived owner-decision union equals exactly F-OD-01..20 | PASS |

## 30. R8 REMEDIATION LEDGER

| R8 remediation ID | R7 P1 finding | R8 exact section | root cause | R8 remediation | current R8 requirement IDs | AC IDs | test IDs | F-EV IDs | AUTHOR DISPOSITION |
|---|---|---|---|---|---|---|---|---|---|
| `F-PLAN-R8-P1-01` | emergency publication canonical-path contradiction | §15, §26, §27, §28, §29 | checker prose and positive path omitted `phase_f_governance/` in one lookup | one full-prefixed path pair, exact ten-step verifier, and positive/counterexample KATs | R8-01 | AC8-01 | T8-01,T8-02 | EV8-01 | REMEDIATED |
| `F-PLAN-R8-P1-02` | monitoring evidence authority, metric cardinality, and measurement ordering | §14, §26, §27, §28, §29 | bare evidence hash, conflicting category counts, and generic sorting rule | complete evidence schema/source map, 4/1/4/6 partition, fixed order, fixed thresholds, exact registry relations | R8-02,R8-03,R8-04 | AC8-02,AC8-03,AC8-04 | T8-03,T8-04,T8-05,T8-06,T8-07,T8-08,T8-09,T8-10 | EV8-02,EV8-03,EV8-04 | REMEDIATED |
| `F-PLAN-R8-P1-03` | campaign-abandonment review cycle and retention-object coverage derivation | §15, §26, §27, §28, §29 | incident tried to contain a future review and retention used an untyped digest/set | incident-first review DAG, typed retention object key, manifest-derived campaign set, release union | R8-05,R8-06,R8-07 | AC8-05,AC8-06,AC8-07 | T8-11,T8-12,T8-13,T8-14,T8-15 | EV8-05,EV8-06,EV8-07 | REMEDIATED |
| `F-PLAN-R8-P1-04` | master schema catalog completeness for every normative PhaseF*V1 identifier | §26, §28, §29 | missing helper and new external schemas plus incomplete metadata rows | mechanically enumerated 89-identifier set with one full metadata row per identifier and set-equality KAT | R8-22 | AC8-22 | T8-16 | EV8-22 | REMEDIATED |
| `F-PLAN-R8-P1-05` | requirement/owner-decision traceability disagreement and executable traceability substance | §27, §28, §29 | two current R7 mappings and identifier-only AC/test/evidence rows | one R8 matrix, derived OD coverage, substantive procedures, and single-source KAT | R8-23,R8-24 | AC8-23,AC8-24 | T8-19 | EV8-23,EV8-24 | REMEDIATED |

No author disposition is `CLOSED`; only the fresh independent R8 reviewer may
close a remediation ID.

## 31. R8 AUTHOR AUDIT AND REQUIRED VALIDATION

The following is the post-edit author audit output. It is not independent
approval. The set, row, reference, and constructive checks in §§26-29 produce
these values; the independent reviewer must recompute them.

```text
NORMATIVE_PHASE_F_IDENTIFIERS=89
CATALOG_PHASE_F_IDENTIFIERS=89
MISSING_CATALOG_IDENTIFIERS=0
EXTRA_CATALOG_IDENTIFIERS=0
CATALOG_DUPLICATE_IDENTIFIER_ROWS=0
INCOMPLETE_SCHEMA_CATALOG_ROWS=0

EMERGENCY_REPOSITORY_PATH_AMBIGUITIES=0
EMERGENCY_GIT_PUBLICATION_AMBIGUITIES=0
CHECKER_EMERGENCY_COMMAND_AMBIGUITIES=0
MONITORING_METRIC_CARDINALITY_AMBIGUITIES=0
MONITORING_ORDER_AMBIGUITIES=0
FIXED_ORDER_SORT_CONTRADICTIONS=0
MONITORING_EVIDENCE_OBJECT_AMBIGUITIES=0
MONITORING_METRIC_EVIDENCE_MAPPING_AMBIGUITIES=0
MONITORING_REGISTRY_EVIDENCE_AMBIGUITIES=0
MONITORING_RESULT_DERIVATION_AMBIGUITIES=0
CAMPAIGN_ABANDONMENT_REVIEW_CYCLES=0
CAMPAIGN_RETENTION_SET_AMBIGUITIES=0
RELEASE_RETENTION_SET_AMBIGUITIES=0
RETENTION_COVERAGE_AMBIGUITIES=0
PRE_RELEASE_RETENTION_POSITIVE_PATH_AMBIGUITIES=0
ORPHAN_EXTERNAL_SCHEMAS=0

CATALOG_TO_REQUIREMENT_GAPS=0
REQUIREMENT_TO_CATALOG_GAPS=0
UNMAPPED_REQUIREMENTS=0
UNMAPPED_ACS=0
UNMAPPED_TESTS=0
UNMAPPED_EVIDENCE=0
UNMAPPED_ODS=0
TRACEABILITY_SUBSTANCE_GAPS=0
CONTRADICTORY_OD_MAPPING_EDGES=0
CONTRADICTORY_CURRENT_TRACEABILITY_TABLES=0
NORMATIVE_CONTRADICTIONS=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES=0
MONITORING_PASS_CONSTRUCTIBLE=yes
PRE_RELEASE_RETENTION_POSITIVE_PATH_AMBIGUITIES=0
COMPLETE_VALID_DAG_CONSTRUCTIBLE=yes

Preserved closed R7/R6 safety/scientific/DAG counters=0
PHASE_E_PROVISIONING_COMPATIBILITY_AMBIGUITIES=0
PRODUCTION_EXECUTION_ORDER_CONTRADICTIONS=0
P2_RELEASE_BYPASS_PATHS=0
```

The current R8 counts are 24 requirements, 24 primary ACs, 20 tests, 24
evidence items, 20 owner decisions, and 89 schema IDs. The OD union is derived
from the matrix and equals exactly F-OD-01 through F-OD-20. The author audit is
not a plan approval.

Before and after editing, run exactly:

```text
git diff --check
cargo fmt --all --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --test phase_e_validation
cargo test --locked --test phase_d_reporting_public_output
```

Required results are all PASS, Clippy diagnostics zero, Phase-E 38/38,
Phase-D 73/73, frozen Phase-E SHA-256
`0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`, frozen
Phase-E Git blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`, and exactly one
changed file:
`docs/engineering_specification/phase_f_physical_evidence_and_production_validation_plan.md`.

## 32. Fresh independent R8 rereview gate and handoff

The R8 independent rereview is `PENDING`. A new independent reviewer must begin
with these five positive paths, in order: (1) one canonical emergency path
pair and exact checker lookup; (2) 15 complete evidence-backed measurements
produce one byte-level PASS record; (3) completed abandonment incident precedes
review and campaign retention derives exactly from the manifest; (4) the
normative PhaseF identifier set equals the catalog set; and (5) the single R8
graph derives exactly F-OD-01..20. Failure of any path is P1. The reviewer then
independently checks preservation of all closed R7/R6 contracts.

```text
MHI V1 PHASE F
R8 PLANNING REMEDIATION HANDOFF

STARTING R7 SHA: e9cef7d7370b084f64eb91a628fb47b0b868dc63
R7 PLAN SHA-256: ab4acec5c9f8f16e8c35d14f2ca83b977a16cacc4ac2505cc5e3bacdf9980c8b
R7 PLAN BLOB: 625413873fab712961e38f6e20b98d00a5110b52
R8 PLAN REVIEW SHA: PENDING fresh independent R8 reviewer
R8 PLAN SHA-256: <computed after final R8 bytes>
R8 PLAN GIT BLOB: <computed after final R8 bytes>
CHANGED FILES: 1 expected

R7 P1 REMEDIATION: 5 grouped findings; R8 forward remediation
F-PLAN-R8-P1-01: REMEDIATED
F-PLAN-R8-P1-02: REMEDIATED
F-PLAN-R8-P1-03: REMEDIATED
F-PLAN-R8-P1-04: REMEDIATED
F-PLAN-R8-P1-05: REMEDIATED

EMERGENCY
canonical repository path: phase_f_governance/emergencies/<DIGEST>/...
alternate normative paths: 0 expected
checker path: MATCH

MONITORING
required metrics: 15
status: 4
quantity: 1
rate: 4
binding: 6
thresholded: 5
monitoring evidence schema: COMPLETE
evidence objects per PASS record: 15 expected
measurement order: FIXED
PASS constructible: yes

CAMPAIGN RETENTION
abandonment review cycle: 0 expected
campaign retention set: COMPLETE
release retention set: COMPLETE
campaign audit constructible: yes

SCHEMA CATALOG
normative PhaseF*V1 count: 89
catalog PhaseF*V1 count: 89
missing rows: 0
extra rows: 0
duplicate rows: 0
incomplete metadata rows: 0

TRACEABILITY
current R8 requirements: 24
ACs: 24
tests: 20
evidence: 24
owner decisions: 20
schemas: 89
unmapped requirements: 0
unmapped ACs: 0
unmapped tests: 0
unmapped evidence: 0
unmapped ODs: 0
catalog->requirement gaps: 0
requirement->catalog gaps: 0
traceability substance gaps: 0
contradictory OD mapping edges: 0

POSITIVE PATH
complete DAG constructible: yes
positive-path ambiguities: 0

BASELINE
diff: PASS
fmt: PASS
check: PASS
strict Clippy: PASS
Clippy diagnostics: 0
Phase E: 38/38
Phase D: 73/73
FROZEN PHASE-E PLAN: PASS
PLAN TAG CREATED: NO expected
IMPLEMENTATION BRANCH: NO expected
F0 STARTED: NO expected
KEYS CREATED: NO expected
EVIDENCE CREATED: NO expected
CLAIMS CREATED: NO expected
WORKTREE CLEAN: yes after one forward commit
READY_FOR_FRESH_PHASE_F_R8_PLAN_REREVIEW: yes
READY_FOR_PHASE_F_PLAN_APPROVAL_TAG: NO expected pending fresh R8 GO
READY_FOR_PHASE_F_IMPLEMENTATION: NO
```

No Phase-F approval tag, implementation branch, F0 activity, key, signature,
trust provisioning, registry record, physical evidence, monitoring evidence,
claim, or monitoring record is created by this planning-only remediation.
