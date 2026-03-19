"""X4 aggregate gate: desktop profile runtime wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_desktop_profile_runtime_v1 as tool  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _work_dir(name: str) -> Path:
    path = ROOT / "out" / "pytest-x4" / name
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)
    return path


def test_desktop_profile_runtime_gate_v1_wiring_and_artifacts():
    required = [
        "docs/desktop/desktop_profile_runtime_v1.md",
        "tools/x4_desktop_runtime_common_v1.py",
        "tools/run_desktop_profile_runtime_v1.py",
        "tests/desktop/test_desktop_profile_runtime_docs_v1.py",
        "tests/desktop/test_desktop_profile_runtime_v1.py",
        "tests/desktop/test_desktop_profile_runtime_gate_v1.py",
        "docs/roadmap/implementation_closure/expansion_breadth.md",
        "docs/M35_EXECUTION_BACKLOG.md",
        "docs/M44_EXECUTION_BACKLOG.md",
        "docs/M48_EXECUTION_BACKLOG.md",
        "docs/M49_EXECUTION_BACKLOG.md",
        "docs/M50_EXECUTION_BACKLOG.md",
        "docs/M51_EXECUTION_BACKLOG.md",
        "docs/M52_EXECUTION_BACKLOG.md",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing X4 runtime artifact: {rel}"

    readme = _read("README.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    summary = _read("docs/roadmap/README.md")
    framework = _read("docs/roadmap/MILESTONE_FRAMEWORK.md")
    expansion = _read("docs/roadmap/implementation_closure/expansion_breadth.md")
    gui_roadmap = _read("docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md")

    assert "make test-desktop-profile-runtime-v1" in readme

    assert "test-desktop-profile-runtime-v1" in makefile
    for entry in [
        "tools/run_desktop_profile_runtime_v1.py --image $(OUT)/os-go-desktop.iso --kernel $(OUT)/kernel-go-desktop.elf --runtime-capture-out $(OUT)/desktop-profile-capture-v1.json --emit-supporting-reports --supporting-dir $(OUT) --out $(OUT)/desktop-profile-runtime-v1.json",
        "tests/desktop/test_desktop_profile_runtime_docs_v1.py",
        "tests/desktop/test_desktop_profile_runtime_v1.py",
        "tests/desktop/test_desktop_profile_runtime_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-desktop-profile-runtime-v1.xml" in makefile

    assert "Desktop profile runtime v1 gate" in ci
    assert "make test-desktop-profile-runtime-v1" in ci
    assert "desktop-profile-runtime-v1-artifacts" in ci
    for artifact in [
        "out/pytest-desktop-profile-runtime-v1.xml",
        "out/desktop-profile-capture-v1.json",
        "out/desktop-profile-runtime-v1.json",
        "out/os-go-desktop.iso",
        "out/kernel-go-desktop.elf",
    ]:
        assert artifact in ci

    assert "historical X4 desktop and workflow backlog is closed on a shared runtime-backed qualification lane" in summary
    assert "historical X4 desktop and workflow backlog is closed on a shared runtime-backed qualification lane" in framework
    assert "The historical X4 desktop and workflow backlog is now runtime-backed" in expansion
    assert "test-desktop-profile-runtime-v1" in gui_roadmap

    for relpath in [
        "docs/M35_EXECUTION_BACKLOG.md",
        "docs/M44_EXECUTION_BACKLOG.md",
        "docs/M48_EXECUTION_BACKLOG.md",
        "docs/M49_EXECUTION_BACKLOG.md",
        "docs/M50_EXECUTION_BACKLOG.md",
        "docs/M51_EXECUTION_BACKLOG.md",
        "docs/M52_EXECUTION_BACKLOG.md",
        ]:
            doc = _read(relpath)
            assert "X4 runtime-backed closure addendum (2026-03-18)" in doc
            lowered = doc.lower()
            assert "desktop profile runtime" in lowered
            assert "qualification lane" in lowered

    work_dir = _work_dir("gate")
    out = work_dir / "desktop-profile-runtime-v1.json"
    capture_out = work_dir / "desktop-profile-capture-v1.json"
    support_dir = work_dir / "support"

    rc = tool.main(
        [
            "--seed",
            "20260318",
            "--fixture",
            "--runtime-capture-out",
            str(capture_out),
            "--emit-supporting-reports",
            "--supporting-dir",
            str(support_dir),
            "--out",
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.desktop_profile_runtime_report.v1"
    assert data["gate_pass"] is True
    assert data["capture"]["digest"]
    assert data["source_reports"]["display_runtime_v1"]["gate_pass"] is True
    assert data["source_reports"]["real_gui_matrix_v2"]["gate_pass"] is True
    assert data["source_reports"]["real_pkg_install_v2"]["gate_pass"] is True
    assert data["source_reports"]["real_catalog_audit_v2"]["gate_pass"] is True

    for name in [
        "desktop-smoke-v1.json",
        "gui-app-matrix-v1.json",
        "display-runtime-v1.json",
        "input-seat-v1.json",
        "window-system-v1.json",
        "gui-runtime-v1.json",
        "toolkit-compat-v1.json",
        "desktop-shell-v1.json",
        "graphical-installer-v1.json",
        "real-gui-matrix-v2.json",
        "real-pkg-install-v2.json",
        "real-catalog-audit-v2.json",
    ]:
        assert (support_dir / name).is_file(), f"missing emitted supporting report: {name}"
