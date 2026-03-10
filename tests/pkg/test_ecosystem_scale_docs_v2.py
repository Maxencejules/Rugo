"""M44 PR-1: ecosystem scale and distribution workflow v2 doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m44_pr1_ecosystem_contract_artifacts_exist():
    required = [
        "docs/M44_EXECUTION_BACKLOG.md",
        "docs/pkg/ecosystem_scale_policy_v2.md",
        "docs/pkg/distribution_workflow_v2.md",
        "docs/desktop/desktop_profile_v2.md",
        "docs/abi/app_compat_tiers_v2.md",
        "tests/pkg/test_ecosystem_scale_docs_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M44 PR-1 artifact: {rel}"


def test_ecosystem_scale_policy_v2_doc_declares_required_tokens():
    doc = _read("docs/pkg/ecosystem_scale_policy_v2.md")
    for token in [
        "Policy ID: `rugo.ecosystem_scale_policy.v2`.",
        "Distribution workflow ID: `rugo.distribution_workflow.v2`.",
        "Desktop profile ID: `rugo.desktop_profile.v2`.",
        "App tier schema ID: `rugo.app_compat_tiers.v2`.",
        "GUI runtime schema: `rugo.real_gui_app_matrix_report.v2`.",
        "Install campaign schema: `rugo.real_pkg_install_campaign_report.v2`.",
        "Catalog audit schema: `rugo.real_catalog_audit_report.v2`.",
        "Total catalog entries: `>= 520`.",
        "Class coverage floor per declared class: `>= 90`.",
        "Catalog metadata completeness ratio: `>= 0.998`.",
        "Signed provenance coverage ratio: `>= 1.0`.",
        "Runtime trace coverage ratio: `>= 1.0`.",
        "Reproducible install ratio: `>= 0.99`.",
        "Unsupported workload ratio: `<= 0.01`.",
        "Policy violation count: `0`.",
        "Local gate: `make test-real-ecosystem-desktop-v2`.",
        "Local sub-gate: `make test-real-app-catalog-v2`.",
        "CI gate: `Real ecosystem desktop v2 gate`.",
        "CI sub-gate: `Real app catalog v2 gate`.",
    ]:
        assert token in doc


def test_distribution_workflow_v2_doc_declares_required_tokens():
    doc = _read("docs/pkg/distribution_workflow_v2.md")
    for token in [
        "Policy ID: `rugo.distribution_workflow.v2`.",
        "Parent ecosystem policy ID: `rugo.ecosystem_scale_policy.v2`.",
        "Parent desktop profile ID: `rugo.desktop_profile.v2`.",
        "Workflow report schema: `rugo.real_catalog_audit_report.v2`.",
        "Install report schema: `rugo.real_pkg_install_campaign_report.v2`.",
        "`ingest`",
        "`vet`",
        "`sign`",
        "`runtime_qualify`",
        "`stage`",
        "`rollout`",
        "`rollback`",
        "Workflow stage completeness ratio: `>= 1.0`.",
        "Release signoff ratio: `>= 1.0`.",
        "Rollback drill pass ratio: `>= 1.0`.",
        "Mirror index consistency ratio: `>= 1.0`.",
        "Replication lag p95 minutes: `<= 10`.",
        "Runtime trace coverage ratio: `>= 1.0`.",
        "Signed artifact ratio: `>= 1.0`.",
        "Unresolved policy exceptions: `0`.",
        "Local gate: `make test-real-ecosystem-desktop-v2`.",
        "Local sub-gate: `make test-real-app-catalog-v2`.",
    ]:
        assert token in doc


def test_m40_m44_roadmap_anchor_declares_m44_gates():
    roadmap = _read("docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md")
    assert "test-real-ecosystem-desktop-v2" in roadmap
    assert "test-real-app-catalog-v2" in roadmap
