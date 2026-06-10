# Kernel Dynamic Memory Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the Rugo kernel a physical frame allocator, a kernel heap
(`GlobalAlloc`), and demand paging for a user heap window — gap-analysis §2.1,
build-list item 1.

**Architecture:** A new `kernel_rs/src/mm.rs` module, compiled
unconditionally in every lane (philosophy rule 5). PMM = bitmap allocator fed
by a new Limine memmap request. Heap = first-fit free-list allocator over a
4 MiB PMM-backed window reached through the HHDM, registered as
`#[global_allocator]`. Demand paging = a user-VA window (16 MiB–24 MiB) whose
pages are allocated and mapped on first touch by the page-fault handler
instead of killing the task. Verified by QEMU serial markers in both the
kernel-only lane (`out/os.iso`) and the default Go lane (`out/os-go.iso`).

**Tech Stack:** Rust no_std + `extern crate alloc` (no external crates),
Limine boot protocol, TinyGo userspace probe, pytest QEMU marker tests.

**Build/test commands on this host (Windows):**
- make: `& C:\mingw64\mingw64\bin\mingw32-make.exe <target>`
- kernel-only image: `mingw32-make image` → `out/os.iso`
- default Go image: `mingw32-make image-go` → `out/os-go.iso`
- tests: `python -m pytest tests/mm/ -v`

---

### Task 1: Limine memmap request + physical frame allocator (PMM)

**Files:**
- Create: `kernel_rs/src/mm.rs`
- Modify: `kernel_rs/src/lib.rs:42-49` (module list: add `pub(crate) mod mm;`)
- Modify: `kernel_rs/src/lib.rs:4884-4892` (`kmain`: call `mm::pmm_init()` after `check_paging()`)
- Test: `tests/mm/test_mm_foundation_v1.py`

- [ ] **Step 1: Write the failing tests**

```python
# tests/mm/test_mm_foundation_v1.py
# Phase 1 acceptance: boot-verified dynamic memory foundation (PMM + heap +
# demand paging). Live runtime evidence per SOURCE_MAP.md - serial markers only.


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = 0
    for marker in markers:
        found = serial.find(marker, pos)
        assert found != -1, f"marker not found in order: {marker}"
        pos = found + len(marker)


def test_pmm_boot_marker_kernel_lane(qemu_serial):
    out = qemu_serial.stdout
    _find_in_order(out, [
        "RUGO: boot ok",
        "MM: paging=on",
        "MM: pmm ok frames=0x",
        "RUGO: halt ok",
    ])
    assert "MM: pmm none" not in out


def test_pmm_boot_marker_go_lane(qemu_serial_go):
    out = qemu_serial_go.stdout
    _find_in_order(out, [
        "RUGO: boot ok",
        "MM: pmm ok frames=0x",
        "GOINIT: start",
        "GOINIT: ready",
        "RUGO: halt ok",
    ])
    assert "MM: pmm none" not in out
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `python -m pytest tests/mm/test_mm_foundation_v1.py -v`
Expected: both FAIL with `marker not found in order: MM: pmm ok frames=0x`
(images boot but never print the marker).

- [ ] **Step 3: Create `kernel_rs/src/mm.rs` with the memmap request and PMM**

```rust
// Dynamic memory foundation: physical frame allocator (PMM), kernel heap,
// and demand paging. Compiled in every lane - no feature gates.
//
// PMM: 1 bit per 4 KiB frame over the first 4 GiB of physical memory,
// populated from the Limine memmap (usable entries only). Frames are
// reached through the Limine HHDM for zeroing.

use crate::{serial_write, serial_write_hex};

pub const FRAME_SIZE: u64 = 4096;
const MAX_PHYS: u64 = 4u64 << 30;
const MAX_FRAMES: usize = (MAX_PHYS / FRAME_SIZE) as usize;
const BITMAP_WORDS: usize = MAX_FRAMES / 64;

const LIMINE_MEMMAP_USABLE: u64 = 0;

#[repr(C)]
struct LimineMemmapEntry {
    base: u64,
    length: u64,
    typ: u64,
}

