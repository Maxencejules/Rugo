"""M53 PR-2: deterministic native-driver bind and lifecycle checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_driver_diagnostics_v1 as diagnostics  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _binding(data: dict, driver: str, profile: str) -> dict:
    rows = [
        entry
        for entry in data["driver_bindings"]
        if entry["driver"] == driver and entry["profile"] == profile
    ]
    assert len(rows) == 1
    return rows[0]


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m53" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_driver_bind_lifecycle_v1_report_is_seed_deterministic():
    first = diagnostics.run_diagnostics(seed=20260311)
    second = diagnostics.run_diagnostics(seed=20260311)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_driver_bind_lifecycle_v1_report_contains_required_baseline_drivers():
    report = diagnostics.run_diagnostics(seed=20260311)
    assert report["schema"] == "rugo.native_driver_diagnostics_report.v1"
    assert report["contract_id"] == "rugo.native_driver_contract.v1"
    assert report["gate_pass"] is True

    for driver, profile in [
        ("virtio-gpu-pci", "modern"),
        ("virtio-scsi-pci", "modern"),
        ("e1000e", "baremetal"),
        ("usb-storage", "baremetal"),
    ]:
        row = _binding(report, driver, profile)
        assert row["status"] == "pass"
        assert "DRV: bind" in row["markers"]
        assert "IRQ: vector bound" in row["markers"]
        assert "DMA: map ok" in row["markers"]
        assert "probe_found" in row["states_observed"]
        assert "runtime_ok" in row["states_observed"]


def test_driver_bind_lifecycle_v1_detects_bind_regression():
    out = _out_path("native-driver-bind-fail.json")
    rc = diagnostics.main(
        [
            "--inject-failure",
            "bind_virtio_gpu",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    row = _binding(data, "virtio-gpu-pci", "modern")
    assert row["status"] == "fail"
    assert "bind_virtio_gpu" in data["failures"]
