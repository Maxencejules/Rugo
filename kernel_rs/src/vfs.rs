// SimpleFS v2: a writable on-disk file tree with directories.
//
// Region layout on the boot disk (base sector 512, clear of the runtime
// state sectors 8..11 and leaving the exec app region at 64+ room to
// grow to its full 16-app table):
//   512:      superblock  magic "VFS2" u32 | version u32 | node_used u32
//   513..516: node table, 64 entries x 32 bytes:
//             name[20] | parent u8 (0xFE = root) | kind u8 (0 free,
//             1 file, 2 dir) | mode u8 (reserved) | pad | start_block u32
//             | size u32
//   517:      block allocation bitmap (1 bit per data block)
//   518..:    512-byte data blocks (block i lives at sector 518 + i)
//
// Files are contiguously allocated in v1; growth past the current run
// reallocates and copies. METADATA writes (node table, bitmap, superblock) are
// crash-consistent via a write-ahead journal (see the journal region below):
// vfs_write/vfs_mkdir/vfs_unlink wrap their metadata flushes in a transaction
// that is committed atomically and replayed at mount after a crash. Data-block
// writes stay write-through (ordered mode) -- a torn data write corrupts only a
// file's bytes, never the directory tree or allocation map.

#![allow(dead_code)]

use crate::{serial_write, serial_write_hex};

pub(crate) const VFS_MAGIC: u32 = 0x32534656; // "VFS2"
const BASE_SECTOR: u64 = 512;
const NODE_SECTORS: u64 = 4;
const BITMAP_SECTOR: u64 = BASE_SECTOR + 1 + NODE_SECTORS; // 517
const DATA_SECTOR: u64 = BITMAP_SECTOR + 1; // 518
pub(crate) const MAX_NODES: usize = 64;
const NODE_SIZE: usize = 32;
const NAME_MAX: usize = 20;
const BLOCK_SIZE: usize = 512;
const MAX_BLOCKS: usize = 1024; // 512 KiB of data on the 1 MiB boot disk
pub(crate) const ROOT: u8 = 0xFE;
pub(crate) const KIND_FREE: u8 = 0;
pub(crate) const KIND_FILE: u8 = 1;
pub(crate) const KIND_DIR: u8 = 2;
pub(crate) const FILE_MAX_BYTES: usize = 8192;

struct VfsState {
    ready: bool,
    nodes: [u8; MAX_NODES * NODE_SIZE],
    bitmap: [u8; BLOCK_SIZE],
}

static mut VFS: VfsState = VfsState {
    ready: false,
    nodes: [0; MAX_NODES * NODE_SIZE],
    bitmap: [0; BLOCK_SIZE],
};

unsafe fn read_sector(sector: u64, dst: &mut [u8]) -> bool {
    if !crate::block_io_dispatch(false, sector, 512, false) {
        return false;
    }
    dst[..512].copy_from_slice(&crate::BLK_DATA_PAGE.0[..512]);
    true
}

