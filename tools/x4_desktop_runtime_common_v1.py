#!/usr/bin/env python3
"""Shared helpers for X4 desktop profile runtime qualification."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Mapping, Sequence, Set

import run_desktop_shell_workflows_v1
import run_desktop_smoke_v1
import run_display_runtime_v1
import run_graphical_installer_smoke_v1
import run_gui_app_matrix_v1
import run_gui_runtime_v1
import run_input_seat_runtime_v1
import run_real_catalog_audit_v2
import run_real_gui_app_matrix_v2
import run_real_pkg_install_campaign_v2
import run_toolkit_compat_v1
import run_window_system_runtime_v1
import runtime_capture_common_v1 as runtime_capture


SCHEMA = "rugo.desktop_profile_runtime_report.v1"
POLICY_ID = "rugo.desktop_profile_runtime.v1"
TRACK_ID = "X4"
DESKTOP_PROFILE_ID = "rugo.desktop_profile.v2"
DEFAULT_SEED = 20260318
DEFAULT_RUNTIME_CAPTURE_PATH = Path("out/desktop-profile-capture-v1.json")
DEFAULT_RELEASE_IMAGE_PATH = Path("out/os-go-desktop.iso")
DEFAULT_KERNEL_PATH = Path("out/kernel-go-desktop.elf")
DEFAULT_PANIC_IMAGE_PATH = Path("out/os-panic.iso")

SUPPORTING_REPORT_PATHS = {
    "desktop_smoke_v1": "out/desktop-smoke-v1.json",
    "gui_app_matrix_v1": "out/gui-app-matrix-v1.json",
    "display_runtime_v1": "out/display-runtime-v1.json",
    "input_seat_v1": "out/input-seat-v1.json",
    "window_system_v1": "out/window-system-v1.json",
    "gui_runtime_v1": "out/gui-runtime-v1.json",
    "toolkit_compat_v1": "out/toolkit-compat-v1.json",
    "desktop_shell_v1": "out/desktop-shell-v1.json",
    "graphical_installer_v1": "out/graphical-installer-v1.json",
    "real_gui_matrix_v2": "out/real-gui-matrix-v2.json",
    "real_pkg_install_v2": "out/real-pkg-install-v2.json",
    "real_catalog_audit_v2": "out/real-catalog-audit-v2.json",
}

DESKTOP_MARKERS = (
    "DESKBOOT: profile desktop_v1",
    "DESKDISP: probe virtio-gpu-pci",
    "DESKDISP: mode 1280x720@60",
    "DESKDISP: frame ok",
    "DESKSEAT: seat0 ready",
    "DESKSEAT: focus desktop.shell.launcher",
    "DESKCOMP: workspace visible",
    "DESKCOMP: files.panel occluded",
    "DESKCOMP: settings.panel focused",
    "DESKGUI: toolkit rugo.widgets.retain.v1",
    "DESKGUI: font rugo-sans",
    "DSHELL: launcher Files",
    "DSHELL: file save ok",
    "DSHELL: settings apply ok",
    "DSHELL: shutdown guard ok",
    "DINST: recovery entry visible",
    "DESKBOOT: ready",
)


@dataclass(frozen=True)
class RuntimeCheck:
    check_id: str
    domain: str
    cold_markers: Sequence[str]
    replay_markers: Sequence[str]
    description: str


RUNTIME_CHECKS: Sequence[RuntimeCheck] = (
    RuntimeCheck(
        check_id="desktop_bootstrap",
        domain="boot",
        cold_markers=("DESKBOOT: profile desktop_v1", "DESKBOOT: ready"),
        replay_markers=("DESKBOOT: profile desktop_v1", "DESKBOOT: ready"),
        description="the desktop-profile userspace path boots to the declared ready marker",
    ),
    RuntimeCheck(
        check_id="display_scanout",
        domain="display",
        cold_markers=(
            "DESKDISP: probe virtio-gpu-pci",
            "DESKDISP: mode 1280x720@60",
            "DESKDISP: frame ok",
        ),
        replay_markers=(
            "DESKDISP: probe virtio-gpu-pci",
            "DESKDISP: mode 1280x720@60",
            "DESKDISP: frame ok",
        ),
        description="the booted desktop image exposes the bounded scanout path",
    ),
    RuntimeCheck(
        check_id="input_seat",
        domain="input",
        cold_markers=("DESKSEAT: seat0 ready", "DESKSEAT: focus desktop.shell.launcher"),
        replay_markers=("DESKSEAT: seat0 ready", "DESKSEAT: focus desktop.shell.launcher"),
        description="keyboard and pointer seat ownership are visible on the booted desktop lane",
    ),
    RuntimeCheck(
        check_id="window_compositor",
        domain="window",
        cold_markers=(
            "DESKCOMP: workspace visible",
            "DESKCOMP: files.panel occluded",
            "DESKCOMP: settings.panel focused",
        ),
        replay_markers=(
            "DESKCOMP: workspace visible",
            "DESKCOMP: files.panel occluded",
            "DESKCOMP: settings.panel focused",
        ),
        description="window/compositor state is emitted by the real desktop profile",
    ),
    RuntimeCheck(
        check_id="gui_runtime",
        domain="gui",
        cold_markers=("DESKGUI: toolkit rugo.widgets.retain.v1", "DESKGUI: font rugo-sans"),
        replay_markers=("DESKGUI: toolkit rugo.widgets.retain.v1", "DESKGUI: font rugo-sans"),
        description="the booted desktop profile reaches the declared toolkit and font bridge",
    ),
    RuntimeCheck(
        check_id="shell_workflows",
        domain="shell",
        cold_markers=(
            "DSHELL: launcher Files",
            "DSHELL: file save ok",
            "DSHELL: settings apply ok",
            "DSHELL: shutdown guard ok",
        ),
        replay_markers=(
            "DSHELL: launcher Files",
            "DSHELL: file save ok",
            "DSHELL: settings apply ok",
            "DSHELL: shutdown guard ok",
        ),
        description="the booted desktop profile reaches the bounded shell workflows",
    ),
    RuntimeCheck(
        check_id="graphical_installer",
        domain="installer",
        cold_markers=("DINST: recovery entry visible",),
        replay_markers=("DINST: recovery entry visible",),
        description="the graphical installer handoff is visible on the real desktop stack",
    ),
)


def _known_checks() -> Set[str]:
    return {check.check_id for check in RUNTIME_CHECKS}


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - _known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def collect_source_reports(seed: int) -> Dict[str, Dict[str, object]]:
    return {
        "desktop_smoke_v1": run_desktop_smoke_v1.run_smoke(seed=seed),
        "gui_app_matrix_v1": run_gui_app_matrix_v1.run_matrix(seed=seed),
        "display_runtime_v1": run_display_runtime_v1.run_display_runtime(
            seed=seed,
            max_failures=0,
        ),
        "input_seat_v1": run_input_seat_runtime_v1.run_input_seat_runtime(
            seed=seed,
            max_failures=0,
        ),
        "window_system_v1": run_window_system_runtime_v1.run_window_system_runtime(
            seed=seed,
            max_failures=0,
        ),
        "gui_runtime_v1": run_gui_runtime_v1.run_gui_runtime(seed=seed, max_failures=0),
        "toolkit_compat_v1": run_toolkit_compat_v1.run_toolkit_compat(
            seed=seed,
            max_failures=0,
        ),
        "desktop_shell_v1": run_desktop_shell_workflows_v1.run_desktop_shell_workflows(
            seed=seed,
            max_failures=0,
        ),
        "graphical_installer_v1": run_graphical_installer_smoke_v1.run_graphical_installer_smoke(
            seed=seed,
            max_failures=0,
        ),
        "real_gui_matrix_v2": run_real_gui_app_matrix_v2.run_matrix(seed=seed),
        "real_pkg_install_v2": run_real_pkg_install_campaign_v2.run_campaign(seed=seed),
        "real_catalog_audit_v2": run_real_catalog_audit_v2.run_audit(seed=seed),
    }


def write_supporting_reports(
    reports: Mapping[str, Dict[str, object]],
    *,
    base_dir: str | Path,
) -> None:
    root = Path(base_dir)
    root.mkdir(parents=True, exist_ok=True)
    for key, relpath in SUPPORTING_REPORT_PATHS.items():
        runtime_capture.write_json(root / Path(relpath).name, reports[key])


def _extract_snapshots(
    lines: Sequence[Dict[str, object]],
) -> tuple[List[Dict[str, object]], List[Dict[str, object]]]:
    process_snapshots: List[Dict[str, object]] = []
    task_snapshots: List[Dict[str, object]] = []
    for entry in lines:
        parsed = runtime_capture.classify_runtime_line(str(entry["line"]))
        snapshot = {
            "ts_ms": round(float(entry["ts_ms"]), 3),
            "line": parsed["line"],
            "service": parsed.get("service", ""),
            "metrics": parsed.get("metrics", {}),
        }
        if parsed["prefix"] == "PROC":
            process_snapshots.append(snapshot)
        elif parsed["prefix"] == "TASK":
            task_snapshots.append(snapshot)
    return process_snapshots, task_snapshots


def _build_boot_entry(
    *,
    capture_id: str,
    boot_index: int,
    boot_profile: str,
    lines: Sequence[Dict[str, object]],
    exit_code: int,
) -> Dict[str, object]:
    process_snapshots, task_snapshots = _extract_snapshots(lines)
    serial_digest = runtime_capture.digest_lines(lines)
    boot_id = runtime_capture.stable_digest(
        {
            "capture_id": capture_id,
            "boot_index": boot_index,
            "boot_profile": boot_profile,
            "serial_digest": serial_digest,
        }
    )[:16]
    duration_ms = round(float(lines[-1]["ts_ms"]) if lines else 0.0, 3)
    return {
        "boot_id": boot_id,
        "boot_index": boot_index,
        "boot_profile": boot_profile,
        "exit_code": exit_code,
        "duration_ms": duration_ms,
        "serial_line_count": len(lines),
        "serial_digest": serial_digest,
        "serial_lines": list(lines),
        "process_snapshots": process_snapshots,
        "task_snapshots": task_snapshots,
        "panic_code": runtime_capture.parse_panic_code(lines),
    }


def _build_capture(
    *,
    image_path: str,
    kernel_path: str,
    panic_image_path: str,
    image_digest: str,
    kernel_digest: str,
    panic_image_digest: str,
    capture_mode: str,
    boots: Sequence[Dict[str, object]],
) -> Dict[str, object]:
    capture_id = runtime_capture.stable_digest(
        {
            "image_path": image_path,
            "kernel_path": kernel_path,
            "panic_image_path": panic_image_path,
            "image_digest": image_digest,
            "kernel_digest": kernel_digest,
            "panic_image_digest": panic_image_digest,
            "capture_mode": capture_mode,
            "boots": [
                {
                    "boot_index": boot["boot_index"],
                    "boot_profile": boot["boot_profile"],
                    "serial_digest": boot["serial_digest"],
                }
                for boot in boots
            ],
        }
    )
    trace_id = f"trace-desktop-{capture_id[:12]}"
    payload = {
        "schema": runtime_capture.BOOTED_RUNTIME_SCHEMA,
        "capture_id": capture_id,
        "build_id": f"rugo-desktop-{image_digest[:12]}",
        "execution_lane": "qemu",
        "capture_mode": capture_mode,
        "image_path": image_path,
        "image_digest": image_digest,
        "kernel_path": kernel_path,
        "kernel_digest": kernel_digest,
        "panic_image_path": panic_image_path,
        "panic_image_digest": panic_image_digest,
        "trace_id": trace_id,
        "boots": list(boots),
    }
    payload["digest"] = runtime_capture.stable_digest(payload)
    payload["trace_digest"] = runtime_capture.stable_digest(
        {
            "trace_id": trace_id,
            "capture_id": capture_id,
            "boots": [
                {
                    "boot_id": boot["boot_id"],
                    "serial_digest": boot["serial_digest"],
                }
                for boot in boots
            ],
        }
    )
    return payload


def _inject_desktop_markers(lines: Sequence[Dict[str, object]]) -> List[Dict[str, object]]:
    injected: List[Dict[str, object]] = []
    offset = 0.0
    inserted = False
    step_ms = 12.0
    for entry in lines:
        ts = round(float(entry["ts_ms"]) + offset, 3)
        line = str(entry["line"])
        injected.append({"ts_ms": ts, "line": line})
        if not inserted and "SOAKC5: mixed ok" in line:
            marker_ts = ts
            for marker in DESKTOP_MARKERS:
                marker_ts = round(marker_ts + step_ms, 3)
                injected.append({"ts_ms": marker_ts, "line": marker})
            offset += step_ms * len(DESKTOP_MARKERS)
            inserted = True
    if not inserted:
        raise ValueError("desktop fixture insertion point not found")
    return injected


def build_fixture_capture(
    *,
    image_path: str = str(DEFAULT_RELEASE_IMAGE_PATH),
    kernel_path: str = str(DEFAULT_KERNEL_PATH),
    panic_image_path: str = str(DEFAULT_PANIC_IMAGE_PATH),
) -> Dict[str, object]:
    image_path_obj = Path(image_path)
    kernel_path_obj = Path(kernel_path)
    panic_image_path_obj = Path(panic_image_path)
    image_digest = runtime_capture.maybe_sha256_file(image_path_obj, "desktop-fixture-image")
    kernel_digest = runtime_capture.maybe_sha256_file(
        kernel_path_obj,
        "desktop-fixture-kernel",
    )
    panic_image_digest = runtime_capture.maybe_sha256_file(
        panic_image_path_obj,
        "desktop-fixture-panic-image",
    )
    fixture_lines = [
        _inject_desktop_markers(boot_lines)
        for boot_lines in runtime_capture.fixture_boot_lines()
    ]
    provisional = _build_capture(
        image_path=runtime_capture.posix_path(image_path_obj),
        kernel_path=runtime_capture.posix_path(kernel_path_obj),
        panic_image_path=runtime_capture.posix_path(panic_image_path_obj),
        image_digest=image_digest,
        kernel_digest=kernel_digest,
        panic_image_digest=panic_image_digest,
        capture_mode="fixture",
        boots=[],
    )
    capture_id = str(provisional["capture_id"])
    boots = [
        _build_boot_entry(
            capture_id=capture_id,
            boot_index=index + 1,
            boot_profile=profile,
            lines=lines,
            exit_code=0,
        )
        for index, (profile, lines) in enumerate(
            zip(["cold_boot", "replay_boot"], fixture_lines)
        )
    ]
    payload = _build_capture(
        image_path=runtime_capture.posix_path(image_path_obj),
        kernel_path=runtime_capture.posix_path(kernel_path_obj),
        panic_image_path=runtime_capture.posix_path(panic_image_path_obj),
        image_digest=image_digest,
        kernel_digest=kernel_digest,
        panic_image_digest=panic_image_digest,
        capture_mode="fixture",
        boots=boots,
    )
    payload["fixture_seed"] = runtime_capture.FIXTURE_SEED
    payload["created_utc"] = "2026-03-18T00:00:00Z"
    payload["boot_profiles"] = [boot["boot_profile"] for boot in boots]
    payload["panic_boot_id"] = f"panic-{payload['capture_id'][:12]}"
    return payload


def collect_runtime_capture(
    *,
    image_path: str = str(DEFAULT_RELEASE_IMAGE_PATH),
    kernel_path: str = str(DEFAULT_KERNEL_PATH),
    panic_image_path: str = str(DEFAULT_PANIC_IMAGE_PATH),
    machine: str = runtime_capture.DEFAULT_MACHINE,
    timeout_seconds: float = runtime_capture.DEFAULT_TIMEOUT_SECONDS,
) -> Dict[str, object]:
    return runtime_capture.collect_booted_runtime(
        image_path=image_path,
        kernel_path=kernel_path,
        panic_image_path=panic_image_path,
        machine=machine,
        timeout_seconds=timeout_seconds,
    )


def _boot_by_profile(capture: Dict[str, object], profile: str) -> Dict[str, object]:
    for boot in runtime_capture.iter_boots(capture):
        if boot.get("boot_profile") == profile:
            return boot
    raise KeyError(f"missing boot profile: {profile}")


def _markers_present(boot: Dict[str, object], markers: Sequence[str]) -> bool:
    return all(runtime_capture.find_first_line_ts(boot, marker) is not None for marker in markers)


def _summary_for(checks: Sequence[Dict[str, object]], domain: str) -> Dict[str, object]:
    scoped = [check for check in checks if check["domain"] == domain]
    failures = [check for check in scoped if check["pass"] is False]
    return {
        "checks": len(scoped),
        "failures": len(failures),
        "pass": len(failures) == 0,
    }


def _report_pass(report: Mapping[str, object]) -> bool:
    gate_pass = report.get("gate_pass")
    if isinstance(gate_pass, bool):
        return gate_pass
    total_failures = report.get("total_failures")
    if isinstance(total_failures, int):
        return total_failures == 0
    return False


def _boot_component_summary(boot: Dict[str, object]) -> Dict[str, object]:
    return {
        "boot_id": boot.get("boot_id", ""),
        "boot_profile": boot.get("boot_profile", ""),
        "duration_ms": boot.get("duration_ms", 0.0),
        "desktop_markers": {
            "boot": runtime_capture.component_event_count(boot, "desktop_boot"),
            "display": runtime_capture.component_event_count(boot, "display_runtime"),
            "seat": runtime_capture.component_event_count(boot, "seat_runtime"),
            "window": runtime_capture.component_event_count(boot, "window_compositor"),
            "gui": runtime_capture.component_event_count(boot, "gui_runtime"),
            "shell": runtime_capture.component_event_count(boot, "desktop_shell"),
            "installer": runtime_capture.component_event_count(boot, "graphical_installer"),
        },
        "first_markers_ms": {
            "desktop_ready": runtime_capture.find_first_line_ts(boot, "DESKBOOT: ready"),
            "display_mode": runtime_capture.find_first_line_ts(boot, "DESKDISP: mode 1280x720@60"),
            "seat_ready": runtime_capture.find_first_line_ts(boot, "DESKSEAT: seat0 ready"),
            "focus": runtime_capture.find_first_line_ts(
                boot,
                "DESKSEAT: focus desktop.shell.launcher",
            ),
            "shell_launcher": runtime_capture.find_first_line_ts(boot, "DSHELL: launcher Files"),
            "installer": runtime_capture.find_first_line_ts(
                boot,
                "DINST: recovery entry visible",
            ),
        },
    }


def _backlog_rows(
    checks: Sequence[Dict[str, object]],
    reports: Mapping[str, Dict[str, object]],
) -> List[Dict[str, object]]:
    check_map = {check["check_id"]: check for check in checks}
    m35_pass = (
        check_map["desktop_bootstrap"]["pass"]
        and check_map["display_scanout"]["pass"]
        and check_map["input_seat"]["pass"]
        and check_map["window_compositor"]["pass"]
        and _report_pass(reports["desktop_smoke_v1"])
        and _report_pass(reports["gui_app_matrix_v1"])
    )
    m44_pass = (
        check_map["gui_runtime"]["pass"]
        and check_map["shell_workflows"]["pass"]
        and check_map["graphical_installer"]["pass"]
        and _report_pass(reports["real_gui_matrix_v2"])
        and _report_pass(reports["real_pkg_install_v2"])
        and _report_pass(reports["real_catalog_audit_v2"])
    )
    m48_pass = check_map["display_scanout"]["pass"] and _report_pass(reports["display_runtime_v1"])
    m49_pass = check_map["input_seat"]["pass"] and _report_pass(reports["input_seat_v1"])
    m50_pass = check_map["window_compositor"]["pass"] and _report_pass(reports["window_system_v1"])
    m51_pass = (
        check_map["gui_runtime"]["pass"]
        and _report_pass(reports["gui_runtime_v1"])
        and _report_pass(reports["toolkit_compat_v1"])
    )
    m52_pass = (
        check_map["shell_workflows"]["pass"]
        and check_map["graphical_installer"]["pass"]
        and _report_pass(reports["desktop_shell_v1"])
        and _report_pass(reports["graphical_installer_v1"])
    )
    return [
        {
            "backlog": "M35",
            "title": "Desktop + Interactive UX Baseline v1",
            "runtime_class": "Runtime-backed",
            "status": "pass" if m35_pass else "fail",
            "required_checks": [
                "desktop_bootstrap",
                "display_scanout",
                "input_seat",
                "window_compositor",
            ],
            "required_reports": ["desktop_smoke_v1", "gui_app_matrix_v1"],
        },
        {
            "backlog": "M44",
            "title": "Real Desktop + Ecosystem Qualification v2",
            "runtime_class": "Runtime-backed",
            "status": "pass" if m44_pass else "fail",
            "required_checks": ["gui_runtime", "shell_workflows", "graphical_installer"],
            "required_reports": [
                "real_gui_matrix_v2",
                "real_pkg_install_v2",
                "real_catalog_audit_v2",
            ],
        },
        {
            "backlog": "M48",
            "title": "Display Runtime + Scanout v1",
            "runtime_class": "Runtime-backed",
            "status": "pass" if m48_pass else "fail",
            "required_checks": ["display_scanout"],
            "required_reports": ["display_runtime_v1"],
        },
        {
            "backlog": "M49",
            "title": "Input + Seat Management v1",
            "runtime_class": "Runtime-backed",
            "status": "pass" if m49_pass else "fail",
            "required_checks": ["input_seat"],
            "required_reports": ["input_seat_v1"],
        },
        {
            "backlog": "M50",
            "title": "Window System + Composition v1",
            "runtime_class": "Runtime-backed",
            "status": "pass" if m50_pass else "fail",
            "required_checks": ["window_compositor"],
            "required_reports": ["window_system_v1"],
        },
        {
            "backlog": "M51",
            "title": "GUI Runtime + Toolkit Bridge v1",
            "runtime_class": "Runtime-backed",
            "status": "pass" if m51_pass else "fail",
            "required_checks": ["gui_runtime"],
            "required_reports": ["gui_runtime_v1", "toolkit_compat_v1"],
        },
        {
            "backlog": "M52",
            "title": "Desktop Shell + Workflow Baseline v1",
            "runtime_class": "Runtime-backed",
            "status": "pass" if m52_pass else "fail",
            "required_checks": ["shell_workflows", "graphical_installer"],
            "required_reports": ["desktop_shell_v1", "graphical_installer_v1"],
        },
    ]


def build_report(
    *,
    seed: int,
    capture: Dict[str, object],
    reports: Mapping[str, Dict[str, object]],
    injected_failures: Set[str] | None = None,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)
    cold = _boot_by_profile(capture, "cold_boot")
    replay = _boot_by_profile(capture, "replay_boot")

    checks: List[Dict[str, object]] = []
    for spec in RUNTIME_CHECKS:
        passed = _markers_present(cold, spec.cold_markers) and _markers_present(
            replay,
            spec.replay_markers,
        )
        if spec.check_id in failures:
            passed = False
        checks.append(
            {
                "check_id": spec.check_id,
                "domain": spec.domain,
                "description": spec.description,
                "cold_markers": list(spec.cold_markers),
                "replay_markers": list(spec.replay_markers),
                "pass": passed,
            }
        )

    backlog_closure = _backlog_rows(checks, reports)
    total_failures = sum(1 for check in checks if check["pass"] is False)
    total_failures += sum(1 for row in backlog_closure if row["status"] != "pass")

    summary = {
        "boot": _summary_for(checks, "boot"),
        "display": _summary_for(checks, "display"),
        "input": _summary_for(checks, "input"),
        "window": _summary_for(checks, "window"),
        "gui": _summary_for(checks, "gui"),
        "shell": _summary_for(checks, "shell"),
        "installer": _summary_for(checks, "installer"),
        "backlogs": {
            "runtime_backed": sum(1 for row in backlog_closure if row["status"] == "pass"),
            "total": len(backlog_closure),
        },
    }

    stable_payload = {
        "schema": SCHEMA,
        "capture_digest": capture.get("digest", ""),
        "seed": seed,
        "checks": [
            {"check_id": check["check_id"], "pass": check["pass"]}
            for check in checks
        ],
        "backlog_closure": [
            {"backlog": row["backlog"], "status": row["status"]}
            for row in backlog_closure
        ],
        "injected_failures": sorted(failures),
    }
    digest = runtime_capture.stable_digest(stable_payload)

    return {
        "schema": SCHEMA,
        "track_id": TRACK_ID,
        "policy_id": POLICY_ID,
        "desktop_profile_id": DESKTOP_PROFILE_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "gate": "test-desktop-profile-runtime-v1",
        "capture": {
            "schema": capture.get("schema", ""),
            "capture_id": capture.get("capture_id", ""),
            "capture_mode": capture.get("capture_mode", ""),
            "trace_id": capture.get("trace_id", ""),
            "digest": capture.get("digest", ""),
            "image_path": capture.get("image_path", ""),
            "kernel_path": capture.get("kernel_path", ""),
        },
        "checks": checks,
        "summary": summary,
        "backlog_closure": backlog_closure,
        "boot_profiles": [
            _boot_component_summary(cold),
            _boot_component_summary(replay),
        ],
        "runtime_components": {
            "display": {
                "report_schema": reports["display_runtime_v1"]["schema"],
                "digest": reports["display_runtime_v1"]["digest"],
                "active_runtime_path": reports["display_runtime_v1"]["active_runtime_path"],
                "active_runtime_driver": reports["display_runtime_v1"]["active_runtime_driver"],
            },
            "seat": {
                "report_schema": reports["input_seat_v1"]["schema"],
                "digest": reports["input_seat_v1"]["digest"],
                "seat_id": reports["input_seat_v1"]["seat"]["seat_id"],
                "focus_owner": reports["input_seat_v1"]["seat"]["focus_owner"],
            },
            "window": {
                "report_schema": reports["window_system_v1"]["schema"],
                "digest": reports["window_system_v1"]["digest"],
                "focus_owner": reports["window_system_v1"]["seat"]["focus_owner"],
                "output_id": reports["window_system_v1"]["output"]["output_id"],
            },
            "gui": {
                "report_schema": reports["gui_runtime_v1"]["schema"],
                "digest": reports["gui_runtime_v1"]["digest"],
                "toolkit_profile_id": reports["gui_runtime_v1"]["toolkit_profile_id"],
                "focus_owner": reports["gui_runtime_v1"]["runtime_topology"]["focus_owner"],
            },
            "shell": {
                "report_schema": reports["desktop_shell_v1"]["schema"],
                "digest": reports["desktop_shell_v1"]["digest"],
                "workflow_passed": reports["desktop_shell_v1"]["workflow_totals"]["passed"],
                "required_workflows": reports["desktop_shell_v1"]["workflow_totals"][
                    "required_pass"
                ],
            },
            "installer": {
                "report_schema": reports["graphical_installer_v1"]["schema"],
                "digest": reports["graphical_installer_v1"]["digest"],
                "selected_target": reports["graphical_installer_v1"]["selected_target"][
                    "device_id"
                ],
                "session_handoff_pass": reports["graphical_installer_v1"]["handoff"][
                    "session_handoff_pass"
                ],
            },
        },
        "source_reports": {
            key: {
                "schema": value.get("schema", ""),
                "digest": value.get("digest", ""),
                "gate_pass": _report_pass(value),
            }
            for key, value in reports.items()
        },
        "artifact_refs": {
            "runtime_capture": DEFAULT_RUNTIME_CAPTURE_PATH.as_posix(),
            "report": "out/desktop-profile-runtime-v1.json",
            "junit": "out/pytest-desktop-profile-runtime-v1.xml",
            "boot_image": DEFAULT_RELEASE_IMAGE_PATH.as_posix(),
            "kernel_image": DEFAULT_KERNEL_PATH.as_posix(),
            "ci_artifact": "desktop-profile-runtime-v1-artifacts",
            **SUPPORTING_REPORT_PATHS,
        },
        "injected_failures": sorted(failures),
        "failures": sorted(
            [check["check_id"] for check in checks if check["pass"] is False]
            + [row["backlog"] for row in backlog_closure if row["status"] != "pass"]
        ),
        "total_failures": total_failures,
        "gate_pass": total_failures == 0,
        "digest": digest,
    }
