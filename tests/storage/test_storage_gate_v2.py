"""M18 aggregate gate: storage reliability v2 contract and gate wiring."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_storage_reliability_v2_gate_wiring_and_artifacts():
    required = [
        "docs/M18_EXECUTION_BACKLOG.md",
        "docs/storage/fs_v2.md",
        "docs/storage/durability_model_v2.md",
        "docs/storage/write_ordering_policy_v2.md",
        "docs/storage/recovery_playbook_v2.md",
        "docs/storage/fault_campaign_v2.md",
        "tools/storage_recover_v2.py",
        "tools/run_storage_powerfail_campaign_v2.py",
        "tests/storage/test_journal_recovery_v2.py",
        "tests/storage/test_metadata_integrity_v2.py",
        "tests/storage/test_powerfail_campaign_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M18 artifact: {rel}"

    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M18_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")

    assert "test-storage-reliability-v2" in makefile
    for entry in [
        "tools/storage_recover_v2.py --check --out $(OUT)/storage-recovery-v2.json",
        "tools/run_storage_powerfail_campaign_v2.py --seed 20260304 --out $(OUT)/storage-powerfail-v2.json",
        "tests/storage/test_journal_recovery_v2.py",
        "tests/storage/test_powerfail_campaign_v2.py",
        "tests/storage/test_metadata_integrity_v2.py",
        "tests/storage/test_storage_gate_v2.py",
    ]:
        assert entry in makefile
    assert "pytest-storage-reliability-v2.xml" in makefile

    assert "Storage reliability v2 gate" in ci
    assert "make test-storage-reliability-v2" in ci
    assert "storage-reliability-v2-artifacts" in ci
    assert "out/storage-recovery-v2.json" in ci
    assert "out/storage-powerfail-v2.json" in ci

    assert "Status: done" in backlog
    assert "M18" in milestones
    assert "M18" in status
