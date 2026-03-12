"""M53 PR-1: firmware blob policy doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_firmware_blob_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/firmware_blob_policy_v1.md")
    for token in [
        "Firmware blob policy ID: `rugo.firmware_blob_policy.v1`.",
        "Parent native driver contract ID: `rugo.native_driver_contract.v1`.",
        "Parent PCIe DMA contract ID: `rugo.pcie_dma_contract.v1`.",
        "Diagnostics schema ID: `rugo.native_driver_diag_schema.v1`.",
        "Firmware manifest schema: `rugo.firmware_manifest.v1`.",
        "`FW: allow signed`",
        "`FW: denied unsigned`",
        "`FW: denied missing manifest`",
        "`FW: denied hash mismatch`",
        "firmware remains outside the base kernel image",
        "measured-boot reference required",
        "`tests/hw/test_firmware_blob_enforcement_v1.py`",
    ]:
        assert token in doc
