# Service Model v2

Date: 2026-03-09  
Milestone: M25 Userspace Service Model + Init v2  
Service Model ID: `rugo.service_model.v2`  
Dependency order schema: `rugo.service_dependency_order.v2`  
Lifecycle report schema: `rugo.service_lifecycle_report.v2`  
Restart report schema: `rugo.restart_policy_report.v2`

## Purpose

Define deterministic userspace service lifecycle behavior for startup, shutdown,
failure handling, and restart decisions.

## Lifecycle states

Services move through these states only:

- `declared`
- `blocked` (waiting for dependencies)
- `starting`
- `running`
- `failed`
- `stopping`
- `stopped`

Any transition outside this state machine is a contract violation.

## Startup and shutdown order

Deterministic startup rule: topological dependency order, then lexical service
name.

Deterministic shutdown rule: reverse startup order.

Dependency cycles are invalid and must fail plan generation before service
activation begins.

## Failure propagation rules

- If a dependency is not `running`, dependent services remain `blocked`.
- Failure of a `critical` dependency blocks `operational` state.
- Failure of a `best-effort` service does not block `operational` state.

## Restart policy contract

Supported policies:

- `never`
- `on-failure`
- `always`

Bounded restart controls:

- Maximum restart attempts per window: `3`.
- Restart window seconds: `60`.
- Backoff sequence seconds: `1, 2, 4`.

If the cap is reached, the service transitions to `failed` and remains stopped
until a manual recovery action.

## Enforcement

- Local gate: `make test-userspace-model-v2`
- CI gate: `Userspace model v2 gate`

Required M25 checks:

- `tests/runtime/test_service_model_docs_v2.py`
- `tests/runtime/test_service_lifecycle_v2.py`
- `tests/runtime/test_service_dependency_order_v2.py`
- `tests/runtime/test_restart_policy_v2.py`
- `tests/runtime/test_userspace_model_gate_v2.py`
