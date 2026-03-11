"""M48 PR-2: deterministic display present timing checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_display_runtime_v1 as runtime  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m48" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_display_present_timing_v1_deterministic_report():
    first = runtime.run_display_runtime(seed=20260311)
    second = runtime.run_display_runtime(seed=20260311)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_display_present_timing_v1_schema_and_thresholds():
    out = _out_path("display-runtime-v1-timing.json")
    rc = runtime.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["present_timing"]["target_refresh_hz"] == 60
    assert data["present_timing"]["frame_budget_ms"] == 16.667
    assert data["present_timing"]["present_latency_p95_ms"] <= 16.667
    assert data["present_timing"]["vblank_jitter_p95_ms"] <= 1.5
    assert data["present_timing"]["timing_checks_pass"] is True
    assert data["buffer_pool"]["scanout_buffer_count"] == 3
    assert data["buffer_pool"]["capture_shadow_count"] == 1
    assert data["buffer_pool"]["integrity_pass"] is True
    assert data["summary"]["timing"]["pass"] is True


def test_display_present_timing_v1_detects_budget_regression():
    out = _out_path("display-runtime-v1-timing-fail.json")
    rc = runtime.main(
        [
            "--inject-failure",
            "present_timing_budget",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "present_timing_budget" in data["failures"]
    assert data["summary"]["timing"]["failures"] >= 1
    assert data["present_timing"]["timing_checks_pass"] is False
