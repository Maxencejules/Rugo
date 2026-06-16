# AP services real syscalls + per-CPU current — contract v1

Status: boot-verified via `make test-smp-syscall-v1` (go lane, `-smp 2`)
Source: `kernel_rs/src/smp.rs` (`ap_run_user_task`, `ap_user_trap`,
`ap_user_selftest`, `PerCpu.current_task`), boot call after the run-queue
self-test in `smp_init`.
Proof: `tests/runtime/test_smp_syscall_v1.py` (plus the multicore assertions in
`tests/runtime/test_smp_runtime_v1.py`).

Full-OS guide Part I.3 (SMP scheduler). The capstone
([`smp_user_task_v1.md`](smp_user_task_v1.md)) made an application processor enter
ring 3, run a self-contained compute payload, and report a result. This advances
it toward a real multi-CPU scheduler: the AP now (a) tracks a **per-CPU `current`
task** through its own GS base, and (b) **services real syscalls** for that task on
its own core — the full ring-3 → ring-0 → ring-3 path on a second CPU.

## Behaviour

`PerCpu` gains a third GS-reachable slot, `current_task` at `gs:[16]` (after
`cpu_index` @0 and `timer_ticks` @8) — the field a per-CPU scheduler reads/writes
on every context switch with no lock and no "which CPU am I" lookup.

When the BSP dispatches the user task (work kind 2):

1. **`ap_run_user_task`** (on the AP) writes the dispatched task id into its own
   `gs:[16]` (`mov qword ptr gs:[16], tid`) before loading the task CR3 and
   `iretq`-ing to ring 3 — exactly the bookkeeping a scheduler does when it puts a
   task on a core.
2. The ring-3 task issues **two real `int 0x80` syscalls** (`sys_time_now`, op
   10). Each takes the ring-3 → ring-0 transition onto the **AP's own per-CPU TSS
   `rsp0`**, runs real kernel code (read + increment the monotonic tick), and
   returns to ring 3. The task reports the **delta** of the two ticks.
3. **`ap_user_trap`** (the `int 0x81` handler, running on the AP) reads `gs:[16]`
   back through GS, publishes it and the syscall delta, then clears `gs:[16]` (the
   task is leaving the core), restores the kernel CR3, and trampolines the AP back
   into kernel code on its own stack.
4. The BSP verifies: the reported `2*arg+1`, that an **AP** (slot ≥ 1) ran it,
   that the per-CPU `current` round-tripped (`== 0x5A`, the dispatched id), and
   that the **syscall tick delta is exactly 1** — only possible if the kernel
   serviced both `int 0x80`s on the AP.

`sys_time_now` is used rather than `sys_debug_write` deliberately:
`sys_debug_write` mirrors output to the framebuffer console, whose mapping lives
under PML4[0] — the half `address_space_create` replaces with the task's private
subtree — so it is **absent from the AP's address space** and would fault. A
non-printing syscall whose effect is observable via its return value proves the
same round-trip without depending on the framebuffer mapping.

## Acceptance

`make test-smp-syscall-v1`: the go lane boots with `-smp 2`; the transcript shows
`SMP: ap-syscall delta=0x0000000000000001` (two real syscalls serviced on the AP,
consecutive ticks differing by 1), `SMP: ap-current=0x000000000000005A` (the
per-CPU `current` set + read back on the AP), and `SMP: ap user task ok`, reaching
`GOINIT: result shutdown-clean` / `RUGO: halt ok` with no `USERPF` / `PF:` and no
`FAIL`.

## Real R4 task + a syscall that resolves the per-CPU current

`ap_r4_migrate_selftest` (`smp.rs`, run right after `ap_user_selftest`) goes a step
further: instead of a free-floating payload it registers a **real `R4_TASKS`
scheduler entry** via `r4_init_task` (its own `pml4_phys` address space + ring-3
`saved_frame` context) in a reserved slot (`R4_MAX_TASKS-1`, kept `Running` and
outside `R4_NUM_TASKS` so the BSP scheduler never races it), migrates *that* task's
CR3 + entry context to the AP, and publishes its **real tid** as the AP's per-CPU
`current`.