unsafe fn write_sector(sector: u64, src: &[u8]) -> bool {
    core::ptr::write_bytes(crate::BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
    crate::BLK_DATA_PAGE.0[..src.len().min(512)]
        .copy_from_slice(&src[..src.len().min(512)]);
    crate::block_io_dispatch(true, sector, 512, false)
}

unsafe fn flush_node_sector(node_idx: usize) -> bool {
    let per = 512 / NODE_SIZE;
    let s = node_idx / per;
    jwrite(
        BASE_SECTOR + 1 + s as u64,
        &VFS.nodes[s * 512..(s + 1) * 512],
    )
}

unsafe fn flush_bitmap() -> bool {
    jwrite(BITMAP_SECTOR, &VFS.bitmap)
}

unsafe fn flush_superblock() -> bool {
    let mut sb = [0u8; 16];
    sb[0..4].copy_from_slice(&VFS_MAGIC.to_le_bytes());
    sb[4..8].copy_from_slice(&2u32.to_le_bytes());
    let used = node_used() as u32;
    sb[8..12].copy_from_slice(&used.to_le_bytes());
    jwrite(BASE_SECTOR, &sb)
}

// ---- Write-ahead journal (full-os guide Part II.5, design #1) ----
// Crash-consistency for SimpleFS METADATA writes (the node table, bitmap, and
// superblock). A mutation opens a transaction; every metadata flush it makes is
// logged to the journal region (the 512-byte payload to a journal data slot, the
// home sector recorded in the header) instead of going straight home. txn_commit
// writes the header with `committed = 1` (the atomic commit point), applies the
// logged sectors to their homes, then clears the flag. A crash between commit and
// clear leaves the header committed; `replay_journal` at mount re-applies the
// logged sectors (idempotent), so the on-disk structure is never left with a
// half-finished multi-sector metadata update. Data-block writes stay write-through
// (ordered-mode style): a torn data write corrupts only that file's bytes, never
// the directory tree / allocation map, and journaling only metadata keeps the
// cached-write path free of stale-read hazards. Journal region is clear of the VFS
// data blocks (which end at sector 1541) and the standalone-journal/cache scratch.
const J_HEADER: u64 = 1543; // journal header sector
const J_DATA: u64 = J_HEADER + 1; // journal data slots (1544..)
const J_MAX: usize = 8; // max metadata sectors per transaction (txns touch <=3)
const J_MAGIC: u32 = 0x31304A56; // "VJ01"

struct JTxn {
    active: bool,
    overflow: bool,
    count: usize,
    targets: [u64; J_MAX],
    // Snapshot of the in-RAM metadata caches taken at txn_begin (before the mutation
    // touches them). The journal rolls back the on-DISK sectors on abort; these roll
    // back the RAM caches in lockstep, so an aborted mutation leaves VFS.nodes /
    // VFS.bitmap exactly matching the (untouched) disk -- no RAM/disk divergence.
    nodes_snap: [u8; MAX_NODES * NODE_SIZE],
    bitmap_snap: [u8; BLOCK_SIZE],
}

static mut JTXN: JTxn = JTxn {
    active: false,
    overflow: false,
    count: 0,
    targets: [0; J_MAX],
    nodes_snap: [0; MAX_NODES * NODE_SIZE],
    bitmap_snap: [0; BLOCK_SIZE],
};

/// Open a transaction. MUST be called BEFORE the mutation modifies VFS.nodes /
/// VFS.bitmap, so the snapshot captures the pristine pre-mutation cache.
unsafe fn txn_begin() {
    JTXN.active = true;
    JTXN.overflow = false;
    JTXN.count = 0;
    JTXN.nodes_snap.copy_from_slice(&VFS.nodes);
    JTXN.bitmap_snap.copy_from_slice(&VFS.bitmap);
}

/// Log one metadata sector write to the open transaction (payload -> a journal
/// slot, home sector recorded). With no transaction open, writes straight through
/// (so format-time / replay-time writes are unaffected).
unsafe fn jwrite(sector: u64, data: &[u8]) -> bool {
    if !JTXN.active {
        return write_sector(sector, data);
    }
    if JTXN.count >= J_MAX {
        JTXN.overflow = true;
        return false;
    }
    let slot = J_DATA + JTXN.count as u64;
    if !write_sector(slot, data) {
        JTXN.overflow = true;
        return false;
    }
    JTXN.targets[JTXN.count] = sector;
    JTXN.count += 1;
    true
}

/// Abort the open transaction: invalidate any journal header (so a stale committed
/// flag can't replay), drop the buffered writes, and ROLL BACK the in-RAM caches to
/// the txn_begin snapshot. Nothing was applied to the FS homes, so disk is exactly as
/// it was before txn_begin; restoring the caches keeps RAM in sync with it.
unsafe fn txn_abort() {
    let clr = [0u8; 512];
    let _ = write_sector(J_HEADER, &clr);
    VFS.nodes.copy_from_slice(&JTXN.nodes_snap);
    VFS.bitmap.copy_from_slice(&JTXN.bitmap_snap);
    JTXN.active = false;
    JTXN.count = 0;
    JTXN.overflow = false;
}

/// Commit the open transaction atomically: write the journal header with
/// `committed = 1` (THE commit point), apply every logged sector to its home, then
/// clear the header. Returns false (after a clean abort that applies nothing) if the
/// transaction overflowed or the header write failed.
unsafe fn txn_commit() -> bool {
    if !JTXN.active {
        return true;
    }
    if JTXN.overflow {
        txn_abort();
        return false;
    }
    let n = JTXN.count;
    let mut hdr = [0u8; 512];
    hdr[0..4].copy_from_slice(&J_MAGIC.to_le_bytes());
    hdr[4..8].copy_from_slice(&1u32.to_le_bytes()); // committed
    hdr[8..12].copy_from_slice(&(n as u32).to_le_bytes());
    let mut i = 0;
    while i < n {
        hdr[12 + i * 4..16 + i * 4].copy_from_slice(&(JTXN.targets[i] as u32).to_le_bytes());
        i += 1;
    }
    if !write_sector(J_HEADER, &hdr) {
        txn_abort();
        return false;
    }
    // Apply: copy each journal slot to its home sector.
    let mut ok = true;
    i = 0;
    while i < n {
        let mut buf = [0u8; 512];
        ok = ok
            && read_sector(J_DATA + i as u64, &mut buf)
            && write_sector(JTXN.targets[i], &buf);
        i += 1;
    }
    // Clear the committed flag ONLY if every apply succeeded. If an apply failed
    // partway (a device I/O error), leave the header committed so the next mount's
    // replay_journal re-drives the remaining (idempotent) applies -- erasing it here
    // would strand a half-applied metadata update with no recovery record. The RAM
    // caches already hold the intended state, which replay makes the disk match.
    if ok {
        let clr = [0u8; 512];
        let _ = write_sector(J_HEADER, &clr);
    }
    JTXN.active = false;
    ok
}

/// At mount: if the journal header is committed (a crash happened between the commit
/// point and the clear), re-apply the logged metadata sectors to their homes, then
/// clear the header. Idempotent. Returns the number of sectors replayed.
unsafe fn replay_journal() -> usize {
    let mut hdr = [0u8; 512];
    if !read_sector(J_HEADER, &mut hdr) {
        return 0;
    }
    let magic = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]);
    let committed = u32::from_le_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]);
    if magic != J_MAGIC || committed != 1 {
        return 0;
    }
    let mut n = u32::from_le_bytes([hdr[8], hdr[9], hdr[10], hdr[11]]) as usize;
    if n > J_MAX {
        n = J_MAX;
    }
    let mut i = 0;
    while i < n {
        let b = 12 + i * 4;
        let tgt = u32::from_le_bytes([hdr[b], hdr[b + 1], hdr[b + 2], hdr[b + 3]]) as u64;
        let mut buf = [0u8; 512];
        if read_sector(J_DATA + i as u64, &mut buf) {
            let _ = write_sector(tgt, &buf);
        }
        i += 1;
    }
    let clr = [0u8; 512];
    let _ = write_sector(J_HEADER, &clr);
    n
}

