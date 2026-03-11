"""M50 aggregate sub-gate: compositor damage wiring and artifacts."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_compositor_damage_v1 as damage  # noqa: E402
import run_window_system_runtime_v1 as runtime  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m50" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_compositor_damage_gate_v1_wiring_and_artifacts():
    required = [
        "docs/M50_EXECUTION_BACKLOG.md",
        "docs/desktop/surface_lifecycle_contract_v1.md",
        "docs/desktop/compositor_damage_policy_v1.md",
        "docs/desktop/window_manager_contract_v2.md",
        "tools/run_window_system_runtime_v1.py",
        "tools/run_compositor_damage_v1.py",
        "tests/desktop/test_window_system_docs_v1.py",
        "tests/desktop/test_surface_lifecycle_v1.py",
        "tests/desktop/test_window_zorder_v1.py",
        "tests/desktop/test_compositor_damage_regions_v1.py",
        "tests/desktop/test_window_resize_move_v1.py",
        "tests/desktop/test_compositor_damage_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M50 sub-gate artifact: {rel}"

    roadmap = _read("docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M50_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-compositor-damage-v1" in roadmap

    assert "test-compositor-damage-v1" in makefile
    for entry in [
        "tools/run_window_system_runtime_v1.py --out $(OUT)/window-system-v1.json",
        "tools/run_compositor_damage_v1.py --out $(OUT)/compositor-damage-v1.json",
        "tests/desktop/test_window_system_docs_v1.py",
        "tests/desktop/test_surface_lifecycle_v1.py",
        "tests/desktop/test_window_zorder_v1.py",
        "tests/desktop/test_compositor_damage_regions_v1.py",
        "tests/desktop/test_window_resize_move_v1.py",
        "tests/desktop/test_compositor_damage_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-compositor-damage-v1.xml" in makefile

    assert "Compositor damage v1 gate" in ci
    assert "make test-compositor-damage-v1" in ci
    assert "compositor-damage-v1-artifacts" in ci
    assert "out/pytest-compositor-damage-v1.xml" in ci
    assert "out/window-system-v1.json" in ci
    assert "out/compositor-damage-v1.json" in ci

    assert "Status: done" in backlog
    assert "| M50 | Window System + Composition v1 | n/a | done |" in milestones
    assert "| **M50** Window System + Composition v1 | n/a | done |" in status
    assert "M50 execution backlog (completed)" in readme

    runtime_out = _out_path("damage-gate-window-system-v1.json")
    damage_out = _out_path("damage-gate-compositor-v1.json")

    assert runtime.main(["--seed", "20260311", "--out", str(runtime_out)]) == 0
    assert damage.main(["--seed", "20260311", "--out", str(damage_out)]) == 0

    runtime_data = json.loads(runtime_out.read_text(encoding="utf-8"))
    damage_data = json.loads(damage_out.read_text(encoding="utf-8"))

    assert runtime_data["schema"] == "rugo.window_system_runtime_report.v1"
    assert runtime_data["summary"]["z_order"]["pass"] is True
    assert runtime_data["summary"]["composition"]["pass"] is True
    assert runtime_data["gate_pass"] is True

    assert damage_data["schema"] == "rugo.compositor_damage_report.v1"
    assert damage_data["summary"]["damage"]["pass"] is True
    assert damage_data["summary"]["clipping"]["pass"] is True
    assert damage_data["gate_pass"] is True
