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

- Window: virtual `[0x0100_0000, 0x0200_0000)` (16 MiB..32 MiB), above the
  fixed user code (4 MiB) and stack (8 MiB) regions, with 8 MiB of
  deliberately unmapped guard space between stack and window.
- A **user-mode** page fault inside the window allocates any missing
  page-table frame plus the data frame from the PMM, installs a
  present+writable+user mapping into the faulting address space (current
  CR3), invalidates the TLB entry, and retries the instruction.
- Quota: 2048 frames (8 MiB) for now. Faults beyond the quota, outside the
  window, in a stack guard zone, or on already-present pages fall through
  to the existing user-fault containment path (task is killed).
- Syscall pointer validation **pre-maps** window pages exactly as the fault
  path would: freshly allocated, never-touched memory is a valid syscall
  buffer. Other kernel-mode touches of unmapped pages stay fatal.
- Layout within the window:
  - `[0x0100_0000, 0x0101_0000)` boot-time demand probe;
  - TinyGo userspace bump heap from `0x0110_0000`
    (`services/go/start.asm`, atomic `lock xadd` bump, pages arrive zeroed
    from the fault path);
  - dynamic task stacks from `0x0190_0000`: 128 KiB strides per spawn slot
    ≥ 5, growing down, with the bottom 16 KiB of every stride a guard zone
    the demand mapper refuses — a runaway stack faults fatally instead of
    silently corrupting its neighbour.

## W^X on dynamic user memory

- `EFER.NXE` is enabled at boot in every lane (marker `MM: nx on`).
- Demand-mapped pages carry PTE bit 63 (NX) except inside the exec app
  window `[0x0140_0000, 0x0180_0000)`, where ELF segments load through
  the same path. The TinyGo heap and all dynamic task stacks are
  therefore non-executable: executing from them faults with error bit 4
  (instruction fetch) and the task is killed.
- Proof: `make test-wx-v1` - the `nxprobe` app calls into its own
  stack and must die at `USERPF ... err=0x...15` while the system
  shuts down cleanly.
- Carry-forward: read-only user code pages, NX for the static service
  stack region, ASLR.

## Marker contract

| Marker | Meaning |
|---|---|
| `MM: pmm ok frames=0x<16-digit hex>` | PMM populated from the Limine memmap |
| `MM: pmm none` | no memmap/HHDM response; static pools only |
| `MM: heap ok size=0x<16-digit hex>` | kernel heap window reserved |
| `MM: heap none` | contiguous heap window unavailable |
| `MM: heap selftest ok` / `... err` | boot-time Box/Vec alloc-free-reuse check |
| `MM: nx on` | EFER.NXE enabled; NX honored on user data pages |
| `MM: demand map va=0x<page>` | one probe-range page mapped on first touch (heap pages above `0x0110_0000` map silently so workload-dependent counts never perturb marker assertions) |
| `GOINIT: mem demand ok` / `... err` | Go init touched 16 window pages successfully |

All markers are asserted in order by `tests/mm/test_mm_foundation_v1.py`
against both `out/os.iso` (kernel-only lane) and `out/os-go.iso` (default Go
lane), including an exact count of 16 `MM: demand map` lines and the absence
of `USERPF:` in a clean default boot.
