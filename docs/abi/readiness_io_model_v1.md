# Readiness I/O Model v1

Date: 2026-03-10  
Milestone: M41 Process + Readiness Compatibility Closure v1  
Status: active release gate

## Objective

Define deterministic readiness/wait-path semantics required by
`compat_profile_v5` with explicit deferred behavior for `epoll` and
`io_uring`.

## Contract identifiers

- Readiness I/O model contract ID: `rugo.readiness_io_model.v1`
- Parent compatibility profile ID: `rugo.compat_profile.v5`
- Surface campaign schema: `rugo.compat_surface_campaign_report.v2`
- POSIX gap report schema: `rugo.posix_gap_report.v2`

## Required checks

- `readiness_poll_wakeup`: `poll` wakeup latency under bounded load.
- `readiness_ppoll_wakeup`: `ppoll` wakeup latency under bounded load.
- `readiness_pselect_wakeup`: `pselect` wakeup latency under bounded load.
- `readiness_eventfd_signal`: `eventfd` wake-to-consume latency.
- `deferred_epoll_enosys`: deferred `epoll` path remains deterministic.
- `deferred_io_uring_enosys`: deferred `io_uring` path remains deterministic.

Thresholds:

- poll wakeup latency: `<= 11 ms`
- ppoll wakeup latency: `<= 10 ms`
- pselect wakeup latency: `<= 10 ms`
- eventfd signal latency: `<= 7 ms`
- deferred `epoll` deterministic `ENOSYS` ratio: `>= 1.0`
- deferred `io_uring` deterministic `ENOSYS` ratio: `>= 1.0`

## Deferred behavior policy

- `epoll` and `io_uring` are deferred in M41 and must fail with deterministic
  `-1`, `ENOSYS`.
- Any non-deterministic deferred outcome is release-blocking.
- Deferred status is tracked in both campaign and POSIX gap artifacts.

## Tooling and gate wiring

- Campaign runner: `tools/run_compat_surface_campaign_v2.py`
- POSIX gap report runner: `tools/run_posix_gap_report_v2.py`
- Local gate: `make test-process-readiness-parity-v1`
- Local sub-gate: `make test-posix-gap-closure-v2`
- CI gate: `Process readiness parity v1 gate`
- CI sub-gate: `POSIX gap closure v2 gate`
