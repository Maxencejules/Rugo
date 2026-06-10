# Dynamic Memory Foundation Contract v1

Status: live runtime (boot-verified)
Source: `kernel_rs/src/mm.rs` (compiled in every lane, no feature gates)
Proof: `make test-mm-foundation-v1`, `tests/mm/test_mm_foundation_v1.py`

This contract covers gap-analysis build-list item 1
(`docs/analysis/full-os-gap-analysis.md` §3): physical frame allocator,
kernel heap, and demand paging.

## Physical frame allocator (PMM)

- Fed by a Limine memmap request; only `USABLE` (type 0) entries are pooled.
- Tracks the first 4 GiB of physical memory, 1 bit per 4 KiB frame
  (128 KiB static bitmap in kernel BSS). Frame 0 is never pooled.
- Frames are zeroed at allocation time through the Limine HHDM.
- API: `alloc_frame()`, `free_frame(phys)`, `alloc_frames_contig(count)`.

## Kernel heap

- One contiguous 4 MiB PMM window, addressed through the HHDM.
- First-fit free list, 32-byte minimum block, address-sorted coalescing on
  free; registered as the Rust `#[global_allocator]`, so `alloc::boxed::Box`
  and `alloc::vec::Vec` work kernel-wide.
- OOM behavior: allocation failure returns null; Rust's default alloc error
  handler panics, which prints the panic marker and exits QEMU - boot tests
  catch it.

## Demand-paged user heap window

- Window: virtual `[0x0100_0000, 0x0180_0000)` (16 MiB..24 MiB), above the
  fixed user code (4 MiB) and stack (8 MiB) regions, with 8 MiB of
  deliberately unmapped guard space between stack and window.
- A **user-mode** page fault inside the window allocates any missing
  page-table frame plus the data frame from the PMM, installs a
  present+writable+user mapping into the faulting address space (current
  CR3), invalidates the TLB entry, and retries the instruction.
- Quota: 1024 frames (4 MiB) for now. Faults beyond the quota, outside the
  window, or on already-present pages fall through to the existing
  user-fault containment path (task is killed).
- Kernel-mode touches of unmapped window pages stay fatal by design: user
  space must touch a page before passing it into a syscall.

## Marker contract

| Marker | Meaning |
|---|---|
| `MM: pmm ok frames=0x<16-digit hex>` | PMM populated from the Limine memmap |
| `MM: pmm none` | no memmap/HHDM response; static pools only |
| `MM: heap ok size=0x<16-digit hex>` | kernel heap window reserved |
| `MM: heap none` | contiguous heap window unavailable |
| `MM: heap selftest ok` / `... err` | boot-time Box/Vec alloc-free-reuse check |
| `MM: demand map va=0x<page>` | one window page mapped on first touch |
| `GOINIT: mem demand ok` / `... err` | Go init touched 16 window pages successfully |

All markers are asserted in order by `tests/mm/test_mm_foundation_v1.py`
against both `out/os.iso` (kernel-only lane) and `out/os-go.iso` (default Go
lane), including an exact count of 16 `MM: demand map` lines and the absence
of `USERPF:` in a clean default boot.
