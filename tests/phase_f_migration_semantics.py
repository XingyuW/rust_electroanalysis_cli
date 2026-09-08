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
HISTORICAL_COMPAT = "76561751dcdc40ae56943a0c09a2d6d0ae201e07"
RETAINED_ATTACK = "bada6282aa2638c92f8f16df344991747ea35f33"


class CommittedMigrationSemanticMatrix(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.directory = Path(tempfile.mkdtemp(prefix="phase-f-migration-semantics-"))
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
        cls.original_migration_pin = m.EXPECTED_MIGRATION_LEDGER_GIT_BLOB
        cls.records: list[dict[str, str]] = []
        print(f"Self-contained migration semantic fixtures: {cls.directory}", flush=True)

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
    def replace_cell(cls, text: str, row_id: str, cell_index: int, replacement: str) -> str:
        lines = text.splitlines(keepends=True)
        for index, line in enumerate(lines):
            if line.startswith(f"| {row_id} |"):
                parts = line.rstrip("\n").split("|")
                parts[cell_index + 1] = f" {replacement} "
                lines[index] = "|".join(parts) + ("\n" if line.endswith("\n") else "")
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
        ledger_path = cls.repo / m.MIGRATION_LEDGER.relative_to(m.ROOT)
        neutralized_pin = m._git_blob_bytes(ledger_path.read_bytes())
        cls.git("add", ".")
        cls.git("commit", "-qm", f"TEST_ONLY migration semantic attack: {name}")
        target = cls.git("rev-parse", "HEAD")
        try:
            # Neutralize only the V1 ledger blob pin.  The production grammar,
            # typed semantics, and public historical entrypoint remain active.
            m.EXPECTED_MIGRATION_LEDGER_GIT_BLOB = neutralized_pin
            with self.assertRaises((m.G3ValidationError, ValueError)) as error:
                final(Raw(target, target), cls.repo)
        finally:
            m.EXPECTED_MIGRATION_LEDGER_GIT_BLOB = cls.original_migration_pin
        self.assertNotIn("publication", str(error.exception))
        cls.records.append({"case": name, "sha": target, "rejected": str(error.exception)})

    @classmethod
    def tearDownClass(cls) -> None:
        (cls.directory / "results.json").write_text(json.dumps(cls.records, indent=2) + "\n")

    def test_published_ledgers_pass_pin_neutralized_grammar(self) -> None:
        for target in (PUBLISHED, HISTORICAL_COMPAT):
            with self.subTest(target=target):
                reader = m.GitTargetSourceReader(type(self).repo, target)
                raw = reader.read_bytes(m.MIGRATION_LEDGER)
                old_pin = m.EXPECTED_MIGRATION_LEDGER_GIT_BLOB
                try:
                    m.EXPECTED_MIGRATION_LEDGER_GIT_BLOB = m._git_blob_bytes(raw)
                    m._validate_migration_ledger_semantics(reader)
                finally:
                    m.EXPECTED_MIGRATION_LEDGER_GIT_BLOB = old_pin

    def test_original_retained_attack_equivalent_rejects(self) -> None:
        # The old retained fixture was a machine-local commit.  Recreate its
        # semantic mutation in this test-owned repository instead of depending
        # on that commit object or its former review-cache directory.
        self.commit_attack(
            f"original retained attack {RETAINED_ATTACK}",
            lambda text: self.replace_in_row(
                text,
                "R11-01",
                "Literal plan tag parser plus real-Git property",
                "retained attacker-controlled replacement",
            ),
        )

    def test_all_migration_cells_remain_semantically_bound(self) -> None:
        for row in m.EXPECTED_MIGRATION_ROWS:
            values = (
                row.r11_requirement,
                row.semantic_obligation,
                row.r12_destination,
                row.semantics,
                row.further_closure,
                row.review_status,
            )
            for cell_index in range(len(values)):
                with self.subTest(row=row.r11_requirement, cell=cell_index + 1):
                    self.commit_attack(
                        f"obligation cell {row.r11_requirement}/{cell_index + 1}",
                        lambda text, row_id=row.r11_requirement, index=cell_index: self.replace_cell(
                            text,
                            row_id,
                            index,
                            f"attacker_obligation_{row_id}_{index + 1}",
                        ),
                    )

        for row in m.EXPECTED_FINDING_MIGRATION_ROWS:
            values = (row.finding, row.new_owner_gate, row.status)
            row_id = row.finding
            for cell_index in range(len(values)):
                with self.subTest(row=row_id, cell=cell_index + 1):
                    self.commit_attack(
                        f"finding cell {row_id}/{cell_index + 1}",
                        lambda text, row_id=row_id, index=cell_index: self.replace_cell(
                            text,
                            row_id,
                            index,
                            f"attacker_finding_{index + 1}",
                        ),
                    )

    def test_complete_migration_semantic_attack_matrix(self) -> None:
        r11_01 = self.row("R11-01")
        r11_02 = self.row("R11-02")
        r11_20 = self.row("R11-20")
        r11_08 = self.row("R11-08")
        r11_18 = self.row("R11-18")
        finding_01 = next(
            line for line in self.original[m.MIGRATION_LEDGER].decode().splitlines(keepends=True)
            if line.startswith("| F-PLAN-R11-P1-01 ")
        )
        finding_05 = next(
            line for line in self.original[m.MIGRATION_LEDGER].decode().splitlines(keepends=True)
            if line.startswith("| F-PLAN-R11-P3-01 ")
        )
        original_text = self.original[m.MIGRATION_LEDGER].decode()
        normative_table = original_text.split(
            "## 2. Normative-obligation migration\n", 1
        )[1].split("## 3. R11 finding migration\n", 1)[0].strip("\n")
        finding_table = original_text.split(
            "## 3. R11 finding migration\n", 1
        )[1].split("## 4. Completeness result\n", 1)[0].strip("\n")
        completeness_block = "```text\n" + "\n".join(m.EXPECTED_MIGRATION_COMPLETENESS) + "\n```"

        def add_before(text: str, marker: str, addition: str) -> str:
            return self.replace_once(text, marker, addition + marker)

        attacks = [
            ("missing R11 row", lambda text: self.replace_once(text, self.row("R11-20"), "")),
            ("duplicate R11 row", lambda text: self.replace_once(text, r11_01, r11_01 + r11_01)),
            ("extra R11 row", lambda text: self.replace_once(text, r11_20, r11_20.replace("R11-20", "R11-99", 1))),
            ("out-of-order R11 row", lambda text: self.replace_once(text, r11_01 + r11_02, r11_02 + r11_01)),
            ("unknown R11 ID", lambda text: self.replace_once(text, "| R11-01 |", "| R11-99 |")),
            ("shadow row before header", lambda text: add_before(text, "| R11 requirement |", r11_01)),
            ("shadow row after table", lambda text: add_before(text, "## 3. R11 finding migration\n", r11_01 + "\n")),
            ("appended second normative table", lambda text: text + "\n" + normative_table + "\n"),
            ("appended second finding table", lambda text: text + "\n" + finding_table + "\n"),
            ("duplicate table header", lambda text: self.replace_once(text, "| R11 requirement |", "| R11 requirement |" + "\n| R11 requirement |")),
            ("duplicate separator", lambda text: self.replace_once(text, "|---|---|---|---|---|---|", "|---|---|---|---|---|---|\n|---|---|---|---|---|---|")),
            ("normative table in HTML comment", lambda text: text + "\n<!--\n" + normative_table + "\n-->\n"),
            ("finding table in HTML comment", lambda text: text + "\n<!--\n" + finding_table + "\n-->\n"),
            ("authority row in HTML comment", lambda text: text + "\n<!--\n" + r11_01 + "-->\n"),
            ("normative table in fenced code", lambda text: text + "\n```text\n" + normative_table + "\n```\n"),
            ("finding table in fenced code", lambda text: text + "\n```text\n" + finding_table + "\n```\n"),
            ("hidden duplicate R11 row", lambda text: text + "\n" + r11_01),
            ("hidden duplicate finding row", lambda text: text + "\n" + finding_05),
            ("row outside canonical section", lambda text: text + "\n| R11-99 | shadow | shadow | shadow | shadow | shadow |\n"),
            ("extra leading boundary pipe", lambda text: self.replace_once(text, r11_01, r11_01.replace("| R11-01 |", "|| R11-01 |", 1))),
            ("extra trailing boundary pipe", lambda text: self.replace_once(text, r11_01, r11_01.replace("PENDING |\n", "PENDING ||\n", 1))),
            ("missing leading boundary pipe", lambda text: self.replace_once(text, r11_01, r11_01[1:])),
            ("missing trailing boundary pipe", lambda text: self.replace_once(text, r11_01, r11_01[:-2] + "\n")),
            ("empty first logical cell", lambda text: self.replace_once(text, r11_01, r11_01.replace("| R11-01 |", "|  |", 1))),
            ("empty last logical cell", lambda text: self.replace_once(text, r11_01, r11_01.replace("| PENDING |\n", "|  |\n", 1))),
            ("trailing ASCII space in semantic cell", lambda text: self.replace_once(text, r11_01, r11_01.replace("PENDING |\n", "PENDING  |\n", 1))),
            ("leading ASCII space in semantic cell", lambda text: self.replace_once(text, r11_01, r11_01.replace("| Literal plan", "|  Literal plan", 1))),
            ("tab in semantic cell", lambda text: self.replace_once(text, r11_01, r11_01.replace("PENDING |\n", "PENDING\t|\n", 1))),
            ("NBSP in semantic cell", lambda text: self.replace_once(text, r11_01, r11_01.replace("PENDING |\n", "PENDING\u00a0|\n", 1))),
            ("trailing space in status", lambda text: self.replace_once(text, r11_01, r11_01.replace("PENDING |\n", "PENDING  |\n", 1))),
            ("leading space in status", lambda text: self.replace_once(text, r11_01, r11_01.replace("| PENDING |\n", "|  PENDING |\n", 1))),
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
            ("contradictory governing rule", lambda text: add_before(text, "## 2. Normative-obligation migration\n", "The migration rows may be ignored.\n\n")),
            ("second contradictory rule paragraph", lambda text: text + "\nThe migration rows may be ignored.\n"),
            ("changed source-rule paragraph", lambda text: self.replace_once(text, "Historical R1–R10 prose", "Attacker R1–R10 prose")),
            ("changed finding disclaimer", lambda text: self.replace_once(text, m.MIGRATION_FINDING_DISCLAIMER, "The author disposition is CLOSED.")),
            ("missing completeness", lambda text: self.replace_once(text, "missing=0", "missing=1")),
            ("unowned completeness", lambda text: self.replace_once(text, "unowned=0", "unowned=1")),
            ("changed owner counts", lambda text: self.replace_once(text, "R12_WIRE_OWNED=5", "R12_WIRE_OWNED=4")),
            ("migrated findings missing", lambda text: self.replace_once(text, "migrated_findings_missing=0", "migrated_findings_missing=1")),
            ("literal block inconsistent", lambda text: self.replace_once(text, "R11_CURRENT_NORMATIVE_OBLIGATIONS=20", "R11_CURRENT_NORMATIVE_OBLIGATIONS=19")),
            ("duplicate completeness block", lambda text: self.replace_once(text, completeness_block, completeness_block + "\n" + completeness_block)),
            ("completeness block outside expected section", lambda text: text + "\n" + completeness_block + "\n"),
        ]
        for name, mutation in attacks:
            with self.subTest(name=name):
                self.commit_attack(name, mutation)
        self.assertGreaterEqual(len(self.records), 60)
        self.assertTrue(all(record["rejected"] for record in self.records))


if __name__ == "__main__":
    unittest.main(verbosity=2)
