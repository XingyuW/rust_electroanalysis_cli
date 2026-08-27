# MHI V1 Phase F — Physical Evidence, Production Trust, and Operational Validation

## 1. Document identity and status

| Field | Authority |
|---|---|
| Document | `docs/engineering_specification/phase_f_physical_evidence_and_production_validation_plan.md` |
| Status | PLANNING-ONLY; author-side internal audit complete; independent plan review not yet performed |
| Milestone | MHI V1 Phase F |
| Supported validation platform | macOS |
| Linux | DEFERRED / UNSUPPORTED / NON-GATING |
| Production physical trust at authoring | `UNPROVISIONED`; zero roots |
| Planning authority | This file after an independent review approves its exact commit and bytes |
| Prohibited inference | This plan is not physical evidence, production trust, approval, a signature, or a physical-validation claim |

Normative terms are `MUST`, `MUST NOT`, `REQUIRED`, and `BLOCKED`. An unresolved
`F-OD` item has no default: the named stage is `NO-GO` until the required signed
decision evidence exists. Repository paths below are relative to the repository root.

## 2. Exact Phase-E release baseline

| Authority | Required value | Inspection evidence |
|---|---|---|
| Starting/local/remote `main` | `14942a30928b88f16914bf0bb103cc0c2a5bfa76` | `git rev-parse`; live `git ls-remote` |
| Reviewed Phase-E implementation | `5148b156fabf1a3bc6316c8a3f17c3dba96fc5cb` | annotated approval tag peel |
| Implementation approval tag | `ism-mechanism-health-v1-e-implementation-approved-r6` → reviewed SHA | live remote tag query |
| Integration tag | `ism-mechanism-health-v1-e-implementation-integrated` → main SHA | live remote tag query |
| Durable remote branches | `main` only | live remote head query; Phase-E temporary branch absent |
| Frozen Phase-E plan SHA-256 | `0b68359f362434ef9f42df21ca553692ae6e3bb3c096881009ab5e9473cc2c33` | `shasum -a 256` |
| Frozen Phase-E plan Git blob | `6fce9d13a42a09027e0e730874a8d80e03e6a7da` | `git hash-object` |
| Phase-E integration tests | 38/38 | pre-authoring execution |
| Phase-D public-output tests | 73/73 | pre-authoring execution |
| Approval unit / nested KAT / compile-fail | 2/2; 1/1; 4/4 | Phase-E R6 authority and `tests/phase_e_validation.rs` |
| Deterministic files / manifest records | 9/9; 8/8 | Phase-E golden bundle |
| Locked full suite | 780 distinct passes, twice | Phase-E R6 approval evidence |
| Doctests / rustdoc warnings | 15 ignored; 6 existing warnings | Phase-E R6 approval evidence |
| Historical artifact fixtures | 48 | Phase-E compatibility inventory |
| Strict Clippy diagnostics | 0 | pre-authoring execution |

`docs/engineering_specification/next_milestone_plan.md` remains immutable Phase-E
R6 authority. Phase F neither edits nor supersedes its historical facts.

## 3. Phase-F problem statement

Phase E proved that the validation framework is implemented. Phase F must prove,
for one exact claim and one exact bounded domain, that real evidence, independent
reference authority, preregistration, cohort lock, production trust, approval,
execution, and release governance form one auditable chain. Phase F does not
change a fit or association into causal mechanism proof.

The planning conclusion is deliberately narrow:

1. `MhiValidationProtocolV1`, `MhiValidationDatasetV1`,
   `OwnerApprovalEvidenceV1`, `PhysicalApprovalTrustStoreV1`, the current
   evaluator, and the current `validation run` CLI route are sufficient for the
   first immutable physical-validation release when the external owner and
   registry documents carry the campaign manifest and preregistration evidence.
2. No new scientific evaluator, artifact kind, dataset schema, report schema,
   CLI route, or runtime signer is justified for the first release.
3. The expected repository implementation is a reviewed production public-key
   trust-store provisioning change, corresponding test updates/additions,
   documentation/release evidence, and the bounded maintenance-only production
   source repair required by `F-MAINT-01`. Scientific validation source remains
   unchanged.
4. Campaign design, raw evidence, method documents, signatures, approval, and
   adjudication are external-authority work and remain blocked by `F-OD-01`–`F-OD-35`.

## 4. Scientific questions

Phase F answers four separate questions, each with its own gate:

1. **Protocol:** is the protocol scientifically complete and registered before scoring? (`F0`, `F1`)
2. **Evidence:** is the cohort physical, immutable, independent, powered, and free of development leakage? (`F2`)
3. **Trust:** are reviewed public roots provisioned without exposing private material or admitting test authority? (`F3`)
4. **Claims:** does the exact reviewed implementation emit only claims authorized by the protocol, domain, evidence, references, trust, and approval? (`F4`, `F5`)

Failure of one question cannot be compensated by success on another.

## 5. Root-cause and gap analysis

| Finding | Repository evidence | Phase-F disposition |
|---|---|---|
| Scientific scoring is complete | `src/mhi_validation/evaluation.rs`, `statistics.rs`, `partition.rs`; Phase-E 38/38 | Reuse without formula or ordering changes. |
| Protocol is closed and preregistration-capable | `MhiValidationProtocolV1`, `ProtocolRegistrationV1`, exact input-byte hash in `protocol.rs` | Reuse schema 1. The immutable registration document contains experimental design, power rationale, raw-manifest specification, limitations, and release wording; its URI/hash are already protocol-bound. |
| Dataset binds cohort semantics | `MhiValidationDatasetV1::computed_cohort_semantic_sha256` binds records, reference sources, lineage-catalog hash, and protocol hash | Reuse schema 1. The external registry record binds the raw/reference manifest hash to this cohort hash before scoring. |
| Raw files are not `ArtifactKind` nodes | closed `ArtifactKind` in `src/domain/artifact.rs` | Do not broaden scientific artifact kinds. Represent raw files in a separately reviewed external content manifest and link them to assessed/reference artifacts in the signed registry record. |
| Reference authority is represented | `ReferenceSourceAuthorityV1`, `ReferenceEndpointV1`, `ReferenceUncertaintyV1` | Reuse; method/QC/chain-of-custody documents are immutable URI+hash entries in the registration/registry documents and are reviewed before approval. |
| Runtime validates document hashes but does not retrieve URIs | `OwnerApprovalEvidenceV1` binds `ImmutableDocumentV1`; verifier checks signatures and bindings | External registry review proves document availability/content. Runtime network retrieval is explicitly not added. |
| Production trust is deliberately unavailable | embedded `config/mhi_physical_approval_trust_store.schema1.json`; early runner gate | Provision reviewed public keys only in F3; no runtime override. |
| V1 trust lacks validity/revocation fields | `PhysicalApprovalTrustRootV1` contains identities and two keys only | V1 is sufficient for one immutable release. Rotation is a new reviewed trust-store build; revocation suspends claims externally and requires replacement/revalidation. A V2 is deferred unless `F-OD-30`/`F-OD-31` require concurrent lifecycle states. |
| Approval already binds all scoring authorities | `OwnerApprovalEvidenceV1::signing_bytes` and `validate` | Reuse schema 1; ordering is proved by immutable registry sequence, not a local timestamp. |
| Current physical fixtures are KATs | `tests/fixtures/phase_e/**`, `approval_kat.rs` | Never treat them as physical evidence. |
| Current route is deterministic and atomic | `reader.rs`, `runners/mhi_validation.rs`, `output.rs` | Reuse; no fallback or new route. |

Inspection authority map:

| Area | Files inspected | Material authority used by this plan |
|---|---|---|
| Project/scientific architecture | `README.md`, `CHANGELOG.md`, `docs/engineering_specification/00_project_overview.md`, `01_system_requirements.md`, `03_module_specifications.md`, `09_testing_and_quality_assurance.md`, `11_build_release_and_operations.md`, `12_change_management_playbook.md`, `13_traceability_matrix.md`, `16_open_questions.md`, `docs/adr/0002-unified-ism-scientific-contract.md` | artifact-only system boundary; no fit-to-causality promotion; deterministic/versioned artifacts; macOS validation authority; unresolved physical-method policy remains owner-owned |
| Phase-E authority | immutable `docs/engineering_specification/next_milestone_plan.md`; `tests/phase_e_validation.rs`; `tests/fixtures/phase_e/expected/author_validation_evidence_ledger.md`; `phase_e_fixture_inventory.schema1.json`; `expected/golden_bundle/**`; production trust-store JSON | frozen formulas, readers, authority gates, test/evidence separation, exact regression counts, unprovisioned production trust |
| Production validation | `src/validation_config.rs`; `src/results/mhi_validation.rs`; every file under `src/mhi_validation/`; `src/runners/mhi_validation.rs`; `src/cli.rs`; `src/main.rs` | closed protocol/dataset/report/approval types; exclusion/separation/evaluation order; opaque verified capabilities; deterministic atomic route |
| Upstream artifacts | `src/results/mechanism.rs`, `src/results/health.rs`, `src/results/artifact_contracts.rs`, `src/reporting/reader.rs`, `src/domain/artifact.rs`, `src/domain/lineage.rs`, Phase-B mechanism and Phase-C health runners/readers | schema-4 scoreable sources, strict artifact/file authority, artifact kinds, semantic identities, acquisition families, experiment scopes, dependency catalog |
| Test-only authority | `src/mhi_validation/approval_kat.rs`; `tests/fixtures/phase_e/approval/**`, `trust/**`, `protocol/**`, `dataset/**`, `reference/**`, `lineage/**`, and `mechanism/physical/**`/`health/physical/**` | fixtures prove software behavior only; KAT identities and apparent `physical` tokens never become production evidence or trust |

## 6. Confirmed facts, owner context, and unresolved decisions

The columns are authority classes; no cell in one column supplies authority to
another.

| CONFIRMED REPOSITORY FACT | OWNER-SUPPLIED CONTEXT | UNRESOLVED DECISION OR ASSUMPTION |
|---|---|---|
| Trust is embedded, `UNPROVISIONED`, and has zero roots (`config/...schema1.json`, `approval.rs`). | First intended application concerns ion-selective membrane sensors for wastewater analysis, particularly ammonium-selective sensing. | Exact claim, analytes, matrices, designs, sensors, batches, campaigns, and temperature domain: `F-OD-01`–`F-OD-06`. |
| Physical/synthetic/constructed/unknown origins and development/validation/holdout roles exist (`validation_config.rs`). | Existing Phase-B/C outputs, OCP features, EIS evidence, calibration/reference measurements, and design/batch/campaign metadata may be relevant. | Reference methods, versions, authorities, blinding, uncertainty, and QC: `F-OD-07`–`F-OD-14`, `F-OD-26`. |
| Reference method, authority, blinding, uncertainty, dependency completeness, scope, and families exist (`validation_config.rs`, `results/mhi_validation.rs`). | The owner supplied no thresholds, sample sizes, identities, keys, or release language. | Power, records, families, strata, labels, metrics, missing data, and splits: `F-OD-15`–`F-OD-25`. |
| Dual approval binds protocol, cohort, claims, endpoints, reference authorities, domain, documents, root, and two signatures (`approval.rs`). | Production trust is unprovisioned. | Owner/registry identity, custody, rotation, revocation: `F-OD-27`–`F-OD-31`. |
| Physical and software outcomes differ; strict readers and exact hashes precede deterministic atomic publication (`evaluation.rs`, `reader.rs`, `output.rs`). | macOS is the sole supported MHI V1 validation platform. | Validity, triggers, release wording, retention/storage: `F-OD-32`–`F-OD-35`. |

## 7. Scope

In scope: close owner decisions; register a physical protocol; design and lock a
real campaign; audit chain of custody and independence; provision public trust;
perform dual approval before scoring; run the exact reviewed macOS binary; review
the deterministic bundle; issue, suspend, supersede, or withdraw an exact bounded
claim; preserve evidence and audit records.

## 8. Non-goals

Excluded: new mechanism models, health classifiers, estimators, Phase-B/C
reassessment logic, new Wilson or power formulas inside the CLI, automatic
mechanism promotion, PNP simulation, plots, GUI, cloud services, databases,
real-time acquisition, instrument control, Linux or Windows certification,
general refactoring, unrelated schema modernization, logging cleanup, unrelated
open questions, and MHI V2. Phase F may execute existing Phase-B/C producers on
physical inputs; it does not change their scientific logic.

## 9. Terminology and closed status vocabulary

| Term | Closed meaning |
|---|---|
| Physical evidence | Observations from actual specimens/instruments with complete approved chain of custody; never synthetic, constructed, unknown, or a Phase-E fixture. |
| Independent unit/family | The exact unit and grouping algorithm approved by `F-OD-15`–`F-OD-25`; no filename inference. |
| Registered | An immutable external record whose content hash and ordering proof are independently retrievable. |
| Cohort locked | Protocol hash, cohort hash, file-manifest hashes, domain, claims, endpoints, references, thresholds, origins, limitations, and code baseline are in one immutable registry record. |
| Outcomes | `PhysicallyValidated`, `SoftwareValidatedOnly`, `DoesNotMeetProtocol`, `Indeterminate`; no aliases. |
| Review decisions | `GO` or `NO-GO` only; accepted P2 debt is separate. |
| Failure classes | hard error; record exclusion; `Indeterminate`; `DoesNotMeetProtocol`; `NO-GO`; claim suspension/withdrawal. |

## 10. Claim ceiling

| ID | Invariant |
|---|---|
| F-SCI-01 (`F-SCI-CEILING-01`) | Synthetic evidence can never support a physical claim. |
| F-SCI-02 (`F-SCI-CEILING-02`) | Constructed evidence can never support a physical claim. |
| F-SCI-03 (`F-SCI-CEILING-03`) | Unknown-origin evidence can never support a physical claim. |
| F-SCI-04 (`F-SCI-CEILING-04`) | Phase-E fixtures, KAT roots, and KAT approvals can never be production evidence or authority. |
| F-SCI-05 (`F-SCI-CEILING-05`) | Software validation is not physical validation. |
| F-SCI-06 (`F-SCI-CEILING-06`) | Fit, correlation, agreement, classification, or low error alone is not physical-mechanism proof. |
| F-SCI-07 (`F-SCI-CEILING-07`) | Same-artifact, same-pipeline, same-signal, shared-event, shared-fit, shared-preprocessing, or shared-model references are non-independent and cannot support a validated mechanism claim. |
| F-SCI-08 (`F-SCI-CEILING-08`) | Phase-C outputs or transformations of their inputs cannot be independent health references. |
| F-SCI-09 (`F-SCI-CEILING-09`) | Insufficient, incomplete, unblinded, uncertain, overlapping, unknown, or underpowered evidence retains the existing hard-error, exclusion, `Indeterminate`, or `DoesNotMeetProtocol` result; no rescue, rounding, pooling, or post-hoc removal. |
| F-SCI-10 (`F-SCI-CEILING-10`) | `PhysicallyValidated` is limited to the exact claim, endpoints, protocol, domain, references, cohort, code, trust store, approval, report, limitations, and validity state. |

