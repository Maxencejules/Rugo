# Benchmark Policy v1

Date: 2026-03-18  
Milestone: M24 Performance Baseline + Regression Budgets v1  
Policy ID: `rugo.benchmark_policy.v1`

## Objective

Define ownership, execution rules, and enforcement semantics for boot-backed
performance baseline and regression checks.

## Ownership

- Primary owner: Runtime maintainers.
- Secondary owner: Release engineering maintainers.
- Escalation SLA for threshold regressions: 3 business days.

## Benchmark classes in scope

- `cpu_service_cycle`
- `memory_diag_snapshot`
- `block_recovery_cycle`
- `network_roundtrip_cycle`
- `service_restart_cycle`
- `mixed_runtime_cycle`

## Reproducibility and provenance rules

- Default release image: `out/os-go.iso`.
- Default kernel image: `out/kernel-go.elf`.
- Fixture seed: `20260318`.
- Minimum boots per capture: `2`.
- Runtime capture lane: `qemu`.
- The performance gate must derive baseline and regression artifacts from the
  same booted runtime capture flow that produced `out/booted-runtime-v1.json`.
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

- `python tools/collect_booted_runtime_v1.py --image out/os-go.iso --kernel out/kernel-go.elf --out out/booted-runtime-v1.json`
- `python tools/run_perf_baseline_v1.py --runtime-capture out/booted-runtime-v1.json --out out/perf-baseline-v1.json`
- `python tools/check_perf_regression_v1.py --baseline out/perf-baseline-v1.json --runtime-capture out/booted-runtime-v1.json --out out/perf-regression-v1.json`
- `make test-perf-regression-v1`

Artifacts:

- Booted runtime capture schema: `rugo.booted_runtime_capture.v1`
- Baseline schema: `rugo.perf_baseline.v1`
- Regression schema: `rugo.perf_regression_report.v1`
