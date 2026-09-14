"""Target-relative historical bootstrap currentness regressions.

The fixture uses deterministic TEST_ONLY key material while preserving the
production REAL wire format and production repository resolver path.
"""

import importlib.util
import json
import sys
import unittest
from pathlib import Path
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "phase_f_historical_bootstrap_currentness",
    ROOT / "docs/engineering_specification/phase_f/generate_phase_f_manifests.py",
)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("unable to load Phase F production module")
m = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = m
SPEC.loader.exec_module(m)


def _resign_fixture_proof(proof, signing_seed, valid_from, valid_until):
    """Rebuild one deterministic fixture proof after changing its window."""

    content_unchanged = proof.pop("content_unchanged", True)
    for key in ("bytes", "canonical_object", "complete_file_sha256"):
        proof.pop(key, None)
    proof["valid_from"] = valid_from
    proof["valid_until"] = valid_until
    proof["head_id"] = m.reviewer_bootstrap_currentness_head_id(proof)
    proof["currentness_proof_id"] = m.reviewer_bootstrap_currentness_proof_id(proof)
    signing_payload = {
        key: proof[key]
        for key in m.REVIEWER_BOOTSTRAP_CURRENTNESS_FIELDS
        if key != "signature"
    }
    proof["signature"] = m._fixture_ed25519_sign(
        signing_seed,
        m.REVIEWER_BOOTSTRAP_CURRENTNESS_DOMAIN
        + m.canonical_jcs_bytes(signing_payload),
    )
    raw = m.canonical_json_bytes(
        {key: proof[key] for key in m.REVIEWER_BOOTSTRAP_CURRENTNESS_FIELDS}
    )
    proof.update(
        {
            "canonical_object": json.loads(raw),
            "bytes": raw,
            "complete_file_sha256": m.sha256_bytes(raw),
            "content_unchanged": content_unchanged,
        }
    )
    return proof


