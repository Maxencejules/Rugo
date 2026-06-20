// Dynamic memory foundation: physical frame allocator (PMM), kernel heap,
// and demand paging. Compiled in every lane - no feature gates.
//
// PMM: 1 bit per 4 KiB frame over the first 4 GiB of physical memory,
// populated from the Limine memmap (usable entries only). Frames are
// reached through the Limine HHDM for zeroing.

use crate::{serial_write, serial_write_hex};
use core::sync::atomic::{AtomicU32, Ordering};

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

// PMM spinlock (full-os Part I.3, kernel-wide state locking / slice 5). The
// physical frame allocator is shared kernel state; on a single CPU every caller
// runs with IF=0 (faults and syscalls are interrupt gates) so a critical section
// could never be preempted, but on SMP two CPUs can allocate/free concurrently and
// race the bitmap read-modify-write -> two CPUs could clear the SAME bit and hand
// out one frame twice. This lock serializes the leaf allocator operations. It is a
// LEAF lock (alloc_frame/free_frame/the contig scans never call each other or take
// any other lock and never fault while held -- they touch only the static bitmap
// and the always-mapped HHDM), and the AP timer-preemption handler never calls the
// PMM, so it can never deadlock. Uncontended (one atomic CAS) on -smp 1.
static PMM_LOCK: AtomicU32 = AtomicU32::new(0);

#[inline]
fn pmm_lock() {
    while PMM_LOCK
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}
#[inline]
fn pmm_unlock() {
    PMM_LOCK.store(0, Ordering::Release);
}

pub fn hhdm_offset() -> u64 {
    unsafe { PMM.hhdm }
}

pub fn phys_to_virt(phys: u64) -> u64 {
    unsafe { phys + PMM.hhdm }
}

/// Number of physical frames currently free (for sysinfo / metrics).
pub fn free_frames() -> u64 {
    pmm_lock();
    let n = unsafe { PMM.free_frames };
    pmm_unlock();
    n
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
/// SMP-safe: the bitmap read-modify-write runs under the PMM lock.
pub fn alloc_frame() -> Option<u64> {
    pmm_lock();
    let r = unsafe { alloc_frame_locked() };
    pmm_unlock();
    r
}

unsafe fn alloc_frame_locked() -> Option<u64> {
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
            core::ptr::write_bytes(phys_to_virt(phys) as *mut u8, 0, FRAME_SIZE as usize);
            return Some(phys);
        }
        w = (w + 1) % BITMAP_WORDS;
        scanned += 1;
    }
    None
}

/// Return a frame to the pool. `phys` must come from `alloc_frame`. SMP-safe.
pub fn free_frame(phys: u64) {
    pmm_lock();
    unsafe { free_frame_locked(phys) };
    pmm_unlock();
}

