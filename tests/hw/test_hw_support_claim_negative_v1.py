"""M47 PR-2: negative-path checks for support claim promotion and audit."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_hw_claim_promotion_v1 as promotion  # noqa: E402
import run_hw_support_tier_audit_v1 as audit  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m47" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_hw_claim_promotion_v1_detects_missing_artifact():
    out = _out_path("hw-claim-promotion-v1-missing-artifact.json")
    rc = promotion.main(
        [
            "--inject-missing-artifact",
            "matrix_report",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "matrix_report" in data["missing_artifacts"]
    assert "artifact_bundle_complete" in data["failures"]


def test_hw_support_tier_audit_v1_detects_tier_drift():
    out = _out_path("hw-support-tier-audit-v1-tier-drift.json")
    rc = audit.main(
        [
            "--inject-tier-drift",
            "virtio-gpu-pci=tier2",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    checks = {check["check_id"]: check["pass"] for check in data["checks"]}
    assert data["gate_pass"] is False
    assert "virtio-gpu-pci" in data["drifted_claims"]
    assert checks["tier_assignments_match_policy"] is False


def test_hw_support_tier_audit_v1_detects_missing_history():
    out = _out_path("hw-support-tier-audit-v1-missing-history.json")
    rc = audit.main(
        [
            "--drop-history",
            "virtio-gpu-pci",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    checks = {check["check_id"]: check["pass"] for check in data["checks"]}
    assert "virtio-gpu-pci" in data["missing_history_claims"]
    assert checks["promotion_history_traceable"] is False


def test_hw_support_tier_audit_v1_detects_unsupported_claim():
    out = _out_path("hw-support-tier-audit-v1-unsupported-claim.json")
    rc = audit.main(
        [
            "--inject-unsupported-claim",
            "wifi",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    checks = {check["check_id"]: check["pass"] for check in data["checks"]}
    assert "wifi" in data["unsupported_promoted_claims"]
    assert checks["unsupported_classes_not_promoted"] is False


def test_hw_support_tier_audit_v1_rejects_unknown_tier_override():
    out = _out_path("hw-support-tier-audit-v1-error.json")
    rc = audit.main(
        [
            "--inject-tier-drift",
            "virtio-gpu-pci=tier9",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
