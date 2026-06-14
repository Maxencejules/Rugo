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

## v1 boundary / carry-forward

- The task migrated to the AP is still a **kernel-built payload** in a private
  address space, and it issues a deliberately **`R4_CURRENT`-free, non-printing**
  syscall (`sys_time_now`). Migrating a **live, scheduled R4 task** mid-execution
  and letting it run the *full* syscall surface on the AP requires making the
  shared scheduler state SMP-safe — `R4_CURRENT`/`R4_TASKS` become per-CPU
  `current` + a locked task table, and every `R4_CURRENT`-touching syscall
  (yield/exit/fork/futex/…) must use the per-CPU current under that lock. That is
  the remaining core rewrite and is carry-forward.
- The per-CPU `current` is set/observed but the BSP still owns scheduling; the APs
  do not yet pull from their run queues into ring 3 autonomously.