unsafe fn free_frame_locked(phys: u64) {
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

// Kernel heap spinlock (full-os Part I.3, kernel-wide locking / slice 5). The
// first-fit free list (HEAP.head + the FreeBlock chain) is shared kernel state that
// every Vec/Box touches through the global allocator; on SMP two CPUs allocating or
// freeing concurrently would corrupt the list (a lost block, one block handed to two
// callers, or broken coalescing). This lock serializes the list operations. It is
// INDEPENDENT of the PMM lock and a LEAF: heap_alloc/heap_dealloc only walk the
// pre-allocated window's free list -- they never call the PMM (the heap window is
// reserved once at heap_init), never recurse into the allocator, and never fault
// while held -- and the AP timer-preemption handler never allocates, so it can never
// deadlock. Uncontended (one atomic CAS) on -smp 1.
static HEAP_LOCK: AtomicU32 = AtomicU32::new(0);

#[inline]
fn heap_lock() {
    while HEAP_LOCK
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}
#[inline]
fn heap_unlock() {
    HEAP_LOCK.store(0, Ordering::Release);
}

unsafe fn heap_alloc(layout: Layout) -> *mut u8 {
    heap_lock();
    let r = heap_alloc_locked(layout);
    heap_unlock();
    r
}

unsafe fn heap_alloc_locked(layout: Layout) -> *mut u8 {
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
    heap_lock();
    heap_dealloc_locked(ptr);
    heap_unlock();
}

unsafe fn heap_dealloc_locked(ptr: *mut u8) {
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
    pmm_lock();
    let r = unsafe { alloc_frames_contig_locked(count) };
    pmm_unlock();
    r
}

unsafe fn alloc_frames_contig_locked(count: usize) -> Option<u64> {
    {
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
pub const DEMAND_END: u64 = 0x0200_0000;  // 32 MiB (16 MiB window)
const DEMAND_MAX_FRAMES: u64 = 2048;      // 8 MiB quota for now

// Stack area for dynamically spawned tasks (slots >= 5): 128 KiB strides
// from DEMAND_STACK_BASE, growing down. The bottom 16 KiB of every stride
// is a guard zone the demand mapper refuses, so a runaway stack faults
// fatally instead of silently walking into its neighbour.
pub const DEMAND_STACK_BASE: u64 = 0x0190_0000;
pub const DEMAND_STACK_STRIDE: u64 = 0x2_0000;
const DEMAND_STACK_GUARD: u64 = 0x4000;

static mut DEMAND_MAPPED: u64 = 0;

const PTE_P_W_U: u64 = 0x07;
const PTE_P_U: u64 = 0x05; // present + user, read-only
const PTE_W: u64 = 0x02;
const PTE_NX: u64 = 1 << 63;
// Software bit (available to the OS): marks a copy-on-write page so the
// fault handler can tell a forked CoW page from a genuinely read-only
// mmap. Without it, an mmap(PROT_READ) write would be silently promoted.
const PTE_COW: u64 = 1 << 9;
const PHYS_MASK: u64 = 0x000F_FFFF_FFFF_F000;

// Copy-on-write refcounts (full-os guide Part I.2). Indexed by frame
// number. The value is the count of owners BEYOND the first: 0 = a single
// owner (the common case, untracked), N = N+1 owners sharing the frame.
// u8 caps at 256 owners, far above R4_MAX_TASKS, so saturation never bites.
static mut COW_REFCOUNT: [u8; MAX_FRAMES] = [0u8; MAX_FRAMES];

#[inline]
unsafe fn cow_incr(phys: u64) {
    let idx = (phys / FRAME_SIZE) as usize;
    if idx < MAX_FRAMES {
        COW_REFCOUNT[idx] = COW_REFCOUNT[idx].saturating_add(1);
    }
}

/// Release one reference to a leaf data frame. If other owners remain,
/// just drop the refcount; only the last owner frees the frame (and rolls
/// back the demand-frame accounting, since DEMAND_MAPPED counts physical
/// frames, not mappings).
#[inline]
unsafe fn cow_release_leaf(phys: u64) {
    let idx = (phys / FRAME_SIZE) as usize;
    if idx < MAX_FRAMES && COW_REFCOUNT[idx] > 0 {
        COW_REFCOUNT[idx] -= 1;
        return;
    }
    free_frame(phys);
    if DEMAND_MAPPED > 0 {
        DEMAND_MAPPED -= 1;
    }
}

// Exec app window: ELF segments load through the same demand path, so
// these pages stay executable. Everything else in the window is data
// (TinyGo heap, task stacks) and gets NX - W^X for dynamic user memory.
const EXEC_WINDOW_BASE: u64 = 0x0140_0000;
const EXEC_WINDOW_END: u64 = 0x0180_0000;

/// Enable EFER.NXE so PTE bit 63 (no-execute) is honored. Called once
/// at boot in every lane, before the first demand mapping.
pub fn enable_nx() {
    unsafe {
        let (mut lo, hi): (u32, u32);
        core::arch::asm!(
            "rdmsr",
            in("ecx") 0xC000_0080u32,
            out("eax") lo,
            out("edx") hi,
            options(nomem, nostack),
        );
        lo |= 1 << 11;
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0xC000_0080u32,
            in("eax") lo,
            in("edx") hi,
            options(nomem, nostack),
        );
    }
    crate::serial_write(b"MM: nx on\n");
}

/// Try to satisfy a user page fault at `va` by mapping a fresh frame.
/// Returns true when mapped (the faulting instruction must be retried).
pub fn try_demand_map(va: u64) -> bool {
    unsafe {
        if !PMM.ready || va < DEMAND_BASE || va >= DEMAND_END {
            return false;
        }
        if va >= DEMAND_STACK_BASE
            && (va - DEMAND_STACK_BASE) % DEMAND_STACK_STRIDE < DEMAND_STACK_GUARD
        {
            return false; // stack guard zone
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
        let nx = if va >= EXEC_WINDOW_BASE && va < EXEC_WINDOW_END {
            0
        } else {
            PTE_NX
        };
        *pt.add(pt_idx) = frame | PTE_P_W_U | nx;
        core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
        DEMAND_MAPPED += 1;
        // Per-page markers only for the boot-probe sub-range; the TinyGo
        // heap above 0x110_0000 maps silently so its page count (which
        // varies with workload) never perturbs marker-count assertions.
        if va < DEMAND_BASE + 0x1_0000 {
            serial_write(b"MM: demand map va=0x");
            serial_write_hex(va & !0xFFF);
            serial_write(b"\n");
        }
        true
    }
}

// ---- per-process address spaces (full-os implementation keystone) ----
//
// A spawned task gets its own PML4. Its kernel-half entries are cloned
// from the shared/boot table (so kernel text, the HHDM and the page-table
// pool stay reachable under any CR3), while PML4[0] gets a fresh, private
// PDPT/PD subtree covering the user region [0, 1 GiB). All user memory
// (exec window, demand heap, demand stacks, args page) lives in that first
// 1 GiB, so a single PD per address space suffices. CR3 is reloaded per
// task in r4_switch_to (flushing the TLB); try_demand_map then resolves
// faults against the faulting task's table automatically, since it reads
// CR3. Releasing an address space walks only the private user subtree, so
// the cloned kernel-half pages are never freed.

/// Create a private address space cloned from `src_pml4_phys`. Returns the
/// new PML4 physical address, or None if frames are exhausted.
pub unsafe fn address_space_create(src_pml4_phys: u64) -> Option<u64> {
    let pml4 = alloc_frame()?;
    let pdpt = match alloc_frame() {
        Some(p) => p,
        None => {
            free_frame(pml4);
            return None;
        }
    };
    let pd = match alloc_frame() {
        Some(p) => p,
        None => {
            free_frame(pdpt);
            free_frame(pml4);
            return None;
        }
    };
    // Clone every entry (kernel-half mappings + HHDM) from the source.
    let src = phys_to_virt(src_pml4_phys & PHYS_MASK) as *const u64;
    let dst = phys_to_virt(pml4) as *mut u64;
    let mut i = 0usize;
    while i < 512 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    // Override the user half with a fresh, private subtree. The PD is left
    // zeroed (alloc_frame zeroes); demand faults fill PD/PT entries.
    *dst = pdpt | PTE_P_W_U;
    *(phys_to_virt(pdpt) as *mut u64) = pd | PTE_P_W_U;
    Some(pml4)
}

/// Resolve (allocating on demand) the user page covering `va` in
/// `pml4_phys`; returns the backing frame's physical address. NX matches
/// try_demand_map: cleared inside the exec window, set everywhere else.
unsafe fn as_get_page(pml4_phys: u64, va: u64) -> Option<u64> {
    let pml4 = phys_to_virt(pml4_phys & PHYS_MASK) as *mut u64;
    let pml4e = *pml4.add(((va >> 39) & 0x1FF) as usize);
    if pml4e & 1 == 0 {
        return None;
    }
    let pdpt = phys_to_virt(pml4e & PHYS_MASK) as *mut u64;
    let pdpte = *pdpt.add(((va >> 30) & 0x1FF) as usize);
    if pdpte & 1 == 0 {
        return None;
    }
    let pd = phys_to_virt(pdpte & PHYS_MASK) as *mut u64;
    let pd_idx = ((va >> 21) & 0x1FF) as usize;
    let mut pde = *pd.add(pd_idx);
    if pde & 1 == 0 {
        let pt = alloc_frame()?;
        *pd.add(pd_idx) = pt | PTE_P_W_U;
        pde = pt | PTE_P_W_U;
    }
    let pt = phys_to_virt(pde & PHYS_MASK) as *mut u64;
    let pt_idx = ((va >> 12) & 0x1FF) as usize;
    let pte = *pt.add(pt_idx);
    if pte & 1 != 0 {
        return Some(pte & PHYS_MASK);
    }
    let frame = alloc_frame()?;
    let nx = if va >= EXEC_WINDOW_BASE && va < EXEC_WINDOW_END {
        0
    } else {
        PTE_NX
    };
    *pt.add(pt_idx) = frame | PTE_P_W_U | nx;
    DEMAND_MAPPED += 1;
    Some(frame)
}

/// Copy `data` into address space `pml4_phys` at user va `va`, mapping
/// pages as needed. Loads ELF segments into a child before it runs.
pub unsafe fn as_copyout(pml4_phys: u64, va: u64, data: &[u8]) -> bool {
    let mut off = 0usize;
    while off < data.len() {
        let cur = va + off as u64;
        let frame = match as_get_page(pml4_phys, cur) {
            Some(f) => f,
            None => return false,
        };
        let page_off = (cur & 0xFFF) as usize;
        let n = core::cmp::min(0x1000 - page_off, data.len() - off);
        core::ptr::copy_nonoverlapping(
            data.as_ptr().add(off),
            (phys_to_virt(frame) as *mut u8).add(page_off),
            n,
        );
        off += n;
    }
    true
}

/// Ensure [va, va+len) is mapped in `pml4_phys`. Fresh frames arrive
/// zeroed, so this is the BSS path.
pub unsafe fn as_map_zeroed(pml4_phys: u64, va: u64, len: usize) -> bool {
    let mut off = 0usize;
    while off < len {
        let cur = va + off as u64;
        if as_get_page(pml4_phys, cur).is_none() {
            return false;
        }
        let page_off = (cur & 0xFFF) as usize;
        let n = core::cmp::min(0x1000 - page_off, len - off);
        off += n;
    }
    true
}

/// Free a private address space: walk PML4[0] -> PDPT[0] -> PD -> PTs,
/// returning every user frame and page-table frame to the PMM. Only the
/// private user subtree is walked, so cloned kernel-half entries are left
/// untouched. The demand-frame accounting is rolled back per leaf freed.
pub unsafe fn address_space_release(pml4_phys: u64) {
    let pml4 = phys_to_virt(pml4_phys & PHYS_MASK) as *mut u64;
    let pml4e = *pml4;
    if pml4e & 1 != 0 {
        let pdpt_phys = pml4e & PHYS_MASK;
        let pdpt = phys_to_virt(pdpt_phys) as *mut u64;
        let pdpte = *pdpt;
        if pdpte & 1 != 0 {
            let pd_phys = pdpte & PHYS_MASK;
            let pd = phys_to_virt(pd_phys) as *mut u64;
            let mut i = 0usize;
            while i < 512 {
                let pde = *pd.add(i);
                if pde & 1 != 0 {
                    let pt_phys = pde & PHYS_MASK;
                    let pt = phys_to_virt(pt_phys) as *mut u64;
                    let mut j = 0usize;
                    while j < 512 {
                        let pte = *pt.add(j);
                        if pte & 1 != 0 {
                            // Leaf data frames may be CoW-shared with another
                            // address space; only the last owner frees them.
                            cow_release_leaf(pte & PHYS_MASK);
                        }
                        j += 1;
                    }
                    free_frame(pt_phys);
                }
                i += 1;
            }
            free_frame(pd_phys);
        }
        free_frame(pdpt_phys);
    }
    free_frame(pml4_phys & PHYS_MASK);
}

// ---- fork + copy-on-write (full-os guide Part I.2) ----

/// Fork `parent_pml4` into a copy-on-write child. The kernel half is
/// cloned; the user leaf data frames are SHARED read-only between parent
/// and child (refcount bumped, W cleared in both), while the page-table
/// frames (PT) are copied so each space has its own tree. The first write
/// from either side traps into cow_break, which gives the writer a private
/// copy. Returns the child PML4 physical address.
pub unsafe fn address_space_fork(parent_pml4: u64) -> Option<u64> {
    // Fresh child PML4/PDPT/PD with the kernel half cloned from the parent.
    let child = address_space_create(parent_pml4)?;
    let child_pml4 = phys_to_virt(child & PHYS_MASK) as *mut u64;
    let child_pdpt = phys_to_virt(*child_pml4 & PHYS_MASK) as *mut u64;
    let child_pd = phys_to_virt(*child_pdpt & PHYS_MASK) as *mut u64;

    let p_pml4 = phys_to_virt(parent_pml4 & PHYS_MASK) as *const u64;
    let p_pml4e = *p_pml4; // user half is PML4[0]
    if p_pml4e & 1 == 0 {
        return Some(child);
    }
    let p_pdpt = phys_to_virt(p_pml4e & PHYS_MASK) as *const u64;
    let p_pdpte = *p_pdpt; // PDPT[0]
    if p_pdpte & 1 == 0 {
        return Some(child);
    }
    let p_pd = phys_to_virt(p_pdpte & PHYS_MASK) as *mut u64;

    let mut i = 0usize;
    while i < 512 {
        let p_pde = *p_pd.add(i);
        if p_pde & 1 != 0 {
            let c_pt_phys = match alloc_frame() {
                Some(f) => f,
                None => {
                    address_space_release(child);
                    return None;
                }
            };
            let p_pt = phys_to_virt(p_pde & PHYS_MASK) as *mut u64;
            let c_pt = phys_to_virt(c_pt_phys) as *mut u64;
            let mut j = 0usize;
            while j < 512 {
                let pte = *p_pt.add(j);
                if pte & 1 != 0 {
                    // Share the data frame between parent and child. A
                    // writable page becomes read-only + CoW so the first write
                    // on either side traps and copies; a genuinely read-only
                    // page (PROT_READ mmap, mprotect-RO) is shared AS-IS with no
                    // CoW mark, so a write to it still faults to containment
                    // (no silent promotion). The refcount is bumped either way
                    // so address_space_release stays balanced.
                    let ro = if pte & PTE_W != 0 {
                        (pte & !PTE_W) | PTE_COW
                    } else {
                        pte
                    };
                    *p_pt.add(j) = ro;
                    *c_pt.add(j) = ro;
                    cow_incr(pte & PHYS_MASK);
                }
                j += 1;
            }
            *child_pd.add(i) = c_pt_phys | PTE_P_W_U;
        }
        i += 1;
    }
    Some(child)
}

/// Resolve a copy-on-write write fault at `va` against the current CR3.
/// Returns true if it was a CoW page and has been made writable (the
/// faulting instruction must retry). If the page has a single owner it is
/// simply re-marked writable; otherwise the writer gets a private copy.
pub unsafe fn cow_break(va: u64) -> bool {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let pml4 = phys_to_virt(cr3 & PHYS_MASK) as *mut u64;
    let pml4e = *pml4.add(((va >> 39) & 0x1FF) as usize);
    if pml4e & 1 == 0 {
        return false;
    }
    let pdpt = phys_to_virt(pml4e & PHYS_MASK) as *mut u64;
    let pdpte = *pdpt.add(((va >> 30) & 0x1FF) as usize);
    if pdpte & 1 == 0 {
        return false;
    }
    let pd = phys_to_virt(pdpte & PHYS_MASK) as *mut u64;
    let pde = *pd.add(((va >> 21) & 0x1FF) as usize);
    if pde & 1 == 0 {
        return false;
    }
    let pt = phys_to_virt(pde & PHYS_MASK) as *mut u64;
    let pt_idx = ((va >> 12) & 0x1FF) as usize;
    let pte = *pt.add(pt_idx);
    // Only a present, CoW-marked page is a candidate. A genuinely
    // read-only mmap (no PTE_COW) must fault through to containment.
    if pte & 1 == 0 || pte & PTE_COW == 0 {
        return false;
    }
    let frame = pte & PHYS_MASK;
    let idx = (frame / FRAME_SIZE) as usize;
    if idx < MAX_FRAMES && COW_REFCOUNT[idx] == 0 {
        // Sole owner: clear the CoW mark and restore write permission.
        *pt.add(pt_idx) = (pte & !PTE_COW) | PTE_W;
        core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
        return true;
    }
    // Shared: take a private copy and drop our reference to the shared one.
    let fresh = match alloc_frame() {
        Some(f) => f,
        None => return false,
    };
    core::ptr::copy_nonoverlapping(
        phys_to_virt(frame) as *const u8,
        phys_to_virt(fresh) as *mut u8,
        FRAME_SIZE as usize,
    );
    if idx < MAX_FRAMES && COW_REFCOUNT[idx] > 0 {
        COW_REFCOUNT[idx] -= 1;
    }
    *pt.add(pt_idx) = fresh | (pte & !PHYS_MASK & !PTE_COW) | PTE_W;
    DEMAND_MAPPED += 1;
    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
    true
}

// ---- mmap / brk / munmap backing (full-os guide Part I.4) ----

/// Map one user page at `va` in the current address space with the given
/// prot bits (1=R, 2=W, 4=X). Idempotent if already present. Returns false
/// only on frame exhaustion.
pub unsafe fn vm_map_current(va: u64, prot: u64) -> bool {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let pml4 = phys_to_virt(cr3 & PHYS_MASK) as *mut u64;
    let pml4e = *pml4.add(((va >> 39) & 0x1FF) as usize);
    if pml4e & 1 == 0 {
        return false;
    }
    let pdpt = phys_to_virt(pml4e & PHYS_MASK) as *mut u64;
    let pdpte = *pdpt.add(((va >> 30) & 0x1FF) as usize);
    if pdpte & 1 == 0 {
        return false;
    }
    let pd = phys_to_virt(pdpte & PHYS_MASK) as *mut u64;
    let pd_idx = ((va >> 21) & 0x1FF) as usize;
    let mut pde = *pd.add(pd_idx);
    if pde & 1 == 0 {
        let pt = match alloc_frame() {
            Some(f) => f,
            None => return false,
        };
        *pd.add(pd_idx) = pt | PTE_P_W_U;
        pde = pt | PTE_P_W_U;
    }
    let pt = phys_to_virt(pde & PHYS_MASK) as *mut u64;
    let pt_idx = ((va >> 12) & 0x1FF) as usize;
    if *pt.add(pt_idx) & 1 != 0 {
        return true; // already mapped
    }
    let frame = match alloc_frame() {
        Some(f) => f,
        None => return false,
    };
    let mut flags = PTE_P_U;
    if prot & 2 != 0 {
        flags |= PTE_W;
    }
    if prot & 4 == 0 {
        flags |= PTE_NX;
    }
    *pt.add(pt_idx) = frame | flags;
    DEMAND_MAPPED += 1;
    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
    true
}

/// Unmap one user page at `va` in the current address space, returning its
/// frame (respecting CoW refcounts). Returns true if a page was unmapped.
pub unsafe fn vm_unmap_current(va: u64) -> bool {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let pml4 = phys_to_virt(cr3 & PHYS_MASK) as *mut u64;
    let pml4e = *pml4.add(((va >> 39) & 0x1FF) as usize);
    if pml4e & 1 == 0 {
        return false;
    }
    let pdpt = phys_to_virt(pml4e & PHYS_MASK) as *mut u64;
    let pdpte = *pdpt.add(((va >> 30) & 0x1FF) as usize);
    if pdpte & 1 == 0 {
        return false;
    }
    let pd = phys_to_virt(pdpte & PHYS_MASK) as *mut u64;
    let pde = *pd.add(((va >> 21) & 0x1FF) as usize);
    if pde & 1 == 0 {
        return false;
    }
    let pt = phys_to_virt(pde & PHYS_MASK) as *mut u64;
    let pt_idx = ((va >> 12) & 0x1FF) as usize;
    let pte = *pt.add(pt_idx);
    if pte & 1 == 0 {
        return false;
    }
    *pt.add(pt_idx) = 0;
    cow_release_leaf(pte & PHYS_MASK);
    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
    true
}

/// Change the protection of one mapped user page at `va` in the current
/// address space (prot 1=R, 2=W, 4=X), preserving its frame. Clears any
/// CoW mark. Returns true if the page was present.
pub unsafe fn vm_protect_current(va: u64, prot: u64) -> bool {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let pml4 = phys_to_virt(cr3 & PHYS_MASK) as *mut u64;
    let pml4e = *pml4.add(((va >> 39) & 0x1FF) as usize);
    if pml4e & 1 == 0 {
        return false;
    }
    let pdpt = phys_to_virt(pml4e & PHYS_MASK) as *mut u64;
    let pdpte = *pdpt.add(((va >> 30) & 0x1FF) as usize);
    if pdpte & 1 == 0 {
        return false;
    }
    let pd = phys_to_virt(pdpte & PHYS_MASK) as *mut u64;
    let pde = *pd.add(((va >> 21) & 0x1FF) as usize);
    if pde & 1 == 0 {
        return false;
    }
    let pt = phys_to_virt(pde & PHYS_MASK) as *mut u64;
    let pt_idx = ((va >> 12) & 0x1FF) as usize;
    let pte = *pt.add(pt_idx);
    if pte & 1 == 0 {
        return false;
    }
    let mut flags = PTE_P_U;
    if prot & 2 != 0 {
        flags |= PTE_W;
    }
    if prot & 4 == 0 {
        flags |= PTE_NX;
    }
    *pt.add(pt_idx) = (pte & PHYS_MASK) | flags;
    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
    true
}

// ---- huge pages (full-os guide Part I.4) ----

const PTE_PS: u64 = 1 << 7; // page-size bit: a PD entry maps a 2 MiB page

/// Allocate `count` physically-contiguous frames whose START frame is a multiple
/// of `align` frames (so the physical base is `align`·4 KiB-aligned). A 2 MiB
/// huge page needs `count = align = 512`. Returns the physical base, or None.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub fn alloc_frames_contig_aligned(count: usize, align: usize) -> Option<u64> {
    pmm_lock();
    let r = unsafe { alloc_frames_contig_aligned_locked(count, align) };
    pmm_unlock();
    r
}

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn alloc_frames_contig_aligned_locked(count: usize, align: usize) -> Option<u64> {
    {
        if !PMM.ready || count == 0 || align == 0 {
            return None;
        }
        let align = align as u64;
        let mut start = align; // skip frame 0; first aligned candidate
        while start + count as u64 <= MAX_FRAMES as u64 {
            let mut ok = true;
            let mut f = start;
            while f < start + count as u64 {
                if PMM.bitmap[(f / 64) as usize] & (1u64 << (f % 64)) == 0 {
                    ok = false;
                    break;
                }
                f += 1;
            }
            if ok {
                let mut f2 = start;
                while f2 < start + count as u64 {
                    PMM.bitmap[(f2 / 64) as usize] &= !(1u64 << (f2 % 64));
                    PMM.free_frames -= 1;
                    f2 += 1;
                }
                let phys = start * FRAME_SIZE;
                core::ptr::write_bytes(
                    phys_to_virt(phys) as *mut u8,
                    0,
                    count * FRAME_SIZE as usize,
                );
                return Some(phys);
            }
            start += align;
        }
        None
    }
}

/// Map a 2 MiB huge page at 2 MiB-aligned `va` in the current address space
/// (full-os guide Part I.4): walk/allocate PML4 → PDPT (4 KiB tables), then
/// install a single PD entry with the page-size bit, so one TLB entry covers
/// 2 MiB instead of 512 4 KiB entries. `prot` bit1 = W, bit2 = X (else NX). A
/// kernel page (no user bit). Returns false on misalignment or frame exhaustion.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub fn free_frames_contig(phys: u64, count: usize) {
    let mut i = 0usize;
    while i < count {
        free_frame(phys + (i as u64) * FRAME_SIZE);
        i += 1;
    }
}

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub unsafe fn vm_map_huge_current(va: u64, prot: u64) -> bool {
    if va & 0x1F_FFFF != 0 {
        return false;
    }
    let phys2m = match alloc_frames_contig_aligned(512, 512) {
        Some(p) => p,
        None => return false,
    };
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let mut table = cr3 & PHYS_MASK;
    // All-or-nothing (like mmio_map_4k): defer linking newly-allocated tables
    // until the whole walk succeeds. On any alloc failure, free the huge region
    // AND every pending table frame and leave the live page tree untouched (a
    // freshly-allocated table is reachable via the HHDM before it is linked, so
    // the walk can still descend into it).
    let mut pending: [(u64, usize, u64); 2] = [(0, 0, 0); 2];
    let mut npending = 0usize;
    for &shift in &[39u32, 30] {
        let t = phys_to_virt(table) as *mut u64;
        let i = ((va >> shift) & 0x1FF) as usize;
        let e = *t.add(i);
        if e & 1 == 0 {
            let new = match alloc_frame() {
                Some(f) => f,
                None => {
                    free_frames_contig(phys2m, 512);
                    let mut k = 0;
                    while k < npending {
                        free_frame(pending[k].2);
                        k += 1;
                    }
                    return false;
                }
            };
            pending[npending] = (table, i, new);
            npending += 1;
            table = new;
        } else {
            table = e & PHYS_MASK;
        }
    }
    let mut k = 0;
    while k < npending {
        let (parent, i, new) = pending[k];
        *(phys_to_virt(parent) as *mut u64).add(i) = new | PTE_P_W_U;
        k += 1;
    }
    let pd = phys_to_virt(table) as *mut u64;
    let pd_idx = ((va >> 21) & 0x1FF) as usize;
    let mut flags = 1u64 | PTE_PS; // present, 2 MiB page, kernel (no user bit)
    if prot & 2 != 0 {
        flags |= PTE_W;
    }
    if prot & 4 == 0 {
        flags |= PTE_NX;
    }
    *pd.add(pd_idx) = phys2m | flags;
    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
    true
}

/// Read the PD entry covering `va` in the current address space (so the self-test
/// can confirm the page-size bit). Returns the raw entry, or None if not present.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn pd_entry_current(va: u64) -> Option<u64> {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let pml4 = phys_to_virt(cr3 & PHYS_MASK) as *mut u64;
    let pml4e = *pml4.add(((va >> 39) & 0x1FF) as usize);
    if pml4e & 1 == 0 {
        return None;
    }
    let pdpt = phys_to_virt(pml4e & PHYS_MASK) as *mut u64;
    let pdpte = *pdpt.add(((va >> 30) & 0x1FF) as usize);
    if pdpte & 1 == 0 {
        return None;
    }
    let pd = phys_to_virt(pdpte & PHYS_MASK) as *mut u64;
    let pde = *pd.add(((va >> 21) & 0x1FF) as usize);
    if pde & 1 == 0 {
        return None;
    }
    Some(pde)
}

