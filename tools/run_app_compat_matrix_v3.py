#!/usr/bin/env python3
"""Run deterministic app compatibility matrix checks for M27."""

from __future__ import annotations

import argparse
from dataclasses import dataclass, replace
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Set


PROFILE_ID = "rugo.compat_profile.v3"
PROFILE_LABEL = "compat_profile_v3"
TIER_SCHEMA = "rugo.app_compat_tiers.v1"
REPORT_SCHEMA = "rugo.app_compat_matrix_report.v3"

CLASS_THRESHOLDS: Dict[str, Dict[str, object]] = {
    "cli": {"tier": "tier_cli", "min_cases": 14, "min_pass_rate": 0.90},
    "runtime": {"tier": "tier_runtime", "min_cases": 10, "min_pass_rate": 0.80},
    "service": {"tier": "tier_service", "min_cases": 8, "min_pass_rate": 0.80},
}


@dataclass(frozen=True)
class AppCompatCase:
    case_id: str
    app_id: str
    app_class: str
    tier: str
    passed: bool
    signed: bool = True
    deterministic: bool = True
    abi_profile: str = PROFILE_LABEL


def _baseline_cases() -> List[AppCompatCase]:
    cases: List[AppCompatCase] = []

    for idx in range(14):
        cases.append(
            AppCompatCase(
                case_id=f"cli-{idx:02d}",
                app_id=f"cli-tool-{idx:02d}",
                app_class="cli",
                tier="tier_cli",
                passed=idx != 13,  # 13/14 pass => 0.9285...
            )
        )

    for idx in range(10):
        cases.append(
            AppCompatCase(
                case_id=f"runtime-{idx:02d}",
                app_id=f"runtime-app-{idx:02d}",
                app_class="runtime",
                tier="tier_runtime",
                passed=idx not in {8, 9},  # 8/10 pass => 0.80
            )
        )

    for idx in range(8):
        cases.append(
            AppCompatCase(
                case_id=f"service-{idx:02d}",
                app_id=f"service-app-{idx:02d}",
                app_class="service",
                tier="tier_service",
                passed=idx != 7,  # 7/8 pass => 0.875
            )
        )

    return cases


def _known_case_ids() -> Set[str]:
    return {case.case_id for case in _baseline_cases()}


def _normalize_case_ids(values: List[str]) -> Set[str]:
    return {value.strip() for value in values if value.strip()}


def _validate_case_ids(label: str, case_ids: Set[str]) -> None:
    unknown = sorted(case_ids - _known_case_ids())
    if unknown:
        raise ValueError(f"unknown case ids in {label}: {', '.join(unknown)}")


def _metric(seed: int, case_id: str, label: str, base: int, spread: int) -> int:
    digest = hashlib.sha256(f"{seed}|{case_id}|{label}".encode("utf-8")).hexdigest()
    return base + (int(digest[:8], 16) % spread)


def _apply_injections(
    cases: List[AppCompatCase],
    force_failures: Set[str],
    force_unsigned: Set[str],
    force_nondeterministic: Set[str],
    force_profile_mismatch: Set[str],
) -> List[AppCompatCase]:
    updated: List[AppCompatCase] = []
    for case in cases:
        candidate = case
        if case.case_id in force_failures:
            candidate = replace(candidate, passed=False)
        if case.case_id in force_unsigned:
            candidate = replace(candidate, signed=False)
        if case.case_id in force_nondeterministic:
            candidate = replace(candidate, deterministic=False)
        if case.case_id in force_profile_mismatch:
            candidate = replace(candidate, abi_profile="compat_profile_v2")
        updated.append(candidate)
    return updated


