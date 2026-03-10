"""M44 PR-1: desktop and app-tier v2 contract doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m44_pr1_desktop_contract_artifacts_exist():
    required = [
        "docs/M44_EXECUTION_BACKLOG.md",
        "docs/desktop/desktop_profile_v2.md",
        "docs/abi/app_compat_tiers_v2.md",
        "docs/pkg/ecosystem_scale_policy_v2.md",
        "docs/pkg/distribution_workflow_v2.md",
        "tests/desktop/test_desktop_docs_v2.py",
        "tests/pkg/test_ecosystem_scale_docs_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M44 PR-1 artifact: {rel}"


def test_desktop_profile_v2_doc_declares_required_tokens():
    doc = _read("docs/desktop/desktop_profile_v2.md")
    for token in [
        "Desktop profile ID: `rugo.desktop_profile.v2`",
        "Desktop runtime schema: `rugo.real_gui_app_matrix_report.v2`",
        "App tier schema: `rugo.app_compat_tiers.v2`",
        "Ecosystem policy ID: `rugo.ecosystem_scale_policy.v2`",
        "| `productivity` | `tier_productivity_runtime` | 8 | 0.875 |",
        "| `media` | `tier_media_runtime` | 6 | 0.833 |",
        "| `utility` | `tier_utility_runtime` | 7 | 0.857 |",
        "Signed provenance ratio must be `>= 1.0`.",
        "Runtime trace coverage ratio must be `>= 1.0`.",
        "Local gate: `make test-real-ecosystem-desktop-v2`.",
        "Local sub-gate: `make test-real-app-catalog-v2`.",
        "CI gate: `Real ecosystem desktop v2 gate`.",
        "CI sub-gate: `Real app catalog v2 gate`.",
    ]:
        assert token in doc


def test_app_compat_tiers_v2_doc_declares_required_tokens():
    doc = _read("docs/abi/app_compat_tiers_v2.md")
    for token in [
        "Tier contract ID: `rugo.app_compat_tiers.v2`.",
        "Parent desktop profile ID: `rugo.desktop_profile.v2`.",
        "Runtime matrix schema: `rugo.real_gui_app_matrix_report.v2`.",
        "Install campaign schema: `rugo.real_pkg_install_campaign_report.v2`.",
        "Audit schema: `rugo.real_catalog_audit_report.v2`.",
        "Minimum eligible cases: `8`.",
        "Minimum pass rate: `0.875`.",
        "Minimum eligible cases: `6`.",
        "Minimum pass rate: `0.833`.",
        "Minimum eligible cases: `7`.",
        "Minimum pass rate: `0.857`.",
        "Every case must link to runtime trace evidence (`runtime_trace_id`).",
        "Every case must declare `runtime_source=runtime_capture`.",
        "Unknown workload classes or tier mismatches are release-blocking.",
        "Local gate: `make test-real-ecosystem-desktop-v2`.",
        "Local sub-gate: `make test-real-app-catalog-v2`.",
    ]:
        assert token in doc


def test_m40_m44_roadmap_anchor_declares_m44_gates():
    roadmap = _read("docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md")
    assert "test-real-ecosystem-desktop-v2" in roadmap
    assert "test-real-app-catalog-v2" in roadmap
