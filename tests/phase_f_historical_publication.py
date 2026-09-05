"""Production-path historical publication regressions; no network or REAL authority."""
import importlib.util
import json
import sys
import unittest
from copy import deepcopy
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("phase_f", ROOT / "docs/engineering_specification/phase_f/generate_phase_f_manifests.py")
m = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = m
SPEC.loader.exec_module(m)
PUBLISHED = "e5deb18990972e6618ac4bc9731766febc0f4173"
ADVANCED = "a" * 40


class CanonicalResponses(m.GitHubApiTransport):
    """Raw fixed API responses exercise the production HTTP response parsers."""
    def __init__(self, target, head, *, exists=True):
        super().__init__(credential_provider=lambda: "TEST_ONLY")
        self.target, self.head, self.exists = target, head, exists
        self.calls = []
        self.end_head = head
        self.reads = 0
        self.commit = {"sha": target, "tree": {"sha": "b" * 40}, "parents": []}
        self.comparison = {"base_commit": {"sha": target}, "merge_base_commit": {"sha": target},
                           "status": "identical" if target == head else "ahead", "behind_by": 0,
                           "ahead_by": 0 if target == head else 1, "total_commits": 0 if target == head else 1}
        self.identity = {"id": 1273879958, "name": "rust_electroanalysis_cli",
                         "full_name": "XingyuW/rust_electroanalysis_cli", "owner": {"login": "XingyuW"}}

    def _get_json_response(self, path):
        self.calls.append(path)
        base = "/repos/XingyuW/rust_electroanalysis_cli"
        if path == base:
            return deepcopy(self.identity), None
        if path == base + "/git/ref/heads/main":
            self.reads += 1
            return {"ref": "refs/heads/main", "object": {"type": "commit", "sha": self.head if self.reads == 1 else self.end_head}}, None
        if path == base + "/git/commits/" + self.target:
            if not self.exists:
                raise m.G3ValidationError("github_protection_resource_missing")
            return deepcopy(self.commit), None
        if path == base + "/compare/" + self.target + "..." + self.head:
            return deepcopy(self.comparison), None
        raise AssertionError("Unexpected canonical API path: " + path)


