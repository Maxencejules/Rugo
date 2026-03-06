# M25 Execution Backlog (Userspace Service Model + Init v2)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Stabilize service lifecycle semantics (init/service/dependency handling) for
deterministic multi-purpose operation.

M25 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Service model foundations exist from earlier milestones.
- M25 formalizes v2 lifecycle, restart, and dependency-order semantics.
- Dedicated userspace model v2 gate is pending.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Service/Init Contract v2

### Objective

Freeze service model and init contract before deeper lifecycle assertions.

### Scope

- Add docs:
  - `docs/runtime/service_model_v2.md`
  - `docs/runtime/init_contract_v2.md`
- Add tests:
  - `tests/runtime/test_service_model_docs_v2.py`

### Primary files

- `docs/runtime/service_model_v2.md`
- `docs/runtime/init_contract_v2.md`
- `tests/runtime/test_service_model_docs_v2.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_service_model_docs_v2.py -v`

### Done criteria for PR-1

- Service/init contracts are explicit, versioned, and test-referenced.

## PR-2: Lifecycle + Dependency Semantics

### Objective

Enforce deterministic startup/shutdown/restart/failure behavior.

### Scope

- Add tests:
  - `tests/runtime/test_service_lifecycle_v2.py`
  - `tests/runtime/test_service_dependency_order_v2.py`
  - `tests/runtime/test_restart_policy_v2.py`

### Primary files

- `tests/runtime/test_service_lifecycle_v2.py`
- `tests/runtime/test_service_dependency_order_v2.py`
- `tests/runtime/test_restart_policy_v2.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_service_lifecycle_v2.py tests/runtime/test_service_dependency_order_v2.py tests/runtime/test_restart_policy_v2.py -v`

### Done criteria for PR-2

- Boot-to-operational state is deterministic.
- Failure and restart policies are executable and bounded.

## PR-3: Userspace Model Gate + Closure

### Objective

Make userspace service model v2 release-blocking.

### Scope

- Add local gate:
  - `Makefile` target `test-userspace-model-v2`
- Add CI step:
  - `Userspace model v2 gate`
- Add aggregate test:
  - `tests/runtime/test_userspace_model_gate_v2.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_userspace_model_gate_v2.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-userspace-model-v2`

### Done criteria for PR-3

- Userspace model v2 gate is required in local and CI lanes.
- M25 can be marked done with evidence pointers.

## Non-goals for M25 backlog

- Full service-manager feature parity with large distributions.
- Unbounded dependency graph complexity without contract updates.

