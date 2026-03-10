"""M47 aggregate sub-gate: support-tier audit wiring and artifacts."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_hw_claim_promotion_v1 as promotion  # noqa: E402
import run_hw_support_tier_audit_v1 as audit  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m47" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_hw_support_tier_gate_v1_wiring_and_artifacts():
    required = [
        "docs/M47_EXECUTION_BACKLOG.md",
        "docs/hw/support_claim_policy_v1.md",
        "docs/hw/bare_metal_promotion_policy_v2.md",
        "docs/hw/support_tier_audit_v1.md",
        "tools/run_hw_claim_promotion_v1.py",
        "tools/run_hw_support_tier_audit_v1.py",
        "tests/hw/test_support_claim_docs_v1.py",
        "tests/hw/test_hw_claim_promotion_v1.py",
        "tests/hw/test_hw_support_tier_audit_v1.py",
        "tests/hw/test_hw_support_claim_negative_v1.py",
        "tests/hw/test_hw_support_tier_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M47 sub-gate artifact: {rel}"

    roadmap = _read("docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M47_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-hw-support-tier-audit-v1" in roadmap

    assert "test-hw-support-tier-audit-v1" in makefile
    for entry in [
        "tools/run_hw_claim_promotion_v1.py --out $(OUT)/hw-claim-promotion-v1.json",
        "tools/run_hw_support_tier_audit_v1.py --out $(OUT)/hw-support-tier-audit-v1.json",
        "tests/hw/test_support_claim_docs_v1.py",
        "tests/hw/test_hw_claim_promotion_v1.py",
        "tests/hw/test_hw_support_tier_audit_v1.py",
        "tests/hw/test_hw_support_claim_negative_v1.py",
        "tests/hw/test_hw_support_tier_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-hw-support-tier-audit-v1.xml" in makefile

    assert "Hardware support tier audit v1 gate" in ci
    assert "make test-hw-support-tier-audit-v1" in ci
    assert "hw-support-tier-audit-v1-artifacts" in ci
    assert "out/pytest-hw-support-tier-audit-v1.xml" in ci
    assert "out/hw-claim-promotion-v1.json" in ci
    assert "out/hw-support-tier-audit-v1.json" in ci

    assert "Status: done" in backlog
    assert "| M47 | Hardware Claim Promotion Program v1 | n/a | done |" in milestones
    assert "| **M47** Hardware Claim Promotion Program v1 | n/a | done |" in status
    assert "M47 execution backlog (completed)" in readme

    promotion_out = _out_path("hw-support-tier-claim-promotion-v1.json")
    audit_out = _out_path("hw-support-tier-audit-v1.json")

    assert promotion.main(["--seed", "20260310", "--out", str(promotion_out)]) == 0
    assert audit.main(["--seed", "20260310", "--out", str(audit_out)]) == 0

    promotion_data = json.loads(promotion_out.read_text(encoding="utf-8"))
    audit_data = json.loads(audit_out.read_text(encoding="utf-8"))

    assert promotion_data["schema"] == "rugo.hw_claim_promotion_report.v1"
    assert promotion_data["gate_pass"] is True
    assert audit_data["schema"] == "rugo.hw_support_tier_audit_report.v1"
    assert audit_data["gate_pass"] is True
