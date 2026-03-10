#!/usr/bin/env python3
"""Run deterministic POSIX/process/readiness gap closure report checks for M41."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set


SCHEMA = "rugo.posix_gap_report.v2"
PROFILE_ID = "rugo.compat_profile.v5"
COVERAGE_MATRIX_DOC = "docs/runtime/syscall_coverage_matrix_v4.md"
DEFAULT_SEED = 20260310

REQUIRED_SURFACES = (
    "waitid",
    "sigprocmask",
    "sigpending",
    "poll",
    "pselect",
    "ppoll",
    "eventfd",
    "sendmsg",
    "recvmsg",
    "socketpair",
)

OPTIONAL_SURFACES = (
    "accept4",
    "clock_nanosleep",
    "signalfd",
    "timerfd_create",
)

DEFERRED_SURFACES: Dict[str, str] = {
    "fork": "ENOSYS",
    "clone": "ENOSYS",
    "epoll": "ENOSYS",
    "io_uring": "ENOSYS",
    "namespaces": "ENOSYS",
    "cgroups": "ENOSYS",
    "af_netlink": "ENOSYS",
    "raw_packet": "ENOSYS",
}


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _normalize_values(values: Sequence[str]) -> Set[str]:
    return {value.strip() for value in values if value.strip()}


def _validate_subset(values: Set[str], allowed: Sequence[str], label: str) -> None:
    unknown = sorted(values - set(allowed))
    if unknown:
        raise ValueError(f"unknown {label}: {', '.join(unknown)}")


def run_report(
    seed: int,
    missing_required: Set[str] | None = None,
    deferred_violations: Set[str] | None = None,
    unexpected: Set[str] | None = None,
) -> Dict[str, object]:
    missing = set() if missing_required is None else set(missing_required)
    violations = set() if deferred_violations is None else set(deferred_violations)
    unexpected_surfaces = set() if unexpected is None else set(unexpected)

    required_rows: List[Dict[str, object]] = []
    for surface in REQUIRED_SURFACES:
        is_missing = surface in missing
        required_rows.append(
            {
                "surface": surface,
                "status": "missing" if is_missing else "implemented",
                "deterministic": not is_missing,
            }
        )

    optional_present = sorted(
        surface for surface in OPTIONAL_SURFACES if (_noise(seed, surface) % 3) != 0
    )

    deferred_rows: List[Dict[str, object]] = []
    for surface in sorted(DEFERRED_SURFACES.keys()):
        expected_errno = DEFERRED_SURFACES[surface]
        violation = surface in violations
        deferred_rows.append(
            {
                "surface": surface,
                "status": "deferred",
                "deterministic": not violation,
                "expected_errno": expected_errno,
                "deterministic_error": "EAGAIN" if violation else expected_errno,
            }
        )

    missing_required_sorted = sorted(
        row["surface"] for row in required_rows if row["status"] == "missing"
    )
    deferred_violation_sorted = sorted(
        row["surface"] for row in deferred_rows if row["deterministic"] is False
    )
    unexpected_sorted = sorted(unexpected_surfaces)

    issues: List[Dict[str, str]] = []
    for surface in missing_required_sorted:
        issues.append({"surface": surface, "reason": "missing_required"})
    for surface in deferred_violation_sorted:
        issues.append({"surface": surface, "reason": "deferred_behavior_nondeterministic"})
    for surface in unexpected_sorted:
        issues.append({"surface": surface, "reason": "unexpected_surface"})
    issues = sorted(issues, key=lambda item: (item["reason"], item["surface"]))

    stable_payload = {
        "schema": SCHEMA,
        "profile_id": PROFILE_ID,
        "seed": seed,
        "required": required_rows,
        "deferred": deferred_rows,
        "optional_present": optional_present,
        "unexpected": unexpected_sorted,
        "issues": issues,
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    gate_pass = len(issues) == 0
    return {
        "schema": SCHEMA,
        "profile_id": PROFILE_ID,
        "coverage_matrix_doc": COVERAGE_MATRIX_DOC,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "required_surfaces": required_rows,
        "optional_present": optional_present,
        "deferred_surfaces": deferred_rows,
        "missing_required": missing_required_sorted,
        "deferred_violations": deferred_violation_sorted,
        "unexpected": unexpected_sorted,
        "issues": issues,
        "total_failures": len(issues),
        "gate_pass": gate_pass,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--inject-missing-required",
        action="append",
        default=[],
        help="mark a required surface as missing",
    )
    parser.add_argument(
        "--inject-deferred-violation",
        action="append",
        default=[],
        help="force a deferred surface to violate deterministic ENOSYS behavior",
    )
    parser.add_argument(
        "--inject-unexpected",
        action="append",
        default=[],
        help="inject unexpected implemented surface identifiers",
    )
    parser.add_argument("--out", default="out/posix-gap-report-v2.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    missing = _normalize_values(args.inject_missing_required)
    violations = _normalize_values(args.inject_deferred_violation)
    unexpected = _normalize_values(args.inject_unexpected)

    try:
        _validate_subset(missing, REQUIRED_SURFACES, "--inject-missing-required surfaces")
        _validate_subset(
            violations,
            DEFERRED_SURFACES.keys(),
            "--inject-deferred-violation surfaces",
        )
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_report(
        seed=args.seed,
        missing_required=missing,
        deferred_violations=violations,
        unexpected=unexpected,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"posix-gap-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
