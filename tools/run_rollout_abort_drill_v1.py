#!/usr/bin/env python3
"""Run deterministic SLO-triggered rollout abort/rollback drill for M33."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


SCHEMA = "rugo.rollout_abort_drill_report.v1"
POLICY_ID = "rugo.canary_slo_policy.v1"


def _report_summary(report: Dict[str, object]) -> Dict[str, object]:
    return {
        "schema": report.get("schema", ""),
        "runtime_capture_digest": report.get("runtime_capture_digest", ""),
        "gate_pass": report.get("gate_pass"),
    }


def _recovery_actions(
    canary_report: Dict[str, object] | None,
    fleet_health_report: Dict[str, object] | None,
    fleet_update_report: Dict[str, object] | None,
) -> List[Dict[str, object]]:
    actions: List[Dict[str, object]] = []
    if canary_report:
        failing_stages = [
            stage["stage"]
            for stage in canary_report.get("stages", [])
            if isinstance(stage, dict)
            and (stage.get("auto_halt") is True or stage.get("pass") is False)
        ]
        if failing_stages:
            actions.append(
                {
                    "action": "halt_rollout_stage",
                    "scope": "canary",
                    "targets": failing_stages,
                }
            )
    if fleet_health_report and fleet_health_report.get("fleet_state") == "degraded":
        actions.append(
            {
                "action": "quarantine_degraded_clusters",
                "scope": "fleet_health",
                "targets": [
                    cluster.get("cluster_id", "")
                    for cluster in fleet_health_report.get("clusters", [])
                    if isinstance(cluster, dict) and cluster.get("within_slo") is False
                ],
            }
        )
    if fleet_update_report:
        rollback_groups = [
            group.get("group_id", "")
            for group in fleet_update_report.get("groups", [])
            if isinstance(group, dict) and group.get("rollback_triggered") is True
        ]
        if rollback_groups:
            actions.append(
                {
                    "action": "rollback_group_versions",
                    "scope": "fleet_update",
                    "targets": rollback_groups,
                }
            )
    if not actions:
        actions.append(
            {
                "action": "rollback_canary_wave",
                "scope": "policy_drill",
                "targets": ["canary"],
            }
        )
    return actions


def run_drill(
    slo_error_rate_threshold: float,
    slo_latency_p95_ms_threshold: int,
    observed_error_rate: float,
    observed_latency_p95_ms: int,
    *,
    canary_report: Dict[str, object] | None = None,
    fleet_health_report: Dict[str, object] | None = None,
    fleet_update_report: Dict[str, object] | None = None,
) -> Dict[str, object]:
    error_breach = observed_error_rate > slo_error_rate_threshold
    latency_breach = observed_latency_p95_ms > slo_latency_p95_ms_threshold
    report_error_breach = False
    report_latency_breach = False
    if canary_report:
        report_error_breach = report_error_breach or bool(canary_report.get("halted"))
        for stage in canary_report.get("stages", []):
            if not isinstance(stage, dict):
                continue
            report_error_breach = report_error_breach or (
                float(stage.get("observed_error_rate", 0.0)) > slo_error_rate_threshold
            )
            report_latency_breach = report_latency_breach or (
                float(stage.get("observed_latency_p95_ms", 0.0))
                > slo_latency_p95_ms_threshold
            )
    if fleet_health_report:
        report_error_breach = report_error_breach or (
            float(fleet_health_report.get("fleet_error_rate", 0.0))
            > slo_error_rate_threshold
        )
        report_latency_breach = report_latency_breach or any(
            float(cluster.get("latency_p95_ms", 0.0)) > slo_latency_p95_ms_threshold
            for cluster in fleet_health_report.get("clusters", [])
            if isinstance(cluster, dict)
        )
    if fleet_update_report:
        report_error_breach = report_error_breach or any(
            float(group.get("success_rate", 1.0)) < float(group.get("min_success_rate", 1.0))
            for group in fleet_update_report.get("groups", [])
            if isinstance(group, dict)
        )
    error_breach = error_breach or report_error_breach
    latency_breach = latency_breach or report_latency_breach
    auto_halt = error_breach or latency_breach
    recovery_actions = _recovery_actions(
        canary_report=canary_report,
        fleet_health_report=fleet_health_report,
        fleet_update_report=fleet_update_report,
    )

    checks = [
        {
            "name": "slo_breach_detected",
            "pass": error_breach or latency_breach,
        },
        {
            "name": "auto_halt_triggered",
            "pass": auto_halt,
        },
        {
            "name": "rollback_triggered",
            "pass": auto_halt and len(recovery_actions) > 0,
        },
    ]
    total_failures = sum(1 for check in checks if check["pass"] is False)

    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "slo_error_rate_threshold": slo_error_rate_threshold,
        "slo_latency_p95_ms_threshold": slo_latency_p95_ms_threshold,
        "observed_error_rate": observed_error_rate,
        "observed_latency_p95_ms": observed_latency_p95_ms,
        "error_breach": error_breach,
        "latency_breach": latency_breach,
        "auto_halt": auto_halt,
        "rollback_triggered": auto_halt,
        "policy_enforced": auto_halt,
        "halt_reason": (
            "slo_breach_detected"
            if auto_halt
            else "no_breach_detected"
        ),
        "input_reports": {
            "canary_report": _report_summary(canary_report) if canary_report else {},
            "fleet_health_report": _report_summary(fleet_health_report)
            if fleet_health_report
            else {},
            "fleet_update_report": _report_summary(fleet_update_report)
            if fleet_update_report
            else {},
        },
        "recovery_actions": recovery_actions,
        "checks": checks,
        "total_failures": total_failures,
    }


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--slo-error-rate-threshold", type=float, default=0.02)
    p.add_argument("--slo-latency-p95-ms-threshold", type=int, default=200)
    p.add_argument("--observed-error-rate", type=float, default=0.08)
    p.add_argument("--observed-latency-p95-ms", type=int, default=245)
    p.add_argument("--canary-report", default="")
    p.add_argument("--fleet-health-report", default="")
    p.add_argument("--fleet-update-report", default="")
    p.add_argument("--max-failures", type=int, default=0)
    p.add_argument("--out", default="out/rollout-abort-drill-v1.json")
    return p


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2
    if not (0.0 <= args.slo_error_rate_threshold <= 1.0):
        print("error: slo-error-rate-threshold must be in [0, 1]")
        return 2
    if not (0.0 <= args.observed_error_rate <= 1.0):
        print("error: observed-error-rate must be in [0, 1]")
        return 2
    if args.slo_latency_p95_ms_threshold <= 0:
        print("error: slo-latency-p95-ms-threshold must be > 0")
        return 2
    if args.observed_latency_p95_ms <= 0:
        print("error: observed-latency-p95-ms must be > 0")
        return 2

    canary_report = (
        json.loads(Path(args.canary_report).read_text(encoding="utf-8"))
        if args.canary_report
        else None
    )
    fleet_health_report = (
        json.loads(Path(args.fleet_health_report).read_text(encoding="utf-8"))
        if args.fleet_health_report
        else None
    )
    fleet_update_report = (
        json.loads(Path(args.fleet_update_report).read_text(encoding="utf-8"))
        if args.fleet_update_report
        else None
    )

    report = run_drill(
        slo_error_rate_threshold=args.slo_error_rate_threshold,
        slo_latency_p95_ms_threshold=args.slo_latency_p95_ms_threshold,
        observed_error_rate=args.observed_error_rate,
        observed_latency_p95_ms=args.observed_latency_p95_ms,
        canary_report=canary_report,
        fleet_health_report=fleet_health_report,
        fleet_update_report=fleet_update_report,
    )
    report["max_failures"] = args.max_failures
    report["meets_target"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"rollout-abort-drill: {out_path}")
    print(f"policy_enforced: {report['policy_enforced']}")
    print(f"total_failures: {report['total_failures']}")
    print(f"meets_target: {report['meets_target']}")
    return 0 if report["meets_target"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