Its ring-3 payload (`AP_R4_CODE`) then issues a **real, user-facing syscall** that
reads its own identity from per-CPU state: **`getuid`** (`sys_proc_ctl` id 51, op 3),
which now reads `R4_TASKS[r4_current_smp()].uid`. `r4_current_smp()` returns
`R4_CURRENT` on the BSP but the AP's own `gs:[16]` on an application processor.
BSP-vs-AP is decided by **x2APIC ID** (`smp::is_bsp`, comparing the live
`IA32_X2APIC_APICID` to the BSP's, guarded by `SMP_AP_COUNT==0` so a uniprocessor
never reads the MSR) — *not* GS, because the BSP's GS base is deliberately left
unset (ring-3 TinyGo uses GS). The BSP stamps a sentinel uid (`0x77`) on the
migrated slot; the task's `getuid` returns it; the BSP confirms `scuid == 0x77`.
Marker: `SMP: ap r4 migrate tid=0x1F cur=0x1F scuid=0x77 ok` — `cur` is the per-CPU
current read back via GS, `scuid` is the uid a **real syscall resolved by indexing
the real task table through per-CPU current on the AP**. This is the per-CPU
`R4_CURRENT` reroute working end to end through a real syscall (not a test op),
indexing the real `R4_TASKS` fields, for a real scheduler task. The reroute is
transparent on the BSP — `getuid` there is byte-for-byte unchanged
(`r4_current_smp() == R4_CURRENT`), still covered by `test_userid_v1`. Asserted by
`tests/runtime/test_smp_runtime_v1.py`. (`sys_sysinfo` op 14 returns the resolved
tid via the same accessor.)

## Concurrent execution — two tasks, two CPUs, at once

`ap_r4_concurrent_selftest` proves genuine simultaneity. The BSP dispatches the AP
task **asynchronously** (`smp_dispatch_async` — it does *not* block) and then meets
it in a rendezvous: the ring-3 task on the AP issues `sys_sysinfo` op 15 →
`smp_rendezvous_ap`, which publishes "arrived" (`SMP_RV=1`) and spins *in kernel on
the AP* for the BSP's ack; the BSP, running concurrently, waits for the arrival,
stores `SMP_RV=2`, and joins. The handshake can only close if both CPUs are live at
the same instant — under the old synchronous dispatch the BSP would be blocked and
the AP's bounded wait would time out. The task returns `0xAC`; marker
`SMP: ap+bsp concurrent rv=0xAC ok`. This is the literal "multiple tasks running on
multiple CPUs": a real ring-3 task on an AP and the BSP executing at the same time.

## v1 boundary / carry-forward

- `getuid` (a real read syscall), `sys_sysinfo` op 14 (read) and op 16 (a write that
  bumps the caller's `yield_count`) now resolve `current` per-CPU (transparent on the
  BSP) — the reroute works both directions, indexing the real `R4_TASKS` fields on an
  AP. The *rest* of the `R4_CURRENT`-touching surface (yield/exit/fork/futex/…) still
  reads the global and runs only on the BSP.
- **The sandbox allowlist gate** (`syscall.rs`, runs on every syscall `nr < 64`) still
  reads `R4_TASKS[R4_CURRENT]`, not `r4_current_smp()`. Routing it per-CPU is blocked
  on a prerequisite: the *synthetic* `ap_user_selftest` uses `0x5A` as its per-CPU
  `current`, which is **out of range** for an `R4_TASKS` index (len = `R4_MAX_TASKS`),
  so indexing the table by per-CPU current there would be OOB. That synthetic test
  must first migrate to a real `R4_TASKS` slot (as `ap_r4_migrate_selftest` already
  does) before the gate can safely use `r4_current_smp`. Carry-forward.
- Routing the remaining read sites through `r4_current_smp`, plus a lock on `R4_TASKS`
  mutations for the write sites so APs can run the full syscall set concurrently, is
  the remaining core rewrite, done incrementally (each batch reviewed + gated).
- The migrated R4 task is dispatched and run once as a boot self-test; the BSP still
  owns the live scheduler and the APs do not yet pull ready tasks from their run
  queues into ring 3 autonomously.