## 11. F0–F5 stage model

| Stage | Entry | Allowed / prohibited | Required artifacts | Responsible / independent review | Exit and blockers | Source/trust/scoring/claim |
|---|---|---|---|---|---|---|
| F0 Readiness and decision freeze | Phase-F plan tag approved; main/tag authority verified | Decide and document only; no campaign acquisition, signing, or scoring | `F-OD-01`–`35` decisions; role assignments; gap disposition | Project owner / all five review roles | All outcome-changing decisions closed; ambiguity is `NO-GO` | Source: plan-authorized only; trust remains unprovisioned; scoring/claim forbidden |
| F1 Campaign registration | F0 `GO` | Author protocol/design/power/manifest specifications; prohibit viewing/scoring holdout outcomes and post-registration thresholds | exact protocol bytes/hash; immutable registration document; split/randomization/blinding/COC plan | Scientific lead / metrology + data governance | Registration ordering proof and complete protocol; missing design or power authority blocks | Source: protocol/test support only; trust may remain unprovisioned; scoring/claim forbidden |
| F2 Acquisition and cohort lock | F1 `GO` | Acquire real data, create upstream artifacts and references, audit/lock; prohibit post-lock edits or cohort movement | raw/reference manifest; Phase-B/C artifacts; lineage catalog; dataset; deviation ledger; registry lock record | Campaign custodian / scientific + data authority | All bytes/hash/identity/independence/power checks pass; mutation creates new cohort | Source: no scientific-logic change; trust may remain unprovisioned; scoring/claim forbidden |
| F3 Trust and pre-scoring approval | F2 `GO` | Provision reviewed public roots, reproducibly build, prepare and dual-sign exact payload; prohibit private material in repo/runtime/CI/logs and prohibit scoring before registration | trust-store diff/hash; key attestations; unsigned/signed approval; build evidence; registry ordering proof | Security custodian and two signers / security + compatibility | provisioned embedded trust, distinct authorities/keys, valid signatures/bindings, approval earlier than scoring | Source may remain unchanged; trust must be provisioned; scoring/claim forbidden |
| F4 Blind execution and adjudication | F3 `GO` | Run only exact route and inputs; prohibit tuning, rescue, replacement, fallback, and unpublished reruns | executable hash/SHA; environment record; command log; 9-file bundle; replay review; deviation ledger | Validation operator / all five roles | every required endpoint/stratum/rule/authority and replay passes; otherwise exact negative/indeterminate result | Source frozen; trust provisioned; scoring allowed once; claim only as candidate result |
| F5 Release and validity | F4 reviewed result; `F-OD-32`–`35` closed; P2 gate met | Issue exact wording, monitor, suspend/withdraw/supersede; prohibit domain generalization | release claim record; validity/monitor plan; incident/rollback record; archive | Release owner / all five roles | unanimous `GO`, zero P0/P1, no integrity/trust/scientific risk, monitoring active | Source frozen; trust provisioned; no public physical claim before final `GO` |

## 12. Owner-decision register

Every decision evidence is a signed immutable owner record with decision ID,
selected value, rationale-document URI+SHA-256, approver identity, and registry
record URI+SHA-256. “Begin” means repository readiness work, never campaign
execution or scoring.

| ID | Title / why repository cannot answer | Value type and allowed grammar; units | Evidence required; behavior effect | Blocked / begin? / no default | Authority / approval evidence |
|---|---|---|---|---|---|
| F-OD-01 | Initial physical claim wording; code has only free-text `statement` | nonempty UTF-8 exact string plus stable `claim_id=[a-z0-9][a-z0-9_-]*` | scientific/legal review; fills `ReleaseClaimV1` and release record | F0–F5 / no / no generic wording | project owner / signed decision |
| F-OD-02 | Target analytes | sorted unique stable IDs | domain evidence; fills every domain selector | F0–F5 / no / no ammonium default | scientific owner / signed decision |
| F-OD-03 | Target matrices | sorted unique stable IDs | matrix characterization; domain/strata | F0–F5 / no / no wastewater subclass default | scientific owner / signed decision |
| F-OD-04 | Sensor designs | sorted unique stable IDs | controlled design records; domain/strata | F0–F5 / no / no design default | product/scientific owner / signed decision |
| F-OD-05 | Sensor/batch/campaign inclusion | closed lists and predicates over stable IDs | manufacturing/campaign records; dataset eligibility | F0–F4 / no / no inclusion inference | campaign owner / signed decision |
| F-OD-06 | Temperature domain | sorted disjoint `[lower,upper)` finite positive kelvin bands | environmental rationale; domain/strata boundaries | F0–F4 / no / no range default | metrology owner / signed decision |
| F-OD-07 | Mechanism reference methods | sorted `{method_id,method_version}` | orthogonality and method-validity dossier; reference rule | F0–F4 / no / no method default | scientific owner / signed decision |
| F-OD-08 | Health reference methods | sorted `{method_id,method_version}` | independent-label validity dossier; reference rule | F0–F4 / no / no method default | clinical/analytical owner / signed decision |
| F-OD-09 | Reference method versions | exact nonempty version strings | immutable method documents; method matching | F0–F4 / no / no “latest” | metrology owner / signed decision |
| F-OD-10 | Reference authorities | sorted unique stable authority IDs | accreditation/competence and conflict review; allowed IDs | F0–F4 / no / no authority default | project owner / signed decision |
| F-OD-11 | Blinding | `require_blinded` only for physical endpoints unless claim is withdrawn to software-only | blinding SOP; protocol state | F0–F4 / no / no inferred blinding | scientific owner / signed decision |
| F-OD-12 | Uncertainty measure IDs | stable IDs | method dossier; protocol/reference matching | F0–F4 / no / no measure alias | metrology owner / signed decision |
| F-OD-13 | Uncertainty units | exact case-sensitive unit strings | SI/measurement traceability; exact unit matching | F0–F4 / no / no conversion | metrology owner / signed decision |
| F-OD-14 | Uncertainty maxima | finite nonnegative binary64 values in F-OD-13 units, inclusive | uncertainty budget; `maximum_inclusive` | F0–F4 / no / no threshold | metrology owner / signed decision |
| F-OD-15 | Minimum eligible records | positive integers per endpoint and stratum | preregistered power analysis; protocol minima | F0–F4 / no / no `2` as campaign default | statistician / signed decision |
| F-OD-16 | Minimum independent families | positive integers per endpoint/stratum | cluster-aware power analysis; protocol minima | F0–F4 / no / no `2` as campaign default | statistician / signed decision |
| F-OD-17 | Required strata | sorted IDs plus closed `StratumPredicateV1` lists | heterogeneity rationale; protocol strata | F0–F4 / no / no omitted subgroup | scientific owner / signed decision |
| F-OD-18 | Health label universe | sorted disjoint positive/negative label sets | independent labeling SOP; health endpoint mappings | F0–F4 / no / no binary coercion | health authority / signed decision |
| F-OD-19 | Mechanism support/contradiction policy | allowed support levels plus fixed `any_contradicted_record_fails`; category eligibility table | orthogonality review; mechanism rules | F0–F4 / no / no proxy promotion | mechanism authority / signed decision |
| F-OD-20 | Acceptance metrics/thresholds | sorted `AcceptanceRuleV1`; finite rates in `[0,1]`, integer counts | prospective analysis and loss tradeoff; rule evaluation | F0–F4 / no / no threshold | statistician + owner / signed decision |
| F-OD-21 | Wilson-bound target use | `point_estimate`, `lower_confidence_bound`, or `upper_confidence_bound` per rate rule | statistical rationale; `RateTargetV1` | F0–F4 / no / no target | statistician / signed decision |
| F-OD-22 | Missing/invalid treatment | closed mapping to hard error, exclusion, indeterminate, or protocol fail | missingness analysis; protocol/deviation SOP | F0–F4 / no / no imputation/rescue | statistician / signed decision |
| F-OD-23 | Cohort split strategy | split unit, ratios/counts, strata, seed/randomization authority, lock timing | leakage/power simulation independent of holdout outcomes | F0–F2 / no / no random split default | statistician/data owner / signed decision |
| F-OD-24 | Sensor reuse across cohorts | `forbidden` or explicit history-separated rule that still records shared identity | carryover evidence; split/lineage behavior | F0–F2 / no / no reuse | scientific owner / signed decision |
| F-OD-25 | Repeated measures independence | mapping of technical/repeated exposure/sensor/batch/sample/day/campaign/reference units to family IDs and effective counts | hierarchical design/power review; family construction | F0–F4 / no / no record independence | statistician / signed decision |
| F-OD-26 | Reference calibration/QC | exact checks, intervals, materials, acceptance limits, deviation behavior | method validation and QC records; source eligibility | F0–F4 / no / no “valid method” shortcut | metrology owner / signed decision |
| F-OD-27 | Production owner identity | one stable authority ID and immutable authority document | identity verification and role charter; trust root/approval | F0–F3 / provisioning prep only / no identity | project owner / signed appointment |
| F-OD-28 | Registry identity | distinct stable authority ID and immutable charter | independence/conflict verification; trust root/approval | F0–F3 / provisioning prep only / no identity | governance owner / signed appointment |
| F-OD-29 | Key custody | approved HSM/offline process, named custodians, quorum, input/transfer rules | security threat model and ceremony rehearsal | F0–F3 / non-key test support only / no local key file | security owner / signed security decision |
| F-OD-30 | Rotation | replacement/addition/removal sequence and report impact | lifecycle threat model; trust-store/rebuild procedure | F0–F5 / test support only / no rotation interval | security owner / signed policy |
| F-OD-31 | Revocation | trigger authority, suspension SLA, registry record, replacement/revalidation rule | incident model; operational state machine | F0–F5 / test support only / no grace period | security + release owner / signed policy |
| F-OD-32 | Claim validity duration | positive duration with unit and review schedule | drift/stability evidence; release record | F0/F5 / earlier implementation yes / no perpetual claim | release owner / signed decision |
| F-OD-33 | Revalidation triggers | exact classification table for section 28 | scientific/change-risk analysis; operations | F0/F5 / earlier implementation yes / no silent continuation | change authority / signed decision |
| F-OD-34 | Release wording | exact four-outcome templates | scientific/legal review; report-to-release projection | F0/F5 / earlier implementation yes / no marketing paraphrase | release owner / signed decision |
| F-OD-35 | Retention/storage | immutable URI grammar, repository/provider, access, duration, backups, replacement authority | data-governance/legal review; package inventory | F0–F5 / tooling prep only / no local-only archive | data owner / signed policy |

## 13. Physical-validation protocol

`MhiValidationProtocolV1` is sufficient unchanged. The normative instance is an
exact UTF-8, no-BOM TOML file accepted by `from_toml`; its original bytes define
`protocol_sha256`. It MUST contain:

- one registered `protocol_id`, title, registration ID, immutable registration
  URI and SHA-256;
- `physical_approval_authority=embedded_trust_root` with the F3 root ID;
- the exact `target_domain` from `F-OD-02`–`06`;
- physical claims/statements (`F-OD-01`, `34`) and supporting endpoints;
- holdout mechanism/health endpoints with allowed methods/versions/authorities,
  required blinding, quantified uncertainty, support/contradiction policy,
  status/label partitions, minima, required strata, and acceptance rules;
- the frozen statistics tokens `wilson_95_v1`, `0.95`, `unavailable`,
  `indeterminate`, and `and`;
- limitations in the immutable registration document and again in signed
  approval/release records.

All owner-supplied fields map to `F-OD-01`–`21`. No threshold or label may be
changed after registration or after any holdout prediction/reference pairing is
opened. The protocol schema, reader behavior, public API, and future-version
rejection remain unchanged.

## 14. Experimental design

The independent experimental unit and cluster hierarchy are not inferable from
the repository and are fixed by `F-OD-23`–`25`. The registration document MUST
enumerate, for each observation, whether it is a sensor, membrane batch, device,
wastewater sample, experiment, acquisition family, campaign, or reference
measurement unit. Technical replicates, repeated reads, repeated exposures, and
multiple algorithm outputs from one source MUST NOT increase independent-family
count or effective sample size unless the signed `F-OD-25` rule explicitly
identifies distinct independent units.

The design MUST freeze before acquisition/scoring: split unit and timing,
stratification, randomization algorithm/seed authority, blinding, prohibited
reuse, discarded-run handling, deviations, and no post-hoc cohort movement.
Sensor identity, batch, sample identity, membrane age, conditioning, prior
exposure, washout, drift, calibration history, order, and environmental history
are recorded in the external campaign manifest. Known reuse is encoded as a
shared family/ancestor and cannot pass separation.

The protocol records or controls, without invented ranges: temperature in
kelvin, pH, ionic strength, conductivity, competing ions, matrix composition,
flow/mixing, exposure duration, equilibration, reference-electrode state, and
instrument configuration. Approved categorical/range claim axes enter
`DomainSelectorV1`/strata; other controlled variables enter the registration
document and limitations. Out-of-range records are not moved into the domain.

## 15. Physical evidence package

The external package manifest uses canonical UTF-8 JSON with duplicate keys
forbidden. Entries sort by `logical_id`; duplicate `logical_id`, URI, or
`sha256` is a hard error. Each entry contains exactly: `logical_id`, `role`,
`immutable_uri`, lowercase 64-hex `sha256`, nonnegative integer `byte_length`,
`media_type`, `format_or_schema`, `producing_authority_id`, `physical` boolean,
`test_only` boolean, `generated` boolean, `direct_dependency_ids`, and
`retention_class_id`. This is an external registered document, not a new
repository artifact/schema and not a runtime input.

