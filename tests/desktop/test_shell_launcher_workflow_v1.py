"""M52 PR-2: shell launcher workflow checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_desktop_shell_workflows_v1 as shell_tool  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _workflow(data: dict, workflow_id: str) -> dict:
    for workflow in data["workflows"]:
        if workflow["workflow_id"] == workflow_id:
            return workflow
    raise AssertionError(f"missing workflow: {workflow_id}")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m52" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_desktop_shell_workflows_v1_is_seed_deterministic():
    first = shell_tool.run_desktop_shell_workflows(seed=20260311)
    second = shell_tool.run_desktop_shell_workflows(seed=20260311)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_shell_launcher_workflow_v1_schema_and_pass():
    out = _out_path("desktop-shell-v1.json")
    rc = shell_tool.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    workflow = _workflow(data, "launcher_open")

    assert data["schema"] == "rugo.desktop_shell_workflow_report.v1"
    assert data["contract_id"] == "rugo.desktop_shell_contract.v1"
    assert data["workflow_profile_id"] == "rugo.session_workflow_profile.v1"
    assert data["gate"] == "test-desktop-shell-v1"
    assert data["workflow_gate"] == "test-desktop-workflows-v1"
    assert data["gate_pass"] is True
    assert data["workflow_totals"]["passed"] == 4
    assert data["summary"]["launcher"]["pass"] is True
    assert data["session_state"]["focus_owner"] == "settings.panel"
    assert data["source_reports"]["gui_runtime"]["schema"] == "rugo.gui_runtime_report.v1"
    assert workflow["category"] == "launcher"
    assert workflow["start_focus"] == "desktop.shell.launcher"
    assert workflow["end_focus"] == "files.panel"
    assert workflow["resulting_window_id"] == "files.panel"
    assert workflow["search_term"] == "Files"
    assert workflow["matched_items"] == ["files.panel"]
    assert workflow["checks_pass"] is True
    assert [step["name"] for step in workflow["steps"]] == [
        "open_launcher",
        "search_files_panel",
        "activate_files_panel",
    ]


def test_shell_launcher_workflow_v1_detects_launcher_regression():
    out = _out_path("desktop-shell-launcher-fail.json")
    rc = shell_tool.main(
        [
            "--inject-failure",
            "launcher_activation_integrity",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "launcher_activation_integrity" in data["failures"]
    assert data["summary"]["launcher"]["failures"] >= 1
