"""M38 PR-1: storage/platform feature contract doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m38_pr1_storage_feature_contract_artifacts_exist():
    required = [
        "docs/M38_EXECUTION_BACKLOG.md",
        "docs/storage/fs_feature_contract_v1.md",
        "docs/storage/snapshot_policy_v1.md",
        "docs/storage/online_resize_policy_v1.md",
        "docs/runtime/platform_feature_profile_v1.md",
        "tests/storage/test_storage_feature_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M38 PR-1 artifact: {rel}"


def test_fs_feature_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/storage/fs_feature_contract_v1.md")
    for token in [
        "Storage feature contract ID: `rugo.storage_feature_contract.v1`.",
        "Snapshot policy ID: `rugo.snapshot_policy.v1`.",
        "Online resize policy ID: `rugo.online_resize_policy.v1`.",
        "Platform profile ID: `rugo.platform_feature_profile.v1`.",
        "Feature campaign schema: `rugo.storage_feature_campaign_report.v1`.",
        "Platform conformance schema: `rugo.platform_feature_conformance_report.v1`.",
        "Local gate: `make test-storage-platform-v1`.",
        "Local sub-gate: `make test-storage-feature-contract-v1`.",
        "CI gate: `Storage platform v1 gate`.",
        "CI sub-gate: `Storage feature contract v1 gate`.",
    ]:
        assert token in doc


def test_snapshot_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/storage/snapshot_policy_v1.md")
    for token in [
        "Policy identifier: `rugo.snapshot_policy.v1`.",
        "Parent storage feature contract: `rugo.storage_feature_contract.v1`.",
        "Feature campaign schema: `rugo.storage_feature_campaign_report.v1`.",
        "Snapshot create latency: `<= 80 ms`.",
        "Snapshot restore integrity ratio: `>= 1.0`.",
        "Snapshot retention policy violations: `0`.",
        "Snapshot orphaned metadata count: `0`.",
    ]:
        assert token in doc


def test_online_resize_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/storage/online_resize_policy_v1.md")
    for token in [
        "Policy identifier: `rugo.online_resize_policy.v1`.",
        "Parent storage feature contract: `rugo.storage_feature_contract.v1`.",
        "Feature campaign schema: `rugo.storage_feature_campaign_report.v1`.",
        "Online grow completion latency: `<= 120 ms`.",
        "Capacity reconcile mismatch count: `0`.",
        "Shrink guard enforcement ratio: `>= 1.0`.",
        "Post-resize fsck error count: `0`.",
    ]:
        assert token in doc


def test_platform_feature_profile_v1_doc_declares_required_tokens():
    doc = _read("docs/runtime/platform_feature_profile_v1.md")
    for token in [
        "Platform conformance policy ID: `rugo.platform_feature_profile.v1`",
        "Platform conformance report schema: `rugo.platform_feature_conformance_report.v1`",
        "Profile requirement schema: `rugo.platform_feature_requirement_set.v1`",
        "Parent storage feature contract ID: `rugo.storage_feature_contract.v1`",
        "`server_storage_dense_v1`",
        "`edge_resilient_v1`",
        "`dev_workstation_v1`",
        "Local gate: `make test-storage-platform-v1`",
        "Local sub-gate: `make test-storage-feature-contract-v1`",
        "CI gate: `Storage platform v1 gate`",
        "CI sub-gate: `Storage feature contract v1 gate`",
    ]:
        assert token in doc


def test_m35_m39_roadmap_anchor_declares_m38_gates():
    roadmap = _read("docs/M35_M39_GENERAL_PURPOSE_EXPANSION_ROADMAP.md")
    assert "test-storage-platform-v1" in roadmap
    assert "test-storage-feature-contract-v1" in roadmap