| Package role | Location class | Immutability/binding |
|---|---|---|
| Protocol and normalized report copy | external registered file; report-generated copy | exact bytes SHA; report carries normalized protocol |
| Raw physical data and experiment metadata | external immutable objects | URI, SHA-256, length, media type; manifest and chain-of-custody ledger |
| Phase-B/C assessed artifacts and dataset/lineage catalog | executable input package | strict file hashes, semantic IDs, catalog hash, dataset cohort hash |
| Reference sources, method and calibration/QC documents | external immutable objects; source files in dataset package when runtime-consumed | source hashes plus registration/manifest/registry bindings |
| Owner authority document and registry record | external immutable documents | URI+SHA bound into signed approval payload |
| Approval and trust store | executable input / embedded build input | approval file SHA and signatures; embedded trust-store SHA |
| Binary/code/environment | release evidence | Git SHA, binary SHA-256, lock/toolchain/platform inventory |
| Nine-file report bundle | generated release candidate | manifest checksums, report reconstruction, atomic publication |
| Release claim, limitations, deviation ledger | external registry/release records | immutable URI+SHA; supersession rather than replacement |

Silent replacement is forbidden. Any changed byte gets a new URI/object version,
new manifest hash, new registry record, new cohort/approval when material, new
run, and new release decision. `F-OD-35` supplies retention, backups, access, and
replacement authority. Large raw datasets are not committed.

## 16. Reference authority and metrology

Every reference endpoint uses the existing fields: source ID, endpoint ID and
binding, method ID/version, authority ID, physical origin, blinding state,
uncertainty, limitations, source dependencies, experiment scope, acquisition
families, chain-of-custody manifest entries, and calibration/QC documents.
Missing source, incomplete dependency graph, unknown physical origin, wrong
method/authority/binding, unapproved blinding, or failed QC follows the exact
cataloged failure; physical requests fail closed before a claim.

Mechanism categories are:

| Category | Support | Contradict | Result ceiling |
|---|---|---|---|
| Direct physical observation | only if `F-OD-07/19` admits it | yes | approved physical mechanism claim |
| Orthogonal physical measurement | only if dependencies prove separation | yes | approved physical mechanism claim |
| Validated proxy | only as the limited claim named by `F-OD-19` | yes | limited wording only |
| Model-derived | no independent support | may contradict if preregistered | `NotAssessed`/limited |
| Same-signal-derived | never | may expose contradiction | never `ValidatedForDomain` |
| Expert interpretation | never alone | may trigger review | `NotAssessed` |
| Unavailable | no | no | `Unavailable`/`Indeterminate` |

OCP/EIS agreement is not automatically independent; shared sensor event, raw
record, preprocessing, fit, or assumption is a declared dependency. Health
labels come from the `F-OD-08/18/26` independent method and are frozen blinded;
Phase-C status, signal-quality transformations, ambiguous states, and post-hoc
labels cannot be references.

No repository method registry is added. Protocol-declared method/version pairs
plus immutable method documents (URI+SHA in registration/manifest/registry) are
sufficient for the first release. A registry schema is reconsidered only if
multiple campaigns require method lifecycle queries not expressible by exact
version/hash.

## 17. Uncertainty and units

For each quantified reference, `measure_id`, finite `value`, exact `unit`, and
finite nonnegative `maximum_inclusive` are mandatory, with calibration evidence,
significant-figure policy, and dependence/covariance treatment in the method
document. Comparison is exactly:

```text
reference_uncertainty_value <= maximum_inclusive
```

Equality passes. One representable value above fails. No implicit unit
conversion, tolerance, rounding, or threshold rounding exists in the runtime.
Any conversion occurs before package lock under the method document, records
source and target values/units/conversion authority, and produces the exact
protocol unit. Unavailable uncertainty, mismatched measure, or mismatched unit
cannot support a physical endpoint. Temperature is kelvin; every claim-relevant
activity, concentration, potential, impedance, time, frequency, score, and
mechanism-reference quantity has an exact unit string fixed by `F-OD-13`.

## 18. Statistical equations and power

The Phase-E operations are unchanged. Mechanism accounting is:

```text
n = s + c + u
support_fraction = s / n
contradiction_fraction = c / n
not_assessed_fraction = u / n
```

Health accounting is:

```text
evaluable_count = TP + TN + FP + FN
sensitivity = TP / (TP + FN)
specificity = TN / (TN + FP)
false_positive_rate = FP / (FP + TN)
false_negative_rate = FN / (FN + TP)
balanced_accuracy = (sensitivity + specificity) / 2
```

Coverage uses the current evaluator denominator: evaluable health rows divided
by eligible rows; `Indeterminate` and `DataQualityInsufficient` remain outside
TP/TN/FP/FN and are separately counted/rated.

Wilson 95% uses `z=1.959963984540054`, then exactly the operation order in
`statistics::wilson_95_checked`: `p=x/n`; `z2=z*z`;
`d=1+z2/n`; `centre=(p+z2/(2*n))/d`;
`radicand=p*(1-p)/n+z2/(4*n*n)`;
`half=z/d*sqrt(radicand)`; clamp lower to 0 and upper to 1. Zero denominator
is `Unavailable`; `x>n` or counts above `2^53` hard-fail; finite/serialization
behavior remains certified.

The sample-size/power method is external, deterministic, and independently
reviewed under `F-OD-15`–`25`; absence blocks F1. It MUST model clustering by
sensor, batch, experiment, family, sample, and reference source, state effect
size/null/alternative, error targets, attrition/missingness, class and stratum
requirements, and produce exact per-endpoint record/family/positive/negative/
stratum minima. Exact minima pass; one below is `Indeterminate`; empty required
strata are `Indeterminate`; no aggregate rescue, post-hoc pooling, subgroup
removal, p-value, or confidence rule is permitted unless preregistered.

## 19. Holdout, lineage, and leakage

Every assessed Phase-B/C artifact carries exact file SHA, artifact kind/schema,
known lineage ID/semantic hash, experiment scope, sensor scope, channel scope,
and acquisition families. Every reference source carries exact source hash,
scope, families, completeness, and dependencies. Raw-file identities and sample,
sensor, batch, and campaign identities live in the external manifest; its graph
maps each raw object to reference and assessed artifacts. The registry reviewer
cross-checks that graph against the repository lineage catalog before approval.

Current closure traversal detects shared artifacts, source hashes, experiments,
families, missing nodes/leaves/ancestors, cycles, incomplete dependencies, and
legacy unknowns. Known overlap cannot pass; unknown separation remains unknown.
Development comparators freeze at registration/cohort lock. Copying/renaming is
detected by hashes and semantic IDs; duplicate samples/sensors/batches are
detected by stable physical-unit IDs and the `F-OD-25` grouping rule. A reference
derived from an assessed artifact is rejected as independent. Filename,
directory, timestamp, and naming convention never prove independence.

## 20. Dataset/schema gap analysis

| Structure | Decision and convention |
|---|---|
| `MhiValidationDatasetV1` | Sufficient unchanged; signed registry record binds its cohort hash to the external physical manifest. |
| `ValidationRecordV1` | Sufficient with stable sensor/campaign/domain IDs and external sample/batch/unit manifest mappings. |
| `ReferenceSourceAuthorityV1` | Sufficient with external method/QC/COC document bindings. |
| `ReferenceEndpointV1` | Sufficient unchanged for exact method/version/authority/blinding/outcome bindings. |
| `ReferenceUncertaintyV1` | Sufficient unchanged; no conversion at runtime. |
| `DatasetProvenanceV1` | Sufficient unchanged and may remain empty for real evidence; fixture-only optional fields never establish physical origin. |
| `OwnerApprovalSourceV1` | Sufficient unchanged; exact relative path, hash, schema, and record ID. |
| `DomainKeyV1` | Sufficient with external manifest carrying sample/batch/history fields; only claim axes enter domain. |
| `ExpectedLineageV1` | Sufficient; physical claims require known lineage, with unknown separation unable to pass. |
| `DeclaredScopeV1` | Sufficient if `F-OD-25` deterministically constructs known acquisition families. |

No additive field or new schema is planned. Operational wall-clock data remains
outside scientific semantic identity. If F1 review proves the external signed
binding cannot enforce an invariant, F1 is `NO-GO` and a new separately reviewed
plan amendment must define exact schema/version/canonicalization/migration/tests;
implementers may not improvise a field.

## 21. Preregistration and cohort lock

The ordering authority is the independent immutable registry, not a local clock.
F1 registers exact protocol bytes/hash and design before holdout scoring. F2
registers a cohort-lock record containing protocol hash, dataset cohort hash,
raw/reference manifest hashes, claims, endpoints, domain, methods, authorities,
strata, thresholds, code baseline, physical origin, and limitations. F3 approval
binds the cohort hash and the immutable registry-record SHA. The registry record
MUST expose an append-only sequence/predecessor or institutional registration ID
proving its existence before the F4 execution record. Any material amendment
gets a new protocol/cohort/approval/run/release; the old approval is never reused.

## 22. Trust-store gap analysis

V1 supports sorted multiple roots, distinct owner/registry IDs and keys, global
authority/key uniqueness, strict canonical non-weak Ed25519 keys, and exact
trust-store hashing. It does not encode dates, domain scope, root status,
revocation, or retirement. It is sufficient for the first immutable release
because one reviewed root is selected by the exact protocol, every report binds
the trust-store hash/root ID, and lifecycle decisions suspend claims externally
and require a new reviewed embedded-store build. It is not a general online PKI.

If `F-OD-30/31` requires simultaneous active/retired/revoked semantics in one
binary, provisioning is blocked and a separately approved additive V2 or
immutable trust registry is required. Phase-F implementers may not silently
encode lifecycle in root IDs.

## 23. Key lifecycle and provisioning

Keys are generated in the `F-OD-29` approved offline/HSM environment using its
approved entropy source. Owner and registry custodians are distinct. Each
public key is exported as canonical 32-byte Ed25519 hex; fingerprint is
`sha256(raw_public_key_bytes)` and is verified out-of-band with authority
documents. Independent reviewers verify identities, distinct keys, non-weak
canonical encoding, root ordering, and the exact trust-store SHA before insertion.

Private keys MUST NOT enter this repository, fixtures, logs, CI, release
artifacts, production configuration/CLI, environment-variable bypasses, or the
runtime. Test roots remain reachable only under test boundaries. Provisioning
changes only public configuration and required tests/docs; the release binary is
reproducibly built and security-reviewed. Rollback restores the last approved
unprovisioned or provisioned public store through a new forward commit; it never
rewrites history. Compromise triggers immediate suspension under `F-OD-31`.

## 24. Owner/registry approval ceremony

1. A ceremony preparer constructs the unsigned canonical payload from locked
   protocol/dataset/registry inputs; the preparer is neither signer.
2. Scientific/data/security reviewers independently recompute every binding and
   the JCS signing bytes (`mhi_owner_approval_signature_v1\0` prefix).
3. Owner signer and registry signer separately receive identical bytes plus a
   printed SHA-256 through the `F-OD-29` channel; each verifies identity and signs offline.
4. A non-signer assembler inserts the two signatures only. No other field changes.
5. Independent verification checks file hash, record ID, both signatures,
   authority/key separation, trust root, protocol/cohort/claim/endpoint/reference/
   domain bindings, physical-origin confirmation, limitations, and registry ordering.
6. The final file hash is written to the immutable registry before F4.
7. Rejected/withdrawn approval is a new registry status record; correction or
   compromise never edits the old bytes and requires a new approval.

No person or agent performs both authority roles. `OwnerApprovalEvidenceV1`
remains sufficient; no timestamp field is added because registry ordering is the
authority.

Signing/provisioning option analysis: a production CLI signer and test signer
are rejected (attack surface/authority leakage); an in-repository shipped binary
is rejected. An offline `xtask`, separate non-shipped binary, separate repository,
or institutional process remains selectable only by `F-OD-29`. The selected
process must be offline unless explicitly approved, accept key input only through
the custody mechanism, retain no key, emit only an approval/signature artifact,
document memory and secure-deletion limits, have canonical-byte KATs and negative
tests, and remain outside production distribution.

| Option | Private-key exposure / production surface | Reproducibility and canonical bytes | Audit, dependencies, tests, usability | Decision |
|---|---|---|---|---|
| No repository signer | none in this repository; zero runtime surface | external process must consume independently verified JCS bytes/hash | no new dependency; audit/test authority is external; operational transfer is defined by F-OD-29 | admissible only when the institutional process proves exact bytes |
| Test-only signer | test KAT keys only; any production use is P0 | existing KATs reproduce canonical construction | testable, but intentionally unusable for custody | prohibited for production |
| Offline `xtask` | key enters a repository-built developer tool process; never production binary | can share audited canonical-byte code and KATs | adds build/dependency/audit surface; distribution must exclude release package | candidate under F-OD-29 security review |
| Separate non-shipped binary in this repository | key enters isolated binary process; production CLI surface remains zero | can pin exact payload format and vectors | explicit Cargo/dependency boundary and negative tests required; accidental shipping risk must be tested | candidate under F-OD-29 security review |
| Separate repository/tool | isolates production repository and dependencies | tool version/source/hash and cross-tool vectors must be frozen | stronger distribution boundary; separate review/release/operations burden | candidate under F-OD-29 security review |
| External institutional signing | key remains in institutional HSM/process | institution must attest exact input bytes/hash and algorithm | highest authority separation; auditability/usability depend on immutable ceremony evidence | candidate under F-OD-29 security review |

## 25. Production execution path

The exact command is:

```sh
electroanalysis validation run \
  --protocol <PATH> \
  --dataset <PATH> \
  --output-dir <PATH> \
  [--overwrite]
```

The frozen path is `cli.rs::ValidationCommand::Run` → `main.rs` artifact-only
dispatch → `runners/mhi_validation.rs::run_mhi_validation` → strict protocol
bytes/TOML/hash → embedded trust load and pre-dataset `UNPROVISIONED` rejection
for physical requests → `ValidationInputs::read` strict dataset/catalog/source
reads and bindings → strict approval read/hash/signatures/bindings →
`evaluate_mhi_validation` → `authorize_publication` report replay →
`publish_authorized_bundle` locked, no-follow, checksum-verified atomic commit.
Success prints the output directory and exits 0; typed errors carry context and
exit nonzero. No failed authority gate has fallback. Structural validation alone
cannot publish. Public callers cannot construct verified trust/approval/input or
publication capabilities.

