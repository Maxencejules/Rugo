"""M26 PR-1: package/repository v3 doc contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m26_pr1_artifacts_exist():
    required = [
        "docs/M26_EXECUTION_BACKLOG.md",
        "docs/pkg/package_format_v3.md",
        "docs/pkg/repository_policy_v3.md",
        "docs/pkg/update_trust_model_v1.md",
        "docs/security/update_key_rotation_policy_v1.md",
        "tests/pkg/test_pkg_contract_docs_v3.py",
        "tests/pkg/test_update_trust_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M26 PR-1 artifact: {rel}"


def test_pkg_docs_declare_required_contract_tokens():
    format_doc = _read("docs/pkg/package_format_v3.md")
    policy_doc = _read("docs/pkg/repository_policy_v3.md")

    for token in [
        "Package Format ID: `rugo.pkg_format.v3`",
        "Package schema: `rugo.pkg.v3`",
        "Repository index schema: `rugo.repo_index.v3`",
        "Rebuild manifest schema: `rugo.pkg_rebuild_manifest.v3`",
        "Rebuild report schema: `rugo.pkg_rebuild_report.v3`",
        "make test-pkg-ecosystem-v3",
        "tools/pkg_rebuild_verify_v3.py",
    ]:
        assert token in format_doc

    for token in [
        "Policy ID: `rugo.repository_policy.v3`",
        "Policy report schema: `rugo.repo_policy_report.v3`",
        "Maximum metadata validity window hours: `168`.",
        "Allowed metadata clock skew seconds: `300`.",
        "make test-update-trust-v1",
        "tools/repo_policy_check_v3.py",
    ]:
        assert token in policy_doc


def test_m26_roadmap_anchor_declares_gate_names():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-pkg-ecosystem-v3" in roadmap
    assert "test-update-trust-v1" in roadmap