#[repr(C)]
struct LimineMemmapResponse {
    revision: u64,
    entry_count: u64,
    entries: *const *const LimineMemmapEntry,
}

#[repr(C)]
struct LimineMemmapRequest {
    id: [u64; 4],
    revision: u64,
    response: *const LimineMemmapResponse,
}

unsafe impl Sync for LimineMemmapRequest {}

#[used]
#[link_section = ".limine_requests"]
static mut MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest {
    id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b,
         0x67cf3d9d378a806f, 0xe304acdfc50c3c62],
    revision: 0,
    response: core::ptr::null(),
};

struct Pmm {
    bitmap: [u64; BITMAP_WORDS],
    free_frames: u64,
    next_word: usize,
    hhdm: u64,
    ready: bool,
}

static mut PMM: Pmm = Pmm {
    bitmap: [0; BITMAP_WORDS],
    free_frames: 0,
    next_word: 0,
    hhdm: 0,
    ready: false,
};

pub fn hhdm_offset() -> u64 {
    unsafe { PMM.hhdm }
}

pub fn phys_to_virt(phys: u64) -> u64 {
    unsafe { phys + PMM.hhdm }
}

/// Populate the PMM from the Limine memmap. Prints `MM: pmm ok frames=0x<n>`
/// on success or `MM: pmm none` when the bootloader gave no memmap (static
/// pools still work in that case).
pub fn pmm_init() {
    unsafe {
        let hhdm_resp = core::ptr::read_volatile(
            core::ptr::addr_of!(crate::HHDM_REQUEST.response));
        let memmap_resp = core::ptr::read_volatile(
            core::ptr::addr_of!(MEMMAP_REQUEST.response));
        if hhdm_resp.is_null() || memmap_resp.is_null() {
            serial_write(b"MM: pmm none\n");
            return;
        }
        PMM.hhdm = (*hhdm_resp).offset;

        let count = (*memmap_resp).entry_count;
        let entries = (*memmap_resp).entries;
        let mut e = 0u64;
        while e < count {
            let entry = *entries.add(e as usize);
            if (*entry).typ == LIMINE_MEMMAP_USABLE {
                let mut frame = (*entry).base / FRAME_SIZE;
                let end = ((*entry).base + (*entry).length) / FRAME_SIZE;
                while frame < end {
                    if frame > 0 && frame < MAX_FRAMES as u64 {
                        PMM.bitmap[(frame / 64) as usize] |= 1u64 << (frame % 64);
                        PMM.free_frames += 1;
                    }
                    frame += 1;
                }
            }
            e += 1;
        }
        PMM.ready = PMM.free_frames > 0;
        if PMM.ready {
            serial_write(b"MM: pmm ok frames=0x");
            serial_write_hex(PMM.free_frames);
            serial_write(b"\n");
        } else {
            serial_write(b"MM: pmm none\n");
        }
    }
}

/// Allocate one zeroed 4 KiB frame. Returns the physical address or None.
pub fn alloc_frame() -> Option<u64> {
    unsafe {
        if !PMM.ready {
            return None;
        }
        let mut scanned = 0usize;
        let mut w = PMM.next_word;
        while scanned < BITMAP_WORDS {
            if PMM.bitmap[w] != 0 {
                let bit = PMM.bitmap[w].trailing_zeros() as u64;
                PMM.bitmap[w] &= !(1u64 << bit);
                PMM.free_frames -= 1;
                PMM.next_word = w;
                let phys = (w as u64 * 64 + bit) * FRAME_SIZE;
                core::ptr::write_bytes(
                    phys_to_virt(phys) as *mut u8, 0, FRAME_SIZE as usize);
                return Some(phys);
            }
            w = (w + 1) % BITMAP_WORDS;
            scanned += 1;
        }
        None
    }
}

/// Return a frame to the pool. `phys` must come from `alloc_frame`.
pub fn free_frame(phys: u64) {
    unsafe {
        let frame = phys / FRAME_SIZE;
        if frame == 0 || frame >= MAX_FRAMES as u64 {
            return;
        }
        let w = (frame / 64) as usize;
        let bit = frame % 64;
        if PMM.bitmap[w] & (1u64 << bit) == 0 {
            PMM.bitmap[w] |= 1u64 << bit;
            PMM.free_frames += 1;
        }
    }
}

