"""M30 PR-2: rollback safety and recovery negative-path checks."""

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_recovery_drill_v3 as recovery_tool  # noqa: E402
import run_upgrade_drill_v3 as upgrade_tool  # noqa: E402


def test_upgrade_drill_v3_rejects_sequence_below_floor(tmp_path: Path):
    out = tmp_path / "upgrade-drill-v3.json"
    rc = upgrade_tool.main(
        [
            "--candidate-sequence",
            "39",
            "--rollback-floor-sequence",
            "40",
            "--out",
            str(out),
        ]
    )
    assert rc == 1
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.upgrade_drill.v3"
    assert data["rollback_safety"]["rollback_floor_enforced"] is False
    assert data["gate_pass"] is False


def test_upgrade_drill_v3_detects_injected_failure(tmp_path: Path):
    out = tmp_path / "upgrade-drill-v3.json"
    rc = upgrade_tool.main(
        [
            "--inject-failure",
            "post_upgrade_health_check",
            "--out",
            str(out),
        ]
    )
    assert rc == 1
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.upgrade_drill.v3"
    assert data["total_failures"] >= 1
    assert data["gate_pass"] is False


def test_recovery_drill_v3_requires_operator_checklist(tmp_path: Path):
    out = tmp_path / "recovery-drill-v3.json"
    rc = recovery_tool.main(
        [
            "--skip-operator-checklist",
            "--out",
            str(out),
        ]
    )
    assert rc == 1
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.recovery_drill.v3"
    assert data["recovery_readiness"]["operator_checklist_completed"] is False
    assert data["gate_pass"] is False


def test_recovery_drill_v3_rejects_unknown_stage_injection(tmp_path: Path):
    out = tmp_path / "recovery-drill-v3.json"
    rc = recovery_tool.main(
        [
            "--inject-failure",
            "unknown_stage",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