Required order, with no commutable authority gate, is:

1. identify the request as software or physical from the validated protocol;
2. load and validate embedded trust for a physical request;
3. reject `UNPROVISIONED` before dataset opening or scientific scoring;
4. validate exact protocol bytes and byte hash;
5. strictly read and structurally/semantically validate dataset bytes;
6. validate protocol/dataset/hash/domain/endpoint bindings;
7. strictly read approval bytes and verify the declared approval file hash;
8. verify both signatures and every approval/trust/protocol/cohort/domain binding;
9. strictly read Phase-B/Phase-C sources and validate reference authority;
10. resolve lineage/reference closure and cohort separation;
11. partition every record exactly once for each endpoint/view;
12. evaluate the frozen scientific metrics, minima, strata, and rules;
13. construct only the four closed release outcomes;
14. reconstruct/replay the complete report and authorize publication;
15. commit the checksum-verified nine-file bundle atomically and return success.

## 26. Physical claim composition

A released physical claim record states exact claim ID and approved wording,
domain, supporting endpoints, protocol version/hash, cohort ID/hash, reference
methods/versions/authorities, code/release SHA and binary hash, trust-store hash
and root ID, approval ID/file hash, report ID/bundle manifest hash, operational
validation date, limitations, validity interval/review status, and registry URI/hash.
Date is provenance, never scientific identity.

## 27. Release wording

Exact templates are supplied by `F-OD-34`. Constraints are fixed:

| Outcome | Trigger / authority | Release, deployment, and scientific citation | Prohibited implication |
|---|---|---|---|
| `PhysicallyValidated` | every supporting endpoint and required stratum meets protocol; physical origin, known separation, power, production trust, dual approval, replay, and F5 `GO` | deploy/cite only exact F-OD-01 wording and domain with limitations/validity | causality beyond mechanism policy; another design/matrix/campaign/range/method/platform |
| `SoftwareValidatedOnly` | software request meets protocol, or only software authority exists | cite software conformance; no deployment represented as physically validated | any physical performance/origin claim |
| `DoesNotMeetProtocol` | required rule false, known overlap, or critical contradiction under current evaluator | physical-claim deployment prohibited; may be cited only as the exact negative protocol result with report/domain/limitations | “nearly validated” or rescued pass |
| `Indeterminate` | unknown separation, insufficient/empty/underpowered view, unavailable required uncertainty/rule, or propagated required-stratum indeterminacy | physical-claim deployment prohibited; may be cited only as indeterminate with the exact reason and limitations | pass/fail coercion, performance conclusion, or pooled rescue |

## 28. Operational validity and revalidation

`F-OD-33` must instantiate this classification before F5. The following are
minimum actions; the signed decision may require a stronger action but never a
weaker one.

| Change/event | Classification / minimum action |
|---|---|
| Phase-B scientific logic | immediate suspension; full physical cohort revalidation |
| Phase-C health logic | immediate suspension; full physical cohort revalidation |
| Phase-E/F evaluator logic | immediate suspension; full physical cohort revalidation |
| Protocol threshold | supersede protocol; full physical cohort revalidation |
| Claim wording | new protocol/approval; full revalidation unless the new wording is strictly narrower and F-OD-33 classifies it documentary before viewing results |
| Reference method | affected-endpoint revalidation; full if prospective comparability is absent |
| Reference method version | affected-endpoint revalidation; full if prospective comparability is absent |
| Reference authority | affected-endpoint revalidation and new approval |
| Uncertainty rule | immediate suspension; full physical cohort revalidation |
| Target-domain expansion | full physical cohort revalidation for the expanded domain |
| Sensor-design change | immediate suspension for changed design; full physical cohort revalidation |
| Membrane-formulation change | immediate suspension for changed formulation; full physical cohort revalidation |
| Fabrication-process change | immediate suspension for changed process; full physical cohort revalidation |
| Solid-contact change | immediate suspension for changed contact; full physical cohort revalidation |
| Reference-electrode change | affected-endpoint revalidation; full if prospective comparability is absent |
| Instrument change | affected-endpoint revalidation; full if prospective comparability is absent |
| Sampling-process change | immediate domain review; full revalidation if the registered sampling authority changes |
| Wastewater-matrix shift | immediate suspension outside old matrix; full validation for new matrix |
| Temperature-range change | immediate suspension outside old range; full validation for added range |
| Key rotation | documentary/security review, new embedded store, new approval, and deterministic endpoint rerun because the report binds trust hash |
| Key revocation | immediate claim suspension; replacement root, approval, rerun, and security GO before reinstatement |
| Trust-store change | documentary/security review, new approval, deterministic rerun; scientific cohort may remain only if its bytes are unchanged |
| Approval correction | immediate suspension; new approval and rerun; new cohort if any cohort-bound field changes |
| Lineage correction | immediate suspension; new cohort identity, approval, and full rerun |
| Discovered leakage | immediate suspension; new independent cohort and full revalidation; withdraw unsupported prior claim |
| Data-integrity incident | immediate suspension; integrity investigation and new cohort/full revalidation unless the incident is proved outside every bound byte |
| Material operational drift | immediate domain review; partial/full revalidation under the preregistered trigger; suspend while domain compliance is unknown |

An editorial change outside every hash-bound authority is documentary review
only; any bound-byte change is not editorial.

Claims expire after the exact `F-OD-32` duration, undergo its periodic review,
and transition only through active, suspended, withdrawn, expired, or superseded
registry states. Audit retention and rollback follow `F-OD-35`; rollback cannot
revive an expired/revoked claim.

## 29. Monitoring and incident response

External operating procedures—not the validation CLI—record domain compliance,
reference and calibration checks, sensor drift, invalid-input, `Indeterminate`,
`DataQualityInsufficient`, exclusion rates, reference uncertainty, software SHA,
trust-store hash/root, and approval ID at the `F-OD-32/33` cadence. Thresholds
come only from those decisions. A breach creates an incident record and the
section-28 action. Network services, databases, instrument control, and real-time
acquisition remain outside Phase F.

## 30. Known P2 disposition

| ID | Evidence/risk | Gate |
|---|---|---|
| F-MAINT-01 | `P2-ARCH-001`: non-UTF-8 early-return `DIR*` resource leak in shared/macOS publication directory handling; resource loss on a malformed-name error path, not known scientific-value corruption | MUST close and independently regression-test before F5 release. F3/F4 may proceed only in an isolated campaign environment with owner-signed temporary P2 disposition; no public claim. |
| F-MAINT-02 | `P2-COMPAT-E-T25`: permanent Phase-D golden coverage is weaker than the independent 14/14 output reproduction; evidence weakness, not a demonstrated byte regression | MUST close by permanent 14/14-equivalent inventory/reproduction coverage before F5 release. F3/F4 may proceed with owner-signed temporary P2 disposition; no public claim. |

## 31. Affected files/APIs/data structures

| Candidate | Planned change / impact / tests | May remain unchanged? |
|---|---|---|
| `config/mhi_physical_approval_trust_store.schema1.json` | F3 changes state to `PROVISIONED` and adds only reviewed public root(s), canonical order; schema/API unchanged; embedded runtime path | No for physical release |
| `src/validation_config.rs` | No new enum/field/formula | Yes |
| `src/results/mhi_validation.rs` | No dataset/report schema change; external registry convention only | Yes |
| `src/mhi_validation/protocol.rs` | No schema/reader change; Phase-F protocol tests exercise current contract | Yes |
| `src/mhi_validation/reader.rs` | No route change; existing strict/hash/no-follow path | Yes |
| `src/mhi_validation/partition.rs` | No separation change; physical campaign must satisfy current closure | Yes |
| `src/mhi_validation/evaluation.rs`, `statistics.rs` | No scientific/statistical change | Yes |
| `src/mhi_validation/approval.rs` | V1 expected unchanged; tests verify production root isolation and lifecycle procedure | Yes unless F-OD-30/31 forces separately planned V2 |
| `src/mhi_validation/output.rs` | No publication change except separate F-MAINT-01 repair cycle before F5 | Yes during F-IMPL-1/3/4 |
| `src/runners/mhi_validation.rs`, `src/cli.rs`, `src/main.rs` | Reuse `validation run`; no public API/CLI change | Yes |
| `src/domain/artifact.rs`, `src/domain/lineage.rs` | No artifact kind/schema or lineage change | Yes |
| `src/mhi_validation/output.rs` maintenance path | F-MAINT-01 closes the `read_child_names` non-UTF-8 early-return `DIR*` leak without changing output bytes/state semantics | No for F-MAINT-01 only |
| `tests/phase_e_validation.rs` | Update the production-unprovisioned assertion only in the provisioning cycle; retain explicit unprovisioned fail-closed fixtures and all Phase-E contracts | No in F-IMPL-3 |
| Future `tests/phase_f_validation.rs` | Add provisioning isolation, immutable external-package convention, full current production route, boundaries/mutations, operational state tests | New test file expected |
| `tests/fixtures/phase_e/**` | Historical/KAT bytes remain unchanged and test-only | Yes |
| Future `tests/fixtures/phase_f/**` | Software-only synthetic/KAT fixtures only; never real evidence or production private material | New fixtures expected |
| Documentation/release evidence | Separate cycle documents and external F-EV records after plan approval | New/changed only within each approved cycle |

Implementation shape: one maintenance-only production-source repair for
`F-MAINT-01`, zero scientific validation production-source changes, one
production public trust-store change, no new schema, no new CLI route, no
extension to the current route; optional offline signer remains outside
production and blocked by `F-OD-29`. The F3 provisioning cycle itself can be a
trust-store-only production change plus tests/evidence.

## 32. Compatibility

Phases A–E, existing commands, artifact kinds, schema readers, historical 48
fixtures, Phase-D golden bytes, 9-file Phase-E bundle, software-only validation,
and `UNPROVISIONED` fail-closed behavior prior to deliberate provisioning remain
authoritative. Provisioning does not change software-only results. Existing
unprovisioned negative tests use test-controlled store bytes, not the production
store. Future versions remain rejected. Phase-B/C outputs are never changed to
improve Phase-F metrics. macOS remains the only gating platform; Linux and
Windows receive no claim.

## 33. Requirement catalog

| ID | Requirement | Evidence basis |
|---|---|---|
| F-R01 | Execute F0–F5 in order; a failed stage blocks later authority. | stage model |
| F-R02 | Close all outcome-changing owner decisions before their blocked stage. | owner-decision register |
| F-R03 | Register protocol before holdout scoring and lock cohort before approval. | protocol/approval contracts |
| F-R04 | Use only exact reviewed macOS code, inputs, trust, approval, and route. | runner/path |
| F-R05 | Reconstruct and independently review every report before release. | output authorization |
| F-R06 | Emit only the four closed outcomes and exact bounded wording. | existing enums |
| F-R07 | Keep physical fixtures/evidence and software tests separate. | Phase-E fixtures |
| F-R08 | Make every material byte immutable or content-addressed. | strict hashes/registry |
| F-R09 | Make all stage/review decisions machine-readable `GO`/`NO-GO`. | R6 workflow |
| F-R10 | Permit no authority/scoring/publication fallback. | opaque capabilities/runner |
| F-R11 | Preserve macOS-only MHI V1 support. | Phase-E R6 |
| F-R12 | Complete four separately reviewed implementation/campaign cycles. | decomposition |
| F-DATA-01 | Maintain a complete external package manifest with exact metadata. | gap analysis |
| F-DATA-02 | Hash and length-bind every raw/reference/document/artifact file. | package contract |
| F-DATA-03 | Record chain of custody from acquisition through release. | external authority need |
| F-DATA-04 | Use real physical origin only for physical requests. | `EvidenceOriginV1` |
| F-DATA-05 | Lock protocol/dataset/manifests/domain/limitations into registry record. | cohort lock |
| F-DATA-06 | Assign stable sample/sensor/batch/campaign/unit identities. | duplicate/leakage gap |
| F-DATA-07 | Treat mutation/replacement as a new cohort and approval. | hash authority |
| F-DATA-08 | Keep large raw evidence external and immutable under F-OD-35. | repository boundary |
| F-DATA-09 | Inventory fixtures, KATs, mutations, real evidence, authority, approval, and release separately. | evidence ceiling |
| F-REF-01 | Bind every endpoint to exact source/method/version/authority. | existing schema |
| F-REF-02 | Require physical, complete-dependency reference sources. | partition gate |
| F-REF-03 | Require blinded physical references. | protocol physical rule |
| F-REF-04 | Bind method/QC/COC documents by URI+SHA. | external authority |
| F-REF-05 | Apply the mechanism category ceiling in section 16. | ADR/repository caveats |
| F-REF-06 | Prohibit Phase-C/self-derived health references. | independence principle |
| F-REF-07 | Encode all shared signal/artifact/model dependencies. | lineage closure |
| F-REF-08 | Preserve unavailable/ambiguous references without coercion. | existing outcomes |
| F-REF-09 | Use protocol-declared methods; add no repository registry for first release. | gap analysis |
| F-STAT-01 | Preserve exact mechanism accounting equations. | evaluator |
| F-STAT-02 | Preserve exact health accounting/coverage separation. | evaluator |
| F-STAT-03 | Preserve exact Wilson formula/order/bounds/serialization. | statistics |
| F-STAT-04 | Use exact unit/measure comparison with inclusive maximum. | partition |
| F-STAT-05 | Freeze deterministic cluster-aware power method before F1 exit. | repository has no power authority |
| F-STAT-06 | Enforce endpoint, class, family, and stratum minima prospectively. | protocol/minima |
| F-STAT-07 | Make underpowered/empty/unavailable views indeterminate. | evaluator |
| F-STAT-08 | Prohibit post-hoc pooling, rounding, tolerance, and subgroup removal. | claim ceiling |
| F-HOLD-01 | Freeze split unit/timing/randomization/blinding before acquisition scoring. | campaign design |
| F-HOLD-02 | Preserve known lineage for assessed physical artifacts. | reader/catalog |
| F-HOLD-03 | Freeze complete reference dependency graphs. | reference schema |
| F-HOLD-04 | Fail known overlap and preserve unknown separation. | partition/evaluator |
| F-HOLD-05 | Detect copied/renamed duplicates by hashes/semantic IDs. | strict identities |
| F-HOLD-06 | Detect duplicate physical units through manifest stable IDs. | external gap control |
| F-HOLD-07 | Count repeated measures only under F-OD-25. | pseudoreplication control |
| F-HOLD-08 | Reject assessed-derived references as independent support. | closure/claim ceiling |
| F-TRUST-01 | Provision only reviewed canonical public roots in embedded store. | approval.rs |
| F-TRUST-02 | Keep owner and registry authorities/keys globally distinct. | store validation |
| F-TRUST-03 | Verify fingerprints, authority documents, ordering, and store hash. | provisioning |
| F-TRUST-04 | Reject unprovisioned/empty/inconsistent/test-only authority. | runner/store validation |
| F-TRUST-05 | Bind protocol to one existing embedded root. | protocol/approval |
| F-TRUST-06 | Bind approval to store/root/authorities and both signatures. | approval verifier |
| F-TRUST-07 | Handle rotation only by reviewed store build and approval rerun. | V1 limit |
| F-TRUST-08 | Suspend immediately on revocation/compromise. | operations |
| F-TRUST-09 | Add V2 only through a new approved plan if lifecycle semantics demand it. | gap analysis |
| F-SEC-01 | No production private key enters repository, runtime, CI, logs, or artifacts. | private-key rules |
| F-SEC-02 | No production signer or runtime trust override exists in CLI/library. | current architecture |
| F-SEC-03 | No test root is reachable by production runtime. | cfg(test)/embedded store |
| F-SEC-04 | Use strict duplicate-free, canonical key/signature verification. | approval.rs |
| F-SEC-05 | Reject traversal, leaf/intermediate symlink, and unsafe paths. | strict readers/output |
| F-SEC-06 | Detect TOCTOU/input/output mutation and replay mismatch. | descriptor authority/output |
| F-SEC-07 | Publish atomically, durably, and without clobber fallback. | output.rs |
| F-SEC-08 | Require independent security review and prohibit self-approval. | review workflow |
| F-OPS-01 | Compose claims with every authority field in section 26. | release record |
| F-OPS-02 | Use exact F-OD-34 wording and domain; prohibit generalization. | claim ceiling |
| F-OPS-03 | Expire/review claims under F-OD-32. | validity |
| F-OPS-04 | Apply the section-28 revalidation minimums and F-OD-33 refinements. | change control |
| F-OPS-05 | Monitor the exact evidence named in section 29 externally. | system boundary |
| F-OPS-06 | Suspend/withdraw/supersede through immutable registry records. | operational authority |
| F-OPS-07 | Retain protocol/data/approval/report/release/audit under F-OD-35. | governance |
| F-OPS-08 | Preserve deterministic rerun; changed bytes produce a new identity. | report identity |
| F-OPS-09 | Record deviations without silently altering eligibility or protocol. | campaign integrity |
| F-OPS-10 | Require all five independent roles and zero unresolved P0/P1 before release. | security review |
| F-COMPAT-01 | Preserve Phases A–D behavior and Phase-D golden bytes. | compatibility tests |
| F-COMPAT-02 | Preserve all completed Phase-E behavior except deliberate store state. | Phase-E tests |
| F-COMPAT-03 | Preserve software-only validation and four outcomes. | evaluator |
| F-COMPAT-04 | Preserve existing CLI and public API route. | cli/runner |
| F-COMPAT-05 | Preserve historical schemas/fixtures and reject future versions. | readers/inventory |
| F-COMPAT-06 | Preserve macOS-only support and non-gating Linux status. | R6 platform authority |
| F-MAINT-01 | Close `P2-ARCH-001` before F5 release. | known debt |
| F-MAINT-02 | Close `P2-COMPAT-E-T25` before F5 release. | known debt |

