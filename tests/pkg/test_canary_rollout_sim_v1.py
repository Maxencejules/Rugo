"""M33 PR-2: canary rollout simulation checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_canary_rollout_sim_v1 as sim  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_canary_rollout_sim_v1_deterministic():
    first = sim.run_sim(
        seed=20260309,
        max_error_rate=0.02,
        max_latency_p95_ms=200,
        fixture=True,
    )
    second = sim.run_sim(
        seed=20260309,
        max_error_rate=0.02,
        max_latency_p95_ms=200,
        fixture=True,
    )
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_canary_rollout_sim_v1_schema_and_gate_pass(tmp_path: Path):
    out = tmp_path / "canary-rollout-sim-v1.json"
    rc = sim.main(["--seed", "20260309", "--fixture", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.canary_rollout_report.v1"
    assert data["policy_id"] == "rugo.staged_rollout_policy.v1"
    assert data["slo_policy_id"] == "rugo.canary_slo_policy.v1"
    assert data["control_plane_mode"] == "runtime_lab"
    assert data["runtime_capture_digest"]
    assert len(data["stages"]) == 3
    assert [stage["latency_p95_ms_threshold"] for stage in data["stages"]] == [200, 220, 250]
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert "halted" in data


def test_canary_rollout_sim_v1_detects_stage_failure(tmp_path: Path):
    out = tmp_path / "canary-rollout-sim-v1-fail.json"
    rc = sim.main(["--fixture", "--inject-failure-stage", "canary", "--out", str(out)])
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["halted"] is True
    assert data["gate_pass"] is False
    failed = [stage for stage in data["stages"] if stage["pass"] is False]
    assert any(stage["stage"] == "canary" for stage in failed)
