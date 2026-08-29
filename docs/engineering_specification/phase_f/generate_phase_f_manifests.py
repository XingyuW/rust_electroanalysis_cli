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
from pathlib import Path


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

REQUIRED_FILENAMES = {
    *(path.name for path in SPECS.values()),
    R11_SOURCE.name,
    MIGRATION_LEDGER.name,
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


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def git_blob(path: Path) -> str:
    return subprocess.check_output(
        ["git", "hash-object", str(path)], cwd=ROOT, text=True
    ).strip()


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
                "test_ids": ["R11-DAG-AUDIT", "R11-TRACE", "R11-CX-08"],
                "future_real_evidence_ids": [],
            }
        )
    actual_ids = [entry["requirement_id"] for entry in entries]
    if actual_ids != EXPECTED_ARCHITECTURE_IDS:
        raise ValueError(f"architecture requirement set mismatch: {actual_ids}")
    return entries


def verification(prefix: str, number: int) -> tuple[list[str], list[str]]:
    if prefix == "F-WIRE":
        tests = ["R11-CAT", "R11-TRACE", "R11-CX-10", "R11-CX-11", "R11-CX-12", "R11-CX-13"]
        if number in (3, 7):
            tests.extend(["R11-POS-PLAN", "R11-POS-TRUST", "R12-POS-SPEC-BUNDLE-TAG"])
        return tests, []
    if prefix == "F-SCI":
        evidence = {
            1: ["EV11-17", "EV11-19"], 2: ["EV11-17"],
            3: ["EV11-17"], 4: ["EV11-17"], 5: ["EV11-17"],
            6: ["EV11-17"], 7: ["EV11-17"], 8: ["EV11-05", "EV11-06", "EV11-17"],
            9: ["EV11-14", "EV11-19"], 10: ["EV11-01", "EV11-02", "EV11-03", "EV11-04", "EV11-05", "EV11-06", "EV11-07", "EV11-08", "EV11-17", "EV11-19", "EV11-20"],
        }[number]
        return ["R11-TRACE", "R11-CX-08", "R11-CX-18"], evidence
    if prefix == "F-OPS":
        tests = {
            1: ["R11-DAG-AUDIT", "R11-CX-08"],
            2: ["R11-DAG-AUDIT", "R11-CX-08"],
            3: ["R11-PROP-MONITORING", "R11-CX-18"],
            4: ["R11-KAT-INCIDENT", "R11-CX-03", "R11-CX-04"],
            5: ["R11-DAG-AUDIT", "R11-CX-08"],
            6: ["R11-KAT-RETENTION-COPY", "R11-CX-14", "R11-CX-15"],
            7: ["R11-KAT-RETENTION-COPY", "R11-CX-01", "R11-CX-02"],
            8: ["R11-DAG-AUDIT", "R11-CX-08", "R11-CX-18"],
        }[number]
        return tests, [f"EV11-{n:02d}" for n in ({1: 17, 2: 17, 3: 7, 4: 4, 5: 17, 6: 5, 7: 3, 8: 17}[number],)]
    if prefix == "F-CNF":
        tests = {
            1: ["R11-KAT-INCIDENT", "R11-POS-PLAN", "R11-POS-TRUST", "R11-CX-04"],
            2: ["R11-KAT-RETENTION-COPY", "R11-CX-01", "R11-CX-02", "R11-CX-14", "R11-CX-15"],
            3: ["R11-PROP-MONITORING", "R11-CX-06", "R11-CX-07", "R11-CX-18"],
            4: ["R11-POS-PLAN", "R11-POS-TRUST", "R12-POS-SPEC-BUNDLE-TAG"],
            5: ["R11-DAG-AUDIT", "R11-CAT", "R11-TRACE", "R11-CX-08", "R11-CX-10", "R11-CX-11", "R11-CX-12", "R11-CX-13", "R11-CX-19", "R11-CX-20"],
            6: ["R11-CX-05", "R11-CX-06", "R11-CX-07"],
            7: ["R11-CX-01", "R11-CX-02", "R11-CX-04"],
            8: ["R11-CX-08", "R11-CX-18"],
        }[number]
        return tests, []
    if prefix == "F-IMPL":
        return ["R11-DAG-AUDIT", "R11-TRACE", "R11-CX-08", "R11-CX-18"], ["EV11-08", "EV11-17"]
    raise AssertionError(prefix)


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
        number = int(id_match.group(2))
        refs = expand_refs(cells[1])
        tests, evidence = verification(prefix, number)
        entries.append(
            {
                "requirement_id": requirement_id,
                "authority_document": str(path.relative_to(ROOT)),
                "authority_anchor": f"#{anchor_match.group(1)}",
                "upstream_requirement_ids": [r for r in refs if not r.startswith("F-OD-")],
                "f0_decision_dependencies": [r for r in refs if r.startswith("F-OD-")],
                "downstream_child_requirements": [],
                "verification_gate": "G2",
                "test_ids": tests,
                "future_real_evidence_ids": evidence,
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


def build_traceability() -> dict[str, object]:
    validate_inventory()
    validate_r11_and_migration()
    validate_f0_decisions()
    entries = parse_architecture()
    for prefix, path in SPECS.items():
        entries.extend(parse_spec(prefix, path))
    validate_traceability(entries)
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
        "requirements": sorted(entries, key=lambda row: row["requirement_id"]),
    }


def build_bundle(trace_sha: str) -> dict[str, object]:
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
        "component_specifications": components,
        "traceability_manifest": {
            "path": str(TRACE_PATH.relative_to(ROOT)),
            "sha256": trace_sha,
        },
        "migration_ledger": {
            "path": str(MIGRATION_LEDGER.relative_to(ROOT)),
            "sha256": sha256(MIGRATION_LEDGER),
        },
        "aggregate_specification_bundle_review_sha256": None,
        "approval_decision": "NO-GO",
        "blocking_reasons": [
            "architecture_plan_tag_absent",
            "f0_decisions_tag_absent",
            "component_independent_reviews_pending",
            "aggregate_specification_bundle_review_absent",
        ],
    }


def main() -> None:
    trace = build_traceability()
    trace_bytes = (json.dumps(trace, indent=2, sort_keys=True) + "\n").encode()
    bundle = build_bundle(sha256_bytes(trace_bytes))
    bundle_bytes = (json.dumps(bundle, indent=2, sort_keys=True) + "\n").encode()
    TRACE_PATH.write_bytes(trace_bytes)
    BUNDLE_PATH.write_bytes(bundle_bytes)


if __name__ == "__main__":
    main()
