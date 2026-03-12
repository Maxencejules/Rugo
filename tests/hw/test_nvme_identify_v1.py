"""M54 PR-2: NVMe identify and namespace diagnostics checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_storage_diagnostics_v1 as diagnostics  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m54" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def _controller(data: dict, driver: str) -> dict:
    rows = [entry for entry in data["controllers"] if entry["driver"] == driver]
    assert len(rows) == 1
    return rows[0]


def test_native_storage_diagnostics_v1_report_is_seed_deterministic():
    first = diagnostics.run_diagnostics(seed=20260312)
    second = diagnostics.run_diagnostics(seed=20260312)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_nvme_identify_v1_schema_and_namespace_coverage():
    out = _out_path("native-storage-identify-v1.json")
    rc = diagnostics.main(["--seed", "20260312", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    nvme = _controller(data, "nvme")

    assert data["schema"] == "rugo.native_storage_diagnostics_report.v1"
    assert data["contract_id"] == "rugo.nvme_ahci_contract.v1"
    assert data["block_flush_contract_id"] == "rugo.block_flush_contract.v1"
    assert data["source_reports"]["matrix_v7"]["schema"] == "rugo.hw_matrix_evidence.v7"
    assert data["artifact_refs"]["diagnostics_report"] == "out/native-storage-v1.json"
    assert nvme["namespaces"][0]["nsid"] == 1
    assert nvme["namespaces"][0]["lba_bytes"] == 4096
    assert nvme["namespaces"][0]["size_gib"] == 64
    assert "NVME: ready" in nvme["markers"]
    assert "NVME: identify ok" in nvme["markers"]
    assert data["gate_pass"] is True
