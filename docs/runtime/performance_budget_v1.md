# Performance Budget v1

Date: 2026-03-09  
Milestone: M24 Performance Baseline + Regression Budgets v1  
Budget ID: `rugo.performance_budget.v1`  
Baseline schema: `rugo.perf_baseline.v1`  
Regression schema: `rugo.perf_regression_report.v1`

## Purpose

Define workload-scoped performance budgets that are enforced in local and CI
release lanes.

## Workload classes and budgets

| Workload class | Primary throughput metric | Primary latency metric | Max throughput regression | Max latency regression |
|---|---|---|---|---|
| `syscall_spam` | `throughput_ops_per_sec` | `latency_p95_us` | 5.0% | 7.0% |
| `ipc_loop` | `throughput_ops_per_sec` | `latency_p95_us` | 5.0% | 7.0% |
| `blk_loop` | `throughput_ops_per_sec` | `latency_p95_us` | 6.0% | 8.0% |
| `pressure_shm` | `throughput_ops_per_sec` | `latency_p95_us` | 6.0% | 9.0% |
| `thread_spawn` | `throughput_ops_per_sec` | `latency_p95_us` | 7.0% | 10.0% |
| `vm_map` | `throughput_ops_per_sec` | `latency_p95_us` | 7.0% | 10.0% |

## Baseline and regression artifacts

- Baseline capture tool: `tools/run_perf_baseline_v1.py`
- Regression check tool: `tools/check_perf_regression_v1.py`
- Baseline artifact: `out/perf-baseline-v1.json`
- Regression artifact: `out/perf-regression-v1.json`

## Enforcement

- Local release gate: `make test-perf-regression-v1`
- CI release gate: `Performance regression v1 gate`
- Regression violations above threshold are release-blocking.

Required M24 checks:

- `tests/runtime/test_perf_budget_docs_v1.py`
- `tests/runtime/test_perf_regression_v1.py`
- `tests/runtime/test_perf_gate_v1.py`
