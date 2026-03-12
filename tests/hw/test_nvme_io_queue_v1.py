"""M54 PR-2: NVMe queue setup and completion diagnostics checks."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_storage_diagnostics_v1 as diagnostics  # noqa: E402


def _audit(data: dict, audit_id: str) -> dict:
    rows = [entry for entry in data["queue_audits"] if entry["audit_id"] == audit_id]
    assert len(rows) == 1
    return rows[0]


def test_nvme_io_queue_v1_has_stable_admin_and_io_audits():
    data = diagnostics.run_diagnostics(seed=20260312)
    admin = _audit(data, "nvme_admin_identify")
    io = _audit(data, "nvme_io_submission")

    assert admin["queue_kind"] == "admin"
    assert admin["depth"] == 32
    assert admin["irq_mode"] == "msix"
    assert admin["marker"] == "NVME: identify ok"
    assert admin["status"] == "pass"

    assert io["queue_kind"] == "io"
    assert io["depth"] == 64
    assert io["irq_mode"] == "msix"
    assert io["fua_supported"] is True
    assert io["marker"] == "NVME: io queue ok"
    assert io["status"] == "pass"

    assert data["summary"]["nvme"]["pass"] is True
