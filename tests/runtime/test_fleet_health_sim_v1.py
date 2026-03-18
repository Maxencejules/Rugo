"""M33 PR-2: fleet health simulation checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_fleet_health_sim_v1 as sim  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_fleet_health_sim_v1_deterministic():
    first = sim.run_sim(
        seed=20260309,
        max_fleet_degraded_ratio=0.05,
        max_fleet_error_rate=0.02,
        fixture=True,
    )
    second = sim.run_sim(
        seed=20260309,
        max_fleet_degraded_ratio=0.05,
        max_fleet_error_rate=0.02,
        fixture=True,
    )
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_fleet_health_sim_v1_schema_and_gate_pass(tmp_path: Path):
    out = tmp_path / "fleet-health-sim-v1.json"
    rc = sim.main(["--seed", "20260309", "--fixture", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.fleet_health_report.v1"
    assert data["policy_id"] == "rugo.fleet_health_policy.v1"
    assert data["control_plane_mode"] == "runtime_lab"
    assert data["runtime_capture_digest"]
    assert data["lab_nodes_total"] == 12
    assert data["fleet_state"] == "healthy"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0


def test_fleet_health_sim_v1_detects_degraded_cluster(tmp_path: Path):
    out = tmp_path / "fleet-health-sim-v1-fail.json"
    rc = sim.main(
        [
            "--fixture",
            "--inject-failure-cluster",
            "core",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["fleet_state"] == "degraded"
    assert data["gate_pass"] is False
    assert data["total_failures"] >= 1
    core = next(entry for entry in data["clusters"] if entry["cluster_id"] == "core")
    assert core["within_slo"] is False
    assert core["nodes"]