/// Crash-recovery self-test for the metadata journal (full-os guide Part II.5):
/// synthesize a committed journal (header + one data slot) targeting a scratch
/// sector -- exactly the on-disk state a crash between commit and clear leaves --
/// run `replay_journal` as a post-crash mount would, and confirm the scratch sector
/// received the journaled payload and the header was cleared. Proves the live FS
/// journal's replay path deterministically, independent of an actual crash. Leaves
/// the journal clean (header cleared) so it never replays spuriously afterwards.
pub(crate) unsafe fn vfs_journal_selftest() -> bool {
    const SCRATCH: u64 = 1560; // free, clear of VFS data + the journal + other scratch
    let pat = *b"VJTESTPATTERN-OK";
    let mut slot = [0u8; 512];
    slot[..pat.len()].copy_from_slice(&pat);
    if !write_sector(J_DATA, &slot) {
        return false;
    }
    let zero = [0u8; 512];
    let _ = write_sector(SCRATCH, &zero); // so a stale match can't pass
    let mut hdr = [0u8; 512];
    hdr[0..4].copy_from_slice(&J_MAGIC.to_le_bytes());
    hdr[4..8].copy_from_slice(&1u32.to_le_bytes()); // committed
    hdr[8..12].copy_from_slice(&1u32.to_le_bytes()); // count = 1
    hdr[12..16].copy_from_slice(&(SCRATCH as u32).to_le_bytes());
    if !write_sector(J_HEADER, &hdr) {
        return false;
    }
    let n = replay_journal();
    let mut got = [0u8; 512];
    let _ = read_sector(SCRATCH, &mut got);
    let mut hdr2 = [0u8; 512];
    let _ = read_sector(J_HEADER, &mut hdr2);
    let committed_after = u32::from_le_bytes([hdr2[4], hdr2[5], hdr2[6], hdr2[7]]);
    let ok = n == 1 && got[..pat.len()] == pat && committed_after == 0;
    if ok {
        serial_write(b"VFS: journal ok\n");
    } else {
        serial_write(b"VFS: journal FAIL\n");
    }
    ok
}

pub(crate) fn vfs_ready() -> bool {
    unsafe { VFS.ready }
}

unsafe fn node_used() -> usize {
    let mut used = 0;
    let mut i = 0;
    while i < MAX_NODES {
        if VFS.nodes[i * NODE_SIZE + 21] != KIND_FREE {
            used += 1;
        }
        i += 1;
    }
    used
}

pub(crate) unsafe fn node_kind(idx: usize) -> u8 {
    VFS.nodes[idx * NODE_SIZE + 21]
}

pub(crate) unsafe fn node_size(idx: usize) -> usize {
    let b = idx * NODE_SIZE + 28;
    u32::from_le_bytes([VFS.nodes[b], VFS.nodes[b + 1], VFS.nodes[b + 2], VFS.nodes[b + 3]])
        as usize
}

unsafe fn node_parent(idx: usize) -> u8 {
    VFS.nodes[idx * NODE_SIZE + 20]
}

