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

// ---------------- kernel heap ----------------
//
// First-fit free-list allocator over one contiguous 4 MiB PMM window,
// addressed through the HHDM. Allocations carry the owning block start just
// before the payload. Free blocks are kept address-sorted and coalesced.

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
