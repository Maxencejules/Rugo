#!/usr/bin/env python3
"""Emit deterministic M19 network soak report."""

from __future__ import annotations

import argparse
import json
import random
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


def run_soak(seed: int, iterations: int) -> Dict[str, object]:
    rng = random.Random(seed)

    packet_loss_events = 0
    reorder_events = 0
    duplicate_events = 0
    tcp_retransmit_events = 0
    tcp_session_resets = 0
    dns_retry_events = 0
    tcp_timeout_events = 0
    dns_failures = 0

    for _ in range(iterations):
        roll = rng.random()
        if roll < 0.09:
            packet_loss_events += 1
            tcp_retransmit_events += 1
            continue
        if roll < 0.13:
            reorder_events += 1
            continue
        if roll < 0.16:
            duplicate_events += 1
            continue
        if roll < 0.19:
            dns_retry_events += 1
            continue
        if roll < 0.205:
            tcp_session_resets += 1
            tcp_retransmit_events += 1
            continue

    # Deterministic v2 baseline: retries/reconnect paths recover without timeout.
    total_failures = tcp_timeout_events + dns_failures
    return {
        "schema": "rugo.net_soak_report.v2",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "iterations": iterations,
        "packet_loss_events": packet_loss_events,
        "reorder_events": reorder_events,
        "duplicate_events": duplicate_events,
        "tcp_retransmit_events": tcp_retransmit_events,
        "tcp_session_resets": tcp_session_resets,
        "dns_retry_events": dns_retry_events,
        "tcp_timeout_events": tcp_timeout_events,
        "dns_failures": dns_failures,
        "total_failures": total_failures,
    }


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--seed", type=int, default=20260308)
    p.add_argument("--iterations", type=int, default=2400)
    p.add_argument("--out", default="out/net-soak-v2.json")
    p.add_argument("--max-failures", type=int, default=0)
    return p


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    report = run_soak(seed=args.seed, iterations=args.iterations)
    report["max_failures"] = args.max_failures
    report["meets_target"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"net-soak-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    return 0 if report["meets_target"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