// Permissions (gap item 10 / item 5 remainder): the node's mode byte
// (offset 22) holds owner/other rw bits; the pad byte (offset 23)
// holds the owner uid. mode 0 is legacy (pre-permission images) and
// reads as the default.
pub(crate) const MODE_OWNER_R: u8 = 0b0001;
pub(crate) const MODE_OWNER_W: u8 = 0b0010;
pub(crate) const MODE_OTHER_R: u8 = 0b0100;
pub(crate) const MODE_OTHER_W: u8 = 0b1000;
pub(crate) const MODE_DEFAULT: u8 = MODE_OWNER_R | MODE_OWNER_W | MODE_OTHER_R;

pub(crate) unsafe fn node_mode(idx: usize) -> u8 {
    let m = VFS.nodes[idx * NODE_SIZE + 22];
    if m == 0 { MODE_DEFAULT } else { m }
}

pub(crate) unsafe fn node_owner(idx: usize) -> u8 {
    VFS.nodes[idx * NODE_SIZE + 23]
}

pub(crate) unsafe fn set_node_owner(idx: usize, owner: u8) -> bool {
    VFS.nodes[idx * NODE_SIZE + 23] = owner;
    flush_node_sector(idx)
}

pub(crate) unsafe fn set_node_mode(idx: usize, mode: u8) -> bool {
    VFS.nodes[idx * NODE_SIZE + 22] = mode;
    flush_node_sector(idx)
}

/// Resolve a path to its node index without creating anything.
pub(crate) unsafe fn vfs_lookup(path: &[u8]) -> Option<usize> {
    vfs_open(path, false)
}

unsafe fn node_start(idx: usize) -> usize {
    let b = idx * NODE_SIZE + 24;
    u32::from_le_bytes([VFS.nodes[b], VFS.nodes[b + 1], VFS.nodes[b + 2], VFS.nodes[b + 3]])
        as usize
}

pub(crate) unsafe fn node_name(idx: usize) -> &'static [u8] {
    let base = idx * NODE_SIZE;
    let mut n = 0;
    while n < NAME_MAX && VFS.nodes[base + n] != 0 {
        n += 1;
    }
    core::slice::from_raw_parts(VFS.nodes.as_ptr().add(base), n)
}

unsafe fn set_node(idx: usize, name: &[u8], parent: u8, kind: u8, start: u32, size: u32) {
    let base = idx * NODE_SIZE;
    core::ptr::write_bytes(VFS.nodes.as_mut_ptr().add(base), 0, NODE_SIZE);
    VFS.nodes[base..base + name.len()].copy_from_slice(name);
    VFS.nodes[base + 20] = parent;
    VFS.nodes[base + 21] = kind;
    VFS.nodes[base + 24..base + 28].copy_from_slice(&start.to_le_bytes());
    VFS.nodes[base + 28..base + 32].copy_from_slice(&size.to_le_bytes());
}

unsafe fn set_node_size(idx: usize, size: u32) {
    let base = idx * NODE_SIZE;
    VFS.nodes[base + 28..base + 32].copy_from_slice(&size.to_le_bytes());
}

unsafe fn set_node_start(idx: usize, start: u32) {
    let base = idx * NODE_SIZE;
    VFS.nodes[base + 24..base + 28].copy_from_slice(&start.to_le_bytes());
}

/// Change a node's name in place, preserving its parent, kind, mode, owner,
/// start and size -- only the name field (bytes [0, NAME_MAX)) is rewritten.
/// Used by vfs_rename (set_node would zero mode/owner). (full-os Part II.5)
unsafe fn set_node_name(idx: usize, name: &[u8]) {
    let base = idx * NODE_SIZE;
    let mut i = 0;
    while i < NAME_MAX {
        VFS.nodes[base + i] = 0;
        i += 1;
    }
    let n = core::cmp::min(name.len(), NAME_MAX);
    VFS.nodes[base..base + n].copy_from_slice(&name[..n]);
}

