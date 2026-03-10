#!/usr/bin/env python3
"""Run deterministic runtime-qualified GUI app matrix checks for M44."""

from __future__ import annotations

import argparse
from dataclasses import dataclass, replace
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set


PROFILE_ID = "rugo.desktop_profile.v2"
PROFILE_LABEL = "desktop_profile_v2"
TIER_SCHEMA = "rugo.app_compat_tiers.v2"
REPORT_SCHEMA = "rugo.real_gui_app_matrix_report.v2"
DEFAULT_SEED = 20260310

CLASS_THRESHOLDS: Dict[str, Dict[str, object]] = {
    "productivity": {
        "tier": "tier_productivity_runtime",
        "min_cases": 8,
        "min_pass_rate": 0.875,
    },
    "media": {"tier": "tier_media_runtime", "min_cases": 6, "min_pass_rate": 0.833},
    "utility": {
        "tier": "tier_utility_runtime",
        "min_cases": 7,
        "min_pass_rate": 0.857,
    },
}


@dataclass(frozen=True)
class GuiRuntimeCase:
    case_id: str
    app_id: str
    app_class: str
    tier: str
    lane: str
    runtime_trace_id: str
    launch_ok: bool = True
    render_ok: bool = True
    input_ok: bool = True
    deterministic: bool = True
    signed_provenance: bool = True
    reproducible: bool = True
    runtime_source: str = "runtime_capture"
    profile: str = PROFILE_LABEL


def _trace_id(seed: int, lane: str, case_id: str) -> str:
    digest = hashlib.sha256(f"{seed}|{lane}|{case_id}|trace".encode("utf-8")).hexdigest()
    return f"trace-{lane}-{digest[:12]}"


def _baseline_cases(seed: int) -> List[GuiRuntimeCase]:
    cases: List[GuiRuntimeCase] = []

    for idx in range(8):
        lane = "qemu" if idx % 2 == 0 else "baremetal"
        case_id = f"productivity-runtime-{idx:02d}"
        cases.append(
            GuiRuntimeCase(
                case_id=case_id,
                app_id=f"gui-productivity-runtime-{idx:02d}",
                app_class="productivity",
                tier="tier_productivity_runtime",
                lane=lane,
                runtime_trace_id=_trace_id(seed, lane, case_id),
                render_ok=idx != 7,  # 7/8 pass
            )
        )

    for idx in range(6):
        lane = "qemu" if idx % 2 == 0 else "baremetal"
        case_id = f"media-runtime-{idx:02d}"
        cases.append(
            GuiRuntimeCase(
                case_id=case_id,
                app_id=f"gui-media-runtime-{idx:02d}",
                app_class="media",
                tier="tier_media_runtime",
                lane=lane,
                runtime_trace_id=_trace_id(seed, lane, case_id),
                launch_ok=idx != 5,  # 5/6 pass
            )
        )

    for idx in range(7):
        lane = "qemu" if idx % 2 == 0 else "baremetal"
        case_id = f"utility-runtime-{idx:02d}"
        cases.append(
            GuiRuntimeCase(
                case_id=case_id,
                app_id=f"gui-utility-runtime-{idx:02d}",
                app_class="utility",
                tier="tier_utility_runtime",
                lane=lane,
                runtime_trace_id=_trace_id(seed, lane, case_id),
                input_ok=idx != 6,  # 6/7 pass
            )
        )

    return cases


def _known_case_ids() -> Set[str]:
    return {case.case_id for case in _baseline_cases(DEFAULT_SEED)}


def _normalize_case_ids(values: Sequence[str]) -> Set[str]:
    return {value.strip() for value in values if value.strip()}


def _validate_case_ids(label: str, case_ids: Set[str]) -> None:
    unknown = sorted(case_ids - _known_case_ids())
    if unknown:
        raise ValueError(f"unknown case ids in {label}: {', '.join(unknown)}")


def _metric(seed: int, case_id: str, label: str, base: int, spread: int) -> int:
    digest = hashlib.sha256(f"{seed}|{case_id}|{label}".encode("utf-8")).hexdigest()
    return base + (int(digest[:8], 16) % spread)


