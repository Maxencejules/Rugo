"""M33 PR-2: fleet update simulation checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_fleet_update_sim_v1 as sim  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_fleet_update_sim_v1_deterministic():
    first = sim.run_sim(
        seed=20260309,
        target_version="2.4.0",
        min_success_rate=0.98,
        fixture=True,
    )
    second = sim.run_sim(
        seed=20260309,
        target_version="2.4.0",
        min_success_rate=0.98,
        fixture=True,
    )
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_fleet_update_sim_v1_schema_and_gate_pass(tmp_path: Path):
    out = tmp_path / "fleet-update-sim-v1.json"
    rc = sim.main(["--seed", "20260309", "--fixture", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.fleet_update_sim_report.v1"
    assert data["policy_id"] == "rugo.fleet_update_policy.v1"
    assert data["control_plane_mode"] == "runtime_lab"
    assert data["runtime_capture_digest"]
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["lab_nodes_total"] == 12
    assert {entry["group_id"] for entry in data["groups"]} == {
        "canary",
        "batch_a",
        "batch_b",
    }
    assert all(entry["nodes"] for entry in data["groups"])


def test_fleet_update_sim_v1_detects_failed_group(tmp_path: Path):
    out = tmp_path / "fleet-update-sim-v1-fail.json"
    rc = sim.main(
        [
            "--fixture",
            "--inject-failure-group",
            "batch_b",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    failed = [entry for entry in data["groups"] if entry["pass"] is False]
    assert any(entry["group_id"] == "batch_b" for entry in failed)
    batch_b = next(entry for entry in data["groups"] if entry["group_id"] == "batch_b")
    assert batch_b["rollback_triggered"] is True
