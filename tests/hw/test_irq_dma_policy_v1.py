"""M53 PR-2: deterministic IRQ and DMA policy checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_driver_diagnostics_v1 as diagnostics  # noqa: E402


def _dma_audit(data: dict, audit_id: str) -> dict:
    rows = [entry for entry in data["dma_audits"] if entry["audit_id"] == audit_id]
    assert len(rows) == 1
    return rows[0]


def _irq_audit(data: dict, driver: str, profile: str) -> dict:
    rows = [
        entry
        for entry in data["irq_audits"]
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


def test_irq_dma_policy_v1_report_passes_strict_contract():
    report = diagnostics.run_diagnostics(seed=20260311)
    assert report["summary"]["irq_dma"]["pass"] is True
    assert report["dma_policy"]["contract_id"] == "rugo.pcie_dma_contract.v1"
    assert report["dma_policy"]["iommu_mode"] == "strict"
    assert report["dma_policy"]["peer_to_peer_dma_allowed"] is False

    assert _irq_audit(report, "virtio-gpu-pci", "modern")["marker"] == "IRQ: vector bound"
    assert _irq_audit(report, "e1000e", "baremetal")["status"] == "pass"
    assert _dma_audit(report, "nvme_admin_submission_queue")["marker"] == "DMA: map ok"
    assert _dma_audit(report, "gpu_ring_bounce_window")["marker"] == "DMA: map bounce"
    unsafe = _dma_audit(report, "wifi_peer_to_peer_denial")
    assert unsafe["marker"] == "DMA: deny unsafe"
    assert unsafe["status"] == "pass"


def test_irq_dma_policy_v1_detects_irq_regression():
    out = _out_path("native-driver-irq-fail.json")
    rc = diagnostics.main(
        [
            "--inject-failure",
            "irq_vector_policy",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert _irq_audit(data, "virtio-gpu-pci", "modern")["status"] == "fail"
    assert "irq_vector_policy" in data["failures"]


def test_irq_dma_policy_v1_detects_unsafe_dma_regression():
    out = _out_path("native-driver-dma-fail.json")
    rc = diagnostics.main(
        [
            "--inject-failure",
            "dma_unsafe_denied",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    unsafe = _dma_audit(data, "wifi_peer_to_peer_denial")
    assert unsafe["status"] == "fail"
    assert "dma_unsafe_denied" in data["failures"]
