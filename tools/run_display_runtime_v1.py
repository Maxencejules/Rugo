#!/usr/bin/env python3
"""Run deterministic display runtime + scanout checks for M48."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_baremetal_io_baseline_v1 as baremetal
import run_hw_matrix_v6 as matrix


SCHEMA = "rugo.display_runtime_report.v1"
CONTRACT_ID = "rugo.display_runtime_contract.v1"
BUFFER_CONTRACT_ID = "rugo.scanout_buffer_contract.v1"
FALLBACK_POLICY_ID = "rugo.gpu_fallback_policy.v1"
FRAME_CAPTURE_SCHEMA = "rugo.display_frame_capture.v1"
DISPLAY_STACK_CONTRACT_ID = "rugo.display_stack_contract.v1"
DEFAULT_SEED = 20260311
PRIMARY_DISPLAY_CLASS = "virtio-gpu-pci"
PRIMARY_DISPLAY_DRIVER = "virtio_gpu_scanout"
PRIMARY_SOURCE_DRIVER = "virtio_gpu_framebuffer"
FALLBACK_DISPLAY_CLASS = "framebuffer-console"
FALLBACK_DISPLAY_DRIVER = "efifb"


@dataclass(frozen=True)
class CheckSpec:
    check_id: str
    domain: str
    metric_key: str
    operator: str  # one of: min, max, eq
    threshold: float
    base: float
    spread: int
    scale: float


BASE_CHECKS: Sequence[CheckSpec] = (
    CheckSpec(
        check_id="virtio_gpu_scanout",
        domain="scanout",
        metric_key="virtio_runtime_errors",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="virtio_present_cadence",
        domain="scanout",
        metric_key="virtio_frame_drop_ratio",
        operator="max",
        threshold=0.005,
        base=0.002,
        spread=4,
        scale=0.001,
    ),
    CheckSpec(
        check_id="buffer_ownership_integrity",
        domain="buffers",
        metric_key="buffer_partial_write_ratio",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="scanout_buffer_depth",
        domain="buffers",
        metric_key="scanout_buffer_depth",
        operator="min",
        threshold=3.0,
        base=3.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="present_timing_budget",
        domain="timing",
        metric_key="present_latency_p95_ms",
        operator="max",
        threshold=16.667,
        base=12.85,
        spread=10,
        scale=0.19,
    ),
    CheckSpec(
        check_id="present_jitter_budget",
        domain="timing",
        metric_key="vblank_jitter_p95_ms",
        operator="max",
        threshold=1.5,
        base=0.68,
        spread=8,
        scale=0.08,
    ),
    CheckSpec(
        check_id="frame_capture_ready",
        domain="capture",
        metric_key="capture_export_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="efifb_fallback_activation",
        domain="fallback",
        metric_key="fallback_activation_ms",
        operator="max",
        threshold=80.0,
        base=37.0,
        spread=20,
        scale=1.7,
    ),
    CheckSpec(
        check_id="efifb_fallback_scanout",
        domain="fallback",
        metric_key="fallback_frame_drop_ratio",
        operator="max",
        threshold=0.01,
        base=0.004,
        spread=4,
        scale=0.001,
    ),
)


def known_checks() -> Set[str]:
    return {spec.check_id for spec in BASE_CHECKS}


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
    delta = 0.001 if scale < 1.0 else 1.0
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


def _coverage_entry(
    coverage: Sequence[Dict[str, object]],
    device: str,
    profile: str | None = None,
) -> Dict[str, object]:
    for row in coverage:
        if row["device"] != device:
            continue
        if profile is not None and row.get("profile") != profile:
            continue
        return row
    raise ValueError(f"missing coverage row for {device!r}")


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def run_display_runtime(
    seed: int,
    injected_failures: Set[str] | None = None,
    max_failures: int = 0,
    force_fallback: bool = False,
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

    matrix_report = matrix.run_matrix(seed=seed, max_failures=0)
    baremetal_report = baremetal.run_baseline(seed=seed, max_failures=0)

    primary_source_green = (
        matrix_report["gate_pass"]
        and matrix_report["desktop_display_checks"]["bridge_pass"]
        and _coverage_entry(
            matrix_report["device_class_coverage"],
            PRIMARY_DISPLAY_CLASS,
            "modern",
        )["status"]
        == "pass"
    )
    fallback_source_green = (
        baremetal_report["gate_pass"]
        and baremetal_report["display_class"] == FALLBACK_DISPLAY_CLASS
        and baremetal_report["display_driver"] == FALLBACK_DISPLAY_DRIVER
    )

    checks.extend(
        [
            {
                "check_id": "virtio_gpu_declared_support",
                "domain": "source",
                "metric_key": "virtio_gpu_declared_support_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0 if primary_source_green else 0.999,
                "pass": primary_source_green,
            },
            {
                "check_id": "efifb_declared_support",
                "domain": "source",
                "metric_key": "efifb_declared_support_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0 if fallback_source_green else 0.999,
                "pass": fallback_source_green,
            },
        ]
    )

    check_pass = {entry["check_id"]: bool(entry["pass"]) for entry in checks}

    primary_checks_pass = (
        primary_source_green
        and check_pass["virtio_gpu_scanout"]
        and check_pass["virtio_present_cadence"]
        and check_pass["buffer_ownership_integrity"]
        and check_pass["scanout_buffer_depth"]
        and check_pass["present_timing_budget"]
        and check_pass["present_jitter_budget"]
        and check_pass["frame_capture_ready"]
    )
    fallback_checks_pass = (
        fallback_source_green
        and check_pass["efifb_fallback_activation"]
        and check_pass["efifb_fallback_scanout"]
        and check_pass["buffer_ownership_integrity"]
        and check_pass["scanout_buffer_depth"]
        and check_pass["present_timing_budget"]
        and check_pass["present_jitter_budget"]
        and check_pass["frame_capture_ready"]
    )

    if force_fallback:
        policy_decision = "forced_fallback"
        active_runtime_path = FALLBACK_DISPLAY_CLASS
        active_runtime_driver = FALLBACK_DISPLAY_DRIVER
    elif primary_checks_pass:
        policy_decision = "primary"
        active_runtime_path = PRIMARY_DISPLAY_CLASS
        active_runtime_driver = PRIMARY_DISPLAY_DRIVER
    else:
        policy_decision = "auto_fallback"
        active_runtime_path = FALLBACK_DISPLAY_CLASS
        active_runtime_driver = FALLBACK_DISPLAY_DRIVER

    frames_presented = 360 + (_noise(seed, "frames_presented") % 24)
    primary_frames_dropped = 0 if check_pass["virtio_present_cadence"] else 1
    fallback_frames_dropped = 0 if check_pass["efifb_fallback_scanout"] else 1

    total_failures = sum(1 for row in checks if row["pass"] is False)
    failures_list = sorted(row["check_id"] for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "seed": seed,
        "force_fallback": force_fallback,
        "matrix_digest": matrix_report["digest"],
        "baremetal_digest": baremetal_report["digest"],
        "checks": [
            {
                "check_id": row["check_id"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "active_runtime_path": active_runtime_path,
        "policy_decision": policy_decision,
        "injected_failures": sorted(failures),
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "contract_id": CONTRACT_ID,
        "buffer_contract_id": BUFFER_CONTRACT_ID,
        "fallback_policy_id": FALLBACK_POLICY_ID,
        "display_stack_contract_id": DISPLAY_STACK_CONTRACT_ID,
        "frame_capture_schema": FRAME_CAPTURE_SCHEMA,
        "seed": seed,
        "gate": "test-display-runtime-v1",
        "scanout_gate": "test-scanout-path-v1",
        "checks": checks,
        "summary": {
            "scanout": _domain_summary(checks, "scanout"),
            "buffers": _domain_summary(checks, "buffers"),
            "timing": _domain_summary(checks, "timing"),
            "capture": _domain_summary(checks, "capture"),
            "fallback": _domain_summary(checks, "fallback"),
            "source": _domain_summary(checks, "source"),
        },
        "primary_runtime": {
            "display_class": PRIMARY_DISPLAY_CLASS,
            "driver": PRIMARY_DISPLAY_DRIVER,
            "declared_support_source_schema": matrix_report["schema"],
            "declared_support_source_digest": matrix_report["digest"],
            "declared_support_driver": PRIMARY_SOURCE_DRIVER,
            "declared_support_pass": primary_source_green,
            "resolution": {"width": 1280, "height": 720},
            "refresh_hz": 60,
            "present_mode": "vblank-locked",
            "frame_drop_ratio": metric_values["virtio_frame_drop_ratio"],
            "frames_presented": frames_presented,
            "frames_dropped": primary_frames_dropped,
            "checks_pass": primary_checks_pass,
        },
        "fallback_runtime": {
            "display_class": FALLBACK_DISPLAY_CLASS,
            "driver": FALLBACK_DISPLAY_DRIVER,
            "declared_support_source_schema": baremetal_report["schema"],
            "declared_support_source_digest": baremetal_report["digest"],
            "declared_support_pass": fallback_source_green,
            "activation_latency_ms": metric_values["fallback_activation_ms"],
            "frame_drop_ratio": metric_values["fallback_frame_drop_ratio"],
            "frames_presented": frames_presented,
            "frames_dropped": fallback_frames_dropped,
            "checks_pass": fallback_checks_pass,
        },
        "active_runtime_path": active_runtime_path,
        "active_runtime_driver": active_runtime_driver,
        "policy_decision": policy_decision,
        "fallback_ready": fallback_checks_pass,
        "buffer_pool": {
            "contract_id": BUFFER_CONTRACT_ID,
            "buffer_strategy": "triple-buffer-plus-capture-shadow",
            "pixel_format": "xrgb8888",
            "width": 1280,
            "height": 720,
            "stride_bytes": 5120,
            "buffer_bytes": 3686400,
            "scanout_buffer_count": 3,
            "capture_shadow_count": 1,
            "total_buffers": 4,
            "ownership_states": [
                "runtime_owned",
                "scanout_pending",
                "display_owned",
                "capture_read_only",
            ],
            "state_counts": {
                "runtime_owned": 1,
                "scanout_pending": 1,
                "display_owned": 1,
                "capture_read_only": 1,
            },
            "integrity_pass": check_pass["buffer_ownership_integrity"]
            and check_pass["scanout_buffer_depth"],
        },
        "present_timing": {
            "target_refresh_hz": 60,
            "frame_budget_ms": 16.667,
            "present_latency_p95_ms": metric_values["present_latency_p95_ms"],
            "vblank_jitter_p95_ms": metric_values["vblank_jitter_p95_ms"],
            "presented_frames": frames_presented,
            "timing_checks_pass": check_pass["present_timing_budget"]
            and check_pass["present_jitter_budget"],
        },
        "capture": {
            "schema": FRAME_CAPTURE_SCHEMA,
            "capture_tool": "tools/capture_display_frame_v1.py",
            "frame_format": "png",
            "default_frame_path": "out/display-frame-v1.png",
            "default_manifest_path": "out/display-frame-v1.json",
            "capture_export_ratio": metric_values["capture_export_ratio"],
            "checks_pass": check_pass["frame_capture_ready"],
        },
        "source_reports": {
            "virtio_platform": {
                "schema": matrix_report["schema"],
                "digest": matrix_report["digest"],
                "gate_pass": matrix_report["gate_pass"],
                "display_class": matrix_report["display_class"],
                "desktop_bridge_green": matrix_report["desktop_display_checks"][
                    "bridge_pass"
                ],
            },
            "baremetal_fallback": {
                "schema": baremetal_report["schema"],
                "digest": baremetal_report["digest"],
                "gate_pass": baremetal_report["gate_pass"],
                "display_class": baremetal_report["display_class"],
                "display_driver": baremetal_report["display_driver"],
            },
        },
        "artifact_refs": {
            "junit": "out/pytest-display-runtime-v1.xml",
            "runtime_report": "out/display-runtime-v1.json",
            "frame_capture_png": "out/display-frame-v1.png",
            "frame_capture_manifest": "out/display-frame-v1.json",
            "virtio_matrix_report": "out/hw-matrix-v6.json",
            "baremetal_fallback_report": "out/baremetal-io-v1.json",
            "ci_artifact": "display-runtime-v1-artifacts",
            "scanout_ci_artifact": "scanout-path-v1-artifacts",
        },
        "force_fallback": force_fallback,
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
        help="force a display runtime check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument(
        "--force-fallback",
        action="store_true",
        help="select the efifb fallback path while keeping runtime checks active",
    )
    parser.add_argument("--out", default="out/display-runtime-v1.json")
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

    report = run_display_runtime(
        seed=args.seed,
        injected_failures=injected_failures,
        max_failures=args.max_failures,
        force_fallback=args.force_fallback,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"display-runtime-report: {out_path}")
    print(f"active_runtime_path: {report['active_runtime_path']}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
