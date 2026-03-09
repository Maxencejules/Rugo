"""M30 PR-2: deterministic upgrade and recovery drill v3 checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_recovery_drill_v3 as recovery_tool  # noqa: E402
import run_upgrade_drill_v3 as upgrade_tool  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_upgrade_drill_v3_is_seed_deterministic():
    first = upgrade_tool.run_upgrade_drill(
        seed=20260309,
        candidate_sequence=42,
        rollback_floor_sequence=40,
    )
    second = upgrade_tool.run_upgrade_drill(
        seed=20260309,
        candidate_sequence=42,
        rollback_floor_sequence=40,
    )
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_recovery_drill_v3_is_seed_deterministic():
    first = recovery_tool.run_recovery_drill(seed=20260309)
    second = recovery_tool.run_recovery_drill(seed=20260309)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_upgrade_drill_v3_schema_and_gate_pass(tmp_path: Path):
    out = tmp_path / "upgrade-drill-v3.json"
    rc = upgrade_tool.main(
        [
            "--seed",
            "20260309",
            "--candidate-sequence",
            "42",
            "--rollback-floor-sequence",
            "40",
            "--out",
            str(out),
        ]
    )
    assert rc == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.upgrade_drill.v3"
    assert data["contract_id"] == "rugo.installer_ux_contract.v3"
    assert data["total_failures"] == 0
    assert data["gate_pass"] is True
    assert [stage["name"] for stage in data["stages"]] == [
        "upgrade_plan_validate",
        "upgrade_apply",
        "post_upgrade_health_check",
        "rollback_safety_check",
    ]
    assert data["rollback_safety"]["schema"] == "rugo.rollback_safety_report.v3"
    assert data["rollback_safety"]["rollback_floor_enforced"] is True
    assert data["rollback_safety"]["unsigned_artifact_rejected"] is True
    assert data["rollback_safety"]["rollback_path_verified"] is True


def test_recovery_drill_v3_schema_and_gate_pass(tmp_path: Path):
    out = tmp_path / "recovery-drill-v3.json"
    rc = recovery_tool.main(["--seed", "20260309", "--out", str(out)])
    assert rc == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.recovery_drill.v3"
    assert data["contract_id"] == "rugo.installer_ux_contract.v3"
    assert data["workflow_id"] == "rugo.recovery_workflow.v3"
    assert data["total_failures"] == 0
    assert data["gate_pass"] is True
    assert [stage["name"] for stage in data["stages"]] == [
        "recovery_entry_validation",
        "rollback_snapshot_mount",
        "state_reconciliation",
        "service_restore_validation",
        "post_recovery_audit",
    ]
    assert data["recovery_readiness"]["operator_checklist_completed"] is True
    assert data["recovery_readiness"]["triage_bundle_required"] is True
    assert data["recovery_readiness"]["state_capture_complete"] is True
