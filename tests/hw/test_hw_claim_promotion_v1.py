"""M47 PR-2: deterministic hardware claim promotion checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_hw_claim_promotion_v1 as promotion  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m47" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_hw_claim_promotion_v1_is_seed_deterministic():
    first = promotion.run_claim_promotion(seed=20260310)
    second = promotion.run_claim_promotion(seed=20260310)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_hw_claim_promotion_v1_schema_and_claims():
    out = _out_path("hw-claim-promotion-v1.json")
    rc = promotion.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.hw_claim_promotion_report.v1"
    assert data["policy_id"] == "rugo.hw_support_claim_policy.v1"
    assert data["baremetal_promotion_policy_id"] == "rugo.hw_baremetal_promotion_policy.v2"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["support_tier_summary"]["tier1"]["promoted_claims"] == 4
    assert data["support_tier_summary"]["tier2"]["promoted_claims"] == 5
    assert data["missing_artifacts"] == []
    claim_index = {row["class_id"]: row for row in data["claims"]}
    assert claim_index["virtio-gpu-pci"]["claim_status"] == "promoted"
    assert claim_index["virtio-gpu-pci"]["support_tier"] == "tier1"
    assert claim_index["virtio-gpu-pci"]["desktop_bound"] is True
    assert claim_index["usb-storage"]["claim_status"] == "promoted"
    assert claim_index["usb-storage"]["support_tier"] == "tier2"
    assert claim_index["usb-storage"]["recovery_bound"] is True
    assert "wifi" in data["unsupported_class_registry"]
