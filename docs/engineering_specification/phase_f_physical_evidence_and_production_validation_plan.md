# MHI V1 Phase F — R11 planning-only semantic fixture / exact catalog closure

## 1. Authority, status, scope, and chronology

This document is the Phase-F R11 planning remediation of the independently
rereviewed R10 plan. It changes only this plan document. It does not create a
schema file, checker, tag, branch, key, signature, trust root, registry record,
physical evidence, monitoring record, claim, production implementation, new
scientific model, or new scientific scope.

This is planning only: no architecture expansion, no self-Git identity, and no
future-file authority in F0.

The starting R10 authority is exact:

| Authority | Value |
|---|---|
| R10 plan-review SHA | `341f9a805f94e8dd2a58c3beb7c3a68cf6adf3c7` |
| R10 plan SHA-256 | `3832fd6feaba98e834f288760c1741fa0a1bdfe1d6a1ab254cf9bcd1ce05e073` |
| R10 plan Git blob | `ca9bf58546f31f18ce0a35046dee1f46b55f9ec0` |
| R10 independent rereview | `P0=0`, `P1=4`, `P2=0`, `P3=1`, `PLAN_DECISION=NO-GO`, `PLAN_AUTHORITY=FAIL` |
| R11 status | forward remediation; independent R11 rereview `PENDING` |
| plan approval tag | absent; must remain absent in R11 |
| implementation branch | absent; must remain absent in R11 |

The immutable Phase-E authority is not changed: integrated baseline
`14942a30928b88f16914bf0bb103cc0c2a5bfa76`, reviewed implementation
`5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb`, frozen plan SHA-256
`0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`, and
frozen plan blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

The current R11 plan-review context is the external authority value
`CURRENT_PLAN_REVIEW_SHA`. It is the exact Git commit SHA frozen after the
final planning-only commit and supplied to the fresh independent plan
reviewer. It is not embedded as a concrete SHA in this plan. At review start
it must equal `HEAD`, local `main`, `origin/main`, and live remote `main`.

R1 was `NO-GO/P1=13`; R2 was `NO-GO/P1=10`; R3 was `NO-GO/P1=19`; R4 was
`NO-GO/P1=14`; R5 was `NO-GO/P1=11`; R6 was `NO-GO/P1=13`; R7 was
`NO-GO/P1=5`; R8 was `NO-GO/P1=4`; R9 was `NO-GO/P1=6 grouped fixture/catalog findings`.
R10 is forward remediation and no rejected version is described as approved.
The exact future order remains: fresh R11
rereview, plan approval, F0, F-IMPL-1 checker and permanent F-MAINT-01/02
closure, readiness, enrollment, genesis, F1, F2, F3, F4, and F5.
F1-F5 remain blocked until the applicable approved tags and authority objects
exist.

`F_IMPL_1_BEFORE_F0_EXIT`, `F_IMPL_2_BEFORE_F0_EXIT`,
`F_IMPL_3_BEFORE_F0_EXIT`, and `F_IMPL_4_BEFORE_F0_EXIT` are forbidden.
R5, R6, R7, R8, R9, and R10 author audits are not independent approval. No R6,
R7, R8, R9, R10, or R11 approval tag,
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
  | registry_record | registry_head | registration_document | validation_manifest | protocol
  | power_method_interface | power_analysis | package_manifest | dependency_audit
  | physical_unit_ledger | identity_audit | location_ledger | chain_of_custody
  | deviation_ledger | metrology_policy | metrology_check_result
  | reference_source_descriptor | reference_result | scientific_admissibility_audit
  | cohort_lock | owner_approval | execution_record | release_record | claim_state
  | reinstatement_approval | monitoring_policy | monitoring_record | incident_record
  | monitoring_evidence | retention_audit | independent_review_bundle | incident_resolution
  | emergency_registry_compromise | checker_build_evidence
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
  | incident_recorded | resolves | retention_audited | scientific_admissibility
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
  | monitoring_recorded | incident_recorded | incident_resolution_recorded
  | retention_audit_recorded
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
| `PhaseFIncidentResolutionV1` | `incident_resolution_id` | `PHASE_F_EXTERNAL_DIGEST_ID_V1` | `mhi_phase_f_incident_resolution_v1\0` | own ID | all |
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
power analysis, and emergency objects. A plan review uses the Git-commit
variant with the external review-context authority `CURRENT_PLAN_REVIEW_SHA`,
for example `{type:"git_commit",git_sha:CURRENT_PLAN_REVIEW_SHA}`. This value
is frozen after the final planning-only commit and supplied to the fresh
independent reviewer; it is not embedded in the plan before that commit
exists. At review start it must equal `HEAD`, local `main`, `origin/main`, and
live remote `main`.
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
| `ism-mechanism-health-v1-f-plan-approved` / `PhaseFPlanApprovalV1` | the frozen `CURRENT_PLAN_REVIEW_SHA` independently reviewed as GO | `plan_review_sha:GIT_SHA_V1,plan_sha256:SHA256_V1,plan_git_blob:GIT_BLOB_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-f0-decisions-approved` / `PhaseFDecisionApprovalV1` | reviewed F0 main | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,plan_review_sha:GIT_SHA_V1,decision_review_sha:GIT_SHA_V1,decision_bundle_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,decision_file_sha256:SHA256_V1,decision_git_blob:GIT_BLOB_V1,decision_count:CANONICAL_UNSIGNED_INTEGER_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-readiness-approved` / `PhaseFReadinessApprovalV1` | integrated F-IMPL-1 | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,f0_decisions_tag:PHASE_F_TAG_NAME_V1,readiness_review_sha:GIT_SHA_V1,readiness_evidence_sha256:SHA256_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-authority-enrollment-approved` / `PhaseFAuthorityEnrollmentApprovalV1` | readiness main | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,f0_decisions_tag:PHASE_F_TAG_NAME_V1,readiness_tag:PHASE_F_TAG_NAME_V1,readiness_main_sha:GIT_SHA_V1,enrollment_sha256:SHA256_V1,owner_authority_id:RUNTIME_STABLE_ID_V1,registry_authority_id:RUNTIME_STABLE_ID_V1,owner_public_key_fingerprint:SHA256_V1,registry_public_key_fingerprint:SHA256_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-trust-provisioning-approved` / `PhaseFTrustProvisioningApprovalV1` | integrated F3 main | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,f0_decisions_tag:PHASE_F_TAG_NAME_V1,readiness_tag:PHASE_F_TAG_NAME_V1,authority_enrollment_tag:PHASE_F_TAG_NAME_V1,enrollment_sha256:SHA256_V1,owner_public_key_fingerprint:SHA256_V1,registry_public_key_fingerprint:SHA256_V1,trust_root_id:RUNTIME_STABLE_ID_V1,trust_review_sha:GIT_SHA_V1,trust_store_git_blob:GIT_BLOB_V1,trust_store_sha256:SHA256_V1,f2_cohort_lock_registry_record_sha256:SHA256_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |
| `ism-mechanism-health-v1-f-physical-validation-released` / `PhaseFPhysicalReleaseApprovalV1` | final F4/F5 main | `phase_f_plan_tag:PHASE_F_TAG_NAME_V1,f0_decisions_tag:PHASE_F_TAG_NAME_V1,readiness_tag:PHASE_F_TAG_NAME_V1,authority_enrollment_tag:PHASE_F_TAG_NAME_V1,trust_provisioning_tag:PHASE_F_TAG_NAME_V1,release_code_sha:GIT_SHA_V1,protocol_sha256:SHA256_V1,cohort_lock_registry_record_sha256:SHA256_V1,owner_approval_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,owner_approval_file_sha256:SHA256_V1,validation_manifest_sha256:SHA256_V1,release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,release_file_sha256:SHA256_V1,release_registry_record_sha256:SHA256_V1,initial_claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,initial_claim_state_file_sha256:SHA256_V1,initial_claim_state_registry_record_sha256:SHA256_V1,review_bundle_sha256:SHA256_V1,approval_decision:GO` |

For the plan tag, validity additionally requires peeled target exactly equal to
`body.plan_review_sha` and the external `CURRENT_PLAN_REVIEW_SHA`; the plan
file SHA-256 and Git blob at that commit must equal `body.plan_sha256` and
`body.plan_git_blob`. A predecessor R7 or R8 target is invalid, and the plan
cannot contain its own future review SHA. Thus
`PLAN_TAG_STALE_TARGET_PATHS=0` and `PLAN_TAG_TARGET_AMBIGUITIES=0`.

All six approval schemas are `TAG_BODY` contracts. A tag body is the exact
ordered printable-ASCII message body plus one final LF, never JSON,
`TOP_LEVEL_WIRE`, a canonical external approval object, or a registry subject.
When its bytes are referenced, its only identity is
`{object_kind:"git_tag_message",object_sha256:SHA-256(exact tag-message bytes)}`.
There is no `trust_provisioning_approval` or `physical_release_approval`
object kind.

All values use the named types in §§2-3. Decisions and counts are not duplicated
in any tag; the referenced bundle independently supplies five rows and the
aggregate rule in §5. Each tag is absent before creation, never moved, and
pushed only after its target and review bundle are live. No Phase-F tag is
created during R9 planning remediation.

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
| `validation_manifest` | SHA-256 of exact validation-manifest bytes bound by the release record |
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
| `incident_resolution` | SHA-256 of complete canonical `PhaseFIncidentResolutionV1` bytes |
| `retention_audit` | SHA-256 of complete canonical retention-audit bytes |
| `independent_review_bundle` | SHA-256 of complete canonical review-bundle bytes |
| `emergency_registry_compromise` | SHA-256 of complete canonical emergency bytes |
| `checker_build_evidence` | SHA-256 of complete canonical `PhaseFCheckerBuildEvidenceV1` bytes |
| `checker_readiness_evidence` | SHA-256 of complete canonical `PhaseFCheckerReadinessEvidenceV1` bytes |
| `f5_release_candidate` | SHA-256 of complete canonical `PhaseFF5ReleaseCandidateV1` bytes |

`PhaseFRegistryRelationV1` is exactly
`{relation_type:PHASE_F_RELATION_TYPE_V1,object_kind:PHASE_F_OBJECT_KIND_V1,
object_sha256:SHA256_V1}`. Relation type is
`authorized_by|depends_on|registered_after|locks|approves|executes|releases|
changes_state_of|supersedes|references|incident_recorded|resolves|retention_audited|
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
| `incident_resolution_recorded` | `incident_resolution_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `incident_resolution` | exactly once `resolves+incident_record`; every resolution after first exactly once `registered_after+incident_resolution` | none |
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
execution_record|trust_provisioning_tag_message|owner_approval|release_record`.
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
`trust_provisioning_tag_message`; `owner_approval_id` requires exactly one
`owner_approval`; and `release_record_id` requires exactly one
`release_record`. No source kind outside the listed mapping is permitted.
For binding metrics the checker parses the named authority and compares exact
fields: `PhaseFExecutionRecordV1.release_code_sha`,
`PhaseFExecutionRecordV1.checker_binary_sha256`,
the `trust_store_sha256` and `trust_root_id` fields parsed from the exact
message bytes of the annotated tag
`ism-mechanism-health-v1-f-trust-provisioning-approved`,
`OwnerApprovalEvidenceV1.approval_record_id`, and
`PhaseFReleaseRecordV1.release_record_id`, respectively. For observational
metrics the checker validates syntax, ID/hash, source references, metric,
window, value type, and measurement equality; it does not infer scientific
truth from the underlying source. For both trust metrics, the source reference
must identify an immutable byte copy of that exact tag message. The checker
resolves the annotated tag, verifies its name, peeled target and prerequisite
tag contract, obtains the exact message bytes, hashes them as
`object_kind="git_tag_message"`, requires exact hash and byte-length equality
with the source reference, parses those bytes as
`PhaseFTrustProvisioningApprovalV1`, and compares the measurement value. A
JSON trust-provisioning object is never accepted.

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
`PhaseFObjectDigestV1`; unit IDs are sorted runtime IDs; evidence references are
sorted `PhaseFObjectReferenceV1`; type/action/status use §2 enums. `other_registered_incident`
requires an immutable incident-type definition document. `scope` is the exact
tagged union `PhaseFIncidentScopeV1`: release scope carries a release ID,
campaign abandonment carries a campaign ID and no release ID, and registry
compromise uses registry-namespace scope. ID and complete-file hash follow
§3/§9. Every newly created incident record has exactly
`incident_status="open"`. It is an immutable detection event and is never
rewritten to `contained`, `resolved`, or `superseded`; those statuses exist only
in a later resolution object. The incident contains no review hash,
review-artifact reference, or future review hash. Campaign-abandonment
construction remains exactly complete incident file -> incident SHA ->
independent review bundle targeting that incident file SHA ->
`incident_recorded` attestation, with no reverse pointer.

`PhaseFIncidentResolutionV1` is exactly
`schema_version,incident_resolution_id,incident_id,incident_record_sha256,
resolution_number,previous_resolution_sha256,effective_at,resolution_status,
evidence_references`. Types are respectively
`JSON_INTEGER_ONE,PHASE_F_EXTERNAL_DIGEST_ID_V1,
PHASE_F_EXTERNAL_DIGEST_ID_V1,SHA256_V1,CANONICAL_UNSIGNED_INTEGER_V1,
SHA256_V1|null,UTC_SECOND_TIMESTAMP_V1,PHASE_F_INCIDENT_STATUS_V1,
NONEMPTY_SORTED_UNIQUE<PhaseFObjectReferenceV1>`. Resolution status is exactly
`contained|resolved|superseded`; `open` is forbidden. Its semantic-ID domain
is `mhi_phase_f_incident_resolution_v1\0`, excluding only
`incident_resolution_id`, and its complete canonical bytes define its file
SHA. The referenced incident SHA must equal the complete incident file SHA.

The first resolution has `resolution_number="0"` and
`previous_resolution_sha256=null`. Every later resolution has a number exactly
one greater than its predecessor and `previous_resolution_sha256` exactly equal
to the predecessor's complete resolution-file SHA. Its `effective_at` is not
earlier than the predecessor. Legal history is no resolution -> unresolved;
`contained` -> unresolved; `contained` -> `resolved`; and `contained` ->
`superseded`. A terminal `resolved` or `superseded` resolution cannot be
followed by another resolution, and earlier files are immutable.

The resolution registry contract adds object kind `incident_resolution`, record
kind `incident_resolution_recorded`, and relation type `resolves`. Every
resolution record has subject `incident_resolution_id`, subject hash equal to
the complete resolution-file SHA, exactly one `resolves+incident_record`, and,
for every resolution after the first, exactly one
`registered_after+incident_resolution`. No release relation is required:
`incident_record` already scopes the release or campaign authority.

For ordinary non-abandonment campaign incidents, the permitted relation set is
only the applicable `authorized_by+decision_bundle`, package/evidence
references, and the required `incident_recorded` subject relation; the
abandonment review relation is not inherited.

`PhaseFRetentionObjectV1` is BYTE-IDENTITY ONLY and exactly the tagged union
`{type:"package_object",object_id:RUNTIME_STABLE_ID_V1,object_sha256:SHA256_V1}`
or `{type:"authority_object",object_kind:PHASE_F_OBJECT_KIND_V1,
object_sha256:SHA256_V1}`. It contains no URI or byte length. Those are
properties of an audit copy, never of a retention identity.

`PhaseFRetentionCopyVerificationV1` is exactly
`{object_reference:PhaseFObjectReferenceV1,verified_at:UTC_SECOND_TIMESTAMP_V1,
result:PHASE_F_RESULT_V1}`. A PASS copy requires an F-OD-20-approved immutable
URI scheme, availability, exact byte length, exact SHA, and retrieved bytes
whose SHA equals `object_reference.sha256`. For one object check, every PASS
copy has `object_reference.sha256` equal to the retention object's
`object_sha256`, and all PASS copies have one identical byte length and
distinct URIs. Copies are sorted by raw ASCII `immutable_uri`, then `sha256`,
then `byte_length`; there is no arbitrary primary-reference field.

`PhaseFRetentionScopeV1` is exactly the tagged union
`{type:"release",release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1}` or
`{type:"campaign",campaign_id:RUNTIME_STABLE_ID_V1,
package_manifest_sha256:SHA256_V1}`. `PhaseFRetentionObjectCheckV1` is exactly
`{object:PhaseFRetentionObjectV1,
copies:NONEMPTY_SORTED_UNIQUE<PhaseFRetentionCopyVerificationV1>,
result:PHASE_F_RESULT_V1}`. Its result is derived. Let `valid_copy_count` be
the PASS copies satisfying the exact object SHA, byte length, URI, availability,
and retrieval checks. PASS requires
`valid_copy_count >= 1 + backup_copy_count`, and for every counted PASS copy
`audited_at >= verified_at` and
`audited_at - verified_at < backup_verification_interval_seconds`. A failed,
stale, unavailable, mismatched, or insufficient copy makes the check NO-GO.
`PhaseFRetentionAuditV1` is exactly
`schema_version,retention_audit_id,scope,policy_sha256,audited_at,object_checks,
result`, with no top-level release ID. Rows sort by the kind/SHA identity key:
package `("package_object",object_id,object_sha256)` or authority
`("authority_object",object_kind,object_sha256)`.

A `retention_audited` record is authorized by the decision bundle and has
exactly one scope-specific relation: release scope uses
`references+release_record`; campaign scope uses `references+package_manifest`.
Every later audit of the same scope has exactly one
`registered_after+retention_audit`. A campaign audit never requires a release
relation, and a release audit never requires a package-manifest relation merely
because its objects came from a package.

Retention starts when an authority object is first referenced by a valid
registry record. After terminal state, the deadline is terminal `effective_at`
plus F0 retention seconds; retrieval is required while `now<deadline`, unless
another nonterminal release references it. Pre-release campaigns end retention
only by an independently reviewed `campaign_abandonment` incident with campaign
scope; the deadline is `incident.detected_at + retention_seconds`, and all
campaign identities are retained until that deadline. Deletion is never silent.
Replacement is an additional verified copy with identical bytes, SHA, and
length recorded in the next audit; different bytes are a new object. If every
copy disappears before deadline, retention failure is required and claim-status
is NOT_ACTIVE.

`CampaignRetentionSetV1` is exactly one authority identity
`{type:"authority_object",object_kind:"package_manifest",object_sha256:SHA256(M)}`
for manifest `M`, plus one package identity for each manifest object `O`:
`{type:"package_object",object_id:O.object_id,
object_sha256:O.object_reference.sha256}`. No URI or length is copied into the
set. `ReleaseRetentionSetV1` is the exact union of the campaign set, static
release identities, accepted monitoring identities, unresolved incident
identities, and the resolution authorities needed by the audited-at decision.

| Retention member | Source authority | Source field / relation | Object kind | SHA rule |
|---|---|---|---|---|
| protocol | release record | `protocol_sha256` binding | `protocol` | exact bound protocol bytes |
| power_analysis | cohort lock | `power_analysis_sha256` binding | `power_analysis` | exact bound complete file |
| cohort_lock | release record | `cohort_lock_record_sha256` binding | `cohort_lock` | exact bound complete file |
| owner_approval | release record | `owner_approval_file_sha256` binding | `owner_approval` | exact certified owner-approval bytes |
| execution_record | release record | `execution_record_sha256` binding | `execution_record` | exact bound complete file |
| release_record | registry chain | `release_registered` subject | `release_record` | exact subject complete-file SHA |
| validation_manifest | release record | `validation_manifest_sha256` binding | `validation_manifest` | exact bound validation-manifest bytes |
| monitoring_policy | release record | `monitoring_policy_sha256` binding | `monitoring_policy` | exact bound complete file |
| metrology_policy | release record | `metrology_policy_sha256` binding | `metrology_policy` | exact bound complete file |
| trust tag message | named Git tag | exact message of `ism-mechanism-health-v1-f-trust-provisioning-approved` | `git_tag_message` | SHA-256 exact annotated-tag message bytes |
| F5 review bundle | initial ACTIVE state | `activation_review_bundle_sha256` | `independent_review_bundle` | exact bound complete file |
| initial state | registry chain | first valid `claim_state_changed` subject | `claim_state` | exact subject complete-file SHA |
| latest state at `audited_at` | registry chain | latest eligible `claim_state_changed` by verified sequence and state chain | `claim_state` | exact eligible subject complete-file SHA |

The validation manifest is explicit because it is a normative release-record
binding. The trust identity is obtained only by resolving the named annotated
tag and hashing its exact message bytes; `trust_provisioning_approval` is not an
object kind. No URI or byte length is derived by this table. The current audit
contains exactly one object check for every identity in the set, with exact set
equality and no registry records or human-selected extras.
`RELEASE_RETENTION_STATIC_IDENTITY_AMBIGUITIES=0`.

An incident is eligible at `audited_at` exactly when
`incident.detected_at <= audited_at` and its `incident_recorded` registry
record's `created_at <= audited_at`. Collect all valid resolution records for
that incident whose `effective_at <= audited_at` and whose registry-record
`created_at <= audited_at`; validate the complete resolution-number and
predecessor-SHA chain. No eligible resolution means UNRESOLVED. Latest eligible
`contained` means UNRESOLVED; `resolved` means RESOLVED; `superseded` means
RESOLVED/SUPERSEDED. Ordering authority is verified registry sequence plus the
exact resolution-number chain; timestamps alone cannot reorder history.
`INCIDENT_AUDIT_TIME_STATUS_AMBIGUITIES=0`.

Release retention includes every release-scoped incident record unresolved at
`audited_at`, including open records with no resolution and contained records.
It excludes resolved/superseded incident records but retains every resolution
authority through the terminal resolution used to prove that exclusion. A
broken chain, sequence conflict, terminal continuation, or ambiguous chronology
is NO-GO.



Release retention includes every `PhaseFMonitoringRecordV1` whose
`monitoring_recorded` attestation is valid, whose `window_end <= audited_at`,
and which is an accepted PASS window under §14's exact current-chain contract.
Unregistered PASS files, suspend records, late unaccepted records, and records
after `audited_at` are excluded. Monitoring membership is exactly
`{type:"authority_object",object_kind:"monitoring_record",
object_sha256:<subject complete-file SHA>}`. Initial/latest state identities
with the same kind and SHA collapse to one member. The audit copy rows carry
all URI and length information; membership never infers either from a SHA.
`RETENTION_IDENTITY_LOCATOR_AMBIGUITIES=0`,
`RELEASE_RETENTION_DEDUP_AMBIGUITIES=0`,
`RELEASE_RETENTION_REFERENCE_DERIVATION_AMBIGUITIES=0`.

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
the one current catalog is the R9 MASTER SCHEMA CATALOG in §33. Each row includes exact
fields, identity, complete-file hash, producer, validator, stage, requirement,
AC, test, evidence, and registry relation. The canonical substantive
requirement/AC/test/evidence procedures were §19.2; every R7 identifier in this
snapshot is a historical cross-reference only and is not current R9 acceptance
authority.

| Schema | Field closure / identity | Producer; validator; stage | Registry relation | Requirement / AC / test / evidence |
|---|---|---|---|---|
| `PhaseFDecisionBundleV1` | §4; §3 ID; complete hash; unsigned | F0; checker; F0 | protocol authority | R5-01 / AC5-01 / T5-01 / EV5-01 |
| `PhaseFIndependentReviewBundleV1` | §5 exact tagged target and five rows; §3 ID; complete hash | independent roles; checker; approvals | tag evidence | R7-02/R7-05 / AC7-02/05 / T7-02/05 / EV7-02/05 |
| `PhaseFReviewTargetV1` | §2 exact `git_commit` or `external_object` tagged union | reviewer; target validator; all review gates | nested in review bundle | R7-02 / AC7-02 / T7-02 / EV7-02 |
| `PhaseFIndependentReviewV1` | §5 exact five role fields with no row ID | independent reviewer; review validator; all gates | nested in review bundle | R7-05 / AC7-05 / T7-05 / EV7-05 |
| `PhaseFProtocolProjectionV1` | §4 exact plan contract; no wire ID | checker; projection; F1 | protocol | R5-03 / AC5-03 / T5-03 / EV5-03 |
| `PhaseFAuthorityEnrollmentV1` | §5.2 current exact enrollment closure; R7 historical accounting | governance; enrollment; readiness | authority_enrolled | R5-04 / AC5-04 / T5-04 / EV5-04 |
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
| `PhaseFPlanApprovalV1` | §6 plan body fields, fixed order, ASCII plus final LF | independent reviewer / tag validator / plan gate | `ism-mechanism-health-v1-f-plan-approved` / historical R7 predecessor target (non-current) | five-role plan review / R7-12 / AC7-12 / T7-12 / EV7-12 |
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
the historical R8 closure and metadata are retained above. The complete current R9 closure and metadata are in §33. `JSON_INTEGER_ONE` is the literal
JSON integer `1`; `SORTED_UNIQUE<T>` is a strictly increasing JSON array whose
member type is exactly `T`; `NONEMPTY_SORTED_UNIQUE<T>` adds nonempty; and
`JCS_OBJECT<T>` means the complete canonical object type `T`. These were closed
R7 constructions, retained for regression accounting. The current R9 catalog
is the single source of catalog authority in §33.

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
and never a top-level release ID. These are historical catalog entries; §33 is
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
current positive-path authority is §34 and §41; where this historical text
differs, the current R9 sections control.

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
operational authority is the R9 requirement matrix in §34. Each historical row
has one primary AC, one test, and one evidence item; no historical row changes
the R8 contract. Every F-OD-01 through F-OD-20 is mapped only by the current
R9 matrix; no additional owner decision exists.

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
reintroduce superseded fields or relations into the historical R8 schemas.
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

## 26. Historical R8 master-schema catalog snapshot (non-normative)

This entire section is retained historical R8 accounting and is not current
authority. The current R9 catalog is §33.

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
| `PhaseFAuthorityEnrollmentV1` | TOP_LEVEL_WIRE | historical R8 §5.1 exact enrollment object | §3 exact domain and own ID | exact canonical file bytes | governance authority | enrollment strict parser | enrollment | subject `authority_enrollment` | R8-22 | AC8-22 | T8-16 | EV8-22 |
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

## 27. Historical R8 normative requirement matrix (non-current)

This entire section is retained historical R8 accounting and is not current
authority. The current R9 matrix is §34.

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

## 28. Historical R8 AC, TEST, AND F-EV executability (non-current)

This entire section is retained historical R8 accounting and is not current
authority. The current R9 traceability is in §§35–37.

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

## 29. Historical R8 positive controls and counterexamples (non-current)

This entire section is retained historical R8 accounting and is not current
authority. The current R9 controls are in §§35–37.

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

## 30. Historical R8 remediation ledger (non-current)

This entire section is retained historical R8 accounting and is not current
authority. The current R9 ledger is §38.

| R8 remediation ID | R7 P1 finding | R8 exact section | root cause | R8 remediation | current R8 requirement IDs | AC IDs | test IDs | F-EV IDs | AUTHOR DISPOSITION |
|---|---|---|---|---|---|---|---|---|---|
| `F-PLAN-R8-P1-01` | emergency publication canonical-path contradiction | §15, §26, §27, §28, §29 | checker prose and positive path omitted `phase_f_governance/` in one lookup | one full-prefixed path pair, exact ten-step verifier, and positive/counterexample KATs | R8-01 | AC8-01 | T8-01,T8-02 | EV8-01 | REMEDIATED |
| `F-PLAN-R8-P1-02` | monitoring evidence authority, metric cardinality, and measurement ordering | §14, §26, §27, §28, §29 | bare evidence hash, conflicting category counts, and generic sorting rule | complete evidence schema/source map, 4/1/4/6 partition, fixed order, fixed thresholds, exact registry relations | R8-02,R8-03,R8-04 | AC8-02,AC8-03,AC8-04 | T8-03,T8-04,T8-05,T8-06,T8-07,T8-08,T8-09,T8-10 | EV8-02,EV8-03,EV8-04 | REMEDIATED |
| `F-PLAN-R8-P1-03` | campaign-abandonment review cycle and retention-object coverage derivation | §15, §26, §27, §28, §29 | incident tried to contain a future review and retention used an untyped digest/set | incident-first review DAG, typed retention object key, manifest-derived campaign set, release union | R8-05,R8-06,R8-07 | AC8-05,AC8-06,AC8-07 | T8-11,T8-12,T8-13,T8-14,T8-15 | EV8-05,EV8-06,EV8-07 | REMEDIATED |
| `F-PLAN-R8-P1-04` | master schema catalog completeness for every normative PhaseF*V1 identifier | §26, §28, §29 | missing helper and new external schemas plus incomplete metadata rows | mechanically enumerated 89-identifier set with one full metadata row per identifier and set-equality KAT | R8-22 | AC8-22 | T8-16 | EV8-22 | REMEDIATED |
| `F-PLAN-R8-P1-05` | requirement/owner-decision traceability disagreement and executable traceability substance | §27, §28, §29 | two current R7 mappings and identifier-only AC/test/evidence rows | one R8 matrix, derived OD coverage, substantive procedures, and single-source KAT | R8-23,R8-24 | AC8-23,AC8-24 | T8-19 | EV8-23,EV8-24 | REMEDIATED |

No author disposition is `CLOSED`; only the fresh independent R8 reviewer may
close a remediation ID.

## 31. Historical R8 author audit and validation (non-current)

This entire section is retained historical R8 accounting and is not current
authority. The current R9 audit is §39.

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

## 32. Historical R8 rereview gate and handoff (non-current)

This entire section is retained historical R8 accounting and is not current
authority. The current R9 handoff is §41.

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
## 33. Historical R9 MASTER SCHEMA CATALOG (NON-CURRENT)

This is a retained historical R9 catalog and is not current authority. Sections
16 and 26 are historical accounting only; the current catalog and matrix are
defined in §42 onward. The R9 catalog was set-equal to every distinct normative identifier matching
`PhaseF[A-Za-z0-9_]*V1` in this complete plan. The R9 additions are
`PhaseFIncidentResolutionV1` and `PhaseFRetentionCopyVerificationV1`; no
external schema file is created.

### 33.1 NORMATIVE_PHASE_F_IDENTIFIER_SET

The mechanically enumerated set is exactly 91 identifiers:

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
PhaseFIncidentResolutionV1
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
PhaseFRetentionCopyVerificationV1
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

### 33.2 HISTORICAL CATALOG_PHASE_F_IDENTIFIER_SET

The following table is the complete current catalog. Every row has the required
identifier, category, exact field closure, identity rule, complete-file hash
meaning, producer, validator, stage, registry behavior, current requirement,
primary AC, test, and F-EV metadata.