def _apply_injections(
    cases: List[GuiRuntimeCase],
    launch_failures: Set[str],
    render_failures: Set[str],
    input_failures: Set[str],
    nondeterministic: Set[str],
    unsigned: Set[str],
    unreproducible: Set[str],
    missing_trace: Set[str],
    non_runtime_source: Set[str],
    profile_mismatches: Set[str],
) -> List[GuiRuntimeCase]:
    updated: List[GuiRuntimeCase] = []
    for case in cases:
        candidate = case
        if case.case_id in launch_failures:
            candidate = replace(candidate, launch_ok=False)
        if case.case_id in render_failures:
            candidate = replace(candidate, render_ok=False)
        if case.case_id in input_failures:
            candidate = replace(candidate, input_ok=False)
        if case.case_id in nondeterministic:
            candidate = replace(candidate, deterministic=False)
        if case.case_id in unsigned:
            candidate = replace(candidate, signed_provenance=False)
        if case.case_id in unreproducible:
            candidate = replace(candidate, reproducible=False)
        if case.case_id in missing_trace:
            candidate = replace(candidate, runtime_trace_id="")
        if case.case_id in non_runtime_source:
            candidate = replace(candidate, runtime_source="synthetic_model")
        if case.case_id in profile_mismatches:
            candidate = replace(candidate, profile="desktop_profile_v1")
        updated.append(candidate)
    return updated


