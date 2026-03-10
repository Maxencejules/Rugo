"""M47 PR-1: support claim policy and audit contract doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m47_pr1_support_claim_artifacts_exist():
    required = [
        "docs/M47_EXECUTION_BACKLOG.md",
        "docs/hw/support_claim_policy_v1.md",
        "docs/hw/bare_metal_promotion_policy_v2.md",
        "docs/hw/support_tier_audit_v1.md",
        "tests/hw/test_support_claim_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M47 PR-1 artifact: {rel}"


def test_support_claim_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/support_claim_policy_v1.md")
    for token in [
        "Milestone: M47 Hardware Claim Promotion Program v1",
        "Policy identifier: `rugo.hw_support_claim_policy.v1`.",
        "Claim promotion report schema: `rugo.hw_claim_promotion_report.v1`.",
        "Support-tier audit report schema: `rugo.hw_support_tier_audit_report.v1`.",
        "Matrix evidence schema: `rugo.hw_matrix_evidence.v6`.",
        "Bare-metal I/O evidence schema: `rugo.baremetal_io_baseline.v1`.",
        "Bare-metal promotion input schema: `rugo.hw_baremetal_promotion_report.v2`.",
        "`virtio-blk-pci` modern",
        "`virtio-net-pci` modern",
        "`virtio-scsi-pci`",
        "`virtio-gpu-pci`",
        "`e1000e`",
        "`rtl8169`",
        "`xhci`",
        "`usb-hid`",
        "`usb-storage`",
        "Local gate: `make test-hw-claim-promotion-v1`.",
        "Local sub-gate: `make test-hw-support-tier-audit-v1`.",
        "CI gate: `Hardware claim promotion v1 gate`.",
        "CI sub-gate: `Hardware support tier audit v1 gate`.",
    ]:
        assert token in doc


def test_bare_metal_promotion_policy_v2_doc_declares_required_tokens():
    doc = _read("docs/hw/bare_metal_promotion_policy_v2.md")
    for token in [
        "Policy identifier: `rugo.hw_baremetal_promotion_policy.v2`.",
        "Input report schema: `rugo.hw_baremetal_promotion_report.v2`.",
        "Support claim policy: `rugo.hw_support_claim_policy.v1`.",
        "Claim promotion report schema: `rugo.hw_claim_promotion_report.v1`.",
        "Minimum consecutive green runs: `12`.",
        "Minimum campaign pass rate: `0.98`.",
        "Maximum tolerated fatal lifecycle errors: `0`.",
        "Maximum tolerated deterministic negative-path violations: `0`.",
        "`intel_q470_e1000e_xhci`",
        "`amd_b550_rtl8169_xhci`",
        "`out/hw-promotion-v2.json`",
        "`out/hw-claim-promotion-v1.json`",
    ]:
        assert token in doc


def test_support_tier_audit_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/support_tier_audit_v1.md")
    for token in [
        "Audit identifier: `rugo.hw_support_tier_audit.v1`.",
        "Report schema: `rugo.hw_support_tier_audit_report.v1`.",
        "Claim policy: `rugo.hw_support_claim_policy.v1`.",
        "Bare-metal promotion policy: `rugo.hw_baremetal_promotion_policy.v2`.",
        "`class_id`",
        "`support_tier`",
        "`claim_status`",
        "`policy_id`",
        "`promotion_policy_id`",
        "`promotion_history`",
        "`virtio-gpu-pci` -> `tier1`",
        "`usb-storage` -> `tier2`",
        "`wifi`",
        "Local sub-gate: `make test-hw-support-tier-audit-v1`.",
        "CI sub-gate: `Hardware support tier audit v1 gate`.",
    ]:
        assert token in doc


def test_m47_roadmap_anchor_declares_claim_gates():
    roadmap = _read("docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md")
    assert "test-hw-claim-promotion-v1" in roadmap
    assert "test-hw-support-tier-audit-v1" in roadmap
