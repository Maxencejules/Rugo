"""M53 PR-1: PCIe DMA contract doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_pcie_dma_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/pcie_dma_contract_v1.md")
    for token in [
        "PCIe DMA contract ID: `rugo.pcie_dma_contract.v1`.",
        "Parent native driver contract ID: `rugo.native_driver_contract.v1`.",
        "Parent lifecycle contract ID: `rugo.driver_lifecycle_report.v6`.",
        "Parent support matrix ID: `rugo.hw.support_matrix.v6`.",
        "`strict`",
        "`passthrough-shadow`",
        "`deny`",
        "`DMA: map ok`",
        "`DMA: map bounce`",
        "`DMA: deny unsafe`",
        "`IRQ: vector bound`",
        "fail-closed",
        "Software validation mandatory",
        "peer-to-peer DMA remains denied",
        "`tests/hw/test_irq_dma_policy_v1.py`",
    ]:
        assert token in doc
