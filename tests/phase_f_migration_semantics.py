"""Committed, self-consistent migration-ledger semantic attack matrix."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HELPER_PATH = ROOT / "tests" / "phase_f_historical_response_structure.py"
spec = importlib.util.spec_from_file_location("phase_f_hrs_helper", HELPER_PATH)
helper = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = helper
assert spec.loader is not None
spec.loader.exec_module(helper)
m = helper.m
Raw = helper.Raw
final = helper.final

PUBLISHED = "e5deb18990972e6618ac4bc9731766febc0f4173"
RETAINED_ATTACK = "bada6282aa2638c92f8f16df344991747ea35f33"
REVIEW_CACHE = Path.home() / "Library" / "Caches" / "Codex" / "reviews" / "rust_electroanalysis_cli"


class CommittedMigrationSemanticMatrix(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.directory = Path(tempfile.mkdtemp(prefix="phase-f-migration-semantics-", dir=REVIEW_CACHE))
        cls.repo = cls.directory / "repository"
        subprocess.run(
            ["git", "clone", "--no-hardlinks", "--no-checkout", str(ROOT), str(cls.repo)],
            check=True,
            capture_output=True,
            text=True,
        )
        cls.git("checkout", "--detach", PUBLISHED)
        cls.git("config", "user.name", "TEST_ONLY migration semantics")
        cls.git("config", "user.email", "migration-semantics@example.invalid")
        cls.paths = [
            m.BUNDLE_PATH,
            m.TRACE_PATH,
            m.AUTHORITY_GRAPH_PATH,
            m.MIGRATION_LEDGER,
        ]
        cls.original = {
            path: (cls.repo / path.relative_to(m.ROOT)).read_bytes()
            for path in cls.paths
        }
        cls.records: list[dict[str, str]] = []
        print(f"Retained migration semantic fixtures: {cls.directory}", flush=True)

    @classmethod
    def git(cls, *args: str) -> str:
        return subprocess.check_output(
            ["git", "--no-replace-objects", *args],
            cwd=cls.repo,
            stderr=subprocess.PIPE,
            text=True,
        ).strip()

    @classmethod
    def save_json(cls, path: Path, value: object) -> None:
        destination = cls.repo / path.relative_to(m.ROOT)
        destination.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")

    @classmethod
    def refresh_downstream_identities(cls) -> None:
        def fixture_path(path: Path) -> Path:
            if path.is_absolute():
                return cls.repo / path.relative_to(m.ROOT)
            return cls.repo / path

        def digest(path: Path) -> str:
            return m.sha256_bytes(fixture_path(path).read_bytes())

        bundle = json.loads(cls.original[m.BUNDLE_PATH])
        trace = json.loads(cls.original[m.TRACE_PATH])
        graph = json.loads(cls.original[m.AUTHORITY_GRAPH_PATH])
        trace["authority_graph"]["sha256"] = digest(m.AUTHORITY_GRAPH_PATH)
        for node in trace["generated_source_sha256s"]:
            trace["generated_source_sha256s"][node] = digest(
                Path(graph["node_identity_rules"][node]["path"])
            )
        cls.save_json(m.TRACE_PATH, trace)

        input_paths = {
            "architecture_plan": m.ARCH,
            "wire_specification": m.SPECS["F-WIRE"],
            "scientific_specification": m.SPECS["F-SCI"],
            "operations_specification": m.SPECS["F-OPS"],
            "conformance_specification": m.SPECS["F-CNF"],
            "implementation_readiness_specification": m.SPECS["F-IMPL"],
            "migration_ledger": m.MIGRATION_LEDGER,
            "normative_traceability_matrix": m.NORMATIVE_MATRIX_PATH,
            "authority_graph": m.AUTHORITY_GRAPH_PATH,
            "generated_traceability_manifest": m.TRACE_PATH,
        }
        inputs = bundle["bundle_inputs"]
        inputs["source_sha256s"] = {
            name: digest(path) for name, path in input_paths.items()
        }
        inputs["authority_graph_sha256"] = digest(m.AUTHORITY_GRAPH_PATH)
        for node, binding in inputs["authority_bindings"].items():
            rule = graph["node_identity_rules"][node]
            if rule["type"] == "repository_file_sha256":
                binding["sha256"] = digest(Path(rule["path"]))
        for field in (
            "architecture_plan",
            "traceability_manifest",
            "migration_ledger",
            "normative_traceability_matrix",
            "authority_graph",
        ):
            path = m.ROOT / bundle[field]["path"]
            bundle[field]["sha256"] = digest(path)
            if "git_blob" in bundle[field]:
                bundle[field]["git_blob"] = m._git_blob_bytes(fixture_path(path).read_bytes())
        inputs["sha256"] = m.sha256_bytes(
            m.canonical_json_bytes({key: value for key, value in inputs.items() if key != "sha256"})
        )
        bundle["target_revision"]["sha256"] = inputs["sha256"]
        cls.save_json(m.BUNDLE_PATH, bundle)

    @classmethod
    def reset_sources(cls) -> None:
        for path, raw in cls.original.items():
            (cls.repo / path.relative_to(m.ROOT)).write_bytes(raw)

    @classmethod
    def mutate_ledger(cls, mutation) -> None:
        path = cls.repo / m.MIGRATION_LEDGER.relative_to(m.ROOT)
        path.write_text(mutation(path.read_text()))

    @classmethod
    def replace_once(cls, text: str, old: str, new: str) -> str:
        if old not in text:
            raise AssertionError(f"fixture mutation anchor missing: {old[:100]}")
        return text.replace(old, new, 1)

    @classmethod
    def replace_in_row(cls, text: str, row_id: str, old: str, new: str) -> str:
        lines = text.splitlines(keepends=True)
        for index, line in enumerate(lines):
            if line.startswith(f"| {row_id}"):
                lines[index] = cls.replace_once(line, old, new)
                return "".join(lines)
        raise AssertionError(f"fixture row missing: {row_id}")

    @classmethod
    def row(cls, row_id: str) -> str:
        text = cls.original[m.MIGRATION_LEDGER].decode()
        return next(line for line in text.splitlines(keepends=True) if line.startswith(f"| {row_id} |"))

    def commit_attack(self, name: str, mutation) -> None:
        cls = type(self)
        cls.reset_sources()
        cls.mutate_ledger(mutation)
        cls.refresh_downstream_identities()
        cls.git("add", ".")
        cls.git("commit", "-qm", f"TEST_ONLY migration semantic attack: {name}")
        target = cls.git("rev-parse", "HEAD")
        with self.assertRaises((m.G3ValidationError, ValueError)) as error:
            final(Raw(target, target), cls.repo)
        self.assertNotIn("publication", str(error.exception))
        cls.records.append({"case": name, "sha": target, "rejected": str(error.exception)})

    @classmethod
    def tearDownClass(cls) -> None:
        (cls.directory / "results.json").write_text(json.dumps(cls.records, indent=2) + "\n")

    def test_retained_reviewer_fixture_rejects(self) -> None:
        with self.assertRaises(m.G3ValidationError):
            final(Raw(RETAINED_ATTACK, RETAINED_ATTACK),
                  "/Users/xingyuwang/Library/Caches/Codex/reviews/rust_electroanalysis_cli/phase-f-hrs-structures-7094c08r/repository")

    def test_complete_migration_semantic_attack_matrix(self) -> None:
        r11_01 = self.row("R11-01")
        r11_02 = self.row("R11-02")
        r11_08 = self.row("R11-08")
        r11_18 = self.row("R11-18")
        finding_01 = next(
            line for line in self.original[m.MIGRATION_LEDGER].decode().splitlines(keepends=True)
            if line.startswith("| F-PLAN-R11-P1-01 ")
        )
        attacks = [
            ("missing R11 row", lambda text: self.replace_once(text, self.row("R11-20"), "")),
            ("duplicate R11 row", lambda text: self.replace_once(text, r11_01, r11_01 + r11_01)),
            ("extra R11 row", lambda text: self.replace_once(text, r11_20 := self.row("R11-20"), r11_20.replace("R11-20", "R11-99", 1))),
            ("out-of-order R11 row", lambda text: self.replace_once(text, r11_01 + r11_02, r11_02 + r11_01)),
            ("unknown R11 ID", lambda text: self.replace_once(text, "| R11-01 |", "| R11-99 |")),
            ("attacker obligation", lambda text: self.replace_in_row(text, "R11-01", "Literal plan tag parser plus real-Git property", "attacker-controlled replacement text")),
            ("empty obligation", lambda text: self.replace_in_row(text, "R11-01", "Literal plan tag parser plus real-Git property", "")),
            ("copied obligation", lambda text: self.replace_in_row(text, "R11-01", "Literal plan tag parser plus real-Git property", "Literal trust tag/parser/binding")),
            ("wrong valid destination", lambda text: self.replace_in_row(text, "R11-01", "F-CNF-004", "F-CNF-005")),
            ("changed destination document", lambda text: self.replace_in_row(text, "R11-01", "Conformance `F-CNF-004`", "Wire `F-WIRE-005`")),
            ("removed multi-destination binding", lambda text: self.replace_in_row(text, "R11-02", "; Conformance `F-CNF-004`", "")),
            ("unauthorized destination", lambda text: self.replace_in_row(text, "R11-01", "Conformance `F-CNF-004`", "Conformance `F-CNF-004`; Operations `F-OPS-001`")),
            ("nonexistent destination", lambda text: self.replace_in_row(text, "R11-01", "F-CNF-004", "F-CNF-999")),
            ("deleted semantics", lambda text: self.replace_in_row(text, "R11-01", "unchanged/refined", "DELETED")),
            ("waived semantics", lambda text: self.replace_in_row(text, "R11-01", "unchanged/refined", "WAIVED")),
            ("approved semantics", lambda text: self.replace_in_row(text, "R11-01", "unchanged/refined", "APPROVED")),
            ("copied semantics", lambda text: self.replace_in_row(text, "R11-01", "unchanged/refined", "unchanged")),
            ("empty semantics", lambda text: self.replace_in_row(text, "R11-01", "unchanged/refined", "")),
            ("R11-08 closure none", lambda text: self.replace_in_row(text, "R11-08", m.EXPECTED_MIGRATION_ROWS[7].further_closure, "none")),
            ("R11-08 closure truncated", lambda text: self.replace_once(text, "authoritative typed R12 graph with exact", "authoritative typed R12 graph")),
            ("R11-18 schema closure", lambda text: self.replace_once(text, "R11 remains 91", "R11 remains 90")),
            ("required closure phrase removed", lambda text: self.replace_once(text, "root SHA-256 binding", "root binding")),
            ("attacker closure", lambda text: self.replace_in_row(text, "R11-02", "none", "attacker closure")),
            ("approved review status", lambda text: self.replace_in_row(text, "R11-01", "PENDING", "APPROVED")),
            ("closed review status", lambda text: self.replace_in_row(text, "R11-01", "PENDING", "CLOSED")),
            ("GO review status", lambda text: self.replace_in_row(text, "R11-01", "PENDING", "GO")),
            ("accepted without review status", lambda text: self.replace_in_row(text, "R11-01", "PENDING", "ACCEPTED_WITHOUT_REVIEW")),
            ("empty review status", lambda text: self.replace_in_row(text, "R11-01", "PENDING", "")),
            ("missing finding", lambda text: self.replace_once(text, finding_01, "")),
            ("duplicate finding", lambda text: self.replace_once(text, finding_01, finding_01 + finding_01)),
            ("changed finding owner", lambda text: self.replace_in_row(text, "F-PLAN-R11-P1-01", "`F-CNF-001,F-CNF-002`, G2 Conformance", "`F-CNF-003`, G2 Conformance")),
            ("closed finding status", lambda text: self.replace_in_row(text, "F-PLAN-R11-P1-01", "OPEN pending independent review", "CLOSED")),
            ("accepted finding status", lambda text: self.replace_in_row(text, "F-PLAN-R11-P1-01", "OPEN pending independent review", "ACCEPTED_WITHOUT_REVIEW")),
            ("wrong R11 SHA", lambda text: self.replace_once(text, m.EXPECTED_R11_SHA256, "0" * 64)),
            ("wrong R11 blob", lambda text: self.replace_once(text, m.EXPECTED_R11_GIT_BLOB, "0" * 40)),
            ("changed lossless rule", lambda text: self.replace_once(text, "single current R11 requirement set", "attacker-controlled current R11 requirement set")),
            ("missing completeness", lambda text: self.replace_once(text, "missing=0", "missing=1")),
            ("unowned completeness", lambda text: self.replace_once(text, "unowned=0", "unowned=1")),
            ("changed owner counts", lambda text: self.replace_once(text, "R12_WIRE_OWNED=5", "R12_WIRE_OWNED=4")),
            ("migrated findings missing", lambda text: self.replace_once(text, "migrated_findings_missing=0", "migrated_findings_missing=1")),
            ("literal block inconsistent", lambda text: self.replace_once(text, "R11_CURRENT_NORMATIVE_OBLIGATIONS=20", "R11_CURRENT_NORMATIVE_OBLIGATIONS=19")),
        ]
        for name, mutation in attacks:
            with self.subTest(name=name):
                self.commit_attack(name, mutation)
        self.assertGreaterEqual(len(self.records), 40)
        self.assertTrue(all(record["rejected"] for record in self.records))


if __name__ == "__main__":
    unittest.main(verbosity=2)
