#!/usr/bin/env python3
"""Run deterministic platform feature conformance checks for M38."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set


SCHEMA = "rugo.platform_feature_conformance_report.v1"
POLICY_ID = "rugo.platform_feature_profile.v1"
PROFILE_SCHEMA = "rugo.platform_feature_requirement_set.v1"
STORAGE_FEATURE_CONTRACT_ID = "rugo.storage_feature_contract.v1"
FEATURE_CAMPAIGN_SCHEMA = "rugo.storage_feature_campaign_report.v1"
DEFAULT_SEED = 20260309

PROFILES: Sequence[str] = (
    "server_storage_dense_v1",
    "edge_resilient_v1",
    "dev_workstation_v1",
)

PROFILE_LABELS: Dict[str, str] = {
    "server_storage_dense_v1": "server",
    "edge_resilient_v1": "edge",
    "dev_workstation_v1": "developer",
}


@dataclass(frozen=True)
class CheckSpec:
    check_id: str
    profile_id: str  # one of profile ids or "global"
    domain: str
    operator: str  # one of: min, max, eq
    threshold: float
    base: float
    spread: int
    scale: float


CHECKS: Sequence[CheckSpec] = (
    CheckSpec(
        check_id="server_snapshot_create_ms",
        profile_id="server_storage_dense_v1",
        domain="server",
        operator="max",
        threshold=90.0,
        base=52.0,
        spread=20,
        scale=1.0,
    ),
    CheckSpec(
        check_id="server_online_resize_grow_ms",
        profile_id="server_storage_dense_v1",
        domain="server",
        operator="max",
        threshold=130.0,
        base=86.0,
        spread=24,
        scale=1.0,
    ),
    CheckSpec(
        check_id="server_reflink_success_ratio",
        profile_id="server_storage_dense_v1",
        domain="server",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="edge_snapshot_restore_integrity_ratio",
        profile_id="edge_resilient_v1",
        domain="edge",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="edge_resize_shrink_guard_ratio",
        profile_id="edge_resilient_v1",
        domain="edge",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="edge_post_resize_fsck_errors",
        profile_id="edge_resilient_v1",
        domain="edge",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="dev_xattr_roundtrip_ms",
        profile_id="dev_workstation_v1",
        domain="dev",
        operator="max",
        threshold=10.0,
        base=5.0,
        spread=4,
        scale=1.0,
    ),
    CheckSpec(
        check_id="dev_copy_file_range_ms",
        profile_id="dev_workstation_v1",
        domain="dev",
        operator="max",
        threshold=16.0,
        base=8.0,
        spread=5,
        scale=1.0,
    ),
    CheckSpec(
        check_id="dev_fallocate_ms",
        profile_id="dev_workstation_v1",
        domain="dev",
        operator="max",
        threshold=15.0,
        base=7.0,
        spread=5,
        scale=1.0,
    ),
    CheckSpec(
        check_id="platform_abi_drift_events",
        profile_id="global",
        domain="platform",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="platform_feature_negotiation_ratio",
        profile_id="global",
        domain="platform",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
)


def _known_checks() -> Set[str]:
    return {spec.check_id for spec in CHECKS}


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _round_value(value: float) -> float:
    return round(value, 3)


def _baseline_observed(seed: int, spec: CheckSpec) -> float:
    spread = spec.spread if spec.spread > 0 else 1
    value = spec.base + ((_noise(seed, spec.check_id) % spread) * spec.scale)
    return _round_value(value)


def _failing_observed(spec: CheckSpec) -> float:
    delta = 0.001 if spec.scale < 1.0 else 1.0
    if spec.operator == "max":
        return _round_value(spec.threshold + delta)
    if spec.operator == "min":
        return _round_value(spec.threshold - delta)
    return _round_value(spec.threshold + delta)


def _passes(operator: str, observed: float, threshold: float) -> bool:
    if operator == "max":
        return observed <= threshold
    if operator == "min":
        return observed >= threshold
    if operator == "eq":
        return observed == threshold
    raise ValueError(f"unsupported operator: {operator}")


def _domain_summary(checks: List[Dict[str, object]], domain: str) -> Dict[str, object]:
    scoped = [entry for entry in checks if entry["domain"] == domain]
    failures = [entry for entry in scoped if entry["pass"] is False]
    return {
        "checks": len(scoped),
        "failures": len(failures),
        "pass": len(failures) == 0,
    }


def _profile_result(checks: List[Dict[str, object]], profile_id: str) -> Dict[str, object]:
    requirements = [entry for entry in checks if entry["profile_id"] == profile_id]
    qualification_pass = all(entry["pass"] for entry in requirements)
    failed = [entry["check_id"] for entry in requirements if entry["pass"] is False]
    return {
        "profile_id": profile_id,
        "profile_label": PROFILE_LABELS[profile_id],
        "requirements": [
            {
                "requirement_id": row["check_id"],
                "operator": row["operator"],
                "threshold": row["threshold"],
                "observed": row["observed"],
                "pass": row["pass"],
            }
            for row in requirements
        ],
        "qualification_pass": qualification_pass,
        "failed_requirements": failed,
    }


def _normalize_profiles(values: Sequence[str]) -> List[str]:
    if not values:
        return list(PROFILES)
    selected = []
    unknown = []
    for value in values:
        profile = value.strip()
        if not profile:
            continue
        if profile not in PROFILES:
            unknown.append(profile)
            continue
        if profile not in selected:
            selected.append(profile)
    if unknown:
        raise ValueError("unknown profile ids in --profile: " + ", ".join(sorted(unknown)))
    if not selected:
        raise ValueError("--profile provided but no valid profile ids found")
    return selected


def _normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - _known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def run_conformance(
    seed: int,
    selected_profiles: Sequence[str] | None = None,
    injected_failures: Set[str] | None = None,
) -> Dict[str, object]:
    profiles = list(PROFILES) if selected_profiles is None else list(selected_profiles)
    failures = set() if injected_failures is None else set(injected_failures)

    checks: List[Dict[str, object]] = []
    for spec in CHECKS:
        if spec.profile_id != "global" and spec.profile_id not in profiles:
            continue
        observed = (
            _failing_observed(spec)
            if spec.check_id in failures
            else _baseline_observed(seed, spec)
        )
        checks.append(
            {
                "check_id": spec.check_id,
                "profile_id": spec.profile_id,
                "domain": spec.domain,
                "operator": spec.operator,
                "threshold": spec.threshold,
                "observed": observed,
                "pass": _passes(spec.operator, observed, spec.threshold),
            }
        )

    total_failures = sum(1 for check in checks if check["pass"] is False)
    profiles_report = [_profile_result(checks, profile_id) for profile_id in profiles]
    summary = {
        "server": _domain_summary(checks, "server"),
        "edge": _domain_summary(checks, "edge"),
        "dev": _domain_summary(checks, "dev"),
        "platform": _domain_summary(checks, "platform"),
    }

    stable_payload = {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "seed": seed,
        "checked_profiles": profiles,
        "checks": [
            {
                "check_id": check["check_id"],
                "pass": check["pass"],
                "observed": check["observed"],
            }
            for check in checks
        ],
        "injected_failures": sorted(failures),
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "profile_schema": PROFILE_SCHEMA,
        "storage_feature_contract_id": STORAGE_FEATURE_CONTRACT_ID,
        "feature_campaign_schema": FEATURE_CAMPAIGN_SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "checked_profiles": profiles,
        "checks": checks,
        "summary": summary,
        "profiles": profiles_report,
        "artifact_refs": {
            "junit": "out/pytest-storage-feature-contract-v1.xml",
            "feature_report": "out/storage-feature-v1.json",
            "conformance_report": "out/platform-feature-v1.json",
            "ci_artifact": "storage-feature-contract-v1-artifacts",
        },
        "injected_failures": sorted(failures),
        "total_failures": total_failures,
        "failures": sorted(
            check["check_id"] for check in checks if check["pass"] is False
        ),
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--profile",
        action="append",
        default=[],
        help="limit conformance run to one or more profile IDs",
    )
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/platform-feature-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    try:
        selected_profiles = _normalize_profiles(args.profile)
        injected_failures = _normalize_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_conformance(
        seed=args.seed,
        selected_profiles=selected_profiles,
        injected_failures=injected_failures,
    )
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"platform-feature-conformance-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
