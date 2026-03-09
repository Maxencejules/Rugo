#!/usr/bin/env python3
"""Evaluate M24 performance regressions against a v1 baseline artifact."""

from __future__ import annotations

import argparse
import json
import random
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple


THROUGHPUT_METRIC = "throughput_ops_per_sec"
LATENCY_METRIC = "latency_p95_us"
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


def _simulate_current_metrics(
    rng: random.Random,
    baseline_metrics: Dict[str, float],
) -> Dict[str, float]:
    throughput = round(
        baseline_metrics[THROUGHPUT_METRIC] * (1.0 + rng.uniform(-0.03, 0.03)),
        3,
    )
    latency = round(
        baseline_metrics[LATENCY_METRIC] * (1.0 + rng.uniform(-0.03, 0.03)),
        3,
    )
    return {
        THROUGHPUT_METRIC: max(0.001, throughput),
        LATENCY_METRIC: max(0.001, latency),
    }


def _evaluate_regression(
    baseline_payload: Dict[str, object],
    seed: int,
    injections: Dict[Tuple[str, str], float],
) -> Dict[str, object]:
    rng = random.Random(seed)
    workload_results: List[Dict[str, object]] = []
    violations: List[Dict[str, object]] = []

    for workload_entry in baseline_payload["workloads"]:
        workload = workload_entry["workload"]
        baseline_metrics = workload_entry["metrics"]
        budgets = workload_entry["budgets"]
        current_metrics = _simulate_current_metrics(rng, baseline_metrics)

        for metric in SUPPORTED_METRICS:
            injected_pct = injections.get((workload, metric))
            if injected_pct is None:
                continue
            if metric == THROUGHPUT_METRIC:
                current_metrics[metric] = round(
                    current_metrics[metric] * (1.0 - injected_pct / 100.0), 3
                )
                current_metrics[metric] = max(0.001, current_metrics[metric])
            else:
                current_metrics[metric] = round(
                    current_metrics[metric] * (1.0 + injected_pct / 100.0), 3
                )

        throughput_reg = round(
            _throughput_regression_pct(
                baseline_metrics[THROUGHPUT_METRIC], current_metrics[THROUGHPUT_METRIC]
            ),
            3,
        )
        latency_reg = round(
            _latency_regression_pct(
                baseline_metrics[LATENCY_METRIC], current_metrics[LATENCY_METRIC]
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
                    "action": "inspect throughput regression and rebaseline only with approval",
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
                    "action": "inspect latency regression and rebaseline only with approval",
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
    parser.add_argument("--seed", type=int, default=None)
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
    seed = args.seed if args.seed is not None else int(baseline_payload.get("seed", 20260309))
    injections = _collect_injections(args.inject_regression)

    evaluation = _evaluate_regression(
        baseline_payload=baseline_payload,
        seed=seed,
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
        "seed": seed,
        "workload_count": int(baseline_payload.get("workload_count", 0)),
        "workload_results": evaluation["workload_results"],
        "violations": evaluation["violations"],
        "total_violations": total_violations,
        "max_violations": args.max_violations,
        "requires_action": total_violations > 0,
        "gate_pass": gate_pass,
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"perf-regression-report: {out_path}")
    print(f"total_violations: {total_violations}")
    return 0 if gate_pass else 1


if __name__ == "__main__":
    raise SystemExit(main())
