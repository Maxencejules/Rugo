# Compatibility Profile v5

Date: 2026-03-10  
Milestone: M41 Process + Readiness Compatibility Closure v1  
Status: active release gate

## Purpose

Advance process/readiness compatibility closure for mainstream userspace
workloads while keeping deferred surfaces explicit and deterministically
unsupported.

## Contract identity

- Compatibility profile identifier: `rugo.compat_profile.v5`.
- Process model contract ID: `rugo.process_model.v4`.
- Readiness I/O contract ID: `rugo.readiness_io_model.v1`.
- POSIX gap report schema: `rugo.posix_gap_report.v2`.
- Compatibility surface campaign schema: `rugo.compat_surface_campaign_report.v2`.

## Profile classes

- `required`: release-blocking M41 process/readiness commitments.
- `optional`: bounded compatibility surfaces that may graduate later.
- `deferred`: explicitly unsupported in v5 and required to fail
  deterministically.

## Required surfaces (`required`)

### Process lifecycle and signal behavior

- `waitid`, `sigprocmask`, `sigpending`.
- deterministic spawn/wait/signal behavior bound to `process_model_v4`.

### Readiness and wait-path behavior

- `poll`, `pselect`, `ppoll`, `eventfd`.
- deterministic readiness checks through `run_compat_surface_campaign_v2.py`.

### POSIX/socket message path

- `sendmsg`, `recvmsg`, `socketpair`.
- deterministic report coverage through `run_posix_gap_report_v2.py`.

## Optional surfaces (`optional`)

- `accept4`
- `clock_nanosleep`
- `signalfd`
- `timerfd_create`

## Deferred surfaces (`deferred`)

The following remain deferred in v5 and are release-blocking if they stop
failing deterministically:

- `fork`
- `clone`
- `epoll`
- `io_uring`
- namespace/cgroup compatibility
- `AF_NETLINK` and raw packet socket parity

Deferred calls must preserve deterministic unsupported behavior (`-1`,
`ENOSYS`) in M41 campaign outputs.

## Gate requirements

- Process/readiness campaign command:
  - `python tools/run_compat_surface_campaign_v2.py --out out/compat-surface-v2.json`
- POSIX gap closure command:
  - `python tools/run_posix_gap_report_v2.py --out out/posix-gap-report-v2.json`
- Local gate: `make test-process-readiness-parity-v1`.
- Local sub-gate: `make test-posix-gap-closure-v2`.
- CI gate: `Process readiness parity v1 gate`.
- CI sub-gate: `POSIX gap closure v2 gate`.

Gate pass requires:

- process/readiness compatibility campaign `gate_pass = true`
- POSIX gap report `gate_pass = true`
- deferred-surface deterministic failure policy remains satisfied

## Related contracts

- `docs/runtime/syscall_coverage_matrix_v4.md`
- `docs/abi/process_model_v4.md`
- `docs/abi/readiness_io_model_v1.md`
- `docs/abi/compat_profile_v4.md`
