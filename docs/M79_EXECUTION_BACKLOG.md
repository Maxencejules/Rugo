# M79 Execution Backlog (Memory Pressure + I/O QoS v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add explicit memory-pressure and I/O QoS semantics so the system can remain
stable under mixed interactive and service load.

M79 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M42_EXECUTION_BACKLOG.md`
- `docs/M78_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Resource control exists, and M78 defines scheduler latency behavior.
- There is no versioned memory-pressure, I/O QoS, or resource-pressure events
  contract in the post-M78 plan.
- OOM and reclaim policy remain mostly implicit implementation behavior.
- M79 must define those semantics before telemetry and reliability work depends
  on them.

## Execution plan

- PR-1: memory pressure contract freeze
- PR-2: reclaim and QoS campaign baseline
- PR-3: pressure/QoS gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Memory Pressure Contract Freeze

### Objective

Define memory-pressure, reclaim, and I/O QoS semantics before implementation
widens stability claims.

### Scope

- Add docs:
  - `docs/runtime/memory_pressure_contract_v1.md`
  - `docs/storage/io_qos_policy_v1.md`
  - `docs/abi/resource_pressure_events_v1.md`
- Add tests:
  - `tests/runtime/test_memory_pressure_docs_v1.py`

### Primary files

- `docs/runtime/memory_pressure_contract_v1.md`
- `docs/storage/io_qos_policy_v1.md`
- `docs/abi/resource_pressure_events_v1.md`
- `tests/runtime/test_memory_pressure_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_memory_pressure_docs_v1.py -v`

### Done criteria for PR-1

- Memory-pressure, I/O QoS, and pressure-event semantics are explicit and
  versioned.
- OOM and deferred-pressure paths remain deterministic and reviewable.

## PR-2: Reclaim and QoS Campaign Baseline

### Objective

Implement deterministic evidence for reclaim, OOM policy, and I/O QoS
behavior.

### Scope

- Add tooling:
  - `tools/run_memory_pressure_v1.py`
  - `tools/run_io_qos_campaign_v1.py`
- Add tests:
  - `tests/runtime/test_memory_reclaim_v1.py`
  - `tests/storage/test_io_qos_v1.py`
  - `tests/runtime/test_oom_policy_v1.py`
  - `tests/runtime/test_memory_pressure_negative_v1.py`

### Primary files

- `tools/run_memory_pressure_v1.py`
- `tools/run_io_qos_campaign_v1.py`
- `tests/runtime/test_memory_reclaim_v1.py`
- `tests/storage/test_io_qos_v1.py`
- `tests/runtime/test_oom_policy_v1.py`
- `tests/runtime/test_memory_pressure_negative_v1.py`

### Acceptance checks

- `python tools/run_memory_pressure_v1.py --out out/memory-pressure-v1.json`
- `python tools/run_io_qos_campaign_v1.py --out out/io-qos-v1.json`
- `python -m pytest tests/runtime/test_memory_reclaim_v1.py tests/storage/test_io_qos_v1.py tests/runtime/test_oom_policy_v1.py tests/runtime/test_memory_pressure_negative_v1.py -v`

### Done criteria for PR-2

- Pressure and QoS artifacts are deterministic and machine-readable.
- `MEM: reclaim ok` and I/O QoS markers are stable across seeded runs.
- Later telemetry and reliability work can reference explicit pressure policy
  IDs.

## PR-3: Memory Pressure Gate + I/O QoS Sub-gate

### Objective

Make memory-pressure and I/O QoS behavior release-blocking for declared system
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-memory-pressure-v1`
  - `Makefile` target `test-io-qos-v1`
- Add CI steps:
  - `Memory pressure v1 gate`
  - `I/O qos v1 gate`
- Add aggregate tests:
  - `tests/runtime/test_memory_pressure_gate_v1.py`
  - `tests/storage/test_io_qos_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_memory_pressure_gate_v1.py`
- `tests/storage/test_io_qos_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-memory-pressure-v1`
- `make test-io-qos-v1`

### Done criteria for PR-3

- Memory-pressure and I/O QoS sub-gates are required in local and CI release
  lanes.
- M79 can be marked done only with deterministic reclaim, OOM, and I/O QoS
  evidence for declared profiles.

## Non-goals for M79 backlog

- observability and performance telemetry work owned by M80
- chaos and fuzz qualification owned by M81
- unbounded autotuning outside declared pressure budgets
