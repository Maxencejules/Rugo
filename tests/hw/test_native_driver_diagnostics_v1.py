"""M53 PR-2: native-driver diagnostics report checks."""

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


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m53" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_native_driver_diagnostics_v1_report_is_seed_deterministic():
    first = diagnostics.run_diagnostics(seed=20260311)
    second = diagnostics.run_diagnostics(seed=20260311)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_native_driver_diagnostics_v1_schema_and_markers():
    out = _out_path("native-driver-diagnostics-v1.json")
    rc = diagnostics.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.native_driver_diagnostics_report.v1"
    assert data["contract_id"] == "rugo.native_driver_contract.v1"
    assert data["pcie_dma_contract_id"] == "rugo.pcie_dma_contract.v1"
    assert data["firmware_blob_policy_id"] == "rugo.firmware_blob_policy.v1"
    assert data["gate"] == "test-native-driver-diagnostics-v1"
    assert data["contract_gate"] == "test-native-driver-contract-v1"
    assert data["source_reports"]["matrix_v6"]["schema"] == "rugo.hw_matrix_evidence.v6"
    assert data["source_reports"]["baremetal_io"]["schema"] == "rugo.baremetal_io_baseline.v1"
    assert data["artifact_refs"]["diagnostics_report"] == "out/native-driver-diagnostics-v1.json"
    assert data["gate_pass"] is True

    markers = {entry["marker"] for entry in data["diagnostic_events"]}
    for marker in [
        "DRV: bind",
        "IRQ: vector bound",
        "DMA: map ok",
        "DMA: map bounce",
        "DMA: deny unsafe",
        "FW: denied unsigned",
    ]:
        assert marker in markers

    for event in data["diagnostic_events"]:
        for field in [
            "event_id",
            "driver",
            "device_class",
            "profile",
            "phase",
            "severity",
            "marker",
            "status",
            "details",
        ]:
            assert field in event


def test_native_driver_diagnostics_v1_rejects_unknown_check_id():
    out = _out_path("native-driver-diagnostics-v1-error.json")
    rc = diagnostics.main(
        [
            "--inject-failure",
            "native_driver_nonexistent_check",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
