# Phase A1 requirement traceability

This document records the implementation-level A1 coverage on
`codex/mhi-v1-a1-lineage-evidence-adapters`.  It deliberately excludes the
Phase B mechanism assessor and Phase C health diagnosis.

| Requirement | Acceptance criterion | Implementation location / public API | Exact permanent test function | Compatibility / scientific risk |
|---|---|---|---|---|
| A1-C1 experiment scope | Single, Aggregate, deterministic aggregate identity, exact propagation, no synthetic member ID | `src/domain/lineage.rs`: `ArtifactExperimentScope`, `AggregateExperimentScopeId`, `ArtifactExperimentScope::propagate` | `a1_t01_t04_scope_identity_and_propagation_are_deterministic` | Legacy scope remains `Unknown`; aggregate artifacts cannot be point-joined |
| A1-RR-03 acquisition family | Trim/preserve-case IDs, sorted/deduplicated known sets, explicit Unknown and conservative union | `src/domain/lineage.rs`: `AcquisitionFamilyId`, `ArtifactAcquisitionFamilies`, `ResolvedAcquisitionFamilies` | `acq_family_unknown_is_distinct_and_never_dropped` | No path, timestamp, filename, or record-based family inference |
| A1-C2 lineage migration | Missing lineage is explicit `LegacyUnknown`; no fabricated ID, dependency, scope, hash, or family | `src/domain/lineage.rs`: `ArtifactLineageState`; `src/domain/artifact.rs`: writer migration envelope | `lineage_root_is_state_based_and_preserves_missing_and_cycles`; `a1_fixture_set_is_tracked_and_migrates_conservatively` | Supported historical payloads remain readable and conservative |
| A1-RR-02 resolver root | Known roots resolve without catalog self-lookup; LegacyUnknown is `Incomplete`, not `RootMissing`; missing and cycles are retained | `resolve_lineage`, `resolve_known_artifact_id` | `lineage_root_is_state_based_and_preserves_missing_and_cycles` | Unknown lineage can never become independent evidence |
| MHI-R4 evidence orthogonality | Valid/missing/not-applicable combinations and typed strength provenance validation | `src/evidence.rs`: `EvidenceRecord::validate`, `EvidenceBundleError` | `evidence_combinations_and_legacy_sources_are_conservative` | Raw values remain `NotAssessed`; no causal conclusion is assigned |
| MHI-R5 / MHI-R25 pairwise provenance | Same source, shared family, shared ancestor, Independent, and Unknown are recomputed from lineage | `classify_independence`, `EvidenceBundleBuilder` | `pairwise_independence_and_builder_use_exact_provenance` | LegacyUnknown, missing lineage, and unknown family block Independent |
| MHI-R27 durable covariance owner | One canonical pair key, exact lookup, duplicate rejection, producer source provenance, deterministic bundle hash | `EvidenceBundle`, `EvidenceBundleBuilder`, `TimescalePairUncertainty` | `labeled_covariance_and_timescale_pair_are_exact_and_durable` | No zero covariance or positional fallback is invented |
| A1-C3 labeled covariance | Square finite symmetric labeled matrix with unique exact axis lookup | `LabeledCovarianceMatrix`, `CovarianceAxis`, `CovarianceAxisId` | `labeled_covariance_and_timescale_pair_are_exact_and_durable` | Legacy positional covariance remains readable but unavailable to A1 |
| A1-RR-01 EIS axes | Complete producer mapping; CPE Q and alpha are distinct; duplicate/wrong axes fail | `EisParameterIdentity`, `labeled_eis_covariance`, `src/results/eis.rs::EisFitStatistics::labeled_parameter_covariance` | `labeled_covariance_and_timescale_pair_are_exact_and_durable`; fixture `current_labeled_covariance.json` | No element-only, display-name, or positional consumer mapping |
| MHI-R3 outer adapters | Public result artifacts are adapted to neutral records with field paths and no strength invention | `src/evidence_adapters.rs`: transient, calibration, EIS, model, estimation, signal adapters | `a1_adapter_reads_public_transient_artifact_without_inventing_strength` | Adapters do not diagnose fouling, reference failure, or mechanisms |
| MHI-R13 open identifiability | Known strings round-trip; unknown nonempty strings remain `Custom`; empty string rejects | `src/model/identifiability.rs`: `KnownIdentifiabilityRequirementKind`, `IdentifiabilityRequirementKind` | `a1_t13_open_identifiability_kinds_preserve_known_and_custom_strings` | Unknown/custom kinds stay `NotAssessed` unless a future assessor is registered |
| MHI-R19 compatibility fixtures | Legacy lineage and unlabeled covariance remain readable; current labeled and aggregate forms are tracked | `tests/fixtures/a1/*` | `a1_fixture_set_is_tracked_and_migrates_conservatively` | No historical identity or covariance labels are fabricated |

The current public writer stamps an explicit `LegacyUnknown` lineage envelope
when a typed legacy result has no authoritative A1 lineage field.  Producers
that can later establish authoritative identity can replace that state through
an explicit `ArtifactLineageState::Known` construction; reserialization alone
does not promote it.
