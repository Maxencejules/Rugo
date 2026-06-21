# clone threads + futex — contract v1

Status: boot-verified via `make test-futex-v1`
Source: `kernel_rs/src/lib.rs` (`sys_proc_ctl` clone op 2, `sys_futex`,
`R4Task.futex_uaddr`, AS-release sibling scan in `r4_exit_and_switch`),
`apps/coreutils/futexprobe.asm`.
Proof: `tests/runtime/test_futex_v1.py`.

Full-OS implementation guide Part I.3 (concurrency primitive) and the
completion of clone from Part I.2.

## clone (sys_proc_ctl op 2)

`rdi` = 2, `rsi` = entry. Spawns a new thread sharing the caller's address
space (same `pml4_phys`), inheriting its caps and sandbox mask, with its own
demand-paged stack. Returns the new tid. Unlike `sys_thread_spawn_r4` (id 1,
gated on `can_spawn`), clone is available to **any** task — a thread adds no
privilege — so spawned apps get real threads.

Because cloned threads share one private address space, that space is
reclaimed only when the **last** active thread on it exits: `r4_exit_and_switch`
scans for any other non-Dead/Exited task on the same `pml4_phys` before
freeing (so a thread exit never frees an address space a sibling still uses).

## futex (sys_futex, ABI v3.2 id 52)

| op | call | args |
|----|------|------|
| 1 | wait | `rsi`=uaddr, `rdx`=val |
| 2 | wake | `rsi`=uaddr, `rdx`=n (0 = all) |

- **wait** reads the u32 at `uaddr`; if it no longer equals `val`, returns 1
  without blocking (the standard race-free check). Otherwise the task records
  `futex_uaddr`, goes Blocked, and yields; on a matching wake it resumes and
  the syscall returns 0.
- **wake** makes Ready up to `n` tasks Blocked on the same `uaddr` **in the
  caller's address space** (matched by `(pml4_phys, uaddr)`), and returns the
  count.

The kernel is single-CPU with interrupt-gated syscalls, so the
check-then-block in wait is race-free against a concurrent wake.

## Markers

`FUTEX: wait tid=0x<tid>`, `FUTEX: wake n=0x<count>`.

## v1 boundary / carry-forward

- Matching is by `(address space, virtual address)` — adequate for threads
  sharing one space; cross-address-space futexes on shared memory (by
  physical frame) are carry-forward.
- No timeouts, requeue, or priority inheritance.
- Wake scans the task table (O(tasks)); a hashed wait-queue is the SMP-era
  optimization.

## Acceptance

`make test-futex-v1`: `probe futexprobe` clones a thread, the parent blocks
in `futex_wait` (`FUTEX: wait`), the child writes the shared word and wakes
it (`FUTEX: wake n=1`), and the parent confirms the shared word changed
(`FUTEXPROBE: woken ok`) — proving shared-memory clone + futex hand-off, with
no deadlock.
