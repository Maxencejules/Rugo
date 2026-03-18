"""M33 aggregate gate: fleet operations v1 wiring and artifact checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_fleet_health_sim_v1 as health_tool  # noqa: E402
import run_fleet_update_sim_v1 as update_tool  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_fleet_ops_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M33_EXECUTION_BACKLOG.md",
        "docs/pkg/fleet_update_policy_v1.md",
        "docs/runtime/fleet_health_policy_v1.md",
        "docs/pkg/staged_rollout_policy_v1.md",
        "docs/runtime/canary_slo_policy_v1.md",
        "tools/run_fleet_update_sim_v1.py",
        "tools/run_fleet_health_sim_v1.py",
        "tools/run_canary_rollout_sim_v1.py",
        "tools/run_rollout_abort_drill_v1.py",
        "tests/pkg/test_fleet_policy_docs_v1.py",
        "tests/pkg/test_fleet_update_sim_v1.py",
        "tests/runtime/test_fleet_health_sim_v1.py",
        "tests/runtime/test_fleet_control_plane_v1.py",
        "tests/runtime/test_fleet_rollout_safety_gate_v1.py",
        "tests/runtime/test_fleet_ops_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M33 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M33_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-fleet-ops-v1" in roadmap
    assert "test-fleet-rollout-safety-v1" in roadmap

    assert "test-fleet-ops-v1" in makefile
    for entry in [
        "tools/collect_booted_runtime_v1.py --image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --panic-image $(OUT)/os-panic.iso --out $(OUT)/booted-runtime-v1.json",
        "tools/run_fleet_update_sim_v1.py --seed 20260309 --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/fleet-update-sim-v1.json",
        "tools/run_fleet_health_sim_v1.py --seed 20260309 --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/fleet-health-sim-v1.json",
        "$(MAKE) test-fleet-rollout-safety-v1",
        "tests/pkg/test_fleet_policy_docs_v1.py",
        "tests/pkg/test_fleet_update_sim_v1.py",
        "tests/runtime/test_fleet_health_sim_v1.py",
        "tests/runtime/test_fleet_control_plane_v1.py",
        "tests/runtime/test_fleet_ops_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-fleet-ops-v1.xml" in makefile

    assert "Fleet ops v1 gate" in ci
    assert "make test-fleet-ops-v1" in ci
    assert "fleet-ops-v1-artifacts" in ci
    assert "out/pytest-fleet-ops-v1.xml" in ci
    assert "out/booted-runtime-v1.json" in ci
    assert "out/fleet-update-sim-v1.json" in ci
    assert "out/fleet-health-sim-v1.json" in ci

    assert "Fleet rollout safety v1 gate" in ci
    assert "make test-fleet-rollout-safety-v1" in ci

    assert "Status: done" in backlog
    assert "M33" in milestones
    assert "M33" in status
    assert "docs/architecture/SOURCE_MAP.md" in readme
    assert "docs/archive/README.md" in readme

    update_out = tmp_path / "fleet-update-sim-v1.json"
    health_out = tmp_path / "fleet-health-sim-v1.json"

    assert update_tool.main(["--seed", "20260309", "--fixture", "--out", str(update_out)]) == 0
    assert health_tool.main(["--seed", "20260309", "--fixture", "--out", str(health_out)]) == 0

    update_data = json.loads(update_out.read_text(encoding="utf-8"))
    health_data = json.loads(health_out.read_text(encoding="utf-8"))

    assert update_data["schema"] == "rugo.fleet_update_sim_report.v1"
    assert update_data["policy_id"] == "rugo.fleet_update_policy.v1"
    assert update_data["control_plane_mode"] == "runtime_lab"
    assert update_data["runtime_capture_digest"]
    assert update_data["total_failures"] == 0
    assert update_data["gate_pass"] is True

    assert health_data["schema"] == "rugo.fleet_health_report.v1"
    assert health_data["policy_id"] == "rugo.fleet_health_policy.v1"
    assert health_data["control_plane_mode"] == "runtime_lab"
    assert health_data["runtime_capture_digest"]
    assert health_data["fleet_state"] == "healthy"
    assert health_data["total_failures"] == 0
    assert health_data["gate_pass"] is True
