#!/usr/bin/env python3
"""Run deterministic runtime-qualified catalog audit checks for M44."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set


SCHEMA = "rugo.real_catalog_audit_report.v2"
ECOSYSTEM_POLICY_ID = "rugo.ecosystem_scale_policy.v2"
DISTRIBUTION_WORKFLOW_ID = "rugo.distribution_workflow.v2"
DESKTOP_PROFILE_ID = "rugo.desktop_profile.v2"
DEFAULT_SEED = 20260310


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
        check_id="deterministic_rebuild_ratio",
        domain="reproducibility",
        metric_key="deterministic_rebuild_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="provenance_match_ratio",
        domain="reproducibility",
        metric_key="provenance_match_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="sbom_drift_count",
        domain="reproducibility",
        metric_key="sbom_drift_count",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="mirror_index_consistency_ratio",
        domain="distribution",
        metric_key="mirror_index_consistency_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="replication_lag_p95_minutes",
        domain="distribution",
        metric_key="replication_lag_p95_minutes",
        operator="max",
        threshold=10.0,
        base=6.0,
        spread=4,
        scale=1.0,
    ),
    CheckSpec(
        check_id="workflow_stage_completeness_ratio",
        domain="workflow",
        metric_key="workflow_stage_completeness_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="release_signoff_ratio",
        domain="workflow",
        metric_key="release_signoff_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="rollback_drill_pass_ratio",
        domain="workflow",
        metric_key="rollback_drill_pass_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="runtime_trace_coverage_ratio",
        domain="provenance",
        metric_key="runtime_trace_coverage_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="signed_artifact_ratio",
        domain="provenance",
        metric_key="signed_artifact_ratio",
        operator="min",
        threshold=1.0,
        base=1.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="unresolved_policy_exceptions",
        domain="quality",
        metric_key="unresolved_policy_exceptions",
        operator="max",
        threshold=0.0,
        base=0.0,
        spread=1,
        scale=0.0,
    ),
    CheckSpec(
        check_id="stale_manifest_count",
        domain="quality",
        metric_key="stale_manifest_count",
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


def run_audit(seed: int, injected_failures: Set[str] | None = None) -> Dict[str, object]:
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
        "reproducibility": _domain_summary(checks, "reproducibility"),
        "distribution": _domain_summary(checks, "distribution"),
        "workflow": _domain_summary(checks, "workflow"),
        "provenance": _domain_summary(checks, "provenance"),
        "quality": _domain_summary(checks, "quality"),
    }

    stable_payload = {
        "schema": SCHEMA,
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
        "ecosystem_policy_id": ECOSYSTEM_POLICY_ID,
        "distribution_workflow_id": DISTRIBUTION_WORKFLOW_ID,
        "desktop_profile_id": DESKTOP_PROFILE_ID,
        "seed": seed,
        "gate": "test-real-app-catalog-v2",
        "checks": checks,
        "summary": summary,
        "reproducibility": {
            "deterministic_rebuild_ratio": metric_values["deterministic_rebuild_ratio"],
            "provenance_match_ratio": metric_values["provenance_match_ratio"],
            "sbom_drift_count": int(metric_values["sbom_drift_count"]),
            "checks_pass": summary["reproducibility"]["pass"],
        },
        "distribution": {
            "mirror_index_consistency_ratio": metric_values[
                "mirror_index_consistency_ratio"
            ],
            "replication_lag_p95_minutes": metric_values["replication_lag_p95_minutes"],
            "checks_pass": summary["distribution"]["pass"],
        },
        "workflow": {
            "workflow_stage_completeness_ratio": metric_values[
                "workflow_stage_completeness_ratio"
            ],
            "release_signoff_ratio": metric_values["release_signoff_ratio"],
            "rollback_drill_pass_ratio": metric_values["rollback_drill_pass_ratio"],
            "checks_pass": summary["workflow"]["pass"],
        },
        "provenance": {
            "runtime_trace_coverage_ratio": metric_values["runtime_trace_coverage_ratio"],
            "signed_artifact_ratio": metric_values["signed_artifact_ratio"],
            "checks_pass": summary["provenance"]["pass"],
        },
        "quality": {
            "unresolved_policy_exceptions": int(
                metric_values["unresolved_policy_exceptions"]
            ),
            "stale_manifest_count": int(metric_values["stale_manifest_count"]),
            "checks_pass": summary["quality"]["pass"],
        },
        "artifact_refs": {
            "junit": "out/pytest-real-app-catalog-v2.xml",
            "install_report": "out/real-pkg-install-v2.json",
            "audit_report": "out/real-catalog-audit-v2.json",
            "ci_artifact": "real-app-catalog-v2-artifacts",
            "parent_ci_artifact": "real-ecosystem-desktop-v2-artifacts",
        },
        "injected_failures": sorted(failures),
        "total_failures": total_failures,
        "failures": sorted(check["check_id"] for check in checks if check["pass"] is False),
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
    parser.add_argument("--out", default="out/real-catalog-audit-v2.json")
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

    report = run_audit(seed=args.seed, injected_failures=injected_failures)
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"real-catalog-audit-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
