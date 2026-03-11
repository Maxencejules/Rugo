"""M48 aggregate gate: display runtime wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import capture_display_frame_v1 as capture  # noqa: E402
import run_display_runtime_v1 as runtime  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m48" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    sidecar = path.with_suffix(".json")
    if sidecar.exists():
        sidecar.unlink()
    return path


def test_display_runtime_gate_v1_wiring_and_artifacts():
    required = [
        "docs/M48_EXECUTION_BACKLOG.md",
        "docs/desktop/display_runtime_contract_v1.md",
        "docs/desktop/scanout_buffer_contract_v1.md",
        "docs/desktop/gpu_fallback_policy_v1.md",
        "tools/run_display_runtime_v1.py",
        "tools/capture_display_frame_v1.py",
        "tests/desktop/test_display_runtime_docs_v1.py",
        "tests/desktop/test_virtio_gpu_scanout_v1.py",
        "tests/desktop/test_efifb_fallback_v1.py",
        "tests/desktop/test_display_present_timing_v1.py",
        "tests/desktop/test_display_frame_capture_v1.py",
        "tests/desktop/test_scanout_path_gate_v1.py",
        "tests/desktop/test_display_runtime_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M48 artifact: {rel}"

    roadmap = _read("docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M48_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-display-runtime-v1" in roadmap
    assert "test-scanout-path-v1" in roadmap

    assert "test-display-runtime-v1" in makefile
    for entry in [
        "tools/run_display_runtime_v1.py --out $(OUT)/display-runtime-v1.json",
        "$(SUBMAKE) test-scanout-path-v1",
        "tests/desktop/test_display_runtime_docs_v1.py",
        "tests/desktop/test_virtio_gpu_scanout_v1.py",
        "tests/desktop/test_efifb_fallback_v1.py",
        "tests/desktop/test_display_present_timing_v1.py",
        "tests/desktop/test_display_frame_capture_v1.py",
        "tests/desktop/test_display_runtime_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-display-runtime-v1.xml" in makefile
    assert "pytest-scanout-path-v1.xml" in makefile

    assert "Display runtime v1 gate" in ci
    assert "make test-display-runtime-v1" in ci
    assert "display-runtime-v1-artifacts" in ci
    assert "out/pytest-display-runtime-v1.xml" in ci
    assert "out/display-runtime-v1.json" in ci
    assert "out/display-frame-v1.png" in ci
    assert "out/display-frame-v1.json" in ci

    assert "Scanout path v1 gate" in ci
    assert "make test-scanout-path-v1" in ci
    assert "scanout-path-v1-artifacts" in ci
    assert "out/pytest-scanout-path-v1.xml" in ci

    assert "Status: done" in backlog
    assert "| M48 | Display Runtime + Scanout v1 | n/a | done |" in milestones
    assert "| **M48** Display Runtime + Scanout v1 | n/a | done |" in status
    assert "M0-M48: done" in readme
    assert "M48 execution backlog (completed)" in readme

    runtime_out = _out_path("display-runtime-v1.json")
    frame_out = _out_path("display-frame-v1.png")

    assert runtime.main(["--seed", "20260311", "--out", str(runtime_out)]) == 0
    assert capture.main(["--seed", "20260311", "--out", str(frame_out)]) == 0

    runtime_data = json.loads(runtime_out.read_text(encoding="utf-8"))
    frame_data = json.loads(frame_out.with_suffix(".json").read_text(encoding="utf-8"))

    assert runtime_data["schema"] == "rugo.display_runtime_report.v1"
    assert runtime_data["contract_id"] == "rugo.display_runtime_contract.v1"
    assert runtime_data["gate_pass"] is True
    assert runtime_data["total_failures"] == 0

    assert frame_data["schema"] == "rugo.display_frame_capture.v1"
    assert frame_data["runtime_schema"] == "rugo.display_runtime_report.v1"
    assert frame_data["gate_pass"] is True
