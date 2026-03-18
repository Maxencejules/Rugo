# Performance Budget v1

Date: 2026-03-18  
Milestone: M24 Performance Baseline + Regression Budgets v1  
Budget ID: `rugo.performance_budget.v1`  
Booted runtime schema: `rugo.booted_runtime_capture.v1`  
Baseline schema: `rugo.perf_baseline.v1`  
Regression schema: `rugo.perf_regression_report.v1`

## Purpose

Define workload-scoped performance budgets that are derived from the booted
default release image and enforced in local and CI release lanes.

## Workload classes and budgets

| Workload class | Primary throughput metric | Primary latency metric | Max throughput regression | Max latency regression |
|---|---|---|---|---|
| `cpu_service_cycle` | `throughput_ops_per_sec` | `latency_p95_us` | 5.0% | 7.0% |
| `memory_diag_snapshot` | `throughput_ops_per_sec` | `latency_p95_us` | 5.0% | 7.0% |
| `block_recovery_cycle` | `throughput_ops_per_sec` | `latency_p95_us` | 6.0% | 8.0% |
| `network_roundtrip_cycle` | `throughput_ops_per_sec` | `latency_p95_us` | 6.0% | 8.0% |
| `service_restart_cycle` | `throughput_ops_per_sec` | `latency_p95_us` | 7.0% | 10.0% |
| `mixed_runtime_cycle` | `throughput_ops_per_sec` | `latency_p95_us` | 7.0% | 10.0% |

## Baseline and regression artifacts

- Booted runtime capture tool: `tools/collect_booted_runtime_v1.py`
- Baseline capture tool: `tools/run_perf_baseline_v1.py`
- Regression check tool: `tools/check_perf_regression_v1.py`
- Default release image: `out/os-go.iso`
- Runtime capture artifact: `out/booted-runtime-v1.json`
- Baseline artifact: `out/perf-baseline-v1.json`
- Regression artifact: `out/perf-regression-v1.json`

## Enforcement

- Local release gate: `make test-perf-regression-v1`
- CI release gate: `Performance regression v1 gate`
- The gate boots `out/os-go.iso`, captures the runtime twice, and derives
  performance budgets from the resulting boot-backed metrics.
- Regression violations above threshold are release-blocking.

Required M24 checks:

- `tests/runtime/test_booted_runtime_capture_v1.py`
- `tests/runtime/test_perf_budget_docs_v1.py`
- `tests/runtime/test_perf_regression_v1.py`
- `tests/runtime/test_perf_gate_v1.py`
