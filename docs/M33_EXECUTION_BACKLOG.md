# M33 Execution Backlog (Fleet-Scale Operations Baseline v1)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Introduce controlled fleet-style operations with deterministic update
orchestration, rollback coordination, and canary rollout safety controls.

M33 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Fleet update and health policy contracts are explicit and versioned.
- Fleet and rollout tools now operate on a runtime-backed lab model derived from
  the booted default-lane capture.
- Fleet ops and rollout-safety checks are required local and CI release gates.

## Execution Result

- PR-1: complete (2026-03-09)
- PR-2: complete (2026-03-09)
- PR-3: complete (2026-03-09)

## PR-1: Fleet + Staged Rollout Contracts

### Objective

Freeze fleet health/update policy and canary rollout SLO criteria.

### Scope

- Add docs:
  - `docs/pkg/fleet_update_policy_v1.md`
  - `docs/runtime/fleet_health_policy_v1.md`
  - `docs/pkg/staged_rollout_policy_v1.md`
  - `docs/runtime/canary_slo_policy_v1.md`
- Add tests:
  - `tests/pkg/test_fleet_policy_docs_v1.py`
  - `tests/pkg/test_rollout_policy_docs_v1.py`

### Primary files

- `docs/pkg/fleet_update_policy_v1.md`
- `docs/runtime/fleet_health_policy_v1.md`
- `docs/pkg/staged_rollout_policy_v1.md`
- `docs/runtime/canary_slo_policy_v1.md`
- `tests/pkg/test_fleet_policy_docs_v1.py`
- `tests/pkg/test_rollout_policy_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/pkg/test_fleet_policy_docs_v1.py tests/pkg/test_rollout_policy_docs_v1.py -v`

### Done criteria for PR-1

- Fleet and rollout-safety policies are explicit, versioned, and test-referenced.

### PR-1 completion summary

- Added docs:
  - `docs/pkg/fleet_update_policy_v1.md`
  - `docs/runtime/fleet_health_policy_v1.md`
  - `docs/pkg/staged_rollout_policy_v1.md`
  - `docs/runtime/canary_slo_policy_v1.md`
- Added executable doc contract checks:
  - `tests/pkg/test_fleet_policy_docs_v1.py`
  - `tests/pkg/test_rollout_policy_docs_v1.py`

## PR-2: Fleet Simulation + Rollout Safety Drills

### Objective

Operationalize multi-node simulation and SLO-triggered rollback behavior.

### Scope

- Add tooling:
  - `tools/run_fleet_update_sim_v1.py`
  - `tools/run_fleet_health_sim_v1.py`
  - `tools/run_canary_rollout_sim_v1.py`
  - `tools/run_rollout_abort_drill_v1.py`
- Add tests:
  - `tests/pkg/test_fleet_update_sim_v1.py`
  - `tests/runtime/test_fleet_health_sim_v1.py`
  - `tests/pkg/test_canary_rollout_sim_v1.py`
  - `tests/runtime/test_rollout_abort_policy_v1.py`

### Primary files

- `tools/run_fleet_update_sim_v1.py`
- `tools/run_fleet_health_sim_v1.py`
- `tools/run_canary_rollout_sim_v1.py`
- `tools/run_rollout_abort_drill_v1.py`
- `tests/pkg/test_fleet_update_sim_v1.py`
- `tests/runtime/test_fleet_health_sim_v1.py`
- `tests/pkg/test_canary_rollout_sim_v1.py`
- `tests/runtime/test_rollout_abort_policy_v1.py`

### Acceptance checks

- `python -m pytest tests/pkg/test_fleet_update_sim_v1.py tests/runtime/test_fleet_health_sim_v1.py tests/pkg/test_canary_rollout_sim_v1.py tests/runtime/test_rollout_abort_policy_v1.py -v`

### Done criteria for PR-2

- Fleet/rollout simulations are runtime-backed and machine-readable.
- Report-driven abort and rollback behavior is auditable.

### PR-2 completion summary

- Added runtime-backed fleet and rollout tooling:
  - `tools/run_fleet_update_sim_v1.py`
  - `tools/run_fleet_health_sim_v1.py`
  - `tools/run_canary_rollout_sim_v1.py`
  - `tools/run_rollout_abort_drill_v1.py`
- Added executable simulation and rollback checks:
  - `tests/pkg/test_fleet_update_sim_v1.py`
  - `tests/runtime/test_fleet_health_sim_v1.py`
  - `tests/pkg/test_canary_rollout_sim_v1.py`
  - `tests/runtime/test_rollout_abort_policy_v1.py`
  - `tests/runtime/test_fleet_control_plane_v1.py`

## PR-3: Fleet Ops Gate + Rollout Safety Sub-gate

### Objective

Make fleet operations and rollout-safety checks release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-fleet-ops-v1`
  - `Makefile` target `test-fleet-rollout-safety-v1`
- Add CI steps:
  - `Fleet ops v1 gate`
  - `Fleet rollout safety v1 gate`
- Add aggregate tests:
  - `tests/runtime/test_fleet_ops_gate_v1.py`
  - `tests/runtime/test_fleet_rollout_safety_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_fleet_ops_gate_v1.py`
- `tests/runtime/test_fleet_rollout_safety_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-fleet-ops-v1`
- `make test-fleet-rollout-safety-v1`

### Done criteria for PR-3

- Fleet ops and rollout-safety gates are required in local and CI lanes.
- M33 can be marked done with runtime-backed rollout and rollback evidence.

### PR-3 completion summary

- Added aggregate gate tests:
  - `tests/runtime/test_fleet_ops_gate_v1.py`
  - `tests/runtime/test_fleet_rollout_safety_gate_v1.py`
- Added local gates:
  - `make test-fleet-ops-v1`
  - `make test-fleet-rollout-safety-v1`
  - JUnit output:
    - `out/pytest-fleet-ops-v1.xml`
    - `out/pytest-fleet-rollout-safety-v1.xml`
- Added CI gates and artifacts:
  - steps:
    - `Fleet ops v1 gate`
    - `Fleet rollout safety v1 gate`
  - artifacts:
    - `fleet-ops-v1-artifacts`
    - `fleet-rollout-safety-v1-artifacts`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M33 backlog

- Broad production fleet deployment outside the scoped runtime-backed lab model.
- Global rollout automation beyond the declared default-lane support surface.
