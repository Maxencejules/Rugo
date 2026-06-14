// DMA allocator (full-os guide Part II.7, driver model): a small pool of
// physically-contiguous frames carved from the PMM at boot and handed to drivers
// as PHYSICAL addresses (devices DMA to physical memory, not the kernel's HHDM
// VAs). Bitmap-managed: dma_alloc finds a contiguous run of free pages, dma_free
// releases it. This is the allocation primitive a virtio/NVMe/e1000 driver uses
// for its descriptor rings and bounce buffers.
//
// v1 boundary: a single fixed-size pool, first-fit, no fragmentation compaction,
// no per-device IOMMU domains. Go lane only (it reserves 1 MiB at boot, which
// would perturb the free-frame accounting other lanes assert).

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, Ordering};

use crate::{serial_write, serial_write_hex};

const DMA_POOL_PAGES: usize = 256; // 1 MiB pool (256 * 4 KiB)
const PAGE: u64 = 4096;

static mut DMA_BASE: u64 = 0; // physical base of the contiguous pool
static mut DMA_USED: [bool; DMA_POOL_PAGES] = [false; DMA_POOL_PAGES];
static DMA_READY: AtomicBool = AtomicBool::new(false);

/// Reserve the DMA pool from the PMM (one contiguous physical run). Idempotent.
pub fn dma_init() {
    unsafe {
        if DMA_READY.load(Ordering::Acquire) {
            return;
        }
        match crate::mm::alloc_frames_contig(DMA_POOL_PAGES) {
            Some(base) => {
                DMA_BASE = base;
                DMA_READY.store(true, Ordering::Release);
                serial_write(b"DMA: pool base=0x");
                serial_write_hex(base);
                serial_write(b" pages=0x");
                serial_write_hex(DMA_POOL_PAGES as u64);
                serial_write(b"\n");
            }
            None => serial_write(b"DMA: pool none\n"),
        }
    }
}

/// Allocate `pages` physically-contiguous, page-aligned frames from the pool.
/// Returns the physical base address, or None if the pool lacks a contiguous run.
pub unsafe fn dma_alloc(pages: usize) -> Option<u64> {
    if !DMA_READY.load(Ordering::Acquire) || pages == 0 || pages > DMA_POOL_PAGES {
        return None;
    }
    // First-fit: scan for `pages` consecutive free slots.
    let mut run = 0usize;
    let mut i = 0usize;
    while i < DMA_POOL_PAGES {
        if !DMA_USED[i] {
            run += 1;
            if run == pages {
                let start = i + 1 - pages;
                let mut k = start;
                while k < start + pages {
                    DMA_USED[k] = true;
                    k += 1;
                }
                return Some(DMA_BASE + (start as u64) * PAGE);
            }
        } else {
            run = 0;
        }
        i += 1;
    }
    None
}

/// Release a region previously returned by `dma_alloc`. `phys`/`pages` must match
/// an allocation; out-of-range frees are ignored.
pub unsafe fn dma_free(phys: u64, pages: usize) {
    if !DMA_READY.load(Ordering::Acquire) || phys < DMA_BASE {
        return;
    }
    let off = phys - DMA_BASE;
    if off % PAGE != 0 {
        return;
    }
    let start = (off / PAGE) as usize;
    if start >= DMA_POOL_PAGES {
        return;
    }
    let mut k = start;
    while k < start + pages && k < DMA_POOL_PAGES {
        DMA_USED[k] = false;
        k += 1;
    }
}

/// Boot self-test: two allocations are page-aligned, non-overlapping, contiguous,
/// and pool-contained; after freeing the first, a same-size re-alloc reuses its
/// exact base (first-fit), proving the bitmap accounts the region correctly.
pub fn dma_selftest() {
    unsafe {
        if !DMA_READY.load(Ordering::Acquire) {
            serial_write(b"DMA: selftest skip\n");
            return;
        }
        let a = dma_alloc(4);
        let b = dma_alloc(2);
        let ok = match (a, b) {
            (Some(pa), Some(pb)) => {
                let aligned = pa % PAGE == 0 && pb % PAGE == 0;
                let nonoverlap = pb >= pa + 4 * PAGE || pa >= pb + 2 * PAGE;
                let contained = pa >= DMA_BASE
                    && pb + 2 * PAGE <= DMA_BASE + (DMA_POOL_PAGES as u64) * PAGE;
                dma_free(pa, 4);
                let reused = dma_alloc(4) == Some(pa); // first-fit reclaims pa
                aligned && nonoverlap && contained && reused
            }
            _ => false,
        };
        if ok {
            serial_write(b"DMA: selftest ok\n");
        } else {
            serial_write(b"DMA: selftest fail\n");
        }
    }
}
