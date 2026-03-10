"""M46 PR-2: deterministic xHCI and USB HID baseline checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_baremetal_io_baseline_v1 as baseline  # noqa: E402


def _coverage_entry(data: dict, device: str) -> dict:
    rows = [entry for entry in data["device_class_coverage"] if entry["device"] == device]
    assert len(rows) == 1
    return rows[0]


def _driver_row(data: dict, driver: str) -> dict:
    rows = [entry for entry in data["driver_lifecycle"] if entry["driver"] == driver]
    assert len(rows) == 1
    return rows[0]


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m46" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_xhci_usb_hid_v1_schema_and_pass():
    out = _out_path("baremetal-io-v1-usb-hid.json")
    rc = baseline.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.baremetal_io_baseline.v1"
    assert data["gate_pass"] is True
    assert data["usb_input"]["controller"] == "xhci"
    assert data["usb_input"]["input_class"] == "usb-hid"
    assert data["usb_input"]["keyboard_latency_p95_ms"] <= 12
    assert data["usb_input"]["pointer_latency_p95_ms"] <= 14
    assert data["usb_input"]["focus_delivery_pass"] is True
    assert data["desktop_input_checks"]["input_class"] == "usb-hid"
    assert data["desktop_input_checks"]["input_checks_pass"] is True
    assert _coverage_entry(data, "usb-hid")["status"] == "pass"
    assert _driver_row(data, "usb-hid")["status"] == "pass"
    assert "hid_ready" in _driver_row(data, "usb-hid")["states_observed"]
    assert "focus_delivery_ready" in _driver_row(data, "usb-hid")["states_observed"]


def test_xhci_usb_hid_v1_detects_focus_delivery_regression():
    out = _out_path("baremetal-io-v1-usb-hid-fail.json")
    rc = baseline.main(
        [
            "--inject-failure",
            "usb_focus_delivery",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["usb_input"]["failures"] >= 1
    assert data["usb_input"]["checks_pass"] is False
    assert data["desktop_input_checks"]["focus_delivery_pass"] is False
    assert _coverage_entry(data, "usb-hid")["status"] == "fail"
    assert _driver_row(data, "usb-hid")["status"] == "fail"
