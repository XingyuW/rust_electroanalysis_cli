# MHI V1 Phase F — R5 planning-only positive-path closure

## 1. Authority, status, scope, and chronology

This document is the Phase-F R5 planning remediation of the independently
rereviewed R4 plan. It changes only this plan document. It does not create a
schema file, checker, tag, branch, key, signature, trust root, registry record,
physical evidence, monitoring record, claim, production implementation, new
scientific model, or new scientific scope.

The starting authority is exact:

| Authority | Value |
|---|---|
| R4 plan-review SHA | `8124dda4d6a358397a4bc899024bdc4a59fbc14c` |
| R4 plan SHA-256 | `52e3f05fc2783f340b1178a757a292dda02e6a6efa3eea05d4cbba7dbe9677f7` |
| R4 plan Git blob | `37547a23ef66bc8f45e8c550de6b67c360d73d5a` |
| R4 rereview | `P0=0`, `P1=14`, `P2=0`, `P3=0`, `PLAN_DECISION=NO-GO`, `PLAN_AUTHORITY=FAIL` |
| R5 status | forward remediation; independent R5 rereview `PENDING` |
| plan approval tag | absent; must remain absent in R5 |
| implementation branch | absent; must remain absent in R5 |

The immutable Phase-E authority is not changed: integrated baseline
`14942a30928b88f16914bf0bb103cc0c2a5bfa76`, reviewed implementation
`5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb`, frozen plan SHA-256
`0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`, and
frozen plan blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

R1 was `NO-GO/P1=13`; R2 was `NO-GO/P1=10`; R3 was `NO-GO/P1=19`; R4 was
`NO-GO/P1=14`. No rejected version is described as approved. The exact future
order remains: R5 rereview, plan approval, F0, F-IMPL-1 checker and permanent
F-MAINT-01/02 closure, readiness, enrollment, genesis, F1, F2, F3, F4, and F5.
F1-F5 remain blocked until the applicable approved tags and authority objects
exist.

`F_IMPL_1_BEFORE_F0_EXIT`, `F_IMPL_2_BEFORE_F0_EXIT`,
`F_IMPL_3_BEFORE_F0_EXIT`, and `F_IMPL_4_BEFORE_F0_EXIT` are forbidden.
R5 author audit is not independent approval.

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
  | physical_release_approval | emergency_registry_compromise
PHASE_F_REVIEW_ROLE_V1 = scientific_metrology | architecture_data | security
  | compatibility | operations_governance