| exact identifier | category | exact field-closure section | semantic-ID rule | complete-file hash | producer | validator | stage | registry behavior | current requirement IDs | current primary AC IDs | current test IDs | current F-EV IDs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `PhaseFArgvV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFAuthorityEnrollmentApprovalV1` | TAG_BODY | §6 exact ASCII enrollment approval body | none | not applicable as JSON; exact annotated-tag message SHA when referenced | independent five-role enrollment gate; tag operator non-authoritative | tag validator | enrollment gate | generic `git_tag_message` only where referenced; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFAuthorityEnrollmentV1` | TOP_LEVEL_WIRE | §5.2 exact enrollment closure | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFChainOfCustodyV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCheckListV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCheckerBuildEvidenceV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCheckerExitCodeV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCheckerReadinessEvidenceV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCheckerReportV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCheckerStdoutV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFClaimStateRecordV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCohortLockRecordV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCommandV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFCustodyEventV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDecisionApprovalV1` | TAG_BODY | §6 exact ASCII decision approval body | none | not applicable as JSON; exact annotated-tag message SHA when referenced | independent five-role F0 gate; tag operator non-authoritative | tag validator | F0 gate | generic `git_tag_message` only where referenced; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDecisionBundleV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDecisionRowV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDecisionValueV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDependencyAuditV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDependencyEdgeV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDeviationEventV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDeviationLedgerRevisionV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFDeviationLedgerV1` | PLAN_ONLY_CONTRACT | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFEndpointMetrologyPolicyV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFEnvironmentEntryV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFExecutionRecordV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFF5ReleaseCandidateV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFIdentityComparisonV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFIncidentRecordV1` | TOP_LEVEL_WIRE | §15 exact open detection-record closure | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | operations / audited retention | subject kind named by §9 where registered | R9-06 | AC9-06 | T9-CX-07,T9-CX-08,T9-CX-24,T9-CX-25,T9-CX-26,T9-CX-27 | EV9-06 |
| `PhaseFIncidentResolutionV1` | TOP_LEVEL_WIRE | §15 exact resolution fields, chain, and §3 ID | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | operations / audited retention | subject kind named by §9 where registered | R9-06 | AC9-06 | T9-CX-11,T9-CX-12,T9-CX-24,T9-CX-25,T9-CX-26,T9-CX-27 | EV9-06 |
| `PhaseFIncidentScopeV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | operations / audited retention | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFIndependentReviewBundleV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFIndependentReviewV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFLODLOQPolicyV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFLocationLedgerV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFLocationV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMethodVersionV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMetricThresholdV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMetrologyCheckResultV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMetrologyCheckSpecV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMetrologyPolicyV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMonitoringBreachV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMonitoringEvidenceV1` | TOP_LEVEL_WIRE | §14 exact evidence closure; trust fields verify tag bytes | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMonitoringMeasurementV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMonitoringPolicyV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMonitoringRecordV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFMonitoringSourceReferenceV1` | NESTED_WIRE | §14 exact source-reference closure including trust tag message | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-02,R9-07 | AC9-02,AC9-07 | T9-CX-13,T9-CX-14,T9-CX-15,T9-CX-16,T9-CX-17 | EV9-02,EV9-07 |
| `PhaseFMonitoringValueV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFNamedDigestV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFObjectDigestV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFObjectReferenceV1` | NESTED_WIRE | §10 exact immutable URI/SHA/byte-length reference | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFOutputSpecV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPackageBindingV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPackageManifestV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPackageObjectV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFParameterSpecV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFParameterValueRowV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPhysicalIdentityAuditV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPhysicalReleaseApprovalV1` | TAG_BODY | §6 exact ASCII physical-release approval body | none | not applicable as JSON; exact annotated-tag message SHA when referenced | independent five-role release gate; tag operator non-authoritative | tag validator | physical-release gate | generic `git_tag_message` only where referenced; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPhysicalUnitLedgerV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPlanApprovalV1` | TAG_BODY | §6 exact ASCII plan tag body and CURRENT_PLAN_REVIEW_SHA target | none | not applicable as JSON; exact annotated-tag message SHA when referenced | independent five-role plan gate; tag operator non-authoritative | tag validator | plan gate | tag message only; generic `git_tag_message` byte identity; no independent registry subject | R9-01 | AC9-01 | T9-CX-01,T9-CX-02,T9-CX-18 | EV9-01 |
| `PhaseFPowerAnalysisRecordV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPowerMethodInterfaceV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFPowerOutputValueV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFProtocolProjectionV1` | PLAN_ONLY_CONTRACT | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFQuantifiedUncertaintyV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFRangeRuleV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFReadinessApprovalV1` | TAG_BODY | §6 exact ASCII readiness approval body | none | not applicable as JSON; exact annotated-tag message SHA when referenced | independent five-role readiness gate; tag operator non-authoritative | tag validator | readiness gate | generic `git_tag_message` only where referenced; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFReferenceAssessmentV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFReferenceResultV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFReferenceSourceDescriptorV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFRegistryCompromiseEmergencyV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFRegistryHeadV1` | TOP_LEVEL_WIRE | §8 exact head fields and signature bytes | none | exact canonical signed head bytes | registry authority | head strict parser | all registry operations | resolver object; no subject row | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFRegistryRecordV1` | TOP_LEVEL_WIRE | §8 exact record fields and signature bytes | none | exact canonical signed record bytes | registry authority | registry strict parser | all registry operations | chain record; subject and relation fields exact | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFRegistryRelationV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFReinstatementApprovalV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFReleaseRecordV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFRetentionAuditV1` | TOP_LEVEL_WIRE | §15 exact retention-audit closure | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | operations / audited retention | subject kind named by §9 where registered | R9-03,R9-04,R9-05 | AC9-03,AC9-04,AC9-05 | T9-KAT-01 | EV9-03,EV9-04,EV9-05 |
| `PhaseFRetentionCopyVerificationV1` | NESTED_WIRE | §15 exact copy-verification fields | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | operations / audited retention | nested; no independent registry subject | R9-07 | AC9-07 | T9-CX-04,T9-CX-05,T9-CX-23 | EV9-07 |
| `PhaseFRetentionObjectCheckV1` | NESTED_WIRE | §15 exact copies/result closure | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | operations / audited retention | nested; no independent registry subject | R9-07 | AC9-07 | T9-CX-03,T9-CX-04,T9-CX-05,T9-CX-06 | EV9-07 |
| `PhaseFRetentionObjectV1` | NESTED_WIRE | §15 exact byte-identity tagged union | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | operations / audited retention | nested; no independent registry subject | R9-03,R9-04,R9-05 | AC9-03,AC9-04,AC9-05 | T9-CX-03,T9-CX-04,T9-CX-05,T9-CX-06,T9-CX-22,T9-CX-23,T9-CX-30 | EV9-03,EV9-04,EV9-05 |
| `PhaseFRetentionScopeV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | operations / audited retention | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFRetrievalVerificationV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFReviewTargetV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFScientificAdmissibilityAuditV1` | TOP_LEVEL_WIRE | §2–§15 exact closure; unchanged by R9 | §3 exact domain and own ID | exact canonical complete-file SHA | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | subject kind named by §9 where registered | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFSensitivityCaseV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFSensitivityOverrideV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFTrustProvisioningApprovalV1` | TAG_BODY | §6 exact ASCII trust tag body | none | not applicable as JSON; exact annotated-tag message SHA when referenced | independent five-role trust gate; tag operator non-authoritative | tag validator | F3 trust gate | tag message only; generic `git_tag_message` byte identity; no independent registry subject | R9-02 | AC9-02 | T9-CX-13,T9-CX-14,T9-CX-15,T9-CX-16,T9-CX-17,T9-CX-19,T9-CX-21 | EV9-02 |
| `PhaseFUncertaintyPolicyV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFUnitEntryV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |
| `PhaseFUnitRuleV1` | NESTED_WIRE | §2–§15 exact closure; unchanged by R9 | none / containing-object-only | containing-object-only | named Phase-F authority | strict schema/authority validator | existing Phase-F stage | nested; no independent registry subject | R9-09 | AC9-09 | T9-CAT | EV9-09 |

The catalog audit requires exact set equality, one row per identifier, no blank
metadata, no grouped identifier row, no duplicate row, and bidirectional
catalog-to-R9-matrix coverage. In particular,
`PhaseFAuthorityEnrollmentV1 -> §5.2`,
`PhaseFTrustProvisioningApprovalV1 -> TAG_BODY / §6 exact tag bytes`, all
six approval schemas are TAG_BODY, and neither obsolete approval object kind is
present. `SCHEMA_CATALOG_SECTION_POINTER_ERRORS=0`,
`APPROVAL_TAG_SCHEMA_CATEGORY_AMBIGUITIES=0`,
`CATALOG_DUPLICATE_IDENTIFIER_ROWS=0`, and
`INCOMPLETE_SCHEMA_CATALOG_ROWS=0`.
## 34. Historical R9 CURRENT NORMATIVE REQUIREMENT MATRIX (NON-CURRENT)

Sections 27 and 28 are historical R8 accounting and are non-current. This is
the retained R9 matrix, not the current matrix. The current R10 matrix is in §43.
The R9 matrix carried forward all still-valid R8 contracts and
splits the four R9 findings into plan-target, trust, static-retention,
incident-chronology, monitoring-membership, and copy-coverage requirements.
`owner_decision_ids` in this table is the only current F-OD mapping source.

| requirement_id | normative_statement | owner_decision_ids | schema_ids | stage | review_roles | primary_ac_id | test_ids | evidence_ids |
|---|---|---|---|---|---|---|---|---|
| `R9-01` | Future plan tag targets only externally frozen CURRENT_PLAN_REVIEW_SHA and matches body and plan bytes at that commit. | none | PhaseFPlanApprovalV1,PhaseFIndependentReviewBundleV1,PhaseFReviewTargetV1 | plan review | architecture_data,security | AC9-01 | T9-POS-PLAN,T9-CX-01,T9-CX-02,T9-CX-18 | EV9-01 |
| `R9-02` | Trust provisioning has one exact annotated-tag message authority and trust monitoring verifies tag bytes and fields. | F-OD-04 | PhaseFTrustProvisioningApprovalV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringEvidenceV1,PhaseFObjectReferenceV1 | F3,F5+ | security,operations_governance | AC9-02 | T9-POS-TRUST,T9-CX-13,T9-CX-14,T9-CX-15,T9-CX-16,T9-CX-17,T9-CX-19,T9-CX-21 | EV9-02 |
| `R9-03` | Campaign retention derives a manifest/object identity set using kind and SHA only. | F-OD-18 | PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetentionAuditV1 | retention | operations_governance,security | AC9-03 | T9-POS-RETENTION,T9-CX-22,T9-CX-30 | EV9-03 |
| `R9-04` | Static release retention derives every mandatory release authority as an exact kind/SHA identity, including validation_manifest and trust tag message. | F-OD-13,F-OD-14,F-OD-15 | PhaseFReleaseRecordV1,PhaseFCohortLockRecordV1,PhaseFExecutionRecordV1,PhaseFMonitoringPolicyV1,PhaseFMetrologyPolicyV1,PhaseFClaimStateRecordV1,PhaseFIndependentReviewBundleV1,PhaseFTrustProvisioningApprovalV1,PhaseFRetentionObjectV1 | F5+ | operations_governance,security | AC9-04 | T9-POS-RETENTION,T9-CX-03,T9-CX-06 | EV9-04 |
| `R9-05` | Immutable incident detections and chained resolutions classify release incidents deterministically at audited_at. | F-OD-16 | PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFRetentionObjectV1 | operations | operations_governance,security | AC9-05 | T9-CX-07,T9-CX-08,T9-CX-11,T9-CX-12,T9-CX-24,T9-CX-25,T9-CX-26,T9-CX-27 | EV9-05 |
| `R9-06` | Release retention includes accepted PASS monitoring identities through audited_at and excludes all unaccepted or future records. | F-OD-19 | PhaseFMonitoringRecordV1,PhaseFMonitoringEvidenceV1,PhaseFRegistryRecordV1,PhaseFRetentionObjectV1 | F5+ | operations_governance | AC9-06 | T9-CX-09,T9-CX-10,T9-CX-28,T9-CX-29 | EV9-06 |
| `R9-07` | Every retention identity has exact verified immutable copies, freshness, byte length/SHA, distinct URI, and required backup count. | F-OD-20 | PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionAuditV1,PhaseFObjectReferenceV1 | retention | security,operations_governance | AC9-07 | T9-POS-RETENTION,T9-CX-04,T9-CX-05,T9-CX-23 | EV9-07 |
| `R9-08` | Current R9 catalog exactly enumerates the normative PhaseF identifier set with complete metadata and no obsolete approval kind. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data | AC9-08 | T9-CAT,T9-CX-19,T9-CX-20 | EV9-08 |
| `R9-09` | Current R9 requirements, catalog, AC, test, and F-EV rows are bidirectionally substantive and use one matrix. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data | AC9-09 | T9-CAT,T9-TRACE | EV9-09 |
| `R9-10` | F0 retains exactly 20 owner decisions and exact runtime projection. | F-OD-01,F-OD-02 | PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFProtocolProjectionV1 | F0 | architecture_data,scientific_metrology | AC9-10 | T9-POS-DAG | EV9-10 |
| `R9-11` | Readiness, unsigned enrollment, genesis, signatures, sequence, and relation contracts remain exact. | F-OD-03,F-OD-04 | PhaseFCheckerBuildEvidenceV1,PhaseFCheckerReadinessEvidenceV1,PhaseFAuthorityEnrollmentV1,PhaseFRegistryRecordV1,PhaseFRegistryHeadV1 | readiness/enrollment | security,compatibility | AC9-11 | T9-POS-DAG | EV9-11 |
| `R9-12` | Retrieval, package classification, dependency, and package relations remain deterministic. | F-OD-05,F-OD-06,F-OD-07,F-OD-08 | PhaseFRetrievalVerificationV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFPackageBindingV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1 | F2 | security,scientific_metrology | AC9-12 | T9-POS-DAG | EV9-12 |
| `R9-13` | Physical identity, pseudoreplication, location, and custody continuity remain exact. | F-OD-09,F-OD-10 | PhaseFPhysicalUnitLedgerV1,PhaseFUnitEntryV1,PhaseFPhysicalIdentityAuditV1,PhaseFIdentityComparisonV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFChainOfCustodyV1,PhaseFCustodyEventV1 | F2-F4 | scientific_metrology,operations_governance | AC9-13 | T9-POS-DAG | EV9-13 |
| `R9-14` | Deviation revisions remain immutable, stable-ID, action-compatible, and acyclic. | F-OD-11 | PhaseFDeviationLedgerV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationEventV1 | F1-F4 | operations_governance,security | AC9-14 | T9-POS-DAG | EV9-14 |
| `R9-15` | Power interface, typed values, ranges, units, sensitivity cases, outputs, review, and registration remain exact. | F-OD-12 | PhaseFPowerMethodInterfaceV1,PhaseFMethodVersionV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFOutputSpecV1,PhaseFPowerOutputValueV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFRangeRuleV1,PhaseFUnitRuleV1,PhaseFPowerAnalysisRecordV1 | F1 | scientific_metrology | AC9-15 | T9-POS-DAG | EV9-15 |
| `R9-16` | Endpoint-qualified metrology and reference provenance/admissibility remain exact without new scientific scope. | none | PhaseFMetrologyPolicyV1,PhaseFEndpointMetrologyPolicyV1,PhaseFCheckListV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyCheckResultV1,PhaseFLODLOQPolicyV1,PhaseFUncertaintyPolicyV1,PhaseFReferenceSourceDescriptorV1,PhaseFReferenceResultV1,PhaseFScientificAdmissibilityAuditV1,PhaseFReferenceAssessmentV1,PhaseFQuantifiedUncertaintyV1 | F0-F2 | scientific_metrology | AC9-16 | T9-POS-DAG | EV9-16 |
| `R9-17` | Claim-state causes, release/state chronology, final release authority, and P2 readiness remain exact. | none | PhaseFClaimStateRecordV1,PhaseFReinstatementApprovalV1,PhaseFIncidentScopeV1,PhaseFReleaseRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFPhysicalReleaseApprovalV1 | F4-F5 | operations_governance,architecture_data | AC9-17 | T9-POS-DAG | EV9-17 |
| `R9-18` | Registry-compromise emergency publication and fail-closed claim status remain acyclic and path-exact. | none | PhaseFRegistryCompromiseEmergencyV1,PhaseFIndependentReviewBundleV1,PhaseFObjectReferenceV1 | emergency | security,operations_governance | AC9-18 | T9-POS-DAG | EV9-18 |
| `R9-19` | Complete Phase-F positive path remains constructible in production runner order with no future-file or self-Git dependency. | none | PhaseFCommandV1,PhaseFArgvV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFCheckerExitCodeV1 | all | architecture_data,compatibility | AC9-19 | T9-POS-DAG,T9-KAT-01 | EV9-19 |
| `R9-20` | Frozen Phase-E and all previously closed R7/R8 safety, scientific, identity, custody, DAG, runtime, and production-order contracts remain unchanged. | none | PhaseFDecisionBundleV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFRetentionAuditV1,PhaseFClaimStateRecordV1 | all | architecture_data,security,scientific_metrology | AC9-20 | T9-POS-DAG | EV9-20 |
| `R9-21` | Fixed 15-metric monitoring contract remains exact except for the trust-source correction. | F-OD-17 | PhaseFMonitoringPolicyV1,PhaseFMetricThresholdV1,PhaseFMonitoringRecordV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringSourceReferenceV1 | F0/F5+ | operations_governance,security | AC9-21 | T9-POS-TRUST,T9-CX-13,T9-CX-14,T9-CX-15,T9-CX-16,T9-CX-17 | EV9-21 |
| `R9-22` | Campaign-abandonment review remains incident-first and cannot introduce a back-pointer or retention cycle. | F-OD-18 | PhaseFIncidentRecordV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFPackageManifestV1 | all | operations_governance,security | AC9-22 | T9-POS-DAG | EV9-22 |
| `R9-23` | Owner-decision coverage has exactly one current mapping source and union F-OD-01 through F-OD-20. | none | PhaseFDecisionBundleV1,PhaseFDecisionRowV1 | plan review | architecture_data | AC9-23 | T9-TRACE | EV9-23 |
| `R9-24` | R9 retention outputs are exact-set equal, de-duplicated, copy-covered, and fail closed on missing authority. | F-OD-20 | PhaseFRetentionAuditV1,PhaseFRetentionObjectV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionCopyVerificationV1 | retention | security,operations_governance | AC9-24 | T9-KAT-01,T9-CX-03,T9-CX-04,T9-CX-05,T9-CX-06,T9-CX-22,T9-CX-23,T9-CX-30 | EV9-24 |
| `R9-25` | One deterministic R9 KAT independently derives the complete release set, incident/monitoring chronology, and full copy coverage. | none | PhaseFReleaseRecordV1,PhaseFClaimStateRecordV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFMonitoringRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionCopyVerificationV1 | F5+ | architecture_data,security,operations_governance | AC9-25 | T9-KAT-01,T9-POS-RETENTION | EV9-25 |

The derived owner-decision union is exactly
`F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-05,F-OD-06,F-OD-07,F-OD-08,F-OD-09,F-OD-10,
F-OD-11,F-OD-12,F-OD-13,F-OD-14,F-OD-15,F-OD-16,F-OD-17,F-OD-18,F-OD-19,F-OD-20`.
No second current requirement-to-decision mapping exists.

## 35. Historical R9 ACCEPTANCE CRITERIA (NON-CURRENT)

Every current R9 acceptance criterion has preconditions, exact inputs, exact
operation, expected result, expected output, and a failure oracle. Release
retention is split into independent static-set, incident-status,
monitoring-membership, and copy-coverage criteria.

| AC ID | preconditions | exact inputs | exact operation | expected result | expected output | failure oracle |
|---|---|---|---|---|---|---|
| `AC9-01` | external plan review context X, plan bytes, review bundle, and future tag | peel the named plan tag and compare target, body.plan_review_sha, X, plan SHA-256, and Git blob | only exact X/body/plan identities pass | plan-tag validation report | predecessor, mismatch, or self-SHA path is INVALID TAG / NO-GO |
| `AC9-02` | trust tag, exact message copy, source reference, and both trust measurements | resolve and name-check the tag; validate peeled prerequisites; hash bytes; compare length and both fields | one exact tag message passes both trust bindings | tag-byte and binding transcript | JSON object, wrong bytes/hash, or field mismatch rejects |
| `AC9-03` | manifest M and every manifest object O | derive package_manifest SHA and package object ID/SHA keys without URI/length | exact campaign identity set passes | CampaignRetentionSetV1 | missing, extra, duplicate, or locator-derived member fails |
| `AC9-04` | release bindings, registry chain, trust tag, F5 bundle, and states | apply every row of the static release derivation table and de-duplicate keys | exact static set passes | static retention derivation report | missing/unresolvable binding is NO-GO |
| `AC9-05` | incident, registry records, resolutions, and audited_at | filter both event and registry creation times; validate number/SHA chain; classify latest eligible status | no resolution/open/contained unresolved; terminal statuses resolved | audited-at incident report | broken chain, terminal continuation, or chronology conflict fails |
| `AC9-06` | monitoring history with accepted PASS, suspend, unregistered, late, and future records | validate attestation, exact PASS/current chain, window_end, and audited_at cutoff | exact accepted monitoring identity set passes | accepted monitoring set | include excluded record or omit accepted PASS fails |
| `AC9-07` | identity set, retrieved copy bytes, and F-OD-20 policy | verify URI scheme, availability, byte length/SHA, URI distinctness, freshness, count, and set equality | full copy coverage passes only at 1+backup_count | retention copy transcript | any missing/extra/bad/stale/insufficient copy is NO-GO |
| `AC9-08` | complete plan and R9 catalog | mechanically enumerate 91 IDs and inspect every metadata cell and uniqueness | 91=91 and all catalog counters zero | catalog audit | missing/extra/duplicate/blank/stale pointer fails |
| `AC9-09` | all current R9 traceability tables | resolve every reference both directions and inspect required substance | all mapping counters zero | traceability audit | historical table treated as current or identifier-only row fails |
| `AC9-10` | twenty F0 rows and runtime projection | validate fixed IDs, values, owners, and projection with no future object fields | exact F0 bundle/projection passes | F0 projection report | 21st row or future field fails |
| `AC9-11` | builds, readiness, enrollment, genesis, and head | validate schemas, unsigned enrollment, strict signatures, sequence, predecessor, relations, and currentness | preserved registry path passes | registry authority transcript | pointer/signature/sequence/relation error fails |
| `AC9-12` | retrieval, package, dependency, and relation fixtures | verify immutable bytes, lengths, hashes, role classification, DAG, and registry relations | package authority passes | package validation report | unavailable object or relation/classification gap fails |
| `AC9-13` | unit/location/custody ledgers and comparisons | recompute native identity, pseudoreplication, location, and event continuity | physical authority passes | identity/custody report | alias, discontinuity, or post-destroy use fails |
| `AC9-14` | ledger ID, prior revision, and new deviation revision | verify immutable predecessor SHA, revision sequence, action compatibility, and acyclicity | one-way revision passes | deviation report | prior mutation or incompatible action fails |
| `AC9-15` | F0 method/version, power interface/analysis, cases, and review | validate typed fields, ranges, units, equality, review-before-registration, and outputs | power authority passes | power review report | missing range/unit or early registration fails |
| `AC9-16` | metrology policies/checks, references, audit, and runtime projection | resolve endpoint/check keys and math; enforce provenance, independence, and projection ceiling | metrology/reference authority passes | scientific audit report | conversion, missing provenance, or scalar leakage fails |
| `AC9-17` | candidate, release, states, tags, registries, P2 result | enforce F4/F5 order, final bindings, state relations, and P2 hard gate | release authority passes only after prerequisites | release chronology report | early tag, P2 bypass, or cause mismatch fails |
| `AC9-18` | emergency/review bytes, exact paths, commit, and live remote | run the ten-step path/ancestry/tree-byte verifier | fail-closed NOT_ACTIVE path passes | emergency transcript | fallback path or byte/ancestry mismatch fails |
| `AC9-19` | command, argv, report, runtime inputs, and all authority stages | derive exact argv and replay production runner order | complete DAG constructible | DAG transcript | future-file or self-Git edge fails |
| `AC9-20` | frozen Phase-E SHA/blob and preservation fixtures | compare frozen hashes and replay all closed contracts | all preservation counters zero | preservation report | frozen mismatch or reopened contract fails |
| `AC9-21` | 15 metrics, 4/1/4/6 partition, five thresholds, evidence, and relations | recompute fixed order/source mappings/thresholds/evidence and exact 15 relations | healthy monitoring PASS and corrected trust source pass | monitoring report | wrong count/order/source/threshold/relation fails |
| `AC9-22` | campaign-abandonment incident, manifest, review, and registry record | construct incident fully before review and verify target/no reverse pointer | acyclic campaign path passes | campaign authority report | review field in incident or future target fails |
| `AC9-23` | R9 matrix owner_decision_ids | derive union and scan for any second current mapping source | exact F-OD-01..20 union passes | OD coverage report | missing/extra OD or second source fails |
| `AC9-24` | full release identity set and object checks/copies | perform exact-set equality, key de-duplication, and all copy checks | release retention PASS | release retention oracle | missing authority, duplicate same-state row, or invalid copy fails |
| `AC9-25` | complete R9 release-retention KAT fixture | independently derive expected set, compare exact identities, then verify copies | KAT PASS with declared campaign/static/monitoring/incident/resolution set | KAT transcript | any derivation or coverage discrepancy is NO-GO |

## 36. Historical R9 POSITIVE RELEASE-RETENTION CONTROL AND TEST PROCEDURES (NON-CURRENT)

### 36.1 Complete positive release-retention example

Construct a valid release `R` with manifest `M` and package objects `O1,O2`;
protocol `P`; power analysis `A`; cohort lock `C`; owner approval `OA`; execution
record `E`; release record `R`; monitoring policy `MP`; metrology policy `MET`;
the exact trust-tag message `TT`; F5 activation review bundle `RB`; initial and
latest claim states `S0,S1`; accepted PASS monitoring records `MR1,MR2`; open
incident `I1`; and incident `I2` with first contained resolution `IR2-C` followed
by terminal resolved resolution `IR2-R`. At an
`audited_at` after `MR1,MR2,IR2` are eligible, the exact expected membership is:

```text
campaign:
  authority_object(package_manifest,SHA(M))
  package_object(O1,SHA(O1))
  package_object(O2,SHA(O2))
authority:
  protocol(P), power_analysis(A), cohort_lock(C), owner_approval(OA),
  execution_record(E), release_record(R), validation_manifest,
  monitoring_policy(MP), metrology_policy(MET), git_tag_message(TT),
  independent_review_bundle(RB), claim_state(S0), claim_state(S1),
  monitoring_record(MR1), monitoring_record(MR2), incident_record(I1),
  incident_resolution(IR2-C)
  incident_resolution(IR2-R)
