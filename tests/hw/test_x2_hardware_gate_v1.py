"""X2 aggregate gate: hardware runtime-backed closure wiring and docs."""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_x2_hardware_runtime_v1 as tool  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _work_dir(name: str) -> Path:
    path = ROOT / "out" / "pytest-x2" / name
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)
    return path


def test_x2_hardware_gate_v1_wiring_and_artifacts():
    required = [
        "docs/hw/x2_hardware_runtime_qualification_v1.md",
        "tools/x2_hardware_runtime_common_v1.py",
        "tools/run_x2_hardware_runtime_v1.py",
        "tests/hw/test_x2_hardware_runtime_v1.py",
        "tests/hw/test_x2_hardware_gate_v1.py",
        "docs/roadmap/implementation_closure/expansion_breadth.md",
        "docs/M9_EXECUTION_BACKLOG.md",
        "docs/M15_EXECUTION_BACKLOG.md",
        "docs/M23_EXECUTION_BACKLOG.md",
        "docs/M37_EXECUTION_BACKLOG.md",
        "docs/M43_EXECUTION_BACKLOG.md",
        "docs/M45_EXECUTION_BACKLOG.md",
        "docs/M46_EXECUTION_BACKLOG.md",
        "docs/M47_EXECUTION_BACKLOG.md",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing X2 artifact: {rel}"

    roadmap = _read("docs/roadmap/implementation_closure/expansion_breadth.md")
    summary = _read("docs/roadmap/README.md")
    framework = _read("docs/roadmap/MILESTONE_FRAMEWORK.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    readme = _read("README.md")

    assert "The historical X2 hardware backlog is now runtime-backed" in roadmap
    for row in [
        "| `M9 Hardware enablement matrix v1` | `Runtime-backed` |",
        "| `M15 Hardware Enablement Matrix v2` | `Runtime-backed` |",
        "| `M23 Hardware Enablement Matrix v3` | `Runtime-backed` |",
        "| `M37 Hardware Breadth + Driver Matrix v4` | `Runtime-backed` |",
        "| `M43 Hardware/Firmware Breadth + SMP v1` | `Runtime-backed` |",
        "| `M45 Modern Virtual Platform Parity v1` | `Runtime-backed` |",
        "| `M46 Bare-Metal I/O Baseline v1` | `Runtime-backed` |",
        "| `M47 Hardware Claim Promotion Program v1` | `Runtime-backed` |",
    ]:
        assert row in roadmap

    assert "historical X2 hardware backlog is closed on a shared runtime-backed qualification lane" in summary
    assert "historical X2 hardware backlog is closed on a shared runtime-backed qualification lane" in framework
    assert "`X2` Hardware, Firmware, and Driver Breadth | done |" in framework
    assert "make test-x2-hardware-runtime-v1" in readme

    assert "test-x2-hardware-runtime-v1" in makefile
    for entry in [
        "tools/run_x2_hardware_runtime_v1.py --emit-supporting-reports --out $(OUT)/x2-hardware-runtime-v1.json",
        "tests/hw/test_x2_hardware_runtime_v1.py",
        "tests/hw/test_x2_hardware_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-x2-hardware-runtime-v1.xml" in makefile

    assert "X2 hardware runtime v1 gate" in ci
    assert "make test-x2-hardware-runtime-v1" in ci
    assert "x2-hardware-runtime-v1-artifacts" in ci
    assert "out/pytest-x2-hardware-runtime-v1.xml" in ci
    assert "out/x2-hardware-runtime-v1.json" in ci
    assert "out/native-driver-diagnostics-v1.json" in ci

    for relpath in [
        "docs/M9_EXECUTION_BACKLOG.md",
        "docs/M15_EXECUTION_BACKLOG.md",
        "docs/M23_EXECUTION_BACKLOG.md",
        "docs/M37_EXECUTION_BACKLOG.md",
        "docs/M43_EXECUTION_BACKLOG.md",
        "docs/M45_EXECUTION_BACKLOG.md",
        "docs/M46_EXECUTION_BACKLOG.md",
        "docs/M47_EXECUTION_BACKLOG.md",
    ]:
        doc = _read(relpath)
        assert "X2 runtime-backed closure addendum (2026-03-18)" in doc
        assert "x2 hardware runtime qualification" in doc.lower()

    work_dir = _work_dir("gate")
    out = work_dir / "x2-hardware-runtime-v1.json"
    support_dir = work_dir / "supporting"
    rc = tool.main(
        [
            "--seed",
            "20260318",
            "--emit-supporting-reports",
            "--supporting-dir",
            str(support_dir),
            "--out",
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.x2_hardware_runtime_report.v1"
    assert data["gate_pass"] is True

    for name in [
        "measured-boot-v1.json",
        "hw-diagnostics-v3.json",
        "hw-matrix-v4.json",
        "hw-firmware-smp-v1.json",
        "hw-matrix-v6.json",
        "baremetal-io-v1.json",
        "hw-claim-promotion-v1.json",
        "native-driver-diagnostics-v1.json",
    ]:
        assert (support_dir / name).is_file(), f"missing emitted supporting report: {name}"
