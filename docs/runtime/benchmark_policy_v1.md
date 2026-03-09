# Benchmark Policy v1

Date: 2026-03-09  
Milestone: M24 Performance Baseline + Regression Budgets v1  
Policy ID: `rugo.benchmark_policy.v1`

## Objective

Define ownership, execution rules, and enforcement semantics for repeatable
performance baseline and regression checks.

## Ownership

- Primary owner: Runtime maintainers.
- Secondary owner: Release engineering maintainers.
- Escalation SLA for threshold regressions: 3 business days.

## Benchmark classes in scope

- `syscall_spam`
- `ipc_loop`
- `blk_loop`
- `pressure_shm`
- `thread_spawn`
- `vm_map`

## Determinism and reproducibility rules

- Default seed: `20260309`.
- Minimum iterations per class: `1200`.
- Same seed and inputs must generate equivalent reports except for timestamps.
- Baselines are versioned artifacts and cannot be overwritten without review.

## Threshold and decision semantics

- Throughput regression percent:
  - `max(0, (baseline - current) / baseline * 100)`.
- Latency regression percent:
  - `max(0, (current - baseline) / baseline * 100)`.
- A workload is failing when either metric exceeds the budget in
  `docs/runtime/performance_budget_v1.md`.
- Gate outcome: `total_violations` must be `0`.

## Required commands and evidence

- `python tools/run_perf_baseline_v1.py --out out/perf-baseline-v1.json`
- `python tools/check_perf_regression_v1.py --baseline out/perf-baseline-v1.json --out out/perf-regression-v1.json`
- `make test-perf-regression-v1`

Artifacts:

- Baseline schema: `rugo.perf_baseline.v1`
- Regression schema: `rugo.perf_regression_report.v1`
