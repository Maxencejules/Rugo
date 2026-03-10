"""M44 PR-2: runtime-qualified catalog reproducibility audit checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_real_catalog_audit_v2 as audit  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _check(data: dict, check_id: str) -> dict:
    rows = [entry for entry in data["checks"] if entry["check_id"] == check_id]
    assert len(rows) == 1
    return rows[0]


def test_real_catalog_audit_v2_report_is_seed_deterministic():
    first = audit.run_audit(seed=20260310)
    second = audit.run_audit(seed=20260310)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_real_catalog_audit_v2_schema_and_pass(tmp_path: Path):
    out = tmp_path / "real-catalog-audit-v2.json"
    rc = audit.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.real_catalog_audit_report.v2"
    assert data["ecosystem_policy_id"] == "rugo.ecosystem_scale_policy.v2"
    assert data["distribution_workflow_id"] == "rugo.distribution_workflow.v2"
    assert data["desktop_profile_id"] == "rugo.desktop_profile.v2"
    assert data["gate"] == "test-real-app-catalog-v2"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["summary"]["reproducibility"]["pass"] is True
    assert data["summary"]["distribution"]["pass"] is True
    assert data["summary"]["workflow"]["pass"] is True
    assert data["summary"]["provenance"]["pass"] is True
    assert data["summary"]["quality"]["pass"] is True
    assert data["reproducibility"]["deterministic_rebuild_ratio"] >= 1.0
    assert data["reproducibility"]["provenance_match_ratio"] >= 1.0
    assert data["reproducibility"]["sbom_drift_count"] == 0
    assert data["distribution"]["mirror_index_consistency_ratio"] >= 1.0
    assert data["distribution"]["replication_lag_p95_minutes"] <= 10
    assert data["workflow"]["workflow_stage_completeness_ratio"] >= 1.0
    assert data["workflow"]["release_signoff_ratio"] >= 1.0
    assert data["workflow"]["rollback_drill_pass_ratio"] >= 1.0
    assert data["provenance"]["runtime_trace_coverage_ratio"] >= 1.0
    assert data["provenance"]["signed_artifact_ratio"] >= 1.0
    assert data["quality"]["unresolved_policy_exceptions"] == 0
    assert data["quality"]["stale_manifest_count"] == 0
    assert _check(data, "deterministic_rebuild_ratio")["pass"] is True
    assert _check(data, "workflow_stage_completeness_ratio")["pass"] is True


def test_real_catalog_audit_v2_detects_rebuild_regression(tmp_path: Path):
    out = tmp_path / "real-catalog-audit-v2-fail.json"
    rc = audit.main(
        [
            "--inject-failure",
            "deterministic_rebuild_ratio",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["reproducibility"]["failures"] >= 1
    assert _check(data, "deterministic_rebuild_ratio")["pass"] is False


def test_real_catalog_audit_v2_rejects_unknown_check_id(tmp_path: Path):
    out = tmp_path / "real-catalog-audit-v2-error.json"
    rc = audit.main(
        [
            "--inject-failure",
            "audit_nonexistent_check_v2",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