## 34. Acceptance-criterion catalog

Each `F-ACnn` has one explicit primary requirement mapping in section 38.
“Decision value” means the signed
`F-OD` value; absence is `NO-GO`, never an inferred threshold.

| AC | Exact authority/input and condition | Expected result; failure class; path/evidence |
|---|---|---|
| F-AC01 | Stage ledger has F0–F5 in order and each predecessor `GO` | continue only in order; otherwise `NO-GO`; F-EV15 |
| F-AC02 | All referenced F-OD records exist and hashes verify | exact values used; missing/mismatch `NO-GO`; F-EV01 |
| F-AC03 | Registry order proves protocol before scoring and lock before approval | pass; otherwise `NO-GO`; F-EV02/F-EV10 |
| F-AC04 | Git/binary/input/store/approval hashes equal reviewed values on macOS | run exact path; mismatch hard error/`NO-GO`; F-T24/F-EV12 |
| F-AC05 | Report reconstruction and 9/9 manifest hashes match | publication candidate; mismatch hard error; F-T25/F-EV13 |
| F-AC06 | Outcome token is one of four and wording equals F-OD-34 | release exact result; otherwise `NO-GO`; F-T26/F-EV14 |
| F-AC07 | Inventory marks every item physical/test-only truthfully | no fixture satisfies F-EV; mismatch `NO-GO`; F-T01/F-EV11 |
| F-AC08 | Every package entry has retrievable URI, exact length and SHA | pass; mutation hard error/new identity; F-T13/F-EV03 |
| F-AC09 | Five reviews use only `GO`/`NO-GO` | unanimous GO required; other/missing `NO-GO`; F-EV15 |
| F-AC10 | Inject failure at every authority gate | no scoring/publication fallback; hard error; F-T24 |
| F-AC11 | Exact-SHA macOS suite passes; Linux result ignored | macOS gate only; macOS failure `NO-GO`; F-EV16 |
| F-AC12 | Four cycle records have exact bases/files/reviews/tags | integrate sequentially; mismatch `NO-GO`; F-EV15 |
| F-AC13 | Manifest fields/sort/uniqueness validate exactly | accepted; missing/duplicate hard error; F-T13/F-EV03 |
| F-AC14 | Recomputed SHA-256 and byte length equal each entry | accepted; any byte mutation hard error; F-T13/F-EV03 |
| F-AC15 | Custody ledger has acquisition-to-lock transitions and distinct actors | accepted; gap `NO-GO`; F-EV04 |
| F-AC16 | Every physical scoring record/source is `physical` | eligible; other origins hard error for physical request; F-T03 |
| F-AC17 | Registry fields equal protocol/cohort/manifest/domain/limitations | lock accepted; mismatch `NO-GO`; F-T14/F-EV10 |
| F-AC18 | Stable unit IDs are nonempty, unique, and map one physical object | accepted; collision `NO-GO`; F-T15/F-EV03 |
| F-AC19 | Any bound byte changes after lock | new cohort/approval/run required; reuse hard error/`NO-GO`; F-T14 |
| F-AC20 | External storage meets exact F-OD-35 retention/access/backup values | accepted; unmet value `NO-GO`; F-EV17 |
| F-AC21 | Six inventories cover all files with no duplicate role | zero orphan/alias items; mismatch `NO-GO`; F-T01/F-EV11 |
| F-AC22 | Endpoint metadata exactly equals protocol rule | eligible; mismatch exclusion/hard physical error; F-T06 |
| F-AC23 | Reference origin physical and dependency completeness complete recursively | eligible; other state hard physical error; F-T07 |
| F-AC24 | blinding=`blinded_to_assessment` for physical endpoints | eligible; other/unknown hard physical error; F-T08 |
| F-AC25 | Method/QC/COC URI/hash entries verify | eligible for review; missing/mutation `NO-GO`; F-EV05/F-EV06 |
| F-AC26 | Mechanism category is admitted by F-OD-19 and independent closure | may support exact ceiling; otherwise not assessed/limited; F-T09/F-EV07 |
| F-AC27 | Health reference dependency excludes Phase-C/self-input derivation | eligible; shared derivation hard physical error; F-T09 |
| F-AC28 | Shared dependency is present in both closures | known overlap/claim fail; omission `NO-GO`; F-T10 |
| F-AC29 | Reference outcome unavailable/ambiguous | exclusion/`Indeterminate`; never coercion; F-T11 |
| F-AC30 | Method/version is protocol-declared and doc hash matches | eligible; otherwise exclusion/hard physical error; F-T06 |
| F-AC31 | `n=s+c+u` and three numerators/denominator reconstruct exactly | pass; mismatch hard replay error; F-T16 |
| F-AC32 | Health counts and coverage denominator reconstruct exactly | pass; mismatch hard replay error; F-T17 |
| F-AC33 | Wilson vectors match exact bits/bytes through `2^53` | pass; mismatch hard error; F-T18 |
| F-AC34 | measure/unit exact and value `<=` maximum | equality eligible; next value excluded/hard physical error; F-T19 |
| F-AC35 | Signed cluster-aware power report fixes all inputs/minima before F1 exit | F1 GO; absent `NO-GO`; F-EV08 |
| F-AC36 | Counts equal all decision minima, including both health classes | pass; any one below `Indeterminate`; F-T20 |
| F-AC37 | Empty/under-minimum/unavailable required view | exact `Indeterminate`; F-T20 |
| F-AC38 | Locked inputs compared to run inputs byte-for-byte | unchanged only; post-hoc change new cohort/`NO-GO`; F-T14 |
| F-AC39 | Split record matches F-OD-23 and predates acquisition/scoring | accepted; mismatch `NO-GO`; F-EV09 |
| F-AC40 | All assessed roots have known matching lineage/catalog nodes | eligible; missing/mismatch hard error; F-T10 |
| F-AC41 | Reference graph complete, acyclic, all nodes present | eligible; missing/cycle/unknown hard physical error; F-T10 |
| F-AC42 | Closure shares artifact/hash/experiment/family | `DoesNotMeetProtocol`; unknown closure `Indeterminate`; F-T10 |
| F-AC43 | Same bytes/semantic ID appear under another path/ID | duplicate/overlap rejection; F-T15 |
| F-AC44 | Manifest physical-unit key maps to >1 record contrary to F-OD-25 | duplicate or shared family; no count inflation; F-T15 |
| F-AC45 | Repeated record grouping follows exact F-OD-25 function | family/effective count matches; mismatch `NO-GO`; F-EV08 |
| F-AC46 | Reference closure reaches assessed source | cannot support; physical path hard error/claim fail; F-T09 |
| F-AC47 | Embedded store contains only reviewed canonical public roots | provisioned store validates; extra/unknown root `NO-GO`; F-T04/F-EV11 |
| F-AC48 | Authority IDs and raw public keys are pairwise distinct | pass; equality hard error; F-T05 |
| F-AC49 | Fingerprints/docs/order/store hash recompute exactly | pass; mismatch `NO-GO`; F-EV11 |
| F-AC50 | Exercise UNPROVISIONED/empty/inconsistent/test-root cases | all reject before scoring; F-T04 |
| F-AC51 | Protocol root exists in embedded store | continue; missing root hard error; F-T05 |
| F-AC52 | Approval bindings/signatures all verify | opaque verified approval; any mutation hard error; F-T05 |
| F-AC53 | Rotation build changes store hash and uses new approval/run | accepted after review; reuse `NO-GO`; F-T29 |
| F-AC54 | Revocation event received under F-OD-31 | claim suspended within decision SLA; failure `NO-GO`; F-EV18 |
| F-AC55 | Lifecycle request exceeds V1 fields | F3 blocked pending new approved plan; F-EV15 |
| F-AC56 | Repository/CI/log/release scan finds production private material | zero matches; any match P0/`NO-GO`; F-T02 |
| F-AC57 | Public API/CLI surface contains signer or trust override | zero routes; any route P0/`NO-GO`; F-T02 |
| F-AC58 | Production build symbols/bytes cannot select test root | pass; reachable test root P0; F-T04 |
| F-AC59 | Duplicate/malformed/noncanonical/weak keys/signatures mutated | deterministic hard errors; F-T05 |
| F-AC60 | Traversal, leaf symlink, intermediate symlink inputs/outputs | deterministic unsafe-path error; F-T22 |
| F-AC61 | Mutate input/output between authority checks or replay | hard error/no committed bundle; F-T23/F-T25 |
| F-AC62 | Inject every atomic publication failure | old complete or new complete generation only; F-T23 |
| F-AC63 | Security author is sole approver or roles overlap | `NO-GO`; F-EV15 |
| F-AC64 | Claim record contains every section-26 field and exact hashes | releasable; missing/mismatch `NO-GO`; F-EV14 |
| F-AC65 | Claim wording/domain equals F-OD-01/02–06/34 | exact release; generalization `NO-GO`; F-EV14 |
| F-AC66 | Current date/review state is inside exact F-OD-32 interval | active; equality at expiry is expired; F-T28/F-EV14 |
| F-AC67 | A section-28 trigger occurs | minimum action plus F-OD-33 applied; under-response `NO-GO`; F-T29 |
| F-AC68 | Monitoring record contains every section-29 metric at decision cadence | active review; missing record suspension per F-OD-33; F-EV18 |
| F-AC69 | Suspension/withdrawal/supersession record references prior claim hash | state changes append-only; overwrite `NO-GO`; F-T28 |
| F-AC70 | Retention audit meets F-OD-35 exact durations/copies/access | pass; missing object `NO-GO`; F-EV17 |
| F-AC71 | Rerun exact immutable campaign twice | all 9 files byte-identical; mismatch hard replay error; F-T27 |
| F-AC72 | Deviation ledger is immutable and rule mapping preregistered | exact treatment; undocumented deviation `NO-GO`; F-EV04 |
| F-AC73 | Five role records unanimous GO, P0=0, P1=0, P2 disposition complete | F5 GO; otherwise NO-GO; F-EV15 |
| F-AC74 | Phase-D 73/73 and 14/14-equivalent bytes pass | compatible; failure `NO-GO`; F-T31 |
| F-AC75 | Phase-E 38 plus derived Phase-F inventory pass after deliberate store update | compatible; failure `NO-GO`; F-T30 |
| F-AC76 | Software protocol rerun before/after provisioning | `SoftwareValidatedOnly`, exact bytes; mismatch `NO-GO`; F-T03 |
| F-AC77 | CLI help/parse/API compile-fail authority unchanged | pass; route/API drift `NO-GO`; F-T24 |
| F-AC78 | Historical 48 fixtures and future-version mutations retain outcomes | pass; drift `NO-GO`; F-T30 |
| F-AC79 | macOS exact-SHA gates pass; no Linux claim text appears | pass; macOS fail/claim text `NO-GO`; F-T32 |
| F-AC80 | Non-UTF-8 directory fault test closes descriptor and suite passes | debt closed before F5; otherwise no release; F-T33 |
| F-AC81 | Permanent inventory reproduces all 14 Phase-D public outputs byte-for-byte | debt closed before F5; otherwise no release; F-T34 |
| F-AC82 | Physical request uses synthetic origin in any supporting record/reference | hard error/no physical claim; F-T03 |
| F-AC83 | Physical request uses constructed origin in any supporting record/reference | hard error/no physical claim; F-T03 |
| F-AC84 | Physical request uses unknown origin in any supporting record/reference | hard error/no physical claim; F-T03 |
| F-AC85 | Production package/store/approval matches any test-only inventory identity | P0 and hard error/`NO-GO`; F-T01,F-T04 |
| F-AC86 | Software request meets every software rule without verified physical authority | outcome exactly `SoftwareValidatedOnly`; F-T03,F-T26 |
| F-AC87 | Mechanism support consists only of fit/correlation/agreement/error evidence | cannot support physical mechanism; `NotAssessed`, limited, or claim failure per F-OD-19; F-T09 |
| F-AC88 | Mechanism reference closure shares artifact/pipeline/signal/event/fit/preprocessing/model dependency | non-independent; cannot support validated mechanism; F-T09,F-T10 |
| F-AC89 | Health reference is Phase-C output or derives from its assessed inputs | non-independent; hard physical failure/no claim; F-T09 |
| F-AC90 | Any required evidence is insufficient/incomplete/unblinded/uncertain/overlapping/unknown/underpowered | exact existing hard error, exclusion, `IND`, or `DNP`; never pass; F-T06–F-T21 |
| F-AC91 | Candidate claim differs in any section-26 authority field from approved/reported value | `NO-GO`; no physical release; F-T25,F-T26,F-EV14 |

