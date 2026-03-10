#!/usr/bin/env python3
"""Collect deterministic bare-metal promotion evidence for M46."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_baremetal_io_baseline_v1 as baseline


SCHEMA = "rugo.hw_baremetal_promotion_report.v2"
PROFILE_ID = "rugo.baremetal_io_profile.v1"
BASELINE_SCHEMA = "rugo.baremetal_io_baseline.v1"
DEFAULT_SEED = 20260310
DEFAULT_CAMPAIGN_RUNS = 12
DEFAULT_REQUIRED_CONSECUTIVE_GREEN = 12
DEFAULT_MIN_PASS_RATE = 0.98
REQUIRED_ARTIFACT_KEYS = (
    "baseline_junit",
    "baseline_report",
    "desktop_report",
    "recovery_report",
    "promotion_report",
)


def _normalize_strings(values: Sequence[str]) -> Set[str]:
    return {value.strip() for value in values if value.strip()}


def _normalize_run_failures(values: Sequence[str], campaign_runs: int) -> Set[int]:
    runs: Set[int] = set()
    for value in values:
        stripped = value.strip()
        if not stripped:
            continue
        try:
            run_id = int(stripped)
        except ValueError as exc:
            raise ValueError(f"invalid run id in --inject-run-failure: {stripped}") from exc
        if run_id < 1 or run_id > campaign_runs:
            raise ValueError(
                f"run id out of range in --inject-run-failure: {run_id} "
                f"(valid range: 1..{campaign_runs})"
            )
        runs.add(run_id)
    return runs


def _validate_missing_artifacts(missing: Set[str]) -> None:
    unknown = sorted(missing - set(REQUIRED_ARTIFACT_KEYS))
    if unknown:
        raise ValueError(
            "unknown artifacts in --inject-missing-artifact: " + ", ".join(unknown)
        )


def _trailing_consecutive_green(run_results: List[Dict[str, object]]) -> int:
    trailing = 0
    for row in reversed(run_results):
        if row["gate_pass"]:
            trailing += 1
            continue
        break
    return trailing


def run_promotion(
    seed: int,
    campaign_runs: int,
    required_consecutive_green: int,
    min_pass_rate: float,
    injected_run_failures: Set[int] | None = None,
    missing_artifacts: Set[str] | None = None,
) -> Dict[str, object]:
    fail_runs = set() if injected_run_failures is None else set(injected_run_failures)
    missing = set() if missing_artifacts is None else set(missing_artifacts)

    run_results: List[Dict[str, object]] = []
    profile_counters: Dict[str, Dict[str, int]] = {}
    desktop_bridge_green = True
    recovery_bridge_green = True
    last_baseline_report: Dict[str, object] | None = None

    for run_id in range(1, campaign_runs + 1):
        run_seed = seed + run_id
        injected = {"e1000e_udp_echo"} if run_id in fail_runs else set()
        report = baseline.run_baseline(
            seed=run_seed,
            injected_failures=injected,
            max_failures=0,
        )
        last_baseline_report = report

        desktop_bridge_green = desktop_bridge_green and bool(
            report["desktop_input_checks"]["input_checks_pass"]
        )
        recovery_bridge_green = recovery_bridge_green and bool(
            report["install_recovery_checks"]["recovery_gate_pass"]
        )

        for profile in report["tier2_profiles"]:
            counters = profile_counters.setdefault(
                profile["profile_id"],
                {"passes": 0, "runs": 0, "manual_exception_required": 0},
            )
            counters["runs"] += 1
            counters["manual_exception_required"] = int(
                profile["manual_exception_required"]
            )
            if profile["status"] == "pass":
                counters["passes"] += 1

        run_results.append(
            {
                "run_id": run_id,
                "seed": run_seed,
                "gate_pass": report["gate_pass"],
                "total_failures": report["total_failures"],
                "digest": report["digest"],
            }
        )

    green_runs = sum(1 for row in run_results if row["gate_pass"])
    failed_runs = campaign_runs - green_runs
    pass_rate = round((green_runs / campaign_runs) if campaign_runs else 0.0, 3)
    trailing_green = _trailing_consecutive_green(run_results)

    artifact_refs = {
        "baseline_junit": "out/pytest-baremetal-io-v1.xml",
        "baseline_report": "out/baremetal-io-v1.json",
        "desktop_report": "out/desktop-smoke-v1.json",
        "recovery_report": "out/recovery-drill-v3.json",
        "promotion_report": "out/hw-promotion-v2.json",
        "ci_artifact": "baremetal-io-v1-artifacts",
        "usb_ci_artifact": "usb-input-removable-v1-artifacts",
    }
    available_artifacts = sorted(
        key for key in REQUIRED_ARTIFACT_KEYS if key not in missing
    )

    profile_results = []
    for profile_id, counters in sorted(profile_counters.items()):
        profile_pass_rate = round(
            (counters["passes"] / counters["runs"]) if counters["runs"] else 0.0,
            3,
        )
        profile_results.append(
            {
                "profile_id": profile_id,
                "board_class": "tier2",
                "required_for_baseline": True,
                "campaign_runs": counters["runs"],
                "pass_rate": profile_pass_rate,
                "manual_exception_required": bool(
                    counters["manual_exception_required"]
                ),
            }
        )

    tier2_floor_met = any(
        profile["pass_rate"] >= min_pass_rate
        and profile["manual_exception_required"] is False
        for profile in profile_results
    )

    policy_checks = [
        {
            "check_id": "campaign_length",
            "operator": "min",
            "threshold": required_consecutive_green,
            "observed": campaign_runs,
            "pass": campaign_runs >= required_consecutive_green,
        },
        {
            "check_id": "campaign_pass_rate",
            "operator": "min",
            "threshold": min_pass_rate,
            "observed": pass_rate,
            "pass": pass_rate >= min_pass_rate,
        },
        {
            "check_id": "trailing_consecutive_green",
            "operator": "min",
            "threshold": required_consecutive_green,
            "observed": trailing_green,
            "pass": trailing_green >= required_consecutive_green,
        },
        {
            "check_id": "artifact_bundle_complete",
            "operator": "eq",
            "threshold": True,
            "observed": len(missing) == 0,
            "pass": len(missing) == 0,
        },
        {
            "check_id": "desktop_usb_bridge_green",
            "operator": "eq",
            "threshold": True,
            "observed": desktop_bridge_green,
            "pass": desktop_bridge_green,
        },
        {
            "check_id": "recovery_bridge_green",
            "operator": "eq",
            "threshold": True,
            "observed": recovery_bridge_green,
            "pass": recovery_bridge_green,
        },
        {
            "check_id": "tier2_profile_floor",
            "operator": "eq",
            "threshold": True,
            "observed": tier2_floor_met,
            "pass": tier2_floor_met,
        },
    ]

    failures = sorted(
        check["check_id"] for check in policy_checks if check["pass"] is False
    )
    total_failures = len(failures)
    gate_pass = total_failures == 0

    stable_payload = {
        "schema": SCHEMA,
        "profile_id": PROFILE_ID,
        "seed": seed,
        "campaign_runs": campaign_runs,
        "required_consecutive_green": required_consecutive_green,
        "min_pass_rate": min_pass_rate,
        "run_results": run_results,
        "missing_artifacts": sorted(missing),
        "policy_checks": [
            {"check_id": check["check_id"], "pass": check["pass"]}
            for check in policy_checks
        ],
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "profile_id": PROFILE_ID,
        "baseline_schema_id": BASELINE_SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "campaign_runs": campaign_runs,
        "required_consecutive_green": required_consecutive_green,
        "min_pass_rate": min_pass_rate,
        "run_results": run_results,
        "summary": {
            "green_runs": green_runs,
            "failed_runs": failed_runs,
            "pass_rate": pass_rate,
            "trailing_consecutive_green": trailing_green,
        },
        "profile_results": profile_results,
        "artifact_refs": artifact_refs,
        "available_artifacts": available_artifacts,
        "missing_artifacts": sorted(missing),
        "policy_checks": policy_checks,
        "desktop_usb_bridge_green": desktop_bridge_green,
        "recovery_bridge_green": recovery_bridge_green,
        "tier2_floor_met": tier2_floor_met,
        "latest_baseline_digest": (
            None if last_baseline_report is None else last_baseline_report["digest"]
        ),
        "injected_run_failures": sorted(fail_runs),
        "total_failures": total_failures,
        "failures": failures,
        "gate_pass": gate_pass,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument("--campaign-runs", type=int, default=DEFAULT_CAMPAIGN_RUNS)
    parser.add_argument(
        "--required-consecutive-green",
        type=int,
        default=DEFAULT_REQUIRED_CONSECUTIVE_GREEN,
    )
    parser.add_argument("--min-pass-rate", type=float, default=DEFAULT_MIN_PASS_RATE)
    parser.add_argument(
        "--inject-run-failure",
        action="append",
        default=[],
        help="force a campaign run to fail by run id (1-indexed)",
    )
    parser.add_argument(
        "--inject-missing-artifact",
        action="append",
        default=[],
        help="remove required artifact key from bundle",
    )
    parser.add_argument("--out", default="out/hw-promotion-v2.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    if args.campaign_runs <= 0:
        print("error: campaign-runs must be > 0")
        return 2
    if args.required_consecutive_green <= 0:
        print("error: required-consecutive-green must be > 0")
        return 2
    if args.required_consecutive_green > args.campaign_runs:
        print("error: required-consecutive-green must be <= campaign-runs")
        return 2
    if args.min_pass_rate < 0.0 or args.min_pass_rate > 1.0:
        print("error: min-pass-rate must be within [0, 1]")
        return 2

    try:
        run_failures = _normalize_run_failures(
            args.inject_run_failure,
            args.campaign_runs,
        )
        missing = _normalize_strings(args.inject_missing_artifact)
        _validate_missing_artifacts(missing)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_promotion(
        seed=args.seed,
        campaign_runs=args.campaign_runs,
        required_consecutive_green=args.required_consecutive_green,
        min_pass_rate=args.min_pass_rate,
        injected_run_failures=run_failures,
        missing_artifacts=missing,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"hw-promotion-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