PHASE_F_CHECKER_DECISION_V1 = pass | no_go | active | not_active | authority_unavailable
PHASE_F_INCIDENT_STATUS_V1 = open | contained | resolved | superseded
PHASE_F_INCIDENT_TYPE_V1 = key_compromise | key_revocation | registry_equivocation
  | data_integrity | custody_break | undeclared_dependency | monitoring_breach
  | reference_qc_breach | domain_breach | retention_failure | other_registered_incident
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
PHASE_F_REFERENCE_RESULT_VALUE_V1 = supports | contradicts | not_assessed | unavailable
  | RUNTIME_CANONICAL_TEXT_V1
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
`PhaseFDecisionValueV1` is exactly one of the 21 F0 value variants in §4,
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
report:PATH_V1,registry_compromised:PATH_V1|null}`. `PATH_V1` is a valid UTF-8
path string with no NUL, CR, or LF and the §7 path resolution rules; it is not
an untyped string. `PATH_V1` may be absolute only when the command argument
explicitly permits an absolute path; relative paths have no process-CWD meaning.
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

## 3. Content-derived external identity

`CONTENT_DERIVED_EXTERNAL_ID_V1` is the one rule for every content-derived
external ID. Let `semantic_payload` be the complete semantic object excluding
only its own semantic-ID field, a complete-file-only registry pointer, and a
signature field added after semantic identity. Then:

```text
semantic_id = "sha256:" + lowercase_hex(
    SHA256(DOMAIN_SEPARATOR_BYTES || JCS(semantic_payload))
)
```

Every domain separator is unique, literal ASCII, and ends with one NUL byte.
The registry pointer is never included in an ID. Runtime-owned IDs remain exact
runtime stable IDs and are never recomputed by this rule.

| Schema / ID field | Domain separator | Fields excluded | Registry pointer excluded? | Signature excluded? |
|---|---|---|---|---|
| `PhaseFDecisionBundleV1.decision_bundle_id` | `mhi_phase_f_decision_bundle_v1\0` | own ID | no | no |
| `PhaseFIndependentReviewBundleV1.review_bundle_id` | `mhi_phase_f_review_bundle_v1\0` | own ID | no | no |
| `PhaseFAuthorityEnrollmentV1.enrollment_id` | `mhi_phase_f_authority_enrollment_v1\0` | own ID | no | no |
| `PhaseFRetrievalVerificationV1.retrieval_id` | `mhi_phase_f_retrieval_v1\0` | own ID | no | no |
| `PhaseFPackageManifestV1.manifest_id` | `mhi_phase_f_package_manifest_v1\0` | own ID | no | no |
| `PhaseFDependencyAuditV1.dependency_audit_id` | `mhi_phase_f_dependency_audit_v1\0` | own ID | no | no |
| `PhaseFPhysicalUnitLedgerV1.unit_ledger_id` | `mhi_phase_f_unit_ledger_v1\0` | own ID | no | no |
| `PhaseFPhysicalIdentityAuditV1.identity_audit_id` | `mhi_phase_f_identity_audit_v1\0` | own ID | no | no |
| `PhaseFLocationLedgerV1.location_ledger_id` | `mhi_phase_f_location_ledger_v1\0` | own ID | no | no |
| `PhaseFChainOfCustodyV1.custody_ledger_id` | `mhi_phase_f_custody_ledger_v1\0` | own ID | no | no |
| `PhaseFDeviationLedgerV1.deviation_ledger_id` / `revision_id` | `mhi_phase_f_deviation_ledger_v1\0` / `mhi_phase_f_deviation_revision_v1\0` | own ID | no | no |
| `PhaseFPowerMethodInterfaceV1.power_method_interface_id` | `mhi_phase_f_power_method_interface_v1\0` | own ID | no | no |
| `PhaseFPowerAnalysisRecordV1.power_analysis_id` / `analysis_id` | `mhi_phase_f_power_analysis_v1\0` | own ID | no | no |
| `PhaseFMetrologyPolicyV1.metrology_policy_id` | `mhi_phase_f_metrology_policy_v1\0` | own ID | no | no |
| `PhaseFReferenceSourceDescriptorV1.reference_source_id` | `mhi_phase_f_reference_source_v1\0` | own ID | no | no |
| `PhaseFReferenceResultV1.reference_result_id` | `mhi_phase_f_reference_result_v1\0` | own ID | no | no |
| `PhaseFCohortLockRecordV1.cohort_lock_id` | `mhi_phase_f_cohort_lock_v1\0` | own ID | no | no |
| `PhaseFExecutionRecordV1.execution_id` | `mhi_phase_f_execution_v1\0` | own ID | no | no |
| `PhaseFReleaseRecordV1.release_record_id` | `mhi_phase_f_release_record_v1\0` | own ID and `registry_record_sha256` | yes | no |
| `PhaseFClaimStateRecordV1.claim_state_record_id` | `mhi_phase_f_claim_state_v1\0` | own ID and `registry_record_sha256` | yes | no |
| `PhaseFReinstatementApprovalV1.reinstatement_id` | `mhi_phase_f_reinstatement_v1\0` | own ID | no | no |
| `PhaseFMonitoringPolicyV1.monitoring_policy_id` | `mhi_phase_f_monitoring_policy_v1\0` | own ID | no | no |
| `PhaseFMonitoringRecordV1.monitoring_record_id` | `mhi_phase_f_monitoring_record_v1\0` | own ID and `registry_record_sha256` | yes | no |
| `PhaseFIncidentRecordV1.incident_id` | `mhi_phase_f_incident_record_v1\0` | own ID | no | no |
| `PhaseFRetentionAuditV1.retention_audit_id` | `mhi_phase_f_retention_audit_v1\0` | own ID | no | no |
| `PhaseFScientificAdmissibilityAuditV1.scientific_admissibility_audit_id` | `mhi_phase_f_scientific_admissibility_audit_v1\0` | own ID | no | no |
| `PhaseFRegistryCompromiseEmergencyV1.emergency_id` | `mhi_phase_f_registry_compromise_emergency_v1\0` | own ID | no | no |

`SEMANTIC_ID_CONSTRUCTION_AMBIGUITIES=0` and `WIRE_IDENTITY_CYCLES=0` require
that every row above be followed literally. A complete-file hash is always
computed only after all semantic fields, registry pointers, and signatures
present in that schema have been inserted.

## 4. F0 decision bundle and runtime projection

`PhaseFDecisionBundleV1` has exactly
`schema_version,decision_bundle_id,decisions`. Each decision has exactly
`decision_id:RUNTIME_STABLE_ID_V1,value:<one exact value below>,
decision_owner_role:PHASE_F_REVIEW_ROLE_V1,rationale_document_sha256:SHA256_V1`.
The 21 IDs occur once in ascending ID order. The bundle ID follows §3.

F-OD-01 through F-OD-20 retain the R4 scientific, runtime, and governance
choices; R5 only makes their construction and validation exact. F-OD-21 is
retained solely as `NON-AUTHORITATIVE OPERATOR METADATA` for operational
coordination. Its principal values are never security, scientific, approval,
tag, registry, or claim authority and are not in tag validity.

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
| 12 | `{power_analysis_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,power_method_id:RUNTIME_STABLE_ID_V1,power_method_version:RUNTIME_CANONICAL_TEXT_V1,power_method_interface:PhaseFObjectReferenceV1}` |
| 13 | `{authority_id:RUNTIME_STABLE_ID_V1,authority_role:"production_owner",authority_document:PhaseFObjectReferenceV1}` |
| 14 | `{authority_id:RUNTIME_STABLE_ID_V1,authority_role:"production_registry",registry_namespace_id:RUNTIME_STABLE_ID_V1,registry_head_resolver_uri:LIVE_REGISTRY_HEAD_URI_V1,registry_head_max_validity_seconds:DURATION_SECONDS_V1}` |
| 15 | `{custody_method_id:RUNTIME_STABLE_ID_V1,custody_procedure_document:PhaseFObjectReferenceV1,owner_custodian_role:RUNTIME_STABLE_ID_V1,registry_custodian_role:RUNTIME_STABLE_ID_V1,required_quorum:CANONICAL_POSITIVE_INTEGER_V1,key_input_channel_id:RUNTIME_STABLE_ID_V1,network_mode:"offline"|"hsm_isolated",key_persistence_allowed:false,production_cli_access_allowed:false}` |
| 16 | `{trigger_actions:[{trigger_code:ROTATION_TRIGGER_V1,required_state:PHASE_F_CLAIM_STATE_V1,revalidation_scope:"endpoint"|"full",new_approval_required:true,new_run_required:true,resolution_mode:PHASE_F_RESOLUTION_MODE_V1}],procedure_document_sha256:SHA256_V1,unsupported_lifecycle_action:"f3_no_go"}` |
| 17 | `{claim_validity_seconds:DURATION_SECONDS_V1,periodic_review_seconds:DURATION_SECONDS_V1,suspension_sla_seconds:DURATION_SECONDS_V1}` |
| 18 | `{deviation_actions:[{deviation_code:RUNTIME_STABLE_ID_V1,required_action:PHASE_F_DEVIATION_ACTION_V1}]}` total over permitted deviation codes |
| 19 | complete `PhaseFMonitoringPolicyV1` from §14 |
| 20 | `{allowed_immutable_uri_schemes:[URI_SCHEME_V1],retention_seconds:DURATION_SECONDS_V1,backup_copy_count:CANONICAL_POSITIVE_INTEGER_V1,backup_verification_interval_seconds:DURATION_SECONDS_V1,authorized_access_role_ids:[RUNTIME_STABLE_ID_V1],replacement_authority_role_id:RUNTIME_STABLE_ID_V1,unavailable_object_action:"no_go"}` |
| 21 | `{operator_metadata:RUNTIME_CANONICAL_TEXT_V1}` only; excluded from all approval/security validity |

`ROTATION_TRIGGER_V1` is the closed enum
`key_rotation|key_compromise|key_revocation|method_version_change|protocol_revision|
domain_expansion|code_change|sensor_design_change|report_withdrawal|superseding_campaign`.
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
independent file bytes. Aggregate GO requires five GO, P0 zero, and P1 zero.

### 5.1 Enrollment is intentionally unsigned

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
name/email, GitHub push actor, Git commit author, and F-OD-21 metadata are not
used for validity. A tag is valid exactly when its name, annotated type, peeled
target, exact body grammar, preceding references, review-bundle hash, GO/P0/P1
values, and referenced objects verify.

Every body is printable ASCII plus one final LF, has the fixed first schema line,
one `name=value` line per listed field in listed order, no blank/duplicate/
unknown/trailing-whitespace line, and no LF or `=` in a value. Every approval
body has `review_bundle_sha256=<SHA256_V1>`.

| Tag / body schema | Target | Required ordered fields after `format_version=1` |
|---|---|---|
| `ism-mechanism-health-v1-f-plan-approved` / `PhaseFPlanApprovalV1` | reviewed R5 main | `plan_review_sha,plan_sha256,plan_git_blob,review_bundle_sha256,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-f0-decisions-approved` / `PhaseFDecisionApprovalV1` | reviewed F0 main | `phase_f_plan_tag,plan_review_sha,decision_review_sha,review_bundle_sha256,decision_bundle_id,decision_file_sha256,decision_git_blob,decision_count,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-readiness-approved` / `PhaseFReadinessApprovalV1` | integrated F-IMPL-1 | `phase_f_plan_tag,f0_decisions_tag,readiness_review_sha,checker_source_review_sha,checker_source_tree,checker_dependency_lock_sha256,checker_binary_sha256,review_bundle_sha256,macos_uname,macos_arch,macos_product_version,macos_build_version,rustc_version,cargo_version,build1,build2,reproducible_binary,f_maint_01,f_maint_02,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-authority-enrollment-approved` / `PhaseFAuthorityEnrollmentApprovalV1` | readiness main | `phase_f_plan_tag,f0_decisions_tag,readiness_tag,readiness_main_sha,enrollment_sha256,owner_authority_id,registry_authority_id,owner_public_key_fingerprint,registry_public_key_fingerprint,review_bundle_sha256,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-trust-provisioning-approved` / `PhaseFTrustProvisioningApprovalV1` | integrated F3 main | `phase_f_plan_tag,f0_decisions_tag,readiness_tag,authority_enrollment_tag,enrollment_sha256,owner_public_key_fingerprint,registry_public_key_fingerprint,trust_root_id,trust_review_sha,trust_store_git_blob,trust_store_sha256,f2_cohort_lock_registry_record_sha256,review_bundle_sha256,macos_uname,macos_arch,macos_product_version,macos_build_version,macos_result,security_decision,compatibility_decision,p0_count,p1_count,approval_decision` |
| `ism-mechanism-health-v1-f-physical-validation-released` / `PhaseFPhysicalReleaseApprovalV1` | final F4/F5 main | `phase_f_plan_tag,f0_decisions_tag,readiness_tag,authority_enrollment_tag,trust_provisioning_tag,release_code_sha,protocol_sha256,cohort_lock_registry_record_sha256,owner_approval_record_id,owner_approval_file_sha256,validation_manifest_sha256,release_record_id,release_file_sha256,release_registry_record_sha256,initial_claim_state_record_id,initial_claim_state_file_sha256,initial_claim_state_registry_record_sha256,review_bundle_sha256,scientific_decision,architecture_decision,security_decision,compatibility_decision,operations_decision,p0_count,p1_count,macos_result,release_decision` |

All SHA/Git/count/ID values use the named types in §§2-3. Four review decision
fields are exactly GO, P0/P1 are exactly `0`, and each tag is absent before
creation, never moved, and pushed only after its target is live. No Phase-F tag
is created during R5.

## 7. Checker build and command authority

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

The only normative command forms are:

```text
phase-f-authority-check verify --kind <OBJECT_KIND> --input <PATH> \
  --context-dir <PATH> --report <PATH>
