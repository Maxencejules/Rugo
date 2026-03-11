"""M48 PR-2: deterministic efifb fallback checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_display_runtime_v1 as runtime  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m48" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_efifb_fallback_v1_schema_and_pass():
    out = _out_path("display-runtime-v1-fallback.json")
    rc = runtime.main(["--seed", "20260311", "--force-fallback", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.display_runtime_report.v1"
    assert data["active_runtime_path"] == "framebuffer-console"
    assert data["active_runtime_driver"] == "efifb"
    assert data["policy_decision"] == "forced_fallback"
    assert data["fallback_ready"] is True
    assert data["fallback_runtime"]["display_class"] == "framebuffer-console"
    assert data["fallback_runtime"]["driver"] == "efifb"
    assert data["fallback_runtime"]["declared_support_source_schema"] == "rugo.baremetal_io_baseline.v1"
    assert data["fallback_runtime"]["checks_pass"] is True
    assert data["fallback_runtime"]["activation_latency_ms"] <= 80
    assert data["gate_pass"] is True


def test_efifb_fallback_v1_detects_scanout_regression():
    out = _out_path("display-runtime-v1-fallback-fail.json")
    rc = runtime.main(
        [
            "--force-fallback",
            "--inject-failure",
            "efifb_fallback_scanout",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["active_runtime_path"] == "framebuffer-console"
    assert "efifb_fallback_scanout" in data["failures"]
    assert data["summary"]["fallback"]["failures"] >= 1
    assert data["fallback_runtime"]["checks_pass"] is False
