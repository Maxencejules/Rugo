// Block buffer cache (full-os guide Part II.5, FS maturity): a small write-back
// LRU over the 512-byte sector I/O path. `cache_read` serves repeated reads from
// RAM; `cache_write` updates the cached sector and marks it dirty, deferring the
// disk write until the line is evicted (flush-on-evict). This is the layer the
// guide places under vfs_read/vfs_write to halve repeated I/O and batch writes.
//
// v1 boundary: a 4-entry fully-associative cache exercised by a self-test; it is
// NOT yet spliced into the live VFS path (that re-routing is carry-forward, like
// the DMA pool). All I/O goes through the shared BLK_DATA_PAGE, so the cache
// keeps its own per-line buffers and copies through it for each disk access.

#![allow(dead_code)]

use crate::{block_io_dispatch, serial_write, storage, BLK_DATA_PAGE};

const CACHE_ENTRIES: usize = 4;
const SECTOR: usize = 512;

#[derive(Clone, Copy)]
struct CacheLine {
    lba: u64,
    valid: bool,
    dirty: bool,
    age: u64, // last-use clock value; lowest = least recently used
    data: [u8; SECTOR],
}

impl CacheLine {
    const EMPTY: Self = Self {
        lba: 0,
        valid: false,
        dirty: false,
        age: 0,
        data: [0u8; SECTOR],
    };
}

static mut CACHE: [CacheLine; CACHE_ENTRIES] = [CacheLine::EMPTY; CACHE_ENTRIES];
static mut CACHE_CLOCK: u64 = 0;
static mut CACHE_HITS: u64 = 0;
static mut CACHE_MISSES: u64 = 0;

unsafe fn tick() -> u64 {
    CACHE_CLOCK += 1;
    CACHE_CLOCK
}

/// Read a sector straight from disk into `buf` (bypasses the cache).
unsafe fn disk_read(lba: u64, buf: &mut [u8; SECTOR]) -> bool {
    if !block_io_dispatch(false, lba, 512, false) {
        return false;
    }
    buf.copy_from_slice(&BLK_DATA_PAGE.0[..SECTOR]);
    true
}

/// Write a sector straight to disk from `buf` (bypasses the cache).
unsafe fn disk_write(lba: u64, buf: &[u8; SECTOR]) -> bool {
    BLK_DATA_PAGE.0[..SECTOR].copy_from_slice(buf);
    block_io_dispatch(true, lba, 512, false)
}

fn find(lba: u64) -> Option<usize> {
    unsafe {
        let mut i = 0;
        while i < CACHE_ENTRIES {
            if CACHE[i].valid && CACHE[i].lba == lba {
                return Some(i);
            }
            i += 1;
        }
        None
    }
}

/// Pick a line to (re)use: a free line if one exists, else the least-recently
/// used valid line — flushing it to disk first when dirty (flush-on-evict).
unsafe fn evict() -> Option<usize> {
    let mut i = 0;
    while i < CACHE_ENTRIES {
        if !CACHE[i].valid {
            return Some(i);
        }
        i += 1;
    }
    let mut victim = 0usize;
    let mut best = u64::MAX;
    i = 0;
    while i < CACHE_ENTRIES {
        if CACHE[i].age < best {
            best = CACHE[i].age;
            victim = i;
        }
        i += 1;
    }
    if CACHE[victim].dirty {
        let lba = CACHE[victim].lba;
        let buf = CACHE[victim].data;
        if !disk_write(lba, &buf) {
            return None;
        }
        CACHE[victim].dirty = false;
    }
    Some(victim)
}

/// Read sector `lba` through the cache into `out`. A hit copies from RAM; a miss
/// evicts the LRU line (flushing it if dirty) and loads from disk.
pub unsafe fn cache_read(lba: u64, out: &mut [u8; SECTOR]) -> bool {
    if let Some(i) = find(lba) {
        out.copy_from_slice(&CACHE[i].data);
        CACHE[i].age = tick();
        CACHE_HITS += 1;
        return true;
    }
    CACHE_MISSES += 1;
    let i = match evict() {
        Some(i) => i,
        None => return false,
    };
    let mut buf = [0u8; SECTOR];
    if !disk_read(lba, &mut buf) {
        return false;
    }
    CACHE[i] = CacheLine { lba, valid: true, dirty: false, age: tick(), data: buf };
    out.copy_from_slice(&buf);
    true
}

