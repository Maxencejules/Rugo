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
// reallocates and copies. Mutations are write-through: the cached node
// table / bitmap sector is rewritten before the call returns. Crash
// journaling is a documented carry-forward.

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
    write_sector(
        BASE_SECTOR + 1 + s as u64,
        &VFS.nodes[s * 512..(s + 1) * 512],
    )
}

unsafe fn flush_bitmap() -> bool {
    write_sector(BITMAP_SECTOR, &VFS.bitmap)
}

unsafe fn flush_superblock() -> bool {
    let mut sb = [0u8; 16];
    sb[0..4].copy_from_slice(&VFS_MAGIC.to_le_bytes());
    sb[4..8].copy_from_slice(&2u32.to_le_bytes());
    let used = node_used() as u32;
    sb[8..12].copy_from_slice(&used.to_le_bytes());
    write_sector(BASE_SECTOR, &sb)
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
    set_node(idx, seg, parent, KIND_FILE, 0, 0);
    if !flush_node_sector(idx) || !flush_superblock() {
        return None;
    }
    Some(idx)
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
    set_node(idx, seg, parent, KIND_DIR, 0, 0);
    flush_node_sector(idx) && flush_superblock()
}

pub(crate) unsafe fn vfs_unlink(path: &[u8]) -> bool {
    if !VFS.ready {
        return false;
    }
    let Some((_, Some(idx), _)) = resolve(path) else {
        return false;
    };
    if node_kind(idx) == KIND_DIR {
        // Only empty directories can be removed.
        let mut i = 0;
        while i < MAX_NODES {
            if node_kind(i) != KIND_FREE && node_parent(i) == idx as u8 {
                return false;
            }
            i += 1;
        }
    } else {
        free_blocks(node_start(idx), blocks_for(node_size(idx)));
        if !flush_bitmap() {
            return false;
        }
    }
    set_node(idx, b"", 0, KIND_FREE, 0, 0);
    flush_node_sector(idx) && flush_superblock()
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
    if need_blocks > cur_blocks {
        // Grow by reallocating a fresh contiguous run and copying.
        let Some(new_start) = alloc_blocks(need_blocks) else {
            return 0;
        };
        let mut b = 0;
        while b < cur_blocks {
            let mut sec = [0u8; 512];
            if !read_sector(DATA_SECTOR + (start + b) as u64, &mut sec)
                || !write_sector(DATA_SECTOR + (new_start + b) as u64, &sec)
            {
                free_blocks(new_start, need_blocks);
                return 0;
            }
            b += 1;
        }
        free_blocks(start, cur_blocks);
        start = new_start;
        set_node_start(idx, start as u32);
        if !flush_bitmap() {
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
            return done;
        }
        if in_off != 0 || n < BLOCK_SIZE {
            // Preserve surrounding bytes in a partial block.
            let _ = read_sector(DATA_SECTOR + block as u64, &mut sec);
        }
        sec[in_off..in_off + n].copy_from_slice(&src[done..done + n]);
        if !write_sector(DATA_SECTOR + block as u64, &sec) {
            return done;
        }
        done += n;
    }
    if end > size {
        set_node_size(idx, end as u32);
    }
    if !flush_node_sector(idx) {
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