/// Allocate `count` physically contiguous frames (used once, by the heap).
pub fn alloc_frames_contig(count: usize) -> Option<u64> {
    unsafe {
        if !PMM.ready || count == 0 {
            return None;
        }
        let mut run = 0usize;
        let mut start = 0u64;
        let mut frame = 1u64;
        while frame < MAX_FRAMES as u64 {
            let w = (frame / 64) as usize;
            let bit = frame % 64;
            if PMM.bitmap[w] & (1u64 << bit) != 0 {
                if run == 0 {
                    start = frame;
                }
                run += 1;
                if run == count {
                    let mut f = start;
                    while f < start + count as u64 {
                        PMM.bitmap[(f / 64) as usize] &= !(1u64 << (f % 64));
                        PMM.free_frames -= 1;
                        f += 1;
                    }
                    let phys = start * FRAME_SIZE;
                    core::ptr::write_bytes(
                        phys_to_virt(phys) as *mut u8, 0,
                        count * FRAME_SIZE as usize);
                    return Some(phys);
                }
            } else {
                run = 0;
            }
            frame += 1;
        }
        None
    }
}
```

- [ ] **Step 4: Wire the module and the init call into `lib.rs`**

In the module list (after `mod memory;` at `lib.rs:43`):

```rust
pub(crate) mod mm;
```

`HHDM_REQUEST` is `static mut` at `lib.rs:394` — make it visible to `mm.rs`
by changing its declaration to `pub(crate) static mut HHDM_REQUEST: ...`
(same for the struct `LimineHhdmRequest` and `LimineHhdmResponse` if the
compiler requires it: add `pub(crate)` to both struct definitions and the
`offset` field).

In `kmain` (`lib.rs:4884`), immediately after `check_paging();`:

```rust
    mm::pmm_init();
```

- [ ] **Step 5: Rebuild both images**

Run: `& C:\mingw64\mingw64\bin\mingw32-make.exe image image-go`
Expected: both ISOs rebuilt without errors.

- [ ] **Step 6: Run tests to verify they pass**

Run: `python -m pytest tests/mm/test_mm_foundation_v1.py -v`
Expected: 2 passed.

- [ ] **Step 7: Run regression suites (boot, trap, go)**

Run: `python -m pytest tests/boot tests/trap tests/go/test_go_user_service.py -v`
Expected: all pass (new marker must not break ordered-marker tests).

- [ ] **Step 8: Commit**

```bash
git add kernel_rs/src/mm.rs kernel_rs/src/lib.rs tests/mm/test_mm_foundation_v1.py
git commit -m "Add physical frame allocator fed by Limine memmap"
```

---

### Task 2: Kernel heap (`GlobalAlloc`)

**Files:**
- Modify: `kernel_rs/src/mm.rs` (append heap section)
- Modify: `kernel_rs/src/lib.rs:1-4` (`extern crate alloc;`)
- Modify: `kernel_rs/src/lib.rs` `kmain` (heap init + self-test after `pmm_init()`)
- Test: `tests/mm/test_mm_foundation_v1.py` (extend)

- [ ] **Step 1: Extend the test file with failing heap tests**

```python
def test_heap_boot_marker_kernel_lane(qemu_serial):
    out = qemu_serial.stdout
    _find_in_order(out, [
        "MM: pmm ok frames=0x",
        "MM: heap ok size=0x400000",
        "MM: heap selftest ok",
        "RUGO: halt ok",
    ])
    assert "MM: heap none" not in out
    assert "MM: heap selftest err" not in out


def test_heap_boot_marker_go_lane(qemu_serial_go):
    out = qemu_serial_go.stdout
    _find_in_order(out, [
        "MM: heap ok size=0x400000",
        "MM: heap selftest ok",
        "GOINIT: ready",
    ])
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `python -m pytest tests/mm/test_mm_foundation_v1.py -v`
Expected: 2 pass (Task 1), 2 FAIL on `MM: heap ok size=0x400000`.

- [ ] **Step 3: Append the heap allocator to `mm.rs`**

