# Per-CPU run queues — contract v1

Status: boot-verified via `make test-smp-runtime-v1` (both `-smp 4` and `-smp 2`)
Source: `kernel_rs/src/smp.rs` (`ap_poll_rq`, `rq_enqueue`, `ap_runqueue_selftest`,
`ap_affinity_selftest`, `AP_RQ_*`; the AP park loops poll their own queue).
Proof: `tests/runtime/test_smp_runtime_v1.py`.

Full-OS guide Part I.3 (SMP scheduler): give **each CPU its own run queue** that
it drains independently — the scheduler data structure that follows the single
work mailbox ([`ap_work_v1.md`](ap_work_v1.md)) and the ring-3-task-on-an-AP
capstone ([`smp_user_task_v1.md`](smp_user_task_v1.md)).

## Behaviour

Each CPU `c` has a private queue (`AP_RQ_KIND[c]`/`AP_RQ_ARG[c]`, `RQ_LEN = 8`)
plus per-CPU `COUNT`/`DONE`/`SUM` atomics:

- **`rq_enqueue(cpu, items)`** (BSP): fills `cpu`'s queue, resets its done/sum
  accumulators, and publishes the item count **last** (so the consumer never sees
  a partially-filled queue).
- **`ap_poll_rq`** (each AP, in its park loop): reads its **own** slot (via the
  GS-based per-CPU index — no cross-CPU lookup or locking), and drains any newly
  enqueued items, running each on its own core and folding the result into its
  per-CPU `SUM`. Only that CPU writes its `DONE`/`SUM`; only the BSP writes its
  `KIND`/`ARG`/`COUNT` before publishing — so the queue is lock-free.

This is the genuine per-CPU run-queue model (vs the single shared mailbox where
one item goes to whichever AP grabs it): N CPUs drain N independent queues
concurrently, each maintaining its own state.

## Acceptance

`make test-smp-runtime-v1`: on both the base `os.iso` `-smp 4` lane and the go
`os-go.iso` `-smp 2` lane, the BSP gives every online AP its own 3-item queue
(sum 1..=100, 1..=200, 1..=300), and the transcript shows `SMP: runqueue ok` —
every AP drained its own queue concurrently and accumulated exactly 70300 — with
no `SMP: runqueue FAIL`.

It also shows `SMP: affinity ok` (`ap_affinity_selftest`): the BSP routes a
**distinct** workload to each core (CPU *s* gets a queue keyed off its slot) and
verifies each core drained exactly **its own** work (a core running the wrong
queue would produce the wrong sum) plus that the grand total across cores matches
— so the whole batch was distributed with nothing lost or double-run. This is the
**per-CPU affinity / load-distribution** step (vs the uniform-broadcast run-queue
test): the basis for a load-balancing scheduler.

## v1 boundary / carry-forward

- The queues hold **kernel work items** (the proven per-CPU execution primitive),
  and the BSP can now **route distinct work to a chosen core** (affinity) and
  distribute a batch across cores. Migrating actual R4 tasks onto per-CPU queues
  with a per-CPU `current`, work stealing between queues, and running the
  scheduler tick on each AP (so ordinary user tasks are scheduled across all
  CPUs, not just the boot self-test) is the remaining SMP-scheduler work — it
  builds directly on these queues plus the per-CPU TSS + ring-3-entry capstone +
  the real-R4-task-migration ([`smp_syscall_v1.md`](smp_syscall_v1.md)).
- Fixed 8-slot queues; a growable/linked run queue is carry-forward.
