#!/usr/bin/env python3
"""Run deterministic desktop shell and session workflow checks for M52."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_gui_runtime_v1 as gui_runtime
import run_toolkit_compat_v1 as toolkit_compat


SCHEMA = "rugo.desktop_shell_workflow_report.v1"
CONTRACT_ID = "rugo.desktop_shell_contract.v1"
WORKFLOW_PROFILE_ID = "rugo.session_workflow_profile.v1"
GRAPHICAL_INSTALLER_SCHEMA = "rugo.graphical_installer_smoke_report.v1"
DEFAULT_SEED = 20260311
LAUNCHER_ID = "desktop.shell.launcher"
TASKBAR_ID = "desktop.shell.taskbar"
STATUS_BAR_ID = "desktop.shell.status_bar"
POWER_MENU_ID = "desktop.shell.power_menu"
WORKSPACE_ID = "desktop.shell.workspace"
FILES_WINDOW_ID = "files.panel"
SETTINGS_WINDOW_ID = "settings.panel"


@dataclass(frozen=True)
class CheckSpec:
    check_id: str
    domain: str
    metric_key: str
    operator: str
    threshold: float
    base: float
    spread: int
    scale: float


BASE_CHECKS: Sequence[CheckSpec] = (
    CheckSpec(
        check_id="launcher_open_budget",
        domain="launcher",
        metric_key="launcher_open_latency_p95_ms",
        operator="max",
        threshold=60.0,
        base=26.0,
        spread=11,
        scale=2.1,
    ),
    CheckSpec(
        check_id="launcher_activation_integrity",
        domain="launcher",
        metric_key="launcher_activation_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="app_switch_latency_budget",
        domain="launcher",
        metric_key="app_switch_latency_p95_ms",
        operator="max",
        threshold=42.0,
        base=18.0,
        spread=10,
        scale=1.5,
    ),
    CheckSpec(
        check_id="shell_focus_restore_integrity",
        domain="settings",
        metric_key="focus_restore_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="file_picker_roundtrip_budget",
        domain="files",
        metric_key="file_picker_roundtrip_p95_ms",
        operator="max",
        threshold=80.0,
        base=41.0,
        spread=12,
        scale=2.0,
    ),
    CheckSpec(
        check_id="file_save_commit_budget",
        domain="files",
        metric_key="file_save_commit_p95_ms",
        operator="max",
        threshold=95.0,
        base=48.0,
        spread=12,
        scale=2.4,
    ),
    CheckSpec(
        check_id="settings_apply_budget",
        domain="settings",
        metric_key="settings_apply_latency_p95_ms",
        operator="max",
        threshold=85.0,
        base=35.0,
        spread=12,
        scale=2.3,
    ),
    CheckSpec(
        check_id="settings_persist_integrity",
        domain="settings",
        metric_key="settings_persist_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="shutdown_request_budget",
        domain="shutdown",
        metric_key="shutdown_request_latency_p95_ms",
        operator="max",
        threshold=55.0,
        base=22.0,
        spread=8,
        scale=1.4,
    ),
    CheckSpec(
        check_id="shutdown_surface_cleanup_integrity",
        domain="shutdown",
        metric_key="shutdown_cleanup_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
)

SOURCE_CHECK_IDS = {"gui_runtime_live", "toolkit_compat_live"}


def known_checks() -> Set[str]:
    return {spec.check_id for spec in BASE_CHECKS} | SOURCE_CHECK_IDS


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _round_value(value: float) -> float:
    return round(value, 3)


def _baseline_observed(seed: int, spec: CheckSpec) -> float:
    spread = spec.spread if spec.spread > 0 else 1
    value = spec.base + ((_noise(seed, spec.check_id) % spread) * spec.scale)
    return _round_value(value)


def _failing_observed(operator: str, threshold: float, scale: float) -> float:
    delta = 1.0 if scale == 0.0 and float(threshold).is_integer() else (
        0.001 if scale < 1.0 else 1.0
    )
    if operator == "max":
        return _round_value(threshold + delta)
    if operator == "min":
        return _round_value(threshold - delta)
    return _round_value(threshold + delta)


def _passes(operator: str, observed: float, threshold: float) -> bool:
    if operator == "max":
        return observed <= threshold
    if operator == "min":
        return observed >= threshold
    if operator == "eq":
        return observed == threshold
    raise ValueError(f"unsupported operator: {operator}")


def _domain_summary(checks: List[Dict[str, object]], domain: str) -> Dict[str, object]:
    scoped = [entry for entry in checks if entry["domain"] == domain]
    failures = [entry for entry in scoped if entry["pass"] is False]
    return {
        "checks": len(scoped),
        "failures": len(failures),
        "pass": len(failures) == 0,
    }


def _metric(seed: int, key: str, label: str, base: float, spread: int, scale: float) -> float:
    return _round_value(base + ((_noise(seed, f"{key}|{label}") % spread) * scale))


def _count(seed: int, key: str, label: str, base: int, spread: int) -> int:
    return base + (_noise(seed, f"{key}|{label}") % spread)


def _step(name: str, ok: bool, latency_ms: float, target: str) -> Dict[str, object]:
    return {
        "name": name,
        "status": "pass" if ok else "fail",
        "latency_ms": latency_ms,
        "target": target,
    }


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def run_desktop_shell_workflows(
    seed: int,
    injected_failures: Set[str] | None = None,
    max_failures: int = 0,
    force_display_fallback: bool = False,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)

    gui_report = gui_runtime.run_gui_runtime(
        seed=seed,
        max_failures=0,
        force_display_fallback=force_display_fallback,
    )
    toolkit_report = toolkit_compat.run_toolkit_compat(
        seed=seed,
        max_failures=0,
        force_display_fallback=force_display_fallback,
    )

    checks: List[Dict[str, object]] = []
    metric_values: Dict[str, float] = {}
    for spec in BASE_CHECKS:
        observed = (
            _failing_observed(spec.operator, spec.threshold, spec.scale)
            if spec.check_id in failures
            else _baseline_observed(seed, spec)
        )
        passed = _passes(spec.operator, observed, spec.threshold)
        checks.append(
            {
                "check_id": spec.check_id,
                "domain": spec.domain,
                "metric_key": spec.metric_key,
                "operator": spec.operator,
                "threshold": spec.threshold,
                "observed": observed,
                "pass": passed,
            }
        )
        metric_values[spec.metric_key] = observed

    gui_runtime_green = (
        gui_report["gate_pass"]
        and gui_report["summary"]["launch"]["pass"]
        and gui_report["summary"]["render"]["pass"]
        and gui_report["summary"]["text"]["pass"]
        and gui_report["summary"]["event_loop"]["pass"]
    )
    toolkit_green = toolkit_report["gate_pass"] and len(toolkit_report["issues"]) == 0

    checks.extend(
        [
            {
                "check_id": "gui_runtime_live",
                "domain": "source",
                "metric_key": "gui_runtime_ready_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0
                if gui_runtime_green and "gui_runtime_live" not in failures
                else 0.999,
                "pass": gui_runtime_green and "gui_runtime_live" not in failures,
            },
            {
                "check_id": "toolkit_compat_live",
                "domain": "source",
                "metric_key": "toolkit_compat_ready_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0
                if toolkit_green and "toolkit_compat_live" not in failures
                else 0.999,
                "pass": toolkit_green and "toolkit_compat_live" not in failures,
            },
        ]
    )

    check_pass = {entry["check_id"]: bool(entry["pass"]) for entry in checks}

    launcher_workflow_pass = all(
        check_pass[name]
        for name in (
            "launcher_open_budget",
            "launcher_activation_integrity",
            "app_switch_latency_budget",
            "gui_runtime_live",
            "toolkit_compat_live",
        )
    )
    file_workflow_pass = all(
        check_pass[name]
        for name in (
            "file_picker_roundtrip_budget",
            "file_save_commit_budget",
            "gui_runtime_live",
        )
    )
    settings_workflow_pass = all(
        check_pass[name]
        for name in (
            "settings_apply_budget",
            "settings_persist_integrity",
            "shell_focus_restore_integrity",
            "gui_runtime_live",
        )
    )
    shutdown_workflow_pass = all(
        check_pass[name]
        for name in (
            "shutdown_request_budget",
            "shutdown_surface_cleanup_integrity",
            "gui_runtime_live",
        )
    )

    launcher_workflow = {
        "workflow_id": "launcher_open",
        "category": "launcher",
        "start_focus": LAUNCHER_ID,
        "end_focus": FILES_WINDOW_ID,
        "resulting_window_id": FILES_WINDOW_ID,
        "steps": [
            _step(
                "open_launcher",
                check_pass["launcher_open_budget"] and check_pass["launcher_activation_integrity"],
                metric_values["launcher_open_latency_p95_ms"],
                LAUNCHER_ID,
            ),
            _step(
                "search_files_panel",
                check_pass["launcher_activation_integrity"],
                _metric(seed, "launcher_open", "search", base=11.0, spread=6, scale=1.0),
                FILES_WINDOW_ID,
            ),
            _step(
                "activate_files_panel",
                check_pass["app_switch_latency_budget"],
                metric_values["app_switch_latency_p95_ms"],
                FILES_WINDOW_ID,
            ),
        ],
        "search_term": "Files",
        "matched_items": ["files.panel"],
        "taskbar_pin_visible": True,
        "checks_pass": launcher_workflow_pass,
    }

    opened_path = "/home/demo/Documents/plan-v1.txt"
    save_path = "/home/demo/Documents/plan-v1-saved.txt"
    file_open_save_workflow = {
        "workflow_id": "file_open_save",
        "category": "files",
        "start_focus": FILES_WINDOW_ID,
        "end_focus": FILES_WINDOW_ID,
        "resulting_window_id": FILES_WINDOW_ID,
        "steps": [
            _step(
                "open_picker",
                check_pass["file_picker_roundtrip_budget"],
                metric_values["file_picker_roundtrip_p95_ms"],
                FILES_WINDOW_ID,
            ),
            _step(
                "load_document",
                check_pass["file_picker_roundtrip_budget"],
                _metric(seed, "file_open_save", "load", base=24.0, spread=8, scale=1.6),
                opened_path,
            ),
            _step(
                "save_document",
                check_pass["file_save_commit_budget"],
                metric_values["file_save_commit_p95_ms"],
                save_path,
            ),
        ],
        "opened_path": opened_path,
        "save_path": save_path,
        "bytes_loaded": _count(seed, "file_open_save", "load-bytes", base=3072, spread=768),
        "bytes_saved": _count(seed, "file_open_save", "save-bytes", base=3328, spread=768),
        "dirty_before_save": True,
        "dirty_after_save": False,
        "recent_files": [save_path, opened_path],
        "save_revision": 2,
        "checks_pass": file_workflow_pass,
    }

    settings_update_workflow = {
        "workflow_id": "settings_update",
        "category": "settings",
        "start_focus": LAUNCHER_ID,
        "end_focus": SETTINGS_WINDOW_ID,
        "resulting_window_id": SETTINGS_WINDOW_ID,
        "steps": [
            _step(
                "activate_settings_panel",
                check_pass["app_switch_latency_budget"],
                _metric(seed, "settings_update", "activate", base=19.0, spread=7, scale=1.2),
                SETTINGS_WINDOW_ID,
            ),
            _step(
                "apply_display_changes",
                check_pass["settings_apply_budget"],
                metric_values["settings_apply_latency_p95_ms"],
                SETTINGS_WINDOW_ID,
            ),
            _step(
                "persist_session_state",
                check_pass["settings_persist_integrity"],
                _metric(seed, "settings_update", "persist", base=17.0, spread=7, scale=1.1),
                "config://desktop/session.json",
            ),
            _step(
                "restore_focus",
                check_pass["shell_focus_restore_integrity"],
                _metric(seed, "settings_update", "restore", base=10.0, spread=5, scale=0.9),
                SETTINGS_WINDOW_ID,
            ),
        ],
        "section": "display",
        "changed_keys": ["accent_color", "scale_percent"],
        "previous_values": {"accent_color": "blue", "scale_percent": 100},
        "new_values": {"accent_color": "amber", "scale_percent": 125},
        "persisted_config_path": "/system/session/config/display.json",
        "persisted": check_pass["settings_persist_integrity"],
        "checks_pass": settings_workflow_pass,
    }

    shutdown_request_workflow = {
        "workflow_id": "shutdown_request",
        "category": "shutdown",
        "start_focus": POWER_MENU_ID,
        "end_focus": POWER_MENU_ID,
        "resulting_window_id": POWER_MENU_ID,
        "steps": [
            _step(
                "open_power_menu",
                check_pass["shutdown_request_budget"],
                metric_values["shutdown_request_latency_p95_ms"],
                POWER_MENU_ID,
            ),
            _step(
                "show_pending_save_guard",
                check_pass["shutdown_surface_cleanup_integrity"],
                _metric(seed, "shutdown_request", "guard", base=13.0, spread=6, scale=1.0),
                FILES_WINDOW_ID,
            ),
            _step(
                "queue_shutdown",
                check_pass["shutdown_surface_cleanup_integrity"],
                _metric(seed, "shutdown_request", "queue", base=9.0, spread=5, scale=0.8),
                POWER_MENU_ID,
            ),
        ],
        "confirmation_required": True,
        "pending_save_guard": True,
        "blocked_on_dirty_windows": [FILES_WINDOW_ID],
        "cleanup_targets": [FILES_WINDOW_ID, SETTINGS_WINDOW_ID, WORKSPACE_ID],
        "ready_for_shutdown": check_pass["shutdown_surface_cleanup_integrity"],
        "checks_pass": shutdown_workflow_pass,
    }

    workflows = [
        launcher_workflow,
        file_open_save_workflow,
        settings_update_workflow,
        shutdown_request_workflow,
    ]

    workflow_totals = {
        "declared": len(workflows),
        "passed": sum(1 for workflow in workflows if workflow["checks_pass"]),
        "required_pass": 4,
    }

    total_failures = sum(1 for row in checks if row["pass"] is False)
    failures_list = sorted(row["check_id"] for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "workflow_profile_id": WORKFLOW_PROFILE_ID,
        "seed": seed,
        "checks": [
            {
                "check_id": row["check_id"],
                "domain": row["domain"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "workflows": [
            {
                "workflow_id": workflow["workflow_id"],
                "checks_pass": workflow["checks_pass"],
                "resulting_window_id": workflow["resulting_window_id"],
            }
            for workflow in workflows
        ],
        "gui_runtime_digest": gui_report["digest"],
        "toolkit_compat_digest": toolkit_report["digest"],
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "contract_id": CONTRACT_ID,
        "workflow_profile_id": WORKFLOW_PROFILE_ID,
        "gui_runtime_contract_id": gui_runtime.CONTRACT_ID,
        "gui_runtime_schema": gui_runtime.SCHEMA,
        "toolkit_compat_schema": toolkit_compat.SCHEMA,
        "graphical_installer_schema": GRAPHICAL_INSTALLER_SCHEMA,
        "seed": seed,
        "gate": "test-desktop-shell-v1",
        "workflow_gate": "test-desktop-workflows-v1",
        "checks": checks,
        "summary": {
            "launcher": _domain_summary(checks, "launcher"),
            "files": _domain_summary(checks, "files"),
            "settings": _domain_summary(checks, "settings"),
            "shutdown": _domain_summary(checks, "shutdown"),
            "source": _domain_summary(checks, "source"),
        },
        "workflow_totals": workflow_totals,
        "shell_components": [
            {
                "component_id": WORKSPACE_ID,
                "role": "workspace",
                "visible": True,
                "interactive": True,
            },
            {
                "component_id": LAUNCHER_ID,
                "role": "launcher",
                "visible": False,
                "interactive": True,
            },
            {
                "component_id": TASKBAR_ID,
                "role": "taskbar",
                "visible": True,
                "interactive": True,
            },
            {
                "component_id": STATUS_BAR_ID,
                "role": "status-bar",
                "visible": True,
                "interactive": False,
            },
            {
                "component_id": POWER_MENU_ID,
                "role": "power-menu",
                "visible": False,
                "interactive": True,
            },
        ],
        "session_state": {
            "seat_id": gui_report["runtime_topology"]["seat_id"],
            "active_display_path": gui_report["runtime_topology"]["active_display_path"],
            "active_display_driver": gui_report["runtime_topology"]["active_display_driver"],
            "focus_owner": SETTINGS_WINDOW_ID,
            "previous_focus_owner": LAUNCHER_ID,
            "workspace_surface_id": "surface.desktop.workspace",
            "visible_windows": [WORKSPACE_ID, FILES_WINDOW_ID, SETTINGS_WINDOW_ID],
            "launcher_last_query": "Files",
            "notifications_visible": 0,
            "pending_shutdown_guard": True,
        },
        "workflows": workflows,
        "source_reports": {
            "gui_runtime": {
                "schema": gui_report["schema"],
                "digest": gui_report["digest"],
                "gate_pass": gui_report["gate_pass"],
                "focus_owner": gui_report["runtime_topology"]["focus_owner"],
                "active_display_path": gui_report["runtime_topology"]["active_display_path"],
            },
            "toolkit_compat": {
                "schema": toolkit_report["schema"],
                "digest": toolkit_report["digest"],
                "gate_pass": toolkit_report["gate_pass"],
                "issues": len(toolkit_report["issues"]),
                "parent_runtime_digest": toolkit_report["source_reports"]["gui_runtime"]["digest"],
            },
        },
        "artifact_refs": {
            "junit": "out/pytest-desktop-shell-v1.xml",
            "workflow_report": "out/desktop-shell-v1.json",
            "graphical_installer_report": "out/graphical-installer-v1.json",
            "gui_runtime_report": "out/gui-runtime-v1.json",
            "toolkit_compat_report": "out/toolkit-compat-v1.json",
            "ci_artifact": "desktop-shell-v1-artifacts",
            "workflow_ci_artifact": "desktop-workflows-v1-artifacts",
        },
        "injected_failures": sorted(failures),
        "force_display_fallback": force_display_fallback,
        "max_failures": max_failures,
        "total_failures": total_failures,
        "failures": failures_list,
        "gate_pass": gate_pass,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a shell workflow check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument(
        "--force-display-fallback",
        action="store_true",
        help="select the efifb display runtime path while keeping shell workflows active",
    )
    parser.add_argument("--out", default="out/desktop-shell-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    try:
        injected_failures = normalize_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_desktop_shell_workflows(
        seed=args.seed,
        injected_failures=injected_failures,
        max_failures=args.max_failures,
        force_display_fallback=args.force_display_fallback,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"desktop-shell-report: {out_path}")
    print(f"workflow_passed: {report['workflow_totals']['passed']}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
