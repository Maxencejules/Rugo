# Init Contract v2

Date: 2026-03-09  
Milestone: M25 Userspace Service Model + Init v2  
Init Contract ID: `rugo.init_contract.v2`  
Boot graph schema: `rugo.init_boot_graph.v2`  
Operational state schema: `rugo.init_operational_state.v2`

## Objective

Pin init behavior so boot-to-operational sequencing is deterministic and
release-testable.

## Boot phases

Phase order: `bootstrap -> core -> services -> operational`.

- `bootstrap`: validate manifest and dependency graph.
- `core`: start required platform services.
- `services`: start declared non-core services in deterministic order.
- `operational`: reached only after all `critical` services are `running`.

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

## Evidence and enforcement

- Lifecycle policy source: `docs/runtime/service_model_v2.md`
- Local gate: `make test-userspace-model-v2`
- CI gate: `Userspace model v2 gate`

Required evidence tests:

- `tests/runtime/test_service_model_docs_v2.py`
- `tests/runtime/test_service_lifecycle_v2.py`
- `tests/runtime/test_service_dependency_order_v2.py`
- `tests/runtime/test_restart_policy_v2.py`
