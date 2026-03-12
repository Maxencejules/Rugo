"""M54 PR-2: native storage fsync integration checks."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))
sys.path.append(str(Path(__file__).resolve().parent))

import run_native_storage_diagnostics_v1 as diagnostics  # noqa: E402
from native_storage_v1_model import NativeStorageDurabilityModel  # noqa: E402


def _flush(data: dict, audit_id: str) -> dict:
    rows = [entry for entry in data["flush_audits"] if entry["audit_id"] == audit_id]
    assert len(rows) == 1
    return rows[0]


def test_nvme_fsync_integration_v1_names_native_device_class():
    model = NativeStorageDurabilityModel(controller="nvme", fua_supported=True)
    model.write_data()
    model.write_metadata()
    model.fua_commit()
    assert model.crash() == (True, True)

    data = diagnostics.run_diagnostics(seed=20260312)
    bridge = data["durability_bridge"]
    fsync = _flush(data, "nvme_fsync_bridge")

    assert bridge["block_flush_contract_id"] == "rugo.block_flush_contract.v1"
    assert bridge["fsync_device_class"] == "nvme"
    assert bridge["required_markers"] == ["BLK: fua ok", "BLK: flush ordered"]

    assert fsync["command"] == "fsync"
    assert fsync["marker"] == "BLK: fua ok"
    assert fsync["data_durable"] is True
    assert fsync["metadata_durable"] is True
    assert fsync["status"] == "pass"
