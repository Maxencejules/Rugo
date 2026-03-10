# Process Model v4

Date: 2026-03-10  
Milestone: M41 Process + Readiness Compatibility Closure v1  
Status: active release gate

## Objective

Define deterministic process lifecycle and signal semantics required by
`compat_profile_v5`.

## Contract identifiers

- Process model contract ID: `rugo.process_model.v4`
- Parent compatibility profile ID: `rugo.compat_profile.v5`
- Surface campaign schema: `rugo.compat_surface_campaign_report.v2`

## Lifecycle model

States:

- `new`
- `ready`
- `running`
- `blocked`
- `zombie`
- `reaped`

Required transitions:

- `new -> ready`: admitted by scheduler.
- `ready -> running`: selected by scheduler.
- `running -> blocked`: waits on resource or syscall boundary.
- `blocked -> ready`: wait condition resolves.
- `running -> zombie`: process exits or receives terminal signal.
- `zombie -> reaped`: parent consumes child result via wait path.

## Required v4 checks

- `process_spawn_exec`: spawn-to-ready latency and deterministic startup.
- `process_wait_reap_once`: wait result consumable exactly once.
- `process_signal_fifo`: non-terminal signals preserve FIFO delivery.
- `process_sigkill_terminal`: `SIGKILL` transitions running task to terminal
  state with deterministic bounded latency.
- `process_pid_reuse_guard`: no duplicate live PID allocation during campaign.

Thresholds:

- spawn-to-ready latency: `<= 130 ms`
- wait/reap latency: `<= 22 ms`
- signal reorder events: `<= 1`
- SIGKILL terminal latency: `<= 35 ms`
- PID reuse guard violations: `<= 0`

## Deferred surface policy

- `fork` and `clone` parity remain deferred in v4.
- Deferred process APIs must fail deterministically with `-1`, `ENOSYS`.
- Deferred behavior is release-blocking if it becomes non-deterministic.

## Tooling and gate wiring

- Campaign runner: `tools/run_compat_surface_campaign_v2.py`
- Local gate: `make test-process-readiness-parity-v1`
- Local sub-gate: `make test-posix-gap-closure-v2`
- CI gate: `Process readiness parity v1 gate`
- CI sub-gate: `POSIX gap closure v2 gate`
