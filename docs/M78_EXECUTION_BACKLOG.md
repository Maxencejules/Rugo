# M78 Execution Backlog (Scheduler Latency + CPU Affinity v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Raise scheduler behavior from a basic fairness baseline to explicit latency,
priority, and CPU-affinity semantics for SMP systems.

M78 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M16_EXECUTION_BACKLOG.md`
- `docs/M43_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Process and scheduler models exist, and SMP/topology work expanded the CPU
  substrate.
- There is no versioned scheduler-latency, affinity, or thread-priority
  contract in the post-M52 plan.
- Performance budgets exist, but interactive and service latency guarantees are
  still implicit.
- M78 must define those semantics before memory pressure and telemetry work
  build on them.

## Execution plan

- PR-1: scheduler latency contract freeze
- PR-2: priority and affinity campaign baseline
- PR-3: scheduler gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Scheduler Latency Contract Freeze

### Objective

Define latency budgets, CPU affinity, and thread-priority semantics before
implementation broadens performance claims.

### Scope

- Add docs:
  - `docs/runtime/scheduler_latency_contract_v1.md`
  - `docs/runtime/cpu_affinity_policy_v1.md`
  - `docs/abi/thread_priority_contract_v1.md`
- Add tests:
  - `tests/sched/test_scheduler_latency_docs_v1.py`

### Primary files

- `docs/runtime/scheduler_latency_contract_v1.md`
- `docs/runtime/cpu_affinity_policy_v1.md`
- `docs/abi/thread_priority_contract_v1.md`
- `tests/sched/test_scheduler_latency_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/sched/test_scheduler_latency_docs_v1.py -v`

### Done criteria for PR-1

- Latency, affinity, and priority semantics are explicit and versioned.
- Deferred scheduler behavior remains deterministic and machine-verifiable.

## PR-2: Priority and Affinity Campaign Baseline

### Objective

Implement deterministic evidence for preemption, affinity balance, and latency
budget behavior.

### Scope

- Add tooling:
  - `tools/run_scheduler_latency_v1.py`
  - `tools/run_affinity_balance_v1.py`
- Add tests:
  - `tests/sched/test_priority_preemption_v1.py`
  - `tests/sched/test_cpu_affinity_v1.py`
  - `tests/runtime/test_scheduler_latency_budget_v1.py`
  - `tests/sched/test_scheduler_negative_v1.py`

### Primary files

- `tools/run_scheduler_latency_v1.py`
- `tools/run_affinity_balance_v1.py`
- `tests/sched/test_priority_preemption_v1.py`
- `tests/sched/test_cpu_affinity_v1.py`
- `tests/runtime/test_scheduler_latency_budget_v1.py`
- `tests/sched/test_scheduler_negative_v1.py`

### Acceptance checks

- `python tools/run_scheduler_latency_v1.py --out out/scheduler-latency-v1.json`
- `python tools/run_affinity_balance_v1.py --out out/affinity-balance-v1.json`
- `python -m pytest tests/sched/test_priority_preemption_v1.py tests/sched/test_cpu_affinity_v1.py tests/runtime/test_scheduler_latency_budget_v1.py tests/sched/test_scheduler_negative_v1.py -v`

### Done criteria for PR-2

- Scheduler artifacts are deterministic and machine-readable.
- `SCHED: rt ok` and affinity markers are stable across seeded runs.
- Later pressure and telemetry work can reference explicit latency policy IDs.

## PR-3: Process Scheduler v3 Gate + Latency Sub-gate

### Objective

Make scheduler latency and affinity behavior release-blocking for declared SMP
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-process-scheduler-v3`
  - `Makefile` target `test-scheduler-latency-v1`
- Add CI steps:
  - `Process scheduler v3 gate`
  - `Scheduler latency v1 gate`
- Add aggregate tests:
  - `tests/sched/test_process_scheduler_gate_v3.py`
  - `tests/runtime/test_scheduler_latency_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/sched/test_process_scheduler_gate_v3.py`
- `tests/runtime/test_scheduler_latency_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-process-scheduler-v3`
- `make test-scheduler-latency-v1`

### Done criteria for PR-3

- Scheduler and latency sub-gates are required in local and CI release lanes.
- M78 can be marked done only with deterministic priority, affinity, and
  latency evidence on declared SMP profiles.

## Non-goals for M78 backlog

- memory pressure and I/O QoS work owned by M79
- full hard real-time policy breadth beyond the declared contract scope
- telemetry and chaos qualification owned by M80-M81
