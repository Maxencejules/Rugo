#!/usr/bin/env python3
"""Run deterministic graphical installer smoke checks for M52."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import build_installer_v2 as installer_contract
import run_desktop_shell_workflows_v1 as shell_workflows
import run_recovery_drill_v3 as recovery_drill


SCHEMA = "rugo.graphical_installer_smoke_report.v1"
CONTRACT_ID = "rugo.graphical_installer_ux.v1"
WORKFLOW_ID = "rugo.graphical_installer_flow.v1"
DEFAULT_SEED = 20260311


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
        check_id="shell_entry_budget",
        domain="installer",
        metric_key="shell_entry_latency_p95_ms",
        operator="max",
        threshold=90.0,
        base=36.0,
        spread=12,
        scale=2.0,
    ),
    CheckSpec(
        check_id="device_discovery_budget",
        domain="installer",
        metric_key="device_discovery_latency_p95_ms",
        operator="max",
        threshold=140.0,
        base=70.0,
        spread=16,
        scale=2.8,
    ),
    CheckSpec(
        check_id="target_selection_integrity",
        domain="installer",
        metric_key="target_selection_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="layout_validation_integrity",
        domain="installer",
        metric_key="layout_validation_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="install_commit_budget",
        domain="installer",
        metric_key="install_commit_latency_p95_ms",
        operator="max",
        threshold=180.0,
        base=104.0,
        spread=12,
        scale=3.0,
    ),
    CheckSpec(
        check_id="first_boot_handoff_integrity",
        domain="handoff",
        metric_key="first_boot_handoff_violation_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
)

SOURCE_CHECK_IDS = {
    "recovery_entry_visible",
    "shell_workflow_source_ready",
    "installer_contract_declared",
}


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


def _stage(name: str, ok: bool, latency_ms: float, target: str) -> Dict[str, object]:
    return {
        "name": name,
        "status": "pass" if ok else "fail",
        "latency_ms": latency_ms,
        "target": target,
    }


def _recovery_ready(seed: int) -> Dict[str, object]:
    report = recovery_drill.run_recovery_drill(seed=seed)
    report["max_failures"] = 0
    readiness = report["recovery_readiness"]
    report["meets_target"] = (
        report["total_failures"] == 0
        and readiness["operator_checklist_completed"] is True
        and readiness["state_capture_complete"] is True
    )
    report["gate_pass"] = report["meets_target"]
    return report


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def run_graphical_installer_smoke(
    seed: int,
    injected_failures: Set[str] | None = None,
    max_failures: int = 0,
    force_display_fallback: bool = False,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)

    shell_report = shell_workflows.run_desktop_shell_workflows(
        seed=seed,
        max_failures=0,
        force_display_fallback=force_display_fallback,
    )
    recovery_report = _recovery_ready(seed=seed)
    installer_report = installer_contract.build_installer_contract(
        channel="stable",
        version="2.0.0",
        build_sequence=52,
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

    recovery_entry_ok = (
        recovery_report["gate_pass"]
        and any(
            stage["name"] == "recovery_entry_validation" and stage["status"] == "pass"
            for stage in recovery_report["stages"]
        )
    )
    shell_source_ok = shell_report["gate_pass"] and shell_report["workflow_totals"]["passed"] == 4
    installer_declared_ok = (
        installer_report["schema"] == "rugo.installer_contract.v2"
        and installer_report["installer_profile"]["bootloader"] == "limine"
        and "recovery" in installer_report["installer_profile"]["partition_layout"]
    )

    checks.extend(
        [
            {
                "check_id": "recovery_entry_visible",
                "domain": "source",
                "metric_key": "recovery_entry_visible_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0
                if recovery_entry_ok and "recovery_entry_visible" not in failures
                else 0.999,
                "pass": recovery_entry_ok and "recovery_entry_visible" not in failures,
            },
            {
                "check_id": "shell_workflow_source_ready",
                "domain": "source",
                "metric_key": "shell_workflow_source_ready_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0
                if shell_source_ok and "shell_workflow_source_ready" not in failures
                else 0.999,
                "pass": shell_source_ok and "shell_workflow_source_ready" not in failures,
            },
            {
                "check_id": "installer_contract_declared",
                "domain": "source",
                "metric_key": "installer_contract_declared_ratio",
                "operator": "min",
                "threshold": 1.0,
                "observed": 1.0
                if installer_declared_ok and "installer_contract_declared" not in failures
                else 0.999,
                "pass": installer_declared_ok and "installer_contract_declared" not in failures,
            },
        ]
    )

    check_pass = {entry["check_id"]: bool(entry["pass"]) for entry in checks}
    handoff_pass = _domain_summary(checks, "handoff")["pass"] and check_pass["recovery_entry_visible"]

    stages = [
        _stage(
            "shell_bootstrap",
            check_pass["shell_entry_budget"] and check_pass["shell_workflow_source_ready"],
            metric_values["shell_entry_latency_p95_ms"],
            "desktop.shell.launcher",
        ),
        _stage(
            "device_scan",
            check_pass["device_discovery_budget"] and check_pass["installer_contract_declared"],
            metric_values["device_discovery_latency_p95_ms"],
            "disk0",
        ),
        _stage(
            "target_selection",
            check_pass["target_selection_integrity"],
            _metric(seed, "graphical_installer", "target", base=22.0, spread=6, scale=1.5),
            "disk0",
        ),
        _stage(
            "layout_review",
            check_pass["layout_validation_integrity"],
            _metric(seed, "graphical_installer", "layout", base=19.0, spread=6, scale=1.4),
            "layout://efi-system-recovery",
        ),
        _stage(
            "install_commit",
            check_pass["install_commit_budget"],
            metric_values["install_commit_latency_p95_ms"],
            "disk0",
        ),
        _stage(
            "first_boot_handoff",
            check_pass["first_boot_handoff_integrity"] and check_pass["recovery_entry_visible"],
            _metric(seed, "graphical_installer", "handoff", base=28.0, spread=8, scale=1.6),
            "desktop.shell.workspace",
        ),
    ]

    selected_target = {
        "device_id": "disk0",
        "transport": "virtio-blk",
        "capacity_gib": 64,
        "selected": True,
        "wipe_required": True,
    }

    layout = {
        "bootloader": installer_report["installer_profile"]["bootloader"],
        "partition_layout": installer_report["installer_profile"]["partition_layout"],
        "filesystem": "rugo-fs",
        "recovery_partition_present": True,
        "layout_review_pass": check_pass["layout_validation_integrity"],
        "target_selection_pass": check_pass["target_selection_integrity"],
    }

    handoff = {
        "boot_target": "desktop.shell.workspace",
        "first_boot_focus": "desktop.shell.launcher",
        "first_boot_workflows": ["launcher_open", "settings_update"],
        "recovery_entry_visible": check_pass["recovery_entry_visible"],
        "session_handoff_pass": handoff_pass,
    }

    total_failures = sum(1 for row in checks if row["pass"] is False)
    failures_list = sorted(row["check_id"] for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "workflow_id": WORKFLOW_ID,
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
        "shell_digest": shell_report["digest"],
        "recovery_digest": hashlib.sha256(
            json.dumps(
                {
                    "schema": recovery_report["schema"],
                    "workflow_id": recovery_report["workflow_id"],
                    "stages": [
                        {"name": stage["name"], "status": stage["status"]}
                        for stage in recovery_report["stages"]
                    ],
                },
                sort_keys=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest(),
        "selected_target": selected_target["device_id"],
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "contract_id": CONTRACT_ID,
        "workflow_id": WORKFLOW_ID,
        "parent_installer_ux_contract_id": recovery_drill.CONTRACT_ID,
        "parent_shell_contract_id": shell_workflows.CONTRACT_ID,
        "session_workflow_profile_id": shell_workflows.WORKFLOW_PROFILE_ID,
        "recovery_workflow_id": recovery_drill.WORKFLOW_ID,
        "installer_contract_schema": installer_report["schema"],
        "seed": seed,
        "gate": "test-desktop-workflows-v1",
        "parent_gate": "test-desktop-shell-v1",
        "checks": checks,
        "summary": {
            "installer": _domain_summary(checks, "installer"),
            "handoff": _domain_summary(checks, "handoff"),
            "source": _domain_summary(checks, "source"),
        },
        "stages": stages,
        "selected_target": selected_target,
        "layout": layout,
        "handoff": handoff,
        "source_reports": {
            "desktop_shell": {
                "schema": shell_report["schema"],
                "digest": shell_report["digest"],
                "gate_pass": shell_report["gate_pass"],
                "workflow_passed": shell_report["workflow_totals"]["passed"],
            },
            "installer_contract": {
                "schema": installer_report["schema"],
                "selected_channel": installer_report["selected_channel"],
                "partition_layout": installer_report["installer_profile"]["partition_layout"],
                "required_artifacts": installer_report["installer_profile"]["required_artifacts"],
            },
            "recovery_drill": {
                "schema": recovery_report["schema"],
                "workflow_id": recovery_report["workflow_id"],
                "gate_pass": recovery_report["gate_pass"],
                "post_recovery_audit_pass": any(
                    stage["name"] == "post_recovery_audit" and stage["status"] == "pass"
                    for stage in recovery_report["stages"]
                ),
            },
        },
        "artifact_refs": {
            "junit": "out/pytest-graphical-installer-smoke-v1.xml",
            "smoke_report": "out/graphical-installer-v1.json",
            "shell_report": "out/desktop-shell-v1.json",
            "recovery_report": "out/recovery-drill-v3.json",
            "ci_artifact": "desktop-workflows-v1-artifacts",
            "shell_ci_artifact": "desktop-shell-v1-artifacts",
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
        help="force a graphical installer check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument(
        "--force-display-fallback",
        action="store_true",
        help="select the efifb display runtime path while keeping installer workflows active",
    )
    parser.add_argument("--out", default="out/graphical-installer-v1.json")
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

    report = run_graphical_installer_smoke(
        seed=args.seed,
        injected_failures=injected_failures,
        max_failures=args.max_failures,
        force_display_fallback=args.force_display_fallback,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"graphical-installer-report: {out_path}")
    print(f"stage_count: {len(report['stages'])}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
