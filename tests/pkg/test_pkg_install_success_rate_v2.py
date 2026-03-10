"""M44 PR-2: runtime-qualified package install success checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_real_pkg_install_campaign_v2 as campaign  # noqa: E402


def _check(data: dict, check_id: str) -> dict:
    rows = [entry for entry in data["checks"] if entry["check_id"] == check_id]
    assert len(rows) == 1
    return rows[0]


def test_real_pkg_install_campaign_v2_schema_and_pass(tmp_path: Path):
    out = tmp_path / "real-pkg-install-v2.json"
    rc = campaign.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.real_pkg_install_campaign_report.v2"
    assert data["ecosystem_policy_id"] == "rugo.ecosystem_scale_policy.v2"
    assert data["distribution_workflow_id"] == "rugo.distribution_workflow.v2"
    assert data["desktop_profile_id"] == "rugo.desktop_profile.v2"
    assert data["app_tier_schema_id"] == "rugo.app_compat_tiers.v2"
    assert data["gate"] == "test-real-app-catalog-v2"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["summary"]["install"]["pass"] is True
    assert data["summary"]["workflow"]["pass"] is True
    assert data["summary"]["provenance"]["pass"] is True
    assert data["summary"]["quality"]["pass"] is True
    assert data["install_success"]["stable_install_success_ratio"] >= 0.990
    assert data["install_success"]["canary_install_success_ratio"] >= 0.975
    assert data["install_success"]["edge_install_success_ratio"] >= 0.950
    assert data["latency"]["stable_install_p95_ms"] <= 65
    assert data["latency"]["canary_install_p95_ms"] <= 80
    assert data["latency"]["edge_install_p95_ms"] <= 95
    assert data["workflow"]["rollback_success_ratio"] >= 1.0
    assert data["provenance"]["runtime_trace_coverage_ratio"] >= 1.0
    assert data["provenance"]["signed_provenance_ratio"] >= 1.0
    assert data["provenance"]["reproducible_install_ratio"] >= 0.99
    assert data["quality"]["metadata_expiry_violations"] == 0
    assert data["quality"]["signature_verification_failures"] == 0
    assert data["quality"]["digest_mismatch_count"] == 0
    assert _check(data, "stable_install_success_ratio")["pass"] is True
    assert _check(data, "runtime_trace_coverage_ratio")["pass"] is True


def test_real_pkg_install_campaign_v2_detects_edge_regression(tmp_path: Path):
    out = tmp_path / "real-pkg-install-v2-fail.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "edge_install_success_ratio",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["install"]["failures"] >= 1
    assert _check(data, "edge_install_success_ratio")["pass"] is False


def test_real_pkg_install_campaign_v2_rejects_unknown_check_id(tmp_path: Path):
    out = tmp_path / "real-pkg-install-v2-error.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "install_nonexistent_check_v2",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
