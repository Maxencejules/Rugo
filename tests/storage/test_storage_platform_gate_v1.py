"""M38 aggregate gate: storage platform v1 wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_platform_feature_conformance_v1 as platform_conformance  # noqa: E402
import run_storage_feature_campaign_v1 as campaign  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_storage_platform_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M38_EXECUTION_BACKLOG.md",
        "docs/storage/fs_feature_contract_v1.md",
        "docs/storage/snapshot_policy_v1.md",
        "docs/storage/online_resize_policy_v1.md",
        "docs/runtime/platform_feature_profile_v1.md",
        "tools/run_storage_feature_campaign_v1.py",
        "tools/run_platform_feature_conformance_v1.py",
        "tests/storage/test_storage_feature_docs_v1.py",
        "tests/storage/test_snapshot_semantics_v1.py",
        "tests/storage/test_online_resize_v1.py",
        "tests/storage/test_advanced_fs_ops_v1.py",
        "tests/runtime/test_platform_feature_profile_v1.py",
        "tests/storage/test_storage_feature_contract_gate_v1.py",
        "tests/storage/test_storage_platform_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M38 artifact: {rel}"

    roadmap = _read("docs/M35_M39_GENERAL_PURPOSE_EXPANSION_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M38_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-storage-platform-v1" in roadmap
    assert "test-storage-feature-contract-v1" in roadmap

    assert "test-storage-platform-v1" in makefile
    for entry in [
        "tools/run_storage_feature_campaign_v1.py --out $(OUT)/storage-feature-v1.json",
        "$(MAKE) test-storage-feature-contract-v1",
        "tests/storage/test_storage_feature_docs_v1.py",
        "tests/storage/test_snapshot_semantics_v1.py",
        "tests/storage/test_online_resize_v1.py",
        "tests/storage/test_advanced_fs_ops_v1.py",
        "tests/runtime/test_platform_feature_profile_v1.py",
        "tests/storage/test_storage_platform_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-storage-platform-v1.xml" in makefile

    assert "Storage platform v1 gate" in ci
    assert "make test-storage-platform-v1" in ci
    assert "storage-platform-v1-artifacts" in ci
    assert "out/pytest-storage-platform-v1.xml" in ci
    assert "out/storage-feature-v1.json" in ci

    assert "Storage feature contract v1 gate" in ci
    assert "make test-storage-feature-contract-v1" in ci
    assert "storage-feature-contract-v1-artifacts" in ci
    assert "out/pytest-storage-feature-contract-v1.xml" in ci
    assert "out/platform-feature-v1.json" in ci

    assert "Status: done" in backlog
    assert "M38" in milestones
    assert "M38" in status
    assert "M38" in readme

    feature_out = tmp_path / "storage-feature-v1.json"
    platform_out = tmp_path / "platform-feature-v1.json"

    assert campaign.main(["--seed", "20260309", "--out", str(feature_out)]) == 0
    assert (
        platform_conformance.main(["--seed", "20260309", "--out", str(platform_out)])
        == 0
    )

    feature_data = json.loads(feature_out.read_text(encoding="utf-8"))
    platform_data = json.loads(platform_out.read_text(encoding="utf-8"))

    assert feature_data["schema"] == "rugo.storage_feature_campaign_report.v1"
    assert feature_data["gate_pass"] is True
    assert feature_data["total_failures"] == 0

    assert platform_data["schema"] == "rugo.platform_feature_conformance_report.v1"
    assert platform_data["policy_id"] == "rugo.platform_feature_profile.v1"
    assert platform_data["gate_pass"] is True
