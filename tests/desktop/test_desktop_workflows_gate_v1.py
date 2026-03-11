"""M52 aggregate sub-gate: desktop workflows wiring and artifacts."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_desktop_shell_workflows_v1 as shell_tool  # noqa: E402
import run_graphical_installer_smoke_v1 as installer_tool  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m52" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_desktop_workflows_gate_v1_wiring_and_artifacts():
    required = [
        "docs/M52_EXECUTION_BACKLOG.md",
        "docs/desktop/desktop_shell_contract_v1.md",
        "docs/desktop/session_workflow_profile_v1.md",
        "docs/build/graphical_installer_ux_v1.md",
        "tools/run_desktop_shell_workflows_v1.py",
        "tools/run_graphical_installer_smoke_v1.py",
        "tests/desktop/test_desktop_shell_docs_v1.py",
        "tests/desktop/test_shell_launcher_workflow_v1.py",
        "tests/desktop/test_file_open_save_workflow_v1.py",
        "tests/desktop/test_settings_workflow_v1.py",
        "tests/build/test_graphical_installer_smoke_v1.py",
        "tests/desktop/test_desktop_workflows_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M52 sub-gate artifact: {rel}"

    roadmap = _read("docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M52_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-desktop-workflows-v1" in roadmap

    assert "test-desktop-workflows-v1" in makefile
    for entry in [
        "tools/run_desktop_shell_workflows_v1.py --out $(OUT)/desktop-shell-v1.json",
        "tools/run_graphical_installer_smoke_v1.py --out $(OUT)/graphical-installer-v1.json",
        "tests/desktop/test_desktop_shell_docs_v1.py",
        "tests/desktop/test_shell_launcher_workflow_v1.py",
        "tests/desktop/test_file_open_save_workflow_v1.py",
        "tests/desktop/test_settings_workflow_v1.py",
        "tests/build/test_graphical_installer_smoke_v1.py",
        "tests/desktop/test_desktop_workflows_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-desktop-workflows-v1.xml" in makefile

    assert "Desktop workflows v1 gate" in ci
    assert "make test-desktop-workflows-v1" in ci
    assert "desktop-workflows-v1-artifacts" in ci
    assert "out/pytest-desktop-workflows-v1.xml" in ci
    assert "out/desktop-shell-v1.json" in ci
    assert "out/graphical-installer-v1.json" in ci

    assert "Status: done" in backlog
    assert "| M52 | Desktop Shell + Workflow Baseline v1 | n/a | done |" in milestones
    assert "| **M52** Desktop Shell + Workflow Baseline v1 | n/a | done |" in status
    assert "M52 execution backlog (completed)" in readme

    shell_out = _out_path("desktop-shell-subgate-v1.json")
    installer_out = _out_path("graphical-installer-subgate-v1.json")

    assert shell_tool.main(["--seed", "20260311", "--out", str(shell_out)]) == 0
    assert installer_tool.main(["--seed", "20260311", "--out", str(installer_out)]) == 0

    shell_data = json.loads(shell_out.read_text(encoding="utf-8"))
    installer_data = json.loads(installer_out.read_text(encoding="utf-8"))

    assert shell_data["summary"]["files"]["pass"] is True
    assert shell_data["summary"]["settings"]["pass"] is True
    assert installer_data["summary"]["installer"]["pass"] is True
    assert installer_data["summary"]["source"]["pass"] is True
    assert installer_data["gate_pass"] is True