```rust
// ---------------- kernel heap ----------------
//
// First-fit free-list allocator over one contiguous 4 MiB PMM window,
// addressed through the HHDM. Blocks carry a usize header (payload size).
// Free blocks are kept address-sorted and coalesced on free.

use core::alloc::{GlobalAlloc, Layout};

const HEAP_FRAMES: usize = 1024; // 4 MiB
const MIN_BLOCK: usize = 32;
const HEADER: usize = core::mem::size_of::<usize>() * 2;

#[repr(C)]
struct FreeBlock {
    size: usize, // total block size including this header
    next: *mut FreeBlock,
}

struct HeapState {
    head: *mut FreeBlock,
    base: u64,
    size: usize,
    ready: bool,
}

static mut HEAP: HeapState = HeapState {
    head: core::ptr::null_mut(),
    base: 0,
    size: 0,
    ready: false,
};

pub fn heap_init() {
    unsafe {
        let phys = match alloc_frames_contig(HEAP_FRAMES) {
            Some(p) => p,
            None => {
                serial_write(b"MM: heap none\n");
                return;
            }
        };
        let base = phys_to_virt(phys);
        let block = base as *mut FreeBlock;
        (*block).size = HEAP_FRAMES * FRAME_SIZE as usize;
        (*block).next = core::ptr::null_mut();
        HEAP.head = block;
        HEAP.base = base;
        HEAP.size = HEAP_FRAMES * FRAME_SIZE as usize;
        HEAP.ready = true;
        serial_write(b"MM: heap ok size=0x");
        serial_write_hex(HEAP.size as u64);
        serial_write(b"\n");
    }
}

unsafe fn heap_alloc(layout: Layout) -> *mut u8 {
    if !HEAP.ready {
        return core::ptr::null_mut();
    }
    let align = layout.align().max(16);
    let need = layout.size().max(1);

    let mut prev: *mut FreeBlock = core::ptr::null_mut();
    let mut cur = HEAP.head;
    while !cur.is_null() {
        let block_start = cur as usize;
        let payload = block_start + HEADER;
        let aligned = (payload + align - 1) & !(align - 1);
        let pad = aligned - payload;
        let total = HEADER + pad + need;
        let total = (total + MIN_BLOCK - 1) & !(MIN_BLOCK - 1);
        if (*cur).size >= total {
            let remain = (*cur).size - total;
            let next = (*cur).next;
            if remain >= MIN_BLOCK {
                let rest = (block_start + total) as *mut FreeBlock;
                (*rest).size = remain;
                (*rest).next = next;
                if prev.is_null() { HEAP.head = rest; } else { (*prev).next = rest; }
                (*cur).size = total;
            } else if prev.is_null() {
                HEAP.head = next;
            } else {
                (*prev).next = next;
            }
            // stash block start just before the payload for dealloc
            let aligned_ptr = aligned as *mut u8;
            *(aligned_ptr.sub(core::mem::size_of::<usize>()) as *mut usize) =
                block_start;
            return aligned_ptr;
        }
        prev = cur;
        cur = (*cur).next;
    }
    core::ptr::null_mut()
}

unsafe fn heap_dealloc(ptr: *mut u8) {
    if ptr.is_null() || !HEAP.ready {
        return;
    }
    let block_start =
        *(ptr.sub(core::mem::size_of::<usize>()) as *const usize);
    let block = block_start as *mut FreeBlock;

    // insert address-sorted, then coalesce with neighbors
    let mut prev: *mut FreeBlock = core::ptr::null_mut();
    let mut cur = HEAP.head;
    while !cur.is_null() && (cur as usize) < block_start {
        prev = cur;
        cur = (*cur).next;
    }
    (*block).next = cur;
    if prev.is_null() { HEAP.head = block; } else { (*prev).next = block; }

    if !cur.is_null() && block_start + (*block).size == cur as usize {
        (*block).size += (*cur).size;
        (*block).next = (*cur).next;
    }
    if !prev.is_null() && prev as usize + (*prev).size == block_start {
        (*prev).size += (*block).size;
        (*prev).next = (*block).next;
    }
}

pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        heap_alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        heap_dealloc(ptr)
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: KernelAllocator = KernelAllocator;

/// Boot self-test: exercise alloc/dealloc/reuse through the alloc crate.
pub fn heap_selftest() {
    if unsafe { !HEAP.ready } {
        return;
    }
    let mut v: alloc::vec::Vec<u8> = alloc::vec::Vec::with_capacity(4096);
    let mut i = 0u32;
    while i < 4096 {
        v.push((i & 0xFF) as u8);
        i += 1;
    }
    let b = alloc::boxed::Box::new(0xA5A5_5A5Au64);
    let ok = v[4095] == 0xFF && *b == 0xA5A5_5A5Au64;
    drop(v);
    drop(b);
    let again = alloc::boxed::Box::new(7u64);
    if ok && *again == 7 {
        serial_write(b"MM: heap selftest ok\n");
    } else {
        serial_write(b"MM: heap selftest err\n");
    }
}
```

