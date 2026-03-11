"""M48 PR-2: deterministic virtio-gpu scanout checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_display_runtime_v1 as runtime  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m48" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_virtio_gpu_scanout_v1_schema_and_pass():
    out = _out_path("display-runtime-v1-virtio.json")
    rc = runtime.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.display_runtime_report.v1"
    assert data["contract_id"] == "rugo.display_runtime_contract.v1"
    assert data["active_runtime_path"] == "virtio-gpu-pci"
    assert data["policy_decision"] == "primary"
    assert data["primary_runtime"]["display_class"] == "virtio-gpu-pci"
    assert data["primary_runtime"]["driver"] == "virtio_gpu_scanout"
    assert data["primary_runtime"]["declared_support_source_schema"] == "rugo.hw_matrix_evidence.v6"
    assert data["primary_runtime"]["declared_support_pass"] is True
    assert data["primary_runtime"]["checks_pass"] is True
    assert data["primary_runtime"]["frame_drop_ratio"] <= 0.005
    assert data["summary"]["scanout"]["pass"] is True
    assert data["gate_pass"] is True


def test_virtio_gpu_scanout_v1_detects_primary_scanout_regression():
    out = _out_path("display-runtime-v1-virtio-fail.json")
    rc = runtime.main(
        [
            "--inject-failure",
            "virtio_gpu_scanout",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "virtio_gpu_scanout" in data["failures"]
    assert data["summary"]["scanout"]["failures"] >= 1
    assert data["primary_runtime"]["checks_pass"] is False
    assert data["fallback_runtime"]["checks_pass"] is True