phase-f-authority-check claim-status --release <PATH> --context-dir <PATH> \
  --registry-head-uri <LIVE_REGISTRY_HEAD_URI_V1> --now <UTC_SECOND_TIMESTAMP_V1> \
  --report <PATH> [--registry-compromised <PATH>]
```

`--kind` is exactly one `PHASE_F_OBJECT_KIND_V1`; `verify` takes exactly one
input, one context directory, and one report path. `claim-status` takes exactly
one release, context directory, live URI, UTC timestamp, and report path. The
optional emergency path is permitted only for §15's exact emergency schema.
Paths are UTF-8, relative paths resolve only from the named context directory,
absolute paths are permitted only when explicitly passed, and symlinks,
directories in file position, traversal, and unsafe files reject. The checker
writes only the requested report.

Stdout is exactly one line: `PASS\n` or `NO-GO\n` for `verify`, and
`ACTIVE\n`, `NOT_ACTIVE\n`, or `AUTHORITY_UNAVAILABLE\n` for `claim-status`.
Stderr is empty on success; on a validated failure it is sorted lines of
`diagnostic_code=<DIAGNOSTIC_CODE_V1>\n`; usage errors have only
`usage_error=<USAGE_CODE_V1>\n`. Exit codes are exact: `0` PASS/ACTIVE, `1`
validated NO-GO/NOT_ACTIVE, `2` AUTHORITY_UNAVAILABLE for claim-status only,
`64` CLI usage error, `65` malformed structured input, `66` required input
unavailable, and `70` internal checker failure. No other code exists.

`PhaseFCheckerReportV1` is exactly
`schema_version,checker_binary_sha256,command,input_sha256s,decision,
diagnostic_codes`. `command:PhaseFCommandV1` is the canonical ordered argv
array for one of the two forms. `input_sha256s` is a sorted array of
`{input_name:RUNTIME_CANONICAL_TEXT_V1,sha256:SHA256_V1}`. The decision uses
`PHASE_F_CHECKER_DECISION_V1`; diagnostic codes are closed:
`MalformedJson,UnknownMember,DuplicateMember,InvalidType,InvalidSemanticId,
InvalidCompleteFileHash,InvalidSignature,InvalidRelation,InvalidTransition,
MissingInput,UnsafePath,ResolverUnavailable,HeadExpired,RegistryEquivocation,
RegistryRegression,MonitoringBreach,RetentionFailure,ProjectionMismatch,
BuildInputMismatch,CommandResultMismatch`. Report decision, stdout, and exit
code must agree.

## 8. Live registry resolver and cryptographic wire

`registry_head_resolver_uri` is `LIVE_REGISTRY_HEAD_URI_V1`, never
`IMMUTABLE_EXTERNAL_URI_V1`. Its exact bytes are F0-bound, but no response hash
is bound to that URI. HTTPS is transport only. Authority is the verified
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
Same sequence/digest is the same head; same sequence/different digest is
equivocation and NOT_ACTIVE; lower than a verified watermark is regression and
AUTHORITY_UNAVAILABLE; higher requires every intervening record. Unavailable
resolver, expired head, bad signature, missing chain object, or equivocation
never uses a cache as ACTIVE authority.

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

`PhaseFRegistryRelationV1` is exactly
`{relation_type:PHASE_F_RELATION_TYPE_V1,object_kind:PHASE_F_OBJECT_KIND_V1,
object_sha256:SHA256_V1}`. Relation type is
`authorized_by|depends_on|registered_after|locks|approves|executes|releases|
changes_state_of|supersedes|references|incident_recorded|retention_audited|
scientific_admissibility`. Every relation is validated against kind and hash
meaning; a bare hash never supplies a subject.

| Record kind | Subject ID / hash | Required relations | Optional relations; all others forbidden |
|---|---|---|---|
| `authority_enrolled` | `enrollment_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `authority_enrollment` | `authorized_by+decision_bundle`; `references+git_tag_message` for plan/F0/readiness/enrollment tags | none |
| `protocol_registered` | `protocol_id:RUNTIME_STABLE_ID_V1` / `protocol` | `authorized_by+decision_bundle`; `depends_on+registration_document` | none |
| `power_registered` | `power_analysis_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `power_analysis` | `authorized_by+decision_bundle`; `depends_on+power_method_interface`; `depends_on+protocol` | none |
| `package_registered` | `manifest_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `package_manifest` | one dependency audit, unit ledger, identity audit, location ledger, custody, deviation, metrology policy; at least one reference result and source descriptor | one `scientific_admissibility+scientific_admissibility_audit` |
| `cohort_locked` | `cohort_lock_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `cohort_lock` | `locks+package_manifest`; `depends_on+protocol`; `depends_on+power_analysis`; `depends_on+deviation_ledger` | none |
| `owner_approval_registered` | `approval_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `owner_approval` | `approves+cohort_lock`; `authorized_by+authority_enrollment` | none |
| `execution_registered` | `execution_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `execution_record` | `executes+cohort_lock`; `authorized_by+owner_approval`; `depends_on+deviation_ledger`; `depends_on+protocol` | none |
| `release_registered` | `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `release_record` | `releases+execution_record`; `authorized_by+owner_approval`; `depends_on+monitoring_policy`; `depends_on+metrology_policy` | none |
| `claim_state_changed` | `claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `claim_state` | `changes_state_of+release_record` | one prior-state `registered_after+claim_state`; one reinstatement dependency; one superseding-release relation only when applicable |
| `monitoring_recorded` | `monitoring_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `monitoring_record` | `references+release_record`; `depends_on+monitoring_policy` | one prior `registered_after+monitoring_record` after first |
| `incident_recorded` | `incident_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `incident_record` | `incident_recorded+release_record` | references only to listed affected evidence |
| `retention_audit_recorded` | `retention_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` / `retention_audit` | `retention_audited+release_record` | one `references+package_manifest` per checked object |

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

`PhaseFDeviationLedgerV1` has exactly
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
revision object; `deviation_ledger_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` identifies
the append-only campaign ledger. Both are derived by the separate domain rows
in §3 and neither is inferred from `revision_number`.

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
all use `value_type=positive_integer`, unit none, and range positive. No
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
overrides and validates again. Scientific reviewer decides adequacy.
`analysis_id` is not a second wire field: wherever that legacy name occurs it
means `power_analysis_id:PHASE_F_EXTERNAL_DIGEST_ID_V1` exactly.

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
`{check_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,check_id:RUNTIME_STABLE_ID_V1,
reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1,performed_at:UTC_SECOND_TIMESTAMP_V1,
method_id:RUNTIME_STABLE_ID_V1,method_version:RUNTIME_CANONICAL_TEXT_V1,
authority_id:RUNTIME_STABLE_ID_V1,measurand_id:RUNTIME_STABLE_ID_V1,
value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1,result:PHASE_F_CHECK_RESULT_V1}`. Checker recomputes
the comparator; manually inconsistent result rejects. Every required calibration
and QC result must pass.

`PhaseFLODLOQPolicyV1` is `{type:"not_applicable"}` or
`{type:"required",lod_value:RUNTIME_F64_V1,lod_unit:UNIT_TEXT_V1,
loq_value:RUNTIME_F64_V1,loq_unit:UNIT_TEXT_V1,below_lod_action:PHASE_F_DEVIATION_ACTION_V1,
between_lod_loq_action:PHASE_F_DEVIATION_ACTION_V1}`. Units match unless the interface defines
conversion before lock; `lod_value<=loq_value`. No untyped LOD/LOQ exists.

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
result_value:PHASE_F_REFERENCE_RESULT_VALUE_V1,result_unit,
limitations,limitations_document_sha256,traceability_document_sha256,
chain_of_custody_sha256`. Types are schema integer, external ID, runtime ID,
runtime stable ID, external source ID, SHA-256, tagged type, runtime ID,
canonical text, runtime ID, exact `BlindingStateV1`, exact quantified
`{type:"quantified",measure_id:RUNTIME_STABLE_ID_V1,value:RUNTIME_F64_V1,
unit:UNIT_TEXT_V1}`, tagged result value, `UNIT_TEXT_V1`, sorted unique
`[RUNTIME_CANONICAL_TEXT_V1]`, and three SHA-256 values. Mechanism adds
`hypothesis_id:RUNTIME_STABLE_ID_V1,outcome:supports|contradicts|not_assessed|unavailable`;
health adds `target:HealthTargetV1,label:RUNTIME_CANONICAL_TEXT_V1`.
`reference_endpoint_id` is never aliased to `reference_result_id`.