## 35. Test catalog

Software tests prove behavior only; they do not prove real origin, acquisition,
method validity, sensor performance, mechanism truth, health performance, or
domain coverage.

| ID | Executable scope |
|---|---|
| F-T01 | Fixture/evidence inventory closure, committed/generated/physical/test-only flags, zero orphans |
| F-T02 | Private-key/signer/runtime-override/source dependency guards |
| F-T03 | software-only remains software-only; physical/synthetic/constructed/unknown origin matrix |
| F-T04 | production store state matrix: unprovisioned, empty provisioned, roots in unprovisioned, test root isolation |
| F-T05 | dual authority/key/signature/root/approval binding positive and all key/signature mutations |
| F-T06 | method/version/authority/blinding/endpoint binding positive and mismatch matrix |
| F-T07 | recursive physical reference-source origin/completeness/cycle/missing-source matrix |
| F-T08 | blinded/unblinded/unknown reference behavior |
| F-T09 | mechanism/health independent-reference and same-source derivation ceiling |
| F-T10 | lineage closure: separated, overlap, unknown, missing, cycle, legacy, incomplete |
| F-T11 | unavailable reference outcome/uncertainty and ambiguous health label behavior |
| F-T12 | target-domain and lower-inclusive/upper-exclusive temperature boundaries |
| F-T13 | external manifest parser/checker KAT: sort, duplicate, URI, SHA, length, media/schema mutation |
| F-T14 | preregistration/cohort-lock/approval ordering and post-lock mutation |
| F-T15 | copied source, duplicate sample/sensor/batch/unit, repeated-measure family counting |
| F-T16 | mechanism accounting all-support/all-contradiction/critical contradiction/zero denominator |
| F-T17 | health TP/TN/FP/FN, all-indeterminate, all-DQI, coverage/class denominators |
| F-T18 | exact Wilson registered decimal/bit/serialization vectors and invalid counts |
| F-T19 | uncertainty measure/unit/exact-maximum/next-representable-above boundaries |
| F-T20 | exact/below record/family/class/stratum minima and empty/underpowered strata |
| F-T21 | exact acceptance metric and Wilson-bound thresholds; rule-unavailable composition |
| F-T22 | malformed/duplicate-key JSON, traversal, leaf and intermediate symlinks |
| F-T23 | TOCTOU, output mutation, atomic no-clobber/overwrite/failure/residue matrix |
| F-T24 | exact CLI production route, early trust precedence, opaque capability compile-fail tests |
| F-T25 | authority-assisted report replay and mutation matrix |
| F-T26 | four claim outcomes, exact domain, physical request satisfying only software criteria |
| F-T27 | deterministic double rerun of same immutable campaign |
| F-T28 | claim active/expiry/suspension/withdrawal/supersession state KAT |
| F-T29 | rotation, compromise, method/protocol/domain/code/design change response table KAT |
| F-T30 | Phase-E 38-test plus historical fixture/future-schema compatibility inventory |
| F-T31 | Phase-D 73-test and permanent 14/14-equivalent golden-output reproduction |
| F-T32 | macOS exact-SHA commands and prohibited Linux/cross-platform release wording scan |
| F-T33 | `P2-ARCH-001` descriptor-close regression |
| F-T34 | `P2-COMPAT-E-T25` strengthened permanent coverage regression |

## 36. Physical evidence catalog

| ID | Non-software evidence and accepting authority |
|---|---|
| F-EV01 | Signed complete owner-decision register; project owner + registry |
| F-EV02 | Immutable protocol registration and ordering proof; registry |
| F-EV03 | Physical package/raw/reference manifest with hashes/lengths/identities; data authority |
| F-EV04 | Chain-of-custody and deviation ledgers; campaign custodian + independent auditor |
| F-EV05 | Reference method/version and traceability documents; metrology reviewer |
| F-EV06 | Reference calibration/QC records; metrology reviewer |
| F-EV07 | Mechanism orthogonality and health-label independence review; scientific reviewer |
| F-EV08 | Cluster-aware sample-size/power report; independent statistician |
| F-EV09 | Split/randomization/blinding record; data authority |
| F-EV10 | Immutable cohort-lock registry record; independent registry |
| F-EV11 | Public-key identity/fingerprint/custody/trust-store review; security reviewer |
| F-EV12 | Exact code/binary/macOS environment/reproducible-build record; compatibility reviewer |
| F-EV13 | Blind execution log, nine-file bundle, and replay review; validation operator + reviewer |
| F-EV14 | Exact release claim/limitations/validity record; release owner + registry |
| F-EV15 | Five independent GO/NO-GO records and P0/P1/P2 ledger; review board |
| F-EV16 | Exact-SHA validation command transcript/inventory; compatibility reviewer |
| F-EV17 | Retention/access/backup audit; data governance |
| F-EV18 | Monitoring, incident, suspension, withdrawal, and revalidation records; operations/security |

## 37. Fixture/evidence inventory

| Inventory | Path/URI and schema | Authority/properties |
|---|---|---|
| Permanent Phase-E software fixtures | `tests/fixtures/phase_e/**`; existing inventory schema 1 | committed, generated=false, physical=false, test-only=true; exact existing lengths/hashes retained |
| KAT trust/approval | `tests/fixtures/phase_e/trust/**`, `approval/**`, `src/mhi_validation/approval_kat.rs` | committed test authority only; never production |
| Mutation temporaries | per-test temporary directories | generated=true, committed=false, physical=false, test-only=true; deleted after test |
| Future Phase-F fixtures | `tests/fixtures/phase_f/**` with derived inventory | committed KATs only; no production keys or real evidence |
| Real physical evidence | F-OD-35 immutable URI namespace; F-EV03 | external, physical=true, test-only=false; never generated by coding agent |
| Method/QC documents | immutable URIs in F-EV05/06 | external, physical/documentary as classified, exact hashes |
| Authority/registry documents | URIs bound in approval and F-EV10/11 | external, documentary, immutable |
| Approval artifact | dataset-relative external package path | generated by ceremony, committed=false, test-only=false, exact hash/signatures |
| Release artifacts | immutable F-EV13/14 URIs | generated after scoring/review, committed=false unless a later plan names a public evidence record |

Each machine inventory row carries path/URI, type, schema/format, byte length,
SHA-256, purpose, F requirement/AC/test/evidence mappings, authority, and the four
boolean classifications. Placeholder fixtures cannot satisfy an F-EV item.

## 38. Traceability matrix

Columns abbreviate production path as `P` (protocol/reader), `A` (approval),
`E` (evaluation), `O` (output), `X` (external campaign/registry), and `G`
(Git/release). Each requirement has exactly one primary AC; tests/evidence may
cover multiple requirements.

