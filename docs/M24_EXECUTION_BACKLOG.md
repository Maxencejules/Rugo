# M24 Execution Backlog (Performance Baseline + Regression Budgets v1)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Establish objective performance baselines and automatic regression rejection for
key workload classes.

M24 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Runtime and subsystem gates now include workload-scoped performance budgets
  derived from the booted default image.
- Boot-backed runtime capture, baseline, and regression artifacts are
  implemented and test-backed.
- Performance regression v1 is release-gated in local and CI lanes with the
  shipped `out/os-go.iso` path wired into the capture flow.

## Execution Result

- PR-1: complete (2026-03-09)
- PR-2: complete (2026-03-09)
- PR-3: complete (2026-03-09)

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

### PR-1 completion summary

- Added performance budget and benchmark policy docs:
  - `docs/runtime/performance_budget_v1.md`
  - `docs/runtime/benchmark_policy_v1.md`
- Added executable PR-1 checks:
  - `tests/runtime/test_perf_budget_docs_v1.py`

## PR-2: Baseline + Regression Tooling

### Objective

Implement boot-backed runtime capture plus baseline and regression checks.

### Scope

- Add tooling:
  - `tools/collect_booted_runtime_v1.py`
  - `tools/runtime_capture_common_v1.py`
  - `tools/run_perf_baseline_v1.py`
  - `tools/check_perf_regression_v1.py`
- Add tests:
  - `tests/runtime/test_booted_runtime_capture_v1.py`
  - `tests/runtime/test_perf_regression_v1.py`

### Primary files

- `tools/collect_booted_runtime_v1.py`
- `tools/runtime_capture_common_v1.py`
- `tools/run_perf_baseline_v1.py`
- `tools/check_perf_regression_v1.py`
- `tests/runtime/test_booted_runtime_capture_v1.py`
- `tests/runtime/test_perf_regression_v1.py`

### Acceptance checks

- `python tools/collect_booted_runtime_v1.py --image out/os-go.iso --kernel out/kernel-go.elf --out out/booted-runtime-v1.json`
- `python tools/run_perf_baseline_v1.py --runtime-capture out/booted-runtime-v1.json --out out/perf-baseline-v1.json`
- `python tools/check_perf_regression_v1.py --baseline out/perf-baseline-v1.json --runtime-capture out/booted-runtime-v1.json --out out/perf-regression-v1.json`
- `python -m pytest tests/runtime/test_booted_runtime_capture_v1.py tests/runtime/test_perf_regression_v1.py -v`

### Done criteria for PR-2

- Baseline and regression artifacts are machine-readable and boot-backed.
- Regressions above threshold are detectable and actionable.

### PR-2 completion summary

- Added boot-backed runtime capture and performance tooling:
  - `tools/collect_booted_runtime_v1.py`
  - `tools/runtime_capture_common_v1.py`
  - `tools/run_perf_baseline_v1.py`
  - `tools/check_perf_regression_v1.py`
- Added executable PR-2 checks:
  - `tests/runtime/test_booted_runtime_capture_v1.py`
  - `tests/runtime/test_perf_regression_v1.py`

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
- M24 can be marked done with regression budget evidence tied to the shipped
  default image.

### PR-3 completion summary

- Added aggregate gate test:
  - `tests/runtime/test_perf_gate_v1.py`
- Added local gate:
  - `make test-perf-regression-v1`
  - runtime capture artifact: `out/booted-runtime-v1.json`
  - JUnit output: `out/pytest-perf-regression-v1.xml`
- Added CI gate + artifact upload:
  - step: `Performance regression v1 gate`
  - artifact: `perf-regression-v1-artifacts`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M24 backlog

- Absolute performance leadership claims across all workloads.
- Unbounded microbenchmark expansion without owner and threshold policy.
