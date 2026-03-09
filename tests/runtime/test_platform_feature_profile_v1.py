"""M38 PR-2: deterministic platform feature profile conformance checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_platform_feature_conformance_v1 as conformance  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _profile(data: dict, profile_id: str) -> dict:
    rows = [entry for entry in data["profiles"] if entry["profile_id"] == profile_id]
    assert len(rows) == 1
    return rows[0]


def test_platform_feature_profile_v1_doc_declares_required_tokens():
    doc = _read("docs/runtime/platform_feature_profile_v1.md")
    for token in [
        "Platform conformance policy ID: `rugo.platform_feature_profile.v1`",
        "Platform conformance report schema: `rugo.platform_feature_conformance_report.v1`",
        "Profile requirement schema: `rugo.platform_feature_requirement_set.v1`",
        "`server_storage_dense_v1`",
        "`edge_resilient_v1`",
        "`dev_workstation_v1`",
        "Local gate: `make test-storage-platform-v1`",
        "Local sub-gate: `make test-storage-feature-contract-v1`",
    ]:
        assert token in doc


def test_platform_feature_conformance_v1_report_is_seed_deterministic():
    first = conformance.run_conformance(seed=20260309)
    second = conformance.run_conformance(seed=20260309)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_platform_feature_conformance_v1_schema_and_pass(tmp_path: Path):
    out = tmp_path / "platform-feature-v1.json"
    rc = conformance.main(["--seed", "20260309", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.platform_feature_conformance_report.v1"
    assert data["policy_id"] == "rugo.platform_feature_profile.v1"
    assert data["profile_schema"] == "rugo.platform_feature_requirement_set.v1"
    assert data["storage_feature_contract_id"] == "rugo.storage_feature_contract.v1"
    assert set(data["checked_profiles"]) == {
        "server_storage_dense_v1",
        "edge_resilient_v1",
        "dev_workstation_v1",
    }
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert _profile(data, "server_storage_dense_v1")["qualification_pass"] is True
    assert _profile(data, "edge_resilient_v1")["qualification_pass"] is True
    assert _profile(data, "dev_workstation_v1")["qualification_pass"] is True


def test_platform_feature_conformance_v1_detects_profile_regression(tmp_path: Path):
    out = tmp_path / "platform-feature-v1-fail.json"
    rc = conformance.main(
        [
            "--inject-failure",
            "edge_post_resize_fsck_errors",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["total_failures"] >= 1
    profile = _profile(data, "edge_resilient_v1")
    assert profile["qualification_pass"] is False
    assert "edge_post_resize_fsck_errors" in profile["failed_requirements"]


def test_platform_feature_conformance_v1_rejects_unknown_check_id(tmp_path: Path):
    out = tmp_path / "platform-feature-v1-error.json"
    rc = conformance.main(
        [
            "--inject-failure",
            "platform_nonexistent_check",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
