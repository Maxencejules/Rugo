"""M46 aggregate sub-gate: USB input/removable wiring and artifacts."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_baremetal_io_baseline_v1 as baseline  # noqa: E402
import run_desktop_smoke_v1 as smoke  # noqa: E402
import run_recovery_drill_v3 as recovery  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m46" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_usb_input_removable_gate_v1_wiring_and_artifacts():
    required = [
        "docs/M46_EXECUTION_BACKLOG.md",
        "docs/hw/baremetal_io_profile_v1.md",
        "docs/hw/usb_input_removable_contract_v1.md",
        "docs/desktop/input_stack_contract_v1.md",
        "tools/run_baremetal_io_baseline_v1.py",
        "tools/run_desktop_smoke_v1.py",
        "tools/run_recovery_drill_v3.py",
        "tests/hw/test_usb_input_removable_docs_v1.py",
        "tests/hw/test_xhci_usb_hid_v1.py",
        "tests/hw/test_usb_storage_v1.py",
        "tests/hw/test_baremetal_io_recovery_v1.py",
        "tests/desktop/test_usb_input_focus_delivery_v1.py",
        "tests/hw/test_usb_input_removable_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M46 sub-gate artifact: {rel}"

    roadmap = _read("docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M46_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-usb-input-removable-v1" in roadmap

    assert "test-usb-input-removable-v1" in makefile
    for entry in [
        "tools/run_baremetal_io_baseline_v1.py --out $(OUT)/baremetal-io-v1.json",
        "tools/run_desktop_smoke_v1.py --input-class usb-hid --input-driver xhci-usb-hid --display-class framebuffer-console --display-driver efifb --boot-transport-class ahci --out $(OUT)/desktop-smoke-v1.json",
        "tools/run_recovery_drill_v3.py --seed 20260309 --out $(OUT)/recovery-drill-v3.json",
        "tests/hw/test_usb_input_removable_docs_v1.py",
        "tests/hw/test_xhci_usb_hid_v1.py",
        "tests/hw/test_usb_storage_v1.py",
        "tests/hw/test_baremetal_io_recovery_v1.py",
        "tests/desktop/test_usb_input_focus_delivery_v1.py",
        "tests/hw/test_usb_input_removable_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-usb-input-removable-v1.xml" in makefile

    assert "USB input removable v1 gate" in ci
    assert "make test-usb-input-removable-v1" in ci
    assert "usb-input-removable-v1-artifacts" in ci
    assert "out/pytest-usb-input-removable-v1.xml" in ci
    assert "out/baremetal-io-v1.json" in ci
    assert "out/desktop-smoke-v1.json" in ci
    assert "out/recovery-drill-v3.json" in ci

    assert "Status: done" in backlog
    assert "| M46 | Bare-Metal I/O Baseline v1 | n/a | done |" in milestones
    assert "| **M46** Bare-Metal I/O Baseline v1 | n/a | done |" in status
    assert "M46 execution backlog (completed)" in readme

    baseline_out = _out_path("usb-removable-baremetal-io-v1.json")
    smoke_out = _out_path("usb-removable-desktop-smoke-v1.json")
    recovery_out = _out_path("usb-removable-recovery-v3.json")

    assert baseline.main(["--seed", "20260310", "--out", str(baseline_out)]) == 0
    assert (
        smoke.main(
            [
                "--input-class",
                "usb-hid",
                "--input-driver",
                "xhci-usb-hid",
                "--display-class",
                "framebuffer-console",
                "--display-driver",
                "efifb",
                "--boot-transport-class",
                "ahci",
                "--out",
                str(smoke_out),
            ]
        )
        == 0
    )
    assert recovery.main(["--seed", "20260309", "--out", str(recovery_out)]) == 0

    baseline_data = json.loads(baseline_out.read_text(encoding="utf-8"))
    smoke_data = json.loads(smoke_out.read_text(encoding="utf-8"))
    recovery_data = json.loads(recovery_out.read_text(encoding="utf-8"))

    assert baseline_data["schema"] == "rugo.baremetal_io_baseline.v1"
    assert baseline_data["usb_input"]["checks_pass"] is True
    assert baseline_data["removable_media"]["checks_pass"] is True
    assert baseline_data["gate_pass"] is True

    assert smoke_data["schema"] == "rugo.desktop_smoke_report.v1"
    assert smoke_data["input_class"] == "usb-hid"
    assert smoke_data["desktop_input_checks"]["focus_delivery_pass"] is True
    assert smoke_data["gate_pass"] is True

    assert recovery_data["schema"] == "rugo.recovery_drill.v3"
    assert recovery_data["workflow_id"] == "rugo.recovery_workflow.v3"
    assert recovery_data["gate_pass"] is True

