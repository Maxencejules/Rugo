# Service Model v2

Date: 2026-03-19
Milestone: M25 Userspace Service Model + Init v2
Service Model ID: `rugo.service_model.v2`
Dependency order schema: `rugo.service_dependency_order.v2`
Lifecycle report schema: `rugo.service_lifecycle_report.v2`
Restart report schema: `rugo.restart_policy_report.v2`

## Purpose

Define deterministic userspace service lifecycle, readiness, restart, and
shutdown behavior for the default Rust-kernel plus Go-userspace lane.

The live default Go lane exercises this model with `timesvc`, `diagsvc`,
`pkgsvc`, and `shell` on the real `go_test` boot path.

## Lifecycle states

Services move through these states only:

- `declared`
- `blocked` (waiting for dependencies)
- `starting`
- `running`
- `ready`
- `failed`
- `stopping`
- `stopped`

Interpretation:

- `running` means the task is alive inside its init path.
- `ready` means the service contract is usable by dependents.
- Dependencies are satisfied only when upstream services are `ready`.

Any transition outside this state machine is a contract violation.

## Startup and shutdown order

Deterministic startup rule: topological dependency order, then lexical service
name.

Deterministic shutdown rule: reverse startup order after session exit.

Dependency cycles are invalid and must fail plan generation before service
activation begins.

## Failure propagation rules

- If a dependency is not `ready`, dependent services remain `blocked`.
- Failure of a required dependency blocks `operational` state.
- Failure of an optional service does not block `operational` state.

The default manifest keeps `timesvc`, `diagsvc`, and `shell` required while
`pkgsvc` is optional on the default lane.

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

## Runtime service contract fields

Each declared service may also carry:

- a startup phase (`core`, `base`, or `session`)
- a bounded startup budget before the manager declares the service wedged
- an optional stop command for controlled shutdown
- an explicit required or optional boot class

The default Go lane uses those fields to:

- reach `operational` only after the required base services are `ready`
- hold `shell` until the required base services report `ready`
- emit deterministic wedge markers instead of waiting forever in `starting`
- apply per-service scheduler class through `sys_sched_set`
- request orderly shutdown of remaining services after the shell completes
- emit `GOSVCM: phase shutdown` before ordered teardown begins
- expose kernel-backed task snapshots through `diagsvc` and `sys_proc_info`
- enforce a storage-only isolation profile on the shipped `pkgsvc` path

## Service result reporting

Lifecycle state and service result are tracked separately.

Representative result tokens:

- `online`
- `runtime-failed`
- `spawn-failed`
- `wedge`
- `restarting`
- `ordered-stop`
- `session-done`
- `shutdown-error`
- `restart-exhausted`

The manager emits terminal reap markers with `res=` outcome context, and
diagnostic snapshots now carry both `svc=` lifecycle and `res=` result fields.

## Enforcement

- Local gate: `make test-userspace-model-v2`
- CI gate: `Userspace model v2 gate`

Required M25 checks:

- `tests/runtime/test_service_model_docs_v2.py`
- `tests/runtime/test_service_lifecycle_v2.py`
- `tests/runtime/test_service_boot_runtime_v2.py`
- `tests/runtime/test_service_dependency_order_v2.py`
- `tests/runtime/test_restart_policy_v2.py`
- `tests/runtime/test_userspace_model_gate_v2.py`

Runtime-backed default-lane evidence:

- `tests/runtime/test_service_boot_runtime_v2.py` boots `make image-go` and
  verifies manifest-driven lifecycle markers from the real TinyGo init/service
  path rather than only deterministic models.
- The live boot path reaps exited service tasks through `sys_wait`, exercises
  bounded restart on the default shell service before the successful run reaches
  `ready`, and emits per-service `res=` outcome markers.
- The same boot path launches `diagsvc`, services a live diagnostic request
  from `shell`, and performs bounded stop control on the remaining services.
- The same boot path launches `pkgsvc` by default, serves a live package/update
  flow when present, and treats package-service failure as optional instead of a
  release-lane boot blocker.
