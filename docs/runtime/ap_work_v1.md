# Cross-CPU work dispatch — contract v1

Status: boot-verified via `make test-smp-runtime-v1` (boots `-smp 4` and `-smp 2`)
Source: `kernel_rs/src/smp.rs` (`WORK_*` mailbox, `run_work`, `ap_poll_work`,
`smp_dispatch_work`, the AP park-loop poll, the `smp_init` self-test).
Proof: `tests/runtime/test_smp_runtime_v1.py`.

Full-OS implementation guide Part I.3 (SMP), the AP-execution slice — the step
that turns the application processors from "checked in and parked" into "running
real dispatched kernel work." It is the execution primitive a per-CPU scheduler
runs tasks on, built on the (already landed) spinlock, IPI, per-CPU LAPIC timer,
TLB shootdown, and per-CPU GS storage.

## Behaviour

A work item is a `(kind, arg)` pair in a small mailbox. Dispatch is generation-
counted and single-in-flight (v1):

- The BSP calls `smp_dispatch_work(kind, arg)`: it writes `WORK_KIND`/`WORK_ARG`,
  clears `WORK_DONE`, sets `WORK_CLAIM` to a fresh generation, and **publishes
  `WORK_GEN` last** (release) so any AP that observes the generation sees a
  consistent item. It then spins (bounded) until `WORK_DONE` reaches the
  generation and returns `WORK_RESULT`.
- Each AP, in its park loop, calls `ap_poll_work` after every wake (the periodic
  LAPIC timer wakes it): if `WORK_GEN` is nonzero it **claims** the item with a
  `compare_exchange(WORK_CLAIM, gen -> 0)` so **exactly one** AP runs it, then
  executes `run_work` on its own core, publishes `WORK_RESULT` (release), and
  sets `WORK_DONE = gen` (release).

Acquire/release ordering across `WORK_GEN`/`WORK_DONE` makes the hand-off
race-free: the claiming AP sees the item, and the BSP sees the result.

`run_work` kind 1 = sum `1..=arg`, computed iteratively (a real workload the
dispatcher independently checks: `sum(1..=1000) == 500500`).

## v1 boundary / carry-forward

- **One work kind, single in-flight item.** A real per-CPU run queue (many items,
  per-CPU queues, work stealing) and dispatching **user tasks** (not just kernel
  computations) is the capstone. Running user tasks on APs additionally needs
  per-CPU GDT/TSS (so an AP takes a ring3→ring0 trap on its own kernel stack) and
  making the scheduler/syscall/page-fault paths SMP-safe (they assume a single
  CPU with interrupts-off syscalls today) — a core change reserved for a
  dedicated effort.
- The AP picks work up on its next LAPIC-timer wake (sub-tick latency); a
  directed IPI wake is a later optimization.

## Acceptance

`make test-smp-runtime-v1`: booted `-smp 4`, the transcript shows
`SMP: ap work ok` after `SMP: percpu ok` (an AP claimed and ran the dispatched
`sum(1..=1000)` and the BSP verified `500500`); booted `-smp 2` (go lane), the
single AP runs the work (`SMP: ap work ok`) and the lane still reaches
`GOSH: session ready` and shuts down cleanly.
