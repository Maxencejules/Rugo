"""M54 PR-2: AHCI port, DMA rw, and flush diagnostics checks."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_storage_diagnostics_v1 as diagnostics  # noqa: E402


def _controller(data: dict, driver: str) -> dict:
    rows = [entry for entry in data["controllers"] if entry["driver"] == driver]
    assert len(rows) == 1
    return rows[0]


def _flush(data: dict, audit_id: str) -> dict:
    rows = [entry for entry in data["flush_audits"] if entry["audit_id"] == audit_id]
    assert len(rows) == 1
    return rows[0]


def test_ahci_rw_v1_has_port_link_dma_and_flush_markers():
    data = diagnostics.run_diagnostics(seed=20260312)
    ahci = _controller(data, "ahci")
    flush = _flush(data, "ahci_cache_flush")

    assert ahci["ports"][0]["port"] == 0
    assert ahci["ports"][0]["link_state"] == "up"
    assert ahci["ports"][0]["ncq_depth"] == 32
    assert "AHCI: port up" in ahci["markers"]
    assert "AHCI: rw ok" in ahci["markers"]
    assert "AHCI: flush ok" in ahci["markers"]

    assert flush["command"] == "cache_flush"
    assert flush["marker"] == "AHCI: flush ok"
    assert flush["status"] == "pass"
    assert data["summary"]["ahci"]["pass"] is True
