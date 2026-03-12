"""M54 PR-1: NVMe/AHCI contract and matrix doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m54_pr1_native_storage_artifacts_exist():
    required = [
        "docs/M54_EXECUTION_BACKLOG.md",
        "docs/hw/nvme_ahci_contract_v1.md",
        "docs/hw/support_matrix_v7.md",
        "docs/storage/block_flush_contract_v1.md",
        "tests/hw/test_nvme_ahci_docs_v1.py",
        "tests/storage/test_block_flush_contract_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M54 PR-1 artifact: {rel}"


def test_nvme_ahci_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/nvme_ahci_contract_v1.md")
    for token in [
        "Native storage contract ID: `rugo.nvme_ahci_contract.v1`.",
        "Parent native driver contract ID: `rugo.native_driver_contract.v1`.",
        "Driver lifecycle contract ID: `rugo.driver_lifecycle_report.v6`.",
        "Support matrix ID: `rugo.hw.support_matrix.v7`.",
        "Block flush contract ID: `rugo.block_flush_contract.v1`.",
        "`nvme`",
        "`ahci`",
        "`NVME: ready`",
        "`NVME: identify ok`",
        "`NVME: io queue ok`",
        "`AHCI: port up`",
        "`AHCI: rw ok`",
        "`AHCI: flush ok`",
        "`BLK: fua ok`",
        "`BLK: flush ordered`",
        "`NVME: namespace missing`",
        "`AHCI: port absent`",
        "Local gate: `make test-native-storage-v1`.",
        "Local sub-gate: `make test-hw-matrix-v7`.",
        "CI gate: `Native storage v1 gate`.",
        "CI sub-gate: `Hardware matrix v7 gate`.",
    ]:
        assert token in doc


def test_support_matrix_v7_doc_declares_required_tokens():
    doc = _read("docs/hw/support_matrix_v7.md")
    for token in [
        "Milestone: M54 Native Storage Drivers v1",
        "Tier 0",
        "Tier 1",
        "Tier 2",
        "Tier 3",
        "Tier 4",
        "Schema identifier: `rugo.hw_matrix_evidence.v7`",
        "Matrix contract ID: `rugo.hw.support_matrix.v7`",
        "Prior matrix contract ID: `rugo.hw.support_matrix.v6`",
        "Driver contract ID: `rugo.driver_lifecycle_report.v6`",
        "Native driver contract ID: `rugo.native_driver_contract.v1`",
        "Native storage contract ID: `rugo.nvme_ahci_contract.v1`",
        "Block flush contract ID: `rugo.block_flush_contract.v1`",
        "Local gate: `make test-hw-matrix-v7`.",
        "Local sub-gate: `make test-native-storage-v1`.",
        "CI gate: `Hardware matrix v7 gate`.",
        "CI sub-gate: `Native storage v1 gate`.",
        "Emulated NVMe is release-blocking",
    ]:
        assert token in doc


def test_m54_roadmap_anchor_declares_native_storage_gates():
    roadmap = _read("docs/POST_G2_EXTENDED_MILESTONES.md")
    assert "test-native-storage-v1" in roadmap
    assert "docs/hw/support_matrix_v7.md" in roadmap
    assert "docs/storage/block_flush_contract_v1.md" in roadmap
