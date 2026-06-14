# mmap / brk / munmap — contract v1

Status: boot-verified via `make test-mmap-v1`
Source: `kernel_rs/src/mm.rs` (`vm_map_current`, `vm_unmap_current`,
`PTE_COW`), `kernel_rs/src/lib.rs` (`sys_vm_ctl`, `R4Task.heap_brk`),
`apps/coreutils/vmprobe.asm`.
Proof: `tests/runtime/test_mmap_v1.py`.

Full-OS implementation guide Part I.4. Builds on the per-process address
space keystone ([`per_task_as_v1.md`](per_task_as_v1.md)); each mapping is
private to the calling task's address space.

## ABI

`sys_vm_ctl` — ABI v3.2 id **50**, op-multiplexed (per the master id table
in the implementation guide §0.2; the section's illustrative 50/51/52 are
superseded by a single multiplexed id):

| op | call | args | returns |
|----|------|------|---------|
| 1 | mmap | `rsi`=va, `rdx`=sz, `r10`=prot | va, or -1 |
| 2 | munmap | `rsi`=va, `rdx`=sz | 0, or -1 |
| 3 | brk | `rsi`=new (0 = query) | old break, or -1 |

prot bits: 1 = READ, 2 = WRITE, 4 = EXEC. A mapping is always present+user;
WRITE adds the writable bit, and the page is NX unless EXEC is set.

## Address layout (per task, in the demand window, below the exec window)

| region | range | use |
|--------|-------|-----|
| brk | `[0x0100_0000, 0x0120_0000)` | contiguous program break |
| mmap | `[0x0120_0000, 0x0140_0000)` | anonymous mappings |

Both lie below the exec window `[0x0140_0000, 0x0180_0000)` and the demand
stacks (`0x0190_0000+`), so they never collide with loaded code or stacks.
`va` and `sz` must be page-aligned; ranges are bounds-checked to their
region. The break starts at `0x0100_0000` (lazily initialized on first use)
and is capped at `0x0120_0000`.

## Semantics

- **mmap** maps each page via `vm_map_current` (idempotent if already
  present); on partial failure it rolls back the pages it mapped and
  returns -1. Frames come from the PMM and are zeroed.
- **munmap** clears each PTE and releases the frame through
  `cow_release_leaf`, so a page shared via fork is only freed by its last
  owner. `DEMAND_MAPPED` (the physical demand-frame count) is decremented
  on real frees.
- **brk** grows by mapping RW pages and shrinks by unmapping; `heap_brk` is
  per-task and reset on slot reuse.

## prot enforcement vs. copy-on-write

A `PTE_COW` software bit (PTE bit 9) distinguishes a forked copy-on-write
page from a genuinely read-only `mmap(PROT_READ)`. The page-fault handler's
`cow_break` acts **only** on pages carrying `PTE_COW`; a write to a
read-only mmap has no such bit, so it faults through to containment
(`USERPF`) and the task is killed. This makes `PROT_READ` real.

## Markers

| Marker | Emitted when |
|--------|--------------|
| `MM: mmap va=0x<va> sz=0x<sz>` | a successful mmap |
| `MM: munmap va=0x<va>` | a munmap |
| `MM: brk 0x<old> -> 0x<new>` | a break change |
| `VMPROBE: ok` / `VMPROBE: ro mapped` | probe progress |

## v1 boundary / carry-forward

- No VMA list, file-backed mappings, `MAP_FIXED` negotiation, huge pages,
  `mprotect`, or swap — all carry-forward.
- `DEMAND_MAX_FRAMES` quota stays global (per-task quota deferred).
- mmap region is a fixed 2 MiB window; a real allocator placing mappings
  anywhere in the user half is future work.

## Acceptance

`make test-mmap-v1`: `probe vmprobe` exercises brk grow + brk memory + mmap
RW + munmap and prints `VMPROBE: ok`; `probe vmprobe ro` maps a read-only
page, reads it, and the write faults (`USERPF` at the page) with no
`VMPROBE: ro WROTE` marker — proving prot enforcement.