class HistoricalBootstrapCurrentness(unittest.TestCase):
    def test_target_relative_legacy_policy_and_current_rejection(self):
        temporary, repository, current_target, fixture_body = m._isolated_real_fixture(
            authority_class="REAL"
        )
        self.addCleanup(temporary.cleanup)

        graph = m._fixture_graph(repository)
        transport = m._fixture_github_api_transport(repository, graph)
        initial = m._make_repository_context_with_transport(
            repository,
            current_target,
            github_api_transport=transport,
        )
        self.assertFalse(initial.resolution["errors"])
        root = initial.reviewer_bootstrap_root
        proof_0 = initial.reviewer_bootstrap_currentness
        if root is None or proof_0 is None:
            self.fail("REAL-format fixture did not resolve bootstrap genesis")

        proof_1 = m._fixture_bootstrap_proof(
            proof_0,
            root,
            b"phase-f-r12-fixture-bootstrap-root-01",
            1,
        )
        _resign_fixture_proof(
            proof_1,
            b"phase-f-r12-fixture-bootstrap-root-01",
            "2020-01-01T00:00:00Z",
            "2020-01-08T00:00:01Z",
        )
        m._fixture_write_bootstrap_record(
            repository, proof_1, "currentness_proofs"
        )
        m._fixture_advance_external_head(repository, graph, root, proof_1)

        with self.assertRaisesRegex(
            m.G3ValidationError,
            "real_reviewer_bootstrap_currentness_lifetime_exceeded",
        ):
            m._validate_reviewer_bootstrap_proof_object(
                initial,
                proof_1,
                initial.reviewer_bootstrap_root_history,
                {
                    proof_0["currentness_proof_id"]: proof_0,
                    proof_1["currentness_proof_id"]: proof_1,
                },
                require_current_window=True,
                _test_only_validation_time=m.TEST_ONLY_VALIDATION_TIME,
            )

        current_rejected = m._make_repository_context_with_transport(
            repository,
            current_target,
            github_api_transport=m._fixture_github_api_transport(
                repository, m._fixture_graph(repository)
            ),
        )
        self.assertTrue(current_rejected.resolution["errors"])

        graph_path = repository / m.AUTHORITY_GRAPH_PATH.relative_to(m.ROOT)
        graph = json.loads(graph_path.read_text())
        graph["reviewer_bootstrap_trust_contract"][
            "currentness_window_policy"
        ] = "valid_from_le_validation_time_le_valid_until"
        graph_path.write_bytes(m.canonical_json_bytes(graph))
        m._fixture_git(
            repository,
            ["add", str(graph_path.relative_to(repository))],
        )
        m._fixture_git(repository, ["commit", "-qm", "TEST_ONLY legacy policy target"])
        legacy_target_1 = m._fixture_git(
            repository, ["rev-parse", "HEAD"]
        ).decode().strip()
        m._fixture_git(
            repository,
            ["commit", "--allow-empty", "-qm", "TEST_ONLY second legacy target"],
        )
        legacy_target_2 = m._fixture_git(
            repository, ["rev-parse", "HEAD"]
        ).decode().strip()
        self.assertNotEqual(legacy_target_1, legacy_target_2)
        publication_remote = Path(
            graph["external_monotonic_head_contract"]["transport"]["remote_url"]
        )
        m._fixture_git(
            repository,
            [
                "push",
                "-q",
                str(publication_remote),
                f"{legacy_target_2}:{m.CANONICAL_PUBLICATION_REF}",
            ],
        )
        for tag_name in (
            m.G3_EXPECTED_FIELDS["phase_f_architecture_plan_tag"],
            m.G3_EXPECTED_FIELDS["phase_f_f0_decisions_tag"],
        ):
            message = m._read_git_tag(repository, tag_name)["message"]
            if not isinstance(message, bytes):
                self.fail(f"fixture tag message missing: {tag_name}")
            m._fixture_annotated_tag(
                repository,
                tag_name,
                legacy_target_2,
                message.replace(current_target.encode(), legacy_target_2.encode()),
            )
        m._fixture_annotated_tag(
            repository,
            m.G3_TAG_NAME,
            legacy_target_2,
            fixture_body.replace(current_target.encode(), legacy_target_2.encode()),
        )

        observed_purposes = []
        original_contract = m._reviewer_bootstrap_trust_contract

        def observe_contract(target_graph, purpose):
            observed_purposes.append(purpose)
            return original_contract(target_graph, purpose)

        with patch.object(
            m,
            "_reviewer_bootstrap_trust_contract",
            side_effect=observe_contract,
        ):
            historical = m._make_repository_context_with_transport(
                repository,
                legacy_target_2,
                resolution_purpose=m.ResolutionPurpose.HISTORICAL_VALIDATION,
                github_api_transport=m._fixture_github_api_transport(
                    repository, graph
                ),
            )

        self.assertFalse(historical.resolution["errors"])
        self.assertEqual(
            historical.reviewer_bootstrap_currentness["sequence"], 1
        )
        self.assertTrue(observed_purposes)
        self.assertTrue(
            all(
                purpose == m.ResolutionPurpose.HISTORICAL_VALIDATION
                for purpose in observed_purposes
            )
        )
        row = historical.objects["migrated_finding_review"]["review_records"][0]
        self.assertEqual(
            m.validate_historical_review_artifact(historical, row),
            {
                "historical_cryptographic_validity": True,
                "currently_authorized": False,
            },
        )
        self.assertEqual(
            m._reviewer_bootstrap_trust_contract(
                historical.graph,
                m.ResolutionPurpose.HISTORICAL_VALIDATION,
            )["currentness_window_policy"],
            "valid_from_le_validation_time_le_valid_until",
        )
        historical.resolution_purpose = m.ResolutionPurpose.CURRENT_AUTHORIZATION
        with self.assertRaisesRegex(
            m.G3ValidationError, "historical_validation_purpose_required"
        ):
            m.validate_historical_review_artifact(historical, row)


if __name__ == "__main__":
    unittest.main()
