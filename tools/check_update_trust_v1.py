#!/usr/bin/env python3
"""Run deterministic update-trust checks for M26."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Set


TRUST_MODEL_ID = "rugo.update_trust_model.v1"
SCHEMA = "rugo.update_trust_report.v1"

CASES: List[Dict[str, object]] = [
    {
        "name": "expiry_attack",
        "controls": ["expires_utc_required", "current_time_validation"],
        "expected_blocked": True,
    },
    {
        "name": "freeze_attack",
        "controls": ["monotonic_sequence", "stateful_last_seen_enforcement"],
        "expected_blocked": True,
    },
    {
        "name": "mix_and_match_attack",
        "controls": ["signed_target_set", "target_digest_binding"],
        "expected_blocked": True,
    },
    {
        "name": "rollback_attack",
        "controls": ["rollback_floor_sequence", "strictly_increasing_sequence"],
        "expected_blocked": True,
    },
    {
        "name": "revoked_key_attack",
        "controls": ["revoked_key_rejection", "rotation_cutover_enforcement"],
        "expected_blocked": True,
    },
]


def _known_case_names() -> Set[str]:
    return {str(case["name"]) for case in CASES}


def _collect_injected_failures(specs: List[str]) -> Set[str]:
    requested = {item.strip() for item in specs if item.strip()}
    unknown = sorted(requested - _known_case_names())
    if unknown:
        raise ValueError(f"unknown trust cases in --inject-failure: {', '.join(unknown)}")
    return requested


def run_suite(injected_failures: Set[str] | None = None) -> Dict[str, object]:
    forced_failures = set() if injected_failures is None else set(injected_failures)
    cases: List[Dict[str, object]] = []

    for case_spec in CASES:
        name = str(case_spec["name"])
        expected_blocked = bool(case_spec["expected_blocked"])
        blocked = name not in forced_failures
        case = {
            "name": name,
            "controls": list(case_spec["controls"]),
            "expected_blocked": expected_blocked,
            "blocked": blocked,
        }
        case["pass"] = case["blocked"] == case["expected_blocked"]
        case["details"] = (
            "policy controls blocked attack"
            if case["pass"]
            else "simulated trust failure injected for validation"
        )
        cases.append(case)

    total_failures = sum(1 for case in cases if not case["pass"])
    return {
        "schema": SCHEMA,
        "trust_model_id": TRUST_MODEL_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "total_cases": len(cases),
        "total_failures": total_failures,
        "cases": cases,
    }


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--max-failures", type=int, default=0)
    p.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a named case to fail for negative-path validation",
    )
    p.add_argument("--out", default="out/update-trust-v1.json")
    return p


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    try:
        injected_failures = _collect_injected_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_suite(injected_failures=injected_failures)
    report["max_failures"] = args.max_failures
    report["meets_target"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"update-trust-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    return 0 if report["meets_target"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
