"""M38 PR-2: deterministic online resize checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_storage_feature_campaign_v1 as campaign  # noqa: E402


def _check(data: dict, check_id: str) -> dict:
    rows = [entry for entry in data["checks"] if entry["check_id"] == check_id]
    assert len(rows) == 1
    return rows[0]


def test_online_resize_v1_schema_and_pass(tmp_path: Path):
    out = tmp_path / "storage-feature-v1-resize.json"
    rc = campaign.main(["--seed", "20260309", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.storage_feature_campaign_report.v1"
    assert data["online_resize_policy_id"] == "rugo.online_resize_policy.v1"
    assert data["summary"]["resize"]["pass"] is True
    assert data["online_resize"]["checks_pass"] is True
    assert data["online_resize"]["resize_grow_ms"] <= 120
    assert data["online_resize"]["resize_capacity_mismatch_count"] == 0
    assert data["online_resize"]["resize_shrink_guard_ratio"] >= 1.0
    assert data["online_resize"]["resize_post_fsck_errors"] == 0
    assert _check(data, "resize_grow_ms")["pass"] is True
    assert _check(data, "resize_shrink_guard_ratio")["pass"] is True


def test_online_resize_v1_detects_grow_regression(tmp_path: Path):
    out = tmp_path / "storage-feature-v1-resize-fail.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "resize_grow_ms",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["resize"]["failures"] >= 1
    assert _check(data, "resize_grow_ms")["pass"] is False


def test_online_resize_v1_detects_shrink_guard_regression(tmp_path: Path):
    out = tmp_path / "storage-feature-v1-resize-guard-fail.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "resize_shrink_guard_ratio",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["resize"]["failures"] >= 1
    assert _check(data, "resize_shrink_guard_ratio")["pass"] is False
