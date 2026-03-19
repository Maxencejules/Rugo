"""Fixture-level checks for the X4 desktop profile runtime aggregate."""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_desktop_profile_runtime_v1 as tool  # noqa: E402


def _work_dir(name: str) -> Path:
    path = ROOT / "out" / "pytest-x4" / name
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)
    return path


def test_desktop_profile_runtime_fixture_passes_and_emits_supporting_reports():
    work_dir = _work_dir("fixture-pass")
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
    capture = json.loads(capture_out.read_text(encoding="utf-8"))

    assert data["schema"] == "rugo.desktop_profile_runtime_report.v1"
    assert data["policy_id"] == "rugo.desktop_profile_runtime.v1"
    assert data["desktop_profile_id"] == "rugo.desktop_profile.v2"
    assert data["gate"] == "test-desktop-profile-runtime-v1"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["capture"]["schema"] == "rugo.booted_runtime_capture.v1"
    assert data["capture"]["capture_mode"] == "fixture"
    assert data["capture"]["image_path"].endswith("out/os-go-desktop.iso")
    assert data["capture"]["kernel_path"].endswith("out/kernel-go-desktop.elf")

    assert capture["schema"] == "rugo.booted_runtime_capture.v1"
    assert capture["capture_mode"] == "fixture"
    assert capture["image_path"].endswith("out/os-go-desktop.iso")
    assert capture["kernel_path"].endswith("out/kernel-go-desktop.elf")
    assert len(capture["boots"]) == 2

    backlogs = {row["backlog"]: row["status"] for row in data["backlog_closure"]}
    assert backlogs == {
        "M35": "pass",
        "M44": "pass",
        "M48": "pass",
        "M49": "pass",
        "M50": "pass",
        "M51": "pass",
        "M52": "pass",
    }

    check_map = {row["check_id"]: row["pass"] for row in data["checks"]}
    for check_id in [
        "desktop_bootstrap",
        "display_scanout",
        "input_seat",
        "window_compositor",
        "gui_runtime",
        "shell_workflows",
        "graphical_installer",
    ]:
        assert check_map[check_id] is True

    boot_markers = {row["boot_profile"]: row["desktop_markers"] for row in data["boot_profiles"]}
    assert boot_markers["cold_boot"]["display"] >= 3
    assert boot_markers["cold_boot"]["seat"] >= 2
    assert boot_markers["cold_boot"]["window"] >= 3
    assert boot_markers["cold_boot"]["gui"] >= 2
    assert boot_markers["cold_boot"]["shell"] >= 4
    assert boot_markers["cold_boot"]["installer"] >= 1
    assert boot_markers["replay_boot"]["boot"] >= 2

    assert data["runtime_components"]["display"]["active_runtime_path"] == "virtio-gpu-pci"
    assert data["runtime_components"]["seat"]["seat_id"] == "seat0"
    assert data["runtime_components"]["window"]["output_id"] == "display-0"
    assert data["runtime_components"]["gui"]["toolkit_profile_id"] == "rugo.toolkit_profile.v1"
    assert data["runtime_components"]["shell"]["workflow_passed"] == 4
    assert data["runtime_components"]["installer"]["session_handoff_pass"] is True

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
        assert (support_dir / name).is_file(), f"missing supporting report: {name}"


def test_desktop_profile_runtime_fixture_failure_propagates_to_backlogs():
    work_dir = _work_dir("fixture-fail")
    out = work_dir / "desktop-profile-runtime-v1.json"
    capture_out = work_dir / "desktop-profile-capture-v1.json"

    rc = tool.main(
        [
            "--seed",
            "20260318",
            "--fixture",
            "--inject-failure",
            "display_scanout",
            "--runtime-capture-out",
            str(capture_out),
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "display_scanout" in data["failures"]
    assert "M35" in data["failures"]
    assert "M48" in data["failures"]