class HistoricalPublication(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary, cls.repository, cls.published, cls.body = m._isolated_real_fixture()
        # The task requires preservation of local-only Git history. Retain fixtures.
        cls.temporary._finalizer.detach()
        m._fixture_git(cls.repository, ["commit", "--allow-empty", "-qm", "local-only target"])
        cls.local = m._fixture_git(cls.repository, ["rev-parse", "HEAD"]).decode().strip()
        cls.graph = m._fixture_graph(cls.repository)
        print("Retained TEST_ONLY fixture:", cls.temporary.name, flush=True)

    def resolve(self, transport, target=None, purpose=m.ResolutionPurpose.HISTORICAL_VALIDATION):
        return m._resolve_external_dependency_with_transport(self.repository, self.graph, "published_normative_target", transport,
                                              target_commit=target or transport.target, resolution_purpose=purpose)

    def reject(self, transport):
        with self.assertRaises(m.G3ValidationError):
            self.resolve(transport)

    def test_H1_H2_H18_valid_binding_and_current_equality(self):
        for target, head in [(self.published, self.published), (self.published, ADVANCED)]:
            t = CanonicalResponses(target, head)
            binding = self.resolve(t)
            self.assertEqual(binding.purpose, m.ResolutionPurpose.HISTORICAL_VALIDATION)
            self.assertTrue(any("/git/commits/" in path for path in t.calls))
            self.assertTrue(any("/compare/" in path for path in t.calls))
            self.assertEqual(t.reads, 2)
        binding = self.resolve(CanonicalResponses(self.published, self.published), purpose=m.ResolutionPurpose.CURRENT_AUTHORIZATION)
        self.assertEqual(binding.purpose, m.ResolutionPurpose.CURRENT_AUTHORIZATION)

    def test_H3_H9_missing_commit(self):
        self.reject(CanonicalResponses(self.local, self.published, exists=False))

    def test_H4_H10_H11_H12_structural_and_relationship_mutations(self):
        mutations = [None, [], {}, {"status": "diverged", "behind_by": 1}, {"status": "behind", "behind_by": 1},
                     {"base_commit": {"sha": ADVANCED}}, {"merge_base_commit": {"sha": ADVANCED}},
                     {"behind_by": True}, {"ahead_by": "1"}, {"total_commits": -1}, {"total_commits": 2},
                     {"status": "identical"}, {"status": "unknown"}, {"base_commit": []}, {"merge_base_commit": None}]
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                t = CanonicalResponses(self.local, self.published)
                if isinstance(mutation, dict) and mutation:
                    t.comparison.update(mutation)
                else:
                    t.comparison = mutation
                self.reject(t)
        for mutation in [None, [], {}, {"sha": ADVANCED}, {"tree": []}, {"parents": [None]}]:
            with self.subTest(commit=mutation):
                t = CanonicalResponses(self.published, self.published)
                if isinstance(mutation, dict) and mutation:
                    t.commit.update(mutation)
                else:
                    t.commit = mutation
                self.reject(t)

    def test_H5_H6_H7_H8_local_mutation_invariance(self):
        repo = self.repository
        def check():
            checkpoint = m._reviewer_bootstrap_checkpoint_path(repo)
            before = checkpoint.read_bytes() if checkpoint.exists() else None
            responses = CanonicalResponses(self.local, self.published, exists=False)
            t = m._fixture_github_api_transport(repo, self.graph)
            t.get_commit = responses.get_commit
            t.compare_commits = responses.compare_commits
            ctx = m._make_repository_context_with_transport(repo, self.local, True, m.ResolutionPurpose.HISTORICAL_VALIDATION, t)
            self.assertIsNone(ctx.publication_binding)
            self.assertEqual(ctx.publication_error_category, "historical_target_not_published")
            self.assertEqual(checkpoint.read_bytes() if checkpoint.exists() else None, before)
        check()
        for parents in [(self.local,), (self.local, ADVANCED)]:
            # The second replacement has a different valid parent object.
            args = ["replace", "--graft", self.published, parents[0]]
            if len(parents) == 2:
                extra = m._fixture_git(repo, ["commit-tree", m._fixture_git(repo, ["rev-parse", "HEAD^{tree}"]).decode().strip(), "-m", "injected object"]).decode().strip()
                args.append(extra)
            m._fixture_git(repo, args)
            try:
                check()
            finally:
                m._fixture_git(repo, ["replace", "-d", self.published])
        grafts = repo / ".git/info/grafts"
        grafts.write_text(self.published + " " + self.local + "\n")
        try:
            check()
        finally:
            grafts.unlink()
        m._fixture_git(repo, ["update-ref", "refs/remotes/origin/main", self.local])
        m._fixture_git(repo, ["config", "remote.origin.url", "ssh://forged.invalid/irrelevant"])
        m._fixture_git(repo, ["config", "core.sshCommand", "false"])
        check()

    def test_H13_main_race(self):
        t = CanonicalResponses(self.published, self.published)
        t.end_head = ADVANCED
        self.reject(t)

    def test_H14_H15_H16_normative_compatibility_and_non_authority(self):
        for target, head in [(PUBLISHED, PUBLISHED), (PUBLISHED, ADVANCED),
                             ("76561751dcdc40ae56943a0c09a2d6d0ae201e07", PUBLISHED)]:
            with self.subTest(target=target, head=head):
                t = CanonicalResponses(target, head)
                ctx = m._make_repository_context_with_transport(ROOT, target, False, m.ResolutionPurpose.HISTORICAL_VALIDATION, t)
                result = m._validate_historical_normative_context(ROOT, ctx)
                self.assertEqual(result, {"historical_publication_valid": True, "historical_normative_structure_valid": True, "current_operational_authority": False})
                with self.assertRaisesRegex(m.G3ValidationError, "historical_validation_cannot_authorize_current"):
                    m.validate_g3_tag(m.G3_TAG_NAME, self.body, ctx)
                ctx.resolution_purpose = m.ResolutionPurpose.CURRENT_AUTHORIZATION
                with self.assertRaisesRegex(m.G3ValidationError, "publication_binding_mismatch"):
                    m.validate_g3_tag(m.G3_TAG_NAME, self.body, ctx)

    def test_final_historical_validation_rechecks_main(self):
        t = CanonicalResponses(PUBLISHED, PUBLISHED)
        ctx = m._make_repository_context_with_transport(ROOT, PUBLISHED, False, m.ResolutionPurpose.HISTORICAL_VALIDATION, t)
        t.end_head = ADVANCED
        with self.assertRaisesRegex(m.G3ValidationError, "publication_head_changed_during_resolution"):
            m._validate_historical_normative_context(ROOT, ctx)

    def test_committed_invalid_graph_rejects_at_production_entrypoint(self):
        graph_path = self.repository / m.AUTHORITY_GRAPH_PATH.relative_to(m.ROOT)
        original = graph_path.read_bytes()
        invalid = json.loads(original)
        invalid["edges"].pop()
        graph_path.write_text(json.dumps(invalid))
        m._fixture_git(self.repository, ["add", str(graph_path)])
        m._fixture_git(self.repository, ["commit", "-qm", "TEST_ONLY structurally invalid historical graph"])
        target = m._fixture_git(self.repository, ["rev-parse", "HEAD"]).decode().strip()
        try:
            with self.assertRaises(ValueError):
                m._make_repository_context_with_transport(self.repository, target, True,
                    m.ResolutionPurpose.HISTORICAL_VALIDATION, CanonicalResponses(target, target))
        finally:
            graph_path.write_bytes(original)
            m._fixture_git(self.repository, ["add", str(graph_path)])
            m._fixture_git(self.repository, ["commit", "-qm", "Restore valid TEST_ONLY graph, preserving attack history"])

    def test_H17_current_unpublished_rejects(self):
        with self.assertRaisesRegex(m.G3ValidationError, "publication_target_mismatch"):
            self.resolve(CanonicalResponses(self.local, self.published), purpose=m.ResolutionPurpose.CURRENT_AUTHORIZATION)

    def test_registry_and_transport_omissions(self):
        hook = m.ExternalResolverHook.RESOLVE_CANONICAL_PUBLICATION
        for replacement in [None, (m.ExternalDiscovery.REPOSITORY_COMMIT, lambda *args: self.published),
                            (m.ExternalDiscovery.GITHUB_REF, lambda *args: self.published)]:
            with patch.dict(m.EXTERNAL_RESOLVER_DISPATCH, {hook: replacement}):
                self.reject(CanonicalResponses(self.published, self.published))
        for method in ["get_commit", "compare_commits", "get_ref"]:
            t = CanonicalResponses(self.published, self.published)
            setattr(t, method, None)
            self.reject(t)
        t = CanonicalResponses(self.published, self.published)
        t.identity["id"] += 1
        self.reject(t)

    def test_public_real_dispatch_cannot_accept_fixture_transport(self):
        with self.assertRaises(TypeError):
            m.resolve_external_dependency(self.repository, self.graph, "published_normative_target",
                github_api_transport=CanonicalResponses(self.published, self.published), target_commit=self.published)
        with self.assertRaises(TypeError):
            m.make_repository_context(self.repository, self.published,
                github_api_transport=CanonicalResponses(self.published, self.published))

    def test_transport_duplicate_json_and_path_injection_reject(self):
        class Response:
            headers = {}
            def __enter__(self):
                return self
            def __exit__(self, *args):
                pass
            def read(self):
                return b'{"sha":"wrong","sha":"also-wrong"}'
        transport = m.GitHubApiTransport(credential_provider=lambda: "TEST_ONLY")
        with patch.object(m, "urlopen", return_value=Response()):
            with self.assertRaises(m.G3ValidationError):
                transport.get_commit(m.CANONICAL_GITHUB_REPOSITORY_IDENTITY, PUBLISHED)
        for target in ["main", "../main", "https://attacker.invalid", "A" * 40, "a" * 39]:
            with self.assertRaises(m.G3ValidationError):
                transport.get_commit(m.CANONICAL_GITHUB_REPOSITORY_IDENTITY, target)
        with self.assertRaises(m.G3ValidationError):
            transport.compare_commits({"repository_full_name": "attacker/repo"}, PUBLISHED, PUBLISHED)

    def test_legacy_contract_is_audited_without_rewriting_or_current_acceptance(self):
        graph = json.loads(m._git_output(ROOT, ["show", PUBLISHED + ":docs/engineering_specification/phase_f/phase_f_r12_authority_graph.json"]))
        original = deepcopy(graph)
        m.validate_r12_authority_graph(graph, m.ResolutionPurpose.HISTORICAL_VALIDATION)
        self.assertEqual(graph, original)
        with self.assertRaises(ValueError):
            m.validate_r12_authority_graph(graph, m.ResolutionPurpose.CURRENT_AUTHORIZATION)

    def test_malformed_historical_graph_rejects(self):
        t = CanonicalResponses(PUBLISHED, PUBLISHED)
        ctx = m._make_repository_context_with_transport(ROOT, PUBLISHED, False, m.ResolutionPurpose.HISTORICAL_VALIDATION, t)
        ctx.graph["edges"].pop()
        with self.assertRaises(ValueError):
            m._validate_historical_normative_context(ROOT, ctx)

    def test_malformed_historical_bundle_rejects(self):
        t = CanonicalResponses(PUBLISHED, PUBLISHED)
        ctx = m._make_repository_context_with_transport(ROOT, PUBLISHED, False, m.ResolutionPurpose.HISTORICAL_VALIDATION, t)
        original = m._git_output
        def corrupt(repository, args):
            raw = original(repository, args)
            if args[0] == "show" and args[1].endswith("phase_f_specification_bundle_manifest.json"):
                bundle = json.loads(raw)
                bundle["bundle_inputs"]["source_sha256s"]["architecture_plan"] = "0" * 64
                return json.dumps(bundle).encode()
            return raw
        with patch.object(m, "_git_output", side_effect=corrupt):
            with self.assertRaises(m.G3ValidationError):
                m._validate_historical_normative_context(ROOT, ctx)


if __name__ == "__main__":
    unittest.main(verbosity=2)
