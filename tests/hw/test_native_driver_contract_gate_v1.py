"""M53 aggregate gate: native-driver contract wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_driver_diagnostics_v1 as diagnostics  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m53" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_native_driver_contract_gate_v1_wiring_and_artifacts():
    required = [
        "docs/M53_EXECUTION_BACKLOG.md",
        "docs/hw/native_driver_contract_v1.md",
        "docs/hw/pcie_dma_contract_v1.md",
        "docs/hw/firmware_blob_policy_v1.md",
        "docs/hw/native_driver_diag_schema_v1.md",
        "tools/run_native_driver_diagnostics_v1.py",
        "tests/hw/test_native_driver_docs_v1.py",
        "tests/hw/test_pcie_dma_contract_v1.py",
        "tests/hw/test_firmware_blob_policy_v1.py",
        "tests/hw/test_driver_bind_lifecycle_v1.py",
        "tests/hw/test_irq_dma_policy_v1.py",
        "tests/hw/test_firmware_blob_enforcement_v1.py",
        "tests/hw/test_native_driver_diagnostics_v1.py",
        "tests/hw/test_native_driver_diag_gate_v1.py",
        "tests/hw/test_native_driver_contract_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M53 artifact: {rel}"

    roadmap = _read("docs/POST_G2_EXTENDED_MILESTONES.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M53_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-native-driver-contract-v1" in roadmap
    assert "test-native-driver-diagnostics-v1" in roadmap

    assert "test-native-driver-contract-v1" in makefile
    for entry in [
        "tools/run_native_driver_diagnostics_v1.py --out $(OUT)/native-driver-diagnostics-v1.json",
        "$(SUBMAKE) test-native-driver-diagnostics-v1",
        "tests/hw/test_native_driver_docs_v1.py",
        "tests/hw/test_pcie_dma_contract_v1.py",
        "tests/hw/test_firmware_blob_policy_v1.py",
        "tests/hw/test_driver_bind_lifecycle_v1.py",
        "tests/hw/test_irq_dma_policy_v1.py",
        "tests/hw/test_firmware_blob_enforcement_v1.py",
        "tests/hw/test_native_driver_diagnostics_v1.py",
        "tests/hw/test_native_driver_contract_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-native-driver-contract-v1.xml" in makefile
    assert "pytest-native-driver-diagnostics-v1.xml" in makefile

    assert "Native driver contract v1 gate" in ci
    assert "make test-native-driver-contract-v1" in ci
    assert "native-driver-contract-v1-artifacts" in ci
    assert "out/pytest-native-driver-contract-v1.xml" in ci
    assert "out/native-driver-diagnostics-v1.json" in ci

    assert "Native driver diagnostics v1 gate" in ci
    assert "make test-native-driver-diagnostics-v1" in ci
    assert "native-driver-diagnostics-v1-artifacts" in ci
    assert "out/pytest-native-driver-diagnostics-v1.xml" in ci

    assert "Status: done" in backlog
    assert "| M53 | Native Driver Contract Expansion v1 | n/a | done |" in milestones
    assert (
        "| **M53** Native Driver Contract Expansion v1 | n/a | done |" in status
    )
    assert "make test-native-driver-contract-v1" in readme
    assert "M43-M53" in readme

    out = _out_path("native-driver-contract-gate-v1.json")
    assert diagnostics.main(["--seed", "20260311", "--out", str(out)]) == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.native_driver_diagnostics_report.v1"
    assert data["contract_id"] == "rugo.native_driver_contract.v1"
    assert data["summary"]["bind"]["pass"] is True
    assert data["summary"]["firmware"]["pass"] is True
    assert data["gate_pass"] is True
