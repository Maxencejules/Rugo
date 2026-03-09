"""M38 PR-2: deterministic advanced filesystem operation checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_storage_feature_campaign_v1 as campaign  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _check(data: dict, check_id: str) -> dict:
    rows = [entry for entry in data["checks"] if entry["check_id"] == check_id]
    assert len(rows) == 1
    return rows[0]


def test_fs_feature_contract_v1_declares_required_advanced_ops():
    contract = _read("docs/storage/fs_feature_contract_v1.md")
    for token in [
        "`reflink`/clone-like copy path must be deterministic and regression-gated.",
        "`fallocate` preallocation path must preserve deterministic bounds.",
        "`copy_file_range` equivalent path must preserve deterministic completion",
        "xattr set/get roundtrip must preserve deterministic key/value integrity.",
    ]:
        assert token in contract


def test_advanced_fs_ops_v1_schema_and_pass(tmp_path: Path):
    out = tmp_path / "storage-feature-v1-fsops.json"
    rc = campaign.main(["--seed", "20260309", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.storage_feature_campaign_report.v1"
    assert data["summary"]["fs_ops"]["pass"] is True
    assert data["advanced_fs_ops"]["checks_pass"] is True
    assert data["advanced_fs_ops"]["fsops_reflink_ms"] <= 30
    assert data["advanced_fs_ops"]["fsops_fallocate_ms"] <= 15
    assert data["advanced_fs_ops"]["fsops_copy_file_range_ms"] <= 16
    assert data["advanced_fs_ops"]["fsops_xattr_roundtrip_ms"] <= 10
    assert data["advanced_fs_ops"]["fsops_dedupe_false_positive_count"] == 0
    assert _check(data, "fsops_reflink_ms")["pass"] is True
    assert _check(data, "fsops_xattr_roundtrip_ms")["pass"] is True


def test_advanced_fs_ops_v1_detects_xattr_regression(tmp_path: Path):
    out = tmp_path / "storage-feature-v1-fsops-fail.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "fsops_xattr_roundtrip_ms",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["fs_ops"]["failures"] >= 1
    assert _check(data, "fsops_xattr_roundtrip_ms")["pass"] is False
