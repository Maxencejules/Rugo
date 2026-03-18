#!/usr/bin/env python3
"""Evaluate M24 performance regressions against a boot-backed v1 baseline."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple

import run_perf_baseline_v1 as baseline_tool
import runtime_capture_common_v1 as runtime_capture


THROUGHPUT_METRIC = baseline_tool.THROUGHPUT_METRIC
LATENCY_METRIC = baseline_tool.LATENCY_METRIC
SUPPORTED_METRICS = {THROUGHPUT_METRIC, LATENCY_METRIC}


def _throughput_regression_pct(baseline: float, current: float) -> float:
    if baseline <= 0:
        return 100.0
    return max(0.0, (baseline - current) / baseline * 100.0)


def _latency_regression_pct(baseline: float, current: float) -> float:
    if baseline <= 0:
        return 100.0
    return max(0.0, (current - baseline) / baseline * 100.0)


def _parse_injection(spec: str) -> Tuple[str, str, float]:
    parts = [part.strip() for part in spec.split(":")]
    if len(parts) != 3:
        raise ValueError(
            "inject-regression format must be workload:metric:percent, "
            f"got '{spec}'"
        )
    workload, metric, raw_percent = parts
    if metric not in SUPPORTED_METRICS:
        raise ValueError(
            f"unsupported metric '{metric}'; expected one of {sorted(SUPPORTED_METRICS)}"
        )
    percent = float(raw_percent)
    if percent < 0:
        raise ValueError("regression percent must be >= 0")
    return workload, metric, percent


def _collect_injections(specs: List[str]) -> Dict[Tuple[str, str], float]:
    injections: Dict[Tuple[str, str], float] = {}
    for spec in specs:
        workload, metric, percent = _parse_injection(spec)
        injections[(workload, metric)] = percent
    return injections


def _apply_injections(
    workload: str,
    metrics: Dict[str, float],
    injections: Dict[Tuple[str, str], float],
) -> Dict[str, float]:
    adjusted = dict(metrics)
    for metric in SUPPORTED_METRICS:
        injected_pct = injections.get((workload, metric))
        if injected_pct is None:
            continue
        if metric == THROUGHPUT_METRIC:
            adjusted[metric] = round(max(0.001, adjusted[metric] * (1.0 - injected_pct / 100.0)), 3)
        else:
            adjusted[metric] = round(adjusted[metric] * (1.0 + injected_pct / 100.0), 3)
    return adjusted


def _evaluate_regression(
    *,
    baseline_payload: Dict[str, object],
    current_baseline: Dict[str, object],
    injections: Dict[Tuple[str, str], float],
) -> Dict[str, object]:
    current_metrics_by_workload = {
        str(entry["workload"]): entry["metrics"]
        for entry in current_baseline.get("workloads", [])
        if isinstance(entry, dict)
    }
    workload_results: List[Dict[str, object]] = []
    violations: List[Dict[str, object]] = []

    for workload_entry in baseline_payload["workloads"]:
        workload = str(workload_entry["workload"])
        baseline_metrics = workload_entry["metrics"]
        budgets = workload_entry["budgets"]
        current_metrics = current_metrics_by_workload.get(workload, dict(baseline_metrics))
        current_metrics = _apply_injections(workload, current_metrics, injections)

        throughput_reg = round(
            _throughput_regression_pct(
                float(baseline_metrics[THROUGHPUT_METRIC]),
                float(current_metrics[THROUGHPUT_METRIC]),
            ),
            3,
        )
        latency_reg = round(
            _latency_regression_pct(
                float(baseline_metrics[LATENCY_METRIC]),
                float(current_metrics[LATENCY_METRIC]),
            ),
            3,
        )

        throughput_budget = float(budgets["max_throughput_regression_pct"])
        latency_budget = float(budgets["max_latency_regression_pct"])
        metric_violations: List[Dict[str, object]] = []
        if throughput_reg > throughput_budget:
            metric_violations.append(
                {
                    "workload": workload,
                    "metric": THROUGHPUT_METRIC,
                    "baseline": baseline_metrics[THROUGHPUT_METRIC],
                    "current": current_metrics[THROUGHPUT_METRIC],
                    "regression_pct": throughput_reg,
                    "threshold_pct": throughput_budget,
                    "action": "inspect boot-backed throughput regression and rebaseline only with approval",
                }
            )
        if latency_reg > latency_budget:
            metric_violations.append(
                {
                    "workload": workload,
                    "metric": LATENCY_METRIC,
                    "baseline": baseline_metrics[LATENCY_METRIC],
                    "current": current_metrics[LATENCY_METRIC],
                    "regression_pct": latency_reg,
                    "threshold_pct": latency_budget,
                    "action": "inspect boot-backed latency regression and rebaseline only with approval",
                }
            )

        violations.extend(metric_violations)
        workload_results.append(
            {
                "workload": workload,
                "baseline_metrics": baseline_metrics,
                "current_metrics": current_metrics,
                "regressions_pct": {
                    THROUGHPUT_METRIC: throughput_reg,
                    LATENCY_METRIC: latency_reg,
                },
                "budgets_pct": {
                    "throughput": throughput_budget,
                    "latency": latency_budget,
                },
                "violations": metric_violations,
                "gate_pass": len(metric_violations) == 0,
            }
        )

    return {
        "workload_results": workload_results,
        "violations": violations,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--runtime-capture", required=True)
    parser.add_argument("--max-violations", type=int, default=0)
    parser.add_argument(
        "--inject-regression",
        action="append",
        default=[],
        help="optional workload:metric:percent injection for tests",
    )
    parser.add_argument("--out", default="out/perf-regression-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    baseline_path = Path(args.baseline)
    baseline_payload = json.loads(baseline_path.read_text(encoding="utf-8"))
    runtime_capture_path = Path(args.runtime_capture)
    current_capture = runtime_capture.read_json(runtime_capture_path)
    current_baseline = baseline_tool.run_baseline(
        current_capture,
        runtime_capture_path=str(runtime_capture_path),
    )
    injections = _collect_injections(args.inject_regression)

    evaluation = _evaluate_regression(
        baseline_payload=baseline_payload,
        current_baseline=current_baseline,
        injections=injections,
    )

    total_violations = len(evaluation["violations"])
    gate_pass = total_violations <= args.max_violations
    report = {
        "schema": "rugo.perf_regression_report.v1",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "baseline_schema": baseline_payload.get("schema", "unknown"),
        "budget_id": baseline_payload.get("budget_id", "unknown"),
        "benchmark_policy_id": baseline_payload.get("benchmark_policy_id", "unknown"),
        "baseline_path": str(baseline_path),
        "runtime_capture_path": str(runtime_capture_path),
        "runtime_capture_digest": current_capture.get("digest", ""),
        "release_image_path": current_capture.get("image_path", ""),
        "release_image_digest": current_capture.get("image_digest", ""),
        "trace_id": current_capture.get("trace_id", ""),
        "workload_count": int(baseline_payload.get("workload_count", 0)),
        "workload_results": evaluation["workload_results"],
        "violations": evaluation["violations"],
        "total_violations": total_violations,
        "max_violations": args.max_violations,
        "requires_action": total_violations > 0,
        "gate_pass": gate_pass,
    }
    report["digest"] = runtime_capture.stable_digest(
        {
            "schema": report["schema"],
            "baseline_path": report["baseline_path"],
            "runtime_capture_digest": report["runtime_capture_digest"],
            "violations": [
                {
                    "workload": violation["workload"],
                    "metric": violation["metric"],
                    "regression_pct": violation["regression_pct"],
                }
                for violation in report["violations"]
            ],
            "gate_pass": gate_pass,
        }
    )

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, report)

    print(f"perf-regression-report: {out_path}")
    print(f"total_violations: {total_violations}")
    print(f"release_image_path: {report['release_image_path']}")
    return 0 if gate_pass else 1


if __name__ == "__main__":
    raise SystemExit(main())
