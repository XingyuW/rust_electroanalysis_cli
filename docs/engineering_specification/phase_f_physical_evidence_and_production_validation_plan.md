# MHI V1 Phase F — Architecture Plan (R12 authority hierarchy)

## 1. Status and governing authority

This is the canonical Phase-F Architecture Plan. It is a forward, planning-only
refactor of R11. It creates no approval, implementation branch, key, signature,
trust root, registry record, physical evidence, monitoring record, claim, or
release. R11 is preserved byte-for-byte at
[`phase_f/phase_f_r11_normative_source.md`](phase_f/phase_f_r11_normative_source.md)
with SHA-256 `987bc6e06a5c43873b844f864cb1f858c6b57c40c18dd0d4ed4a4edcf32dec3d`
and Git blob `34ab62d094c4cb0bb31a40dc7a192ed304faf981`.

Phase F uses layered authority. P0 and P1 always block the gate that owns the
affected requirement. An artifact with an open P0/P1 is ineligible for a
downstream authority bundle. A child specification may refine but never weaken,
override, reinterpret, or silently default a parent. A discovered architecture
gap stops downstream work and requires a forward plan revision and fresh review.

The immutable Phase-E authority remains: integrated baseline
`14942a30928b88f16914bf0bb103cc0c2a5bfa76`, reviewed implementation
`5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb`, frozen plan SHA-256
`0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33`,
and frozen plan blob `6fce9d13a42a09027e0e730874a8d80e03e6a7da`.

## 2. Scope, objectives, and non-goals

<a id="f-arch-001"></a>
`F-ARCH-001` Phase F shall establish independently reviewable authority for
physical mechanism and health validation without changing frozen Phase-E
behavior. Phase F covers F0 owner decisions, implementation readiness,
authority enrollment, scientific protocol/power/package/cohort authority,
trust provisioning, physical execution/release, and continued operations.

<a id="f-arch-002"></a>
`F-ARCH-002` Phase F shall not create a new scientific model or silently expand
the approved claim domain. Synthetic, test, constructed, model-derived, or
same-signal-derived material is not physical release evidence except where an
approved scientific claim ceiling expressly permits that category.

<a id="f-arch-003"></a>
`F-ARCH-003` Phase-E compatibility and the existing P2 hard gate remain
mandatory. Any Phase-E change requires an explicit upstream reopening.

## 3. Authority hierarchy and document ownership

<a id="f-arch-004"></a>
`F-ARCH-004` The authority tiers are:

1. this Architecture Plan;
2. the approved F0 bundle containing exactly `F-OD-01..20`;
3. the five independently reviewed specifications below;
4. the derived traceability manifest and specification-bundle manifest;
5. implementation/readiness, enrollment, scientific authority, trust, release,
   and live operations artifacts in gate order.

| Document | Requirement namespace | Exclusive responsibility |
|---|---|---|
| Architecture Plan | `F-ARCH-*` | scope, authority, gates, F0 semantics, invariants, ceilings, ownership |
| Wire and Authority Specification | `F-WIRE-*` | exact machine-facing schemas, serialization, identity, tags, registry, trust wire |
| Scientific Validation Specification | `F-SCI-*` | physical admissibility, independence, endpoints, power, metrology, claim meaning |
| Operations and Lifecycle Specification | `F-OPS-*` | state, monitoring, incidents, compromise, retention, currentness |
| Conformance and KAT Specification | `F-CNF-*` | executable tests, fixtures, properties, constructive audits |
| Implementation and Readiness Specification | `F-IMPL-*` | software boundaries, CLI, builds, reproducibility, readiness evidence |

The Architecture Plan does not own the full schema catalog, complete field
catalog, KAT bytes or SHA tables, test URIs, exhaustive fixtures/tests, exact
build transcripts/module layout, or generated inverse traceability tables.

<a id="f-arch-005"></a>
`F-ARCH-005` Every subordinate requirement shall declare `derived_from` with at
least one exact `F-ARCH-*` or applicable `F-OD-*`. Every architecture
requirement is self-closed or assigned to an exact document and gate.

## 4. F0 owner-decision authority

<a id="f-arch-006"></a>
`F-ARCH-006` The F0 decision bundle contains each ID `F-OD-01` through
`F-OD-20` exactly once in ascending order and no twenty-first decision. It owns
concrete values; this plan owns the following semantics and value shapes:

| ID | Decision and exact value shape |
|---|---|
| `F-OD-01` | Protocol identity: `{protocol_id:RUNTIME_STABLE_ID_V1,title:RUNTIME_CANONICAL_TEXT_V1}`. |
| `F-OD-02` | Registration authority: `{registration_id:RUNTIME_STABLE_ID_V1,immutable_reference_uri:RUNTIME_URI_V1,document_sha256:SHA256_V1}`. |
| `F-OD-03` | `DomainSelectorDecisionV1`: five ordered axes each `{type:"allowed",ids:[RUNTIME_STABLE_ID_V1]}` plus `temperature:{type:"bands",bands:[{lower_kelvin_inclusive:RUNTIME_F64_V1,upper_kelvin_exclusive:RUNTIME_F64_V1}]}`. |
| `F-OD-04` | Trust root: `{trust_root_id:RUNTIME_STABLE_ID_V1}`. |
| `F-OD-05` | `{mechanism_endpoints:[MechanismEndpointDecisionV1]}`. |
| `F-OD-06` | `{health_endpoints:[HealthEndpointDecisionV1]}`. |
| `F-OD-07` | `{claims:[{claim_id:RUNTIME_STABLE_ID_V1,statement:RUNTIME_CANONICAL_TEXT_V1,domain:DomainSelectorDecisionV1,supporting_endpoint_ids:[RUNTIME_STABLE_ID_V1]}]}`. |
| `F-OD-08` | One row per `SCIENTIFIC_EVIDENCE_CATEGORY_V1`, each `{category:SCIENTIFIC_EVIDENCE_CATEGORY_V1,may_support:BOOLEAN_V1,may_contradict:BOOLEAN_V1,claim_ceiling:SCIENTIFIC_CLAIM_CEILING_V1}`; `model_derived` and `same_signal_derived` cannot support physical claims; expert interpretation cannot support alone; unavailable is `false,false,unavailable`. |
| `F-OD-09` | `{split_unit:RUNTIME_STABLE_ID_V1,allocations:{development:CANONICAL_DECIMAL_V1,validation:CANONICAL_DECIMAL_V1,holdout:CANONICAL_DECIMAL_V1},stratification_keys:[RUNTIME_STABLE_ID_V1],randomization_algorithm_id:RUNTIME_STABLE_ID_V1,seed_authority:RUNTIME_STABLE_ID_V1,split_execution_authority_id:RUNTIME_STABLE_ID_V1,lock_point:"before_outcome_access",post_hoc_movement:"forbidden"}`. |
| `F-OD-10` | `{unit_kinds:[RUNTIME_STABLE_ID_V1],independent_kind_by_endpoint:[{endpoint_id:RUNTIME_STABLE_ID_V1,unit_kind:RUNTIME_STABLE_ID_V1}],identity_issuance_procedure_sha256:SHA256_V1,parent_child_rules:[{parent_kind:RUNTIME_STABLE_ID_V1,child_kind:RUNTIME_STABLE_ID_V1,procedure_document_sha256:SHA256_V1}],repeat_handling:"same_family_no_increment"}`. |
| `F-OD-11` | Complete `PhaseFMetrologyPolicyV1` as wire-closed by `F-WIRE-006` and scientifically interpreted by `F-SCI-006,F-SCI-007`. |
| `F-OD-12` | `{power_method_id:RUNTIME_STABLE_ID_V1,power_method_version:RUNTIME_CANONICAL_TEXT_V1}`; no power-interface identity, URI, or byte length exists during F0. |
| `F-OD-13` | `{authority_id:RUNTIME_STABLE_ID_V1,authority_role:"production_owner",authority_document:PhaseFObjectReferenceV1}`. |
| `F-OD-14` | `{authority_id:RUNTIME_STABLE_ID_V1,authority_role:"production_registry",registry_namespace_id:RUNTIME_STABLE_ID_V1,registry_head_resolver_uri:LIVE_REGISTRY_HEAD_URI_V1,registry_head_max_validity_seconds:DURATION_SECONDS_V1}`. |
| `F-OD-15` | `{custody_method_id:RUNTIME_STABLE_ID_V1,custody_procedure_document:PhaseFObjectReferenceV1,owner_custodian_role:RUNTIME_STABLE_ID_V1,registry_custodian_role:RUNTIME_STABLE_ID_V1,required_quorum:CANONICAL_POSITIVE_INTEGER_V1,key_input_channel_id:RUNTIME_STABLE_ID_V1,network_mode:"offline"|"hsm_isolated",key_persistence_allowed:false,production_cli_access_allowed:false}`. |
| `F-OD-16` | `{trigger_actions:[{trigger_code:ROTATION_TRIGGER_V1,required_state:PHASE_F_CLAIM_STATE_V1,revalidation_scope:"endpoint"|"full",new_approval_required:BOOLEAN_V1,new_run_required:BOOLEAN_V1,resolution_mode:PHASE_F_RESOLUTION_MODE_V1}],procedure_document_sha256:SHA256_V1,unsupported_lifecycle_action:"f3_no_go"}`; exactly one row for every `ROTATION_TRIGGER_V1`, with no missing, duplicate, or extra row. |
| `F-OD-17` | `{claim_validity_seconds:DURATION_SECONDS_V1,periodic_review_seconds:DURATION_SECONDS_V1,suspension_sla_seconds:DURATION_SECONDS_V1}`. |
| `F-OD-18` | `{deviation_actions:[{deviation_code:RUNTIME_STABLE_ID_V1,required_action:PHASE_F_DEVIATION_ACTION_V1}]}` total over permitted deviation codes. |
| `F-OD-19` | Complete `PhaseFMonitoringPolicyV1` as wire-closed by `F-WIRE-006` and operationally interpreted by `F-OPS-003`. |
| `F-OD-20` | `{allowed_immutable_uri_schemes:[URI_SCHEME_V1],retention_seconds:DURATION_SECONDS_V1,backup_copy_count:CANONICAL_POSITIVE_INTEGER_V1,backup_verification_interval_seconds:DURATION_SECONDS_V1,authorized_access_role_ids:[RUNTIME_STABLE_ID_V1],replacement_authority_role_id:RUNTIME_STABLE_ID_V1,unavailable_object_action:"no_go"}`. |

