# Rugo — Implementation Guide: From Hobby OS to Fully Functioning OS

Date: 2026-06-13
Companion to: [`docs/analysis/full-os-gap-analysis.md`](full-os-gap-analysis.md) (what is missing) and the per-subsystem contracts under `docs/runtime/`.

## Purpose & scope

The gap analysis enumerates what Rugo lacks to be a "fully functioning OS." This
document is the *constructive* counterpart: **how to build each missing piece**,
grounded in Rugo's actual source (real file:line, struct names, syscall ids) and
its conventions — not generic OS theory.

It is organized by **dependency order**, not by importance. The single keystone —
**per-process address spaces** — unblocks fork, concurrent processes, isolation,
SMP TLB management, ASLR, and ultimately a GUI. Almost everything else sits
downstream of it. Build top-to-bottom.

Each section follows the same template: *Status/prerequisites → Design for Rugo →
Concrete changes → ABI additions → Acceptance (the `make test-*-v1` + markers +
contract doc) → Gotchas → Effort/risk*. Effort tags (S/M/L/XL) are rough
order-of-magnitude, not estimates.

> The section drafts were each written against the current source. Where a section
> proposes a specific syscall id, **§0.2 (the master ID table) governs** — the
> per-section numbers are illustrative.

---

## §0.1 — Conventions every change must follow

These are non-negotiable repo rules; every step below assumes them.

- **One product lane.** New subsystems land in the unified `go_test` kernel
  (`= image-go = ISO_GO_PATH`) or are compiled unconditionally. **Never** add a
  new test-only feature lane — that was the §2.3 fragmentation the gap-closure
  work eliminated. Build through the Makefile (`mingw32-make <target>`), never
  bare `cargo` (it picks the MSVC toolchain and fails to link).
- **Boot-verified acceptance.** Every feature ships a `make test-<name>-v1`
  target → `tests/runtime/test_<name>_v1.py` that boots via the
  `qemu_go_c4_runtime` fixture and asserts **serial/screen markers emitted by
  runtime code**. No seeded JSON; no unit-only proofs. Pair it with a
  `docs/runtime/<name>_v1.md` contract that records the v1 boundary.
- **Markers.** `serial_write_hex` emits 16 zero-padded hex digits — assert the
  padded form. Anchor only on single-write kernel/`lineBuilder` markers; never on
  echoed shell prompt lines (the shell echoes char-by-char and async markers
  splice them).
- **ABI is frozen and additive.** Documented in `docs/abi/syscall_v3.md`. The v3
  window `28..47` is **full** (46 = `sys_spawn`, 47 = `sys_fs_ctl`). The additive
  v3.2 window `48..63` is partly used (48 = `sys_signal_ctl`, 49 =
  `sys_net_query`). Prefer **op-multiplexing under an existing syscall** (as
  `fs_ctl`/`net_query`/`signal_ctl` already do) over burning a new id. See §0.2.
- **Zero external crates.** In-repo implementations only. The kernel is `no_std`
  with a custom `#[global_allocator]` heap (`mm.rs`); `build-std = core+alloc`, so
  `alloc::{Vec, Box, String}` are available in the kernel.
- **Size budgets.** TinyGo userspace `gousr.bin` caps at 28 KiB; the
  kernel-embedded Go image has a ~32 KiB code-page budget (adding the 9th code
  page is a documented procedure); C apps cap at `EXEC_APP_MAX_BYTES` (64 KiB).
  Any feature that grows these must say so.
- **Toolchain reality.** Host `gcc`/`ld` are PE-only — C apps compile
  `-mabi=sysv`, link in PE, then rewrap via `tools/pe_to_elf_v1.py`. `rust-lld`
  can crash transiently (rerun). Use a **git worktree** for any parallel build:
  editing kernel sources while a `make test-qemu` runs corrupts the run, and a
  kernel-source change invalidates all ~50 feature-lane builds (~30+ min rebuild).

---

## §0.2 — Master syscall-id allocation (authoritative)

The v3.2 window `48..63` has 14 free slots (`50..63`). To avoid exhausting it,
**most new capability is added as new *ops* under the four multiplexed syscalls**,
and only genuinely new call shapes consume a fresh id. Proposed allocation
(supersedes any id a section names):

| id | syscall | owner section |
|----|---------|---------------|
| 48 | `sys_signal_ctl` (existing; add ops: mask, sigreturn-rt, kill-by-tid) | Security/process |
| 49 | `sys_net_query` (existing; add ops: listen, accept, route, dhcp-renew, icmp) | Networking |
| 50 | `sys_vm_ctl` (mmap / munmap / mprotect / brk — op-multiplexed) | Memory |
| 51 | `sys_proc_ctl` (fork / clone / exec-replace / setpriority — op-multiplexed) | fork/exec/threads |
| 52 | `sys_futex` (wait / wake) | SMP/concurrency |
| 53 | `sys_time` (clock_gettime / gettimeofday / nanosleep / timer_* — op-multiplexed) | Time |
| 54 | `sys_getrandom` | Security |
| 55 | `sys_mount` (mount / umount / pivot — op-multiplexed) | Filesystem |
| 56 | `sys_ioctl` (TTY/pty + generic device control — op-multiplexed) | Userspace |
| 57 | `sys_poll` (select/poll/epoll-style fd readiness) | IPC |
| 58 | `sys_power` (shutdown / reboot / suspend via ACPI) | Power |
| 59 | `sys_sandbox` (pledge/unveil-style restriction) | Security |
| 60 | `sys_dlctl` (dynamic-linker support: map-segment / resolve) | Userspace |
| 61 | `sys_sysinfo` (/proc-style metrics, dmesg read) | Observability |
| 62 | reserved | — |
| 63 | **v4 escape** — reserved as the trigger for a v4 ABI window decision | — |

Rule: when a section says "new syscall id," map it to a row above; if it needs an
op under an existing multiplexed syscall, add the op and document it in
`docs/abi/syscall_v3.md` rather than consuming an id.

---

## §0.3 — Dependency-ordered roadmap

```
                    ┌─────────────────────────────────────────┐
                    │  PART I — FOUNDATIONS (critical path)    │
                    └─────────────────────────────────────────┘
   [1] Per-process address spaces  ◄── the keystone; nothing real concurrent
        │                               or isolated exists without it
        ├──► [2] fork / exec / clone + copy-on-write + user threads
        ├──► [3] SMP: per-CPU sched + kernel locking + IPIs (TLB shootdown
        │          needs per-task CR3 from [1]; locking needs [2]'s frame refcounts)
        └──► [4] Memory: mmap/brk, swap, huge pages, user malloc/free

                    ┌─────────────────────────────────────────┐
                    │  PART II — CORE SUBSYSTEMS               │
                    └─────────────────────────────────────────┘
   [5] Filesystem maturity (journaling, cache, mounts, FAT, partitions, /proc,/dev)
   [6] Networking maturity (TCP retransmit/listeners/multi-conn, DORA, ICMP, routing, IPv6)
   [7] Driver model + buses (registry, PCIe ECAM, USB/XHCI+HID, DMA, e1000)
        │   ([6] interrupt-driven RX and [7] benefit from [3]'s locking)

                    ┌─────────────────────────────────────────┐
                    │  PART III — HUMAN INTERFACE             │
                    └─────────────────────────────────────────┘
   [8] Input + graphics stack + audio
        │   (mouse/HID from [7]; window server needs concurrent procs from [1]/[2])

                    ┌─────────────────────────────────────────┐
                    │  PART IV — SYSTEM SERVICES              │
                    └─────────────────────────────────────────┘
   [9]  Time/timekeeping + power/ACPI (wait queues unblock blocking reads)
   [10] Security: RNG ──► ASLR (needs [1]); RNG+crypto ──► TLS (in [6]); sandbox,
        multi-user, secure boot, disk encryption

                    ┌─────────────────────────────────────────┐
                    │  PART V — USERSPACE & OPERATIONS        │
                    └─────────────────────────────────────────┘
   [11] rlibc completion, TTY/pty + job control, dynamic linker/.so, package
        manager + repo + update fetch, installer, UEFI + bare metal, crash dumps,
        dmesg/syslog, /proc metrics, self-hosting
```

**Suggested milestone cuts:**
- **Milestone A (isolation):** [1] + [2] — real processes. Single biggest leap.
- **Milestone B (multicore):** [3] + [4].
- **Milestone C (operable):** [5] + [6] + [9-time].
- **Milestone D (usable bare metal):** [7] + [8] + UEFI/installer from [11].
- **Milestone E (hardened):** [10] + the rest of [11].

---


# Part I — Foundations (the critical path)


### Per-process address spaces (keystone)

**Status today / prerequisites** — All tasks share a single `USER_PML4` (lib.rs line 1547) initialized at boot. `r4_switch_to` (lib.rs ~2804) restores all 22 general-purpose registers but does **not** reload CR3, so the TLB remains the same. The exec window `[0x0140_0000, 0x0180_0000)` is single-occupancy: `EXEC_APP_TID` (lib.rs 3116) is checked at spawn; a second app blocks with "busy" (lib.rs 3247). Demand paging in `mm.rs::try_demand_map` (line 414) walks `cr2 & PHYS_MASK` indirectly via `cr3`, but installs mappings into the kernel's assumed-current CR3 without per-task metadata. The interrupt/scheduler stack (`stack_top()`) is shared. This is a hard architectural ceiling: pipelines run sequentially (exec_v1.md), and no real concurrency is possible. Prerequisites: **Dynamic Memory Foundation** (heap, PMM, demand paging in lib.rs) must land first; **R4 task model** (R4Task struct at lib.rs 2607) already exists, but has no AddressSpace field.

**Design for Rugo** — Each spawned task gets a heap-allocated `AddressSpace` struct holding its own PML4 root frame (from the PMM). At spawn (sys_spawn_v1, line 3218), clone the kernel higher-half mappings (PML4[256..512], covering kernel text and the HHDM) from the boot kernel PML4 into the new task's PML4, isolating user-half (PML4[0..256]) so each task owns its demand-paged heap and stack region. On every `r4_switch_to`, read the task's PML4 physical address and reload CR3 — this flushes the global TLB (cost: ~11 cycles on modern CPUs; mitigated by PCID if we enable EFER bit 17 later, but v1 pays the cost). Embed an `address_space_id: u64` field in R4Task so `try_demand_map` can locate the correct PML4 via the task table, not indirectly via CR3 (avoids a volatile read). On task exit (r4_exit_and_switch), release the AddressSpace: walk the PML4's user-half PD entries, free the page-table frames, free the PDPT and PML4 frames themselves. The exec app window becomes per-task instead of global: allocate a new [0x0140_0000, 0x0180_0000) window in each task's address space. This lifts the single-occupancy block and allows concurrent spawned apps.

