# M80 Execution Backlog (Observability Pipeline + Perf Telemetry v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add explicit telemetry and perf-counter semantics so regressions can be
measured automatically against both Rugo and legacy-C baselines.

M80 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M29_EXECUTION_BACKLOG.md`
- `docs/M79_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Observability and crash diagnostics exist, and M79 defines pressure and QoS
  behavior that now need measurable baselines.
- There is no versioned telemetry, perf-counter, or legacy-baseline diff
  contract in the post-M79 plan.
- Performance comparisons remain partially ad hoc and not uniformly
  release-gated.
- M80 must define those semantics before chaos qualification depends on them.

## Execution plan

- PR-1: telemetry contract freeze
- PR-2: perf telemetry and diff campaign baseline
- PR-3: observability v3 gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Telemetry Contract Freeze

### Objective

Define metric registry, perf-counter, and baseline-diff semantics before
implementation broadens observability claims.

### Scope

- Add docs:
  - `docs/runtime/telemetry_contract_v1.md`
  - `docs/runtime/perf_counter_abi_v1.md`
  - `docs/build/perf_baseline_policy_v1.md`
- Add tests:
  - `tests/runtime/test_telemetry_docs_v1.py`

### Primary files

- `docs/runtime/telemetry_contract_v1.md`
- `docs/runtime/perf_counter_abi_v1.md`
- `docs/build/perf_baseline_policy_v1.md`
- `tests/runtime/test_telemetry_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_telemetry_docs_v1.py -v`

### Done criteria for PR-1

- Telemetry, perf-counter, and baseline-diff semantics are explicit and
  versioned.
- Unsupported metrics and redaction requirements remain deterministic.

## PR-2: Perf Telemetry and Diff Campaign Baseline

### Objective

Implement deterministic evidence for telemetry collection, perf counters, and
legacy-baseline comparison.

### Scope

- Add tooling:
  - `tools/collect_perf_telemetry_v1.py`
  - `tools/run_legacy_baseline_diff_v1.py`
- Add tests:
  - `tests/runtime/test_metric_registry_v1.py`
  - `tests/runtime/test_perf_counter_abi_v1.py`
  - `tests/runtime/test_legacy_baseline_diff_v1.py`
  - `tests/runtime/test_perf_telemetry_negative_v1.py`

### Primary files

- `tools/collect_perf_telemetry_v1.py`
- `tools/run_legacy_baseline_diff_v1.py`
- `tests/runtime/test_metric_registry_v1.py`
- `tests/runtime/test_perf_counter_abi_v1.py`
- `tests/runtime/test_legacy_baseline_diff_v1.py`
- `tests/runtime/test_perf_telemetry_negative_v1.py`

### Acceptance checks

- `python tools/collect_perf_telemetry_v1.py --out out/perf-telemetry-v1.json`
- `python tools/run_legacy_baseline_diff_v1.py --out out/legacy-baseline-diff-v1.json`
- `python -m pytest tests/runtime/test_metric_registry_v1.py tests/runtime/test_perf_counter_abi_v1.py tests/runtime/test_legacy_baseline_diff_v1.py tests/runtime/test_perf_telemetry_negative_v1.py -v`

### Done criteria for PR-2

- Telemetry artifacts are deterministic and machine-readable.
- `OBS: metric ok` and baseline-diff markers are stable.
- Reliability and release consumers can reference explicit telemetry contract
  IDs.

## PR-3: Observability v3 Gate + Perf Telemetry Sub-gate

### Objective

Make telemetry and baseline-diff behavior release-blocking for declared
performance profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-observability-v3`
  - `Makefile` target `test-perf-telemetry-v1`
- Add CI steps:
  - `Observability v3 gate`
  - `Perf telemetry v1 gate`
- Add aggregate tests:
  - `tests/runtime/test_observability_gate_v3.py`
  - `tests/runtime/test_perf_telemetry_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_observability_gate_v3.py`
- `tests/runtime/test_perf_telemetry_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-observability-v3`
- `make test-perf-telemetry-v1`

### Done criteria for PR-3

- Observability and perf-telemetry sub-gates are required in local and CI
  release lanes.
- M80 can be marked done only with deterministic metric, counter, and
  baseline-diff evidence for declared profiles.

## Non-goals for M80 backlog

- chaos and fuzz qualification work owned by M81
- production backend deployment beyond declared artifact generation
- unlimited telemetry retention or cardinality outside policy
