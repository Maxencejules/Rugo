"""M50 PR-2: deterministic surface lifecycle checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_window_system_runtime_v1 as runtime  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m50" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_surface_lifecycle_v1_schema_and_states():
    out = _out_path("window-system-v1-lifecycle.json")
    rc = runtime.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    phases = [row["phase"] for row in data["lifecycle_log"]]
    states = {surface["window_id"]: surface["state"] for surface in data["surfaces"]}

    assert data["schema"] == "rugo.window_system_runtime_report.v1"
    assert data["surface_contract_id"] == "rugo.surface_lifecycle_contract.v1"
    assert data["summary"]["lifecycle"]["pass"] is True
    assert data["surface_counts"]["active"] == 3
    assert data["surface_counts"]["retired"] == 1
    assert "create" in phases
    assert "map" in phases
    assert "activate" in phases
    assert "destroy" in phases
    assert states["desktop.shell.workspace"] == "visible"
    assert states["files.panel"] == "occluded"
    assert states["settings.panel"] == "focused"
    assert data["retired_surfaces"][0]["window_id"] == "toast.network"
    assert data["retired_surfaces"][0]["final_state"] == "destroyed"


def test_surface_lifecycle_v1_detects_state_regression():
    out = _out_path("window-system-v1-lifecycle-fail.json")
    rc = runtime.main(
        [
            "--inject-failure",
            "surface_visibility_integrity",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "surface_visibility_integrity" in data["failures"]
    assert data["summary"]["lifecycle"]["failures"] >= 1
    assert data["surface_audit"]["checks_pass"] is False
    assert len(data["surface_audit"]["state_violations"]) >= 1
