"""M50 PR-2: deterministic compositor damage-region checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_compositor_damage_v1 as damage  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m50" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def _phase(data: dict, name: str) -> dict:
    matches = [entry for entry in data["phases"] if entry["phase"] == name]
    assert len(matches) == 1
    return matches[0]


def test_compositor_damage_regions_v1_schema_and_union():
    out = _out_path("compositor-damage-v1.json")
    rc = damage.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    move_phase = _phase(data, "window_move")
    resize_phase = _phase(data, "window_resize")

    assert data["schema"] == "rugo.compositor_damage_report.v1"
    assert data["damage_policy_id"] == "rugo.compositor_damage_policy.v1"
    assert data["summary"]["damage"]["pass"] is True
    assert data["summary"]["present"]["pass"] is True
    assert data["gate_pass"] is True
    assert move_phase["union_rect"] == {"x": 200, "y": 140, "width": 680, "height": 460}
    assert resize_phase["union_rect"] == {"x": 240, "y": 180, "width": 720, "height": 460}
    assert data["clip_snapshots"][1]["window_id"] == "files.panel"
    assert len(data["clip_snapshots"][1]["visible_regions"]) >= 2


def test_compositor_damage_regions_v1_detects_union_regression():
    out = _out_path("compositor-damage-v1-fail.json")
    rc = damage.main(
        [
            "--inject-failure",
            "damage_region_union",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "damage_region_union" in data["failures"]
    assert data["summary"]["damage"]["failures"] >= 1
