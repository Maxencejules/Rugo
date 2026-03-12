"""M54 aggregate gate: native storage wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_storage_diagnostics_v1 as diagnostics  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m54" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_native_storage_gate_v1_wiring_and_artifacts():
    required = [
        "docs/M54_EXECUTION_BACKLOG.md",
        "docs/hw/nvme_ahci_contract_v1.md",
        "docs/hw/support_matrix_v7.md",
        "docs/storage/block_flush_contract_v1.md",
        "tools/run_hw_matrix_v7.py",
        "tools/run_native_storage_diagnostics_v1.py",
        "tests/hw/test_nvme_ahci_docs_v1.py",
        "tests/storage/test_block_flush_contract_v1.py",
        "tests/hw/test_nvme_identify_v1.py",
        "tests/hw/test_nvme_io_queue_v1.py",
        "tests/hw/test_ahci_rw_v1.py",
        "tests/storage/test_nvme_fsync_integration_v1.py",
        "tests/hw/test_native_storage_negative_v1.py",
        "tests/hw/test_native_storage_gate_v1.py",
        "tests/hw/test_hw_gate_v7.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M54 artifact: {rel}"

    roadmap = _read("docs/POST_G2_EXTENDED_MILESTONES.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M54_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-native-storage-v1" in roadmap
    assert "test-hw-matrix-v7" in roadmap
    assert "docs/hw/nvme_ahci_contract_v1.md" in roadmap
    assert "docs/storage/block_flush_contract_v1.md" in roadmap

    assert "test-native-storage-v1" in makefile
    for entry in [
        "tools/run_native_storage_diagnostics_v1.py --out $(OUT)/native-storage-v1.json",
        "$(SUBMAKE) test-hw-matrix-v7",
        "tests/hw/test_nvme_ahci_docs_v1.py",
        "tests/storage/test_block_flush_contract_v1.py",
        "tests/hw/test_nvme_identify_v1.py",
        "tests/hw/test_nvme_io_queue_v1.py",
        "tests/hw/test_ahci_rw_v1.py",
        "tests/storage/test_nvme_fsync_integration_v1.py",
        "tests/hw/test_native_storage_negative_v1.py",
        "tests/hw/test_native_storage_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-native-storage-v1.xml" in makefile

    assert "Native storage v1 gate" in ci
    assert "make test-native-storage-v1" in ci
    assert "native-storage-v1-artifacts" in ci
    assert "out/pytest-native-storage-v1.xml" in ci
    assert "out/native-storage-v1.json" in ci
    assert "out/hw-matrix-v7.json" in ci

    assert "Status: done" in backlog
    assert "| M54 | Native Storage Drivers v1 | n/a | done |" in milestones
    assert "| **M54** Native Storage Drivers v1 | n/a | done |" in status
    assert "make test-native-storage-v1" in readme
    assert "M43-M54" in readme

    out = _out_path("native-storage-gate-v1.json")
    assert diagnostics.main(["--seed", "20260312", "--out", str(out)]) == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.native_storage_diagnostics_report.v1"
    assert data["contract_id"] == "rugo.nvme_ahci_contract.v1"
    assert data["summary"]["nvme"]["pass"] is True
    assert data["summary"]["ahci"]["pass"] is True
    assert data["summary"]["flush"]["pass"] is True
    assert data["gate_pass"] is True