Projection is total: endpoint ID, reference endpoint ID, source ID, mechanism
hypothesis/outcome or health target/label, method ID/version, authority ID,
blinding state, quantified uncertainty fields, and limitations copy field-for-
field into `ReferenceEndpointV1`. `result_value` and `result_unit` are the
validated external measurement pair: for the runtime endpoint its tagged value
is the exact outcome/label field and its unit is the policy unit; neither is
silently dropped or inferred. No result can produce an endpoint with a missing
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
metrology_policy_sha256,valid_from,valid_until,limitations,registry_record_sha256`.
Claim ID is runtime stable; statement is canonical text; hashes are SHA-256;
times are UTC with `valid_from<valid_until`; limitations sort unique; registry
pointer is `SHA256_V1|null` only during construction. ID excludes own ID and
pointer; registry subject uses semantic ID, then pointer and complete-file hash
are added. There is no identity cycle.

`PhaseFClaimStateRecordV1` is exactly
`schema_version,claim_state_record_id,claim_id,release_record_id,
previous_claim_state_record_id,state,reason_code,effective_at,
superseding_release_record_id,reinstatement_approval_sha256,limitations,
registry_record_sha256`. Only previous, superseding, reinstatement hash, and
registry pointer are nullable. State is
`PHASE_F_CLAIM_STATE_V1`; `reason_code:PHASE_F_CLAIM_REASON_V1` is the exact table:

| Reason | Legal prior → next | Extra authority |
|---|---|---|
| `initial_release` | none → active | release and final approval |
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

`PhaseFReinstatementApprovalV1` is exactly
`schema_version,reinstatement_id,claim_id,suspended_state_record_id,
suspension_reason,required_corrective_action,corrective_evidence_sha256s,
execution_record_sha256,review_bundle_sha256,scientific_decision,
architecture_decision,security_decision,compatibility_decision,
operations_decision,p0_count,p1_count,approval_decision`. It requires exactly
five independent GO decisions, P0/P1 zero, and the exact allowed trigger row.

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
value:RUNTIME_F64_V1,unit:UNIT_TEXT_V1|null}`; all actions are `suspend`.

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
window_end,policy_sha256,measurements,breaches,result,registry_record_sha256`.
Every required metric appears once in policy order. Measurement is
`{metric_id:PHASE_F_MONITORING_METRIC_V1,value:PhaseFMonitoringValueV1}`; status uses the
metric vocabulary, numeric uses `RUNTIME_F64_V1`, and bindings use named ID/hash
types. Breach is
`{metric_id:PHASE_F_MONITORING_METRIC_V1,breach_code:PHASE_F_BREACH_CODE_V1,
evidence_sha256:SHA256_V1}` sorted by metric ID. Result is derived: pass iff
all required metrics exist once, types are correct, statuses healthy, bindings
equal release authority, thresholds pass, and evidence exists; otherwise
suspend. Declared result must equal recomputation. At `now>=due`, claim is
NOT_ACTIVE and a suspension transition is required.

An accepted monitoring window is exactly a structurally valid, recomputed-pass,
registry-bound, current-chain record for the correct release and policy. Only
its `window_end` anchors the next due time. First `window_start` is initial
ACTIVE `effective_at`; each later PASS starts at prior accepted `window_end`;
there is no overlap or gap greater than the interval. A suspend record is never
accepted.

## 15. Retention, incidents, and compromise

`PhaseFIncidentRecordV1` is exactly
`schema_version,incident_id,claim_id,release_record_id,incident_type,detected_at,
affected_object_sha256s,affected_unit_ids,evidence_references,required_action,
incident_status`. Affected object entries are sorted
`PhaseFObjectDigestV1`; unit IDs are
sorted runtime IDs; evidence references are sorted `PhaseFObjectReferenceV1`;
type/action/status use §2 enums. `other_registered_incident` requires an
immutable incident-type definition document. ID and complete-file hash follow
§3/§9.

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
explicit campaign-abandonment incident plus policy duration. Deletion is never
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
affected_claim_ids,incident_record_sha256,security_operations_review_bundle_sha256,
repository_commit_sha,repository_blob_sha,action`, where action is the
literal `suspend_all_active_claims`. IDs/hashes use named types and ID follows
§3. Repository commit/blob and a review bundle with GO `security` and
`operations_governance` rows are independent authority; registry key is not
used. `--registry-compromised` accepts only this object, exact commit/blob, and
review bundle and immediately returns NOT_ACTIVE for affected claims. No
unsigned flag or revoked-root bypass exists.

