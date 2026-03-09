"""M26 PR-1: update trust and key-rotation docs v1 contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_update_trust_docs_v1_present():
    required = [
        "docs/M26_EXECUTION_BACKLOG.md",
        "docs/pkg/update_trust_model_v1.md",
        "docs/security/update_key_rotation_policy_v1.md",
        "tools/check_update_trust_v1.py",
        "tools/run_update_key_rotation_drill_v1.py",
        "tests/pkg/test_update_trust_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M26 artifact: {rel}"


def test_update_trust_docs_declare_required_contract_tokens():
    trust_doc = _read("docs/pkg/update_trust_model_v1.md")
    rotation_doc = _read("docs/security/update_key_rotation_policy_v1.md")

    for token in [
        "Trust Model ID: `rugo.update_trust_model.v1`",
        "Report schema: `rugo.update_trust_report.v1`",
        "rollback attack",
        "replay/freeze attack",
        "mix-and-match target metadata attack",
        "Maximum allowed trust failures: `0`.",
        "make test-update-trust-v1",
    ]:
        assert token in trust_doc

    for token in [
        "Policy ID: `rugo.update_key_rotation_policy.v1`",
        "Drill report schema: `rugo.update_key_rotation_drill.v1`",
        "Stage 1: `old_key_only`",
        "Stage 2: `overlap_window`",
        "Stage 3: `new_key_primary`",
        "Stage 4: `old_key_revoked`",
        "Stage 5: `revocation_enforced`",
        "Maximum overlap window days: `14`.",
        "Revocation propagation SLA hours: `24`.",
    ]:
        assert token in rotation_doc