/// Mount the region, formatting it in place when the magic is absent.
pub(crate) unsafe fn vfs_mount() {
    let mut sb = [0u8; 512];
    if !read_sector(BASE_SECTOR, &mut sb) {
        serial_write(b"VFS: io err\n");
        return;
    }
    let magic = u32::from_le_bytes([sb[0], sb[1], sb[2], sb[3]]);
    if magic != VFS_MAGIC {
        core::ptr::write_bytes(VFS.nodes.as_mut_ptr(), 0, VFS.nodes.len());
        core::ptr::write_bytes(VFS.bitmap.as_mut_ptr(), 0, VFS.bitmap.len());
        let mut s = 0u64;
        let mut ok = true;
        while s < NODE_SECTORS {
            ok = ok
                && write_sector(
                    BASE_SECTOR + 1 + s,
                    &VFS.nodes[(s as usize) * 512..(s as usize + 1) * 512],
                );
            s += 1;
        }
        ok = ok && flush_bitmap();
        VFS.ready = ok;
        ok = ok && flush_superblock();
        if ok {
            serial_write(b"VFS: format ok\n");
        } else {
            VFS.ready = false;
            serial_write(b"VFS: io err\n");
        }
        return;
    }
    // Recover any committed-but-unapplied metadata transaction BEFORE loading the
    // node table / bitmap, so we read a structurally-consistent FS (full-os II.5).
    let replayed = replay_journal();
    if replayed != 0 {
        serial_write(b"VFS: journal replay n=0x");
        serial_write_hex(replayed as u64);
        serial_write(b"\n");
    }
    let mut s = 0u64;
    while s < NODE_SECTORS {
        let mut sec = [0u8; 512];
        if !read_sector(BASE_SECTOR + 1 + s, &mut sec) {
            serial_write(b"VFS: io err\n");
            return;
        }
        VFS.nodes[(s as usize) * 512..(s as usize + 1) * 512].copy_from_slice(&sec);
        s += 1;
    }
    let mut bm = [0u8; 512];
    if !read_sector(BITMAP_SECTOR, &mut bm) {
        serial_write(b"VFS: io err\n");
        return;
    }
    VFS.bitmap.copy_from_slice(&bm);
    VFS.ready = true;
    serial_write(b"VFS: mount ok files=0x");
    serial_write_hex(node_used() as u64);
    serial_write(b"\n");
}