Owner-key compromise uses the un-compromised registry/governance path to append
the exact suspend/withdraw state. Recovery requires new root, approval, run,
and release when F0 says so; compromised key never revokes itself.

## 16. Master schema catalog

This is the one catalog for every external R5 schema. Each row includes field
closure, identity, producer, validator, stage, requirement, AC, test, evidence,
and registry relation.

| Schema | Field closure / identity | Producer; validator; stage | Registry relation | Requirement / AC / test / evidence |
|---|---|---|---|---|
| `PhaseFDecisionBundleV1` | §4; §3 ID; complete hash; unsigned | F0; checker; F0 | protocol authority | R5-01 / AC5-01 / T5-01 / EV5-01 |
| `PhaseFIndependentReviewBundleV1` | §5 exact five rows; §3 ID; complete hash | independent roles; checker; approvals | tag evidence | R5-02 / AC5-02 / T5-02 / EV5-02 |
| `PhaseFProtocolProjectionV1` | §4 exact plan contract; no wire ID | checker; projection; F1 | protocol | R5-03 / AC5-03 / T5-03 / EV5-03 |
| `PhaseFAuthorityEnrollmentV1` | §5.1 unsigned; §3 ID/file hash | governance; enrollment; readiness | authority_enrolled | R5-04 / AC5-04 / T5-04 / EV5-04 |
| `PhaseFCheckerReportV1` | §7 exact fields; canonical hash | checker; report validator; all | none | R5-05 / AC5-05 / T5-05 / EV5-05 |
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
| `PhaseFDeviationLedgerV1` | §11 append-only events; §3 ID/hash | campaign; deviation; F2-F4 | package/execution | R5-17 / AC5-17 / T5-17 / EV5-17 |
| `PhaseFPowerMethodInterfaceV1` | §12 unit/range/output rows; §3 ID/hash | statistician; power; F1 | power dependency | R5-18 / AC5-18 / T5-18 / EV5-18 |
| `PhaseFPowerAnalysisRecordV1` | §12 params/cases; §3 ID/hash | statistician; power; F1 | power subject | R5-19 / AC5-19 / T5-19 / EV5-19 |
| `PhaseFMetrologyPolicyV1` | §13 Cartesian methods/checks; §3 ID/hash | metrology; policy; F0/F2 | package/release | R5-20 / AC5-20 / T5-20 / EV5-20 |
| `PhaseFMetrologyCheckSpecV1` | §13 exact fields | metrology; policy; F2 | nested policy | R5-21 / AC5-21 / T5-21 / EV5-21 |
| `PhaseFMetrologyCheckResultV1` | §13 exact fields/math; complete hash | laboratory; result; F2 | package evidence | R5-22 / AC5-22 / T5-22 / EV5-22 |
| `PhaseFReferenceSourceDescriptorV1` | §13 runtime types; §3 ID/hash | laboratory/data; source; F2 | package dependency | R5-23 / AC5-23 / T5-23 / EV5-23 |
| `PhaseFReferenceResultV1` | §13 total projection; §3 ID/hash | laboratory; reference; F2 | package dependency | R5-24 / AC5-24 / T5-24 / EV5-24 |
| `PhaseFScientificAdmissibilityAuditV1` | exact fields below; §3 ID/hash | scientific reviewer/checker; F2 | scientific_admissibility | R5-25 / AC5-25 / T5-25 / EV5-25 |
| `PhaseFCohortLockRecordV1` | §14 exact hashes; §3 ID/hash | campaign; cohort; F2 | cohort_locked | R5-26 / AC5-26 / T5-26 / EV5-26 |
| `PhaseFExecutionRecordV1` | §14 exact time/result; §3 ID/hash | release; execution; F4 | execution_registered | R5-27 / AC5-27 / T5-27 / EV5-27 |
| `PhaseFReleaseRecordV1` | §14 pointer construction; §3 ID/hash | release; release; F5 | release_registered | R5-28 / AC5-28 / T5-28 / EV5-28 |
| `PhaseFClaimStateRecordV1` | §14 transition; §3 ID/hash | governance; state; F5+ | claim_state_changed | R5-29 / AC5-29 / T5-29 / EV5-29 |
| `PhaseFReinstatementApprovalV1` | §14 five-role review; §3 ID/hash | reviewers; reinstatement; F5+ | state dependency | R5-30 / AC5-30 / T5-30 / EV5-30 |
| `PhaseFMonitoringPolicyV1` | §14 metric vocabulary; §3 ID/hash | F0; monitoring; F5+ | release dependency | R5-31 / AC5-31 / T5-31 / EV5-31 |
| `PhaseFMonitoringRecordV1` | §14 derived result/window; §3 ID/hash | operations; monitoring; F5+ | monitoring_recorded | R5-32 / AC5-32 / T5-32 / EV5-32 |
| `PhaseFIncidentRecordV1` | §15 exact fields/enums; §3 ID/hash | operations/governance; incident; all | incident_recorded | R5-33 / AC5-33 / T5-33 / EV5-33 |
| `PhaseFRetentionAuditV1` | §15 exact checks/result; §3 ID/hash | operations; retention; all | retention_audited | R5-34 / AC5-34 / T5-34 / EV5-34 |
| `PhaseFRegistryCompromiseEmergencyV1` | §15 independent path; §3 ID/hash | security/operations; emergency; claim-status | emergency input | R5-35 / AC5-35 / T5-35 / EV5-35 |
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
| `PhaseFCheckerReportV1` | `schema_version:JSON_INTEGER_ONE`; `checker_binary_sha256:SHA256_V1`; `command:PhaseFCommandV1`; `input_sha256s:SORTED_UNIQUE<PhaseFNamedDigestV1>`; `decision:PHASE_F_CHECKER_DECISION_V1`; `diagnostic_codes:SORTED_UNIQUE<DIAGNOSTIC_CODE_V1>` |
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
| `PhaseFDeviationLedgerV1` | `schema_version:JSON_INTEGER_ONE`; `deviation_ledger_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `revision_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `campaign_id:RUNTIME_STABLE_ID_V1`; `revision_number:CANONICAL_UNSIGNED_INTEGER_V1`; `previous_revision_sha256:SHA256_V1|null`; `events:SORTED_UNIQUE<PhaseFDeviationEventV1>` |
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
| `PhaseFMetrologyCheckResultV1` | `check_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `check_id:RUNTIME_STABLE_ID_V1`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `performed_at:UTC_SECOND_TIMESTAMP_V1`; `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `measurand_id:RUNTIME_STABLE_ID_V1`; `value:RUNTIME_F64_V1`; `unit:UNIT_TEXT_V1`; `result:PHASE_F_CHECK_RESULT_V1` |
| `PhaseFReferenceSourceDescriptorV1` | `schema_version:JSON_INTEGER_ONE`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `source_file_sha256:SHA256_V1`; `evidence_origin:EvidenceOriginV1`; `dependency_completeness:ReferenceDependencyCompletenessV1`; `experiment_scope:ArtifactExperimentScope`; `acquisition_families:ArtifactAcquisitionFamilies`; `direct_dependencies:SORTED_UNIQUE<ReferenceDependencyV1>` |
| `PhaseFReferenceResultV1` | `schema_version:JSON_INTEGER_ONE`; `reference_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_id:RUNTIME_STABLE_ID_V1`; `reference_endpoint_id:RUNTIME_STABLE_ID_V1`; `reference_source_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `reference_source_descriptor_sha256:SHA256_V1`; `reference_type:mechanism|health`; `method_id:RUNTIME_STABLE_ID_V1`; `method_version:RUNTIME_CANONICAL_TEXT_V1`; `authority_id:RUNTIME_STABLE_ID_V1`; `blinding_state:BlindingStateV1`; `uncertainty:PhaseFQuantifiedUncertaintyV1`; `result_value:PHASE_F_REFERENCE_RESULT_VALUE_V1`; `result_unit:UNIT_TEXT_V1`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>`; `limitations_document_sha256:SHA256_V1`; `traceability_document_sha256:SHA256_V1`; `chain_of_custody_sha256:SHA256_V1`; mechanism branch adds `hypothesis_id:RUNTIME_STABLE_ID_V1,outcome:supports|contradicts|not_assessed|unavailable`; health branch adds `target:HealthTargetV1,label:RUNTIME_CANONICAL_TEXT_V1` |
| `PhaseFScientificAdmissibilityAuditV1` | `schema_version:JSON_INTEGER_ONE`; `scientific_admissibility_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `protocol_sha256:SHA256_V1`; `package_manifest_sha256:SHA256_V1`; `dependency_audit_sha256:SHA256_V1`; `identity_audit_sha256:SHA256_V1`; `reference_assessments:NONEMPTY_SORTED_UNIQUE<PhaseFReferenceAssessmentV1>`; `reviewer_role:scientific_metrology`; `result:PHASE_F_RESULT_V1` |
| `PhaseFReferenceAssessmentV1` | `reference_result_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `endpoint_id:RUNTIME_STABLE_ID_V1`; `evidence_category:SCIENTIFIC_EVIDENCE_CATEGORY_V1`; `claim_ceiling:SCIENTIFIC_CLAIM_CEILING_V1`; `dependency_status:known_separated|known_overlap|unknown`; `identity_status:distinct|same|unknown`; `admissibility:physical_support_allowed|limited_only|not_assessed|unavailable|not_admissible` |
| `PhaseFCohortLockRecordV1` | `schema_version:JSON_INTEGER_ONE`; `cohort_lock_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `protocol_sha256:SHA256_V1`; `package_manifest_sha256:SHA256_V1`; `power_analysis_sha256:SHA256_V1`; `dependency_audit_sha256:SHA256_V1`; `physical_unit_ledger_sha256:SHA256_V1`; `identity_audit_sha256:SHA256_V1`; `location_ledger_sha256:SHA256_V1`; `chain_of_custody_sha256:SHA256_V1`; `deviation_ledger_sha256:SHA256_V1`; `metrology_policy_sha256:SHA256_V1`; `scientific_admissibility_audit_sha256:SHA256_V1`; `reference_result_sha256s:SORTED_UNIQUE<SHA256_V1>`; `reference_source_descriptor_sha256s:SORTED_UNIQUE<SHA256_V1>`; `locked_at:UTC_SECOND_TIMESTAMP_V1` |
| `PhaseFExecutionRecordV1` | `schema_version:JSON_INTEGER_ONE`; `execution_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `cohort_lock_record_sha256:SHA256_V1`; `owner_approval_file_sha256:SHA256_V1`; `protocol_sha256:SHA256_V1`; `deviation_ledger_sha256:SHA256_V1`; `release_code_sha:GIT_SHA_V1`; `checker_binary_sha256:SHA256_V1`; `validation_manifest_sha256:SHA256_V1`; `started_at:UTC_SECOND_TIMESTAMP_V1`; `completed_at:UTC_SECOND_TIMESTAMP_V1`; `result:PHASE_F_RESULT_V1` |
| `PhaseFReleaseRecordV1` | `schema_version:JSON_INTEGER_ONE`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `claim_statement:RUNTIME_CANONICAL_TEXT_V1`; `release_code_sha:GIT_SHA_V1`; `protocol_sha256:SHA256_V1`; `cohort_lock_record_sha256:SHA256_V1`; `owner_approval_file_sha256:SHA256_V1`; `execution_record_sha256:SHA256_V1`; `validation_manifest_sha256:SHA256_V1`; `monitoring_policy_sha256:SHA256_V1`; `metrology_policy_sha256:SHA256_V1`; `valid_from:UTC_SECOND_TIMESTAMP_V1`; `valid_until:UTC_SECOND_TIMESTAMP_V1`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>`; `registry_record_sha256:SHA256_V1|null` |
| `PhaseFClaimStateRecordV1` | `schema_version:JSON_INTEGER_ONE`; `claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `previous_claim_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1|null`; `state:PHASE_F_CLAIM_STATE_V1`; `reason_code:PHASE_F_CLAIM_REASON_V1`; `effective_at:UTC_SECOND_TIMESTAMP_V1`; `superseding_release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1|null`; `reinstatement_approval_sha256:SHA256_V1|null`; `limitations:SORTED_UNIQUE<RUNTIME_CANONICAL_TEXT_V1>`; `registry_record_sha256:SHA256_V1|null` |
| `PhaseFReinstatementApprovalV1` | `schema_version:JSON_INTEGER_ONE`; `reinstatement_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `suspended_state_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `suspension_reason:PHASE_F_CLAIM_REASON_V1`; `required_corrective_action:RUNTIME_CANONICAL_TEXT_V1`; `corrective_evidence_sha256s:SORTED_UNIQUE<SHA256_V1>`; `execution_record_sha256:SHA256_V1`; `review_bundle_sha256:SHA256_V1`; `scientific_decision:PHASE_F_DECISION_V1`; `architecture_decision:PHASE_F_DECISION_V1`; `security_decision:PHASE_F_DECISION_V1`; `compatibility_decision:PHASE_F_DECISION_V1`; `operations_decision:PHASE_F_DECISION_V1`; `p0_count:CANONICAL_UNSIGNED_INTEGER_V1`; `p1_count:CANONICAL_UNSIGNED_INTEGER_V1`; `approval_decision:PHASE_F_DECISION_V1` |
| `PhaseFMonitoringPolicyV1` | `schema_version:JSON_INTEGER_ONE`; `monitoring_policy_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `monitoring_interval_seconds:DURATION_SECONDS_V1`; `required_metrics:FIXED_ORDER<PHASE_F_MONITORING_METRIC_V1>`; `metric_thresholds:SORTED_UNIQUE<PhaseFMetricThresholdV1>`; `missing_monitoring_action:suspend`; `domain_breach_action:suspend`; `reference_qc_breach_action:suspend` |
| `PhaseFMetricThresholdV1` | `metric_id:PHASE_F_MONITORING_NUMERIC_METRIC_V1`; `comparator:greater_than_or_equal|less_than_or_equal`; `value:RUNTIME_F64_V1`; `unit:UNIT_TEXT_V1|null` |
| `PhaseFMonitoringRecordV1` | `schema_version:JSON_INTEGER_ONE`; `monitoring_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `window_start:UTC_SECOND_TIMESTAMP_V1`; `window_end:UTC_SECOND_TIMESTAMP_V1`; `policy_sha256:SHA256_V1`; `measurements:NONEMPTY_SORTED_UNIQUE<PhaseFMonitoringMeasurementV1>`; `breaches:SORTED_UNIQUE<PhaseFMonitoringBreachV1>`; `result:PHASE_F_MONITORING_RESULT_V1`; `registry_record_sha256:SHA256_V1|null` |
| `PhaseFMonitoringMeasurementV1` | `metric_id:PHASE_F_MONITORING_METRIC_V1`; `value:PhaseFMonitoringValueV1` |
| `PhaseFMonitoringBreachV1` | `metric_id:PHASE_F_MONITORING_METRIC_V1`; `breach_code:PHASE_F_BREACH_CODE_V1`; `evidence_sha256:SHA256_V1` |
| `PhaseFIncidentRecordV1` | `schema_version:JSON_INTEGER_ONE`; `incident_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `claim_id:RUNTIME_STABLE_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `incident_type:PHASE_F_INCIDENT_TYPE_V1`; `detected_at:UTC_SECOND_TIMESTAMP_V1`; `affected_object_sha256s:SORTED_UNIQUE<PhaseFObjectDigestV1>`; `affected_unit_ids:SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `evidence_references:SORTED_UNIQUE<PhaseFObjectReferenceV1>`; `required_action:PHASE_F_INCIDENT_ACTION_V1`; `incident_status:PHASE_F_INCIDENT_STATUS_V1` |
| `PhaseFRetentionAuditV1` | `schema_version:JSON_INTEGER_ONE`; `retention_audit_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `release_record_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `policy_sha256:SHA256_V1`; `audited_at:UTC_SECOND_TIMESTAMP_V1`; `object_checks:NONEMPTY_SORTED_UNIQUE<PhaseFRetentionObjectCheckV1>`; `result:PHASE_F_RESULT_V1` |
| `PhaseFRetentionObjectCheckV1` | `object_sha256:PhaseFObjectDigestV1`; `primary_available:BOOLEAN_V1`; `primary_verified:BOOLEAN_V1`; `verified_backup_count:CANONICAL_UNSIGNED_INTEGER_V1`; `last_backup_verification_at:UTC_SECOND_TIMESTAMP_V1`; `result:PHASE_F_RESULT_V1` |
| `PhaseFRegistryCompromiseEmergencyV1` | `schema_version:JSON_INTEGER_ONE`; `emergency_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `registry_namespace_id:RUNTIME_STABLE_ID_V1`; `incident_id:PHASE_F_EXTERNAL_DIGEST_ID_V1`; `declared_at:UTC_SECOND_TIMESTAMP_V1`; `affected_claim_ids:NONEMPTY_SORTED_UNIQUE<RUNTIME_STABLE_ID_V1>`; `incident_record_sha256:SHA256_V1`; `security_operations_review_bundle_sha256:SHA256_V1`; `repository_commit_sha:GIT_SHA_V1`; `repository_blob:GIT_BLOB_V1`; `action:suspend_all_active_claims` |

