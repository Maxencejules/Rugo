"""M38 PR-2: deterministic snapshot semantics checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_storage_feature_campaign_v1 as campaign  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _check(data: dict, check_id: str) -> dict:
    rows = [entry for entry in data["checks"] if entry["check_id"] == check_id]
    assert len(rows) == 1
    return rows[0]


def test_snapshot_semantics_v1_report_is_seed_deterministic():
    first = campaign.run_campaign(seed=20260309)
    second = campaign.run_campaign(seed=20260309)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_snapshot_semantics_v1_schema_and_pass(tmp_path: Path):
    out = tmp_path / "storage-feature-v1-snapshot.json"
    rc = campaign.main(["--seed", "20260309", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.storage_feature_campaign_report.v1"
    assert data["storage_feature_contract_id"] == "rugo.storage_feature_contract.v1"
    assert data["snapshot_policy_id"] == "rugo.snapshot_policy.v1"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["summary"]["snapshot"]["pass"] is True
    assert data["snapshot"]["checks_pass"] is True
    assert data["snapshot"]["snapshot_create_ms"] <= 80
    assert data["snapshot"]["snapshot_restore_integrity_ratio"] >= 1.0
    assert data["snapshot"]["snapshot_retention_violations"] == 0
    assert data["snapshot"]["snapshot_orphan_refs"] == 0
    assert _check(data, "snapshot_create_ms")["pass"] is True
    assert _check(data, "snapshot_restore_integrity_ratio")["pass"] is True


def test_snapshot_semantics_v1_detects_restore_integrity_regression(tmp_path: Path):
    out = tmp_path / "storage-feature-v1-snapshot-fail.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "snapshot_restore_integrity_ratio",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["snapshot"]["failures"] >= 1
    assert _check(data, "snapshot_restore_integrity_ratio")["pass"] is False


def test_snapshot_semantics_v1_rejects_unknown_check_id(tmp_path: Path):
    out = tmp_path / "storage-feature-v1-snapshot-error.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "snapshot_nonexistent_check",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