/// Write sector `lba` through the cache (write-back): update/install the line and
/// mark it dirty; the disk write is deferred until the line is evicted.
pub unsafe fn cache_write(lba: u64, buf: &[u8; SECTOR]) -> bool {
    if let Some(i) = find(lba) {
        CACHE[i].data.copy_from_slice(buf);
        CACHE[i].dirty = true;
        CACHE[i].age = tick();
        return true;
    }
    let i = match evict() {
        Some(i) => i,
        None => return false,
    };
    CACHE[i] = CacheLine { lba, valid: true, dirty: true, age: tick(), data: *buf };
    true
}

/// Flush every dirty line to disk (e.g. before unmount). Returns false on I/O
/// failure.
pub unsafe fn cache_flush_all() -> bool {
    let mut i = 0;
    while i < CACHE_ENTRIES {
        if CACHE[i].valid && CACHE[i].dirty {
            let lba = CACHE[i].lba;
            let buf = CACHE[i].data;
            if !disk_write(lba, &buf) {
                return false;
            }
            CACHE[i].dirty = false;
        }
        i += 1;
    }
    true
}

/// Reset the cache to empty (used by the self-test).
unsafe fn cache_reset() {
    let mut i = 0;
    while i < CACHE_ENTRIES {
        CACHE[i] = CacheLine::EMPTY;
        i += 1;
    }
    CACHE_CLOCK = 0;
    CACHE_HITS = 0;
    CACHE_MISSES = 0;
}

/// Block-cache self-test: proves write-back deferral, flush-on-evict, LRU
/// eviction, and read-hit caching against scratch sectors. Emits
/// `CACHE: selftest ok` / `fail` (or `skip` with no disk). Returns 1 on success.
pub fn cache_selftest() -> u64 {
    unsafe {
        if !storage::r4_storage_available() {
            serial_write(b"CACHE: selftest skip\n");
            return 0;
        }
        cache_reset();
        const BASE: u64 = 1610; // free scratch region (SimpleFS ends ~1542)
        // Warm-up read: the first virtio-blk op after a device (re)init can
        // return a stale/empty buffer; discard one before the real sequence.
        let mut warm = [0u8; SECTOR];
        let _ = disk_read(BASE, &mut warm);
        let old = [0xAAu8; SECTOR];
        let new = [0xBBu8; SECTOR];

        // Seed the disk with OLD directly, then write NEW through the cache.
        if !disk_write(BASE, &old) {
            serial_write(b"CACHE: selftest fail\n");
            return 0;
        }
        if !cache_write(BASE, &new) {
            serial_write(b"CACHE: selftest fail\n");
            return 0;
        }
        // Write-back: the disk must still hold OLD (the write was deferred).
        let mut probe = [0u8; SECTOR];
        if !disk_read(BASE, &mut probe) || probe[0] != 0xAA {
            serial_write(b"CACHE: selftest fail\n");
            return 0;
        }
        // Touch 4 other sectors to evict BASE (the LRU line) -> flush NEW to disk.
        let mut tmp = [0u8; SECTOR];
        let mut k = 0u64;
        while k < CACHE_ENTRIES as u64 {
            if !cache_read(BASE + 1 + k, &mut tmp) {
                serial_write(b"CACHE: selftest fail\n");
                return 0;
            }
            k += 1;
        }
        // Flush-on-evict: the disk must now hold NEW.
        if !disk_read(BASE, &mut probe) || probe[0] != 0xBB {
            serial_write(b"CACHE: selftest fail\n");
            return 0;
        }
        // Read-hit caching: a fresh read is a miss; the immediate repeat is a hit.
        let miss_before = CACHE_MISSES;
        let hit_before = CACHE_HITS;
        if !cache_read(BASE, &mut probe) || probe[0] != 0xBB {
            serial_write(b"CACHE: selftest fail\n");
            return 0;
        }
        if !cache_read(BASE, &mut probe) {
            serial_write(b"CACHE: selftest fail\n");
            return 0;
        }
        if CACHE_MISSES != miss_before + 1 || CACHE_HITS != hit_before + 1 {
            serial_write(b"CACHE: selftest fail\n");
            return 0;
        }
        serial_write(b"CACHE: selftest ok\n");
        1
    }
}