The aliases used by this audit are defined exactly in §§2, 3, 5, 7, 9-15;
there is no free-form `string`, `integer`, `hash`, `value`, `status`, `policy`,
`record`, `document`, `object`, `relation`, or `role` field. The audit result is
`UNTYPED_NORMATIVE_FIELDS=0`.

## 17. Valid object construction order

| Object | Inputs and canonicalization | ID / signature | Complete-file hash; relation; next |
|---|---|---|---|
| F0 decision bundle | F0 values, JCS, exact 21 IDs | §3; unsigned | hash; review/tag; protocol |
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
| release record | execution/policies/interval/null pointer | §3 release ID | pointer then hash; release_registered; state |
| initial active state | release/no prior/initial_release | §3 claim-state ID | pointer then hash; state; monitoring |
| monitoring pass | correct window/all metrics/recomputed pass | §3 monitoring ID | pointer then hash; monitoring_recorded |
| incident | verified evidence/exact consequence | §3 incident ID | hash; incident_recorded; suspension |
| suspension | incident/monitoring evidence/legal transition | §3 state ID | pointer then hash; state; remediation |
| reinstatement | permitted trigger/five GO/corrective evidence | §3 reinstatement ID | hash; state dependency; active state |
| retention audit | object hashes/primary-backup verification/policy | §3 audit ID | hash; retention_audited; next audit |

