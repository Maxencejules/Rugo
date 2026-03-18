#!/usr/bin/env python3
"""Run deterministic fleet update orchestration simulation for M33."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set, Tuple

import t4_runtime_qualification_common_v1 as runtime_qual

SCHEMA = "rugo.fleet_update_sim_report.v1"
POLICY_ID = "rugo.fleet_update_policy.v1"
DEFAULT_SEED = 20260309
GROUPS: Sequence[Tuple[str, int, str]] = runtime_qual.fleet_lab_layout()


def _known_groups() -> Set[str]:
    return {group for group, _, _ in GROUPS}


def _parse_injected_failures(values: Sequence[str]) -> Set[str]:
    requested = {value.strip() for value in values if value.strip()}
    unknown = sorted(requested - _known_groups())
    if unknown:
        raise ValueError(f"unknown group ids in --inject-failure-group: {', '.join(unknown)}")
    return requested


def run_sim(
    seed: int,
    target_version: str,
    min_success_rate: float,
    injected_failure_groups: Set[str] | None = None,
    *,
    runtime_capture_payload: Dict[str, object] | None = None,
    runtime_capture_path: str = "",
    fixture: bool = False,
) -> Dict[str, object]:
    failures = set() if injected_failure_groups is None else set(injected_failure_groups)
    capture, capture_source = runtime_qual.load_runtime_capture(
        runtime_capture_path=runtime_capture_path,
        fixture=fixture,
    ) if runtime_capture_payload is None else (runtime_capture_payload, runtime_capture_path or "provided")
    lab_nodes = runtime_qual.build_fleet_lab(
        capture,
        seed=seed,
        target_version=target_version,
        injected_failure_groups=failures,
    )
    groups: List[Dict[str, object]] = []
    stage_blocked = False
    group_failures = 0

    for group_id, _nodes_total, current_version in GROUPS:
        group_nodes = [node for node in lab_nodes if node["group_id"] == group_id]
        nodes_total = len(group_nodes)
        healthy_nodes = [node for node in group_nodes if node["healthy"] is True]
        success_rate = round((len(healthy_nodes) / max(1, nodes_total)), 4)
        passes = success_rate >= min_success_rate
        promoted = (not stage_blocked) and passes
        rollback_triggered = (not passes) or stage_blocked
        nodes_updated = len(healthy_nodes) if promoted else 0

        if not passes:
            group_failures += 1
            stage_blocked = True
        elif stage_blocked:
            group_failures += 1

        groups.append(
            {
                "group_id": group_id,
                "current_version": current_version,
                "target_version": target_version,
                "nodes_total": nodes_total,
                "nodes_updated": nodes_updated,
                "success_rate": success_rate,
                "min_success_rate": min_success_rate,
                "promoted": promoted,
                "rollback_triggered": rollback_triggered,
                "pass": passes and not stage_blocked,
                "nodes": group_nodes,
            }
        )

    checks = [
        {
            "name": "group_set_non_empty",
            "pass": len(groups) > 0,
        },
        {
            "name": "all_groups_meet_success_rate",
            "pass": all(entry["success_rate"] >= min_success_rate for entry in groups),
        },
        {
            "name": "promotion_stops_after_first_failure",
            "pass": all(
                (not groups[idx - 1]["rollback_triggered"]) or (not entry["promoted"])
                for idx, entry in enumerate(groups)
                if idx > 0
            ),
        },
        {
            "name": "runtime_capture_bound",
            "pass": bool(capture.get("digest")),
        },
    ]

    total_failures = group_failures + sum(1 for check in checks if check["pass"] is False)
    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "control_plane_mode": "runtime_lab",
        "runtime_capture_path": capture_source,
        "runtime_capture_digest": capture.get("digest", ""),
        "release_image_path": capture.get("image_path", ""),
        "release_image_digest": capture.get("image_digest", ""),
        "target_version": target_version,
        "lab_nodes_total": len(lab_nodes),
        "groups": groups,
        "checks": checks,
        "total_failures": total_failures,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument("--target-version", default="2.4.0")
    parser.add_argument("--min-success-rate", type=float, default=0.98)
    parser.add_argument("--inject-failure-group", action="append", default=[])
    parser.add_argument(
        "--runtime-capture",
        default="",
        help="booted runtime capture backing the fleet control-plane lab",
    )
    parser.add_argument(
        "--fixture",
        action="store_true",
        help="use the deterministic booted runtime fixture instead of out/booted-runtime-v1.json",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/fleet-update-sim-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2
    if not (0.0 < args.min_success_rate <= 1.0):
        print("error: min-success-rate must be in (0, 1]")
        return 2

    try:
        injected_failure_groups = _parse_injected_failures(args.inject_failure_group)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_sim(
        seed=args.seed,
        target_version=args.target_version,
        min_success_rate=args.min_success_rate,
        injected_failure_groups=injected_failure_groups,
        runtime_capture_path=args.runtime_capture,
        fixture=args.fixture,
    )
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"fleet-update-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
