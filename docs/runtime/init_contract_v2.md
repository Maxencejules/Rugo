# Init Contract v2

Date: 2026-03-19
Milestone: M25 Userspace Service Model + Init v2
Init Contract ID: `rugo.init_contract.v2`
Boot graph schema: `rugo.init_boot_graph.v2`
Operational state schema: `rugo.init_operational_state.v2`

## Objective

Pin init behavior so boot, session bring-up, and shutdown sequencing remain
deterministic and release-testable on the default lane.

## Boot phases

Phase order: `bootstrap -> core -> services -> operational -> session -> shutdown -> ready`.

- `bootstrap`: validate manifest and dependency graph.
- `core`: start required platform services.
- `services`: start declared non-core base services in deterministic order.
- `operational`: reached only after all required base services are `ready`.
- `session`: start the default shell session after the required base services
  are `ready`.
- `shutdown`: perform ordered reverse teardown after session completion.
- `ready`: reached only after init records a final result and all required
  services are reaped cleanly.

## Service classes

- Required class: `critical`.
- Optional class: `best-effort`.

Failure policy: failure of a `critical` service blocks `operational`.

`best-effort` failures are reported but do not block transition to
`operational`.

## Determinism rules

- Determinism rule: identical manifests must produce identical start/shutdown
  plans.
- Cycle policy: dependency cycles are release-blocking configuration errors.
- Missing dependencies are release-blocking configuration errors.

## Timing budget

Boot-to-operational timeout budget: `45s`.

If timeout is exceeded, init must emit a failure report and leave the system in
non-operational state.

## Final result contract

The init process owns the final boot outcome and emits:

- `GOINIT: result shutdown-clean`
- `GOINIT: result boot-failed`
- `GOINIT: result session-failed`
- `GOINIT: result shutdown-failed`

That result line is emitted before `GOINIT: ready` on success and before
`GOINIT: err` on failure.

## Evidence and enforcement

- Lifecycle policy source: `docs/runtime/service_model_v2.md`
- Local gate: `make test-userspace-model-v2`
- CI gate: `Userspace model v2 gate`

Required evidence tests:

- `tests/runtime/test_service_model_docs_v2.py`
- `tests/runtime/test_service_lifecycle_v2.py`
- `tests/runtime/test_service_boot_runtime_v2.py`
- `tests/runtime/test_service_dependency_order_v2.py`
- `tests/runtime/test_restart_policy_v2.py`

Runtime-backed boot evidence:

- `tests/runtime/test_service_boot_runtime_v2.py` boots the default Go lane and
  verifies that `bootstrap -> core -> services -> operational -> session -> shutdown -> ready`
  is exercised by the real service manager rather than only by model tests.
- `tests/runtime/test_process_scheduler_runtime_v2.py` verifies that the same
  init path blocks in `sys_wait`, reaps child services, performs bounded
  restart on the live booted system, and emits the explicit shutdown phase.
- `tests/runtime/test_service_control_runtime_v1.py` verifies that the same
  boot path applies scheduler class to spawned services and exposes kernel task
  snapshots through `diagsvc`.
