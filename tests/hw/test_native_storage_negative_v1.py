"""M54 PR-2: native storage negative-path diagnostics checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_storage_diagnostics_v1 as diagnostics  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m54" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_native_storage_negative_v1_detects_missing_namespace_regression():
    out = _out_path("native-storage-negative-v1.json")
    rc = diagnostics.main(
        [
            "--inject-failure",
            "negative_namespace_missing",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["negative_path"]["failures"] == 1
    assert data["negative_paths"]["nvme_missing_namespace"]["marker"] == "NVME: namespace missing"
    assert data["negative_paths"]["nvme_missing_namespace"]["status"] == "fail"


def test_native_storage_negative_v1_rejects_unknown_check_id():
    out = _out_path("native-storage-negative-v1-error.json")
    rc = diagnostics.main(
        [
            "--inject-failure",
            "native_storage_nonexistent_check",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