`ROTATION_TRIGGER_V1` is the closed ten-value enum `key_rotation`,
`key_compromise`, `key_revocation`, `method_version_change`,
`protocol_revision`, `domain_expansion`, `code_change`,
`sensor_design_change`, `report_withdrawal`, and `superseding_campaign`.
`PHASE_F_RESOLUTION_MODE_V1` is closed to
`same_release_reinstatement_allowed`, `new_release_required`, and
`withdraw_only`. F0-to-runtime projection constructs TOML and parses exactly
`MhiValidationProtocolV1::from_toml`; every runtime field and binary64 bit is
compared. Missing, extra, normalized, transformed, defaulted, or
unrepresentable values are F0/F1 NO-GO.

The exact expanded shapes and runtime projection preserved from R11 §4 are
wire-owned by `F-WIRE-004`; scientific interpretation is owned by
`F-SCI-003..008`; operational interpretation is owned by `F-OPS-002..006`.
There are no hidden defaults and every dependency names the exact F-OD ID.

## 5. Minimal governance core

<a id="f-arch-007"></a>
`F-ARCH-007` Independent review has exactly the roles
`scientific_metrology`, `architecture_data`, `security`, `compatibility`, and
`operations_governance`. Aggregate P0/P1 counts are the arithmetic sums of
role counts. Aggregate GO is valid if and only if all five rows are GO and both
aggregate counts are zero. Every other state is NO-GO.

<a id="f-arch-008"></a>
`F-ARCH-008` Architecture approval binds the reviewed plan commit and authorizes
only F0 closure and subordinate-specification preparation. F0 approval binds the
exact 20-decision bundle and authorizes only finalization of F0-dependent specs.
Each component specification requires its own five-role GO review. Specification
bundle approval binds the exact component bytes/reviews, architecture approval,
F0 approval, traceability manifest, and aggregate bundle review with
`approval_decision=GO`.

<a id="f-arch-009"></a>
`F-ARCH-009` The durable Phase-F tag set adds exactly
`ism-mechanism-health-v1-f-specification-bundle-approved`. Its annotated body
shall bind the architecture tag, F0 tag, exact bundle-manifest SHA-256, exact
aggregate review-bundle SHA-256, and `approval_decision=GO`. Exact grammar is
owned by `F-WIRE-007` and tested by `F-CNF-004`.

## 6. Gates and sequencing

<a id="f-arch-010"></a>
`F-ARCH-010` The gates are sequential and bind the exact approved upstream
authority:

| Gate | Required authority | Authorization |
|---|---|---|
| G0 | Architecture review GO, P0=0/P1=0 | F0 closure and subordinate-spec preparation only |
| G1 | 20-decision F0 GO, P0=0/P1=0 | F0-dependent specification finalization |
| G2 | Five individual specification reviews GO | Bundle assembly eligibility |
| G3 | Specification-bundle approval tag | Phase-F implementation may begin |
| G4 | Implementation/readiness and reproducible checker/build evidence GO | Authority enrollment |
| G5 | Enrollment review/tag and registry bootstrap | Physical-validation authority operation |
| G6 | F1/F2 protocol, power, package, cohort, scientific admissibility | Owner approval/trust preparation |
| G7 | Trust-provisioning approval | Production execution eligibility |
| G8 | Execution, release, F5 candidate, initial-state review GO | Final release tag eligibility |
| G9 | Physical-validation release tag | Released Phase-F physical claim |
| G10 | Monitoring, incidents, retention, live authority | Continued ACTIVE status |

