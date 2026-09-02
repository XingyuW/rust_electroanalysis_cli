#!/usr/bin/env python3
"""Generate the non-semantic Phase-F traceability and candidate bundle manifests.

The generator is deliberately strict: incomplete or ambiguous authority input
must fail before either derived manifest is rewritten.
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from copy import deepcopy
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
DOC_ROOT = ROOT / "docs" / "engineering_specification"
PHASE_F = DOC_ROOT / "phase_f"
ARCH = DOC_ROOT / "phase_f_physical_evidence_and_production_validation_plan.md"
SPECS = {
    "F-WIRE": PHASE_F / "phase_f_wire_and_authority_spec.md",
    "F-SCI": PHASE_F / "phase_f_scientific_validation_spec.md",
    "F-OPS": PHASE_F / "phase_f_operations_and_lifecycle_spec.md",
    "F-CNF": PHASE_F / "phase_f_conformance_and_kat_spec.md",
    "F-IMPL": PHASE_F / "phase_f_implementation_readiness_spec.md",
}
TRACE_PATH = PHASE_F / "phase_f_traceability_manifest.json"
BUNDLE_PATH = PHASE_F / "phase_f_specification_bundle_manifest.json"
R11_SOURCE = PHASE_F / "phase_f_r11_normative_source.md"
MIGRATION_LEDGER = PHASE_F / "phase_f_r11_to_r12_migration_ledger.md"
NORMATIVE_MATRIX_PATH = PHASE_F / "phase_f_r12_normative_traceability_matrix.json"
AUTHORITY_GRAPH_PATH = PHASE_F / "phase_f_r12_authority_graph.json"

REQUIRED_FILENAMES = {
    *(path.name for path in SPECS.values()),
    R11_SOURCE.name,
    MIGRATION_LEDGER.name,
    NORMATIVE_MATRIX_PATH.name,
    AUTHORITY_GRAPH_PATH.name,
    Path(__file__).name,
}
GENERATED_FILENAMES = {TRACE_PATH.name, BUNDLE_PATH.name}
ALLOWED_FILENAMES = REQUIRED_FILENAMES | GENERATED_FILENAMES

EXPECTED_ARCHITECTURE_IDS = [f"F-ARCH-{number:03d}" for number in range(1, 23)]
EXPECTED_SPEC_IDS = {
    "F-WIRE": [f"F-WIRE-{number:03d}" for number in range(1, 10)],
    "F-SCI": [f"F-SCI-{number:03d}" for number in range(1, 11)],
    "F-OPS": [f"F-OPS-{number:03d}" for number in range(1, 9)],
    "F-CNF": [f"F-CNF-{number:03d}" for number in range(1, 9)],
    "F-IMPL": [f"F-IMPL-{number:03d}" for number in range(1, 8)],
}
EXPECTED_F0_IDS = [f"F-OD-{number:02d}" for number in range(1, 21)]
EXPECTED_R11_IDS = [f"R11-{number:02d}" for number in range(1, 21)]
EXPECTED_R11_FINDINGS = [
    "F-PLAN-R11-P1-01",
    "F-PLAN-R11-P1-02",
    "F-PLAN-R11-P1-03",
    "F-PLAN-R11-P1-04",
    "F-PLAN-R11-P3-01",
]
EXPECTED_R11_SHA256 = "987bc6e06a5c43873b844f864cb1f858c6b57c40c18dd0d4ed4a4edcf32dec3d"
EXPECTED_R11_GIT_BLOB = "34ab62d094c4cb0bb31a40dc7a192ed304faf981"
EXPECTED_R11_LINE_COUNT = 6188
EXPECTED_R11_BYTE_COUNT = 653370
EXPECTED_R11_TEST_COUNT = 28
EXPECTED_R11_EVIDENCE_COUNT = 20
EXPECTED_R12_SCHEMA_COUNT = 96
R12_SCHEMA_IDS = {
    "PhaseFSpecificationBundleApprovalV1",
    "PhaseFMigratedFindingReviewV1",
    "PhaseFReviewerActorAttestationV1",
    "PhaseFReviewerBootstrapTrustRootV1",
    "PhaseFReviewerBootstrapCurrentnessProofV1",
}
EXPECTED_R12_REQUIREMENT_COUNT = 64
EXPECTED_MIGRATED_FINDINGS = {
    "F-PLAN-R11-P1-01",
    "F-PLAN-R11-P1-02",
    "F-PLAN-R11-P1-03",
    "F-PLAN-R11-P1-04",
    "F-PLAN-R11-P3-01",
}
EXPECTED_MIGRATED_FINDING_SEVERITIES = {
    "F-PLAN-R11-P1-01": 1,
    "F-PLAN-R11-P1-02": 1,
    "F-PLAN-R11-P1-03": 1,
    "F-PLAN-R11-P1-04": 1,
    "F-PLAN-R11-P3-01": 3,
}
MIGRATED_FINDING_DISPOSITIONS = {
    "OPEN",
    "PARTIALLY_CLOSED",
    "PENDING",
    "TECHNICALLY_CLOSED",
    "NON_BLOCKING_DEBT",
    "SUPERSEDED",
    "INVALIDATED",
}
MIGRATED_REVIEW_DECISIONS = {
    "GO",
    "GO_WITH_DOCUMENTED_NON_BLOCKING_DEBT",
    "NO-GO",
}
MIGRATED_REVIEW_RECORD_FIELDS = {
    "role",
    "reviewer_authority_id",
    "reviewed_target",
    "review_artifact_id",
    "decision",
    "review_sha256",
    "lifecycle",
    "independence_relation",
}
REVIEW_ROLE_ORDER = (
    "scientific_metrology",
    "architecture_data",
    "security",
    "compatibility",
    "operations_governance",
)
REVIEW_ROLES = set(REVIEW_ROLE_ORDER)
R11_REVIEW_OBJECT_KINDS = {
    "decision_bundle",
    "git_tag_message",
    "authority_enrollment",
    "registry_record",
    "registry_head",
    "registration_document",
    "validation_manifest",
    "protocol",
    "power_method_interface",
    "power_analysis",
    "package_manifest",
    "dependency_audit",
    "physical_unit_ledger",
    "identity_audit",
    "location_ledger",
    "chain_of_custody",
    "deviation_ledger",
    "metrology_policy",
    "metrology_check_result",
    "reference_source_descriptor",
    "reference_result",
    "scientific_admissibility_audit",
    "cohort_lock",
    "owner_approval",
    "execution_record",
    "release_record",
    "claim_state",
    "reinstatement_approval",
    "monitoring_policy",
    "monitoring_record",
    "incident_record",
    "monitoring_evidence",
    "retention_audit",
    "independent_review_bundle",
    "incident_resolution",
    "emergency_registry_compromise",
    "checker_build_evidence",
    "checker_readiness_evidence",
    "f5_release_candidate",
}
R11_EXTERNAL_OBJECT_KIND_BY_AUTHORITY_KIND = {
    "PhaseFDecisionBundleV1": "decision_bundle",
    "PhaseFAuthorityEnrollmentV1": "authority_enrollment",
    "PhaseFRegistryRecordV1": "registry_record",
    "PhaseFRegistryHeadV1": "registry_head",
    "PhaseFPackageManifestV1": "package_manifest",
    "PhaseFDependencyAuditV1": "dependency_audit",
    "PhaseFPhysicalUnitLedgerV1": "physical_unit_ledger",
    "PhaseFPhysicalIdentityAuditV1": "identity_audit",
    "PhaseFLocationLedgerV1": "location_ledger",
    "PhaseFChainOfCustodyV1": "chain_of_custody",
    "PhaseFDeviationLedgerRevisionV1": "deviation_ledger",
    "PhaseFPowerMethodInterfaceV1": "power_method_interface",
    "PhaseFPowerAnalysisRecordV1": "power_analysis",
    "PhaseFMetrologyPolicyV1": "metrology_policy",
    "PhaseFMetrologyCheckResultV1": "metrology_check_result",
    "PhaseFReferenceSourceDescriptorV1": "reference_source_descriptor",
    "PhaseFReferenceResultV1": "reference_result",
    "PhaseFCohortLockRecordV1": "cohort_lock",
    "PhaseFExecutionRecordV1": "execution_record",
    "PhaseFReleaseRecordV1": "release_record",
    "PhaseFClaimStateRecordV1": "claim_state",
    "PhaseFReinstatementApprovalV1": "reinstatement_approval",
    "PhaseFMonitoringPolicyV1": "monitoring_policy",
    "PhaseFMonitoringRecordV1": "monitoring_record",
    "PhaseFMonitoringEvidenceV1": "monitoring_evidence",
    "PhaseFIncidentRecordV1": "incident_record",
    "PhaseFIncidentResolutionV1": "incident_resolution",
    "PhaseFRetentionAuditV1": "retention_audit",
    "PhaseFScientificAdmissibilityAuditV1": "scientific_admissibility_audit",
    "PhaseFRegistryCompromiseEmergencyV1": "emergency_registry_compromise",
    "PhaseFCheckerBuildEvidenceV1": "checker_build_evidence",
    "PhaseFCheckerReadinessEvidenceV1": "checker_readiness_evidence",
    "PhaseFF5ReleaseCandidateV1": "f5_release_candidate",
    "PhaseFIndependentReviewBundleV1": "independent_review_bundle",
}
INDEPENDENT_REVIEW_BUNDLE_FIELDS = {
    "schema_version",
    "review_bundle_id",
    "target",
    "reviews",
    "aggregate_p0_count",
    "aggregate_p1_count",
    "aggregate_decision",
}
REVIEW_ROW_FIELDS = {
    "role",
    "decision",
    "p0_count",
    "p1_count",
    "finding_ids",
    "review_artifact_reference",
}
CANONICAL_UNSIGNED_INTEGER_PATTERN = r"0|[1-9][0-9]*"
REVIEW_ARTIFACT_URI_PREFIX = "phase-f-authority://review-artifacts/"
REVIEWER_ACTOR_ATTESTATION_URI_PREFIX = (
    "phase-f-authority://reviewer-actor-attestations/"
)
ACTOR_IDENTITY_DIGEST_DOMAIN = b"mhi_phase_f_reviewer_actor_identity_v1\0"
REVIEWER_ACTOR_ATTESTATION_DOMAIN = (
    b"mhi_phase_f_reviewer_actor_attestation_v1\0"
)
REVIEWER_BOOTSTRAP_ROOT_DOMAIN = (
    b"mhi_phase_f_reviewer_bootstrap_trust_root_v1\0"
)
REVIEWER_BOOTSTRAP_CURRENTNESS_DOMAIN = (
    b"mhi_phase_f_reviewer_bootstrap_currentness_proof_v1\0"
)
REVIEWER_BOOTSTRAP_SUBJECT_REGISTRY_DOMAIN = (
    b"mhi_phase_f_reviewer_bootstrap_subject_registry_v1\0"
)
AUTHORITY_ENROLLMENT_DOMAIN = b"mhi_phase_f_authority_enrollment_v1\0"
REVIEWER_BOOTSTRAP_STAGE = "PRE_G0_REVIEWER_BOOTSTRAP"
REVIEWER_BOOTSTRAP_SCOPE = [
    "reviewer_actor_attestation",
    "reviewer_currentness",
    "reviewer_subject_registry",
]
REVIEWER_BOOTSTRAP_TRUST_SOURCE = "reviewer_bootstrap"
REVIEWER_BOOTSTRAP_TRUST_CONTRACT_KEYS = {
    "stage",
    "root_path",
    "currentness_proof_path",
    "root_authority_kind",
    "currentness_proof_authority_kind",
    "root_id",
    "root_public_key_fingerprint",
    "allowed_purposes",
    "transition_policy",
    "currentness_window_policy",
}
REVIEWER_BOOTSTRAP_ROOT_FIELDS = {
    "root_id",
    "authority_kind",
    "schema_version",
    "authority_class",
    "stage",
    "root_public_key",
    "root_public_key_fingerprint",
    "authority_scope",
    "subject_uniqueness_policy",
    "evidence_retention_policy",
    "rotation_policy",
    "compromise_policy",
    "lifecycle",
    "stale",
    "superseded_by",
    "invalidated",
}
REVIEWER_BOOTSTRAP_CURRENTNESS_FIELDS = {
    "currentness_proof_id",
    "authority_kind",
    "schema_version",
    "authority_class",
    "stage",
    "root_id",
    "root_sha256",
    "sequence",
    "previous_proof_id",
    "head_id",
    "current_verifier_authority_id",
    "current_verifier_public_key",
    "current_verifier_public_key_fingerprint",
    "subject_registry_head_sha256",
    "subject_bindings",
    "valid_from",
    "valid_until",
    "root_lifecycle",
    "root_revoked",
    "root_compromised",
    "root_superseded_by",
    "verifier_lifecycle",
    "verifier_revoked",
    "verifier_compromised",
    "verifier_superseded_by",
    "lifecycle",
    "stale",
    "superseded_by",
    "invalidated",
    "signature",
}
REVIEWER_BOOTSTRAP_TRUST_SOURCE_FIELDS = {
    "type",
    "root_id",
    "root_sha256",
    "currentness_proof_id",
    "currentness_proof_sha256",
}
RUNTIME_STABLE_ID_PATTERN = r"[A-Za-z0-9][A-Za-z0-9._:-]*"
UTC_SECOND_TIMESTAMP_PATTERN = (
    r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z"
)
ED25519_PUBLIC_KEY_PATTERN = r"[0-9a-f]{64}"
ED25519_SIGNATURE_PATTERN = r"[0-9a-f]{128}"
REVIEW_ARTIFACT_FIELDS = {
    "review_artifact_id",
    "authority_kind",
    "schema_version",
    "authority_class",
    "reviewer_authority_id",
    "role",
    "reviewed_target",
    "decision",
    "p0_count",
    "p1_count",
    "p2_count",
    "finding_ids",
    "independence_relation",
    "lifecycle",
    "stale",
    "superseded_by",
    "invalidated",
}
REVIEW_BUNDLE_NODES = {
    "architecture_review",
    "f0_review",
    "component_wire_review",
    "component_scientific_review",
    "component_operations_review",
    "component_conformance_review",
    "component_implementation_review",
    "aggregate_review",
    "readiness_review",
}
GRAPH_EDGE_TYPES = {
    "approves",
    "binds",
    "generated_from",
    "hashes",
    "requires",
    "reviews",
    "targets",
}
SERIALIZED_BINDING_POLICIES = {"all", "selected", "none"}
SERIALIZED_BINDING_CATEGORIES = {
    "approval_review_binding",
    "generated_source_binding",
    "prerequisite_digest_binding",
    "review_target_binding",
    "serialized_digest_binding",
    "serialized_identity_binding",
}
SERIALIZED_BINDING_VALUE_SOURCES = {
    "authority_descriptor",
    "source_sha256",
    "review_target",
}
SERIALIZED_BINDING_FIELD_SEMANTICS = {
    "authority_bindings": ("serialized_identity_binding", "authority_descriptor"),
    "generated_source_sha256s": ("generated_source_binding", "source_sha256"),
    "bound_authority_sha256s": ("serialized_digest_binding", "source_sha256"),
    "review_sha256": ("approval_review_binding", "source_sha256"),
    "target_sha256": ("review_target_binding", "source_sha256"),
    "target": ("review_target_binding", "review_target"),
    "target_bundle_inputs_sha256": ("review_target_binding", "source_sha256"),
    "reviewed_migration_ledger_sha256": ("serialized_digest_binding", "source_sha256"),
    "reviewed_normative_traceability_matrix_sha256": (
        "serialized_digest_binding",
        "source_sha256",
    ),
    "reviewed_traceability_manifest_sha256": ("serialized_digest_binding", "source_sha256"),
    "bundle_input_fingerprint_sha256": ("prerequisite_digest_binding", "source_sha256"),
    "target_bundle_manifest_sha256": ("review_target_binding", "source_sha256"),
}
SERIALIZED_BINDING_CARDINALITIES = {
    "authority_bindings": "one_per_source",
    "generated_source_sha256s": "one_per_source",
    "bound_authority_sha256s": "one_per_source",
    "target_sha256": "exactly_one",
    "target": "exactly_one",
    "review_sha256": "exactly_one",
    "target_bundle_inputs_sha256": "exactly_one",
    "reviewed_migration_ledger_sha256": "exactly_one",
    "reviewed_normative_traceability_matrix_sha256": "exactly_one",
    "reviewed_traceability_manifest_sha256": "exactly_one",
    "bundle_input_fingerprint_sha256": "exactly_one",
    "target_bundle_manifest_sha256": "exactly_one",
}
BINDING_ROOT_CONTRACT = {
    "root": "edges[].binding_obligation",
    "edge_contract": "edge_contract",
    "node_binding_fields": "DERIVED_NON_NORMATIVE",
    "binding_semantics": "DERIVED_NON_NORMATIVE",
    "serialized_binding_fields": "DERIVED_NON_NORMATIVE",
    "relation_policies": "DERIVED_NON_NORMATIVE",
}
GRAPH_STAGE_NAMES = {
    0: "architecture",
    1: "architecture_review",
    2: "architecture_approval",
    3: "f0_bundle",
    4: "f0_review",
    5: "f0_approval",
    6: "specification_inputs",
    7: "component_review",
    8: "derived_traceability",
    9: "bundle_inputs",
    10: "migrated_review",
    11: "bundle_manifest",
    12: "aggregate_review",
    13: "g3",
    14: "readiness_specification",
    15: "readiness_review",
    16: "readiness_approval",
    17: "implementation_gate",
}
EXPECTED_GRAPH_NODE_IDS = {
    "architecture_plan",
    "architecture_review",
    "architecture_approval",
    "f0_decision_bundle",
    "f0_review",
    "f0_approval",
    "component_wire_spec",
    "component_scientific_spec",
    "component_operations_spec",
    "component_conformance_spec",
    "component_implementation_spec",
    "component_wire_review",
    "component_scientific_review",
    "component_operations_review",
    "component_conformance_review",
    "component_implementation_review",
    "normative_traceability_matrix",
    "migration_ledger",
    "generated_traceability_manifest",
    "specification_bundle_inputs",
    "migrated_finding_review",
    "specification_bundle_manifest",
    "aggregate_review",
    "g3_approval_tag",
    "implementation_readiness_specification",
    "readiness_review",
    "readiness_approval",
    "phase_f_implementation_gate",
}
EXPECTED_IDENTITY_CYCLE_RULES = {
    "self_file",
    "self_registry",
    "self_git_commit",
    "self_release_record",
    "self_review_object",
    "self_bundle",
}
R12_G3_TEST_IDS = [
    "R12-G3-AUTHORITY-CONTEXT-POS",
    "R12-G3-ARCHITECTURE-REVIEW-BUNDLE-POSITIVE",
    "R12-G3-ARCHITECTURE-REVIEW-NEGATIVE-MATRIX",
    "R12-G3-REVIEW-START-GIT-PUBLISHED",
    "R12-G3-REVIEW-START-GIT-MISMATCH",
    "R12-G3-MISSING-ARCH-APPROVAL",
    "R12-G3-STALE-ARCH-APPROVAL",
    "R12-G3-MISSING-F0-APPROVAL",
    "R12-G3-WRONG-F0-TARGET",
    "R12-G3-MISSING-COMPONENT-REVIEW",
    "R12-G3-STALE-COMPONENT-REVIEW",
    "R12-G3-MISSING-MIGRATED-REVIEW",
    "R12-G3-MIGRATED-WRONG-BUNDLE",
    "R12-G3-MIGRATED-WRONG-LEDGER",
    "R12-G3-MIGRATED-WRONG-COMMIT",
    "R12-G3-MIGRATED-HASH-MISMATCH",
    "R12-G3-MIGRATED-INCOMPLETE-DISPOSITION",
    "R12-G3-MIGRATED-STALE",
    "R12-G3-MIGRATED-SUPERSEDED",
    "R12-G3-MIGRATED-NON-INDEPENDENT",
    "R12-G3-MISSING-AGGREGATE",
    "R12-G3-AGGREGATE-WRONG-BUNDLE",
    "R12-G3-AGGREGATE-HASH-MISMATCH",
    "R12-G3-MANIFEST-HASH-MISMATCH",
    "R12-G3-MANIFEST-CHANGED",
    "R12-G3-WRONG-COMMIT",
    "R12-G3-LIGHTWEIGHT-TAG",
    "R12-G3-MISSING-REAL-PREREQUISITES",
    "R12-G3-SYNTHETIC-CANNOT-AUTHORIZE-REAL",
    "R12-G3-MIGRATED-DISPOSITION-ENUM",
    "R12-G3-MIGRATED-REVIEW-RECORDS",
    "R12-G3-MIGRATED-INPUT-FINGERPRINT",
    "R12-G3-REAL-FORMAT-POSITIVE",
    "R12-G3-REAL-FORMAT-NEGATIVE-MATRIX",
    "R12-G3-REAL-ACTOR-ATTESTATION-POSITIVE",
    "R12-G3-REAL-ACTOR-ATTESTATION-NEGATIVE-MATRIX",
    "R12-G3-MIGRATED-PENDING-P1",
    "R12-G3-MIGRATED-OPEN-P1",
    "R12-G3-MIGRATED-PARTIALLY-CLOSED-P1",
    "R12-G3-MIGRATED-COUNT-MISMATCH",
    "R12-G3-MIGRATED-MISSING-FINDING",
    "R12-G3-MIGRATED-DUPLICATE-FINDING",
    "R12-G3-MIGRATED-UNKNOWN-FINDING",
    "R12-G3-MIGRATED-MISSING-ROLE",
    "R12-G3-MIGRATED-DUPLICATE-ROLE",
    "R12-G3-MIGRATED-INDEPENDENCE",
    "R12-G3-MIGRATED-WRONG-REVIEW-TARGET",
    "R12-G3-MIGRATED-WRONG-REVIEW-HASH",
    "R12-G3-MIGRATED-WRONG-BUNDLE-FINGERPRINT",
    "R12-G3-MIGRATED-ARCH-APPROVAL-ID",
    "R12-G3-MIGRATED-ARCH-APPROVAL-DIGEST",
    "R12-G3-MIGRATED-ARCH-APPROVAL-TARGET",
    "R12-G3-MIGRATED-F0-APPROVAL-ID",
    "R12-G3-MIGRATED-F0-APPROVAL-DIGEST",
    "R12-G3-MIGRATED-F0-APPROVAL-TARGET",
    "R12-G3-MIGRATED-COMPONENT-SPEC-HASH",
    "R12-G3-MIGRATED-COMPONENT-REVIEW-ID",
    "R12-G3-MIGRATED-LEDGER-HASH",
    "R12-G3-MIGRATED-NORMATIVE-HASH",
    "R12-G3-MIGRATED-TRACEABILITY-HASH",
    "R12-G3-MIGRATED-TARGET-COMMIT",
]
R12_TRACE_TEST_IDS = [
    "R12-TRACE-SEMANTIC-SUBSTITUTION",
    "R12-TRACE-WRONG-KAT",
    "R12-TRACE-WRONG-EVIDENCE",
    "R12-TRACE-WRONG-AUDIT",
    "R12-TRACE-WRONG-CATEGORY",
    "R12-TRACE-CROSS-REQUIREMENT",
    "R12-TRACE-EXTRA-MAPPING",
    "R12-TRACE-MISSING-MAPPING",
    "R12-TRACE-SCHEMA-INVERSE",
]
R12_DAG_TEST_IDS = [
    "R12-DAG-VALID",
    "R12-DAG-UNKNOWN-NODE",
    "R12-DAG-UNKNOWN-EDGE",
    "R12-DAG-DUPLICATE-NODE",
    "R12-DAG-SELF-EDGE",
    "R12-DAG-PREREQUISITE-CYCLE",
    "R12-DAG-HASH-CYCLE",
    "R12-DAG-FUTURE-OBJECT",
    "R12-DAG-G3-BYPASS",
    "R12-DAG-IMPLEMENTATION-BYPASS",
    "R12-DAG-REVIEW-CYCLE",
    "R12-DAG-SELF-GIT",
    "R12-DAG-ALTERNATIVE-BYPASS",
    "R12-DAG-G3-BEFORE-AGGREGATE",
    "R12-DAG-TYPED-EDGE-CONTRACT",
    "R12-DAG-IDENTITY-RULE-CONTRACT",
    "R12-DAG-SEMANTIC-AUDITS",
    "R12-DAG-BINDING-EQUALITY",
]
EXPECTED_R12_TEST_CATALOG_IDS = {
    "R12-POS-SPEC-BUNDLE-TAG",
    *R12_G3_TEST_IDS,
    *R12_TRACE_TEST_IDS,
    *R12_DAG_TEST_IDS,
}

G3_TAG_NAME = "ism-mechanism-health-v1-f-specification-bundle-approved"
G3_TAG_FIELDS = (
    "phase_f_architecture_plan_tag",
    "phase_f_f0_decisions_tag",
    "specification_bundle_manifest_sha256",
    "aggregate_review_bundle_sha256",
    "approval_decision",
    "schema_version",
)
G3_EXPECTED_FIELDS = {
    "phase_f_architecture_plan_tag": "ism-mechanism-health-v1-f-plan-approved",
    "phase_f_f0_decisions_tag": "ism-mechanism-health-v1-f-f0-decisions-approved",
    "specification_bundle_manifest_sha256": "0" * 64,
    "aggregate_review_bundle_sha256": "1" * 64,
    "approval_decision": "GO",
    "schema_version": "1",
}
G3_FIXTURE_BODY = (
    b"phase_f_architecture_plan_tag=ism-mechanism-health-v1-f-plan-approved\n"
    b"phase_f_f0_decisions_tag=ism-mechanism-health-v1-f-f0-decisions-approved\n"
    b"specification_bundle_manifest_sha256=" + b"0" * 64 + b"\n"
    b"aggregate_review_bundle_sha256=" + b"1" * 64 + b"\n"
    b"approval_decision=GO\n"
    b"schema_version=1\n"
)
G3_FIXTURE_BYTE_LENGTH = 379
G3_FIXTURE_SHA256 = "af3f94a1a5ae85f2e62d8a0ad54e66b3bd985cd150805a5750528befa15027b6"
G3_LEGACY_FIELDS = {"architecture_plan_tag", "f0_decisions_tag"}
AUTHORITY_ENROLLMENT_APPROVAL_TAG = (
    "ism-mechanism-health-v1-f-authority-enrollment-approved"
)
AUTHORITY_ENROLLMENT_APPROVAL_FIELDS = (
    "phase_f_plan_tag",
    "f0_decisions_tag",
    "readiness_tag",
    "readiness_main_sha",
    "enrollment_sha256",
    "owner_authority_id",
    "registry_authority_id",
    "owner_public_key_fingerprint",
    "registry_public_key_fingerprint",
    "review_bundle_sha256",
    "approval_decision",
)

G3_KAT_MUTATIONS = (
    {
        "id": "R12-NEG-G3-WRONG-FIELD-NAME",
        "operation": "replace first key phase_f_architecture_plan_tag with phase_f_architecture_plan",
        "expected_category": "unknown_field",
    },
    {
        "id": "R12-NEG-G3-LEGACY-FIELD-NAME",
        "operation": "replace first key with legacy unprefixed architecture_plan_tag",
        "expected_category": "legacy_field_name",
    },
    {
        "id": "R12-NEG-G3-MISSING-REQUIRED-FIELD",
        "operation": "remove the complete aggregate_review_bundle_sha256 line and its LF",
        "expected_category": "missing_required_field",
    },
    {
        "id": "R12-NEG-G3-DUPLICATE-FIELD",
        "operation": "insert a second approval_decision=GO line immediately before schema_version=1",
        "expected_category": "duplicate_field",
    },
    {
        "id": "R12-NEG-G3-UNEXPECTED-FIELD",
        "operation": "replace the final schema_version=1 line with unexpected_field=x",
        "expected_category": "unexpected_field",
    },
    {
        "id": "R12-NEG-G3-WRONG-LINE-ORDER",
        "operation": "swap the first and second complete lines",
        "expected_category": "wrong_field_order",
    },
    {
        "id": "R12-NEG-G3-SCHEMA-VERSION",
        "operation": "replace schema_version=1 with schema_version=2",
        "expected_category": "invalid_schema_version",
    },
    {
        "id": "R12-NEG-G3-MALFORMED-TAG-NAME",
        "operation": "replace the input tag name with the deterministic malformed name",
        "expected_category": "invalid_tag_name",
    },
    {
        "id": "R12-NEG-G3-WRONG-ARCHITECTURE-BINDING",
        "operation": "replace the architecture-plan tag value with the F0 tag value",
        "expected_category": "wrong_architecture_plan_binding",
    },
    {
        "id": "R12-NEG-G3-WRONG-F0-BINDING",
        "operation": "replace the F0 tag value with the architecture-plan tag value",
        "expected_category": "wrong_f0_decisions_binding",
    },
    {
        "id": "R12-NEG-G3-WRONG-BUNDLE-HASH",
        "operation": "replace the first manifest-hash zero with ASCII a",
        "expected_category": "wrong_bundle_hash",
    },
    {
        "id": "R12-NEG-G3-MALFORMED-SHA",
        "operation": "replace the first aggregate-hash one with ASCII z",
        "expected_category": "malformed_sha256",
    },
    {
        "id": "R12-NEG-G3-TRAILING-WHITESPACE",
        "operation": "replace approval_decision=GO with approval_decision=GO plus one space",
        "expected_category": "trailing_whitespace",
    },
    {
        "id": "R12-NEG-G3-MISSING-DELIMITER",
        "operation": "replace the first equals delimiter with one ASCII space",
        "expected_category": "missing_delimiter",
    },
    {
        "id": "R12-NEG-G3-INVALID-NEWLINE",
        "operation": "replace the first LF with CRLF",
        "expected_category": "invalid_newline",
    },
    {
        "id": "R12-NEG-G3-EXTRA-TRAILING-CONTENT",
        "operation": "append trailing plus LF after the required final LF",
        "expected_category": "extra_trailing_content",
    },
    {
        "id": "R12-NEG-G3-TRUNCATED-CONTENT",
        "operation": "remove the final ten bytes, producing a partial final field",
        "expected_category": "truncated_content",
    },
    {
        "id": "R12-NEG-G3-MISSING-FINAL-NEWLINE",
        "operation": "remove exactly the required final LF byte",
        "expected_category": "missing_final_newline",
    },
    {
        "id": "R12-NEG-G3-WRONG-APPROVAL-VALUE",
        "operation": "replace approval_decision=GO with approval_decision=NO-GO",
        "expected_category": "invalid_approval_decision",
    },
)


class G3ValidationError(ValueError):
    """A deterministic failure category for the G3 authority validator."""

    def __init__(self, category: str):
        self.category = category
        super().__init__(category)


@dataclass
class G3AuthorityContext:
    """The common prerequisite interface for synthetic and real G3 checks."""

    mode: str
    graph: dict[str, Any]
    objects: dict[str, dict[str, Any]]
    bundle_manifest_sha256: str | None
    aggregate_review_sha256: str | None
    expected_target_commit: str
    tag: dict[str, Any]
    component_sha256s: list[str]
    component_sha_by_node: dict[str, str]
    architecture_plan_sha256: str
    f0_decisions_sha256: str | None
    authority_graph_sha256: str
    authority_graph_bytes: bytes
    real_authority_requested: bool = False
    reviewer_authorities: dict[str, dict[str, Any]] = field(default_factory=dict)
    review_artifacts: dict[str, dict[str, Any]] = field(default_factory=dict)
    authority_enrollment: dict[str, Any] | None = None
    reviewer_bootstrap_root: dict[str, Any] | None = None
    reviewer_bootstrap_currentness: dict[str, Any] | None = None
    reviewer_actor_attestations: dict[str, dict[str, Any]] = field(default_factory=dict)
    remediation_authority_id: str | None = None
    remediation_actor_identity_digest: str | None = None
    allow_test_only_authority: bool = False
    resolution: dict[str, Any] = field(default_factory=dict)


def canonical_json_bytes(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode(
        "utf-8"
    )


def canonical_jcs_bytes(value: object) -> bytes:
    """Return the JCS payload bytes used by the inherited R11 semantic ID rule."""

    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def reviewer_actor_identity_digest(actor_subject_id: str) -> str:
    """Derive the stable reviewer identity from the authority-issued subject."""

    return sha256_bytes(
        ACTOR_IDENTITY_DIGEST_DOMAIN
        + canonical_jcs_bytes({"actor_subject_id": actor_subject_id})
    )


def reviewer_actor_attestation_id(attestation: dict[str, Any]) -> str:
    payload = {
        key: value
        for key, value in attestation.items()
        if key not in {"attestation_id", "signature"}
    }
    return "sha256:" + sha256_bytes(
        REVIEWER_ACTOR_ATTESTATION_DOMAIN + canonical_jcs_bytes(payload)
    )


def _reviewer_bootstrap_wire_object(value: dict[str, Any]) -> dict[str, Any]:
    canonical_object = value.get("canonical_object")
    return canonical_object if isinstance(canonical_object, dict) else value


def reviewer_bootstrap_root_id(root: dict[str, Any]) -> str:
    wire_object = _reviewer_bootstrap_wire_object(root)
    payload = {key: value for key, value in wire_object.items() if key != "root_id"}
    return "sha256:" + sha256_bytes(
        REVIEWER_BOOTSTRAP_ROOT_DOMAIN + canonical_jcs_bytes(payload)
    )


def reviewer_bootstrap_subject_registry_head_sha256(
    sequence: int, subject_bindings: list[dict[str, Any]]
) -> str:
    return sha256_bytes(
        REVIEWER_BOOTSTRAP_SUBJECT_REGISTRY_DOMAIN
        + canonical_jcs_bytes(
            {"sequence": sequence, "subject_bindings": subject_bindings}
        )
    )


def reviewer_bootstrap_currentness_head_id(proof: dict[str, Any]) -> str:
    wire_object = _reviewer_bootstrap_wire_object(proof)
    payload = {
        key: value
        for key, value in wire_object.items()
        if key not in {"currentness_proof_id", "head_id", "signature"}
    }
    return "sha256:" + sha256_bytes(
        REVIEWER_BOOTSTRAP_CURRENTNESS_DOMAIN + canonical_jcs_bytes(payload)
    )


def reviewer_bootstrap_currentness_proof_id(proof: dict[str, Any]) -> str:
    wire_object = _reviewer_bootstrap_wire_object(proof)
    payload = {
        key: value
        for key, value in wire_object.items()
        if key not in {"currentness_proof_id", "signature"}
    }
    return "sha256:" + sha256_bytes(
        REVIEWER_BOOTSTRAP_CURRENTNESS_DOMAIN + canonical_jcs_bytes(payload)
    )


def authority_enrollment_id(enrollment: dict[str, Any]) -> str:
    payload = {
        key: value for key, value in enrollment.items() if key != "enrollment_id"
    }
    return "sha256:" + sha256_bytes(
        AUTHORITY_ENROLLMENT_DOMAIN + canonical_jcs_bytes(payload)
    )


def independent_review_bundle_id(bundle: dict[str, Any]) -> str:
    payload = {
        key: value
        for key, value in bundle.items()
        if key != "review_bundle_id"
    }
    return "sha256:" + sha256_bytes(
        b"mhi_phase_f_review_bundle_v1\0" + canonical_jcs_bytes(payload)
    )


def _is_independent_review_bundle(record: dict[str, Any]) -> bool:
    return record.get("authority_kind") == "PhaseFIndependentReviewBundleV1"


def _review_bundle_scope_sha(
    context: G3AuthorityContext, node_id: str
) -> str | None:
    edges = _graph_edges(context.graph, _graph_nodes(context.graph))
    sources = [
        edge["from"]
        for edge in edges
        if edge["to"] == node_id and edge["type"] == "reviews"
    ]
    if len(sources) != 1:
        return None
    source = context.objects.get(sources[0])
    value = source.get("sha256") if isinstance(source, dict) else None
    return value if isinstance(value, str) else None


def _review_target_for_source(
    source_authority_kind: str, target_commit: str, source_sha256: str | None
) -> dict[str, str]:
    object_kind = R11_EXTERNAL_OBJECT_KIND_BY_AUTHORITY_KIND.get(
        source_authority_kind
    )
    if object_kind is None:
        return {"type": "git_commit", "git_sha": target_commit}
    if not isinstance(source_sha256, str) or not re.fullmatch(
        r"[0-9a-f]{64}", source_sha256
    ):
        raise G3ValidationError("review_target_source_digest_malformed")
    return {
        "type": "external_object",
        "object_kind": object_kind,
        "object_sha256": source_sha256,
    }


def _review_bundle_target(
    context: G3AuthorityContext, node_id: str
) -> dict[str, str]:
    nodes = _graph_nodes(context.graph)
    edges = _graph_edges(context.graph, nodes)
    sources = [
        edge["from"]
        for edge in edges
        if edge["to"] == node_id and edge["type"] == "reviews"
    ]
    if len(sources) != 1:
        raise G3ValidationError(f"{node_id}_target_source_mismatch")
    source_record = context.objects.get(sources[0])
    if not isinstance(source_record, dict):
        raise G3ValidationError(f"{node_id}_target_source_missing")
    source_kind = nodes[sources[0]]["authority_kind"]
    return _review_target_for_source(
        source_kind, context.expected_target_commit, source_record.get("sha256")
    )


def _validate_review_target(
    node_id: str, target: object, expected: dict[str, str]
) -> None:
    if not isinstance(target, dict):
        raise G3ValidationError(f"{node_id}_target_mismatch")
    target_type = target.get("type")
    if target_type == "git_commit":
        valid_shape = set(target) == {"type", "git_sha"} and isinstance(
            target.get("git_sha"), str
        ) and re.fullmatch(r"[0-9a-f]{40}", target["git_sha"]) is not None
    elif target_type == "external_object":
        valid_shape = (
            set(target) == {"type", "object_kind", "object_sha256"}
            and isinstance(target.get("object_kind"), str)
            and target["object_kind"] in R11_REVIEW_OBJECT_KINDS
            and isinstance(target.get("object_sha256"), str)
            and re.fullmatch(r"[0-9a-f]{64}", target["object_sha256"]) is not None
        )
    else:
        valid_shape = False
    if not valid_shape or target != expected:
        raise G3ValidationError(f"{node_id}_target_mismatch")


def _graph_nodes(graph: dict[str, Any]) -> dict[str, dict[str, Any]]:
    nodes = graph.get("nodes")
    if not isinstance(nodes, list):
        raise ValueError("R12 graph nodes must be an array")
    by_id: dict[str, dict[str, Any]] = {}
    for node in nodes:
        if not isinstance(node, dict) or not isinstance(node.get("id"), str):
            raise ValueError("R12 graph node is malformed")
        node_id = node["id"]
        if node_id in by_id:
            raise ValueError(f"duplicate R12 graph node: {node_id}")
        if not isinstance(node.get("authority_kind"), str) or not node["authority_kind"]:
            raise ValueError(f"R12 graph node kind missing: {node_id}")
        stage = node.get("creation_stage")
        if not isinstance(stage, int) or stage not in GRAPH_STAGE_NAMES:
            raise ValueError(f"invalid R12 graph creation stage: {node_id}")
        if set(node) != {"id", "authority_kind", "creation_stage", "binding_fields"}:
            raise ValueError(f"R12 graph node field closure is not exact: {node_id}")
        if not isinstance(node["binding_fields"], list):
            raise ValueError(f"R12 graph node binding-field contract is malformed: {node_id}")
        by_id[node_id] = node
    return by_id


def _graph_edges(
    graph: dict[str, Any], nodes: dict[str, dict[str, Any]]
) -> list[dict[str, Any]]:
    edges = graph.get("edges")
    if not isinstance(edges, list):
        raise ValueError("R12 graph edges must be an array")
    seen: set[tuple[str, str, str]] = set()
    result: list[dict[str, Any]] = []
    for edge in edges:
        if not isinstance(edge, dict):
            raise ValueError("R12 graph edge is malformed")
        if set(edge) != {"from", "to", "type", "binding_obligation"}:
            raise ValueError("R12 graph edge field closure is not exact")
        source, target, edge_type = (
            edge.get("from"),
            edge.get("to"),
            edge.get("type"),
        )
        if not all(isinstance(value, str) for value in (source, target, edge_type)):
            raise ValueError("R12 graph edge fields are malformed")
        if source not in nodes or target not in nodes:
            raise ValueError(f"R12 graph edge references unknown node: {source}->{target}")
        if edge_type not in GRAPH_EDGE_TYPES:
            raise ValueError(f"unknown R12 graph edge type: {edge_type}")
        if source == target:
            raise ValueError(f"R12 graph self edge: {source}")
        key = (source, target, edge_type)
        if key in seen:
            raise ValueError(f"duplicate R12 graph edge: {key}")
        seen.add(key)
        obligation = edge["binding_obligation"]
        if not isinstance(obligation, dict):
            raise ValueError(f"R12 edge binding obligation is malformed: {key}")
        kind = obligation.get("kind")
        if kind == "none":
            if set(obligation) != {"kind"}:
                raise ValueError(f"R12 none binding obligation is not closed: {key}")
        elif kind == "serialized_binding":
            required = {
                "kind",
                "destination_field",
                "category",
                "value_semantics",
                "cardinality",
                "target_object_kind",
            }
            if set(obligation) != required:
                raise ValueError(f"R12 serialized binding obligation is not closed: {key}")
            field_name = obligation["destination_field"]
            category = obligation["category"]
            value_semantics = obligation["value_semantics"]
            cardinality = obligation["cardinality"]
            target_kind = obligation["target_object_kind"]
            if (
                not isinstance(field_name, str)
                or field_name not in SERIALIZED_BINDING_FIELD_SEMANTICS
                or not isinstance(category, str)
                or not isinstance(value_semantics, str)
                or SERIALIZED_BINDING_FIELD_SEMANTICS[field_name]
                != (category, value_semantics)
                or not isinstance(cardinality, str)
                or SERIALIZED_BINDING_CARDINALITIES.get(field_name) != cardinality
                or target_kind != nodes[target]["authority_kind"]
            ):
                raise ValueError(f"R12 serialized binding obligation value is invalid: {key}")
            object_fields = graph.get("object_field_contracts")
            if not isinstance(object_fields, dict) or not isinstance(
                object_fields.get(target), list
            ) or field_name not in object_fields[target]:
                raise ValueError(f"R12 binding field is absent from object schema: {key}")
        else:
            raise ValueError(f"R12 binding obligation kind is unknown: {key}")
        result.append(
            {
                "from": source,
                "to": target,
                "type": edge_type,
                "binding_obligation": deepcopy(obligation),
            }
        )
    return result


def _topological_order(
    nodes: dict[str, dict[str, Any]], edges: list[dict[str, Any]]
) -> list[str]:
    outgoing: dict[str, list[str]] = {node_id: [] for node_id in nodes}
    indegree = {node_id: 0 for node_id in nodes}
    for edge in edges:
        outgoing[edge["from"]].append(edge["to"])
        indegree[edge["to"]] += 1
    ready = [node_id for node_id in nodes if indegree[node_id] == 0]
    ready.sort(key=lambda node_id: (nodes[node_id]["creation_stage"], node_id))
    order: list[str] = []
    while ready:
        node_id = ready.pop(0)
        order.append(node_id)
        for child in sorted(outgoing[node_id]):
            indegree[child] -= 1
            if indegree[child] == 0:
                ready.append(child)
        ready.sort(key=lambda candidate: (nodes[candidate]["creation_stage"], candidate))
    if len(order) != len(nodes):
        raise ValueError("R12 artifact prerequisite cycle")
    return order


def _ancestors(
    node_id: str, edges: list[dict[str, Any]], excluded: set[str] | None = None
) -> set[str]:
    excluded = excluded or set()
    reverse: dict[str, list[str]] = {}
    for edge in edges:
        if edge["from"] in excluded or edge["to"] in excluded:
            continue
        reverse.setdefault(edge["to"], []).append(edge["from"])
    found: set[str] = set()
    stack = list(reverse.get(node_id, []))
    while stack:
        current = stack.pop()
        if current in found or current in excluded:
            continue
        found.add(current)
        stack.extend(reverse.get(current, []))
    return found


def _audit_record(
    name: str, passed: bool, checked_nodes: int, checked_edges: int,
    violation_path: list[str] | None = None,
) -> dict[str, Any]:
    return {
        "name": name,
        "passed": passed,
        "checked_nodes": checked_nodes,
        "checked_edges": checked_edges,
        "violation_path": violation_path or [],
    }


def _identity_rule_contract(graph: dict[str, Any]) -> dict[str, dict[str, set[str]]]:
    contract = graph.get("identity_rule_contract")
    if not isinstance(contract, dict) or not contract:
        raise ValueError("R12 identity-rule contract is missing")
    result: dict[str, dict[str, set[str]]] = {}
    for rule_type, fields in contract.items():
        if not isinstance(rule_type, str) or not isinstance(fields, dict):
            raise ValueError("R12 identity-rule contract is malformed")
        required = fields.get("required_fields")
        optional = fields.get("optional_fields")
        if (
            not isinstance(required, list)
            or not isinstance(optional, list)
            or len(required) != len(set(required))
            or len(optional) != len(set(optional))
            or any(not isinstance(name, str) or not name for name in required + optional)
            or set(required).intersection(optional)
        ):
            raise ValueError(f"R12 identity-rule field contract is malformed: {rule_type}")
        result[rule_type] = {"required": set(required), "optional": set(optional)}
    return result


def _typed_edge_contract(graph: dict[str, Any]) -> set[str]:
    contract = graph.get("typed_edge_contract")
    if not isinstance(contract, list) or len(contract) != len(set(contract)):
        raise ValueError("R12 typed-edge contract is malformed")
    if any(not isinstance(item, str) or item.count("|") != 2 for item in contract):
        raise ValueError("R12 typed-edge contract entry is malformed")
    return set(contract)


def _node_edge_contract(
    graph: dict[str, Any], nodes: dict[str, dict[str, Any]]
) -> set[tuple[str, str, str]]:
    contract = graph.get("edge_contract")
    if not isinstance(contract, list):
        raise ValueError("R12 exact node-edge contract is missing")
    result: set[tuple[str, str, str]] = set()
    for item in contract:
        if not isinstance(item, str) or item.count("|") != 2:
            raise ValueError("R12 exact node-edge contract entry is malformed")
        source, edge_type, target = item.split("|")
        if source not in nodes or target not in nodes or edge_type not in GRAPH_EDGE_TYPES:
            raise ValueError("R12 exact node-edge contract references an invalid value")
        key = (source, edge_type, target)
        if key in result:
            raise ValueError(f"duplicate R12 exact node-edge contract: {key}")
        result.add(key)
    return result


def _binding_semantics(
    graph: dict[str, Any], nodes: dict[str, dict[str, Any]]
) -> tuple[dict[str, str], list[dict[str, str]]]:
    semantics = graph.get("binding_semantics")
    if not isinstance(semantics, dict) or set(semantics) != {
        "authority_status", "derived_from", "relation_policies", "serialized_rules"
    }:
        raise ValueError("R12 binding semantics are not closed")
    if (
        semantics["authority_status"] != "DERIVED_NON_NORMATIVE"
        or semantics["derived_from"] != "edges[].binding_obligation"
    ):
        raise ValueError("R12 binding semantics are not marked as derived")
    policies = semantics["relation_policies"]
    if not isinstance(policies, dict) or set(policies) != GRAPH_EDGE_TYPES:
        raise ValueError("R12 binding relation-policy catalog is not closed")
    for edge_type, policy in policies.items():
        if (
            not isinstance(policy, dict)
            or set(policy) != {"required_input", "serialized_binding"}
            or policy["required_input"] is not True
            or policy["serialized_binding"] not in SERIALIZED_BINDING_POLICIES
        ):
            raise ValueError(f"R12 binding relation policy is malformed: {edge_type}")

    rules = semantics["serialized_rules"]
    if not isinstance(rules, list):
        raise ValueError("R12 serialized binding semantic rules are malformed")
    seen: set[tuple[str, str, str]] = set()
    normalized: list[dict[str, str]] = []
    for rule in rules:
        required = {
            "target",
            "type",
            "source",
            "field",
            "category",
            "value",
            "cardinality",
            "target_object_kind",
        }
        if not isinstance(rule, dict) or set(rule) != required:
            raise ValueError("R12 serialized binding semantic rule is malformed")
        target = rule["target"]
        edge_type = rule["type"]
        source = rule["source"]
        field_name = rule["field"]
        category = rule["category"]
        value_source = rule["value"]
        cardinality = rule["cardinality"]
        target_object_kind = rule["target_object_kind"]
        if (
            not isinstance(target, str)
            or target not in nodes
            or not isinstance(edge_type, str)
            or edge_type not in GRAPH_EDGE_TYPES
            or not isinstance(source, str)
            or (source != "*" and source not in nodes)
            or not isinstance(field_name, str)
            or not field_name
            or not isinstance(category, str)
            or category not in SERIALIZED_BINDING_CATEGORIES
            or not isinstance(value_source, str)
            or value_source not in SERIALIZED_BINDING_VALUE_SOURCES
            or SERIALIZED_BINDING_FIELD_SEMANTICS.get(field_name)
            != (category, value_source)
            or SERIALIZED_BINDING_CARDINALITIES.get(field_name) != cardinality
            or target_object_kind != nodes[target]["authority_kind"]
        ):
            raise ValueError("R12 serialized binding semantic rule value is invalid")
        key = (target, edge_type, source)
        if key in seen:
            raise ValueError(f"duplicate R12 serialized binding semantic rule: {key}")
        seen.add(key)
        schema_fields = graph.get("object_field_contracts", {}).get(target)
        if isinstance(schema_fields, list) and field_name not in schema_fields:
            raise ValueError(f"R12 binding field is absent from object schema: {target}/{field_name}")
        normalized.append(
            {
                "target": target,
                "type": edge_type,
                "source": source,
                "field": field_name,
                "category": category,
                "value": value_source,
                "cardinality": cardinality,
                "target_object_kind": target_object_kind,
            }
        )
    return {edge_type: policies[edge_type]["serialized_binding"] for edge_type in GRAPH_EDGE_TYPES}, normalized


def _semantic_rule_key(rule: dict[str, str]) -> tuple[str, str, str, str, str, str, str, str]:
    return (
        rule["target"],
        rule["type"],
        rule["source"],
        rule["field"],
        rule["category"],
        rule["value"],
        rule["cardinality"],
        rule["target_object_kind"],
    )


def _canonical_semantic_rules(
    rules: list[dict[str, str]],
) -> list[tuple[str, str, str, str, str, str, str, str]]:
    return sorted(_semantic_rule_key(rule) for rule in rules)


def _edge_binding_rule(
    edge: dict[str, Any], nodes: dict[str, dict[str, Any]]
) -> dict[str, str] | None:
    obligation = edge["binding_obligation"]
    if obligation["kind"] == "none":
        return None
    return {
        "target": edge["to"],
        "type": edge["type"],
        "source": edge["from"],
        "field": obligation["destination_field"],
        "category": obligation["category"],
        "value": obligation["value_semantics"],
        "cardinality": obligation["cardinality"],
        "target_object_kind": nodes[edge["to"]]["authority_kind"],
    }


def _derived_binding_rules(
    nodes: dict[str, dict[str, Any]], edges: list[dict[str, Any]]
) -> list[dict[str, str]]:
    """Derive compact semantic rules from the complete edge obligations."""

    grouped: dict[
        tuple[str, str, str, str, str, str, str], list[str]
    ] = {}
    edge_counts: dict[tuple[str, str], int] = {}
    for edge in edges:
        key = (edge["to"], edge["type"])
        edge_counts[key] = edge_counts.get(key, 0) + 1
        rule = _edge_binding_rule(edge, nodes)
        if rule is None:
            continue
        descriptor = (
            rule["target"],
            rule["type"],
            rule["field"],
            rule["category"],
            rule["value"],
            rule["cardinality"],
            rule["target_object_kind"],
        )
        grouped.setdefault(descriptor, []).append(rule["source"])

    result: list[dict[str, str]] = []
    for descriptor, sources in grouped.items():
        target, edge_type, field_name, category, value, cardinality, target_kind = descriptor
        source_values = (
            ["*"]
            if len(sources) > 1 and len(sources) == edge_counts[(target, edge_type)]
            else sorted(sources)
        )
        result.extend(
            {
                "target": target,
                "type": edge_type,
                "source": source,
                "field": field_name,
                "category": category,
                "value": value,
                "cardinality": cardinality,
                "target_object_kind": target_kind,
            }
            for source in source_values
        )
    return result


def derive_node_binding_fields(
    nodes: dict[str, dict[str, Any]], edges: list[dict[str, Any]]
) -> dict[str, list[dict[str, str]]]:
    """Project the edge root into the retained node-level compatibility mirror."""

    derived = {node_id: [] for node_id in nodes}
    for rule in _derived_binding_rules(nodes, edges):
        derived[rule["target"]].append(
            {
                "field": rule["field"],
                "type": rule["type"],
                "source": rule["source"],
                "category": rule["category"],
                "value": rule["value"],
                "cardinality": rule["cardinality"],
                "target_object_kind": rule["target_object_kind"],
            }
        )
    return derived


def derive_required_semantic_rules(
    nodes: dict[str, dict[str, Any]],
    edges: list[dict[str, Any]],
    object_field_contracts: dict[str, list[str]],
) -> dict[str, Any]:
    """Derive the complete binding-rule universe from exact edge obligations."""

    if not isinstance(object_field_contracts, dict):
        raise ValueError("R12 object-field contract is malformed")
    if any(target not in nodes for target in object_field_contracts):
        raise ValueError("R12 object-field contract references an unknown node")
    if any(
        not isinstance(target, str)
        or not isinstance(fields, list)
        or len(fields) != len(set(fields))
        or any(not isinstance(field_name, str) or not field_name for field_name in fields)
        for target, fields in object_field_contracts.items()
    ):
        raise ValueError("R12 object-field contract is malformed")

    rules = _derived_binding_rules(nodes, edges)
    for rule in rules:
        schema_fields = object_field_contracts.get(rule["target"])
        if not isinstance(schema_fields, list) or rule["field"] not in schema_fields:
            raise ValueError(
                "R12 edge binding field is absent from object schema: "
                f"{rule['target']}/{rule['field']}"
            )
    policies: dict[str, str] = {}
    for edge_type in sorted(GRAPH_EDGE_TYPES):
        typed_edges = [edge for edge in edges if edge["type"] == edge_type]
        serialized_edges = [
            edge for edge in typed_edges if edge["binding_obligation"]["kind"] == "serialized_binding"
        ]
        if not serialized_edges:
            policies[edge_type] = "none"
        elif len(serialized_edges) == len(typed_edges):
            policies[edge_type] = "all"
        else:
            policies[edge_type] = "selected"

    return {"relation_policies": policies, "serialized_rules": rules}


def derive_required_inputs(
    nodes: dict[str, dict[str, Any]], edges: list[dict[str, Any]]
) -> dict[str, list[str]]:
    """Derive every prerequisite row from the immutable graph edge inventory."""

    return {
        target: sorted({edge["from"] for edge in edges if edge["to"] == target})
        for target in nodes
    }


def derive_binding_projection(
    graph: dict[str, Any],
    nodes: dict[str, dict[str, Any]],
    edges: list[dict[str, Any]],
) -> list[dict[str, str]]:
    """Derive concrete bindings from edge obligations and compare mirrors exactly."""

    independent = derive_required_semantic_rules(
        nodes, edges, graph.get("object_field_contracts")
    )
    declared_policies, declared_rules = _binding_semantics(graph, nodes)
    if (
        declared_policies != independent["relation_policies"]
        or _canonical_semantic_rules(declared_rules)
        != _canonical_semantic_rules(independent["serialized_rules"])
    ):
        raise ValueError(
            "R12 declared binding semantics do not equal the independent semantic-rule projection"
        )
    policies = independent["relation_policies"]
    rules = independent["serialized_rules"]
    projection: list[dict[str, str]] = []
    for rule in rules:
        matching_edges = [
            edge
            for edge in edges
            if edge["to"] == rule["target"]
            and edge["type"] == rule["type"]
            and (rule["source"] == "*" or edge["from"] == rule["source"])
        ]
        if not matching_edges:
            raise ValueError(
                "R12 serialized binding semantic rule has no matching edge: "
                f"{rule['target']}/{rule['type']}/{rule['source']}"
            )
        projection.extend(
            {
                "source": edge["from"],
                "relation": edge["type"],
                "target": edge["to"],
                "field": rule["field"],
                "category": rule["category"],
                "value": rule["value"],
            }
            for edge in matching_edges
        )

    for edge_type, policy in policies.items():
        typed_edges = [edge for edge in edges if edge["type"] == edge_type]
        for edge in typed_edges:
            matching_rules = [
                rule
                for rule in rules
                if rule["target"] == edge["to"]
                and rule["type"] == edge_type
                and (rule["source"] == "*" or rule["source"] == edge["from"])
            ]
            if policy == "all" and len(matching_rules) != 1:
                raise ValueError(
                    "R12 serialized binding semantic coverage mismatch: "
                    f"{edge['from']}/{edge_type}/{edge['to']}"
                )
            if policy == "none" and matching_rules:
                raise ValueError(
                    f"R12 non-serialized relation has a binding rule: {edge_type}"
                )
            if policy == "selected" and len(matching_rules) > 1:
                raise ValueError(
                    "R12 serialized binding semantic rule overlaps an edge: "
                    f"{edge['from']}/{edge_type}/{edge['to']}"
                )
    return projection


def derive_serialized_binding_contract(
    graph: dict[str, Any],
    nodes: dict[str, dict[str, Any]],
    edges: list[dict[str, Any]],
) -> dict[str, dict[str, dict[str, str]]]:
    """Project the independent binding descriptors into the checked-in mirror shape."""

    projection = derive_binding_projection(graph, nodes, edges)
    rules = derive_required_semantic_rules(
        nodes, edges, graph.get("object_field_contracts")
    )["serialized_rules"]
    derived: dict[str, dict[str, dict[str, str]]] = {}
    for rule in rules:
        if not any(
            descriptor["target"] == rule["target"]
            and descriptor["relation"] == rule["type"]
            and (rule["source"] == "*" or descriptor["source"] == rule["source"])
            for descriptor in projection
        ):
            raise ValueError(
                "R12 serialized binding semantic rule is not represented in projection: "
                f"{rule['target']}/{rule['type']}/{rule['source']}"
            )
        derived.setdefault(rule["target"], {}).setdefault(rule["type"], {})[
            rule["source"]
        ] = rule["field"]
    return derived


REVIEW_REFERENCE_CONTRACT_SHAPE = {
    "artifact_uri_prefix": REVIEW_ARTIFACT_URI_PREFIX,
    "actor_attestation_uri_prefix": REVIEWER_ACTOR_ATTESTATION_URI_PREFIX,
    "bundle_schema": {
        "authority_kind": "PhaseFIndependentReviewBundleV1",
        "fields": sorted(INDEPENDENT_REVIEW_BUNDLE_FIELDS),
        "role_order": list(REVIEW_ROLE_ORDER),
    },
    "reviewer": {
        "authority_path_template": ".phase_f_authority/reviewer_identities/{reviewer_authority_id}.json",
        "authority_kind": "PhaseFReviewerIdentityV1",
        "id_field": "reviewer_authority_id",
        "digest_excluded_fields": ["reviewer_authority_id"],
        "required_fields": [
            "reviewer_authority_id",
            "authority_kind",
            "schema_version",
            "authority_class",
            "actor_identity_digest",
            "actor_attestation_id",
            "actor_attestation_reference",
            "permitted_review_roles",
            "lifecycle",
            "stale",
            "superseded_by",
            "invalidated",
        ],
    },
    "actor_attestation": {
        "authority_path_template": ".phase_f_authority/reviewer_actor_attestations/{attestation_id}.json",
        "authority_kind": "PhaseFReviewerActorAttestationV1",
        "id_field": "attestation_id",
        "digest_excluded_fields": ["attestation_id", "signature"],
        "required_fields": [
            "attestation_id",
            "authority_kind",
            "schema_version",
            "actor_subject_id",
            "actor_class",
            "actor_identity_evidence_sha256",
            "trust_source",
            "eligible_role",
            "role_eligibility_evidence_sha256",
            "independence_evidence_sha256",
            "independence_excluded_actor_identity_digest",
            "eligibility_verifier_authority_id",
            "independence_verifier_authority_id",
            "created_at",
            "lifecycle",
            "stale",
            "superseded_by",
            "invalidated",
            "signature",
        ],
    },
    "authority_enrollment": {
        "authority_path": ".phase_f_authority/authority_enrollment.json",
        "authority_kind": "PhaseFAuthorityEnrollmentV1",
        "id_field": "enrollment_id",
        "digest_excluded_fields": ["enrollment_id"],
        "required_fields": [
            "schema_version",
            "enrollment_id",
            "phase_f_plan_tag",
            "f0_decisions_tag",
            "readiness_tag",
            "owner_authority_id",
            "registry_authority_id",
            "owner_public_key",
            "registry_public_key",
            "owner_public_key_fingerprint",
            "registry_public_key_fingerprint",
            "owner_authority_document",
            "registry_authority_document",
            "custody_policy_sha256",
            "created_at",
        ],
    },
    "artifact": {
        "authority_path_template": ".phase_f_authority/review_artifacts/{review_artifact_id}.json",
        "authority_kind": "PhaseFReviewArtifactV1",
        "id_field": "review_artifact_id",
        "digest_excluded_fields": ["review_artifact_id"],
        "required_fields": [
            "review_artifact_id",
            "authority_kind",
            "schema_version",
            "authority_class",
            "reviewer_authority_id",
            "role",
            "reviewed_target",
            "decision",
            "p0_count",
            "p1_count",
            "p2_count",
            "finding_ids",
            "independence_relation",
            "lifecycle",
            "stale",
            "superseded_by",
            "invalidated",
        ],
    },
    "remediation_author": {
        "authority_path": ".phase_f_authority/remediation_authority.json",
        "authority_kind": "PhaseFImplementationAuthorIdentityV1",
        "id_field": "authority_id",
        "digest_excluded_fields": ["authority_id"],
        "required_fields": [
            "authority_id",
            "authority_kind",
            "schema_version",
            "authority_class",
            "actor_identity_digest",
            "lifecycle",
            "stale",
            "superseded_by",
            "invalidated",
        ],
    },
}


def _review_reference_contract(graph: dict[str, Any]) -> dict[str, dict[str, Any]]:
    contract = graph.get("review_reference_contract")
    if contract != REVIEW_REFERENCE_CONTRACT_SHAPE:
        raise ValueError("R12 review-reference contract is not closed")
    return contract


def _edge_key(edge: dict[str, Any], nodes: dict[str, dict[str, Any]]) -> str:
    return "|".join(
        (
            nodes[edge["from"]]["authority_kind"],
            edge["type"],
            nodes[edge["to"]]["authority_kind"],
        )
    )


def _validate_serialized_binding_contract(
    graph: dict[str, Any], nodes: dict[str, dict[str, Any]], edges: list[dict[str, Any]]
) -> None:
    contract = graph.get("serialized_binding_fields")
    if not isinstance(contract, dict):
        raise ValueError("R12 serialized binding contract is missing")
    expected = derive_serialized_binding_contract(graph, nodes, edges)
    if contract != expected:
        raise ValueError(
            "R12 serialized binding contract does not equal the independent graph projection"
        )


def _validate_node_binding_fields(
    nodes: dict[str, dict[str, Any]], edges: list[dict[str, Any]]
) -> None:
    expected = derive_node_binding_fields(nodes, edges)
    for node_id, node in nodes.items():
        if node.get("binding_fields") != expected[node_id]:
            raise ValueError(
                "R12 node binding-field mirror does not equal the edge-root projection: "
                f"{node_id}"
            )


def _find_identity_cycle(
    graph: dict[str, Any], nodes: dict[str, dict[str, Any]]
) -> list[str] | None:
    rules = graph["node_identity_rules"]
    outgoing = {
        node_id: list(rules[node_id].get("identity_dependencies", []))
        for node_id in nodes
    }
    for node_id, dependencies in outgoing.items():
        if any(dependency not in nodes for dependency in dependencies):
            return [node_id, *[dependency for dependency in dependencies if dependency not in nodes]]
    visiting: list[str] = []
    visited: set[str] = set()

    def visit(node_id: str) -> list[str] | None:
        if node_id in visiting:
            return [*visiting[visiting.index(node_id):], node_id]
        if node_id in visited:
            return None
        visiting.append(node_id)
        for dependency in outgoing[node_id]:
            found = visit(dependency)
            if found:
                return found
        visiting.pop()
        visited.add(node_id)
        return None

    for node_id in nodes:
        found = visit(node_id)
        if found:
            return found
    return None


def _reviewer_bootstrap_trust_contract(graph: dict[str, Any]) -> dict[str, Any]:
    contract = graph.get("reviewer_bootstrap_trust_contract")
    if not isinstance(contract, dict) or set(contract) != REVIEWER_BOOTSTRAP_TRUST_CONTRACT_KEYS:
        raise ValueError("R12 reviewer bootstrap trust contract is not closed")
    if (
        contract["stage"] != REVIEWER_BOOTSTRAP_STAGE
        or contract["root_path"] != ".phase_f_authority/reviewer_bootstrap/trust_root.json"
        or contract["currentness_proof_path"]
        != ".phase_f_authority/reviewer_bootstrap/currentness_proof.json"
        or contract["root_authority_kind"] != "PhaseFReviewerBootstrapTrustRootV1"
        or contract["currentness_proof_authority_kind"]
        != "PhaseFReviewerBootstrapCurrentnessProofV1"
        or not isinstance(contract["root_id"], str)
        or not re.fullmatch(r"sha256:[0-9a-f]{64}", contract["root_id"])
        or not isinstance(contract["root_public_key_fingerprint"], str)
        or not re.fullmatch(r"[0-9a-f]{64}", contract["root_public_key_fingerprint"])
        or contract["allowed_purposes"] != REVIEWER_BOOTSTRAP_SCOPE
        or contract["transition_policy"]
        != "bootstrap_root_permanent_for_reviewer_identities"
        or contract["currentness_window_policy"]
        != "valid_from_le_validation_time_le_valid_until"
    ):
        raise ValueError("R12 reviewer bootstrap trust contract metadata mismatch")
    return contract


def _external_trust_dependency_audit(
    graph: dict[str, Any]
) -> list[str] | None:
    contract = graph.get("external_trust_dependency_contract")
    if not isinstance(contract, dict) or set(contract) != {"nodes", "edges", "terminal_roots"}:
        return ["external_trust_dependency_contract"]
    raw_nodes = contract["nodes"]
    raw_edges = contract["edges"]
    roots = contract["terminal_roots"]
    if (
        not isinstance(raw_nodes, list)
        or not isinstance(raw_edges, list)
        or not isinstance(roots, list)
        or len(raw_nodes) != len({node.get("id") for node in raw_nodes if isinstance(node, dict)})
        or any(
            not isinstance(node, dict)
            or set(node) != {"id", "stage"}
            or not isinstance(node["id"], str)
            or not isinstance(node["stage"], int)
            for node in raw_nodes
        )
    ):
        return ["external_trust_dependency_contract", "nodes"]
    node_stages = {node["id"]: node["stage"] for node in raw_nodes}
    if not roots or any(root not in node_stages for root in roots):
        return ["external_trust_dependency_contract", "terminal_roots"]
    outgoing: dict[str, list[str]] = {node_id: [] for node_id in node_stages}
    for edge in raw_edges:
        if (
            not isinstance(edge, dict)
            or set(edge) != {"from", "to"}
            or edge["from"] not in node_stages
            or edge["to"] not in node_stages
            or edge["from"] == edge["to"]
            or edge["to"] in outgoing[edge["from"]]
        ):
            return ["external_trust_dependency_contract", "edges"]
        outgoing[edge["from"]].append(edge["to"])
        if node_stages[edge["from"]] >= node_stages[edge["to"]]:
            return [edge["from"], edge["to"]]
    visiting: list[str] = []
    visited: set[str] = set()

    def visit(node_id: str) -> list[str] | None:
        if node_id in visiting:
            return [*visiting[visiting.index(node_id):], node_id]
        if node_id in visited:
            return None
        visiting.append(node_id)
        for dependent in outgoing[node_id]:
            found = visit(dependent)
            if found:
                return found
        visiting.pop()
        visited.add(node_id)
        return None

    for node_id in node_stages:
        found = visit(node_id)
        if found:
            return found
    return None


def _semantic_graph_audits(
    graph: dict[str, Any], nodes: dict[str, dict[str, Any]], edges: list[dict[str, Any]], order: list[str]
) -> list[dict[str, Any]]:
    audits: list[dict[str, Any]] = []
    identity_cycle = _find_identity_cycle(graph, nodes)
    audits.append(_audit_record("hash_cycle", identity_cycle is None, len(nodes), len(edges), identity_cycle))

    self_git_path: list[str] | None = None
    for node_id, rule in graph["node_identity_rules"].items():
        if rule["type"] == "git_commit_identity" and (
            rule.get("commit_source") == "self" or rule.get("self_referential") is True
        ):
            self_git_path = [node_id, "future_git_commit"]
            break
        if node_id in rule.get("identity_dependencies", []):
            self_git_path = [node_id, node_id]
            break
    audits.append(_audit_record("self_git_cycle", self_git_path is None, len(nodes), len(edges), self_git_path))

    review_target_path: list[str] | None = None
    for node_id, rule in graph["node_identity_rules"].items():
        target = rule.get("review_target_node")
        if target is None:
            continue
        if target not in nodes:
            review_target_path = [node_id, str(target)]
            break
        incoming = [edge for edge in edges if edge["to"] == node_id and edge["type"] in {"reviews", "targets"}]
        if target not in {edge["from"] for edge in incoming}:
            review_target_path = [target, node_id]
            break
        if nodes[target]["creation_stage"] >= nodes[node_id]["creation_stage"]:
            review_target_path = [target, node_id]
            break
    audits.append(_audit_record("review_target_cycle", review_target_path is None, len(nodes), len(edges), review_target_path))

    future_path: list[str] | None = None
    for edge in edges:
        if nodes[edge["from"]]["creation_stage"] >= nodes[edge["to"]]["creation_stage"]:
            future_path = [edge["from"], edge["to"]]
            break
    if future_path is None:
        for node_id, rule in graph["node_identity_rules"].items():
            for dependency in rule.get("identity_dependencies", []):
                if dependency in nodes and nodes[dependency]["creation_stage"] >= nodes[node_id]["creation_stage"]:
                    future_path = [dependency, node_id]
                    break
            if future_path:
                break
    audits.append(_audit_record("future_object", future_path is None, len(nodes), len(edges), future_path))

    self_reference_path: list[str] | None = None
    for node_id, rule in graph["node_identity_rules"].items():
        if rule.get("self_reference") is True:
            self_reference_path = [node_id, node_id]
            break
    audits.append(_audit_record("self_reference", self_reference_path is None, len(nodes), len(edges), self_reference_path))

    required_inputs = derive_required_inputs(nodes, edges)
    g3 = "g3_approval_tag"
    implementation = graph["implementation_gate_node"]

    def closure(node_id: str, seen: set[str] | None = None) -> set[str]:
        seen = set() if seen is None else seen
        if node_id in seen:
            return set()
        seen.add(node_id)
        found = {node_id}
        for dependency in required_inputs[node_id]:
            found.update(closure(dependency, seen))
        seen.remove(node_id)
        return found

    g3_closure = closure(g3)
    required = set(graph["g3_required_nodes"])
    g3_bypass = sorted(required - g3_closure)
    audits.append(_audit_record("g3_bypass", not g3_bypass, len(nodes), len(edges), g3_bypass))
    implementation_closure = closure(implementation)
    implementation_path = [] if g3 in implementation_closure else [implementation, g3]
    audits.append(_audit_record("implementation_bypass", not implementation_path, len(nodes), len(edges), implementation_path))
    external_path = _external_trust_dependency_audit(graph)
    audits.append(
        _audit_record(
            "external_trust_dependency_cycle",
            external_path is None,
            len(graph.get("external_trust_dependency_contract", {}).get("nodes", [])),
            len(graph.get("external_trust_dependency_contract", {}).get("edges", [])),
            external_path,
        )
    )
    return audits


def validate_r12_authority_graph(graph: dict[str, Any]) -> dict[str, Any]:
    if graph.get("schema_version") != 1:
        raise ValueError("R12 graph schema version mismatch")
    if graph.get("edge_direction") != "from_existing_prerequisite_to_constructed_dependent":
        raise ValueError("R12 graph edge direction mismatch")
    semantics = graph.get("edge_type_semantics")
    if not isinstance(semantics, dict) or set(semantics) != GRAPH_EDGE_TYPES:
        raise ValueError("R12 graph edge semantics are not closed")
    identity_rules = graph.get("identity_cycle_rules")
    if not isinstance(identity_rules, dict) or set(identity_rules) != EXPECTED_IDENTITY_CYCLE_RULES:
        raise ValueError("R12 identity-cycle rules are not closed")
    if any(
        not isinstance(rule, dict)
        or set(rule) != {"audit", "prohibited"}
        or not isinstance(rule["audit"], str)
        or rule["prohibited"] is not True
        for rule in identity_rules.values()
    ):
        raise ValueError("R12 identity-cycle rule is not typed")
    _reviewer_bootstrap_trust_contract(graph)
    if not isinstance(graph.get("external_trust_dependency_contract"), dict):
        raise ValueError("R12 external trust dependency contract is missing")
    nodes = _graph_nodes(graph)
    if set(nodes) != EXPECTED_GRAPH_NODE_IDS:
        raise ValueError("R12 graph node catalog is not closed")
    edges = _graph_edges(graph, nodes)
    typed_contract = _typed_edge_contract(graph)
    actual_typed_edges = {_edge_key(edge, nodes) for edge in edges}
    if not actual_typed_edges.issubset(typed_contract):
        raise ValueError("R12 graph edge violates typed-edge contract")
    if actual_typed_edges != typed_contract:
        raise ValueError("R12 typed-edge contract has unused or missing tuple")
    exact_edge_contract = _node_edge_contract(graph, nodes)
    actual_node_edges = {
        (edge["from"], edge["type"], edge["to"]) for edge in edges
    }
    if actual_node_edges != exact_edge_contract:
        raise ValueError("R12 exact node-edge contract has unused or missing edge")
    _review_reference_contract(graph)
    identity_contract = _identity_rule_contract(graph)
    node_identity_rules = graph.get("node_identity_rules")
    if not isinstance(node_identity_rules, dict) or set(node_identity_rules) != set(nodes):
        raise ValueError("R12 node identity-rule catalog is not closed")
    for node_id, rule in node_identity_rules.items():
        if not isinstance(rule, dict) or not isinstance(rule.get("type"), str):
            raise ValueError(f"R12 node identity rule is malformed: {node_id}")
        rule_type = rule["type"]
        if rule_type not in identity_contract:
            raise ValueError(f"R12 unknown identity-rule type: {rule_type}")
        allowed = {"type", *identity_contract[rule_type]["required"], *identity_contract[rule_type]["optional"]}
        if set(rule) - allowed or not identity_contract[rule_type]["required"].issubset(rule):
            raise ValueError(f"R12 identity-rule field closure mismatch: {node_id}")
        if any(not isinstance(dependency, str) or dependency not in nodes for dependency in rule.get("identity_dependencies", [])):
            raise ValueError(f"R12 identity-rule dependency is unknown: {node_id}")
    binding_root = graph.get("binding_root_contract")
    if binding_root != BINDING_ROOT_CONTRACT:
        raise ValueError("R12 binding-root contract is not closed")
    _validate_node_binding_fields(nodes, edges)
    _validate_serialized_binding_contract(graph, nodes, edges)
    order = _topological_order(nodes, edges)

    for edge in edges:
        if nodes[edge["from"]]["creation_stage"] > nodes[edge["to"]]["creation_stage"]:
            raise ValueError(f"R12 future-object dependency: {edge}")

    digest_edges = [
        edge for edge in edges if edge["type"] in {"hashes", "binds"}
    ]
    _topological_order(nodes, digest_edges)

    g3_node = "g3_approval_tag"
    implementation_node = graph.get("implementation_gate_node")
    if g3_node not in nodes or not isinstance(implementation_node, str):
        raise ValueError("R12 graph gate nodes are missing")
    if implementation_node != "phase_f_implementation_gate" or implementation_node not in nodes:
        raise ValueError("R12 implementation gate node is unknown")
    required = graph.get("g3_required_nodes")
    if not isinstance(required, list) or len(required) != len(set(required)):
        raise ValueError("R12 G3 required-node list is malformed")
    if any(node_id not in nodes for node_id in required):
        raise ValueError("R12 G3 required-node list references unknown node")
    required_inputs = graph.get("required_inputs")
    if not isinstance(required_inputs, dict) or set(required_inputs) != set(nodes):
        raise ValueError("R12 graph required-input closure is incomplete")
    derived_required_inputs = derive_required_inputs(nodes, edges)
    edge_keys = {(edge["from"], edge["to"]) for edge in edges}
    g3_edge_sources = {
        edge["from"] for edge in edges if edge["to"] == g3_node
    }
    if set(required) != g3_edge_sources:
        raise ValueError("R12 G3 required-node list does not match exact graph")
    if set(required_inputs[g3_node]) != g3_edge_sources:
        raise ValueError("R12 G3 required-input closure does not match exact graph")
    aggregate_dependencies = graph.get("aggregate_review_dependency_nodes")
    if (
        not isinstance(aggregate_dependencies, list)
        or any(not isinstance(node_id, str) for node_id in aggregate_dependencies)
        or len(aggregate_dependencies) != len(set(aggregate_dependencies))
        or any(node_id not in nodes for node_id in aggregate_dependencies)
    ):
        raise ValueError("R12 aggregate-review dependency contract is malformed")
    expected_aggregate_dependencies = set(
        edge["from"]
        for edge in edges
        if edge["to"] == g3_node
        and edge["type"] == "requires"
        and edge["from"].startswith("component_")
        and edge["from"].endswith("_review")
    ) | {"migrated_finding_review", "specification_bundle_manifest", "generated_traceability_manifest"}
    if set(aggregate_dependencies) != expected_aggregate_dependencies:
        raise ValueError("R12 aggregate-review dependency contract is not exact")
    for target, dependencies in required_inputs.items():
        if (
            not isinstance(dependencies, list)
            or len(dependencies) != len(set(dependencies))
            or any(dependency not in nodes for dependency in dependencies)
            or target in dependencies
        ):
            raise ValueError(f"R12 graph required-input row is malformed: {target}")
        if any((dependency, target) not in edge_keys for dependency in dependencies):
            raise ValueError(f"R12 graph required-input edge is undeclared: {target}")
        if set(dependencies) != set(derived_required_inputs[target]):
            raise ValueError(f"R12 graph required-input closure is not exact: {target}")
    g3_ancestors = _ancestors(g3_node, edges)
    missing = sorted(set(required) - g3_ancestors)
    if missing:
        raise ValueError(f"R12 G3 mandatory predecessor missing: {missing}")

    def required_closure(node_id: str, seen: set[str] | None = None) -> set[str]:
        seen = set() if seen is None else seen
        if node_id in seen:
            raise ValueError(f"R12 required-input cycle at {node_id}")
        seen.add(node_id)
        result = {node_id}
        for dependency in required_inputs[node_id]:
            result.update(required_closure(dependency, seen))
        seen.remove(node_id)
        return result

    g3_required_closure = required_closure(g3_node)
    if not set(required).issubset(g3_required_closure):
        raise ValueError("R12 G3 bypass path")
    implementation_closure = required_closure(implementation_node)
    if g3_node not in implementation_closure:
        raise ValueError("R12 implementation-readiness bypasses G3")
    if implementation_node not in implementation_closure or not required_inputs["implementation_readiness_specification"] == [g3_node]:
        raise ValueError("R12 implementation gate remains reachable without G3")

    audits = _semantic_graph_audits(graph, nodes, edges, order)
    failed_audit = next((audit for audit in audits if not audit["passed"]), None)
    if failed_audit:
        raise ValueError(
            f"R12 {failed_audit['name']} audit failed: {failed_audit['violation_path']}"
        )
    return {
        "node_count": len(nodes),
        "edge_count": len(edges),
        "edge_types": sorted({edge["type"] for edge in edges}),
        "topological_order": order,
        "g3_ancestor_count": len(g3_ancestors),
        "g3_required_count": len(required),
        "hash_cycle": any(a["name"] == "hash_cycle" and not a["passed"] for a in audits),
        "self_reference": any(a["name"] == "self_reference" and not a["passed"] for a in audits),
        "self_git_cycle": any(a["name"] == "self_git_cycle" and not a["passed"] for a in audits),
        "review_target_cycle": any(a["name"] == "review_target_cycle" and not a["passed"] for a in audits),
        "future_object": any(a["name"] == "future_object" and not a["passed"] for a in audits),
        "g3_bypass": any(a["name"] == "g3_bypass" and not a["passed"] for a in audits),
        "implementation_bypass": any(a["name"] == "implementation_bypass" and not a["passed"] for a in audits),
        "external_trust_dependency_cycle": any(
            a["name"] == "external_trust_dependency_cycle" and not a["passed"]
            for a in audits
        ),
        "audits": audits,
    }


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


# The authoritative checker uses ed25519-dalek's strict verifier.  The
# generator keeps a dependency-free verifier for REAL reference resolution so
# a structurally plausible signature cannot stand in for a cryptographic one.
_ED25519_P = 2**255 - 19
_ED25519_Q = 2**252 + 27742317777372353535851937790883648493
_ED25519_D = (-121665 * pow(121666, _ED25519_P - 2, _ED25519_P)) % _ED25519_P
_ED25519_I = pow(2, (_ED25519_P - 1) // 4, _ED25519_P)
_ED25519_IDENTITY = (0, 1)
_ED25519_B_Y = (4 * pow(5, _ED25519_P - 2, _ED25519_P)) % _ED25519_P


def _ed25519_xrecover(y: int) -> int:
    xx = ((y * y - 1) * pow(_ED25519_D * y * y + 1, _ED25519_P - 2, _ED25519_P)) % _ED25519_P
    x = pow(xx, (_ED25519_P + 3) // 8, _ED25519_P)
    if (x * x - xx) % _ED25519_P != 0:
        x = (x * _ED25519_I) % _ED25519_P
    if (x * x - xx) % _ED25519_P != 0:
        raise ValueError("invalid ed25519 point")
    return _ED25519_P - x if x & 1 else x


def _ed25519_decode_point(encoded: bytes) -> tuple[int, int]:
    if len(encoded) != 32:
        raise ValueError("invalid ed25519 point length")
    value = int.from_bytes(encoded, "little")
    sign = value >> 255
    y = value & ((1 << 255) - 1)
    if y >= _ED25519_P:
        raise ValueError("noncanonical ed25519 point")
    x = _ed25519_xrecover(y)
    if x == 0 and sign:
        raise ValueError("invalid ed25519 point sign")
    if (x & 1) != sign:
        x = _ED25519_P - x
    point = (x, y)
    if _ed25519_scalarmult(point, 8) == _ED25519_IDENTITY:
        raise ValueError("weak ed25519 point")
    return point


def _ed25519_encode_point(point: tuple[int, int]) -> bytes:
    x, y = point
    return (y | ((x & 1) << 255)).to_bytes(32, "little")


def _ed25519_add(left: tuple[int, int], right: tuple[int, int]) -> tuple[int, int]:
    x1, y1 = left
    x2, y2 = right
    product = (_ED25519_D * x1 * x2 * y1 * y2) % _ED25519_P
    x3 = ((x1 * y2 + x2 * y1) * pow(1 + product, _ED25519_P - 2, _ED25519_P)) % _ED25519_P
    y3 = ((y1 * y2 + x1 * x2) * pow(1 - product, _ED25519_P - 2, _ED25519_P)) % _ED25519_P
    return x3, y3


def _ed25519_scalarmult(point: tuple[int, int], scalar: int) -> tuple[int, int]:
    result = _ED25519_IDENTITY
    addend = point
    while scalar:
        if scalar & 1:
            result = _ed25519_add(result, addend)
        addend = _ed25519_add(addend, addend)
        scalar >>= 1
    return result


_ED25519_B = (_ed25519_xrecover(_ED25519_B_Y), _ED25519_B_Y)


def verify_ed25519_strict(
    public_key_hex: str, signature_hex: str, message: bytes
) -> bool:
    try:
        public_key = bytes.fromhex(public_key_hex)
        signature = bytes.fromhex(signature_hex)
        if len(public_key) != 32 or len(signature) != 64:
            return False
        public_point = _ed25519_decode_point(public_key)
        r_point = _ed25519_decode_point(signature[:32])
        scalar = int.from_bytes(signature[32:], "little")
        if scalar >= _ED25519_Q:
            return False
        challenge = int.from_bytes(
            hashlib.sha512(signature[:32] + public_key + message).digest(), "little"
        ) % _ED25519_Q
        expected = _ed25519_add(r_point, _ed25519_scalarmult(public_point, challenge))
        actual = _ed25519_scalarmult(_ED25519_B, scalar)
        return actual == expected
    except (ValueError, OverflowError):
        return False


def _fixture_ed25519_keypair(seed: bytes) -> tuple[str, str]:
    digest = hashlib.sha512(seed).digest()
    scalar_bytes = bytearray(digest[:32])
    scalar_bytes[0] &= 248
    scalar_bytes[31] &= 63
    scalar_bytes[31] |= 64
    scalar = int.from_bytes(scalar_bytes, "little")
    public_key = _ed25519_encode_point(_ed25519_scalarmult(_ED25519_B, scalar))
    return public_key.hex(), digest[32:].hex()


def _fixture_ed25519_sign(seed: bytes, message: bytes) -> str:
    digest = hashlib.sha512(seed).digest()
    scalar_bytes = bytearray(digest[:32])
    scalar_bytes[0] &= 248
    scalar_bytes[31] &= 63
    scalar_bytes[31] |= 64
    scalar = int.from_bytes(scalar_bytes, "little")
    public_key = _ed25519_encode_point(_ed25519_scalarmult(_ED25519_B, scalar))
    nonce = int.from_bytes(hashlib.sha512(digest[32:] + message).digest(), "little") % _ED25519_Q
    encoded_nonce = _ed25519_encode_point(_ed25519_scalarmult(_ED25519_B, nonce))
    challenge = int.from_bytes(
        hashlib.sha512(encoded_nonce + public_key + message).digest(), "little"
    ) % _ED25519_Q
    response = (nonce + challenge * scalar) % _ED25519_Q
    return (encoded_nonce + response.to_bytes(32, "little")).hex()


def git_blob(path: Path) -> str:
    return subprocess.check_output(
        ["git", "hash-object", str(path)], cwd=ROOT, text=True
    ).strip()


def parse_g3_tag(tag_name: str, body: bytes) -> dict[str, str]:
    if tag_name != G3_TAG_NAME:
        raise G3ValidationError("invalid_tag_name")
    if not isinstance(body, bytes):
        raise G3ValidationError("body_is_not_bytes")
    if not body.endswith(b"\n"):
        last_line = body.rsplit(b"\n", 1)[-1]
        if last_line.startswith(b"schema_") and last_line != b"schema_version=1":
            raise G3ValidationError("truncated_content")
        raise G3ValidationError("missing_final_newline")
    if b"\r" in body:
        raise G3ValidationError("invalid_newline")
    if any(byte > 0x7F for byte in body):
        raise G3ValidationError("non_ascii_body")

    lines = body[:-1].split(b"\n")
    if len(lines) < len(G3_TAG_FIELDS):
        raise G3ValidationError("missing_required_field")
    if len(lines) > len(G3_TAG_FIELDS):
        names = [line.split(b"=", 1)[0] for line in lines if b"=" in line]
        if len(names) != len(set(names)):
            raise G3ValidationError("duplicate_field")
        if lines[-1].startswith(b"trailing"):
            raise G3ValidationError("extra_trailing_content")
        raise G3ValidationError("unexpected_field")

    fields: dict[str, str] = {}
    for expected_name, line in zip(G3_TAG_FIELDS, lines):
        if not line:
            raise G3ValidationError("blank_line")
        if b"=" not in line:
            raise G3ValidationError("missing_delimiter")
        raw_name, raw_value = line.split(b"=", 1)
        try:
            name = raw_name.decode("ascii")
            value = raw_value.decode("ascii")
        except UnicodeDecodeError as error:
            raise G3ValidationError("non_ascii_body") from error
        if name in G3_LEGACY_FIELDS:
            raise G3ValidationError("legacy_field_name")
        if name == "unexpected_field":
            raise G3ValidationError("unexpected_field")
        if name not in G3_TAG_FIELDS:
            raise G3ValidationError("unknown_field")
        if name in fields:
            raise G3ValidationError("duplicate_field")
        if name != expected_name:
            raise G3ValidationError("wrong_field_order")
        if not value or value != value.strip():
            raise G3ValidationError("trailing_whitespace")
        if "=" in value:
            raise G3ValidationError("unexpected_value_delimiter")
        fields[name] = value

    for field in G3_TAG_FIELDS:
        if field not in fields:
            raise G3ValidationError("missing_required_field")
    if not re.fullmatch(r"[0-9a-f]{64}", fields["specification_bundle_manifest_sha256"]):
        raise G3ValidationError("malformed_sha256")
    if not re.fullmatch(r"[0-9a-f]{64}", fields["aggregate_review_bundle_sha256"]):
        raise G3ValidationError("malformed_sha256")
    if fields["approval_decision"] != "GO":
        raise G3ValidationError("invalid_approval_decision")
    if fields["schema_version"] != "1":
        raise G3ValidationError("invalid_schema_version")
    if fields["phase_f_architecture_plan_tag"] != G3_EXPECTED_FIELDS[
        "phase_f_architecture_plan_tag"
    ]:
        raise G3ValidationError("wrong_architecture_plan_binding")
    if fields["phase_f_f0_decisions_tag"] != G3_EXPECTED_FIELDS[
        "phase_f_f0_decisions_tag"
    ]:
        raise G3ValidationError("wrong_f0_decisions_binding")
    return fields


def _record_target(record: dict[str, Any]) -> str | None:
    for field_name in (
        "target_sha256",
        "target_bundle_inputs_sha256",
        "target_bundle_manifest_sha256",
        "target_git_commit",
        "target_commit",
    ):
        value = record.get(field_name)
        if isinstance(value, str):
            return value
    return None


def _authority_descriptor(record: dict[str, Any]) -> dict[str, str | None]:
    return {
        "authority_id": record.get("authority_id")
        if isinstance(record.get("authority_id"), str)
        else None,
        "sha256": record.get("sha256")
        if isinstance(record.get("sha256"), str)
        else None,
        "target": _record_target(record),
    }


def _graph_edges_for(
    graph: dict[str, Any], target: str, edge_type: str
) -> list[dict[str, str]]:
    nodes = _graph_nodes(graph)
    return [
        edge
        for edge in _graph_edges(graph, nodes)
        if edge["to"] == target and edge["type"] == edge_type
    ]


def _binding_field(
    contract: dict[str, dict[str, dict[str, str]]], target: str, edge_type: str, source: str
) -> str | None:
    target_contract = contract.get(target, {})
    relation = target_contract.get(edge_type, {})
    return relation.get(source) or relation.get("*")


def _migrated_review_input_fingerprint(record: dict[str, Any]) -> str:
    payload = {
        "target_git_commit": record.get("target_git_commit"),
        "target_bundle_inputs_sha256": record.get("target_bundle_inputs_sha256"),
        "reviewed_migration_ledger_sha256": record.get("reviewed_migration_ledger_sha256"),
        "reviewed_normative_traceability_matrix_sha256": record.get(
            "reviewed_normative_traceability_matrix_sha256"
        ),
        "reviewed_traceability_manifest_sha256": record.get(
            "reviewed_traceability_manifest_sha256"
        ),
        "reviewed_component_sha256s": record.get("reviewed_component_sha256s"),
        "reviewed_finding_ids": record.get("reviewed_finding_ids"),
        "finding_dispositions": record.get("finding_dispositions"),
    }
    return sha256_bytes(canonical_json_bytes(payload))


def _disposition_counts(
    dispositions: dict[str, str],
) -> tuple[dict[str, int], bool]:
    counts = {"p0_count": 0, "p1_count": 0, "p2_count": 0}
    unresolved = False
    for finding_id, disposition in dispositions.items():
        severity = EXPECTED_MIGRATED_FINDING_SEVERITIES[finding_id]
        if disposition == "TECHNICALLY_CLOSED":
            continue
        if disposition == "NON_BLOCKING_DEBT":
            if severity != 2:
                raise G3ValidationError("invalid_non_blocking_debt_severity")
            counts["p2_count"] += 1
            continue
        unresolved = True
        if severity in (0, 1, 2):
            counts[f"p{severity}_count"] += 1
    return counts, unresolved


def _parse_json_without_duplicates(value: bytes) -> object:
    def reject_duplicate_pairs(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, item in pairs:
            if key in result:
                raise ValueError(f"duplicate JSON member: {key}")
            result[key] = item
        return result

    return json.loads(value.decode("utf-8"), object_pairs_hook=reject_duplicate_pairs)


def _reference_identity_matches(
    context: G3AuthorityContext,
    reference_type: str,
    record: dict[str, Any],
) -> bool:
    contract = _review_reference_contract(context.graph)[reference_type]
    expected = record.get("sha256")
    if not isinstance(expected, str) or not re.fullmatch(r"[0-9a-f]{64}", expected):
        return False
    if context.mode == "synthetic":
        return (
            record.get("digest_valid") is True
            and record.get("expected_sha256") == expected
            and record.get("content_unchanged", True) is True
        )
    if record.get("content_unchanged", True) is not True:
        return False
    canonical_object = record.get("canonical_object")
    if not isinstance(canonical_object, dict):
        return False
    if record.get("bytes") != canonical_json_bytes(canonical_object):
        return False
    payload = {
        key: value
        for key, value in canonical_object.items()
        if key not in contract["digest_excluded_fields"]
    }
    return sha256_bytes(canonical_json_bytes(payload)) == expected


def _validate_reviewer_actor_binding(
    context: G3AuthorityContext, reviewer: dict[str, Any], role: str
) -> None:
    if context.mode not in {"real", "real_test"}:
        return
    contract = _review_reference_contract(context.graph)
    attestation_id = reviewer.get("actor_attestation_id")
    reference = reviewer.get("actor_attestation_reference")
    if not isinstance(attestation_id, str) or not re.fullmatch(
        r"sha256:[0-9a-f]{64}", attestation_id
    ):
        raise G3ValidationError("reviewer_actor_attestation_id_malformed")
    if not isinstance(reference, dict) or set(reference) != {
        "immutable_uri", "sha256", "byte_length"
    }:
        raise G3ValidationError("reviewer_actor_attestation_reference_schema_mismatch")
    if (
        reference.get("immutable_uri")
        != f"{REVIEWER_ACTOR_ATTESTATION_URI_PREFIX}{attestation_id}"
        or not isinstance(reference.get("sha256"), str)
        or not re.fullmatch(r"[0-9a-f]{64}", reference["sha256"])
        or not isinstance(reference.get("byte_length"), str)
        or not re.fullmatch(CANONICAL_UNSIGNED_INTEGER_PATTERN, reference["byte_length"])
    ):
        raise G3ValidationError("reviewer_actor_attestation_reference_malformed")
    attestation = context.reviewer_actor_attestations.get(attestation_id)
    if attestation is None:
        raise G3ValidationError("unresolved_reviewer_actor_attestation")
    if attestation.get("authority_kind") != contract["actor_attestation"]["authority_kind"]:
        raise G3ValidationError("wrong_reviewer_actor_attestation_kind")
    if attestation.get("attestation_id") != attestation_id:
        raise G3ValidationError("reviewer_actor_attestation_identity_mismatch")
    if attestation.get("signature_verified") is not True:
        raise G3ValidationError("invalid_reviewer_actor_attestation_signature")
    if (
        not isinstance(attestation.get("bytes"), bytes)
        or reference["sha256"] != attestation.get("complete_file_sha256")
        or reference["sha256"] != sha256_bytes(attestation["bytes"])
        or reference["byte_length"] != str(len(attestation["bytes"]))
    ):
        raise G3ValidationError("reviewer_actor_attestation_reference_mismatch")
    if (
        attestation.get("schema_version") != 1
        or attestation.get("actor_class") != "natural_person"
        or attestation.get("eligible_role") != role
        or attestation.get("lifecycle") != "ACTIVE"
        or attestation.get("stale") is not False
        or attestation.get("superseded_by") is not None
        or attestation.get("invalidated") is not False
    ):
        raise G3ValidationError("stale_reviewer_actor_attestation")
    root, currentness, subject_index = _validate_reviewer_bootstrap_context(context)
    trust_source = attestation.get("trust_source")
    if (
        not isinstance(trust_source, dict)
        or set(trust_source) != REVIEWER_BOOTSTRAP_TRUST_SOURCE_FIELDS
        or trust_source.get("type") != REVIEWER_BOOTSTRAP_TRUST_SOURCE
        or trust_source.get("root_id") != root.get("root_id")
        or trust_source.get("root_sha256") != root.get("complete_file_sha256")
        or trust_source.get("currentness_proof_id")
        != currentness.get("currentness_proof_id")
        or trust_source.get("currentness_proof_sha256")
        != currentness.get("complete_file_sha256")
        or attestation.get("eligibility_verifier_authority_id")
        != currentness.get("current_verifier_authority_id")
        or attestation.get("independence_verifier_authority_id")
        != currentness.get("current_verifier_authority_id")
    ):
        raise G3ValidationError("reviewer_actor_attestation_trust_source_mismatch")
    actor_subject_id = attestation.get("actor_subject_id")
    if not isinstance(actor_subject_id, str) or not re.fullmatch(
        RUNTIME_STABLE_ID_PATTERN, actor_subject_id
    ):
        raise G3ValidationError("malformed_reviewer_actor_subject")
    identity_evidence = attestation.get("actor_identity_evidence_sha256")
    subject_binding = subject_index.get(actor_subject_id)
    if (
        subject_binding is None
        or subject_binding.get("identity_evidence_sha256") != identity_evidence
    ):
        raise G3ValidationError("reviewer_actor_subject_registry_mismatch")
    actor_digest = reviewer.get("actor_identity_digest")
    if actor_digest != reviewer_actor_identity_digest(actor_subject_id):
        raise G3ValidationError("reviewer_identity_digest_mismatch")
    if (
        context.remediation_actor_identity_digest is None
        or attestation.get("independence_excluded_actor_identity_digest")
        != context.remediation_actor_identity_digest
        or actor_digest == context.remediation_actor_identity_digest
    ):
        raise G3ValidationError("non_independent_migrated_review")
    signing_payload = {
        key: attestation[key]
        for key in context.graph["review_reference_contract"]["actor_attestation"][
            "required_fields"
        ]
        if key != "signature"
    }
    if not verify_ed25519_strict(
        currentness["current_verifier_public_key"],
        attestation.get("signature", ""),
        REVIEWER_ACTOR_ATTESTATION_DOMAIN + canonical_jcs_bytes(signing_payload),
    ):
        raise G3ValidationError("invalid_reviewer_actor_attestation_signature")


def _validate_review_reference(
    context: G3AuthorityContext, row: dict[str, Any]
) -> None:
    role = row.get("role")
    reviewer_id = row.get("reviewer_authority_id")
    artifact_id = row.get("review_artifact_id")
    if not isinstance(role, str) or role not in REVIEW_ROLES:
        raise G3ValidationError("non_independent_migrated_review")
    if not isinstance(reviewer_id, str) or not reviewer_id:
        raise G3ValidationError("unresolved_reviewer_identity")
    if not isinstance(artifact_id, str) or not artifact_id:
        raise G3ValidationError("unresolved_review_artifact_identity")
    reviewer = context.reviewer_authorities.get(reviewer_id)
    artifact = context.review_artifacts.get(artifact_id)
    if reviewer is None:
        raise G3ValidationError("unresolved_reviewer_identity")
    if artifact is None:
        raise G3ValidationError("unresolved_review_artifact_identity")
    reviewer_contract = _review_reference_contract(context.graph)["reviewer"]
    artifact_contract = _review_reference_contract(context.graph)["artifact"]
    if reviewer.get("authority_kind") != reviewer_contract["authority_kind"]:
        raise G3ValidationError("wrong_reviewer_identity_kind")
    if artifact.get("authority_kind") != artifact_contract["authority_kind"]:
        raise G3ValidationError("wrong_review_artifact_kind")
    if reviewer.get("reviewer_authority_id") != reviewer_id:
        raise G3ValidationError("reviewer_identity_mismatch")
    if artifact.get("review_artifact_id") != artifact_id:
        raise G3ValidationError("review_artifact_identity_mismatch")
    if reviewer.get("schema_version") != 1 or artifact.get("schema_version") != 1:
        raise G3ValidationError("review_reference_schema_mismatch")
    if reviewer.get("authority_class") not in {"REAL", "TEST_ONLY"} or artifact.get(
        "authority_class"
    ) not in {"REAL", "TEST_ONLY"}:
        raise G3ValidationError("review_reference_classification_malformed")
    if reviewer.get("authority_class") != artifact.get("authority_class"):
        raise G3ValidationError("review_reference_classification_mismatch")
    if (
        context.mode == "real"
        and reviewer.get("authority_class") == "TEST_ONLY"
    ):
        raise G3ValidationError("synthetic_review_reference_in_real_mode")
    if not _reference_identity_matches(context, "reviewer", reviewer):
        raise G3ValidationError("reviewer_identity_digest_mismatch")
    if not _reference_identity_matches(context, "artifact", artifact):
        raise G3ValidationError("review_artifact_digest_mismatch")
    if (
        reviewer.get("lifecycle") != "ACTIVE"
        or reviewer.get("stale") is not False
        or reviewer.get("superseded_by") is not None
        or reviewer.get("invalidated") is not False
        or artifact.get("lifecycle") != "ACTIVE"
        or artifact.get("stale") is not False
        or artifact.get("superseded_by") is not None
        or artifact.get("invalidated") is not False
    ):
        raise G3ValidationError("stale_migrated_review_record")
    permitted_roles = reviewer.get("permitted_review_roles")
    if (
        not isinstance(permitted_roles, list)
        or any(not isinstance(value, str) or value not in REVIEW_ROLES for value in permitted_roles)
        or len(permitted_roles) != len(set(permitted_roles))
        or role not in permitted_roles
    ):
        raise G3ValidationError("reviewer_role_not_permitted")
    if (
        artifact.get("reviewer_authority_id") != reviewer_id
        or artifact.get("role") != role
        or artifact.get("reviewed_target") != row.get("reviewed_target")
        or artifact.get("decision") != row.get("decision")
    ):
        raise G3ValidationError("review_artifact_binding_mismatch")
    actor_digest = reviewer.get("actor_identity_digest")
    if not isinstance(actor_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", actor_digest):
        if context.mode in {"real", "real_test"}:
            raise G3ValidationError("reviewer_identity_digest_malformed")
    if (
        context.mode in {"real", "real_test"}
        and context.remediation_actor_identity_digest is not None
        and actor_digest == context.remediation_actor_identity_digest
    ):
        raise G3ValidationError("non_independent_migrated_review")
    _validate_reviewer_actor_binding(context, reviewer, role)


def _require_distinct_reviewer_actor_digests(
    context: G3AuthorityContext, reviewer_ids: list[str], category: str
) -> None:
    actor_digests = [
        context.reviewer_authorities[reviewer_id].get("actor_identity_digest")
        for reviewer_id in reviewer_ids
    ]
    if len(set(actor_digests)) != len(actor_digests):
        raise G3ValidationError(category)


def _object_digest_matches(context: G3AuthorityContext, record: dict[str, Any]) -> bool:
    expected = record.get("sha256")
    if not isinstance(expected, str) or not re.fullmatch(r"[0-9a-f]{64}", expected):
        return False
    if context.mode == "synthetic":
        return (
            record.get("digest_valid") is True
            and record.get("expected_sha256") == expected
            and record.get("content_unchanged", True) is True
        )
    if record.get("content_unchanged", True) is not True:
        return False
    canonical_object = record.get("canonical_object")
    if isinstance(canonical_object, dict):
        if record.get("bytes") != canonical_json_bytes(canonical_object):
            return False
        identity_rule = context.graph["node_identity_rules"].get(record.get("node_id"), {})
        excluded = identity_rule.get("exclude_fields", [])
        payload = {
            key: value for key, value in canonical_object.items() if key not in excluded
        }
        return sha256_bytes(canonical_json_bytes(payload)) == expected
    value = record.get("bytes")
    return isinstance(value, bytes) and sha256_bytes(value) == expected


def _bootstrap_subject_index(proof: dict[str, Any]) -> dict[str, dict[str, str]]:
    bindings = proof.get("subject_bindings")
    if not isinstance(bindings, list) or not bindings:
        raise G3ValidationError("bootstrap_subject_registry_missing")
    if bindings != sorted(bindings, key=lambda value: value.get("actor_subject_id", "")):
        raise G3ValidationError("bootstrap_subject_registry_order_mismatch")
    by_subject: dict[str, dict[str, str]] = {}
    by_evidence: dict[str, str] = {}
    for binding in bindings:
        if not isinstance(binding, dict) or set(binding) != {
            "actor_subject_id", "identity_evidence_sha256", "subject_status"
        }:
            raise G3ValidationError("bootstrap_subject_registry_schema_mismatch")
        subject = binding["actor_subject_id"]
        evidence = binding["identity_evidence_sha256"]
        if (
            not isinstance(subject, str)
            or not re.fullmatch(RUNTIME_STABLE_ID_PATTERN, subject)
            or not isinstance(evidence, str)
            or not re.fullmatch(r"[0-9a-f]{64}", evidence)
            or binding["subject_status"] != "ACTIVE"
            or subject in by_subject
        ):
            raise G3ValidationError("bootstrap_subject_registry_alias_or_malformed")
        if evidence in by_evidence and by_evidence[evidence] != subject:
            raise G3ValidationError("bootstrap_subject_registry_alias_or_malformed")
        by_subject[subject] = binding
        by_evidence[evidence] = subject
    expected_head = reviewer_bootstrap_subject_registry_head_sha256(
        proof.get("sequence"), bindings
    )
    if proof.get("subject_registry_head_sha256") != expected_head:
        raise G3ValidationError("bootstrap_subject_registry_head_mismatch")
    return by_subject


def _validate_reviewer_bootstrap_context(
    context: G3AuthorityContext,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, dict[str, str]]]:
    root = context.reviewer_bootstrap_root
    proof = context.reviewer_bootstrap_currentness
    contract = _reviewer_bootstrap_trust_contract(context.graph)
    if not isinstance(root, dict) or not isinstance(proof, dict):
        raise G3ValidationError("unresolved_reviewer_bootstrap_trust")
    if (
        root.get("authority_kind") != contract["root_authority_kind"]
        or root.get("root_id") != contract["root_id"]
        or reviewer_bootstrap_root_id(root) != root.get("root_id")
        or root.get("stage") != REVIEWER_BOOTSTRAP_STAGE
        or root.get("schema_version") != 1
        or root.get("authority_class") not in {"REAL", "TEST_ONLY"}
        or (root.get("authority_class") == "TEST_ONLY" and context.mode == "real")
        or root.get("authority_scope") != REVIEWER_BOOTSTRAP_SCOPE
        or root.get("subject_uniqueness_policy") != "one_natural_person_one_subject"
        or root.get("evidence_retention_policy")
        != "retain_external_identity_evidence_hash_binding"
        or root.get("rotation_policy") != "forward_signed_replacement_only"
        or root.get("compromise_policy") != "immediate_reject"
        or root.get("lifecycle") != "ACTIVE"
        or root.get("stale") is not False
        or root.get("superseded_by") is not None
        or root.get("invalidated") is not False
        or not isinstance(root.get("root_public_key"), str)
        or not re.fullmatch(ED25519_PUBLIC_KEY_PATTERN, root["root_public_key"])
        or root.get("root_public_key_fingerprint")
        != contract["root_public_key_fingerprint"]
        or sha256_bytes(bytes.fromhex(root["root_public_key"]))
        != root.get("root_public_key_fingerprint")
    ):
        raise G3ValidationError("invalid_reviewer_bootstrap_root")
    root_bytes = root.get("bytes")
    if not isinstance(root_bytes, bytes) or root.get("complete_file_sha256") != sha256_bytes(root_bytes):
        raise G3ValidationError("reviewer_bootstrap_root_hash_mismatch")
    if (
        proof.get("authority_kind") != contract["currentness_proof_authority_kind"]
        or proof.get("authority_class") != root.get("authority_class")
        or proof.get("stage") != REVIEWER_BOOTSTRAP_STAGE
        or proof.get("schema_version") != 1
        or proof.get("root_id") != root.get("root_id")
        or proof.get("root_sha256") != root.get("complete_file_sha256")
        or proof.get("currentness_proof_id")
        != reviewer_bootstrap_currentness_proof_id(proof)
        or proof.get("head_id") != reviewer_bootstrap_currentness_head_id(proof)
        or not isinstance(proof.get("sequence"), int)
        or proof.get("sequence") < 0
        or (proof.get("sequence") == 0 and proof.get("previous_proof_id") is not None)
        or (proof.get("sequence") > 0 and not isinstance(proof.get("previous_proof_id"), str))
        or not isinstance(proof.get("current_verifier_authority_id"), str)
        or not re.fullmatch(RUNTIME_STABLE_ID_PATTERN, proof["current_verifier_authority_id"])
        or not isinstance(proof.get("current_verifier_public_key"), str)
        or not re.fullmatch(ED25519_PUBLIC_KEY_PATTERN, proof["current_verifier_public_key"])
        or sha256_bytes(bytes.fromhex(proof["current_verifier_public_key"]))
        != proof.get("current_verifier_public_key_fingerprint")
        or proof.get("root_lifecycle") != root.get("lifecycle")
        or proof.get("root_revoked") is not False
        or proof.get("root_compromised") is not False
        or proof.get("root_superseded_by") is not None
        or proof.get("verifier_lifecycle") != "ACTIVE"
        or proof.get("verifier_revoked") is not False
        or proof.get("verifier_compromised") is not False
        or proof.get("verifier_superseded_by") is not None
        or proof.get("lifecycle") != "ACTIVE"
        or proof.get("stale") is not False
        or proof.get("superseded_by") is not None
        or proof.get("invalidated") is not False
    ):
        raise G3ValidationError("invalid_reviewer_bootstrap_currentness")
    for field_name in ("valid_from", "valid_until"):
        if not isinstance(proof.get(field_name), str) or not re.fullmatch(
            UTC_SECOND_TIMESTAMP_PATTERN, proof[field_name]
        ):
            raise G3ValidationError("malformed_reviewer_bootstrap_validity_window")
    try:
        valid_from = datetime.strptime(proof["valid_from"], "%Y-%m-%dT%H:%M:%SZ").replace(
            tzinfo=timezone.utc
        )
        valid_until = datetime.strptime(proof["valid_until"], "%Y-%m-%dT%H:%M:%SZ").replace(
            tzinfo=timezone.utc
        )
    except ValueError as error:
        raise G3ValidationError("malformed_reviewer_bootstrap_validity_window") from error
    now = datetime.now(timezone.utc)
    if not valid_from <= now <= valid_until:
        raise G3ValidationError("stale_reviewer_bootstrap_currentness")
    subject_index = _bootstrap_subject_index(proof)
    proof_bytes = proof.get("bytes")
    if not isinstance(proof_bytes, bytes) or proof.get("complete_file_sha256") != sha256_bytes(proof_bytes):
        raise G3ValidationError("reviewer_bootstrap_currentness_hash_mismatch")
    signature = proof.get("signature")
    signing_payload = {
        key: proof[key]
        for key in REVIEWER_BOOTSTRAP_CURRENTNESS_FIELDS
        if key != "signature"
    }
    if (
        not isinstance(signature, str)
        or not re.fullmatch(ED25519_SIGNATURE_PATTERN, signature)
        or not verify_ed25519_strict(
            root["root_public_key"],
            signature,
            REVIEWER_BOOTSTRAP_CURRENTNESS_DOMAIN
            + canonical_jcs_bytes(signing_payload),
        )
    ):
        raise G3ValidationError("invalid_reviewer_bootstrap_currentness_signature")
    return root, proof, subject_index


def _validate_authority_graph_root(context: G3AuthorityContext) -> None:
    if (
        not isinstance(context.authority_graph_bytes, bytes)
        or sha256_bytes(context.authority_graph_bytes) != context.authority_graph_sha256
    ):
        raise G3ValidationError("authority_graph_identity_mismatch")
    try:
        parsed = _parse_json_without_duplicates(context.authority_graph_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise G3ValidationError("authority_graph_bytes_malformed") from error
    if parsed != context.graph:
        raise G3ValidationError("authority_graph_identity_mismatch")
    inputs = context.objects.get("specification_bundle_inputs")
    if (
        not isinstance(inputs, dict)
        or inputs.get("authority_graph_sha256") != context.authority_graph_sha256
    ):
        raise G3ValidationError("authority_graph_binding_mismatch")


def _require_authority_object(
    context: G3AuthorityContext, node_id: str
) -> dict[str, Any]:
    record = context.objects.get(node_id)
    if record is None:
        raise G3ValidationError(f"missing_{node_id}")
    nodes = _graph_nodes(context.graph)
    expected_kind = nodes[node_id]["authority_kind"]
    if record.get("node_id") != node_id or record.get("authority_kind") != expected_kind:
        raise G3ValidationError(f"wrong_{node_id}_kind")
    if record.get("schema_version") != 1 or not _object_digest_matches(context, record):
        raise G3ValidationError(f"{node_id}_hash_mismatch")
    if record.get("lifecycle") != "ACTIVE" or record.get("stale") is not False:
        raise G3ValidationError(f"stale_{node_id}")
    if record.get("invalidated") is not False or record.get("superseded_by") is not None:
        raise G3ValidationError(f"superseded_{node_id}")
    return record


def _validate_independent_review_bundle(
    context: G3AuthorityContext, node_id: str, record: dict[str, Any]
) -> None:
    """Validate the exact inherited R11 review bundle at a graph node."""

    canonical_object = record.get("canonical_object", record)
    if not isinstance(canonical_object, dict) or set(canonical_object) != INDEPENDENT_REVIEW_BUNDLE_FIELDS:
        raise G3ValidationError(f"{node_id}_schema_mismatch")
    if canonical_object.get("schema_version") != 1:
        raise G3ValidationError(f"{node_id}_schema_mismatch")
    review_bundle_id = canonical_object.get("review_bundle_id")
    if (
        not isinstance(review_bundle_id, str)
        or not re.fullmatch(r"sha256:[0-9a-f]{64}", review_bundle_id)
        or review_bundle_id != independent_review_bundle_id(canonical_object)
    ):
        raise G3ValidationError(f"{node_id}_identity_mismatch")

    target = canonical_object.get("target")
    _validate_review_target(node_id, target, _review_bundle_target(context, node_id))

    rows = canonical_object.get("reviews")
    if not isinstance(rows, list) or len(rows) != len(REVIEW_ROLE_ORDER):
        raise G3ValidationError(f"{node_id}_role_coverage_incomplete")
    if [row.get("role") if isinstance(row, dict) else None for row in rows] != list(REVIEW_ROLE_ORDER):
        raise G3ValidationError(f"{node_id}_role_order_mismatch")

    scope_sha = _review_bundle_scope_sha(context, node_id)
    reviewer_ids: list[str] = []
    reviewer_actor_digests: list[str] = []
    artifact_ids: list[str] = []
    p0_total = 0
    p1_total = 0
    for row in rows:
        if not isinstance(row, dict) or set(row) != REVIEW_ROW_FIELDS:
            raise G3ValidationError(f"{node_id}_review_row_schema_mismatch")
        if row["decision"] not in {"GO", "NO-GO"}:
            raise G3ValidationError(f"{node_id}_review_decision_invalid")
        for count_name in ("p0_count", "p1_count"):
            value = row[count_name]
            if not isinstance(value, str) or not re.fullmatch(
                CANONICAL_UNSIGNED_INTEGER_PATTERN, value
            ):
                raise G3ValidationError(f"{node_id}_count_malformed")
        finding_ids = row["finding_ids"]
        if (
            not isinstance(finding_ids, list)
            or finding_ids != sorted(finding_ids)
            or len(finding_ids) != len(set(finding_ids))
            or any(
                not isinstance(finding_id, str)
                or not finding_id
                or not re.fullmatch(r"[A-Za-z0-9._:-]+", finding_id)
                for finding_id in finding_ids
            )
        ):
            raise G3ValidationError(f"{node_id}_finding_ids_malformed")
        reference = row["review_artifact_reference"]
        if not isinstance(reference, dict) or set(reference) != {
            "immutable_uri", "sha256", "byte_length"
        }:
            raise G3ValidationError(f"{node_id}_artifact_reference_schema_mismatch")
        uri = reference["immutable_uri"]
        if (
            not isinstance(uri, str)
            or not uri.startswith(REVIEW_ARTIFACT_URI_PREFIX)
            or not isinstance(reference["sha256"], str)
            or not re.fullmatch(r"[0-9a-f]{64}", reference["sha256"])
            or not isinstance(reference["byte_length"], str)
            or not re.fullmatch(CANONICAL_UNSIGNED_INTEGER_PATTERN, reference["byte_length"])
        ):
            raise G3ValidationError(f"{node_id}_artifact_reference_malformed")
        artifact_id = uri.removeprefix(REVIEW_ARTIFACT_URI_PREFIX)
        if not re.fullmatch(r"[0-9a-f]{64}", artifact_id):
            raise G3ValidationError(f"{node_id}_artifact_reference_malformed")
        artifact = context.review_artifacts.get(artifact_id)
        if artifact is None:
            raise G3ValidationError("unresolved_review_artifact_identity")
        if artifact.get("review_artifact_id") != artifact_id:
            raise G3ValidationError("review_artifact_identity_mismatch")
        if artifact.get("authority_kind") != "PhaseFReviewArtifactV1":
            raise G3ValidationError("wrong_review_artifact_kind")
        if artifact.get("authority_class") not in {"REAL", "TEST_ONLY"}:
            raise G3ValidationError("review_artifact_classification_malformed")
        if artifact.get("authority_class") == "TEST_ONLY" and context.mode == "real":
            raise G3ValidationError("synthetic_artifact_in_real_mode")
        if artifact.get("lifecycle") != "ACTIVE" or artifact.get("stale") is not False:
            raise G3ValidationError("stale_review_artifact")
        if artifact.get("superseded_by") is not None or artifact.get("invalidated") is not False:
            raise G3ValidationError("superseded_review_artifact")
        if artifact.get("role") != row["role"] or artifact.get("decision") != row["decision"]:
            raise G3ValidationError("review_artifact_binding_mismatch")
        if scope_sha is not None and artifact.get("reviewed_target") != scope_sha:
            raise G3ValidationError(f"{node_id}_artifact_target_mismatch")
        if artifact.get("p0_count") != row["p0_count"] or artifact.get("p1_count") != row["p1_count"]:
            raise G3ValidationError(f"{node_id}_artifact_count_mismatch")
        if not isinstance(artifact.get("p2_count"), str) or not re.fullmatch(
            CANONICAL_UNSIGNED_INTEGER_PATTERN, artifact["p2_count"]
        ):
            raise G3ValidationError("review_artifact_count_malformed")
        if artifact.get("finding_ids") != finding_ids:
            raise G3ValidationError(f"{node_id}_artifact_findings_mismatch")
        if artifact.get("independence_relation") != {
            "type": "distinct_reviewer_authority",
            "reviewer_authority_id": artifact.get("reviewer_authority_id"),
        }:
            raise G3ValidationError("non_independent_review")
        reviewer_id = artifact.get("reviewer_authority_id")
        reviewer = context.reviewer_authorities.get(reviewer_id)
        if reviewer is None:
            raise G3ValidationError("unresolved_reviewer_identity")
        if reviewer.get("authority_kind") != "PhaseFReviewerIdentityV1":
            raise G3ValidationError("wrong_reviewer_identity_kind")
        if reviewer.get("authority_class") not in {"REAL", "TEST_ONLY"}:
            raise G3ValidationError("reviewer_classification_malformed")
        if reviewer.get("authority_class") == "TEST_ONLY" and context.mode == "real":
            raise G3ValidationError("synthetic_reviewer_in_real_mode")
        if artifact.get("authority_class") != reviewer.get("authority_class"):
            raise G3ValidationError("review_reference_classification_mismatch")
        if reviewer.get("permitted_review_roles") != [row["role"]]:
            raise G3ValidationError("reviewer_role_not_permitted")
        actor_digest = reviewer.get("actor_identity_digest")
        if not isinstance(actor_digest, str) or not re.fullmatch(r"[0-9a-f]{64}", actor_digest):
            raise G3ValidationError("reviewer_identity_digest_malformed")
        if (
            context.mode in {"real", "real_test"}
            and context.remediation_actor_identity_digest is not None
            and actor_digest == context.remediation_actor_identity_digest
        ):
            raise G3ValidationError("non_independent_review")
        if reviewer.get("lifecycle") != "ACTIVE" or reviewer.get("stale") is not False:
            raise G3ValidationError("stale_reviewer_identity")
        if reviewer.get("superseded_by") is not None or reviewer.get("invalidated") is not False:
            raise G3ValidationError("superseded_reviewer_identity")
        _validate_reviewer_actor_binding(context, reviewer, row["role"])
        if not isinstance(reviewer_id, str) or not re.fullmatch(r"[0-9a-f]{64}", reviewer_id):
            raise G3ValidationError("reviewer_identity_malformed")
        reviewer_ids.append(reviewer_id)
        reviewer_actor_digests.append(actor_digest)
        artifact_ids.append(artifact_id)
        if context.mode == "synthetic":
            if reference["sha256"] != artifact.get("reference_sha256") or reference["byte_length"] != artifact.get("reference_byte_length"):
                raise G3ValidationError(f"{node_id}_artifact_reference_mismatch")
        else:
            artifact_bytes = artifact.get("bytes")
            if not isinstance(artifact_bytes, bytes) or reference["sha256"] != sha256_bytes(artifact_bytes) or reference["byte_length"] != str(len(artifact_bytes)):
                raise G3ValidationError(f"{node_id}_artifact_reference_mismatch")
        p0_total += int(row["p0_count"])
        p1_total += int(row["p1_count"])

    if (
        len(set(reviewer_ids)) != len(reviewer_ids)
        or len(set(reviewer_actor_digests)) != len(reviewer_actor_digests)
        or len(set(artifact_ids)) != len(artifact_ids)
    ):
        raise G3ValidationError("non_independent_review")
    if canonical_object["aggregate_p0_count"] != str(p0_total) or canonical_object["aggregate_p1_count"] != str(p1_total):
        raise G3ValidationError(f"{node_id}_aggregate_count_mismatch")
    if not all(row["decision"] == "GO" for row in rows) or p0_total != 0 or p1_total != 0:
        expected_decision = "NO-GO"
    else:
        expected_decision = "GO"
    if canonical_object["aggregate_decision"] != expected_decision:
        raise G3ValidationError(f"{node_id}_aggregate_decision_mismatch")


def _validate_graph_object_bindings(context: G3AuthorityContext) -> None:
    """Validate builder/validator objects against the edge-root projection."""
    graph = context.graph
    nodes = _graph_nodes(graph)
    edges = _graph_edges(graph, nodes)
    binding_contract = derive_serialized_binding_contract(graph, nodes, edges)

    for target in nodes:
        record = context.objects.get(target)
        if record is None:
            continue
        for edge_type in ("binds", "generated_from"):
            incoming = [edge for edge in edges if edge["to"] == target and edge["type"] == edge_type]
            if not incoming:
                continue
            field_name = _binding_field(binding_contract, target, edge_type, "*")
            if field_name is None:
                continue
            actual = record.get(field_name)
            expected_sources = sorted(edge["from"] for edge in incoming)
            if field_name in {"authority_bindings", "bound_authority_sha256s"}:
                if not isinstance(actual, dict) or sorted(actual) != expected_sources:
                    raise G3ValidationError(f"{target}_binding_set_mismatch")
                for source in expected_sources:
                    source_record = context.objects.get(source)
                    if source_record is None:
                        raise G3ValidationError(f"missing_{source}")
                    if field_name == "authority_bindings":
                        if actual[source] != _authority_descriptor(source_record):
                            raise G3ValidationError(f"{target}_{source}_binding_mismatch")
                    elif actual[source] != source_record.get("sha256"):
                        raise G3ValidationError(f"{target}_{source}_binding_mismatch")
            elif field_name == "generated_source_sha256s":
                if not isinstance(actual, dict) or sorted(actual) != expected_sources:
                    raise G3ValidationError(f"{target}_binding_set_mismatch")
                for source in expected_sources:
                    source_record = context.objects.get(source)
                    if source_record is None or actual[source] != source_record.get("sha256"):
                        raise G3ValidationError(f"{target}_{source}_binding_mismatch")

    for target, relations in binding_contract.items():
        target_record = context.objects.get(target)
        if target_record is None:
            continue
        for edge_type, sources in relations.items():
            for source in sources:
                if source == "*" or edge_type in {"binds", "generated_from"}:
                    continue
                field_name = _binding_field(binding_contract, target, edge_type, source)
                if field_name is None:
                    continue
                source_record = context.objects.get(source)
                if source_record is None:
                    raise G3ValidationError(f"missing_{source}")
                if field_name == "target" and _is_independent_review_bundle(target_record):
                    expected = _review_bundle_target(context, target)
                else:
                    expected = source_record.get("sha256")
                if target_record.get(field_name) != expected:
                    raise G3ValidationError(f"{target}_{source}_binding_mismatch")

    for target, relation_types in binding_contract.items():
        target_record = context.objects.get(target)
        if target_record is None:
            continue
        for edge_type in ("reviews", "targets", "approves", "requires"):
            incoming = [
                edge for edge in edges if edge["to"] == target and edge["type"] == edge_type
            ]
            for edge in incoming:
                field_name = _binding_field(binding_contract, target, edge_type, edge["from"])
                if field_name is None:
                    continue
                source_record = context.objects.get(edge["from"])
                if source_record is None:
                    raise G3ValidationError(f"missing_{edge['from']}")
                if field_name == "target" and _is_independent_review_bundle(target_record):
                    expected = _review_bundle_target(context, target)
                else:
                    expected = source_record.get("sha256")
                if target_record.get(field_name) != expected:
                    raise G3ValidationError(
                        f"{target}_{edge['from']}_{edge_type}_binding_mismatch"
                    )


def _validate_available_direct_review_bundles(
    context: G3AuthorityContext,
) -> None:
    """Validate every direct review bundle already resolved in this context."""

    for node_id in sorted(REVIEW_BUNDLE_NODES):
        record = context.objects.get(node_id)
        if record is not None:
            _validate_independent_review_bundle(context, node_id, record)


def _validate_migrated_review(
    context: G3AuthorityContext, record: dict[str, Any]
) -> None:
    required_fields = {
        "migrated_finding_review_id",
        "target_git_commit",
        "target_bundle_inputs_sha256",
        "reviewed_migration_ledger_sha256",
        "reviewed_normative_traceability_matrix_sha256",
        "reviewed_traceability_manifest_sha256",
        "reviewed_component_sha256s",
        "reviewed_finding_ids",
        "finding_dispositions",
        "reviewer_roles",
        "review_records",
        "review_input_fingerprint",
        "p0_count",
        "p1_count",
        "p2_count",
        "decision",
        "created_stage",
        "producer",
        "validator",
        "lifecycle",
        "stale",
        "superseded_by",
        "invalidated",
    }
    if any(field not in record for field in required_fields):
        raise G3ValidationError("migrated_review_schema_incomplete")
    if (
        record.get("migrated_finding_review_id") != record.get("sha256")
        or record.get("validator") != "validate_migrated_finding_review"
        or record.get("created_stage") != 10
    ):
        raise G3ValidationError("migrated_review_schema_mismatch")
    if context.mode in {"real", "real_test"}:
        canonical_object = record.get("canonical_object")
        if not isinstance(canonical_object, dict) or any(
            canonical_object.get(field) != record.get(field) for field in required_fields
        ):
            raise G3ValidationError("migrated_review_identity_mismatch")
        if (
            not isinstance(context.remediation_authority_id, str)
            or not re.fullmatch(r"[0-9a-f]{64}", context.remediation_authority_id)
            or not isinstance(context.remediation_actor_identity_digest, str)
            or not re.fullmatch(
                r"[0-9a-f]{64}", context.remediation_actor_identity_digest
            )
        ):
            raise G3ValidationError("unresolved_remediation_authority")
    if record.get("target_git_commit") != context.expected_target_commit:
        raise G3ValidationError("migrated_review_target_commit_mismatch")
    if record.get("target_bundle_inputs_sha256") != context.objects[
        "specification_bundle_inputs"
    ].get("sha256"):
        raise G3ValidationError("migrated_review_target_mismatch")
    if record.get("reviewed_migration_ledger_sha256") != context.objects[
        "migration_ledger"
    ].get("sha256"):
        raise G3ValidationError("migrated_review_ledger_mismatch")
    if record.get("reviewed_normative_traceability_matrix_sha256") != context.objects[
        "normative_traceability_matrix"
    ].get("sha256"):
        raise G3ValidationError("migrated_review_normative_matrix_mismatch")
    if record.get("reviewed_traceability_manifest_sha256") != context.objects[
        "generated_traceability_manifest"
    ].get("sha256"):
        raise G3ValidationError("migrated_review_traceability_mismatch")
    if record.get("reviewed_component_sha256s") != sorted(context.component_sha256s):
        raise G3ValidationError("migrated_review_component_mismatch")
    if record.get("reviewed_finding_ids") != sorted(EXPECTED_MIGRATED_FINDINGS):
        raise G3ValidationError("incomplete_migrated_finding_coverage")
    dispositions = record.get("finding_dispositions")
    if (
        not isinstance(dispositions, dict)
        or set(dispositions) != EXPECTED_MIGRATED_FINDINGS
        or any(
            not isinstance(value, str) or value not in MIGRATED_FINDING_DISPOSITIONS
            for value in dispositions.values()
        )
    ):
        raise G3ValidationError("invalid_migrated_finding_disposition")
    if record.get("review_input_fingerprint") != _migrated_review_input_fingerprint(record):
        raise G3ValidationError("migrated_review_input_fingerprint_mismatch")
    review_records = record.get("review_records")
    if (
        not isinstance(review_records, list)
        or len(review_records) != len(REVIEW_ROLES)
        or any(
            not isinstance(row, dict) or set(row) != MIGRATED_REVIEW_RECORD_FIELDS
            for row in review_records
        )
    ):
        raise G3ValidationError("migrated_review_review_record_schema_mismatch")
    roles = [row["role"] for row in review_records]
    reviewer_ids = [row["reviewer_authority_id"] for row in review_records]
    artifact_ids = [row["review_artifact_id"] for row in review_records]
    review_hashes = [row["review_sha256"] for row in review_records]
    if (
        sorted(roles) != sorted(REVIEW_ROLES)
        or len(set(roles)) != len(roles)
        or any(not isinstance(value, str) or not value for value in reviewer_ids)
        or len(set(reviewer_ids)) != len(reviewer_ids)
        or any(not isinstance(value, str) or not value for value in artifact_ids)
        or len(set(artifact_ids)) != len(artifact_ids)
        or len(set(review_hashes)) != len(review_hashes)
        or record.get("reviewer_roles") != sorted(roles)
    ):
        raise G3ValidationError("non_independent_migrated_review")
    artifact_digests: list[str] = []
    for row in review_records:
        if row["reviewed_target"] != record["review_input_fingerprint"]:
            raise G3ValidationError("migrated_review_review_target_mismatch")
        if row["decision"] not in {"GO", "NO-GO"}:
            raise G3ValidationError("migrated_review_review_decision_invalid")
        if row["lifecycle"] != "ACTIVE":
            raise G3ValidationError("stale_migrated_review_record")
        if (
            not isinstance(row["review_sha256"], str)
            or not re.fullmatch(r"[0-9a-f]{64}", row["review_sha256"])
            or row["independence_relation"]
            != {
                "type": "distinct_reviewer_authority",
                "reviewer_authority_id": row["reviewer_authority_id"],
            }
        ):
            raise G3ValidationError("non_independent_migrated_review")
        row_payload = dict(row)
        row_payload.pop("review_sha256")
        if sha256_bytes(canonical_json_bytes(row_payload)) != row["review_sha256"]:
            raise G3ValidationError("migrated_review_review_hash_mismatch")
        _validate_review_reference(context, row)
        artifact_digests.append(
            context.review_artifacts[row["review_artifact_id"]].get("sha256")
        )
    _require_distinct_reviewer_actor_digests(
        context, reviewer_ids, "non_independent_migrated_review"
    )
    if len(set(artifact_digests)) != len(artifact_digests):
        raise G3ValidationError("non_independent_migrated_review")
    if record.get("producer") != "independent_review_panel":
        raise G3ValidationError("non_independent_migrated_review")
    if any(not isinstance(record.get(name), int) or record[name] < 0 for name in ("p0_count", "p1_count", "p2_count")):
        raise G3ValidationError("malformed_migrated_review_counts")
    expected_counts, unresolved = _disposition_counts(dispositions)
    if any(record[name] != value for name, value in expected_counts.items()):
        raise G3ValidationError("migrated_review_count_disposition_mismatch")
    review_rows_go = all(row["decision"] == "GO" for row in review_records)
    expected_decision = "NO-GO" if (
        unresolved
        or expected_counts["p0_count"]
        or expected_counts["p1_count"]
        or not review_rows_go
    ) else (
        "GO_WITH_DOCUMENTED_NON_BLOCKING_DEBT"
        if expected_counts["p2_count"]
        else "GO"
    )
    if record.get("decision") != expected_decision:
        raise G3ValidationError("migrated_review_not_go")


def _validate_review_collection(
    context: G3AuthorityContext, fields: dict[str, str]
) -> None:
    graph_nodes = _graph_nodes(context.graph)
    graph_edges = _graph_edges(context.graph, graph_nodes)
    component_nodes = sorted(
        edge["from"]
        for edge in graph_edges
        if edge["to"] == "g3_approval_tag"
        and edge["type"] == "requires"
        and edge["from"].startswith("component_")
        and edge["from"].endswith("_review")
    )
    if not component_nodes:
        raise G3ValidationError("component_review_set_mismatch")
    for node_id in component_nodes:
        record = _require_authority_object(context, node_id)
        _validate_independent_review_bundle(context, node_id, record)
        if record.get("aggregate_decision") != "GO":
            raise G3ValidationError(f"{node_id}_not_go")
        spec_node = node_id.removesuffix("_review") + "_spec"
        if spec_node not in graph_nodes or _review_bundle_scope_sha(context, node_id) != context.component_sha_by_node[spec_node]:
            raise G3ValidationError(f"wrong_{node_id}_target")

    architecture_review = _require_authority_object(context, "architecture_review")
    _validate_independent_review_bundle(context, "architecture_review", architecture_review)
    if architecture_review.get("aggregate_decision") != "GO":
        raise G3ValidationError("architecture_review_not_go")
    f0_review = _require_authority_object(context, "f0_review")
    _validate_independent_review_bundle(context, "f0_review", f0_review)
    if f0_review.get("aggregate_decision") != "GO":
        raise G3ValidationError("f0_review_not_go")

    migrated = _require_authority_object(context, "migrated_finding_review")
    _validate_migrated_review(context, migrated)
    if migrated.get("decision") != "GO":
        raise G3ValidationError("migrated_review_not_go")

    aggregate = _require_authority_object(context, "aggregate_review")
    _validate_independent_review_bundle(context, "aggregate_review", aggregate)
    if aggregate.get("aggregate_decision") != "GO":
        raise G3ValidationError("aggregate_review_not_go")
    required_dependencies = context.graph.get("aggregate_review_dependency_nodes")
    if (
        not isinstance(required_dependencies, list)
        or any(not isinstance(node_id, str) for node_id in required_dependencies)
        or len(required_dependencies) != len(set(required_dependencies))
        or any(node_id not in graph_nodes for node_id in required_dependencies)
        or set(required_dependencies) != set(component_nodes) | {
            "migrated_finding_review",
            "specification_bundle_manifest",
            "generated_traceability_manifest",
        }
    ):
        raise G3ValidationError("aggregate_dependency_closure_incomplete")
    if _review_bundle_scope_sha(context, "aggregate_review") != context.objects[
        "specification_bundle_manifest"
    ].get("sha256"):
        raise G3ValidationError("aggregate_target_mismatch")
    if aggregate.get("sha256") != fields["aggregate_review_bundle_sha256"]:
        raise G3ValidationError("aggregate_hash_mismatch")


def validate_g3_tag(
    tag_name: str, body_bytes: bytes, context: G3AuthorityContext
) -> dict[str, str]:
    """Validate G3 wire bytes and the complete real/synthetic authority closure."""

    fields = parse_g3_tag(tag_name, body_bytes)
    if not isinstance(context, G3AuthorityContext):
        raise G3ValidationError("invalid_validation_context")
    if context.mode not in {"synthetic", "real", "real_test"}:
        raise G3ValidationError("invalid_validation_context_mode")
    if context.mode in {"synthetic", "real_test"} and context.real_authority_requested:
        raise G3ValidationError("synthetic_cannot_authorize_real")

    _validate_authority_graph_root(context)

    tag = context.tag
    if tag.get("exists") is not True:
        raise G3ValidationError("missing_real_g3_tag")
    if tag.get("annotated") is not True or tag.get("object_type") != "tag":
        raise G3ValidationError("lightweight_tag")
    if tag.get("peeled_commit") != context.expected_target_commit:
        raise G3ValidationError("g3_target_mismatch")
    if tag.get("message") != body_bytes:
        raise G3ValidationError("g3_message_mismatch")
    if context.bundle_manifest_sha256 is None:
        raise G3ValidationError("missing_specification_bundle_manifest")
    if context.aggregate_review_sha256 is None:
        raise G3ValidationError("missing_aggregate_review")
    if fields["specification_bundle_manifest_sha256"] != context.bundle_manifest_sha256:
        raise G3ValidationError("wrong_bundle_hash")
    if fields["aggregate_review_bundle_sha256"] != context.aggregate_review_sha256:
        raise G3ValidationError("wrong_aggregate_review_hash")

    try:
        graph_audit = validate_r12_authority_graph(context.graph)
    except ValueError as error:
        raise G3ValidationError("invalid_authority_graph") from error
    if graph_audit["g3_required_count"] != len(context.graph["g3_required_nodes"]):
        raise G3ValidationError("invalid_authority_graph")
    for node_id in context.graph["g3_required_nodes"]:
        _require_authority_object(context, node_id)
    _validate_graph_object_bindings(context)
    _validate_available_direct_review_bundles(context)

    bundle_inputs = _require_authority_object(context, "specification_bundle_inputs")
    manifest = _require_authority_object(context, "specification_bundle_manifest")
    if manifest.get("sha256") != fields["specification_bundle_manifest_sha256"]:
        raise G3ValidationError("wrong_bundle_hash")
    if manifest.get("status") != "READY_FOR_G3" or manifest.get("eligible_for_g3") is not True:
        raise G3ValidationError("manifest_not_eligible")
    if manifest.get("target_commit") != context.expected_target_commit:
        raise G3ValidationError("manifest_target_mismatch")
    if manifest.get("bundle_input_fingerprint_sha256") != bundle_inputs.get("sha256"):
        raise G3ValidationError("manifest_input_binding_mismatch")

    architecture = _require_authority_object(context, "architecture_approval")
    if architecture.get("decision") != "GO" or architecture.get("p0_count") != 0 or architecture.get("p1_count") != 0:
        raise G3ValidationError("architecture_approval_not_go")
    if architecture.get("tag_name") != fields["phase_f_architecture_plan_tag"]:
        raise G3ValidationError("wrong_architecture_plan_binding")
    if architecture.get("target_sha256") != context.architecture_plan_sha256:
        raise G3ValidationError("wrong_architecture_plan_target")
    f0 = _require_authority_object(context, "f0_approval")
    if f0.get("decision") != "GO" or f0.get("p0_count") != 0 or f0.get("p1_count") != 0:
        raise G3ValidationError("f0_approval_not_go")
    if f0.get("tag_name") != fields["phase_f_f0_decisions_tag"]:
        raise G3ValidationError("wrong_f0_decisions_binding")
    if f0.get("target_sha256") != context.f0_decisions_sha256:
        raise G3ValidationError("wrong_f0_target")
    _validate_review_collection(context, fields)
    return fields


def check_g3_kat(tag_name: str, body: bytes) -> dict[str, object]:
    try:
        context = make_synthetic_context()
        context.tag["message"] = body
        fields = validate_g3_tag(tag_name, body, context)
    except G3ValidationError as error:
        return {"result": "REJECT", "category": error.category}
    return {"result": "PASS", "category": "valid", "decoded_fields": fields}


def apply_g3_mutation(mutation_id: str) -> tuple[str, bytes]:
    body = G3_FIXTURE_BODY
    tag_name = G3_TAG_NAME
    replacements = {
        "R12-NEG-G3-WRONG-FIELD-NAME": (
            b"phase_f_architecture_plan_tag=",
            b"phase_f_architecture_plan=",
        ),
        "R12-NEG-G3-LEGACY-FIELD-NAME": (
            b"phase_f_architecture_plan_tag=",
            b"architecture_plan_tag=",
        ),
        "R12-NEG-G3-SCHEMA-VERSION": (b"schema_version=1\n", b"schema_version=2\n"),
        "R12-NEG-G3-WRONG-ARCHITECTURE-BINDING": (
            b"phase_f_architecture_plan_tag=ism-mechanism-health-v1-f-plan-approved",
            b"phase_f_architecture_plan_tag=ism-mechanism-health-v1-f-f0-decisions-approved",
        ),
        "R12-NEG-G3-WRONG-F0-BINDING": (
            b"phase_f_f0_decisions_tag=ism-mechanism-health-v1-f-f0-decisions-approved",
            b"phase_f_f0_decisions_tag=ism-mechanism-health-v1-f-plan-approved",
        ),
        "R12-NEG-G3-TRAILING-WHITESPACE": (
            b"approval_decision=GO\n",
            b"approval_decision=GO \n",
        ),
        "R12-NEG-G3-MISSING-DELIMITER": (
            b"phase_f_architecture_plan_tag=",
            b"phase_f_architecture_plan_tag ",
        ),
        "R12-NEG-G3-INVALID-NEWLINE": (
            b"ism-mechanism-health-v1-f-plan-approved\n",
            b"ism-mechanism-health-v1-f-plan-approved\r\n",
        ),
        "R12-NEG-G3-WRONG-APPROVAL-VALUE": (
            b"approval_decision=GO\n",
            b"approval_decision=NO-GO\n",
        ),
    }
    if mutation_id in replacements:
        old, new = replacements[mutation_id]
        body = body.replace(old, new, 1)
    elif mutation_id == "R12-NEG-G3-MISSING-REQUIRED-FIELD":
        line = b"aggregate_review_bundle_sha256=" + b"1" * 64 + b"\n"
        body = body.replace(line, b"", 1)
    elif mutation_id == "R12-NEG-G3-DUPLICATE-FIELD":
        body = body.replace(
            b"schema_version=1\n", b"approval_decision=GO\nschema_version=1\n", 1
        )
    elif mutation_id == "R12-NEG-G3-UNEXPECTED-FIELD":
        body = body.replace(b"schema_version=1\n", b"unexpected_field=x\n", 1)
    elif mutation_id == "R12-NEG-G3-WRONG-LINE-ORDER":
        lines = body.splitlines(keepends=True)
        lines[0], lines[1] = lines[1], lines[0]
        body = b"".join(lines)
    elif mutation_id == "R12-NEG-G3-MALFORMED-TAG-NAME":
        tag_name = "ism-mechanism-health-v1-f-specification-bundl-approved"
    elif mutation_id == "R12-NEG-G3-WRONG-BUNDLE-HASH":
        body = body.replace(
            b"specification_bundle_manifest_sha256=" + b"0" * 64,
            b"specification_bundle_manifest_sha256=a" + b"0" * 63,
            1,
        )
    elif mutation_id == "R12-NEG-G3-MALFORMED-SHA":
        body = body.replace(
            b"aggregate_review_bundle_sha256=" + b"1" * 64,
            b"aggregate_review_bundle_sha256=z" + b"1" * 63,
            1,
        )
    elif mutation_id == "R12-NEG-G3-EXTRA-TRAILING-CONTENT":
        body += b"trailing\n"
    elif mutation_id == "R12-NEG-G3-TRUNCATED-CONTENT":
        body = body[:-10]
    elif mutation_id == "R12-NEG-G3-MISSING-FINAL-NEWLINE":
        body = body[:-1]
    else:
        raise ValueError(f"unknown G3 mutation: {mutation_id}")
    return tag_name, body


def parse_pipe_row(line: str) -> list[str]:
    if not line.startswith("|") or not line.rstrip().endswith("|"):
        raise ValueError(f"invalid catalog row: {line}")
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def parse_r11_test_catalog() -> dict[str, dict[str, str]]:
    text = R11_SOURCE.read_text()
    section = text.split("### 53.10", 1)[1].split("### 53.11", 1)[0]
    catalog: dict[str, dict[str, str]] = {}
    for line in section.splitlines():
        if not line.startswith("| R11-"):
            continue
        cells = parse_pipe_row(line)
        if len(cells) != 9:
            raise ValueError(f"R11 test catalog column count: {line}")
        test_id = cells[0]
        if test_id in catalog:
            raise ValueError(f"duplicate test ID: {test_id}")
        if any(not cell for cell in cells):
            raise ValueError(f"blank R11 test catalog cell: {test_id}")
        catalog[test_id] = {
            "kat_class": cells[1],
            "fixture_scope": cells[2],
            "expected_result": cells[6],
        }
    if len(catalog) != EXPECTED_R11_TEST_COUNT:
        raise ValueError(f"R11 test catalog count: {len(catalog)}")
    return catalog


def parse_r11_evidence_catalog() -> dict[str, dict[str, str]]:
    text = R11_SOURCE.read_text()
    section = text.split("### 53.11", 1)[1].split("### 53.12", 1)[0]
    catalog: dict[str, dict[str, str]] = {}
    for line in section.splitlines():
        if not line.startswith("| EV11-"):
            continue
        cells = parse_pipe_row(line)
        if len(cells) != 5:
            raise ValueError(f"R11 evidence catalog column count: {line}")
        evidence_id = cells[0]
        if evidence_id in catalog:
            raise ValueError(f"duplicate evidence ID: {evidence_id}")
        if any(not cell for cell in cells):
            raise ValueError(f"blank R11 evidence catalog cell: {evidence_id}")
        catalog[evidence_id] = {"artifact": cells[1], "oracle": cells[4]}
    if len(catalog) != EXPECTED_R11_EVIDENCE_COUNT:
        raise ValueError(f"R11 evidence catalog count: {len(catalog)}")
    return catalog


def parse_r12_test_catalog(text: str | None = None) -> dict[str, dict[str, str]]:
    text = SPECS["F-CNF"].read_text() if text is None else text
    section = text.split("## 3. Current executable catalog", 1)[1].split(
        "### 3.1", 1
    )[0]
    catalog: dict[str, dict[str, str]] = {}
    for line in section.splitlines():
        if not line.startswith("| R12-"):
            continue
        cells = parse_pipe_row(line)
        if len(cells) != 9:
            raise ValueError(f"R12 test catalog column count: {line}")
        test_id = cells[0]
        if test_id in catalog:
            raise ValueError(f"duplicate test ID: {test_id}")
        if any(not cell for cell in cells):
            raise ValueError(f"blank R12 test catalog cell: {test_id}")
        catalog[test_id] = {
            "kat_class": cells[1],
            "fixture_scope": cells[2],
            "expected_result": cells[6],
        }
    if set(catalog) != EXPECTED_R12_TEST_CATALOG_IDS:
        raise ValueError(f"R12 test catalog set: {sorted(catalog)}")
    row = catalog["R12-POS-SPEC-BUNDLE-TAG"]
    if row != {
        "kat_class": "literal_kat",
        "fixture_scope": "g3_specification_bundle_tag",
        "expected_result": "PASS with exact decoded fields",
    }:
        raise ValueError(f"R12 test catalog metadata: {row}")
    positive_ids = {
        "R12-POS-SPEC-BUNDLE-TAG",
        "R12-G3-AUTHORITY-CONTEXT-POS",
        "R12-G3-ARCHITECTURE-REVIEW-BUNDLE-POSITIVE",
        "R12-G3-REVIEW-START-GIT-PUBLISHED",
        "R12-G3-REAL-FORMAT-POSITIVE",
        "R12-G3-REAL-ACTOR-ATTESTATION-POSITIVE",
        "R12-DAG-VALID",
    }
    for test_id in EXPECTED_R12_TEST_CATALOG_IDS - positive_ids:
        if catalog[test_id]["kat_class"] != "constructive_plan_audit":
            raise ValueError(f"R12 constructive test category: {test_id}")
        if catalog[test_id]["expected_result"] != "REJECT":
            raise ValueError(f"R12 constructive test result: {test_id}")
    return catalog


def load_reference_catalogs() -> tuple[dict[str, dict[str, str]], dict[str, dict[str, str]]]:
    tests = parse_r11_test_catalog()
    r12_tests = parse_r12_test_catalog()
    if set(tests).intersection(r12_tests):
        raise ValueError("R11/R12 test catalog ID collision")
    tests.update(r12_tests)
    evidence = parse_r11_evidence_catalog()
    return tests, evidence


def load_normative_matrix() -> list[dict[str, Any]]:
    try:
        matrix = json.loads(NORMATIVE_MATRIX_PATH.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError("R12 normative traceability matrix is unreadable") from error
    if (
        matrix.get("schema_version") != 1
        or matrix.get("artifact_kind") != "phase_f_r12_normative_traceability_matrix"
        or matrix.get("authority_status") != "NORMATIVE_CANDIDATE"
    ):
        raise ValueError("R12 normative traceability matrix metadata mismatch")
    rows = matrix.get("requirements")
    if not isinstance(rows, list) or len(rows) != EXPECTED_R12_REQUIREMENT_COUNT:
        raise ValueError("R12 normative matrix requirement count mismatch")
    required_fields = {
        "requirement_id",
        "authority_document",
        "authority_anchor",
        "upstream_requirement_ids",
        "f0_decision_dependencies",
        "validation_category",
        "expected_lifecycle_stage",
        "test_ids",
        "kat_ids",
        "constructive_audit_ids",
        "property_test_ids",
        "future_real_evidence_ids",
        "schema_ids",
    }
    ids: set[str] = set()
    for row in rows:
        if not isinstance(row, dict) or set(row) != required_fields:
            raise ValueError("R12 normative matrix row closure mismatch")
        requirement_id = row["requirement_id"]
        if not isinstance(requirement_id, str) or requirement_id in ids:
            raise ValueError(f"duplicate R12 normative matrix requirement: {requirement_id}")
        ids.add(requirement_id)
        for field in required_fields - {"requirement_id", "authority_document", "authority_anchor", "validation_category", "expected_lifecycle_stage"}:
            value = row[field]
            if not isinstance(value, list) or len(value) != len(set(value)) or any(not isinstance(item, str) or not item for item in value):
                raise ValueError(f"R12 normative matrix list closure: {requirement_id}/{field}")
        partition = set(row["kat_ids"]) | set(row["constructive_audit_ids"]) | set(row["property_test_ids"])
        if set(row["test_ids"]) != partition or len(partition) != len(row["test_ids"]):
            raise ValueError(f"R12 normative matrix test partition mismatch: {requirement_id}")
    expected_ids = set(EXPECTED_ARCHITECTURE_IDS) | {
        requirement_id for ids_for_prefix in EXPECTED_SPEC_IDS.values() for requirement_id in ids_for_prefix
    }
    if ids != expected_ids:
        raise ValueError(f"R12 normative matrix ID set mismatch: {sorted(ids)}")
    return rows


def validate_wire_catalog() -> None:
    wire_text = SPECS["F-WIRE"].read_text()
    grammar = "\n".join(
        [
            "phase_f_architecture_plan_tag=<annotated tag name>",
            "phase_f_f0_decisions_tag=<annotated tag name>",
            "specification_bundle_manifest_sha256=<SHA256_V1>",
            "aggregate_review_bundle_sha256=<SHA256_V1>",
            "approval_decision=GO",
            "schema_version=1",
        ]
    ) + "\n"
    if wire_text.count(grammar) != 1:
        raise ValueError("G3 grammar is missing or duplicated")

    section = wire_text.split("## 4. Current R12 schema catalog closure", 1)[1].split(
        "## 6. Review gate", 1
    )[0]
    rows = {}
    for line in section.splitlines():
        if not line.startswith("| PhaseF"):
            continue
        cells = parse_pipe_row(line)
        if len(cells) != 9:
            raise ValueError(f"schema catalog column count: {line}")
        identifier = cells[0]
        if identifier in rows:
            raise ValueError(f"duplicate schema catalog row: {identifier}")
        if any(not cell for cell in cells):
            raise ValueError(f"blank schema catalog cell: {identifier}")
        rows[identifier] = cells

    inherited = parse_schema_catalog_ids(R11_SOURCE.read_text())
    expected = set(inherited) | R12_SCHEMA_IDS
    if len(inherited) != 91 or set(rows) != R12_SCHEMA_IDS:
        raise ValueError(
            f"R12 schema catalog delta mismatch: inherited={len(inherited)}, rows={sorted(rows)}"
        )
    if len(expected) != EXPECTED_R12_SCHEMA_COUNT:
        raise ValueError(f"R12 schema set count: {len(expected)}")
    expected_row = [
        "PhaseFSpecificationBundleApprovalV1",
        "TAG_BODY",
        "#schema-def-PhaseFSpecificationBundleApprovalV1",
        "no JSON semantic ID; SHA-256 of the exact six-line annotated tag-message bytes including the final LF",
        "independent five-role specification-bundle approval gate",
        "exact §3 tag-name/body parser plus target, architecture approval, F0 approval, five component-review, traceability, migrated-finding, aggregate-review, and `approval_decision=GO` validator",
        "G3 specification-bundle approval, after architecture/F0 approvals and all five component reviews",
        "TAG_BODY; Git annotated-tag message only; no registry subject and no registry record",
        "INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFSpecificationBundleApprovalV1)",
    ]
    if rows["PhaseFSpecificationBundleApprovalV1"] != expected_row:
        raise ValueError("R12 schema catalog metadata mismatch")
    anchor = '<a id="schema-def-PhaseFSpecificationBundleApprovalV1"></a>'
    if wire_text.count(anchor) != 1:
        raise ValueError("R12 schema definition anchor missing or duplicated")
    migrated_row = rows["PhaseFMigratedFindingReviewV1"]
    if migrated_row != [
        "PhaseFMigratedFindingReviewV1",
        "TOP_LEVEL_WIRE",
        "#schema-def-PhaseFMigratedFindingReviewV1",
        "no registry subject before G3; SHA-256 of the complete canonical review object excluding its own ID field",
        "independent migrated-finding review panel",
        "strict migrated-review schema, closed finding-disposition/count/decision validator, exact bundle-input target, concrete five-role review records and independence, lifecycle, staleness, and hash validator",
        "G2 review prerequisite for the specification bundle",
        "external authority object; registry publication is prohibited before later gate authority",
        "INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMigratedFindingReviewV1)",
    ]:
        raise ValueError("migrated-finding schema catalog metadata mismatch")
    migrated_anchor = '<a id="schema-def-PhaseFMigratedFindingReviewV1"></a>'
    if wire_text.count(migrated_anchor) != 1:
        raise ValueError("migrated-finding schema definition anchor missing or duplicated")
    actor_row = rows["PhaseFReviewerActorAttestationV1"]
    if actor_row != [
        "PhaseFReviewerActorAttestationV1",
        "SIGNED_EXTERNAL_AUTHORITY",
        "#schema-def-PhaseFReviewerActorAttestationV1",
        "sha256:<lowercase_hex>; SHA-256 of the domain-separated JCS semantic payload excluding attestation_id and signature; complete-file SHA-256 covers every field including signature",
        "reviewer-bootstrap-verifier-issued natural-person reviewer actor eligibility and independence attestation",
        "strict schema, domain-separated identity derivation, tagged trust-source binding, subject-registry anti-alias, role evidence, lifecycle, currentness, and strict Ed25519 signature verification",
        "REAL reviewer identity prerequisite for every five-role review bundle",
        "external signed authority object; rooted in the permanent pre-G0 bootstrap domain; no reviewer back-pointer or downstream registry enrollment is permitted",
        "INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReviewerActorAttestationV1)",
    ]:
        raise ValueError("reviewer actor attestation schema catalog metadata mismatch")
    actor_anchor = '<a id="schema-def-PhaseFReviewerActorAttestationV1"></a>'
    if wire_text.count(actor_anchor) != 1:
        raise ValueError("reviewer actor attestation definition anchor missing or duplicated")
    bootstrap_root_row = rows["PhaseFReviewerBootstrapTrustRootV1"]
    if bootstrap_root_row != [
        "PhaseFReviewerBootstrapTrustRootV1",
        "EXTERNAL_TRUST_ANCHOR",
        "#schema-def-PhaseFReviewerBootstrapTrustRootV1",
        "sha256:<lowercase_hex>; SHA-256 of the domain-separated canonical semantic payload excluding root_id; complete-file SHA-256 covers every field",
        "normative terminal pre-G0 reviewer bootstrap trust root and subject-uniqueness policy",
        "strict schema, graph-pinned root identity and key fingerprint, narrow purpose scope, lifecycle, rotation, and compromise validation",
        "PRE_G0_REVIEWER_BOOTSTRAP; before G0 and every downstream review gate",
        "immutable external trust anchor; not a Phase F registry record and cannot authorize scientific, architecture, release, or unrelated registry mutations",
        "INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReviewerBootstrapTrustRootV1)",
    ]:
        raise ValueError("reviewer bootstrap trust-root schema catalog metadata mismatch")
    bootstrap_root_anchor = '<a id="schema-def-PhaseFReviewerBootstrapTrustRootV1"></a>'
    if wire_text.count(bootstrap_root_anchor) != 1:
        raise ValueError("reviewer bootstrap trust-root definition anchor missing or duplicated")
    bootstrap_currentness_row = rows["PhaseFReviewerBootstrapCurrentnessProofV1"]
    if bootstrap_currentness_row != [
        "PhaseFReviewerBootstrapCurrentnessProofV1",
        "SIGNED_EXTERNAL_AUTHORITY",
        "#schema-def-PhaseFReviewerBootstrapCurrentnessProofV1",
        "sha256:<lowercase_hex>; SHA-256 of the domain-separated canonical semantic payload excluding currentness_proof_id and signature; complete-file SHA-256 covers every field including signature",
        "root-signed pre-G0 reviewer verifier, subject-registry, and currentness snapshot",
        "strict schema, root signature, root binding, sequence/head, validity window, verifier key, subject-head uniqueness, revocation, compromise, and supersession validation",
        "PRE_G0_REVIEWER_BOOTSTRAP; current proof required before every REAL reviewer identity",
        "external signed authority object; bootstrap reviewer trust only and no architecture, release, or downstream approval authority",
        "INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFReviewerBootstrapCurrentnessProofV1)",
    ]:
        raise ValueError("reviewer bootstrap currentness schema catalog metadata mismatch")
    bootstrap_currentness_anchor = '<a id="schema-def-PhaseFReviewerBootstrapCurrentnessProofV1"></a>'
    if wire_text.count(bootstrap_currentness_anchor) != 1:
        raise ValueError("reviewer bootstrap currentness definition anchor missing or duplicated")


def parse_schema_catalog_ids(text: str) -> list[str]:
    section = text.split("### 53.12", 1)[1].split("The inverse projection", 1)[0]
    ids: list[str] = []
    for line in section.splitlines():
        if not line.startswith("| PhaseF"):
            continue
        cells = parse_pipe_row(line)
        if len(cells) != 9:
            raise ValueError(f"R11 schema catalog column count: {line}")
        ids.append(cells[0])
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate R11 schema catalog ID")
    return ids


def validate_kat_spec() -> None:
    text = SPECS["F-CNF"].read_text()
    if len(G3_FIXTURE_BODY) != G3_FIXTURE_BYTE_LENGTH:
        raise ValueError("G3 fixture byte length constant mismatch")
    if sha256_bytes(G3_FIXTURE_BODY) != G3_FIXTURE_SHA256:
        raise ValueError("G3 fixture SHA-256 constant mismatch")
    required_literals = [
        "fixture_id=R12-POS-SPEC-BUNDLE-TAG",
        f"fixture_byte_length={G3_FIXTURE_BYTE_LENGTH}",
        f"fixture_sha256={G3_FIXTURE_SHA256}",
        G3_FIXTURE_BODY.decode("ascii").rstrip("\n"),
        G3_FIXTURE_BODY.hex(),
        "operation=validate_g3_tag(tag_name,body_bytes,synthetic_context)",
    ]
    for literal in required_literals:
        if literal not in text:
            raise ValueError(f"R12 KAT specification is missing: {literal[:80]}")
    for mutation in G3_KAT_MUTATIONS:
        mutation_row = next(
            (
                line
                for line in text.splitlines()
                if line.startswith(f"| {mutation['id']} |")
            ),
            None,
        )
        if mutation_row is None or mutation["expected_category"] not in mutation_row:
            raise ValueError(f"R12 KAT mutation is missing: {mutation['id']}")

    positive = check_g3_kat(G3_TAG_NAME, G3_FIXTURE_BODY)
    if positive.get("result") != "PASS" or positive.get("decoded_fields") != G3_EXPECTED_FIELDS:
        raise ValueError(f"G3 positive KAT failed: {positive}")
    for mutation in G3_KAT_MUTATIONS:
        tag_name, body = apply_g3_mutation(mutation["id"])
        result = check_g3_kat(tag_name, body)
        if result != {"result": "REJECT", "category": mutation["expected_category"]}:
            raise ValueError(f"G3 mutation {mutation['id']} result: {result}")


def validate_reference_catalogs(
    entries: list[dict[str, object]],
    test_catalog: dict[str, dict[str, str]],
    evidence_catalog: dict[str, dict[str, str]],
) -> None:
    if len(test_catalog) != EXPECTED_R11_TEST_COUNT + len(EXPECTED_R12_TEST_CATALOG_IDS):
        raise ValueError(f"test catalog count: {len(test_catalog)}")
    if len(evidence_catalog) != EXPECTED_R11_EVIDENCE_COUNT:
        raise ValueError(f"evidence catalog count: {len(evidence_catalog)}")
    referenced_tests: set[str] = set()
    referenced_evidence: set[str] = set()
    for entry in entries:
        test_ids = list(entry["test_ids"])
        evidence_ids = list(entry["future_real_evidence_ids"])
        if len(test_ids) != len(set(test_ids)):
            raise ValueError(f"duplicate test reference in {entry['requirement_id']}")
        if len(evidence_ids) != len(set(evidence_ids)):
            raise ValueError(f"duplicate evidence reference in {entry['requirement_id']}")
        unknown_tests = sorted(set(test_ids) - set(test_catalog))
        unknown_evidence = sorted(set(evidence_ids) - set(evidence_catalog))
        if unknown_tests or unknown_evidence:
            raise ValueError(
                f"undefined traceability reference for {entry['requirement_id']}; "
                f"tests={unknown_tests}, evidence={unknown_evidence}"
            )
        referenced_tests.update(test_ids)
        referenced_evidence.update(evidence_ids)
        for test_id in test_ids:
            if test_id.startswith("R12-") and test_catalog[test_id]["kat_class"] not in {
                "literal_kat",
                "constructive_plan_audit",
                "property_test",
            }:
                raise ValueError(f"R12 test has wrong catalog category: {test_id}")
        if set(test_ids).intersection(evidence_ids):
            raise ValueError(f"test/evidence identifier collision in {entry['requirement_id']}")
    if referenced_tests != set(test_catalog):
        raise ValueError(
            f"orphan or unreferenced test catalog IDs: {sorted(set(test_catalog) - referenced_tests)}"
        )
    if referenced_evidence != set(evidence_catalog):
        raise ValueError(
            f"orphan or unreferenced evidence IDs: {sorted(set(evidence_catalog) - referenced_evidence)}"
        )


def expand_refs(value: str) -> list[str]:
    found: list[str] = []
    pattern = re.compile(r"(F-(?:ARCH|OD|WIRE|SCI|OPS|CNF|IMPL))-(\d{2,3})(?:\.\.(\d{2,3}))?")
    for match in pattern.finditer(value):
        prefix, first_text, last_text = match.groups()
        width = len(first_text)
        first = int(first_text)
        last = int(last_text) if last_text else first
        for number in range(first, last + 1):
            ref = f"{prefix}-{number:0{width}d}"
            if ref not in found:
                found.append(ref)
    return found


def validate_inventory() -> None:
    if not ARCH.is_file():
        raise ValueError(f"missing authority document: {ARCH}")
    actual = {path.name for path in PHASE_F.iterdir() if path.is_file()}
    missing = sorted(REQUIRED_FILENAMES - actual)
    unexpected = sorted(actual - ALLOWED_FILENAMES)
    if missing or unexpected:
        raise ValueError(
            f"Phase-F authority inventory mismatch; missing={missing}, "
            f"unexpected={unexpected}"
        )


def validate_r11_and_migration() -> None:
    raw = R11_SOURCE.read_bytes()
    if sha256_bytes(raw) != EXPECTED_R11_SHA256:
        raise ValueError("preserved R11 source SHA-256 does not match the authority")
    if git_blob(R11_SOURCE) != EXPECTED_R11_GIT_BLOB:
        raise ValueError("preserved R11 source Git blob does not match the authority")
    if len(raw) != EXPECTED_R11_BYTE_COUNT or raw.count(b"\n") != EXPECTED_R11_LINE_COUNT:
        raise ValueError("preserved R11 source line/byte counts do not match the authority")

    source = R11_SOURCE.read_text()
    matrix = source.split("### 53.9", 1)[0].split("### 53.8", 1)[1]
    r11_ids = re.findall(r"^\| (R11-\d{2}) \|", matrix, re.MULTILINE)
    if r11_ids != EXPECTED_R11_IDS:
        raise ValueError(f"R11 requirement set mismatch: {r11_ids}")

    ledger = MIGRATION_LEDGER.read_text()
    migration_ids = re.findall(r"^\| (R11-\d{2}) \|", ledger, re.MULTILINE)
    if migration_ids != EXPECTED_R11_IDS:
        raise ValueError(f"R11 migration set mismatch: {migration_ids}")
    findings = re.findall(r"^\| (F-PLAN-R11-[^ |]+) ", ledger, re.MULTILINE)
    if findings != EXPECTED_R11_FINDINGS:
        raise ValueError(f"R11 finding migration set mismatch: {findings}")


def validate_f0_decisions() -> None:
    text = ARCH.read_text()
    section = text.split("## 5. Minimal governance core", 1)[0].split(
        "## 4. F0 owner-decision authority", 1
    )[1]
    decision_ids = re.findall(r"^\| `F-OD-(\d{2})` \|", section, re.MULTILINE)
    expected_numbers = [number.removeprefix("F-OD-") for number in EXPECTED_F0_IDS]
    if decision_ids != expected_numbers:
        raise ValueError(f"F0 decision set mismatch: {decision_ids}")


def parse_architecture() -> list[dict[str, object]]:
    text = ARCH.read_text()
    entries: list[dict[str, object]] = []
    for match in re.finditer(
        r'<a id="(f-arch-\d{3})"></a>\n`(F-ARCH-\d{3})`', text
    ):
        requirement_id = match.group(2)
        entries.append(
            {
                "requirement_id": requirement_id,
                "authority_document": str(ARCH.relative_to(ROOT)),
                "authority_anchor": f"#{match.group(1)}",
                "upstream_requirement_ids": [],
                "f0_decision_dependencies": [],
                "downstream_child_requirements": [],
                "verification_gate": "G0",
                "test_ids": [],
                "future_real_evidence_ids": [],
            }
        )
    actual_ids = [entry["requirement_id"] for entry in entries]
    if actual_ids != EXPECTED_ARCHITECTURE_IDS:
        raise ValueError(f"architecture requirement set mismatch: {actual_ids}")
    return entries


def parse_spec(prefix: str, path: Path) -> list[dict[str, object]]:
    entries: list[dict[str, object]] = []
    for line in path.read_text().splitlines():
        if not line.startswith("| <a id="):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        id_match = re.search(r"`(F-(?:WIRE|SCI|OPS|CNF|IMPL)-(\d{3}))`", cells[0])
        anchor_match = re.search(r'id="([^"]+)"', cells[0])
        if not id_match or not anchor_match:
            raise ValueError(f"invalid requirement row: {line}")
        requirement_id = id_match.group(1)
        refs = expand_refs(cells[1])
        entries.append(
            {
                "requirement_id": requirement_id,
                "authority_document": str(path.relative_to(ROOT)),
                "authority_anchor": f"#{anchor_match.group(1)}",
                "upstream_requirement_ids": [r for r in refs if not r.startswith("F-OD-")],
                "f0_decision_dependencies": [r for r in refs if r.startswith("F-OD-")],
                "downstream_child_requirements": [],
                "verification_gate": "G2",
                "test_ids": [],
                "future_real_evidence_ids": [],
            }
        )
    actual_ids = [entry["requirement_id"] for entry in entries]
    if actual_ids != EXPECTED_SPEC_IDS[prefix]:
        raise ValueError(f"{prefix} requirement set mismatch: {actual_ids}")
    return entries


def validate_traceability(entries: list[dict[str, object]]) -> None:
    expected_ids = EXPECTED_ARCHITECTURE_IDS + [
        requirement_id
        for prefix in SPECS
        for requirement_id in EXPECTED_SPEC_IDS[prefix]
    ]
    actual_ids = [entry["requirement_id"] for entry in entries]
    if len(actual_ids) != len(set(actual_ids)):
        raise ValueError("duplicate requirement ID")
    if sorted(actual_ids) != sorted(expected_ids):
        raise ValueError(
            f"complete Phase-F requirement set mismatch: {sorted(actual_ids)}"
        )

    known_ids = set(actual_ids)
    known_f0_ids = set(EXPECTED_F0_IDS)
    anchors: set[tuple[str, str]] = set()
    parent_map: dict[str, list[str]] = {}
    for entry in entries:
        requirement_id = entry["requirement_id"]
        path = ROOT / str(entry["authority_document"])
        anchor = str(entry["authority_anchor"])[1:]
        if not path.is_file():
            raise ValueError(f"missing authority document: {path}")
        occurrences = path.read_text().count(f'id="{anchor}"')
        if occurrences != 1:
            raise ValueError(
                f"authority anchor {anchor} in {path} occurs {occurrences} times"
            )
        anchor_key = (str(path), anchor)
        if anchor_key in anchors:
            raise ValueError(f"duplicate authority anchor: {anchor_key}")
        anchors.add(anchor_key)

        parents = list(entry["upstream_requirement_ids"])
        decisions = list(entry["f0_decision_dependencies"])
        if not requirement_id.startswith("F-ARCH-") and not parents and not decisions:
            raise ValueError(f"orphan child requirement: {requirement_id}")
        unknown_parents = sorted(set(parents) - known_ids)
        unknown_decisions = sorted(set(decisions) - known_f0_ids)
        if unknown_parents or unknown_decisions:
            raise ValueError(
                f"unknown dependency for {requirement_id}; "
                f"parents={unknown_parents}, decisions={unknown_decisions}"
            )
        parent_map[requirement_id] = parents

    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(requirement_id: str) -> None:
        if requirement_id in visiting:
            raise ValueError(f"requirement dependency cycle at {requirement_id}")
        if requirement_id in visited:
            return
        visiting.add(requirement_id)
        for parent_id in parent_map[requirement_id]:
            visit(parent_id)
        visiting.remove(requirement_id)
        visited.add(requirement_id)

    for requirement_id in actual_ids:
        visit(requirement_id)


def validate_semantic_traceability(
    entries: list[dict[str, Any]], matrix: list[dict[str, Any]], test_catalog: dict[str, dict[str, str]], evidence_catalog: dict[str, dict[str, str]]
) -> dict[str, dict[str, list[str]]]:
    normative = {row["requirement_id"]: row for row in matrix}
    generated = {row["requirement_id"]: row for row in entries}
    if set(normative) != set(generated):
        raise ValueError("normative/generated traceability requirement set mismatch")
    mapping: dict[str, dict[str, list[str]]] = {}
    for requirement_id in sorted(normative):
        expected = normative[requirement_id]
        actual = generated[requirement_id]
        for field in (
            "test_ids",
            "kat_ids",
            "constructive_audit_ids",
            "property_test_ids",
            "future_real_evidence_ids",
            "schema_ids",
        ):
            if actual.get(field) != expected[field]:
                raise ValueError(
                    f"semantic traceability mismatch for {requirement_id}/{field}"
                )
        test_ids = list(expected["test_ids"])
        if set(test_ids) != set(expected["kat_ids"]) | set(expected["constructive_audit_ids"]) | set(expected["property_test_ids"]):
            raise ValueError(f"test category partition mismatch for {requirement_id}")
        for test_id in test_ids:
            if test_id not in test_catalog:
                raise ValueError(f"undefined normative test ID: {test_id}")
        for evidence_id in expected["future_real_evidence_ids"]:
            if evidence_id not in evidence_catalog:
                raise ValueError(f"undefined normative evidence ID: {evidence_id}")
        mapping[requirement_id] = {
            "test_ids": test_ids,
            "future_real_evidence_ids": list(expected["future_real_evidence_ids"]),
            "schema_ids": list(expected["schema_ids"]),
        }
    return mapping


def validate_schema_usage(matrix: list[dict[str, Any]]) -> dict[str, list[str]]:
    inherited = set(parse_schema_catalog_ids(R11_SOURCE.read_text()))
    schema_ids = inherited | R12_SCHEMA_IDS
    forward: dict[str, set[str]] = {}
    inverse: dict[str, set[str]] = {schema_id: set() for schema_id in schema_ids}
    for row in matrix:
        requirement_id = row["requirement_id"]
        listed = row["schema_ids"]
        unknown = sorted(set(listed) - schema_ids)
        if unknown:
            raise ValueError(f"unknown schema usage for {requirement_id}: {unknown}")
        forward[requirement_id] = set(listed)
        for schema_id in listed:
            inverse[schema_id].add(requirement_id)
    orphaned = sorted(schema_id for schema_id, requirements in inverse.items() if not requirements)
    if orphaned:
        raise ValueError(f"orphan schema usage: {orphaned}")
    result = {
        schema_id: sorted(requirements) for schema_id, requirements in sorted(inverse.items())
    }
    # Re-projecting the inverse must recover every forward relationship.
    for requirement_id, listed in forward.items():
        recovered = {
            schema_id for schema_id, requirements in result.items() if requirement_id in requirements
        }
        if recovered != listed:
            raise ValueError(f"schema usage inverse mismatch for {requirement_id}")
    return result


def load_r12_authority_graph() -> tuple[dict[str, Any], dict[str, Any]]:
    try:
        graph = json.loads(AUTHORITY_GRAPH_PATH.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError("R12 authority graph is unreadable") from error
    return graph, validate_r12_authority_graph(graph)


def load_phase_f_entries() -> tuple[
    list[dict[str, object]], dict[str, dict[str, str]], dict[str, dict[str, str]]
]:
    matrix = load_normative_matrix()
    entries = parse_architecture()
    for prefix, path in SPECS.items():
        entries.extend(parse_spec(prefix, path))
    test_catalog, evidence_catalog = load_reference_catalogs()
    matrix_by_id = {row["requirement_id"]: row for row in matrix}
    for entry in entries:
        requirement_id = entry["requirement_id"]
        row = matrix_by_id[requirement_id]
        for field in ("authority_document", "authority_anchor", "upstream_requirement_ids", "f0_decision_dependencies"):
            if entry[field] != row[field]:
                raise ValueError(f"normative matrix document binding mismatch: {requirement_id}/{field}")
        entry.update(
            {
                "validation_category": row["validation_category"],
                "expected_lifecycle_stage": row["expected_lifecycle_stage"],
                "test_ids": list(row["test_ids"]),
                "kat_ids": list(row["kat_ids"]),
                "constructive_audit_ids": list(row["constructive_audit_ids"]),
                "property_test_ids": list(row["property_test_ids"]),
                "future_real_evidence_ids": list(row["future_real_evidence_ids"]),
                "schema_ids": list(row["schema_ids"]),
            }
        )
    validate_traceability(entries)
    validate_semantic_traceability(entries, matrix, test_catalog, evidence_catalog)
    validate_schema_usage(matrix)
    validate_reference_catalogs(entries, test_catalog, evidence_catalog)
    return entries, test_catalog, evidence_catalog


def build_traceability() -> dict[str, object]:
    validate_inventory()
    validate_r11_and_migration()
    validate_f0_decisions()
    validate_wire_catalog()
    validate_kat_spec()
    graph, graph_audit = load_r12_authority_graph()
    entries, test_catalog, evidence_catalog = load_phase_f_entries()
    by_id = {entry["requirement_id"]: entry for entry in entries}
    for child in entries:
        for parent_id in child["upstream_requirement_ids"]:
            by_id[parent_id]["downstream_child_requirements"].append(
                child["requirement_id"]
            )
    for entry in entries:
        entry["downstream_child_requirements"] = sorted(set(entry["downstream_child_requirements"]))
    generated_source_sha256s = {
        edge["from"]: sha256(
            ROOT / graph["node_identity_rules"][edge["from"]]["path"]
        )
        for edge in _graph_edges_for(graph, "generated_traceability_manifest", "generated_from")
    }
    return {
        "schema_version": 1,
        "artifact_kind": "phase_f_derived_traceability_manifest",
        "semantic_authority": False,
        "generation_rule": "docs/engineering_specification/phase_f/generate_phase_f_manifests.py",
        "reference_catalogs": {
            "tests": {
                "r11_source": str(R11_SOURCE.relative_to(ROOT)),
                "r12_source": str(SPECS["F-CNF"].relative_to(ROOT)),
                "count": len(test_catalog),
            },
            "future_real_evidence": {
                "source": str(R11_SOURCE.relative_to(ROOT)),
                "count": len(evidence_catalog),
            },
        },
        "normative_matrix": {
            "path": str(NORMATIVE_MATRIX_PATH.relative_to(ROOT)),
            "sha256": sha256(NORMATIVE_MATRIX_PATH),
            "requirement_count": EXPECTED_R12_REQUIREMENT_COUNT,
        },
        "authority_graph": {
            "path": str(AUTHORITY_GRAPH_PATH.relative_to(ROOT)),
            "sha256": sha256(AUTHORITY_GRAPH_PATH),
            "audit": graph_audit,
        },
        "schema_usage": validate_schema_usage(load_normative_matrix()),
        "generated_source_sha256s": generated_source_sha256s,
        "requirements": sorted(entries, key=lambda row: row["requirement_id"]),
    }


def build_bundle_inputs(trace_sha: str) -> dict[str, object]:
    input_paths = {
        "architecture_plan": ARCH,
        "wire_specification": SPECS["F-WIRE"],
        "scientific_specification": SPECS["F-SCI"],
        "operations_specification": SPECS["F-OPS"],
        "conformance_specification": SPECS["F-CNF"],
        "implementation_readiness_specification": SPECS["F-IMPL"],
        "migration_ledger": MIGRATION_LEDGER,
        "normative_traceability_matrix": NORMATIVE_MATRIX_PATH,
        "authority_graph": AUTHORITY_GRAPH_PATH,
    }
    source_sha256s = {
        name: sha256(path) for name, path in sorted(input_paths.items())
    }
    source_sha256s["generated_traceability_manifest"] = trace_sha
    graph = json.loads(AUTHORITY_GRAPH_PATH.read_text())
    validate_r12_authority_graph(graph)
    authority_bindings: dict[str, dict[str, str | None]] = {}
    for edge in _graph_edges_for(graph, "specification_bundle_inputs", "binds"):
        source = edge["from"]
        rule = graph["node_identity_rules"][source]
        if rule["type"] == "repository_file_sha256":
            authority_bindings[source] = {
                "authority_id": None,
                "sha256": sha256(ROOT / rule["path"]),
                "target": None,
            }
        else:
            authority_bindings[source] = {
                "authority_id": None,
                "sha256": None,
                "target": None,
            }
    payload = {
        "schema_version": 1,
        "artifact_kind": "phase_f_specification_bundle_inputs",
        "authority_graph_sha256": sha256(AUTHORITY_GRAPH_PATH),
        "source_sha256s": source_sha256s,
        "authority_bindings": authority_bindings,
    }
    return {
        **payload,
        "sha256": sha256_bytes(canonical_json_bytes(payload)),
    }


def build_bundle(trace_sha: str) -> dict[str, object]:
    bundle_inputs = build_bundle_inputs(trace_sha)
    input_fingerprint = str(bundle_inputs["sha256"])
    components = []
    for path in SPECS.values():
        components.append(
            {
                "path": str(path.relative_to(ROOT)),
                "sha256": sha256(path),
                "git_blob": git_blob(path),
                "independent_review_bundle_sha256": None,
                "review_status": "PENDING",
                "p0_count": None,
                "p1_count": None,
            }
        )
    return {
        "schema_version": 1,
        "artifact_kind": "phase_f_specification_bundle_manifest_candidate",
        "status": "DRAFT_NO_AUTHORITY",
        "eligible_for_g3": False,
        "architecture_plan": {
            "path": str(ARCH.relative_to(ROOT)),
            "sha256": sha256(ARCH),
            "git_blob": git_blob(ARCH),
            "approved_tag": None,
        },
        "f0_decisions": {"approved_tag": None, "decision_bundle_sha256": None},
        "bundle_inputs": bundle_inputs,
        "component_specifications": components,
        "traceability_manifest": {
            "path": str(TRACE_PATH.relative_to(ROOT)),
            "sha256": trace_sha,
        },
        "migration_ledger": {
            "path": str(MIGRATION_LEDGER.relative_to(ROOT)),
            "sha256": sha256(MIGRATION_LEDGER),
        },
        "normative_traceability_matrix": {
            "path": str(NORMATIVE_MATRIX_PATH.relative_to(ROOT)),
            "sha256": sha256(NORMATIVE_MATRIX_PATH),
        },
        "authority_graph": {
            "path": str(AUTHORITY_GRAPH_PATH.relative_to(ROOT)),
            "sha256": sha256(AUTHORITY_GRAPH_PATH),
        },
        "target_revision": {
            "type": "source_input_fingerprint",
            "sha256": input_fingerprint,
        },
        "migrated_finding_review": {
            "schema": "PhaseFMigratedFindingReviewV1",
            "authority_id": None,
            "sha256": None,
            "target_git_commit": None,
            "target_bundle_inputs_sha256": None,
            "reviewed_migration_ledger_sha256": None,
            "reviewed_normative_traceability_matrix_sha256": None,
            "reviewed_traceability_manifest_sha256": None,
            "review_records": None,
            "review_input_fingerprint": None,
            "review_status": "ABSENT",
        },
        "aggregate_specification_bundle_review_sha256": None,
        "approval_decision": "NO-GO",
        "blocking_reasons": [
            "architecture_plan_tag_absent",
            "f0_decisions_tag_absent",
            "component_independent_reviews_pending",
            "aggregate_specification_bundle_review_absent",
            "migrated_finding_review_pending",
        ],
    }


def component_node_id(prefix: str) -> str:
    return {
        "F-WIRE": "component_wire_spec",
        "F-SCI": "component_scientific_spec",
        "F-OPS": "component_operations_spec",
        "F-CNF": "component_conformance_spec",
        "F-IMPL": "component_implementation_spec",
    }[prefix]


def _migrated_review_rows(target: str, synthetic: bool) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for role in sorted(REVIEW_ROLES):
        reviewer_prefix = "synthetic:" if synthetic else "reviewer:"
        artifact_prefix = "synthetic:" if synthetic else "review-artifact:"
        row: dict[str, Any] = {
            "role": role,
            "reviewer_authority_id": f"{reviewer_prefix}{role}",
            "reviewed_target": target,
            "review_artifact_id": f"{artifact_prefix}{role}",
            "decision": "GO",
            "review_sha256": "",
            "lifecycle": "ACTIVE",
            "independence_relation": {
                "type": "distinct_reviewer_authority",
                "reviewer_authority_id": f"{reviewer_prefix}{role}",
            },
        }
        row_payload = dict(row)
        row_payload.pop("review_sha256")
        row["review_sha256"] = sha256_bytes(canonical_json_bytes(row_payload))
        rows.append(row)
    return rows


def _synthetic_review_references(
    graph: dict[str, Any], rows: list[dict[str, Any]]
) -> tuple[dict[str, dict[str, Any]], dict[str, dict[str, Any]]]:
    contract = _review_reference_contract(graph)
    reviewers: dict[str, dict[str, Any]] = {}
    artifacts: dict[str, dict[str, Any]] = {}
    for index, row in enumerate(rows, start=1):
        reviewer_id = row["reviewer_authority_id"]
        artifact_id = row["review_artifact_id"]
        reviewers[reviewer_id] = {
            "reviewer_authority_id": reviewer_id,
            "authority_kind": contract["reviewer"]["authority_kind"],
            "schema_version": 1,
            "authority_class": "TEST_ONLY",
            "actor_identity_digest": f"{index:064x}",
            "permitted_review_roles": [row["role"]],
            "lifecycle": "ACTIVE",
            "stale": False,
            "superseded_by": None,
            "invalidated": False,
            "sha256": f"{index + 100:064x}",
            "expected_sha256": f"{index + 100:064x}",
            "digest_valid": True,
            "content_unchanged": True,
        }
        artifacts[artifact_id] = {
            "review_artifact_id": artifact_id,
            "authority_kind": contract["artifact"]["authority_kind"],
            "schema_version": 1,
            "authority_class": "TEST_ONLY",
            "reviewer_authority_id": reviewer_id,
            "role": row["role"],
            "reviewed_target": row["reviewed_target"],
            "decision": row["decision"],
            "p0_count": "0",
            "p1_count": "0",
            "p2_count": "0",
            "finding_ids": [],
            "independence_relation": {
                "type": "distinct_reviewer_authority",
                "reviewer_authority_id": reviewer_id,
            },
            "lifecycle": "ACTIVE",
            "stale": False,
            "superseded_by": None,
            "invalidated": False,
            "sha256": f"{index + 200:064x}",
            "expected_sha256": f"{index + 200:064x}",
            "digest_valid": True,
            "content_unchanged": True,
        }
    return reviewers, artifacts


def _synthetic_independent_review_bundle(
    graph: dict[str, Any],
    node_id: str,
    target_commit: str,
    scope_sha: str | None,
    reviewer_authorities: dict[str, dict[str, Any]],
    review_artifacts: dict[str, dict[str, Any]],
    review_target: dict[str, str] | None = None,
) -> dict[str, Any]:
    rows: list[dict[str, Any]] = []
    identity_offset = len(reviewer_authorities) + 1
    for index, role in enumerate(REVIEW_ROLE_ORDER, start=identity_offset):
        reviewer_id = f"{index:064x}"
        artifact_id = f"{index + 20:064x}"
        reviewer_authorities[reviewer_id] = {
            "reviewer_authority_id": reviewer_id,
            "authority_kind": "PhaseFReviewerIdentityV1",
            "schema_version": 1,
            "authority_class": "TEST_ONLY",
            "actor_identity_digest": f"{index + 40:064x}",
            "permitted_review_roles": [role],
            "lifecycle": "ACTIVE",
            "stale": False,
            "superseded_by": None,
            "invalidated": False,
            "digest_valid": True,
            "expected_sha256": f"{index + 60:064x}",
            "sha256": f"{index + 60:064x}",
            "content_unchanged": True,
        }
        artifact = {
            "review_artifact_id": artifact_id,
            "authority_kind": "PhaseFReviewArtifactV1",
            "schema_version": 1,
            "authority_class": "TEST_ONLY",
            "reviewer_authority_id": reviewer_id,
            "role": role,
            "reviewed_target": scope_sha,
            "decision": "GO",
            "p0_count": "0",
            "p1_count": "0",
            "p2_count": "0",
            "finding_ids": [],
            "independence_relation": {
                "type": "distinct_reviewer_authority",
                "reviewer_authority_id": reviewer_id,
            },
            "lifecycle": "ACTIVE",
            "stale": False,
            "superseded_by": None,
            "invalidated": False,
            "reference_sha256": f"{index + 80:064x}",
            "reference_byte_length": "1",
            "digest_valid": True,
            "expected_sha256": f"{index + 100:064x}",
            "sha256": artifact_id,
            "content_unchanged": True,
        }
        review_artifacts[artifact_id] = artifact
        rows.append(
            {
                "role": role,
                "decision": "GO",
                "p0_count": "0",
                "p1_count": "0",
                "finding_ids": [],
                "review_artifact_reference": {
                    "immutable_uri": f"{REVIEW_ARTIFACT_URI_PREFIX}{artifact_id}",
                    "sha256": artifact["reference_sha256"],
                    "byte_length": artifact["reference_byte_length"],
                },
            }
        )
    bundle = {
        "schema_version": 1,
        "review_bundle_id": "",
        "target": review_target or {"type": "git_commit", "git_sha": target_commit},
        "reviews": rows,
        "aggregate_p0_count": "0",
        "aggregate_p1_count": "0",
        "aggregate_decision": "GO",
    }
    bundle["review_bundle_id"] = independent_review_bundle_id(bundle)
    record = _synthetic_record(
        graph,
        node_id,
        f"{200 + len(reviewer_authorities):064x}",
        **bundle,
    )
    record["canonical_object"] = bundle
    return record


def _synthetic_record(
    graph: dict[str, Any], node_id: str, digest: str, **fields: Any
) -> dict[str, Any]:
    return {
        "node_id": node_id,
        "authority_kind": _graph_nodes(graph)[node_id]["authority_kind"],
        "schema_version": 1,
        "sha256": digest,
        "expected_sha256": digest,
        "digest_valid": True,
        "content_unchanged": True,
        "lifecycle": "ACTIVE",
        "stale": False,
        "superseded_by": None,
        "invalidated": False,
        **fields,
    }


def make_synthetic_context() -> G3AuthorityContext:
    graph_bytes = AUTHORITY_GRAPH_PATH.read_bytes()
    graph = json.loads(graph_bytes)
    validate_r12_authority_graph(graph)
    component_paths = list(SPECS.values())
    component_sha256s = [sha256(path) for path in component_paths]
    component_sha_by_node = {
        component_node_id(prefix): sha256(path) for prefix, path in SPECS.items()
    }
    target = "f" * 40
    bundle_inputs_sha = "2" * 64
    trace_sha = "4" * 64
    migration_sha = "5" * 64
    matrix_sha = "3" * 64
    reviewer_authorities: dict[str, dict[str, Any]] = {}
    review_artifacts: dict[str, dict[str, Any]] = {}
    objects: dict[str, dict[str, Any]] = {
        "architecture_plan": _synthetic_record(
            graph, "architecture_plan", sha256(ARCH), bytes=ARCH.read_bytes()
        ),
        "architecture_approval": _synthetic_record(
            graph,
            "architecture_approval",
            "a" * 64,
            authority_id="synthetic:architecture-approval",
            tag_name=G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"],
            target_sha256=sha256(ARCH),
            review_sha256="",
            decision="GO",
            p0_count=0,
            p1_count=0,
        ),
        "f0_decision_bundle": _synthetic_record(
            graph, "f0_decision_bundle", "9" * 64
        ),
        "f0_approval": _synthetic_record(
            graph,
            "f0_approval",
            "b" * 64,
            authority_id="synthetic:f0-approval",
            tag_name=G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"],
            target_sha256="9" * 64,
            review_sha256="",
            decision="GO",
            p0_count=0,
            p1_count=0,
        ),
        "normative_traceability_matrix": _synthetic_record(
            graph, "normative_traceability_matrix", matrix_sha
        ),
        "migration_ledger": _synthetic_record(
            graph, "migration_ledger", migration_sha
        ),
        "generated_traceability_manifest": _synthetic_record(
            graph, "generated_traceability_manifest", trace_sha,
            generated_source_sha256s={
                "architecture_plan": sha256(ARCH),
                "normative_traceability_matrix": matrix_sha,
                "migration_ledger": migration_sha,
                **{
                    component_node_id(prefix): sha256(path)
                    for prefix, path in SPECS.items()
                },
            },
        ),
        "specification_bundle_inputs": _synthetic_record(
            graph,
            "specification_bundle_inputs",
            bundle_inputs_sha,
            authority_id="synthetic:bundle-inputs",
            authority_graph_sha256=sha256_bytes(graph_bytes),
        ),
    }
    for prefix, path in SPECS.items():
        node_id = component_node_id(prefix)
        objects[node_id] = _synthetic_record(
            graph, node_id, sha256(path), bytes=path.read_bytes()
        )
    objects["implementation_readiness_specification"] = _synthetic_record(
        graph,
        "implementation_readiness_specification",
        sha256(SPECS["F-IMPL"]),
        bytes=SPECS["F-IMPL"].read_bytes(),
    )
    objects["architecture_review"] = _synthetic_independent_review_bundle(
        graph,
        "architecture_review",
        target,
        sha256(ARCH),
        reviewer_authorities,
        review_artifacts,
    )
    objects["f0_review"] = _synthetic_independent_review_bundle(
        graph,
        "f0_review",
        target,
        "9" * 64,
        reviewer_authorities,
        review_artifacts,
        review_target={
            "type": "external_object",
            "object_kind": "decision_bundle",
            "object_sha256": "9" * 64,
        },
    )
    for node_id, spec_node in (
        ("component_wire_review", "component_wire_spec"),
        ("component_scientific_review", "component_scientific_spec"),
        ("component_operations_review", "component_operations_spec"),
        ("component_conformance_review", "component_conformance_spec"),
        ("component_implementation_review", "component_implementation_spec"),
    ):
        objects[node_id] = _synthetic_independent_review_bundle(
            graph,
            node_id,
            target,
            objects[spec_node]["sha256"],
            reviewer_authorities,
            review_artifacts,
        )
    objects["architecture_approval"]["review_sha256"] = objects[
        "architecture_review"
    ]["sha256"]
    objects["f0_approval"]["review_sha256"] = objects["f0_review"]["sha256"]
    objects["specification_bundle_inputs"]["authority_bindings"] = {
        source: _authority_descriptor(objects[source])
        for source in sorted(
            edge["from"]
            for edge in _graph_edges_for(graph, "specification_bundle_inputs", "binds")
        )
    }
    migrated = _synthetic_record(
        graph,
        "migrated_finding_review",
        "d" * 64,
        migrated_finding_review_id="d" * 64,
        target_git_commit=target,
        target_bundle_inputs_sha256=bundle_inputs_sha,
        reviewed_migration_ledger_sha256=migration_sha,
        reviewed_normative_traceability_matrix_sha256=matrix_sha,
        reviewed_traceability_manifest_sha256=trace_sha,
        reviewed_component_sha256s=sorted(component_sha256s),
        reviewed_finding_ids=sorted(EXPECTED_MIGRATED_FINDINGS),
        finding_dispositions={
            finding_id: "TECHNICALLY_CLOSED"
            for finding_id in sorted(EXPECTED_MIGRATED_FINDINGS)
        },
        reviewer_roles=sorted(REVIEW_ROLES),
        review_records=[],
        p0_count=0,
        p1_count=0,
        p2_count=0,
        decision="GO",
        created_stage=10,
        producer="independent_review_panel",
        validator="validate_migrated_finding_review",
    )
    migrated["review_input_fingerprint"] = _migrated_review_input_fingerprint(migrated)
    migrated["review_records"] = _migrated_review_rows(
        migrated["review_input_fingerprint"], synthetic=True
    )
    migrated_reviewer_authorities, migrated_review_artifacts = _synthetic_review_references(
        graph, migrated["review_records"]
    )
    reviewer_authorities.update(migrated_reviewer_authorities)
    review_artifacts.update(migrated_review_artifacts)
    objects["migrated_finding_review"] = migrated
    objects["specification_bundle_manifest"] = _synthetic_record(
        graph,
        "specification_bundle_manifest",
        "0" * 64,
        status="READY_FOR_G3",
        eligible_for_g3=True,
        target_commit=target,
        bundle_input_fingerprint_sha256=bundle_inputs_sha,
        bound_authority_sha256s={},
        bytes=b"synthetic-complete-bundle-manifest",
    )
    objects["aggregate_review"] = _synthetic_independent_review_bundle(
        graph,
        "aggregate_review",
        target,
        "0" * 64,
        reviewer_authorities,
        review_artifacts,
    )
    objects["readiness_review"] = _synthetic_independent_review_bundle(
        graph,
        "readiness_review",
        target,
        objects["implementation_readiness_specification"]["sha256"],
        reviewer_authorities,
        review_artifacts,
    )
    objects["aggregate_review"]["sha256"] = "1" * 64
    objects["aggregate_review"]["expected_sha256"] = "1" * 64
    objects["specification_bundle_manifest"]["bound_authority_sha256s"] = {
        source: objects[source]["sha256"]
        for source in sorted(
            edge["from"]
            for edge in _graph_edges_for(graph, "specification_bundle_manifest", "binds")
        )
    }
    return G3AuthorityContext(
        mode="synthetic",
        graph=graph,
        objects=objects,
        bundle_manifest_sha256="0" * 64,
        aggregate_review_sha256="1" * 64,
        expected_target_commit=target,
        tag={
            "exists": True,
            "annotated": True,
            "object_type": "tag",
            "peeled_commit": target,
            "message": G3_FIXTURE_BODY,
        },
        component_sha256s=component_sha256s,
        component_sha_by_node=component_sha_by_node,
        architecture_plan_sha256=sha256(ARCH),
        f0_decisions_sha256="9" * 64,
        authority_graph_sha256=sha256_bytes(graph_bytes),
        authority_graph_bytes=graph_bytes,
        reviewer_authorities=reviewer_authorities,
        review_artifacts=review_artifacts,
        remediation_authority_id="synthetic:remediation-author",
        remediation_actor_identity_digest="e" * 64,
    )


def _git_output(repository: Path, arguments: list[str]) -> bytes:
    return subprocess.check_output(
        ["git", *arguments], cwd=repository, stderr=subprocess.DEVNULL
    )


def validate_review_start_git_state(
    repository: Path, reviewed_target: str, live_main_sha: str
) -> dict[str, str]:
    """Require the review target to be the same commit at every review-start anchor."""

    if not isinstance(reviewed_target, str) or not re.fullmatch(
        r"[0-9a-f]{40}", reviewed_target
    ):
        raise G3ValidationError("review_start_git_mismatch")
    if not isinstance(live_main_sha, str) or not re.fullmatch(
        r"[0-9a-f]{40}", live_main_sha
    ):
        raise G3ValidationError("review_start_git_mismatch")
    anchors = {
        "reviewed_target": reviewed_target,
        "HEAD": reviewed_target,
        "local_main": reviewed_target,
        "origin_main": reviewed_target,
        "live_main": live_main_sha,
    }
    for label, ref in (("HEAD", "HEAD"), ("local_main", "main"), ("origin_main", "origin/main")):
        try:
            anchors[label] = _git_output(repository, ["rev-parse", f"{ref}^{{commit}}"]).decode().strip()
        except (OSError, subprocess.CalledProcessError) as error:
            raise G3ValidationError("review_start_git_mismatch") from error
    if any(
        not isinstance(value, str)
        or not re.fullmatch(r"[0-9a-f]{40}", value)
        or value != reviewed_target
        for value in anchors.values()
    ):
        raise G3ValidationError("review_start_git_mismatch")
    return anchors


def _publication_ref_sha(repository: Path, ref: str) -> str:
    try:
        value = _git_output(repository, ["rev-parse", f"{ref}^{{commit}}"]).decode().strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise G3ValidationError("publication_git_state_unverified") from error
    if not re.fullmatch(r"[0-9a-f]{40}", value):
        raise G3ValidationError("publication_git_state_unverified")
    return value


def _publication_is_ancestor(
    repository: Path, ancestor: str, descendant: str
) -> None:
    try:
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", ancestor, descendant],
            cwd=repository,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise G3ValidationError("publication_non_fast_forward") from error


def read_live_remote_main_sha(repository: Path, remote: str = "origin") -> str:
    """Read the live remote main ref without relying on a stale tracking ref."""

    try:
        output = subprocess.check_output(
            ["git", "ls-remote", "--heads", remote, "refs/heads/main"],
            cwd=repository,
            stderr=subprocess.PIPE,
        ).decode("ascii")
    except (OSError, UnicodeDecodeError, subprocess.CalledProcessError) as error:
        raise G3ValidationError("publication_live_state_unverified") from error
    rows = [line.split() for line in output.splitlines() if line.strip()]
    if len(rows) != 1 or len(rows[0]) != 2 or rows[0][1] != "refs/heads/main":
        raise G3ValidationError("publication_live_state_unverified")
    live_sha = rows[0][0]
    if not re.fullmatch(r"[0-9a-f]{40}", live_sha):
        raise G3ValidationError("publication_live_state_unverified")
    return live_sha


def validate_safe_publication_preflight(
    repository: Path,
    reviewed_sha: str,
    expected_old_sha: str,
    live_main_sha: str,
) -> dict[str, str]:
    """Validate the exact local, ancestry, and live-remote publication preconditions."""

    sha_values = (reviewed_sha, expected_old_sha, live_main_sha)
    if any(not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{40}", value) for value in sha_values):
        raise G3ValidationError("publication_git_state_unverified")
    try:
        dirty = _git_output(
            repository, ["status", "--porcelain=v1", "--untracked-files=all"]
        ).decode("utf-8")
    except (OSError, UnicodeDecodeError, subprocess.CalledProcessError) as error:
        raise G3ValidationError("publication_git_state_unverified") from error
    if dirty:
        raise G3ValidationError("publication_dirty_worktree")
    anchors = {
        "reviewed_sha": reviewed_sha,
        "expected_old_sha": expected_old_sha,
        "live_main_sha": live_main_sha,
        "HEAD": _publication_ref_sha(repository, "HEAD"),
        "local_main": _publication_ref_sha(repository, "main"),
        "origin_main": _publication_ref_sha(repository, "origin/main"),
    }
    if anchors["HEAD"] != reviewed_sha or anchors["local_main"] != reviewed_sha:
        raise G3ValidationError("publication_reviewed_sha_mismatch")
    if live_main_sha != expected_old_sha:
        raise G3ValidationError("publication_live_main_race")
    _publication_is_ancestor(repository, anchors["origin_main"], reviewed_sha)
    _publication_is_ancestor(repository, expected_old_sha, reviewed_sha)
    return anchors


def publish_reviewed_sha_with_lease(
    repository: Path,
    reviewed_sha: str,
    expected_old_sha: str,
    remote: str = "origin",
) -> dict[str, str]:
    """Publish one reviewed SHA through an exact remote compare-and-swap lease."""

    live_before = read_live_remote_main_sha(repository, remote)
    validate_safe_publication_preflight(
        repository, reviewed_sha, expected_old_sha, live_before
    )
    try:
        subprocess.run(
            [
                "git",
                "push",
                "--atomic",
                f"--force-with-lease=refs/heads/main:{expected_old_sha}",
                remote,
                f"{reviewed_sha}:refs/heads/main",
            ],
            cwd=repository,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise G3ValidationError("publication_push_failed") from error
    try:
        _git_output(
            repository,
            ["fetch", "--quiet", remote, "refs/heads/main:refs/remotes/origin/main"],
        )
        live_after = read_live_remote_main_sha(repository, remote)
        anchors = validate_review_start_git_state(repository, reviewed_sha, live_after)
        dirty = _git_output(
            repository, ["status", "--porcelain=v1", "--untracked-files=all"]
        ).decode("utf-8")
    except (OSError, UnicodeDecodeError, subprocess.CalledProcessError, G3ValidationError) as error:
        raise G3ValidationError("publication_postcondition_unverified") from error
    if dirty or live_after != reviewed_sha:
        raise G3ValidationError("publication_postcondition_unverified")
    return anchors


def _empty_tag() -> dict[str, Any]:
    return {
        "exists": False,
        "annotated": False,
        "object_type": None,
        "peeled_commit": None,
        "message": None,
    }


def _read_git_tag(repository: Path, tag_name: str) -> dict[str, Any]:
    tag_ref = f"refs/tags/{tag_name}"
    tag = _empty_tag()
    try:
        object_type = _git_output(repository, ["cat-file", "-t", tag_ref]).decode().strip()
    except subprocess.CalledProcessError:
        return tag
    if object_type is not None:
        tag["exists"] = True
        tag["object_type"] = object_type
        tag["annotated"] = object_type == "tag"
        if tag["annotated"]:
            tag["peeled_commit"] = _git_output(
                repository, ["rev-parse", f"{tag_ref}^{{commit}}"]
            ).decode().strip()
            raw_tag = _git_output(repository, ["cat-file", "tag", tag_ref])
            if b"\n\n" in raw_tag:
                tag["message"] = raw_tag.split(b"\n\n", 1)[1]
    return tag


def _parse_authority_tag(
    tag_name: str, tag: dict[str, Any], node_id: str, authority_id: str,
    authority_sha256: str, target_commit: str,
) -> None:
    if tag.get("exists") is not True:
        raise G3ValidationError(f"missing_{node_id}_tag")
    if tag.get("annotated") is not True or tag.get("object_type") != "tag":
        raise G3ValidationError(f"lightweight_{node_id}_tag")
    if tag.get("peeled_commit") != target_commit:
        raise G3ValidationError(f"{node_id}_target_mismatch")
    expected = (
        f"authority_node_id={node_id}\n"
        f"authority_id={authority_id}\n"
        f"authority_sha256={authority_sha256}\n"
        f"target_git_commit={target_commit}\n"
        "schema_version=1\n"
    ).encode("ascii")
    if tag.get("message") != expected:
        raise G3ValidationError(f"{node_id}_message_mismatch")


def _validate_authority_enrollment_approval_tag(
    tag: dict[str, Any], enrollment: dict[str, Any]
) -> None:
    if tag.get("exists") is not True:
        raise G3ValidationError("missing_authority_enrollment_approval_tag")
    if tag.get("annotated") is not True or tag.get("object_type") != "tag":
        raise G3ValidationError("lightweight_authority_enrollment_approval_tag")
    message = tag.get("message")
    if not isinstance(message, bytes) or not message.endswith(b"\n"):
        raise G3ValidationError("malformed_authority_enrollment_approval_tag")
    if b"\r" in message or any(byte > 0x7F for byte in message):
        raise G3ValidationError("malformed_authority_enrollment_approval_tag")
    lines = message[:-1].split(b"\n")
    if len(lines) != len(AUTHORITY_ENROLLMENT_APPROVAL_FIELDS):
        raise G3ValidationError("malformed_authority_enrollment_approval_tag")
    fields: dict[str, str] = {}
    for expected_name, line in zip(AUTHORITY_ENROLLMENT_APPROVAL_FIELDS, lines):
        if not line or b"=" not in line:
            raise G3ValidationError("malformed_authority_enrollment_approval_tag")
        raw_name, raw_value = line.split(b"=", 1)
        try:
            name = raw_name.decode("ascii")
            value = raw_value.decode("ascii")
        except UnicodeDecodeError as error:
            raise G3ValidationError(
                "malformed_authority_enrollment_approval_tag"
            ) from error
        if (
            name != expected_name
            or name in fields
            or not value
            or value != value.strip()
            or "=" in value
        ):
            raise G3ValidationError("malformed_authority_enrollment_approval_tag")
        fields[name] = value
    if (
        fields["phase_f_plan_tag"]
        != G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"]
        or fields["f0_decisions_tag"]
        != G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"]
        or fields["readiness_tag"] != "ism-mechanism-health-v1-f-readiness-approved"
        or not re.fullmatch(r"[0-9a-f]{40}", fields["readiness_main_sha"])
        or fields["enrollment_sha256"] != enrollment["complete_file_sha256"]
        or fields["owner_authority_id"] != enrollment["owner_authority_id"]
        or fields["registry_authority_id"] != enrollment["registry_authority_id"]
        or fields["owner_public_key_fingerprint"]
        != enrollment["owner_public_key_fingerprint"]
        or fields["registry_public_key_fingerprint"]
        != enrollment["registry_public_key_fingerprint"]
        or not re.fullmatch(r"[0-9a-f]{64}", fields["review_bundle_sha256"])
        or fields["approval_decision"] != "GO"
        or tag.get("peeled_commit") != fields["readiness_main_sha"]
    ):
        raise G3ValidationError("authority_enrollment_approval_binding_mismatch")


def _load_real_json_authority(
    repository: Path, graph: dict[str, Any], node_id: str, path: Path
) -> dict[str, Any]:
    try:
        raw = path.read_bytes()
        decoded = _parse_json_without_duplicates(raw)
    except FileNotFoundError as error:
        raise G3ValidationError(f"missing_{node_id}") from error
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise G3ValidationError(f"malformed_{node_id}") from error
    if not isinstance(decoded, dict) or canonical_json_bytes(decoded) != raw:
        raise G3ValidationError(f"noncanonical_{node_id}")
    expected_fields = graph["object_field_contracts"].get(node_id)
    if not isinstance(expected_fields, list) or set(decoded) != set(expected_fields):
        raise G3ValidationError(f"{node_id}_schema_mismatch")
    rule = graph["node_identity_rules"][node_id]
    excluded = rule.get("exclude_fields", [])
    identity_payload = {
        key: value for key, value in decoded.items() if key not in excluded
    }
    identity = sha256_bytes(canonical_json_bytes(identity_payload))
    if rule["type"] != "canonical_object_sha256_excluding_field":
        identity = sha256_bytes(raw)
    record = dict(decoded)
    record.update(
        {
            "node_id": node_id,
            "authority_kind": _graph_nodes(graph)[node_id]["authority_kind"],
            "bytes": raw,
            "canonical_object": decoded,
            "sha256": identity,
            "expected_sha256": identity,
            "content_unchanged": True,
            "lifecycle": "ACTIVE",
            "stale": False,
            "superseded_by": None,
            "invalidated": False,
        }
    )
    return record


def _load_real_authority_enrollment(
    repository: Path, graph: dict[str, Any], allow_test_only: bool
) -> dict[str, Any]:
    """Load the inherited unsigned R11 enrollment as the attestation trust root."""

    del allow_test_only  # R11 enrollment has no authority-class field.
    contract = _review_reference_contract(graph)["authority_enrollment"]
    path = repository / contract["authority_path"]
    try:
        raw = path.read_bytes()
        decoded = _parse_json_without_duplicates(raw)
    except FileNotFoundError as error:
        raise G3ValidationError("missing_authority_enrollment") from error
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise G3ValidationError("malformed_authority_enrollment") from error
    if not isinstance(decoded, dict) or canonical_json_bytes(decoded) != raw:
        raise G3ValidationError("noncanonical_authority_enrollment")
    if set(decoded) != set(contract["required_fields"]):
        raise G3ValidationError("authority_enrollment_schema_mismatch")
    if decoded.get("schema_version") != 1:
        raise G3ValidationError("authority_enrollment_schema_mismatch")
    enrollment_id = decoded.get("enrollment_id")
    if not isinstance(enrollment_id, str) or not re.fullmatch(r"sha256:[0-9a-f]{64}", enrollment_id):
        raise G3ValidationError("malformed_authority_enrollment_id")
    if authority_enrollment_id(decoded) != enrollment_id:
        raise G3ValidationError("authority_enrollment_digest_mismatch")
    for field_name in ("owner_authority_id", "registry_authority_id"):
        if not isinstance(decoded.get(field_name), str) or not re.fullmatch(
            RUNTIME_STABLE_ID_PATTERN, decoded[field_name]
        ):
            raise G3ValidationError("malformed_authority_enrollment_authority_id")
    if (
        decoded.get("phase_f_plan_tag")
        != G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"]
        or decoded.get("f0_decisions_tag")
        != G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"]
        or decoded.get("readiness_tag")
        != "ism-mechanism-health-v1-f-readiness-approved"
    ):
        raise G3ValidationError("authority_enrollment_tag_binding_mismatch")
    for field_name in ("owner_public_key", "registry_public_key"):
        if not isinstance(decoded.get(field_name), str) or not re.fullmatch(
            ED25519_PUBLIC_KEY_PATTERN, decoded[field_name]
        ):
            raise G3ValidationError("malformed_authority_enrollment_public_key")
    for key_field, fingerprint_field in (
        ("owner_public_key", "owner_public_key_fingerprint"),
        ("registry_public_key", "registry_public_key_fingerprint"),
    ):
        fingerprint = decoded.get(fingerprint_field)
        if not isinstance(fingerprint, str) or not re.fullmatch(r"[0-9a-f]{64}", fingerprint):
            raise G3ValidationError("malformed_authority_enrollment_fingerprint")
        if sha256_bytes(bytes.fromhex(decoded[key_field])) != fingerprint:
            raise G3ValidationError("authority_enrollment_fingerprint_mismatch")
    for field_name in (
        "owner_authority_document",
        "registry_authority_document",
    ):
        reference = decoded.get(field_name)
        if not isinstance(reference, dict) or set(reference) != {
            "immutable_uri", "sha256", "byte_length"
        }:
            raise G3ValidationError("authority_enrollment_reference_schema_mismatch")
        if (
            not isinstance(reference["immutable_uri"], str)
            or not reference["immutable_uri"]
            or not isinstance(reference["sha256"], str)
            or not re.fullmatch(r"[0-9a-f]{64}", reference["sha256"])
            or not isinstance(reference["byte_length"], str)
            or not re.fullmatch(CANONICAL_UNSIGNED_INTEGER_PATTERN, reference["byte_length"])
        ):
            raise G3ValidationError("authority_enrollment_reference_malformed")
    if not isinstance(decoded.get("custody_policy_sha256"), str) or not re.fullmatch(
        r"[0-9a-f]{64}", decoded["custody_policy_sha256"]
    ):
        raise G3ValidationError("malformed_authority_enrollment_custody_policy")
    if not isinstance(decoded.get("created_at"), str) or not re.fullmatch(
        UTC_SECOND_TIMESTAMP_PATTERN, decoded["created_at"]
    ):
        raise G3ValidationError("malformed_authority_enrollment_timestamp")
    record = dict(decoded)
    record.update(
        {
            "bytes": raw,
            "canonical_object": decoded,
            "complete_file_sha256": sha256_bytes(raw),
            "content_unchanged": True,
        }
    )
    approval_tag = _read_git_tag(repository, AUTHORITY_ENROLLMENT_APPROVAL_TAG)
    _validate_authority_enrollment_approval_tag(approval_tag, record)
    record["enrollment_approval_tag"] = approval_tag
    return record


def _load_real_reviewer_bootstrap_trust(
    repository: Path, graph: dict[str, Any], allow_test_only: bool
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Resolve the pre-G0 terminal root and its signed currentness snapshot."""

    contract = _reviewer_bootstrap_trust_contract(graph)

    def load(path: Path, fields: set[str], kind: str) -> dict[str, Any]:
        try:
            raw = path.read_bytes()
            decoded = _parse_json_without_duplicates(raw)
        except FileNotFoundError as error:
            raise G3ValidationError(f"missing_{kind}") from error
        except (OSError, UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
            raise G3ValidationError(f"malformed_{kind}") from error
        if not isinstance(decoded, dict) or canonical_json_bytes(decoded) != raw:
            raise G3ValidationError(f"noncanonical_{kind}")
        if set(decoded) != fields:
            raise G3ValidationError(f"{kind}_schema_mismatch")
        record = dict(decoded)
        record.update(
            {
                "bytes": raw,
                "canonical_object": decoded,
                "complete_file_sha256": sha256_bytes(raw),
                "content_unchanged": True,
            }
        )
        return record

    root = load(
        repository / contract["root_path"],
        REVIEWER_BOOTSTRAP_ROOT_FIELDS,
        "reviewer_bootstrap_root",
    )
    proof = load(
        repository / contract["currentness_proof_path"],
        REVIEWER_BOOTSTRAP_CURRENTNESS_FIELDS,
        "reviewer_bootstrap_currentness",
    )
    if root.get("authority_kind") != contract["root_authority_kind"]:
        raise G3ValidationError("reviewer_bootstrap_root_schema_mismatch")
    if proof.get("authority_kind") != contract["currentness_proof_authority_kind"]:
        raise G3ValidationError("reviewer_bootstrap_currentness_schema_mismatch")

    probe = G3AuthorityContext(
        mode="real_test" if allow_test_only else "real",
        graph=graph,
        objects={},
        bundle_manifest_sha256=None,
        aggregate_review_sha256=None,
        expected_target_commit="",
        tag={},
        component_sha256s=[],
        component_sha_by_node={},
        architecture_plan_sha256="",
        f0_decisions_sha256=None,
        authority_graph_sha256="",
        authority_graph_bytes=b"",
        reviewer_bootstrap_root=root,
        reviewer_bootstrap_currentness=proof,
    )
    _validate_reviewer_bootstrap_context(probe)
    return root, proof


def _load_real_reviewer_actor_attestation(
    repository: Path,
    graph: dict[str, Any],
    path: Path,
    expected_id: str,
    bootstrap_root: dict[str, Any],
    bootstrap_currentness: dict[str, Any],
) -> dict[str, Any]:
    contract = _review_reference_contract(graph)["actor_attestation"]
    try:
        raw = path.read_bytes()
        decoded = _parse_json_without_duplicates(raw)
    except FileNotFoundError as error:
        raise G3ValidationError("missing_reviewer_actor_attestation") from error
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise G3ValidationError("malformed_reviewer_actor_attestation") from error
    if not isinstance(decoded, dict) or canonical_json_bytes(decoded) != raw:
        raise G3ValidationError("noncanonical_reviewer_actor_attestation")
    if set(decoded) != set(contract["required_fields"]):
        raise G3ValidationError("reviewer_actor_attestation_schema_mismatch")
    attestation_id = decoded.get("attestation_id")
    if (
        not isinstance(attestation_id, str)
        or not re.fullmatch(r"sha256:[0-9a-f]{64}", attestation_id)
        or attestation_id != expected_id
        or reviewer_actor_attestation_id(decoded) != attestation_id
    ):
        raise G3ValidationError("reviewer_actor_attestation_digest_mismatch")
    if decoded.get("authority_kind") != contract["authority_kind"] or decoded.get("schema_version") != 1:
        raise G3ValidationError("reviewer_actor_attestation_schema_mismatch")
    if not isinstance(decoded.get("actor_subject_id"), str) or not re.fullmatch(
        RUNTIME_STABLE_ID_PATTERN, decoded["actor_subject_id"]
    ):
        raise G3ValidationError("malformed_reviewer_actor_subject")
    if decoded.get("actor_class") != "natural_person":
        raise G3ValidationError("ineligible_reviewer_actor_class")
    for field_name in (
        "actor_identity_evidence_sha256",
        "role_eligibility_evidence_sha256",
        "independence_evidence_sha256",
        "independence_excluded_actor_identity_digest",
    ):
        if not isinstance(decoded.get(field_name), str) or not re.fullmatch(
            r"[0-9a-f]{64}", decoded[field_name]
        ):
            raise G3ValidationError("malformed_reviewer_actor_evidence")
    trust_source = decoded.get("trust_source")
    if (
        not isinstance(trust_source, dict)
        or set(trust_source) != REVIEWER_BOOTSTRAP_TRUST_SOURCE_FIELDS
        or trust_source.get("type") != REVIEWER_BOOTSTRAP_TRUST_SOURCE
        or trust_source.get("root_id") != bootstrap_root.get("root_id")
        or trust_source.get("root_sha256") != bootstrap_root.get("complete_file_sha256")
        or trust_source.get("currentness_proof_id")
        != bootstrap_currentness.get("currentness_proof_id")
        or trust_source.get("currentness_proof_sha256")
        != bootstrap_currentness.get("complete_file_sha256")
        or not isinstance(decoded.get("eligible_role"), str)
        or decoded.get("eligible_role") not in REVIEW_ROLES
        or decoded.get("eligibility_verifier_authority_id")
        != bootstrap_currentness.get("current_verifier_authority_id")
        or decoded.get("independence_verifier_authority_id")
        != bootstrap_currentness.get("current_verifier_authority_id")
    ):
        raise G3ValidationError("reviewer_actor_attestation_trust_source_mismatch")
    if not isinstance(decoded.get("created_at"), str) or not re.fullmatch(
        UTC_SECOND_TIMESTAMP_PATTERN, decoded["created_at"]
    ):
        raise G3ValidationError("malformed_reviewer_actor_attestation_timestamp")
    if (
        decoded.get("lifecycle") != "ACTIVE"
        or decoded.get("stale") is not False
        or decoded.get("superseded_by") is not None
        or decoded.get("invalidated") is not False
    ):
        raise G3ValidationError("stale_reviewer_actor_attestation")
    signature = decoded.get("signature")
    if not isinstance(signature, str) or not re.fullmatch(ED25519_SIGNATURE_PATTERN, signature):
        raise G3ValidationError("malformed_reviewer_actor_attestation_signature")
    signing_payload = {key: value for key, value in decoded.items() if key != "signature"}
    if not verify_ed25519_strict(
        bootstrap_currentness["current_verifier_public_key"],
        signature,
        REVIEWER_ACTOR_ATTESTATION_DOMAIN + canonical_jcs_bytes(signing_payload),
    ):
        raise G3ValidationError("invalid_reviewer_actor_attestation_signature")
    record = dict(decoded)
    record.update(
        {
            "bytes": raw,
            "canonical_object": decoded,
            "complete_file_sha256": sha256_bytes(raw),
            "content_unchanged": True,
            "signature_verified": True,
        }
    )
    return record


def _load_real_repository_file(
    repository: Path, target_commit: str, graph: dict[str, Any], node_id: str
) -> dict[str, Any]:
    rule = graph["node_identity_rules"][node_id]
    relative_path = rule["path"]
    try:
        raw = _git_output(repository, ["show", f"{target_commit}:{relative_path}"])
    except subprocess.CalledProcessError as error:
        raise G3ValidationError(f"missing_{node_id}") from error
    record: dict[str, Any] = {
        "node_id": node_id,
        "authority_kind": _graph_nodes(graph)[node_id]["authority_kind"],
        "schema_version": 1,
        "sha256": sha256_bytes(raw),
        "expected_sha256": sha256_bytes(raw),
        "content_unchanged": True,
        "lifecycle": "ACTIVE",
        "stale": False,
        "superseded_by": None,
        "invalidated": False,
        "bytes": raw,
    }
    if relative_path.endswith(".json"):
        try:
            decoded = _parse_json_without_duplicates(raw)
        except (UnicodeDecodeError, json.JSONDecodeError, ValueError):
            decoded = None
        if isinstance(decoded, dict):
            record["decoded_object"] = decoded
            record.update(
                {
                    key: value
                    for key, value in decoded.items()
                    if key not in {"node_id", "authority_kind", "schema_version"}
                }
            )
    return record


def _load_real_reference_json(
    repository: Path,
    graph: dict[str, Any],
    reference_type: str,
    path: Path,
    expected_id: str | None = None,
) -> dict[str, Any]:
    contract = _review_reference_contract(graph)[reference_type]
    prefix = {
        "reviewer": "reviewer_identity",
        "artifact": "review_artifact_identity",
        "remediation_author": "remediation_authority",
    }[reference_type]
    try:
        raw = path.read_bytes()
        decoded = _parse_json_without_duplicates(raw)
    except FileNotFoundError as error:
        raise G3ValidationError(f"missing_{prefix}") from error
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise G3ValidationError(f"malformed_{prefix}") from error
    if not isinstance(decoded, dict) or canonical_json_bytes(decoded) != raw:
        raise G3ValidationError(f"noncanonical_{prefix}")
    if set(decoded) != set(contract["required_fields"]):
        raise G3ValidationError(f"{prefix}_schema_mismatch")
    id_field = contract["id_field"]
    identifier = decoded.get(id_field)
    if not isinstance(identifier, str) or not re.fullmatch(r"[0-9a-f]{64}", identifier):
        raise G3ValidationError(f"malformed_{prefix}_id")
    if expected_id is not None and identifier != expected_id:
        raise G3ValidationError(f"{prefix}_id_mismatch")
    identity_payload = {
        key: value
        for key, value in decoded.items()
        if key not in contract["digest_excluded_fields"]
    }
    identity = sha256_bytes(canonical_json_bytes(identity_payload))
    if identity != identifier:
        raise G3ValidationError(f"{prefix}_digest_mismatch")
    record = dict(decoded)
    record.update(
        {
            "bytes": raw,
            "canonical_object": decoded,
            "sha256": identity,
            "expected_sha256": identity,
            "content_unchanged": True,
        }
    )
    return record


def _resolve_real_review_references(
    repository: Path,
    graph: dict[str, Any],
    migrated: dict[str, Any] | None,
    review_bundles: dict[str, dict[str, Any]],
    resolution: dict[str, Any],
    allow_test_only: bool,
) -> tuple[
    dict[str, dict[str, Any]],
    dict[str, dict[str, Any]],
    dict[str, dict[str, Any]],
    dict[str, Any] | None,
    dict[str, Any] | None,
    dict[str, Any] | None,
    str | None,
    str | None,
]:
    contract = _review_reference_contract(graph)
    reviewers: dict[str, dict[str, Any]] = {}
    artifacts: dict[str, dict[str, Any]] = {}
    reviewer_actor_attestations: dict[str, dict[str, Any]] = {}
    authority_enrollment: dict[str, Any] | None = None
    reviewer_bootstrap_root: dict[str, Any] | None = None
    reviewer_bootstrap_currentness: dict[str, Any] | None = None
    remediation_authority_id: str | None = None
    remediation_actor_identity_digest: str | None = None

    try:
        reviewer_bootstrap_root, reviewer_bootstrap_currentness = (
            _load_real_reviewer_bootstrap_trust(repository, graph, allow_test_only)
        )
    except G3ValidationError as error:
        resolution["errors"].append(
            {"node_id": "reviewer_bootstrap_trust", "category": error.category}
        )
        if error.category.startswith("missing_"):
            resolution["missing"].append(
                {"node_id": "reviewer_bootstrap_trust", "category": error.category}
            )
    else:
        resolution["resolved_node_ids"].extend(
            ["reviewer_bootstrap_root", "reviewer_bootstrap_currentness"]
        )

    author_path = repository / contract["remediation_author"]["authority_path"]
    try:
        author = _load_real_reference_json(
            repository, graph, "remediation_author", author_path
        )
    except G3ValidationError as error:
        resolution["errors"].append(
            {"node_id": "remediation_authority", "category": error.category}
        )
        if error.category.startswith("missing_"):
            resolution["missing"].append(
                {"node_id": "remediation_authority", "category": error.category}
            )
    else:
        actor_identity_digest = author.get("actor_identity_digest")
        if (
            author.get("authority_kind")
            != contract["remediation_author"]["authority_kind"]
            or author.get("schema_version") != 1
            or author.get("authority_class") not in {"REAL", "TEST_ONLY"}
            or (author.get("authority_class") == "TEST_ONLY" and not allow_test_only)
            or author.get("lifecycle") != "ACTIVE"
            or author.get("stale") is not False
            or author.get("superseded_by") is not None
            or author.get("invalidated") is not False
            or not isinstance(actor_identity_digest, str)
            or not re.fullmatch(r"[0-9a-f]{64}", actor_identity_digest)
        ):
            resolution["errors"].append(
                {"node_id": "remediation_authority", "category": "invalid_remediation_authority"}
            )
        else:
            remediation_authority_id = author[contract["remediation_author"]["id_field"]]
            remediation_actor_identity_digest = actor_identity_digest
            resolution["resolved_node_ids"].append("remediation_authority")

    def reject_test_only(prefix: str, record: dict[str, Any]) -> None:
        if record.get("authority_class") == "TEST_ONLY" and not allow_test_only:
            resolution["errors"].append(
                {"node_id": prefix, "category": "synthetic_authority_in_real_mode"}
            )

    def load_reviewer(reviewer_id: object) -> None:
        if not isinstance(reviewer_id, str) or not re.fullmatch(r"[0-9a-f]{64}", reviewer_id):
            resolution["errors"].append(
                {"node_id": f"reviewer_identity:{reviewer_id}", "category": "unresolved_reviewer_identity"}
            )
            return
        if reviewer_id in reviewers:
            return
        reviewer_path = repository / contract["reviewer"]["authority_path_template"].replace(
            "{reviewer_authority_id}", reviewer_id
        )
        try:
            reviewer = _load_real_reference_json(
                repository, graph, "reviewer", reviewer_path, reviewer_id
            )
        except G3ValidationError as error:
            resolution["errors"].append(
                {"node_id": f"reviewer_identity:{reviewer_id}", "category": error.category}
            )
            if error.category.startswith("missing_"):
                resolution["missing"].append(
                    {"node_id": f"reviewer_identity:{reviewer_id}", "category": error.category}
                )
            return
        reviewers[reviewer_id] = reviewer
        reject_test_only(f"reviewer_identity:{reviewer_id}", reviewer)
        resolution["resolved_node_ids"].append(f"reviewer_identity:{reviewer_id}")
        attestation_id = reviewer.get("actor_attestation_id")
        if (
            reviewer_bootstrap_root is None
            or reviewer_bootstrap_currentness is None
            or not isinstance(attestation_id, str)
            or not re.fullmatch(r"sha256:[0-9a-f]{64}", attestation_id)
        ):
            resolution["errors"].append(
                {
                    "node_id": f"reviewer_identity:{reviewer_id}",
                    "category": "unresolved_reviewer_actor_attestation",
                }
            )
            return
        if attestation_id in reviewer_actor_attestations:
            return
        attestation_path = repository / contract["actor_attestation"][
            "authority_path_template"
        ].replace("{attestation_id}", attestation_id)
        try:
            attestation = _load_real_reviewer_actor_attestation(
                repository,
                graph,
                attestation_path,
                attestation_id,
                reviewer_bootstrap_root,
                reviewer_bootstrap_currentness,
            )
        except G3ValidationError as error:
            resolution["errors"].append(
                {
                    "node_id": f"reviewer_actor_attestation:{attestation_id}",
                    "category": error.category,
                }
            )
            if error.category.startswith("missing_"):
                resolution["missing"].append(
                    {
                        "node_id": f"reviewer_actor_attestation:{attestation_id}",
                        "category": error.category,
                    }
                )
            return
        reviewer_actor_attestations[attestation_id] = attestation
        resolution["resolved_node_ids"].append(
            f"reviewer_actor_attestation:{attestation_id}"
        )

    def load_artifact(artifact_id: object, reference: dict[str, Any] | None = None) -> None:
        if not isinstance(artifact_id, str) or not re.fullmatch(r"[0-9a-f]{64}", artifact_id):
            resolution["errors"].append(
                {"node_id": f"review_artifact:{artifact_id}", "category": "unresolved_review_artifact_identity"}
            )
            return
        if artifact_id in artifacts:
            artifact = artifacts[artifact_id]
        else:
            artifact_path = repository / contract["artifact"]["authority_path_template"].replace(
                "{review_artifact_id}", artifact_id
            )
            try:
                artifact = _load_real_reference_json(
                    repository, graph, "artifact", artifact_path, artifact_id
                )
            except G3ValidationError as error:
                resolution["errors"].append(
                    {"node_id": f"review_artifact:{artifact_id}", "category": error.category}
                )
                if error.category.startswith("missing_"):
                    resolution["missing"].append(
                        {"node_id": f"review_artifact:{artifact_id}", "category": error.category}
                    )
                return
            artifacts[artifact_id] = artifact
            reject_test_only(f"review_artifact:{artifact_id}", artifact)
            resolution["resolved_node_ids"].append(f"review_artifact:{artifact_id}")
        if reference is not None:
            raw = artifact.get("bytes")
            if (
                not isinstance(raw, bytes)
                or reference.get("sha256") != sha256_bytes(raw)
                or reference.get("byte_length") != str(len(raw))
            ):
                resolution["errors"].append(
                    {"node_id": f"review_artifact:{artifact_id}", "category": "review_artifact_reference_mismatch"}
                )
        load_reviewer(artifact.get("reviewer_authority_id"))

    for node_id, bundle_record in review_bundles.items():
        bundle = bundle_record.get("canonical_object")
        rows = bundle.get("reviews") if isinstance(bundle, dict) else None
        if not isinstance(rows, list):
            continue
        for row in rows:
            reference = row.get("review_artifact_reference") if isinstance(row, dict) else None
            if not isinstance(reference, dict) or set(reference) != {
                "immutable_uri", "sha256", "byte_length"
            }:
                resolution["errors"].append(
                    {"node_id": f"{node_id}:review_artifact", "category": "review_artifact_reference_schema_mismatch"}
                )
                continue
            uri = reference.get("immutable_uri")
            artifact_id = (
                uri.removeprefix(REVIEW_ARTIFACT_URI_PREFIX)
                if isinstance(uri, str) and uri.startswith(REVIEW_ARTIFACT_URI_PREFIX)
                else None
            )
            if not isinstance(uri, str) or uri != f"{REVIEW_ARTIFACT_URI_PREFIX}{artifact_id}":
                resolution["errors"].append(
                    {"node_id": f"{node_id}:review_artifact", "category": "unresolved_review_artifact_identity"}
                )
                continue
            load_artifact(artifact_id, reference)

    rows = migrated.get("review_records") if isinstance(migrated, dict) else None
    if isinstance(rows, list):
        for row in rows:
            if not isinstance(row, dict):
                continue
            artifact_id = row.get("review_artifact_id")
            load_artifact(artifact_id)
            load_reviewer(row.get("reviewer_authority_id"))
    for attestation_id, attestation in reviewer_actor_attestations.items():
        if (
            remediation_actor_identity_digest is None
            or attestation.get("independence_excluded_actor_identity_digest")
            != remediation_actor_identity_digest
        ):
            resolution["errors"].append(
                {
                    "node_id": f"reviewer_actor_attestation:{attestation_id}",
                    "category": "reviewer_actor_independence_evidence_mismatch",
                }
            )
    return (
        reviewers,
        artifacts,
        reviewer_actor_attestations,
        authority_enrollment,
        reviewer_bootstrap_root,
        reviewer_bootstrap_currentness,
        remediation_authority_id,
        remediation_actor_identity_digest,
    )


def _resolve_real_authority(
    repository: Path,
    graph: dict[str, Any],
    target_commit: str,
    allow_test_only: bool,
) -> tuple[
    dict[str, dict[str, Any]],
    dict[str, Any],
    dict[str, Any],
    dict[str, dict[str, Any]],
    dict[str, dict[str, Any]],
    dict[str, dict[str, Any]],
    dict[str, Any] | None,
    dict[str, Any] | None,
    dict[str, Any] | None,
    str | None,
    str | None,
]:
    nodes = _graph_nodes(graph)
    edges = _graph_edges(graph, nodes)
    order = _topological_order(nodes, edges)
    required_closure = _ancestors("g3_approval_tag", edges) | {"g3_approval_tag"}
    objects: dict[str, dict[str, Any]] = {}
    resolution: dict[str, Any] = {
        "requested_node_ids": [node_id for node_id in order if node_id in required_closure],
        "resolved_node_ids": [],
        "missing": [],
        "errors": [],
    }
    tags: dict[str, dict[str, Any]] = {}
    for node_id in order:
        if node_id not in required_closure or node_id == "g3_approval_tag":
            continue
        rule = graph["node_identity_rules"][node_id]
        try:
            if rule["type"] == "repository_file_sha256":
                record = _load_real_repository_file(repository, target_commit, graph, node_id)
            elif rule["type"] in {"authority_file_sha256", "canonical_object_sha256_excluding_field"}:
                record = _load_real_json_authority(
                    repository, graph, node_id, repository / rule["path"]
                )
            elif rule["type"] == "annotated_tag_message_sha256":
                record = _load_real_json_authority(
                    repository, graph, node_id, repository / rule["authority_path"]
                )
                tag = _read_git_tag(repository, rule["tag_name"])
                tags[node_id] = tag
                _parse_authority_tag(
                    rule["tag_name"], tag, node_id, record["authority_id"],
                    record["sha256"], target_commit,
                )
            else:
                raise G3ValidationError(f"unknown_discovery_rule_{node_id}")
        except G3ValidationError as error:
            resolution["errors"].append(
                {"node_id": node_id, "category": error.category}
            )
            if error.category.startswith("missing_"):
                resolution["missing"].append(
                    {"node_id": node_id, "category": error.category}
                )
            continue
        objects[node_id] = record
        resolution["resolved_node_ids"].append(node_id)
    (
        reviewer_authorities,
        review_artifacts,
        reviewer_actor_attestations,
        authority_enrollment,
        reviewer_bootstrap_root,
        reviewer_bootstrap_currentness,
        remediation_authority_id,
        remediation_actor_identity_digest,
    ) = _resolve_real_review_references(
        repository,
        graph,
        objects.get("migrated_finding_review"),
        {node_id: objects[node_id] for node_id in REVIEW_BUNDLE_NODES if node_id in objects},
        resolution,
        allow_test_only,
    )
    g3_tag = _read_git_tag(repository, G3_TAG_NAME)
    resolution["authority_tags"] = {
        node_id: {
            "exists": tag["exists"],
            "annotated": tag["annotated"],
            "peeled_commit": tag["peeled_commit"],
        }
        for node_id, tag in sorted(tags.items())
    }
    resolution["authority_tags"]["g3_approval_tag"] = {
        "exists": g3_tag["exists"],
        "annotated": g3_tag["annotated"],
        "peeled_commit": g3_tag["peeled_commit"],
    }
    resolution["g3_tag"] = {
        "exists": g3_tag["exists"],
        "annotated": g3_tag["annotated"],
        "peeled_commit": g3_tag["peeled_commit"],
    }
    if not g3_tag["exists"]:
        resolution["missing"].append(
            {"node_id": "g3_approval_tag", "category": "missing_g3_approval_tag"}
        )
    elif g3_tag["annotated"] is not True or g3_tag["object_type"] != "tag":
        resolution["errors"].append(
            {"node_id": "g3_approval_tag", "category": "lightweight_g3_approval_tag"}
        )
    elif g3_tag["peeled_commit"] != target_commit:
        resolution["errors"].append(
            {"node_id": "g3_approval_tag", "category": "g3_target_mismatch"}
        )
    return (
        objects,
        g3_tag,
        resolution,
        reviewer_authorities,
        review_artifacts,
        reviewer_actor_attestations,
        authority_enrollment,
        reviewer_bootstrap_root,
        reviewer_bootstrap_currentness,
        remediation_authority_id,
        remediation_actor_identity_digest,
    )


def make_repository_context(
    repository: Path | None = None,
    target_ref: str = "HEAD",
    allow_test_only: bool = False,
) -> G3AuthorityContext:
    repository = ROOT if repository is None else Path(repository).resolve()
    target = _git_output(repository, ["rev-parse", target_ref]).decode().strip()
    graph_relative_path = str(AUTHORITY_GRAPH_PATH.relative_to(ROOT))
    graph_bytes = _git_output(repository, ["show", f"{target}:{graph_relative_path}"])
    graph = _parse_json_without_duplicates(graph_bytes)
    if not isinstance(graph, dict):
        raise G3ValidationError("authority_graph_bytes_malformed")
    validate_r12_authority_graph(graph)
    (
        objects,
        tag,
        resolution,
        reviewer_authorities,
        review_artifacts,
        reviewer_actor_attestations,
        authority_enrollment,
        reviewer_bootstrap_root,
        reviewer_bootstrap_currentness,
        remediation_authority_id,
        remediation_actor_identity_digest,
    ) = _resolve_real_authority(repository, graph, target, allow_test_only)
    component_nodes = [
        component_node_id(prefix) for prefix in SPECS
    ]
    component_sha_by_node = {
        node_id: objects[node_id]["sha256"]
        for node_id in component_nodes
        if node_id in objects
    }
    component_sha256s = [
        component_sha_by_node.get(node_id, "")
        for node_id in component_nodes
    ]
    architecture_plan_sha256 = (
        objects["architecture_plan"]["sha256"]
        if "architecture_plan" in objects
        else None
    )
    f0_decisions_sha256 = (
        objects["f0_decision_bundle"]["sha256"]
        if "f0_decision_bundle" in objects
        else None
    )
    return G3AuthorityContext(
        mode="real_test" if allow_test_only else "real",
        graph=graph,
        objects=objects,
        bundle_manifest_sha256=objects.get("specification_bundle_manifest", {}).get("sha256"),
        aggregate_review_sha256=objects.get("aggregate_review", {}).get("sha256"),
        expected_target_commit=target,
        tag=tag,
        component_sha256s=component_sha256s,
        component_sha_by_node=component_sha_by_node,
        architecture_plan_sha256=architecture_plan_sha256 or "",
        f0_decisions_sha256=f0_decisions_sha256,
        authority_graph_sha256=sha256_bytes(graph_bytes),
        authority_graph_bytes=graph_bytes,
        allow_test_only_authority=allow_test_only,
        reviewer_authorities=reviewer_authorities,
        review_artifacts=review_artifacts,
        authority_enrollment=authority_enrollment,
        reviewer_bootstrap_root=reviewer_bootstrap_root,
        reviewer_bootstrap_currentness=reviewer_bootstrap_currentness,
        reviewer_actor_attestations=reviewer_actor_attestations,
        remediation_authority_id=remediation_authority_id,
        remediation_actor_identity_digest=remediation_actor_identity_digest,
        resolution=resolution,
    )


def _fixture_git(repository: Path, arguments: list[str], input_bytes: bytes | None = None) -> bytes:
    return subprocess.check_output(
        ["git", *arguments], cwd=repository, input=input_bytes, stderr=subprocess.PIPE
    )


def _fixture_authority_payload(
    graph: dict[str, Any], node_id: str, **fields: Any
) -> dict[str, Any]:
    return {
        "node_id": node_id,
        "authority_kind": _graph_nodes(graph)[node_id]["authority_kind"],
        "schema_version": 1,
        "lifecycle": "ACTIVE",
        "stale": False,
        "superseded_by": None,
        "invalidated": False,
        **fields,
    }


def _fixture_write_authority(
    repository: Path, graph: dict[str, Any], node_id: str, payload: dict[str, Any]
) -> str:
    rule = graph["node_identity_rules"][node_id]
    if _graph_nodes(graph)[node_id]["authority_kind"] == "PhaseFIndependentReviewBundleV1":
        if set(payload) != INDEPENDENT_REVIEW_BUNDLE_FIELDS:
            raise ValueError(f"fixture review bundle schema mismatch: {node_id}")
        digest = sha256_bytes(canonical_json_bytes(payload))
        relative_path = rule["path"]
        path = repository / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(canonical_json_bytes(payload))
        return digest
    if rule["type"] == "canonical_object_sha256_excluding_field":
        identity_payload = {
            key: value
            for key, value in payload.items()
            if key not in rule["exclude_fields"]
        }
        digest = sha256_bytes(canonical_json_bytes(identity_payload))
        payload = {**payload, "migrated_finding_review_id": digest}
    else:
        digest = sha256_bytes(canonical_json_bytes(payload))
    relative_path = rule.get("path") or rule["authority_path"]
    path = repository / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_json_bytes(payload))
    return digest


def _fixture_write_reference(
    repository: Path,
    graph: dict[str, Any],
    reference_type: str,
    payload: dict[str, Any],
) -> str:
    contract = _review_reference_contract(graph)[reference_type]
    identity_payload = {
        key: value
        for key, value in payload.items()
        if key not in contract["digest_excluded_fields"]
    }
    digest = sha256_bytes(canonical_json_bytes(identity_payload))
    complete = {contract["id_field"]: digest, **payload}
    path_template = contract.get("authority_path_template")
    relative_path = (
        path_template.replace("{reviewer_authority_id}", digest)
        .replace("{review_artifact_id}", digest)
        if isinstance(path_template, str)
        else contract["authority_path"]
    )
    path = repository / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_json_bytes(complete))
    return digest


def _fixture_annotated_tag(
    repository: Path, tag_name: str, target_commit: str, message: bytes
) -> None:
    raw = (
        f"object {target_commit}\n"
        "type commit\n"
        f"tag {tag_name}\n"
        "tagger Phase F Test <phase-f-test@example.invalid> 1 +0000\n"
        "\n"
    ).encode("ascii") + message
    tag_object = _fixture_git(repository, ["mktag"], raw).decode().strip()
    _fixture_git(repository, ["update-ref", f"refs/tags/{tag_name}", tag_object])


def _isolated_real_fixture(
    populate_authority: bool = True,
    authority_class: str = "TEST_ONLY",
) -> tuple[tempfile.TemporaryDirectory[str], Path, str, bytes]:
    graph = json.loads(AUTHORITY_GRAPH_PATH.read_text())
    bootstrap_root_seed = b"phase-f-r12-fixture-bootstrap-root-01"
    bootstrap_verifier_seed = b"phase-f-r12-fixture-bootstrap-verifier"
    bootstrap_root_public_key, _ = _fixture_ed25519_keypair(bootstrap_root_seed)
    bootstrap_verifier_public_key, _ = _fixture_ed25519_keypair(
        bootstrap_verifier_seed
    )
    bootstrap_root = {
        "root_id": "",
        "authority_kind": "PhaseFReviewerBootstrapTrustRootV1",
        "schema_version": 1,
        "authority_class": authority_class,
        "stage": REVIEWER_BOOTSTRAP_STAGE,
        "root_public_key": bootstrap_root_public_key,
        "root_public_key_fingerprint": sha256_bytes(
            bytes.fromhex(bootstrap_root_public_key)
        ),
        "authority_scope": REVIEWER_BOOTSTRAP_SCOPE,
        "subject_uniqueness_policy": "one_natural_person_one_subject",
        "evidence_retention_policy": "retain_external_identity_evidence_hash_binding",
        "rotation_policy": "forward_signed_replacement_only",
        "compromise_policy": "immediate_reject",
        "lifecycle": "ACTIVE",
        "stale": False,
        "superseded_by": None,
        "invalidated": False,
    }
    bootstrap_root["root_id"] = reviewer_bootstrap_root_id(bootstrap_root)
    graph["reviewer_bootstrap_trust_contract"]["root_id"] = bootstrap_root[
        "root_id"
    ]
    graph["reviewer_bootstrap_trust_contract"][
        "root_public_key_fingerprint"
    ] = bootstrap_root["root_public_key_fingerprint"]
    subject_bindings = [
        {
            "actor_subject_id": f"fixture-natural-person-{index}",
            "identity_evidence_sha256": sha256_bytes(
                f"fixture-identity-evidence-{index}".encode()
            ),
            "subject_status": "ACTIVE",
        }
        for index in range(1, len(REVIEW_ROLE_ORDER) + 1)
    ]
    bootstrap_currentness = {
        "currentness_proof_id": "",
        "authority_kind": "PhaseFReviewerBootstrapCurrentnessProofV1",
        "schema_version": 1,
        "authority_class": authority_class,
        "stage": REVIEWER_BOOTSTRAP_STAGE,
        "root_id": bootstrap_root["root_id"],
        "root_sha256": sha256_bytes(canonical_json_bytes(bootstrap_root)),
        "sequence": 0,
        "previous_proof_id": None,
        "head_id": "",
        "current_verifier_authority_id": "fixture-bootstrap-verifier",
        "current_verifier_public_key": bootstrap_verifier_public_key,
        "current_verifier_public_key_fingerprint": sha256_bytes(
            bytes.fromhex(bootstrap_verifier_public_key)
        ),
        "subject_registry_head_sha256": reviewer_bootstrap_subject_registry_head_sha256(
            0, subject_bindings
        ),
        "subject_bindings": subject_bindings,
        "valid_from": "2020-01-01T00:00:00Z",
        "valid_until": "2099-12-31T23:59:59Z",
        "root_lifecycle": "ACTIVE",
        "root_revoked": False,
        "root_compromised": False,
        "root_superseded_by": None,
        "verifier_lifecycle": "ACTIVE",
        "verifier_revoked": False,
        "verifier_compromised": False,
        "verifier_superseded_by": None,
        "lifecycle": "ACTIVE",
        "stale": False,
        "superseded_by": None,
        "invalidated": False,
        "signature": "",
    }
    bootstrap_currentness["head_id"] = reviewer_bootstrap_currentness_head_id(
        bootstrap_currentness
    )
    bootstrap_currentness["currentness_proof_id"] = (
        reviewer_bootstrap_currentness_proof_id(bootstrap_currentness)
    )
    bootstrap_currentness["signature"] = _fixture_ed25519_sign(
        bootstrap_root_seed,
        REVIEWER_BOOTSTRAP_CURRENTNESS_DOMAIN
        + canonical_jcs_bytes(
            {
                key: value
                for key, value in bootstrap_currentness.items()
                if key != "signature"
            }
        ),
    )
    bootstrap_root_bytes = canonical_json_bytes(bootstrap_root)
    bootstrap_currentness_bytes = canonical_json_bytes(bootstrap_currentness)
    review_root = Path.home() / "Library" / "Caches" / "Codex" / "reviews" / "rust_electroanalysis_cli"
    review_root.mkdir(parents=True, exist_ok=True)
    temporary = tempfile.TemporaryDirectory(
        prefix="phase-f-r12-authority-", dir=review_root
    )
    repository = Path(temporary.name)
    _fixture_git(repository, ["init", "-q"])
    _fixture_git(repository, ["config", "user.name", "Phase F Test"])
    _fixture_git(repository, ["config", "user.email", "phase-f-test@example.invalid"])
    graph_destination = repository / AUTHORITY_GRAPH_PATH.relative_to(ROOT)
    graph_destination.parent.mkdir(parents=True, exist_ok=True)
    graph_bytes = canonical_json_bytes(graph)
    graph_destination.write_bytes(graph_bytes)
    for rule in graph["node_identity_rules"].values():
        if rule["type"] == "repository_file_sha256":
            source = ROOT / rule["path"]
            destination = repository / rule["path"]
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(source.read_bytes())
    _fixture_git(repository, ["add", "."])
    _fixture_git(repository, ["commit", "-qm", "fixture source input"])
    target_commit = _fixture_git(repository, ["rev-parse", "HEAD"]).decode().strip()
    if not populate_authority:
        return temporary, repository, target_commit, b""
    bootstrap_root_path = repository / ".phase_f_authority/reviewer_bootstrap/trust_root.json"
    bootstrap_currentness_path = repository / ".phase_f_authority/reviewer_bootstrap/currentness_proof.json"
    bootstrap_root_path.parent.mkdir(parents=True, exist_ok=True)
    bootstrap_root_path.write_bytes(bootstrap_root_bytes)
    bootstrap_currentness_path.write_bytes(bootstrap_currentness_bytes)
    nodes = _graph_nodes(graph)
    lifecycle_fields = {
        "lifecycle": "ACTIVE",
        "stale": False,
        "superseded_by": None,
        "invalidated": False,
    }
    remediation_actor_identity_digest = reviewer_actor_identity_digest(
        "fixture-remediation-author"
    )
    remediation_authority_id = _fixture_write_reference(
        repository,
        graph,
        "remediation_author",
        {
            "authority_kind": "PhaseFImplementationAuthorIdentityV1",
            "schema_version": 1,
            "authority_class": authority_class,
            "actor_identity_digest": remediation_actor_identity_digest,
            **lifecycle_fields,
        },
    )

    owner_seed = b"phase-f-r12-fixture-owner-seed-0001"
    registry_seed = b"phase-f-r12-fixture-registry-seed-01"
    owner_public_key, _ = _fixture_ed25519_keypair(owner_seed)
    registry_public_key, _ = _fixture_ed25519_keypair(registry_seed)
    enrollment_payload = {
        "schema_version": 1,
        "enrollment_id": "",
        "phase_f_plan_tag": G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"],
        "f0_decisions_tag": G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"],
        "readiness_tag": "ism-mechanism-health-v1-f-readiness-approved",
        "owner_authority_id": "fixture-owner",
        "registry_authority_id": "fixture-registry",
        "owner_public_key": owner_public_key,
        "registry_public_key": registry_public_key,
        "owner_public_key_fingerprint": sha256_bytes(bytes.fromhex(owner_public_key)),
        "registry_public_key_fingerprint": sha256_bytes(bytes.fromhex(registry_public_key)),
        "owner_authority_document": {
            "immutable_uri": "phase-f-test://owner-document",
            "sha256": "d" * 64,
            "byte_length": "0",
        },
        "registry_authority_document": {
            "immutable_uri": "phase-f-test://registry-document",
            "sha256": "e" * 64,
            "byte_length": "0",
        },
        "custody_policy_sha256": "f" * 64,
        "created_at": "2026-01-01T00:00:00Z",
    }
    enrollment_payload["enrollment_id"] = authority_enrollment_id(enrollment_payload)
    enrollment_path = repository / ".phase_f_authority/authority_enrollment.json"
    enrollment_path.parent.mkdir(parents=True, exist_ok=True)
    enrollment_path.write_bytes(canonical_json_bytes(enrollment_payload))
    enrollment_file_sha256 = sha256_bytes(enrollment_path.read_bytes())

    actor_attestation_ids: dict[str, str] = {}
    actor_attestation_references: dict[str, dict[str, str]] = {}
    for index, role in enumerate(REVIEW_ROLE_ORDER, start=1):
        attestation = {
            "attestation_id": "",
            "authority_kind": "PhaseFReviewerActorAttestationV1",
            "schema_version": 1,
            "actor_subject_id": f"fixture-natural-person-{index}",
            "actor_class": "natural_person",
            "actor_identity_evidence_sha256": sha256_bytes(
                f"fixture-identity-evidence-{index}".encode()
            ),
            "trust_source": {
                "type": REVIEWER_BOOTSTRAP_TRUST_SOURCE,
                "root_id": bootstrap_root["root_id"],
                "root_sha256": sha256_bytes(bootstrap_root_bytes),
                "currentness_proof_id": bootstrap_currentness[
                    "currentness_proof_id"
                ],
                "currentness_proof_sha256": sha256_bytes(
                    bootstrap_currentness_bytes
                ),
            },
            "eligible_role": role,
            "role_eligibility_evidence_sha256": sha256_bytes(
                f"fixture-role-evidence-{role}".encode()
            ),
            "independence_evidence_sha256": sha256_bytes(
                f"fixture-independence-evidence-{index}".encode()
            ),
            "independence_excluded_actor_identity_digest": remediation_actor_identity_digest,
            "eligibility_verifier_authority_id": bootstrap_currentness[
                "current_verifier_authority_id"
            ],
            "independence_verifier_authority_id": bootstrap_currentness[
                "current_verifier_authority_id"
            ],
            "created_at": "2026-01-01T00:00:00Z",
            **lifecycle_fields,
            "signature": "",
        }
        attestation["attestation_id"] = reviewer_actor_attestation_id(attestation)
        signing_payload = {
            key: value for key, value in attestation.items() if key != "signature"
        }
        attestation["signature"] = _fixture_ed25519_sign(
            bootstrap_verifier_seed,
            REVIEWER_ACTOR_ATTESTATION_DOMAIN + canonical_jcs_bytes(signing_payload),
        )
        attestation_path = repository / ".phase_f_authority/reviewer_actor_attestations" / (
            f"{attestation['attestation_id']}.json"
        )
        attestation_path.parent.mkdir(parents=True, exist_ok=True)
        attestation_path.write_bytes(canonical_json_bytes(attestation))
        attestation_id = attestation["attestation_id"]
        actor_attestation_ids[role] = attestation_id
        attestation_bytes = attestation_path.read_bytes()
        actor_attestation_references[role] = {
            "immutable_uri": f"{REVIEWER_ACTOR_ATTESTATION_URI_PREFIX}{attestation_id}",
            "sha256": sha256_bytes(attestation_bytes),
            "byte_length": str(len(attestation_bytes)),
        }

    def source_digest(node_id: str) -> str:
        rule = graph["node_identity_rules"][node_id]
        return sha256(repository / rule["path"])

    source_records = {
        node_id: {
            "node_id": node_id,
            "authority_kind": nodes[node_id]["authority_kind"],
            "schema_version": 1,
            "sha256": source_digest(node_id),
            "target": None,
        }
        for node_id, rule in graph["node_identity_rules"].items()
        if rule["type"] == "repository_file_sha256"
    }

    reviewer_ids: dict[str, str] = {}
    for role in REVIEW_ROLE_ORDER:
        reviewer_ids[role] = _fixture_write_reference(
            repository,
            graph,
            "reviewer",
            {
                "authority_kind": "PhaseFReviewerIdentityV1",
                "schema_version": 1,
                "authority_class": authority_class,
                "actor_identity_digest": reviewer_actor_identity_digest(
                    f"fixture-natural-person-{list(REVIEW_ROLE_ORDER).index(role) + 1}"
                ),
                "actor_attestation_id": actor_attestation_ids[role],
                "actor_attestation_reference": actor_attestation_references[role],
                "permitted_review_roles": [role],
                **lifecycle_fields,
            },
        )

    def write_review_bundle(node_id: str, scope_sha: str) -> str:
        review_sources = [
            edge["from"]
            for edge in graph["edges"]
            if edge["to"] == node_id and edge["type"] == "reviews"
        ]
        if len(review_sources) != 1:
            raise AssertionError(f"fixture review target source mismatch: {node_id}")
        review_target = _review_target_for_source(
            nodes[review_sources[0]]["authority_kind"], target_commit, scope_sha
        )
        rows: list[dict[str, Any]] = []
        for role in REVIEW_ROLE_ORDER:
            artifact_id = _fixture_write_reference(
                repository,
                graph,
                "artifact",
                {
                    "authority_kind": "PhaseFReviewArtifactV1",
                    "schema_version": 1,
                    "authority_class": authority_class,
                    "reviewer_authority_id": reviewer_ids[role],
                    "role": role,
                    "reviewed_target": scope_sha,
                    "decision": "GO",
                    "p0_count": "0",
                    "p1_count": "0",
                    "p2_count": "0",
                    "finding_ids": [],
                    "independence_relation": {
                        "type": "distinct_reviewer_authority",
                        "reviewer_authority_id": reviewer_ids[role],
                    },
                    **lifecycle_fields,
                },
            )
            artifact_path = repository / graph["review_reference_contract"]["artifact"][
                "authority_path_template"
            ].replace("{review_artifact_id}", artifact_id)
            artifact_bytes = artifact_path.read_bytes()
            rows.append(
                {
                    "role": role,
                    "decision": "GO",
                    "p0_count": "0",
                    "p1_count": "0",
                    "finding_ids": [],
                    "review_artifact_reference": {
                        "immutable_uri": f"{REVIEW_ARTIFACT_URI_PREFIX}{artifact_id}",
                        "sha256": sha256_bytes(artifact_bytes),
                        "byte_length": str(len(artifact_bytes)),
                    },
                }
            )
        bundle = {
            "schema_version": 1,
            "review_bundle_id": "",
            "target": review_target,
            "reviews": rows,
            "aggregate_p0_count": "0",
            "aggregate_p1_count": "0",
            "aggregate_decision": "GO",
        }
        bundle["review_bundle_id"] = independent_review_bundle_id(bundle)
        return _fixture_write_authority(repository, graph, node_id, bundle)

    arch_review_sha = write_review_bundle(
        "architecture_review", source_records["architecture_plan"]["sha256"]
    )
    arch_approval_payload = _fixture_authority_payload(
        graph, "architecture_approval", authority_id="fixture:architecture-approval",
        tag_name=G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"],
        target_sha256=source_records["architecture_plan"]["sha256"],
        review_sha256=arch_review_sha, decision="GO", p0_count=0, p1_count=0,
    )
    arch_approval_sha = _fixture_write_authority(
        repository, graph, "architecture_approval", arch_approval_payload
    )
    f0_bundle_payload = _fixture_authority_payload(
        graph, "f0_decision_bundle", authority_id="fixture:f0-decision-bundle"
    )
    f0_bundle_sha = _fixture_write_authority(
        repository, graph, "f0_decision_bundle", f0_bundle_payload
    )
    f0_review_sha = write_review_bundle("f0_review", f0_bundle_sha)
    f0_approval_payload = _fixture_authority_payload(
        graph, "f0_approval", authority_id="fixture:f0-approval",
        tag_name=G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"],
        target_sha256=f0_bundle_sha, review_sha256=f0_review_sha,
        decision="GO", p0_count=0, p1_count=0,
    )
    f0_approval_sha = _fixture_write_authority(
        repository, graph, "f0_approval", f0_approval_payload
    )
    authority_records = {
        "architecture_approval": {
            "authority_id": "fixture:architecture-approval",
            "sha256": arch_approval_sha,
            "target": source_records["architecture_plan"]["sha256"],
        },
        "f0_approval": {
            "authority_id": "fixture:f0-approval",
            "sha256": f0_approval_sha,
            "target": f0_bundle_sha,
        },
    }
    input_bindings = {}
    for edge in _graph_edges_for(graph, "specification_bundle_inputs", "binds"):
        source = edge["from"]
        input_bindings[source] = (
            authority_records[source]
            if source in authority_records
            else {
                "authority_id": None,
                "sha256": source_records[source]["sha256"],
                "target": None,
            }
        )
    input_payload = _fixture_authority_payload(
        graph, "specification_bundle_inputs",
        authority_id="fixture:bundle-inputs",
        authority_graph_sha256=sha256_bytes(graph_bytes),
        authority_bindings=input_bindings,
    )
    input_sha = _fixture_write_authority(
        repository, graph, "specification_bundle_inputs", input_payload
    )
    component_sha_by_node = {
        node_id: source_records[node_id]["sha256"]
        for node_id in (
            "component_wire_spec", "component_scientific_spec",
            "component_operations_spec", "component_conformance_spec",
            "component_implementation_spec",
        )
    }
    component_review_sha = {
        node_id: write_review_bundle(node_id, component_sha_by_node[spec_node])
        for node_id, spec_node in (
            ("component_wire_review", "component_wire_spec"),
            ("component_scientific_review", "component_scientific_spec"),
            ("component_operations_review", "component_operations_spec"),
            ("component_conformance_review", "component_conformance_spec"),
            ("component_implementation_review", "component_implementation_spec"),
        )
    }
    migrated = _fixture_authority_payload(
        graph, "migrated_finding_review",
        target_git_commit=target_commit,
        target_bundle_inputs_sha256=input_sha,
        reviewed_migration_ledger_sha256=source_records["migration_ledger"]["sha256"],
        reviewed_normative_traceability_matrix_sha256=source_records["normative_traceability_matrix"]["sha256"],
        reviewed_traceability_manifest_sha256=source_records["generated_traceability_manifest"]["sha256"],
        reviewed_component_sha256s=sorted(component_sha_by_node.values()),
        reviewed_finding_ids=sorted(EXPECTED_MIGRATED_FINDINGS),
        finding_dispositions={
            finding_id: "TECHNICALLY_CLOSED"
            for finding_id in sorted(EXPECTED_MIGRATED_FINDINGS)
        },
        reviewer_roles=sorted(REVIEW_ROLES),
        review_records=[],
        review_input_fingerprint="",
        p0_count=0, p1_count=0, p2_count=0, decision="GO",
        created_stage=10, producer="independent_review_panel",
        validator="validate_migrated_finding_review",
    )
    migrated["review_input_fingerprint"] = _migrated_review_input_fingerprint(migrated)
    migrated["review_records"] = []
    for role in sorted(REVIEW_ROLES):
        artifact_id = _fixture_write_reference(
            repository,
            graph,
            "artifact",
            {
                "authority_kind": "PhaseFReviewArtifactV1",
                "schema_version": 1,
                "authority_class": authority_class,
                "reviewer_authority_id": reviewer_ids[role],
                "role": role,
                "reviewed_target": migrated["review_input_fingerprint"],
                "decision": "GO",
                "p0_count": "0",
                "p1_count": "0",
                "p2_count": "0",
                "finding_ids": [],
                "independence_relation": {
                    "type": "distinct_reviewer_authority",
                    "reviewer_authority_id": reviewer_ids[role],
                },
                **lifecycle_fields,
            },
        )
        row: dict[str, Any] = {
            "role": role,
            "reviewer_authority_id": reviewer_ids[role],
            "reviewed_target": migrated["review_input_fingerprint"],
            "review_artifact_id": artifact_id,
            "decision": "GO",
            "review_sha256": "",
            "lifecycle": "ACTIVE",
            "independence_relation": {
                "type": "distinct_reviewer_authority",
                "reviewer_authority_id": reviewer_ids[role],
            },
        }
        row_payload = dict(row)
        row_payload.pop("review_sha256")
        row["review_sha256"] = sha256_bytes(canonical_json_bytes(row_payload))
        migrated["review_records"].append(row)
    migrated_sha = _fixture_write_authority(
        repository, graph, "migrated_finding_review", migrated
    )
    bound_authorities = {
        source: (
            component_review_sha[source]
            if source in component_review_sha
            else source_records[source]["sha256"]
            if source in source_records
            else authority_records[source]["sha256"]
            if source in authority_records
            else migrated_sha
        )
        for source in sorted(
            edge["from"]
            for edge in _graph_edges_for(graph, "specification_bundle_manifest", "binds")
        )
    }
    manifest = _fixture_authority_payload(
        graph, "specification_bundle_manifest",
        authority_id="fixture:bundle-manifest", status="READY_FOR_G3",
        eligible_for_g3=True, target_commit=target_commit,
        bundle_input_fingerprint_sha256=input_sha,
        bound_authority_sha256s=bound_authorities,
    )
    manifest_sha = _fixture_write_authority(
        repository, graph, "specification_bundle_manifest", manifest
    )
    aggregate_sha = write_review_bundle("aggregate_review", manifest_sha)
    if "implementation_readiness_specification" in source_records:
        write_review_bundle(
            "readiness_review",
            source_records["implementation_readiness_specification"]["sha256"],
        )
    _fixture_git(repository, ["add", "."])
    _fixture_git(repository, ["commit", "-qm", "fixture authority objects"])
    _fixture_annotated_tag(
        repository, G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"],
        target_commit,
        (
            f"authority_node_id=architecture_approval\n"
            f"authority_id=fixture:architecture-approval\n"
            f"authority_sha256={arch_approval_sha}\n"
            f"target_git_commit={target_commit}\n"
            "schema_version=1\n"
        ).encode("ascii"),
    )
    _fixture_annotated_tag(
        repository, G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"],
        target_commit,
        (
            f"authority_node_id=f0_approval\n"
            f"authority_id=fixture:f0-approval\n"
            f"authority_sha256={f0_approval_sha}\n"
            f"target_git_commit={target_commit}\n"
            "schema_version=1\n"
        ).encode("ascii"),
    )
    _fixture_annotated_tag(
        repository,
        AUTHORITY_ENROLLMENT_APPROVAL_TAG,
        target_commit,
        (
            f"phase_f_plan_tag={enrollment_payload['phase_f_plan_tag']}\n"
            f"f0_decisions_tag={enrollment_payload['f0_decisions_tag']}\n"
            f"readiness_tag={enrollment_payload['readiness_tag']}\n"
            f"readiness_main_sha={target_commit}\n"
            f"enrollment_sha256={enrollment_file_sha256}\n"
            f"owner_authority_id={enrollment_payload['owner_authority_id']}\n"
            f"registry_authority_id={enrollment_payload['registry_authority_id']}\n"
            f"owner_public_key_fingerprint={enrollment_payload['owner_public_key_fingerprint']}\n"
            f"registry_public_key_fingerprint={enrollment_payload['registry_public_key_fingerprint']}\n"
            f"review_bundle_sha256={'a' * 64}\n"
            "approval_decision=GO\n"
        ).encode("ascii"),
    )
    g3_body = (
        G3_FIXTURE_BODY.replace(b"0" * 64, manifest_sha.encode(), 1)
        .replace(b"1" * 64, aggregate_sha.encode(), 1)
    )
    _fixture_annotated_tag(repository, G3_TAG_NAME, target_commit, g3_body)
    return temporary, repository, target_commit, g3_body


def run_regression_self_tests() -> None:
    global _validate_graph_object_bindings

    trace = build_traceability()
    entries = trace["requirements"]
    matrix = load_normative_matrix()
    test_catalog, evidence_catalog = load_reference_catalogs()
    graph = json.loads(AUTHORITY_GRAPH_PATH.read_text())

    def reject_value_error(label: str, operation: object) -> None:
        try:
            operation()
        except ValueError:
            return
        raise AssertionError(f"regression did not reject: {label}")

    def must_reject(label: str, mutate: object) -> None:
        mutant = deepcopy(entries)
        mutate(mutant)
        reject_value_error(
            label,
            lambda: validate_reference_catalogs(mutant, test_catalog, evidence_catalog),
        )

    must_reject(
        "undefined test ID",
        lambda rows: rows[0]["test_ids"].append("R12-UNDEFINED-TEST"),
    )
    must_reject(
        "undefined KAT/fixture ID",
        lambda rows: rows[0]["test_ids"].append("R12-UNDEFINED-KAT"),
    )
    must_reject(
        "undefined evidence ID",
        lambda rows: next(
            row for row in rows if row["requirement_id"] == "F-SCI-001"
        )["future_real_evidence_ids"].append("EV11-UNDEFINED"),
    )

    r12_text = SPECS["F-CNF"].read_text()
    catalog_row = next(
        line for line in r12_text.splitlines() if line.startswith("| R12-")
    )
    reject_value_error(
        "duplicate R12 catalog ID",
        lambda: parse_r12_test_catalog(
            r12_text.replace(catalog_row, f"{catalog_row}\n{catalog_row}", 1)
        ),
    )

    def semantic_reject(label: str, mutate: object) -> None:
        mutant = deepcopy(entries)
        mutate(mutant)
        reject_value_error(
            label,
            lambda: validate_semantic_traceability(
                mutant, matrix, test_catalog, evidence_catalog
            ),
        )

    semantic_reject(
        "catalog-valid semantic substitution",
        lambda rows: next(
            row for row in rows if row["requirement_id"] == "F-OPS-004"
        ).update(
            {
                "test_ids": ["R11-CAT"],
                "kat_ids": [],
                "constructive_audit_ids": ["R11-CAT"],
                "property_test_ids": [],
                "future_real_evidence_ids": ["EV11-01"],
            }
        ),
    )
    semantic_reject(
        "wrong KAT mapping",
        lambda rows: next(
            row for row in rows if row["requirement_id"] == "F-CNF-004"
        )["kat_ids"].__setitem__(0, "R11-POS-TRUST"),
    )
    semantic_reject(
        "wrong evidence mapping",
        lambda rows: next(
            row for row in rows if row["requirement_id"] == "F-SCI-001"
        ).update({"future_real_evidence_ids": ["EV11-01"]}),
    )
    semantic_reject(
        "wrong audit mapping",
        lambda rows: next(
            row for row in rows if row["requirement_id"] == "F-CNF-005"
        ).update({"constructive_audit_ids": ["R11-CX-01"]}),
    )
    semantic_reject(
        "wrong test category",
        lambda rows: next(
            row for row in rows if row["requirement_id"] == "F-OPS-003"
        ).update(
            {
                "kat_ids": list(
                    next(
                        row
                        for row in rows
                        if row["requirement_id"] == "F-OPS-003"
                    )["property_test_ids"]
                ),
                "property_test_ids": [],
            }
        ),
    )

    def swap_requirement_mapping(rows: list[dict[str, Any]]) -> None:
        left = next(row for row in rows if row["requirement_id"] == "F-OPS-003")
        right = next(row for row in rows if row["requirement_id"] == "F-OPS-004")
        for field in (
            "test_ids",
            "kat_ids",
            "constructive_audit_ids",
            "property_test_ids",
            "future_real_evidence_ids",
            "schema_ids",
        ):
            left[field], right[field] = deepcopy(right[field]), deepcopy(left[field])

    semantic_reject("cross-requirement mapping swap", swap_requirement_mapping)

    def add_extra_mapping(rows: list[dict[str, Any]]) -> None:
        row = next(row for row in rows if row["requirement_id"] == "F-ARCH-001")
        extra_id = next(test_id for test_id in sorted(test_catalog) if test_id not in row["test_ids"])
        row["test_ids"].append(extra_id)
        row["constructive_audit_ids"].append(extra_id)

    semantic_reject("extra mapping", add_extra_mapping)

    def remove_mapping(rows: list[dict[str, Any]]) -> None:
        row = next(row for row in rows if row["requirement_id"] == "F-ARCH-001")
        removed = row["test_ids"].pop()
        for field in ("kat_ids", "constructive_audit_ids", "property_test_ids"):
            if removed in row[field]:
                row[field].remove(removed)

    semantic_reject("missing mapping", remove_mapping)

    schema_mutant = deepcopy(matrix)
    for schema_row in schema_mutant:
        schema_row["schema_ids"] = [
            schema_id
            for schema_id in schema_row["schema_ids"]
            if schema_id != "PhaseFMigratedFindingReviewV1"
        ]
    reject_value_error("schema inverse omission", lambda: validate_schema_usage(schema_mutant))

    validate_r12_authority_graph(graph)

    def graph_reject(label: str, mutate: object) -> None:
        mutant = deepcopy(graph)
        mutate(mutant)
        reject_value_error(label, lambda: validate_r12_authority_graph(mutant))

    def semantic_graph_reject(
        label: str, audit_name: str, mutate: object
    ) -> None:
        mutant = deepcopy(graph)
        mutate(mutant)
        nodes = _graph_nodes(mutant)
        edges = _graph_edges(mutant, nodes)
        order = _topological_order(nodes, edges)
        audit = next(
            item
            for item in _semantic_graph_audits(mutant, nodes, edges, order)
            if item["name"] == audit_name
        )
        if audit["passed"] or not audit["violation_path"]:
            raise AssertionError(f"semantic audit did not change: {label}: {audit}")
        reject_value_error(label, lambda: validate_r12_authority_graph(mutant))

    graph_reject(
        "unknown graph node",
        lambda value: value["edges"][0].update({"to": "unknown_node"}),
    )
    graph_reject(
        "unknown graph edge type",
        lambda value: value["edges"][0].update({"type": "unknown_edge"}),
    )
    graph_reject(
        "graph valid-type wrong-semantic binding",
        lambda value: next(
            edge
            for edge in value["edges"]
            if edge["from"] == "architecture_approval"
            and edge["to"] == "specification_bundle_inputs"
        ).update({"type": "requires"}),
    )
    graph_reject(
        "graph missing required typed edge",
        lambda value: value["edges"].remove(
            next(
                edge
                for edge in value["edges"]
                if edge["from"] == "architecture_approval"
                and edge["to"] == "specification_bundle_inputs"
            )
        ),
    )
    graph_reject(
        "duplicate graph node",
        lambda value: value["nodes"].append(deepcopy(value["nodes"][0])),
    )
    graph_reject(
        "graph self edge",
        lambda value: value["edges"].append(
            {"from": "architecture_plan", "to": "architecture_plan", "type": "requires"}
        ),
    )
    graph_reject(
        "graph prerequisite cycle",
        lambda value: value["edges"].extend(
            [
                {"from": "component_wire_spec", "to": "component_scientific_spec", "type": "requires"},
                {"from": "component_scientific_spec", "to": "component_wire_spec", "type": "requires"},
            ]
        ),
    )
    graph_reject(
        "graph hash cycle",
        lambda value: value["edges"].extend(
            [
                {"from": "component_wire_review", "to": "component_scientific_review", "type": "hashes"},
                {"from": "component_scientific_review", "to": "component_wire_review", "type": "hashes"},
            ]
        ),
    )
    graph_reject(
        "graph future-object dependency",
        lambda value: value["edges"].append(
            {"from": "g3_approval_tag", "to": "specification_bundle_manifest", "type": "requires"}
        ),
    )
    graph_reject(
        "graph G3 bypass",
        lambda value: (
            value["g3_required_nodes"].remove("aggregate_review"),
            value["required_inputs"]["g3_approval_tag"].remove("aggregate_review"),
        ),
    )
    graph_reject(
        "graph implementation bypass",
        lambda value: value["required_inputs"].update(
            {"implementation_readiness_specification": []}
        ),
    )
    graph_reject(
        "graph review target cycle",
        lambda value: value["edges"].extend(
            [
                {"from": "component_wire_review", "to": "component_scientific_review", "type": "reviews"},
                {"from": "component_scientific_review", "to": "component_wire_review", "type": "targets"},
            ]
        ),
    )
    graph_reject(
        "graph self-Git identity cycle",
        lambda value: value["edges"].append(
            {"from": "g3_approval_tag", "to": "g3_approval_tag", "type": "hashes"}
        ),
    )
    graph_reject(
        "graph alternative bypass",
        lambda value: value["g3_required_nodes"].remove("migrated_finding_review"),
    )
    graph_reject(
        "graph G3-before-aggregate ordering",
        lambda value: value["edges"].append(
            {"from": "g3_approval_tag", "to": "aggregate_review", "type": "requires"}
        ),
    )
    graph_reject(
        "graph typed edge contract",
        lambda value: value["typed_edge_contract"].pop(),
    )
    graph_reject(
        "graph identity rule contract",
        lambda value: value["identity_rule_contract"]["repository_file_sha256"][
            "required_fields"
        ].append("unmodeled_identity_field"),
    )
    semantic_graph_reject(
        "graph semantic self-Git audit",
        "self_git_cycle",
        lambda value: value["node_identity_rules"].update(
            {
                "g3_approval_tag": {
                    "type": "git_commit_identity",
                    "commit_source": "self",
                }
            }
        ),
    )
    semantic_graph_reject(
        "graph semantic hash-cycle audit",
        "hash_cycle",
        lambda value: (
            value["node_identity_rules"]["migration_ledger"].update(
                {"identity_dependencies": ["normative_traceability_matrix"]}
            ),
            value["node_identity_rules"]["normative_traceability_matrix"].update(
                {"identity_dependencies": ["migration_ledger"]}
            ),
        ),
    )
    semantic_graph_reject(
        "graph semantic review-target audit",
        "review_target_cycle",
        lambda value: value["node_identity_rules"]["component_wire_review"].update(
            {"review_target_node": "aggregate_review"}
        ),
    )
    semantic_graph_reject(
        "graph semantic future-object audit",
        "future_object",
        lambda value: value["node_identity_rules"]["component_wire_review"].update(
            {"identity_dependencies": ["aggregate_review"]}
        ),
    )
    semantic_graph_reject(
        "graph semantic self-reference audit",
        "self_reference",
        lambda value: value["node_identity_rules"]["component_wire_review"].update(
            {"self_reference": True}
        ),
    )
    graph_reject(
        "graph serialized binding equality",
        lambda value: value["serialized_binding_fields"][
            "specification_bundle_manifest"
        ].update({"binds": {}}),
    )

    graph_nodes = _graph_nodes(graph)
    graph_edges = _graph_edges(graph, graph_nodes)
    independent_semantics = derive_required_semantic_rules(
        graph_nodes, graph_edges, graph["object_field_contracts"]
    )
    independent_rules = independent_semantics["serialized_rules"]
    selected_rules = [
        rule
        for rule in independent_rules
        if independent_semantics["relation_policies"][rule["type"]] == "selected"
    ]
    semantic_rule_policy_counts = {
        policy: sum(
            value == policy for value in independent_semantics["relation_policies"].values()
        )
        for policy in sorted(SERIALIZED_BINDING_POLICIES)
    }
    selected_rule_relation_counts = {
        relation: sum(rule["type"] == relation for rule in selected_rules)
        for relation in sorted(GRAPH_EDGE_TYPES)
        if any(rule["type"] == relation for rule in selected_rules)
    }
    selected_rule_target_counts = {
        target: sum(rule["target"] == target for rule in selected_rules)
        for target in sorted({rule["target"] for rule in selected_rules})
    }
    if len(independent_rules) != 20 or len(selected_rules) != 12:
        raise AssertionError(
            "independent semantic-rule inventory has unexpected cardinality: "
            f"total={len(independent_rules)} selected={len(selected_rules)}"
        )
    if derive_required_semantic_rules(
        graph_nodes, graph_edges, graph["object_field_contracts"]
    ) != independent_semantics:
        raise AssertionError("independent semantic-rule derivation is not deterministic")
    binding_projection = derive_binding_projection(graph, graph_nodes, graph_edges)
    binding_contract = derive_serialized_binding_contract(graph, graph_nodes, graph_edges)
    binding_edges = [
        edge
        for edge in graph_edges
        if edge["binding_obligation"]["kind"] == "serialized_binding"
    ]
    none_binding_edges = [
        edge
        for edge in graph_edges
        if edge["binding_obligation"]["kind"] == "none"
    ]
    if len(binding_edges) != len(binding_projection) or len(graph_edges) != 76:
        raise AssertionError(
            "edge binding inventory is not exact: "
            f"edges={len(graph_edges)} binding={len(binding_edges)} "
            f"projection={len(binding_projection)}"
        )

    def raw_edge(value: dict[str, Any], edge: dict[str, Any]) -> dict[str, Any]:
        return next(
            candidate
            for candidate in value["edges"]
            if candidate["from"] == edge["from"]
            and candidate["type"] == edge["type"]
            and candidate["to"] == edge["to"]
        )

    def declared_rule_for_edge(
        value: dict[str, Any], edge: dict[str, Any]
    ) -> dict[str, str]:
        obligation = edge["binding_obligation"]
        return next(
            rule
            for rule in value["binding_semantics"]["serialized_rules"]
            if rule["target"] == edge["to"]
            and rule["type"] == edge["type"]
            and rule["field"] == obligation["destination_field"]
            and rule["category"] == obligation["category"]
            and rule["value"] == obligation["value_semantics"]
            and rule["cardinality"] == obligation["cardinality"]
            and rule["target_object_kind"] == obligation["target_object_kind"]
            and (rule["source"] == "*" or rule["source"] == edge["from"])
        )

    def node_binding_mirror(rule: dict[str, str]) -> dict[str, str]:
        return {
            "field": rule["field"],
            "type": rule["type"],
            "source": rule["source"],
            "category": rule["category"],
            "value": rule["value"],
            "cardinality": rule["cardinality"],
            "target_object_kind": rule["target_object_kind"],
        }

    binding_obligation_structural_deletion_tests = 0
    binding_obligation_structural_malformed_tests = 0
    for edge in binding_edges:
        mutant = deepcopy(graph)
        raw_edge(mutant, edge).pop("binding_obligation")
        binding_obligation_structural_deletion_tests += 1
        reject_value_error(
            f"missing edge binding obligation {edge['from']}/{edge['type']}/{edge['to']}",
            lambda mutant=mutant: validate_r12_authority_graph(mutant),
        )
        for malformed in (
            None,
            {},
            {"kind": "unknown"},
            {"kind": "serialized_binding"},
        ):
            mutant = deepcopy(graph)
            raw_edge(mutant, edge)["binding_obligation"] = malformed
            binding_obligation_structural_malformed_tests += 1
            reject_value_error(
                f"malformed edge binding obligation {edge['from']}/{edge['type']}/{edge['to']}",
                lambda mutant=mutant: validate_r12_authority_graph(mutant),
            )

    node_mirror_downstream_tests = 0
    schema_mirror_downstream_tests = 0
    semantic_rule_downstream_tests = 0
    full_root_fixed_downstream_tests = 0
    for edge in binding_edges:
        rule = declared_rule_for_edge(graph, edge)
        mutant = deepcopy(graph)
        node = next(node for node in mutant["nodes"] if node["id"] == edge["to"])
        node["binding_fields"].remove(node_binding_mirror(rule))
        node_mirror_downstream_tests += 1
        reject_value_error(
            f"root-fixed node mirror shrink {edge['from']}/{edge['type']}/{edge['to']}",
            lambda mutant=mutant: validate_r12_authority_graph(mutant),
        )

        mutant = deepcopy(graph)
        mutant["object_field_contracts"][edge["to"]].remove(rule["field"])
        schema_mirror_downstream_tests += 1
        reject_value_error(
            f"root-fixed schema mirror shrink {edge['from']}/{edge['type']}/{edge['to']}",
            lambda mutant=mutant: validate_r12_authority_graph(mutant),
        )

        mutant = deepcopy(graph)
        mutant["binding_semantics"]["serialized_rules"].remove(rule)
        semantic_rule_downstream_tests += 1
        reject_value_error(
            f"root-fixed semantic mirror shrink {edge['from']}/{edge['type']}/{edge['to']}",
            lambda mutant=mutant: validate_r12_authority_graph(mutant),
        )

        mutant = deepcopy(graph)
        target_relations = mutant["serialized_binding_fields"][rule["target"]][rule["type"]]
        target_relations.pop(rule["source"])
        if not target_relations:
            mutant["serialized_binding_fields"][rule["target"]].pop(rule["type"])
        if not mutant["serialized_binding_fields"][rule["target"]]:
            mutant["serialized_binding_fields"].pop(rule["target"])
        node = next(node for node in mutant["nodes"] if node["id"] == rule["target"])
        node["binding_fields"].remove(node_binding_mirror(rule))
        mutant["binding_semantics"]["serialized_rules"].remove(rule)
        full_root_fixed_downstream_tests += 1
        reject_value_error(
            f"root-fixed full downstream shrink {edge['from']}/{edge['type']}/{edge['to']}",
            lambda mutant=mutant: validate_r12_authority_graph(mutant),
        )

    def add_unauthorized_none_binding(value: dict[str, Any], edge: dict[str, Any]) -> None:
        target = edge["to"]
        field_name = "target_sha256"
        value["object_field_contracts"].setdefault(target, [])
        if field_name not in value["object_field_contracts"][target]:
            value["object_field_contracts"][target].append(field_name)
        node = next(node for node in value["nodes"] if node["id"] == target)
        node["binding_fields"].append(
            {
                "field": field_name,
                "type": edge["type"],
                "source": edge["from"],
                "category": "review_target_binding",
                "value": "source_sha256",
                "cardinality": "exactly_one",
                "target_object_kind": node["authority_kind"],
            }
        )
        value["binding_semantics"]["relation_policies"][edge["type"]][
            "serialized_binding"
        ] = "selected"
        value["binding_semantics"]["serialized_rules"].append(
            {
                "target": target,
                "type": edge["type"],
                "source": edge["from"],
                "field": field_name,
                "category": "review_target_binding",
                "value": "source_sha256",
                "cardinality": "exactly_one",
                "target_object_kind": node["authority_kind"],
            }
        )
        value["serialized_binding_fields"].setdefault(target, {}).setdefault(
            edge["type"], {}
        )[edge["from"]] = field_name

    unauthorized_none_downstream_tests = 0
    for edge in none_binding_edges:
        mutant = deepcopy(graph)
        add_unauthorized_none_binding(mutant, edge)
        unauthorized_none_downstream_tests += 1
        reject_value_error(
            f"unauthorized downstream binding on none edge {edge['from']}/{edge['type']}/{edge['to']}",
            lambda mutant=mutant: validate_r12_authority_graph(mutant),
        )

    def valid_none_to_serialized_root_change(value: dict[str, Any]) -> None:
        edge = raw_edge(
            value,
            {
                "from": "architecture_plan",
                "type": "requires",
                "to": "normative_traceability_matrix",
            },
        )
        edge["binding_obligation"] = {
            "kind": "serialized_binding",
            "destination_field": "reviewed_migration_ledger_sha256",
            "category": "serialized_digest_binding",
            "value_semantics": "source_sha256",
            "cardinality": "exactly_one",
            "target_object_kind": graph_nodes[edge["to"]]["authority_kind"],
        }
        value["object_field_contracts"].setdefault(edge["to"], []).append(
            "reviewed_migration_ledger_sha256"
        )
        refresh_derived_binding_mirrors(value)

    def valid_destination_root_change(value: dict[str, Any]) -> None:
        edge = raw_edge(
            value,
            {
                "from": "migration_ledger",
                "type": "binds",
                "to": "migrated_finding_review",
            },
        )
        edge["binding_obligation"]["destination_field"] = (
            "reviewed_normative_traceability_matrix_sha256"
        )
        refresh_derived_binding_mirrors(value)

    def valid_kind_root_change(value: dict[str, Any]) -> None:
        raw_edge(
            value,
            {
                "from": "architecture_plan",
                "type": "reviews",
                "to": "architecture_review",
            },
        )["binding_obligation"] = {"kind": "none"}
        refresh_derived_binding_mirrors(value)

    root_property_mutators = [
        ("obligation_kind", valid_kind_root_change, True),
        ("destination_field", valid_destination_root_change, True),
        (
            "binding_category",
            lambda value: raw_edge(
                value,
                {
                    "from": "architecture_plan",
                    "type": "reviews",
                    "to": "architecture_review",
                },
            )["binding_obligation"].__setitem__(
                "category", "serialized_digest_binding"
            ),
            False,
        ),
        (
            "destination_field_invalid",
            lambda value: raw_edge(
                value,
                {
                    "from": "architecture_plan",
                    "type": "reviews",
                    "to": "architecture_review",
                },
            )["binding_obligation"].__setitem__("destination_field", "review_sha256"),
            False,
        ),
        (
            "value_semantics",
            lambda value: raw_edge(
                value,
                {
                    "from": "architecture_plan",
                    "type": "reviews",
                    "to": "architecture_review",
                },
            )["binding_obligation"].__setitem__(
                "value_semantics", "authority_descriptor"
            ),
            False,
        ),
        (
            "target_object_kind",
            lambda value: raw_edge(
                value,
                {
                    "from": "architecture_plan",
                    "type": "reviews",
                    "to": "architecture_review",
                },
            )["binding_obligation"].__setitem__(
                "target_object_kind", "PhaseFPlanApprovalV1"
            ),
            False,
        ),
        (
            "cardinality",
            lambda value: raw_edge(
                value,
                {
                    "from": "architecture_plan",
                    "type": "reviews",
                    "to": "architecture_review",
                },
            )["binding_obligation"].__setitem__("cardinality", "one_per_source"),
            False,
        ),
        (
            "unknown_property",
            lambda value: raw_edge(
                value,
                {
                    "from": "architecture_plan",
                    "type": "reviews",
                    "to": "architecture_review",
                },
            )["binding_obligation"].__setitem__("unexpected", True),
            False,
        ),
    ]
    root_identity_mutation_tests = 0
    root_identity_mutations_changed = 0
    valid_root_change_tests = 0
    invalid_root_property_tests = 0

    def refresh_derived_binding_mirrors(value: dict[str, Any]) -> None:
        nodes = _graph_nodes(value)
        edges = _graph_edges(value, nodes)
        independent = derive_required_semantic_rules(
            nodes, edges, value["object_field_contracts"]
        )
        value["binding_semantics"]["relation_policies"] = {
            edge_type: {
                "required_input": True,
                "serialized_binding": policy,
            }
            for edge_type, policy in independent["relation_policies"].items()
        }
        value["binding_semantics"]["serialized_rules"] = deepcopy(
            independent["serialized_rules"]
        )
        derived_nodes = derive_node_binding_fields(nodes, edges)
        for node in value["nodes"]:
            node["binding_fields"] = derived_nodes[node["id"]]
        value["serialized_binding_fields"] = derive_serialized_binding_contract(
            value, nodes, edges
        )

    old_root_sha256 = sha256_bytes(AUTHORITY_GRAPH_PATH.read_bytes())
    for label, mutate, valid in root_property_mutators:
        mutant = deepcopy(graph)
        mutate(mutant)
        root_sha256 = sha256_bytes(canonical_json_bytes(mutant))
        root_identity_mutation_tests += 1
        if root_sha256 == old_root_sha256:
            raise AssertionError(f"binding root identity did not change: {label}")
        root_identity_mutations_changed += 1
        if valid:
            validate_r12_authority_graph(mutant)
            valid_root_change_tests += 1
        else:
            invalid_root_property_tests += 1
            reject_value_error(
                f"invalid binding-root property mutation {label}",
                lambda mutant=mutant: validate_r12_authority_graph(mutant),
            )
    old_bundle_inputs = build_bundle_inputs("4" * 64)
    new_bundle_payload = {
        key: deepcopy(value)
        for key, value in old_bundle_inputs.items()
        if key != "sha256"
    }
    valid_root_graph = deepcopy(graph)
    valid_none_to_serialized_root_change(valid_root_graph)
    valid_root_bytes = canonical_json_bytes(valid_root_graph)
    valid_root_sha256 = sha256_bytes(valid_root_bytes)
    new_bundle_payload["authority_graph_sha256"] = valid_root_sha256
    new_bundle_payload["source_sha256s"]["authority_graph"] = valid_root_sha256
    new_bundle_inputs_sha256 = sha256_bytes(canonical_json_bytes(new_bundle_payload))
    if (
        valid_root_sha256 == old_root_sha256
        or new_bundle_inputs_sha256 == old_bundle_inputs["sha256"]
    ):
        raise AssertionError("binding-root change did not change bundle/input identity")
    relation_map_entries = [
        (target, edge_type)
        for target, relations in graph["serialized_binding_fields"].items()
        for edge_type in relations
    ]
    relation_map_mutation_counts = {
        "delete": 0,
        "empty": 0,
        "rename": 0,
        "move": 0,
        "extra_source": 0,
        "remove_source": 0,
        "wrong_field": 0,
        "duplicate": 0,
    }
    def remove_declared_rule_and_mirror(
        value: dict[str, Any], rule: dict[str, str]
    ) -> None:
        value["binding_semantics"]["serialized_rules"].remove(rule)
        relation_map = value["serialized_binding_fields"][rule["target"]][rule["type"]]
        relation_map.pop(rule["source"])
        if not relation_map:
            value["serialized_binding_fields"][rule["target"]].pop(rule["type"])
        if not value["serialized_binding_fields"][rule["target"]]:
            value["serialized_binding_fields"].pop(rule["target"])
        node = next(node for node in value["nodes"] if node["id"] == rule["target"])
        node["binding_fields"].remove(
            {
                "field": rule["field"],
                "type": rule["type"],
                "source": rule["source"],
                "category": rule["category"],
                "value": rule["value"],
                "cardinality": rule["cardinality"],
                "target_object_kind": rule["target_object_kind"],
            }
        )

    def binding_graph_reject(label: str, mutate: object) -> None:
        mutant = deepcopy(graph)
        mutate(mutant)
        reject_value_error(label, lambda: validate_r12_authority_graph(mutant))

    for target, edge_type in relation_map_entries:
        source_map = graph["serialized_binding_fields"][target][edge_type]
        binding_graph_reject(
            f"serialized binding relation deletion {target}/{edge_type}",
            lambda value, target=target, edge_type=edge_type: value[
                "serialized_binding_fields"
            ][target].pop(edge_type),
        )
        relation_map_mutation_counts["delete"] += 1
        binding_graph_reject(
            f"empty serialized binding relation {target}/{edge_type}",
            lambda value, target=target, edge_type=edge_type: value[
                "serialized_binding_fields"
            ][target].__setitem__(edge_type, {}),
        )
        relation_map_mutation_counts["empty"] += 1
        replacement_relation = next(
            candidate for candidate in sorted(GRAPH_EDGE_TYPES) if candidate != edge_type
        )
        binding_graph_reject(
            f"renamed serialized binding relation {target}/{edge_type}",
            lambda value, target=target, edge_type=edge_type, replacement_relation=replacement_relation: (
                value["serialized_binding_fields"][target].__setitem__(
                    replacement_relation,
                    value["serialized_binding_fields"][target].pop(edge_type),
                )
            ),
        )
        relation_map_mutation_counts["rename"] += 1
        moved_target = next(
            candidate
            for candidate in sorted(graph["serialized_binding_fields"])
            if candidate != target and edge_type not in graph["serialized_binding_fields"][candidate]
        )
        binding_graph_reject(
            f"moved serialized binding relation {target}/{edge_type}",
            lambda value, target=target, edge_type=edge_type, moved_target=moved_target: value[
                "serialized_binding_fields"
            ].setdefault(moved_target, {}).__setitem__(
                edge_type, value["serialized_binding_fields"][target].pop(edge_type)
            ),
        )
        relation_map_mutation_counts["move"] += 1
        nonmatching_source = next(
            node_id
            for node_id in sorted(graph_nodes)
            if node_id != target
            and not any(
                edge["to"] == target
                and edge["type"] == edge_type
                and edge["from"] == node_id
                for edge in graph_edges
            )
        )
        first_field = next(iter(source_map.values()))
        binding_graph_reject(
            f"extra serialized binding source {target}/{edge_type}",
            lambda value, target=target, edge_type=edge_type, nonmatching_source=nonmatching_source, first_field=first_field: value[
                "serialized_binding_fields"
            ][target][edge_type].__setitem__(nonmatching_source, first_field),
        )
        relation_map_mutation_counts["extra_source"] += 1
        source_to_remove = next(iter(source_map))
        binding_graph_reject(
            f"removed serialized binding source {target}/{edge_type}",
            lambda value, target=target, edge_type=edge_type, source_to_remove=source_to_remove: value[
                "serialized_binding_fields"
            ][target][edge_type].pop(source_to_remove),
        )
        relation_map_mutation_counts["remove_source"] += 1
        replacement_field = (
            "review_sha256" if first_field != "review_sha256" else "target_sha256"
        )
        binding_graph_reject(
            f"wrong serialized binding field {target}/{edge_type}",
            lambda value, target=target, edge_type=edge_type, source_to_remove=source_to_remove, replacement_field=replacement_field: value[
                "serialized_binding_fields"
            ][target][edge_type].__setitem__(source_to_remove, replacement_field),
        )
        relation_map_mutation_counts["wrong_field"] += 1
        semantic_rule = next(
            rule
            for rule in graph["binding_semantics"]["serialized_rules"]
            if rule["target"] == target
            and rule["type"] == edge_type
            and rule["source"] in source_map
        )
        binding_graph_reject(
            f"duplicate serialized binding semantic rule {target}/{edge_type}",
            lambda value, semantic_rule=semantic_rule: value[
                "binding_semantics"
            ]["serialized_rules"].append(deepcopy(semantic_rule)),
        )
        relation_map_mutation_counts["duplicate"] += 1

    selected_rule_deletions_tested = 0
    selected_rule_deletion_accepted = 0
    coordinated_rule_mirror_deletions_tested = 0
    coordinated_rule_mirror_deletion_accepted = 0
    for rule in selected_rules:
        rule_only = deepcopy(graph)
        rule_only["binding_semantics"]["serialized_rules"].remove(rule)
        selected_rule_deletions_tested += 1
        try:
            validate_r12_authority_graph(rule_only)
        except ValueError:
            pass
        else:
            selected_rule_deletion_accepted += 1
        coordinated = deepcopy(graph)
        remove_declared_rule_and_mirror(coordinated, rule)
        coordinated_rule_mirror_deletions_tested += 1
        try:
            validate_r12_authority_graph(coordinated)
        except ValueError:
            pass
        else:
            coordinated_rule_mirror_deletion_accepted += 1
    if selected_rule_deletion_accepted or coordinated_rule_mirror_deletion_accepted:
        raise AssertionError(
            "semantic-rule deletion mutation was accepted: "
            f"rule_only={selected_rule_deletion_accepted} "
            f"rule_and_mirror={coordinated_rule_mirror_deletion_accepted}"
        )

    semantic_rule_category_mutations = 0
    semantic_rule_value_mutations = 0
    semantic_rule_field_mutations = 0
    semantic_rule_source_mutations = 0
    semantic_rule_relation_mutations = 0
    for rule_index, semantic_rule in enumerate(graph["binding_semantics"]["serialized_rules"]):
        for replacement_category in sorted(SERIALIZED_BINDING_CATEGORIES - {semantic_rule["category"]}):
            binding_graph_reject(
                f"wrong serialized binding category {rule_index}/{replacement_category}",
                lambda value, rule_index=rule_index, replacement_category=replacement_category: value[
                    "binding_semantics"
                ]["serialized_rules"][rule_index].__setitem__(
                    "category", replacement_category
                ),
            )
            semantic_rule_category_mutations += 1
        for replacement_value in sorted(SERIALIZED_BINDING_VALUE_SOURCES - {semantic_rule["value"]}):
            binding_graph_reject(
                f"wrong serialized binding value semantics {rule_index}/{replacement_value}",
                lambda value, rule_index=rule_index, replacement_value=replacement_value: value[
                    "binding_semantics"
                ]["serialized_rules"][rule_index].__setitem__("value", replacement_value),
            )
            semantic_rule_value_mutations += 1
        target_fields = graph["object_field_contracts"].get(semantic_rule["target"], [])
        replacement_field = next(
            (
                field_name
                for field_name in target_fields
                if field_name != semantic_rule["field"]
                and field_name in SERIALIZED_BINDING_FIELD_SEMANTICS
            ),
            next(
                field_name
                for field_name in sorted(SERIALIZED_BINDING_FIELD_SEMANTICS)
                if field_name != semantic_rule["field"]
            ),
        )
        binding_graph_reject(
            f"wrong serialized binding field {rule_index}",
            lambda value, rule_index=rule_index, replacement_field=replacement_field: value[
                "binding_semantics"
            ]["serialized_rules"][rule_index].__setitem__("field", replacement_field),
        )
        semantic_rule_field_mutations += 1
        replacement_source = next(
            node_id
            for node_id in sorted(graph_nodes)
            if node_id != semantic_rule["target"]
            and node_id != semantic_rule["source"]
            and node_id != "*"
        )
        binding_graph_reject(
            f"wrong serialized binding source {rule_index}",
            lambda value, rule_index=rule_index, replacement_source=replacement_source: value[
                "binding_semantics"
            ]["serialized_rules"][rule_index].__setitem__(
                "source", replacement_source
            ),
        )
        semantic_rule_source_mutations += 1
        replacement_relation = next(
            relation
            for relation in sorted(GRAPH_EDGE_TYPES)
            if relation != semantic_rule["type"]
        )
        binding_graph_reject(
            f"wrong serialized binding relation {rule_index}",
            lambda value, rule_index=rule_index, replacement_relation=replacement_relation: value[
                "binding_semantics"
            ]["serialized_rules"][rule_index].__setitem__(
                "type", replacement_relation
            ),
        )
        semantic_rule_relation_mutations += 1

    if binding_contract != graph["serialized_binding_fields"] or not binding_projection:
        raise AssertionError("independent binding projection does not match canonical mirror")

    required_inputs = graph["required_inputs"]
    derived_inputs = derive_required_inputs(graph_nodes, graph_edges)
    declared_input_count = sum(len(dependencies) for dependencies in required_inputs.values())
    derived_input_count = sum(len(dependencies) for dependencies in derived_inputs.values())
    if declared_input_count != derived_input_count or any(
        set(required_inputs[target]) != set(derived_inputs[target]) for target in graph_nodes
    ):
        raise AssertionError("canonical required-input projection is not exact")
    required_input_deletions_tested = 0
    for target, dependencies in required_inputs.items():
        for dependency in dependencies:
            mutant = deepcopy(graph)
            mutant["required_inputs"][target].remove(dependency)
            required_input_deletions_tested += 1
            reject_value_error(
                f"required-input deletion {target}<-{dependency}",
                lambda mutant=mutant: validate_r12_authority_graph(mutant),
            )

    required_input_extra_tests = 0
    required_input_replacement_tests = 0
    required_input_duplicate_tests = 0
    required_input_whole_list_tests = 0
    required_input_empty_tests = 0
    for target, dependencies in required_inputs.items():
        candidate = next(
            node_id
            for node_id in sorted(graph_nodes)
            if node_id != target
            and node_id not in dependencies
            and any(
                edge["from"] == node_id or edge["to"] == node_id
                for edge in graph_edges
            )
        )
        mutant = deepcopy(graph)
        mutant["required_inputs"][target].append(candidate)
        required_input_extra_tests += 1
        reject_value_error(
            f"unauthorized required input {target}<-{candidate}",
            lambda mutant=mutant: validate_r12_authority_graph(mutant),
        )
        if dependencies:
            replacement = next(
                node_id
                for node_id in sorted(graph_nodes)
                if node_id != target and node_id not in dependencies
            )
            mutant = deepcopy(graph)
            mutant["required_inputs"][target][0] = replacement
            required_input_replacement_tests += 1
            reject_value_error(
                f"replaced required input {target}",
                lambda mutant=mutant: validate_r12_authority_graph(mutant),
            )
            mutant = deepcopy(graph)
            mutant["required_inputs"][target].append(dependencies[0])
            required_input_duplicate_tests += 1
            reject_value_error(
                f"duplicate required input {target}",
                lambda mutant=mutant: validate_r12_authority_graph(mutant),
            )
            mutant = deepcopy(graph)
            mutant["required_inputs"][target] = []
            required_input_empty_tests += 1
            reject_value_error(
                f"empty required-input list {target}",
                lambda mutant=mutant: validate_r12_authority_graph(mutant),
            )
            mutant = deepcopy(graph)
            mutant["required_inputs"].pop(target)
            required_input_whole_list_tests += 1
            reject_value_error(
                f"removed required-input list {target}",
                lambda mutant=mutant: validate_r12_authority_graph(mutant),
            )

    reordered = deepcopy(graph)
    for dependencies in reordered["required_inputs"].values():
        dependencies.reverse()
    validate_r12_authority_graph(reordered)

    import inspect

    if tuple(inspect.signature(derive_binding_projection).parameters) != (
        "graph", "nodes", "edges"
    ) or tuple(inspect.signature(derive_required_inputs).parameters) != (
        "nodes", "edges"
    ) or tuple(inspect.signature(derive_required_semantic_rules).parameters) != (
        "nodes", "edges", "object_field_contracts"
    ):
        raise AssertionError("binding/prerequisite derivation accepts mutable downstream structures")
    if "binding_semantics" in derive_required_semantic_rules.__code__.co_names:
        raise AssertionError("independent semantic-rule derivation reads binding_semantics")

    class MutableContractAccessGuard(dict[str, Any]):
        def __getitem__(self, key: str) -> Any:
            if key == "serialized_binding_fields":
                raise AssertionError("independent binding projection read mutable mirror")
            return super().__getitem__(key)

        def get(self, key: str, default: Any = None) -> Any:
            if key == "serialized_binding_fields":
                raise AssertionError("independent binding projection read mutable mirror")
            return super().get(key, default)

    guarded_graph = MutableContractAccessGuard(graph)
    derive_binding_projection(guarded_graph, graph_nodes, graph_edges)
    derive_serialized_binding_contract(guarded_graph, graph_nodes, graph_edges)

    semantic_rule_candidates = []
    for target, fields in sorted(graph["object_field_contracts"].items()):
        for edge_type in sorted(GRAPH_EDGE_TYPES):
            sources = sorted(
                {
                    edge["from"]
                    for edge in graph_edges
                    if edge["to"] == target and edge["type"] == edge_type
                }
            )
            sources.append("*")
            for source in sources:
                for field_name in fields:
                    semantics = SERIALIZED_BINDING_FIELD_SEMANTICS.get(field_name)
                    if semantics is None:
                        continue
                    category, value_source = semantics
                    semantic_rule_candidates.append(
                        {
                            "target": target,
                            "type": edge_type,
                            "source": source,
                            "field": field_name,
                            "category": category,
                            "value": value_source,
                            "cardinality": SERIALIZED_BINDING_CARDINALITIES[field_name],
                            "target_object_kind": graph_nodes[target]["authority_kind"],
                        }
                    )
    independent_rule_keys = {
        _semantic_rule_key(rule) for rule in independent_rules
    }
    unauthorized_semantic_rule_candidates = [
        candidate
        for candidate in semantic_rule_candidates
        if _semantic_rule_key(candidate) not in independent_rule_keys
    ]
    accepted_unauthorized_semantic_rules = 0
    for candidate in unauthorized_semantic_rule_candidates:
        mutant = deepcopy(graph)
        mutant["binding_semantics"]["serialized_rules"].append(candidate)
        try:
            validate_r12_authority_graph(mutant)
        except ValueError:
            continue
        accepted_unauthorized_semantic_rules += 1
    if accepted_unauthorized_semantic_rules:
        raise AssertionError(
            "exhaustive unauthorized semantic-rule probe accepted "
            f"{accepted_unauthorized_semantic_rules}"
        )

    semantic_rule_policy_mutations = 0
    accepted_semantic_rule_policy_mutations = 0
    derived_policies = independent_semantics["relation_policies"]
    for edge_type, policy in sorted(derived_policies.items()):
        for replacement_policy in sorted(SERIALIZED_BINDING_POLICIES - {policy}):
            mutant = deepcopy(graph)
            mutant["binding_semantics"]["relation_policies"][edge_type][
                "serialized_binding"
            ] = replacement_policy
            semantic_rule_policy_mutations += 1
            try:
                validate_r12_authority_graph(mutant)
            except ValueError:
                continue
            accepted_semantic_rule_policy_mutations += 1
    if accepted_semantic_rule_policy_mutations:
        raise AssertionError(
            "unauthorized semantic-rule policy mutation was accepted: "
            f"{accepted_semantic_rule_policy_mutations}"
        )

    authorized_node_edges = {
        (edge["from"], edge["type"], edge["to"]) for edge in graph_edges
    }
    candidate_edges = [
        (source, edge_type, target)
        for source in sorted(graph_nodes)
        for edge_type in sorted(GRAPH_EDGE_TYPES)
        for target in sorted(graph_nodes)
        if source != target
    ]
    unauthorized_candidates = [
        candidate
        for candidate in candidate_edges
        if candidate not in authorized_node_edges
    ]
    accepted_unauthorized_edges = 0
    for source, edge_type, target in unauthorized_candidates:
        mutant = deepcopy(graph)
        mutant["edges"].append(
            {"from": source, "type": edge_type, "to": target}
        )
        try:
            validate_r12_authority_graph(mutant)
        except ValueError:
            continue
        accepted_unauthorized_edges += 1
    if accepted_unauthorized_edges:
        raise AssertionError(
            f"exhaustive unauthorized-edge probe accepted {accepted_unauthorized_edges}"
        )

    authorized_edge_canonical_passes = 0
    authorized_edge_removals_rejected = 0
    authorized_edge_retypes_rejected = 0
    authorized_edge_redirects_rejected = 0
    for source, edge_type, target in sorted(authorized_node_edges):
        validate_r12_authority_graph(graph)
        authorized_edge_canonical_passes += 1
        removal = deepcopy(graph)
        removal["edges"].remove(
            next(
                edge for edge in removal["edges"]
                if edge["from"] == source
                and edge["type"] == edge_type
                and edge["to"] == target
            )
        )
        try:
            validate_r12_authority_graph(removal)
        except ValueError:
            authorized_edge_removals_rejected += 1
        for replacement in sorted(GRAPH_EDGE_TYPES - {edge_type}):
            retyped = deepcopy(graph)
            retyped["edges"].remove(
                next(
                    edge for edge in retyped["edges"]
                    if edge["from"] == source
                    and edge["type"] == edge_type
                    and edge["to"] == target
                )
            )
            retyped["edges"].append(
                {"from": source, "type": replacement, "to": target}
            )
            try:
                validate_r12_authority_graph(retyped)
            except ValueError:
                authorized_edge_retypes_rejected += 1
        for destination in sorted(graph_nodes):
            if destination == target:
                continue
            redirected = deepcopy(graph)
            redirected["edges"].remove(
                next(
                    edge for edge in redirected["edges"]
                    if edge["from"] == source
                    and edge["type"] == edge_type
                    and edge["to"] == target
                )
            )
            redirected["edges"].append(
                {"from": source, "type": edge_type, "to": destination}
            )
            try:
                validate_r12_authority_graph(redirected)
            except ValueError:
                authorized_edge_redirects_rejected += 1
    if authorized_edge_removals_rejected != len(authorized_node_edges):
        raise AssertionError("authorized-edge removal mutation was accepted")
    expected_retypes = len(authorized_node_edges) * (len(GRAPH_EDGE_TYPES) - 1)
    if authorized_edge_retypes_rejected != expected_retypes:
        raise AssertionError("authorized-edge retyping mutation was accepted")
    expected_redirects = len(authorized_node_edges) * (len(graph_nodes) - 1)
    if authorized_edge_redirects_rejected != expected_redirects:
        raise AssertionError("authorized-edge redirection mutation was accepted")

    synthetic = make_synthetic_context()
    graph_positive = validate_r12_authority_graph(graph)
    if any(not audit["passed"] for audit in graph_positive["audits"]):
        raise AssertionError(f"graph positive audit failed: {graph_positive['audits']}")
    positive = validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, synthetic)
    if positive != G3_EXPECTED_FIELDS:
        raise AssertionError(f"synthetic G3 authority positive result: {positive}")

    def synthetic_root_change_context() -> tuple[G3AuthorityContext, str, str]:
        context = deepcopy(synthetic)
        valid_kind_root_change(context.graph)
        root_bytes = canonical_json_bytes(context.graph)
        root_sha256 = sha256_bytes(root_bytes)
        input_payload = {
            key: deepcopy(value)
            for key, value in old_bundle_inputs.items()
            if key != "sha256"
        }
        input_payload["authority_graph_sha256"] = root_sha256
        input_payload["source_sha256s"]["authority_graph"] = root_sha256
        input_sha256 = sha256_bytes(canonical_json_bytes(input_payload))
        context.authority_graph_bytes = root_bytes
        context.authority_graph_sha256 = root_sha256
        context.objects["specification_bundle_inputs"][
            "authority_graph_sha256"
        ] = root_sha256
        context.objects["specification_bundle_inputs"]["sha256"] = input_sha256
        context.objects["specification_bundle_inputs"]["expected_sha256"] = input_sha256
        context.objects["specification_bundle_manifest"][
            "bundle_input_fingerprint_sha256"
        ] = input_sha256
        return context, root_sha256, input_sha256

    migrated_root_staleness_tests = 0
    aggregate_root_staleness_tests = 0
    migrated_stale_context, _, _ = synthetic_root_change_context()
    migrated_root_staleness_tests += 1
    reject_value_error(
        "migrated review stale after normative binding-root change",
        lambda: validate_g3_tag(
            G3_TAG_NAME, G3_FIXTURE_BODY, migrated_stale_context
        ),
    )

    aggregate_stale_context, _, changed_input_sha256 = synthetic_root_change_context()
    migrated = aggregate_stale_context.objects["migrated_finding_review"]
    migrated["target_bundle_inputs_sha256"] = changed_input_sha256
    migrated["review_input_fingerprint"] = _migrated_review_input_fingerprint(migrated)
    for row in migrated["review_records"]:
        row["reviewed_target"] = migrated["review_input_fingerprint"]
        aggregate_stale_context.review_artifacts[row["review_artifact_id"]][
            "reviewed_target"
        ] = migrated["review_input_fingerprint"]
        refresh_row_payload = dict(row)
        refresh_row_payload.pop("review_sha256")
        row["review_sha256"] = sha256_bytes(
            canonical_json_bytes(refresh_row_payload)
        )
    changed_manifest_sha256 = "a" * 64
    aggregate_stale_context.objects["specification_bundle_manifest"][
        "sha256"
    ] = changed_manifest_sha256
    aggregate_stale_context.objects["specification_bundle_manifest"][
        "expected_sha256"
    ] = changed_manifest_sha256
    aggregate_stale_context.bundle_manifest_sha256 = changed_manifest_sha256
    aggregate_body = G3_FIXTURE_BODY.replace(
        b"specification_bundle_manifest_sha256=" + b"0" * 64,
        b"specification_bundle_manifest_sha256="
        + changed_manifest_sha256.encode(),
        1,
    )
    aggregate_stale_context.tag["message"] = aggregate_body
    aggregate_root_staleness_tests += 1
    reject_value_error(
        "aggregate review stale after normative binding-root change",
        lambda: validate_g3_tag(
            G3_TAG_NAME, aggregate_body, aggregate_stale_context
        ),
    )

    def remove_synthetic_binding_field(
        context: G3AuthorityContext, rule: dict[str, str]
    ) -> None:
        actual = context.objects[rule["target"]].get(rule["field"])
        if rule["field"] == "target" and _is_independent_review_bundle(
            context.objects[rule["target"]]
        ):
            context.objects[rule["target"]].pop(rule["field"], None)
        elif isinstance(actual, dict):
            actual.pop(rule["source"], None)
        else:
            context.objects[rule["target"]].pop(rule["field"], None)

    coordinated_rule_mirror_builder_deletions_tested = 0
    coordinated_rule_mirror_builder_deletion_accepted = 0
    coordinated_four_layer_deletions_tested = 0
    coordinated_four_layer_deletion_accepted = 0
    for rule in selected_rules:
        mutant = deepcopy(synthetic)
        remove_declared_rule_and_mirror(mutant.graph, rule)
        remove_synthetic_binding_field(mutant, rule)
        coordinated_rule_mirror_builder_deletions_tested += 1
        try:
            validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant)
        except ValueError:
            pass
        else:
            coordinated_rule_mirror_builder_deletion_accepted += 1

        mutant = deepcopy(synthetic)
        remove_declared_rule_and_mirror(mutant.graph, rule)
        remove_synthetic_binding_field(mutant, rule)
        original_binding_validator = _validate_graph_object_bindings
        _validate_graph_object_bindings = lambda context: None
        coordinated_four_layer_deletions_tested += 1
        try:
            try:
                validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant)
            except ValueError:
                pass
            else:
                coordinated_four_layer_deletion_accepted += 1
        finally:
            _validate_graph_object_bindings = original_binding_validator
    if (
        coordinated_rule_mirror_builder_deletion_accepted
        or coordinated_four_layer_deletion_accepted
    ):
        raise AssertionError(
            "coordinated semantic-rule shrink mutation was accepted: "
            f"three_layer={coordinated_rule_mirror_builder_deletion_accepted} "
            f"four_layer={coordinated_four_layer_deletion_accepted}"
        )

    coordinated_schema_downstream_deletions_tested = 0
    coordinated_schema_downstream_deletion_accepted = 0
    for rule in selected_rules:
        mutant = deepcopy(synthetic)
        mutant.graph["object_field_contracts"][rule["target"]].remove(rule["field"])
        remove_declared_rule_and_mirror(mutant.graph, rule)
        remove_synthetic_binding_field(mutant, rule)
        coordinated_schema_downstream_deletions_tested += 1
        try:
            validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant)
        except ValueError:
            pass
        else:
            coordinated_schema_downstream_deletion_accepted += 1
    if coordinated_schema_downstream_deletion_accepted:
        raise AssertionError(
            "coordinated schema/node/downstream shrink mutation was accepted: "
            f"{coordinated_schema_downstream_deletion_accepted}"
        )

    explicit_g3_bypass_cases = {
        "architecture review target": (
            "architecture_review", "target"
        ),
        "architecture approval review_sha256": (
            "architecture_approval", "review_sha256"
        ),
        "F0 review target": ("f0_review", "target"),
        "F0 approval review_sha256": ("f0_approval", "review_sha256"),
    }
    explicit_g3_bypass_tests = 0
    explicit_g3_bypass_accepted = 0
    for label, (target, field_name) in explicit_g3_bypass_cases.items():
        rule = next(
            rule
            for rule in selected_rules
            if rule["target"] == target and rule["field"] == field_name
        )
        mutant = deepcopy(synthetic)
        remove_declared_rule_and_mirror(mutant.graph, rule)
        remove_synthetic_binding_field(mutant, rule)
        explicit_g3_bypass_tests += 1
        try:
            validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant)
        except ValueError:
            pass
        else:
            explicit_g3_bypass_accepted += 1
            raise AssertionError(f"explicit G3 bypass accepted: {label}")

    serialized_binding_deletions_tested = 0
    serialized_binding_extra_tests = 0
    for descriptor in binding_projection:
        mutant = deepcopy(synthetic)
        target = descriptor["target"]
        field_name = descriptor["field"]
        source = descriptor["source"]
        actual = mutant.objects[target].get(field_name)
        if field_name == "target" and _is_independent_review_bundle(mutant.objects[target]):
            if actual != _review_bundle_target(mutant, target):
                raise AssertionError(f"synthetic binding builder omitted {target}/{source}")
            mutant.objects[target].pop(field_name, None)
        elif isinstance(actual, dict):
            if source not in actual:
                raise AssertionError(f"synthetic binding builder omitted {target}/{source}")
            actual.pop(source)
        else:
            mutant.objects[target].pop(field_name, None)
        serialized_binding_deletions_tested += 1
        reject_value_error(
            f"serialized binding deletion {target}/{descriptor['relation']}/{source}",
            lambda mutant=mutant: validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant),
        )

    for target, relations in binding_contract.items():
        for edge_type, sources in relations.items():
            field_name = next(iter(sources.values()))
            actual = synthetic.objects[target].get(field_name)
            if not isinstance(actual, dict):
                continue
            expected_sources = {
                edge["from"]
                for edge in graph_edges
                if edge["to"] == target
                and edge["type"] == edge_type
            }
            extra_source = next(
                node_id
                for node_id in sorted(graph_nodes)
                if node_id not in expected_sources and node_id != target
            )
            mutant = deepcopy(synthetic)
            value = (
                _authority_descriptor(mutant.objects[extra_source])
                if field_name == "authority_bindings"
                else mutant.objects[extra_source]["sha256"]
            )
            mutant.objects[target][field_name][extra_source] = value
            serialized_binding_extra_tests += 1
            reject_value_error(
                f"extra serialized binding {target}/{edge_type}/{extra_source}",
                lambda mutant=mutant: validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant),
            )

    def g3_reject(label: str, mutate: object) -> None:
        mutant = deepcopy(synthetic)
        mutate(mutant)
        reject_value_error(
            label, lambda: validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant)
        )

    def refresh_review_row_hash(row: dict[str, Any]) -> None:
        row_payload = dict(row)
        row_payload.pop("review_sha256")
        row["review_sha256"] = sha256_bytes(canonical_json_bytes(row_payload))

    decision_combinations_tested = 0
    invalid_decision_combinations_accepted = 0
    base_rows = synthetic.objects["migrated_finding_review"]["review_records"]
    for mask in range(1 << len(base_rows)):
        mutant = deepcopy(synthetic)
        rows = mutant.objects["migrated_finding_review"]["review_records"]
        for index, row in enumerate(rows):
            row["decision"] = "NO-GO" if mask & (1 << index) else "GO"
            refresh_review_row_hash(row)
            mutant.review_artifacts[row["review_artifact_id"]]["decision"] = row[
                "decision"
            ]
        mutant.objects["migrated_finding_review"]["decision"] = (
            "GO" if mask == 0 else "NO-GO"
        )
        decision_combinations_tested += 1
        try:
            validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant)
        except ValueError:
            if mask == 0:
                raise AssertionError("all-GO migrated review vector was rejected")
        else:
            if mask != 0:
                invalid_decision_combinations_accepted += 1
    if decision_combinations_tested != 32 or invalid_decision_combinations_accepted:
        raise AssertionError(
            f"migrated decision enumeration: {decision_combinations_tested}/32, "
            f"accepted invalid={invalid_decision_combinations_accepted}"
        )

    missing_role_cases_tested = 0
    for index in range(len(base_rows)):
        mutant = deepcopy(synthetic)
        mutant.objects["migrated_finding_review"]["review_records"].pop(index)
        missing_role_cases_tested += 1
        reject_value_error(
            f"missing migrated review role {index}",
            lambda mutant=mutant: validate_g3_tag(
                G3_TAG_NAME, G3_FIXTURE_BODY, mutant
            ),
        )

    duplicate_artifact_cases_tested = 0
    duplicate_reviewer_cases_tested = 0
    for left in range(len(base_rows)):
        for right in range(left + 1, len(base_rows)):
            artifact_mutant = deepcopy(synthetic)
            artifact_rows = artifact_mutant.objects["migrated_finding_review"][
                "review_records"
            ]
            artifact_rows[right]["review_artifact_id"] = artifact_rows[left][
                "review_artifact_id"
            ]
            refresh_review_row_hash(artifact_rows[right])
            duplicate_artifact_cases_tested += 1
            reject_value_error(
                f"duplicate review artifact pair {left},{right}",
                lambda mutant=artifact_mutant: validate_g3_tag(
                    G3_TAG_NAME, G3_FIXTURE_BODY, mutant
                ),
            )

            reviewer_mutant = deepcopy(synthetic)
            reviewer_rows = reviewer_mutant.objects["migrated_finding_review"][
                "review_records"
            ]
            reviewer_rows[right]["reviewer_authority_id"] = reviewer_rows[left][
                "reviewer_authority_id"
            ]
            reviewer_rows[right]["independence_relation"] = {
                "type": "distinct_reviewer_authority",
                "reviewer_authority_id": reviewer_rows[left][
                    "reviewer_authority_id"
                ],
            }
            refresh_review_row_hash(reviewer_rows[right])
            duplicate_reviewer_cases_tested += 1
            reject_value_error(
                f"duplicate reviewer pair {left},{right}",
                lambda mutant=reviewer_mutant: validate_g3_tag(
                    G3_TAG_NAME, G3_FIXTURE_BODY, mutant
                ),
            )
    if duplicate_artifact_cases_tested != 10 or duplicate_reviewer_cases_tested != 10:
        raise AssertionError("five-role pairwise enumeration is incomplete")

    unresolved_reviewer_cases_tested = 0
    unresolved_artifact_cases_tested = 0
    role_mismatch_cases_tested = 0
    for index, row_template in enumerate(base_rows):
        unresolved_reviewer = deepcopy(synthetic)
        unresolved_rows = unresolved_reviewer.objects["migrated_finding_review"][
            "review_records"
        ]
        unresolved_rows[index]["reviewer_authority_id"] = "e" * 64
        unresolved_rows[index]["independence_relation"] = {
            "type": "distinct_reviewer_authority",
            "reviewer_authority_id": "e" * 64,
        }
        refresh_review_row_hash(unresolved_rows[index])
        unresolved_reviewer_cases_tested += 1
        reject_value_error(
            f"unresolved reviewer identity {index}",
            lambda mutant=unresolved_reviewer: validate_g3_tag(
                G3_TAG_NAME, G3_FIXTURE_BODY, mutant
            ),
        )

        unresolved_artifact = deepcopy(synthetic)
        unresolved_artifact_rows = unresolved_artifact.objects[
            "migrated_finding_review"
        ]["review_records"]
        unresolved_artifact_rows[index]["review_artifact_id"] = "f" * 64
        refresh_review_row_hash(unresolved_artifact_rows[index])
        unresolved_artifact_cases_tested += 1
        reject_value_error(
            f"unresolved review artifact identity {index}",
            lambda mutant=unresolved_artifact: validate_g3_tag(
                G3_TAG_NAME, G3_FIXTURE_BODY, mutant
            ),
        )

        role_mismatch = deepcopy(synthetic)
        role_rows = role_mismatch.objects["migrated_finding_review"]["review_records"]
        artifact = role_mismatch.review_artifacts[role_rows[index]["review_artifact_id"]]
        artifact["role"] = sorted(REVIEW_ROLES - {row_template["role"]})[0]
        role_mismatch_cases_tested += 1
        reject_value_error(
            f"review artifact role mismatch {index}",
            lambda mutant=role_mismatch: validate_g3_tag(
                G3_TAG_NAME, G3_FIXTURE_BODY, mutant
            ),
        )
    if (
        unresolved_reviewer_cases_tested != 5
        or unresolved_artifact_cases_tested != 5
        or role_mismatch_cases_tested != 5
    ):
        raise AssertionError("five-role identity enumeration is incomplete")

    review_bundle_matrix_negative_tests = 0
    direct_target_matrix_tests = 0
    direct_target_matrix_negative_tests = 0
    direct_actor_pair_tests = 0
    available_direct_review_entrypoint_negative_tests = 0

    def direct_bundle_reject(label: str, mutate: object) -> None:
        nonlocal review_bundle_matrix_negative_tests
        mutant = deepcopy(synthetic)
        bundle = mutant.objects["architecture_review"]["canonical_object"]
        mutate(mutant, bundle)
        if "reviews" in bundle and "target" in bundle:
            bundle["review_bundle_id"] = independent_review_bundle_id(bundle)
        review_bundle_matrix_negative_tests += 1
        reject_value_error(
            label,
            lambda: validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant),
        )

    def direct_artifact(mutant: G3AuthorityContext, row: dict[str, Any]) -> dict[str, Any]:
        artifact_id = row["review_artifact_reference"]["immutable_uri"].removeprefix(
            REVIEW_ARTIFACT_URI_PREFIX
        )
        return mutant.review_artifacts[artifact_id]

    direct_review_nodes = sorted(
        node_id for node_id in REVIEW_BUNDLE_NODES if node_id in synthetic.objects
    )
    for node_id in direct_review_nodes:
        bundle = synthetic.objects[node_id]["canonical_object"]
        expected_target = _review_bundle_target(synthetic, node_id)
        if bundle["target"] != expected_target:
            raise AssertionError(
                f"direct review target matrix mismatch for {node_id}: "
                f"{bundle['target']} != {expected_target}"
            )
        _validate_independent_review_bundle(synthetic, node_id, synthetic.objects[node_id])
        direct_target_matrix_tests += 1

        opposite_target = (
            {"type": "git_commit", "git_sha": "e" * 40}
            if expected_target["type"] == "external_object"
            else {
                "type": "external_object",
                "object_kind": "decision_bundle",
                "object_sha256": "e" * 64,
            }
        )
        mutant = deepcopy(synthetic)
        mutant_bundle = mutant.objects[node_id]["canonical_object"]
        mutant_bundle["target"] = opposite_target
        mutant_bundle["review_bundle_id"] = independent_review_bundle_id(mutant_bundle)
        direct_target_matrix_negative_tests += 1
        reject_value_error(
            f"direct review opposite target type {node_id}",
            lambda mutant=mutant, node_id=node_id: _validate_independent_review_bundle(
                mutant, node_id, mutant.objects[node_id]
            ),
        )

        malformed_target = deepcopy(expected_target)
        digest_field = "git_sha" if expected_target["type"] == "git_commit" else "object_sha256"
        malformed_target[digest_field] = "A" * len(malformed_target[digest_field])
        mutant = deepcopy(synthetic)
        mutant_bundle = mutant.objects[node_id]["canonical_object"]
        mutant_bundle["target"] = malformed_target
        mutant_bundle["review_bundle_id"] = independent_review_bundle_id(mutant_bundle)
        direct_target_matrix_negative_tests += 1
        reject_value_error(
            f"direct review malformed target digest {node_id}",
            lambda mutant=mutant, node_id=node_id: _validate_independent_review_bundle(
                mutant, node_id, mutant.objects[node_id]
            ),
        )

        wrong_identity_target = deepcopy(expected_target)
        wrong_identity_target[digest_field] = "e" * len(wrong_identity_target[digest_field])
        mutant = deepcopy(synthetic)
        mutant_bundle = mutant.objects[node_id]["canonical_object"]
        mutant_bundle["target"] = wrong_identity_target
        mutant_bundle["review_bundle_id"] = independent_review_bundle_id(mutant_bundle)
        direct_target_matrix_negative_tests += 1
        reject_value_error(
            f"direct review wrong target identity {node_id}",
            lambda mutant=mutant, node_id=node_id: _validate_independent_review_bundle(
                mutant, node_id, mutant.objects[node_id]
            ),
        )

        for left, right in ((left, right) for left in range(5) for right in range(left + 1, 5)):
            mutant = deepcopy(synthetic)
            mutant_rows = mutant.objects[node_id]["canonical_object"]["reviews"]
            left_reviewer_id = direct_artifact(mutant, mutant_rows[left])["reviewer_authority_id"]
            right_reviewer_id = direct_artifact(mutant, mutant_rows[right])["reviewer_authority_id"]
            mutant.reviewer_authorities[right_reviewer_id]["actor_identity_digest"] = mutant.reviewer_authorities[left_reviewer_id]["actor_identity_digest"]
            direct_actor_pair_tests += 1
            reject_value_error(
                f"direct review duplicate actor pair {node_id} {left},{right}",
                lambda mutant=mutant, node_id=node_id: _validate_independent_review_bundle(
                mutant, node_id, mutant.objects[node_id]
            ),
        )

    readiness_entrypoint_target_mutant = deepcopy(synthetic)
    readiness_entrypoint_bundle = readiness_entrypoint_target_mutant.objects[
        "readiness_review"
    ]["canonical_object"]
    readiness_entrypoint_target = dict(
        _review_bundle_target(readiness_entrypoint_target_mutant, "readiness_review")
    )
    readiness_entrypoint_digest_field = (
        "git_sha"
        if readiness_entrypoint_target["type"] == "git_commit"
        else "object_sha256"
    )
    readiness_entrypoint_target[readiness_entrypoint_digest_field] = "e" * len(
        readiness_entrypoint_target[readiness_entrypoint_digest_field]
    )
    readiness_entrypoint_bundle["target"] = readiness_entrypoint_target
    readiness_entrypoint_bundle["review_bundle_id"] = independent_review_bundle_id(
        readiness_entrypoint_bundle
    )
    available_direct_review_entrypoint_negative_tests += 1
    reject_value_error(
        "G3 entrypoint readiness target mutation",
        lambda: validate_g3_tag(
            G3_TAG_NAME, G3_FIXTURE_BODY, readiness_entrypoint_target_mutant
        ),
    )

    readiness_entrypoint_actor_mutant = deepcopy(synthetic)
    readiness_entrypoint_rows = readiness_entrypoint_actor_mutant.objects[
        "readiness_review"
    ]["canonical_object"]["reviews"]
    left_reviewer_id = direct_artifact(
        readiness_entrypoint_actor_mutant, readiness_entrypoint_rows[0]
    )["reviewer_authority_id"]
    right_reviewer_id = direct_artifact(
        readiness_entrypoint_actor_mutant, readiness_entrypoint_rows[1]
    )["reviewer_authority_id"]
    readiness_entrypoint_actor_mutant.reviewer_authorities[right_reviewer_id][
        "actor_identity_digest"
    ] = readiness_entrypoint_actor_mutant.reviewer_authorities[left_reviewer_id][
        "actor_identity_digest"
    ]
    available_direct_review_entrypoint_negative_tests += 1
    reject_value_error(
        "G3 entrypoint readiness actor mutation",
        lambda: validate_g3_tag(
            G3_TAG_NAME, G3_FIXTURE_BODY, readiness_entrypoint_actor_mutant
        ),
    )

    direct_bundle_reject(
        "architecture review missing role",
        lambda _context, bundle: bundle["reviews"].pop(0),
    )
    direct_bundle_reject(
        "architecture review duplicate role",
        lambda _context, bundle: bundle["reviews"][1].update(
            {"role": bundle["reviews"][0]["role"]}
        ),
    )
    direct_bundle_reject(
        "architecture review wrong reviewer",
        lambda context, bundle: direct_artifact(context, bundle["reviews"][0]).update(
            {
                "reviewer_authority_id": direct_artifact(
                    context, bundle["reviews"][1]
                )["reviewer_authority_id"]
            }
        ),
    )
    direct_bundle_reject(
        "architecture review unresolved reviewer",
        lambda context, bundle: direct_artifact(context, bundle["reviews"][0]).update(
            {
                "reviewer_authority_id": "e" * 64,
                "independence_relation": {
                    "type": "distinct_reviewer_authority",
                    "reviewer_authority_id": "e" * 64,
                },
            }
        ),
    )
    direct_bundle_reject(
        "architecture review duplicate reviewer",
        lambda context, bundle: direct_artifact(context, bundle["reviews"][1]).update(
            {
                "reviewer_authority_id": direct_artifact(
                    context, bundle["reviews"][0]
                )["reviewer_authority_id"]
            }
        ),
    )
    direct_bundle_reject(
        "architecture review missing artifact",
        lambda context, bundle: context.review_artifacts.pop(
            bundle["reviews"][0]["review_artifact_reference"]["immutable_uri"].removeprefix(
                REVIEW_ARTIFACT_URI_PREFIX
            )
        ),
    )
    direct_bundle_reject(
        "architecture review duplicate artifact",
        lambda _context, bundle: bundle["reviews"][1].update(
            {"review_artifact_reference": bundle["reviews"][0]["review_artifact_reference"]}
        ),
    )
    direct_bundle_reject(
        "architecture review wrong artifact target",
        lambda context, bundle: direct_artifact(context, bundle["reviews"][0]).update(
            {"reviewed_target": "e" * 64}
        ),
    )
    direct_bundle_reject(
        "architecture review wrong artifact reviewer",
        lambda context, bundle: direct_artifact(context, bundle["reviews"][0]).update(
            {
                "reviewer_authority_id": direct_artifact(
                    context, bundle["reviews"][1]
                )["reviewer_authority_id"]
            }
        ),
    )
    direct_bundle_reject(
        "architecture review wrong artifact role",
        lambda context, bundle: direct_artifact(context, bundle["reviews"][0]).update(
            {"role": bundle["reviews"][1]["role"]}
        ),
    )
    direct_bundle_reject(
        "architecture review wrong artifact decision",
        lambda context, bundle: direct_artifact(context, bundle["reviews"][0]).update(
            {"decision": "NO-GO"}
        ),
    )
    direct_bundle_reject(
        "architecture review wrong artifact digest",
        lambda _context, bundle: bundle["reviews"][0][
            "review_artifact_reference"
        ].update({"sha256": "e" * 64}),
    )
    direct_bundle_reject(
        "architecture review stale artifact",
        lambda context, bundle: direct_artifact(context, bundle["reviews"][0]).update(
            {"stale": True}
        ),
    )
    direct_bundle_reject(
        "architecture review wrong target",
        lambda _context, bundle: bundle["target"].update({"git_sha": "e" * 40}),
    )
    direct_bundle_reject(
        "architecture review aggregate count mismatch",
        lambda _context, bundle: bundle.update({"aggregate_p0_count": "1"}),
    )
    direct_bundle_reject(
        "architecture review unknown field",
        lambda _context, bundle: bundle.update({"unknown": True}),
    )
    direct_bundle_reject(
        "architecture review missing field",
        lambda _context, bundle: bundle.pop("aggregate_decision"),
    )
    real_test_only = deepcopy(synthetic)
    real_test_only.mode = "real"
    reject_value_error(
        "synthetic review bundle in real mode",
        lambda: _validate_independent_review_bundle(
            real_test_only,
            "architecture_review",
            real_test_only.objects["architecture_review"],
        ),
    )

    g3_reject(
        "migrated review serialized decision contradicts derived GO",
        lambda context: context.objects["migrated_finding_review"].update(
            {"decision": "NO-GO"}
        ),
    )

    g3_reject("missing architecture approval", lambda context: context.objects.pop("architecture_approval"))
    g3_reject("stale architecture approval", lambda context: context.objects["architecture_approval"].update({"stale": True}))
    g3_reject("missing F0 approval", lambda context: context.objects.pop("f0_approval"))
    g3_reject("wrong F0 target", lambda context: context.objects["f0_approval"].update({"target_sha256": "c" * 64}))
    g3_reject("missing component review", lambda context: context.objects.pop("component_wire_review"))
    g3_reject("stale component review", lambda context: context.objects["component_wire_review"].update({"stale": True}))
    g3_reject("missing migrated review", lambda context: context.objects.pop("migrated_finding_review"))
    g3_reject("migrated review wrong bundle", lambda context: context.objects["migrated_finding_review"].update({"target_bundle_inputs_sha256": "f" * 64}))
    g3_reject("migrated review wrong ledger", lambda context: context.objects["migrated_finding_review"].update({"reviewed_migration_ledger_sha256": "f" * 64}))
    g3_reject("migrated review wrong commit", lambda context: context.objects["migrated_finding_review"].update({"target_git_commit": "wrong-target"}))
    g3_reject("migrated review hash mismatch", lambda context: context.objects["migrated_finding_review"].update({"expected_sha256": "e" * 64}))
    g3_reject(
        "migrated review incomplete disposition",
        lambda context: context.objects["migrated_finding_review"]["finding_dispositions"].pop("F-PLAN-R11-P1-01"),
    )
    g3_reject(
        "migrated review open disposition",
        lambda context: context.objects["migrated_finding_review"][
            "finding_dispositions"
        ].__setitem__("F-PLAN-R11-P1-01", "OPEN"),
    )
    g3_reject(
        "migrated review pending P1 disposition",
        lambda context: context.objects["migrated_finding_review"][
            "finding_dispositions"
        ].__setitem__("F-PLAN-R11-P1-01", "PENDING"),
    )
    g3_reject(
        "migrated review partially closed P1 disposition",
        lambda context: context.objects["migrated_finding_review"][
            "finding_dispositions"
        ].__setitem__("F-PLAN-R11-P1-01", "PARTIALLY_CLOSED"),
    )
    g3_reject(
        "migrated review count mismatch",
        lambda context: context.objects["migrated_finding_review"].update(
            {"p1_count": 1}
        ),
    )
    g3_reject(
        "migrated review unknown disposition",
        lambda context: context.objects["migrated_finding_review"][
            "finding_dispositions"
        ].__setitem__("F-PLAN-R11-P1-01", "CLOSED"),
    )
    g3_reject(
        "migrated review missing finding",
        lambda context: context.objects["migrated_finding_review"][
            "finding_dispositions"
        ].pop("F-PLAN-R11-P1-01"),
    )
    g3_reject(
        "migrated review duplicate finding",
        lambda context: context.objects["migrated_finding_review"][
            "reviewed_finding_ids"
        ].append("F-PLAN-R11-P1-01"),
    )
    g3_reject(
        "migrated review unknown finding",
        lambda context: context.objects["migrated_finding_review"][
            "finding_dispositions"
        ].__setitem__("F-PLAN-UNKNOWN", "TECHNICALLY_CLOSED"),
    )
    g3_reject(
        "migrated review decision mismatch",
        lambda context: context.objects["migrated_finding_review"].update(
            {"decision": "NO-GO"}
        ),
    )
    g3_reject(
        "migrated review normative input mismatch",
        lambda context: context.objects["migrated_finding_review"].update(
            {"reviewed_normative_traceability_matrix_sha256": "f" * 64}
        ),
    )
    g3_reject(
        "migrated review records missing",
        lambda context: context.objects["migrated_finding_review"].update(
            {"review_records": []}
        ),
    )
    g3_reject(
        "migrated review records duplicate role",
        lambda context: context.objects["migrated_finding_review"][
            "review_records"
        ][1].update(
            {
                "role": context.objects["migrated_finding_review"][
                    "review_records"
                ][0]["role"]
            }
        ),
    )
    g3_reject(
        "migrated review missing role",
        lambda context: context.objects["migrated_finding_review"][
            "review_records"
        ].pop(),
    )
    g3_reject(
        "migrated review independence mismatch",
        lambda context: context.objects["migrated_finding_review"][
            "review_records"
        ][0].update({"independence_relation": {"type": "same_reviewer"}}),
    )
    g3_reject(
        "migrated review record target mismatch",
        lambda context: context.objects["migrated_finding_review"][
            "review_records"
        ][0].update({"reviewed_target": "e" * 64}),
    )
    g3_reject(
        "migrated review record hash mismatch",
        lambda context: context.objects["migrated_finding_review"][
            "review_records"
        ][0].update({"review_sha256": "e" * 64}),
    )
    g3_reject(
        "migrated review fingerprint mismatch",
        lambda context: context.objects["migrated_finding_review"].update(
            {"review_input_fingerprint": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review architecture approval ID stale",
        lambda context: context.objects["architecture_approval"].update(
            {"authority_id": "synthetic:replacement-architecture-approval"}
        ),
    )
    g3_reject(
        "migrated review architecture approval digest stale",
        lambda context: context.objects["architecture_approval"].update(
            {"sha256": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review architecture approval target stale",
        lambda context: context.objects["architecture_approval"].update(
            {"target_sha256": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review F0 approval ID stale",
        lambda context: context.objects["f0_approval"].update(
            {"authority_id": "synthetic:replacement-f0-approval"}
        ),
    )
    g3_reject(
        "migrated review F0 approval digest stale",
        lambda context: context.objects["f0_approval"].update(
            {"sha256": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review F0 approval target stale",
        lambda context: context.objects["f0_approval"].update(
            {"target_sha256": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review component specification stale",
        lambda context: context.objects["component_wire_spec"].update(
            {"sha256": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review component review stale",
        lambda context: context.objects["component_wire_review"].update(
            {"sha256": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review ledger stale",
        lambda context: context.objects["migration_ledger"].update(
            {"sha256": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review normative matrix stale",
        lambda context: context.objects["normative_traceability_matrix"].update(
            {"sha256": "e" * 64}
        ),
    )
    g3_reject(
        "migrated review traceability manifest stale",
        lambda context: context.objects["generated_traceability_manifest"].update(
            {"sha256": "e" * 64}
        ),
    )
    g3_reject("migrated review stale", lambda context: context.objects["migrated_finding_review"].update({"stale": True}))
    g3_reject("migrated review superseded", lambda context: context.objects["migrated_finding_review"].update({"superseded_by": "new-review"}))
    g3_reject("migrated review non-independent", lambda context: context.objects["migrated_finding_review"].update({"producer": "remediation_agent"}))
    g3_reject("missing aggregate review", lambda context: context.objects.pop("aggregate_review"))
    g3_reject(
        "aggregate review wrong bundle",
        lambda context: context.objects["aggregate_review"].update(
            {"target": {"type": "git_commit", "git_sha": "e" * 40}}
        ),
    )
    g3_reject("aggregate review hash mismatch", lambda context: context.objects["aggregate_review"].update({"sha256": "e" * 64}))
    g3_reject("manifest hash mismatch", lambda context: context.objects["specification_bundle_manifest"].update({"sha256": "e" * 64}))
    g3_reject("manifest changed", lambda context: context.objects["specification_bundle_manifest"].update({"content_unchanged": False}))
    g3_reject("wrong G3 target commit", lambda context: context.tag.update({"peeled_commit": "wrong-target"}))
    g3_reject("lightweight G3 tag", lambda context: context.tag.update({"annotated": False, "object_type": "commit"}))
    review_start_git_positive_tests = 0
    review_start_git_negative_tests = 0
    review_start_temporary, review_start_repository, review_start_target, _ = _isolated_real_fixture(
        populate_authority=False
    )
    try:
        _fixture_git(
            review_start_repository,
            ["update-ref", "refs/heads/main", review_start_target],
        )
        _fixture_git(
            review_start_repository,
            ["update-ref", "refs/remotes/origin/main", review_start_target],
        )
        anchors = validate_review_start_git_state(
            review_start_repository, review_start_target, review_start_target
        )
        if any(value != review_start_target for value in anchors.values()):
            raise AssertionError(f"review-start anchors were not equal: {anchors}")
        review_start_git_positive_tests += 1
        branch_ref = _git_output(
            review_start_repository, ["symbolic-ref", "--short", "HEAD"]
        ).decode().strip()
        _fixture_git(
            review_start_repository,
            ["commit", "--allow-empty", "-qm", "review-start mismatch fixture"],
        )
        mismatch_sha = _git_output(
            review_start_repository, ["rev-parse", "HEAD"]
        ).decode().strip()
        _fixture_git(
            review_start_repository,
            ["update-ref", f"refs/heads/{branch_ref}", review_start_target],
        )
        for label, ref in (
            ("HEAD", f"refs/heads/{branch_ref}"),
            ("local main", "refs/heads/main"),
            ("origin main", "refs/remotes/origin/main"),
        ):
            original = _git_output(
                review_start_repository, ["rev-parse", f"{ref}^{{commit}}"]
            ).decode().strip()
            _fixture_git(
                review_start_repository,
                ["update-ref", ref, mismatch_sha],
            )
            try:
                reject_value_error(
                    f"review-start {label} mismatch",
                    lambda: validate_review_start_git_state(
                        review_start_repository, review_start_target, review_start_target
                    ),
                )
            finally:
                _fixture_git(review_start_repository, ["update-ref", ref, original])
            review_start_git_negative_tests += 1
        reject_value_error(
            "review-start live main mismatch",
            lambda: validate_review_start_git_state(
                review_start_repository, review_start_target, mismatch_sha
            ),
        )
        review_start_git_negative_tests += 1
    finally:
        review_start_temporary.cleanup()
    safe_publication_positive_tests = 0
    safe_publication_negative_tests = 0
    safe_publication_race_tests = 0
    publication_temporary, publication_repository, publication_old_sha, _ = _isolated_real_fixture(
        populate_authority=False
    )
    publication_sandbox = tempfile.TemporaryDirectory(
        prefix="phase-f-publication-", dir=Path(publication_temporary.name).parent
    )
    try:
        publication_root = Path(publication_sandbox.name)
        publication_remote = publication_root / "remote.git"
        _fixture_git(
            publication_repository,
            ["init", "--bare", "-q", str(publication_remote)],
        )
        _fixture_git(publication_repository, ["branch", "-M", "main"])
        _fixture_git(
            publication_repository,
            ["remote", "add", "origin", str(publication_remote)],
        )
        _fixture_git(
            publication_repository, ["push", "-q", "-u", "origin", "main"]
        )
        _fixture_git(
            publication_repository,
            ["--git-dir", str(publication_remote), "symbolic-ref", "HEAD", "refs/heads/main"],
        )
        candidate_sha = _fixture_git(
            publication_repository,
            ["commit", "--allow-empty", "-qm", "safe publication candidate"],
        )
        candidate_sha = _fixture_git(
            publication_repository, ["rev-parse", "HEAD"]
        ).decode().strip()
        live_before = read_live_remote_main_sha(publication_repository)
        if live_before != publication_old_sha:
            raise AssertionError("publication fixture remote did not start at expected old SHA")
        preflight = validate_safe_publication_preflight(
            publication_repository, candidate_sha, publication_old_sha, live_before
        )
        if preflight["HEAD"] != candidate_sha or preflight["origin_main"] != publication_old_sha:
            raise AssertionError(f"publication preflight anchors were wrong: {preflight}")
        published = publish_reviewed_sha_with_lease(
            publication_repository, candidate_sha, publication_old_sha
        )
        if any(value != candidate_sha for value in published.values()):
            raise AssertionError(f"publication postcondition anchors were wrong: {published}")
        safe_publication_positive_tests += 1

        dirty_marker = publication_repository / "publication-dirty-marker"
        dirty_marker.write_text("dirty\n")
        safe_publication_negative_tests += 1
        reject_value_error(
            "safe publication dirty worktree",
            lambda: validate_safe_publication_preflight(
                publication_repository, candidate_sha, candidate_sha, candidate_sha
            ),
        )
        dirty_marker.unlink()

        next_candidate_sha = _fixture_git(
            publication_repository,
            ["commit", "--allow-empty", "-qm", "safe publication race candidate"],
        )
        next_candidate_sha = _fixture_git(
            publication_repository, ["rev-parse", "HEAD"]
        ).decode().strip()
        peer_repository = publication_root / "peer"
        _fixture_git(
            publication_repository,
            ["clone", "-q", str(publication_remote), str(peer_repository)],
        )
        _fixture_git(peer_repository, ["config", "user.name", "Phase F Peer Test"])
        _fixture_git(
            peer_repository,
            ["config", "user.email", "phase-f-peer@example.invalid"],
        )
        _fixture_git(
            peer_repository,
            ["commit", "--allow-empty", "-qm", "remote race"],
        )
        race_sha = _fixture_git(peer_repository, ["rev-parse", "HEAD"]).decode().strip()
        _fixture_git(peer_repository, ["push", "-q", "origin", "main"])
        safe_publication_negative_tests += 1
        safe_publication_race_tests += 1
        reject_value_error(
            "safe publication remote race",
            lambda: publish_reviewed_sha_with_lease(
                publication_repository, next_candidate_sha, candidate_sha
            ),
        )
        if read_live_remote_main_sha(publication_repository) != race_sha:
            raise AssertionError("remote race fixture was unexpectedly overwritten")

        non_ff_remote = publication_root / "non-fast-forward.git"
        _fixture_git(
            publication_repository,
            ["init", "--bare", "-q", str(non_ff_remote)],
        )
        _fixture_git(
            publication_repository,
            ["push", "-q", str(non_ff_remote), f"{publication_old_sha}:refs/heads/main"],
        )
        _fixture_git(
            publication_repository,
            ["--git-dir", str(non_ff_remote), "symbolic-ref", "HEAD", "refs/heads/main"],
        )
        non_ff_repository = publication_root / "non-ff"
        _fixture_git(
            publication_repository,
            ["clone", "-q", str(non_ff_remote), str(non_ff_repository)],
        )
        _fixture_git(non_ff_repository, ["config", "user.name", "Phase F Non-FF Test"])
        _fixture_git(
            non_ff_repository,
            ["config", "user.email", "phase-f-non-ff@example.invalid"],
        )
        _fixture_git(non_ff_repository, ["switch", "--orphan", "divergent"])
        _fixture_git(
            non_ff_repository,
            ["commit", "--allow-empty", "-qm", "divergent candidate"],
        )
        divergent_sha = _fixture_git(
            non_ff_repository, ["rev-parse", "HEAD"]
        ).decode().strip()
        _fixture_git(non_ff_repository, ["update-ref", "refs/heads/main", "HEAD"])
        _fixture_git(non_ff_repository, ["symbolic-ref", "HEAD", "refs/heads/main"])
        _fixture_git(non_ff_repository, ["branch", "-D", "divergent"])
        safe_publication_negative_tests += 1
        reject_value_error(
            "safe publication non-fast-forward candidate",
            lambda: validate_safe_publication_preflight(
                non_ff_repository, divergent_sha, publication_old_sha, publication_old_sha
            ),
        )
        safe_publication_negative_tests += 1
        reject_value_error(
            "safe publication live state unavailable",
            lambda: read_live_remote_main_sha(publication_repository, "missing-remote"),
        )
    finally:
        publication_sandbox.cleanup()
        publication_temporary.cleanup()
    target_root_resolution_tests = 0
    root_change_staleness_tests = 0
    missing_temporary, missing_repository, missing_target, _ = _isolated_real_fixture(
        populate_authority=False
    )
    try:
        real_context = make_repository_context(missing_repository, missing_target)
        real_body = G3_FIXTURE_BODY.replace(
            b"specification_bundle_manifest_sha256=" + b"0" * 64,
            b"specification_bundle_manifest_sha256="
            + (real_context.bundle_manifest_sha256 or "0" * 64).encode(),
            1,
        )
        reject_value_error(
            "missing real G3 prerequisites",
            lambda: validate_g3_tag(G3_TAG_NAME, real_body, real_context),
        )
    finally:
        missing_temporary.cleanup()
    fixture_temporary, fixture_repository, fixture_target, fixture_body = _isolated_real_fixture()
    try:
        fixture_context = make_repository_context(
            fixture_repository, fixture_target, allow_test_only=True
        )
        if fixture_context.resolution["errors"] or fixture_context.resolution["missing"]:
            raise AssertionError(
                f"real fixture resolution failed: {fixture_context.resolution}"
            )
        fixture_positive = validate_g3_tag(
            G3_TAG_NAME, fixture_body, fixture_context
        )
        if fixture_positive["approval_decision"] != "GO":
            raise AssertionError(f"real fixture positive result: {fixture_positive}")
        if (
            not fixture_context.remediation_authority_id
            or not re.fullmatch(r"[0-9a-f]{64}", fixture_context.remediation_authority_id)
            or any(
                not re.fullmatch(r"[0-9a-f]{64}", identifier)
                for identifier in fixture_context.reviewer_authorities
            )
            or any(
                not re.fullmatch(r"[0-9a-f]{64}", identifier)
                for identifier in fixture_context.review_artifacts
            )
            or len(fixture_context.reviewer_actor_attestations) != len(REVIEW_ROLES)
            or any(
                record.get("signature_verified") is not True
                for record in fixture_context.reviewer_actor_attestations.values()
            )
            or fixture_context.reviewer_bootstrap_root is None
            or fixture_context.reviewer_bootstrap_currentness is None
        ):
            raise AssertionError("real fixture resolved an incomplete actor authority chain")

        production_temporary, production_repository, production_target, production_body = (
            _isolated_real_fixture(authority_class="REAL")
        )
        try:
            production_context = make_repository_context(
                production_repository, production_target
            )
            if (
                production_context.mode != "real"
                or production_context.allow_test_only_authority
                or hasattr(production_context, "r11_currentness_verified")
                or production_context.reviewer_bootstrap_root is None
                or production_context.reviewer_bootstrap_currentness is None
            ):
                raise AssertionError(
                    "production-format fixture did not resolve the signed bootstrap chain"
                )
            production_positive = validate_g3_tag(
                G3_TAG_NAME, production_body, production_context
            )
            if production_positive["approval_decision"] != "GO":
                raise AssertionError(
                    f"production-format fixture positive result: {production_positive}"
                )
        finally:
            production_temporary.cleanup()

        fixture_graph_nodes = _graph_nodes(fixture_context.graph)
        direct_target_matrix_tests += 1
        readiness_source = "implementation_readiness_specification"
        readiness_source_rule = fixture_context.graph["node_identity_rules"][
            readiness_source
        ]
        fixture_context.objects[readiness_source] = _load_real_repository_file(
            fixture_repository,
            fixture_target,
            fixture_context.graph,
            readiness_source,
        )
        readiness_sha = sha256(fixture_repository / readiness_source_rule["path"])
        readiness_target = _review_target_for_source(
            fixture_graph_nodes[readiness_source]["authority_kind"],
            fixture_target,
            readiness_sha,
        )
        readiness_rule = fixture_context.graph["node_identity_rules"]["readiness_review"]
        readiness_record = _parse_json_without_duplicates(
            (fixture_repository / readiness_rule["path"]).read_bytes()
        )
        if not isinstance(readiness_record, dict):
            raise AssertionError("readiness review fixture was not an object")
        fixture_context.objects["readiness_review"] = _load_real_json_authority(
            fixture_repository,
            fixture_context.graph,
            "readiness_review",
            fixture_repository / readiness_rule["path"],
        )
        readiness_bundle = fixture_context.objects["readiness_review"]["canonical_object"]
        for readiness_row in readiness_bundle["reviews"]:
            readiness_artifact_id = readiness_row["review_artifact_reference"][
                "immutable_uri"
            ].removeprefix(REVIEW_ARTIFACT_URI_PREFIX)
            readiness_artifact_path = fixture_repository / fixture_context.graph[
                "review_reference_contract"
            ]["artifact"]["authority_path_template"].replace(
                "{review_artifact_id}", readiness_artifact_id
            )
            fixture_context.review_artifacts[readiness_artifact_id] = _load_real_reference_json(
                fixture_repository,
                fixture_context.graph,
                "artifact",
                readiness_artifact_path,
                readiness_artifact_id,
            )
        _validate_independent_review_bundle(
            fixture_context,
            "readiness_review",
            fixture_context.objects["readiness_review"],
        )
        _validate_review_target(
            "readiness_review", readiness_record.get("target"), readiness_target
        )
        reject_value_error(
            "readiness review Git target substituted for external target",
            lambda: _validate_review_target(
                "readiness_review",
                {"type": "external_object", "object_kind": "decision_bundle", "object_sha256": "e" * 64},
                readiness_target,
            ),
        )
        wrong_readiness_identity = dict(readiness_target)
        readiness_digest_field = (
            "git_sha"
            if readiness_target["type"] == "git_commit"
            else "object_sha256"
        )
        wrong_readiness_identity[readiness_digest_field] = "e" * len(
            wrong_readiness_identity[readiness_digest_field]
        )
        reject_value_error(
            "readiness review wrong target identity",
            lambda: _validate_review_target(
                "readiness_review",
                wrong_readiness_identity,
                readiness_target,
            ),
        )
        direct_target_matrix_negative_tests += 1
        direct_target_matrix_negative_tests += 1

        def real_direct_duplicate_actor_mutation(
            context: G3AuthorityContext, node_id: str, left: int, right: int
        ) -> None:
            bundle = context.objects[node_id]["canonical_object"]
            rows = bundle["reviews"]
            left_artifact_id = rows[left]["review_artifact_reference"][
                "immutable_uri"
            ].removeprefix(REVIEW_ARTIFACT_URI_PREFIX)
            right_artifact_id = rows[right]["review_artifact_reference"][
                "immutable_uri"
            ].removeprefix(REVIEW_ARTIFACT_URI_PREFIX)
            left_reviewer_id = context.review_artifacts[left_artifact_id][
                "reviewer_authority_id"
            ]
            right_reviewer = deepcopy(
                context.reviewer_authorities[
                    context.review_artifacts[right_artifact_id][
                        "reviewer_authority_id"
                    ]
                ]["canonical_object"]
            )
            right_reviewer["actor_identity_digest"] = context.reviewer_authorities[
                left_reviewer_id
            ]["actor_identity_digest"]
            right_reviewer_id = sha256_bytes(
                canonical_json_bytes(
                    {
                        key: value
                        for key, value in right_reviewer.items()
                        if key != "reviewer_authority_id"
                    }
                )
            )
            right_reviewer["reviewer_authority_id"] = right_reviewer_id
            right_reviewer_bytes = canonical_json_bytes(right_reviewer)
            right_reviewer_record = dict(right_reviewer)
            right_reviewer_record.update(
                {
                    "bytes": right_reviewer_bytes,
                    "canonical_object": right_reviewer,
                    "sha256": right_reviewer_id,
                    "expected_sha256": right_reviewer_id,
                    "content_unchanged": True,
                }
            )
            context.reviewer_authorities[right_reviewer_id] = right_reviewer_record

            right_artifact = deepcopy(
                context.review_artifacts[right_artifact_id]["canonical_object"]
            )
            right_artifact["reviewer_authority_id"] = right_reviewer_id
            right_artifact_id = sha256_bytes(
                canonical_json_bytes(
                    {
                        key: value
                        for key, value in right_artifact.items()
                        if key != "review_artifact_id"
                    }
                )
            )
            right_artifact["review_artifact_id"] = right_artifact_id
            right_artifact_bytes = canonical_json_bytes(right_artifact)
            right_artifact_record = dict(right_artifact)
            right_artifact_record.update(
                {
                    "bytes": right_artifact_bytes,
                    "canonical_object": right_artifact,
                    "sha256": right_artifact_id,
                    "expected_sha256": right_artifact_id,
                    "content_unchanged": True,
                }
            )
            context.review_artifacts[right_artifact_id] = right_artifact_record
            rows[right]["review_artifact_reference"] = {
                "immutable_uri": f"{REVIEW_ARTIFACT_URI_PREFIX}{right_artifact_id}",
                "sha256": sha256_bytes(right_artifact_bytes),
                "byte_length": str(len(right_artifact_bytes)),
            }
            bundle["review_bundle_id"] = independent_review_bundle_id(bundle)

        real_direct_actor_pair_tests = 0
        for review_node in ("architecture_review", "readiness_review"):
            for left, right in (
                (left, right) for left in range(5) for right in range(left + 1, 5)
            ):
                mutant = deepcopy(fixture_context)
                real_direct_duplicate_actor_mutation(
                    mutant, review_node, left, right
                )
                real_direct_actor_pair_tests += 1
                reject_value_error(
                    f"real direct duplicate actor pair {review_node} {left},{right}",
                    lambda mutant=mutant, review_node=review_node: _validate_independent_review_bundle(
                        mutant, review_node, mutant.objects[review_node]
                    ),
                )
        if real_direct_actor_pair_tests != 20:
            raise AssertionError("real-format direct actor pair enumeration is incomplete")

        fixture_graph_path = fixture_repository / AUTHORITY_GRAPH_PATH.relative_to(ROOT)
        original_fixture_graph_bytes = fixture_graph_path.read_bytes()
        fixture_graph_path.write_bytes(valid_root_bytes)
        target_root_context = make_repository_context(
            fixture_repository, fixture_target, allow_test_only=True
        )
        if (
            target_root_context.authority_graph_bytes != original_fixture_graph_bytes
            or target_root_context.authority_graph_sha256
            != sha256_bytes(original_fixture_graph_bytes)
        ):
            raise AssertionError(
                "real resolver accepted a worktree graph in place of the selected target root"
            )
        target_root_resolution_tests += 1
        fixture_graph_path.write_bytes(original_fixture_graph_bytes)
        fixture_graph_path.write_bytes(valid_root_bytes)
        _fixture_git(
            fixture_repository,
            ["add", str(fixture_graph_path.relative_to(fixture_repository))],
        )
        _fixture_git(
            fixture_repository, ["commit", "-qm", "fixture normative binding-root change"]
        )
        changed_root_target = _fixture_git(
            fixture_repository, ["rev-parse", "HEAD"]
        ).decode().strip()
        changed_root_context = make_repository_context(
            fixture_repository, changed_root_target
        )
        if changed_root_context.authority_graph_sha256 != valid_root_sha256:
            raise AssertionError("changed target did not resolve the changed graph root")
        reject_value_error(
            "stale authority after normative binding-root change",
            lambda: validate_g3_tag(
                G3_TAG_NAME, fixture_body, changed_root_context
            ),
        )
        root_change_staleness_tests += 1

        real_negative_cases_tested = 0
        real_actor_positive_tests = 1
        real_actor_negative_tests = 0

        def real_reject(label: str, mutate: object) -> None:
            nonlocal real_negative_cases_tested
            mutant = deepcopy(fixture_context)
            mutate(mutant)
            reject_value_error(
                label,
                lambda: validate_g3_tag(G3_TAG_NAME, fixture_body, mutant),
            )
            real_negative_cases_tested += 1

        def real_reviewer_actor_mutation(
            context: G3AuthorityContext, row_index: int
        ) -> None:
            migrated_record = context.objects["migrated_finding_review"]
            row = migrated_record["review_records"][row_index]
            original = context.reviewer_authorities[row["reviewer_authority_id"]]
            canonical = deepcopy(original["canonical_object"])
            canonical["actor_identity_digest"] = context.remediation_actor_identity_digest
            identity_payload = {
                key: value
                for key, value in canonical.items()
                if key != "reviewer_authority_id"
            }
            replacement_id = sha256_bytes(canonical_json_bytes(identity_payload))
            canonical["reviewer_authority_id"] = replacement_id
            raw = canonical_json_bytes(canonical)
            replacement = dict(canonical)
            replacement.update(
                {
                    "bytes": raw,
                    "canonical_object": canonical,
                    "sha256": replacement_id,
                    "expected_sha256": replacement_id,
                    "content_unchanged": True,
                }
            )
            context.reviewer_authorities[replacement_id] = replacement
            original_artifact_id = row["review_artifact_id"]
            artifact = context.review_artifacts.pop(original_artifact_id)
            artifact_canonical = deepcopy(artifact["canonical_object"])
            artifact_canonical["reviewer_authority_id"] = replacement_id
            artifact_identity_payload = {
                key: value
                for key, value in artifact_canonical.items()
                if key != "review_artifact_id"
            }
            replacement_artifact_id = sha256_bytes(
                canonical_json_bytes(artifact_identity_payload)
            )
            artifact_canonical["review_artifact_id"] = replacement_artifact_id
            artifact_raw = canonical_json_bytes(artifact_canonical)
            replacement_artifact = dict(artifact_canonical)
            replacement_artifact.update(
                {
                    "bytes": artifact_raw,
                    "canonical_object": artifact_canonical,
                    "sha256": replacement_artifact_id,
                    "expected_sha256": replacement_artifact_id,
                    "content_unchanged": True,
                }
            )
            context.review_artifacts[replacement_artifact_id] = replacement_artifact
            row["reviewer_authority_id"] = replacement_id
            row["review_artifact_id"] = replacement_artifact_id
            row["independence_relation"] = {
                "type": "distinct_reviewer_authority",
                "reviewer_authority_id": replacement_id,
            }
            refresh_review_row_hash(row)

        def migrated_reviewer_and_attestation(
            context: G3AuthorityContext, row_index: int = 0
        ) -> tuple[dict[str, Any], dict[str, Any]]:
            row = context.objects["migrated_finding_review"]["review_records"][row_index]
            reviewer = context.reviewer_authorities[row["reviewer_authority_id"]]
            attestation = context.reviewer_actor_attestations[
                reviewer["actor_attestation_id"]
            ]
            return reviewer, attestation

        def actor_reject(label: str, mutate: object) -> None:
            nonlocal real_actor_negative_tests
            real_reject(label, mutate)
            real_actor_negative_tests += 1

        actor_reject(
            "real arbitrary actor digest",
            lambda context: migrated_reviewer_and_attestation(context)[0].update(
                {"actor_identity_digest": "a" * 64}
            ),
        )
        actor_reject(
            "real missing actor attestation",
            lambda context: context.reviewer_actor_attestations.pop(
                migrated_reviewer_and_attestation(context)[0]["actor_attestation_id"]
            ),
        )
        actor_reject(
            "real fake bootstrap root",
            lambda context: context.__setattr__("reviewer_bootstrap_root", None),
        )
        actor_reject(
            "real bootstrap trust-source cross-wire",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {"trust_source": {
                    "type": REVIEWER_BOOTSTRAP_TRUST_SOURCE,
                    "root_id": "sha256:" + "e" * 64,
                    "root_sha256": "e" * 64,
                    "currentness_proof_id": "sha256:" + "e" * 64,
                    "currentness_proof_sha256": "e" * 64,
                }}
            ),
        )
        actor_reject(
            "real future G5 enrollment dependency",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {"trust_source": {
                    "type": "phase_f_registry",
                    "root_id": "sha256:" + "e" * 64,
                    "root_sha256": "e" * 64,
                    "currentness_proof_id": "sha256:" + "e" * 64,
                    "currentness_proof_sha256": "e" * 64,
                }}
            ),
        )
        actor_reject(
            "real invalid actor attestation signature",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {"signature_verified": False}
            ),
        )
        actor_reject(
            "real wrong role eligibility",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {"eligible_role": "security"}
            ),
        )
        actor_reject(
            "real stale actor attestation",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {"stale": True}
            ),
        )
        actor_reject(
            "real superseded actor attestation",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {"superseded_by": "sha256:" + "e" * 64}
            ),
        )
        actor_reject(
            "real invalidated actor attestation",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {"invalidated": True}
            ),
        )
        actor_reject(
            "real actor bootstrap proof hash cross-wire",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {"trust_source": {
                    "type": REVIEWER_BOOTSTRAP_TRUST_SOURCE,
                    "root_id": context.reviewer_bootstrap_root["root_id"],
                    "root_sha256": context.reviewer_bootstrap_root["complete_file_sha256"],
                    "currentness_proof_id": context.reviewer_bootstrap_currentness["currentness_proof_id"],
                    "currentness_proof_sha256": "e" * 64,
                }}
            ),
        )
        actor_reject(
            "real actor/remediation alias",
            lambda context: migrated_reviewer_and_attestation(context)[1].update(
                {
                    "independence_excluded_actor_identity_digest": "e" * 64,
                }
            ),
        )
        actor_reject(
            "real reviewer/attestation cross-wire",
            lambda context: migrated_reviewer_and_attestation(context)[0].update(
                {
                    "actor_attestation_id": context.reviewer_authorities[
                        context.objects["migrated_finding_review"]["review_records"][1][
                            "reviewer_authority_id"
                        ]
                    ]["actor_attestation_id"],
                }
            ),
        )

        def mutate_same_subject_multiple_enrollments(
            context: G3AuthorityContext,
        ) -> None:
            rows = context.objects["migrated_finding_review"]["review_records"]
            first_reviewer = context.reviewer_authorities[rows[0]["reviewer_authority_id"]]
            first_attestation = context.reviewer_actor_attestations[
                first_reviewer["actor_attestation_id"]
            ]
            subject = first_attestation["actor_subject_id"]
            digest = first_reviewer["actor_identity_digest"]
            for row in rows[1:]:
                reviewer = context.reviewer_authorities[row["reviewer_authority_id"]]
                attestation = context.reviewer_actor_attestations[
                    reviewer["actor_attestation_id"]
                ]
                attestation["actor_subject_id"] = subject
                reviewer["actor_identity_digest"] = digest

        actor_reject(
            "real same-subject multiple-enrollment alias",
            mutate_same_subject_multiple_enrollments,
        )
        actor_reject(
            "real same-subject five-key alias",
            lambda context: [
                context.reviewer_authorities[row["reviewer_authority_id"]].update(
                    {
                        "actor_identity_digest": context.reviewer_authorities[
                            context.objects["migrated_finding_review"]["review_records"][0][
                                "reviewer_authority_id"
                            ]
                        ]["actor_identity_digest"]
                    }
                )
                for row in context.objects["migrated_finding_review"]["review_records"][1:]
            ],
        )
        actor_reject(
            "real bootstrap currentness proof unavailable",
            lambda context: (
                context.__setattr__("mode", "real"),
                context.__setattr__("allow_test_only_authority", False),
                context.__setattr__("reviewer_bootstrap_currentness", None),
            ),
        )
        actor_reject(
            "real stale bootstrap root",
            lambda context: context.reviewer_bootstrap_root.update({"stale": True}),
        )
        actor_reject(
            "real superseded bootstrap root",
            lambda context: context.reviewer_bootstrap_root.update(
                {"superseded_by": "sha256:" + "e" * 64}
            ),
        )
        actor_reject(
            "real invalidated bootstrap root",
            lambda context: context.reviewer_bootstrap_root.update({"invalidated": True}),
        )
        actor_reject(
            "real revoked bootstrap root proof",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"root_revoked": True}
            ),
        )
        actor_reject(
            "real compromised bootstrap root proof",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"root_compromised": True}
            ),
        )
        actor_reject(
            "real revoked bootstrap verifier",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"verifier_revoked": True}
            ),
        )
        actor_reject(
            "real compromised bootstrap verifier",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"verifier_compromised": True}
            ),
        )
        actor_reject(
            "real stale bootstrap currentness proof",
            lambda context: context.reviewer_bootstrap_currentness.update({"stale": True}),
        )
        actor_reject(
            "real superseded bootstrap currentness proof",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"superseded_by": "sha256:" + "e" * 64}
            ),
        )
        actor_reject(
            "real invalidated bootstrap currentness proof",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"invalidated": True}
            ),
        )
        actor_reject(
            "real bootstrap currentness head mismatch",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"head_id": "sha256:" + "e" * 64}
            ),
        )
        actor_reject(
            "real bootstrap subject-head mismatch",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"subject_registry_head_sha256": "e" * 64}
            ),
        )
        actor_reject(
            "real bootstrap invalid signature",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"signature": "00" * 64}
            ),
        )

        def mutate_valid_signature_from_wrong_bootstrap_signer(
            context: G3AuthorityContext,
        ) -> None:
            proof = context.reviewer_bootstrap_currentness
            if proof is None:
                raise AssertionError("fixture currentness proof missing")
            signing_payload = {
                key: proof[key]
                for key in REVIEWER_BOOTSTRAP_CURRENTNESS_FIELDS
                if key != "signature"
            }
            proof["signature"] = _fixture_ed25519_sign(
                b"phase-f-r12-fixture-wrong-signer",
                REVIEWER_BOOTSTRAP_CURRENTNESS_DOMAIN
                + canonical_jcs_bytes(signing_payload),
            )
            proof["bytes"] = canonical_json_bytes(
                {key: proof[key] for key in REVIEWER_BOOTSTRAP_CURRENTNESS_FIELDS}
            )
            proof["complete_file_sha256"] = sha256_bytes(proof["bytes"])

        actor_reject(
            "real bootstrap proof signed by another valid-looking key",
            mutate_valid_signature_from_wrong_bootstrap_signer,
        )
        actor_reject(
            "real bootstrap missing predecessor chain",
            lambda context: context.reviewer_bootstrap_currentness.update(
                {"sequence": 1}
            ),
        )
        actor_reject(
            "real graph-pinned arbitrary bootstrap root",
            lambda context: context.graph["reviewer_bootstrap_trust_contract"].update(
                {"root_id": "sha256:" + "e" * 64}
            ),
        )
        actor_reject(
            "real TEST_ONLY bootstrap material in REAL mode",
            lambda context: context.__setattr__("mode", "real"),
        )

        same_person_different_subject = deepcopy(
            fixture_context.reviewer_bootstrap_currentness
        )
        same_person_different_subject["subject_bindings"][1][
            "identity_evidence_sha256"
        ] = same_person_different_subject["subject_bindings"][0][
            "identity_evidence_sha256"
        ]
        same_person_different_subject["subject_registry_head_sha256"] = (
            reviewer_bootstrap_subject_registry_head_sha256(
                same_person_different_subject["sequence"],
                same_person_different_subject["subject_bindings"],
            )
        )
        reject_value_error(
            "bootstrap same-person different-subject alias",
            lambda: _bootstrap_subject_index(same_person_different_subject),
        )

        def remove_real_binding(
            context: G3AuthorityContext, target: str, relation: str, source: str
        ) -> None:
            nodes = _graph_nodes(context.graph)
            edges = _graph_edges(context.graph, nodes)
            descriptor = next(
                descriptor
                for descriptor in derive_binding_projection(context.graph, nodes, edges)
                if descriptor["target"] == target
                and descriptor["relation"] == relation
                and descriptor["source"] == source
            )
            field_name = descriptor["field"]
            actual = context.objects[target].get(field_name)
            if field_name == "target" and _is_independent_review_bundle(
                context.objects[target]
            ):
                context.objects[target].pop(field_name, None)
            elif isinstance(actual, dict):
                actual.pop(source, None)
            else:
                context.objects[target].pop(field_name, None)

        real_reject(
            "real missing architecture binding",
            lambda context: remove_real_binding(
                context, "specification_bundle_inputs", "binds", "architecture_approval"
            ),
        )
        real_reject(
            "real missing F0 binding",
            lambda context: remove_real_binding(
                context, "specification_bundle_inputs", "binds", "f0_approval"
            ),
        )
        real_reject(
            "real missing component binding",
            lambda context: remove_real_binding(
                context, "specification_bundle_manifest", "binds", "component_wire_review"
            ),
        )
        real_reject(
            "real missing migrated-review binding",
            lambda context: remove_real_binding(
                context, "specification_bundle_manifest", "binds", "migrated_finding_review"
            ),
        )
        real_reject(
            "real missing generated-source binding",
            lambda context: remove_real_binding(
                context, "generated_traceability_manifest", "generated_from", "architecture_plan"
            ),
        )
        real_reject(
            "real missing aggregate target binding",
            lambda context: remove_real_binding(
                context, "aggregate_review", "targets", "specification_bundle_manifest"
            ),
        )
        real_reject(
            "real missing G3-required prerequisite",
            lambda context: context.objects.pop("normative_traceability_matrix"),
        )

        real_reject(
            "real missing authority object",
            lambda context: context.objects.pop("architecture_approval"),
        )
        real_reject(
            "real malformed authority object",
            lambda context: context.objects["component_wire_review"].update(
                {"canonical_object": {"malformed": True}}
            ),
        )
        real_reject(
            "real noncanonical authority object",
            lambda context: context.objects["component_wire_review"].update(
                {"canonical_object": {"node_id": "component_wire_review"}}
            ),
        )
        real_reject(
            "real wrong authority digest",
            lambda context: context.objects["architecture_approval"].update(
                {"sha256": "e" * 64}
            ),
        )
        real_reject(
            "real stale architecture approval",
            lambda context: context.objects["architecture_approval"].update(
                {"stale": True}
            ),
        )
        real_reject(
            "real wrong architecture target",
            lambda context: context.objects["architecture_approval"].update(
                {"target_sha256": "e" * 64}
            ),
        )
        real_reject(
            "real missing component review",
            lambda context: context.objects.pop("component_wire_review"),
        )
        real_reject(
            "real wrong component target",
            lambda context: context.objects["component_wire_review"].update(
                {"target": {"type": "git_commit", "git_sha": "e" * 40}}
            ),
        )
        real_reject(
            "real stale component review",
            lambda context: context.objects["component_wire_review"].update(
                {"stale": True}
            ),
        )
        real_reject(
            "real superseded F0 approval",
            lambda context: context.objects["f0_approval"].update(
                {"superseded_by": "fixture:replacement"}
            ),
        )
        real_reject(
            "real stale F0 approval",
            lambda context: context.objects["f0_approval"].update(
                {"stale": True}
            ),
        )
        real_reject(
            "real wrong F0 target",
            lambda context: context.objects["f0_approval"].update(
                {"target_sha256": "e" * 64}
            ),
        )
        real_reject(
            "real wrong F0 digest",
            lambda context: context.objects["f0_approval"].update(
                {"sha256": "e" * 64}
            ),
        )
        real_reject(
            "real invalidated aggregate review",
            lambda context: context.objects["aggregate_review"].update(
                {"invalidated": True}
            ),
        )
        real_reject(
            "real migrated unresolved disposition",
            lambda context: context.objects["migrated_finding_review"][
                "finding_dispositions"
            ].update({"F-PLAN-R11-P1-01": "OPEN"}),
        )
        real_reject(
            "real migrated review row missing",
            lambda context: context.objects["migrated_finding_review"].update(
                {"review_records": []}
            ),
        )
        real_reject(
            "real missing migrated review",
            lambda context: context.objects.pop("migrated_finding_review"),
        )
        real_reject(
            "real migrated target wrong",
            lambda context: context.objects["migrated_finding_review"].update(
                {"target_bundle_inputs_sha256": "e" * 64}
            ),
        )
        real_reject(
            "real migrated digest wrong",
            lambda context: context.objects["migrated_finding_review"].update(
                {"sha256": "e" * 64}
            ),
        )
        real_reject(
            "real migrated review identity duplicate",
            lambda context: context.objects["migrated_finding_review"][
                "review_records"
            ][1].update(
                {
                    "reviewer_authority_id": context.objects[
                        "migrated_finding_review"
                    ]["review_records"][0]["reviewer_authority_id"]
                }
            ),
        )
        real_reject(
            "real migrated review artifact identity duplicate",
            lambda context: (
                context.objects["migrated_finding_review"]["review_records"][1].update(
                    {
                        "review_artifact_id": context.objects[
                            "migrated_finding_review"
                        ]["review_records"][0]["review_artifact_id"]
                    }
                ),
                refresh_review_row_hash(
                    context.objects["migrated_finding_review"]["review_records"][1]
                ),
            ),
        )
        real_reject(
            "real unresolved reviewer identity",
            lambda context: (
                context.objects["migrated_finding_review"]["review_records"][0].update(
                    {
                        "reviewer_authority_id": "e" * 64,
                        "independence_relation": {
                            "type": "distinct_reviewer_authority",
                            "reviewer_authority_id": "e" * 64,
                        },
                    }
                ),
                refresh_review_row_hash(
                    context.objects["migrated_finding_review"]["review_records"][0]
                ),
            ),
        )
        real_reject(
            "real unresolved review artifact identity",
            lambda context: (
                context.objects["migrated_finding_review"]["review_records"][0].update(
                    {"review_artifact_id": "f" * 64}
                ),
                refresh_review_row_hash(
                    context.objects["migrated_finding_review"]["review_records"][0]
                ),
            ),
        )
        real_reject(
            "real implementation author substituted as reviewer",
            lambda context: real_reviewer_actor_mutation(context, 0),
        )
        real_reject(
            "real review artifact decision NO-GO",
            lambda context: (
                context.review_artifacts[
                    context.objects["migrated_finding_review"]["review_records"][0][
                        "review_artifact_id"
                    ]
                ].update({"decision": "NO-GO"}),
                context.objects["migrated_finding_review"]["review_records"][0].update(
                    {"decision": "NO-GO"}
                ),
                refresh_review_row_hash(
                    context.objects["migrated_finding_review"]["review_records"][0]
                ),
            ),
        )
        real_reject(
            "real review artifact role mismatch",
            lambda context: context.review_artifacts[
                context.objects["migrated_finding_review"]["review_records"][0][
                    "review_artifact_id"
                ]
            ].update({"role": "security"}),
        )
        real_reject(
            "real stale review artifact",
            lambda context: context.review_artifacts[
                context.objects["migrated_finding_review"]["review_records"][0][
                    "review_artifact_id"
                ]
            ].update({"stale": True}),
        )
        real_reject(
            "real review artifact wrong target",
            lambda context: context.review_artifacts[
                context.objects["migrated_finding_review"]["review_records"][0][
                    "review_artifact_id"
                ]
            ].update({"reviewed_target": "e" * 64}),
        )
        real_reject(
            "real migrated review target stale",
            lambda context: context.objects["migrated_finding_review"][
                "review_records"
            ][0].update({"reviewed_target": "e" * 64}),
        )
        real_reject(
            "real migrated review fingerprint stale",
            lambda context: context.objects["migrated_finding_review"].update(
                {"review_input_fingerprint": "e" * 64}
            ),
        )
        real_reject(
            "real migrated review target commit wrong",
            lambda context: context.objects["migrated_finding_review"].update(
                {"target_git_commit": "e" * 40}
            ),
        )
        real_reject(
            "real specification input binding missing",
            lambda context: context.objects["specification_bundle_inputs"][
                "authority_bindings"
            ].pop("architecture_approval"),
        )
        real_reject(
            "real specification input binding substituted",
            lambda context: context.objects["specification_bundle_inputs"][
                "authority_bindings"
            ]["architecture_approval"].update({"sha256": "e" * 64}),
        )
        real_reject(
            "real generated-source binding mismatch",
            lambda context: context.objects["generated_traceability_manifest"][
                "generated_source_sha256s"
            ].update({"architecture_plan": "e" * 64}),
        )
        real_reject(
            "real manifest bound-authority mismatch",
            lambda context: context.objects["specification_bundle_manifest"][
                "bound_authority_sha256s"
            ].update({"migrated_finding_review": "e" * 64}),
        )
        real_reject(
            "real aggregate target mismatch",
            lambda context: context.objects["aggregate_review"].update(
                {"target": {"type": "git_commit", "git_sha": "e" * 40}}
            ),
        )
        real_reject(
            "real manifest wrong hash",
            lambda context: context.objects["specification_bundle_manifest"].update(
                {"sha256": "e" * 64}
            ),
        )
        real_reject(
            "real manifest changed bytes",
            lambda context: context.objects["specification_bundle_manifest"].update(
                {"content_unchanged": False}
            ),
        )
        real_reject(
            "real manifest wrong target",
            lambda context: context.objects["specification_bundle_manifest"].update(
                {"target_commit": "e" * 40}
            ),
        )
        real_reject(
            "real stale bundle inputs",
            lambda context: context.objects["specification_bundle_inputs"].update(
                {"sha256": "e" * 64}
            ),
        )
        real_reject(
            "real missing aggregate review",
            lambda context: context.objects.pop("aggregate_review"),
        )
        real_reject(
            "real stale aggregate review",
            lambda context: context.objects["aggregate_review"].update(
                {"stale": True}
            ),
        )
        real_reject(
            "real aggregate wrong digest",
            lambda context: context.objects["aggregate_review"].update(
                {"sha256": "e" * 64}
            ),
        )
        real_reject(
            "real G3 tag peel mismatch",
            lambda context: context.tag.update({"peeled_commit": "e" * 40}),
        )
        real_reject(
            "real G3 tag message mismatch",
            lambda context: context.tag.update({"message": b"wrong\n"}),
        )
        real_reject(
            "real G3 lightweight tag",
            lambda context: context.tag.update(
                {"annotated": False, "object_type": "commit"}
            ),
        )

        def resolver_reject(
            label: str, relative_path: str, replacement: bytes | None
        ) -> None:
            path = fixture_repository / relative_path
            original = path.read_bytes() if path.exists() else None
            if replacement is None:
                path.unlink()
            else:
                path.write_bytes(replacement)
            try:
                resolved = make_repository_context(
                    fixture_repository, fixture_target, allow_test_only=True
                )
                if not resolved.resolution["errors"]:
                    raise AssertionError(f"real resolver accepted mutation: {label}")
            finally:
                if original is None:
                    path.unlink(missing_ok=True)
                else:
                    path.write_bytes(original)

        resolver_reject(
            "real resolver missing authority file",
            ".phase_f_authority/architecture_review.json",
            None,
        )
        resolver_reject(
            "real resolver malformed authority file",
            ".phase_f_authority/f0_review.json",
            b"{\n",
        )
        resolver_reject(
            "real resolver noncanonical authority file",
            ".phase_f_authority/component_wire_review.json",
            b"{ "
            + (
                fixture_repository / ".phase_f_authority/component_wire_review.json"
            ).read_bytes()[1:],
        )
        resolver_reject(
            "real resolver schema mismatch",
            ".phase_f_authority/aggregate_review.json",
            canonical_json_bytes(
                {
                    key: value
                    for key, value in fixture_context.objects["aggregate_review"][
                        "canonical_object"
                    ].items()
                    if key != "aggregate_decision"
                }
            ),
        )
        resolver_reject(
            "real resolver missing remediation author",
            ".phase_f_authority/remediation_authority.json",
            None,
        )

        def resolver_tag_reject(label: str, tag_name: str) -> None:
            ref = f"refs/tags/{tag_name}"
            original = _git_output(fixture_repository, ["rev-parse", ref]).decode().strip()
            _fixture_git(fixture_repository, ["update-ref", "-d", ref])
            try:
                resolved = make_repository_context(
                    fixture_repository, fixture_target, allow_test_only=True
                )
                if not resolved.resolution["errors"]:
                    raise AssertionError(f"real resolver accepted mutation: {label}")
            finally:
                _fixture_git(fixture_repository, ["update-ref", ref, original])

        resolver_tag_reject(
            "real resolver missing architecture tag",
            G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"],
        )
        resolver_tag_reject(
            "real resolver missing F0 tag",
            G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"],
        )
    finally:
        fixture_temporary.cleanup()
    g3_reject(
        "synthetic context cannot authorize real",
        lambda context: context.__setattr__("real_authority_requested", True),
    )

    anchor_mutant = deepcopy(entries)
    anchor_mutant[0]["authority_anchor"] = "#undefined-anchor"
    try:
        validate_traceability(anchor_mutant)
    except ValueError:
        pass
    else:
        raise AssertionError("undefined anchor regression did not reject")
    print(
        "PHASE_F_SELF_TEST_PASS "
        f"requirements={len(entries)} tests={len(test_catalog)} evidence={len(evidence_catalog)} "
        f"g3_mutations={len(G3_KAT_MUTATIONS)} g3_authority_tests={len(R12_G3_TEST_IDS)} "
        f"traceability_tests={len(R12_TRACE_TEST_IDS)} dag_tests={len(R12_DAG_TEST_IDS)} "
        f"decision_vectors={decision_combinations_tested} invalid_decision_vectors={invalid_decision_combinations_accepted} "
        f"missing_roles={missing_role_cases_tested} duplicate_artifacts={duplicate_artifact_cases_tested} "
        f"duplicate_reviewers={duplicate_reviewer_cases_tested} unresolved_reviewers={unresolved_reviewer_cases_tested} "
        f"unresolved_artifacts={unresolved_artifact_cases_tested} role_mismatches={role_mismatch_cases_tested} "
        f"review_bundle_matrix_negative={review_bundle_matrix_negative_tests} "
        f"direct_target_matrix={direct_target_matrix_tests} "
        f"direct_target_matrix_negative={direct_target_matrix_negative_tests} "
        f"direct_actor_pair_tests={direct_actor_pair_tests} "
        f"available_direct_review_entrypoint_negative_tests={available_direct_review_entrypoint_negative_tests} "
        f"real_direct_actor_pair_tests={real_direct_actor_pair_tests} "
        f"real_actor_positive_tests={real_actor_positive_tests} "
        f"real_actor_negative_tests={real_actor_negative_tests} "
        f"graph_candidates={len(candidate_edges)} graph_authorized={len(authorized_node_edges)} "
        f"graph_unauthorized={len(unauthorized_candidates)} graph_accepted_unauthorized={accepted_unauthorized_edges} "
        f"edge_canonical_passes={authorized_edge_canonical_passes} edge_removals_rejected={authorized_edge_removals_rejected} "
        f"edge_retypes_rejected={authorized_edge_retypes_rejected} edge_redirects_rejected={authorized_edge_redirects_rejected} "
        f"binding_edges={len(binding_edges)} none_binding_edges={len(none_binding_edges)} "
        f"binding_obligation_structural_deletions={binding_obligation_structural_deletion_tests} "
        f"binding_obligation_structural_deletion_accepted=0 "
        f"binding_obligation_malformed_tests={binding_obligation_structural_malformed_tests} "
        f"binding_obligation_malformed_accepted=0 "
        f"node_mirror_downstream_tests={node_mirror_downstream_tests} node_mirror_downstream_accepted=0 "
        f"schema_mirror_downstream_tests={schema_mirror_downstream_tests} schema_mirror_downstream_accepted=0 "
        f"semantic_rule_downstream_tests={semantic_rule_downstream_tests} semantic_rule_downstream_accepted=0 "
        f"full_root_fixed_downstream_tests={full_root_fixed_downstream_tests} full_root_fixed_downstream_accepted=0 "
        f"unauthorized_none_downstream_tests={unauthorized_none_downstream_tests} unauthorized_none_downstream_accepted=0 "
        f"root_property_mutations={root_identity_mutation_tests} root_property_hash_changes={root_identity_mutations_changed} "
        f"valid_root_changes={valid_root_change_tests} invalid_root_property_tests={invalid_root_property_tests} "
        f"bundle_root_fingerprint_changed=1 target_root_resolution_tests={target_root_resolution_tests} "
        f"root_change_staleness_tests={root_change_staleness_tests} explicit_g3_bypass_tests={explicit_g3_bypass_tests} "
        f"explicit_g3_bypass_accepted={explicit_g3_bypass_accepted} "
        f"review_start_git_positive={review_start_git_positive_tests} "
        f"review_start_git_negative={review_start_git_negative_tests} "
        f"safe_publication_positive={safe_publication_positive_tests} "
        f"safe_publication_negative={safe_publication_negative_tests} "
        f"safe_publication_race_tests={safe_publication_race_tests} "
        f"semantic_rules_derived={len(independent_rules)} semantic_rules_declared={len(graph['binding_semantics']['serialized_rules'])} "
        f"selected_rules={len(selected_rules)} "
        f"rules_all={semantic_rule_policy_counts['all']} rules_none={semantic_rule_policy_counts['none']} rules_selected={semantic_rule_policy_counts['selected']} "
        f"selected_by_relation={','.join(f'{key}:{value}' for key, value in selected_rule_relation_counts.items())} "
        f"selected_by_target={','.join(f'{key}:{value}' for key, value in selected_rule_target_counts.items())} "
        f"selected_rule_deletions_tested={selected_rule_deletions_tested} "
        f"selected_rule_deletion_accepted={selected_rule_deletion_accepted} "
        f"coordinated_rule_mirror_deletions_tested={coordinated_rule_mirror_deletions_tested} "
        f"coordinated_rule_mirror_deletion_accepted={coordinated_rule_mirror_deletion_accepted} "
        f"coordinated_rule_mirror_builder_deletions_tested={coordinated_rule_mirror_builder_deletions_tested} "
        f"coordinated_rule_mirror_builder_deletion_accepted={coordinated_rule_mirror_builder_deletion_accepted} "
        f"coordinated_four_layer_deletions_tested={coordinated_four_layer_deletions_tested} "
        f"coordinated_four_layer_deletion_accepted={coordinated_four_layer_deletion_accepted} "
        f"coordinated_schema_downstream_deletions_tested={coordinated_schema_downstream_deletions_tested} "
        f"coordinated_schema_downstream_deletion_accepted={coordinated_schema_downstream_deletion_accepted} "
        f"migrated_root_staleness_tests={migrated_root_staleness_tests} "
        f"aggregate_root_staleness_tests={aggregate_root_staleness_tests} "
        f"unauthorized_semantic_rule_candidates={len(unauthorized_semantic_rule_candidates)} "
        f"accepted_unauthorized_semantic_rules={accepted_unauthorized_semantic_rules} "
        f"semantic_rule_category_mutations={semantic_rule_category_mutations} "
        f"semantic_rule_value_mutations={semantic_rule_value_mutations} "
        f"semantic_rule_field_mutations={semantic_rule_field_mutations} "
        f"semantic_rule_source_mutations={semantic_rule_source_mutations} "
        f"semantic_rule_relation_mutations={semantic_rule_relation_mutations} "
        f"semantic_rule_policy_mutations={semantic_rule_policy_mutations} "
        f"accepted_semantic_rule_policy_mutations={accepted_semantic_rule_policy_mutations} "
        f"serialized_relation_maps={len(relation_map_entries)} "
        f"serialized_relation_map_deletions={relation_map_mutation_counts['delete']} "
        f"serialized_relation_map_deletion_accepted=0 "
        f"serialized_relation_map_mutations={sum(relation_map_mutation_counts.values())} "
        f"serialized_relation_map_mutation_accepted=0 "
        f"serialized_semantic_rule_mutations={semantic_rule_category_mutations + semantic_rule_value_mutations + semantic_rule_field_mutations + semantic_rule_source_mutations + semantic_rule_relation_mutations} "
        f"serialized_semantic_rule_mutation_accepted=0 "
        f"serialized_binding_entries={len(binding_projection)} "
        f"serialized_binding_deletions={serialized_binding_deletions_tested} "
        f"serialized_binding_deletion_accepted=0 "
        f"serialized_binding_extra_tests={serialized_binding_extra_tests} "
        f"serialized_binding_extra_accepted=0 "
        f"required_inputs_derived={derived_input_count} required_inputs_declared={declared_input_count} "
        f"required_input_deletions={required_input_deletions_tested} required_input_deletion_accepted=0 "
        f"required_input_extra_tests={required_input_extra_tests} required_input_extra_accepted=0 "
        f"required_input_replacement_tests={required_input_replacement_tests} required_input_replacement_accepted=0 "
        f"required_input_duplicate_tests={required_input_duplicate_tests} required_input_duplicate_accepted=0 "
        f"required_input_whole_list_tests={required_input_whole_list_tests} required_input_whole_list_accepted=0 "
        f"required_input_empty_tests={required_input_empty_tests} required_input_empty_accepted=0 "
        f"real_negative_cases={real_negative_cases_tested}"
    )


def main() -> None:
    if len(sys.argv) > 1:
        if sys.argv[1] == "--check-kat" and len(sys.argv) == 2:
            validate_inventory()
            validate_r11_and_migration()
            validate_wire_catalog()
            validate_kat_spec()
            print(
                "PHASE_F_KAT_PASS "
                f"fixture_bytes={G3_FIXTURE_BYTE_LENGTH} mutations={len(G3_KAT_MUTATIONS)}"
            )
            return
        if sys.argv[1] == "--self-test" and len(sys.argv) == 2:
            run_regression_self_tests()
            return
        raise SystemExit("usage: generate_phase_f_manifests.py [--check-kat|--self-test]")
    trace = build_traceability()
    trace_bytes = (json.dumps(trace, indent=2, sort_keys=True) + "\n").encode()
    bundle = build_bundle(sha256_bytes(trace_bytes))
    bundle_bytes = (json.dumps(bundle, indent=2, sort_keys=True) + "\n").encode()
    TRACE_PATH.write_bytes(trace_bytes)
    BUNDLE_PATH.write_bytes(bundle_bytes)


if __name__ == "__main__":
    main()