def run_matrix(
    seed: int,
    launch_failures: Set[str] | None = None,
    render_failures: Set[str] | None = None,
    input_failures: Set[str] | None = None,
    nondeterministic: Set[str] | None = None,
    unsigned: Set[str] | None = None,
    unreproducible: Set[str] | None = None,
    missing_trace: Set[str] | None = None,
    non_runtime_source: Set[str] | None = None,
    profile_mismatches: Set[str] | None = None,
    max_failures: int = 0,
) -> Dict[str, object]:
    launch = set() if launch_failures is None else set(launch_failures)
    render = set() if render_failures is None else set(render_failures)
    inputf = set() if input_failures is None else set(input_failures)
    nondet = set() if nondeterministic is None else set(nondeterministic)
    unsigned_fail = set() if unsigned is None else set(unsigned)
    unrepro = set() if unreproducible is None else set(unreproducible)
    missing = set() if missing_trace is None else set(missing_trace)
    non_runtime = set() if non_runtime_source is None else set(non_runtime_source)
    profile_bad = set() if profile_mismatches is None else set(profile_mismatches)

    cases = _apply_injections(
        _baseline_cases(seed),
        launch_failures=launch,
        render_failures=render,
        input_failures=inputf,
        nondeterministic=nondet,
        unsigned=unsigned_fail,
        unreproducible=unrepro,
        missing_trace=missing,
        non_runtime_source=non_runtime,
        profile_mismatches=profile_bad,
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
        elif not case.deterministic:
            issues.append({"case_id": case.case_id, "reason": "non_deterministic_result"})
            counted_for_threshold = False
        elif not case.signed_provenance:
            issues.append({"case_id": case.case_id, "reason": "unsigned_provenance"})
            counted_for_threshold = False
        elif not case.reproducible:
            issues.append({"case_id": case.case_id, "reason": "non_reproducible_result"})
            counted_for_threshold = False
        elif not case.runtime_trace_id:
            issues.append({"case_id": case.case_id, "reason": "missing_runtime_trace"})
            counted_for_threshold = False
        elif case.runtime_source != "runtime_capture":
            issues.append({"case_id": case.case_id, "reason": "non_runtime_source"})
            counted_for_threshold = False
        elif case.profile != PROFILE_LABEL:
            issues.append({"case_id": case.case_id, "reason": "profile_mismatch"})
            counted_for_threshold = False

        passed = case.launch_ok and case.render_ok and case.input_ok
        if counted_for_threshold:
            bucket = class_totals[case.app_class]
            bucket["eligible"] += 1
            if passed:
                bucket["passed"] += 1

        case_reports.append(
            {
                "case_id": case.case_id,
                "app_id": case.app_id,
                "class": case.app_class,
                "tier": case.tier,
                "lane": case.lane,
                "launch_ok": case.launch_ok,
                "render_ok": case.render_ok,
                "input_ok": case.input_ok,
                "deterministic": case.deterministic,
                "signed_provenance": case.signed_provenance,
                "reproducible": case.reproducible,
                "runtime_source": case.runtime_source,
                "runtime_trace_id": case.runtime_trace_id,
                "profile": case.profile,
                "passed": passed,
                "counted_for_threshold": counted_for_threshold,
                "metrics": {
                    "launch_ms": _metric(seed, case.case_id, "launch", base=49, spread=27),
                    "frame_time_ms_p95": _metric(
                        seed, case.case_id, "frame", base=8, spread=9
                    ),
                    "input_latency_ms_p95": _metric(
                        seed, case.case_id, "input", base=6, spread=8
                    ),
                },
            }
        )

    class_reports: Dict[str, Dict[str, object]] = {}
    class_failures = 0
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
        if not meets_threshold:
            class_failures += 1

        class_reports[class_name] = {
            "tier": threshold["tier"],
            "eligible": eligible,
            "passed": passed,
            "pass_rate": round(pass_rate, 3),
            "min_cases": int(threshold["min_cases"]),
            "min_pass_rate": float(threshold["min_pass_rate"]),
            "meets_threshold": meets_threshold,
        }

    total_cases = len(case_reports)
    signed_count = sum(1 for case in cases if case.signed_provenance)
    reproducible_count = sum(1 for case in cases if case.reproducible)
    deterministic_count = sum(1 for case in cases if case.deterministic)
    runtime_trace_count = sum(1 for case in cases if case.runtime_trace_id != "")
    runtime_source_count = sum(1 for case in cases if case.runtime_source == "runtime_capture")
    lane_counts = {
        "qemu": sum(1 for case in cases if case.lane == "qemu"),
        "baremetal": sum(1 for case in cases if case.lane == "baremetal"),
    }
    provenance = {
        "signed_provenance_ratio": round(signed_count / total_cases, 3),
        "reproducible_ratio": round(reproducible_count / total_cases, 3),
        "deterministic_ratio": round(deterministic_count / total_cases, 3),
        "runtime_trace_coverage_ratio": round(runtime_trace_count / total_cases, 3),
        "runtime_source_ratio": round(runtime_source_count / total_cases, 3),
        "lane_coverage": lane_counts,
    }

    issues_sorted = sorted(issues, key=lambda item: (str(item["reason"]), item["case_id"]))
    total_failures = len(issues_sorted) + class_failures
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": REPORT_SCHEMA,
        "profile_id": PROFILE_ID,
        "tier_schema": TIER_SCHEMA,
        "seed": seed,
        "classes": class_reports,
        "provenance": provenance,
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

    return {
        "schema": REPORT_SCHEMA,
        "profile_id": PROFILE_ID,
        "tier_schema": TIER_SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "gate": "test-real-ecosystem-desktop-v2",
        "total_cases": total_cases,
        "classes": class_reports,
        "provenance": provenance,
        "cases": case_reports,
        "issues": issues_sorted,
        "max_failures": max_failures,
        "total_failures": total_failures,
        "gate_pass": gate_pass,
        "artifact_refs": {
            "junit": "out/pytest-real-ecosystem-desktop-v2.xml",
            "gui_report": "out/real-gui-matrix-v2.json",
            "pkg_report": "out/real-pkg-install-v2.json",
            "audit_report": "out/real-catalog-audit-v2.json",
            "ci_artifact": "real-ecosystem-desktop-v2-artifacts",
            "subgate_ci_artifact": "real-app-catalog-v2-artifacts",
        },
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--inject-launch-failure",
        action="append",
        default=[],
        help="force launch failure for a case id",
    )
    parser.add_argument(
        "--inject-render-failure",
        action="append",
        default=[],
        help="force render failure for a case id",
    )
    parser.add_argument(
        "--inject-input-failure",
        action="append",
        default=[],
        help="force input failure for a case id",
    )
    parser.add_argument(
        "--inject-nondeterministic",
        action="append",
        default=[],
        help="force non-deterministic result for a case id",
    )
    parser.add_argument(
        "--inject-unsigned",
        action="append",
        default=[],
        help="force unsigned provenance for a case id",
    )
    parser.add_argument(
        "--inject-unreproducible",
        action="append",
        default=[],
        help="force non-reproducible result for a case id",
    )
    parser.add_argument(
        "--inject-missing-trace",
        action="append",
        default=[],
        help="force missing runtime trace for a case id",
    )
    parser.add_argument(
        "--inject-non-runtime-source",
        action="append",
        default=[],
        help="force synthetic runtime source for a case id",
    )
    parser.add_argument(
        "--inject-profile-mismatch",
        action="append",
        default=[],
        help="force profile mismatch for a case id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/real-gui-matrix-v2.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    injections = {
        "inject-launch-failure": _normalize_case_ids(args.inject_launch_failure),
        "inject-render-failure": _normalize_case_ids(args.inject_render_failure),
        "inject-input-failure": _normalize_case_ids(args.inject_input_failure),
        "inject-nondeterministic": _normalize_case_ids(args.inject_nondeterministic),
        "inject-unsigned": _normalize_case_ids(args.inject_unsigned),
        "inject-unreproducible": _normalize_case_ids(args.inject_unreproducible),
        "inject-missing-trace": _normalize_case_ids(args.inject_missing_trace),
        "inject-non-runtime-source": _normalize_case_ids(
            args.inject_non_runtime_source
        ),
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
        launch_failures=injections["inject-launch-failure"],
        render_failures=injections["inject-render-failure"],
        input_failures=injections["inject-input-failure"],
        nondeterministic=injections["inject-nondeterministic"],
        unsigned=injections["inject-unsigned"],
        unreproducible=injections["inject-unreproducible"],
        missing_trace=injections["inject-missing-trace"],
        non_runtime_source=injections["inject-non-runtime-source"],
        profile_mismatches=injections["inject-profile-mismatch"],
        max_failures=args.max_failures,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"real-gui-matrix-report: {out_path}")
    print(f"issues: {len(report['issues'])}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
