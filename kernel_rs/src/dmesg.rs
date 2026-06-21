//! dmesg ring buffer (full-os guide Part V.11 / IV.10): a heap-free fixed ring
//! that captures every `serial_write` line. Oldest bytes are overwritten once
//! full; reads return the most recent `len` bytes in oldest->newest order, which
//! userspace queries via `sys_sysinfo` op 4.
//!
//! Extracted from `lib.rs` (gap #9, maintainability). `serial_write` mirrors into
//! this ring via `crate::dmesg::klog_append`; the module depends only on
//! `crate::memory::copyout_user`.

use crate::memory::copyout_user;

const KLOG_CAP: usize = 8192;
static mut KLOG: [u8; KLOG_CAP] = [0; KLOG_CAP];
static mut KLOG_HEAD: usize = 0;
static mut KLOG_LEN: usize = 0;

/// Append `s` to the dmesg ring (mirrored from `serial_write`).
pub(crate) unsafe fn klog_append(s: &[u8]) {
    for &b in s {
        KLOG[KLOG_HEAD] = b;
        KLOG_HEAD = (KLOG_HEAD + 1) % KLOG_CAP;
        if KLOG_LEN < KLOG_CAP {
            KLOG_LEN += 1;
        }
    }
}

/// Copy the most recent `min(len, valid)` bytes of the dmesg ring into the user
/// buffer at `ptr`, in oldest->newest order. Returns the count copied, or
/// u64::MAX if the user buffer is unwritable.
pub(crate) unsafe fn klog_read(ptr: u64, len: usize) -> u64 {
    let n = core::cmp::min(len, KLOG_LEN);
    if n == 0 {
        return 0;
    }
    let start = (KLOG_HEAD + KLOG_CAP - n) % KLOG_CAP;
    let first = core::cmp::min(n, KLOG_CAP - start);
    if copyout_user(ptr, &KLOG[start..start + first], first).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if n > first
        && copyout_user(ptr + first as u64, &KLOG[0..n - first], n - first).is_err()
    {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    n as u64
}
