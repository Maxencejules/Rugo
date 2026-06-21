# Swap / page eviction — contract v1

Status: boot-verified via `make test-swap-v1`
Source: `kernel_rs/src/mm.rs` (`swap_out_current`, `try_swap_in`, `leaf_pte_ptr`,
`swap_selftest`, `SWAP_USED`, `PTE_SWAPPED`), `kernel_rs/src/lib.rs`
(`blk_write_page`/`blk_read_page`), swap-in wired in `kernel_rs/src/trap.rs`.
Proof: `tests/runtime/test_swap_v1.py`.

Full-OS guide Part I.4 (memory), swap: evict a user page to disk to reclaim its
physical frame, and fault it back in on next access.

## Behaviour

A fixed swap area (16 slots × 4 KiB) on a scratch disk region (`SWAP_BASE_LBA`):

- **`swap_out_current(va)`**: walk the current address space to `va`'s leaf PTE;
  if present, write its frame to a free swap slot (`blk_write_page`, 8 sectors),
  mark the PTE **swapped** (present=0, `PTE_SWAPPED`, slot index in the high
  bits), free the physical frame, and `invlpg`.
- **`try_swap_in(va)`** (called from the page-fault handler before user-fault
  containment): if `va`'s PTE is a swapped entry, allocate a fresh frame, read the
  page back from its slot (`blk_read_page`), remap it present, free the slot, and
  retry. A normal fault (not swapped) falls through unchanged.

Huge pages (PD `PTE_PS`) are not swapped (`leaf_pte_ptr` returns None for them).

## Acceptance

`make test-swap-v1`: the boot self-test maps a user page, writes a 4 KiB pattern,
evicts it (confirming the PTE became swapped/not-present and the frame freed),
swaps it back in, and reads it byte-exact through its VA — proving the page
survived the round-trip frame → disk → fresh frame: `SWAP: roundtrip ok`, with no
`SWAP: fail` / `SWAP: skip`. The fork/CoW/mmap/demand-paging tests stay green
(the swap-in check is additive on the fault path, after CoW, before containment).

## v1 boundary / carry-forward

- A fixed-size swap file (16 pages) with no eviction **policy** — the kernel does
  not yet *choose* victims under memory pressure (an LRU/clock page-replacement
  daemon is carry-forward). `swap_out_current` is the eviction primitive; the
  self-test drives it explicitly, and the live fault path restores swapped pages.
- One swap slot per page (no compression, no shared-page accounting); the swap
  area is a scratch LBA range, not a managed swap partition.
