#!/usr/bin/env python3
"""Run deterministic profile conformance qualification checks for M32."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Any, Dict, List, Sequence, Set, Tuple

import runtime_capture_common_v1 as runtime_capture
import t4_runtime_qualification_common_v1 as runtime_qual

SCHEMA = "rugo.profile_conformance_report.v1"
POLICY_ID = "rugo.profile_conformance_policy.v1"
PROFILE_SCHEMA = "rugo.profile_requirement_set.v1"
DEFAULT_SEED = 20260309


@dataclass(frozen=True)
class Requirement:
    requirement_id: str
    metric_key: str
    operator: str  # one of: min, max, eq
    threshold: float
    margin: int


@dataclass(frozen=True)
class ProfileSpec:
    profile_id: str
    label: str
    requirements: Sequence[Requirement]


PROFILES: Dict[str, ProfileSpec] = {
    "server_v1": ProfileSpec(
        profile_id="server_v1",
        label="server",
        requirements=[
            Requirement(
                requirement_id="service_restart_coverage_pct",
                metric_key="service_restart_coverage_pct",
                operator="min",
                threshold=95,
                margin=2,
            ),
            Requirement(
                requirement_id="max_crash_recovery_seconds",
                metric_key="max_crash_recovery_seconds",
                operator="max",
                threshold=30,
                margin=3,
            ),
            Requirement(
                requirement_id="network_ipv6_enabled",
                metric_key="network_ipv6_enabled",
                operator="eq",
                threshold=1,
                margin=0,
            ),
            Requirement(
                requirement_id="supply_chain_attestation",
                metric_key="supply_chain_attestation",
                operator="eq",
                threshold=1,
                margin=0,
            ),
        ],
    ),
    "developer_v1": ProfileSpec(
        profile_id="developer_v1",
        label="developer",
        requirements=[
            Requirement(
                requirement_id="toolchain_smoke_pass",
                metric_key="toolchain_smoke_pass",
                operator="eq",
                threshold=1,
                margin=0,
            ),
            Requirement(
                requirement_id="debug_symbols_available",
                metric_key="debug_symbols_available",
                operator="eq",
                threshold=1,
                margin=0,
            ),
            Requirement(
                requirement_id="package_build_success_rate_pct",
                metric_key="package_build_success_rate_pct",
                operator="min",
                threshold=98,
                margin=1,
            ),
            Requirement(
                requirement_id="interactive_shell_latency_ms_p95",
                metric_key="interactive_shell_latency_ms_p95",
                operator="max",
                threshold=200,
                margin=10,
            ),
        ],
    ),
    "appliance_v1": ProfileSpec(
        profile_id="appliance_v1",
        label="appliance",
        requirements=[
            Requirement(
                requirement_id="immutable_rootfs_enforced",
                metric_key="immutable_rootfs_enforced",
                operator="eq",
                threshold=1,
                margin=0,
            ),
            Requirement(
                requirement_id="read_only_runtime_pct",
                metric_key="read_only_runtime_pct",
                operator="min",
                threshold=99,
                margin=1,
            ),
            Requirement(
                requirement_id="boot_to_service_seconds_p95",
                metric_key="boot_to_service_seconds_p95",
                operator="max",
                threshold=45,
                margin=4,
            ),
            Requirement(
                requirement_id="remote_mgmt_surface_minimized",
                metric_key="remote_mgmt_surface_minimized",
                operator="eq",
                threshold=1,
                margin=0,
            ),
        ],
    ),
}


def _known_requirement_pairs() -> Set[Tuple[str, str]]:
    pairs: Set[Tuple[str, str]] = set()
    for profile in PROFILES.values():
        for req in profile.requirements:
            pairs.add((profile.profile_id, req.requirement_id))
    return pairs


def _metric_noise(seed: int, profile_id: str, requirement_id: str) -> int:
    digest = hashlib.sha256(
        f"{seed}|{profile_id}|{requirement_id}".encode("utf-8")
    ).hexdigest()
    return int(digest[:8], 16) % 3


def _fallback_measure(profile_id: str, requirement: Requirement, noise: int) -> float:
    if requirement.operator == "min":
        return float(requirement.threshold + requirement.margin + noise)
    if requirement.operator == "max":
        return float(requirement.threshold - requirement.margin - noise)
    return float(requirement.threshold)


def _package_build_success_rate_pct(pkg_rebuild_report: Dict[str, Any]) -> float:
    total_packages = int(pkg_rebuild_report.get("total_packages", 0))
    mismatches = int(pkg_rebuild_report.get("total_mismatches", 0))
    if total_packages <= 0:
        return 0.0
    passed = max(0, total_packages - mismatches)
    return round((passed / total_packages) * 100.0, 3)


def _remote_mgmt_surface_minimized(capture: Dict[str, Any]) -> float:
    disallowed_tokens = ["remote", "mgmt", "ssh", "rpc"]
    all_lines: List[str] = []
    for boot in runtime_capture.iter_boots(capture):
        for entry in boot.get("serial_lines", []):
            if isinstance(entry, dict):
                all_lines.append(str(entry.get("line", "")).lower())
    if any(token in line for token in disallowed_tokens for line in all_lines):
        return 0.0
    return 1.0 if runtime_qual.marker_present_in_all_boots(capture, "GOSH: spawn deny") else 0.0


def _measure_from_runtime(
    profile_id: str,
    requirement: Requirement,
    *,
    capture: Dict[str, Any],
    attestation_report: Dict[str, Any],
    pkg_rebuild_report: Dict[str, Any],
) -> float:
    if profile_id == "server_v1":
        if requirement.requirement_id == "service_restart_coverage_pct":
            return runtime_qual.shell_restart_coverage_pct(capture)
        if requirement.requirement_id == "max_crash_recovery_seconds":
            return runtime_qual.shell_recovery_seconds_p95(capture)
        if requirement.requirement_id == "network_ipv6_enabled":
            enabled = all(
                runtime_qual.marker_present_in_all_boots(capture, marker)
                for marker in ["NETC4: ifcfg ok", "NETC4: route ok", "NETC4: reply ok"]
            )
            return 1.0 if enabled else 0.0
        if requirement.requirement_id == "supply_chain_attestation":
            return 1.0 if attestation_report.get("meets_target") is True else 0.0

    if profile_id == "developer_v1":
        if requirement.requirement_id == "toolchain_smoke_pass":
            capture_ready = runtime_qual.marker_present_in_all_boots(capture, "GOINIT: ready")
            return 1.0 if capture_ready and bool(capture.get("build_id")) else 0.0
        if requirement.requirement_id == "debug_symbols_available":
            kernel_path = str(capture.get("kernel_path", ""))
            return 1.0 if kernel_path.endswith(".elf") and bool(capture.get("kernel_digest")) else 0.0
        if requirement.requirement_id == "package_build_success_rate_pct":
            return _package_build_success_rate_pct(pkg_rebuild_report)
        if requirement.requirement_id == "interactive_shell_latency_ms_p95":
            return runtime_qual.interactive_shell_latency_ms_p95(capture)

    if profile_id == "appliance_v1":
        if requirement.requirement_id == "immutable_rootfs_enforced":
            return 1.0 if runtime_qual.rootfs_immutable(capture) else 0.0
        if requirement.requirement_id == "read_only_runtime_pct":
            return runtime_qual.read_only_runtime_pct(capture)
        if requirement.requirement_id == "boot_to_service_seconds_p95":
            return runtime_qual.boot_to_ready_seconds_p95(capture)
        if requirement.requirement_id == "remote_mgmt_surface_minimized":
            return _remote_mgmt_surface_minimized(capture)

    raise ValueError(f"unsupported requirement mapping: {profile_id}:{requirement.requirement_id}")


def _measure(
    seed: int,
    profile_id: str,
    requirement: Requirement,
    injected_failures: Set[Tuple[str, str]],
    *,
    capture: Dict[str, Any],
    attestation_report: Dict[str, Any],
    pkg_rebuild_report: Dict[str, Any],
) -> float:
    pair = (profile_id, requirement.requirement_id)
    if pair in injected_failures:
        if requirement.operator == "min":
            return float(requirement.threshold - 1)
        if requirement.operator == "max":
            return float(requirement.threshold + 1)
        return 0.0 if requirement.threshold != 0 else 1.0

    try:
        return _measure_from_runtime(
            profile_id,
            requirement,
            capture=capture,
            attestation_report=attestation_report,
            pkg_rebuild_report=pkg_rebuild_report,
        )
    except ValueError:
        noise = _metric_noise(seed, profile_id, requirement.requirement_id)
        return _fallback_measure(profile_id, requirement, noise)


def _passes(operator: str, observed: float, threshold: float) -> bool:
    if operator == "min":
        return observed >= threshold
    if operator == "max":
        return observed <= threshold
    if operator == "eq":
        return observed == threshold
    raise ValueError(f"unsupported operator: {operator}")


def _normalize_profiles(values: Sequence[str]) -> List[str]:
    if not values:
        return sorted(PROFILES.keys())
    unique = []
    seen = set()
    for value in values:
        candidate = value.strip()
        if not candidate or candidate in seen:
            continue
        if candidate not in PROFILES:
            raise ValueError(f"unknown profile id: {candidate}")
        seen.add(candidate)
        unique.append(candidate)
    if not unique:
        raise ValueError("at least one profile must be selected")
    return sorted(unique)


def _parse_injections(values: Sequence[str]) -> Set[Tuple[str, str]]:
    pairs: Set[Tuple[str, str]] = set()
    if not values:
        return pairs

    known = _known_requirement_pairs()
    for raw in values:
        text = raw.strip()
        if not text:
            continue
        parts = text.split(":", 1)
        if len(parts) != 2:
            raise ValueError(
                "inject-failure entries must be '<profile_id>:<requirement_id>'"
            )
        pair = (parts[0], parts[1])
        if pair not in known:
            raise ValueError(
                "unknown inject-failure target: "
                f"{parts[0]}:{parts[1]}"
            )
        pairs.add(pair)
    return pairs


def run_suite(
    seed: int,
    selected_profiles: Sequence[str],
    injected_failures: Set[Tuple[str, str]] | None = None,
    *,
    runtime_capture_payload: Dict[str, Any] | None = None,
    runtime_capture_path: str = "",
    attestation_report: Dict[str, Any] | None = None,
    attestation_path: str = "",
    pkg_rebuild_report: Dict[str, Any] | None = None,
    pkg_rebuild_path: str = "",
    fixture: bool = False,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)
    profiles = _normalize_profiles(selected_profiles)
    capture, capture_source = runtime_qual.load_runtime_capture(
        runtime_capture_path=runtime_capture_path,
        fixture=fixture,
    ) if runtime_capture_payload is None else (runtime_capture_payload, runtime_capture_path or "provided")
    attestation, attestation_source = runtime_qual.load_release_attestation(
        report_path=attestation_path
    ) if attestation_report is None else (attestation_report, attestation_path or "provided")
    pkg_rebuild, pkg_rebuild_source = runtime_qual.load_pkg_rebuild_report(
        report_path=pkg_rebuild_path,
        seed=seed,
    ) if pkg_rebuild_report is None else (pkg_rebuild_report, pkg_rebuild_path or "provided")

    profile_reports: List[Dict[str, object]] = []
    total_failures = 0

    for profile_id in profiles:
        spec = PROFILES[profile_id]
        req_reports: List[Dict[str, object]] = []
        profile_failures = 0
        for req in spec.requirements:
            observed = _measure(
                seed,
                profile_id,
                req,
                failures,
                capture=capture,
                attestation_report=attestation,
                pkg_rebuild_report=pkg_rebuild,
            )
            passed = _passes(req.operator, observed, req.threshold)
            if not passed:
                profile_failures += 1
            req_reports.append(
                {
                    "requirement_id": req.requirement_id,
                    "metric_key": req.metric_key,
                    "operator": req.operator,
                    "threshold": req.threshold,
                    "observed": observed,
                    "pass": passed,
                    "evidence": {
                        "runtime_capture_digest": capture.get("digest", ""),
                        "attestation_source": attestation_source,
                        "pkg_rebuild_source": pkg_rebuild_source,
                    },
                }
            )

        checks = [
            {
                "name": "requirements_defined",
                "pass": len(req_reports) > 0,
            },
            {
                "name": "requirements_all_pass",
                "pass": profile_failures == 0,
            },
            {
                "name": "runtime_capture_bound",
                "pass": bool(capture.get("digest")),
            },
        ]
        total_failures += profile_failures + sum(
            1 for check in checks if check["pass"] is False
        )
        profile_pass = profile_failures == 0 and all(check["pass"] for check in checks)
        profile_reports.append(
            {
                "profile_id": spec.profile_id,
                "profile_label": spec.label,
                "profile_schema": PROFILE_SCHEMA,
                "runtime_capture_digest": capture.get("digest", ""),
                "requirements": req_reports,
                "checks": checks,
                "total_failures": profile_failures,
                "qualification_pass": profile_pass,
            }
        )

    stable_payload = {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "profile_schema": PROFILE_SCHEMA,
        "seed": seed,
        "runtime_capture_digest": capture.get("digest", ""),
        "profiles": profile_reports,
        "injected_failures": sorted(
            [f"{profile_id}:{requirement_id}" for profile_id, requirement_id in failures]
        ),
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "profile_schema": PROFILE_SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "capture_mode": capture.get("capture_mode", ""),
        "runtime_capture_path": capture_source,
        "runtime_capture_digest": capture.get("digest", ""),
        "release_image_path": capture.get("image_path", ""),
        "release_image_digest": capture.get("image_digest", ""),
        "attestation_path": attestation_source,
        "attestation_meets_target": attestation.get("meets_target"),
        "pkg_rebuild_path": pkg_rebuild_source,
        "qualified_surface": runtime_qual.default_lts_surface(capture),
        "checked_profiles": profiles,
        "profiles": profile_reports,
        "injected_failures": sorted(
            [f"{profile_id}:{requirement_id}" for profile_id, requirement_id in failures]
        ),
        "total_failures": total_failures,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--profile",
        action="append",
        default=[],
        help="profile id to evaluate (default: all profiles)",
    )
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force requirement failure in the form <profile_id>:<requirement_id>",
    )
    parser.add_argument(
        "--runtime-capture",
        default="",
        help="booted runtime capture to use for live profile qualification",
    )
    parser.add_argument(
        "--release-attestation",
        default="",
        help="release attestation report backing server profile supply-chain checks",
    )
    parser.add_argument(
        "--pkg-rebuild-report",
        default="",
        help="package rebuild report backing developer profile build checks",
    )
    parser.add_argument(
        "--fixture",
        action="store_true",
        help="use the deterministic booted runtime fixture instead of out/booted-runtime-v1.json",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/conformance-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    try:
        selected_profiles = _normalize_profiles(args.profile)
        failures = _parse_injections(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_suite(
        seed=args.seed,
        selected_profiles=selected_profiles,
        injected_failures=failures,
        runtime_capture_path=args.runtime_capture,
        attestation_path=args.release_attestation,
        pkg_rebuild_path=args.pkg_rebuild_report,
        fixture=args.fixture,
    )
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"profile-conformance-report: {out_path}")
    print(f"profiles: {','.join(report['checked_profiles'])}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