- [ ] **Step 4: Wire alloc crate and init calls in `lib.rs`**

After `#![allow(static_mut_refs)]` (line 2):

```rust
extern crate alloc;
```

In `kmain`, directly after `mm::pmm_init();`:

```rust
    mm::heap_init();
    mm::heap_selftest();
```

- [ ] **Step 5: Rebuild and run tests**

Run: `& C:\mingw64\mingw64\bin\mingw32-make.exe image image-go; python -m pytest tests/mm/test_mm_foundation_v1.py -v`
Expected: 4 passed.

- [ ] **Step 6: Regression run**

Run: `python -m pytest tests/boot tests/trap tests/go/test_go_user_service.py -v`
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add kernel_rs/src/mm.rs kernel_rs/src/lib.rs tests/mm/test_mm_foundation_v1.py
git commit -m "Add kernel heap allocator over PMM-backed window"
```

---

### Task 3: Demand paging for the user heap window

**Files:**
- Modify: `kernel_rs/src/mm.rs` (append demand-paging section)
- Modify: `kernel_rs/src/trap.rs:49-66` (page-fault path: try demand map before kill)
- Test: `tests/mm/test_mm_foundation_v1.py` (extend in Task 4 — kernel side has no user trigger yet)

- [ ] **Step 1: Append demand paging to `mm.rs`**

```rust
// ---------------- demand paging ----------------
//
// User heap window mapped on first touch. The window sits in PD slots 8..11
// of the live user address space (PML4[0] -> PDPT[0] -> PD), far above the
// fixed code (4 MiB) and stack (8 MiB) regions. The fault handler walks the
// CURRENT CR3 (kernel mappings are cloned into every user PML4), allocates
// missing page-table frames and the data frame from the PMM, and retries.

pub const DEMAND_BASE: u64 = 0x0100_0000; // 16 MiB
pub const DEMAND_END: u64 = 0x0180_0000;  // 24 MiB (8 MiB window)
const DEMAND_MAX_FRAMES: u64 = 1024;      // 4 MiB quota for now

static mut DEMAND_MAPPED: u64 = 0;

const PTE_P_W_U: u64 = 0x07;
const PHYS_MASK: u64 = 0x000F_FFFF_FFFF_F000;

