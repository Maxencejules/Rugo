# Go Userspace Bootstrap v1

Date: 2026-03-19
Status: active default-lane boot contract
Contract ID: `rugo.go_boot_contract.v1`

## Goal

Make the default Rust-kernel plus Go-userspace lane boot like a small,
deterministic base OS instead of a demo-shaped launch sequence.

## Current Audit

Kernel-to-userspace handoff was already real:

1. `kmain` logs `RUGO: boot ok`, enables paging, initializes descriptor tables,
   maps the single Go user image, seeds task `0`, and enters ring 3.
2. `goinit` is the first userspace task and obtains the kernel-provided spawn
   entry through `sysSpawnEntry`.
3. `gosvcm` already existed as the default Go service manager with a manifest,
   restart loop, shutdown path, and diagnostic snapshots.

The gaps were mostly contract clarity:

- service roles were implicit in code, not declared as part of boot policy
- `running` doubled as both "task is alive" and "service is usable"
- shell launch lived in the same broad phase as the supporting base services
- restart budgets and required-service behavior were present but implicit
- serial output showed state changes, but not the boot plan or phase contract

## Boot Contract

Boot phases on the default lane:

1. `kernel entry`
   `kmain` completes early kernel init, maps the Go image, seeds task `0`, and
   transfers to userspace.
2. `goinit bootstrap`
   `goinit` validates the kernel handoff, builds the deterministic start plan,
   and starts `gosvcm`.
3. `gosvcm contract declaration`
   `gosvcm` declares every default service and logs its role, phase,
   dependency set, restart policy, and required or optional class.
4. `core phase`
   `timesvc` must reach `ready`.
5. `base phase`
   `diagsvc` must reach `ready`; `pkgsvc` is launched here as an optional base
   service.
6. `session phase`
   `shell` starts only after all required base services are `ready`.
7. `bounded shutdown`
   After shell completion, `gosvcm` stops services in reverse dependency order
   and reaps every child before `GOINIT: ready`.

## Default Service Roles

- `goinit`: first userspace task and bootstrap coordinator
- `gosvcm`: service manager for the default manifest
- `timesvc`: required time service, critical scheduler class
- `diagsvc`: required diagnostics and operator snapshot service
- `pkgsvc`: optional package or platform state service for the default lane
- `shell`: required session entrypoint, started after base services are ready

## Lifecycle Model

Service lifecycle states on the default lane:

- `declared`
- `blocked`
- `starting`
- `running`
- `ready`
- `failed`
- `stopping`
- `stopped`

Interpretation:

- `running` means the task is alive inside its init path
- `ready` means the service contract is usable by dependents
- `failed` means bring-up or runtime failed and restart policy applies
- `stopped` means ordered shutdown completed before reap

The manager also emits reap markers with terminal context such as
`GOSVCM: reap shell failed` and `GOSVCM: reap timesvc stopped`.

## Dependency And Restart Policy

Default manifest:

- `timesvc`: no dependencies, `phase=core`, `required`, `restart=on-failure/3`
- `diagsvc`: depends on `timesvc`, `phase=base`, `required`, `restart=on-failure/3`
- `pkgsvc`: depends on `timesvc`, `phase=base`, `optional`, `restart=on-failure/3`
- `shell`: depends on `timesvc` and `diagsvc`, `phase=session`, `required`,
  `restart=on-failure/2`

Policy rules:

- required services exhaust their restart budget => boot fails
- optional services may fail closed without failing the boot contract
- scheduler class and requiredness are separate:
  `timesvc` is `critical`; the other default services are `best-effort`

## Serial Contract

Key markers on the default lane:

- `GOSVCM: plan ...`
- `GOSVCM: phase core`
- `TIMESVC: ready`
- `SVC: timesvc ready`
- `GOSVCM: phase base`
- `DIAGSVC: ready`
- `PKGSVC: ready`
- `GOINIT: operational`
- `GOSVCM: phase session`
- `SVC: shell ready`
- `GOSVCM: phase shutdown`
- `GOSVCM: reap ... failed|stopped`
- `GOINIT: result shutdown-clean`
- `GOINIT: ready`

These markers are intentionally small and serial-friendly. They are sufficient
to prove deterministic ordering, readiness, restart behavior, and shutdown on
the public default lane without inventing a larger orchestration framework.

## Validation

Primary commands:

```bash
mingw32-make image-demo
mingw32-make boot-demo
mingw32-make smoke-demo
python -m pytest tests/go/test_go_user_service.py tests/runtime/test_service_boot_runtime_v2.py tests/runtime/test_process_scheduler_runtime_v2.py tests/runtime/test_service_control_runtime_v1.py -v
```

Fixture-backed tooling checks:

```bash
python tools/collect_booted_runtime_v1.py --fixture --out out/booted-runtime-v1-bootcontract.json
python tools/run_perf_baseline_v1.py --runtime-capture out/booted-runtime-v1-bootcontract.json --out out/perf-baseline-v1-bootcontract.json
```

## Next Step

The next bounded improvement is failure-injection coverage for the optional
`pkgsvc` degraded boot path, so the non-blocking package-service policy is
proven by live runtime evidence instead of by contract alone.
