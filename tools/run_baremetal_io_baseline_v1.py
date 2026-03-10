#!/usr/bin/env python3
"""Run deterministic bare-metal I/O baseline checks for M46."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_desktop_smoke_v1 as desktop_smoke
import run_recovery_drill_v3 as recovery_drill


SCHEMA = "rugo.baremetal_io_baseline.v1"
PROFILE_ID = "rugo.baremetal_io_profile.v1"
DRIVER_CONTRACT_ID = "rugo.driver_lifecycle_report.v6"
USB_INPUT_REMOVABLE_CONTRACT_ID = "rugo.usb_input_removable_contract.v1"
INPUT_CONTRACT_ID = "rugo.input_stack_contract.v1"
RECOVERY_WORKFLOW_ID = "rugo.recovery_workflow.v3"
DEFAULT_SEED = 20260310
DEFAULT_DISPLAY_CLASS = "framebuffer-console"
DEFAULT_DISPLAY_DRIVER = "efifb"
DEFAULT_BOOT_TRANSPORT_CLASS = "ahci"
DEFAULT_INPUT_CLASS = "usb-hid"
DEFAULT_INPUT_DRIVER = "xhci-usb-hid"
DEFAULT_REMOVABLE_MEDIA_CLASS = "usb-storage"


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
        check_id="e1000e_udp_echo",
        domain="network",
        metric_key="e1000e_udp_echo_failures",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="e1000e_link_stable",
        domain="network",
        metric_key="e1000e_link_stable_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="rtl8169_udp_echo",
        domain="network",
        metric_key="rtl8169_udp_echo_failures",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="rtl8169_link_stable",
        domain="network",
        metric_key="rtl8169_link_stable_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="xhci_enumeration",
        domain="usb_input",
        metric_key="xhci_enumeration_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="usb_storage_enumeration",
        domain="removable",
        metric_key="usb_storage_enumeration_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="usb_storage_mount",
        domain="removable",
        metric_key="usb_storage_mount_latency_ms",
        operator="max",
        threshold=400.0,
        base=185.0,
        spread=25,
        scale=4.0,
    ),
    CheckSpec(
        check_id="negative_e1000e_missing_deterministic",
        domain="negative_path",
        metric_key="e1000e_missing_marker_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="negative_rtl8169_missing_deterministic",
        domain="negative_path",
        metric_key="rtl8169_missing_marker_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="negative_usb_hid_missing_deterministic",
        domain="negative_path",
        metric_key="usb_hid_missing_marker_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="negative_usb_storage_missing_deterministic",
        domain="negative_path",
        metric_key="usb_storage_missing_marker_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
)


def known_checks() -> Set[str]:
    return {spec.check_id for spec in BASE_CHECKS} | {
        "usb_keyboard_latency",
        "usb_pointer_latency",
        "usb_focus_delivery",
        "usb_repeat_consistency",
        "desktop_input_bridge",
        "recovery_media_bootstrap",
        "recovery_post_audit",
    }


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


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def _metric(seed: int, label: str, base: int, spread: int) -> int:
    return base + (_noise(seed, label) % spread)


def _lifecycle_row(
    driver: str,
    device_class: str,
    passed: bool,
    extra_states: Sequence[str] | None = None,
) -> Dict[str, object]:
    states = [
        "probe_found",
        "init_ready",
        "runtime_ok",
        "irq_vector_bound",
        "irq_vector_retarget",
        "cpu_affinity_balance",
        "reset_recover",
    ]
    if extra_states:
        states.extend(extra_states)
    if passed:
        return {
            "driver": driver,
            "device_class": device_class,
            "profile": "baremetal",
            "states_observed": states,
            "probe_attempts": 1,
            "probe_successes": 1,
            "init_failures": 0,
            "runtime_errors": 0,
            "irq_vector_bound": True,
            "irq_vector_retarget": 1,
            "affinity_balance_events": 1,
            "recoveries": 0,
            "fatal_errors": 0,
            "status": "pass",
        }

    return {
        "driver": driver,
        "device_class": device_class,
        "profile": "baremetal",
        "states_observed": ["probe_missing", "error_fatal"],
        "probe_attempts": 1,
        "probe_successes": 0,
        "init_failures": 1,
        "runtime_errors": 1,
        "irq_vector_bound": False,
        "irq_vector_retarget": 0,
        "affinity_balance_events": 0,
        "recoveries": 0,
        "fatal_errors": 1,
        "status": "fail",
    }


def run_baseline(
    seed: int,
    injected_failures: Set[str] | None = None,
    max_failures: int = 0,
    display_class: str = DEFAULT_DISPLAY_CLASS,
    display_driver: str = DEFAULT_DISPLAY_DRIVER,
    boot_transport_class: str = DEFAULT_BOOT_TRANSPORT_CLASS,
    input_class: str = DEFAULT_INPUT_CLASS,
    input_driver: str = DEFAULT_INPUT_DRIVER,
    removable_media_class: str = DEFAULT_REMOVABLE_MEDIA_CLASS,
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

    desktop_failures: Set[str] = set()
    desktop_failure_map = {
        "usb_keyboard_latency": "input_keyboard_latency",
        "usb_pointer_latency": "input_pointer_latency",
        "usb_focus_delivery": "input_focus_delivery",
        "usb_repeat_consistency": "input_repeat_consistency",
        "desktop_input_bridge": "input_focus_delivery",
    }
    for check_id, desktop_check in desktop_failure_map.items():
        if check_id in failures:
            desktop_failures.add(desktop_check)

    desktop_report = desktop_smoke.run_smoke(
        seed=seed,
        injected_failures=desktop_failures,
        display_class=display_class,
        display_driver=display_driver,
        boot_transport_class=boot_transport_class,
        input_class=input_class,
        input_driver=input_driver,
    )

    recovery_failures: Set[str] = set()
    if "recovery_media_bootstrap" in failures:
        recovery_failures.add("recovery_entry_validation")
    if "recovery_post_audit" in failures:
        recovery_failures.add("post_recovery_audit")

    recovery_report = recovery_drill.run_recovery_drill(
        seed=seed,
        forced_failures=recovery_failures,
        operator_checklist_completed=True,
    )
    recovery_report["max_failures"] = 0
    recovery_report["meets_target"] = recovery_report["total_failures"] == 0
    recovery_report["gate_pass"] = recovery_report["meets_target"]

    external_checks = [
        {
            "check_id": "usb_keyboard_latency",
            "domain": "usb_input",
            "metric_key": "usb_keyboard_latency_p95_ms",
            "operator": "max",
            "threshold": 12.0,
            "observed": desktop_report["input"]["keyboard_latency_p95_ms"],
        },
        {
            "check_id": "usb_pointer_latency",
            "domain": "usb_input",
            "metric_key": "usb_pointer_latency_p95_ms",
            "operator": "max",
            "threshold": 14.0,
            "observed": desktop_report["input"]["pointer_latency_p95_ms"],
        },
        {
            "check_id": "usb_focus_delivery",
            "domain": "usb_input",
            "metric_key": "usb_input_delivery_ratio",
            "operator": "min",
            "threshold": 0.995,
            "observed": desktop_report["input"]["input_delivery_ratio"],
        },
        {
            "check_id": "usb_repeat_consistency",
            "domain": "usb_input",
            "metric_key": "usb_dropped_events",
            "operator": "max",
            "threshold": 2.0,
            "observed": float(desktop_report["input"]["dropped_events"]),
        },
        {
            "check_id": "desktop_input_bridge",
            "domain": "usb_input",
            "metric_key": "desktop_input_bridge_ratio",
            "operator": "min",
            "threshold": 1.0,
            "observed": (
                1.0
                if desktop_report["desktop_input_checks"]["input_checks_pass"]
                else 0.999
            ),
        },
        {
            "check_id": "recovery_media_bootstrap",
            "domain": "recovery",
            "metric_key": "recovery_media_bootstrap_ratio",
            "operator": "min",
            "threshold": 1.0,
            "observed": 1.0 if recovery_report["gate_pass"] else 0.999,
        },
        {
            "check_id": "recovery_post_audit",
            "domain": "recovery",
            "metric_key": "recovery_post_audit_ratio",
            "operator": "min",
            "threshold": 1.0,
            "observed": (
                1.0
                if any(
                    stage["name"] == "post_recovery_audit" and stage["status"] == "pass"
                    for stage in recovery_report["stages"]
                )
                else 0.999
            ),
        },
    ]
    for row in external_checks:
        row["pass"] = _passes(row["operator"], row["observed"], row["threshold"])
        checks.append(row)

    check_pass = {row["check_id"]: bool(row["pass"]) for row in checks}

    network_summary = _domain_summary(checks, "network")
    usb_input_summary = _domain_summary(checks, "usb_input")
    removable_summary = _domain_summary(checks, "removable")
    recovery_summary = _domain_summary(checks, "recovery")
    negative_summary = _domain_summary(checks, "negative_path")

    e1000e_pass = check_pass["e1000e_udp_echo"] and check_pass["e1000e_link_stable"]
    rtl8169_pass = check_pass["rtl8169_udp_echo"] and check_pass["rtl8169_link_stable"]
    usb_input_pass = usb_input_summary["pass"] and check_pass["xhci_enumeration"]
    removable_pass = (
        removable_summary["pass"]
        and recovery_summary["pass"]
        and check_pass["usb_storage_enumeration"]
        and check_pass["usb_storage_mount"]
    )

    tier2_profiles = [
        {
            "profile_id": "intel_q470_e1000e_xhci",
            "tier": "tier2",
            "wired_nic": "e1000e",
            "input_class": input_class,
            "removable_media_class": removable_media_class,
            "manual_exception_required": False,
            "status": "pass" if e1000e_pass and usb_input_pass and removable_pass else "fail",
        },
        {
            "profile_id": "amd_b550_rtl8169_xhci",
            "tier": "tier2",
            "wired_nic": "rtl8169",
            "input_class": input_class,
            "removable_media_class": removable_media_class,
            "manual_exception_required": False,
            "status": "pass" if rtl8169_pass and usb_input_pass and removable_pass else "fail",
        },
    ]

    device_class_coverage = [
        {
            "device": "e1000e",
            "class": "network",
            "required": True,
            "status": "pass" if e1000e_pass else "fail",
        },
        {
            "device": "rtl8169",
            "class": "network",
            "required": True,
            "status": "pass" if rtl8169_pass else "fail",
        },
        {
            "device": "xhci",
            "class": "usb-host",
            "required": True,
            "status": "pass" if check_pass["xhci_enumeration"] else "fail",
        },
        {
            "device": "usb-hid",
            "class": "input",
            "required": True,
            "desktop_bound": True,
            "status": "pass" if usb_input_pass else "fail",
        },
        {
            "device": removable_media_class,
            "class": "removable",
            "required": True,
            "recovery_bound": True,
            "status": "pass" if removable_pass else "fail",
        },
    ]

    driver_lifecycle = [
        _lifecycle_row("e1000e", "network", e1000e_pass, extra_states=["link_ready"]),
        _lifecycle_row("rtl8169", "network", rtl8169_pass, extra_states=["link_ready"]),
        _lifecycle_row(
            "xhci",
            "usb-host",
            check_pass["xhci_enumeration"],
            extra_states=["hid_ready"],
        ),
        _lifecycle_row(
            "usb-hid",
            "input",
            usb_input_pass,
            extra_states=["hid_ready", "focus_delivery_ready"],
        ),
        _lifecycle_row(
            removable_media_class,
            "removable",
            removable_pass,
            extra_states=["media_ready", "recovery_media_bootstrap"],
        ),
    ]

    wired_nic = {
        "e1000e": {
            "driver": "e1000e",
            "probe_latency_ms": _metric(seed, "e1000e_probe_latency_ms", 14, 12),
            "udp_echo_pass": check_pass["e1000e_udp_echo"],
            "link_ready": check_pass["e1000e_link_stable"],
            "irq_retarget_events": 1 if e1000e_pass else 0,
            "status": "pass" if e1000e_pass else "fail",
        },
        "rtl8169": {
            "driver": "rtl8169",
            "probe_latency_ms": _metric(seed, "rtl8169_probe_latency_ms", 16, 12),
            "udp_echo_pass": check_pass["rtl8169_udp_echo"],
            "link_ready": check_pass["rtl8169_link_stable"],
            "irq_retarget_events": 1 if rtl8169_pass else 0,
            "status": "pass" if rtl8169_pass else "fail",
        },
    }

    usb_input = {
        "controller": "xhci",
        "input_class": input_class,
        "driver": input_driver,
        "xhci_enumeration_pass": check_pass["xhci_enumeration"],
        "keyboard_latency_p95_ms": desktop_report["input"]["keyboard_latency_p95_ms"],
        "pointer_latency_p95_ms": desktop_report["input"]["pointer_latency_p95_ms"],
        "input_delivery_ratio": desktop_report["input"]["input_delivery_ratio"],
        "dropped_events": desktop_report["input"]["dropped_events"],
        "focus_delivery_pass": desktop_report["desktop_input_checks"]["focus_delivery_pass"],
        "checks_pass": usb_input_pass,
    }

    install_recovery_checks = {
        "workflow_id": recovery_report["workflow_id"],
        "source_schema": recovery_report["schema"],
        "removable_media_class": removable_media_class,
        "qualifying_stages": [
            "recovery_entry_validation",
            "post_recovery_audit",
        ],
        "recovery_entry_validation_pass": any(
            stage["name"] == "recovery_entry_validation" and stage["status"] == "pass"
            for stage in recovery_report["stages"]
        ),
        "post_recovery_audit_pass": any(
            stage["name"] == "post_recovery_audit" and stage["status"] == "pass"
            for stage in recovery_report["stages"]
        ),
        "recovery_gate_pass": recovery_report["gate_pass"],
    }

    removable_media = {
        "device_class": removable_media_class,
        "driver": removable_media_class,
        "enumeration_pass": check_pass["usb_storage_enumeration"],
        "mount_latency_ms": metric_values["usb_storage_mount_latency_ms"],
        "mount_pass": check_pass["usb_storage_mount"],
        "recovery_media_bootstrap_pass": check_pass["recovery_media_bootstrap"],
        "post_recovery_audit_pass": check_pass["recovery_post_audit"],
        "checks_pass": removable_pass,
    }

    negative_paths = {
        "e1000e_probe_missing": {
            "marker": "NET: e1000e not found",
            "deterministic": check_pass["negative_e1000e_missing_deterministic"],
            "status": (
                "pass" if check_pass["negative_e1000e_missing_deterministic"] else "fail"
            ),
        },
        "rtl8169_probe_missing": {
            "marker": "NET: rtl8169 not found",
            "deterministic": check_pass["negative_rtl8169_missing_deterministic"],
            "status": (
                "pass" if check_pass["negative_rtl8169_missing_deterministic"] else "fail"
            ),
        },
        "usb_hid_missing": {
            "marker": "USB: hid not found",
            "deterministic": check_pass["negative_usb_hid_missing_deterministic"],
            "status": (
                "pass" if check_pass["negative_usb_hid_missing_deterministic"] else "fail"
            ),
        },
        "usb_storage_missing": {
            "marker": "USBSTOR: not found",
            "deterministic": check_pass["negative_usb_storage_missing_deterministic"],
            "status": (
                "pass"
                if check_pass["negative_usb_storage_missing_deterministic"]
                else "fail"
            ),
        },
    }

    total_failures = sum(1 for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "profile_id": PROFILE_ID,
        "seed": seed,
        "display_class": display_class,
        "boot_transport_class": boot_transport_class,
        "input_class": input_class,
        "removable_media_class": removable_media_class,
        "desktop_smoke_digest": desktop_report["digest"],
        "recovery_failures": sorted(recovery_failures),
        "checks": [
            {
                "check_id": row["check_id"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "injected_failures": sorted(failures),
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "profile_id": PROFILE_ID,
        "driver_contract_id": DRIVER_CONTRACT_ID,
        "usb_input_removable_contract_id": USB_INPUT_REMOVABLE_CONTRACT_ID,
        "input_contract_id": INPUT_CONTRACT_ID,
        "recovery_workflow_id": RECOVERY_WORKFLOW_ID,
        "seed": seed,
        "gate": "test-baremetal-io-baseline-v1",
        "checks": checks,
        "summary": {
            "network": network_summary,
            "usb_input": usb_input_summary,
            "removable": removable_summary,
            "recovery": recovery_summary,
            "negative_path": negative_summary,
        },
        "tier2_profiles": tier2_profiles,
        "device_class_coverage": device_class_coverage,
        "driver_lifecycle": driver_lifecycle,
        "wired_nic": wired_nic,
        "usb_input": usb_input,
        "removable_media": removable_media,
        "display_class": display_class,
        "display_driver": display_driver,
        "boot_transport_class": boot_transport_class,
        "input_class": input_class,
        "input_driver": input_driver,
        "removable_media_class": removable_media_class,
        "desktop_input_checks": {
            **desktop_report["desktop_input_checks"],
            "source_schema": desktop_report["schema"],
            "source_digest": desktop_report["digest"],
        },
        "install_recovery_checks": install_recovery_checks,
        "negative_paths": negative_paths,
        "artifact_refs": {
            "junit": "out/pytest-baremetal-io-v1.xml",
            "baseline_report": "out/baremetal-io-v1.json",
            "desktop_smoke_report": "out/desktop-smoke-v1.json",
            "recovery_report": "out/recovery-drill-v3.json",
            "promotion_report": "out/hw-promotion-v2.json",
            "ci_artifact": "baremetal-io-v1-artifacts",
            "usb_ci_artifact": "usb-input-removable-v1-artifacts",
        },
        "injected_failures": sorted(failures),
        "max_failures": max_failures,
        "total_failures": total_failures,
        "failures": sorted(row["check_id"] for row in checks if row["pass"] is False),
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
        help="force a check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--display-class", default=DEFAULT_DISPLAY_CLASS)
    parser.add_argument("--display-driver", default=DEFAULT_DISPLAY_DRIVER)
    parser.add_argument("--boot-transport-class", default=DEFAULT_BOOT_TRANSPORT_CLASS)
    parser.add_argument("--input-class", default=DEFAULT_INPUT_CLASS)
    parser.add_argument("--input-driver", default=DEFAULT_INPUT_DRIVER)
    parser.add_argument("--removable-media-class", default=DEFAULT_REMOVABLE_MEDIA_CLASS)
    parser.add_argument("--out", default="out/baremetal-io-v1.json")
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

    report = run_baseline(
        seed=args.seed,
        injected_failures=injected_failures,
        max_failures=args.max_failures,
        display_class=args.display_class,
        display_driver=args.display_driver,
        boot_transport_class=args.boot_transport_class,
        input_class=args.input_class,
        input_driver=args.input_driver,
        removable_media_class=args.removable_media_class,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"baremetal-io-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
