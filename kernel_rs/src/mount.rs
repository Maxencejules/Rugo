// Mount table (full-os guide Part II.5): a path-prefix -> filesystem registry
// with longest-prefix matching on component boundaries. This generalizes the
// hardcoded /data, /tmp, /dev, /proc, /mnt routing in sys_open into a table a
// `mount` syscall could populate.
//
// v1 boundary: the registry + matching logic + a self-test. Re-routing the live
// sys_open path through it (so a new mount appears in the namespace without a code
// change) is carry-forward; the existing hardcoded routes still work today. This
// is the same mechanism-before-wiring staging the DMA pool / block cache use.

#![allow(dead_code)]

use crate::serial_write;

const MOUNT_MAX: usize = 8;
const PREFIX_MAX: usize = 24;

#[derive(Clone, Copy)]
struct Mount {
    prefix: [u8; PREFIX_MAX],
    prefix_len: usize,
    fs_type: u8,
    active: bool,
}

static mut MOUNTS: [Mount; MOUNT_MAX] = [Mount {
    prefix: [0; PREFIX_MAX],
    prefix_len: 0,
    fs_type: 0,
    active: false,
}; MOUNT_MAX];

unsafe fn mount_clear() {
    let mut i = 0;
    while i < MOUNT_MAX {
        MOUNTS[i].active = false;
        i += 1;
    }
}

/// Register `prefix` -> `fs_type`. Returns false if the table is full or the
/// prefix is too long.
unsafe fn mount_register(prefix: &[u8], fs_type: u8) -> bool {
    if prefix.is_empty() || prefix.len() > PREFIX_MAX {
        return false;
    }
    let mut i = 0;
    while i < MOUNT_MAX {
        if !MOUNTS[i].active {
            MOUNTS[i].prefix[..prefix.len()].copy_from_slice(prefix);
            MOUNTS[i].prefix_len = prefix.len();
            MOUNTS[i].fs_type = fs_type;
            MOUNTS[i].active = true;
            return true;
        }
        i += 1;
    }
    false
}

/// Longest-prefix match of `path` against the mount table, on component
/// boundaries (so "/data" matches "/data" and "/data/x" but NOT "/database").
/// Returns the matched mount's `fs_type`.
unsafe fn mount_lookup(path: &[u8]) -> Option<u8> {
    let mut best: Option<u8> = None;
    let mut best_len = 0usize;
    let mut i = 0;
    while i < MOUNT_MAX {
        if MOUNTS[i].active {
            let pl = MOUNTS[i].prefix_len;
            if pl <= path.len() && MOUNTS[i].prefix[..pl] == path[..pl] {
                // The prefix must end on a path-component boundary: either it is
                // the whole path, the next char is '/', or the prefix itself ends
                // in '/' (the root mount "/").
                let boundary =
                    pl == path.len() || path[pl] == b'/' || MOUNTS[i].prefix[pl - 1] == b'/';
                if boundary && (best.is_none() || pl > best_len) {
                    best = Some(MOUNTS[i].fs_type);
                    best_len = pl;
                }
            }
        }
        i += 1;
    }
    best
}

/// Mount-table self-test (full-os guide Part II.5): register overlapping mounts
/// and confirm longest-prefix, component-boundary matching, and root fallback.
/// Emits `MOUNT: table ok` / `fail`.
pub fn mount_selftest() -> u64 {
    unsafe {
        mount_clear();
        let reg = mount_register(b"/", 9)
            && mount_register(b"/data", 0)
            && mount_register(b"/mnt", 1)
            && mount_register(b"/data/special", 4);
        let ok = reg
            && mount_lookup(b"/data/file") == Some(0)      // under /data
            && mount_lookup(b"/mnt/HELLO.TXT") == Some(1)  // under /mnt
            && mount_lookup(b"/data/special/x") == Some(4) // nested: longest wins
            && mount_lookup(b"/other") == Some(9)          // falls back to root
            && mount_lookup(b"/database/x") == Some(9)     // NOT /data (boundary)
            && mount_lookup(b"/data") == Some(0);          // exact match
        mount_clear();
        if ok {
            serial_write(b"MOUNT: table ok\n");
            1
        } else {
            serial_write(b"MOUNT: table fail\n");
            0
        }
    }
}
