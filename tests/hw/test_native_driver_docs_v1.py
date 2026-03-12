"""M53 PR-1: native-driver contract and diagnostics schema doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m53_pr1_native_driver_artifacts_exist():
    required = [
        "docs/M53_EXECUTION_BACKLOG.md",
        "docs/hw/native_driver_contract_v1.md",
        "docs/hw/pcie_dma_contract_v1.md",
        "docs/hw/firmware_blob_policy_v1.md",
        "docs/hw/native_driver_diag_schema_v1.md",
        "tests/hw/test_native_driver_docs_v1.py",
        "tests/hw/test_pcie_dma_contract_v1.py",
        "tests/hw/test_firmware_blob_policy_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M53 PR-1 artifact: {rel}"


def test_native_driver_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/native_driver_contract_v1.md")
    for token in [
        "Native driver contract ID: `rugo.native_driver_contract.v1`.",
        "Parent lifecycle contract ID: `rugo.driver_lifecycle_report.v6`.",
        "Parent support matrix ID: `rugo.hw.support_matrix.v6`.",
        "PCIe DMA contract ID: `rugo.pcie_dma_contract.v1`.",
        "Firmware blob policy ID: `rugo.firmware_blob_policy.v1`.",
        "Diagnostics schema ID: `rugo.native_driver_diag_schema.v1`.",
        "`nvme`",
        "`ahci`",
        "`native-gpu`",
        "`wifi-pcie`",
        "`DRV: bind`",
        "`IRQ: vector bound`",
        "`DMA: map ok`",
        "`FW: denied unsigned`",
        "Local gate: `make test-native-driver-contract-v1`.",
        "Local sub-gate: `make test-native-driver-diagnostics-v1`.",
        "CI gate: `Native driver contract v1 gate`.",
        "CI sub-gate: `Native driver diagnostics v1 gate`.",
    ]:
        assert token in doc


def test_native_driver_diag_schema_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/native_driver_diag_schema_v1.md")
    for token in [
        "Schema identifier: `rugo.native_driver_diagnostics_report.v1`",
        "Parent contract ID: `rugo.native_driver_contract.v1`.",
        "PCIe DMA contract ID: `rugo.pcie_dma_contract.v1`.",
        "Firmware blob policy ID: `rugo.firmware_blob_policy.v1`.",
        "Lifecycle contract ID: `rugo.driver_lifecycle_report.v6`.",
        "Support matrix ID: `rugo.hw.support_matrix.v6`.",
        "`driver_bindings`",
        "`irq_audits`",
        "`dma_policy`",
        "`firmware_policy`",
        "`diagnostic_events`",
        "`DRV: bind`",
        "`DMA: deny unsafe`",
        "`FW: denied hash mismatch`",
    ]:
        assert token in doc


def test_m53_roadmap_anchor_declares_native_driver_gates():
    roadmap = _read("docs/POST_G2_EXTENDED_MILESTONES.md")
    assert "test-native-driver-contract-v1" in roadmap
    assert "docs/hw/native_driver_diag_schema_v1.md" in roadmap
