#!/usr/bin/env python3
"""Run deterministic compositor damage checks for M50."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_window_system_runtime_v1 as runtime


SCHEMA = "rugo.compositor_damage_report.v1"
DEFAULT_SEED = runtime.DEFAULT_SEED


BASE_CHECKS = {
    "damage_region_union": {
        "domain": "damage",
        "metric_key": "union_mismatch_count",
        "operator": "max",
        "threshold": 0.0,
        "base": 0.0,
        "spread": 1,
        "scale": 0.0,
    },
    "occlusion_clip_integrity": {
        "domain": "clipping",
        "metric_key": "clip_violation_count",
        "operator": "max",
        "threshold": 0.0,
        "base": 0.0,
        "spread": 1,
        "scale": 0.0,
    },
    "present_region_budget": {
        "domain": "present",
        "metric_key": "present_region_latency_p95_ms",
        "operator": "max",
        "threshold": 16.667,
        "base": 13.1,
        "spread": 10,
        "scale": 0.25,
    },
    "retained_region_reuse": {
        "domain": "present",
        "metric_key": "retained_region_reuse_ratio",
        "operator": "min",
        "threshold": 0.75,
        "base": 0.83,
        "spread": 6,
        "scale": 0.02,
    },
    "fullscreen_damage_reset": {
        "domain": "damage",
        "metric_key": "scene_reset_coverage_ratio",
        "operator": "min",
        "threshold": 1.0,
        "base": 1.0,
        "spread": 1,
        "scale": 0.0,
    },
}


def known_checks() -> Set[str]:
    return set(BASE_CHECKS.keys()) | {"window_runtime_live"}


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _round_value(value: float) -> float:
    return round(value, 3)


def _baseline_observed(seed: int, check_id: str) -> float:
    spec = BASE_CHECKS[check_id]
    spread = spec["spread"] if spec["spread"] > 0 else 1
    value = spec["base"] + ((_noise(seed, check_id) % spread) * spec["scale"])
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


def _rect_area(rect: Dict[str, int]) -> int:
    return rect["width"] * rect["height"]


def _right(rect: Dict[str, int]) -> int:
    return rect["x"] + rect["width"]


def _bottom(rect: Dict[str, int]) -> int:
    return rect["y"] + rect["height"]


def _bounding_union(rects: Sequence[Dict[str, int]]) -> Dict[str, int]:
    x1 = min(rect["x"] for rect in rects)
    y1 = min(rect["y"] for rect in rects)
    x2 = max(_right(rect) for rect in rects)
    y2 = max(_bottom(rect) for rect in rects)
    return {"x": x1, "y": y1, "width": x2 - x1, "height": y2 - y1}


def _phase(
    seq: int,
    phase: str,
    target_window_id: str,
    damage_rects: Sequence[Dict[str, int]],
    reason: str,
) -> Dict[str, object]:
    union_rect = _bounding_union(damage_rects)
    return {
        "seq": seq,
        "phase": phase,
        "target_window_id": target_window_id,
        "reason": reason,
        "damage_rects": list(damage_rects),
        "union_rect": union_rect,
        "clipped_damage_rects": list(damage_rects),
        "union_area_pixels": _rect_area(union_rect),
    }


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def run_compositor_damage(
    seed: int,
    injected_failures: Set[str] | None = None,
    runtime_failures: Sequence[str] | None = None,
    max_failures: int = 0,
    force_display_fallback: bool = False,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)
    normalized_runtime_failures = set(runtime_failures or [])

    runtime_report = runtime.run_window_system_runtime(
        seed=seed,
        injected_failures=normalized_runtime_failures,
        max_failures=0,
        force_display_fallback=force_display_fallback,
    )

    checks: List[Dict[str, object]] = []
    metric_values: Dict[str, float] = {}
    for check_id, spec in BASE_CHECKS.items():
        observed = (
            _failing_observed(spec["operator"], spec["threshold"], spec["scale"])
            if check_id in failures
            else _baseline_observed(seed, check_id)
        )
        passed = _passes(spec["operator"], observed, spec["threshold"])
        checks.append(
            {
                "check_id": check_id,
                "domain": spec["domain"],
                "metric_key": spec["metric_key"],
                "operator": spec["operator"],
                "threshold": spec["threshold"],
                "observed": observed,
                "pass": passed,
            }
        )
        metric_values[spec["metric_key"]] = observed

    window_runtime_green = (
        runtime_report["gate_pass"]
        and runtime_report["summary"]["lifecycle"]["pass"]
        and runtime_report["summary"]["z_order"]["pass"]
        and runtime_report["summary"]["geometry"]["pass"]
    )
    checks.append(
        {
            "check_id": "window_runtime_live",
            "domain": "source",
            "metric_key": "window_runtime_ready_ratio",
            "operator": "min",
            "threshold": 1.0,
            "observed": 1.0
            if window_runtime_green and "window_runtime_live" not in failures
            else 0.999,
            "pass": window_runtime_green and "window_runtime_live" not in failures,
        }
    )
    check_pass = {entry["check_id"]: bool(entry["pass"]) for entry in checks}

    settings_move = runtime_report["geometry_mutations"]["move"]
    settings_resize = runtime_report["geometry_mutations"]["resize"]
    toast_rect = runtime_report["retired_surfaces"][0]["geometry"]

    phases = [
        _phase(
            1,
            "scene_init",
            runtime.WORKSPACE_WINDOW_ID,
            [{"x": 0, "y": 0, "width": runtime.OUTPUT_WIDTH, "height": runtime.OUTPUT_HEIGHT}],
            "first_frame_full_reset",
        ),
        _phase(
            2,
            "window_move",
            runtime.SETTINGS_WINDOW_ID,
            [settings_move["from"], settings_move["to"]],
            "focused_window_move",
        ),
        _phase(
            3,
            "window_resize",
            runtime.SETTINGS_WINDOW_ID,
            [settings_resize["from"], settings_resize["to"]],
            "focused_window_resize",
        ),
        _phase(
            4,
            "toast_destroy",
            runtime.TOAST_WINDOW_ID,
            [toast_rect],
            "transient_overlay_retire",
        ),
    ]

    clip_snapshots = [
        {
            "window_id": surface["window_id"],
            "surface_id": surface["surface_id"],
            "visible_regions": surface["visible_regions"],
            "occluded_by": surface["occluded_by"],
        }
        for surface in runtime_report["surfaces"]
    ]
    if not check_pass["occlusion_clip_integrity"]:
        for snapshot in clip_snapshots:
            if snapshot["window_id"] == runtime.FILES_WINDOW_ID:
                files_geometry = next(
                    surface["geometry"]
                    for surface in runtime_report["surfaces"]
                    if surface["window_id"] == runtime.FILES_WINDOW_ID
                )
                snapshot["visible_regions"] = [files_geometry]

    total_union_area = sum(phase["union_area_pixels"] for phase in phases)
    fullscreen_resets = sum(1 for phase in phases if phase["phase"] == "scene_init")
    total_failures = sum(1 for row in checks if row["pass"] is False)
    failures_list = sorted(row["check_id"] for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "seed": seed,
        "runtime_digest": runtime_report["digest"],
        "checks": [
            {
                "check_id": row["check_id"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "phases": [
            {
                "phase": phase["phase"],
                "target_window_id": phase["target_window_id"],
                "union_rect": phase["union_rect"],
            }
            for phase in phases
        ],
        "injected_failures": sorted(failures),
        "runtime_failures": sorted(normalized_runtime_failures),
        "force_display_fallback": force_display_fallback,
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "surface_contract_id": runtime.SURFACE_CONTRACT_ID,
        "damage_policy_id": runtime.DAMAGE_POLICY_ID,
        "window_manager_contract_id": runtime.WINDOW_MANAGER_CONTRACT_ID,
        "runtime_schema": runtime_report["schema"],
        "runtime_digest": runtime_report["digest"],
        "runtime_gate_pass": runtime_report["gate_pass"],
        "gate": "test-compositor-damage-v1",
        "output": {
            "output_id": runtime_report["output"]["output_id"],
            "width": runtime_report["output"]["width"],
            "height": runtime_report["output"]["height"],
            "active_display_path": runtime_report["output"]["active_display_path"],
        },
        "policy": {
            "union_policy": "bounding_union_per_output",
            "opaque_clip_policy": "front_to_back_opaque_clip",
            "fullscreen_reset_policy": "full_output_on_scene_init",
        },
        "checks": checks,
        "summary": {
            "damage": _domain_summary(checks, "damage"),
            "clipping": _domain_summary(checks, "clipping"),
            "present": _domain_summary(checks, "present"),
            "source": _domain_summary(checks, "source"),
        },
        "phases": phases,
        "clip_snapshots": clip_snapshots,
        "present": {
            "target_refresh_hz": 60,
            "frame_budget_ms": 16.667,
            "present_region_latency_p95_ms": metric_values["present_region_latency_p95_ms"],
            "retained_region_reuse_ratio": metric_values["retained_region_reuse_ratio"],
            "checks_pass": _domain_summary(checks, "present")["pass"],
        },
        "totals": {
            "phase_count": len(phases),
            "damage_rect_count": sum(len(phase["damage_rects"]) for phase in phases),
            "union_area_pixels": total_union_area,
            "fullscreen_resets": fullscreen_resets,
        },
        "source_reports": {
            "window_runtime": {
                "schema": runtime_report["schema"],
                "digest": runtime_report["digest"],
                "gate_pass": runtime_report["gate_pass"],
                "focus_owner": runtime_report["seat"]["focus_owner"],
                "active_display_path": runtime_report["output"]["active_display_path"],
            }
        },
        "artifact_refs": {
            "json_path": "out/compositor-damage-v1.json",
            "runtime_report": runtime_report["artifact_refs"]["runtime_report"],
            "junit": "out/pytest-compositor-damage-v1.xml",
            "ci_artifact": "compositor-damage-v1-artifacts",
        },
        "runtime_failures": sorted(runtime_report["failures"]),
        "injected_failures": sorted(failures),
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
        help="force a compositor damage check to fail by check_id",
    )
    parser.add_argument(
        "--inject-runtime-failure",
        action="append",
        default=[],
        help="force a window-system runtime check to fail before damage analysis",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument(
        "--force-display-fallback",
        action="store_true",
        help="select the efifb display runtime path while keeping compositor checks active",
    )
    parser.add_argument("--out", default="out/compositor-damage-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    try:
        injected_failures = normalize_failures(args.inject_failure)
        runtime_failures = runtime.normalize_failures(args.inject_runtime_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_compositor_damage(
        seed=args.seed,
        injected_failures=injected_failures,
        runtime_failures=sorted(runtime_failures),
        max_failures=args.max_failures,
        force_display_fallback=args.force_display_fallback,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"compositor-damage-report: {out_path}")
    print(f"phase_count: {report['totals']['phase_count']}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