| Requirement | Question | AC | File/API | Path | Test | Evidence | Artifact | Owner decision | Review | Severity | Gate |
|---|---|---|---|---|---|---|---|---|---|---|---|
| F-R01 | ordered gates | F-AC01 | plan | X/G | F-T29 | F-EV15 | stage ledger | F-OD-33 | operations | P1 | F0–F5 |
| F-R02 | decisions | F-AC02 | protocol/config | X | F-T14 | F-EV01 | decision register | F-OD-01–35 | all | P1 | F0 |
| F-R03 | ordering | F-AC03 | protocol/approval | P/A/X | F-T14 | F-EV02,F-EV10 | registry | F-OD-23 | data | P0 | F1–F3 |
| F-R04 | exact execution | F-AC04 | runner | P/A/E/O | F-T24 | F-EV12 | binary/input set | F-OD-35 | compatibility | P1 | F4 |
| F-R05 | replay | F-AC05 | output | O | F-T25 | F-EV13 | bundle | — | architecture | P1 | F4 |
| F-R06 | outcomes | F-AC06 | validation_config/evaluation | E/X | F-T26 | F-EV14 | claim | F-OD-34 | scientific | P1 | F5 |
| F-R07 | evidence split | F-AC07 | tests/fixtures | X | F-T01 | F-EV11 | inventories | — | data | P0 | F2–F5 |
| F-R08 | immutability | F-AC08 | readers/registry | P/X | F-T13 | F-EV03 | manifest | F-OD-35 | data | P0 | F2 |
| F-R09 | review vocabulary | F-AC09 | release records | G/X | — | F-EV15 | reviews | — | all | P1 | all |
| F-R10 | no fallback | F-AC10 | runner/output | P/A/E/O | F-T24 | F-EV16 | command log | — | security | P0 | F3–F4 |
| F-R11 | macOS | F-AC11 | cfg/release | G | F-T32 | F-EV16 | validation record | — | compatibility | P1 | F4–F5 |
| F-R12 | cycles | F-AC12 | workflow | G/X | — | F-EV15 | integration records | — | architecture | P1 | all |
| F-SCI-01 | synthetic ceiling | F-AC82 | dataset/partition | P/E | F-T03 | F-EV03 | dataset/manifest | — | scientific | P0 | F2/F4 |
| F-SCI-02 | constructed ceiling | F-AC83 | dataset/partition | P/E | F-T03 | F-EV03 | dataset/manifest | — | scientific | P0 | F2/F4 |
| F-SCI-03 | unknown ceiling | F-AC84 | dataset/partition | P/E | F-T03 | F-EV03 | dataset/manifest | — | scientific | P0 | F2/F4 |
| F-SCI-04 | test ceiling | F-AC85 | fixtures/config/approval | A/X | F-T01,F-T04 | F-EV11 | inventories/store | — | security | P0 | F3 |
| F-SCI-05 | software ceiling | F-AC86 | evaluation | E/O | F-T03,F-T26 | F-EV13 | report | — | scientific | P0 | F4/F5 |
| F-SCI-06 | association ceiling | F-AC87 | evaluation/external | E/X | F-T09 | F-EV07 | mechanism review | F-OD-19 | scientific | P0 | F1/F4 |
| F-SCI-07 | mechanism independence | F-AC88 | lineage/partition | E | F-T09,F-T10 | F-EV07 | dependency graph | F-OD-19,F-OD-25 | scientific | P0 | F2/F4 |
| F-SCI-08 | health independence | F-AC89 | partition/external | E/X | F-T09 | F-EV07 | label review | F-OD-08,F-OD-18 | scientific | P0 | F2/F4 |
| F-SCI-09 | no rescue | F-AC90 | protocol/partition/evaluation | P/E | F-T06–F-T21 | F-EV13 | report | F-OD-11–25 | scientific | P0 | F4 |
| F-SCI-10 | exact scope | F-AC91 | report/release | O/X | F-T25,F-T26 | F-EV14 | claim record | F-OD-01–35 | all | P0 | F5 |
| F-DATA-01 | package | F-AC13 | external manifest | X | F-T13 | F-EV03 | manifest | F-OD-35 | data | P0 | F2 |
| F-DATA-02 | bytes | F-AC14 | readers/manifest | P/X | F-T13 | F-EV03 | all inputs | F-OD-35 | data | P0 | F2 |
| F-DATA-03 | custody | F-AC15 | external ledger | X | — | F-EV04 | COC ledger | F-OD-35 | operations | P0 | F2 |
| F-DATA-04 | origin | F-AC16 | dataset/partition | P/E | F-T03 | F-EV03 | dataset | — | scientific | P0 | F2/F4 |
| F-DATA-05 | lock | F-AC17 | dataset/approval | A/X | F-T14 | F-EV10 | lock record | F-OD-01–21 | data | P0 | F2 |
| F-DATA-06 | physical IDs | F-AC18 | manifest | X | F-T15 | F-EV03 | manifest | F-OD-25 | data | P1 | F2 |
| F-DATA-07 | mutation | F-AC19 | reader/registry | P/X | F-T14 | F-EV10 | supersession | F-OD-33 | operations | P0 | F2–F5 |
| F-DATA-08 | storage | F-AC20 | external store | X | — | F-EV17 | retention audit | F-OD-35 | governance | P1 | F2/F5 |
| F-DATA-09 | inventories | F-AC21 | tests/external | X | F-T01 | F-EV11 | inventories | — | data | P1 | F2–F5 |
| F-REF-01 | binding | F-AC22 | results/protocol | P/E | F-T06 | F-EV05 | reference rows | F-OD-07,F-OD-08,F-OD-09,F-OD-10 | metrology | P0 | F1/F4 |
| F-REF-02 | origin/closure | F-AC23 | partition | E | F-T07 | F-EV07 | source graph | F-OD-07–10 | scientific | P0 | F2/F4 |
| F-REF-03 | blinding | F-AC24 | protocol/partition | P/E | F-T08 | F-EV09 | blinding record | F-OD-11 | scientific | P0 | F1/F4 |
| F-REF-04 | documents | F-AC25 | registration/registry | X | F-T13 | F-EV05,F-EV06 | method/QC docs | F-OD-26 | metrology | P0 | F1/F2 |
| F-REF-05 | mechanism ceiling | F-AC26 | evaluation/external | E/X | F-T09 | F-EV07 | orthogonality review | F-OD-19 | scientific | P0 | F1/F4 |
| F-REF-06 | health independence | F-AC27 | partition/external | E/X | F-T09 | F-EV07 | label review | F-OD-08,F-OD-18 | scientific | P0 | F2/F4 |
| F-REF-07 | dependencies | F-AC28 | lineage/partition | E | F-T10 | F-EV03 | graph | F-OD-25 | data | P0 | F2 |
| F-REF-08 | unavailable | F-AC29 | evaluation | E | F-T11 | F-EV13 | report | F-OD-22 | scientific | P1 | F4 |
| F-REF-09 | method declaration | F-AC30 | protocol | P | F-T06 | F-EV05 | protocol/docs | F-OD-07–09 | metrology | P1 | F1 |
| F-STAT-01 | mechanism math | F-AC31 | evaluation | E/O | F-T16 | F-EV13 | report | F-OD-20 | statistics | P1 | F4 |
| F-STAT-02 | health math | F-AC32 | evaluation | E/O | F-T17 | F-EV13 | report | F-OD-18,F-OD-20 | statistics | P1 | F4 |
| F-STAT-03 | Wilson | F-AC33 | statistics | E/O | F-T18 | F-EV16 | vectors/report | F-OD-21 | statistics | P1 | F4 |
| F-STAT-04 | uncertainty | F-AC34 | partition | E | F-T19 | F-EV06 | protocol/reference | F-OD-12–14 | metrology | P0 | F4 |
| F-STAT-05 | power | F-AC35 | external | X | — | F-EV08 | power report | F-OD-15–25 | statistics | P0 | F1 |
| F-STAT-06 | minima | F-AC36 | protocol/evaluation | P/E | F-T20 | F-EV08 | protocol | F-OD-15,F-OD-16,F-OD-17,F-OD-18 | statistics | P1 | F1/F4 |
| F-STAT-07 | underpower | F-AC37 | evaluation | E | F-T20 | F-EV13 | report | F-OD-22 | statistics | P1 | F4 |
| F-STAT-08 | no post-hoc | F-AC38 | registry | X | F-T14 | F-EV10 | lock/run | F-OD-20–23 | scientific | P0 | F2–F4 |
| F-HOLD-01 | split | F-AC39 | registration | X | F-T14 | F-EV09 | split record | F-OD-23 | data | P0 | F1 |
| F-HOLD-02 | assessed lineage | F-AC40 | reader/lineage | P/E | F-T10 | F-EV03 | catalog | F-OD-25 | data | P0 | F2 |
| F-HOLD-03 | reference graph | F-AC41 | partition | E | F-T07,F-T10 | F-EV03 | graph | F-OD-25 | data | P0 | F2 |
| F-HOLD-04 | separation | F-AC42 | partition/evaluation | E | F-T10 | F-EV13 | leakage table | F-OD-24,F-OD-25 | scientific | P0 | F4 |
| F-HOLD-05 | copied bytes | F-AC43 | reader/dataset | P/E | F-T15 | F-EV03 | manifest/dataset | F-OD-25 | data | P0 | F2 |
| F-HOLD-06 | physical duplicates | F-AC44 | manifest | X | F-T15 | F-EV03 | manifest | F-OD-25 | data | P0 | F2 |
| F-HOLD-07 | repeats | F-AC45 | manifest/dataset | X/E | F-T15 | F-EV08 | grouping rule | F-OD-25 | statistics | P1 | F1/F2 |
| F-HOLD-08 | derived reference | F-AC46 | partition | E | F-T09 | F-EV07 | graph | F-OD-19 | scientific | P0 | F2/F4 |
| F-TRUST-01 | public roots | F-AC47 | config/approval | A | F-T04 | F-EV11 | trust store | F-OD-27–29 | security | P0 | F3 |
| F-TRUST-02 | separation | F-AC48 | approval | A | F-T05 | F-EV11 | root | F-OD-27,F-OD-28 | security | P0 | F3 |
| F-TRUST-03 | review/hash | F-AC49 | config | A/G | — | F-EV11 | trust review | F-OD-29 | security | P0 | F3 |
| F-TRUST-04 | fail closed | F-AC50 | runner/approval | A | F-T04 | F-EV16 | test logs | — | security | P0 | F3 |
| F-TRUST-05 | root binding | F-AC51 | protocol/approval | P/A | F-T05 | F-EV11 | protocol/root | F-OD-27–29 | security | P0 | F3 |
| F-TRUST-06 | approval | F-AC52 | approval | A | F-T05 | F-EV10 | approval | F-OD-27–29 | security | P0 | F3 |
| F-TRUST-07 | rotation | F-AC53 | config/operations | A/X | F-T29 | F-EV18 | new store/approval | F-OD-30 | security | P1 | F5 |
| F-TRUST-08 | revocation | F-AC54 | operations | X | F-T29 | F-EV18 | incident record | F-OD-31 | security | P0 | F5 |
| F-TRUST-09 | V2 gate | F-AC55 | plan | G | — | F-EV15 | new plan | F-OD-30,F-OD-31 | architecture | P1 | F3 |
| F-SEC-01 | private keys | F-AC56 | repository/build | G | F-T02 | F-EV11 | scan | F-OD-29 | security | P0 | F3 |
| F-SEC-02 | signer/override | F-AC57 | cli/api | P/A | F-T02 | F-EV11 | API audit | F-OD-29 | security | P0 | F3 |
| F-SEC-03 | test isolation | F-AC58 | approval_kat/config | A | F-T04 | F-EV11 | build audit | — | security | P0 | F3 |
| F-SEC-04 | crypto strictness | F-AC59 | approval | A | F-T05 | F-EV16 | KAT | — | security | P0 | F3 |
| F-SEC-05 | paths | F-AC60 | reader/output | P/O | F-T22 | F-EV16 | test log | — | security | P0 | F4 |
| F-SEC-06 | TOCTOU/replay | F-AC61 | reader/output | P/O | F-T23,F-T25 | F-EV13 | bundle | — | security | P0 | F4 |
| F-SEC-07 | atomic output | F-AC62 | output | O | F-T23 | F-EV13 | bundle | — | architecture | P1 | F4 |
| F-SEC-08 | independent review | F-AC63 | workflow | G/X | — | F-EV15 | review records | — | security | P0 | all |
| F-OPS-01 | claim fields | F-AC64 | report/release | O/X | F-T26 | F-EV14 | claim | F-OD-34 | operations | P1 | F5 |
| F-OPS-02 | wording/domain | F-AC65 | protocol/release | P/X | F-T12,F-T26 | F-EV14 | claim | F-OD-01,F-OD-02,F-OD-03,F-OD-04,F-OD-05,F-OD-06,F-OD-34 | scientific | P0 | F5 |
| F-OPS-03 | expiry | F-AC66 | operations | X | F-T28 | F-EV14 | validity | F-OD-32 | operations | P1 | F5 |
| F-OPS-04 | revalidation | F-AC67 | operations | X | F-T29 | F-EV18 | trigger record | F-OD-33 | all | P0 | F5 |
| F-OPS-05 | monitoring | F-AC68 | external SOP | X | — | F-EV18 | monitor record | F-OD-32,F-OD-33 | operations | P1 | F5 |
| F-OPS-06 | state transitions | F-AC69 | registry | X | F-T28 | F-EV18 | status record | F-OD-31–33 | operations | P0 | F5 |
| F-OPS-07 | retention | F-AC70 | storage | X | — | F-EV17 | audit | F-OD-35 | governance | P1 | F5 |
| F-OPS-08 | rerun | F-AC71 | output | O | F-T27 | F-EV13 | bundles | — | compatibility | P1 | F4 |
| F-OPS-09 | deviations | F-AC72 | external ledger | X | — | F-EV04 | deviation ledger | F-OD-22 | scientific | P0 | F2–F4 |
| F-OPS-10 | final reviews | F-AC73 | workflow | G/X | — | F-EV15 | review package | — | all | P0 | F5 |
| F-COMPAT-01 | A–D | F-AC74 | baseline tests | G | F-T31 | F-EV16 | golden | — | compatibility | P1 | F3–F5 |
| F-COMPAT-02 | E | F-AC75 | phase_e tests | G | F-T30 | F-EV16 | suite | — | compatibility | P1 | F3–F5 |
| F-COMPAT-03 | software | F-AC76 | evaluation | E/O | F-T03 | F-EV16 | bundle | — | scientific | P1 | F3–F5 |
| F-COMPAT-04 | CLI/API | F-AC77 | cli/runner | P | F-T24 | F-EV16 | help/compile | — | architecture | P1 | F3 |
| F-COMPAT-05 | schemas | F-AC78 | readers/fixtures | P | F-T30 | F-EV16 | inventory | — | compatibility | P1 | F3 |
| F-COMPAT-06 | platform | F-AC79 | cfg/docs | G | F-T32 | F-EV16 | release record | — | compatibility | P1 | F5 |
| F-MAINT-01 | DIR leak | F-AC80 | output::read_child_names | O | F-T33 | F-EV16 | regression | — | architecture | P2→block | F5 |
| F-MAINT-02 | D coverage | F-AC81 | tests/fixtures | G | F-T34 | F-EV16 | inventory/golden | — | compatibility | P2→block | F5 |

Closure: requirements 91; acceptance criteria 91; tests 34; evidence items 18;
owner decisions 35. Unmapped requirements, ACs, tests, evidence, owner decisions,
and orphan fixtures are all 0 by the section-37 inventory and this matrix.

## 39. Counterexample matrix

All rows assume a physical request unless stated. `HE` means hard error before
claim/publication; `DNP` means `DoesNotMeetProtocol`; `IND` means
`Indeterminate`. The authority decision is unambiguous.

| Case(s) | Input/stage | Authority; record/endpoint/claim; error/limitation | Mapping |
|---|---|---|---|
| P01 | software-only valid/F4 | software authority; eligible/meets/`SoftwareValidatedOnly` | F-SCI-05,F-T03 |
| P02–P16 | valid physical origin, provisioned trust, distinct authorities/keys, dual signatures, exact bindings, allowed method/authority, blinded quantified reference, complete graph, separated holdout, exact minima, all strata/rules pass/F3–F4 | GO; eligible/meets/`PhysicallyValidated` only exact domain | F-R04,F-TRUST-01–06,F-T05–F-T21,F-EV03–13 |
| N01 | UNPROVISIONED/F3 | reject before dataset; HE/no claim | F-TRUST-04,F-T04 |
| N02 | PROVISIONED empty/F3 | invalid store; HE/no claim | F-TRUST-04,F-T04 |
| N03 | UNPROVISIONED roots/F3 | invalid store; HE/no claim | F-TRUST-04,F-T04 |
| N04 | test root in production/F3 | P0; HE/no claim | F-SEC-03,F-T04 |
| N05–N07 | synthetic/constructed/unknown/F2/F4 | reject physical authority; HE/no claim | F-SCI-01–03,F-T03 |
| N08 | physical confirmation false/F3 | invalid approval; HE/no claim | F-DATA-04,F-T05 |
| N09–N11 | approval/owner sig/registry sig missing/F3 | HE/no claim | F-TRUST-06,F-T05 |
| N12–N13 | wrong owner/registry key/F3 | HE/no claim | F-TRUST-06,F-T05 |
| N14–N15 | same authority/same key/F3 | invalid store; HE/no claim | F-TRUST-02,F-T05 |
| N16–N20 | weak/malformed/noncanonical key or malformed/noncanonical signature/F3 | deterministic crypto HE/no claim | F-SEC-04,F-T05 |
| N21–N29 | wrong store/root/protocol/cohort/claim/endpoint/reference-authority/domain/purpose/F3 | binding HE/no claim | F-TRUST-05–06,F-T05 |
| N30 | approval after scoring or no ordering proof/F3 | registry NO-GO; no scoring/release | F-R03,F-T14,F-EV10 |
| N31 | approval file hash mismatch/F3 | HE/no claim | F-TRUST-06,F-T05 |
| N32 | data mutated after lock/F2–F4 | new cohort required; HE/NO-GO | F-DATA-07,F-T14 |
| N33 | duplicate source under path/F2 | duplicate/overlap; no count; HE/IND per closure | F-HOLD-05,F-T15 |
| N34 | duplicate sample under record ID/F2 | duplicate/shared family; no count; NO-GO | F-HOLD-06,F-T15 |
| N35 | known holdout overlap/F4 | eligible accounting exposes overlap; endpoint DNP; claim DNP | F-HOLD-04,F-T10 |
| N36 | unknown separation/F4 | endpoint IND; claim IND | F-HOLD-04,F-T10 |
| N37 | incomplete graph/F2/F4 | physical authority HE; no claim | F-HOLD-03,F-T07 |
| N38 | missing reference source/F2/F4 | HE/no claim | F-REF-02,F-T07 |
| N39 | reference derived from assessed artifact/F2/F4 | non-independent; HE/no physical support | F-HOLD-08,F-T09 |
| N40–N41 | disallowed method/authority/F4 | excluded; physical authority HE/no claim | F-REF-01,F-T06 |
| N42–N43 | unblinded/unknown blinding/F4 | excluded; physical authority HE/no claim | F-REF-03,F-T08 |
| N44–N47 | unavailable uncertainty, wrong measure/unit, above max/F4 | excluded; physical authority HE/no claim | F-STAT-04,F-T19 |
| N48 | unavailable reference outcome/F4 | excluded/IND; no physical claim | F-REF-08,F-T11 |
| N49–N50 | insufficient records/families/F4 | endpoint IND; claim IND | F-STAT-06–07,F-T20 |
| N51–N52 | empty/underpowered required stratum/F4 | stratum and overall IND; claim IND | F-STAT-07,F-T20 |
| N53–N54 | missing positive/negative class/F4 | metric unavailable/required rule IND; claim IND | F-STAT-06,F-T17,F-T20 |
| N55 | critical mechanism contradiction/F4 | endpoint DNP; claim DNP | F-SCI-09,F-T16 |
| N56 | physical request meets only software criteria/F4 | no fallback; no physical claim | F-R10,F-T26 |
| N57 | domain mismatch/F1/F4 | protocol/read/binding HE or not applicable; no generalized claim | F-OPS-02,F-T12 |
| N58 | expired/revoked root/claim/F5 | suspend/expired; no active claim | F-TRUST-08,F-T28–29 |
| N59 | malformed/duplicate JSON/F1–F4 | strict HE/no claim | F-SEC-04,F-T22 |
| N60–N62 | traversal/leaf/intermediate symlink/F1–F4 | unsafe-path HE/no claim | F-SEC-05,F-T22 |
| N63 | TOCTOU mutation/F4 | HE/no committed output | F-SEC-06,F-T23 |
| N64 | report replay mismatch/F4 | authorization HE/no publication | F-R05,F-T25 |
| N65 | output mutation/F4 | verification HE/no accepted bundle | F-SEC-06,F-T23 |
| N66 | atomic publication failure/F4 | old or new complete only; nonzero exit/no new claim | F-SEC-07,F-T23 |
| B01–B02 | records exact minimum / one below | pass / IND; integer equality exact | F-STAT-06,F-T20 |
| B03–B04 | families exact minimum / one below | pass / IND; integer equality exact | F-STAT-06,F-T20 |
| B05–B06 | uncertainty exact max / next representable above | eligible / exclude-HE; no tolerance | F-STAT-04,F-T19 |
| B07–B08 | temperature at lower inclusive / upper exclusive | included / excluded from domain | F-OPS-02,F-T12 |
| B09 | metric exact threshold | comparison passes for inclusive comparator | F-STAT-08,F-T21 |
| B10 | Wilson bound exact threshold | comparison passes; no rounding | F-STAT-03,F-T21 |
| B11 | zero denominator | metric unavailable; required rule IND | F-STAT-03,F-T18 |
| B12 | all support | `s=n,c=u=0`; outcome per exact rules | F-STAT-01,F-T16 |
| B13 | all contradiction | `c=n`; critical policy DNP | F-STAT-01,F-T16 |
| B14 | all health indeterminate | evaluable 0; coverage 0; endpoint IND if required rule unavailable | F-STAT-02,F-T17 |
| B15 | all health DQI | evaluable 0; DQI separately counted; IND | F-STAT-02,F-T17 |
| B16 | exact positive/negative minima | pass only when both F-OD minima met | F-STAT-06,F-T20 |
| O01 | key rotation | new store hash, approval, run; old claim superseded/reviewed | F-TRUST-07,F-T29 |
| O02 | key compromise | immediate suspension, revocation record, new root/approval/run | F-TRUST-08,F-T29 |
| O03 | method version change | affected endpoint at minimum; full if comparability absent | F-OPS-04,F-T29 |
| O04 | protocol revision | new registration/cohort/approval/full run | F-OPS-04,F-T29 |
| O05 | domain expansion | new full cohort for added domain | F-OPS-02,F-T29 |
| O06 | code change | suspend and full rerun unless F-OD-33 exact documentary class applies; evaluator changes always full | F-OPS-04,F-T29 |
| O07 | sensor-design change | new domain/full cohort | F-OPS-04,F-T29 |
| O08 | report withdrawal | append withdrawal; deployment/citation prohibited | F-OPS-06,F-T28 |
| O09 | superseding campaign | new identities; append supersession link | F-OPS-06,F-T28 |
| O10 | identical rerun | 9/9 bytes identical | F-OPS-08,F-T27 |