<a id="f-arch-011"></a>
`F-ARCH-011` `READY_FOR_PHASE_F_IMPLEMENTATION=yes` is impossible before G3.
No Phase-F implementation branch may be created before G0, G1, all five G2
reviews, and G3 are complete.

<a id="f-arch-012"></a>
`F-ARCH-012` Physical evidence is eligible only when collection/use occurs under
approved G0-G5 authority plus applicable registered protocol, power,
metrology, custody, package, and cohort authority. Retrospective promotion is
forbidden unless the approved scientific specification defines and validates
an explicit path; the default is no retrospective promotion.

## 7. Scientific and production invariants

<a id="f-arch-013"></a>
`F-ARCH-013` Independent physical units—not measurements, repeated runs,
aliquots, rows, or derived values—determine evidence independence. Same-source
or same-signal derivations cannot establish orthogonal support.

<a id="f-arch-014"></a>
`F-ARCH-014` Parent/child identity, dependency, custody, cohort locking,
partitioning, blinding, reference authority, uncertainty, calibration/QC,
LOD/LOQ, statistical power, and endpoint-qualified acceptance must be closed
before a physical claim. Contradiction is fail-closed and claim ceilings are
never raised by missing or weaker evidence.

<a id="f-arch-015"></a>
`F-ARCH-015` Production owner and registry are separately appointed by F0,
enrolled only after readiness, and operate only within registered authority.
Operators, Git taggers, commit authors, and push actors are not approval
authorities. Compromised or unavailable authority cannot be bypassed.

<a id="f-arch-016"></a>
`F-ARCH-016` Claim state is an append-only authority progression with ACTIVE,
SUSPENDED, WITHDRAWN, EXPIRED, and SUPERSEDED semantics. Initial ACTIVE requires
the full F5 review chain. Monitoring breach, incident, expiry, compromise,
retention failure, or authority unavailability has fail-closed consequences.

## 8. No-hole invariants

<a id="f-arch-017"></a>
`F-ARCH-017` The following are hard gates:

- total architecture ownership and no orphan child;
- refinement only and conflict invalidates the bundle;
- total implementation and conformance coverage;
- future-real-evidence coverage for every physical/scientific oracle;
- exact F0 dependencies and no hidden defaults;
- no future-object, registry back-pointer, self-Git, release, claim-state,
  monitoring, retention, or deviation identity cycle;
- no approval bypass;
- no unresolved migrated finding;
- no implementation before G3;
- no test/synthetic/constructed/same-signal promotion;
- no Phase-E weakening.

## 9. Severity routing and escalation

<a id="f-arch-018"></a>
`F-ARCH-018` Plan-level P1 includes a missing phase/gate/authority/role, F0
semantic ambiguity, scientific ceiling/admissibility/independence defect,
pseudoreplication route, state/monitoring/incident/retention/trust ambiguity,
missing closure owner, bypass, impossible authority DAG, or Phase-E
incompatibility. Corresponding exact wire, scientific, operations, conformance,
and implementation defects are P1 at their owning G2 review and block G3.

<a id="f-arch-019"></a>
`F-ARCH-019` A subordinate P1 is classified `LOCAL_SPEC_DEFECT` or
`UPSTREAM_ARCHITECTURE_DEFECT`. Local defects require fix and re-review of that
spec. Upstream defects stop progression, require a forward plan revision and
fresh architecture review, and invalidate dependent approvals until reconciled.

## 10. Supersession and release chain

<a id="f-arch-020"></a>
`F-ARCH-020` Approval artifacts and tags are immutable. Corrections use a
forward revision, new revision-specific approval, explicit supersession in the
next bundle manifest, and renewed compatibility declarations from every child.
No release may bind a superseded plan.

<a id="f-arch-021"></a>
`F-ARCH-021` Final release transitively binds Architecture → F0 → Specification
Bundle → Readiness → Enrollment → registered scientific/physical authority →
production owner approval → Trust → Execution → Release record → five-role F5
review → initial ACTIVE → physical release tag. Operations continuously require
monitoring, live registry authority, incident handling, retention, and claim
currentness. No link is optional.

## 11. Migration and present authorization state

<a id="f-arch-022"></a>
`F-ARCH-022` The R11-to-R12 ledger is authoritative for migration completeness;
the traceability manifest is derived and contains no unique normative prose.
The specification bundle is GO only with zero aggregate P0/P1 across all five
specs, traceability integrity, and migrated-finding review.

Current state: all new documents and manifests are review candidates only.
Architecture approval, F0 approval, individual spec approvals, and G3 approval
are absent. Therefore implementation, keys, enrollment, evidence, claims, and
release remain prohibited and `READY_FOR_PHASE_F_IMPLEMENTATION=NO`.
