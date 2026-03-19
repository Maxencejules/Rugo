"""X2 runtime-backed hardware qualification report checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_x2_hardware_runtime_v1 as tool  # noqa: E402
import x2_hardware_runtime_common_v1 as common  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-x2" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def _target(data: dict, target_id: str) -> dict:
    rows = [row for row in data["runtime_targets"] if row["target_id"] == target_id]
    assert len(rows) == 1
    return rows[0]


def test_x2_hardware_runtime_report_is_seed_deterministic():
    reports = common.collect_source_reports(seed=20260318)
    first = common.build_report(seed=20260318, reports=reports)
    second = common.build_report(seed=20260318, reports=reports)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_x2_hardware_runtime_report_schema_and_foundation():
    out = _out_path("x2-hardware-runtime-v1.json")
    rc = tool.main(["--seed", "20260318", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.x2_hardware_runtime_report.v1"
    assert data["track_id"] == "X2"
    assert data["policy_id"] == "rugo.x2_hardware_runtime_qualification.v1"
    assert data["device_registry_schema"] == "rugo.x2_device_registry.v1"
    assert data["gate"] == "test-x2-hardware-runtime-v1"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["summary"]["device_registry"]["pass"] is True
    assert data["summary"]["probe_bind"]["pass"] is True
    assert data["summary"]["firmware"]["pass"] is True
    assert data["summary"]["smp"]["pass"] is True
    assert data["summary"]["runtime_targets"]["qualified"] == 8
    assert data["summary"]["backlogs"]["runtime_backed"] == 8

    class_ids = {row["class_id"] for row in data["device_registry"]}
    for class_id in [
        "virtio-blk-pci-transitional",
        "virtio-net-pci-transitional",
        "ahci",
        "nvme",
        "virtio-blk-pci-modern",
        "virtio-net-pci-modern",
        "virtio-scsi-pci",
        "virtio-gpu-pci",
        "e1000e",
        "rtl8169",
        "xhci",
        "usb-hid",
        "usb-storage",
    ]:
        assert class_id in class_ids

    assert any(
        row["class_id"] == "virtio-gpu-pci"
        and row["bind_observed"] is True
        and "DRV: bind" in row["bind_markers"]
        for row in data["probe_bind_lifecycle"]
    )
    assert data["firmware_runtime"]["measured_boot"]["policy_pass"] is True
    assert data["firmware_runtime"]["firmware_blobs"]["policy_id"] == "rugo.firmware_blob_policy.v1"
    assert data["smp_runtime"]["interrupt_model_id"] == "rugo.smp_interrupt_model.v1"
    assert "SMP: affinity balanced" in data["smp_runtime"]["required_markers"]

    for target_id in [
        "qemu-q35-transitional",
        "qemu-i440fx-transitional",
        "qemu-q35-firmware-smp",
        "qemu-i440fx-firmware-smp",
        "qemu-q35-modern-virtio",
        "qemu-i440fx-modern-virtio",
        "intel-q470-e1000e-xhci",
        "amd-b550-rtl8169-xhci",
    ]:
        row = _target(data, target_id)
        assert row["qualification_pass"] is True
        assert row["marker_sequence_ok"] is True
        assert row["capture"]["capture_mode"] == "fixture"
        assert row["capture"]["serial_digest"]

    backlog_rows = {row["backlog"]: row for row in data["backlog_closure"]}
    for backlog_id in ["M9", "M15", "M23", "M37", "M43", "M45", "M46", "M47"]:
        assert backlog_rows[backlog_id]["runtime_class"] == "Runtime-backed"
        assert backlog_rows[backlog_id]["status"] == "pass"


def test_x2_hardware_runtime_report_detects_target_regression():
    out = _out_path("x2-hardware-runtime-v1-fail.json")
    rc = tool.main(
        [
            "--seed",
            "20260318",
            "--inject-failure",
            "target_intel_q470_e1000e_xhci",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "target_intel_q470_e1000e_xhci" in data["failures"]
    assert _target(data, "intel-q470-e1000e-xhci")["qualification_pass"] is False


def test_x2_hardware_runtime_report_rejects_unknown_check_id():
    out = _out_path("x2-hardware-runtime-v1-error.json")
    rc = tool.main(
        [
            "--inject-failure",
            "target_nonexistent_hardware_class",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