## 40. TWO INDEPENDENT IMPLEMENTERS

| Material question | Closed adjudication |
|---|---|
| Physical evidence | section 9 plus F-SCI-01–04; unresolved domain facts block on F-OD-02–06 |
| Independent family/unit; repeated measures; sensor/batch reuse | exact F-OD-23–25 grouping function; no implementer choice |
| Independent reference | section 16 closure and F-OD-07–10/19/26; shared source cannot support |
| Reference units/uncertainty/missingness | F-OD-12–14/22; exact strings, inclusive comparison, no conversion/tolerance |
| Holdout overlap/unknown | known overlap DNP; unknown IND; no filename authority |
| Class/stratum/sample minima and power | F-OD-15–18/20–21 plus F-EV08; absence blocks F1 |
| Approval order/cohort mutation | immutable registry sequence; mutation creates new identity/approval/run |
| Trust lifecycle/key compromise | F-OD-29–31 and sections 22–24/28; V1 overflow blocks F3 |
| Claim expiry/revalidation/release wording | F-OD-32–34; no perpetual/default wording |
| Dataset storage | F-OD-35 external immutable store; no repository convenience copy |
| Operational versus scientific identity | timestamps excluded; exact scientific bytes/hashes included |

`MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN = 0`: where a value is not known,
the exact F-OD item blocks the stage and neither implementer may choose it.

## 41. Implementation decomposition

Four cycles are required because campaign evidence, trust custody, and blind
execution have different authorities and cannot safely share one review boundary.

| Cycle | Exact base/allowed work | Review/validation | Integration/tag/cleanup |
|---|---|---|---|
| F-IMPL-1 readiness/test support/debt | approved Phase-F plan tag; only plan-authorized docs, Phase-F KAT fixtures/tests, and F-MAINT repairs; no roots/evidence/signatures | architecture, scientific, security, compatibility; full commands | one temporary `codex/mhi-v1-f-physical-validation`; cycle approval tag named by later implementation prompt; merge then delete remote/local branch |
| F-IMPL-2 campaign and lock | integrated F-IMPL-1; external evidence/protocol/dataset/registry records only, repository changes only if plan explicitly inventories non-evidence support docs | scientific/metrology + data/operations; F-EV01–10 | no fabricated repo evidence; immutable external registry identity; branch only for any approved repo delta, then merge/delete |
| F-IMPL-3 trust provisioning | integrated F-IMPL-1 and F2 GO; config public roots, exact test updates, provisioning evidence only | security + compatibility; full commands, reproducible release build | reviewed approval tag, merge forward, verify remote, delete branch; no private key |
| F-IMPL-4 blind validation/release | integrated provisioned main, locked package and pre-scoring approval | all five roles, exact-SHA macOS validation, F-EV12–18 | validation/release records and separately authorized release tag; no long-lived branch |

One temporary Phase-F branch exists at a time; scientific logic, schemas, CLI,
Cargo, unrelated docs/tests, and Phase-E tags are non-goals unless a new approved
plan amendment explicitly authorizes them.

## 42. Validation commands

Run at F-IMPL-1 and F-IMPL-3 before commit, after commit, and independently on
the exact review SHA; run the complete set again at F-IMPL-4. Any nonzero exit is
`NO-GO` and requires a forward fix plus complete rerun.

```sh
git diff --check
cargo fmt --all --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
cargo test --locked --all
cargo test --doc --locked
cargo doc --locked --workspace --no-deps
cargo build --locked --release
cargo test --locked --test phase_e_validation
cargo test --locked --test phase_d_reporting_public_output
cargo test --locked --test phase_f_validation
```

Expected baseline inventory before Phase-F additions: Phase E 38/38, Phase D
73/73, approval unit 2/2, nested KAT 1/1, compile-fail 4/4, deterministic files
9/9, manifest 8/8, full suite 780 passes per run, 15 ignored doctests, 6 existing
rustdoc warnings, 48 historical fixtures, Clippy diagnostics 0. Future exact
counts are derived from `--list` and the fixture inventory at the review SHA,
never frozen speculatively. Named critical Phase-F tests correspond exactly to
F-T01–F-T34; independent evidence records the command, SHA, platform/toolchain,
inventory, exit status, and output hash. Linux is non-gating.

## 43. Git workflow

This plan is authored directly on `main` as exactly one new file, one forward
commit (`docs(plan): define MHI V1 Phase F physical validation`), normal push,
no force operation. Freeze after push:

```text
PHASE_F_PLAN_REVIEW_SHA=<git rev-parse HEAD>
PHASE_F_PLAN_SHA256=<shasum -a 256 plan>
PHASE_F_PLAN_GIT_BLOB=<git hash-object plan>
```

No plan tag is created during authoring. A fresh independent GO may create
`ism-mechanism-health-v1-f-plan-approved` targeting the exact review SHA. Only
then may `codex/mhi-v1-f-physical-validation` exist, one temporary branch at a
time. Each integrated cycle verifies remote preservation, deletes the temporary
branch, and restores main-only durable state. No amend/reset/rebase/squash/force
push or historical tag movement is permitted.

## 44. Independent review workflow

| Role | Expertise/scope/evidence | GO/NO-GO rule and independence |
|---|---|---|
| SCIENTIFIC / METROLOGY | ISE/wastewater domain, reference methods, uncertainty, power, claims; review F-EV01–10 and equations | GO only with no scientific validity/integrity risk and all relevant ACs verified; campaign/model author not sole reviewer |
| ARCHITECTURE / DATA AUTHORITY | schemas, identities, lineage, strict I/O, atomic output, storage; run/review F-T01,10,13–15,22–25 | GO only with zero P0/P1 and complete package/traceability |
| SECURITY / TRUST AND KEY MANAGEMENT | threat model, custody, Ed25519, root isolation/lifecycle, ceremony; review F-T02,04,05 and F-EV11 | GO only with zero key exposure/test-authority path/self-approval |
| COMPATIBILITY | exact-SHA macOS, APIs/schemas/fixtures/goldens; execute section 42 | GO only with all commands/inventories pass and no unsupported platform claim |
| OPERATIONS / DATA GOVERNANCE | registration order, COC, retention, monitoring, incident/revalidation/release | GO only with F-EV14–18 complete and executable |

Severity: P0 permits unauthorized physical claim, key exposure, evidence
fabrication/loss, or integrity bypass; P1 is any required invariant, scientific
validity, compatibility, authority, data-loss, or acceptance failure; P2 is
bounded non-blocking debt with explicit owner disposition and closure gate.
Unresolved P0/P1, scientific/integrity/trust risk, unexplained loss, failed
validation, compatibility break, or unverified AC requires `NO-GO`. The
implementation author is never sole approver.

## 45. Acceptance and failure gates

- F0 fails if any outcome-changing decision is ambiguous.
- F1 fails if protocol/design/power/split/blinding is incomplete or not registered before scoring.
- F2 fails if physical origin, lineage, custody, method/QC/uncertainty, independence, or cohort identity is unproven.
- F3 fails if custody, authority/key separation, root identity, build, signature, binding, or approval ordering is unsafe or unverifiable.
- F4 fails if any endpoint, stratum, rule, authority gate, exact input, deterministic rerun, or replay fails.
- F5 fails if wording, validity, monitoring, suspension, retention, P2 closure, or revalidation is incomplete.

No undocumented judgment converts failure to success. Phase-F completion requires
all applicable F-ACs, F-Ts, and F-EVs; unanimous five-role GO; P0=0; P1=0;
F-MAINT-01/02 closed; exact macOS validation; and a clean, main-only repository.

## 46. Owner-decision blockers

| Stage | Blocking decisions |
|---|---|
| F0 | F-OD-01–35 |
| F1 | F-OD-01–23, F-OD-25–26, F-OD-32–35 |
| F2 | F-OD-02–18, F-OD-22–26, F-OD-35 |
| F3 | F-OD-01–14, F-OD-27–31, F-OD-35 |
| F4 | F-OD-01–26, F-OD-27–31, F-OD-34–35 |
| F5 | F-OD-01–35 |

At plan authoring, resolved owner decisions = 0 and unresolved = 35. The plan
may be approved because every interface, authority, blocked stage, and prohibited
default is closed; physical campaign execution may not begin.

## 47. Final planning handoff template

```text
MHI V1 PHASE F — PLANNING-ONLY SPECIFICATION HANDOFF
repository/path/branch:
starting and final main/origin/live SHAs:
Phase-E tags and frozen plan hashes: PASS|FAIL
PHASE_F_PLAN_REVIEW_SHA:
PHASE_F_PLAN_SHA256:
PHASE_F_PLAN_GIT_BLOB:
changed files (must be exactly this plan):
F0–F5/content sections: COMPLETE|INCOMPLETE
counts: F-R=12 F-SCI=10 F-DATA=9 F-REF=9 F-STAT=8 F-HOLD=8
        F-TRUST=9 F-SEC=8 F-OPS=10 F-COMPAT=6 F-MAINT=2
        F-OD=35 F-AC=91 F-T=34 F-EV=18
unmapped requirements/ACs/tests/evidence/ODs/orphan fixtures: 0/0/0/0/0/0
internal audit defaults/disagreements/contradictions/claim bypass/test authority/
private keys/synthetic physical/same-source independence paths: 0/0/0/0/0/0/0/0
owner decisions resolved/unresolved: 0/35
expected shape: production source yes (F-MAINT-01 maintenance only);
trust-store-only provisioning cycle yes;
new schema no; offline signer undetermined by F-OD-29; new CLI route no;
multiple cycles yes
validation: diff/fmt/check/clippy/Phase-E 38/38/Phase-D 73/73
git safety flags all NO; worktree clean yes
next action: fresh independent Phase-F plan review
```

Author-side internal audit result:

```text
SCIENTIFIC_DEFAULTS_INVENTED=0
UNMAPPED_REQUIREMENTS=0
UNMAPPED_ACCEPTANCE_CRITERIA=0
UNMAPPED_TESTS=0
UNMAPPED_EVIDENCE_ITEMS=0
UNMAPPED_OWNER_DECISIONS=0
ORPHAN_FIXTURES=0
MATERIAL_IMPLEMENTER_DISAGREEMENTS_AFTER_PLAN=0
NORMATIVE_CONTRADICTIONS=0
PHYSICAL_CLAIM_BEFORE_APPROVAL_PATHS=0
TEST_AUTHORITY_TO_PRODUCTION_PATHS=0
PRIVATE_KEY_REPOSITORY_PATHS=0
SYNTHETIC_TO_PHYSICAL_CLAIM_PATHS=0
SAME_SOURCE_REFERENCE_INDEPENDENCE_PATHS=0
```

This internal audit is not an independent review and does not record `GO`.
