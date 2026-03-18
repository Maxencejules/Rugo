"""M34 acceptance: maturity bundle includes rollout safety drill evidence."""

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_canary_rollout_sim_v1 as canary  # noqa: E402
import run_rollout_abort_drill_v1 as abort  # noqa: E402


def test_maturity_rollout_safety_v1(tmp_path: Path):
    canary_out = tmp_path / "canary-rollout-sim-v1.json"
    abort_out = tmp_path / "rollout-abort-drill-v1.json"
    assert canary.main(["--fixture", "--out", str(canary_out)]) == 0
    assert abort.main(["--out", str(abort_out)]) == 0
    canary_data = json.loads(canary_out.read_text(encoding="utf-8"))
    abort_data = json.loads(abort_out.read_text(encoding="utf-8"))
    assert canary_data["schema"] == "rugo.canary_rollout_report.v1"
    assert canary_data["control_plane_mode"] == "runtime_lab"
    assert abort_data["policy_enforced"] is True
