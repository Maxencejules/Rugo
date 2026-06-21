//! Security audit log (full-os guide Part IV.10): a small ring, distinct from
//! dmesg, holding structured security events (capability/sandbox denials and
//! privileged operations) that userspace can query via `sys_sysinfo` op 7.
//! Records WHO (tid) tried WHAT (syscall nr) and was denied.
//!
//! Extracted from `lib.rs` (gap #9, maintainability). Depends only on
//! `crate::R4_CURRENT` (the caller tid, read via descendant access) and
//! `crate::memory::copyout_user`.

use crate::memory::copyout_user;

const AUDIT_CAP: usize = 1024;
static mut AUDIT: [u8; AUDIT_CAP] = [0; AUDIT_CAP];
static mut AUDIT_HEAD: usize = 0;
static mut AUDIT_LEN: usize = 0;

unsafe fn audit_byte(b: u8) {
    AUDIT[AUDIT_HEAD] = b;
    AUDIT_HEAD = (AUDIT_HEAD + 1) % AUDIT_CAP;
    if AUDIT_LEN < AUDIT_CAP {
        AUDIT_LEN += 1;
    }
}

unsafe fn audit_write(s: &[u8]) {
    for &b in s {
        audit_byte(b);
    }
}

unsafe fn audit_hex(v: u64) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut shift: i32 = 60;
    while shift >= 0 {
        audit_byte(HEX[((v >> shift) & 0xF) as usize]);
        shift -= 4;
    }
}

/// Record a denied/privileged security event: tag + syscall nr + caller tid.
pub(crate) unsafe fn audit_event(tag: &[u8], nr: u64) {
    audit_write(b"AUDIT: ");
    audit_write(tag);
    audit_write(b" nr=0x");
    audit_hex(nr);
    audit_write(b" tid=0x");
    audit_hex(crate::R4_CURRENT as u64);
    audit_write(b"\n");
}

/// Copy the most recent `len` bytes of the audit ring (oldest->newest) to the
/// user buffer; returns the count or u64::MAX on a bad buffer.
pub(crate) unsafe fn audit_read(ptr: u64, len: usize) -> u64 {
    let n = core::cmp::min(len, AUDIT_LEN);
    if n == 0 {
        return 0;
    }
    let start = (AUDIT_HEAD + AUDIT_CAP - n) % AUDIT_CAP;
    let first = core::cmp::min(n, AUDIT_CAP - start);
    if copyout_user(ptr, &AUDIT[start..start + first], first).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if n > first
        && copyout_user(ptr + first as u64, &AUDIT[0..n - first], n - first).is_err()
    {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    n as u64
}

/// Boot self-test for the security audit checkpoints (full-os guide Part IV.10).
/// The sandbox-deny gate (syscall.rs) and the power path (sys_power) now record
/// an audit event; this drives the same `audit_event` calls those sites make and
/// confirms each lands in the ring with the expected tag + syscall nr, read back
/// exactly the bytes appended (the way sys_sysinfo op 7 exposes them to userspace).
pub(crate) unsafe fn audit_checkpoint_selftest() {
    let h0 = AUDIT_HEAD;
    audit_event(b"sandbox-deny", 0x31); // nr 49 (sys_net_query): a filtered syscall
    audit_event(b"power-off", 58);
    let h1 = AUDIT_HEAD;
    // Linearize exactly the bytes the two calls appended ([h0, h1) over the ring).
    let mut tmp = [0u8; 256];
    let mut k = 0usize;
    let mut i = h0;
    while i != h1 && k < tmp.len() {
        tmp[k] = AUDIT[i];
        i = (i + 1) % AUDIT_CAP;
        k += 1;
    }
    let buf = &tmp[..k];
    let ok = crate::slice_contains(buf, b"sandbox-deny nr=0x0000000000000031")
        && crate::slice_contains(buf, b"power-off nr=0x000000000000003A");
    if ok {
        crate::serial_write(b"AUDIT: checkpoints ok\n");
    } else {
        crate::serial_write(b"AUDIT: checkpoints fail\n");
    }
}
