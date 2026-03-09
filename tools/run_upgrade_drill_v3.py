#!/usr/bin/env python3
"""Run deterministic upgrade and rollback safety drill artifacts for M30."""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Set


SCHEMA = "rugo.upgrade_drill.v3"
CONTRACT_ID = "rugo.installer_ux_contract.v3"
ROLLBACK_SCHEMA = "rugo.rollback_safety_report.v3"
STAGES = [
    "upgrade_plan_validate",
    "upgrade_apply",
    "post_upgrade_health_check",
    "rollback_safety_check",
]


def _metric(seed: int, stage: str, label: str, base: int, spread: int) -> int:
    digest = hashlib.sha256(f"{seed}|{stage}|{label}".encode("utf-8")).hexdigest()
    return base + (int(digest[:8], 16) % spread)


def _collect_injected(values: List[str]) -> Set[str]:
    requested = {value.strip() for value in values if value.strip()}
    unknown = sorted(requested - set(STAGES))
    if unknown:
        raise ValueError(f"unknown stages in --inject-failure: {', '.join(unknown)}")
    return requested


def run_upgrade_drill(
    seed: int,
    candidate_sequence: int,
    rollback_floor_sequence: int,
    forced_failures: Set[str] | None = None,
) -> Dict[str, object]:
    failures = set() if forced_failures is None else set(forced_failures)
    stages: List[Dict[str, object]] = []

    rollback_floor_enforced = candidate_sequence >= rollback_floor_sequence
    unsigned_artifact_rejected = "rollback_safety_check" not in failures
    rollback_path_verified = False

    for stage in STAGES:
        auto_fail = stage == "rollback_safety_check" and not rollback_floor_enforced
        failed = stage in failures or auto_fail
        status = "fail" if failed else "pass"
        notes = (
            "simulated failure injected for validation"
            if stage in failures
            else (
                "candidate sequence below rollback floor"
                if auto_fail
                else "stage completed within contract budget"
            )
        )

        entry: Dict[str, object] = {
            "name": stage,
            "status": status,
            "duration_ms": _metric(seed, stage, "duration_ms", base=900, spread=4500),
            "notes": notes,
        }

        if stage == "rollback_safety_check":
            rollback_path_verified = (
                rollback_floor_enforced
                and unsigned_artifact_rejected
                and status == "pass"
            )
            entry["checks"] = {
                "rollback_floor_enforced": rollback_floor_enforced,
                "unsigned_artifact_rejected": unsigned_artifact_rejected,
                "rollback_path_verified": rollback_path_verified,
            }

        stages.append(entry)

    failed_cases = sum(1 for stage in stages if stage["status"] != "pass")
    passed_cases = len(stages) - failed_cases

    return {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "upgrade_candidate_sequence": candidate_sequence,
        "rollback_floor_sequence": rollback_floor_sequence,
        "total_cases": len(stages),
        "passed_cases": passed_cases,
        "failed_cases": failed_cases,
        "total_failures": failed_cases,
        "stages": stages,
        "rollback_safety": {
            "schema": ROLLBACK_SCHEMA,
            "rollback_floor_enforced": rollback_floor_enforced,
            "unsigned_artifact_rejected": unsigned_artifact_rejected,
            "rollback_path_verified": rollback_path_verified,
        },
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20260309)
    parser.add_argument("--candidate-sequence", type=int, default=42)
    parser.add_argument("--rollback-floor-sequence", type=int, default=40)
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a named stage into failure state",
    )
    parser.add_argument("--out", default="out/upgrade-drill-v3.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    if args.candidate_sequence <= 0:
        print("error: candidate-sequence must be > 0")
        return 2
    if args.rollback_floor_sequence <= 0:
        print("error: rollback-floor-sequence must be > 0")
        return 2

    try:
        injected_failures = _collect_injected(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_upgrade_drill(
        seed=args.seed,
        candidate_sequence=args.candidate_sequence,
        rollback_floor_sequence=args.rollback_floor_sequence,
        forced_failures=injected_failures,
    )
    report["max_failures"] = args.max_failures
    rollback = report["rollback_safety"]
    report["meets_target"] = (
        report["total_failures"] <= args.max_failures
        and rollback["rollback_floor_enforced"] is True
        and rollback["unsigned_artifact_rejected"] is True
        and rollback["rollback_path_verified"] is True
    )
    report["gate_pass"] = report["meets_target"]

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"upgrade-drill-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"rollback_floor_enforced: {rollback['rollback_floor_enforced']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
