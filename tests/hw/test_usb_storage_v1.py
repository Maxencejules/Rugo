"""M46 PR-2: deterministic USB storage and recovery bridge checks."""

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


def test_usb_storage_v1_schema_and_pass():
    out = _out_path("baremetal-io-v1-usb-storage.json")
    rc = baseline.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.baremetal_io_baseline.v1"
    assert data["gate_pass"] is True
    assert data["removable_media"]["device_class"] == "usb-storage"
    assert data["removable_media"]["mount_latency_ms"] <= 400
    assert data["install_recovery_checks"]["workflow_id"] == "rugo.recovery_workflow.v3"
    assert data["install_recovery_checks"]["recovery_gate_pass"] is True
    assert _coverage_entry(data, "usb-storage")["status"] == "pass"
    row = _driver_row(data, "usb-storage")
    assert row["status"] == "pass"
    assert "media_ready" in row["states_observed"]
    assert "recovery_media_bootstrap" in row["states_observed"]


def test_usb_storage_v1_detects_mount_regression():
    out = _out_path("baremetal-io-v1-usb-storage-fail.json")
    rc = baseline.main(
        [
            "--inject-failure",
            "usb_storage_mount",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["removable"]["failures"] >= 1
    assert data["removable_media"]["checks_pass"] is False
    assert _coverage_entry(data, "usb-storage")["status"] == "fail"
    assert _driver_row(data, "usb-storage")["status"] == "fail"
