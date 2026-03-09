"""M38 aggregate sub-gate: storage feature contract v1 wiring and artifacts."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_platform_feature_conformance_v1 as conformance  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_storage_feature_contract_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M38_EXECUTION_BACKLOG.md",
        "docs/storage/fs_feature_contract_v1.md",
        "docs/runtime/platform_feature_profile_v1.md",
        "tools/run_platform_feature_conformance_v1.py",
        "tests/storage/test_storage_feature_docs_v1.py",
        "tests/runtime/test_platform_feature_profile_v1.py",
        "tests/storage/test_storage_feature_contract_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M38 feature-contract artifact: {rel}"

    roadmap = _read("docs/M35_M39_GENERAL_PURPOSE_EXPANSION_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M38_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-storage-feature-contract-v1" in roadmap

    assert "test-storage-feature-contract-v1" in makefile
    for entry in [
        "tools/run_platform_feature_conformance_v1.py --out $(OUT)/platform-feature-v1.json",
        "tests/storage/test_storage_feature_docs_v1.py",
        "tests/runtime/test_platform_feature_profile_v1.py",
        "tests/storage/test_storage_feature_contract_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-storage-feature-contract-v1.xml" in makefile

    assert "Storage feature contract v1 gate" in ci
    assert "make test-storage-feature-contract-v1" in ci
    assert "storage-feature-contract-v1-artifacts" in ci
    assert "out/pytest-storage-feature-contract-v1.xml" in ci
    assert "out/platform-feature-v1.json" in ci

    assert "Status: done" in backlog
    assert "M38" in milestones
    assert "M38" in status
    assert "M38" in readme

    out = tmp_path / "platform-feature-v1.json"
    assert conformance.main(["--seed", "20260309", "--out", str(out)]) == 0
    report = json.loads(out.read_text(encoding="utf-8"))
    assert report["schema"] == "rugo.platform_feature_conformance_report.v1"
    assert report["policy_id"] == "rugo.platform_feature_profile.v1"
    assert report["gate_pass"] is True
    assert report["total_failures"] == 0
