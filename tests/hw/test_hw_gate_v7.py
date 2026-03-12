"""M54 aggregate sub-gate: hardware matrix v7 wiring and artifacts."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_hw_matrix_v7 as matrix  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m54" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def _coverage_entry(data: dict, device: str) -> dict:
    rows = [entry for entry in data["device_class_coverage"] if entry["device"] == device]
    assert len(rows) == 1
    return rows[0]


def test_hw_matrix_v7_gate_wiring_and_artifacts():
    required = [
        "docs/M54_EXECUTION_BACKLOG.md",
        "docs/hw/nvme_ahci_contract_v1.md",
        "docs/hw/support_matrix_v7.md",
        "docs/storage/block_flush_contract_v1.md",
        "tools/run_hw_matrix_v7.py",
        "tests/hw/test_nvme_ahci_docs_v1.py",
        "tests/hw/test_nvme_identify_v1.py",
        "tests/hw/test_nvme_io_queue_v1.py",
        "tests/hw/test_ahci_rw_v1.py",
        "tests/hw/test_native_storage_negative_v1.py",
        "tests/storage/test_nvme_fsync_integration_v1.py",
        "tests/hw/test_hw_gate_v7.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M54 sub-gate artifact: {rel}"

    roadmap = _read("docs/POST_G2_EXTENDED_MILESTONES.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M54_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-hw-matrix-v7" in roadmap

    assert "test-hw-matrix-v7" in makefile
    for entry in [
        "tools/run_hw_matrix_v7.py --out $(OUT)/hw-matrix-v7.json",
        "tests/hw/test_nvme_ahci_docs_v1.py",
        "tests/storage/test_block_flush_contract_v1.py",
        "tests/hw/test_nvme_identify_v1.py",
        "tests/hw/test_nvme_io_queue_v1.py",
        "tests/hw/test_ahci_rw_v1.py",
        "tests/storage/test_nvme_fsync_integration_v1.py",
        "tests/hw/test_native_storage_negative_v1.py",
        "tests/hw/test_hw_gate_v7.py",
    ]:
        assert entry in makefile
    assert "pytest-hw-matrix-v7.xml" in makefile

    assert "Hardware matrix v7 gate" in ci
    assert "make test-hw-matrix-v7" in ci
    assert "hw-matrix-v7-artifacts" in ci
    assert "out/pytest-hw-matrix-v7.xml" in ci
    assert "out/hw-matrix-v7.json" in ci

    assert "Status: done" in backlog
    assert "| M54 | Native Storage Drivers v1 | n/a | done |" in milestones
    assert "| **M54** Native Storage Drivers v1 | n/a | done |" in status
    assert "make test-hw-matrix-v7" in readme

    out = _out_path("hw-matrix-v7.json")
    assert matrix.main(["--seed", "20260312", "--out", str(out)]) == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.hw_matrix_evidence.v7"
    assert data["matrix_contract_id"] == "rugo.hw.support_matrix.v7"
    assert data["native_storage_contract_id"] == "rugo.nvme_ahci_contract.v1"
    assert data["block_flush_contract_id"] == "rugo.block_flush_contract.v1"
    assert data["source_reports"]["matrix_v6"]["schema"] == "rugo.hw_matrix_evidence.v6"
    assert data["flush_contract_checks"]["status"] == "pass"
    assert _coverage_entry(data, "nvme")["status"] == "pass"
    assert _coverage_entry(data, "ahci")["status"] == "pass"
    assert data["gate_pass"] is True
