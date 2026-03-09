"""M27 PR-1: compatibility profile v3 and app-tier doc contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m27_pr1_artifacts_exist():
    required = [
        "docs/M27_EXECUTION_BACKLOG.md",
        "docs/abi/compat_profile_v3.md",
        "docs/abi/app_compat_tiers_v1.md",
        "tests/compat/test_app_tier_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M27 PR-1 artifact: {rel}"


def test_m27_docs_declare_required_contract_tokens():
    profile_doc = _read("docs/abi/compat_profile_v3.md")
    tier_doc = _read("docs/abi/app_compat_tiers_v1.md")

    for token in [
        "Compatibility profile identifier: `rugo.compat_profile.v3`.",
        "App compatibility tier schema: `rugo.app_compat_tiers.v1`.",
        "App compatibility report schema: `rugo.app_compat_matrix_report.v3`.",
        "Local gate: `make test-app-compat-v3`",
        "Required tool: `tools/run_app_compat_matrix_v3.py`",
        "| `cli` | `tier_cli` | 14 | 0.90 |",
        "| `runtime` | `tier_runtime` | 10 | 0.80 |",
        "| `service` | `tier_service` | 8 | 0.80 |",
    ]:
        assert token in profile_doc

    for token in [
        "Tier contract ID: `rugo.app_compat_tiers.v1`",
        "Parent compatibility profile: `rugo.compat_profile.v3`",
        "Report schema: `rugo.app_compat_matrix_report.v3`",
        "Tier `tier_cli`",
        "Tier `tier_runtime`",
        "Tier `tier_service`",
        "Local gate: `make test-app-compat-v3`",
        "App compatibility v3 gate",
    ]:
        assert token in tier_doc


def test_m27_roadmap_anchor_declares_gate_name():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-app-compat-v3" in roadmap