/// Boot self-test (full-os guide Part I.4): map a 2 MiB huge page, confirm the PD
/// entry carries the page-size bit, and read/write through it at offset 0 and the
/// last 8 bytes of the 2 MiB (proving the single mapping spans the whole region).
/// Emits `HUGEPAGE: 2M ok` / `2M fail`.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub fn huge_page_selftest() {
    unsafe {
        // A 2 MiB-aligned VA in the same otherwise-unused high kernel PML4 slot
        // as the device-MMIO window (1 GiB in, a distinct PDPT/PD subtree).
        const HV: u64 = 0xFFFF_FF00_4000_0000;
        if !vm_map_huge_current(HV, 2) {
            serial_write(b"HUGEPAGE: 2M fail\n");
            return;
        }
        let pde = match pd_entry_current(HV) {
            Some(e) => e,
            None => {
                serial_write(b"HUGEPAGE: 2M fail\n");
                return;
            }
        };
        if pde & PTE_PS == 0 {
            serial_write(b"HUGEPAGE: 2M fail\n"); // present but not a 2 MiB page
            return;
        }
        let lo = HV as *mut u64;
        let hi = (HV + 0x20_0000 - 8) as *mut u64; // last 8 bytes of the 2 MiB
        core::ptr::write_volatile(lo, 0xCAFE_BABE_1234_5678);
        core::ptr::write_volatile(hi, 0x0BAD_F00D_DEAD_BEEF);
        if core::ptr::read_volatile(lo) == 0xCAFE_BABE_1234_5678
            && core::ptr::read_volatile(hi) == 0x0BAD_F00D_DEAD_BEEF
        {
            serial_write(b"HUGEPAGE: 2M ok\n");
        } else {
            serial_write(b"HUGEPAGE: 2M fail\n");
        }
    }
}

