"""M33 runtime-backed fleet control-plane checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_canary_rollout_sim_v1 as canary_tool  # noqa: E402
import run_fleet_health_sim_v1 as fleet_health  # noqa: E402
import run_fleet_update_sim_v1 as fleet_update  # noqa: E402
import run_rollout_abort_drill_v1 as abort_tool  # noqa: E402


def test_fleet_control_plane_v1_coordinates_report_driven_rollback(tmp_path: Path):
    canary_out = tmp_path / "canary-rollout-sim-v1.json"
    update_out = tmp_path / "fleet-update-sim-v1.json"
    health_out = tmp_path / "fleet-health-sim-v1.json"
    abort_out = tmp_path / "rollout-abort-drill-v1.json"

    assert (
        canary_tool.main(
            [
                "--fixture",
                "--inject-failure-stage",
                "canary",
                "--out",
                str(canary_out),
            ]
        )
        == 1
    )
    assert (
        fleet_update.main(
            [
                "--fixture",
                "--inject-failure-group",
                "batch_b",
                "--out",
                str(update_out),
            ]
        )
        == 1
    )
    assert (
        fleet_health.main(
            [
                "--fixture",
                "--inject-failure-cluster",
                "edge",
                "--out",
                str(health_out),
            ]
        )
        == 1
    )
    assert (
        abort_tool.main(
            [
                "--observed-error-rate",
                "0.01",
                "--observed-latency-p95-ms",
                "90",
                "--canary-report",
                str(canary_out),
                "--fleet-health-report",
                str(health_out),
                "--fleet-update-report",
                str(update_out),
                "--out",
                str(abort_out),
            ]
        )
        == 0
    )

    report = json.loads(abort_out.read_text(encoding="utf-8"))
    assert report["auto_halt"] is True
    assert report["rollback_triggered"] is True
    assert report["policy_enforced"] is True
    assert report["input_reports"]["canary_report"]["schema"] == "rugo.canary_rollout_report.v1"
    assert report["input_reports"]["fleet_health_report"]["schema"] == "rugo.fleet_health_report.v1"
    assert report["input_reports"]["fleet_update_report"]["schema"] == "rugo.fleet_update_sim_report.v1"
    targets = [target for action in report["recovery_actions"] for target in action["targets"]]
    assert "canary" in targets
    assert "batch_b" in targets
    assert "edge" in targets
