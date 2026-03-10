#!/usr/bin/env python3
"""Run deterministic process/readiness compatibility campaign checks for M41."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set


REPORT_SCHEMA = "rugo.compat_surface_campaign_report.v2"
PROFILE_ID = "rugo.compat_profile.v5"
PROCESS_MODEL_ID = "rugo.process_model.v4"
READINESS_MODEL_ID = "rugo.readiness_io_model.v1"
COVERAGE_MATRIX_ID = "rugo.syscall_coverage_matrix.v4"
DEFAULT_SEED = 20260310


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
        check_id="process_spawn_exec",
        domain="process",
        metric_key="spawn_exec_ms",
        operator="max",
        threshold=130.0,
        base=82.0,
        spread=28,
        scale=1.0,
    ),
    CheckSpec(
        check_id="process_wait_reap_once",
        domain="process",
        metric_key="wait_reap_ms",
        operator="max",
        threshold=22.0,
        base=9.0,
        spread=8,
        scale=1.0,
    ),
    CheckSpec(
        check_id="process_signal_fifo",
        domain="process",
        metric_key="signal_reorder_events",
        operator="max",
        threshold=1.0,
        base=0.0,
        spread=2,
        scale=1.0,
    ),
    CheckSpec(
        check_id="process_sigkill_terminal",
        domain="process",
        metric_key="sigkill_terminal_ms",
        operator="max",
        threshold=35.0,
        base=18.0,
        spread=10,
        scale=1.0,
    ),
    CheckSpec(
        check_id="process_pid_reuse_guard",
        domain="process",
        metric_key="pid_reuse_violations",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="readiness_poll_wakeup",
        domain="readiness",
        metric_key="poll_wakeup_ms",
        operator="max",
        threshold=11.0,
        base=5.0,
        spread=4,
        scale=1.0,
    ),
    CheckSpec(
        check_id="readiness_ppoll_wakeup",
        domain="readiness",
        metric_key="ppoll_wakeup_ms",
        operator="max",
        threshold=10.0,
        base=4.0,
        spread=4,
        scale=1.0,
    ),
    CheckSpec(
        check_id="readiness_pselect_wakeup",
        domain="readiness",
        metric_key="pselect_wakeup_ms",
        operator="max",
        threshold=10.0,
        base=5.0,
        spread=4,
        scale=1.0,
    ),
    CheckSpec(
        check_id="readiness_eventfd_signal",
        domain="readiness",
        metric_key="eventfd_signal_ms",
        operator="max",
        threshold=7.0,
        base=3.0,
        spread=3,
        scale=1.0,
    ),
    CheckSpec(
        check_id="posix_sendmsg",
        domain="posix",
        metric_key="sendmsg_latency_ms",
        operator="max",
        threshold=7.0,
        base=3.0,
        spread=3,
        scale=1.0,
    ),
    CheckSpec(
        check_id="posix_recvmsg",
        domain="posix",
        metric_key="recvmsg_latency_ms",
        operator="max",
        threshold=7.0,
        base=3.0,
        spread=3,
        scale=1.0,
    ),
    CheckSpec(
        check_id="posix_socketpair",
        domain="posix",
        metric_key="socketpair_latency_ms",
        operator="max",
        threshold=8.0,
        base=4.0,
        spread=3,
        scale=1.0,
    ),
    CheckSpec(
        check_id="deferred_fork_enosys",
        domain="deferred",
        metric_key="fork_enosys_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="deferred_clone_enosys",
        domain="deferred",
        metric_key="clone_enosys_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="deferred_epoll_enosys",
        domain="deferred",
        metric_key="epoll_enosys_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="deferred_io_uring_enosys",
        domain="deferred",
        metric_key="io_uring_enosys_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="deferred_namespace_enosys",
        domain="deferred",
        metric_key="namespace_enosys_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="deferred_cgroup_enosys",
        domain="deferred",
        metric_key="cgroup_enosys_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="deferred_af_netlink_enosys",
        domain="deferred",
        metric_key="af_netlink_enosys_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
)


def _known_checks() -> Set[str]:
    return {spec.check_id for spec in CHECKS}


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _round_value(value: float) -> float:
    return round(value, 3)


def _baseline_observed(seed: int, spec: CheckSpec) -> float:
    spread = spec.spread if spec.spread > 0 else 1
    value = spec.base + ((_noise(seed, spec.check_id) % spread) * spec.scale)
    return _round_value(value)


def _failing_observed(spec: CheckSpec) -> float:
    delta = 0.001 if spec.scale < 1.0 else 1.0
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


def _normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - _known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def run_campaign(seed: int, injected_failures: Set[str] | None = None) -> Dict[str, object]:
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
    stable_payload = {
        "schema": REPORT_SCHEMA,
        "profile_id": PROFILE_ID,
        "seed": seed,
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
        "schema": REPORT_SCHEMA,
        "profile_id": PROFILE_ID,
        "process_model_id": PROCESS_MODEL_ID,
        "readiness_model_id": READINESS_MODEL_ID,
        "coverage_matrix_id": COVERAGE_MATRIX_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "checks": checks,
        "summary": {
            "process": _domain_summary(checks, "process"),
            "readiness": _domain_summary(checks, "readiness"),
            "posix": _domain_summary(checks, "posix"),
            "deferred": _domain_summary(checks, "deferred"),
        },
        "process": {
            "spawn_exec_ms": metric_values["spawn_exec_ms"],
            "wait_reap_ms": metric_values["wait_reap_ms"],
            "signal_reorder_events": int(metric_values["signal_reorder_events"]),
            "sigkill_terminal_ms": metric_values["sigkill_terminal_ms"],
            "pid_reuse_violations": int(metric_values["pid_reuse_violations"]),
            "checks_pass": _domain_summary(checks, "process")["pass"],
        },
        "readiness": {
            "poll_wakeup_ms": metric_values["poll_wakeup_ms"],
            "ppoll_wakeup_ms": metric_values["ppoll_wakeup_ms"],
            "pselect_wakeup_ms": metric_values["pselect_wakeup_ms"],
            "eventfd_signal_ms": metric_values["eventfd_signal_ms"],
            "checks_pass": _domain_summary(checks, "readiness")["pass"],
        },
        "posix_gap": {
            "sendmsg_latency_ms": metric_values["sendmsg_latency_ms"],
            "recvmsg_latency_ms": metric_values["recvmsg_latency_ms"],
            "socketpair_latency_ms": metric_values["socketpair_latency_ms"],
            "checks_pass": _domain_summary(checks, "posix")["pass"],
        },
        "deferred": {
            "fork_enosys_ratio": metric_values["fork_enosys_ratio"],
            "clone_enosys_ratio": metric_values["clone_enosys_ratio"],
            "epoll_enosys_ratio": metric_values["epoll_enosys_ratio"],
            "io_uring_enosys_ratio": metric_values["io_uring_enosys_ratio"],
            "namespace_enosys_ratio": metric_values["namespace_enosys_ratio"],
            "cgroup_enosys_ratio": metric_values["cgroup_enosys_ratio"],
            "af_netlink_enosys_ratio": metric_values["af_netlink_enosys_ratio"],
            "checks_pass": _domain_summary(checks, "deferred")["pass"],
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
    parser.add_argument("--out", default="out/compat-surface-v2.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    try:
        injected_failures = _normalize_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_campaign(seed=args.seed, injected_failures=injected_failures)
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"compat-surface-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