// ---- swap / page eviction (full-os guide Part I.4) ----
//
// Evict a present user page to a disk swap area, freeing its physical frame and
// leaving a "swapped" marker in the PTE (present=0); a later access page-faults
// and `try_swap_in` allocates a fresh frame, reads the page back, and remaps it.
// A small fixed swap area (16 slots × 4 KiB) on a scratch disk region. Go lane
// only (it needs the block device). v1: a fixed-size swap file, no eviction
// policy / LRU clock; choosing victims is the carry-forward.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const SWAP_BASE_LBA: u64 = 1700; // free scratch region (cache uses 1610..)
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const SWAP_SLOTS: usize = 16;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const PTE_SWAPPED: u64 = 1 << 10; // software bit: a swapped-out (present=0) page
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut SWAP_USED: [bool; SWAP_SLOTS] = [false; SWAP_SLOTS];

/// Walk the current address space to the leaf PTE pointer for `va`, whether or
/// not the leaf is present (so it serves both eviction and swap-in). Returns None
/// if an upper level is absent.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn leaf_pte_ptr(va: u64) -> Option<*mut u64> {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let pml4 = phys_to_virt(cr3 & PHYS_MASK) as *mut u64;
    let pml4e = *pml4.add(((va >> 39) & 0x1FF) as usize);
    if pml4e & 1 == 0 {
        return None;
    }
    let pdpt = phys_to_virt(pml4e & PHYS_MASK) as *mut u64;
    let pdpte = *pdpt.add(((va >> 30) & 0x1FF) as usize);
    if pdpte & 1 == 0 {
        return None;
    }
    let pd = phys_to_virt(pdpte & PHYS_MASK) as *mut u64;
    let pde = *pd.add(((va >> 21) & 0x1FF) as usize);
    if pde & 1 == 0 || pde & PTE_PS != 0 {
        return None; // absent, or a 2 MiB huge page (not swappable here)
    }
    let pt = phys_to_virt(pde & PHYS_MASK) as *mut u64;
    Some(pt.add(((va >> 12) & 0x1FF) as usize))
}

