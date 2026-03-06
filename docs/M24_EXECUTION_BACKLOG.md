# M24 Execution Backlog (Performance Baseline + Regression Budgets v1)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Establish objective performance baselines and automatic regression rejection for
key workload classes.

M24 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Runtime and subsystem gates exist, but unified performance budgets are absent.
- M24 introduces v1 benchmark policy and regression budget enforcement.
- No dedicated performance regression gate currently exists.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Budget + Benchmark Contract

### Objective

Freeze benchmark classes and regression budget semantics.

### Scope

- Add docs:
  - `docs/runtime/performance_budget_v1.md`
  - `docs/runtime/benchmark_policy_v1.md`
- Add tests:
  - `tests/runtime/test_perf_budget_docs_v1.py`

### Primary files

- `docs/runtime/performance_budget_v1.md`
- `docs/runtime/benchmark_policy_v1.md`
- `tests/runtime/test_perf_budget_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_perf_budget_docs_v1.py -v`

### Done criteria for PR-1

- Performance budgets are versioned and workload-scoped.
- Benchmark policy has explicit ownership and thresholds.

## PR-2: Baseline + Regression Tooling

### Objective

Implement deterministic baseline capture and regression checks.

### Scope

- Add tooling:
  - `tools/run_perf_baseline_v1.py`
  - `tools/check_perf_regression_v1.py`
- Add tests:
  - `tests/runtime/test_perf_regression_v1.py`

### Primary files

- `tools/run_perf_baseline_v1.py`
- `tools/check_perf_regression_v1.py`
- `tests/runtime/test_perf_regression_v1.py`

### Acceptance checks

- `python tools/run_perf_baseline_v1.py --out out/perf-baseline-v1.json`
- `python tools/check_perf_regression_v1.py --baseline out/perf-baseline-v1.json --out out/perf-regression-v1.json`
- `python -m pytest tests/runtime/test_perf_regression_v1.py -v`

### Done criteria for PR-2

- Baseline and regression artifacts are machine-readable and deterministic.
- Regressions above threshold are detectable and actionable.

## PR-3: Performance Gate + Closure

### Objective

Promote performance regression checks to release-blocking status.

### Scope

- Add local gate:
  - `Makefile` target `test-perf-regression-v1`
- Add CI step:
  - `Performance regression v1 gate`
- Add aggregate test:
  - `tests/runtime/test_perf_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_perf_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-perf-regression-v1`

### Done criteria for PR-3

- Performance gate is required in local and CI release lanes.
- M24 can be marked done with regression budget evidence.

## Non-goals for M24 backlog

- Absolute performance leadership claims across all workloads.
- Unbounded microbenchmark expansion without owner and threshold policy.

