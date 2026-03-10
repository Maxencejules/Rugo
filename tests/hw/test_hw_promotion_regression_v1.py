"""M47 PR-2: regression coverage for hardware claim promotion."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_hw_claim_promotion_v1 as promotion  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m47" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_hw_claim_promotion_v1_detects_matrix_regression():
    out = _out_path("hw-claim-promotion-v1-matrix-regression.json")
    rc = promotion.main(
        [
            "--inject-matrix-failure",
            "desktop_display_bridge",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    claim_index = {row["class_id"]: row for row in data["claims"]}
    assert data["gate_pass"] is False
    assert claim_index["virtio-gpu-pci"]["claim_status"] == "evidence_only"
    assert "matrix_claim_bundle_green" in data["failures"]
    assert "matrix_targets_promoted" in data["failures"]


def test_hw_claim_promotion_v1_detects_baremetal_regression():
    out = _out_path("hw-claim-promotion-v1-baremetal-regression.json")
    rc = promotion.main(
        [
            "--inject-baremetal-failure",
            "recovery_media_bootstrap",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    claim_index = {row["class_id"]: row for row in data["claims"]}
    assert data["gate_pass"] is False
    assert claim_index["usb-storage"]["claim_status"] == "evidence_only"
    assert "baremetal_claim_bundle_green" in data["failures"]
    assert "baremetal_targets_promoted" in data["failures"]
