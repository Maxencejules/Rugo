# SMP user task on an application processor — contract v1

Status: boot-verified via `make test-smp-runtime-v1` (the `-smp 2` go lane)
Source: `kernel_rs/src/smp.rs` (`ap_user_selftest`, `ap_run_user_task`,
`ap_user_trap`, `ap_user_done`, `AP_USER_CODE`, the work-kind-2 arm of
`ap_poll_work`, the per-CPU TSS setup in `ap_entry`),
`kernel_rs/src/arch_x86.rs` (per-CPU `TSS`/`tss_init_cpu`, `enter_ring3_with_arg`,
`AP_KSTACK`/`ap_kstack_top`, the vector-0x81 IDT gate),
`kernel_rs/src/trap.rs` (vector 129 routing), `arch/x86_64/isr.asm`
(`isr_stub_129`).
Proof: `tests/runtime/test_smp_runtime_v1.py::test_default_lane_boots_clean_on_multicore`.

Full-OS implementation guide Part I.3 (SMP), scheduler capstone: an **application
processor runs a ring-3 USER task on its own core**. The earlier SMP slices made
APs run *kernel* work ([`smp_lock_v1.md`](smp_lock_v1.md),
[`smp_v1.md`](smp_v1.md), [`percpu_v1.md`](percpu_v1.md), TLB shootdown, the
cross-CPU work dispatcher); this slice crosses into ring 3 on an AP — the thing a
real SMP scheduler must do.

## What was missing, and the core piece

The kernel had a **single** TSS with one `rsp0`. A ring-3→ring-0 transition
(syscall/interrupt) reads `rsp0` from the running CPU's TSS, so two CPUs sharing
one TSS would take traps onto the *same* kernel stack and corrupt each other —
which is why running user code on an AP was the documented "core rewrite".

v1 makes the **TSS per-CPU**. The GDT holds up to `MAX_TSS` (8) sixteen-byte TSS
descriptors (CPU `c` at `GDT[5 + 2c]`, selector `0x28 + 0x10·c`); each CPU has
its own `TSS[c]` with a private `rsp0` pointing at its own kernel stack
(`AP_KSTACK[c]`, 16 KiB, 16-aligned). The BSP stays slot 0 / selector 0x28
(unchanged). Each AP installs and loads (`ltr`) its own descriptor in `ap_entry`
before checking in, so once every AP is online they are all ready to take a
ring-3 task. Distinct slots are distinct GDT words, so the concurrent installs
never tear a descriptor another CPU uses.

## Flow

1. **BSP builds the task** (`ap_user_selftest`, run inside `smp_init` with
   interrupts disabled, before the Go runtime starts). It creates a private
   address space with the same machinery spawned apps use
   (`mm::address_space_create` clones the kernel half;
   `mm::as_copyout`/`as_map_zeroed` map the code page at `0x0140_0000` in the
   exec window — so NX is cleared — and a stack page). It publishes the CR3,
   entry VA, and stack top, then dispatches **work kind 2** through the existing
   cross-CPU work mailbox.
2. **An AP claims it** (`ap_poll_work`, woken by its periodic LAPIC timer). The
   `WORK_CLAIM` CAS guarantees exactly one AP runs it. For kind 2 the AP calls
   `ap_run_user_task`: it confirms it owns a TSS (per-CPU slot < `MAX_TSS`, read
   from `gs:[0]`), saves the kernel CR3, loads the user CR3, and `iretq`s to ring
   3 via `enter_ring3_with_arg` with the argument in RDI (IF clear, so no timer
   lands mid-task).
3. **The user code runs in ring 3 on the AP.** The 11-byte payload
   (`AP_USER_CODE`) computes `RDI = arg·2 + 1` and executes `int 0x81`. It
   touches no memory, so it never demand-faults (every page is premapped) — the
   AP performs no PMM mutation that could race the BSP.
4. **Return to the kernel** (`ap_user_trap`, vector 0x81 → `trap_handler` case
   129). It records the reported value (RDI = `frame[9]`) and the AP's slot
   (`gs:[0]`), restores the kernel CR3, and rewrites the trap frame to `iretq`
   into `ap_user_done` in ring 0 on this AP's own kernel stack — the same
   ring-3→kernel trampoline pattern the uniprocessor lane uses
   (`m3_return_to_kernel_halt`). `ap_user_done` publishes completion
   (`WORK_RESULT`/`WORK_DONE`) and resumes normal AP polling.
5. **BSP verifies + reclaims.** `smp_dispatch_work` returns the reported value;
   the BSP checks `result == arg·2 + 1` **and** the running CPU was an AP (slot
   ≥ 1), emitting `SMP: ap user task ok`. Because the AP restored the kernel CR3
   *before* publishing `WORK_DONE` (and the BSP waits on it with Acquire
   ordering), the AP is provably off the user address space by the time the BSP
   `address_space_release`s it — no use-after-free, no leak.

## Why the proof is not fakeable

`getval`-style reasoning: `result == arg·2 + 1` can only be produced by executing
the payload, and it was reported through a ring-3-only gate (DPL=3 vector 0x81)
taken from `CS` with RPL=3. The reporting CPU's per-CPU slot is ≥ 1, and the BSP
is spinning in ring 0 (never enters ring 3) during the test — so the code
demonstrably ran in ring 3 **on an application processor**, took the trap onto
that AP's own kernel stack, and returned cleanly (`RUGO: halt ok`, no `USERPF`).

## v1 boundary / carry-forward

- **One task, dispatched once, from the boot self-test.** This is the capstone
  *mechanism* (per-CPU TSS + ring-3 entry/exit on an AP), not yet a running
  multi-CPU scheduler. A real SMP scheduler — per-CPU run queues, migrating R4
  tasks onto APs, load balancing, per-CPU `current`, SMP-safe syscall dispatch
  on every core — builds on exactly these primitives and is carry-forward.
- **`MAX_TSS = 8`.** CPUs beyond slot 8 still run kernel work + park but cannot
  take a ring-3 task (they have no TSS); the dispatch path degrades gracefully
  (reports a sentinel, still unblocks the BSP). Growing this is mechanical.
- **Kernel-embedded payload, IF-clear ring 3.** The task is a fixed in-kernel
  blob run with interrupts disabled (so it cannot itself be preempted on the AP).
  Running a *preemptible* user task on an AP (its LAPIC timer driving a real
  per-CPU scheduler tick) is the next step.
- **Shared work mailbox, single in-flight item.** Reuses the one-slot kind-based
  dispatcher; a real per-CPU run queue replaces it.

## Acceptance

`make test-smp-runtime-v1`: booting `os-go.iso` with `-smp 2`, the transcript
shows the full SMP self-test chain ending `SMP: ap work ok`, then
`SMP: ap user task ok` (an AP ran the ring-3 task and reported `arg·2+1` from
slot ≥ 1), then the Go lane reaches `GOSH: session ready`,
`GOINIT: result shutdown-clean`, and `RUGO: halt ok` — with no `USERPF` and no
`FAIL`. The base `os.iso` `-smp 4` lane is unaffected (the capstone is go-lane
gated).
