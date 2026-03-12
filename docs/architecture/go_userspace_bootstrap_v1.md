# Go Userspace Bootstrap v1

Date: 2026-03-11  
Status: canonical demo path  
Architecture ID: `rugo.go_userspace_bootstrap.v1`

## Goal

Make the default Go lane prove a real boot-to-userspace story instead of a
single marker binary.

## Current bootstrap shape

- Kernel mechanisms stay in Rust under `kernel_rs/`.
- Userspace policy stays in Go under `services/go/`.
- `go_test` is the canonical TinyGo-first demo lane.
- The current bootstrap is one Go image loaded by the kernel, using the
  existing R4 cooperative task, IPC, and service-registry kernel path.

Current task graph:

1. `goinit` is the first Go task entered from the kernel.
2. `gosvcm` is spawned by `goinit` and acts as the service manager.
3. `timesvc` is launched by `gosvcm`, creates an IPC endpoint, and registers
   itself through `sys_svc_register`.
4. `diagsvc` is launched after `timesvc`, exposes a small diagnostic/control
   endpoint, and reports live service-manager accounting from the default lane.
5. `gosh` is launched after service registration, resolves `timesvc` and
   `diagsvc` through `sys_svc_lookup`, exercises policy-denial paths, requests
   time service work, consumes a diagnostic snapshot, and then drives bounded
   shutdown.

This is intentionally a bootstrap-grade userspace stack, not a fake simulation:
the shell-to-service path crosses the kernel syscall boundary for registry,
endpoint creation, IPC, scheduling, and time reads.

## Why this is credible

- `goinit`, `gosvcm`, `gosh`, and `timesvc` are all real Go code paths.
- Boot sequencing is deterministic and visible on serial.
- The service uses actual kernel syscalls:
  - `sys_ipc_endpoint_create`
  - `sys_svc_register`
  - `sys_svc_lookup`
  - `sys_ipc_send`
  - `sys_ipc_recv`
  - `sys_time_now`
  - `sys_thread_spawn`
  - `sys_thread_exit`
  - `sys_yield`
- The kernel still owns all mechanism:
  task slots, IPC buffering, registry validation, context switching, and halt.

## Current limits

- This is a single-address-space bootstrap image, not a full multiprocess
  `spawn+exec` userspace.
- The current TinyGo lane still relies on a bounded bootstrap image mapped into
  four contiguous 4 KiB user pages (`16 KiB` total).
- The stack is cooperative because it uses the existing R4 task model.

## Serial contract

Expected boot markers for the canonical path:

- `GOINIT: start`
- `GOINIT: svcmgr up`
- `GOSVCM: start`
- `TIMESVC: start`
- `TIMESVC: ready`
- `DIAGSVC: start`
- `DIAGSVC: ready`
- `GOSVCM: shell`
- `GOSH: start`
- `GOSH: lookup ok`
- `TIMESVC: req ok`
- `TIMESVC: time ok`
- `DIAGSVC: snapshot`
- `GOSH: diag ok`
- `GOSH: reply ok`
- `GOINIT: ready`

The QEMU acceptance gate is `tests/go/test_go_user_service.py`.

## Demo and verification

Demo command:

```bash
make demo-go
```

Targeted validation:

```bash
make image-go
python -m pytest tests/go/test_go_user_service.py -v
```
