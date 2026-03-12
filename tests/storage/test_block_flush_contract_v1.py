"""M54 PR-1: block flush contract doc and model checks."""

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(Path(__file__).resolve().parent))

from native_storage_v1_model import NativeStorageDurabilityModel  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_block_flush_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/storage/block_flush_contract_v1.md")
    for token in [
        "Block flush contract ID: `rugo.block_flush_contract.v1`.",
        "Parent storage contract: `docs/storage/fs_v1.md`.",
        "Parent native storage contract ID: `rugo.nvme_ahci_contract.v1`.",
        "Support matrix ID: `rugo.hw.support_matrix.v7`.",
        "`BLK: fua ok`",
        "`BLK: flush ordered`",
        "`BLK: flush timeout`",
        "`NVME: ready`",
        "`AHCI: port up`",
        "`device_class`",
        "`command`",
        "`latency_ms`",
        "Local gate: `make test-native-storage-v1`.",
        "Local sub-gate: `make test-hw-matrix-v7`.",
    ]:
        assert token in doc


def test_nvme_fua_commit_persists_pending_data_and_metadata():
    model = NativeStorageDurabilityModel(controller="nvme", fua_supported=True)
    model.write_data()
    model.write_metadata()
    model.fua_commit()
    assert model.crash() == (True, True)


def test_ahci_cache_flush_persists_pending_data_and_metadata():
    model = NativeStorageDurabilityModel(controller="ahci", fua_supported=False)
    model.write_data()
    model.write_metadata()
    model.cache_flush()
    assert model.crash() == (True, True)


def test_crash_before_flush_loses_pending_data_and_metadata():
    model = NativeStorageDurabilityModel(controller="ahci", fua_supported=False)
    model.write_data()
    model.write_metadata()
    assert model.crash() == (False, False)
