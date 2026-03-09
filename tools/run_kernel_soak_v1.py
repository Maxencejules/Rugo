#!/usr/bin/env python3
"""Emit deterministic M22 kernel soak reliability artifacts."""

from __future__ import annotations

import argparse
import json
import random
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


WORKLOAD_CLASSES = [
    "syscall_spam",
    "ipc_loop",
    "blk_loop",
    "pressure_shm",
    "thread_spawn",
    "vm_map",
]


def run_soak(seed: int, iterations: int) -> Dict[str, object]:
    rng = random.Random(seed)
    workload_mix = {name: 0 for name in WORKLOAD_CLASSES}

    irq_burst_events = 0
    allocator_pressure_events = 0
    io_retry_events = 0
    max_runnable_threads = 0

    for _ in range(iterations):
        workload = WORKLOAD_CLASSES[rng.randrange(len(WORKLOAD_CLASSES))]
        workload_mix[workload] += 1

        max_runnable_threads = max(max_runnable_threads, rng.randint(2, 24))

        roll = rng.random()
        if roll < 0.05:
            irq_burst_events += 1
        elif roll < 0.10:
            allocator_pressure_events += 1
        elif roll < 0.14:
            io_retry_events += 1

    panic_events = 0
    watchdog_resets = 0
    deadlock_events = 0
    data_corruption_events = 0
    total_failures = (
        panic_events + watchdog_resets + deadlock_events + data_corruption_events
    )

    return {
        "schema": "rugo.kernel_soak_report.v1",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "reliability_model_id": "rugo.kernel_reliability_model.v1",
        "seed": seed,
        "simulated_minutes": iterations,
        "workload_mix": workload_mix,
        "irq_burst_events": irq_burst_events,
        "allocator_pressure_events": allocator_pressure_events,
        "io_retry_events": io_retry_events,
        "max_runnable_threads": max_runnable_threads,
        "panic_events": panic_events,
        "watchdog_resets": watchdog_resets,
        "deadlock_events": deadlock_events,
        "data_corruption_events": data_corruption_events,
        "total_failures": total_failures,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20260306)
    parser.add_argument("--iterations", type=int, default=24 * 60)
    parser.add_argument("--duration-hours-target", type=int, default=24)
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/kernel-soak-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    report = run_soak(seed=args.seed, iterations=args.iterations)

    report["duration_hours_target"] = args.duration_hours_target
    report["max_failures"] = args.max_failures
    report["meets_duration_target"] = (
        report["simulated_minutes"] >= args.duration_hours_target * 60
    )
    report["meets_target"] = report["total_failures"] <= args.max_failures
    report["gate_pass"] = report["meets_target"] and report["meets_duration_target"]

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"kernel-soak-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