def run_matrix(
    seed: int,
    force_failures: Set[str] | None = None,
    force_unsigned: Set[str] | None = None,
    force_nondeterministic: Set[str] | None = None,
    force_profile_mismatch: Set[str] | None = None,
) -> Dict[str, object]:
    failure_injections = set() if force_failures is None else set(force_failures)
    unsigned_injections = set() if force_unsigned is None else set(force_unsigned)
    nondeterministic_injections = (
        set() if force_nondeterministic is None else set(force_nondeterministic)
    )
    profile_mismatch_injections = (
        set() if force_profile_mismatch is None else set(force_profile_mismatch)
    )

    cases = _apply_injections(
        _baseline_cases(),
        force_failures=failure_injections,
        force_unsigned=unsigned_injections,
        force_nondeterministic=nondeterministic_injections,
        force_profile_mismatch=profile_mismatch_injections,
    )

    class_totals = {
        class_name: {"eligible": 0, "passed": 0}
        for class_name in sorted(CLASS_THRESHOLDS.keys())
    }
    case_reports: List[Dict[str, object]] = []
    issues: List[Dict[str, object]] = []

    for case in sorted(cases, key=lambda item: item.case_id):
        expected = CLASS_THRESHOLDS.get(case.app_class)
        counted_for_threshold = True

        if expected is None:
            issues.append({"case_id": case.case_id, "reason": "unknown_class"})
            counted_for_threshold = False
        elif case.tier != expected["tier"]:
            issues.append(
                {
                    "case_id": case.case_id,
                    "reason": "tier_mismatch",
                    "expected_tier": expected["tier"],
                    "actual_tier": case.tier,
                }
            )
            counted_for_threshold = False
        elif not case.signed:
            issues.append({"case_id": case.case_id, "reason": "unsigned_artifact"})
            counted_for_threshold = False
        elif case.abi_profile != PROFILE_LABEL:
            issues.append({"case_id": case.case_id, "reason": "abi_profile_mismatch"})
            counted_for_threshold = False
        elif not case.deterministic:
            issues.append(
                {"case_id": case.case_id, "reason": "non_deterministic_result"}
            )
            counted_for_threshold = False

        if counted_for_threshold:
            bucket = class_totals[case.app_class]
            bucket["eligible"] += 1
            if case.passed:
                bucket["passed"] += 1

        case_reports.append(
            {
                "case_id": case.case_id,
                "app_id": case.app_id,
                "class": case.app_class,
                "tier": case.tier,
                "signed": case.signed,
                "deterministic": case.deterministic,
                "abi_profile": case.abi_profile,
                "passed": case.passed,
                "counted_for_threshold": counted_for_threshold,
                "metrics": {
                    "startup_ms": _metric(seed, case.case_id, "startup", base=12, spread=18),
                    "steady_rss_kib": _metric(
                        seed, case.case_id, "rss", base=1024, spread=4096
                    ),
                    "syscall_count": _metric(
                        seed, case.case_id, "syscalls", base=80, spread=320
                    ),
                },
            }
        )

    class_reports: Dict[str, Dict[str, object]] = {}
    thresholds_pass = True
    for class_name in sorted(CLASS_THRESHOLDS.keys()):
        threshold = CLASS_THRESHOLDS[class_name]
        stats = class_totals[class_name]
        eligible = int(stats["eligible"])
        passed = int(stats["passed"])
        pass_rate = (passed / eligible) if eligible else 0.0
        meets_threshold = (
            eligible >= int(threshold["min_cases"])
            and pass_rate >= float(threshold["min_pass_rate"])
        )
        thresholds_pass = thresholds_pass and meets_threshold

        class_reports[class_name] = {
            "tier": threshold["tier"],
            "eligible": eligible,
            "passed": passed,
            "pass_rate": pass_rate,
            "min_cases": int(threshold["min_cases"]),
            "min_pass_rate": float(threshold["min_pass_rate"]),
            "meets_threshold": meets_threshold,
        }

    issues_sorted = sorted(issues, key=lambda item: (str(item["reason"]), item["case_id"]))
    stable_payload = {
        "profile_id": PROFILE_ID,
        "tier_schema": TIER_SCHEMA,
        "seed": seed,
        "classes": class_reports,
        "issues": issues_sorted,
        "cases": [
            {
                "case_id": item["case_id"],
                "class": item["class"],
                "passed": item["passed"],
                "counted_for_threshold": item["counted_for_threshold"],
            }
            for item in case_reports
        ],
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    gate_pass = thresholds_pass and not issues_sorted
    return {
        "schema": REPORT_SCHEMA,
        "profile_id": PROFILE_ID,
        "tier_schema": TIER_SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "total_cases": len(case_reports),
        "classes": class_reports,
        "cases": case_reports,
        "issues": issues_sorted,
        "gate_pass": gate_pass,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20260309)
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a case id to fail",
    )
    parser.add_argument(
        "--inject-unsigned",
        action="append",
        default=[],
        help="force a case id to be treated as unsigned",
    )
    parser.add_argument(
        "--inject-nondeterministic",
        action="append",
        default=[],
        help="force a case id to be treated as non-deterministic",
    )
    parser.add_argument(
        "--inject-profile-mismatch",
        action="append",
        default=[],
        help="force a case id to use an ABI profile mismatch",
    )
    parser.add_argument("--out", default="out/app-compat-matrix-v3.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    injections = {
        "inject-failure": _normalize_case_ids(args.inject_failure),
        "inject-unsigned": _normalize_case_ids(args.inject_unsigned),
        "inject-nondeterministic": _normalize_case_ids(args.inject_nondeterministic),
        "inject-profile-mismatch": _normalize_case_ids(args.inject_profile_mismatch),
    }
    try:
        for label, ids in injections.items():
            _validate_case_ids(label, ids)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_matrix(
        seed=args.seed,
        force_failures=injections["inject-failure"],
        force_unsigned=injections["inject-unsigned"],
        force_nondeterministic=injections["inject-nondeterministic"],
        force_profile_mismatch=injections["inject-profile-mismatch"],
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"app-compat-matrix-report: {out_path}")
    print(f"issues: {len(report['issues'])}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