**Concrete changes**:
- **lib.rs**: Add `AddressSpace` struct with `pml4_phys: u64` field (the root frame's physical address). Add `address_space_id: u64` to R4Task (line 2607) to identify the task's AddressSpace in the heap-backed table.
- **lib.rs**: In `sys_spawn_v1` (line 3218), call a new `address_space_create()` → allocates PML4 frame (PMM), PDPT frame, clones kernel higher-half from the boot kernel PML4 (lines 2041–2059), stores the new PML4 physical address in the child's `address_space_id` (maps to a heap R4Task ID). Remove the single-occupancy check (`if EXEC_APP_TID >= 0`); tasks can coexist now.
- **lib.rs**: Modify `r4_switch_to` (line 2804) to reload CR3 with the target task's AddressSpace PML4 physical address: `asm!("mov cr3, {}", in(reg) tasks[tid].pml4_phys)`.
- **lib.rs**: In `r4_exit_and_switch` (search for this function), call a new `address_space_release(tid)` → walks the task's PML4 user-half, frees PT/PD/PDPT/PML4 frames back to the PMM in correct order.
- **mm.rs**: Modify `try_demand_map` (line 414) signature to take the target task's address_space_id. Read the task table to fetch the correct PML4 physical address, then walk that PML4, not CR3. This isolates page-fault handling from the current CR3.
- **trap.rs**: At page-fault entry (trap_handler int 14, line 49), pass `R4_CURRENT` to `try_demand_map` so it targets the faulting task's address space.

**ABI additions** — None. The per-task address space is a kernel-internal change; syscall signatures do not change.

**Acceptance** — Target `make test-concurrent-exec-v1` boots two or more spawned external apps from the app region simultaneously and uses pipes to join them (e.g., `cat /data/file | wc` runs in parallel stages, not sequentially). Contract doc: `docs/runtime/per_task_as_v1.md` records the AddressSpace struct layout, cloning strategy, and CR3 reload cost/TLB implications. Runtime markers: `SPAWN: <name> as_ok <as_id>` when AddressSpace is created; `PF: <task> as=0x<id>` at demand map; `ASRELEASE: <task> as=0x<id>` at exit. Python test (`tests/runtime/test_concurrent_exec_v1.py`) spawns 2–3 ELFs with stdin/stdout pipes, asserts both produce output (overlap on SERIAL output proves concurrency), and verifies EOF propagation on exit.

**Gotchas**:
- **TLB cost**: Every task switch flushes the global TLB; IF a workload has >10 switches/sec, measure and consider PCID (CR4 bit 17, EFER bit 17). For now, accept it and document the trade-off.
- **Shared kernel heap**: The kernel's 4 MiB heap (mm.rs ~20) is shared; spinlocks are not implemented yet, so heap alloc/free must be done only at points where preemption is disabled (task creation/destruction are safe; do NOT call from demand page or IPC hot paths). Flag this as a risk and call out when phase 2 must add per-cpu spinlocks.
- **Single-occupancy is gone, but exec app window layout is now per-task**: Pipelines now run concurrently, which is the feature, but it means you can no longer rely on a global EXEC_APP_TID sentinel. Each task has its own [0x0140_0000, 0x0180_0000) region; do NOT try to refactor it as a global resource.
- **Page-table walk isolation**: try_demand_map now uses the target task's PML4, not CR3. This is correct but means a bug in address_space_create (kernel mapping clone) will silently affect all spawned tasks — test clone_kernel_mappings thoroughly.

**Effort / risk** — **M (Medium)**. The core change is heap-allocated AddressSpace + CR3 reload + PMM frame release on exit — straightforward, but the page-table walk refactor in try_demand_map must be correct and the kernel-mapping clone must be verified. Main risk: a missed kernel-high mapping or a cloning bug will corrupt all spawned tasks. Mitigation: unit test the clone in isolation (boot-verify with a marker on every PML4[256..512] entry cloned), and run test-concurrent-exec-v1 with heavy pipe traffic (cats of large files). Estimate ~3–4 days to implement, test, and verify TLB behavior.


### fork / exec / clone + copy-on-write + user threads

**Status today / prerequisites**

The Rugo kernel currently has three isolated subsystems that must be unified:

1. **Deferred stubs** (lib.rs ~5835): `sys_fork_deferred_v1()` and `sys_clone_deferred_v1()` both return `-1` (all lanes). These are placeholder syscall 43/44.

2. **exec via sys_spawn** (lib.rs ~3218): The `go_test` lane supports spawning child processes from a filesystem (sys_spawn = id 46). Each child gets its own task slot in the heap-backed `R4_TASKS` table and inherits the parent's capability flags via `r4_init_task()` (lib.rs ~2716). However, all tasks share a single `USER_PML4` page table (lib.rs ~1547); address spaces are not isolated.

3. **Single-level thread creation** (`sys_thread_spawn_r4`, lib.rs ~3598): Current threads share the spawner's address space and stack (allocated via `r4_stack_top_for_slot()`). No copy-on-write or per-task page tables exist. The wait/reap path is implemented (lib.rs ~3646) with parent–child tracking via `parent_tid` and task state transitions (Ready/Running/Blocked/Exited/Dead).

This section lands **fork** (syscall 43) and **clone** (syscall 44) in the v3.2 window (48..63 are reserved; we'll use 50 for fork, 51 for clone), along with **copy-on-write** frame sharing and **per-task address spaces**. The existing exec and wait infrastructure remain; we extend it to isolate address spaces.

**Design for Rugo**

*Why this approach:* Rugo's kernel is demand-paged (mm.rs ~414) with a single physical memory allocator (PMM bitmap). Per-process page tables require per-frame refcounting so pages can be safely shared under CoW and freed when the last reference drops. We avoid forking the entire kernel heap; instead, we fork only user-space mappings (code + stack + demand-paged heap). The exec() path reuses the syscall id 46 mechanism but clears old mappings before loading the new ELF.

The arch-specific gain is that `r4_switch_to()` (lib.rs ~2804) already restores user `saved_frame[17..21]` (RIP/CS/RFLAGS/RSP/SS) per task; we extend it to load CR3 (PML4 phys addr) from a new per-task field. Demand paging in trap.rs ~49 reads CR3 to find the faulting task's page tree, so CoW write faults can be handled there without global state.

*CoW mechanics:* On fork, PTEs for parent data/heap regions are marked read-only (PTE_P_RO_U, clearing the W bit). When a task writes, the page-fault handler detects (error_code & 0x2 != 0), allocates a fresh frame via `alloc_frame()`, copies the old frame from HHDM, and updates the child's PTE. Parent unaffected. Stale refcounts are cleaned up on task exit (r4_exit_and_switch ~3042).

*Stack allocation:* The existing `r4_stack_top_for_slot()` allocates a unique stack per task. For clone (shared address space), the child inherits the parent's PML4 phys addr (no page-table copy); for fork (isolated space), we copy the parent's PML4/PDPT/PD but share code pages initially (marked read-only to trap writes).

**Concrete changes**

- **kernel_rs/src/lib.rs:**
  - Add `struct R4Task` field `pml4_phys: u64` (default = 0 for initial shared state; non-zero = per-task PML4 physical address).
  - Add static `COW_REFCOUNTS: [u16; MAX_FRAMES]` (per-frame refcount, u16 to avoid overflow on 64K+ forks; unlikely but safe). Initialize from PMM on boot; increment on frame assignment, decrement on page-table teardown.
  - Modify `r4_init_task()` (~2716) to copy parent's pml4_phys if non-zero; else leave 0 (shared).
  - Add `unsafe fn sys_fork_v1() -> u64` at syscall id 50: allocate new task slot, clone parent's user PML4/PDPT/PD (not code/stack), mark cloned data/heap PTEs read-only, increment refcounts on cloned frames, initialize child frame with parent's entry point and args, set parent_tid, return child tid. On error, clean up partial copies and return -1.
  - Add `unsafe fn sys_clone_v1(entry: u64) -> u64` at syscall id 51: like sys_thread_spawn_r4 but uses parent's pml4_phys directly (no copy). Allocate new stack in demand zone. Return child tid.
  - Modify `r4_switch_to()` (~2804) to check if task has non-zero pml4_phys; if so, load it into CR3 before restoring frame. Add `core::arch::asm!("mov cr3, {}", in(reg) pml4_phys)`.
  - Modify `r4_cleanup_task_resources()` (~3446) to call new `r4_free_task_pml4(tid)` if task has per-task page tables.
  - Add `unsafe fn r4_free_task_pml4(tid: usize)` to walk the task's PML4/PDPT/PD, decrement refcounts on each frame in the page table tree and data region, free leaf PT pages via `free_frame()`, free the PML4 itself.

- **kernel_rs/src/trap.rs:**
  - Extend page-fault handler (~49) to detect write faults (error_code & 0x2): if PTE present but not writable, allocate a new frame, copy from old frame via HHDM, update PTE to writable, decrement refcount on old frame, increment on new frame. This is the CoW trap path.

- **kernel_rs/src/mm.rs:**
  - Add `COW_REFCOUNT_INIT` call in `pmm_init()` to allocate the refcount array and initialize each entry to 0 (or 1 for already-allocated frames). Keep it as `static mut` alongside the PMM bitmap.
  - Add `pub fn get_frame_refcount(phys: u64) -> u16` and `pub unsafe fn set_frame_refcount(phys: u64, count: u16)` for CoW book-keeping.

- **docs/abi/syscall_v3.md:**
  - Update ID window: `48..63` is now v3.2. Add entries:
    - `50` = `sys_fork` (op: fork parent to child, CoW child PTEs, return child tid to parent, 0 to child; E_RANGE if no slots, E_PERM if can_spawn denied).
    - `51` = `sys_clone` (op: entry point, allocate shared-address-space task, return tid; E_INVAL if entry not in code region, E_RANGE if no slots).

**ABI additions**

- Syscall 50: `sys_fork` — no arguments, returns child tid (or -1). Child receives rax=0; parent receives rax=child_tid. Page tables isolated via CoW.
- Syscall 51: `sys_clone` — rdi = entry point (RIP for new thread), returns tid (or -1). Address space shared with parent (same pml4_phys). Stack is unique, allocated in demand zone.
- Document in `docs/abi/syscall_v3.md` under the v3.2 window (48..63, which now has 50/51 allocated; 48=signal_ctl, 49=net_query, 50/51=fork/clone).

**Acceptance**

- `make test-fork-v1`: Boots go_test, creates a parent task that forks itself, writes to heap in child, asserts different values are visible in parent. Runtime test `tests/runtime/test_fork_v1.py` checks:
  - `USERPF: ...` markers for CoW write faults (at least 2–4 per fork, depending on heap activity).
  - `EXEC: fork ok` or `FORK: child tid=0x<N>` marker emitted by child after resumption.
  - Parent reads its original heap value; child reads a modified value (proves CoW isolation).
  - Contract doc `docs/runtime/fork_v1.md` specifies: child starts at same code entry, receives rax=0; parent resumes, receives rax=child_tid; both share COW-protected memory until write. Kill on invalid flags (e.g., fork from external app without can_spawn flag).

- `make test-clone-v1`: Two tasks spawn via sys_clone (syscall 51), write to a shared address at the same va, see each other's writes (proves shared address space). Serial markers: `CLONE: child tid=0x<N>` on child creation, `SCHED: shared addr writes ok` or similar after both tasks increment a global counter and observe the sum.

- `make test-fork-exec-v1`: Parent forks, child forks again (two levels), verify the second child has correct isolation from grandparent and parent.

**Gotchas**

- **Single-occupancy exec window conflict**: The `go_test` lane reserves [0x0140_0000, 0x0180_0000) for single-spawn apps. On fork, the child must not inherit this window mapping; clear it before returning or mark it read-only + unmap on write. Alternatively, ensure forked children do not use sys_spawn.

- **Refcount overflow**: With u16 refcounts, 65K+ tasks can overflow. Unlikely in practice (R4_MAX_TASKS is ~256 typically), but assert at boot or log a warning if MAX_TASKS * 4 (average pages per task) exceeds 65K.

- **Demand paging + CoW stack**: The stack demand-maps on write, so the initial CoW stack clone (read-only) will immediately fault on the first push. Ensure the trap handler handles this case: write fault on demand-unmapped page = allocate fresh frame (not a CoW copy), mark writable.

- **CR3 reload in ring 3**: The spec requires `mov cr3, ...` to be executed in ring 0 only. Do not let user code call sys_set_cr3; only r4_switch_to (in exception context) loads per-task CR3. Verify no other path reloads it.

- **Shared USER_PML4 vs. per-task**: Once fork lands, some tasks use shared USER_PML4 (pml4_phys = 0) and others use private PML4s. The r4_switch_to logic must handle both: if pml4_phys == 0, skip the CR3 reload (stay on the kernel's global table); otherwise reload. During boot and before first fork, all tasks use the shared table.

- **IF=0 in CoW trap**: The page-fault handler is entered with IF=0 (interrupts disabled). If the CoW copy loop is slow (e.g., large page), other tasks may not preempt. This is acceptable for 4 KiB copies, but note it in the contract doc.

**Effort / risk**

- **Size: Large (L).** Requires modifications to core structures (R4Task, PMM, trap handler), a new refcount array (~64 KiB for 16M frames, acceptable), and three new functions (fork, CoW trap, cleanup). The exec/wait skeleton is stable; risk is in CoW correctness and refcount integrity.

- **Main risk: Refcount coherency.** Errors in incrementing/decrementing refcounts on fork, CoW, and task exit can lead to use-after-free (page freed while still mapped) or leaks (page never freed). Mitigation: unit tests that fork many times and verify heap values; a static refcount audit at shutdown that asserts all refcounts == 0 or 1 (for pages still in the kernel heap).

- **Secondary risk: Multi-level page-table walks.** If the PML4 cloning logic misses a PDPT or PD layer, the child page table is incomplete, causing page-faults in child that are not demand-mapped (because the intermediate tables are missing). Mitigation: careful unit tests that fork and immediately write to multiple addresses in code/data/heap regions.


### SMP: per-CPU scheduling, kernel locking, IPIs

**Status today / prerequisites** — Limine SMP enumerate is live (smp.rs:66 `smp_init()`); every AP checks in on `APS_ONLINE` atomic counter and parks in `ap_entry` (smp.rs:56). Scheduling stays on BSP (single `R4_CURRENT` global in lib.rs:2683). Must land AFTER per-process address spaces (each task has its own page tables loaded on switch; lib.rs:2040, 2219 install per-task PML4). Prerequisite: `docs/runtime/smp_v1.md` contract complete.

**Design for Rugo** — A single shared `USER_PML4` (lib.rs:1547) is loaded on every task switch, so CR3 reloads are single-CPU only. Per-CPU scaling requires demultiplexing: each AP gets its own GDT/TSS/IDT (arch_x86.rs:71–144), a per-CPU run queue with a spinlock, per-CPU `current_task` pointer, and per-CPU LAPIC timer for preemption. Global mutable statics that are currently read-only or protected by IF=0 (R4_TASKS table, PMM bitmap, heap allocator, VFS/net state) must be guarded by kernel spinlocks since APs will execute the same trap handlers concurrently. Demand paging (mm.rs:416 `try_demand_map`) modifies the shared user page tables, so it needs atomic frame allocation. IPIs (Inter-Processor Interrupts) via x2APIC for reschedule (kick an AP out of user code to check the scheduler) and TLB shootdown (every AP must invalidate the target VA via `invlpg` to maintain coherency when a shared PML4 entry changes). Kernel spin-acquire always disables interrupts (IF=0), avoiding deadlock from an interrupt trying to acquire a held lock.

**Concrete changes** —
  - Add `kernel_rs/src/spinlock.rs`: Ticket lock (compare-exchange-free; acquire spins on `READ`, release increments NEXT; no heap needed).
  - Per-CPU data structure (`SmpCpuData`) in smp.rs: `cpu_id`, `lapic_id`, `gdt` (7 u64), `tss`, `idt` (array of interrupt handlers), `run_queue` (Vec<usize> of Ready task ids), `current_task` (usize), `preempt_count`.
  - `static mut SMP_CPU_DATA: [SmpCpuData; MAX_CPUS]` (smp.rs), initialized at boot by BSP for all online CPUs.
  - Replace IF=0 critical sections with spinlock acquire/release pairs: protect R4_TASKS access (lib.rs:2682), PMM bitmap (mm.rs:57), heap alloc/free, VFS inode cache, TCP/socket state. Each protected static gets a spinlock global (e.g., `R4_TASKS_LOCK`, `PMM_LOCK`).
  - `smp_ap_init()`: each AP (from ap_entry via new LAPIC timer init) loads its own GDT/TSS, sets up local IDT with LAPIC EOI paths, initializes per-CPU run queue to empty, sets up per-CPU preemption via LAPIC LVT timer (32-bit count/period, ~10ms ticks).
  - New syscall sys_smp_ctl (id 50, v3.2 window): op 1 = query CPU count, op 2 = pin to CPU (hint only; returns -1 on v4 if not implemented).
  - `r4_timer_preempt` becomes per-CPU: fetch `R4_CURRENT` from `SMP_CPU_DATA[my_cpu].current_task` instead of global, use per-CPU run queue. Scheduler (r4_find_ready) now searches per-CPU queue first, then steals from other queues.
  - IPI vector (e.g., 0x50 for reschedule, 0x51 for TLB shootdown) handlers in arch_x86.rs.
  - `invlpg` is local only; for TLB shootdown when PML4 changes (e.g., add/remove a VFS page, SHM attach), send IPI to all other CPUs and wait for ACK.

**ABI additions** —
  - `sys_smp_ctl` (id 50): documented in docs/abi/syscall_v3.md under v3.2 window (48..63). Op codes: 1=get_cpu_count (rax=count or -1), 2=affinity_hint (rdi=cpu_id, returns 0 or -1).
  - IPI vectors 0x50 (reschedule), 0x51 (TLB shootdown) added to IDT.

**Acceptance** — `make test-smp-v1` target boots the kernel with `-smp 4` (4 CPUs in QEMU). Proof file `tests/runtime/test_smp_v1.py` asserts:
  - `SMP: cpus=0x0000000000000004` (all 4 detected)
  - `SMP: aps online=0x0000000000000003` (3 APs check in, BSP runs scheduler)
  - Three or more tasks spawn and run concurrently on separate cores; `SCHED: dispatch tid=0x<tid>` markers show tasks switching on different CPUs.
  - Demand paging during concurrent execution completes without deadlock; frame alloc is serialized via `PMM_LOCK`.
  - `RUGO: halt ok` on completion.
  Contract doc `docs/runtime/smp_v1.md` formalizes the per-CPU invariants, lock ordering (spinlock < heap), and IPI retry semantics.

**Gotchas** —
  - **Lock ordering**: never acquire a spinlock while holding another spinlock, and never acquire a spinlock from within an interrupt handler on the same CPU (deadlock). Always disable IF on entry to any lock-protected section.
  - **Single-occupancy exec window** (lib.rs:2104–2109): remains global (not per-CPU) because only one app can load at a time. A second spawn while an app is resident must wait or fail; add a spinlock around EXEC_APP_TID.
  - **Shared USER_PML4**: every task shares the same PML4 physical page; demand-paging frame alloc and PTE updates must be atomic. Any missed TLB shootdown leaves stale TLB entries on a peer CPU, silently violating VAS separation.
  - **Marker splicing**: the boot-probe sub-range (mm.rs:474, va < DEMAND_BASE + 0x1_0000) emits demand-map markers; with parallel demand on multiple CPUs, marker order is nondeterministic. The test must count *sets* of markers, not order.
  - **Heap contention**: the global alloc/free path (default #[global_allocator]) must serialize. One spinlock covers the entire heap, not per-slab locks.
  - **VFS/net state**: vfs.rs (inode table), net.rs (socket table), tcp.rs (connection table) all modify global statics. Each needs its own lock or a unified lock if they interact.

**Effort / risk** — XL. Main risks: (1) introducing deadlock from lock-ordering mistakes or IF=0 violations, (2) TLB coherency bugs (stale entries on a peer after PML4 change without IPI), (3) frame alloc under load (demand paging from multiple CPUs competes for PMM bitmap; stress test must not OOM), (4) marker stability in tests (output splicing from 4 CPUs makes deterministic assertions hard; use counters, not sequence checks). Requires close reading of x2APIC spec (Intel SDM vol. 3, ch. 10) for LVT/EOI semantics and IPI delivery. Recommend implementing in phases: (1) spinlocks + protect R4_TASKS, (2) per-CPU GDT/TSS/IDT, (3) LAPIC timer per CPU, (4) IPIs, (5) per-CPU scheduler.
```


### Memory: mmap/brk, swap, huge pages, guard pages, user malloc

**Status today / prerequisites** — Demand paging is live (memory_v1 contract); the DEMAND_BASE window [0x0100_0000, 0x0200_0000) auto-maps on page fault via try_demand_map (mm.rs ~414). Per-task stacks in slots ≥5 use demand-paged strides from DEMAND_STACK_BASE (~2804 each, guarded by 16 KiB unmappable zones) with guard enforcement already in place (mm.rs ~421). TinyGo userspace has a bump heap in DEMAND_BASE+0x110_0000 with atomic xadd. Current rlibc malloc (rlibc.c ~92) is a bump allocator (sbrk-backed, no free). R4Task is heap-backed (lib.rs ~2682, R4_MAX_TASKS=32 in go_test). Prerequisites: establish per-task address-space tracking to enable swappable mappings (today USER_PML4 is shared; mmap/brk require independent address spaces or a COW fork mechanism). Carry-forward: read-only user code pages, NX on data (memory_v1).

**Design for Rugo** — Add three syscalls (50=sys_mmap, 51=sys_brk, 52=sys_munmap) to the v3.2 window (48..63, currently 50-63 free). mmap(va, sz, prot, flags) allocates anonymous heap in the demand window and installs PTEs; brk(new_brk_addr) extends/shrinks a per-task bump boundary; munmap(va, sz) clears and frees pages. All three route to a new map.rs module with a per-task VMA list (stored in a Vec inside R4Task) tracking mapped ranges, permissions, and backing (anonymous vs. file-backed for later). Demand-mapped pages from try_demand_map remain automatic, now attributed to the implicit [DEMAND_BASE, heap_brk) reservation. Swap is deferred (v3.3 or v4); huge pages use a MAP_HUGE flag (prot bits 24..31) decoded in try_demand_map to set PD PS bit if size ≥ 2 MiB and aligned. Guard pages are pre-installed by mmap(prot=0) — the demand mapper rejects faults in unmapped zones just as it rejects DEMAND_STACK_GUARD. Real malloc in rlibc sits on top of mmap/brk: malloc calls mmap for large allocations (>1 KiB), sbrk for small; free merges and returns to the arena.

**Concrete changes** —

- **kernel_rs/src/map.rs** (new): Per-task VMA (virtual memory area) list: struct VMA { va: u64, sz: u64, prot: u64, flags: u64, file: Option<usize>, off: u64 }. Per-task heap_brk u64 field. Helper alloc_and_install_pages(tid, va, sz, prot, huge) that allocates from PMM, sets PTE bits (NX if !(prot & EXEC), PS if huge), fills DEMAND_MAPPED quota. Return E_RANGE if outside window or over quota, E_FAULT on bad ptr, E_INVAL on alignment.

- **kernel_rs/src/lib.rs**: Add to R4Task (lib.rs ~2607): vmas: Vec<VMA>, heap_brk: u64. Initialize to empty vec and DEMAND_BASE in r4_init_task (~2716).

- **kernel_rs/src/mm.rs**: Refactor try_demand_map (~414) to check if the page falls in a VMA or the implicit [DEMAND_BASE, heap_brk) and match prot bits. Return false if prot=0 (guard). Pre-calculate huge-page eligibility from VMA.prot bits 24..31; if set and aligned, use PD PS=1 instead of PT entries.

- **kernel_rs/src/syscall.rs**: Add dispatch entries (go_test only):
  ```
  50 => sys_mmap_v1(arg1, arg2, arg3, arg4, arg5, arg6) [arg4=r10, arg5=r8, arg6=r9]
  51 => sys_brk_v1(arg1)
  52 => sys_munmap_v1(arg1, arg2)
  ```

- **libc/rlibc.c**: Extend with sys_mmap, sys_brk, sys_munmap wrappers. Rewrite malloc: segregate into "small" (<1 KiB, bump from sbrk) and "large" (mmap directly). Implement free via mmap metadata + lazy coalescing; calls munmap on release.

- **Syscall IDs**: 50=sys_mmap, 51=sys_brk, 52=sys_munmap (v3.2 window 48..63).

**ABI additions** —

| ID | Name | Args | Return | Error classes |
|---|---|---|---|---|
| 50 | `sys_mmap` | va, sz, prot, flags, (reserved, reserved) | va (or -1) | E_INVAL (align), E_RANGE (outside window, quota), E_FAULT (bad ptr), E_AGAIN (no frames) |
| 51 | `sys_brk` | new_brk_addr | old_brk_addr | E_INVAL (< base), E_RANGE (> window end) |
| 52 | `sys_munmap` | va, sz | 0 or -1 | E_INVAL (align), E_FAULT (not mapped) |

prot bits: 1=R, 2=W, 4=X; bits 24..31 encode huge-page class (1=2MiB hint, 2=1GiB hint); flags: 1=ANON, 2=SHARED, 4=FIXED.

Update docs/abi/syscall_v3.md to allocate 50, 51, 52 in v3.2 window and define prot/flags encoding.

**Acceptance** — `make test-mmap-v1-go` (new Makefile target, or add to go_test suite). Test fixture tests/runtime/test_mmap_v1.py boots via qemu_go_c4_runtime:
1. Allocate 1 page via mmap(0x0110_5000, 0x1000, 3, 1); assert SERIAL markers `MM: mmap va=0x01105000 sz=0x1000` and `MM: pte install va=0x...`. Touch page (read/write), verify zeroed and writable.
2. Extend heap with brk(0x0120_0000); read/write across boundary.
3. Allocate 2 MiB via mmap(0x0130_0000, 0x200_000, 3, 5) [flags=ANON|HUGE_2M]; assert `MM: pde ps set` (PD PS bit) and single fault.
4. munmap(0x01105000, 0x1000); re-fault must fail or allocate fresh frame.
5. malloc/free cycle: 100 allocations across small/large boundary, assert all freed and reused.
6. Spawn a child task; verify independent heap_brk (brk in child does not affect parent).

Markers:
- `MM: mmap va=0x<va> sz=0x<sz>` (all mmap calls)
- `MM: huge 2mib` or `MM: huge 1gib` (huge-page setup)
- `MM: pde ps set` (PD PS bit installed)
- `MM: munmap va=0x<va> freed 0x<frames>` (frame count)
- `MALLOC: alloc 0x<sz> -> 0x<ptr>` / `MALLOC: free 0x<ptr>` (stdlib)

Contract doc docs/runtime/memory_v1_mmap_brk.md covers: virtual address layout, per-task VMA semantics, prot bits and huge-page encoding, guard-page semantics (prot=0), quota enforcement (DEMAND_MAX_FRAMES), mmap/munmap atomicity within a fault, brk boundary crossing, malloc arena layout, free-list coalescing.

**Gotchas** —

- **Single USER_PML4**: Today all tasks share one PML4 (lib.rs ~1547); mmap in one task makes pages visible to all. Correct design is per-task pml4 (copy PDPT/PD/PT per task, kernel mappings cloned in); deferred (called out as v4 decision in ABI doc). For v3.2, document that mmap is task-local in spec but enforce it at test time with isolation tests.

- **DEMAND_MAPPED quota**: Shared across all tasks; mmap in TinyGo + malloc in a spawned app can collide. Add per-task quota field (R4Task) and split DEMAND_MAX_FRAMES; reject with E_RANGE when task quota exhausted.

- **Size budget**: rlibc is part of gousr.bin (28 KiB cap). mmap/brk/munmap syscall stubs are ~100 bytes each; malloc rewrite to ~200 lines is ~3 KiB compiled. Verify `size out/gousr.bin` post-build; if over budget, move large malloc to a separate service.

- **Marker splicing**: test_mmap_v1.py must exclude demand-mapped TinyGo heap pages (va ≥ 0x0110_0000 to 0x0120_0000) from the test's page-count assertions, as noted in memory_v1.md. Use `head_count` on raw SERIAL before heap pages arrive.

- **Guard-zone alignment**: DEMAND_STACK_GUARD is 16 KiB; a task's stack guard starts at (slot*stride + guard_base) and must never be faulted. mmap must refuse to allocate va that falls in [DEMAND_STACK_BASE, DEMAND_STACK_BASE+R4_MAX_TASKS*DEMAND_STACK_STRIDE) when prot != 0, and allow prot=0 only for explicit guard setup.

- **Swap deferred**: do NOT implement page-out to disk in v3.2. DEMAND_MAX_FRAMES stays 2048 (8 MiB). Document in ABI that swap is reserved for v4 and quota will not increase without major refactoring.

**Effort / risk** — **L** (500–700 lines Rust + 150 C). Risk is moderate: per-task address space unification (shared PML4 breaks isolation assumptions) and malloc correctness (free-list coalescing can silently corrupt if VMA tracking mismatches PT state). Mitigate with strict boot-time test assertions (demand quota tracking, malloc reuse verification) and add an `mmap_sanity_check` kernel function to verify VMA consistency against live PTEs on context switch (debug build only).



# Part II — Core subsystems


### Filesystem maturity: journaling, cache, mounts, a 2nd FS, partitions, /proc-/dev

**Status today / prerequisites** — SimpleFS v2 (vfs.rs ~2-548) provides writable on-disk files and directories over a single 512 KiB region (sectors 512–1023 on the boot disk). The VFS layer supports open/read/write/mkdir/unlink with uid-based permissions, but lacks durability guarantees (write-through only, no journal), zero block/page caching (direct sector I/O in vfs_read/vfs_write), single-mount semantics (/data is hard-coded), and no real partitioning. The block_io_dispatch (~storage.rs ~52, lib.rs) dispatches 512-byte sectors to either ATA or NVMe; sectors 8–11 are reserved for runtime state (R4 storage), and sector 512+ is VFS. A journaling subsystem must land first to support a write-ahead journal for crash consistency; then build cache and multi-mount infra on top.

**Design for Rugo** — adopt a three-phase evolution respecting the existing heap and FD routing:

1. **Crash-consistent writes**: Write-Ahead Journal (WAJ) in vfs.rs ~400: before mutating the VFS (node table, bitmap, file content), serialize the redo intent to a dedicated journal sector (e.g. sector 506, leaving 507–511 free). On a write syscall, copy nodes/bitmap deltas into a journal buffer, flush the journal sector atomically, then apply mutations. On boot, vfs_mount() at lib.rs ~419 checks for a non-empty journal and replays it before loading the node table. This is identical to the r4_storage recovery pattern in storage.rs ~314–357 (r4_storage_boot_recover, R4_STORAGE_JOURNAL_MAGIC, flags=1 to mark in-flight).

2. **Page/buffer cache**: Introduce a block cache layer (cache.rs ~new) — a 4-entry LRU backed by heap-allocated 512-byte buffers. Lift all sector I/O through a cache_read(sector) and cache_write(sector, buf, dirty_bit) interface. When an entry is evicted, flush it to disk if dirty. vfs_read and vfs_write call cache_read/cache_write instead of block_io_dispatch directly. This caps memory overhead at ~2 KiB and halves typical I/O on repeated access.

3. **Mount table and multi-FS routing**: A mount_registry (vfs.rs ~new, 8-entry table: path → (fs_type, fs_root_block, fs_private)). At open time (sys_open_v1 ~588), parse the path prefix to find the mount point, then dispatch (via fn pointers or a match on fs_type) to the mounted FS handler. v1 registers /data → SimpleFS and /sys → a pseudo-FS factory. Later mounts (/mnt/usb → FAT, etc.) register dynamically.

4. **FAT12/16 read-only support**: Build fat.rs (~200 lines, no external crates) — parse the boot sector for cluster size and FAT table location, cache the FAT in memory (~2 KiB for FAT12 on a 1.44 MiB floppy), and implement find_cluster(filename) → data_sector. Wire it as mount handler for sys_open on /mnt paths. Write-support defers to v4 (complexity of cluster chains, allocation, defrag).

5. **MBR/GPT partition discovery**: At block_driver_probe (lib.rs ~409), after driver init, call block_discover_partitions() to read sector 0 (MBR), parse partition entries, and expose each as a synthetic block device fd (e.g. /dev/sda1). Store partition offsets in a partition_table array (64-byte entries). sys_blk_read dispatches to the correct partition's data region via partition_table[dev_id].offset.

6. **Pseudo-filesystems**: Introduce pseudo-fs.rs (~new, shared by /proc, /dev, /tmp):
   - **/proc**: task listing (ps output formatted as files /proc/\<tid\>/stat, /proc/\<tid\>/status). Implement on-demand via a pseudo_fs_read handler that generates content for \<tid\>/\* on each read.
   - **/dev**: character devices (console, null, urandom, zero). Console reads from kb_read (existing), writes to serial_write. Urandom seeds from r4_storage recovery timestamp or a xorshift PRNG in lib.rs.
   - **/tmp**: a 256 KiB tmpfs (heap-backed, lost on reboot). Reuse vfs.rs node/bitmap logic in a temporary in-memory tree; sync_tmpfs_to_heap on each write instead of block I/O.

**Concrete changes**:
- **vfs.rs**: add journal sector (507) and JOURNAL_MAGIC (0x4A524E31); insert journal buffer (480 bytes) into VfsState; wrap vfs_write/vfs_mkdir/vfs_unlink with journal_begin/journal_commit helpers (record redo ops). vfs_mount calls replay_journal at line 188.
- **cache.rs** (new): struct BlockCacheEntry { sector, buf [512], dirty }, cache_lru (4 entries, heap-backed). cache_read(sector) and cache_write(sector, &buf) with eviction on LRU age. No associative lookup—a simple linear table.
- **vfs.rs** extend: move read_sector/write_sector → cache_read/cache_write. Adjust vfs_mount to flush cache at line 235.
- **mount.rs** (new): struct Mount { path_prefix [32], fs_type u8 (0=SimpleFS, 1=FAT, 2=Pseudo), root_block u32, priv u64 }, MAX_MOUNTS=8. mount_lookup(path) → Option<&Mount>. Wire into sys_open_v1 ~588.
- **fat.rs** (new): fat_find_root(partition_sector) → root_dir_sector; fat_lookup(cluster_num) → next_cluster_num; fat_stat(filename) → (size, start_cluster). No write.
- **storage.rs** (~409): block_discover_partitions() → reads sector 0, fills partition_table[16] with MBR entries (offset, size); calls sys_blk_read dispatcher to add +partition_table[id].offset to sector addr.
- **pseudo_fs.rs** (new): pseudo_stat(path, &uid) → (kind, size); pseudo_read(path, offset, &buf, &uid) for /proc/\<tid\>/stat (format "tid=<n> state=<s> rss=<pages>") and /dev/null/zero/urandom reads. Generate on-the-fly, no disk I/O.
- **lib.rs**: update m8_path_matches and sys_open_v1 to route /dev/\*, /proc/\* → pseudo_fs handlers via mount_lookup. Add M8FdKind variants (ProcFile, DevNull, DevUrandom, TmpFile).
- **docs/abi/**: update syscall_v3.md to document sys_fs_ctl op 4 (mount path fs_type root_block). Allocate sys_id 50 (next free in 48–63 window) as sys_mount_v1 if mount ops require a dedicated syscall; else fold into sys_fs_ctl.

**ABI additions**:
- sys_fs_ctl op 4 (mount path root_block): mounts a block device region (FAT or other) at a user-specified prefix. Requires R4_TASK_CAP_STORAGE. Returns 0 or -1.
- Syscall ID 50 reserved (sys_mount_v1) if multi-arg mount config proves too cramped for sys_fs_ctl ops; else v3.x additive.
- Partition table: standard MBR layout (sector 0, offsets 446–509), no new contract needed.
- /proc, /dev, /tmp paths documented in a new docs/runtime/pseudo_fs_v1.md contract.

**Acceptance**:
- `make test-vfs-journal-v1`: writes a file, injects a crash (qemu halt) mid-write, then reboots and verifies the file's old content is intact or the write replayed atomically. Test markers: "VFS: journal begin op=<op>", "VFS: journal flush ok", "VFS: replay ok" or "VFS: no replay needed".
- `make test-fat-readdir-v1`: mounts a FAT12 USB image at /mnt/usb, reads /mnt/usb/README.TXT, verifies content. Markers: "FAT: mount ok clusters=<n>", "FAT: lookup ok size=<bytes>", file content echoed.
- `make test-proc-stat-v1`: reads /proc/1/stat and /proc/self/stat, verifies format matches "tid=\d+ state=\S+ rss=\d+". Markers: "PROC: read ok", stat line present.
- `make test-mount-multi-v1`: mounts SimpleFS at /data and FAT at /mnt/usb, reads from both in sequence, checks isolation. Markers: "MOUNT: registered /data 0x<addr>", "MOUNT: registered /mnt/usb 0x<addr>", reads succeed.
- Contract doc: docs/runtime/vfs_maturity_v1.md covering journal recovery, cache coherency (flush-on-evict guarantees), mount semantics, FAT parsing, partition offsets, pseudo-FS generation (no seeding).

**Gotchas**:
- **Journal replay idempotence**: if a journal entry is replayed twice (e.g., boot hangs after fsync but before journal clear), the mutation must be safe to re-apply. Use sequence numbers (like r4_storage_seq in storage.rs ~34) to detect repeated replays.
- **Cache flush on eviction**: if a dirty cache entry is evicted during a FAT read (e.g., reading /mnt/usb/big.bin while SimpleFS is being written), the FAT data might stale. Enforce cache_flush_all() before cross-FS syscalls, or tag cache entries with mount_id to isolate.
- **Partition offset arithmetic**: block_io_dispatch currently takes absolute sector numbers. Wrap it with a partition-aware dispatcher that adds partition_table[dev_id].offset before calling the driver. Ensure sys_blk_read args are user-facing offsets (0 = first sector of the partition), not disk-absolute.
- **Size budgets**: the Go image is 8 code pages (32 KiB). cache.rs (2 KiB) + mount.rs (512 bytes) + fat.rs (3 KiB) + pseudo_fs.rs (2 KiB) = ~7.5 KiB, leaving headroom. The journal buffer (480 bytes in VfsState) is acceptable.
- **Single-occupancy caveat**: exec window (0x0140_0000–0x0180_0000) is still single-occupancy. Reading /mnt/usb/large_binary.elf doesn't preempt exec; user code must buffer it or spawn multiple times.
- **Marker splicing in pseudo-FS**: /proc and /dev reads are generated on-the-fly. If test_proc_stat_v1 spawns concurrent tasks that dump /proc/\<tid\> in a loop, stat lines may interleave. Use serial_write for atomic markers (e.g., "PROC: read ok <tid>=\d+\n"), not echoed user output.
- **Reserved sectors**: sectors 8–11 (r4_storage), 512–517 (SimpleFS superblock/nodes), 506–507 (reserved for journal in this design). Document in storage.rs top-of-file comment.

**Effort / risk**: **L/XL**. Journal (M), cache layer (M), mount table (S), FAT (L due to cluster chaining logic), MBR parsing (S), pseudo-FS (M due to multi-path codegen). Risk: cache coherency bugs if mount isolation fails; journal replay logic if sequence handling is off; FAT cluster chain traversal if the FAT table is cached incorrectly. Mitigate with boot-verified test suite (test_vfs_journal_v1, test_fat_readdir_v1, test_mount_multi_v1) that exercises normal and crash/recovery paths.


### Networking maturity: TCP retransmit/listeners/multi-conn, DORA, ICMP, routing, IPv6, loopback

**Status today / prerequisites**

Wire TCP (gap-analysis item 6) is live: `tcp.rs` implements one outbound IPv4 connection over VirtIO (SYN handshake → ACK, PSH/ACK data both ways, FIN/RST teardown). DHCP and DNS clients (both `netcfg.rs`, DISCOVER + A query) complete the item's named scope. Both run on the PIT-tick RX pump (`net_rx_pump` ~190 in `net.rs`, 8-frame budget per tick). The socket table (`R4_SOCKETS[16]`, `net.rs` ~563) already holds listener state + accept-rendezvous logic for loopback AF_INET6. No prerequisite sections; this grows the existing transport layer in-place.

**Design for Rugo**

Grow from single-occupancy TCP to a connection table (one live outbound connection per socket, up to 16 concurrent), adding:

1. **Per-connection state tracking**: Replace the global `CONN` (tcp.rs line 37) with entries in `R4_SOCKETS[sid].state == 8` for wire TCP (currently diverted in `sys_socket_connect_r4` line 935). Each socket tracks: local/remote IP/port, send/recv sequence numbers, RX buffering (256 bytes per socket, up from 1 KiB shared), and retransmit timer metadata.

2. **Retransmission + RTO**: Add exponential backoff timers indexed by socket. Clock them on PIT ticks (same frequency as RX pump). Retransmit SYN if ACK is pending; retransmit data if no ACK within timeout window. Keep the QEMU loss-free model as acceptance baseline but seed the timer logic for real-world LANs.

3. **Send/recv window management**: Track offered window (from TCP header), sent-but-unacked bytes, and flow-control state. `tcp_send` already blocks on state == 8 (line 1048 in net.rs); make it respect available window. `tcp_recv` must drain buffered data on demand.

4. **Listen/accept on the wire**: Extend `sys_socket_accept_r4` (line 1009) to accept incoming TCP connections. Today it only works for loopback AF_INET6 rendezvous. For AF_INET, demultiplex inbound SYN on (src_ip, src_port, dst_port) and allocate a socket in state 4 (established pending handshake completion). Move the current `tcp_input` (line 187) from single-connection match to listener lookup.

5. **Full DHCP DORA**: `netcfg.rs` currently sends DISCOVER only (line 115) and extracts yiaddr from OFFER. Add REQUEST after OFFER to confirm lease; parse ACK with lease time and renewal timer. Background a task to refresh before expiry (e.g., at T+lease/2).

6. **ICMP echo (ping)**: Add `icmp.rs` with echo-request handler + responder. Demultiplex at `net_rx_pump` (line 225) on `ip[9] == 1` (IPPROTO_ICMP). Respond with echo-reply, preserving payload and identifier/sequence.

7. **Routing table + per-route lookup**: The infrastructure exists (`R4_NET_ROUTES[8]`, `net.rs` line 560, `r4_net_find_route` line 698). Extend `sys_socket_connect_r4` to consult routes before TCP connect, extracting the next-hop MAC via ARP.

8. **IPv6 on the wire**: Mirror TCP/UDP/ICMP logic for IPv6 (src/dst in 16-byte fields, different extension headers, ICMPv6 checksums). Loopback AF_INET6 is already plumbed; wire AF_INET6 is the carry-forward.

9. **Interrupt-driven RX vs. PIT polling**: Move from polling in `net_rx_pump` (line 194) to VirtIO RX queue interrupt (ISR bit in device status). Post an interrupt handler in `idt.rs` and replace the polling budget with event-driven reception. This reduces latency for high-packet-rate scenarios.

10. **Per-connection timers**: Build a separate timer wheel (e.g., 256-entry hash of socket IDs, firing on PIT ticks). Today all sockets share the global pump tick; split into per-socket RTO + keep-alive clocks for better scaling.

**Concrete changes**

- **`tcp.rs`**: Refactor global `CONN` into per-socket state hosted in `R4_SOCKETS[sid]`. Add fields `snd_una`, `snd_wnd` (remote window), `rcv_wnd`, `rto`, `retransmit_count`, `last_ack_time`. Rewrite `tcp_input` to accept (listener IP, port) and demux on inbound 5-tuple. Add `tcp_retransmit` timer-driven handler.

- **`net.rs`**: Extend `R4Socket` (line 517) with `snd_una` (u32), `snd_wnd` (u16), `retransmit_time` (u64), `state == 8` parsing (in-progress outbound wire TCP), and `state == 9` (inbound SYN received, awaiting ACK completion). Add `r4_net_wire_demux(src_ip, src_port, dst_port)` → Option<socket_id>. Extend `net_rx_pump` to call `r4_net_find_listener` and allocate on inbound SYN. Add per-tick retransmit loop in `net_rx_pump`.

- **`netcfg.rs`**: Add DHCP REQUEST state (Q_DHCP_REQUEST) after OFFER. Parse ACK for lease time; spawn background renewal timer (polled at op 3 or driven by PIT).

- **New `icmp.rs`**: Echo-request handler, checksum, response TX. Integrate into `net_rx_pump` demux.

- **Syscall ABI**: Allocate syscall id **50** (next free in v3 window 28..47) for `sys_socket_send_v2` (takes socket + window hints) or fold window state into `sys_socket_recv_r4` return (high bits = available window). If the return value scheme is too tight, reserve id **50** for future socket options. No new ops; existing socket calls remain backward-compatible.

- **Makefile**: Add `make test-tcp-v2` target (outbound multi-conn retransmit over lossy link simulator in QEMU) and `make test-tcp-server-v1` (listen + accept over the wire). Reuse `qemu_go_c4_runtime` fixture.

**ABI additions**

No new syscall IDs required if window management folds into existing calls. If a new info syscall is needed (e.g., to query per-socket RTO), allocate id **50** (v3 window 28..47 has slots 28..30 free after id 29 = `sys_sched_set_r4`); document in `docs/abi/syscall_v3.md`. The DHCP lease timer is internal to `netcfg.rs` and opaque to userspace (op 3 = poll result, no changes needed).

**Acceptance**

- `make test-tcp-v2`: Boots via `qemu_go_c4_runtime` with a host-side `iperf` or custom TCP echo server. Userspace sends 256 bytes, drops first SYN and retransmits after 100ms (simulated loss), verifies payload round-trip and `TCP: retransmit` markers on SERIAL. Contract in `docs/runtime/tcp_v2.md`.

- `make test-tcp-server-v1`: Listener binds on AF_INET 10.0.2.15:8000, host connects, userspace calls `sys_socket_accept`, reads "HELLO", replies "WORLD". Markers: `TCP: syn received`, `TCP: accepted`, payload verified. Contract in `docs/runtime/tcp_server_v1.md`.

- `make test-icmp-v1`: Host `ping 10.0.2.15`, kernel echoes. Marker `ICMP: echo reply tx`. Contract in `docs/runtime/icmp_v1.md`.

- `make test-dhcp-v2`: DISCOVER → OFFER → REQUEST → ACK flow; marker `DHCP: request sent`, `DHCP: ack received`. Contract in `docs/runtime/netcfg_v2.md`.

- Existing `make test-tcp-v1`, `test-netcfg-v1` remain unbroken (loopback AF_INET6 unchanged; wire TCP single-connection path preserved with retransmit disabled at baseline).

**Gotchas**

- **Single-occupancy exec window collision**: Wire TCP holds `EXEC_APP_TID` socket during the connection (state 8). If userspace spawns a thread, that thread cannot open wire sockets (only loopback AF_INET6). Document the 1:1 mapping in the feature gate.

- **Socket table indexing**: Today `sys_socket_connect_r4` (line 935) checks `!tcp_connect()` and diverts; if two tasks race to open wire AF_INET sockets, the second fails. This is correct (one wire socket per kernel execution window), but errors are silent. Add diagnostics.

- **Retransmit + loopback rendezvous**: The AF_INET6 loopback accept path (line 955, `r4_net_find_listener`) must not collide with wire TCP retransmit timers. Separate the two: wire TCP runs in interrupt/RX context (IF=0); loopback is synchronous inside syscalls. No race, but document isolation.

- **RX buffer size budget**: Each socket now buffers 256 bytes (up from none for wire sockets). 16 sockets × 256 = 4 KiB; well under the 32 KiB heap budget. ICMP echoes add negligible overhead (no buffering, immediate reply).

- **Marker splicing on retransmit**: Serial output `TCP: retransmit` must not splice with userspace echo. Ensure kernel timestamps all markers and test harness anchors on exact newlines (never on prompt).

- **Loss simulation**: QEMU's slirp is loss-free by default. To test retransmit, either use `tc` (traffic control) on the host or add a probabilistic drop in QEMU's `-net` config. The test must be repeatable (seed the RNG if needed).

- **IPv6 scoping**: Loopback AF_INET6 uses hardcoded 127.0.0.1 and ::1; wire AF_INET6 needs link-local address assignment (DHCPv6 or stateless autoconfiguration). Defer to v3 (this is IPv6-on-wire, listed as carry-forward).

**Effort / risk**

**L (large, 2–3 weeks)**: 

- Retransmit + RTO logic is correctness-critical; test against synthetic loss and real timeouts.
- Listener demux adds complexity to `tcp_input` state machine (currently simple linear FSM for one connection).
- Interrupt-driven RX is a separate project (refactor `virtio_net_recv`); can ship without it (polling is acceptable for QEMU's latency).
- DHCP lease timer requires background task or piggyback on PIT (the latter is simpler and acceptable).

**Risk**: Retransmit timeout tuning under QEMU emulation variability; loopback/wire collision if AF_INET6 listen logic regresses; socket table exhaustion if sockets leak (add audit trail). Mitigate with detailed tracing markers and pre-landing socket leak test.


### Driver model + buses: device registry, PCIe ECAM, USB/XHCI+HID, DMA, e1000

**Status today / prerequisites**

Rugo currently probes PCI bus 0 via raw config-space reads (lib.rs:4364–4436, `pci_read32`/`pci_write32` using fixed I/O ports 0xCF8–0xCFF). Only two device classes are hardcoded: virtio-blk and virtio-net (legacy I/O transport, BAR0 I/O-space probe). A proof-of-concept NVMe driver lives in `runtime/native.rs` with its own probe path and MSI/X setup. The interrupt controller is the legacy 8259A PIC (sched.rs:17–27, vectors remapped to 32–47) with PS/2 keyboard polling via `kbd.rs` (IRQ1 unmasked, but console loop polls directly because interrupts are masked). There is no abstraction: each driver manages its own claim slots (`PCI_CLAIMED`, lib.rs:4410), static state, and MMIO/DMA buffers.

**Design for Rugo**

This section lands BEFORE implementing XHCI, HID, or e1000 drivers, and refactors the existing hardcoded probes into a unified driver model:

1. **Device registry**: a static array of `struct Device` (bus/dev/func BDF, vendor/device IDs, driver name) populated at boot by a PCIe enumeration loop (`pci_enumerate_bus`). A global claim bitmap replaces scattered `PCI_CLAIMED` slot tables.

2. **Driver registry + probe contract**: a static `struct DriverProbe { name, vendor_mask, device_list, probe_fn }` array. The init loop calls each registered driver's `probe_fn(bdf)`, returning a driver-specific opaque handle. Drivers register once at boot, never per-probe. This eliminates the ad-hoc vendor/device ID matching in lib.rs:4426–4436.

3. **Interrupt model**: extend `pic_unmask` to accept a generic interrupt source (IRQ, MSI/X vector) and store per-driver IRQ-to-handler routing in a small static table (`struct IrqHandler { driver_id, fn_ptr }`). MSI/X setup is deferred to the probe path; legacy IRQ enumeration is automatic.

4. **DMA allocator**: a kernel-side pool (`dma_alloc(size) -> phys_addr`) in mm.rs that carves pages from the PMM bitmap and returns physical addresses to drivers. Drivers pass `kv2p_delta` at init time (one value, kernel-wide, lib.rs:1547).

5. **Virtio and NVMe refactoring**: both become registered drivers. The hardcoded `pci_find_virtio_blk()` call (lib.rs:4474) is replaced by the init loop discovering and attaching them by vendor/device ID. NVMe moves its probe path (`runtime::native::probe_nvme`) into kernel code (new file `kernel_rs/src/nvme_driver.rs`).

Why this design:
- **Single entry point**: all device discovery and attachment happens in one deterministic sequence (no hidden dependencies).
- **Proof by boot**: each driver's lifecycle (probe, attach, claim) is visible in serial markers.
- **Future drivers**: USB/XHCI, e1000, and higher-order buses (PCIe ECAM, IOMMU) plug into the registry without code duplication.
- **Interrupt handling**: a small driver-id → handler map lets the pit/kbd IRQ entry point dispatch to the correct driver without conditional logic.

**Concrete changes**

- **kernel_rs/src/lib.rs**:
  - Relocate `pci_read32`, `pci_write32`, `pci_bar0_iobase` to a new module `kernel_rs/src/pci.rs` (public).
  - Remove `pci_find_device`, `pci_claim_device`, `PCI_CLAIMED` array; add `struct Device { bdf: PciBdf, vendor: u16, device: u16, driver_name: &'static [u8], claim_owner: u8 }` (claim_owner = 0 = unclaimed).
  - Add `static mut DEVICE_REGISTRY: [Device; 32]` and `static mut DEVICE_COUNT: usize`.
  - Add `unsafe fn pci_enumerate_bus(bus: u8, registry: &mut [Device])` — scans bus 0 (qemu), populates vendor/device, returns count.
  - Add `struct DriverProbe { name: &'static [u8], vendor: u16, device_ids: &'static [u16], probe: unsafe fn(bdf: PciBdf) -> bool }` and `static DRIVER_REGISTRY: [DriverProbe; 4]` (virtio-blk, virtio-net, NVMe, placeholder for e1000).
  - Add `unsafe fn drivers_probe_all()` — iterates devices, matches drivers, calls probe_fn, sets claim_owner on success.
  - Call `pci_enumerate_bus(0, &mut DEVICE_REGISTRY)` and `drivers_probe_all()` in `entry_rust()` before any hardcoded device init (right after `pic_init()`).
  - Emit serial markers: `PROBE: <bus>:<dev>:<func> vendor=0x<vid> device=0x<did>`, `ATTACH: <driver_name>`.

- **kernel_rs/src/pci.rs** (new):
  - Expose `pci_read32`, `pci_write32` (move from lib.rs).
  - Add public `fn pci_device_class(bdf: PciBdf) -> (u8, u8, u8)` — reads class/subclass/progif.
  - Add public `fn pci_bar_read(bdf: PciBdf, bar_idx: u8) -> u32` (wraps offset 0x10 + 4*idx).

- **kernel_rs/src/dma.rs** (new, ~50 lines):
  - `static mut DMA_POOL_BITMAP: [u8; 128]` (tracks 1024 pages = 4 MiB DMA pool).
  - `unsafe fn dma_alloc(pages: usize) -> Option<u64>` — finds contiguous free pages in bitmap, returns phys addr.
  - `unsafe fn dma_free(phys: u64, pages: usize)`.
  - Drivers must call `dma_alloc` for vring/descriptor rings; initialize with physical address.

- **kernel_rs/src/mm.rs**:
  - Reserve pages [0xFFFFFFFFF0000000..0xFFFFFFFFF0400000] for the DMA pool (phys 0x0..0x400000 via HHDM). Update comments.

- **kernel_rs/src/nvme_driver.rs** (new, extracted from runtime/native.rs):
  - `pub struct NvmeDriver { bdf: PciBdf, mmio_base: u64, io_qid: u16 }`.
  - `pub unsafe fn nvme_probe(bdf: PciBdf) -> bool` — uses `pci_bar_read`, `dma_alloc` instead of static buffers.
  - Reuse the command/response loop from native.rs, but store state in a heap-backed driver struct.

- **kernel_rs/src/lib.rs** (refactor existing virtio):
  - `unsafe fn virtio_probe(bdf: PciBdf, device_id: u16) -> bool` — called by `DRIVER_REGISTRY` entry.
  - Keep existing `blk_kv2p`, `block_read`, `block_write` but remove hardcoded `pci_find_virtio_blk()`.
  - Update `block_driver_probe()` (lib.rs:4600) to check if a virtio driver was already attached (via registry).

- **kernel_rs/src/sched.rs**:
  - Add `static mut IRQ_HANDLERS: [(u8, u32); 8]` mapping (irq_num, driver_id).
  - Export `pub fn irq_dispatch(irq: u8)` — called by trap.rs IRQ entry, looks up handler.

- **kernel_rs/src/trap.rs**:
  - In the IRQ32–47 entry points, call `sched::irq_dispatch(irq_num)` instead of hardcoded `kbd_irq()` on IRQ1.

- **docs/abi/syscall_v3.md**: no change (device discovery is kernel-internal, not a syscall).

**ABI additions**

None — driver probing is kernel-internal boot-time behavior. No new syscalls required. The design leaves room for a future `sys_device_query` (id 50) to enumerate devices to userspace, but that is out of scope here.

**Acceptance**

`make test-drivers-v1` (new target in Makefile, `mingw32-make test-drivers-v1`):

```makefile
build-drivers: $(ASM_OBJS) boot/linker.ld $(GO_USER_BIN)
	cd kernel_rs && $(CARGO) build --release --features go_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-drivers.elf $(ASM_OBJS) $(KERNEL_LIB)

image-drivers: build-drivers $(APP_ELFS)
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-drivers.elf ISO_NAME=os-drivers.iso bash tools/mkimage.sh

test-drivers-v1: image-drivers
	pytest tests/runtime/test_drivers_v1.py -xvs -k qemu_go_c4_runtime
```

**test_drivers_v1.py** (new, tests/runtime/):

```python
def test_device_registry_and_probe(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime
    out = boot("shutdown\n").stdout
    
    find_in_order(out, [
        "PROBE: 0:00:00 vendor=0x1AF4 device=0x1001",  # virtio-blk
        "ATTACH: virtio-blk-pci",
        "PROBE: 0:00:01 vendor=0x1AF4 device=0x1000",  # virtio-net
        "ATTACH: virtio-net-pci",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok"
    ])
    assert "PROBE:" in out
    assert out.count("ATTACH:") >= 2
```

**docs/runtime/driver_model_v1.md** (new contract):

```markdown
# Driver Model Contract v1

Status: boot-verified via make test-drivers-v1
Source: kernel_rs/src/{pci.rs, dma.rs, lib.rs}, kernel_rs/src/nvme_driver.rs
Proof: tests/runtime/test_drivers_v1.py

Device enumeration, driver probe/attach, claim semantics, and DMA allocation.

## Probe lifecycle

1. Boot: pci_enumerate_bus(0) scans 32 devices, populates DEVICE_REGISTRY
2. Each driver's probe_fn(bdf) is called in-order; returns true=claimed
3. Claim semantics: one driver per BDF, stored in Device.claim_owner

## Marker contract

| Marker | Format |
|--------|--------|
| PROBE | `PROBE: <bus>:<dev>:<func> vendor=0x<vid> device=0x<did>` |
| ATTACH | `ATTACH: <driver_name>` |

## DMA contract

Drivers call dma_alloc(num_pages) at probe time; physical address is returned.
No fragmentation handling (reserved pool is 4 MiB max).
```

**Gotchas**

- **ECAM later**: this design scans config-space I/O ports only (no ECAM BAR). Multi-bus or hotplug requires ECAM enum; that's a separate section.
- **Claim atomicity**: the registry is populated once at boot with interrupts off. Hot-removal/reinsertion requires a runtime claim API (out of scope).
- **IRQ/MSI dispatch**: the `IRQ_HANDLERS` table is tiny (8 entries). e1000 with MSI/X will need a larger routing table or a hash; plan ahead.
- **DMA pool size**: 4 MiB is hardcoded. A device requiring >1 MiB descriptors will fail gracefully (dma_alloc returns None), not silently.
- **NVMe moved to kernel**: moving `runtime::native` code into `nvme_driver.rs` requires copy-paste + refactoring to use the new dma_alloc API. Test it under go_test before removing native.rs entirely.
- **Virtio refactor risk**: the existing `block_read` and `block_write` rely on static buffer addresses. Once they're allocated from the DMA pool, ensure all kv2p_delta math is preserved (it is: single shared delta).

**Effort / risk**

**Size**: M (400–500 lines new/refactored code).  
**Risk**: Medium.

- The registry pattern is straightforward (no hash tables, no allocator).
- Virtio/NVMe refactoring is mechanical (move static bufs → dma_alloc, update addresses).
- Interrupt dispatch adds indirection but no new IRQ logic.
- Main hazard: if a driver probe_fn modifies global state and then returns false, the kernel may have half-initialized state; mitigation = early return on any probe failure, no cleanup needed (devices are independent at this stage).



# Part III — Human interface


### Input, graphics stack, and audio

**Status today / prerequisites** — Framebuffer text console and PS/2 keyboard are implemented (`kernel_rs/src/fb.rs`, `kernel_rs/src/kbd.rs`, tested via `make test-console-v1`). This section builds a three-part human-interface subsystem: input event dispatch for mouse and extended keyboard, a real 2D graphics layer with a minimal compositor, and audio playback. All must land in the `go_test` lane (the unified kernel) and be boot-verified.

**Design for Rugo** — Input: extend `kbd.rs` with USB HID mouse support (virtio-input or emulated PS/2 mouse via 0x60/0x64), unfold extended scancodes (0xE0 prefix for arrows, numpad), and push decoded events into a kernel ring buffer that userspace reads via `sys_input_poll_v2` (new, id 50 from the 48–63 window). Graphics: replace `fb_write`'s character-at-a-time raster with a double-buffered linear framebuffer in a `compositor.rs` module; add a `gfx.rs` layer (PSF font loading, blitting, dirty-rect damage tracking) and a tiny `winsrv.rs` (window server) as a task offering IPC ports for draw commands (`sys_winsrv_create`, `sys_winsrv_blit`, `sys_winsrv_flip`); all via syscalls in the 50–63 window. Audio: a minimal AC97 or virtio-snd driver in `audio.rs` that feeds a ring buffer, with `sys_audio_write_v1` (id 51) queueing samples; no mixing needed initially — single exclusive stream. The key constraint is singleness: the shared `USER_PML4` (lib.rs ~1547) means windows cannot have private address spaces, so a global damage bitmap and a single ring buffer per subsystem (input, audio) are canonical. Acceptance is marker-driven: `make test-graphics-v1` boots and a screendump of drawn rectangles; `make test-input-v1` sends mouse clicks via QMP and asserts HID event bytes appeared in the ring buffer; `make test-audio-v1` writes samples and asserts DMA queued.

**Concrete changes**

- **Input subsystem:**
  - Extend `kernel_rs/src/kbd.rs`: add `struct MouseEvent { x: i16, y: i16, btn_l: bool, btn_r: bool }` and a shared `INPUT_EVENTS: [InputEvent; 256]` ring buffer with head/tail, unpacked from PS/2 mouse port 0x60 on IRQ12 or virtio-input queue.
  - Add extended scancode map: 0xE0 prefix codes for `UP`, `DOWN`, `LEFT`, `RIGHT` (0xE048, 0xE050, 0xE04B, 0xE04D).
  - Dispatch IRQ12 to a new `mouse_irq()` handler; expose `input_pop() -> Option<InputEvent>` (union of kbd byte + mouse event).
  - New file `kernel_rs/src/input.rs`: `pub fn sys_input_poll_v2(mask: u64) -> u64` — polls input ring for next event, returns opaque u64 event ID (encoded type + payload), or 0 if none. Syscall id 50.
  
- **Graphics subsystem:**
  - New file `kernel_rs/src/gfx.rs`: `struct Framebuffer { base: u64, width: u64, height: u64, pitch: u64, pixels: [u32] }` wrapping Limine's linear FB; `pub fn blit_rect(x: u64, y: u64, w: u64, h: u64, color: u32)` for solid-color fills; `pub fn load_psf_font(data: &[u8]) -> bool` (PSF2 header parsing). Keep embedded 8x8 as fallback.
  - New file `kernel_rs/src/compositor.rs`: `struct DamageRegion { x, y, w, h }` + a 32-entry damage list; `pub fn mark_dirty(x: u64, y: u64, w: u64, h: u64)` appends to damage list; `pub fn flush_damage()` redraws only dirty rects to the Limine FB (no double-buffering initially — single framebuffer with damage tracking). Marker: `GFX: damage <N>` for region count flushed.
  - New file `kernel_rs/src/winsrv.rs` (optional for M1): window server as a Go task spawned at boot; listens on an IPC endpoint; clients send draw-command packets (draw_rect, draw_text, flip). For now, just echo draw calls to serial and mark damage.
  - Modify `kernel_rs/src/lib.rs` to call `gfx::blit_rect` instead of the per-character `fb::draw_glyph` in the text console path (or keep both: fb for serial-mirrored output, gfx for higher-res UI).
  - New syscalls in id range 51–53: `sys_gfx_blit_v1` (arg1=x, arg2=y, arg3=w, r10=h, r8=color); `sys_gfx_mark_dirty_v1` (args as above); `sys_gfx_read_damage_v1` (return damage list as packed u64 quadruplets). All return -1 on out-of-bounds.
  
- **Audio subsystem:**
  - New file `kernel_rs/src/audio.rs`: detect AC97 or virtio-snd PCI device at boot; set up DMA ring. For AC97, write to port 0x100 (mixer) and 0x200 (bus master PCM out control). For virtio, allocate a buffer descriptor and queue via the device config.
  - Expose `struct AudioFrame { left: i16, right: i16 }` and a 4 KiB ring buffer (256 frames stereo).
  - `pub fn sys_audio_write_v1(ptr: u64, len: u64) -> u64` (syscall id 52): copy samples from user buffer into the ring; trigger DMA if the queue was empty. Return bytes written or -1 on fault.
  - Marker: `AUDIO: init <device>` (AC97 or virtio-snd), `AUDIO: queued <frames>`.

**ABI additions**

- Syscall id 50: `sys_input_poll_v2` (op 1 = poll keyboard, op 2 = poll mouse, op 3 = poll all; return u64 event or 0).
- Syscall id 51: `sys_gfx_blit_v1` (args: x, y, w, h, color; return -1 on error).
- Syscall id 52: `sys_gfx_mark_dirty_v1` (args: x, y, w, h; return 0).
- Syscall id 53: `sys_audio_write_v1` (args: user_ptr, len; return bytes written or -1).
- Update `docs/abi/syscall_v3.md` section "v3.2 expansion window (48–63)" to list these four new ids in the "allocated so far" table.
- New contract doc `docs/runtime/input_events_v1.md`: layout of InputEvent union (kbd_byte | mouse_x + mouse_y + buttons), marshalling via syscall 50.
- New contract doc `docs/runtime/graphics_v1.md`: gfx-syscall semantics, damage-region flushing, PSF font loading, pixel coordinate space.
- New contract doc `docs/runtime/audio_v1.md`: AC97 vs virtio-snd negotiation, sample format (s16 stereo LE), ring buffer semantics.

**Acceptance**

- **Input:** `make test-input-v1` — boots, QMP sends a mouse-click sequence (e.g., left-button at (640, 480)); userspace app calls `sys_input_poll_v2` in a loop and asserts at least one mouse event with matching x, y, button. Markers: `INPUT: on`, `INPUT: mouse-click x=<hex> y=<hex> btn=<hex>`.
- **Graphics:** `make test-graphics-v1` — boots; a userspace app calls `sys_gfx_blit_v1` to draw a 100x100 red rect at (100, 100), then `sys_gfx_mark_dirty_v1` for that region, then `sys_gfx_read_damage_v1` to confirm it's in the dirty list. Captures a QMP screendump and asserts >1000 red (0xFF0000) pixels in the [100, 200)² region. Markers: `GFX: init <res>`, `GFX: blit OK`.
- **Audio:** `make test-audio-v1` — boots; userspace writes 64 samples via `sys_audio_write_v1`; kernel forwards to AC97/virtio-snd and asserts the device's DMA pointer advanced. Serial markers: `AUDIO: init AC97`, `AUDIO: write OK <len>`, `AUDIO: dma-advance <frames>`.
- Test script `tests/runtime/test_input_v1.py`, `test_graphics_v1.py`, `test_audio_v1.py` (as per console_runtime_v1 pattern).

**Gotchas**

- **Scancode parsing:** Extended scancodes (0xE0 prefix, e.g., arrow keys) require state in `kbd.rs` to track the 0xE0 prefix and merge the next byte. The MAP table only handles 0x00–0x39; any code >= 0x3A must be silently dropped or handled as an extended sequence.
- **Mouse coordinate signaling:** PS/2 mouse sends three bytes (status, dx, dy) where dx/dy are signed. The status byte encodes which buttons are down. Decoding requires tracking state across IRQ1 calls; buffer three bytes before emitting an event.
- **Damage tracking and compositing atomicity:** Marking damage and flushing must not race with a task drawing to the framebuffer. Since the single `USER_PML4` means all tasks share the FB address, damage flushes must be serialized or protected by a spinlock. The simplest safe pattern: damage list is per-task-local (TLS) and `sys_gfx_mark_dirty` atomically appends; `sys_gfx_read_damage` in a loop drains it.
- **Framebuffer pixel format:** Limine v8 guarantees 32-bpp XRGB with masks in the framebuffer response struct. Endianness is little (0x00RRGGBB in memory); confirm color math before shipping.
- **Audio DMA deadlock:** If AC97 is not initialized fully (PCI BAR mapping, codec reset), writes to control registers will hang. Gate audio behind a successful PCI probe + codec-ready poll; if it times out, disable audio and return E_UNSUP from syscall.
- **Size budgets:** Input ring (256 events × 8 B = 2 KiB), damage list (32 rects × 16 B = 512 B), audio ring (4 KiB): total ~7 KiB kernel overhead. Fits easily in the ~32 KiB kernel budget. The window server (if added) will be a userspace task and counts against the single 64 KiB app slot (EXEC_APP_TID at 0x0140_0000).
- **Lane conformance:** All three subsystems must compile unconditionally in the go_test lane (no new feature gates, unlike console_v1 which has fallback in sched_test). Use `#[cfg(feature = "go_test")]` for the IRQ12 mouse handler, not a new test gate.
- **Limine adoption:** Ensure the framebuffer request in fb.rs is already live before gfx.rs tries to blit. Call `fb_init()` before `gfx_init()` in `kmain()`.

**Effort / risk** — **L (large)**. Input extends kbd.rs (~50 lines for mouse IRQ12 + 30 for extended scancodes); input.rs and gfx.rs are each ~100–150 lines of straightforward memory operations; audio.rs is ~150–200 lines (PCI probe, AC97 or virtio-snd boilerplate). Compositor.rs is ~50 lines (damage tracking). Three test scripts, each ~200 lines (similar to console_v1). Total delta: ~1.5 KLOC kernel + ~0.6 KLOC tests. Main risks: (1) mouse state machine (3-byte buffering) can drop events if IRQ timing is wrong; (2) damage atomicity under task preemption; (3) AC97 codec initialization varies by QEMU emulation version (rerun build if ld crashes mid-link). Recommend: develop input and gfx in parallel (independent), then audio; use git worktree for audio driver debugging (AC97 PCI config is finicky).



# Part IV — System services


### Time/timekeeping and power/ACPI

**Status today / prerequisites** — The kernel has a stub monotonic timer (go lane: incremented per `sys_time_now()` call; other lanes: return 0). Shutdown uses QEMU debug-exit (arch_x86.rs:46, port 0xF4). The PIT (8254) is initialized for preemptive scheduling (sched.rs:48–53, freq=100 Hz today) but clock-wall-clock, user timers (nanosleep/timerfd), wait queues, and real ACPI are absent. Services infrastructure exists (services/go/runtime.go defines phases: phaseCore, phaseServices, phaseSession, phaseShutdown) with boot-critical timesvc declared. Prerequisites: this section requires **nothing else** and ships standalone; TCP/netcfg (syscalls 31–40) should land first so DNS/SNTP clients can operate on the wire.

**Design for Rugo** — Split into two kernel syscalls and one boot service:
1. **sys_clock_gettime (v3.2 id=50)**: read CMOS RTC at boot (via port 0x70/0x71 and ACPI FADT if present) to initialize an NTP-synchronized wall-clock (struct kept in kernel, opaque to userspace). Userspace calls sys_clock_gettime(clockid, &timespec) to read monotonic or wall time; the monotonic tick increments on PIT IRQ (today it increments per syscall — move that to the IRQ handler). The implementation avoids ACPI parsing in ring 0 — the boot service discovers the RTC frequency and any ACPI descriptor table pointers, and stashes them in a shared memory region so the kernel can interrogate without parsing.
2. **sys_timerfd_create (v3.2 id=51)**: create a userspace timer FD backed by a kernel wait queue (new structure in the R4Task). When the timer expires, the task wakes; the shell loops with sys_poll including the FD. This is a prerequisite for nanosleep and cron-like scheduling in userspace without spinning.
3. **ACPI boot service**: a Go daemon (timesvc in services/go/) that runs in phaseCore. It reads the RSDP (physically at 0xE0000–0xFFFFF or via EFI pointer), parses RSDT/XSDT/FADT, extracts RTC address space (typically CMOS ports, sometimes MMIO), and broadcasts the result via IPC to init. It also enumerates S-state support (S5=shutdown, S4=hibernate, S1=light sleep) and battery/thermal objects (DSDT). Real shutdown/reboot replace qemu_exit with a real PM1a_CNT write or ACPI _OFF method invocation. All ACPI parsing is userspace-only; the kernel exposes a syscall (sys_acpi_cmd, v3.2 id=52) to reboot/shutdown (the service holds the FADT and issues the command on admin request).

**Concrete changes** —
- **kernel_rs/src/lib.rs**: Add a timekeeping subsystem:
  - `struct TimekeeperState { rtc_port: u16, wall_base: u64, wall_ticks: u64, ... }` (static mut behind `feature="go_test"`).
  - Expand `R4Task` with `timer_queue_head: u64` (linked list of pending timers per task; allocate from kernel heap).
  - Modify `sys_time_now()` (line 290) to return wall_base + (PIT ticks / freq), not a counter.
- **kernel_rs/src/arch_x86.rs**: Add `rdtsc() -> u64` inline asm and `cmos_read(reg: u8) -> u8` via ports 0x70/0x71.
- **kernel_rs/src/sched.rs**: In the PIT IRQ handler (add one if it doesn't exist in the go lane), increment a kernel-global tick counter and wake any expired timers from the task's queue.
- **kernel_rs/src/syscall.rs**: Add dispatch for syscalls 50, 51, 52 (go lane only, behind `feature="go_test"`):
  - `50` → `sys_clock_gettime(arg1: clockid, arg2: *timespec)` — read monotonic or wall clock.
  - `51` → `sys_timerfd_create(arg1: flags, arg2: *itimerspec)` — allocate FD, enqueue timer.
  - `52` → `sys_acpi_cmd(arg1: op)` — shutdown (op=0), reboot (op=1); validate caller is uid 0.
- **services/go/acpi.go** (new file): RSDP discovery, RSDT/XSDT/FADT parsing (minimal: RTC port, S-states, shutdown method). Boot-critical service that IPC-sends discovered RTC address to init. On admin request (shell `shutdown` / `reboot` commands), calls sys_acpi_cmd.
- **services/go/runtime.go**: Register timesvc spec (already present; line 129–138) as dependency-free phaseCore, stop command = cmdStop ('Q'). Implement timesvc event loop: read sys_net_query for DHCP/DNS, call SNTP client code on a 1 Hz timer, update kernel wall clock via a privileged syscall (or shared memory write with atomics).
- **docs/abi/syscall_v3.2.md** (new section or update to syscall_v3.md): Document v3.2 window 48..63; add entries for 50, 51, 52.

**ABI additions** —
- Syscall 50: `sys_clock_gettime(clockid: u64 rdi, timespec_ptr: u64 rsi) -> i64 rax`
  - Returns 0 on success, -1 on fault (E_FAULT if timespec_ptr invalid).
  - Supported clockids: 0 = CLOCK_MONOTONIC, 1 = CLOCK_REALTIME.
  - Contract doc: docs/runtime/clock_v1.md (same marker contract as console_v1: `CLOCK: <clockid> ok` after each successful read).
- Syscall 51: `sys_timerfd_create(flags: u64 rdi, itimerspec_ptr: u64 rsi) -> i64 rax`
  - Returns FD on success (0–15 range), -1 on error (E_RANGE if no FD slots, E_FAULT).
  - Markers: `TIMER: <id> allocated`, `TIMER: <id> expired` (serial_write_hex).
- Syscall 52: `sys_acpi_cmd(op: u64 rdi) -> i64 rax`
  - op=0 (shutdown), op=1 (reboot); returns -1 (E_PERM if uid != 0, E_UNSUP if S-state unavailable).
  - Markers: `ACPI: shutdown initiated`, `ACPI: reboot initiated`.
- Window 48–49 remain as allocated in ABI (signals, netcfg); 50–52 are new; 53–63 reserved.

**Acceptance** —
- `make test-clock-v1`: boots, calls sys_clock_gettime(MONOTONIC) 3 times in a tight loop, asserts return values increase strictly, emits marker `CLOCK: monotonic ok`. Reads CMOS via the acpi service at boot and confirms wall clock advances, marker `CLOCK: wall ok`.
- `make test-timer-v1`: creates a timerfd with 100 ms timeout, yields/polls until expiry, asserts wakeup time is within 110–200 ms (accounting for PIT jitter), marker `TIMER: 0 expired`.
- `make test-acpi-v1`: acpi service starts, discovers FADT, logs marker `ACPI: fadt found`; if running on real hardware with battery, logs `ACPI: battery 0xNNNN` (object address). On VM, graceful fallback (marker `ACPI: no fadt, cmos only`).
- `make test-shutdown-v1`: shell command `shutdown` → calls sys_acpi_cmd(0) → marker `ACPI: shutdown initiated` → boot transcript ends cleanly (no qemu_exit debug log after the marker).
- Contract docs: docs/runtime/clock_v1.md, docs/runtime/timerfd_v1.md, docs/runtime/acpi_v1.md (each with status, source files, proof targets, marker contract table, and carry-forward notes).

**Gotchas** —
- **Single USER_PML4 (lib.rs ~1547)**: wall clock must be kernel-global, not per-task; the shared memory trick (acpi service writes RTC time to a known kernel address, kernel IPC-reads it) avoids locking complexity.
- **RTC CMOS decay**: CMOS batteries die; gracefully fall back to PIT-only monotonic if RTC reads fail or produce obviously stale values (e.g., year < 2020).
- **ACPI parsing in userspace**: the kernel must not parse DSDT bytecode (complex, bloats kernel). The service enumerates S-states by scanning FADT S5a/S5b (shutdown power values) and optional S4a/S4b (hibernate); real _OFF method execution is future work (carry-forward).
- **Marker splicing**: sys_time_now in a loop will emit many MONOTONIC reads; the test must tolerate that. The timerfd marker must be emitted by the kernel at the moment the timer expires (via serial_write_hex in the scheduler IRQ), not by the service's response to a syscall — if the test yields post-expiry, the marker has already fired.
- **Size budget**: timekeeping adds ~400 bytes (struct, globals) to the kernel; timerfd per-task overhead is negligible (one u64 head pointer + link node on heap, deferred allocation). ACPI parsing in the service may hit the 28 KiB gousr.bin cap if minimal (estimate 2–4 KiB for RSDP/RSDT/FADT parsing); if it overflows, land a second code page or defer battery/thermal parsing (carry-forward).
- **Console spinning (carry-forward)**: the console_v1 contract notes that blocking reads spin the kernel with IF=0. With wait queues in place, a future console_v2 can sleep the task; do not wire the timer queue to console reads in this section — that's a separate task.

**Effort / risk** — **M (medium)**: ~350 lines kernel code (timekeeping + syscall dispatch + PIT IRQ wakeup), ~400 lines for acpi.go (RSDP/FADT parsing), ~200 for timerfd FD plumbing. Main risks: (1) ACPI RSDP is sometimes in UEFI's memory map, not the E0000–FFFFF legacy zone — requires fallback or EFI handoff (handled by deferring to service discovery), (2) CMOS RTC may not be present on all VMs (graceful no-op, monotonic clock alone is sufficient for tests), (3) timer queue per-task allocation during syscall may fail under memory pressure (return E_RANGE, don't leak).


### Security: RNG, ASLR, sandboxing, multi-user, crypto, secure boot

**Status today / prerequisites** — Rugo has a ring 0/3 split with per-task capability bits (`R4Task.cap_flags` at lib.rs:2623, 2 bits: `R4_TASK_CAP_STORAGE|R4_TASK_CAP_NETWORK`) and per-FD rights (`HANDLE_RIGHT_READ|WRITE|POLL` in security.rs). A basic multi-user model exists: tasks carry a `uid` field (lib.rs:2642), root (uid=0) runs boot services, and spawned external apps run as uid 100; VFS enforces mode bits (owner/other rw/r) and owner-based unlink rules (vfs_v1.md "Users and permissions"). `sys_sec_profile_set` (syscall 27, lib.rs:1882) toggles between Default and Restricted profiles for the calling task but does no enforcement. **Prerequisite:** the production lane must be unified (today M3/R4 are feature-gated compile-time kernels); demanding dynamic address spaces and working multiuser requires that. Pending: kernel CSPRNG seeded from RDRAND/RDSEED, getrandom syscall + /dev/urandom, ASLR (randomized user code/stack/mmap bases), a pledge/unveil-style syscall allowlist per-task, in-repo AEAD crypto (extending the existing SHA-256/HMAC in services/go/hash.go for TLS), audit logging, and measured boot beyond serial markers.

**Design for Rugo** — Implement a cascade of four tight features in the `go_test` lane (unified kernel):

1. **Kernel CSPRNG + getrandom (syscall 50, v3.2 window)**: add a 64-byte entropy pool seeded once at boot from RDRAND (with fallback to tight timing entropy from PIT + entropy from first DHCP/TCP handshake packets). Expose `sys_getrandom(buf_ptr, len)` returning bytes read; also mount `/dev/urandom` (already a fixed VFS path in SimpleFS) as a read-only file that pulls from the pool. The pool must have sufficient mixing: a simple Xorshift64\* on each 8-byte word refresh, incremented by PIT ticks. Keep the pool in `kernel_rs/src/rng.rs` as a sealed static (no extern crate — stay `no_std`). Seed happens once in `mm_init()` after Limine; getrandom copies to user with bounds checking, returns -1 (E_FAULT) on bad pointers. Deliver via `make test-rng-v1`, asserting `serial_write_hex` of the first 16 bytes emitted by the pool match a canonical test vector.

2. **ASLR (address space layout randomization)**: randomize user code and stack bases per-task at spawn time, drawn from the RNG pool. Replace the fixed `USER_CODE_VA=0x40_0000` and `USER_STACK_TOP=0x80_0000` constants with per-task offsets. For simplicity, allocate per-task page tables: each spawned task gets its own `PML4` (allocated from the PMM heap, so this requires careful budgeting for max R4_MAX_TASKS table allocations). Randomize code base to a 1 MiB-aligned slot in [0x1_0000, 0x100_0000) and stack to [0x8_0000, 0x200_0000), ensuring no overlap. Update `r4_switch_to` (lib.rs:~2804) to reload CR3 on task switch (today it does not; the single shared `USER_PML4` is only set once). This is a breaking change to the per-process address space model documented in the memory contract and in syscall_v3.md ("a SINGLE shared USER_PML4"), so call out an explicit v4 window decision if it spills v3.2. A simpler v3.2 alternative: stay with the shared page table but randomize the virtual addresses within it; the gaps module analysis lists ASLR as missing but does not assume per-process page tables.

3. **Syscall allowlist (pledge/unveil) via `sys_sec_filter` (syscall 51, v3.2 window)**: extend the cap_flags model with a per-task bitmask of allowed syscalls. `sys_sec_filter(allow_mask: u64) -> 0` restricts the calling task to only the syscalls set in the mask; a subsequent call with a mask *wider* than the current one returns -1 (E_PERM, monotonic enforcement). At syscall dispatch, check `R4_TASKS[R4_CURRENT].sec_filter_mask & (1 << syscall_nr)` before routing; deny with -1 (E_NOSYS). Start tasks with all bits set; the shell can restrict children before exec. Proof: `make test-sec-filter-v1`, running a spawned task that calls `sys_sec_filter(0x3)` (allow only syscalls 0 and 1: debug_write and thread_spawn) and verify an attempt at syscall 3 (yield) returns -1.

4. **Multi-user auth via login service (in `services/go`)**: add an `/etc/passwd`-style VFS file (root-owned, mode 0o100: other-read only, binary records) holding uid/username/password-hash tuples. Add a login syscall (`sys_login(user_ptr, pass_ptr) -> uid`) that hashes the password (reuse SHA-256 from hash.go), compares against the file, and returns the uid on success or -1 on mismatch. Update the shell to prompt for login on boot; successful login spawns the shell with the user's uid. Record uid in audit logs (below). Proof: `make test-users-v1` already exists (vfs_v1.md) and checks that a uid 100 task is denied write on uid 0 files; extend it to exercise login.

5. **Audit logging**: add a `sys_audit_write(msg_ptr, len) -> len_written` syscall (52, v3.2) to log security events to an in-memory ring buffer (4 KiB), emitting to serial on every call. Each entry: `[timestamp (8B), uid (1B), syscall_nr (2B), result (1B), msg (variable)]`. Kernel preempts audit writes (no interrupts) and atomically bumps the head pointer. Proof: `make test-audit-v1`, verifying that a uid 100 task attempting `sys_sec_filter(-1)` emits a denied marker.

6. **Measured boot markers** (lightweight alternative to TPM): on every boot, compute a cumulative SHA-256 over [kernel image hash] + [boot cmdline] + [loaded packages]. Emit the final digest to serial at init completion with a marker `BOOT: measured <hex>`. Store in a sealed NVRAM-like region (or hardcoded check on reboot). Proof: `make test-secure-boot-v1`, verifying the marker is stable across reboots and changes if the kernel/packages differ.

**Concrete changes**:
- Add `kernel_rs/src/rng.rs`: `pub fn rng_init()`, `pub fn rng_read(buf: &mut [u8]) -> u64` (bytes read), static entropy pool (64 bytes), init from RDRAND+fallback. Call from `mm_init()` after Limine memmap.
- Edit `kernel_rs/src/lib.rs`:
  - Add `pub(crate) mod rng` at the top.
  - Expand `R4Task` with `sec_filter_mask: u64` field (initialized to `u64::MAX`).
  - Add constants for new syscall ids (50, 51, 52) in the go_test lane.
  - Per-task PML4 allocation in `sys_thread_spawn_r4` (index allocate/deallocate in a PMM heap or static pool).
  - Update `r4_switch_to` to reload CR3 from task-specific PML4.
  - At syscall dispatch: check sec_filter_mask before routing.
  - Implement `sys_getrandom_v1(buf, len)`, `sys_sec_filter_v1(mask)`, `sys_audit_write_v1(ptr, len)` syscalls.
- Edit `kernel_rs/src/vfs.rs`: add `/etc/passwd` node (KIND_FILE, uid 0, mode 0o100 — other-read only) with hardcoded root/testuser entries; implement password-hash lookup.
- Add `kernel_rs/src/audit.rs`: in-memory ring buffer, per-task write methods.
- Edit `services/go/shell_session.go`: add login prompt on startup, call `sys_login` before spawning shell.
- Edit `docs/abi/syscall_v3.md`: document syscalls 50 (getrandom), 51 (sec_filter), 52 (audit_write) in the 48..63 v3.2 window. If per-task page tables are chosen for ASLR, document the v4 window decision.

**ABI additions**:
- Syscall 50: `sys_getrandom(buf_ptr: u64, len: u64) -> u64` — read random bytes into user buffer; return bytes written or -1 (E_FAULT).
- Syscall 51: `sys_sec_filter(allow_mask: u64) -> i64` — restrict syscall access; return 0 or -1 (E_PERM if mask is broader than current).
- Syscall 52: `sys_audit_write(msg_ptr: u64, len: u64) -> u64` — write audit log; return bytes written or -1 (E_FAULT).
- Update `docs/abi/syscall_v3.md` to list these in the 48..63 window.
- New contract doc: `docs/runtime/rng_v1.md` (entropy source, pool mixing, seeding from RDRAND + timing entropy).
- New contract doc: `docs/runtime/aslr_v1.md` (per-task base randomization, overlap guarantees, address space layout).
- New contract doc: `docs/runtime/sec_filter_v1.md` (syscall allowlist semantics, monotonic enforcement).
- New contract doc: `docs/runtime/audit_v1.md` (ring buffer schema, uid/syscall/result logging, serial emission).

**Acceptance**:
- `make test-rng-v1`: boot with go_test, verify the RNG pool emits 16 zero-padded hex bytes matching a test vector (e.g., the first 16 bytes after seeding from a fixed RDRAND mock). Runtime code calls `rng_read` and emits `RNG: <hex16>` to serial; the test asserts the marker.
- `make test-aslr-v1`: spawn two tasks and verify their code bases differ (read from proc/task info or a new syscall returning task layout). Assert that no two spawned tasks in a single boot have the same code base.
- `make test-sec-filter-v1`: spawn a child, call `sys_sec_filter(0x3)` to allow only syscalls 0 & 1, attempt syscall 3 (yield), verify return is -1; emit `SEC: filter denied syscall 3` marker.
- `make test-auth-v1`: extend existing `test_users_v1.py` to verify that `sys_login("root", "password")` returns uid 0 and that an incorrect password returns -1.
- `make test-audit-v1`: attempt a denied syscall (e.g., `sys_sec_filter(-1)`) from uid 100, verify `AUDIT: [timestamp] uid=100 syscall=51 result=-1` marker is emitted.

**Gotchas**:
- **Per-task page tables budgeting**: allocating R4_MAX_TASKS PML4s from a 32 KiB kernel heap eats fast. Use a static pool or PMM allocation; if PMM, watch the boot-time frame budget (measured in tests/boot/*). At R4_MAX_TASKS=16, that is 64 KiB overhead, risking OOM on small test fixtures. Consider a v3.2 simpler model: no per-task PT, randomize only the offsets within the shared PT (guard-page padding between regions ensures no true randomization, but defeats naive linear probes).
- **RDRAND availability**: QEMU default CPU flags may lack RDRAND. Add fallback to PIT timing entropy; test harness must either enable the flag or accept the fallback path.
- **Audit buffer wraparound**: if the ring fills before serial drains, audit messages are lost. Emit every write immediately (no buffering) or size the buffer conservatively (4 KiB ≈ 40–50 entries at typical sizes).
- **Marker splicing**: "AUDIT:" prefix must not appear in any other kernel output or tests fail on false matches. Scan existing markers first; use "AUDIT_LOG:" if there is a collision.
- **IF=0 sections (feature gates)**: all new security syscalls land *inside* `#[cfg(all(feature = "go_test", ...))]` blocks to keep M3/R4 separate (per RUGO_CONVENTIONS, no new test-only feature lanes). Verify the dispatch tables include the feature guards.
- **`sys_login` pointer validation**: validate user_ptr and pass_ptr for legality before dereferencing. On invalid pointers, return -1 (E_FAULT).

**Effort / risk** — **Size: XL** (roughly 1500–2000 lines across new modules + syscall routing); **Risk: high**. Per-task page tables (needed for true ASLR) require refactoring the memory model (r4_switch_to, task spawning, PMM allocation) — a high-churn change. The simpler alternative (randomize within a shared PT) is lower-risk but weaker. RNG seeding from RDRAND is straightforward but its fallback (timing entropy) is fragile in test fixtures with fixed execution patterns. Audit logging is low-risk once serialization is right. The biggest risk: unifying M3/R4 is a prerequisite; attempting these features in isolation will collide with the current dual-lane architecture (e.g., syscall 50 dispatch duplicated in both M3 and R4 feature blocks).



# Part V — Userspace & operations


### Userspace, dynamic linking, and platform/ops

**Status today / prerequisites** — rlibc (`libc/rlibc.c` ~ libc.h) provides a bump-heap allocator, basic syscall wrappers (open/read/write/spawn/wait on v3 ABI), and printf over stdin/stdout pipes; crt0.asm (libc/crt0.asm ~17) hands off args, pipe fds, and jumps main() at entry. ELF loading is static: the kernel's elf_v1_validate_image() and elf_v1_build_auxv() (~177–302 lib.rs) check headers but do NOT relocate; payloads are pre-linked PE at 0x1400000, then pe_to_elf_v1.py (~1–80 tools/) re-wraps them as one-segment ET_EXEC images with no PLT/GOT. Pkgsvc already handles SHA-256 signatures and state machines (services/go/pkgsvc.go ~125–317); shell_session.go spawns apps by name (spawnRun, ~331–378) and consumes their exit codes. Gaps: no errno, no buffered FILE*, no lseek syscall, no env vars, no real malloc with free, no dynamic linker (PIE/ASLR/PLT/GOT), no TTY layer or job control (console reads spin in libc ~66), and no UEFI/real installer (Limine stub only).

**Design for Rugo** — Userspace lands in two phases. **Phase 1 (libc v2 + TTY/job control):** extend rlibc with errno (thread-local via TLS in user_alloc, v3 syscall 44 fork/clone codepaths will reserve TLS base), buffered FILE* (stack-allocated 1 KB buffers per stream), a sys_lseek syscall (v3.2 id 50, op 1 = seek, returns offset mod 2^32 in rax hi/lo), and environment variables (passed in auxvec in crt0). Implement TTY/pty via a userspace daemon that wraps pipes (sys_pty_ctl v3.2 id 51, op 1 = alloc pty, op 2 = set foreground). Add job control markers to shell_session (GOSH) via signal handlers (sys_signal_ctl v3 id 48, already exists). **Phase 2 (dynamic linker):** split the ELF loader into two paths: static (today, for coreutils in the package store) and dynamic (libdl wrapper that loads .so from /data, resolves GOT/PLT). Use a minimal loader ~2 KiB (rlibc stays under 28 KiB footprint) that applies R_X86_64_REL32, R_X86_64_GLOB_DAT, and R_X86_64_JUMP_SLOT from .dynamic sections; no lazy binding yet. Coreutils (echo, cat, ls, ps) ship as static binaries until Phase 2. **Phase 3 (ops):** add a sys_crashdump_write v3.2 id 52 that emits a minimal 256-byte header (tid, cs:rip, fault addr, errno) to the journal ringbuffer (already in mm.rs ~1547 USER_PML4 demand-paging triggers), plus a /proc-style dmesg client tool. Installer v1 rewrites tools/pe_to_elf_v1.py output to use the UEFI boot path; QEMU-only for now. This respects the single USER_PML4 (no per-task paging), avoids new crate deps, and layers incrementally behind go_test feature.

**Concrete changes**

- **rlibc v2 (errno + lseek + env vars + FILE*):** Extend libc/rlibc.c (lines 77–98 heap, 167–274 stdio) to add per-fd static buffers (M8_FD_TABLE already indexed by fd at kernel_rs/lib.rs ~641–647); add sys_lseek (SYS_LSEEK = 50, SYS_PTY_CTL = 51) to libc/include/rugo/libc.h; implement lseek/read/write looping to interleave small chunks and recover from EAGAIN. Crt0.asm ~20 now passes auxvec pointer in r10 (ABI additive, kernel fills via elf_v1_build_auxv after unpacking /proc/self/environ-like strings). Errno is thread-local in kernel_rs/src/mm.rs demand-map: call sys_tls_get (~call r9 with TCB vaddr) to read/write errno_val at offset 0. Add FILE struct (FILE {fd, buf[1024], off, dirty}).

- **TTY/job control layer:** New file services/go/tty_daemon.go (300 lines) that calls sys_pty_ctl to allocate N pty pairs, listens on /dev/tty* pipes, echoes input to all tasks in the foreground group, and marshals signals via sys_signal_ctl. Shell_session.go ~31–57 now reads from a pty fd (negotiated at startup via sys_pty_ctl op 1 = create) instead of spinning in console_read_byte(). Kernel syscall stub in kernel_rs/src/syscall.rs routes 51 to empty handler (return 0) until daemon lands.

- **Dynamic linker (Phase 2):** New libc/libdl.c (400 lines) with dlopen(name, flags) → loads ELF from /data/<name>.so, validates e_type == ET_DYN (PIE), maps segments via sys_vm_map, relocates .rel.dyn/.rel.plt from .dynamic, returns handle opaque to dlsym. Kernel elf_v1_validate_image() (~177 lib.rs) now rejects ET_EXEC if a dlopen() call is live (tracked in R4_TASKS[CURRENT].loader_state). No lazy PLT binding; all relocs apply immediately. Package store toolchain compiles libc as libm.so (shared).

- **Syscall ABI additions:** SYS_LSEEK (50), SYS_PTY_CTL (51), SYS_CRASHDUMP_WRITE (52) in v3.2 window; each with 1 op (kernel stubs return -E_UNSUP today). Add docs/abi/syscall_v3.2_addenda.md.

- **Size impact check:** rlibc v2 adds ~3 KiB (FILE + TLS stubs); libdl adds ~2 KiB; stays within 28 KiB userspace cap. Kernel stubs ~200 bytes.

**ABI additions** — Document in docs/abi/syscall_v3.2_addenda.md:
- **SYS_LSEEK (50):** arg1=fd, arg2=offset_lo, arg3=offset_hi, arg4=whence (0=SEEK_SET, 1=SEEK_CUR, 2=SEEK_END); return new_offset_lo (hi bits in rdx on 64-bit, but truncate for now). Error class E_INVAL if fd bad or whence > 2; E_RANGE if seek past 2^32.
- **SYS_PTY_CTL (51):** arg1=op (1=alloc_pair, 2=set_foreground_tid, 3=recv_signal), arg2=tid_hint; return pair_id or -1. Op 1 allocates master/slave pty fds, op 2 sets foreground group for SIGWINCH/SIGPIPE, op 3 polls for pending signal, blocks if none.
- **SYS_CRASHDUMP_WRITE (52):** arg1=header_ptr, arg2=header_len (must be 256); writes to kernel journal ringbuffer, returns 0 or E_FAULT.

**Acceptance** — `make test-libc-v2` boots via qemu_go_c4_runtime with a test app that exercises errno (open nonexistent, check errno==ENOENT), lseek (open file, seek(10), read must return from offset 10), FILE buffering (write 10 small chunks, see single flush marker), and env vars (check $PATH set by crt0). Serial markers: `LIBC_V2: errno ok\n`, `LIBC_V2: lseek ok\n`, `LIBC_V2: file_buffering ok\n`, `LIBC_V2: env ok\n`. Contract doc docs/runtime/libc_v2.md describes the three gaps closed (errno, stdio buffering, lseek) and sizes. `make test-tty-v1` verifies TTY daemon spawns, allocates pty pair, and shell reads from pty master (not console) with serial marker `TTY_V1: job_control ok\n`. `make test-libdl-v1` (Phase 2, deferred) loads a tiny .so from /data that exports a symbol, dlopen succeeds, dlsym resolves it, relocs applied, call site lands in .so code (verified by marker in .so text + signal handler catch).

**Gotchas**

- **TLS and demand paging:** errno via TLS requires the kernel to allocate a user TCB page (vaddr 0x01700000) on first access (sys_tls_get stubs into a demand-map call). A task's TCB is single per lifetime (no migrate), so the R4_TASKS[tid] record must persist it across yields.
- **pty multiplexing and preemption:** The TTY daemon runs with IF=0 in go_test lane (no preemption) or via I/O wait on pty read; a blocked shell task yields, releasing the daemon to flush echoes to all pty slaves. Marker placement is critical: do NOT anchor on a shell prompt line (those are echoed interactively).
- **Dynamic linker and single-occupancy exec window:** The exec window [0x01400000, 0x01800000) is single-task, single-occupancy. If a second task spawns while one is loading a .so, the loader stalls; this is acceptable for v1 (no parallel spawns). If a .so is loaded at 0x01500000, the static ELF at 0x01400000 and remaining heap at 0x01600000 compete; the loader must refuse if segments would overlap (check in dlopen, return ENOEXEC).
- **ABI freeze and v3.2 carve-out:** Slots 50–52 are reserved; if more than 14 new syscalls are needed (window 48–63 has 49 & 48 used), declare a v4 decision explicitly.

**Effort / risk** — **rlibc v2:** M effort (errno + FILE ~300 LOC, lseek syscall + handler ~150 LOC, env var auxvec plumbing ~100 LOC). Risk: off-by-one in FILE buffering if flush marker timing splices with concurrent output; mitigate by testing with -smp=2 in pytest fixture (add it to qemu_go_c4_runtime).  **TTY/job control:** M effort (tty_daemon ~300 LOC, syscall stub ~50). Risk: daemon deadlock if a foreground task blocks on pty read while another tries to write; mitigate by timeout in sys_pty_ctl op 3 (return E_TIMEOUT after 5s). **libdl:** L effort (minimal loader ~400 LOC, no IFUNC/weak/versioning). Risk: GOT/PLT relocation off-by-one if section layout varies; mitigate by hard-coding .dynamic offsets in the build (add section layout checks to elf_v1_validate_image for .so). **Installer:** L effort (UEFI shim ~200 LOC, partition detection ~100). Risk: disk I/O hangs in real hardware before QEMU-only gate lifts; accept as deferred to Phase 3.
