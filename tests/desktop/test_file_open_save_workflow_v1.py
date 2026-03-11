"""M52 PR-2: file open/save workflow checks."""

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


def test_file_open_save_workflow_v1_artifacts_pass():
    out = _out_path("desktop-shell-file-v1.json")
    rc = shell_tool.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    workflow = _workflow(data, "file_open_save")

    assert data["summary"]["files"]["pass"] is True
    assert workflow["category"] == "files"
    assert workflow["start_focus"] == "files.panel"
    assert workflow["end_focus"] == "files.panel"
    assert workflow["opened_path"] == "/home/demo/Documents/plan-v1.txt"
    assert workflow["save_path"] == "/home/demo/Documents/plan-v1-saved.txt"
    assert workflow["dirty_before_save"] is True
    assert workflow["dirty_after_save"] is False
    assert workflow["save_revision"] == 2
    assert workflow["recent_files"][0] == "/home/demo/Documents/plan-v1-saved.txt"
    assert workflow["bytes_saved"] >= workflow["bytes_loaded"]
    assert workflow["checks_pass"] is True


def test_file_open_save_workflow_v1_detects_save_regression():
    out = _out_path("desktop-shell-file-fail.json")
    rc = shell_tool.main(
        [
            "--inject-failure",
            "file_save_commit_budget",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "file_save_commit_budget" in data["failures"]
    assert data["summary"]["files"]["failures"] >= 1
