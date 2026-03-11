"""M52 PR-2: graphical installer smoke checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_graphical_installer_smoke_v1 as installer_tool  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m52" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_graphical_installer_smoke_v1_is_seed_deterministic():
    first = installer_tool.run_graphical_installer_smoke(seed=20260311)
    second = installer_tool.run_graphical_installer_smoke(seed=20260311)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_graphical_installer_smoke_v1_schema_and_gate_pass():
    out = _out_path("graphical-installer-v1.json")
    rc = installer_tool.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.graphical_installer_smoke_report.v1"
    assert data["contract_id"] == "rugo.graphical_installer_ux.v1"
    assert data["workflow_id"] == "rugo.graphical_installer_flow.v1"
    assert data["parent_installer_ux_contract_id"] == "rugo.installer_ux_contract.v3"
    assert data["parent_shell_contract_id"] == "rugo.desktop_shell_contract.v1"
    assert data["session_workflow_profile_id"] == "rugo.session_workflow_profile.v1"
    assert data["gate"] == "test-desktop-workflows-v1"
    assert data["parent_gate"] == "test-desktop-shell-v1"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert [stage["name"] for stage in data["stages"]] == [
        "shell_bootstrap",
        "device_scan",
        "target_selection",
        "layout_review",
        "install_commit",
        "first_boot_handoff",
    ]
    assert data["selected_target"]["device_id"] == "disk0"
    assert data["layout"]["partition_layout"] == ["efi", "system", "recovery"]
    assert data["layout"]["recovery_partition_present"] is True
    assert data["handoff"]["boot_target"] == "desktop.shell.workspace"
    assert data["handoff"]["first_boot_focus"] == "desktop.shell.launcher"
    assert data["source_reports"]["desktop_shell"]["gate_pass"] is True
    assert data["source_reports"]["recovery_drill"]["gate_pass"] is True


def test_graphical_installer_smoke_v1_detects_layout_regression():
    out = _out_path("graphical-installer-layout-fail.json")
    rc = installer_tool.main(
        [
            "--inject-failure",
            "layout_validation_integrity",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "layout_validation_integrity" in data["failures"]
    assert data["summary"]["installer"]["failures"] >= 1
