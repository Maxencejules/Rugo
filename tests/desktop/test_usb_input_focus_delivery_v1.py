"""M46 PR-2: desktop USB input focus-delivery bridge checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_desktop_smoke_v1 as smoke  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m46" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_usb_input_focus_delivery_v1_schema_and_pass():
    out = _out_path("desktop-smoke-v1-usb-input.json")
    rc = smoke.main(
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
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.desktop_smoke_report.v1"
    assert data["input_class"] == "usb-hid"
    assert data["input_device"]["driver"] == "xhci-usb-hid"
    assert data["input_device"]["desktop_qualified"] is True
    assert data["desktop_input_checks"]["input_class"] == "usb-hid"
    assert data["desktop_input_checks"]["input_checks_pass"] is True
    assert data["desktop_input_checks"]["focus_delivery_pass"] is True
    assert data["desktop_input_checks"]["qualifying_checks"] == [
        "input_keyboard_latency",
        "input_pointer_latency",
        "input_focus_delivery",
        "input_repeat_consistency",
    ]


def test_usb_input_focus_delivery_v1_detects_focus_failure():
    out = _out_path("desktop-smoke-v1-usb-input-fail.json")
    rc = smoke.main(
        [
            "--input-class",
            "usb-hid",
            "--input-driver",
            "xhci-usb-hid",
            "--inject-failure",
            "input_focus_delivery",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["desktop_input_checks"]["focus_delivery_pass"] is False
    assert data["input_device"]["desktop_qualified"] is False