/// Evict the present user page at `va` (current address space) to a free swap
/// slot: write its frame to disk, mark the PTE swapped (present=0 + slot index),
/// and free the frame. Returns false if `va` is not a present page or no swap
/// slot / disk is available.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub unsafe fn swap_out_current(va: u64) -> bool {
    let pte_ptr = match leaf_pte_ptr(va) {
        Some(p) => p,
        None => return false,
    };
    let pte = *pte_ptr;
    if pte & 1 == 0 {
        return false; // not present -> nothing to evict
    }
    let frame = pte & PHYS_MASK;
    let mut slot = SWAP_SLOTS;
    let mut i = 0;
    while i < SWAP_SLOTS {
        if !SWAP_USED[i] {
            slot = i;
            break;
        }
        i += 1;
    }
    if slot == SWAP_SLOTS {
        return false; // swap full
    }
    if !crate::blk_write_page(SWAP_BASE_LBA + (slot as u64) * 8, frame) {
        return false;
    }
    SWAP_USED[slot] = true;
    // Leave a swapped marker: present=0, PTE_SWAPPED set, slot index in bits 12+.
    *pte_ptr = ((slot as u64) << 12) | PTE_SWAPPED;
    free_frame(frame);
    if DEMAND_MAPPED > 0 {
        DEMAND_MAPPED -= 1;
    }
    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
    true
}

