"""M33 PR-2: rollout abort policy enforcement drill."""

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_rollout_abort_drill_v1 as drill  # noqa: E402


def test_rollout_abort_policy_v1_enforces_halt(tmp_path: Path):
    out = tmp_path / "rollout-abort-drill-v1.json"
    rc = drill.main(["--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.rollout_abort_drill_report.v1"
    assert data["policy_id"] == "rugo.canary_slo_policy.v1"
    assert data["auto_halt"] is True
    assert data["rollback_triggered"] is True
    assert data["policy_enforced"] is True
    assert data["halt_reason"] == "slo_breach_detected"
    assert data["recovery_actions"]
    assert data["meets_target"] is True
    assert data["total_failures"] == 0


def test_rollout_abort_policy_v1_detects_missing_halt(tmp_path: Path):
    out = tmp_path / "rollout-abort-drill-v1-fail.json"
    rc = drill.main(
        [
            "--observed-error-rate",
            "0.01",
            "--observed-latency-p95-ms",
            "90",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["auto_halt"] is False
    assert data["rollback_triggered"] is False
    assert data["policy_enforced"] is False
    assert data["halt_reason"] == "no_breach_detected"
    assert data["meets_target"] is False
    assert data["total_failures"] >= 1