/// Try to satisfy a user page fault at `va` by mapping a fresh frame.
/// Returns true when mapped (the faulting instruction must be retried).
pub fn try_demand_map(va: u64) -> bool {
    unsafe {
        if !PMM.ready || va < DEMAND_BASE || va >= DEMAND_END {
            return false;
        }
        if DEMAND_MAPPED >= DEMAND_MAX_FRAMES {
            return false;
        }
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3,
                         options(nomem, nostack));
        let pml4 = phys_to_virt(cr3 & PHYS_MASK) as *mut u64;
        let pml4e = *pml4;
        if pml4e & 1 == 0 {
            return false;
        }
        let pdpt = phys_to_virt(pml4e & PHYS_MASK) as *mut u64;
        let pdpte = *pdpt;
        if pdpte & 1 == 0 {
            return false;
        }
        let pd = phys_to_virt(pdpte & PHYS_MASK) as *mut u64;
        let pd_idx = ((va >> 21) & 0x1FF) as usize;
        let mut pde = *pd.add(pd_idx);
        if pde & 1 == 0 {
            let pt_phys = match alloc_frame() {
                Some(p) => p,
                None => return false,
            };
            *pd.add(pd_idx) = pt_phys | PTE_P_W_U;
            pde = pt_phys | PTE_P_W_U;
        }
        let pt = phys_to_virt(pde & PHYS_MASK) as *mut u64;
        let pt_idx = ((va >> 12) & 0x1FF) as usize;
        if *pt.add(pt_idx) & 1 != 0 {
            // present but faulted: protection error, not demand - let it die
            return false;
        }
        let frame = match alloc_frame() {
            Some(p) => p,
            None => return false,
        };
        *pt.add(pt_idx) = frame | PTE_P_W_U;
        core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
        DEMAND_MAPPED += 1;
        serial_write(b"MM: demand map va=0x");
        serial_write_hex(va & !0xFFF);
        serial_write(b"\n");
        true
    }
}
```

- [ ] **Step 2: Hook the page-fault handler in `trap.rs`**

In the `14 =>` arm, inside the `if cs & 3 == 3` user branch, BEFORE the
`#[cfg(feature = "go_test")]` USERPF print block (trap.rs:51):

```rust
                let cr2: u64;
                core::arch::asm!("mov {}, cr2", out(reg) cr2,
                                 options(nomem, nostack));
                if crate::mm::try_demand_map(cr2) {
                    return;
                }
```

(The existing `go_test` block re-reads CR2 — leave it as-is; it now only
runs for faults the demand mapper refused.)

- [ ] **Step 3: Rebuild both images, verify no regressions**

Run: `& C:\mingw64\mingw64\bin\mingw32-make.exe image image-go; python -m pytest tests/mm tests/boot tests/trap tests/user/test_user_fault.py tests/go/test_go_user_service.py -v`
Expected: all pass. `tests/user/test_user_fault.py` still passes because the
fault address it provokes is outside the demand window.

- [ ] **Step 4: Commit**

```bash
git add kernel_rs/src/mm.rs kernel_rs/src/trap.rs
git commit -m "Map user heap window on demand in the page-fault handler"
```

---

### Task 4: Go userspace demand probe + go-lane runtime test

**Files:**
- Create: `services/go/memprobe.go`
- Modify: `services/go/runtime.go:272-296` (`bootRuntime`: run probe after bootstrap log)
- Test: `tests/mm/test_mm_foundation_v1.py` (extend)

- [ ] **Step 1: Extend the test file with failing demand tests**

```python
def test_demand_paging_go_lane(qemu_serial_go):
    out = qemu_serial_go.stdout
    _find_in_order(out, [
        "GOINIT: bootstrap",
        "MM: demand map va=0x",
        "GOINIT: mem demand ok",
        "GOINIT: svcmgr up",
        "GOINIT: ready",
        "RUGO: halt ok",
    ])
    assert out.count("MM: demand map va=0x") == 16
    assert "GOINIT: mem demand err" not in out
    assert "USERPF:" not in out
```

- [ ] **Step 2: Run to verify it fails**

Run: `python -m pytest tests/mm/test_mm_foundation_v1.py -v`
Expected: previous 4 pass, new one FAILS on `MM: demand map va=0x`.

- [ ] **Step 3: Create `services/go/memprobe.go`**

```go
package main

import "unsafe"

// Demand-paging probe: touch 16 pages of the kernel's demand window
// (16 MiB..24 MiB) before any service starts. Each first touch must fault,
// be mapped by the kernel, and then behave as ordinary zeroed memory.

const (
	demandProbeBase  = uintptr(0x01000000)
	demandProbePages = 16
	demandPageSize   = uintptr(0x1000)
)

var (
	msgMemProbeOK  = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'm', 'e', 'm', ' ', 'd', 'e', 'm', 'a', 'n', 'd', ' ', 'o', 'k', '\n'}
	msgMemProbeErr = [...]byte{'G', 'O', 'I', 'N', 'I', 'T', ':', ' ', 'm', 'e', 'm', ' ', 'd', 'e', 'm', 'a', 'n', 'd', ' ', 'e', 'r', 'r', '\n'}
)

func memDemandProbe() bool {
	var idx uintptr
	for idx = 0; idx < demandProbePages; idx++ {
		p := (*byte)(unsafe.Pointer(demandProbeBase + idx*demandPageSize))
		if *p != 0 {
			return false
		}
		*p = byte(idx + 1)
		if *p != byte(idx+1) {
			return false
		}
	}
	return true
}

func runMemDemandProbe() {
	if memDemandProbe() {
		log(msgMemProbeOK[:])
		return
	}
	log(msgMemProbeErr[:])
}
```

