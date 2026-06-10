# Default-Lane Preemptive Scheduling Contract v1

Status: live runtime (boot-verified)
Source: `kernel_rs/src/lib.rs` (`r4_timer_preempt`), `kernel_rs/src/sched.rs`
(PIC/PIT helpers), `kernel_rs/src/arch_x86.rs` (`enter_ring3_preemptible`)
Proof: `make test-sched-preempt-v1`, `tests/sched/test_preempt_default_lane_v1.py`

This contract closes gap-analysis build-list item 3
(`docs/analysis/full-os-gap-analysis.md` §2.2/§3): the PIT/PIC path is no
longer `sched_test`-only — the default Go product lane preempts user tasks.

## Kernel mechanics

- The go-lane boot remaps the 8259A PICs to vectors 32..47, masks every line
  except IRQ0, programs the PIT at 100 Hz, and prints
  `SCHED: preempt on hz=100` before entering ring 3.
- User tasks run with RFLAGS.IF=1 (`enter_ring3_preemptible`; spawned tasks
  are seeded with IF in `r4_init_task`). Kernel code keeps IF=0 (interrupt
  gates, no `sti`), so handlers never nest and no kernel locking is needed
  on one CPU.
- Vector 32 lands in `r4_timer_preempt`: EOI first, then — only when the
  tick interrupted ring 3 and another task is Ready — the running task's
  full 22-slot frame is saved verbatim and the next Ready task is dispatched.
  Unlike the yield path, RAX is preserved: an involuntary switch is not a
  syscall return. The first such switch prints `SCHED: preempt hit` exactly
  once.
- The compat composition (`go_test` + `compat_real_test`) never programs the
  PIC, so it stays cooperative: IF is not seeded there and vector 32 is not
  routed to the preemptor.
- `qemu_exit` drains the UART transmitter and settles briefly before the
  debug-exit write, so fast exits cannot truncate the serial transcript.

## Userspace contract (preemption-safe init protocol)

The supervisor protocol in `services/go/` no longer assumes children run
only at yield points:

- **Spawn handshake**: a child latches its identity from `spawnServiceID`,
  acks, and holds at a per-service gate until the supervisor has applied
  isolation limits and the scheduling class. No task can act before its
  limits exist, and the identity handoff cannot be overwritten.
- **Log-before-publish**: `setServiceState` emits its `SVC:` line before
  storing the state byte; observers that react to the byte can never get
  their output ahead of the line that explains it.
- **Stop-before-send**: shutdown logs `GOSVCM: stop X`, publishes
  `stopping`, and only then delivers the stop IPC — a child that stops
  instantly can no longer have its `stopped` state overwritten.
- **Single-write log lines**: every multi-part line is composed in a
  stack `lineBuilder` and emitted with one `sys_debug_write`; lines are
  atomic on the wire.
- **Atomic allocator**: the TinyGo bump heap lives in the demand-paged
  window at `0x110_0000` (seeded while single-threaded, bumped with
  `lock xadd`); the kernel pre-maps window pages during syscall pointer
  validation so freshly allocated, never-touched buffers are valid syscall
  arguments.

## What is deliberately NOT deterministic

- The interleaving of sibling services' shutdown handling and the
  `sysWait` reap order across siblings depend on scheduling. Acceptance
  tests assert per-service causal chains and counts, never cross-sibling
  order.
- Sampled peer task states in `TASK:` snapshots (`st=`) are point-in-time
  reads.

## Marker contract

| Marker | Meaning |
|---|---|
| `SCHED: preempt on hz=100` | PIT programmed, default lane preemptive (once) |
| `SCHED: preempt hit` | first involuntary task switch (exactly once) |

All earlier `GOSVCM:`/`SVC:`/service markers keep their meaning; their
cross-task ordering guarantees are those enforced by the protocol above.
