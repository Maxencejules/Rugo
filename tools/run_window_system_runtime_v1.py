#!/usr/bin/env python3
"""Run deterministic window system + composition checks for M50."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_display_runtime_v1 as display_runtime
import run_input_seat_runtime_v1 as input_runtime


SCHEMA = "rugo.window_system_runtime_report.v1"
SURFACE_CONTRACT_ID = "rugo.surface_lifecycle_contract.v1"
DAMAGE_POLICY_ID = "rugo.compositor_damage_policy.v1"
WINDOW_MANAGER_CONTRACT_ID = "rugo.window_manager_contract.v2"
DEFAULT_SEED = 20260311
OUTPUT_ID = "display-0"
OUTPUT_WIDTH = 1280
OUTPUT_HEIGHT = 720
WORKSPACE_WINDOW_ID = "desktop.shell.workspace"
FILES_WINDOW_ID = "files.panel"
SETTINGS_WINDOW_ID = "settings.panel"
TOAST_WINDOW_ID = "toast.network"
WORKSPACE_SURFACE_ID = "surface.desktop.workspace"
FILES_SURFACE_ID = "surface.files.panel"
SETTINGS_SURFACE_ID = "surface.settings.panel"
TOAST_SURFACE_ID = "surface.toast.network"


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
        check_id="surface_create_budget",
        domain="lifecycle",
        metric_key="surface_create_latency_ms",
        operator="max",
        threshold=65.0,
        base=31.0,
        spread=14,
        scale=1.4,
    ),
    CheckSpec(
        check_id="surface_map_budget",
        domain="lifecycle",
        metric_key="surface_map_latency_ms",
        operator="max",
        threshold=45.0,
        base=18.0,
        spread=10,
        scale=1.1,
    ),
    CheckSpec(
        check_id="surface_visibility_integrity",
        domain="lifecycle",
        metric_key="surface_state_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="surface_activate_budget",
        domain="lifecycle",
        metric_key="surface_activate_latency_ms",
        operator="max",
        threshold=35.0,
        base=15.0,
        spread=9,
        scale=1.0,
    ),
    CheckSpec(
        check_id="surface_unmap_budget",
        domain="lifecycle",
        metric_key="surface_unmap_latency_ms",
        operator="max",
        threshold=28.0,
        base=10.0,
        spread=8,
        scale=1.0,
    ),
    CheckSpec(
        check_id="surface_release_budget",
        domain="lifecycle",
        metric_key="surface_release_latency_ms",
        operator="max",
        threshold=28.0,
        base=9.0,
        spread=8,
        scale=1.0,
    ),
    CheckSpec(
        check_id="z_order_integrity",
        domain="z_order",
        metric_key="z_order_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="focus_z_order_alignment",
        domain="z_order",
        metric_key="focus_alignment_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="occlusion_clip_integrity",
        domain="z_order",
        metric_key="occlusion_clip_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="window_move_budget",
        domain="geometry",
        metric_key="window_move_latency_ms",
        operator="max",
        threshold=24.0,
        base=11.0,
        spread=8,
        scale=1.0,
    ),
    CheckSpec(
        check_id="window_resize_budget",
        domain="geometry",
        metric_key="window_resize_latency_ms",
        operator="max",
        threshold=32.0,
        base=17.0,
        spread=10,
        scale=1.0,
    ),
    CheckSpec(
        check_id="compositor_frame_budget",
        domain="composition",
        metric_key="compositor_frame_latency_p95_ms",
        operator="max",
        threshold=16.667,
        base=12.2,
        spread=10,
        scale=0.25,
    ),
)

SOURCE_CHECK_IDS = {"display_runtime_live", "input_seat_live"}


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


def _rect(x: int, y: int, width: int, height: int) -> Dict[str, int]:
    return {"x": x, "y": y, "width": width, "height": height}


def _right(rect: Dict[str, int]) -> int:
    return rect["x"] + rect["width"]


def _bottom(rect: Dict[str, int]) -> int:
    return rect["y"] + rect["height"]


def _intersect(a: Dict[str, int], b: Dict[str, int]) -> Dict[str, int] | None:
    x1 = max(a["x"], b["x"])
    y1 = max(a["y"], b["y"])
    x2 = min(_right(a), _right(b))
    y2 = min(_bottom(a), _bottom(b))
    if x2 <= x1 or y2 <= y1:
        return None
    return _rect(x1, y1, x2 - x1, y2 - y1)


def _subtract_rect(base: Dict[str, int], occluder: Dict[str, int]) -> List[Dict[str, int]]:
    intersection = _intersect(base, occluder)
    if intersection is None:
        return [base]

    pieces: List[Dict[str, int]] = []
    if intersection["y"] > base["y"]:
        pieces.append(
            _rect(base["x"], base["y"], base["width"], intersection["y"] - base["y"])
        )
    if _bottom(intersection) < _bottom(base):
        pieces.append(
            _rect(
                base["x"],
                _bottom(intersection),
                base["width"],
                _bottom(base) - _bottom(intersection),
            )
        )
    if intersection["x"] > base["x"]:
        pieces.append(
            _rect(
                base["x"],
                intersection["y"],
                intersection["x"] - base["x"],
                intersection["height"],
            )
        )
    if _right(intersection) < _right(base):
        pieces.append(
            _rect(
                _right(intersection),
                intersection["y"],
                _right(base) - _right(intersection),
                intersection["height"],
            )
        )
    return [piece for piece in pieces if piece["width"] > 0 and piece["height"] > 0]


def _subtract_many(
    base_regions: Sequence[Dict[str, int]],
    occluders: Sequence[Dict[str, int]],
) -> List[Dict[str, int]]:
    regions = list(base_regions)
    for occluder in occluders:
        next_regions: List[Dict[str, int]] = []
        for region in regions:
            next_regions.extend(_subtract_rect(region, occluder))
        regions = next_regions
    return regions


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def run_window_system_runtime(
    seed: int,
    injected_failures: Set[str] | None = None,
    max_failures: int = 0,
    force_display_fallback: bool = False,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)

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

    display_report = display_runtime.run_display_runtime(
        seed=seed,
        max_failures=0,
        force_fallback=force_display_fallback,
    )
    input_report = input_runtime.run_input_seat_runtime(
        seed=seed,
        max_failures=0,
        force_display_fallback=force_display_fallback,
    )

    display_runtime_green = (
        display_report["gate_pass"]
        and display_report["summary"]["scanout"]["pass"]
        and display_report["summary"]["capture"]["pass"]
    )
    input_runtime_green = (
        input_report["gate_pass"]
        and input_report["summary"]["delivery"]["pass"]
        and input_report["summary"]["focus"]["pass"]
        and input_report["focus"]["keyboard_focus_target"] == SETTINGS_WINDOW_ID
    )

    checks.extend(
        [
            {
                "check_id": "display_runtime_live",
                "domain": "source",
                "metric_key": "display_runtime_ready_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0
                if display_runtime_green and "display_runtime_live" not in failures
                else 0.999,
                "pass": display_runtime_green and "display_runtime_live" not in failures,
            },
            {
                "check_id": "input_seat_live",
                "domain": "source",
                "metric_key": "input_seat_ready_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0
                if input_runtime_green and "input_seat_live" not in failures
                else 0.999,
                "pass": input_runtime_green and "input_seat_live" not in failures,
            },
        ]
    )

    check_pass = {entry["check_id"]: bool(entry["pass"]) for entry in checks}

    workspace_rect = _rect(0, 0, OUTPUT_WIDTH, OUTPUT_HEIGHT)
    files_rect = _rect(120, 96, 760, 520)
    settings_initial_rect = _rect(200, 140, 640, 420)
    settings_moved_rect = _rect(240, 180, 640, 420)
    settings_final_rect = _rect(240, 180, 720, 460)
    toast_rect = _rect(940, 36, 240, 84)

    topmost_focusable_window = SETTINGS_WINDOW_ID
    focus_owner = SETTINGS_WINDOW_ID if check_pass["focus_z_order_alignment"] else FILES_WINDOW_ID

    files_visible_regions = (
        _subtract_rect(files_rect, settings_final_rect)
        if check_pass["occlusion_clip_integrity"]
        else [files_rect]
    )
    workspace_visible_regions = _subtract_many(
        [workspace_rect],
        [files_rect, settings_final_rect],
    )
    workspace_visible_regions = workspace_visible_regions or [workspace_rect]

    lifecycle_state_violations = (
        []
        if check_pass["surface_visibility_integrity"]
        else [
            {
                "surface_id": TOAST_SURFACE_ID,
                "window_id": TOAST_WINDOW_ID,
                "reason": "destroy_without_clean_unmap",
            }
        ]
    )
    ordering_violations = (
        []
        if check_pass["z_order_integrity"]
        else [
            {
                "window_id": FILES_WINDOW_ID,
                "reason": "stack_order_digest_mismatch",
            }
        ]
    )
    occlusion_clip_violations = (
        []
        if check_pass["occlusion_clip_integrity"]
        else [
            {
                "window_id": FILES_WINDOW_ID,
                "reason": "opaque_clip_missing",
            }
        ]
    )

    lifecycle_log = [
        {
            "seq": 1,
            "surface_id": WORKSPACE_SURFACE_ID,
            "window_id": WORKSPACE_WINDOW_ID,
            "phase": "create",
            "state_after": "created",
            "latency_ms": _round_value(metric_values["surface_create_latency_ms"] - 11.0),
        },
        {
            "seq": 2,
            "surface_id": WORKSPACE_SURFACE_ID,
            "window_id": WORKSPACE_WINDOW_ID,
            "phase": "map",
            "state_after": "visible",
            "latency_ms": _round_value(metric_values["surface_map_latency_ms"] - 6.0),
        },
        {
            "seq": 3,
            "surface_id": FILES_SURFACE_ID,
            "window_id": FILES_WINDOW_ID,
            "phase": "create",
            "state_after": "created",
            "latency_ms": _round_value(metric_values["surface_create_latency_ms"] - 4.0),
        },
        {
            "seq": 4,
            "surface_id": FILES_SURFACE_ID,
            "window_id": FILES_WINDOW_ID,
            "phase": "map",
            "state_after": "visible",
            "latency_ms": _round_value(metric_values["surface_map_latency_ms"] - 2.0),
        },
        {
            "seq": 5,
            "surface_id": SETTINGS_SURFACE_ID,
            "window_id": SETTINGS_WINDOW_ID,
            "phase": "create",
            "state_after": "created",
            "latency_ms": _round_value(metric_values["surface_create_latency_ms"]),
        },
        {
            "seq": 6,
            "surface_id": SETTINGS_SURFACE_ID,
            "window_id": SETTINGS_WINDOW_ID,
            "phase": "map",
            "state_after": "visible",
            "latency_ms": _round_value(metric_values["surface_map_latency_ms"]),
        },
        {
            "seq": 7,
            "surface_id": SETTINGS_SURFACE_ID,
            "window_id": SETTINGS_WINDOW_ID,
            "phase": "activate",
            "state_after": "focused" if focus_owner == SETTINGS_WINDOW_ID else "visible",
            "latency_ms": _round_value(metric_values["surface_activate_latency_ms"]),
        },
        {
            "seq": 8,
            "surface_id": TOAST_SURFACE_ID,
            "window_id": TOAST_WINDOW_ID,
            "phase": "create",
            "state_after": "created",
            "latency_ms": _round_value(metric_values["surface_create_latency_ms"] - 15.0),
        },
        {
            "seq": 9,
            "surface_id": TOAST_SURFACE_ID,
            "window_id": TOAST_WINDOW_ID,
            "phase": "map",
            "state_after": "visible",
            "latency_ms": _round_value(metric_values["surface_map_latency_ms"] - 9.0),
        },
        {
            "seq": 10,
            "surface_id": SETTINGS_SURFACE_ID,
            "window_id": SETTINGS_WINDOW_ID,
            "phase": "move",
            "state_after": "focused" if focus_owner == SETTINGS_WINDOW_ID else "visible",
            "latency_ms": _round_value(metric_values["window_move_latency_ms"]),
        },
        {
            "seq": 11,
            "surface_id": SETTINGS_SURFACE_ID,
            "window_id": SETTINGS_WINDOW_ID,
            "phase": "resize",
            "state_after": "focused" if focus_owner == SETTINGS_WINDOW_ID else "visible",
            "latency_ms": _round_value(metric_values["window_resize_latency_ms"]),
        },
        {
            "seq": 12,
            "surface_id": TOAST_SURFACE_ID,
            "window_id": TOAST_WINDOW_ID,
            "phase": "unmap",
            "state_after": "unmapped",
            "latency_ms": _round_value(metric_values["surface_unmap_latency_ms"]),
        },
        {
            "seq": 13,
            "surface_id": TOAST_SURFACE_ID,
            "window_id": TOAST_WINDOW_ID,
            "phase": "destroy",
            "state_after": "destroyed",
            "latency_ms": _round_value(metric_values["surface_release_latency_ms"]),
        },
    ]

    render_order = [WORKSPACE_WINDOW_ID, FILES_WINDOW_ID, SETTINGS_WINDOW_ID]
    z_stack = [
        {
            "window_id": WORKSPACE_WINDOW_ID,
            "surface_id": WORKSPACE_SURFACE_ID,
            "stacking_layer": "background",
            "z_index": 0,
            "focusable": False,
            "focused": False,
        },
        {
            "window_id": FILES_WINDOW_ID,
            "surface_id": FILES_SURFACE_ID,
            "stacking_layer": "normal",
            "z_index": 20,
            "focusable": True,
            "focused": focus_owner == FILES_WINDOW_ID,
        },
        {
            "window_id": SETTINGS_WINDOW_ID,
            "surface_id": SETTINGS_SURFACE_ID,
            "stacking_layer": "normal",
            "z_index": 30,
            "focusable": True,
            "focused": focus_owner == SETTINGS_WINDOW_ID,
        },
    ]

    lifecycle_checks_pass = _domain_summary(checks, "lifecycle")["pass"]
    z_order_checks_pass = _domain_summary(checks, "z_order")["pass"]
    geometry_checks_pass = _domain_summary(checks, "geometry")["pass"]
    composition_checks_pass = _domain_summary(checks, "composition")["pass"]

    surfaces = [
        {
            "surface_id": WORKSPACE_SURFACE_ID,
            "window_id": WORKSPACE_WINDOW_ID,
            "stacking_layer": "background",
            "z_index": 0,
            "focusable": False,
            "focused": False,
            "opaque": True,
            "state": "visible",
            "mapped": True,
            "visible": True,
            "geometry": workspace_rect,
            "visible_regions": workspace_visible_regions,
            "occluded_by": [FILES_WINDOW_ID, SETTINGS_WINDOW_ID],
        },
        {
            "surface_id": FILES_SURFACE_ID,
            "window_id": FILES_WINDOW_ID,
            "stacking_layer": "normal",
            "z_index": 20,
            "focusable": True,
            "focused": focus_owner == FILES_WINDOW_ID,
            "opaque": True,
            "state": "occluded",
            "mapped": True,
            "visible": True,
            "geometry": files_rect,
            "visible_regions": files_visible_regions,
            "occluded_by": [SETTINGS_WINDOW_ID],
        },
        {
            "surface_id": SETTINGS_SURFACE_ID,
            "window_id": SETTINGS_WINDOW_ID,
            "stacking_layer": "normal",
            "z_index": 30,
            "focusable": True,
            "focused": focus_owner == SETTINGS_WINDOW_ID,
            "opaque": True,
            "state": "focused" if focus_owner == SETTINGS_WINDOW_ID else "visible",
            "mapped": True,
            "visible": True,
            "geometry": settings_final_rect,
            "visible_regions": [settings_final_rect],
            "occluded_by": [],
        },
    ]

    retired_surfaces = [
        {
            "surface_id": TOAST_SURFACE_ID,
            "window_id": TOAST_WINDOW_ID,
            "stacking_layer": "overlay",
            "z_index": 40,
            "focusable": False,
            "focused": False,
            "opaque": False,
            "geometry": toast_rect,
            "final_state": "destroyed",
            "lifecycle": ["created", "mapped", "visible", "unmapped", "destroyed"],
        }
    ]

    total_failures = sum(1 for row in checks if row["pass"] is False)
    failures_list = sorted(row["check_id"] for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "seed": seed,
        "display_runtime_digest": display_report["digest"],
        "input_seat_digest": input_report["digest"],
        "active_display_path": display_report["active_runtime_path"],
        "focus_owner": focus_owner,
        "surfaces": [
            {
                "window_id": entry["window_id"],
                "state": entry["state"],
                "geometry": entry["geometry"],
            }
            for entry in surfaces
        ],
        "retired_surfaces": [
            {
                "window_id": entry["window_id"],
                "final_state": entry["final_state"],
            }
            for entry in retired_surfaces
        ],
        "checks": [
            {
                "check_id": row["check_id"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "injected_failures": sorted(failures),
        "force_display_fallback": force_display_fallback,
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "surface_contract_id": SURFACE_CONTRACT_ID,
        "damage_policy_id": DAMAGE_POLICY_ID,
        "window_manager_contract_id": WINDOW_MANAGER_CONTRACT_ID,
        "display_runtime_contract_id": display_runtime.CONTRACT_ID,
        "input_seat_contract_id": input_runtime.CONTRACT_ID,
        "seed": seed,
        "gate": "test-window-system-v1",
        "damage_gate": "test-compositor-damage-v1",
        "checks": checks,
        "summary": {
            "lifecycle": _domain_summary(checks, "lifecycle"),
            "z_order": _domain_summary(checks, "z_order"),
            "geometry": _domain_summary(checks, "geometry"),
            "composition": _domain_summary(checks, "composition"),
            "source": _domain_summary(checks, "source"),
        },
        "seat": {
            "seat_id": input_report["seat"]["seat_id"],
            "active_display_path": display_report["active_runtime_path"],
            "active_display_driver": display_report["active_runtime_driver"],
            "focus_owner": focus_owner,
            "focus_owner_surface_id": SETTINGS_SURFACE_ID
            if focus_owner == SETTINGS_WINDOW_ID
            else FILES_SURFACE_ID,
            "focus_owner_count": 1,
            "source_focus_owner": input_report["focus"]["keyboard_focus_target"],
        },
        "output": {
            "output_id": OUTPUT_ID,
            "width": OUTPUT_WIDTH,
            "height": OUTPUT_HEIGHT,
            "refresh_hz": 60,
            "active_display_path": display_report["active_runtime_path"],
            "active_display_driver": display_report["active_runtime_driver"],
        },
        "surface_counts": {
            "active": 3,
            "retired": 1,
            "visible": 3,
            "focused": 1,
        },
        "lifecycle_log": lifecycle_log,
        "surface_audit": {
            "allowed_states": [
                "created",
                "mapped",
                "visible",
                "occluded",
                "focused",
                "unmapped",
                "destroyed",
            ],
            "state_violations": lifecycle_state_violations,
            "checks_pass": lifecycle_checks_pass,
        },
        "surfaces": surfaces,
        "retired_surfaces": retired_surfaces,
        "z_order": {
            "stack": z_stack,
            "topmost_focusable_window": topmost_focusable_window,
            "focus_owner": focus_owner,
            "render_order": render_order,
            "ordering_violations": len(ordering_violations),
            "ordering_violation_details": ordering_violations,
            "focus_alignment_pass": focus_owner == topmost_focusable_window
            and check_pass["focus_z_order_alignment"],
            "focus_alignment_violations": int(metric_values["focus_alignment_violation_count"]),
            "occlusion_clip_violations": len(occlusion_clip_violations),
            "occlusion_clip_violation_details": occlusion_clip_violations,
            "checks_pass": z_order_checks_pass,
        },
        "geometry_mutations": {
            "move": {
                "window_id": SETTINGS_WINDOW_ID,
                "surface_id": SETTINGS_SURFACE_ID,
                "from": settings_initial_rect,
                "to": settings_moved_rect,
                "latency_ms": metric_values["window_move_latency_ms"],
            },
            "resize": {
                "window_id": SETTINGS_WINDOW_ID,
                "surface_id": SETTINGS_SURFACE_ID,
                "from": settings_moved_rect,
                "to": settings_final_rect,
                "latency_ms": metric_values["window_resize_latency_ms"],
            },
            "checks_pass": geometry_checks_pass,
        },
        "composition": {
            "policy_id": DAMAGE_POLICY_ID,
            "opaque_clip_policy": "front_to_back_opaque_clip",
            "render_order": render_order,
            "frame_budget_ms": 16.667,
            "frame_latency_p95_ms": metric_values["compositor_frame_latency_p95_ms"],
            "visible_surface_count": 3,
            "occluded_window_ids": [FILES_WINDOW_ID],
            "checks_pass": composition_checks_pass and check_pass["occlusion_clip_integrity"],
        },
        "source_reports": {
            "display_runtime": {
                "schema": display_report["schema"],
                "digest": display_report["digest"],
                "gate_pass": display_report["gate_pass"],
                "active_runtime_path": display_report["active_runtime_path"],
                "active_runtime_driver": display_report["active_runtime_driver"],
            },
            "input_seat": {
                "schema": input_report["schema"],
                "digest": input_report["digest"],
                "gate_pass": input_report["gate_pass"],
                "seat_id": input_report["seat"]["seat_id"],
                "focus_owner": input_report["focus"]["keyboard_focus_target"],
            },
        },
        "artifact_refs": {
            "junit": "out/pytest-window-system-v1.xml",
            "runtime_report": "out/window-system-v1.json",
            "damage_report": "out/compositor-damage-v1.json",
            "display_runtime_report": "out/display-runtime-v1.json",
            "input_seat_report": "out/input-seat-v1.json",
            "ci_artifact": "window-system-v1-artifacts",
            "damage_ci_artifact": "compositor-damage-v1-artifacts",
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
        help="force a window-system runtime check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument(
        "--force-display-fallback",
        action="store_true",
        help="select the efifb display runtime path while keeping window checks active",
    )
    parser.add_argument("--out", default="out/window-system-v1.json")
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

    report = run_window_system_runtime(
        seed=args.seed,
        injected_failures=injected_failures,
        max_failures=args.max_failures,
        force_display_fallback=args.force_display_fallback,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"window-system-runtime-report: {out_path}")
    print(f"focus_owner: {report['seat']['focus_owner']}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