- [ ] **Step 4: Call the probe from `bootRuntime` (runtime.go:280)**

After `log(msgGoInitBootstrap[:])`:

```go
	runMemDemandProbe()
```

- [ ] **Step 5: Rebuild the Go lane and run tests**

Run: `& C:\mingw64\mingw64\bin\mingw32-make.exe image-go; python -m pytest tests/mm/test_mm_foundation_v1.py -v`
Expected: 5 passed. If `tools/build_go.sh` fails with "binary too large",
report it — do not raise the 28 KiB cap silently.

- [ ] **Step 6: Full go-lane regression**

Run: `python -m pytest tests/go tests/runtime/test_service_boot_runtime_v2.py tests/pkg/test_default_shell_app_runtime_v1.py -v`
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add services/go/memprobe.go services/go/runtime.go tests/mm/test_mm_foundation_v1.py
git commit -m "Probe demand-paged user heap window from Go init"
```

---

### Task 5: Makefile target, docs, gates

**Files:**
- Modify: `Makefile` (add `test-mm-foundation-v1` target + .PHONY entry)
- Create: `docs/runtime/memory_v1.md`
- Modify: `docs/architecture/SOURCE_MAP.md` (add `kernel_rs/src/mm.rs` to the runtime tree)
- Modify: `README.md` (What Is Live: add the mm foundation proof line)

- [ ] **Step 1: Add Makefile target (next to test-userspace-model-v2)**

```makefile
test-mm-foundation-v1: image image-go
	$(PYTHON) -m pytest tests/mm/test_mm_foundation_v1.py -v --junitxml=$(OUT)/pytest-mm-foundation-v1.xml
```

Add `test-mm-foundation-v1` to the `.PHONY` list.

- [ ] **Step 2: Write `docs/runtime/memory_v1.md`** — contract doc: PMM
bitmap over Limine memmap, heap window size, demand window
`[0x0100_0000, 0x0180_0000)`, quota 1024 frames, marker formats
(`MM: pmm ok frames=0x<n>`, `MM: heap ok size=0x400000`,
`MM: heap selftest ok`, `MM: demand map va=0x<page>`), and the rule that
kernel-mode touches of unmapped demand pages stay fatal.

- [ ] **Step 3: Update SOURCE_MAP.md and README.md** — add `mm.rs` as
boot-verified runtime source; README "What Is Live" gets
`make test-mm-foundation-v1` with proof `tests/mm/test_mm_foundation_v1.py`.

- [ ] **Step 4: Run the new target end-to-end**

Run: `& C:\mingw64\mingw64\bin\mingw32-make.exe test-mm-foundation-v1`
Expected: 5 passed, junit XML written.

- [ ] **Step 5: Commit**

```bash
git add Makefile docs/runtime/memory_v1.md docs/architecture/SOURCE_MAP.md README.md
git commit -m "Wire mm foundation gate target and runtime contract docs"
```

---

## Self-Review Notes

- Spec coverage: gap §3.1 = PMM (Task 1) + heap (Task 2) + demand paging with
  a real page-fault handler (Tasks 3-4). ASLR/COW/swap/mmap are later phases.
- The demand window deliberately avoids 8-16 MiB (guard gap above the stack).
- `alloc_error_handler`: stable default (panic) covers OOM; the panic handler
  already prints `RUGO: panic` and exits — boot tests would catch it.
- M3-lane kernels also get PMM+heap markers (unconditional) — ordered-marker
  tests tolerate inserted lines; count/absence assertions checked in Task 1
  Step 7 regression run.
