"""M50 PR-2: deterministic window move/resize checks."""

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


def test_window_resize_move_v1_geometry_and_budget():
    out = _out_path("window-system-v1-geometry.json")
    rc = runtime.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    move = data["geometry_mutations"]["move"]
    resize = data["geometry_mutations"]["resize"]

    assert data["summary"]["geometry"]["pass"] is True
    assert move["window_id"] == "settings.panel"
    assert move["from"] == {"x": 200, "y": 140, "width": 640, "height": 420}
    assert move["to"] == {"x": 240, "y": 180, "width": 640, "height": 420}
    assert move["latency_ms"] <= 24.0
    assert resize["from"] == {"x": 240, "y": 180, "width": 640, "height": 420}
    assert resize["to"] == {"x": 240, "y": 180, "width": 720, "height": 460}
    assert resize["latency_ms"] <= 32.0
    assert data["geometry_mutations"]["checks_pass"] is True


def test_window_resize_move_v1_detects_resize_regression():
    out = _out_path("window-system-v1-geometry-fail.json")
    rc = runtime.main(
        [
            "--inject-failure",
            "window_resize_budget",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "window_resize_budget" in data["failures"]
    assert data["summary"]["geometry"]["failures"] >= 1
    assert data["geometry_mutations"]["resize"]["latency_ms"] > 32.0