No step signs enrollment, trusts a tag pusher, binds a live response to URI
hash, or makes a claim ACTIVE without current head, chain, release, state,
accepted monitoring window, retention audit, and final tag.

## 18. Historical regression repair and traceability substance

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

Each R5 requirement has a primary AC proving its complete positive path. There
are 36 requirements, 36 primary ACs, 36 tests, 36 evidence records, 21 F0
owner decisions, and no unmapped schema. Every test/evidence item is an
executable KAT/transcript with exact inputs and deterministic oracle.

## 19. Cumulative normative counterexamples

Every historical case remains independently replayable. `NO-GO` means checker
failure; public claim is NOT_ACTIVE except exact ACTIVE or
AUTHORITY_UNAVAILABLE. R5 cases are appended, not substituted.

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

## 20. R5 remediation ledger

Only a fresh independent R5 reviewer may close a finding. Author dispositions are
limited to `REMEDIATED` or `OPEN`; no author row uses `CLOSED`.

| R4 finding | R5 section | Remediation | Requirement / AC / test / evidence | Author disposition |
|---|---|---|---|---|
| F-PLAN-R5-P1-01 primitive/field typing and semantic-ID construction incomplete | §2-4 | terminal primitive registry, exact nested fields, one ID rule/table | R5-01/R5-03; AC5-01/03; T5-01/03; EV5-01/03 | REMEDIATED |
| F-PLAN-R5-P1-02 enrollment/registry/head cryptographic wire incomplete | §5.1, §8-9 | unsigned enrollment, exact strict signatures, exact signing bytes | R5-04/R5-07/R5-08; AC5-04/07/08; T5-04/07/08; EV5-04/07/08 | REMEDIATED |
| F-PLAN-R5-P1-03 durable-tag creator identity not verifiable | §6 | operator non-authority and review-bundle hash evidence | R5-02/R5-36; AC5-02/36; T5-02/36; EV5-02/36 | REMEDIATED |
| F-PLAN-R5-P1-04 checker build/command authority incomplete | §7 | clean env, two builds, exact commands/report/exit codes | R5-05; AC5-05; T5-05; EV5-05 | REMEDIATED |
| F-PLAN-R5-P1-05 live resolver conflicts with immutable URI | §4, §8 | `LIVE_REGISTRY_HEAD_URI_V1` only for live head | R5-08; AC5-08; T5-08; EV5-08 | REMEDIATED |
| F-PLAN-R5-P1-06 registry relations/subjects/incident/retention incomplete | §9, §15 | exhaustive kinds, hash meanings, typed tuples, two exact schemas | R5-06/R5-33/R5-34; AC5-06/33/34; T5-06/33/34; EV5-06/33/34 | REMEDIATED |
| F-PLAN-R5-P1-07 package invariants/scientific admissibility incomplete | §10, §16 | role matrix and concrete audit with checker rules | R5-11/R5-25; AC5-11/25; T5-11/25; EV5-11/25 | REMEDIATED |
| F-PLAN-R5-P1-08 physical unit lacks native identity value | §11 | issuer, native identifier, basis, document hash, uniqueness key | R5-13/R5-14; AC5-13/14; T5-13/14; EV5-13/14 | REMEDIATED |
| F-PLAN-R5-P1-09 custody continuity/deviation semantics incomplete | §11 | next-source invariant, processed rule, terminal destruction, action matrix | R5-16/R5-17; AC5-16/17; T5-16/17; EV5-16/17 | REMEDIATED |
| F-PLAN-R5-P1-10 power method/output/sensitivity interface incomplete | §12 | exact unit rule, outputs, overrides, recomputation | R5-18/R5-19; AC5-18/19; T5-18/19; EV5-18/19 | REMEDIATED |
| F-PLAN-R5-P1-11 metrology/calibration/QC/LOD-LOQ incomplete | §13 | exact method arrays, check specs/results, typed LOD/LOQ | R5-20/R5-22; AC5-20/22; T5-20/22; EV5-20/22 | REMEDIATED |
| F-PLAN-R5-P1-12 reference-result runtime projection incomplete | §13 | common fields, tagged branches, source binding, total mapping | R5-23/R5-24; AC5-23/24; T5-23/24; EV5-23/24 | REMEDIATED |
| F-PLAN-R5-P1-13 claim reason/transition/reinstatement incomplete | §14 | exhaustive transition matrix and trigger resolution mode | R5-29/R5-30; AC5-29/30; T5-29/30; EV5-29/30 | REMEDIATED |
| F-PLAN-R5-P1-14 monitoring/retention/incident calculations incomplete | §14-15 | closed statuses, derived results/windows, retention and incident effects | R5-31/R5-34; AC5-31/34; T5-31/34; EV5-31/34 | REMEDIATED |

