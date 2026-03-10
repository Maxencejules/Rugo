"""M47 PR-2: deterministic support-tier audit checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_hw_support_tier_audit_v1 as audit  # noqa: E402


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


def test_hw_support_tier_audit_v1_is_seed_deterministic():
    first = audit.run_audit(seed=20260310)
    second = audit.run_audit(seed=20260310)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_hw_support_tier_audit_v1_schema_and_gate_pass():
    out = _out_path("hw-support-tier-audit-v1.json")
    rc = audit.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.hw_support_tier_audit_report.v1"
    assert data["audit_id"] == "rugo.hw_support_tier_audit.v1"
    assert data["claim_policy_id"] == "rugo.hw_support_claim_policy.v1"
    assert data["gate_pass"] is True
    assert data["claim_report_gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["observed_tier_summary"]["tier1"]["promoted_claims"] == 4
    assert data["observed_tier_summary"]["tier2"]["promoted_claims"] == 5
    checks = {check["check_id"]: check["pass"] for check in data["checks"]}
    assert checks["tier_assignments_match_policy"] is True
    assert checks["promotion_history_traceable"] is True
    assert checks["unsupported_classes_not_promoted"] is True
