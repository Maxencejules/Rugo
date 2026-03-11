"""M52 PR-1: desktop shell, workflow, and graphical installer doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m52_pr1_desktop_shell_artifacts_exist():
    required = [
        "docs/M52_EXECUTION_BACKLOG.md",
        "docs/desktop/desktop_shell_contract_v1.md",
        "docs/desktop/session_workflow_profile_v1.md",
        "docs/build/graphical_installer_ux_v1.md",
        "tests/desktop/test_desktop_shell_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M52 PR-1 artifact: {rel}"


def test_desktop_shell_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/desktop/desktop_shell_contract_v1.md")
    for token in [
        "Desktop shell contract ID: `rugo.desktop_shell_contract.v1`.",
        "Parent GUI runtime contract ID: `rugo.gui_runtime_contract.v1`.",
        "Parent desktop profile ID: `rugo.desktop_profile.v2`.",
        "Session workflow profile ID: `rugo.session_workflow_profile.v1`.",
        "Shell workflow report schema: `rugo.desktop_shell_workflow_report.v1`.",
        "Graphical installer smoke schema: `rugo.graphical_installer_smoke_report.v1`.",
        "`desktop.shell.launcher`",
        "`desktop.shell.taskbar`",
        "`desktop.shell.power_menu`",
        "`launcher_open_budget`",
        "`file_save_commit_budget`",
        "`settings_persist_integrity`",
        "`shutdown_surface_cleanup_integrity`",
        "Local gate: `make test-desktop-shell-v1`.",
        "Local sub-gate: `make test-desktop-workflows-v1`.",
        "CI gate: `Desktop shell v1 gate`.",
        "CI sub-gate: `Desktop workflows v1 gate`.",
    ]:
        assert token in doc


def test_session_workflow_profile_v1_doc_declares_required_tokens():
    doc = _read("docs/desktop/session_workflow_profile_v1.md")
    for token in [
        "Session workflow profile ID: `rugo.session_workflow_profile.v1`.",
        "Parent desktop shell contract ID: `rugo.desktop_shell_contract.v1`.",
        "Parent GUI runtime contract ID: `rugo.gui_runtime_contract.v1`.",
        "Shell workflow report schema: `rugo.desktop_shell_workflow_report.v1`.",
        "Graphical installer smoke schema: `rugo.graphical_installer_smoke_report.v1`.",
        "`launcher_open`",
        "`file_open_save`",
        "`settings_update`",
        "`shutdown_request`",
        "`graphical_installer_smoke`",
        "Minimum required passing shell workflows: `4`.",
        "Minimum required passing installer workflows: `1`.",
    ]:
        assert token in doc


def test_graphical_installer_ux_v1_doc_declares_required_tokens():
    doc = _read("docs/build/graphical_installer_ux_v1.md")
    for token in [
        "Graphical installer UX ID: `rugo.graphical_installer_ux.v1`.",
        "Parent installer UX contract ID: `rugo.installer_ux_contract.v3`.",
        "Parent desktop shell contract ID: `rugo.desktop_shell_contract.v1`.",
        "Recovery workflow ID: `rugo.recovery_workflow.v3`.",
        "Installer contract schema: `rugo.installer_contract.v2`.",
        "Graphical installer smoke schema: `rugo.graphical_installer_smoke_report.v1`.",
        "`shell_bootstrap`",
        "`device_scan`",
        "`layout_review`",
        "`first_boot_handoff`",
        "`recovery_entry_visible` must remain `>= 1.0`.",
        "Maximum allowed graphical installer failures: `0`.",
    ]:
        assert token in doc


def test_m52_roadmap_anchor_declares_desktop_shell_gates():
    roadmap = _read("docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md")
    assert "test-desktop-shell-v1" in roadmap
    assert "test-desktop-workflows-v1" in roadmap
