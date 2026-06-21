# fork / clone + copy-on-write — contract v1

Status: boot-verified via `make test-fork-v1`
Source: `kernel_rs/src/mm.rs` (`COW_REFCOUNT`, `cow_incr`, `cow_release_leaf`,
`address_space_fork`, `cow_break`), `kernel_rs/src/lib.rs` (`sys_proc_ctl`,
`sys_fork_v1`), `kernel_rs/src/trap.rs` (CoW write-fault path),
`apps/coreutils/forkprobe.asm`.
Proof: `tests/runtime/test_fork_v1.py`.

Full-OS implementation guide Part I.2. Builds directly on the per-process
address space keystone ([`per_task_as_v1.md`](per_task_as_v1.md)).

## ABI

`sys_proc_ctl` — ABI v3.2 id **51**, op-multiplexed (per the master id
table in the implementation guide §0.2; the section's illustrative 50/51
are superseded):

- **op 1 = fork.** No args. Duplicates the caller. Returns the child tid to
  the parent (`rax`) and `0` to the child. Only a task that already owns a
  private address space may fork (the boot task on the shared table cannot,
  in v1) — otherwise -1.
- **op 2 = clone.** `rdi` = op selector (2); `rsi` (= a2) = entry point. Spawns a new thread sharing the
  caller's address space (same `pml4_phys`), with its own demand-paged
  stack. Returns the new tid, or -1. (Thin wrapper over the existing
  thread-spawn path, which the keystone already made address-space aware.)

The legacy stubs at ids 43/44 (`sys_fork_deferred`/`sys_clone_deferred`,
always -1) remain for ABI continuity; new code uses id 51.

## Copy-on-write mechanics

`address_space_fork(parent_pml4)`:

1. Create a fresh child PML4/PDPT/PD with the kernel half cloned (as for any
   private space).
2. Walk the parent's user subtree (PML4[0]→PDPT[0]→PD→PTs). For each present
   PT, allocate a **copied** child PT; for each present leaf, clear the
   writable bit in **both** parent and child PTEs and bump the frame's
   refcount. Page-table frames are private per space; leaf data frames are
   shared read-only.
3. The parent's TLB is flushed (CR3 reload) after the marks so it, too,
   faults on the next write.

`COW_REFCOUNT[frame]` counts owners **beyond the first**: `0` = sole owner
(the common, untracked case), `N` = `N+1` owners. `DEMAND_MAPPED` continues
to count *physical* demand frames, so fork does not change it (no new
frames); a CoW break that allocates a copy increments it; freeing the last
owner decrements it.

`cow_break(va)` (called from the page-fault handler on a present write
fault, before fault containment):

- **Sole owner** (`refcount == 0`): just restore the writable bit — no copy.
- **Shared** (`refcount > 0`): allocate a fresh frame, copy the page,
  repoint the writer's PTE (writable), drop one refcount on the shared
  frame. Remaining owners keep sharing; when only one is left it becomes a
  sole owner and its eventual write is a no-copy re-mark.

`address_space_release` releases leaves through `cow_release_leaf`, so a
forked frame is freed only by its last owner; page-table frames are always
freed. Only genuinely-writable source pages are marked CoW at fork; a
read-only page (`mmap(PROT_READ)`, mprotect-RO) is shared as-is so a later
write still faults to containment rather than being silently promoted.

## Kernel writes into a forked address space

After fork the parent's (and child's) writable pages are read-only+CoW, so
the user-access path is CoW-aware: `check_page_user_perms` accepts a CoW page
for WRITE, and `copyout_user` breaks CoW across the destination (private
writable frame) before the store — so `read`/`getrandom`/`waitpid(&status)`
into a not-yet-written post-fork buffer work correctly. A child's exit
status is delivered into the **parent's** address space explicitly
(`mm::as_copyout` against the parent's PML4), since the waker runs while the
exiting child's / shared table is current, not the parent's.

## Markers

| Marker | Emitted when |
|--------|--------------|
| `FORK: child tid=0x<tid> as=0x<pml4>` | a fork child is created |
| `FORKPROBE: child ok wrote private` | child's CoW write was private |
| `FORKPROBE: parent ok cow-isolated` | parent's copy survived the child |
| `FORKPROBE: * FAIL ...` | a CoW violation (must never appear) |

CoW breaks are silent (no `USERPF:` marker): `cow_break` resolves the fault
before the containment path.

## v1 boundary / carry-forward

- No `exec`-replace or `setpriority` ops yet (reserved under this id).
- A fork child that exits without being reaped stays a zombie until slot
  reuse; full parent/child reaping semantics for fork (vs. spawn+wait) are
  carry-forward.
- `u8` refcounts cap at 256 owners — far above `R4_MAX_TASKS` (32).
- CoW copy runs with interrupts disabled (page-fault context); fine for
  4 KiB pages.

## Acceptance

`make test-fork-v1`: `forkprobe` writes a sentinel, forks, the child writes
a different value to the same global and confirms its private copy, and the
parent confirms its own copy is unchanged. The test asserts the `FORK`
marker, both success markers, no `FAIL`, no `USERPF`, exactly one fork, and
an `ASRELEASE` (the child's space reclaimed on exit).
