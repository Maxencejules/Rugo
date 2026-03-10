"""M46 PR-2: deterministic bare-metal I/O promotion and recovery evidence checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_hw_promotion_evidence_v2 as promotion  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m46" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_baremetal_io_promotion_v2_is_seed_deterministic():
    first = promotion.run_promotion(
        seed=20260310,
        campaign_runs=12,
        required_consecutive_green=12,
        min_pass_rate=0.98,
    )
    second = promotion.run_promotion(
        seed=20260310,
        campaign_runs=12,
        required_consecutive_green=12,
        min_pass_rate=0.98,
    )
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_baremetal_io_promotion_v2_schema_and_gate_pass():
    out = _out_path("hw-promotion-v2.json")
    rc = promotion.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.hw_baremetal_promotion_report.v2"
    assert data["profile_id"] == "rugo.baremetal_io_profile.v1"
    assert data["baseline_schema_id"] == "rugo.baremetal_io_baseline.v1"
    assert data["gate_pass"] is True
    assert data["summary"]["pass_rate"] >= 0.98
    assert data["summary"]["trailing_consecutive_green"] >= 12
    assert data["desktop_usb_bridge_green"] is True
    assert data["recovery_bridge_green"] is True
    assert data["tier2_floor_met"] is True
    assert data["missing_artifacts"] == []


def test_baremetal_io_promotion_v2_detects_missing_artifact():
    out = _out_path("hw-promotion-v2-missing-artifact.json")
    rc = promotion.main(
        [
            "--inject-missing-artifact",
            "baseline_junit",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "baseline_junit" in data["missing_artifacts"]
    assert "artifact_bundle_complete" in data["failures"]
