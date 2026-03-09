#!/usr/bin/env python3
"""Collect deterministic diagnostic snapshot artifacts for M29."""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Set


SCHEMA = "rugo.diagnostic_snapshot.v2"
CONTRACT_ID = "rugo.observability_contract.v2"
CHECKS = [
    "scheduler_latency",
    "memory_pressure",
    "filesystem_service",
    "package_service",
    "network_service",
]


def _metric(seed: int, name: str, label: str, base: int, spread: int) -> int:
    digest = hashlib.sha256(f"{seed}|{name}|{label}".encode("utf-8")).hexdigest()
    return base + (int(digest[:8], 16) % spread)


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


def collect_snapshot(
    seed: int,
    trace_bundle: Dict[str, object] | None = None,
    unhealthy_checks: Set[str] | None = None,
) -> Dict[str, object]:
    forced_unhealthy = set() if unhealthy_checks is None else set(unhealthy_checks)
    checks: List[Dict[str, object]] = []

    for check_name in CHECKS:
        healthy = check_name not in forced_unhealthy
        checks.append(
            {
                "name": check_name,
                "status": "ok" if healthy else "degraded",
                "latency_ms": _metric(seed, check_name, "latency", base=2, spread=12),
                "details": (
                    "diagnostic check within expected bounds"
                    if healthy
                    else "simulated degraded check injected for validation"
                ),
            }
        )

    unhealthy_count = sum(1 for item in checks if item["status"] != "ok")
    trace_ref: Dict[str, object] = {
        "attached": False,
        "schema": "",
        "contract_id": "",
        "gate_pass": None,
    }
    if trace_bundle is not None:
        trace_ref = {
            "attached": True,
            "schema": trace_bundle.get("schema", ""),
            "contract_id": trace_bundle.get("contract_id", ""),
            "gate_pass": trace_bundle.get("gate_pass"),
        }

    return {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "health_checks": checks,
        "resource_snapshot": {
            "cpu_load_pct": _metric(seed, "resource", "cpu", base=18, spread=23),
            "memory_used_mb": _metric(seed, "resource", "memory", base=220, spread=90),
            "io_queue_depth": _metric(seed, "resource", "io", base=1, spread=6),
        },
        "trace_reference": trace_ref,
        "unhealthy_checks": unhealthy_count,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20260309)
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

    trace_bundle: Dict[str, object] | None = None
    if args.trace_bundle:
        trace_bundle = json.loads(Path(args.trace_bundle).read_text(encoding="utf-8"))

    report = collect_snapshot(
        seed=args.seed,
        trace_bundle=trace_bundle,
        unhealthy_checks=injected_unhealthy,
    )
    report["max_unhealthy_checks"] = args.max_unhealthy_checks
    report["gate_pass"] = report["unhealthy_checks"] <= args.max_unhealthy_checks

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"diagnostic-snapshot-report: {out_path}")
    print(f"unhealthy_checks: {report['unhealthy_checks']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
