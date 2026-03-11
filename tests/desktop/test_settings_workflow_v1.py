"""M52 PR-2: settings workflow checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_desktop_shell_workflows_v1 as shell_tool  # noqa: E402


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


def test_settings_workflow_v1_artifacts_pass():
    out = _out_path("desktop-shell-settings-v1.json")
    rc = shell_tool.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    workflow = _workflow(data, "settings_update")

    assert data["summary"]["settings"]["pass"] is True
    assert workflow["category"] == "settings"
    assert workflow["start_focus"] == "desktop.shell.launcher"
    assert workflow["end_focus"] == "settings.panel"
    assert workflow["section"] == "display"
    assert workflow["changed_keys"] == ["accent_color", "scale_percent"]
    assert workflow["previous_values"] == {"accent_color": "blue", "scale_percent": 100}
    assert workflow["new_values"] == {"accent_color": "amber", "scale_percent": 125}
    assert workflow["persisted_config_path"] == "/system/session/config/display.json"
    assert workflow["persisted"] is True
    assert workflow["checks_pass"] is True


def test_settings_workflow_v1_detects_persist_regression():
    out = _out_path("desktop-shell-settings-fail.json")
    rc = shell_tool.main(
        [
            "--inject-failure",
            "settings_persist_integrity",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "settings_persist_integrity" in data["failures"]
    assert data["summary"]["settings"]["failures"] >= 1
