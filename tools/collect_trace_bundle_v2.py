#!/usr/bin/env python3
"""Collect deterministic observability trace bundle artifacts for M29."""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Set


SCHEMA = "rugo.trace_bundle.v2"
CONTRACT_ID = "rugo.observability_contract.v2"
DEFAULT_SERVICES = ["init", "svcman", "pkgd", "netd"]


def _metric(seed: int, service: str, label: str, base: int, spread: int) -> int:
    digest = hashlib.sha256(f"{seed}|{service}|{label}".encode("utf-8")).hexdigest()
    return base + (int(digest[:8], 16) % spread)


def _known_services() -> Set[str]:
    return set(DEFAULT_SERVICES)


def _collect_injected(values: List[str], arg_name: str) -> Set[str]:
    injected = {value.strip() for value in values if value.strip()}
    unknown = sorted(injected - _known_services())
    if unknown:
        raise ValueError(f"unknown services in {arg_name}: {', '.join(unknown)}")
    return injected


def collect_trace_bundle(
    seed: int,
    window_seconds: int,
    inject_errors: Set[str] | None = None,
    inject_drops: Set[str] | None = None,
) -> Dict[str, object]:
    error_services = set() if inject_errors is None else set(inject_errors)
    drop_services = set() if inject_drops is None else set(inject_drops)
    services: List[Dict[str, object]] = []

    total_spans = 0
    total_errors = 0
    total_dropped_spans = 0

    for name in DEFAULT_SERVICES:
        span_count = _metric(seed, name, "span_count", base=180, spread=140)
        dropped = 1 if name in drop_services else 0
        errors = 1 if name in error_services else 0

        service_entry = {
            "service": name,
            "span_count": span_count,
            "error_events": errors,
            "dropped_spans": dropped,
            "latency_p95_ms": _metric(seed, name, "latency", base=4, spread=16),
            "status": "degraded" if (errors or dropped) else "ok",
        }
        services.append(service_entry)

        total_spans += span_count
        total_errors += errors
        total_dropped_spans += dropped

    return {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "window_seconds": window_seconds,
        "sources": services,
        "totals": {
            "total_services": len(services),
            "total_spans": total_spans,
            "total_errors": total_errors,
            "total_dropped_spans": total_dropped_spans,
        },
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20260309)
    parser.add_argument("--window-seconds", type=int, default=300)
    parser.add_argument("--max-errors", type=int, default=0)
    parser.add_argument("--max-dropped-spans", type=int, default=0)
    parser.add_argument(
        "--inject-error",
        action="append",
        default=[],
        help="force a service to report an error event",
    )
    parser.add_argument(
        "--inject-drop",
        action="append",
        default=[],
        help="force a service to report dropped spans",
    )
    parser.add_argument("--out", default="out/trace-bundle-v2.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    try:
        inject_errors = _collect_injected(args.inject_error, "--inject-error")
        inject_drops = _collect_injected(args.inject_drop, "--inject-drop")
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = collect_trace_bundle(
        seed=args.seed,
        window_seconds=args.window_seconds,
        inject_errors=inject_errors,
        inject_drops=inject_drops,
    )
    totals = report["totals"]
    report["max_errors"] = args.max_errors
    report["max_dropped_spans"] = args.max_dropped_spans
    report["gate_pass"] = (
        totals["total_errors"] <= args.max_errors
        and totals["total_dropped_spans"] <= args.max_dropped_spans
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"trace-bundle-report: {out_path}")
    print(f"total_errors: {totals['total_errors']}")
    print(f"total_dropped_spans: {totals['total_dropped_spans']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
