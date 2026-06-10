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
