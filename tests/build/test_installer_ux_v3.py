"""M30 PR-1: installer/recovery UX v3 doc contract checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m30_pr1_artifacts_exist():
    required = [
        "docs/M30_EXECUTION_BACKLOG.md",
        "docs/build/installer_ux_v3.md",
        "docs/build/recovery_workflow_v3.md",
        "tests/build/test_installer_ux_v3.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M30 PR-1 artifact: {rel}"


def test_installer_ux_v3_doc_declares_required_tokens():
    doc = _read("docs/build/installer_ux_v3.md")
    for token in [
        "Installer UX contract ID: `rugo.installer_ux_contract.v3`",
        "Upgrade drill schema: `rugo.upgrade_drill.v3`",
        "Recovery drill schema: `rugo.recovery_drill.v3`",
        "Rollback safety schema: `rugo.rollback_safety_report.v3`",
        "Deterministic seed: `20260309`.",
        "Maximum allowed upgrade failures: `0`.",
        "Local gate: `make test-ops-ux-v3`",
        "CI gate: `Ops UX v3 gate`",
    ]:
        assert token in doc


def test_recovery_workflow_v3_doc_declares_required_tokens():
    doc = _read("docs/build/recovery_workflow_v3.md")
    for token in [
        "Recovery workflow ID: `rugo.recovery_workflow.v3`",
        "Recovery drill schema: `rugo.recovery_drill.v3`",
        "Rollback safety schema: `rugo.rollback_safety_report.v3`",
        "`recovery_entry_validation`",
        "`rollback_snapshot_mount`",
        "`state_reconciliation`",
        "`service_restore_validation`",
        "`post_recovery_audit`",
        "`rollback_floor_enforced` must be `true`.",
        "`unsigned_artifact_rejected` must be `true`.",
        "`rollback_path_verified` must be `true`.",
        "Maximum allowed recovery failures: `0`.",
        "make test-ops-ux-v3",
    ]:
        assert token in doc


def test_m30_roadmap_anchor_declares_ops_ux_gate():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-ops-ux-v3" in roadmap
