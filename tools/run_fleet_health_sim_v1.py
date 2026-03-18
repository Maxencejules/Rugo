#!/usr/bin/env python3
"""Run deterministic fleet health simulation for M33 rollout gates."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set, Tuple

import t4_runtime_qualification_common_v1 as runtime_qual

SCHEMA = "rugo.fleet_health_report.v1"
POLICY_ID = "rugo.fleet_health_policy.v1"
DEFAULT_SEED = 20260309
CLUSTERS: Sequence[Tuple[str, int]] = (
    ("canary", 30),
    ("core", 420),
    ("edge", 550),
)


def _known_clusters() -> Set[str]:
    return {cluster for cluster, _ in CLUSTERS}


def _parse_injected_failures(values: Sequence[str]) -> Set[str]:
    requested = {value.strip() for value in values if value.strip()}
    unknown = sorted(requested - _known_clusters())
    if unknown:
        raise ValueError(f"unknown cluster ids in --inject-failure-cluster: {', '.join(unknown)}")
    return requested


def run_sim(
    seed: int,
    max_fleet_degraded_ratio: float,
    max_fleet_error_rate: float,
    injected_failure_clusters: Set[str] | None = None,
    *,
    runtime_capture_payload: Dict[str, object] | None = None,
    runtime_capture_path: str = "",
    fixture: bool = False,
) -> Dict[str, object]:
    failures = set() if injected_failure_clusters is None else set(injected_failure_clusters)
    capture, capture_source = runtime_qual.load_runtime_capture(
        runtime_capture_path=runtime_capture_path,
        fixture=fixture,
    ) if runtime_capture_payload is None else (runtime_capture_payload, runtime_capture_path or "provided")
    lab_nodes = runtime_qual.build_fleet_lab(
        capture,
        seed=seed,
        target_version="2.4.0",
        injected_failure_clusters=failures,
    )
    clusters: List[Dict[str, object]] = []

    total_nodes = 0
    degraded_nodes_total = 0
    weighted_error_sum = 0.0

    for cluster_id, _nodes_total in CLUSTERS:
        cluster_nodes = [node for node in lab_nodes if node["cluster_id"] == cluster_id]
        nodes_total = len(cluster_nodes)
        degraded_nodes = sum(1 for node in cluster_nodes if node["healthy"] is not True)
        error_rate = round(
            sum(float(node["error_rate"]) for node in cluster_nodes) / max(1, nodes_total),
            4,
        )
        latencies = sorted(float(node["shell_latency_ms_p95"]) for node in cluster_nodes)
        latency_p95_ms = round(latencies[-1] if latencies else 0.0, 3)

        degraded_ratio = round(degraded_nodes / nodes_total, 4)
        within_slo = error_rate <= max_fleet_error_rate and degraded_ratio <= max_fleet_degraded_ratio

        clusters.append(
            {
                "cluster_id": cluster_id,
                "nodes_total": nodes_total,
                "nodes_degraded": degraded_nodes,
                "degraded_ratio": degraded_ratio,
                "error_rate": error_rate,
                "latency_p95_ms": latency_p95_ms,
                "within_slo": within_slo,
                "nodes": cluster_nodes,
            }
        )

        total_nodes += nodes_total
        degraded_nodes_total += degraded_nodes
        weighted_error_sum += error_rate * nodes_total

    fleet_degraded_ratio = round(degraded_nodes_total / total_nodes, 4)
    fleet_error_rate = round(weighted_error_sum / total_nodes, 4)

    checks = [
        {
            "name": "clusters_present",
            "pass": len(clusters) > 0,
        },
        {
            "name": "fleet_degraded_ratio_within_budget",
            "pass": fleet_degraded_ratio <= max_fleet_degraded_ratio,
        },
        {
            "name": "fleet_error_rate_within_budget",
            "pass": fleet_error_rate <= max_fleet_error_rate,
        },
        {
            "name": "all_clusters_within_slo",
            "pass": all(cluster["within_slo"] for cluster in clusters),
        },
        {
            "name": "runtime_capture_bound",
            "pass": bool(capture.get("digest")),
        },
    ]
    total_failures = sum(1 for check in checks if check["pass"] is False)
    fleet_state = "healthy" if total_failures == 0 else "degraded"

    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "control_plane_mode": "runtime_lab",
        "runtime_capture_path": capture_source,
        "runtime_capture_digest": capture.get("digest", ""),
        "release_image_path": capture.get("image_path", ""),
        "max_fleet_degraded_ratio": max_fleet_degraded_ratio,
        "max_fleet_error_rate": max_fleet_error_rate,
        "fleet_state": fleet_state,
        "fleet_degraded_ratio": fleet_degraded_ratio,
        "fleet_error_rate": fleet_error_rate,
        "lab_nodes_total": len(lab_nodes),
        "clusters": clusters,
        "checks": checks,
        "total_failures": total_failures,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument("--max-fleet-degraded-ratio", type=float, default=0.05)
    parser.add_argument("--max-fleet-error-rate", type=float, default=0.02)
    parser.add_argument("--inject-failure-cluster", action="append", default=[])
    parser.add_argument(
        "--runtime-capture",
        default="",
        help="booted runtime capture backing the fleet health lab",
    )
    parser.add_argument(
        "--fixture",
        action="store_true",
        help="use the deterministic booted runtime fixture instead of out/booted-runtime-v1.json",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/fleet-health-sim-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2
    if not (0.0 <= args.max_fleet_degraded_ratio <= 1.0):
        print("error: max-fleet-degraded-ratio must be in [0, 1]")
        return 2
    if not (0.0 <= args.max_fleet_error_rate <= 1.0):
        print("error: max-fleet-error-rate must be in [0, 1]")
        return 2

    try:
        injected_failure_clusters = _parse_injected_failures(args.inject_failure_cluster)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_sim(
        seed=args.seed,
        max_fleet_degraded_ratio=args.max_fleet_degraded_ratio,
        max_fleet_error_rate=args.max_fleet_error_rate,
        injected_failure_clusters=injected_failure_clusters,
        runtime_capture_path=args.runtime_capture,
        fixture=args.fixture,
    )
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"fleet-health-report: {out_path}")
    print(f"fleet_state: {report['fleet_state']}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
