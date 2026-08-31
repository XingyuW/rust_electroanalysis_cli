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
from copy import deepcopy
from dataclasses import dataclass
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
EXPECTED_R11_TEST_COUNT = 28
EXPECTED_R11_EVIDENCE_COUNT = 20
EXPECTED_R12_SCHEMA_COUNT = 93
R12_SCHEMA_IDS = {
    "PhaseFSpecificationBundleApprovalV1",
    "PhaseFMigratedFindingReviewV1",
}
EXPECTED_R12_REQUIREMENT_COUNT = 64
EXPECTED_MIGRATED_FINDINGS = {
    "F-PLAN-R11-P1-01",
    "F-PLAN-R11-P1-02",
    "F-PLAN-R11-P1-03",
    "F-PLAN-R11-P1-04",
    "F-PLAN-R11-P3-01",
}
REVIEW_ROLES = {
    "scientific_metrology",
    "architecture_data",
    "security",
    "compatibility",
    "operations_governance",
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
EXPECTED_G3_REQUIRED_NODES = {
    "architecture_approval",
    "f0_approval",
    "component_wire_review",
    "component_scientific_review",
    "component_operations_review",
    "component_conformance_review",
    "component_implementation_review",
    "normative_traceability_matrix",
    "generated_traceability_manifest",
    "migration_ledger",
    "migrated_finding_review",
    "specification_bundle_manifest",
    "aggregate_review",
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
    bundle_manifest_sha256: str
    aggregate_review_sha256: str
    expected_target_commit: str
    tag: dict[str, Any]
    component_sha256s: list[str]
    component_sha_by_node: dict[str, str]
    architecture_plan_sha256: str
    f0_decisions_sha256: str
    real_authority_requested: bool = False


def canonical_json_bytes(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode(
        "utf-8"
    )


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
        by_id[node_id] = node
    return by_id


def _graph_edges(
    graph: dict[str, Any], nodes: dict[str, dict[str, Any]]
) -> list[dict[str, str]]:
    edges = graph.get("edges")
    if not isinstance(edges, list):
        raise ValueError("R12 graph edges must be an array")
    seen: set[tuple[str, str, str]] = set()
    result: list[dict[str, str]] = []
    for edge in edges:
        if not isinstance(edge, dict):
            raise ValueError("R12 graph edge is malformed")
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
        result.append({"from": source, "to": target, "type": edge_type})
    return result


def _topological_order(
    nodes: dict[str, dict[str, Any]], edges: list[dict[str, str]]
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
    node_id: str, edges: list[dict[str, str]], excluded: set[str] | None = None
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
    if any(not isinstance(rule, str) or not rule for rule in identity_rules.values()):
        raise ValueError("R12 identity-cycle rule is blank")
    nodes = _graph_nodes(graph)
    if set(nodes) != EXPECTED_GRAPH_NODE_IDS:
        raise ValueError("R12 graph node catalog is not closed")
    edges = _graph_edges(graph, nodes)
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
    if set(required) != EXPECTED_G3_REQUIRED_NODES:
        raise ValueError("R12 G3 required-node catalog is not closed")
    if any(node_id not in nodes for node_id in required):
        raise ValueError("R12 G3 required-node list references unknown node")
    required_inputs = graph.get("required_inputs")
    if not isinstance(required_inputs, dict) or set(required_inputs) != set(nodes):
        raise ValueError("R12 graph required-input closure is incomplete")
    edge_keys = {(edge["from"], edge["to"]) for edge in edges}
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
    if set(required_inputs[g3_node]) != set(required):
        raise ValueError("R12 G3 required-input closure does not match required nodes")
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

    return {
        "node_count": len(nodes),
        "edge_count": len(edges),
        "edge_types": sorted({edge["type"] for edge in edges}),
        "topological_order": order,
        "g3_ancestor_count": len(g3_ancestors),
        "g3_required_count": len(required),
        "hash_cycle": False,
        "self_reference": False,
        "self_git_cycle": False,
        "review_target_cycle": False,
        "future_object": False,
        "g3_bypass": False,
        "implementation_bypass": False,
    }


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


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
    canonical_object = record.get("canonical_object")
    if isinstance(canonical_object, dict):
        payload = dict(canonical_object)
        payload.pop("migrated_finding_review_id", None)
        return sha256_bytes(canonical_json_bytes(payload)) == expected
    value = record.get("bytes")
    return isinstance(value, bytes) and sha256_bytes(value) == expected


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


def _validate_migrated_review(
    context: G3AuthorityContext, record: dict[str, Any]
) -> None:
    required_fields = {
        "migrated_finding_review_id",
        "target_git_commit",
        "target_bundle_inputs_sha256",
        "reviewed_migration_ledger_sha256",
        "reviewed_traceability_manifest_sha256",
        "reviewed_component_sha256s",
        "reviewed_finding_ids",
        "finding_dispositions",
        "reviewer_roles",
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
    if context.mode == "real":
        canonical_object = record.get("canonical_object")
        if not isinstance(canonical_object, dict) or any(
            canonical_object.get(field) != record.get(field) for field in required_fields
        ):
            raise G3ValidationError("migrated_review_identity_mismatch")
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
        or any(not isinstance(value, str) or not value for value in dispositions.values())
    ):
        raise G3ValidationError("incomplete_migrated_finding_coverage")
    if record.get("reviewer_roles") != sorted(REVIEW_ROLES):
        raise G3ValidationError("non_independent_migrated_review")
    if record.get("producer") != "independent_review_panel":
        raise G3ValidationError("non_independent_migrated_review")
    if any(not isinstance(record.get(name), int) or record[name] < 0 for name in ("p0_count", "p1_count", "p2_count")):
        raise G3ValidationError("malformed_migrated_review_counts")
    if record.get("decision") != "GO" or record["p0_count"] != 0 or record["p1_count"] != 0:
        raise G3ValidationError("migrated_review_not_go")


def _validate_review_collection(
    context: G3AuthorityContext, fields: dict[str, str]
) -> None:
    graph_nodes = _graph_nodes(context.graph)
    component_nodes = sorted(
        node_id
        for node_id in context.graph["g3_required_nodes"]
        if node_id.startswith("component_") and node_id.endswith("_review")
    )
    if len(component_nodes) != 5:
        raise G3ValidationError("component_review_set_mismatch")
    for node_id in component_nodes:
        record = _require_authority_object(context, node_id)
        if record.get("decision") != "GO" or record.get("p0_count") != 0 or record.get("p1_count") != 0:
            raise G3ValidationError(f"{node_id}_not_go")
        spec_node = node_id.removesuffix("_review") + "_spec"
        if spec_node not in graph_nodes or record.get("target_sha256") != context.component_sha_by_node[spec_node]:
            raise G3ValidationError(f"wrong_{node_id}_target")

    migrated = _require_authority_object(context, "migrated_finding_review")
    _validate_migrated_review(context, migrated)

    aggregate = _require_authority_object(context, "aggregate_review")
    if aggregate.get("target_bundle_manifest_sha256") != fields[
        "specification_bundle_manifest_sha256"
    ]:
        raise G3ValidationError("aggregate_target_mismatch")
    required_dependencies = set(component_nodes) | {
        "migrated_finding_review",
        "specification_bundle_manifest",
        "generated_traceability_manifest",
    }
    if not required_dependencies.issubset(set(aggregate.get("dependency_node_ids", []))):
        raise G3ValidationError("aggregate_dependency_closure_incomplete")
    if aggregate.get("decision") != "GO" or aggregate.get("p0_count") != 0 or aggregate.get("p1_count") != 0:
        raise G3ValidationError("aggregate_review_not_go")
    if aggregate.get("sha256") != fields["aggregate_review_bundle_sha256"]:
        raise G3ValidationError("aggregate_hash_mismatch")


def validate_g3_tag(
    tag_name: str, body_bytes: bytes, context: G3AuthorityContext
) -> dict[str, str]:
    """Validate G3 wire bytes and the complete real/synthetic authority closure."""

    fields = parse_g3_tag(tag_name, body_bytes)
    if not isinstance(context, G3AuthorityContext):
        raise G3ValidationError("invalid_validation_context")
    if context.mode not in {"synthetic", "real"}:
        raise G3ValidationError("invalid_validation_context_mode")
    if context.mode == "synthetic" and context.real_authority_requested:
        raise G3ValidationError("synthetic_cannot_authorize_real")
    if fields["specification_bundle_manifest_sha256"] != context.bundle_manifest_sha256:
        raise G3ValidationError("wrong_bundle_hash")
    if fields["aggregate_review_bundle_sha256"] != context.aggregate_review_sha256:
        raise G3ValidationError("wrong_aggregate_review_hash")

    tag = context.tag
    if tag.get("exists") is not True:
        raise G3ValidationError("missing_real_g3_tag")
    if tag.get("annotated") is not True or tag.get("object_type") != "tag":
        raise G3ValidationError("lightweight_tag")
    if tag.get("peeled_commit") != context.expected_target_commit:
        raise G3ValidationError("g3_target_mismatch")
    if tag.get("message") != body_bytes:
        raise G3ValidationError("g3_message_mismatch")

    try:
        graph_audit = validate_r12_authority_graph(context.graph)
    except ValueError as error:
        raise G3ValidationError("invalid_authority_graph") from error
    if graph_audit["g3_required_count"] != len(context.graph["g3_required_nodes"]):
        raise G3ValidationError("invalid_authority_graph")
    for node_id in context.graph["g3_required_nodes"]:
        _require_authority_object(context, node_id)

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
        fields = validate_g3_tag(tag_name, body, make_synthetic_context())
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
    literal_ids = {"R12-POS-SPEC-BUNDLE-TAG", "R12-G3-AUTHORITY-CONTEXT-POS"}
    for test_id in EXPECTED_R12_TEST_CATALOG_IDS - literal_ids:
        if catalog[test_id]["kat_class"] != "constructive_plan_audit":
            raise ValueError(f"R12 constructive test category: {test_id}")
        if catalog[test_id]["expected_result"] != "REJECT" and test_id not in {"R12-DAG-VALID"}:
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
        "strict migrated-review schema, exact bundle-input target, five-role independence, finding disposition, lifecycle, staleness, and hash validator",
        "G2 review prerequisite for the specification bundle",
        "external authority object; registry publication is prohibited before later gate authority",
        "INVERSE(R12_CURRENT_NORMATIVE_REQUIREMENT_MATRIX,PhaseFMigratedFindingReviewV1)",
    ]:
        raise ValueError("migrated-finding schema catalog metadata mismatch")
    migrated_anchor = '<a id="schema-def-PhaseFMigratedFindingReviewV1"></a>'
    if wire_text.count(migrated_anchor) != 1:
        raise ValueError("migrated-finding schema definition anchor missing or duplicated")


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
    if sha256(R11_SOURCE) != EXPECTED_R11_SHA256:
        raise ValueError("preserved R11 source SHA-256 does not match the authority")
    if git_blob(R11_SOURCE) != EXPECTED_R11_GIT_BLOB:
        raise ValueError("preserved R11 source Git blob does not match the authority")

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
    payload = {
        "schema_version": 1,
        "artifact_kind": "phase_f_specification_bundle_inputs",
        "source_sha256s": source_sha256s,
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
            "reviewed_traceability_manifest_sha256": None,
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
    graph = json.loads(AUTHORITY_GRAPH_PATH.read_text())
    validate_r12_authority_graph(graph)
    component_paths = list(SPECS.values())
    component_sha256s = [sha256(path) for path in component_paths]
    component_sha_by_node = {
        component_node_id(prefix): sha256(path) for prefix, path in SPECS.items()
    }
    target = "synthetic-g3-target-commit"
    bundle_inputs_sha = "2" * 64
    trace_sha = "4" * 64
    migration_sha = "5" * 64
    matrix_sha = "3" * 64
    objects: dict[str, dict[str, Any]] = {
        "architecture_approval": _synthetic_record(
            graph,
            "architecture_approval",
            "a" * 64,
            tag_name=G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"],
            target_sha256=sha256(ARCH),
            decision="GO",
            p0_count=0,
            p1_count=0,
        ),
        "f0_approval": _synthetic_record(
            graph,
            "f0_approval",
            "b" * 64,
            tag_name=G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"],
            target_sha256="b" * 64,
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
            graph, "generated_traceability_manifest", trace_sha
        ),
        "specification_bundle_inputs": _synthetic_record(
            graph,
            "specification_bundle_inputs",
            bundle_inputs_sha,
            source_sha256s={
                "normative_traceability_matrix": matrix_sha,
                "migration_ledger": migration_sha,
                "generated_traceability_manifest": trace_sha,
            },
        ),
    }
    for index, node_id in enumerate(
        [
            "component_wire_review",
            "component_scientific_review",
            "component_operations_review",
            "component_conformance_review",
            "component_implementation_review",
        ],
        start=1,
    ):
        objects[node_id] = _synthetic_record(
            graph,
            node_id,
            f"{index + 10:064x}",
            decision="GO",
            p0_count=0,
            p1_count=0,
            target_sha256=component_sha256s[index - 1],
        )
    objects["migrated_finding_review"] = _synthetic_record(
        graph,
        "migrated_finding_review",
        "d" * 64,
        migrated_finding_review_id="d" * 64,
        target_git_commit=target,
        target_bundle_inputs_sha256=bundle_inputs_sha,
        reviewed_migration_ledger_sha256=migration_sha,
        reviewed_traceability_manifest_sha256=trace_sha,
        reviewed_component_sha256s=sorted(component_sha256s),
        reviewed_finding_ids=sorted(EXPECTED_MIGRATED_FINDINGS),
        finding_dispositions={
            finding_id: "PENDING_FRESH_INDEPENDENT_REREVIEW"
            for finding_id in sorted(EXPECTED_MIGRATED_FINDINGS)
        },
        reviewer_roles=sorted(REVIEW_ROLES),
        p0_count=0,
        p1_count=0,
        p2_count=0,
        decision="GO",
        created_stage=10,
        producer="independent_review_panel",
        validator="validate_migrated_finding_review",
    )
    objects["specification_bundle_manifest"] = _synthetic_record(
        graph,
        "specification_bundle_manifest",
        "0" * 64,
        status="READY_FOR_G3",
        eligible_for_g3=True,
        target_commit=target,
        bundle_input_fingerprint_sha256=bundle_inputs_sha,
        bytes=b"synthetic-complete-bundle-manifest",
    )
    objects["aggregate_review"] = _synthetic_record(
        graph,
        "aggregate_review",
        "1" * 64,
        target_bundle_manifest_sha256="0" * 64,
        dependency_node_ids=[
            "component_wire_review",
            "component_scientific_review",
            "component_operations_review",
            "component_conformance_review",
            "component_implementation_review",
            "migrated_finding_review",
            "specification_bundle_manifest",
            "generated_traceability_manifest",
        ],
        decision="GO",
        p0_count=0,
        p1_count=0,
        bytes=b"synthetic-complete-aggregate-review",
    )
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
        f0_decisions_sha256="b" * 64,
    )


def make_repository_context() -> G3AuthorityContext:
    graph = json.loads(AUTHORITY_GRAPH_PATH.read_text())
    target = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    tag_ref = f"refs/tags/{G3_TAG_NAME}"
    tag: dict[str, Any] = {
        "exists": False,
        "annotated": False,
        "object_type": None,
        "peeled_commit": None,
        "message": None,
    }
    try:
        object_type = subprocess.check_output(
            ["git", "cat-file", "-t", tag_ref],
            cwd=ROOT,
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
    except subprocess.CalledProcessError:
        object_type = None
    if object_type is not None:
        tag["exists"] = True
        tag["object_type"] = object_type
        tag["annotated"] = object_type == "tag"
        if tag["annotated"]:
            tag["peeled_commit"] = subprocess.check_output(
                ["git", "rev-parse", f"{tag_ref}^{{commit}}"], cwd=ROOT, text=True
            ).strip()
            raw_tag = subprocess.check_output(
                ["git", "cat-file", "tag", tag_ref], cwd=ROOT
            )
            tag["message"] = raw_tag.split(b"\n\n", 1)[1]
    component_sha256s = [sha256(path) for path in SPECS.values()]
    return G3AuthorityContext(
        mode="real",
        graph=graph,
        objects={},
        bundle_manifest_sha256=sha256(BUNDLE_PATH),
        aggregate_review_sha256="1" * 64,
        expected_target_commit=target,
        tag=tag,
        component_sha256s=component_sha256s,
        component_sha_by_node={
            component_node_id(prefix): sha256(path) for prefix, path in SPECS.items()
        },
        architecture_plan_sha256=sha256(ARCH),
        f0_decisions_sha256="b" * 64,
    )


def run_regression_self_tests() -> None:
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

    graph_reject(
        "unknown graph node",
        lambda value: value["edges"][0].update({"to": "unknown_node"}),
    )
    graph_reject(
        "unknown graph edge type",
        lambda value: value["edges"][0].update({"type": "unknown_edge"}),
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

    synthetic = make_synthetic_context()
    positive = validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, synthetic)
    if positive != G3_EXPECTED_FIELDS:
        raise AssertionError(f"synthetic G3 authority positive result: {positive}")

    def g3_reject(label: str, mutate: object) -> None:
        mutant = deepcopy(synthetic)
        mutate(mutant)
        reject_value_error(
            label, lambda: validate_g3_tag(G3_TAG_NAME, G3_FIXTURE_BODY, mutant)
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
    g3_reject("migrated review stale", lambda context: context.objects["migrated_finding_review"].update({"stale": True}))
    g3_reject("migrated review superseded", lambda context: context.objects["migrated_finding_review"].update({"superseded_by": "new-review"}))
    g3_reject("migrated review non-independent", lambda context: context.objects["migrated_finding_review"].update({"producer": "remediation_agent"}))
    g3_reject("missing aggregate review", lambda context: context.objects.pop("aggregate_review"))
    g3_reject("aggregate review wrong bundle", lambda context: context.objects["aggregate_review"].update({"target_bundle_manifest_sha256": "f" * 64}))
    g3_reject("aggregate review hash mismatch", lambda context: context.objects["aggregate_review"].update({"sha256": "e" * 64}))
    g3_reject("manifest hash mismatch", lambda context: context.objects["specification_bundle_manifest"].update({"sha256": "e" * 64}))
    g3_reject("manifest changed", lambda context: context.objects["specification_bundle_manifest"].update({"content_unchanged": False}))
    g3_reject("wrong G3 target commit", lambda context: context.tag.update({"peeled_commit": "wrong-target"}))
    g3_reject("lightweight G3 tag", lambda context: context.tag.update({"annotated": False, "object_type": "commit"}))
    real_context = make_repository_context()
    real_body = G3_FIXTURE_BODY.replace(
        b"specification_bundle_manifest_sha256=" + b"0" * 64,
        b"specification_bundle_manifest_sha256=" + real_context.bundle_manifest_sha256.encode(),
        1,
    )
    reject_value_error(
        "missing real G3 prerequisites",
        lambda: validate_g3_tag(G3_TAG_NAME, real_body, real_context),
    )
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
        f"traceability_tests={len(R12_TRACE_TEST_IDS)} dag_tests={len(R12_DAG_TEST_IDS)}"
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