## 21. Internal author audit

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
REAL_MANIFEST_VALIDATION_AUTHORITY=CLOSED
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
```

The constructive audit asks for one valid instance of every catalog schema,
every semantic ID, every complete-file hash, both signing payloads, one genesis-
through-active chain, monitoring pass/breach, permitted reinstatement,
retention audit, reference/runtime projection, power analysis, metrology check,
checker invocation/report, and pusher-independent tag. Any guess is a P1.

## 22. Required validation and handoff

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
`docs(plan): close Phase F constructive authority`. Do not amend, reset, rebase,
squash, force-push, tag, create an implementation branch, start F0, generate
keys/signatures, provision trust, or create evidence/registry/monitoring/claim
records. Immediately before push, verify local `main`, `origin/main`, and live
remote `main` all equal the required R4 starting SHA; if live remote cannot be
verified, stop before push. After a successful push, record the R5 commit SHA,
plan SHA-256, and Git blob and require a clean worktree. No later commit occurs
before fresh independent R5 rereview.

## 23. Required R5 planning-remediation handoff

```text
MHI V1 PHASE F
R5 PLANNING REMEDIATION HANDOFF

STARTING R4 SHA: 8124dda4d6a358397a4bc899024bdc4a59fbc14c
R4 PLAN SHA-256: 52e3f05fc2783f340b1178a757a292dda02e6a6efa3eea05d4cbba7dbe9677f7
R4 PLAN BLOB: 37547a23ef66bc8f45e8c550de6b67c360d73d5a
R5 PLAN REVIEW SHA: <filled only by fresh independent R5 reviewer>
R5 PLAN SHA-256: <filled after final plan bytes>
R5 PLAN GIT BLOB: <filled after final plan bytes>
CHANGED FILES: 1 expected

F-PLAN-R5-P1-01: REMEDIATED
F-PLAN-R5-P1-02: REMEDIATED
F-PLAN-R5-P1-03: REMEDIATED
F-PLAN-R5-P1-04: REMEDIATED
F-PLAN-R5-P1-05: REMEDIATED
F-PLAN-R5-P1-06: REMEDIATED
F-PLAN-R5-P1-07: REMEDIATED
F-PLAN-R5-P1-08: REMEDIATED
F-PLAN-R5-P1-09: REMEDIATED
F-PLAN-R5-P1-10: REMEDIATED
F-PLAN-R5-P1-11: REMEDIATED
F-PLAN-R5-P1-12: REMEDIATED
F-PLAN-R5-P1-13: REMEDIATED
F-PLAN-R5-P1-14: REMEDIATED

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
CATALOG_REQUIREMENTS: 36
CATALOG_ACS: 36
CATALOG_TESTS: 36
CATALOG_EVIDENCE: 36
CATALOG_OWNER_DECISIONS: 21
CATALOG_EXTERNAL_SCHEMAS_UNMAPPED: 0
TRACEABILITY_SUBSTANCE_GAPS: 0
LOST_R1_NORMATIVE_OBLIGATIONS: 0
ORPHAN_EXTERNAL_SCHEMAS: 0

R5 INDEPENDENT REREVIEW: PENDING
FROZEN PHASE-E PLAN: PASS if SHA/blob verify unchanged
PLAN TAG CREATED: NO
IMPLEMENTATION BRANCH: NO
F0 STARTED: NO
TRUST CHANGED: NO
KEYS CREATED: NO
EVIDENCE CREATED: NO
CLAIM CREATED: NO
READY_FOR_FRESH_PHASE_F_R5_PLAN_REREVIEW: yes
READY_FOR_PHASE_F_PLAN_APPROVAL_TAG: NO pending fresh R5 GO
READY_FOR_PHASE_F_IMPLEMENTATION: NO
```
