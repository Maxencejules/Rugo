"""M33 aggregate gate: fleet rollout safety wiring and artifact checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_canary_rollout_sim_v1 as canary  # noqa: E402
import run_rollout_abort_drill_v1 as abort  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_fleet_rollout_safety_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M33_EXECUTION_BACKLOG.md",
        "docs/pkg/staged_rollout_policy_v1.md",
        "docs/runtime/canary_slo_policy_v1.md",
        "tools/run_canary_rollout_sim_v1.py",
        "tools/run_rollout_abort_drill_v1.py",
        "tests/pkg/test_rollout_policy_docs_v1.py",
        "tests/pkg/test_canary_rollout_sim_v1.py",
        "tests/runtime/test_rollout_abort_policy_v1.py",
        "tests/runtime/test_fleet_control_plane_v1.py",
        "tests/runtime/test_fleet_rollout_safety_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing gate artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M33_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-fleet-rollout-safety-v1" in roadmap
    assert "test-fleet-ops-v1" in roadmap

    assert "test-fleet-rollout-safety-v1" in makefile
    for entry in [
        "tools/collect_booted_runtime_v1.py --image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --panic-image $(OUT)/os-panic.iso --out $(OUT)/booted-runtime-v1.json",
        "tools/run_canary_rollout_sim_v1.py --seed 20260309 --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/canary-rollout-sim-v1.json",
        "tools/run_fleet_update_sim_v1.py --seed 20260309 --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/fleet-update-sim-v1.json",
        "tools/run_fleet_health_sim_v1.py --seed 20260309 --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/fleet-health-sim-v1.json",
        "tools/run_rollout_abort_drill_v1.py --canary-report $(OUT)/canary-rollout-sim-v1.json --fleet-health-report $(OUT)/fleet-health-sim-v1.json --fleet-update-report $(OUT)/fleet-update-sim-v1.json --out $(OUT)/rollout-abort-drill-v1.json",
        "tests/pkg/test_rollout_policy_docs_v1.py",
        "tests/pkg/test_canary_rollout_sim_v1.py",
        "tests/runtime/test_rollout_abort_policy_v1.py",
        "tests/runtime/test_fleet_control_plane_v1.py",
        "tests/runtime/test_fleet_rollout_safety_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-fleet-rollout-safety-v1.xml" in makefile

    assert "Fleet rollout safety v1 gate" in ci
    assert "make test-fleet-rollout-safety-v1" in ci
    assert "fleet-rollout-safety-v1-artifacts" in ci
    assert "out/pytest-fleet-rollout-safety-v1.xml" in ci
    assert "out/booted-runtime-v1.json" in ci
    assert "out/canary-rollout-sim-v1.json" in ci
    assert "out/fleet-update-sim-v1.json" in ci
    assert "out/fleet-health-sim-v1.json" in ci
    assert "out/rollout-abort-drill-v1.json" in ci

    assert "Status: done" in backlog
    assert "M33" in milestones
    assert "M33" in status
    assert "docs/architecture/SOURCE_MAP.md" in readme
    assert "docs/archive/README.md" in readme

    canary_out = tmp_path / "canary-rollout-sim-v1.json"
    abort_out = tmp_path / "rollout-abort-drill-v1.json"
    assert canary.main(["--seed", "20260309", "--fixture", "--out", str(canary_out)]) == 0
    assert abort.main(["--out", str(abort_out)]) == 0

    canary_data = json.loads(canary_out.read_text(encoding="utf-8"))
    abort_data = json.loads(abort_out.read_text(encoding="utf-8"))
    assert canary_data["schema"] == "rugo.canary_rollout_report.v1"
    assert canary_data["policy_id"] == "rugo.staged_rollout_policy.v1"
    assert canary_data["control_plane_mode"] == "runtime_lab"
    assert canary_data["runtime_capture_digest"]
    assert canary_data["total_failures"] == 0
    assert canary_data["gate_pass"] is True
    assert abort_data["schema"] == "rugo.rollout_abort_drill_report.v1"
    assert abort_data["policy_id"] == "rugo.canary_slo_policy.v1"
    assert abort_data["policy_enforced"] is True
    assert abort_data["meets_target"] is True
