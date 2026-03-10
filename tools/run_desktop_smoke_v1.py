#!/usr/bin/env python3
"""Run deterministic desktop/session/input smoke checks for M35."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set


SCHEMA = "rugo.desktop_smoke_report.v1"
POLICY_ID = "rugo.desktop_profile.v1"
DISPLAY_CONTRACT_ID = "rugo.display_stack_contract.v1"
WINDOW_CONTRACT_ID = "rugo.window_manager_contract.v1"
INPUT_CONTRACT_ID = "rugo.input_stack_contract.v1"
DEFAULT_SEED = 20260309


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


CHECKS: Sequence[CheckSpec] = (
    CheckSpec(
        check_id="display_mode_set",
        domain="display",
        metric_key="mode_set_ms",
        operator="max",
        threshold=220.0,
        base=180.0,
        spread=20,
        scale=1.0,
    ),
    CheckSpec(
        check_id="display_scanout_stable",
        domain="display",
        metric_key="frame_drop_ratio",
        operator="max",
        threshold=0.01,
        base=0.003,
        spread=5,
        scale=0.001,
    ),
    CheckSpec(
        check_id="session_handshake_ready",
        domain="session",
        metric_key="session_handshake_ms",
        operator="max",
        threshold=120.0,
        base=86.0,
        spread=18,
        scale=1.0,
    ),
    CheckSpec(
        check_id="session_desktop_ready",
        domain="session",
        metric_key="desktop_ready_ms",
        operator="max",
        threshold=350.0,
        base=240.0,
        spread=60,
        scale=1.0,
    ),
    CheckSpec(
        check_id="input_keyboard_latency",
        domain="input",
        metric_key="keyboard_latency_p95_ms",
        operator="max",
        threshold=12.0,
        base=8.0,
        spread=4,
        scale=1.0,
    ),
    CheckSpec(
        check_id="input_pointer_latency",
        domain="input",
        metric_key="pointer_latency_p95_ms",
        operator="max",
        threshold=14.0,
        base=9.0,
        spread=4,
        scale=1.0,
    ),
    CheckSpec(
        check_id="input_focus_delivery",
        domain="input",
        metric_key="input_delivery_ratio",
        operator="min",
        threshold=0.995,
        base=0.997,
        spread=3,
        scale=0.001,
    ),
    CheckSpec(
        check_id="input_repeat_consistency",
        domain="input",
        metric_key="dropped_events",
        operator="max",
        threshold=2.0,
        base=0.0,
        spread=2,
        scale=1.0,
    ),
    CheckSpec(
        check_id="window_create_ok",
        domain="window",
        metric_key="window_create_ms",
        operator="max",
        threshold=80.0,
        base=41.0,
        spread=18,
        scale=1.0,
    ),
    CheckSpec(
        check_id="window_map_ok",
        domain="window",
        metric_key="window_map_ms",
        operator="max",
        threshold=55.0,
        base=26.0,
        spread=12,
        scale=1.0,
    ),
    CheckSpec(
        check_id="window_focus_switch",
        domain="window",
        metric_key="focus_switch_ms",
        operator="max",
        threshold=40.0,
        base=18.0,
        spread=10,
        scale=1.0,
    ),
    CheckSpec(
        check_id="window_close_ok",
        domain="window",
        metric_key="window_close_ms",
        operator="max",
        threshold=35.0,
        base=12.0,
        spread=8,
        scale=1.0,
    ),
)


def _known_checks() -> Set[str]:
    return {spec.check_id for spec in CHECKS}


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _round_value(value: float) -> float:
    return round(value, 3) if isinstance(value, float) else value


def _baseline_observed(seed: int, spec: CheckSpec) -> float:
    value = spec.base + ((_noise(seed, spec.check_id) % spec.spread) * spec.scale)
    return _round_value(value)


def _failing_observed(spec: CheckSpec) -> float:
    delta = 0.001 if spec.scale < 1.0 or isinstance(spec.threshold, float) else 1.0
    if spec.operator == "max":
        return _round_value(spec.threshold + delta)
    if spec.operator == "min":
        return _round_value(spec.threshold - delta)
    return _round_value(spec.threshold + delta)


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


def _check_by_id(checks: List[Dict[str, object]], check_id: str) -> Dict[str, object]:
    matches = [entry for entry in checks if entry["check_id"] == check_id]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one check entry for {check_id}")
    return matches[0]


def _normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - _known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def normalize_failures(values: Sequence[str]) -> Set[str]:
    return _normalize_failures(values)


def run_smoke(
    seed: int,
    injected_failures: Set[str] | None = None,
    display_class: str = "virtio-gpu-pci",
    display_driver: str = "virtio_gpu_framebuffer",
    boot_transport_class: str = "virtio-blk-pci-modern",
    input_class: str = "integrated-input",
    input_driver: str = "desktop_input_core",
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)

    checks: List[Dict[str, object]] = []
    metric_values: Dict[str, float] = {}
    for spec in CHECKS:
        observed = (
            _failing_observed(spec)
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

    total_failures = sum(1 for check in checks if check["pass"] is False)
    display_summary = _domain_summary(checks, "display")
    session_summary = _domain_summary(checks, "session")
    input_summary = _domain_summary(checks, "input")
    window_summary = _domain_summary(checks, "window")
    bridge_pass = display_summary["pass"] and session_summary["pass"]
    focus_delivery_pass = bool(_check_by_id(checks, "input_focus_delivery")["pass"])
    stable_payload = {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "seed": seed,
        "display_class": display_class,
        "display_driver": display_driver,
        "boot_transport_class": boot_transport_class,
        "input_class": input_class,
        "input_driver": input_driver,
        "checks": [
            {
                "check_id": check["check_id"],
                "pass": check["pass"],
                "observed": check["observed"],
            }
            for check in checks
        ],
        "injected_failures": sorted(failures),
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "display_contract_id": DISPLAY_CONTRACT_ID,
        "window_contract_id": WINDOW_CONTRACT_ID,
        "input_contract_id": INPUT_CONTRACT_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "boot_transport_class": boot_transport_class,
        "display_class": display_class,
        "checks": checks,
        "summary": {
            "display": display_summary,
            "session": session_summary,
            "input": input_summary,
            "window": window_summary,
        },
        "display": {
            "device_class": display_class,
            "driver": display_driver,
            "scanout_model": "framebuffer",
            "mode_set_ms": metric_values["mode_set_ms"],
            "frame_drop_ratio": metric_values["frame_drop_ratio"],
            "checks_pass": display_summary["pass"],
        },
        "display_device": {
            "device_class": display_class,
            "driver": display_driver,
            "scanout_model": "framebuffer",
            "desktop_qualified": bridge_pass,
        },
        "desktop_display_checks": {
            "contract_id": DISPLAY_CONTRACT_ID,
            "display_class": display_class,
            "display_driver": display_driver,
            "boot_transport_class": boot_transport_class,
            "qualifying_checks": [
                "display_mode_set",
                "display_scanout_stable",
                "session_handshake_ready",
                "session_desktop_ready",
            ],
            "display_checks_pass": display_summary["pass"],
            "session_checks_pass": session_summary["pass"],
            "bridge_pass": bridge_pass,
        },
        "session": {
            "session_handshake_ms": metric_values["session_handshake_ms"],
            "desktop_ready_ms": metric_values["desktop_ready_ms"],
            "checks_pass": session_summary["pass"],
        },
        "input": {
            "device_class": input_class,
            "driver": input_driver,
            "keyboard_latency_p95_ms": metric_values["keyboard_latency_p95_ms"],
            "pointer_latency_p95_ms": metric_values["pointer_latency_p95_ms"],
            "input_delivery_ratio": metric_values["input_delivery_ratio"],
            "dropped_events": int(metric_values["dropped_events"]),
            "checks_pass": input_summary["pass"],
        },
        "input_class": input_class,
        "input_device": {
            "device_class": input_class,
            "driver": input_driver,
            "desktop_qualified": input_summary["pass"],
        },
        "desktop_input_checks": {
            "contract_id": INPUT_CONTRACT_ID,
            "input_class": input_class,
            "input_driver": input_driver,
            "qualifying_checks": [
                "input_keyboard_latency",
                "input_pointer_latency",
                "input_focus_delivery",
                "input_repeat_consistency",
            ],
            "input_checks_pass": input_summary["pass"],
            "focus_delivery_pass": focus_delivery_pass,
            "delivery_ratio": metric_values["input_delivery_ratio"],
        },
        "window_lifecycle": {
            "window_create_ms": metric_values["window_create_ms"],
            "window_map_ms": metric_values["window_map_ms"],
            "focus_switch_ms": metric_values["focus_switch_ms"],
            "window_close_ms": metric_values["window_close_ms"],
            "checks_pass": window_summary["pass"],
        },
        "injected_failures": sorted(failures),
        "total_failures": total_failures,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--display-class", default="virtio-gpu-pci")
    parser.add_argument("--display-driver", default="virtio_gpu_framebuffer")
    parser.add_argument("--boot-transport-class", default="virtio-blk-pci-modern")
    parser.add_argument("--input-class", default="integrated-input")
    parser.add_argument("--input-driver", default="desktop_input_core")
    parser.add_argument("--out", default="out/desktop-smoke-v1.json")
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

    report = run_smoke(
        seed=args.seed,
        injected_failures=injected_failures,
        display_class=args.display_class,
        display_driver=args.display_driver,
        boot_transport_class=args.boot_transport_class,
        input_class=args.input_class,
        input_driver=args.input_driver,
    )
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"desktop-smoke-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
