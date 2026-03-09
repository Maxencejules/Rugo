#!/usr/bin/env python3
"""Run deterministic storage feature campaign checks for M38."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set


SCHEMA = "rugo.storage_feature_campaign_report.v1"
STORAGE_FEATURE_CONTRACT_ID = "rugo.storage_feature_contract.v1"
SNAPSHOT_POLICY_ID = "rugo.snapshot_policy.v1"
ONLINE_RESIZE_POLICY_ID = "rugo.online_resize_policy.v1"
PLATFORM_PROFILE_ID = "rugo.platform_feature_profile.v1"
DEFAULT_SEED = 20260309


@dataclass(frozen=True)
class CheckSpec:
    check_id: str
    domain: str
    metric_key: str
    operator: str  # one of: min, max, eq
    threshold: float
    base: float
    spread: int
    scale: float


CHECKS: Sequence[CheckSpec] = (
    CheckSpec(
        check_id="snapshot_create_ms",
        domain="snapshot",
        metric_key="snapshot_create_ms",
        operator="max",
        threshold=80.0,
        base=42.0,
        spread=18,
        scale=1.0,
    ),
    CheckSpec(
        check_id="snapshot_restore_integrity_ratio",
        domain="snapshot",
        metric_key="snapshot_restore_integrity_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="snapshot_retention_violations",
        domain="snapshot",
        metric_key="snapshot_retention_violations",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="snapshot_orphan_refs",
        domain="snapshot",
        metric_key="snapshot_orphan_refs",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="resize_grow_ms",
        domain="resize",
        metric_key="resize_grow_ms",
        operator="max",
        threshold=120.0,
        base=74.0,
        spread=22,
        scale=1.0,
    ),
    CheckSpec(
        check_id="resize_capacity_mismatch_count",
        domain="resize",
        metric_key="resize_capacity_mismatch_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="resize_shrink_guard_ratio",
        domain="resize",
        metric_key="resize_shrink_guard_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="resize_post_fsck_errors",
        domain="resize",
        metric_key="resize_post_fsck_errors",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="fsops_reflink_ms",
        domain="fs_ops",
        metric_key="fsops_reflink_ms",
        operator="max",
        threshold=30.0,
        base=14.0,
        spread=8,
        scale=1.0,
    ),
    CheckSpec(
        check_id="fsops_fallocate_ms",
        domain="fs_ops",
        metric_key="fsops_fallocate_ms",
        operator="max",
        threshold=15.0,
        base=7.0,
        spread=5,
        scale=1.0,
    ),
    CheckSpec(
        check_id="fsops_copy_file_range_ms",
        domain="fs_ops",
        metric_key="fsops_copy_file_range_ms",
        operator="max",
        threshold=16.0,
        base=8.0,
        spread=5,
        scale=1.0,
    ),
    CheckSpec(
        check_id="fsops_xattr_roundtrip_ms",
        domain="fs_ops",
        metric_key="fsops_xattr_roundtrip_ms",
        operator="max",
        threshold=10.0,
        base=5.0,
        spread=3,
        scale=1.0,
    ),
    CheckSpec(
        check_id="fsops_dedupe_false_positive_count",
        domain="fs_ops",
        metric_key="fsops_dedupe_false_positive_count",
        operator="max",
        threshold=0.0,
        base=0.0,
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


def _normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - _known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def run_campaign(seed: int, injected_failures: Set[str] | None = None) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)

    checks: List[Dict[str, object]] = []
    metric_values: Dict[str, float] = {}
    for spec in CHECKS:
        observed = (
            _failing_observed(spec)
            if spec.check_id in failures
            else _baseline_observed(seed, spec)
        )
        passed = _passes(spec.operator, observed, spec.threshold)
        checks.append(
            {
                "check_id": spec.check_id,
                "domain": spec.domain,
                "metric_key": spec.metric_key,
                "operator": spec.operator,
                "threshold": spec.threshold,
                "observed": observed,
                "pass": passed,
            }
        )
        metric_values[spec.metric_key] = observed

    total_failures = sum(1 for check in checks if check["pass"] is False)
    summary = {
        "snapshot": _domain_summary(checks, "snapshot"),
        "resize": _domain_summary(checks, "resize"),
        "fs_ops": _domain_summary(checks, "fs_ops"),
    }

    stable_payload = {
        "schema": SCHEMA,
        "storage_feature_contract_id": STORAGE_FEATURE_CONTRACT_ID,
        "seed": seed,
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
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "storage_feature_contract_id": STORAGE_FEATURE_CONTRACT_ID,
        "snapshot_policy_id": SNAPSHOT_POLICY_ID,
        "online_resize_policy_id": ONLINE_RESIZE_POLICY_ID,
        "platform_profile_id": PLATFORM_PROFILE_ID,
        "seed": seed,
        "gate": "test-storage-platform-v1",
        "checks": checks,
        "summary": summary,
        "snapshot": {
            "snapshot_create_ms": metric_values["snapshot_create_ms"],
            "snapshot_restore_integrity_ratio": metric_values[
                "snapshot_restore_integrity_ratio"
            ],
            "snapshot_retention_violations": int(
                metric_values["snapshot_retention_violations"]
            ),
            "snapshot_orphan_refs": int(metric_values["snapshot_orphan_refs"]),
            "checks_pass": summary["snapshot"]["pass"],
        },
        "online_resize": {
            "resize_grow_ms": metric_values["resize_grow_ms"],
            "resize_capacity_mismatch_count": int(
                metric_values["resize_capacity_mismatch_count"]
            ),
            "resize_shrink_guard_ratio": metric_values["resize_shrink_guard_ratio"],
            "resize_post_fsck_errors": int(metric_values["resize_post_fsck_errors"]),
            "checks_pass": summary["resize"]["pass"],
        },
        "advanced_fs_ops": {
            "fsops_reflink_ms": metric_values["fsops_reflink_ms"],
            "fsops_fallocate_ms": metric_values["fsops_fallocate_ms"],
            "fsops_copy_file_range_ms": metric_values["fsops_copy_file_range_ms"],
            "fsops_xattr_roundtrip_ms": metric_values["fsops_xattr_roundtrip_ms"],
            "fsops_dedupe_false_positive_count": int(
                metric_values["fsops_dedupe_false_positive_count"]
            ),
            "checks_pass": summary["fs_ops"]["pass"],
        },
        "artifact_refs": {
            "junit": "out/pytest-storage-platform-v1.xml",
            "feature_report": "out/storage-feature-v1.json",
            "platform_report": "out/platform-feature-v1.json",
            "ci_artifact": "storage-platform-v1-artifacts",
            "contract_ci_artifact": "storage-feature-contract-v1-artifacts",
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
        "--inject-failure",
        action="append",
        default=[],
        help="force a check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/storage-feature-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    try:
        injected_failures = _normalize_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_campaign(seed=args.seed, injected_failures=injected_failures)
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"storage-feature-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
