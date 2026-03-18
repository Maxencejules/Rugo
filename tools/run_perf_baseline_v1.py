#!/usr/bin/env python3
"""Emit M24 performance baseline artifacts from booted runtime capture."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence

import runtime_capture_common_v1 as runtime_capture


THROUGHPUT_METRIC = "throughput_ops_per_sec"
LATENCY_METRIC = "latency_p95_us"

WORKLOAD_SPECS = [
    {
        "workload": "cpu_service_cycle",
        "max_throughput_regression_pct": 5.0,
        "max_latency_regression_pct": 7.0,
    },
    {
        "workload": "memory_diag_snapshot",
        "max_throughput_regression_pct": 5.0,
        "max_latency_regression_pct": 7.0,
    },
    {
        "workload": "block_recovery_cycle",
        "max_throughput_regression_pct": 6.0,
        "max_latency_regression_pct": 8.0,
    },
    {
        "workload": "network_roundtrip_cycle",
        "max_throughput_regression_pct": 6.0,
        "max_latency_regression_pct": 8.0,
    },
    {
        "workload": "service_restart_cycle",
        "max_throughput_regression_pct": 7.0,
        "max_latency_regression_pct": 10.0,
    },
    {
        "workload": "mixed_runtime_cycle",
        "max_throughput_regression_pct": 7.0,
        "max_latency_regression_pct": 10.0,
    },
]


def _total_duration_seconds(capture: Dict[str, object]) -> float:
    total_ms = sum(
        float(boot.get("duration_ms", 0.0))
        for boot in runtime_capture.iter_boots(capture)
    )
    return max(total_ms / 1000.0, 0.001)


def _task_metric(boot: Dict[str, object], service: str, key: str) -> int:
    snapshot = runtime_capture.latest_task_snapshot(boot, service)
    if not isinstance(snapshot, dict):
        return 0
    metrics = snapshot.get("metrics", {})
    if not isinstance(metrics, dict):
        return 0
    value = metrics.get(key, 0)
    return int(value) if isinstance(value, int) else 0


def _proc_metric(boot: Dict[str, object], service: str, key: str) -> int:
    latest: Dict[str, object] | None = None
    for snapshot in boot.get("process_snapshots", []):
        if not isinstance(snapshot, dict):
            continue
        if snapshot.get("service") != service:
            continue
        latest = snapshot
    if not isinstance(latest, dict):
        return 0
    metrics = latest.get("metrics", {})
    if not isinstance(metrics, dict):
        return 0
    value = metrics.get(key, 0)
    return int(value) if isinstance(value, int) else 0


def _line_latency_us(
    boot: Dict[str, object],
    start_marker: str,
    end_marker: str,
) -> float:
    start_ms = runtime_capture.find_first_line_ts(boot, start_marker)
    end_ms = runtime_capture.find_first_line_ts(boot, end_marker)
    if start_ms is None or end_ms is None or end_ms < start_ms:
        return 0.0
    return round((end_ms - start_ms) * 1000.0, 3)


def _snapshot_completion_us(boot: Dict[str, object]) -> List[float]:
    completions: List[float] = []
    lines = boot.get("serial_lines", [])
    if not isinstance(lines, list):
        return completions

    for index, entry in enumerate(lines):
        if not isinstance(entry, dict):
            continue
        line = str(entry.get("line", ""))
        if "DIAGSVC: snapshot" not in line:
            continue
        start_ms = float(entry.get("ts_ms", 0.0))
        task_seen = 0
        end_ms = start_ms
        for follow in lines[index + 1 :]:
            if not isinstance(follow, dict):
                continue
            follow_line = str(follow.get("line", ""))
            if follow_line.startswith("TASK: "):
                task_seen += 1
                end_ms = float(follow.get("ts_ms", start_ms))
                if task_seen >= 3:
                    break
        completions.append(round((end_ms - start_ms) * 1000.0, 3))
    return completions


def _workload_metrics(capture: Dict[str, object]) -> Dict[str, Dict[str, float]]:
    total_duration_seconds = _total_duration_seconds(capture)
    boots = list(runtime_capture.iter_boots(capture))

    total_run = sum(
        _task_metric(boot, service, "run")
        for boot in boots
        for service in ["timesvc", "diagsvc", "shell"]
    )
    total_diag_snapshots = sum(
        len(runtime_capture.lines_containing(boot, "DIAGSVC: snapshot")) for boot in boots
    )
    total_blk = sum(_task_metric(boot, "shell", "blk") for boot in boots)
    total_net = sum(
        runtime_capture.component_event_count(boot, "network") for boot in boots
    )
    total_restart = sum(
        _proc_metric(boot, "shell", key) for boot in boots for key in ["s", "r", "x"]
    )
    total_events = sum(
        int(boot.get("serial_line_count", 0)) for boot in boots if isinstance(boot, dict)
    )

    cpu_latencies = [
        _line_latency_us(boot, "RUGO: boot ok", "GOINIT: ready") for boot in boots
    ]
    snapshot_latencies = [
        latency for boot in boots for latency in _snapshot_completion_us(boot)
    ]
    block_latencies = [
        _line_latency_us(boots[0], "STORC4: block ready", "STORC4: journal staged")
        if len(boots) >= 1
        else 0.0,
        _line_latency_us(boots[1], "RECOV: replay ok", "STORC4: fsync ok")
        if len(boots) >= 2
        else 0.0,
    ]
    network_latencies = [
        _line_latency_us(boot, "NETC4: listen ok", "NETC4: reply ok") for boot in boots
    ]
    restart_latencies = [
        _line_latency_us(boot, "SVC: shell starting", "SVC: shell running") for boot in boots
    ]
    mixed_latencies = [
        _line_latency_us(boot, "ISOC5: observe ok", "SOAKC5: mixed ok") for boot in boots
    ]

    return {
        "cpu_service_cycle": {
            THROUGHPUT_METRIC: round(total_run / total_duration_seconds, 3),
            LATENCY_METRIC: runtime_capture.p95_ms(cpu_latencies),
        },
        "memory_diag_snapshot": {
            THROUGHPUT_METRIC: round(total_diag_snapshots / total_duration_seconds, 3),
            LATENCY_METRIC: runtime_capture.p95_ms(snapshot_latencies),
        },
        "block_recovery_cycle": {
            THROUGHPUT_METRIC: round(total_blk / total_duration_seconds, 3),
            LATENCY_METRIC: runtime_capture.p95_ms(block_latencies),
        },
        "network_roundtrip_cycle": {
            THROUGHPUT_METRIC: round(total_net / total_duration_seconds, 3),
            LATENCY_METRIC: runtime_capture.p95_ms(network_latencies),
        },
        "service_restart_cycle": {
            THROUGHPUT_METRIC: round(total_restart / total_duration_seconds, 3),
            LATENCY_METRIC: runtime_capture.p95_ms(restart_latencies),
        },
        "mixed_runtime_cycle": {
            THROUGHPUT_METRIC: round(total_events / total_duration_seconds, 3),
            LATENCY_METRIC: runtime_capture.p95_ms(mixed_latencies),
        },
    }


def run_baseline(
    runtime_capture_payload: Dict[str, object],
    runtime_capture_path: str = "",
) -> Dict[str, object]:
    workload_metrics = _workload_metrics(runtime_capture_payload)
    workloads: List[Dict[str, object]] = []
    for spec in WORKLOAD_SPECS:
        metrics = workload_metrics[spec["workload"]]
        workloads.append(
            {
                "workload": spec["workload"],
                "metrics": {
                    THROUGHPUT_METRIC: round(float(metrics[THROUGHPUT_METRIC]), 3),
                    LATENCY_METRIC: round(float(metrics[LATENCY_METRIC]), 3),
                },
                "budgets": {
                    "max_throughput_regression_pct": spec["max_throughput_regression_pct"],
                    "max_latency_regression_pct": spec["max_latency_regression_pct"],
                },
                "source_boot_profiles": list(runtime_capture_payload.get("boot_profiles", [])),
            }
        )

    stable_payload = {
        "schema": "rugo.perf_baseline.v1",
        "runtime_capture_digest": runtime_capture_payload.get("digest", ""),
        "release_image_digest": runtime_capture_payload.get("image_digest", ""),
        "workloads": [
            {
                "workload": item["workload"],
                "metrics": item["metrics"],
                "budgets": item["budgets"],
            }
            for item in workloads
        ],
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": "rugo.perf_baseline.v1",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "budget_id": "rugo.performance_budget.v1",
        "benchmark_policy_id": "rugo.benchmark_policy.v1",
        "runtime_capture_schema": runtime_capture_payload.get("schema", ""),
        "runtime_capture_path": runtime_capture_path,
        "runtime_capture_digest": runtime_capture_payload.get("digest", ""),
        "release_image_path": runtime_capture_payload.get("image_path", ""),
        "release_image_digest": runtime_capture_payload.get("image_digest", ""),
        "kernel_path": runtime_capture_payload.get("kernel_path", ""),
        "kernel_digest": runtime_capture_payload.get("kernel_digest", ""),
        "build_id": runtime_capture_payload.get("build_id", ""),
        "execution_lane": runtime_capture_payload.get("execution_lane", "qemu"),
        "trace_id": runtime_capture_payload.get("trace_id", ""),
        "workload_count": len(workloads),
        "boot_count": len(list(runtime_capture.iter_boots(runtime_capture_payload))),
        "workloads": workloads,
        "digest": digest,
    }


def load_runtime_capture(path: str) -> Dict[str, object]:
    return runtime_capture.read_json(Path(path))


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime-capture", required=True)
    parser.add_argument("--out", default="out/perf-baseline-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    capture = load_runtime_capture(args.runtime_capture)
    report = run_baseline(capture, runtime_capture_path=args.runtime_capture)

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, report)
    print(f"perf-baseline-report: {out_path}")
    print(f"workloads: {report['workload_count']}")
    print(f"release_image_path: {report['release_image_path']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