unsafe fn find_child(parent: u8, name: &[u8]) -> Option<usize> {
    if name.is_empty() || name.len() > NAME_MAX {
        return None;
    }
    let mut i = 0;
    while i < MAX_NODES {
        if node_kind(i) != KIND_FREE && node_parent(i) == parent {
            if node_name(i) == name {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

unsafe fn free_node_slot() -> Option<usize> {
    let mut i = 0;
    while i < MAX_NODES {
        if node_kind(i) == KIND_FREE {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Resolve a path below `/data` (the leading `/data` must already be
/// stripped). Returns (parent_index, Some(node)) when the leaf exists,
/// or (parent_index, None) when only the leaf is missing.
unsafe fn resolve(path: &[u8]) -> Option<(u8, Option<usize>, &[u8])> {
    let mut parent: u8 = ROOT;
    let mut rest = path;
    while !rest.is_empty() && rest[0] == b'/' {
        rest = &rest[1..];
    }
    if rest.is_empty() {
        return Some((ROOT, None, b""));
    }
    loop {
        let mut seg_end = 0;
        while seg_end < rest.len() && rest[seg_end] != b'/' {
            seg_end += 1;
        }
        let seg = &rest[..seg_end];
        if seg.is_empty() || seg.len() > NAME_MAX {
            return None;
        }
        let is_leaf = seg_end == rest.len()
            || rest[seg_end..].iter().all(|&c| c == b'/');
        if is_leaf {
            return Some((parent, find_child(parent, seg), seg));
        }
        match find_child(parent, seg) {
            Some(idx) if node_kind(idx) == KIND_DIR => {
                parent = idx as u8;
            }
            _ => return None,
        }
        rest = &rest[seg_end + 1..];
    }
}

unsafe fn alloc_blocks(count: usize) -> Option<usize> {
    if count == 0 {
        return Some(0);
    }
    let mut run = 0;
    let mut start = 0;
    let mut b = 0;
    while b < MAX_BLOCKS {
        let free = VFS.bitmap[b / 8] & (1 << (b % 8)) == 0;
        if free {
            if run == 0 {
                start = b;
            }
            run += 1;
            if run == count {
                let mut i = start;
                while i < start + count {
                    VFS.bitmap[i / 8] |= 1 << (i % 8);
                    i += 1;
                }
                return Some(start);
            }
        } else {
            run = 0;
        }
        b += 1;
    }
    None
}

unsafe fn free_blocks(start: usize, count: usize) {
    let mut i = start;
    while i < start + count && i < MAX_BLOCKS {
        VFS.bitmap[i / 8] &= !(1 << (i % 8));
        i += 1;
    }
}

/// Count the data blocks currently marked free in the bitmap (used by the
/// truncate self-test to prove the trailing blocks were released).
unsafe fn count_free_blocks() -> usize {
    let mut n = 0;
    let mut b = 0;
    while b < MAX_BLOCKS {
        if VFS.bitmap[b / 8] & (1 << (b % 8)) == 0 {
            n += 1;
        }
        b += 1;
    }
    n
}

fn blocks_for(bytes: usize) -> usize {
    (bytes + BLOCK_SIZE - 1) / BLOCK_SIZE
}

/// Look up an existing node; optionally create a missing file leaf.
pub(crate) unsafe fn vfs_open(path: &[u8], create: bool) -> Option<usize> {
    if !VFS.ready {
        return None;
    }
    let (parent, leaf, seg) = resolve(path)?;
    if seg.is_empty() {
        // Bare /data: the root directory itself (sentinel node index).
        return Some(ROOT as usize);
    }
    if let Some(idx) = leaf {
        return Some(idx);
    }
    if !create {
        return None;
    }
    let idx = free_node_slot()?;
    // Journal the new file's node-table + superblock entry atomically (begin before
    // set_node so the cache snapshot is pristine for rollback on abort).
    txn_begin();
    set_node(idx, seg, parent, KIND_FILE, 0, 0);
    if flush_node_sector(idx) && flush_superblock() {
        // txn_commit handles its own state (it aborts on overflow and leaves the
        // header committed for replay on an apply failure); don't abort after it.
        if txn_commit() {
            Some(idx)
        } else {
            None
        }
    } else {
        txn_abort();
        None
    }
}

pub(crate) unsafe fn vfs_mkdir(path: &[u8]) -> bool {
    if !VFS.ready {
        return false;
    }
    let Some((parent, leaf, seg)) = resolve(path) else {
        return false;
    };
    if leaf.is_some() || seg.is_empty() {
        return false;
    }
    let Some(idx) = free_node_slot() else {
        return false;
    };
    // Journal the node-table + superblock update atomically (full-os Part II.5).
    // txn_begin before set_node so the cache snapshot is pristine for rollback.
    txn_begin();
    set_node(idx, seg, parent, KIND_DIR, 0, 0);
    if flush_node_sector(idx) && flush_superblock() {
        txn_commit()
    } else {
        txn_abort();
        false
    }
}

pub(crate) unsafe fn vfs_unlink(path: &[u8]) -> bool {
    if !VFS.ready {
        return false;
    }
    let Some((_, Some(idx), _)) = resolve(path) else {
        return false;
    };
    let is_dir = node_kind(idx) == KIND_DIR;
    if is_dir {
        // Only empty directories can be removed (pure validation, before the txn).
        let mut i = 0;
        while i < MAX_NODES {
            if node_kind(i) != KIND_FREE && node_parent(i) == idx as u8 {
                return false;
            }
            i += 1;
        }
    }
    // Journal the bitmap (freed blocks) + node-table + superblock atomically, so a
    // crash never leaves the node freed while the bitmap still marks its blocks used
    // (or vice versa). (full-os guide Part II.5.)
    txn_begin();
    if !is_dir {
        free_blocks(node_start(idx), blocks_for(node_size(idx)));
        if !flush_bitmap() {
            txn_abort();
            return false;
        }
    }
    set_node(idx, b"", 0, KIND_FREE, 0, 0);
    if flush_node_sector(idx) && flush_superblock() {
        txn_commit()
    } else {
        txn_abort();
        false
    }
}

/// Rename a file or directory to a new name within the SAME parent directory
/// (full-os guide Part II.5): the node keeps its kind, mode, owner, data blocks
/// and size -- only its name changes. Fails if the source is missing, or the new
/// name is empty / too long / slash-bearing / already names a sibling. The
/// node-table write is journaled atomically with the superblock, exactly like
/// vfs_mkdir/vfs_unlink, so a crash never leaves a half-renamed entry.
pub(crate) unsafe fn vfs_rename(path: &[u8], new_name: &[u8]) -> bool {
    if !VFS.ready || new_name.is_empty() || new_name.len() > NAME_MAX {
        return false;
    }
    if new_name.iter().any(|&c| c == b'/') {
        return false; // a leaf name only -- rename does not move across directories
    }
    let Some((parent, Some(idx), _)) = resolve(path) else {
        return false;
    };
    if find_child(parent, new_name).is_some() {
        return false; // the target name is already used in this directory
    }
    // txn_begin before the mutation so the rollback snapshot is pristine.
    txn_begin();
    set_node_name(idx, new_name);
    if flush_node_sector(idx) && flush_superblock() {
        txn_commit()
    } else {
        txn_abort();
        false
    }
}

/// Boot self-test for rename (full-os guide Part II.5): create a scratch file,
/// write a marker, rename it within /data, and confirm the NEW name resolves to
/// the SAME node with the content intact while the OLD name is gone -- then remove
/// it, so the on-disk FS is left exactly as found (net-neutral: the scratch node
/// and its block are freed, so the persisted-VFS tests that reuse the disk are
/// unaffected). The scratch names are used by nothing else.
pub(crate) unsafe fn vfs_rename_selftest() -> bool {
    if !VFS.ready {
        return false;
    }
    // Defensive: clear any residue from a crashed prior boot.
    let _ = vfs_unlink(b"/.rnsrc");
    let _ = vfs_unlink(b"/.rndst");
    let payload = b"rename-payload-42";
    let pass = (|| -> Option<()> {
        let src = vfs_open(b"/.rnsrc", true)?; // create the scratch file
        if vfs_write(src, 0, payload) != payload.len() {
            return None;
        }
        if !vfs_rename(b"/.rnsrc", b".rndst") {
            return None;
        }
        if vfs_lookup(b"/.rnsrc").is_some() {
            return None; // the old name must no longer resolve
        }
        let dst = vfs_lookup(b"/.rndst")?; // the new name resolves...
        if dst != src {
            return None; // ...to the very same underlying node
        }
        let mut buf = [0u8; 32];
        let n = vfs_read(dst, 0, &mut buf);
        if &buf[..n] != payload {
            return None; // content preserved across the rename
        }
        Some(())
    })();
    // Always clean up so the FS is byte-equivalent to before (net-neutral).
    let _ = vfs_unlink(b"/.rndst");
    let _ = vfs_unlink(b"/.rnsrc");
    if pass.is_some() {
        serial_write(b"VFS: rename ok\n");
        true
    } else {
        serial_write(b"VFS: rename fail\n");
        false
    }
}

/// Truncate a file to `new_size` bytes (full-os guide Part II.5): SHRINK only --
/// release the trailing data blocks no longer covered and update the size. A grow
/// request (new_size > current) is rejected in v1 (a write is what extends a file).
/// The freed bitmap + the node entry are journaled atomically (so a crash never
/// leaves a block freed while the node still claims it, or vice versa). Fails on a
/// missing path, a directory, or a grow.
pub(crate) unsafe fn vfs_truncate(path: &[u8], new_size: usize) -> bool {
    if !VFS.ready {
        return false;
    }
    let Some((_, Some(idx), _)) = resolve(path) else {
        return false;
    };
    if node_kind(idx) != KIND_FILE {
        return false;
    }
    let old_size = node_size(idx);
    if new_size > old_size {
        return false; // shrink-only in v1
    }
    if new_size == old_size {
        return true; // nothing to do
    }
    let start = node_start(idx);
    let old_blocks = blocks_for(old_size);
    let new_blocks = blocks_for(new_size);
    txn_begin();
    if new_blocks < old_blocks {
        free_blocks(start + new_blocks, old_blocks - new_blocks);
        if !flush_bitmap() {
            txn_abort();
            return false;
        }
    }
    set_node_size(idx, new_size as u32);
    if flush_node_sector(idx) && flush_superblock() {
        txn_commit()
    } else {
        txn_abort();
        false
    }
}

/// Boot self-test for truncate (full-os guide Part II.5): create a 3-block scratch
/// file with a per-byte pattern, shrink it to one block, and confirm the size, the
/// surviving first block's bytes, that reads past the new size are empty, AND that
/// exactly the two trailing blocks were returned to the free bitmap -- then remove
/// it, leaving the FS net-neutral (safe on the reused persisted-VFS disk).
pub(crate) unsafe fn vfs_truncate_selftest() -> bool {
    if !VFS.ready {
        return false;
    }
    let _ = vfs_unlink(b"/.trunc");
    let pass = (|| -> Option<()> {
        let idx = vfs_open(b"/.trunc", true)?;
        let mut data = [0u8; 1536]; // 3 blocks
        let mut i = 0;
        while i < data.len() {
            data[i] = (i & 0xFF) as u8;
            i += 1;
        }
        if vfs_write(idx, 0, &data) != data.len() || node_size(idx) != 1536 {
            return None;
        }
        let free_before = count_free_blocks();
        if !vfs_truncate(b"/.trunc", 512) || node_size(idx) != 512 {
            return None;
        }
        // blocks_for(1536)=3 -> blocks_for(512)=1, so exactly 2 blocks freed.
        if count_free_blocks() != free_before + 2 {
            return None;
        }
        let mut buf = [0u8; 600];
        if vfs_read(idx, 0, &mut buf) != 512 {
            return None;
        }
        let mut k = 0;
        while k < 512 {
            if buf[k] != (k & 0xFF) as u8 {
                return None; // surviving first block content preserved
            }
            k += 1;
        }
        if vfs_read(idx, 512, &mut buf) != 0 {
            return None; // the truncated-away region reads empty
        }
        Some(())
    })();
    let _ = vfs_unlink(b"/.trunc");
    if pass.is_some() {
        serial_write(b"VFS: truncate ok\n");
        true
    } else {
        serial_write(b"VFS: truncate fail\n");
        false
    }
}

/// stat: returns (kind, size).
pub(crate) unsafe fn vfs_stat(path: &[u8]) -> Option<(u8, usize)> {
    if !VFS.ready {
        return None;
    }
    let (_, leaf, _) = resolve(path)?;
    let idx = leaf?;
    Some((node_kind(idx), node_size(idx)))
}

pub(crate) unsafe fn vfs_read(idx: usize, offset: usize, dst: &mut [u8]) -> usize {
    if !VFS.ready || node_kind(idx) != KIND_FILE {
        return 0;
    }
    let size = node_size(idx);
    if offset >= size {
        return 0;
    }
    let want = dst.len().min(size - offset);
    let start = node_start(idx);
    let mut done = 0;
    while done < want {
        let pos = offset + done;
        let block = start + pos / BLOCK_SIZE;
        let mut sec = [0u8; 512];
        if !read_sector(DATA_SECTOR + block as u64, &mut sec) {
            return done;
        }
        let in_off = pos % BLOCK_SIZE;
        let n = (BLOCK_SIZE - in_off).min(want - done);
        dst[done..done + n].copy_from_slice(&sec[in_off..in_off + n]);
        done += n;
    }
    done
}

pub(crate) unsafe fn vfs_write(idx: usize, offset: usize, src: &[u8]) -> usize {
    if !VFS.ready || node_kind(idx) != KIND_FILE {
        return 0;
    }
    let size = node_size(idx);
    if offset > size {
        return 0;
    }
    let end = offset + src.len();
    if end > FILE_MAX_BYTES {
        return 0;
    }
    let cur_blocks = blocks_for(size);
    let need_blocks = blocks_for(end);
    let mut start = node_start(idx);
    // Journal this write's METADATA updates (the bitmap on a realloc + the node
    // entry) atomically; the data blocks below stay write-through (ordered mode).
    // (full-os guide Part II.5.) Every early exit after this aborts the txn, leaving
    // the on-disk structure exactly as it was.
    txn_begin();
    if need_blocks > cur_blocks {
        // Grow by reallocating a fresh contiguous run and copying.
        let Some(new_start) = alloc_blocks(need_blocks) else {
            txn_abort();
            return 0;
        };
        let mut b = 0;
        while b < cur_blocks {
            let mut sec = [0u8; 512];
            if !read_sector(DATA_SECTOR + (start + b) as u64, &mut sec)
                || !write_sector(DATA_SECTOR + (new_start + b) as u64, &sec)
            {
                free_blocks(new_start, need_blocks);
                txn_abort();
                return 0;
            }
            b += 1;
        }
        free_blocks(start, cur_blocks);
        start = new_start;
        set_node_start(idx, start as u32);
        if !flush_bitmap() {
            txn_abort();
            return 0;
        }
    }
    let mut done = 0;
    while done < src.len() {
        let pos = offset + done;
        let block = start + pos / BLOCK_SIZE;
        let in_off = pos % BLOCK_SIZE;
        let n = (BLOCK_SIZE - in_off).min(src.len() - done);
        let mut sec = [0u8; 512];
        if (in_off != 0 || n < BLOCK_SIZE)
            && pos < size
            && !read_sector(DATA_SECTOR + block as u64, &mut sec)
        {
            txn_abort();
            return done;
        }
        if in_off != 0 || n < BLOCK_SIZE {
            // Preserve surrounding bytes in a partial block.
            let _ = read_sector(DATA_SECTOR + block as u64, &mut sec);
        }
        sec[in_off..in_off + n].copy_from_slice(&src[done..done + n]);
        if !write_sector(DATA_SECTOR + block as u64, &sec) {
            txn_abort();
            return done;
        }
        done += n;
    }
    if end > size {
        set_node_size(idx, end as u32);
    }
    if !flush_node_sector(idx) {
        txn_abort();
        return 0;
    }
    if !txn_commit() {
        return 0;
    }
    done
}

/// Pack dirent records for the children of `dir` starting at child index
/// `cursor`. Record: name[24] | kind u8 | pad[3] | size u32 = 32 bytes.
/// Returns (bytes_written, next_cursor).
pub(crate) unsafe fn vfs_readdir(
    dir: usize,
    cursor: usize,
    dst: &mut [u8],
) -> (usize, usize) {
    if !VFS.ready {
        return (0, cursor);
    }
    let parent = if dir == ROOT as usize { ROOT } else { dir as u8 };
    let mut written = 0;
    let mut idx = cursor;
    while idx < MAX_NODES && written + 32 <= dst.len() {
        if node_kind(idx) != KIND_FREE && node_parent(idx) == parent {
            let name = node_name(idx);
            let rec = &mut dst[written..written + 32];
            core::ptr::write_bytes(rec.as_mut_ptr(), 0, 32);
            rec[..name.len()].copy_from_slice(name);
            rec[24] = node_kind(idx);
            let size = node_size(idx) as u32;
            rec[28..32].copy_from_slice(&size.to_le_bytes());
            written += 32;
        }
        idx += 1;
    }
    (written, idx)
}