/// Page-fault path: if `va` maps a swapped-out page, allocate a fresh frame, read
/// the page back from its swap slot, remap it, and free the slot. Returns true if
/// it was a swapped page that has been restored (the faulting access must retry).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub unsafe fn try_swap_in(va: u64) -> bool {
    let pte_ptr = match leaf_pte_ptr(va) {
        Some(p) => p,
        None => return false,
    };
    let pte = *pte_ptr;
    if pte & 1 != 0 || pte & PTE_SWAPPED == 0 {
        return false; // present, or not a swapped entry
    }
    let slot = ((pte >> 12) & (SWAP_SLOTS as u64 - 1)) as usize;
    let frame = match alloc_frame() {
        Some(f) => f,
        None => return false,
    };
    if !crate::blk_read_page(SWAP_BASE_LBA + (slot as u64) * 8, frame) {
        free_frame(frame);
        return false;
    }
    SWAP_USED[slot] = false;
    let nx = if va >= EXEC_WINDOW_BASE && va < EXEC_WINDOW_END {
        0
    } else {
        PTE_NX
    };
    *pte_ptr = frame | PTE_P_W_U | nx;
    DEMAND_MAPPED += 1;
    core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack));
    true
}

/// Swap self-test (full-os guide Part I.4): map a user page in a fresh address
/// space, write a pattern, evict it to disk (frame freed, PTE marked swapped),
/// then swap it back in and confirm the page reads back byte-exact through its VA
/// — proving the page survived the round-trip frame -> disk -> fresh frame. Emits
/// `SWAP: roundtrip ok` / `fail` / `skip` (no disk).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub fn swap_selftest() {
    unsafe {
        if !crate::storage::r4_storage_available() {
            serial_write(b"SWAP: skip\n");
            return;
        }
        let kcr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kcr3, options(nomem, nostack));
        let ucr3 = match address_space_create(kcr3) {
            Some(p) => p,
            None => {
                serial_write(b"SWAP: fail\n");
                return;
            }
        };
        const VA: u64 = 0x0013_0000; // a user data VA (writable, NX)
        let frame = match as_get_page(ucr3, VA) {
            Some(f) => f,
            None => {
                address_space_release(ucr3);
                serial_write(b"SWAP: fail\n");
                return;
            }
        };
        // Write a pattern into the backing frame via the HHDM.
        let fp = phys_to_virt(frame) as *mut u8;
        let mut i = 0usize;
        while i < 4096 {
            *fp.add(i) = (i & 0xFF) as u8;
            i += 1;
        }
        // Run swap on the test address space.
        core::arch::asm!("mov cr3, {}", in(reg) ucr3, options(nostack));
        let evicted = swap_out_current(VA);
        let pte_swapped = leaf_pte_ptr(VA).map(|p| *p & 1 == 0 && *p & PTE_SWAPPED != 0);
        let restored = if evicted { try_swap_in(VA) } else { false };
        // Read back through the VA (now present); ring 0 may read a user page.
        let mut ok = evicted && pte_swapped == Some(true) && restored;
        if ok {
            let vp = VA as *const u8;
            let mut j = 0usize;
            while j < 4096 {
                if *vp.add(j) != (j & 0xFF) as u8 {
                    ok = false;
                    break;
                }
                j += 1;
            }
        }
        core::arch::asm!("mov cr3, {}", in(reg) kcr3, options(nostack));
        address_space_release(ucr3);
        if ok {
            serial_write(b"SWAP: roundtrip ok\n");
        } else {
            serial_write(b"SWAP: fail\n");
        }
    }
}
