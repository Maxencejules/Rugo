#!/usr/bin/env python3
"""Collect M29 diagnostic snapshot artifacts from booted runtime capture."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Set

import runtime_capture_common_v1 as runtime_capture


SCHEMA = "rugo.diagnostic_snapshot.v2"
CONTRACT_ID = "rugo.observability_contract.v2"
CHECKS = [
    "service_manager",
    "memory_pressure",
    "filesystem_recovery",
    "network_stack",
    "isolation_observer",
]


def _known_checks() -> Set[str]:
    return set(CHECKS)


def _collect_injected(values: List[str]) -> Set[str]:
    requested = {value.strip() for value in values if value.strip()}
    unknown = sorted(requested - _known_checks())
    if unknown:
        raise ValueError(
            f"unknown checks in --inject-unhealthy-check: {', '.join(unknown)}"
        )
    return requested


def _check_statuses(capture: Dict[str, object]) -> Dict[str, bool]:
    boots = list(runtime_capture.iter_boots(capture))
    return {
        "service_manager": all(
            runtime_capture.find_first_line_ts(boot, "GOSVCM: start") is not None
            and runtime_capture.find_first_line_ts(boot, "GOINIT: ready") is not None
            for boot in boots
        ),
        "memory_pressure": sum(
            len(runtime_capture.lines_containing(boot, "DIAGSVC: snapshot")) for boot in boots
        )
        >= 2,
        "filesystem_recovery": (
            len(boots) >= 2
            and runtime_capture.find_first_line_ts(boots[0], "STORC4: journal staged") is not None
            and runtime_capture.find_first_line_ts(boots[1], "RECOV: replay ok") is not None
            and runtime_capture.find_first_line_ts(boots[1], "STORC4: fsync ok") is not None
        ),
        "network_stack": all(
            runtime_capture.find_first_line_ts(boot, "NETC4: reply ok") is not None
            for boot in boots
        ),
        "isolation_observer": all(
            runtime_capture.find_first_line_ts(boot, "ISOC5: observe ok") is not None
            and runtime_capture.find_first_line_ts(boot, "SOAKC5: mixed ok") is not None
            for boot in boots
        ),
    }


def collect_snapshot(
    runtime_capture_payload: Dict[str, object],
    trace_bundle: Dict[str, object] | None = None,
    unhealthy_checks: Set[str] | None = None,
) -> Dict[str, object]:
    forced_unhealthy = set() if unhealthy_checks is None else set(unhealthy_checks)
    statuses = _check_statuses(runtime_capture_payload)
    boots = list(runtime_capture.iter_boots(runtime_capture_payload))
    checks: List[Dict[str, object]] = []

    for check_name in CHECKS:
        healthy = statuses.get(check_name, False) and check_name not in forced_unhealthy
        checks.append(
            {
                "name": check_name,
                "status": "ok" if healthy else "degraded",
                "details": (
                    "boot-backed runtime capture is within expected bounds"
                    if healthy
                    else "runtime-backed diagnostic check exceeded expected bounds"
                ),
            }
        )

    unhealthy_count = sum(1 for item in checks if item["status"] != "ok")
    trace_ref: Dict[str, object] = {
        "attached": False,
        "schema": "",
        "contract_id": "",
        "gate_pass": None,
        "trace_id": "",
        "trace_digest": "",
    }
    if trace_bundle is not None:
        trace_ref = {
            "attached": True,
            "schema": trace_bundle.get("schema", ""),
            "contract_id": trace_bundle.get("contract_id", ""),
            "gate_pass": trace_bundle.get("gate_pass"),
            "trace_id": trace_bundle.get("trace_id", ""),
            "trace_digest": trace_bundle.get("trace_digest", ""),
        }

    def _metric(boot: Dict[str, object], service: str, key: str) -> int:
        snapshot = runtime_capture.latest_task_snapshot(boot, service)
        if not isinstance(snapshot, dict):
            return 0
        metrics = snapshot.get("metrics", {})
        if not isinstance(metrics, dict):
            return 0
        value = metrics.get(key, 0)
        return int(value) if isinstance(value, int) else 0

    cpu_run_total = sum(
        _metric(boot, "timesvc", "run")
        + _metric(boot, "diagsvc", "run")
        + _metric(boot, "shell", "run")
        for boot in boots
    )
    shell_yields = sum(_metric(boot, "shell", "y") for boot in boots)
    block_ops = sum(_metric(boot, "shell", "blk") for boot in boots)
    socket_ops = sum(_metric(boot, "shell", "sock") for boot in boots)

    stable_payload = {
        "schema": SCHEMA,
        "runtime_capture_digest": runtime_capture_payload.get("digest", ""),
        "trace_digest": trace_ref["trace_digest"],
        "checks": [
            {"name": check["name"], "status": check["status"]} for check in checks
        ],
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "runtime_capture_schema": runtime_capture_payload.get("schema", ""),
        "runtime_capture_digest": runtime_capture_payload.get("digest", ""),
        "release_image_path": runtime_capture_payload.get("image_path", ""),
        "release_image_digest": runtime_capture_payload.get("image_digest", ""),
        "build_id": runtime_capture_payload.get("build_id", ""),
        "health_checks": checks,
        "resource_snapshot": {
            "cpu_run_total": cpu_run_total,
            "memory_pressure_score": shell_yields + len(boots),
            "io_block_ops": block_ops,
            "socket_ops": socket_ops,
        },
        "trace_reference": trace_ref,
        "unhealthy_checks": unhealthy_count,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime-capture", required=True)
    parser.add_argument("--trace-bundle", default="")
    parser.add_argument("--max-unhealthy-checks", type=int, default=0)
    parser.add_argument(
        "--inject-unhealthy-check",
        action="append",
        default=[],
        help="force a named check into degraded status",
    )
    parser.add_argument("--out", default="out/diagnostic-snapshot-v2.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    try:
        injected_unhealthy = _collect_injected(args.inject_unhealthy_check)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    capture = runtime_capture.read_json(Path(args.runtime_capture))
    trace_bundle: Dict[str, object] | None = None
    if args.trace_bundle:
        trace_bundle = json.loads(Path(args.trace_bundle).read_text(encoding="utf-8"))

    report = collect_snapshot(
        runtime_capture_payload=capture,
        trace_bundle=trace_bundle,
        unhealthy_checks=injected_unhealthy,
    )
    report["max_unhealthy_checks"] = args.max_unhealthy_checks
    report["gate_pass"] = report["unhealthy_checks"] <= args.max_unhealthy_checks

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, report)

    print(f"diagnostic-snapshot-report: {out_path}")
    print(f"unhealthy_checks: {report['unhealthy_checks']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