```

`I2` is not an unresolved-incident member, but both `IR2-C` and `IR2-R` are
retained to prove its resolved status. If `S0` and `S1` have the same kind and SHA, only one state
identity is present. For every identity, provide exactly one object check and at
least `1 + backup_copy_count` PASS copy rows; every counted row has an approved
immutable URI, exact retrieved byte length and SHA, distinct URI, and
`audited_at - verified_at < backup_verification_interval_seconds`. The exact
identity set and every copy check must PASS; a missing/extra identity, guessed
URI/length, wrong copy SHA, stale copy, or insufficient count is NO-GO.

### 36.2 Historical R9 TEST PROCEDURES

Every current R9 test has fixture construction, exact invocation, expected exact
result, and a negative mutation. The R9 counterexamples are stable review
fixtures, not production runtime or evidence artifacts.

| test ID | fixture construction | exact invocation | expected exact result | negative mutation |
|---|---|---|---|---|
| `R9-CX-01` | R9 review at X; future tag peels predecessor R8 SHA; body names R8. | R9 plan-tag validator with target, body, plan bytes, review context X | INVALID TAG / NO-GO | replace predecessor target with X and body X -> R9-CX-02 |
| `R9-CX-02` | R9 review at X; tag peels X; body.plan_review_sha=X; plan SHA/blob match X. | plan-tag validator with exact annotated tag and external CURRENT_PLAN_REVIEW_SHA=X | PASS | change body SHA, peeled target, plan SHA, or blob -> INVALID |
| `R9-CX-03` | One required authority identity is derived but no retention object check exists. | release retention exact-set and coverage validator | FAIL; missing required check | add the missing check with valid copies -> PASS |
| `R9-CX-04` | Identity kind/SHA is correct but a copy reference has a different SHA. | retention copy verifier against retrieved bytes and object identity | FAIL | replace copy SHA and bytes with the identity SHA -> PASS |
| `R9-CX-05` | F-OD-20 backup_copy_count is b but only b valid PASS copies exist. | retention object check with valid freshness and count | FAIL; requires at least 1+b | add one extra valid fresh copy -> PASS |
| `R9-CX-06` | Initial and latest state have the same kind and SHA but appear in two object_checks rows. | retention set exact-equality and key de-dup validator | FAIL; duplicate identity key | collapse the two rows to one -> PASS |
| `R9-CX-07` | Resolved incident record is present in unresolved-incident membership. | audited_at incident classifier and release-set derivation | FAIL | remove incident record but retain terminal resolution authority -> PASS |
| `R9-CX-08` | Open incident has no retention membership. | audited_at incident classifier and release-set derivation | FAIL | include the open incident identity -> PASS |
| `R9-CX-09` | Monitoring suspend record has a valid registry attestation. | accepted monitoring membership derivation | FAIL; suspend is never accepted | replace with accepted PASS record -> PASS |
| `R9-CX-10` | Accepted PASS monitoring record through audited_at is omitted. | accepted monitoring membership exact-set validator | FAIL | include its monitoring_record identity -> PASS |
| `R9-CX-11` | Resolution registry record names a predecessor SHA different from the actual previous resolution. | incident resolution chain validator | FAIL | use exact previous complete-file SHA -> PASS |
| `R9-CX-12` | Terminal resolved resolution receives a later resolution. | incident resolution chain validator | FAIL | remove later resolution or make prior status contained -> PASS |
| `R9-CX-13` | Trust source bytes are valid JSON representing a supposed trust-provisioning object. | trust monitoring source validator | reject; no JSON trust authority exists | replace with exact tag-message bytes -> PASS |
| `R9-CX-14` | Trust source bytes are exact bytes of the named annotated tag and tag validation passes. | trust tag resolver, byte hash/length verifier, and monitoring binding validator | PASS | alter one message byte -> R9-CX-15 |
| `R9-CX-15` | Source hash is correct but source bytes differ from repository tag message. | trust tag byte equality verifier | reject | restore exact repository tag-message bytes -> PASS |
| `R9-CX-16` | Tag trust_root_id differs from monitoring trust_root_id. | trust field binding comparator | binding mismatch | make both exact tag value -> PASS |
| `R9-CX-17` | Tag trust_store_sha256 equals monitoring trust_store_sha256. | trust field binding comparator | PASS | change one value -> binding mismatch |
| `R9-CX-18` | Future plan tag targets R8 SHA while R9 externally frozen review SHA differs. | plan-tag stale-target test with X and R8 SHA | INVALID TAG / NO-GO | target and body both X -> PASS |
| `R9-CX-19` | PhaseFTrustProvisioningApprovalV1 catalog category is TOP_LEVEL_WIRE. | R9 catalog category audit | catalog consistency failure | change category to TAG_BODY -> PASS |
| `R9-CX-20` | Obsolete object kind trust_provisioning_approval remains in the R9 object-kind enum. | object-kind enum/hash-table audit | plan consistency failure | remove kind and retain only git_tag_message -> PASS |
| `R9-CX-21` | Trust tag is represented only as authority_object/git_tag_message with exact message SHA. | retention and monitoring identity derivation | PASS | add a second trust approval object kind -> FAIL |
| `R9-CX-22` | Release membership attempts to infer immutable URI from a registry SHA. | release-retention derivation audit | invalid algorithm; membership must use kind/SHA only | move URI and length to copy-verification rows -> PASS |
| `R9-CX-23` | One retention identity has two valid distinct immutable copies with exact SHA/length and fresh verification. | retention copy verifier with backup policy | PASS if count/freshness policy is satisfied | make URI duplicate or stale -> FAIL |
| `R9-CX-24` | Incident has no resolution eligible at audited_at. | audited_at incident classifier | unresolved | add eligible contained resolution -> remains unresolved; add terminal resolved -> resolved |
| `R9-CX-25` | Latest eligible resolution status is contained. | audited_at incident classifier with valid chain | unresolved | change status to resolved with valid terminal resolution -> resolved |
| `R9-CX-26` | Latest eligible terminal resolution status is resolved. | audited_at incident classifier with valid chain | resolved | change status to contained -> unresolved |
| `R9-CX-27` | Resolution exists only after audited_at. | audited_at incident classifier using both effective_at and registry created_at | incident remains unresolved at audited_at | move resolution before audited_at with valid chain -> classify by it |
| `R9-CX-28` | Monitoring record is registered after audited_at. | monitoring membership derivation using registry sequence and created_at | not included | register before audited_at and satisfy acceptance -> included |
| `R9-CX-29` | Monitoring PASS is registered before audited_at, accepted, and window_end <= audited_at. | monitoring membership derivation | included | make it suspend or unaccepted -> excluded |
| `R9-CX-30` | Initial and latest claim-state complete-file SHA are identical. | retention identity key de-duplication | one retention identity | change latest SHA -> two distinct state identities |
| `T9-CAT` | Complete plan with exactly the mechanically enumerated R9 catalog. | catalog set/metadata audit over PhaseF identifiers | PASS; exact 91 rows and zero catalog counters | delete a row or blank a required cell -> FAIL |
| `T9-TRACE` | R9 matrix, catalog, AC, test, and F-EV tables with all references. | traceability resolver and owner-decision union audit | PASS; all unmapped and contradiction counters zero | add a second current OD mapping or identifier-only row -> FAIL |
| `T9-KAT-01` | Construct release R, manifest M/O1/O2, P/A/C/OA/E/R/MP/MET/TT/RB/S0/S1, MR1/MR2, I1, IR2, audited_at, and valid copies. | independent ReleaseRetentionSetV1 derivation, exact-set comparison, then copy verifier | PASS; exact expected identities and full copy coverage | mutate any identity, incident status/time, monitoring acceptance, copy SHA, URI, or count -> NO-GO |
| `T9-POS-PLAN` | Current plan review context X and positive future plan-tag fixture. | future plan-tag validator with X supplied externally | PASS | target predecessor R8 -> R9-CX-01 |
| `T9-POS-TRUST` | Exact trust tag message, source copy, and matching trust measurements. | trust tag resolver and monitoring source validator | PASS | supply JSON trust object -> R9-CX-13 |
| `T9-POS-RETENTION` | Full static/campaign/incident/monitoring identity set and 1+backup copies. | release-retention derivation and copy coverage validator | PASS | omit one member -> R9-CX-03 |
| `T9-POS-DAG` | All existing Phase-F authority fixtures in production runner order. | positive-path DAG replay and preservation validator | PASS; COMPLETE_VALID_DAG_CONSTRUCTIBLE=yes | add a future-file or self-Git edge -> NO-GO |

## 37. Historical R9 F-EV EVIDENCE ORACLES (NON-CURRENT)

Each current R9 F-EV names a real artifact, producer or authority, immutable
identity, and acceptance/review oracle. Planning-only examples describe later
artifacts; none is created by this edit.

| F-EV ID | real artifact | producer/authority | immutable identity | acceptance/review oracle |
|---|---|---|---|---|
| `EV9-01` | future plan review bundle, annotated plan tag, and plan bytes at external X | independent reviewer and Git tag validator | review-bundle SHA; peeled X; plan SHA-256 and Git blob | target/body equality and plan-byte equality |
| `EV9-02` | actual trust-provisioning annotated-tag message and trust monitoring source copies | independent trust gate, tag validator, operations authority | exact tag-message SHA and byte length; trust source reference | tag name/peel/prerequisites, exact bytes, TAG_BODY parse, trust field comparison |
| `EV9-03` | actual package manifest, manifest objects, and campaign retention audit | campaign authority and retention auditor | manifest/object complete hashes and kind/ID/SHA set | exact campaign membership, no URI/length derivation |
| `EV9-04` | actual release/cohort/execution/owner/policy/state/review authority | release authority and registry authority | exact kind/SHA identities from source bindings | static derivation table and exact-set oracle |
| `EV9-05` | actual incident and incident-resolution files plus registry records | operations/governance authority and registry authority | incident/resolution complete-file SHA and registry sequence | chain, effective_at, created_at, and audited_at classification |
| `EV9-06` | actual registered monitoring records and accepted PASS windows | operations authority and registry authority | monitoring subject complete-file SHA | accepted PASS, attestation, window_end, due-chain, and audited_at oracle |
| `EV9-07` | actual retention copy references, retrieved bytes, and retrieval/hash transcripts | retention auditor and copy retriever | each immutable URI, exact byte length, and SHA | URI scheme, availability, byte/hash equality, freshness, distinctness, count |
| `EV9-08` | complete final plan and current catalog rows | plan author and independent reviewer | final plan SHA/blob and catalog row identities | regex/set equality, metadata completeness, pointer and duplicate audit |
| `EV9-09` | current R9 requirement/AC/test/F-EV graph | plan author and independent reviewer | row bytes and referenced artifact IDs | bidirectional resolution and substance audit |
| `EV9-10` | F0 decision bundle and runtime projection | F0 authority and compatibility reviewer | decision bundle/projection complete hashes | exact 20 decisions and projection |
| `EV9-11` | two build transcripts, readiness, enrollment, signed genesis/head | checker, governance, and registry authorities | complete-file hashes, signatures, sequence | readiness/enrollment/registry validation |
| `EV9-12` | retrieved external objects, package manifest, dependency audit | retrieval/package authorities | URI/length/SHA plus complete package hashes | retrieval, classification, dependency, relation oracle |
| `EV9-13` | physical unit/location/custody ledgers and comparison/audit records | campaign, laboratory, and custody authorities | complete ledger/audit hashes and native identities | no alias, pseudoreplication, discontinuity, or post-destroy use |
| `EV9-14` | deviation revisions and event records | campaign/deviation authority | stable ledger ID and revision complete SHA | immutable predecessor and action compatibility |
| `EV9-15` | power interface, typed analysis, sensitivity cases, and review | statistician and independent scientific reviewers | content IDs and reviewed complete-file SHA | range/unit/method equality and pre-registration |
| `EV9-16` | metrology policy/checks and reference source/result/admissibility files | metrology laboratory and runtime authority | policy/result/source/audit hashes | endpoint lookup, exact math, provenance, independence, projection ceiling |
| `EV9-17` | F5 candidate, release, state chain, approval tags, and registry records | release authority and independent reviewers | candidate/release/state/tag bytes and registry hashes | F4/F5 order, P2 gate, and binding oracle |
| `EV9-18` | emergency/review files and later full-prefixed Git tree | security authority, independent reviewers, Git | emergency/review/file/tree hashes and commit SHA | exact ten-step path, ancestry, and byte equality |
| `EV9-19` | command, argv, report, runtime inputs, and construction transcript | checker and compatibility authorities | command/report and input hashes | exact runner order and no cycle |
| `EV9-20` | frozen Phase-E plan and closed-contract replay fixtures | architecture and security authorities | required frozen SHA/blob and replay transcript | preservation comparison |
| `EV9-21` | monitoring policy, thresholds, 15 measurements/evidence files, relations | F0 owner, operations, and registry authorities | policy/record/evidence/relation hashes | fixed order, 4/1/4/6, five thresholds, exact trust source |
| `EV9-22` | campaign-abandonment incident, review bundle, manifest, registry record | campaign operator, independent reviewers, registry authority | incident/review/registry complete hashes | incident-first target and no reverse pointer |
| `EV9-23` | current R9 matrix and owner-decision cells | plan author and independent reviewer | matrix bytes and plan SHA/blob | union exactly F-OD-01..20 and one mapping source |
| `EV9-24` | complete release retention audit and all object checks/copies | retention authority and independent auditor | audit complete SHA; identity and copy hashes | exact set equality, dedup, copy SHA/length/URI/freshness/count |
| `EV9-25` | R9 full positive KAT transcript and independently derived expected set | independent plan reviewer and retention auditor | KAT fixture hashes and audit transcript identity | reproduce exact campaign/static/monitoring/incident/resolution set and PASS copies |

`RELEASE_RETENTION_EVIDENCE_ORACLE_GAPS=0` and
`CURRENT_EVIDENCE_ORACLE_GAPS=0`.
## 38. Historical R9 REMEDIATION LEDGER (NON-CURRENT)

Exactly four stable R9 remediation IDs are defined. Author disposition is only
`REMEDIATED` or `OPEN`; only the fresh independent R9 reviewer may close a
finding.

| R8 P1 finding | R9 section | root cause | R9 remediation | current R9 requirements | ACs | tests | F-EV | AUTHOR DISPOSITION |
|---|---|---|---|---|---|---|---|---|
| `F-PLAN-R9-P1-01` — future plan tag targeted stale R7 commit | §§5.1, 6, 34–37 | plan tag contract named a predecessor revision and could create a self-SHA cycle | external `CURRENT_PLAN_REVIEW_SHA`; target/body/plan SHA/blob equality; stale-target counterexamples | R9-01 | AC9-01 | T9-POS-PLAN,T9-CX-01,T9-CX-02,T9-CX-18 | EV9-01 | REMEDIATED |
| `F-PLAN-R9-P1-02` — trust tag body and canonical object had dual identity | §§2, 6, 9, 14, 33, 34–37 | tag body was also described as a special trust object and monitoring source | all approvals TAG_BODY; remove special kinds; exact `git_tag_message` source and repository-byte verification | R9-02,R9-21 | AC9-02,AC9-21 | T9-POS-TRUST,T9-CX-13,T9-CX-14,T9-CX-15,T9-CX-16,T9-CX-17,T9-CX-19,T9-CX-20,T9-CX-21 | EV9-02,EV9-21 | REMEDIATED |
| `F-PLAN-R9-P1-03` — retention identities could not derive locators or audited-at incidents | §15 and §§33–37 | retention membership mixed registry identity with current storage copies and mutable incident status | byte-only kind/SHA identities; copy-verification rows; incident resolution chain; audited-at sequence/time algorithm; deduplication | R9-03,R9-04,R9-05,R9-06,R9-07,R9-24,R9-25 | AC9-03,AC9-04,AC9-05,AC9-06,AC9-07,AC9-24,AC9-25 | T9-KAT-01,T9-POS-RETENTION,T9-CX-03,T9-CX-04,T9-CX-05,T9-CX-06,T9-CX-07,T9-CX-08,T9-CX-09,T9-CX-10,T9-CX-11,T9-CX-12,T9-CX-22,T9-CX-23,T9-CX-24,T9-CX-25,T9-CX-26,T9-CX-27,T9-CX-28,T9-CX-29,T9-CX-30 | EV9-03,EV9-04,EV9-05,EV9-06,EV9-07,EV9-24,EV9-25 | REMEDIATED |
| `F-PLAN-R9-P1-04` — retention tests/evidence did not exercise release authority contract | §§34–37 | R8 coverage tested only campaign manifest equality | separate static, incident, monitoring, and copy ACs/tests/F-EV plus positive release KAT and negative controls | R9-04,R9-05,R9-06,R9-07,R9-24,R9-25 | AC9-04,AC9-05,AC9-06,AC9-07,AC9-24,AC9-25 | T9-KAT-01,T9-POS-RETENTION,T9-CX-03,T9-CX-04,T9-CX-05,T9-CX-06,T9-CX-07,T9-CX-08,T9-CX-09,T9-CX-10,T9-CX-11,T9-CX-12,T9-CX-22,T9-CX-23,T9-CX-24,T9-CX-25,T9-CX-26,T9-CX-27,T9-CX-28,T9-CX-29,T9-CX-30 | EV9-04,EV9-05,EV9-06,EV9-07,EV9-24,EV9-25 | REMEDIATED |

## 39. Historical R9 AUTHOR AUDIT AND FRESH REREVIEW GATE (NON-CURRENT)

The author audit is not independent approval. The following counters are the
required R9 audit result:

PLAN_TAG_STALE_TARGET_PATHS=0
PLAN_TAG_TARGET_AMBIGUITIES=0
TAG_BODY_VS_OBJECT_IDENTITY_AMBIGUITIES=0
TRUST_MONITORING_SOURCE_AMBIGUITIES=0
SCHEMA_CATALOG_SECTION_POINTER_ERRORS=0
RETENTION_IDENTITY_LOCATOR_AMBIGUITIES=0
RELEASE_RETENTION_STATIC_IDENTITY_AMBIGUITIES=0
INCIDENT_AUDIT_TIME_STATUS_AMBIGUITIES=0
RELEASE_RETENTION_DEDUP_AMBIGUITIES=0
RELEASE_RETENTION_REFERENCE_DERIVATION_AMBIGUITIES=0
RELEASE_RETENTION_TEST_COVERAGE_GAPS=0
RELEASE_RETENTION_EVIDENCE_ORACLE_GAPS=0
APPROVAL_TAG_SCHEMA_CATEGORY_AMBIGUITIES=0
OBJECT_KIND_HASH_TABLE_AMBIGUITIES=0
ORPHAN_EXTERNAL_SCHEMAS=0
INCOMPLETE_SCHEMA_CATALOG_ROWS=0
CATALOG_DUPLICATE_IDENTIFIER_ROWS=0
CATALOG_TO_REQUIREMENT_GAPS=0
REQUIREMENT_TO_CATALOG_GAPS=0
UNMAPPED_REQUIREMENTS=0
UNMAPPED_ACS=0
UNMAPPED_TESTS=0
UNMAPPED_EVIDENCE=0
UNMAPPED_ODS=0
TRACEABILITY_SUBSTANCE_GAPS=0
CURRENT_TEST_PROCEDURE_GAPS=0
CURRENT_EVIDENCE_ORACLE_GAPS=0
CONTRADICTORY_CURRENT_TRACEABILITY_TABLES=0
NORMATIVE_CONTRADICTIONS=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES=0

Previously closed safety, scientific, identity, custody, DAG, runtime, and
production-order counters remain zero. The four R9 findings are not marked
CLOSED. A fresh independent reviewer must begin with the four positive controls:
plan tag target/body at external X; one exact trust tag message; deterministic
kind/SHA release retention with audited-at status; and substantive split
traceability for static, incident, monitoring, and copy coverage. Failure of any
control is P1. R9 independent rereview remains PENDING.

## 40. Historical R9 BASELINE VALIDATION AND FROZEN PHASE-E AUTHORITY (NON-CURRENT)

Before and after the planning-only edit, the required commands are:

```text
git diff --check
cargo fmt --all --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --test phase_e_validation
cargo test --locked --test phase_d_reporting_public_output
```

Required result is all PASS: Phase E 38/38, Phase D 73/73, and zero strict
Clippy diagnostics. The frozen Phase-E plan remains unchanged and must retain
SHA-256 `0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`
and Git blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

## 41. Historical MHI V1 PHASE F R9 PLANNING REMEDIATION HANDOFF (NON-CURRENT)

STARTING R8 SHA:
`ebed7b24102b1575fb950a8d15adc03c2eb11c22`

R8 PLAN SHA-256:
`043e7811a2e93bb88912b483334bcac82a8b6cabd8206531945889678216e934`

R8 PLAN BLOB:
`1aed756793cf42c284d41e96ce2d8247a250e93f`

R9 PLAN REVIEW SHA:
`<externally frozen after final planning-only commit>`

R9 PLAN SHA-256:
`<computed after final R9 bytes>`

R9 PLAN GIT BLOB:
`<computed after final R9 bytes>`

CHANGED FILES:
1 expected

R8 P1 REMEDIATION

F-PLAN-R9-P1-01:
REMEDIATED

F-PLAN-R9-P1-02:
REMEDIATED

F-PLAN-R9-P1-03:
REMEDIATED

F-PLAN-R9-P1-04:
REMEDIATED

PLAN TAG

stale revision target:
0 expected

current review SHA externally frozen:
yes after final commit; not embedded

target/body equality rule:
COMPLETE

TRUST

trust approval category:
TAG_BODY expected

special trust approval object kind:
ABSENT expected

trust monitoring source:
trust_provisioning_tag_message expected

second trust object:
NO expected

trust monitoring binding:
COMPLETE

RETENTION

membership uses kind/SHA only:
yes

copy verification carries URI/length:
yes

campaign set:
COMPLETE

static release set:
COMPLETE

accepted monitoring membership:
COMPLETE

incident resolution schema:
COMPLETE

audited_at incident classification:
COMPLETE

initial/latest dedup:
COMPLETE

copy count/freshness:
COMPLETE

RELEASE RETENTION TESTING

static-set tests:
COMPLETE

state-dedup test:
COMPLETE

monitoring-membership tests:
COMPLETE

incident-status tests:
COMPLETE

copy-reference tests:
COMPLETE

trust-tag retention test:
COMPLETE

CATALOG

normative identifiers:
91

catalog identifiers:
91

missing:
0

extra:
0

duplicates:
0

incomplete metadata:
0

enrollment closure pointer:
CORRECT

all approval schemas TAG_BODY:
6/6

TRACEABILITY

current R9 requirements:
25

ACs:
25

tests:
37

evidence:
25

owner decisions:
20

schemas:
91

unmapped requirements:
0

unmapped ACs:
0

unmapped tests:
0

unmapped evidence:
0

unmapped ODs:
0

traceability substance gaps:
0

test-procedure gaps:
0

evidence-oracle gaps:
0

POSITIVE PATH

complete DAG constructible:
yes

construction ambiguities:
0

AUTHOR AUDIT

PLAN_TAG_STALE_TARGET_PATHS=0
PLAN_TAG_TARGET_AMBIGUITIES=0
TAG_BODY_VS_OBJECT_IDENTITY_AMBIGUITIES=0
TRUST_MONITORING_SOURCE_AMBIGUITIES=0
SCHEMA_CATALOG_SECTION_POINTER_ERRORS=0
RETENTION_IDENTITY_LOCATOR_AMBIGUITIES=0
RELEASE_RETENTION_STATIC_IDENTITY_AMBIGUITIES=0
INCIDENT_AUDIT_TIME_STATUS_AMBIGUITIES=0
RELEASE_RETENTION_DEDUP_AMBIGUITIES=0
RELEASE_RETENTION_REFERENCE_DERIVATION_AMBIGUITIES=0
RELEASE_RETENTION_TEST_COVERAGE_GAPS=0
RELEASE_RETENTION_EVIDENCE_ORACLE_GAPS=0
APPROVAL_TAG_SCHEMA_CATEGORY_AMBIGUITIES=0
OBJECT_KIND_HASH_TABLE_AMBIGUITIES=0
ORPHAN_EXTERNAL_SCHEMAS=0
INCOMPLETE_SCHEMA_CATALOG_ROWS=0
CATALOG_DUPLICATE_IDENTIFIER_ROWS=0
CATALOG_TO_REQUIREMENT_GAPS=0
REQUIREMENT_TO_CATALOG_GAPS=0
UNMAPPED_REQUIREMENTS=0
UNMAPPED_ACS=0
UNMAPPED_TESTS=0
UNMAPPED_EVIDENCE=0
UNMAPPED_ODS=0
TRACEABILITY_SUBSTANCE_GAPS=0
CURRENT_TEST_PROCEDURE_GAPS=0
CURRENT_EVIDENCE_ORACLE_GAPS=0
CONTRADICTORY_CURRENT_TRACEABILITY_TABLES=0
NORMATIVE_CONTRADICTIONS=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES=0

BASELINE

diff:
PASS

fmt:
PASS

check:
PASS

strict Clippy:
PASS

Clippy diagnostics:
0

Phase E:
38/38

Phase D:
73/73

FROZEN PHASE-E PLAN:
PASS

PLAN TAG CREATED:
NO expected

IMPLEMENTATION BRANCH:
NO expected

F0 STARTED:
NO expected

KEYS CREATED:
NO expected

EVIDENCE CREATED:
NO expected

CLAIMS CREATED:
NO expected

WORKTREE CLEAN:
yes after one forward commit

READY_FOR_FRESH_PHASE_F_R9_PLAN_REREVIEW:
yes

READY_FOR_PHASE_F_PLAN_APPROVAL_TAG:
NO expected pending fresh R9 GO

READY_FOR_PHASE_F_IMPLEMENTATION:
NO

## 42. Historical R10 MASTER SCHEMA CATALOG AND FIXTURE AUTHORITY (NON-CURRENT)

Sections 33-52 are retained historical R9/R10 appendices and are excluded from
current extraction. The current authority begins at §53 and is R11. R10 remained
planning only. All fixtures below are historical plan-embedded KAT definitions,
not production evidence, schema files, registry records, physical evidence,
monitoring evidence, claims, keys, signatures, or tags.

R10 preserved every closed R9 contract and changed no authority architecture or
scientific scope. R1 through R8 were NO-GO with P1 counts 13, 10, 19, 14, 11,
13, 5, and 4. R9 was NO-GO with six grouped fixture/catalog findings. R10 is
forward remediation; the independent R10 rereview was PENDING and is now the
starting authority for R11.

### 42.1 Historical R10 normative identifier set

Mechanical enumeration of PhaseF[A-Za-z0-9_]*V1 produces exactly 91
identifiers. The current master set is:

~~~text
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
PhaseFIncidentResolutionV1
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
PhaseFRetentionCopyVerificationV1
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
~~~

Each identifier occurs exactly once in the current catalog below. Historical
catalog rows are not current rows.

### 42.2 Historical R10 master schema catalog

Category rules are exact: TOP_LEVEL_WIRE is an independently serialized or
materialized file with a complete byte representation; NESTED_WIRE is only a
member/value inside the named parent and is never an independent authority;
TAG_BODY is the annotated-tag message grammar; PLAN_ONLY_CONTRACT has no
standalone wire artifact. The three confirmed corrections are
PhaseFChainOfCustodyV1, PhaseFCheckerReportV1, and PhaseFRetrievalVerificationV1
to TOP_LEVEL_WIRE.

| exact identifier | category | exact field-closure pointer | semantic-ID rule | complete-file hash meaning | concrete producer | actual validator | exact stage/set | exact registry behavior | derived current requirement IDs | derived current AC IDs | derived current test IDs | derived current F-EV IDs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| PhaseFArgvV1 | NESTED_WIRE | §2 PhaseFArgvV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F checker build/readiness operation | Phase-F schema strict parser plus field/type consistency validator | readiness and checker-invocation stages | nested field of PhaseFCommandV1 and PhaseFCheckerReportV1; no independent registry subject | R10-08,R10-09,R10-10,R10-20 | AC10-08,AC10-09,AC10-10,AC10-20 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-20 |
| PhaseFAuthorityEnrollmentApprovalV1 | TAG_BODY | §2 PhaseFAuthorityEnrollmentApprovalV1 exact primitive/helper definition | no JSON semantic ID; annotated-tag message identity only | SHA-256 of exact annotated-tag message bytes when referenced | independent enrollment gate | annotated-tag grammar plus target/body/prerequisite validator | enrollment approval gate | annotated-tag message only; no registry subject | R10-08,R10-09,R10-10 | AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-TRACE | EV10-08,EV10-09,EV10-10 |
| PhaseFAuthorityEnrollmentV1 | TOP_LEVEL_WIRE | §5.2 PhaseFAuthorityEnrollmentV1 exact unsigned enrollment fields | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFAuthorityEnrollmentV1 bytes | authority-enrollment operation | enrollment strict parser plus identity/field validator | enrollment stage | independent registry subject; exact object kind authority_enrollment | R10-08,R10-09,R10-10,R10-12 | AC10-08,AC10-09,AC10-10,AC10-12 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-12 |
| PhaseFChainOfCustodyV1 | TOP_LEVEL_WIRE | §11 PhaseFChainOfCustodyV1 exact physical/deviation definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFChainOfCustodyV1 bytes | campaign laboratory and custody authority | custody strict parser plus continuity/terminal-unit validator | F2-F4 physical-validation stages | independent registry subject; exact object kind chain_of_custody | R10-08,R10-09,R10-10,R10-14 | AC10-08,AC10-09,AC10-10,AC10-14 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-14 |
| PhaseFCheckListV1 | NESTED_WIRE | §2 PhaseFCheckListV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | nested field of PhaseFMetrologyPolicyV1; no independent registry subject | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFCheckerBuildEvidenceV1 | TOP_LEVEL_WIRE | §7 PhaseFCheckerBuildEvidenceV1 exact checker definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFCheckerBuildEvidenceV1 bytes | Phase-F checker build/readiness operation | Phase-F schema strict parser plus field/type consistency validator | readiness and checker-invocation stages | independent registry subject; exact object kind checker_build_evidence | R10-08,R10-09,R10-10,R10-12 | AC10-08,AC10-09,AC10-10,AC10-12 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-12 |
| PhaseFCheckerExitCodeV1 | NESTED_WIRE | §2 PhaseFCheckerExitCodeV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F checker build/readiness operation | Phase-F schema strict parser plus field/type consistency validator | readiness and checker-invocation stages | nested field of PhaseFCheckerReportV1; no independent registry subject | R10-08,R10-09,R10-10,R10-20 | AC10-08,AC10-09,AC10-10,AC10-20 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-20 |
| PhaseFCheckerReadinessEvidenceV1 | TOP_LEVEL_WIRE | §7 PhaseFCheckerReadinessEvidenceV1 exact checker definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFCheckerReadinessEvidenceV1 bytes | Phase-F checker build/readiness operation | Phase-F schema strict parser plus field/type consistency validator | readiness and checker-invocation stages | independent registry subject; exact object kind checker_readiness_evidence | R10-08,R10-09,R10-10,R10-12 | AC10-08,AC10-09,AC10-10,AC10-12 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-12 |
| PhaseFCheckerReportV1 | TOP_LEVEL_WIRE | §7 PhaseFCheckerReportV1 exact checker definition | no content-derived semantic ID; explicit report operation identity | SHA-256 of complete canonical PhaseFCheckerReportV1 bytes | Phase-F checker build/readiness operation | checker-report strict parser plus command/result consistency validator | readiness and checker-invocation stages | standalone evidence file at explicit output path; not registered | R10-08,R10-09,R10-10,R10-20 | AC10-08,AC10-09,AC10-10,AC10-20 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-20 |
| PhaseFCheckerStdoutV1 | NESTED_WIRE | §2 PhaseFCheckerStdoutV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F checker build/readiness operation | Phase-F schema strict parser plus field/type consistency validator | readiness and checker-invocation stages | nested field of PhaseFCheckerReportV1; no independent registry subject | R10-08,R10-09,R10-10,R10-20 | AC10-08,AC10-09,AC10-10,AC10-20 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-20 |
| PhaseFClaimStateRecordV1 | TOP_LEVEL_WIRE | §14 PhaseFClaimStateRecordV1 exact release/monitoring definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFClaimStateRecordV1 bytes | release authority | Phase-F schema strict parser plus field/type consistency validator | F4-F5 release/state stages | independent registry subject; exact object kind claim_state | R10-04,R10-08,R10-09,R10-10,R10-18,R10-24,R10-25 | AC10-04,AC10-08,AC10-09,AC10-10,AC10-18,AC10-24,AC10-25 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31,T10-KAT-RETENTION,T10-POS-DAG,T10-POS-RETENTION,T10-TRACE | EV10-04,EV10-08,EV10-09,EV10-10,EV10-18,EV10-24,EV10-25 |
| PhaseFCohortLockRecordV1 | TOP_LEVEL_WIRE | §14 PhaseFCohortLockRecordV1 exact release/monitoring definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFCohortLockRecordV1 bytes | release authority | Phase-F schema strict parser plus field/type consistency validator | F4-F5 release/state stages | independent registry subject; exact object kind cohort_lock | R10-04,R10-08,R10-09,R10-10 | AC10-04,AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-CX-21,T10-POS-RETENTION,T10-TRACE | EV10-04,EV10-08,EV10-09,EV10-10 |
| PhaseFCommandV1 | NESTED_WIRE | §2 PhaseFCommandV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F checker build/readiness operation | command grammar and command/result consistency validator | readiness and checker-invocation stages | nested field of PhaseFCheckerReadinessEvidenceV1 and PhaseFCheckerReportV1; no independent registry subject | R10-08,R10-09,R10-10,R10-20 | AC10-08,AC10-09,AC10-10,AC10-20 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-20 |
| PhaseFCustodyEventV1 | NESTED_WIRE | §11 PhaseFCustodyEventV1 exact physical/deviation definition | no independent identity; parent bytes only | containing-object canonical bytes | campaign laboratory and custody authority | physical/custody strict parser plus identity/continuity validator | F2-F4 physical-validation stages | nested field of PhaseFChainOfCustodyV1; no independent registry subject | R10-08,R10-09,R10-10,R10-14 | AC10-08,AC10-09,AC10-10,AC10-14 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-14 |
| PhaseFDecisionApprovalV1 | TAG_BODY | §4 PhaseFDecisionApprovalV1 exact decision definition | no JSON semantic ID; annotated-tag message identity only | SHA-256 of exact annotated-tag message bytes when referenced | independent F0 decision gate | annotated-tag grammar plus target/body/prerequisite validator | F0 approval gate | annotated-tag message only; no registry subject | R10-08,R10-09,R10-10 | AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-TRACE | EV10-08,EV10-09,EV10-10 |
| PhaseFDecisionBundleV1 | TOP_LEVEL_WIRE | §4 PhaseFDecisionBundleV1 exact decision definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFDecisionBundleV1 bytes | F0 decision authority | Phase-F schema strict parser plus field/type consistency validator | F0 decision-bundle construction | independent registry subject; exact object kind decision_bundle | R10-08,R10-09,R10-10,R10-11,R10-23 | AC10-08,AC10-09,AC10-10,AC10-11,AC10-23 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-11,EV10-23 |
| PhaseFDecisionRowV1 | NESTED_WIRE | §4 PhaseFDecisionRowV1 exact decision definition | no independent identity; parent bytes only | containing-object canonical bytes | F0 decision authority | Phase-F schema strict parser plus field/type consistency validator | F0 decision-bundle construction | nested field of PhaseFDecisionBundleV1; no independent registry subject | R10-08,R10-09,R10-10,R10-11,R10-23 | AC10-08,AC10-09,AC10-10,AC10-11,AC10-23 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-11,EV10-23 |
| PhaseFDecisionValueV1 | NESTED_WIRE | §2 PhaseFDecisionValueV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | F0 decision authority | Phase-F schema strict parser plus field/type consistency validator | F0 decision-bundle construction | nested field of PhaseFDecisionRowV1; no independent registry subject | R10-08,R10-09,R10-10,R10-11 | AC10-08,AC10-09,AC10-10,AC10-11 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-11 |
| PhaseFDependencyAuditV1 | TOP_LEVEL_WIRE | §10 PhaseFDependencyAuditV1 exact retrieval/package definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFDependencyAuditV1 bytes | retrieval/package authority | retrieval/package strict parser plus classification/DAG validator | F2 retrieval/package stage | independent registry subject; exact object kind dependency_audit | R10-08,R10-09,R10-10,R10-13 | AC10-08,AC10-09,AC10-10,AC10-13 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-13 |
| PhaseFDependencyEdgeV1 | NESTED_WIRE | §10 PhaseFDependencyEdgeV1 exact retrieval/package definition | no independent identity; parent bytes only | containing-object canonical bytes | retrieval/package authority | retrieval/package strict parser plus classification/DAG validator | F2 retrieval/package stage | nested field of PhaseFDependencyAuditV1; no independent registry subject | R10-08,R10-09,R10-10,R10-13 | AC10-08,AC10-09,AC10-10,AC10-13 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-13 |
| PhaseFDeviationEventV1 | NESTED_WIRE | §11 PhaseFDeviationEventV1 exact physical/deviation definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F plan authority | Phase-F schema strict parser plus field/type consistency validator | F1-F4 deviation stages | nested field of PhaseFDeviationLedgerRevisionV1; no independent registry subject | R10-08,R10-09,R10-10,R10-15 | AC10-08,AC10-09,AC10-10,AC10-15 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-15 |
| PhaseFDeviationLedgerRevisionV1 | TOP_LEVEL_WIRE | §11 PhaseFDeviationLedgerRevisionV1 exact physical/deviation definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFDeviationLedgerRevisionV1 bytes | Phase-F plan authority | Phase-F schema strict parser plus field/type consistency validator | F1-F4 deviation stages | independent registry subject; exact object kind deviation_ledger | R10-08,R10-09,R10-10,R10-15 | AC10-08,AC10-09,AC10-10,AC10-15 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-15 |
| PhaseFDeviationLedgerV1 | PLAN_ONLY_CONTRACT | §11 PhaseFDeviationLedgerV1 exact physical/deviation definition | no artifact identity; planning construct only | not applicable; no standalone bytes | plan author | plan consistency validator; no runtime artifact validator | F1-F4 deviation stages | plan-only contract; no standalone artifact or registry subject | R10-08,R10-09,R10-10,R10-15 | AC10-08,AC10-09,AC10-10,AC10-15 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-15 |
| PhaseFEndpointMetrologyPolicyV1 | NESTED_WIRE | §13 PhaseFEndpointMetrologyPolicyV1 exact metrology/reference definition | no independent identity; parent bytes only | containing-object canonical bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | nested field of the exact parent field named by §13 PhaseFEndpointMetrologyPolicyV1 exact metrology/reference definition; no independent registry subject | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFEnvironmentEntryV1 | NESTED_WIRE | §2 PhaseFEnvironmentEntryV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F plan authority | Phase-F schema strict parser plus field/type consistency validator | the exact parent operation named in this row | nested field of PhaseFCheckerBuildEvidenceV1 and PhaseFCheckerReadinessEvidenceV1; no independent registry subject | R10-08,R10-09,R10-10 | AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-TRACE | EV10-08,EV10-09,EV10-10 |
| PhaseFExecutionRecordV1 | TOP_LEVEL_WIRE | §14 PhaseFExecutionRecordV1 exact release/monitoring definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFExecutionRecordV1 bytes | release authority | Phase-F schema strict parser plus field/type consistency validator | F4-F5 release/state stages | independent registry subject; exact object kind execution_record | R10-04,R10-08,R10-09,R10-10 | AC10-04,AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-CX-21,T10-POS-RETENTION,T10-TRACE | EV10-04,EV10-08,EV10-09,EV10-10 |
| PhaseFF5ReleaseCandidateV1 | TOP_LEVEL_WIRE | §5 PhaseFF5ReleaseCandidateV1 exact review definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFF5ReleaseCandidateV1 bytes | independent review panel | Phase-F schema strict parser plus field/type consistency validator | F4-F5 release/state stages | independent registry subject; exact object kind f5_release_candidate | R10-08,R10-09,R10-10,R10-18 | AC10-08,AC10-09,AC10-10,AC10-18 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-18 |
| PhaseFIdentityComparisonV1 | NESTED_WIRE | §11 PhaseFIdentityComparisonV1 exact physical/deviation definition | no independent identity; parent bytes only | containing-object canonical bytes | campaign laboratory and custody authority | physical/custody strict parser plus identity/continuity validator | F2-F4 physical-validation stages | nested field of PhaseFPhysicalIdentityAuditV1; no independent registry subject | R10-08,R10-09,R10-10,R10-14 | AC10-08,AC10-09,AC10-10,AC10-14 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-14 |
| PhaseFIncidentRecordV1 | TOP_LEVEL_WIRE | §15 PhaseFIncidentRecordV1 exact incident/retention definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFIncidentRecordV1 bytes | operations/governance incident-resolution authority | incident strict parser plus chronology/status validator | operations after incident detection through F5+ audited retention | independent registry subject; exact object kind incident_record | R10-05,R10-08,R10-09,R10-10,R10-22,R10-24,R10-25 | AC10-05,AC10-08,AC10-09,AC10-10,AC10-22,AC10-24,AC10-25 | T10-CAT,T10-CX-01,T10-CX-02,T10-CX-03,T10-CX-04,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31,T10-KAT-RETENTION,T10-POS-DAG,T10-POS-RETENTION,T10-TRACE | EV10-05,EV10-08,EV10-09,EV10-10,EV10-22,EV10-24,EV10-25 |
| PhaseFIncidentResolutionV1 | TOP_LEVEL_WIRE | §15 PhaseFIncidentResolutionV1 exact incident/retention definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFIncidentResolutionV1 bytes | operations/governance incident-resolution authority | incident-resolution strict parser plus resolution-chain validator | operations after incident detection through F5+ audited retention | independent registry subject; exact object kind incident_resolution | R10-05,R10-08,R10-09,R10-10,R10-24,R10-25 | AC10-05,AC10-08,AC10-09,AC10-10,AC10-24,AC10-25 | T10-CAT,T10-CX-01,T10-CX-02,T10-CX-03,T10-CX-04,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31,T10-KAT-RETENTION,T10-POS-RETENTION,T10-TRACE | EV10-05,EV10-08,EV10-09,EV10-10,EV10-24,EV10-25 |
| PhaseFIncidentScopeV1 | NESTED_WIRE | §2 PhaseFIncidentScopeV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | operations/governance incident-resolution authority | incident strict parser plus chronology/status validator | operations after incident detection through F5+ audited retention | nested field of PhaseFIncidentRecordV1; no independent registry subject | R10-08,R10-09,R10-10,R10-18,R10-22 | AC10-08,AC10-09,AC10-10,AC10-18,AC10-22 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-18,EV10-22 |
| PhaseFIndependentReviewBundleV1 | TOP_LEVEL_WIRE | §5 PhaseFIndependentReviewBundleV1 exact review definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFIndependentReviewBundleV1 bytes | independent review panel | Phase-F schema strict parser plus field/type consistency validator | the exact parent operation named in this row | independent registry subject; exact object kind independent_review_bundle | R10-01,R10-04,R10-08,R10-09,R10-10,R10-19,R10-22 | AC10-01,AC10-04,AC10-08,AC10-09,AC10-10,AC10-19,AC10-22 | T10-CAT,T10-CX-01,T10-CX-02,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-CX-21,T10-POS-DAG,T10-POS-PLAN,T10-POS-RETENTION,T10-TRACE | EV10-01,EV10-04,EV10-08,EV10-09,EV10-10,EV10-19,EV10-22 |
| PhaseFIndependentReviewV1 | NESTED_WIRE | §5 PhaseFIndependentReviewV1 exact review definition | no independent identity; parent bytes only | containing-object canonical bytes | independent review panel | Phase-F schema strict parser plus field/type consistency validator | the exact parent operation named in this row | nested field of PhaseFIndependentReviewBundleV1; no independent registry subject | R10-08,R10-09,R10-10 | AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-TRACE | EV10-08,EV10-09,EV10-10 |
| PhaseFLODLOQPolicyV1 | NESTED_WIRE | §2 PhaseFLODLOQPolicyV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | nested field of PhaseFMetrologyPolicyV1; no independent registry subject | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFLocationLedgerV1 | TOP_LEVEL_WIRE | §11 PhaseFLocationLedgerV1 exact physical/deviation definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFLocationLedgerV1 bytes | campaign laboratory and custody authority | physical/custody strict parser plus identity/continuity validator | F2-F4 physical-validation stages | independent registry subject; exact object kind location_ledger | R10-08,R10-09,R10-10,R10-14 | AC10-08,AC10-09,AC10-10,AC10-14 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-14 |
| PhaseFLocationV1 | NESTED_WIRE | §11 PhaseFLocationV1 exact physical/deviation definition | no independent identity; parent bytes only | containing-object canonical bytes | campaign laboratory and custody authority | physical/custody strict parser plus identity/continuity validator | F2-F4 physical-validation stages | nested field of PhaseFLocationLedgerV1; no independent registry subject | R10-08,R10-09,R10-10,R10-14 | AC10-08,AC10-09,AC10-10,AC10-14 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-14 |
| PhaseFMethodVersionV1 | NESTED_WIRE | §12 PhaseFMethodVersionV1 exact power definition | no independent identity; parent bytes only | containing-object canonical bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | nested field of PhaseFPowerMethodInterfaceV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFMetricThresholdV1 | NESTED_WIRE | §14 PhaseFMetricThresholdV1 exact release/monitoring definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F plan authority | Phase-F schema strict parser plus field/type consistency validator | F0 policy and F5+ monitoring stages | nested field of PhaseFMonitoringPolicyV1; no independent registry subject | R10-08,R10-09,R10-10,R10-21 | AC10-08,AC10-09,AC10-10,AC10-21 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-POS-TRUST,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-21 |
| PhaseFMetrologyCheckResultV1 | TOP_LEVEL_WIRE | §13 PhaseFMetrologyCheckResultV1 exact metrology/reference definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFMetrologyCheckResultV1 bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | independent registry subject; exact object kind metrology_check_result | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFMetrologyCheckSpecV1 | NESTED_WIRE | §13 PhaseFMetrologyCheckSpecV1 exact metrology/reference definition | no independent identity; parent bytes only | containing-object canonical bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | nested field of PhaseFMetrologyPolicyV1; no independent registry subject | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFMetrologyPolicyV1 | TOP_LEVEL_WIRE | §13 PhaseFMetrologyPolicyV1 exact metrology/reference definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFMetrologyPolicyV1 bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | independent registry subject; exact object kind metrology_policy | R10-04,R10-08,R10-09,R10-10,R10-17 | AC10-04,AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-CX-21,T10-POS-DAG,T10-POS-RETENTION,T10-TRACE | EV10-04,EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFMonitoringBreachV1 | NESTED_WIRE | §14 PhaseFMonitoringBreachV1 exact release/monitoring definition | no independent identity; parent bytes only | containing-object canonical bytes | operations monitoring authority | monitoring strict parser plus fixed-order/acceptance validator | F0 policy and F5+ monitoring stages | nested field of PhaseFMonitoringRecordV1; no independent registry subject | R10-08,R10-09,R10-10,R10-21 | AC10-08,AC10-09,AC10-10,AC10-21 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-POS-TRUST,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-21 |
| PhaseFMonitoringEvidenceV1 | TOP_LEVEL_WIRE | §14 PhaseFMonitoringEvidenceV1 exact release/monitoring definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFMonitoringEvidenceV1 bytes | operations monitoring authority | monitoring strict parser plus fixed-order/acceptance validator | F0 policy and F5+ monitoring stages | independent registry subject; exact object kind monitoring_evidence | R10-02,R10-06,R10-08,R10-09,R10-10,R10-21 | AC10-02,AC10-06,AC10-08,AC10-09,AC10-10,AC10-21 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-19,T10-CX-20,T10-CX-22,T10-CX-23,T10-POS-DAG,T10-POS-TRUST,T10-TRACE | EV10-02,EV10-06,EV10-08,EV10-09,EV10-10,EV10-21 |
| PhaseFMonitoringMeasurementV1 | NESTED_WIRE | §14 PhaseFMonitoringMeasurementV1 exact release/monitoring definition | no independent identity; parent bytes only | containing-object canonical bytes | operations monitoring authority | monitoring strict parser plus fixed-order/acceptance validator | F0 policy and F5+ monitoring stages | nested field of PhaseFMonitoringRecordV1; no independent registry subject | R10-08,R10-09,R10-10,R10-21 | AC10-08,AC10-09,AC10-10,AC10-21 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-POS-TRUST,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-21 |
| PhaseFMonitoringPolicyV1 | TOP_LEVEL_WIRE | §14 PhaseFMonitoringPolicyV1 exact release/monitoring definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFMonitoringPolicyV1 bytes | operations monitoring authority | monitoring strict parser plus fixed-order/acceptance validator | F0 policy and F5+ monitoring stages | independent registry subject; exact object kind monitoring_policy | R10-04,R10-08,R10-09,R10-10,R10-21 | AC10-04,AC10-08,AC10-09,AC10-10,AC10-21 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-CX-21,T10-POS-DAG,T10-POS-RETENTION,T10-POS-TRUST,T10-TRACE | EV10-04,EV10-08,EV10-09,EV10-10,EV10-21 |
| PhaseFMonitoringRecordV1 | TOP_LEVEL_WIRE | §14 PhaseFMonitoringRecordV1 exact release/monitoring definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFMonitoringRecordV1 bytes | operations monitoring authority | monitoring strict parser plus fixed-order/acceptance validator | F0 policy and F5+ monitoring stages | independent registry subject; exact object kind monitoring_record | R10-06,R10-08,R10-09,R10-10,R10-21,R10-24,R10-25 | AC10-06,AC10-08,AC10-09,AC10-10,AC10-21,AC10-24,AC10-25 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31,T10-KAT-RETENTION,T10-POS-DAG,T10-POS-RETENTION,T10-POS-TRUST,T10-TRACE | EV10-06,EV10-08,EV10-09,EV10-10,EV10-21,EV10-24,EV10-25 |
| PhaseFMonitoringSourceReferenceV1 | NESTED_WIRE | §13 PhaseFMonitoringSourceReferenceV1 exact metrology/reference definition | no independent identity; parent bytes only | containing-object canonical bytes | operations monitoring authority | monitoring strict parser plus fixed-order/acceptance validator | F0-F2 metrology/reference stages | nested field of PhaseFMonitoringEvidenceV1; no independent registry subject | R10-02,R10-08,R10-09,R10-10,R10-21 | AC10-02,AC10-08,AC10-09,AC10-10,AC10-21 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-POS-TRUST,T10-TRACE | EV10-02,EV10-08,EV10-09,EV10-10,EV10-21 |
| PhaseFMonitoringValueV1 | NESTED_WIRE | §2 PhaseFMonitoringValueV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | operations monitoring authority | monitoring strict parser plus fixed-order/acceptance validator | F0 policy and F5+ monitoring stages | nested field of PhaseFMonitoringMeasurementV1; no independent registry subject | R10-08,R10-09,R10-10,R10-21 | AC10-08,AC10-09,AC10-10,AC10-21 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-POS-TRUST,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-21 |
| PhaseFNamedDigestV1 | NESTED_WIRE | §2 PhaseFNamedDigestV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F plan authority | Phase-F schema strict parser plus field/type consistency validator | the exact parent operation named in this row | nested field of PhaseFObjectDigestV1 and PhaseFIncidentRecordV1; no independent registry subject | R10-08,R10-09,R10-10 | AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-TRACE | EV10-08,EV10-09,EV10-10 |
| PhaseFObjectDigestV1 | NESTED_WIRE | §2 PhaseFObjectDigestV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | Phase-F plan authority | Phase-F schema strict parser plus field/type consistency validator | the exact parent operation named in this row | nested field of PhaseFIncidentRecordV1; no independent registry subject | R10-08,R10-09,R10-10 | AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-TRACE | EV10-08,EV10-09,EV10-10 |
| PhaseFObjectReferenceV1 | NESTED_WIRE | §10 PhaseFObjectReferenceV1 exact retrieval/package definition | no independent identity; parent bytes only | containing-object canonical bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | nested field of PhaseFRetrievalVerificationV1, PhaseFPackageManifestV1, PhaseFRetentionCopyVerificationV1, and PhaseFMonitoringEvidenceV1; no independent registry subject | R10-02,R10-07,R10-08,R10-09,R10-10,R10-19,R10-25 | AC10-02,AC10-07,AC10-08,AC10-09,AC10-10,AC10-19,AC10-25 | T10-CAT,T10-CX-05,T10-CX-06,T10-CX-07,T10-CX-08,T10-CX-09,T10-CX-10,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-30,T10-CX-31,T10-KAT-RETENTION,T10-POS-DAG,T10-POS-TRUST,T10-TRACE | EV10-02,EV10-07,EV10-08,EV10-09,EV10-10,EV10-19,EV10-25 |
| PhaseFOutputSpecV1 | NESTED_WIRE | §12 PhaseFOutputSpecV1 exact power definition | no independent identity; parent bytes only | containing-object canonical bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | nested field of PhaseFPowerMethodInterfaceV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFPackageBindingV1 | NESTED_WIRE | §10 PhaseFPackageBindingV1 exact retrieval/package definition | no independent identity; parent bytes only | containing-object canonical bytes | retrieval/package authority | retrieval/package strict parser plus classification/DAG validator | F2 retrieval/package stage | nested field of PhaseFPackageManifestV1; no independent registry subject | R10-08,R10-09,R10-10,R10-13 | AC10-08,AC10-09,AC10-10,AC10-13 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-13 |
| PhaseFPackageManifestV1 | TOP_LEVEL_WIRE | §10 PhaseFPackageManifestV1 exact retrieval/package definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFPackageManifestV1 bytes | retrieval/package authority | retrieval/package strict parser plus classification/DAG validator | F2 retrieval/package stage | independent registry subject; exact object kind package_manifest | R10-03,R10-08,R10-09,R10-10,R10-13,R10-22,R10-25 | AC10-03,AC10-08,AC10-09,AC10-10,AC10-13,AC10-22,AC10-25 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-28,T10-CX-29,T10-KAT-RETENTION,T10-POS-DAG,T10-TRACE | EV10-03,EV10-08,EV10-09,EV10-10,EV10-13,EV10-22,EV10-25 |
| PhaseFPackageObjectV1 | NESTED_WIRE | §10 PhaseFPackageObjectV1 exact retrieval/package definition | no independent identity; parent bytes only | containing-object canonical bytes | retrieval/package authority | retrieval/package strict parser plus classification/DAG validator | F2 retrieval/package stage | nested field of PhaseFPackageManifestV1; no independent registry subject | R10-03,R10-08,R10-09,R10-10,R10-13,R10-25 | AC10-03,AC10-08,AC10-09,AC10-10,AC10-13,AC10-25 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-28,T10-CX-29,T10-KAT-RETENTION,T10-POS-DAG,T10-TRACE | EV10-03,EV10-08,EV10-09,EV10-10,EV10-13,EV10-25 |
| PhaseFParameterSpecV1 | NESTED_WIRE | §12 PhaseFParameterSpecV1 exact power definition | no independent identity; parent bytes only | containing-object canonical bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | nested field of PhaseFPowerMethodInterfaceV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFParameterValueRowV1 | NESTED_WIRE | §12 PhaseFParameterValueRowV1 exact power definition | no independent identity; parent bytes only | containing-object canonical bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | nested field of PhaseFPowerMethodInterfaceV1 and PhaseFPowerAnalysisRecordV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFPhysicalIdentityAuditV1 | TOP_LEVEL_WIRE | §11 PhaseFPhysicalIdentityAuditV1 exact physical/deviation definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFPhysicalIdentityAuditV1 bytes | campaign laboratory and custody authority | physical/custody strict parser plus identity/continuity validator | F2-F4 physical-validation stages | independent registry subject; exact object kind identity_audit | R10-08,R10-09,R10-10,R10-14 | AC10-08,AC10-09,AC10-10,AC10-14 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-14 |
| PhaseFPhysicalReleaseApprovalV1 | TAG_BODY | §11 PhaseFPhysicalReleaseApprovalV1 exact physical/deviation definition | no JSON semantic ID; annotated-tag message identity only | SHA-256 of exact annotated-tag message bytes when referenced | independent physical-release gate | annotated-tag grammar plus target/body/prerequisite validator | physical-release approval gate | annotated-tag message only; no registry subject | R10-08,R10-09,R10-10,R10-18 | AC10-08,AC10-09,AC10-10,AC10-18 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-18 |
| PhaseFPhysicalUnitLedgerV1 | TOP_LEVEL_WIRE | §11 PhaseFPhysicalUnitLedgerV1 exact physical/deviation definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFPhysicalUnitLedgerV1 bytes | campaign laboratory and custody authority | physical/custody strict parser plus identity/continuity validator | F2-F4 physical-validation stages | independent registry subject; exact object kind physical_unit_ledger | R10-08,R10-09,R10-10,R10-14 | AC10-08,AC10-09,AC10-10,AC10-14 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-14 |
| PhaseFPlanApprovalV1 | TAG_BODY | §2 PhaseFPlanApprovalV1 exact primitive/helper definition | no JSON semantic ID; annotated-tag message identity only | SHA-256 of exact annotated-tag message bytes when referenced | independent plan-review gate | annotated-tag grammar plus target/body/prerequisite validator | plan-review approval gate | annotated-tag message only; no registry subject | R10-01,R10-08,R10-09,R10-10 | AC10-01,AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-01,T10-CX-02,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-PLAN,T10-TRACE | EV10-01,EV10-08,EV10-09,EV10-10 |
| PhaseFPowerAnalysisRecordV1 | TOP_LEVEL_WIRE | §12 PhaseFPowerAnalysisRecordV1 exact power definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFPowerAnalysisRecordV1 bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | independent registry subject; exact object kind power_analysis | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFPowerMethodInterfaceV1 | TOP_LEVEL_WIRE | §12 PhaseFPowerMethodInterfaceV1 exact power definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFPowerMethodInterfaceV1 bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | independent registry subject; exact object kind power_method_interface | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFPowerOutputValueV1 | NESTED_WIRE | §2 PhaseFPowerOutputValueV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | nested field of PhaseFPowerAnalysisRecordV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFProtocolProjectionV1 | PLAN_ONLY_CONTRACT | §4 PhaseFProtocolProjectionV1 exact decision definition | no artifact identity; planning construct only | not applicable; no standalone bytes | plan author | plan consistency validator; no runtime artifact validator | the exact parent operation named in this row | plan-only contract; no standalone artifact or registry subject | R10-08,R10-09,R10-10,R10-11 | AC10-08,AC10-09,AC10-10,AC10-11 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-11 |
| PhaseFQuantifiedUncertaintyV1 | NESTED_WIRE | §2 PhaseFQuantifiedUncertaintyV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | nested field of PhaseFMetrologyPolicyV1; no independent registry subject | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFRangeRuleV1 | NESTED_WIRE | §2 PhaseFRangeRuleV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | nested field of PhaseFParameterSpecV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFReadinessApprovalV1 | TAG_BODY | §2 PhaseFReadinessApprovalV1 exact primitive/helper definition | no JSON semantic ID; annotated-tag message identity only | SHA-256 of exact annotated-tag message bytes when referenced | independent readiness gate | annotated-tag grammar plus target/body/prerequisite validator | readiness approval gate | annotated-tag message only; no registry subject | R10-08,R10-09,R10-10 | AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-TRACE | EV10-08,EV10-09,EV10-10 |
| PhaseFReferenceAssessmentV1 | NESTED_WIRE | §13 PhaseFReferenceAssessmentV1 exact metrology/reference definition | no independent identity; parent bytes only | containing-object canonical bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | nested field of PhaseFReferenceResultV1; no independent registry subject | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFReferenceResultV1 | TOP_LEVEL_WIRE | §13 PhaseFReferenceResultV1 exact metrology/reference definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFReferenceResultV1 bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | independent registry subject; exact object kind reference_result | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFReferenceSourceDescriptorV1 | TOP_LEVEL_WIRE | §13 PhaseFReferenceSourceDescriptorV1 exact metrology/reference definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFReferenceSourceDescriptorV1 bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | independent registry subject; exact object kind reference_source_descriptor | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFRegistryCompromiseEmergencyV1 | TOP_LEVEL_WIRE | §15 PhaseFRegistryCompromiseEmergencyV1 exact emergency fields | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFRegistryCompromiseEmergencyV1 bytes | security emergency authority | emergency strict parser plus exact path/claim-status validator | emergency stage | independent registry subject; exact object kind emergency_registry_compromise | R10-08,R10-09,R10-10,R10-19 | AC10-08,AC10-09,AC10-10,AC10-19 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-19 |
| PhaseFRegistryHeadV1 | TOP_LEVEL_WIRE | §8 PhaseFRegistryHeadV1 exact registry definition | sequence/predecessor identity from §8; no §3 semantic ID | SHA-256 of complete canonical PhaseFRegistryHeadV1 bytes | registry authority | registry strict parser plus sequence/signature/relation validator | all registry operations | registry resolver object; no authority subject row | R10-08,R10-09,R10-10,R10-12 | AC10-08,AC10-09,AC10-10,AC10-12 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-12 |
| PhaseFRegistryRecordV1 | TOP_LEVEL_WIRE | §8 PhaseFRegistryRecordV1 exact registry definition | sequence/predecessor identity from §8; no §3 semantic ID | SHA-256 of complete canonical PhaseFRegistryRecordV1 bytes | registry authority | registry strict parser plus sequence/signature/relation validator | all registry operations | signed registry-chain record; exact subject and relation fields | R10-05,R10-06,R10-08,R10-09,R10-10,R10-12 | AC10-05,AC10-06,AC10-08,AC10-09,AC10-10,AC10-12 | T10-CAT,T10-CX-01,T10-CX-02,T10-CX-03,T10-CX-04,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-POS-DAG,T10-TRACE | EV10-05,EV10-06,EV10-08,EV10-09,EV10-10,EV10-12 |
| PhaseFRegistryRelationV1 | NESTED_WIRE | §9 PhaseFRegistryRelationV1 exact relation definition | no independent identity; parent bytes only | containing-object canonical bytes | registry authority | registry strict parser plus sequence/signature/relation validator | all registry operations | nested field of PhaseFRegistryRecordV1; no independent registry subject | R10-05,R10-08,R10-09,R10-10 | AC10-05,AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-01,T10-CX-02,T10-CX-03,T10-CX-04,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-TRACE | EV10-05,EV10-08,EV10-09,EV10-10 |
| PhaseFReinstatementApprovalV1 | TOP_LEVEL_WIRE | §14 PhaseFReinstatementApprovalV1 exact release/monitoring definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFReinstatementApprovalV1 bytes | release authority | Phase-F schema strict parser plus field/type consistency validator | F4-F5 release/state stages | independent registry subject; exact object kind reinstatement_approval | R10-08,R10-09,R10-10,R10-18 | AC10-08,AC10-09,AC10-10,AC10-18 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-18 |
| PhaseFReleaseRecordV1 | TOP_LEVEL_WIRE | §14 PhaseFReleaseRecordV1 exact release/monitoring definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFReleaseRecordV1 bytes | release authority | Phase-F schema strict parser plus field/type consistency validator | F4-F5 release/state stages | independent registry subject; exact object kind release_record | R10-04,R10-08,R10-09,R10-10,R10-18,R10-25 | AC10-04,AC10-08,AC10-09,AC10-10,AC10-18,AC10-25 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-KAT-RETENTION,T10-POS-DAG,T10-POS-RETENTION,T10-TRACE | EV10-04,EV10-08,EV10-09,EV10-10,EV10-18,EV10-25 |
| PhaseFRetentionAuditV1 | TOP_LEVEL_WIRE | §15 PhaseFRetentionAuditV1 exact incident/retention definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFRetentionAuditV1 bytes | retention auditor | retention strict parser plus exact-set validator | campaign pre-release and release-retention operations | independent registry subject; exact object kind retention_audit | R10-03,R10-07,R10-08,R10-09,R10-10,R10-24,R10-25 | AC10-03,AC10-07,AC10-08,AC10-09,AC10-10,AC10-24,AC10-25 | T10-CAT,T10-CX-05,T10-CX-06,T10-CX-07,T10-CX-08,T10-CX-09,T10-CX-10,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31,T10-KAT-RETENTION,T10-POS-RETENTION,T10-TRACE | EV10-03,EV10-07,EV10-08,EV10-09,EV10-10,EV10-24,EV10-25 |
| PhaseFRetentionCopyVerificationV1 | NESTED_WIRE | §15 PhaseFRetentionCopyVerificationV1 exact incident/retention definition | no independent identity; parent bytes only | containing-object canonical bytes | retention auditor | retention copy retrieval/hash/count/freshness validator | campaign pre-release and release-retention operations | nested field of PhaseFRetentionObjectCheckV1; no independent registry subject | R10-07,R10-08,R10-09,R10-10,R10-24,R10-25 | AC10-07,AC10-08,AC10-09,AC10-10,AC10-24,AC10-25 | T10-CAT,T10-CX-05,T10-CX-06,T10-CX-07,T10-CX-08,T10-CX-09,T10-CX-10,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31,T10-KAT-RETENTION,T10-POS-RETENTION,T10-TRACE | EV10-07,EV10-08,EV10-09,EV10-10,EV10-24,EV10-25 |
| PhaseFRetentionObjectCheckV1 | NESTED_WIRE | §15 PhaseFRetentionObjectCheckV1 exact incident/retention definition | no independent identity; parent bytes only | containing-object canonical bytes | retention auditor | retention strict parser plus exact-set validator | campaign pre-release and release-retention operations | nested field of PhaseFRetentionAuditV1; no independent registry subject | R10-07,R10-08,R10-09,R10-10,R10-24,R10-25 | AC10-07,AC10-08,AC10-09,AC10-10,AC10-24,AC10-25 | T10-CAT,T10-CX-05,T10-CX-06,T10-CX-07,T10-CX-08,T10-CX-09,T10-CX-10,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31,T10-KAT-RETENTION,T10-POS-RETENTION,T10-TRACE | EV10-07,EV10-08,EV10-09,EV10-10,EV10-24,EV10-25 |
| PhaseFRetentionObjectV1 | NESTED_WIRE | §15 PhaseFRetentionObjectV1 exact incident/retention definition | no independent identity; parent bytes only | containing-object canonical bytes | retention auditor | retention strict parser plus exact-set validator | campaign pre-release and release-retention operations | nested field of PhaseFRetentionAuditV1; no independent registry subject | R10-03,R10-04,R10-05,R10-06,R10-08,R10-09,R10-10,R10-24 | AC10-03,AC10-04,AC10-05,AC10-06,AC10-08,AC10-09,AC10-10,AC10-24 | T10-CAT,T10-CX-01,T10-CX-02,T10-CX-03,T10-CX-04,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-17,T10-CX-18,T10-CX-19,T10-CX-20,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31,T10-POS-RETENTION,T10-TRACE | EV10-03,EV10-04,EV10-05,EV10-06,EV10-08,EV10-09,EV10-10,EV10-24 |
| PhaseFRetentionScopeV1 | NESTED_WIRE | §15 PhaseFRetentionScopeV1 exact incident/retention definition | no independent identity; parent bytes only | containing-object canonical bytes | retention auditor | retention strict parser plus exact-set validator | campaign pre-release and release-retention operations | nested field of PhaseFRetentionAuditV1; no independent registry subject | R10-03,R10-08,R10-09,R10-10 | AC10-03,AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-CX-28,T10-CX-29,T10-TRACE | EV10-03,EV10-08,EV10-09,EV10-10 |
| PhaseFRetrievalVerificationV1 | TOP_LEVEL_WIRE | §10 PhaseFRetrievalVerificationV1 exact retrieval/package definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFRetrievalVerificationV1 bytes | retrieval/package authority | retrieval-verification strict parser plus URI/hash/length verifier | F2 retrieval/package stage | standalone evidence file at explicit output path; not registered | R10-08,R10-09,R10-10,R10-13 | AC10-08,AC10-09,AC10-10,AC10-13 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-13 |
| PhaseFReviewTargetV1 | NESTED_WIRE | §5 PhaseFReviewTargetV1 exact review definition | no independent identity; parent bytes only | containing-object canonical bytes | independent review panel | Phase-F schema strict parser plus field/type consistency validator | the exact parent operation named in this row | nested field of PhaseFIndependentReviewBundleV1; no independent registry subject | R10-01,R10-08,R10-09,R10-10 | AC10-01,AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-01,T10-CX-02,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-PLAN,T10-TRACE | EV10-01,EV10-08,EV10-09,EV10-10 |
| PhaseFScientificAdmissibilityAuditV1 | TOP_LEVEL_WIRE | §2 PhaseFScientificAdmissibilityAuditV1 exact primitive/helper definition | §3 exact domain and own declared identity field | SHA-256 of complete canonical PhaseFScientificAdmissibilityAuditV1 bytes | Phase-F plan authority | Phase-F schema strict parser plus field/type consistency validator | the exact parent operation named in this row | independent registry subject; exact object kind scientific_admissibility_audit | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFSensitivityCaseV1 | NESTED_WIRE | §12 PhaseFSensitivityCaseV1 exact power definition | no independent identity; parent bytes only | containing-object canonical bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | nested field of PhaseFPowerAnalysisRecordV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFSensitivityOverrideV1 | NESTED_WIRE | §2 PhaseFSensitivityOverrideV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | power-analysis authority | power strict parser plus type/range/unit validator | F1 power-analysis stage | nested field of PhaseFSensitivityCaseV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |
| PhaseFTrustProvisioningApprovalV1 | TAG_BODY | §2 PhaseFTrustProvisioningApprovalV1 exact primitive/helper definition | no JSON semantic ID; annotated-tag message identity only | SHA-256 of exact annotated-tag message bytes when referenced | independent trust-provisioning gate | annotated-tag grammar plus target/body/prerequisite validator | F3 trust approval gate | annotated-tag message only; no registry subject | R10-02,R10-04,R10-08,R10-09,R10-10 | AC10-02,AC10-04,AC10-08,AC10-09,AC10-10 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16,T10-CX-19,T10-CX-20,T10-CX-21,T10-POS-RETENTION,T10-POS-TRUST,T10-TRACE | EV10-02,EV10-04,EV10-08,EV10-09,EV10-10 |
| PhaseFUncertaintyPolicyV1 | NESTED_WIRE | §2 PhaseFUncertaintyPolicyV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | metrology laboratory | metrology strict parser plus endpoint/provenance validator | F0-F2 metrology/reference stages | nested field of PhaseFMetrologyPolicyV1; no independent registry subject | R10-08,R10-09,R10-10,R10-17 | AC10-08,AC10-09,AC10-10,AC10-17 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-17 |
| PhaseFUnitEntryV1 | NESTED_WIRE | §11 PhaseFUnitEntryV1 exact physical/deviation definition | no independent identity; parent bytes only | containing-object canonical bytes | campaign laboratory and custody authority | physical/custody strict parser plus identity/continuity validator | F2-F4 physical-validation stages | nested field of PhaseFPhysicalUnitLedgerV1; no independent registry subject | R10-08,R10-09,R10-10,R10-14 | AC10-08,AC10-09,AC10-10,AC10-14 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-14 |
| PhaseFUnitRuleV1 | NESTED_WIRE | §2 PhaseFUnitRuleV1 exact primitive/helper definition | no independent identity; parent bytes only | containing-object canonical bytes | campaign laboratory and custody authority | physical/custody strict parser plus identity/continuity validator | F2-F4 physical-validation stages | nested field of PhaseFParameterSpecV1; no independent registry subject | R10-08,R10-09,R10-10,R10-16 | AC10-08,AC10-09,AC10-10,AC10-16 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-19,T10-CX-20,T10-POS-DAG,T10-TRACE | EV10-08,EV10-09,EV10-10,EV10-16 |

No row uses a generic closure, producer, validator, stage, or registry
placeholder. The category KAT is:

~~~text
PhaseFChainOfCustodyV1=TOP_LEVEL_WIRE
PhaseFCheckerReportV1=TOP_LEVEL_WIRE
PhaseFRetrievalVerificationV1=TOP_LEVEL_WIRE
PhaseFTrustProvisioningApprovalV1=TAG_BODY
PhaseFIncidentResolutionV1=TOP_LEVEL_WIRE
PhaseFRetentionCopyVerificationV1=NESTED_WIRE
~~~

### 42.3 Historical R10 catalog derivation rule

R10 CURRENT NORMATIVE REQUIREMENT MATRIX in §43 is the only normative
requirement/F-OD/schema mapping source. Catalog traceability columns are a
MANDATORY DERIVED INTEGRITY PROJECTION, not an independently authored mapping
graph. For schema S, current requirement IDs are all §43 rows whose literal
schema_ids contain S; current AC, test, and F-EV IDs are the sorted unions of
the primary AC, test, and evidence cells on those rows. Any mismatch makes the
plan invalid. Incident schemas derive from incident-resolution and retention
requirements; monitoring-source references derive from monitoring requirements;
retention audit derives from every current retention row that lists it.

## 43. Historical R10 normative requirement matrix (NON-CURRENT)

This is the one current R10 matrix. The R9 matrix in §34 is historical and
non-current. Every row has a normative statement, owner decisions, literal
schema IDs, stage, review roles, primary AC, tests, and F-EVs.

| requirement_id | normative_statement | owner_decision_ids | schema_ids | stage | review_roles | primary_ac_id | test_ids | evidence_ids |
|---|---|---|---|---|---|---|---|---|
| R10-01 | Future plan approval uses external CURRENT_PLAN_REVIEW_SHA and exact tag/body/plan equality. | none | PhaseFPlanApprovalV1,PhaseFIndependentReviewBundleV1,PhaseFReviewTargetV1 | plan review | architecture_data,security | AC10-01 | T10-POS-PLAN,T10-CX-01,T10-CX-02 | EV10-01 |
| R10-02 | Trust provisioning has one exact annotated-tag-message authority and exact trust monitoring binding. | F-OD-04 | PhaseFTrustProvisioningApprovalV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringEvidenceV1,PhaseFObjectReferenceV1 | F3,F5+ | security,operations_governance | AC10-02 | T10-POS-TRUST,T10-CX-13,T10-CX-14,T10-CX-15,T10-CX-16 | EV10-02 |
| R10-03 | Campaign retention is the exact manifest/object kind-and-SHA set with no locator derivation. | F-OD-18 | PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetentionAuditV1 | retention | operations_governance,security | AC10-03 | T10-CX-28,T10-CX-29 | EV10-03 |
| R10-04 | Static release retention includes every bound release authority and de-duplicates kind/SHA identities. | F-OD-13,F-OD-14,F-OD-15 | PhaseFReleaseRecordV1,PhaseFCohortLockRecordV1,PhaseFExecutionRecordV1,PhaseFMonitoringPolicyV1,PhaseFMetrologyPolicyV1,PhaseFClaimStateRecordV1,PhaseFIndependentReviewBundleV1,PhaseFTrustProvisioningApprovalV1,PhaseFRetentionObjectV1 | F5+ | operations_governance,security | AC10-04 | T10-POS-RETENTION,T10-CX-21 | EV10-04 |
| R10-05 | Incident membership uses the exact contained-before-terminal state machine at audited_at. | F-OD-16 | PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFRetentionObjectV1 | operations after incident detection | operations_governance,security | AC10-05 | T10-CX-01,T10-CX-02,T10-CX-03,T10-CX-04,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27 | EV10-05 |
| R10-06 | Release retention includes accepted PASS monitoring through audited_at and excludes all unaccepted/future records. | F-OD-19 | PhaseFMonitoringRecordV1,PhaseFMonitoringEvidenceV1,PhaseFRegistryRecordV1,PhaseFRetentionObjectV1 | F5+ | operations_governance | AC10-06 | T10-CX-22,T10-CX-23 | EV10-06 |
| R10-07 | Every retention identity has exact immutable copies, SHA, length, distinct URI, freshness, and 1+backup count. | F-OD-20 | PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionAuditV1,PhaseFObjectReferenceV1 | retention | security,operations_governance | AC10-07 | T10-CX-05,T10-CX-06,T10-CX-07,T10-CX-08,T10-CX-09,T10-CX-10,T10-CX-30,T10-CX-31 | EV10-07 |
| R10-08 | Current catalog categories match actual standalone wire, nested wire, tag-body, and plan-only use. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data | AC10-08 | T10-CAT,T10-CX-11,T10-CX-12,T10-CX-20 | EV10-08 |
| R10-09 | Current catalog metadata names exact closure, producer, validator, stage, and registry behavior for every row. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data,security | AC10-09 | T10-CAT,T10-CX-13,T10-CX-19 | EV10-09 |
| R10-10 | Current catalog traceability is mechanically the sorted inverse projection of this matrix. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data | AC10-10 | T10-TRACE,T10-CX-14,T10-CX-15 | EV10-10 |
| R10-11 | F0 retains exactly 20 owner decisions and exact runtime projection. | F-OD-01,F-OD-02 | PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFProtocolProjectionV1 | F0 | architecture_data,scientific_metrology | AC10-11 | T10-POS-DAG | EV10-11 |
| R10-12 | Readiness, unsigned enrollment, registry genesis, signatures, sequence, and relations remain exact. | F-OD-03,F-OD-04 | PhaseFCheckerBuildEvidenceV1,PhaseFCheckerReadinessEvidenceV1,PhaseFAuthorityEnrollmentV1,PhaseFRegistryRecordV1,PhaseFRegistryHeadV1 | readiness/enrollment | security,compatibility | AC10-12 | T10-POS-DAG | EV10-12 |
| R10-13 | Retrieval, package classification, dependencies, and package relations remain deterministic. | F-OD-05,F-OD-06,F-OD-07,F-OD-08 | PhaseFRetrievalVerificationV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFPackageBindingV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1 | F2 | security,scientific_metrology | AC10-13 | T10-POS-DAG | EV10-13 |
| R10-14 | Physical identity, pseudoreplication, location, and custody continuity remain exact. | F-OD-09,F-OD-10 | PhaseFPhysicalUnitLedgerV1,PhaseFUnitEntryV1,PhaseFPhysicalIdentityAuditV1,PhaseFIdentityComparisonV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFChainOfCustodyV1,PhaseFCustodyEventV1 | F2-F4 | scientific_metrology,operations_governance | AC10-14 | T10-POS-DAG | EV10-14 |
| R10-15 | Deviation revisions remain immutable, stable-ID, action-compatible, and acyclic. | F-OD-11 | PhaseFDeviationLedgerV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationEventV1 | F1-F4 | operations_governance,security | AC10-15 | T10-POS-DAG | EV10-15 |
| R10-16 | Power interface, typed values, ranges, units, sensitivity cases, outputs, review, and registration remain exact. | F-OD-12 | PhaseFPowerMethodInterfaceV1,PhaseFMethodVersionV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFOutputSpecV1,PhaseFPowerOutputValueV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFRangeRuleV1,PhaseFUnitRuleV1,PhaseFPowerAnalysisRecordV1 | F1 | scientific_metrology | AC10-16 | T10-POS-DAG | EV10-16 |
| R10-17 | Endpoint-qualified metrology and reference provenance/admissibility remain exact without new scope. | none | PhaseFMetrologyPolicyV1,PhaseFEndpointMetrologyPolicyV1,PhaseFCheckListV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyCheckResultV1,PhaseFLODLOQPolicyV1,PhaseFUncertaintyPolicyV1,PhaseFQuantifiedUncertaintyV1,PhaseFReferenceSourceDescriptorV1,PhaseFReferenceResultV1,PhaseFReferenceAssessmentV1,PhaseFScientificAdmissibilityAuditV1 | F0-F2 | scientific_metrology | AC10-17 | T10-POS-DAG | EV10-17 |
| R10-18 | Claim state causes, release chronology, final release authority, and P2 readiness remain exact. | none | PhaseFClaimStateRecordV1,PhaseFReinstatementApprovalV1,PhaseFIncidentScopeV1,PhaseFReleaseRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFPhysicalReleaseApprovalV1 | F4-F5 | operations_governance,architecture_data | AC10-18 | T10-POS-DAG | EV10-18 |
| R10-19 | Registry-compromise emergency publication and fail-closed status remain acyclic and path-exact. | none | PhaseFRegistryCompromiseEmergencyV1,PhaseFIndependentReviewBundleV1,PhaseFObjectReferenceV1 | emergency | security,operations_governance | AC10-19 | T10-POS-DAG | EV10-19 |
| R10-20 | The complete positive path remains constructible in production runner order with no future-file or self-Git edge. | none | PhaseFCommandV1,PhaseFArgvV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFCheckerExitCodeV1 | all | architecture_data,compatibility | AC10-20 | T10-POS-DAG | EV10-20 |
| R10-21 | The fixed 15-metric monitoring contract and trust-source binding remain exact. | F-OD-17 | PhaseFMonitoringPolicyV1,PhaseFMetricThresholdV1,PhaseFMonitoringRecordV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1 | F0/F5+ | operations_governance,security | AC10-21 | T10-POS-TRUST,T10-POS-DAG | EV10-21 |
| R10-22 | Campaign-abandonment review remains incident-first and cannot introduce a reverse pointer or retention cycle. | F-OD-18 | PhaseFIncidentRecordV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFPackageManifestV1 | all | operations_governance,security | AC10-22 | T10-POS-DAG | EV10-22 |
| R10-23 | Owner-decision coverage has exactly one current source and union F-OD-01 through F-OD-20. | none | PhaseFDecisionBundleV1,PhaseFDecisionRowV1 | plan review | architecture_data | AC10-23 | T10-TRACE | EV10-23 |
| R10-24 | Release retention is exact-set equal, de-duplicated, incident-aware, monitoring-aware, copy-covered, and fail-closed. | F-OD-20 | PhaseFRetentionAuditV1,PhaseFRetentionObjectV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionCopyVerificationV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFMonitoringRecordV1,PhaseFClaimStateRecordV1 | retention | security,operations_governance | AC10-24 | T10-POS-RETENTION,T10-CX-17,T10-CX-18,T10-CX-21,T10-CX-22,T10-CX-23,T10-CX-24,T10-CX-25,T10-CX-26,T10-CX-27,T10-CX-28,T10-CX-29,T10-CX-30,T10-CX-31 | EV10-24 |
| R10-25 | One deterministic R10 KAT derives the complete release set, chronology, and literal two-copy coverage. | none | PhaseFReleaseRecordV1,PhaseFClaimStateRecordV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFMonitoringRecordV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFRetentionAuditV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionCopyVerificationV1,PhaseFObjectReferenceV1 | F5+ | architecture_data,security,operations_governance | AC10-25 | T10-KAT-RETENTION,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19 | EV10-25 |

The derived owner-decision union from owner_decision_ids is exactly:

~~~text
F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-05,F-OD-06,F-OD-07,F-OD-08,F-OD-09,F-OD-10,
F-OD-11,F-OD-12,F-OD-13,F-OD-14,F-OD-15,F-OD-16,F-OD-17,F-OD-18,F-OD-19,F-OD-20
~~~

There is no F-OD-21 and no second current mapping source.

## 44. Historical R10 retention KAT profile and literal byte table

R10_RETENTION_KAT_F0 is one explicit PLAN-EMBEDDED TEST-ONLY profile. THESE
ARE KAT-ONLY TEST VALUES. THEY DO NOT PRE-DECIDE FUTURE F0:

~~~text
allowed_immutable_uri_schemes=["fixture+sha256"]
retention_seconds="7200"
backup_copy_count="1"
backup_verification_interval_seconds="3600"
authorized_access_role_ids=["r10.retention.fixture"]
replacement_authority_role_id="r10.retention.fixture"
unavailable_object_action="no_go"
audited_at=2026-01-01T02:00:00Z
copy-A verified_at=2026-01-01T01:15:00Z
copy-B verified_at=2026-01-01T01:30:00Z
audited_at-copy-A=2700 seconds
audited_at-copy-B=1800 seconds
~~~

Each identity requires exactly one primary verified copy plus one backup:
two valid fresh distinct copies. Every source byte object is the UTF-8 byte
string r10-retention-fixture/<LABEL> followed by LF, where LABEL is the literal
label in the table and contains neither its SHA, URI, nor length.

| label | exact UTF-8 fixture bytes | byte_length | sha256 |
|---|---|---:|---|
| manifest-M | r10-retention-fixture/manifest-M\n | 33 | df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 |
| package-O1 | r10-retention-fixture/package-O1\n | 33 | 90ee35572a33af574c0c8fdb137aee2a7b6d325a7cc60efb36c942fa2d11065f |
| package-O2 | r10-retention-fixture/package-O2\n | 33 | 24050263ca4a8ec011c7d718e1d0fc52b2350d67265b925b636b75f0a87374ee |
| protocol-P | r10-retention-fixture/protocol-P\n | 33 | 73fb452c36f96f4db361f5bbca287c20f6a899296db86ec064992de91880fcd3 |
| power-A | r10-retention-fixture/power-A\n | 30 | 380d4d36ac0f8bae7d2558eb22e7003739b86f2b2222d9f507fbf3967672a58a |
| cohort-C | r10-retention-fixture/cohort-C\n | 31 | 9d75af9d77ddc8ff159d961e0f35ac8b1d3eee6e412e007b90faf49161a6b196 |
| owner-OA | r10-retention-fixture/owner-OA\n | 31 | d06f4b22f31603f062847547a6aefeac400d6556a392df1a499b78c0eb031dc4 |
| execution-E | r10-retention-fixture/execution-E\n | 34 | 4e13d9bc0c22f4acd5d5ae6e9f500d143dd17ca40f15c6719830215914b59876 |
| release-R | r10-retention-fixture/release-R\n | 32 | e136326de2b7a883df571265505752f7600422519de2ce823ca322633c1cd856 |
| validation-VM | r10-retention-fixture/validation-VM\n | 36 | a2e77f55d1a719a28db66e299f9500dd7c0e826cf86f9ebf50f021c2b3ebc8e7 |
| monitoring-policy-MP | r10-retention-fixture/monitoring-policy-MP\n | 43 | 22b90909658de8d2fead19836e5b71d3bc8e92ac6ef941434cc7b0bac19be6fb |
| metrology-policy-MET | r10-retention-fixture/metrology-policy-MET\n | 43 | e592b9ff6c5ec6890910097154bbfed958cf9383651aed3f41072ef208cc3f64 |
| trust-tag-TT | r10-retention-fixture/trust-tag-TT\n | 35 | 09c14e2106d90c93d25b3c2793d97f94ec74e87f77a817d004bca714723113f9 |
| f5-review-RB | r10-retention-fixture/f5-review-RB\n | 35 | 62dbce3dc5b0203ad287599e35b246ae81710859535b4486de3e7126637ad56f |
| claim-S0 | r10-retention-fixture/claim-S0\n | 31 | 7b6387ff4259fafe1418268a9c80c9aa6811c26e744f629e92f3355ea2b6d69e |
| claim-S1 | r10-retention-fixture/claim-S1\n | 31 | 390de3582256a63f592d43f418927c80828947a127d21eb5ef42eb760ae50003 |
| monitor-MR1 | r10-retention-fixture/monitor-MR1\n | 34 | d38e0ed279e3ce7c601a4ca25e46d7a7489268d911c904bed43e2a2159d9e035 |
| monitor-MR2 | r10-retention-fixture/monitor-MR2\n | 34 | 6a2ddbc9c8ab34f013ea6202a5b062e6a892e247cb6c1a2ef6f50f91b0c0c524 |
| incident-I1 | r10-retention-fixture/incident-I1\n | 34 | 98d48c525f84456d66549129982203dd903747d05eac2a55d03e8137d9e8976d |
| incident-I2 | r10-retention-fixture/incident-I2\n | 34 | 5bb81df36e55d4ec3e3e4d894c163adb9cba2f9ae009c502b26a1d4eca08733e |
| resolution-I2-C | r10-retention-fixture/resolution-I2-C\n | 38 | f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c |
| resolution-I2-R | r10-retention-fixture/resolution-I2-R\n | 38 | 373cf9999b808695b6dde6971de2b8c9946f766c2ddbb35804d2b885bfd48d55 |

The excluded incident-I2 row is materialized so its complete incident identity
is reproducible, but it is not a retention member.
RETENTION_KAT_SYMBOLIC_BYTE_VALUES=0.

## 45. Historical R10 positive incident fixture and expected retention set

The positive incident fixture is:

~~~text
I1 incident file: incident_status=open; detected_at=2025-12-31T23:00:00Z
I1 incident_recorded created_at=2025-12-31T23:05:00Z

I2 incident file: incident_status=open; detected_at=2026-01-01T00:10:00Z
I2 incident_recorded created_at=2026-01-01T00:20:00Z

IR2-C resolution file: incident_id=I2; resolution_number="0";
previous_resolution_sha256=null; resolution_status=contained;
effective_at=2026-01-01T00:30:00Z
IR2-C incident_resolution_recorded created_at=2026-01-01T00:40:00Z

IR2-R resolution file: incident_id=I2; resolution_number="1";
previous_resolution_sha256=f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c;
resolution_status=resolved; effective_at=2026-01-01T01:00:00Z
IR2-R incident_resolution_recorded created_at=2026-01-01T01:10:00Z;
registered_after+incident_resolution=IR2-C

IR2-C.effective_at < IR2-R.effective_at <= audited_at
both incident_resolution_recorded(IR2-C) and incident_resolution_recorded(IR2-R)
are valid and created no later than audited_at
~~~

previous_resolution_sha256 is the literal SHA of the complete canonical IR2-C
fixture bytes declared in §44. At audited_at, I1 has no eligible resolution and
is retained as incident_record(I1). I2 is RESOLVED and is not retained as
incident_record(I2). Both incident_resolution(IR2-C) and
incident_resolution(IR2-R) are retained because both complete objects are
required to validate the contained-before-terminal proof.

The exact ReleaseRetentionSetV1 has these 21 members:

| identity key | fixture label | literal SHA-256 |
|---|---|---|
| package_manifest | manifest-M | df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 |
| package_object:O1 | package-O1 | 90ee35572a33af574c0c8fdb137aee2a7b6d325a7cc60efb36c942fa2d11065f |
| package_object:O2 | package-O2 | 24050263ca4a8ec011c7d718e1d0fc52b2350d67265b925b636b75f0a87374ee |
| protocol | protocol-P | 73fb452c36f96f4db361f5bbca287c20f6a899296db86ec064992de91880fcd3 |
| power_analysis | power-A | 380d4d36ac0f8bae7d2558eb22e7003739b86f2b2222d9f507fbf3967672a58a |
| cohort_lock | cohort-C | 9d75af9d77ddc8ff159d961e0f35ac8b1d3eee6e412e007b90faf49161a6b196 |
| owner_approval | owner-OA | d06f4b22f31603f062847547a6aefeac400d6556a392df1a499b78c0eb031dc4 |
| execution_record | execution-E | 4e13d9bc0c22f4acd5d5ae6e9f500d143dd17ca40f15c6719830215914b59876 |
| release_record | release-R | e136326de2b7a883df571265505752f7600422519de2ce823ca322633c1cd856 |
| validation_manifest | validation-VM | a2e77f55d1a719a28db66e299f9500dd7c0e826cf86f9ebf50f021c2b3ebc8e7 |
| monitoring_policy | monitoring-policy-MP | 22b90909658de8d2fead19836e5b71d3bc8e92ac6ef941434cc7b0bac19be6fb |
| metrology_policy | metrology-policy-MET | e592b9ff6c5ec6890910097154bbfed958cf9383651aed3f41072ef208cc3f64 |
| git_tag_message | trust-tag-TT | 09c14e2106d90c93d25b3c2793d97f94ec74e87f77a817d004bca714723113f9 |
| independent_review_bundle | f5-review-RB | 62dbce3dc5b0203ad287599e35b246ae81710859535b4486de3e7126637ad56f |
| claim_state:S0 | claim-S0 | 7b6387ff4259fafe1418268a9c80c9aa6811c26e744f629e92f3355ea2b6d69e |
| claim_state:S1 | claim-S1 | 390de3582256a63f592d43f418927c80828947a127d21eb5ef42eb760ae50003 |
| monitoring_record:MR1 | monitor-MR1 | d38e0ed279e3ce7c601a4ca25e46d7a7489268d911c904bed43e2a2159d9e035 |
| monitoring_record:MR2 | monitor-MR2 | 6a2ddbc9c8ab34f013ea6202a5b062e6a892e247cb6c1a2ef6f50f91b0c0c524 |
| incident_record:I1 | incident-I1 | 98d48c525f84456d66549129982203dd903747d05eac2a55d03e8137d9e8976d |
| incident_resolution:IR2-C | resolution-I2-C | f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c |
| incident_resolution:IR2-R | resolution-I2-R | 373cf9999b808695b6dde6971de2b8c9946f766c2ddbb35804d2b885bfd48d55 |

The set contains campaign manifest/object identities, static release
authorities, distinct S0/S1 state identities, accepted MR1/MR2 monitoring
identities, I1, and both IR2 resolution authorities. It does not contain
incident-I2. Expected retention-member count is the literal integer 21.
INCIDENT_POSITIVE_FIXTURE_CONSISTENT=yes.

## 46. Historical R10 positive copy coverage

The following is the complete two-copy table for every one of the 21 expected
members. Each member has exactly one copy-A and one copy-B row; every row is a
literal PASS. Copy URIs are immutable and distinct.

| identity key | copy | URI | SHA | byte_length | verified_at | result |
|---|---|---|---|---:|---|---|
| package_manifest | copy-A | fixture+sha256://copy-a/df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 | df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 | 33 | 2026-01-01T01:15:00Z | PASS |
| package_manifest | copy-B | fixture+sha256://copy-b/df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 | df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 | 33 | 2026-01-01T01:30:00Z | PASS |
| package_object:O1 | copy-A | fixture+sha256://copy-a/90ee35572a33af574c0c8fdb137aee2a7b6d325a7cc60efb36c942fa2d11065f | 90ee35572a33af574c0c8fdb137aee2a7b6d325a7cc60efb36c942fa2d11065f | 33 | 2026-01-01T01:15:00Z | PASS |
| package_object:O1 | copy-B | fixture+sha256://copy-b/90ee35572a33af574c0c8fdb137aee2a7b6d325a7cc60efb36c942fa2d11065f | 90ee35572a33af574c0c8fdb137aee2a7b6d325a7cc60efb36c942fa2d11065f | 33 | 2026-01-01T01:30:00Z | PASS |
| package_object:O2 | copy-A | fixture+sha256://copy-a/24050263ca4a8ec011c7d718e1d0fc52b2350d67265b925b636b75f0a87374ee | 24050263ca4a8ec011c7d718e1d0fc52b2350d67265b925b636b75f0a87374ee | 33 | 2026-01-01T01:15:00Z | PASS |
| package_object:O2 | copy-B | fixture+sha256://copy-b/24050263ca4a8ec011c7d718e1d0fc52b2350d67265b925b636b75f0a87374ee | 24050263ca4a8ec011c7d718e1d0fc52b2350d67265b925b636b75f0a87374ee | 33 | 2026-01-01T01:30:00Z | PASS |
| protocol | copy-A | fixture+sha256://copy-a/73fb452c36f96f4db361f5bbca287c20f6a899296db86ec064992de91880fcd3 | 73fb452c36f96f4db361f5bbca287c20f6a899296db86ec064992de91880fcd3 | 33 | 2026-01-01T01:15:00Z | PASS |
| protocol | copy-B | fixture+sha256://copy-b/73fb452c36f96f4db361f5bbca287c20f6a899296db86ec064992de91880fcd3 | 73fb452c36f96f4db361f5bbca287c20f6a899296db86ec064992de91880fcd3 | 33 | 2026-01-01T01:30:00Z | PASS |
| power_analysis | copy-A | fixture+sha256://copy-a/380d4d36ac0f8bae7d2558eb22e7003739b86f2b2222d9f507fbf3967672a58a | 380d4d36ac0f8bae7d2558eb22e7003739b86f2b2222d9f507fbf3967672a58a | 30 | 2026-01-01T01:15:00Z | PASS |
| power_analysis | copy-B | fixture+sha256://copy-b/380d4d36ac0f8bae7d2558eb22e7003739b86f2b2222d9f507fbf3967672a58a | 380d4d36ac0f8bae7d2558eb22e7003739b86f2b2222d9f507fbf3967672a58a | 30 | 2026-01-01T01:30:00Z | PASS |
| cohort_lock | copy-A | fixture+sha256://copy-a/9d75af9d77ddc8ff159d961e0f35ac8b1d3eee6e412e007b90faf49161a6b196 | 9d75af9d77ddc8ff159d961e0f35ac8b1d3eee6e412e007b90faf49161a6b196 | 31 | 2026-01-01T01:15:00Z | PASS |
| cohort_lock | copy-B | fixture+sha256://copy-b/9d75af9d77ddc8ff159d961e0f35ac8b1d3eee6e412e007b90faf49161a6b196 | 9d75af9d77ddc8ff159d961e0f35ac8b1d3eee6e412e007b90faf49161a6b196 | 31 | 2026-01-01T01:30:00Z | PASS |
| owner_approval | copy-A | fixture+sha256://copy-a/d06f4b22f31603f062847547a6aefeac400d6556a392df1a499b78c0eb031dc4 | d06f4b22f31603f062847547a6aefeac400d6556a392df1a499b78c0eb031dc4 | 31 | 2026-01-01T01:15:00Z | PASS |
| owner_approval | copy-B | fixture+sha256://copy-b/d06f4b22f31603f062847547a6aefeac400d6556a392df1a499b78c0eb031dc4 | d06f4b22f31603f062847547a6aefeac400d6556a392df1a499b78c0eb031dc4 | 31 | 2026-01-01T01:30:00Z | PASS |
| execution_record | copy-A | fixture+sha256://copy-a/4e13d9bc0c22f4acd5d5ae6e9f500d143dd17ca40f15c6719830215914b59876 | 4e13d9bc0c22f4acd5d5ae6e9f500d143dd17ca40f15c6719830215914b59876 | 34 | 2026-01-01T01:15:00Z | PASS |
| execution_record | copy-B | fixture+sha256://copy-b/4e13d9bc0c22f4acd5d5ae6e9f500d143dd17ca40f15c6719830215914b59876 | 4e13d9bc0c22f4acd5d5ae6e9f500d143dd17ca40f15c6719830215914b59876 | 34 | 2026-01-01T01:30:00Z | PASS |
| release_record | copy-A | fixture+sha256://copy-a/e136326de2b7a883df571265505752f7600422519de2ce823ca322633c1cd856 | e136326de2b7a883df571265505752f7600422519de2ce823ca322633c1cd856 | 32 | 2026-01-01T01:15:00Z | PASS |
| release_record | copy-B | fixture+sha256://copy-b/e136326de2b7a883df571265505752f7600422519de2ce823ca322633c1cd856 | e136326de2b7a883df571265505752f7600422519de2ce823ca322633c1cd856 | 32 | 2026-01-01T01:30:00Z | PASS |
| validation_manifest | copy-A | fixture+sha256://copy-a/a2e77f55d1a719a28db66e299f9500dd7c0e826cf86f9ebf50f021c2b3ebc8e7 | a2e77f55d1a719a28db66e299f9500dd7c0e826cf86f9ebf50f021c2b3ebc8e7 | 36 | 2026-01-01T01:15:00Z | PASS |
| validation_manifest | copy-B | fixture+sha256://copy-b/a2e77f55d1a719a28db66e299f9500dd7c0e826cf86f9ebf50f021c2b3ebc8e7 | a2e77f55d1a719a28db66e299f9500dd7c0e826cf86f9ebf50f021c2b3ebc8e7 | 36 | 2026-01-01T01:30:00Z | PASS |
| monitoring_policy | copy-A | fixture+sha256://copy-a/22b90909658de8d2fead19836e5b71d3bc8e92ac6ef941434cc7b0bac19be6fb | 22b90909658de8d2fead19836e5b71d3bc8e92ac6ef941434cc7b0bac19be6fb | 43 | 2026-01-01T01:15:00Z | PASS |
| monitoring_policy | copy-B | fixture+sha256://copy-b/22b90909658de8d2fead19836e5b71d3bc8e92ac6ef941434cc7b0bac19be6fb | 22b90909658de8d2fead19836e5b71d3bc8e92ac6ef941434cc7b0bac19be6fb | 43 | 2026-01-01T01:30:00Z | PASS |
| metrology_policy | copy-A | fixture+sha256://copy-a/e592b9ff6c5ec6890910097154bbfed958cf9383651aed3f41072ef208cc3f64 | e592b9ff6c5ec6890910097154bbfed958cf9383651aed3f41072ef208cc3f64 | 43 | 2026-01-01T01:15:00Z | PASS |
| metrology_policy | copy-B | fixture+sha256://copy-b/e592b9ff6c5ec6890910097154bbfed958cf9383651aed3f41072ef208cc3f64 | e592b9ff6c5ec6890910097154bbfed958cf9383651aed3f41072ef208cc3f64 | 43 | 2026-01-01T01:30:00Z | PASS |
| git_tag_message | copy-A | fixture+sha256://copy-a/09c14e2106d90c93d25b3c2793d97f94ec74e87f77a817d004bca714723113f9 | 09c14e2106d90c93d25b3c2793d97f94ec74e87f77a817d004bca714723113f9 | 35 | 2026-01-01T01:15:00Z | PASS |
| git_tag_message | copy-B | fixture+sha256://copy-b/09c14e2106d90c93d25b3c2793d97f94ec74e87f77a817d004bca714723113f9 | 09c14e2106d90c93d25b3c2793d97f94ec74e87f77a817d004bca714723113f9 | 35 | 2026-01-01T01:30:00Z | PASS |
| independent_review_bundle | copy-A | fixture+sha256://copy-a/62dbce3dc5b0203ad287599e35b246ae81710859535b4486de3e7126637ad56f | 62dbce3dc5b0203ad287599e35b246ae81710859535b4486de3e7126637ad56f | 35 | 2026-01-01T01:15:00Z | PASS |
| independent_review_bundle | copy-B | fixture+sha256://copy-b/62dbce3dc5b0203ad287599e35b246ae81710859535b4486de3e7126637ad56f | 62dbce3dc5b0203ad287599e35b246ae81710859535b4486de3e7126637ad56f | 35 | 2026-01-01T01:30:00Z | PASS |
| claim_state:S0 | copy-A | fixture+sha256://copy-a/7b6387ff4259fafe1418268a9c80c9aa6811c26e744f629e92f3355ea2b6d69e | 7b6387ff4259fafe1418268a9c80c9aa6811c26e744f629e92f3355ea2b6d69e | 31 | 2026-01-01T01:15:00Z | PASS |
| claim_state:S0 | copy-B | fixture+sha256://copy-b/7b6387ff4259fafe1418268a9c80c9aa6811c26e744f629e92f3355ea2b6d69e | 7b6387ff4259fafe1418268a9c80c9aa6811c26e744f629e92f3355ea2b6d69e | 31 | 2026-01-01T01:30:00Z | PASS |
| claim_state:S1 | copy-A | fixture+sha256://copy-a/390de3582256a63f592d43f418927c80828947a127d21eb5ef42eb760ae50003 | 390de3582256a63f592d43f418927c80828947a127d21eb5ef42eb760ae50003 | 31 | 2026-01-01T01:15:00Z | PASS |
| claim_state:S1 | copy-B | fixture+sha256://copy-b/390de3582256a63f592d43f418927c80828947a127d21eb5ef42eb760ae50003 | 390de3582256a63f592d43f418927c80828947a127d21eb5ef42eb760ae50003 | 31 | 2026-01-01T01:30:00Z | PASS |
| monitoring_record:MR1 | copy-A | fixture+sha256://copy-a/d38e0ed279e3ce7c601a4ca25e46d7a7489268d911c904bed43e2a2159d9e035 | d38e0ed279e3ce7c601a4ca25e46d7a7489268d911c904bed43e2a2159d9e035 | 34 | 2026-01-01T01:15:00Z | PASS |
| monitoring_record:MR1 | copy-B | fixture+sha256://copy-b/d38e0ed279e3ce7c601a4ca25e46d7a7489268d911c904bed43e2a2159d9e035 | d38e0ed279e3ce7c601a4ca25e46d7a7489268d911c904bed43e2a2159d9e035 | 34 | 2026-01-01T01:30:00Z | PASS |
| monitoring_record:MR2 | copy-A | fixture+sha256://copy-a/6a2ddbc9c8ab34f013ea6202a5b062e6a892e247cb6c1a2ef6f50f91b0c0c524 | 6a2ddbc9c8ab34f013ea6202a5b062e6a892e247cb6c1a2ef6f50f91b0c0c524 | 34 | 2026-01-01T01:15:00Z | PASS |
| monitoring_record:MR2 | copy-B | fixture+sha256://copy-b/6a2ddbc9c8ab34f013ea6202a5b062e6a892e247cb6c1a2ef6f50f91b0c0c524 | 6a2ddbc9c8ab34f013ea6202a5b062e6a892e247cb6c1a2ef6f50f91b0c0c524 | 34 | 2026-01-01T01:30:00Z | PASS |
| incident_record:I1 | copy-A | fixture+sha256://copy-a/98d48c525f84456d66549129982203dd903747d05eac2a55d03e8137d9e8976d | 98d48c525f84456d66549129982203dd903747d05eac2a55d03e8137d9e8976d | 34 | 2026-01-01T01:15:00Z | PASS |
| incident_record:I1 | copy-B | fixture+sha256://copy-b/98d48c525f84456d66549129982203dd903747d05eac2a55d03e8137d9e8976d | 98d48c525f84456d66549129982203dd903747d05eac2a55d03e8137d9e8976d | 34 | 2026-01-01T01:30:00Z | PASS |
| incident_resolution:IR2-C | copy-A | fixture+sha256://copy-a/f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c | f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c | 38 | 2026-01-01T01:15:00Z | PASS |
| incident_resolution:IR2-C | copy-B | fixture+sha256://copy-b/f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c | f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c | 38 | 2026-01-01T01:30:00Z | PASS |
| incident_resolution:IR2-R | copy-A | fixture+sha256://copy-a/373cf9999b808695b6dde6971de2b8c9946f766c2ddbb35804d2b885bfd48d55 | 373cf9999b808695b6dde6971de2b8c9946f766c2ddbb35804d2b885bfd48d55 | 38 | 2026-01-01T01:15:00Z | PASS |
| incident_resolution:IR2-R | copy-B | fixture+sha256://copy-b/373cf9999b808695b6dde6971de2b8c9946f766c2ddbb35804d2b885bfd48d55 | 373cf9999b808695b6dde6971de2b8c9946f766c2ddbb35804d2b885bfd48d55 | 38 | 2026-01-01T01:30:00Z | PASS |

The positive KAT has exactly 42 PASS copy rows, two per identity, with no
conditional PASS language. RETENTION_KAT_SYMBOLIC_COPY_VALUES=0 and
RETENTION_POSITIVE_KAT_FULLY_MATERIALIZED=yes.

## 48. Historical R10 acceptance criteria

Every current R10 acceptance criterion has preconditions, exact inputs,
operation, expected result/output, and a failure oracle.

| AC ID | preconditions | exact inputs | exact operation | expected result | expected output | failure oracle |
|---|---|---|---|---|---|---|
| AC10-01 | External X, plan bytes, review bundle, and future plan tag | peeled plan tag target/body, CURRENT_PLAN_REVIEW_SHA=X, plan SHA-256, and plan Git blob | Peel tag; compare target/body/X and plan SHA-256/Git blob | Only exact X/body/plan identities pass | plan-tag report | stale target, mismatch, or self-SHA path is INVALID TAG / NO-GO |
| AC10-02 | Trust approval tag, exact tag-message bytes, source reference, and trust measurements | trust approval tag message, exact tag-message SHA/length, trust source reference, and five trust measurements | Resolve tag, validate prerequisites, hash bytes, compare length and trust fields | One exact tag message and matching fields pass | trust binding transcript | JSON authority object, wrong bytes/hash, or field mismatch rejects |
| AC10-03 | Manifest M and O1/O2 package objects | manifest-M, package-O1/O2 kind-SHA bindings, and campaign scope | Derive package/object kind-SHA keys without URI or length | Exact campaign set passes | campaign set report | missing, extra, duplicate, or locator-derived member fails |
| AC10-04 | Release bindings, registry chain, trust tag, F5 review, states, and static rows | protocol-P, power-A, cohort-C, owner-OA, execution-E, release-R, validation-VM, policy/state/review/trust bindings | Apply every static derivation row and de-duplicate kind/SHA keys | Exact static release set passes | static retention report | missing or unresolvable binding is NO-GO |
| AC10-05 | I1/I2, IR2-C/IR2-R, registry records, and audited_at | I1/I2, IR2-C/IR2-R fields and hashes, registry records, and audited_at=2026-01-01T02:00:00Z | Filter event/registry times; validate number/predecessor chain; classify latest eligible status | No resolution/contained is unresolved; valid resolved/superseded terminal is resolved | audited-at incident report | broken chain, terminal continuation, illegal first terminal, or chronology conflict fails |
| AC10-06 | Accepted PASS, suspend, unaccepted, late, and future monitoring records | 15 monitoring records/measurement windows, acceptance statuses, current chain, and audited_at | Validate attestation, current chain, window_end, acceptance, and cutoff | Only exact accepted monitoring set passes | monitoring membership report | included excluded record or omitted accepted record fails |
| AC10-07 | Every expected identity, two copy rows, retrieved bytes, and test-only F0 policy | 21 expected identities, two copy rows each, fixture URI/SHA/length values, retrieved bytes, and the KAT profile | Verify scheme, availability, byte length/SHA, URI distinctness, freshness, count, set equality | Every identity has two valid fresh distinct PASS copies | copy coverage transcript | missing/extra/bad/stale/duplicate/insufficient copy is NO-GO |
| AC10-08 | Complete plan and all 91 current identifiers | 91 PhaseF identifiers and one current catalog row per identifier | Enumerate identifiers and compare category to actual wire use | 91=91 and category mismatch count is zero | catalog category report | missing, extra, duplicate, or wrong category fails |
| AC10-09 | Complete plan and all 91 current rows | 91 current catalog rows and their exact definition sections | Inspect exact closure, producer, validator, stage, registry behavior, and banned-string cells | All metadata counters are zero | catalog metadata report | blank, broad, generic, or conflicting metadata fails |
| AC10-10 | R10 matrix and current catalog | R10 matrix rows and current catalog traceability cells | Project matrix rows inversely by schema ID; sort and compare cells | Catalog projection equals matrix inverse | traceability report | independent mapping, route mismatch, or second source fails |
| AC10-11 | Twenty F0 rows and runtime projection | 20 F0 owner-decision rows and the runtime projection | Validate fixed IDs, values, owners, and projection | Exact 20-decision bundle passes | F0 projection report | 21st row or projection drift fails |
| AC10-12 | Builds, readiness, enrollment, genesis, head, and records | build transcripts, readiness, enrollment, genesis/head, and registry records | Validate schemas, unsigned enrollment, signatures, sequence, predecessor, relations | Preserved registry path passes | registry transcript | pointer/signature/sequence/relation error fails |
| AC10-13 | Retrieval, package, dependency, and relation fixtures | retrieval/package/dependency fixtures, bytes, classifications, and relations | Verify bytes, lengths, hashes, classifications, DAG, and relations | Package authority passes | package report | unavailable object or relation/classification gap fails |
| AC10-14 | Unit/location/custody ledgers and comparisons | unit/location/custody ledgers and identity comparisons | Recompute native identity, pseudoreplication, location, and continuity | Physical authority passes | identity/custody report | alias, discontinuity, or post-destroy use fails |
| AC10-15 | Ledger ID, prior revision, and new deviation revision | ledger/revision records, prior SHA, sequence, and action fields | Verify immutable predecessor SHA, sequence, action compatibility, acyclicity | One-way deviation revision passes | deviation report | prior mutation or incompatible action fails |
| AC10-16 | F0 method/version, power interface/analysis, cases, and review | method/version, interface/analysis fields, sensitivity cases, review, and registration | Validate typed fields, ranges, units, equality, review-before-registration, outputs | Power authority passes | power report | missing range/unit or early registration fails |
| AC10-17 | Metrology policies/checks, references, audit, and projection | metrology/check/reference files, endpoint keys, inputs, and provenance | Resolve endpoint/check keys and math; enforce provenance, independence, ceiling | Metrology/reference authority passes | scientific audit report | conversion, missing provenance, or scalar leakage fails |
| AC10-18 | Candidate, release, states, tags, registries, and P2 result | F5 candidate/release/state/tag/registry records and the P2 result | Enforce F4/F5 order, bindings, state relations, and P2 gate | Release authority passes only after prerequisites | release chronology report | early tag, P2 bypass, or cause mismatch fails |
| AC10-19 | Emergency/review bytes, exact paths, commit, and live remote | emergency/review bytes, exact paths, commit/tree, and live remote | Run the ten-step emergency path and tree-byte verifier | Fail-closed NOT_ACTIVE path passes | emergency transcript | fallback path or byte/ancestry mismatch fails |
| AC10-20 | Command, argv, report, runtime inputs, and all authority stages | command/argv/report/runtime inputs and preserved authority stages | Derive argv and replay production runner order | Complete DAG is constructible | DAG transcript | future-file or self-Git edge fails |
| AC10-21 | 15 metrics, 4/1/4/6 partition, five thresholds, evidence, and relations | 15 ordered metrics, 4/1/4/6 membership, five thresholds, and trust source | Recompute order, mappings, thresholds, evidence, and 15 relations | Healthy monitoring PASS and exact trust source pass | monitoring report | wrong count/order/source/threshold/relation fails |
| AC10-22 | Campaign-abandonment incident, manifest, review, and registry record | campaign incident, review bundle, manifest, and registry record | Construct incident before review; verify target and no reverse pointer | Acyclic campaign path passes | campaign report | review field in incident or future target fails |
| AC10-23 | R10 matrix owner_decision_ids | R10 matrix owner_decision_ids and all current mapping text | Derive union and scan for a second current source | Exact F-OD-01..20 union passes | OD coverage report | missing/extra OD or second source fails |
| AC10-24 | Full release set, chronology, and object checks/copies | 21 expected identities, incident/monitoring/state records, and copy checks | Perform exact-set equality, de-duplication, status, acceptance, copy checks | Release retention PASS | release-retention report | missing authority, duplicate identity, or invalid copy fails |
| AC10-25 | Complete literal R10 retention KAT | 22 literal byte rows, 21 expected identities, 42 copy rows, exact times, and the KAT profile | Hash every byte row; derive set; compare copies and count | KAT PASS with 21 members and 42 PASS copies | R10 KAT transcript | any derivation, hash, length, URI, chronology, or coverage discrepancy is NO-GO |

## 49. Historical R10 test procedures

Every current test has complete fixture construction, exact invocation,
expected exact result, and a negative mutation. Exact PASS claims use the
literal values in §§44-46 or the preserved exact contracts in §§2-15.

| test ID | complete fixture construction | exact invocation | expected exact result | negative mutation |
|---|---|---|---|---|
| T10-CX-01 | I2 has only terminal IR2-R and no IR2-C; other positive literals unchanged | incident-chain validator at audited_at 2026-01-01T02:00:00Z | PLAN FIXTURE FAILURE | add IR2-C number 0 contained with previous null -> PASS |
| T10-CX-02 | I2 open; IR2-C number 0 contained/previous null at 00:30; IR2-R number 1 resolved/previous f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c at 01:00; both registry records before 02:00 | incident-resolution validator plus audited-at classifier | PASS; I2 RESOLVED and both resolution identities retained | change predecessor or make either registry record late -> FAIL |
| T10-CX-03 | Use CX-02; move IR2-R effective_at and registry created_at to 2026-01-01T02:01:00Z | audited-at classifier | I2 UNRESOLVED; IR2-C latest eligible | move IR2-R back to 01:00 -> RESOLVED |
| T10-CX-04 | Use CX-02 and append resolution number 2 after terminal IR2-R | resolution-chain validator | FAIL; terminal successor rejected | remove successor -> PASS |
| T10-CX-05 | R10_RETENTION_KAT_F0 says backup_copy_count=1 but positive copy table omits that literal | KAT completeness validator | fixture incompleteness / FAIL | restore literal profile and two copy rows -> PASS |
| T10-CX-06 | Profile backup_copy_count=1; every member has copy-A at 01:15 and copy-B at 01:30 | copy count/freshness validator | PASS | remove either copy -> FAIL |
| T10-CX-07 | For manifest-M keep SHA df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 but change copy-A length 33 to 32 | copy hash/length validator | FAIL | restore length 33 -> PASS |
| T10-CX-08 | Make manifest-M copy-B URI equal to fixture+sha256://copy-a/df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 | URI distinctness validator | FAIL | restore copy-B URI -> PASS |
| T10-CX-09 | Set copy-A verified_at to 2026-01-01T01:00:00Z while audited_at remains 02:00:00Z | freshness validator | FAIL; exact age 3600 and policy is strict less-than | restore 01:15:00Z -> PASS |
| T10-CX-10 | Set copy-A verified_at to 2026-01-01T01:15:00Z while audited_at remains 02:00:00Z | freshness validator | PASS; age 2700 seconds | set time to 01:00:00Z -> FAIL |
| T10-CX-11 | Set PhaseFChainOfCustodyV1 catalog category to NESTED_WIRE | catalog category audit | FAIL | set TOP_LEVEL_WIRE -> PASS |
| T10-CX-12 | Set PhaseFCheckerReportV1 category to TOP_LEVEL_WIRE | catalog category audit | PASS | set NESTED_WIRE -> FAIL |
| T10-CX-13 | Put the literal banned phrase §2–§15 exact closure; unchanged by R9 in a current row | current-catalog banned-string lint | FAIL | replace with exact definition pointer -> PASS |
| T10-CX-14 | Set incident record/resolution traceability to monitoring-only | matrix inverse projection audit | catalog/matrix derivation FAIL | derive incident and retention rows -> PASS |
| T10-CX-15 | Project every current catalog row from all matrix rows containing that schema and sort IDs | matrix inverse projection audit | PASS | delete or add one projected ID -> FAIL |
| T10-CX-16 | Hash all 22 literal byte rows as UTF-8 r10-retention-fixture/LABEL plus LF; verify every SHA and length | independent R10 KAT byte/hash audit | PASS | mutate one byte row -> FAIL |
| T10-CX-17 | Change resolution-I2-C SHA to 64 zeroes | R10 KAT byte/hash audit | FAIL | restore f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c -> PASS |
| T10-CX-18 | Change literal expected retention count 21 to 20 | R10 KAT cardinality audit | FAIL | restore literal count 21 -> PASS |
| T10-CX-19 | Delete any literal byte, length, SHA, URI, time, or policy input from the positive KAT | R10 KAT completeness audit | FAIL; incomplete fixture | restore deleted literal -> PASS |
| T10-CX-20 | Keep 91 identifiers and metadata but set one category wrong | catalog set and metadata audit | FAIL | restore actual category -> PASS |
| T10-CX-21 | Make claim-S0 and claim-S1 the same claim_state kind and SHA; retain one object-check row | retention identity de-duplication validator | PASS; one state identity | change claim-S1 SHA -> two identities and FAIL |
| T10-CX-22 | MR1 is registered before audited_at, accepted PASS, window_end=2026-01-01T01:00:00Z, and all 15 evidence bindings pass | accepted monitoring membership validator | PASS; MR1 included | omit MR1 from the exact set -> FAIL |
| T10-CX-23 | MR2 is registered before audited_at but has result=SUSPEND and an otherwise valid attestation | accepted monitoring membership validator | PASS release set excludes MR2 | mutate result to accepted PASS -> included; omit exclusion rule -> FAIL |
| T10-CX-24 | I1 is open, detected and registered before audited_at, and has no eligible resolution | audited-at incident classifier | PASS; incident_record(I1) retained | omit incident_record(I1) -> FAIL |
| T10-CX-25 | I2 has only IR2-C number 0 contained with previous null and effective_at=2026-01-01T00:30:00Z | audited-at incident classifier | PASS; I2 UNRESOLVED | add valid IR2-R -> RESOLVED |
| T10-CX-26 | I2 has IR2-C then IR2-S number 1 superseded with predecessor SHA f78e0507178453370a1e7e6f4dc93cf7e0133f2ef4f469a3e59d7320d1b6922c | incident-resolution chain validator | PASS; I2 RESOLVED/SUPERSEDED | change first status to superseded -> FAIL |
| T10-CX-27 | IR2-R number 1 has previous_resolution_sha256=0000000000000000000000000000000000000000000000000000000000000000 | incident-resolution chain validator | FAIL; predecessor mismatch | restore literal IR2-C SHA -> PASS |
| T10-CX-28 | Delete protocol-P from the campaign identity set while its manifest binding remains present | campaign exact-set validator | FAIL; missing retention identity | restore protocol-P membership -> PASS |
| T10-CX-29 | Add excluded incident-I2 to the expected retention identity set | campaign/release exact-set validator | FAIL; extra retention identity | remove incident-I2 -> PASS |
| T10-CX-30 | Keep manifest-M URI and length but replace copy-A SHA with 64 zeroes | retention copy hash validator | FAIL | restore manifest-M SHA -> PASS |
| T10-CX-31 | Remove copy-B for one expected member while backup_copy_count remains 1 | retention copy count validator | FAIL; only one valid copy | restore copy-B -> PASS |
| T10-KAT-RETENTION | Use profile, 22 byte rows, 21 expected identities, 42 copy rows, complete IR2-C/IR2-R chain, and exact times | independent ReleaseRetentionSetV1 derivation, hash/length audit, and copy verifier | PASS; expected cardinality 21 | mutate any identity/hash/length/URI/time/chain/count -> NO-GO |
| T10-POS-RETENTION | Use complete R10 positive release-retention KAT in §§44-46 | release retention derivation and copy coverage validator | PASS | omit one expected member -> FAIL |
| T10-POS-TRUST | Use preserved exact annotated trust-tag-message contract and trust source identity | trust tag resolver and monitoring source validator | PASS | supply JSON trust object -> FAIL |
| T10-POS-PLAN | Use external X and exact future tag body/target/plan bytes | plan-tag validator | PASS | target R9 predecessor instead of X -> FAIL |
| T10-POS-DAG | Use preserved §§2-15 authority fixtures in production runner order | positive-path DAG replay and preservation validator | PASS; COMPLETE_VALID_DAG_CONSTRUCTIBLE=yes | add future-file or self-Git edge -> NO-GO |
| T10-CAT | Use all 91 current identifiers and one catalog row per identifier | catalog set/category/metadata audit | PASS; 91=91 and all catalog counters zero | delete, duplicate, or blank a row -> FAIL |
| T10-TRACE | Use R10 matrix, derived catalog, AC, test, and F-EV tables | bidirectional traceability and OD union audit | PASS; all unmapped and contradiction counters zero | add a second current mapping source -> FAIL |

The exact positive backup test is backup_copy_count=1 plus two fresh distinct
copies -> PASS; one fresh copy -> FAIL. The exact time boundary is 2700
seconds -> PASS and 3600 seconds -> FAIL.
RELEASE_RETENTION_TEST_COVERAGE_GAPS=0.

## 50. Historical R10 F-EV evidence oracles

These rows describe future REAL artifacts and are not created by this
planning-only edit. No row claims release-retention PASS solely from the plan
KAT. The incident oracle requires the complete contained-before-terminal chain
when a terminal status is asserted. The copy oracle requires exact URI,
length, SHA, retrieved bytes, verified_at, count, and freshness.

| F-EV ID | future real artifact | producer/authority | immutable identity | acceptance/review oracle |
|---|---|---|---|---|
| EV10-01 | future plan review bundle, annotated plan tag, and plan bytes at external X | independent reviewer and Git tag validator | review-bundle SHA; peeled X; plan SHA-256 and Git blob | target/body equality and plan-byte equality |
| EV10-02 | actual trust-provisioning annotated-tag message and monitoring source copies | independent trust gate, tag validator, and operations authority | exact tag-message SHA and length; source reference | tag prerequisites, exact bytes, TAG_BODY parse, trust-field comparison |
| EV10-03 | actual package manifest, manifest objects, and campaign audit | campaign authority and retention auditor | manifest/object complete hashes and kind/ID/SHA set | exact campaign membership and no locator derivation |
| EV10-04 | actual release/cohort/execution/owner/policy/state/review authorities | release and registry authorities | exact kind/SHA identities from source bindings | static derivation and exact-set oracle |
| EV10-05 | actual incident and complete contained-before-terminal resolution files plus registry records | operations/governance and registry authorities | incident/resolution complete-file hashes and registry sequence | complete chain, effective_at, created_at, and audited_at classification |
| EV10-06 | actual registered monitoring records and accepted PASS windows | operations and registry authorities | monitoring subject complete-file hashes | accepted PASS, attestation, window_end, due-chain, and cutoff |
| EV10-07 | actual retention copy references, retrieved bytes, and retrieval/hash transcripts | retention auditor and copy retriever | each immutable URI, byte length, SHA, verified_at | scheme, availability, byte/hash equality, freshness, distinctness, count |
| EV10-08 | complete final plan and all current catalog rows | plan author and independent reviewer | final plan SHA/blob and catalog row identities | regex/set equality, category/metadata completeness, pointers, duplicates |
| EV10-09 | current catalog metadata and exact definition locations | plan author and independent reviewer | row bytes and definition section IDs | no generic phrase, blank cell, or broad pointer |
| EV10-10 | current R10 matrix and derived catalog cells | plan author and independent reviewer | matrix row bytes and sorted projection cells | bidirectional resolution and exact inverse projection |
| EV10-11 | F0 decision bundle and runtime projection | F0 authority and compatibility reviewer | decision bundle/projection complete hashes | exact 20 decisions and projection |
| EV10-12 | two build transcripts, readiness, enrollment, signed genesis/head | checker, governance, and registry authorities | complete-file hashes, signatures, sequence | readiness/enrollment/registry validation |
| EV10-13 | retrieved external objects, package manifest, dependency audit | retrieval/package authorities | URI/length/SHA plus complete package hashes | retrieval, classification, dependency, relation oracle |
| EV10-14 | physical unit/location/custody ledgers and comparison/audit records | campaign, laboratory, and custody authorities | complete ledger/audit hashes and native identities | no alias, pseudoreplication, discontinuity, post-destroy use |
| EV10-15 | deviation revisions and event records | campaign/deviation authority | stable ledger ID and revision complete SHA | immutable predecessor and action compatibility |
| EV10-16 | power interface, typed analysis, sensitivity cases, and review | statistician and independent scientific reviewers | content IDs and reviewed complete-file hashes | range/unit/method equality and pre-registration |
| EV10-17 | metrology policy/checks and reference source/result/admissibility files | metrology laboratory and runtime authority | policy/result/source/audit hashes | endpoint lookup, exact math, provenance, independence, ceiling |
| EV10-18 | F5 candidate, release, state chain, approval tags, and registry records | release authority and independent reviewers | candidate/release/state/tag bytes and registry hashes | F4/F5 order, P2 gate, binding oracle |
| EV10-19 | emergency/review files and later full-prefixed Git tree | security authority, independent reviewers, and Git | emergency/review/file/tree hashes and commit SHA | exact ten-step path, ancestry, byte equality |
| EV10-20 | command, argv, report, runtime inputs, and construction transcript | checker and compatibility authorities | command/report and input hashes | exact runner order and no cycle |
| EV10-21 | monitoring policy, thresholds, 15 measurements/evidence files, and relations | F0 owner, operations, and registry authorities | policy/record/evidence/relation hashes | fixed order, 4/1/4/6, five thresholds, trust source |
| EV10-22 | campaign-abandonment incident, review bundle, manifest, and registry record | campaign operator, independent reviewers, and registry authority | incident/review/registry complete hashes | incident-first target and no reverse pointer |
| EV10-23 | current R10 matrix and owner-decision cells | plan author and independent reviewer | matrix bytes and plan SHA/blob | union exactly F-OD-01..20 and one current mapping source |
| EV10-24 | complete release retention audit and all object checks/copies | retention authority and independent auditor | audit complete SHA; identity and copy hashes | exact set, de-duplication, chronology, and copy SHA/length/URI/freshness/count |
| EV10-25 | R10 full positive KAT transcript and independently derived expected set | independent plan reviewer and retention auditor | fixture hashes, copy transcript, and audit identity | reproduce exact 21-member set and 42 PASS copies; KAT never substitutes for real evidence |

RELEASE_RETENTION_EVIDENCE_ORACLE_GAPS=0.

## 51. Historical R10 remediation ledger and author audit

Exactly six R10 remediation IDs are created. Author disposition is only
REMEDIATED or OPEN; only a fresh independent R10 reviewer may mark a finding
CLOSED.

| R9 rereview P1 group | R10 exact section | root cause | exact remediation | current R10 requirements | ACs | tests | F-EVs | AUTHOR DISPOSITION |
|---|---|---|---|---|---|---|---|---|
| F-PLAN-R10-P1-01 | §§45-46 | Positive incident fixture used a terminal alias and omitted the required contained predecessor. | Literal I2 -> IR2-C number 0 contained -> IR2-R number 1 resolved chain, predecessor SHA, times, registry records, and exact membership. | R10-05,R10-24,R10-25 | AC10-05,AC10-24,AC10-25 | T10-CX-01,T10-CX-02,T10-CX-03,T10-CX-04,T10-CX-18,T10-KAT-RETENTION | EV10-05,EV10-24,EV10-25 | REMEDIATED |
| F-PLAN-R10-P1-02 | §§44-46 | Positive copy KAT was symbolic and delegated bytes, locators, lengths, times, and policy to invention. | Explicit test-only profile, 22 literal byte rows, 21 expected members, 42 literal PASS rows, exact times and count. | R10-07,R10-24,R10-25 | AC10-07,AC10-24,AC10-25 | T10-CX-05,T10-CX-06,T10-CX-07,T10-CX-08,T10-CX-09,T10-CX-10,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-KAT-RETENTION | EV10-07,EV10-24,EV10-25 | REMEDIATED |
| F-PLAN-R10-P1-03 | §42.2 | Three independently serialized artifacts were classified as NESTED_WIRE. | Set ChainOfCustody, CheckerReport, and RetrievalVerification to TOP_LEVEL_WIRE and audit all remaining identifiers. | R10-08 | AC10-08 | T10-CX-11,T10-CX-12,T10-CX-20,T10-CAT | EV10-08 | REMEDIATED |
| F-PLAN-R10-P1-04 | §42.2 | 73 rows used generic closure, producer, validator, stage, or registry metadata. | Give all 91 rows one exact definition pointer and concrete metadata; lint banned phrases. | R10-09 | AC10-09 | T10-CX-13,T10-CX-19,T10-CX-20,T10-CAT | EV10-09 | REMEDIATED |
| F-PLAN-R10-P1-05 | §§42.2-43 | Catalog traceability was independently authored and misrouted incident, monitoring-source, and retention-audit rows. | Make §43 the only current mapping source and derive every catalog cell from it. | R10-10 | AC10-10 | T10-CX-14,T10-CX-15,T10-TRACE | EV10-10 | REMEDIATED |
| F-PLAN-R10-P1-06 | §§45-49 | Deterministic PASS claims relied on incomplete incident/copy fixtures and incomplete real-artifact oracles. | Add separate retention tests, 21-member KAT, 42 copy rows, and future real-artifact F-EV oracles. | R10-05,R10-06,R10-07,R10-24,R10-25 | AC10-05,AC10-06,AC10-07,AC10-24,AC10-25 | T10-CX-01,T10-CX-02,T10-CX-03,T10-CX-04,T10-CX-05,T10-CX-06,T10-CX-07,T10-CX-08,T10-CX-09,T10-CX-10,T10-CX-16,T10-CX-17,T10-CX-18,T10-CX-19,T10-KAT-RETENTION | EV10-05,EV10-06,EV10-07,EV10-24,EV10-25 | REMEDIATED |

The following counters are author audit results after the constructive and
mechanical checks above. They are not independent approval:

~~~text
INCIDENT_POSITIVE_FIXTURE_CONSISTENT=yes
INCIDENT_COUNTEREXAMPLE_SEMANTIC_GAPS=0
RETENTION_POSITIVE_KAT_FULLY_MATERIALIZED=yes
RETENTION_KAT_SYMBOLIC_BYTE_VALUES=0
RETENTION_KAT_SYMBOLIC_COPY_VALUES=0
RELEASE_RETENTION_TEST_COVERAGE_GAPS=0
RELEASE_RETENTION_EVIDENCE_ORACLE_GAPS=0
CATALOG_CATEGORY_MISMATCHES=0
CATALOG_FIELD_CLOSURE_AMBIGUITIES=0
CATALOG_PRODUCER_AMBIGUITIES=0
CATALOG_VALIDATOR_AMBIGUITIES=0
CATALOG_STAGE_AMBIGUITIES=0
CATALOG_REGISTRY_BEHAVIOR_AMBIGUITIES=0
CATALOG_GENERIC_METADATA_AMBIGUITIES=0
CATALOG_TRACEABILITY_DERIVATION_MISMATCHES=0
INCOMPLETE_SCHEMA_CATALOG_ROWS=0
INCORRECT_SCHEMA_CATALOG_ROWS=0
CONTRADICTORY_CURRENT_TRACEABILITY_TABLES=0
TRACEABILITY_SUBSTANCE_GAPS=0
CURRENT_TEST_PROCEDURE_GAPS=0
CURRENT_EVIDENCE_ORACLE_GAPS=0
POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
NORMATIVE_CONTRADICTIONS=0

synthetic-to-physical=0
constructed-to-physical=0
unknown-to-physical=0
test-to-physical=0
same-source independence=0
undeclared-dependency independence=0
pseudoreplication=0
underpowered-to-pass=0
F4-to-active=0
compromised-authority bypass=0
private-key repository paths=0
Phase-E compatibility=CLOSED
production runner order=PASS
P2 gate=PASS
~~~

## 52. Historical R10 baseline, frozen authority, and handoff

Before and after this planning-only edit, run:

~~~text
git diff --check
cargo fmt --all --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --test phase_e_validation
cargo test --locked --test phase_d_reporting_public_output
~~~

Required results are all PASS, Phase E 38/38, Phase D 73/73, and zero strict
Clippy diagnostics. Frozen Phase-E SHA-256 remains
0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33 and its
Git blob remains 6fce9d13a42a09027e0e730874a8d80e03e6a7da.

~~~text
MHI V1 PHASE F
R10 PLANNING REMEDIATION HANDOFF

STARTING R9 SHA: 1084257636e74d16d1f64da1062bb187e58b46f6
R9 PLAN SHA-256: 2b0c79858e8f82b4ae756555d69ba5caa12cc1cce9a5fb13bd300d1e1c755adb
R9 PLAN GIT BLOB: 304ce6c4931ae1b105829abca64b559e98916d01
R10 PLAN REVIEW SHA: <externally frozen after final planning-only commit>
R10 PLAN SHA-256: <computed after final R10 bytes>
R10 PLAN GIT BLOB: <computed after final R10 bytes>
CHANGED FILES: 1 expected

F-PLAN-R10-P1-01: REMEDIATED
F-PLAN-R10-P1-02: REMEDIATED
F-PLAN-R10-P1-03: REMEDIATED
F-PLAN-R10-P1-04: REMEDIATED
F-PLAN-R10-P1-05: REMEDIATED
F-PLAN-R10-P1-06: REMEDIATED

contained predecessor present: yes
terminal resolution present: yes
both resolution objects retained: yes
fixture consistent: yes
test F0 backup count: 1
test F0 interval: 3600
audited_at: 2026-01-01T02:00:00Z
copy-A verified_at: 2026-01-01T01:15:00Z
copy-B verified_at: 2026-01-01T01:30:00Z
literal byte rows: 22
literal SHA rows: 22
literal byte-length rows: 22
literal URI rows: 42
symbolic placeholders remaining: 0
expected retention-member count: 21
KAT PASS constructible: yes

normative identifier count: 91
catalog identifier count: 91
missing: 0
extra: 0
duplicates: 0
category mismatches: 0
field-closure ambiguities: 0
producer ambiguities: 0
validator ambiguities: 0
stage ambiguities: 0
registry-behavior ambiguities: 0
generic metadata rows: 0
incorrect rows: 0

current R10 requirements: 25
ACs: 25
tests: 38
evidence: 25
owner decisions: 20
unmapped requirements: 0
unmapped ACs: 0
unmapped tests: 0
unmapped evidence: 0
unmapped ODs: 0
catalog traceability derivation mismatches: 0
contradictory current mapping tables: 0
traceability substance gaps: 0
test-procedure gaps: 0
evidence-oracle gaps: 0
complete DAG constructible: yes
construction ambiguities: 0

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
READY_FOR_FRESH_PHASE_F_R10_PLAN_REREVIEW: yes
READY_FOR_PHASE_F_PLAN_APPROVAL_TAG: NO expected pending fresh R10 GO
READY_FOR_PHASE_F_IMPLEMENTATION: NO
~~~

R10 requires a NEW independent reviewer to begin with six positive checks:
the complete I2 chain and both retained resolutions; literal byte/SHA/length/
URI/time/count reproducibility; every actual catalog category; exact catalog
metadata; matrix-inverse traceability; and complete test/evidence closure.
Failure of any is P1. R10 remains unapproved pending that rereview. No
Phase-F approval tag, implementation branch, F0 activity, key, signature,
trust provisioning, registry record, physical evidence, monitoring evidence,
claim, or production change is authorized by this plan.

## 53. R11 current authority, layer separation, and status

Sections 16-52 are historical R7/R8/R9/R10 accounting and are not current
authority. Sections 2-15 remain the closed Phase-F primitive, serialization,
identity, registry, scientific, monitoring, incident, retention, and safety
contracts. Sections 53 onward are the one current R11 planning authority.
R11 changes no authority architecture, scientific scope, production runtime,
checker implementation, F0 decision, monitoring contract, or Phase-E
compatibility. It remains planning only; no approval tag, implementation
branch, F0 activity, key, signature, trust provisioning, registry record,
physical evidence, monitoring evidence, claim, or production artifact is
created by this edit.

The required R11 status history is: R1 NO-GO/P1=13; R2 NO-GO/P1=10;
R3 NO-GO/P1=19; R4 NO-GO/P1=14; R5 NO-GO/P1=11; R6 NO-GO/P1=13;
R7 NO-GO/P1=5; R8 NO-GO/P1=4; R9 NO-GO/P1=6 grouped; R10 NO-GO/P1=4/P3=1;
R11 forward remediation; R11 independent rereview PENDING. No rejected revision
is approved. Author dispositions below are only REMEDIATED or OPEN. Only a
fresh independent R11 reviewer may close an R11 P1 finding.

### 53.1 Exact KAT class enum and three test layers

The plan-only enum is exactly:

~~~text
R11_KAT_CLASS_V1 =
  production_schema_kat
  retention_storage_kat
  property_test
  constructive_plan_audit
~~~

Every current R11 test row has kat_class. The meanings are normative:

1. production_schema_kat tests actual PhaseF serialization, semantic IDs,
   canonical bytes, schema fields, and schema semantics.
2. retention_storage_kat tests an already validated kind/SHA retention identity
   and its copies: URI, SHA, byte length, availability, freshness, count, and
   set equality. It does not prove that upstream production objects are
   schema-valid.
3. property_test tests a logical invariant over symbolic variables.
4. constructive_plan_audit tests explicit plan-level structures such as an
   adjacency list, identifier set, matrix rows, or test-row input sets. It does
   not instantiate Phase-F production objects.

No current test may silently move between layers. A literal PASS or FAIL in a
production_schema_kat or retention_storage_kat has all validator-required
inputs in the row or in the explicitly named prevalidated-input contract.

### 53.2 Storage-only retention fixture

R11RetentionStorageFixtureV1 is a plan-only storage-copy contract, not a
production wire schema. It is used only after upstream retention membership
identity has been validated. Its exact conceptual fields are:

| field | exact meaning |
|---|---|
| fixture_label | test label for one storage payload |
| assumed_object_kind | prevalidated retention object kind |
| assumed_object_id_or_null | package object ID, or null for authority identity |
| assumed_prevalidated_object_sha256 | test input to the isolated copy validator |
| storage_bytes | opaque bytes at the storage location |
| storage_byte_length | exact length of storage_bytes |
| copy_a_uri | first immutable storage URI |
| copy_b_uri | second distinct immutable storage URI |
| copy_a_verified_at | verification timestamp for copy A |
| copy_b_verified_at | verification timestamp for copy B |

The assumed_prevalidated_object_sha256 field is TEST INPUT to the isolated
retention copy validator. The contract does not assert that storage_bytes parse
as PhaseFReleaseRecordV1, PhaseFMonitoringRecordV1, PhaseFClaimStateRecordV1,
PhaseFPowerAnalysisRecordV1, or any other production schema.

The `r11-retention-storage/<LABEL>\n` byte fixtures are opaque STORAGE-COPY test
payloads used only after a kind/SHA retention identity has been supplied by an
upstream prevalidated-identity fixture.

They do not establish schema validity, semantic-ID validity, registry validity,
release validity, monitoring validity, incident validity, physical evidence
validity, or claims. Their only tested predicates are SHA equality, length
equality, URI scheme, copy distinctness, availability, verification freshness,
copy cardinality, and retention set equality.

RETENTION_KAT_BYTE_SCOPE=STORAGE_ONLY.
STORAGE_FIXTURE_TO_SCHEMA_BYTE_PROMOTION_PATHS=0.

### 53.3 Retention composition and prevalidated identities

The retention composition theorem is:

IF A. upstream current Phase-F validators have already produced a valid
ReleaseRetentionSetV1 kind/SHA identity set; AND B. incident membership has
been established by the real incident schema KAT; AND C. the retention
storage/copy KAT validates exact storage copies; THEN the retention
copy/coverage layer PASS condition is constructible. This theorem does not
substitute for upstream production validation.

The campaign set remains exactly the package manifest plus every package object.
Protocol-P is a STATIC RELEASE AUTHORITY, not a campaign-set member. Power
analysis, cohort lock, owner approval, execution, release, validation manifest,
monitoring policy, trust tag, F5 review, and claim state are static release
authorities or release bindings; none is a campaign member.

| identity key | object kind | object ID if package object | prevalidated SHA-256 | upstream validator responsible |
|---|---|---|---|---|
| manifest-M | package_manifest | null | df963ee5224c52c91bcb5f4ec0aa2ff5def708c2989f6eb592dcb269311eef98 | package manifest strict parser plus campaign-set validator |
| package-O1 | package_object | O1 | 90ee35572a33af574c0c8fdb137aee2a7b6d325a7cc60efb36c942fa2d11065f | package object strict parser plus manifest binding validator |
| package-O2 | package_object | O2 | 24050263ca4a8ec011c7d718e1d0fc52b2350d67265b925b636b75f0a87374ee | package object strict parser plus manifest binding validator |
| protocol-P | protocol | null | 73fb452c36f96f4db361f5bbca287c20f6a899296db86ec064992de91880fcd3 | protocol registration plus release-binding validator |
| power-A | power_analysis | null | 380d4d36ac0f8bae7d2558eb22e7003739b86f2b2222d9f507fbf3967672a58a | power-analysis parser plus cohort binding validator |
| cohort-C | cohort_lock | null | 9d75af9d77ddc8ff159d961e0f35ac8b1d3eee6e412e007b90faf49161a6b196 | cohort strict parser plus lock/release binding validator |
| owner-OA | owner_approval | null | d06f4b22f31603f062847547a6aefeac400d6556a392df1a499b78c0eb031dc4 | certified owner-approval parser plus release binding validator |
| execution-E | execution_record | null | 4e13d9bc0c22f4acd5d5ae6e9f500d143dd17ca40f15c6719830215914b59876 | execution strict parser plus release binding validator |
| release-R | release_record | null | e136326de2b7a883df571265505752f7600422519de2ce823ca322633c1cd856 | release strict parser plus release_registered subject validator |
| validation-VM | validation_manifest | null | a2e77f55d1a719a28db66e299f9500dd7c0e826cf86f9ebf50f021c2b3ebc8e7 | validation-manifest binding validator |
| monitoring-policy-MP | monitoring_policy | null | 22b90909658de8d2fead19836e5b71d3bc8e92ac6ef941434cc7b0bac19be6fb | monitoring policy strict parser plus release binding validator |
| metrology-policy-MET | metrology_policy | null | e592b9ff6c5ec6890910097154bbfed958cf9383651aed3f41072ef208cc3f64 | metrology policy strict parser plus release binding validator |
| trust-tag-TT | git_tag_message | null | 09c14e2106d90c93d25b3c2793d97f94ec74e87f77a817d004bca714723113f9 | annotated-tag parser plus trust-field binding validator |
| f5-review-RB | independent_review_bundle | null | 62dbce3dc5b0203ad287599e35b246ae81710859535b4486de3e7126637ad56f | review-bundle parser plus F5 target/aggregate validator |
| claim-S0 | claim_state | null | 7b6387ff4259fafe1418268a9c80c9aa6811c26e744f629e92f3355ea2b6d69e | claim-state parser plus initial-state registry validator |
| claim-S1 | claim_state | null | 390de3582256a63f592d43f418927c80828947a127d21eb5ef42eb760ae50003 | claim-state parser plus state-chain registry validator |
| monitor-MR1 | monitoring_record | null | d38e0ed279e3ce7c601a4ca25e46d7a7489268d911c904bed43e2a2159d9e035 | monitoring parser plus accepted-window registry validator |
| monitor-MR2 | monitoring_record | null | 6a2ddbc9c8ab34f013ea6202a5b062e6a892e247cb6c1a2ef6f50f91b0c0c524 | monitoring parser plus accepted-window registry validator |
| incident-I1 | incident_record | null | 98d48c525f84456d66549129982203dd903747d05eac2a55d03e8137d9e8976d | incident parser plus audited-at classifier |
| IR2-C | incident_resolution | null | e1213d5261a13111eb857c401009cbf247662d1f28a5abf1f28c90aa0cd6cccf | incident-resolution schema KAT plus chain validator |
| IR2-R | incident_resolution | null | 66464841fd24cf4f5de17fd731dbaac7edb7c820f8885fb4dbf417bb77dd1b3c | incident-resolution schema KAT plus chain validator |

Only the first three rows are campaign identities. The remaining rows are
static release identities, accepted monitoring identities, incident I1, and
the two resolution authorities. A static identity is assumed valid only under
this named table; its opaque storage copy never promotes into a PhaseF schema
object.

### 53.4 Literal production-schema incident KAT

The following are literal JCS JSON bytes. They are production_schema_kat inputs
for PhaseFIncidentRecordV1 and PhaseFIncidentResolutionV1, not production
evidence. Every evidence reference is literal and fully bound by URI, SHA, and
byte length.

I2 uses the exact domain separator `mhi_phase_f_incident_record_v1\0` and
excludes only `incident_id` from its semantic payload. IR2-C and IR2-R use the
exact domain separator `mhi_phase_f_incident_resolution_v1\0` and exclude only
their own `incident_resolution_id`; each semantic ID is `sha256` plus the
lowercase hash of domain bytes followed by the JCS semantic payload. No
registry pointer is included.

I2 semantic ID is
sha256:c53b6f2230cbe25034dfcfe572cb845e1e29fe0a9f730549ed0ec464d71b8353.
I2 canonical JCS bytes are exactly:

~~~json
{"affected_object_sha256s":[{"object_kind":"release_record","object_sha256":"2222222222222222222222222222222222222222222222222222222222222222"}],"affected_unit_ids":["unit-I2"],"detected_at":"2026-01-01T00:10:00Z","evidence_references":[{"byte_length":"97","immutable_uri":"https://r11.invalid/evidence/i2.json","sha256":"3333333333333333333333333333333333333333333333333333333333333333"}],"incident_id":"sha256:c53b6f2230cbe25034dfcfe572cb845e1e29fe0a9f730549ed0ec464d71b8353","incident_status":"open","incident_type":"data_integrity","required_action":"suspend","schema_version":1,"scope":{"release_record_id":"sha256:1111111111111111111111111111111111111111111111111111111111111111","type":"release"}}
~~~

I2 byte length is 704 and complete-file SHA-256 is
3a6281cddf82f4f0b69abf6aff8b15347aae8c71af46d252f93c012f00b0d55f.

IR2-C has semantic ID
sha256:4f0846d55e1f38335cc8e1a62f963e95c374c6351f2c9358143e673ec76a7dc7.
Its canonical JCS bytes are exactly:

~~~json
{"effective_at":"2026-01-01T00:30:00Z","evidence_references":[{"byte_length":"101","immutable_uri":"https://r11.invalid/evidence/ir2-c.json","sha256":"4444444444444444444444444444444444444444444444444444444444444444"}],"incident_id":"sha256:c53b6f2230cbe25034dfcfe572cb845e1e29fe0a9f730549ed0ec464d71b8353","incident_record_sha256":"3a6281cddf82f4f0b69abf6aff8b15347aae8c71af46d252f93c012f00b0d55f","incident_resolution_id":"sha256:4f0846d55e1f38335cc8e1a62f963e95c374c6351f2c9358143e673ec76a7dc7","previous_resolution_sha256":null,"resolution_number":"0","resolution_status":"contained","schema_version":1}
~~~

IR2-C byte length is 607 and complete-file SHA-256 is
e1213d5261a13111eb857c401009cbf247662d1f28a5abf1f28c90aa0cd6cccf.

IR2-R has semantic ID
sha256:42e6ff499eac64a5abb5858a4eb2ac260a2d833ebcaf39bea65f7f10579069b9.
Its canonical JCS bytes are exactly:

~~~json
{"effective_at":"2026-01-01T01:00:00Z","evidence_references":[{"byte_length":"101","immutable_uri":"https://r11.invalid/evidence/ir2-r.json","sha256":"5555555555555555555555555555555555555555555555555555555555555555"}],"incident_id":"sha256:c53b6f2230cbe25034dfcfe572cb845e1e29fe0a9f730549ed0ec464d71b8353","incident_record_sha256":"3a6281cddf82f4f0b69abf6aff8b15347aae8c71af46d252f93c012f00b0d55f","incident_resolution_id":"sha256:42e6ff499eac64a5abb5858a4eb2ac260a2d833ebcaf39bea65f7f10579069b9","previous_resolution_sha256":"e1213d5261a13111eb857c401009cbf247662d1f28a5abf1f28c90aa0cd6cccf","resolution_number":"1","resolution_status":"resolved","schema_version":1}
~~~

IR2-R byte length is 668 and complete-file SHA-256 is
66464841fd24cf4f5de17fd731dbaac7edb7c820f8885fb4dbf417bb77dd1b3c.
The IR2-R predecessor equals the exact complete canonical IR2-C SHA:
e1213d5261a13111eb857c401009cbf247662d1f28a5abf1f28c90aa0cd6cccf.
INCIDENT_SCHEMA_FIXTURE_SHA_BINDING_AMBIGUITIES=0.

For the incident-resolution retention members, R11RetentionStorageFixtureV1.storage_bytes
is the exact byte-for-byte copy of the corresponding literal IR2-C or IR2-R
canonical JSON above. Therefore those two storage-copy rows have exact
storage-byte lengths and SHA-256 values 607/e1213d5261a13111eb857c401009cbf247662d1f28a5abf1f28c90aa0cd6cccf
and 668/66464841fd24cf4f5de17fd731dbaac7edb7c820f8885fb4dbf417bb77dd1b3c.
All other retention-storage rows remain STORAGE_ONLY opaque payloads.

The incident KAT has no opaque predecessor. I2 is open, IR2-C is number
0/contained, and IR2-R is number 1/resolved. The two resolution objects are
retained for chain proof; incident-I2 is not retained as an unresolved member.

### 53.5 Exact plan and trust parser KATs

R11-POS-PLAN is a literal function-level parser/binding KAT. Let
X=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa be a test commit token, not a Git
claim. The literal peeled_target is X and the literal body bytes are exactly:

~~~text
format_version=1
plan_review_sha=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
plan_sha256=1111111111111111111111111111111111111111111111111111111111111111
plan_git_blob=2222222222222222222222222222222222222222
review_bundle_sha256=3333333333333333333333333333333333333333333333333333333333333333
approval_decision=GO
~~~

The body has byte length 313 and SHA-256
50b16c2f262e84e4ddbedc848eeb2e6df73fcebda19c01f93d6e8d8548e82d77. The
function-level surface is only target/body binding; it does not claim that X
exists in Git. A separate production property requires the named annotated tag
to resolve to X and to carry the same plan bytes before any real approval.

R11-POS-TRUST is a literal PhaseFTrustProvisioningApprovalV1 parser/binding KAT.
The exact ASCII message bytes, including the final LF, are:

~~~text
format_version=1
phase_f_plan_tag=ism-mechanism-health-v1-f-plan-approved
f0_decisions_tag=ism-mechanism-health-v1-f-f0-decisions-approved
readiness_tag=ism-mechanism-health-v1-f-readiness-approved
authority_enrollment_tag=ism-mechanism-health-v1-f-authority-enrollment-approved
enrollment_sha256=1111111111111111111111111111111111111111111111111111111111111111
owner_public_key_fingerprint=2222222222222222222222222222222222222222222222222222222222222222
registry_public_key_fingerprint=3333333333333333333333333333333333333333333333333333333333333333
trust_root_id=trust-root-r11
trust_review_sha=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
trust_store_git_blob=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
trust_store_sha256=4444444444444444444444444444444444444444444444444444444444444444
f2_cohort_lock_registry_record_sha256=5555555555555555555555555555555555555555555555555555555555555555
review_bundle_sha256=6666666666666666666666666666666666666666666666666666666666666666
approval_decision=GO
~~~

The exact message byte length is 996 and SHA-256 is
9122e2684e6b4084e1b85bfed9d7d5aa3ead291a3cb5727741e9e4cdee8a1c01.
Parsed trust_root_id is trust-root-r11 and parsed trust_store_sha256 is
4444444444444444444444444444444444444444444444444444444444444444. The
monitoring source reference uses object_kind=git_tag_message, this SHA, and
byte_length=996; the two monitoring values equal these parsed literals. The
production property additionally requires the real named annotated tag to
resolve to identical message bytes. The parser KAT does not prove Git
resolution.

### 53.6 Constructive DAG, campaign/static correction, and monitoring scope

R11-DAG-AUDIT uses this explicit node set:

~~~text
{plan_review, plan_tag, f0_bundle, f0_review, f0_tag, readiness, enrollment,
 genesis, protocol, power, package, cohort, owner_approval, trust_tag,
 execution, release, f5_review, claim_state, monitoring, incident_resolution,
 retention}
~~~

Its exact directed edge list is:

~~~text
plan_review->plan_tag
plan_tag->f0_bundle
f0_bundle->f0_review
f0_review->f0_tag
f0_tag->readiness
readiness->enrollment
enrollment->genesis
genesis->protocol
protocol->power
power->package
package->cohort
cohort->owner_approval
owner_approval->trust_tag
trust_tag->execution
execution->release
release->f5_review
f5_review->claim_state
claim_state->monitoring
monitoring->incident_resolution
incident_resolution->retention
~~~

The audit runs deterministic cycle detection, future-edge detection, and
self-Git-edge detection. Expected plan-level result is ACYCLIC. No artifact
fixture is implied by this audit.

T10-CX-28 is replaced by two current R11 cases. Case A removes package-O1
from campaign membership and expects campaign/release retention-set FAIL. Case
B removes protocol-P, which is explicitly the STATIC RELEASE AUTHORITY, and
expects release-retention-set FAIL. Protocol-P is never a campaign member.
Removing protocol-P is a static-authority failure, while removing package-O1
is a campaign-set failure. CAMPAIGN_STATIC_MEMBERSHIP_CONTRADICTIONS=0.

The only current monitoring positive test is a narrow property test for the
metric-to-source-kind mapping. A complete monitoring PASS requires the literal
15-metric policy, all five thresholds, all 15 evidence objects, all 15
measurement rows, release bindings, and registry relations; no current row
claims that full monitoring PASS from an unspecified fixture.

### 53.7 Canonical definition anchors and exact usage matrix

The following is the one canonical definition/index paragraph for distributed
definitions. Each current PhaseF identifier has exactly one HTML definition
anchor and one exact source. The cited source is adjacent to or directly
defines the current normative schema; inline schemas introduced in §10-§15
are canonically named here without creating production files. Catalog pointer
cells use the exact HTML anchor ID, never a section range.

| identifier | canonical definition anchor | exact current definition source |
|---|---|---|
| PhaseFArgvV1 | <a id="schema-def-PhaseFArgvV1"></a>SCHEMA_DEF[PhaseFArgvV1] | §2 closed helper definition |
| PhaseFAuthorityEnrollmentApprovalV1 | <a id="schema-def-PhaseFAuthorityEnrollmentApprovalV1"></a>SCHEMA_DEF[PhaseFAuthorityEnrollmentApprovalV1] | §6 exact annotated-tag body definition |
| PhaseFAuthorityEnrollmentV1 | <a id="schema-def-PhaseFAuthorityEnrollmentV1"></a>SCHEMA_DEF[PhaseFAuthorityEnrollmentV1] | §5.2/§6 authority definition |
| PhaseFChainOfCustodyV1 | <a id="schema-def-PhaseFChainOfCustodyV1"></a>SCHEMA_DEF[PhaseFChainOfCustodyV1] | §11 physical/deviation definition |
| PhaseFCheckListV1 | <a id="schema-def-PhaseFCheckListV1"></a>SCHEMA_DEF[PhaseFCheckListV1] | §2 closed helper definition |
| PhaseFCheckerBuildEvidenceV1 | <a id="schema-def-PhaseFCheckerBuildEvidenceV1"></a>SCHEMA_DEF[PhaseFCheckerBuildEvidenceV1] | §7 checker/readiness definition |
| PhaseFCheckerExitCodeV1 | <a id="schema-def-PhaseFCheckerExitCodeV1"></a>SCHEMA_DEF[PhaseFCheckerExitCodeV1] | §7 checker/readiness definition |
| PhaseFCheckerReadinessEvidenceV1 | <a id="schema-def-PhaseFCheckerReadinessEvidenceV1"></a>SCHEMA_DEF[PhaseFCheckerReadinessEvidenceV1] | §7 checker/readiness definition |
| PhaseFCheckerReportV1 | <a id="schema-def-PhaseFCheckerReportV1"></a>SCHEMA_DEF[PhaseFCheckerReportV1] | §7 checker/readiness definition |
| PhaseFCheckerStdoutV1 | <a id="schema-def-PhaseFCheckerStdoutV1"></a>SCHEMA_DEF[PhaseFCheckerStdoutV1] | §7 checker/readiness definition |
| PhaseFClaimStateRecordV1 | <a id="schema-def-PhaseFClaimStateRecordV1"></a>SCHEMA_DEF[PhaseFClaimStateRecordV1] | §14 release/monitoring definition |
| PhaseFCohortLockRecordV1 | <a id="schema-def-PhaseFCohortLockRecordV1"></a>SCHEMA_DEF[PhaseFCohortLockRecordV1] | §14 release/monitoring definition |
| PhaseFCommandV1 | <a id="schema-def-PhaseFCommandV1"></a>SCHEMA_DEF[PhaseFCommandV1] | §7 checker/readiness definition |
| PhaseFCustodyEventV1 | <a id="schema-def-PhaseFCustodyEventV1"></a>SCHEMA_DEF[PhaseFCustodyEventV1] | §11 physical/deviation definition |
| PhaseFDecisionApprovalV1 | <a id="schema-def-PhaseFDecisionApprovalV1"></a>SCHEMA_DEF[PhaseFDecisionApprovalV1] | §6 exact annotated-tag body definition |
| PhaseFDecisionBundleV1 | <a id="schema-def-PhaseFDecisionBundleV1"></a>SCHEMA_DEF[PhaseFDecisionBundleV1] | §4 F0 decision/projection definition |
| PhaseFDecisionRowV1 | <a id="schema-def-PhaseFDecisionRowV1"></a>SCHEMA_DEF[PhaseFDecisionRowV1] | §4 F0 decision/projection definition |
| PhaseFDecisionValueV1 | <a id="schema-def-PhaseFDecisionValueV1"></a>SCHEMA_DEF[PhaseFDecisionValueV1] | §2 closed helper definition |
| PhaseFDependencyAuditV1 | <a id="schema-def-PhaseFDependencyAuditV1"></a>SCHEMA_DEF[PhaseFDependencyAuditV1] | §10 retrieval/package definition |
| PhaseFDependencyEdgeV1 | <a id="schema-def-PhaseFDependencyEdgeV1"></a>SCHEMA_DEF[PhaseFDependencyEdgeV1] | §10 retrieval/package definition |
| PhaseFDeviationEventV1 | <a id="schema-def-PhaseFDeviationEventV1"></a>SCHEMA_DEF[PhaseFDeviationEventV1] | §11 physical/deviation definition |
| PhaseFDeviationLedgerRevisionV1 | <a id="schema-def-PhaseFDeviationLedgerRevisionV1"></a>SCHEMA_DEF[PhaseFDeviationLedgerRevisionV1] | §11 physical/deviation definition |
| PhaseFDeviationLedgerV1 | <a id="schema-def-PhaseFDeviationLedgerV1"></a>SCHEMA_DEF[PhaseFDeviationLedgerV1] | §11 physical/deviation definition |
| PhaseFEndpointMetrologyPolicyV1 | <a id="schema-def-PhaseFEndpointMetrologyPolicyV1"></a>SCHEMA_DEF[PhaseFEndpointMetrologyPolicyV1] | §13 metrology/reference definition |
| PhaseFEnvironmentEntryV1 | <a id="schema-def-PhaseFEnvironmentEntryV1"></a>SCHEMA_DEF[PhaseFEnvironmentEntryV1] | §2 closed helper definition |
| PhaseFExecutionRecordV1 | <a id="schema-def-PhaseFExecutionRecordV1"></a>SCHEMA_DEF[PhaseFExecutionRecordV1] | §14 release/monitoring definition |
| PhaseFF5ReleaseCandidateV1 | <a id="schema-def-PhaseFF5ReleaseCandidateV1"></a>SCHEMA_DEF[PhaseFF5ReleaseCandidateV1] | §5 review and F5-candidate definition |
| PhaseFIdentityComparisonV1 | <a id="schema-def-PhaseFIdentityComparisonV1"></a>SCHEMA_DEF[PhaseFIdentityComparisonV1] | §11 physical/deviation definition |
| PhaseFIncidentRecordV1 | <a id="schema-def-PhaseFIncidentRecordV1"></a>SCHEMA_DEF[PhaseFIncidentRecordV1] | §15 incident/retention definition |
| PhaseFIncidentResolutionV1 | <a id="schema-def-PhaseFIncidentResolutionV1"></a>SCHEMA_DEF[PhaseFIncidentResolutionV1] | §15 incident/retention definition |
| PhaseFIncidentScopeV1 | <a id="schema-def-PhaseFIncidentScopeV1"></a>SCHEMA_DEF[PhaseFIncidentScopeV1] | §2 closed helper definition |
| PhaseFIndependentReviewBundleV1 | <a id="schema-def-PhaseFIndependentReviewBundleV1"></a>SCHEMA_DEF[PhaseFIndependentReviewBundleV1] | §5 review and F5-candidate definition |
| PhaseFIndependentReviewV1 | <a id="schema-def-PhaseFIndependentReviewV1"></a>SCHEMA_DEF[PhaseFIndependentReviewV1] | §5 review and F5-candidate definition |
| PhaseFLODLOQPolicyV1 | <a id="schema-def-PhaseFLODLOQPolicyV1"></a>SCHEMA_DEF[PhaseFLODLOQPolicyV1] | §2 closed helper definition |
| PhaseFLocationLedgerV1 | <a id="schema-def-PhaseFLocationLedgerV1"></a>SCHEMA_DEF[PhaseFLocationLedgerV1] | §11 physical/deviation definition |
| PhaseFLocationV1 | <a id="schema-def-PhaseFLocationV1"></a>SCHEMA_DEF[PhaseFLocationV1] | §11 physical/deviation definition |
| PhaseFMethodVersionV1 | <a id="schema-def-PhaseFMethodVersionV1"></a>SCHEMA_DEF[PhaseFMethodVersionV1] | §13 metrology/reference definition |
| PhaseFMetricThresholdV1 | <a id="schema-def-PhaseFMetricThresholdV1"></a>SCHEMA_DEF[PhaseFMetricThresholdV1] | §14 release/monitoring definition |
| PhaseFMetrologyCheckResultV1 | <a id="schema-def-PhaseFMetrologyCheckResultV1"></a>SCHEMA_DEF[PhaseFMetrologyCheckResultV1] | §13 metrology/reference definition |
| PhaseFMetrologyCheckSpecV1 | <a id="schema-def-PhaseFMetrologyCheckSpecV1"></a>SCHEMA_DEF[PhaseFMetrologyCheckSpecV1] | §13 metrology/reference definition |
| PhaseFMetrologyPolicyV1 | <a id="schema-def-PhaseFMetrologyPolicyV1"></a>SCHEMA_DEF[PhaseFMetrologyPolicyV1] | §13 metrology/reference definition |
| PhaseFMonitoringBreachV1 | <a id="schema-def-PhaseFMonitoringBreachV1"></a>SCHEMA_DEF[PhaseFMonitoringBreachV1] | §14 release/monitoring definition |
| PhaseFMonitoringEvidenceV1 | <a id="schema-def-PhaseFMonitoringEvidenceV1"></a>SCHEMA_DEF[PhaseFMonitoringEvidenceV1] | §14 release/monitoring definition |
| PhaseFMonitoringMeasurementV1 | <a id="schema-def-PhaseFMonitoringMeasurementV1"></a>SCHEMA_DEF[PhaseFMonitoringMeasurementV1] | §14 release/monitoring definition |
| PhaseFMonitoringPolicyV1 | <a id="schema-def-PhaseFMonitoringPolicyV1"></a>SCHEMA_DEF[PhaseFMonitoringPolicyV1] | §14 release/monitoring definition |
| PhaseFMonitoringRecordV1 | <a id="schema-def-PhaseFMonitoringRecordV1"></a>SCHEMA_DEF[PhaseFMonitoringRecordV1] | §14 release/monitoring definition |
| PhaseFMonitoringSourceReferenceV1 | <a id="schema-def-PhaseFMonitoringSourceReferenceV1"></a>SCHEMA_DEF[PhaseFMonitoringSourceReferenceV1] | §14 release/monitoring definition |
| PhaseFMonitoringValueV1 | <a id="schema-def-PhaseFMonitoringValueV1"></a>SCHEMA_DEF[PhaseFMonitoringValueV1] | §2 closed helper definition |
| PhaseFNamedDigestV1 | <a id="schema-def-PhaseFNamedDigestV1"></a>SCHEMA_DEF[PhaseFNamedDigestV1] | §2 closed helper definition |
| PhaseFObjectDigestV1 | <a id="schema-def-PhaseFObjectDigestV1"></a>SCHEMA_DEF[PhaseFObjectDigestV1] | §2 closed helper definition |
| PhaseFObjectReferenceV1 | <a id="schema-def-PhaseFObjectReferenceV1"></a>SCHEMA_DEF[PhaseFObjectReferenceV1] | §10 retrieval/package definition |
| PhaseFOutputSpecV1 | <a id="schema-def-PhaseFOutputSpecV1"></a>SCHEMA_DEF[PhaseFOutputSpecV1] | §12 power definition |
| PhaseFPackageBindingV1 | <a id="schema-def-PhaseFPackageBindingV1"></a>SCHEMA_DEF[PhaseFPackageBindingV1] | §10 retrieval/package definition |
| PhaseFPackageManifestV1 | <a id="schema-def-PhaseFPackageManifestV1"></a>SCHEMA_DEF[PhaseFPackageManifestV1] | §10 retrieval/package definition |
| PhaseFPackageObjectV1 | <a id="schema-def-PhaseFPackageObjectV1"></a>SCHEMA_DEF[PhaseFPackageObjectV1] | §10 retrieval/package definition |
| PhaseFParameterSpecV1 | <a id="schema-def-PhaseFParameterSpecV1"></a>SCHEMA_DEF[PhaseFParameterSpecV1] | §12 power definition |
| PhaseFParameterValueRowV1 | <a id="schema-def-PhaseFParameterValueRowV1"></a>SCHEMA_DEF[PhaseFParameterValueRowV1] | §12 power definition |
| PhaseFPhysicalIdentityAuditV1 | <a id="schema-def-PhaseFPhysicalIdentityAuditV1"></a>SCHEMA_DEF[PhaseFPhysicalIdentityAuditV1] | §11 physical/deviation definition |
| PhaseFPhysicalReleaseApprovalV1 | <a id="schema-def-PhaseFPhysicalReleaseApprovalV1"></a>SCHEMA_DEF[PhaseFPhysicalReleaseApprovalV1] | §6 exact annotated-tag body definition |
| PhaseFPhysicalUnitLedgerV1 | <a id="schema-def-PhaseFPhysicalUnitLedgerV1"></a>SCHEMA_DEF[PhaseFPhysicalUnitLedgerV1] | §11 physical/deviation definition |
| PhaseFPlanApprovalV1 | <a id="schema-def-PhaseFPlanApprovalV1"></a>SCHEMA_DEF[PhaseFPlanApprovalV1] | §6 exact annotated-tag body definition |
| PhaseFPowerAnalysisRecordV1 | <a id="schema-def-PhaseFPowerAnalysisRecordV1"></a>SCHEMA_DEF[PhaseFPowerAnalysisRecordV1] | §12 power definition |
| PhaseFPowerMethodInterfaceV1 | <a id="schema-def-PhaseFPowerMethodInterfaceV1"></a>SCHEMA_DEF[PhaseFPowerMethodInterfaceV1] | §12 power definition |
| PhaseFPowerOutputValueV1 | <a id="schema-def-PhaseFPowerOutputValueV1"></a>SCHEMA_DEF[PhaseFPowerOutputValueV1] | §12 power definition |
| PhaseFProtocolProjectionV1 | <a id="schema-def-PhaseFProtocolProjectionV1"></a>SCHEMA_DEF[PhaseFProtocolProjectionV1] | §4 F0 decision/projection definition |
| PhaseFQuantifiedUncertaintyV1 | <a id="schema-def-PhaseFQuantifiedUncertaintyV1"></a>SCHEMA_DEF[PhaseFQuantifiedUncertaintyV1] | §2 closed helper definition |
| PhaseFRangeRuleV1 | <a id="schema-def-PhaseFRangeRuleV1"></a>SCHEMA_DEF[PhaseFRangeRuleV1] | §2 closed helper definition |
| PhaseFReadinessApprovalV1 | <a id="schema-def-PhaseFReadinessApprovalV1"></a>SCHEMA_DEF[PhaseFReadinessApprovalV1] | §6 exact annotated-tag body definition |
| PhaseFReferenceAssessmentV1 | <a id="schema-def-PhaseFReferenceAssessmentV1"></a>SCHEMA_DEF[PhaseFReferenceAssessmentV1] | §13 metrology/reference definition |
| PhaseFReferenceResultV1 | <a id="schema-def-PhaseFReferenceResultV1"></a>SCHEMA_DEF[PhaseFReferenceResultV1] | §13 metrology/reference definition |
| PhaseFReferenceSourceDescriptorV1 | <a id="schema-def-PhaseFReferenceSourceDescriptorV1"></a>SCHEMA_DEF[PhaseFReferenceSourceDescriptorV1] | §13 metrology/reference definition |
| PhaseFRegistryCompromiseEmergencyV1 | <a id="schema-def-PhaseFRegistryCompromiseEmergencyV1"></a>SCHEMA_DEF[PhaseFRegistryCompromiseEmergencyV1] | §15 incident/retention definition |
| PhaseFRegistryHeadV1 | <a id="schema-def-PhaseFRegistryHeadV1"></a>SCHEMA_DEF[PhaseFRegistryHeadV1] | §8-§9 registry definition |
| PhaseFRegistryRecordV1 | <a id="schema-def-PhaseFRegistryRecordV1"></a>SCHEMA_DEF[PhaseFRegistryRecordV1] | §8-§9 registry definition |
| PhaseFRegistryRelationV1 | <a id="schema-def-PhaseFRegistryRelationV1"></a>SCHEMA_DEF[PhaseFRegistryRelationV1] | §8-§9 registry definition |
| PhaseFReinstatementApprovalV1 | <a id="schema-def-PhaseFReinstatementApprovalV1"></a>SCHEMA_DEF[PhaseFReinstatementApprovalV1] | §14 release/monitoring definition |
| PhaseFReleaseRecordV1 | <a id="schema-def-PhaseFReleaseRecordV1"></a>SCHEMA_DEF[PhaseFReleaseRecordV1] | §14 release/monitoring definition |
| PhaseFRetentionAuditV1 | <a id="schema-def-PhaseFRetentionAuditV1"></a>SCHEMA_DEF[PhaseFRetentionAuditV1] | §15 incident/retention definition |
| PhaseFRetentionCopyVerificationV1 | <a id="schema-def-PhaseFRetentionCopyVerificationV1"></a>SCHEMA_DEF[PhaseFRetentionCopyVerificationV1] | §15 incident/retention definition |
| PhaseFRetentionObjectCheckV1 | <a id="schema-def-PhaseFRetentionObjectCheckV1"></a>SCHEMA_DEF[PhaseFRetentionObjectCheckV1] | §15 incident/retention definition |
| PhaseFRetentionObjectV1 | <a id="schema-def-PhaseFRetentionObjectV1"></a>SCHEMA_DEF[PhaseFRetentionObjectV1] | §15 incident/retention definition |
| PhaseFRetentionScopeV1 | <a id="schema-def-PhaseFRetentionScopeV1"></a>SCHEMA_DEF[PhaseFRetentionScopeV1] | §15 incident/retention definition |
| PhaseFRetrievalVerificationV1 | <a id="schema-def-PhaseFRetrievalVerificationV1"></a>SCHEMA_DEF[PhaseFRetrievalVerificationV1] | §10 retrieval/package definition |
| PhaseFReviewTargetV1 | <a id="schema-def-PhaseFReviewTargetV1"></a>SCHEMA_DEF[PhaseFReviewTargetV1] | §5 review and F5-candidate definition |
| PhaseFScientificAdmissibilityAuditV1 | <a id="schema-def-PhaseFScientificAdmissibilityAuditV1"></a>SCHEMA_DEF[PhaseFScientificAdmissibilityAuditV1] | §13 scientific-admissibility definition |
| PhaseFSensitivityCaseV1 | <a id="schema-def-PhaseFSensitivityCaseV1"></a>SCHEMA_DEF[PhaseFSensitivityCaseV1] | §12 power definition |
| PhaseFSensitivityOverrideV1 | <a id="schema-def-PhaseFSensitivityOverrideV1"></a>SCHEMA_DEF[PhaseFSensitivityOverrideV1] | §2 closed helper definition |
| PhaseFTrustProvisioningApprovalV1 | <a id="schema-def-PhaseFTrustProvisioningApprovalV1"></a>SCHEMA_DEF[PhaseFTrustProvisioningApprovalV1] | §6 exact annotated-tag body definition |
| PhaseFUncertaintyPolicyV1 | <a id="schema-def-PhaseFUncertaintyPolicyV1"></a>SCHEMA_DEF[PhaseFUncertaintyPolicyV1] | §2 closed helper definition |
| PhaseFUnitEntryV1 | <a id="schema-def-PhaseFUnitEntryV1"></a>SCHEMA_DEF[PhaseFUnitEntryV1] | §11 physical/deviation definition |
| PhaseFUnitRuleV1 | <a id="schema-def-PhaseFUnitRuleV1"></a>SCHEMA_DEF[PhaseFUnitRuleV1] | §2 closed helper definition |
|---|---|---|

The inline closures made canonical by this index are exact: PhaseFPackageObjectV1
is object_id,object_reference,media_type,format_or_schema,producing_authority_id,
physical,test_only,generated,retention_class_id; PhaseFPackageBindingV1 is
binding_id,role,object_id,physical_unit_ids,direct_dependency_binding_ids;
PhaseFLocationV1 is location_id,location_type,authority_id,
identity_document_sha256; PhaseFMethodVersionV1 is method_id,method_version;
PhaseFMetricThresholdV1 is metric_id,comparator,value,unit; PhaseFOutputSpecV1
is output_id,value_type,unit_rule,range_rule; PhaseFParameterSpecV1 is
parameter_id,value_type,unit_rule,required,range_rule; PhaseFParameterValueRowV1
is parameter_id,value; PhaseFSensitivityCaseV1 is case_id,parameter_overrides,
outputs; PhaseFSensitivityOverrideV1 is parameter_id,value; PhaseFUnitEntryV1 is
unit_id,unit_kind,identity_issuer_authority_id,native_identifier,identity_basis,
identity_basis_document_sha256,parent_unit_ids,independent_family_id,
source_object_ids; PhaseFReferenceAssessmentV1 is reference_result_id,endpoint_id,
evidence_category,claim_ceiling,dependency_status,identity_status,admissibility;
and PhaseFIndependentReviewV1 is role,decision,p0_count,p1_count,finding_ids,
review_artifact_reference. These are current plan definitions and do not create
wire files.

The exact usage matrix below has one row per current parent-field occurrence of
every NESTED_WIRE schema. Its producer, validator, and stage columns are derived
from the named parent operation in that row; no row uses a wildcard context.

| usage_id | nested_schema | parent_schema.field | parent_definition_anchor | producer_context | validator_context | stage_context |
|---|---|---|---|---|---|---|
| USG-001-01 | PhaseFArgvV1 | PhaseFCheckerReportV1.argv | #schema-def-PhaseFCheckerReportV1 | checker build/readiness operation | command/result consistency validator | readiness/checker-invocation |
| USG-002-01 | PhaseFCheckListV1 | PhaseFEndpointMetrologyPolicyV1.calibration_policy | #schema-def-PhaseFEndpointMetrologyPolicyV1 | metrology laboratory | metrology policy plus endpoint/check validator | F0-F2 metrology/reference |
| USG-002-02 | PhaseFCheckListV1 | PhaseFEndpointMetrologyPolicyV1.qc_policy | #schema-def-PhaseFEndpointMetrologyPolicyV1 | metrology laboratory | metrology policy plus endpoint/check validator | F0-F2 metrology/reference |
| USG-003-01 | PhaseFCheckerExitCodeV1 | PhaseFCheckerReportV1.exit_code | #schema-def-PhaseFCheckerReportV1 | checker invocation | checker-report result consistency validator | checker-invocation |
| USG-004-01 | PhaseFCheckerStdoutV1 | PhaseFCheckerReportV1.stdout | #schema-def-PhaseFCheckerReportV1 | checker invocation | checker-report result consistency validator | checker-invocation |
| USG-005-01 | PhaseFCommandV1 | PhaseFCheckerReportV1.command | #schema-def-PhaseFCheckerReportV1 | checker invocation | command grammar plus checker-report consistency validator | checker-invocation |
| USG-006-01 | PhaseFCustodyEventV1 | PhaseFChainOfCustodyV1.events | #schema-def-PhaseFChainOfCustodyV1 | campaign laboratory and custody authority | custody continuity and terminal-unit validator | F2-F4 physical-validation |
| USG-007-01 | PhaseFDecisionRowV1 | PhaseFDecisionBundleV1.decisions | #schema-def-PhaseFDecisionBundleV1 | F0 decision authority | decision-bundle strict parser plus exact 20-row validator | F0 decision-bundle |
| USG-008-01 | PhaseFDecisionValueV1 | PhaseFDecisionRowV1.value | #schema-def-PhaseFDecisionRowV1 | F0 decision authority | decision-specific closed-value validator | F0 decision-bundle |
| USG-009-01 | PhaseFDependencyEdgeV1 | PhaseFDependencyAuditV1.edges | #schema-def-PhaseFDependencyAuditV1 | retrieval/package authority | dependency classification and DAG validator | F2 retrieval/package |
| USG-010-01 | PhaseFDeviationEventV1 | PhaseFDeviationLedgerRevisionV1.events | #schema-def-PhaseFDeviationLedgerRevisionV1 | Phase-F deviation authority | deviation action and revision validator | F1-F4 deviation |
| USG-011-01 | PhaseFEndpointMetrologyPolicyV1 | PhaseFMetrologyPolicyV1.endpoint_policies | #schema-def-PhaseFMetrologyPolicyV1 | metrology laboratory | endpoint-qualified metrology validator | F0-F2 metrology/reference |
| USG-012-01 | PhaseFEnvironmentEntryV1 | PhaseFCheckerBuildEvidenceV1.environment | #schema-def-PhaseFCheckerBuildEvidenceV1 | checker build authority | environment completeness and freshness validator | readiness |
| USG-013-01 | PhaseFIdentityComparisonV1 | PhaseFPhysicalIdentityAuditV1.comparisons | #schema-def-PhaseFPhysicalIdentityAuditV1 | physical identity authority | identity alias/independence validator | F2 physical-validation |
| USG-014-01 | PhaseFIncidentScopeV1 | PhaseFIncidentRecordV1.scope | #schema-def-PhaseFIncidentRecordV1 | incident operations authority | incident scope union validator | incident/retention |
| USG-015-01 | PhaseFIndependentReviewV1 | PhaseFIndependentReviewBundleV1.reviews | #schema-def-PhaseFIndependentReviewBundleV1 | independent five-role review panel | role uniqueness and decision-count validator | all review gates |
| USG-016-01 | PhaseFLODLOQPolicyV1 | PhaseFEndpointMetrologyPolicyV1.lod_loq_policy | #schema-def-PhaseFEndpointMetrologyPolicyV1 | metrology laboratory | LOD/LOQ exact-unit validator | F0-F2 metrology/reference |
| USG-017-01 | PhaseFLocationV1 | PhaseFLocationLedgerV1.locations | #schema-def-PhaseFLocationLedgerV1 | location authority | location type and identity-document validator | F2 physical-validation |
| USG-018-01 | PhaseFMethodVersionV1 | PhaseFEndpointMetrologyPolicyV1.allowed_methods | #schema-def-PhaseFEndpointMetrologyPolicyV1 | metrology laboratory | method/version Cartesian-policy validator | F0-F2 metrology/reference |
| USG-019-01 | PhaseFMetricThresholdV1 | PhaseFMonitoringPolicyV1.metric_thresholds | #schema-def-PhaseFMonitoringPolicyV1 | monitoring policy authority | five-threshold completeness and comparator validator | F0 monitoring-policy |
| USG-020-01 | PhaseFMetrologyCheckSpecV1 | PhaseFCheckListV1.check_specs | #schema-def-PhaseFCheckListV1 | metrology laboratory | endpoint/check lookup and threshold validator | F0-F2 metrology/reference |
| USG-021-01 | PhaseFMonitoringBreachV1 | PhaseFMonitoringRecordV1.breaches | #schema-def-PhaseFMonitoringRecordV1 | monitoring operations authority | recomputed-breach-set validator | F5+ monitoring |
| USG-022-01 | PhaseFMonitoringMeasurementV1 | PhaseFMonitoringRecordV1.measurements | #schema-def-PhaseFMonitoringRecordV1 | monitoring operations authority | 15-metric fixed-order and evidence-binding validator | F5+ monitoring |
| USG-023-01 | PhaseFMonitoringSourceReferenceV1 | PhaseFMonitoringEvidenceV1.source_references | #schema-def-PhaseFMonitoringEvidenceV1 | monitoring operations authority | metric-to-source-kind mapping validator | F5+ monitoring |
| USG-024-01 | PhaseFMonitoringValueV1 | PhaseFMonitoringEvidenceV1.value | #schema-def-PhaseFMonitoringEvidenceV1 | monitoring operations authority | metric-specific value-type validator | F5+ monitoring evidence |
| USG-024-02 | PhaseFMonitoringValueV1 | PhaseFMonitoringMeasurementV1.value | #schema-def-PhaseFMonitoringMeasurementV1 | monitoring operations authority | metric-specific value-type validator | F5+ monitoring |
| USG-025-01 | PhaseFNamedDigestV1 | PhaseFCheckerReportV1.input_sha256s | #schema-def-PhaseFCheckerReportV1 | checker invocation | named-input digest and report consistency validator | checker-invocation |
| USG-026-01 | PhaseFObjectDigestV1 | PhaseFIncidentRecordV1.affected_object_sha256s | #schema-def-PhaseFIncidentRecordV1 | incident operations authority | object-kind/hash and incident-scope validator | incident/retention |
| USG-027-01 | PhaseFObjectReferenceV1 | PhaseFDecisionValueV1.authority_document | #schema-def-PhaseFDecisionValueV1 | F0 decision authority | object-reference URI/hash/length validator plus F0 projection check | F0 |
| USG-027-02 | PhaseFObjectReferenceV1 | PhaseFDecisionValueV1.custody_procedure_document | #schema-def-PhaseFDecisionValueV1 | F0 decision authority | object-reference URI/hash/length validator plus F0 projection check | F0 |
| USG-027-03 | PhaseFObjectReferenceV1 | PhaseFAuthorityEnrollmentV1.owner_authority_document | #schema-def-PhaseFAuthorityEnrollmentV1 | authority-enrollment operation | enrollment strict parser plus reference resolver | enrollment |
| USG-027-04 | PhaseFObjectReferenceV1 | PhaseFAuthorityEnrollmentV1.registry_authority_document | #schema-def-PhaseFAuthorityEnrollmentV1 | authority-enrollment operation | enrollment strict parser plus reference resolver | enrollment |
| USG-027-05 | PhaseFObjectReferenceV1 | PhaseFIndependentReviewV1.review_artifact_reference | #schema-def-PhaseFIndependentReviewV1 | independent five-role review panel | review-bundle artifact-reference validator | all review gates |
| USG-027-06 | PhaseFObjectReferenceV1 | PhaseFRetrievalVerificationV1.object_reference | #schema-def-PhaseFRetrievalVerificationV1 | retrieval/package authority | retrieval URI/hash/length verifier | F2 retrieval |
| USG-027-07 | PhaseFObjectReferenceV1 | PhaseFPackageObjectV1.object_reference | #schema-def-PhaseFPackageObjectV1 | retrieval/package authority | package-object reference and manifest-binding validator | F2 package |
| USG-027-08 | PhaseFObjectReferenceV1 | PhaseFMetrologyCheckSpecV1.procedure_document | #schema-def-PhaseFMetrologyCheckSpecV1 | metrology laboratory | metrology procedure-reference resolver | F0-F2 metrology/reference |
| USG-027-09 | PhaseFObjectReferenceV1 | PhaseFMonitoringSourceReferenceV1.object_reference | #schema-def-PhaseFMonitoringSourceReferenceV1 | monitoring operations authority | monitoring source URI/hash/length and source-kind validator | F5+ monitoring |
| USG-027-10 | PhaseFObjectReferenceV1 | PhaseFIncidentRecordV1.evidence_references | #schema-def-PhaseFIncidentRecordV1 | incident operations authority | incident evidence-reference resolver | incident/retention |
| USG-027-11 | PhaseFObjectReferenceV1 | PhaseFIncidentResolutionV1.evidence_references | #schema-def-PhaseFIncidentResolutionV1 | incident operations authority | resolution evidence-reference resolver | incident/retention |
| USG-027-12 | PhaseFObjectReferenceV1 | PhaseFRetentionCopyVerificationV1.object_reference | #schema-def-PhaseFRetentionCopyVerificationV1 | retention auditor | copy URI/hash/length/freshness validator | retention |
| USG-028-01 | PhaseFOutputSpecV1 | PhaseFPowerMethodInterfaceV1.output_spec | #schema-def-PhaseFPowerMethodInterfaceV1 | power-analysis authority | power output type/unit/range validator | F1 power |
| USG-029-01 | PhaseFPackageBindingV1 | PhaseFPackageManifestV1.bindings | #schema-def-PhaseFPackageManifestV1 | retrieval/package authority | package role/unit/dependency validator | F2 package |
| USG-030-01 | PhaseFPackageObjectV1 | PhaseFPackageManifestV1.objects | #schema-def-PhaseFPackageManifestV1 | retrieval/package authority | package object reference/role/flag validator | F2 package |
| USG-031-01 | PhaseFParameterSpecV1 | PhaseFPowerMethodInterfaceV1.parameter_specs | #schema-def-PhaseFPowerMethodInterfaceV1 | power-analysis authority | parameter type/unit/range validator | F1 power |
| USG-032-01 | PhaseFParameterValueRowV1 | PhaseFPowerAnalysisRecordV1.parameters | #schema-def-PhaseFPowerAnalysisRecordV1 | power-analysis authority | declared-parameter and value-type validator | F1 power |
| USG-033-01 | PhaseFPowerOutputValueV1 | PhaseFPowerAnalysisRecordV1.outputs | #schema-def-PhaseFPowerAnalysisRecordV1 | power-analysis authority | declared-output and value-type validator | F1 power |
| USG-033-02 | PhaseFPowerOutputValueV1 | PhaseFSensitivityCaseV1.outputs | #schema-def-PhaseFSensitivityCaseV1 | power-analysis authority | sensitivity output and base-semantics validator | F1 power |
| USG-034-01 | PhaseFQuantifiedUncertaintyV1 | PhaseFReferenceResultV1.uncertainty | #schema-def-PhaseFReferenceResultV1 | metrology laboratory | uncertainty unit/value validator | F0-F2 metrology/reference |
| USG-035-01 | PhaseFRangeRuleV1 | PhaseFParameterSpecV1.range_rule | #schema-def-PhaseFParameterSpecV1 | power-analysis authority | range/type/unit validator | F1 power |
| USG-035-02 | PhaseFRangeRuleV1 | PhaseFOutputSpecV1.range_rule | #schema-def-PhaseFOutputSpecV1 | power-analysis authority | range/type/unit validator | F1 power |
| USG-036-01 | PhaseFReferenceAssessmentV1 | PhaseFScientificAdmissibilityAuditV1.reference_assessments | #schema-def-PhaseFScientificAdmissibilityAuditV1 | scientific/metrology reviewer | admissibility,identity,and dependency validator | F2 scientific audit |
| USG-037-01 | PhaseFRegistryRelationV1 | PhaseFRegistryRecordV1.relations | #schema-def-PhaseFRegistryRecordV1 | registry authority | relation kind/object-kind/hash validator | all registry operations |
| USG-038-01 | PhaseFRetentionCopyVerificationV1 | PhaseFRetentionObjectCheckV1.copies | #schema-def-PhaseFRetentionObjectCheckV1 | retention auditor | copy SHA/length/scheme/availability/freshness/count validator | retention |
| USG-039-01 | PhaseFRetentionObjectCheckV1 | PhaseFRetentionAuditV1.object_checks | #schema-def-PhaseFRetentionAuditV1 | retention auditor | exact-set and per-object copy validator | retention |
| USG-040-01 | PhaseFRetentionObjectV1 | PhaseFRetentionObjectCheckV1.object | #schema-def-PhaseFRetentionObjectCheckV1 | retention auditor | kind/SHA identity and package-object binding validator | retention |
| USG-041-01 | PhaseFRetentionScopeV1 | PhaseFRetentionAuditV1.scope | #schema-def-PhaseFRetentionAuditV1 | retention auditor | release/campaign scope and relation validator | retention |
| USG-042-01 | PhaseFReviewTargetV1 | PhaseFIndependentReviewBundleV1.target | #schema-def-PhaseFIndependentReviewBundleV1 | independent five-role review panel | Git/external target union validator | all review gates |
| USG-043-01 | PhaseFSensitivityCaseV1 | PhaseFPowerAnalysisRecordV1.sensitivity_cases | #schema-def-PhaseFPowerAnalysisRecordV1 | power-analysis authority | sensitivity completeness and base-semantics validator | F1 power |
| USG-044-01 | PhaseFSensitivityOverrideV1 | PhaseFSensitivityCaseV1.parameter_overrides | #schema-def-PhaseFSensitivityCaseV1 | power-analysis authority | declared parameter override validator | F1 power |
| USG-045-01 | PhaseFUncertaintyPolicyV1 | R11 canonical helper-use record for PhaseFUncertaintyPolicyV1 | #schema-def-R11 canonical helper-use record for PhaseFUncertaintyPolicyV1 | the exact enclosing operation named by the usage record | intrinsic parser plus the exact enclosing semantic validator named by the usage record | the exact stage named by the usage record |
| USG-046-01 | PhaseFUnitEntryV1 | PhaseFPhysicalUnitLedgerV1.entries | #schema-def-PhaseFPhysicalUnitLedgerV1 | campaign laboratory | native identity and parent-child validator | F2 physical-validation |
| USG-047-01 | PhaseFUnitRuleV1 | PhaseFParameterSpecV1.unit_rule | #schema-def-PhaseFParameterSpecV1 | power-analysis authority | unit-rule and range consistency validator | F1 power |
| USG-047-02 | PhaseFUnitRuleV1 | PhaseFOutputSpecV1.unit_rule | #schema-def-PhaseFOutputSpecV1 | power-analysis authority | unit-rule and range consistency validator | F1 power |

OBJECT_REFERENCE_USAGE_GAPS=0. For PhaseFObjectReferenceV1 the closed usage set
contains exactly the 12 current parent fields listed in its rows: two F0
decision-value fields, two enrollment-document fields, one review-artifact
field, one retrieval field, one package-object field, one metrology-procedure
field, one monitoring-source field, two incident fields, and one retention-copy
field. No producer, validator, or stage is inherited from another parent.

### 53.8 R11 current normative requirement matrix

The historical R10 matrix in §43 is HISTORICAL / NON-CURRENT. This is the one
current R11 matrix. It carries forward valid R10 obligations and adds KAT layer
separation, real incident JSON, storage-only retention scope, test
classification/completeness, definition anchors, usage matrix, exact catalog
metadata, campaign/static consistency, and Markdown structural validity. The
schema ID cells are literal identifier lists; no schema is implied by a range.

| requirement_id | normative_statement | owner_decision_ids | schema_ids | stage | review_roles | primary_ac_id | test_ids | evidence_ids |
|---|---|---|---|---|---|---|---|---|
| R11-01 | Plan-tag parser binding is tested with literal X, literal body fields, exact bytes, and separate real-Git resolution property. | none | PhaseFPlanApprovalV1,PhaseFIndependentReviewBundleV1,PhaseFReviewTargetV1 | plan review | architecture_data,security | AC11-01 | R11-POS-PLAN,R11-CX-16,R11-CX-18 | EV11-01 |
| R11-02 | Trust provisioning parser binding is literal ASCII message parsing with exact trust fields and separate real-tag resolution property. | F-OD-04 | PhaseFTrustProvisioningApprovalV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringValueV1 | F3/F5+ | security,operations_governance | AC11-02 | R11-POS-TRUST,R11-CX-17,R11-CX-18 | EV11-02 |
| R11-03 | Retention storage bytes are opaque copy payloads used only after prevalidated kind/SHA identity. | F-OD-20 | PhaseFRetentionObjectV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionAuditV1,PhaseFObjectReferenceV1 | retention | security,operations_governance | AC11-03 | R11-KAT-RETENTION-COPY,R11-CX-01,R11-CX-02 | EV11-03 |
| R11-04 | Incident progression uses literal schema-valid I2, IR2-C, and IR2-R objects with exact semantic and complete-file hashes. | F-OD-16 | PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1 | incident/retention | operations_governance,security | AC11-04 | R11-KAT-INCIDENT,R11-CX-03,R11-CX-04 | EV11-04 |
| R11-05 | Release retention is composed from upstream identities, incident status, accepted monitoring, static identities, and exact set equality. | F-OD-13,F-OD-14,F-OD-15,F-OD-19,F-OD-20 | PhaseFReleaseRecordV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFMonitoringRecordV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFRetentionAuditV1 | F5+ retention | operations_governance,security | AC11-05 | R11-KAT-RETENTION-COPY,R11-CX-14,R11-CX-15 | EV11-05 |
| R11-06 | Campaign membership is manifest plus every package object; protocol-P and all static release authorities remain outside that campaign set. | F-OD-18 | PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFRetentionScopeV1,PhaseFRetentionObjectV1 | campaign/retention | operations_governance,security | AC11-06 | R11-CX-14,R11-CX-15 | EV11-06 |
| R11-07 | Monitoring positive coverage is either a fully literal 15-metric KAT or a narrow property; no unspecified full PASS exists. | F-OD-17,F-OD-19 | PhaseFMonitoringPolicyV1,PhaseFMetricThresholdV1,PhaseFMonitoringRecordV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1 | F0/F5+ | operations_governance,security | AC11-07 | R11-PROP-MONITORING,R11-CX-18 | EV11-07 |
| R11-08 | Constructive DAG audit uses explicit authority nodes and edges, with cycle/future/self-Git checks. | none | PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFCommandV1,PhaseFArgvV1,PhaseFCheckerReportV1 | plan review | architecture_data,compatibility | AC11-08 | R11-DAG-AUDIT,R11-CX-08,R11-CX-09 | EV11-08 |
| R11-09 | Every current PhaseF identifier has one exact stable definition anchor and one catalog row. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data | AC11-09 | R11-CAT,R11-CX-12,R11-CX-20 | EV11-09 |
| R11-10 | Every NESTED_WIRE schema has explicit parent-field usage rows and ObjectReference use is exhaustive. | none | PhaseFArgvV1,PhaseFCheckListV1,PhaseFCheckerExitCodeV1,PhaseFCheckerStdoutV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFIdentityComparisonV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckSpecV1,PhaseFMonitoringBreachV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPowerOutputValueV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReferenceAssessmentV1,PhaseFRegistryRelationV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFReviewTargetV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data,security | AC11-10 | R11-CX-10,R11-CX-11 | EV11-10 |
| R11-11 | Every current catalog row has exact category, closure anchor, producer, validator, stage, and registry behavior. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data,security,compatibility | AC11-11 | R11-CAT,R11-CX-10,R11-CX-13 | EV11-11 |
| R11-12 | Catalog traceability is only the sorted inverse projection of this matrix. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data | AC11-12 | R11-TRACE,R11-CX-12 | EV11-12 |
| R11-13 | Every literal KAT row has zero unbound required inputs; property and constructive rows state their result type without a fictional PASS fixture. | none | PhaseFCheckerReportV1,PhaseFMonitoringRecordV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFRetentionCopyVerificationV1 | plan review | architecture_data,compatibility | AC11-13 | R11-CX-05,R11-CX-06,R11-CX-07,R11-CX-18 | EV11-13 |
| R11-14 | Plan-embedded KATs never become F-EV real evidence, owner approval, metrology evidence, registry evidence, monitoring evidence, or claims. | none | PhaseFObjectReferenceV1,PhaseFIncidentRecordV1,PhaseFMonitoringEvidenceV1,PhaseFRetentionAuditV1 | plan review | architecture_data,security | AC11-14 | R11-CX-01,R11-CX-02 | EV11-14 |
| R11-15 | Campaign/static terminology is exact: package manifest/objects only are campaign identities and protocol-P is a STATIC RELEASE AUTHORITY. | F-OD-18 | PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFReleaseRecordV1,PhaseFRetentionObjectV1 | campaign/retention | operations_governance,security | AC11-15 | R11-CX-14,R11-CX-15 | EV11-15 |
| R11-16 | Markdown fences are balanced and current headings render outside accidental code blocks. | none | PhaseFCheckerReportV1,PhaseFIncidentResolutionV1,PhaseFRetentionAuditV1 | plan review | architecture_data | AC11-16 | R11-CX-19,R11-CX-20 | EV11-16 |
| R11-17 | All closed safety, scientific, physical identity, custody, emergency, monitoring, Phase-E compatibility, runner order, and P2 contracts remain unchanged. | F-OD-01,F-OD-02,F-OD-03,F-OD-05,F-OD-06,F-OD-07,F-OD-08,F-OD-09,F-OD-10,F-OD-11,F-OD-12,F-OD-16,F-OD-17,F-OD-19 | PhaseFDecisionBundleV1,PhaseFPowerAnalysisRecordV1,PhaseFPhysicalUnitLedgerV1,PhaseFChainOfCustodyV1,PhaseFMonitoringPolicyV1,PhaseFClaimStateRecordV1,PhaseFRegistryCompromiseEmergencyV1 | all preserved stages | scientific_metrology,security,compatibility | AC11-17 | R11-CX-08,R11-CX-18 | EV11-17 |
| R11-18 | The current schema set is exactly the 91 literal identifiers listed in §53.7 and the inverse projection covers each one. | none | PhaseFArgvV1,PhaseFAuthorityEnrollmentApprovalV1,PhaseFAuthorityEnrollmentV1,PhaseFChainOfCustodyV1,PhaseFCheckListV1,PhaseFCheckerBuildEvidenceV1,PhaseFCheckerExitCodeV1,PhaseFCheckerReadinessEvidenceV1,PhaseFCheckerReportV1,PhaseFCheckerStdoutV1,PhaseFClaimStateRecordV1,PhaseFCohortLockRecordV1,PhaseFCommandV1,PhaseFCustodyEventV1,PhaseFDecisionApprovalV1,PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFDependencyAuditV1,PhaseFDependencyEdgeV1,PhaseFDeviationEventV1,PhaseFDeviationLedgerRevisionV1,PhaseFDeviationLedgerV1,PhaseFEndpointMetrologyPolicyV1,PhaseFEnvironmentEntryV1,PhaseFExecutionRecordV1,PhaseFF5ReleaseCandidateV1,PhaseFIdentityComparisonV1,PhaseFIncidentRecordV1,PhaseFIncidentResolutionV1,PhaseFIncidentScopeV1,PhaseFIndependentReviewBundleV1,PhaseFIndependentReviewV1,PhaseFLODLOQPolicyV1,PhaseFLocationLedgerV1,PhaseFLocationV1,PhaseFMethodVersionV1,PhaseFMetricThresholdV1,PhaseFMetrologyCheckResultV1,PhaseFMetrologyCheckSpecV1,PhaseFMetrologyPolicyV1,PhaseFMonitoringBreachV1,PhaseFMonitoringEvidenceV1,PhaseFMonitoringMeasurementV1,PhaseFMonitoringPolicyV1,PhaseFMonitoringRecordV1,PhaseFMonitoringSourceReferenceV1,PhaseFMonitoringValueV1,PhaseFNamedDigestV1,PhaseFObjectDigestV1,PhaseFObjectReferenceV1,PhaseFOutputSpecV1,PhaseFPackageBindingV1,PhaseFPackageManifestV1,PhaseFPackageObjectV1,PhaseFParameterSpecV1,PhaseFParameterValueRowV1,PhaseFPhysicalIdentityAuditV1,PhaseFPhysicalReleaseApprovalV1,PhaseFPhysicalUnitLedgerV1,PhaseFPlanApprovalV1,PhaseFPowerAnalysisRecordV1,PhaseFPowerMethodInterfaceV1,PhaseFPowerOutputValueV1,PhaseFProtocolProjectionV1,PhaseFQuantifiedUncertaintyV1,PhaseFRangeRuleV1,PhaseFReadinessApprovalV1,PhaseFReferenceAssessmentV1,PhaseFReferenceResultV1,PhaseFReferenceSourceDescriptorV1,PhaseFRegistryCompromiseEmergencyV1,PhaseFRegistryHeadV1,PhaseFRegistryRecordV1,PhaseFRegistryRelationV1,PhaseFReinstatementApprovalV1,PhaseFReleaseRecordV1,PhaseFRetentionAuditV1,PhaseFRetentionCopyVerificationV1,PhaseFRetentionObjectCheckV1,PhaseFRetentionObjectV1,PhaseFRetentionScopeV1,PhaseFRetrievalVerificationV1,PhaseFReviewTargetV1,PhaseFScientificAdmissibilityAuditV1,PhaseFSensitivityCaseV1,PhaseFSensitivityOverrideV1,PhaseFTrustProvisioningApprovalV1,PhaseFUncertaintyPolicyV1,PhaseFUnitEntryV1,PhaseFUnitRuleV1 | plan review | architecture_data | AC11-18 | R11-CAT,R11-TRACE | EV11-18 |
| R11-19 | Future F-EV rows refer only to future real artifacts; KAT values are test inputs and never future evidence. | none | PhaseFIndependentReviewBundleV1,PhaseFMonitoringEvidenceV1,PhaseFRetentionAuditV1 | future validation | architecture_data,security | AC11-19 | R11-CX-01,R11-CX-02,R11-CX-04 | EV11-19 |
| R11-20 | Owner-decision coverage is derived only from this matrix and equals exactly F-OD-01 through F-OD-20 with no F-OD-21. | F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-05,F-OD-06,F-OD-07,F-OD-08,F-OD-09,F-OD-10,F-OD-11,F-OD-12,F-OD-13,F-OD-14,F-OD-15,F-OD-16,F-OD-17,F-OD-18,F-OD-19,F-OD-20 | PhaseFDecisionBundleV1,PhaseFDecisionRowV1,PhaseFDecisionValueV1,PhaseFProtocolProjectionV1 | F0 | architecture_data,scientific_metrology | AC11-20 | R11-TRACE | EV11-20 |

The catalog traceability rule is exact: for each schema S, requirements are all
matrix rows whose literal schema_ids contain S; ACs are those rows' primary AC
IDs; tests are the sorted union of those rows' test IDs; F-EVs are the sorted
union of those rows' evidence IDs. No manual catalog override, second current
mapping table, or hand-maintained exception is permitted.

### 53.9 R11 acceptance criteria

| AC ID | exact inputs | operation | expected result | failure oracle |
|---|---|---|---|---|
| AC11-01 | literal X, peeled target, six plan-body values, body length 313, body SHA | parse body and test target/body predicate only | PASS | wrong target/body/length/SHA fails; Git resolution is a separate property |
| AC11-02 | 996-byte literal ASCII trust message, message SHA, parsed trust_root_id, parsed trust_store_sha256, matching monitoring values | parse exact ordered lines and compare field bindings | PASS | JSON object, wrong bytes/hash/length, or field mismatch fails |
| AC11-03 | named R11RetentionStorageFixtureV1, prevalidated kind/SHA table, two literal copy rows, exact URIs/lengths/times | validate storage-copy predicates only | PASS FOR RETENTION COPY/COVERAGE LAYER | any schema-validity or full-Phase-F claim is an overclaim |
| AC11-04 | literal I2, IR2-C, IR2-R canonical JSON and all listed IDs/hashes | recompute semantic IDs, complete hashes, number chain, and predecessor | PASS | any byte/ID/hash/predecessor mismatch fails |
| AC11-05 | prevalidated retention identities, incident membership, accepted monitoring identities, and exact expected set | compose upstream identities and compare exact set | PASS FOR RETENTION COPY/COVERAGE LAYER | missing/extra/unresolved identity fails |
| AC11-06 | manifest-M and package-O1/O2 identity rows | derive campaign set | PASS | adding protocol or removing a package object fails |
| AC11-07 | explicit source-kind/value property variables, or a fully bound 15-metric fixture | run the selected narrow property or complete validator | PROPERTY or PASS only for complete fixture | symbolic variable may not claim literal PASS |
| AC11-08 | explicit R11-DAG-AUDIT node and edge lists | run cycle, future-edge, and self-Git-edge detection | ACYCLIC | any cycle or undeclared edge fails |
| AC11-09 | 91 identifier set and 91 unique HTML anchors | compare set and anchor multiplicity | PASS | missing, duplicate, or extra identifier/anchor fails |
| AC11-10 | full usage matrix including all ObjectReference parent fields | compare usage rows to current definitions | PASS | omitted or wildcard context fails |
| AC11-11 | 91 catalog rows and operational metadata profile cells | compare category/closure/producer/validator/stage/registry cells | PASS | blank, generic, or contradictory cell fails |
| AC11-12 | current R11 matrix and catalog traceability projection | recompute sorted inverse projection | PASS | manual mapping or mismatch fails |
| AC11-13 | current test table with required/declaration input sets and expected-result type | compute unbound-required-input count for every row | PASS with zero for literal KATs | any incomplete literal KAT fails |
| AC11-14 | KAT and F-EV tables | scan for test-to-real-evidence promotion edges | PASS with zero paths | KAT referenced as future evidence fails |
| AC11-15 | campaign/static terminology scan and membership rows | compare terms to exact campaign set | PASS | protocol-P or static authority labeled campaign member fails |
| AC11-16 | complete Markdown source | track backtick and tilde state, length, and headings | PASS | open/mismatched fence or heading inside accidental fence fails |
| AC11-17 | closed R10 contracts, F0 20 rows, Phase-E hashes, and safety counters | preservation comparison | PASS | architecture/scientific/runtime regression fails |
| AC11-18 | literal 91-name set plus catalog | compare identifier sets and duplicates | PASS | count continuity without set equality fails |
| AC11-19 | future artifact declarations and KAT labels | separate future real evidence from test material | PASS | any KAT-to-F-EV promotion fails |
| AC11-20 | owner_decision_ids in this matrix | derive union | exactly F-OD-01..F-OD-20 | F-OD-21, missing ID, or second mapping source fails |

### 53.10 Current R11 test procedures and completeness audit

For every row below: test_id, kat_class, fixture_scope, prevalidated_inputs,
literal_inputs, operation, expected_result, negative_mutation, and
UNBOUND_REQUIRED_INPUT_COUNT are all present. A literal executable KAT has zero
unbound required inputs even when its expected result is FAIL. A property test
has expected result type PROPERTY. A constructive plan audit has expected result
type AUDIT RESULT.

| test_id | kat_class | fixture_scope | prevalidated_inputs | literal_inputs | operation | expected_result | negative_mutation | UNBOUND_REQUIRED_INPUT_COUNT |
|---|---|---|---|---|---|---|---|---:|
| R11-POS-PLAN | production_schema_kat | plan-tag parser only | none | X, target X, six ordered body values, length 313, body SHA | target/body binding predicate | PASS | replace target with R10 commit | 0 |
| R11-POS-TRUST | production_schema_kat | trust tag body parser only | none | 996-byte ASCII body, message SHA, trust_root_id, trust_store_sha256, matching source URI/SHA/length | tag parse, byte hash/length, parsed-field binding | PASS | replace trust source with JSON object | 0 |
| R11-KAT-INCIDENT | production_schema_kat | I2/IR2-C/IR2-R complete schema files | none | three literal JCS objects, semantic IDs, 704/607/668 lengths, complete SHAs | schema parse, semantic-ID recomputation, predecessor binding | PASS | use opaque storage SHA as predecessor | 0 |
| R11-KAT-RETENTION-COPY | retention_storage_kat | storage-copy coverage only | R11_PREVALIDATED_RETENTION_IDENTITIES | storage labels, kind/SHA identities, two URIs, byte lengths, availability, times, count | copy SHA/length/scheme/freshness/distinctness/set validator | PASS FOR RETENTION COPY/COVERAGE LAYER | parse an opaque payload as release JSON | 0 |
| R11-PROP-MONITORING | property_test | metric-to-source-kind local property | none | symbolic metric M and source kind K | prove allowed(M,K) iff exact mapping table permits K | PROPERTY | state literal monitoring PASS for symbolic M/K | 0 |
| R11-DAG-AUDIT | constructive_plan_audit | explicit plan graph | none | exact node set and edge list in §53.6 | cycle/future/self-Git audit | AUDIT RESULT: ACYCLIC | add retention->plan_review edge | 0 |
| R11-CAT | constructive_plan_audit | identifier/anchor/catalog set | none | 91 identifiers, 91 anchors, 91 catalog rows | set equality, multiplicity, metadata profile audit | AUDIT RESULT: PASS | delete one row or duplicate one anchor | 0 |
| R11-TRACE | constructive_plan_audit | matrix inverse projection | none | literal R11 matrix rows and schema lists | derive AC/test/F-EV/catalog projections and OD union | AUDIT RESULT: PASS | add a second mapping source | 0 |
| R11-CX-01 | constructive_plan_audit | storage-versus-schema classification | none | storage label release-R and declared schema role release record | test scope classifier | FAIL: fixture-scope failure | claim storage bytes are canonical release JSON | 0 |
| R11-CX-02 | retention_storage_kat | storage-only precondition | R11_PREVALIDATED_RETENTION_IDENTITIES | assumed kind/SHA plus opaque bytes and two complete copy rows | isolated copy validator | PASS FOR RETENTION COPY/COVERAGE LAYER | remove prevalidated identity | 0 |
| R11-CX-03 | constructive_plan_audit | incident predecessor source audit | none | IR2-C predecessor replaced by opaque storage text | verify predecessor source class | FAIL: incident schema KAT | bind predecessor to canonical IR2-C SHA | 0 |
| R11-CX-04 | production_schema_kat | complete incident chain | none | literal I2/IR2-C/IR2-R objects and exact hashes | verify schema-valid chain and predecessor | PASS | change one canonical byte | 0 |
| R11-CX-05 | constructive_plan_audit | test-row completeness model | none | explicit required-set {policy_sha} and declared-set {} in the row-under-test | calculate completeness audit result | AUDIT RESULT: FAIL | declared set includes policy_sha | 0 |
| R11-CX-06 | property_test | quantified copy-count property | none | symbolic B with B>=0 and symbolic valid_copy_count | evaluate valid_copy_count < 1+B implies result != PASS | PROPERTY | replace quantified implication with literal PASS | 0 |
| R11-CX-07 | property_test | quantified copy-count property | none | symbolic B with B>=0 | detect literal PASS claim for unspecified fixture | PROPERTY RESULT: FAIL | state only quantified implication | 0 |
| R11-CX-08 | constructive_plan_audit | DAG node/edge audit | none | explicit node and edge list | run plan-level graph checks | AUDIT RESULT: PASS | omit a major authority node | 0 |
| R11-CX-09 | constructive_plan_audit | test specification text | none | T10-POS-DAG sentence claiming §§2-15 authority fixtures | classify procedure wording | AUDIT RESULT: FAIL | replace with R11-DAG-AUDIT and explicit edges | 0 |
| R11-CX-10 | constructive_plan_audit | catalog producer metadata | none | PhaseFObjectReferenceV1 row plus all USAGE_SET rows | compare producer set to usage-derived producers | AUDIT RESULT: FAIL | list every usage producer | 0 |
| R11-CX-11 | constructive_plan_audit | ObjectReference usage set | none | all 12 exact parent fields listed in matrix | compare usage set | AUDIT RESULT: PASS | delete one parent field | 0 |
| R11-CX-12 | constructive_plan_audit | catalog anchor pointer | none | one identifier, anchor index, and catalog pointer | compare pointer to anchor | AUDIT RESULT: FAIL | point to #schema-def-ID | 0 |
| R11-CX-13 | constructive_plan_audit | catalog validator profile | none | top-level row with parser but omitted semantic validator | compare validator profile to schema-specific profile | AUDIT RESULT: FAIL | restore all semantic checks | 0 |
| R11-CX-14 | constructive_plan_audit | campaign/static membership | none | mutation labels protocol-P as campaign member | compare exact campaign set and static release authority set | AUDIT RESULT: FAIL | label protocol-P STATIC RELEASE AUTHORITY | 0 |
| R11-CX-15 | constructive_plan_audit | campaign membership | none | package manifest plus package-O1 as package object | derive campaign set | AUDIT RESULT: PASS | remove package-O1 from manifest/object set | 0 |
| R11-CX-16 | production_schema_kat | plan-tag parser only | none | literal X/body/length/SHA/target | binding predicate | PASS | use undeclared symbolic X | 0 |
| R11-CX-17 | production_schema_kat | trust parser only | none | literal message bytes/hash/length/parsed fields | parse and compare trust values | PASS | omit trust_store_sha256 | 0 |
| R11-CX-18 | property_test | production Git-resolution condition | none | symbolic real tag T and literal parser-KAT output | state real-tag resolution is additionally required | PROPERTY | claim parser KAT proves Git tag | 0 |
| R11-CX-19 | constructive_plan_audit | whole Markdown file | none | final fence-state scan | detect open fence at EOF | AUDIT RESULT: FAIL | close fence and rerun | 0 |
| R11-CX-20 | constructive_plan_audit | whole Markdown file | none | all current headings and fence-state trace | verify headings render outside fences | AUDIT RESULT: PASS | leave heading inside accidental fence | 0 |

The current test set intentionally has 28 rows, not R10's 38. All retained
requirements have substantive coverage through literal KATs, properties, or
constructive audits. No incomplete literal PASS test remains.
LITERAL_KAT_UNBOUND_INPUTS=0.
PROPERTY_TEST_LITERAL_PASS_OVERCLAIMS=0.
CURRENT_TEST_PROCEDURE_GAPS=0.

### 53.11 Current F-EV evidence oracles

F-EV rows describe future REAL artifacts only. They are never supplied by the
plan-embedded KATs. Plan text, KAT bytes, storage copies, owner decisions, and
test transcripts are not F-EV.

| F-EV ID | future real artifact | exact producer/authority | immutable identity | acceptance oracle |
|---|---|---|---|---|
| EV11-01 | real plan review bundle, plan tag, and final plan bytes | independent reviewer and Git tag validator | review bundle SHA; peeled commit; plan SHA and Git blob | target/body and plan-byte equality |
| EV11-02 | real trust-provisioning annotated tag and exact source copies | independent trust gate and operations authority | exact message SHA/length and trust fields | tag prerequisites, bytes, parsed fields, monitoring binding |
| EV11-03 | real retention copies after upstream identity validation | retention auditor and copy retriever | kind/SHA identity plus URI/length/time/copy transcript | exact copy coverage only |
| EV11-04 | real incident and contained-before-terminal resolution files | operations/governance and registry authorities | complete-file hashes, semantic IDs, sequence | schema, chain, registry chronology, audited-at status |
| EV11-05 | real release retention audit and exact member set | release/retention authorities | audit SHA and kind/SHA identities | static, campaign, monitoring, incident, resolution composition |
| EV11-06 | real package manifest and campaign objects | campaign/package authority | manifest/object complete hashes | manifest plus every object and no static member |
| EV11-07 | real monitoring policy, 15 evidence objects, record, and registry relations | F0 owner, operations, registry authority | policy/record/evidence hashes | exact 15 metrics, thresholds, source map, bindings, relations |
| EV11-08 | real authority graph and runner transcript | checker, governance, and compatibility authorities | command/report/input hashes | acyclic production order and no self-Git/future edge |
| EV11-09 | final current schema-definition anchors and catalog | plan author and independent reviewer | plan SHA/blob and row bytes | one anchor and one row per current identifier |
| EV11-10 | final ObjectReference usage matrix | plan author and independent reviewer | matrix bytes and plan SHA | every current parent field and derived context |
| EV11-11 | row-by-row catalog metadata review | plan author and independent reviewer | catalog row bytes | category, closure, producer, validator, stage, registry behavior |
| EV11-12 | matrix and inverse catalog projection | plan author and independent reviewer | matrix/catalog bytes | exact inverse, no second mapping source |
| EV11-13 | complete test specification and independent replay | plan author and independent reviewer | test-row bytes and replay transcript | zero unbound literal inputs and proper result types |
| EV11-14 | future real evidence separation review | independent reviewer | plan and F-EV row hashes | no KAT-to-real-evidence promotion edge |
| EV11-15 | campaign/static membership review | campaign and release authorities | manifest/release binding hashes | protocol-P static authority; package members only |
| EV11-16 | final Markdown rendering/fence transcript | plan author and independent reviewer | final plan SHA and lint transcript | balanced fences and normal heading/table rendering |
| EV11-17 | Phase-E compatibility and closed safety/scientific contract replay | compatibility and scientific reviewers | frozen Phase-E SHA/blob and replay hashes | no regression, P2 gate intact |
| EV11-18 | complete current identifier-set audit | plan author and independent reviewer | regex output, plan SHA/blob | set equality and multiplicity |
| EV11-19 | future KAT/evidence separation audit | independent reviewer | plan/test/F-EV hashes | KAT material never treated as real evidence |
| EV11-20 | current owner-decision matrix projection | plan author and independent reviewer | matrix bytes and plan SHA/blob | union exactly F-OD-01..20 |

TEST_TO_REAL_EVIDENCE_PROMOTION_PATHS=0.

### 53.12 Current R11 catalog and inverse projection

The following is the complete one-row-per-identifier catalog. Categories are
actual serialization roles: TOP_LEVEL_WIRE has complete JCS file bytes;
NESTED_WIRE is only an explicitly named enclosing field; TAG_BODY is the exact
annotated-tag message; PLAN_ONLY_CONTRACT has no wire or registry subject.

| identifier | category | exact field-closure pointer | semantic identity / complete-file hash meaning | concrete producer | actual validator | exact stage/set | exact registry behavior | traceability |
|---|---|---|---|---|---|---|---|---|
| PhaseFArgvV1 | NESTED_WIRE | #schema-def-PhaseFArgvV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFArgvV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFArgvV1] | derived union in USAGE_SET[PhaseFArgvV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFArgvV1) |
| PhaseFAuthorityEnrollmentApprovalV1 | TAG_BODY | #schema-def-PhaseFAuthorityEnrollmentApprovalV1 | no JSON semantic ID; SHA-256 exact ordered tag-message bytes | independent approval gate | annotated-tag grammar plus target/body/prerequisite validator | enrollment approval | TAG_BODY; Git annotated-tag message only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFAuthorityEnrollmentApprovalV1) |
| PhaseFAuthorityEnrollmentV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFAuthorityEnrollmentV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | authority-enrollment operation | enrollment strict parser plus identity/field validator | enrollment | TOP_LEVEL registered subject; record kind authority_enrolled; object kind authority_enrollment | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFAuthorityEnrollmentV1) |
| PhaseFChainOfCustodyV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFChainOfCustodyV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | campaign laboratory and custody authority | custody strict parser plus continuity/terminal-unit validation | F2-F4 physical-validation | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFChainOfCustodyV1) |
| PhaseFCheckListV1 | NESTED_WIRE | #schema-def-PhaseFCheckListV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFCheckListV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFCheckListV1] | derived union in USAGE_SET[PhaseFCheckListV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCheckListV1) |
| PhaseFCheckerBuildEvidenceV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFCheckerBuildEvidenceV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | checker build/readiness operation | checker-build strict parser plus freshness/transcript/toolchain validator | readiness/checker-invocation | TOP_LEVEL unregistered evidence; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCheckerBuildEvidenceV1) |
| PhaseFCheckerExitCodeV1 | NESTED_WIRE | #schema-def-PhaseFCheckerExitCodeV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFCheckerExitCodeV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFCheckerExitCodeV1] | derived union in USAGE_SET[PhaseFCheckerExitCodeV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCheckerExitCodeV1) |
| PhaseFCheckerReadinessEvidenceV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFCheckerReadinessEvidenceV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | checker build/readiness operation | readiness strict parser plus two-build/source/maintenance validator | readiness/checker-invocation | TOP_LEVEL unregistered evidence; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCheckerReadinessEvidenceV1) |
| PhaseFCheckerReportV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFCheckerReportV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | checker invocation | checker-report strict parser plus command/argv/stdout/exit/decision consistency validator | checker-invocation | TOP_LEVEL unregistered evidence; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCheckerReportV1) |
| PhaseFCheckerStdoutV1 | NESTED_WIRE | #schema-def-PhaseFCheckerStdoutV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFCheckerStdoutV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFCheckerStdoutV1] | derived union in USAGE_SET[PhaseFCheckerStdoutV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCheckerStdoutV1) |
| PhaseFClaimStateRecordV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFClaimStateRecordV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | release authority | claim-state strict parser plus transition/cause/relation validator | F4-F5+ release/state | TOP_LEVEL registered subject; record kind claim_state_changed; object kind claim_state | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFClaimStateRecordV1) |
| PhaseFCohortLockRecordV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFCohortLockRecordV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | release authority after scientific lock | cohort strict parser plus binding/chronology validator | F2-F4 cohort lock | TOP_LEVEL registered subject; record kind cohort_locked; object kind cohort_lock | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCohortLockRecordV1) |
| PhaseFCommandV1 | NESTED_WIRE | #schema-def-PhaseFCommandV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFCommandV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFCommandV1] | derived union in USAGE_SET[PhaseFCommandV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCommandV1) |
| PhaseFCustodyEventV1 | NESTED_WIRE | #schema-def-PhaseFCustodyEventV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFCustodyEventV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFCustodyEventV1] | derived union in USAGE_SET[PhaseFCustodyEventV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFCustodyEventV1) |
| PhaseFDecisionApprovalV1 | TAG_BODY | #schema-def-PhaseFDecisionApprovalV1 | no JSON semantic ID; SHA-256 exact ordered tag-message bytes | independent approval gate | annotated-tag grammar plus target/body/prerequisite validator | F0 approval | TAG_BODY; Git annotated-tag message only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDecisionApprovalV1) |
| PhaseFDecisionBundleV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFDecisionBundleV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | F0 decision authority | decision-bundle strict parser plus exact 20-value/projection validator | F0 decision-bundle | TOP_LEVEL unregistered authority file; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDecisionBundleV1) |
| PhaseFDecisionRowV1 | NESTED_WIRE | #schema-def-PhaseFDecisionRowV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFDecisionRowV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFDecisionRowV1] | derived union in USAGE_SET[PhaseFDecisionRowV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDecisionRowV1) |
| PhaseFDecisionValueV1 | NESTED_WIRE | #schema-def-PhaseFDecisionValueV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFDecisionValueV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFDecisionValueV1] | derived union in USAGE_SET[PhaseFDecisionValueV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDecisionValueV1) |
| PhaseFDependencyAuditV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFDependencyAuditV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | retrieval/package authority | dependency strict parser plus manifest/DAG/classification validator | F2 retrieval/package | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDependencyAuditV1) |
| PhaseFDependencyEdgeV1 | NESTED_WIRE | #schema-def-PhaseFDependencyEdgeV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFDependencyEdgeV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFDependencyEdgeV1] | derived union in USAGE_SET[PhaseFDependencyEdgeV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDependencyEdgeV1) |
| PhaseFDeviationEventV1 | NESTED_WIRE | #schema-def-PhaseFDeviationEventV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFDeviationEventV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFDeviationEventV1] | derived union in USAGE_SET[PhaseFDeviationEventV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDeviationEventV1) |
| PhaseFDeviationLedgerRevisionV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFDeviationLedgerRevisionV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | Phase-F deviation authority | deviation-revision strict parser plus predecessor/action/event validator | F1-F4 deviation | TOP_LEVEL unregistered evidence; package/cohort/execution bindings; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDeviationLedgerRevisionV1) |
| PhaseFDeviationLedgerV1 | PLAN_ONLY_CONTRACT | #schema-def-PhaseFDeviationLedgerV1 | no artifact identity; no complete-file bytes | plan author | plan consistency validator; no runtime artifact validator | F1-F4 deviation planning | PLAN_ONLY; no wire and no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFDeviationLedgerV1) |
| PhaseFEndpointMetrologyPolicyV1 | NESTED_WIRE | #schema-def-PhaseFEndpointMetrologyPolicyV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFEndpointMetrologyPolicyV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFEndpointMetrologyPolicyV1] | derived union in USAGE_SET[PhaseFEndpointMetrologyPolicyV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFEndpointMetrologyPolicyV1) |
| PhaseFEnvironmentEntryV1 | NESTED_WIRE | #schema-def-PhaseFEnvironmentEntryV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFEnvironmentEntryV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFEnvironmentEntryV1] | derived union in USAGE_SET[PhaseFEnvironmentEntryV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFEnvironmentEntryV1) |
| PhaseFExecutionRecordV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFExecutionRecordV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | release authority | execution strict parser plus cohort/owner/protocol/time/result validator | F4 execution | TOP_LEVEL registered subject; record kind execution_registered; object kind execution_record | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFExecutionRecordV1) |
| PhaseFF5ReleaseCandidateV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFF5ReleaseCandidateV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | release authority | F5-candidate strict parser plus binding/chronology validator | F5 pre-ACTIVE | TOP_LEVEL unregistered evidence; reviewed candidate target; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFF5ReleaseCandidateV1) |
| PhaseFIdentityComparisonV1 | NESTED_WIRE | #schema-def-PhaseFIdentityComparisonV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFIdentityComparisonV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFIdentityComparisonV1] | derived union in USAGE_SET[PhaseFIdentityComparisonV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFIdentityComparisonV1) |
| PhaseFIncidentRecordV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFIncidentRecordV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | incident operations authority | incident strict parser plus scope/type/action/open-status validator | incident detection | TOP_LEVEL registered subject; record kind incident_recorded; object kind incident_record | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFIncidentRecordV1) |
| PhaseFIncidentResolutionV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFIncidentResolutionV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | incident operations authority | resolution strict parser plus incident binding/number/predecessor/legal progression validator | incident resolution | TOP_LEVEL registered subject; record kind incident_resolution_recorded; object kind incident_resolution | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFIncidentResolutionV1) |
| PhaseFIncidentScopeV1 | NESTED_WIRE | #schema-def-PhaseFIncidentScopeV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFIncidentScopeV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFIncidentScopeV1] | derived union in USAGE_SET[PhaseFIncidentScopeV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFIncidentScopeV1) |
| PhaseFIndependentReviewBundleV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFIndependentReviewBundleV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | independent five-role review panel | review-bundle strict parser plus target/role/count/aggregate validator | all review gates | TOP_LEVEL unregistered review evidence; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFIndependentReviewBundleV1) |
| PhaseFIndependentReviewV1 | NESTED_WIRE | #schema-def-PhaseFIndependentReviewV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFIndependentReviewV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFIndependentReviewV1] | derived union in USAGE_SET[PhaseFIndependentReviewV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFIndependentReviewV1) |
| PhaseFLODLOQPolicyV1 | NESTED_WIRE | #schema-def-PhaseFLODLOQPolicyV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFLODLOQPolicyV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFLODLOQPolicyV1] | derived union in USAGE_SET[PhaseFLODLOQPolicyV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFLODLOQPolicyV1) |
| PhaseFLocationLedgerV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFLocationLedgerV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | location authority | location-ledger strict parser plus identity/type validator | F2 physical-validation | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFLocationLedgerV1) |
| PhaseFLocationV1 | NESTED_WIRE | #schema-def-PhaseFLocationV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFLocationV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFLocationV1] | derived union in USAGE_SET[PhaseFLocationV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFLocationV1) |
| PhaseFMethodVersionV1 | NESTED_WIRE | #schema-def-PhaseFMethodVersionV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFMethodVersionV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFMethodVersionV1] | derived union in USAGE_SET[PhaseFMethodVersionV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMethodVersionV1) |
| PhaseFMetricThresholdV1 | NESTED_WIRE | #schema-def-PhaseFMetricThresholdV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFMetricThresholdV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFMetricThresholdV1] | derived union in USAGE_SET[PhaseFMetricThresholdV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMetricThresholdV1) |
| PhaseFMetrologyCheckResultV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFMetrologyCheckResultV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | metrology laboratory | check-result strict parser plus endpoint/check lookup/comparator/unit validator | F2 metrology/reference | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMetrologyCheckResultV1) |
| PhaseFMetrologyCheckSpecV1 | NESTED_WIRE | #schema-def-PhaseFMetrologyCheckSpecV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFMetrologyCheckSpecV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFMetrologyCheckSpecV1] | derived union in USAGE_SET[PhaseFMetrologyCheckSpecV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMetrologyCheckSpecV1) |
| PhaseFMetrologyPolicyV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFMetrologyPolicyV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | metrology authority | metrology-policy strict parser plus endpoint/method/check/LOD-LOQ validator | F0-F2 metrology/reference | TOP_LEVEL unregistered authority file; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMetrologyPolicyV1) |
| PhaseFMonitoringBreachV1 | NESTED_WIRE | #schema-def-PhaseFMonitoringBreachV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFMonitoringBreachV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFMonitoringBreachV1] | derived union in USAGE_SET[PhaseFMonitoringBreachV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMonitoringBreachV1) |
| PhaseFMonitoringEvidenceV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFMonitoringEvidenceV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | monitoring operations authority | monitoring-evidence strict parser plus source-kind/metric/window/value validator | F5+ monitoring evidence | TOP_LEVEL unregistered evidence; referenced by monitoring_recorded; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMonitoringEvidenceV1) |
| PhaseFMonitoringMeasurementV1 | NESTED_WIRE | #schema-def-PhaseFMonitoringMeasurementV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFMonitoringMeasurementV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFMonitoringMeasurementV1] | derived union in USAGE_SET[PhaseFMonitoringMeasurementV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMonitoringMeasurementV1) |
| PhaseFMonitoringPolicyV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFMonitoringPolicyV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | F0 owner and monitoring authority | monitoring-policy strict parser plus fixed-order/threshold/action validator | F0 monitoring policy | TOP_LEVEL unregistered authority file; release binding only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMonitoringPolicyV1) |
| PhaseFMonitoringRecordV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFMonitoringRecordV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | monitoring operations authority | monitoring strict parser plus 15-metric/evidence/breach/window/registry validator | F5+ monitoring | TOP_LEVEL registered subject; record kind monitoring_recorded; object kind monitoring_record | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMonitoringRecordV1) |
| PhaseFMonitoringSourceReferenceV1 | NESTED_WIRE | #schema-def-PhaseFMonitoringSourceReferenceV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFMonitoringSourceReferenceV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFMonitoringSourceReferenceV1] | derived union in USAGE_SET[PhaseFMonitoringSourceReferenceV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMonitoringSourceReferenceV1) |
| PhaseFMonitoringValueV1 | NESTED_WIRE | #schema-def-PhaseFMonitoringValueV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFMonitoringValueV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFMonitoringValueV1] | derived union in USAGE_SET[PhaseFMonitoringValueV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMonitoringValueV1) |
| PhaseFNamedDigestV1 | NESTED_WIRE | #schema-def-PhaseFNamedDigestV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFNamedDigestV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFNamedDigestV1] | derived union in USAGE_SET[PhaseFNamedDigestV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFNamedDigestV1) |
| PhaseFObjectDigestV1 | NESTED_WIRE | #schema-def-PhaseFObjectDigestV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFObjectDigestV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFObjectDigestV1] | derived union in USAGE_SET[PhaseFObjectDigestV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFObjectDigestV1) |
| PhaseFObjectReferenceV1 | NESTED_WIRE | #schema-def-PhaseFObjectReferenceV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFObjectReferenceV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFObjectReferenceV1] | derived union in USAGE_SET[PhaseFObjectReferenceV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFObjectReferenceV1) |
| PhaseFOutputSpecV1 | NESTED_WIRE | #schema-def-PhaseFOutputSpecV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFOutputSpecV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFOutputSpecV1] | derived union in USAGE_SET[PhaseFOutputSpecV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFOutputSpecV1) |
| PhaseFPackageBindingV1 | NESTED_WIRE | #schema-def-PhaseFPackageBindingV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFPackageBindingV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFPackageBindingV1] | derived union in USAGE_SET[PhaseFPackageBindingV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPackageBindingV1) |
| PhaseFPackageManifestV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFPackageManifestV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | retrieval/package authority | package strict parser plus object/role/binding/duplicate/DAG validator | F2 package | TOP_LEVEL registered subject; record kind package_registered; object kind package_manifest | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPackageManifestV1) |
| PhaseFPackageObjectV1 | NESTED_WIRE | #schema-def-PhaseFPackageObjectV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFPackageObjectV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFPackageObjectV1] | derived union in USAGE_SET[PhaseFPackageObjectV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPackageObjectV1) |
| PhaseFParameterSpecV1 | NESTED_WIRE | #schema-def-PhaseFParameterSpecV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFParameterSpecV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFParameterSpecV1] | derived union in USAGE_SET[PhaseFParameterSpecV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFParameterSpecV1) |
| PhaseFParameterValueRowV1 | NESTED_WIRE | #schema-def-PhaseFParameterValueRowV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFParameterValueRowV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFParameterValueRowV1] | derived union in USAGE_SET[PhaseFParameterValueRowV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFParameterValueRowV1) |
| PhaseFPhysicalIdentityAuditV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFPhysicalIdentityAuditV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | physical identity authority | identity-audit strict parser plus alias/independence validator | F2 physical-validation | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPhysicalIdentityAuditV1) |
| PhaseFPhysicalReleaseApprovalV1 | TAG_BODY | #schema-def-PhaseFPhysicalReleaseApprovalV1 | no JSON semantic ID; SHA-256 exact ordered tag-message bytes | independent approval gate | annotated-tag grammar plus target/body/prerequisite validator | physical release approval | TAG_BODY; Git annotated-tag message only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPhysicalReleaseApprovalV1) |
| PhaseFPhysicalUnitLedgerV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFPhysicalUnitLedgerV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | campaign laboratory | unit-ledger strict parser plus native-key/parent-child validator | F2 physical-validation | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPhysicalUnitLedgerV1) |
| PhaseFPlanApprovalV1 | TAG_BODY | #schema-def-PhaseFPlanApprovalV1 | no JSON semantic ID; SHA-256 exact ordered tag-message bytes | independent approval gate | annotated-tag grammar plus target/body/prerequisite validator | plan approval | TAG_BODY; Git annotated-tag message only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPlanApprovalV1) |
| PhaseFPowerAnalysisRecordV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFPowerAnalysisRecordV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | power-analysis authority | power strict parser plus interface/type/range/unit/sensitivity/review-order validator | F1 power | TOP_LEVEL registered subject; record kind power_registered; object kind power_analysis | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPowerAnalysisRecordV1) |
| PhaseFPowerMethodInterfaceV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFPowerMethodInterfaceV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | power-analysis authority | power-interface strict parser plus method/parameter/output/range/unit validator | F1 power | TOP_LEVEL unregistered authority file; power relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPowerMethodInterfaceV1) |
| PhaseFPowerOutputValueV1 | NESTED_WIRE | #schema-def-PhaseFPowerOutputValueV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFPowerOutputValueV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFPowerOutputValueV1] | derived union in USAGE_SET[PhaseFPowerOutputValueV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFPowerOutputValueV1) |
| PhaseFProtocolProjectionV1 | PLAN_ONLY_CONTRACT | #schema-def-PhaseFProtocolProjectionV1 | no artifact identity; no complete-file bytes | F0 projection audit | plan consistency validator; no runtime artifact validator | F0 projection | PLAN_ONLY; no wire and no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFProtocolProjectionV1) |
| PhaseFQuantifiedUncertaintyV1 | NESTED_WIRE | #schema-def-PhaseFQuantifiedUncertaintyV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFQuantifiedUncertaintyV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFQuantifiedUncertaintyV1] | derived union in USAGE_SET[PhaseFQuantifiedUncertaintyV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFQuantifiedUncertaintyV1) |
| PhaseFRangeRuleV1 | NESTED_WIRE | #schema-def-PhaseFRangeRuleV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFRangeRuleV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFRangeRuleV1] | derived union in USAGE_SET[PhaseFRangeRuleV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRangeRuleV1) |
| PhaseFReadinessApprovalV1 | TAG_BODY | #schema-def-PhaseFReadinessApprovalV1 | no JSON semantic ID; SHA-256 exact ordered tag-message bytes | independent approval gate | annotated-tag grammar plus target/body/prerequisite validator | readiness approval | TAG_BODY; Git annotated-tag message only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReadinessApprovalV1) |
| PhaseFReferenceAssessmentV1 | NESTED_WIRE | #schema-def-PhaseFReferenceAssessmentV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFReferenceAssessmentV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFReferenceAssessmentV1] | derived union in USAGE_SET[PhaseFReferenceAssessmentV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReferenceAssessmentV1) |
| PhaseFReferenceResultV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFReferenceResultV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | metrology laboratory | reference-result strict parser plus endpoint/provenance/independence/ceiling validator | F2 metrology/reference | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReferenceResultV1) |
| PhaseFReferenceSourceDescriptorV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFReferenceSourceDescriptorV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | metrology laboratory | source-descriptor strict parser plus origin/completeness/dependency validator | F2 metrology/reference | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReferenceSourceDescriptorV1) |
| PhaseFRegistryCompromiseEmergencyV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFRegistryCompromiseEmergencyV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | security emergency authority | emergency strict parser plus exact-path/review/tree/claim-status validator | emergency | TOP_LEVEL unregistered emergency file; outer Git publication attestation; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRegistryCompromiseEmergencyV1) |
| PhaseFRegistryHeadV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFRegistryHeadV1 | no content-derived semantic ID; SHA-256 complete JCS file bytes | registry authority | registry-head strict parser plus namespace/sequence/signature resolver validator | all registry operations | TOP_LEVEL resolver object; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRegistryHeadV1) |
| PhaseFRegistryRecordV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFRegistryRecordV1 | no content-derived semantic ID; SHA-256 complete JCS file bytes | registry authority | registry-record strict parser plus sequence/predecessor/signature/subject/relation validator | all registry operations | TOP_LEVEL signed registry-chain record; not a registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRegistryRecordV1) |
| PhaseFRegistryRelationV1 | NESTED_WIRE | #schema-def-PhaseFRegistryRelationV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFRegistryRelationV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFRegistryRelationV1] | derived union in USAGE_SET[PhaseFRegistryRelationV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRegistryRelationV1) |
| PhaseFReinstatementApprovalV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFReinstatementApprovalV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | release/governance authority | reinstatement strict parser plus trigger/evidence/review validator | F5+ state transition | TOP_LEVEL unregistered governance evidence; referenced by claim-state relation; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReinstatementApprovalV1) |
| PhaseFReleaseRecordV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFReleaseRecordV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | release authority | release strict parser plus binding/chronology/registry validator | F4-F5 release | TOP_LEVEL registered subject; record kind release_registered; object kind release_record | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReleaseRecordV1) |
| PhaseFRetentionAuditV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFRetentionAuditV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | retention auditor | retention strict parser plus exact-set/identity/copy/chronology validator | campaign pre-release and release-retention | TOP_LEVEL registered subject; record kind retention_audit_recorded; object kind retention_audit | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRetentionAuditV1) |
| PhaseFRetentionCopyVerificationV1 | NESTED_WIRE | #schema-def-PhaseFRetentionCopyVerificationV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFRetentionCopyVerificationV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFRetentionCopyVerificationV1] | derived union in USAGE_SET[PhaseFRetentionCopyVerificationV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRetentionCopyVerificationV1) |
| PhaseFRetentionObjectCheckV1 | NESTED_WIRE | #schema-def-PhaseFRetentionObjectCheckV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFRetentionObjectCheckV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFRetentionObjectCheckV1] | derived union in USAGE_SET[PhaseFRetentionObjectCheckV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRetentionObjectCheckV1) |
| PhaseFRetentionObjectV1 | NESTED_WIRE | #schema-def-PhaseFRetentionObjectV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFRetentionObjectV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFRetentionObjectV1] | derived union in USAGE_SET[PhaseFRetentionObjectV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRetentionObjectV1) |
| PhaseFRetentionScopeV1 | NESTED_WIRE | #schema-def-PhaseFRetentionScopeV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFRetentionScopeV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFRetentionScopeV1] | derived union in USAGE_SET[PhaseFRetentionScopeV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRetentionScopeV1) |
| PhaseFRetrievalVerificationV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFRetrievalVerificationV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | retrieval/package authority | retrieval strict parser plus URI/hash/length/availability verifier | F2 retrieval | TOP_LEVEL unregistered evidence; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFRetrievalVerificationV1) |
| PhaseFReviewTargetV1 | NESTED_WIRE | #schema-def-PhaseFReviewTargetV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFReviewTargetV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFReviewTargetV1] | derived union in USAGE_SET[PhaseFReviewTargetV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReviewTargetV1) |
| PhaseFScientificAdmissibilityAuditV1 | TOP_LEVEL_WIRE | #schema-def-PhaseFScientificAdmissibilityAuditV1 | §3 content-derived semantic ID excluding only own ID; SHA-256 complete JCS file bytes | scientific/metrology reviewer | admissibility strict parser plus identity/dependency/ceiling validator | F2 scientific audit | TOP_LEVEL unregistered evidence; package relation only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFScientificAdmissibilityAuditV1) |
| PhaseFSensitivityCaseV1 | NESTED_WIRE | #schema-def-PhaseFSensitivityCaseV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFSensitivityCaseV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFSensitivityCaseV1] | derived union in USAGE_SET[PhaseFSensitivityCaseV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFSensitivityCaseV1) |
| PhaseFSensitivityOverrideV1 | NESTED_WIRE | #schema-def-PhaseFSensitivityOverrideV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFSensitivityOverrideV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFSensitivityOverrideV1] | derived union in USAGE_SET[PhaseFSensitivityOverrideV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFSensitivityOverrideV1) |
| PhaseFTrustProvisioningApprovalV1 | TAG_BODY | #schema-def-PhaseFTrustProvisioningApprovalV1 | no JSON semantic ID; SHA-256 exact ordered tag-message bytes | independent approval gate | annotated-tag grammar plus target/body/prerequisite validator | F3 trust approval | TAG_BODY; Git annotated-tag message only; no registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFTrustProvisioningApprovalV1) |
| PhaseFUncertaintyPolicyV1 | NESTED_WIRE | #schema-def-PhaseFUncertaintyPolicyV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFUncertaintyPolicyV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFUncertaintyPolicyV1] | derived union in USAGE_SET[PhaseFUncertaintyPolicyV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFUncertaintyPolicyV1) |
| PhaseFUnitEntryV1 | NESTED_WIRE | #schema-def-PhaseFUnitEntryV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFUnitEntryV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFUnitEntryV1] | derived union in USAGE_SET[PhaseFUnitEntryV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFUnitEntryV1) |
| PhaseFUnitRuleV1 | NESTED_WIRE | #schema-def-PhaseFUnitRuleV1 | no independent identity; containing parent JCS bytes define storage identity | derived from USAGE_SET[PhaseFUnitRuleV1] | intrinsic type validator plus every enclosing semantic check in USAGE_SET[PhaseFUnitRuleV1] | derived union in USAGE_SET[PhaseFUnitRuleV1] | NESTED; no independent registry subject | INVERSE(R11_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFUnitRuleV1) |

The inverse projection is the only traceability source for the final column.
Every NESTED_WIRE producer, validator, and stage cell is derived from the
fully enumerated usage matrix; no nested schema inherits a registry subject.
The row for PhaseFObjectReferenceV1 therefore has all 12 producer contexts and
all 12 enclosing validator/stage contexts, rather than a single laboratory
producer.

CATALOG_WRONG_SECTION_POINTERS=0.
CATALOG_FIELD_CLOSURE_AMBIGUITIES=0.
CATALOG_PRODUCER_AMBIGUITIES=0.
CATALOG_VALIDATOR_AMBIGUITIES=0.
CATALOG_STAGE_AMBIGUITIES=0.
CATALOG_REGISTRY_BEHAVIOR_AMBIGUITIES=0.
SEMANTICALLY_GENERIC_CATALOG_ROWS=0.
CATALOG_TRACEABILITY_DERIVATION_MISMATCHES=0.
SCHEMA_DEFINITION_ANCHOR_MISSING=0.
SCHEMA_DEFINITION_ANCHOR_DUPLICATES=0.
CATALOG_CATEGORY_MISMATCHES=0.

### 53.13 R11 remediation ledger

Exactly these five current R11 remediation IDs are created. Each author
disposition is REMEDIATED or OPEN; none is CLOSED.

| review finding | root cause | R11 exact sections | remediation | current R11 requirements | ACs | tests | F-EVs | AUTHOR DISPOSITION |
|---|---|---|---|---|---|---|---|---|
| F-PLAN-R11-P1-01 | R10 conflated arbitrary retention storage-copy bytes with canonical production-schema object bytes. | §§53.2-53.4,53.10 | split production_schema_kat from retention_storage_kat; define R11RetentionStorageFixtureV1 and prevalidated identity precondition; bind incident retention members to real canonical JSON. | R11-03,R11-04,R11-05,R11-14 | AC11-03,AC11-04,AC11-05,AC11-14 | R11-KAT-INCIDENT,R11-KAT-RETENTION-COPY,R11-CX-01,R11-CX-02,R11-CX-03,R11-CX-04 | EV11-03,EV11-04,EV11-05,EV11-14 | REMEDIATED |
| F-PLAN-R11-P1-02 | R10 catalog used non-exact metadata and section pointers that did not resolve to stable definitions. | §§53.7,53.8,53.12 | add one stable anchor per current identifier, exact distributed closures, complete usage matrix, and row-by-row operational metadata. | R11-09,R11-10,R11-11,R11-12,R11-18 | AC11-09,AC11-10,AC11-11,AC11-12,AC11-18 | R11-CAT,R11-TRACE,R11-CX-10,R11-CX-11,R11-CX-12,R11-CX-13 | EV11-09,EV11-10,EV11-11,EV11-12,EV11-18 | REMEDIATED |
| F-PLAN-R11-P1-03 | R10 positive procedures used symbolic or incomplete plan/trust/DAG and monitoring surfaces. | §§53.5-53.6,53.10 | materialize literal plan/trust KATs, replace T10-POS-DAG with explicit R11-DAG-AUDIT, and narrow monitoring to a property unless all 15 predicates are literal. | R11-01,R11-02,R11-07,R11-08,R11-13 | AC11-01,AC11-02,AC11-07,AC11-08,AC11-13 | R11-POS-PLAN,R11-POS-TRUST,R11-DAG-AUDIT,R11-PROP-MONITORING,R11-CX-05,R11-CX-06,R11-CX-07,R11-CX-08,R11-CX-09,R11-CX-16,R11-CX-17,R11-CX-18 | EV11-01,EV11-02,EV11-07,EV11-08,EV11-13 | REMEDIATED |
| F-PLAN-R11-P1-04 | T10-CX-28 treated protocol-P as a campaign-set member. | §§53.3,53.6,53.9-53.11 | define campaign as manifest plus package objects; classify protocol-P as STATIC RELEASE AUTHORITY; use package-O1 for campaign removal and protocol-P for static-authority removal. | R11-05,R11-06,R11-15 | AC11-05,AC11-06,AC11-15 | R11-CX-14,R11-CX-15 | EV11-05,EV11-06,EV11-15 | REMEDIATED |
| F-PLAN-R11-P3-01 | An accidental unmatched Markdown fence caused later R10 sections to render as code. | §§42-52 historical fence repair and §§53.10-53.12 fence audit | close the historical R7 handoff fence, remove the stray fence before historical R10 §48, and run full backtick/tilde state lint. | R11-16 | AC11-16 | R11-CX-19,R11-CX-20 | EV11-16 | REMEDIATED |

### 53.14 Author audit, baseline, and R11 handoff

These are author checks, not independent approval. The current normative
identifier count is 91, catalog count is 91, current R11 test count is 28,
current requirements/ACs/F-EVs are 20 each, and owner-decision union is exactly
F-OD-01 through F-OD-20. All counters below were recomputed from the current
R11 sections after editing:

~~~text
RETENTION_KAT_BYTE_SCOPE=STORAGE_ONLY
STORAGE_FIXTURE_TO_SCHEMA_BYTE_PROMOTION_PATHS=0
INCIDENT_SCHEMA_FIXTURE_SHA_BINDING_AMBIGUITIES=0
KAT_SCOPE_OVERCLAIM_PATHS=0
CAMPAIGN_STATIC_MEMBERSHIP_CONTRADICTIONS=0
LITERAL_KAT_UNBOUND_INPUTS=0
PROPERTY_TEST_LITERAL_PASS_OVERCLAIMS=0
CURRENT_TEST_PROCEDURE_GAPS=0
SCHEMA_DEFINITION_ANCHOR_MISSING=0
SCHEMA_DEFINITION_ANCHOR_DUPLICATES=0
OBJECT_REFERENCE_USAGE_GAPS=0
CATALOG_CATEGORY_MISMATCHES=0
CATALOG_FIELD_CLOSURE_AMBIGUITIES=0
CATALOG_PRODUCER_AMBIGUITIES=0
CATALOG_VALIDATOR_AMBIGUITIES=0
CATALOG_STAGE_AMBIGUITIES=0
CATALOG_REGISTRY_BEHAVIOR_AMBIGUITIES=0
SEMANTICALLY_GENERIC_CATALOG_ROWS=0
CATALOG_TRACEABILITY_DERIVATION_MISMATCHES=0
CONTRADICTORY_CURRENT_TRACEABILITY_TABLES=0
TRACEABILITY_SUBSTANCE_GAPS=0
TEST_TO_REAL_EVIDENCE_PROMOTION_PATHS=0
UNMATCHED_BACKTICK_FENCES=0
UNMATCHED_TILDE_FENCES=0
HEADINGS_INSIDE_ACCIDENTAL_FENCE=0
MARKDOWN_STRUCTURE_PASS=yes
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_R11=0
POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES=0
NORMATIVE_CONTRADICTIONS=0
UNMAPPED_REQUIREMENTS=0
UNMAPPED_ACS=0
UNMAPPED_TESTS=0
UNMAPPED_EVIDENCE=0
UNMAPPED_ODS=0
EXTRA_OD_REFERENCES=0
TRACEABILITY_SUBSTANCE_GAPS=0
TEST_TO_REAL_EVIDENCE_PROMOTION_PATHS=0
SYNTHETIC_TO_PHYSICAL=0
CONSTRUCTED_TO_PHYSICAL=0
UNKNOWN_TO_PHYSICAL=0
TEST_TO_PHYSICAL=0
SAME_SOURCE_INDEPENDENCE=0
UNDECLARED_DEPENDENCY_INDEPENDENCE=0
PSEUDOREPLICATION=0
UNDERPOWERED_TO_PASS=0
F4_TO_ACTIVE=0
COMPROMISED_AUTHORITY_BYPASS=0
PRIVATE_KEY_REPOSITORY_PATHS=0
PHASE_E_COMPATIBILITY=CLOSED
PRODUCTION_RUNNER_ORDER=PASS
P2_GATE=PASS
~~~

Two conceptual implementers independently reading only this R11 plan reach the
same determinations: storage rows are STORAGE_ONLY; I2/IR2-C/IR2-R are real
canonical JSON; IR2-R predecessor is the exact IR2-C complete-file SHA;
campaign members are manifest plus package objects; protocol-P is STATIC
RELEASE AUTHORITY; ObjectReference contexts are the 12 matrix rows; producer,
validator, stage, anchor, and inverse-projection cells are derived as stated;
and the Markdown ending is outside every fence.
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_R11=0.
The positive path is compositional: production schema contracts, specific
complete schema KATs, the constructive DAG audit, retention membership
precondition, storage-copy KAT, and future real-evidence oracles. No layer
supplies another layer's evidence without an explicit precondition.
POSITIVE_PATH_CONSTRUCTION_AMBIGUITIES=0.

Required baseline commands are:
~~~text
git diff --check
cargo fmt --all --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --test phase_e_validation
cargo test --locked --test phase_d_reporting_public_output
~~~
The frozen Phase-E plan must remain SHA-256
0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33 and Git blob
6fce9d13a42a09027e0e730874a8d80e03e6a7da. The live-remote verification is
required; if DNS or authorization prevents it, the handoff records NOT VERIFIED
and no push is attempted.

MHI V1 PHASE F
R11 PLANNING REMEDIATION HANDOFF

STARTING R10 SHA:
341f9a805f94e8dd2a58c3beb7c3a68cf6adf3c7
R10 PLAN SHA-256:
3832fd6feaba98e834f288760c1741fa0a1bdfe1d6a1ab254cf9bcd1ce05e073
R10 PLAN GIT BLOB:
ca9bf58546f31f18ce0a35046dee1f46b55f9ec0
R11 PLAN REVIEW SHA:
<externally frozen after final planning-only commit; not embedded in plan>
R11 PLAN SHA-256:
<computed after final R11 bytes; not embedded in plan>
R11 PLAN GIT BLOB:
<computed after final R11 bytes; not embedded in plan>
CHANGED FILES:
1 expected

R10 OPEN P1 REMEDIATION:
F-PLAN-R11-P1-01: REMEDIATED
F-PLAN-R11-P1-02: REMEDIATED
F-PLAN-R11-P1-03: REMEDIATED
F-PLAN-R11-P1-04: REMEDIATED
P3:
F-PLAN-R11-P3-01: REMEDIATED

KAT LAYERS:
storage fixture role: STORAGE_ONLY
storage-to-schema promotion paths: 0
incident fixture role: FULL_SCHEMA_OBJECT
I2 semantic ID: sha256:c53b6f2230cbe25034dfcfe572cb845e1e29fe0a9f730549ed0ec464d71b8353
IR2-C semantic ID: sha256:4f0846d55e1f38335cc8e1a62f963e95c374c6351f2c9358143e673ec76a7dc7
IR2-C complete SHA: e1213d5261a13111eb857c401009cbf247662d1f28a5abf1f28c90aa0cd6cccf
IR2-R semantic ID: sha256:42e6ff499eac64a5abb5858a4eb2ac260a2d833ebcaf39bea65f7f10579069b9
IR2-R predecessor: MATCH

TEST MODEL:
literal executable KAT count: 8
property test count: 4
constructive plan audit count: 16
literal KAT unbound inputs: 0
property tests claiming literal PASS: 0
incomplete PASS tests: 0

MEMBERSHIP:
campaign/static contradictions: 0
protocol-P classified as: STATIC RELEASE AUTHORITY
campaign member negative test uses: package-O1

CATALOG:
normative PhaseF identifier count: 91
catalog identifier count: 91
missing identifiers: 0
extra identifiers: 0
duplicate identifiers: 0
missing definition anchors: 0
duplicate definition anchors: 0
category mismatches: 0
field-closure ambiguities: 0
producer ambiguities: 0
validator ambiguities: 0
stage ambiguities: 0
registry-behavior ambiguities: 0
ObjectReference usage gaps: 0
semantically generic rows: 0
traceability derivation mismatches: 0

TRACEABILITY:
current R11 requirements: 20
ACs: 20
tests: 28
F-EVs: 20
owner decisions: 20 expected
unmapped requirements: 0
unmapped ACs: 0
unmapped tests: 0
unmapped evidence: 0
unmapped ODs: 0
extra ODs: 0
contradictory current tables: 0
traceability substance gaps: 0

MARKDOWN:
unmatched backtick fences: 0
unmatched tilde fences: 0
headings inside accidental fence: 0
structure: PASS

POSITIVE PATH:
complete DAG constructible: yes
construction ambiguities: 0

BASELINE:
diff: PENDING
fmt: PENDING
check: PENDING
strict Clippy: PENDING
Clippy diagnostics: PENDING
Phase E: PENDING/38
Phase D: PENDING/73
FROZEN PHASE-E PLAN: PENDING
LIVE REMOTE MAIN: NOT VERIFIED (DNS resolution failure)
PLAN TAG CREATED: NO expected
IMPLEMENTATION BRANCH: NO expected
F0 STARTED: NO
KEYS CREATED: NO
EVIDENCE CREATED: NO
CLAIMS CREATED: NO
WORKTREE CLEAN: after one forward commit
READY_FOR_FRESH_PHASE_F_R11_PLAN_REREVIEW: yes after local checks; remote verification pending
READY_FOR_PHASE_F_PLAN_APPROVAL_TAG: NO expected pending fresh R11 GO
READY_FOR_PHASE_F_IMPLEMENTATION: NO
